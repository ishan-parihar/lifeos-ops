//! Health metrics tool — calculate G_z, P_z, A_z, C_z holonic health metrics
//!
//! Per LifeOS_v4_Architecture.md §2.3:
//! - G_z (Agape): Integrative coherence — how well the holon maintains balance while metabolizing novelty
//! - P_z (Eros): Transcendental tension — how strongly the holon is polarized toward evolutionary restructuring
//! - A_z (Agency): Boundary resistance — protects the holon's state from perturbation
//! - C_z (Communion): Field conductance — enables coupling with environment
//!
//! G_z × P_z = Total Metabolic Health. Both metrics are required; neither alone is sufficient.

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

#[derive(Debug, Deserialize)]
pub struct HealthMetricsParams {
    /// Metric: "G_z", "P_z", "A_z", "C_z", or "all"
    pub metric: String,
    /// Optional date range
    pub range: Option<String>,
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "metric": { "type": "string", "enum": ["G_z", "P_z", "A_z", "C_z", "all"], "description": "Health metric: G_z (integrative coherence), P_z (evolutionary tension), A_z (boundary resistance), C_z (field conductance), all (complete metabolic health)" },
            "range": { "type": "string", "description": "Date range: today, this_week, this_month, this_quarter" }
        },
        "required": ["metric"]
    })
}

pub async fn execute(
    params: &HealthMetricsParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let range = params.range.as_deref().unwrap_or("this_week");
    let date_filter = crate::util::date_filter::build_date_filter(range, None);

    let mut result = serde_json::json!({
        "analysis": "health_metrics",
        "range": range
    });

    match params.metric.as_str() {
        "G_z" => {
            let gz = calculate_g_z(config, notion, schema_cache, &date_filter).await;
            result["G_z"] = gz;
        }
        "P_z" => {
            let pz = calculate_p_z(config, notion, schema_cache, &date_filter).await;
            result["P_z"] = pz;
        }
        "A_z" => {
            let az = calculate_a_z(config, notion, schema_cache, &date_filter).await;
            result["A_z"] = az;
        }
        "C_z" => {
            let cz = calculate_c_z(config, notion, schema_cache, &date_filter).await;
            result["C_z"] = cz;
        }
        "all" => {
            let gz = calculate_g_z(config, notion, schema_cache, &date_filter).await;
            let pz = calculate_p_z(config, notion, schema_cache, &date_filter).await;
            let az = calculate_a_z(config, notion, schema_cache, &date_filter).await;
            let cz = calculate_c_z(config, notion, schema_cache, &date_filter).await;

            let gz_score = gz.get("score").and_then(|v| v.as_f64()).unwrap_or(50.0);
            let pz_score = pz.get("score").and_then(|v| v.as_f64()).unwrap_or(50.0);
            let az_score = az.get("score").and_then(|v| v.as_f64()).unwrap_or(50.0);
            let cz_score = cz.get("score").and_then(|v| v.as_f64()).unwrap_or(50.0);

            result["G_z"] = gz;
            result["P_z"] = pz;
            result["A_z"] = az;
            result["C_z"] = cz;
            result["total_health"] = serde_json::json!({
                "metabolic_score": (gz_score + pz_score) / 2.0,
                "G_z": gz_score,
                "P_z": pz_score,
                "A_z": az_score,
                "C_z": cz_score,
                "metabolic_status": metabolic_status(gz_score, pz_score),
                "drive_balance": drive_balance_status(az_score, cz_score)
            });
        }
        _ => return Err(format!("Unknown metric: {}. Use G_z, P_z, A_z, C_z, or all.", params.metric)),
    }

    Ok(crate::toon_format::encode(&result))
}

// ── G_z: Integrative Coherence (Agape) ──────────────────────────────
//
// Measures how well the holon metabolizes novelty — the rate at which
// Catalyst entries in Potentiator get reflected as Experience in Matrix.
//
// Components:
// 1. Digestion rate: how many Potentiator entries have progressed beyond "Raw"
// 2. Reflection ratio: Matrix entry count relative to Potentiator (healthy = 0.5-1.0)
// 3. Cross-scale coherence: entries that span both reservoirs via relations

