//! Shared reservoir query helper for tool modules.

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

/// Query a single reservoir database, applying date filter if available.
/// Returns total count, has_more, status_distribution, and digestion_distribution.
pub async fn query_reservoir(
    config: &LifeOSConfig,
    notion: &NotionClient,
    key: &str,
    date_filter: &Option<serde_json::Value>,
    page_size: u32,
) -> serde_json::Value {
    let db = match crate::config::get_db(config, key) {
        Some(db) => db,
        None => return serde_json::json!({ "total": 0 }),
    };

    let mut query = serde_json::json!({ "page_size": page_size });
    if let Some(ref filter) = date_filter {
        if let Some(date_prop) = db.properties.get("date") {
            let mut f = filter.clone();
            f["property"] = serde_json::json!(date_prop);
            query["filter"] = f;
        }
    }

    match notion.query_data_source(db.ds_id(), &query).await {
        Ok(result) => {
            let mut status_dist: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            let mut digestion_dist: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            for page in &result.results {
                let status = crate::transform::extract_string(page, "Status");
                *status_dist.entry(status).or_insert(0) += 1;
                let digestion = crate::transform::extract_string(page, "Digestion Status");
                if !digestion.is_empty() {
                    *digestion_dist.entry(digestion).or_insert(0) += 1;
                }
            }
            serde_json::json!({
                "total": result.results.len(),
                "has_more": result.has_more,
                "status_distribution": status_dist,
                "digestion_distribution": digestion_dist
            })
        }
        Err(_) => serde_json::json!({ "total": 0 }),
    }
}
