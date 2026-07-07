# LifeOS v4.1 — Full-Scale Refactor Plan
# =============================================================================
# **PURPOSE:** Plan the refactor of the current 5-DB (State/Possibility/Process/
#              Identity/World) into the new 5-DB (Trajectory/Logbook/Synthesis/
#              Profile/Context).
#
# **BASED ON:** The live audit of the current 5 DBs (see audit data below).
#
# **APPROACH:** Refactor in place (rename + add entry-types + move entries +
#               re-point relations), not build fresh.
#
# **COMPANION:** FORMAL_SPEC_v4.1.md (the target schema)

---

## 1. Current State Audit (Live from Notion, 2026-07-06)

| Current DB | Properties | Entries | Entry-types (select options) |
|------------|------------|---------|------------------------------|
| **State (matrix)** | 25 | 39 | Entry Type: Pattern, Threshold, Foundation (3) |
| **Possibility (potentiator)** | 27 | 6,900 | Entry Type: Activity, Subjective, Relational, Systemic, Diet, Financial, Observation, Goal, Vision, Aspiration (10) |
| **Process (nexus)** | 35 | 797 | Category: Opportunity, Directive, Risk, Insight, Reflection, Integration, Pattern, Note, Knowledge-Category, Knowledge-Atom, Decision, Crisis, Transformation-Event (13); Kind: Catalyst, Experience, Transformation, Choice (4) |
| **Identity (significator)** | 30 | 92 | Entry Type: Purpose, Value, Principle, Identity-Statement, Pillar, Strategic-Ideal (6) |
| **World (greatway)** | 27 | 611 | Item Type: Annual Goal, Quarterly Goal, Project, Task, System, Resource, Sprint, Milestone, Budget, Campaign, Content, Person, Group, Community, Organization, Network, Movement, Place (18) |
| **TOTAL** | **144** | **8,439** | — |

### Key observations from the audit

1. **World already has the Trajectory entry-types.** Annual Goal, Quarterly Goal, Project, Task, Milestone, Campaign, Content — these are 7 of the 12 Trajectory entry-types. They're already in World.
2. **World also has Context entry-types.** Person, Group, Community, Organization, Network, Movement, Place, (Financial-Account is missing — was in a separate legacy DB). These need to move OUT of World into Context.
3. **Identity has the Reference entry-types.** Purpose, Value, Principle, Identity-Statement, Pillar, Strategic-Ideal — these are 6 of the 12 Trajectory entry-types. They need to move FROM Identity TO Trajectory (World).
4. **Possibility has 6 log entry-types + 4 non-log entry-types.** Activity/Diet/Financial/Subjective/Relational/Systemic → Logbook. Goal/Vision/Aspiration → Trajectory. Observation → Synthesis.
5. **State has 3 entry-types, all → Profile.** Pattern→Trait, Threshold→Trait, Foundation→Asset.
6. **Process has 13 entry-types.** Most → Synthesis. Pattern → Profile. Decision → Trajectory (Task) or Synthesis (Note).
7. **Properties to preserve:** World has Progress, Target, Start Date, End Date, Priority, Status, Quadrant, Tier — all useful for Trajectory. Identity has Last Reviewed, Review Cadence, Status — useful for Trajectory's Reference layer.
8. **Ghost-database relations** still present (0baacff9-* in Possibility.Crystallizes Into, Process.Sourced From, Process.Rewrites (Potentiator), Process.Sends Experience To (Potentiator), World.Manifests As, World.Sub-holon Of, World.Related to Potentiator (People)). These need cleanup during refactor.
9. **Process.Status has emojis** (💡 Identified, ✅ Activated, 🏆 Capitalized, 🧊 Archived). Keep as-is (config already matches).
10. **Identity has both Stage AND Life-Era** (legacy from v0.10.2 rename). Stage is empty; Life-Era has 3 options. Clean up: delete Stage.

---

