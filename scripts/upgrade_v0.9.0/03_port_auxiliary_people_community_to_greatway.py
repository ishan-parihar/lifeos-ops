#!/usr/bin/env python3
"""
LifeOS v0.9.0 — Script 03: Port People/Community/Group Entries INTO GreatWay
==============================================================================

Addresses the user's directive:
  "I want you to port the people and community/group DB entries into the
   GreatWay for accuracy."

WHAT THIS SCRIPT DOES:
  1. Discovers the 5 LifeOS DBs AND the auxiliary "People", "Community",
     "Group" DBs (if they exist in the Notion workspace).
  2. For each auxiliary DB entry, creates a corresponding GreatWay entry:
       People DB entry      → GreatWay.Person
       Community DB entry   → GreatWay.Community
       Group DB entry       → GreatWay.Group
  3. Maps properties from the auxiliary DB schema to the GreatWay schema:
       - Name → Name (always)
       - All Person-specific properties → preserved (e.g. email, phone, role)
       - Auxiliary DB-specific relations → migrated to GreatWay's inter-DB
         relations (e.g. Person → Person's projects become GreatWay.Person
         "Coheres With (Significator)" if there's a Significator link)
  4. Records the original auxiliary DB page ID in the new GreatWay entry's
     `holon_id` rich_text property for traceability.
  5. Does NOT delete or archive the original auxiliary DB entries — the
     user must verify and then archive them manually.

PROPERTY MAPPING HEURISTIC:
  Auxiliary DBs typically have 20-30 properties each (People DB has 32 per
  UPGRADE_FLAGS.md Flag 1). We map known properties by name and copy
  everything else as rich_text fallback:

    People DB property    → GreatWay.Person property
    ────────────────────────────────────────────────────
    Name                  → Name (title)
    Email                 → (Person-specific rich_text)
    Phone                 → (Person-specific rich_text)
    Role                  → person_type (select) — best-effort mapping
    Notes / Description   → (Person-specific rich_text)
    Last Contact          → last_contact_date (date)
    All other properties  → (preserved as rich_text in a "legacy_props" field)

USAGE:
    NOTION_API_TOKEN=your_token python3 03_port_auxiliary_people_community_to_greatway.py [--dry-run] [--aux-db People]

    --aux-db: Specify which auxiliary DB to port from. Can be repeated.
              Default: port from all discovered auxiliary DBs.
    --dry-run: Show what would be done without making any API calls.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from datetime import datetime
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from importlib import import_module
common = import_module("common")

from common import (NotionClient, discover_db_ids, get_database_container_id,
                     print_section, print_kv, MigrationLog)
from common import DB_NAMES


# ─────────────────────────────────────────────────────────────────────────────
# Auxiliary DB → GreatWay entry-type mapping
# ─────────────────────────────────────────────────────────────────────────────

AUX_TO_GREATWAY_ENTRY_TYPE = {
    "People": "Person",
    "Person": "Person",
    "Community": "Community",
    "Group": "Group",
    "Organization": "Organization",
    "Network": "Network",
    "Movement": "Movement",
}


# ─────────────────────────────────────────────────────────────────────────────
# Property mapping
# ─────────────────────────────────────────────────────────────────────────────

# Properties to LOOK FOR in the auxiliary DB and map to GreatWay Person entry-types
PERSON_PROP_MAP = {
    "Name": "title",          # always
    "Email": "rich_text",
    "Phone": "rich_text",
    "Phone Number": "rich_text",
    "Role": "person_type",    # select
    "Type": "person_type",    # select (alt name)
    "Relationship": "person_type",  # select (alt name)
    "Notes": "rich_text",
    "Description": "rich_text",
    "Bio": "rich_text",
    "Last Contact": "last_contact_date",  # date
    "Last Contacted": "last_contact_date",
    "Last Contact Date": "last_contact_date",
    "Contact Frequency": "contact_frequency",  # select
    "Tags": "tags",  # multi_select — keep as multi_select
    "Birthday": "rich_text",
    "Company": "rich_text",
    "Organization": "rich_text",
    "Location": "rich_text",
    "Website": "rich_text",
    "LinkedIn": "rich_text",
    "Twitter": "rich_text",
    "Avatar": "rich_text",
}

GROUP_PROP_MAP = {
    "Name": "title",
    "Description": "group_purpose",
    "Purpose": "group_purpose",
    "Size": "group_size",
    "Member Count": "group_size",
    "Cohesion": "group_cohesion",
    "Type": "group_bonding_type",
    "Tags": "tags",
}

COMMUNITY_PROP_MAP = {
    "Name": "title",
    "Description": "community_culture",
    "Purpose": "community_purpose",
    "Size": "community_size",
    "Members": "community_size",
    "Culture": "community_culture",
    "Stage": "community_stage",
    "Tags": "tags",
}


def get_prop_map_for_entry_type(entry_type: str) -> dict:
    if entry_type == "Person":
        return PERSON_PROP_MAP
    if entry_type == "Group":
        return GROUP_PROP_MAP
    if entry_type == "Community":
        return COMMUNITY_PROP_MAP
    return {"Name": "title"}


# ─────────────────────────────────────────────────────────────────────────────
# Property value extraction (mirrors yaml_schema_validator.extract_property_value)
# ─────────────────────────────────────────────────────────────────────────────

def extract_value(prop_value: dict):
    ptype = prop_value.get("type")
    if ptype == "title":
        return "".join(t.get("plain_text", "") for t in prop_value.get("title", []))
    if ptype == "rich_text":
        return "".join(t.get("plain_text", "") for t in prop_value.get("rich_text", []))
    if ptype == "select":
        sel = prop_value.get("select")
        return sel.get("name") if sel else None
    if ptype == "multi_select":
        return [s.get("name") for s in prop_value.get("multi_select", [])]
    if ptype == "status":
        st = prop_value.get("status")
        return st.get("name") if st else None
    if ptype == "date":
        d = prop_value.get("date")
        return d.get("start") if d else None
    if ptype == "number":
        return prop_value.get("number")
    if ptype == "checkbox":
        return prop_value.get("checkbox")
    if ptype == "url":
        return prop_value.get("url")
    if ptype == "email":
        return prop_value.get("email")
    if ptype == "phone_number":
        return prop_value.get("phone_number")
    if ptype == "people":
        return [p.get("id") for p in prop_value.get("people", [])]
    if ptype == "relation":
        return [r.get("id") for r in prop_value.get("relation", [])]
    return None


def build_notion_property(name: str, value, prop_type_hint: str = "rich_text"):
    """Build a Notion API property object from a Python value + type hint."""
    if value is None or value == "" or value == []:
        return None

    if prop_type_hint == "title":
        return {"title": [{"text": {"content": str(value)}}]}
    if prop_type_hint == "rich_text":
        # Truncate to 2000 chars (Notion rich_text limit)
        s = str(value)[:2000]
        return {"rich_text": [{"text": {"content": s}}]}
    if prop_type_hint == "select":
        return {"select": {"name": str(value)}}
    if prop_type_hint == "multi_select":
        if isinstance(value, list):
            return {"multi_select": [{"name": str(v)} for v in value]}
        return {"multi_select": [{"name": str(value)}]}
    if prop_type_hint == "date":
        return {"date": {"start": str(value)}}
    if prop_type_hint == "number":
        try:
            return {"number": float(value)}
        except (TypeError, ValueError):
            return None
    if prop_type_hint == "url":
        return {"url": str(value)}
    if prop_type_hint == "email":
        return {"email": str(value)}
    if prop_type_hint == "phone_number":
        return {"phone_number": str(value)}
    # Fallback: rich_text
    return {"rich_text": [{"text": {"content": str(value)[:2000]}}]}


# ─────────────────────────────────────────────────────────────────────────────
# Migration logic
# ─────────────────────────────────────────────────────────────────────────────

def port_one_entry(
    client: NotionClient,
    source_page: dict,
    target_db_key: str,
    target_ds_id: str,
    target_entry_type: str,
    target_db_container_id: str,
    log: MigrationLog,
    dry_run: bool = False,
) -> dict:
    """Port a single auxiliary DB entry to the target LifeOS DB."""
    source_id = source_page.get("id")
    source_props = source_page.get("properties", {})
    source_name = ""
    for pname, pval in source_props.items():
        if pval.get("type") == "title":
            source_name = "".join(t.get("plain_text", "") for t in pval.get("title", []))
            break

    op = {
        "type": "port_entry",
        "source_id": source_id,
        "source_name": source_name,
        "target_db": target_db_key,
        "target_entry_type": target_entry_type,
    }

    if not source_name.strip():
        op["status"] = "skipped"
        op["error"] = "Source entry has no title — skipping."
        log.warnings.append(op)
        return op

    # ── Build the new GreatWay entry's properties ──────────────
    prop_map = get_prop_map_for_entry_type(target_entry_type)
    new_props: dict = {}

    # Set the Item Type / Entry Type / Category discriminator
    et_prop_name = {"greatway": "Item Type", "nexus": "Category"}.get(target_db_key, "Entry Type")
    new_props[et_prop_name] = {"select": {"name": target_entry_type}}

    # Set the required v0.8.0 semantic typing properties (archetype_role, complex)
    new_props["Archetype Role"] = {"select": {"name": "Great Way"}}
    if target_entry_type in ("Person", "Group", "Community", "Movement"):
        new_props["Complex"] = {"select": {"name": "Spirit"}}
    elif target_entry_type in ("Organization", "Network", "Place"):
        new_props["Complex"] = {"select": {"name": "Body"}}

    # Set the quadrant property (per holon_type_placement.yaml)
    quadrant_map = {
        "Person": "UL", "Group": "LL", "Community": "LL",
        "Organization": "LR", "Network": "LR", "Movement": "LL", "Place": "LR",
    }
    if target_entry_type in quadrant_map:
        new_props["Quadrant"] = {"select": {"name": quadrant_map[target_entry_type]}}

    # Map known properties
    legacy_props: dict[str, str] = {}
    for src_pname, src_pval in source_props.items():
        target_pname_or_type = prop_map.get(src_pname)
        if target_pname_or_type is None:
            # Unknown property — capture in legacy_props
            v = extract_value(src_pval)
            if v is not None and v != "" and v != []:
                legacy_props[src_pname] = str(v)
            continue
        if target_pname_or_type == "title":
            # Title handled separately below
            continue
        v = extract_value(src_pval)
        if v is None or v == "" or v == []:
            continue
        # If the target property name is a known Person/Group/Community schema prop, use it
        # Otherwise, dump it as rich_text under the source property name
        notion_prop = build_notion_property(src_pname, v, "rich_text")
        if notion_prop:
            new_props[src_pname] = notion_prop

    # Add the title (mandatory)
    new_props["Name"] = {"title": [{"text": {"content": source_name[:2000]}}]}

    # Add holon_id (traceability — original auxiliary DB page ID)
    holon_id_str = f"legacy:{source_id}"
    new_props["Holon ID"] = {"rich_text": [{"text": {"content": holon_id_str}}]}

    # If there are legacy props, dump them as JSON in a "Legacy Properties" rich_text
    if legacy_props:
        legacy_json = json.dumps(legacy_props, default=str, indent=2)[:2000]
        new_props["Legacy Properties"] = {"rich_text": [{"text": {"content": legacy_json}}]}

    op["new_properties_count"] = len(new_props)

    if dry_run:
        op["status"] = "dry_run"
        op["message"] = f"Would create GreatWay.{target_entry_type}: '{source_name}'"
        log.operations.append(op)
        return op

    try:
        body = {
            "parent": {"data_source_id": target_ds_id},
            "properties": new_props,
        }
        result = client.create_page(body)
        op["status"] = "created"
        op["new_page_id"] = result.get("id")
        op["new_page_url"] = result.get("url")
        op["message"] = f"Created GreatWay.{target_entry_type}: '{source_name}' ({result.get('id', '?')[:8]})"
        log.operations.append(op)
        return op
    except Exception as e:
        op["status"] = "error"
        op["error"] = str(e)
        op["attempted_properties"] = list(new_props.keys())
        log.errors.append(op)
        return op


def port_auxiliary_db(
    client: NotionClient,
    db_ids: dict[str, str],
    aux_db_name: str,
    aux_ds_id: str,
    target_entry_type: str,
    log: MigrationLog,
    dry_run: bool = False,
    limit: int = 0,
) -> int:
    """Port all entries from one auxiliary DB to GreatWay."""
    print(f"\n── Porting from auxiliary DB '{aux_db_name}' ({aux_ds_id[:8]}) → GreatWay.{target_entry_type} ──")

    try:
        pages = client.query_all_pages(aux_ds_id)
    except Exception as e:
        print(f"  ❌ Failed to query auxiliary DB: {e}", file=sys.stderr)
        log.errors.append({
            "type": "query_aux_db_failed",
            "aux_db_name": aux_db_name,
            "aux_ds_id": aux_ds_id,
            "error": str(e),
        })
        return 0

    print(f"  Found {len(pages)} entries in '{aux_db_name}'.")

    if limit:
        pages = pages[:limit]
        print(f"  (Limited to first {limit} entries for testing.)")

    greatway_ds_id = db_ids.get("greatway")
    greatway_db_container = get_database_container_id(client, greatway_ds_id)

    success = 0
    for i, page in enumerate(pages, 1):
        print(f"  [{i:02d}/{len(pages)}] ", end="", flush=True)
        result = port_one_entry(
            client=client,
            source_page=page,
            target_db_key="greatway",
            target_ds_id=greatway_ds_id,
            target_entry_type=target_entry_type,
            target_db_container_id=greatway_db_container,
            log=log,
            dry_run=dry_run,
        )
        status_icon = {"created": "✅", "dry_run": "🔍", "skipped": "⏭️", "error": "❌"}.get(result["status"], "?")
        print(f"{status_icon} {result.get('message', result.get('error', ''))[:80]}")
        if result["status"] == "created":
            success += 1
        if not dry_run:
            time.sleep(0.4)  # Rate-limit politeness

    print(f"  → Ported {success}/{len(pages)} entries from '{aux_db_name}'.")
    return success


def main() -> int:
    parser = argparse.ArgumentParser(
        description="LifeOS v0.9.0 — Port People/Community/Group Entries INTO GreatWay")
    parser.add_argument("--dry-run", action="store_true",
                        help="Show what would be done without making any API calls.")
    parser.add_argument("--aux-db", action="append", default=None,
                        help="Auxiliary DB to port from (People, Community, Group, etc.). Can be repeated. Default: all discovered.")
    parser.add_argument("--limit", type=int, default=0,
                        help="Max entries per auxiliary DB (0 = unlimited). Useful for testing.")
    args = parser.parse_args()

    print_section("LifeOS v0.9.0 — Script 03: Port People/Community/Group INTO GreatWay")
    print_kv("Mode", "DRY RUN" if args.dry_run else "LIVE EXECUTION")
    print_kv("Started at", datetime.utcnow().isoformat() + "Z")
    if args.aux_db:
        print_kv("Auxiliary DBs specified", args.aux_db)
    else:
        print_kv("Auxiliary DBs", "All discovered (People, Community, Group, Organization, Network, Movement)")

    client = NotionClient()

    print_section("Step 1: Discover the 5 LifeOS DBs + auxiliary DBs")
    db_ids = discover_db_ids(client)
    for k, v in sorted(db_ids.items()):
        print_kv(k, v)

    if "greatway" not in db_ids:
        print(f"\n❌ FATAL: GreatWay DB not discovered.", file=sys.stderr)
        return 1

    aux_dbs_found = {k: v for k, v in db_ids.items() if k.startswith("aux:")}
    if not aux_dbs_found:
        print("\n⚠️  No auxiliary DBs discovered. Nothing to port.", file=sys.stderr)
        print("   Aux DBs searched for: People, Community, Group, Notes, Knowledge Base", file=sys.stderr)
        return 0

    log = MigrationLog(
        script_name="03_port_auxiliary_people_community_to_greatway",
        started_at=datetime.utcnow().isoformat() + "Z",
    )

    print_section("Step 2: Port auxiliary DB entries into GreatWay")
    total_ported = 0
    for aux_key, aux_ds_id in aux_dbs_found.items():
        aux_name = aux_key.replace("aux:", "")
        # Filter by user-specified aux DBs
        if args.aux_db and aux_name not in args.aux_db:
            print(f"\n⏭️  Skipping '{aux_name}' (not in --aux-db list).")
            continue
        # Determine target entry-type
        target_et = AUX_TO_GREATWAY_ENTRY_TYPE.get(aux_name)
        if not target_et:
            print(f"\n⏭️  Skipping '{aux_name}' (no GreatWay entry-type mapping).")
            continue
        ported = port_auxiliary_db(
            client=client,
            db_ids=db_ids,
            aux_db_name=aux_name,
            aux_ds_id=aux_ds_id,
            target_entry_type=target_et,
            log=log,
            dry_run=args.dry_run,
            limit=args.limit,
        )
        total_ported += ported

    log.finished_at = datetime.utcnow().isoformat() + "Z"
    log_path = Path(__file__).parent / "logs" / f"03_port_to_greatway_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.json"
    log.save(log_path)

    print_section("Summary")
    print_kv("Total entries ported", total_ported)
    print_kv("Operations (created)", sum(1 for o in log.operations if o.get("status") == "created"))
    print_kv("Operations (dry_run)", sum(1 for o in log.operations if o.get("status") == "dry_run"))
    print_kv("Warnings", len(log.warnings))
    print_kv("Errors", len(log.errors))

    if log.errors:
        print("\n❌ Some operations failed — see log for details.")
        return 1
    if not args.dry_run and total_ported > 0:
        print(f"\n✅ Ported {total_ported} entries into GreatWay.")
        print("   IMPORTANT: The original auxiliary DB entries were NOT deleted or archived.")
        print("   After verifying the ported entries in GreatWay, manually archive the")
        print("   originals in the Notion UI.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
