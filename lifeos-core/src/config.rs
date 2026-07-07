use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

// ── Embedded Default Config ─────────────────────────────────────────
/// The canonical v5 holonic config, compiled into the binary.
/// Used as fallback when no lifeos.config.json is found on disk,
/// enabling zero-config startup for any Notion workspace with the
/// same LifeOS architecture (database names match, IDs are auto-discovered).
///
/// NOTE: This file is `lifeos.config.default.json` (NOT `lifeos.config.json`)
/// so it does NOT conflict with the user's runtime config (which is
/// gitignored at `lifeos.config.json`). The placeholder `data_source_id`
/// UUIDs are intentionally invalid — `resolve_all_data_sources` will
/// auto-discover real IDs by name on first run via the Notion Search API.
const EMBEDDED_CONFIG_JSON: &str = include_str!("../../lifeos.config.default.json");

/// Parse the embedded default config. Panics if the embedded JSON is invalid
/// (this is a compile-time guarantee — if lifeos.config.json is malformed,
/// the build will fail).
pub(crate) fn embedded_config() -> LifeOSConfig {
    serde_json::from_str(EMBEDDED_CONFIG_JSON)
        .expect("Embedded lifeos.config.json is invalid — this is a compile-time error")
}

// ── v5 Holonic Architecture ─────────────────────────────────────────

