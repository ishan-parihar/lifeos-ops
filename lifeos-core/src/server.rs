//! LifeOS MCP Server — direct stdio JSON-RPC with line-by-line reading

use std::sync::Arc;
use std::io::{BufRead, Write, BufReader};
use serde_json::{json, Value};

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::tools;

/// LifeOS MCP Server — manual JSON-RPC over stdio
pub struct LifeosServer {
    pub config: Arc<LifeOSConfig>,
    pub notion: Arc<NotionClient>,
}

impl LifeosServer {
    pub fn new(config: LifeOSConfig, notion: Arc<NotionClient>) -> Self {
        Self { config: Arc::new(config), notion }
    }

    /// Run stdio MCP server (reads line-by-line, writes single-line JSON)
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin.lock());

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        self.handle_message(trimmed).await;
                    }
                }
                Err(e) => {
                    tracing::error!("Read error: {}", e);
                    break;
                }
            }
            line.clear();
        }
        Ok(())
    }

    fn send(&self, msg: Value) {
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "{}", serde_json::to_string(&msg).unwrap());
        let _ = out.flush();
    }

    fn ok(&self, id: &Value, result: Value) {
        self.send(json!({"jsonrpc":"2.0","id":id,"result":result}));
    }

    fn err(&self, id: &Value, code: i32, message: &str) {
        self.send(json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}}));
    }

    async fn handle_message(&self, raw: &str) {
        let req: Value = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => { tracing::warn!("Parse error: {}", e); return; }
        };

        let id = req.get("id").unwrap_or(&json!(null)).clone();
        let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("").to_string();

        match method.as_str() {
            "notifications/initialized" => {}

            "initialize" => {
                tracing::info!("Client initialized (protocol: {})",
                    req["params"]["protocolVersion"].as_str().unwrap_or("unknown"));
                let db_count = self.config.databases.len();
                self.ok(&id, json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "lifeos-mcp", "version": "0.1.0" },
                    "instructions": format!(
                        "LifeOS MCP server with {} databases and 7 tools: query, mutate, intelligence_briefing, data_science, review_pipeline, strategic_simulator, sync_note",
                        db_count
                    )
                }));
            }

            "ping" => self.ok(&id, json!({})),

            "tools/list" => {
                let tools = tools::get_tool_definitions(&self.config, &self.notion).await;
                self.ok(&id, json!({ "tools": tools }));
            }

            "tools/call" => {
                let params = req["params"].clone();
                let tool_name = params["name"].as_str().unwrap_or("").to_string();
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                tracing::info!("Tool call: {}", tool_name);

                match tools::call_tool(&tool_name, &args, &self.config, &self.notion).await {
                    Ok(text) => self.ok(&id, json!({
                        "content": [{ "type": "text", "text": text }]
                    })),
                    Err(e) => {
                        tracing::error!("Tool {} failed: {}", tool_name, e);
                        self.err(&id, -32603, &format!("Tool execution failed: {}", e));
                    }
                }
            }

            _ => {
                tracing::warn!("Unknown method: {}", method);
                self.err(&id, -32601, &format!("Method not found: {method}"));
            }
        }
    }
}
