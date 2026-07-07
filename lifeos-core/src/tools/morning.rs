//! `morning` tool — aggregated morning view across all 5 DBs.
//!
//! The primary user UX entry point. Returns:
//! - Active goals (Trajectory: Annual-Goal + Quarterly-Goal, Status=Active)
//! - Today's tasks (Trajectory: Task, Status≠Done, sorted by End Date)
//! - Recent logs (Logbook: last 5 entries, sorted by Date desc)
//! - Recent synthesis (Synthesis: last 5 entries, sorted by Date desc)
//! - Profile gaps (Profile: entries where Current Value ≠ Target Value)
//! - Pending interactions (Context: Person entries with Status=Active + no recent log)

use std::sync::Arc;
use serde_json::{json, Value};
use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

pub fn schema() -> Value {
    json!({"type": "object", "properties": {}})
}

pub async fn execute(
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    let mut report = String::new();
    report.push_str("LifeOS Morning View\n");
    report.push_str("=" .repeat(60).as_str());
    report.push_str("\n\n");

    // 1. Active Goals from Trajectory
    report.push_str("── ACTIVE GOALS ──\n");
    match query_trajectory_goals(config, notion).await {
        Ok(goals) => {
            if goals.is_empty() {
                report.push_str("  No active goals found.\n");
            } else {
                for g in &goals {
                    report.push_str(&format!("  • {} ({})\n", g.name, g.goal_type));
                }
            }
        }
        Err(e) => report.push_str(&format!("  Error: {}\n", e)),
    }

    // 2. Today's Tasks from Trajectory
    report.push_str("\n── TODAY'S TASKS ──\n");
    match query_trajectory_tasks(config, notion).await {
        Ok(tasks) => {
            if tasks.is_empty() {
                report.push_str("  No active tasks.\n");
            } else {
                for t in &tasks {
                    report.push_str(&format!("  • {} [{}]\n", t.name,
                        t.priority.as_deref().unwrap_or("none")));
                }
            }
        }
        Err(e) => report.push_str(&format!("  Error: {}\n", e)),
    }

    // 3. Recent Logbook entries
    report.push_str("\n── RECENT LOGS (last 5) ──\n");
    match query_recent_logs(config, notion).await {
        Ok(logs) => {
            if logs.is_empty() {
                report.push_str("  No recent logs.\n");
            } else {
                for l in &logs {
                    report.push_str(&format!("  • [{}] {} ({})\n", l.date, l.name, l.entry_type));
                }
            }
        }
        Err(e) => report.push_str(&format!("  Error: {}\n", e)),
    }

    // 4. Recent Synthesis entries
    report.push_str("\n── RECENT SYNTHESIS (last 5) ──\n");
    match query_recent_synthesis(config, notion).await {
        Ok(entries) => {
            if entries.is_empty() {
                report.push_str("  No recent synthesis entries.\n");
            } else {
                for e in &entries {
                    report.push_str(&format!("  • [{}] {} ({})\n", e.date, e.name, e.category));
                }
            }
        }
        Err(e) => report.push_str(&format!("  Error: {}\n", e)),
    }

    // 5. Profile gaps
    report.push_str("\n── PROFILE GAPS ──\n");
    match query_profile_gaps(config, notion).await {
        Ok(gaps) => {
            if gaps.is_empty() {
                report.push_str("  No profile gaps detected.\n");
            } else {
                for g in &gaps {
                    report.push_str(&format!("  • {}: {} → {} ({})\n",
                        g.name, g.current, g.target, g.category));
                }
            }
        }
        Err(e) => report.push_str(&format!("  Error: {}\n", e)),
    }

    // 6. Context summary
    report.push_str("\n── CONTEXT SUMMARY ──\n");
    match query_context_summary(config, notion).await {
        Ok(summary) => {
            report.push_str(&format!("  Active People: {}\n", summary.active_people));
            report.push_str(&format!("  Active Communities: {}\n", summary.active_communities));
        }
        Err(e) => report.push_str(&format!("  Error: {}\n", e)),
    }

    report.push_str("\n");
    report.push_str("=" .repeat(60).as_str());
    report.push_str("\nTo capture a new log, use: capture \"your text here\"\n");
    report.push_str("To check cycle health, use: cycle_health\n");

    Ok(report)
}

struct GoalInfo { name: String, goal_type: String }
struct TaskInfo { name: String, priority: Option<String> }
struct LogInfo { name: String, date: String, entry_type: String }
struct SynthesisInfo { name: String, date: String, category: String }
struct GapInfo { name: String, current: String, target: String, category: String }
struct ContextSummary { active_people: usize, active_communities: usize }

fn get_title(props: &std::collections::HashMap<String, PropertyValue>) -> String {
    match props.get("Name") {
        Some(PropertyValue::Title { title, .. }) if !title.is_empty() =>
            title.first().and_then(|t| t.plain_text.clone()).unwrap_or_default(),
        _ => String::new(),
    }
}

fn get_select(props: &std::collections::HashMap<String, PropertyValue>, name: &str) -> Option<String> {
    match props.get(name) {
        Some(PropertyValue::Select { select, .. }) => select.as_ref().map(|s| s.name.clone()),
        Some(PropertyValue::Status { status, .. }) => status.as_ref().map(|s| s.name.clone()),
        _ => None,
    }
}

fn get_rich_text(props: &std::collections::HashMap<String, PropertyValue>, name: &str) -> String {
    match props.get(name) {
        Some(PropertyValue::RichText { rich_text, .. }) if !rich_text.is_empty() =>
            rich_text.first().and_then(|t| t.plain_text.clone()).unwrap_or_default(),
        _ => String::new(),
    }
}

