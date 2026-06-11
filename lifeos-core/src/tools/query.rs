//! Query tool — database query with filters, sorts, and presets

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

/// Query tool parameters
#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub database: String,
    pub filter_property: Option<String>,
    pub filter_value: Option<String>,
    pub filter_type: Option<String>,
    pub sort_property: Option<String>,
    pub sort_direction: Option<String>,
    pub limit: Option<u32>,
    pub return_properties: Option<Vec<String>>,
    pub preset: Option<String>,
}

/// Tool schema for MCP tools/list — enriched with schema cache context
pub fn schema(config: &LifeOSConfig, schema_cache: &SchemaCache) -> serde_json::Value {
    let mut obj = serde_json::json!({
        "type": "object",
        "properties": {
            "database": { "type": "string", "description": "Database key (activity_log, tasks, projects, etc.)" },
            "filter_property": { "type": "string", "description": "Config-key of the property to filter on. Check database schema below for valid keys." },
            "filter_value": { "type": "string", "description": "Value to filter for" },
            "filter_type": { "type": "string", "enum": ["select", "status", "rich_text", "title", "date", "checkbox", "url", "email", "phone_number", "number", "multi_select"], "description": "Property type for filter — use the actual Notion column type, not 'select' for status fields or vice versa" },
            "sort_property": { "type": "string", "description": "Property to sort by" },
            "sort_direction": { "type": "string", "enum": ["ascending", "descending"] },
            "limit": { "type": "integer", "minimum": 1, "maximum": 100, "description": "Max results (default: 50)" },
            "return_properties": { "type": "array", "items": { "type": "string" }, "description": "Specific config-keys to return" },
            "preset": { "type": "string", "enum": ["active", "this_week", "this_month", "needs_review"], "description": "Intelligent preset query" }
        },
        "required": ["database"]
    });

    // Inject per-database property descriptions for AI-agent context
    let db_help: serde_json::Value = serde_json::Value::Object(
        config.databases.keys().map(|db_key| {
            let desc = schema_cache.describe_db_properties(db_key);
            (db_key.clone(), serde_json::Value::String(desc))
        }).collect()
    );
    if let Some(props) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
        props.insert("_db_schemas".to_string(), serde_json::json!({
            "type": "object",
            "description": "Available database keys and their property schemas",
            "properties": db_help
        }));
    }

    obj
}

/// Execute the query tool
pub async fn execute(
    params: &QueryParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let db = crate::get_db(config, &params.database)
        .ok_or_else(|| format!("Unknown database: {}", params.database))?;

    let limit = params.limit.unwrap_or(50).min(100) as u64;
    let mut body = serde_json::json!({ "page_size": limit });

    // Handle preset filters — schema-aware: detect actual prop_type + use valid enum values
    if let Some(ref preset) = params.preset {
        let now = chrono::Utc::now();
        match preset.as_str() {
            "active" => {
                if let Some(prop) = db.properties.get("status") {
                    let actual_type = schema_cache.get_prop_type(&params.database, "status").unwrap_or("select");
                    body["filter"] = build_filter_with_prop(prop, actual_type, "Active");
                }
            }
            "this_week" => {
                if let Some(prop) = db.properties.get("date") {
                    let start = (now - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
                    body["filter"] = serde_json::json!({
                        "property": prop, "date": { "on_or_after": start }
                    });
                }
            }
            "this_month" => {
                if let Some(prop) = db.properties.get("date") {
                    let start = (now - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
                    body["filter"] = serde_json::json!({
                        "property": prop, "date": { "on_or_after": start }
                    });
                }
            }
            "needs_review" => {
                if let Some(prop) = db.properties.get("status") {
                    let actual_type = schema_cache.get_prop_type(&params.database, "status").unwrap_or("select");
                    body["filter"] = build_filter_with_prop(prop, actual_type, "Needs Review");
                }
            }
            _ => {}
        }
    }

    // Handle manual filter — map config-key → Notion property name, validate type against schema
    if let (Some(ref prop_key), Some(ref val)) = (&params.filter_property, &params.filter_value) {
        let notion_prop = db.properties.get(prop_key).map(|s| s.as_str()).unwrap_or(prop_key.as_str());
        let filter_type = params.filter_type.as_deref()
            .or_else(|| schema_cache.get_prop_type(&params.database, prop_key))
            .unwrap_or("rich_text");
        body["filter"] = build_filter(notion_prop, filter_type, val);
    }

    // Handle sort
    if let Some(ref sort_prop) = params.sort_property {
        let direction = params.sort_direction.as_deref().unwrap_or("descending");
        body["sorts"] = serde_json::json!([
            { "property": sort_prop, "direction": direction }
        ]);
    }

    // Execute query
    let result = notion.query_data_source(db.ds_id(), &body).await?;

    let items: Vec<serde_json::Value> = result.results.iter().map(|page| {
        let title = crate::transform::extract_title(page);
        let mut item = serde_json::json!({ "title": title, "id": page.id });

        // Add requested properties
        if let Some(ref props) = params.return_properties {
            for prop_key in props {
                if let Some(notion_name) = db.properties.get(prop_key) {
                    let val = crate::transform::extract_string(page, notion_name);
                    if !val.is_empty() {
                        item[prop_key] = serde_json::json!(val);
                    }
                }
            }
        }
        item
    }).collect();

    // Build result with TOON encoding
    let mut data = serde_json::Map::new();
    data.insert("__schema".into(), serde_json::json!({
        "database": &params.database, "name": &db.name
    }));
    data.insert(params.database.clone(), serde_json::json!(items));
    data.insert("meta".into(), serde_json::json!({
        "count": result.results.len(),
        "has_more": result.has_more
    }));
    let toon_data = serde_json::Value::Object(data);

    Ok(crate::toon_format::encode(&toon_data))
}

/// Build filter using the Notion property name (already resolved).
fn build_filter(property: &str, filter_type: &str, value: &str) -> serde_json::Value {
    match filter_type {
        "select" => serde_json::json!({ "property": property, "select": { "equals": value } }),
        "status" => serde_json::json!({ "property": property, "status": { "equals": value } }),
        "rich_text" => serde_json::json!({ "property": property, "rich_text": { "contains": value } }),
        "title" => serde_json::json!({ "property": property, "title": { "contains": value } }),
        "date" => serde_json::json!({ "property": property, "date": { "equals": value } }),
        "checkbox" => serde_json::json!({ "property": property, "checkbox": { "equals": value == "true" } }),
        "url" => serde_json::json!({ "property": property, "url": { "contains": value } }),
        "email" => serde_json::json!({ "property": property, "email": { "contains": value } }),
        "phone_number" => serde_json::json!({ "property": property, "phone_number": { "contains": value } }),
        "number" => {
            if let Ok(n) = value.parse::<f64>() {
                serde_json::json!({ "property": property, "number": { "equals": n } })
            } else {
                serde_json::json!({ "property": property, "rich_text": { "contains": value } })
            }
        }
        "multi_select" => serde_json::json!({ "property": property, "multi_select": { "contains": value } }),
        _ => serde_json::json!({ "property": property, "rich_text": { "contains": value } }),
    }
}

/// Build filter given a Notion property name and schema-derived type.
fn build_filter_with_prop(notion_prop: &str, actual_type: &str, value: &str) -> serde_json::Value {
    build_filter(notion_prop, actual_type, value)
}
