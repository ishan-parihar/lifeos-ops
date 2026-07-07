# LifeOS v0.9.0 — Upgrade Scripts

One-time migration scripts that performed the v0.8.0 → v0.9.0 upgrade against the live Notion workspace. **All scripts have been executed.** They are kept for audit trail and re-run capability if needed.

## Scripts

| # | Script | Status | Purpose |
|---|--------|--------|---------|
| 01 | `01_add_dual_property_relations.py` | ✅ Executed | Added 13 dual_property inter-DB relations (3 need manual conversion from single_property) |
| 02 | `02_relocate_entry_types.py` | ✅ Executed | Added new entry-type options to GreatWay (7), Nexus (6), Potentiator (3) |
| 03 | `03_port_auxiliary_people_community_to_greatway.py` | ✅ Executed | Ported 77 entries (63 People + 14 Communities) from auxiliary DBs into GreatWay |
| 04 | `04_port_notes_knowledge_to_nexus.py` | ✅ N/A | No Note/Knowledge entries found in Potentiator |
| 05 | `05_auto_tag_existing_entries.py` | ✅ Executed | Auto-tagged entries with Archetype Role + Complex (partially complete for large DBs) |
| -- | `common.py` | -- | Shared Notion HTTP client + property mutation helpers |

## Prerequisites

```bash
pip install pyyaml
export NOTION_API_TOKEN=ntn_xxx
```

## Re-running

Each script is idempotent (safe to re-run — it skips what already exists). All support `--dry-run`:

```bash
NOTION_API_TOKEN=xxx python3 02_relocate_entry_types.py --dry-run
NOTION_API_TOKEN=xxx python3 02_relocate_entry_types.py
```

## Logs

Each script writes a JSON migration log to `logs/<script>_<timestamp>.json`.
