//! Health metrics tool — calculate G_z and P_z holonic health metrics

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

#[derive(Debug, Deserialize)]
pub struct HealthMetricsParams {
    /// Metric: "G_z" (lesser cycle), "P_z" (greater cycle), or "both"
    pub metric: String,
    /// Optional date range
    pub range: Option<String>,
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "metric": { "type": "string", "enum": ["G_z", "P_z", "both"], "description": "Health metric to calculate" },
            "range": { "type": "string", "description": "Date range: today, this_week, this_month, this_quarter" }
        },
        "required": ["metric"]
    })
}

pub async fn execute(
    params: &HealthMetricsParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let range = params.range.as_deref().unwrap_or("this_week");
    let date_filter = build_date_filter(range);

    let mut result = serde_json::json!({
        "analysis": "health_metrics",
        "range": range
    });

    match params.metric.as_str() {
        "G_z" => {
            let gz = calculate_g_z(config, notion, &date_filter).await;
            result["G_z"] = gz;
        }
        "P_z" => {
            let pz = calculate_p_z(config, notion, &date_filter).await;
            result["P_z"] = pz;
        }
        "both" => {
            let gz = calculate_g_z(config, notion, &date_filter).await;
            let pz = calculate_p_z(config, notion, &date_filter).await;
            let gz_score = gz.get("score").and_then(|v| v.as_f64()).unwrap_or(50.0);
            let pz_score = pz.get("score").and_then(|v| v.as_f64()).unwrap_or(50.0);

            result["G_z"] = gz;
            result["P_z"] = pz;
            result["total_health"] = serde_json::json!({
                "score": (gz_score + pz_score) / 2.0,
                "G_z": gz_score,
                "P_z": pz_score,
                "metabolic_status": metabolic_status(gz_score, pz_score)
            });
        }
        _ => return Err(format!("Unknown metric: {}", params.metric)),
    }

    Ok(crate::toon_format::encode(&result))
}

async fn calculate_g_z(
    config: &LifeOSConfig,
    notion: &NotionClient,
    date_filter: &Option<serde_json::Value>,
) -> serde_json::Value {
    let matrix = query_reservoir(config, notion, "matrix", date_filter).await;
    let potentiator = query_reservoir(config, notion, "potentiator", date_filter).await;

    let m_total = matrix.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let p_total = potentiator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let volume_balance = if m_total + p_total > 0.0 {
        1.0 - (m_total - p_total).abs() / (m_total + p_total)
    } else {
        0.5
    };

    let activity_volume = (m_total + p_total).min(30.0) / 30.0;

    let digestion_health = potentiator
        .get("digestion_distribution")
        .and_then(|d| d.as_object())
        .map(|obj| {
            let crystallized = obj.get("Crystallized").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let total: f64 = obj.values().filter_map(|v| v.as_f64()).sum();
            if total > 0.0 {
                crystallized / total
            } else {
                0.5
            }
        })
        .unwrap_or(0.5);

    let score = (volume_balance * 40.0 + activity_volume * 30.0 + digestion_health * 30.0).min(100.0);

    serde_json::json!({
        "score": score,
        "components": {
            "volume_balance": volume_balance * 100.0,
            "activity_volume": activity_volume * 100.0,
            "digestion_health": digestion_health * 100.0
        },
        "matrix_entries": m_total,
        "potentiator_entries": p_total,
        "interpretation": gz_interpretation(score)
    })
}

async fn calculate_p_z(
    config: &LifeOSConfig,
    notion: &NotionClient,
    date_filter: &Option<serde_json::Value>,
) -> serde_json::Value {
    let significator = query_reservoir(config, notion, "significator", date_filter).await;
    let greatway = query_reservoir(config, notion, "greatway", date_filter).await;
    let nexus = query_reservoir(config, notion, "nexus", date_filter).await;

    let s_total = significator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let g_total = greatway.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let n_total = nexus.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let strategic_ratio = if s_total > 0.0 {
        (g_total / s_total).min(5.0) / 5.0
    } else {
        0.0
    };

    let nexus_activity = n_total.min(20.0) / 20.0;
    let execution_volume = g_total.min(30.0) / 30.0;

    let score = (strategic_ratio * 40.0 + nexus_activity * 30.0 + execution_volume * 30.0).min(100.0);

    serde_json::json!({
        "score": score,
        "components": {
            "strategic_ratio": strategic_ratio * 100.0,
            "nexus_activity": nexus_activity * 100.0,
            "execution_volume": execution_volume * 100.0
        },
        "significator_entries": s_total,
        "greatway_entries": g_total,
        "nexus_entries": n_total,
        "interpretation": pz_interpretation(score)
    })
}

async fn query_reservoir(
    config: &LifeOSConfig,
    notion: &NotionClient,
    key: &str,
    date_filter: &Option<serde_json::Value>,
) -> serde_json::Value {
    let db = match crate::config::get_db(config, key) {
        Some(db) => db,
        None => return serde_json::json!({ "total": 0 }),
    };

    let mut query = serde_json::json!({ "page_size": 100 });
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

fn gz_interpretation(score: f64) -> &'static str {
    if score > 75.0 {
        "Excellent integrative coherence — the lesser cycle is metabolizing effectively"
    } else if score > 50.0 {
        "Good coherence — minor digestive inefficiencies present"
    } else if score > 30.0 {
        "Moderate coherence — the lesser cycle needs attention"
    } else {
        "Poor coherence — the lesser cycle is fragmented or overloaded"
    }
}

fn pz_interpretation(score: f64) -> &'static str {
    if score > 75.0 {
        "High evolutionary tension — strong drive toward restructuring"
    } else if score > 50.0 {
        "Moderate tension — steady evolutionary progress"
    } else if score > 30.0 {
        "Low tension — the greater cycle may be stalling"
    } else {
        "Minimal tension — stagnation risk, consider catalytic intervention"
    }
}

fn metabolic_status(gz: f64, pz: f64) -> &'static str {
    match (gz > 50.0, pz > 50.0) {
        (true, true) => "thriving — both cycles active and balanced",
        (true, false) => "stable but stagnant — good metabolism, low evolution",
        (false, true) => "aspirational but unstable — high drive, weak foundation",
        (false, false) => "crisis — both cycles need attention",
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
