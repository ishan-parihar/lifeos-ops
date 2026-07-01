//! Shared reservoir query helper for tool modules.

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

/// Query a single reservoir database, applying date filter if available.
/// Returns total count, has_more, status_distribution, and digestion_distribution.
/// Accepts both reservoir and satellite keys via resolve_db.
pub async fn query_reservoir(
    config: &LifeOSConfig,
    notion: &NotionClient,
    key: &str,
    date_filter: &Option<serde_json::Value>,
    page_size: u32,
) -> serde_json::Value {
    let db = match crate::config::resolve_db(config, key) {
        Some(crate::config::ResolvedDb::Reservoir(_k, db)) => db,
        Some(crate::config::ResolvedDb::Satellite(_rk, _sk, sat)) => {
            return query_ds(notion, sat.ds_id(), &sat.properties, date_filter, page_size).await;
        }
        None => return serde_json::json!({ "total": 0 }),
    };
    query_ds(notion, db.ds_id(), &db.properties, date_filter, page_size).await
}

async fn query_ds(
    notion: &NotionClient,
    ds_id: &str,
    properties: &std::collections::HashMap<String, String>,
    date_filter: &Option<serde_json::Value>,
    page_size: u32,
) -> serde_json::Value {
    let mut query = serde_json::json!({ "page_size": page_size });
    if let Some(ref filter) = date_filter {
        if let Some(date_prop) = properties.get("date") {
            let mut f = filter.clone();
            f["property"] = serde_json::json!(date_prop);
            query["filter"] = f;
        }
    }

    match notion.query_data_source(ds_id, &query).await {
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
