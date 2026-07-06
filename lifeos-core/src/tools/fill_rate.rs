//! `fill_rate` tool — audit property fill rates per DB.
//!
//! Scans a database and reports what percentage of entries have each property
//! populated. Properties with <5% fill after 30 days of use are YAGNI candidates
//! for deletion (per AGENTS.md §6.2).
//!
//! Read-only. Useful for data-driven cleanup: instead of guessing which properties
//! are unused, run this tool to see exactly which ones have low fill rates.

use std::sync::Arc;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

#[derive(Debug, Deserialize)]
pub struct FillRateParams {
    /// Database key to audit. Required.
    pub database: String,
    /// Max entries to sample (default 200). The Notion API caps at 100/page,
    /// so we paginate up to this limit.
    pub limit: Option<u32>,
    /// Fill-rate threshold (default 5.0). Properties below this % are flagged
    /// as YAGNI candidates.
    pub threshold: Option<f32>,
}

pub fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "database": { "type": "string", "description": "DB key to audit (matrix, potentiator, nexus, significator, greatway)" },
            "limit": { "type": "integer", "minimum": 1, "maximum": 1000, "description": "Max entries to sample (default 200)" },
            "threshold": { "type": "number", "minimum": 0, "maximum": 100, "description": "Fill-rate % threshold for YAGNI flagging (default 5.0)" }
        },
        "required": ["database"]
    })
}

/// Check if a property value is "populated" (non-empty).
fn is_populated(prop: &PropertyValue) -> bool {
    use crate::notion::types::PropertyValue::*;
    match prop {
        Title { title, .. } => !title.is_empty(),
        RichText { rich_text, .. } => !rich_text.is_empty(),
        Select { select, .. } => select.is_some(),
        Status { status, .. } => status.is_some(),
        MultiSelect { multi_select, .. } => !multi_select.is_empty(),
        Date { date, .. } => date.is_some(),
        Number { number, .. } => number.is_some(),
        Checkbox { checkbox, .. } => *checkbox, // true = populated
        People { people, .. } => !people.is_empty(),
        Relation { relation, .. } => !relation.is_empty(),
        Url { url, .. } => url.is_some(),
        Email { email, .. } => email.is_some(),
        PhoneNumber { phone_number, .. } => phone_number.is_some(),
        Formula { formula, .. } => {
            // Formula is "populated" if it produces a non-null result
            formula.string.is_some() || formula.number.is_some()
                || formula.boolean.is_some() || formula.date.is_some()
        }
        Rollup { rollup, .. } => {
            // Rollup is populated if it has any values
            rollup.array.as_ref().map(|v| !v.is_empty()).unwrap_or(false)
                || rollup.number.is_some()
                || rollup.string.is_some()
        }
        CreatedTime { .. } => true,  // Always populated
        LastEditedTime { .. } => true, // Always populated
        CreatedBy { .. } => true,
        LastEditedBy { .. } => true,
        Files { files, .. } => !files.is_empty(),
        UniqueId { unique_id, .. } => unique_id.is_some(),
        Button { .. } => false, // Buttons are actions, not data
    }
}

pub async fn execute(
    params: &FillRateParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let limit = params.limit.unwrap_or(200).min(1000) as u64;
    let threshold = params.threshold.unwrap_or(5.0);

    let db = crate::config::resolve_db(config, &params.database)
        .ok_or_else(|| format!("Unknown database: {}", params.database))?;
    let ds_id = db.ds_id().to_string();
    let db_name = db.name.clone();

    // Paginate through entries up to `limit`
    let mut all_pages = Vec::new();
    let mut cursor: Option<String> = None;
    while all_pages.len() < limit as usize {
        let mut body = json!({ "page_size": 100 });
        if let Some(c) = &cursor {
            body["start_cursor"] = json!(c);
        }
        let resp = notion.query_data_source(&ds_id, &body).await
            .map_err(|e| format!("Query failed: {}", e))?;
        let page_count = resp.results.len();
        all_pages.extend(resp.results);
        if !resp.has_more || page_count == 0 {
            break;
        }
        cursor = resp.next_cursor;
    }
    let total = all_pages.len();

    // Get the DB's property names from schema_cache
    let prop_names = schema_cache.get_db_property_names(&params.database)
        .ok_or_else(|| format!("No schema cached for database: {}", params.database))?;

    // Count populated entries per property
    let mut fill_counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for prop_name in &prop_names {
        fill_counts.insert(prop_name.clone(), 0);
    }

    for page in &all_pages {
        for (prop_name, prop_value) in &page.properties {
            if is_populated(prop_value) {
                *fill_counts.entry(prop_name.clone()).or_insert(0) += 1;
            }
        }
    }

    // Build report
    let mut report = String::new();
    report.push_str(&format!("LifeOS fill_rate — DB: {} ({})\n", db_name, params.database));
    report.push_str(&format!("Sampled: {} entries (limit was {})\n", total, limit));
    report.push_str(&format!("YAGNI threshold: <{}% fill\n\n", threshold));

    // Sort properties by fill rate ascending (lowest first = top YAGNI candidates)
    let mut sorted_props: Vec<(String, u64)> = fill_counts.into_iter().collect();
    sorted_props.sort_by_key(|(_, count)| *count);

    report.push_str(&format!("{:<40} {:>8} {:>10} {}\n", "Property", "Fill", "Rate %", "Status"));
    report.push_str(&("-".repeat(75) + "\n"));

    let mut yagni_count = 0u64;
    let mut low_count = 0u64;
    let mut healthy_count = 0u64;

    for (prop_name, count) in &sorted_props {
        let rate = if total > 0 { (*count as f64 / total as f64) * 100.0 } else { 0.0 };
        let status = if rate < threshold as f64 {
            yagni_count += 1;
            "🔴 YAGNI"
        } else if rate < 30.0 {
            low_count += 1;
            "🟡 low"
        } else {
            healthy_count += 1;
            "🟢 ok"
        };
        report.push_str(&format!("{:<40} {:>5}/{:<5} {:>8.1}% {}\n",
            prop_name, count, total, rate, status));
    }

    report.push_str(&format!("\n── Summary ──\n"));
    report.push_str(&format!("  Total properties:    {}\n", sorted_props.len()));
    report.push_str(&format!("  🔴 YAGNI (<{}% fill):  {}  ← candidates for deletion\n", threshold as i32, yagni_count));
    report.push_str(&format!("  🟡 Low ({}-30% fill):  {}  ← review usage\n", threshold as i32, low_count));
    report.push_str(&format!("  🟢 Healthy (>30%):    {}\n", healthy_count));

    if yagni_count > 0 {
        report.push_str(&format!("\n  YAGNI candidates (properties to consider deleting):\n"));
        for (prop_name, count) in &sorted_props {
            let rate = if total > 0 { (*count as f64 / total as f64) * 100.0 } else { 0.0 };
            if rate < threshold as f64 {
                report.push_str(&format!("    - {} ({:.1}% fill)\n", prop_name, rate));
            }
        }
        report.push_str("\n  Per AGENTS.md §6.2: 'If a property has 0% fill rate after 30 days, delete it.'\n");
        report.push_str("  Review each candidate's semantic value before deletion.\n");
    }

    Ok(report)
}
