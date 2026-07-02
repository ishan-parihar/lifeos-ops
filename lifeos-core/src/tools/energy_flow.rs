//! Energy flow tool — trace currency flow across the holonic spiral
//!
//! Per LifeOS_v4_Architecture.md §2: "The 5 DBs form an energy-flow spiral —
//! a single continuous pathway through the polarity pairs, with bi-directional flow."
//!
//! This tool traces how currencies (Catalyst, Experience, Transformation, Choice)
//! flow between reservoirs through their relation properties, showing the spiral
//! in action rather than just listing entries per database.

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

#[derive(Debug, Deserialize)]
pub struct EnergyFlowParams {
    /// Scope: "lesser_cycle", "greater_cycle", "full_spiral", or specific reservoir
    pub scope: String,
    /// Currency to trace: "Catalyst", "Experience", "Transformation", "Choice", or "all"
    pub currency: Option<String>,
    /// Optional specific entry ID to trace across reservoirs
    pub entry_id: Option<String>,
    /// Limit per database (default 10)
    pub limit: Option<u32>,
    /// Show metabolism scores for each entry (default: false)
    pub show_metabolism: Option<bool>,
}

pub fn schema(config: &LifeOSConfig) -> serde_json::Value {
    let mut scopes: Vec<String> = vec!["lesser_cycle".into(), "greater_cycle".into(), "full_spiral".into()];
    for key in config.all_reservoir_keys() {
        scopes.push(key);
    }
    serde_json::json!({
        "type": "object",
        "properties": {
            "scope": { "type": "string", "enum": scopes, "description": "Scope: lesser_cycle, greater_cycle, full_spiral, or any reservoir key" },
            "currency": { "type": "string", "enum": ["Catalyst", "Experience", "Transformation", "Choice", "all"], "description": "Currency to trace (default: all)" },
            "entry_id": { "type": "string", "description": "Optional specific entry ID to trace across reservoirs" },
            "limit": { "type": "integer", "minimum": 1, "maximum": 50, "description": "Limit per database (default 10)" },
            "show_metabolism": { "type": "boolean", "description": "Include per-entry metabolism scores (default: false)" }
        },
        "required": ["scope"]
    })
}

pub async fn execute(
    params: &EnergyFlowParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    if let Some(ref entry_id) = params.entry_id {
        return trace_entry(entry_id, config, notion, schema_cache).await;
    }

    let limit = params.limit.unwrap_or(10);
    let currency = params.currency.as_deref().unwrap_or("all");
    let show_metabolism = params.show_metabolism.unwrap_or(false);

    // Determine which reservoirs to query
    let reservoir_keys: Vec<String> = match params.scope.as_str() {
        "lesser_cycle" => config.cycle_reservoirs("lesser"),
        "greater_cycle" => config.cycle_reservoirs("greater"),
        "full_spiral" => config.all_reservoir_keys(),
        other => {
            if config.databases.contains_key(other) {
                vec![other.to_string()]
            } else {
                return Err(format!("Unknown scope: {}. Use lesser_cycle, greater_cycle, full_spiral, or a reservoir key.", other));
            }
        }
    };

    // Collect entries from all reservoirs in scope
    let mut reservoir_entries: std::collections::HashMap<String, Vec<serde_json::Value>> = std::collections::HashMap::new();

    for res_key in &reservoir_keys {
        let (ds_id, archetype, currency_in, currency_out) = match crate::config::resolve_db(config, res_key) {
            Some(db) => {
                (db.ds_id().to_string(),
                 db.archetype.as_deref().unwrap_or("unknown").to_string(),
                 db.currency_in.as_deref().unwrap_or("?").to_string(),
                 db.currency_out.as_deref().unwrap_or("?").to_string())
            }
            None => continue,
        };

        let query = serde_json::json!({ "page_size": limit });
        if let Ok(page_result) = notion.query_data_source(&ds_id, &query).await {
            let items: Vec<serde_json::Value> = page_result.results.iter().map(|p| {
                let title = crate::transform::extract_title(p);
                let status = crate::transform::extract_string(p, "Status");
                let mut entry = serde_json::json!({
                    "id": p.id,
                    "title": title,
                    "status": status,
                    "currency_in": &currency_in,
                    "currency_out": &currency_out,
                });
                if show_metabolism {
                    entry["metabolism"] = score_entry_metabolism(p, &archetype);
                }
                entry
            }).collect();

            reservoir_entries.insert(res_key.clone(), items);
        }
    }

    // Find cross-reservoir flow paths via relation properties
    let flow_paths = find_flow_paths(config, notion, schema_cache, &reservoir_entries, &reservoir_keys, limit).await;

    // Build the spiral visualization
    let spiral = build_spiral_visualization(config, &reservoir_keys);

    // Filter by currency if specified
    let filtered_entries = if currency != "all" {
        reservoir_entries.iter().map(|(k, entries)| {
            let filtered: Vec<serde_json::Value> = entries.iter()
                .filter(|e| {
                    let ci = e.get("currency_in").and_then(|v| v.as_str()).unwrap_or("");
                    let co = e.get("currency_out").and_then(|v| v.as_str()).unwrap_or("");
                    ci == currency || co == currency
                })
                .cloned()
                .collect();
            (k.clone(), filtered)
        }).collect()
    } else {
        reservoir_entries
    };

    let mut result = serde_json::json!({
        "analysis": "energy_flow",
        "scope": params.scope,
        "currency_filter": currency,
        "spiral": spiral,
        "reservoirs": {},
        "flow_paths": flow_paths,
        "summary": {}
    });

    // Build reservoir summaries
    let mut total_entries = 0;
    let mut total_flows = 0;
    for (res_key, entries) in &filtered_entries {
        result["reservoirs"][res_key] = serde_json::json!({
            "entries": entries,
            "count": entries.len(),
        });
        total_entries += entries.len();
    }

    // Count flow paths
    if let Some(paths) = result["flow_paths"].as_array() {
        total_flows = paths.len();
    }

    result["summary"] = serde_json::json!({
        "total_entries": total_entries,
        "total_flow_paths": total_flows,
        "reservoirs_in_scope": reservoir_keys.len(),
        "spiral_health": if total_flows > 0 { "active" } else { "dormant — no cross-reservoir flows detected" }
    });

    Ok(crate::toon_format::encode(&result))
}

