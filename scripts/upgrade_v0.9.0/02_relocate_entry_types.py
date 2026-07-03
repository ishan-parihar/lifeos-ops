#!/usr/bin/env python3
"""
LifeOS v0.9.0 — Script 02: Relocate Entry-Types
================================================

Addresses UPGRADE_FLAGS.md Flag 1 + the user's v0.9.0 relocation directive:

  PORT INTO GreatWay:
    - Person        (from Significator — external holons belong in the Great Way)
    - Group         (from Significator)
    - Community     (from Significator)
    - Organization  (NEW — formal collective holons per HoloOS doc 08.5 §1.2)
    - Network       (NEW — informal collective holons)
    - Movement      (NEW — ideological collective holons per type_codes T15)
    - Place         (NEW — geographic operating environment per type_codes T12)

  PORT INTO Nexus:
    - Note                  (from Potentiator — raw notes are Catalyst-class currency)
    - Knowledge-Category    (from Potentiator — digested notes are Experience-class currency)
    - Knowledge-Atom        (NEW — single synthesized knowledge unit)
    - Decision              (NEW — Choice-class currency per HoloOS doc 03.1 §3 stage 9)
    - Crisis                (NEW — Transformation-kind per HoloOS doc 03.1 §3 stage 8)
    - Transformation-Event  (NEW — named macro-transition)

WHAT THIS SCRIPT DOES:
  1. Discovers the 5 LifeOS DBs.
  2. For GreatWay: adds new options to the "Item Type" select property:
     Person, Group, Community, Organization, Network, Movement, Place
  3. For Nexus: adds new options to the "Category" select property:
     Note, Knowledge-Category, Knowledge-Atom, Decision, Crisis, Transformation-Event
  4. (Optional) For Potentiator: keeps Note/Knowledge-Category options for
     backward-compat (does NOT delete — the user can archive later).
  5. (Optional) For Significator: keeps Person/Group options for backward-compat.
  6. Saves a migration log.

NOTE on the relocation strategy:
  The Notion API does NOT support moving a page from one database to another
  via a single call. The actual porting of ENTRIES (the data itself) is done
  by scripts 03 (People/Community/Group → GreatWay) and 04 (Notes/Knowledge
  → Nexus). This script ONLY adds the new entry-type OPTIONS so the target
  DBs are ready to receive the ported entries.

USAGE:
    NOTION_API_TOKEN=your_token python3 02_relocate_entry_types.py [--dry-run]
"""

from __future__ import annotations

import argparse
import sys
import time
from datetime import datetime
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from importlib import import_module
common = import_module("common")

from common import (NotionClient, discover_db_ids, get_database_container_id,
                     get_data_source_schema,
                     add_select_option_to_existing_property, print_section, print_kv, MigrationLog)


# ─────────────────────────────────────────────────────────────────────────────
# New entry-type options to add to each DB
# ─────────────────────────────────────────────────────────────────────────────

# Per the user's directive + UPGRADE_FLAGS.md Flag 1
ENTRY_TYPE_RELOCATIONS = [
    # ── GreatWay: external holons (relocated from Significator + new) ──
    {
        "db": "greatway",
        "prop_name": "Item Type",   # per lifeos.config.default.json
        "prop_type": "select",
        "new_options": [
            "Person",          # relocated from Significator
            "Group",           # relocated from Significator
            "Community",       # relocated from Significator
            "Organization",    # new — HoloOS doc 08.5 §1.2
            "Network",         # new — informal collective holons
            "Movement",        # new — HoloOS doc type_codes T15
            "Place",           # new — HoloOS doc type_codes T12
        ],
        "rationale": "External holons belong in the Great Way (HoloOS doc 08.5). "
                     "Bonding occurs at the Significator⇄Great-Way surface (HoloOS doc 02.3 §4).",
    },

    # ── Nexus: notes + knowledge + decisions (relocated from Potentiator + new) ──
    {
        "db": "nexus",
        "prop_name": "Category",    # per lifeos.config.default.json
        "prop_type": "select",
        "new_options": [
            "Note",                  # relocated from Potentiator — Catalyst-kind currency
            "Knowledge-Category",    # relocated from Potentiator — Experience-kind currency
            "Knowledge-Atom",        # new — single synthesized knowledge unit
            "Decision",              # new — Choice-kind currency (HoloOS doc 03.1 §3 stage 9)
            "Crisis",                # new — Transformation-kind (acute threshold event)
            "Transformation-Event",  # new — named macro-transition
        ],
        "rationale": "Notes (Catalyst-class) and Knowledge (Experience-class) flow through "
                     "the contact boundary, which IS the Nexus (HoloOS doc 02.1 §5). The "
                     "Potentiator DB was 'too flat in terms of data aggregation' (user "
                     "directive) — moving these to Nexus enables proper currency-flow tracking.",
    },

    # ── Potentiator: add Goal/Vision/Aspiration (relocated FROM GreatWay per Flag 3) ──
    # Per HoloOS doc 00.md §2.1: Potentiator = "reachable possibility field that receives
    # integrated state-update and returns refined future input." Goals/Visions/Aspirations
    # ARE latent future-input — they belong in Potentiator, not GreatWay.
    {
        "db": "potentiator",
        "prop_name": "Entry Type",
        "prop_type": "select",
        "new_options": [
            "Goal",         # relocated from GreatWay — latent future-input
            "Vision",       # relocated from GreatWay — latent future-input
            "Aspiration",   # relocated from GreatWay — latent future-input
        ],
        "rationale": "Goals/Visions/Aspirations are latent future-input — they belong in the "
                     "Potentiator (HoloOS doc 00.md §2.1), not the GreatWay (which is the "
                     "operating environment). Addresses Flag 3 (GreatWay.Goal naming issue).",
    },
]


