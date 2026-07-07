# LifeOS v4.1 — Formal DB Schema Specification (5-DB Merged Structure)
# =============================================================================
# **STATUS:** Approved. Implementation-ready.
# **CHANGE FROM v4.0:** Vision + Compass + Action merged into ONE DB ("Trajectory").
#                       The teleological pull is now the parent/child hierarchy.
# **COMPANION FILES:**
#   - formal_schema_v4.1.yaml — machine-readable version (for tooling)
#   - BLUEPRINT_v4.md — the design rationale (why 7 DBs → why 5 DBs)
#   - BRAINSTORM_5DB_merger.md — the merger analysis (viability + efficacy)

---

## 1. The 5 DBs at a Glance

| # | DB | Layer | Purpose | Entry-types | Discriminator |
|---|-----|-------|---------|-------------|---------------|
| 1 | **Trajectory** | A+B+C (merged) | The teleological hierarchy — pull IS the parent/child tree | Purpose, Value, Principle, Vision-Statement, Identity-Statement, Annual-Goal, Quarterly-Goal, Milestone, Project, Task, Campaign, Content | `Type` (select, 12 options) |
| 2 | **Logbook** | B — Historical Record | Objective ground-reality capture (6 channels) | Activity, Diet, Financial, Subjective, Relational, Systemic | `Entry Type` (select) |
| 3 | **Synthesis** | B — Historical Record | Logs → insights (polar pair) | Note, Opportunity, Strength, Directive, Risk | `Type` (select) + `Polarity` (select) |
| 4 | **Profile** | B — Historical Record | Cumulative state mirror (RPG status) | Trait, Metric, Capacity, Asset | `Type` (select) |
| 5 | **Context** | C — Action Interface | The environment (who/what is around) | Person, Community, Organization, Financial-Account, Place | `Type` (select) |

**Universal properties (on all 5 DBs):** `Name` (title), `Created Time`, `Last Edited Time` (Notion auto)

---

## 2. DB 1: Trajectory (The Merged Hierarchy)

**Purpose:** The teleological hierarchy. The pull IS the parent/child tree. Open one DB → expand the hierarchy → see the pull from Purpose to today's Task.

### 2.1 The 3 Layers (within Trajectory)

| Layer | Entry-types | Churn | Review cadence |
|-------|-------------|-------|----------------|
| **Reference** (timeless) | Purpose, Value, Principle, Vision-Statement, Identity-Statement | Years | Annual |
| **Strategic** (temporal) | Annual-Goal, Quarterly-Goal, Milestone | Quarters | Quarterly |
| **Execution** (daily) | Project, Task, Campaign, Content | Days | Daily |

### 2.2 The Hierarchy (parent/child via `Parent` self-relation)

```
Purpose ──────────── (constrains via `Serves Value`) ────────┐
Value ────────────── (constrains via `Serves Value`) ────────┤
Principle ────────── (constrains via `Serves Value`) ────────┤
                                                              ▼
Vision-Statement ──→ Annual-Goal ──→ Quarterly-Goal ──→ Project ──→ Task
                                          │                  │
                                          │                  ├──→ Campaign ──→ Content
                                          │                  │
                                          └── Milestone ◄───┘ (checkpoint, linked not parent)
Identity-Statement (informs the whole hierarchy, not a parent)
```

### 2.3 Entry-types (12 total)

| Entry-type | Layer | What it holds | Example |
|------------|-------|---------------|---------|
| `Purpose` | Reference | The deepest "why" — the user's core reason for being | "To become the fullest expression of consciousness I'm capable of" |
| `Value` | Reference | An enduring commitment — non-negotiable | "Integrity over convenience" |
| `Principle` | Reference | An operating rule — how decisions are made | "Default to the option that compounds" |
| `Vision-Statement` | Reference | A time-bound articulation of the ideal-future | "By 2030, I am a sovereign consciousness-prosthetic architect" |
| `Identity-Statement` | Reference | Who the user is becoming | "I am a person who builds systems that outlive me" |
| `Annual-Goal` | Strategic | A yearly target derived from Vision | "2026: Ship LifeOS v1.0" |
| `Quarterly-Goal` | Strategic | A quarterly decomposition of an Annual-Goal | "Q3 2026: Complete LifeOS blueprint + implement" |
| `Milestone` | Strategic | An event-bound checkpoint (not time-bound) | "LifeOS first full synthesis cycle complete" |
| `Project` | Execution | A multi-step deliverable aligned with a Goal | "Implement LifeOS v1.0 DB schema" |
| `Task` | Execution | An atomic unit of work | "Write Trajectory DB entry-types spec" |
| `Campaign` | Execution | A coordinated multi-content effort | "LifeOS launch campaign" |
| `Content` | Execution | A single content piece | "LifeOS architecture blog post" |

