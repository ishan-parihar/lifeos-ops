# LifeOS v0.9.0 — YAML Schema Directory

This directory contains the **3-tier YAML schema hierarchy** for the LifeOS Notion system, grounded in the HoloOS holonic ontology (`HoloOS/_THEORY/02_Ontology/`).

## Schema Hierarchy

```
schemas/
├── universal/
│   └── holon_coordinate.yaml          ← applies to EVERY entry in EVERY DB
├── per_db/
│   ├── matrix.yaml                    ← applies to every Matrix entry
│   ├── potentiator.yaml               ← applies to every Potentiator entry
│   ├── nexus.yaml                     ← applies to every Nexus entry
│   ├── significator.yaml              ← applies to every Significator entry
│   └── greatway.yaml                  ← applies to every GreatWay entry
└── per_entry_type/
    ├── matrix__pattern.yaml           ← applies to Matrix entries with Entry Type = Pattern
    ├── matrix__practice.yaml
    ├── ... (57 files total)
    └── greatway__place.yaml
```

A property is required for an entry IFF `required: true` at ANY applicable layer (universal → per_db → per_entry_type).

## Schema File Format

Every schema file has this structure:

```yaml
schema_version: "0.9.0"
schema_type: universal | per_db | per_entry_type
applies_to_db: <db_key>              # for per_db and per_entry_type only
applies_to_entry_type: <entry_type>  # for per_entry_type only
inherits_from:                       # for per_entry_type only
  - universal/holon_coordinate.yaml
  - per_db/<db>.yaml
documentation_ref: "<HoloOS doc path>"

properties:
  <prop_name>:
    notion_type: <select|multi_select|status|rich_text|number|date|...>
    required: true|false
    description: |
      Multi-line description citing the HoloOS doc passage that grounds this property.
    options:                          # for select/multi_select/status only
      - <option_1>
      - <option_2>

relations:                            # for per_db only — intra-DB hierarchy relations
  - name: <prop_name>
    target_db: <db_key>
    direction: dual_property
    dual_backlink: <backlink_name>
    cardinality: one_to_many | many_to_many | ...
    description: <relation semantics>

inter_db_relations:                   # for per_db only — cross-DB relations
  - name: <prop_name>
    target_db: <db_key>
    direction: dual_property
    dual_backlink: <backlink_name>
    cardinality: <cardinality>
    description: <relation semantics>

entry_types:                          # for per_db only — declared entry-types
  - <entry_type_1>
  - <entry_type_2>

default_archetype_mapping:            # for per_db only — used by auto-tag script 05
  <entry_type>:
    archetype_role: <role>
    complex: <Mind|Body|Spirit|None>

validation_rules:                     # cross-property rules (any layer)
  - id: <rule_id>
    description: <rule_description>
    applies_to_db: <db_key>           # optional — restrict rule to one DB
    rule: |
      <Python-like DSL expression>
```

## Validation DSL

The `validation_rules.rule` field uses a small Python-like DSL. Supported patterns:

```python
# Pattern 1: if/elif/else with assert_no_relations
if entry.kind == "Catalyst":
    assert_no_relations(entry, ["Tension", "Counter-Tension"])
elif entry.kind == "Choice":
    assert_no_relations(entry, ["Updates", "Sourced From", "Tension"])

# Pattern 2: if with assert + YAML parse check
if entry.holon_type is not None:
    assert entry.valence_signature is not None
    assert entry.valence_signature parses as valid YAML
    assert "complexes" in entry.valence_signature

# Pattern 3: bare assert with == for both sides
assert (entry.stage_code is None) == (entry.holon_type is None)

# Pattern 4: bare assert with in {...} or [...]
assert entry.complex in {"Mind", "Body", "Spirit", "None"}
assert entry.archetype_role in ["Matrix", "Potentiator", ...]
```

Both the Python reference validator (`scripts/upgrade_v0.9.0/yaml_schema_validator.py`) and the Rust validator (`lifeos-core/src/util/yaml_schemas.rs`) evaluate this DSL.

## File Counts (verified by `lifeos validate-yaml --self-test`)

| Layer | Files | Properties (per file) | Validation Rules |
|-------|-------|-----------------------|------------------|
| universal | 1 | 16 | 4 |
| per_db | 5 | 6–11 each | 0 (rules are in per_entry_type) |
| per_entry_type | 57 | 2–5 each | 0–2 each |
| **Total** | **63** | — | — |

## Per-DB Entry-Type Counts

| DB | Declared entry-types | per_entry_type files |
|----|----------------------|---------------------|
| matrix | 8 (Pattern, Practice, Inventory, Foundation, Threshold, Active Project, Habit, Routine) | 8 |
| potentiator | 10 (Activity, Subjective, Relational, Systemic, Diet, Financial, Observation, Goal, Vision, Aspiration) | 10 |
| nexus | 13 (Opportunity, Insight, Reflection, Integration, Pattern, Risk, Directive, Note, Knowledge-Category, Knowledge-Atom, Decision, Crisis, Transformation-Event) | 13 |
| significator | 7 (Purpose, Value, Principle, Identity-Statement, Pillar, Strategic-Ideal, Archetype) | 7 |
| greatway | 19 (Annual Goal, Quarterly Goal, Goal, Project, Task, System, Resource, Sprint, Milestone, Budget, Campaign, Content, Person, Group, Community, Organization, Network, Movement, Place) | 19 |

