use std::sync::Arc;

use lifeos_core::{LifeosServer, load_config, NotionClient, resolve_all_data_sources};

pub async fn run_server() {
    let mut config = match load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let token = std::env::var("NOTION_API_TOKEN").unwrap_or_else(|_| {
        eprintln!("NOTION_API_TOKEN environment variable is required");
        std::process::exit(1);
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
            failures.len(),
            config.databases.len()
        );
    } else {
        tracing::info!("All {} databases resolved successfully", config.databases.len());
    }

    // Create the actual client with resolved config
    let notion = Arc::new(NotionClient::new(config.clone(), token));
    let server = LifeosServer::new(config, notion);

    tracing::info!("Starting LifeOS MCP server on stdio");

    if let Err(e) = server.run().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
