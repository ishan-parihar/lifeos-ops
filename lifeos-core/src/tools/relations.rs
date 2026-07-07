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

    // ── Bug F fix: parse YAML Metadata into structured fields ──
    // Each of the 5 reservoirs has a "YAML Metadata" rich_text property that
    // stores per-entry-type structured metadata (e.g. Matrix.Pattern requires
    // pattern_type, manifestations, shadow_link, frequency). Parse it and add
    // as a top-level `yaml_metadata` field for easy access.
    if let Some(yaml_str) = page.properties.iter()
        .find(|(name, _)| name.to_lowercase().contains("yaml") && name.to_lowercase().contains("metadata"))
        .and_then(|(_, v)| match v {
            PropertyValue::RichText { rich_text, .. } => Some(rich_text.iter()
                .filter_map(|rt| rt.plain_text.as_deref())
                .collect::<String>()),
            _ => None,
        })
    {
        if !yaml_str.trim().is_empty() {
            // Try parsing as YAML; if it fails, include the raw string under `raw`.
            match serde_yaml::from_str::<serde_yaml::Value>(&yaml_str) {
                Ok(yaml_value) => {
                    if let serde_yaml::Value::Mapping(map) = &yaml_value {
                        let mut parsed = serde_json::Map::new();
                        for (k, v) in map {
                            let key = match k {
                                serde_yaml::Value::String(s) => s.clone(),
                                _ => continue,
                            };
                            let json_val = serde_yaml_to_json(v);
                            parsed.insert(key, json_val);
                        }
                        result["yaml_metadata"] = serde_json::json!({
                            "parsed": serde_json::Value::Object(parsed),
                            "raw": yaml_str,
                        });
                    } else {
                        result["yaml_metadata"] = serde_json::json!({
                            "raw": yaml_str,
                            "parse_error": "YAML is not a mapping",
                        });
                    }
                }
                Err(e) => {
                    result["yaml_metadata"] = serde_json::json!({
                        "raw": yaml_str,
                        "parse_error": format!("{}", e),
                    });
                }
            }
        }
    }

    if !resolved_relations.is_empty() {
        result["relations"] = serde_json::json!(resolved_relations);
    }

    Ok(crate::toon_format::encode(&result))
}

/// Convert a serde_yaml::Value to a serde_json::Value for the yaml_metadata field.
fn serde_yaml_to_json(v: &serde_yaml::Value) -> serde_json::Value {
    match v {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::json!(b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() { serde_json::json!(i) }
            else if let Some(u) = n.as_u64() { serde_json::json!(u) }
            else if let Some(f) = n.as_f64() { serde_json::json!(f) }
            else { serde_json::Value::Null }
        }
        serde_yaml::Value::String(s) => serde_json::json!(s),
        serde_yaml::Value::Sequence(arr) => {
            serde_json::Value::Array(arr.iter().map(serde_yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                let key = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    serde_yaml::Value::Bool(b) => b.to_string(),
                    serde_yaml::Value::Number(n) => n.to_string(),
                    _ => continue,
                };
                obj.insert(key, serde_yaml_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
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
    config: &Arc<LifeOSConfig>,
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

        // ponytail: v4.1 layer labels for Trajectory entries. Replaces the
        // deleted trace_trajectory tool. Reference/Strategic/Execution are
        // the 3 architectural layers defined in FORMAL_SPEC_v4.1.md §2.1.
        let entry_type = entry_type_of(&page, config, &db_key);
        let layer = if db_key == "trajectory" {
            entry_type.as_deref().map(trajectory_layer).unwrap_or_default()
        } else {
            String::new()
        };

        let mut node = serde_json::json!({
            "id": current_id,
            "title": title,
            "database": db_key,
            "level": level,
        });
        if let Some(et) = entry_type {
            node["entry_type"] = serde_json::Value::String(et);
        }
        if !layer.is_empty() {
            node["layer"] = serde_json::Value::String(layer);
        }
        chain.push(node);

        // Find the best "parent" relation to walk up
        match find_parent_relation(&page, config, schema_cache) {
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

// ── get_backlinks ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BacklinksParams {
    /// Page ID to find backlinks for
    pub page_id: String,
    /// Optional database key to search within (searches all if omitted)
    pub database: Option<String>,
}

pub fn schema_backlinks() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "page_id": { "type": "string", "description": "Page ID to find backlinks for" },
            "database": { "type": "string", "description": "Optional DB key to search within" }
        },
        "required": ["page_id"]
    })
}

pub async fn execute_backlinks(
    params: &BacklinksParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    // Determine which databases to search
    let db_keys: Vec<String> = if let Some(ref db) = params.database {
        vec![db.clone()]
    } else {
        config.all_database_keys()
    };

    let mut backlinks: Vec<serde_json::Value> = Vec::new();

    for db_key in &db_keys {
        let ds_id = match crate::config::resolve_db(config, db_key) {
            Some(db) => db.ds_id().to_string(),
            None => continue,
        };

        let query = serde_json::json!({ "page_size": 100 });
        if let Ok(result) = notion.query_data_source(&ds_id, &query).await {
            for page in &result.results {
                for (prop_name, prop_value) in &page.properties {
                    if let PropertyValue::Relation { relation, .. } = prop_value {
                        if relation.iter().any(|r| r.id == params.page_id) {
                            let title = crate::transform::extract_title(page);
                            backlinks.push(serde_json::json!({
                                "id": page.id,
                                "title": title,
                                "database": db_key,
                                "property": prop_name,
                            }));
                        }
                    }
                }
            }
        }
    }

    let data = serde_json::json!({
        "backlinks": {
            "target": params.page_id,
            "entries": backlinks,
            "count": backlinks.len(),
        }
    });

    Ok(crate::toon_format::encode(&data))
}

// ── link ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LinkParams {
    /// Source page ID
    pub source_page: String,
    /// Target page ID to link to
    pub target_page: String,
    /// Relation property name on the source page
    pub property: String,
}

