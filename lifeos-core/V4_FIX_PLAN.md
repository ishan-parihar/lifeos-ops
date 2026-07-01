# LifeOS v4 — Bug Fix & Architecture Alignment Plan

> **For agentic workers:** Execute this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix all remaining bugs from the v4 upgrade and ensure full architectural alignment with `LifeOS_v4_Architecture.md`.

**Architecture:** The v4 holonic architecture has 5 reservoirs (Matrix, Potentiator, Significator, GreatWay, Nexus) with nested satellites. 4 currencies flow through a spiral. All 4 drives operate at both contact boundaries symmetrically. The fix plan addresses: (1) broken sync pipeline for satellites, (2) drive assessment symmetrical model, (3) energy flow tracing improvements, (4) Nexus transmutation analysis, and (5) documentation.

**Tech Stack:** Rust, Notion API (2025-09-03), tokio, serde_json, chrono

## Global Constraints
- Rust edition 2021, `cargo check` must pass with zero errors
- Follow existing code conventions (TOON encoding, config-key → Notion-name mapping)
- All tools return TOON-encoded output
- All new code uses `resolve_db` for database resolution (not `get_db` for satellite keys)

---

## Priority 1: Critical — Sync Pipeline (Satellites Unreachable)

### Task 1: Make `pull.rs` Reservoir-Aware

**Problem:** `pull_database_since` only resolves from `config.databases.get(db_key)`, which only contains the 5 core reservoirs. Satellites are never pulled. Running `lifeos pull` syncs 5 DBs; the other 25 satellite DBs are silently ignored.

**Files:**
- Modify: `lifeos-core/src/sync/pull.rs`
- Modify: `lifeos/src/main.rs` (cmd_pull to pass satellite keys)

**Interfaces:**
- Consumes: `crate::config::resolve_db`, `crate::config::ResolvedDb`
- Produces: Updated `PullReport` that includes satellite pages in counts

- [ ] **Step 1: Update `pull_database_since` to use `resolve_db`**

Replace the opening lookup in `pull_database_since` (lines ~62-64):

```rust
// OLD:
let db = config
    .databases
    .get(db_key)
    .ok_or_else(|| format!("Database key '{}' not found in config", db_key))?;
```

With:

```rust
// NEW: Use resolve_db to support both reservoir and satellite keys
let (ds_id, db_name, properties) = match crate::config::resolve_db(config, db_key) {
    Some(crate::config::ResolvedDb::Reservoir(_key, db)) => {
        (db.ds_id().to_string(), db.name.clone(), db.properties.clone())
    }
    Some(crate::config::ResolvedDb::Satellite(_, _, sat)) => {
        (sat.ds_id().to_string(), sat.name.clone(), sat.properties.clone())
    }
    None => return Err(format!("Database key '{}' not found in config", db_key)),
};
```

- [ ] **Step 2: Replace all `db.ds_id()` and `db.properties` references in the function**

Replace `db.ds_id()` → `&ds_id` and `db.properties` → `&properties` throughout the function. The key line:

```rust
let pages = notion
    .query_data_source_all_since(&ds_id, since)
    .await?;
```

And in the frontmatter extraction:

```rust
let frontmatter_yaml = match extract_properties_yaml(page, &properties, &title_cache) {
```

- [ ] **Step 3: Update `cmd_pull` in `main.rs` to iterate satellites**

In `lifeos/src/main.rs`, the `cmd_pull` function (around line 143) only collects reservoir keys. Change the `db_keys` collection to also include satellite keys when the user doesn't filter by specific keys:

```rust
// After the existing db_keys collection, add satellite expansion
let db_keys: Vec<&String> = if let Some(filter) = db_filter {
    // User specified explicit keys — resolve each (may be reservoir or satellite)
    filter
        .split(',')
        .map(|s| s.trim().to_string())
        .filter_map(|k| {
            if crate::config::resolve_db(config, &k).is_some() {
                // We need owned keys; use a different approach
                // Actually, we need to expand to satellite keys
                None // placeholder — see full implementation below
            } else {
                tracing::warn!("Unknown database key: {k}");
                None
            }
        })
        .collect()
} else {
    // No filter — expand all reservoirs to include their satellites
    let mut keys = Vec::new();
    for (key, db) in &config.databases {
        keys.push(key);
        for sat_key in db.satellites.keys() {
            keys.push(sat_key);
        }
    }
    keys
};
```

