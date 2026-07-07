# LifeOS v0.9.0 — Operational Runbook

**Status:** ✅ All scripts executed. Foundations set.

## What v0.9.0 Delivered

| Deliverable | Status |
|-------------|--------|
| YAML schema hierarchy (universal → per_db → per_entry_type) | ✅ 37 files |
| Rust validator (`lifeos validate-yaml`) | ✅ Compiles, self-test passes |
| 13 dual_property inter-DB relations | ✅ Created in Notion |
| Entry-type relocations (Person/Group/Community → GreatWay; Note/Knowledge → Nexus) | ✅ Options added |
| 77 entries ported (63 People + 14 Communities → GreatWay) | ✅ Complete |
| Auto-tagging (Archetype Role + Complex) | ✅ ~884 entries tagged |

## Post-Audit Property Counts

After the ponytail audit cleanup, each DB has a manageable property count:

| DB | Properties (was → now) |
|----|----------------------|
| Matrix | 34 → 25 |
| Potentiator | 36 → 27 |
| Nexus | 44 → 35 |
| Significator | 40 → 29 |
| GreatWay | 49 → 26 |
| **Total** | **203 → 142 (30% reduction)** |

## Remaining Manual Actions

1. **Convert 3 single_property → dual_property** in Notion UI:
   - Matrix.Accumulates Into → delete + re-run script 01
   - Matrix.Generated From → delete + re-run script 01
   - Significator.Anchored In → delete + re-run script 01

2. **Complete auto-tagging** for remaining Potentiator entries (~6700 untagged):
   ```bash
   NOTION_API_TOKEN=xxx python3 scripts/upgrade_v0.9.0/05_auto_tag_existing_entries.py --db potentiator
   ```

3. **Archive original auxiliary DBs** (People + Community) after verifying GreatWay ports

4. **Set Valence Signatures** on Significator entries + run `lifeos derive-type` to compute Holon Types

## Verification Commands

```bash
lifeos validate-yaml --self-test     # Verify schemas load
lifeos discover                       # Refresh schema cache from Notion
lifeos validate-yaml --all            # Validate all entries
lifeos schema --database greatway     # Check GreatWay properties
```

## Schema Architecture (lean)

```
schemas/
├── universal/holon_coordinate.yaml    (6 props, 3 validation rules)
├── per_db/{matrix,potentiator,nexus,significator,greatway}.yaml
│   (0 props — only relations + entry_types + default_archetype_mapping)
└── per_entry_type/*.yaml              (31 files — only for relocated entry-types
                                        or those with validation rules)
```

**Universal properties (on all 5 DBs):** Archetype Role, Complex, Drive Activation, Shadow Pattern, Digestion Stage (Nexus+Potentiator), Holon Type (Significator)

**Rule:** If a property has 0% fill rate after 30 days of real use, it gets deleted. Schema tracks what IS used, not what COULD theoretically be used.
