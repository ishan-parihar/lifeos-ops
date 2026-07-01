//! Relational graph tools — get_page, expand, trace, ancestors
//!
//! These tools enable AI-agents to navigate the relational graph between databases,
//! resolving relation IDs to titled entries and traversing hierarchies.

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

// ── get_page ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GetPageParams {
    /// Page ID to fetch
    pub page_id: String,
    /// Optional database key hint (speeds up property resolution)
    pub database: Option<String>,
}

pub fn schema_get_page() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "page_id": { "type": "string", "description": "Notion page ID to fetch" },
            "database": { "type": "string", "description": "Optional DB key hint for property resolution" }
        },
        "required": ["page_id"]
    })
}

pub async fn execute_get_page(
    params: &GetPageParams,
    _config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let page = notion.get_page(&params.page_id).await?;

    // Determine which database this page belongs to
    let parent_ds_id = page.parent.as_ref().and_then(|p| p.data_source_id.as_deref()).map(String::from);
    let db_key = parent_ds_id.as_deref()
        .and_then(|id| schema_cache.resolve_db_key_from_id(id))
        .or(params.database.as_deref());

    let title = crate::transform::extract_title(&page);
    let mut resolved_relations: Vec<serde_json::Value> = Vec::new();
    let mut all_properties: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    for (prop_name, prop_value) in &page.properties {
        match prop_value {
            PropertyValue::Relation { relation, .. } => {
                let mut resolved = Vec::new();
                for rel in relation {
                    // Fetch each related page ONCE
                    if let Ok(target_page) = notion.get_page(&rel.id).await {
                        let target_title = crate::transform::extract_title(&target_page);
                        let target_db = target_page.parent.as_ref()
                            .and_then(|p| p.data_source_id.as_deref())
                            .and_then(|id| schema_cache.resolve_db_key_from_id(id))
                            .unwrap_or("unknown");
                        resolved.push(serde_json::json!({
                            "id": rel.id,
                            "title": target_title,
                            "database": target_db,
                        }));
                    } else {
                        resolved.push(serde_json::json!({
                            "id": rel.id,
                            "title": "unresolved",
                            "database": "unknown",
                        }));
                    }
                }
                // Include titles as a simple list for readability
                let titles: Vec<String> = resolved.iter()
                    .filter_map(|r| r.get("title").and_then(|t| t.as_str()).map(String::from))
                    .collect();
                all_properties.insert(prop_name.clone(), serde_json::json!(titles));

                resolved_relations.push(serde_json::json!({
                    "property": prop_name,
                    "entries": resolved,
                }));
            }
            _ => {
                if let Some(val) = crate::transform::extract_property_value(&page, prop_name) {
                    if !val.is_null() {
                        all_properties.insert(prop_name.clone(), val);
                    }
                }
            }
        }
    }

    let mut result = serde_json::json!({
        "id": page.id,
        "title": title,
        "database": db_key.unwrap_or("unknown"),
        "properties": all_properties,
    });

    if !resolved_relations.is_empty() {
        result["relations"] = serde_json::json!(resolved_relations);
    }

    Ok(crate::toon_format::encode(&result))
}

// ── expand ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ExpandParams {
    /// List of page IDs to expand
    pub page_ids: Vec<String>,
}

pub fn schema_expand() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "page_ids": {
                "type": "array",
                "items": { "type": "string" },
                "description": "List of page IDs to expand with titles and database keys"
            }
        },
        "required": ["page_ids"]
    })
}

pub async fn execute_expand(
    params: &ExpandParams,
    _config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let mut expanded = Vec::new();

    for page_id in &params.page_ids {
        match notion.get_page(page_id).await {
            Ok(page) => {
                let title = crate::transform::extract_title(&page);
                let db_key = page.parent.as_ref()
                    .and_then(|p| p.data_source_id.as_deref())
                    .and_then(|id| schema_cache.resolve_db_key_from_id(id))
                    .unwrap_or("unknown");
                expanded.push(serde_json::json!({
                    "id": page_id,
                    "title": title,
                    "database": db_key,
                }));
            }
            Err(e) => {
                expanded.push(serde_json::json!({
                    "id": page_id,
                    "error": e,
                }));
            }
        }
    }

    let data = serde_json::json!({
        "expanded": expanded,
        "count": expanded.len(),
    });

    Ok(crate::toon_format::encode(&data))
}

// ── trace ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TraceParams {
    /// Starting page ID
    pub page_id: String,
    /// Max depth to follow relations (default: 2)
    pub depth: Option<u32>,
}

pub fn schema_trace() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "page_id": { "type": "string", "description": "Starting page ID to trace from" },
            "depth": { "type": "integer", "minimum": 1, "maximum": 3, "description": "Max relation depth (default: 2, max 3)" }
        },
        "required": ["page_id"]
    })
}

pub async fn execute_trace(
    params: &TraceParams,
    _config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let max_depth = params.depth.unwrap_or(2).min(3);
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut nodes: Vec<serde_json::Value> = Vec::new();
    let mut edges: Vec<serde_json::Value> = Vec::new();

    trace_entry(
        &params.page_id,
        0,
        max_depth,
        notion,
        schema_cache,
        &mut visited,
        &mut nodes,
        &mut edges,
    ).await;

    let data = serde_json::json!({
        "trace": {
            "root": params.page_id,
            "max_depth": max_depth,
            "nodes": nodes,
            "edges": edges,
            "node_count": nodes.len(),
            "edge_count": edges.len(),
        }
    });

    Ok(crate::toon_format::encode(&data))
}