**Note:** The exact implementation needs care with lifetimes. The simplest approach: change `db_keys` from `Vec<&String>` to `Vec<String>` and adjust the filter/exclude logic accordingly.

- [ ] **Step 4: Typecheck**

Run: `cargo check 2>&1`
Expected: Clean compile, zero errors

- [ ] **Step 5: Commit**

```bash
git add lifeos-core/src/sync/pull.rs lifeos/src/main.rs
git commit -m "fix(sync): make pull reservoir-aware — sync satellites alongside reservoirs"
```

---

### Task 2: Make `push.rs` Reservoir-Aware

**Problem:** `push_database` only resolves from `config.databases.get(db_key)`, ignoring satellites. Pushed entries only go to reservoir DBs.

**Files:**
- Modify: `lifeos-core/src/sync/push.rs`
- Modify: `lifeos/src/main.rs` (cmd_push to pass satellite keys)

**Interfaces:**
- Consumes: `crate::config::resolve_db`, `crate::config::ResolvedDb`
- Produces: Updated `PushReport` that includes satellite page counts

- [ ] **Step 1: Update `push_database` to use `resolve_db`**

Replace the opening lookup in `push_database` (lines ~191-193):

```rust
// OLD:
let db = config
    .databases
    .get(db_key)
    .ok_or_else(|| format!("Database key '{}' not found in config", db_key))?;
```

With:

```rust
// NEW: Use resolve_db to support both reservoir and satellite keys
let (ds_id, db_name, properties) = match crate::config::resolve_db(config, db_key) {
    Some(crate::config::ResolvedDb::Reservoir(_key, db)) => {
        (db.ds_id().to_string(), db.name.clone(), db.properties.clone())
    }
    Some(crate::config::ResolvedDb::Satellite(_, _, sat)) => {
        (sat.ds_id().to_string(), sat.name.clone(), sat.properties.clone())
    }
    None => return Err(format!("Database key '{}' not found in config", db_key)),
};
```

- [ ] **Step 2: Replace all `db.ds_id()` and `db.properties` references**

Replace:
- `db.ds_id()` → `&ds_id` in `push_created_page` call
- `db.properties` → `&properties` in `push_updated_page` and `push_created_page` calls
- `db.name` → `&db_name` in logging

The key call becomes:

```rust
match push_created_page(
    notion,
    &ds_id,
    &title,
    frontmatter,
    body,
    &properties,
    dry_run,
)
.await
```

- [ ] **Step 3: Update `cmd_push` in `main.rs` to iterate satellites**

Same pattern as Task 1 Step 3 — expand `db_keys` to include satellite keys:

```rust
// No filter — expand all reservoirs to include their satellites
let mut keys = Vec::new();
for (key, db) in &config.databases {
    keys.push(key);
    for sat_key in db.satellites.keys() {
        keys.push(sat_key);
    }
}
keys
```

Change `db_keys` type from `Vec<&String>` to `Vec<String>` to accommodate both reservoir and satellite keys.

- [ ] **Step 4: Typecheck**

Run: `cargo check 2>&1`
Expected: Clean compile, zero errors

- [ ] **Step 5: Commit**

```bash
git add lifeos-core/src/sync/push.rs lifeos/src/main.rs
git commit -m "fix(sync): make push reservoir-aware — push satellites alongside reservoirs"
```

---

## Priority 2: Architecture Alignment — Symmetrical Drive Model

### Task 3: Fix `drive_assessment.rs` — All 4 Drives at Both Boundaries

**Problem:** Per `LifeOS_v4_Architecture.md` §1.4: *"All four drives — Agency, Communion, Eros, Agape — operate at BOTH contact boundaries."* The current implementation separates them: Agency/Communion for lesser, Eros/Agape for greater. This is architecturally incorrect.

