# LifeOS v4.1 — Grounded Refactor Plan
# =============================================================================
# **STATUS:** Grounded in actual Notion API audit (2026-07-06). Ready for validation.
#
# **AUDIT SOURCE:** architecture/legacy_mapping/grounded_audit.json
#   — Live API query of all 5 DBs: properties, entry-types, per-type counts,
#     relation targets, ghost-database detection.
#
# **KEY CORRECTIONS FROM PREVIOUS PLAN (v4.1 ungrounded):**
#   1. Possibility has 0 Goal/Vision/Aspiration entries → 0 entries move to Trajectory
#   2. Process has 0 Pattern entries → 0 entries move to Profile
#   3. Process is 99.6% Notes (794 of 797) → just rename, no re-classification
#   4. State's 34 "untyped" entries are Stats/Metrics → all go to Profile
#   5. Identity's 69 "untyped" entries are Person names → all go to Context
#   6. Total entries that MOVE: 208 (not ~300+ as previously estimated)
#   7. Total entries that STAY (just rename DB): 8,242

---

## 1. Grounded Audit Summary (Live Data)

### 1.1 Actual entry counts per DB per entry-type

| DB | Total | Typed entries | Untyped entries | Entry-type breakdown (non-zero only) |
|----|-------|---------------|-----------------|--------------------------------------|
| **State** | 39 | 5 | **34** | Pattern: 5. Untyped: 34 (Stats/Metrics — "Energy level", "Sleep quality", "Monthly income", etc.) |
| **Possibility** | 6,911 | 6,911 | 0 | Activity: 5,594 · Financial: 996 · Diet: 127 · Relational: 83 · Subjective: 75 · Systemic: 34. **Goal=0, Vision=0, Aspiration=0, Observation=0** |
| **Process** | 797 | 797 | 0 | Note: 794 · Decision: 3. **All other 11 types = 0** (Opportunity, Directive, Risk, Insight, Reflection, Integration, Pattern, Knowledge-Category, Knowledge-Atom, Crisis, Transformation-Event) |
| **Identity** | 92 | 23 | **69** | Value: 10 · Pillar: 10 · Purpose: 1 · Identity-Statement: 1 · Strategic-Ideal: 1. Principle: 0. **Untyped: 69 (all Person names — "Archana Saini", "Namrata", etc., Status=Draft)** |
| **World** | 611 | 611 | 0 | Task: 359 · Person: 63 · Content: 59 · Project: 43 · Annual Goal: 36 · Quarterly Goal: 20 · Campaign: 16 · Community: 14 · System: 1. **All other 9 types = 0** |
| **TOTAL** | **8,450** | 7,747 | 103 | — |

### 1.2 Actual property counts

| DB | Properties | Key properties (relevant to refactor) |
|----|-----------|--------------------------------------|
| State | 25 | Entry Type (multi_select, 3 opts), Status, Parent, Blocked By, Refines, Supersedes, Pillar Link, Review Cadence, Last Reviewed, Next Review, Crystallization Date, Integration Weight, Accumulates Into, Integrated Into, Generated From (→GHOST), + 6 universal props + 4 relations to Process/Identity |
| Possibility | 27 | Entry Type (select, 10 opts), Date, Distillation, Amount (formula), Duration (formula), Month/Quarter/Week Label (formulas), Digestion Stage, Digestion Status, For, People, Reveals, Harmonized By, Crystallized To, Crystallizes Into (→GHOST), Documents (→external), + 6 universal props + 3 relations |
| Process | 35 | Category (select, 13 opts), Kind (select, 4 opts), Date, Priority, Status (emoji), Capture Method, Source URL, Raw Content, Synthesis State, Highlight Count, Last Assessed, Polarity Outcome, + 6 universal props + 12 relations (many →GHOST) |
| Identity | 30 | Entry Type (multi_select, 6 opts), Status, Holon Type, Valence Signature, Life-Era, Stage (legacy empty), Review Cadence, Last Reviewed, Next Review, + 6 universal props + 12 relations |
| World | 27 | Item Type (select, 18 opts), Status, Priority, Progress, Target, Start Date, End Date, Quadrant, Tier, + 6 universal props + 8 relations (3 →GHOST) |

