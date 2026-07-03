//! Audit tools — orphans, validation, suggest-links
//!
//! These tools help surface data-quality issues in the LifeOS Notion workspace:
//!   - `orphans`: entries with zero populated relations (the holonic spiral is dormant
//!     for these entries — they need linking to participate in energy flow)
//!   - `validate`: entries grouped by Validation formula status (✅ Valid, ❌ Invalid,
//!     📦 Legacy, ⚠️ No Entry Type) — surfaces entries that need YAML metadata cleanup
//!   - `suggest_links`: for each orphan, finds likely cross-reservoir links via
//!     title similarity (using `strsim::normalized_levenshtein`)

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

// ── orphans ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct OrphansParams {
    /// Optional: filter to a specific database
    pub database: Option<String>,
    /// Max results per database (default: 50)
    pub limit: Option<u32>,
}

pub fn schema_orphans() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "database": { "type": "string", "description": "Optional DB key to filter orphans to. Omit to scan all 5 reservoirs." },
            "limit": { "type": "integer", "minimum": 1, "maximum": 200, "description": "Max orphans per database (default: 50)" }
        }
    })
}

pub async fn execute_orphans(
    params: &OrphansParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let limit = params.limit.unwrap_or(50).min(200) as u64;
    let db_keys: Vec<String> = if let Some(ref db) = params.database {
        if !config.databases.contains_key(db) {
            return Err(format!("Unknown database: {}. Valid: {}", db,
                config.databases.keys().cloned().collect::<Vec<_>>().join(", ")));
        }
        vec![db.clone()]
    } else {
        config.all_database_keys()
    };

    let mut per_db: Vec<serde_json::Value> = Vec::new();
    let mut total_orphans = 0usize;
    let mut total_scanned = 0usize;

    for db_key in &db_keys {
        let ds_id = match crate::config::resolve_db(config, db_key) {
            Some(db) => db.ds_id().to_string(),
            None => continue,
        };
        let db_name = config.databases.get(db_key).map(|d| d.name.clone()).unwrap_or_default();
        let db_archetype = config.databases.get(db_key)
            .and_then(|d| d.archetype.clone()).unwrap_or_default();

        // Query up to `limit` entries and identify those with zero relations
        let query = serde_json::json!({ "page_size": limit });
        let result = match notion.query_data_source(&ds_id, &query).await {
            Ok(r) => r,
            Err(e) => {
                per_db.push(serde_json::json!({
                    "database": db_key,
                    "name": db_name,
                    "error": e,
                }));
                continue;
            }
        };

        let mut orphans: Vec<serde_json::Value> = Vec::new();
        let mut entries_with_relations = 0usize;
        for page in &result.results {
            let mut has_relation = false;
            for (_prop_name, prop_value) in &page.properties {
                if let PropertyValue::Relation { relation, .. } = prop_value {
                    if !relation.is_empty() {
                        has_relation = true;
                        break;
                    }
                }
            }
            if !has_relation {
                let title = crate::transform::extract_title(page);
                let entry_type = extract_entry_type(page, config, db_key);
                let status = extract_status_string(page);
                orphans.push(serde_json::json!({
                    "id": page.id,
                    "title": title,
                    "entry_type": entry_type,
                    "status": status,
                }));
            } else {
                entries_with_relations += 1;
            }
        }

        let scanned = result.results.len();
        total_orphans += orphans.len();
        total_scanned += scanned;
        per_db.push(serde_json::json!({
            "database": db_key,
            "name": db_name,
            "archetype": db_archetype,
            "scanned": scanned,
            "entries_with_relations": entries_with_relations,
            "orphan_count": orphans.len(),
            "orphans": orphans,
        }));
    }

    let data = serde_json::json!({
        "orphans": {
            "total_orphans": total_orphans,
            "total_scanned": total_scanned,
            "orphan_rate": if total_scanned > 0 {
                (total_orphans as f64 / total_scanned as f64 * 100.0).round() / 100.0
            } else { 0.0 },
            "per_database": per_db,
        }
    });

    Ok(crate::toon_format::encode(&data))
}

// ── validate ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ValidateParams {
    /// Optional: filter to a specific database
    pub database: Option<String>,
    /// Filter by validation status: valid, invalid, legacy, missing, all
    pub status: String,
    /// Max results per database (default: 50)
    pub limit: Option<u32>,
}

