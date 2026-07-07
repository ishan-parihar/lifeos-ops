//! Workflow commands — combine multiple tools into user-friendly workflows.
//!
//! `daily` — runs relational_gaps + holonic_synthesis + recent entries in one call.
//! `dashboard` — shows orphan count, recent entries, top gaps, health metrics.

use std::sync::Arc;


use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

// ── daily ────────────────────────────────────────────────────────────────

pub fn schema_daily() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {},
        "description": "Run daily review: relational gaps + holonic synthesis + recent entries."
    })
}

pub async fn execute_daily(
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let mut output = String::new();
    output.push_str("LifeOS Daily Review\n");
    output.push_str(&"=".repeat(80));
    output.push('\n');

    // 1. Relational gaps summary
    output.push_str("\n── RELATIONAL GAPS ──\n");
    let gaps_params = crate::tools::relational_gaps::RelationalGapsParams {
        database: None,
        min_relations: Some(0),
        limit: Some(20),
    };
    match crate::tools::relational_gaps::execute(&gaps_params, config, notion, schema_cache).await {
        Ok(result) => {
            // Extract just the summary from the TOON-encoded result
            if let Ok(data) = crate::toon_format::decode(&result) {
                if let Some(gaps) = data.get("relational_gaps") {
                    output.push_str(&format!("  Total entries with 0 relations: {}\n",
                        gaps.get("total_gaps").unwrap_or(&serde_json::json!(0))));
                }
            }
        }
        Err(e) => output.push_str(&format!("  Error: {}\n", e)),
    }

    // 2. Holonic synthesis summary
    output.push_str("\n── HOLONIC SYNTHESIS ──\n");
    let synth_params = crate::tools::holonic_synthesis::HolonicSynthesisParams {
        page_id: None,
        days_back: Some(1),
    };
    match crate::tools::holonic_synthesis::execute(&synth_params, config, notion, schema_cache).await {
        Ok(result) => {
            if let Ok(data) = crate::toon_format::decode(&result) {
                if let Some(synth) = data.get("holonic_synthesis") {
                    if let Some(recs) = synth.get("recommendations").and_then(|r| r.as_array()) {
                        if recs.is_empty() {
                            output.push_str("  No critical bottlenecks detected.\n");
                        } else {
                            for (i, rec) in recs.iter().enumerate().take(3) {
                                output.push_str(&format!("  {}. {}\n", i + 1,
                                    rec.as_str().unwrap_or("unknown")));
                            }
                        }
                    }
                }
            }
        }
        Err(e) => output.push_str(&format!("  Error: {}\n", e)),
    }

    // 3. Recent entries (last 5 across all DBs)
    output.push_str("\n── RECENT ENTRIES (last 5 per DB) ──\n");
    for db_key in ["matrix", "potentiator", "nexus", "significator", "greatway"] {
        let db = match crate::config::resolve_db(config, db_key) {
            Some(db) => db,
            None => continue,
        };
        let query = serde_json::json!({
            "page_size": 5,
            "sorts": [{"property": "last_edited_time", "direction": "descending"}]
        });
        if let Ok(result) = notion.query_data_source(db.ds_id(), &query).await {
            let pages = result.results;
            if !pages.is_empty() {
                output.push_str(&format!("\n  {} ({} recent):\n", db_key.to_uppercase(), pages.len()));
                for page in pages.iter().take(5) {
                    let title = crate::transform::extract_title(page);
                    output.push_str(&format!("    - {}\n", &title[..title.len().min(60)]));
                }
            }
        }
    }

    let data = serde_json::json!({
        "daily": {
            "report": output,
        }
    });
    Ok(crate::toon_format::encode(&data))
}

// ── dashboard ────────────────────────────────────────────────────────────

pub fn schema_dashboard() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {},
        "description": "LifeOS dashboard: orphan count per DB, recent entries, top gaps, health metrics."
    })
}

