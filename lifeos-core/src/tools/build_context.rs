//! `build_context` tool — assembles a complete relational neighborhood for a single entry.
//!
//! Read-only. Returns the entry + all outgoing relations + all incoming backlinks
//! + depth-2 neighborhood + gap analysis. One call replaces 3+ calls.
use std::collections::HashSet;

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::{NotionPage, PropertyValue};
use crate::util::schema_engine::SchemaCache;

#[derive(Debug, Deserialize)]
pub struct BuildContextParams {
    pub page_id: String,
    pub depth: Option<u32>,
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "page_id": { "type": "string", "description": "Notion page ID to build context for" },
            "depth": { "type": "integer", "minimum": 1, "maximum": 3, "description": "Neighborhood depth (default: 1, max: 3)" }
        },
        "required": ["page_id"]
    })
}

pub async fn execute(
    params: &BuildContextParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let depth = params.depth.unwrap_or(1).min(3);
    let page = notion.get_page(&params.page_id).await?;
    let title = crate::transform::extract_title(&page);

    // Determine which DB this page belongs to
    let parent = &page.parent;
    let parent = parent.as_ref().ok_or("Page has no parent")?;
    let ds_id = parent.data_source_id.as_deref()
        .or(parent.database_id.as_deref())
        .ok_or("Page has no data_source_id or database_id parent")?;
    let db_key = resolve_db_key_from_ds_id(config, ds_id)
        .ok_or_else(|| format!("Could not resolve DB for data_source_id {}", ds_id))?;

    // Get relation edges for this DB
    let rel_edges = schema_cache.get_relation_edges(&db_key);
    let rel_prop_names: Vec<String> = rel_edges.iter().map(|e| e.prop_name.clone()).collect();

    // 1. Outgoing relations
    let mut outgoing: Vec<serde_json::Value> = Vec::new();
    for prop_name in &rel_prop_names {
        if let Some(PropertyValue::Relation { relation, .. }) = page.properties.get(prop_name) {
            for rel_item in relation {
                if let Ok(target_page) = notion.get_page(&rel_item.id).await {
                    let target_title = crate::transform::extract_title(&target_page);
                    let target_parent = target_page.parent.as_ref();
                    let target_db = target_parent.and_then(|p| p.data_source_id.as_deref())
                        .or(target_parent.and_then(|p| p.database_id.as_deref()))
                        .and_then(|id| resolve_db_key_from_ds_id(config, id))
                        .unwrap_or_else(|| "unknown".to_string());
                    let target_et = extract_entry_type(&target_page, config, &target_db);
                    outgoing.push(serde_json::json!({
                        "property": prop_name,
                        "target_id": rel_item.id,
                        "target_title": target_title,
                        "target_db": target_db,
                        "target_entry_type": target_et,
                    }));
                }
            }
        }
    }

    // 2. Incoming backlinks — scan all DBs for pages that reference this page
    let mut incoming: Vec<serde_json::Value> = Vec::new();
    for scan_db_key in config.all_database_keys() {
        let scan_db = match crate::config::resolve_db(config, &scan_db_key) {
            Some(db) => db,
            None => continue,
        };
        let scan_edges = schema_cache.get_relation_edges(&scan_db_key);
        if scan_edges.is_empty() { continue; }

        let query = serde_json::json!({ "page_size": 100 });
        let result = match notion.query_data_source(scan_db.ds_id(), &query).await {
            Ok(r) => r,
            Err(_) => continue,
        };

        for scan_page in &result.results {
            for edge in scan_edges {
                if let Some(PropertyValue::Relation { relation, .. }) = scan_page.properties.get(&edge.prop_name) {
                    for rel_item in relation {
                        if rel_item.id == params.page_id {
                            let scan_title = crate::transform::extract_title(scan_page);
                            let scan_et = extract_entry_type(scan_page, config, &scan_db_key);
                            incoming.push(serde_json::json!({
                                "source_db": scan_db_key,
                                "source_id": scan_page.id,
                                "source_title": scan_title,
                                "source_entry_type": scan_et,
                                "via_property": edge.prop_name,
                            }));
                        }
                    }
                }
            }
        }
    }

    // 3. Gap analysis — which expected relations are missing?
    let populated_props: HashSet<String> = rel_prop_names.iter()
        .filter(|pn| {
            page.properties.get(*pn)
                .and_then(|v| match v {
                    PropertyValue::Relation { relation, .. } => Some(!relation.is_empty()),
                    _ => None,
                })
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    let gaps: Vec<String> = rel_prop_names.iter()
        .filter(|pn| !populated_props.contains(*pn))
        .cloned()
        .collect();

    // 4. Depth-2 neighborhood (if requested)
    let mut neighborhood: Vec<serde_json::Value> = Vec::new();
    if depth >= 2 {
        for out in &outgoing {
            let target_id = out["target_id"].as_str().unwrap_or("");
            if let Ok(target_page) = notion.get_page(target_id).await {
                let target_db = out["target_db"].as_str().unwrap_or("");
                let target_edges = schema_cache.get_relation_edges(target_db);
                for edge in target_edges {
                    if let Some(PropertyValue::Relation { relation, .. }) = target_page.properties.get(&edge.prop_name) {
                        for rel_item in relation {
                            if rel_item.id != params.page_id {
                                if let Ok(n2_page) = notion.get_page(&rel_item.id).await {
                                    let n2_title = crate::transform::extract_title(&n2_page);
                                    neighborhood.push(serde_json::json!({
                                        "via": format!("{} → {}", out["property"], edge.prop_name),
                                        "entry_id": rel_item.id,
                                        "entry_title": n2_title,
                                    }));
                                }
                            }
                        }
                    }
                }
            }
            // Limit depth-2 lookups to avoid excessive API calls
            if neighborhood.len() > 20 { break; }
        }
    }

    let entry_type = extract_entry_type(&page, config, &db_key);
    let entry_summary = serde_json::json!({
        "id": page.id,
        "title": title,
        "db": db_key,
        "entry_type": entry_type,
        "url": page.url,
    });

    let data = serde_json::json!({
        "build_context": {
            "entry": entry_summary,
            "outgoing_relations": outgoing,
            "incoming_relations": incoming,
            "neighborhood_depth_2": neighborhood,
            "gap_analysis": {
                "expected_relation_props": rel_prop_names,
                "populated_relation_props": populated_props.iter().collect::<Vec<_>>(),
                "missing_relation_props": gaps,
            },
            "summary": {
                "outgoing_count": outgoing.len(),
                "incoming_count": incoming.len(),
                "neighborhood_count": neighborhood.len(),
                "gap_count": gaps.len(),
            }
        }
    });

    Ok(crate::toon_format::encode(&data))
}

fn resolve_db_key_from_ds_id(config: &LifeOSConfig, ds_id: &str) -> Option<String> {
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

fn extract_entry_type(page: &NotionPage, config: &LifeOSConfig, db_key: &str) -> Option<String> {
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
