//! LifeOS MCP Server — direct stdio JSON-RPC with line-by-line reading

use std::sync::Arc;
use std::io::{BufRead, Write, BufReader};
use serde_json::{json, Value};

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::tools;
use crate::util::schema_engine::SchemaCache;

/// LifeOS MCP Server — manual JSON-RPC over stdio
pub struct LifeosServer {
    pub config: Arc<LifeOSConfig>,
    pub notion: Arc<NotionClient>,
    pub schema_cache: Arc<SchemaCache>,
}

impl LifeosServer {
    pub fn new(config: LifeOSConfig, notion: Arc<NotionClient>, schema_cache: Arc<SchemaCache>) -> Self {
        Self { config: Arc::new(config), notion, schema_cache }
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

        let is_notification = req.get("id").is_none();
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
                    "capabilities": { "tools": {}, "resources": {} },
                    "serverInfo": { "name": "lifeos-mcp", "version": "0.1.0" },
                    "instructions": format!(
                        "LifeOS v4 Holonic MCP server with {} reservoir databases (matrix, potentiator, significator, greatway, nexus) + satellites. 16 tools: get_schema, query, query_override, mutate, intelligence_briefing, data_science, review_pipeline, strategic_simulator, sync_note, energy_flow, drive_assessment, health_metrics, get_page, expand, trace, ancestors. Relations are shown as (relation→target_db) in schemas. Call get_schema first.",
                        db_count
                    )
                }));
            }

            "ping" => self.ok(&id, json!({})),

            "tools/list" => {
                let tools = tools::get_tool_definitions(&self.config, &self.notion, &self.schema_cache).await;
                self.ok(&id, json!({ "tools": tools }));
            }

            "tools/call" => {
                let params = req["params"].clone();
                let tool_name = params["name"].as_str().unwrap_or("").to_string();
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                tracing::info!("Tool call: {}", tool_name);
                match tools::call_tool(&tool_name, &args, &self.config, &self.notion, &self.schema_cache).await {
                    Ok(text) => self.ok(&id, json!({
                        "content": [{ "type": "text", "text": text }]
                    })),
                    Err(e) => {
                        tracing::error!("Tool {} failed: {}", tool_name, e);
                        self.err(&id, -32603, &format!("Tool execution failed: {}", e));
                    }
                }
            }
            "resources/list" => {
                self.ok(&id, json!({
                    "resources": [{
                        "uri": "lifeos://db-schemas",
                        "name": "Database Schemas",
                        "description": "All LifeOS database schemas with property names, types, and valid enum options",
                        "mimeType": "text/plain"
                    }, {
                        "uri": "lifeos://relation-graph",
                        "name": "Relational Graph",
                        "description": "Full DB-to-DB relation map showing which properties link which databases",
                        "mimeType": "text/plain"
                    }]
                }));
            }

            "resources/read" => {
                let uri = req["params"]["uri"].as_str().unwrap_or("");
                match uri {
                    "lifeos://db-schemas" => {
                        let mut output = String::new();
                        for key in self.schema_cache.db_keys() {
                            let desc = self.schema_cache.describe_db_properties(key);
                            output.push_str(&format!("  {}: {}\n", key, desc));
                        }
                        let text = format!("Database schemas:\n{}", output);
                        self.ok(&id, json!({
                            "contents": [{
                                "uri": uri,
                                "mimeType": "text/plain",
                                "text": text
                            }]
                        }));
                    }
                    "lifeos://relation-graph" => {
                        let text = self.schema_cache.describe_relation_graph();
                        self.ok(&id, json!({
                            "contents": [{
                                "uri": uri,
                                "mimeType": "text/plain",
                                "text": text
                            }]
                        }));
                    }
                    _ => self.err(&id, -32601, &format!("Unknown resource: {}", uri)),
                }
            }

            _ => {
                tracing::warn!("Unknown method: {}", method);
                if !is_notification {
                    self.err(&id, -32601, &format!("Method not found: {method}"));
                }
            }
        }
    }
}
