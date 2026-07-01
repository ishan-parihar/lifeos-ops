# LifeOS v4 MCP Upgrade — Implementation Plan

> Actionable step-by-step plan derived from `V4_MCP_AUDIT.md`
> Date: 2026-07-01

---

## Phase 1: Config Restructuring (Foundation)

### Step 1.1: Extend `DbConfig` in `config.rs`

Add v4 holonic metadata fields to the `DbConfig` struct. All new fields use `#[serde(default)]` for backward compatibility.

**File:** `lifeos-core/src/config.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub name: String,
    #[serde(rename = "data_source_id")]
    pub database_id: String,
    pub agent: String,
    pub properties: HashMap<String, String>,

    // ── v4 Holonic Metadata ──
    /// Archetype role: "matrix", "potentiator", "significator", "greatway", "nexus"
    #[serde(default)]
    pub archetype: Option<String>,
    /// Scale: "current-stage" or "all-stage"
    #[serde(default)]
    pub scale: Option<String>,
    /// Dimension: "intra-holonic", "extra-holonic", "inter-holonic", or "both"
    #[serde(default)]
    pub dimension: Option<String>,
    /// Primary currency this reservoir ingests
    #[serde(default)]
    pub currency_in: Option<String>,
    /// Primary currency this reservoir produces
    #[serde(default)]
    pub currency_out: Option<String>,
    /// Which cycle this reservoir participates in: "lesser", "greater", or "both"
    #[serde(default)]
    pub cycle: Option<String>,
    /// Nested satellite databases (only for core reservoirs)
    #[serde(default)]
    pub satellites: Option<HashMap<String, SatelliteDbConfig>>,

    #[serde(skip)]
    pub resolved_data_source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatelliteDbConfig {
    pub name: String,
    pub data_source_id: String,
    /// Role within the reservoir: e.g. "potentiator_logs", "greatway_commitments"
    #[serde(default)]
    pub role: Option<String>,
    pub properties: HashMap<String, String>,
    #[serde(skip)]
    pub resolved_data_source_id: Option<String>,
}
```

### Step 1.2: Add Holonic Top-Level Config

**File:** `lifeos-core/src/config.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolonicConfig {
    pub version: String,
    pub currencies: Vec<String>,
    pub drives: Vec<String>,
    pub cycles: CycleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleConfig {
    pub lesser: CycleDefinition,
    pub greater: CycleDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleDefinition {
    pub reservoirs: Vec<String>,
    pub metric: String,
}

// Add to LifeOSConfig:
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifeOSConfig {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: RateLimitConfig,
    pub databases: HashMap<String, DbConfig>,
    #[serde(default)]
    pub holonic: Option<HolonicConfig>,
    #[serde(default)]
    pub briefings: Option<BriefingConfig>,
    #[serde(default)]
    pub notion: Option<NotionConfig>,
}
```

### Step 1.3: Update `lifeos.config.json`

Add the `holonic` block and archetype metadata to the 5 core DBs. Nest satellites under their parents.

