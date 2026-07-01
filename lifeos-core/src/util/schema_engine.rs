//! Schema Engine — fetches and caches Notion database schemas
//!
//! Provides two layers:
//! - `SchemaEngine`: per-data-source schema caching (original)
//! - `SchemaCache`: config-aware caching with reservoir → satellite hierarchy

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::NotionDataSource;

/// Schema engine with raw data-source caching
pub struct SchemaEngine {
    notion: Arc<NotionClient>,
    schema_cache: Arc<Mutex<HashMap<String, NotionDataSource>>>,
}

impl SchemaEngine {
    pub fn new(notion: Arc<NotionClient>) -> Self {
        Self {
            notion,
            schema_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_schema(&self, data_source_id: &str) -> Result<NotionDataSource, String> {
        {
            let cache = self.schema_cache.lock().await;
            if let Some(schema) = cache.get(data_source_id) {
                return Ok(schema.clone());
            }
        }
        let ds = self.notion.get_data_source(data_source_id).await?;
        {
            let mut cache = self.schema_cache.lock().await;
            cache.insert(data_source_id.to_string(), ds.clone());
        }
        Ok(ds)
    }

    pub async fn get_enum_options(
        &self,
        data_source_id: &str,
        property_name: &str,
    ) -> Result<Vec<String>, String> {
        let schema = self.get_schema(data_source_id).await?;
        Ok(extract_options_from_schema(&schema, property_name))
    }
}

// ── SchemaCache: config-aware, pre-warmed, keyed by config-key ──

#[derive(Debug, Clone)]
pub struct PropInfo {
    pub notion_name: String,
    pub prop_type: String,
    pub enum_options: Vec<String>,
}

/// Config-key-aware property type cache with reservoir hierarchy.
pub struct SchemaCache {
    /// db_key → (config_key → PropInfo) — includes both reservoirs and satellites
    dbs: HashMap<String, HashMap<String, PropInfo>>,
    /// All database keys in insertion order (reservoirs first, then satellites)
    db_keys: Vec<String>,
    /// reservoir_key → Vec<satellite_key>
    reservoir_satellites: HashMap<String, Vec<String>>,
    /// satellite_key → reservoir_key (reverse lookup)
    satellite_to_reservoir: HashMap<String, String>,
}

impl SchemaCache {
    /// Pre-warm the cache by fetching ALL database schemas (reservoirs + satellites).
    pub async fn init(config: &Arc<LifeOSConfig>, notion: &Arc<NotionClient>) -> Self {
        let mut dbs: HashMap<String, HashMap<String, PropInfo>> = HashMap::new();
        let mut db_keys: Vec<String> = Vec::new();
        let mut reservoir_satellites: HashMap<String, Vec<String>> = HashMap::new();
        let mut satellite_to_reservoir: HashMap<String, String> = HashMap::new();

        let engine = Arc::new(SchemaEngine::new(notion.clone()));
        let semaphore = Arc::new(tokio::sync::Semaphore::new(4));

        // Collect all fetch tasks: reservoirs + satellites
        struct FetchTask {
            key: String,
            ds_id: String,
            props: HashMap<String, String>,
        }

        let mut tasks: Vec<FetchTask> = Vec::new();

        for (db_key, db_cfg) in &config.databases {
            // Ensure all reservoirs appear in the map, even those with no satellites
            reservoir_satellites.insert(db_key.clone(), Vec::new());

            // Reservoir itself
            tasks.push(FetchTask {
                key: db_key.clone(),
                ds_id: db_cfg.ds_id().to_string(),
                props: db_cfg.properties.clone(),
            });

            // Satellites
            for (sat_key, sat_cfg) in &db_cfg.satellites {
                tasks.push(FetchTask {
                    key: sat_key.clone(),
                    ds_id: sat_cfg.ds_id().to_string(),
                    props: sat_cfg.properties.clone(),
                });
                reservoir_satellites
                    .entry(db_key.clone())
                    .or_default()
                    .push(sat_key.clone());
                satellite_to_reservoir.insert(sat_key.clone(), db_key.clone());
            }
        }

        // Execute all fetches concurrently
        let mut futures = Vec::new();
        for task in tasks {
            let key = task.key;
            let ds_id = task.ds_id;
            let props = task.props;
            let eng = engine.clone();
            let sem = semaphore.clone();
            futures.push(async move {
                let _permit = sem.acquire().await;
                let info = eng.get_schema(&ds_id).await.ok().and_then(|schema| {
                    build_prop_info_map(&props, &schema)
                });
                (key, info)
            });
        }

        let results = futures::future::join_all(futures).await;

        for (key, info_opt) in results {
            db_keys.push(key.clone());
            dbs.insert(key, info_opt.unwrap_or_default());
        }

        Self { dbs, db_keys, reservoir_satellites, satellite_to_reservoir }
    }

    pub fn get_prop_type(&self, db_key: &str, config_key: &str) -> Option<&str> {
        self.dbs
            .get(db_key)
            .and_then(|props| props.get(config_key))
            .map(|info| info.prop_type.as_str())
    }

    pub fn get_enum_options(&self, db_key: &str, config_key: &str) -> Option<&[String]> {
        self.dbs
            .get(db_key)
            .and_then(|props| props.get(config_key))
            .map(|info| info.enum_options.as_slice())
            .filter(|opts| !opts.is_empty())
    }

    pub fn db_keys(&self) -> &[String] {
        &self.db_keys
    }

    /// Get the reservoir key that owns a satellite.
    pub fn reservoir_for(&self, satellite_key: &str) -> Option<&str> {
        self.satellite_to_reservoir.get(satellite_key).map(|s| s.as_str())
    }

    /// Check if a key is a reservoir (not a satellite).
    pub fn is_reservoir(&self, key: &str) -> bool {
        self.reservoir_satellites.contains_key(key)
    }



    /// Build a hierarchical description for a reservoir showing its satellites.
    pub fn describe_reservoir(&self, reservoir_key: &str, config: &LifeOSConfig) -> String {
        let mut output = String::new();

        // Reservoir header
        if let Some(db_cfg) = config.databases.get(reservoir_key) {
            let archetype = db_cfg.archetype.as_deref().unwrap_or("unknown");
            let scale = db_cfg.scale.as_deref().unwrap_or("unknown");
            let dimension = db_cfg.dimension.as_deref().unwrap_or("unknown");
            let cycle = db_cfg.cycle.as_deref().unwrap_or("unknown");

            output.push_str(&format!(
                "{} [{}] ({}, {}, {}):\n",
                db_cfg.name, archetype, scale, dimension, cycle
            ));

            // Reservoir own properties
            if let Some(props) = self.dbs.get(reservoir_key) {
                let desc = format_properties(props);
                if !desc.is_empty() {
                    output.push_str(&format!("  Properties: {}\n", desc));
                }
            }

            // Satellites
            if let Some(satellites) = self.reservoir_satellites.get(reservoir_key) {
                if !satellites.is_empty() {
                    output.push_str(&format!("  Satellites ({}):\n", satellites.len()));
                    for sat_key in satellites {
                        if let Some(sat_name) = config.databases.get(reservoir_key)
                            .and_then(|db| db.satellites.get(sat_key))
                            .map(|s| s.name.as_str())
                        {
                            let sat_desc = self.dbs.get(sat_key)
                                .map(|p| format_properties(p))
                                .unwrap_or_default();
                            output.push_str(&format!("    {}: {} {}\n", sat_key, sat_name,
                                if sat_desc.is_empty() { String::new() } else { format!("({})", sat_desc) }
                            ));
                        }
                    }
                }
            }
        }

        output
    }

    /// Describe all properties for a single database (reservoir or satellite).
    pub fn describe_db_properties(&self, db_key: &str) -> String {
        let Some(props) = self.dbs.get(db_key) else {
            return String::new();
        };
        format_properties(props)
    }
}

fn format_properties(props: &HashMap<String, PropInfo>) -> String {
    let parts: Vec<String> = props.iter().map(|(key, info)| {
        let type_hint = match info.prop_type.as_str() {
            "select" | "status" => {
                if info.enum_options.is_empty() {
                    format!("({})", info.prop_type)
                } else {
                    format!("({}:{})", info.prop_type, info.enum_options.join("/"))
                }
            }
            "multi_select" => {
                if info.enum_options.is_empty() {
                    "(multi_select)".to_string()
                } else {
                    format!("(multi_select:{})", info.enum_options.join("/"))
                }
            }
            t => format!("({})", t),
        };
        format!("{}{}", key, type_hint)
    }).collect();
    parts.join(", ")
}

// ── Helpers ──

fn build_prop_info_map(
    property_mapping: &HashMap<String, String>,
    schema: &NotionDataSource,
) -> Option<HashMap<String, PropInfo>> {
    let mut result = HashMap::new();
    for (config_key, notion_name) in property_mapping {
        let Some(prop_schema) = schema.properties.get(notion_name) else {
            continue;
        };
        let options = extract_options(&prop_schema);
        result.insert(
            config_key.clone(),
            PropInfo {
                notion_name: notion_name.clone(),
                prop_type: prop_schema.prop_type.clone(),
                enum_options: options,
            },
        );
    }
    Some(result)
}

fn extract_options(prop: &crate::notion::types::PropertySchema) -> Vec<String> {
    match prop.prop_type.as_str() {
        "select" => prop.select.as_ref()
            .and_then(|s| s.options.as_ref())
            .map(|opts| opts.iter().map(|o| o.name.clone()).collect())
            .unwrap_or_default(),
        "multi_select" => prop.multi_select.as_ref()
            .and_then(|s| s.options.as_ref())
            .map(|opts| opts.iter().map(|o| o.name.clone()).collect())
            .unwrap_or_default(),
        "status" => prop.status.as_ref()
            .and_then(|s| s.options.as_ref())
            .map(|opts| opts.iter().map(|o| o.name.clone()).collect())
            .unwrap_or_default(),
        _ => vec![],
    }
}

fn extract_options_from_schema(schema: &NotionDataSource, property_name: &str) -> Vec<String> {
    if let Some(prop) = schema.properties.get(property_name) {
        return extract_options(prop);
    }
    let lower = property_name.to_lowercase();
    for (name, prop) in &schema.properties {
        if name.to_lowercase() == lower {
            return extract_options(prop);
        }
    }
    vec![]
}
