//! `surface_synthesis` tool — scan recent Logbook entries for patterns.

use std::sync::Arc;
use std::collections::HashMap;
use serde_json::{json, Value};
use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

pub fn schema() -> Value { json!({"type": "object", "properties": {}}) }

pub async fn execute(
    config: &Arc<LifeOSConfig>, notion: &Arc<NotionClient>, _sc: &SchemaCache,
) -> Result<String, String> {
    let mut report = String::new();
    report.push_str("Synthesis Surfacing — Pattern Detection\n");
    report.push_str(&"=".repeat(60));
    report.push_str("\n\n");

    let log_db = crate::config::resolve_db(config, "logbook").ok_or("Logbook not found")?;
    let et_prop = log_db.entry_type_property.clone().unwrap_or_else(|| "Entry Type".to_string());

    // Get recent 50 logs
    let resp = notion.query_data_source(log_db.ds_id(), &json!({
        "page_size": 50,
        "sorts": [{"property": "Date", "direction": "descending"}]
    })).await?;

    // Group by entry type
    let mut by_type: HashMap<String, Vec<String>> = HashMap::new();
    for page in &resp.results {
        let title = crate::transform::extract_title(page);
        let entry_type = page.properties.get(&et_prop)
            .and_then(|v| match v {
                PropertyValue::Select { select, .. } => select.as_ref().map(|s| s.name.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "unknown".to_string());
        by_type.entry(entry_type).or_default().push(title);
    }

    report.push_str(&format!("Scanned {} recent Logbook entries.\n\n", resp.results.len()));

    // Analyze each type
    for (et, entries) in by_type.iter() {
        report.push_str(&format!("── {} ({} entries) ──\n", et, entries.len()));

        if entries.len() >= 5 {
            report.push_str(&format!("  Pattern: High {} frequency ({} entries in recent 50)\n", et, entries.len()));
            match et.as_str() {
                "Activity" => report.push_str("  Suggestion: Create an Opportunity entry — capitalize on activity momentum\n"),
                "Financial" => report.push_str("  Suggestion: Create a Directive entry — review spending patterns\n"),
                "Subjective" => report.push_str("  Suggestion: Create a Note entry — synthesize subjective insights\n"),
                "Relational" => report.push_str("  Suggestion: Create an Opportunity entry — relational patterns detected\n"),
                "Diet" => report.push_str("  Suggestion: Create a Directive entry — review diet patterns\n"),
                _ => {}
            }
        }

        // Show recent titles
        for title in entries.iter().take(3) {
            report.push_str(&format!("  • {}\n", title));
        }
        if entries.len() > 3 {
            report.push_str(&format!("  ... and {} more\n", entries.len() - 3));
        }
        report.push('\n');
    }

    report.push_str(&"=".repeat(60));
    report.push_str("\nTo create a synthesis entry, use: mutate --operation create --database synthesis --properties '{\"Name\":\"...\",\"Category\":\"Opportunity\"}'\n");

    Ok(report)
}
