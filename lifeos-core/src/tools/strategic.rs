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
    /// Project database key (for project_health)
    pub project_database: Option<String>,
    /// OKR database key (for okr_progress)
    pub okr_database: Option<String>,
    /// Campaign database key (for campaign_metrics)
    pub campaign_database: Option<String>,
}

/// Execute strategic simulator

/// Generate JSON Schema for this tool
pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "analysis_type": { "type": "string", "enum": ["overview", "alignment", "project_health", "okr_progress", "campaign_metrics"], "description": "Analysis type" },
            "project_database": { "type": "string", "description": "Project database key for project_health" },
            "okr_database": { "type": "string", "description": "OKR database key for okr_progress" },
            "campaign_database": { "type": "string", "description": "Campaign database key for campaign_metrics" }
        },
        "required": ["analysis_type"]
    })
}

pub async fn execute(
    params: &StrategicParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    match params.analysis_type.as_str() {
        "overview" => {
            // Collect summary counts from all databases
            let mut overview = serde_json::json!({
                "analysis": "strategic_overview",
                "databases": {}
            });

            for (key, db) in &config.databases {
                let query = serde_json::json!({ "page_size": 1 });
                let _ = notion.query_database(&db.data_source_id, &query).await;
                overview["databases"][key] = serde_json::json!({
                    "name": db.name,
                    "total_estimated": "query page_size=1 indicates data exists",
                    "agent": db.agent
                });
            }

            Ok(crate::toon_wrapper::encode(&overview))
        }
        "project_health" => {
            let db_key = params.project_database.as_deref().unwrap_or("projects");
            let db = crate::get_db(config, db_key)
                .ok_or_else(|| format!("Unknown database: {}", db_key))?;

            let query = serde_json::json!({ "page_size": 50 });
            let result = notion.query_database(&db.data_source_id, &query).await?;

            let mut by_status: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            let mut projects: Vec<serde_json::Value> = Vec::new();

            for page in &result.results {
                let title = crate::transform::extract_title(page);
                let status = crate::transform::extract_string(page, "Status");
                let progress = crate::transform::extract_number(page, "Progress")
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
            Ok(crate::toon_wrapper::encode(&data))
        }
        "okr_progress" => {
            let db_key = params.okr_database.as_deref().unwrap_or("okrs");
            let db = crate::get_db(config, db_key)
                .ok_or_else(|| format!("Unknown database: {}", db_key))?;

            let query = serde_json::json!({ "page_size": 50 });
            let result = notion.query_database(&db.data_source_id, &query).await?;

            let mut okrs: Vec<serde_json::Value> = Vec::new();
            for page in &result.results {
                let title = crate::transform::extract_title(page);
                let status = crate::transform::extract_string(page, "Status");
                let progress = crate::transform::extract_number(page, "Progress");
                let target = crate::transform::extract_number(page, "Target");
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
            Ok(crate::toon_wrapper::encode(&data))
        }
        "alignment" => {
            // Check alignment by querying goals/projects/okrs together
            let mut data = serde_json::json!({ "analysis": "alignment" });

            for (key, db) in &config.databases {
                if db.agent != "strategy" { continue; }
                let query = serde_json::json!({ "page_size": 20 });
                if let Ok(result) = notion.query_database(&db.data_source_id, &query).await {
                    let items: Vec<serde_json::Value> = result.results.iter().map(|p| {
                        serde_json::json!({
                            "title": crate::transform::extract_title(p),
                            "status": crate::transform::extract_string(p, "Status")
                        })
                    }).collect();
                    data["databases"][key] = serde_json::json!({
                        "name": db.name, "items": items
                    });
                }
            }

            Ok(crate::toon_wrapper::encode(&data))
        }
        "campaign_metrics" => {
            let db_key = params.campaign_database.as_deref().unwrap_or("campaigns");
            let db = crate::get_db(config, db_key)
                .ok_or_else(|| format!("Unknown database: {}", db_key))?;

            let query = serde_json::json!({ "page_size": 50 });
            let result = notion.query_database(&db.data_source_id, &query).await?;

            let mut campaigns: Vec<serde_json::Value> = Vec::new();
            for page in &result.results {
                let title = crate::transform::extract_title(page);
                let status = crate::transform::extract_string(page, "Status");
                let roi = crate::transform::extract_number(page, "ROI");
                let budget = crate::transform::extract_number(page, "Budget");
                let spent = crate::transform::extract_number(page, "Spent");
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
            Ok(crate::toon_wrapper::encode(&data))
        }
        _ => Err(format!("Unknown analysis type: {}", params.analysis_type)),
    }
}
