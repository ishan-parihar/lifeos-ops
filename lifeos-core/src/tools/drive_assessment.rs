//! Drive assessment tool — evaluate Agency/Communion/Eros/Agape at each boundary

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

#[derive(Debug, Deserialize)]
pub struct DriveAssessmentParams {
    /// Boundary: "lesser" (Matrix⇌Potentiator), "greater" (Significator⇌GreatWay), or "both"
    pub boundary: String,
    /// Optional date range: "today", "this_week", "this_month"
    pub range: Option<String>,
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "boundary": { "type": "string", "enum": ["lesser", "greater", "both"], "description": "Boundary to assess" },
            "range": { "type": "string", "description": "Date range: today, this_week, this_month" }
        },
        "required": ["boundary"]
    })
}

pub async fn execute(
    params: &DriveAssessmentParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let range = params.range.as_deref().unwrap_or("this_week");
    let mut result = serde_json::json!({
        "analysis": "drive_assessment",
        "boundary": params.boundary,
        "range": range
    });

    let date_filter = build_date_filter(range);

    match params.boundary.as_str() {
        "lesser" => {
            // Assess Agency (A_z) vs Communion (C_z) at Matrix⇌Potentiator boundary
            let matrix_stats = query_reservoir_stats(config, notion, "matrix", &date_filter).await;
            let potentiator_stats = query_reservoir_stats(config, notion, "potentiator", &date_filter).await;

            let g_z = calculate_g_z(&matrix_stats, &potentiator_stats);

            result["lesser_boundary"] = serde_json::json!({
                "matrix": matrix_stats,
                "potentiator": potentiator_stats,
                "G_z_coherence": g_z,
                "drives": {
                    "Agency": format_assessment("Agency", g_z),
                    "Communion": format_assessment("Communion", g_z)
                }
            });
        }
        "greater" => {
            // Assess Eros (P_z) vs Agape at Significator⇌GreatWay boundary
            let significator_stats = query_reservoir_stats(config, notion, "significator", &date_filter).await;
            let greatway_stats = query_reservoir_stats(config, notion, "greatway", &date_filter).await;

            let p_z = calculate_p_z(&significator_stats, &greatway_stats);

            result["greater_boundary"] = serde_json::json!({
                "significator": significator_stats,
                "greatway": greatway_stats,
                "P_z_tension": p_z,
                "drives": {
                    "Eros": format_assessment("Eros", p_z),
                    "Agape": format_assessment("Agape", p_z)
                }
            });
        }
        "both" => {
            let matrix_stats = query_reservoir_stats(config, notion, "matrix", &date_filter).await;
            let potentiator_stats = query_reservoir_stats(config, notion, "potentiator", &date_filter).await;
            let significator_stats = query_reservoir_stats(config, notion, "significator", &date_filter).await;
            let greatway_stats = query_reservoir_stats(config, notion, "greatway", &date_filter).await;

            let g_z = calculate_g_z(&matrix_stats, &potentiator_stats);
            let p_z = calculate_p_z(&significator_stats, &greatway_stats);

            result["lesser_boundary"] = serde_json::json!({
                "matrix": matrix_stats,
                "potentiator": potentiator_stats,
                "G_z_coherence": g_z
            });
            result["greater_boundary"] = serde_json::json!({
                "significator": significator_stats,
                "greatway": greatway_stats,
                "P_z_tension": p_z
            });
            result["overall_health"] = serde_json::json!({
                "G_z": g_z,
                "P_z": p_z,
                "total_health": (g_z + p_z) / 2.0,
                "balance": if (g_z - p_z).abs() < 20.0 { "balanced" } else if g_z > p_z { "grounded" } else { "aspirational" }
            });
        }
        _ => return Err(format!("Unknown boundary: {}", params.boundary)),
    }

    Ok(crate::toon_format::encode(&result))
}

async fn query_reservoir_stats(
    config: &LifeOSConfig,
    notion: &NotionClient,
    reservoir_key: &str,
    date_filter: &Option<serde_json::Value>,
) -> serde_json::Value {
    let db = match crate::config::get_db(config, reservoir_key) {
        Some(db) => db,
        None => return serde_json::json!({ "error": "not found" }),
    };

    let mut query = serde_json::json!({ "page_size": 50 });
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
            for page in &result.results {
                let status = crate::transform::extract_string(page, "Status");
                *status_dist.entry(status).or_insert(0) += 1;
            }
            serde_json::json!({
                "total": result.results.len(),
                "has_more": result.has_more,
                "status_distribution": status_dist
            })
        }
        Err(e) => serde_json::json!({ "error": e }),
    }
}

fn calculate_g_z(matrix: &serde_json::Value, potentiator: &serde_json::Value) -> f64 {
    let m_count = matrix.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let p_count = potentiator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    // G_z: integrative coherence — balance between matrix and potentiator activity
    if m_count + p_count == 0.0 { return 50.0; }
    let balance = 1.0 - (m_count - p_count).abs() / (m_count + p_count);
    let volume_factor = (m_count + p_count).min(20.0) / 20.0;
    (balance * 60.0 + volume_factor * 40.0).min(100.0)
}

fn calculate_p_z(significator: &serde_json::Value, greatway: &serde_json::Value) -> f64 {
    let s_count = significator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let g_count = greatway.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    // P_z: transcendental tension — greatway activity relative to significator
    if s_count + g_count == 0.0 { return 50.0; }
    let activity_ratio = if s_count > 0.0 { g_count / s_count } else { 0.0 };
    let normalized = (activity_ratio.min(3.0) / 3.0) * 70.0 + 30.0;
    normalized.min(100.0)
}

fn format_assessment(drive: &str, score: f64) -> String {
    match drive {
        "Agency" if score > 70.0 => "Strong boundary resistance — healthy autonomy".to_string(),
        "Agency" if score > 40.0 => "Moderate boundary — some erosion possible".to_string(),
        "Agency" => "Weak boundary — high permeability to perturbation".to_string(),
        "Communion" if score > 70.0 => "Strong field conductance — deep coupling".to_string(),
        "Communion" if score > 40.0 => "Moderate coupling — room for deeper connection".to_string(),
        "Communion" => "Weak coupling — isolated from environment".to_string(),
        "Eros" if score > 70.0 => "High evolutionary tension — strong drive toward restructuring".to_string(),
        "Eros" if score > 40.0 => "Moderate tension — steady evolution".to_string(),
        "Eros" => "Low tension — stagnation risk".to_string(),
        "Agape" if score > 70.0 => "Strong integrative coherence — healthy metabolism".to_string(),
        "Agape" if score > 40.0 => "Moderate coherence — some digestive inefficiency".to_string(),
        "Agape" => "Weak coherence — overloaded or fragmented".to_string(),
        _ => "Assessment unavailable".to_string(),
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
        _ => None,
    }
}
