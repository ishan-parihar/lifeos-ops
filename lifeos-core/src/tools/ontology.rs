//! Ontology module — HoloOS archetype definitions and typology tools.
//!
//! Grounded in `_THEORY/02_Ontology/` from the HoloOS repo. Provides:
//! - The 22 named archetypes (7 roles × 3 complexes + Choice meta-pivot)
//! - The 8 functional roles (M, P, C, E, S, T, G, Ch)
//! - The 3 complexes (Mind, Body, Spirit)
//! - The 4 drives (Agency, Communion, Eros, Agape)
//! - The 4 shadows (Dark-Addiction, Dark-Allergy, Golden-Addiction, Golden-Allergy)
//! - The 5 holon types (Donor, Acceptor, Sharer, Multivalent, Noble)
//! - The 9 digestion stages
//! - archetype-index command: lists all 22 archetypes with mappings
//! - derive-type command: computes Holon Type from Valence Signature
//! - valence-signature command: outputs per-complex register profile

use serde::Deserialize;
use std::sync::Arc;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

// ── Constants ────────────────────────────────────────────────────────

pub const ARCHETYPE_ROLES: &[&str] = &[
    "Matrix", "Potentiator", "Catalyst", "Experience",
    "Significator", "Transformation", "Great Way", "Choice",
];

pub const COMPLEXES: &[&str] = &["Mind", "Body", "Spirit", "None"];

pub const DRIVES: &[&str] = &["Agency", "Communion", "Eros", "Agape"];

pub const SHADOWS: &[&str] = &[
    "None", "Dark-Addiction", "Dark-Allergy", "Golden-Addiction", "Golden-Allergy",
];

pub const HOLON_TYPES: &[&str] = &[
    "Donor", "Acceptor", "Sharer", "Multivalent", "Noble",
];

