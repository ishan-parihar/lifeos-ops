use std::time::{Duration, Instant};
use std::sync::Arc;
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use tokio::sync::Mutex;
use serde_json::Value;

use crate::config::LifeOSConfig;
use crate::notion::types::*;

const BASE_URL: &str = "https://api.notion.com";
const MAX_RETRIES: u32 = 5;
const RETRY_BASE_DELAY_MS: u64 = 1000;

fn get_jitter_ms() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let seed = (nanos ^ (nanos >> 17)) as u64;
    seed % 500
}

#[derive(Clone)]
pub struct NotionClient {
    config: LifeOSConfig,
    token: String,
    http: Client,
    _last_request: Arc<Mutex<Instant>>,
}

impl NotionClient {
    pub fn new(config: LifeOSConfig, token: String) -> Self {
        Self {
            config,
            token,
            http: Client::new(),
            _last_request: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn api_version(&self) -> &str {
        &self.config.api_version
    }

    async fn rate_limit(&self) {
        // Pre-emptive rate limiting is disabled to allow concurrent/parallel requests.
        // Reactive rate limiting (retry on 429) is handled in execute() and resolve_data_source_id().
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{}", BASE_URL, path);
        self.http
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", &self.config.api_version)
            .header("Content-Type", "application/json")
    }

    async fn execute<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<T, String> {
        self.rate_limit().await;
        let mut attempt = 0;
        loop {
            let mut req = self.request(method.clone(), path);
            if let Some(b) = body {
                req = req.body(b.to_string());
            }
            let resp = req.send().await.map_err(|e| format!("Connection: {}", e))?;
            let status = resp.status();
            if status == StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RETRIES {
                attempt += 1;
                let delay = RETRY_BASE_DELAY_MS * 2u64.pow(attempt) + get_jitter_ms();
                tracing::warn!("Rate limited, retry {}ms (attempt {})", delay, attempt);
                tokio::time::sleep(Duration::from_millis(delay)).await;
                continue;
            }
            let bytes = resp.bytes().await.map_err(|e| format!("Read: {}", e))?;
            if !status.is_success() {
                return Err(format!(
                    "Notion {} {}: {}",
                    status.as_u16(),
                    path,
                    String::from_utf8_lossy(&bytes)
                ));
            }
            return serde_json::from_slice(&bytes).map_err(|e| format!("Parse: {}", e));
        }
    }

    // --- Data Source APIs (2025-09-03) ---

    pub async fn query_data_source(&self, data_source_id: &str, body: &Value) -> Result<QueryResponse, String> {
        self.execute(Method::POST, &format!("/v1/data_sources/{data_source_id}/query"), Some(body)).await
    }

    pub async fn query_data_source_all(&self, data_source_id: &str) -> Result<Vec<NotionPage>, String> {
        self.query_data_source_all_since(data_source_id, None).await
    }

    pub async fn query_data_source_all_since(
        &self,
        data_source_id: &str,
        after: Option<&str>,
    ) -> Result<Vec<NotionPage>, String> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let mut body = serde_json::json!({ "page_size": 100 });
            if let Some(after) = after {
                body["filter"] = serde_json::json!({
                    "timestamp": "last_edited_time",
                    "last_edited_time": { "after": after }
                });
            }
            if let Some(c) = &cursor {
                body["start_cursor"] = serde_json::json!(c);
            }
            let resp = self.query_data_source(data_source_id, &body).await?;
            all.extend(resp.results);
            if resp.has_more {
                cursor = resp.next_cursor;
            } else {
                break;
            }
        }
        Ok(all)
    }

    pub async fn get_data_source(&self, id: &str) -> Result<NotionDataSource, String> {
        self.execute(Method::GET, &format!("/v1/data_sources/{id}"), None).await
    }

    // --- Database APIs (container level) ---

    pub async fn get_database(&self, id: &str) -> Result<NotionDatabase, String> {
        self.execute(Method::GET, &format!("/v1/databases/{id}"), None).await
    }