pub fn schema_link() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "source_page": { "type": "string", "description": "Source page ID" },
            "target_page": { "type": "string", "description": "Target page ID to link to" },
            "property": { "type": "string", "description": "Relation property name on source page" }
        },
        "required": ["source_page", "target_page", "property"]
    })
}

pub async fn execute_link(
    params: &LinkParams,
    _config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let source_page = notion.get_page(&params.source_page).await?;

    let existing_ids: Vec<String> = match source_page.properties.get(&params.property) {
        Some(PropertyValue::Relation { relation, .. }) => {
            relation.iter().map(|r| r.id.clone()).collect()
        }
        _ => Vec::new(),
    };

    let mut new_ids: Vec<serde_json::Value> = existing_ids.iter()
        .map(|id| serde_json::json!({ "id": id }))
        .collect();

    if !existing_ids.contains(&params.target_page) {
        new_ids.push(serde_json::json!({ "id": &params.target_page }));
    }

    // update_page() already wraps in {"properties": ...}, so pass the inner map directly.
    let update_props = serde_json::json!({
        &params.property: { "relation": new_ids }
    });

    notion.update_page(&params.source_page, &update_props).await?;

    let source_title = crate::transform::extract_title(&source_page);
    let target_title = notion.get_page(&params.target_page).await
        .ok()
        .map(|p| crate::transform::extract_title(&p))
        .unwrap_or_else(|| "unknown".to_string());

    let already_existed = existing_ids.contains(&params.target_page);

    // U-1 (v0.10.3): Append per-relation semantic hint so AI agents learn
    // what each relation means ontologically without reading the full ontology.
    let hint = crate::tools::quick_link::relation_semantic_hint(&params.property);

    let data = serde_json::json!({
        "link": {
            "source": { "id": &params.source_page, "title": source_title },
            "target": { "id": &params.target_page, "title": target_title },
            "property": params.property,
            "action": if already_existed { "already_existed" } else { "created" },
            "total_relations": new_ids.len(),
            "semantic_hint": hint,
        }
    });

    Ok(format!("{}\n── Semantic hint ──\n  Relation '{}': {}\n  See ONTOLOGY.md for full context.",
        crate::toon_format::encode(&data), params.property, hint))
}

// ── graph_metrics ─────────────────────────────────────────────────

pub fn schema_graph_metrics() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {}
    })
}

pub async fn execute_graph_metrics(
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let mut db_metrics: Vec<serde_json::Value> = Vec::new();
    let mut total_entries: usize = 0;
    let mut total_relation_edges: usize = 0;
    let mut orphan_entries: Vec<serde_json::Value> = Vec::new();

    for db_key in config.all_database_keys() {
        let ds_id = match crate::config::resolve_db(config, &db_key) {
            Some(db) => db.ds_id().to_string(),
            None => continue,
        };

        let query = serde_json::json!({ "page_size": 100 });
        match notion.query_data_source(&ds_id, &query).await {
            Ok(result) => {
                let entry_count = result.results.len();
                total_entries += entry_count;

                let mut relation_count: usize = 0;
                let mut entries_without_relations: usize = 0;

                for page in &result.results {
                    let mut has_any_relation = false;
                    for (_prop_name, prop_value) in &page.properties {
                        if let PropertyValue::Relation { relation, .. } = prop_value {
                            if !relation.is_empty() {
                                relation_count += relation.len();
                                has_any_relation = true;
                            }
                        }
                    }
                    if !has_any_relation {
                        entries_without_relations += 1;
                        let title = crate::transform::extract_title(page);
                        if orphan_entries.len() < 20 {
                            orphan_entries.push(serde_json::json!({
                                "id": page.id,
                                "title": title,
                                "database": db_key,
                            }));
                        }
                    }
                }

                total_relation_edges += relation_count;

                db_metrics.push(serde_json::json!({
                    "database": db_key,
                    "entries": entry_count,
                    "relations": relation_count,
                    "entries_without_relations": entries_without_relations,
                    "relation_density": if entry_count > 0 {
                        (relation_count as f64 / entry_count as f64 * 100.0).round() / 100.0
                    } else {
                        0.0
                    },
                }));
            }
            Err(_) => {
                db_metrics.push(serde_json::json!({
                    "database": db_key,
                    "error": "query_failed",
                }));
            }
        }
    }

    let data = serde_json::json!({
        "graph_metrics": {
            "total_entries": total_entries,
            "total_relation_edges": total_relation_edges,
            "overall_density": if total_entries > 0 {
                (total_relation_edges as f64 / total_entries as f64 * 100.0).round() / 100.0
            } else {
                0.0
            },
            "orphan_entries": orphan_entries,
            "orphan_count_sampled": orphan_entries.len(),
            "databases": db_metrics,
        }
    });

    Ok(crate::toon_format::encode(&data))
}

