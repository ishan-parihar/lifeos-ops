use std::time::{Duration, Instant};
use std::sync::Arc;
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use tokio::sync::Mutex;
use serde_json::Value;

use crate::config::LifeOSConfig;
use crate::notion::types::*;

const BASE_URL: &str = "https://api.notion.com";
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 1000;

#[derive(Clone)]
pub struct NotionClient {
    config: LifeOSConfig,
    token: String,
    http: Client,
    last_request: Arc<Mutex<Instant>>,
}

impl NotionClient {
    pub fn new(config: LifeOSConfig, token: String) -> Self {
        Self {
            config,
            token,
            http: Client::new(),
            last_request: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn api_version(&self) -> &str {
        &self.config.api_version
    }

    async fn rate_limit(&self) {
        let min_interval = Duration::from_secs_f64(1.0 / self.config.rate_limit.requests_per_second);
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();
        if elapsed < min_interval {
            tokio::time::sleep(min_interval - elapsed).await;
        }
        *last = Instant::now();
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
                let delay = RETRY_BASE_DELAY_MS * 2u64.pow(attempt);
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
        let resp = self.http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", "2025-09-03")
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| format!("Connection: {}", e))?;
        let bytes = resp.bytes().await.map_err(|e| format!("Read: {}", e))?;
        let val: Value = serde_json::from_slice(&bytes).map_err(|e| format!("Parse: {}", e))?;
        if val.get("code").is_some() {
            return Err(format!("Notion {} /v1/databases/{}: {}", 
                val.get("status").and_then(|s| s.as_u64()).unwrap_or(0),
                database_id,
                val.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error")));
        }
        val.get("data_sources")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|ds| ds.get("id"))
            .and_then(|id| id.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("No data_sources found for database {database_id}"))
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
}

/// Resolve all database_ids in config to their data_source_ids.
/// Mutates config in place. Returns list of (db_key, error) for each failure.
pub async fn resolve_all_data_sources(config: &mut LifeOSConfig, notion: &NotionClient) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    for (key, db) in config.databases.iter_mut() {
        match notion.resolve_data_source_id(&db.database_id).await {
            Ok(ds_id) => {
                tracing::info!("Resolved {key}: {} → {ds_id}", db.database_id);
                db.resolved_data_source_id = Some(ds_id);
            }
            Err(e) => {
                let msg = format!("Could not resolve data_source_id for {key} ({}): {e}", db.database_id);
                tracing::warn!("{msg}");
                failures.push((key.clone(), e));
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
