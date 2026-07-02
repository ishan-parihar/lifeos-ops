//! Shared reservoir query helper and energetic utility functions.

use std::sync::Arc;
use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::NotionPage;

/// Query a single reservoir database, applying date filter if available.
/// Returns total count, has_more, status_distribution, and digestion_distribution.
/// Accepts both reservoir and satellite keys via resolve_db.
pub async fn query_reservoir(
    config: &LifeOSConfig,
    notion: &NotionClient,
    key: &str,
    date_filter: &Option<serde_json::Value>,
    page_size: u32,
) -> serde_json::Value {
    let db = match crate::config::resolve_db(config, key) {
        Some(crate::config::ResolvedDb::Reservoir(_k, db)) => db,
        Some(crate::config::ResolvedDb::Satellite(_rk, _sk, sat)) => {
            return query_ds(notion, sat.ds_id(), &sat.properties, date_filter, page_size).await;
        }
        None => return serde_json::json!({ "total": 0 }),
    };
    query_ds(notion, db.ds_id(), &db.properties, date_filter, page_size).await
}

async fn query_ds(
    notion: &NotionClient,
    ds_id: &str,
    properties: &std::collections::HashMap<String, String>,
    date_filter: &Option<serde_json::Value>,
    page_size: u32,
) -> serde_json::Value {
    let mut query = serde_json::json!({ "page_size": page_size });
    if let Some(ref filter) = date_filter {
        if let Some(date_prop) = properties.get("date") {
            let mut f = filter.clone();
            f["property"] = serde_json::json!(date_prop);
            query["filter"] = f;
        }
    }

    match notion.query_data_source(ds_id, &query).await {
        Ok(result) => {
            let mut status_dist: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            let mut digestion_dist: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            for page in &result.results {
                let status = crate::transform::extract_string(page, "Status");
                *status_dist.entry(status).or_insert(0) += 1;
                let digestion = crate::transform::extract_string(page, "Digestion Status");
                if !digestion.is_empty() {
                    *digestion_dist.entry(digestion).or_insert(0) += 1;
                }
            }
            serde_json::json!({
                "total": result.results.len(),
                "has_more": result.has_more,
                "status_distribution": status_dist,
                "digestion_distribution": digestion_dist
            })
        }
        Err(_) => serde_json::json!({ "total": 0 }),
    }
}

// ── Energetic Utility Functions ──────────────────────────────────────

/// Determine which reservoir (or satellite) owns a Notion page.
/// Returns the reservoir key (or satellite key if it's a satellite).
pub fn get_entry_reservoir(page: &NotionPage, config: &LifeOSConfig) -> Option<String> {
    let parent_ds_id = page.parent.as_ref().and_then(|p| p.data_source_id.as_deref())?;
    let ds_id = parent_ds_id;

    for (res_key, db) in &config.databases {
        // Check if it's the reservoir itself
        if ds_id == db.ds_id() {
            return Some(res_key.clone());
        }
        // Check satellites
        for (sat_key, sat) in &db.satellites {
            if ds_id == sat.ds_id() {
                return Some(sat_key.clone());
            }
        }
    }
    None
}

/// Get the archetype for a reservoir key.
pub fn get_reservoir_archetype(reservoir_key: &str, config: &LifeOSConfig) -> Option<String> {
    // Direct reservoir match
    if let Some(db) = config.databases.get(reservoir_key) {
        return db.archetype.clone();
    }
    // Satellite — look up parent reservoir's archetype
    for (_res_key, db) in &config.databases {
        if db.satellites.contains_key(reservoir_key) {
            return db.archetype.clone();
        }
    }
    None
}

/// Get the parent reservoir key for a satellite key.
pub fn parent_reservoir(satellite_key: &str, config: &LifeOSConfig) -> Option<String> {
    for (res_key, db) in &config.databases {
        if db.satellites.contains_key(satellite_key) {
            return Some(res_key.clone());
        }
    }
    None
}

/// Determine which "effective" reservoir an entry belongs to for energetic purposes.
/// Satellites inherit their parent reservoir's archetype.
pub fn effective_reservoir(key: &str, config: &LifeOSConfig) -> Option<String> {
    if config.databases.contains_key(key) {
        Some(key.to_string())
    } else {
        parent_reservoir(key, config)
    }
}

/// Get the status of an entry from its Status property.
pub fn get_entry_status(page: &NotionPage) -> String {
    crate::transform::extract_string(page, "Status")
}