**Files:**
- Modify: `lifeos-core/src/tools/drive_assessment.rs`

**Interfaces:**
- Consumes: `crate::config::get_db`, `NotionClient::query_data_source`
- Produces: Updated TOON output with all 4 drives assessed at each boundary

**Architecture Reference (§1.4 + §2.3):**
- **Agency (A_z):** Boundary resistance — protects state from perturbation. Measured by: how many entries have "Active" status (resisting archival) vs total entries.
- **Communion (C_z):** Field conductance — enables coupling with environment. Measured by: how many entries have relations/connections to other DBs.
- **Eros (P_z):** Greater cycle tension — drive toward evolutionary restructuring. Measured by: G-to-S activity ratio + Nexus activity.
- **Agape (G_z):** Lesser cycle coherence — integrative metabolism. Measured by: Matrix-Potentiator balance + digestion health.

- [ ] **Step 1: Rewrite `execute` to assess all 4 drives at each boundary**

Replace the entire `execute` function body with:

```rust
pub async fn execute(
    params: &DriveAssessmentParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    let range = params.range.as_deref().unwrap_or("this_week");
    let date_filter = build_date_filter(range);

    // Query all 4 core reservoirs
    let matrix_stats = query_reservoir_stats(config, notion, "matrix", &date_filter).await;
    let potentiator_stats = query_reservoir_stats(config, notion, "potentiator", &date_filter).await;
    let significator_stats = query_reservoir_stats(config, notion, "significator", &date_filter).await;
    let greatway_stats = query_reservoir_stats(config, notion, "greatway", &date_filter).await;
    let nexus_stats = query_reservoir_stats(config, notion, "nexus", &date_filter).await;

    // Calculate health metrics
    let g_z = calculate_g_z(&matrix_stats, &potentiator_stats);
    let p_z = calculate_p_z(&significator_stats, &greatway_stats, &nexus_stats);

    // Calculate drive scores at each boundary
    // Lesser boundary (Matrix ⇄ Potentiator via Nexus)
    let lesser_agency = calculate_agency(&matrix_stats, &potentiator_stats);
    let lesser_communion = calculate_communion(&matrix_stats, &potentiator_stats);
    let lesser_eros = g_z; // Eros at lesser = coherence tension
    let lesser_agape = g_z; // Agape at lesser = integrative coherence

    // Greater boundary (Significator ⇄ GreatWay via Nexus)
    let greater_agency = calculate_agency(&significator_stats, &greatway_stats);
    let greater_communion = calculate_communion(&significator_stats, &greatway_stats);
    let greater_eros = p_z; // Eros at greater = transcendental tension
    let greater_agape = p_z; // Agape at greater = cross-scale coherence

    let mut result = serde_json::json!({
        "analysis": "drive_assessment",
        "boundary": params.boundary,
        "range": range,
        "lesser_boundary": {
            "matrix": matrix_stats,
            "potentiator": potentiator_stats,
            "G_z_coherence": g_z,
            "drives": {
                "Agency": { "score": lesser_agency, "assessment": format_assessment("Agency", lesser_agency) },
                "Communion": { "score": lesser_communion, "assessment": format_assessment("Communion", lesser_communion) },
                "Eros": { "score": lesser_eros, "assessment": format_assessment("Eros", lesser_eros) },
                "Agape": { "score": lesser_agape, "assessment": format_assessment("Agape", lesser_agape) }
            }
        },
        "greater_boundary": {
            "significator": significator_stats,
            "greatway": greatway_stats,
            "P_z_tension": p_z,
            "drives": {
                "Agency": { "score": greater_agency, "assessment": format_assessment("Agency", greater_agency) },
                "Communion": { "score": greater_communion, "assessment": format_assessment("Communion", greater_communion) },
                "Eros": { "score": greater_eros, "assessment": format_assessment("Eros", greater_eros) },
                "Agape": { "score": greater_agape, "assessment": format_assessment("Agape", greater_agape) }
            }
        },
        "nexus": nexus_stats,
        "overall_health": {
            "G_z": g_z,
            "P_z": p_z,
            "total_health": (g_z + p_z) / 2.0,
            "balance": if (g_z - p_z).abs() < 20.0 { "balanced" } else if g_z > p_z { "grounded" } else { "aspirational" }
        }
    });

    // Filter by requested boundary
    if params.boundary == "lesser" {
        result.as_object_mut().unwrap().remove("greater_boundary");
        result.as_object_mut().unwrap().remove("nexus");
    } else if params.boundary == "greater" {
        result.as_object_mut().unwrap().remove("lesser_boundary");
        result.as_object_mut().unwrap().remove("nexus");
    }

    Ok(crate::toon_format::encode(&result))
}
```