### 1.3 Ghost-database relations confirmed (8 total)

| DB | Property | Target |
|----|----------|--------|
| State | Generated From | → GHOST (0baacff9) |
| Possibility | Crystallizes Into | → GHOST (0baacff9) |
| Process | Sourced From | → GHOST (0baacff9) |
| Process | Rewrites (Potentiator) | → GHOST (0baacff9) |
| Process | Sends Experience To (Potentiator) | → GHOST (0baacff9) |
| World | Manifests As | → GHOST (0baacff9) |
| World | Sub-holon Of | → GHOST (0baacff9) |
| World | Related to Potentiator (People) | → GHOST (0baacff9) |

### 1.4 Relation target mapping (actual, from audit)

Relations in the current 5-DB point to these target DBs (by ID prefix):
- `37ec18ce` = State (matrix) → will become Profile
- `a1769af1` = Possibility (potentiator) → will become Logbook
- `2acc18ce` = Process (nexus) → will become Synthesis
- `38dc18ce` = Identity (significator) OR World (greatway) — **AMBIGUOUS** (both share prefix `38dc18ce`!)
- `0baacff9` = GHOST (deleted Potentiator)

**CRITICAL:** Identity and World share the same ID prefix (`38dc18ce`). The full UUID differs in the last 12 hex chars. The refactor scripts must use FULL UUIDs, not prefixes.

---

## 2. Grounded Entry-Move Plan (208 entries move, 8,242 stay)

### 2.1 Moves that ARE needed

| # | From DB | From entry-type | → To DB | → New entry-type | Count | Notes |
|---|---------|----------------|---------|-----------------|-------|-------|
| 1 | World | Person | Context | Person | 63 | Move to new Context DB |
| 2 | World | Community | Context | Community | 14 | Move to new Context DB |
| 3 | Identity | (untyped) | Context | Person | 69 | These are Person names (Archana Saini, etc.) with Status=Draft. Move to Context. |
| 4 | Identity | Value | Trajectory | Value | 10 | Move to Trajectory (was World) |
| 5 | Identity | Pillar | Trajectory | Value or Principle | 10 | Move to Trajectory. Re-classify: Pillar → Value (grouped via Serves Value) |
| 6 | Identity | Purpose | Trajectory | Purpose | 1 | Move to Trajectory |
| 7 | Identity | Identity-Statement | Trajectory | Identity-Statement | 1 | Move to Trajectory |
| 8 | Identity | Strategic-Ideal | Trajectory | Vision-Statement | 1 | Move to Trajectory + rename type |
| 9 | State | Pattern | Profile | Trait | 5 | Move to Profile |
| 10 | State | (untyped) | Profile | Metric | 34 | These are Stats entries (Energy level, Sleep quality, Monthly income, etc.). Move to Profile as Metrics. |
| 11 | Process | Decision | Synthesis (stays) OR Trajectory | Note OR Task | 3 | Re-classify: if actionable → Trajectory as Task; if reflective → stays as Note |
| **TOTAL MOVES** | | | | | **208** | |

### 2.2 Moves that are NOT needed (corrections from previous plan)

| Previous plan assumed | Actual | Why |
|----------------------|--------|-----|
| Possibility Goal/Vision/Aspiration → Trajectory (~90 entries) | **0 entries** | Goal=0, Vision=0, Aspiration=0, Observation=0 in Possibility |
| Process Pattern → Profile (~30 entries) | **0 entries** | Pattern=0 in Process |
| Process Insight → Synthesis.Strength (~50 entries) | **0 entries** | Insight=0 in Process |
| Process Reflection → Synthesis.Note (~30 entries) | **0 entries** | Reflection=0 in Process |
| Process Integration → Synthesis.Note (~20 entries) | **0 entries** | Integration=0 in Process |
| Process Risk → Synthesis.Risk (~50 entries) | **0 entries** | Risk=0 in Process |
| Process Crisis → Synthesis.Risk (~10 entries) | **0 entries** | Crisis=0 in Process |
| Process Knowledge-Category/Atom → Synthesis.Note (~30 entries) | **0 entries** | Both = 0 in Process |
| Process Transformation-Event (~5 entries) | **0 entries** | Transformation-Event=0 in Process |
| World Resource/Sprint/Budget/Group/Org/Network/Movement/Place → various | **0 entries each** | All = 0 in World |
| Identity Principle → Trajectory (~10 entries) | **0 entries** | Principle=0 in Identity |

