//! LifeOS tool implementations

use std::sync::Arc;
use serde_json::Value;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::util::schema_engine::SchemaCache;

pub mod query;
pub mod mutate;
pub mod intelligence;
pub mod data_science;
pub mod review;
pub mod strategic;
pub mod sync_note;

fn enrich_database_param(schema: &mut Value, param_name: &str, schema_cache: &SchemaCache) {
    let db_keys: Vec<Value> = schema_cache.db_keys().iter().map(|k| Value::String(k.clone())).collect();
    if let Some(props) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
        if let Some(param) = props.get_mut(param_name) {
            param["enum"] = Value::Array(db_keys);
        }
    }
}

/// Get tool definitions in MCP format (JSON Schema per tool)
pub async fn get_tool_definitions(config: &LifeOSConfig, _notion: &NotionClient, schema_cache: &SchemaCache) -> Vec<Value> {
    let mut query_schema = query::schema(config, schema_cache);
    enrich_database_param(&mut query_schema, "database", schema_cache);

    let mut mutate_schema = mutate::schema();
    enrich_database_param(&mut mutate_schema, "database", schema_cache);

    let mut data_science_schema = data_science::schema();
    enrich_database_param(&mut data_science_schema, "database", schema_cache);
    enrich_database_param(&mut data_science_schema, "database_b", schema_cache);

    let mut strategic_schema = strategic::schema();
    enrich_database_param(&mut strategic_schema, "project_database", schema_cache);
    enrich_database_param(&mut strategic_schema, "okr_database", schema_cache);
    enrich_database_param(&mut strategic_schema, "campaign_database", schema_cache);

    // Build db schema context for tool descriptions
    let db_schema_desc: String = schema_cache.db_keys().iter()
        .map(|k| format!("  {}: {}", k, schema_cache.describe_db_properties(k)))
        .collect::<Vec<_>>()
        .join("\n");

    vec![
        tool_def("query", format!(
            "Unified high-fidelity query tool. Supports property filters, sort orders, limit 100, and presets (active, this_week, this_month, needs_review). Returns TOON-encoded results.\n\nDatabase schemas:\n{}\n\nUse the _db_schemas field in inputSchema for valid config-keys and their types/options.", db_schema_desc
        ), query_schema),
        tool_def("mutate", format!(
            "Create, update, delete, or upsert entries across all LifeOS databases. Returns TOON operation summary. Values auto-map to correct Notion types (select, status, url, email, multi_select, people, relation, files, date, number, checkbox) based on schema.\n\nDatabase schemas:\n{}", db_schema_desc
        ), mutate_schema),
        tool_def("intelligence_briefing", format!(
            "Role-based analysis: CEO, COO, CMO, CRO, CFO, CHO, or module-focused. Returns TOON-encoded analysis. See _db_schemas in inputSchema for valid database property types and filter options.\n\nDatabase schemas:\n{}", db_schema_desc
        ), intelligence::schema(schema_cache)),
        tool_def("data_science", format!(
            "Temporal patterns, trajectories, correlations, and weekday profiles. Returns TOON-encoded insights.\n\nDatabase schemas:\n{}", db_schema_desc
        ), data_science_schema),
        tool_def("review_pipeline", format!(
            "Periodic reviews: daily, weekly, monthly, quarterly, journal. Returns TOON-encoded review.\n\nDatabase schemas:\n{}", db_schema_desc
        ), review::schema()),
        tool_def("strategic_simulator", format!(
            "Cross-database strategic analysis: OKR alignment, project health, campaign performance. Returns TOON-encoded analysis.\n\nDatabase schemas:\n{}", db_schema_desc
        ), strategic_schema),
        tool_def("sync_note", format!(
            "Bidirectional Notion ↔ local markdown sync. Returns sync summary in TOON format.\n\nDatabase schemas:\n{}", db_schema_desc
        ), sync_note::schema()),
    ]
}

fn tool_def(name: &str, desc: String, schema: Value) -> Value {
    serde_json::json!({"name": name, "description": desc, "inputSchema": schema})
}

/// Call a tool by name from raw JSON args
pub async fn call_tool(
    name: &str,
    args: &Value,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    match name {
        "query" => {
            let params: query::QueryParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid query params: {}", e))?;
            query::execute(&params, config, notion, schema_cache).await
        }
        "mutate" => {
            let params: mutate::MutateParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid mutate params: {}", e))?;
            mutate::execute(&params, config, notion, schema_cache).await
        }
        "intelligence_briefing" => {
            let params: intelligence::IntelligenceParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid briefing params: {}", e))?;
            intelligence::execute(&params, config, notion, schema_cache).await
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