pub const DIGESTION_STAGES: &[&str] = &[
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

/// The 22 named archetypes: 7 roles × 3 complexes + Choice meta-pivot.
/// Each entry: (number, name, role, complex, polarity_tendency)
pub const ARCHETYPES: &[(u8, &str, &str, &str, &str)] = &[
    (1,  "Matrix of the Mind",         "Matrix",         "Mind",   "STO"),
    (2,  "Potentiator of the Mind",    "Potentiator",    "Mind",   "STO"),
    (3,  "Catalyst of the Mind",       "Catalyst",       "Mind",   "STO"),
    (4,  "Experience of the Mind",     "Experience",     "Mind",   "STO"),
    (5,  "Significator of the Mind",   "Significator",   "Mind",   "STO"),
    (6,  "Transformation of the Mind", "Transformation", "Mind",   "STO"),
    (7,  "Great Way of the Mind",      "Great Way",      "Mind",   "STO"),
    (8,  "Matrix of the Body",         "Matrix",         "Body",   "STO"),
    (9,  "Potentiator of the Body",    "Potentiator",    "Body",   "STO"),
    (10, "Catalyst of the Body",       "Catalyst",       "Body",   "STO"),
    (11, "Experience of the Body",     "Experience",     "Body",   "STO"),
    (12, "Significator of the Body",   "Significator",   "Body",   "STO"),
    (13, "Transformation of the Body", "Transformation", "Body",   "STO"),
    (14, "Great Way of the Body",      "Great Way",      "Body",   "STO"),
    (15, "Matrix of the Spirit",       "Matrix",         "Spirit", "STO"),
    (16, "Potentiator of the Spirit",  "Potentiator",    "Spirit", "STO"),
    (17, "Catalyst of the Spirit",     "Catalyst",       "Spirit", "STO"),
    (18, "Experience of the Spirit",   "Experience",     "Spirit", "STO"),
    (19, "Significator of the Spirit", "Significator",   "Spirit", "STO"),
    (20, "Transformation of the Spirit","Transformation","Spirit", "STO"),
    (21, "Great Way of the Spirit",    "Great Way",      "Spirit", "STO"),
    (22, "The Choice",                 "Choice",         "None",   "STO or STS"),
];

/// The reservoir each functional role belongs to (for mapping archetypes to DBs).
pub const ROLE_TO_RESERVOIR: &[(&str, &str)] = &[
    ("Matrix",         "matrix"),
    ("Potentiator",    "potentiator"),
    ("Catalyst",       "matrix"),        // Catalyst is ingested BY Matrix
    ("Experience",     "potentiator"),   // Experience is stored IN Potentiator
    ("Significator",   "significator"),
    ("Transformation", "nexus"),         // Transformation is the contact-boundary
    ("Great Way",      "greatway"),
    ("Choice",         "greatway"),      // Choice is emitted INTO Great Way
];

// ── archetype-index command ─────────────────────────────────────────

pub fn execute_archetype_index() -> String {
    let mut output = String::new();
    output.push_str("HoloOS Archetype Index — 22 Named Archetypes\n");
    output.push_str(&"=".repeat(60));
    output.push('\n');
    output.push_str("\n7 functional roles × 3 complexes (Mind/Body/Spirit) + 1 Choice meta-pivot = 22.\n\n");

    output.push_str("Legend:\n");
    output.push_str("  #     — archetype number (1-22)\n");
    output.push_str("  Role  — functional role (M/P/C/E/S/T/G/Ch)\n");
    output.push_str("  Cx    — complex (Mind=UL / Body=UR / Spirit=Emergent)\n");
    output.push_str("  Resv  — which LifeOS reservoir this archetype typically instantiates\n");
    output.push_str("  Pol   — polarity tendency (STO=Service-to-Others / STS=Service-to-Self)\n\n");

    output.push_str(&format!(
        "{:<4} {:<30} {:<15} {:<8} {:<14} {:<10}\n",
        "#", "Archetype Name", "Role", "Complex", "Reservoir", "Polarity"
    ));
    output.push_str(&"-".repeat(81));
    output.push('\n');

    for (num, name, role, complex, polarity) in ARCHETYPES {
        let reservoir = ROLE_TO_RESERVOIR
            .iter()
            .find(|(r, _)| *r == *role)
            .map(|(_, resv)| *resv)
            .unwrap_or("?");
        output.push_str(&format!(
            "{:<4} {:<30} {:<15} {:<8} {:<14} {:<10}\n",
            num, name, role, complex, reservoir, polarity
        ));
    }

    output.push_str("\n\nKey relationships:\n");
    output.push_str("  • Roles 1-7 (Mind)   → interior-model domain (UL)\n");
    output.push_str("  • Roles 8-14 (Body)  → operational-substrate domain (UR)\n");
    output.push_str("  • Roles 15-21 (Spirit) → emergent-channel domain (UL+LL)\n");
    output.push_str("  • Role 22 (Choice)   → meta-pivot, closes the greater cycle\n\n");
    output.push_str("Usage in LifeOS:\n");
    output.push_str("  Tag entries with `Archetype Role` (select) + `Complex` (select).\n");
    output.push_str("  The combination (role + complex) identifies which of the 22 archetypes\n");
    output.push_str("  the entry instantiates. Use `lifeos query <db> --archetype Matrix --complex Body`\n");
    output.push_str("  to find all entries instantiating 'Matrix of the Body' (#8).\n");

    output
}

// ── derive-type command ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DeriveTypeParams {
    /// Significator page ID to derive the type for.
    pub page_id: String,
}

pub fn schema_derive_type() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "page_id": { "type": "string", "description": "Significator page ID to derive Holon Type for" }
        },
        "required": ["page_id"]
    })
}

