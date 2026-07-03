# ONTOLOGY.md — LifeOS Native Ontological Foundation

> **Source:** Distilled from HoloOS `_THEORY/02_Ontology/` (58 docs).
> **Purpose:** Grounds the LifeOS 5-DB architecture in its ontological theory.
> Every LifeOS design decision must trace back to a principle documented here.

---

## 1. The Core Thesis

Every holon — atom, cell, organism, organization, civilization — runs **one invariant
systems architecture** with **two symmetrical but inverted metabolic cycles** operating
through a shared contact-boundary.

LifeOS operationalizes this as **5 databases**: 4 reservoirs + 1 contact-boundary.

---

## 2. The 8 Functional Roles → 5 LifeOS DBs

| Role | Symbol | LifeOS DB | Definition |
|------|--------|-----------|------------|
| Matrix | M | **Matrix** | Current-state organizer — preserves identity, classifies input, converts perturbation into state-update |
| Potentiator | P | **Potentiator** | Latent-state generator — reachable possibility field that returns refined future input |
| Catalyst | C | *(currency in Nexus)* | Input perturbation — unprocessed pressure crossing the boundary |
| Experience | E | *(currency in Nexus)* | Integrated state-update — processed input stored as adaptation |
| Significator | S | **Significator** | Persistent identity-pattern — continuity reservoir exposed to macro-transition |
| Transformation | T | **Nexus** | Threshold restructuring event — the contact-boundary itself |
| Great Way | G | **GreatWay** | Operating environment — context that receives commitments, generates pressure |
| Choice | Ch | *(currency in Nexus)* | Directional commitment — polarity output emitted into the environment |

**Key design decision:** Catalyst, Experience, and Choice are **currencies** (they flow
through the contact-boundary), not reservoirs. They live as entry-types within the Nexus DB,
discriminated by the `Kind` property. This is why LifeOS has 5 DBs, not 8.

---

## 3. The Two Cycles

### Lesser Cycle (Matrix ⇌ Potentiator)
- **Catalyst** flows: extra-holonic → contact-boundary → Matrix (ingestion)
- **Experience** flows: Matrix → contact-boundary → Potentiator (refinement)
- Matrix processes C, stores E. Potentiator processes E, stores C.
- **Boundary:** M⇄P, regulated by Eros↔Agape (vertical drive axis)
- **Health metric:** G_z (Goldilocks coherence — rewards balance)

### Greater Cycle (Significator ⇌ Great Way)
- **Transformation** flows: Great Way → contact-boundary → Significator
- **Choice** flows: Significator → contact-boundary → Great Way
- **Boundary:** S⇄G, regulated by Agency↔Communion (horizontal drive axis)
- **Health metric:** P_z (Polarization power — rewards commitment)
- **Ratcheting:** Transformation fires when pressure + latent pull exceed threshold

### The Shared Contact-Boundary
The Nexus (Transformation) is **shared** between both cycles. This is structurally
critical: the same membrane processes Catalyst/Experience (lesser) AND
Transformation/Choice (greater). The `Kind` property on Nexus entries discriminates
which currency is currently transmuting.

---

## 4. The Fractal Coupling (Inter-DB Hierarchy)

The holonic architecture is fractal — every holon's components are themselves holons.
This creates the inter-DB parent-child relationships:

| Holon H's component | IS the parent holon's... | LifeOS relation |
|---------------------|--------------------------|-----------------|
| Great Way (environment) | parent's **Potentiator** | GreatWay.Sub-holon Of → Potentiator.Contains Great Way Of |
| Significator (identity) | parent's **Matrix** | Significator.Sub-holon Of → Matrix.Contains Significator Of |
| Transformation (contact-boundary) | parent's **Catalyst** | Nexus → Matrix (via Updates relation) |
| Choice (output) | parent's **Experience** | Nexus → Potentiator (via Sourced From relation) |

This is the structural basis for the 13 dual_property inter-DB relations added in v0.9.0.

---

## 5. The 4 Drives

All 4 drives operate at BOTH contact boundaries (M⇌P AND S⇌G):

