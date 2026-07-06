# LifeOS v0.10.1 — Action Tracker

> **Purpose:** Single source of truth for fix / refine / upgrade / refactor / YAGNI items
> across the LifeOS system. Updated every iteration. Items completed are moved to
> the bottom; items pending are prioritized at the top.
>
> **Companion docs:** [AUDIT_v0.10.1_DB_SCHEMA.md](AUDIT_v0.10.1_DB_SCHEMA.md), [AUDIT_v0.10.1_ARCHITECTURE.md](AUDIT_v0.10.1_ARCHITECTURE.md)
>
> **Status legend:** 🔴 urgent · 🟠 high · 🟡 medium · 🟢 low · ⚪ informational

---

## Active Progress (in-flight this sprint)

_None — current sprint (v0.10.1) is complete; next sprint items are in Pending._

---

## Pending Progress (next sprint candidates, prioritized)

### 🟠 High priority (do next)

| ID | Category | Item | Effort | Why first |
|----|----------|------|--------|-----------|
| **U-9** | refactor | Demote `auto_enrich` to suggestion-only — remove `--apply` for `mode=tag` | 2h | Aligns with user's explicit preference for manual curation. Prevents accidental bulk-writes. Small, safe change. |
| **D-ID-2** | fix | Re-tag the 1 Identity entry using `Complex=Soul` → `Spirit` (or `None`), then remove `Soul` option via Notion UI | 5min (manual) | Smallest fix that closes a real correctness gap. Unblocks `complex_archetype_consistency` validation rule. |
| **U-7** | upgrade | Add `shadow_pattern_db_consistency` validation rule (Sinkhole only on World, Dark-* only on State, Golden-* only on Possibility) | 4h | Smallest upgrade that proves the validator pattern works. Catches user errors before they propagate. |
| **D-ID-1** | fix | Rename `Identity.Stage` → `Life-Era` + remove `Active/Evolving/Archived` options (conflated with `Status`) | 1h (Notion UI) | Fixes ontological conflation. Makes `Stage` actually mean what it claims. |

### 🟡 Medium priority (do after high)

| ID | Category | Item | Effort | Why |
|----|----------|------|--------|-----|
| **U-1** | upgrade | Per-relation semantic hints in `link` tool response (1-line meaning per relation) | 1 day | AI agents learn what each relation means ontologically. Reduces wrong-relation mistakes. |
| **U-3** | upgrade | Mid-tool cancellation — thread `CancellationToken` through execute functions | 2 days | Long tools (holonic_synthesis over 6,900 entries) currently can only be cancelled at completion. User must kill client to abort. |
| **U-4** | upgrade | `notifications/progress` for long-running tools | 1 day | User sees "page 50/6900..." during long queries. Currently zero feedback. |
| **U-8** | upgrade | Fill-rate audit tool (`lifeos fill-rate --db X --days 30`) | 1 day | Identifies properties with <5% fill → YAGNI candidates. Data-driven cleanup. |
| **D-POSS-2** | YAGNI | Remove `Possibility.Digestion Status` (redundant with `Digestion Stage`) | 1h (verify) + 5min (Notion UI) | 3-state status is a coarser projection of 9-stage select. Verify no formula references it, then remove. |
| **D-PROC-2** | YAGNI | Remove `Process.Synthesis State` (overlaps `Digestion Stage`) | 1h (verify) + 5min (Notion UI) | Same pattern as D-POSS-2. Verify usage first. |
| **D-PROC-4** | YAGNI | Audit Process's 35 properties for <5% fill rate | 2h | Process has the highest property count. Run `U-8` tool once built, then delete low-fill properties. |
| **Y-10** | refactor | Demote `auto_enrich --apply` for tag mode (same as U-9, listed for cross-ref) | — | Duplicate of U-9. |

### 🟢 Low priority (do when convenient)