// ── ancestors (hierarchy traversal) ──────────────────────────────

/// Derive the set of "upward" database keys from config + relation graph.
fn derive_hierarchy_up(config: &crate::config::LifeOSConfig, schema_cache: &SchemaCache) -> Vec<String> {
    let mut upward_keys: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Strategy 1: Find databases that are targets but never sources in the relation graph
    let mut all_targets: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut all_sources: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (db_key, edges) in schema_cache.all_relation_edges() {
        all_sources.insert(db_key.clone());
        for edge in edges {
            all_targets.insert(edge.target_db.clone());
        }
    }
    let leaf_targets: Vec<String> = all_targets.difference(&all_sources).cloned().collect();
    for key in &leaf_targets {
        upward_keys.insert(key.clone());
    }

    // Strategy 2: Databases with scale "all-stage" and dimension "intra-holonic"
    for (key, db) in &config.databases {
        if db.scale.as_deref() == Some("all-stage") && db.dimension.as_deref() == Some("intra-holonic") {
            upward_keys.insert(key.clone());
        }
    }

    // Strategy 3: Property names that indicate "parent" direction
    for (key, db) in &config.databases {
        for prop_name in db.properties.keys() {
            let lower = prop_name.to_lowercase();
            if lower.contains("parent") || lower.contains("annual") || lower.contains("quarter")
                || lower.contains("vision") || lower.contains("value") || lower.contains("pillar") {
                if let Some(edges) = schema_cache.all_relation_edges().get(key) {
                    for edge in edges {
                        upward_keys.insert(edge.target_db.clone());
                    }
                }
            }
        }
    }

    upward_keys.into_iter().collect()
}

/// Find a "parent" relation for hierarchical navigation.
fn find_parent_relation(
    page: &crate::notion::types::NotionPage,
    config: &crate::config::LifeOSConfig,
    schema_cache: &SchemaCache,
) -> Option<String> {
    let hierarchy_up = derive_hierarchy_up(config, schema_cache);

    let parent_ds_id = page.parent.as_ref().and_then(|p| p.data_source_id.as_deref());

    if let Some(ds_id) = parent_ds_id {
        if let Some(db_key) = schema_cache.resolve_db_key_from_id(ds_id) {
            let edges = schema_cache.get_relation_edges(db_key);
            let upward_targets: Vec<&str> = edges.iter()
                .filter(|e| hierarchy_up.contains(&e.target_db))
                .map(|e| e.prop_name.as_str())
                .collect();

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
                if lower.contains("parent") || lower.contains("annual") || lower.contains("quarter")
                    || lower.contains("vision") || lower.contains("value") || lower.contains("pillar")
                    || lower.contains("year") || lower.contains("month") || lower.contains("week")
                    || lower.contains("strategic") {
                    return Some(first.id.clone());
                }
            }
        }
    }

    None
}

/// Extract the entry-type (select option) for a page using the DB's configured
/// entry_type_property. Returns None if the property is missing or empty.
fn entry_type_of(
    page: &crate::notion::types::NotionPage,
    config: &crate::config::LifeOSConfig,
    db_key: &str,
) -> Option<String> {
    let et_prop = config.databases.get(db_key)
        .and_then(|d| d.entry_type_property.clone())?;
    match page.properties.get(&et_prop)? {
        PropertyValue::Select { select, .. } => select.as_ref().map(|s| s.name.clone()),
        PropertyValue::MultiSelect { multi_select, .. } => multi_select.first().map(|s| s.name.clone()),
        _ => None,
    }
}

/// Map a Trajectory entry-type to its v4.1 architectural layer.
/// Reference: timeless (Purpose/Value/Principle/Vision-Statement/Identity-Statement)
/// Strategic: temporal (Annual-Goal/Quarterly-Goal/Milestone)
/// Execution: daily (Project/Task/Campaign/Content)
/// ponytail: replaces the deleted trace_trajectory tool's layer logic.
fn trajectory_layer(entry_type: &str) -> String {
    match entry_type {
        "Purpose" | "Value" | "Principle" | "Vision-Statement" | "Identity-Statement" => "Reference",
        "Annual-Goal" | "Quarterly-Goal" | "Milestone" => "Strategic",
        "Project" | "Task" | "Campaign" | "Content" => "Execution",
        _ => "",
    }.to_string()
}
