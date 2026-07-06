//! `auto_enrich` tool — infer universal properties + parent links from entry-type.
//!
//! Solves the user's #1 pain: "how the hell can I also log these entries manually
//! AND do all the other work." Instead of forcing the user to set Archetype Role,
//! Complex, Drive Activation on every daily-log entry, this tool infers them
//! from the DB + entry-type using a deterministic rule map.
//!
//! Two modes:
//!   - `tag`  : set Archetype Role / Complex / Drive Activation on entries missing them.
//!              Safe, idempotent, never overwrites existing values.
//!   - `link` : DRY-RUN only — reports which parent relations WOULD be set per entry-type.
//!              Actual application deferred to v0.11 (requires multi-step resolution
//!              of "active parent" by status filter, which is risky to auto-apply).

use std::sync::Arc;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

#[derive(Debug, Deserialize)]
pub struct AutoEnrichParams {
    pub mode: String,
    pub database: Option<String>,
    pub limit: Option<u32>,
    #[serde(default)]
    pub apply: bool,
}

pub fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "mode": { "type": "string", "enum": ["tag", "link"], "description": "tag = set Archetype Role/Complex/Drive Activation; link = report which parent relations WOULD be set" },
            "database": { "type": "string", "description": "Optional DB key to filter. Omit to scan all 5." },
            "limit": { "type": "integer", "minimum": 1, "maximum": 500, "description": "Max entries per DB (default 50)" },
            "apply": { "type": "boolean", "description": "false (default) = dry-run; true = write changes to Notion (link mode always dry-run)" }
        },
        "required": ["mode"]
    })
}

// ── Rule map ─────────────────────────────────────────────────────────────────
// Returns (archetype_role, complex, drives) for a (db_key, entry_type) pair.
type Rule = (Option<&'static str>, Option<&'static str>, &'static [&'static str]);

