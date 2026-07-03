//! Data science tool — entry-type-aware aggregation, trends, correlations, and intelligence synthesis
//!
//! Re-architected for v5 5-DB holonic architecture. Every analysis supports:
//! - `entry_type` filtering (Activity, Diet, Project, etc.)
//! - `cycle` parameter (lesser/greater)
//! - Paginated Notion queries (handles has_more/cursor)
//! - Statistical summaries with interpretations for AI agent consumption

use std::sync::Arc;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::NotionPage;

// ══════════════════════════════════════════════════════════════════════
// Parameters
// ══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize, Clone)]
pub struct DataScienceParams {
    /// Analysis type: aggregate, profile, trend, compare, correlate, summarize
    pub analysis_type: String,
    /// Primary database to analyze
    pub database: String,
    /// Secondary database (for correlate)
    #[serde(default)]
    pub database_b: Option<String>,
    /// Number of days to look back (default: 30)
    #[serde(default)]
    pub days_back: Option<i64>,
    /// Filter by entry type within the database (e.g., "Activity" for potentiator)
    #[serde(default)]
    pub entry_type: Option<String>,
    /// Property to analyze (for aggregate group_by or trend metric)
    #[serde(default)]
    pub property: Option<String>,
    /// Metric property name for trend analysis (numeric property)
    #[serde(default)]
    pub metric_property: Option<String>,
    /// Group results by: status, date, entry_type, week, month
    #[serde(default)]
    pub group_by: Option<String>,
    /// Period for trend/compare: week, month, quarter
    #[serde(default)]
    pub period: Option<String>,
    /// Query all reservoirs in a cycle (lesser or greater) instead of a single DB
    #[serde(default)]
    pub cycle: Option<String>,
    /// Correlation metric for correlate: count, timing
    #[serde(default)]
    pub correlation_metric: Option<String>,
}

// ══════════════════════════════════════════════════════════════════════
// MCP Schema
// ══════════════════════════════════════════════════════════════════════

pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "analysis_type": {
                "type": "string",
                "enum": ["aggregate", "profile", "trend", "compare", "correlate", "summarize"],
                "description": "aggregate: count/summarize entries by group; profile: DB snapshot; trend: time-series direction; compare: period-over-period; correlate: cross-DB correlation; summarize: intelligent summary"
            },
            "database": { "type": "string", "description": "Primary database to analyze (matrix, potentiator, significator, greatway, nexus)" },
            "database_b": { "type": "string", "description": "Secondary database for correlate analysis" },
            "days_back": { "type": "integer", "description": "Days to look back (default: 30)" },
            "entry_type": { "type": "string", "description": "Filter by entry type (e.g., 'Activity' for potentiator, 'Project' for greatway). Use get_schema to discover valid entry types." },
            "property": { "type": "string", "description": "Property key to group by (for aggregate) or analyze" },
            "metric_property": { "type": "string", "description": "Numeric property for trend analysis (auto-detected if omitted)" },
            "group_by": { "type": "string", "enum": ["status", "date", "entry_type", "week", "month"], "description": "Group results by this dimension (for aggregate)" },
            "period": { "type": "string", "enum": ["week", "month", "quarter"], "description": "Period for trend/compare analysis (default: week)" },
            "cycle": { "type": "string", "enum": ["lesser", "greater"], "description": "Analyze across all reservoirs in a cycle instead of a single DB" },
            "correlation_metric": { "type": "string", "enum": ["count", "timing"], "description": "How to measure correlation (default: count)" }
        },
        "required": ["analysis_type", "database"]
    })
}

// ══════════════════════════════════════════════════════════════════════
// Pagination Helper
// ══════════════════════════════════════════════════════════════════════

/// Maximum pages to fetch per query (prevents runaway costs)
const MAX_PAGES: usize = 100;
/// Entries per page
const PAGE_SIZE: u64 = 100;

/// Paginated Notion query — fetches ALL matching entries up to MAX_PAGES.
async fn fetch_all_pages(
    notion: &NotionClient,
    ds_id: &str,
    filter: Option<&serde_json::Value>,
    max_pages: usize,
) -> Result<Vec<NotionPage>, String> {
    let mut all_pages = Vec::new();
    let mut start_cursor: Option<String> = None;
    let mut pages_fetched = 0;

    loop {
        if pages_fetched >= max_pages {
            break;
        }

        let mut body = serde_json::json!({ "page_size": PAGE_SIZE });
        if let Some(ref f) = filter {
            body["filter"] = (*f).clone();
        }
        if let Some(ref cursor) = start_cursor {
            body["start_cursor"] = serde_json::json!(cursor);
        }

        let result = notion.query_data_source(ds_id, &body).await?;
        let count = result.results.len();
        all_pages.extend(result.results);

        if !result.has_more || count == 0 {
            break;
        }

        start_cursor = result.next_cursor;
        pages_fetched += 1;
    }

    Ok(all_pages)
}

