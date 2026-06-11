//! Intelligence briefing tool — role-based and module-specific analysis

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

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
pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "mode": { "type": "string", "enum": ["role", "module"], "description": "Briefing mode" },
            "role": { "type": "string", "enum": ["CEO", "COO", "CMO", "CRO", "CFO", "CHO"], "description": "Role key when mode=role" },
            "module": { "type": "string", "description": "Module key when mode=module" },
            "range": { "type": "string", "description": "Date range: today, this_week, this_month, this_quarter or ISO date" }
        },
        "required": ["mode"]
    })
}

pub async fn execute(
    params: &IntelligenceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let range = params.range.as_deref().unwrap_or("this_week");
    let date_filter = build_date_filter(range);

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
                    let mut query = serde_json::json!({ "page_size": target.limit.unwrap_or(10) });
                    if let Some(ref date_filter) = date_filter {
                        if target.date_filter.unwrap_or(false) {
                            query["filter"] = date_filter.clone();
                        }
                    }
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
                .ok_or_else(|| format!("Unknown module: {}", module_display))?;

            let mut data = serde_json::json!({
                "briefing_type": "module",
                "module": module_display,
                "range": range
            });

            let mut errors: Vec<String> = Vec::new();
            for target in targets {
                if let Some(db) = crate::get_db(config, &target.db) {
                    let mut query = serde_json::json!({ "page_size": target.limit.unwrap_or(10) });
                    if let Some(ref date_filter) = date_filter {
                        if target.date_filter.unwrap_or(false) {
                            query["filter"] = date_filter.clone();
                        }
                    }
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

fn build_date_filter(range: &str) -> Option<serde_json::Value> {
    let now = chrono::Utc::now();
    match range {
        "today" => Some(serde_json::json!({
            "date": { "equals": now.format("%Y-%m-%d").to_string() }
        })),
        "this_week" => {
            let start = (now - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
            Some(serde_json::json!({ "date": { "on_or_after": start } }))
        }
        "this_month" => {
            let start = (now - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
            Some(serde_json::json!({ "date": { "on_or_after": start } }))
        }
        "this_quarter" => {
            let start = (now - chrono::Duration::days(90)).format("%Y-%m-%d").to_string();
            Some(serde_json::json!({ "date": { "on_or_after": start } }))
        }
        _ => None,
    }
}
