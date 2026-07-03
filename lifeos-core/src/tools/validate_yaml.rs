//! `validate-yaml` tool — validate Notion entries against the v0.9.0 YAML schemas.
//!
//! Loads the 3-tier YAML schema hierarchy (universal → per_db → per_entry_type)
//! and validates Notion entries against the applicable schema layers.
//!
//! ## CLI usage
//!   lifeos validate-yaml --self-test           # Validate the schema files themselves
//!   lifeos validate-yaml --db matrix            # Validate all entries in one DB
//!   lifeos validate-yaml --all                  # Validate all entries in all 5 DBs
//!   lifeos validate-yaml --page-id <id>         # Validate a single entry
//!
//! ## MCP tool name
//!   `validate_yaml`

use std::sync::Arc;

use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;
use crate::util::yaml_schemas::{YamlSchemaRegistry, ValidationResult, validate_entry};

#[derive(Debug, Deserialize)]
pub struct ValidateYamlParams {
    /// Optional: filter to a specific database (matrix/potentiator/nexus/significator/greatway)
    pub database: Option<String>,
    /// Validate a single Notion page by ID
    pub page_id: Option<String>,
    /// Validate the schema files themselves (no Notion API calls)
    pub self_test: Option<bool>,
    /// Validate all entries in all 5 DBs
    pub all: Option<bool>,
    /// Max entries per DB (default: 0 = unlimited)
    pub limit: Option<u32>,
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "database": {
                "type": "string",
                "enum": ["matrix", "potentiator", "nexus", "significator", "greatway"],
                "description": "Optional DB key to validate. Omit if using --all or --page-id."
            },
            "page_id": { "type": "string", "description": "Single Notion page ID to validate." },
            "self_test": { "type": "boolean", "description": "Validate the schema files themselves (no Notion API)." },
            "all": { "type": "boolean", "description": "Validate all entries in all 5 DBs." },
            "limit": { "type": "integer", "minimum": 0, "description": "Max entries per DB (0 = unlimited). Default: 0." }
        }
    })
}

pub async fn execute(
    params: &ValidateYamlParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    // Step 1: Discover the schemas directory
    let schemas_dir = YamlSchemaRegistry::discover_schemas_dir()
        .ok_or_else(|| "Could not discover schemas directory. Set LIFEOS_SCHEMAS_DIR env var or run from the lifeos-ops repo root.".to_string())?;
    tracing::info!("Loading YAML schemas from: {}", schemas_dir.display());

    // Step 2: Load all schemas
    let registry = YamlSchemaRegistry::load(&schemas_dir);
    if !registry.load_errors.is_empty() {
        let mut msg = String::from("Schema load errors:\n");
        for e in &registry.load_errors {
            msg.push_str(&format!("  - {}\n", e));
        }
        return Err(msg);
    }

    // Step 3: Self-test (if requested)
    if params.self_test.unwrap_or(false) {
        return Ok(execute_self_test(&registry));
    }

    // Step 4: Validate entries
    let limit = params.limit.unwrap_or(0) as u64;
    let mut result = ValidationResult::default();

    if let Some(ref page_id) = params.page_id {
        let page = notion.get_page(page_id).await?;
        // Determine the DB from the page parent
        let parent = page.parent.as_ref()
            .ok_or("Page has no parent")?;
        let ds_id = parent.data_source_id.as_deref()
            .or(parent.database_id.as_deref())
            .ok_or("Page parent has no data_source_id or database_id")?;
        let db_key = resolve_db_key_from_ds_id(config, ds_id)
            .ok_or_else(|| format!("Could not resolve DB key for data_source_id {}", ds_id))?;
        let (errs, warns) = validate_entry(&db_key, &page, &registry);
        result.errors.extend(errs);
        result.warnings.extend(warns);
        result.entry_count = 1;
        result.validated_count = 1;
    } else if params.all.unwrap_or(false) {
        for db_key in config.databases.keys() {
            let db_result = validate_db(db_key, limit, config, notion, &registry).await?;
            result.merge(db_result);
        }
    } else if let Some(ref db_key) = params.database {
        if !config.databases.contains_key(db_key) {
            return Err(format!("Unknown database: {}. Valid: {}", db_key,
                config.databases.keys().cloned().collect::<Vec<_>>().join(", ")));
        }
        let db_result = validate_db(db_key, limit, config, notion, &registry).await?;
        result.merge(db_result);
    } else {
        return Err("Must specify --self-test, --page-id, --database, or --all.".to_string());
    }

    result.valid = result.errors.is_empty();
    Ok(format_result(&result))
}

