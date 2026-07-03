//! YAML Schema Engine — v0.9.0 (lean)
//!
//! Loads the 3-tier YAML schema hierarchy and validates Notion entries.
//!
//! Schema hierarchy:
//!   1. `universal/holon_coordinate.yaml`     (every entry)
//!   2. `per_db/<db>.yaml`                    (every entry in a DB)
//!   3. `per_entry_type/<db>__<entry>.yaml`   (one entry-type in one DB)
//!
//! Validation rules are hardcoded Rust match statements (not a DSL).
//! The 3 rules are simple enough that a mini-interpreter was YAGNI.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_yaml::Value as YamlValue;

use crate::notion::types::{NotionPage, PropertyValue};

pub const DB_KEYS: &[&str] = &["matrix", "potentiator", "nexus", "significator", "greatway"];

const ENTRY_TYPE_PROP: &[(&str, &str)] = &[
    ("matrix", "Entry Type"),
    ("potentiator", "Entry Type"),
    ("significator", "Entry Type"),
    ("greatway", "Item Type"),
    ("nexus", "Category"),
];

// ── Data structures ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PropertySchema {
    pub notion_type: String,
    pub required: bool,
    pub options: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub rule_id: String,
    pub applies_to_db: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SchemaLayer {
    pub schema_type: SchemaType,
    pub applies_to_db: Option<String>,
    pub applies_to_entry_type: Option<String>,
    pub properties: HashMap<String, PropertySchema>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    Universal,
    PerDb,
    PerEntryType,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub db: String,
    pub entry_type: Option<String>,
    pub page_id: Option<String>,
    pub page_title: Option<String>,
    pub rule_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
    pub entry_count: usize,
    pub validated_count: usize,
}

impl ValidationResult {
    pub fn merge(&mut self, other: ValidationResult) {
        self.valid = self.valid && other.valid;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.entry_count += other.entry_count;
        self.validated_count += other.validated_count;
    }

    pub fn summary(&self) -> String {
        let status = if self.valid { "✅ PASS" } else { "❌ FAIL" };
        format!(
            "Validation Result: {}\n  Entries scanned:    {}\n  Entries validated:  {}\n  Errors:             {}\n  Warnings:           {}",
            status, self.entry_count, self.validated_count, self.errors.len(), self.warnings.len()
        )
    }
}

// ── Schema registry ─────────────────────────────────────────────────────

pub struct YamlSchemaRegistry {
    pub universal: Option<SchemaLayer>,
    pub per_db: HashMap<String, SchemaLayer>,
    pub per_entry_type: HashMap<(String, String), SchemaLayer>,
    pub load_errors: Vec<String>,
    pub schemas_dir: PathBuf,
}

