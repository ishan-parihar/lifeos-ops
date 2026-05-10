//! Instruction Injector — generates schema-aware agent instructions
//!
//! Mirrors the TypeScript src/utils/instruction-injector.ts

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaEngine;

/// Cached instructions
struct InstructionEntry {
    data: serde_json::Value,
}

/// Generates schema-aware instructions for the LLM agent
pub struct InstructionInjector {
    schema_engine: Arc<SchemaEngine>,
    config: LifeOSConfig,
    cache: Arc<Mutex<HashMap<String, InstructionEntry>>>,
}

impl InstructionInjector {
    pub fn new(schema_engine: Arc<SchemaEngine>, config: LifeOSConfig) -> Self {
        Self {
            schema_engine,
            config,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Generate instructions for a database
    pub async fn generate(&self, db_key: &str) -> serde_json::Value {
        // Check cache
        {
            let cache = self.cache.lock().await;
            if let Some(entry) = cache.get(db_key) {
                return entry.data.clone();
            }
        }

        let db = match crate::config::get_db(&self.config, db_key) {
            Some(d) => d,
            None => return serde_json::json!({
                "_note": "Unknown database",
                "database": db_key
            }),
        };

        // Try to fetch schema
        let schema = self.schema_engine.get_schema(&db.data_source_id).await;

        let instructions = match schema {
            Ok(ds) => {
                let mut props = serde_json::Map::new();
                for (notion_name, prop_schema) in &ds.properties {
                    let mut info = serde_json::json!({
                        "type": prop_schema.prop_type,
                        "notion_property_name": notion_name,
                    });

                    // Add enum options for select/status/multi_select
                    if prop_schema.prop_type == "select" {
                        if let Some(opts) = prop_schema.select.as_ref().and_then(|s| s.options.as_ref()) {
                            info["options"] = serde_json::json!(opts.iter().map(|o| o.name.clone()).collect::<Vec<_>>());
                        }
                    }
                    if prop_schema.prop_type == "status" {
                        if let Some(opts) = prop_schema.status.as_ref().and_then(|s| s.options.as_ref()) {
                            info["options"] = serde_json::json!(opts.iter().map(|o| o.name.clone()).collect::<Vec<_>>());
                        }
                    }
                    if prop_schema.prop_type == "multi_select" {
                        if let Some(opts) = prop_schema.multi_select.as_ref().and_then(|s| s.options.as_ref()) {
                            info["options"] = serde_json::json!(opts.iter().map(|o| o.name.clone()).collect::<Vec<_>>());
                        }
                    }

                    // Map config key to notion property name
                    for (config_key, config_name) in &db.properties {
                        if config_name == notion_name {
                            props.insert(config_key.clone(), info);
                            break;
                        }
                    }
                }

                serde_json::json!({
                    "_note": "Auto-generated schema guidelines",
                    "database": db_key,
                    "properties": props,
                })
            }
            Err(e) => serde_json::json!({
                "_note": "Schema inaccessible",
                "database": db_key,
                "error": e,
                "fallback_properties": db.properties.keys().collect::<Vec<_>>(),
            }),
        };

        // Cache
        {
            let mut cache = self.cache.lock().await;
            cache.insert(db_key.to_string(), InstructionEntry { data: instructions.clone() });
        }

        instructions
    }
}
