//! Data science tool — temporal analysis, trajectories, correlations

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

/// Data science parameters
#[derive(Debug, Deserialize)]
pub struct DataScienceParams {
    /// Analysis type: temporal, trajectories, correlations, weekday_profile, patterns
    pub analysis_type: String,
    /// Primary database to analyze
    pub database: String,
    /// Secondary database (for correlations)
    pub database_b: Option<String>,
    /// Number of days to look back (default: 30)
    pub days_back: Option<i64>,
    /// Property to analyze
    pub property: Option<String>,
    /// Metric property (for numerical analysis)
    pub metric_property: Option<String>,
}

/// Execute data science analysis

/// Generate JSON Schema for this tool
pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "analysis_type": { "type": "string", "enum": ["temporal", "trajectories", "correlations", "weekday_profile", "patterns"], "description": "Analysis type" },
            "database": { "type": "string", "description": "Primary database to analyze" },
            "database_b": { "type": "string", "description": "Secondary database for correlations" },
            "days_back": { "type": "integer", "description": "Days to look back (default: 30)" },
            "property": { "type": "string", "description": "Property to analyze" },
            "metric_property": { "type": "string", "description": "Metric property for numerical analysis" }
        },
        "required": ["analysis_type", "database"]
    })
}

pub async fn execute(
    params: &DataScienceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let days = params.days_back.unwrap_or(30);
    let since = (chrono::Utc::now() - chrono::Duration::days(days))
        .format("%Y-%m-%d").to_string();

    match params.analysis_type.as_str() {
        "temporal" => {
            let db = crate::config::get_db(config, &params.database)
                .ok_or_else(|| format!("Unknown database: {}", params.database))?;
            let date_prop = date_prop_for(db)?;
            let query = serde_json::json!({
                "page_size": 100,
                "filter": { "property": date_prop, "date": { "on_or_after": since } }
            });
            let result = notion.query_data_source(db.ds_id(), &query).await?;

            let mut by_date: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
            for page in &result.results {
                let d = crate::transform::extract_date(page, date_prop);
                if !d.is_empty() {
                    *by_date.entry(d[..10.min(d.len())].to_string()).or_insert(0) += 1;
                }
            }

            let data = serde_json::json!({
                "analysis": "temporal",
                "database": params.database,
                "days_analyzed": days,
                "total_entries": result.results.len(),
                "date_distribution": by_date.iter().map(|(d, c)| {
                    serde_json::json!({ "date": d, "count": c })
                }).collect::<Vec<_>>()
            });
            Ok(crate::toon_format::encode(&data))
        }
        "trajectories" => {
            let db = crate::config::get_db(config, &params.database)
                .ok_or_else(|| format!("Unknown database: {}", params.database))?;
            let metric = params.metric_property.as_deref()
                .or_else(|| db.properties.keys().find(|k| k.as_str() == "energy" || k.as_str() == "mood" || k.as_str() == "score").map(|s| s.as_str()))
                .ok_or("metric_property required for trajectories")?;
            let date_prop = date_prop_for(db)?;

            let query = serde_json::json!({
                "page_size": 100,
                "filter": { "property": date_prop, "date": { "on_or_after": since } }
            });
            let result = notion.query_data_source(db.ds_id(), &query).await?;

            let mut trajectory: Vec<serde_json::Value> = Vec::new();
            for page in &result.results {
                let d = crate::transform::extract_date(page, date_prop);
                let val = crate::transform::extract_number(page, metric);
                if !d.is_empty() && val.is_some() {
                    trajectory.push(serde_json::json!({ "date": d, metric: val, "title": crate::transform::extract_title(page) }));
                }
            }

            let data = serde_json::json!({
                "analysis": "trajectory",
                "database": params.database,
                "metric": metric,
                "data_points": trajectory.len(),
                "trajectory": trajectory
            });
            Ok(crate::toon_format::encode(&data))
        }
        "weekday_profile" => {
            let db = crate::config::get_db(config, &params.database)
                .ok_or_else(|| format!("Unknown database: {}", params.database))?;
            let date_prop = date_prop_for(db)?;
            let query = serde_json::json!({
                "page_size": 100,
                "filter": { "property": date_prop, "date": { "on_or_after": since } }
            });
            let result = notion.query_data_source(db.ds_id(), &query).await?;

            let mut profile: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            for page in &result.results {
                let d = crate::transform::extract_date(page, date_prop);
                if d.len() >= 10 {
                    if let Ok(naive) = chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d") {
                        let weekday = format!("{}", naive.format("%A"));
                        *profile.entry(weekday).or_insert(0) += 1;
                    }
                }
            }

            let data = serde_json::json!({
                "analysis": "weekday_profile",
                "database": params.database,
                "profile": profile
            });
            Ok(crate::toon_format::encode(&data))
        }
        _ => Err(format!("Unknown analysis type: {}", params.analysis_type)),
    }
}

/// Resolve a Notion date property name from config keys, or return an error.
fn date_prop_for(db: &crate::config::DbConfig) -> Result<&str, String> {
    db.properties.get("date")
        .or_else(|| db.properties.get("action_date"))
        .or_else(|| db.properties.get("created_date"))
        .map(|s| s.as_str())
        .ok_or_else(|| format!("No date property configured for '{}' — expected one of: date, action_date, created_date", db.name))
}