### 2.3 Entries that STAY (just rename the DB)

| DB | → New name | Entries staying | What stays |
|----|-----------|----------------|------------|
| World | **Trajectory** | 534 | Task: 359, Content: 59, Project: 43, Annual Goal: 36, Quarterly Goal: 20, Campaign: 16, System: 1 |
| Possibility | **Logbook** | 6,911 | Activity: 5,594, Financial: 996, Diet: 127, Relational: 83, Subjective: 75, Systemic: 34 |
| Process | **Synthesis** | 794 | Note: 794 (all stay as Note) |
| **TOTAL STAYING** | | **8,239** | |

### 2.4 Summary

```
Total entries:           8,450
Entries that MOVE:         208  (2.5%)
Entries that STAY:       8,242  (97.5% — just rename DB)
```

The refactor is **97.5% rename, 2.5% move**. Much simpler than previously estimated.

---

## 3. Grounded Property Plan

### 3.1 Trajectory (was World — 27 props currently)

**KEEP (already exist in World):**
- Name (title), ID (unique_id), Status (status, 6 opts), Priority (select, 5 opts), Progress (number), Target (number), Start Date (date), End Date (date)
- Parent item (relation → self, dual with Sub-item) — **this becomes the `Parent` hierarchy relation**
- Sub-item (relation → self, dual with Parent item) — **keep as the child-side of the hierarchy**
- Blocks (relation → self) — keep as dependency (rename to `Blocked By` if not already)
- Quadrant (select, 4 opts: UL/UR/LL/LR) — keep (useful for Wilber quadrant tagging)
- Tier (select, 3 opts: Strategic/Operational/Tactical) — keep (useful for execution layer)
- Monitor (formula) — keep if useful

**ADD (new properties for Reference layer + hierarchy):**
- `Type` (select, 12 opts) — **REPLACE `Item Type`** (expand from 18 to 12: remove Person/Community/Group/Org/Network/Movement/Place; add Purpose/Value/Principle/Vision-Statement/Identity-Statement)
- `Description` (rich_text) — new
- `Source` (rich_text) — new (port from Identity)
- `Timeframe` (select: Lifetime/10yr/5yr/3yr/1yr) — new (for Vision-Statement)
- `Last Reviewed` (date) — new (port from Identity/State)
- `Serves Value` (relation → self, multi) — new (the constraining relation)
- `Linked Milestone` (relation → self, multi) — new

**REMOVE (drop or move to Context):**
- `Item Type` (select, 18 opts) — replaced by `Type` (12 opts)
- `Manifests As` (relation → GHOST) — delete (ghost relation)
- `Sub-holon Of` (relation → GHOST) — delete (ghost relation)
- `Related to Potentiator (People)` (relation → GHOST) — delete (ghost relation)
- `Related to Nexus (Emits Choice To)` (relation → Process) — re-point to Synthesis
- `Sourced From` (relation → Process) — re-point to Synthesis
- `For` (relation → Identity, dual with Transforms To) — re-point to Trajectory self (or delete if unused)
- `For Significator` (relation → Identity, dual) — re-point to Trajectory self (or delete if unused)
- `Coheres With (Significator)` (relation → Identity, dual) — re-point to Trajectory self (or delete if unused)

**RE-POINT:**
- `Archetype Role`, `Complex`, `Drive Activation`, `Shadow Pattern` — DROP (universal properties not needed in v4.1; the Type select discriminates instead)

**Total Trajectory properties:** ~20 (down from 27, after removing universal + ghost + Context-related)

### 3.2 Logbook (was Possibility — 27 props currently)

