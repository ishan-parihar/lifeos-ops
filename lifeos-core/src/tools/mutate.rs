//! Mutate tool — create, update, delete, upsert entries

use std::collections::HashMap;
use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

/// Mutate tool parameters
#[derive(Debug, Deserialize)]
pub struct MutateParams {
    /// Operation: create, update, delete, upsert
    pub operation: String,
    /// Target database key
    pub database: String,
    /// Properties to set (config-key → value; value may be a simple scalar or Notion API object)
    pub properties: Option<serde_json::Value>,
    /// Page ID (required for update/delete)
    pub page_id: Option<String>,
    /// Target name for fuzzy resolution (alternative to page_id)
    pub target_name: Option<String>,
}

/// Generate JSON Schema for this tool
pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "operation": { "type": "string", "enum": ["create", "update", "delete", "upsert"], "description": "Operation to perform" },
            "database": { "type": "string", "description": "Target database key" },
            "properties": {
                "type": "object",
                "description": "Key-value properties using config keys. Simple values auto-detect type: 'Done' for select/status fields becomes {\"select\":{\"name\":\"Done\"}}; 'https://...' becomes {\"url\":\"...\"}. For arrays use [\"A\",\"B\"] for multi_select, or pass Notion API format objects."
            },
            "page_id": { "type": "string", "description": "Page ID for update/delete" },
            "target_name": { "type": "string", "description": "Fuzzy page name to resolve for upsert/update/delete" }
        },
        "required": ["operation", "database"]
    })
}

const NOTION_PROP_TYPES: &[&str] = &[
    "title", "rich_text", "select", "status", "multi_select",
    "date", "number", "checkbox", "people", "relation",
    "url", "email", "phone_number", "files",
];

fn is_notion_format(val: &serde_json::Value) -> bool {
    val.as_object()
        .map(|obj| NOTION_PROP_TYPES.iter().any(|t| obj.contains_key(*t)))
        .unwrap_or(false)
}

fn looks_like_date(s: &str) -> bool {
    s.len() >= 10
        && s.as_bytes()[4] == b'-'
        && s.as_bytes()[7] == b'-'
        && s[..4].parse::<u32>().is_ok()
}

fn coerce_value(
    value: &serde_json::Value,
    config_key: &str,
    prop_type: Option<&str>,
) -> serde_json::Value {
    if is_notion_format(value) {
        return value.clone();
    }
    match value {
        serde_json::Value::String(s) => {
            match prop_type {
                Some("title") => serde_json::json!({"title": [{"type": "text", "text": {"content": s}}]}),
                Some("select") => serde_json::json!({"select": {"name": s}}),
                Some("status") => serde_json::json!({"status": {"name": s}}),
                Some("url") => serde_json::json!({"url": s}),
                Some("email") => serde_json::json!({"email": s}),
                Some("phone_number") => serde_json::json!({"phone_number": s}),
                _ => {
                    if config_key == "title" {
                        serde_json::json!({"title": [{"type": "text", "text": {"content": s}}]})
                    } else if looks_like_date(s) {
                        serde_json::json!({"date": {"start": s}})
                    } else {
                        serde_json::json!({"rich_text": [{"type": "text", "text": {"content": s}}]})
                    }
                }
            }
        }
        serde_json::Value::Array(arr) => {
            match prop_type {
                Some("multi_select") => {
                    let names: Vec<serde_json::Value> = arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| serde_json::json!({"name": s}))
                        .collect();
                    serde_json::json!({"multi_select": names})
                }
                Some("people") => {
                    let users: Vec<serde_json::Value> = arr.iter()
                        .filter_map(|v| {
                            let id = v.get("id").and_then(|i| i.as_str()).or_else(|| v.as_str())?;
                            let name = v.get("name").and_then(|n| n.as_str()).unwrap_or("");
                            Some(serde_json::json!({"object": "user", "id": id, "name": name}))
                        })
                        .collect();
                    serde_json::json!({"people": users})
                }
                Some("relation") => {
                    let ids: Vec<serde_json::Value> = arr.iter()
                        .filter_map(|v| {
                            let id = v.get("id").and_then(|i| i.as_str()).or_else(|| v.as_str())?;
                            Some(serde_json::json!({"id": id}))
                        })
                        .collect();
                    serde_json::json!({"relation": ids})
                }
                Some("files") => {
                    let files: Vec<serde_json::Value> = arr.iter()
                        .filter_map(|v| {
                            let name = v.get("name").and_then(|n| n.as_str()).unwrap_or("file");
                            let url = v.get("url").and_then(|u| u.as_str()).unwrap_or("");
                            Some(serde_json::json!({"name": name, "type": "external", "external": {"url": url}}))
                        })
                        .collect();
                    serde_json::json!({"files": files})
                }
                _ => {
                    // If user passes ["id1","id2"] without prop_type, try to detect from context
                    let has_objects = arr.iter().any(|v| v.is_object());
                    if !has_objects && arr.iter().all(|v| v.is_string()) {
                        let ids: Vec<serde_json::Value> = arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| serde_json::json!({"id": s}))
                            .collect();
                        serde_json::json!({"relation": ids})
                    } else {
                        value.clone()
                    }
                }
            }
        }
        serde_json::Value::Number(n) => serde_json::json!({"number": n}),
        serde_json::Value::Bool(b) => serde_json::json!({"checkbox": b}),
        _ => value.clone(),
    }
}