fn rule_for(db_key: &str, entry_type: &str) -> Option<Rule> {
    match (db_key, entry_type) {
        // ── State (Matrix) ──
        ("matrix", "Pattern")     => Some((Some("Matrix"),    Some("Body"),    &["Agency"])),
        ("matrix", "Threshold")   => Some((Some("Matrix"),    Some("Mind"),    &["Agency"])),
        ("matrix", "Foundation")  => Some((Some("Matrix"),    Some("Spirit"),  &["Agape"])),

        // ── Possibility (Potentiator) — daily logs + future-pull ──
        ("potentiator", "Activity")   => Some((Some("Potentiator"), Some("Body"),    &["Eros"])),
        ("potentiator", "Diet")       => Some((Some("Potentiator"), Some("Body"),    &["Agape"])),
        ("potentiator", "Financial")  => Some((Some("Potentiator"), Some("Mind"),    &["Agency"])),
        ("potentiator", "Subjective") => Some((Some("Potentiator"), Some("Mind"),    &["Eros"])),
        ("potentiator", "Relational") => Some((Some("Potentiator"), Some("Spirit"),  &["Communion"])),
        ("potentiator", "Systemic")   => Some((Some("Potentiator"), Some("Mind"),    &["Agency"])),
        ("potentiator", "Observation")=> Some((Some("Potentiator"), Some("Mind"),    &[])),
        ("potentiator", "Goal")       => Some((Some("Potentiator"), Some("Mind"),    &["Eros"])),
        ("potentiator", "Vision")     => Some((Some("Potentiator"), Some("Spirit"),  &["Eros"])),
        ("potentiator", "Aspiration") => Some((Some("Potentiator"), Some("Spirit"),  &["Eros"])),

        // ── Process (Nexus) — contact-boundary ──
        ("nexus", "Opportunity")          => Some((Some("Catalyst"),        Some("Mind"),    &["Eros"])),
        ("nexus", "Risk")                 => Some((Some("Catalyst"),        Some("Mind"),    &[])),
        ("nexus", "Directive")            => Some((Some("Choice"),          Some("Mind"),    &["Agency"])),
        ("nexus", "Insight")              => Some((Some("Experience"),      Some("Mind"),    &[])),
        ("nexus", "Reflection")           => Some((Some("Experience"),      Some("Mind"),    &["Agape"])),
        ("nexus", "Integration")          => Some((Some("Experience"),      Some("Spirit"),  &["Agape"])),
        ("nexus", "Pattern")              => Some((Some("Experience"),      Some("Mind"),    &[])),
        ("nexus", "Note")                 => Some((Some("Catalyst"),        Some("Mind"),    &[])),
        ("nexus", "Knowledge-Category")   => Some((Some("Experience"),      Some("Mind"),    &[])),
        ("nexus", "Knowledge-Atom")       => Some((Some("Experience"),      Some("Mind"),    &[])),
        ("nexus", "Decision")             => Some((Some("Choice"),          Some("Mind"),    &["Agency"])),
        ("nexus", "Crisis")               => Some((Some("Transformation"),  Some("Body"),    &["Eros"])),
        ("nexus", "Transformation-Event") => Some((Some("Transformation"),  Some("Spirit"),  &["Eros"])),

        // ── Identity (Significator) ──
        ("significator", "Purpose")           => Some((Some("Significator"), Some("Spirit"),  &["Eros"])),
        ("significator", "Value")             => Some((Some("Significator"), Some("Mind"),    &["Agape"])),
        ("significator", "Principle")         => Some((Some("Significator"), Some("Mind"),    &["Agency"])),
        ("significator", "Identity-Statement")=> Some((Some("Significator"), Some("Spirit"),  &["Eros"])),
        ("significator", "Pillar")            => Some((Some("Significator"), Some("Body"),    &["Agape"])),
        ("significator", "Strategic-Ideal")   => Some((Some("Significator"), Some("Mind"),    &["Eros"])),

        // ── World (GreatWay) ──
        ("greatway", "Annual Goal")   => Some((Some("Great Way"), Some("Mind"),    &["Agency"])),
        ("greatway", "Quarterly Goal")=> Some((Some("Great Way"), Some("Mind"),    &["Agency"])),
        ("greatway", "Goal")          => Some((Some("Great Way"), Some("Mind"),    &["Agency"])),
        ("greatway", "Project")       => Some((Some("Great Way"), Some("Mind"),    &["Agency"])),
        ("greatway", "Task")          => Some((Some("Great Way"), Some("Body"),    &["Agency"])),
        ("greatway", "System")        => Some((Some("Great Way"), Some("Mind"),    &["Agape"])),
        ("greatway", "Resource")      => Some((Some("Great Way"), Some("Body"),    &[])),
        ("greatway", "Sprint")        => Some((Some("Great Way"), Some("Body"),    &["Eros"])),
        ("greatway", "Milestone")     => Some((Some("Great Way"), Some("Mind"),    &["Eros"])),
        ("greatway", "Budget")        => Some((Some("Great Way"), Some("Mind"),    &["Agency"])),
        ("greatway", "Campaign")      => Some((Some("Great Way"), Some("Body"),    &["Communion"])),
        ("greatway", "Content")       => Some((Some("Great Way"), Some("Spirit"),  &["Communion"])),
        ("greatway", "Person")        => Some((Some("Great Way"), Some("Spirit"),  &["Communion"])),
        ("greatway", "Group")         => Some((Some("Great Way"), Some("Spirit"),  &["Communion"])),
        ("greatway", "Community")     => Some((Some("Great Way"), Some("Spirit"),  &["Communion"])),
        ("greatway", "Organization")  => Some((Some("Great Way"), Some("Mind"),    &["Communion"])),
        ("greatway", "Network")       => Some((Some("Great Way"), Some("Mind"),    &["Communion"])),
        ("greatway", "Movement")      => Some((Some("Great Way"), Some("Spirit"),  &["Eros"])),
        ("greatway", "Place")         => Some((Some("Great Way"), Some("Body"),    &[])),

        _ => None,
    }
}

