//! `relational_gaps` tool — surfaces entries with zero or sparse relations.
//!
//! Read-only. Shows what SHOULD be linked based on the ontology but isn't.
//! Groups by DB and entry-type, shows ontology-expected relations that are missing.

use std::collections::HashMap;
use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

#[derive(Debug, Deserialize)]
pub struct RelationalGapsParams {
    pub database: Option<String>,
    pub min_relations: Option<u32>,
    pub limit: Option<u32>,
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "database": { "type": "string", "enum": ["matrix", "potentiator", "nexus", "significator", "greatway"], "description": "Optional DB key. Omit to scan all 5." },
            "min_relations": { "type": "integer", "minimum": 0, "description": "Show entries with at most this many relations (default: 0 = orphans only)" },
            "limit": { "type": "integer", "minimum": 1, "maximum": 500, "description": "Max entries per DB (default: 100)" }
        }
    })
}

pub async fn execute(
    params: &RelationalGapsParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let min_rel = params.min_relations.unwrap_or(0) as usize;
    let limit = params.limit.unwrap_or(100).min(500) as u64;

    let db_keys: Vec<String> = if let Some(ref db) = params.database {
        if !config.databases.contains_key(db) {
            return Err(format!("Unknown database: {}", db));
        }
        vec![db.clone()]
    } else {
        config.all_database_keys()
    };

    let mut output = String::new();
    output.push_str("Relational Gaps Report\n");
    output.push_str(&"=".repeat(80));
    output.push('\n');
    output.push_str(&format!("Filter: entries with ≤{} relations\n\n", min_rel));

    let mut total_gaps = 0usize;

    for db_key in &db_keys {
        let db = match crate::config::resolve_db(config, db_key) {
            Some(db) => db,
            None => continue,
        };
        let ds_id = db.ds_id();

        // Get relation properties for this DB
        let rel_edges = schema_cache.get_relation_edges(db_key);
        let rel_prop_names: Vec<&str> = rel_edges.iter().map(|e| e.prop_name.as_str()).collect();

        // Query entries
        let query = serde_json::json!({ "page_size": limit });
        let result = match notion.query_data_source(ds_id, &query).await {
            Ok(r) => r,
            Err(e) => {
                output.push_str(&format!("{}: query failed: {}\n", db_key, e));
                continue;
            }
        };

        let mut gaps_by_entry_type: HashMap<String, Vec<(String, String, Vec<String>)>> = HashMap::new();
        let mut total_orphans = 0usize;
        let mut total_scanned = 0usize;

        for page in &result.results {
            total_scanned += 1;
            let mut rel_count = 0;
            let mut populated_props: Vec<String> = Vec::new();
            let mut empty_props: Vec<String> = Vec::new();

            for prop_name in &rel_prop_names {
                if let Some(PropertyValue::Relation { relation, .. }) = page.properties.get(*prop_name) {
                    if !relation.is_empty() {
                        rel_count += relation.len();
                        populated_props.push(prop_name.to_string());
                    } else {
                        empty_props.push(prop_name.to_string());
                    }
                }
            }

            if rel_count <= min_rel {
                total_orphans += 1;
                let title = crate::transform::extract_title(page);
                let entry_type = extract_entry_type(page, config, db_key);
                gaps_by_entry_type
                    .entry(entry_type.unwrap_or_else(|| "(uncategorized)".to_string()))
                    .or_default()
                    .push((page.id.clone(), title, empty_props));
            }
        }

        total_gaps += total_orphans;
        let db_name = config.databases.get(db_key).map(|d| d.name.clone()).unwrap_or_default();

        output.push_str(&format!("── {} ({}) — {}/{} entries with ≤{} relations ──\n",
            db_name, db_key, total_orphans, total_scanned, min_rel));

        for (entry_type, entries) in gaps_by_entry_type.iter() {
            output.push_str(&format!("  {} ({} entries with gaps):\n", entry_type, entries.len()));
            // Show expected relations (the empty props for this entry-type)
            if !entries.is_empty() {
                let sample = &entries[0];
                if !sample.2.is_empty() {
                    output.push_str(&format!("    Expected relations (empty): {}\n", sample.2.join(", ")));
                }
            }
            // Show first 5 entries
            for (id, title, _) in entries.iter().take(5).collect::<Vec<_>>() {
                output.push_str(&format!("      - {} ({})\n", truncate(&title, 60), &id[..8.min(id.len())]));
            }
            if entries.len() > 5 {
                output.push_str(&format!("      ... and {} more\n", entries.len() - 5));
            }
        }
        output.push('\n');
    }

    output.push_str(&format!("Total entries with gaps: {}\n", total_gaps));

    let data = serde_json::json!({
        "relational_gaps": {
            "total_gaps": total_gaps,
            "filter": format!("≤{} relations", min_rel),
            "report": output,
        }
    });

    Ok(crate::toon_format::encode(&data))
}

fn extract_entry_type(page: &crate::notion::types::NotionPage, config: &LifeOSConfig, db_key: &str) -> Option<String> {
    let et_prop = config.databases.get(db_key)
        .and_then(|db| db.entry_type_property.clone())
        .unwrap_or_else(|| "Entry Type".to_string());
    page.properties.get(&et_prop)
        .and_then(|v| match v {
            PropertyValue::Select { select, .. } => select.as_ref().map(|o| o.name.clone()),
            PropertyValue::MultiSelect { multi_select, .. } => {
                multi_select.first().map(|o| o.name.clone())
            }
            _ => None,
        })
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max]) }
}