/// Translate config-key property names → Notion property names and coerce values
/// using schema type info for accurate formatting.
fn map_properties(
    props: &serde_json::Value,
    property_mapping: &HashMap<String, String>,
    db_key: &str,
    schema_cache: &SchemaCache,
) -> serde_json::Value {
    let Some(map) = props.as_object() else { return props.clone() };
    let mut result = serde_json::Map::new();
    for (key, value) in map {
        let notion_key = property_mapping.get(key).map(|s| s.as_str()).unwrap_or(key.as_str());
        let prop_type = schema_cache.get_prop_type(db_key, key);
        result.insert(notion_key.to_string(), coerce_value(value, key, prop_type));
    }
    serde_json::Value::Object(result)
}

pub async fn execute(
    params: &MutateParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let db = crate::get_db(config, &params.database)
        .ok_or_else(|| format!("Unknown database: {}", params.database))?;

    match params.operation.as_str() {
        "create" => {
            let props = params.properties.as_ref()
                .ok_or("properties required for create")?;
            let mapped = map_properties(props, &db.properties, &params.database, schema_cache);
            let body = serde_json::json!({
                "parent": { "data_source_id": db.ds_id() },
                "properties": mapped
            });
            let page = notion.create_page(&body).await?;
            let title = crate::transform::extract_title(&page);
            Ok(format!("Created: {} ({})", title, page.id))
        }
        "update" => {
            let page_id = resolve_page_id(params, notion, config).await?;
            let props = params.properties.as_ref()
                .ok_or("properties required for update")?;
            let mapped = map_properties(props, &db.properties, &params.database, schema_cache);
            let page = notion.update_page(&page_id, &mapped).await?;
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
            let target_name = params.target_name.as_deref()
                .or_else(|| params.properties.as_ref()
                    .and_then(|p| p.get("Name"))
                    .and_then(|v| v.as_str()))
                .ok_or("target_name or 'Name' property required for upsert")?;

            let title_notion_name = db.properties.get("title").map(|s| s.as_str()).unwrap_or("Name");
            let query_body = serde_json::json!({
                "page_size": 10,
                "filter": {
                    "property": title_notion_name,
                    "title": { "equals": target_name }
                }
            });
            let result = notion.query_data_source(db.ds_id(), &query_body).await?;

            let props = params.properties.as_ref()
                .ok_or("properties required for upsert")?;
            let mapped = map_properties(props, &db.properties, &params.database, schema_cache);

            if let Some(page) = result.results.first() {
                notion.update_page(&page.id, &mapped).await?;
                Ok(format!("Upsert (updated): {}", target_name))
            } else {
                let body = serde_json::json!({
                    "parent": { "data_source_id": db.ds_id() },
                    "properties": mapped
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
    if let Some(ref matches) = result.matches {
        return Err(format!(
            "Ambiguous name '{}': {} matches found. Use page_id instead:\n  - {}",
            name,
            matches.len(),
            matches.join("\n  - ")
        ));
    }
    result.id.ok_or_else(|| format!("Could not resolve '{}' in {} database", name, params.database))
}
