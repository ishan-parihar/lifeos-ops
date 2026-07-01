mod mcp;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use clap::Parser;

use lifeos_core::{
    Cli, Commands, PageCommand, load_config, LifeOSConfig, NotionClient,
    resolve_all_data_sources,
    vault::{read_index, write_index},
    sync::{self},
};
use lifeos_core::config::{config_path, save_config};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    dotenvy::dotenv().ok();

    match cli.command {
        // MCP has its own token/config resolution (fallback to config.notion.api_key)
        Commands::MCP => {
            mcp::run_server().await;
        }
        command => {
            let notion_token = match std::env::var("NOTION_API_TOKEN") {
                Ok(t) => t,
                Err(_) => {
                    tracing::error!("NOTION_API_TOKEN not set in environment or .env file");
                    std::process::exit(1);
                }
            };

            match command {
                Commands::Init { config: config_path } => {
                    let cfg = resolve_config(config_path.as_deref());
                    let vault_dir = resolve_vault_dir();
                    let notion = NotionClient::new(cfg, notion_token);
                    if let Err(e) = cmd_init(&notion, &vault_dir).await {
                        tracing::error!("Init failed: {e}");
                        std::process::exit(1);
                    }
                }
                Commands::Pull {
                    databases,
                    exclude,
                    incremental,
                    config: config_path,
                } => {
                    let (cfg, notion) = resolve_config_with_ds(config_path.as_deref(), &notion_token).await;
                    let vault_dir = resolve_vault_dir();
                    if let Err(e) = cmd_pull(&notion, &cfg, &vault_dir, databases.as_deref(), exclude.as_deref(), incremental).await {
                        tracing::error!("Pull failed: {e}");
                        std::process::exit(1);
                    }
                }
                Commands::Push {
                    databases,
                    config: config_path,
                    dry_run,
                } => {
                    let (cfg, notion) = resolve_config_with_ds(config_path.as_deref(), &notion_token).await;
                    let vault_dir = resolve_vault_dir();
                    if let Err(e) = cmd_push(&notion, &cfg, &vault_dir, databases.as_deref(), dry_run).await {
                        tracing::error!("Push failed: {e}");
                        std::process::exit(1);
                    }
                }
                Commands::Watch {
                    config: config_path,
                    debounce_ms,
                } => {
                    let (cfg, notion) = resolve_config_with_ds(config_path.as_deref(), &notion_token).await;
                    let vault_dir = resolve_vault_dir();
                    if let Err(e) = cmd_watch(&notion, &cfg, &vault_dir, debounce_ms).await {
                        tracing::error!("Watch failed: {e}");
                        std::process::exit(1);
                    }
                }
                Commands::Page { action } => {
                    let (cfg, notion) = resolve_config_with_ds(None, &notion_token).await;
                    let vault_dir = resolve_vault_dir();
                    match action {
                        PageCommand::New { db_key, title, config: _ } => {
                            if let Err(e) = sync::cmd_page_new(&notion, &cfg, &vault_dir, &db_key, &title).await {
                                tracing::error!("Page new failed: {e}");
                                std::process::exit(1);
                            }
                        }
                        PageCommand::Edit { page_id, config: _ } => {
                            if let Err(e) = sync::cmd_page_edit(&notion, &cfg, &vault_dir, &page_id).await {
                                tracing::error!("Page edit failed: {e}");
                                std::process::exit(1);
                            }
                        }
                        PageCommand::Diff { page_id, config: _ } => {
                            if let Err(e) = sync::cmd_page_diff(&notion, &cfg, &vault_dir, &page_id).await {
                                tracing::error!("Page diff failed: {e}");
                                std::process::exit(1);
                            }
                        }
                        PageCommand::Merge { page_id, config: _ } => {
                            if let Err(e) = sync::cmd_page_merge(&notion, &cfg, &vault_dir, &page_id).await {
                                tracing::error!("Page merge failed: {e}");
                                std::process::exit(1);
                            }
                        }
                    }
                }
                Commands::MCP => unreachable!(),
                Commands::Discover { config: config_path_arg } => {
                    let cfg = resolve_config(config_path_arg.as_deref());
                    if let Err(e) = cmd_discover(cfg, &notion_token).await {
                        tracing::error!("Discover failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

fn resolve_config(config_path: Option<&str>) -> LifeOSConfig {
    if let Some(path) = config_path {
        std::env::set_var("LIFEOS_CONFIG", path);
    }
    match load_config() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("{}", e);
            std::process::exit(1);
        }
    }
}

async fn resolve_config_with_ds(config_path: Option<&str>, token: &str) -> (LifeOSConfig, NotionClient) {
    let mut cfg = resolve_config(config_path);
    let notion = NotionClient::new(cfg.clone(), token.to_string());
    let failures = resolve_all_data_sources(&mut cfg, &notion).await;
    if !failures.is_empty() {
        tracing::warn!("{}/{} databases unresolved", failures.len(), cfg.databases.len());
        for (db_key, _) in &failures {
            tracing::warn!("  unresolved database: {db_key}");
        }
    }
    let notion = NotionClient::new(cfg.clone(), token.to_string());
    (cfg, notion)
}

fn resolve_vault_dir() -> PathBuf {
    let dir = std::env::var("LIFEOs_VAULT")
        .unwrap_or_else(|_| "./vault".to_string());
    PathBuf::from(dir)
}

async fn cmd_init(_notion: &NotionClient, vault_dir: &Path) -> Result<(), String> {
    tracing::info!("Initializing vault at {}", vault_dir.display());
    std::fs::create_dir_all(vault_dir)
        .map_err(|e| format!("Create vault dir: {e}"))?;

    let empty_index = HashMap::new();
    write_index(vault_dir, &empty_index)?;

    tracing::info!(
        "Vault initialized at {}. Run `lifeos pull` to sync all databases.",
        vault_dir.display()
    );
    Ok(())
}

async fn cmd_pull(
    notion: &NotionClient,
    config: &LifeOSConfig,
    vault_dir: &Path,
    db_filter: Option<&str>,
    exclude: Option<&str>,
    incremental: bool,
) -> Result<(), String> {
    let exclude_set: std::collections::HashSet<String> = exclude
        .map(|s| s.split(',').map(|k| k.trim().to_string()).collect())
        .unwrap_or_default();

    // Expand to include satellites alongside reservoirs
    let all_db_keys: Vec<String> = if let Some(filter) = db_filter {
        filter
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    } else {
        let mut keys = Vec::new();
        for (key, db) in &config.databases {
            keys.push(key.clone());
            for sat_key in db.satellites.keys() {
                keys.push(sat_key.clone());
            }
        }
        keys
    };

    let db_keys: Vec<&String> = all_db_keys
        .iter()
        .filter(|k| !exclude_set.contains(k.as_str()))
        .collect();

    let mut index = read_index(vault_dir)?;
    let mut last_pull = if incremental {
        let t = lifeos_core::vault::read_last_pull_times(vault_dir)?;
        Some(t)
    } else {
        None
    };
    let mut global_report = sync::pull::PullReport {
        db_key: "ALL".to_string(),
        pages_processed: 0,
        files_created: 0,
        files_updated: 0,
        errors: vec![],
    };

    for db_key in &db_keys {
        let since = last_pull
            .as_ref()
            .and_then(|lp| lp.per_db.get(*db_key).map(|s| s.as_str()));
        match sync::pull::pull_database_since(
            notion, config, db_key, vault_dir, &mut index, since,
        )
        .await
        {
            Ok(report) => {
                global_report.pages_processed += report.pages_processed;
                global_report.files_created += report.files_created;
                global_report.files_updated += report.files_updated;
                tracing::info!(
                    "Pulled {}: {} processed, {} created, {} updated",
                    db_key,
                    report.pages_processed,
                    report.files_created,
                    report.files_updated,
                );
            }
            Err(e) => {
                tracing::error!("Pull failed for {db_key}: {e}");
                global_report.errors.push(format!("{db_key}: {e}"));
            }
        }
        if let Some(ref mut lp) = last_pull {
            lp.per_db
                .insert(db_key.to_string(), lifeos_core::vault::utc_now_iso());
        }
    }

    if let Some(lp) = last_pull {
        lifeos_core::vault::write_last_pull_times(vault_dir, &lp)?;
    }

    write_index(vault_dir, &index)?;

    tracing::info!(
        "Pull complete: {} pages, {} created, {} updated, {} errors",
        global_report.pages_processed,
        global_report.files_created,
        global_report.files_updated,
        global_report.errors.len(),
    );

    Ok(())
}

async fn cmd_push(
    notion: &NotionClient,
    config: &LifeOSConfig,
    vault_dir: &Path,
    db_filter: Option<&str>,
    dry_run: bool,
) -> Result<(), String> {
    // Expand to include satellites alongside reservoirs
    let all_db_keys: Vec<String> = if let Some(filter) = db_filter {
        filter
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    } else {
        let mut keys = Vec::new();
        for (key, db) in &config.databases {
            keys.push(key.clone());
            for sat_key in db.satellites.keys() {
                keys.push(sat_key.clone());
            }
        }
        keys
    };

    let db_keys: Vec<&String> = all_db_keys.iter().collect();

    let index = read_index(vault_dir)?;
    let mut global_report = sync::push::PushReport {
        db_key: "ALL".to_string(),
        pages_created: 0,
        pages_updated: 0,
        errors: vec![],
    };

    for db_key in &db_keys {
        match sync::push_database(notion, config, db_key, vault_dir, &index, dry_run).await {
            Ok(report) => {
                global_report.pages_created += report.pages_created;
                global_report.pages_updated += report.pages_updated;
                tracing::info!(
                    "Pushed {}: {} created, {} updated",
                    db_key,
                    report.pages_created,
                    report.pages_updated,
                );
            }
            Err(e) => {
                tracing::error!("Push failed for {db_key}: {e}");
                global_report.errors.push(format!("{db_key}: {e}"));
            }
        }
    }

    tracing::info!(
        "Push complete: {} created, {} updated, {} errors",
        global_report.pages_created,
        global_report.pages_updated,
        global_report.errors.len(),
    );

    Ok(())
}

async fn cmd_watch(
    notion: &NotionClient,
    config: &LifeOSConfig,
    vault_dir: &Path,
    debounce_ms: u64,
) -> Result<(), String> {
    tracing::info!(
        "Watching vault at {} for changes (debounce: {debounce_ms}ms)...",
        vault_dir.display()
    );
    sync::watch_vault(notion, config, vault_dir, debounce_ms).await
}

async fn cmd_discover(mut cfg: LifeOSConfig, token: &str) -> Result<(), String> {
    let path = config_path()
        .ok_or_else(|| "Could not find lifeos.config.json. Run from project root or set LIFEOS_CONFIG.".to_string())?;

    let notion = NotionClient::new(cfg.clone(), token.to_string());

    println!("Scanning Notion for databases...");
    let notion_dbs = notion.search_databases().await?;
    println!("Found {} databases in Notion", notion_dbs.len());

    let mut updated = 0;
    let mut not_found = Vec::new();

    // Discover reservoirs
    for db_config in cfg.databases.values_mut() {
        if let Some((id, _)) = notion_dbs.iter().find(|(_, title)| title == &db_config.name) {
            let old_id = std::mem::replace(&mut db_config.database_id, id.clone());
            if old_id != db_config.database_id {
                println!("  [UPDATED] {} : {} -> {}", db_config.name, &old_id[..8.min(old_id.len())], &db_config.database_id[..8.min(db_config.database_id.len())]);
            }
            updated += 1;
        } else {
            not_found.push(db_config.name.clone());
        }
        // Discover satellites
        for sat in db_config.satellites.values_mut() {
            if let Some((id, _)) = notion_dbs.iter().find(|(_, title)| title == &sat.name) {
                let old_id = std::mem::replace(&mut sat.database_id, id.clone());
                if old_id != sat.database_id {
                    println!("  [UPDATED] {} (satellite) : {} -> {}", sat.name, &old_id[..8.min(old_id.len())], &sat.database_id[..8.min(sat.database_id.len())]);
                }
                updated += 1;
            } else {
                not_found.push(sat.name.clone());
            }
        }
    }

    save_config(&cfg, &path).map_err(|e| e.to_string())?;

    println!("\nDiscover complete:");
    println!("  Updated: {} databases (reservoirs + satellites)", updated);
    println!("  Not found: {}", not_found.len());
    if !not_found.is_empty() {
        for name in &not_found {
            println!("    - {name}");
        }
    }

    Ok(())
}