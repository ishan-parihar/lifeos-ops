# LifeOS v0.10.1 — DB Schema + Structure Audit

> **Audit date:** 2026-07-06
> **Auditor:** LifeOS-Architect
> **Scope:** The 5 LifeOS DBs in Notion (renamed in v0.10.1: State, Possibility, Process, Identity, World) — schema, entry counts, relation topology, drift from canonical.
> **Method:** Direct Notion API queries (API version 2025-09-03) + `lifeos dashboard` + `lifeos discover` + per-DB schema inspection via `/v1/data_sources/{id}`.
> **Companion docs:** [AUDIT_v0.10.1_ARCHITECTURE.md](AUDIT_v0.10.1_ARCHITECTURE.md), [AUDIT_v0.10.1_ACTION_TRACKER.md](AUDIT_v0.10.1_ACTION_TRACKER.md), [ONTOLOGY.md](ONTOLOGY.md)

---

## 1. Executive Summary

| DB (new name) | Old name | Entries | Properties | Universal props (6) | Drift items |
|---------------|----------|---------|------------|---------------------|-------------|
| **State** | Matrix | 39 | 25 | ✅ 6/6 | 2 |
| **Possibility** | Potentiator | 6,900 | 27 | ✅ 6/6 | 3 |
| **Process** | Nexus | 797 | 35 | ✅ 6/6 | 4 |
| **Identity** | Significator | 92 | 29 | ⚠ 6/6 + 1 extra | 3 |
| **World** | GreatWay | 608 | 27 | ✅ 6/6 | 2 |
| **TOTAL** | — | **8,436** | **143** | — | **14** |

**Total workspace databases visible to integration: 142** — only 5 are LifeOS; the other 137 are legacy/backup/misc (out of scope per user direction).

