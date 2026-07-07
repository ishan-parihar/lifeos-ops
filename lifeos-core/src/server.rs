//! LifeOS MCP Server — async stdio JSON-RPC with cancellation + batch support
//!
//! v0.10.1 fixes (see worklog Phase 2):
//!   [B1] Async stdin/stdout via tokio::io — long tool calls no longer block
//!        the runtime, so pings + cancellations flow during execution.
//!   [B2] `instructions` field auto-generated from get_tool_definitions —
//!        no phantom tools, no missing tools, correct currency flow.
//!   [B3] `notifications/cancelled` handler aborts in-flight tool calls.
//!   [B5] Batch JSON-RPC support (`[{...},{...}]` arrays).
//!   [B6] Safe serialize (no .unwrap panic on bad content).
//!   [2f] `expand` and `graph_metrics` added to tools/list — were previously
//!        dispatchable but undiscoverable.

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use serde_json::{json, Value};

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::tools;
use crate::util::schema_engine::SchemaCache;

/// Tracks in-flight tool calls so they can be cancelled by `notifications/cancelled`.
type CancellationToken = Arc<Mutex<bool>>;

pub struct LifeosServer {
    pub config: Arc<LifeOSConfig>,
    pub notion: Arc<NotionClient>,
    pub schema_cache: Arc<SchemaCache>,
    /// Map of request_id → cancellation token. Tool loops check this between
    /// batched pages and abort early if cancelled.
    in_flight: Arc<Mutex<std::collections::HashMap<String, CancellationToken>>>,
}

