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
