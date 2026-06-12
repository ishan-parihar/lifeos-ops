//! Intelligence briefing tool — role-based and module-specific analysis

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

/// Intelligence briefing parameters
#[derive(Debug, Deserialize)]
pub struct IntelligenceParams {
    /// Briefing mode: role or module
    pub mode: String,
    /// Role key when mode=role (e.g., CEO, COO, CMO, CRO, CFO, CHO)
    pub role: Option<String>,
    /// Module key when mode=module
    pub module: Option<String>,
    /// Date range: "today", "this_week", "this_month", "this_quarter" or ISO date
    pub range: Option<String>,
}

/// Execute intelligence briefing

/// Generate JSON Schema for this tool
pub fn schema(schema_cache: &SchemaCache) -> serde_json::Value {
    let mut obj = serde_json::json!({
        "type": "object",
        "properties": {
            "mode": { "type": "string", "enum": ["role", "module"], "description": "Briefing mode" },
            "role": { "type": "string", "enum": ["CEO", "COO", "CMO", "CRO", "CFO", "CHO"], "description": "Role key when mode=role" },
            "module": { "type": "string", "enum": ["productivity", "health", "strategic", "financial", "content", "journaling"], "description": "Module key when mode=module" },
            "range": { "type": "string", "description": "Date range: today, this_week, this_month, this_quarter or ISO date" }
        },
        "required": ["mode"]
    });

    // Inject database property context
    let db_help: serde_json::Value = serde_json::Value::Object(
        schema_cache.db_keys().iter().map(|db_key| {
            let desc = schema_cache.describe_db_properties(db_key);
            (db_key.clone(), serde_json::Value::String(desc))
        }).collect()
    );
    if let Some(props) = obj.get_mut("properties").and_then(|p| p.as_object_mut()) {
        props.insert("_db_schemas".to_string(), serde_json::json!({
            "type": "object",
            "description": "Available databases and their property schemas with valid select/status/multi_select options",
            "properties": db_help
        }));
    }

    obj
}

/// Walk a filter tree and correct `"status"` ↔ `"select"` type keys
/// when they don't match the actual Notion property type from SchemaCache.
fn correct_filter_type(
    filter: &serde_json::Value,
    db_key: &str,
    schema_cache: &SchemaCache,
    property_mapping: &std::collections::HashMap<String, String>,
) -> serde_json::Value {
    use serde_json::Value;
    match filter {
        Value::Object(map) => {
            // Compound filter: "or" or "and"
            if let Some(compound_key) = map.get("or").or_else(|| map.get("and")) {
                let key = if map.contains_key("or") { "or" } else { "and" };
                let corrected: Vec<Value> = compound_key.as_array()
                    .map(|arr| arr.iter().map(|c| correct_filter_type(c, db_key, schema_cache, property_mapping)).collect())
                    .unwrap_or_default();
                let mut result = serde_json::Map::new();
                result.insert(key.to_string(), Value::Array(corrected));
                return Value::Object(result);
            }

            // Simple filter with "property" key
            if let Some(prop_name) = map.get("property").and_then(|v| v.as_str()) {
                let config_key = property_mapping.iter()
                    .find(|(_, v)| v.as_str() == prop_name)
                    .map(|(k, _)| k.as_str());

                if let Some(cfg_key) = config_key {
                    let actual_type = schema_cache.get_prop_type(db_key, cfg_key);
                    if let Some(actual) = actual_type {
                        let filterable = ["select", "status", "rich_text", "title", "date", "checkbox", "url", "email", "phone_number", "number", "multi_select"];
                        for type_key in &filterable {
                            if map.contains_key(*type_key) && *type_key != actual {
                                let mut result = serde_json::Map::new();
                                for (k, v) in map.iter() {
                                    if k == type_key {
                                        result.insert(actual.to_string(), v.clone());
                                    } else {
                                        result.insert(k.clone(), v.clone());
                                    }
                                }
                                return Value::Object(result);
                            }
                        }
                    }
                }
            }

            filter.clone()
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| correct_filter_type(v, db_key, schema_cache, property_mapping)).collect())
        }
        _ => filter.clone(),
    }
}