async fn calculate_g_z(
    config: &LifeOSConfig,
    notion: &NotionClient,
    _schema_cache: &SchemaCache,
    date_filter: &Option<serde_json::Value>,
) -> serde_json::Value {
    let matrix = match config.reservoir_by_archetype("matrix") {
        Some((k, _)) => crate::tools::shared::query_reservoir(config, notion, k, date_filter, 100).await,
        None => serde_json::json!({"total": 0}),
    };

    let potentiator = match config.reservoir_by_archetype("potentiator") {
        Some((k, _)) => crate::tools::shared::query_reservoir(config, notion, k, date_filter, 100).await,
        None => serde_json::json!({"total": 0}),
    };

    let m_total = matrix.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let p_total = potentiator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);

    // Component 1: Digestion rate — what fraction of Potentiator entries have been processed
    // "Raw" entries are unprocessed catalyst; "Crystallized" are fully digested
    let digestion_rate = potentiator
        .get("digestion_distribution")
        .and_then(|d| d.as_object())
        .map(|obj| {
            let raw = obj.get("Raw").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let crystallized = obj.get("Crystallized").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let total: f64 = obj.values().filter_map(|v| v.as_f64()).sum();
            if total > 0.0 {
                // Healthy digestion: few Raw, many Crystallized
                let raw_penalty = (raw / total) * 50.0; // 0-50 penalty for unprocessed
                let crystallized_bonus = (crystallized / total) * 50.0; // 0-50 bonus for processed
                (100.0 - raw_penalty + crystallized_bonus) / 2.0
            } else {
                50.0 // neutral when no data
            }
        })
        .unwrap_or(50.0);

    // Component 2: Reflection ratio — Matrix should reflect a healthy subset of Potentiator
    // If P >> M: catalyst is accumulating faster than it's being digested (overwhelmed)
    // If M >> P: Matrix is pulling more than Potentiator generates (healthy pull)
    // If M ≈ P: balanced (good)
    // If M ≈ 0 and P >> 0: no digestion happening (bad)
    let reflection_ratio = if p_total > 0.0 {
        let ratio = m_total / p_total;
        // Sweet spot: 0.3-1.0 (Matrix processes 30-100% of Potentiator volume)
        if ratio >= 0.3 && ratio <= 1.0 {
            80.0 + (1.0 - (ratio - 0.65).abs() / 0.35) * 20.0
        } else if ratio < 0.3 {
            // Under-digesting: Matrix is too small relative to Potentiator
            ratio / 0.3 * 60.0
        } else {
            // Over-digesting or stale Matrix (Matrix has more than Potentiator)
            (1.0 / ratio).min(1.0) * 70.0
        }
    } else if m_total > 0.0 {
        // Potentiator empty but Matrix has entries — stale experience with no new catalyst
        30.0
    } else {
        50.0 // both empty — neutral
    };

    // Component 3: Activity volume — more entries = more material to metabolize
    let activity_volume = ((m_total + p_total).min(50.0) / 50.0) * 100.0;

    // Weighted composite: digestion_rate is most important, then reflection, then volume
    let score = (digestion_rate * 0.45 + reflection_ratio * 0.35 + activity_volume * 0.20).min(100.0);

    serde_json::json!({
        "score": score,
        "components": {
            "digestion_rate": (digestion_rate * 10.0).round() / 10.0,
            "reflection_ratio": (reflection_ratio * 10.0).round() / 10.0,
            "activity_volume": (activity_volume * 10.0).round() / 10.0
        },
        "matrix_entries": m_total,
        "potentiator_entries": p_total,
        "interpretation": gz_interpretation(score, digestion_rate, reflection_ratio)
    })
}

// ── P_z: Transcendental Tension (Eros) ──────────────────────────────
//
// Measures how strongly the holon is polarized toward evolutionary
// restructuring. High P_z = strong drive toward transformation.
// Low P_z = stagnation risk.
//
// Components:
// 1. Strategic-execution tension: ratio of Significator to GreatWay activity
//    (Significator > GreatWay = high tension = aspirational)
//    (GreatWay > Significator = low tension = purely operational)
// 2. Nexus processing: how active the contact-boundary is
// 3. Evolutionary pressure: entries that bridge greater-cycle and lesser-cycle

