# LifeOS v4 — MCP/CLI Audit & Upgrade Plan

> Generated 2026-07-01 | Contrasts current `lifeos.config.json` against `LifeOS_v4_Architecture.md`

---

## §1. Current State: The Flat Config (26 Databases, No Holonic Awareness)

The current `lifeos.config.json` defines 26 databases as **flat, independent entities** with no awareness of the 5-DB holonic architecture. Each database has:
- A `name`, `data_source_id`, `agent` (productivity/strategic/journaling), and `properties` map
- No concept of archetype role, currency flow, cycle membership, or holonic level

### Current Database Inventory

| # | Config Key | Notion Name | Agent | v4 Reservoir |
|---|-----------|-------------|-------|-------------|
| 1 | `matrix` | Matrix | — | **Matrix** (core) |
| 2 | `potentiator` | Potentiator | — | **Potentiator** (core) |
| 3 | `significator` | Significator | — | **Significator** (core) |
| 4 | `greatway` | GreatWay | — | **GreatWay** (core) |
| 5 | `nexus` | Nexus | — | **Nexus** (core) |
| 6 | `days` | Days | productivity | Potentiator (scaffold) |
| 7 | `weeks` | Weeks | productivity | Potentiator (scaffold) |
| 8 | `months` | Months | productivity | Potentiator (scaffold) |
| 9 | `quarters` | Quarters | strategic | GreatWay (scaffold) |
| 10 | `years` | Years | strategic | GreatWay (scaffold) |
| 11 | `activity_log` | Activity Log | productivity | Potentiator |
| 12 | `activity_types` | Activity Types | productivity | Potentiator |
| 13 | `diet_log` | Diet Log | journaling | Potentiator |
| 14 | `financial_log` | Financial Log | journaling | Potentiator |
| 15 | `financial_accounts` | Financial Accounts | journaling | Potentiator |
| 16 | `subjective_journal` | Subjective Journal | journaling | Potentiator |
| 17 | `relational_journal` | Relational Journal | journaling | Potentiator |
| 18 | `systemic_journal` | Systemic Journal | journaling | Potentiator |
| 19 | `tasks` | Tasks | productivity | GreatWay |
| 20 | `projects` | Projects | strategic | GreatWay |
| 21 | `quarterly_goals` | Quarterly Goals | strategic | GreatWay |
| 22 | `annual_goals` | Annual Goals | strategic | GreatWay |
| 23 | `campaigns` | Campaign Management | strategic | GreatWay |
| 24 | `content_pipeline` | Content Pipeline | strategic | GreatWay |
| 25 | `people` | People | strategic | Significator |
| 26 | `values` | Values & Core Principles | strategic | Significator |
| 27 | `vision` | Vision | strategic | Significator |
| 28 | `opportunities_strengths` | Opportunities & Strengths Log | strategic | Nexus |
| 29 | `directives_risk_log` | Directives & Risk Log | strategic | Nexus |
| 30 | `reports` | Reports | productivity | — (meta) |
| 31 | `notes_management` | Notes Management | productivity | — (meta) |

---

## §2. The Gap: What the MCP Lacks vs v4 Architecture

### 2.1 No Holonic Structure Awareness

**Current:** Config has flat `databases` map with `agent` field (productivity/strategic/journaling).
**v4 Requires:** 5 reservoirs + 1 contact-boundary, each with archetype role, scale, dimension, currency flows.

**Gap:** The `agent` field is a legacy分类 that doesn't map to the holonic architecture. There's no concept of:
- Which reservoir a database belongs to
- What archetype role it plays
- What currencies flow through it
- What cycle it participates in (lesser/greater/both)

### 2.2 No Currency Flow Model

**Current:** Tools treat all databases as independent query targets.
**v4 Requires:** 4 currencies (Catalyst, Experience, Transformation, Choice) flowing through the spiral.

**Gap:** No tool understands:
- That Matrix ingests Catalyst and produces Experience
- That Potentiator ingests Experience and produces Catalyst
- That Nexus transmutes all 4 currencies
- That the flow is bi-directional through the spiral