## 2. The Refactor Map (Current → New)

### 2.1 DB Renames

| Current DB | → New DB | Action |
|------------|----------|--------|
| World (greatway) | **Trajectory** | Rename. Add Reference entry-types from Identity. Remove Context entry-types (→ new Context DB). |
| Possibility (potentiator) | **Logbook** | Rename. Remove non-log entry-types (Goal/Vision/Aspiration → Trajectory, Observation → Synthesis). Keep 6 log entry-types. |
| Process (nexus) | **Synthesis** | Rename. Remove Pattern (→ Profile). Remove Decision (→ Trajectory as Task). Add Strength entry-type. Keep Note/Opportunity/Directive/Risk. |
| Identity (significator) | **Profile** | Rename. Remove Reference entry-types (→ Trajectory). Add Trait/Metric/Capacity/Asset entry-types. Receive Pattern/Threshold/Foundation from State + Holon-Type/Valence/Life-Era from old Identity. |
| State (matrix) | (dissolved) | Entries move to Profile (Pattern→Trait, Threshold→Trait, Foundation→Asset). DB archived. |
| (new) | **Context** | Create new DB. Receive Person/Community/Org/Place from World. Add Financial-Account (from legacy or new). |

### 2.2 Entry-Type Moves

| From DB | Entry-type | → To DB | → New entry-type | Entries (approx) | Action |
|---------|------------|---------|-----------------|------------------|--------|
| World | Annual Goal | Trajectory | Annual-Goal | ~20 | Rename (stays) |
| World | Quarterly Goal | Trajectory | Quarterly-Goal | ~30 | Rename (stays) |
| World | Project | Trajectory | Project | ~50 | Stays |
| World | Task | Trajectory | Task | ~100 | Stays |
| World | System | Trajectory | Project | ~20 | Re-classify |
| World | Resource | Trajectory | Project | ~15 | Re-classify |
| World | Sprint | Trajectory | Task | ~10 | Re-classify |
| World | Milestone | Trajectory | Milestone | ~10 | Stays |
| World | Budget | Trajectory | Project | ~10 | Re-classify (or → Logbook.Financial if transactional) |
| World | Campaign | Trajectory | Campaign | ~10 | Stays |
| World | Content | Trajectory | Content | ~50 | Stays |
| World | Person | Context | Person | ~63 | MOVE to Context DB |
| World | Group | Context | Community | ~5 | MOVE to Context |
| World | Community | Context | Community | ~14 | MOVE to Context |
| World | Organization | Context | Organization | ~10 | MOVE to Context |
| World | Network | Context | Community | ~5 | MOVE + re-classify |
| World | Movement | Context | Community | ~5 | MOVE + re-classify |
| World | Place | Context | Place | ~10 | MOVE to Context |
| Identity | Purpose | Trajectory | Purpose | ~5 | MOVE to Trajectory |
| Identity | Value | Trajectory | Value | ~15 | MOVE to Trajectory |
| Identity | Principle | Trajectory | Principle | ~10 | MOVE to Trajectory |
| Identity | Identity-Statement | Trajectory | Identity-Statement | ~5 | MOVE to Trajectory |
| Identity | Pillar | Trajectory | Value or Principle | ~5 | MOVE + re-classify (group via Serves Value) |
| Identity | Strategic-Ideal | Trajectory | Vision-Statement | ~5 | MOVE + rename |
| Identity | (Holon Type) | Profile | Trait | ~5 | MOVE (property → entry) |
| Identity | (Valence Signature) | Profile | Trait | ~5 | MOVE (property → entry) |
| Identity | (Life-Era) | Profile | Trait | ~3 | MOVE (property → entry) |
| Possibility | Activity | Logbook | Activity | ~3000 | Stays (rename DB) |
| Possibility | Diet | Logbook | Diet | ~1000 | Stays |
| Possibility | Financial | Logbook | Financial | ~500 | Stays |
| Possibility | Subjective | Logbook | Subjective | ~1000 | Stays |
| Possibility | Relational | Logbook | Relational | ~500 | Stays |
| Possibility | Systemic | Logbook | Systemic | ~300 | Stays |
| Possibility | Goal | Trajectory | Annual-Goal or Quarterly-Goal | ~50 | MOVE to Trajectory |
| Possibility | Vision | Trajectory | Vision-Statement | ~20 | MOVE to Trajectory |
| Possibility | Aspiration | Trajectory | Identity-Statement or Vision-Statement | ~20 | MOVE to Trajectory |
| Possibility | Observation | Synthesis | Note | ~100 | MOVE to Synthesis |
| Process | Note | Synthesis | Note | ~300 | Stays (rename DB) |
| Process | Opportunity | Synthesis | Opportunity | ~100 | Stays |
| Process | Directive | Synthesis | Directive | ~50 | Stays |
| Process | Risk | Synthesis | Risk | ~50 | Stays |
| Process | Insight | Synthesis | Strength or Note | ~50 | Re-classify |
| Process | Reflection | Synthesis | Note | ~30 | Re-classify |
| Process | Integration | Synthesis | Note | ~20 | Re-classify |
| Process | Pattern | Profile | Trait | ~30 | MOVE to Profile |
| Process | Decision | Trajectory or Synthesis | Task or Note | ~20 | Re-classify (actionable → Trajectory; reflective → Synthesis) |
| Process | Crisis | Synthesis | Risk | ~10 | Re-classify |
| Process | Transformation-Event | Synthesis or Trajectory | Note or Identity-Statement | ~5 | Re-classify (identity-shifting → Trajectory; else Synthesis) |
| Process | Knowledge-Category | Synthesis | Note | ~10 | Re-classify |
| Process | Knowledge-Atom | Synthesis | Note | ~20 | Re-classify |
| State | Pattern | Profile | Trait | ~20 | MOVE to Profile |
| State | Threshold | Profile | Trait | ~10 | MOVE to Profile |
| State | Foundation | Profile | Asset | ~9 | MOVE to Profile |

