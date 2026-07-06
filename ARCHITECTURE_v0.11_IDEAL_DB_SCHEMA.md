# LifeOS v0.11 — Ideal DB Schema Architecture

> **Document type:** Architecture proposal (design only — no code changes yet)
> **Author:** LifeOS-Architect
> **Date:** 2026-07-06
> **Objective:** Design the ideal 5-DB schema (entry-types + relation properties) that fully absorbs all functions of the legacy 22-DB structure, providing optimal UX for both the Notion Dashboard user and AI agents via MCP/CLI.
> **Companion docs:** [ONTOLOGY.md](ONTOLOGY.md), [AUDIT_v0.10.1_DB_SCHEMA.md](AUDIT_v0.10.1_DB_SCHEMA.md), [AUDIT_v0.10.3_EXECUTION_REPORT.md](AUDIT_v0.10.3_EXECUTION_REPORT.md)

---

## 0. Executive Summary

The legacy LifeOS had **22 databases**: 5 current (State/Possibility/Process/Identity/World) + 11 legacy LifeOS DBs (People, Community, Knowledge Categories, Activity Types, Key Metrics, Lines of Development, Drives, States, Levels, Quadrants, Types) + 5 calendar DBs (Days/Weeks/Months/Quarters/Years) + 1 (Notes Management, merged into Process).

The legacy structure was rich but **fragmented**: relations crossed 22 DBs, making queries slow and AI agents lost. The v0.9.0 consolidation merged the auxiliary DBs into the 5, but the **entry-type taxonomy + relation property set was never redesigned to absorb the lost functions**. Result: the 5-DB structure has the right *shape* but is missing the *granularity* to replace the 22-DB.

This document proposes the **ideal v0.11 schema**: expanded entry-types per DB, a complete intra-DB + inter-DB relation property set, and a mapping showing how every legacy DB function is absorbed. No code changes — this is the design spec for the next implementation sprint.

---

## 1. The Legacy 22-DB Structure — Function Inventory

### 1.1 The 5 current DBs (v0.10.3)

| DB | Role | Current entry-types | Current relations |
|----|------|---------------------|-------------------|
| **State** (matrix) | Current-state organizer | Pattern, Threshold, Foundation (3) | 11 |
| **Possibility** (potentiator) | Latent-state generator | Activity, Subjective, Relational, Systemic, Diet, Financial, Observation, Goal, Vision, Aspiration (10) | 10 |
| **Process** (nexus) | Contact-boundary | Opportunity, Directive, Risk, Insight, Reflection, Integration, Pattern, Note, Knowledge-Category, Knowledge-Atom, Decision, Crisis, Transformation-Event (13) | 16 |
| **Identity** (significator) | Persistent identity | Purpose, Value, Principle, Identity-Statement, Pillar, Strategic-Ideal (6) | 15 |
| **World** (greatway) | Operating environment | Annual Goal, Quarterly Goal, Project, Task, System, Resource, Sprint, Milestone, Budget, Campaign, Content, Person, Group, Community, Organization, Network, Movement, Place (18) | 11 |

### 1.2 The 11 legacy LifeOS DBs (merged in v0.9.0, functions not fully absorbed)