    /// Resolve database_id → data_source_id by fetching the database container.
    /// Always uses Notion-Version: 2025-09-03 regardless of config, since that's
    /// the version that returns the data_sources array.
    pub async fn resolve_data_source_id(&self, database_id: &str) -> Result<String, String> {
        self.rate_limit().await;
        let url = format!("{}/v1/databases/{}", BASE_URL, database_id);
        let mut attempt = 0;
        loop {
            let resp = self.http
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Notion-Version", "2025-09-03")
                .header("Content-Type", "application/json")
                .send()
                .await
                .map_err(|e| format!("Connection: {}", e))?;
            
            let status = resp.status();
            if status == StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RETRIES {
                attempt += 1;
                let delay = RETRY_BASE_DELAY_MS * 2u64.pow(attempt) + get_jitter_ms();
                tracing::warn!("Rate limited on resolve_data_source_id, retry {}ms (attempt {})", delay, attempt);
                tokio::time::sleep(Duration::from_millis(delay)).await;
                continue;
            }

            let bytes = resp.bytes().await.map_err(|e| format!("Read: {}", e))?;
            let val: Value = serde_json::from_slice(&bytes).map_err(|e| format!("Parse: {}", e))?;
            if !status.is_success() || val.get("code").is_some() {
                return Err(format!("Notion {} /v1/databases/{}: {}", 
                    status.as_u16(),
                    database_id,
                    val.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error")));
            }
            return val.get("data_sources")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|ds| ds.get("id"))
                .and_then(|id| id.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| format!("No data_sources found for database {database_id}"));
        }
    }

    // --- Page APIs ---

    pub async fn get_page(&self, page_id: &str) -> Result<NotionPage, String> {
        self.execute(Method::GET, &format!("/v1/pages/{page_id}"), None).await
    }

    pub async fn create_page(&self, body: &Value) -> Result<NotionPage, String> {
        self.execute(Method::POST, "/v1/pages", Some(body)).await
    }

    pub async fn update_page_properties(&self, page_id: &str, properties: &Value) -> Result<NotionPage, String> {
        let body = serde_json::json!({ "properties": properties });
        self.execute(Method::PATCH, &format!("/v1/pages/{page_id}"), Some(&body)).await
    }

    pub async fn update_page_full(&self, page_id: &str, body: &Value) -> Result<NotionPage, String> {
        self.execute(Method::PATCH, &format!("/v1/pages/{page_id}"), Some(body)).await
    }

    pub async fn update_page(&self, page_id: &str, properties: &Value) -> Result<NotionPage, String> {
        let body = serde_json::json!({ "properties": properties });
        self.execute(Method::PATCH, &format!("/v1/pages/{page_id}"), Some(&body)).await
    }

    pub async fn archive_page(&self, page_id: &str) -> Result<NotionPage, String> {
        let body = serde_json::json!({ "archived": true });
        self.execute(Method::PATCH, &format!("/v1/pages/{page_id}"), Some(&body)).await
    }

    // --- Block APIs ---

    pub async fn get_page_blocks(&self, page_id: &str) -> Result<Vec<NotionBlock>, String> {
        let mut blocks = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let path = match &cursor {
                Some(c) => format!("/v1/blocks/{page_id}/children?page_size=100&start_cursor={c}"),
                None => format!("/v1/blocks/{page_id}/children?page_size=100"),
            };
            let result: BlockListResponse = self.execute(Method::GET, &path, None).await?;
            blocks.extend(result.results);
            if result.has_more {
                cursor = result.next_cursor;
            } else {
                break;
            }
        }
        Ok(blocks)
    }

    pub async fn append_blocks(&self, block_id: &str, children: Vec<Value>) -> Result<(), String> {
        for chunk in children.chunks(100) {
            let body = serde_json::json!({ "children": chunk });
            self.execute::<()>(Method::PATCH, &format!("/v1/blocks/{block_id}/children"), Some(&body)).await?;
        }
        Ok(())
    }

    pub async fn update_block(&self, block_id: &str, block_type: &str, content: &Value) -> Result<(), String> {
        let body = serde_json::json!({ block_type: content });
        self.execute::<()>(Method::PATCH, &format!("/v1/blocks/{block_id}"), Some(&body)).await?;
        Ok(())
    }

    pub async fn delete_block(&self, block_id: &str) -> Result<(), String> {
        self.execute::<Value>(Method::DELETE, &format!("/v1/blocks/{block_id}"), None).await?;
        Ok(())
    }

    /// Search all databases accessible by the integration token.
    /// Returns list of (id, title) pairs for database-type objects only.
    pub async fn search_databases(&self) -> Result<Vec<(String, String)>, String> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let mut body = serde_json::json!({
                "filter": { "value": "data_source", "property": "object" },
                "page_size": 100
            });
            if let Some(c) = &cursor {
                body["start_cursor"] = serde_json::json!(c);
            }
            let resp: SearchResponse = self.execute(Method::POST, "/v1/search", Some(&body)).await?;
            for item in resp.results {
                let title: String = item.title.iter()
                    .filter_map(|rt| rt.plain_text.as_deref())
                    .collect();
                all.push((item.id, title));
            }
            if resp.has_more {
                cursor = resp.next_cursor;
            } else {
                break;
            }
        }
        tracing::info!("Found {} databases via Notion Search API", all.len());
        Ok(all)
    }
}