async fn trace_entry(
    page_id: &str,
    current_depth: u32,
    max_depth: u32,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
    visited: &mut std::collections::HashSet<String>,
    nodes: &mut Vec<serde_json::Value>,
    edges: &mut Vec<serde_json::Value>,
) {
    if visited.contains(page_id) || current_depth > max_depth {
        return;
    }
    visited.insert(page_id.to_string());

    let page = match notion.get_page(page_id).await {
        Ok(p) => p,
        Err(_) => return,
    };

    let title = crate::transform::extract_title(&page);
    let db_key = page.parent.as_ref()
        .and_then(|p| p.data_source_id.as_deref())
        .and_then(|id| schema_cache.resolve_db_key_from_id(id))
        .unwrap_or("unknown");

    nodes.push(serde_json::json!({
        "id": page_id,
        "title": title,
        "database": db_key,
        "depth": current_depth,
    }));

    // Follow relation properties (depth-limited)
    if current_depth < max_depth {
        for (prop_name, prop_value) in &page.properties {
            if let PropertyValue::Relation { relation, .. } = prop_value {
                for rel in relation {
                    edges.push(serde_json::json!({
                        "from": page_id,
                        "to": &rel.id,
                        "property": prop_name,
                    }));

                    Box::pin(trace_entry(
                        &rel.id,
                        current_depth + 1,
                        max_depth,
                        notion,
                        schema_cache,
                        visited,
                        nodes,
                        edges,
                    )).await;
                }
            }
        }
    }
}

// ── ancestors ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AncestorsParams {
    /// Starting page ID
    pub page_id: String,
    /// Max levels to walk up (default: 5)
    pub max_levels: Option<u32>,
}

pub fn schema_ancestors() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "page_id": { "type": "string", "description": "Starting page ID to walk up from" },
            "max_levels": { "type": "integer", "minimum": 1, "maximum": 10, "description": "Max levels to walk up (default: 5)" }
        },
        "required": ["page_id"]
    })
}

pub async fn execute_ancestors(
    params: &AncestorsParams,
    _config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let max_levels = params.max_levels.unwrap_or(5).min(10);
    let mut chain: Vec<serde_json::Value> = Vec::new();
    let mut current_id = params.page_id.clone();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

    for level in 0..max_levels {
        if visited.contains(&current_id) {
            break;
        }
        visited.insert(current_id.clone());

        let page = match notion.get_page(&current_id).await {
            Ok(p) => p,
            Err(_) => break,
        };

        let title = crate::transform::extract_title(&page);
        let db_key = page.parent.as_ref()
            .and_then(|p| p.data_source_id.as_deref())
            .and_then(|id| schema_cache.resolve_db_key_from_id(id))
            .unwrap_or("unknown");

        chain.push(serde_json::json!({
            "id": current_id,
            "title": title,
            "database": db_key,
            "level": level,
        }));

        // Find the best "parent" relation to walk up
        match find_parent_relation(&page, schema_cache) {
            Some(id) => current_id = id,
            None => break,
        }
    }

    let data = serde_json::json!({
        "ancestors": {
            "root": &params.page_id,
            "chain": chain,
            "depth": chain.len(),
        }
    });

    Ok(crate::toon_format::encode(&data))
}

/// Hierarchical priority: which DB keys represent "upward" in the hierarchy.
/// When a relation points to one of these, it's a parent link.
const HIERARCHY_UP: &[&str] = &[
    "annual_goals", "years",
    "quarterly_goals", "quarters",
    "projects", "months", "weeks",
    "parent_task",
    "vision", "values",
];

/// Find a "parent" relation for hierarchical navigation.
/// Uses the schema relation graph to determine which relations point upward.
fn find_parent_relation(
    page: &crate::notion::types::NotionPage,
    schema_cache: &SchemaCache,
) -> Option<String> {
    // First: check if we know the source database and can use the relation graph
    let parent_ds_id = page.parent.as_ref().and_then(|p| p.data_source_id.as_deref());

    if let Some(ds_id) = parent_ds_id {
        if let Some(db_key) = schema_cache.resolve_db_key_from_id(ds_id) {
            let edges = schema_cache.get_relation_edges(db_key);
            // Find which relation properties point to "upward" databases
            let upward_targets: Vec<&str> = edges.iter()
                .filter(|e| HIERARCHY_UP.contains(&e.target_db.as_str()))
                .map(|e| e.prop_name.as_str())
                .collect();

            // Now find the first matching relation in the page
            for prop_name in &upward_targets {
                if let Some(PropertyValue::Relation { relation, .. }) = page.properties.get(*prop_name) {
                    if let Some(first) = relation.first() {
                        return Some(first.id.clone());
                    }
                }
            }
        }
    }

    // Fallback: heuristic property name matching
    for (prop_name, prop_value) in &page.properties {
        if let PropertyValue::Relation { relation, .. } = prop_value {
            if let Some(first) = relation.first() {
                let lower = prop_name.to_lowercase();
                for target in HIERARCHY_UP {
                    if lower.contains(target) || lower == *target {
                        return Some(first.id.clone());
                    }
                }
            }
        }
    }

    None
}