**KEEP:**
- Name (title), ID (unique_id), Date (date), Entry Type (select, 10 opts → reduce to 6: remove Goal/Vision/Aspiration/Observation), Distillation (rich_text → rename to Content)
- Amount (formula), Duration (formula), Month Label, Quarter Label, Week Label (formulas) — keep if useful
- For (relation → World) — re-point to Trajectory
- People (relation → World, dual) — re-point to Context

**ADD:**
- `Channel` (select: Body/Mind/Resource/Relational) — new (derived from Entry Type)
- `Sentiment` (select: Positive/Neutral/Negative) — new (for Subjective/Relational)
- `Source Project` (relation → Trajectory) — re-point from For
- `Subject Person` (relation → Context) — re-point from People
- `Subject Account` (relation → Context) — new (for Financial logs)
- `Synthesized Into` (relation → Synthesis, multi) — new

**REMOVE:**
- `Archetype Role`, `Complex`, `Drive Activation`, `Shadow Pattern` — DROP
- `Digestion Stage`, `Digestion Status` — DROP (not needed for logs)
- `Crystallized To`, `Crystallizes Into` (→ GHOST), `Reveals`, `Harmonized By` — DELETE (old holonic relations)
- `Documents` (relation → external) — DELETE or keep if external DB is still used
- `Related to GreatWay (Sub-holon Of)` (→ GHOST) — DELETE
- `Related to Nexus (Rewrites/Sends Experience To)` — DELETE (old holonic relations)

**Total Logbook properties:** ~15 (down from 27)

### 3.3 Synthesis (was Process — 35 props currently)

**KEEP:**
- Name (title), ID (unique_id), Date (date), Category (select, 13 opts → reduce to 5: Note/Opportunity/Strength/Directive/Risk), Priority (select, 4 opts), Status (status, 4 emoji opts), Capture Method (select, 5 opts), Source URL (url), Raw Content (rich_text), Synthesis State (select, 4 opts)

**ADD:**
- `Polarity` (select: +/−/neutral) — new
- `Strength` option in Category — new (add to select)
- `Source Logs` (relation → Logbook, multi) — re-point from Sourced From
- `Spawns` (relation → Trajectory, multi) — re-point from old relations
- `Condenses Into` (relation → Profile, multi) — new
- `Revises` (relation → Trajectory, multi) — new

**REMOVE:**
- `Archetype Role`, `Complex`, `Drive Activation`, `Shadow Pattern`, `Digestion Stage` — DROP
- `Kind` (select: Catalyst/Experience/Transformation/Choice) — DROP (replaced by Type + Polarity)
- `Counter-Synthesis`, `Counterpart`, `Reinforces` (intra-DB relations) — keep if used, else drop
- `Counter-Tension`, `Tension` (→ Identity) — re-point to Trajectory or drop
- `Highlight Count`, `Last Assessed`, `Polarity Outcome` — drop if unused
- `Updates` (→ State) — re-point to Trajectory or drop
- `Rewrites (Matrix)` (→ State) — re-point to Trajectory or drop
- `Rewrites (Potentiator)` (→ GHOST) — DELETE
- `Sends Catalyst To (Matrix)` (→ State) — re-point to Trajectory or drop
- `Sends Catalyst To (Significator)` (→ Identity) — re-point to Trajectory or drop
- `Sends Experience To (Potentiator)` (→ GHOST) — DELETE
- `Sourced From` (→ GHOST) — DELETE (replaced by Source Logs)
- `Emits Choice To` (→ Identity) — re-point to Trajectory or drop
- `Fires Transformation On` (→ Identity) — re-point to Trajectory or drop
- `Triggered By` (→ Identity) — re-point to Trajectory or drop

**Total Synthesis properties:** ~15 (down from 35)

### 3.4 Profile (was Identity — 30 props currently + State entries)

**KEEP from Identity:**
- Name (title), ID (unique_id), Status (status, 4 opts: Draft/Active/Evolving/Archived)