| Legacy DB | Properties | Function | Current absorption status |
|-----------|------------|----------|---------------------------|
| **People** (31 props) | Aspirational Drive, City, Core Shadow, Desired Trajectory, Developmental Altitude, Dominant Power Strategy, Engagement Blueprint, Explanatory Style, Influence Toolkit (65 opts!), Networking Profile, Primary Center of Intelligence, Primary Conflict Style, Professional Domain, Relationship Status, Stability Profile, Temporal Focus, Value Exchange Balance, last_interaction_sentiment, timezone, etc. | Rich CRM for people — drives, shadows, networking, conflict styles, developmental altitude | ⚠ **PARTIAL** — World has `Person` entry-type but NO rich properties. All 31 People properties lost. |
| **Community** (9 props) | Type (Professional/Personal/Spiritual/Creative/Investment/Mastermind), Strategic Value (Core/Supporting/Peripheral), Covenant/Shared Purpose, Cities (rollup), People (relation) | Community/group CRM with strategic value classification | ⚠ **PARTIAL** — World has `Community` entry-type but no Type/Strategic Value properties. |
| **Knowledge Categories** (7 props) | Archive, Priority Topic, Documents DB (relation), Goods & Services Vault (relation), Notes and Meetings (relation), Related Knowledge Categories | Knowledge taxonomy with document linking | ⚠ **PARTIAL** — Process has `Knowledge-Category` entry-type but no document-linking relations. |
| **Activity Types** (7 props) | category (Exercise/Recovery/Nutrition/Work/Mindfulness/Social/Chore/Commute/Entertainment/Learning), Duration, Frequency, Habit, is_health_tracked, target_per_week | Activity taxonomy with health-tracking + frequency targets | ❌ **LOST** — No equivalent. Possibility.Activity entries have no category or frequency target. |
| **Key Metrics** (6 props) | Category, Change (formula), Definition, Frequency, Unit | KPI tracking with units + frequency | ❌ **LOST** — No equivalent anywhere. |
| **Lines of Development** (5 props) | Active, Description, Quadrants (relation), Roles and Designations (relation) | Developmental line tracking (Wilber-style) | ❌ **LOST** — No equivalent. |
| **Drives** (5 props) | Challenges, Description, Formula, Pathologies | The 4 drives (Agency/Communion/Eros/Agape) with challenges + pathologies | ⚠ **PARTIAL** — Universal property `Drive Activation` exists but has no descriptive backing. |
| **States** (5 props) | Benefits, Challenges, Description, Formula | State descriptions (developmental states) | ❌ **LOST** — No equivalent. |
| **Levels** (72 props!) | 60+ relations to psychology/development DBs + Capacities, Energy Ray Center, Healthy Expression, Self, Society | Developmental level assessment — the master index of personal development | ❌ **LOST** — No equivalent. This was the richest DB. |
| **Quadrants** (5 props) | Description, Formula, Types (relation), Lines of Development (relation) | Wilber 4-quadrant framework | ⚠ **PARTIAL** — World has `Quadrant` property (UL/UR/LL/LR) but no descriptive backing. |
| **Types** (8 props) | Challenges, Description, Identification, Language to Influence | Holon type descriptions (Donor/Acceptor/etc.) | ⚠ **PARTIAL** — Identity has `Holon Type` property but no descriptive backing. |

### 1.3 The 5 calendar DBs (time-based organization)

| Legacy DB | Function | Current absorption |
|-----------|----------|-------------------|
| **Days** | Daily logs + time-based entry organization | ⚠ Possibility has daily-log entry-types (Activity/Diet/etc.) but no Date-based grouping |
| **Weeks** | Weekly review + aggregation | ❌ No equivalent |
| **Months** | Monthly review + aggregation | ❌ No equivalent |
| **Quarters** | Quarterly review + OKR tracking | ⚠ World has `Quarterly Goal` entry-type but no Quarter grouping |
| **Years** | Annual review + annual goal tracking | ⚠ World has `Annual Goal` entry-type but no Year grouping |

### 1.4 The function gap summary

The 5-DB structure is missing:
1. **Rich people CRM** (31 properties from People DB)
2. **Activity taxonomy** (category, frequency, health-tracking from Activity Types DB)
3. **KPI/metrics tracking** (Key Metrics DB)
4. **Developmental line tracking** (Lines of Development DB)
5. **Developmental level assessment** (Levels DB — the master index)
6. **Descriptive backing for universal properties** (Drives, States, Quadrants, Types — these were reference DBs)
7. **Time-based grouping** (Days/Weeks/Months/Quarters/Years calendar)
8. **Document linking** (Knowledge Categories → Documents DB relation)

---

## 2. The Ideal v0.11 Architecture — Design Principles

### 2.1 Three-tier entry-type taxonomy

Every DB has 3 tiers of entry-types:

| Tier | Purpose | UX | Example |
|------|---------|-----|---------|
| **Operational** | Daily-use, high-volume, low-curation | Quick capture, auto-typed | Possibility.Activity, World.Task, Process.Note |
| **Structural** | Medium-frequency, curated | Manual creation, relation-rich | State.Pattern, World.Project, Identity.Purpose |
| **Reference** | Low-frequency, high-curation, descriptive backing for universal properties | Manually curated, ontology-anchored | Identity.Archetype, World.Quadrant-Definition |

### 2.2 Relation property design rules

1. **Every relation has a semantic hint** (U-1, implemented v0.10.3)
2. **Intra-DB relations** encode hierarchy (Parent/Sub-item, Blocked By, Refines, Supersedes)
3. **Inter-DB relations** encode the fractal coupling (13 dual_property from HoloOS doc 08.5) + currency flow (Process as the hub)
4. **Reference relations** link operational entries to their ontological backing (e.g. World.Person → Identity.Archetype for their developmental level)
5. **No relation without a purpose** — YAGNI. If a relation has <5% fill after 30 days, delete it (enforced by `fill_rate` tool, v0.10.3)

### 2.3 The "Reference DB" pattern

