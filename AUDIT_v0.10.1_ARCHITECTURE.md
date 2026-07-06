# LifeOS v0.10.1 — Architecture Audit

> **Audit date:** 2026-07-06
> **Auditor:** LifeOS-Architect
> **Scope:** Rust codebase (lifeos-core lib + lifeos binary), MCP server, config system, sync engine, schemas, scripts, build/test infrastructure.
> **Method:** Static analysis (cargo build, wc -l, grep) + runtime tests (14-test MCP smoke suite, validate-yaml --self-test, discover, dashboard).
> **Companion docs:** [AUDIT_v0.10.1_DB_SCHEMA.md](AUDIT_v0.10.1_DB_SCHEMA.md), [AUDIT_v0.10.1_ACTION_TRACKER.md](AUDIT_v0.10.1_ACTION_TRACKER.md), [AGENTS.md](AGENTS.md)

---

## 1. Executive Summary

| Metric | Value |
|--------|-------|
| Rust workspace | 2 crates (lifeos-core lib + lifeos binary) |
| Rust LOC | 17,007 |
| Python LOC (scripts/) | 2,339 |
| YAML LOC (schemas/) | 2,759 |
| Markdown LOC (docs) | 1,496 |
| **Total project LOC** | **23,601** |
| MCP tools listed | 30 in `tools/list` + `expand` + `graph_metrics` = **31** |
| MCP dispatch arms | 36 (includes 6 backward-compat aliases) |
| YAML schemas | 37 (1 universal + 5 per_db + 31 per_entry_type) |
| Notion API version | 2025-09-03 (data_source abstraction) |
| Build status | ✅ 0 errors, 0 warnings |
| Test status | ✅ 14/14 MCP smoke tests pass |

**Architecture verdict:** Sound. The v0.10.0→v0.10.1 consolidation removed deadweight (34→30 tools, fixed double-wrap bugs, fixed clap collisions, async MCP server). Remaining issues are mostly YAGNI candidates and deferred features.

---

## 2. Workspace Structure