// ── Property extraction helpers ──────────────────────────────────────────────

fn get_select_name(prop: &PropertyValue) -> Option<String> {
    match prop {
        PropertyValue::Select { select, .. } => select.as_ref().map(|s| s.name.clone()),
        PropertyValue::Status { status, .. } => status.as_ref().map(|s| s.name.clone()),
        _ => None,
    }
}

fn get_multi_select_names(prop: &PropertyValue) -> Vec<String> {
    match prop {
        PropertyValue::MultiSelect { multi_select, .. } => multi_select.iter().map(|s| s.name.clone()).collect(),
        _ => Vec::new(),
    }
}

fn get_entry_type_from_prop(prop: &PropertyValue) -> Option<String> {
    match prop {
        PropertyValue::MultiSelect { multi_select, .. } => multi_select.first().map(|s| s.name.clone()),
        PropertyValue::Select { select, .. } => select.as_ref().map(|s| s.name.clone()),
        PropertyValue::Status { status, .. } => status.as_ref().map(|s| s.name.clone()),
        _ => None,
    }
}

fn extract_title(props: &std::collections::HashMap<String, PropertyValue>) -> String {
    match props.get("Name") {
        Some(PropertyValue::Title { title, .. }) if !title.is_empty() =>
            title.first()
                .and_then(|t| t.plain_text.clone())
                .unwrap_or_else(|| "(untitled)".to_string()),
        _ => "(untitled)".to_string(),
    }
}

// ─── Main execute ────────────────────────────────────────────────────────────