impl YamlSchemaRegistry {
    pub fn load(schemas_dir: &Path) -> Self {
        let mut registry = Self {
            universal: None,
            per_db: HashMap::new(),
            per_entry_type: HashMap::new(),
            load_errors: Vec::new(),
            schemas_dir: schemas_dir.to_path_buf(),
        };

        // Universal
        let uni_path = schemas_dir.join("universal").join("holon_coordinate.yaml");
        if uni_path.exists() {
            match Self::load_layer(&uni_path, SchemaType::Universal) {
                Ok(layer) => registry.universal = Some(layer),
                Err(e) => registry.load_errors.push(format!("Failed to load {}: {}", uni_path.display(), e)),
            }
        } else {
            registry.load_errors.push(format!("Missing universal schema: {}", uni_path.display()));
        }

        // Per-DB
        for db_key in DB_KEYS {
            let path = schemas_dir.join("per_db").join(format!("{}.yaml", db_key));
            if path.exists() {
                match Self::load_layer(&path, SchemaType::PerDb) {
                    Ok(layer) => {
                        if layer.applies_to_db.as_deref() == Some(db_key) {
                            registry.per_db.insert(db_key.to_string(), layer);
                        }
                    }
                    Err(e) => registry.load_errors.push(format!("Failed to load {}: {}", path.display(), e)),
                }
            }
        }

        // Per-entry-type
        let pet_dir = schemas_dir.join("per_entry_type");
        if pet_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&pet_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                        continue;
                    }
                    if let Ok(layer) = Self::load_layer(&path, SchemaType::PerEntryType) {
                        if let (Some(db), Some(et)) = (&layer.applies_to_db, &layer.applies_to_entry_type) {
                            registry.per_entry_type.insert((db.clone(), et.clone()), layer);
                        }
                    }
                }
            }
        }

        registry
    }

    pub fn discover_schemas_dir() -> Option<PathBuf> {
        if let Ok(env_path) = std::env::var("LIFEOS_SCHEMAS_DIR") {
            let p = PathBuf::from(env_path);
            if p.exists() {
                return Some(p);
            }
        }
        for candidate in &["./schemas", "../schemas", "../../schemas"] {
            let p = PathBuf::from(candidate);
            if p.exists() && p.join("universal").exists() {
                return Some(p);
            }
        }
        let mut cwd = std::env::current_dir().ok()?;
        for _ in 0..8 {
            let candidate = cwd.join("schemas");
            if candidate.join("universal").join("holon_coordinate.yaml").exists() {
                return Some(candidate);
            }
            if !cwd.pop() {
                break;
            }
        }
        None
    }

    fn load_layer(path: &Path, schema_type: SchemaType) -> Result<SchemaLayer, String> {
        let raw_text = std::fs::read_to_string(path).map_err(|e| format!("IO: {}", e))?;
        let raw: YamlValue = serde_yaml::from_str(&raw_text).map_err(|e| format!("YAML parse: {}", e))?;

        let applies_to_db = raw.get("applies_to_db").and_then(|v| v.as_str()).map(|s| s.to_string());
        let applies_to_entry_type = raw.get("applies_to_entry_type").and_then(|v| v.as_str()).map(|s| s.to_string());

        let mut properties: HashMap<String, PropertySchema> = HashMap::new();
        if let Some(props_map) = raw.get("properties").and_then(|v| v.as_mapping()) {
            for (key, val) in props_map {
                let name = key.as_str().unwrap_or("").to_string();
                if name.is_empty() {
                    continue;
                }
                properties.insert(name, PropertySchema {
                    notion_type: val.get("notion_type").and_then(|v| v.as_str()).unwrap_or("rich_text").to_string(),
                    required: val.get("required").and_then(|v| v.as_bool()).unwrap_or(false),
                    options: val.get("options")
                        .and_then(|v| v.as_sequence())
                        .map(|seq| seq.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default(),
                });
            }
        }

        let mut validation_rules: Vec<ValidationRule> = Vec::new();
        if let Some(rules_seq) = raw.get("validation_rules").and_then(|v| v.as_sequence()) {
            for rule_val in rules_seq {
                let rule_id = rule_val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if !rule_id.is_empty() {
                    validation_rules.push(ValidationRule {
                        rule_id,
                        applies_to_db: rule_val.get("applies_to_db").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    });
                }
            }
        }

        Ok(SchemaLayer {
            schema_type,
            applies_to_db,
            applies_to_entry_type,
            properties,
            validation_rules,
        })
    }

    pub fn layers_for(&self, db: &str, entry_type: Option<&str>) -> Vec<&SchemaLayer> {
        let mut layers: Vec<&SchemaLayer> = Vec::new();
        if let Some(uni) = &self.universal {
            layers.push(uni);
        }
        if let Some(per_db) = self.per_db.get(db) {
            layers.push(per_db);
        }
        if let Some(et) = entry_type {
            if let Some(per_et) = self.per_entry_type.get(&(db.to_string(), et.to_string())) {
                layers.push(per_et);
            }
        }
        layers
    }

    pub fn self_test(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.universal.is_none() {
            issues.push("universal/holon_coordinate.yaml not loaded".to_string());
            return issues;
        }
        for db in DB_KEYS {
            if !self.per_db.contains_key(*db) {
                issues.push(format!("per_db/{}.yaml not loaded", db));
            }
        }
        issues
    }
}