async fn calculate_p_z(
    config: &LifeOSConfig,
    notion: &NotionClient,
    _schema_cache: &SchemaCache,
    date_filter: &Option<serde_json::Value>,
) -> serde_json::Value {
    let significator = match config.reservoir_by_archetype("significator") {
        Some((k, _)) => crate::tools::shared::query_reservoir(config, notion, k, date_filter, 100).await,
        None => serde_json::json!({"total": 0}),
    };

    let greatway = match config.reservoir_by_archetype("greatway") {
        Some((k, _)) => crate::tools::shared::query_reservoir(config, notion, k, date_filter, 100).await,
        None => serde_json::json!({"total": 0}),
    };

    let nexus = match config.reservoir_by_archetype("nexus") {
        Some((k, _)) => crate::tools::shared::query_reservoir(config, notion, k, date_filter, 100).await,
        None => serde_json::json!({"total": 0}),
    };

    let s_total = significator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let g_total = greatway.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let n_total = nexus.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);

    // Component 1: Strategic-execution tension
    // High tension = Significator entries significantly outnumber GreatWay entries
    // This means the holon has strong aspirational vision but hasn't fully committed
    // Low tension = GreatWay dominates = purely operational, no evolutionary drive
    let strategic_tension = if s_total > 0.0 {
        let ratio = g_total / s_total;
        // Sweet spot: 0.3-0.8 (Significator leads, GreatWay follows)
        // Tension increases as GreatWay grows relative to Significator
        if ratio <= 0.3 {
            // Very high tension — strong strategic vision, little execution
            // This is actually aspirational but not grounded
            60.0 + (0.3 - ratio) / 0.3 * 30.0
        } else if ratio <= 0.8 {
            // Balanced tension — healthy evolutionary pressure
            70.0 + (0.8 - ratio) / 0.5 * 30.0
        } else if ratio <= 2.0 {
            // Moderate tension — execution is catching up
            40.0 + (2.0 - ratio) / 1.2 * 30.0
        } else {
            // Low tension — over-execution, operational grind
            (3.0 / ratio).min(1.0) * 40.0
        }
    } else if g_total > 0.0 {
        // No Significator but active GreatWay = pure execution without vision
        15.0
    } else {
        50.0
    };

    // Component 2: Nexus activity — the contact-boundary transmutes all currencies
    // Active nexus = high transmutation = strong evolutionary processing
    let nexus_activity = if n_total > 0.0 {
        // More nexus entries = more transmutation happening
        (n_total.min(30.0) / 30.0) * 100.0
    } else {
        20.0 // dormant nexus = low evolutionary processing
    };

    // Component 3: Evolutionary pressure — combined activity of greater cycle + nexus
    // This captures the "drive" of the system
    let evolutionary_pressure = ((s_total + g_total + n_total).min(60.0) / 60.0) * 100.0;

    // Weighted composite: strategic tension is the primary driver
    let score = (strategic_tension * 0.50 + nexus_activity * 0.25 + evolutionary_pressure * 0.25).min(100.0);

    serde_json::json!({
        "score": score,
        "components": {
            "strategic_tension": (strategic_tension * 10.0).round() / 10.0,
            "nexus_activity": (nexus_activity * 10.0).round() / 10.0,
            "evolutionary_pressure": (evolutionary_pressure * 10.0).round() / 10.0
        },
        "significator_entries": s_total,
        "greatway_entries": g_total,
        "nexus_entries": n_total,
        "interpretation": pz_interpretation(score, strategic_tension, nexus_activity)
    })
}

// ── A_z: Agency (Boundary Resistance) ───────────────────────────────
//
// Measures how well the holon protects its state from perturbation.
// High A_z = strong autonomy, resistance to external disruption.
// Low A_z = high permeability, vulnerable to perturbation.
//
// Measured by: ratio of "stable" entries (with established status) across
// both intra-holonic databases (Matrix + Significator).