/// Fetch entries from a database with optional entry_type filter.
async fn fetch_entries(
    notion: &NotionClient,
    config: &LifeOSConfig,
    db_key: &str,
    entry_type: Option<&str>,
    days_back: i64,
) -> Result<Vec<NotionPage>, String> {
    let db = config.databases.get(db_key)
        .ok_or_else(|| format!("Unknown database: {}", db_key))?;

    let ds_id = db.ds_id();
    let properties = &db.properties;

    // Build date filter
    let since = (chrono::Utc::now() - chrono::Duration::days(days_back))
        .format("%Y-%m-%d").to_string();
    let date_prop = properties.get("date")
        .or_else(|| properties.get("action_date"))
        .or_else(|| properties.get("created_date"));

    let mut filters: Vec<serde_json::Value> = Vec::new();

    // Date filter
    if let Some(dp) = date_prop {
        filters.push(serde_json::json!({
            "property": dp,
            "date": { "on_or_after": since }
        }));
    }

    // Entry type filter
    if let Some(et) = entry_type {
        // Use the DB's configured entry_type_property name (authoritative); fall back to "Entry Type"
        let et_notion_name = db.entry_type_notion_name().unwrap_or("Entry Type");
        // The DB's entry_type_property_type is auto-corrected at runtime by
        // `SchemaCache::propagate_to_config` (called from main.rs resolve_with_schema),
        // which sets `entry_type_property_type` to match the live Notion schema when
        // discovered_properties is populated. As a defensive fallback for callers that
        // didn't go through resolve_with_schema, the original config value is used.
        let et_type = db.entry_type_property_type.clone();
        filters.push(build_entry_type_filter(et_notion_name, et, &et_type));
    }

    let filter = if filters.is_empty() {
        None
    } else if filters.len() == 1 {
        Some(filters.into_iter().next().unwrap())
    } else {
        Some(serde_json::json!({ "and": filters }))
    };

    fetch_all_pages(notion, ds_id, filter.as_ref(), MAX_PAGES).await
}

/// Determine the Notion filter type for the entry_type property.
/// select properties use {"select": {"equals": ...}}, multi_select use {"multi_select": {"contains": ...}}.
fn build_entry_type_filter(prop_name: &str, value: &str, prop_type: &str) -> serde_json::Value {
    match prop_type {
        "multi_select" => serde_json::json!({
            "property": prop_name,
            "multi_select": { "contains": value }
        }),
        _ => serde_json::json!({
            "property": prop_name,
            "select": { "equals": value }
        }),
    }
}

/// Fetch entries from all reservoirs in a cycle.
async fn fetch_cycle_entries(
    notion: &NotionClient,
    config: &LifeOSConfig,
    cycle: &str,
    entry_type: Option<&str>,
    days_back: i64,
) -> Result<HashMap<String, Vec<NotionPage>>, String> {
    let reservoir_keys = config.cycle_reservoirs(cycle);
    let mut result = HashMap::new();

    for key in &reservoir_keys {
        let pages = fetch_entries(notion, config, key, entry_type, days_back).await?;
        result.insert(key.clone(), pages);
    }

    Ok(result)
}

// ══════════════════════════════════════════════════════════════════════
// Statistical Helpers
// ══════════════════════════════════════════════════════════════════════

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() { return 0.0; }
    values.iter().sum::<f64>() / values.len() as f64
}

fn pearson_correlation(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len().min(ys.len()) as f64;
    if n < 2.0 { return 0.0; }

    let mx = mean(xs);
    let my = mean(ys);

    let mut sum_xy = 0.0;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;

    for i in 0..xs.len().min(ys.len()) {
        let dx = xs[i] - mx;
        let dy = ys[i] - my;
        sum_xy += dx * dy;
        sum_x2 += dx * dx;
        sum_y2 += dy * dy;
    }

    let denom = (sum_x2 * sum_y2).sqrt();
    if denom == 0.0 { return 0.0; }
    sum_xy / denom
}

fn change_pct(current: f64, previous: f64) -> f64 {
    if previous == 0.0 {
        if current > 0.0 { 100.0 } else { 0.0 }
    } else {
        ((current - previous) / previous * 100.0 * 10.0).round() / 10.0
    }
}

fn direction_label(pct: f64) -> &'static str {
    if pct > 10.0 { "up" }
    else if pct < -10.0 { "down" }
    else { "stable" }
}

// ══════════════════════════════════════════════════════════════════════
// Entry Extraction Helpers
// ══════════════════════════════════════════════════════════════════════

fn extract_entry_type(page: &NotionPage, entry_type_prop: &str) -> String {
    crate::transform::extract_string(page, entry_type_prop)
}

fn extract_status(page: &NotionPage, properties: &HashMap<String, String>) -> String {
    let status_prop = properties.get("status").map(|s| s.as_str()).unwrap_or("Status");
    crate::transform::extract_string(page, status_prop)
}