### 2.4 Properties

| Property | Type | Options / Notes | Applies to |
|----------|------|-----------------|------------|
| `Name` | title | — | All |
| `Type` | select | Purpose, Value, Principle, Vision-Statement, Identity-Statement, Annual-Goal, Quarterly-Goal, Milestone, Project, Task, Campaign, Content | All |
| `Description` | rich_text | The articulation / what it is | All |
| `Status` | status | Draft, Active, Evolving, Archived (Reference); Future, Ideation, Paused, Active, Done, Cancelled (Strategic+Execution) | All |
| `Parent` | relation → Trajectory (self) | Parent in the hierarchy (e.g. Task → Project → Quarterly-Goal → Annual-Goal → Vision-Statement) | Strategic + Execution |
| `Serves Value` | relation → Trajectory (self, multi) | Which Purpose/Value/Principle this entry aligns with (the constraining relation) | Strategic + Execution |
| `Source` | rich_text | Where this came from (a book, a mentor, a crisis) | Reference |
| `Timeframe` | select | Lifetime, 10yr, 5yr, 3yr, 1yr | Vision-Statement |
| `Last Reviewed` | date | When the user last reflected on this | Reference |
| `Year` | select | 2024, 2025, 2026, 2027, 2028, 2029, 2030 | Annual-Goal, Quarterly-Goal |
| `Quarter` | select | Q1, Q2, Q3, Q4 | Quarterly-Goal |
| `Progress` | number | 0-100 (%) | Strategic + Execution |
| `Target` | number | The measurable target | Strategic + Execution |
| `Start Date` | date | — | Strategic + Execution |
| `End Date` | date | — | Strategic + Execution |
| `Priority` | select | Critical, High, Medium, Low | Execution |
| `Blocked By` | relation → Trajectory (self, multi) | Dependency | Execution |
| `Assigned To` | relation → Context (Person) | Who is responsible | Execution |
| `Involves` | relation → Context (multi) | Which People/Communities are involved | Execution |
| `Generates Logs` | relation → Logbook (multi) | Which Logbook entries executing this generated | Execution |
| `Constrains` | relation → Trajectory (self, multi) | Which Projects/Tasks this Value/Principle constrains (inverse of Serves Value) | Reference |
| `Spawned By` | relation → Synthesis | Which Directive spawned this (if applicable) | Execution |
| `Measured By` | relation → Profile (multi) | Which Profile metrics track this | Strategic |
| `Linked Milestone` | relation → Trajectory (self, multi) | Which Milestones this Goal/Project is linked to | Strategic + Execution |

**Property count:** ~23 (most are empty per entry-type — saved views hide them)

### 2.5 Saved Views (essential for the merged DB)

| View | Filter | Sort | Purpose |
|------|--------|------|---------|
| `Vision` | Type = Purpose/Value/Principle/Vision-Statement/Identity-Statement | Last Reviewed desc | Annual review of timeless entries |
| `Goals` | Type = Annual-Goal/Quarterly-Goal/Milestone | Year + Quarter asc | Quarterly review of strategic layer |
| `Actions` | Type = Project/Task/Campaign/Content | Status + Priority | Daily review of execution layer |
| `Hierarchy` | All | Parent asc | The full teleological pull tree |
| `Today` | Type = Task + Status = Active + End Date = today | Priority asc | What to do today |
| `This Quarter` | Type = Quarterly-Goal + Quarter = current | Progress asc | Current quarter focus |

---

## 3. DB 2: Logbook (unchanged from v4.0)

**Purpose:** Objective ground-reality capture. ONE DB, 6 entry-types.

### Entry-types

