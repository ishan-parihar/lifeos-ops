//! TODO: Implement this tool
use std::sync::Arc;
use serde_json::{json, Value};
use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

pub fn schema() -> Value {
    json!({"type": "object", "properties": {}})
}

pub async fn execute(
    _config: &Arc<LifeOSConfig>,
    _notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    Ok("Tool not yet implemented.".to_string())
}
