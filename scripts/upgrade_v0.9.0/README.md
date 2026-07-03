# LifeOS v0.9.0 — Upgrade Scripts

This directory contains the Python scripts that perform the LifeOS v0.9.0 upgrade against your live Notion workspace. Each script addresses one or more of the upgrade directives from `UPGRADE_FLAGS.md` and the user's v0.9.0 directive.

## Prerequisites

1. **Python 3.9+** with `pyyaml` installed:
   ```bash
   pip install pyyaml
   ```

2. **Notion API token** with read+write access to your LifeOS workspace:
   ```bash
   export NOTION_API_TOKEN=secret_xxx
   ```

3. **Run order matters** — the scripts build on each other:
   - Script 02 (relocate entry-types) MUST run before scripts 03 + 04 (port entries)
   - Script 06 (apply schemas) MUST run before the YAML validator can fully check entry schemas
   - Script 01 (dual_property relations) is independent — can run anytime
   - Script 05 (auto-tag) is independent — can run anytime after v0.8.0 (which added the semantic properties)

## Run Order

The recommended run order for a fresh upgrade from v0.8.0 → v0.9.0:

```bash
# Step 1: Add new entry-type OPTIONS to GreatWay + Nexus + Potentiator
#         (must run BEFORE porting entries — target DBs need the options)
NOTION_API_TOKEN=xxx python3 02_relocate_entry_types.py --dry-run
NOTION_API_TOKEN=xxx python3 02_relocate_entry_types.py

# Step 2: Apply YAML schema properties to Notion
#         (creates the per-DB extended properties like eta_M, gamma_tensor, etc.)
NOTION_API_TOKEN=xxx python3 06_apply_yaml_schemas_to_notion.py --dry-run
NOTION_API_TOKEN=xxx python3 06_apply_yaml_schemas_to_notion.py

# Step 3: Add dual_property relations
#         (converts the 7 single_property relations from v0.8.0 to dual_property
#          and adds new inter-DB hierarchical relations)
NOTION_API_TOKEN=xxx python3 01_add_dual_property_relations.py --dry-run
NOTION_API_TOKEN=xxx python3 01_add_dual_property_relations.py

# Step 4: Auto-tag existing entries with the v0.8.0 semantic properties
#         (Flag 4 — populates Archetype Role + Complex + Digestion Stage
#          based on each entry's entry-type, using the default_archetype_mapping
#          from the per_db/*.yaml schemas)
NOTION_API_TOKEN=xxx python3 05_auto_tag_existing_entries.py --dry-run
NOTION_API_TOKEN=xxx python3 05_auto_tag_existing_entries.py

# Step 5: Port People/Community/Group entries from auxiliary DBs INTO GreatWay
NOTION_API_TOKEN=xxx python3 03_port_auxiliary_people_community_to_greatway.py --dry-run
NOTION_API_TOKEN=xxx python3 03_port_auxiliary_people_community_to_greatway.py

# Step 6: Port Notes/Knowledge-Categories FROM Potentiator INTO Nexus
#         (the user noted "Potentiator is very flat in terms of data aggregation" —
#          this relocates the Note/Knowledge entries to Nexus where they fit
#          the Catalyst/Experience currency-flow ontology)
NOTION_API_TOKEN=xxx python3 04_port_notes_knowledge_to_nexus.py --dry-run
NOTION_API_TOKEN=xxx python3 04_port_notes_knowledge_to_nexus.py
# Optional: archive the originals after verifying
# NOTION_API_TOKEN=xxx python3 04_port_notes_knowledge_to_nexus.py --archive-source

# Step 7: Refresh the LifeOS schema cache + validate
lifeos discover
lifeos validate-yaml --all
```

## Scripts