/// Derive the Holon Type from a Significator entry's Valence Signature.
///
/// Per `_THEORY/02_Ontology/02.3_Holonic_Typology_Derivator.md` and
/// `02.4_Significator_Valence_and_Type.md`:
///
/// - **Donor**: net hyper-ingestion (addiction-side) → STO-leaning, radiative
/// - **Acceptor**: net hypo-ingestion (allergy-side) → STS-leaning, absorptive
/// - **Sharer**: balanced throughput, both currencies flowing → covalent-capable
/// - **Multivalent**: multiple open registers, mixed polarity → adaptable
/// - **Noble**: no deficit at this octave; self-contained → inert/graduated
///
/// The Valence Signature is a YAML rich_text property on Significator entries
/// with the per-complex register profile:
/// ```yaml
/// octave_depth: 3
/// complexes:
///   mind:
///     register: matrix-over      # or matrix-under, potentiator-over, potentiator-under, balanced, closed
///     magnitude: 0.7
///   body:
///     register: balanced
///     magnitude: 0.5
///   spirit:
///     register: closed
///     magnitude: 0.0
/// polarity_disposition: donor    # computed: net sign of all complex magnitudes
/// ```
pub async fn execute_derive_type(
    params: &DeriveTypeParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let page = notion.get_page(&params.page_id).await?;

    // Extract the Valence Signature YAML
    let yaml_str: String = page.properties.iter()
        .find(|(name, _)| name.to_lowercase().contains("valence") && name.to_lowercase().contains("signature"))
        .and_then(|(_, v)| match v {
            crate::notion::types::PropertyValue::RichText { rich_text, .. } => {
                Some(rich_text.iter().filter_map(|rt| rt.plain_text.as_deref()).collect())
            }
            _ => None,
        })
        .unwrap_or_default();

    if yaml_str.trim().is_empty() {
        return Err(format!(
            "Significator entry {} has no Valence Signature. Set the 'Valence Signature' \
rich_text property with per-complex register profile YAML. See `lifeos archetype-index` \
for the 22 archetypes and `lifeos valence-signature --help` for the YAML format.",
            &params.page_id[..8.min(params.page_id.len())]
        ));
    }

    // Parse the YAML
    let valence: serde_yaml::Value = serde_yaml::from_str(&yaml_str)
        .map_err(|e| format!("Failed to parse Valence Signature YAML: {}", e))?;

    // Extract per-complex registers
    let mut registers: Vec<(String, String, f64)> = Vec::new(); // (complex, register, magnitude)

    if let Some(complexes) = valence.get("complexes").and_then(|v| v.as_mapping()) {
        for (complex_key, complex_val) in complexes {
            let complex_name = complex_key.as_str().unwrap_or("unknown").to_string();
            if let Some(complex_map) = complex_val.as_mapping() {
                let register = complex_map.get("register")
                    .and_then(|v| v.as_str())
                    .unwrap_or("closed")
                    .to_string();
                let magnitude = complex_map.get("magnitude")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                registers.push((complex_name, register, magnitude));
            }
        }
    }

    // Compute polarity disposition
    // Positive magnitude = donor (addiction-side, over-full, must shed)
    // Negative magnitude = acceptor (allergy-side, starved, must pull in)
    // But we need to account for the register direction:
    //   matrix-over, potentiator-over → donor (positive)
    //   matrix-under, potentiator-under → acceptor (negative)
    //   balanced → 0
    //   closed → 0 (noble candidate)
    let mut net_polarity: f64 = 0.0;
    let mut open_count = 0;
    let mut closed_count = 0;
    let mut mixed = false;
    let mut has_donor = false;
    let mut has_acceptor = false;

    for (_complex, register, magnitude) in &registers {
        match register.as_str() {
            "matrix-over" | "potentiator-over" => {
                net_polarity += magnitude;
                open_count += 1;
                has_donor = true;
            }
            "matrix-under" | "potentiator-under" => {
                net_polarity -= magnitude;
                open_count += 1;
                has_acceptor = true;
            }
            "balanced" => {
                open_count += 1;
            }
            "closed" => {
                closed_count += 1;
            }
            _ => {}
        }
    }

    if has_donor && has_acceptor {
        mixed = true;
    }

    // Derive Holon Type
    let holon_type = if closed_count == registers.len() && !registers.is_empty() {
        "Noble".to_string()
    } else if mixed {
        "Multivalent".to_string()
    } else if net_polarity > 0.1 {
        "Donor".to_string()
    } else if net_polarity < -0.1 {
        "Acceptor".to_string()
    } else if open_count > 0 {
        "Sharer".to_string()
    } else {
        "Noble".to_string()
    };

    // Build the result
    let title = crate::transform::extract_title(&page);
    let data = serde_json::json!({
        "derive_type": {
            "page_id": params.page_id,
            "title": title,
            "valence_signature_raw": yaml_str,
            "per_complex": registers.iter().map(|(c, r, m)| {
                serde_json::json!({"complex": c, "register": r, "magnitude": m})
            }).collect::<Vec<_>>(),
            "analysis": {
                "open_registers": open_count,
                "closed_registers": closed_count,
                "has_donor": has_donor,
                "has_acceptor": has_acceptor,
                "mixed": mixed,
                "net_polarity": (net_polarity * 100.0).round() / 100.0,
            },
            "derived_holon_type": holon_type,
        }
    });

    Ok(crate::toon_format::encode(&data))
}