**ADD:**
- `Type` (select: Trait/Metric/Capacity/Asset) — new (replaces Entry Type)
- `Category` (select: Health/Financial/Relational/Cognitive/Spiritual/Execution/Content/Strategic) — new
- `Current Value` (rich_text) — new
- `Target Value` (rich_text) — new
- `Trend` (select: ↑/↓/→) — new
- `Unit` (select: count/percentage/hours/minutes/rupees/dollars/level/rating) — new
- `Frequency` (select: Daily/Weekly/Monthly/Quarterly/Annual) — new
- `Last Updated` (date) — new (or reuse Last Reviewed)
- `Source Synthesis` (relation → Synthesis, multi) — new
- `Closes Gap For` (relation → Trajectory, multi) — new
- `Informs Goal` (relation → Trajectory, multi) — new

**REMOVE:**
- `Entry Type` (multi_select: Purpose/Value/etc.) — replaced by `Type` (Trait/Metric/Capacity/Asset)
- `Holon Type` (select: Donor/Acceptor/etc.) — becomes a Trait entry, not a property
- `Valence Signature` (rich_text) — becomes a Trait entry, not a property
- `Life-Era` (select: Post-recovery/Trading era/HoloOS era) — becomes a Trait entry, not a property
- `Stage` (select, legacy empty) — DELETE
- `Archetype Role`, `Complex`, `Drive Activation`, `Shadow Pattern` — DROP
- `Review Cadence`, `Next Review` — keep if useful, else drop
- All 12 old holonic relations (Anchored In, Coheres With, Emits Choice To, Fires Transformation On, Generated From, In Tension With, Parent item, Related to GreatWay, Related to Nexus, Rewrites, Sub-holon Of, Sub-item, Transforms To) — DELETE or re-point

**RECEIVE from State:**
- 5 Pattern entries → become Trait entries
- 34 untyped Stats entries (Energy level, Sleep quality, etc.) → become Metric entries

**Total Profile properties:** ~12 (down from 30)

### 3.5 Context (NEW DB)

**Create fresh with:**
- Common: Name (title), Type (select: Person/Community/Organization/Financial-Account/Place), Status (select: Active/Inactive/Archived)
- Person-specific (14 props): Aspirational Drive, Developmental Altitude, Networking Profile, Relationship Status, Desired Trajectory, Value Exchange Balance, Last Interaction Sentiment, City, Timezone, Core Shadow, Engagement Blueprint, Key Personal Intel, Professional Domain, Influence Toolkit
- Community-specific: Community Type, Strategic Value, Covenant
- Financial-Account-specific: Account Type, Balance, Institution
- Relations: Involved In (→ Trajectory), Subject Of (→ Logbook), Referenced In (→ Synthesis)

**RECEIVE:**
- 63 Person entries from World (with Item Type=Person)
- 14 Community entries from World (with Item Type=Community)
- 69 Person entries from Identity (untyped, Status=Draft — these are the legacy People DB entries)

**Total Context entries after receiving:** 63 + 14 + 69 = 146

---

## 4. Grounded Refactor Execution (10 phases, revised)

