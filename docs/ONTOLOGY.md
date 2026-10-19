# ONTOLOGY.md — LifeOS v4.1 Consciousness-Prosthetic Architecture

> **Source:** Evolved from HoloOS ontological theory + v4.1 5-DB merger.
> **Purpose:** Grounds the LifeOS architecture in its teleological-aim:
> shaping the causal chain of the user's life toward an ideal-future.
> Every LifeOS design decision must trace back to a principle documented here.

---

## 1. The Core Thesis

LifeOS is a **consciousness-prosthetic** — an external scaffold that shapes
the causal chain of the user's life toward an ideal-future. It does this by
running **one causal amplification cycle** through **5 databases** across
**3 functional layers**.

The 3 layers + 3 flows + 1 cycle define the entire architecture:

```
                    ┌─────────────────────────────────────┐
                    │  Layer A — Teleological Pull        │
                    │  (Trajectory DB)                    │
                    │                                     │
                    │  Purpose → Value → Vision-Statement │
                    │     → Annual-Goal → Quarterly-Goal  │
                    │     → Project → Task                │
                    └────────┬────────────────────────────┘
                             │ Pull (downward)
                             ▼
   ┌─────────────────────────────────────────────────────────┐
   │  Layer B — Historical Record                            │
   │                                                         │
   │  Logbook ──── Ground ───▶ Synthesis ──── Condense ───▶ Profile
   │  (capture)               (process)                  (state mirror)│
   └─────────────────────────────────────────────────────────┘
                             │ Feedback (loop)
                             ▼
                    ┌─────────────────────────────────────┐
                    │  Layer C — Action Interface         │
                    │  (Context DB)                       │
                    │  People / Communities / Orgs /      │
                    │  Financial-Accounts / Places        │
                    └─────────────────────────────────────┘
```

---

## 2. The 5 DBs

| # | DB | Layer | Purpose | Entry-types |
|---|-----|-------|---------|-------------|
| 1 | **Trajectory** | A (Pull) | The teleological hierarchy — pull IS the parent/child tree | Purpose, Value, Principle, Vision-Statement, Identity-Statement, Annual-Goal, Quarterly-Goal, Milestone, Project, Task, Campaign, Content |
| 2 | **Logbook** | B (Record) | Objective ground-reality capture — 6 channels | Activity, Diet, Financial, Subjective, Relational, Systemic |
| 3 | **Synthesis** | B (Record) | Logs → insights (polar ± pair) | Note, Opportunity, Strength, Directive, Risk |
| 4 | **Profile** | B (Record) | Cumulative state mirror (RPG status) | Trait, Metric, Capacity, Asset |
| 5 | **Context** | C (Action) | The environment (who/what is around) | Person, Community, Organization, Financial-Account, Place |

**Universal properties (on all 5 DBs):** `Name` (title), `Created Time`, `Last Edited Time` (Notion auto)

---

## 3. The 3 Functional Layers

### Layer A — Teleological Pull (Trajectory DB)
**Purpose:** Articulate + simulate the drive toward the ideal-future.
The pull IS the parent/child tree — open one DB, expand the hierarchy,
see the pull from Purpose to today's Task.

The 3 internal layers of Trajectory:
| Layer | Entry-types | Churn | Review cadence |
|-------|-------------|-------|----------------|
| **Reference** (timeless) | Purpose, Value, Principle, Vision-Statement, Identity-Statement | Years | Annual |
| **Strategic** (temporal) | Annual-Goal, Quarterly-Goal, Milestone | Quarters | Quarterly |
| **Execution** (daily) | Project, Task, Campaign, Content | Days | Daily |

### Layer B — Historical Record (Logbook + Synthesis + Profile)
**Purpose:** Keep tabs on history + current trajectory/trends.

Three sub-systems, each with a distinct role:
- **Logbook** — Objective ground-reality capture. ONE DB, 6 entry-types
  (one per channel: Body/Resource/Mind/Relational).