**Changes to `lifeos.config.json`:**
1. Add top-level `"holonic": { ... }` block
2. Add `archetype`, `scale`, `dimension`, `currency_in`, `currency_out`, `cycle` to matrix, potentiator, significator, greatway, nexus
3. Add `satellites` map to each core DB with nested satellite configs
4. Remove flat satellite entries from top-level `databases` (they're now nested)

### Step 1.4: Update `SchemaCache` for Reservoir Hierarchy

**File:** `lifeos-core/src/util/schema_engine.rs`

Add methods to `SchemaCache`:
- `get_reservoir(key: &str) -> Option<&str>` — returns which reservoir a DB belongs to
- `get_satellites(reservoir: &str) -> Option<&[String]>` — returns satellite keys for a reservoir
- `get_reservoir_properties(reservoir: &str) -> Option<HashMap<String, PropInfo>>` — merged properties of all satellites in a reservoir
- `describe_reservoir(reservoir: &str) -> String` — hierarchical description

---

## Phase 2: Tool Awareness (Intelligence)

### Step 2.1: Update `get_schema` for Hierarchical View

**File:** `lifeos-core/src/tools/mod.rs`

Update `execute_get_schema` to:
1. If no database specified: return all 5 reservoirs with their satellites
2. If reservoir specified: return that reservoir + all its satellites
3. If satellite specified: return that satellite's schema

Format output as:
```
Matrix (intransient realist, current-stage, intra-holonic):
  Currency: Catalyst → Experience | Cycle: lesser
  Properties: entry_type(multi_select), review_cadence(select), status(status:Active/Evolving/Archived)
  
  Satellites:
    (none currently)
```

### Step 2.2: Add Reservoir/Cycle Query Support to `query`

**File:** `lifeos-core/src/tools/query.rs`

Add new parameters:
```rust
pub struct QueryParams {
    // ... existing fields ...
    /// Query all DBs in a reservoir (e.g., "potentiator" to query all its satellites)
    pub reservoir: Option<String>,
    /// Query all DBs in a cycle (e.g., "lesser" for matrix+potentiator)
    pub cycle: Option<String>,
}
```

When `reservoir` is specified:
1. Look up all satellite DBs under that reservoir
2. Query each satellite
3. Merge results with reservoir metadata

When `cycle` is specified:
1. Look up reservoirs in that cycle from `holonic.cycles`
2. Query all reservoirs and their satellites
3. Merge results with cycle metadata

### Step 2.3: Create `energy_flow` Tool

**New File:** `lifeos-core/src/tools/energy_flow.rs`

```rust
//! Energy flow tool — trace currency flow across the holonic spiral

pub struct EnergyFlowParams {
    /// Scope: "lesser_cycle", "greater_cycle", "full_spiral", or specific reservoir
    pub scope: String,
    /// Optional specific entry ID to trace
    pub entry_id: Option<String>,
    /// Currency to trace: "Catalyst", "Experience", "Transformation", "Choice", or "all"
    pub currency: Option<String>,
}

pub fn schema() -> serde_json::Value { ... }

pub async fn execute(params: &EnergyFlowParams, config: &LifeOSConfig, notion: &NotionClient) -> Result<String, String> {
    // 1. Identify which reservoirs are in scope
    // 2. Query each reservoir for entries matching the currency
    // 3. Trace cross-reservoir relationships (via relations)
    // 4. Return energy flow map with currency paths
}
```

### Step 2.4: Create `drive_assessment` Tool

**New File:** `lifeos-core/src/tools/drive_assessment.rs`

```rust
//! Drive assessment tool — evaluate Agency/Communion/Eros/Agape at each boundary

pub struct DriveAssessmentParams {
    /// Boundary: "lesser" (Matrix⇌Potentiator), "greater" (Significator⇌GreatWay), or "both"
    pub boundary: String,
    /// Optional date range for assessment
    pub range: Option<String>,
}

pub fn schema() -> serde_json::Value { ... }

pub async fn execute(params: &DriveAssessmentParams, config: &LifeOSConfig, notion: &NotionClient) -> Result<String, String> {
    // 1. Query the relevant reservoirs
    // 2. Analyze entry distribution across types/states
    // 3. Calculate drive balance (A_z vs C_z)
    // 4. Return assessment with recommendations
}
```

### Step 2.5: Create `health_metrics` Tool

**New File:** `lifeos-core/src/tools/health_metrics.rs`

```rust
//! Health metrics tool — calculate G_z and P_z holonic health metrics

pub struct HealthMetricsParams {
    /// Metric: "G_z" (lesser cycle), "P_z" (greater cycle), or "both"
    pub metric: String,
    /// Optional date range
    pub range: Option<String>,
}

pub fn schema() -> serde_json::Value { ... }

pub async fn execute(params: &HealthMetricsParams, config: &LifeOSConfig, notion: &NotionClient) -> Result<String, String> {
    // G_z (Agape): Integrative coherence of Matrix⇌Potentiator
    //   - Ratio of Experience entries digested vs raw Catalyst
    //   - Balance between activity types
    //   - Journal consistency
    //
    // P_z (Eros): Transcendental tension of Significator⇌GreatWay
    //   - Progress of Goals → Projects → Tasks
    //   - Strategic alignment score
    //   - Evolutionary momentum
    //
    // Return scores with breakdowns and recommendations
}
```

### Step 2.6: Register New Tools

**File:** `lifeos-core/src/tools/mod.rs`

Add module declarations:
```rust
pub mod energy_flow;
pub mod drive_assessment;
pub mod health_metrics;
```

Add to `get_tool_definitions`:
```rust
tool_def("energy_flow", "Trace currency flow across the holonic spiral. Shows how Catalyst/Experience/Transformation/Choice moves through Matrix, Potentiator, Significator, GreatWay, and Nexus.", energy_flow::schema()),
tool_def("drive_assessment", "Evaluate Agency/Communion/Eros/Agape drive balance at each boundary. Returns balance metrics and recommendations.", drive_assessment::schema()),
tool_def("health_metrics", "Calculate G_z (integrative coherence) and P_z (transcendental tension) health metrics for the holonic system.", health_metrics::schema()),
```

Add to `call_tool`:
```rust
"energy_flow" => { ... }
"drive_assessment" => { ... }
"health_metrics" => { ... }
```

---

## Phase 3: Briefing System Upgrade

### Step 3.1: Add Holonic Briefing Modes

**File:** `lifeos-core/src/tools/intelligence.rs`

Add new briefing modes to `IntelligenceParams.mode` enum:
```rust
"lesser_cycle" | "greater_cycle" | "nexus" | "drive_balance" | "reservoir_health"
```

### Step 3.2: Implement `lesser_cycle` Briefing

Queries Matrix + Potentiator, analyzes:
- Catalyst ingestion rate (entries in Potentiator)
- Experience generation (entries in Matrix)
- Digestion status distribution (Raw/Digesting/Crystallized)
- G_z health estimate

### Step 3.3: Implement `greater_cycle` Briefing

Queries Significator + GreatWay, analyzes:
- Strategic alignment (Goals → Projects → Tasks)
- Transformation flow (Nexus entries → Significator)
- P_z health estimate
- Evolutionary momentum

### Step 3.4: Implement `nexus` Briefing

Queries Nexus + satellites, analyzes:
- Currency transmutation efficiency
- Opportunity activation rate
- Directive/Risk assessment
- Contact-boundary health

### Step 3.5: Implement `drive_balance` Briefing

Analyzes all 5 reservoirs, calculates:
- Agency (boundary resistance) vs Communion (field conductance)
- Eros (greater cycle tension) vs Agape (lesser cycle coherence)
- Balance recommendations

---

## Phase 4: Sync & Mutation Upgrade

### Step 4.1: Reservoir-Aware Pull

**File:** `lifeos-core/src/sync/pull.rs`

Update `pull_database` to:
1. Accept reservoir name (pull all satellites)
2. Accept cycle name (pull all reservoirs in cycle)
3. Add reservoir metadata to pulled files

### Step 4.2: Reservoir-Aware Push

**File:** `lifeos-core/src/sync/push.rs`

Update `push_database` to:
1. Accept reservoir name (push all satellites)
2. Understand reservoir hierarchy when pushing
3. Add cross-reservoir cascade option

### Step 4.3: Reservoir-Aware Mutate

**File:** `lifeos-core/src/tools/mutate.rs`

Update `MutateParams` to:
1. Accept `reservoir` parameter (auto-resolve satellite from reservoir + role)
2. Add optional `cascade` parameter for cross-reservoir updates

---

## Phase 5: Documentation & Polish

### Step 5.1: Update MCP Server Instructions

**File:** `lifeos-core/src/server.rs`

Update the `instructions` string to include v4-aware tool descriptions and holonic architecture overview.

### Step 5.2: Update `README.md`

Add v4 architecture section explaining:
- The 5 DBs as 4 reservoirs + 1 contact-boundary
- The energy-flow spiral
- The 4 currencies and 4 drives
- How the MCP tools map to the architecture

### Step 5.3: Update `discover` Command

**File:** `lifeos/src/main.rs`

Update `cmd_discover` to:
1. Detect v4 holonic structure in Notion
2. Auto-assign archetype roles based on DB names
3. Validate reservoir completeness
