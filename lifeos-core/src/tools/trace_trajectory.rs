//! `trace_trajectory` tool — walk Trajectory parent/child hierarchy.

use std::sync::Arc;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::PropertyValue;
use crate::util::schema_engine::SchemaCache;

#[derive(Debug, Deserialize)]
pub struct TraceTrajectoryParams {
    pub page_id: String,
}

pub fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "page_id": {"type": "string", "description": "The page ID to trace from (walks up the hierarchy)"}
        },
        "required": ["page_id"]
    })
}

pub async fn execute(
    params: &TraceTrajectoryParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _sc: &SchemaCache,
) -> Result<String, String> {
    let db = crate::config::resolve_db(config, "trajectory").ok_or("Trajectory DB not found")?;
    let et_prop = db.entry_type_property.clone().unwrap_or_else(|| "Item Type".to_string());

    let mut report = String::new();
    report.push_str("Trajectory Trace\n");
    report.push_str(&"=".repeat(60));
    report.push_str("\n\n");

    let mut current_id = params.page_id.clone();
    let mut depth = 0;
    let max_depth = 10;

    loop {
        if depth >= max_depth {
            report.push_str("  (max depth reached)\n");
            break;
        }

        let page = notion.get_page(&current_id).await?;
        let title = crate::transform::extract_title(&page);
        let item_type = page.properties.get(&et_prop)
            .and_then(|v| match v {
                PropertyValue::Select { select, .. } => select.as_ref().map(|s| s.name.clone()),
                PropertyValue::MultiSelect { multi_select, .. } => multi_select.first().map(|s| s.name.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "unknown".to_string());

        let layer = match item_type.as_str() {
            "Purpose" | "Value" | "Principle" | "Vision-Statement" | "Identity-Statement" => "Reference",
            "Annual-Goal" | "Quarterly-Goal" | "Milestone" => "Strategic",
            _ => "Execution",
        };

        let indent = "  ".repeat(depth + 1);
        report.push_str(&format!("{}[{}] {} ({})\n", indent, layer, title, item_type));

        // Find parent
        let parent_id = page.properties.values()
            .filter_map(|v| match v {
                PropertyValue::Relation { relation, .. } if !relation.is_empty() => {
                    Some(relation[0].id.clone())
                }
                _ => None,
            })
            .next();

        match parent_id {
            Some(pid) if pid != current_id => {
                current_id = pid;
                depth += 1;
            }
            _ => {
                report.push_str(&format!("{}↑ (top of hierarchy)\n", "  ".repeat(depth + 2)));
                break;
            }
        }
    }

    Ok(report)
}
