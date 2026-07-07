//! `quick_link` tool — link two entries by title (auto-resolves page IDs).
//!
//! v0.10.3 (U-1 + parity): Wraps the title-resolution + link logic that was
//! previously inline in main.rs's QuickLink CLI command. Now both CLI and MCP
//! can call the same execute function.
//!
//! Per-relation semantic hints are included in the response (U-1) — each
//! relation name maps to a 1-line ontological meaning so AI agents learn
//! what each relation represents.

use std::sync::Arc;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;
use crate::util::id_resolver;
use crate::tools::relations;

#[derive(Debug, Deserialize)]
pub struct QuickLinkParams {
    pub source_db: String,
    pub source_title: String,
    pub target_db: String,
    pub target_title: String,
    pub property: String,
}

pub fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "source_db": { "type": "string", "description": "Source database key (matrix, potentiator, nexus, significator, greatway)" },
            "source_title": { "type": "string", "description": "Source entry title (fuzzy match)" },
            "target_db": { "type": "string", "description": "Target database key" },
            "target_title": { "type": "string", "description": "Target entry title (fuzzy match)" },
            "property": { "type": "string", "description": "Relation property name on source page (e.g. 'Generated From', 'Sub-holon Of')" }
        },
        "required": ["source_db", "source_title", "target_db", "target_title", "property"]
    })
}

/// Per-relation semantic hints (U-1). Maps relation property names to a
/// 1-line ontological meaning. AI agents see this in the link response and
/// learn what each relation represents without reading the full ontology.
///
/// Public so that `relations::execute_link` can reuse the same hint map
/// (ensures CLI `link` and MCP `link` both get semantic hints).
pub fn relation_semantic_hint(property: &str) -> &'static str {
    match property {
        // Fractal coupling (HoloOS doc 08.5)
        "Sub-holon Of" => "Fractal coupling: this entry is a component of a larger holon (HoloOS doc 08.5)",
        "Anchored In" => "Identity is anchored in this State entry (fractal coupling, S↔M)",
        "Coheres With" | "Coheres With (Significator)" => "Identity coheres with this external holon in World (bonding surface)",
        "For Significator" => "World entry is for this Identity (bonding surface)",
        "Transforms To" | "For" => "Identity transforms to this World entry (greater cycle: T → Ch)",

        // Nexus currency flow (HoloOS doc 03.1 §3)
        "Sends Catalyst To (Matrix)" | "Sends Catalyst To (Significator)" => "Process sends Catalyst to this reservoir (lesser/greater cycle ingestion)",
        "Sends Experience To (Potentiator)" => "Process sends Experience to Possibility (lesser cycle refinement)",
        "Rewrites (Matrix)" | "Rewrites (Potentiator)" => "Process rewrites this reservoir entry (downward causation after Transformation fires)",
        "Updates" => "Process updates this State entry (Catalyst ingestion)",
        "Sourced From" => "Process sourced from this Possibility entry (Experience origin)",
        "Emits Choice To" => "Process emits Choice to this World entry (greater cycle output)",
        "Fires Transformation On" => "Process fires Transformation on this Identity entry (threshold event)",
        "Triggered By" => "Process triggered by this Identity entry (Significator threshold)",

        // Intra-DB hierarchy
        "Parent" | "Parent item" => "Hierarchical parent within the same DB",
        "Sub-item" => "Hierarchical child within the same DB",
        "Blocked By" | "Blocks" => "Dependency: this entry is blocked by / blocks the target",
        "Refines" => "This entry refines the target (intra-DB evolution)",
        "Supersedes" => "This entry supersedes the target (intra-DB versioning)",
        "Accumulates Into" => "This entry accumulates into the target (upward flow)",
        "Generated From" => "This State entry was generated from this Possibility entry (Catalyst origin)",
        "Integrated Into" => "This State entry integrates into this Identity entry (E → S accumulation)",
        "Crystallized To" | "Crystallizes Into" => "This Possibility entry crystallizes into a State pattern (E → C)",
        "Reveals" => "This Possibility entry reveals a State pattern",
        "Harmonized By" => "This Possibility entry is harmonized by a Process entry",
        "Pillar Link" => "This State entry links to an Identity pillar",

        // People / external holons
        "People" | "Related to Potentiator (People)" => "Person from World linked to this Possibility entry",

        // Tension / counter-relations
        "Tension" | "Counter-Tension" | "In Tension With" => "Tension relation: this entry is in tension with the target",
        "Counter-Synthesis" | "Counterpart" | "Reinforces" => "Dialectical relation: counter-synthesis / counterpart / reinforcement",

        // Default
        _ => "(no semantic hint available — see ONTOLOGY.md for relation meaning)",
    }
}

pub async fn execute(
    params: &QuickLinkParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    // Resolve source title → page ID
    let source_result = id_resolver::resolve_target_id(
        notion, config, &params.source_db, Some(&params.source_title), None
    ).await;
    let source_id = source_result.id.ok_or_else(|| {
        format!("Could not find '{}' in {}", params.source_title, params.source_db)
    })?;

    // Resolve target title → page ID
    let target_result = id_resolver::resolve_target_id(
        notion, config, &params.target_db, Some(&params.target_title), None
    ).await;
    let target_id = target_result.id.ok_or_else(|| {
        format!("Could not find '{}' in {}", params.target_title, params.target_db)
    })?;

    // Create the link via the existing relations::execute_link.
    // Note: execute_link already appends a semantic hint (U-1), so we don't
    // add a second one here — just return its result directly.
    let link_params = relations::LinkParams {
        source_page: source_id.clone(),
        target_page: target_id.clone(),
        property: params.property.clone(),
    };
    let link_result = relations::execute_link(&link_params, config, notion, schema_cache).await?;
    let _ = schema_cache; // suppress unused warning
    Ok(link_result)
}