### 2.3 No Drive/Health Metrics

**Current:** No concept of Agency, Communion, Eros, Agape drives or G_z/P_z health metrics.
**v4 Requires:** 4 drives regulating every boundary, G_z (lesser cycle health) and P_z (greater cycle health).

**Gap:** No tool can:
- Measure G_z (integrative coherence of Matrix⇌Potentiator cycle)
- Measure P_z (transcendental tension of Significator⇌GreatWay cycle)
- Assess drive balance (A_z vs C_z at each boundary)

### 2.4 No Energy-Flow Spiral

**Current:** Tools query individual databases. No cross-reservoir energy tracing.
**v4 Requires:** Understanding the spiral: Matrix → Potentiator → Nexus → Significator → GreatWay → (back).

**Gap:** No tool can:
- Trace how a Catalyst entry in Potentiator eventually becomes a Choice in Significator
- Show the transformation pipeline across reservoirs
- Detect bottlenecks in the energy flow

### 2.5 Legacy Briefing System

**Current:** Briefings organized by C-suite roles (CEO, COO, CMO, CRO, CFO, CHO) and modules.
**v4 Requires:** Briefings organized by holonic level (intra/extra/inter) and cycle (lesser/greater).

**Gap:** The role-based system doesn't map to the v4 architecture. A CEO briefing touches all 5 reservoirs but doesn't understand the energy-flow relationships between them.

---

## §3. The v4 Reservoir Mapping (Current DBs → 5 Core DBs)

### 3.1 Matrix (Intransient Realist) — Current-State Organizer

**Notion DB:** `37ec18ce-5aab-81b4-a206-fc058e2504fa`
**Archetype Role:** Submergent-unconscious current-state organizer
**Scale:** Current-stage, Intra-holonic
**Currency Flow:** Ingests Catalyst → Produces Experience
**Status:** ✅ DB exists in Notion, ✅ Configured in MCP

**Properties (from Notion):**
- Entry Type (multi_select)
- Review Cadence (select)
- Status (status: Active/Evolving/Archived)
- + body/content blocks

**Gap:** Config doesn't tag this as "Matrix archetype" or define its currency role.

### 3.2 Potentiator (Transient Idealist) — Latent-State Generator

**Notion DB:** `0baacff9-75c7-43b0-9449-efd5011c0afc`
**Archetype Role:** Emergent-unconscious latent possibility-space
**Scale:** Current-stage, Extra-holonic
**Currency Flow:** Ingests Experience → Produces Catalyst
**Status:** ✅ DB exists in Notion, ✅ Configured in MCP

**Satellite DBs (currently flat in config):**
- `activity_log` → Activity Log (potentiator_logs)
- `activity_types` → Activity Types (potentiator_logs)
- `diet_log` → Diet Log (potentiator_logs)
- `financial_log` → Financial Log (potentiator_logs)
- `financial_accounts` → Financial Accounts (potentiator_logs)
- `subjective_journal` → Subjective Journal (potentiator_logs)
- `relational_journal` → Relational Journal (potentiator_logs)
- `systemic_journal` → Systemic Journal (potentiator_logs)
- `days` → Days (potentiator_scaffold)
- `weeks` → Weeks (potentiator_scaffold)
- `months` → Months (potentiator_scaffold)

**Properties (from Notion):**
- Digestion Status (status: Raw/Digesting/Crystallized)
- Entry Type (select)
- + body/content blocks

**Gap:** 11 satellite DBs are flat in config, not grouped under Potentiator.

### 3.3 Significator (Intransient Idealist) — Persistent Identity-Pattern

**Notion DB:** `38dc18ce-5aab-80ed-9946-ff7f7680a897`
**Archetype Role:** All-stage intra-holonic persistent identity-pattern
**Scale:** All-stage, Intra-holonic
**Currency Flow:** Ingests Transformation → Produces Choice
**Status:** ✅ DB exists in Notion, ✅ Configured in MCP

