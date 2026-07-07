//! `holonic_synthesis` tool — traces currency flow across the holonic spiral.
//!
//! Read-only. Shows how Catalyst → Experience → Transformation → Choice flows
//! through the 5 DBs. Identifies bottlenecks and recommends action areas.

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

#[derive(Debug, Deserialize)]
pub struct HolonicSynthesisParams {
    pub page_id: Option<String>,
    pub days_back: Option<u32>,
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "page_id": { "type": "string", "description": "Optional: trace flow from a specific entry" },
            "days_back": { "type": "integer", "minimum": 1, "maximum": 365, "description": "Days to look back (default: 7)" }
        }
    })
}

pub async fn execute(
    params: &HolonicSynthesisParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let days_back = params.days_back.unwrap_or(7);

    // If page_id is provided, trace from that entry
    if let Some(ref page_id) = params.page_id {
        return trace_from_entry(page_id, config, notion, schema_cache).await;
    }

    // Otherwise, do a time-range synthesis
    let mut output = String::new();
    output.push_str(&format!("Holonic Synthesis — Last {} days\n", days_back));
    output.push_str(&"=".repeat(80));
    output.push('\n');

    // Query each DB for recent entries and relation usage
    let mut db_stats: Vec<serde_json::Value> = Vec::new();
    let mut recommendations: Vec<String> = Vec::new();

    for db_key in ["matrix", "potentiator", "nexus", "significator", "greatway"] {
        let db = match crate::config::resolve_db(config, db_key) {
            Some(db) => db,
            None => continue,
        };
        let ds_id = db.ds_id();
        let rel_edges = schema_cache.get_relation_edges(db_key);

        let query = serde_json::json!({ "page_size": 100 });
        let result = match notion.query_data_source(ds_id, &query).await {
            Ok(r) => r,
            Err(e) => {
                output.push_str(&format!("{}: query failed: {}\n", db_key, e));
                continue;
            }
        };

        let total = result.results.len();
        let mut orphaned = 0;
        let mut rel_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for page in &result.results {
            let mut has_rel = false;
            for edge in rel_edges {
                if let Some(PropertyValue::Relation { relation, .. }) = page.properties.get(&edge.prop_name) {
                    if !relation.is_empty() {
                        has_rel = true;
                        *rel_counts.entry(edge.prop_name.clone()).or_default() += relation.len();
                    }
                }
            }
            if !has_rel {
                orphaned += 1;
            }
        }

        let orphan_pct = if total > 0 { orphaned * 100 / total } else { 0 };
        let status = if orphan_pct > 90 { "DORMANT" }
                     else if orphan_pct > 50 { "SPARSE" }
                     else if orphan_pct > 20 { "ACTIVE" }
                     else { "DENSE" };

        output.push_str(&format!("\n── {} ({}) ──\n", db_key.to_uppercase(), db.name));
        output.push_str(&format!("  Entries scanned: {}\n", total));
        output.push_str(&format!("  Orphaned (0 relations): {} ({}%)\n", orphaned, orphan_pct));
        output.push_str(&format!("  Status: {}\n", status));

        if !rel_counts.is_empty() {
            output.push_str("  Relation usage:\n");
            let mut sorted: Vec<_> = rel_counts.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            for (prop, count) in sorted.iter().take(5) {
                output.push_str(&format!("    {}: {} links\n", prop, count));
            }
        }

        if rel_edges.is_empty() {
            output.push_str("  No relation properties defined.\n");
        } else if rel_counts.is_empty() {
            output.push_str(&format!("  ⚠️  {} relation properties defined, ALL unused\n", rel_edges.len()));
        }

        db_stats.push(serde_json::json!({
            "db": db_key,
            "total": total,
            "orphaned": orphaned,
            "orphan_pct": orphan_pct,
            "status": status,
            "relation_props_defined": rel_edges.len(),
            "relation_props_used": rel_counts.len(),
        }));

        // Generate recommendations
        if orphan_pct > 90 && total > 10 {
            recommendations.push(format!(
                "{}: {} entries orphaned ({}%) — start with entries that have clear titles for matching",
                db_key, orphaned, orphan_pct
            ));
        }
    }

    // Currency flow analysis
    output.push_str("\n── CURRENCY FLOW ANALYSIS ──\n");

    // Check Nexus Kind distribution (currency transmutation)
    let nexus_db = crate::config::resolve_db(config, "nexus");
    if let Some(nexus) = nexus_db {
        let query = serde_json::json!({ "page_size": 100 });
        if let Ok(result) = notion.query_data_source(nexus.ds_id(), &query).await {
            let mut kind_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            for page in &result.results {
                if let Some(PropertyValue::Select { select, .. }) = page.properties.get("Kind") {
                    if let Some(ref sel) = *select {
                        *kind_counts.entry(sel.name.clone()).or_default() += 1;
                    }
                }
            }
            output.push_str("  Nexus currency distribution:\n");
            for kind in ["Catalyst", "Experience", "Transformation", "Choice"] {
                let count = kind_counts.get(kind).copied().unwrap_or(0);
                let indicator = if count > 0 { "✅" } else { "❌" };
                output.push_str(&format!("    {} {}: {} entries\n", indicator, kind, count));
            }

            // Identify bottleneck
            let catalyst_count = kind_counts.get("Catalyst").copied().unwrap_or(0);
            let experience_count = kind_counts.get("Experience").copied().unwrap_or(0);
            let transformation_count = kind_counts.get("Transformation").copied().unwrap_or(0);
            let choice_count = kind_counts.get("Choice").copied().unwrap_or(0);

            if catalyst_count > 0 && experience_count == 0 {
                output.push_str("\n  ⚠️  BOTTLENECK: Catalysts accumulating but no Experience being generated\n");
                output.push_str("     → Catalysts are not being digested into Experience (Matrix digestion dormant)\n");
                recommendations.push("Nexus bottleneck: Catalysts not being digested into Experience — review Matrix ingestion".to_string());
            }
            if experience_count > 0 && transformation_count == 0 {
                output.push_str("\n  ⚠️  BOTTLENECK: Experience accumulating but no Transformation firing\n");
                output.push_str("     → Significator not reaching threshold for Transformation\n");
            }
            if transformation_count > 0 && choice_count == 0 {
                output.push_str("\n  ⚠️  BOTTLENECK: Transformations firing but no Choice being emitted\n");
                output.push_str("     → Significator not committing to directional output\n");
            }
        }
    }

    // Recommendations
    output.push_str("\n── RECOMMENDATIONS (for user review) ──\n");
    if recommendations.is_empty() {
        output.push_str("  No critical gaps detected.\n");
    } else {
        for (i, rec) in recommendations.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, rec));
        }
    }

    let data = serde_json::json!({
        "holonic_synthesis": {
            "days_back": days_back,
            "db_stats": db_stats,
            "recommendations": recommendations,
            "report": output,
        }
    });

    Ok(crate::toon_format::encode(&data))
}