```
lifeos-ops/                         # Cargo workspace (resolver = "2")
├── Cargo.toml                      # workspace manifest, edition 2021
├── Cargo.lock                      # pinned deps
├── lifeos-core/                    # library crate (the engine)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # public API surface
│       ├── config.rs               # LifeOSConfig, DbConfig, HolonicConfig (427 LOC)
│       ├── cli/mod.rs              # clap command tree (553 LOC)
│       ├── notion/                 # Notion API client
│       │   ├── client.rs           # NotionClient + resolve_all_data_sources (514 LOC)
│       │   └── types.rs            # typed PropertyValue enum + structs
│       ├── util/
│       │   ├── schema_engine.rs    # SchemaCache — auto-discovering
│       │   ├── yaml_schemas.rs     # 3-tier YAML validator
│       │   ├── id_resolver.rs      # fuzzy title → page ID
│       │   └── date_filter.rs      # date-range filter construction
│       ├── tools/                  # 23 tool modules (one file per tool/group)
│       │   ├── mod.rs              # THE registry (366 LOC) — tool_def + dispatch
│       │   ├── query.rs            # query + query_override
│       │   ├── mutate.rs           # create/update/delete/upsert
│       │   ├── relations.rs        # get_page, expand, trace, ancestors, backlinks, link, graph_metrics (781 LOC — largest)
│       │   ├── relation_ops.rs     # unlink, batch_link
│       │   ├── build_context.rs    # one-call relational neighborhood
│       │   ├── holonic_synthesis.rs # currency flow + G_z/P_z (consolidated)
│       │   ├── energy_flow.rs      # [BACKWARD-COMPAT ALIAS] → holonic_synthesis
│       │   ├── drive_assessment.rs # [BACKWARD-COMPAT ALIAS] → holonic_synthesis
│       │   ├── health_metrics.rs   # [BACKWARD-COMPAT ALIAS] → holonic_synthesis
│       │   ├── relational_gaps.rs  # orphan + gap surfacing
│       │   ├── relational_graph.rs # inter-DB relation tree
│       │   ├── suggest_categorization.rs # entry-type suggestions (read-only)
│       │   ├── suggest_links.rs    # (in audit.rs) link suggestions
│       │   ├── audit.rs            # orphans, validate (legacy), suggest_links (542 LOC)
│       │   ├── validate_yaml.rs    # 3-tier YAML schema validator
│       │   ├── ontology.rs         # archetype_index, derive_type, valence_signature
│       │   ├── intelligence.rs     # role/cycle briefings (557 LOC)
│       │   ├── data_science.rs     # temporal patterns (1110 LOC — 2nd largest)
│       │   ├── review.rs           # daily/weekly/monthly/quarterly reviews
│       │   ├── strategic.rs        # OKR/project/campaign simulator
│       │   ├── sync_note.rs        # bidirectional Notion ↔ markdown sync
│       │   ├── workflows.rs        # daily + dashboard commands
│       │   └── auto_enrich.rs      # [NEW v0.10.1] entry-type→property suggestions
│       ├── transform/              # Notion blocks ↔ markdown
│       │   ├── blocks_to_md.rs
│       │   ├── md_to_blocks.rs
│       │   └── properties.rs
│       ├── sync/                   # bidirectional Notion ↔ vault sync
│       │   ├── pull.rs, push.rs, merge.rs, watch.rs, page.rs
│       ├── vault/mod.rs            # local markdown vault index
│       ├── toon_format.rs          # YAML-front-matter text format encoder
│       └── server.rs               # MCP JSON-RPC server (async, v0.10.1 — 389 LOC)
├── lifeos/                         # binary crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                 # CLI dispatch (840 LOC)
│       └── mcp.rs                  # MCP runner (70 LOC)
├── schemas/                        # 3-tier YAML schema hierarchy
│   ├── universal/holon_coordinate.yaml
│   ├── per_db/{matrix,potentiator,nexus,significator,greatway}.yaml
│   └── per_entry_type/*.yaml       # 31 files
├── scripts/                        # Python migration/audit scripts
│   └── upgrade_v0.9.0/             # 5 one-time migration scripts (executed) + common.py
├── skill/                          # [NEW v0.10.1] bundled AI-agent skills
│   ├── notion-cli/SKILL.md
│   └── workspace-lint/             # validator + references
├── workspace-lint.yaml             # [NEW v0.10.1] project lint config
├── lifeos.config.default.json      # embedded default config (include_str!'d)
├── .env                            # NOTION_API_TOKEN (gitignored)
├── AGENTS.md, README.md, ONTOLOGY.md
└── install.sh                      # one-line install script
```

---

## 3. Tool Inventory (31 tools)

### 3.1 Tools listed in `tools/list` (30)

