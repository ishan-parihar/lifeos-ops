//! Energy flow tool — trace currency flow across the holonic spiral

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
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
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "scope": { "type": "string", "enum": ["lesser_cycle", "greater_cycle", "full_spiral", "matrix", "potentiator", "significator", "greatway", "nexus"], "description": "Scope of energy flow analysis" },
            "currency": { "type": "string", "enum": ["Catalyst", "Experience", "Transformation", "Choice", "all"], "description": "Currency to trace (default: all)" },
            "entry_id": { "type": "string", "description": "Optional specific entry ID to trace across reservoirs" },
            "limit": { "type": "integer", "minimum": 1, "maximum": 50, "description": "Limit per database (default 10)" }
        },
        "required": ["scope"]
    })
}

pub async fn execute(
    params: &EnergyFlowParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    // If tracing a specific entry, fetch it and show its relations
    if let Some(ref entry_id) = params.entry_id {
        return trace_entry(entry_id, config, notion).await;
    }

    let limit = params.limit.unwrap_or(10);
    let currency = params.currency.as_deref().unwrap_or("all");

    // Define the spiral path and currency flows per reservoir
    let spiral_flow = serde_json::json!({
        "lesser_cycle": {
            "path": ["potentiator", "matrix"],
            "flows": {
                "potentiator": { "in": "Experience", "out": "Catalyst" },
                "matrix": { "in": "Catalyst", "out": "Experience" }
            }
        },
        "greater_cycle": {
            "path": ["significator", "greatway"],
            "flows": {
                "significator": { "in": "Transformation", "out": "Choice" },
                "greatway": { "in": "Choice", "out": "Transformation" }
            }
        },
        "nexus": {
            "position": "contact-boundary",
            "flows": { "in": "all", "out": "all" }
        }
    });

    // Determine which reservoirs to query
    let reservoirs: Vec<&str> = match params.scope.as_str() {
        "lesser_cycle" => vec!["matrix", "potentiator"],
        "greater_cycle" => vec!["significator", "greatway"],
        "full_spiral" => vec!["matrix", "potentiator", "significator", "greatway", "nexus"],
        other => vec![other],
    };

    let mut result = serde_json::json!({
        "analysis": "energy_flow",
        "scope": params.scope,
        "currency_filter": currency,
        "spiral": spiral_flow,
        "reservoirs": {}
    });

    for res_key in &reservoirs {
        // Use resolve_db to support both reservoir and satellite keys
        let (ds_id, archetype, cycle, currency_in, currency_out, satellites) = match crate::config::resolve_db(config, res_key) {
            Some(crate::config::ResolvedDb::Reservoir(_k, db)) => {
                (db.ds_id().to_string(), db.archetype.as_deref().unwrap_or("unknown").to_string(),
                 db.cycle.as_deref().unwrap_or("unknown").to_string(),
                 db.currency_in.as_deref().unwrap_or("?").to_string(),
                 db.currency_out.as_deref().unwrap_or("?").to_string(),
                 db.satellites.clone())
            }
            Some(crate::config::ResolvedDb::Satellite(_rk, _sk, sat)) => {
                (sat.ds_id().to_string(), "satellite".to_string(),
                 "unknown".to_string(), "?".to_string(), "?".to_string(),
                 std::collections::HashMap::new())
            }
            None => continue,
        };

        let query = serde_json::json!({ "page_size": limit });
        if let Ok(page_result) = notion.query_data_source(&ds_id, &query).await {
            let items: Vec<serde_json::Value> = page_result.results.iter().map(|p| {
                let title = crate::transform::extract_title(p);
                let status = crate::transform::extract_string(p, "Status");
                serde_json::json!({ "title": title, "status": status })
            }).collect();

            result["reservoirs"][res_key] = serde_json::json!({
                "archetype": archetype,
                "cycle": cycle,
                "currency_flow": { "in": currency_in, "out": currency_out },
                "entry_count": items.len(),
                "has_more": page_result.has_more,
                "entries": items
            });

            // Query satellites
            for (sat_key, sat_cfg) in &satellites {
                let sat_query = serde_json::json!({ "page_size": limit });
                if let Ok(sat_result) = notion.query_data_source(sat_cfg.ds_id(), &sat_query).await {
                    let sat_items: Vec<serde_json::Value> = sat_result.results.iter().map(|p| {
                        let title = crate::transform::extract_title(p);
                        serde_json::json!({ "title": title })
                    }).collect();

                    result["reservoirs"][res_key]["satellites"][sat_key] = serde_json::json!({
                        "name": sat_cfg.name,
                        "role": sat_cfg.role.as_deref().unwrap_or("unknown"),
                        "entry_count": sat_items.len(),
                        "has_more": sat_result.has_more,
                        "entries": sat_items
                    });
                }
            }
        }
    }

    Ok(crate::toon_format::encode(&result))
}

/// Trace a specific entry across reservoirs — find its owner and relations
async fn trace_entry(
    entry_id: &str,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let page = notion.get_page(entry_id).await?;
    let title = crate::transform::extract_title(&page);

    // Determine which reservoir owns this page by checking parent data_source_id
    let parent_ds_id = page.parent.as_ref().and_then(|p| p.data_source_id.as_deref());
    let mut owner_reservoir = "unknown".to_string();
    let mut owner_archetype = "unknown".to_string();
    if let Some(ds_id) = parent_ds_id {
        'outer: for (key, db) in &config.databases {
            if ds_id == db.ds_id() {
                owner_reservoir = key.clone();
                owner_archetype = db.archetype.as_deref().unwrap_or("unknown").to_string();
                break;
            }
            for (sat_key, sat) in &db.satellites {
                if ds_id == sat.ds_id() {
                    owner_reservoir = format!("{}→{}", key, sat_key);
                    owner_archetype = sat.role.as_deref().unwrap_or("satellite").to_string();
                    break 'outer;
                }
            }
        }
    }

    // Get relations from page properties
    let mut relations: Vec<serde_json::Value> = Vec::new();
    for (prop_name, prop_value) in &page.properties {
        if let crate::notion::types::PropertyValue::Relation { relation, .. } = prop_value {
            for rel in relation {
                relations.push(serde_json::json!({
                    "property": prop_name,
                    "target_id": rel.id
                }));
            }
        }
    }

    let result = serde_json::json!({
        "analysis": "entry_trace",
        "entry": {
            "id": entry_id,
            "title": title,
            "owner": owner_reservoir,
            "archetype": owner_archetype,
        },
        "relations": relations,
        "relation_count": relations.len(),
        "spiral_position": {
            "description": format!("This entry lives in {} ({}) — part of the holonic spiral", owner_reservoir, owner_archetype),
        }
    });

    Ok(crate::toon_format::encode(&result))
}
