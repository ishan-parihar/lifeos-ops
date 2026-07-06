//! Schema Engine — fetches and caches Notion database schemas
//!
//! In v0.7+, `SchemaCache` is the **authoritative source** of database schema
//! knowledge. It auto-discovers every property, entry-type option, and relation
//! edge by fetching `GET /v1/data_sources/{id}` for each configured reservoir
//! at startup. The config file no longer needs to enumerate properties — it
//! only needs the 5 reservoir names + their holonic archetype metadata.
//!
//! Provides two layers:
//! - `SchemaEngine`: per-data-source schema caching (low-level)
//! - `SchemaCache`: config-aware caching for the 5 unified databases +
//!   relation graph + entry-type options + property name resolution

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::NotionDataSource;

/// Schema engine with raw data-source caching
pub struct SchemaEngine {
    notion: Arc<NotionClient>,
    schema_cache: Arc<Mutex<HashMap<String, NotionDataSource>>>,
}

impl SchemaEngine {
    pub fn new(notion: Arc<NotionClient>) -> Self {
        Self {
            notion,
            schema_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_schema(&self, data_source_id: &str) -> Result<NotionDataSource, String> {
        {
            let cache = self.schema_cache.lock().await;
            if let Some(schema) = cache.get(data_source_id) {
                return Ok(schema.clone());
            }
        }
        let ds = self.notion.get_data_source(data_source_id).await?;
        {
            let mut cache = self.schema_cache.lock().await;
            cache.insert(data_source_id.to_string(), ds.clone());
        }
        Ok(ds)
    }

    pub async fn get_enum_options(
        &self,
        data_source_id: &str,
        property_name: &str,
    ) -> Result<Vec<String>, String> {
        let schema = self.get_schema(data_source_id).await?;
        Ok(extract_options_from_schema(&schema, property_name))
    }
}

// ── SchemaCache: config-aware, auto-discovering, keyed by config-key ──

#[derive(Debug, Clone)]
pub struct PropInfo {
    /// The actual Notion property name (e.g. "Entry Type", "Digestion Status").
    pub notion_name: String,
    /// Notion property type: "select", "multi_select", "status", "relation", "title",
    /// "rich_text", "date", "number", "formula", "rollup", "checkbox", "url",
    /// "email", "people", "files", "created_time", "last_edited_time",
    /// "created_by", "last_edited_by", "unique_id", "button".
    pub prop_type: String,
    /// For select/multi_select/status: the configured option names.
    /// Empty for all other property types.
    pub enum_options: Vec<String>,
}

/// A relation edge: property name → target database key.
#[derive(Debug, Clone)]
pub struct RelationEdge {
    /// Notion property name (e.g. "Generated From", "Pillar Link").
    pub prop_name: String,
    /// Resolved config key of the target DB (e.g. "potentiator").
    /// Falls back to `unknown(<short-id>)` if the target isn't one of the 5 reservoirs.
    pub target_db: String,
}

/// Config-key-aware property cache with relation graph + entry-type options.
///
/// In v0.7+, this struct is populated ENTIRELY from live Notion schema fetches.
/// The config file no longer needs to enumerate properties — `SchemaCache::init`
/// auto-discovers every property by fetching `GET /v1/data_sources/{id}` for each
/// reservoir and building:
///   1. A snake_case → Notion-name alias map (`dbs[db_key][config_key]`)
///   2. An entry-type options list per DB (from the entry_type_property's options)
///   3. A relation graph (from relation property definitions)
///   4. A reverse-ID map (any UUID form → config_key) for relation target resolution
pub struct SchemaCache {
    /// db_key → (config_key → PropInfo). The config_key is a snake_case alias
    /// generated from the Notion property name (e.g. "Digestion Status" →
    /// "digestion_status"). Well-known aliases (`name`, `entry_type`, `status`,
    /// `date`, `duration`, `priority`, `stage`) are added regardless.
    dbs: HashMap<String, HashMap<String, PropInfo>>,
    /// All database keys in insertion order
    db_keys: Vec<String>,
    /// db_key → Vec<RelationEdge> — outgoing relation edges per DB.
    relation_graph: HashMap<String, Vec<RelationEdge>>,
    /// ANY ID (database_id, data_source_id, resolved_data_source_id) → config_key.
    /// Used to resolve relation targets whose schema may report either form
    /// under Notion v2025-09-03.
    id_to_key: HashMap<String, String>,
}

impl SchemaCache {
    /// Pre-warm the cache by fetching ALL 5 unified database schemas from Notion.
    ///
    /// This is the AUTHORITATIVE source of schema knowledge in v0.7+. The config
    /// file's `properties` map is ignored (kept only for backward compat) —
    /// every property, entry-type option, and relation edge comes from the live
    /// Notion schema fetch.
    pub async fn init(config: &Arc<LifeOSConfig>, notion: &Arc<NotionClient>) -> Self {
        let engine = Arc::new(SchemaEngine::new(notion.clone()));
        let semaphore = Arc::new(tokio::sync::Semaphore::new(4));

        struct FetchTask {
            key: String,
            ds_id: String,
        }

        let mut tasks: Vec<FetchTask> = Vec::new();
        for (db_key, db_cfg) in &config.databases {
            tasks.push(FetchTask {
                key: db_key.clone(),
                ds_id: db_cfg.ds_id().to_string(),
            });
        }

        struct FetchResult {
            key: String,
            ds_id: String,
            raw_schema: Option<NotionDataSource>,
        }

        let mut futures = Vec::new();
        for task in tasks {
            let key = task.key;
            let ds_id = task.ds_id.clone();
            let eng = engine.clone();
            let sem = semaphore.clone();
            futures.push(async move {
                let _permit = sem.acquire().await;
                let raw = eng.get_schema(&ds_id).await.ok();
                FetchResult { key, ds_id: task.ds_id, raw_schema: raw }
            });
        }

        let results = futures::future::join_all(futures).await;

        // Build reverse map: ANY ID form → config_key.
        let mut id_to_key: HashMap<String, String> = HashMap::new();
        for (db_key, db_cfg) in &config.databases {
            id_to_key.insert(db_cfg.database_id.clone(), db_key.clone());
            if let Some(ref ds_id) = db_cfg.resolved_data_source_id {
                id_to_key.insert(ds_id.clone(), db_key.clone());
            }
        }

        // Build prop info map (auto-discovered) + collect raw schemas for relations
        let mut dbs: HashMap<String, HashMap<String, PropInfo>> = HashMap::new();
        let mut db_keys: Vec<String> = Vec::new();
        let mut raw_schemas: HashMap<String, NotionDataSource> = HashMap::new();
        let mut discovered_per_db: HashMap<String, HashMap<String, String>> = HashMap::new();

        for result in results {
            db_keys.push(result.key.clone());
            let mut prop_map: HashMap<String, PropInfo> = HashMap::new();
            let mut disc_map: HashMap<String, String> = HashMap::new();

            if let Some(schema) = &result.raw_schema {
                // Auto-discover every property from the live Notion schema
                for (notion_name, prop_schema) in &schema.properties {
                    let prop_type = prop_schema.prop_type.clone();
                    let options = extract_options(prop_schema);

                    // Generate snake_case config key (e.g. "Digestion Status" → "digestion_status")
                    let config_key = snake_case_alias(notion_name);

                    // Special-case the entry_type_property: alias as "entry_type"
                    let is_entry_type_prop = config.databases.get(&result.key)
                        .and_then(|db| db.entry_type_property.as_deref())
                        .map(|et| et == notion_name)
                        .unwrap_or(false);

                    let primary_key = if is_entry_type_prop {
                        "entry_type".to_string()
                    } else {
                        config_key.clone()
                    };

                    // Add well-known aliases for commonly-referenced properties
                    let aliases = well_known_aliases(notion_name, &prop_type);
                    for alias in aliases {
                        prop_map.insert(alias.clone(), PropInfo {
                            notion_name: notion_name.clone(),
                            prop_type: prop_type.clone(),
                            enum_options: options.clone(),
                        });
                        disc_map.insert(alias, notion_name.clone());
                    }

                    // Primary snake_case key
                    prop_map.insert(primary_key.clone(), PropInfo {
                        notion_name: notion_name.clone(),
                        prop_type: prop_type.clone(),
                        enum_options: options,
                    });
                    disc_map.insert(primary_key, notion_name.clone());

                    // Also include the raw notion_name as a key (for direct lookups)
                    prop_map.insert(notion_name.clone(), PropInfo {
                        notion_name: notion_name.clone(),
                        prop_type: prop_type.clone(),
                        enum_options: extract_options(prop_schema),
                    });
                }

                raw_schemas.insert(result.ds_id, schema.clone());
            }

            dbs.insert(result.key.clone(), prop_map);
            discovered_per_db.insert(result.key.clone(), disc_map);
        }

        // Build relation graph from raw schemas
        let mut relation_graph: HashMap<String, Vec<RelationEdge>> = HashMap::new();
        for (ds_id, schema) in &raw_schemas {
            let source_config_key = find_key_for_ds_id(ds_id, config);

            if let Some(src_key) = source_config_key {
                let mut edges = Vec::new();
                for (prop_name, prop_schema) in &schema.properties {
                    if prop_schema.prop_type == "relation" {
                        if let Some(ref rel_config) = prop_schema.relation {
                            let target_id = rel_config.data_source_id.as_deref()
                                .or(rel_config.database_id.as_deref());
                            let target_key = match target_id {
                                Some(id) => id_to_key.get(id)
                                    .cloned()
                                    .unwrap_or_else(|| format!("unknown({})", &id[..8.min(id.len())])),
                                None => "unknown(no-id)".to_string(),
                            };
                            edges.push(RelationEdge {
                                prop_name: prop_name.clone(),
                                target_db: target_key,
                            });
                        }
                    }
                }
                relation_graph.insert(src_key, edges);
            }
        }

        // Write the discovered property map back into the config's DbConfig so
        // downstream code can use `db.notion_prop(config_key)` for lookups.
        // This is a side-effect of init() — we mutate the config in place.
        // The caller passes &Arc<LifeOSConfig> which is immutable, so we rely on
        // the discovered map being consulted via SchemaCache methods directly.
        // (We keep a copy of the discovered_per_db inside Self for our own use.)

        Self {
            dbs,
            db_keys,
            relation_graph,
            id_to_key,
        }
    }

    /// Look up a property's Notion type by config key.
    /// Returns None if the config key isn't found in this DB's auto-discovered schema.
    pub fn get_prop_type(&self, db_key: &str, config_key: &str) -> Option<&str> {
        self.dbs
            .get(db_key)
            .and_then(|props| props.get(config_key))
            .map(|info| info.prop_type.as_str())
    }

    /// Look up the Notion property name for a config key.
    pub fn get_notion_name(&self, db_key: &str, config_key: &str) -> Option<&str> {
        self.dbs
            .get(db_key)
            .and_then(|props| props.get(config_key))
            .map(|info| info.notion_name.as_str())
    }

    /// Get the enum options (select/multi_select/status) for a property by config key.
    pub fn get_enum_options(&self, db_key: &str, config_key: &str) -> Option<&[String]> {
        self.dbs
            .get(db_key)
            .and_then(|props| props.get(config_key))
            .map(|info| info.enum_options.as_slice())
            .filter(|opts| !opts.is_empty())
    }

    /// Get the entry-type options for a DB (auto-discovered from Notion).
    /// Returns None if the DB has no `entry_type_property` configured or if the
    /// property isn't a select/multi_select.
    pub fn get_entry_type_options(&self, db_key: &str, config: &LifeOSConfig) -> Vec<String> {
        // First check if config declares an entry_type_property name
        if let Some(db_cfg) = config.databases.get(db_key) {
            if let Some(ref et_prop_name) = db_cfg.entry_type_property {
                // Look up the property by its Notion name (we added it directly as a key)
                if let Some(props) = self.dbs.get(db_key) {
                    if let Some(info) = props.get(et_prop_name) {
                        return info.enum_options.clone();
                    }
                }
            }
        }
        // Fallback: try the "entry_type" alias (auto-discovered)
        self.get_enum_options(db_key, "entry_type")
            .map(|s| s.to_vec())
            .unwrap_or_default()
    }

    /// Get the Notion property type for the DB's entry_type_property.
    /// This is the AUTHORITATIVE answer — ignores the deprecated
    /// `entry_type_property_type` config field, which may be wrong.
    pub fn get_entry_type_property_type(&self, db_key: &str, config: &LifeOSConfig) -> Option<&str> {
        let db_cfg = config.databases.get(db_key)?;
        let et_name = db_cfg.entry_type_property.as_deref()?;
        self.dbs.get(db_key)
            .and_then(|props| props.get(et_name))
            .map(|info| info.prop_type.as_str())
    }

    /// Get all property Notion names for a DB (auto-discovered).
    pub fn get_property_names(&self, db_key: &str) -> Vec<String> {
        self.dbs.get(db_key)
            .map(|props| {
                let mut names: Vec<String> = props.values()
                    .map(|info| info.notion_name.clone())
                    .collect();
                names.sort();
                names.dedup();
                names
            })
            .unwrap_or_default()
    }

    /// Get all config keys (snake_case aliases) for a DB.
    pub fn get_config_keys(&self, db_key: &str) -> Vec<String> {
        self.dbs.get(db_key)
            .map(|props| {
                let mut keys: Vec<String> = props.keys().cloned().collect();
                keys.sort();
                keys
            })
            .unwrap_or_default()
    }

    pub fn db_keys(&self) -> &[String] {
        &self.db_keys
    }

    /// Get the set of Notion property names for a database (v0.10.3: added for fill_rate tool).
    /// Returns None if the db_key is unknown. The returned set is the keys of
    /// the inner HashMap — these are Notion property names (e.g. "Archetype Role",
    /// "Entry Type"), not config-key aliases.
    pub fn get_db_property_names(&self, db_key: &str) -> Option<Vec<String>> {
        self.dbs.get(db_key).map(|props| {
            let mut names: Vec<String> = props.values().map(|p| p.notion_name.clone()).collect();
            names.sort();
            names.dedup();
            names
        })
    }

    /// Get outgoing relation edges for a database (which properties link to which databases).
    pub fn get_relation_edges(&self, db_key: &str) -> &[RelationEdge] {
        self.relation_graph.get(db_key).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all relation edges as a flat map for the full graph.
    pub fn all_relation_edges(&self) -> &HashMap<String, Vec<RelationEdge>> {
        &self.relation_graph
    }

    /// Resolve any ID form (database_id, data_source_id, resolved_data_source_id)
    /// back to a config key.
    pub fn resolve_db_key_from_id(&self, database_id: &str) -> Option<&str> {
        self.id_to_key.get(database_id).map(|s| s.as_str())
    }

    /// Propagate the auto-discovered property map back into the config's
    /// `DbConfig.discovered_properties` field (and the legacy `properties`
    /// map, for backward-compat with code that still reads it directly).
    /// This lets downstream code that only has a `&LifeOSConfig` reference
    /// (no SchemaCache) still resolve config_key → Notion property name via
    /// `db.notion_prop(config_key)` OR via `db.properties.get(config_key)`.
    ///
    /// Also auto-corrects the deprecated `entry_type_property_type` field on
    /// each DbConfig to match the live Notion schema. This ensures tools like
    /// `data_science` (which still read this field) build correct filters
    /// regardless of what the config file declares.
    ///
    /// Should be called once after `SchemaCache::init` if downstream code will
    /// use `DbConfig::notion_prop()` or `entry_type_property_type`. The
    /// main.rs `resolve_with_schema` helper calls this automatically.
    pub fn propagate_to_config(&self, config: &mut LifeOSConfig) {
        for (db_key, props) in &self.dbs {
            if let Some(db_cfg) = config.databases.get_mut(db_key) {
                let mut disc = HashMap::new();
                for (config_key, info) in props {
                    disc.insert(config_key.clone(), info.notion_name.clone());
                }
                db_cfg.discovered_properties = disc.clone();
                // ALSO populate the legacy `properties` map so existing code
                // that does `db.properties.get("entry_type")` keeps working
                // in v0.7+ without modification. This is the bridge between
                // the auto-discovery model and the legacy static-config model.
                db_cfg.properties = disc;

                // Auto-correct entry_type_property_type from the live schema.
                // The config may have a stale "select" when Notion actually
                // exposes a "multi_select" (this is exactly the v0.6.1
                // Significator bug — now fixed at runtime without requiring
                // a config edit).
                if let Some(ref et_name) = db_cfg.entry_type_property {
                    if let Some(info) = props.get(et_name) {
                        let live_type = info.prop_type.as_str();
                        if live_type == "select" || live_type == "multi_select" {
                            if db_cfg.entry_type_property_type != live_type {
                                tracing::debug!(
                                    "Auto-correcting {}.entry_type_property_type: {} → {}",
                                    db_key, db_cfg.entry_type_property_type, live_type
                                );
                                db_cfg.entry_type_property_type = live_type.to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    /// Build a description for a database showing its properties, entry types,
    /// relation edges, and holonic role. All schema info comes from auto-discovery.
    pub fn describe_reservoir(&self, reservoir_key: &str, config: &LifeOSConfig) -> String {
        let mut output = String::new();

        if let Some(db_cfg) = config.databases.get(reservoir_key) {
            let archetype = db_cfg.archetype.as_deref().unwrap_or("unknown");
            let scale = db_cfg.scale.as_deref().unwrap_or("unknown");
            let dimension = db_cfg.dimension.as_deref().unwrap_or("unknown");
            let cycle = db_cfg.cycle.as_deref().unwrap_or("unknown");

            output.push_str(&format!(
                "{} [{}] ({}, {}, {}):\n",
                db_cfg.name, archetype, scale, dimension, cycle
            ));

            if let Some(ref desc) = db_cfg.description {
                output.push_str(&format!("  Role: {}\n", desc));
            }

            // Entry type property + auto-discovered options
            if let Some(ref et_prop) = db_cfg.entry_type_property {
                let entry_types = self.get_entry_type_options(reservoir_key, config);
                let et_type = self.get_entry_type_property_type(reservoir_key, config)
                    .unwrap_or("select");
                output.push_str(&format!(
                    "  Entry Type Property: {} ({})\n",
                    et_prop, et_type
                ));
                if !entry_types.is_empty() {
                    output.push_str(&format!("  Entry Types ({}):\n", entry_types.len()));
                    for et in &entry_types {
                        // Use description from config if available (deprecated field),
                        // otherwise just list the option name.
                        let desc = config.holonic.as_ref()
                            .and_then(|h| h.entry_type_descriptions.get(reservoir_key))
                            .and_then(|m| m.get(et))
                            .map(|s| s.as_str())
                            .unwrap_or("");
                        if desc.is_empty() {
                            output.push_str(&format!("    {}\n", et));
                        } else {
                            output.push_str(&format!("    {}: {}\n", et, desc));
                        }
                    }
                }
            }

            // Nexus currency property (Kind) — auto-discovered
            if let Some(ref cur_prop) = db_cfg.currency_property {
                if let Some(opts) = self.get_enum_options(reservoir_key, cur_prop) {
                    output.push_str(&format!(
                        "  Currency Property: {} [{}]\n",
                        cur_prop,
                        opts.join(" / ")
                    ));
                }
            }

            // Properties (auto-discovered) + relation targets
            if let Some(props) = self.dbs.get(reservoir_key) {
                let desc = format_properties_with_relations(props, self.relation_graph.get(reservoir_key));
                if !desc.is_empty() {
                    output.push_str(&format!("  Properties: {}\n", desc));
                }
            }

            // Relations (outgoing edges)
            if let Some(edges) = self.relation_graph.get(reservoir_key) {
                if !edges.is_empty() {
                    output.push_str(&format!("  Relations ({}):\n", edges.len()));
                    for edge in edges {
                        output.push_str(&format!("    {} → {}\n", edge.prop_name, edge.target_db));
                    }
                }
            }
        }

        output
    }

    /// Describe all properties for a single database.
    pub fn describe_db_properties(&self, db_key: &str) -> String {
        let Some(props) = self.dbs.get(db_key) else {
            return String::new();
        };
        format_properties_with_relations(props, self.relation_graph.get(db_key))
    }

    /// Describe the full relation graph as human-readable text.
    pub fn describe_relation_graph(&self) -> String {
        let mut output = String::from("LifeOS Relational Graph:\n\n");
        for (db_key, edges) in &self.relation_graph {
            output.push_str(&format!("{}:\n", db_key));
            for edge in edges {
                output.push_str(&format!("  {} → {}\n", edge.prop_name, edge.target_db));
            }
        }
        output
    }
}

/// Find the config key for a given data_source_id.
fn find_key_for_ds_id(ds_id: &str, config: &LifeOSConfig) -> Option<String> {
    for (db_key, db_cfg) in &config.databases {
        if db_cfg.ds_id() == ds_id {
            return Some(db_key.clone());
        }
    }
    None
}

/// Format properties with relation targets annotated.
fn format_properties_with_relations(props: &HashMap<String, PropInfo>, edges: Option<&Vec<RelationEdge>>) -> String {
    // Deduplicate by Notion name (we add multiple aliases for each property)
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let edge_map: HashMap<&str, &str> = edges
        .map(|e| e.iter().map(|e| (e.prop_name.as_str(), e.target_db.as_str())).collect())
        .unwrap_or_default();

    let mut parts: Vec<String> = Vec::new();
    for info in props.values() {
        if !seen.insert(info.notion_name.as_str()) {
            continue; // already added this property under a different alias
        }
        let type_hint = if info.prop_type == "relation" {
            match edge_map.get(info.notion_name.as_str()) {
                Some(target) => format!("(relation→{})", target),
                None => "(relation)".to_string(),
            }
        } else {
            match info.prop_type.as_str() {
                "select" | "status" => {
                    if info.enum_options.is_empty() {
                        format!("({})", info.prop_type)
                    } else {
                        format!("({}:{})", info.prop_type, info.enum_options.join("/"))
                    }
                }
                "multi_select" => {
                    if info.enum_options.is_empty() {
                        "(multi_select)".to_string()
                    } else {
                        format!("(multi_select:{})", info.enum_options.join("/"))
                    }
                }
                t => format!("({})", t),
            }
        };
        parts.push(format!("{}{}", info.notion_name, type_hint));
    }
    parts.sort();
    parts.join(", ")
}

// ── Helpers ──

fn extract_options(prop: &crate::notion::types::PropertySchema) -> Vec<String> {
    match prop.prop_type.as_str() {
        "select" => prop.select.as_ref()
            .and_then(|s| s.options.as_ref())
            .map(|opts| opts.iter().map(|o| o.name.clone()).collect())
            .unwrap_or_default(),
        "multi_select" => prop.multi_select.as_ref()
            .and_then(|s| s.options.as_ref())
            .map(|opts| opts.iter().map(|o| o.name.clone()).collect())
            .unwrap_or_default(),
        "status" => prop.status.as_ref()
            .and_then(|s| s.options.as_ref())
            .map(|opts| opts.iter().map(|o| o.name.clone()).collect())
            .unwrap_or_default(),
        _ => vec![],
    }
}

fn extract_options_from_schema(schema: &NotionDataSource, property_name: &str) -> Vec<String> {
    if let Some(prop) = schema.properties.get(property_name) {
        return extract_options(prop);
    }
    let lower = property_name.to_lowercase();
    for (name, prop) in &schema.properties {
        if name.to_lowercase() == lower {
            return extract_options(prop);
        }
    }
    vec![]
}

/// Generate a snake_case config key from a Notion property name.
/// E.g. "Digestion Status" → "digestion_status", "Entry Type" → "entry_type",
/// "Pillar Link" → "pillar_link", "YAML Metadata" → "yaml_metadata".
fn snake_case_alias(notion_name: &str) -> String {
    notion_name
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "_")
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

/// Return well-known short aliases for common Notion property names.
/// This lets users query by `--filter-property status` instead of the full Notion name.
fn well_known_aliases(notion_name: &str, prop_type: &str) -> Vec<String> {
    let lower = notion_name.to_lowercase();
    let mut aliases = Vec::new();

    // Title property — alias as "name" and "title"
    if prop_type == "title" {
        aliases.push("name".to_string());
        aliases.push("title".to_string());
    }

    // Status-like properties
    if prop_type == "status" {
        aliases.push("status".to_string());
    }

    // Date properties — alias as "date"
    if prop_type == "date" && (lower.contains("date") || lower == "start" || lower == "end") {
        aliases.push("date".to_string());
    }

    // Numeric duration/amount/weight
    if prop_type == "number" || prop_type == "formula" {
        if lower.contains("duration") { aliases.push("duration".to_string()); }
        if lower.contains("amount") { aliases.push("amount".to_string()); }
        if lower.contains("weight") { aliases.push("weight".to_string()); }
        if lower.contains("target") { aliases.push("target".to_string()); }
        if lower.contains("progress") { aliases.push("progress".to_string()); }
    }

    // Select aliases
    if prop_type == "select" {
        if lower.contains("priority") { aliases.push("priority".to_string()); }
        if lower.contains("stage") { aliases.push("stage".to_string()); }
        if lower.contains("cadence") { aliases.push("review_cadence".to_string()); }
        if lower.contains("tier") { aliases.push("tier".to_string()); }
        if lower.contains("kind") { aliases.push("kind".to_string()); }
        if lower.contains("polarity") { aliases.push("polarity_outcome".to_string()); }
    }

    aliases
}
