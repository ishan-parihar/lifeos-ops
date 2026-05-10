use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub name: String,
    pub data_source_id: String,
    pub agent: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: f64,
    pub cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingTarget {
    pub db: String,
    pub filter: Option<serde_json::Value>,
    pub limit: Option<usize>,
    pub date_filter: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingConfig {
    pub roles: HashMap<String, Vec<BriefingTarget>>,
    pub modules: HashMap<String, Vec<BriefingTarget>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeOSConfig {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: RateLimitConfig,
    pub databases: HashMap<String, DbConfig>,
    #[serde(default)]
    pub briefings: Option<BriefingConfig>,
}

fn default_api_version() -> String {
    "2025-09-03".to_string()
}

fn default_rate_limit() -> RateLimitConfig {
    RateLimitConfig { requests_per_second: 3.0, cache_ttl_seconds: 300 }
}

pub fn load_config() -> Result<LifeOSConfig, ConfigError> {
    let paths = vec![
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

pub fn get_db<'a>(config: &'a LifeOSConfig, key: &str) -> Option<&'a DbConfig> {
    config.databases.get(key)
}

pub fn get_dbs_by_agent<'a>(config: &'a LifeOSConfig, agent: &str) -> Vec<(&'a String, &'a DbConfig)> {
    config.databases.iter().filter(|(_, db)| db.agent == agent).collect()
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