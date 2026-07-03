mod mcp;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use clap::Parser;

use std::sync::Arc;
use lifeos_core::{
    Cli, Commands, PageCommand, load_config, LifeOSConfig, NotionClient,
    resolve_all_data_sources,
    vault::{read_index, write_index},
    sync::{self},
    SchemaCache,
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
        // validate-yaml --self-test only validates local YAML schema files — no Notion API needed
        Commands::ValidateYaml { self_test: true, .. } => {
            let schemas_dir = lifeos_core::util::yaml_schemas::YamlSchemaRegistry::discover_schemas_dir();
            match schemas_dir {
                Some(dir) => {
                    let registry = lifeos_core::util::yaml_schemas::YamlSchemaRegistry::load(&dir);
                    if !registry.load_errors.is_empty() {
                        eprintln!("Schema load errors:");
                        for e in &registry.load_errors {
                            eprintln!("  - {e}");
                        }
                        std::process::exit(1);
                    }
                    let issues = registry.self_test();
                    if issues.is_empty() {
                        println!("✅ All schemas passed self-test.");
                        println!("  Loaded: 1 universal + {} per_db + {} per_entry_type schemas",
                            registry.per_db.len(), registry.per_entry_type.len());
                        if let Some(uni) = &registry.universal {
                            println!("  Universal layer: {} properties, {} validation rules",
                                uni.properties.len(), uni.validation_rules.len());
                        }
                        for db in ["matrix", "potentiator", "nexus", "significator", "greatway"] {
                            if let Some(layer) = registry.per_db.get(db) {
                                let pet_count = registry.per_entry_type.keys().filter(|(d, _)| d == db).count();
                                println!("  per_db/{}.yaml: {} properties, {} entry-types, {} per_entry_type files",
                                    db, layer.properties.len(),
                                    layer_raw_entry_types_count(&dir, db), pet_count);
                            }
                        }
                    } else {
                        eprintln!("❌ {} schema self-test issues:", issues.len());
                        for i in &issues {
                            eprintln!("  - {i}");
                        }
                        std::process::exit(1);
                    }
                }
                None => {
                    eprintln!("Could not discover schemas directory.");
                    eprintln!("Set LIFEOS_SCHEMAS_DIR env var or run from the lifeos-ops repo root.");
                    std::process::exit(1);
                }
            }
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
                Commands::GetPage { page_id, database } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let result = lifeos_core::tools::relations::execute_get_page(
                        &lifeos_core::tools::relations::GetPageParams { page_id, database },
                        &cfg, &notion, &sc,
                    ).await;
                    match result { Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); } }
                }
                Commands::Expand { ids } => {
                    let page_ids: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let result = lifeos_core::tools::relations::execute_expand(
                        &lifeos_core::tools::relations::ExpandParams { page_ids },
                        &cfg, &notion, &sc,
                    ).await;
                    match result { Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); } }
                }
                Commands::Trace { page_id, depth } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let result = lifeos_core::tools::relations::execute_trace(
                        &lifeos_core::tools::relations::TraceParams { page_id, depth: Some(depth) },
                        &cfg, &notion, &sc,
                    ).await;
                    match result { Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); } }
                }
                Commands::Ancestors { page_id, max_levels } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let result = lifeos_core::tools::relations::execute_ancestors(
                        &lifeos_core::tools::relations::AncestorsParams { page_id, max_levels: Some(max_levels) },
                        &cfg, &notion, &sc,
                    ).await;
                    match result { Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); } }
                }
                Commands::Backlinks { page_id, database } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let result = lifeos_core::tools::relations::execute_backlinks(
                        &lifeos_core::tools::relations::BacklinksParams { page_id, database },
                        &cfg, &notion, &sc,
                    ).await;
                    match result { Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); } }
                }
                Commands::Link { source, target, property } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let result = lifeos_core::tools::relations::execute_link(
                        &lifeos_core::tools::relations::LinkParams { source_page: source, target_page: target, property },
                        &cfg, &notion, &sc,
                    ).await;
                    match result { Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); } }
                }
                Commands::GraphMetrics => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let result = lifeos_core::tools::relations::execute_graph_metrics(
                        &cfg, &notion, &sc,
                    ).await;
                    match result { Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); } }
                }
                Commands::Schema { database } => {
                    let (cfg, _notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let result = lifeos_core::tools::execute_get_schema(database.as_deref(), &sc, &cfg);
                    println!("{result}");
                }
                Commands::Query { database, filter_property, filter_value, filter_type, sort_property, sort_direction, limit, preset, entry_type, cycle, archetype, complex, drive, shadow, digestion_stage } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::query::QueryParams {
                        database, filter_property, filter_value, filter_type,
                        sort_property, sort_direction, limit: Some(limit),
                        return_properties: None, preset, entry_type, cycle,
                        archetype, complex, drive, shadow, digestion_stage,
                    };
                    match lifeos_core::tools::query::execute(&params, &cfg, &notion, &sc).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::Mutate { operation, database, page_id, properties, dry_run } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let props: Option<serde_json::Value> = properties.and_then(|p| serde_json::from_str(&p).ok());
                    let _ = dry_run; // CLI flag — tool operates directly
                    let params = lifeos_core::tools::mutate::MutateParams {
                        operation, database, page_id, properties: props, target_name: None,
                    };
                    match lifeos_core::tools::mutate::execute(&params, &cfg, &notion, &sc).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::Intelligence { mode, role, module, range } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::intelligence::IntelligenceParams {
                        mode, role, module, range, overrides: None,
                    };
                    match lifeos_core::tools::intelligence::execute(&params, &cfg, &notion, &sc).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::DataScience { analysis_type, database, database_b, days_back, property, metric_property, entry_type, group_by, period, cycle, correlation_metric } => {
                    let (cfg, notion, _sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::data_science::DataScienceParams {
                        analysis_type, database, database_b, days_back, property, metric_property,
                        entry_type, group_by, period, cycle, correlation_metric,
                    };
                    match lifeos_core::tools::data_science::execute(&params, &cfg, &notion).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::Review { review_type } => {
                    let (cfg, notion, _sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::review::ReviewParams {
                        review_type, date: None, databases: None,
                    };
                    match lifeos_core::tools::review::execute(&params, &cfg, &notion).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::Strategic { analysis_type, project_database, okr_database, campaign_database } => {
                    let (cfg, notion, _sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::strategic::StrategicParams {
                        analysis_type, project_database, okr_database, campaign_database,
                    };
                    match lifeos_core::tools::strategic::execute(&params, &cfg, &notion).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::EnergyFlow { scope, currency, limit } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::energy_flow::EnergyFlowParams { show_metabolism: None,
                        scope, currency, entry_id: None, limit,
                    };
                    match lifeos_core::tools::energy_flow::execute(&params, &cfg, &notion, &sc).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::DriveAssessment { boundary, range } => {
                    let (cfg, notion, _sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::drive_assessment::DriveAssessmentParams { boundary, range };
                    match lifeos_core::tools::drive_assessment::execute(&params, &cfg, &notion).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::HealthMetrics { metric, range } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::health_metrics::HealthMetricsParams { metric, range };
                    match lifeos_core::tools::health_metrics::execute(&params, &cfg, &notion, &sc).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
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
                Commands::Orphans { database, limit } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::audit::OrphansParams {
                        database, limit: Some(limit),
                    };
                    match lifeos_core::tools::audit::execute_orphans(&params, &cfg, &notion, &sc).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::Validate { database, status, limit } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::audit::ValidateParams {
                        database, status, limit: Some(limit),
                    };
                    match lifeos_core::tools::audit::execute_validate(&params, &cfg, &notion, &sc).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::SuggestLinks { source, target, threshold, limit } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::audit::SuggestLinksParams {
                        source, target, threshold, limit: Some(limit),
                    };
                    match lifeos_core::tools::audit::execute_suggest_links(&params, &cfg, &notion, &sc).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::ArchetypeIndex => {
                    let result = lifeos_core::tools::ontology::execute_archetype_index();
                    println!("{result}");
                }
                Commands::DeriveType { page_id } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::ontology::DeriveTypeParams { page_id };
                    match lifeos_core::tools::ontology::execute_derive_type(&params, &cfg, &notion, &sc).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::ValenceSignature { page_id, format } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::ontology::ValenceSignatureParams { page_id, format };
                    match lifeos_core::tools::ontology::execute_valence_signature(&params, &cfg, &notion, &sc).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
                Commands::ValidateYaml { self_test: _, all, database, page_id, limit } => {
                    let (cfg, notion, sc) = resolve_with_schema(None, &notion_token).await;
                    let params = lifeos_core::tools::validate_yaml::ValidateYamlParams {
                        database, page_id, self_test: Some(false), all: Some(all), limit: Some(limit),
                    };
                    match lifeos_core::tools::validate_yaml::execute(&params, &cfg, &notion, &sc).await {
                        Ok(t) => println!("{t}"), Err(e) => { tracing::error!("{e}"); std::process::exit(1); }
                    }
                }
            }
        }
    }
}

/// Helper for the validate-yaml --self-test path: count entry-types declared
/// in a per_db/<db>.yaml file. Delegates to lifeos-core.
fn layer_raw_entry_types_count(schemas_dir: &Path, db: &str) -> usize {
    lifeos_core::util::yaml_schemas::count_declared_entry_types(schemas_dir, db)
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

async fn resolve_config_with_ds(config_path_arg: Option<&str>, token: &str) -> (LifeOSConfig, NotionClient) {
    let mut cfg = resolve_config(config_path_arg);
    let notion = NotionClient::new(cfg.clone(), token.to_string());
    let failures = resolve_all_data_sources(&mut cfg, &notion).await;
    if !failures.is_empty() {
        tracing::warn!("{}/{} databases unresolved", failures.len(), cfg.databases.len());
        for (db_key, _) in &failures {
            tracing::warn!("  unresolved database: {db_key}");
        }
    }
    // Persist auto-discovered config so subsequent runs are instant.
    // Only saves when the embedded fallback was used (no file on disk).
    if config_path().is_none() {
        let save_path = std::path::PathBuf::from("lifeos.config.json");
        if let Err(e) = save_config(&cfg, &save_path) {
            tracing::warn!("Could not save auto-discovered config: {e}");
        } else {
            tracing::info!("Saved auto-discovered config to {}", save_path.display());
        }
    }
    let notion = NotionClient::new(cfg.clone(), token.to_string());
    (cfg, notion)
}

/// Resolve config + Notion client + SchemaCache in one call.
///
/// This is the primary initialization path for any CLI command that needs schema
/// awareness. It:
///   1. Loads the config file (or embedded default).
///   2. Resolves all 5 data_source_ids via Notion Search API (auto-discover by name).
///   3. Initializes SchemaCache — fetches every DB's full schema (properties,
///      entry-type options, relation edges) from Notion in parallel.
///   4. Propagates the auto-discovered property map back into the config so
///      downstream code using `DbConfig::notion_prop()` works without needing
///      direct SchemaCache access.
async fn resolve_with_schema(config_path: Option<&str>, token: &str) -> (Arc<LifeOSConfig>, Arc<NotionClient>, Arc<SchemaCache>) {
    let (mut cfg, notion) = resolve_config_with_ds(config_path, token).await;
    let notion = Arc::new(notion);
    let sc = SchemaCache::init(&Arc::new(cfg.clone()), &notion).await;
    // Propagate auto-discovered properties back into config for downstream code
    sc.propagate_to_config(&mut cfg);
    let cfg = Arc::new(cfg);
    (cfg, notion, Arc::new(sc))
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

    // In v5, all_db_keys = just the 5 reservoir keys (no satellites)
    let all_db_keys: Vec<String> = if let Some(filter) = db_filter {
        filter
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    } else {
        config.databases.keys().cloned().collect()
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
    // In v5, all_db_keys = just the 5 reservoir keys (no satellites)
    let all_db_keys: Vec<String> = if let Some(filter) = db_filter {
        filter
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    } else {
        config.databases.keys().cloned().collect()
    };

    let db_keys: Vec<&String> = all_db_keys.iter().collect();

    let index = read_index(vault_dir)?;
    let mut global_report = sync::push::PushReport {

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

    // Build a case-insensitive name → (id, title) lookup so that "Matrix"
    // matches "matrix", "MATRIX", etc. This matches the behavior of the
    // auto-discover path in resolve_all_data_sources.
    let name_map: std::collections::HashMap<String, (String, String)> = notion_dbs
        .iter()
        .map(|(id, title)| (title.to_lowercase(), (id.clone(), title.clone())))
        .collect();

    // Discover the 5 unified databases
    for db_config in cfg.databases.values_mut() {
        let lookup_key = db_config.name.to_lowercase();
        if let Some((id, _)) = name_map.get(&lookup_key) {
            let old_id = std::mem::replace(&mut db_config.database_id, id.clone());
            if old_id != db_config.database_id {
                println!("  [UPDATED] {} : {} -> {}", db_config.name, &old_id[..8.min(old_id.len())], &db_config.database_id[..8.min(db_config.database_id.len())]);
            }
            updated += 1;
        } else {
            not_found.push(db_config.name.clone());
        }
    }

    // Resolve data source IDs (validates each ID via /v1/data_sources/{id})
    let notion = NotionClient::new(cfg.clone(), token.to_string());
    let failures = resolve_all_data_sources(&mut cfg, &notion).await;
    if !failures.is_empty() {
        tracing::warn!("{}/{} databases unresolved", failures.len(), cfg.databases.len());
        for (db_key, _) in &failures {
            tracing::warn!("  unresolved database: {db_key}");
        }
    }

    // ── Full schema sync ──
    // In v0.7+, `discover` also fetches the live schema for each DB and
    // propagates auto-discovered properties + entry-type options back into
    // the config. This means the config file stays in sync with Notion
    // without manual edits.
    println!("\nFetching full schemas from Notion...");
    let cfg_arc = Arc::new(cfg.clone());
    let notion_arc = Arc::new(notion.clone());
    let sc = SchemaCache::init(&cfg_arc, &notion_arc).await;
    sc.propagate_to_config(&mut cfg);
    let props_synced: usize = cfg.databases.values()
        .map(|db| db.discovered_properties.len())
        .sum();
    println!("  Synced {} property mappings across {} databases", props_synced, cfg.databases.len());

    // Report entry-type options discovered per DB
    for (db_key, db_cfg) in &cfg.databases {
        if db_cfg.entry_type_property.is_some() {
            let opts = sc.get_entry_type_options(db_key, &cfg);
            if !opts.is_empty() {
                println!("  {} entry types ({}): {}", db_cfg.name, opts.len(), opts.join(", "));
            }
        }
    }

    // Report relation edges discovered
    let total_edges: usize = sc.all_relation_edges().values().map(|v| v.len()).sum();
    println!("  Discovered {} relation edges across all databases", total_edges);

    save_config(&cfg, &path).map_err(|e| e.to_string())?;

    println!("\nDiscover complete:");
    println!("  Updated: {} databases", updated);
    println!("  Schemas synced: {} property mappings", props_synced);
    println!("  Relations discovered: {}", total_edges);
    println!("  Not found: {}", not_found.len());
    if !not_found.is_empty() {
        for name in &not_found {
            println!("    - {name}");
        }
    }
    println!("\n  Config saved to: {}", path.display());

    Ok(())
}