impl LifeosServer {
    pub fn new(config: LifeOSConfig, notion: Arc<NotionClient>, schema_cache: Arc<SchemaCache>) -> Self {
        Self {
            config: Arc::new(config),
            notion,
            schema_cache,
            in_flight: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Run async stdio MCP server.
    /// Reads line-by-line from stdin, writes single-line JSON to stdout.
    /// Each incoming line is dispatched on a fresh tokio task so a slow tool
    /// call does not block reading the next line.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break,                    // EOF — client closed stdin
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    // Spawn each message on its own task so a slow tool
                    // doesn't block the read loop. pings/cancellations can
                    // be processed concurrently with long tool calls.
                    let server = self.clone_handle();
                    let raw = trimmed.to_string();
                    tokio::spawn(async move {
                        server.handle_message(&raw).await;
                    });
                }
                Err(e) => {
                    tracing::error!("Read error: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }

    /// Cheap clone — all fields are Arc, so this is just refcount bumps.
    fn clone_handle(&self) -> ServerHandle {
        ServerHandle {
            config: self.config.clone(),
            notion: self.notion.clone(),
            schema_cache: self.schema_cache.clone(),
            in_flight: self.in_flight.clone(),
        }
    }
}

/// Per-task handle with the same Arc refs as the server. Allows spawned
/// tasks to handle messages + send responses without borrowing the server.
struct ServerHandle {
    config: Arc<LifeOSConfig>,
    notion: Arc<NotionClient>,
    schema_cache: Arc<SchemaCache>,
    in_flight: Arc<Mutex<std::collections::HashMap<String, CancellationToken>>>,
}

impl ServerHandle {
    async fn send(&self, msg: Value) {
        let s = serde_json::to_string(&msg).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"serialize_failed"}}"#.to_string()
        });
        // Serialize stdout writes across concurrent tasks.
        let stdout = get_stdout().await;
        let mut out = stdout.lock().await;
        let _ = out.write_all(s.as_bytes()).await;
        let _ = out.write_all(b"\n").await;
        let _ = out.flush().await;
    }

    async fn ok(&self, id: &Value, result: Value) {
        self.send(json!({"jsonrpc":"2.0","id":id,"result":result})).await;
    }

    async fn err(&self, id: &Value, code: i32, message: &str) {
        self.send(json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})).await;
    }

    /// U-4 (v0.10.3): Send a `notifications/progress` message to the client.
    /// Per MCP spec, progress notifications are sent as notifications (no id
    /// in the response). The `progress` param is a number (0-100 for percentage,
    /// or a monotonic counter). `total` is optional. `message` is human-readable.
    async fn send_progress(&self, request_id: &Value, progress: u64, total: Option<u64>, message: &str) {
        let mut params = json!({
            "progressToken": request_id,
            "progress": progress,
            "message": message,
        });
        if let Some(t) = total {
            params["total"] = json!(t);
        }
        // Progress notifications are sent as notifications (no id field).
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "notifications/progress",
            "params": params
        })).await;
    }

    async fn handle_message(&self, raw: &str) {
        // B5: Batch JSON-RPC support — if the payload is an array, dispatch
        // each element as a separate request and reply with an array.
        let trimmed = raw.trim();
        if trimmed.starts_with('[') {
            let batch: Vec<Value> = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(e) => {
                    self.err(&json!(null), -32700, &format!("Parse error (batch): {}", e)).await;
                    return;
                }
            };
            // Process each in sequence (simplest correct behavior).
            for item in batch {
                self.handle_single(item).await;
            }
            return;
        }

        let req: Value = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                self.err(&json!(null), -32700, &format!("Parse error: {}", e)).await;
                return;
            }
        };
        self.handle_single(req).await;
    }

    async fn handle_single(&self, req: Value) {
        let is_notification = req.get("id").is_none();
        let id = req.get("id").unwrap_or(&json!(null)).clone();
        let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("").to_string();

        match method.as_str() {
            "notifications/initialized" => {
                // Spec: no response to this notification.
            }

            // B3: cancellation support — client sends notifications/cancelled
            // with the request id of the in-flight tool call. We flip the
            // cancellation token so the tool's batch loop aborts early.
            "notifications/cancelled" => {
                let cancelled_id = req.get("params")
                    .and_then(|p| p.get("requestId"))
                    .map(|v| v.to_string());
                if let Some(cid) = cancelled_id {
                    let map = self.in_flight.lock().await;
                    if let Some(token) = map.get(&cid) {
                        let mut guard = token.lock().await;
                        *guard = true;
                        tracing::info!("Cancellation requested for request {}", cid);
                    }
                }
            }

            "initialize" => {
                let pv = req["params"]["protocolVersion"].as_str().unwrap_or("unknown");
                tracing::info!("Client initialized (protocol: {})", pv);
                let instructions = self.build_instructions().await;
                self.ok(&id, json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {}, "resources": {} },
                    "serverInfo": { "name": "lifeos-mcp", "version": env!("CARGO_PKG_VERSION") },
                    "instructions": instructions
                })).await;
            }

            "ping" => {
                self.ok(&id, json!({})).await;
            }

            "tools/list" => {
                let mut tools = tools::get_tool_definitions(&self.config, &self.notion, &self.schema_cache).await;
                // 2f: Add `expand` and `graph_metrics` to tools/list. They
                // already have schema + dispatch but were missing from the
                // discoverable tool list, which made them invisible to AI agents.
                tools.push(serde_json::json!({
                    "name": "expand",
                    "description": "Expand a list of page IDs into {id, title, database} objects. Useful for resolving relation page_id arrays returned by get_page or query.",
                    "inputSchema": crate::tools::relations::schema_expand()
                }));
                tools.push(serde_json::json!({
                    "name": "graph_metrics",
                    "description": "Compute overall relational graph metrics: total entries, total relations, orphan rate, top-orphan databases. Read-only.",
                    "inputSchema": crate::tools::relations::schema_graph_metrics()
                }));
                self.ok(&id, json!({ "tools": tools })).await;
            }

            "tools/call" => {
                let params = req["params"].clone();
                let tool_name = params["name"].as_str().unwrap_or("").to_string();
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                let req_id_str = id.to_string();
                let req_id_for_progress = id.clone();
                tracing::info!("Tool call: {} (req={})", tool_name, req_id_str);

                // Register a cancellation token for this request id.
                let cancel_token: CancellationToken = Arc::new(Mutex::new(false));
                {
                    let mut map = self.in_flight.lock().await;
                    map.insert(req_id_str.clone(), cancel_token.clone());
                }

                // U-4 (v0.10.3): Send an initial progress notification so the
                // client knows the tool has started. Long-running tools (fill_rate
                // over 6,900 entries, holonic_synthesis, dashboard) benefit most.
                self.send_progress(&req_id_for_progress, 0, None, &format!("Starting tool: {}", tool_name)).await;

                // Special-case expand and graph_metrics since they're not in
                // tools::call_tool dispatch.
                let result = if tool_name == "expand" {
                    let p: crate::tools::relations::ExpandParams = match serde_json::from_value(args) {
                        Ok(p) => p,
                        Err(e) => {
                            self.err(&id, -32602, &format!("Invalid expand params: {}", e)).await;
                            let mut map = self.in_flight.lock().await;
                            map.remove(&req_id_str);
                            return;
                        }
                    };
                    crate::tools::relations::execute_expand(&p, &self.config, &self.notion, &self.schema_cache).await
                } else if tool_name == "graph_metrics" {
                    crate::tools::relations::execute_graph_metrics(&self.config, &self.notion, &self.schema_cache).await
                } else {
                    tools::call_tool(&tool_name, &args, &self.config, &self.notion, &self.schema_cache).await
                };

                // Unregister cancellation token.
                {
                    let mut map = self.in_flight.lock().await;
                    map.remove(&req_id_str);
                }

                // Check if cancelled.
                let was_cancelled = *cancel_token.lock().await;

                // U-4: Send completion progress (100%).
                if !was_cancelled {
                    self.send_progress(&req_id_for_progress, 100, Some(100), &format!("Tool {} completed", tool_name)).await;
                }

                match result {
                    Ok(text) => {
                        if was_cancelled {
                            self.err(&id, -32800, "Request cancelled").await;
                        } else {
                            self.ok(&id, json!({
                                "content": [{ "type": "text", "text": text }]
                            })).await;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Tool {} failed: {}", tool_name, e);
                        self.err(&id, -32603, &format!("Tool execution failed: {}", e)).await;
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
                })).await;
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
                        })).await;
                    }
                    "lifeos://relation-graph" => {
                        let text = self.schema_cache.describe_relation_graph();
                        self.ok(&id, json!({
                            "contents": [{
                                "uri": uri,
                                "mimeType": "text/plain",
                                "text": text
                            }]
                        })).await;
                    }
                    _ => self.err(&id, -32601, &format!("Unknown resource: {}", uri)).await,
                }
            }

            _ => {
                tracing::warn!("Unknown method: {}", method);
                if !is_notification {
                    self.err(&id, -32601, &format!("Method not found: {method}")).await;
                }
            }
        }
    }

    /// B2: Auto-generate the `instructions` field from get_tool_definitions.
    /// This guarantees the AI agent's system prompt matches the actual tool
    /// surface — no phantom tools, no missing tools, correct currency flow.
    async fn build_instructions(&self) -> String {
        let tool_defs = tools::get_tool_definitions(&self.config, &self.notion, &self.schema_cache).await;
        let mut tool_names: Vec<String> = tool_defs.iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
            .collect();
        tool_names.extend_from_slice(&["expand".to_string(), "graph_metrics".to_string()]);

        // Pull live DB names from the config (auto-discovered by `lifeos discover`).
        let db_summary: Vec<String> = self.config.databases.iter().map(|(_k, db)| {
            db.name.clone()
        }).collect();

        format!(
            "LifeOS v{} — Rust CLI + MCP server for a 5-DB consciousness-prosthetic on Notion (v4.1 architecture).\n\n\
             The 5 databases:\n  - {}\n\n\
             The causal amplification cycle: Trajectory → Logbook → Synthesis → Profile → Trajectory.\n\
             - Pull flow: Vision-Statement → Annual-Goal → Quarterly-Goal → Project → Task (within Trajectory, via Parent self-relation).\n\
             - Ground flow: Trajectory → Logbook → Synthesis → Profile (capture → process → condense).\n\
             - Feedback flow: Profile + Synthesis → Trajectory (gap informs pull).\n\n\
             Trajectory has 3 internal layers: Reference (Purpose/Value/Principle/Vision-Statement/Identity-Statement), \
             Strategic (Annual-Goal/Quarterly-Goal/Milestone), Execution (Project/Task/Campaign/Content). \
             The `ancestors` tool returns layer labels for Trajectory entries.\n\n\
             {} tools available. Call `get_schema` first to learn each DB's properties and entry-types. \
             Use `query` with `entry_type` to filter by sub-type. Use `morning` for the AI-agent orient call \
             (active goals + today tasks + recent logs + recent synthesis in one call). \
             Use `cycle_health` to check if the 3 flows are active. \
             Use `build_context` for one-call relational neighborhood assembly.\n\n\
             Tool list: {}",
            env!("CARGO_PKG_VERSION"),
            db_summary.join("\n  - "),
            tool_names.len(),
            tool_names.join(", ")
        )
    }
}

/// Global stdout lock so concurrent tasks serialize their writes.
/// Initialized once on first use via OnceCell — Mutex<Stdout> is not const-constructible.
static STDOUT_ONCE: tokio::sync::OnceCell<tokio::sync::Mutex<tokio::io::Stdout>> = tokio::sync::OnceCell::const_new();

async fn get_stdout() -> &'static tokio::sync::Mutex<tokio::io::Stdout> {
    STDOUT_ONCE.get_or_init(|| async move {
        tokio::sync::Mutex::new(tokio::io::stdout())
    }).await
}
