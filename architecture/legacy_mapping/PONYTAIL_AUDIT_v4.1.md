# Ponytail Audit — LifeOS v4.1 (2026-07-07)

> Over-engineering audit only, not correctness. Scanned the whole tree.
> Ranked biggest cut first.

---

## FINDINGS (ranked biggest cut first)

```
delete   5 dead .rs files (drive_assessment, energy_flow, health_metrics, holonic_synthesis, ontology) — not in pub mod, unreachable. Replace with nothing. [lifeos-core/src/tools/]
delete   6 executed v0.9.0 migration scripts (01-05 + common.py) — already ran, one-time. Replace with nothing. [scripts/upgrade_v0.9.0/]
delete   18 historical architecture docs (BLUEPRINT_v4, REFINEMENT_v2, R_AND_D_v3, BRAINSTORM_*, REFACTOR_PLAN_*, AUDIT_v0.10.*, etc.) — superseded by v4.1. Keep only FORMAL_SPEC_v4.1 + UTILITY_LAYER_BRAINSTORM. [architecture/legacy_mapping/]
delete   AUDIT_ponytail_ontology.md, AUDIT_v0.10.1_*, AUDIT_v0.10.2_*, AUDIT_v0.10.3_*, UPGRADE_v0.9.0_PLAN.md — all historical snapshots. Replace with nothing. [root + architecture/]
delete   config.rs deprecated holonic structs (TransmutationDef, NexusFiringConfig, DriveEffectDef, CycleConfig, CycleDefinition) + their methods (transmutation_def, nexus_firing_config, cycle_reservoirs) — 60 LOC dead code. Replace with nothing. [lifeos-core/src/config.rs:69-101, 417-430]
delete   config.rs deprecated fields (currencies, drives, cycles, transmutation_map, nexus_firing, drive_effects, entry_type_descriptions) — all #[serde(default)] + unused in v4.1. Replace with v4.1 fields only. [lifeos-core/src/config.rs:37-62]
delete   config.rs deprecated DbConfig fields (entry_type_descriptions, properties) — both marked DEPRECATED. Replace with nothing. [lifeos-core/src/config.rs:145-170]
delete   query.rs query_override (120 LOC, lines 441-561) — backward-compat alias that delegates to query with override param. Replace with `query` param. [lifeos-core/src/tools/query.rs]
delete   intelligence.rs drive_balance + lesser_cycle + greater_cycle + nexus + reservoir_health modes — all reference old holonic concepts. Keep only role + module modes. [lifeos-core/src/tools/intelligence.rs:30,523-525]
delete   audit.rs execute_validate (180 LOC, lines 161-280) — old Notion formula check, superseded by validate_yaml. Replace with nothing. [lifeos-core/src/tools/audit.rs]
delete   audit.rs execute_suggest_links — overlaps with suggest_categorization + relational_gaps. Replace with relational_gaps. [lifeos-core/src/tools/audit.rs]
yagni    sync/ module (1780 LOC across 5 files) — bidirectional Notion↔markdown vault sync. Only 1 implementation (pull/push/merge/watch). If the user doesn't use the local vault, this is dead weight. Verify usage before cutting. [lifeos-core/src/sync/]
yagni    toon_format.rs (125 LOC) — custom YAML-front-matter text encoder/decoder. Only 1 consumer (tools that return TOON format). Replace with serde_json::to_string_pretty. [lifeos-core/src/toon_format.rs]
yagni    vault/mod.rs (69 LOC) — local markdown vault index. Only used by sync module. If sync is cut, this goes too. [lifeos-core/src/vault/mod.rs]
shrink   data_science.rs (1110 LOC) — largest tool file. Has old holonic cycle params (lesser/greater) that are now dead. Strip cycle logic, keep entry_type + temporal analysis. ~400 LOC removable. [lifeos-core/src/tools/data_science.rs]
shrink   query.rs (723 LOC) — has both query + query_override. Merge override into query as a parameter. ~120 LOC removable. [lifeos-core/src/tools/query.rs]
shrink   config.rs (448 LOC) — after removing deprecated fields + structs, ~100 LOC removable. [lifeos-core/src/config.rs]
shrink   intelligence.rs (533 LOC) — after removing 5 dead modes, ~200 LOC removable. [lifeos-core/src/tools/intelligence.rs]
shrink   37 YAML schema files — per_entry_type files reference old entry-types that no longer exist (Pattern, Threshold, Knowledge-Category, etc.). Delete stale files, keep only v4.1-relevant ones. ~15 files removable. [schemas/per_entry_type/]
delete   4 visualization Python scripts (visualize*.py) — one-time generation, not needed at runtime. Move to scripts/archive/. [architecture/legacy_mapping/visualize*.py]
delete   5 visualization PNG files — generated artifacts, not source code. Add to .gitignore. [architecture/legacy_mapping/visualizations/*.png]
```

