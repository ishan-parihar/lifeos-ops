use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests_per_second: f64,
    last_request: Mutex<Instant>,
    burst_allowance: Mutex<usize>,
    burst_size: usize,
}

impl RateLimiter {
    pub fn new(requests_per_second: f64, burst_size: usize) -> Self {
        Self {
            requests_per_second,
            last_request: Mutex::new(Instant::now()),
            burst_allowance: Mutex::new(burst_size),
            burst_size,
        }
    }

    async fn acquire(&self) {
        let min_interval = Duration::from_secs_f64(1.0 / self.requests_per_second);

        let mut last = self.last_request.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last);

        if elapsed < min_interval {
            drop(last);
            sleep(min_interval - elapsed).await;
            let mut last = self.last_request.lock().await;
            *last = Instant::now();
        } else {
            *last = now;
        }

        let mut burst = self.burst_allowance.lock().await;
        if *burst < self.burst_size {
            *burst += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct NotionClient {
    client: Client,
    token: String,
    rate_limiter: Arc<RateLimiter>,
    api_version: String,
}

impl NotionClient {
    pub fn new(token: String, rate_limit: crate::RateLimitConfig, api_version: String) -> Self {
        let client = Client::builder()
            .user_agent("lifeos/0.1.0")
            .build()
            .expect("reqwest client should build");
        Self {
            client,
            token,
            rate_limiter: Arc::new(RateLimiter::new(
                rate_limit.requests_per_second,
                rate_limit.burst_size,
            )),
            api_version,
        }
    }

    pub fn with_default_api_version(token: String, rate_limit: crate::RateLimitConfig) -> Self {
        Self::new(token, rate_limit, "2025-09-03".to_string())
    }

    async fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<T> {
        self.rate_limiter.acquire();
        self._request(method, path, body).await
    }

    async fn _request<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<T> {
        let url = format!("https://api.notion.com/v1{}", path);
        let mut req = self.client.request(reqwest::Method::from_bytes(method.as_bytes()).unwrap(), &url);
        req = req.header("Authorization", format!("Bearer {}", self.token));
        req = req.header("Notion-Version", &self.api_version);
        req = req.header("Content-Type", "application/json");

        if let Some(b) = body {
            req = req.json(&b);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!("Notion API error {}: {}", status, text);
        }

        serde_json::from_str(&text).context("Failed to parse Notion response")
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request("GET", path, None).await
    }

    pub async fn post<T: DeserializeOwned>(&self, path: &str, body: Value) -> Result<T> {
        self.request("POST", path, Some(body)).await
    }

    pub async fn patch<T: DeserializeOwned>(&self, path: &str, body: Value) -> Result<T> {
        self.request("PATCH", path, Some(body)).await
    }

    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request("DELETE", path, None).await
    }

    pub async fn search(&self, query: Option<&str>, filter: Option<&str>, sort: Option<Value>, start_cursor: Option<&str>, page_size: Option<u32>) -> Result<Value> {
        let mut body = serde_json::json!({});
        if let Some(q) = query {
            body["query"] = serde_json::json!(q);
        }
        if let Some(f) = filter {
            body["filter"] = serde_json::json!({ "property": "object", "value": f });
        }
        if let Some(s) = sort {
            body["sort"] = s;
        }
        if let Some(c) = start_cursor {
            body["start_cursor"] = serde_json::json!(c);
        }
        if let Some(p) = page_size {
            body["page_size"] = serde_json::json!(p);
        }
        self.post("/search", body).await
    }

    pub async fn get_page(&self, page_id: &str) -> Result<Value> {
        self.get(&format!("/pages/{}", page_id)).await
    }

    pub async fn get_page_property(&self, page_id: &str, property_id: &str) -> Result<Value> {
        self.get(&format!("/pages/{}/properties/{}", page_id, property_id)).await
    }

    pub async fn create_page(&self, parent_id: &str, parent_type: &str, properties: Value, children: Option<Vec<Value>>) -> Result<Value> {
        let mut body = serde_json::json!({
            "parent": { "type": parent_type },
            "properties": properties
        });
        if let Some(c) = children {
            body["children"] = serde_json::json!(c);
        }
        self.post("/pages", body).await
    }

    pub async fn update_page(&self, page_id: &str, properties: Value, archived: Option<bool>, in_trash: Option<bool>) -> Result<Value> {
        let mut body = serde_json::json!({ "properties": properties });
        if let Some(a) = archived {
            body["archived"] = serde_json::json!(a);
        }
        if let Some(t) = in_trash {
            body["in_trash"] = serde_json::json!(t);
        }
        self.patch(&format!("/pages/{}", page_id), body).await
    }

    pub async fn get_block_children(&self, block_id: &str, page_size: Option<u32>, start_cursor: Option<&str>) -> Result<Value> {
        let mut path = format!("/blocks/{}/children", block_id);
        let mut params = vec![];
        if let Some(p) = page_size {
            params.push(format!("page_size={}", p));
        }
        if let Some(c) = start_cursor {
            params.push(format!("start_cursor={}", c));
        }
        if !params.is_empty() {
            path.push_str(&format!("?{}", params.join("&")));
        }
        self.get(&path).await
    }

    pub async fn append_block_children(&self, block_id: &str, children: Vec<Value>, after: Option<&str>) -> Result<Value> {
        let mut body = serde_json::json!({ "children": children });
        if let Some(a) = after {
            body["after"] = serde_json::json!(a);
        }
        self.patch(&format!("/blocks/{}/children", block_id), body).await
    }

    pub async fn update_block(&self, block_id: &str, block_type: &str, block: Value) -> Result<Value> {
        let body = serde_json::json!({ block_type: block });
        self.patch(&format!("/blocks/{}", block_id), body).await
    }

    pub async fn delete_block(&self, block_id: &str) -> Result<Value> {
        let body = serde_json::json!({ "archived": true });
        self.patch(&format!("/blocks/{}", block_id), body).await
    }

    pub async fn get_comments(&self, block_id: &str, page_size: Option<u32>, start_cursor: Option<&str>) -> Result<Value> {
        let mut path = format!("/comments?block_id={}", block_id);
        let mut params = vec![];
        if let Some(p) = page_size {
            params.push(format!("page_size={}", p));
        }
        if let Some(c) = start_cursor {
            params.push(format!("start_cursor={}", c));
        }
        if !params.is_empty() {
            path.push_str(&format!("&{}", params.join("&")));
        }
        self.get(&path).await
    }

    pub async fn create_comment(&self, parent_page_id: Option<&str>, discussion_id: Option<&str>, rich_text: Vec<Value>) -> Result<Value> {
        let mut body = serde_json::json!({ "rich_text": rich_text });
        if let Some(pid) = parent_page_id {
            body["parent"] = serde_json::json!({ "page_id": pid });
        }
        if let Some(did) = discussion_id {
            body["discussion_id"] = serde_json::json!(did);
        }
        self.post("/comments", body).await
    }

    pub async fn get_databases(&self, query: Option<&str>, filter: Option<Value>, sort: Option<Vec<Value>>, start_cursor: Option<&str>, page_size: Option<u32>) -> Result<Value> {
        let mut body = serde_json::json!({});
        if let Some(q) = query {
            body["query"] = serde_json::json!(q);
        }
        if let Some(f) = filter {
            body["filter"] = f;
        }
        if let Some(s) = sort {
            body["sort"] = serde_json::json!(s);
        }
        if let Some(c) = start_cursor {
            body["start_cursor"] = serde_json::json!(c);
        }
        if let Some(p) = page_size {
            body["page_size"] = serde_json::json!(p);
        }
        self.post("/databases/search", body).await
    }

    pub async fn create_data_source(&self, parent_page_id: &str, properties: Value, title: Vec<Value>) -> Result<Value> {
        let body = serde_json::json!({
            "parent": { "page_id": parent_page_id },
            "properties": properties,
            "title": title
        });
        self.post("/data_sources", body).await
    }

    pub async fn get_data_source(&self, data_source_id: &str) -> Result<Value> {
        self.get(&format!("/data_sources/{}", data_source_id)).await
    }

    pub async fn update_data_source(&self, data_source_id: &str, description: Option<Vec<Value>>, title: Option<Vec<Value>>, properties: Option<Value>) -> Result<Value> {
        let mut body = serde_json::json!({});
        if let Some(d) = description {
            body["description"] = serde_json::json!(d);
        }
        if let Some(t) = title {
            body["title"] = serde_json::json!(t);
        }
        if let Some(p) = properties {
            body["properties"] = p;
        }
        self.patch(&format!("/data_sources/{}", data_source_id), body).await
    }

    pub async fn delete_data_source(&self, data_source_id: &str) -> Result<Value> {
        let body = serde_json::json!({ "archived": true });
        self.patch(&format!("/data_sources/{}", data_source_id), body).await
    }

    pub async fn list_data_source_templates(&self, data_source_id: &str) -> Result<Value> {
        self.get(&format!("/data_sources/{}/templates", data_source_id)).await
    }

    pub async fn query_data_source(&self, data_source_id: &str, filter: Option<Value>, sorts: Option<Vec<Value>>, start_cursor: Option<&str>, page_size: Option<u32>) -> Result<Value> {
        let mut body = serde_json::json!({});
        if let Some(f) = filter {
            body["filter"] = f;
        }
        if let Some(s) = sorts {
            body["sorts"] = serde_json::json!(s);
        }
        if let Some(c) = start_cursor {
            body["start_cursor"] = serde_json::json!(c);
        }
        if let Some(p) = page_size {
            body["page_size"] = serde_json::json!(p);
        }
        self.post(&format!("/data_sources/{}/queries", data_source_id), body).await
    }

    pub async fn get_user(&self, user_id: &str) -> Result<Value> {
        self.get(&format!("/users/{}", user_id)).await
    }

    pub async fn list_users(&self) -> Result<Value> {
        self.get("/users").await
    }

    pub async fn get_self_user(&self) -> Result<Value> {
        self.get("/users/me").await
    }
}