//! Energetics tools — transmute, process_currency, trigger_nexus, apply_drive
//!
//! These tools let the AI-agent OPERATE the energy-flow spiral, not just observe it.
//! They implement the four core energetic operations described in the v4 architecture:
//!
//! - **transmute**: Core Nexus operation — move an entry's currency across a cycle boundary
//! - **process_currency**: Lifecycle advancement — advance an entry through its status progression
//! - **trigger_nexus**: Threshold detection and restructuring — fire the Nexus when pressure exceeds threshold
//! - **apply_drive**: Drive application — regulate boundary conditions with Agency/Communion/Eros/Agape

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;



// ══════════════════════════════════════════════════════════════════════
// Tool 1: transmute
// ══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct TransmuteParams {
    /// ID of the entry to transmute FROM
    pub source_entry: String,
    /// Reservoir to transmute INTO
    pub target_reservoir: String,
    /// Type of currency transmutation
    pub transmutation_type: String,
    /// Content for the new entry
    #[serde(default)]
    pub content: Option<TransmuteContent>,
    /// Create relation from target back to source (default: true)
    #[serde(default = "default_true")]
    pub link_back: bool,
    /// Create a Nexus entry logging this transmutation (default: true)
    #[serde(default = "default_true")]
    pub nexus_log: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Deserialize)]
pub struct TransmuteContent {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub properties: Option<serde_json::Value>,
}

pub fn transmute_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "source_entry": { "type": "string", "description": "ID of the entry to transmute FROM" },
            "target_reservoir": { "type": "string", "enum": ["matrix", "potentiator", "significator", "greatway", "nexus"], "description": "Reservoir to transmute INTO" },
            "transmutation_type": { "type": "string", "enum": ["catalyst_to_experience", "experience_to_catalyst", "choice_to_transformation", "transformation_to_choice", "to_nexus", "from_nexus"], "description": "Type of currency transmutation" },
            "content": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Title for the new entry" },
                    "description": { "type": "string", "description": "Content/body of the transmuted entry" },
                    "properties": { "type": "object", "description": "Additional properties (key-value pairs using config keys)" }
                },
                "required": ["title"]
            },
            "link_back": { "type": "boolean", "description": "Create a relation from target back to source (default: true)" },
            "nexus_log": { "type": "boolean", "description": "Create a Nexus entry logging this transmutation (default: true)" }
        },
        "required": ["source_entry", "target_reservoir", "transmutation_type"]
    })
}