- [ ] **Step 2: Add drive calculation functions**

Add these new functions after the existing `calculate_p_z`:

```rust
/// Agency (A_z): Boundary resistance — how well the holon protects its state from perturbation.
/// Measured by: ratio of "Active" entries (resisting change) to total entries.
fn calculate_agency(left: &serde_json::Value, right: &serde_json::Value) -> f64 {
    let l_total = left.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let r_total = right.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let l_active = left.get("status_distribution")
        .and_then(|d| d.get("Active"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let r_active = right.get("status_distribution")
        .and_then(|d| d.get("Active"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let total = l_total + r_total;
    if total == 0.0 { return 50.0; }
    let active_ratio = (l_active + r_active) / total;
    // Higher active ratio = stronger boundary = higher Agency
    (active_ratio * 60.0 + (total.min(20.0) / 20.0) * 40.0).min(100.0)
}

/// Communion (C_z): Field conductance — how well the holon couples with its environment.
/// Measured by: diversity of status values (more diversity = more engagement).
fn calculate_communion(left: &serde_json::Value, right: &serde_json::Value) -> f64 {
    let mut all_statuses: std::collections::HashSet<String> = std::collections::HashSet::new();
    for source in [left, right] {
        if let Some(dist) = source.get("status_distribution").and_then(|d| d.as_object()) {
            for key in dist.keys() {
                all_statuses.insert(key.clone());
            }
        }
    }
    let l_total = left.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let r_total = right.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total = l_total + r_total;
    if total == 0.0 { return 50.0; }
    // Shannon-like diversity: more unique statuses = more conductance
    let diversity = all_statuses.len() as f64;
    let diversity_score = (diversity / 5.0).min(1.0); // normalize: 5+ statuses = max
    let volume_score = (total.min(20.0) / 20.0);
    (diversity_score * 50.0 + volume_score * 50.0).min(100.0)
}
```

- [ ] **Step 3: Update `calculate_p_z` to include nexus stats**

Replace the existing `calculate_p_z` with:

```rust
fn calculate_p_z(significator: &serde_json::Value, greatway: &serde_json::Value, nexus: &serde_json::Value) -> f64 {
    let s_count = significator.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let g_count = greatway.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let n_count = nexus.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    if s_count + g_count == 0.0 { return 50.0; }
    let activity_ratio = if s_count > 0.0 { g_count / s_count } else { 0.0 };
    let strategic_score = (activity_ratio.min(3.0) / 3.0) * 40.0;
    let nexus_score = (n_count.min(20.0) / 20.0) * 30.0;
    let volume_score = ((s_count + g_count).min(30.0) / 30.0) * 30.0;
    (strategic_score + nexus_score + volume_score).min(100.0)
}
```

- [ ] **Step 4: Update `query_reservoir_stats` to handle nexus (no date property)**

The nexus DB may not have a `date` property. The current code already handles this gracefully (the `if let Some(date_prop)` guard), but add a comment:

```rust
// Nexus and some reservoirs may not have a date property — skip date filter if absent
```

- [ ] **Step 5: Typecheck**

Run: `cargo check 2>&1`
Expected: Clean compile

- [ ] **Step 6: Commit**

```bash
git add lifeos-core/src/tools/drive_assessment.rs
git commit -m "fix(architecture): assess all 4 drives at both boundaries per symmetrical model"
```

---

## Priority 3: Energy Flow Tracing Improvements

### Task 4: Add `entry_id` parameter and cross-reservoir relation tracing to `energy_flow`