fn execute_self_test(registry: &YamlSchemaRegistry) -> String {
    let issues = registry.self_test();
    let mut out = String::new();
    if issues.is_empty() {
        out.push_str("✅ All schemas passed self-test.\n\n");
    } else {
        out.push_str(&format!("❌ {} schema self-test issues found:\n", issues.len()));
        for i in &issues {
            out.push_str(&format!("  - {}\n", i));
        }
        return out;
    }
    if let Some(uni) = &registry.universal {
        out.push_str(&format!("  Universal layer:        1 schema, {} properties, {} rules\n",
            uni.properties.len(), uni.validation_rules.len()));
    }
    for db in ["matrix", "potentiator", "nexus", "significator", "greatway"] {
        if let Some(layer) = registry.per_db.get(db) {
            let pet_count = registry.per_entry_type.keys().filter(|(d, _)| d == db).count();
            out.push_str(&format!("  per_db/{}.yaml:        {} properties, {} rules, {} entry-types, {} per_entry_type files\n",
                db, layer.properties.len(), layer.validation_rules.len(),
                layer.applies_to_entry_type.as_deref().map(|_| 0).unwrap_or_else(|| 0), pet_count));
        }
    }
    out.push_str(&format!("  per_entry_type total:   {} schemas\n", registry.per_entry_type.len()));
    out
}

async fn validate_db(
    db_key: &str,
    limit: u64,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    registry: &YamlSchemaRegistry,
) -> Result<ValidationResult, String> {
    let db = crate::config::resolve_db(config, db_key)
        .ok_or_else(|| format!("Unknown DB: {}", db_key))?;
    let ds_id = db.ds_id();

    let mut result = ValidationResult::default();
    let mut page_count = 0u64;

    // Use query_data_source_all for pagination
    let pages = notion.query_data_source_all(ds_id).await?;
    for page in pages {
        if limit > 0 && page_count >= limit { break; }
        page_count += 1;
        let (errs, warns) = validate_entry(db_key, &page, registry);
        result.errors.extend(errs);
        result.warnings.extend(warns);
        result.entry_count += 1;
        result.validated_count += 1;
    }
    result.valid = result.errors.is_empty();
    Ok(result)
}

fn resolve_db_key_from_ds_id<'a>(config: &'a LifeOSConfig, ds_id: &str) -> Option<&'a str> {
    for (key, db) in &config.databases {
        if db.database_id == ds_id {
            return Some(key);
        }
        if let Some(ref resolved) = db.resolved_data_source_id {
            if resolved == ds_id {
                return Some(key);
            }
        }
    }
    None
}

fn format_result(result: &ValidationResult) -> String {
    let mut out = String::new();
    out.push_str(&result.summary());
    out.push_str("\n\n");
    if !result.errors.is_empty() {
        out.push_str(&format!("First 20 errors:\n"));
        for e in result.errors.iter().take(20) {
            out.push_str(&format!("  ❌ [{}/{}] {}: {}\n",
                e.db, e.entry_type.as_deref().unwrap_or("?"),
                e.page_title.as_deref().unwrap_or("?"), e.message));
        }
        if result.errors.len() > 20 {
            out.push_str(&format!("  ... and {} more errors.\n", result.errors.len() - 20));
        }
    }
    if !result.warnings.is_empty() {
        out.push_str(&format!("\nFirst 10 warnings:\n"));
        for w in result.warnings.iter().take(10) {
            out.push_str(&format!("  ⚠️  [{}/{}] {}: {}\n",
                w.db, w.entry_type.as_deref().unwrap_or("?"),
                w.page_title.as_deref().unwrap_or("?"), w.message));
        }
        if result.warnings.len() > 10 {
            out.push_str(&format!("  ... and {} more warnings.\n", result.warnings.len() - 10));
        }
    }
    out
}