| # | Tool | Source | Purpose |
|---|------|--------|---------|
| 1 | `get_schema` | mod.rs | DB schemas with entry types + holonic roles |
| 2 | `query` | query.rs | Query any DB with filters, sort, entry_type, cycle, AI override |
| 3 | `mutate` | mutate.rs | Create/update/delete/upsert entries |
| 4 | `intelligence_briefing` | intelligence.rs | Role or cycle briefing (CEO/COO, lesser/greater/nexus) |
| 5 | `data_science` | data_science.rs | Temporal patterns, trajectories, correlations |
| 6 | `review_pipeline` | review.rs | Daily/weekly/monthly/quarterly reviews |
| 7 | `strategic_simulator` | strategic.rs | Cross-DB strategic analysis: OKRs, projects, campaigns |
| 8 | `sync_note` | sync_note.rs | Bidirectional Notion ↔ markdown sync |
| 9 | `holonic_synthesis` | holonic_synthesis.rs | Currency flow + G_z/P_z + drive assessments (consolidated) |
| 10 | `get_page` | relations.rs | Fetch entry with relations resolved to titles |
| 11 | `build_context` | build_context.rs | Relational neighborhood assembly (one call) |
| 12 | `trace` | relations.rs | Follow relations N levels deep |
| 13 | `ancestors` | relations.rs | Walk up hierarchy to root |
| 14 | `backlinks` | relations.rs | Find entries referencing a given page |
| 15 | `relational_graph` | relational_graph.rs | Inter-DB hierarchy tree with link counts |
| 16 | `link` | relations.rs | Create a relation between two entries |
| 17 | `unlink` | relation_ops.rs | Remove a single relation |
| 18 | `batch_link` | relation_ops.rs | Create multiple relations in one call |
| 19 | `orphans` | audit.rs | List entries with zero populated relations |
| 20 | `relational_gaps` | relational_gaps.rs | Surface orphaned + missing-ontology-expected relations |
| 21 | `validate_yaml` | validate_yaml.rs | Validate entries against 3-tier YAML schemas |
| 22 | `suggest_links` | audit.rs | Suggest cross-reservoir links via title similarity |
| 23 | `suggest_categorization` | suggest_categorization.rs | Suggest entry-types for uncategorized entries |
| 24 | `archetype_index` | ontology.rs | List all 22 HoloOS archetypes |
| 25 | `derive_type` | ontology.rs | Derive Holon Type from Valence Signature YAML |
| 26 | `valence_signature` | ontology.rs | Generate Valence Signature YAML template |
| 27 | `daily` | workflows.rs | Daily review workflow (gaps + synthesis + recent) |
| 28 | `dashboard` | workflows.rs | Overview: orphans, recent, gaps, health |
| 29 | `auto_enrich` | auto_enrich.rs | [NEW v0.10.1] Infer universal properties from entry-type |

### 3.2 Tools added in server.rs `tools/list` response (2)

| # | Tool | Source | Purpose |
|---|------|--------|---------|
| 30 | `expand` | relations.rs | Expand page IDs → {id, title, database} objects |
| 31 | `graph_metrics` | relations.rs | Overall relational graph metrics |

### 3.3 Backward-compat aliases (6, kept for legacy AI agents)

| Alias | Delegates to | Status |
|-------|--------------|--------|
| `query_override` | `query` (with override param) | Could remove in v0.11 |
| `validate` | `audit::execute_validate` (legacy Notion formula check) | Could remove in v0.11 |
| `energy_flow` | `holonic_synthesis` | Could remove in v0.11 |
| `drive_assessment` | `holonic_synthesis` | Could remove in v0.11 |
| `health_metrics` | `holonic_synthesis` | Could remove in v0.11 |
| `expand` (in dispatch but not tools/list) | — | Fixed in v0.10.1 — now in tools/list |

**YAGNI candidate:** Remove the 5 backward-compat aliases in v0.11. They add ~50 LOC to mod.rs and confuse AI agents (6 phantom tools in dispatch but not list). Verify no client depends on them first.

---

## 4. MCP Server (server.rs, 389 LOC)

### 4.1 Transport (v0.10.1 fixes)

| Bug | Status | Notes |
|-----|--------|-------|
| **B1** Sync stdin blocking async runtime | ✅ Fixed | Now uses `tokio::io::stdin()` + `AsyncBufReadExt`. Each message spawns on its own task. |
| **B2** Stale `instructions` field | ✅ Fixed | Auto-generated from `get_tool_definitions`. Mentions all 31 tools, correct currency flow. |
| **B3** No `notifications/cancelled` handler | ✅ Fixed | In-flight token map. **Mid-tool cancellation NOT implemented** — token checked at completion only. See Action U-3. |
| **B4** No `notifications/progress` | ❌ Not implemented | Long tools (holonic_synthesis over 6,900 entries) give no feedback. See Action U-4. |
| **B5** No batch JSON-RPC support | ✅ Fixed | Array payloads dispatched as separate requests. |
| **B6** `serde_json::to_string().unwrap()` panic | ✅ Fixed | Returns fallback error JSON. |
| **B7** stderr tracing pollution | ⚠ Partially mitigated | `RUST_LOG=warn` in smoke test. Production users can set `RUST_LOG=error`. |