### 2.3 Property Changes

**Trajectory (was World) — properties to ADD:**
- `Type` (select, 12 options) — replaces `Item Type` (expand from 18 to 12 by removing Context types + adding Reference types)
- `Parent` (relation → Trajectory self) — NEW. The hierarchy.
- `Serves Value` (relation → Trajectory self, multi) — NEW. The constraining relation.
- `Description` (rich_text) — NEW (or repurpose existing)
- `Source` (rich_text) — NEW (from Identity)
- `Timeframe` (select: Lifetime/10yr/5yr/3yr/1yr) — NEW (from Identity or fresh)
- `Last Reviewed` (date) — NEW (from Identity)
- `Linked Milestone` (relation → Trajectory self, multi) — NEW

**Trajectory — properties to KEEP from World:**
- `Status`, `Progress`, `Target`, `Start Date`, `End Date`, `Priority`, `Name`, `ID`

**Trajectory — properties to REMOVE (move to Context or drop):**
- `Quadrant` (Wilber — not in v4.1 spec; keep if used, else drop)
- `Tier` (Strategic/Operational/Tactical — keep if used, else drop)
- `Monitor` (formula — keep if useful)
- All relations to Identity/Process/Possibility (re-point to new DB names)

**Logbook (was Possibility) — properties to KEEP:**
- `Name`, `Date`, `Entry Type` (rename from `Entry Type`), `Content` (rename from `Distillation` or add), `Amount`, `Duration`, `Sentiment`
- `People` relation → re-point to Context
- `For` relation → re-point to Trajectory (Project/Task)