pub async fn execute_dashboard(
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let mut output = String::new();
    output.push_str("LifeOS Dashboard\n");
    output.push_str(&"=".repeat(80));
    output.push('\n');

    // Per-DB summary: total entries, orphan count, top entry-type
    output.push_str("\n── DATABASE OVERVIEW ──\n");
    output.push_str(&format!("{:<15} {:<10} {:<10} {:<15} {}\n",
        "DB", "Entries", "Orphans", "Top Type", "Orphan %"));
    output.push_str(&"-".repeat(70));
    output.push('\n');

    let mut total_entries = 0usize;
    let mut total_orphans = 0usize;

    for db_key in ["matrix", "potentiator", "nexus", "significator", "greatway"] {
        let db = match crate::config::resolve_db(config, db_key) {
            Some(db) => db,
            None => continue,
        };
        let rel_edges = schema_cache.get_relation_edges(db_key);
        let query = serde_json::json!({ "page_size": 100 });
        let result = match notion.query_data_source(db.ds_id(), &query).await {
            Ok(r) => r,
            Err(_) => continue,
        };

        let total = result.results.len();
        let mut orphaned = 0;
        let mut et_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        let et_prop_name = config.databases.get(db_key)
            .and_then(|d| d.entry_type_property.clone())
            .unwrap_or_else(|| "Entry Type".to_string());

        for page in &result.results {
            let mut has_rel = false;
            for edge in rel_edges {
                if let Some(crate::notion::types::PropertyValue::Relation { relation, .. }) = page.properties.get(&edge.prop_name) {
                    if !relation.is_empty() {
                        has_rel = true;
                        break;
                    }
                }
            }
            if !has_rel {
                orphaned += 1;
            }

            // Count entry-types
            if let Some(crate::notion::types::PropertyValue::Select { select, .. }) = page.properties.get(&et_prop_name) {
                if let Some(ref sel) = *select {
                    *et_counts.entry(sel.name.clone()).or_default() += 1;
                }
            }
        }

        let orphan_pct = if total > 0 { orphaned * 100 / total } else { 0 };
        let top_et = et_counts.iter().max_by_key(|(_, c)| *c).map(|(k, _)| k.as_str()).unwrap_or("(none)");

        output.push_str(&format!("{:<15} {:<10} {:<10} {:<15} {}%\n",
            db_key, total, orphaned, top_et, orphan_pct));

        total_entries += total;
        total_orphans += orphaned;
    }

    let overall_orphan_pct = if total_entries > 0 { total_orphans * 100 / total_entries } else { 0 };
    output.push_str(&format!("\n  TOTAL: {} entries, {} orphans ({}% overall orphan rate)\n",
        total_entries, total_orphans, overall_orphan_pct));

    // Relational graph summary
    output.push_str("\n── RELATIONAL GRAPH SUMMARY ──\n");
    let graph_params = crate::tools::relational_graph::RelationalGraphParams {
        database: None,
        show_counts: Some(true),
    };
    match crate::tools::relational_graph::execute(&graph_params, config, notion, schema_cache).await {
        Ok(result) => {
            if let Ok(data) = crate::toon_format::decode(&result) {
                if let Some(graph) = data.get("relational_graph") {
                    if let Some(summary) = graph.get("summary") {
                        output.push_str(&format!("  Total relation properties: {}\n",
                            summary.get("total_relation_properties").unwrap_or(&serde_json::json!(0))));
                        output.push_str(&format!("  Properties with links: {}\n",
                            summary.get("properties_with_links").unwrap_or(&serde_json::json!(0))));
                        output.push_str(&format!("  Utilization: {}%\n",
                            summary.get("utilization_pct").unwrap_or(&serde_json::json!(0))));
                        output.push_str(&format!("  Total links: {}\n",
                            summary.get("total_links").unwrap_or(&serde_json::json!(0))));
                    }
                }
            }
        }
        Err(e) => output.push_str(&format!("  Error: {}\n", e)),
    }

    let data = serde_json::json!({
        "dashboard": {
            "total_entries": total_entries,
            "total_orphans": total_orphans,
            "overall_orphan_pct": overall_orphan_pct,
            "report": output,
        }
    });
    Ok(crate::toon_format::encode(&data))
}
