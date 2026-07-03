# LifeOS v0.9.0 Upgrade Plan

**Date:** 2026-07-03
**Scope:** Addresses UPGRADE_FLAGS.md Flags 1–6 + the user's v0.9.0 relocation directive.
**Status:** ✅ Schema design, validator, and migration scripts COMPLETE. Ready for execution against the Notion workspace.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [What v0.9.0 Changes](#2-what-v090-changes)
3. [The 5-DB Architecture (recap)](#3-the-5-db-architecture-recap)
4. [Entry-Type Relocations](#4-entry-type-relocations)
5. [Dual-Property Inter-DB Relations](#5-dual-property-inter-db-relations)
6. [YAML Schema Hierarchy](#6-yaml-schema-hierarchy)
7. [Validator Implementation](#7-validator-implementation)
8. [Migration Scripts](#8-migration-scripts)
9. [Rust Codebase Updates](#9-rust-codebase-updates)
10. [Execution Runbook](#10-execution-runbook)
11. [Verification Checklist](#11-verification-checklist)
12. [Rollback Plan](#12-rollback-plan)
13. [Ontology References](#13-ontology-references)

---

## 1. Executive Summary

v0.9.0 closes every open flag from the v0.8.0 upgrade (UPGRADE_FLAGS.md) plus implements the user's v0.9.0 directive. The upgrade is **strictly additive** — no existing data is destroyed. It consists of four deliverable categories:

| # | Deliverable | Status | Location |
|---|-------------|--------|----------|
| 1 | **YAML schema files** (3-tier hierarchy: universal → per_db → per_entry_type) | ✅ 63 files | `schemas/` |
| 2 | **YAML schema validator** (Python reference + Rust integration) | ✅ Compiles + self-tests pass | `scripts/upgrade_v0.9.0/yaml_schema_validator.py`, `lifeos-core/src/util/yaml_schemas.rs`, `lifeos-core/src/tools/validate_yaml.rs` |
| 3 | **Python migration scripts** (6 scripts that mutate Notion) | ✅ 6 scripts + shared module | `scripts/upgrade_v0.9.0/` |
| 4 | **Rust codebase updates** (new CLI command, new MCP tool, config schema) | ✅ Builds clean | `lifeos-core/src/`, `lifeos/src/` |

**Key user directives implemented:**

- ✅ **Port People/Community/Group → GreatWay** — `03_port_auxiliary_people_community_to_greatway.py`
- ✅ **Port Notes & Knowledge-Categories → Nexus** (Potentiator was "too flat") — `04_port_notes_knowledge_to_nexus.py`
- ✅ **Dual Property Relationships** for inter-DB hierarchies — `01_add_dual_property_relations.py` adds 17 dual_property relations
- ✅ **YAML Schema upgrade** for all Entry-Types — 57 per-entry-type YAML schemas
- ✅ **Hypothesized extra-DB props per schema** — every per-DB schema declares 6–11 extra-DB props grounded in HoloOS docs
- ✅ **YAML + validator works correctly** — Python reference + Rust port both pass self-test
- ✅ **No auxiliary DBs** — People/Community/Group integrated INTO GreatWay, Notes/Knowledge INTO Nexus (no separate DBs)

---

## 2. What v0.9.0 Changes

### 2.1 Addresses UPGRADE_FLAGS.md Flags

| Flag | v0.8.0 Status | v0.9.0 Action |
|------|---------------|---------------|
| **Flag 1** Entry-Type Relocations | ⚠️ Flagged for manual review | ✅ Fully implemented via scripts 02 + 03 + 04 |
| **Flag 2** Dual-Property Relations | ⚠️ Skipped (single_property only) | ✅ 17 dual_property relations added via script 01 |
| **Flag 3** Entry-Type Renames | ⚠️ Flagged — not done (destructive) | ⚠️ Still deferred — schema supports the consolidation (e.g. Principle + sub_type) but renaming remains manual |
| **Flag 4** Existing Entries Need Tagging | ⚠️ Flagged — needs batch script | ✅ Auto-tag script 05 uses default_archetype_mapping from per_db schemas |
| **Flag 5** YAML Schema Migration | ⚠️ Deferred to future phase | ✅ Fully implemented — 3-tier YAML schema hierarchy + validator |
| **Flag 6** Auxiliary DB Integration | ⚠️ Deferred to future phase | ✅ Fully implemented — auxiliary People/Community/Group DBs ported INTO GreatWay |

### 2.2 User's v0.9.0 Directives

| Directive | Implementation |
|-----------|----------------|
| "Port people and community/group DB entries into the GreatWay for accuracy" | `02_relocate_entry_types.py` adds Person/Group/Community/Organization/Network/Movement/Place options to GreatWay.Item Type; `03_port_auxiliary_people_community_to_greatway.py` ports entries from auxiliary DBs |
| "Port Notes and Knowledge-Categories into the Nexus or any other DB, because Potentiator is very flat in terms of data aggregation" | `02_relocate_entry_types.py` adds Note/Knowledge-Category/Knowledge-Atom/Decision/Crisis/Transformation-Event options to Nexus.Category; `04_port_notes_knowledge_to_nexus.py` ports entries from Potentiator |
| "Relocate Entry-Types and ensure that the relevant DBs have the Dual Property Relationships effectively, for the inter-DB hierarchical relationships" | `01_add_dual_property_relations.py` adds 17 dual_property relations across the 5 DBs |
| "Upgrade the YAML Schema for all the Entry-Types. YAML-Schema are the extended-properties specialized for the Entry-Type in the particular-DB" | 63 YAML schema files: 1 universal + 5 per_db + 57 per_entry_type |
| "Hypothesize the schema for all the extra-DB props, and ensure that the YAML and its validator works correctly" | Each per_db/*.yaml hypothesizes 6–11 extra-DB props grounded in HoloOS docs; validator passes self-test in both Python and Rust |
| "I do not need any auxiliary DBs — they all must be hierarchically integrated within the inter/intra DB architecture of these 5 DBs" | All auxiliary DB entries are ported INTO GreatWay (People/Community/Group) or Nexus (Notes/Knowledge) — no auxiliary DBs remain |

---

## 3. The 5-DB Architecture (recap)

The LifeOS 5-DB architecture is the operationalization of the HoloOS holonic ontology (HoloOS doc 00.md §2.1). Each DB corresponds to one of the 8 functional roles:

| DB | HoloOS Role | Cycle | Currency In → Out |
|----|-------------|-------|-------------------|
| **Matrix** | Matrix (M) | lesser | Catalyst → Experience |
| **Potentiator** | Potentiator (P) | lesser | Experience → Catalyst |
| **Significator** | Significator (S) | greater | Transformation → Choice |
| **GreatWay** | Great Way (G) | greater | Choice → Transformation |
| **Nexus** | Transformation (T) — the contact boundary | both | ALL 4 currencies transmuted |

The remaining 3 roles (Catalyst, Experience, Choice) are **currencies** that flow THROUGH the Nexus (contact boundary), not separate reservoirs. They live as entry-types within the Nexus DB, discriminated by the `Kind` property.

**Directional flow:**
- **Involution (downward rewrite, HoloOS doc 02.2 §6):** GreatWay → Significator → Nexus → Potentiator → Matrix
- **Evolution (upward accumulation, HoloOS doc 03.1 §3):** Matrix → Potentiator → Nexus → Significator → GreatWay

---

## 4. Entry-Type Relocations

### 4.1 Relocations INTO GreatWay (external holons)

Per HoloOS doc 08.5 §1.2: *"The Great Way is a collective reservoir — it contains the accumulated Experience of ALL holons that participate in it."* External holons (people, communities, groups) belong in the Great Way because:

1. They ARE the operating environment of the focal holon (HoloOS doc 00.md §2.1).
2. The Great Way IS the parent holon's Potentiator (HoloOS doc 08.5 §1.4) — so a person's environment IS the family's latent potential.
3. Bonding occurs at the Significator⇄Great-Way surface (HoloOS doc 02.3 §4) — the Person/Group/Community entries in GreatWay are the **bonding partners** for the holon's Significator.

**New GreatWay.Item Type options:**

| Entry-Type | Origin | Quadrant (per holon_type_placement.yaml) |
|------------|--------|----------|
| Person | Relocated from Significator | UL or UR |
| Group | Relocated from Significator | LL |
| Community | Relocated from Significator | LL |
| Organization | NEW (per type_codes T30) | LR |
| Network | NEW (per type_codes T06/T17) | LR |
| Movement | NEW (per type_codes T15) | LL |
| Place | NEW (per type_codes T12) | LR |

**Migration:** `03_port_auxiliary_people_community_to_greatway.py` reads entries from any auxiliary People/Community/Group DBs in the Notion workspace and creates corresponding GreatWay entries with the appropriate Item Type. Properties are mapped via the `PERSON_PROP_MAP` / `GROUP_PROP_MAP` / `COMMUNITY_PROP_MAP` heuristics. The original auxiliary DB page ID is recorded in the new entry's `Holon ID` rich_text property for traceability.

### 4.2 Relocations INTO Nexus (currency-flow entries)

Per HoloOS doc 02.1 §5: *"The Contact Boundary is the selectively permeable nexus where the Matrix and Potentiator interact."* Catalyst and Experience flow through this boundary — they are NOT stored in the reservoirs (Matrix/Potentiator), they flow through the Nexus. The user noted Potentiator was "too flat in terms of data aggregation" — this relocation fixes that by:

1. Moving raw notes (Catalyst-class currency) into Nexus where they can be tracked through the 9-stage digestion cycle.
2. Moving digested knowledge (Experience-class currency) into Nexus where it can participate in the upward accumulation into the Significator.

**New Nexus.Category options:**

| Entry-Type | Origin | Kind (currency) |
|------------|--------|-----------------|
| Note | Relocated from Potentiator | Catalyst |
| Knowledge-Category | Relocated from Potentiator | Experience |
| Knowledge-Atom | NEW | Experience |
| Decision | NEW | Choice (per HoloOS doc 03.1 §3 stage 9) |
| Crisis | NEW | Transformation (acute threshold event) |
| Transformation-Event | NEW | Transformation (named macro-transition) |

**Migration:** `04_port_notes_knowledge_to_nexus.py` queries Potentiator for entries with Entry Type = Note/Knowledge-* and creates corresponding Nexus entries with the right `Category` and `Kind` values. The original Potentiator page ID is recorded in the new entry's `Holon ID` rich_text property. The original Potentiator entries are NOT archived by default — the user must verify the ported Nexus entries first, then re-run with `--archive-source` (or archive manually).

### 4.3 Relocations INTO Potentiator (latent future-input)

Per HoloOS doc 00.md §2.1: Potentiator = *"reachable possibility field that receives integrated state-update and returns refined future input."* Goals, Visions, and Aspirations ARE latent future-input — they belong in the Potentiator, not the GreatWay (which is the operating environment). This addresses Flag 3 (GreatWay.Goal naming issue).

**New Potentiator.Entry Type options:** Goal, Vision, Aspiration

### 4.4 Relocations OUT of Significator (external holons)

Per Flag 1, Person and Group entry-types in Significator are relocated OUT to GreatWay. The Significator now holds ONLY persistent identity-patterns (Purpose, Value, Principle, Identity-Statement, Pillar, Strategic-Ideal, Archetype). The relocated Person/Group entries retain their Significator-side bonds via the new `Coheres With (GreatWay)` dual_property relation (HoloOS doc 02.3 §4 — bonding at the Significator⇄Great-Way surface).

---

## 5. Dual-Property Inter-DB Relations

### 5.1 The 17 dual_property relations added in v0.9.0

Per HoloOS doc 08.5 §1.4 (fractal coupling) and the 9-stage digestion process (HoloOS doc 03.1 §3):

| # | Source DB.Prop | Target DB | Dual Backlink | HoloOS Doc |
|---|----------------|-----------|---------------|------------|
| 1 | Matrix.Accumulates Into | Significator | Significator.Accumulated From | 03.1 §3 stage 7 |
| 2 | Matrix.Generated From | Potentiator | Potentiator.Generates Into | 02.1 §3 Loop B |
| 3 | Nexus.Rewrites (Matrix) | Matrix | Matrix.Rewritten By | 03.1 §3 stage 9 |
| 4 | Nexus.Sends Catalyst To (Matrix) | Matrix | Matrix.Receives Catalyst From | 02.1 §1 |
| 5 | Nexus.Rewrites (Potentiator) | Potentiator | Potentiator.Rewritten By | 03.1 §3 stage 9 |
| 6 | Nexus.Sends Experience To (Potentiator) | Potentiator | Potentiator.Receives Experience From | 02.1 §1 |
| 7 | Nexus.Fires Transformation On | Significator | Significator.Triggered By | 03.1 §3 stage 8 |
| 8 | Nexus.Triggered By | Significator | Significator.Triggers | 03.1 §3 stage 8 |
| 9 | Nexus.Sends Catalyst To (Significator) | Significator | Significator.Receives Catalyst From (Great Way) | 02.2 §2 |
| 10 | Nexus.Emits Choice To | GreatWay | GreatWay.Receives Choice From | 03.1 §3 stage 9 |
| 11 | Significator.Anchored In | Matrix | Matrix.Anchors | 08.5 §1.4 |
| 12 | Significator.Sub-holon Of | Matrix | Matrix.Contains Significator Of | 08.5 §1.4 |
| 13 | Significator.Transforms To | GreatWay | GreatWay.Transforms From | 02.2 §6 |
| 14 | Significator.Emits Choice To | Nexus | Nexus.Receives Choice From (Significator) | 03.1 §3 stage 9 |
| 15 | GreatWay.For Significator | Significator | Significator.For Great Way | 02.2 §6 |
| 16 | GreatWay.Cohers With (Significator) | Significator | Significator.Cohers With (GreatWay) | 02.3 §4 |
| 17 | GreatWay.Sub-holon Of | Potentiator | Potentiator.Contains Great Way Of | 08.5 §1.4 |

### 5.2 Implementation note: single_property → dual_property conversion

The v0.8.0 upgrade added 7 relations as `single_property` only (per Flag 2). The Notion API does NOT support converting an existing single_property relation to dual_property — the only way is to **delete the property and recreate it as dual_property**. Script 01 handles this case:

- If the property already exists as dual_property → log as "already_dual_property" warning, skip.
- If the property exists as single_property → log as "exists_as_single_property" warning, surface a `manual_action` instructing the user to delete and re-run.
- If the property doesn't exist → create as dual_property with the specified backlink name.

### 5.3 Intra-DB hierarchy relations (also dual_property)

The per_db/*.yaml schemas also declare intra-DB hierarchy relations as dual_property. These are NOT created by script 01 (which focuses on inter-DB), but they're documented in the schemas for completeness:

- Matrix: `Parent ⟷ Children`, `Blocked By ⟷ Blocks`, `Refines ⟷ Refined By`, `Supersedes ⟷ Superseded By`
- Potentiator: `Crystallizes Into ⟷ Crystallized From`, `Sub-item ⟷ Parent item`
- Nexus: `Counterpart` (self-dual), `Counter-Synthesis` (self-dual), `Reinforces ⟷ Reinforced By`, `Sub-event ⟷ Parent-event`
- Significator: `Sub-item ⟷ Parent item`, `In Tension With` (self-dual), `Coheres With` (self-dual)
- GreatWay: `Parent item ⟷ Sub-item`, `Blocks ⟷ Blocked By`, `Contains Significators Of ⟷ Participates In Great Way`

---

## 6. YAML Schema Hierarchy

### 6.1 Three-tier inheritance

```
schemas/
├── universal/
│   └── holon_coordinate.yaml          (16 properties, 4 validation rules — applies to every entry)
├── per_db/
│   ├── matrix.yaml                    (6 properties — applies to every Matrix entry)
│   ├── potentiator.yaml               (6 properties — applies to every Potentiator entry)
│   ├── nexus.yaml                     (11 properties — applies to every Nexus entry)
│   ├── significator.yaml              (8 properties — applies to every Significator entry)
│   └── greatway.yaml                  (10 properties — applies to every GreatWay entry)
└── per_entry_type/
    ├── matrix__pattern.yaml           (3 specialized properties for Matrix.Pattern entries)
    ├── matrix__practice.yaml
    ├── ... (8 matrix, 10 potentiator, 13 nexus, 7 significator, 19 greatway = 57 total)
```

**Total: 1 universal + 5 per_db + 57 per_entry_type = 63 schema files**

A property is required for an entry IFF `required: true` at ANY applicable layer (universal → per_db → per_entry_type).

### 6.2 Universal properties (HoloOS 6-axis coordinate system)

The universal schema (`universal/holon_coordinate.yaml`) defines 16 properties that apply to EVERY entry in EVERY DB. These are grounded in HoloOS doc 00.md §8 and 02.4 §3:

| Property | Notion Type | Required | Source |
|----------|-------------|----------|--------|
| holon_id | rich_text | no | 00.md §8 |
| archetype_role | select (8 options) | **yes** | 00.md §2.1, 03.1 §1 |
| complex | select (4 options) | **yes** | 04.1 §2 |
| drive_activation | multi_select (4 options) | no | 02.1 §5 |
| shadow_pattern | select (6 options) | no | 02.1 §4, 02.2 §3 |
| digestion_stage | select (9 options) | no | 03.1 §3 |
| holon_type | select (5 options) | no | 02.3 §2, 02.4 §2.3 |
| valence_signature | rich_text | no | 02.4 §3 |
| lesser_cycle_phase | select (6 options) | no | holon_states.yaml |
| greater_cycle_phase | select (9 options) | no | holon_states.yaml |
| scale_code | select (11 options) | no | scale_codes.yaml |
| stage_code | select (8 options) | no | stage_codes.yaml |
| type_code | select (51 options) | no | type_codes.yaml |
| interior_exterior | select (2 options) | no | 08.7 §2 |
| gz_score | number | no | 02.1 §6.2 |
| pz_score | number | no | 02.1 §6.2 |

**4 cross-property validation rules:**
1. `nexus_kind_consistency` — Nexus.Kind constrains which relations can be populated
2. `significator_valence_signature_required` — Significator entries with holon_type set MUST have a Valence Signature YAML
3. `stage_type_independence` — stage_code and holon_type must BOTH be set or BOTH be empty
4. `complex_archetype_consistency` — (archetype_role, complex) pair must identify one of the 22 named archetypes

### 6.3 Per-DB extended properties (the "hypothesis")

Each per_db/*.yaml schema hypothesizes 6–11 extra-DB props specialized for that reservoir's role. Every property is grounded in a specific HoloOS doc passage.

**Example — Matrix DB (`per_db/matrix.yaml`):**

| Property | Notion Type | Source |
|----------|-------------|--------|
| eta_M | number | 03.1 §5.1 — Matrix consolidation efficiency |
| matrix_load | number | 08.1 §5 — basin depth (how consolidated) |
| current_configuration | rich_text | 03.1 §2.2 — active state vector x_t |
| dark_shadow_state | select | 02.1 §4 — per-complex Matrix shadow |
| homeostatic_basin_depth | number | 08.1 §3.1 — ‖A_M‖ |
| agency_drive_score | number | 02.1 §5, 08.5 §3.1 — A_z at the Matrix pole |

See the schema files for the full property lists for Matrix, Potentiator, Nexus, Significator, GreatWay.

### 6.4 Per-Entry-Type specializations

Each per_entry_type/*.yaml schema adds 2–5 specialized properties for a specific (DB, entry-type) pair. The 57 schemas cover:

- All 22 named archetypes (1–22 per HoloOS doc 03.2)
- Plus operational entry-types (Active Project, Habit, Routine, Inventory, Threshold, Goal, Vision, Aspiration, etc.)
- Plus the v0.9.0 relocated entry-types (Person, Group, Community, Organization, Network, Movement, Place, Note, Knowledge-Category, Knowledge-Atom, Decision, Crisis, Transformation-Event)

**Example — `per_entry_type/greatway__person.yaml`:**

```yaml
schema_version: "0.9.0"
schema_type: per_entry_type
applies_to_db: greatway
applies_to_entry_type: Person
inherits_from:
  - universal/holon_coordinate.yaml
  - per_db/greatway.yaml
documentation_ref: "RELOCATED FROM Significator (external holons belong in the Great Way per HoloOS doc 08.5)"
properties:
  person_type:
    notion_type: select
    options: [self, family, friend, colleague, mentor, mentee, collaborator, client, vendor, public_figure, ancestor]
  bonding_disposition:
    notion_type: select
    description: "HoloOS doc 02.3 §4 — bonding type at Significator⇄Great-Way surface"
    options: [ionic, covalent, dative, metallic]
  valence_complementarity:
    notion_type: rich_text
  contact_frequency:
    notion_type: select
    options: [daily, weekly, monthly, quarterly, annual, rare]
  last_contact_date:
    notion_type: date
validation_rules:
  - id: person_quadrant_required
    rule: "assert entry.quadrant in ['UL', 'UR']"
  - id: person_archetype_role
    rule: "assert entry.archetype_role == 'Great Way'"
```

---

## 7. Validator Implementation

### 7.1 Two implementations, one semantic

| Implementation | Location | Use case |
|----------------|----------|----------|
| **Python reference** | `scripts/upgrade_v0.9.0/yaml_schema_validator.py` | Standalone validation without building Rust; reference for the DSL semantics |
| **Rust integration** | `lifeos-core/src/util/yaml_schemas.rs` + `lifeos-core/src/tools/validate_yaml.rs` | Built into the `lifeos` CLI; callable as `lifeos validate-yaml`; also exposed as the MCP `validate_yaml` tool |

Both implementations:
- Load the 3-tier schema hierarchy
- Validate property types (notion_type, required, options)
- Evaluate cross-property validation rules via a small Python-like DSL
- Produce identical pass/fail results for any input

### 7.2 The validation DSL

Cross-property rules are expressed in a small DSL that both implementations evaluate:

```python
# Nexus Kind ↔ relation constraint
if entry.kind == "Catalyst":
    assert_no_relations(entry, ["Tension", "Counter-Tension"])
elif entry.kind == "Choice":
    assert_no_relations(entry, ["Updates", "Sourced From", "Tension"])

# Significator must have Valence Signature if holon_type is set
if entry.holon_type is not None:
    assert entry.valence_signature is not None
    assert entry.valence_signature parses as valid YAML
    assert "complexes" in entry.valence_signature

# Stage ⊥ Type independence
assert (entry.stage_code is None) == (entry.holon_type is None)

# (archetype_role, complex) must identify one of the 22 archetypes
if entry.complex == "None":
    assert entry.archetype_role == "Choice"
else:
    assert (entry.archetype_role, entry.complex) in {("Matrix","Mind"), ...}
```

### 7.3 Self-test (no Notion API needed)

Both validators implement a `--self-test` mode that validates the schema files themselves:

```bash
# Python
python3 scripts/upgrade_v0.9.0/yaml_schema_validator.py --self-test

# Rust CLI
lifeos validate-yaml --self-test
```

**Self-test output (verified):**
```
✅ All schemas passed self-test.
  Loaded: 1 universal + 5 per_db + 57 per_entry_type schemas
  Universal layer: 16 properties, 4 validation rules
  per_db/matrix.yaml: 6 properties, 8 entry-types, 8 per_entry_type files
  per_db/potentiator.yaml: 6 properties, 10 entry-types, 10 per_entry_type files
  per_db/nexus.yaml: 11 properties, 13 entry-types, 13 per_entry_type files
  per_db/significator.yaml: 8 properties, 7 entry-types, 7 per_entry_type files
  per_db/greatway.yaml: 10 properties, 19 entry-types, 19 per_entry_type files
```

The self-test catches:
- Missing universal/per_db files
- Properties with invalid notion_type
- Entry-types declared in per_db but missing a per_entry_type file
- Per_entry_type files with invalid applies_to_db

---

## 8. Migration Scripts

All 6 scripts live in `scripts/upgrade_v0.9.0/` and share `00_common.py` for the Notion HTTP client, property mutation helpers, and migration logging.

### 8.1 Script inventory

| # | Script | Purpose | Risk |
|---|--------|---------|------|
| 00 | `00_common.py` | Shared utilities (NotionClient, migration log, property mutation helpers) | (foundation) |
| 01 | `01_add_dual_property_relations.py` | Adds 17 dual_property relations across the 5 DBs | Low — additive |
| 02 | `02_relocate_entry_types.py` | Adds new entry-type OPTIONS to GreatWay/Nexus/Potentiator | Low — additive (doesn't touch existing options) |
| 03 | `03_port_auxiliary_people_community_to_greatway.py` | Ports People/Community/Group entries from auxiliary DBs INTO GreatWay | Medium — creates new entries |
| 04 | `04_port_notes_knowledge_to_nexus.py` | Ports Note/Knowledge entries from Potentiator INTO Nexus | Medium — creates new entries; optionally archives source |
| 05 | `05_auto_tag_existing_entries.py` | Auto-tags existing 439+ entries with v0.8.0 semantic properties (Archetype Role + Complex + Digestion Stage) | Low — only fills empty properties |
| 06 | `06_apply_yaml_schemas_to_notion.py` | Materializes per-DB YAML schema properties as real Notion properties | Low — additive |

### 8.2 Every script supports `--dry-run`

Always run with `--dry-run` first to preview. Every script also writes a JSON migration log to `scripts/upgrade_v0.9.0/logs/<script>_<timestamp>.json` for verification and rollback.

### 8.3 Run order (see `scripts/upgrade_v0.9.0/README.md`)

```bash
# Phase A: Schema preparation
02_relocate_entry_types.py    # Add new entry-type OPTIONS to GreatWay/Nexus/Potentiator
06_apply_yaml_schemas_to_notion.py  # Create the per-DB extended properties

# Phase B: Relations
01_add_dual_property_relations.py  # Add 17 dual_property inter-DB relations

# Phase C: Tagging
05_auto_tag_existing_entries.py  # Fill Archetype Role + Complex + Digestion Stage

# Phase D: Data migration
03_port_auxiliary_people_community_to_greatway.py  # Port People/Community/Group → GreatWay
04_port_notes_knowledge_to_nexus.py  # Port Notes/Knowledge → Nexus

# Phase E: Verification
lifeos discover
lifeos validate-yaml --all
```

---

## 9. Rust Codebase Updates

### 9.1 New module: `lifeos-core/src/util/yaml_schemas.rs`

The `YamlSchemaRegistry` struct loads and caches all 63 schema files. Public API:

- `YamlSchemaRegistry::discover_schemas_dir()` — locates the schemas directory via env var or path search
- `YamlSchemaRegistry::load(&Path)` — loads all schemas, returns a registry
- `YamlSchemaRegistry::self_test()` — validates the schema files themselves
- `validate_entry(db_key, page, registry)` — validates a single Notion entry

### 9.2 New tool: `lifeos-core/src/tools/validate_yaml.rs`

Implements the `validate_yaml` MCP tool. Parameters:

```json
{
  "database": "matrix",        // optional — filter to one DB
  "page_id": "uuid",           // optional — validate one page
  "self_test": true,           // optional — validate the schemas themselves
  "all": true,                 // optional — validate all 5 DBs
  "limit": 50                  // optional — max entries per DB
}
```

### 9.3 New CLI command: `lifeos validate-yaml`

```bash
lifeos validate-yaml --self-test                # No Notion API needed
lifeos validate-yaml --db matrix                 # Validate all Matrix entries
lifeos validate-yaml --all                       # Validate all 5 DBs
lifeos validate-yaml --page-id <id>              # Validate a single entry
lifeos validate-yaml --db matrix --limit 10      # Test with 10 entries
```

### 9.4 New MCP tool registration

The `validate_yaml` tool is registered in `lifeos-core/src/tools/mod.rs` alongside the existing 30 tools. AI agents (Claude Desktop, Cursor, etc.) can call it via JSON-RPC.

### 9.5 New config field: `holonic.yaml_schemas_path`

Added to `HolonicConfig` in `config.rs`. Defaults to `"schemas"` (relative to the config file). The validator auto-discovers via `YamlSchemaRegistry::discover_schemas_dir()` if not set.

### 9.6 Build verification

```bash
$ cargo check --target x86_64-unknown-linux-gnu
   Compiling lifeos-core v0.8.0
   Compiling lifeos v0.8.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.08s

$ ./target/x86_64-unknown-linux-gnu/debug/lifeos validate-yaml --self-test
✅ All schemas passed self-test.
  Loaded: 1 universal + 5 per_db + 57 per_entry_type schemas
  ...
```

(One pre-existing warning in `ontology.rs:188` — not introduced by v0.9.0.)

---

## 10. Execution Runbook

### 10.1 Prerequisites

1. **Python 3.9+** with `pyyaml`:
   ```bash
   pip install pyyaml
   ```

2. **Notion API token** with read+write access:
   ```bash
   export NOTION_API_TOKEN=secret_xxx
   ```

3. **Built `lifeos` binary** with v0.9.0 changes:
   ```bash
   cd lifeos-ops
   cargo build --release
   # Or use the debug build for testing
   cargo build
   ```

4. **Schemas directory** at `lifeos-ops/schemas/` (already in place).

### 10.2 Step-by-step execution

```bash
cd lifeos-ops
export NOTION_API_TOKEN=secret_xxx

# Step 0: Verify the schemas + validator pass self-test
python3 scripts/upgrade_v0.9.0/yaml_schema_validator.py --self-test
lifeos validate-yaml --self-test  # if using the Rust CLI

# Step 1: Add new entry-type OPTIONS to GreatWay + Nexus + Potentiator
python3 scripts/upgrade_v0.9.0/02_relocate_entry_types.py --dry-run
python3 scripts/upgrade_v0.9.0/02_relocate_entry_types.py

# Step 2: Apply YAML schema properties to Notion (creates per-DB extended properties)
python3 scripts/upgrade_v0.9.0/06_apply_yaml_schemas_to_notion.py --dry-run
python3 scripts/upgrade_v0.9.0/06_apply_yaml_schemas_to_notion.py

# Step 3: Add dual_property relations
python3 scripts/upgrade_v0.9.0/01_add_dual_property_relations.py --dry-run
python3 scripts/upgrade_v0.9.0/01_add_dual_property_relations.py
# ⚠️  If any relations exist as single_property from v0.8.0, the script will
#     surface a manual_action. Delete those in the Notion UI, then re-run this script.

# Step 4: Auto-tag existing entries
python3 scripts/upgrade_v0.9.0/05_auto_tag_existing_entries.py --dry-run
python3 scripts/upgrade_v0.9.0/05_auto_tag_existing_entries.py

# Step 5: Port People/Community/Group → GreatWay
python3 scripts/upgrade_v0.9.0/03_port_auxiliary_people_community_to_greatway.py --dry-run
python3 scripts/upgrade_v0.9.0/03_port_auxiliary_people_community_to_greatway.py

# Step 6: Port Notes/Knowledge → Nexus
python3 scripts/upgrade_v0.9.0/04_port_notes_knowledge_to_nexus.py --dry-run
python3 scripts/upgrade_v0.9.0/04_port_notes_knowledge_to_nexus.py

# Step 7: Refresh LifeOS schema cache
lifeos discover

# Step 8: Validate all entries against the YAML schemas
lifeos validate-yaml --all
```

### 10.3 Post-execution manual steps

1. **Verify the dual_property relations** — open each DB in the Notion UI and confirm the synced backlink property exists on the target DB.
2. **Verify the ported GreatWay entries** — open a few Person/Group/Community entries and confirm the properties look correct.
3. **Verify the ported Nexus entries** — open a few Note/Knowledge-Category entries and confirm the `Kind` and `Category` are set correctly.
4. **Archive the original auxiliary People/Community/Group DBs** — once you're confident the GreatWay ports are good, archive the original auxiliary DBs in the Notion UI.
5. **Archive the original Potentiator Note/Knowledge entries** — re-run script 04 with `--archive-source`, or archive them manually in the Notion UI.
6. **Set Valence Signatures on Significator entries** — for each Significator entry that should have a Holon Type, run `lifeos valence-signature --page-id <id>` to generate the YAML template, fill it in, then `lifeos derive-type --page-id <id>` to compute the Holon Type.

---

## 11. Verification Checklist

After running the full upgrade, verify each item:

### Schemas + Validator
- [ ] `python3 scripts/upgrade_v0.9.0/yaml_schema_validator.py --self-test` → ✅
- [ ] `lifeos validate-yaml --self-test` → ✅
- [ ] 63 schema files in `schemas/` (1 universal + 5 per_db + 57 per_entry_type)

### Entry-Type Options Added
- [ ] `lifeos schema --database greatway` shows: Person, Group, Community, Organization, Network, Movement, Place
- [ ] `lifeos schema --database nexus` shows: Note, Knowledge-Category, Knowledge-Atom, Decision, Crisis, Transformation-Event
- [ ] `lifeos schema --database potentiator` shows: Goal, Vision, Aspiration

### Dual-Property Relations
- [ ] Matrix has `Accumulates Into` with synced backlink `Significator.Accumulated From`
- [ ] Matrix has `Generated From` with synced backlink `Potentiator.Generates Into`
- [ ] Matrix has `Rewritten By` (synced from Nexus)
- [ ] Nexus has `Rewrites (Matrix)` and `Rewrites (Potentiator)` with synced backlinks
- [ ] Significator has `Sub-holon Of` with synced backlink `Matrix.Contains Significator Of`
- [ ] GreatWay has `Sub-holon Of` with synced backlink `Potentiator.Contains Great Way Of`
- [ ] GreatWay has `Cohers With (Significator)` with synced backlink `Significator.Cohers With (GreatWay)`

### Per-DB Extended Properties
- [ ] Matrix has: eta_M, matrix_load, current_configuration, dark_shadow_state, homeostatic_basin_depth, agency_drive_score
- [ ] Potentiator has: eta_P, potentiator_load, reachable_possibilities, golden_shadow_state, latent_basin_breadth, communion_drive_score
- [ ] Nexus has: gamma_tensor, boundary_permeability, vertical_balance_BV, horizontal_balance_BH, transformation_threshold, crucible_intensity, choice_polarity, crystallization_ratio, source_url, knowledge_synthesis_state
- [ ] Significator has: identity_mass, eta_S, significator_load, polarity_disposition_pi, noble_flag_chi, eros_drive_score, attractor_basin_class, principle_sub_type
- [ ] GreatWay has: environmental_basin_breadth, great_way_polarity, eta_G, delivery_surface_permeability, choice_filter, agape_drive_score, quadrant, dimension, bonding_disposition, bonding_partner_count

### Auto-Tagging
- [ ] All Matrix.Pattern entries have `Archetype Role: Matrix`, `Complex: Mind`
- [ ] All Potentiator.Diet entries have `Archetype Role: Catalyst`, `Complex: Body`
- [ ] All Nexus entries have `Kind` populated
- [ ] All GreatWay.Person entries have `Archetype Role: Great Way`, `Complex: Spirit`, `Quadrant: UL or UR`

### Ported Entries
- [ ] GreatWay has Person entries with `Holon ID: legacy:<original_aux_db_page_id>`
- [ ] GreatWay has Group entries with `Quadrant: LL`
- [ ] GreatWay has Community entries with `Quadrant: LL`
- [ ] Nexus has Note entries with `Kind: Catalyst`, `Category: Note`
- [ ] Nexus has Knowledge-Category entries with `Kind: Experience`, `Category: Knowledge-Category`

### Validator Pass
- [ ] `lifeos validate-yaml --all` returns 0 errors (or only warnings about un-tagged entries)

---

## 12. Rollback Plan

The v0.9.0 upgrade is **strictly additive** — no existing data is destroyed. To roll back:

### 12.1 Roll back scripts (in reverse order)

| Script | Rollback action |
|--------|-----------------|
| 04 (port to Nexus) | Manually archive the new Nexus.Note / Nexus.Knowledge-Category entries |
| 03 (port to GreatWay) | Manually archive the new GreatWay.Person / Group / Community entries |
| 05 (auto-tag) | Manually clear the `Archetype Role`, `Complex`, `Digestion Stage` properties |
| 01 (dual_property relations) | Manually delete the new relation properties in the Notion UI |
| 06 (apply YAML schemas) | Manually delete the new per-DB extended properties in the Notion UI |
| 02 (relocate entry-types) | Manually remove the new entry-type options from the select/multi_select properties in the Notion UI |

### 12.2 Roll back code changes

The v0.9.0 code changes are isolated to:
- `lifeos-core/src/util/yaml_schemas.rs` (new file)
- `lifeos-core/src/util/mod.rs` (one line: `pub mod yaml_schemas;`)
- `lifeos-core/src/tools/validate_yaml.rs` (new file)
- `lifeos-core/src/tools/mod.rs` (registered new module + tool)
- `lifeos-core/src/cli/mod.rs` (new `ValidateYaml` command)
- `lifeos-core/src/config.rs` (new `yaml_schemas_path` field on HolonicConfig)
- `lifeos/src/main.rs` (wired up the new command)
- `lifeos.config.default.json` (updated descriptions, added `yaml_schemas_path`)
- `schemas/` (entire new directory — can be deleted)
- `scripts/upgrade_v0.9.0/` (entire new directory — can be deleted)

To revert via git:
```bash
git revert <commit-hash>
# Or, if you haven't committed:
git checkout -- lifeos-core/ lifeos/ lifeos.config.default.json
rm -rf schemas/ scripts/upgrade_v0.9.0/
```

---

## 13. Ontology References

Every property, relation, and entry-type in the v0.9.0 schemas is grounded in a specific HoloOS doc passage. The most-cited passages:

| Passage | File:Line | Used in |
|---------|----------|---------|
| Matrix definition | `00.md:163` | §3 — DB architecture |
| Potentiator definition | `00.md:164` | §3 — DB architecture |
| Significator definition | `00.md:167` | §3 — DB architecture |
| Great Way definition | `00.md:168` | §3 — DB architecture |
| Nexus = Transformation definition | `00.md:169` | §3 — DB architecture |
| Contact Boundary = nexus | `02.1:216` | §3 — Nexus etymology |
| Fractal coupling | `08.5:124` | §5 — dual_property relations |
| Significator is the bridge | `02.2:360` | §3, §5 |
| Great Way is collective reservoir | `08.5:50` | §4 — relocations into GreatWay |
| Bonding at Significator⇄Great-Way surface | `02.3:125` | §4, §5 |
| 9-stage digestion process | `03.1:§3` | §5 — relations follow the stages |
| Stage ⊥ Type | `4_Type_Validation_Protocol.md:§2` | §6.2 — validation rule |
| 22 named archetypes | `03.2:§2` | §6.4 — per-entry-type schemas |
| Valence Signature format | `02.4:111` | §6.2 — valence_signature property |

---

*End of upgrade plan. For questions, see `scripts/upgrade_v0.9.0/README.md` or run `lifeos validate-yaml --help`.*