**Logbook — properties to REMOVE:**
- `Archetype Role`, `Complex`, `Drive Activation`, `Shadow Pattern`, `Digestion Stage`, `Digestion Status` (universal properties — drop from Logbook; they belong on Profile/Trajectory, not logs)
- `Crystallized To`, `Crystallizes Into`, `Reveals`, `Harmonized By` (old holonic relations — drop or re-point)
- `Documents` relation (external — drop)
- `Sub-holon Of`, `Rewrites`, `Sends Experience To` (old holonic relations — drop)
- Formula properties (Amount, Duration, Month Label, Quarter Label, Week Label) — keep if useful, else drop

**Synthesis (was Process) — properties to KEEP:**
- `Name`, `Date`, `Category` (rename to `Type`), `Kind` (keep or drop — polar structure is in `Polarity`), `Priority`, `Status`, `Capture Method`, `Source URL`, `Raw Content`, `Synthesis State`
- Relations → re-point: `Updates` → Trajectory, `Sourced From` → Logbook, `Rewrites` → Trajectory, `Sends Catalyst/Experience To` → Logbook, `Emits Choice To` → Trajectory, `Fires Transformation On` → Trajectory, `Triggered By` → Trajectory

**Synthesis — properties to ADD:**
- `Polarity` (select: +/−/neutral) — NEW
- `Source Logs` (relation → Logbook, multi) — re-point from `Sourced From`
- `Spawns` (relation → Trajectory, multi) — re-point from old relations
- `Condenses Into` (relation → Profile, multi) — NEW
- `Revises` (relation → Trajectory, multi) — NEW

**Synthesis — properties to REMOVE:**
- `Archetype Role`, `Complex`, `Drive Activation`, `Shadow Pattern`, `Digestion Stage` (universal properties — drop)
- `Counter-Synthesis`, `Counterpart`, `Reinforces`, `Counter-Tension`, `Tension` (intra-DB dialectical relations — keep if used, else drop)
- `Highlight Count`, `Last Assessed`, `Polarity Outcome` (drop if unused)

**Profile (was Identity) — properties to ADD:**
- `Type` (select: Trait/Metric/Capacity/Asset) — NEW (replaces `Entry Type`)
- `Category` (select: Health/Financial/Relational/Cognitive/Spiritual/Execution/Content/Strategic) — NEW
- `Current Value` (rich_text) — NEW
- `Target Value` (rich_text) — NEW
- `Trend` (select: ↑/↓/→) — NEW
- `Unit` (select) — NEW
- `Frequency` (select) — NEW
- `Source Synthesis` (relation → Synthesis, multi) — NEW
- `Closes Gap For` (relation → Trajectory, multi) — NEW
- `Informs Goal` (relation → Trajectory, multi) — NEW

**Profile — properties to REMOVE (move to Trajectory or drop):**
- `Entry Type` (Purpose/Value/etc.) → moves to Trajectory
- `Holon Type`, `Valence Signature`, `Life-Era` → become Profile Traits (entries, not properties)
- `Archetype Role`, `Complex`, `Drive Activation`, `Shadow Pattern` (universal — drop)
- All old holonic relations (Anchored In, Coheres With, Emits Choice To, Fires Transformation On, Generated From, In Tension With, Parent item, Related to GreatWay, Related to Nexus, Rewrites, Sub-holon Of, Sub-item, Transforms To) — drop or re-point
- `Stage` (legacy, empty) — delete
- `Review Cadence`, `Next Review` — keep if useful, else drop

**Context (NEW DB) — properties:**
- See FORMAL_SPEC_v4.1.md §6 for the full Context schema (common + Person-specific 14 + Community-specific + Financial-Account-specific)

---

## 3. Refactor Execution Order (12 phases)

