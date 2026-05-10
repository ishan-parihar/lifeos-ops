use std::sync::Arc;

use lifeos_core::{LifeosServer, load_config, NotionClient};

pub async fn run_server() {
    let config = match load_config() {
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

    let notion = Arc::new(NotionClient::new(config.clone(), token));
    let server = LifeosServer::new(config, notion);

    tracing::info!("Starting LifeOS MCP server on stdio");

    if let Err(e) = server.run().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}