/// Top-level holonic architecture configuration.
///
/// NOTE: In v0.7+, the `entry_type_descriptions` field is deprecated — entry types
/// are auto-discovered from Notion at runtime by `SchemaCache::init`. The field is
/// kept (with `#[serde(default)]`) for backward compatibility with older configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolonicConfig {
    pub version: String,
    pub currencies: Vec<String>,
    pub drives: Vec<String>,
    pub cycles: CycleConfig,
    /// Status progressions per reservoir: reservoir_key → ordered status stages.
    /// Used by health_metrics + drive_assessment. Optional — if omitted, falls back
    /// to hardcoded defaults per archetype.
    #[serde(default)]
    pub status_progressions: HashMap<String, Vec<String>>,
    /// Transmutation map: transmutation_type → { source, target }
    #[serde(default)]
    pub transmutation_map: HashMap<String, TransmutationDef>,
    /// Nexus firing thresholds
    #[serde(default)]
    pub nexus_firing: Option<NexusFiringConfig>,
    /// Drive effects per boundary
    #[serde(default)]
    pub drive_effects: HashMap<String, DriveEffectDef>,
    /// v0.9.0: Path to the YAML schemas directory (relative to the config file
    /// or absolute). If omitted, the validator auto-discovers via
    /// `YamlSchemaRegistry::discover_schemas_dir()`.
    #[serde(default)]
    pub yaml_schemas_path: Option<String>,
    /// DEPRECATED in v0.7+. Entry type descriptions per DB. Kept for backward
    /// compatibility — entry types themselves are now auto-discovered from Notion.
    /// Use the `lifeos schema` command to see live entry types per DB.
    #[serde(default)]
    pub entry_type_descriptions: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmutationDef {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusFiringConfig {
    #[serde(default = "default_gz_threshold")]
    pub gz_threshold: f64,
    #[serde(default = "default_pz_threshold")]
    pub pz_threshold: f64,
    #[serde(default = "default_pressure_threshold")]
    pub pressure_threshold: f64,
}

fn default_gz_threshold() -> f64 { 35.0 }
fn default_pz_threshold() -> f64 { 75.0 }
fn default_pressure_threshold() -> f64 { 110.0 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveEffectDef {
    pub lesser: String,
    pub greater: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleConfig {
    pub lesser: CycleDefinition,
    pub greater: CycleDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleDefinition {
    pub reservoirs: Vec<String>,
    pub metric: String,
}

// ── Core Database Config ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub name: String,
    /// The Notion data_source_id — the ID used for `/v1/data_sources/{id}/query`.
    /// Placeholder UUIDs (`00000000-...`) are auto-discovered by `lifeos discover`
    /// via the Notion Search API (matched by `name`).
    #[serde(rename = "data_source_id")]
    pub database_id: String,

    // ── v5 Holonic Metadata (the ONLY required identity fields) ──
    /// Archetype role: "matrix", "potentiator", "significator", "greatway", "nexus"
    #[serde(default)]
    pub archetype: Option<String>,
    /// Scale: "current-stage" or "all-stage"
    #[serde(default)]
    pub scale: Option<String>,
    /// Dimension: "intra-holonic", "extra-holonic", "inter-holonic", or "both"
    #[serde(default)]
    pub dimension: Option<String>,
    /// Primary currency this reservoir ingests
    #[serde(default)]
    pub currency_in: Option<String>,
    /// Primary currency this reservoir produces
    #[serde(default)]
    pub currency_out: Option<String>,
    /// Which cycle: "lesser", "greater", or "both"
    #[serde(default)]
    pub cycle: Option<String>,
    /// Human-readable description of this DB's holonic role
    #[serde(default)]
    pub description: Option<String>,
    /// The Notion property name that holds entry-type discrimination
    /// (e.g., "Entry Type" for Potentiator, "Item Type" for GreatWay, "Category" for Nexus).
    /// Required for entry-type filtering; if omitted, `lifeos schema` will report
    /// no entry types even if the property exists in Notion.
    #[serde(default)]
    pub entry_type_property: Option<String>,
    /// DEPRECATED in v0.7+. Auto-discovered from Notion at runtime by SchemaCache.
    /// Kept for backward compatibility — if present and the live Notion schema
    /// disagrees, the Notion schema wins.
    #[serde(default = "default_entry_type_property_type")]
    pub entry_type_property_type: String,
    /// For Nexus only: the Notion property name that tags entries with a currency
    /// (Catalyst/Experience/Transformation/Choice). Used by `energy-flow` to filter
    /// Nexus entries by currency flow.
    #[serde(default)]
    pub currency_property: Option<String>,

    // ── DEPRECATED: legacy static property map ──
    /// Legacy field kept ONLY so old config files continue to parse. In v0.7+ this
    /// is unused — the live Notion schema is auto-discovered at runtime by
    /// `SchemaCache::init` and stored in `discovered_properties`. Always empty in
    /// fresh configs. Direct access to this field is discouraged; use
    /// `DbConfig::notion_prop()` which consults `discovered_properties` first and
    /// falls back to this map only as a legacy safety net.
    #[serde(default)]
    pub properties: HashMap<String, String>,

    // ── Auto-discovered at runtime (NOT in config file) ──
    /// Resolved at runtime via GET /v1/data_sources/{id} (or auto-discovered by name).
    #[serde(skip)]
    pub resolved_data_source_id: Option<String>,
    /// Auto-discovered property map: config_key (snake_case alias) → Notion property name.
    /// Populated by `SchemaCache::init` from the live Notion schema. Empty in config file.
    #[serde(skip)]
    pub discovered_properties: HashMap<String, String>,
}

impl DbConfig {
    /// Returns the data_source_id if resolved, otherwise falls back to database_id.
    pub fn ds_id(&self) -> &str {
        self.resolved_data_source_id.as_deref().unwrap_or(&self.database_id)
    }

    /// Get the Notion property name for a config key. Prefers auto-discovered
    /// properties (which are always authoritative since they come from Notion);
    /// falls back to the legacy static `properties` map (for pre-v0.7 configs).
    ///
    /// Auto-discovery keys use snake_case aliases of the actual Notion names:
    ///   - `name` → "Name" (title)
    ///   - `entry_type` → entry_type_property (e.g. "Entry Type")
    ///   - `status` → "Status" / "Digestion Status" (per DB)
    ///   - `<snake_case>` → matching Notion property name
    pub fn notion_prop(&self, config_key: &str) -> Option<&str> {
        if let Some(n) = self.discovered_properties.get(config_key) {
            return Some(n.as_str());
        }
        self.properties.get(config_key).map(|s| s.as_str())
    }

    /// Get the entry_type_property Notion name, with fallback.
    pub fn entry_type_notion_name(&self) -> Option<&str> {
        self.entry_type_property.as_deref()
            .or_else(|| self.discovered_properties.get("entry_type").map(|s| s.as_str()))
    }
}

// ── Briefing Config ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingFilters {
    #[serde(rename = "static")]
    pub default_filter: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingTarget {
    pub db: String,
    pub intent: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub filter: Option<serde_json::Value>,
    #[serde(default)]
    pub filters: Option<BriefingFilters>,
    pub limit: Option<usize>,
    pub date_filter: Option<bool>,
    #[serde(default)]
    pub sort: Option<serde_json::Value>,
}

impl BriefingTarget {
    pub fn effective_filter(&self) -> Option<&serde_json::Value> {
        self.filters.as_ref()
            .and_then(|f| f.default_filter.as_ref())
            .or(self.filter.as_ref())
    }

    pub fn filter_description(&self) -> Option<&str> {
        self.filters.as_ref()
            .and_then(|f| f.description.as_deref())
            .or(self.description.as_deref())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingConfig {
    pub roles: HashMap<String, Vec<BriefingTarget>>,
    pub modules: HashMap<String, Vec<BriefingTarget>>,
}

// ── Top-Level Config ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionConfig {
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitConfig {
    pub requests_per_second: f64,
    pub cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifeOSConfig {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: RateLimitConfig,
    /// The 5 core reservoirs — the unified LifeOS databases.
    /// Entry types are discriminated by select/multi_select properties within each DB.
    pub databases: HashMap<String, DbConfig>,
    #[serde(default)]
    pub holonic: Option<HolonicConfig>,
    #[serde(default)]
    pub briefings: Option<BriefingConfig>,
    #[serde(default)]
    pub notion: Option<NotionConfig>,
}

fn default_entry_type_property_type() -> String {
    "select".to_string()
}

fn default_api_version() -> String {
    "2025-09-03".to_string()
}

fn default_rate_limit() -> RateLimitConfig {
    RateLimitConfig { requests_per_second: 3.0, cache_ttl_seconds: 300 }
}

// ── Config Loading / Saving ─────────────────────────────────────────

pub fn config_path() -> Option<PathBuf> {
    let paths = vec![
        std::env::var("LIFEOS_CONFIG").ok().map(PathBuf::from),
        std::env::var("LIFEOs_CONFIG").ok().map(PathBuf::from),
        Some(PathBuf::from("lifeos.config.json")),
        Some(PathBuf::from("../lifeos.config.json")),
    ];
    paths.into_iter().flatten().find(|p| p.exists())
}

pub fn save_config(config: &LifeOSConfig, path: &PathBuf) -> Result<(), ConfigError> {
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| ConfigError::Parse(path.clone(), e))?;
    std::fs::write(path, json)
        .map_err(|e| ConfigError::Io(path.clone(), e))?;
    tracing::info!("Config saved to {}", path.display());
    Ok(())
}

pub fn load_config() -> Result<LifeOSConfig, ConfigError> {
    let paths = vec![
        std::env::var("LIFEOS_CONFIG").ok().map(PathBuf::from),
        std::env::var("LIFEOs_CONFIG").ok().map(PathBuf::from),
        Some(PathBuf::from("lifeos.config.json")),
        Some(PathBuf::from("../lifeos.config.json")),
    ];
    for path in paths.into_iter().flatten() {
        if path.exists() {
            let raw = std::fs::read_to_string(&path).map_err(|e| ConfigError::Io(path.clone(), e))?;
            let config: LifeOSConfig = serde_json::from_str(&raw)
                .map_err(|e| ConfigError::Parse(path.clone(), e))?;
            tracing::info!("Loaded config from {}", path.display());
            return Ok(config);
        }
    }
    // Fallback: use the compiled-in default config.
    // Database IDs in this config may not match the user's Notion workspace,
    // but resolve_all_data_sources will auto-discover them by name.
    tracing::info!("No lifeos.config.json found on disk — using embedded default config");
    Ok(embedded_config())
}

// ── Resolution Helpers ──────────────────────────────────────────────

/// Get a core reservoir by key.
pub fn get_db<'a>(config: &'a LifeOSConfig, key: &str) -> Option<&'a DbConfig> {
    config.databases.get(key)
}

/// Resolve a database key to its DbConfig.
/// In v5, there are only 5 databases — no satellites.
pub fn resolve_db<'a>(config: &'a LifeOSConfig, key: &str) -> Option<&'a DbConfig> {
    config.databases.get(key)
}



impl LifeOSConfig {
    /// Get reservoir keys for a given cycle ("lesser" or "greater") from holonic config.
    /// Falls back to iterating databases if holonic config is missing.
    pub fn cycle_reservoirs(&self, cycle: &str) -> Vec<String> {
        if let Some(ref holonic) = self.holonic {
            let cycle_def = match cycle {
                "lesser" => Some(&holonic.cycles.lesser),
                "greater" => Some(&holonic.cycles.greater),
                _ => None,
            };
            if let Some(cdef) = cycle_def {
                return cdef.reservoirs.clone();
            }
        }
        // Fallback: iterate config.databases, filter by cycle attribute
        self.databases.iter()
            .filter(|(_, db)| db.cycle.as_deref() == Some(cycle))
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Get all reservoir keys from config (top-level database keys).
    pub fn all_reservoir_keys(&self) -> Vec<String> {
        self.databases.keys().cloned().collect()
    }

    /// Find a reservoir by archetype.
    pub fn reservoir_by_archetype(&self, archetype: &str) -> Option<(&str, &DbConfig)> {
        self.databases.iter()
            .find(|(_, db)| db.archetype.as_deref() == Some(archetype))
            .map(|(k, db)| (k.as_str(), db))
    }

    /// Get all database keys (just the 5 reservoirs in v5).
    pub fn all_database_keys(&self) -> Vec<String> {
        self.databases.keys().cloned().collect()
    }

    /// Get the status progression for a reservoir from holonic config.
    /// Falls back to default progressions if not configured.
    pub fn status_progression(&self, reservoir_key: &str) -> Vec<String> {
        if let Some(ref holonic) = self.holonic {
            if let Some(progression) = holonic.status_progressions.get(reservoir_key) {
                return progression.clone();
            }
        }
        // Default progressions per archetype
        match reservoir_key {
            "matrix" => vec!["Active".into(), "Evolving".into(), "Archived".into()],
            "potentiator" => vec!["Raw".into(), "Digesting".into(), "Crystallized".into()],
            "significator" => vec!["Draft".into(), "Active".into(), "Evolving".into(), "Archived".into()],
            "greatway" => vec!["Future".into(), "Ideation".into(), "Paused".into(), "Active".into(), "Done".into(), "Cancelled".into()],
            "nexus" => vec!["💡 Identified".into(), "✅ Activated".into(), "🏆 Capitalized".into(), "🧊 Archived".into()],
            _ => vec![],
        }
    }

    /// Get the entry type descriptions for a database.
    pub fn entry_type_descriptions(&self, db_key: &str) -> Option<&HashMap<String, String>> {
        self.holonic.as_ref()?.entry_type_descriptions.get(db_key)
    }

    /// Get the transmutation source/target pair for a transmutation type.
    pub fn transmutation_def(&self, transmutation_type: &str) -> Option<&TransmutationDef> {
        self.holonic.as_ref()?.transmutation_map.get(transmutation_type)
    }

    /// Get the nexus firing config.
    pub fn nexus_firing_config(&self) -> NexusFiringConfig {
        self.holonic.as_ref()
            .and_then(|h| h.nexus_firing.clone())
            .unwrap_or_else(|| NexusFiringConfig {
                gz_threshold: 35.0,
                pz_threshold: 75.0,
                pressure_threshold: 110.0,
            })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    NotFound,
    Io(PathBuf, std::io::Error),
    Parse(PathBuf, serde_json::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NotFound => write!(f, "lifeos.config.json not found"),
            ConfigError::Io(path, e) => write!(f, "IO error reading {}: {}", path.display(), e),
            ConfigError::Parse(path, e) => write!(f, "Parse error in {}: {}", path.display(), e),
        }
    }
}
