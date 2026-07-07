//! `cycle_health` tool — check if the v4.1 causal amplification cycle is running.

use std::sync::Arc;
use serde_json::{json, Value};
use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

pub fn schema() -> Value { json!({"type": "object", "properties": {}}) }

pub async fn execute(
    config: &Arc<LifeOSConfig>, notion: &Arc<NotionClient>, _sc: &SchemaCache,
) -> Result<String, String> {
    let mut report = String::new();
    report.push_str("LifeOS Cycle Health Check\n");
    report.push_str(&"=".repeat(60));
    report.push_str("\n\n");

    // 1. Pull Flow: Trajectory entries with Parent relation
    report.push_str("── PULL FLOW (Trajectory hierarchy) ──\n");
    let traj_db = crate::config::resolve_db(config, "trajectory").ok_or("Trajectory not found")?;
    let traj_resp = notion.query_data_source(traj_db.ds_id(), &json!({"page_size": 100})).await?;
    let total_traj = traj_resp.results.len();
    let with_parent = traj_resp.results.iter().filter(|p|
        p.properties.values().any(|v| matches!(v, crate::notion::types::PropertyValue::Relation { relation, .. } if !relation.is_empty()))
    ).count();
    let pull_health = if total_traj > 0 { (with_parent as f64 / total_traj as f64 * 100.0) as u64 } else { 0 };
    report.push_str(&format!("  Total Trajectory entries: {}\n", total_traj));
    report.push_str(&format!("  Entries with relations: {} ({}%)\n", with_parent, pull_health));
    report.push_str(&format!("  Status: {}\n", if pull_health > 30 { "✅ healthy" } else if pull_health > 10 { "⚠ weak" } else { "🔴 dormant" }));

    // 2. Ground Flow: Logbook entries with Synthesized Into
    report.push_str("\n── GROUND FLOW (Logbook → Synthesis) ──\n");
    let log_db = crate::config::resolve_db(config, "logbook").ok_or("Logbook not found")?;
    let log_resp = notion.query_data_source(log_db.ds_id(), &json!({"page_size": 100})).await?;
    let total_logs = log_resp.results.len();
    let with_synth = log_resp.results.iter().filter(|p|
        p.properties.get("Synthesized Into").map(|v| matches!(v, crate::notion::types::PropertyValue::Relation { relation, .. } if !relation.is_empty())).unwrap_or(false)
    ).count();
    let ground_health = if total_logs > 0 { (with_synth as f64 / total_logs as f64 * 100.0) as u64 } else { 0 };
    report.push_str(&format!("  Sampled Logbook entries: {}\n", total_logs));
    report.push_str(&format!("  With Synthesized Into links: {} ({}%)\n", with_synth, ground_health));
    report.push_str(&format!("  Status: {}\n", if ground_health > 10 { "✅ healthy" } else if ground_health > 1 { "⚠ weak" } else { "🔴 dormant — logs not synthesizing" }));

    // 3. Feedback Flow: Profile entries with Closes Gap For
    report.push_str("\n── FEEDBACK FLOW (Profile → Trajectory) ──\n");
    let prof_db = crate::config::resolve_db(config, "profile").ok_or("Profile not found")?;
    let prof_resp = notion.query_data_source(prof_db.ds_id(), &json!({"page_size": 100})).await?;
    let total_prof = prof_resp.results.len();
    let with_gap = prof_resp.results.iter().filter(|p|
        p.properties.get("Closes Gap For").map(|v| matches!(v, crate::notion::types::PropertyValue::Relation { relation, .. } if !relation.is_empty())).unwrap_or(false)
    ).count();
    let feedback_health = if total_prof > 0 { (with_gap as f64 / total_prof as f64 * 100.0) as u64 } else { 0 };
    report.push_str(&format!("  Profile entries: {}\n", total_prof));
    report.push_str(&format!("  With Closes Gap For links: {} ({}%)\n", with_gap, feedback_health));
    report.push_str(&format!("  Status: {}\n", if feedback_health > 10 { "✅ healthy" } else if feedback_health > 1 { "⚠ weak" } else { "🔴 dormant — gaps not informing trajectory" }));

    report.push_str(&format!("\n{}\n", "=".repeat(60)));
    report.push_str("Recommendations:\n");
    if pull_health < 30 { report.push_str("  • Link Trajectory entries to parents (use `link` or `quick_link`)\n"); }
    if ground_health < 10 { report.push_str("  • Synthesize Logbook entries into Synthesis (use `surface_synthesis`)\n"); }
    if feedback_health < 10 { report.push_str("  • Link Profile entries to Trajectory via Closes Gap For\n"); }
    if pull_health >= 30 && ground_health >= 10 && feedback_health >= 10 {
        report.push_str("  • Cycle is healthy! All 3 flows are active.\n");
    }

    Ok(report)
}