The legacy Drives/States/Levels/Quadrants/Types DBs were **reference DBs** — they held descriptive backing for universal properties. In the 5-DB structure, these become **reference entry-types** within the appropriate DB:

| Legacy reference DB | → | Ideal location | Entry-type |
|---------------------|---|----------------|------------|
| Drives (4 drives) | → | Identity | `Archetype-Drive` (reference) |
| States (developmental states) | → | State | `State-Definition` (reference) |
| Levels (developmental levels) | → | Identity | `Developmental-Level` (reference) |
| Quadrants (Wilber 4-quadrant) | → | World | `Quadrant-Definition` (reference) |
| Types (5 holon types) | → | Identity | `Type-Definition` (reference) |
| Lines of Development | → | Identity | `Line-of-Development` (reference) |

This keeps the ontological backing IN the 5-DB structure (not in separate DBs) while preserving the descriptive richness.

---

## 3. Ideal Entry-Type Taxonomy per DB

### 3.1 State (matrix) — Current-state organizer

**Current entry-types (3):** Pattern, Threshold, Foundation

**Ideal entry-types (6) — add 3 reference types:**

| Entry-type | Tier | Purpose | Status |
|------------|------|---------|--------|
| `Pattern` | structural | Recurring behavior/practice | ✅ exists |
| `Threshold` | structural | Boundary/limit | ✅ exists |
| `Foundation` | structural | Load-bearing structure | ✅ exists |
| `State-Definition` | reference | Descriptive backing for developmental states (benefits/challenges/formula) — absorbs legacy `States` DB | 🆕 NEW |
| `Practice` | operational | Daily/weekly practice (subcategory of Pattern, with frequency target) — absorbs Activity Types frequency tracking | 🆕 NEW |
| `Inventory` | operational | Current-state inventory (assets, resources, current holdings) | 🆕 NEW |

**Property additions:**
- `Frequency Target` (select: Daily/Weekly/Monthly/Quarterly/Annual) — for Practice entries
- `Health-Tracked` (checkbox) — for Practice entries that feed health metrics
- `Category` (select: Exercise/Recovery/Nutrition/Work/Mindfulness/Social/Chore/Commute/Entertainment/Learning) — for Practice entries (absorbs Activity Types `category`)

### 3.2 Possibility (potentiator) — Latent-state generator

**Current entry-types (10):** Activity, Subjective, Relational, Systemic, Diet, Financial, Observation, Goal, Vision, Aspiration

**Ideal entry-types (12) — add 2:**

| Entry-type | Tier | Purpose | Status |
|------------|------|---------|--------|
| `Activity` | operational | Activity log entry | ✅ exists |
| `Subjective` | operational | Subjective journal | ✅ exists |
| `Relational` | operational | Relational journal | ✅ exists |
| `Systemic` | operational | Systemic journal | ✅ exists |
| `Diet` | operational | Diet log | ✅ exists |
| `Financial` | operational | Financial log | ✅ exists |
| `Observation` | operational | Observation | ✅ exists |
| `Goal` | structural | Goal (future-pull) | ✅ exists |
| `Vision` | structural | Vision | ✅ exists |
| `Aspiration` | structural | Aspiration | ✅ exists |
| `Key-Metric` | operational | KPI measurement entry — absorbs legacy `Key Metrics` DB | 🆕 NEW |
| `Metric-Definition` | reference | KPI definition (unit, frequency, category) — absorbs legacy `Key Metrics` DB schema | 🆕 NEW |

**Property additions:**
- `Metric Value` (number) — for Key-Metric entries
- `Metric Unit` (select: count/percentage/hours/minutes/rupees/dollars/BMI/heart-rate) — for Key-Metric + Metric-Definition
- `Metric Frequency` (select: Daily/Weekly/Monthly/Quarterly/Annual) — for Metric-Definition
- `Metric Category` (select: Health/Financial/Productivity/Relational/Spiritual/Cognitive) — for Metric-Definition

### 3.3 Process (nexus) — Contact-boundary

**Current entry-types (13):** Opportunity, Directive, Risk, Insight, Reflection, Integration, Pattern, Note, Knowledge-Category, Knowledge-Atom, Decision, Crisis, Transformation-Event

**Ideal entry-types (15) — add 2:**

