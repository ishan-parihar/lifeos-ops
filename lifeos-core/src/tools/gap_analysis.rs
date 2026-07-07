//! `gap_analysis` tool — compare Profile vs Vision to show gaps.

use std::sync::Arc;
use serde_json::{json, Value};
use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

pub fn schema() -> Value { json!({"type": "object", "properties": {}}) }

fn get_text(props: &std::collections::HashMap<String, PropertyValue>, name: &str) -> String {
    match props.get(name) {
        Some(PropertyValue::RichText { rich_text, .. }) if !rich_text.is_empty() =>
            rich_text.first().and_then(|t| t.plain_text.clone()).unwrap_or_default(),
        _ => String::new(),
    }
}

fn get_select(props: &std::collections::HashMap<String, PropertyValue>, name: &str) -> String {
    match props.get(name) {
        Some(PropertyValue::Select { select, .. }) => select.as_ref().map(|s| s.name.clone()).unwrap_or_default(),
        _ => String::new(),
    }
}

pub async fn execute(
    config: &Arc<LifeOSConfig>, notion: &Arc<NotionClient>, _sc: &SchemaCache,
) -> Result<String, String> {
    let mut report = String::new();
    report.push_str("Gap Analysis: Profile vs Vision\n");
    report.push_str(&"=".repeat(60));
    report.push_str("\n\n");

    // Get Profile entries
    let prof_db = crate::config::resolve_db(config, "profile").ok_or("Profile not found")?;
    let prof_resp = notion.query_data_source(prof_db.ds_id(), &json!({"page_size": 100})).await?;

    let mut gaps = Vec::new();
    let mut on_track = Vec::new();

    for page in &prof_resp.results {
        let name = crate::transform::extract_title(page);
        let current = get_text(&page.properties, "Current Value");
        let target = get_text(&page.properties, "Target Value");
        let category = get_select(&page.properties, "Category");
        let trend = get_select(&page.properties, "Trend");

        if target.is_empty() { continue; }

        if current == target {
            on_track.push(format!("  ✅ {} ({}): {} = {}", name, category, current, target));
        } else {
            gaps.push(format!("  🔴 {} ({}): {} → {} ({})", name, category, current, target, trend));
        }
    }

    report.push_str(&format!("── GAPS ({} entries) ──\n", gaps.len()));
    if gaps.is_empty() {
        report.push_str("  No gaps detected.\n");
    } else {
        for g in &gaps { report.push_str(g); report.push('\n'); }
    }

    report.push_str(&format!("\n── ON TRACK ({} entries) ──\n", on_track.len()));
    if on_track.is_empty() {
        report.push_str("  Nothing on track yet.\n");
    } else {
        for o in &on_track { report.push_str(o); report.push('\n'); }
    }

    report.push_str(&format!("\n{}\n", "=".repeat(60)));
    report.push_str(&format!("Summary: {} gaps, {} on track out of {} profile entries\n",
        gaps.len(), on_track.len(), prof_resp.results.len()));

    Ok(report)
}
