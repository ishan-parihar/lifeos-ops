//! Strategic simulator tool — cross-database strategic analysis

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

/// Strategic simulator parameters
#[derive(Debug, Deserialize)]
pub struct StrategicParams {
    /// Analysis type: alignment, project_health, okr_progress, campaign_metrics, overview
    pub analysis_type: String,
    /// Project database key (for project_health) — accepts database key (e.g., "greatway")
    pub project_database: Option<String>,
    /// OKR database key (for okr_progress) — accepts database key
    pub okr_database: Option<String>,
    /// Campaign database key (for campaign_metrics) — accepts database key
    pub campaign_database: Option<String>,
}

/// Generate JSON Schema for this tool
pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "analysis_type": { "type": "string", "enum": ["overview", "alignment", "project_health", "okr_progress", "campaign_metrics"], "description": "Analysis type" },
            "project_database": { "type": "string", "description": "Project database key for project_health (e.g., 'greatway', auto-discovered if omitted)" },
            "okr_database": { "type": "string", "description": "OKR/quarterly goals database key for okr_progress (auto-discovered if omitted)" },
            "campaign_database": { "type": "string", "description": "Campaign database key for campaign_metrics (auto-discovered if omitted)" }
        },
        "required": ["analysis_type"]
    })
}

/// Auto-discover entries by entry type within a database.
/// In v5, entries are filtered by their Entry Type / Item Type property.

/// Resolve a database key for strategic analysis.
/// Accepts a reservoir key and returns (ds_id, properties, name).
fn resolve_for_analysis<'a>(
    config: &'a LifeOSConfig,
    key: Option<&str>,
    fallback_archetype: &str,
) -> Result<(&'a str, &'a std::collections::HashMap<String, String>, &'a str), String> {
    if let Some(k) = key {
        let db = crate::config::resolve_db(config, k)
            .ok_or_else(|| format!("Unknown database: {}", k))?;
        Ok((db.ds_id(), &db.properties, db.name.as_str()))
    } else {
        // Fallback to archetype (e.g. "greatway" for projects)
        config.reservoir_by_archetype(fallback_archetype)
            .map(|(_, db)| (db.ds_id(), &db.properties, db.name.as_str()))
            .ok_or_else(|| format!("No database found for archetype '{}'", fallback_archetype))
    }
}

