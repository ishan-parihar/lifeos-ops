//! Drive assessment tool — evaluate Agency/Communion/Eros/Agape at each boundary
//!
//! Per LifeOS_v4_Architecture.md §1.4: "All four drives — Agency, Communion,
//! Eros, Agape — operate at BOTH contact boundaries."

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
    let date_filter = crate::util::date_filter::build_date_filter(range, None);

    // Query all 4 core reservoirs + nexus
    let matrix_stats = crate::tools::shared::query_reservoir(config, notion, "matrix", &date_filter, 50).await;
    let potentiator_stats = crate::tools::shared::query_reservoir(config, notion, "potentiator", &date_filter, 50).await;
    let significator_stats = crate::tools::shared::query_reservoir(config, notion, "significator", &date_filter, 50).await;
    let greatway_stats = crate::tools::shared::query_reservoir(config, notion, "greatway", &date_filter, 50).await;
    let nexus_stats = crate::tools::shared::query_reservoir(config, notion, "nexus", &date_filter, 50).await;

    // Calculate health metrics
    let g_z = calculate_g_z(&matrix_stats, &potentiator_stats);
    let p_z = calculate_p_z(&significator_stats, &greatway_stats, &nexus_stats);

    // Lesser boundary (Matrix ⇄ Potentiator via Nexus) — all 4 drives
    let lesser_agency = calculate_agency(&matrix_stats, &potentiator_stats);
    let lesser_communion = calculate_communion(&matrix_stats, &potentiator_stats);
    // Eros at lesser = catalyst pressure (how strongly potentiator drives change)
    let lesser_eros = calculate_eros_lesser(&matrix_stats, &potentiator_stats);
    // Agape at lesser = integrative coherence (how well matrix digests catalyst)
    let lesser_agape = g_z;

    // Greater boundary (Significator ⇄ GreatWay via Nexus) — all 4 drives
    let greater_agency = calculate_agency(&significator_stats, &greatway_stats);
    let greater_communion = calculate_communion(&significator_stats, &greatway_stats);
    // Eros at greater = evolutionary tension (how strongly greatway pushes restructuring)
    let greater_eros = calculate_eros_greater(&significator_stats, &greatway_stats, &nexus_stats);
    // Agape at greater = cross-scale coherence (how well significator integrates)
    let greater_agape = p_z;

    let mut result = serde_json::json!({
        "analysis": "drive_assessment",
        "boundary": params.boundary,
        "range": range,
        "lesser_boundary": {
            "matrix": matrix_stats,
            "potentiator": potentiator_stats,
            "G_z_coherence": g_z,
            "drives": {
                "Agency": { "score": lesser_agency, "assessment": format_assessment("Agency", lesser_agency) },
                "Communion": { "score": lesser_communion, "assessment": format_assessment("Communion", lesser_communion) },
                "Eros": { "score": lesser_eros, "assessment": format_assessment("Eros", lesser_eros) },
                "Agape": { "score": lesser_agape, "assessment": format_assessment("Agape", lesser_agape) }
            }
        },
        "greater_boundary": {
            "significator": significator_stats,
            "greatway": greatway_stats,
            "P_z_tension": p_z,
            "drives": {
                "Agency": { "score": greater_agency, "assessment": format_assessment("Agency", greater_agency) },
                "Communion": { "score": greater_communion, "assessment": format_assessment("Communion", greater_communion) },
                "Eros": { "score": greater_eros, "assessment": format_assessment("Eros", greater_eros) },
                "Agape": { "score": greater_agape, "assessment": format_assessment("Agape", greater_agape) }
            }
        },
        "nexus": nexus_stats,
        "overall_health": {
            "G_z": g_z,
            "P_z": p_z,
            "total_health": (g_z + p_z) / 2.0,
            "balance": if (g_z - p_z).abs() < 20.0 { "balanced" } else if g_z > p_z { "grounded" } else { "aspirational" }
        }
    });

    // Filter by requested boundary
    if params.boundary == "lesser" {
        result.as_object_mut().unwrap().remove("greater_boundary");
        result.as_object_mut().unwrap().remove("nexus");
    } else if params.boundary == "greater" {
        result.as_object_mut().unwrap().remove("lesser_boundary");
        result.as_object_mut().unwrap().remove("nexus");
    }

    Ok(crate::toon_format::encode(&result))
}


