#!/usr/bin/env python3
"""
LifeOS v0.9.0 — Script 01: Add Dual-Property Relations to Notion
=================================================================

Addresses UPGRADE_FLAGS.md Flag 2 — "Dual-Property Relations (NOT ADDED)".

The v0.8.0 upgrade added 7 new relation properties as single_property only.
This script converts the proposed inter-DB hierarchical relations to
dual_property (synced backlink) relations, per the HoloOS ontology's
fractal-coupling architecture (HoloOS doc 08.5 §1.4).

WHAT THIS SCRIPT DOES:
  1. Discovers the 5 LifeOS DBs by name in your Notion workspace.
  2. For each proposed dual_property relation:
     a. Checks if the relation already exists on the source DB.
     b. If it exists as single_property: prints a warning (Notion API
        doesn't support converting relation types — would need to delete
        and recreate, which requires manual verification).
     c. If it doesn't exist: creates it as dual_property with the
        specified backlink name on the target DB.
  3. Saves a migration log for verification.

RELATIONS TO ADD (per schemas/per_db/*.yaml `inter_db_relations` lists):

  Matrix → Significator:
    Accumulates Into  ⟷  Significator.Accumulated From

  Matrix → Potentiator:
    Generated From  ⟷  Potentiator.Generates Into

  Nexus → Matrix:
    Rewrites (Matrix)  ⟷  Matrix.Rewritten By
    Sends Catalyst To (Matrix)  ⟷  Matrix.Receives Catalyst From

  Nexus → Potentiator:
    Rewrites (Potentiator)  ⟷  Potentiator.Rewritten By
    Sends Experience To (Potentiator)  ⟷  Potentiator.Receives Experience From

  Nexus → Significator:
    Fires Transformation On  ⟷  Significator.Triggered By
    Triggered By  ⟷  Significator.Triggers
    Sends Catalyst To (Significator)  ⟷  Significator.Receives Catalyst From (Great Way)

  Nexus → GreatWay:
    Emits Choice To  ⟷  GreatWay.Receives Choice From

  Significator → Matrix:
    Anchored In  ⟷  Matrix.Anchors
    Sub-holon Of  ⟷  Matrix.Contains Significator Of

  Significator → GreatWay:
    Transforms To  ⟷  GreatWay.Transforms From

  Significator → Nexus:
    Emits Choice To  ⟷  Nexus.Receives Choice From (Significator)

  GreatWay → Significator:
    For Significator  ⟷  Significator.For Great Way
    Coheres With (Significator)  ⟷  Significator.Cohers With (GreatWay)

  GreatWay → Potentiator:
    Sub-holon Of  ⟷  Potentiator.Contains Great Way Of

USAGE:
    NOTION_API_TOKEN=your_token python3 01_add_dual_property_relations.py [--dry-run]

The --dry-run flag lists what WOULD be done without making any API calls.
"""

from __future__ import annotations

import argparse
import sys
import time
from datetime import datetime
from pathlib import Path

# Add parent directory to path for common module import
sys.path.insert(0, str(Path(__file__).parent))
from importlib import import_module
common = import_module("common")

from common import (NotionClient, discover_db_ids, get_database_container_id,
                     add_relation_property, print_section, print_kv, MigrationLog)


# ─────────────────────────────────────────────────────────────────────────────
# The proposed dual_property relations, per the per_db schemas.
# Each tuple: (source_db, prop_name, target_db, dual_backlink_name, notes)
# ─────────────────────────────────────────────────────────────────────────────