## Relocations (v0.9.0)

| Entry-Type | From | To | Rationale |
|-----------|------|-----|-----------|
| Person | Significator | GreatWay | External holons belong in the Great Way (HoloOS doc 08.5) |
| Group | Significator | GreatWay | Same as Person |
| Community | Significator | GreatWay | Same as Person |
| Organization | (new) | GreatWay | Formal collective holons (HoloOS doc type_codes T30) |
| Network | (new) | GreatWay | Informal collective holons |
| Movement | (new) | GreatWay | Ideological collective holons (HoloOS doc type_codes T15) |
| Place | (new) | GreatWay | Geographic operating environment (HoloOS doc type_codes T12) |
| Note | Potentiator | Nexus | Raw notes are Catalyst-class currency (HoloOS doc 02.1 §1) |
| Knowledge-Category | Potentiator | Nexus | Digested notes are Experience-class currency |
| Knowledge-Atom | (new) | Nexus | Single synthesized knowledge unit |
| Decision | (new) | Nexus | Choice-class currency (HoloOS doc 03.1 §3 stage 9) |
| Crisis | (new) | Nexus | Transformation-kind (acute threshold event) |
| Transformation-Event | (new) | Nexus | Named macro-transition |
| Goal | GreatWay | Potentiator | Goals are latent future-input (HoloOS doc 00.md §2.1) — addresses Flag 3 |
| Vision | GreatWay | Potentiator | Same as Goal |
| Aspiration | GreatWay | Potentiator | Same as Goal |

## Dual-Property Relations (v0.9.0)

17 dual_property relations added across the 5 DBs. See `UPGRADE_v0.9.0_PLAN.md` §5 for the complete list with HoloOS doc references.

## How to Validate

```bash
# Self-test (no Notion API needed) — verifies the schemas themselves
lifeos validate-yaml --self-test

# Validate all entries in a DB
NOTION_API_TOKEN=xxx lifeos validate-yaml --db matrix

# Validate all entries in all 5 DBs
NOTION_API_TOKEN=xxx lifeos validate-yaml --all

# Validate a single entry by page ID
NOTION_API_TOKEN=xxx lifeos validate-yaml --page-id <uuid>

# Python reference validator (same semantics)
python3 scripts/upgrade_v0.9.0/yaml_schema_validator.py --self-test
NOTION_API_TOKEN=xxx python3 scripts/upgrade_v0.9.0/yaml_schema_validator.py --all
```

## Ontology Grounding

Every property, relation, and entry-type in these schemas is grounded in a specific HoloOS doc passage. The `description` field of each property cites the source. Key references:

- `HoloOS/_THEORY/02_Ontology/00.md` — system-theory terminology contract (8 functional roles)
- `HoloOS/_THEORY/02_Ontology/02.1_Microcosmic_Metabolic_Architecture.md` — lesser cycle, drives, shadows
- `HoloOS/_THEORY/02_Ontology/02.2_Macrocosmic_Metabolic_Architecture.md` — greater cycle, Significator as bridge
- `HoloOS/_THEORY/02_Ontology/02.3_Holonic_Typology_Derivator.md` — 5 Holon Types (Donor/Acceptor/Sharer/Multivalent/Noble)
- `HoloOS/_THEORY/02_Ontology/02.4_Significator_Valence_and_Type.md` — Valence Signature format
- `HoloOS/_THEORY/02_Ontology/03.1_Universal_Archetype_Anatomy.md` — 9-stage digestion process
- `HoloOS/_THEORY/02_Ontology/03.2_22_Named_Archetypes_Index.md` — the 22 named archetypes
- `HoloOS/_THEORY/02_Ontology/04.2.1/2/3` — Mind/Body/Spirit complex architectures
- `HoloOS/_THEORY/02_Ontology/08.5_Extra_Holonic_Deepening.md` — fractal coupling (parent/child holon relations)
- `HoloOS/_THEORY/02_Ontology/08.7_Interior_Exterior_vs_Substrate_Superstrate.md` — I/E vs Sub/Sup disambiguation
- `HoloOS/_THEORY/01_Epistemology/4_Type_Validation_Protocol.md` — Type ⊥ Stage independence
- `HoloOS/_INSTRUMENTS/schemas/taxonomy/type_codes.yaml` — 51 concrete holon types (T01–T51)
- `HoloOS/_INSTRUMENTS/schemas/taxonomy/scale_codes.yaml` — organizational scales (S00–S80)
- `HoloOS/_INSTRUMENTS/schemas/taxonomy/holon_type_placement.yaml` — quadrant + dimension per type
