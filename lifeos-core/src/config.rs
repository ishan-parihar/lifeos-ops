use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

// ── Embedded Default Config ─────────────────────────────────────────
const EMBEDDED_CONFIG_JSON: &str = include_str!("../../lifeos.config.default.json");

pub(crate) fn embedded_config() -> LifeOSConfig {
    serde_json::from_str(EMBEDDED_CONFIG_JSON)
        .expect("Embedded lifeos.config.json is invalid — this is a compile-time error")
}

// ── v4.1 Consciousness-Prosthetic Architecture ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolonicConfig {
    pub version: String,
    #[serde(default)]
    pub architecture: Option<String>,
    #[serde(default)]
    pub layers: Option<serde_json::Value>,
    #[serde(default)]
    pub flows: Option<serde_json::Value>,
    #[serde(default)]
    pub cycle: Option<String>,
    #[serde(default)]
    pub status_progressions: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub yaml_schemas_path: Option<String>,
    // Legacy fields kept as #[serde(default)] for backward compat with old config files
    #[serde(default)]
    pub currencies: Vec<String>,
    #[serde(default)]
    pub drives: Vec<String>,
    #[serde(default)]
    pub cycles: Option<serde_json::Value>,
    #[serde(default)]
    pub transmutation_map: Option<serde_json::Value>,
    #[serde(default)]
    pub nexus_firing: Option<serde_json::Value>,
    #[serde(default)]
    pub drive_effects: Option<serde_json::Value>,
    #[serde(default)]
    pub entry_type_descriptions: HashMap<String, HashMap<String, String>>,
}

// ── Core Database Config ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub name: String,
    #[serde(rename = "data_source_id")]
    pub database_id: String,
    #[serde(default)]
    pub archetype: Option<String>,
    #[serde(default)]
    pub layer: Option<String>,
    #[serde(default)]
    pub entry_type_property: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub currency_in: Option<String>,
    #[serde(default)]
    pub currency_out: Option<String>,
    #[serde(default)]
    pub cycle: Option<String>,
    #[serde(default)]
    pub scale: Option<String>,
    #[serde(default)]
    pub dimension: Option<String>,
    #[serde(default)]
    pub currency_property: Option<String>,
    #[serde(default)]
    pub entry_type_property_type: String,
    #[serde(default)]
    pub properties: HashMap<String, String>,
    #[serde(skip)]
    pub resolved_data_source_id: Option<String>,
    #[serde(skip)]
    pub discovered_properties: HashMap<String, String>,
}

impl DbConfig {
    pub fn ds_id(&self) -> &str {
        self.resolved_data_source_id.as_deref().unwrap_or(&self.database_id)
    }

    pub fn notion_prop(&self, config_key: &str) -> Option<&str> {
        if let Some(n) = self.discovered_properties.get(config_key) {
            return Some(n.as_str());
        }
        self.properties.get(config_key).map(|s| s.as_str())
    }

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
    pub databases: HashMap<String, DbConfig>,
    #[serde(default)]
    pub holonic: Option<HolonicConfig>,
    #[serde(default)]
    pub briefings: Option<BriefingConfig>,
    #[serde(default)]
    pub notion: Option<NotionConfig>,
}

fn default_entry_type_property_type() -> String { "select".to_string() }
fn default_api_version() -> String { "2025-09-03".to_string() }
fn default_rate_limit() -> RateLimitConfig {
    RateLimitConfig { requests_per_second: 3.0, cache_ttl_seconds: 300 }
}

// ── Config Loading / Saving ─────────────────────────────────────────

pub fn config_path() -> Option<PathBuf> {
    let paths = vec![
        std::env::var("LIFEOS_CONFIG").ok().map(PathBuf::from),
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
    tracing::info!("No lifeos.config.json found on disk — using embedded default config");
    Ok(embedded_config())
}

// ── Resolution Helpers ──────────────────────────────────────────────

pub fn get_db<'a>(config: &'a LifeOSConfig, key: &str) -> Option<&'a DbConfig> {
    config.databases.get(key)
}

pub fn resolve_db<'a>(config: &'a LifeOSConfig, key: &str) -> Option<&'a DbConfig> {
    config.databases.get(key)
}

impl LifeOSConfig {
    pub fn cycle_reservoirs(&self, _cycle: &str) -> Vec<String> {
        // v4.1: cycles are deprecated — return empty
        Vec::new()
    }

    pub fn all_reservoir_keys(&self) -> Vec<String> {
        self.databases.keys().cloned().collect()
    }

    pub fn reservoir_by_archetype(&self, archetype: &str) -> Option<(&str, &DbConfig)> {
        self.databases.iter()
            .find(|(_, db)| db.archetype.as_deref() == Some(archetype))
            .map(|(k, db)| (k.as_str(), db))
    }

    pub fn all_database_keys(&self) -> Vec<String> {
        self.databases.keys().cloned().collect()
    }

    pub fn status_progression(&self, reservoir_key: &str) -> Vec<String> {
        if let Some(ref holonic) = self.holonic {
            if let Some(progression) = holonic.status_progressions.get(reservoir_key) {
                return progression.clone();
            }
        }
        match reservoir_key {
            "trajectory" => vec!["Future".into(), "Ideation".into(), "Paused".into(), "Active".into(), "Done".into(), "Cancelled".into()],
            "synthesis" => vec!["💡 Identified".into(), "✅ Activated".into(), "🏆 Capitalized".into(), "🧊 Archived".into()],
            "profile" => vec!["Draft".into(), "Active".into(), "Evolving".into(), "Archived".into()],
            "context" => vec!["Active".into(), "Inactive".into(), "Archived".into()],
            _ => vec![],
        }
    }

    pub fn entry_type_descriptions(&self, db_key: &str) -> Option<&HashMap<String, String>> {
        self.holonic.as_ref()?.entry_type_descriptions.get(db_key)
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