## IMPLEMENTATION GAPS + BUGS

```
BUG     intelligence.rs:524 — drive_balance mode returns stub string but is still listed in schema enum. Remove from enum.
BUG     intelligence.rs — lesser_cycle/greater_cycle/nexus/reservoir_health modes reference old holonic config fields that are now Option (will panic if accessed). Fix or remove.
BUG     data_science.rs — references old holonic cycle params (lesser/greater). Will return empty results in v4.1.
GAP     capture.rs — doesn't link to Context.Person for Relational logs (auto-detection doesn't resolve person names).
GAP     morning.rs — "No active goals found" because filter uses "Annual-Goal" but Notion data has "Annual Goal" (space, not hyphen). Need to handle both.
GAP     trace_trajectory.rs — doesn't handle entries with no Parent relation (returns "top of hierarchy" even if the entry is a Task with no parent set).
GAP     surface_synthesis.rs — only counts by entry type, doesn't detect actual content patterns (e.g. "running 5x/week"). Needs NLP or keyword clustering.
GAP     No WebUI exists — the architecture mentions Notion Dashboard as the frontend, but no custom web interface has been built.
GAP     No CLI commands for the 6 new tools (morning, capture, cycle_health, trace_trajectory, gap_analysis, surface_synthesis) — only accessible via MCP.
GAP     validate_yaml — references old holonic validation rules (nexus_kind_consistency, stage_type_independence, complex_archetype_consistency, shadow_pattern_db_consistency) that no longer apply in v4.1.
```

## WHAT TO EXPAND / ENHANCE / EVOLVE

```
EXPAND  morning tool — add "Where I am in my large-scale trajectory" (trace from today's Task up to Vision-Statement)
EXPAND  capture tool — add voice-to-text + mobile quick-capture interface
EXPAND  cycle_health — add per-flow recommendations + auto-link suggestions
EXPAND  surface_synthesis — add actual pattern detection (keyword clustering, frequency analysis, anomaly detection)
EXPAND  gap_analysis — add trend visualization (↑/↓/→) + projected time-to-close
EVOLVE  intelligence_briefing — redesign for v4.1 layers (teleological_pull / historical_record / action_interface) instead of old roles/cycles
EVOLVE  suggest_categorization — rewrite heuristics for v4.1 entry-types (Purpose/Value/Principle/Vision-Statement/etc.)
EVOLVE  data_science — strip old holonic cycle logic, add v4.1 layer-based analysis (Reference trends, Strategic progress, Execution velocity)
EVOLVE  YAML schemas — rewrite for v4.1 entry-types, add Context YAML formula validation
EVOLVE  AGENTS.md — update for v4.1 tool list (35 tools, 6 new utility tools), remove old holonic references
BUILD   WebUI — custom dashboard for the morning view (trajectory position + today's tasks + profile gaps + capture interface)
BUILD   CLI commands for 6 new tools (lifeos morning, lifeos capture "text", lifeos cycle-health, etc.)
```

---

## NET SUMMARY

| Category | Lines removable | Files removable |
|----------|----------------|----------------|
| Dead .rs files | 2,095 | 5 |
| Dead Python scripts | 2,339 | 6 |
| Historical docs | ~5,000 | 18 |
| Deprecated config structs | ~100 | 0 |
| Deprecated query_override | ~120 | 0 |
| Deprecated intelligence modes | ~200 | 0 |
| Deprecated audit.rs functions | ~280 | 0 |
| Stale YAML schemas | ~800 | 15 |
| Visualization artifacts | ~1,500 | 9 |
| **TOTAL** | **~12,434** | **53** |

**Net lines removable: ~12,434** (out of ~36,906 total = 34% reduction)
**Net files removable: 53** (out of ~144 total = 37% reduction)
**Dependencies removable: 0** (all deps are still used by remaining code)

**After cleanup: ~24,472 LOC across ~91 files.** Lean, focused, every file serving v4.1.

---

*Ponytail audit complete. 34% of the repo is dead weight from the old holonic architecture. The biggest wins: delete 5 dead .rs files (2,095 LOC), delete 18 historical docs (5,000 LOC), shrink config.rs + intelligence.rs + data_science.rs (700 LOC).*