pub async fn execute_transmute(
    params: &TransmuteParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    // 1. Fetch source entry
    let source_page = notion.get_page(&params.source_entry).await?;

    // 2. Determine source reservoir
    let source_reservoir = crate::tools::shared::get_entry_reservoir(&source_page, config)
        .ok_or_else(|| "Could not determine source reservoir".to_string())?;

    // 3. Validate transmutation type matches source/target pair
    let effective_source = crate::tools::shared::effective_reservoir(&source_reservoir, config)
        .unwrap_or_else(|| source_reservoir.clone());

    // Check config map first, fall back to archetype-based inference
    let valid_source = if let Some(tt_def) = config.transmutation_def(&params.transmutation_type) {
        tt_def.source == effective_source
    } else {
        // Fallback: infer from archetypes
        let inferred = crate::tools::shared::get_transmutation_type(
            &effective_source, &params.target_reservoir, config,
        );
        inferred.as_deref() == Some(params.transmutation_type.as_str())
    };

    if !valid_source {
        return Err(format!(
            "Transmutation type '{}' is not valid for source '{}' (effective: '{}')",
            params.transmutation_type, source_reservoir, effective_source
        ));
    }

    // 4. Resolve target reservoir
    let target_db = config.databases.get(&params.target_reservoir)
        .ok_or_else(|| format!("Unknown target reservoir: {}", params.target_reservoir))?;

    let target_ds_id = target_db.ds_id().to_string();
    let target_props = target_db.properties.clone();

    // 5. Build target entry properties
    let mut properties = serde_json::Map::new();

    if let Some(ref content) = params.content {
        // Title
        if let Some(title_prop) = target_props.get("title") {
            properties.insert(title_prop.clone(), serde_json::json!({
                "title": [{ "text": { "content": content.title } }]
            }));
        }

        // Status: start at first stage of target reservoir's progression
        if let Some(status_prop) = target_props.get("status") {
            let progression = config.status_progression(&params.target_reservoir);
            let initial_status = progression.first().map(|s| s.as_str()).unwrap_or("Raw");
            properties.insert(status_prop.clone(), serde_json::json!({
                "status": { "name": initial_status }
            }));
        }

        // User-provided properties (config keys → Notion property names)
        if let Some(ref user_props) = content.properties {
            if let Some(obj) = user_props.as_object() {
                for (key, val) in obj {
                    if let Some(notion_name) = target_props.get(key) {
                        // Simple value mapping based on type detection
                        match val {
                            serde_json::Value::String(s) => {
                                properties.insert(notion_name.clone(), serde_json::json!({
                                    "rich_text": [{ "text": { "content": s } }]
                                }));
                            }
                            serde_json::Value::Number(n) => {
                                properties.insert(notion_name.clone(), serde_json::json!({
                                    "number": n
                                }));
                            }
                            serde_json::Value::Bool(b) => {
                                properties.insert(notion_name.clone(), serde_json::json!({
                                    "checkbox": b
                                }));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    } else {
        // No content provided — create minimal entry
        if let Some(title_prop) = target_props.get("title") {
            let source_title = crate::transform::extract_title(&source_page);
            let auto_title = format!("{} (transmuted)", source_title);
            properties.insert(title_prop.clone(), serde_json::json!({
                "title": [{ "text": { "content": auto_title } }]
            }));
        }
    }

    // 6. Create target entry
    let create_body = serde_json::json!({
        "parent": { "data_source_id": target_ds_id },
        "properties": properties,
    });

    let target_page = notion.create_page(&create_body).await?;

    // 7. Update source entry status to final stage
    let effective_source_key = crate::tools::shared::effective_reservoir(&source_reservoir, config)
        .unwrap_or_else(|| source_reservoir.clone());
    let source_db = config.databases.get(&effective_source_key);
    if let Some(db) = source_db {
        let progression = config.status_progression(&effective_source_key);
        if let Some(final_status) = progression.last() {
            let _ = crate::tools::shared::update_entry_status(
                notion, &params.source_entry, final_status, &db.properties,
            ).await;
        }
    }

    // 8. Link back
    if params.link_back {
        // Find a relation property on the target that points to the source reservoir
        let edges = schema_cache.get_relation_edges(&params.target_reservoir);
        for edge in edges {
            if edge.target_db == effective_source {
                let link_body = serde_json::json!({
                    "properties": {
                        edge.prop_name.clone(): {
                            "relation": [{ "id": &params.source_entry }]
                        }
                    }
                });
                let _ = notion.update_page(&target_page.id, &link_body).await;
                break;
            }
        }
    }

    // 9. Nexus log
    let mut nexus_result = serde_json::json!({});
    if params.nexus_log {
        match crate::tools::shared::create_nexus_log(
            config, notion, &params.transmutation_type,
            &params.source_entry, &target_page.id,
            &effective_source, &params.target_reservoir,
        ).await {
            Ok(log) => nexus_result = log,
            Err(e) => {
                nexus_result = serde_json::json!({ "error": e });
            }
        }
    }

    let source_title = crate::transform::extract_title(&source_page);
    let target_title = crate::transform::extract_title(&target_page);

    Ok(crate::toon_format::encode(&serde_json::json!({
        "analysis": "transmute",
        "transmutation_type": params.transmutation_type,
        "source": {
            "id": params.source_entry,
            "title": source_title,
            "reservoir": effective_source,
            "status": crate::tools::shared::get_entry_status(&source_page),
        },
        "target": {
            "id": target_page.id,
            "title": target_title,
            "reservoir": params.target_reservoir,
            "status": crate::tools::shared::get_entry_status(&target_page),
        },
        "nexus_log": nexus_result,
        "interpretation": format!(
            "Transmuted '{}' ({}) → '{}' ({}) via {}",
            source_title, effective_source, target_title, params.target_reservoir,
            params.transmutation_type
        ),
    })))
}

// ══════════════════════════════════════════════════════════════════════
// Tool 2: process_currency
// ══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct ProcessCurrencyParams {
    /// ID of the entry to process
    pub entry_id: String,
    /// Specific status to advance to (optional — if omitted, advances one stage)
    #[serde(default)]
    pub advance_to: Option<String>,
    /// If entry reaches final stage, automatically transmute to target reservoir
    #[serde(default)]
    pub auto_transmute: bool,
    /// Target reservoir for auto-transmute
    #[serde(default)]
    pub target_reservoir: Option<String>,
    /// Content to add/update when advancing
    #[serde(default)]
    pub enrichment: Option<serde_json::Value>,
    /// Process multiple entries at once
    #[serde(default)]
    pub batch: Option<Vec<String>>,
}

pub fn process_currency_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "entry_id": { "type": "string", "description": "ID of the entry to process" },
            "advance_to": { "type": "string", "description": "Specific status to advance to (optional — if omitted, advances one stage)" },
            "auto_transmute": { "type": "boolean", "description": "If entry reaches final stage, automatically transmute (default: false)" },
            "target_reservoir": { "type": "string", "enum": ["matrix", "potentiator", "significator", "greatway"], "description": "Target reservoir for auto-transmute" },
            "enrichment": { "type": "object", "description": "Content to add/update when advancing" },
            "batch": { "type": "array", "items": { "type": "string" }, "description": "Process multiple entry IDs" }
        },
        "required": ["entry_id"]
    })
}

pub async fn execute_process_currency(
    params: &ProcessCurrencyParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    // Handle batch mode — delegate to single-entry helper to avoid recursive async
    if let Some(ref batch) = params.batch {
        let mut results = Vec::new();
        for entry_id in batch {
            let single_params = ProcessCurrencyParams {
                entry_id: entry_id.clone(),
                advance_to: params.advance_to.clone(),
                auto_transmute: params.auto_transmute,
                target_reservoir: params.target_reservoir.clone(),
                enrichment: params.enrichment.clone(),
                batch: None,
            };
            match process_single_entry(&single_params, config, notion, schema_cache).await {
                Ok(result) => results.push(serde_json::json!({ "entry_id": entry_id, "status": "ok", "result": result })),
                Err(e) => results.push(serde_json::json!({ "entry_id": entry_id, "status": "error", "error": e })),
            }
        }
        return Ok(crate::toon_format::encode(&serde_json::json!({
            "analysis": "process_currency_batch",
            "results": results,
            "processed": results.len(),
        })));
    }

    process_single_entry(params, config, notion, schema_cache).await
}

/// Process a single entry — extracted to avoid recursive async.
async fn process_single_entry(
    params: &ProcessCurrencyParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    // 1. Fetch entry
    let page = notion.get_page(&params.entry_id).await?;

    // 2. Determine reservoir
    let reservoir_key = crate::tools::shared::get_entry_reservoir(&page, config)
        .ok_or_else(|| "Could not determine entry's reservoir".to_string())?;

    let effective_key = crate::tools::shared::effective_reservoir(&reservoir_key, config)
        .unwrap_or_else(|| reservoir_key.clone());

    // 3. Get current status
    let current_status = crate::tools::shared::get_entry_status(&page);

    // 4. Determine next status
    let next_status = if let Some(ref target) = params.advance_to {
        // Validate target is in the progression
        let progression = config.status_progression(&effective_key);
        if progression.iter().any(|s| s.to_lowercase() == target.to_lowercase()) {
            target.clone()
        } else {
            return Err(format!(
                "Status '{}' is not in the progression for {}: {:?}",
                target, effective_key, progression
            ));
        }
    } else {
        crate::tools::shared::advance_status(&effective_key, &current_status, config)
            .ok_or_else(|| format!(
                "Entry '{}' is already at final stage '{}' in {}",
                params.entry_id, current_status, effective_key
            ))?
    };

    // 5. Apply enrichment if provided
    if let Some(ref enrichment) = params.enrichment {
        if let Some(obj) = enrichment.as_object() {
            let db = config.databases.get(&effective_key)
                .or_else(|| {
                    // It's a satellite — find parent
                    for (_rk, db) in &config.databases {
                        if db.satellites.contains_key(&reservoir_key) {
                            return Some(db);
                        }
                    }
                    None
                });
            if let Some(db) = db {
                let mut update_props = serde_json::Map::new();
                for (key, val) in obj {
                    if let Some(notion_name) = db.properties.get(key) {
                        match val {
                            serde_json::Value::String(s) => {
                                update_props.insert(notion_name.clone(), serde_json::json!({
                                    "rich_text": [{ "text": { "content": s } }]
                                }));
                            }
                            serde_json::Value::Number(n) => {
                                update_props.insert(notion_name.clone(), serde_json::json!({ "number": n }));
                            }
                            _ => {}
                        }
                    }
                }
                if !update_props.is_empty() {
                    let _ = notion.update_page(&params.entry_id, &serde_json::json!({
                        "properties": update_props
                    })).await;
                }
            }
        }
    }

    // 6. Update status
    let db = config.databases.get(&effective_key)
        .ok_or_else(|| format!("Database config not found for {}", effective_key))?;
    crate::tools::shared::update_entry_status(
        notion, &params.entry_id, &next_status, &db.properties,
    ).await?;

    let title = crate::transform::extract_title(&page);

    // 7. Check if we reached final stage
    let reached_final = crate::tools::shared::is_final_stage(&effective_key, &next_status, config);

    // 8. Auto-transmute if at final stage
    let mut transmute_result = serde_json::json!(null);
    if reached_final && params.auto_transmute {
        let target = params.target_reservoir.clone()
            .or_else(|| crate::tools::shared::auto_transmute_target(&effective_key, config))
            .ok_or_else(|| format!("No auto-transmute target for {}", effective_key))?;

        let transmutation_type = crate::tools::shared::get_transmutation_type(
            &effective_key, &target, config,
        ).unwrap_or_else(|| "unknown".into());

        let transmute_params = TransmuteParams {
            source_entry: params.entry_id.clone(),
            target_reservoir: target.clone(),
            transmutation_type,
            content: None, // auto-generate
            link_back: true,
            nexus_log: true,
        };
        transmute_result = match execute_transmute(&transmute_params, config, notion, schema_cache).await {
            Ok(r) => serde_json::json!({ "status": "auto_transmuted", "result": r }),
            Err(e) => serde_json::json!({ "status": "transmute_failed", "error": e }),
        };
    }

    let progression = config.status_progression(&effective_key);

    Ok(crate::toon_format::encode(&serde_json::json!({
        "analysis": "process_currency",
        "entry": {
            "id": params.entry_id,
            "title": title,
            "reservoir": effective_key,
        },
        "old_status": current_status,
        "new_status": next_status,
        "progression": progression,
        "progress": if progression.is_empty() {
            "N/A".to_string()
        } else {
            format!("{}/{}", 
                progression.iter().position(|s| s.to_lowercase() == next_status.to_lowercase()).unwrap_or(0) + 1,
                progression.len()
            )
        },
        "reached_final_stage": reached_final,
        "auto_transmuted": transmute_result,
        "interpretation": if reached_final {
            format!("'{}' has completed its currency lifecycle at '{}' and is ready for transmutation", title, next_status)
        } else {
            format!("'{}' advanced from '{}' to '{}' — {} stages remaining",
                title, current_status, next_status,
                progression.iter().position(|s| s.to_lowercase() == next_status.to_lowercase())
                    .map(|p| progression.len() - p - 1)
                    .unwrap_or(0)
            )
        },
    })))
}

// ══════════════════════════════════════════════════════════════════════
// Tool 3: trigger_nexus
// ══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct TriggerNexusParams {
    /// detect: check if Nexus should fire; execute: actually fire; dry_run: show what would happen
    pub mode: String,
    /// Override default thresholds
    #[serde(default)]
    pub threshold_override: Option<NexusThresholds>,
    /// Which cycles to restructure
    #[serde(default = "default_full_scope")]
    pub restructure_scope: String,
}

