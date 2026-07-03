//! `unlink` and `batch_link` tools — deliberate relationship updates.
//!
//! Write tools. `unlink` removes a single relation. `batch_link` creates
//! multiple relations in one call (but each must be explicitly specified —
//! no auto-population).

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

// ── unlink ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UnlinkParams {
    pub source_page: String,
    pub target_page: String,
    pub property: String,
}

pub fn schema_unlink() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "source_page": { "type": "string", "description": "Source page ID" },
            "target_page": { "type": "string", "description": "Target page ID to unlink from" },
            "property": { "type": "string", "description": "Relation property name on source page" }
        },
        "required": ["source_page", "target_page", "property"]
    })
}

pub async fn execute_unlink(
    params: &UnlinkParams,
    _config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let source_page = notion.get_page(&params.source_page).await?;

    // Get existing relations
    let existing_ids: Vec<String> = match source_page.properties.get(&params.property) {
        Some(PropertyValue::Relation { relation, .. }) => {
            relation.iter().map(|r| r.id.clone()).collect()
        }
        _ => Vec::new(),
    };

    // Remove the target page
    let new_ids: Vec<serde_json::Value> = existing_ids.iter()
        .filter(|id| *id != &params.target_page)
        .map(|id| serde_json::json!({ "id": id }))
        .collect();

    let update_body = serde_json::json!({
        "properties": {
            &params.property: { "relation": new_ids }
        }
    });

    notion.update_page(&params.source_page, &update_body).await?;

    let source_title = crate::transform::extract_title(&source_page);
    let target_title = notion.get_page(&params.target_page).await
        .ok()
        .map(|p| crate::transform::extract_title(&p))
        .unwrap_or_else(|| "unknown".to_string());

    let data = serde_json::json!({
        "unlink": {
            "source_page": params.source_page,
            "source_title": source_title,
            "target_page": params.target_page,
            "target_title": target_title,
            "property": params.property,
            "remaining_relations": new_ids.len(),
        }
    });

    Ok(crate::toon_format::encode(&data))
}

// ── batch_link ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BatchLinkParams {
    /// List of links to create. Each link: { source_page, target_page, property }
    pub links: Vec<BatchLinkItem>,
}

#[derive(Debug, Deserialize)]
pub struct BatchLinkItem {
    pub source_page: String,
    pub target_page: String,
    pub property: String,
}

pub fn schema_batch_link() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "links": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "source_page": { "type": "string", "description": "Source page ID" },
                        "target_page": { "type": "string", "description": "Target page ID to link to" },
                        "property": { "type": "string", "description": "Relation property name on source page" }
                    },
                    "required": ["source_page", "target_page", "property"]
                },
                "description": "List of explicit links to create. Each must be specified — no auto-population."
            }
        },
        "required": ["links"]
    })
}

pub async fn execute_batch_link(
    params: &BatchLinkParams,
    _config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let mut results: Vec<serde_json::Value> = Vec::new();
    let mut success_count = 0;
    let mut error_count = 0;

    for item in &params.links {
        // Fetch the source page to get existing relations
        let source_page = match notion.get_page(&item.source_page).await {
            Ok(p) => p,
            Err(e) => {
                error_count += 1;
                results.push(serde_json::json!({
                    "source_page": item.source_page,
                    "target_page": item.target_page,
                    "property": item.property,
                    "status": "error",
                    "error": format!("Failed to fetch source: {}", e),
                }));
                continue;
            }
        };

        // Get existing relation IDs
        let existing_ids: Vec<String> = match source_page.properties.get(&item.property) {
            Some(PropertyValue::Relation { relation, .. }) => {
                relation.iter().map(|r| r.id.clone()).collect()
            }
            _ => Vec::new(),
        };

        // Add the new target (if not already linked)
        if existing_ids.contains(&item.target_page) {
            results.push(serde_json::json!({
                "source_page": item.source_page,
                "target_page": item.target_page,
                "property": item.property,
                "status": "already_linked",
            }));
            success_count += 1;
            continue;
        }

        let mut new_ids: Vec<serde_json::Value> = existing_ids.iter()
            .map(|id| serde_json::json!({ "id": id }))
            .collect();
        new_ids.push(serde_json::json!({ "id": &item.target_page }));

        let update_body = serde_json::json!({
            "properties": {
                &item.property: { "relation": new_ids }
            }
        });

        match notion.update_page(&item.source_page, &update_body).await {
            Ok(_) => {
                success_count += 1;
                results.push(serde_json::json!({
                    "source_page": item.source_page,
                    "target_page": item.target_page,
                    "property": item.property,
                    "status": "linked",
                }));
            }
            Err(e) => {
                error_count += 1;
                results.push(serde_json::json!({
                    "source_page": item.source_page,
                    "target_page": item.target_page,
                    "property": item.property,
                    "status": "error",
                    "error": e,
                }));
            }
        }
    }

    let data = serde_json::json!({
        "batch_link": {
            "total_requested": params.links.len(),
            "success_count": success_count,
            "error_count": error_count,
            "results": results,
        }
    });

    Ok(crate::toon_format::encode(&data))
}
