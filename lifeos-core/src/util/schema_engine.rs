//! Schema Engine — fetches and caches Notion database schemas

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::notion::client::NotionClient;
use crate::notion::types::NotionDataSource;

/// Schema engine with caching
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
        // Check cache
        {
            let cache = self.schema_cache.lock().await;
            if let Some(schema) = cache.get(data_source_id) {
                return Ok(schema.clone());
            }
        }

        // Fetch from API
        let ds = self.notion.get_data_source(data_source_id).await?;

        // Store in cache
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

        // Extract options from schema
        let extract = |prop: &crate::notion::types::PropertySchema| -> Vec<String> {
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
        };

        if let Some(prop) = schema.properties.get(property_name) {
            return Ok(extract(prop));
        }

        // Try case-insensitive match (non-recursive)
        let lower = property_name.to_lowercase();
        for (name, prop) in &schema.properties {
            if name.to_lowercase() == lower {
                return Ok(extract(prop));
            }
        }

        Ok(vec![])
    }
}