### 4.2 Server architecture

```
LifeosServer (owns Arc<LifeOSConfig>, Arc<NotionClient>, Arc<SchemaCache>, Arc<Mutex<in_flight map>>)
  └── run() — async stdin loop, spawns ServerHandle per message
       ├── handle_message(raw) — parse + dispatch (batch or single)
       │    └── handle_single(req) — match on method
       │         ├── initialize → build_instructions() (auto-gen)
       │         ├── ping → ok({})
       │         ├── tools/list → get_tool_definitions + expand + graph_metrics
       │         ├── tools/call → register cancellation token, dispatch, check cancel, respond
       │         ├── notifications/cancelled → flip cancellation token
       │         ├── resources/list → 2 static resources
       │         └── resources/read → db-schemas or relation-graph
       └── send(msg) — async stdout write via OnceCell<Mutex<Stdout>>
```

**Concurrency model:** Each incoming JSON-RPC message spawns a fresh tokio task. Multiple tools can execute concurrently. Pings/cancellations are never blocked by long tools. ✅

---

## 5. Config System

### 5.1 Config loading priority (config.rs)

```
1. $LIFEOS_CONFIG env var (if set)
2. ./lifeos.config.json (if exists)
3. ../lifeos.config.json (if exists)
4. Embedded default (include_str!("../lifeos.config.default.json"))
```

### 5.2 Config schema (LifeOSConfig)

```rust
LifeOSConfig {
    api_version: String,                  // "2025-09-03"
    rate_limit: RateLimitConfig,          // 3.0 req/sec, 300s cache
    databases: HashMap<String, DbConfig>, // 5 DBs (keys: matrix, potentiator, nexus, significator, greatway)
    holonic: Option<HolonicConfig>,       // version, currencies, drives, cycles, status_progressions, transmutation_map, nexus_firing, drive_effects, yaml_schemas_path
    briefings: Option<BriefingConfig>,    // roles + modules
    notion: Option<NotionConfig>,         // api_key fallback (env preferred)
}
```

### 5.3 Runtime resolution

On `lifeos discover` or first MCP start:
1. Load config (from file or embedded default)
2. For each DB: if `data_source_id` is placeholder (`00000000-*`), search Notion by name → resolve real ID
3. `SchemaCache::init` fetches all 5 DB schemas in parallel → 297 property mappings
4. `propagate_to_config` writes discovered properties back into config
5. If no config file existed, save resolved config to `./lifeos.config.json` (bootstrap fix, v0.10.1)

### 5.4 Config drift items

- **C-1:** `lifeos.config.json` is gitignored but `lifeos.config.default.json` is committed + `include_str!`'d. After Phase 3, both have the new DB names. ✅
- **C-2:** `entry_type_descriptions` field in HolonicConfig is **deprecated** (v0.7+) but still in the struct for backward compat. YAGNI candidate — remove in v0.11.
- **C-3:** `properties` field in DbConfig is **legacy** (v0.7+ uses `discovered_properties`). Still present for fallback. YAGNI candidate.

---

## 6. Schema System (3-tier YAML)

### 6.1 Hierarchy

```
schemas/
├── universal/holon_coordinate.yaml       (6 props, 3 validation rules — applies to ALL entries)
├── per_db/{matrix,potentiator,nexus,significator,greatway}.yaml
│   (0 props — only relations + entry_types + default_archetype_mapping)
└── per_entry_type/*.yaml                  (31 files — only for relocated entry-types or those with validation rules)
```

