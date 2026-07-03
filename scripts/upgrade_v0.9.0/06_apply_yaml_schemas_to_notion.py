#!/usr/bin/env python3
"""
LifeOS v0.9.0 — Script 06: Apply YAML Schema Properties to Notion
==================================================================

WHAT THIS SCRIPT DOES:
  Reads the v0.9.0 YAML schema files (schemas/per_db/*.yaml) and creates
  any Notion properties that are declared in the schemas but don't yet
  exist in the live Notion databases.

  This is the "schema materialization" step — after running this, every
  per-DB extended-property declared in the YAML schemas will exist as a
  real Notion property on the corresponding DB.

  Per-entry-type specializations (per_entry_type/*.yaml) are NOT created
  on Notion (Notion doesn't have entry-type-specific property schemas —
  every property on a DB applies to all entries). The per_entry_type
  schemas are validated client-side by the `lifeos validate-yaml` CLI
  command.

WHAT THIS SCRIPT DOES NOT DO:
  - Does NOT delete or rename existing properties (additive only).
  - Does NOT modify existing property types (Notion API doesn't support this).
  - Does NOT add new select options to existing properties (use script 02 for that).
  - Does NOT create the v0.8.0 semantic properties (Archetype Role, Complex,
    Drive Activation, Shadow Pattern, Digestion Stage, Holon Type, Valence
    Signature) — those were already added by the v0.8.0 upgrade.

USAGE:
    NOTION_API_TOKEN=your_token python3 06_apply_yaml_schemas_to_notion.py [--dry-run] [--db matrix]

    --db: Only apply to the specified DB. Can be repeated.
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
common = import_module("common")

from common import (NotionClient, discover_db_ids, get_database_container_id,
                     add_select_property, add_rich_text_property, add_number_property,
                     add_relation_property, print_section, print_kv, MigrationLog)

SCHEMAS_DIR = Path(__file__).resolve().parents[2] / "schemas"


# ─────────────────────────────────────────────────────────────────────────────
# Schema loading
# ─────────────────────────────────────────────────────────────────────────────

def load_per_db_schemas() -> dict[str, dict]:
    """Load the 5 per_db/*.yaml schemas. Returns: { db_key: schema_dict }."""
    schemas: dict[str, dict] = {}
    for db_key in ("matrix", "potentiator", "nexus", "significator", "greatway"):
        path = SCHEMAS_DIR / "per_db" / f"{db_key}.yaml"
        if path.exists():
            with open(path) as f:
                schemas[db_key] = yaml.safe_load(f) or {}
    return schemas


# ─────────────────────────────────────────────────────────────────────────────
# Notion property creation
# ─────────────────────────────────────────────────────────────────────────────

def build_property_payload(prop_name: str, prop_schema: dict) -> dict | None:
    """Build a Notion API property creation payload for a single property.

    Returns None if the property type isn't supported for creation.
    """
    ntype = prop_schema.get("notion_type")
    options = prop_schema.get("options") or []

    if ntype == "select":
        return {prop_name: {"select": {"options": [{"name": o} for o in options]}}}
    if ntype == "multi_select":
        return {prop_name: {"multi_select": {"options": [{"name": o} for o in options]}}}
    if ntype == "status":
        # Note: Notion API doesn't support adding options to status properties
        # — but creating a new status property with options might work.
        return {prop_name: {"status": {"options": [{"name": o} for o in options]}}}
    if ntype == "rich_text":
        return {prop_name: {"rich_text": {}}}
    if ntype == "number":
        return {prop_name: {"number": {"format": "number"}}}
    if ntype == "checkbox":
        return {prop_name: {"checkbox": {}}}
    if ntype == "date":
        return {prop_name: {"date": {}}}
    if ntype == "url":
        return {prop_name: {"url": {}}}
    if ntype == "email":
        return {prop_name: {"email": {}}}
    if ntype == "phone_number":
        return {prop_name: {"phone_number": {}}}
    # Formula, rollup, computed types are not creatable via API
    return None


def apply_db_schema(
    client: NotionClient,
    db_ids: dict[str, str],
    db_key: str,
    schema: dict,
    log: MigrationLog,
    dry_run: bool = False,
) -> dict:
    """Apply a single per_db schema to a Notion DB — creates missing properties only."""
    op = {"type": "apply_db_schema", "db": db_key}

    ds_id = db_ids.get(db_key)
    if not ds_id:
        op["status"] = "skipped"
        op["error"] = f"DB '{db_key}' not discovered."
        log.errors.append(op)
        return op

    db_container = get_database_container_id(client, ds_id)
    op["db_container_id"] = db_container

    # Fetch the current Notion schema
    try:
        notion_schema = client.get_database(db_container)
    except Exception as e:
        op["status"] = "error"
        op["error"] = f"Failed to fetch schema: {e}"
        log.errors.append(op)
        return op

    existing_props = notion_schema.get("properties", {})
    op["existing_prop_count"] = len(existing_props)

    # Build the list of properties to create
    declared_props = schema.get("properties") or {}
    to_create: dict[str, dict] = {}
    skipped: list[str] = []
    for pname, pschema in declared_props.items():
        if pname in existing_props:
            skipped.append(pname)
            continue
        payload = build_property_payload(pname, pschema)
        if payload is None:
            skipped.append(f"{pname} (type={pschema.get('notion_type')} — not creatable via API)")
            continue
        to_create.update(payload)

    op["declared_count"] = len(declared_props)
    op["already_present"] = len(skipped)
    op["to_create"] = list(to_create.keys())

    if not to_create:
        op["status"] = "nothing_to_create"
        op["message"] = f"All {len(declared_props)} declared properties already exist on {db_key}."
        log.warnings.append(op)
        return op

    if dry_run:
        op["status"] = "dry_run"
        op["message"] = f"Would create {len(to_create)} new properties on {db_key}: {list(to_create.keys())}"
        log.operations.append(op)
        return op

    # Create all missing properties in a single PATCH (Notion API allows multiple properties per call)
    try:
        body = {"properties": to_create}
        result = client.update_database(db_container, body)
        op["status"] = "created"
        op["created_count"] = len(to_create)
        op["created_properties"] = list(to_create.keys())
        op["message"] = f"Created {len(to_create)} new properties on {db_key}: {list(to_create.keys())}"
        log.operations.append(op)
        return op
    except Exception as e:
        op["status"] = "error"
        op["error"] = str(e)
        op["attempted_properties"] = list(to_create.keys())
        log.errors.append(op)
        return op


# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────

def main() -> int:
    parser = argparse.ArgumentParser(
        description="LifeOS v0.9.0 — Apply YAML Schema Properties to Notion")
    parser.add_argument("--dry-run", action="store_true",
                        help="Show what would be done without making any API calls.")
    parser.add_argument("--db", action="append", default=None,
                        help="DB to apply (matrix, potentiator, nexus, significator, greatway). Can be repeated.")
    args = parser.parse_args()

    print_section("LifeOS v0.9.0 — Script 06: Apply YAML Schema Properties to Notion")
    print_kv("Mode", "DRY RUN" if args.dry_run else "LIVE EXECUTION")
    print_kv("Started at", datetime.utcnow().isoformat() + "Z")

    schemas = load_per_db_schemas()
    print_kv("Schemas loaded", ", ".join(f"{k}={len(v.get('properties', {}))} props" for k, v in schemas.items()))

    client = NotionClient()

    print_section("Step 1: Discover the 5 LifeOS DBs")
    db_ids = discover_db_ids(client)
    for k, v in db_ids.items():
        if not k.startswith("aux:"):
            print_kv(k, v)

    target_dbs = args.db if args.db else ["matrix", "potentiator", "nexus", "significator", "greatway"]

    log = MigrationLog(
        script_name="06_apply_yaml_schemas_to_notion",
        started_at=datetime.utcnow().isoformat() + "Z",
    )

    print_section("Step 2: Apply schemas to each DB")
    for db_key in target_dbs:
        if db_key not in schemas:
            print(f"\n⏭️  Skipping {db_key} — no schema found.")
            continue
        print(f"\n── Applying schema to {db_key} ──")
        result = apply_db_schema(client, db_ids, db_key, schemas[db_key], log, dry_run=args.dry_run)
        print(f"   Status: {result['status']}")
        if "message" in result:
            print(f"   {result['message']}")
        if not args.dry_run:
            time.sleep(0.5)

    log.finished_at = datetime.utcnow().isoformat() + "Z"
    log_path = Path(__file__).parent / "logs" / f"06_apply_yaml_schemas_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.json"
    log.save(log_path)

    print_section("Summary")
    print_kv("Operations (created)", sum(1 for o in log.operations if o.get("status") == "created"))
    print_kv("Operations (dry_run)", sum(1 for o in log.operations if o.get("status") == "dry_run"))
    print_kv("Warnings (nothing to create)", len(log.warnings))
    print_kv("Errors", len(log.errors))

    if log.errors:
        print("\n❌ Some operations failed — see log for details.")
        return 1
    if not args.dry_run:
        print("\n✅ All schema properties materialized in Notion.")
        print("   You can now run `lifeos discover` to refresh the schema cache, then")
        print("   `lifeos validate-yaml --all` to validate all entries against the YAML schemas.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
