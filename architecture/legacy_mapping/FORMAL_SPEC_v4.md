# LifeOS v4 — Formal DB Schema Specification
# =============================================================================
# **STATUS:** Approved blueprint, formalized for implementation.
# **PURPOSE:** Complete DB-schema-level spec for the 7-DB LifeOS structure.
#              This document is the single source of truth for building the
#              Notion system + the Rust tooling that operates on it.
#
# **COMPANION FILES:**
#   - formal_schema.yaml — machine-readable version of this spec (for tooling)
#   - visualize_formal_spec.py — generates the formal structure visualization
#   - BLUEPRINT_v4.md — the design rationale (why 7 DBs, why these layers)
#
# **STRUCTURE:**
#   §1  The 7 DBs at a glance
#   §2  Per-DB schemas (entry-types, properties, types, options)
#   §3  Relation map (inter-DB + intra-DB, all cardinalities + semantic hints)
#   §4  The 3 flows + the cycle (formal trigger + direction specification)
#   §5  Migration mapping (current 5-DB → new 7-DB, entry-type by entry-type)
#   §6  Implementation order (which DB to build first, second, etc.)

---

## 1. The 7 DBs at a Glance

| # | DB | Layer | Purpose | Entry-types | Discriminator |
|---|-----|-------|---------|-------------|---------------|
| 1 | **Vision** | A — Teleological Pull | Articulate the ideal-future | Purpose, Value, Principle, Vision-Statement, Identity-Statement | `Type` (select) |
| 2 | **Compass** | A — Teleological Pull | Time-bound trajectory decomposition | Annual-Goal, Quarterly-Goal, Milestone | `Type` (select) |
| 3 | **Logbook** | B — Historical Record | Objective ground-reality capture (6 channels) | Activity, Diet, Financial, Subjective, Relational, Systemic | `Entry Type` (select) |
| 4 | **Synthesis** | B — Historical Record | Logs → insights (polar pair) | Note, Opportunity, Strength, Directive, Risk | `Type` (select) + `Polarity` (select) |
| 5 | **Profile** | B — Historical Record | Cumulative state mirror (RPG status) | Trait, Metric, Capacity, Asset | `Type` (select) |
| 6 | **Action** | C — Action Interface | What to do + how (hierarchy) | Project, Task, Campaign, Content | `Type` (select) |
| 7 | **Context** | C — Action Interface | The environment (who/what is around) | Person, Community, Organization, Financial-Account, Place | `Type` (select) |

**Universal properties (on all 7 DBs):** `Name` (title), `Created Time`, `Last Edited Time` (Notion auto)

---

## 2. Per-DB Schemas

### DB 1: Vision (Layer A)