/// Public helper: count entry-types declared in per_db/<db>.yaml.
pub fn count_declared_entry_types(schemas_dir: &Path, db: &str) -> usize {
    let path = schemas_dir.join("per_db").join(format!("{}.yaml", db));
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return 0,
    };
    let raw: YamlValue = match serde_yaml::from_str(&text) {
        Ok(v) => v,
        Err(_) => return 0,
    };
    raw.get("entry_types")
        .and_then(|v| v.as_sequence())
        .map(|s| s.len())
        .unwrap_or(0)
}

// ── Property extraction ─────────────────────────────────────────────────

fn extract_prop_string(value: &PropertyValue) -> Option<String> {
    match value {
        PropertyValue::Title { title, .. } => {
            let s: String = title.iter().filter_map(|rt| rt.plain_text.as_deref()).collect();
            if s.is_empty() { None } else { Some(s) }
        }
        PropertyValue::RichText { rich_text, .. } => {
            let s: String = rich_text.iter().filter_map(|rt| rt.plain_text.as_deref()).collect();
            if s.is_empty() { None } else { Some(s) }
        }
        PropertyValue::Select { select, .. } => select.as_ref().map(|o| o.name.clone()),
        PropertyValue::MultiSelect { multi_select, .. } => {
            let names: Vec<String> = multi_select.iter().map(|o| o.name.clone()).collect();
            if names.is_empty() { None } else { Some(names.join(", ")) }
        }
        PropertyValue::Status { status, .. } => status.as_ref().map(|o| o.name.clone()),
        PropertyValue::Date { date, .. } => date.as_ref().map(|d| d.start.clone()),
        PropertyValue::Number { number, .. } => number.as_ref().map(|n| n.to_string()),
        PropertyValue::Checkbox { checkbox, .. } => Some(checkbox.to_string()),
        PropertyValue::Url { url, .. } => url.clone(),
        PropertyValue::Email { email, .. } => email.clone(),
        PropertyValue::PhoneNumber { phone_number, .. } => phone_number.clone(),
        PropertyValue::Formula { formula, .. } => {
            formula.string.clone()
                .or_else(|| formula.number.as_ref().map(|n| n.to_string()))
                .or_else(|| formula.boolean.map(|b| b.to_string()))
        }
        PropertyValue::Relation { relation, .. } => {
            if relation.is_empty() { None } else { Some(relation.len().to_string()) }
        }
        _ => None,
    }
}

pub fn extract_entry_props(page: &NotionPage, db_key: &str) -> (Option<String>, HashMap<String, String>) {
    let et_prop_name = ENTRY_TYPE_PROP.iter()
        .find(|(k, _)| *k == db_key)
        .map(|(_, v)| *v)
        .unwrap_or("Entry Type");

    let mut entry_type: Option<String> = None;
    let mut flat: HashMap<String, String> = HashMap::new();

    for (name, value) in &page.properties {
        if let Some(str_val) = extract_prop_string(value) {
            if name == et_prop_name {
                entry_type = Some(str_val.clone());
                flat.insert("entry_type".to_string(), str_val.clone());
            }
            flat.insert(name.clone(), str_val);
        }
    }

    // Title fallback
    if let Some(PropertyValue::Title { title, .. }) = page.properties.get("Name") {
        let t: String = title.iter().filter_map(|rt| rt.plain_text.as_deref()).collect();
        if !t.is_empty() {
            flat.insert("title".to_string(), t);
        }
    }

    // Normalize Nexus Kind
    if db_key == "nexus" {
        if let Some(k) = flat.get("Kind").cloned() {
            flat.insert("kind".to_string(), k);
        }
    }

    (entry_type, flat)
}