/// Find flow paths between reservoirs by following relation properties.
///
/// For each entry in a reservoir, check if any of its relation properties
/// point to entries in a different reservoir that's in the same cycle.
/// This reveals how currencies flow through the spiral.
async fn find_flow_paths(
    config: &LifeOSConfig,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
    reservoir_entries: &std::collections::HashMap<String, Vec<serde_json::Value>>,
    reservoir_keys: &[String],
    limit: u32,
) -> Vec<serde_json::Value> {
    let mut flow_paths: Vec<serde_json::Value> = Vec::new();

    // For each reservoir, get its outgoing relation edges
    for src_key in reservoir_keys {
        if !reservoir_entries.contains_key(src_key) {
            continue;
        }

        let edges = schema_cache.get_relation_edges(src_key);

        // For each relation edge, check if the target is in another reservoir in scope
        for edge in edges {
            if !reservoir_keys.contains(&edge.target_db) {
                continue; // target not in scope
            }

            // Query a few entries from the source to find actual relation links
            let ds_id = match crate::config::resolve_db(config, src_key) {
                Some(db) => db.ds_id().to_string(),
                None => continue,
            };

            let query = serde_json::json!({ "page_size": limit.min(5) });
            if let Ok(result) = notion.query_data_source(&ds_id, &query).await {
                for page in &result.results {
                    if let Some(PropertyValue::Relation { relation, .. }) = page.properties.get(&edge.prop_name) {
                        for rel in relation {
                            // Check if this related page is in the target reservoir
                            let target_ds_id = match crate::config::resolve_db(config, &edge.target_db) {
                                Some(db) => db.ds_id().to_string(),
                                None => continue,
                            };

                            // Fetch the target page to confirm it's in the target DB
                            if let Ok(target_page) = notion.get_page(&rel.id).await {
                                let target_parent_ds = target_page.parent.as_ref()
                                    .and_then(|p| p.data_source_id.as_deref());

                                if let Some(tp_ds) = target_parent_ds {
                                    if tp_ds == target_ds_id {
                                        let src_title = crate::transform::extract_title(page);
                                        let tgt_title = crate::transform::extract_title(&target_page);

                                        // Determine currency flow direction
                                        let src_currency_out = get_reservoir_currency_out(config, src_key);
                                        let _tgt_currency_in = get_reservoir_currency_in(config, &edge.target_db);

                                        flow_paths.push(serde_json::json!({
                                            "from": {
                                                "reservoir": src_key,
                                                "entry": src_title,
                                                "entry_id": page.id,
                                            },
                                            "to": {
                                                "reservoir": &edge.target_db,
                                                "entry": tgt_title,
                                                "entry_id": rel.id,
                                            },
                                            "via_property": edge.prop_name,
                                            "currency_flow": {
                                                "currency": src_currency_out,
                                                "direction": format!("{} → {}", src_key, edge.target_db),
                                            }
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    flow_paths
}

/// Score an entry's metabolism based on its properties and archetype.
///
/// Each entry is a wave-particle: a specific instance AND an energy-processing node.
/// The metabolism score indicates how well the entry is processing its primary currency.
fn score_entry_metabolism(page: &crate::notion::types::NotionPage, archetype: &str) -> serde_json::Value {
    let mut completeness = 0.0;
    let mut total_props = 0;

    // Count non-empty properties as a proxy for completeness
    for (_name, value) in &page.properties {
        total_props += 1;
        match value {
            PropertyValue::Title { title, .. } => {
                if !title.is_empty() { completeness += 1.0; }
            }
            PropertyValue::RichText { rich_text, .. } => {
                if !rich_text.is_empty() { completeness += 1.0; }
            }
            PropertyValue::Number { number, .. } => {
                if number.is_some() { completeness += 1.0; }
            }
            PropertyValue::Select { select, .. } => {
                if select.is_some() { completeness += 1.0; }
            }
            PropertyValue::Status { status, .. } => {
                if status.is_some() { completeness += 1.0; }
            }
            PropertyValue::Date { date, .. } => {
                if date.is_some() { completeness += 1.0; }
            }
            PropertyValue::Checkbox { .. } => {
                completeness += 1.0; // checkbox is always set
            }
            PropertyValue::Relation { .. } => {
                completeness += 1.0; // relation exists even if empty
            }
            _ => {}
        }
    }

    let completeness_pct = if total_props > 0 {
        (completeness / total_props as f64 * 100.0).round() / 100.0
    } else {
        0.0
    };

    // Count relations as a proxy for cross-reservoir connectivity
    let relation_count: usize = page.properties.values().filter(|v| {
        matches!(v, PropertyValue::Relation { relation, .. } if !relation.is_empty())
    }).count();

    // Archetype-specific scoring
    let archetype_bonus = match archetype {
        "matrix" => {
            // Matrix should have crystallized experience — status "Completed" or "Integrated"
            let status = crate::transform::extract_string(page, "Status");
            match status.as_str() {
                "Completed" | "Integrated" | "Active" => 20.0,
                "In Progress" => 10.0,
                _ => 5.0,
            }
        }
        "potentiator" => {
            // Potentiator should have raw catalyst — check digestion status
            let digestion = crate::transform::extract_string(page, "Digestion Status");
            match digestion.as_str() {
                "Raw" => 15.0,        // unprocessed but present
                "Processing" => 20.0,  // actively being digested
                "Crystallized" => 25.0, // fully digested
                _ => 10.0,
            }
        }
        "significator" => {
            // Significator should have strategic orientation
            let stage = crate::transform::extract_string(page, "Stage");
            if !stage.is_empty() && stage != "unknown" { 20.0 } else { 10.0 }
        }
        "greatway" => {
            // GreatWay should have execution status
            let status = crate::transform::extract_string(page, "Status");
            match status.as_str() {
                "Active" | "In Progress" => 20.0,
                "Completed" => 15.0,
                _ => 10.0,
            }
        }
        "nexus" => {
            // Nexus should have category and kind
            let cat = crate::transform::extract_string(page, "Category");
            let kind = crate::transform::extract_string(page, "Kind");
            let cat_score = if !cat.is_empty() && cat != "unknown" { 10.0 } else { 0.0 };
            let kind_score = if !kind.is_empty() && kind != "unknown" { 10.0 } else { 0.0 };
            cat_score + kind_score
        }
        _ => 10.0,
    };

    let score = (completeness_pct * 40.0 + (relation_count as f64 * 10.0).min(30.0) + archetype_bonus).min(100.0);

    serde_json::json!({
        "score": (score * 10.0).round() / 10.0,
        "completeness": completeness_pct,
        "relations": relation_count,
        "archetype_bonus": archetype_bonus,
        "interpretation": metabolism_interpretation(score, archetype)
    })
}

fn metabolism_interpretation(score: f64, archetype: &str) -> &'static str {
    let base = if score > 75.0 {
        "Well-metabolized"
    } else if score > 50.0 {
        "Partially metabolized"
    } else if score > 25.0 {
        "Under-metabolized"
    } else {
        "Stagnant"
    };

    match archetype {
        "matrix" if score < 30.0 => "Stagnant — not yet crystallized into experience",
        "potentiator" if score < 30.0 => "Stagnant — catalyst not being digested",
        "significator" if score < 30.0 => "Stagnant — identity pattern not evolving",
        "greatway" if score < 30.0 => "Stagnant — commitments not being executed",
        "nexus" if score < 30.0 => "Stagnant — transmutation not occurring",
        _ => base,
    }
}

/// Get the currency_out for a reservoir
fn get_reservoir_currency_out(config: &LifeOSConfig, key: &str) -> String {
    config.databases.get(key)
        .and_then(|db| db.currency_out.as_deref())
        .unwrap_or("?")
        .to_string()
}

/// Get the currency_in for a reservoir
fn get_reservoir_currency_in(config: &LifeOSConfig, key: &str) -> String {
    config.databases.get(key)
        .and_then(|db| db.currency_in.as_deref())
        .unwrap_or("?")
        .to_string()
}

/// Build a visual representation of the spiral structure
fn build_spiral_visualization(config: &LifeOSConfig, reservoir_keys: &[String]) -> serde_json::Value {
    let lesser = config.cycle_reservoirs("lesser");
    let greater = config.cycle_reservoirs("greater");
    let nexus_key = config.reservoir_by_archetype("nexus").map(|(k, _)| k.to_string());

    serde_json::json!({
        "structure": "holonic_spiral",
        "lesser_cycle": {
            "reservoirs": lesser.iter().map(|k| {
                let db = config.databases.get(k);
                serde_json::json!({
                    "key": k,
                    "archetype": db.and_then(|d| d.archetype.as_deref()).unwrap_or("unknown"),
                    "currency_in": db.and_then(|d| d.currency_in.as_deref()).unwrap_or("?"),
                    "currency_out": db.and_then(|d| d.currency_out.as_deref()).unwrap_or("?"),
                    "in_scope": reservoir_keys.contains(k),
                })
            }).collect::<Vec<_>>(),
            "metric": "G_z (integrative coherence)"
        },
        "contact_boundary": {
            "key": nexus_key.as_deref().unwrap_or("nexus"),
            "archetype": "nexus",
            "role": "transmutation — processes all 4 currencies",
            "in_scope": reservoir_keys.iter().any(|k| k.as_str() == nexus_key.as_deref().unwrap_or("nexus")),
        },
        "greater_cycle": {
            "reservoirs": greater.iter().map(|k| {
                let db = config.databases.get(k);
                serde_json::json!({
                    "key": k,
                    "archetype": db.and_then(|d| d.archetype.as_deref()).unwrap_or("unknown"),
                    "currency_in": db.and_then(|d| d.currency_in.as_deref()).unwrap_or("?"),
                    "currency_out": db.and_then(|d| d.currency_out.as_deref()).unwrap_or("?"),
                    "in_scope": reservoir_keys.contains(k),
                })
            }).collect::<Vec<_>>(),
            "metric": "P_z (evolutionary tension)"
        },
        "currencies": config.holonic.as_ref()
            .map(|h| serde_json::json!(h.currencies))
            .unwrap_or(serde_json::json!(["Catalyst", "Experience", "Transformation", "Choice"])),
    })
}

/// Trace a specific entry across reservoirs — find its owner, relations, and currency role
async fn trace_entry(
    entry_id: &str,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let page = notion.get_page(entry_id).await?;
    let title = crate::transform::extract_title(&page);

    // Determine which reservoir owns this page
    let parent_ds_id = page.parent.as_ref().and_then(|p| p.data_source_id.as_deref());
    let mut owner_reservoir = "unknown".to_string();
    let mut owner_archetype = "unknown".to_string();
    let mut currency_in = "?".to_string();
    let mut currency_out = "?".to_string();

    if let Some(ds_id) = parent_ds_id {
        for (key, db) in &config.databases {
            if ds_id == db.ds_id() {
                owner_reservoir = key.clone();
                owner_archetype = db.archetype.as_deref().unwrap_or("unknown").to_string();
                currency_in = db.currency_in.as_deref().unwrap_or("?").to_string();
                currency_out = db.currency_out.as_deref().unwrap_or("?").to_string();
                break;
            }
        }
    }

    // Get all relations from this entry
    let mut outgoing_flows: Vec<serde_json::Value> = Vec::new();
    let mut incoming_flows: Vec<serde_json::Value> = Vec::new();

    for (prop_name, prop_value) in &page.properties {
        if let PropertyValue::Relation { relation, .. } = prop_value {
            for rel in relation {
                if let Ok(target_page) = notion.get_page(&rel.id).await {
                    let target_title = crate::transform::extract_title(&target_page);
                    let target_ds_id = target_page.parent.as_ref()
                        .and_then(|p| p.data_source_id.as_deref());

                    // Find target reservoir
                    let mut target_reservoir = "unknown".to_string();
                    let mut target_archetype = "unknown".to_string();
                    if let Some(tds) = target_ds_id {
                        for (key, db) in &config.databases {
                            if tds == db.ds_id() {
                                target_reservoir = key.clone();
                                target_archetype = db.archetype.as_deref().unwrap_or("unknown").to_string();
                                break;
                            }
                        }
                    }

                    // Determine if this is a same-cycle or cross-cycle flow
                    let src_res_key = owner_reservoir.split("→").next().unwrap_or("unknown");
                    let src_cycle = config.databases.get(src_res_key)
                        .and_then(|db| db.cycle.as_deref())
                        .unwrap_or("unknown");
                    let tgt_res_key = target_reservoir.split("→").next().unwrap_or("unknown");
                    let tgt_cycle = config.databases.get(tgt_res_key)
                        .and_then(|db| db.cycle.as_deref())
                        .unwrap_or("unknown");

                    let flow_type = if src_cycle == tgt_cycle && src_cycle != "unknown" {
                        "intra-cycle"
                    } else if src_cycle != "unknown" && tgt_cycle != "unknown" {
                        "cross-cycle"
                    } else {
                        "unknown"
                    };

                    outgoing_flows.push(serde_json::json!({
                        "property": prop_name,
                        "target": {
                            "id": rel.id,
                            "title": target_title,
                            "reservoir": target_reservoir,
                            "archetype": target_archetype,
                        },
                        "flow_type": flow_type,
                    }));
                }
            }
        }
    }

    // Find backlinks — entries that reference this page
    let search_keys: Vec<String> = config.all_reservoir_keys();
    for db_key in &search_keys {
        let ds_id = match crate::config::resolve_db(config, db_key) {
            Some(db) => db.ds_id().to_string(),
            None => continue,
        };
        let query = serde_json::json!({ "page_size": 20 });
        if let Ok(result) = notion.query_data_source(&ds_id, &query).await {
            for page in &result.results {
                for (prop_name, prop_value) in &page.properties {
                    if let PropertyValue::Relation { relation, .. } = prop_value {
                        if relation.iter().any(|r| r.id == entry_id) {
                            let bl_title = crate::transform::extract_title(page);
                            incoming_flows.push(serde_json::json!({
                                "from": {
                                    "id": page.id,
                                    "title": bl_title,
                                    "reservoir": db_key,
                                },
                                "via_property": prop_name,
                            }));
                        }
                    }
                }
            }
        }
    }

    // Metabolism score
    let metabolism = score_entry_metabolism(&page, &owner_archetype);

    let result = serde_json::json!({
        "analysis": "entry_trace",
        "entry": {
            "id": entry_id,
            "title": title,
            "reservoir": owner_reservoir,
            "archetype": owner_archetype,
            "currency_in": currency_in,
            "currency_out": currency_out,
        },
        "metabolism": metabolism,
        "outgoing_flows": outgoing_flows,
        "outgoing_count": outgoing_flows.len(),
        "incoming_flows": incoming_flows,
        "incoming_count": incoming_flows.len(),
        "spiral_position": format!(
            "This entry lives in {} ({}) — it processes {} and generates {}",
            owner_reservoir, owner_archetype,
            currency_in, currency_out
        ),
    });

    Ok(crate::toon_format::encode(&result))
}
