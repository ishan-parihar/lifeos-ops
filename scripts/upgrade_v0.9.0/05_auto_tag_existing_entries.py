#!/usr/bin/env python3
"""
LifeOS v0.9.0 — Script 05: Auto-Tag Existing Entries with v0.8.0 Semantic Properties
====================================================================================

Addresses UPGRADE_FLAGS.md Flag 4 — "Existing Entries Need Tagging".

The v0.8.0 upgrade added 24 new semantic typing properties (Archetype Role,
Complex, Drive Activation, Shadow Pattern, Digestion Stage, Holon Type,
Valence Signature) to all 5 DBs, but left them EMPTY. This script reads the
`default_archetype_mapping` from each per_db/*.yaml schema file and uses it
to populate `archetype_role` + `complex` for every existing entry, based on
the entry's entry_type.

WHAT THIS SCRIPT DOES:
  1. Loads the per_db/*.yaml schemas (which contain `default_archetype_mapping`
     tables mapping each entry_type to its default archetype_role + complex).
  2. Discovers the 5 LifeOS DBs.
  3. For each DB, queries all entries and:
     a. Reads the entry_type discriminator (Entry Type / Item Type / Category).
     b. Looks up the default (archetype_role, complex) for that entry_type.
     c. Updates the entry's `Archetype Role` and `Complex` properties.
     d. Optionally sets `Digestion Stage` based on the entry's current Status
        (Matrix.Active → "1 - Latent State", Potentiator.Crystallized → "7 - Significator Accumulation", etc.)
  4. Saves a migration log.

NOTE: This script does NOT set Holon Type or Valence Signature — those
require per-entry analysis (Holon Type is derived FROM the Valence Signature
per `lifeos derive-type`). The user can set Valence Signatures manually,
then run `lifeos derive-type --page-id <id>` for each Significator entry.

USAGE:
    NOTION_API_TOKEN=your_token python3 05_auto_tag_existing_entries.py [--dry-run] [--db matrix]

    --db: Only tag entries in the specified DB. Can be repeated.
    --dry-run: Show what would be done without making any API calls.
"""

from __future__ import annotations

import argparse
import sys
import time
from datetime import datetime
from pathlib import Path

import yaml

sys.path.insert(0, str(Path(__file__).parent))
from importlib import import_module
common = import_module("00_common")

from common import (NotionClient, discover_db_ids, get_database_container_id,
                     print_section, print_kv, MigrationLog)

SCHEMAS_DIR = Path(__file__).resolve().parents[2] / "schemas"

# ─────────────────────────────────────────────────────────────────────────────
# Status → Digestion Stage mapping (heuristic)
# ─────────────────────────────────────────────────────────────────────────────

STATUS_TO_DIGESTION = {
    # Matrix statuses
    "Active": "1 - Latent State",
    "Evolving": "8 - Transformation Threshold",
    "Archived": "9 - Choice & Rewrite",
    # Potentiator statuses
    "Raw": "2 - Boundary Contact",
    "Digesting": "4 - Matrix Digestion",
    "Crystallized": "6 - Potentiator Digestion",
    # Significator statuses
    "Draft": "1 - Latent State",
    # GreatWay statuses
    "Future": "1 - Latent State",
    "Ideation": "2 - Boundary Contact",
    "Paused": "3 - Matrix Ingestion",
    "Done": "9 - Choice & Rewrite",
    "Cancelled": "9 - Choice & Rewrite",
    # Nexus statuses (with emoji prefix variants)
    "Identified": "1 - Latent State",
    "Activated": "5 - Potentiator Ingestion",
    "Capitalized": "9 - Choice & Rewrite",
    "💡 Identified": "1 - Latent State",
    "✅ Activated": "5 - Potentiator Ingestion",
    "🏆 Capitalized": "9 - Choice & Rewrite",
    "🧊 Archived": "9 - Choice & Rewrite",
}


# ─────────────────────────────────────────────────────────────────────────────
# Schema loading
# ─────────────────────────────────────────────────────────────────────────────

def load_default_archetype_mappings() -> dict[str, dict[str, dict[str, str]]]:
    """Load the default_archetype_mapping from each per_db/*.yaml schema.

    Returns: { db_key: { entry_type: { archetype_role, complex } } }
    """
    mappings: dict[str, dict[str, dict[str, str]]] = {}
    for db_key in ("matrix", "potentiator", "nexus", "significator", "greatway"):
        path = SCHEMAS_DIR / "per_db" / f"{db_key}.yaml"
        if not path.exists():
            continue
        with open(path) as f:
            schema = yaml.safe_load(f) or {}
        mapping = schema.get("default_archetype_mapping") or {}
        mappings[db_key] = mapping
    return mappings


# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────

def extract_value(prop_value: dict):
    ptype = prop_value.get("type")
    if ptype == "select":
        sel = prop_value.get("select")
        return sel.get("name") if sel else None
    if ptype == "status":
        st = prop_value.get("status")
        return st.get("name") if st else None
    if ptype == "multi_select":
        return [s.get("name") for s in prop_value.get("multi_select", [])]
    if ptype == "title":
        return "".join(t.get("plain_text", "") for t in prop_value.get("title", []))
    if ptype == "rich_text":
        return "".join(t.get("plain_text", "") for t in prop_value.get("rich_text", []))
    return None


# DB → entry-type property name
ENTRY_TYPE_PROP = {
    "matrix": "Entry Type",
    "potentiator": "Entry Type",
    "significator": "Entry Type",
    "greatway": "Item Type",
    "nexus": "Category",
}


def tag_entry(
    client: NotionClient,
    db_key: str,
    page: dict,
    mapping: dict[str, dict[str, str]],
    log: MigrationLog,
    dry_run: bool = False,
    skip_already_tagged: bool = True,
) -> dict:
    """Tag a single entry with default archetype_role + complex + digestion_stage."""
    page_id = page.get("id")
    props = page.get("properties", {})

    # Get the title for logging
    title = ""
    for pname, pval in props.items():
        if pval.get("type") == "title":
            title = "".join(t.get("plain_text", "") for t in pval.get("title", []))
            break

    # Get the entry_type
    et_prop_name = ENTRY_TYPE_PROP.get(db_key, "Entry Type")
    et_prop = props.get(et_prop_name)
    if not et_prop:
        op = {"type": "tag_entry", "db": db_key, "page_id": page_id, "title": title,
              "status": "skipped", "reason": f"No '{et_prop_name}' property found on entry."}
        log.warnings.append(op)
        return op

    entry_type = extract_value(et_prop)
    if not entry_type:
        op = {"type": "tag_entry", "db": db_key, "page_id": page_id, "title": title,
              "status": "skipped", "reason": f"'{et_prop_name}' is empty."}
        log.warnings.append(op)
        return op

    # Look up the default archetype_role + complex
    defaults = mapping.get(entry_type)
    if not defaults:
        op = {"type": "tag_entry", "db": db_key, "page_id": page_id, "title": title,
              "entry_type": entry_type,
              "status": "skipped", "reason": f"No default archetype mapping for entry_type '{entry_type}' in {db_key}."}
        log.warnings.append(op)
        return op

    archetype_role = defaults.get("archetype_role")
    complex_val = defaults.get("complex")

    # Check if already tagged
    existing_ar = extract_value(props.get("Archetype Role", {}))
    existing_cx = extract_value(props.get("Complex", {}))
    if skip_already_tagged and existing_ar and existing_cx:
        op = {"type": "tag_entry", "db": db_key, "page_id": page_id, "title": title,
              "entry_type": entry_type,
              "status": "already_tagged",
              "existing_archetype_role": existing_ar,
              "existing_complex": existing_cx}
        log.warnings.append(op)
        return op

    # Build the update properties
    update_props: dict = {}
    if archetype_role and archetype_role != existing_ar:
        update_props["Archetype Role"] = {"select": {"name": archetype_role}}
    if complex_val and complex_val != existing_cx:
        update_props["Complex"] = {"select": {"name": complex_val}}

    # Set Digestion Stage from Status (heuristic)
    status_val = extract_value(props.get("Status", {})) or extract_value(props.get("Digestion Status", {}))
    if status_val:
        digestion = STATUS_TO_DIGESTION.get(status_val)
        if digestion:
            existing_ds = extract_value(props.get("Digestion Stage", {}))
            if digestion != existing_ds:
                update_props["Digestion Stage"] = {"select": {"name": digestion}}

    if not update_props:
        op = {"type": "tag_entry", "db": db_key, "page_id": page_id, "title": title,
              "entry_type": entry_type,
              "status": "nothing_to_update"}
        log.warnings.append(op)
        return op

    op = {"type": "tag_entry", "db": db_key, "page_id": page_id, "title": title,
          "entry_type": entry_type,
          "archetype_role": archetype_role, "complex": complex_val,
          "update_props": list(update_props.keys())}

    if dry_run:
        op["status"] = "dry_run"
        op["message"] = f"Would tag '{title[:40]}' ({entry_type}): AR={archetype_role}, Cx={complex_val}"
        log.operations.append(op)
        return op

    try:
        client.update_page(page_id, update_props)
        op["status"] = "tagged"
        op["message"] = f"Tagged '{title[:40]}' ({entry_type}): AR={archetype_role}, Cx={complex_val}"
        log.operations.append(op)
        return op
    except Exception as e:
        op["status"] = "error"
        op["error"] = str(e)
        log.errors.append(op)
        return op