# ─────────────────────────────────────────────────────────────────────────────
# Main logic
# ─────────────────────────────────────────────────────────────────────────────

def add_entry_type_options(
    client: NotionClient,
    db_ids: dict[str, str],
    db_key: str,
    prop_name: str,
    prop_type: str,
    new_options: list[str],
    log: MigrationLog,
    dry_run: bool = False,
) -> dict:
    """Add new options to a select/multi_select property on a Notion DB."""
    op = {
        "type": "add_entry_type_options",
        "db": db_key,
        "prop_name": prop_name,
        "prop_type": prop_type,
        "new_options": new_options,
    }

    ds_id = db_ids.get(db_key)
    if not ds_id:
        op["status"] = "skipped"
        op["error"] = f"DB '{db_key}' not discovered."
        log.errors.append(op)
        return op

    op["data_source_id"] = ds_id

    # Fetch current schema from the DATA_SOURCE (API 2025-09-03: properties live on data_source, not database container)
    try:
        schema = get_data_source_schema(client, ds_id)
    except Exception as e:
        op["status"] = "error"
        op["error"] = f"Failed to fetch schema for {db_key}: {e}"
        log.errors.append(op)
        return op

    existing_prop = schema.get(prop_name)
    if not existing_prop:
        op["status"] = "error"
        op["error"] = f"Property '{prop_name}' not found on DB '{db_key}'. Available: {list(schema.keys())}"
        log.errors.append(op)
        return op

    # Check the property type matches
    actual_type = existing_prop.get("type")
    if actual_type != prop_type:
        op["status"] = "error"
        op["error"] = f"Property '{prop_name}' on '{db_key}' is type '{actual_type}', expected '{prop_type}'."
        log.errors.append(op)
        return op

    existing_options = [o.get("name") for o in existing_prop.get(prop_type, {}).get("options", [])]
    op["existing_options"] = existing_options
    truly_new = [o for o in new_options if o not in existing_options]
    op["truly_new_options"] = truly_new

    if not truly_new:
        op["status"] = "already_present"
        op["message"] = f"All {len(new_options)} options already exist on {db_key}.{prop_name}."
        log.warnings.append(op)
        return op

    if dry_run:
        op["status"] = "dry_run"
        op["message"] = f"Would add {len(truly_new)} new options to {db_key}.{prop_name}: {truly_new}"
        log.operations.append(op)
        return op

    try:
        result = add_select_option_to_existing_property(
            client=client,
            data_source_id=ds_id,
            prop_name=prop_name,
            new_options=truly_new,
            prop_type=prop_type,
        )
        op["status"] = "created"
        op["added_options"] = truly_new
        op["message"] = f"Added {len(truly_new)} new options to {db_key}.{prop_name}: {truly_new}"
        log.operations.append(op)
        return op
    except Exception as e:
        op["status"] = "error"
        op["error"] = str(e)
        log.errors.append(op)
        return op


def main() -> int:
    parser = argparse.ArgumentParser(description="LifeOS v0.9.0 — Relocate Entry-Types")
    parser.add_argument("--dry-run", action="store_true",
                        help="Show what would be done without making any API calls.")
    args = parser.parse_args()

    print_section("LifeOS v0.9.0 — Script 02: Relocate Entry-Types")
    print_kv("Mode", "DRY RUN" if args.dry_run else "LIVE EXECUTION")
    print_kv("Started at", datetime.utcnow().isoformat() + "Z")
    print_kv("DBs to update", len(ENTRY_TYPE_RELOCATIONS))

    client = NotionClient()

    print_section("Step 1: Discover the 5 LifeOS DBs")
    db_ids = discover_db_ids(client)
    for k, v in db_ids.items():
        if not k.startswith("aux:"):
            print_kv(k, v)
    missing = [db for db in ["matrix", "potentiator", "nexus", "significator", "greatway"] if db not in db_ids]
    if missing:
        print(f"\n❌ FATAL: Missing required DBs: {missing}", file=sys.stderr)
        return 1

    log = MigrationLog(
        script_name="02_relocate_entry_types",
        started_at=datetime.utcnow().isoformat() + "Z",
    )

    print_section("Step 2: Add new entry-type options to target DBs")
    for i, spec in enumerate(ENTRY_TYPE_RELOCATIONS, 1):
        print(f"\n[{i:02d}/{len(ENTRY_TYPE_RELOCATIONS)}] {spec['db']}.{spec['prop_name']}  ({spec['prop_type']})")
        print(f"     Rationale: {spec['rationale']}")
        print(f"     New options: {spec['new_options']}")
        result = add_entry_type_options(
            client=client,
            db_ids=db_ids,
            db_key=spec["db"],
            prop_name=spec["prop_name"],
            prop_type=spec["prop_type"],
            new_options=spec["new_options"],
            log=log,
            dry_run=args.dry_run,
        )
        print(f"     Status: {result['status']}")
        if "message" in result:
            print(f"     {result['message']}")
        if not args.dry_run:
            time.sleep(0.5)

    log.finished_at = datetime.utcnow().isoformat() + "Z"
    log_path = Path(__file__).parent / "logs" / f"02_relocate_entry_types_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.json"
    log.save(log_path)

    print_section("Summary")
    print_kv("Operations (created)", sum(1 for o in log.operations if o.get("status") == "created"))
    print_kv("Operations (dry_run)", sum(1 for o in log.operations if o.get("status") == "dry_run"))
    print_kv("Warnings (already present)", len(log.warnings))
    print_kv("Errors", len(log.errors))

    if log.errors:
        print("\n❌ Some operations failed — see log for details.")
        return 1
    if not args.dry_run and not log.errors:
        print("\n✅ All entry-type relocations completed. Run scripts 03 and 04 next to port the actual entries.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
