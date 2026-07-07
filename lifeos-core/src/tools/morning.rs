//! `morning` tool — aggregated orientation view across all 5 DBs.
//!
//! The primary AI-agent "orient" call. One round-trip replaces 4+ queries.
//! Returns TOON-encoded JSON (parity with `daily` / `dashboard`).
//!
//! Returns:
//! - active_goals: Trajectory Annual-Goal + Quarterly-Goal where Status ≠ Done/Cancelled
//! - todays_tasks: Trajectory Task where Status ≠ Done, sorted by End Date asc
//! - recent_logs:  last 5 Logbook entries sorted by Date desc
//! - recent_synthesis: last 5 Synthesis entries sorted by Date desc
//!
//! ponytail: dropped 2 sections from v0 original:
//!   - profile_gaps: redundant with `query database=profile` (agent composes)
//!   - context_summary: too thin (just 2 counts); agent gets more from `query context`

use std::sync::Arc;
use serde_json::{json, Value};
use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::{NotionPage, PropertyValue};
use crate::util::schema_engine::SchemaCache;

pub fn schema() -> Value {
    json!({"type": "object", "properties": {}, "description": "No args. Returns active goals, today's tasks, recent logs, recent synthesis."})
}

pub async fn execute(
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let active_goals = query_traj_goals(config, notion).await.unwrap_or_default();
    let todays_tasks = query_traj_tasks(config, notion).await.unwrap_or_default();
    let recent_logs = query_recent(config, notion, "logbook", "Entry Type", "Date").await.unwrap_or_default();
    let recent_synthesis = query_recent(config, notion, "synthesis", "Category", "Date").await.unwrap_or_default();

    let data = json!({
        "morning": {
            "active_goals": active_goals,
            "todays_tasks": todays_tasks,
            "recent_logs": recent_logs,
            "recent_synthesis": recent_synthesis,
        }
    });
    Ok(crate::toon_format::encode(&data))
}

// ── helpers ──────────────────────────────────────────────────────────────

fn title(page: &NotionPage) -> String {
    crate::transform::extract_title(page)
}

fn select(page: &NotionPage, name: &str) -> Option<String> {
    match page.properties.get(name) {
        Some(PropertyValue::Select { select, .. }) => select.as_ref().map(|s| s.name.clone()),
        Some(PropertyValue::Status { status, .. }) => status.as_ref().map(|s| s.name.clone()),
        _ => None,
    }
}

fn date(page: &NotionPage, name: &str) -> Option<String> {
    match page.properties.get(name) {
        Some(PropertyValue::Date { date, .. }) => date.as_ref().map(|d| d.start.clone()),
        _ => None,
    }
}

/// Query Trajectory for active Annual-Goal + Quarterly-Goal entries.
async fn query_traj_goals(config: &LifeOSConfig, notion: &NotionClient) -> Result<Vec<Value>, String> {
    let db = crate::config::resolve_db(config, "trajectory").ok_or("Trajectory DB not found")?;
    let et_prop = db.entry_type_property.clone().unwrap_or_else(|| "Item Type".to_string());

    let body = json!({
        "page_size": 30,
        "filter": {"or": [
            {"property": et_prop, "select": {"equals": "Annual Goal"}},
            {"property": et_prop, "select": {"equals": "Quarterly Goal"}}
        ]}
    });
    let resp = notion.query_data_source(db.ds_id(), &body).await?;
    let out = resp.results.iter()
        .filter(|p| {
            let s = select(p, "Status").unwrap_or_default();
            s != "Done" && s != "Cancelled"
        })
        .map(|p| json!({
            "id": p.id,
            "title": title(p),
            "type": select(p, &et_prop).unwrap_or_default(),
            "status": select(p, "Status").unwrap_or_default(),
        }))
        .collect();
    Ok(out)
}

/// Query Trajectory for active Tasks sorted by End Date ascending.
async fn query_traj_tasks(config: &LifeOSConfig, notion: &NotionClient) -> Result<Vec<Value>, String> {
    let db = crate::config::resolve_db(config, "trajectory").ok_or("Trajectory DB not found")?;
    let et_prop = db.entry_type_property.clone().unwrap_or_else(|| "Item Type".to_string());

    let body = json!({
        "page_size": 15,
        "filter": {"and": [
            {"property": et_prop, "select": {"equals": "Task"}},
            {"property": "Status", "status": {"does_not_equal": "Done"}}
        ]},
        "sorts": [{"property": "End Date", "direction": "ascending"}]
    });
    let resp = notion.query_data_source(db.ds_id(), &body).await?;
    let out = resp.results.iter()
        .map(|p| json!({
            "id": p.id,
            "title": title(p),
            "priority": select(p, "Priority").unwrap_or_default(),
            "end_date": date(p, "End Date").unwrap_or_default(),
        }))
        .collect();
    Ok(out)
}

/// Generic "last N entries" query for any DB with a Date property.
async fn query_recent(
    config: &LifeOSConfig,
    notion: &NotionClient,
    db_key: &str,
    et_prop_default: &str,
    date_prop: &str,
) -> Result<Vec<Value>, String> {
    let db = crate::config::resolve_db(config, db_key)
        .ok_or_else(|| format!("{} DB not found", db_key))?;
    let et_prop = db.entry_type_property.clone().unwrap_or_else(|| et_prop_default.to_string());

    let body = json!({
        "page_size": 5,
        "sorts": [{"property": date_prop, "direction": "descending"}]
    });
    let resp = notion.query_data_source(db.ds_id(), &body).await?;
    let out = resp.results.iter()
        .map(|p| json!({
            "id": p.id,
            "title": title(p),
            "date": date(p, date_prop).unwrap_or_default(),
            "type": select(p, &et_prop).unwrap_or_default(),
        }))
        .collect();
    Ok(out)
}