fn default_full_scope() -> String { "full".into() }

#[derive(Debug, Deserialize)]
pub struct NexusThresholds {
    #[serde(default)]
    pub gz_threshold: Option<f64>,
    #[serde(default)]
    pub pz_threshold: Option<f64>,
    #[serde(default)]
    pub pressure_threshold: Option<f64>,
}

pub fn trigger_nexus_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "mode": { "type": "string", "enum": ["detect", "execute", "dry_run"], "description": "detect: check if Nexus should fire; execute: fire; dry_run: preview" },
            "threshold_override": {
                "type": "object",
                "properties": {
                    "gz_threshold": { "type": "number", "description": "G_z threshold (default: 35)" },
                    "pz_threshold": { "type": "number", "description": "P_z threshold (default: 75)" },
                    "pressure_threshold": { "type": "number", "description": "Combined pressure threshold (default: 110)" }
                }
            },
            "restructure_scope": { "type": "string", "enum": ["lesser_only", "greater_only", "full"], "description": "Which cycles to restructure (default: full)" }
        },
        "required": ["mode"]
    })
}

pub async fn execute_trigger_nexus(
    params: &TriggerNexusParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    // 1. Calculate G_z and P_z directly (avoid TOON round-trip)
    let date_filter = None;
    let gz_val = crate::tools::health_metrics::calculate_g_z(
        config, notion, schema_cache, &date_filter,
    ).await;
    let pz_val = crate::tools::health_metrics::calculate_p_z(
        config, notion, schema_cache, &date_filter,
    ).await;

    let gz_score = gz_val.get("score").and_then(|v| v.as_f64()).unwrap_or(50.0);
    let pz_score = pz_val.get("score").and_then(|v| v.as_f64()).unwrap_or(50.0);

    // 2. Apply thresholds
    let firing_config = config.nexus_firing_config();
    let gz_threshold = params.threshold_override.as_ref()
        .and_then(|t| t.gz_threshold)
        .unwrap_or(firing_config.gz_threshold);
    let pz_threshold = params.threshold_override.as_ref()
        .and_then(|t| t.pz_threshold)
        .unwrap_or(firing_config.pz_threshold);
    let pressure_threshold = params.threshold_override.as_ref()
        .and_then(|t| t.pressure_threshold)
        .unwrap_or(firing_config.pressure_threshold);

    // 3. Calculate pressure
    // pressure = P_z (drive) + (100 - G_z) (deficit) — high when drive is strong AND coherence is weak
    let pressure = pz_score + (100.0 - gz_score);
    let should_fire = pressure > pressure_threshold && pz_score > pz_threshold;

    // 4. Find entries ready for promotion (at final stage of lesser/greater cycles)
    let lesser_keys = config.cycle_reservoirs("lesser");
    let greater_keys = config.cycle_reservoirs("greater");

    let mut ready_entries: std::collections::HashMap<String, Vec<serde_json::Value>> = std::collections::HashMap::new();

    let all_keys: Vec<String> = match params.restructure_scope.as_str() {
        "lesser_only" => lesser_keys.clone(),
        "greater_only" => greater_keys.clone(),
        _ => {
            let mut keys = lesser_keys.clone();
            keys.extend(greater_keys.clone());
            keys
        }
    };

    for res_key in &all_keys {
        let db = match config.databases.get(res_key) {
            Some(db) => db,
            None => continue,
        };
        let progression = config.status_progression(res_key);
        let final_status = match progression.last() {
            Some(s) => s.clone(),
            None => continue,
        };

        // Query entries at the final stage
        let status_prop_name = db.properties.get("status")
            .map(|s| s.as_str())
            .unwrap_or("Status");
        // All LifeOS DBs use "status" type — this is safe for the current schema
        if let Ok(result) = crate::tools::shared::query_entries_with_status(
            notion, db.ds_id(), status_prop_name, "status", &final_status, 50,
        ).await {
            let entries: Vec<serde_json::Value> = result.iter().map(|p| {
                serde_json::json!({
                    "id": p.id,
                    "title": crate::transform::extract_title(p),
                    "reservoir": res_key,
                    "status": final_status,
                })
            }).collect();
            if !entries.is_empty() {
                ready_entries.insert(res_key.clone(), entries);
            }
        }
    }

    let total_ready: usize = ready_entries.values().map(|v| v.len()).sum();

    // 5. Execute if mode is execute and should_fire
    let mut promotions: Vec<serde_json::Value> = Vec::new();
    if params.mode == "execute" && should_fire {
        for (res_key, entries) in &ready_entries {
            let progression = config.status_progression(res_key);
            let current_final = match progression.last() {
                Some(s) => s.clone(),
                None => continue,
            };

            // For lesser cycle: promote final stage and reset to Raw (new octave)
            // For greater cycle: promote final stage
            let is_lesser = lesser_keys.contains(res_key);
            let next_status = if is_lesser {
                // Reset to beginning of next octave
                progression.first().cloned().unwrap_or_else(|| "Raw".into())
            } else {
                // Promote within the greater cycle
                progression.first().cloned().unwrap_or_else(|| "Proposed".into())
            };

            let db = match config.databases.get(res_key) {
                Some(db) => db,
                None => continue,
            };

            for entry in entries {
                if let Some(entry_id) = entry.get("id").and_then(|v| v.as_str()) {
                    let _ = crate::tools::shared::update_entry_status(
                        notion, entry_id, &next_status, &db.properties,
                    ).await;
                    promotions.push(serde_json::json!({
                        "entry_id": entry_id,
                        "title": entry.get("title"),
                        "reservoir": res_key,
                        "from": current_final,
                        "to": next_status,
                    }));
                }
            }
        }

        // Create Nexus restructuring entry
        let _ = crate::tools::shared::create_nexus_event(
            config, notion,
            "octave_shift",
            "Restructuring",
            &format!("pressure: {:.0}, promoted: {} entries", pressure, promotions.len()),
        ).await;
    }

    Ok(crate::toon_format::encode(&serde_json::json!({
        "analysis": "trigger_nexus",
        "mode": params.mode,
        "health": {
            "G_z": gz_score,
            "P_z": pz_score,
            "pressure": (pressure * 10.0).round() / 10.0,
        },
        "thresholds": {
            "gz_threshold": gz_threshold,
            "pz_threshold": pz_threshold,
            "pressure_threshold": pressure_threshold,
        },
        "should_fire": should_fire,
        "entries_ready": total_ready,
        "ready_by_reservoir": ready_entries,
        "promotions": promotions,
        "interpretation": if should_fire {
            format!(
                "Nexus SHOULD FIRE — pressure ({:.0}) exceeds threshold ({:.0}). P_z ({:.0}) > threshold ({:.0}). {} entries ready for promotion.",
                pressure, pressure_threshold, pz_score, pz_threshold, total_ready
            )
        } else {
            format!(
                "Nexus not ready — pressure ({:.0}) below threshold ({:.0}). {} entries at final stage but threshold not exceeded.",
                pressure, pressure_threshold, total_ready
            )
        },
    })))
}