/// Get the next status in a reservoir's progression, given the current status.
/// Returns None if already at the final stage.
pub fn advance_status(reservoir_key: &str, current_status: &str, config: &LifeOSConfig) -> Option<String> {
    let progression = config.status_progression(reservoir_key);
    let current_lower = current_status.to_lowercase();
    for (i, stage) in progression.iter().enumerate() {
        if stage.to_lowercase() == current_lower {
            return progression.get(i + 1).cloned();
        }
    }
    None
}

/// Check if an entry is at the final stage of its progression.
pub fn is_final_stage(reservoir_key: &str, current_status: &str, config: &LifeOSConfig) -> bool {
    let progression = config.status_progression(reservoir_key);
    if let Some(last) = progression.last() {
        last.to_lowercase() == current_status.to_lowercase()
    } else {
        false
    }
}

/// Determine the transmutation type needed when an entry at the final stage
/// of one reservoir should generate its out-currency in the target reservoir.
pub fn get_transmutation_type(
    source_reservoir: &str,
    target_reservoir: &str,
    config: &LifeOSConfig,
) -> Option<String> {
    if let Some(ref holonic) = config.holonic {
        for (tt, def) in &holonic.transmutation_map {
            if def.source == source_reservoir && def.target == target_reservoir {
                return Some(tt.clone());
            }
        }
    }
    // Fallback: infer from archetype pairs
    match (source_reservoir, target_reservoir) {
        ("potentiator", "matrix") => Some("catalyst_to_experience".into()),
        ("matrix", "potentiator") => Some("experience_to_catalyst".into()),
        ("significator", "greatway") => Some("choice_to_transformation".into()),
        ("greatway", "significator") => Some("transformation_to_choice".into()),
        _ => None,
    }
}

/// Determine the automatic transmutation target for a reservoir at its final stage.
pub fn auto_transmute_target(reservoir_key: &str, config: &LifeOSConfig) -> Option<String> {
    // Get the archetype to determine the flow direction
    let archetype = get_reservoir_archetype(reservoir_key, config)?;
    match archetype.as_str() {
        "matrix" => Some("potentiator".into()),  // Matrix generates Experience → Potentiator
        "potentiator" => Some("matrix".into()),   // Potentiator generates Catalyst → Matrix
        "significator" => Some("greatway".into()), // Significator generates Choice → GreatWay
        "greatway" => Some("significator".into()), // GreatWay generates Transformation → Significator
        _ => None,
    }
}

/// Create a Nexus event entry (for non-transmutation events like restructuring, drive application).
pub async fn create_nexus_event(
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    event_type: &str,
    category: &str,
    details: &str,
) -> Result<serde_json::Value, String> {
    let nexus_key = config.reservoir_by_archetype("nexus")
        .map(|(k, _)| k.to_string())
        .unwrap_or_else(|| "nexus".into());

    let (nexus_ds_id, _, nexus_props) = match crate::config::resolve_db(config, &nexus_key) {
        Some(crate::config::ResolvedDb::Reservoir(_, db)) => {
            (db.ds_id().to_string(), db.name.clone(), db.properties.clone())
        }
        _ => return Err("Nexus database not found".into()),
    };

    let title = format!("{}: {}", event_type, details);
    let mut properties = serde_json::Map::new();

    if let Some(title_prop) = nexus_props.get("title") {
        properties.insert(title_prop.clone(), serde_json::json!({
            "title": [{ "text": { "content": title } }]
        }));
    }
    if let Some(cat_prop) = nexus_props.get("category") {
        properties.insert(cat_prop.clone(), serde_json::json!({
            "select": { "name": category }
        }));
    }
    if let Some(lt_prop) = nexus_props.get("log_type") {
        properties.insert(lt_prop.clone(), serde_json::json!({
            "select": { "name": event_type }
        }));
    }
    if let Some(status_prop) = nexus_props.get("status") {
        properties.insert(status_prop.clone(), serde_json::json!({
            "status": { "name": "Processed" }
        }));
    }

    let create_body = serde_json::json!({
        "parent": { "data_source_id": nexus_ds_id },
        "properties": properties,
    });

    let nexus_page = notion.create_page(&create_body).await?;

    Ok(serde_json::json!({
        "nexus_entry_id": nexus_page.id,
        "nexus_entry_title": title,
        "event_type": event_type,
        "category": category,
        "details": details,
    }))
}

