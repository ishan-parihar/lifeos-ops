use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

// ── v4 Holonic Architecture ─────────────────────────────────────────

/// A satellite database nested under a core reservoir.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatelliteDbConfig {
    pub name: String,
    #[serde(rename = "data_source_id")]
    pub database_id: String,
    /// Role within the reservoir, e.g. "potentiator_logs", "greatway_commitments"
    #[serde(default)]
    pub role: Option<String>,
    pub properties: HashMap<String, String>,
    /// Resolved at runtime via GET /v1/databases/{database_id} → data_sources[0].id
    #[serde(skip)]
    pub resolved_data_source_id: Option<String>,
}

impl SatelliteDbConfig {
    pub fn ds_id(&self) -> &str {
        self.resolved_data_source_id.as_deref().unwrap_or(&self.database_id)
    }
}

/// Top-level holonic architecture configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolonicConfig {
    pub version: String,
    pub currencies: Vec<String>,
    pub drives: Vec<String>,
    pub cycles: CycleConfig,
    /// Status progressions per reservoir: reservoir_key → ordered status stages
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
    /// The Notion database container ID (what you see in URLs)
    #[serde(rename = "data_source_id")]
    pub database_id: String,
    pub properties: HashMap<String, String>,

    // ── v4 Holonic Metadata ──
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
    /// Nested satellite databases (only for core reservoirs)
    #[serde(default)]
    pub satellites: HashMap<String, SatelliteDbConfig>,

    /// Resolved at runtime via GET /v1/databases/{database_id} → data_sources[0].id
    #[serde(skip)]
    pub resolved_data_source_id: Option<String>,
}

impl DbConfig {
    /// Returns the data_source_id if resolved, otherwise falls back to database_id.
    pub fn ds_id(&self) -> &str {
        self.resolved_data_source_id.as_deref().unwrap_or(&self.database_id)
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
    /// Only the 5 core reservoirs. Satellites are nested inside each.
    pub databases: HashMap<String, DbConfig>,
    #[serde(default)]
    pub holonic: Option<HolonicConfig>,
    #[serde(default)]
    pub briefings: Option<BriefingConfig>,
    #[serde(default)]
    pub notion: Option<NotionConfig>,
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
    Err(ConfigError::NotFound)
}

// ── Resolution Helpers ──────────────────────────────────────────────

/// Get a core reservoir by key.
pub fn get_db<'a>(config: &'a LifeOSConfig, key: &str) -> Option<&'a DbConfig> {
    config.databases.get(key)
}

/// Resolve a database key — checks reservoirs first, then satellites.
/// Returns (reservoir_key, DbConfig or SatelliteDbConfig).
pub enum ResolvedDb<'a> {
    Reservoir(&'a str, &'a DbConfig),
    Satellite(&'a str, &'a str, &'a SatelliteDbConfig), // (reservoir_key, sat_key, sat_config)
}

pub fn resolve_db<'a>(config: &'a LifeOSConfig, key: &str) -> Option<ResolvedDb<'a>> {
    // Direct reservoir match — use iterator key for correct lifetime
    for (k, db) in &config.databases {
        if k == key {
            return Some(ResolvedDb::Reservoir(k, db));
        }
    }
    // Search satellites — iterate to get correct lifetime on sat_key
    for (res_key, res_db) in &config.databases {
        for (sat_key, sat) in &res_db.satellites {
            if sat_key == key {
                return Some(ResolvedDb::Satellite(res_key, sat_key, sat));
            }
        }
    }
    None
}

/// Resolve a database key to (ds_id, name, properties) — works for both reservoirs and satellites.
/// This is the primary accessor for tools that need to query any database by key.
pub fn resolve_db_access<'a>(config: &'a LifeOSConfig, key: &str) -> Option<(&'a str, &'a str, &'a HashMap<String, String>)> {
    match resolve_db(config, key) {
        Some(ResolvedDb::Reservoir(_, db)) => Some((db.ds_id(), &db.name, &db.properties)),
        Some(ResolvedDb::Satellite(_, _, sat)) => Some((sat.ds_id(), &sat.name, &sat.properties)),
        None => None,
    }
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

    /// Get all satellite keys under a given reservoir.
    pub fn satellite_keys(&self, reservoir_key: &str) -> Vec<String> {
        self.databases.get(reservoir_key)
            .map(|db| db.satellites.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all database keys (reservoirs + satellites) as a flat list.
    pub fn all_database_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.databases.keys().cloned().collect();
        for db in self.databases.values() {
            for sat_key in db.satellites.keys() {
                keys.push(sat_key.clone());
            }
        }
        keys
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
            "matrix" => vec!["Raw".into(), "Processing".into(), "Crystallized".into(), "Integrated".into()],
            "potentiator" => vec!["Raw".into(), "Processing".into(), "Crystallized".into(), "Catalytic".into()],
            "significator" => vec!["Latent".into(), "Emerging".into(), "Established".into(), "Directional".into()],
            "greatway" => vec!["Proposed".into(), "Committed".into(), "Active".into(), "Actualized".into()],
            "nexus" => vec!["Observed".into(), "Processed".into(), "Integrated".into()],
            _ => vec![],
        }
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