// ══════════════════════════════════════════════════════════════════════
// Tool 4: apply_drive
// ══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct ApplyDriveParams {
    /// Drive to apply
    pub drive: String,
    /// Boundary: "lesser" (Matrix⇌Potentiator), "greater" (Significator⇌GreatWay), or "both"
    pub boundary: String,
    /// How strongly to apply
    #[serde(default = "default_moderate")]
    pub intensity: String,
    /// Specific entry IDs to apply the drive to
    #[serde(default)]
    pub target_entries: Option<Vec<String>>,
    /// Why this drive is being applied (logged in Nexus)
    #[serde(default)]
    pub rationale: Option<String>,
}

fn default_moderate() -> String { "moderate".into() }

pub fn apply_drive_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "drive": { "type": "string", "enum": ["Agency", "Communion", "Eros", "Agape"], "description": "Drive to apply" },
            "boundary": { "type": "string", "enum": ["lesser", "greater", "both"], "description": "Which boundary to apply the drive at" },
            "intensity": { "type": "string", "enum": ["gentle", "moderate", "strong"], "description": "How strongly (default: moderate)" },
            "target_entries": { "type": "array", "items": { "type": "string" }, "description": "Specific entry IDs (optional — if omitted, applies to all entries at boundary)" },
            "rationale": { "type": "string", "description": "Why this drive is being applied (logged in Nexus)" }
        },
        "required": ["drive", "boundary"]
    })
}