/// Resolve all database_ids in config to their data_source_ids.
/// Mutates config in place. Returns list of (db_key, error) for each failure.
///
/// When a database_id fails to resolve (e.g. wrong workspace), auto-discovers
/// the database by name via the Notion Search API and updates the config.
pub async fn resolve_all_data_sources(config: &mut LifeOSConfig, notion: &NotionClient) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(4));

    // Collect all (key, db_id, name) triples to resolve — just the 5 unified databases
    let mut work_items: Vec<(String, String, String)> = Vec::new();
    for (key, db) in config.databases.iter() {
        work_items.push((key.clone(), db.database_id.clone(), db.name.clone()));
    }

    // Resolve all concurrently using a helper
    let mut handles = Vec::new();
    for (key, db_id, name) in work_items {
        let notion_clone = notion.clone();
        let sem = semaphore.clone();
        handles.push(tokio::task::spawn(async move {
            let _permit = sem.acquire().await;
            let res = notion_clone.resolve_data_source_id(&db_id).await;
            (key, db_id, name, res)
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(r) => results.push(r),
            Err(e) => tracing::error!("Resolution task panicked: {:?}", e),
        }
    }

    // First pass: resolve what we can directly
    let mut unresolved: Vec<(String, String)> = Vec::new(); // (key, name)
    for (key, db_id, name, res) in results {
        match res {
            Ok(ds_id) => {
                tracing::info!("Resolved {key}: {db_id} → {ds_id}");
                if let Some(db) = config.databases.get_mut(&key) {
                    db.resolved_data_source_id = Some(ds_id.clone());
                }
            }
            Err(e) => {
                // Both 404 (wrong workspace/deleted) and other errors trigger auto-discover.
                // A 404 could mean the embedded ID doesn't exist in this workspace.
                tracing::warn!("{key}: Could not resolve {db_id}: {e} — will attempt name-based discovery");
                unresolved.push((key, name));
            }
        }
    }

    // Second pass: auto-discover unresolved databases by name.
    // search_databases() returns database container IDs (same type as database_id
    // in config — confirmed by the existing discover command in main.rs).
    // We set database_id to the found container ID, then resolve it to a
    // data_source_id via the Notion API.
    if !unresolved.is_empty() {
        tracing::info!("Auto-discovering {} unresolved databases by name...", unresolved.len());
        match notion.search_databases().await {
            Ok(notion_dbs) => {
                // Build name → (container_id, title) lookup for exact matching
                let name_map: std::collections::HashMap<String, (String, String)> = notion_dbs
                    .into_iter()
                    .map(|(id, title)| (title.to_lowercase(), (id, title)))
                    .collect();

                for (key, expected_name) in &unresolved {
                    let lookup_key = expected_name.to_lowercase();
                    if let Some((found_db_id, found_title)) = name_map.get(&lookup_key) {
                        tracing::info!("Auto-discovered {key}: \"{expected_name}\" → {found_db_id} (matched \"{found_title}\")");
                        // Update database_id (container ID) in config
                        if let Some(db) = config.databases.get_mut(key) {
                            db.database_id = found_db_id.clone();
                        }
                        // Resolve container ID → data_source_id
                        match notion.resolve_data_source_id(found_db_id).await {
                            Ok(ds_id) => {
                                tracing::info!("  Resolved {key} data_source: {found_db_id} → {ds_id}");
                                if let Some(db) = config.databases.get_mut(key) {
                                    db.resolved_data_source_id = Some(ds_id.clone());
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Auto-discovered {key} ({found_db_id}) but could not resolve data_source: {e}");
                                failures.push((key.clone(), e));
                            }
                        }
                    } else {
                        tracing::warn!("Auto-discover: no Notion database found matching \"{expected_name}\" for {key}");
                        failures.push((key.clone(), format!("No database named \"{expected_name}\" found in Notion workspace")));
                    }
                }
            }
            Err(e) => {
                tracing::error!("Auto-discover: failed to search Notion databases: {e}");
                for (key, name) in unresolved {
                    failures.push((key, format!("Search failed: {e} (looking for \"{name}\")")));
                }
            }
        }
    }

    if !failures.is_empty() {
        tracing::error!(
            "Data source resolution: {}/{} databases failed",
            failures.len(),
            config.databases.len()
        );
    }
    failures
}