async fn calculate_a_z(
    config: &LifeOSConfig,
    notion: &NotionClient,
    _schema_cache: &SchemaCache,
    date_filter: &Option<serde_json::Value>,
) -> serde_json::Value {
    // Intra-holonic databases: Matrix (current-stage) + Significator (all-stage)
    let matrix = match config.reservoir_by_archetype("matrix") {
        Some((k, _)) => crate::tools::shared::query_reservoir(config, notion, k, date_filter, 100).await,
        None => serde_json::json!({"total": 0}),
    };

    let significator = match config.reservoir_by_archetype("significator") {
        Some((k, _)) => crate::tools::shared::query_reservoir(config, notion, k, date_filter, 100).await,
        None => serde_json::json!({"total": 0}),
    };

    let m_total = matrix.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let s_total = significator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total = m_total + s_total;

    if total == 0.0 {
        return serde_json::json!({
            "score": 50.0,
            "components": { "stability": 50.0, "autonomy": 50.0, "volume": 0.0 },
            "interpretation": "No intra-holonic data — cannot assess boundary resistance"
        });
    }

    // Component 1: Stability — fraction of entries with "stable" status
    // (Active, In Progress, Completed — not "Archived" or "Needs Review")
    let stable_statuses: Vec<&str> = vec!["Active", "In Progress", "Completed", "Active/On Track"];
    let stable_count: f64 = [matrix.clone(), significator.clone()].iter().filter_map(|db| {
        db.get("status_distribution")
            .and_then(|d| d.as_object())
            .map(|dist| {
                stable_statuses.iter()
                    .filter_map(|s| dist.get(*s).and_then(|v| v.as_f64()))
                    .sum::<f64>()
            })
    }).sum();

    let stability = (stable_count / total * 100.0).min(100.0);

    // Component 2: Autonomy — entries with defined structure (not just raw)
    // Entries with multiple properties filled = more structured = more autonomous
    let autonomy = if total > 0.0 {
        // Use total as a proxy for structured entries (entries that exist have some structure)
        // In practice, entries with more properties are more autonomous
        (total.min(30.0) / 30.0) * 80.0 + 20.0 // base 20 + up to 80 from volume
    } else {
        50.0
    };

    // Component 3: Volume — more entries = stronger boundary
    let volume = (total.min(50.0) / 50.0) * 100.0;

    let score = (stability * 0.40 + autonomy * 0.30 + volume * 0.30).min(100.0);

    serde_json::json!({
        "score": score,
        "components": {
            "stability": (stability * 10.0).round() / 10.0,
            "autonomy": (autonomy * 10.0).round() / 10.0,
            "volume": (volume * 10.0).round() / 10.0
        },
        "matrix_entries": m_total,
        "significator_entries": s_total,
        "interpretation": az_interpretation(score)
    })
}

// ── C_z: Communion (Field Conductance) ──────────────────────────────
//
// Measures how well the holon couples with its environment.
// High C_z = deep engagement with external world.
// Low C_z = isolation, disconnection from environment.
//
// Measured by: cross-reservoir relation density and diversity of
// status values across extra-holonic databases (Potentiator + GreatWay).

