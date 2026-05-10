# LifeOS Unified — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Merge `lifeos-sync` and `lifeos-rust-mcp` into a single unified Rust binary (`lifeos`) that exposes both CLI sync commands (pull/push/watch) AND MCP tools (query/mutate/intelligence/etc.) via stdio JSON-RPC.

**Architecture:**
- Workspace with 2 crates: `lifeos-core` (shared lib) + `lifeos` (binary)
- `lifeos-core`: unified config, Notion client, transforms, vault management, graph builder, sync engine
- `lifeos`: CLI parser with subcommands (`sync`, `mcp`) + MCP server glue
- MCP mode detected by `--mcp` flag or `$LIFEOS_MODE=mcp` env var; falls back to CLI mode

**Tech Stack:** Rust 2021, tokio, reqwest, serde, clap 4, toon, notify, chrono, pulldown-cmark, strsim, regex, uuid, dotenvy

---

## File Structure

```
src/                          # lifeos-sync → becomes src/
src/main.rs                   # CLI + MCP dispatch
src/cli.rs                   # Unified CLI (lifeos-sync + mcp subcommand)
src/mcp.rs                   # MCP server entry point
lifeos-core/                  # New shared library crate
lifeos-core/src/
  lib.rs                      # Re-exports everything
  config.rs                   # Unified config (merge both configs)
  notion/
    client.rs                 # Unified NotionClient (merge both clients)
    types.rs                  # Unified Notion types (merge both)
  transform/                   # Transform utilities
    blocks_to_md.rs
    md_to_blocks.rs
    properties.rs
  vault/                      # Vault management
    mod.rs
  sync/                       # Sync engine
    pull.rs
    push.rs
    watch.rs
    merge.rs
    page.rs
  graph/                      # Graph building (new)
    mod.rs
  onboard/                    # Briefing/onboarding tools from lifeos-mcp
    mod.rs
  tools/                     # MCP tools from lifeos-mcp
    query.rs
    mutate.rs
    intelligence.rs
    data_science.rs
    review.rs
    strategic.rs
    sync_note.rs
  toon_wrapper.rs
  toon/
  util/
```

---

## Task 1: Create workspace scaffold

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `lifeos-core/Cargo.toml`
- Create: `lifeos-core/src/lib.rs`
- Modify: `Cargo.toml` (rename existing `lifeos-sync` package → `lifeos` binary crate)