DUAL_PROPERTY_RELATIONS = [
    # ── Matrix → Significator (HoloOS doc 03.1 §3 stage 7) ──
    ("matrix", "Accumulates Into", "significator", "Accumulated From",
     "Stage 7: Experience accumulates upward from Matrix into Significator."),

    # ── Matrix → Potentiator (HoloOS doc 02.1 §3 Loop B) ──
    ("matrix", "Generated From", "potentiator", "Generates Into",
     "Loop B: Potentiator returns refined Catalyst to Matrix."),

    # ── Nexus → Matrix (HoloOS doc 03.1 §3 stages 4, 9) ──
    ("nexus", "Rewrites (Matrix)", "matrix", "Rewritten By",
     "Stage 9: Transformation fires, downward rewrite of Matrix."),
    ("nexus", "Sends Catalyst To (Matrix)", "matrix", "Receives Catalyst From",
     "Stage 2-3: Catalyst-kind Nexus entry carries Catalyst into Matrix."),

    # ── Nexus → Potentiator (HoloOS doc 03.1 §3 stages 6, 9) ──
    ("nexus", "Rewrites (Potentiator)", "potentiator", "Rewritten By",
     "Stage 9: Transformation fires, downward rewrite of Potentiator."),
    ("nexus", "Sends Experience To (Potentiator)", "potentiator", "Receives Experience From",
     "Stage 5-6: Experience-kind Nexus entry carries Experience into Potentiator."),

    # ── Nexus → Significator (HoloOS doc 03.1 §3 stages 7, 8, 9) ──
    ("nexus", "Fires Transformation On", "significator", "Triggered By",
     "Stage 8: Transformation-kind Nexus entry acts on Significator."),
    ("nexus", "Triggered By", "significator", "Triggers",
     "Stage 8: Significator's threshold-crossing triggered this Nexus entry."),
    ("nexus", "Sends Catalyst To (Significator)", "significator", "Receives Catalyst From (Great Way)",
     "Stage 2: Catalyst-kind Nexus entry carries Great-Way Catalyst to Significator."),

    # ── Nexus → GreatWay (HoloOS doc 03.1 §3 stage 9) ──
    ("nexus", "Emits Choice To", "greatway", "Receives Choice From",
     "Stage 9: Choice-kind Nexus entry carries Choice into Great Way."),

    # ── Significator → Matrix (HoloOS doc 08.5 §1.4) ──
    ("significator", "Anchored In", "matrix", "Anchors",
     "Fractal coupling: S of sub-holon = M of parent."),
    ("significator", "Sub-holon Of", "matrix", "Contains Significator Of",
     "Fractal coupling: Significator is a sub-holon within parent's Matrix."),

    # ── Significator → GreatWay (HoloOS doc 02.2 §6) ──
    ("significator", "Transforms To", "greatway", "Transforms From",
     "Greater cycle: Significator faces Transformation pressure from Great Way."),

    # ── Significator → Nexus (HoloOS doc 03.1 §3 stage 9) ──
    ("significator", "Emits Choice To", "nexus", "Receives Choice From (Significator)",
     "Stage 9: Significator emits Choice through Nexus entry."),

    # ── GreatWay → Significator (HoloOS doc 02.2 §6, 02.3 §4) ──
    ("greatway", "For Significator", "significator", "For Great Way",
     "Greater cycle: GreatWay entry exerts pressure on Significator."),
    ("greatway", "Coheres With (Significator)", "significator", "Coheres With (GreatWay)",
     "Bonding: Significator⇄GreatWay surface bonding (ionic/covalent/dative/metallic)."),

    # ── GreatWay → Potentiator (HoloOS doc 08.5 §1.4) ──
    ("greatway", "Sub-holon Of", "potentiator", "Contains Great Way Of",
     "Fractal coupling: G of sub-holon = P of parent."),
]


# ─────────────────────────────────────────────────────────────────────────────
# Main logic
# ─────────────────────────────────────────────────────────────────────────────

def find_existing_relation(db_schema: dict, prop_name: str) -> dict | None:
    """Look up a relation property by name on a database schema."""
    props = db_schema.get("properties", {})
    return props.get(prop_name)


def is_dual_property(rel_prop: dict) -> bool:
    """Check if a relation property is dual_property or single_property."""
    rel = rel_prop.get("relation", {})
    return rel.get("type") == "dual_property"