### 6.2 Validation rules (3, hardcoded in Rust)

1. `nexus_kind_consistency` — Process.Kind constrains which relations can populate
2. `stage_type_independence` — Digestion Stage and Holon Type both set or both empty
3. `complex_archetype_consistency` — (role, complex) must be one of 22 named archetypes

### 6.3 Schema drift items

- **S-1:** Per-DB YAML files declare 0 properties (all props come from universal). This is by design — the per_db layer only adds relations + entry_types. ✅
- **S-2:** 31 per_entry_type files exist. Some may be stale (entry-types that were renamed or removed). Run `lifeos validate-yaml --all` to check.
- **S-3:** **Missing validation rule:** `shadow_pattern_db_consistency` (Sinkhole only on World, Dark-* only on State, Golden-* only on Possibility). See Action U-7.

---

## 7. Sync Engine (sync/)

Bidirectional Notion ↔ local markdown vault sync. 5 files:
- `pull.rs` — Notion → vault
- `push.rs` — vault → Notion
- `merge.rs` — conflict resolution
- `watch.rs` — filesystem watcher (notify-debouncer-mini)
- `page.rs` — single-page sync

**Status:** Functional but untested in this audit. The `sync_note` MCP tool wraps it. **Untested in v0.10.1 smoke suite** — add a sync round-trip test in v0.11.

---

## 8. Build + Test Infrastructure

### 8.1 Build

```bash
. "$HOME/.cargo/env"
cargo build --target x86_64-unknown-linux-gnu
```
- ✅ 0 errors, 0 warnings
- Binary: `target/x86_64-unknown-linux-gnu/debug/lifeos` (136 MB debug, ~10 MB release with LTO + strip)
- Release profile: `opt-level=3, lto=true, strip=true`

### 8.2 Test suite

| Test | Command | Status |
|------|---------|--------|
| Schema self-test | `lifeos validate-yaml --self-test` | ✅ 37 schemas load |
| Discover smoke | `lifeos discover` | ✅ 5 DBs resolve, 297 mappings, 63 edges |
| Dashboard | `lifeos dashboard` | ✅ Returns (86% orphan rate baseline) |
| MCP smoke (14 tests) | `python3 /home/z/my-project/scripts/mcp_smoke_test.py` | ✅ 14/14 pass |
| workspace-lint | `python3 skill/workspace-lint/scripts/workspace_lint.py` | ✅ 0 errors, exit 0 |

### 8.3 Missing test coverage

- **T-1:** No `cargo test` unit tests (Rust inline tests). All testing is via the smoke script.
- **T-2:** No sync round-trip test (pull → modify → push → verify).
- **T-3:** No `mutate` round-trip test (create → get_page → update → delete).
- **T-4:** No `link` round-trip test in the smoke suite (was verified manually in Phase 5, not automated).

---

## 9. YAGNI Candidates (deadweight + redundancies)