pub async fn execute(
    params: &StrategicParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    match params.analysis_type.as_str() {
        "overview" => {
            let mut overview = serde_json::json!({
                "analysis": "strategic_overview",
                "databases": {}
            });

            for (key, db) in &config.databases {
                let query = serde_json::json!({ "page_size": 1 });
                match notion.query_data_source(db.ds_id(), &query).await {
                    Ok(result) => {
                        overview["databases"][key] = serde_json::json!({
                            "name": db.name,
                            "accessible": true,
                            "has_entries": !result.results.is_empty(),
                            "has_more": result.has_more,
                            "archetype": db.archetype.as_deref().unwrap_or("unknown")
                        });
                    }
                    Err(_) => {
                        overview["databases"][key] = serde_json::json!({
                            "name": db.name,
                            "accessible": false,
                            "archetype": db.archetype.as_deref().unwrap_or("unknown")
                        });
                    }
                }
            }

            Ok(crate::toon_format::encode(&overview))
        }
        "project_health" => {
            let (ds_id, properties, _name) = resolve_for_analysis(
                config,
                params.project_database.as_deref(),
                "greatway",
            )?;

            let query = serde_json::json!({ "page_size": 50 });
            let result = notion.query_data_source(ds_id, &query).await?;

            let mut by_status: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            let mut projects: Vec<serde_json::Value> = Vec::new();

            let status_prop = properties.get("status").map(|s| s.as_str()).unwrap_or("Status");
            let progress_prop = properties.get("progress").map(|s| s.as_str()).unwrap_or("Progress");

            for page in &result.results {
                let title = crate::transform::extract_title(page);
                let status = crate::transform::extract_string(page, status_prop);
                let progress = crate::transform::extract_number(page, progress_prop)
                    .unwrap_or(0.0);
                *by_status.entry(status.clone()).or_insert(0) += 1;
                projects.push(serde_json::json!({
                    "title": title, "status": status, "progress": progress
                }));
            }

            let data = serde_json::json!({
                "analysis": "project_health",
                "total_projects": result.results.len(),
                "by_status": by_status,
                "projects": projects
            });
            Ok(crate::toon_format::encode(&data))
        }
        "okr_progress" => {
            let (ds_id, properties, _name) = resolve_for_analysis(
                config,
                params.okr_database.as_deref(),
                "greatway",
            )?;

            let query = serde_json::json!({ "page_size": 50 });
            let result = notion.query_data_source(ds_id, &query).await?;

            let status_prop = properties.get("status").map(|s| s.as_str()).unwrap_or("Status");
            let progress_prop = properties.get("progress").map(|s| s.as_str()).unwrap_or("Progress");
            let target_prop = properties.get("target").map(|s| s.as_str()).unwrap_or("Target");

            let mut okrs: Vec<serde_json::Value> = Vec::new();
            for page in &result.results {
                let title = crate::transform::extract_title(page);
                let status = crate::transform::extract_string(page, status_prop);
                let progress = crate::transform::extract_number(page, progress_prop);
                let target = crate::transform::extract_number(page, target_prop);
                okrs.push(serde_json::json!({
                    "title": title, "status": status,
                    "progress": progress, "target": target
                }));
            }

            let data = serde_json::json!({
                "analysis": "okr_progress",
                "total_okrs": okrs.len(),
                "okrs": okrs
            });
            Ok(crate::toon_format::encode(&data))
        }
        "alignment" => {
            // Check alignment by querying all reservoirs in the greater cycle + significator
            let mut data = serde_json::json!({ "analysis": "alignment" });

            let greater_keys = config.cycle_reservoirs("greater");
            let mut check_keys: Vec<String> = greater_keys;
            // Also include any reservoir with greatway or significator archetype
            for (key, db) in &config.databases {
                let arch = db.archetype.as_deref().unwrap_or("");
                if (arch == "greatway" || arch == "significator") && !check_keys.contains(key) {
                    check_keys.push(key.clone());
                }
            }

            for key in &check_keys {
                let db = match crate::config::resolve_db(config, key) {
                    Some(db) => db,
                    None => continue,
                };
                let ds_id = db.ds_id();
                let name = db.name.as_str();
                let query = serde_json::json!({ "page_size": 20 });
                if let Ok(result) = notion.query_data_source(ds_id, &query).await {
                    let items: Vec<serde_json::Value> = result.results.iter().map(|p| {
                        serde_json::json!({
                            "title": crate::transform::extract_title(p),
                            "status": crate::transform::extract_string(p, "Status")
                        })
                    }).collect();
                    data["databases"][key] = serde_json::json!({
                        "name": name, "items": items
                    });
                }
            }

            Ok(crate::toon_format::encode(&data))
        }
        "campaign_metrics" => {
            let (ds_id, properties, _name) = resolve_for_analysis(
                config,
                params.campaign_database.as_deref(),
                "greatway",
            )?;

            let query = serde_json::json!({ "page_size": 50 });
            let result = notion.query_data_source(ds_id, &query).await?;

            let status_prop = properties.get("status").map(|s| s.as_str()).unwrap_or("Status");
            let roi_prop = properties.get("roi").map(|s| s.as_str()).unwrap_or("ROI");
            let budget_prop = properties.get("budget").map(|s| s.as_str()).unwrap_or("Budget");
            let spent_prop = properties.get("spent").map(|s| s.as_str()).unwrap_or("Spent");

            let mut campaigns: Vec<serde_json::Value> = Vec::new();
            for page in &result.results {
                let title = crate::transform::extract_title(page);
                let status = crate::transform::extract_string(page, status_prop);
                let roi = crate::transform::extract_number(page, roi_prop);
                let budget = crate::transform::extract_number(page, budget_prop);
                let spent = crate::transform::extract_number(page, spent_prop);
                campaigns.push(serde_json::json!({
                    "title": title, "status": status,
                    "roi": roi, "budget": budget, "spent": spent
                }));
            }

            let data = serde_json::json!({
                "analysis": "campaign_metrics",
                "total_campaigns": campaigns.len(),
                "campaigns": campaigns
            });
            Ok(crate::toon_format::encode(&data))
        }
        _ => Err(format!("Unknown analysis type: {}", params.analysis_type)),
    }
}