def add_dual_relation(
    client: NotionClient,
    db_ids: dict[str, str],
    source_db: str,
    prop_name: str,
    target_db: str,
    dual_backlink_name: str,
    log: MigrationLog,
    notes: str = "",
    dry_run: bool = False,
) -> dict:
    """Add a single dual_property relation. Returns the operation record."""
    op = {
        "type": "add_dual_property_relation",
        "source_db": source_db,
        "prop_name": prop_name,
        "target_db": target_db,
        "dual_backlink_name": dual_backlink_name,
        "notes": notes,
    }

    # Discover source + target DB container IDs
    src_ds_id = db_ids.get(source_db)
    tgt_ds_id = db_ids.get(target_db)
    if not src_ds_id or not tgt_ds_id:
        op["status"] = "skipped"
        op["error"] = f"Source or target DB not discovered: source={source_db}({src_ds_id}), target={target_db}({tgt_ds_id})"
        log.errors.append(op)
        return op

    src_db_container = get_database_container_id(client, src_ds_id)
    tgt_db_container = get_database_container_id(client, tgt_ds_id)
    op["src_db_container_id"] = src_db_container
    op["tgt_db_container_id"] = tgt_db_container

    # Fetch source DB schema and check if the property already exists
    src_schema = client.get_database(src_db_container)
    existing = find_existing_relation(src_schema, prop_name)

    if existing:
        if is_dual_property(existing):
            op["status"] = "already_dual_property"
            op["message"] = f"Property '{prop_name}' on {source_db} is already dual_property."
            log.warnings.append(op)
            return op
        else:
            op["status"] = "exists_as_single_property"
            op["message"] = (
                f"Property '{prop_name}' on {source_db} exists as single_property. "
                "Notion API does NOT support converting relation types — "
                "you must delete and recreate it manually in the Notion UI."
            )
            op["manual_action"] = (
                f"1. Open {source_db} in Notion UI. "
                f"2. Delete the '{prop_name}' relation property. "
                f"3. Re-run this script (it will create '{prop_name}' as dual_property)."
            )
            log.warnings.append(op)
            return op

    # Create the dual_property relation
    if dry_run:
        op["status"] = "dry_run"
        op["message"] = f"Would create dual_property '{prop_name}' on {source_db} → {target_db} (backlink: '{dual_backlink_name}')."
        log.operations.append(op)
        return op

    try:
        result = add_relation_property(
            client=client,
            database_container_id=src_db_container,
            prop_name=prop_name,
            target_database_id=tgt_db_container,
            dual_property=True,
            dual_property_name=dual_backlink_name,
        )
        op["status"] = "created"
        op["result_id"] = result.get("id")
        op["message"] = f"Created dual_property '{prop_name}' on {source_db} → {target_db} (backlink: '{dual_backlink_name}')."
        log.operations.append(op)
        return op
    except Exception as e:
        op["status"] = "error"
        op["error"] = str(e)
        log.errors.append(op)
        return op


def main() -> int:
    parser = argparse.ArgumentParser(description="LifeOS v0.9.0 — Add Dual-Property Relations to Notion")
    parser.add_argument("--dry-run", action="store_true",
                        help="Show what would be done without making any API calls.")
    args = parser.parse_args()

    print_section("LifeOS v0.9.0 — Script 01: Add Dual-Property Relations")
    print_kv("Mode", "DRY RUN" if args.dry_run else "LIVE EXECUTION")
    print_kv("Started at", datetime.utcnow().isoformat() + "Z")
    print_kv("Relations to add", len(DUAL_PROPERTY_RELATIONS))

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
        script_name="01_add_dual_property_relations",
        started_at=datetime.utcnow().isoformat() + "Z",
    )

    print_section("Step 2: Add dual_property relations")
    for i, (src, prop, tgt, backlink, notes) in enumerate(DUAL_PROPERTY_RELATIONS, 1):
        print(f"\n[{i:02d}/{len(DUAL_PROPERTY_RELATIONS)}] {src}.{prop}  ⟷  {tgt}.{backlink}")
        print(f"     Notes: {notes}")
        result = add_dual_relation(client, db_ids, src, prop, tgt, backlink, log, notes, dry_run=args.dry_run)
        print(f"     Status: {result['status']}")
        if "message" in result:
            print(f"     {result['message']}")
        if "manual_action" in result:
            print(f"     ⚠️  MANUAL ACTION REQUIRED:")
            print(f"     {result['manual_action']}")
        # Brief pause to be polite to the Notion API
        if not args.dry_run:
            time.sleep(0.5)

    log.finished_at = datetime.utcnow().isoformat() + "Z"
    log_path = Path(__file__).parent / "logs" / f"01_dual_property_relations_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.json"
    log.save(log_path)

    print_section("Summary")
    print_kv("Operations (created)", sum(1 for o in log.operations if o.get("status") == "created"))
    print_kv("Operations (dry_run)", sum(1 for o in log.operations if o.get("status") == "dry_run"))
    print_kv("Warnings (already_dual / exists_as_single)", len(log.warnings))
    print_kv("Errors", len(log.errors))

    if log.errors:
        print("\n❌ Some operations failed — see log for details.")
        return 1
    if log.warnings and not args.dry_run:
        print("\n⚠️  Some relations require manual action — see warnings above.")
    if not args.dry_run and not log.warnings:
        print("\n✅ All dual_property relations added successfully.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
