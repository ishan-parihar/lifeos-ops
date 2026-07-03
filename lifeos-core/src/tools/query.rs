//! Query tool — database query with filters, sorts, and presets

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

/// Property types that cannot be used in Notion filters
const NON_FILTERABLE: &[&str] = &["formula", "rollup", "button", "unique_id"];

/// Query tool parameters
#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub database: String,
    pub filter_property: Option<String>,
    pub filter_value: Option<String>,
    pub filter_type: Option<String>,
    pub sort_property: Option<String>,
    pub sort_direction: Option<String>,
    pub limit: Option<u32>,
    pub return_properties: Option<Vec<String>>,
    pub preset: Option<String>,
    /// Filter by entry type within a database (e.g., "Activity" for Potentiator, "Project" for GreatWay)
    pub entry_type: Option<String>,
    /// Query all reservoirs in a cycle ("lesser" or "greater")
    pub cycle: Option<String>,
    /// Filter by Archetype Role (Matrix/Potentiator/Catalyst/Experience/Significator/Transformation/Great Way/Choice)
    #[serde(default)]
    pub archetype: Option<String>,
    /// Filter by Complex (Mind/Body/Spirit/None)
    #[serde(default)]
    pub complex: Option<String>,
    /// Filter by Drive Activation (Agency/Communion/Eros/Agape) — multi_select contains
    #[serde(default)]
    pub drive: Option<String>,
    /// Filter by Shadow Pattern (None/Dark-Addiction/Dark-Allergy/Golden-Addiction/Golden-Allergy)
    #[serde(default)]
    pub shadow: Option<String>,
    /// Filter by Digestion Stage (1-9 or full name)
    #[serde(default)]
    pub digestion_stage: Option<String>,
}

/// Tool schema for MCP tools/list — enriched with schema cache context
pub fn schema(_config: &LifeOSConfig, _schema_cache: &SchemaCache) -> serde_json::Value {
    let obj = serde_json::json!({
        "type": "object",
        "properties": {
            "database": { "type": "string", "description": "Database key (matrix, potentiator, significator, greatway, nexus)" },
            "filter_property": { "type": "string", "description": "Config-key of the property to filter on. Check database schema below for valid keys." },
            "filter_value": { "type": "string", "description": "Value to filter for" },
            "filter_type": { "type": "string", "enum": ["select", "status", "rich_text", "title", "date", "checkbox", "url", "email", "phone_number", "number", "multi_select"], "description": "Property type for filter — use the actual Notion column type, not 'select' for status fields or vice versa" },
            "sort_property": { "type": "string", "description": "Property to sort by" },
            "sort_direction": { "type": "string", "enum": ["ascending", "descending"] },
            "limit": { "type": "integer", "minimum": 1, "maximum": 100, "description": "Max results (default: 50)" },
            "return_properties": { "type": "array", "items": { "type": "string" }, "description": "Specific config-keys to return" },
            "preset": { "type": "string", "enum": ["active", "this_week", "this_month", "needs_review"], "description": "Intelligent preset query" },
            "entry_type": { "type": "string", "description": "Filter by entry type within a database. Use get_schema to discover valid entry types per DB. E.g., 'Activity' for potentiator, 'Project' for greatway, 'Person' for significator." },
            "cycle": { "type": "string", "enum": ["lesser", "greater"], "description": "Query all reservoirs in a cycle at once. lesser = matrix+potentiator (current-stage), greater = significator+greatway (all-stage)" }
        },
        "required": ["database"]
    });

    obj
}