pub fn schema_validate() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "database": { "type": "string", "description": "Optional DB key to filter to. Omit to scan all 5 reservoirs." },
            "status": { "type": "string", "enum": ["valid", "invalid", "legacy", "missing", "all"], "description": "Filter by validation status (default: all)" },
            "limit": { "type": "integer", "minimum": 1, "maximum": 200, "description": "Max entries per database (default: 50)" }
        }
    })
}

pub async fn execute_validate(
    params: &ValidateParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let limit = params.limit.unwrap_or(50).min(200) as u64;
    let status_filter = params.status.as_str();
    let db_keys: Vec<String> = if let Some(ref db) = params.database {
        if !config.databases.contains_key(db) {
            return Err(format!("Unknown database: {}. Valid: {}", db,
                config.databases.keys().cloned().collect::<Vec<_>>().join(", ")));
        }
        vec![db.clone()]
    } else {
        config.all_database_keys()
    };

    let mut per_db: Vec<serde_json::Value> = Vec::new();
    let mut total_valid = 0usize;
    let mut total_invalid = 0usize;
    let mut total_legacy = 0usize;
    let mut total_missing = 0usize;
    let mut total_scanned = 0usize;

    for db_key in &db_keys {
        let ds_id = match crate::config::resolve_db(config, db_key) {
            Some(db) => db.ds_id().to_string(),
            None => continue,
        };
        let db_name = config.databases.get(db_key).map(|d| d.name.clone()).unwrap_or_default();

        let query = serde_json::json!({ "page_size": limit });
        let result = match notion.query_data_source(&ds_id, &query).await {
            Ok(r) => r,
            Err(e) => {
                per_db.push(serde_json::json!({
                    "database": db_key,
                    "name": db_name,
                    "error": e,
                }));
                continue;
            }
        };

        let mut by_status: std::collections::HashMap<String, Vec<serde_json::Value>> =
            std::collections::HashMap::new();

        for page in &result.results {
            let validation = extract_validation(page);
            let bucket = classify_validation(&validation);
            // Apply status filter
            let passes_filter = match status_filter {
                "all" => true,
                "valid" => bucket == "valid",
                "invalid" => bucket == "invalid",
                "legacy" => bucket == "legacy",
                "missing" => bucket == "missing",
                _ => true,
            };
            if !passes_filter { continue; }

            let title = crate::transform::extract_title(page);
            let entry_type = extract_entry_type(page, config, db_key);

            by_status.entry(bucket.to_string()).or_default().push(serde_json::json!({
                "id": page.id,
                "title": title,
                "entry_type": entry_type,
                "validation": validation,
            }));
        }

        // Tally
        let v = by_status.get("valid").map(|v| v.len()).unwrap_or(0);
        let i = by_status.get("invalid").map(|v| v.len()).unwrap_or(0);
        let l = by_status.get("legacy").map(|v| v.len()).unwrap_or(0);
        let m = by_status.get("missing").map(|v| v.len()).unwrap_or(0);
        let scanned = result.results.len();
        total_valid += v;
        total_invalid += i;
        total_legacy += l;
        total_missing += m;
        total_scanned += scanned;

        per_db.push(serde_json::json!({
            "database": db_key,
            "name": db_name,
            "scanned": scanned,
            "by_status": {
                "valid": by_status.get("valid").cloned().unwrap_or_default(),
                "invalid": by_status.get("invalid").cloned().unwrap_or_default(),
                "legacy": by_status.get("legacy").cloned().unwrap_or_default(),
                "missing": by_status.get("missing").cloned().unwrap_or_default(),
            },
            "counts": {
                "valid": v,
                "invalid": i,
                "legacy": l,
                "missing": m,
            }
        }));
    }

    let data = serde_json::json!({
        "validation": {
            "total_scanned": total_scanned,
            "counts": {
                "valid": total_valid,
                "invalid": total_invalid,
                "legacy": total_legacy,
                "missing": total_missing,
            },
            "filter": status_filter,
            "per_database": per_db,
        }
    });

    Ok(crate::toon_format::encode(&data))
}

// ── suggest_links ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SuggestLinksParams {
    /// Optional: source database to find orphans in
    pub source: Option<String>,
    /// Optional: target database to suggest links into
    pub target: Option<String>,
    /// Min similarity score (0.0–1.0, default: 0.5)
    pub threshold: f64,
    /// Max orphans to suggest links for (default: 20)
    pub limit: Option<u32>,
}