fn calculate_g_z(matrix: &serde_json::Value, potentiator: &serde_json::Value) -> f64 {
    let m_count = matrix.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let p_count = potentiator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    if m_count + p_count == 0.0 { return 50.0; }
    let balance = 1.0 - (m_count - p_count).abs() / (m_count + p_count);
    let volume_factor = (m_count + p_count).min(20.0) / 20.0;
    (balance * 60.0 + volume_factor * 40.0).min(100.0)
}

fn calculate_p_z(significator: &serde_json::Value, greatway: &serde_json::Value, nexus: &serde_json::Value) -> f64 {
    let s_count = significator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let g_count = greatway.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let n_count = nexus.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    if s_count + g_count == 0.0 { return 50.0; }
    let activity_ratio = if s_count > 0.0 { g_count / s_count } else { 0.0 };
    let strategic_score = (activity_ratio.min(3.0) / 3.0) * 40.0;
    let nexus_score = (n_count.min(20.0) / 20.0) * 30.0;
    let volume_score = ((s_count + g_count).min(30.0) / 30.0) * 30.0;
    strategic_score + nexus_score + volume_score
}

/// Agency (A_z): Boundary resistance — how well the holon protects its state from perturbation.
/// Measured by: ratio of "Active" entries (resisting change) to total entries.
fn calculate_agency(left: &serde_json::Value, right: &serde_json::Value) -> f64 {
    let l_total = left.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let r_total = right.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let l_active = left.get("status_distribution")
        .and_then(|d| d.get("Active"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let r_active = right.get("status_distribution")
        .and_then(|d| d.get("Active"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let total = l_total + r_total;
    if total == 0.0 { return 50.0; }
    let active_ratio = (l_active + r_active) / total;
    (active_ratio * 60.0 + (total.min(20.0) / 20.0) * 40.0).min(100.0)
}

/// Communion (C_z): Field conductance — how well the holon couples with its environment.
/// Measured by: diversity of status values (more diversity = more engagement).
fn calculate_communion(left: &serde_json::Value, right: &serde_json::Value) -> f64 {
    let mut all_statuses: std::collections::HashSet<String> = std::collections::HashSet::new();
    for source in [left, right] {
        if let Some(dist) = source.get("status_distribution").and_then(|d| d.as_object()) {
            for key in dist.keys() {
                all_statuses.insert(key.clone());
            }
        }
    }
    let l_total = left.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let r_total = right.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total = l_total + r_total;
    if total == 0.0 { return 50.0; }
    let diversity = all_statuses.len() as f64;
    let diversity_score = (diversity / 5.0).min(1.0);
    let volume_score = total.min(20.0) / 20.0;
    diversity_score * 50.0 + volume_score * 50.0
}

/// Eros at lesser boundary: catalyst pressure — how strongly the potentiator
/// pushes new catalyst toward the matrix. Measured by potentiator activity
/// relative to matrix (high potentiator = strong eros pressure).
fn calculate_eros_lesser(matrix: &serde_json::Value, potentiator: &serde_json::Value) -> f64 {
    let m_count = matrix.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let p_count = potentiator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    if m_count + p_count == 0.0 { return 50.0; }
    // High potentiator-to-matrix ratio = strong eros pressure
    let ratio = if m_count > 0.0 { p_count / m_count } else { p_count };
    (ratio.min(3.0) / 3.0 * 60.0 + ((m_count + p_count).min(20.0) / 20.0) * 40.0).min(100.0)
}

/// Eros at greater boundary: evolutionary tension — how strongly the operating
/// environment pushes restructuring. Measured by greatway+significator activity
/// relative to nexus processing.
fn calculate_eros_greater(significator: &serde_json::Value, greatway: &serde_json::Value, nexus: &serde_json::Value) -> f64 {
    let s_count = significator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let g_count = greatway.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let n_count = nexus.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total = s_count + g_count;
    if total == 0.0 { return 50.0; }
    // High execution (greatway) relative to nexus processing = unprocessed tension
    let tension_ratio = if n_count > 0.0 { g_count / n_count } else { g_count };
    (tension_ratio.min(3.0) / 3.0 * 50.0 + (total.min(20.0) / 20.0) * 50.0).min(100.0)
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