| Entry-type | Tier | Purpose | Status |
|------------|------|---------|--------|
| `Opportunity` | structural | Catalyst-kind opportunity | ✅ exists |
| `Directive` | structural | Choice-kind directive | ✅ exists |
| `Risk` | structural | Catalyst-kind risk | ✅ exists |
| `Insight` | structural | Experience-kind insight | ✅ exists |
| `Reflection` | structural | Experience-kind reflection | ✅ exists |
| `Integration` | structural | Experience-kind integration | ✅ exists |
| `Pattern` | structural | Experience-kind pattern recognition | ✅ exists |
| `Note` | operational | Catalyst-kind raw note | ✅ exists |
| `Knowledge-Category` | reference | Knowledge taxonomy node | ✅ exists |
| `Knowledge-Atom` | structural | Discrete knowledge unit | ✅ exists |
| `Decision` | structural | Choice-kind decision | ✅ exists |
| `Crisis` | structural | Transformation-kind crisis | ✅ exists |
| `Transformation-Event` | structural | Transformation-kind event | ✅ exists |
| `Meeting` | operational | Meeting note (absorbs Notes Management DB function) | 🆕 NEW |
| `Document-Link` | operational | External document link (absorbs Knowledge Categories → Documents DB relation) | 🆕 NEW |

**Property additions:**
- `Source URL` (url) — ✅ already exists, used by Document-Link
- `Capture Method` (select) — ✅ already exists
- `Meeting Date` (date) — for Meeting entries
- `Attendees` (relation → World.Person) — for Meeting entries

### 3.4 Identity (significator) — Persistent identity-pattern

**Current entry-types (6):** Purpose, Value, Principle, Identity-Statement, Pillar, Strategic-Ideal

**Ideal entry-types (11) — add 5 reference types (absorbs legacy reference DBs):**

| Entry-type | Tier | Purpose | Status |
|------------|------|---------|--------|
| `Purpose` | structural | Life purpose | ✅ exists |
| `Value` | structural | Core value | ✅ exists |
| `Principle` | structural | Operating principle | ✅ exists |
| `Identity-Statement` | structural | Identity statement | ✅ exists |
| `Pillar` | structural | Life pillar | ✅ exists |
| `Strategic-Ideal` | structural | Strategic ideal | ✅ exists |
| `Archetype-Drive` | reference | Descriptive backing for the 4 drives (Agency/Communion/Eros/Agape) — absorbs legacy `Drives` DB | 🆕 NEW |
| `Developmental-Level` | reference | Developmental level definition — absorbs legacy `Levels` DB | 🆕 NEW |
| `Line-of-Development` | reference | Developmental line definition — absorbs legacy `Lines of Development` DB | 🆕 NEW |
| `Type-Definition` | reference | Holon type definition (Donor/Acceptor/Sharer/Multivalent/Noble) — absorbs legacy `Types` DB | 🆕 NEW |
| `Archetype` | reference | 22-archetype definition (role + complex combination) — absorbs legacy `Quadrants` DB function | 🆕 NEW |

**Property additions:**
- `Drive Formula` (rich_text) — for Archetype-Drive entries (the G_z/P_z formula)
- `Drive Challenges` (rich_text) — for Archetype-Drive entries (failure modes)
- `Drive Pathologies` (rich_text) — for Archetype-Drive entries
- `Level Capacities` (rich_text) — for Developmental-Level entries
- `Level Challenges` (rich_text) — for Developmental-Level entries
- `Line Description` (rich_text) — for Line-of-Development entries
- `Line Active` (checkbox) — for Line-of-Development entries

### 3.5 World (greatway) — Operating environment

**Current entry-types (18):** Annual Goal, Quarterly Goal, Project, Task, System, Resource, Sprint, Milestone, Budget, Campaign, Content, Person, Group, Community, Organization, Network, Movement, Place

**Ideal entry-types (22) — add 4:**

| Entry-type | Tier | Purpose | Status |
|------------|------|---------|--------|
| `Annual Goal` | structural | Annual goal | ✅ exists |
| `Quarterly Goal` | structural | Quarterly goal | ✅ exists |
| `Project` | structural | Project | ✅ exists |
| `Task` | operational | Task | ✅ exists |
| `System` | structural | System/routine | ✅ exists |
| `Resource` | operational | Resource | ✅ exists |
| `Sprint` | operational | Sprint | ✅ exists |
| `Milestone` | structural | Milestone | ✅ exists |
| `Budget` | operational | Budget entry | ✅ exists |
| `Campaign` | operational | Campaign | ✅ exists |
| `Content` | operational | Content piece | ✅ exists |
| `Person` | operational | Person (external holon) | ✅ exists |
| `Group` | operational | Group | ✅ exists |
| `Community` | operational | Community | ✅ exists |
| `Organization` | operational | Organization | ✅ exists |
| `Network` | operational | Network | ✅ exists |
| `Movement` | operational | Movement | ✅ exists |
| `Place` | operational | Place | ✅ exists |
| `Quadrant-Definition` | reference | Wilber 4-quadrant definition — absorbs legacy `Quadrants` DB | 🆕 NEW |
| `Time-Period` | reference | Time period grouping (Day/Week/Month/Quarter/Year) — absorbs legacy calendar DBs | 🆕 NEW |
| `Financial-Account` | operational | Financial account (absorbs Finance DBs) | 🆕 NEW |
| `Role` | reference | Role/designation (absorbs Roles and Designations DB) | 🆕 NEW |

