//! YAML Schema Engine — v0.9.0
//!
//! Loads the 3-tier YAML schema hierarchy and validates Notion entries
//! against the applicable schema layers.
//!
//! Schema hierarchy (most-general → most-specific):
//!   1. `schemas/universal/holon_coordinate.yaml`     (every entry)
//!   2. `schemas/per_db/<db>.yaml`                    (every entry in a given DB)
//!   3. `schemas/per_entry_type/<db>__<entry>.yaml`   (entries of one type in one DB)
//!
//! A property is required for an entry IFF it is `required: true` at ANY
//! layer that applies to that entry (universal → per_db → per_entry_type).
//!
//! ## Usage
//!
//! The `YamlSchemaRegistry` is constructed from a schemas directory and
//! caches all loaded schemas. Use `validate_entry` to validate a single
//! Notion page against its applicable schema layers.
//!
//! ## Validation rules
//!
//! Cross-property validation rules are expressed in a small Python-like DSL
//! (see `universal/holon_coordinate.yaml` for examples). The DSL is evaluated
//! by `eval_rule`, which supports:
//!   - `if entry.<prop> == "<value>": ...`
//!   - `assert entry.<prop> in {...}`
//!   - `assert (entry.<a> is None) == (entry.<b> is None)`
//!   - `assert_no_relations(entry, [...])`
//!
//! ## Reference implementation
//!
//! The Python reference validator at
//! `scripts/upgrade_v0.9.0/yaml_schema_validator.py` mirrors these semantics.
//! The two implementations should produce identical results for any input.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_yaml::Value as YamlValue;

use crate::notion::types::{NotionPage, PropertyValue};

// ── Constants ───────────────────────────────────────────────────────────

pub const DB_KEYS: &[&str] = &["matrix", "potentiator", "nexus", "significator", "greatway"];

const VALID_NOTION_TYPES: &[&str] = &[
    "title", "rich_text", "select", "multi_select", "status", "date",
    "number", "checkbox", "people", "relation", "url", "email",
    "phone_number", "files", "formula", "rollup",
    "created_time", "last_edited_time", "created_by", "last_edited_by",
    "unique_id", "button",
];