| # | Script | Purpose | Addresses |
|---|--------|---------|-----------|
| 00 | `00_common.py` | Shared Notion HTTP client, migration log, property mutation helpers | (foundation) |
| 01 | `01_add_dual_property_relations.py` | Adds 17 dual_property relations across the 5 DBs | Flag 2 |
| 02 | `02_relocate_entry_types.py` | Adds new entry-type OPTIONS to GreatWay (Person/Group/Community/Org/Network/Movement/Place) + Nexus (Note/Knowledge-Category/Knowledge-Atom/Decision/Crisis/Transformation-Event) + Potentiator (Goal/Vision/Aspiration) | Flag 1 + user directive |
| 03 | `03_port_auxiliary_people_community_to_greatway.py` | Ports entries from auxiliary People/Community/Group DBs INTO GreatWay | User directive |
| 04 | `04_port_notes_knowledge_to_nexus.py` | Ports Note/Knowledge entries from Potentiator INTO Nexus (where Catalyst/Experience currencies flow) | User directive |
| 05 | `05_auto_tag_existing_entries.py` | Auto-tags all 439+ existing entries with the v0.8.0 semantic properties (Archetype Role + Complex + Digestion Stage) using the default_archetype_mapping | Flag 4 |
| 06 | `06_apply_yaml_schemas_to_notion.py` | Materializes the per-DB YAML schema properties as real Notion properties | Flag 5 |
| -- | `generate_per_entry_type_schemas.py` | (Already run) generates the 57 per-entry-type YAML schemas | Phase 1 |
| -- | `yaml_schema_validator.py` | The Python reference validator (validates entries against the 3-tier YAML schema hierarchy) | Flag 5 |

## Logs

Each script writes a JSON migration log to `logs/<script>_<timestamp>.json`. Keep these logs for verification and rollback planning. They record:

- Every operation attempted (created / dry_run / skipped / error)
- Every warning (e.g. property already exists, entry has no title)
- Every error (with the Notion API response body for debugging)
- The exact timestamps and durations

## Dry-Run Mode

**Always run with `--dry-run` first.** Every script supports `--dry-run` to preview what would be done without making any API calls. The dry-run output shows:

- What entries/properties/relations WOULD be created
- What already exists (skipped)
- What requires manual action (e.g. existing single_property relations can't be auto-converted — see Script 01)

## Rollback

The scripts are **additive** — they create new properties, new entry-type options, new relations, and new ported entries. They do NOT delete or archive existing data, with ONE exception:

- Script 04 (`--archive-source` flag) archives the original Potentiator entries after porting to Nexus. **Do not use this flag until you have verified the ported Nexus entries.**

To roll back any other script:

- **Script 01 (dual_property relations)**: Manually delete the new relation properties in the Notion UI.
- **Script 02 (entry-type options)**: Manually remove the new options from the select/multi_select property in the Notion UI.
- **Script 03 (port to GreatWay)**: Manually archive the new GreatWay entries.
- **Script 05 (auto-tag)**: Manually clear the `Archetype Role`, `Complex`, `Digestion Stage` properties.
- **Script 06 (apply YAML schemas)**: Manually delete the new properties in the Notion UI.

## Verification

After running all scripts, verify the upgrade:

```bash
# 1. Refresh schema cache
lifeos discover

# 2. View the new schema (should show all per-DB extended properties)
lifeos schema --database matrix
lifeos schema --database nexus
lifeos schema --database greatway

# 3. Validate all entries against the YAML schemas
lifeos validate-yaml --all

# 4. Check that dual_property relations exist
lifeos schema --database matrix   # should show "Accumulates Into" with dual backlink "Accumulated From"

# 5. Check the ported entries
lifeos query greatway --entry-type Person
lifeos query nexus --entry-type Note
lifeos query nexus --entry-type Knowledge-Category
```

## Ontology References

Every property, relation, and entry-type in the schemas is grounded in a specific HoloOS doc passage. See:

- `schemas/universal/holon_coordinate.yaml` — universal properties (16 properties, 4 validation rules)
- `schemas/per_db/*.yaml` — per-DB extended properties + relations + entry-type lists
- `schemas/per_entry_type/*.yaml` — 57 per-entry-type specializations (one per (DB, entry-type) pair)
- `UPGRADE_v0.9.0_PLAN.md` — the comprehensive design document

Each schema file's `description` field cites the exact HoloOS doc passage that grounds it.