**Satellite DBs (currently flat in config):**
- `vision` → Vision (significator_identities)
- `values` → Values & Core Principles (significator_identities)
- `people` → People (significator_identities)
- (Communities — not yet in config)

**Properties (from Notion):**
- Status (status: Draft/Active/Evolving/Archived)
- Stage (select)
- + body/content blocks

**Gap:** 3 satellite DBs are flat, not grouped under Significator.

### 3.4 GreatWay (Transient Realist) — Operating Environment

**Notion DB:** `38dc18ce-5aab-8079-b805-ec3c476260b6`
**Archetype Role:** All-stage extra-holonic operating environment
**Scale:** All-stage, Extra-holonic
**Currency Flow:** Ingests Choice → Produces Transformation
**Status:** ✅ DB exists in Notion, ✅ Configured in MCP

**Satellite DBs (currently flat in config):**
- `annual_goals` → Annual Goals (greatway_commitments)
- `quarterly_goals` → Quarterly Goals (greatway_commitments)
- `projects` → Projects (greatway_commitments)
- `tasks` → Tasks (greatway_commitments)
- `campaigns` → Campaign Management (greatway_commitments)
- `content_pipeline` → Content Pipeline (greatway_commitments)
- `quarters` → Quarters (greatway_scaffold)
- `years` → Years (greatway_scaffold)

**Properties (from Notion):**
- Item Type (select)
- Priority (select: Critical/High/Medium/Low/None)
- Status (status: Future/Ideation/Paused/Active/Done/Cancelled)
- + body/content blocks

**Gap:** 8 satellite DBs are flat, not grouped under GreatWay.

### 3.5 Nexus (Transformation Archetype) — Contact-Boundary

**Notion DB:** `2acc18ce-5aab-80f8-b13b-f5e18b1b5272`
**Archetype Role:** Contact-boundary where all 4 currencies are transmuted
**Scale:** Inter-holonic (both)
**Currency Flow:** Ingests ALL 4 → Produces ALL 4 (transmuted)
**Status:** ✅ DB exists in Notion, ✅ Configured in MCP

**Satellite DBs (currently flat in config):**
- `opportunities_strengths` → Opportunities & Strengths Log (nexus_entries)
- `directives_risk_log` → Directives & Risk Log (nexus_entries)

**Properties (from Notion):**
- Status (status: Identified/Activated/Capitalized/Archived)
- Category (select)
- + body/content blocks

**Gap:** 2 satellite DBs are flat, not grouped under Nexus.

---

## §4. Config Schema Upgrade: v4-Aware Structure

### 4.1 New `LifeOSConfig` Schema

