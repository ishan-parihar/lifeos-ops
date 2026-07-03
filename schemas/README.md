# LifeOS v0.9.0 — YAML Schema Directory

Lean 3-tier schema hierarchy for the LifeOS Notion system.

## Architecture

```
schemas/
├── universal/
│   └── holon_coordinate.yaml          (6 props, 3 rules — every entry)
├── per_db/
│   ├── matrix.yaml                    (relations + entry_types only)
│   ├── potentiator.yaml
│   ├── nexus.yaml
│   ├── significator.yaml
│   └── greatway.yaml
└── per_entry_type/                    (31 files — relocated entry-types + validation rules)
```

**Total: 37 schema files** (down from 63 after ponytail audit).

## Universal Properties (6)

These are the ONLY Notion properties on all 5 DBs. Everything else is YAML-schema-only.

| Property | Type | Required | Purpose |
|----------|------|----------|---------|
| Archetype Role | select (8) | yes | One of 8 functional roles |
| Complex | select (4) | yes | Mind/Body/Spirit/None |
| Drive Activation | multi_select (4) | no | Active drives |
| Shadow Pattern | select (6) | no | Shadow state |
| Digestion Stage | select (9) | no | 9-stage process (Nexus+Potentiator) |
| Holon Type | select (5) | no | Valence-class type (Significator) |

## Validation Rules (3, hardcoded in Rust)

1. `nexus_kind_consistency` — Nexus.Kind constrains which relations can be populated
2. `stage_type_independence` — stage_code and holon_type must both be set or both empty
3. `complex_archetype_consistency` — (archetype_role, complex) must be one of the 22 named archetypes

## How to Validate

```bash
lifeos validate-yaml --self-test     # Verify schemas load
lifeos validate-yaml --all           # Validate all entries (requires NOTION_API_TOKEN)
lifeos validate-yaml --db matrix     # Validate one DB
lifeos validate-yaml --page-id <id>  # Validate one entry
```

## Design Principle

**If a property has 0% fill rate after 30 days of real use, it gets deleted.** Schema tracks what IS used, not what COULD theoretically be used. Entry-type sub-typing uses Notion formulas or CLI automations, not manual select properties.
