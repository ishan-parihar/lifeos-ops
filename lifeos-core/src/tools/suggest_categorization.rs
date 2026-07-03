//! `suggest_categorization` tool — suggests entry-types for uncategorized entries.
//!
//! Read-only. Uses title keyword heuristics to suggest entry-types.
//! Returns confidence + reasoning. NEVER writes — the user must call `mutate`
//! to apply each suggestion deliberately.

use std::collections::HashMap;
use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

#[derive(Debug, Deserialize)]
pub struct SuggestCategorizationParams {
    pub database: Option<String>,
    pub limit: Option<u32>,
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "database": { "type": "string", "enum": ["matrix", "potentiator", "nexus", "significator", "greatway"], "description": "DB to scan. Omit to scan all 5." },
            "limit": { "type": "integer", "minimum": 1, "maximum": 200, "description": "Max suggestions per DB (default: 20)" }
        }
    })
}

pub async fn execute(
    params: &SuggestCategorizationParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let limit = params.limit.unwrap_or(20).min(200) as u64;

    let db_keys: Vec<String> = if let Some(ref db) = params.database {
        if !config.databases.contains_key(db) {
            return Err(format!("Unknown database: {}", db));
        }
        vec![db.clone()]
    } else {
        config.all_database_keys()
    };

    let mut all_suggestions: Vec<serde_json::Value> = Vec::new();

    for db_key in &db_keys {
        let db = match crate::config::resolve_db(config, db_key) {
            Some(db) => db,
            None => continue,
        };
        let ds_id = db.ds_id();
        let et_prop_name = db.entry_type_property.clone().unwrap_or_else(|| "Entry Type".to_string());

        // Get available entry-type options
        let et_options: Vec<String> = schema_cache.get_entry_type_options(db_key, config);

        // Query uncategorized entries (entry_type is empty)
        let query = serde_json::json!({
            "page_size": limit,
            "filter": {
                "or": [
                    { "property": &et_prop_name, "select": { "is_empty": true } },
                    { "property": &et_prop_name, "multi_select": { "is_empty": true } }
                ]
            }
        });

        let result = match notion.query_data_source(ds_id, &query).await {
            Ok(r) => r,
            Err(e) => {
                // Fallback: query without filter and filter manually
                let query = serde_json::json!({ "page_size": limit });
                let result = notion.query_data_source(ds_id, &query).await
                    .map_err(|e2| format!("Query failed: {} / fallback: {}", e, e2))?;
                result
            }
        };

        let mut db_suggestions: Vec<serde_json::Value> = Vec::new();

        for page in &result.results {
            // Check if entry-type is empty
            let et_prop = page.properties.get(&et_prop_name);
            let has_et = match et_prop {
                Some(PropertyValue::Select { select, .. }) => select.is_some(),
                Some(PropertyValue::MultiSelect { multi_select, .. }) => !multi_select.is_empty(),
                _ => false,
            };
            if has_et { continue; }

            let title = crate::transform::extract_title(page);
            if title.trim().is_empty() { continue; }

            // Generate suggestion based on title keywords
            if let Some((suggested_et, confidence, reason)) = suggest_entry_type(db_key, &title, &et_options) {
                db_suggestions.push(serde_json::json!({
                    "page_id": page.id,
                    "title": title,
                    "db": db_key,
                    "suggested_entry_type": suggested_et,
                    "confidence": confidence,
                    "reason": reason,
                    "apply_command": format!(
                        "lifeos mutate --operation update --database {} --page-id {} --properties '{{\"{}\": \"{}\"}}'",
                        db_key, page.id, et_prop_name, suggested_et
                    ),
                }));
            }
        }

        if !db_suggestions.is_empty() {
            all_suggestions.extend(db_suggestions);
        }
    }

    let data = serde_json::json!({
        "suggest_categorization": {
            "total_suggestions": all_suggestions.len(),
            "suggestions": all_suggestions,
            "note": "These are suggestions only. Apply each deliberately via 'lifeos mutate' or the mutate MCP tool.",
        }
    });

    Ok(crate::toon_format::encode(&data))
}