pub async fn execute_apply_drive(
    params: &ApplyDriveParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    // Determine boundary reservoirs
    let boundary_keys: Vec<String> = match params.boundary.as_str() {
        "lesser" => config.cycle_reservoirs("lesser"),
        "greater" => config.cycle_reservoirs("greater"),
        "both" => {
            let mut keys = config.cycle_reservoirs("lesser");
            keys.extend(config.cycle_reservoirs("greater"));
            keys
        }
        _ => return Err(format!("Unknown boundary: {}", params.boundary)),
    };

    let mut actions: Vec<serde_json::Value> = Vec::new();
    let mut entries_affected = 0;

    for res_key in &boundary_keys {
        let db = match config.databases.get(res_key) {
            Some(db) => db,
            None => continue,
        };
        let archetype = db.archetype.as_deref().unwrap_or("unknown");

        // Determine which status to target based on drive + archetype
        let target_status = match (params.drive.as_str(), archetype) {
            // Agency: strengthen boundaries — target "Active" entries (already resisting)
            ("Agency", _) => Some("Active"),
            // Communion: open to new input — target "Raw" entries (signal readiness)
            ("Communion", _) => Some("Raw"),
            // Eros: intensify drive — target all entries (batch acceleration)
            ("Eros", _) => None,
            // Agape: integrate experience — target "Processing"/"Crystallized" entries
            ("Agape", _) => Some("Processing"),
            _ => None,
        };

        // Resolve the Notion status property name from config
        let status_prop_name = db.properties.get("status")
            .map(|s| s.as_str())
            .unwrap_or("Status");

        // Query entries to affect
        let entries = if let Some(ref target_ids) = params.target_entries {
            // Use specific entries
            let mut specific = Vec::new();
            for id in target_ids {
                if let Ok(page) = notion.get_page(id).await {
                    specific.push(page);
                }
            }
            specific
        } else if let Some(status) = target_status {
            crate::tools::shared::query_entries_with_status(notion, db.ds_id(), status_prop_name, "status", status, 20).await.unwrap_or_default()
        } else {
            crate::tools::shared::query_all_entries(notion, db.ds_id(), 20).await.unwrap_or_default()
        };

        let intensity_multiplier = match params.intensity.as_str() {
            "gentle" => 1,
            "moderate" => 5,
            "strong" => 10,
            _ => 5,
        };

        let entries_to_process: Vec<_> = entries.into_iter().take(intensity_multiplier).collect();
        let count = entries_to_process.len();

        for page in &entries_to_process {
            let entry_title = crate::transform::extract_title(page);

            match params.drive.as_str() {
                "Agency" => {
                    // Strengthen: ensure status is "Active" (resisting change)
                    if let Some(status_prop) = db.properties.get("status") {
                        let body = serde_json::json!({
                            "properties": {
                                status_prop: { "status": { "name": "Active" } }
                            }
                        });
                        let _ = notion.update_page(&page.id, &body).await;
                    }
                    actions.push(serde_json::json!({
                        "action": "strengthen_boundary",
                        "entry": entry_title,
                        "reservoir": res_key,
                    }));
                }
                "Communion" => {
                    // Open: advance from current status (signal readiness for digestion)
                    let current = crate::tools::shared::get_entry_status(page);
                    if let Some(next) = crate::tools::shared::advance_status(res_key, &current, config) {
                        let _ = crate::tools::shared::update_entry_status(
                            notion, &page.id, &next, &db.properties,
                        ).await;
                        actions.push(serde_json::json!({
                            "action": "open_boundary",
                            "entry": entry_title,
                            "reservoir": res_key,
                            "from": current,
                            "to": next,
                        }));
                    }
                }
                "Eros" => {
                    // Intensify: advance one stage (batch acceleration)
                    let current = crate::tools::shared::get_entry_status(page);
                    if let Some(next) = crate::tools::shared::advance_status(res_key, &current, config) {
                        let _ = crate::tools::shared::update_entry_status(
                            notion, &page.id, &next, &db.properties,
                        ).await;
                        actions.push(serde_json::json!({
                            "action": "intensify_drive",
                            "entry": entry_title,
                            "reservoir": res_key,
                            "from": current,
                            "to": next,
                        }));
                    }
                }
                "Agape" => {
                    // Integrate: advance "Processing" → "Crystallized" (consolidate learning)
                    let current = crate::tools::shared::get_entry_status(page);
                    if let Some(next) = crate::tools::shared::advance_status(res_key, &current, config) {
                        let _ = crate::tools::shared::update_entry_status(
                            notion, &page.id, &next, &db.properties,
                        ).await;
                        actions.push(serde_json::json!({
                            "action": "integrate_experience",
                            "entry": entry_title,
                            "reservoir": res_key,
                            "from": current,
                            "to": next,
                        }));
                    }
                }
                _ => {}
            }
        }
        entries_affected += count;
    }

    // Log the drive application in Nexus
    let rationale = params.rationale.clone().unwrap_or_else(|| {
        format!("AI-agent applied {} at {} boundary with {} intensity", params.drive, params.boundary, params.intensity)
    });

    let _ = crate::tools::shared::create_nexus_event(
        config, notion,
        &format!("drive_{}", params.drive.to_lowercase()),
        "Drive Application",
        &format!("{} at {} boundary ({}): {}", params.drive, params.boundary, params.intensity, rationale),
    ).await;

    Ok(crate::toon_format::encode(&serde_json::json!({
        "analysis": "apply_drive",
        "drive": params.drive,
        "boundary": params.boundary,
        "intensity": params.intensity,
        "entries_affected": entries_affected,
        "actions": actions,
        "nexus_log": {
            "type": format!("drive_{}", params.drive.to_lowercase()),
            "rationale": rationale,
        },
        "interpretation": format!(
            "Applied {} at {} boundary ({}) — {} entries affected via {} actions",
            params.drive, params.boundary, params.intensity,
            entries_affected, actions.len()
        ),
    })))
}