| Drive | Symbol | Axis | Structural Pole | LifeOS Property |
|-------|--------|------|-----------------|-----------------|
| Agency | A_z | Horizontal-contractive (boundary preservation) | Matrix | `Drive Activation` multi_select |
| Communion | C_z | Horizontal-expansive (field coupling) | Potentiator | `Drive Activation` multi_select |
| Eros | P_z | Vertical-ascendent (scale jumps) | Contact-boundary gradient | `Drive Activation` multi_select |
| Agape | G_z | Vertical-descendent (stabilization) | Closed cycle efficiency | `Drive Activation` multi_select |

**Formula:** A_z = 100·exp(-|ln(Ω_A)|), where Ω_A = (M·η_M)/(|C|+ε)
**Failure modes:** Agency→isolation, Communion→confluence, Eros→inflation, Agape→stagnation

---

## 6. The 4 Shadows (+ Sinkhole)

| Shadow | Reservoir | Sign | LifeOS `Shadow Pattern` option |
|--------|-----------|------|-------------------------------|
| Dark-Addiction | Matrix (surplus) | +1 (donor) | `Dark-Addiction` |
| Dark-Allergy | Matrix (deficit) | -1 (acceptor) | `Dark-Allergy` |
| Golden-Addiction | Potentiator (surplus) | +1 (donor) | `Golden-Addiction` |
| Golden-Allergy | Potentiator (deficit) | -1 (acceptor) | `Golden-Allergy` |
| Sinkhole of Indifference | Great Way (Choice-starvation) | 0 (depolarized) | `Sinkhole of Indifference` |

**Critical:** A holon can be metabolically efficient (high G_z) yet depolarized (low P_z)
— the sinkhole. G_z alone doesn't guarantee evolutionary direction; P_z is required.

---

## 7. Health Metrics: G_z × P_z

### G_z — Agape (Integrative Coherence)
```
G_z = 100 · (A_z/100 · C_z/100 · B_H · B_V)^(1/4)
```
- B_H = min(A_z, C_z) / max(A_z, C_z) — horizontal balance
- B_V = min(Eros, Agape) / max(Eros, Agape) — vertical balance
- **Rewards balance.** Geometric mean → any factor near 0 collapses G_z.
- Thresholds: <30 severe distortion; 30-70 sub-optimal; >70 dynamic equilibrium

### P_z — Eros (Transcendental Tension)
```
P_z = 100 · ∇Ψ · cos(θ_alignment)
```
- ∇Ψ = |P - M| / (P + M + ε) — structural potential gradient
- cos(θ_alignment) — behavioral output aligned with core Choice
- **Rewards commitment.** Neutrality is the pathology (sinkhole).
- Thresholds: <30 depolarized; 30-70 polarizing; >70 committed ascent

### Total Metabolic Health = G_z × P_z
Both required, neither sufficient alone.

---

## 8. The 9-Stage Digestion Process

The canonical metabolic cycle (HoloOS doc 03.1 §3). This is the intra-DB flow within
the Nexus (contact-boundary):

1. **Latent State** — (M_t, P_t) at rest
2. **Boundary Contact** — c_in = B(c_env; Ag, Cm) — Catalyst enters
3. **Matrix Ingestion** — M_t ⊕ c_in
4. **Matrix Digestion** — e_t = Digest(M_t, c_in; η_M) — Experience generated
5. **Potentiator Ingestion** — P_t ⊕ e_t
6. **Potentiator Digestion** — c'_{t+1} = Potentiate(P_t, e_t; η_P, Er, Agp) — refined Catalyst
7. **Significator Accumulation** — S_t = S_{t-1} + w_t · e_t
8. **Transformation Threshold** — f(G_t, P_t, S_t) > T_thresh → Trigger T_t
9. **Choice & Downward Rewrite** — Ch_t = Choice(S_t, T_t; η_S) → rewrites M, P

**LifeOS `Digestion Stage` property** tracks an entry's position in this cycle.
Nexus entries MUST set this; reservoir entries MAY set it (projected/snapshotted).

---

## 9. The 5 Holon Types

Type = the Significator's invariant bonding-disposition toward the Great Way.

| Type | Valence State | Bonding | LifeOS `Holon Type` option |
|------|--------------|---------|---------------------------|
| Donor | Open, addiction-side | STO-leaning, radiative | `Donor` |
| Acceptor | Open, allergy-side | STS-leaning, absorptive | `Acceptor` |
| Sharer | Balanced (high G_z) | Covalent-capable | `Sharer` |
| Multivalent | Several open registers | Multi-register | `Multivalent` |
| Noble | Closed (no deficit) | Inert (graduated OR sinkhole) | `Noble` |

