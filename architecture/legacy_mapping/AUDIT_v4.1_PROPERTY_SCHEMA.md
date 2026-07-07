# LifeOS v4.1 — Post-Migration Property Schema Audit + Refactor Plan
# =============================================================================
# **AUDIT DATE:** 2026-07-07
# **AUDIT SOURCE:** Live Notion API query of all 5 refactored DBs
#                  (architecture/legacy_mapping/post_migration_audit.json)
#
# **FINDING:** The migration successfully moved entries + renamed DBs + added
# new properties. BUT the old properties (from the legacy 5-DB structure) were
# NOT cleaned up. Every DB still carries dead weight: universal properties
# (Archetype Role, Complex, Drive Activation, Shadow Pattern, Digestion Stage)
# that don't belong in the v4.1 structure, old holonic relations that point to
# the wrong DBs or to GHOST, and legacy properties that no longer serve a
# purpose.
#
# This document is the per-DB audit + the refactor plan to make everything
# optimal.

---

## 1. Executive Summary

| DB | Current props | Target props (v4.1 spec) | Props to DELETE | Props to ADD | Props to RE-POINT | Status |
|----|--------------|--------------------------|-----------------|--------------|-------------------|--------|
| **Trajectory** | 27 | ~20 | 7 | 3 | 4 | 🔴 Needs cleanup |
| **Logbook** | 27 | ~12 | 15 | 3 | 2 | 🔴 Needs heavy cleanup |
| **Synthesis** | 36 | ~15 | 21 | 2 | 4 | 🔴 Needs heavy cleanup |
| **Profile** | 36 | ~12 | 24 | 1 | 0 | 🔴 Needs heavy cleanup |
| **Context** | 23 | 23 | 0 | 0 | 0 | ✅ OPTIMAL |
| **TOTAL** | 149 | ~82 | **67** | **9** | **10** | — |

**67 properties to delete** (45% of current properties are dead weight).
**9 properties to add** (the v4.1 relations that connect the 5 DBs).
**10 relations to re-point** (old holonic relations pointing to wrong DBs or GHOST).

---

## 2. Per-DB Audit + Refactor Plan

### 2.1 Trajectory (27 props → target ~20)

**Current properties (27):**