def tag_db(
    client: NotionClient,
    db_ids: dict[str, str],
    db_key: str,
    mapping: dict[str, dict[str, str]],
    log: MigrationLog,
    dry_run: bool = False,
    limit: int = 0,
) -> int:
    """Tag all entries in a single DB."""
    print(f"\n── Tagging entries in {db_key} ──")

    if db_key not in db_ids:
        print(f"  ❌ DB '{db_key}' not discovered.")
        return 0
    if not mapping:
        print(f"  ⏭️  No default_archetype_mapping found for {db_key} in schemas/per_db/{db_key}.yaml.")
        return 0

    ds_id = db_ids[db_key]
    pages = client.query_all_pages(ds_id)
    if limit:
        pages = pages[:limit]
    print(f"  Found {len(pages)} entries in {db_key}.")

    success = 0
    for i, page in enumerate(pages, 1):
        print(f"  [{i:02d}/{len(pages)}] ", end="", flush=True)
        result = tag_entry(client, db_key, page, mapping, log, dry_run=dry_run)
        status_icon = {"tagged": "✅", "dry_run": "🔍", "skipped": "⏭️",
                       "already_tagged": "⏭️", "nothing_to_update": "⏭️", "error": "❌"}.get(result["status"], "?")
        msg = result.get("message") or result.get("reason") or ""
        print(f"{status_icon} {msg[:80]}")
        if result["status"] == "tagged":
            success += 1
        if not dry_run:
            time.sleep(0.35)
    print(f"  → Tagged {success}/{len(pages)} entries.")
    return success


def main() -> int:
    parser = argparse.ArgumentParser(
        description="LifeOS v0.9.0 — Auto-Tag Existing Entries with Semantic Properties")
    parser.add_argument("--dry-run", action="store_true",
                        help="Show what would be done without making any API calls.")
    parser.add_argument("--db", action="append", default=None,
                        help="DB to tag (matrix, potentiator, nexus, significator, greatway). Can be repeated. Default: all 5.")
    parser.add_argument("--limit", type=int, default=0,
                        help="Max entries per DB (0 = unlimited). Useful for testing.")
    args = parser.parse_args()

    print_section("LifeOS v0.9.0 — Script 05: Auto-Tag Existing Entries")
    print_kv("Mode", "DRY RUN" if args.dry_run else "LIVE EXECUTION")
    print_kv("Started at", datetime.utcnow().isoformat() + "Z")

    # Load the default archetype mappings from the schema files
    mappings = load_default_archetype_mappings()
    print_kv("Schema mappings loaded", ", ".join(f"{k}={len(v)}" for k, v in mappings.items()))

    client = NotionClient()

    print_section("Step 1: Discover the 5 LifeOS DBs")
    db_ids = discover_db_ids(client)
    for k, v in db_ids.items():
        if not k.startswith("aux:"):
            print_kv(k, v)

    target_dbs = args.db if args.db else ["matrix", "potentiator", "nexus", "significator", "greatway"]

    log = MigrationLog(
        script_name="05_auto_tag_existing_entries",
        started_at=datetime.utcnow().isoformat() + "Z",
    )

    print_section("Step 2: Tag entries in each DB")
    total_tagged = 0
    for db_key in target_dbs:
        if db_key not in mappings:
            print(f"\n⏭️  Skipping {db_key} — no mapping found.")
            continue
        tagged = tag_db(client, db_ids, db_key, mappings[db_key], log, dry_run=args.dry_run, limit=args.limit)
        total_tagged += tagged

    log.finished_at = datetime.utcnow().isoformat() + "Z"
    log_path = Path(__file__).parent / "logs" / f"05_auto_tag_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.json"
    log.save(log_path)

    print_section("Summary")
    print_kv("Total entries tagged", total_tagged)
    print_kv("Operations (tagged)", sum(1 for o in log.operations if o.get("status") == "tagged"))
    print_kv("Operations (dry_run)", sum(1 for o in log.operations if o.get("status") == "dry_run"))
    print_kv("Warnings (skipped / already tagged)", len(log.warnings))
    print_kv("Errors", len(log.errors))

    if log.errors:
        print("\n❌ Some operations failed — see log for details.")
        return 1
    if not args.dry_run:
        print(f"\n✅ Auto-tagging complete. {total_tagged} entries tagged with default Archetype Role + Complex.")
        print("   NOTE: Holon Type and Valence Signature must be set manually per Significator entry,")
        print("   then run `lifeos derive-type --page-id <id>` to compute the Holon Type.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