async fn trace_from_entry(
    page_id: &str,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let page = notion.get_page(page_id).await?;
    let title = crate::transform::extract_title(&page);
    let parent = &page.parent;
    let parent = parent.as_ref().ok_or("Page has no parent")?;
    let ds_id = parent.data_source_id.as_deref()
        .or(parent.database_id.as_deref())
        .ok_or("Page has no parent")?;
    let db_key = resolve_db_key(config, ds_id)
        .ok_or_else(|| format!("Could not resolve DB for {}", ds_id))?;

    let mut output = format!("Holonic Synthesis — Entry Trace\n{}\n\n", "=".repeat(80));
    output.push_str(&format!("Entry: \"{}\" ({})\n", title, db_key));
    output.push_str(&format!("Page ID: {}\n\n", page_id));

    // Trace outgoing relations
    output.push_str("Outgoing relations:\n");
    for (prop_name, prop_value) in &page.properties {
        if let PropertyValue::Relation { relation, .. } = prop_value {
            if !relation.is_empty() {
                output.push_str(&format!("  {} → {} target(s)\n", prop_name, relation.len()));
                for rel in relation.iter().take(3) {
                    if let Ok(target) = notion.get_page(&rel.id).await {
                        let target_title = crate::transform::extract_title(&target);
                        let target_parent = target.parent.as_ref();
                        let target_db = target_parent.and_then(|p| p.data_source_id.as_deref())
                            .or(target_parent.and_then(|p| p.database_id.as_deref()))
                            .and_then(|id| resolve_db_key(config, id))
                            .unwrap_or_else(|| "?".to_string());
                        output.push_str(&format!("    → \"{}\" ({})\n", truncate_str(&target_title, 50), target_db));
                    }
                }
            }
        }
    }

    let data = serde_json::json!({
        "holonic_synthesis": {
            "trace_from": page_id,
            "entry_title": title,
            "entry_db": db_key,
            "report": output,
        }
    });

    Ok(crate::toon_format::encode(&data))
}

fn resolve_db_key(config: &LifeOSConfig, ds_id: &str) -> Option<String> {
    for (key, db) in &config.databases {
        if db.database_id == ds_id {
            return Some(key.clone());
        }
        if let Some(ref resolved) = db.resolved_data_source_id {
            if resolved == ds_id {
                return Some(key.clone());
            }
        }
    }
    None
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max]) }
}
