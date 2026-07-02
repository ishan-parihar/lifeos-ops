//! Schema Engine — fetches and caches Notion database schemas
//!
//! Provides two layers:
//! - `SchemaEngine`: per-data-source schema caching (original)
//! - `SchemaCache`: config-aware caching for the 5 unified databases + relation graph

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

/// A relation edge: property name → target database key.
#[derive(Debug, Clone)]
pub struct RelationEdge {
    pub prop_name: String,
    pub target_db: String,
}

/// Config-key-aware property type cache with relation graph.
pub struct SchemaCache {
    /// db_key → (config_key → PropInfo) — the 5 unified databases
    dbs: HashMap<String, HashMap<String, PropInfo>>,
    /// All database keys in insertion order
    db_keys: Vec<String>,
    /// db_key → Vec<RelationEdge> — which properties link to which databases
    relation_graph: HashMap<String, Vec<RelationEdge>>,
    /// database_id → config_key (reverse map for resolving relation targets)
    id_to_key: HashMap<String, String>,
}

impl SchemaCache {
    /// Pre-warm the cache by fetching ALL 5 unified database schemas.
    pub async fn init(config: &Arc<LifeOSConfig>, notion: &Arc<NotionClient>) -> Self {
        let mut dbs: HashMap<String, HashMap<String, PropInfo>> = HashMap::new();
        let mut db_keys: Vec<String> = Vec::new();

        let engine = Arc::new(SchemaEngine::new(notion.clone()));
        let semaphore = Arc::new(tokio::sync::Semaphore::new(4));

        // Collect all fetch tasks: the 5 unified databases
        struct FetchTask {
            key: String,
            ds_id: String,
            props: HashMap<String, String>,
        }

        let mut tasks: Vec<FetchTask> = Vec::new();

        for (db_key, db_cfg) in &config.databases {
            tasks.push(FetchTask {
                key: db_key.clone(),
                ds_id: db_cfg.ds_id().to_string(),
                props: db_cfg.properties.clone(),
            });
        }

        // Execute all fetches concurrently — also capture raw schemas for relation extraction
        struct FetchResult {
            key: String,
            ds_id: String,
            props: Option<HashMap<String, PropInfo>>,
            raw_schema: Option<NotionDataSource>,
        }

        let mut futures = Vec::new();
        for task in tasks {
            let key = task.key;
            let ds_id = task.ds_id.clone();
            let props = task.props;
            let eng = engine.clone();
            let sem = semaphore.clone();
            futures.push(async move {
                let _permit = sem.acquire().await;
                let raw = eng.get_schema(&ds_id).await.ok();
                let prop_info = raw.as_ref().and_then(|schema| {
                    build_prop_info_map(&props, schema)
                });
                FetchResult { key, ds_id: task.ds_id, props: prop_info, raw_schema: raw }
            });
        }

        let results = futures::future::join_all(futures).await;

        // Build reverse map: database_id → config_key
        let mut id_to_key: HashMap<String, String> = HashMap::new();
        for (db_key, db_cfg) in &config.databases {
            id_to_key.insert(db_cfg.database_id.clone(), db_key.clone());
        }

        // Collect raw schemas and prop info
        let mut raw_schemas: HashMap<String, NotionDataSource> = HashMap::new();
        for result in results {
            db_keys.push(result.key.clone());
            if let Some(props) = result.props {
                dbs.insert(result.key.clone(), props);
            } else {
                dbs.insert(result.key.clone(), HashMap::new());
            }
            if let Some(schema) = result.raw_schema {
                raw_schemas.insert(result.ds_id, schema);
            }
        }

        // Build relation graph from raw schemas
        let mut relation_graph: HashMap<String, Vec<RelationEdge>> = HashMap::new();
        for (ds_id, schema) in &raw_schemas {
            // Find which config key owns this schema
            let source_config_key = find_key_for_ds_id(ds_id, config);

            if let Some(src_key) = source_config_key {
                let mut edges = Vec::new();
                for (prop_name, prop_schema) in &schema.properties {
                    if prop_schema.prop_type == "relation" {
                        if let Some(ref rel_config) = prop_schema.relation {
                            let target_key = id_to_key.get(&rel_config.database_id)
                                .cloned()
                                .unwrap_or_else(|| format!("unknown({})", &rel_config.database_id[..8.min(rel_config.database_id.len())]));
                            edges.push(RelationEdge {
                                prop_name: prop_name.clone(),
                                target_db: target_key,
                            });
                        }
                    }
                }
                relation_graph.insert(src_key, edges);
            }
        }

        Self { dbs, db_keys, relation_graph, id_to_key }
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

    /// Get outgoing relation edges for a database (which properties link to which databases).
    pub fn get_relation_edges(&self, db_key: &str) -> &[RelationEdge] {
        self.relation_graph.get(db_key).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all relation edges as a flat list for the full graph.
    pub fn all_relation_edges(&self) -> &HashMap<String, Vec<RelationEdge>> {
        &self.relation_graph
    }

    /// Resolve a database_id back to a config key.
    pub fn resolve_db_key_from_id(&self, database_id: &str) -> Option<&str> {
        self.id_to_key.get(database_id).map(|s| s.as_str())
    }

    /// Build a description for a database showing its properties, entry types, and holonic role.
    pub fn describe_reservoir(&self, reservoir_key: &str, config: &LifeOSConfig) -> String {
        let mut output = String::new();

        if let Some(db_cfg) = config.databases.get(reservoir_key) {
            let archetype = db_cfg.archetype.as_deref().unwrap_or("unknown");
            let scale = db_cfg.scale.as_deref().unwrap_or("unknown");
            let dimension = db_cfg.dimension.as_deref().unwrap_or("unknown");
            let cycle = db_cfg.cycle.as_deref().unwrap_or("unknown");

            output.push_str(&format!(
                "{} [{}] ({}, {}, {}):\n",
                db_cfg.name, archetype, scale, dimension, cycle
            ));

            // Description
            if let Some(ref desc) = db_cfg.description {
                output.push_str(&format!("  Role: {}\n", desc));
            }

            // Entry type property name (which Notion property to filter on)
            if let Some(ref et_prop) = db_cfg.entry_type_property {
                output.push_str(&format!("  Entry Type Property: {}\n", et_prop));
            }

            // Entry types (from config descriptions)
            if let Some(entry_types) = config.entry_type_descriptions(reservoir_key) {
                output.push_str(&format!("  Entry Types ({}):\n", entry_types.len()));
                for (et_name, et_desc) in entry_types {
                    output.push_str(&format!("    {}: {}\n", et_name, et_desc));
                }
            }

            // Properties + relations
            if let Some(props) = self.dbs.get(reservoir_key) {
                let desc = format_properties_with_relations(props, self.relation_graph.get(reservoir_key));
                if !desc.is_empty() {
                    output.push_str(&format!("  Properties: {}\n", desc));
                }
            }
        }

        output
    }

    /// Describe all properties for a single database.
    pub fn describe_db_properties(&self, db_key: &str) -> String {
        let Some(props) = self.dbs.get(db_key) else {
            return String::new();
        };
        format_properties_with_relations(props, self.relation_graph.get(db_key))
    }

    /// Describe the full relation graph as human-readable text.
    pub fn describe_relation_graph(&self) -> String {
        let mut output = String::from("LifeOS Relational Graph:\n\n");
        for (db_key, edges) in &self.relation_graph {
            output.push_str(&format!("{}:\n", db_key));
            for edge in edges {
                output.push_str(&format!("  {} → {}\n", edge.prop_name, edge.target_db));
            }
        }
        output
    }
}

/// Find the config key for a given data_source_id.
fn find_key_for_ds_id(ds_id: &str, config: &LifeOSConfig) -> Option<String> {
    for (db_key, db_cfg) in &config.databases {
        if db_cfg.ds_id() == ds_id {
            return Some(db_key.clone());
        }
    }
    None
}



/// Format properties with relation targets annotated.
fn format_properties_with_relations(props: &HashMap<String, PropInfo>, edges: Option<&Vec<RelationEdge>>) -> String {
    let edge_map: HashMap<&str, &str> = edges
        .map(|e| e.iter().map(|e| (e.prop_name.as_str(), e.target_db.as_str())).collect())
        .unwrap_or_default();

    let parts: Vec<String> = props.iter().map(|(key, info)| {
        let type_hint = if info.prop_type == "relation" {
            match edge_map.get(key.as_str()) {
                Some(target) => format!("(relation→{})", target),
                None => "(relation)".to_string(),
            }
        } else {
            match info.prop_type.as_str() {
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
            }
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
