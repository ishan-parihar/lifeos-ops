//! LifeOS tool implementations

use std::sync::Arc;
use serde_json::Value;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

pub mod query;
pub mod mutate;
pub mod intelligence;
pub mod data_science;
pub mod review;
pub mod strategic;
pub mod sync_note;

/// Get tool definitions in MCP format (JSON Schema per tool)
pub async fn get_tool_definitions(config: &LifeOSConfig, _notion: &NotionClient) -> Vec<Value> {
    vec![
        tool_def("query", "Unified high-fidelity query tool. Supports property filters (select, status, rich_text, title, date), sort orders, limit 100, and presets (active, this_week, this_month, needs_review). Returns TOON-encoded results.", query::schema(config)),
        tool_def("mutate", "Create, update, delete, or upsert entries across all LifeOS databases. Returns TOON operation summary.", mutate::schema()),
        tool_def("intelligence_briefing", "Role-based analysis: CEO, COO, CMO, CRO, CFO, CHO, or module-focused. Returns TOON-encoded analysis.", intelligence::schema()),
        tool_def("data_science", "Temporal patterns, trajectories, correlations, and weekday profiles. Returns TOON-encoded insights.", data_science::schema()),
        tool_def("review_pipeline", "Periodic reviews: daily, weekly, monthly, quarterly, journal. Returns TOON-encoded review.", review::schema()),
        tool_def("strategic_simulator", "Cross-database strategic analysis: OKR alignment, project health, campaign performance. Returns TOON-encoded analysis.", strategic::schema()),
        tool_def("sync_note", "Bidirectional Notion ↔ local markdown sync. Returns sync summary in TOON format.", sync_note::schema()),
    ]
}

fn tool_def(name: &str, desc: &str, schema: Value) -> Value {
    serde_json::json!({"name": name, "description": desc, "inputSchema": schema})
}

/// Call a tool by name from raw JSON args
pub async fn call_tool(
    name: &str,
    args: &Value,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    match name {
        "query" => {
            let params: query::QueryParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid query params: {}", e))?;
            query::execute(&params, config, notion).await
        }
        "mutate" => {
            let params: mutate::MutateParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid mutate params: {}", e))?;
            mutate::execute(&params, config, notion).await
        }
        "intelligence_briefing" => {
            let params: intelligence::IntelligenceParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid briefing params: {}", e))?;
            intelligence::execute(&params, config, notion).await
        }
        "data_science" => {
            let params: data_science::DataScienceParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid data_science params: {}", e))?;
            data_science::execute(&params, config, notion).await
        }
        "review_pipeline" => {
            let params: review::ReviewParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid review params: {}", e))?;
            review::execute(&params, config, notion).await
        }
        "strategic_simulator" => {
            let params: strategic::StrategicParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid strategic params: {}", e))?;
            strategic::execute(&params, config, notion).await
        }
        "sync_note" => {
            let params: sync_note::SyncNoteParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid sync_note params: {}", e))?;
            sync_note::execute(&params, config, notion).await
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}
