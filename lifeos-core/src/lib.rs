//! LifeOS Core — unified library
//!
//! Provides: config, notion client, vault management, sync engine,
//! transforms, MCP tools, briefing, data science, strategic analysis,
//! and bidirectional Notion ↔ vault sync operations.

pub mod cli;
pub mod config;
pub mod notion;
pub mod transform;
pub mod vault;
pub mod sync;
pub mod tools;
pub mod util;
pub mod toon_format;
pub mod server;

pub use cli::{Cli, Commands, PageCommand};
pub use config::{LifeOSConfig, load_config, config_path, save_config, get_db, resolve_db, DbConfig, SatelliteDbConfig, HolonicConfig, RateLimitConfig, BriefingConfig, BriefingTarget, ConfigError, ResolvedDb};
pub use notion::client::{NotionClient, resolve_all_data_sources};
pub use server::LifeosServer;
pub use util::schema_engine::SchemaCache;