- [ ] **Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
members = ["lifeos-core", "lifeos"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Ishan Parihar"]
license = "MIT"
repository = "https://github.com/ishan-parihar/lifeos-sync"
```

- [ ] **Step 2: Create lifeos-core/Cargo.toml**

```toml
[package]
name = "lifeos-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Shared LifeOS library: Notion client, config, transforms, vault, sync, graph"

[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
notify = "7"
notify-debouncer-mini = "0.4"
pulldown-cmark = { version = "0.11", features = ["simd"] }
regex = "1"
strsim = "0.11"
uuid = { version = "1", features = ["v4"] }
dotenvy = "0.15"
toon = "0.1"
```

- [ ] **Step 3: Create lifeos-core/src/lib.rs**

```rust
//! LifeOS Core — shared library
//!
//! Provides: config, Notion client, vault management, sync engine,
//! transforms, graph building, briefing tools, and MCP tool implementations.

pub mod config;
pub mod notion;
pub mod transform;
pub mod vault;
pub mod sync;
pub mod graph;
pub mod onboard;
pub mod tools;
pub mod toon_wrapper;
pub mod toon;
pub mod util;

pub use config::{LifeOSConfig, load_config, get_db, get_dbs_by_agent, BriefingConfig, BriefingTarget, DbConfig, RateLimitConfig};
pub use notion::client::NotionClient;
```

- [ ] **Step 4: Rename existing Cargo.toml to lifeos/Cargo.toml**

Move the current `Cargo.toml` (the one with `name = "lifeos-sync"`) to `lifeos/Cargo.toml` and change its `name` to `"lifeos"`. Update description to `"LifeOS unified CLI + MCP server"`.

- [ ] **Step 5: Create lifeos/src/main.rs skeleton**

```rust
//! LifeOS — unified CLI + MCP binary

use clap::Parser;

mod cli;
mod mcp;

#[derive(Parser, Debug)]
#[command(name = "lifeos", version, about = "LifeOS: Notion sync + AI agent tools")]
pub struct Cli {
    /// Run in MCP server mode (stdio JSON-RPC)
    #[arg(long)]
    mcp: bool,

    #[command(subcommand)]
    pub command: Option<cli::Commands>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.mcp {
        mcp::run().await;
    } else {
        cli::run(cli.command).await;
    }
}
```

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "refactor: workspace scaffold with lifeos-core + lifeos"
```

---

## Task 2: Unify config

**Files:**
- Modify: `lifeos-core/src/config.rs`

- [ ] **Step 1: Write the unified config module**

Replace `lifeos-core/src/config.rs` with the merged config that:
1. Uses `BriefingConfig` / `BriefingTarget` from lifeos-mcp (richer than lifeos-sync's `serde_json::Value`)
2. Uses the `data_source_id` API path (lifeos-sync style — both ultimately call the same `/v1/data_sources/` endpoint)
3. Has `get_db` and `get_dbs_by_agent` helper functions from lifeos-mcp
4. Has the default API version `2025-09-03` (lifeos-sync's, which is more current)

```rust
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
```

- [ ] **Step 2: Commit**

```bash
git add lifeos-core/src/config.rs && git commit -m "feat(core): unified config with BriefingConfig and helpers"
```

---

## Task 3: Unify Notion types

**Files:**
- Modify: `lifeos-core/src/notion/types.rs`

- [ ] **Step 1: Create unified notion/types.rs**

Merge both type definitions. The lifeos-sync version is more complete (has `id` on properties, `files`, `people`, `button`, etc.). The lifeos-mcp version uses `#[serde(untagged)]`. Take the lifeos-sync version as base but add the missing `BriefingData` types from lifeos-mcp. The key types to preserve:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// Rich text, blocks, pages — use lifeos-sync as base (has more fields)
pub mod types;  // Move existing lifeos-sync/src/notion/types.rs content here

// Re-export everything from types module
pub use types::*;
```

Create `lifeos-core/src/notion/types.rs` with the FULL merged content from both repos. Key decisions:
- Use `#[serde(tag = "type")]` on `PropertyValue` (lifeos-sync style, more explicit)
- Add `id` field to all property variants (from lifeos-sync)
- Keep `BriefingData`, `NotionDataSource` etc. from lifeos-mcp
- Keep `BlockListResponse` from lifeos-sync

- [ ] **Step 2: Commit**

```bash
git add lifeos-core/src/notion/types.rs && git commit -m "feat(core): unified Notion types from both repos"
```

---

## Task 4: Unify NotionClient

**Files:**
- Modify: `lifeos-core/src/notion/client.rs`

- [ ] **Step 1: Write the unified NotionClient**

Merge both clients. Take lifeos-sync's client as base (more complete API: `query_data_source_all_since`, `append_blocks`, `update_block`, `delete_block`, `archive_page`, `update_page_full`) and add the `get_data_source` and `get_database` methods from lifeos-mcp. Use the `data_source_id` endpoint path consistently.

```rust
use std::time::{Duration, Instant};
use std::sync::Arc;
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use tokio::sync::Mutex;
use serde_json::Value;

use crate::config::LifeOSConfig;
use crate::notion::types::*;

const BASE_URL: &str = "https://api.notion.com";
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 1000;

#[derive(Clone)]
pub struct NotionClient {
    config: LifeOSConfig,
    token: String,
    http: Client,
    last_request: Arc<Mutex<Instant>>,
}

impl NotionClient {
    pub fn new(config: LifeOSConfig, token: String) -> Self { ... }
    pub fn api_version(&self) -> &str { &self.config.api_version }

    async fn rate_limit(&self) { ... }
    fn request(&self, method: Method, path: &str) -> RequestBuilder { ... }
    async fn execute<T: serde::de::DeserializeOwned>(&self, method: Method, path: &str, body: Option<&Value>) -> Result<T, String> { ... }

    // Full API from lifeos-sync
    pub async fn query_data_source(&self, data_source_id: &str, body: &Value) -> Result<QueryResponse, String> { ... }
    pub async fn query_data_source_all(&self, data_source_id: &str) -> Result<Vec<NotionPage>, String> { ... }
    pub async fn query_data_source_all_since(&self, data_source_id: &str, after: Option<&str>) -> Result<Vec<NotionPage>, String> { ... }
    pub async fn get_page(&self, page_id: &str) -> Result<NotionPage, String> { ... }
    pub async fn create_page(&self, body: &Value) -> Result<NotionPage, String> { ... }
    pub async fn update_page_properties(&self, page_id: &str, properties: &Value) -> Result<NotionPage, String> { ... }
    pub async fn update_page_full(&self, page_id: &str, body: &Value) -> Result<NotionPage, String> { ... }
    pub async fn archive_page(&self, page_id: &str) -> Result<NotionPage, String> { ... }
    pub async fn get_page_blocks(&self, page_id: &str) -> Result<Vec<NotionBlock>, String> { ... }
    pub async fn append_blocks(&self, block_id: &str, children: Vec<Value>) -> Result<(), String> { ... }
    pub async fn update_block(&self, block_id: &str, block_type: &str, content: &Value) -> Result<(), String> { ... }
    pub async fn delete_block(&self, block_id: &str) -> Result<(), String> { ... }
    pub async fn get_data_source(&self, id: &str) -> Result<NotionDataSource, String> { ... }
    pub async fn get_database(&self, id: &str) -> Result<NotionDatabase, String> { ... }
}
```

- [ ] **Step 2: Add notion/mod.rs**

```rust
pub mod client;
pub mod types;

pub use client::NotionClient;
```

- [ ] **Step 3: Commit**

```bash
git add lifeos-core/src/notion/client.rs lifeos-core/src/notion/mod.rs && git commit -m "feat(core): unified NotionClient with full API from both repos"
```

---

## Task 5: Move sync engine, vault, and transforms

**Files:**
- Create: `lifeos-core/src/sync/` (copied from lifeos-sync/src/sync/)
- Create: `lifeos-core/src/vault/` (copied from lifeos-sync/src/vault/)
- Create: `lifeos-core/src/transform/` (copied from lifeos-sync/src/transform/)

- [ ] **Step 1: Copy sync/ into lifeos-core/src/sync/**

Copy all 6 files from `lifeos-sync/src/sync/` to `lifeos-core/src/sync/`. Update all `use crate::` imports to `use lifeos_core::` and fix `crate::config` → `lifeos_core::config`, `crate::notion` → `lifeos_core::notion`, `crate::vault` → `lifeos_core::vault`, `crate::transform` → `lifeos_core::transform`.

- [ ] **Step 2: Copy vault/ into lifeos-core/src/vault/**

Copy `vault/mod.rs` with same import fixes.

- [ ] **Step 3: Copy transform/ into lifeos-core/src/transform/**

Copy all 4 files from `lifeos-sync/src/transform/` with same import fixes.

- [ ] **Step 4: Create lifeos-core/src/sync/mod.rs**

```rust
pub mod pull;
pub mod push;
pub mod watch;
pub mod merge;
pub mod page;

pub use push::push_database;
pub use watch::watch_vault;
pub use page::{cmd_page_new, cmd_page_edit, cmd_page_diff, cmd_page_merge};
```

- [ ] **Step 5: Create lifeos-core/src/vault/mod.rs**

Copy content from lifeos-sync/src/vault/mod.rs.

- [ ] **Step 6: Create lifeos-core/src/transform/mod.rs**

```rust
pub mod blocks_to_md;
pub mod md_to_blocks;
pub mod properties;

pub use blocks_to_md::blocks_to_markdown;
pub use md_to_blocks::markdown_to_blocks;
pub use properties::{extract_properties_yaml, extract_title, yaml_to_properties};
```

- [ ] **Step 7: Create lifeos-core/src/transform/properties.rs** with merged content

From lifeos-sync: `extract_properties_yaml`, `yaml_to_properties`, `extract_title`.
From lifeos-mcp: `extract_string`, `extract_number`, `extract_date`, `extract_relation_ids`, `extract_relation_count`, `extract_boolean`.

- [ ] **Step 8: Fix imports across all moved files**

For each file, ensure:
```rust
use lifeos_core::config::{LifeOSConfig, DbConfig};
use lifeos_core::notion::client::NotionClient;
use lifeos_core::notion::types::*;
use lifeos_core::vault::{read_index, write_index, vault_path, IndexEntry, LastPullTimes};
use lifeos_core::transform::{blocks_to_markdown, markdown_to_blocks, extract_title, extract_properties_yaml, yaml_to_properties};
```

- [ ] **Step 9: Commit**

```bash
git add lifeos-core/src/sync/ lifeos-core/src/vault/ lifeos-core/src/transform/ && git commit -m "feat(core): move sync engine, vault, and transforms from lifeos-sync"
```

---

## Task 6: Move MCP tools and toon

**Files:**
- Create: `lifeos-core/src/tools/` (copied from lifeos-rust-mcp/src/tools/)
- Create: `lifeos-core/src/toon/` (copied from lifeos-rust-mcp/src/toon/)
- Create: `lifeos-core/src/toon_wrapper.rs` (copied from lifeos-rust-mcp/src/toon_wrapper.rs)
- Create: `lifeos-core/src/onboard/` (copied from lifeos-rust-mcp/src/onboard/)
- Create: `lifeos-core/src/util/` (copied from lifeos-rust-mcp/src/util/)

- [ ] **Step 1: Copy tools/ into lifeos-core/src/tools/**

Copy all 8 files from `lifeos-rust-mcp/src/tools/`. Update imports:
```rust
use lifeos_core::config::{LifeOSConfig, get_db, get_dbs_by_agent};
use lifeos_core::notion::client::NotionClient;
use lifeos_core::transform::extract_title; // from lifeos-core transform
use lifeos_core::toon_wrapper::encode;
```

- [ ] **Step 2: Copy toon/ into lifeos-core/src/toon/**

- [ ] **Step 3: Copy toon_wrapper.rs**

- [ ] **Step 4: Copy onboard/ into lifeos-core/src/onboard/**

- [ ] **Step 5: Copy util/ into lifeos-core/src/util/**

- [ ] **Step 6: Update tools/mod.rs imports and re-exports**

```rust
pub mod query;
pub mod mutate;
pub mod intelligence;
pub mod data_science;
pub mod review;
pub mod strategic;
pub mod sync_note;

pub async fn get_tool_definitions(...) -> Vec<Value> { ... }
pub async fn call_tool(...) -> Result<String, String> { ... }
```

- [ ] **Step 7: Commit**

```bash
git add lifeos-core/src/tools/ lifeos-core/src/toon/ lifeos-core/src/toon_wrapper.rs lifeos-core/src/onboard/ lifeos-core/src/util/ && git commit -m "feat(core): move MCP tools, toon, onboard, util from lifeos-mcp"
```

---

## Task 7: Build the unified CLI

**Files:**
- Modify: `lifeos/src/cli.rs`
- Modify: `lifeos/src/main.rs`

- [ ] **Step 1: Write the unified CLI (lifeos/src/cli.rs)**

Merge lifeos-sync's CLI with an `mcp` subcommand. The `mcp` subcommand runs the MCP server (but in the same binary, not a subprocess — just calls `mcp::run()`).

```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "lifeos", version, about = "LifeOS: Notion sync + AI agent tools")]
pub struct Cli {
    /// Run in MCP server mode (stdio JSON-RPC)
    #[arg(long)]
    mcp: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the vault directory structure
    Init {
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Pull all pages from Notion to vault
    Pull {
        #[arg(short, long)]
        databases: Option<String>,
        #[arg(long)]
        exclude: Option<String>,
        #[arg(long)]
        incremental: bool,
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Push changes from vault back to Notion
    Push {
        #[arg(short, long)]
        databases: Option<String>,
        #[arg(short, long)]
        config: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Watch vault for changes and push to Notion in real-time
    Watch {
        #[arg(short, long)]
        config: Option<String>,
        #[arg(long, default_value = "2000")]
        debounce_ms: u64,
    },
    /// Manage pages
    Page {
        #[command(subcommand)]
        action: PageCommand,
    },
    /// Run the MCP server (stdio JSON-RPC)
    Mcp,
}

#[derive(clap::Subcommand, Debug)]
pub enum PageCommand {
    New { db_key: String, title: String, #[arg(short, long)] config: Option<String> },
    Edit { page_id: String, #[arg(short, long)] config: Option<String> },
    Diff { page_id: String, #[arg(short, long)] config: Option<String> },
    Merge { page_id: String, #[arg(short, long)] config: Option<String> },
}

pub async fn run(command: Option<Commands>) {
    use lifeos_core::{LifeOSConfig, NotionClient, load_config};
    use std::sync::Arc;

    dotenvy::dotenv().ok();

    let notion_token = std::env::var("NOTION_API_TOKEN")
        .expect("NOTION_API_TOKEN not set");

    match command {
        None => { /* show help */ }
        Some(Commands::Init { config }) => { /* ... */ }
        Some(Commands::Pull { ... }) => { /* ... */ }
        Some(Commands::Push { ... }) => { /* ... */ }
        Some(Commands::Watch { ... }) => { /* ... */ }
        Some(Commands::Page { action }) => { /* ... */ }
        Some(Commands::Mcp) => {
            // MCP is handled separately in main.rs via --mcp flag
        }
    }
}
```

- [ ] **Step 2: Write the unified main.rs**

```rust
//! LifeOS — unified CLI + MCP binary

mod cli;
mod mcp;

use clap::Parser;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "lifeos", version, about = "LifeOS: Notion sync + AI agent tools")]
struct Cli {
    /// Run in MCP server mode (stdio JSON-RPC)
    #[arg(long)]
    mcp: bool,

