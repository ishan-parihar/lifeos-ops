//! LifeOS tool implementations — v0.10.0 consolidated holonic architecture

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
pub mod shared;
pub mod relations;
pub mod audit;
pub mod validate_yaml;
pub mod relational_gaps;
pub mod build_context;
pub mod suggest_categorization;
pub mod relational_graph;
pub mod relation_ops;
pub mod workflows;
pub mod auto_enrich;
pub mod fill_rate;
pub mod quick_link;
pub mod morning;
pub mod capture;
pub mod cycle_health;
pub mod trace_trajectory;
pub mod gap_analysis;
pub mod surface_synthesis;

fn enrich_database_param(schema: &mut Value, param_name: &str, schema_cache: &SchemaCache) {
    let db_keys: Vec<Value> = schema_cache.db_keys().iter().map(|k| Value::String(k.clone())).collect();
    if let Some(props) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
        if let Some(param) = props.get_mut(param_name) {
            param["enum"] = Value::Array(db_keys);
        }
    }
}

/// Get tool definitions in MCP format (JSON Schema per tool)
/// v0.10.0: Consolidated from 34 → 28 tools (merged duplicates)
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
        // ── Schema/Query ──
        tool_def("get_schema", "Database schemas with entry types and holonic roles. Call first to understand the 5-DB architecture.".to_string(), get_schema_schema(config)),
        tool_def("query", "Query any of the 5 databases with filters, sort, entry_type, cycle, or AI filter override.".to_string(), query_schema),
        tool_def("mutate", "Create/update/delete entries across all databases.".to_string(), mutate_schema),

        // ── Intelligence ──
        tool_def("intelligence_briefing", "Role or cycle briefing (CEO-CFO, lesser/greater/nexus).".to_string(), intelligence::schema(schema_cache)),
        tool_def("data_science", "Temporal patterns, trajectories, correlations.".to_string(), data_science_schema),
        tool_def("review_pipeline", "Daily/weekly/monthly/quarterly reviews.".to_string(), review::schema()),
        tool_def("strategic_simulator", "Cross-DB strategic analysis: OKRs, projects, campaigns.".to_string(), strategic_schema),
        tool_def("sync_note", "Bidirectional Notion ↔ markdown sync.".to_string(), sync_note::schema()),


        // ── Relational Navigation ──
        tool_def("get_page", "Fetch entry with all relations resolved to titles.".to_string(), relations::schema_get_page()),
        tool_def("build_context", "Assemble complete relational neighborhood for an entry: outgoing + incoming + depth-2 neighborhood + gap analysis. One call replaces 3+.".to_string(), build_context::schema()),
        tool_def("trace", "Follow relations N levels deep from any entry.".to_string(), relations::schema_trace()),
        tool_def("ancestors", "Walk up hierarchy from entry to root.".to_string(), relations::schema_ancestors()),
        tool_def("backlinks", "Find all entries that reference a given page.".to_string(), relations::schema_backlinks()),
        tool_def("relational_graph", "High-level relational graph overview: inter-DB hierarchy tree with link counts.".to_string(), relational_graph::schema()),

        // ── Relational Write (deliberate, no auto-population) ──
        tool_def("link", "Create a relation between two entries.".to_string(), relations::schema_link()),
        tool_def("unlink", "Remove a single relation between two entries.".to_string(), relation_ops::schema_unlink()),
        tool_def("batch_link", "Create multiple relations in one call. Each must be explicitly specified.".to_string(), relation_ops::schema_batch_link()),

        // ── Audit & Validation ──
        tool_def("orphans", "List entries with zero populated relations.".to_string(), audit::schema_orphans()),
        tool_def("relational_gaps", "Surface entries with zero or sparse relations + expected relations that are missing. Read-only.".to_string(), relational_gaps::schema()),
        tool_def("validate_yaml", "Validate entries against the v0.9.0 YAML schema hierarchy. Also checks old Validation formula status.".to_string(), validate_yaml::schema()),
        tool_def("suggest_links", "Suggest likely cross-reservoir links for orphan entries via title similarity.".to_string(), audit::schema_suggest_links()),
        tool_def("suggest_categorization", "Suggest entry-types for uncategorized entries based on title heuristics. Never writes.".to_string(), suggest_categorization::schema()),

        // ── Workflow Commands (v0.10.0) ──
        tool_def("daily", "Run daily review: relational gaps + holonic synthesis + recent entries in one call.".to_string(), workflows::schema_daily()),
        tool_def("dashboard", "LifeOS dashboard: orphan count per DB, recent entries, top gaps, health metrics in one call.".to_string(), workflows::schema_dashboard()),

        // ── Auto-Enrichment (v0.10.2 — suggestion-only) ──
        tool_def("auto_enrich", "READ-ONLY advisor. Scans entries missing universal properties (Archetype Role / Complex / Drive Activation) and reports rule-map suggestions. User applies each manually via `mutate`. Modes: tag (property suggestions), link (parent-relation suggestions).".to_string(), auto_enrich::schema()),

        // ── Fill-Rate Audit (v0.10.3 — U-8) ──
        tool_def("fill_rate", "Audit property fill rates per DB. Reports what % of entries have each property populated. Properties with <5% fill are flagged as YAGNI candidates for deletion. Read-only, data-driven cleanup.".to_string(), fill_rate::schema()),

        // ── Quick Link by title (v0.10.3 — parity + U-1 semantic hints) ──
        tool_def("quick_link", "Link two entries by title (auto-resolves page IDs via fuzzy match). Response includes a per-relation semantic hint explaining what the relation means ontologically. Use this when you know the titles but not the page IDs.".to_string(), quick_link::schema()),

        // ── v4.1 Utility Layer ──
        tool_def("morning", "Aggregated morning view across all 5 DBs: active goals, today tasks, recent logs, recent synthesis, profile gaps. Primary user UX entry point.".to_string(), morning::schema()),
        tool_def("capture", "Quick logging with auto-detection. Pass text; tool detects entry type and creates Logbook entry.".to_string(), capture::schema()),
        tool_def("cycle_health", "Check if the v4.1 causal amplification cycle is running. Reports pull/ground/feedback flow health + recommendations.".to_string(), cycle_health::schema()),
        tool_def("trace_trajectory", "Walk Trajectory parent/child hierarchy from any entry up to Vision-Statement. Returns full chain with layer labels.".to_string(), trace_trajectory::schema()),
        tool_def("gap_analysis", "Compare Profile vs Vision to show gaps between current state and ideal-future.".to_string(), gap_analysis::schema()),
        tool_def("surface_synthesis", "Scan recent Logbook entries for patterns. Suggests Synthesis entries to create. Activates ground-truth flow.".to_string(), surface_synthesis::schema()),
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
/// v0.10.0: Consolidated — old tool names are kept as backward-compatible aliases
pub async fn call_tool(
    name: &str,
    args: &Value,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    schema_cache: &SchemaCache,
) -> Result<String, String> {
    match name {
        // ── Schema/Query ──
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
            // Backward-compatible alias — delegates to query with override
            let params: query::QueryOverrideParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid query_override params: {}", e))?;
            query::execute_override(&params, config, notion, schema_cache).await
        }
        "mutate" => {
            let params: mutate::MutateParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid mutate params: {}", e))?;
            mutate::execute(&params, config, notion, schema_cache).await
        }

        // ── Intelligence ──
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

        // ── Relational Navigation ──
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
        "build_context" => {
            let params: build_context::BuildContextParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid build_context params: {}", e))?;
            build_context::execute(&params, config, notion, schema_cache).await
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
        "graph_metrics" => {
            relations::execute_graph_metrics(config, notion, schema_cache).await
        }
        "relational_graph" => {
            let params: relational_graph::RelationalGraphParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid relational_graph params: {}", e))?;
            relational_graph::execute(&params, config, notion, schema_cache).await
        }

        // ── Relational Write ──
        "link" => {
            let params: relations::LinkParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid link params: {}", e))?;
            relations::execute_link(&params, config, notion, schema_cache).await
        }
        "unlink" => {
            let params: relation_ops::UnlinkParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid unlink params: {}", e))?;
            relation_ops::execute_unlink(&params, config, notion, schema_cache).await
        }
        "batch_link" => {
            let params: relation_ops::BatchLinkParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid batch_link params: {}", e))?;
            relation_ops::execute_batch_link(&params, config, notion, schema_cache).await
        }

        // ── Audit & Validation ──
        "orphans" => {
            let params: audit::OrphansParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid orphans params: {}", e))?;
            audit::execute_orphans(&params, config, notion, schema_cache).await
        }
        "validate" => {
            // Backward-compatible alias — delegates to old audit::validate
            let params: audit::ValidateParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid validate params: {}", e))?;
            audit::execute_validate(&params, config, notion, schema_cache).await
        }
        "validate_yaml" => {
            let params: validate_yaml::ValidateYamlParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid validate_yaml params: {}", e))?;
            validate_yaml::execute(&params, config, notion, schema_cache).await
        }
        "relational_gaps" => {
            let params: relational_gaps::RelationalGapsParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid relational_gaps params: {}", e))?;
            relational_gaps::execute(&params, config, notion, schema_cache).await
        }
        "suggest_links" => {
            let params: audit::SuggestLinksParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid suggest_links params: {}", e))?;
            audit::execute_suggest_links(&params, config, notion, schema_cache).await
        }
        "suggest_categorization" => {
            let params: suggest_categorization::SuggestCategorizationParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid suggest_categorization params: {}", e))?;
            suggest_categorization::execute(&params, config, notion, schema_cache).await
        }

        // ── Workflow Commands (v0.10.0) ──
        "daily" => {
            workflows::execute_daily(config, notion, schema_cache).await
        }
        "dashboard" => {
            workflows::execute_dashboard(config, notion, schema_cache).await
        }

        // ── Auto-Enrichment (v0.10.2) ──
        "auto_enrich" => {
            let params: auto_enrich::AutoEnrichParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid auto_enrich params: {}", e))?;
            auto_enrich::execute(&params, config, notion, schema_cache).await
        }

        // ── Fill-Rate Audit (v0.10.3) ──
        "fill_rate" => {
            let params: fill_rate::FillRateParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid fill_rate params: {}", e))?;
            fill_rate::execute(&params, config, notion, schema_cache).await
        }

        // ── Quick Link by title (v0.10.3) ──
        "quick_link" => {
            let params: quick_link::QuickLinkParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid quick_link params: {}", e))?;
            quick_link::execute(&params, config, notion, schema_cache).await
        }

        // ── v4.1 Utility Layer ──
        "morning" => {
            morning::execute(config, notion, schema_cache).await
        }
        "capture" => {
            let params: capture::CaptureParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid capture params: {}", e))?;
            capture::execute(&params, config, notion, schema_cache).await
        }
        "cycle_health" => {
            cycle_health::execute(config, notion, schema_cache).await
        }
        "trace_trajectory" => {
            let params: trace_trajectory::TraceTrajectoryParams = serde_json::from_value(args.clone())
                .map_err(|e| format!("Invalid trace_trajectory params: {}", e))?;
            trace_trajectory::execute(&params, config, notion, schema_cache).await
        }
        "gap_analysis" => {
            gap_analysis::execute(config, notion, schema_cache).await
        }
        "surface_synthesis" => {
            surface_synthesis::execute(config, notion, schema_cache).await
        }

        _ => Err(format!("Unknown tool: {}", name)),
    }
}

/// Execute the get_schema tool — returns the 5 unified databases with entry types and holonic roles
pub fn execute_get_schema(database: Option<&str>, schema_cache: &SchemaCache, config: &LifeOSConfig) -> String {
    let mut output = String::new();

    let databases: Vec<&String> = if let Some(req_db) = database {
        if config.databases.contains_key(req_db) {
            vec![config.databases.keys().find(|k| *k == req_db).unwrap()]
        } else {
            return format!(
                "No database found for '{}'. Available databases: {}",
                req_db,
                config.databases.keys().cloned().collect::<Vec<_>>().join(", ")
            );
        }
    } else {
        config.databases.keys().collect()
    };

    for key in databases {
        let desc = schema_cache.describe_reservoir(key, config);
        output.push_str(&desc);
        output.push('\n');
    }

    if output.is_empty() {
        "No database schemas available.".to_string()
    } else {
        format!("LifeOS v5 — The 5-DB Holonic Architecture:\n\nEach database stores a specific currency in the energy-flow spiral. Entries are discriminated by Entry Type / Item Type / Category properties within each DB.\n\n{}", output)
    }
}