pub async fn execute(
    params: &AutoEnrichParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let limit = params.limit.unwrap_or(50).min(500) as u64;
    let apply = params.apply && params.mode == "tag"; // link mode always dry-run

    let db_keys: Vec<String> = if let Some(ref db) = params.database {
        if !config.databases.contains_key(db) {
            return Err(format!("Unknown database: {}", db));
        }
        vec![db.clone()]
    } else {
        config.all_database_keys()
    };

    let mode_label = if apply { "APPLY" } else { "DRY-RUN" };
    let mut report = String::new();
    report.push_str(&format!("LifeOS auto_enrich — mode={} {} — limit={}/DB\n\n",
        params.mode, mode_label, limit));

    let mut total_processed = 0u64;
    let mut total_changed = 0u64;
    let mut total_skipped_no_rule = 0u64;
    let mut total_skipped_already_set = 0u64;
    let mut errors: Vec<String> = Vec::new();

    for db_key in &db_keys {
        let db = match crate::config::resolve_db(config, db_key) {
            Some(db) => db,
            None => continue,
        };
        let ds_id = db.ds_id().to_string();
        let et_prop_name = db.entry_type_property.clone()
            .unwrap_or_else(|| "Entry Type".to_string());

        // Query up to `limit` pages
        let query_body = json!({ "page_size": limit.min(100) });
        let query_resp = notion.query_data_source(&ds_id, &query_body).await
            .map_err(|e| format!("Query {} failed: {}", db_key, e))?;
        let pages = query_resp.results;

        report.push_str(&format!("── {} ({} entries scanned) ──\n",
            db.name, pages.len()));

        for page in &pages {
            total_processed += 1;
            let page_id = page.id.clone();

            // Resolve entry-type
            let et_prop = match page.properties.get(&et_prop_name) {
                Some(p) => p,
                None => { total_skipped_no_rule += 1; continue; }
            };
            let entry_type = match get_entry_type_from_prop(et_prop) {
                Some(t) => t,
                None => { total_skipped_no_rule += 1; continue; }
            };

            // Look up the rule
            let rule = match rule_for(db_key, &entry_type) {
                Some(r) => r,
                None => { total_skipped_no_rule += 1; continue; }
            };
            let (default_role, default_complex, default_drives) = rule;

            if params.mode == "tag" {
                // ── TAG MODE: set Archetype Role / Complex / Drive Activation ──
                let ar_prop = page.properties.get("Archetype Role");
                let cx_prop = page.properties.get("Complex");
                let dr_prop = page.properties.get("Drive Activation");

                let role_missing = ar_prop.and_then(get_select_name).is_none();
                let complex_missing = cx_prop.and_then(get_select_name).is_none();
                let drives_missing = dr_prop.map(get_multi_select_names)
                    .map(|v| v.is_empty()).unwrap_or(true);

                if role_missing || complex_missing || drives_missing {
                    let mut update_props = serde_json::Map::new();
                    if role_missing {
                        if let Some(role) = default_role {
                            update_props.insert("Archetype Role".to_string(),
                                json!({"select": {"name": role}}));
                        }
                    }
                    if complex_missing {
                        if let Some(complex) = default_complex {
                            update_props.insert("Complex".to_string(),
                                json!({"select": {"name": complex}}));
                        }
                    }
                    if drives_missing && !default_drives.is_empty() {
                        let ms_arr: Vec<Value> = default_drives.iter()
                            .map(|d| json!({"name": d})).collect();
                        update_props.insert("Drive Activation".to_string(),
                            json!({"multi_select": ms_arr}));
                    }

                    if update_props.is_empty() {
                        total_skipped_already_set += 1;
                        continue;
                    }

                    let title = extract_title(&page.properties);
                    report.push_str(&format!("  • [{}] {} ({}) ← {}\n",
                        db_key, title, entry_type,
                        update_props.keys().cloned().collect::<Vec<_>>().join(", ")));

                    if apply {
                        match notion.update_page_properties(&page_id, &Value::Object(update_props.into())).await {
                            Ok(_) => total_changed += 1,
                            Err(e) => errors.push(format!("{}.{} ({}): {}", db_key, title, page_id, e)),
                        }
                    } else {
                        total_changed += 1;
                    }
                } else {
                    total_skipped_already_set += 1;
                }
            } else if params.mode == "link" {
                // ── LINK MODE: dry-run only — report suggestions ──
                let title = extract_title(&page.properties);
                let suggestion = match (db_key.as_str(), entry_type.as_str()) {
                    ("potentiator", "Activity") => Some(("For", "World", "active Project")),
                    ("potentiator", "Diet") => Some(("Reveals", "State", "active Diet-kind Pattern")),
                    ("potentiator", "Subjective") => Some(("For", "Identity", "active Identity-Statement")),
                    ("potentiator", "Relational") => Some(("People", "World", "Person entry from title")),
                    ("nexus", "Note") => Some(("Updates", "State", "Pattern this note refines")),
                    ("nexus", "Decision") => Some(("Emits Choice To", "World", "Project this decision affects")),
                    _ => None,
                };
                if let Some((prop, target_db, hint)) = suggestion {
                    report.push_str(&format!("  • [{}] {} ({}) → {}.{}  ({})\n",
                        db_key, title, entry_type, target_db, prop, hint));
                    total_changed += 1;
                } else {
                    total_skipped_no_rule += 1;
                }
            }
        }
        report.push('\n');
    }

    report.push_str("── Summary ──\n");
    report.push_str(&format!("  Processed:             {}\n", total_processed));
    report.push_str(&format!("  {} changes:     {}\n", if apply { "Applied" } else { "Would-apply" }, total_changed));
    report.push_str(&format!("  Skipped (no rule):     {}\n", total_skipped_no_rule));
    report.push_str(&format!("  Skipped (already set): {}\n", total_skipped_already_set));
    if !errors.is_empty() {
        report.push_str(&format!("\n  ⚠ {} errors:\n", errors.len()));
        for e in errors.iter().take(10) {
            report.push_str(&format!("    - {}\n", e));
        }
    }
    if !apply && total_changed > 0 && params.mode == "tag" {
        report.push_str("\n  Re-run with apply=true to write these changes to Notion.\n");
    }
    if params.mode == "link" {
        report.push_str("\n  Link mode is dry-run only in v0.10. Auto-link application deferred to v0.11.\n");
    }
    Ok(report)
}
