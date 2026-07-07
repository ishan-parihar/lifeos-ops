use std::sync::Arc;

use lifeos_core::{LifeosServer, load_config, config_path, save_config, NotionClient, resolve_all_data_sources, SchemaCache};

pub async fn run_server() {
    let mut config = match load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let token = std::env::var("NOTION_API_TOKEN").unwrap_or_else(|_| {
        config
            .notion
            .as_ref()
            .and_then(|n| n.api_key.clone())
            .unwrap_or_else(|| {
                eprintln!("NOTION_API_TOKEN environment variable or notion.api_key in config is required");
                std::process::exit(1);
            })
    });

    // Create a temporary client to resolve data source IDs
    let resolver = NotionClient::new(config.clone(), token.clone());
    let failures = resolve_all_data_sources(&mut config, &resolver).await;

    if !failures.is_empty() {
        for (db_key, err) in &failures {
            tracing::error!("  ✗ {db_key}: {err}");
        }
        tracing::warn!(
            "MCP server starting with {}/{} databases unresolved — tools targeting these databases will fail",
            failures.len(), config.databases.len()
        );
    } else {
        tracing::info!("All {} databases resolved successfully", config.databases.len());
    }

    // Persist auto-discovered config so subsequent runs are instant.
    // Only saves when the embedded fallback was used (no file on disk).
    if config_path().is_none() {
        let save_path = std::path::PathBuf::from("lifeos.config.json");
        if let Err(e) = save_config(&config, &save_path) {
            tracing::warn!("Could not save auto-discovered config: {e}");
        } else {
            tracing::info!("Saved auto-discovered config to {}", save_path.display());
        }
    }

    // Create the actual client with resolved config
    let config = Arc::new(config);
    let notion = Arc::new(NotionClient::new((*config).clone(), token));

    tracing::info!("Pre-warming schema cache...");
    let schema_cache = SchemaCache::init(&config, &notion).await;
    let schema_cache = Arc::new(schema_cache);
    tracing::info!("Schema cache ready with {} databases", schema_cache.db_keys().len());

    let server = LifeosServer::new((*config).clone(), notion, schema_cache);

    tracing::info!("Starting LifeOS MCP server on stdio");

    if let Err(e) = server.run().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