// ── Validation engine ───────────────────────────────────────────────────

pub fn validate_entry(
    db_key: &str,
    page: &NotionPage,
    registry: &YamlSchemaRegistry,
) -> (Vec<ValidationError>, Vec<ValidationError>) {
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationError> = Vec::new();

    let (entry_type, flat) = extract_entry_props(page, db_key);
    let page_id = page.id.clone();
    let page_title = flat.get("title").cloned();

    let layers = registry.layers_for(db_key, entry_type.as_deref());
    if layers.is_empty() {
        warnings.push(ValidationError {
            db: db_key.to_string(), entry_type: entry_type.clone(),
            page_id: Some(page_id), page_title: page_title.clone(),
            rule_id: "no-schema".to_string(),
            message: "No applicable schema layers found.".to_string(),
        });
        return (errors, warnings);
    }

    // 1. Per-property required + options checks
    let mut merged_props: HashMap<String, &PropertySchema> = HashMap::new();
    for layer in &layers {
        for (pname, ps) in &layer.properties {
            merged_props.insert(pname.clone(), ps);
        }
    }

    for (pname, ps) in &merged_props {
        let value = flat.get(pname).cloned();
        if ps.required && value.is_none() {
            errors.push(ValidationError {
                db: db_key.to_string(), entry_type: entry_type.clone(),
                page_id: Some(page_id.clone()), page_title: page_title.clone(),
                rule_id: "required-missing".to_string(),
                message: format!("Required property '{}' is missing or empty.", pname),
            });
            continue;
        }
        let value = match value { Some(v) => v, None => continue };
        if value.is_empty() { continue; }
        if !ps.options.is_empty() && (ps.notion_type == "select" || ps.notion_type == "multi_select" || ps.notion_type == "status") {
            let vals: Vec<&str> = value.split(", ").collect();
            let bad: Vec<&str> = vals.iter().filter(|v| !ps.options.contains(&v.to_string())).copied().collect();
            if !bad.is_empty() {
                errors.push(ValidationError {
                    db: db_key.to_string(), entry_type: entry_type.clone(),
                    page_id: Some(page_id.clone()), page_title: page_title.clone(),
                    rule_id: "invalid-option".to_string(),
                    message: format!("Property '{}' has value(s) {:?} not in allowed options: {:?}", pname, bad, ps.options),
                });
            }
        }
    }

    // 2. Hardcoded validation rules (replaces the 500-line DSL evaluator)
    for layer in &layers {
        for rule in &layer.validation_rules {
            if let Some(rule_db) = &rule.applies_to_db {
                if rule_db != db_key { continue; }
            }
            if let Err(msg) = eval_hardcoded_rule(&rule.rule_id, &flat, db_key) {
                errors.push(ValidationError {
                    db: db_key.to_string(), entry_type: entry_type.clone(),
                    page_id: Some(page_id.clone()), page_title: page_title.clone(),
                    rule_id: rule.rule_id.clone(),
                    message: msg,
                });
            }
        }
    }

    (errors, warnings)
}

