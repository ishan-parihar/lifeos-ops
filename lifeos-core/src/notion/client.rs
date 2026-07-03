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

    /// Resolve an ID stored in `DbConfig.database_id` (serialized as `data_source_id`
    /// in JSON config) to a usable data_source_id for query/create calls.
    ///
    /// Under Notion API version 2025-09-03, **database container IDs and data source
    /// IDs are distinct UUIDs**. A single Notion "database" exposes BOTH:
    ///   - `GET /v1/databases/{container_id}` → returns the container with a
    ///     `data_sources` array (one or more data source IDs).
    ///   - `GET /v1/data_sources/{data_source_id}` → returns the data source itself
    ///     (schema + properties), and is the ID used for `/query` and page parents.
    ///
    /// The LifeOS config field `DbConfig.database_id` (serde-renamed to
    /// `data_source_id` in JSON) is populated by:
    ///   1. `lifeos discover` — which writes the ID returned by Notion Search API
    ///      filtered to `object == "data_source"`, i.e. an actual data_source_id.
    ///   2. The user copying an ID from a Notion URL — which historically could be
    ///      either form.
    ///
    /// Therefore the resolver must accept **both** forms:
    ///   - If the ID is a data_source_id (the common case post-2025-09-03), validate
    ///     it via `GET /v1/data_sources/{id}` and return it as-is.
    ///   - Otherwise, fall back to `GET /v1/databases/{id}` and extract
    ///     `data_sources[0].id` from the container.
    ///
    /// Always uses Notion-Version: 2025-09-03 since that is the version that
    /// returns the `data_sources` array on the database endpoint.
    pub async fn resolve_data_source_id(&self, database_id: &str) -> Result<String, String> {
        // Fast path: try the data_sources endpoint first. This is the common case
        // post-2025-09-03 (config holds a data_source_id from discover or a Notion
        // URL that points at the data source).
        let ds_err = match self.resolve_via_data_source_endpoint(database_id).await {
            Ok(ds_id) => return Ok(ds_id),
            Err(e) => {
                tracing::debug!(
                    "ID {} did not resolve via /v1/data_sources ({}); falling back to /v1/databases",
                    database_id,
                    e
                );
                e
            }
        };

        // Legacy fallback: try the databases endpoint. This handles old configs
        // that stored an actual database container ID.
        match self.resolve_via_database_endpoint(database_id).await {
            Ok(ds_id) => Ok(ds_id),
            Err(db_err) => Err(format!(
                "Could not resolve {} as data_source ({}) or database container ({})",
                database_id, ds_err, db_err
            )),
        }
    }

    /// Try `GET /v1/data_sources/{id}` — if 200, the ID is already a valid
    /// data_source_id and we return it directly.
    async fn resolve_via_data_source_endpoint(&self, id: &str) -> Result<String, String> {
        self.rate_limit().await;
        let url = format!("{}/v1/data_sources/{}", BASE_URL, id);
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
                tracing::warn!(
                    "Rate limited on /v1/data_sources resolve, retry {}ms (attempt {})",
                    delay, attempt
                );
                tokio::time::sleep(Duration::from_millis(delay)).await;
                continue;
            }

            let bytes = resp.bytes().await.map_err(|e| format!("Read: {}", e))?;
            let val: Value = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Parse: {}", e))?;

            if !status.is_success() || val.get("code").is_some() {
                return Err(format!(
                    "Notion {} /v1/data_sources/{}: {}",
                    status.as_u16(),
                    id,
                    val.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error")
                ));
            }

            // The response is the data source object itself — its `id` field is
            // the canonical data_source_id. Defensive: if for some reason `id`
            // is missing, trust the input (it just validated successfully).
            let resolved = val.get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| id.to_string());
            return Ok(resolved);
        }
    }

    /// Try `GET /v1/databases/{id}` — extract `data_sources[0].id` from the
    /// container response. Used as a legacy fallback for old configs that
    /// stored an actual database container ID.
    async fn resolve_via_database_endpoint(&self, database_id: &str) -> Result<String, String> {
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
                tracing::warn!(
                    "Rate limited on /v1/databases resolve, retry {}ms (attempt {})",
                    delay, attempt
                );
                tokio::time::sleep(Duration::from_millis(delay)).await;
                continue;
            }

            let bytes = resp.bytes().await.map_err(|e| format!("Read: {}", e))?;
            let val: Value = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Parse: {}", e))?;
            if !status.is_success() || val.get("code").is_some() {
                return Err(format!(
                    "Notion {} /v1/databases/{}: {}",
                    status.as_u16(),
                    database_id,
                    val.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error")
                ));
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
    // search_databases() filters Notion Search to `object == "data_source"`, so
    // the IDs it returns ARE data_source_ids under Notion v2025-09-03. We write
    // the discovered ID into `database_id` (the serde-renamed `data_source_id`
    // JSON field) and then re-resolve — the resolver will recognize it via the
    // `/v1/data_sources/{id}` fast path and store it as `resolved_data_source_id`.
    if !unresolved.is_empty() {
        tracing::info!("Auto-discovering {} unresolved databases by name...", unresolved.len());
        match notion.search_databases().await {
            Ok(notion_dbs) => {
                // Build name → (data_source_id, title) lookup for case-insensitive matching
                let name_map: std::collections::HashMap<String, (String, String)> = notion_dbs
                    .into_iter()
                    .map(|(id, title)| (title.to_lowercase(), (id, title)))
                    .collect();

                for (key, expected_name) in &unresolved {
                    let lookup_key = expected_name.to_lowercase();
                    if let Some((found_db_id, found_title)) = name_map.get(&lookup_key) {
                        tracing::info!("Auto-discovered {key}: \"{expected_name}\" → {found_db_id} (matched \"{found_title}\")");
                        // Update database_id field (serialized as data_source_id in JSON).
                        // The discovered ID is already a data_source_id, so subsequent
                        // runs hit the fast path in resolve_data_source_id.
                        if let Some(db) = config.databases.get_mut(key) {
                            db.database_id = found_db_id.clone();
                        }
                        // Validate + cache as resolved_data_source_id
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