| Entry-type | Channel | What it captures |
|------------|---------|------------------|
| `Activity` | Body | Physical activity + time use |
| `Diet` | Body | Food + nutrition |
| `Financial` | Resource | Transactions + money flow |
| `Subjective` | Mind | Subjective experience + learning + spirituality |
| `Relational` | Relational | Interactions + relational dynamics |
| `Systemic` | Mind | Process observations + workflow notes |

### Properties

| Property | Type | Options / Notes |
|----------|------|-----------------|
| `Name` | title | Quick summary |
| `Date` | date | When this happened (primary time index) |
| `Entry Type` | select | Activity, Diet, Financial, Subjective, Relational, Systemic |
| `Channel` | select | Body, Mind, Resource, Relational |
| `Content` | rich_text | The log content |
| `Amount` | number | For Financial (amount) or Diet (calories) |
| `Duration` | number | For Activity (minutes) |
| `Sentiment` | select | Positive, Neutral, Negative |
| `Source Project` | relation → Trajectory (Project/Task) | Which Project/Task this belongs to |
| `Subject Person` | relation → Context | For Relational logs |
| `Subject Account` | relation → Context | For Financial logs |
| `Synthesized Into` | relation → Synthesis (multi) | Which Synthesis entries this fed |

---

## 4. DB 3: Synthesis (unchanged from v4.0)

**Purpose:** Where logs become insights. Polar structure (± pair + raw).

### Entry-types

| Entry-type | Polarity | What it holds |
|------------|----------|---------------|
| `Note` | neutral | Raw synthesis capture |
| `Opportunity` | + | Positive trajectory signal — capitalize |
| `Strength` | + | Enduring capacity revealed — leverage |
| `Directive` | − | Corrective imperative — fix |
| `Risk` | − | Negative trajectory signal — what's breaking |

### Properties

| Property | Type | Options / Notes |
|----------|------|-----------------|
| `Name` | title | — |
| `Date` | date | When synthesized |
| `Type` | select | Note, Opportunity, Strength, Directive, Risk |
| `Polarity` | select | +, −, neutral |
| `Source Logs` | relation → Logbook (multi) | Which logs this synthesized from |
| `Priority` | select | Critical, High, Medium, Low |
| `Status` | select | raw, annotated, synthesized, applied |
| `Capture Method` | select | manual, web_clipper, api_ingest, voice_memo, import |
| `Source URL` | url | For web clips |
| `Spawns` | relation → Trajectory (multi) | For Directives — which Actions this spawned |
| `Condenses Into` | relation → Profile (multi) | Which Profile traits this condensed into |
| `Revises` | relation → Trajectory (multi) | Which Vision entries this reaffirms/revises |

---

## 5. DB 4: Profile (unchanged from v4.0)

**Purpose:** The cumulative state mirror. Bridge between history and the pull.

### Entry-types

| Entry-type | What it holds |
|------------|---------------|
| `Trait` | Enduring characteristic condensed from long-term trends |
| `Metric` | Measurable indicator with current value + trend |
| `Capacity` | Skill/capability level |
| `Asset` | Foundational asset built |

### Properties

| Property | Type | Options / Notes |
|----------|------|-----------------|
| `Name` | title | — |
| `Type` | select | Trait, Metric, Capacity, Asset |
| `Category` | select | Health, Financial, Relational, Cognitive, Spiritual, Execution, Content, Strategic |
| `Current Value` | rich_text | Current state |
| `Target Value` | rich_text | Ideal-future target |
| `Trend` | select | ↑, ↓, → |
| `Unit` | select | count, percentage, hours, minutes, rupees, dollars, level, rating |
| `Frequency` | select | Daily, Weekly, Monthly, Quarterly, Annual |
| `Last Updated` | date | — |
| `Source Synthesis` | relation → Synthesis (multi) | Which Synthesis entries condensed into this |
| `Closes Gap For` | relation → Trajectory (multi) | Which Vision entries this shows the gap for |
| `Informs Goal` | relation → Trajectory (multi) | Which Goals this informs |

---

## 6. DB 5: Context (unchanged from v4.0)

**Purpose:** The environment. Who/what is around the user.

### Entry-types

| Entry-type | What it holds |
|------------|---------------|
| `Person` | A person (rich CRM — 14 curated properties) |
| `Community` | A group/community |
| `Organization` | An organization |
| `Financial-Account` | A financial account |
| `Place` | A physical location |

