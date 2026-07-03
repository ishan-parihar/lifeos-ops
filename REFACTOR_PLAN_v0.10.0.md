# LifeOS v0.10.0 — Refactor Plan: Deliberate Relational Architecture

**Date:** 2026-07-03
**Principle:** Every relation is a deliberate choice. Tools surface gaps and suggest connections; the user (or AI agent acting explicitly on the user's behalf) approves each one. No auto-population of complex data.

---

## Problem Statement

The v0.9.0 audit revealed:

1. **4 of 5 DBs have near-zero relational density** (Potentiator 100% orphaned, Significator 100% orphaned, Nexus 95% orphaned)
2. **No tool helps the user SEE relational gaps** — they must manually query each DB
3. **No tool helps the user BUILD context** for an entry — AI agents must call 3+ tools to understand an entry's neighborhood
4. **No tool traces currency flow** across the holonic spiral
5. **77% of Matrix + Significator entries are uncategorized** — no tool suggests entry-types
6. **AI agents lack the surface area** to perform relational mapping, densification, and synthesis

The solution is NOT automation. The solution is **deliberate-choice tooling** — tools that surface what needs attention, suggest what could be linked, and let the user decide.

---

## Design Principle

```
┌─────────────────────────────────────────────────────────────┐
│  TOOL SURFACES  →  USER/AI DELIBERATES  →  TOOL EXECUTES   │
│  (read-only)       (approves each)         (write, one at   │
│                                                 a time)      │
└─────────────────────────────────────────────────────────────┘
```

Every write tool must:
1. Accept explicit parameters (what to link, what to tag)
2. Never infer or auto-populate
3. Return exactly what was changed
4. Be reversible (archive, not delete)

---

## 4 New MCP/CLI Tools

### Tool 1: `relational_gaps` (read-only)

**Purpose:** Surfaces entries with zero or sparse relations, grouped by DB and entry-type. Shows what SHOULD be linked based on the ontology but isn't.

**CLI:** `lifeos relational-gaps [--db <db>] [--entry-type <et>] [--min-relations 0]`

**MCP:** `relational_gaps` tool with params: `database`, `entry_type`, `min_relations` (default: 0)

**Output example:**
```
Relational Gaps Report — 7,847 entries with 0 relations
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

POTENTIATOR (6,876 entries, 100% orphaned)
  Activity:     5,560 entries with 0 relations
                → Expected: "Generated From" → Matrix (if linked to a Pattern)
                → Expected: "People" → GreatWay.Person (for relational activities)
  Financial:      996 entries with 0 relations
                → Expected: "Crystallized To" → Matrix (if linked to a budget/inventory)
  Relational:      83 entries with 0 relations
                → Expected: "People" → GreatWay.Person (relational logs should link to persons)

SIGNIFICATOR (89 entries, 100% orphaned)
  (uncategorized): 69 entries with 0 relations
                → Expected: "Anchored In" → Matrix (identity patterns anchored in state)
  Pillar:          10 entries with 0 relations
                → Expected: "Coheres With" → other Significator entries
  Value:           10 entries with 0 relations
                → Expected: "Coheres With" → other Significator entries

NEXUS (794 entries, 95% orphaned)
  Note:           757 entries with 0 relations
                → Expected: "Updates" → Matrix (if note is about a Matrix entry)
                → Expected: "Sourced From" → Potentiator (if note originated from an activity)
```

**Implementation:** ~150 lines in `lifeos-core/src/tools/relational_gaps.rs`. Queries each DB, counts relations per entry, groups by entry-type, prints ontology-expected relations that are missing.

---

### Tool 2: `build_context` (read-only)

**Purpose:** Assembles a complete relational neighborhood for a single entry — the entry itself + all related entries across all 5 DBs + their types + their relations. One call replaces 3+ calls (`get_page` + `backlinks` + `trace`).

**CLI:** `lifeos build-context --page-id <id> [--depth 2]`

**MCP:** `build_context` tool with params: `page_id`, `depth` (default: 1, max: 3)

**Output:** Structured JSON with:
```json
{
  "entry": { "id": "...", "title": "...", "db": "matrix", "entry_type": "Pattern", "properties": {...} },
  "relations": {
    "outgoing": [
      { "prop": "Pillar Link", "target": { "id": "...", "title": "...", "db": "significator", "entry_type": "Pillar" } }
    ],
    "incoming": [
      { "prop": "Generated From", "source": { "id": "...", "title": "...", "db": "potentiator", "entry_type": "Activity" } }
    ]
  },
  "neighborhood": {
    "depth_2": [
      { "via": "Pillar Link → Coheres With", "entry": { "id": "...", "title": "...", "db": "significator" } }
    ]
  },
  "gaps": [
    "No 'Accumulates Into' relation → Significator (expected for Matrix.Pattern entries)",
    "No 'Generated From' relation ← Potentiator (expected for Matrix entries)"
  ]
}
```

**Implementation:** ~200 lines in `lifeos-core/src/tools/build_context.rs`. Calls `get_page` + `backlinks` + `trace` internally, merges results, adds gap analysis.

---

### Tool 3: `holonic_synthesis` (read-only)

**Purpose:** Traces currency flow across the holonic spiral for a given entry or time range. Shows how Catalyst → Experience → Transformation → Choice flows through the 5 DBs.

**CLI:** `lifeos holonic-synthesis --page-id <id>` OR `lifeos holonic-synthesis --days 7`

**MCP:** `holonic_synthesis` tool with params: `page_id` (optional), `days_back` (optional, default: 7)

**Output example:**
```
Holonic Synthesis — Last 7 days
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

LESSER CYCLE (Matrix ⇌ Potentiator)
  Catalyst IN → Matrix:        0 entries
  Experience OUT → Potentiator: 0 entries
  Refined Catalyst ← Potentiator: 0 entries
  Status: DORMANT — no currency flowing

GREATER CYCLE (Significator ⇌ GreatWay)
  Transformation IN → Significator: 0 entries
  Choice OUT → GreatWay:         0 entries
  Status: DORMANT — no currency flowing

NEXUS (Contact Boundary)
  Catalyst-kind entries: 794 (all Note category)
  Experience-kind: 0
  Transformation-kind: 0
  Choice-kind: 0
  Status: CONGESTED with Catalyst, no transmutation happening

RELATIONAL FLOW (Potentiator ⇌ GreatWay)
  Potentiator.Relational → GreatWay.Person: 5 links
  Status: MINIMAL — only 5 of 83 Relational entries linked to Persons

RECOMMENDATIONS (for user review):
  1. 757 Nexus.Note entries have no relations — review and link manually
  2. 6,876 Potentiator entries are orphaned — start with Relational (83 entries)
  3. 89 Significator entries are orphaned — start with Pillar (10) + Value (10)
```

**Implementation:** ~250 lines in `lifeos-core/src/tools/holonic_synthesis.rs`. Queries each DB's recent entries, traces relation paths, identifies flow bottlenecks.

---

### Tool 4: `suggest_categorization` (read-only, suggests only)

**Purpose:** Suggests entry-types for uncategorized entries based on title/content heuristics. Does NOT auto-assign — returns suggestions for the user to approve via `mutate`.

**CLI:** `lifeos suggest-categorization --db matrix [--limit 20]`

**MCP:** `suggest_categorization` tool with params: `database`, `limit` (default: 20)

**Output example:**
```
Categorization Suggestions — Matrix (34 uncategorized entries)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. "Identity Reconstruction Post-Recovery"
   Suggested: Pattern (confidence: high — title matches "reconstruction" pattern)
   Reason: Title describes a recurring identity configuration

2. "Holonic Architecture Understanding"
   Suggested: Pattern (confidence: high — title describes a cognitive pattern)
   Reason: Title describes a mental model / understanding pattern

3. "MQL5 Trading System Development"
   Suggested: Pattern (confidence: medium — could be Active Project)
   Reason: Title describes a system, but "development" suggests active work

To apply: lifeos mutate --operation update --database matrix --page-id <id> \\
  --properties '{"Entry Type": "Pattern"}'
```

**Implementation:** ~200 lines in `lifeos-core/src/tools/suggest_categorization.rs`. Uses title keyword matching + entry-type definitions to suggest categories. Returns confidence + reasoning. Never writes.

---

## Tool Surface Summary (Post-v0.10.0)

| Category | Tools | New? |
|----------|-------|------|
| Schema/Query | get_schema, query, query_override, mutate | existing |
| Relational Read | get_page, expand, trace, ancestors, backlinks, graph_metrics | existing |
| Relational Write | link | existing |
| **Relational Intelligence** | **relational_gaps, build_context, holonic_synthesis** | **NEW** |
| **Categorization** | **suggest_categorization** | **NEW** |
| Audit | orphans, validate, validate_yaml, suggest_links | existing |
| Ontology | archetype_index, derive_type, valence_signature | existing |
| Intelligence | intelligence_briefing, data_science, review_pipeline, strategic_simulator | existing |
| Energy/Health | energy_flow, drive_assessment, health_metrics | existing |
| Sync | sync_note, pull, push, watch | existing |

**Total: 27 → 31 MCP tools** (4 new read-only tools)

---

## What These Tools Enable for AI Agents

### Scenario 1: AI agent wants to understand an entry's full context
**Before (v0.9.0):** Call `get_page` → call `backlinks` → call `trace` → manually merge results → miss gaps
**After (v0.10.0):** Call `build_context` once → get entry + all relations + neighborhood + gap analysis

### Scenario 2: AI agent wants to help densify relations
**Before (v0.9.0):** Call `orphans` → get list of orphaned entries → no guidance on what to link
**After (v0.10.0):** Call `relational_gaps` → get ontology-expected relations per entry-type → call `suggest_categorization` for uncategorized entries → present suggestions to user → user approves → call `link` or `mutate`

### Scenario 3: AI agent wants to synthesize insights across the holonic spiral
**Before (v0.9.0):** Call `energy_flow` → get raw currency data → no synthesis
**After (v0.10.0):** Call `holonic_synthesis` → get flow analysis + bottleneck identification + recommendations → present to user for action

### Scenario 4: User wants to categorize 69 uncategorized Significator entries
**Before (v0.9.0):** Open each entry in Notion UI → manually select entry-type
**After (v0.10.0):** Call `suggest_categorization --db significator` → review suggestions → call `mutate` for each approved suggestion

---

## Implementation Plan

### Phase 1: `relational_gaps` tool (highest impact)
- **File:** `lifeos-core/src/tools/relational_gaps.rs` (~150 lines)
- **CLI:** Add `RelationalGaps` command to `cli/mod.rs`
- **MCP:** Register in `tools/mod.rs`
- **Effort:** 2 hours
- **Value:** Surfaces the 7,847 orphaned entries with ontology-expected relations

### Phase 2: `build_context` tool (highest AI agent value)
- **File:** `lifeos-core/src/tools/build_context.rs` (~200 lines)
- **CLI:** Add `BuildContext` command
- **MCP:** Register
- **Effort:** 3 hours
- **Value:** Single-call context assembly for AI agents

### Phase 3: `holonic_synthesis` tool (highest strategic value)
- **File:** `lifeos-core/src/tools/holonic_synthesis.rs` (~250 lines)
- **CLI:** Add `HolonicSynthesis` command
- **MCP:** Register
- **Effort:** 4 hours
- **Value:** Traces currency flow, identifies bottlenecks, recommends action areas

### Phase 4: `suggest_categorization` tool (highest cleanup value)
- **File:** `lifeos-core/src/tools/suggest_categorization.rs` (~200 lines)
- **CLI:** Add `SuggestCategorization` command
- **MCP:** Register
- **Effort:** 3 hours
- **Value:** Suggests entry-types for 103 uncategorized Matrix + Significator entries

**Total effort:** ~12 hours, ~800 lines of new Rust code, 0 lines of auto-population logic.

---

## What This Plan Does NOT Do

- ❌ No auto-population of relations
- ❌ No auto-assignment of entry-types
- ❌ No bulk relation creation without explicit per-entry approval
- ❌ No inference of user preferences
- ❌ No background automation

Every write operation remains a deliberate, explicit choice. The tools only make the user's choices **informed** by surfacing what needs attention and **efficient** by reducing the number of API calls needed to understand context.

---

## Success Metrics

After v0.10.0, the user should be able to:

1. Run `lifeos relational-gaps` and immediately see which entries need attention
2. Run `lifeos build-context --page-id <id>` and get a complete picture of any entry
3. Run `lifeos holonic-synthesis --days 7` and see where currency flow is blocked
4. Run `lifeos suggest-categorization --db significator` and get actionable suggestions
5. An AI agent can call `build_context` to understand an entry, then call `link` to create a relation the user approved

The relational density will increase through **deliberate human action**, guided by tooling — not through automation that could create incorrect links in complex data.
