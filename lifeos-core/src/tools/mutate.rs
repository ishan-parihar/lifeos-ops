//! Mutate tool — create, update, delete, upsert entries

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

/// Mutate tool parameters
#[derive(Debug, Deserialize)]
pub struct MutateParams {
    /// Operation: create, update, delete, upsert
    pub operation: String,
    /// Target database key
    pub database: String,
    /// Properties to set (key-value pairs for create/update/upsert)
    pub properties: Option<serde_json::Value>,
    /// Page ID (required for update/delete)
    pub page_id: Option<String>,
    /// Target name for fuzzy resolution (alternative to page_id)
    pub target_name: Option<String>,
}

/// Execute the mutate tool

/// Generate JSON Schema for this tool
pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "operation": { "type": "string", "enum": ["create", "update", "delete", "upsert"], "description": "Operation to perform" },
            "database": { "type": "string", "description": "Target database key" },
            "properties": { "type": "object", "description": "Key-value properties for create/update/upsert" },
            "page_id": { "type": "string", "description": "Page ID for update/delete" },
            "target_name": { "type": "string", "description": "Fuzzy page name to resolve for upsert" }
        },
        "required": ["operation", "database"]
    })
}

pub async fn execute(
    params: &MutateParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let db = crate::get_db(config, &params.database)
        .ok_or_else(|| format!("Unknown database: {}", params.database))?;

    match params.operation.as_str() {
        "create" => {
            let props = params.properties.as_ref()
                .ok_or("properties required for create")?;
            let body = serde_json::json!({
                "parent": { "database_id": db.data_source_id },
                "properties": props
            });
            let page = notion.create_page(&body).await?;
            let title = crate::transform::extract_title(&page);
            Ok(format!("Created: {} ({})", title, page.id))
        }
        "update" => {
            let page_id = resolve_page_id(params, notion, config).await?;
            let props = params.properties.as_ref()
                .ok_or("properties required for update")?;
            let page = notion.update_page(&page_id, props).await?;
            let title = crate::transform::extract_title(&page);
            Ok(format!("Updated: {} ({})", title, page.id))
        }
        "delete" | "archive" => {
            let page_id = resolve_page_id(params, notion, config).await?;
            let page = notion.archive_page(&page_id).await?;
            let title = crate::transform::extract_title(&page);
            Ok(format!("Archived: {} ({})", title, page.id))
        }
        "upsert" => {
            // Try to find existing page by title, create if not found
            let target_name = params.target_name.as_deref()
                .or_else(|| params.properties.as_ref()
                    .and_then(|p| p.get("Name"))
                    .and_then(|v| v.as_str()))
                .ok_or("target_name or 'Name' property required for upsert")?;

            let query_body = serde_json::json!({
                "page_size": 10,
                "filter": {
                    "property": "Name",
                    "title": { "equals": target_name }
                }
            });
            let result = notion.query_database(&db.data_source_id, &query_body).await?;

            let props = params.properties.as_ref()
                .ok_or("properties required for upsert")?;

            if let Some(page) = result.results.first() {
                notion.update_page(&page.id, props).await?;
                Ok(format!("Upsert (updated): {}", target_name))
            } else {
                let body = serde_json::json!({
                    "parent": { "database_id": db.data_source_id },
                    "properties": props
                });
                notion.create_page(&body).await?;
                Ok(format!("Upsert (created): {}", target_name))
            }
        }
        _ => Err(format!("Unknown operation: {}", params.operation)),
    }
}

/// Resolve page ID from explicit id or fuzzy name match
async fn resolve_page_id(
    params: &MutateParams,
    notion: &NotionClient,
    config: &LifeOSConfig,
) -> Result<String, String> {
    if let Some(ref id) = params.page_id {
        if !id.is_empty() { return Ok(id.clone()); }
    }
    let name = params.target_name.as_deref()
        .ok_or("page_id or target_name required")?;
    let result = crate::util::id_resolver::resolve_target_id(
        notion, config, &params.database, Some(name), None
    ).await;
    result.id.ok_or_else(|| format!("Could not resolve '{}'", name))
}