**Type ⊥ Stage** — they are independent. Stage = how full the engine is (dynamic).
Type = the invariant shape of the valence deficit (stable under excitation).

**Derivation:** `lifeos derive-type --page-id <id>` computes Holon Type from the
Valence Signature YAML on Significator entries.

---

## 10. The 22 Named Archetypes

7 functional roles × 3 complexes (Mind/Body/Spirit) + Choice = 22.

The 8 functional roles are the **operators**; the 22 named archetypes are the
**operands** (domain-specific elaborations). LifeOS tracks both:
- `Archetype Role` (select, 8 options) — the functional role
- `Complex` (select, 4 options: Mind/Body/Spirit/None) — the substrate face

The combination (role, complex) identifies which of the 22 archetypes an entry
instantiates. See `lifeos archetype-index` for the full mapping.

---

## 11. LifeOS Design Principles (Derived from Ontology)

1. **5 DBs, not 8** — Currencies (C, E, Ch) flow through the Nexus, they don't have their own reservoirs.

2. **Nexus.Kind discriminates currency** — A Nexus entry tagged `Kind: Catalyst` carries
   Catalyst; `Kind: Experience` carries Experience; etc. The `mutate` tool enforces
   that Kind constrains which relations can be populated.

3. **Dual-property relations encode the fractal coupling** — The 13 inter-DB dual_property
   relations (added v0.9.0) are not arbitrary; they operationalize the parent-child holon
   relationships from §4.

4. **Every relation is deliberate** — Tools surface gaps (`relational_gaps`) and suggest
   connections (`suggest_categorization`), but the user must approve each. No auto-population.

5. **G_z × P_z = Total Health** — `health_metrics` computes both. A holon with high G_z
   but low P_z is in the sinkhole of indifference (efficient but going nowhere).

6. **Type ⊥ Stage** — `Holon Type` and `Digestion Stage` are independent properties.
   Never collapse them. Type is stable; Stage is dynamic.

7. **The Significator is the bridge** — It lives in BOTH cycles (lesser accumulation +
   greater transformation). Most inter-DB relations pass through it.

8. **The Great Way holds external holons** — People, communities, groups are external
   holons in the operating environment. They belong in GreatWay, not Significator.

---

## 12. Ontology → Implementation Map

| Ontology concept | LifeOS implementation | Tool |
|------------------|----------------------|------|
| 8 functional roles | `Archetype Role` select property | `archetype-index` |
| 4 complexes | `Complex` select property | `archetype-index` |
| 4 drives | `Drive Activation` multi_select | `drive_assessment` |
| 5 shadows | `Shadow Pattern` select | — |
| 9 digestion stages | `Digestion Stage` select | `holonic_synthesis` |
| 5 holon types | `Holon Type` select | `derive_type` |
| Valence signature | `Valence Signature` rich_text (YAML) | `valence_signature` |
| G_z / P_z metrics | Computed from entry data | `health_metrics` |
| Fractal coupling | 13 dual_property inter-DB relations | `relational_graph` |
| Contact-boundary | Nexus DB (Kind discriminates currency) | `holonic_synthesis` |
| Relational gaps | Orphaned entries | `relational_gaps` |
| Context assembly | Relational neighborhood | `build_context` |

---

## References

- **HoloOS `_THEORY/02_Ontology/00.md`** — The master map + G_z/P_z articulation
- **HoloOS `_THEORY/02_Ontology/02.1`** — Microcosmic metabolic architecture (lesser cycle)
- **HoloOS `_THEORY/02_Ontology/02.2`** — Macrocosmic metabolic architecture (greater cycle)
- **HoloOS `_THEORY/02_Ontology/03.1`** — Universal archetype anatomy (9-stage process)
- **HoloOS `_THEORY/02_Ontology/03.2`** — 22 named archetypes index
- **HoloOS `_THEORY/02_Ontology/08.5`** — Extra-holonic deepening (fractal coupling)
- **HoloOS `_THEORY/01_Epistemology/4_Type_Validation_Protocol.md`** — Type ⊥ Stage independence