### Properties

**Common:** Name, Type (Person/Community/Organization/Financial-Account/Place), Status (Active/Inactive/Archived)

**Person-specific (14 properties):** Aspirational Drive, Developmental Altitude, Networking Profile, Relationship Status, Desired Trajectory, Value Exchange Balance, Last Interaction Sentiment, City, Timezone, Core Shadow, Engagement Blueprint, Key Personal Intel, Professional Domain, Influence Toolkit

**Community-specific:** Community Type, Strategic Value, Covenant

**Financial-Account-specific:** Account Type, Balance, Institution

### Relations OUT

| Relation | Target | Cardinality |
|----------|--------|-------------|
| `Involved In` | → Trajectory (multi) | many-to-many |
| `Subject Of` | → Logbook (multi) | one-to-many |
| `Referenced In` | → Synthesis (multi) | one-to-many |

---

## 7. Relation Map (Complete)

### 7.1 Inter-DB relations (10 total — down from 18 in v4.0)

| # | From DB | Property | To DB | Cardinality | Flow | Semantic hint |
|---|---------|----------|-------|-------------|------|---------------|
| 1 | Trajectory | `Generates Logs` | Logbook | one-to-many | Ground | Executing this Action generated these Logs |
| 2 | Trajectory | `Assigned To` / `Involves` | Context | many-to-many | — | Person/Community involved in this Action |
| 3 | Trajectory | `Spawned By` | Synthesis | many-to-one | Ground (reverse) | Which Directive spawned this Action |
| 4 | Trajectory | `Measured By` | Profile | many-to-many | Feedback | Which Profile metrics track this Goal |
| 5 | Logbook | `Source Project` | Trajectory | many-to-one | Ground (reverse) | This Log belongs to this Project/Task |
| 6 | Logbook | `Subject Person` / `Subject Account` | Context | many-to-one | — | This Log is about this Person/Account |
| 7 | Logbook | `Synthesized Into` | Synthesis | many-to-many | Ground | This Log fed into these Synthesis entries |
| 8 | Synthesis | `Spawns` | Trajectory | one-to-many | Ground (reverse) | Directives spawn corrective Actions |
| 9 | Synthesis | `Condenses Into` | Profile | many-to-many | Ground | Long-term insights condense into Profile traits |
| 10 | Synthesis | `Revises` | Trajectory | many-to-many | Feedback | Long-term insights reaffirm/revise Vision |
| 11 | Profile | `Closes Gap For` | Trajectory | many-to-many | Feedback | Shows the gap between current state and ideal-future |
| 12 | Profile | `Informs Goal` | Trajectory | many-to-many | Feedback | Profile informs which Goals are needed |

### 7.2 Intra-DB relations (3 total — all in Trajectory)

| # | DB | Property | Target | Cardinality | Purpose |
|---|-----|----------|--------|-------------|---------|
| 1 | Trajectory | `Parent` | Trajectory (self) | many-to-one | The hierarchy: Task → Project → Quarterly-Goal → Annual-Goal → Vision-Statement |
| 2 | Trajectory | `Serves Value` | Trajectory (self, multi) | many-to-many | Which Purpose/Value/Principle this entry aligns with (the constraining relation) |
| 3 | Trajectory | `Blocked By` | Trajectory (self, multi) | many-to-many | Dependency (e.g. Task A blocked by Task B) |
| 4 | Trajectory | `Linked Milestone` | Trajectory (self, multi) | many-to-many | Which Milestones this Goal/Project is linked to |

**Relation count: 12 inter-DB + 4 intra-DB = 16 total (down from 22 in v4.0).** The merger eliminated 6 cross-DB relations that are now intra-DB self-relations.

---

## 8. The 3 Flows + The Cycle

### Flow 1 — Teleological Pull (within Trajectory, downward)

**Path:** Vision-Statement → Annual-Goal → Quarterly-Goal → Project → Task

**What flows:** The ideal-future shape propagates downward through the parent/child hierarchy. This is now STRUCTURAL — it's the tree shape of the DB, not a cross-DB relation.

**Constraint:** Purpose/Value/Principle constrain the hierarchy via `Serves Value` (a self-relation, not a flow).

