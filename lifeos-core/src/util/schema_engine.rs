//! Schema Engine — fetches and caches Notion database schemas
//!
//! Provides two layers:
//! - `SchemaEngine`: per-data-source schema caching (original)
//! - `SchemaCache`: config-aware caching with config-key → property-type mapping

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

    /// Get the schema for a database/data source
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

    /// Get the available enum options for a property
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

/// Per-property cached info derived from the Notion API schema.
#[derive(Debug, Clone)]
pub struct PropInfo {
    /// The Notion property name (e.g. "Status", "Tags")
    pub notion_name: String,
    /// The Notion property type (e.g. "select", "status", "rich_text", "title", "url", …)
    pub prop_type: String,
    /// Valid option names for select / status / multi_select (empty for other types)
    pub enum_options: Vec<String>,
}

/// Config-key-aware property type cache.
///
/// Maps `db_key → (config_key → PropInfo)` so tools can look up
/// the Notion property type and enum options for any config-key.
pub struct SchemaCache {
    /// db_key → (config_key → PropInfo)
    dbs: HashMap<String, HashMap<String, PropInfo>>,
    /// All database keys in insertion order
    db_keys: Vec<String>,
}

impl SchemaCache {
    /// Pre-warm the cache by fetching ALL database schemas and
    /// building the config-key → property-type mapping.
    pub async fn init(config: &Arc<LifeOSConfig>, notion: &Arc<NotionClient>) -> Self {
        let mut dbs: HashMap<String, HashMap<String, PropInfo>> = HashMap::new();
        let mut db_keys: Vec<String> = Vec::new();

        let engine = Arc::new(SchemaEngine::new(notion.clone()));

        let mut futures = Vec::new();
        for (db_key, db_cfg) in &config.databases {
            let key = db_key.clone();
            let ds_id = db_cfg.ds_id().to_string();
            let eng = engine.clone();
            let props = db_cfg.properties.clone();
            futures.push(async move {
                let info = eng.get_schema(&ds_id).await.ok().and_then(|schema| {
                    build_prop_info_map(&props, &schema)
                });
                (key, info)
            });
        }

        let mut results: Vec<(String, Option<HashMap<String, PropInfo>>)> = Vec::new();
        for fut in futures {
            results.push(fut.await);
        }

        for (key, info_opt) in results {
            db_keys.push(key.clone());
            if let Some(info) = info_opt {
                dbs.insert(key, info);
            } else {
                dbs.insert(key, HashMap::new());
            }
        }

        Self { dbs, db_keys }
    }

    /// Returns the Notion property type for a given db_key + config_key.
    /// E.g. `cache.get_prop_type("tasks", "status") → Some("select")`
    pub fn get_prop_type(&self, db_key: &str, config_key: &str) -> Option<&str> {
        self.dbs
            .get(db_key)
            .and_then(|props| props.get(config_key))
            .map(|info| info.prop_type.as_str())
    }

    /// Returns the enum option names for select/status/multi_select properties.
    pub fn get_enum_options(&self, db_key: &str, config_key: &str) -> Option<&[String]> {
        self.dbs
            .get(db_key)
            .and_then(|props| props.get(config_key))
            .map(|info| info.enum_options.as_slice())
            .filter(|opts| !opts.is_empty())
    }

    /// Returns all database keys.
    pub fn db_keys(&self) -> &[String] {
        &self.db_keys
    }

    /// Returns all property info for a database (config-key → PropInfo).
    pub fn properties_of(&self, db_key: &str) -> Option<&HashMap<String, PropInfo>> {
        self.dbs.get(db_key)
    }

    /// Build a human-readable compact description for a database showing
    /// available config-keys and their types. Useful for schema injection.
    ///
    /// Example: `"tasks: title(text), status(select:Active,Done), priority(select:High,Medium,Low)"`
    pub fn describe_db_properties(&self, db_key: &str) -> String {
        let Some(props) = self.dbs.get(db_key) else {
            return String::new();
        };
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
}

// ── Helpers ──

/// Build a config-key → PropInfo map by cross-referencing the DbConfig
/// property-name mapping against the NotionDataSource property schemas.
fn build_prop_info_map(
    property_mapping: &HashMap<String, String>,
    schema: &NotionDataSource,
) -> Option<HashMap<String, PropInfo>> {
    let mut result = HashMap::new();
    for (config_key, notion_name) in property_mapping {
        // Skip properties not found in the Notion schema (don't abort entire map)
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

/// Extract enum options directly from a loaded NotionDataSource (for SchemaEngine compatibility)
fn extract_options_from_schema(schema: &NotionDataSource, property_name: &str) -> Vec<String> {
    if let Some(prop) = schema.properties.get(property_name) {
        return extract_options(prop);
    }
    // Try case-insensitive match
    let lower = property_name.to_lowercase();
    for (name, prop) in &schema.properties {
        if name.to_lowercase() == lower {
            return extract_options(prop);
        }
    }
    vec![]
}