async fn calculate_c_z(
    config: &LifeOSConfig,
    notion: &NotionClient,
    _schema_cache: &SchemaCache,
    date_filter: &Option<serde_json::Value>,
) -> serde_json::Value {
    // Extra-holonic databases: Potentiator (current-stage) + GreatWay (all-stage)
    let potentiator = match config.reservoir_by_archetype("potentiator") {
        Some((k, _)) => crate::tools::shared::query_reservoir(config, notion, k, date_filter, 100).await,
        None => serde_json::json!({"total": 0}),
    };

    let greatway = match config.reservoir_by_archetype("greatway") {
        Some((k, _)) => crate::tools::shared::query_reservoir(config, notion, k, date_filter, 100).await,
        None => serde_json::json!({"total": 0}),
    };

    let p_total = potentiator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let g_total = greatway.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total = p_total + g_total;

    if total == 0.0 {
        return serde_json::json!({
            "score": 50.0,
            "components": { "diversity": 50.0, "engagement": 50.0, "volume": 0.0 },
            "interpretation": "No extra-holonic data — cannot assess field conductance"
        });
    }

    // Component 1: Status diversity — more distinct statuses = more engagement modes
    let mut all_statuses: std::collections::HashSet<String> = std::collections::HashSet::new();
    for source in [&potentiator, &greatway] {
        if let Some(dist) = source.get("status_distribution").and_then(|d| d.as_object()) {
            for key in dist.keys() {
                all_statuses.insert(key.clone());
            }
        }
    }
    let diversity = (all_statuses.len() as f64 / 5.0).min(1.0) * 100.0;

    // Component 2: Cross-scale engagement — having entries in both extra-holonic DBs
    let engagement = if p_total > 0.0 && g_total > 0.0 {
        // Both active = strong field conductance
        let balance = 1.0 - (p_total - g_total).abs() / (p_total + g_total);
        (balance * 60.0 + 40.0).min(100.0)
    } else if p_total > 0.0 || g_total > 0.0 {
        // Only one active = partial engagement
        40.0
    } else {
        20.0
    };

    // Component 3: Volume — more entries = stronger environmental coupling
    let volume = (total.min(50.0) / 50.0) * 100.0;

    let score = (diversity * 0.35 + engagement * 0.40 + volume * 0.25).min(100.0);

    serde_json::json!({
        "score": score,
        "components": {
            "diversity": (diversity * 10.0).round() / 10.0,
            "engagement": (engagement * 10.0).round() / 10.0,
            "volume": (volume * 10.0).round() / 10.0
        },
        "potentiator_entries": p_total,
        "greatway_entries": g_total,
        "unique_statuses": all_statuses.len(),
        "interpretation": cz_interpretation(score)
    })
}

// ── Interpretations ─────────────────────────────────────────────────

fn gz_interpretation(score: f64, digestion: f64, reflection: f64) -> String {
    let base = if score > 75.0 {
        "Excellent integrative coherence"
    } else if score > 50.0 {
        "Good coherence"
    } else if score > 30.0 {
        "Moderate coherence — the lesser cycle needs attention"
    } else {
        "Poor coherence — the lesser cycle is fragmented"
    };

    let detail = if digestion < 40.0 {
        " — Potentiator entries are accumulating without being digested into Matrix"
    } else if reflection < 40.0 {
        " — Matrix is not reflecting Potentiator activity adequately"
    } else {
        ""
    };

    format!("{}{}", base, detail)
}

fn pz_interpretation(score: f64, tension: f64, nexus_act: f64) -> String {
    let base = if score > 75.0 {
        "High evolutionary tension — strong drive toward restructuring"
    } else if score > 50.0 {
        "Moderate tension — steady evolutionary progress"
    } else if score > 30.0 {
        "Low tension — the greater cycle may be stalling"
    } else {
        "Minimal tension — stagnation risk, consider catalytic intervention"
    };

    let detail = if nexus_act < 30.0 {
        " — Nexus transmutation is low, currencies are not flowing between cycles"
    } else if tension < 30.0 {
        " — Execution dominates strategy, consider revisiting Significator"
    } else {
        ""
    };

    format!("{}{}", base, detail)
}

fn az_interpretation(score: f64) -> &'static str {
    if score > 75.0 {
        "Strong boundary resistance — healthy autonomy, state is well-protected"
    } else if score > 50.0 {
        "Moderate boundary — some erosion possible under sustained perturbation"
    } else if score > 30.0 {
        "Weak boundary — high permeability, vulnerable to external disruption"
    } else {
        "Minimal boundary — holon state is unprotected, consider consolidating"
    }
}

fn cz_interpretation(score: f64) -> &'static str {
    if score > 75.0 {
        "Strong field conductance — deep coupling with environment"
    } else if score > 50.0 {
        "Moderate conductance — room for deeper environmental engagement"
    } else if score > 30.0 {
        "Weak conductance — partially isolated from environment"
    } else {
        "Minimal conductance — holon is disconnected from its operating environment"
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

fn drive_balance_status(az: f64, cz: f64) -> &'static str {
    let diff = (az - cz).abs();
    if diff < 15.0 {
        "balanced — Agency and Communion are in healthy equilibrium"
    } else if az > cz {
        "autonomous — strong boundaries but risk of isolation"
    } else {
        "open — strong environmental coupling but risk of boundary erosion"
    }
}