pub fn schema_suggest_links() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "source": { "type": "string", "description": "Source DB key to find orphans in. Omit to use all 5 reservoirs." },
            "target": { "type": "string", "description": "Target DB key to suggest links into. Omit to consider all other reservoirs as targets." },
            "threshold": { "type": "number", "minimum": 0.0, "maximum": 1.0, "description": "Min similarity score (default: 0.5)" },
            "limit": { "type": "integer", "minimum": 1, "maximum": 100, "description": "Max orphans to suggest links for (default: 20)" }
        }
    })
}

pub async fn execute_suggest_links(
    params: &SuggestLinksParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let limit = params.limit.unwrap_or(20).min(100) as u64;
    let threshold = params.threshold.clamp(0.0, 1.0);

    // Determine source DB(s)
    let source_keys: Vec<String> = if let Some(ref src) = params.source {
        if !config.databases.contains_key(src) {
            return Err(format!("Unknown source database: {}", src));
        }
        vec![src.clone()]
    } else {
        config.all_database_keys()
    };

    // Determine target DB(s) — must differ from source
    let target_keys: Vec<String> = if let Some(ref tgt) = params.target {
        if !config.databases.contains_key(tgt) {
            return Err(format!("Unknown target database: {}", tgt));
        }
        vec![tgt.clone()]
    } else {
        config.all_database_keys()
    };

    // For each source DB: fetch orphans, then for each orphan, find similar entries in target DBs
    let mut all_suggestions: Vec<serde_json::Value> = Vec::new();
    let mut total_orphans_processed = 0usize;

    for src_key in &source_keys {
        if total_orphans_processed >= limit as usize { break; }

        let ds_id = match crate::config::resolve_db(config, src_key) {
            Some(db) => db.ds_id().to_string(),
            None => continue,
        };
        let src_name = config.databases.get(src_key).map(|d| d.name.clone()).unwrap_or_default();

        // Get the relation edges FROM this source DB (auto-discovered)
        let edges = schema_cache.get_relation_edges(src_key).to_vec();
        if edges.is_empty() {
            continue; // no outgoing relations to suggest
        }

        // Query up to `limit` entries from source
        let query = serde_json::json!({ "page_size": limit });
        let result = match notion.query_data_source(&ds_id, &query).await {
            Ok(r) => r,
            Err(_) => continue,
        };

        // For each target DB that has an edge from source, fetch its entries
        let mut target_entries: std::collections::HashMap<String, Vec<(String, String)>> =
            std::collections::HashMap::new();
        for edge in &edges {
            let target_db = &edge.target_db;
            if !target_keys.contains(target_db) { continue; }
            if target_db == src_key { continue; } // skip self-relations
            if target_entries.contains_key(target_db) { continue; }

            if let Some(target_cfg) = crate::config::resolve_db(config, target_db) {
                let tq = serde_json::json!({ "page_size": 100 });
                if let Ok(tr) = notion.query_data_source(target_cfg.ds_id(), &tq).await {
                    let entries: Vec<(String, String)> = tr.results.iter()
                        .map(|p| (crate::transform::extract_title(p), p.id.clone()))
                        .collect();
                    target_entries.insert(target_db.clone(), entries);
                }
            }
        }

        // For each orphan in source, score against all target entries
        for page in &result.results {
            if total_orphans_processed >= limit as usize { break; }

            // Check if this entry is an orphan (zero populated relations)
            let mut has_relation = false;
            for (_prop_name, prop_value) in &page.properties {
                if let PropertyValue::Relation { relation, .. } = prop_value {
                    if !relation.is_empty() {
                        has_relation = true;
                        break;
                    }
                }
            }
            if has_relation { continue; }

            let src_title = crate::transform::extract_title(page);
            if src_title.trim().is_empty() { continue; }

            total_orphans_processed += 1;

            let mut suggestions: Vec<serde_json::Value> = Vec::new();
            for edge in &edges {
                let target_db = &edge.target_db;
                if !target_keys.contains(target_db) { continue; }
                if target_db == src_key { continue; }
                let Some(target_list) = target_entries.get(target_db) else { continue; };

                // Score each target entry by title similarity
                let mut scored: Vec<(f64, &str, &str)> = target_list.iter()
                    .map(|(title, id)| {
                        let score = title_similarity(&src_title, title);
                        (score, title.as_str(), id.as_str())
                    })
                    .filter(|(score, _, _)| *score >= threshold)
                    .collect();
                scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

                // Top 3 suggestions per target DB
                for (score, title, id) in scored.iter().take(3) {
                    suggestions.push(serde_json::json!({
                        "target_database": target_db,
                        "target_id": id,
                        "target_title": title,
                        "via_property": edge.prop_name,
                        "similarity": (score * 100.0).round() / 100.0,
                    }));
                }
            }

            if !suggestions.is_empty() {
                let entry_type = extract_entry_type(page, config, src_key);
                all_suggestions.push(serde_json::json!({
                    "source_database": src_key,
                    "source_name": src_name,
                    "source_id": page.id,
                    "source_title": src_title,
                    "source_entry_type": entry_type,
                    "suggestions": suggestions,
                }));
            }
        }
    }

    let data = serde_json::json!({
        "suggest_links": {
            "threshold": threshold,
            "orphans_processed": total_orphans_processed,
            "orphans_with_suggestions": all_suggestions.len(),
            "results": all_suggestions,
        }
    });

    Ok(crate::toon_format::encode(&data))
}