```json
{
  "apiVersion": "2025-09-03",
  "rateLimit": { "requestsPerSecond": 3.0, "cacheTtlSeconds": 300 },
  "holonic": {
    "version": "4.0",
    "currencies": ["Catalyst", "Experience", "Transformation", "Choice"],
    "drives": ["Agency", "Communion", "Eros", "Agape"],
    "cycles": {
      "lesser": { "reservoirs": ["matrix", "potentiator"], "metric": "G_z" },
      "greater": { "reservoirs": ["significator", "greatway"], "metric": "P_z" }
    }
  },
  "databases": {
    "matrix": {
      "name": "Matrix",
      "data_source_id": "37ec18ce-...",
      "archetype": "matrix",
      "scale": "current-stage",
      "dimension": "intra-holonic",
      "currency_in": "Catalyst",
      "currency_out": "Experience",
      "cycle": "lesser",
      "properties": { ... },
      "satellites": {}
    },
    "potentiator": {
      "name": "Potentiator",
      "data_source_id": "0baacff9-...",
      "archetype": "potentiator",
      "scale": "current-stage",
      "dimension": "extra-holonic",
      "currency_in": "Experience",
      "currency_out": "Catalyst",
      "cycle": "lesser",
      "properties": { ... },
      "satellites": {
        "activity_log": { "name": "Activity Log", "data_source_id": "...", "role": "potentiator_logs" },
        "diet_log": { "name": "Diet Log", "data_source_id": "...", "role": "potentiator_logs" },
        "financial_log": { "name": "Financial Log", "data_source_id": "...", "role": "potentiator_logs" },
        "subjective_journal": { "name": "Subjective Journal", "data_source_id": "...", "role": "potentiator_logs" },
        "relational_journal": { "name": "Relational Journal", "data_source_id": "...", "role": "potentiator_logs" },
        "systemic_journal": { "name": "Systemic Journal", "data_source_id": "...", "role": "potentiator_logs" },
        "days": { "name": "Days", "data_source_id": "...", "role": "potentiator_scaffold" },
        "weeks": { "name": "Weeks", "data_source_id": "...", "role": "potentiator_scaffold" },
        "months": { "name": "Months", "data_source_id": "...", "role": "potentiator_scaffold" }
      }
    },
    "significator": {
      "name": "Significator",
      "data_source_id": "38dc18ce-...-9946",
      "archetype": "significator",
      "scale": "all-stage",
      "dimension": "intra-holonic",
      "currency_in": "Transformation",
      "currency_out": "Choice",
      "cycle": "greater",
      "properties": { ... },
      "satellites": {
        "vision": { "name": "Vision", "data_source_id": "...", "role": "significator_identities" },
        "values": { "name": "Values & Core Principles", "data_source_id": "...", "role": "significator_identities" },
        "people": { "name": "People", "data_source_id": "...", "role": "significator_identities" }
      }
    },
    "greatway": {
      "name": "GreatWay",
      "data_source_id": "38dc18ce-...-b805",
      "archetype": "greatway",
      "scale": "all-stage",
      "dimension": "extra-holonic",
      "currency_in": "Choice",
      "currency_out": "Transformation",
      "cycle": "greater",
      "properties": { ... },
      "satellites": {
        "annual_goals": { "name": "Annual Goals", "data_source_id": "...", "role": "greatway_commitments" },
        "quarterly_goals": { "name": "Quarterly Goals", "data_source_id": "...", "role": "greatway_commitments" },
        "projects": { "name": "Projects", "data_source_id": "...", "role": "greatway_commitments" },
        "tasks": { "name": "Tasks", "data_source_id": "...", "role": "greatway_commitments" },
        "campaigns": { "name": "Campaign Management", "data_source_id": "...", "role": "greatway_commitments" },
        "content_pipeline": { "name": "Content Pipeline", "data_source_id": "...", "role": "greatway_commitments" }
      }
    },
    "nexus": {
      "name": "Nexus",
      "data_source_id": "2acc18ce-...-b272",
      "archetype": "nexus",
      "scale": "inter-holonic",
      "dimension": "both",
      "currency_in": "all",
      "currency_out": "all",
      "cycle": "both",
      "properties": { ... },
      "satellites": {
        "opportunities_strengths": { "name": "Opportunities & Strengths Log", "data_source_id": "...", "role": "nexus_entries" },
        "directives_risk_log": { "name": "Directives & Risk Log", "data_source_id": "...", "role": "nexus_entries" }
      }
    }
  },
  "briefings": { ... }
}
```

### 4.2 Key Schema Changes

1. **Top-level `holonic` block** — declares the v4 architecture version, currencies, drives, and cycle membership
2. **Archetype metadata per DB** — `archetype`, `scale`, `dimension`, `currency_in`, `currency_out`, `cycle`
3. **Satellites nested under parent reservoir** — instead of flat, satellite DBs are grouped under their reservoir
4. **`agent` field replaced by `archetype`** — the legacy agent classification is superseded by the holonic role

---

## §5. Tool Upgrade Requirements

### 5.1 `get_schema` — Add Holonic Context

**Current:** Returns flat list of databases with property types.
**Upgrade:** Return hierarchical view with reservoir → satellite structure, archetype roles, and currency flows.

### 5.2 `query` — Add Cross-Reservoir Queries

**Current:** Queries a single database.
**Upgrade:** Support `cycle: "lesser"` or `cycle: "greater"` to query across all reservoirs in a cycle. Support `reservoir: "potentiator"` to query all satellites under a reservoir.