**Relation topology:** 63 inter-DB relation edges discovered. 13 dual_property relations encode the fractal coupling. 5 relations still carry a legacy `database_id` field pointing to the deleted pre-v0.9.0 Potentiator (`0baacff9-*`) — functionally harmless (the `data_source_id` field is correct, and that's what Notion API 2025-09-03 uses) but cosmetically misleading.

**Orphan rate:** 86% (373/431 sampled; real orphan rate across all 8,436 entries is likely higher since daily logs dominate and rarely have relations).

---

## 2. Per-DB Schema Inventory

### 2.1 State (was Matrix) — 39 entries, 25 properties

**Holonic role:** Current-state organizer (lesser cycle). Ingests Catalyst, produces Experience.
**Entry-types (3):** Pattern, Threshold, Foundation
**Status:** Active → Evolving → Archived
**Currency flow:** Catalyst in → Experience out

**Property inventory:**
| # | Property | Type | Notes |
|---|----------|------|-------|
| 1 | Name | title | — |
| 2 | Entry Type | multi_select (3) | Pattern, Threshold, Foundation |
| 3 | Status | status (3) | Active, Evolving, Archived |
| 4 | Archetype Role | select (8) | ✅ universal |
| 5 | Complex | select (4) | ✅ Mind/Body/Spirit/None |
| 6 | Drive Activation | multi_select (4) | ✅ universal |
| 7 | Shadow Pattern | select (6) | ✅ includes Sinkhole (added v0.10.1) |
| 8 | Review Cadence | select (4) | Annual, Bi-Annual, Quarterly, Event-Driven |
| 9 | Crystallization Date | date | — |
| 10 | Last Reviewed | date | — |
| 11 | Next Review | date | — |
| 12 | Integration Weight | number | — |
| 13 | ID | unique_id | — |
| 14 | YAML Metadata | rich_text | — |
| 15-25 | 11 relation props | relation | see §3 below |

**Drift items:**
- **D-STATE-1:** `Generated From` relation's `database_id` field still points to ghost `0baacff9-*` (data_source_id is correct → real Potentiator). Cosmetic only.
- **D-STATE-2:** No `Digestion Stage` property. Per universal schema, Digestion Stage is on Process + Possibility only — State is correct to omit. ✅

### 2.2 Possibility (was Potentiator) — 6,900 entries, 27 properties

**Holonic role:** Latent-state generator (lesser cycle). Ingests Experience, produces refined Catalyst.
**Entry-types (10):** Activity, Subjective, Relational, Systemic, Diet, Financial, Observation, Goal, Vision, Aspiration
**Status:** Raw → Digesting → Crystallized
**Currency flow:** Experience in → Catalyst out

**Property inventory (key items):**
| Property | Type | Notes |
|----------|------|-------|
| Entry Type | select (10) | **NOTE:** select (not multi_select) — only one type per entry |
| Digestion Stage | select (9) | ✅ universal (Process + Possibility only) |
| Digestion Status | status (3) | Raw, Digesting, Crystallized — **redundant with Digestion Stage?** see D-POSS-2 |
| Date | date | Used by daily logs |
| Amount / Duration / Month Label / Quarter Label / Week Label | formula | Auto-computed from Date |
| Distillation | rich_text | — |
| Documents | relation (dual) | → target `df692710-*` (Documents DB — external, not LifeOS) |
| 8 other relation props | relation | see §3 |

**Drift items:**
- **D-POSS-1:** `Crystallizes Into` relation's `database_id` → ghost `0baacff9-*` (data_source_id correct). Cosmetic.
- **D-POSS-2:** **Redundancy between `Digestion Stage` (9-stage select) and `Digestion Status` (3-state status: Raw/Digesting/Crystallized).** The 9-stage select already encodes progression (stages 1-2 = Raw, 3-6 = Digesting, 7-9 = Crystallized). `Digestion Status` is a coarser projection of the same information. YAGNI candidate — consider removing `Digestion Status` and replacing any references with a formula.
- **D-POSS-3:** `Documents` relation points to a Documents DB (`df692710-*`) that is NOT one of the 5 LifeOS DBs. This is an external integration. Either (a) archive the relation, or (b) document that external DBs are intentionally linked.

### 2.3 Process (was Nexus) — 797 entries, 35 properties

**Holonic role:** Contact-boundary (shared between lesser + greater cycles). All 4 currencies transmute here.
**Entry-types (13):** Opportunity, Directive, Risk, Insight, Reflection, Integration, Pattern, Note, Knowledge-Category, Knowledge-Atom, Decision, Crisis, Transformation-Event
**Status:** 💡 Identified → ✅ Activated → 🏆 Capitalized → 🧊 Archived
**Currency property:** `Kind` (Catalyst / Experience / Transformation / Choice)

**Property inventory (key items):**
| Property | Type | Notes |
|----------|------|-------|
| Category | select (13) | Entry-type discriminator |
| Kind | select (4) | Currency discriminator — **the only DB with this property** |
| Digestion Stage | select (9) | ✅ universal (Process + Possibility only) |
| Capture Method | select (5) | manual, web_clipper, api_ingest, voice_memo, import |
| Synthesis State | select (4) | raw_note, annotated, synthesized, applied — **overlaps with Digestion Stage?** see D-PROC-2 |
| Polarity Outcome | select (3) | Reconciled, Intensified, Crystallized |
| Priority | select (4) | Critical, High, Medium, Low |
| Highlight Count | number | — |
| Last Assessed | date | — |
| Raw Content | rich_text | — |
| Source URL | url | — |
| 14 relation props | relation | see §3 |

**Drift items:**
- **D-PROC-1:** **Status emojis.** Options are `💡 Identified`, `✅ Activated`, `🏆 Capitalized`, `🧊 Archived`. Config now matches (reconciled in v0.10.1). But: emojis in status options make filtering harder (user must type the emoji). Consider stripping emojis in a future v0.11 migration — requires Notion UI (API can't rename status options).
- **D-PROC-2:** **`Synthesis State` (4 options) overlaps with `Digestion Stage` (9 options).** `raw_note` ≈ stage 1-2, `annotated` ≈ stage 3-4, `synthesized` ≈ stage 5-7, `applied` ≈ stage 8-9. YAGNI candidate — but `Synthesis State` may be a user-facing simplification. Verify usage before removing.
- **D-PROC-3:** `Sourced From`, `Rewrites (Potentiator)`, `Sends Experience To (Potentiator)` — all 3 relations' `database_id` → ghost `0baacff9-*` (data_source_id correct). Cosmetic.
- **D-PROC-4:** 35 properties is the highest count of any DB. **Review for YAGNI candidates** — `Counter-Synthesis`, `Counterpart`, `Reinforces`, `Counter-Tension` are all intra-DB relations whose usage should be checked. If any have <5% fill rate after 30 days, delete them (per §6.2 of AGENTS.md).

### 2.4 Identity (was Significator) — 92 entries, 29 properties

**Holonic role:** Persistent identity-pattern (greater cycle). Ingests Transformation, emits Choice. Lives in BOTH cycles.
**Entry-types (6):** Purpose, Value, Principle, Identity-Statement, Pillar, Strategic-Ideal
**Status:** Draft → Active → Evolving → Archived
**Currency flow:** Transformation in → Choice out

**Property inventory (key items):**
| Property | Type | Notes |
|----------|------|-------|
| Entry Type | multi_select (6) | — |
| Holon Type | select (5) | ✅ universal (Identity only) — Donor/Acceptor/Sharer/Multivalent/Noble |
| Valence Signature | rich_text | YAML — used by `derive_type` tool |
| Stage | select (6) | **Post-recovery, Trading era, HoloOS era, Active, Evolving, Archived** — see D-ID-1 |
| Complex | select (5) | ⚠ **DRIFT:** has `Soul` option (canonical is 4) |
| Review Cadence | select (4) | Annual, Bi-Annual, Quarterly, Event-Driven |
| 11 relation props | relation | see §3 |

**Drift items:**
- **D-ID-1:** **`Stage` select has biographical options** (`Post-recovery`, `Trading era`, `HoloOS era`) mixed with status-like options (`Active`, `Evolving`, `Archived`). This conflates two concepts: (a) the holon's life-era and (b) its current status. The `Status` property already covers (b). Recommend: rename `Stage` to `Life-Era` (or similar) and remove `Active/Evolving/Archived` from its options.
- **D-ID-2:** **`Complex` has `Soul` option (5 options, canonical is 4).** 1 entry uses it ("Identity (Vyre)"). Manual cleanup required: re-tag that entry to `Spirit` (or `None`), then remove `Soul` from the select via Notion UI.
- **D-ID-3:** `Anchored In`, `Generated From`, `Rewrites` — 3 relations to State. Verify these are all distinct concepts or if any are redundant. `Anchored In` (State↔Identity dual) is the canonical fractal-coupling relation. `Generated From` and `Rewrites` may be legacy.

### 2.5 World (was GreatWay) — 608 entries, 27 properties

**Holonic role:** Operating environment (greater cycle). Ingests Choice, produces Transformation.
**Entry-types (18):** Annual Goal, Quarterly Goal, Project, Task, System, Resource, Sprint, Milestone, Budget, Campaign, Content, Person, Group, Community, Organization, Network, Movement, Place
**Status:** Future → Ideation → Paused → Active → Done → Cancelled
**Currency flow:** Choice in → Transformation out

**Property inventory (key items):**
| Property | Type | Notes |
|----------|------|-------|
| Item Type | select (18) | Entry-type discriminator — **largest entry-type set** |
| Priority | select (5) | Critical, High, Medium, Low, None |
| Quadrant | select (4) | UL, UR, LL, LR — Wilber quadrants |
| Tier | select (3) | Strategic, Operational, Tactical |
| Progress | number | — |
| Target | number | — |
| Start Date / End Date | date | — |
| Monitor | formula | — |
| 9 relation props | relation | see §3 |

**Drift items:**
- **D-WORLD-1:** `Manifests As`, `Sub-holon Of`, `Related to Potentiator (People)` — 3 relations' `database_id` → ghost `0baacff9-*` (data_source_id correct). Cosmetic.
- **D-WORLD-2:** **18 entry-types is a lot.** Some may be underused. Run `lifeos suggest-categorization --database greatway` to check fill rates. Candidates for consolidation: `Annual Goal` + `Quarterly Goal` + `Goal` → could collapse to `Goal` with a separate `Timeframe` property. But this is a UX decision — defer to user.

---

## 3. Relation Topology

### 3.1 Inter-DB relation edges (13 dual_property, encoded in v0.9.0)

All 13 fractal-coupling relations from HoloOS doc 08.5 are present as dual_property:

| # | Source DB | Property | Target DB | Reciprocal | Status |
|---|-----------|----------|-----------|------------|--------|
| 1 | State | Related to Process (Rewrites (Matrix)) | Process | Rewrites (Matrix) | ✅ |
| 2 | State | Related to Process (Sends Catalyst To (Matrix)) | Process | Sends Catalyst To (Matrix) | ✅ |
| 3 | State | Related to Identity (Sub-holon Of) | Identity | Sub-holon Of | ✅ |
| 4 | Possibility | Related to World (Sub-holon Of) | World | Sub-holon Of | ✅ |
| 5 | Possibility | Related to Process (Rewrites (Potentiator)) | Process | Rewrites (Potentiator) | ✅ |
| 6 | Possibility | Related to Process (Sends Experience To (Potentiator)) | Process | Sends Experience To (Potentiator) | ✅ |
| 7 | Identity | Related to World (Coheres With (Significator)) | World | Coheres With (Significator) | ✅ |
| 8 | Identity | Related to World (For Significator) | World | For Significator | ✅ |
| 9 | Identity | Related to Process (Fires Transformation On) | Process | Fires Transformation On | ✅ |
| 10 | Identity | Related to Process (Sends Catalyst To (Significator)) | Process | Sends Catalyst To (Significator) | ✅ |
| 11 | Identity | Related to Process (Triggered By) | Process | Triggered By | ✅ |
| 12 | Identity | Emits Choice To | Process | Related to Identity (Emits Choice To) | ✅ |
| 13 | World | Related to Process (Emits Choice To) | Process | Emits Choice To | ✅ |

### 3.2 Ghost-database residue (cosmetic, not functional)

5 relations still carry `database_id` field pointing to the deleted pre-v0.9.0 Potentiator (`0baacff9-75c7-43b0-9449-efd5011c0afc`). The `data_source_id` field (used by Notion API 2025-09-03 for actual querying) is correct → real Potentiator (`a1769af1-*`).

| DB | Property | database_id (legacy) | data_source_id (used) |
|----|----------|---------------------|----------------------|
| State | Generated From | `0baacff9-*` (ghost) | `a1769af1-*` (real) ✅ |
| Possibility | Crystallizes Into | `0baacff9-*` (ghost) | `a1769af1-*` (real) ✅ |
| Process | Sourced From | `0baacff9-*` (ghost) | `a1769af1-*` (real) ✅ |
| Process | Rewrites (Potentiator) | `0baacff9-*` (ghost) | `a1769af1-*` (real) ✅ |
| Process | Sends Experience To (Potentiator) | `0baacff9-*` (ghost) | `a1769af1-*` (real) ✅ |
| World | Manifests As | `0baacff9-*` (ghost) | `a1769af1-*` (real) ✅ |
| World | Sub-holon Of | `0baacff9-*` (ghost) | `a1769af1-*` (real) ✅ |
| World | Related to Potentiator (People) | `0baacff9-*` (ghost) | `a1769af1-*` (real) ✅ |

**Functional impact:** None — `lifeos link` works (verified end-to-end in Phase 5).
**Cosmetic impact:** Notion UI may display the relation target as "deleted database" in some views.
**Fix:** Low priority — would require deleting + recreating each relation property via Notion UI (API doesn't support changing `database_id` of an existing relation). Defer.

### 3.3 Intra-DB relations (hierarchical)

Each DB has self-referential relations for parent/child hierarchies:
- State: `Parent`, `Blocked By`, `Refines`, `Supersedes` (4)
- Possibility: (none — flat structure)
- Process: `Counter-Synthesis`, `Counterpart`, `Reinforces` (3)
- Identity: `Coheres With`, `In Tension With`, `Parent item` ↔ `Sub-item` (3 dual + 2 single)
- World: `Blocks`, `Parent item` ↔ `Sub-item` (1 single + 1 dual)

**YAGNI candidate:** Verify fill rates. Intra-DB relations with <5% fill after 30 days should be deleted per §6.2 of AGENTS.md.

---

## 4. Universal Properties Compliance

Per [schemas/universal/holon_coordinate.yaml](schemas/universal/holon_coordinate.yaml), the 6 universal properties should be on all 5 DBs:

| Property | State | Possibility | Process | Identity | World |
|----------|-------|-------------|---------|----------|-------|
| Archetype Role (8 options) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Complex (4 options) | ✅ 4 | ✅ 4 | ✅ 4 | ⚠ **5** (has `Soul`) | ✅ 4 |
| Drive Activation (4 options) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Shadow Pattern (6 options) | ✅ 6 | ✅ 6 | ✅ 6 | ✅ 6 | ✅ 6 |
| Digestion Stage (9 options) | ✅ absent (correct) | ✅ present | ✅ present | ✅ absent (correct) | ✅ absent (correct) |
| Holon Type (5 options) | ✅ absent (correct) | ✅ absent (correct) | ✅ absent (correct) | ✅ present | ✅ absent (correct) |

**Compliance:** 29/30 cells correct. 1 drift: Identity.Complex has `Soul` (D-ID-2).

---

## 5. Validation Rules Compliance

Per the 3 hardcoded validation rules in [schemas/universal/holon_coordinate.yaml](schemas/universal/holon_coordinate.yaml):

| Rule | Status | Notes |
|------|--------|-------|
| `nexus_kind_consistency` | ✅ Enforced | Process.Kind constrains which relations can populate. `mutate` tool validates. |
| `stage_type_independence` | ✅ Enforced | Digestion Stage and Holon Type must both be set or both empty. `validate_yaml` checks. |
| `complex_archetype_consistency` | ⚠ Partially enforced | (role, complex) must be one of 22 named archetypes. **Blocked by D-ID-2**: `Soul` complex is not in the 22-archetype map → any Identity entry with `Complex=Soul` fails validation. |

**Missing rule (proposed):** `shadow_pattern_db_consistency` — Sinkhole of Indifference should only be valid on World (Great Way Choice-starvation); Dark-* only on State (Matrix); Golden-* only on Possibility (Potentiator). Currently any shadow can be set on any DB. See Action Tracker item U-7.

---

## 6. Entry Count Reality Check

| DB | Real entry count | Dashboard sample (capped at 100) | Orphan % (sampled) | Implication |
|----|------------------|--------------------------------|---------------------|-------------|
| State | 39 | 39 | 17% | Low orphan rate — well-curated |
| Possibility | 6,900 | 100 | 100% | **Daily logs dominate; most have no relations** (expected) |
| Process | 797 | 100 | 100% | Notes/Insights accumulate without being linked |
| Identity | 92 | 92 | 100% | **All 92 orphaned** — no Identity entry has a relation to World or State |
| World | 608 | 100 | 74% | Most People/Projects lack relations to Identity/Possibility |
| **TOTAL** | **8,436** | 431 | 86% (sampled) | — |

**Key insight:** The 86% orphan rate is dominated by Possibility (6,900 daily-log entries). Daily logs (Activity/Diet/Financial/Subjective/Relational/Systemic) are *inherently* low-relation — they're raw catalyst capture. The orphan rate for *ontological* entries (Identity, Process notes that should be linked, World projects that should connect to Identity) is the real signal:
- Identity: 92/92 orphaned = **100%** ← critical
- World: 74% orphaned = **high** ← critical
- Process: 100% sampled orphaned = **critical** (Notes should link to Patterns/Insights)

**The holonic spiral is structurally sound but operationally dormant.** The fix is NOT bulk-tagging (user prefers manual curation) — it's making manual curation easier via better tooling. See Action Tracker.

---

## 7. Summary of Drift Items (14 total)

| ID | DB | Severity | Description | Fix effort |
|----|-----|----------|-------------|------------|
| D-STATE-1 | State | low | Generated From database_id → ghost | cosmetic, defer |
| D-POSS-1 | Possibility | low | Crystallizes Into database_id → ghost | cosmetic, defer |
| D-POSS-2 | Possibility | medium | Digestion Status redundant with Digestion Stage | verify usage, then YAGNI |
| D-POSS-3 | Possibility | low | Documents relation → external DB | document or archive |
| D-PROC-1 | Process | low | Status emojis make filtering harder | Notion UI rename (v0.11) |
| D-PROC-2 | Process | medium | Synthesis State overlaps Digestion Stage | verify usage, then YAGNI |
| D-PROC-3 | Process | low | 3 relations database_id → ghost | cosmetic, defer |
| D-PROC-4 | Process | medium | 35 properties — review for YAGNI | run fill-rate audit |
| D-ID-1 | Identity | medium | Stage conflates life-era + status | rename + clean options |
| D-ID-2 | Identity | high | Complex has `Soul` (5 options, canon is 4) | re-tag 1 entry, remove option |
| D-ID-3 | Identity | low | 3 State-relations may be redundant | verify usage |
| D-WORLD-1 | World | low | 3 relations database_id → ghost | cosmetic, defer |
| D-WORLD-2 | World | low | 18 entry-types — some may be underused | suggest-categorization audit |
| D-UNIVERSAL | All | medium | No `shadow_pattern_db_consistency` validation rule | add rule (Action U-7) |

See [AUDIT_v0.10.1_ACTION_TRACKER.md](AUDIT_v0.10.1_ACTION_TRACKER.md) for the prioritized fix list.