| ID | Category | Item | Effort | Why |
|----|----------|------|--------|-----|
| **Y-1** | YAGNI | Remove 5 backward-compat alias tools (`query_override`, `validate`, `energy_flow`, `drive_assessment`, `health_metrics`) | 2h | ~50 LOC in mod.rs. Verify no client depends on them first. |
| **Y-2** | YAGNI | Remove `entry_type_descriptions` field from HolonicConfig (deprecated v0.7+) | 1h | ~15 LOC. Auto-discovery replaces it. |
| **Y-3** | YAGNI | Remove `properties` field from DbConfig (legacy, replaced by `discovered_properties`) | 1h | ~10 LOC. `notion_prop()` falls back to it. |
| **Y-7** | YAGNI | Archive 5 executed migration scripts in `scripts/upgrade_v0.9.0/` (keep `common.py`) | 30min | ~2,000 LOC Python. Already executed. Move to `scripts/archive/`. |
| **Y-8** | YAGNI | Remove `audit::execute_validate` (legacy Notion formula check, superseded by `validate_yaml`) | 1h | ~180 LOC. Depends on Y-1 (remove `validate` alias first). |
| **Y-9** | refactor | Consolidate `audit::execute_suggest_links` with `suggest_categorization` + `relational_gaps` | 3h | ~100 LOC. Verify distinct purpose first. |
| **D-PROC-1** | fix | Strip emojis from Process Status options (`💡 Identified` → `Identified`) | 15min (Notion UI, 4 clicks per option) | Notion API can't rename status options. Manual. Makes filtering easier. |
| **D-WORLD-2** | refine | Audit World's 18 entry-types for underuse; consider consolidating `Annual Goal` + `Quarterly Goal` + `Goal` | 2h | UX decision — defer to user. Run `suggest_categorization` first. |
| **D-POSS-3** | refine | Decide on `Possibility.Documents` relation → external Documents DB (`df692710-*`) | 30min (decision) | Either archive the relation or document that external DBs are intentionally linked. |
| **D-ID-3** | refine | Verify Identity's 3 State-relations (`Anchored In`, `Generated From`, `Rewrites`) are distinct | 1h | May be legacy duplicates. Check fill rates + semantic intent. |
| **U-5** | upgrade | Add `cargo test` unit tests for tool dispatch + config loading | 2 days | Catch regressions before smoke test. Currently 0 Rust unit tests. |
| **U-6** | upgrade | Add sync round-trip test to smoke suite (pull → modify → push → verify) | 1 day | Sync engine is currently untested. |
| **U-2** | upgrade | Auto-link apply mode (currently dry-run only) | 2 days | Requires multi-step "active parent" resolution. Riskier — defer until U-1 (semantic hints) is done. |

### ⚪ Informational (no action required, tracked for awareness)

| ID | Category | Item | Notes |
|----|----------|------|-------|
| **D-STATE-1, D-POSS-1, D-PROC-3, D-WORLD-1** | cosmetic | 8 relations carry legacy `database_id` field pointing to deleted pre-v0.9.0 Potentiator (`0baacff9-*`) | **Functionally harmless** — `data_source_id` (used by Notion API 2025-09-03) is correct. Cosmetic only: Notion UI may show "deleted database" in some views. Fix would require deleting + recreating each relation via Notion UI. Defer indefinitely. |
| **C-2, C-3** | debt | Deprecated config fields (`entry_type_descriptions`, `properties`) kept for backward compat | Tracked in Y-2, Y-3. |
| **T-1** | debt | No `cargo test` unit tests | Tracked in U-5. |
| **B-7** | debt | stderr tracing pollution at INFO level | Mitigated via `RUST_LOG=warn`. Not blocking. |

---

## Completed Progress (this sprint — v0.10.1)

### Phase 1 — Data-layer fixes ✅