| Property | Type | Status | Action |
|----------|------|--------|--------|
| Name | title | ✅ KEEP | — |
| ID | unique_id | ✅ KEEP | — |
| Item Type | select (16 opts) | ✅ KEEP | Rename to `Type` via Notion UI (API can't rename properties) |
| Status | status (6 opts) | ✅ KEEP | — |
| Priority | select (5 opts) | ✅ KEEP | — |
| Progress | number | ✅ KEEP | — |
| Target | number | ✅ KEEP | — |
| Start Date | date | ✅ KEEP | — |
| End Date | date | ✅ KEEP | — |
| Quadrant | select (4 opts) | ✅ KEEP | Useful for Wilber quadrant tagging |
| Tier | select (3 opts) | ✅ KEEP | Useful for execution layer |
| Parent item | relation → self (dual) | ✅ KEEP | The hierarchy. Rename to `Parent` via UI. |
| Sub-item | relation → self (dual) | ✅ KEEP | The child side of hierarchy. |
| Blocks | relation → self | ✅ KEEP | Dependency. Rename to `Blocked By` via UI. |
| Serves Value | relation → self (dual) | ✅ KEEP | Added in Phase 6. The constraining relation. |
| Monitor | formula | ⚠ KEEP IF USED | Check fill rate; delete if <5% |
| **Archetype Role** | select (8 opts) | ❌ DELETE | Universal property — not in v4.1 spec. Type select discriminates instead. |
| **Complex** | select (4 opts) | ❌ DELETE | Universal property — not in v4.1 spec. |
| **Drive Activation** | multi_select (4 opts) | ❌ DELETE | Universal property — not in v4.1 spec. |
| **Shadow Pattern** | select (6 opts) | ❌ DELETE | Universal property — not in v4.1 spec. Belongs on Profile (as Trait). |
| **Manifests As** | relation → GHOST | ❌ DELETE | Ghost relation (0baacff9) |
| **Sub-holon Of** | relation → GHOST | ❌ DELETE | Ghost relation (0baacff9) |
| **Related to Potentiator (People)** | relation → GHOST | ❌ DELETE | Ghost relation (0baacff9) |
| **For** | relation → self (dual) | ⚠ RE-POINT | Old holonic relation. Re-point to Trajectory self OR delete if unused. |
| **For Significator** | relation → self (dual) | ⚠ RE-POINT | Same — old holonic. Delete if unused. |
| **Coheres With (Significator)** | relation → self (dual) | ⚠ RE-POINT | Same — old holonic. Delete if unused. |
| **Related to Nexus (Emits Choice To)** | relation → Synthesis | ⚠ RE-POINT | Re-point to Synthesis (already correct target). Rename to `Spawned By`. |
| **Sourced From** | relation → Synthesis | ⚠ RE-POINT | Re-point to Synthesis. Rename to `Synthesized From`. |

**Props to ADD:**
- `Description` (rich_text) — NEW
- `Source` (rich_text) — NEW (for Reference entries)
- `Timeframe` (select: Lifetime/10yr/5yr/3yr/1yr) — NEW (for Vision-Statement)
- `Last Reviewed` (date) — NEW (for Reference entries)

**Props to DELETE: 7** (4 universal + 3 ghost)
**Props to ADD: 4**
**Props to RE-POINT/RENAME: 5** (For, For Significator, Coheres With, Related to Nexus, Sourced From)

**Target: ~24 properties** (27 - 7 + 4 = 24, minus some old holonic relations that get deleted = ~20)

---

### 2.2 Logbook (27 props → target ~12)

**Current properties (27):**

| Property | Type | Status | Action |
|----------|------|--------|--------|
| Name | title | ✅ KEEP | — |
| ID | unique_id | ✅ KEEP | — |
| Date | date | ✅ KEEP | Primary time index |
| Entry Type | select (6 opts) | ✅ KEEP | The discriminator |
| Distillation | rich_text | ✅ KEEP | Rename to `Content` via UI |
| Amount | formula | ⚠ KEEP IF USED | Check if formula produces useful data |
| Duration | formula | ⚠ KEEP IF USED | Same |
| Month Label | formula | ⚠ KEEP IF USED | Useful for grouping |
| Quarter Label | formula | ⚠ KEEP IF USED | Same |
| Week Label | formula | ⚠ KEEP IF USED | Same |
| For | relation → Trajectory | ⚠ RE-POINT | Already points to World (now Trajectory). Rename to `Source Project`. |
| People | relation → Trajectory | ❌ RE-POINT | Points to World (now Trajectory) — should point to Context. Rename to `Subject Person`. |
| **Archetype Role** | select (8 opts) | ❌ DELETE | Universal — not needed for logs |
| **Complex** | select (4 opts) | ❌ DELETE | Universal — not needed |
| **Drive Activation** | multi_select (4 opts) | ❌ DELETE | Universal — not needed |
| **Shadow Pattern** | select (6 opts) | ❌ DELETE | Universal — not needed |
| **Digestion Stage** | select (9 opts) | ❌ DELETE | Not needed for logs |
| **Digestion Status** | status (3 opts) | ❌ DELETE | Redundant with Entry Type |
| **YAML Metadata** | rich_text | ❌ DELETE | Legacy |
| **Crystallized To** | relation → State (now Profile) | ❌ DELETE | Old holonic relation |
| **Crystallizes Into** | relation → GHOST | ❌ DELETE | Ghost relation |
| **Documents** | relation → external DB | ❌ DELETE | External DB not part of LifeOS |
| **Harmonized By** | relation → Process (now Synthesis) | ❌ DELETE | Old holonic relation |
| **Reveals** | relation → State (now Profile) | ❌ DELETE | Old holonic relation |
| **Related to GreatWay (Sub-holon Of)** | relation → Trajectory | ❌ DELETE | Old holonic relation |
| **Related to Nexus (Rewrites (Potentiator))** | relation → Synthesis | ❌ DELETE | Old holonic relation |
| **Related to Nexus (Sends Experience To (Potentiator))** | relation → Synthesis | ❌ DELETE | Old holonic relation |

**Props to ADD:**
- `Channel` (select: Body/Mind/Resource/Relational) — NEW
- `Sentiment` (select: Positive/Neutral/Negative) — NEW
- `Synthesized Into` (relation → Synthesis, multi) — NEW

**Props to DELETE: 15**
**Props to ADD: 3**
**Props to RE-POINT: 2** (For → Source Project; People → Subject Person → Context)

**Target: ~15 properties** (27 - 15 + 3 = 15)

---

### 2.3 Synthesis (36 props → target ~15)

**Current properties (36):**

| Property | Type | Status | Action |
|----------|------|--------|--------|
| Name | title | ✅ KEEP | — |
| ID | unique_id | ✅ KEEP | — |
| Date | date | ✅ KEEP | — |
| Category | select (5 opts) | ✅ KEEP | The discriminator (Note/Opportunity/Strength/Directive/Risk) |
| Polarity | select (3 opts) | ✅ KEEP | Added in Phase 9 |
| Priority | select (4 opts) | ✅ KEEP | — |
| Status | status (4 emoji opts) | ✅ KEEP | — |
| Capture Method | select (5 opts) | ✅ KEEP | — |
| Source URL | url | ✅ KEEP | — |
| Raw Content | rich_text | ✅ KEEP | — |
| Synthesis State | select (4 opts) | ✅ KEEP | — |
| **Archetype Role** | select (8 opts) | ❌ DELETE | Universal |
| **Complex** | select (4 opts) | ❌ DELETE | Universal |
| **Drive Activation** | multi_select (4 opts) | ❌ DELETE | Universal |
| **Shadow Pattern** | select (6 opts) | ❌ DELETE | Universal |
| **Digestion Stage** | select (9 opts) | ❌ DELETE | Not needed |
| **Kind** | select (4 opts) | ❌ DELETE | Replaced by Category + Polarity |
| **Highlight Count** | number | ❌ DELETE | Unused |
| **Last Assessed** | date | ❌ DELETE | Unused |
| **Polarity Outcome** | select (3 opts) | ❌ DELETE | Unused |
| **Counter-Synthesis** | relation → self | ❌ DELETE | Old dialectical relation |
| **Counterpart** | relation → self | ❌ DELETE | Old dialectical relation |
| **Reinforces** | relation → self | ❌ DELETE | Old dialectical relation |
| **Counter-Tension** | relation → Profile | ❌ DELETE | Old holonic relation |
| **Tension** | relation → Profile | ❌ DELETE | Old holonic relation |
| **Updates** | relation → State (now Profile) | ❌ DELETE | Old holonic relation |
| **Rewrites (Matrix)** | relation → State (now Profile) | ❌ DELETE | Old holonic relation |
| **Rewrites (Potentiator)** | relation → GHOST | ❌ DELETE | Ghost relation |
| **Sends Catalyst To (Matrix)** | relation → State (now Profile) | ❌ DELETE | Old holonic relation |
| **Sends Catalyst To (Significator)** | relation → Profile | ❌ DELETE | Old holonic relation |
| **Sends Experience To (Potentiator)** | relation → GHOST | ❌ DELETE | Ghost relation |
| **Sourced From** | relation → GHOST | ❌ DELETE | Ghost relation |
| **Emits Choice To** | relation → Profile | ⚠ RE-POINT | Re-point to Trajectory. Rename to `Revises`. |
| **Fires Transformation On** | relation → Profile | ⚠ RE-POINT | Re-point to Trajectory. Rename to `Spawns`. |
| **Triggered By** | relation → Profile | ❌ DELETE | Old holonic relation |
| **Related to Significator (Emits Choice To)** | relation → Profile | ❌ DELETE | Duplicate of Emits Choice To |

**Props to ADD:**
- `Source Logs` (relation → Logbook, multi) — NEW (replaces Sourced From)
- `Condenses Into` (relation → Profile, multi) — NEW

**Props to DELETE: 21**
**Props to ADD: 2**
**Props to RE-POINT: 2** (Emits Choice To → Trajectory; Fires Transformation On → Trajectory)

**Target: ~15 properties** (36 - 21 + 2 - 2 re-pointed = 15)

---

### 2.4 Profile (36 props → target ~12)

**Current properties (36):**

| Property | Type | Status | Action |
|----------|------|--------|--------|
| Name | title | ✅ KEEP | — |
| ID | unique_id | ✅ KEEP | — |
| Status | status (4 opts) | ✅ KEEP | — |
| Entry Type | multi_select (10 opts) | ✅ KEEP | Reduce to 4: Trait/Metric/Capacity/Asset (remove Purpose/Value/Principle/Identity-Statement/Pillar/Strategic-Ideal — those moved to Trajectory) |
| Category | select (8 opts) | ✅ KEEP | Added in Phase 7 |
| Current Value | rich_text | ✅ KEEP | Added in Phase 7 |
| Target Value | rich_text | ✅ KEEP | Added in Phase 7 |
| Trend | select (3 opts) | ✅ KEEP | Added in Phase 7 |
| Unit | select (8 opts) | ✅ KEEP | Added in Phase 7 |
| Frequency | select (5 opts) | ✅ KEEP | Added in Phase 7 |
| Last Reviewed | date | ✅ KEEP | Rename to `Last Updated` via UI |
| **Archetype Role** | select (8 opts) | ❌ DELETE | Universal |
| **Complex** | select (4 opts) | ❌ DELETE | Universal |
| **Drive Activation** | multi_select (4 opts) | ❌ DELETE | Universal |
| **Shadow Pattern** | select (6 opts) | ❌ DELETE | Universal |
| **Holon Type** | select (5 opts) | ❌ DELETE | Becomes a Trait entry, not a property |
| **Valence Signature** | rich_text | ❌ DELETE | Becomes a Trait entry, not a property |
| **Life-Era** | select (3 opts) | ❌ DELETE | Becomes a Trait entry, not a property |
| **Stage** | select (3 opts) | ❌ DELETE | Legacy empty (duplicate of Life-Era) |
| **Review Cadence** | select (4 opts) | ❌ DELETE | Not needed for Profile |
| **Next Review** | date | ❌ DELETE | Not needed |
| **Anchored In** | relation → State (now Profile) | ❌ DELETE | Old holonic relation |
| **Coheres With** | relation → self | ❌ DELETE | Old holonic relation |
| **Emits Choice To** | relation → Synthesis | ❌ DELETE | Old holonic relation |
| **Generated From** | relation → State (now Profile) | ❌ DELETE | Old holonic relation |
| **In Tension With** | relation → self | ❌ DELETE | Old holonic relation |
| **Parent item** | relation → self (dual) | ❌ DELETE | Not needed for Profile |
| **Related to GreatWay (Coheres With (Significator))** | relation → self | ❌ DELETE | Old holonic relation |
| **Related to GreatWay (For Significator)** | relation → self | ❌ DELETE | Old holonic relation |
| **Related to Nexus (Fires Transformation On)** | relation → Synthesis | ❌ DELETE | Old holonic relation |
| **Related to Nexus (Sends Catalyst To (Significator))** | relation → Synthesis | ❌ DELETE | Old holonic relation |
| **Related to Nexus (Triggered By)** | relation → Synthesis | ❌ DELETE | Old holonic relation |
| **Rewrites** | relation → State (now Profile) | ❌ DELETE | Old holonic relation |
| **Sub-holon Of** | relation → State (now Profile) | ❌ DELETE | Old holonic relation |
| **Sub-item** | relation → self (dual) | ❌ DELETE | Not needed for Profile |
| **Transforms To** | relation → self (dual) | ❌ DELETE | Old holonic relation |

**Props to ADD:**
- `Source Synthesis` (relation → Synthesis, multi) — NEW
- `Closes Gap For` (relation → Trajectory, multi) — NEW
- `Informs Goal` (relation → Trajectory, multi) — NEW

**Props to DELETE: 24**
**Props to ADD: 3**

**Target: ~15 properties** (36 - 24 + 3 = 15)

---

### 2.5 Context (23 props → target 23)

**Current properties (23):** All match the v4.1 spec. ✅

**No changes needed.** Context is already optimal.

**Props to ADD (relations OUT):**
- `Involved In` (relation → Trajectory, multi) — NEW
- `Subject Of` (relation → Logbook, multi) — NEW
- `Referenced In` (relation → Synthesis, multi) — NEW

**Target: 26 properties** (23 + 3 relations)

---

## 3. The Inter-DB Relation Refactor (10 relations to create/re-point)

These are the v4.1 spec relations that connect the 5 DBs. Most don't exist yet — the old holonic relations need to be deleted first, then these created fresh.

| # | From DB | Property | To DB | Cardinality | Action |
|---|---------|----------|-------|-------------|--------|
| 1 | Trajectory | `Generates Logs` | Logbook | one-to-many | CREATE NEW |
| 2 | Trajectory | `Assigned To` / `Involves` | Context | many-to-many | CREATE NEW |
| 3 | Trajectory | `Spawned By` | Synthesis | many-to-one | RE-POINT (was Related to Nexus) |
| 4 | Logbook | `Source Project` | Trajectory | many-to-one | RE-POINT (was For) |
| 5 | Logbook | `Subject Person` | Context | many-to-one | RE-POINT (was People) |
| 6 | Logbook | `Synthesized Into` | Synthesis | many-to-many | CREATE NEW |
| 7 | Synthesis | `Source Logs` | Logbook | many-to-many | CREATE NEW |
| 8 | Synthesis | `Spawns` | Trajectory | one-to-many | RE-POINT (was Fires Transformation On) |
| 9 | Synthesis | `Condenses Into` | Profile | many-to-many | CREATE NEW |
| 10 | Synthesis | `Revises` | Trajectory | many-to-many | RE-POINT (was Emits Choice To) |
| 11 | Profile | `Source Synthesis` | Synthesis | many-to-many | CREATE NEW |
| 12 | Profile | `Closes Gap For` | Trajectory | many-to-many | CREATE NEW |
| 13 | Profile | `Informs Goal` | Trajectory | many-to-many | CREATE NEW |
| 14 | Context | `Involved In` | Trajectory | many-to-many | CREATE NEW |
| 15 | Context | `Subject Of` | Logbook | many-to-many | CREATE NEW |
| 16 | Context | `Referenced In` | Synthesis | many-to-many | CREATE NEW |

---

## 4. Execution Plan (4 phases)

### Phase A: DELETE dead properties (67 properties across 4 DBs)

| DB | Props to delete | Method |
|----|----------------|--------|
| Trajectory | 7 (4 universal + 3 ghost) | PATCH data_source with `{"properties": {"Archetype Role": null, ...}}` |
| Logbook | 15 (4 universal + Digestion Stage/Status + YAML Metadata + 8 old relations + Crystallizes Into ghost) | Same |
| Synthesis | 21 (4 universal + Digestion Stage + Kind + 3 unused + 14 old relations) | Same |
| Profile | 24 (4 universal + Holon Type + Valence + Life-Era + Stage + Review Cadence + Next Review + 14 old relations) | Same |

**Note:** Notion API doesn't support property deletion via `null`. Properties must be deleted via Notion UI. I'll write a script that lists exactly which properties to delete per DB, and the user deletes them via UI (or I can try setting them to an empty config).

### Phase B: ADD new properties + relations (9 props + 16 relations)

| DB | Props to add | Method |
|----|-------------|--------|
| Trajectory | Description, Source, Timeframe, Last Reviewed | PATCH data_source |
| Logbook | Channel, Sentiment, Synthesized Into (→ Synthesis) | PATCH data_source |
| Synthesis | Source Logs (→ Logbook), Condenses Into (→ Profile) | PATCH data_source |
| Profile | Source Synthesis (→ Synthesis), Closes Gap For (→ Trajectory), Informs Goal (→ Trajectory) | PATCH data_source |
| Context | Involved In (→ Trajectory), Subject Of (→ Logbook), Referenced In (→ Synthesis) | PATCH data_source |
| Trajectory | Generates Logs (→ Logbook), Assigned To (→ Context), Involves (→ Context) | PATCH data_source |
| Synthesis | Spawns (→ Trajectory), Revises (→ Trajectory) | PATCH data_source |

### Phase C: RENAME properties (via Notion UI)

| DB | Current name | → New name |
|----|-------------|------------|
| Trajectory | Item Type | Type |
| Trajectory | Parent item | Parent |
| Trajectory | Related to Nexus (Emits Choice To) | Spawned By |
| Trajectory | Sourced From | Synthesized From |
| Logbook | Distillation | Content |
| Logbook | For | Source Project |
| Logbook | People | Subject Person |
| Profile | Last Reviewed | Last Updated |
| Profile | Entry Type | Type (reduce to 4 opts: Trait/Metric/Capacity/Asset) |

**Note:** Notion API can't rename properties in-place. These must be done via Notion UI.

### Phase D: VERIFY

- [ ] Trajectory: ~20 properties, 16 entry-types, Parent + Serves Value self-relations, Generates Logs → Logbook, Involves → Context
- [ ] Logbook: ~15 properties, 6 entry-types, Source Project → Trajectory, Subject Person → Context, Synthesized Into → Synthesis
- [ ] Synthesis: ~15 properties, 5 entry-types + Polarity, Source Logs → Logbook, Spawns → Trajectory, Condenses Into → Profile, Revises → Trajectory
- [ ] Profile: ~15 properties, 4 entry-types, Source Synthesis → Synthesis, Closes Gap For → Trajectory, Informs Goal → Trajectory
- [ ] Context: ~26 properties, 5 entry-types, Involved In → Trajectory, Subject Of → Logbook, Referenced In → Synthesis
- [ ] No ghost relations (0baacff9) remain
- [ ] No universal properties (Archetype Role, Complex, Drive Activation, Shadow Pattern, Digestion Stage) remain on any DB
- [ ] Total properties: ~82 (down from 149 — 45% reduction)

---

## 5. Summary

The migration successfully moved entries + renamed DBs, but left 67 dead properties (45% of the total) from the old holonic structure. These are:

1. **Universal properties** (20 total): Archetype Role, Complex, Drive Activation, Shadow Pattern — on all 4 non-Context DBs. Not in v4.1 spec.
2. **Digestion Stage + Digestion Status** (3 total): On Logbook + Synthesis. Not needed.
3. **Old holonic relations** (35 total): Sub-holon Of, Rewrites, Sends Catalyst/Experience To, Emits Choice To, Fires Transformation On, Triggered By, etc. — all pointing to wrong DBs or GHOST.
4. **Legacy properties** (9 total): Kind, Highlight Count, Last Assessed, Polarity Outcome, Holon Type, Valence Signature, Life-Era, Stage, Review Cadence, Next Review, YAML Metadata.

The cleanup is straightforward: delete 67 properties, add 9 new ones + 16 new relations, rename 9 properties. The result: ~82 properties total (down from 149), with every property serving a clear purpose in the v4.1 consciousness-prosthetic architecture.

---

*Post-migration property audit + refactor plan. Ready for execution.*
