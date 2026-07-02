//! LifeOS tool implementations — v4 holonic architecture

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
pub mod energy_flow;
pub mod drive_assessment;
pub mod health_metrics;
pub mod shared;
pub mod relations;

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

    let health_schema = health_metrics::schema();
    let drive_schema = drive_assessment::schema();

    vec![
        tool_def("get_schema", "Database schemas by reservoir \u{2192} satellite. Call first.".to_string(), get_schema_schema(config)),
        tool_def("query", "Query any DB with filters, sort, reservoir, or cycle.".to_string(), query_schema),
        tool_def("query_override", "Schema-validated query with AI filter override.".to_string(), query::schema_override(config, schema_cache)),
        tool_def("mutate", "Create/update/delete entries across all databases.".to_string(), mutate_schema),
        tool_def("intelligence_briefing", "Role or cycle briefing (CEO-CFO, lesser/greater/nexus).".to_string(), intelligence::schema(schema_cache)),
        tool_def("data_science", "Temporal patterns, trajectories, correlations.".to_string(), data_science_schema),
        tool_def("review_pipeline", "Daily/weekly/monthly/quarterly reviews.".to_string(), review::schema()),
        tool_def("strategic_simulator", "Cross-DB strategic analysis: OKRs, projects, campaigns.".to_string(), strategic_schema),
        tool_def("sync_note", "Bidirectional Notion ↔ markdown sync.".to_string(), sync_note::schema()),
        tool_def("energy_flow", "Trace currency flow across the holonic spiral.".to_string(), energy_flow::schema(config)),
        tool_def("drive_assessment", "Assess 4 drives at lesser/greater boundary.".to_string(), drive_schema),
        tool_def("health_metrics", "G_z + P_z holonic health metrics.".to_string(), health_schema),
        tool_def("get_page", "Fetch entry with all relations resolved to titles.".to_string(), relations::schema_get_page()),
        tool_def("expand", "Batch resolve relation IDs to titled entries.".to_string(), relations::schema_expand()),
        tool_def("trace", "Follow relations N levels deep from any entry.".to_string(), relations::schema_trace()),
        tool_def("ancestors", "Walk up hierarchy from entry to root.".to_string(), relations::schema_ancestors()),
        tool_def("backlinks", "Find all entries that reference a given page.".to_string(), relations::schema_backlinks()),
        tool_def("link", "Create a relation between two entries.".to_string(), relations::schema_link()),
        tool_def("graph_metrics", "Orphan detection, relation density, broken links.".to_string(), relations::schema_graph_metrics()),
    ]
}

fn get_schema_schema(config: &LifeOSConfig) -> Value {
    let reservoir_keys: Vec<Value> = config.databases.keys().map(|k| Value::String(k.clone())).collect();
    serde_json::json!({
        "type": "object",
        "properties": {
            "database": {
                "type": "string",
                "enum": reservoir_keys,
                "description": "Optional reservoir key to filter. Omit to return all database schemas."
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
            Ok(execute_get_schema(database, schema_cache, config))
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
        "energy_flow" => {
            let params: energy_flow::EnergyFlowParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid energy_flow params: {}", e))?;
            energy_flow::execute(&params, config, notion, schema_cache).await
        }
        "drive_assessment" => {
            let params: drive_assessment::DriveAssessmentParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid drive_assessment params: {}", e))?;
            drive_assessment::execute(&params, config, notion).await
        }
        "health_metrics" => {
            let params: health_metrics::HealthMetricsParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid health_metrics params: {}", e))?;
            health_metrics::execute(&params, config, notion).await
        }
        "get_page" => {
            let params: relations::GetPageParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid get_page params: {}", e))?;
            relations::execute_get_page(&params, config, notion, schema_cache).await
        }
        "expand" => {
            let params: relations::ExpandParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid expand params: {}", e))?;
            relations::execute_expand(&params, config, notion, schema_cache).await
        }
        "trace" => {
            let params: relations::TraceParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid trace params: {}", e))?;
            relations::execute_trace(&params, config, notion, schema_cache).await
        }
        "ancestors" => {
            let params: relations::AncestorsParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid ancestors params: {}", e))?;
            relations::execute_ancestors(&params, config, notion, schema_cache).await
        }
        "backlinks" => {
            let params: relations::BacklinksParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid backlinks params: {}", e))?;
            relations::execute_backlinks(&params, config, notion, schema_cache).await
        }
        "link" => {
            let params: relations::LinkParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid link params: {}", e))?;
            relations::execute_link(&params, config, notion, schema_cache).await
        }
        "graph_metrics" => {
            relations::execute_graph_metrics(config, notion, schema_cache).await
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

/// Execute the get_schema tool — returns hierarchical v4 holonic database schemas
pub fn execute_get_schema(database: Option<&str>, schema_cache: &SchemaCache, config: &LifeOSConfig) -> String {
    let mut output = String::new();

    // Filter to specific reservoir if requested
    let reservoirs: Vec<&String> = if let Some(req_db) = database {
        if config.databases.contains_key(req_db) {
            vec![config.databases.keys().find(|k| *k == req_db).unwrap()]
        } else if let Some(res_key) = schema_cache.reservoir_for(req_db) {
            // Requested a satellite — show its parent reservoir
            vec![config.databases.keys().find(|k| k.as_str() == res_key).unwrap()]
        } else {
            return format!(
                "No reservoir found for '{}'. Available reservoirs: {}",
                req_db,
                config.databases.keys().cloned().collect::<Vec<_>>().join(", ")
            );
        }
    } else {
        config.databases.keys().collect()
    };

    for key in reservoirs {
        let desc = schema_cache.describe_reservoir(key, config);
        output.push_str(&desc);
        output.push('\n');
    }

    if output.is_empty() {
        "No database schemas available.".to_string()
    } else {
        format!("LifeOS v4 Holonic Database Schemas:\n{}", output)
    }
}