**Property additions for Person entries (absorbs legacy People DB's 31 properties — curated subset):**
- `Aspirational Drive` (select: Security & Stability / Connection & Belonging / Status & Recognition / Mastery & Impact / Growth & Understanding)
- `Developmental Altitude` (select: LVL 3 Red / LVL 4 Amber / LVL 5 Orange / LVL 6 Green / LVL 7 Turquoise)
- `Relationship Status` (select: Family Member / Mentor / Close Friend / Close Acquaintance / Coworker / Acquaintance)
- `Networking Profile` (select: Key Ally / Active Collaborator / Mentor/Advisor / Protégé/Mentee / Peer/Sounding Board / Inactive / Archive)
- `Desired Trajectory` (select: Deepen / Maintain / Activate / Graceful Exit / Inactive)
- `Value Exchange Balance` (select: I am in Credit / Balanced / I am in Debt)
- `Last Interaction Sentiment` (select: Positive / Neutral / Tense / Negative)
- `City` (select: 20+ city options from legacy People DB)
- `Timezone` (select: IST / EST / PST / GMT / GST)
- `Core Shadow` (select: Fear of Insignificance / Fear of Rejection / Fear of Chaos/Uncertainty / Fear of Powerlessness/Domination)
- `Engagement Blueprint` (rich_text)
- `Key Personal Intel` (rich_text)
- `Professional Domain` (rich_text)
- `Influence Toolkit` (multi_select — top 15 from legacy 65 options, curated)

**Property additions for Community entries (absorbs legacy Community DB):**
- `Community Type` (select: Professional / Personal / Spiritual / Creative / Investment / Mastermind)
- `Strategic Value` (select: Core / Supporting / Peripheral)
- `Covenant` (rich_text) — shared purpose

**Property additions for Time-Period entries (absorbs legacy calendar DBs):**
- `Period Type` (select: Day / Week / Month / Quarter / Year)
- `Period Start` (date)
- `Period End` (date)
- `Period Label` (formula — e.g. "2026-W27", "2026-Q3", "2026-07")

---

## 4. Ideal Relation Property Set

### 4.1 Inter-DB relations (fractal coupling + currency flow)

**Already implemented (13 dual_property, v0.9.0):** ✅ Keep all

| From | Property | To | Currency |
|------|----------|-----|----------|
| State | Related to Process (Rewrites (Matrix)) | Process | Transformation ↓ |
| State | Related to Process (Sends Catalyst To (Matrix)) | Process | Catalyst ↑ |
| State | Related to Identity (Sub-holon Of) | Identity | Fractal coupling |
| Possibility | Related to World (Sub-holon Of) | World | Fractal coupling |
| Possibility | Related to Process (Rewrites (Potentiator)) | Process | Transformation ↓ |
| Possibility | Related to Process (Sends Experience To (Potentiator)) | Process | Experience ↑ |
| Identity | Related to World (Coheres With (Significator)) | World | Bonding surface |
| Identity | Related to World (For Significator) | World | Bonding surface |
| Identity | Related to Process (Fires Transformation On) | Process | Transformation ↓ |
| Identity | Related to Process (Sends Catalyst To (Significator)) | Process | Catalyst ↑ |
| Identity | Related to Process (Triggered By) | Process | Transformation trigger |
| Identity | Emits Choice To | Process | Choice ↑ |
| World | Related to Process (Emits Choice To) | Process | Choice ↑ |

**NEW inter-DB relations to add (5):**

| From | Property | To | Purpose |
|------|----------|-----|---------|
| World.Person | `Anchored To Archetype` | Identity.Archetype | Link a person to their developmental archetype (reference relation) |
| World.Person | `At Developmental Level` | Identity.Developmental-Level | Link a person to their assessed developmental level |
| World.Time-Period | `Contains Entries` | All 5 DBs (via Date) | Time-based grouping — entries with Date in the period |
| Possibility.Key-Metric | `Measures` | State.Practice / World.Project | Link a metric to what it measures |
| Process.Meeting | `Attended By` | World.Person | Meeting attendees (replaces legacy Knowledge Categories → Notes relation) |

### 4.2 Intra-DB relations (hierarchy + dialectics)

**State (matrix) — 4 intra-DB relations:**
- `Parent` ↔ `Sub-item` (hierarchy) ✅ exists
- `Blocked By` (dependency) ✅ exists
- `Refines` (evolution) ✅ exists
- `Supersedes` (versioning) ✅ exists
- 🆕 `State-Definition-For` (links a State-Definition reference entry to the State entries it describes)

**Possibility (potentiator) — 3 intra-DB relations:**
- `Crystallized To` → State (inter-DB, already exists)
- `Reveals` → State (inter-DB, already exists)
- 🆕 `Metric-Definition-For` (links a Metric-Definition reference entry to Key-Metric entries)
- 🆕 `Goal-Refines-To` (links a Goal entry to a more refined sub-goal)

**Process (nexus) — 5 intra-DB relations:**
- `Counter-Synthesis` ✅ exists
- `Counterpart` ✅ exists
- `Reinforces` ✅ exists
- `Counter-Tension` ✅ exists (to Identity)
- 🆕 `Knowledge-Category-Contains` (links a Knowledge-Category to its Knowledge-Atoms)
- 🆕 `Meeting-Produced` (links a Meeting to the Notes/Insights/Decisions it produced)

**Identity (significator) — 5 intra-DB relations:**
- `Coheres With` ✅ exists
- `In Tension With` ✅ exists
- `Parent item` ↔ `Sub-item` ✅ exists
- 🆕 `Archetype-Activates-Drive` (links an Archetype entry to its Archetype-Drive entries)
- 🆕 `Line-Of-Development-For` (links a Line-of-Development to the Developmental-Levels it spans)
- 🆕 `Type-Definition-For` (links a Type-Definition to Identity entries of that type)

**World (greatway) — 5 intra-DB relations:**
- `Blocks` ✅ exists
- `Parent item` ↔ `Sub-item` ✅ exists
- 🆕 `Time-Period-Contains` (links a Time-Period to entries in that period — for goals, tasks, metrics)
- 🆕 `Quadrant-Definition-For` (links a Quadrant-Definition to World entries in that quadrant)
- 🆕 `Role-Held-By` (links a Role entry to the Person holding it)
- 🆕 `Community-Contains-People` (links a Community to its People — already partially exists via `Related to Potentiator (People)`)

---

## 5. Legacy DB → 5-DB Function Mapping

This table proves every legacy DB function is absorbed by the ideal v0.11 schema.

| Legacy DB | Legacy function | → 5-DB location | Entry-type | Properties absorbed |
|-----------|----------------|-----------------|------------|---------------------|
| **People** (31 props) | Rich people CRM | World | `Person` | 14 curated Person properties (Aspirational Drive, Developmental Altitude, Relationship Status, Networking Profile, Desired Trajectory, Value Exchange Balance, Last Interaction Sentiment, City, Timezone, Core Shadow, Engagement Blueprint, Key Personal Intel, Professional Domain, Influence Toolkit) |
| **Community** (9 props) | Community CRM | World | `Community` | Community Type, Strategic Value, Covenant (3 properties) |
| **Knowledge Categories** (7 props) | Knowledge taxonomy | Process | `Knowledge-Category` | Priority Topic (renamed), Document-Link relation, Related Knowledge Categories (intra-DB) |
| **Activity Types** (7 props) | Activity taxonomy | State | `Practice` | Category, Frequency Target, Health-Tracked, Duration (4 properties) |
| **Key Metrics** (6 props) | KPI tracking | Possibility | `Key-Metric` + `Metric-Definition` | Metric Value, Metric Unit, Metric Frequency, Metric Category (4 properties) |
| **Lines of Development** (5 props) | Developmental lines | Identity | `Line-of-Development` | Line Description, Line Active (2 properties) + `Line-Of-Development-For` relation |
| **Drives** (5 props) | 4 drives backing | Identity | `Archetype-Drive` | Drive Formula, Drive Challenges, Drive Pathologies (3 properties) |
| **States** (5 props) | State definitions | State | `State-Definition` | Benefits, Challenges, Description, Formula (4 properties) |
| **Levels** (72 props) | Developmental levels | Identity | `Developmental-Level` | Level Capacities, Level Challenges (2 properties) + relations to other reference entries |
| **Quadrants** (5 props) | Wilber 4-quadrant | World | `Quadrant-Definition` | Description, Formula (2 properties) |
| **Types** (8 props) | Holon type definitions | Identity | `Type-Definition` | Challenges, Description, Identification, Language to Influence (4 properties) |
| **Days** | Daily grouping | World | `Time-Period` (Period Type=Day) | Period Start, Period End, Period Label |
| **Weeks** | Weekly grouping | World | `Time-Period` (Period Type=Week) | Same |
| **Months** | Monthly grouping | World | `Time-Period` (Period Type=Month) | Same |
| **Quarters** | Quarterly grouping | World | `Time-Period` (Period Type=Quarter) | Same |
| **Years** | Annual grouping | World | `Time-Period` (Period Type=Year) | Same |
| **Notes Management** | Meeting notes | Process | `Meeting` | Meeting Date, Attendees |
| **Roles and Designations** | Role tracking | World | `Role` | Role-Held-By relation |
| **Documents DB** | Document linking | Process | `Document-Link` | Source URL (already exists) |
| **Financial Accounts** | Account tracking | World | `Financial-Account` | (properties TBD in implementation) |
| **Activity Types** (category) | Activity categorization | State | `Practice` (Category property) | 10 category options |
| **Knowledge Categories** (priority) | Priority topics | Process | `Knowledge-Category` | Priority Topic checkbox |

**Result: 22 legacy DBs → fully absorbed into 5 DBs via 11 new entry-types + ~30 new properties + 11 new relations.**

---

## 6. The UX Layer — Notion Dashboard + AI Agent

### 6.1 Notion Dashboard UX

**For the user via Notion Frontend:**

1. **5 DB views** (not 22) — each DB has filtered views per entry-type:
   - State: "Patterns" view, "Practices" view, "Foundations" view, "State Definitions" view
   - Possibility: "Daily Logs" view (Activity/Diet/Financial/etc.), "Goals" view, "Metrics" view
   - Process: "Notes" view, "Insights" view, "Decisions" view, "Meetings" view, "Knowledge" view
   - Identity: "Purpose & Values" view, "Archetypes" view (reference), "Levels" view (reference)
   - World: "Goals & Projects" view, "People" view, "Communities" view, "Time Periods" view

2. **Reference entries are pinned** — each DB has a "Reference" section at the top showing the archetype/drive/level/type definitions, so the user always sees the ontological backing.

3. **Time-Period entries auto-create** — a daily/weekly/monthly script creates Time-Period entries, and entries with Date properties auto-link to their containing Time-Period.

4. **Person entries are rich** — the 14 curated Person properties give the user a full CRM without needing a separate People DB.

### 6.2 AI Agent UX (MCP/CLI)

**For AI agents:**

1. **`get_schema`** returns the full entry-type taxonomy per DB, with reference entry-types marked. AI agents learn that `Archetype-Drive` is a reference entry-type, not an operational one.

2. **Semantic hints** (U-1, v0.10.3) on every relation tell the AI agent what each relation means ontologically.

3. **`fill_rate`** (U-8, v0.10.3) identifies which entry-types/properties are underused, guiding the AI agent's suggestions.

4. **`auto_enrich`** (v0.10.2, suggestion-only) suggests entry-types + universal properties for uncategorized entries. The AI agent presents these to the user for manual approval.

5. **`build_context`** assembles the relational neighborhood for an entry — now including reference entries (Archetype, Drive, Level) so the AI agent has full ontological context.

6. **Time-aware queries** — `query --time-period "2026-W27"` returns all entries in that week, across all 5 DBs.

---

## 7. Implementation Plan (for next sprint)

### Phase 1: Add new entry-types (Notion API, ~2h)
- State: add `State-Definition`, `Practice`, `Inventory`
- Possibility: add `Key-Metric`, `Metric-Definition`
- Process: add `Meeting`, `Document-Link`
- Identity: add `Archetype-Drive`, `Developmental-Level`, `Line-of-Development`, `Type-Definition`, `Archetype`
- World: add `Quadrant-Definition`, `Time-Period`, `Financial-Account`, `Role`

### Phase 2: Add new properties (Notion API, ~3h)
- State.Practice: Frequency Target, Health-Tracked, Category
- State.State-Definition: Benefits, Challenges, Description, Formula
- Possibility.Key-Metric + Metric-Definition: Metric Value, Metric Unit, Metric Frequency, Metric Category
- Identity reference types: Drive Formula, Drive Challenges, Drive Pathologies, Level Capacities, Level Challenges, Line Description, Line Active
- World.Person: 14 curated properties (Aspirational Drive, Developmental Altitude, etc.)
- World.Community: Community Type, Strategic Value, Covenant
- World.Time-Period: Period Type, Period Start, Period End, Period Label

### Phase 3: Add new relations (Notion API, ~2h)
- 5 new inter-DB relations (§4.1)
- 11 new intra-DB relations (§4.2)

### Phase 4: Update YAML schemas (~2h)
- Add per_entry_type YAML files for each new entry-type
- Update per_db YAML files with new relations

### Phase 5: Update config + rebuild (~1h)
- Update `lifeos.config.default.json` with new entry-types
- Rebuild binary
- Run `validate-yaml --self-test`
- Run MCP smoke test

### Phase 6: Semantic hints for new relations (~1h)
- Extend `quick_link::relation_semantic_hint()` map with the 16 new relations

### Phase 7: Migrate legacy data (~4h, optional)
- Port People entries' 14 properties from legacy People DB → World.Person
- Port Community entries' 3 properties from legacy Community DB → World.Community
- Port Activity Types → State.Practice
- Port Key Metrics → Possibility.Key-Metric + Metric-Definition
- Port Drives/States/Levels/Quadrants/Types → Identity/State/World reference entries

**Total: ~15h implementation (2 days).** No ontology drift — all new entry-types trace to HoloOS docs.

---

## 8. Gap Analysis — What This Architecture Solves

| Gap from §1.4 | Solution in v0.11 |
|---------------|-------------------|
| Rich people CRM (31 properties) | World.Person with 14 curated properties (the most useful subset) |
| Activity taxonomy | State.Practice entry-type with Category + Frequency Target |
| KPI tracking | Possibility.Key-Metric + Metric-Definition |
| Developmental line tracking | Identity.Line-of-Development reference entry-type |
| Developmental level assessment | Identity.Developmental-Level reference entry-type |
| Descriptive backing for universal properties | Identity.Archetype-Drive, State.State-Definition, World.Quadrant-Definition, Identity.Type-Definition, Identity.Archetype |
| Time-based grouping | World.Time-Period entry-type (Day/Week/Month/Quarter/Year) |
| Document linking | Process.Document-Link entry-type + Source URL property |

**All 8 gaps closed.** The 5-DB structure now performs all functions of the legacy 22-DB structure.

---

## 9. YAGNI Check

Every proposed addition passes the YAGNI test (per AGENTS.md §6.5):

| Addition | YAGNI justification |
|----------|---------------------|
| 11 new entry-types | Each absorbs a specific legacy DB function — not speculative |
| ~30 new properties | Each maps to a legacy property that was actually used — not "for the future" |
| 16 new relations | Each encodes a specific ontological relationship — no decorative relations |
| 14 curated Person properties | Subset of legacy 31 — the most-used ones (curated, not exhaustive) |

**What we're NOT adding (YAGNI):**
- We're NOT porting all 31 People properties — 14 curated subset is enough
- We're NOT porting all 72 Levels properties — 2 (Capacities, Challenges) + relations is enough
- We're NOT adding the 65-option Influence Toolkit — top 15 curated subset
- We're NOT adding speculative "for the future" properties
- We're NOT keeping the 5 backward-compat alias tools (separate YAGNI item Y-1)

---

## 10. Architectural Preferences Adherence

1. ✅ **No bulk-tagging** — all new entry-types/properties are populated by manual curation
2. ✅ **Manual relation curation** — all new relations require explicit user action
3. ✅ **5 DBs only** — no new DBs created; all functions absorbed into the 5
4. ✅ **YAGNI aggressive** — curated subsets, not exhaustive ports
5. ✅ **Renamed DBs are canon** — all new entry-types use the new DB names (State/Possibility/Process/Identity/World)
6. ✅ **HoloOS ontology is foundation** — reference entry-types (Archetype-Drive, Developmental-Level, etc.) trace directly to HoloOS docs 02.3, 02.4, 08.5

---

## 11. Next Steps

This is a **design document only**. No code changes made. To implement:

1. Review this architecture with the user
2. Get approval on the entry-type taxonomy + property set
3. Execute the 7-phase implementation plan in §7
4. Migrate legacy data (optional, Phase 7)
5. Update audit reports to reflect v0.11

**Decision required from user:**
- [ ] Approve the 11 new entry-types?
- [ ] Approve the ~30 new properties?
- [ ] Approve the 16 new relations?
- [ ] Approve the 14 curated Person properties (from legacy 31)?
- [ ] Approve the reference entry-type pattern (Drives/States/Levels/etc. as reference entries within the 5 DBs)?
- [ ] Approve the Time-Period entry-type (absorbs calendar DBs)?

Once approved, implementation is ~15h (2 days).

---

*Architecture proposed 2026-07-06 by LifeOS-Architect (Task ID 8). Awaiting user approval before implementation.*