### 5.3 `query_override` — Schema-Aware with v4 Context

**Current:** Validates filters against single DB schema.
**Upgrade:** When querying a reservoir, validate against all satellite schemas. Return currency-flow context in results.

### 5.4 `mutate` — Reservoir-Aware Operations

**Current:** Creates/updates in a single DB.
**Upgrade:** When mutating a satellite, understand which reservoir it belongs to. Optionally trigger cross-reservoir cascades (e.g., creating a Task in GreatWay may need to update Nexus).

### 5.5 `intelligence_briefing` — Holonic Briefings

**Current:** Role-based (CEO, COO, etc.) or module-based.
**Upgrade:** Add holonic briefing modes:
- `lesser_cycle` — Matrix + Potentiator energy flow
- `greater_cycle` — Significator + GreatWay energy flow
- `nexus_transmutation` — All currencies through Nexus
- `drive_balance` — Agency/Communion/Eros/Agape assessment
- `reservoir_health` — G_z and P_z metrics

### 5.6 NEW: `energy_flow` Tool

**Purpose:** Trace currency flow across the spiral.
**Input:** `entry_id` or `reservoir` or `cycle`
**Output:** How Catalyst/Experience/Transformation/Choice flows through the system for the given scope.

### 5.7 NEW: `drive_assessment` Tool

**Purpose:** Evaluate the 4 drives at each boundary.
**Input:** `boundary` (lesser/greater/both)
**Output:** A_z (Agency), C_z (Communion), balance assessment, and recommendations.

### 5.8 NEW: `health_metrics` Tool

**Purpose:** Calculate G_z and P_z health metrics.
**Input:** `metric` (G_z/P_z/both)
**Output:** Integrative coherence (G_z) and transcendental tension (P_z) scores with breakdowns.

---

## §6. Briefing System Upgrade

### 6.1 Current Briefing Roles → v4 Mapping

| Current Role | v4 Holonic Level | Primary Reservoirs |
|-------------|-----------------|-------------------|
| CEO | All (holonic overview) | All 5 DBs |
| COO | Intra-holonic (current) | Matrix, Potentiator |
| CMO | Extra-holonic (all-stage) | GreatWay (content/campaigns) |
| CRO | Intra-holonic (all-stage) | Significator (people) |
| CFO | Extra-holonic (current) | Potentiator (financial) |
| CHO | Inter-holonic | Nexus (health/opportunities) |

### 6.2 New Briefing Architecture

```json
{
  "briefings": {
    "holonic": {
      "lesser_cycle": {
        "reservoirs": ["matrix", "potentiator"],
        "intent": "Current-stage energy flow health",
        "metrics": ["G_z"]
      },
      "greater_cycle": {
        "reservoirs": ["significator", "greatway"],
        "intent": "All-stage evolutionary tension",
        "metrics": ["P_z"]
      },
      "nexus": {
        "reservoirs": ["nexus"],
        "intent": "Currency transmutation efficiency"
      }
    },
    "legacy_roles": { ... }
  }
}
```

---

## §7. Phased Implementation Roadmap

### Phase 1: Config Restructuring (Foundation)
**Goal:** Restructure `lifeos.config.json` to v4-aware schema without breaking existing tools.

1. Add `holonic` top-level block to config
2. Add `archetype`, `scale`, `dimension`, `currency_in`, `currency_out`, `cycle` to each core DB
3. Nest satellite DBs under their parent reservoir in `satellites` map
4. Maintain backward compatibility: flat `databases` map still works for legacy tools
5. Update `DbConfig` struct in `config.rs` to include new fields (with `#[serde(default)]`)

**Files to modify:**
- `lifeos.config.json` — add holonic metadata
- `lifeos-core/src/config.rs` — extend `DbConfig` with v4 fields
- `lifeos-core/src/util/schema_engine.rs` — `SchemaCache` understands reservoir hierarchy