fn extract_date_str(page: &NotionPage, properties: &HashMap<String, String>) -> String {
    let date_prop = properties.get("date")
        .or_else(|| properties.get("action_date"))
        .or_else(|| properties.get("created_date"));
    if let Some(dp) = date_prop {
        crate::transform::extract_date(page, dp)
    } else {
        String::new()
    }
}

fn extract_numeric_value(page: &NotionPage, prop_name: &str) -> Option<f64> {
    crate::transform::extract_number(page, prop_name)
}

/// Auto-detect a numeric property from the config keys.
fn detect_metric_property(properties: &HashMap<String, String>) -> Option<String> {
    // Look for common metric property names in config keys
    for candidate in &["energy", "mood", "score", "value", "amount", "duration", "intensity", "rating"] {
        if properties.contains_key(*candidate) {
            return Some(candidate.to_string());
        }
    }
    // Look for any numeric property in the config
    for (key, _) in properties {
        if key.ends_with("_score") || key.ends_with("_value") || key.ends_with("_amount") {
            return Some(key.clone());
        }
    }
    None
}

// ══════════════════════════════════════════════════════════════════════
// Analysis: aggregate
// ══════════════════════════════════════════════════════════════════════

async fn analyze_aggregate(
    params: &DataScienceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let days = params.days_back.unwrap_or(30);

    // Support cycle mode
    if let Some(ref cycle) = params.cycle {
        let cycle_entries = fetch_cycle_entries(notion, config, cycle, params.entry_type.as_deref(), days).await?;
        let group_by = params.group_by.as_deref().unwrap_or("status");

        let mut all_groups: HashMap<String, i64> = HashMap::new();
        let mut total = 0i64;
        let mut per_db: HashMap<String, serde_json::Value> = HashMap::new();

        for (db_key, pages) in &cycle_entries {
            let db = config.databases.get(db_key.as_str());
            let properties = db.map(|d| &d.properties).cloned().unwrap_or_default();
            let mut groups: HashMap<String, i64> = HashMap::new();

            for page in pages {
                let group_value = match group_by {
                    "status" => extract_status(page, &properties),
                    "entry_type" => {
                        let et_prop = properties.get("entry_type").map(|s| s.as_str()).unwrap_or("Entry Type");
                        extract_entry_type(page, et_prop)
                    }
                    "date" => {
                        let d = extract_date_str(page, &properties);
                        if d.len() >= 10 { d[..10].to_string() } else { d }
                    }
                    "week" => {
                        let d = extract_date_str(page, &properties);
                        if d.len() >= 10 {
                            if let Ok(naive) = chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d") {
                                format!("W{}", naive.format("%U"))
                            } else { "unknown".into() }
                        } else { "unknown".into() }
                    }
                    "month" => {
                        let d = extract_date_str(page, &properties);
                        if d.len() >= 7 { d[..7].to_string() } else { d }
                    }
                    _ => "all".into(),
                };

                if !group_value.is_empty() && group_value != "unknown" {
                    *groups.entry(group_value).or_insert(0) += 1;
                }
                total += 1;
            }

            per_db.insert(db_key.clone(), serde_json::json!({
                "total": pages.len(),
                "groups": groups,
            }));
        }

        // Merge groups across DBs
        for (_, db_data) in &per_db {
            if let Some(groups) = db_data.get("groups").and_then(|g| g.as_object()) {
                for (k, v) in groups {
                    *all_groups.entry(k.clone()).or_insert(0) += v.as_i64().unwrap_or(0);
                }
            }
        }

        let interpretation = format!(
            "Cycle '{}' has {} total entries across {} reservoirs. Grouped by {}: top groups are {}",
            cycle, total, cycle_entries.len(), group_by,
            top_groups_string(&all_groups, 5)
        );

        return Ok(crate::toon_format::encode(&serde_json::json!({
            "analysis": "aggregate",
            "mode": "cycle",
            "cycle": cycle,
            "group_by": group_by,
            "total": total,
            "groups": all_groups,
            "per_database": per_db,
            "interpretation": interpretation,
        })));
    }

    // Single database mode
    let pages = fetch_entries(notion, config, &params.database, params.entry_type.as_deref(), days).await?;
    let db = config.databases.get(&params.database)
        .ok_or_else(|| format!("Unknown database: {}", params.database))?;
    let properties = &db.properties;
    let group_by = params.group_by.as_deref().unwrap_or("status");

    let mut groups: HashMap<String, i64> = HashMap::new();
    let mut dates: BTreeMap<String, i64> = BTreeMap::new();

    for page in &pages {
        let group_value = match group_by {
            "status" => extract_status(page, properties),
            "entry_type" => {
                let et_prop = properties.get("entry_type").map(|s| s.as_str()).unwrap_or("Entry Type");
                extract_entry_type(page, et_prop)
            }
            "date" => {
                let d = extract_date_str(page, properties);
                if d.len() >= 10 { d[..10].to_string() } else { d }
            }
            "week" => {
                let d = extract_date_str(page, properties);
                if d.len() >= 10 {
                    if let Ok(naive) = chrono::NaiveDate::parse_from_str(&d[..10], "%Y-%m-%d") {
                        format!("W{}", naive.format("%U"))
                    } else { "unknown".into() }
                } else { "unknown".into() }
            }
            "month" => {
                let d = extract_date_str(page, properties);
                if d.len() >= 7 { d[..7].to_string() } else { d }
            }
            _ => "all".into(),
        };

        if !group_value.is_empty() && group_value != "unknown" {
            *groups.entry(group_value.clone()).or_insert(0) += 1;
        }

        // Track date distribution for daily rate
        let d = extract_date_str(page, properties);
        if d.len() >= 10 {
            *dates.entry(d[..10].to_string()).or_insert(0) += 1;
        }
    }

    let daily_rate = if dates.len() > 0 {
        pages.len() as f64 / dates.len() as f64
    } else {
        0.0
    };

    let interpretation = format!(
        "'{}' has {} entries ({} days back). Grouped by {}: top groups are {}. Avg {:.1} entries/day.",
        params.database, pages.len(), days, group_by,
        top_groups_string(&groups, 5),
        daily_rate
    );

    Ok(crate::toon_format::encode(&serde_json::json!({
        "analysis": "aggregate",
        "database": params.database,
        "entry_type_filter": params.entry_type,
        "group_by": group_by,
        "total": pages.len(),
        "groups": groups,
        "date_range": {
            "earliest": dates.keys().next(),
            "latest": dates.keys().next_back(),
            "unique_dates": dates.len(),
        },
        "daily_rate": (daily_rate * 10.0).round() / 10.0,
        "interpretation": interpretation,
    })))
}