// DB → entry-type property name (per lifeos.config.default.json)
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
    pub name: String,
    pub notion_type: String,
    pub required: bool,
    pub description: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub rule_id: String,
    pub description: String,
    pub rule_expr: String,
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
    pub layer: String,
    pub property_name: Option<String>,
    pub rule_id: String,
    pub severity: String,
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
    /// Load all 3-tier schemas from the given directory.
    /// The directory should contain:
    ///   - `universal/holon_coordinate.yaml`
    ///   - `per_db/{matrix,potentiator,nexus,significator,greatway}.yaml`
    ///   - `per_entry_type/*.yaml` (any number)
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
                        } else {
                            registry.load_errors.push(format!(
                                "per_db/{}.yaml applies_to_db mismatch: expected {}, got {:?}",
                                db_key, db_key, layer.applies_to_db
                            ));
                        }
                    }
                    Err(e) => registry.load_errors.push(format!("Failed to load {}: {}", path.display(), e)),
                }
            } else {
                registry.load_errors.push(format!("Missing per_db schema: {}", path.display()));
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
                    match Self::load_layer(&path, SchemaType::PerEntryType) {
                        Ok(layer) => {
                            if let (Some(db), Some(et)) = (&layer.applies_to_db, &layer.applies_to_entry_type) {
                                registry.per_entry_type.insert((db.clone(), et.clone()), layer);
                            }
                        }
                        Err(e) => registry.load_errors.push(format!("Failed to load {}: {}", path.display(), e)),
                    }
                }
            }
        }

        registry
    }

    /// Try to locate the schemas directory.
    /// Search order:
    ///   1. `$LIFEOS_SCHEMAS_DIR` env var
    ///   2. `./schemas/` (current working dir)
    ///   3. `../schemas/` (parent dir — for when running from lifeos/ or lifeos-core/)
    ///   4. The `schemas/` directory at the repo root (auto-detected by walking up)
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
        // Walk up from CWD looking for a `schemas/universal/holon_coordinate.yaml` file
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
        let raw_text = std::fs::read_to_string(path)
            .map_err(|e| format!("IO: {}", e))?;
        let raw: YamlValue = serde_yaml::from_str(&raw_text)
            .map_err(|e| format!("YAML parse: {}", e))?;

        let applies_to_db = raw.get("applies_to_db")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let applies_to_entry_type = raw.get("applies_to_entry_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Properties
        let mut properties: HashMap<String, PropertySchema> = HashMap::new();
        if let Some(props_map) = raw.get("properties").and_then(|v| v.as_mapping()) {
            for (key, val) in props_map {
                let name = key.as_str().unwrap_or("").to_string();
                if name.is_empty() { continue; }
                let notion_type = val.get("notion_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("rich_text")
                    .to_string();
                let required = val.get("required")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let description = val.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let options = val.get("options")
                    .and_then(|v| v.as_sequence())
                    .map(|seq| seq.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                properties.insert(name.clone(), PropertySchema {
                    name, notion_type, required, description, options,
                });
            }
        }

        // Validation rules
        let mut validation_rules: Vec<ValidationRule> = Vec::new();
        if let Some(rules_seq) = raw.get("validation_rules").and_then(|v| v.as_sequence()) {
            for rule_val in rules_seq {
                let rule_id = rule_val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let description = rule_val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let rule_expr = rule_val.get("rule").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let applies_to_db = rule_val.get("applies_to_db").and_then(|v| v.as_str()).map(|s| s.to_string());
                if !rule_id.is_empty() {
                    validation_rules.push(ValidationRule { rule_id, description, rule_expr, applies_to_db });
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

    /// Return the 3 applicable schema layers for a (db, entry_type) pair.
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

    /// Self-test: validate the schema files themselves.
    /// Returns a list of issues (empty = OK).
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
        // Check property notion_types
        let all_layers: Vec<&SchemaLayer> = self.universal.iter()
            .chain(self.per_db.values())
            .chain(self.per_entry_type.values())
            .collect();
        for layer in all_layers {
            for (pname, ps) in &layer.properties {
                if !VALID_NOTION_TYPES.contains(&ps.notion_type.as_str()) {
                    issues.push(format!(
                        "{:?} {}/{}: property '{}' has invalid notion_type '{}'",
                        layer.schema_type, layer.applies_to_db.as_deref().unwrap_or("?"),
                        layer.applies_to_entry_type.as_deref().unwrap_or("?"),
                        pname, ps.notion_type
                    ));
                }
            }
        }
        // Cross-check: every entry-type declared in per_db must have a per_entry_type file
        for (db, _layer) in &self.per_db {
            if let Some(et_seq) = layer_raw_entry_types(&self.schemas_dir, db) {
                for et in et_seq {
                    if !self.per_entry_type.contains_key(&(db.clone(), et.clone())) {
                        issues.push(format!(
                            "per_db/{}.yaml declares entry-type '{}' but no per_entry_type file exists",
                            db, et
                        ));
                    }
                }
            }
        }
        issues
    }
}

fn layer_raw_entry_types(schemas_dir: &Path, db: &str) -> Option<Vec<String>> {
    let path = schemas_dir.join("per_db").join(format!("{}.yaml", db));
    let text = std::fs::read_to_string(&path).ok()?;
    let raw: YamlValue = serde_yaml::from_str(&text).ok()?;
    let seq = raw.get("entry_types")?.as_sequence()?;
    Some(seq.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
}

/// Public helper: count the entry-types declared in `per_db/<db>.yaml`.
/// Used by the CLI self-test path.
pub fn count_declared_entry_types(schemas_dir: &Path, db: &str) -> usize {
    layer_raw_entry_types(schemas_dir, db).map(|v| v.len()).unwrap_or(0)
}

// ── Property extraction (Notion → flat HashMap for validation) ──────────

/// Extract a flat HashMap of property values from a Notion page.
/// Returns (entry_type, flat_props) where flat_props maps property name → string value.
pub fn extract_entry_props(page: &NotionPage, db_key: &str) -> (Option<String>, HashMap<String, String>) {
    let et_prop_name = ENTRY_TYPE_PROP.iter()
        .find(|(k, _)| *k == db_key)
        .map(|(_, v)| *v)
        .unwrap_or("Entry Type");

    let mut entry_type: Option<String> = None;
    let mut flat: HashMap<String, String> = HashMap::new();

    for (name, value) in &page.properties {
        let str_val = extract_prop_string(value);
        if name == et_prop_name {
            entry_type = str_val.clone();
            if let Some(ref et) = str_val {
                flat.insert("entry_type".to_string(), et.clone());
            }
        }
        if let Some(ref s) = str_val {
            flat.insert(name.clone(), s.clone());
        }
    }

    // Title fallback
    let title = page.properties.iter()
        .find(|(_, v)| matches!(v, PropertyValue::Title { title, .. } if !title.is_empty()))
        .and_then(|(k, _)| Some(k.clone()))
        .or_else(|| Some("Name".to_string()));
    if let Some(title_key) = title {
        if let Some(PropertyValue::Title { title, .. }) = page.properties.get(&title_key) {
            let t: String = title.iter().filter_map(|rt| rt.plain_text.as_deref()).collect();
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

// ── Validation engine ───────────────────────────────────────────────────

/// Validate a single Notion entry against its applicable schema layers.
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
            layer: "universal".to_string(), property_name: None,
            rule_id: "no-schema".to_string(), severity: "warning".to_string(),
            message: "No applicable schema layers found.".to_string(),
        });
        return (errors, warnings);
    }

    // 1. Per-property type + required + options checks
    let mut merged_props: HashMap<String, &PropertySchema> = HashMap::new();
    for layer in &layers {
        for (pname, ps) in &layer.properties {
            merged_props.insert(pname.clone(), ps);
            let snake = snake_case(pname);
            if snake != *pname {
                merged_props.entry(snake).or_insert(ps);
            }
        }
    }

    for (pname, ps) in &merged_props {
        let value = flat.get(p_name_lookup(pname, &flat, ps)).cloned();
        if ps.required && value.is_none() {
            errors.push(ValidationError {
                db: db_key.to_string(), entry_type: entry_type.clone(),
                page_id: Some(page_id.clone()), page_title: page_title.clone(),
                layer: "any".to_string(), property_name: Some(pname.clone()),
                rule_id: "required-missing".to_string(), severity: "error".to_string(),
                message: format!("Required property '{}' is missing or empty.", pname),
            });
            continue;
        }
        let value = match value { Some(v) => v, None => continue };
        if value.is_empty() { continue; }
        // Options check
        if !ps.options.is_empty() && (ps.notion_type == "select" || ps.notion_type == "multi_select" || ps.notion_type == "status") {
            let vals: Vec<&str> = value.split(", ").collect();
            let bad: Vec<&str> = vals.iter().filter(|v| !ps.options.contains(&v.to_string())).copied().collect();
            if !bad.is_empty() {
                errors.push(ValidationError {
                    db: db_key.to_string(), entry_type: entry_type.clone(),
                    page_id: Some(page_id.clone()), page_title: page_title.clone(),
                    layer: "any".to_string(), property_name: Some(pname.clone()),
                    rule_id: "invalid-option".to_string(), severity: "error".to_string(),
                    message: format!("Property '{}' has value(s) {:?} not in allowed options: {:?}", pname, bad, ps.options),
                });
            }
        }
    }

    // 2. Cross-property validation rules
    for layer in &layers {
        for rule in &layer.validation_rules {
            if let Some(rule_db) = &rule.applies_to_db {
                if rule_db != db_key { continue; }
            }
            match eval_rule(&rule.rule_expr, &flat) {
                Ok(true) => {},
                Ok(false) => {
                    errors.push(ValidationError {
                        db: db_key.to_string(), entry_type: entry_type.clone(),
                        page_id: Some(page_id.clone()), page_title: page_title.clone(),
                        layer: "cross-property".to_string(), property_name: None,
                        rule_id: rule.rule_id.clone(), severity: "error".to_string(),
                        message: format!("Validation rule '{}' failed: {}", rule.rule_id, rule.description),
                    });
                }
                Err(e) => {
                    warnings.push(ValidationError {
                        db: db_key.to_string(), entry_type: entry_type.clone(),
                        page_id: Some(page_id.clone()), page_title: page_title.clone(),
                        layer: "cross-property".to_string(), property_name: None,
                        rule_id: rule.rule_id.clone(), severity: "warning".to_string(),
                        message: format!("Validation rule '{}' could not be evaluated: {}", rule.rule_id, e),
                    });
                }
            }
        }
    }

    (errors, warnings)
}

fn p_name_lookup<'a>(pname: &'a str, _flat: &'a HashMap<String, String>, _ps: &PropertySchema) -> &'a str {
    // Try the property name directly; the caller's flat.get() will return None
    // if the key isn't present, which is the desired behavior.
    pname
}

fn snake_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_upper = false;
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_upper {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
            prev_upper = true;
        } else if c == ' ' || c == '-' {
            out.push('_');
            prev_upper = false;
        } else {
            out.push(c);
            prev_upper = false;
        }
    }
    out
}

// ── Rule evaluator (small Python-like DSL) ──────────────────────────────

/// Evaluate a validation rule expression.
/// Returns Ok(true) if the rule passes, Ok(false) if it fails,
/// Err if the rule couldn't be evaluated.
pub fn eval_rule(rule_expr: &str, entry: &HashMap<String, String>) -> Result<bool, String> {
    let expr = rule_expr.trim();
    // We support a small subset of the Python-like DSL:
    //   if entry.<prop> == "<value>":
    //       assert_no_relations(entry, [...])
    //   if entry.<prop> is not None:
    //       assert entry.<prop> in {...}
    //   assert (entry.<a> is None) == (entry.<b> is None)
    //   assert entry.<prop> in {"v1", "v2"}
    //   assert entry.<prop> == "<value>"
    //
    // For now, we implement these by parsing line-by-line.

    if expr.starts_with("if ") {
        return eval_if_rule(expr, entry);
    }
    if expr.starts_with("assert ") {
        return Ok(eval_assert(&expr["assert ".len()..], entry));
    }
    Err(format!("Unsupported rule expression: {}", expr))
}

fn eval_if_rule(expr: &str, entry: &HashMap<String, String>) -> Result<bool, String> {
    // Parse: if <condition>:\n    <body>
    let mut lines = expr.lines();
    let if_line = lines.next().ok_or("Empty if rule")?;
    let condition = if_line.trim_start_matches("if ").trim_end_matches(':').trim();

    // Evaluate the condition
    let cond_holds = eval_condition(condition, entry)?;

    if cond_holds {
        // Evaluate the body (assert / assert_no_relations)
        for body_line in lines {
            let line = body_line.trim();
            if line.is_empty() { continue; }
            if line.starts_with("assert ") {
                if !eval_assert(&line["assert ".len()..], entry) {
                    return Ok(false);
                }
            } else if line.starts_with("assert_no_relations(") {
                if !eval_assert_no_relations(line, entry) {
                    return Ok(false);
                }
            } else if line.starts_with("elif ") || line.starts_with("else:") {
                // Skip elif/else branches when the first if holds
                break;
            }
        }
    }
    Ok(true)
}

fn eval_condition(cond: &str, entry: &HashMap<String, String>) -> Result<bool, String> {
    // entry.<prop> == "<value>"
    // entry.<prop> is not None
    // entry.<prop> is None
    if let Some(rest) = cond.strip_prefix("entry.") {
        if let Some((prop, op_val)) = rest.split_once(' ') {
            let prop = prop.trim();
            let value = entry.get(prop).cloned();
            let op_val = op_val.trim();
            if op_val == "is not None" {
                return Ok(value.is_some() && !value.as_ref().map(|s| s.is_empty()).unwrap_or(true));
            }
            if op_val == "is None" {
                return Ok(value.is_none() || value.as_ref().map(|s| s.is_empty()).unwrap_or(true));
            }
            if let Some((op, rhs)) = op_val.split_once(' ') {
                let op = op.trim();
                let rhs = rhs.trim().trim_matches('"');
                match op {
                    "==" => return Ok(value.as_deref() == Some(rhs)),
                    "!=" => return Ok(value.as_deref() != Some(rhs)),
                    _ => return Err(format!("Unsupported operator: {}", op)),
                }
            }
        }
    }
    Err(format!("Unsupported condition: {}", cond))
}

fn eval_assert(expr: &str, entry: &HashMap<String, String>) -> bool {
    // (entry.<a> is None) == (entry.<b> is None)
    // entry.<prop> in {"v1", "v2"}
    // entry.<prop> in [...]
    // entry.<prop> == "<value>"
    // entry.<prop> parses as valid YAML
    // "<key>" in entry.<prop>

    let expr = expr.trim();

    // Try: (entry.<a> is None) == (entry.<b> is None)
    if expr.starts_with('(') {
        // Find matching parens
        if let Some(close1) = find_matching_paren(expr, 0) {
            let left = &expr[1..close1];
            let rest = expr[close1+1..].trim();
            if let Some(rhs) = rest.strip_prefix("==") {
                let rhs = rhs.trim();
                if rhs.starts_with('(') {
                    if let Some(close2) = find_matching_paren(rhs, 0) {
                        let right = &rhs[1..close2];
                        let left_val = eval_simple_condition(left, entry);
                        let right_val = eval_simple_condition(right, entry);
                        return left_val == right_val;
                    }
                }
            }
        }
    }

    // Try: entry.<prop> in {...} or [...]
    if let Some(rest) = expr.strip_prefix("entry.") {
        if let Some((prop, op_rhs)) = rest.split_once(' ') {
            let prop = prop.trim();
            let op_rhs = op_rhs.trim();
            let value = entry.get(prop).cloned().unwrap_or_default();
            if let Some(rhs) = op_rhs.strip_prefix("in ") {
                let rhs = rhs.trim();
                let items: Vec<String> = if rhs.starts_with('{') {
                    // {"v1", "v2", ...}
                    let inner = rhs.trim_start_matches('{').trim_end_matches('}');
                    inner.split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else if rhs.starts_with('[') {
                    let inner = rhs.trim_start_matches('[').trim_end_matches(']');
                    inner.split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else {
                    return false;
                };
                return items.contains(&value);
            }
            if let Some(rhs) = op_rhs.strip_prefix("== ") {
                let rhs = rhs.trim().trim_matches('"');
                return value == rhs;
            }
            if op_rhs == "parses as valid YAML" {
                return serde_yaml::from_str::<YamlValue>(&value).is_ok();
            }
        }
    }

    if expr.starts_with('"') {
        if let Some(close) = expr[1..].find('"') {
            let key = &expr[1..1+close];
            let rest = expr[1+close+1..].trim();
            if let Some(rest) = rest.strip_prefix("in ") {
                if let Some(prop) = rest.strip_prefix("entry.") {
                    let prop = prop.trim();
                    let value = entry.get(prop).cloned().unwrap_or_default();
                    return value.contains(key);
                }
            }
        }
    }

    false
}

fn eval_simple_condition(cond: &str, entry: &HashMap<String, String>) -> bool {
    let cond = cond.trim();
    if let Some(rest) = cond.strip_prefix("entry.") {
        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.len() >= 3 && parts[1] == "is" {
            let prop = parts[0];
            let value = entry.get(prop).cloned().unwrap_or_default();
            if parts[2] == "None" {
                return value.is_empty();
            }
            if parts[2] == "not" && parts.len() >= 4 && parts[3] == "None" {
                return !value.is_empty();
            }
        }
    }
    false
}

fn find_matching_paren(s: &str, start: usize) -> Option<usize> {
    let mut depth = 0;
    let bytes = s.as_bytes();
    for i in start..bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn eval_assert_no_relations(line: &str, entry: &HashMap<String, String>) -> bool {
    // assert_no_relations(entry, ["prop1", "prop2"])
    // We treat "relation populated" as the property having a non-empty value
    // (for relation properties, the value is the count of relations as a string).
    if let Some(start) = line.find('[') {
        if let Some(end) = line.find(']') {
            let inner = &line[start+1..end];
            let props: Vec<&str> = inner.split(',')
                .map(|s| s.trim().trim_matches('"'))
                .collect();
            for prop in props {
                if let Some(val) = entry.get(prop) {
                    if !val.is_empty() && val != "0" {
                        return false;
                    }
                }
            }
            return true;
        }
    }
    true
}
