//! Notion API client with rate limiting and caching

use std::time::{Duration, Instant};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use reqwest::{Client, RequestBuilder, StatusCode};

use crate::config::LifeOSConfig;
use crate::notion::types::*;

const BASE_URL: &str = "https://api.notion.com";
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 1000;

/// Notion API client
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

    async fn rate_limit(&self) {
        let min_interval = Duration::from_secs_f64(1.0 / self.config.rate_limit.requests_per_second);
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();
        if elapsed < min_interval {
            tokio::time::sleep(min_interval - elapsed).await;
        }
        *last = Instant::now();
    }

    fn request(&self, method: reqwest::Method, path: &str) -> RequestBuilder {
        let url = format!("{}{}", BASE_URL, path);
        self.http.request(method, &url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", &self.config.api_version)
            .header("Content-Type", "application/json")
    }

    async fn execute<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<T, String> {
        self.rate_limit().await;
        let mut attempt = 0;
        loop {
            let mut req = self.request(method.clone(), path);
            if let Some(b) = body {
                req = req.body(b.to_string());
            }
            let resp = req.send().await
                .map_err(|e| format!("Connection error: {}", e))?;
            let status = resp.status();
            if status == StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RETRIES {
                attempt += 1;
                let delay = RETRY_BASE_DELAY_MS * 2u64.pow(attempt);
                tracing::warn!("Rate limited, retrying in {}ms (attempt {})", delay, attempt);
                tokio::time::sleep(Duration::from_millis(delay)).await;
                continue;
            }
            let bytes = resp.bytes().await
                .map_err(|e| format!("Read error: {}", e))?;
            if !status.is_success() {
                return Err(format!("Notion API error {}: {}", status.as_u16(),
                    String::from_utf8_lossy(&bytes)));
            }
            return serde_json::from_slice(&bytes)
                .map_err(|e| format!("Parse error: {}", e));
        }
    }

    pub async fn query_data_source(&self, data_source_id: &str, body: &serde_json::Value) -> Result<QueryResponse, String> {
        self.execute(reqwest::Method::POST, &format!("/v1/data_sources/{}/query", data_source_id), Some(body)).await
    }

    pub async fn get_data_source(&self, id: &str) -> Result<NotionDataSource, String> {
        self.execute(reqwest::Method::GET, &format!("/v1/data_sources/{}", id), None).await
    }



    pub async fn get_database(&self, id: &str) -> Result<NotionDatabase, String> {
        self.execute(reqwest::Method::GET, &format!("/v1/databases/{}", id), None).await
    }

    pub async fn get_page(&self, page_id: &str) -> Result<NotionPage, String> {
        self.execute(reqwest::Method::GET, &format!("/v1/pages/{}", page_id), None).await
    }

    pub async fn create_page(&self, body: &serde_json::Value) -> Result<NotionPage, String> {
        self.execute(reqwest::Method::POST, "/v1/pages", Some(body)).await
    }

    pub async fn update_page(&self, page_id: &str, properties: &serde_json::Value) -> Result<NotionPage, String> {
        let body = serde_json::json!({ "properties": properties });
        self.execute(reqwest::Method::PATCH, &format!("/v1/pages/{}", page_id), Some(&body)).await
    }

    pub async fn archive_page(&self, page_id: &str) -> Result<NotionPage, String> {
        let body = serde_json::json!({ "archived": true });
        self.execute(reqwest::Method::PATCH, &format!("/v1/pages/{}", page_id), Some(&body)).await
    }

    pub async fn get_page_blocks(&self, page_id: &str) -> Result<Vec<NotionBlock>, String> {
        let mut blocks = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let path = match &cursor {
                Some(c) => format!("/v1/blocks/{}/children?page_size=100&start_cursor={}", page_id, c),
                None => format!("/v1/blocks/{}/children?page_size=100", page_id),
            };
            let result: BlockListResponse = self.execute(reqwest::Method::GET, &path, None).await?;
            blocks.extend(result.results);
            if result.has_more { cursor = result.next_cursor; } else { break; }
        }
        Ok(blocks)
    }
}

/// Block list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockListResponse {
    pub object: String,
    pub results: Vec<NotionBlock>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}