/// Create a Nexus log entry documenting a transmutation event.
pub async fn create_nexus_log(
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    transmutation_type: &str,
    source_entry_id: &str,
    target_entry_id: &str,
    source_reservoir: &str,
    target_reservoir: &str,
) -> Result<serde_json::Value, String> {
    let nexus_key = config.reservoir_by_archetype("nexus")
        .map(|(k, _)| k.to_string())
        .unwrap_or_else(|| "nexus".into());

    let (nexus_ds_id, _, nexus_props) = match crate::config::resolve_db(config, &nexus_key) {
        Some(crate::config::ResolvedDb::Reservoir(_, db)) => {
            (db.ds_id().to_string(), db.name.clone(), db.properties.clone())
        }
        _ => return Err("Nexus database not found".into()),
    };

    // Build the title
    let title = format!("{}: {} → {}", transmutation_type, source_reservoir, target_reservoir);

    // Build properties
    let mut properties = serde_json::Map::new();

    // Title
    if let Some(title_prop) = nexus_props.get("title") {
        properties.insert(title_prop.clone(), serde_json::json!({
            "title": [{ "text": { "content": title } }]
        }));
    }

    // Category: "Transmutation"
    if let Some(cat_prop) = nexus_props.get("category") {
        properties.insert(cat_prop.clone(), serde_json::json!({
            "select": { "name": "Transmutation" }
        }));
    }

    // Log Type: the transmutation type
    if let Some(lt_prop) = nexus_props.get("log_type") {
        properties.insert(lt_prop.clone(), serde_json::json!({
            "select": { "name": transmutation_type }
        }));
    }

    // Status: "Processed"
    if let Some(status_prop) = nexus_props.get("status") {
        properties.insert(status_prop.clone(), serde_json::json!({
            "status": { "name": "Processed" }
        }));
    }

    // Create the Nexus page
    let create_body = serde_json::json!({
        "parent": { "data_source_id": nexus_ds_id },
        "properties": properties,
    });

    let nexus_page = notion.create_page(&create_body).await?;

    Ok(serde_json::json!({
        "nexus_entry_id": nexus_page.id,
        "nexus_entry_title": title,
        "transmutation_type": transmutation_type,
        "source": source_entry_id,
        "target": target_entry_id,
    }))
}

/// Query entries from a specific database with optional status filter.
/// `status_prop_name` is the Notion property name for the Status column.
/// `prop_type` is the actual Notion property type ("status" or "select").
pub async fn query_entries_with_status(
    notion: &NotionClient,
    ds_id: &str,
    status_prop_name: &str,
    prop_type: &str,
    status: &str,
    page_size: u32,
) -> Result<Vec<NotionPage>, String> {
    // Build the filter using the correct type key (status vs select)
    let filter_value = match prop_type {
        "select" => serde_json::json!({
            "property": status_prop_name,
            "select": { "equals": status }
        }),
        _ => serde_json::json!({
            "property": status_prop_name,
            "status": { "equals": status }
        }),
    };
    let query = serde_json::json!({
        "page_size": page_size,
        "filter": filter_value,
    });
    let result = notion.query_data_source(ds_id, &query).await?;
    Ok(result.results)
}

/// Query all entries from a database (no filter).
pub async fn query_all_entries(
    notion: &NotionClient,
    ds_id: &str,
    page_size: u32,
) -> Result<Vec<NotionPage>, String> {
    let query = serde_json::json!({ "page_size": page_size });
    let result = notion.query_data_source(ds_id, &query).await?;
    Ok(result.results)
}

/// Update a page's Status property using the config-key-to-Notion mapping.
pub async fn update_entry_status(
    notion: &NotionClient,
    page_id: &str,
    new_status: &str,
    properties: &std::collections::HashMap<String, String>,
) -> Result<(), String> {
    // Resolve the Notion property name from the config "status" key
    let status_prop = properties.get("status")
        .map(|s| s.as_str())
        .unwrap_or("Status");

    // Determine the actual Notion property type (status vs select) from the property name
    // We use status filter format since all LifeOS DBs use status-type properties
    let body = serde_json::json!({
        "properties": {
            status_prop: {
                "status": { "name": new_status }
            }
        }
    });

    notion.update_page(page_id, &body).await?;
    Ok(())
}