fn top_groups_string(groups: &HashMap<String, i64>, n: usize) -> String {
    let mut sorted: Vec<_> = groups.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    sorted.iter().take(n)
        .map(|(k, v)| format!("{}({})", k, v))
        .collect::<Vec<_>>()
        .join(", ")
}

// ══════════════════════════════════════════════════════════════════════
// Analysis: profile
// ══════════════════════════════════════════════════════════════════════

async fn analyze_profile(
    params: &DataScienceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let days = params.days_back.unwrap_or(30);
    let pages = fetch_entries(notion, config, &params.database, params.entry_type.as_deref(), days).await?;
    let db = config.databases.get(&params.database)
        .ok_or_else(|| format!("Unknown database: {}", params.database))?;
    let properties = &db.properties;

    // Entry type distribution
    let et_prop = properties.get("entry_type").map(|s| s.as_str());
    let mut entry_type_dist: HashMap<String, i64> = HashMap::new();
    let mut status_dist: HashMap<String, i64> = HashMap::new();
    let mut dates: Vec<String> = Vec::new();

    for page in &pages {
        // Entry type
        if let Some(etp) = et_prop {
            let et = extract_entry_type(page, etp);
            if !et.is_empty() {
                *entry_type_dist.entry(et).or_insert(0) += 1;
            }
        }

        // Status
        let status = extract_status(page, properties);
        if !status.is_empty() {
            *status_dist.entry(status).or_insert(0) += 1;
        }

        // Date
        let d = extract_date_str(page, properties);
        if !d.is_empty() {
            dates.push(d);
        }
    }

    // Date stats
    dates.sort();
    let unique_dates: usize = {
        let mut s: Vec<&str> = dates.iter().map(|d| &d[..d.len().min(10)]).collect();
        s.dedup();
        s.len()
    };
    let daily_rate = if unique_dates > 0 { pages.len() as f64 / unique_dates as f64 } else { 0.0 };

    // Detect available numeric properties
    let numeric_props: Vec<String> = properties.keys()
        .filter(|k| k.starts_with("energy") || k.starts_with("mood") || k.starts_with("score")
            || k.starts_with("amount") || k.starts_with("duration") || k.starts_with("value")
            || k.starts_with("intensity") || k.starts_with("rating") || k.ends_with("_score"))
        .cloned()
        .collect();

    let interpretation = format!(
        "'{}' profile: {} entries, {} entry types, {} statuses, {:.1} entries/day over {} unique dates. Available numeric properties: {}.",
        params.database, pages.len(), entry_type_dist.len(), status_dist.len(),
        daily_rate, unique_dates,
        if numeric_props.is_empty() { "none".into() } else { numeric_props.join(", ") }
    );

    Ok(crate::toon_format::encode(&serde_json::json!({
        "analysis": "profile",
        "database": params.database,
        "entry_type_filter": params.entry_type,
        "total": pages.len(),
        "entry_type_distribution": entry_type_dist,
        "status_distribution": status_dist,
        "date_stats": {
            "unique_dates": unique_dates,
            "earliest": dates.first(),
            "latest": dates.last(),
            "daily_rate": (daily_rate * 10.0).round() / 10.0,
        },
        "available_numeric_properties": numeric_props,
        "interpretation": interpretation,
    })))
}

// ══════════════════════════════════════════════════════════════════════
// Analysis: trend
// ══════════════════════════════════════════════════════════════════════