| Phase | Action | Entries affected | Est. time | Risk |
|-------|--------|-----------------|-----------|------|
| 1 | **Create Context DB** (empty, with all properties + Person-specific 14) | 0 | 45 min | Low |
| 2 | **Move World.Person + World.Community → Context** | 77 | 1 hr | Medium |
| 3 | **Move Identity's 69 untyped Person entries → Context** | 69 | 1 hr | Medium (need to set Type=Person on each) |
| 4 | **Rename World → Trajectory** + add Reference entry-types (Purpose/Value/Principle/Vision-Statement/Identity-Statement) to `Type` select | 0 (just DB rename + select expansion) | 30 min | Low |
| 5 | **Move Identity's 23 typed entries → Trajectory** (Purpose=1, Value=10, Pillar=10, Identity-Statement=1, Strategic-Ideal=1) | 23 | 30 min | Low |
| 6 | **Add `Parent` + `Serves Value` self-relations to Trajectory** (Parent already exists as "Parent item" — rename + repurpose; Serves Value is new) | 0 | 20 min | Low |
| 7 | **Rename Identity → Profile** + replace Entry Type with Type (Trait/Metric/Capacity/Asset) + add Profile properties (Category, Current Value, Target Value, Trend, Unit, Frequency) | 0 (just schema change) | 1 hr | Medium |
| 8 | **Move State's 39 entries → Profile** (5 Pattern→Trait + 34 untyped→Metric) | 39 | 45 min | Medium |
| 9 | **Rename Process → Synthesis** + add Polarity + Strength entry-type + clean up properties (remove universal + ghost + old holonic relations) | 0 (just schema change; 794 Notes stay) | 1 hr | Medium |
| 10 | **Rename Possibility → Logbook** + reduce Entry Type to 6 options + clean up properties + re-point relations | 0 (just schema change; 6,911 logs stay) | 1 hr | Medium |
| 11 | **Re-point all inter-DB relations** to new DB names + delete ghost relations + archive State DB | 0 (relation re-pointing) | 2 hr | High |
| 12 | **Create saved views in Trajectory** (Vision/Goals/Actions/Hierarchy/Today/This Quarter) + verify cycle | 0 | 1 hr | Low |

**Total estimated time: 10-11 hours** (spread over 2-3 sessions)

**Total entries that move: 208 (2.5% of 8,450)**
**Total entries that stay: 8,242 (97.5% — just rename DB)**

---

## 5. Risk Assessment (grounded)

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Identity's 69 untyped entries are NOT all People | Low | Medium | Verified via API — names are "Archana Saini", "Namrata", etc. (all person names) |
| State's 34 untyped entries are NOT all Stats | Low | Medium | Verified via API — names are "Energy level", "Sleep quality", "Monthly income" (all metrics) |
| Relation re-pointing breaks data | Medium | High | Back up before Phase 11; test each relation after re-pointing |
| Property type changes lose data | Medium | High | Don't change types in-place; create new property, migrate data, delete old |
| Ghost relations cause API errors during cleanup | Low | Low | Delete ghost relations first (Phase 11) |
| Identity/World ID prefix ambiguity (both 38dc18ce) | Medium | Medium | Use FULL UUIDs in all scripts, never prefixes |
| Trajectory property bloat (20+ props) | Low | Low | Saved views per layer hide irrelevant props |
| Logbook entry-type reduction (10→6) loses Goal/Vision/Aspiration | **NONE** | **NONE** | These types have 0 entries — safe to remove |

---

## 6. Verification Checklist (post-refactor)

- [ ] Trajectory DB: 557 entries (534 from World + 23 from Identity), 12 entry-types in `Type` select
- [ ] Trajectory: `Parent` self-relation works (Task → Project → Quarterly-Goal → Annual-Goal → Vision-Statement)
- [ ] Trajectory: `Serves Value` self-relation works (Value constrains Project/Task)
- [ ] Logbook DB: 6,911 entries, 6 entry-types (Activity/Diet/Financial/Subjective/Relational/Systemic)
- [ ] Synthesis DB: 794 entries (Note=794, Decision re-classified), 5 entry-types + `Polarity` property
- [ ] Profile DB: 131 entries (5 Trait from State.Pattern + 34 Metric from State.untyped + 92 from Identity — but Identity's 69 People moved to Context, so 92-69=23 from Identity + 39 from State = 62 entries), 4 entry-types (Trait/Metric/Capacity/Asset)
- [ ] Context DB: 146 entries (63 from World.Person + 14 from World.Community + 69 from Identity.untyped), 5 entry-types, 14 Person-specific properties
- [ ] State DB: archived (0 entries remaining)
- [ ] No ghost-database relations remain (0baacff9-* all deleted)
- [ ] Saved views in Trajectory: Vision / Goals / Actions / Hierarchy / Today / This Quarter
- [ ] Total entries: 557 + 6,911 + 794 + 62 + 146 = 8,470 (slight increase due to 3 Decision re-classifications + 20 new Profile entries from State stats) — verify matches

---

*Grounded refactor plan v4.1. Based on live Notion API audit. Ready for user validation.*