### Phase 2: Tool Awareness (Intelligence)
**Goal:** Make tools understand the holonic structure.

1. Update `get_schema` to return hierarchical view with reservoir → satellite structure
2. Add `reservoir` and `cycle` parameters to `query` tool
3. Update `intelligence_briefing` with holonic briefing modes
4. Add `energy_flow` tool for cross-reservoir currency tracing
5. Add `drive_assessment` tool for boundary drive evaluation
6. Add `health_metrics` tool for G_z/P_z calculation

**Files to modify:**
- `lifeos-core/src/tools/mod.rs` — register new tools
- `lifeos-core/src/tools/query.rs` — add reservoir/cycle query support
- `lifeos-core/src/tools/intelligence.rs` — add holonic briefing modes
- `lifeos-core/src/tools/energy_flow.rs` (new) — currency flow tracing
- `lifeos-core/src/tools/drive_assessment.rs` (new) — drive evaluation
- `lifeos-core/src/tools/health_metrics.rs` (new) — G_z/P_z metrics

### Phase 3: Briefing System Upgrade (Analysis)
**Goal:** Replace legacy C-suite briefings with holonic-aware analysis.

1. Implement `lesser_cycle` briefing (Matrix + Potentiator energy flow)
2. Implement `greater_cycle` briefing (Significator + GreatWay energy flow)
3. Implement `nexus_transmutation` briefing (all currencies through Nexus)
4. Implement `drive_balance` briefing (Agency/Communion/Eros/Agape)
5. Implement `reservoir_health` briefing (G_z and P_z with breakdowns)
6. Maintain legacy role briefings for backward compatibility

### Phase 4: Sync & Mutation Upgrade (Operations)
**Goal:** Make pull/push/mutate reservoir-aware.

1. Update `pull` to understand reservoir hierarchy (pull entire reservoir or individual satellites)
2. Update `push` to understand reservoir hierarchy
3. Update `mutate` to understand which reservoir a satellite belongs to
4. Add optional cross-reservoir cascade triggers in `mutate`
5. Update `sync_note` to understand reservoir context

### Phase 5: Documentation & Polish
**Goal:** Complete documentation and ensure production readiness.

1. Write v4 MCP API documentation
2. Update `get_schema` descriptions with holonic context
3. Add v4 architecture guide to `README.md`
4. Update MCP server instructions string with v4-aware tool list
5. Add v4 architecture validation to `discover` command

---

## §8. Migration Strategy

### Backward Compatibility
- Flat `databases` map continues to work (legacy mode)
- New `satellites` field is additive
- `agent` field is preserved but superseded by `archetype`
- All existing tools work unchanged in Phase 1

### Config Migration Path
1. Phase 1: Add holonic metadata alongside existing flat structure
2. Phase 2: New tools read from holonic structure
3. Phase 3: Briefings use holonic structure
4. Phase 4: Sync tools use holonic structure
5. Phase 5: Deprecate flat structure (optional)

### Notion DB Changes
- No changes to Notion databases required
- All v4 metadata lives in `lifeos.config.json`
- Satellite DBs remain as separate Notion DBs (not merged)

---

## §9. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Breaking existing tool calls | High | Backward-compatible schema with `#[serde(default)]` |
| Config file becomes too large | Medium | Satellite DBs nested, not duplicated |
| SchemaCache performance | Medium | Pre-warm with reservoir hierarchy |
| Briefing regression | High | Legacy briefings preserved alongside holonic |
| Notion API rate limits | Medium | Reservoir-aware batching in queries |

---

## §10. Success Criteria

1. ✅ All 26 databases mapped to v4 reservoir structure
2. ✅ Config reflects holonic architecture (archetype, scale, dimension, currencies)
3. ✅ Tools understand reservoir hierarchy
4. ✅ Holonic briefings available (lesser_cycle, greater_cycle, nexus)
5. ✅ Energy flow tracing works across reservoirs
6. ✅ G_z and P_z health metrics calculable
7. ✅ Legacy tools continue to work unchanged
8. ✅ No breaking changes to existing MCP protocol