**Problem:** The plan specified an `entry_id` parameter for tracing specific entries, and cross-reservoir relationship tracing via relations. Neither is implemented.

**Files:**
- Modify: `lifeos-core/src/tools/energy_flow.rs`

- [ ] **Step 1: Add `entry_id` parameter to `EnergyFlowParams` and schema**

Add to the struct:

```rust
#[derive(Debug, Deserialize)]
pub struct EnergyFlowParams {
    pub scope: String,
    pub currency: Option<String>,
    /// Optional specific entry ID to trace across reservoirs
    pub entry_id: Option<String>,
    pub limit: Option<u32>,
}
```

Update schema to include:

```rust
"entry_id": { "type": "string", "description": "Optional specific entry ID to trace across reservoirs" }
```

- [ ] **Step 2: Implement entry tracing when `entry_id` is provided**

At the top of `execute`, add:

```rust
// If tracing a specific entry, fetch it and show its relations
if let Some(ref entry_id) = params.entry_id {
    return trace_entry(entry_id, config, notion, schema_cache).await;
}
```

Add the `trace_entry` function:

```rust
async fn trace_entry(
    entry_id: &str,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
    _schema_cache: &SchemaCache,
) -> Result<String, String> {
    // Fetch the page to find which reservoir it belongs to
    let page = notion.get_page(entry_id).await?;
    let title = crate::transform::extract_title(&page);

    // Determine which reservoir owns this page by checking parent database
    let mut owner_reservoir = "unknown".to_string();
    let mut owner_archetype = "unknown".to_string();
    for (key, db) in &config.databases {
        // Check if this page's parent data_source_id matches
        if page.parent.as_ref().and_then(|p| p.get("data_source_id"))
            .and_then(|v| v.as_str())
            .map(|id| id == db.ds_id())
            .unwrap_or(false)
        {
            owner_reservoir = key.clone();
            owner_archetype = db.archetype.as_deref().unwrap_or("unknown").to_string();
            break;
        }
        // Check satellites
        for (sat_key, sat) in &db.satellites {
            if page.parent.as_ref().and_then(|p| p.get("data_source_id"))
                .and_then(|v| v.as_str())
                .map(|id| id == sat.ds_id())
                .unwrap_or(false)
            {
                owner_reservoir = format!("{}→{}", key, sat_key);
                owner_archetype = sat.role.as_deref().unwrap_or("satellite").to_string();
                break;
            }
        }
    }

    // Get relations from page properties
    let mut relations: Vec<serde_json::Value> = Vec::new();
    for (prop_name, prop_value) in &page.properties {
        if let crate::notion::types::PropertyValue::Relation { relation, .. } = prop_value {
            for rel in relation {
                if let Some(rel_id) = rel.get("id").and_then(|v| v.as_str()) {
                    relations.push(serde_json::json!({
                        "property": prop_name,
                        "target_id": rel_id
                    }));
                }
            }
        }
    }

    let mut result = serde_json::json!({
        "analysis": "entry_trace",
        "entry": {
            "id": entry_id,
            "title": title,
            "owner": owner_reservoir,
            "archetype": owner_archetype,
        },
        "relations": relations,
        "relation_count": relations.len(),
        "spiral_position": {
            "description": format!("This entry lives in {} ({}) — part of the holonic spiral", owner_reservoir, owner_archetype),
        }
    });

    Ok(crate::toon_format::encode(&result))
}
```

- [ ] **Step 3: Typecheck**

Run: `cargo check 2>&1`
Expected: Clean compile

- [ ] **Step 4: Commit**

```bash
git add lifeos-core/src/tools/energy_flow.rs
git commit -m "feat(energy_flow): add entry_id tracing and cross-reservoir relation discovery"
```

---

## Priority 4: Nexus Transmutation Analysis

### Task 5: Enhance `nexus` briefing mode with transmutation analysis

**Problem:** Per `LifeOS_v4_Architecture.md` §2.2, Nexus is where all 4 currencies are *transmuted*. The current nexus briefing just queries entries without analyzing currency flow patterns.

**Files:**
- Modify: `lifeos-core/src/tools/intelligence.rs` (nexus briefing mode)