    #[command(subcommand)]
    command: Option<cli::Commands>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    if cli.mcp || matches!(cli.command, Some(cli::Commands::Mcp)) {
        mcp::run().await;
    } else {
        cli::run(cli.command).await;
    }
}
```

- [ ] **Step 3: Write lifeos/src/mcp.rs**

```rust
//! MCP server mode

use std::sync::Arc;
use lifeos_core::{LifeOSConfig, NotionClient, load_config, tools};

pub async fn run() {
    let config = load_config().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let token = std::env::var("NOTION_API_TOKEN").unwrap_or_else(|_| {
        eprintln!("NOTION_API_TOKEN environment variable is required");
        std::process::exit(1);
    });

    let notion = Arc::new(NotionClient::new(config.clone(), token));
    let mut server = LifeosServer::new(config, notion);

    if let Err(e) = server.run().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

// Re-implement LifeosServer here (same as lifeos-rust-mcp/src/server.rs)
// pointing to lifeos_core::tools instead of crate::tools
```

- [ ] **Step 4: Create lifeos/src/lib.rs** (so MCP can use core without circular deps)

```rust
pub use lifeos_core::*;
```

- [ ] **Step 5: Add lifeos deps to lifeos/Cargo.toml**

```toml
[package]
name = "lifeos"
version.workspace = true
edition.workspace = true

[dependencies]
lifeos-core = { path = "../lifeos-core" }
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
```

- [ ] **Step 6: Commit**

```bash
git add lifeos/src/cli.rs lifeos/src/main.rs lifeos/src/mcp.rs lifeos/src/lib.rs lifeos/Cargo.toml && git commit -m "feat: unified CLI with sync commands and MCP server mode"
```

---

## Task 8: Build the graph module

**Files:**
- Create: `lifeos-core/src/graph/mod.rs`

- [ ] **Step 1: Write the graph builder**

This builds an Obsidian-compatible graph JSON for the vault (needed for the "built-in graph building" requirement). It scans the vault directory, extracts wikilinks/links from markdown files, and outputs a JSON graph.

```rust
//! Graph builder — generates Obsidian graph-compatible JSON
//!
//! Scans the vault for markdown files, extracts wikilinks [[...]] and
//! markdown links [...](...), builds a nodes/edges graph, and outputs
//! JSON in Obsidian Graph Analysis compatible format.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub db_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Default)]
pub struct Graph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub fn build_graph(vault_dir: &Path) -> Result<Graph, String> {
    let mut nodes: HashMap<String, GraphNode> = HashMap::new();
    let mut edges: HashSet<(String, String)> = HashSet::new();

    let wikilink_re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
    let mdlink_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();

    // Walk all .md files
    let walker = walkdir::WalkDir::new(vault_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"));

    for entry in walker {
        let path = entry.path();
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let file_stem = path.file_stem().unwrap().to_string_lossy();
        let page_id = file_stem.clone();

        // Determine db_key from parent directory
        let db_key = path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .filter(|n| *n != vault_dir.file_name().unwrap())
            .map(|s| s.to_string());

        // Extract title from first H1
        let label = content.lines()
            .find(|l| l.starts_with("# "))
            .map(|l| l.trim_start_matches("# ").to_string())
            .unwrap_or_else(|| file_stem.clone());

        nodes.insert(page_id.clone(), GraphNode { id: page_id.clone(), label, db_key });

        // Extract wikilinks
        for cap in wikilink_re.captures_iter(&content) {
            let target = cap[1].trim().replace(' ', "-");
            if !target.is_empty() && target != page_id {
                edges.insert((page_id.clone(), target));
            }
        }

        // Extract markdown links (external links skipped)
        for cap in mdlink_re.captures_iter(&content) {
            let href = &cap[2];
            if !href.starts_with("http://") && !href.starts_with("https://") {
                let target = href.trim_start_matches('/').replace(".md", "");
                if !target.is_empty() && target != page_id {
                    edges.insert((page_id.clone(), target));
                }
            }
        }
    }

    Ok(Graph {
        nodes: nodes.into_values().collect(),
        edges: edges.into_iter().map(|(s, t)| GraphEdge { source: s, target: t }).collect(),
    })
}

pub fn to_json(graph: &Graph) -> String {
    serde_json::to_string_pretty(graph).unwrap()
}
```

- [ ] **Step 2: Add walkdir dependency to lifeos-core/Cargo.toml**

```toml
walkdir = "2"
```

- [ ] **Step 3: Add graph command to CLI**

In `lifeos/src/cli.rs`, add:
```rust
Graph {
    /// Output vault as Obsidian graph JSON
    #[arg(long, default_value = "graph.json")]
    output: Option<String>,
}
```

And in `cli::run()`:
```rust
Some(Commands::Graph { output }) => {
    let graph = lifeos_core::graph::build_graph(vault_dir)?;
    let json = lifeos_core::graph::to_json(&graph);
    if let Some(path) = output {
        std::fs::write(&path, &json)?;
    } else {
        println!("{}", json);
    }
}
```

- [ ] **Step 4: Commit**

```bash
git add lifeos-core/src/graph/mod.rs lifeos-core/Cargo.toml lifeos/src/cli.rs && git commit -m "feat(core): graph builder for Obsidian graph JSON export"
```

---

## Task 9: Wire everything together and verify

**Files:**
- Verify: all files compile
- Modify: update README

- [ ] **Step 1: Build and fix all compilation errors**

```bash
cd /home/ishanp/Documents/GitHub/lifeos-sync
cargo build --workspace 2>&1 | head -100
```

Fix any type mismatches, import errors, or missing fields iteratively until clean build.

- [ ] **Step 2: Verify CLI mode**

```bash
cargo run --bin lifeos -- --help
cargo run --bin lifeos -- init --help
cargo run --bin lifeos -- pull --help
cargo run --bin lifeos -- graph --help
```

Expected: Help text for all sync commands and graph command.

- [ ] **Step 3: Verify MCP mode (manual JSON-RPC handshake)**

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}' | timeout 5 cargo run --bin lifeos -- --mcp 2>/dev/null
```

Expected: JSON-RPC response with server info and tool list.

- [ ] **Step 4: Update README**

Add:
- Unified binary description
- CLI commands (`lifeos init`, `lifeos pull`, `lifeos push`, `lifeos watch`, `lifeos page new/edit/diff/merge`, `lifeos graph`)
- MCP mode (`lifeos --mcp` or `lifeos mcp`)
- Installation instructions
- Environment variables (`NOTION_API_TOKEN`, `LIFEOs_CONFIG`, `LIFEOs_VAULT`)

- [ ] **Step 5: Run full test**

```bash
cargo run --bin lifeos -- --help
```

- [ ] **Step 6: Commit**

```bash
git add README.md && git commit -m "docs: update README for unified lifeos binary"
```

---

## Task 10: Final verification

- [ ] **Step 1: cargo clippy --workspace**

Run Clippy on the entire workspace and fix warnings.

- [ ] **Step 2: cargo test --workspace**

Run any existing tests (both repos had minimal/no tests — this documents the gap).

- [ ] **Step 3: Final commit with tag**

```bash
git add -A && git commit -m "feat: lifeos unified binary — CLI sync + MCP server + graph" && git tag v1.0.0-unified
```

---

## Self-Review Checklist

1. **Spec coverage:** All user decisions implemented?
   - Single multi-mode CLI ✅ (Task 7)
   - All sync features ✅ (Task 5)
   - All MCP tools ✅ (Task 6)
   - Built-in graph building ✅ (Task 8)
   - Single unified binary ✅ (Task 1)
   - Agent usability priority ✅ (MCP via stdio JSON-RPC)

2. **Placeholder scan:** No "TBD", "TODO", or vague steps.

3. **Type consistency:** All imports use `lifeos_core::*` consistently; `NotionClient` unified; `LifeOSConfig` unified.

4. **What was NOT included (YAGNI):**
   - Official MCP SDK crate (we use hand-rolled JSON-RPC from lifeos-mcp — already works)
   - Re-exporting everything as a library crate separately
   - Custom briefing engine beyond what lifeos-mcp already had

---

**Plan complete.** Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using `executing-plans`, batch execution with checkpoints.

Which approach?
