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

    vec![
        tool_def("get_schema", "Returns database schemas. Call this first to see available databases and their properties. Pass a database name to filter to a single database.".to_string(), get_schema_schema()),
        tool_def("query", "Unified high-fidelity query tool. Supports property filters, sort orders, limit 100, and presets (active, this_week, this_month, needs_review). Call get_schema first to see available databases and their properties.".to_string(), query_schema),
        tool_def("query_override", "Schema-validated query with AI override. Validates filter property names and types against the database schema before execution. Call get_schema first to see available databases and their properties.".to_string(), query::schema_override(config, schema_cache)),
        tool_def("mutate", "Create, update, delete, or upsert entries across all LifeOS databases. Values auto-map to correct Notion types based on schema.".to_string(), mutate_schema),
        tool_def("intelligence_briefing", "Role-based analysis: CEO, COO, CMO, CRO, CFO, CHO, or module-focused. Returns TOON-encoded analysis. Call get_schema first to see available databases and their properties.".to_string(), intelligence::schema(schema_cache)),
        tool_def("data_science", "Temporal patterns, trajectories, correlations, and weekday profiles. Returns TOON-encoded insights.".to_string(), data_science_schema),
        tool_def("review_pipeline", "Periodic reviews: daily, weekly, monthly, quarterly, journal. Returns TOON-encoded review.".to_string(), review::schema()),
        tool_def("strategic_simulator", "Cross-database strategic analysis: OKR alignment, project health, campaign performance. Returns TOON-encoded analysis.".to_string(), strategic_schema),
        tool_def("sync_note", "Bidirectional Notion ↔ local markdown sync. Returns sync summary in TOON format.".to_string(), sync_note::schema()),
    ]
}

/// Schema for the get_schema tool
fn get_schema_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "database": {
                "type": "string",
                "description": "Optional database name to filter. Omit to return all database schemas."
            }
        }
    })
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
        "get_schema" => {
            let database = args.get("database").and_then(|v| v.as_str());
            Ok(execute_get_schema(database, schema_cache))
        }
        "query" => {
            let params: query::QueryParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid query params: {}", e))?;
            query::execute(&params, config, notion, schema_cache).await
        }
        "query_override" => {
            let params: query::QueryOverrideParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid query_override params: {}", e))?;
            query::execute_override(&params, config, notion, schema_cache).await
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

/// Execute the get_schema tool — returns formatted database schemas
fn execute_get_schema(database: Option<&str>, schema_cache: &SchemaCache) -> String {
    let mut output = String::new();

    for key in schema_cache.db_keys() {
        // If a specific database was requested, skip others
        if let Some(req_db) = database {
            if key != req_db {
                continue;
            }
        }
        let desc = schema_cache.describe_db_properties(key);
        output.push_str(&format!("  {}: {}\n", key, desc));
    }

    if output.is_empty() {
        if let Some(req_db) = database {
            format!("No schema found for database '{}'. Available: {}",
                req_db,
                schema_cache.db_keys().join(", "))
        } else {
            "No database schemas available.".to_string()
        }
    } else {
        format!("Database schemas:\n{}", output)
    }
}