- [ ] **Step 1: Replace the nexus briefing arm with transmutation analysis**

Replace the `"nexus"` arm in `execute` with:

```rust
"nexus" => {
    let mut data = serde_json::json!({
        "briefing_type": "nexus",
        "description": "Contact-boundary transmutation: all 4 currencies (Catalyst, Experience, Transformation, Choice)",
        "range": range
    });
    let mut errors: Vec<String> = Vec::new();

    // Query Nexus reservoir
    if let Some(db) = crate::config::get_db(config, "nexus") {
        let mut query = serde_json::json!({ "page_size": 20 });
        if let Some(date_prop) = db.properties.get("date") {
            if let Some(df) = build_date_filter(range, date_prop) {
                query["filter"] = df;
            }
        }
        match notion.query_data_source(db.ds_id(), &query).await {
            Ok(result) => {
                let items: Vec<serde_json::Value> = result.results.iter()
                    .map(|p| {
                        let title = crate::transform::extract_title(p);
                        let category = crate::transform::extract_string(p, "Category");
                        let log_type = crate::transform::extract_string(p, "Log Type");
                        serde_json::json!({
                            "title": title,
                            "id": p.id,
                            "category": category,
                            "log_type": log_type
                        })
                    }).collect();

                // Analyze transmutation patterns
                let mut category_dist: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
                let mut log_type_dist: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
                for item in &items {
                    let cat = item["category"].as_str().unwrap_or("unknown");
                    let lt = item["log_type"].as_str().unwrap_or("unknown");
                    *category_dist.entry(cat.to_string()).or_insert(0) += 1;
                    *log_type_dist.entry(lt.to_string()).or_insert(0) += 1;
                }

                data["nexus"] = serde_json::json!({
                    "entries": items,
                    "count": items.len(),
                    "transmutation_analysis": {
                        "category_distribution": category_dist,
                        "log_type_distribution": log_type_dist,
                        "currencies_active": ["Catalyst", "Experience", "Transformation", "Choice"],
                        "interpretation": nexus_interpretation(items.len(), &category_dist)
                    }
                });
            }
            Err(e) => { data["_errors"] = serde_json::json!([e]); }
        }
    }

    // Also query nexus satellites for completeness
    if let Some(db) = crate::config::get_db(config, "nexus") {
        for (sat_key, sat_cfg) in &db.satellites {
            let query = serde_json::json!({ "page_size": 10 });
            if let Ok(result) = notion.query_data_source(sat_cfg.ds_id(), &query).await {
                let count = result.results.len();
                data["satellites"][sat_key] = serde_json::json!({
                    "name": sat_cfg.name,
                    "role": sat_cfg.role,
                    "entry_count": count
                });
            }
        }
    }

    Ok(crate::toon_format::encode(&data))
}
```

- [ ] **Step 2: Add `nexus_interpretation` helper function**

Add after the existing `build_date_filter`:

```rust
fn nexus_interpretation(count: usize, categories: &std::collections::HashMap<String, i64>) -> String {
    if count == 0 {
        "No nexus entries in range — contact-boundary is dormant".to_string()
    } else if count > 20 {
        format!("High nexus activity ({count} entries) — active transmutation across all 4 currencies")
    } else {
        let dominant = categories.iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| format!("{} ({})", k, v))
            .unwrap_or_else(|| "unknown".to_string());
        format!("Moderate nexus activity ({count} entries) — dominant category: {}", dominant)
    }
}
```

- [ ] **Step 3: Typecheck**

Run: `cargo check 2>&1`
Expected: Clean compile

- [ ] **Step 4: Commit**

```bash
git add lifeos-core/src/tools/intelligence.rs
git commit -m "feat(intelligence): nexus briefing now analyzes currency transmutation patterns"
```

---

## Priority 5: Documentation

### Task 6: Add v4 Architecture Section to README.md

**Problem:** No documentation explaining the v4 holonic architecture, the 5 DBs, currencies, drives, or how MCP tools map to the architecture.

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add v4 Architecture section to README.md**

Add the following section after the existing content:

```markdown
## LifeOS v4 — Holonic Architecture

LifeOS operationalizes the HoloOS holonic architecture into 5 Notion databases organized as **4 reservoirs + 1 contact-boundary**.

### The 5 Reservoirs

| DB | Archetype | Scale | Currency In → Out |
|---|---|---|---|
| **Matrix** | Matrix | Current-stage, Intra-holonic | Catalyst → Experience |
| **Potentiator** | Potentiator | Current-stage, Extra-holonic | Experience → Catalyst |
| **Significator** | Significator | All-stage, Intra-holonic | Transformation → Choice |
| **GreatWay** | Great Way | All-stage, Extra-holonic | Choice → Transformation |
| **Nexus** | Transformation | Contact-boundary | ALL 4 (transmuted) |

### The 4 Currencies

- **Catalyst** (extra → intra): Perturbation ingested by Matrix
- **Experience** (intra → extra): Update generated by Matrix, ingested by Potentiator
- **Transformation** (extra → intra): Frame-change ingested by Significator
- **Choice** (intra → extra): Direction emitted by Significator, ingested by Great Way

### The 4 Drives

All four drives operate at **both** contact boundaries (Matrix⇌Potentiator AND Significator⇌GreatWay):

- **Agency (A_z)**: Boundary resistance — protects state from perturbation
- **Communion (C_z)**: Field conductance — enables coupling with environment
- **Eros (P_z)**: Transcendental tension — drive toward evolutionary restructuring
- **Agape (G_z)**: Integrative coherence — how well the holon metabolizes novelty

### Health Metrics

- **G_z** (lesser cycle): Integrative coherence of Matrix⇌Potentiator — target >70
- **P_z** (greater cycle): Transcendental tension of Significator⇌GreatWay — target >70
- **G_z × P_z** = Total metabolic health (both required, neither sufficient alone)

### MCP Tools

| Tool | Purpose |
|---|---|
| `get_schema` | Hierarchical database schemas by reservoir → satellite |
| `query` | Unified query with reservoir/cycle support |
| `query_override` | Schema-validated query with AI override |
| `mutate` | Create/update/delete entries across all DBs |
| `intelligence_briefing` | Role-based + holonic cycle briefings |
| `data_science` | Temporal patterns, trajectories, correlations |
| `review_pipeline` | Daily/weekly/monthly/quarterly reviews |
| `strategic_simulator` | Cross-database strategic analysis |
| `sync_note` | Bidirectional Notion ↔ markdown sync |
| `energy_flow` | Trace currency flow across the holonic spiral |
| `drive_assessment` | Evaluate all 4 drives at each boundary |
| `health_metrics` | Calculate G_z and P_z health metrics |
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add v4 holonic architecture section to README"
```

---

## Final Validation

### Task 7: Full Typecheck + Code Review

- [ ] **Step 1: Run full cargo check**

```bash
cargo check 2>&1
```

Expected: Zero errors, minimal warnings

- [ ] **Step 2: Spawn code-reviewer-mimo**

Review all changes across Tasks 1-6 for:
- Correct use of `resolve_db` vs `get_db`
- Lifetime safety in pull/push changes
- Architectural alignment with symmetrical drive model
- TOON encoding consistency
- No dead code or unused imports

- [ ] **Step 3: Fix any issues found by reviewer**

- [ ] **Step 4: Final commit if needed**

```bash
git add -A && git commit -m "fix: v4 architecture alignment — sync, drives, energy flow, nexus, docs"
```

---

## Summary of Changes

| Task | File(s) | Fix | Priority |
|------|---------|-----|----------|
| 1 | `pull.rs`, `main.rs` | Pull satellites alongside reservoirs | Critical |
| 2 | `push.rs`, `main.rs` | Push satellites alongside reservoirs | Critical |
| 3 | `drive_assessment.rs` | All 4 drives at both boundaries | High |
| 4 | `energy_flow.rs` | entry_id tracing + relation discovery | Medium |
| 5 | `intelligence.rs` | Nexus transmutation analysis | Medium |
| 6 | `README.md` | v4 architecture documentation | Low |