| ID | Item | LOC saved | Risk | Recommendation |
|----|------|-----------|------|----------------|
| **Y-1** | 5 backward-compat alias tools (`query_override`, `validate`, `energy_flow`, `drive_assessment`, `health_metrics`) | ~50 LOC in mod.rs | Low — verify no client depends on them | Remove in v0.11 |
| **Y-2** | `entry_type_descriptions` field in HolonicConfig (deprecated v0.7+) | ~15 LOC in config.rs | Low — auto-discovery replaces it | Remove in v0.11 |
| **Y-3** | `properties` field in DbConfig (legacy, replaced by `discovered_properties`) | ~10 LOC in config.rs | Low — `notion_prop()` falls back to it | Remove in v0.11 |
| **Y-4** | `Possibility.Digestion Status` (3-state) — redundant with `Digestion Stage` (9-state) | 1 Notion property | Medium — verify no formula/script references it | Verify usage, then remove via Notion UI |
| **Y-5** | `Process.Synthesis State` (4-state) — overlaps `Digestion Stage` | 1 Notion property | Medium — may be user-facing simplification | Verify usage, then remove |
| **Y-6** | `Identity.Stage` (life-era + status conflated) | 1 Notion property | Medium — needs rename + option cleanup | Rename to `Life-Era`, remove Active/Evolving/Archived |
| **Y-7** | 5 executed migration scripts in `scripts/upgrade_v0.9.0/` (keep `common.py`, archive rest) | ~2,000 LOC Python | Low — already executed | Move to `scripts/archive/upgrade_v0.9.0/` |
| **Y-8** | `audit::execute_validate` (legacy Notion formula check, superseded by `validate_yaml`) | ~180 LOC in audit.rs | Low — `validate_yaml` is the replacement | Remove after Y-1 |
| **Y-9** | `audit::execute_suggest_links` — overlaps with `suggest_categorization` + `relational_gaps` | ~100 LOC in audit.rs | Medium — verify distinct purpose | Consolidate in v0.11 |
| **Y-10** | `auto_enrich --mode tag --apply=true` path | ~30 LOC in auto_enrich.rs | Low — user prefers manual curation | Remove `--apply` for tag mode; keep as suggestion-only |

---

## 10. Upgrade Candidates (additions)

| ID | Item | Effort | Value |
|----|------|--------|-------|
| **U-1** | Per-relation semantic hints in `link` tool response | 1 day | AI agents learn what each relation means ontologically |
| **U-2** | Auto-link apply mode (currently dry-run only) | 2 days | User can opt-in to auto-link daily logs to active parents |
| **U-3** | Mid-tool cancellation (thread CancellationToken through execute functions) | 2 days | Long tools can be aborted mid-batch, not just at completion |
| **U-4** | `notifications/progress` for long-running tools | 1 day | User sees "page 50/6900..." during holonic_synthesis |
| **U-5** | `cargo test` unit tests for tool dispatch + config loading | 2 days | Catch regressions before smoke test |
| **U-6** | Sync round-trip test in smoke suite | 1 day | Verify pull → modify → push → verify |
| **U-7** | `shadow_pattern_db_consistency` validation rule | 4 hours | Sinkhole only on World, Dark-* only on State, Golden-* only on Possibility |
| **U-8** | Fill-rate audit tool (`lifeos fill-rate --db X --days 30`) | 1 day | Identify properties with <5% fill → YAGNI candidates |
| **U-9** | Demote `auto_enrich` to suggestion-only (remove `--apply` for tag mode) | 2 hours | Aligns with user's manual-curation preference |
| **U-10** | Rename `Identity.Stage` → `Life-Era` + clean options | 1 hour (Notion UI) | Fixes D-ID-1 |

---

## 11. Architecture Verdict

The LifeOS architecture is **structurally sound and operationally functional** as of v0.10.1. The v0.10.0→v0.10.1 sprint:

- ✅ Fixed all critical MCP transport bugs (B1, B2, B5, B6)
- ✅ Renamed 5 DBs to plain-English (AI-agent UX)
- ✅ Added `auto_enrich` tool (suggestion engine for universal properties)
- ✅ Fixed 3 pre-existing bugs (clap collisions, double-wrap in link/unlink/batch_link)
- ✅ Bundled skills (notion-cli, workspace-lint) + codified dev protocol (AGENTS.md)

**Remaining work is prioritized in [AUDIT_v0.10.1_ACTION_TRACKER.md](AUDIT_v0.10.1_ACTION_TRACKER.md).**

The 10 YAGNI candidates (Y-1 through Y-10) would remove ~2,400 LOC of deadweight. The 10 upgrade candidates (U-1 through U-10) would add ~10 days of work but unlock the next tier of operational maturity (mid-tool cancellation, progress feedback, fill-rate audits, semantic hints).

**No urgent fixes required.** The system is stable for daily use.