### Flow 2 — Ground-Truth (Trajectory → Logbook → Synthesis → Profile)

**Path:** Trajectory (Action executed) → Logbook (capture) → Synthesis (process) → Profile (condense)

**What flows:** Objective ground-reality data — actions produce logs, logs synthesize into insights, insights condense into the cumulative state.

### Flow 3 — Feedback (Profile + Synthesis → Trajectory)

**Path:** Profile (gap) → Trajectory (Vision/Goals revised); Synthesis (long-term insights) → Trajectory (Vision reaffirmed/revised)

**What flows:** The gap signal + long-term insights inform the trajectory simulation.

### The Cycle

```
Trajectory → Logbook → Synthesis → Profile → Trajectory
  (pull +    (capture)  (process)   (condense)  (feedback)
   action)
```

**4 hops (down from 6 in v4.0).** Tighter feedback = faster amplification.

---

## 9. Migration Mapping (Current 5-DB → New 5-DB)

### 9.1 Current World (GreatWay) → Trajectory + Context

| Current entry-type | → New DB | → New entry-type | Notes |
|--------------------|----------|-----------------|-------|
| Annual Goal | Trajectory | Annual-Goal | Direct map |
| Quarterly Goal | Trajectory | Quarterly-Goal | Direct map |
| Project | Trajectory | Project | Direct map |
| Task | Trajectory | Task | Direct map |
| System | Trajectory | Project OR Profile (Asset) | If operational → Trajectory; if foundational → Profile |
| Resource | Trajectory | Project OR Profile (Asset) | Same logic |
| Sprint | Trajectory | Task or Project | Determine from scope |
| Milestone | Trajectory | Milestone | Direct map |
| Budget | Trajectory | Project OR Logbook (Financial) | If budget entry → Trajectory; if transaction → Logbook |
| Campaign | Trajectory | Campaign | Direct map |
| Content | Trajectory | Content | Direct map |
| Person | Context | Person | Direct map (port 14 properties) |
| Group | Context | Community | Direct map |
| Community | Context | Community | Direct map |
| Organization | Context | Organization | Direct map |
| Network | Context | Community or Organization | Determine from content |
| Movement | Context | Community | Direct map |
| Place | Context | Place | Direct map |

### 9.2 Current Identity (Significator) → Trajectory + Profile

| Current entry-type | → New DB | → New entry-type | Notes |
|--------------------|----------|-----------------|-------|
| Purpose | Trajectory | Purpose | Direct map |
| Value | Trajectory | Value | Direct map |
| Principle | Trajectory | Principle | Direct map |
| Identity-Statement | Trajectory | Identity-Statement | Direct map |
| Pillar | Trajectory | Value or Principle | Pillar → Trajectory (grouped via Serves Value) |
| Strategic-Ideal | Trajectory | Vision-Statement | Direct map |
| Holon Type | Profile | Trait | Type → trait |
| Valence Signature | Profile | Trait (rich_text) | Signature → trait |
| Life-Era (was Stage) | Profile | Trait | Era → trait |
| (Stats was here) | Profile | (entire DB) | Stats → Profile DB |

### 9.3 Current Possibility (Potentiator) → Logbook + Trajectory + Synthesis

| Current entry-type | → New DB | → New entry-type | Notes |
|--------------------|----------|-----------------|-------|
| Activity | Logbook | Activity | Direct map |
| Diet | Logbook | Diet | Direct map |
| Financial | Logbook | Financial | Direct map |
| Subjective | Logbook | Subjective | Direct map |
| Relational | Logbook | Relational | Direct map |
| Systemic | Logbook | Systemic | Direct map |
| Goal | Trajectory | Annual-Goal or Quarterly-Goal | Determine timeframe |
| Vision | Trajectory | Vision-Statement | Direct map |
| Aspiration | Trajectory | Identity-Statement or Vision-Statement | Determine from content |
| Observation | Synthesis | Note | Raw observation → synthesis note |

### 9.4 Current Process (Nexus) → Synthesis + Profile + Trajectory