/// Core trend calculation — returns raw serde_json::Value for reuse by compare.
async fn compute_trend(
    params: &DataScienceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<serde_json::Value, String> {
    let period_days = match params.period.as_deref().unwrap_or("week") {
        "month" => 30,
        "quarter" => 90,
        _ => 7,
    };

    // Fetch current period and previous period
    let current_pages = fetch_entries(notion, config, &params.database, params.entry_type.as_deref(), period_days).await?;
    let previous_start = period_days;
    let previous_end = period_days * 2;
    let db = config.databases.get(&params.database)
        .ok_or_else(|| format!("Unknown database: {}", params.database))?;

    // For previous period, we need to fetch with a different date range
    // Build a manual query for previous period
    let since_prev = (chrono::Utc::now() - chrono::Duration::days(previous_end))
        .format("%Y-%m-%d").to_string();
    let until_prev = (chrono::Utc::now() - chrono::Duration::days(previous_start))
        .format("%Y-%m-%d").to_string();

    let date_prop = db.properties.get("date")
        .or_else(|| db.properties.get("action_date"))
        .or_else(|| db.properties.get("created_date"));

    let mut prev_filters: Vec<serde_json::Value> = Vec::new();
    if let Some(dp) = date_prop {
        prev_filters.push(serde_json::json!({
            "property": dp,
            "date": { "on_or_after": since_prev }
        }));
        prev_filters.push(serde_json::json!({
            "property": dp,
            "date": { "before": until_prev }
        }));
    }
    if let Some(ref et) = params.entry_type {
        if let Some(et_prop) = db.properties.get("entry_type") {
            let et_type = db.entry_type_property_type.as_str();
            prev_filters.push(build_entry_type_filter(et_prop, et, et_type));
        }
    }
    let prev_filter = if prev_filters.is_empty() { None }
        else if prev_filters.len() == 1 { Some(prev_filters.into_iter().next().unwrap()) }
        else { Some(serde_json::json!({ "and": prev_filters })) };

    let previous_pages = fetch_all_pages(notion, db.ds_id(), prev_filter.as_ref(), MAX_PAGES).await?;

    let current_count = current_pages.len() as f64;
    let previous_count = previous_pages.len() as f64;
    let pct = change_pct(current_count, previous_count);
    let dir = direction_label(pct);

    // Daily series for current period
    let mut daily: BTreeMap<String, i64> = BTreeMap::new();
    for page in &current_pages {
        let d = extract_date_str(page, &db.properties);
        if d.len() >= 10 {
            *daily.entry(d[..10].to_string()).or_insert(0) += 1;
        }
    }

    // Numeric metric trend (if metric_property specified)
    let mut metric_trend: Option<serde_json::Value> = None;
    let metric_key = params.metric_property.clone()
        .or_else(|| detect_metric_property(&db.properties));

    if let Some(ref mk) = metric_key {
        let current_vals: Vec<f64> = current_pages.iter()
            .filter_map(|p| extract_numeric_value(p, mk))
            .collect();
        let previous_vals: Vec<f64> = previous_pages.iter()
            .filter_map(|p| extract_numeric_value(p, mk))
            .collect();

        let current_mean = mean(&current_vals);
        let previous_mean = mean(&previous_vals);
        let metric_pct = change_pct(current_mean, previous_mean);

        metric_trend = Some(serde_json::json!({
            "metric": mk,
            "current_mean": (current_mean * 100.0).round() / 100.0,
            "previous_mean": (previous_mean * 100.0).round() / 100.0,
            "change_pct": metric_pct,
            "direction": direction_label(metric_pct),
            "current_samples": current_vals.len(),
            "previous_samples": previous_vals.len(),
        }));
    }

    let interpretation = format!(
        "'{}' trend ({}): {} entries this {} vs {} last {} ({:+.1}% {}).{}",
        params.database, params.entry_type.as_deref().unwrap_or("all"),
        current_count as i64, params.period.as_deref().unwrap_or("week"),
        previous_count as i64, params.period.as_deref().unwrap_or("week"),
        pct, dir,
        if let Some(ref mt) = metric_trend {
            let mk_name = mt["metric"].as_str().unwrap_or("");
            let mp = mt["change_pct"].as_f64().unwrap_or(0.0);
            format!(" Metric '{}' trend: {:+.1}%", mk_name, mp)
        } else { String::new() }
    );

    Ok(serde_json::json!({
        "analysis": "trend",
        "database": params.database,
        "entry_type_filter": params.entry_type,
        "period": params.period.as_deref().unwrap_or("week"),
        "current_period": {
            "count": current_count as i64,
            "period_days": period_days,
        },
        "previous_period": {
            "count": previous_count as i64,
            "period_days": period_days,
        },
        "change_pct": pct,
        "direction": dir,
        "daily_series": daily,
        "metric_trend": metric_trend,
        "interpretation": interpretation,
    }))
}

async fn analyze_trend(
    params: &DataScienceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let result = compute_trend(params, config, notion).await?;
    Ok(crate::toon_format::encode(&result))
}

// ══════════════════════════════════════════════════════════════════════
// Analysis: compare
// ══════════════════════════════════════════════════════════════════════

async fn analyze_compare(
    params: &DataScienceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let result_value = compute_trend(params, config, notion).await?;
    let mut result = result_value;
    result["analysis"] = serde_json::json!("compare");

    let current = result["current_period"]["count"].as_i64().unwrap_or(0);
    let previous = result["previous_period"]["count"].as_i64().unwrap_or(0);
    let pct = result["change_pct"].as_f64().unwrap_or(0.0);

    let notable = if pct > 50.0 {
        format!("Significant increase: {} more entries ({:+.1}%)", current - previous, pct)
    } else if pct < -50.0 {
        format!("Significant decrease: {} fewer entries ({:+.1}%)", previous - current, pct)
    } else if pct > 20.0 {
        format!("Moderate increase ({:+.1}%)", pct)
    } else if pct < -20.0 {
        format!("Moderate decrease ({:+.1}%)", pct)
    } else {
        "Stable — no significant change".to_string()
    };

    result["notable_changes"] = serde_json::json!(notable);
    result["interpretation"] = serde_json::json!(format!(
        "'{}' {} comparison: {} vs {} entries ({:+.1}%). {}",
        params.database, params.period.as_deref().unwrap_or("week"),
        current, previous, pct, notable
    ));

    Ok(crate::toon_format::encode(&result))
}

// ══════════════════════════════════════════════════════════════════════
// Analysis: correlate
// ══════════════════════════════════════════════════════════════════════

async fn analyze_correlate(
    params: &DataScienceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let days = params.days_back.unwrap_or(30);
    let db_b = params.database_b.as_deref()
        .ok_or("database_b is required for correlate analysis")?;

    let pages_a = fetch_entries(notion, config, &params.database, params.entry_type.as_deref(), days).await?;
    let pages_b = fetch_entries(notion, config, db_b, None, days).await?;

    let db_a_config = config.databases.get(&params.database)
        .ok_or_else(|| format!("Unknown database: {}", params.database))?;
    let db_b_config = config.databases.get(db_b)
        .ok_or_else(|| format!("Unknown database: {}", db_b))?;

    let metric = params.correlation_metric.as_deref().unwrap_or("count");

    match metric {
        "count" => {
            // Daily count correlation
            let mut daily_a: BTreeMap<String, i64> = BTreeMap::new();
            let mut daily_b: BTreeMap<String, i64> = BTreeMap::new();

            for page in &pages_a {
                let d = extract_date_str(page, &db_a_config.properties);
                if d.len() >= 10 {
                    *daily_a.entry(d[..10].to_string()).or_insert(0) += 1;
                }
            }
            for page in &pages_b {
                let d = extract_date_str(page, &db_b_config.properties);
                if d.len() >= 10 {
                    *daily_b.entry(d[..10].to_string()).or_insert(0) += 1;
                }
            }

            // Align dates
            let all_dates: Vec<String> = {
                let mut s: std::collections::HashSet<String> = daily_a.keys().chain(daily_b.keys()).cloned().collect();
                let mut v: Vec<String> = s.drain().collect();
                v.sort();
                v
            };

            let xs: Vec<f64> = all_dates.iter().map(|d| *daily_a.get(d).unwrap_or(&0) as f64).collect();
            let ys: Vec<f64> = all_dates.iter().map(|d| *daily_b.get(d).unwrap_or(&0) as f64).collect();

            let r = pearson_correlation(&xs, &ys);
            let r_abs = r.abs();

            let strength = if r_abs > 0.7 { "strong" }
                else if r_abs > 0.4 { "moderate" }
                else if r_abs > 0.2 { "weak" }
                else { "negligible" };

            let direction = if r > 0.0 { "positive" } else { "negative" };

            let interpretation = format!(
                "'{}' and '{}' have a {} {} correlation (r={:.3}). {} shared dates analyzed.",
                params.database, db_b, strength, direction, r, all_dates.len()
            );

            Ok(crate::toon_format::encode(&serde_json::json!({
                "analysis": "correlate",
                "database_a": params.database,
                "database_b": db_b,
                "metric": "daily_count",
                "correlation_coefficient": (r * 1000.0).round() / 1000.0,
                "strength": strength,
                "direction": direction,
                "shared_dates": all_dates.len(),
                "data_points": all_dates.iter().map(|d| serde_json::json!({
                    "date": d,
                    "count_a": daily_a.get(d).unwrap_or(&0),
                    "count_b": daily_b.get(d).unwrap_or(&0),
                })).collect::<Vec<_>>(),
                "interpretation": interpretation,
            })))
        }
        "timing" => {
            // Timing correlation — do entries cluster on the same days?
            let days_a: std::collections::HashSet<String> = pages_a.iter()
                .filter_map(|p| {
                    let d = extract_date_str(p, &db_a_config.properties);
                    if d.len() >= 10 { Some(d[..10].to_string()) } else { None }
                }).collect();
            let days_b: std::collections::HashSet<String> = pages_b.iter()
                .filter_map(|p| {
                    let d = extract_date_str(p, &db_b_config.properties);
                    if d.len() >= 10 { Some(d[..10].to_string()) } else { None }
                }).collect();

            let overlap: std::collections::HashSet<&String> = days_a.intersection(&days_b).collect();
            let union: std::collections::HashSet<&String> = days_a.union(&days_b).collect();

            let jaccard = if union.is_empty() { 0.0 } else { overlap.len() as f64 / union.len() as f64 };

            let interpretation = format!(
                "'{}' and '{}' share {} out of {} active days (Jaccard index: {:.3}).",
                params.database, db_b, overlap.len(), union.len(), jaccard
            );

            Ok(crate::toon_format::encode(&serde_json::json!({
                "analysis": "correlate",
                "database_a": params.database,
                "database_b": db_b,
                "metric": "timing",
                "jaccard_index": (jaccard * 1000.0).round() / 1000.0,
                "shared_active_days": overlap.len(),
                "total_unique_days": union.len(),
                "days_a_active": days_a.len(),
                "days_b_active": days_b.len(),
                "interpretation": interpretation,
            })))
        }
        _ => Err(format!("Unknown correlation_metric: {}. Use 'count' or 'timing'.", metric)),
    }
}

// ══════════════════════════════════════════════════════════════════════
// Analysis: summarize
// ══════════════════════════════════════════════════════════════════════

async fn analyze_summarize(
    params: &DataScienceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let days = params.days_back.unwrap_or(30);

    // Support cycle mode
    if let Some(ref cycle) = params.cycle {
        let cycle_entries = fetch_cycle_entries(notion, config, cycle, params.entry_type.as_deref(), days).await?;
        let mut total = 0i64;
        let mut all_status: HashMap<String, i64> = HashMap::new();
        let mut all_entry_types: HashMap<String, i64> = HashMap::new();
        let mut per_db_summaries: Vec<serde_json::Value> = Vec::new();

        for (db_key, pages) in &cycle_entries {
            let db = config.databases.get(db_key.as_str());
            let properties = db.map(|d| &d.properties).cloned().unwrap_or_default();
            let et_prop = properties.get("entry_type").map(|s| s.as_str());

            let mut status: HashMap<String, i64> = HashMap::new();
            let mut entry_types: HashMap<String, i64> = HashMap::new();

            for page in pages {
                let s = extract_status(page, &properties);
                if !s.is_empty() { *status.entry(s).or_insert(0) += 1; }
                if let Some(etp) = et_prop {
                    let et = extract_entry_type(page, etp);
                    if !et.is_empty() { *entry_types.entry(et).or_insert(0) += 1; }
                }
                total += 1;
            }

            // Merge into global
            for (k, v) in &status { *all_status.entry(k.clone()).or_insert(0) += v; }
            for (k, v) in &entry_types { *all_entry_types.entry(k.clone()).or_insert(0) += v; }

            per_db_summaries.push(serde_json::json!({
                "database": db_key,
                "total": pages.len(),
                "top_status": status.iter().max_by_key(|(_, v)| *v).map(|(k, _)| k.clone()),
            }));
        }

        let action_items = generate_action_items_cycle(cycle, &all_status, total);

        return Ok(crate::toon_format::encode(&serde_json::json!({
            "analysis": "summarize",
            "mode": "cycle",
            "cycle": cycle,
            "period_days": days,
            "total": total,
            "entry_types": all_entry_types,
            "status_distribution": all_status,
            "per_database": per_db_summaries,
            "action_items": action_items,
            "interpretation": format!(
                "Cycle '{}': {} total entries across {} reservoirs. Entry types: {}. Status: {}.",
                cycle, total, cycle_entries.len(),
                top_groups_string(&all_entry_types, 5),
                top_groups_string(&all_status, 3)
            ),
        })));
    }

    // Single database summary
    let pages = fetch_entries(notion, config, &params.database, params.entry_type.as_deref(), days).await?;
    let db = config.databases.get(&params.database)
        .ok_or_else(|| format!("Unknown database: {}", params.database))?;
    let properties = &db.properties;

    let et_prop = properties.get("entry_type").map(|s| s.as_str());
    let mut status_dist: HashMap<String, i64> = HashMap::new();
    let mut entry_type_dist: HashMap<String, i64> = HashMap::new();
    let mut dates: Vec<String> = Vec::new();

    for page in &pages {
        let s = extract_status(page, properties);
        if !s.is_empty() { *status_dist.entry(s).or_insert(0) += 1; }

        if let Some(etp) = et_prop {
            let et = extract_entry_type(page, etp);
            if !et.is_empty() { *entry_type_dist.entry(et).or_insert(0) += 1; }
        }

        let d = extract_date_str(page, properties);
        if !d.is_empty() { dates.push(d); }
    }

    // Daily rate
    let unique_dates: Vec<&str> = {
        let mut s: Vec<&str> = dates.iter().map(|d| &d[..d.len().min(10)]).collect();
        s.dedup();
        s
    };
    let daily_rate = if !unique_dates.is_empty() { pages.len() as f64 / unique_dates.len() as f64 } else { 0.0 };

    // Trend (compare to previous period)
    let period_days = match params.period.as_deref().unwrap_or("week") {
        "month" => 30,
        "quarter" => 90,
        _ => 7,
    };
    let since_prev = (chrono::Utc::now() - chrono::Duration::days(period_days * 2))
        .format("%Y-%m-%d").to_string();
    let until_prev = (chrono::Utc::now() - chrono::Duration::days(period_days))
        .format("%Y-%m-%d").to_string();

    let date_prop = properties.get("date")
        .or_else(|| properties.get("action_date"))
        .or_else(|| properties.get("created_date"));

    let prev_count = if let Some(dp) = date_prop {
        let filter = serde_json::json!({
            "and": [
                { "property": dp, "date": { "on_or_after": since_prev } },
                { "property": dp, "date": { "before": until_prev } }
            ]
        });
        let prev_pages = fetch_all_pages(notion, db.ds_id(), Some(&filter), 10).await?;
        prev_pages.len() as f64
    } else {
        0.0
    };

    let week_pct = change_pct(pages.len() as f64, prev_count);

    // Action items
    let mut action_items: Vec<String> = Vec::new();
    if pages.is_empty() {
        action_items.push("No entries found in this period. Data gap detected.".into());
    } else if daily_rate < 1.0 {
        action_items.push(format!("Low activity: {:.1} entries/day. Consider increasing input.", daily_rate));
    }
    if let Some((status, count)) = status_dist.iter().max_by_key(|(_, v)| *v) {
        if status == "Raw" && *count as f64 / pages.len() as f64 > 0.5 {
            action_items.push("Majority of entries are Raw — digestion backlog detected.".into());
        }
    }
    if week_pct < -30.0 {
        action_items.push(format!("Week-over-week decline of {:.1}%. Investigate causes.", week_pct));
    }

    let interpretation = format!(
        "'{}': {} entries over {} days ({:.1}/day). {} this week vs {} last ({:+.1}%). Top entry types: {}. Top status: {}.{}",
        params.database, pages.len(), days, daily_rate,
        pages.len(), prev_count, week_pct,
        top_groups_string(&entry_type_dist, 3),
        top_groups_string(&status_dist, 2),
        if action_items.is_empty() { " No action items.".into() } else { format!(" {} action items.", action_items.len()) }
    );

    Ok(crate::toon_format::encode(&serde_json::json!({
        "analysis": "summarize",
        "database": params.database,
        "entry_type_filter": params.entry_type,
        "period_days": days,
        "total": pages.len(),
        "daily_rate": (daily_rate * 10.0).round() / 10.0,
        "week_over_week_pct": week_pct,
        "entry_type_distribution": entry_type_dist,
        "status_distribution": status_dist,
        "date_range": {
            "earliest": dates.first(),
            "latest": dates.last(),
            "unique_dates": unique_dates.len(),
        },
        "action_items": action_items,
        "interpretation": interpretation,
    })))
}

fn generate_action_items_cycle(cycle: &str, status_dist: &HashMap<String, i64>, total: i64) -> Vec<String> {
    let mut items = Vec::new();

    if total == 0 {
        items.push(format!("No entries in {} cycle — data gap detected.", cycle));
        return items;
    }

    if let Some((status, count)) = status_dist.iter().max_by_key(|(_, v)| *v) {
        let pct = *count as f64 / total as f64;
        if status == "Raw" && pct > 0.5 {
            items.push("Digestion backlog: majority of entries are Raw.".into());
        }
        if status == "Archived" && pct > 0.5 {
            items.push("Most entries archived — consider pruning or refreshing.".into());
        }
    }

    if status_dist.len() <= 1 {
        items.push("Low status diversity — entries may be stagnating in one state.".into());
    }

    items
}

// ══════════════════════════════════════════════════════════════════════
// Main Execute
// ══════════════════════════════════════════════════════════════════════

pub async fn execute(
    params: &DataScienceParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    match params.analysis_type.as_str() {
        "aggregate" => analyze_aggregate(params, config, notion).await,
        "profile" => analyze_profile(params, config, notion).await,
        "trend" => analyze_trend(params, config, notion).await,
        "compare" => analyze_compare(params, config, notion).await,
        "correlate" => analyze_correlate(params, config, notion).await,
        "summarize" => analyze_summarize(params, config, notion).await,
        // Legacy support
        "temporal" => analyze_aggregate(params, config, notion).await,
        "trajectories" => analyze_trend(params, config, notion).await,
        "weekday_profile" => {
            let mut p = params.clone();
            p.group_by = Some("date".into());
            analyze_aggregate(&p, config, notion).await
        }
        _ => Err(format!(
            "Unknown analysis type: {}. Use: aggregate, profile, trend, compare, correlate, summarize.",
            params.analysis_type
        )),
    }
}