**Purpose:** Articulate the ideal-future. The teleological attractor. Timeless (evolves slowly, doesn't change daily).

#### Entry-types (discriminated by `Type` property)

| Entry-type | What it holds | Example |
|------------|---------------|---------|
| `Purpose` | The deepest "why" — the user's core reason for being | "To become the fullest expression of consciousness I'm capable of" |
| `Value` | An enduring commitment — non-negotiable | "Integrity over convenience" |
| `Principle` | An operating rule — how decisions are made | "Default to the option that compounds" |
| `Vision-Statement` | A time-bound articulation of the ideal-future | "By 2030, I am a sovereign consciousness-prosthetic architect" |
| `Identity-Statement` | Who the user is becoming | "I am a person who builds systems that outlive me" |

#### Properties

| Property | Type | Options / Notes |
|----------|------|-----------------|
| `Name` | title | — |
| `Description` | rich_text | The articulation itself |
| `Type` | select | Purpose, Value, Principle, Vision-Statement, Identity-Statement |
| `Source` | rich_text | Where this came from (a book, a mentor, a crisis, etc.) |
| `Timeframe` | select | Lifetime, 10yr, 5yr, 3yr, 1yr (only for Vision-Statement; empty otherwise) |
| `Status` | status | Draft, Active, Evolving, Archived |
| `Last Reviewed` | date | When the user last reflected on this |
| `Pillar` | relation → Vision (self) | Groups related Values/Principles into a pillar |
| `Decomposes Into` | relation → Compass (multi) | Which Compass goals this Vision-Statement decomposes into |
| `Constrains` | relation → Action (multi) | Which Projects/Actions this Value/Principle constrains |

---

### DB 2: Compass (Layer A)

**Purpose:** Time-bound trajectory decomposition. How the ideal-future pulls across time. Temporal (changes quarterly).

#### Entry-types (discriminated by `Type` property)

| Entry-type | What it holds | Example |
|------------|---------------|---------|
| `Annual-Goal` | A yearly target derived from Vision | "2026: Ship LifeOS v1.0" |
| `Quarterly-Goal` | A quarterly decomposition of an Annual-Goal | "Q3 2026: Complete LifeOS blueprint + implement" |
| `Milestone` | An event-bound checkpoint (not time-bound) | "LifeOS first full synthesis cycle complete" |

#### Properties

| Property | Type | Options / Notes |
|----------|------|-----------------|
| `Name` | title | — |
| `Type` | select | Annual-Goal, Quarterly-Goal, Milestone |
| `Year` | select | 2024, 2025, 2026, 2027, 2028, 2029, 2030 |
| `Quarter` | select | Q1, Q2, Q3, Q4 (only for Quarterly-Goal) |
| `Status` | status | Future, Ideation, Active, Done, Cancelled |
| `Progress` | number | 0-100 (%) |
| `Target` | number | The measurable target (if quantifiable) |
| `Start Date` | date | — |
| `End Date` | date | — |
| `Parent Goal` | relation → Compass (self) | Annual-Goal is parent of Quarterly-Goal |
| `Derives From` | relation → Vision | The Vision entry this goal traces to |
| `Serves Value` | relation → Vision (multi) | Which Values/Principles this goal serves |
| `Decomposes Into` | relation → Action (multi) | Which Projects this goal decomposes into |
| `Measured By` | relation → Profile (multi) | Which Profile metrics track this goal's progress |

---

### DB 3: Logbook (Layer B)

**Purpose:** Objective ground-reality capture. ONE DB, 6 entry-types (the 6 channels). Daily-use, high-volume.

#### Entry-types (discriminated by `Entry Type` property)

| Entry-type | Channel | What it captures | Example |
|------------|---------|------------------|---------|
| `Activity` | Body | Physical activity + time use | "45min run, 6am" |
| `Diet` | Body | Food + nutrition | "Lunch: dal, rice, sabzi" |
| `Financial` | Resource | Transactions + money flow | "₹2000 groceries" |
| `Subjective` | Mind | Subjective experience + learning + spirituality | "Realized X about Y" |
| `Relational` | Relational | Interactions + relational dynamics | "1:1 with Ishaan — deep" |
| `Systemic` | Mind | Process observations + workflow notes | "LifeOS morning view feels slow" |

#### Properties

| Property | Type | Options / Notes |
|----------|------|-----------------|
| `Name` | title | Quick summary |
| `Date` | date | When this happened (the primary time index) |
| `Entry Type` | select | Activity, Diet, Financial, Subjective, Relational, Systemic |
| `Channel` | select | Body, Mind, Resource, Relational (derived from Entry Type, explicit for filtering) |
| `Content` | rich_text | The log content itself |
| `Amount` | number | For Financial (transaction amount) or Diet (calories) |
| `Duration` | number | For Activity (minutes) |
| `Sentiment` | select | Positive, Neutral, Negative (for Subjective/Relational) |
| `Source Project` | relation → Action | Which Project this log belongs to |
| `Source Task` | relation → Action | Which Task this log belongs to |
| `Subject Person` | relation → Context | For Relational logs — who was the interaction with |
| `Subject Account` | relation → Context | For Financial logs — which account |
| `Synthesized Into` | relation → Synthesis (multi) | Which Synthesis entries this log fed into |

---

### DB 4: Synthesis (Layer B)

**Purpose:** Where logs become insights. The synthesis pipeline. Polar structure (± pair + raw input).

#### Entry-types (discriminated by `Type` property, with `Polarity`)

| Entry-type | Polarity | What it holds | Example |
|------------|----------|---------------|---------|
| `Note` | neutral | Raw synthesis capture (meeting notes, web clips, voice memos) | "Meeting with Ishaan — 3 key decisions" |
| `Opportunity` | + | A positive trajectory signal — capitalize on it | "Trading system backtest showing 70% win rate" |
| `Strength` | + | An enduring capacity revealed — leverage it | "I structure complex systems fast" |
| `Directive` | − | A corrective imperative — what needs fixing | "Stop skipping morning workouts" |
| `Risk` | − | A negative trajectory signal — what's breaking | "Financial burn rate too high for Q3" |

#### Properties

| Property | Type | Options / Notes |
|----------|------|-----------------|
| `Name` | title | — |
| `Date` | date | When this insight was synthesized |
| `Type` | select | Note, Opportunity, Strength, Directive, Risk |
| `Polarity` | select | +, −, neutral (derived from Type, explicit for filtering) |
| `Source Logs` | relation → Logbook (multi) | Which Logbook entries this synthesized from |
| `Priority` | select | Critical, High, Medium, Low |
| `Status` | select | raw, annotated, synthesized, applied |
| `Capture Method` | select | manual, web_clipper, api_ingest, voice_memo, import |
| `Source URL` | url | For web clips |
| `Spawns` | relation → Action (multi) | For Directives — which corrective Projects/Tasks this spawned |
| `Condenses Into` | relation → Profile (multi) | Which Profile traits this condensed into (long-term) |
| `Revises` | relation → Vision (multi) | Which Vision entries this reaffirms or revises (long-term) |

---

### DB 5: Profile (Layer B)

**Purpose:** The cumulative state mirror. The "RPG status report" across all dimensions. Bridge between history (Layer B) and the pull (Layer A) — shows the GAP.

#### Entry-types (discriminated by `Type` property)

| Entry-type | What it holds | Example |
|------------|---------------|---------|
| `Trait` | An enduring characteristic condensed from long-term trends | "Discipline: High" |
| `Metric` | A measurable indicator with current value + trend | "Net worth: ₹X (↑12% YoY)" |
| `Capacity` | A skill/capability level | "Systems thinking: Lvl 4" |
| `Asset` | A foundational asset built | "LifeOS itself" |

#### Properties

| Property | Type | Options / Notes |
|----------|------|-----------------|
| `Name` | title | — |
| `Type` | select | Trait, Metric, Capacity, Asset |
| `Category` | select | Health, Financial, Relational, Cognitive, Spiritual, Execution, Content, Strategic |
| `Current Value` | rich_text | The current state (string to allow "High", "Lvl 4", "₹X") |
| `Target Value` | rich_text | The ideal-future target (for gap calculation) |
| `Trend` | select | ↑, ↓, → |
| `Unit` | select | count, percentage, hours, minutes, rupees, dollars, level, rating |
| `Frequency` | select | Daily, Weekly, Monthly, Quarterly, Annual |
| `Last Updated` | date | — |
| `Source Synthesis` | relation → Synthesis (multi) | Which Synthesis entries condensed into this |
| `Closes Gap For` | relation → Vision (multi) | Which Vision entries this Profile shows the gap for |
| `Informs Goal` | relation → Compass (multi) | Which Compass goals this Profile informs |

---

### DB 6: Action (Layer C)

**Purpose:** The actionable hierarchy. What to do + how. Daily-use, high-churn.

#### Entry-types (discriminated by `Type` property)

| Entry-type | What it holds | Example |
|------------|---------------|---------|
| `Project` | A multi-step deliverable aligned with a Goal | "Implement LifeOS v1.0 DB schema" |
| `Task` | An atomic unit of work | "Write Vision DB entry-types spec" |
| `Campaign` | A coordinated multi-content effort | "LifeOS launch campaign" |
| `Content` | A single content piece | "LifeOS architecture blog post" |

#### Properties

| Property | Type | Options / Notes |
|----------|------|-----------------|
| `Name` | title | — |
| `Type` | select | Project, Task, Campaign, Content |
| `Status` | status | Future, Ideation, Paused, Active, Done, Cancelled |
| `Priority` | select | Critical, High, Medium, Low |
| `Progress` | number | 0-100 (%) |
| `Start Date` | date | — |
| `End Date` | date | — |
| `Parent Project` | relation → Action (self) | For Task → Project hierarchy |
| `Parent Goal` | relation → Compass | Which Goal this Action traces to |
| `Spawned By` | relation → Synthesis | Which Directive spawned this Action (if applicable) |
| `Assigned To` | relation → Context (Person) | Who is responsible |
| `Involves` | relation → Context (multi) | Which People/Communities are involved |
| `Blocked By` | relation → Action (self) | Dependency |
| `Generates Logs` | relation → Logbook (multi) | Which Logbook entries executing this generated |
| `Constrains Value` | relation → Vision (multi) | Which Values/Principles this Action must not violate |

---

### DB 7: Context (Layer C)

**Purpose:** The environment. Who/what is around the user. Persistent (changes slowly).

#### Entry-types (discriminated by `Type` property)

| Entry-type | What it holds | Example |
|------------|---------------|---------|
| `Person` | A person (rich CRM) | "Ishaan — Mentor, Lvl 5 Orange" |
| `Community` | A group/community | "HoloOS research collective" |
| `Organization` | An organization | "Z.ai" |
| `Financial-Account` | A financial account | "HDFC Savings" |
| `Place` | A physical location | "Noida office" |

#### Properties (common to all entry-types)

| Property | Type | Options / Notes |
|----------|------|-----------------|
| `Name` | title | — |
| `Type` | select | Person, Community, Organization, Financial-Account, Place |
| `Status` | select | Active, Inactive, Archived |

#### Person-specific properties (only populated for `Type=Person`)

| Property | Type | Options |
|----------|------|---------|
| `Aspirational Drive` | select | Security & Stability, Connection & Belonging, Status & Recognition, Mastery & Impact, Growth & Understanding |
| `Developmental Altitude` | select | LVL 3 Red, LVL 4 Amber, LVL 5 Orange, LVL 6 Green, LVL 7 Turquoise |
| `Networking Profile` | select | Key Ally, Active Collaborator, Mentor/Advisor, Protégé/Mentee, Peer/Sounding Board, Inactive, Archive |
| `Relationship Status` | select | Family Member, Mentor, Close Friend, Close Acquaintance, Coworker, Acquaintance |
| `Desired Trajectory` | select | Deepen, Maintain, Activate, Graceful Exit, Inactive |
| `Value Exchange Balance` | select | I am in Credit, Balanced, I am in Debt |
| `Last Interaction Sentiment` | select | Positive, Neutral, Tense, Negative |
| `City` | select | (20+ city options — port from legacy People DB) |
| `Timezone` | select | IST, EST, PST, GMT, GST |
| `Core Shadow` | select | Fear of Insignificance, Fear of Rejection, Fear of Chaos/Uncertainty, Fear of Powerlessness/Domination |
| `Engagement Blueprint` | rich_text | — |
| `Key Personal Intel` | rich_text | — |
| `Professional Domain` | rich_text | — |
| `Influence Toolkit` | multi_select | (top 15 from legacy 65 options — curated) |

#### Community-specific properties (only for `Type=Community`)

| Property | Type | Options |
|----------|------|---------|
| `Community Type` | select | Professional, Personal, Spiritual, Creative, Investment, Mastermind |
| `Strategic Value` | select | Core, Supporting, Peripheral |
| `Covenant` | rich_text | Shared purpose |

#### Financial-Account-specific properties (only for `Type=Financial-Account`)

| Property | Type | Options |
|----------|------|---------|
| `Account Type` | select | Checking, Savings, Investment, Credit, Crypto |
| `Balance` | number | Current balance |
| `Institution` | select | (port from legacy) |

#### Relations OUT (all entry-types)

| Relation | Target | Cardinality | Semantic hint |
|----------|--------|-------------|---------------|
| `Involved In` | → Action (multi) | many-to-many | Person/Community is involved in these Projects |
| `Subject Of` | → Logbook (multi) | one-to-many | Person is subject of Relational logs; Account is subject of Financial logs |
| `Referenced In` | → Synthesis (multi) | one-to-many | Person/Account referenced in these Synthesis entries |

---

## 3. Relation Map (Inter-DB + Intra-DB)

### 3.1 Inter-DB relations (12 total)

| # | From DB | Property | To DB | Cardinality | Flow | Semantic hint |
|---|---------|----------|-------|-------------|------|---------------|
| 1 | Vision | `Decomposes Into` | Compass | one-to-many | Pull | A Vision-Statement decomposes into time-bound Goals |
| 2 | Vision | `Constrains` | Action | one-to-many | Pull | Values/Principles constrain which Actions are valid |
| 3 | Compass | `Derives From` | Vision | many-to-one | Pull (reverse) | Each Goal traces back to a Vision entry |
| 4 | Compass | `Serves Value` | Vision | many-to-many | Pull | Which Values this Goal serves |
| 5 | Compass | `Decomposes Into` | Action | one-to-many | Pull | Each Goal decomposes into Projects |
| 6 | Compass | `Measured By` | Profile | many-to-many | Feedback | Which Profile metrics track this Goal's progress |
| 7 | Action | `Parent Goal` | Compass | many-to-one | Pull (reverse) | Which Goal this Action traces to |
| 8 | Action | `Spawned By` | Synthesis | many-to-one | Ground (reverse) | Which Directive spawned this Action |
| 9 | Action | `Generates Logs` | Logbook | one-to-many | Ground | Executing this Action generated these Logs |
| 10 | Action | `Involves` / `Assigned To` | Context | many-to-many | — | Person/Community involved in this Action |
| 11 | Logbook | `Synthesized Into` | Synthesis | many-to-many | Ground | This Log synthesized into these Synthesis entries |
| 12 | Synthesis | `Condenses Into` | Profile | many-to-many | Ground | Long-term insights condense into Profile traits |
| 13 | Synthesis | `Revises` | Vision | many-to-many | Feedback | Long-term insights reaffirm/revise Vision |
| 14 | Synthesis | `Spawns` | Action | one-to-many | Ground (reverse) | Directives spawn corrective Actions |
| 15 | Profile | `Closes Gap For` | Vision | many-to-many | Feedback | Shows the gap between current state and ideal-future |
| 16 | Profile | `Informs Goal` | Compass | many-to-many | Feedback | Profile informs which Goals are needed |
| 17 | Logbook | `Source Project` / `Source Task` | Action | many-to-one | Ground (reverse) | This Log belongs to this Project/Task |
| 18 | Logbook | `Subject Person` / `Subject Account` | Context | many-to-one | — | This Log is about this Person/Account |

### 3.2 Intra-DB relations (4 total)

| # | DB | Property | Target | Cardinality | Purpose |
|---|-----|----------|--------|-------------|---------|
| 1 | Vision | `Pillar` | Vision (self) | many-to-one | Groups related Values/Principles into a pillar |
| 2 | Compass | `Parent Goal` | Compass (self) | many-to-one | Annual-Goal → Quarterly-Goal hierarchy |
| 3 | Action | `Parent Project` | Action (self) | many-to-one | Task → Project hierarchy |
| 4 | Action | `Blocked By` | Action (self) | many-to-many | Dependency tracking |

---

## 4. The 3 Flows + The Cycle (Formal Specification)

### Flow 1 — Teleological Pull (Layer A → Layer C, downward)

**Direction:** Vision → Compass → Action

**Trigger:** User sets or updates a Vision entry → Compass goals update to reflect the new trajectory → Action projects align to the new goals.

**Path:**
```
Vision (Vision-Statement)
  → [Decomposes Into] →
Compass (Annual-Goal → Quarterly-Goal)
  → [Decomposes Into] →
Action (Project → Task)
```

**What flows:** The teleological pull — the ideal-future shape propagates downward through time-bound goals into daily actions.

**Constraint:** Vision also `Constrains` Action directly (Values/Principles constrain which Actions are valid, even if the Action doesn't trace to a specific Goal).

---

### Flow 2 — Ground-Truth (Layer C → Layer B, upward)

**Direction:** Action → Logbook → Synthesis → Profile

**Trigger:** User executes an Action → generates a Logbook entry → Synthesis processes the log → long-term insights condense into Profile.

**Path:**
```
Action (executed)
  → [Generates Logs] →
Logbook (Activity/Diet/Financial/etc.)
  → [Synthesized Into] →
Synthesis (Note → Opportunity/Directive)
  → [Condenses Into] →
Profile (Trait/Metric/Capacity/Asset)
```

**What flows:** Objective ground-reality data — actions produce logs, logs synthesize into insights, insights condense into the cumulative state.

**Polar structure:** Synthesis has a (+) pole (Opportunities/Strengths — leverage) and a (−) pole (Directives/Risks — correct). Both receive from the SAME logs.

---

### Flow 3 — Feedback (Layer B → Layer A, loop)

**Direction:** Profile → Vision + Compass

**Trigger:** Profile shows the GAP between current state and ideal-future → user revises Vision/Goals based on reality.

**Path:**
```
Profile (current state)
  → [Closes Gap For] →
Vision (shows the gap → user revises)
  → [Informs Goal] →
Compass (trajectory revised based on Profile)
```

**What flows:** The gap signal — Profile shows where the user actually is, which informs the trajectory simulation in Vision/Compass.

**Effect:** The pull becomes REALISTIC (not fantasy). The user revises Vision/Goals based on actual progress, not wishful thinking.

---

### The Cycle (causal amplification)

```
Vision → Compass → Action → Logbook → Synthesis → Profile → Vision
  (pull)  (decompose)  (execute)  (capture)  (synthesize)  (condense)  (feedback)
```

**Each loop amplifies:**
- The pull becomes more precise (Profile informs Vision)
- The actions become more aligned (Synthesis corrects Action)
- The history becomes richer (Action generates more Logbook)
- The self-understanding deepens (Synthesis condenses into Profile)

---

## 5. Migration Mapping (Current 5-DB → New 7-DB)

### 5.1 Current State (matrix) → Profile + (Logbook if any)

| Current entry-type | → New DB | → New entry-type | Notes |
|--------------------|----------|-----------------|-------|
| Pattern | Profile | Trait | Recurring behavior → enduring trait |
| Threshold | Profile | Trait | Boundary → trait |
| Foundation | Profile | Asset | Load-bearing structure → asset |

### 5.2 Current Possibility (potentiator) → Logbook + Vision + Synthesis

| Current entry-type | → New DB | → New entry-type | Notes |
|--------------------|----------|-----------------|-------|
| Activity | Logbook | Activity | Direct map |
| Diet | Logbook | Diet | Direct map |
| Financial | Logbook | Financial | Direct map |
| Subjective | Logbook | Subjective | Direct map |
| Relational | Logbook | Relational | Direct map |
| Systemic | Logbook | Systemic | Direct map |
| Goal | Compass | Annual-Goal or Quarterly-Goal | Determine timeframe from entry |
| Vision | Vision | Vision-Statement | Direct map |
| Aspiration | Vision | Identity-Statement or Vision-Statement | Determine from content |
| Observation | Synthesis | Note | Raw observation → raw synthesis |

### 5.3 Current Process (nexus) → Logbook + Synthesis + Profile + Action

| Current entry-type | → New DB | → New entry-type | Notes |
|--------------------|----------|-----------------|-------|
| Note | Logbook | Note (new entry-type) OR Synthesis | TBD — Note is capture AND synthesis. Default: Synthesis (Note) |
| Knowledge-Category | Synthesis | Note (with sub-type) | Taxonomy node → synthesis note |
| Knowledge-Atom | Synthesis | Note (with sub-type) | Discrete knowledge → synthesis note |
| Opportunity | Synthesis | Opportunity | Direct map |
| Insight | Synthesis | Strength or Note | Determine from content |
| Reflection | Synthesis | Note | Direct map |
| Integration | Synthesis | Note | Direct map |
| Pattern | Profile | Trait | Pattern recognition → trait |
| Risk | Synthesis | Risk | Direct map |
| Directive | Synthesis | Directive | Direct map |
| Decision | Synthesis | Note OR Action (Task) | If actionable → Action; if reflective → Synthesis |
| Crisis | Synthesis | Risk | Direct map |
| Transformation-Event | Synthesis | Note OR Vision (Identity-Statement update) | If identity-shifting → Vision; else Synthesis |

### 5.4 Current Identity (significator) → Vision + Profile

| Current entry-type | → New DB | → New entry-type | Notes |
|--------------------|----------|-----------------|-------|
| Purpose | Vision | Purpose | Direct map |
| Value | Vision | Value | Direct map |
| Principle | Vision | Principle | Direct map |
| Identity-Statement | Vision | Identity-Statement | Direct map |
| Pillar | Vision | Value or Principle | Pillar → Vision (Pillar self-relation) |
| Strategic-Ideal | Vision | Vision-Statement | Direct map |
| (Stats was here) | Profile | (entire DB) | Stats → Profile DB |
| Holon Type | Profile | Trait | Type → trait |
| Valence Signature | Profile | Trait (rich_text) | Signature → trait |
| Life-Era (was Stage) | Profile | Trait | Era → trait |

### 5.5 Current World (greatway) → Compass + Action + Context

| Current entry-type | → New DB | → New entry-type | Notes |
|--------------------|----------|-----------------|-------|
| Annual Goal | Compass | Annual-Goal | Direct map |
| Quarterly Goal | Compass | Quarterly-Goal | Direct map |
| Project | Action | Project | Direct map |
| Task | Action | Task | Direct map |
| System | Action | Project OR Profile (Asset) | If operational → Action; if foundational → Profile |
| Resource | Action | Project OR Profile (Asset) | Same logic |
| Sprint | Action | Task or Project | Determine from scope |
| Milestone | Compass | Milestone | Direct map |
| Budget | Action | Project OR Logbook (Financial) | If budget entry → Action; if transaction → Logbook |
| Campaign | Action | Campaign | Direct map |
| Content | Action | Content | Direct map |
| Person | Context | Person | Direct map (port 14 properties) |
| Group | Context | Community | Direct map |
| Community | Context | Community | Direct map |
| Organization | Context | Organization | Direct map |
| Network | Context | Community or Organization | Determine from content |
| Movement | Context | Community | Direct map |
| Place | Context | Place | Direct map |

### 5.6 Migration principles

1. **No data loss.** Every entry-type in the current 5-DB has a clear destination in the new 7-DB.
2. **Properties follow.** Each entry's properties migrate with it (e.g. Person's 14 CRM properties → Context.Person).
3. **Relations re-point.** After migration, relations are re-pointed to the new DB locations (e.g. World.Person → Action.Project relation becomes Context.Person → Action.Project).
4. **Build fresh, port gradually.** Recommended: build the 7 new DBs fresh in Notion, then port entries from the 5 old DBs in batches (Logbook first, then Context, then Action, then Synthesis, then Compass, then Vision, then Profile last).

---

## 6. Implementation Order

Build the 7 DBs in this order (each depends on the previous):

| Order | DB | Why this order |
|-------|-----|----------------|
| 1 | **Context** | No dependencies. People/Accounts are referenced by everything else. |
| 2 | **Logbook** | Depends on Context (Subject Person/Account). Daily-use, so build early to start capturing. |
| 3 | **Vision** | No dependencies on other DBs (timeless). Build before Compass so Compass can derive from it. |
| 4 | **Compass** | Depends on Vision (Derives From). |
| 5 | **Action** | Depends on Compass (Parent Goal) + Context (Involves) + Synthesis (Spawned By — but Synthesis comes later, so this relation is added after Synthesis). |
| 6 | **Synthesis** | Depends on Logbook (Source Logs) + Action (Spawns — but Action came before, so this is fine) + Vision (Revises) + Profile (Condenses Into — but Profile comes later, so this relation is added after Profile). |
| 7 | **Profile** | Depends on Synthesis (Source Synthesis) + Vision (Closes Gap For) + Compass (Informs Goal). Built last because it's the condensation of everything. |

**Total build time estimate:** 2-3 hours in Notion (creating 7 DBs + ~80 properties + ~19 relations). Migration of existing data: 4-8 hours depending on volume.

---

*Formal spec v4. Single source of truth for the LifeOS 7-DB structure. Ready for implementation.*