/// Hardcoded validation rules — replaces the 500-line mini Python DSL interpreter.
/// Each rule is a simple match arm that checks the entry's flat property map.
fn eval_hardcoded_rule(rule_id: &str, entry: &HashMap<String, String>, _db_key: &str) -> Result<(), String> {
    match rule_id {
        "nexus_kind_consistency" => {
            let kind = entry.get("kind").or_else(|| entry.get("Kind")).map(|s| s.as_str()).unwrap_or("");
            // Check forbidden relations based on Kind
            let forbidden = match kind {
                "Catalyst" => vec!["Tension", "Counter-Tension"],
                "Experience" => vec!["Updates", "Tension", "Counter-Tension"],
                "Transformation" => vec!["Updates", "Sourced From"],
                "Choice" => vec!["Updates", "Sourced From", "Tension"],
                _ => return Ok(()), // Unknown kind — don't block
            };
            for prop in &forbidden {
                if let Some(val) = entry.get(*prop) {
                    if !val.is_empty() && val != "0" {
                        return Err(format!(
                            "Kind '{}' entries cannot have '{}' relations populated (ontological constraint)",
                            kind, prop
                        ));
                    }
                }
            }
            Ok(())
        }
        "stage_type_independence" => {
            let stage = entry.get("stage_code").or_else(|| entry.get("Stage Code"));
            let htype = entry.get("holon_type").or_else(|| entry.get("Holon Type"));
            let stage_empty = stage.is_none() || stage.map(|s| s.is_empty()).unwrap_or(true);
            let htype_empty = htype.is_none() || htype.map(|s| s.is_empty()).unwrap_or(true);
            if stage_empty != htype_empty {
                return Err("stage_code and holon_type must both be set or both be empty".to_string());
            }
            Ok(())
        }
        "complex_archetype_consistency" => {
            let complex = entry.get("complex").or_else(|| entry.get("Complex")).map(|s| s.as_str()).unwrap_or("");
            let role = entry.get("archetype_role").or_else(|| entry.get("Archetype Role")).map(|s| s.as_str()).unwrap_or("");
            if complex == "None" {
                if role != "Choice" {
                    return Err(format!("complex=None is only valid with archetype_role=Choice, got '{}'", role));
                }
            } else if !complex.is_empty() && !role.is_empty() {
                let valid_pairs = [
                    ("Matrix","Mind"),("Potentiator","Mind"),("Catalyst","Mind"),
                    ("Experience","Mind"),("Significator","Mind"),
                    ("Transformation","Mind"),("Great Way","Mind"),
                    ("Matrix","Body"),("Potentiator","Body"),("Catalyst","Body"),
                    ("Experience","Body"),("Significator","Body"),
                    ("Transformation","Body"),("Great Way","Body"),
                    ("Matrix","Spirit"),("Potentiator","Spirit"),("Catalyst","Spirit"),
                    ("Experience","Spirit"),("Significator","Spirit"),
                    ("Transformation","Spirit"),("Great Way","Spirit"),
                ];
                if !valid_pairs.contains(&(role, complex)) {
                    return Err(format!("Invalid (archetype_role='{}', complex='{}') pair — must be one of the 22 named archetypes", role, complex));
                }
            }
            Ok(())
        }
        // Per-entry-type rules (Kind constraints for Nexus entry-types)
        "note_kind_constraint" | "insight_kind_constraint" | "opportunity_kind_constraint" | "risk_kind_constraint" => {
            let kind = entry.get("kind").or_else(|| entry.get("Kind")).map(|s| s.as_str()).unwrap_or("");
            if !kind.is_empty() && kind != "Catalyst" {
                return Err(format!("Kind must be 'Catalyst' for this entry-type, got '{}'", kind));
            }
            Ok(())
        }
        "reflection_kind_constraint" | "integration_kind_constraint" | "knowledge_category_kind_constraint" | "knowledge_atom_kind_constraint" => {
            let kind = entry.get("kind").or_else(|| entry.get("Kind")).map(|s| s.as_str()).unwrap_or("");
            if !kind.is_empty() && kind != "Experience" {
                return Err(format!("Kind must be 'Experience' for this entry-type, got '{}'", kind));
            }
            Ok(())
        }
        "pattern_kind_constraint" | "crisis_kind_constraint" | "transformation_event_kind_constraint" => {
            let kind = entry.get("kind").or_else(|| entry.get("Kind")).map(|s| s.as_str()).unwrap_or("");
            if !kind.is_empty() && kind != "Transformation" {
                return Err(format!("Kind must be 'Transformation' for this entry-type, got '{}'", kind));
            }
            Ok(())
        }
        "directive_kind_constraint" | "decision_kind_constraint" => {
            let kind = entry.get("kind").or_else(|| entry.get("Kind")).map(|s| s.as_str()).unwrap_or("");
            if !kind.is_empty() && kind != "Choice" {
                return Err(format!("Kind must be 'Choice' for this entry-type, got '{}'", kind));
            }
            Ok(())
        }
        "diet_must_be_catalyst_role" | "financial_must_be_catalyst_role" | "observation_must_be_catalyst_role" => {
            let role = entry.get("archetype_role").or_else(|| entry.get("Archetype Role")).map(|s| s.as_str()).unwrap_or("");
            if !role.is_empty() && role != "Catalyst" {
                return Err(format!("archetype_role must be 'Catalyst' for this entry-type, got '{}'", role));
            }
            Ok(())
        }
        // GreatWay external-holon quadrant constraints
        "person_quadrant_required" => {
            let q = entry.get("quadrant").or_else(|| entry.get("Quadrant")).map(|s| s.as_str()).unwrap_or("");
            if !q.is_empty() && q != "UL" && q != "UR" {
                return Err(format!("Person entries must have quadrant UL or UR, got '{}'", q));
            }
            Ok(())
        }
        "group_quadrant_required" | "community_quadrant_required" | "movement_quadrant_required" => {
            let q = entry.get("quadrant").or_else(|| entry.get("Quadrant")).map(|s| s.as_str()).unwrap_or("");
            if !q.is_empty() && q != "LL" {
                return Err(format!("This entry-type must have quadrant=LL, got '{}'", q));
            }
            Ok(())
        }
        "organization_quadrant_required" | "network_quadrant_required" | "place_quadrant_required" => {
            let q = entry.get("quadrant").or_else(|| entry.get("Quadrant")).map(|s| s.as_str()).unwrap_or("");
            if !q.is_empty() && q != "LR" {
                return Err(format!("This entry-type must have quadrant=LR, got '{}'", q));
            }
            Ok(())
        }
        "person_archetype_role" => {
            let role = entry.get("archetype_role").or_else(|| entry.get("Archetype Role")).map(|s| s.as_str()).unwrap_or("");
            if !role.is_empty() && role != "Great Way" {
                return Err(format!("Person entries must have archetype_role='Great Way', got '{}'", role));
            }
            Ok(())
        }
        // Significator principle sub-type constraints
        "purpose_principle_sub_type" => {
            let st = entry.get("principle_sub_type").or_else(|| entry.get("Principle Sub Type")).map(|s| s.as_str()).unwrap_or("");
            if !st.is_empty() && st != "Purpose" {
                return Err(format!("principle_sub_type must be 'Purpose', got '{}'", st));
            }
            Ok(())
        }
        "value_principle_sub_type" => {
            let st = entry.get("principle_sub_type").or_else(|| entry.get("Principle Sub Type")).map(|s| s.as_str()).unwrap_or("");
            if !st.is_empty() && st != "Value" {
                return Err(format!("principle_sub_type must be 'Value', got '{}'", st));
            }
            Ok(())
        }
        "principle_principle_sub_type" => {
            let st = entry.get("principle_sub_type").or_else(|| entry.get("Principle Sub Type")).map(|s| s.as_str()).unwrap_or("");
            if !st.is_empty() && st != "Principle" {
                return Err(format!("principle_sub_type must be 'Principle', got '{}'", st));
            }
            Ok(())
        }
        // Matrix threshold
        "threshold_must_have_trigger" => {
            let tv = entry.get("trigger_threshold_value");
            let tc = entry.get("trigger_catalyst_class");
            if (tv.is_none() || tv.map(|s| s.is_empty()).unwrap_or(true))
                && (tc.is_none() || tc.map(|s| s.is_empty()).unwrap_or(true)) {
                return Err("Threshold entries must specify trigger_threshold_value OR trigger_catalyst_class".to_string());
            }
            Ok(())
        }
        _ => Ok(()), // Unknown rule — don't block (forward-compatible)
    }
}