| Current entry-type | → New DB | → New entry-type | Notes |
|--------------------|----------|-----------------|-------|
| Note | Synthesis | Note | Direct map |
| Knowledge-Category | Synthesis | Note (with sub-type) | Taxonomy node → synthesis note |
| Knowledge-Atom | Synthesis | Note (with sub-type) | Discrete knowledge → synthesis note |
| Opportunity | Synthesis | Opportunity | Direct map |
| Insight | Synthesis | Strength or Note | Determine from content |
| Reflection | Synthesis | Note | Direct map |
| Integration | Synthesis | Note | Direct map |
| Pattern | Profile | Trait | Pattern recognition → trait |
| Risk | Synthesis | Risk | Direct map |
| Directive | Synthesis | Directive | Direct map |
| Decision | Synthesis OR Trajectory | Note OR Task | If actionable → Trajectory (Task); if reflective → Synthesis (Note) |
| Crisis | Synthesis | Risk | Direct map |
| Transformation-Event | Synthesis OR Trajectory | Note OR Identity-Statement | If identity-shifting → Trajectory; else Synthesis |

### 9.5 Current State (Matrix) → Profile

| Current entry-type | → New DB | → New entry-type | Notes |
|--------------------|----------|-----------------|-------|
| Pattern | Profile | Trait | Recurring behavior → enduring trait |
| Threshold | Profile | Trait | Boundary → trait |
| Foundation | Profile | Asset | Load-bearing structure → asset |

### 9.6 Migration principles

1. **No data loss.** Every entry-type in the current 5-DB has a clear destination.
2. **Properties follow.** Each entry's properties migrate with it.
3. **Relations re-point.** After migration, relations re-point to the new DB locations.
4. **World + Identity merge into Trajectory.** This is the biggest change — World's Goals/Projects/Tasks + Identity's Purpose/Values merge into ONE DB.
5. **Build the Trajectory DB first** (it's the merged World+Identity), then refactor the other 4 DBs in place.

---

## 10. Implementation Order (Refactor Approach)

Since the new 5-DB maps closely to the current 5-DB, we REFACTOR in place (not build fresh):

| Order | Action | What happens |
|-------|--------|--------------|
| 1 | **Rename World → Trajectory** | The World DB becomes Trajectory. It already has Goals/Projects/Tasks/Campaigns/Content. |
| 2 | **Add Reference entry-types to Trajectory** | Add Purpose, Value, Principle, Vision-Statement, Identity-Statement as new `Type` options. Port these from Identity DB. |
| 3 | **Add `Parent` self-relation to Trajectory** | This creates the hierarchy (Task → Project → Quarterly-Goal → Annual-Goal → Vision-Statement). |
| 4 | **Add `Serves Value` self-relation to Trajectory** | The constraining relation (Purpose/Value/Principle constrain Goals/Projects/Tasks). |
| 5 | **Move Stats-related entries from Identity → Profile** | Rename Identity DB to Profile. Move Holon Type, Valence Signature, Life-Era → Profile as Traits. Move Pattern/Threshold/Foundation from State → Profile. |
| 6 | **Rename State → Logbook** (if State has log entries) OR **merge State into Logbook** | State's logs (if any) → Logbook. State's Pattern/Threshold/Foundation → Profile (already done in step 5). |
| 7 | **Rename Process → Synthesis** | Process already has Notes/Opportunities/Directives/Risks. Add Strength entry-type. Clean up. |
| 8 | **Rename Possibility → Logbook** | Possibility already has the 6 logs (Activity/Diet/Financial/Subjective/Relational/Systemic). Move Goals/Vision/Aspiration → Trajectory (already done in step 2-3). Move Observation → Synthesis. |
| 9 | **Split Context from Trajectory** | Trajectory's Person/Community/Organization/Financial-Account/Place → new Context DB. (OR: if World already has these, move them out.) |
| 10 | **Re-point all relations** | Update relation properties to point to the new DB names. |
| 11 | **Create saved views in Trajectory** | Vision view, Goals view, Actions view, Hierarchy view, Today view, This Quarter view. |
| 12 | **Test the cycle** | Verify: Trajectory → Logbook → Synthesis → Profile → Trajectory flows work end-to-end. |

**Estimated time:** 4-6 hours of Notion work (renaming, adding entry-types, creating relations, setting up views, re-pointing relations). Plus 2-4 hours for data migration of entries that move between DBs.

---

*Formal spec v4.1. The 5-DB merged structure. Ready for implementation.*
