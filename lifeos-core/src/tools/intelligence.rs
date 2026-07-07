//! Intelligence briefing tool — role-based and module-specific analysis

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

/// Intelligence briefing parameters
#[derive(Debug, Deserialize)]
pub struct IntelligenceParams {
    /// Briefing mode: role or module
    pub mode: String,
    /// Role key when mode=role (e.g., CEO, COO, CMO, CRO, CFO, CHO)
    pub role: Option<String>,
    /// Module key when mode=module
    pub module: Option<String>,
    /// Date range: "today", "this_week", "this_month", "this_quarter" or ISO date
    pub range: Option<String>,
    /// Per-database overrides: { db_key: { filter: {...}, sort: {...} } }
    pub overrides: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Generate JSON Schema for this tool
pub fn schema(_schema_cache: &SchemaCache) -> serde_json::Value {
    let obj = serde_json::json!({
        "type": "object",
        "properties": {
            "mode": { "type": "string", "enum": ["role", "module", "lesser_cycle", "greater_cycle", "nexus", "drive_balance", "reservoir_health"], "description": "Briefing mode: role/module for config-driven briefings, lesser_cycle/greater_cycle/nexus/drive_balance/reservoir_health for holonic analysis" },
            "role": { "type": "string", "enum": ["CEO", "COO", "CMO", "CRO", "CFO", "CHO"], "description": "Role key when mode=role" },
            "module": { "type": "string", "enum": ["productivity", "health", "strategic", "financial", "content", "journaling"], "description": "Module key when mode=module" },
            "range": { "type": "string", "description": "Date range: today, this_week, this_month, this_quarter or ISO date" },
            "overrides": {
                "type": "object",
                "description": "Per-database overrides: { db_key: { filter: {...}, sort: {...} } }.",
                "additionalProperties": {
                    "type": "object",
                    "properties": {
                        "filter": { "type": "object", "description": "Notion API filter to override the static default" },
                        "sort": { "type": "object", "description": "Notion API sort to override the default sort" }
                    }
                }
            }
        },
        "required": ["mode"]
    });

    obj
}

/// Non-filterable Notion property types that cannot be used in filters.
const NON_FILTERABLE_TYPES: &[&str] = &[
    "formula", "rollup", "created_by", "last_edited_by",
    "created_time", "last_edited_time", "button", "unique_id",
];

/// Module-specific entry type filters: when a module targets a reservoir,
/// we query by entry type within that unified database.
fn module_entry_type_filters(module_key: &str) -> Vec<(&'static str, Vec<&'static str>)> {
    // Returns (db_key, entry_type_values) for the module
    match module_key {
        "health" => vec![
            ("potentiator", vec!["Activity", "Diet", "Subjective"]),
        ],
        "financial" => vec![
            ("potentiator", vec!["Financial"]),
        ],
        "journaling" => vec![
            ("potentiator", vec!["Subjective", "Relational", "Systemic"]),
        ],
        "content" => vec![
            ("greatway", vec!["Content", "Campaign"]),
        ],
        "productivity" => vec![
            ("greatway", vec!["Task", "Project"]),
        ],
        "strategic" => vec![
            ("greatway", vec!["Project", "Annual Goal", "Quarterly Goal"]),
        ],
        _ => Vec::new(),
    }
}

/// Walk a filter tree and correct type keys (`"status"` ↔ `"select"`)
/// based on the actual Notion property type from SchemaCache.
fn correct_filter_type(
    filter: &serde_json::Value,
    db_key: &str,
    schema_cache: &SchemaCache,
    property_mapping: &std::collections::HashMap<String, String>,
) -> serde_json::Value {
    use serde_json::Value;
    match filter {
        Value::Object(map) => {
            // Compound filter: "or" or "and" — recurse + filter nulls
            if let Some(compound_key) = map.get("or").or_else(|| map.get("and")) {
                let key = if map.contains_key("or") { "or" } else { "and" };
                let corrected: Vec<Value> = compound_key.as_array()
                    .map(|arr| {
                        arr.iter()
                            .map(|c| correct_filter_type(c, db_key, schema_cache, property_mapping))
                            .filter(|v| !v.is_null())
                            .collect()
                    })
                    .unwrap_or_default();
                if corrected.is_empty() {
                    return Value::Null;
                }
                if corrected.len() == 1 {
                    return corrected.into_iter().next().unwrap();
                }
                let mut result = serde_json::Map::new();
                result.insert(key.to_string(), Value::Array(corrected));
                return Value::Object(result);
            }

            // Simple filter with "property" key
            if let Some(prop_name) = map.get("property").and_then(|v| v.as_str()) {
                let prop_lower = prop_name.to_lowercase();
                let config_key = property_mapping.iter()
                    .find(|(_, v)| v.to_lowercase() == prop_lower)
                    .map(|(k, _)| k.as_str());

                if let Some(cfg_key) = config_key {
                    let actual_type = schema_cache.get_prop_type(db_key, cfg_key);
                    if let Some(actual) = actual_type {
                        if NON_FILTERABLE_TYPES.contains(&actual) {
                            return Value::Null;
                        }
                        let filterable = ["select", "status", "rich_text", "title", "date", "checkbox", "url", "email", "phone_number", "number", "multi_select"];
                        for type_key in &filterable {
                            if map.contains_key(*type_key) && *type_key != actual {
                                let mut result = serde_json::Map::new();
                                for (k, v) in map.iter() {
                                    if k == type_key {
                                        result.insert(actual.to_string(), v.clone());
                                    } else {
                                        result.insert(k.clone(), v.clone());
                                    }
                                }
                                return Value::Object(result);
                            }
                        }
                    }
                }
            }

            filter.clone()
        }
        Value::Array(arr) => {
            let corrected: Vec<Value> = arr.iter()
                .map(|v| correct_filter_type(v, db_key, schema_cache, property_mapping))
                .filter(|v| !v.is_null())
                .collect();
            if corrected.is_empty() {
                Value::Null
            } else {
                Value::Array(corrected)
            }
        }
        _ => filter.clone(),
    }
}

fn build_target_query(
    target: &crate::config::BriefingTarget,
    db: &crate::config::DbConfig,
    db_key: &str,
    range: &str,
    schema_cache: &SchemaCache,
    override_filter: Option<&serde_json::Value>,
    override_sort: Option<&serde_json::Value>,
) -> (serde_json::Value, serde_json::Value) {
    let mut query = serde_json::json!({ "page_size": target.limit.unwrap_or(10) });

    let static_filter = target.effective_filter();
    let final_filter = override_filter.or(static_filter);

    if let Some(ref f) = final_filter {
        let corrected = correct_filter_type(f, db_key, schema_cache, &db.properties);
        if !corrected.is_null() {
            query["filter"] = corrected;
        }
    }

    if target.date_filter.unwrap_or(false) {
        let date_prop = db.properties.get("date").map(|s| s.as_str()).unwrap_or("Date");
        if let Some(df) = crate::util::date_filter::build_date_filter(range, Some(date_prop)) {
            if query.get("filter").is_some() {
                let combined = serde_json::json!({
                    "and": [query["filter"].clone(), df]
                });
                query["filter"] = combined;
            } else {
                query["filter"] = df;
            }
        }
    }

    if let Some(sort) = override_sort.or(target.sort.as_ref()) {
        query["sorts"] = sort.clone();
    }

    let meta = serde_json::json!({
        "applied_filters": {
            "database": db_key,
            "static_filter": static_filter,
            "override_filter": override_filter,
            "final_filter": final_filter,
            "sort": override_sort.or(target.sort.as_ref()),
            "description": target.filter_description()
        }
    });

    (query, meta)
}

/// Execute a briefing for a list of targets using resolve_db.
async fn execute_briefing_targets(
    targets: &[crate::config::BriefingTarget],
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
    range: &str,
    overrides: &Option<std::collections::HashMap<String, serde_json::Value>>,
) -> (serde_json::Value, Vec<String>) {
    let mut data = serde_json::json!({});
    let mut all_meta: Vec<serde_json::Value> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for target in targets {
        let db = match crate::config::resolve_db(config, &target.db) {
            Some(db) => db,
            None => {
                errors.push(format!("Unknown database in briefing: {}", target.db));
                continue;
            }
        };
        let target_override = overrides.as_ref().and_then(|o| o.get(&target.db));
        let ov_filter = target_override.and_then(|o| o.get("filter"));
        let ov_sort = target_override.and_then(|o| o.get("sort"));
        let (query, meta) = build_target_query(target, db, &target.db, range, schema_cache, ov_filter, ov_sort);
        all_meta.push(meta);
        match notion.query_data_source(db.ds_id(), &query).await {
            Ok(result) => {
                let items: Vec<serde_json::Value> = result.results.iter()
                    .map(|p| {
                        let title = crate::transform::extract_title(p);
                        serde_json::json!({ "title": title, "id": p.id })
                    }).collect();
                data[&target.db] = serde_json::json!(items);
            }
            Err(e) => {
                errors.push(format!("{}: {}", target.db, e));
            }
        }
    }

    if !errors.is_empty() {
        data["_errors"] = serde_json::json!(errors.clone());
    }
    if !all_meta.is_empty() {
        data["_meta"] = serde_json::json!({ "per_database": all_meta });
    }

    (data, errors)
}

pub async fn execute(
    params: &IntelligenceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let range = params.range.as_deref().unwrap_or("this_week");

    match params.mode.as_str() {
        "role" => {
            let role_display = params.role.as_deref().unwrap_or("CEO");
            let role_key = role_display.to_lowercase();
            let targets = config.briefings.as_ref()
                .and_then(|b| b.roles.get(role_key.as_str()))
                .ok_or_else(|| format!("Unknown role: {} — valid roles: CEO, COO, CMO, CRO, CFO, CHO", role_display))?;

            let mut data = serde_json::json!({
                "briefing_type": "role",
                "role": role_display,
                "range": range
            });

            let (briefing_data, _errors) = execute_briefing_targets(targets, config, notion, schema_cache, range, &params.overrides).await;
            // Merge briefing data into the output
            if let Some(obj) = briefing_data.as_object() {
                for (k, v) in obj {
                    data[k] = v.clone();
                }
            }

            Ok(crate::toon_format::encode(&data))
        }
        "module" => {
            let module_display = params.module.as_deref().unwrap_or("productivity");
            let module_key = module_display.to_lowercase();
            let targets = config.briefings.as_ref()
                .and_then(|b| b.modules.get(module_key.as_str()))
                .ok_or_else(|| format!("Unknown module: {} — valid modules: productivity, health, strategic, financial, content, journaling", module_display))?;

            let mut data = serde_json::json!({
                "briefing_type": "module",
                "module": module_display,
                "range": range
            });

            // Execute config-driven targets from briefings config
            let (briefing_data, _errors) = execute_briefing_targets(targets, config, notion, schema_cache, range, &params.overrides).await;
            if let Some(obj) = briefing_data.as_object() {
                for (k, v) in obj {
                    data[k] = v.clone();
                }
            }

            // Query module-specific entry types for richer data
            // (e.g., health module filters Potentiator by Activity, Diet, Subjective entry types)
            let entry_type_filters = module_entry_type_filters(&module_key);
            if !entry_type_filters.is_empty() {
                let mut entry_type_data = serde_json::json!({});
                let mut et_errors: Vec<String> = Vec::new();
                for (db_key, entry_types) in &entry_type_filters {
                    if let Some(db) = crate::config::resolve_db(config, db_key) {
                        let mut db_entries = serde_json::json!({});
                        // Use the DB's configured entry_type_property (authoritative) —
                        // fall back to "entry_type" alias auto-discovered by SchemaCache.
                        let et_notion_name = db.entry_type_notion_name()
                            .unwrap_or("Entry Type");
                        // Get the ACTUAL Notion property type from the live schema
                        // (auto-discovered). Falls back to the DB's deprecated config
                        // entry_type_property_type, then to "select".
                        let et_prop_type = schema_cache
                            .get_prop_type(db_key, et_notion_name)
                            .or_else(|| schema_cache.get_prop_type(db_key, "entry_type"))
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| db.entry_type_property_type.clone());
                        for entry_type in entry_types {
                            let mut query = serde_json::json!({ "page_size": 20 });
                            // Build entry-type filter using the correct property type:
                            //   - select     → {"select": {"equals": ...}}
                            //   - multi_select → {"multi_select": {"contains": ...}}
                            let filter = match et_prop_type.as_str() {
                                "multi_select" => serde_json::json!({
                                    "property": et_notion_name,
                                    "multi_select": { "contains": entry_type }
                                }),
                                _ => serde_json::json!({
                                    "property": et_notion_name,
                                    "select": { "equals": entry_type }
                                }),
                            };
                            query["filter"] = filter;
                            match notion.query_data_source(db.ds_id(), &query).await {
                                Ok(result) => {
                                    let items: Vec<serde_json::Value> = result.results.iter()
                                        .map(|p| serde_json::json!({
                                            "title": crate::transform::extract_title(p),
                                            "id": p.id,
                                        })).collect();
                                    db_entries[entry_type] = serde_json::json!({
                                        "entries": items,
                                        "count": items.len(),
                                    });
                                }
                                Err(e) => et_errors.push(format!("{}.{}: {}", db_key, entry_type, e)),
                            }
                        }
                        entry_type_data[db_key] = db_entries;
                    }
                }
                if !entry_type_data.as_object().unwrap_or(&serde_json::Map::new()).is_empty() {
                    data["entry_types"] = entry_type_data;
                }
                if !et_errors.is_empty() {
                    let existing_errors = data.get("_errors")
                        .and_then(|e| e.as_array().cloned())
                        .unwrap_or_default();
                    let mut all_errors: Vec<serde_json::Value> = existing_errors;
                    for e in et_errors {
                        all_errors.push(serde_json::json!(e));
                    }
                    data["_errors"] = serde_json::json!(all_errors);
                }
            }

            Ok(crate::toon_format::encode(&data))
        }
        "lesser_cycle" => {
            let mut data = serde_json::json!({
                "briefing_type": "lesser_cycle",
                "description": "Current-stage energy flow — cycle reservoirs from holonic config",
                "range": range
            });
            let mut errors: Vec<String> = Vec::new();
            // Derive cycle reservoirs from config instead of hardcoding
            let lesser_keys = config.cycle_reservoirs("lesser");
            for key in &lesser_keys {
                if let Some(db) = crate::config::get_db(config, key) {
                    let mut query = serde_json::json!({ "page_size": 20 });
                    if let Some(date_prop) = db.properties.get("date") {
                        if let Some(df) = crate::util::date_filter::build_date_filter(range, Some(date_prop)) {
                            query["filter"] = df;
                        }
                    }
                    match notion.query_data_source(db.ds_id(), &query).await {
                        Ok(result) => {
                            let items: Vec<serde_json::Value> = result.results.iter()
                                .map(|p| serde_json::json!({
                                    "title": crate::transform::extract_title(p),
                                    "id": p.id,
                                    "archetype": db.archetype.as_deref().unwrap_or("unknown"),
                                    "currency_in": db.currency_in.as_deref().unwrap_or("?"),
                                    "currency_out": db.currency_out.as_deref().unwrap_or("?"),
                                })).collect();
                            data[key] = serde_json::json!({ "entries": items, "count": items.len() });
                        }
                        Err(e) => errors.push(format!("{}: {}", key, e)),
                    }
                }
            }
            if !errors.is_empty() { data["_errors"] = serde_json::json!(errors); }
            Ok(crate::toon_format::encode(&data))
        }
        "greater_cycle" => {
            let mut data = serde_json::json!({
                "briefing_type": "greater_cycle",
                "description": "All-stage evolutionary tension — cycle reservoirs from holonic config",
                "range": range
            });
            let mut errors: Vec<String> = Vec::new();
            // Derive cycle reservoirs from config instead of hardcoding
            let greater_keys = config.cycle_reservoirs("greater");
            for key in &greater_keys {
                if let Some(db) = crate::config::get_db(config, key) {
                    let query = serde_json::json!({ "page_size": 20 });
                    match notion.query_data_source(db.ds_id(), &query).await {
                        Ok(result) => {
                            let items: Vec<serde_json::Value> = result.results.iter()
                                .map(|p| serde_json::json!({
                                    "title": crate::transform::extract_title(p),
                                    "id": p.id,
                                    "archetype": db.archetype.as_deref().unwrap_or("unknown"),
                                })).collect();
                            data[key] = serde_json::json!({ "entries": items, "count": items.len() });
                        }
                        Err(e) => errors.push(format!("{}: {}", key, e)),
                    }
                }
            }
            if !errors.is_empty() { data["_errors"] = serde_json::json!(errors); }
            Ok(crate::toon_format::encode(&data))
        }
        "nexus" => {
            let mut data = serde_json::json!({
                "briefing_type": "nexus",
                "description": "Contact-boundary transmutation: all currencies",
                "range": range
            });
            let mut errors: Vec<String> = Vec::new();

            // Discover nexus by archetype from config — no hardcoded name
            let nexus_key = config.reservoir_by_archetype("nexus").map(|(k, _)| k.to_string());
            if let Some(ref nexus_k) = nexus_key {
                if let Some(db) = crate::config::get_db(config, nexus_k) {
                    let mut query = serde_json::json!({ "page_size": 20 });
                    if let Some(date_prop) = db.properties.get("date") {
                        if let Some(df) = crate::util::date_filter::build_date_filter(range, Some(date_prop)) {
                            query["filter"] = df;
                        }
                    }
                    match notion.query_data_source(db.ds_id(), &query).await {
                        Ok(result) => {
                            let items: Vec<serde_json::Value> = result.results.iter()
                                .map(|p| {
                                    let title = crate::transform::extract_title(p);
                                    let category = crate::transform::extract_string(p, "Category");
                                    let kind = crate::transform::extract_string(p, "Kind");
                                    serde_json::json!({
                                        "title": title,
                                        "id": p.id,
                                        "category": category,
                                        "kind": kind
                                    })
                                }).collect();

                            let mut category_dist: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
                            let mut kind_dist: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
                            for item in &items {
                                let cat = item["category"].as_str().unwrap_or("unknown");
                                let kind = item["kind"].as_str().unwrap_or("unknown");
                                *category_dist.entry(cat.to_string()).or_insert(0) += 1;
                                *kind_dist.entry(kind.to_string()).or_insert(0) += 1;
                            }

                            data["nexus"] = serde_json::json!({
                                "entries": items,
                                "count": items.len(),
                                "transmutation_analysis": {
                                    "category_distribution": category_dist,
                                    "kind_distribution": kind_dist,
                                    "currencies_active": config.holonic.as_ref()
                                        .map(|h| serde_json::json!(h.currencies))
                                        .unwrap_or(serde_json::json!(["Catalyst", "Experience", "Transformation", "Choice"])),
                                    "interpretation": nexus_interpretation(items.len(), &category_dist)
                                }
                            });
                        }
                        Err(e) => { errors.push(format!("nexus: {}", e)); }
                    }
                }
            }

            if !errors.is_empty() { data["_errors"] = serde_json::json!(errors); }
            Ok(crate::toon_format::encode(&data))
        }
        "drive_balance" => {
            crate::tools::drive_assessment::execute(
                &crate::tools::drive_assessment::DriveAssessmentParams {
                    boundary: "both".to_string(),
                    range: Some(range.to_string()),
                },
                config, notion,
            ).await
        }
        "reservoir_health" => {
            crate::tools::health_metrics::execute(
                &crate::tools::health_metrics::HealthMetricsParams {
                    metric: "all".to_string(),
                    range: Some(range.to_string()),
                },
                config, notion, schema_cache,
            ).await
        }
        _ => Err(format!("Unknown mode: {}", params.mode)),
    }
}

fn nexus_interpretation(count: usize, categories: &std::collections::HashMap<String, i64>) -> String {
    if count == 0 {
        "No nexus entries in range — contact-boundary is dormant".to_string()
    } else if count > 20 {
        format!("High nexus activity ({count} entries) — active transmutation across all currencies")
    } else {
        let dominant = categories.iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| format!("{} ({})", k, v))
            .unwrap_or_else(|| "unknown".to_string());
        format!("Moderate nexus activity ({count} entries) — dominant category: {}", dominant)
    }
}