/// Execute the query tool
pub async fn execute(
    params: &QueryParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    // If cycle is specified, query across all reservoirs in that cycle
    if let Some(ref cycle) = params.cycle {
        return execute_cycle_query(cycle, params, config, notion, schema_cache).await;
    }

    // Resolve database
    let db = match crate::config::resolve_db(config, &params.database) {
        Some(db) => db,
        None => return Err(format!("Unknown database: {}", params.database)),
    };
    let ds_id = db.ds_id();
    let name = &db.name;

    let limit = params.limit.unwrap_or(50).min(100) as u64;
    let mut body = serde_json::json!({ "page_size": limit });

    // Handle entry_type filter — use auto-discovered property name + type.
    // `db.notion_prop("entry_type")` returns the Notion property name (e.g. "Entry Type"
    // for Matrix, "Item Type" for GreatWay, "Category" for Nexus). Falls back to the
    // DB's configured `entry_type_property` if discovery hasn't run.
    if let Some(ref entry_type) = params.entry_type {
        // Try entry_type_property first, then auto-discovered "entry_type" alias
        let et_notion_name: Option<String> = db.entry_type_property.clone()
            .or_else(|| db.notion_prop("entry_type").map(|s| s.to_string()));
        if let Some(et_prop) = et_notion_name {
            // Get the ACTUAL property type from the live schema (auto-discovered).
            // Falls back to the deprecated config `entry_type_property_type`.
            let actual_type = schema_cache.get_prop_type(&params.database, &et_prop)
                .or_else(|| schema_cache.get_prop_type(&params.database, "entry_type"))
                .unwrap_or(db.entry_type_property_type.as_str());
            body["filter"] = build_filter(&et_prop, actual_type, entry_type);
        } else {
            return Err(format!(
                "Database '{}' has no entry_type_property configured and no auto-discovered \
'entry_type' property. Set `entry_type_property` in the config or run `lifeos discover` \
to auto-detect the property name.",
                params.database
            ));
        }
    }

    // ── Semantic typing filters (v0.8+ ontology properties) ──
    // These are additive AND filters — combined with entry_type/preset filters via "and".
    let mut semantic_filters: Vec<serde_json::Value> = Vec::new();

    // Archetype Role filter (select equals)
    if let Some(ref archetype) = params.archetype {
        if let Some(notion_name) = db.notion_prop("archetype_role").or(db.notion_prop("Archetype Role")) {
            let resolved = resolve_enum_value(schema_cache, &params.database, "archetype_role", archetype);
            let resolved = if &resolved == archetype {
                resolve_enum_value(schema_cache, &params.database, notion_name, archetype)
            } else { resolved };
            semantic_filters.push(serde_json::json!({
                "property": notion_name,
                "select": { "equals": resolved }
            }));
        }
    }

    // Complex filter (select equals)
    if let Some(ref complex) = params.complex {
        if let Some(notion_name) = db.notion_prop("complex").or(db.notion_prop("Complex")) {
            let resolved = resolve_enum_value(schema_cache, &params.database, "complex", complex);
            let resolved = if &resolved == complex {
                resolve_enum_value(schema_cache, &params.database, notion_name, complex)
            } else { resolved };
            semantic_filters.push(serde_json::json!({
                "property": notion_name,
                "select": { "equals": resolved }
            }));
        }
    }

    // Drive Activation filter (multi_select contains)
    if let Some(ref drive) = params.drive {
        if let Some(notion_name) = db.notion_prop("drive_activation").or(db.notion_prop("Drive Activation")) {
            semantic_filters.push(serde_json::json!({
                "property": notion_name,
                "multi_select": { "contains": drive }
            }));
        }
    }

    // Shadow Pattern filter (select equals)
    if let Some(ref shadow) = params.shadow {
        if let Some(notion_name) = db.notion_prop("shadow_pattern").or(db.notion_prop("Shadow Pattern")) {
            let resolved = resolve_enum_value(schema_cache, &params.database, "shadow_pattern", shadow);
            let resolved = if &resolved == shadow {
                resolve_enum_value(schema_cache, &params.database, notion_name, shadow)
            } else { resolved };
            semantic_filters.push(serde_json::json!({
                "property": notion_name,
                "select": { "equals": resolved }
            }));
        }
    }

    // Digestion Stage filter (select equals) — supports stage number or full name
    if let Some(ref stage) = params.digestion_stage {
        if let Some(notion_name) = db.notion_prop("digestion_stage").or(db.notion_prop("Digestion Stage")) {
            // Resolve stage input: "1" → "1 - Latent State", or pass through if already full
            let resolved_stage = resolve_digestion_stage(stage);
            semantic_filters.push(serde_json::json!({
                "property": notion_name,
                "select": { "equals": resolved_stage }
            }));
        }
    }

    // Combine semantic filters with any existing filter via "and"
    if !semantic_filters.is_empty() {
        if let Some(existing) = body.get("filter").cloned() {
            if semantic_filters.len() == 1 {
                body["filter"] = serde_json::json!({ "and": [existing, semantic_filters[0]] });
            } else {
                let mut all = vec![existing];
                all.extend(semantic_filters);
                body["filter"] = serde_json::json!({ "and": all });
            }
        } else if semantic_filters.len() == 1 {
            body["filter"] = semantic_filters[0].clone();
        } else {
            body["filter"] = serde_json::json!({ "and": semantic_filters });
        }
    }

    // Handle preset filters — schema-aware: use auto-discovered property names
    if params.entry_type.is_none() {
        if let Some(ref preset) = params.preset {
            let now = chrono::Utc::now();
            match preset.as_str() {
                "active" => {
                    if let Some(prop) = db.notion_prop("status") {
                        let actual_type = schema_cache.get_prop_type(&params.database, "status")
                            .or_else(|| schema_cache.get_prop_type(&params.database, prop))
                            .unwrap_or("select");
                        if !NON_FILTERABLE.contains(&actual_type) {
                            // Emoji-aware matching: if the user passes "Active" but the actual
                            // option is "✅ Activated", match by alpha substring. See
                            // `resolve_enum_value` for details.
                            let resolved = resolve_enum_value(schema_cache, &params.database, "status", "Active");
                            body["filter"] = build_filter_with_prop(prop, actual_type, &resolved);
                        }
                    }
                }
                "this_week" => {
                    if let Some(prop) = db.notion_prop("date") {
                        let start = (now - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
                        body["filter"] = serde_json::json!({
                            "property": prop, "date": { "on_or_after": start }
                        });
                    }
                }
                "this_month" => {
                    if let Some(prop) = db.notion_prop("date") {
                        let start = (now - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
                        body["filter"] = serde_json::json!({
                            "property": prop, "date": { "on_or_after": start }
                        });
                    }
                }
                "needs_review" => {
                    if let Some(prop) = db.notion_prop("status") {
                        let actual_type = schema_cache.get_prop_type(&params.database, "status")
                            .or_else(|| schema_cache.get_prop_type(&params.database, prop))
                            .unwrap_or("select");
                        if !NON_FILTERABLE.contains(&actual_type) {
                            let resolved = resolve_enum_value(schema_cache, &params.database, "status", "Needs Review");
                            body["filter"] = build_filter_with_prop(prop, actual_type, &resolved);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Handle manual filter — resolve config_key → Notion property name (via auto-discovery),
    // validate type against schema, and apply emoji-aware matching for enum types.
    if let (Some(ref prop_key), Some(ref val)) = (&params.filter_property, &params.filter_value) {
        // Try direct config-key lookup (auto-discovered), then treat the input itself
        // as a Notion property name (so users can pass either form).
        let notion_prop = db.notion_prop(prop_key).unwrap_or(prop_key.as_str());
        let filter_type = params.filter_type.as_deref()
            .or_else(|| schema_cache.get_prop_type(&params.database, prop_key))
            .or_else(|| schema_cache.get_prop_type(&params.database, notion_prop))
            .unwrap_or("rich_text");
        // Emoji-aware enum value resolution: if the user types "Identified" but
        // the actual Notion status option is "💡 Identified", match by alpha substring.
        let resolved_val = resolve_enum_value(schema_cache, &params.database, prop_key, val);
        let resolved_val = if &resolved_val == val {
            // Try with the notion_prop too (in case prop_key was the Notion name)
            resolve_enum_value(schema_cache, &params.database, notion_prop, val)
        } else {
            resolved_val
        };
        body["filter"] = build_filter(notion_prop, filter_type, &resolved_val);
    }

    // Handle sort
    if let Some(ref sort_prop) = params.sort_property {
        // Resolve config_key → Notion name for sort property too
        let notion_sort = db.notion_prop(sort_prop).unwrap_or(sort_prop.as_str());
        let direction = params.sort_direction.as_deref().unwrap_or("descending");
        body["sorts"] = serde_json::json!([
            { "property": notion_sort, "direction": direction }
        ]);
    }

    // Execute query
    let result = notion.query_data_source(&ds_id, &body).await?;

    let items: Vec<serde_json::Value> = result.results.iter().map(|page| {
        let title = crate::transform::extract_title(page);
        let mut item = serde_json::json!({ "title": title, "id": page.id });

        if let Some(ref props) = params.return_properties {
            for prop_key in props {
                // Resolve config_key → Notion property name for extraction
                if let Some(notion_name) = db.notion_prop(prop_key) {
                    if let Some(val) = crate::transform::extract_property_value(page, notion_name) {
                        if !val.is_null() {
                            item[prop_key] = val;
                        }
                    }
                }
            }
        }
        item
    }).collect();

    // Build result with TOON encoding
    let mut data = serde_json::Map::new();
    data.insert("__schema".into(), serde_json::json!({
        "database": &params.database, "name": name
    }));
    data.insert(params.database.clone(), serde_json::json!(items));
    data.insert("meta".into(), serde_json::json!({
        "count": result.results.len(),
        "has_more": result.has_more
    }));
    let toon_data = serde_json::Value::Object(data);

    Ok(crate::toon_format::encode(&toon_data))
}

/// Resolve a user-supplied enum value against the actual Notion option list,
/// handling emoji-prefixed options (Bug C fix).
///
/// Notion's Nexus `Status` property has emoji-prefixed options like
/// `💡 Identified`, `✅ Activated`, `🏆 Capitalized`, `🧊 Archived`. The user
/// types `Identified` (no emoji) and expects a match. This function:
///   1. Returns the value as-is if it exactly matches a Notion option.
///   2. Otherwise looks for an option whose alpha-only form equals the value
///      (e.g. "Identified" matches "💡 Identified").
///   3. Otherwise looks for an option whose alpha-only form contains the value
///      case-insensitively.
///   4. Returns the original value if no match found (Notion will return zero
///      results — better than silently matching the wrong option).
fn resolve_enum_value(
    schema_cache: &SchemaCache,
    db_key: &str,
    prop_key: &str,
    value: &str,
) -> String {
    // Direct match — fast path
    if let Some(opts) = schema_cache.get_enum_options(db_key, prop_key) {
        if opts.iter().any(|o| o == value) {
            return value.to_string();
        }
        // Try alpha-only matching: strip non-alphanumeric chars from each option
        // and compare case-insensitively.
        let value_alpha = alpha_only(value);
        let value_lower = value.to_lowercase();
        // Exact alpha-only match
        for opt in opts {
            let opt_alpha = alpha_only(opt);
            if opt_alpha == value_alpha && !opt_alpha.is_empty() {
                return opt.clone();
            }
        }
        // Substring match (case-insensitive)
        for opt in opts {
            let opt_lower = opt.to_lowercase();
            if opt_lower.contains(&value_lower) && !value_lower.is_empty() {
                return opt.clone();
            }
        }
    }
    value.to_string()
}

/// Strip non-alphanumeric/space chars and trim, for emoji-aware matching.
fn alpha_only(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .trim()
        .to_lowercase()
}

/// Resolve a digestion stage input ("1", "1 - Latent State", "Latent State") to the
/// full Notion option name ("1 - Latent State").
fn resolve_digestion_stage(input: &str) -> String {
    let stages = [
        "1 - Latent State",
        "2 - Boundary Contact",
        "3 - Matrix Ingestion",
        "4 - Matrix Digestion",
        "5 - Potentiator Ingestion",
        "6 - Potentiator Digestion",
        "7 - Significator Accumulation",
        "8 - Transformation Threshold",
        "9 - Choice & Rewrite",
    ];
    // Exact match
    for s in &stages {
        if *s == input { return s.to_string(); }
    }
    // Match by leading number ("1" → "1 - Latent State")
    for s in &stages {
        if s.starts_with(&format!("{} ", input)) || s.starts_with(&format!("{} -", input)) {
            return s.to_string();
        }
    }
    // Match by substring (case-insensitive)
    let lower = input.to_lowercase();
    for s in &stages {
        if s.to_lowercase().contains(&lower) {
            return s.to_string();
        }
    }
    // No match — return as-is (Notion will return 0 results, which is the right behavior)
    input.to_string()
}

/// Build filter using the Notion property name (already resolved).
fn build_filter(property: &str, filter_type: &str, value: &str) -> serde_json::Value {
    match filter_type {
        "select" => serde_json::json!({ "property": property, "select": { "equals": value } }),
        "status" => serde_json::json!({ "property": property, "status": { "equals": value } }),
        "rich_text" => serde_json::json!({ "property": property, "rich_text": { "contains": value } }),
        "title" => serde_json::json!({ "property": property, "title": { "contains": value } }),
        "date" => serde_json::json!({ "property": property, "date": { "equals": value } }),
        "checkbox" => serde_json::json!({ "property": property, "checkbox": { "equals": value == "true" } }),
        "url" => serde_json::json!({ "property": property, "url": { "contains": value } }),
        "email" => serde_json::json!({ "property": property, "email": { "contains": value } }),
        "phone_number" => serde_json::json!({ "property": property, "phone_number": { "contains": value } }),
        "number" => {
            if let Ok(n) = value.parse::<f64>() {
                serde_json::json!({ "property": property, "number": { "equals": n } })
            } else {
                serde_json::json!({ "property": property, "rich_text": { "contains": value } })
            }
        }
        "multi_select" => serde_json::json!({ "property": property, "multi_select": { "contains": value } }),
        _ => serde_json::json!({ "property": property, "rich_text": { "contains": value } }),
    }
}

/// Build filter given a Notion property name and schema-derived type.
fn build_filter_with_prop(notion_prop: &str, actual_type: &str, value: &str) -> serde_json::Value {
    build_filter(notion_prop, actual_type, value)
}

// ─── query_override tool ────────────────────────────────────────────

/// Parameters for query_override — AI-validated query with override capability
#[derive(Debug, Deserialize)]
pub struct QueryOverrideParams {
    pub database: String,
    pub filter: Option<serde_json::Value>,
    pub sort: Option<serde_json::Value>,
    pub limit: Option<u32>,
    pub return_properties: Option<Vec<String>>,
    /// Filter by entry type within a database
    #[serde(default)]
    pub entry_type: Option<String>,
}

/// MCP schema for query_override tool
pub fn schema_override(_config: &LifeOSConfig, _schema_cache: &SchemaCache) -> serde_json::Value {
    let obj = serde_json::json!({
        "type": "object",
        "properties": {
            "database": { "type": "string", "description": "Database key (matrix, potentiator, significator, greatway, nexus)" },
            "entry_type": { "type": "string", "description": "Filter by entry type within a database. Use get_schema to discover valid entry types. E.g., 'Activity' for potentiator, 'Project' for greatway." },
            "filter": {
                "type": "object",
                "description": "Notion filter object. Validate property names and types against the database schema before passing.",
                "properties": {
                    "property": { "type": "string", "description": "Config-key of the property (will be resolved to Notion name)" },
                    "operator": { "type": "string", "enum": ["equals", "contains", "starts_with", "ends_with", "before", "after", "on_or_before", "on_or_after"], "description": "Filter operator" },
                    "value": { "type": "string", "description": "Filter value" }
                }
            },
            "sort": {
                "type": "object",
                "description": "Sort configuration",
                "properties": {
                    "property": { "type": "string", "description": "Config-key to sort by" },
                    "direction": { "type": "string", "enum": ["ascending", "descending"] }
                }
            },
            "limit": { "type": "integer", "minimum": 1, "maximum": 100, "description": "Max results (default: 50)" },
            "return_properties": { "type": "array", "items": { "type": "string" }, "description": "Specific config-keys to return" }
        },
        "required": ["database"]
    });

    obj
}

/// Execute query_override — validates filter against SchemaCache, then executes
pub async fn execute_override(
    params: &QueryOverrideParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let db = match crate::config::resolve_db(config, &params.database) {
        Some(db) => db,
        None => return Err(format!("Unknown database: {}", params.database)),
    };
    let ds_id = db.ds_id();
    let db_name = &db.name;

    let limit = params.limit.unwrap_or(50).min(100) as u64;
    let mut body = serde_json::json!({ "page_size": limit });

    // Handle entry_type filter — use auto-discovered property name + type
    if let Some(ref entry_type) = params.entry_type {
        let et_notion_name: Option<String> = db.entry_type_property.clone()
            .or_else(|| db.notion_prop("entry_type").map(|s| s.to_string()));
        if let Some(et_prop) = et_notion_name {
            let actual_type = schema_cache.get_prop_type(&params.database, &et_prop)
                .or_else(|| schema_cache.get_prop_type(&params.database, "entry_type"))
                .unwrap_or(db.entry_type_property_type.as_str());
            body["filter"] = build_filter(&et_prop, actual_type, entry_type);
        } else {
            return Err(format!(
                "Database '{}' has no entry_type property. Set `entry_type_property` in config or run `lifeos discover`.",
                params.database
            ));
        }
    }

    if let Some(ref filter_obj) = params.filter {
        let prop_key = filter_obj.get("property")
            .and_then(|v| v.as_str())
            .ok_or("Filter missing 'property' field")?;
        let operator = filter_obj.get("operator")
            .and_then(|v| v.as_str())
            .unwrap_or("equals");
        let value = filter_obj.get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Use auto-discovered property name lookup
        let notion_prop = db.notion_prop(prop_key).unwrap_or(prop_key);

        let prop_type = schema_cache.get_prop_type(&params.database, prop_key)
            .or_else(|| schema_cache.get_prop_type(&params.database, notion_prop))
            .unwrap_or("rich_text");
        if NON_FILTERABLE.contains(&prop_type) {
            // Build list of filterable Notion property names from auto-discovery
            let filterable: Vec<String> = schema_cache.get_property_names(&params.database).iter()
                .filter(|n| {
                    schema_cache.get_prop_type(&params.database, n)
                        .map(|t| !NON_FILTERABLE.contains(&t))
                        .unwrap_or(true)
                })
                .cloned()
                .collect();
            return Err(format!(
                "Property '{}' is type '{}' which cannot be filtered. Valid filterable properties: {}",
                prop_key, prop_type, filterable.join(", ")
            ));
        }

        // Emoji-aware enum value resolution
        let resolved_val = resolve_enum_value(schema_cache, &params.database, prop_key, value);
        let resolved_val = if resolved_val == value {
            resolve_enum_value(schema_cache, &params.database, notion_prop, value)
        } else {
            resolved_val
        };

        body["filter"] = match operator {
            "equals" => build_filter(notion_prop, prop_type, &resolved_val),
            "contains" => serde_json::json!({ "property": notion_prop, "rich_text": { "contains": value } }),
            "starts_with" => serde_json::json!({ "property": notion_prop, "rich_text": { "starts_with": value } }),
            "ends_with" => serde_json::json!({ "property": notion_prop, "rich_text": { "ends_with": value } }),
            "before" => serde_json::json!({ "property": notion_prop, "date": { "before": value } }),
            "after" => serde_json::json!({ "property": notion_prop, "date": { "after": value } }),
            "on_or_before" => serde_json::json!({ "property": notion_prop, "date": { "on_or_before": value } }),
            "on_or_after" => serde_json::json!({ "property": notion_prop, "date": { "on_or_after": value } }),
            _ => build_filter(notion_prop, prop_type, &resolved_val),
        };
    }

    if let Some(ref sort_obj) = params.sort {
        let sort_prop = sort_obj.get("property")
            .and_then(|v| v.as_str())
            .unwrap_or("date");
        let direction = sort_obj.get("direction")
            .and_then(|v| v.as_str())
            .unwrap_or("descending");
        // Resolve config_key → Notion name for sort
        let notion_sort = db.notion_prop(sort_prop).unwrap_or(sort_prop);
        body["sorts"] = serde_json::json!([
            { "property": notion_sort, "direction": direction }
        ]);
    }

    let result = notion.query_data_source(&ds_id, &body).await?;

    let items: Vec<serde_json::Value> = result.results.iter().map(|page| {
        let title = crate::transform::extract_title(page);
        let mut item = serde_json::json!({ "title": title, "id": page.id });

        if let Some(ref props) = params.return_properties {
            for prop_key in props {
                if let Some(notion_name) = db.notion_prop(prop_key) {
                    if let Some(val) = crate::transform::extract_property_value(page, notion_name) {
                        if !val.is_null() {
                            item[prop_key] = val;
                        }
                    }
                }
            }
        }
        item
    }).collect();

    let mut data = serde_json::Map::new();
    data.insert("__schema".into(), serde_json::json!({
        "database": &params.database, "name": db_name
    }));
    data.insert(params.database.clone(), serde_json::json!(items));
    data.insert("meta".into(), serde_json::json!({
        "count": result.results.len(),
        "has_more": result.has_more,
        "applied_filter": params.filter,
        "applied_sort": params.sort,
    }));
    let toon_data = serde_json::Value::Object(data);

    Ok(crate::toon_format::encode(&toon_data))
}


// ─── Cycle & Reservoir Query Helpers ────────────────────────────────

/// Query across all reservoirs in a cycle (lesser or greater)
async fn execute_cycle_query(
    cycle: &str,
    params: &QueryParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let reservoir_keys = config.cycle_reservoirs(cycle);
    if reservoir_keys.is_empty() {
        return Err(format!("Unknown cycle: {}. Use 'lesser' or 'greater'.", cycle));
    }

    let mut all_items: Vec<serde_json::Value> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let limit = params.limit.unwrap_or(20).min(100);

    for res_key in &reservoir_keys {
        let db = match crate::config::resolve_db(config, res_key) {
            Some(db) => db,
            None => continue,
        };

        let mut body = serde_json::json!({ "page_size": limit });

        // Apply entry_type filter within each reservoir (if specified)
        // Uses auto-discovered property name + type.
        if let Some(ref entry_type) = params.entry_type {
            let et_notion_name: Option<String> = db.entry_type_property.clone()
                .or_else(|| db.notion_prop("entry_type").map(|s| s.to_string()));
            if let Some(et_prop) = et_notion_name {
                let actual_type = schema_cache.get_prop_type(res_key, &et_prop)
                    .or_else(|| schema_cache.get_prop_type(res_key, "entry_type"))
                    .unwrap_or(db.entry_type_property_type.as_str());
                body["filter"] = build_filter(&et_prop, actual_type, entry_type);
            }
        }

        // Apply preset if specified (only if entry_type not set)
        if params.entry_type.is_none() {
            if let Some(ref preset) = params.preset {
                let now = chrono::Utc::now();
                match preset.as_str() {
                    "this_week" => {
                        if let Some(prop) = db.notion_prop("date") {
                            let start = (now - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
                            body["filter"] = serde_json::json!({ "property": prop, "date": { "on_or_after": start } });
                        }
                    }
                    "this_month" => {
                        if let Some(prop) = db.notion_prop("date") {
                            let start = (now - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
                            body["filter"] = serde_json::json!({ "property": prop, "date": { "on_or_after": start } });
                        }
                    }
                    _ => {}
                }
            }
        }

        match notion.query_data_source(db.ds_id(), &body).await {
            Ok(result) => {
                for page in &result.results {
                    let title = crate::transform::extract_title(page);
                    all_items.push(serde_json::json!({
                        "title": title,
                        "id": page.id,
                        "reservoir": res_key,
                        "archetype": db.archetype.as_deref().unwrap_or("unknown"),
                        "currency_in": db.currency_in.as_deref().unwrap_or("?"),
                        "currency_out": db.currency_out.as_deref().unwrap_or("?"),
                    }));
                }
            }
            Err(e) => errors.push(format!("{}: {}", res_key, e)),
        }
    }

    let mut data = serde_json::Map::new();
    data.insert("__schema".into(), serde_json::json!({
        "cycle": cycle,
        "reservoirs": reservoir_keys
    }));
    data.insert("results".into(), serde_json::json!(all_items));
    data.insert("meta".into(), serde_json::json!({
        "count": all_items.len(),
        "cycle": cycle,
        "errors": errors
    }));

    Ok(crate::toon_format::encode(&serde_json::Value::Object(data)))
}