fn get_date(props: &std::collections::HashMap<String, PropertyValue>, name: &str) -> String {
    match props.get(name) {
        Some(PropertyValue::Date { date, .. }) => date.as_ref().map(|d| d.start.clone()).unwrap_or_default(),
        _ => String::new(),
    }
}

async fn query_trajectory_goals(config: &LifeOSConfig, notion: &NotionClient) -> Result<Vec<GoalInfo>, String> {
    let db = crate::config::resolve_db(config, "trajectory").ok_or("Trajectory DB not found")?;
    let ds_id = db.ds_id().to_string();
    let et_prop = db.entry_type_property.clone().unwrap_or_else(|| "Item Type".to_string());

    let body = json!({
        "page_size": 20,
        "filter": {"or": [
            {"property": et_prop, "select": {"equals": "Annual-Goal"}},
            {"property": et_prop, "select": {"equals": "Quarterly-Goal"}},
        ]},
        "sorts": [{"property": "Status", "direction": "ascending"}]
    });
    let resp = notion.query_data_source(&ds_id, &body).await?;
    let mut goals = Vec::new();
    for page in resp.results {
        let name = get_title(&page.properties);
        let goal_type = get_select(&page.properties, &et_prop).unwrap_or_default();
        let status = get_select(&page.properties, "Status").unwrap_or_default();
        if status != "Done" && status != "Cancelled" {
            goals.push(GoalInfo { name, goal_type });
        }
    }
    Ok(goals)
}

async fn query_trajectory_tasks(config: &LifeOSConfig, notion: &NotionClient) -> Result<Vec<TaskInfo>, String> {
    let db = crate::config::resolve_db(config, "trajectory").ok_or("Trajectory DB not found")?;
    let ds_id = db.ds_id().to_string();
    let et_prop = db.entry_type_property.clone().unwrap_or_else(|| "Item Type".to_string());

    let body = json!({
        "page_size": 15,
        "filter": {"and": [
            {"property": et_prop, "select": {"equals": "Task"}},
            {"property": "Status", "status": {"does_not_equal": "Done"}},
        ]},
        "sorts": [{"property": "End Date", "direction": "ascending"}]
    });
    let resp = notion.query_data_source(&ds_id, &body).await?;
    let mut tasks = Vec::new();
    for page in resp.results {
        let name = get_title(&page.properties);
        let priority = get_select(&page.properties, "Priority");
        tasks.push(TaskInfo { name, priority });
    }
    Ok(tasks)
}

async fn query_recent_logs(config: &LifeOSConfig, notion: &NotionClient) -> Result<Vec<LogInfo>, String> {
    let db = crate::config::resolve_db(config, "logbook").ok_or("Logbook DB not found")?;
    let ds_id = db.ds_id().to_string();
    let et_prop = db.entry_type_property.clone().unwrap_or_else(|| "Entry Type".to_string());

    let body = json!({
        "page_size": 5,
        "sorts": [{"property": "Date", "direction": "descending"}]
    });
    let resp = notion.query_data_source(&ds_id, &body).await?;
    let mut logs = Vec::new();
    for page in resp.results {
        let name = get_title(&page.properties);
        let date = get_date(&page.properties, "Date");
        let entry_type = get_select(&page.properties, &et_prop).unwrap_or_default();
        logs.push(LogInfo { name, date, entry_type });
    }
    Ok(logs)
}

async fn query_recent_synthesis(config: &LifeOSConfig, notion: &NotionClient) -> Result<Vec<SynthesisInfo>, String> {
    let db = crate::config::resolve_db(config, "synthesis").ok_or("Synthesis DB not found")?;
    let ds_id = db.ds_id().to_string();
    let et_prop = db.entry_type_property.clone().unwrap_or_else(|| "Category".to_string());

    let body = json!({
        "page_size": 5,
        "sorts": [{"property": "Date", "direction": "descending"}]
    });
    let resp = notion.query_data_source(&ds_id, &body).await?;
    let mut entries = Vec::new();
    for page in resp.results {
        let name = get_title(&page.properties);
        let date = get_date(&page.properties, "Date");
        let category = get_select(&page.properties, &et_prop).unwrap_or_default();
        entries.push(SynthesisInfo { name, date, category });
    }
    Ok(entries)
}

async fn query_profile_gaps(config: &LifeOSConfig, notion: &NotionClient) -> Result<Vec<GapInfo>, String> {
    let db = crate::config::resolve_db(config, "profile").ok_or("Profile DB not found")?;
    let ds_id = db.ds_id().to_string();

    let body = json!({"page_size": 20});
    let resp = notion.query_data_source(&ds_id, &body).await?;
    let mut gaps = Vec::new();
    for page in resp.results {
        let name = get_title(&page.properties);
        let current = get_rich_text(&page.properties, "Current Value");
        let target = get_rich_text(&page.properties, "Target Value");
        let category = get_select(&page.properties, "Category").unwrap_or_default();
        if !target.is_empty() && current != target {
            gaps.push(GapInfo { name, current, target, category });
        }
    }
    Ok(gaps)
}

async fn query_context_summary(config: &LifeOSConfig, notion: &NotionClient) -> Result<ContextSummary, String> {
    let db = crate::config::resolve_db(config, "context").ok_or("Context DB not found")?;
    let ds_id = db.ds_id().to_string();

    let body = json!({
        "page_size": 100,
        "filter": {"property": "Status", "select": {"equals": "Active"}}
    });
    let resp = notion.query_data_source(&ds_id, &body).await?;
    let mut people = 0;
    let mut communities = 0;
    for page in resp.results {
        let t = get_select(&page.properties, "Type").unwrap_or_default();
        match t.as_str() {
            "Person" => people += 1,
            "Community" => communities += 1,
            _ => {}
        }
    }
    Ok(ContextSummary { active_people: people, active_communities: communities })
}