| Phase | Action | Estimated time | Risk |
|-------|--------|----------------|------|
| 1 | **Create Context DB** in Notion (empty, with all properties) | 30 min | Low |
| 2 | **Move Context entries from World → Context** (Person, Community, Org, Place — ~107 entries) | 1 hr | Medium (data move) |
| 3 | **Rename World → Trajectory** | 5 min | Low |
| 4 | **Add Reference entry-types to Trajectory** (Purpose, Value, Principle, Vision-Statement, Identity-Statement — add as `Type` options) | 15 min | Low |
| 5 | **Move Reference entries from Identity → Trajectory** (~45 entries) | 45 min | Medium |
| 6 | **Move Goal/Vision/Aspiration from Possibility → Trajectory** (~90 entries) | 45 min | Medium |
| 7 | **Add `Parent` + `Serves Value` self-relations to Trajectory** | 15 min | Low |
| 8 | **Rename Identity → Profile** + add Profile entry-types (Trait/Metric/Capacity/Asset) + add Profile properties | 1 hr | Medium |
| 9 | **Move Pattern/Threshold/Foundation from State → Profile** (~39 entries) + move Pattern from Process → Profile (~30 entries) | 1 hr | Medium |
| 10 | **Rename Process → Synthesis** + add `Polarity` + `Strength` entry-type + re-classify entries (Insight→Strength/Note, Reflection→Note, etc.) | 1.5 hr | Medium |
| 11 | **Rename Possibility → Logbook** + remove universal properties (Archetype Role, Complex, etc.) + re-point relations | 1 hr | Medium |
| 12 | **Re-point all inter-DB relations** to new DB names + create saved views in Trajectory + archive State DB | 2 hr | High (relation re-pointing) |

**Total estimated time: 10-12 hours** (can be spread over 2-3 sessions).

---

## 4. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Data loss during entry moves | Low | High | Back up each DB before moving (export to CSV/Markdown) |
| Relation breakage during re-pointing | Medium | Medium | Test each relation after re-pointing; keep old relations until new ones verified |
| Property data loss when changing property types | Medium | High | Don't change types in-place — create new property, migrate data, then delete old |
| Entry-type re-classification errors (e.g. Insight→Strength vs Note) | Medium | Low | User reviews each re-classification manually (per AGENTS.md §6.1 — no bulk changes) |
| Ghost-database relations (0baacff9-*) cause errors | Low | Low | Already identified; clean up during Phase 12 |
| Mixed-churn confusion in Trajectory (Purpose next to Task) | Medium | Low | Saved views per layer (Vision/Goals/Actions) — set up in Phase 12 |
| 6,900 Possibility entries slow to migrate | Low | Low | Only ~90 entries move OUT (Goal/Vision/Aspiration); 6,810 stay (rename DB) |

---

## 5. Verification Checklist (post-refactor)

- [ ] Trajectory DB has 12 entry-types (5 Reference + 3 Strategic + 4 Execution)
- [ ] Trajectory has `Parent` self-relation (hierarchy works)
- [ ] Trajectory has `Serves Value` self-relation (constraining works)
- [ ] Logbook DB has 6 entry-types (Activity/Diet/Financial/Subjective/Relational/Systemic)
- [ ] Synthesis DB has 5 entry-types (Note/Opportunity/Strength/Directive/Risk) + `Polarity` property
- [ ] Profile DB has 4 entry-types (Trait/Metric/Capacity/Asset) + Category/Current Value/Target Value/Trend
- [ ] Context DB has 5 entry-types (Person/Community/Organization/Financial-Account/Place) + 14 Person properties
- [ ] Inter-DB relations: Trajectory↔Logbook, Trajectory↔Synthesis, Trajectory↔Profile, Trajectory↔Context, Logbook↔Synthesis, Logbook↔Context, Synthesis↔Profile
- [ ] No ghost-database relations (0baacff9-*) remain
- [ ] State DB archived (entries moved to Profile)
- [ ] Saved views in Trajectory: Vision / Goals / Actions / Hierarchy / Today / This Quarter
- [ ] Entry counts match: ~8,439 total entries across 5 DBs (Trajectory ~400, Logbook ~6,800, Synthesis ~750, Profile ~120, Context ~120)

---

*Refactor plan v4.1. Ready for execution.*
