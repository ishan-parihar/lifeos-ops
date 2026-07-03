//! `relational_graph` tool — high-level relational graph overview.
//!
//! Read-only. Shows the inter-DB relation structure as a visual tree,
//! with counts of actual links per DB pair. Helps AI agents and users
//! understand the LifeOS hierarchy at a glance.
use std::collections::HashMap;

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

#[derive(Debug, Deserialize)]
pub struct RelationalGraphParams {
    pub database: Option<String>,
    pub show_counts: Option<bool>,
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "database": { "type": "string", "enum": ["matrix", "potentiator", "nexus", "significator", "greatway"], "description": "Optional: focus on one DB's relations. Omit for full graph." },
            "show_counts": { "type": "boolean", "description": "Show actual link counts per relation (default: true)" }
        }
    })
}

pub async fn execute(
    params: &RelationalGraphParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let show_counts = params.show_counts.unwrap_or(true);

    // Build the relation graph structure from schema_cache
    let all_edges = schema_cache.all_relation_edges();

    // Build a matrix: source_db → target_db → [(prop_name, count)]
    let mut graph: HashMap<String, HashMap<String, Vec<(String, usize)>>> = HashMap::new();

    // Initialize all DB pairs
    for src in config.all_database_keys() {
        graph.entry(src.clone()).or_default();
    }

    // Count actual links per relation property
    for (src_db, edges) in all_edges {
        if let Some(ref filter_db) = params.database {
            if filter_db.as_str() != src_db.as_str() { continue; }
        }

        let db = match crate::config::resolve_db(config, &src_db) {
            Some(db) => db,
            None => continue,
        };
        let ds_id = db.ds_id();

        // Query entries to count relation usage
        let query = serde_json::json!({ "page_size": 100 });
        let result = match notion.query_data_source(ds_id, &query).await {
            Ok(r) => r,
            Err(_) => continue,
        };

        for edge in edges {
            let mut count = 0;
            for page in &result.results {
                if let Some(PropertyValue::Relation { relation, .. }) = page.properties.get(&edge.prop_name) {
                    count += relation.len();
                }
            }

            let target_db = edge.target_db.clone();
            graph
                .entry(src_db.clone())
                .or_default()
                .entry(target_db)
                .or_default()
                .push((edge.prop_name.clone(), count));
        }
    }

    // Build visual output
    let mut output = String::new();
    output.push_str("LifeOS Relational Graph — High-Level Overview\n");
    output.push_str(&"=".repeat(80));
    output.push('\n');

    // ASCII tree representation
    output.push_str("\n── INTER-DB HIERARCHY (relation properties → target DBs) ──\n\n");

    let db_order = ["matrix", "potentiator", "nexus", "significator", "greatway"];
    let db_display = |key: &str| -> String {
        match key {
            "matrix" => "Matrix (M)".to_string(),
            "potentiator" => "Potentiator (P)".to_string(),
            "nexus" => "Nexus (N)".to_string(),
            "significator" => "Significator (S)".to_string(),
            "greatway" => "GreatWay (G)".to_string(),
            other => other.to_string(),
        }
    };

    for src_db in &db_order {
        if let Some(ref filter_db) = params.database {
            if filter_db != src_db { continue; }
        }

        let targets = match graph.get(*src_db) {
            Some(t) => t,
            None => continue,
        };

        if targets.is_empty() {
            output.push_str(&format!("  {} → (no relations defined)\n", db_display(src_db)));
            continue;
        }

        output.push_str(&format!("  {}\n", db_display(src_db)));

        let mut sorted_targets: Vec<(&String, &Vec<(String, usize)>)> = targets.iter().collect();
        sorted_targets.sort_by(|a, b| a.0.cmp(b.0));

        for (i, (target_db, props)) in sorted_targets.iter().enumerate() {
            let is_last = i == sorted_targets.len() - 1;
            let prefix = if is_last { "    └──" } else { "    ├──" };
            output.push_str(&format!("{} → {}\n", prefix, db_display(target_db)));

            for (j, (prop_name, count)) in props.iter().enumerate() {
                let is_last_prop = j == props.len() - 1;
                let prop_prefix = if is_last { "        └──" } else { "        ├──" };
                if show_counts {
                    let indicator = if *count > 0 { "✅" } else { "⬜" };
                    output.push_str(&format!("{} {} {} ({} links)\n", prop_prefix, indicator, prop_name, count));
                } else {
                    output.push_str(&format!("{} {}\n", prop_prefix, prop_name));
                }
            }
        }
        output.push('\n');
    }

    // Summary stats
    output.push_str("── SUMMARY ──\n");
    let mut total_props = 0;
    let mut total_used = 0;
    let mut total_links = 0;

    for src_db in &db_order {
        if let Some(targets) = graph.get(*src_db) {
            for props in targets.values() {
                for (_, count) in props {
                    total_props += 1;
                    if *count > 0 { total_used += 1; }
                    total_links += count;
                }
            }
        }
    }

    output.push_str(&format!("  Total relation properties: {}\n", total_props));
    output.push_str(&format!("  Properties with links: {} ({}%)\n", total_used, if total_props > 0 { total_used * 100 / total_props } else { 0 }));
    output.push_str(&format!("  Total links across all DBs: {}\n", total_links));
    output.push_str(&format!("  Utilization: {:.1}%\n", if total_props > 0 { total_used as f64 / total_props as f64 * 100.0 } else { 0.0 }));

    // Build JSON structure
    let graph_json: Vec<serde_json::Value> = db_order.iter()
        .filter(|src_db| {
            if let Some(ref filter_db) = params.database {
                return filter_db == *src_db;
            }
            true
        })
        .filter_map(|src_db| {
            let targets = graph.get(*src_db)?;
            let target_json: Vec<serde_json::Value> = targets.iter()
                .map(|(target_db, props)| {
                    serde_json::json!({
                        "target_db": target_db,
                        "relations": props.iter().map(|(name, count)| {
                            serde_json::json!({ "property": name, "link_count": count })
                        }).collect::<Vec<_>>()
                    })
                })
                .collect();
            Some(serde_json::json!({
                "source_db": src_db,
                "targets": target_json,
            }))
        })
        .collect();

    let data = serde_json::json!({
        "relational_graph": {
            "show_counts": show_counts,
            "graph": graph_json,
            "summary": {
                "total_relation_properties": total_props,
                "properties_with_links": total_used,
                "utilization_pct": if total_props > 0 { total_used * 100 / total_props } else { 0 },
                "total_links": total_links,
            },
            "report": output,
        }
    });

    Ok(crate::toon_format::encode(&data))
}