// ── valence-signature command ───────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ValenceSignatureParams {
    /// Significator page ID to generate a valence signature template for.
    pub page_id: String,
    /// Optional: output format (template or full)
    pub format: Option<String>,
}

pub fn schema_valence_signature() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "page_id": { "type": "string", "description": "Significator page ID" },
            "format": { "type": "string", "enum": ["template", "full"], "description": "Output format (default: template)" }
        },
        "required": ["page_id"]
    })
}

/// Generate a Valence Signature YAML template for a Significator entry.
/// The user fills in the registers and magnitudes, then runs `derive-type`.
pub async fn execute_valence_signature(
    params: &ValenceSignatureParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let _ = config; // suppress unused warning
    let page = notion.get_page(&params.page_id).await?;
    let title = crate::transform::extract_title(&page);

    // Determine which complex this Significator entry belongs to (from the Complex property)
    let complex = page.properties.iter()
        .find(|(name, _)| *name == "Complex")
        .and_then(|(_, v)| match v {
            crate::notion::types::PropertyValue::Select { select, .. } => {
                select.as_ref().map(|o| o.name.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| "Mind".to_string());

    // Generate the YAML template
    let yaml_template = format!(
        r#"octave_depth: 3
complexes:
  mind:
    register: balanced      # one of: matrix-over, matrix-under, potentiator-over, potentiator-under, balanced, closed
    magnitude: 0.5          # 0.0 to 1.0
  body:
    register: balanced
    magnitude: 0.5
  spirit:
    register: closed        # Spirit is closed by default — opens only when Mind + Body are integrated
    magnitude: 0.0
polarity_disposition: sharer  # auto-computed by `lifeos derive-type`
# This entry's primary complex: {primary_complex}
# Entry title: {title}
"#,
        primary_complex = complex,
        title = title,
    );

    let format = params.format.as_deref().unwrap_or("template");

    let data = if format == "full" {
        serde_json::json!({
            "valence_signature": {
                "page_id": params.page_id,
                "title": title,
                "primary_complex": complex,
                "available_complexes": COMPLEXES,
                "available_registers": [
                    "matrix-over", "matrix-under",
                    "potentiator-over", "potentiator-under",
                    "balanced", "closed"
                ],
                "yaml_template": yaml_template,
                "instructions": "1. Copy the yaml_template into the 'Valence Signature' rich_text property of this entry. 2. Edit the register and magnitude values per complex. 3. Run `lifeos derive-type --page-id <id>` to compute the Holon Type.",
            }
        })
    } else {
        serde_json::json!({
            "valence_signature": {
                "page_id": params.page_id,
                "title": title,
                "primary_complex": complex,
                "yaml_template": yaml_template,
            }
        })
    };

    // Also include the schema_cache info for reference
    let _ = schema_cache; // suppress unused warning

    Ok(crate::toon_format::encode(&data))
}

/// Get the Notion property name for a semantic typing property on a DB.
/// Returns the first property name that matches case-insensitively.
pub fn find_property_name<'a>(
    page: &'a crate::notion::types::NotionPage,
    search: &str,
) -> Option<&'a str> {
    let lower = search.to_lowercase();
    page.properties.keys()
        .find(|k| k.to_lowercase() == lower)
        .map(|s| s.as_str())
}