/// Suggest an entry-type based on title keywords.
/// Returns (entry_type, confidence, reason).
fn suggest_entry_type(db_key: &str, title: &str, available_options: &[String]) -> Option<(String, String, String)> {
    let title_lower = title.to_lowercase();

    // DB-specific heuristics
    let suggestions: Vec<(String, String, String)> = match db_key {
        "matrix" => {
            let mut s = Vec::new();
            if title_lower.contains("pattern") || title_lower.contains("recurring") || title_lower.contains("cycle") {
                s.push(("Pattern".to_string(), "high".to_string(), "Title describes a recurring pattern".to_string()));
            }
            if title_lower.contains("practice") || title_lower.contains("routine") || title_lower.contains("habit") {
                s.push(("Practice".to_string(), "high".to_string(), "Title describes a practice or routine".to_string()));
            }
            if title_lower.contains("inventory") || title_lower.contains("audit") || title_lower.contains("stock") {
                s.push(("Inventory".to_string(), "high".to_string(), "Title describes an inventory or audit".to_string()));
            }
            if title_lower.contains("threshold") || title_lower.contains("trigger") || title_lower.contains("breaking point") {
                s.push(("Threshold".to_string(), "high".to_string(), "Title describes a threshold or trigger".to_string()));
            }
            if title_lower.contains("foundation") || title_lower.contains("baseline") || title_lower.contains("core") {
                s.push(("Foundation".to_string(), "medium".to_string(), "Title describes a foundation or baseline".to_string()));
            }
            // Default for Matrix
            if s.is_empty() {
                s.push(("Pattern".to_string(), "low".to_string(), "Default Matrix entry-type — title doesn't match specific keywords".to_string()));
            }
            s
        }
        "significator" => {
            let mut s = Vec::new();
            if title_lower.contains("purpose") || title_lower.contains("mission") || title_lower.contains("calling") {
                s.push(("Purpose".to_string(), "high".to_string(), "Title describes purpose or mission".to_string()));
            }
            if title_lower.contains("value") || title_lower.contains("principle") || title_lower.contains("belief") {
                s.push(("Value".to_string(), "high".to_string(), "Title describes a value or principle".to_string()));
            }
            if title_lower.contains("pillar") || title_lower.contains("foundation") || title_lower.contains("pillar") {
                s.push(("Pillar".to_string(), "high".to_string(), "Title describes a pillar or foundational element".to_string()));
            }
            if title_lower.contains("identity") || title_lower.contains("self") || title_lower.contains("who am i") {
                s.push(("Identity-Statement".to_string(), "high".to_string(), "Title describes identity".to_string()));
            }
            if title_lower.contains("ideal") || title_lower.contains("vision") || title_lower.contains("aspiration") {
                s.push(("Strategic-Ideal".to_string(), "medium".to_string(), "Title describes an ideal or aspiration".to_string()));
            }
            s
        }
        "nexus" => {
            let mut s = Vec::new();
            if title_lower.contains("dream") || title_lower.contains("note") || title_lower.contains("observation") {
                s.push(("Note".to_string(), "high".to_string(), "Title describes a note or observation".to_string()));
            }
            if title_lower.contains("insight") || title_lower.contains("realization") || title_lower.contains("aha") {
                s.push(("Insight".to_string(), "high".to_string(), "Title describes an insight or realization".to_string()));
            }
            if title_lower.contains("reflect") || title_lower.contains("journal") || title_lower.contains("diary") {
                s.push(("Reflection".to_string(), "high".to_string(), "Title describes a reflection or journal entry".to_string()));
            }
            if title_lower.contains("risk") || title_lower.contains("threat") || title_lower.contains("danger") {
                s.push(("Risk".to_string(), "high".to_string(), "Title describes a risk or threat".to_string()));
            }
            if title_lower.contains("decision") || title_lower.contains("choice") || title_lower.contains("decided") {
                s.push(("Decision".to_string(), "high".to_string(), "Title describes a decision or choice".to_string()));
            }
            if title_lower.contains("crisis") || title_lower.contains("emergency") || title_lower.contains("breakdown") {
                s.push(("Crisis".to_string(), "high".to_string(), "Title describes a crisis or emergency".to_string()));
            }
            // Default for Nexus
            if s.is_empty() {
                s.push(("Note".to_string(), "low".to_string(), "Default Nexus entry-type".to_string()));
            }
            s
        }
        "greatway" => {
            let mut s = Vec::new();
            if title_lower.contains("task") || title_lower.contains("todo") || title_lower.contains("action item") {
                s.push(("Task".to_string(), "high".to_string(), "Title describes a task or action item".to_string()));
            }
            if title_lower.contains("project") || title_lower.contains("initiative") {
                s.push(("Project".to_string(), "high".to_string(), "Title describes a project".to_string()));
            }
            if title_lower.contains("content") || title_lower.contains("article") || title_lower.contains("post") {
                s.push(("Content".to_string(), "high".to_string(), "Title describes content".to_string()));
            }
            if title_lower.contains("campaign") || title_lower.contains("launch") {
                s.push(("Campaign".to_string(), "high".to_string(), "Title describes a campaign".to_string()));
            }
            if title_lower.contains("milestone") || title_lower.contains("deadline") {
                s.push(("Milestone".to_string(), "high".to_string(), "Title describes a milestone".to_string()));
            }
            s
        }
        "potentiator" => {
            let mut s = Vec::new();
            if title_lower.contains("activity") || title_lower.contains("work") || title_lower.contains("session") {
                s.push(("Activity".to_string(), "medium".to_string(), "Title describes an activity".to_string()));
            }
            if title_lower.contains("financial") || title_lower.contains("payment") || title_lower.contains("transaction") {
                s.push(("Financial".to_string(), "high".to_string(), "Title describes a financial transaction".to_string()));
            }
            if title_lower.contains("diet") || title_lower.contains("food") || title_lower.contains("meal") {
                s.push(("Diet".to_string(), "high".to_string(), "Title describes food/diet".to_string()));
            }
            s
        }
        _ => Vec::new(),
    };

    // Filter to only suggest available options
    for (et, conf, reason) in &suggestions {
        if available_options.contains(et) {
            return Some((et.clone(), conf.clone(), reason.clone()));
        }
    }

    suggestions.into_iter().next()
}