fn build_target_query(
    target: &crate::config::BriefingTarget,
    db: &crate::config::DbConfig,
    db_key: &str,
    range: &str,
    schema_cache: &SchemaCache,
) -> serde_json::Value {
    let mut query = serde_json::json!({ "page_size": target.limit.unwrap_or(10) });

    if let Some(ref static_filter) = target.filter {
        let corrected = correct_filter_type(static_filter, db_key, schema_cache, &db.properties);
        query["filter"] = corrected;
    }

    if target.date_filter.unwrap_or(false) {
        let date_prop = db.properties.get("date").map(|s| s.as_str()).unwrap_or("Date");
        if let Some(df) = build_date_filter(range, date_prop) {
            if query.get("filter").is_some() {
                let combined = serde_json::json!({
                    "and": [query["filter"].clone(), df]
                });
                query["filter"] = combined;
            } else {
                query["filter"] = df;
            }
        }
    }

    query
}

pub async fn execute(
    params: &IntelligenceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let range = params.range.as_deref().unwrap_or("this_week");

    match params.mode.as_str() {
        "role" => {
            let role_display = params.role.as_deref().unwrap_or("CEO");
            let role_key = role_display.to_lowercase();
            let targets = config.briefings.as_ref()
                .and_then(|b| b.roles.get(role_key.as_str()))
                .ok_or_else(|| format!("Unknown role: {} — valid roles: CEO, COO, CMO, CRO, CFO, CHO", role_display))?;

            let mut data = serde_json::json!({
                "briefing_type": "role",
                "role": role_display,
                "range": range
            });

            let mut errors: Vec<String> = Vec::new();
            for target in targets {
                if let Some(db) = crate::get_db(config, &target.db) {
                    let query = build_target_query(target, db, &target.db, range, schema_cache);
                    match notion.query_data_source(db.ds_id(), &query).await {
                        Ok(result) => {
                            let items: Vec<serde_json::Value> = result.results.iter()
                                .map(|p| {
                                    let title = crate::transform::extract_title(p);
                                    serde_json::json!({ "title": title, "id": p.id })
                                }).collect();
                            data[&target.db] = serde_json::json!(items);
                        }
                        Err(e) => {
                            errors.push(format!("{}: {}", target.db, e));
                        }
                    }
                }
            }
            if !errors.is_empty() {
                data["_errors"] = serde_json::json!(errors);
            }

            Ok(crate::toon_format::encode(&data))
        }
        "module" => {
            let module_display = params.module.as_deref().unwrap_or("productivity");
            let module_key = module_display.to_lowercase();
            let targets = config.briefings.as_ref()
                .and_then(|b| b.modules.get(module_key.as_str()))
                .ok_or_else(|| format!("Unknown module: {} — valid modules: productivity, health, strategic, financial, content, journaling", module_display))?;

            let mut data = serde_json::json!({
                "briefing_type": "module",
                "module": module_display,
                "range": range
            });

            let mut errors: Vec<String> = Vec::new();
            for target in targets {
                if let Some(db) = crate::get_db(config, &target.db) {
                    let query = build_target_query(target, db, &target.db, range, schema_cache);
                    match notion.query_data_source(db.ds_id(), &query).await {
                        Ok(result) => {
                            let items: Vec<serde_json::Value> = result.results.iter()
                                .map(|p| {
                                    let title = crate::transform::extract_title(p);
                                    serde_json::json!({ "title": title, "id": p.id })
                                }).collect();
                            data[&target.db] = serde_json::json!(items);
                        }
                        Err(e) => {
                            errors.push(format!("{}: {}", target.db, e));
                        }
                    }
                }
            }
            if !errors.is_empty() {
                data["_errors"] = serde_json::json!(errors);
            }

            Ok(crate::toon_format::encode(&data))
        }
        _ => Err(format!("Unknown mode: {}", params.mode)),
    }
}

fn build_date_filter(range: &str, date_prop: &str) -> Option<serde_json::Value> {
    let now = chrono::Utc::now();
    match range {
        "today" => Some(serde_json::json!({
            "property": date_prop,
            "date": { "equals": now.format("%Y-%m-%d").to_string() }
        })),
        "this_week" => {
            let start = (now - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
            Some(serde_json::json!({
                "property": date_prop,
                "date": { "on_or_after": start }
            }))
        }
        "this_month" => {
            let start = (now - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
            Some(serde_json::json!({
                "property": date_prop,
                "date": { "on_or_after": start }
            }))
        }
        "this_quarter" => {
            let start = (now - chrono::Duration::days(90)).format("%Y-%m-%d").to_string();
            Some(serde_json::json!({
                "property": date_prop,
                "date": { "on_or_after": start }
            }))
        }
        _ => None,
    }
}