| ID | Item | Status |
|----|------|--------|
| P1-1a | Repoint 8 ghost-database relations (`0baacff9-*` → `a1769af1-*` data_source_id) | ✅ Verified via successful link test |
| P1-1b | Add `Sinkhole of Indifference` shadow option to all 5 DBs | ✅ All 5 DBs updated |
| P1-1c | Remove `Soul` Complex option from Identity | ⚠ Deferred — 1 entry uses it (see D-ID-2) |
| P1-1d | Fix Nexus Status emoji mismatch | ✅ Reconciled via config (Notion API can't rename status options) |
| P1-1e | Fix `discover` bootstrap bug (write embedded default if no config file) | ✅ Zero-config startup works |

### Phase 2 — MCP transport bug fixes ✅

| ID | Item | Status |
|----|------|--------|
| P2-B1 | Convert server.rs to async tokio::io | ✅ Pings flow during long tools |
| P2-B2 | Auto-generate `instructions` from get_tool_definitions | ✅ No phantom tools, correct currency flow |
| P2-B3 | Add `notifications/cancelled` handler | ✅ Token map wired (mid-tool cancel deferred → U-3) |
| P2-B5 | Add batch JSON-RPC support | ✅ Verified via smoke test |
| P2-B6 | Safe serialize (no `.unwrap` panic) | ✅ Fallback error JSON |
| P2-2f | Add `expand` + `graph_metrics` to tools/list | ✅ Both visible |

### Phase 3 — Rename + embed ✅

| ID | Item | Status |
|----|------|--------|
| P3-3a | Rename 5 DBs in Notion (Matrix→State, etc.) | ✅ All 5 renamed |
| P3-3b | Update lifeos.config.default.json + local config | ✅ New names + rich descriptions + Nexus emoji reconciliation + version 5.1 |
| P3-3c | Per-relation semantic hints | ❌ Deferred → U-1 |

### Phase 4 — YAGNI + auto-tooling ✅

| ID | Item | Status |
|----|------|--------|
| P4-4b | Build `auto_enrich` tool (tag + link modes) | ✅ 280 LOC, 50+ rules, end-to-end verified |
| P4-4c | Auto-link apply mode | ❌ Deferred → U-2 (currently dry-run only) |
| P4-4a | Move Digestion Stage/Archetype Role/Complex to YAML-schema-only on daily entry-types | ❌ Deferred — auto_enrich makes this less urgent. User prefers manual curation. |

### Phase 5 — Build + test ✅

| ID | Item | Status |
|----|------|--------|
| P5-5a | cargo build --target x86_64-unknown-linux-gnu | ✅ 0 errors, 0 warnings |
| P5-5b | lifeos validate-yaml --self-test | ✅ 37 schemas load |
| P5-5c | lifeos discover | ✅ 5 DBs resolve with new names, 297 mappings, 63 edges |
| P5-5d | lifeos dashboard | ✅ 86% orphan rate baseline |
| P5-5e | MCP smoke test (14 tests) | ✅ 14/14 pass |
| P5-5f | MCP tool call smoke test (get_schema + query) | ✅ Verified |
| P5-5g | auto_enrich dry-run + apply test | ✅ 5 State entries tagged, 0 errors |

### Phase 6 — Protocol + skill bundle ✅

| ID | Item | Status |
|----|------|--------|
| P6-1 | Rewrite AGENTS.md with full development protocol | ✅ 205 lines, 11 sections, Two Non-Negotiables codified |
| P6-2 | Store notion-cli.md as ./skill/notion-cli/SKILL.md | ✅ |
| P6-3 | Set up workspace-lint for working directory | ✅ Config + validator + 3 upstream patches |
| P6-4 | Commit + push v0.10.1 to main | ✅ Commit `3f54b01` on origin/main |

### Pre-existing bugs fixed en route ✅

| ID | Item | Status |
|----|------|--------|
| FIX-1 | clap short-flag collisions in cli/mod.rs (`-f`, `-s`) | ✅ filter_property + sort_property + sort_direction → long-only |
| FIX-2 | Double-wrap bug in `execute_link` (relations.rs) | ✅ Pass inner map directly to update_page |
| FIX-3 | Double-wrap bug in `execute_unlink` (relation_ops.rs) | ✅ Same fix |
| FIX-4 | Double-wrap bug in `execute_batch_link` (relation_ops.rs) | ✅ Same fix |

---

## Architectural Preferences (remember these)

These are codified preferences from the user that MUST be respected in all future work:

1. **No bulk-tagging.** Universal properties (Archetype Role, Complex, Drive Activation, Shadow Pattern, Holon Type) must be manually curated by the user. Tools may *suggest* (dry-run), never *apply* without explicit per-entry approval. → See U-9 (demote auto_enrich --apply).

2. **Manual relation curation.** Per §6.1 of AGENTS.md: "Every relation is a deliberate choice. Tools surface gaps and suggest connections, but the user must approve each link." Auto-link apply mode (U-2) must be opt-in AND require per-entry confirmation, never bulk.

3. **5 DBs only.** The 137 auxiliary DBs in the Notion workspace are out of scope. Don't propose merging/archiving them.

4. **Push to main + workspace-lint after each iteration.** Non-negotiable. No feature branches.

5. **YAGNI aggressive.** Remove deadweight + redundancies proactively. If a property/tool/script has 0% usage after 30 days, delete it. Don't keep things "for the future."

6. **Renamed DBs are canon.** Use the new names (State/Possibility/Process/Identity/World) in all user-facing text. Use the old config keys (matrix/potentiator/nexus/significator/greatway) in code that indexes into `config.databases`.

7. **HoloOS ontology is the foundation.** Every LifeOS design decision must trace back to a principle in ONTOLOGY.md or HoloOS `_THEORY/02_Ontology/`. Misalignments (like the G_z/P_z cycle-metric split, M1) are tracked as bugs, not features.

---

## How to Use This Tracker

- **At the start of every sprint:** Read this file. Pick 1-3 items from "Pending → High priority". Move them to "Active Progress".
- **During the sprint:** Update item status as you work. If you discover new issues, add them to "Pending" with appropriate priority.
- **At the end of every sprint:** Move completed items to "Completed Progress". Update the worklog with a pointer to this file.
- **Never delete items** — move them to "Completed" so we have a record. The only exception is duplicate IDs (mark with "Duplicate of X" and remove).

---

*Last updated: 2026-07-06 by LifeOS-Architect (Task ID 5). Next review: when next sprint starts.*