- **Synthesis** — Where logs become insights. Polar ± pair: positive
  (Opportunity/Strength), negative (Directive/Risk), neutral (Note).
- **Profile** — The cumulative state mirror. Bridge between history and
  the pull. Traits/Metrics/Capacities/Assets with current vs target values.

### Layer C — Action Interface (Context DB)
**Purpose:** Tell the user what to do + how. The environment the user
operates within. People, Communities, Organizations, Financial-Accounts,
Places — the entities the user acts through and on.

---

## 4. The 3 Flows + The Cycle

### Flow 1 — Teleological Pull (downward, within Trajectory)
**Path:** Vision-Statement → Annual-Goal → Quarterly-Goal → Project → Task
**What flows:** The ideal-future shape propagates downward through the
parent/child hierarchy. This is STRUCTURAL — it's the tree shape of the
DB, not a cross-DB relation.
**Constraint:** Purpose/Value/Principle constrain the hierarchy via
`Serves Value` (a self-relation, not a flow).

### Flow 2 — Ground-Truth (Trajectory → Logbook → Synthesis → Profile)
**Path:** Trajectory (Action executed) → Logbook (capture) → Synthesis
(process) → Profile (condense)
**What flows:** Objective ground-reality data — actions produce logs,
logs synthesize into insights, insights condense into the cumulative state.

### Flow 3 — Feedback (Profile + Synthesis → Trajectory)
**Path:** Profile (gap) → Trajectory (Vision/Goals revised); Synthesis
(long-term insights) → Trajectory (Vision reaffirmed/revised)
**What flows:** The gap signal + long-term insights inform the
trajectory simulation. Profile's `Closes Gap For` + `Informs Goal`
relations encode this.

### The Cycle

```
Trajectory → Logbook → Synthesis → Profile → Trajectory
  (pull +    (capture)  (process)   (condense)  (feedback)
   action)
```

**4 hops.** Tighter feedback = faster amplification. Each cycle through
the loop amplifies the user's causal capacity — actions get sharper,
logs get denser, synthesis gets deeper, profile gets clearer, the pull
gets more accurate.

---

## 5. The 12 Inter-DB Relations

| # | From DB | Property | To DB | Flow |
|---|---------|----------|-------|------|
| 1 | Trajectory | `Generates Logs` | Logbook | Ground |
| 2 | Trajectory | `Assigned To` / `Involves` | Context | — |
| 3 | Trajectory | `Spawned By` | Synthesis | Ground (rev) |
| 4 | Trajectory | `Measured By` | Profile | Feedback |
| 5 | Logbook | `Source Project` | Trajectory | Ground (rev) |
| 6 | Logbook | `Subject Person` / `Subject Account` | Context | — |
| 7 | Logbook | `Synthesized Into` | Synthesis | Ground |
| 8 | Synthesis | `Spawns` | Trajectory | Ground (rev) |
| 9 | Synthesis | `Condenses Into` | Profile | Ground |
| 10 | Synthesis | `Revises` | Trajectory | Feedback |
| 11 | Profile | `Closes Gap For` | Trajectory | Feedback |
| 12 | Profile | `Informs Goal` | Trajectory | Feedback |

**Plus 4 intra-DB self-relations in Trajectory:**
`Parent`, `Serves Value`, `Blocked By`, `Linked Milestone`.

---

## 6. The Context YAML Formula

Each DB has ONE formula property that outputs per-entry-type YAML context.
This replaces redundant self-inferential properties with a single
computed field that AI agents can parse.

Example (Trajectory entry of type Task):
```yaml
type: Task
status: Active
parent: "Implement LifeOS v1.0"
serves_value: ["Compounding leverage", "Sovereignty"]
end_date: 2026-09-30
priority: Critical
assigned_to: []
```

The formula reads the entry's other properties and emits structured YAML
based on entry-type. Five formulas total (one per DB).

