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
    /// Limit per database (default 10)
    pub limit: Option<u32>,
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "scope": { "type": "string", "enum": ["lesser_cycle", "greater_cycle", "full_spiral", "matrix", "potentiator", "significator", "greatway", "nexus"], "description": "Scope of energy flow analysis" },
            "currency": { "type": "string", "enum": ["Catalyst", "Experience", "Transformation", "Choice", "all"], "description": "Currency to trace (default: all)" },
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
        let db = match crate::config::get_db(config, res_key) {
            Some(db) => db,
            None => continue,
        };

        // Query the reservoir itself
        let query = serde_json::json!({ "page_size": limit });
        if let Ok(page_result) = notion.query_data_source(db.ds_id(), &query).await {
            let items: Vec<serde_json::Value> = page_result.results.iter().map(|p| {
                let title = crate::transform::extract_title(p);
                let status = crate::transform::extract_string(p, "Status");
                serde_json::json!({ "title": title, "status": status })
            }).collect();

            let archetype = db.archetype.as_deref().unwrap_or("unknown");
            let cycle = db.cycle.as_deref().unwrap_or("unknown");
            let currency_in = db.currency_in.as_deref().unwrap_or("?");
            let currency_out = db.currency_out.as_deref().unwrap_or("?");

            result["reservoirs"][res_key] = serde_json::json!({
                "archetype": archetype,
                "cycle": cycle,
                "currency_flow": { "in": currency_in, "out": currency_out },
                "entry_count": items.len(),
                "has_more": page_result.has_more,
                "entries": items
            });

            // Query satellites
            for (sat_key, sat_cfg) in &db.satellites {
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
