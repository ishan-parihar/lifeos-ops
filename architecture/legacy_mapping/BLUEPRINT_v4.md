# LifeOS Blueprint v4 — The Notion System Structure
# =============================================================================
# A DB-schema-level implementation plan for the LifeOS Consciousness-Prosthetic.
#
# This document answers:
#   1. How many DBs? What are they named?
#   2. What entry-types does each DB hold?
#   3. How do the DBs connect (relations)?
#   4. How does the teleological pull flow through them?
#   5. How does the system keep tabs on history + current trajectory/trends?
#   6. How does the system tell the user what to do and how?
#
# Design principle: the DBs must serve 3 simultaneous functions:
#   (A) ARTICULATE + SIMULATE the teleological pull toward the ideal-future
#   (B) KEEP TABS on history + current-trajectory/trends
#   (C) TELL the user what to do and how
#
# This is a BLUEPRINT, not a final spec. It's meant to be concrete enough to
# build in Notion, but flexible enough to refine.

---

## 1. The 3 Functional Layers (what the system does)

Before naming DBs, let's establish what the system must DO. Every DB serves one or more of these 3 functions:

### Layer A — The Teleological Pull (the ideal-future attractor)
**Function:** Articulate and simulate the drive toward the ideal-future.

This layer holds:
- The articulation of the ideal-future itself (what we're pulled toward)
- The decomposition of that ideal into time-bound trajectories (how the pull manifests across time)
- The alignment constraints (what the trajectory must not violate)

**Question it answers:** "Where am I being pulled? What's the ideal trajectory?"

### Layer B — The Historical Record + Current Trajectory (the ground-truth)
**Function:** Keep tabs on history + current-trajectory/trends.

This layer holds:
- Objective logs of what happened (the raw ground-reality data)
- Synthesized insights from those logs (what the data means)
- The current state profile (the cumulative traits/patterns)

**Question it answers:** "Where have I been? Where am I now? What are the trends?"

### Layer C — The Action Interface (what to do + how)
**Function:** Tell the user what to do and how.

This layer holds:
- The actionable hierarchy (goals → projects → tasks)
- The corrective directives (what to fix)
- The leverage opportunities (what to double down on)

**Question it answers:** "What do I do today? How do I do it?"

### The 3 layers INTERLOCK — they're not sequential

Layer A pulls Layer C (the ideal-future shapes the actions).
Layer B feeds Layer A (history/trends inform the trajectory simulation).
Layer B feeds Layer C (current state determines what actions are needed).
Layer C generates Layer B (actions become logs → history).

This interlock is the system. The DBs are the structure that holds it.

---

## 2. The DB Structure (7 DBs)

I propose **7 DBs**, organized by the 3 layers. Each DB has a single clear purpose.

```
LAYER A — TELEOLOGICAL PULL (2 DBs)
├── 1. Vision       — the ideal-future articulation
└── 2. Compass      — the time-bound trajectory decomposition

LAYER B — HISTORICAL RECORD + CURRENT TRAJECTORY (3 DBs)
├── 3. Logbook      — the 6 objective logs (one DB, 6 entry-types)
├── 4. Synthesis    — insights synthesized from logs
└── 5. Profile      — the cumulative state/trait mirror (Stats)

LAYER C — ACTION INTERFACE (2 DBs)
├── 6. Action       — the actionable hierarchy (goals → projects → tasks)
└── 7. Context      — the environment (people, communities, accounts)
```

**Why 7 (not 5, not 22)?**

- **Not 5** (the current structure): the current 5-DB collapses Layer A + Layer C into one DB (World holds both Vision/Values AND Projects/Tasks/People). This conflation is why the system feels muddled — the teleological pull and the action interface need different structures.
- **Not 22** (the legacy structure): the legacy 22-DB split Layer B into 6 separate log DBs + 3 separate synthesis DBs. The 6 logs serve ONE purpose (objective capture) and should be ONE DB with 6 entry-types. The 3 synthesis DBs (Opportunities/Directives/Notes) serve ONE purpose (insight synthesis) and should be ONE DB with 3 entry-types.
- **7 is the minimum** that separates the 3 functional layers while keeping each DB's purpose clear.

---

## 3. The 7 DBs — Detailed Structure

### DB 1: Vision (Layer A — Teleological Pull)
**Purpose:** Articulate the ideal-future. The teleological attractor.

| Entry-type | What it holds | Example |
|------------|---------------|---------|
| `Purpose` | The deepest "why" — the user's core reason for being | "To become the fullest expression of consciousness I'm capable of" |
| `Value` | An enduring commitment — what the user holds as non-negotiable | "Integrity over convenience" |
| `Principle` | An operating rule — how the user makes decisions | "Default to the option that compounds" |
| `Vision-Statement` | A time-bound articulation of the ideal-future | "By 2030, I am a sovereign consciousness-prosthetic architect" |
| `Identity-Statement` | Who the user is becoming | "I am a person who builds systems that outlive me" |

**Key properties:** Name, Description, Type (the entry-type discriminator), Source, Last Reviewed, Status (Draft/Active/Evolving/Archived)

**Relations:**
- `Vision → Compass` (one-to-many): a Vision-Statement decomposes into Compass entries (Annual-Goals, Quarterly-Goals)
- `Vision → Action` (one-to-many): a Value/Principle constrains Action entries (Projects must align with a Value)

**Why this is a separate DB:** The teleological pull needs its own space. If Vision/Values are mixed with Projects/Tasks, the pull gets buried in operational noise.

---

### DB 2: Compass (Layer A — Teleological Pull)
**Purpose:** The time-bound trajectory decomposition. How the ideal-future pulls across time.

| Entry-type | What it holds | Example |
|------------|---------------|---------|
| `Annual-Goal` | A yearly target derived from Vision | "2026: Ship LifeOS v1.0" |
| `Quarterly-Goal` | A quarterly decomposition of an Annual-Goal | "Q3 2026: Complete LifeOS blueprint + implement" |
| `Milestone` | A significant checkpoint (not time-bound, but event-bound) | "LifeOS first full synthesis cycle complete" |

**Key properties:** Name, Timeframe (Year/Quarter), Status, Progress, Target, Start Date, End Date, Parent (the Vision entry it derives from), Pillar (the Value it serves)

**Relations:**
- `Compass → Vision` (many-to-one): each goal traces back to a Vision entry
- `Compass → Action` (one-to-many): each goal decomposes into Action entries (Projects)
- `Compass → Profile` (one-to-many): each goal has progress metrics tracked in Profile

**Why this is a separate DB from Vision:** Vision is timeless (the articulation). Compass is temporal (the decomposition across time). Mixing them conflates the eternal with the time-bound.

---

### DB 3: Logbook (Layer B — Historical Record)
**Purpose:** Objective ground-reality capture. ONE DB, 6 entry-types (the 6 logs).

| Entry-type | What it captures | Channel |
|------------|------------------|---------|
| `Activity` | Physical activity + time use | Body |
| `Diet` | Food + nutrition | Body |
| `Financial` | Transactions + money flow | Resource |
| `Subjective` | Subjective experience + learning + spirituality | Mind |
| `Relational` | Interactions + relational dynamics | Relational |
| `Systemic` | Process observations + workflow notes | Mind |

**Key properties:** Name, Date, Entry-Type (the discriminator), Content/Amount, related Project/Task/Person (if applicable)

**Relations:**
- `Logbook → Action` (many-to-one): a log entry can belong to a Project/Task
- `Logbook → Context` (many-to-one): a Relational log belongs to a Person; a Financial log belongs to a Financial-Account
- `Logbook → Synthesis` (many-to-one): a log entry synthesizes into an Insight (via the synthesis pipeline)

**Why ONE DB (not 6):** The 6 logs serve ONE purpose — objective capture. They differ only in channel (Body/Mind/Relational/Resource), not in function. Splitting them into 6 DBs created 6 places to log, which is friction. ONE DB with 6 entry-types + a Date filter gives the same granularity with 1/6 the friction.

---

### DB 4: Synthesis (Layer B — Historical Record)
**Purpose:** Where logs become insights. The synthesis pipeline.

| Entry-type | What it holds | Pole |
|------------|---------------|------|
| `Note` | Raw synthesis capture (meeting notes, web clips, voice memos, raw reflections) | Neutral |
| `Opportunity` | A positive trajectory signal — what's working, capitalize on it | (+) |
| `Strength` | An enduring capacity revealed by the logs — leverage it | (+) |
| `Directive` | A corrective imperative — what needs fixing | (−) |
| `Risk` | A negative trajectory signal — what's breaking | (−) |

**Key properties:** Name, Date, Type (the discriminator), Polarity (+/−/neutral), Source-Logs (which Logbook entries it synthesized from), Priority, Status (raw/annotated/synthesized/applied), Spawned-Action (if a Directive spawned a Project/Task)

**Relations:**
- `Synthesis ← Logbook` (many-to-one): each Synthesis entry is fed by Logbook entries
- `Synthesis → Vision` (many-to-one): long-term insights reaffirm/revise Values/Vision
- `Synthesis → Profile` (many-to-one): long-term insights condense into Profile traits
- `Synthesis → Action` (one-to-many): Directives spawn corrective Projects/Tasks

**Why ONE DB (not 3):** Opportunities/Strengths and Directives/Risks are polar opposites receiving from the SAME logs. They're a dialectical pair, not independent DBs. Notes are the raw input to both poles. Keeping them in ONE DB with a Polarity property lets the user see the full synthesis picture in one view, and lets the AI agent understand the polar structure directly.

---

### DB 5: Profile (Layer B — Historical Record)
**Purpose:** The cumulative state mirror. The "RPG status report" across all dimensions.

| Entry-type | What it holds | Example |
|------------|---------------|---------|
| `Trait` | An enduring characteristic that has condensed from long-term trends | "Discipline: High" |
| `Metric` | A measurable indicator with a current value + trend | "Net worth: ₹X (↑12% YoY)" |
| `Capacity` | A skill/capability level | "Systems thinking: Lvl 4" |
| `Asset` | A foundational asset the user has built | "LifeOS itself" |

**Key properties:** Name, Category (Health/Financial/Relational/Cognitive/Spiritual/Execution/Content), Current-Value, Trend (↑/↓/→), Unit, Frequency, Last-Updated, Source-Synthesis (which Synthesis entries condensed into this)

**Relations:**
- `Profile ← Synthesis` (many-to-one): traits/metrics condense from Synthesis entries
- `Profile ← Logbook` (indirect, via Synthesis): the raw data ultimately feeds Profile
- `Profile → Vision` (one-to-many): the current Profile shows the gap between current state and ideal-future
- `Profile → Compass` (one-to-many): the Profile informs which goals are needed

**Why this is a separate DB:** Profile is the BRIDGE between Layer B (history) and Layer A (teleological pull). It shows the user where they ARE (cumulative state) so they can see the gap to where they're PULLED (Vision). Without Profile, the user has logs + synthesis but no consolidated self-understanding.

---

### DB 6: Action (Layer C — Action Interface)
**Purpose:** The actionable hierarchy. What to do + how.

| Entry-type | What it holds | Example |
|------------|---------------|---------|
| `Project` | A multi-step deliverable aligned with a Goal | "Implement LifeOS v1.0 DB schema" |
| `Task` | An atomic unit of work | "Write Vision DB entry-types spec" |
| `Campaign` | A coordinated multi-content effort | "LifeOS launch campaign" |
| `Content` | A single content piece | "LifeOS architecture blog post" |

**Key properties:** Name, Status, Priority, Progress, Start Date, End Date, Parent-Project (intra-DB hierarchy), Parent-Goal (→ Compass), Assigned-Person (→ Context), Blocked-By (intra-DB)

**Relations:**
- `Action ← Compass` (many-to-one): each Project traces to a Quarterly/Annual Goal
- `Action ← Synthesis` (many-to-one): Directives spawn corrective Projects/Tasks
- `Action → Logbook` (one-to-many): executing a Task generates Logbook entries
- `Action → Context` (many-to-one): a Project involves People; a Task is assigned to a Person
- `Action → Action` (intra-DB hierarchy): Project → Task → Sub-task

**Why this is a separate DB:** Action is the execution layer — it changes daily, has its own hierarchy (Project → Task), and connects to both the pull (Compass) and the ground (Logbook). Mixing it with Vision/Compass would bury daily actions under timeless articulations.

---

### DB 7: Context (Layer C — Action Interface)
**Purpose:** The environment. Who/what is around the user.

| Entry-type | What it holds | Example |
|------------|---------------|---------|
| `Person` | A person (rich CRM — 14 curated properties from legacy) | "Ishaan — Mentor, Lvl 5 Orange" |
| `Community` | A group/community | "HoloOS research collective" |
| `Organization` | An organization | "Z.ai" |
| `Financial-Account` | A financial account | "HDFC Savings" |
| `Place` | A physical location | "Noida office" |

**Key properties:** Name, Type (the discriminator), + type-specific properties (Person has Aspirational-Drive, Developmental-Altitude, Networking-Profile, etc.; Community has Type, Strategic-Value, Covenant; Financial-Account has Balance, Institution)

**Relations:**
- `Context → Action` (many-to-many): People are involved in Projects; Tasks assigned to People
- `Context → Logbook` (one-to-many): a Person is the subject of Relational logs; an Account owns Financial logs
- `Context → Synthesis` (one-to-many): relational/financial insights reference People/Accounts

**Why this is a separate DB:** Context is persistent (People/Accounts don't change daily) but is referenced by everything else. It needs its own space so it can be curated independently of the actions/logs that reference it.

---

## 4. How the 7 DBs Connect (the relational topology)

```
LAYER A — TELEOLOGICAL PULL
                    ┌─────────────┐
                    │   VISION    │  (Purpose/Values/Principles/Vision/Identity)
                    │   (DB 1)    │
                    └──────┬──────┘
                           │ decomposes into
                           ▼
                    ┌─────────────┐
                    │   COMPASS   │  (Annual-Goals/Quarterly-Goals/Milestones)
                    │   (DB 2)    │
                    └──────┬──────┘
                           │ pulls
                           ▼
LAYER C — ACTION    ┌─────────────┐         ┌─────────────┐
                    │   ACTION    │◄────────│   CONTEXT   │
                    │   (DB 6)    │ involves│   (DB 7)    │
                    │ Projects/   ├────────►│ People/     │
                    │ Tasks/      │ assigned│ Comm/       │
                    │ Campaigns/  │         │ Accts/      │
                    │ Content     │         │ Places      │
                    └──────┬──────┘         └──────┬──────┘
                           │ generates             │
                           │ (executing actions    │
                           │  produces logs)       │ (people/accounts
                           ▼                       │  are log subjects)
LAYER B — HISTORICAL┌─────────────┐         ┌─────────────┐
                    │   LOGBOOK   │◄────────│             │
                    │   (DB 3)    │ subject │             │
                    │ 6 logs:     │         │             │
                    │ Activity/   │         │             │
                    │ Diet/       │         │             │
                    │ Financial/  │         │             │
                    │ Subjective/ │         │             │
                    │ Relational/ │         │             │
                    │ Systemic    │         │             │
                    └──────┬──────┘         └─────────────┘
                           │ synthesizes into
                           ▼
                    ┌─────────────┐
                    │  SYNTHESIS  │  (Notes/Opportunities/Strengths/
                    │   (DB 4)    │   Directives/Risks)
                    │ Polar: +/−  │
                    └──────┬──────┘
                           │ condenses into
                           ▼
                    ┌─────────────┐
                    │   PROFILE   │  (Traits/Metrics/Capacities/Assets)
                    │   (DB 5)    │  (the cumulative state mirror)
                    └──────┬──────┘
                           │
                           │ shows the GAP between current state and ideal-future
                           │ (feeds back to Layer A — informs trajectory simulation)
                           ▼
                    (back to VISION — the teleological pull is informed
                     by the gap Profile reveals)
```

### The 3 flows through the DBs

**Flow 1 — The Teleological Pull (Layer A → Layer C):**
Vision → Compass → Action. The ideal-future articulates → decomposes into time-bound goals → pulls the daily actions. This is the DOWNWARD pull.

**Flow 2 — The Ground-Truth Flow (Layer C → Layer B):**
Action → Logbook → Synthesis → Profile. Executing actions generates logs → logs synthesize into insights → insights condense into the cumulative profile. This is the UPWARD condensation.

**Flow 3 — The Feedback Loop (Layer B → Layer A):**
Profile → Vision/Compass. The Profile shows the gap between current state and ideal-future. This gap INFORMS the trajectory simulation — the user revises Vision/Goals based on where they actually are. This is the feedback that keeps the pull REALISTIC (not fantasy).

**The 3 flows interlock into a cycle:**
```
Vision → Compass → Action → Logbook → Synthesis → Profile → Vision
  (pull)  (decompose)  (execute)  (capture)  (synthesize)  (condense)  (feedback)
```

This cycle IS the causal amplification. Each loop:
- The pull becomes more precise (Profile informs Vision)
- The actions become more aligned (Synthesis corrects Action)
- The history becomes richer (Action generates more Logbook)
- The self-understanding deepens (Synthesis condenses into Profile)

---

## 5. The Teleological Pull — How it Works in Practice

The user asked: "how does LifeOS help articulate and simulate the drive and the pull toward the ideal-future?"

### Articulation (static)
The pull is ARTICULATED in Vision (DB 1):
- `Purpose` = the deepest why
- `Values` = the alignment constraints
- `Principles` = the decision rules
- `Vision-Statement` = the time-bound ideal-future
- `Identity-Statement` = who the user is becoming

These are the user's articulation of the attractor. They don't change daily — they evolve as the user's understanding deepens (informed by Profile feedback).

### Simulation (dynamic)
The pull is SIMULATED through the Compass → Action → Profile feedback loop:

1. **Compass decomposes the Vision** into Annual-Goals → Quarterly-Goals. This is the temporal simulation — "if I'm pulled toward X by 2030, what must be true by 2026? By Q3 2026?"

2. **Action translates Goals into Projects/Tasks.** This is the operational simulation — "to achieve Q3 2026 goal, what Projects must I run? What Tasks must I do today?"

3. **Profile measures the GAP.** This is the reality check — "my current Profile shows I'm at Lvl 4 in systems thinking, but my Vision requires Lvl 6. The gap is 2 levels. My trajectory must close that gap."

4. **The gap feeds back to Compass.** The user revises the trajectory — "given my current Profile, the 2030 Vision is achievable if I gain 1 level per year. Q3 2026 goal: reach Lvl 5."

This loop IS the simulation. The pull isn't a static statement — it's a continuously-refined trajectory that adapts to the user's actual progress.

### How the user experiences the pull (the morning view)

When the user opens LifeOS in the morning, they see:

1. **The pull (Layer A):** The current Vision-Statement + this quarter's Compass goals. "Where am I being pulled?"
2. **The gap (Layer B → A bridge):** The Profile traits that are ON TRACK (green) vs. BEHIND (red). "How far am I from the ideal?"
3. **The trends (Layer B):** Recent Synthesis entries (Opportunities + Directives). "What's the data saying?"
4. **The actions (Layer C):** Today's Tasks. "What do I do today?"
5. **The accomplishments (Layer B + C):** Recently completed Projects + recently gained Profile traits. "What have I achieved?"
6. **The capture interface (Layer B entry):** Quick-log buttons for the 6 Logbook channels. "Update anything in 10 seconds."

---

## 6. DB Count Justification (why 7, not 5 or 22)

### Why not 5 (the current structure)?
The current 5-DB structure (State/Possibility/Process/Identity/World) has 3 problems this blueprint fixes:

1. **World conflates Layer A + Layer C.** World holds both Vision/Values (Layer A — the pull) AND Projects/Tasks/People (Layer C — the action). These serve different functions and need different structures. The blueprint separates them into Vision + Compass (Layer A) and Action + Context (Layer C).

2. **No standalone Profile.** The current Stats is buried inside Identity, but Profile is the BRIDGE between history (Layer B) and the pull (Layer A). It needs its own DB so the gap is visible.

3. **Process conflates logs + synthesis.** The current Process holds both Notes (which are raw capture) AND Insights/Decisions (which are synthesis). These are different stages. The blueprint separates them into Logbook (capture) and Synthesis (insights).

### Why not 22 (the legacy structure)?
The legacy 22-DB structure has 2 problems this blueprint fixes:

1. **6 separate log DBs = friction.** The 6 logs serve ONE purpose (objective capture) and differ only in channel. The blueprint merges them into 1 Logbook with 6 entry-types + a Date filter.

2. **3 separate synthesis DBs = fragmentation.** Opportunities/Directives/Notes are a polar pair + raw input. The blueprint merges them into 1 Synthesis DB with a Polarity property.

### Why 7 is the minimum
- Layer A needs 2 DBs (Vision for the timeless articulation, Compass for the temporal decomposition) because timeless + temporal don't mix.
- Layer B needs 3 DBs (Logbook for capture, Synthesis for insights, Profile for cumulative state) because capture + processing + condensation are 3 distinct functions.
- Layer C needs 2 DBs (Action for the hierarchy, Context for the environment) because actions + environment have different lifecycles.
- 2 + 3 + 2 = 7. Fewer would conflate functions; more would fragment them.

---

## 7. Migration Path (from current 5-DB to proposed 7-DB)

This is a high-level migration sketch — not a detailed plan.

| Current DB | → Proposed DB(s) | What moves |
|------------|-----------------|------------|
| State (Matrix) | → Logbook (partial) + Profile (partial) | State's logs (Activity/Diet/etc.) → Logbook; State's Pattern/Threshold → Profile (as Traits/Capacities) |
| Possibility (Potentiator) | → Logbook (partial) + Profile (partial) | Possibility's logs → Logbook; Possibility's Goals/Vision/Aspiration → Vision or Compass |
| Process (Nexus) | → Logbook (partial) + Synthesis | Process's Notes → Logbook (as Note entry-type) or Synthesis (as Note); Process's Insights/Decisions/Opportunities/Directives → Synthesis |
| Identity (Significator) | → Vision + Profile | Identity's Purpose/Values/Principles → Vision; Identity's Stats → Profile |
| World (GreatWay) | → Compass + Action + Context | World's Annual/Quarterly Goals → Compass; World's Projects/Tasks/Campaigns/Content → Action; World's People/Communities/Organizations/Financial-Accounts/Places → Context |

**Migration complexity:** Medium. The current 5 DBs split cleanly along the 3-layer boundary. No data loss — every entry-type has a clear destination.

---

## 8. Open Questions for the User

1. **Is 7 the right DB count?** Too many? Too few? Specifically — should Vision + Compass be ONE DB (with a Timeframe property discriminating timeless vs. temporal)? Or should Logbook + Synthesis be ONE DB (with a Stage property discriminating raw vs. synthesized)?

2. **The Profile DB.** Is this the right name? Is "Profile" the right concept? (Alternatives: "State", "Mirror", "Self", "Status".) Should it hold Traits/Metrics/Capacities/Assets, or is there a different taxonomy?

3. **The Synthesis DB polarity structure.** I have 5 entry-types: Note (neutral), Opportunity/Strength (+), Directive/Risk (−). Should Opportunity and Strength be ONE entry-type (with a sub-type property), or stay separate? Same for Directive/Risk.

4. **The Context DB.** Should Financial-Account live here, or in a separate Financial DB? The legacy structure had a separate Financial-System. Mixing Accounts with People/Communities might feel wrong.

5. **The migration path.** Is the high-level migration sketch (§7) the right approach? Or should the 7-DB be built FRESH and the 5-DB data ported over gradually?

6. **What's missing?** Does this blueprint capture everything you expect of LifeOS? Is there a function that doesn't fit into any of the 7 DBs?

---

*Blueprint v4. A concrete DB-schema-level implementation plan. 7 DBs across 3 functional layers. Awaiting user reaction to the 6 open questions before finalizing.*
