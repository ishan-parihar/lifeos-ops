//! `capture` tool — quick <10s logging with auto-detection.
//!
//! Pass a text string; the tool detects the entry type (Activity/Diet/Financial/
//! Subjective/Relational/Systemic) and creates a Logbook entry automatically.

use std::sync::Arc;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

#[derive(Debug, Deserialize)]
pub struct CaptureParams {
    /// The text to capture (e.g. "Ate dal rice for lunch", "45min run", "Meeting with Ishaan")
    pub text: String,
    /// Optional: override the auto-detected entry type
    pub entry_type: Option<String>,
}

pub fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "text": {"type": "string", "description": "The text to capture. Auto-detects entry type from keywords."},
            "entry_type": {"type": "string", "description": "Optional: override auto-detection (Activity/Diet/Financial/Subjective/Relational/Systemic)"}
        },
        "required": ["text"]
    })
}

fn detect_entry_type(text: &str) -> &'static str {
    let t = text.to_lowercase();
    // Diet keywords
    if t.contains("ate") || t.contains("eat") || t.contains("lunch") || t.contains("dinner")
        || t.contains("breakfast") || t.contains("meal") || t.contains("calories")
        || t.contains("food") || t.contains("snack") || t.contains("drink") {
        return "Diet";
    }
    // Financial keywords
    if t.contains("rs") || t.contains("₹") || t.contains("$") || t.contains("spent")
        || t.contains("bought") || t.contains("paid") || t.contains("income")
        || t.contains("expense") || t.contains("salary") || t.contains("transfer") {
        return "Financial";
    }
    // Relational keywords
    if t.contains("meeting") || t.contains("call") || t.contains("talked")
        || t.contains("met with") || t.contains("conversation") || t.contains("chat") {
        return "Relational";
    }
    // Subjective keywords
    if t.contains("feel") || t.contains("realized") || t.contains("learned")
        || t.contains("dream") || t.contains("meditat") || t.contains("spiritual")
        || t.contains("insight") || t.contains("reflection") {
        return "Subjective";
    }
    // Systemic keywords
    if t.contains("process") || t.contains("system") || t.contains("workflow")
        || t.contains("improvement") || t.contains("bug") || t.contains("fix") {
        return "Systemic";
    }
    // Default: Activity
    "Activity"
}

pub async fn execute(
    params: &CaptureParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    let entry_type = params.entry_type.as_deref().unwrap_or_else(|| detect_entry_type(&params.text));

    let db = crate::config::resolve_db(config, "logbook")
        .ok_or("Logbook DB not found")?;
    let ds_id = db.ds_id().to_string();

    let properties = serde_json::json!({
        "Name": {"title": [{"type": "text", "text": {"content": &params.text}}]},
        "Entry Type": {"select": {"name": entry_type}},
        "Date": {"date": {"start": chrono::Utc::now().format("%Y-%m-%d").to_string()}},
        "Content": {"rich_text": [{"type": "text", "text": {"content": &params.text}}]},
    });

    let body = json!({"parent": {"data_source_id": ds_id}, "properties": properties});
    let resp = notion.create_page(&body).await?;

    let page_id = resp.id;
    Ok(format!("✓ Captured: \"{}\" → Logbook ({})\n  Page ID: {}", params.text, entry_type, page_id))
}