---

## 7. LifeOS Design Principles

1. **5 DBs, not 22, not 8** — The merger of Vision+Compass+Action into
   Trajectory eliminated 6 cross-DB relations that are now intra-DB
   self-relations. The 5-DB structure is the minimum that separates the
   3 functional layers.

2. **The pull IS the hierarchy** — In v4.1, the teleological pull is not
   a flow between DBs; it's the parent/child tree within Trajectory. Open
   one DB → expand the hierarchy → see the pull from Purpose to today's Task.

3. **Every relation is a deliberate choice** — Tools surface gaps
   (`relational_gaps`) and suggest connections (`suggest_links`,
   `suggest_categorization`), but the user (or AI agent acting
   explicitly) must approve each link. NO auto-population. auto_enrich
   is suggestion-only since v0.10.2.

4. **Synthesis is polar** — Note/Opportunity/Strength/Directive/Risk
   encode the ± polarity of insight. Polarity is a first-class concept,
   not a tag.

5. **Profile is the RPG status mirror** — Current Value vs Target Value
   for every Trait/Metric/Capacity/Asset. The gap between them IS the
   feedback signal to Trajectory.

6. **The cycle is the unit of health** — `cycle_health` checks whether
   each of the 3 flows (Pull / Ground / Feedback) has active links. A
   healthy cycle = active links in all 3 flows. Dormant flow = stuck cycle.

7. **YAGNI aggressively** — Don't add a property "for future use." Don't
   keep two tools that do the same thing. The v4.1 YAGNI cleanup removed
   4 redundant utility tools (capture / trace_trajectory / gap_analysis /
   surface_synthesis) because AI agents compose them from primitives
   (query / mutate / ancestors).

8. **MCP/CLI tools are for AI agents, not the user** — The user operates
   via Notion UI directly. Tools exist to give AI agents operational
   access to the 5 DBs. A dedicated tool is justified ONLY when:
   (a) it eliminates multiple round-trips, (b) it encodes architectural
   semantics the agent would miss, (c) it writes state in a specific
   way the agent shouldn't second-guess.

---

## 8. Tool → Architecture Map

| Architectural concept | LifeOS tool | Notes |
|----------------------|-------------|-------|
| Teleological pull (hierarchy walk) | `ancestors` | Walks up Parent self-relation. Returns layer labels (Reference/Strategic/Execution) for Trajectory entries. |
| Active goals + today's tasks + recent logs | `morning` | AI-agent "orient" call. One round-trip replaces 4+ queries. |
| Cycle health (3 flows active?) | `cycle_health` | Encodes the 3-flow architectural semantics. |
| Ground-truth flow | `query` + `mutate` + `link` | Agent composes: capture log → link to project → synthesize → condense. |
| Feedback flow | `query` Profile + `link` to Trajectory | Agent composes: identify gap → link to Goal/Vision. |
| Relational neighborhood | `build_context` | One call: outgoing + incoming + depth-2 + gap analysis. |
| Orphan detection | `orphans`, `relational_gaps` | Read-only audits. |
| Schema discovery | `get_schema` | First call any agent makes. |
| Validation | `validate_yaml` | Checks entries against YAML schema hierarchy. |

---

## References

- **architecture/legacy_mapping/FORMAL_SPEC_v4.1.md** — The complete
  machine-readable DB schema spec (5 DBs, all properties, all relations,
  3 flows, cycle, migration mapping).
- **architecture/legacy_mapping/PONYTAIL_AUDIT_v4.1.md** — The 34%
  dead-weight audit that preceded v4.1.
- **HoloOS `_THEORY/02_Ontology/`** — The pre-v4.1 ontological theory
  (Matrix/Potentiator/Nexus/Significator/GreatWay). Superseded by v4.1
  but referenced for historical context.

---

*v4.1 — the consciousness-prosthetic. The cycle is the unit of health.*