// ── Helpers ──

/// Extract the entry-type value from a page. Uses the DB's configured
/// entry_type_property, falling back to "Entry Type" / "Item Type" / "Category".
fn extract_entry_type(page: &crate::notion::types::NotionPage, config: &LifeOSConfig, db_key: &str) -> String {
    let et_prop = config.databases.get(db_key)
        .and_then(|db| db.entry_type_property.clone())
        .or_else(|| {
            // Fallback: try common names
            for candidate in &["Entry Type", "Item Type", "Category"] {
                if page.properties.contains_key(*candidate) {
                    return Some(candidate.to_string());
                }
            }
            None
        });
    if let Some(prop) = et_prop {
        crate::transform::extract_string(page, &prop)
    } else {
        String::new()
    }
}

/// Extract the Validation formula's string result. Returns empty if no
/// Validation property exists.
fn extract_validation(page: &crate::notion::types::NotionPage) -> String {
    for (name, prop) in &page.properties {
        if name.to_lowercase().contains("validation") {
            if let PropertyValue::Formula { formula, .. } = prop {
                if let Some(ref s) = formula.string { return s.clone(); }
            }
        }
    }
    String::new()
}

/// Classify a Validation formula result into a bucket:
/// - `valid`: contains "✅"
/// - `invalid`: contains "❌"
/// - `legacy`: contains "📦" or "Legacy"
/// - `missing`: contains "⚠️" or empty / no Validation property
fn classify_validation(validation: &str) -> &'static str {
    if validation.is_empty() { return "missing"; }
    if validation.contains("✅") { return "valid"; }
    if validation.contains("❌") { return "invalid"; }
    if validation.contains("📦") || validation.to_lowercase().contains("legacy") { return "legacy"; }
    if validation.contains("⚠️") { return "missing"; }
    "missing"
}

/// Extract the first status-like property value from a page.
fn extract_status_string(page: &crate::notion::types::NotionPage) -> String {
    for (name, prop) in &page.properties {
        let lower = name.to_lowercase();
        if lower == "status" || lower == "digestion status" {
            if let PropertyValue::Status { status, .. } = prop {
                if let Some(ref s) = status { return s.name.clone(); }
            }
        }
    }
    String::new()
}

/// Compute title similarity using normalized Levenshtein + token overlap.
/// Returns a score in [0.0, 1.0].
fn title_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() { return 0.0; }
    let la = a.to_lowercase();
    let lb = b.to_lowercase();
    // Exact match
    if la == lb { return 1.0; }
    // Substring match boost
    if la.contains(&lb) || lb.contains(&la) { return 0.85; }
    // Levenshtein similarity
    let lev = strsim::normalized_levenshtein(&la, &lb);
    // Token overlap (Jaccard)
    let tokens_a: std::collections::HashSet<&str> = la.split_whitespace().collect();
    let tokens_b: std::collections::HashSet<&str> = lb.split_whitespace().collect();
    let intersection = tokens_a.intersection(&tokens_b).count() as f64;
    let union = tokens_a.union(&tokens_b).count() as f64;
    let jaccard = if union > 0.0 { intersection / union } else { 0.0 };
    // Weighted combination
    (lev * 0.6 + jaccard * 0.4).max(0.0).min(1.0)
}
