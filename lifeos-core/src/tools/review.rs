//! Review pipeline tool — periodic reviews across all databases

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

/// Review parameters
#[derive(Debug, Deserialize)]
pub struct ReviewParams {
    /// Review type: daily, weekly, monthly, quarterly, journal
    pub review_type: String,
    /// Optional specific date for daily review
    pub date: Option<String>,
    /// Optional specific databases to review (comma-separated)
    pub databases: Option<String>,
}

/// Execute review pipeline

/// Generate JSON Schema for this tool
pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "review_type": { "type": "string", "enum": ["daily", "weekly", "monthly", "quarterly", "journal"], "description": "Review type" },
            "date": { "type": "string", "description": "Optional specific date for daily review (ISO format)" },
            "databases": { "type": "string", "description": "Optional specific databases to review (comma-separated)" }
        },
        "required": ["review_type"]
    })
}

pub async fn execute(
    params: &ReviewParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let (_date_filter, period_label, days_back) = match params.review_type.as_str() {
        "daily" => {
            let d = params.date.clone().unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
            (format!("on_or_after:{}", d), format!("Day: {}", &d[..10]), 1i64)
        }
        "weekly" => {
            let d = (chrono::Utc::now() - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
            (format!("on_or_after:{}", d), "This Week".into(), 7)
        }
        "monthly" => {
            let d = (chrono::Utc::now() - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
            (format!("on_or_after:{}", d), "This Month".into(), 30)
        }
        "quarterly" => {
            let d = (chrono::Utc::now() - chrono::Duration::days(90)).format("%Y-%m-%d").to_string();
            (format!("on_or_after:{}", d), "This Quarter".into(), 90)
        }
        "journal" => ("all".into(), "Journal Review".into(), 0),
        _ => return Err(format!("Unknown review type: {}", params.review_type)),
    };

    // Collect data from all relevant databases
    let mut review_data = serde_json::json!({
        "review_type": params.review_type,
        "period": period_label,
        "databases_reviewed": {}
    });

    // Filter databases if specified
    let db_filters: Vec<&str> = params.databases.as_ref()
        .map(|d| d.split(',').map(|s| s.trim()).collect())
        .unwrap_or_default();

    for (key, db) in &config.databases {
        if params.review_type == "journal" && db.agent != "journal" { continue; }
        if !db_filters.is_empty() && !db_filters.contains(&key.as_str()) { continue; }

        let since = (chrono::Utc::now() - chrono::Duration::days(days_back))
            .format("%Y-%m-%d").to_string();
        let date_prop = db.properties.get("date")
            .or_else(|| db.properties.get("action_date"))
            .or_else(|| db.properties.get("created_date"))
            .map(|s| s.as_str())
            .unwrap_or("Last edited time");
        let query = if params.review_type == "journal" {
            serde_json::json!({ "page_size": 50 })
        } else {
            serde_json::json!({
                "page_size": 50,
                "filter": { "property": date_prop, "date": { "on_or_after": since } }
            })
        };

        if let Ok(result) = notion.query_data_source(db.ds_id(), &query).await {
            let items: Vec<serde_json::Value> = result.results.iter().map(|p| {
                let title = crate::transform::extract_title(p);
                serde_json::json!({ "title": title, "id": p.id, "last_edited": &p.last_edited_time[..10] })
            }).collect();

            review_data["databases_reviewed"][key] = serde_json::json!({
                "count": items.len(),
                "items": items
            });
        }
    }

    Ok(crate::toon_format::encode(&review_data))
}
