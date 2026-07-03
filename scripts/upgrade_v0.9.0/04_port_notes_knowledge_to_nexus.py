#!/usr/bin/env python3
"""
LifeOS v0.9.0 — Script 04: Port Notes/Knowledge-Categories FROM Potentiator INTO Nexus
=======================================================================================

Addresses the user's directive:
  "I also want the Notes, and Knowledge-Categories into the Nexus or any
   other DB, because the Potentiator is very flat in terms of data
   aggregation."

WHAT THIS SCRIPT DOES:
  1. Discovers the 5 LifeOS DBs.
  2. Queries the Potentiator DB for entries with Entry Type = "Note" or
     "Knowledge-Category" (or similar: "Knowledge", "Knowledge Base",
     "Notes", "Knowledge Atom", "Insight", "Reflection", "Knowledge-Atom").
  3. For each matching entry, creates a corresponding Nexus entry:
       Potentiator.Note                → Nexus.Note           (Kind: Catalyst)
       Potentiator.Knowledge-Category  → Nexus.Knowledge-Category (Kind: Experience)
       Potentiator.Knowledge           → Nexus.Knowledge-Category (Kind: Experience)
       Potentiator.Knowledge-Atom      → Nexus.Knowledge-Atom     (Kind: Experience)
  4. Maps properties from Potentiator schema to Nexus schema (Kind, Category,
     etc.) and preserves the original Potentiator entry's content.
  5. Records the original Potentiator page ID in the new Nexus entry's
     `holon_id` rich_text property for traceability.
  6. (Optional, --archive-source flag) Archives the original Potentiator
     entries after successful porting.

WHY MOVE NOTES & KNOWLEDGE TO NEXUS (per HoloOS ontology):
  - Per HoloOS doc 00.md §2.1: Catalyst = "any unprocessed pressure that
    crosses the holon's boundary." A raw Note IS a Catalyst.
  - Per HoloOS doc 00.md §2.1: Experience = "the processed result of
    Catalyst digestion, stored as changed configuration and future
    response-bias." A digested Knowledge-Category IS an Experience.
  - Per HoloOS doc 02.1 §5: "The Contact Boundary is the selectively
    permeable nexus where the Matrix and Potentiator interact." Catalyst
    and Experience flow THROUGH this boundary — they are NOT stored in
    the reservoirs (Matrix/Potentiator), they flow through the Nexus.
  - The Potentiator DB was "too flat in terms of data aggregation" (user
    directive) — Notes and Knowledge entries don't fit the Potentiator's
    role as latent-state generator. They are currency-flow entries.

PROPERTY MAPPING (Potentiator → Nexus):
  Potentiator property  → Nexus property
  ─────────────────────────────────────────────
  Name                  → Name (title)
  Entry Type            → Category (select — Note / Knowledge-Category / Knowledge-Atom)
  (computed)            → Kind (select — Catalyst for Note, Experience for Knowledge-*)
  Description           → raw_content (rich_text)
  Notes                 → raw_content (rich_text)
  Content               → raw_content (rich_text)
  Source URL            → source_url (url)
  URL                   → source_url (url)
  Status                → knowledge_synthesis_state (select, mapped)
  Tags                  → (preserved as multi_select)
  All other properties  → (preserved as rich_text in "Legacy Properties" field)

USAGE:
    NOTION_API_TOKEN=your_token python3 04_port_notes_knowledge_to_nexus.py [--dry-run] [--archive-source]

    --dry-run: Show what would be done without making any API calls.
    --archive-source: Archive the original Potentiator entries after porting (default: do NOT archive).
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
common = import_module("00_common")

from common import (NotionClient, discover_db_ids, get_database_container_id,
                     print_section, print_kv, MigrationLog)


# ─────────────────────────────────────────────────────────────────────────────
# Potentiator entry-type → Nexus entry-type + Kind mapping
# ─────────────────────────────────────────────────────────────────────────────

POTENTIATOR_TO_NEXUS_MAPPING = {
    # entry-type in Potentiator → (Nexus.Category, Nexus.Kind)
    "Note":                 ("Note",                 "Catalyst"),
    "Notes":                ("Note",                 "Catalyst"),
    "Knowledge":            ("Knowledge-Category",   "Experience"),
    "Knowledge-Base":       ("Knowledge-Category",   "Experience"),
    "Knowledge Base":       ("Knowledge-Category",   "Experience"),
    "Knowledge-Category":   ("Knowledge-Category",   "Experience"),
    "Knowledge Category":   ("Knowledge-Category",   "Experience"),
    "Knowledge-Atom":       ("Knowledge-Atom",       "Experience"),
    "Knowledge Atom":       ("Knowledge-Atom",       "Experience"),
    "Insight":              ("Note",                 "Catalyst"),  # treat as Catalyst-class
    "Reflection":           ("Knowledge-Category",   "Experience"),  # treat as Experience-class
}

# Status mapping (Potentiator → Nexus synthesis state)
STATUS_TO_SYNTHESIS = {
    "Raw": "raw_note",
    "Digesting": "annotated",
    "Crystallized": "synthesized",
    "Active": "applied",
    "Done": "applied",
    "Draft": "raw_note",
    "In Progress": "annotated",
    "Synthesized": "synthesized",
    "Applied": "applied",
}

# Property mapping
POTENTIATOR_PROP_MAP = {
    "Name": "title",
    "Description": "raw_content",
    "Notes": "raw_content",
    "Content": "raw_content",
    "Body": "raw_content",
    "Source URL": "source_url",
    "URL": "source_url",
    "Source": "source_url",
    "Link": "source_url",
    "Author": "rich_text",
    "Tags": "tags",
    "Topic": "rich_text",
    "Category": "rich_text",  # the entry_type discriminator — handled separately
    "Date": "rich_text",
    "Created": "rich_text",
    "Highlights": "highlight_count",
    "Highlight Count": "highlight_count",
}


# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────

def extract_value(prop_value: dict):
    """Extract a Python value from a Notion property value object."""
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
    if ptype == "url":
        return prop_value.get("url")
    if ptype == "people":
        return [p.get("id") for p in prop_value.get("people", [])]
    if ptype == "relation":
        return [r.get("id") for r in prop_value.get("relation", [])]
    return None


def build_notion_property(name: str, value, prop_type_hint: str = "rich_text"):
    if value is None or value == "" or value == []:
        return None
    if prop_type_hint == "title":
        return {"title": [{"text": {"content": str(value)[:2000]}}]}
    if prop_type_hint == "rich_text":
        return {"rich_text": [{"text": {"content": str(value)[:2000]}}]}
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
    return {"rich_text": [{"text": {"content": str(value)[:2000]}}]}


def find_matching_potentiator_entries(client: NotionClient, potentiometer_ds_id: str) -> list[dict]:
    """Query the Potentiator DB and return only entries with Note/Knowledge entry-types."""
    all_pages = client.query_all_pages(potentiometer_ds_id)
    matching = []
    for page in all_pages:
        props = page.get("properties", {})
        # Find the "Entry Type" property (could be select or multi_select)
        et_prop = props.get("Entry Type")
        if not et_prop:
            continue
        et_value = extract_value(et_prop)
        if isinstance(et_value, list):
            et_values = et_value
        else:
            et_values = [et_value] if et_value else []
        for v in et_values:
            if v in POTENTIATOR_TO_NEXUS_MAPPING:
                matching.append((page, v))
                break
    return matching


def port_one_note_to_nexus(
    client: NotionClient,
    source_page: dict,
    source_entry_type: str,
    target_ds_id: str,
    log: MigrationLog,
    dry_run: bool = False,
    archive_source: bool = False,
) -> dict:
    """Port a single Potentiator.Note/Knowledge-* entry to Nexus."""
    source_id = source_page.get("id")
    source_props = source_page.get("properties", {})

    # Find the title
    source_name = ""
    for pname, pval in source_props.items():
        if pval.get("type") == "title":
            source_name = "".join(t.get("plain_text", "") for t in pval.get("title", []))
            break

    # Determine Nexus target entry-type + Kind
    nexus_category, nexus_kind = POTENTIATOR_TO_NEXUS_MAPPING[source_entry_type]

    op = {
        "type": "port_note_to_nexus",
        "source_id": source_id,
        "source_name": source_name,
        "source_entry_type": source_entry_type,
        "target_db": "nexus",
        "target_entry_type": nexus_category,
        "target_kind": nexus_kind,
    }

    if not source_name.strip():
        op["status"] = "skipped"
        op["error"] = "Source entry has no title — skipping."
        log.warnings.append(op)
        return op

    # ── Build the new Nexus entry's properties ──────────────
    new_props: dict = {}

    # Required: Category (entry-type discriminator)
    new_props["Category"] = {"select": {"name": nexus_category}}

    # Required: Kind (currency discriminator)
    new_props["Kind"] = {"select": {"name": nexus_kind}}

    # Required: v0.8.0 semantic typing (archetype_role, complex)
    new_props["Archetype Role"] = {"select": {"name": "Catalyst" if nexus_kind == "Catalyst" else "Experience"}}
    new_props["Complex"] = {"select": {"name": "Mind"}}  # notes/knowledge are Mind-complex

    # Set the digestion_stage
    if nexus_kind == "Catalyst":
        new_props["Digestion Stage"] = {"select": {"name": "1 - Latent State"}}
    else:
        new_props["Digestion Stage"] = {"select": {"name": "4 - Matrix Digestion"}}

    # Map known properties
    legacy_props: dict[str, str] = {}
    for src_pname, src_pval in source_props.items():
        if src_pname == "Entry Type":
            continue  # handled separately
        target_pname_or_type = POTENTIATOR_PROP_MAP.get(src_pname)
        if target_pname_or_type is None:
            # Unknown property — capture in legacy_props
            v = extract_value(src_pval)
            if v is not None and v != "" and v != []:
                legacy_props[src_pname] = str(v)
            continue
        if target_pname_or_type == "title":
            continue  # title handled separately
        v = extract_value(src_pval)
        if v is None or v == "" or v == []:
            continue
        # Special mappings
        if target_pname_or_type == "highlight_count":
            notion_prop = build_notion_property("Highlight Count", v, "number")
            if notion_prop:
                new_props["Highlight Count"] = notion_prop
        elif target_pname_or_type == "source_url":
            notion_prop = build_notion_property("Source URL", v, "url")
            if notion_prop:
                new_props["Source URL"] = notion_prop
        elif target_pname_or_type == "raw_content":
            # Combine all content-bearing fields
            existing = new_props.get("Raw Content", {}).get("rich_text", [{}])[0].get("text", {}).get("content", "")
            combined = (existing + "\n\n" + str(v) if existing else str(v))[:2000]
            new_props["Raw Content"] = {"rich_text": [{"text": {"content": combined}}]}
        elif target_pname_or_type == "tags":
            notion_prop = build_notion_property("Tags", v, "multi_select")
            if notion_prop:
                new_props["Tags"] = notion_prop
        else:
            # Generic rich_text dump
            notion_prop = build_notion_property(src_pname, v, "rich_text")
            if notion_prop:
                new_props[src_pname] = notion_prop

    # Set the synthesis state based on the source Status (if any)
    src_status_prop = source_props.get("Status") or source_props.get("Digestion Status")
    if src_status_prop:
        src_status = extract_value(src_status_prop)
        synthesis = STATUS_TO_SYNTHESIS.get(src_status)
        if synthesis:
            new_props["Synthesis State"] = {"select": {"name": synthesis}}

    # Add the title (mandatory)
    new_props["Name"] = {"title": [{"text": {"content": source_name[:2000]}}]}

    # Add holon_id (traceability)
    new_props["Holon ID"] = {"rich_text": [{"text": {"content": f"legacy:potentiator:{source_id}"}}]}

    # Dump legacy props as JSON in a "Legacy Properties" rich_text
    if legacy_props:
        legacy_json = json.dumps(legacy_props, default=str, indent=2)[:2000]
        new_props["Legacy Properties"] = {"rich_text": [{"text": {"content": legacy_json}}]}

    op["new_properties_count"] = len(new_props)

    if dry_run:
        op["status"] = "dry_run"
        op["message"] = f"Would create Nexus.{nexus_category} (Kind={nexus_kind}): '{source_name}'"
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
        op["message"] = f"Created Nexus.{nexus_category} (Kind={nexus_kind}): '{source_name}' ({result.get('id', '?')[:8]})"

        # Optionally archive the source
        if archive_source:
            try:
                client.archive_page(source_id)
                op["source_archived"] = True
                op["message"] += " [SOURCE ARCHIVED]"
            except Exception as e:
                op["source_archive_error"] = str(e)

        log.operations.append(op)
        return op
    except Exception as e:
        op["status"] = "error"
        op["error"] = str(e)
        op["attempted_properties"] = list(new_props.keys())
        log.errors.append(op)
        return op


def main() -> int:
    parser = argparse.ArgumentParser(
        description="LifeOS v0.9.0 — Port Notes/Knowledge-Categories FROM Potentiator INTO Nexus")
    parser.add_argument("--dry-run", action="store_true",
                        help="Show what would be done without making any API calls.")
    parser.add_argument("--archive-source", action="store_true",
                        help="Archive the original Potentiator entries after successful porting (default: do NOT archive).")
    parser.add_argument("--limit", type=int, default=0,
                        help="Max entries to port (0 = unlimited). Useful for testing.")
    args = parser.parse_args()

    print_section("LifeOS v0.9.0 — Script 04: Port Notes/Knowledge FROM Potentiator INTO Nexus")
    print_kv("Mode", "DRY RUN" if args.dry_run else "LIVE EXECUTION")
    print_kv("Started at", datetime.utcnow().isoformat() + "Z")
    print_kv("Archive source entries", args.archive_source)

    client = NotionClient()

    print_section("Step 1: Discover the 5 LifeOS DBs")
    db_ids = discover_db_ids(client)
    for k, v in db_ids.items():
        if not k.startswith("aux:"):
            print_kv(k, v)

    if "potentiator" not in db_ids or "nexus" not in db_ids:
        print(f"\n❌ FATAL: Potentiator or Nexus DB not discovered.", file=sys.stderr)
        return 1

    print_section("Step 2: Find Potentiator entries with Note/Knowledge entry-types")
    matching = find_matching_potentiator_entries(client, db_ids["potentiator"])
    print_kv("Total Potentiator entries with Note/Knowledge entry-types", len(matching))

    if not matching:
        print("\n⚠️  No Potentiator entries match the Note/Knowledge entry-types.")
        print("   Searched for: " + ", ".join(POTENTIATOR_TO_NEXUS_MAPPING.keys()))
        return 0

    # Show breakdown by entry-type
    by_et: dict[str, int] = {}
    for _, et in matching:
        by_et[et] = by_et.get(et, 0) + 1
    print("\n   Breakdown:")
    for et, count in sorted(by_et.items()):
        print_kv(et, count, indent=7)

    if args.limit:
        matching = matching[:args.limit]
        print(f"\n   (Limited to first {args.limit} entries for testing.)")

    log = MigrationLog(
        script_name="04_port_notes_knowledge_to_nexus",
        started_at=datetime.utcnow().isoformat() + "Z",
    )

    print_section(f"Step 3: Port {len(matching)} entries into Nexus")
    success = 0
    for i, (page, source_et) in enumerate(matching, 1):
        print(f"  [{i:02d}/{len(matching)}] ", end="", flush=True)
        result = port_one_note_to_nexus(
            client=client,
            source_page=page,
            source_entry_type=source_et,
            target_ds_id=db_ids["nexus"],
            log=log,
            dry_run=args.dry_run,
            archive_source=args.archive_source,
        )
        status_icon = {"created": "✅", "dry_run": "🔍", "skipped": "⏭️", "error": "❌"}.get(result["status"], "?")
        print(f"{status_icon} {result.get('message', result.get('error', ''))[:80]}")
        if result["status"] == "created":
            success += 1
        if not args.dry_run:
            time.sleep(0.4)

    log.finished_at = datetime.utcnow().isoformat() + "Z"
    log_path = Path(__file__).parent / "logs" / f"04_port_to_nexus_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.json"
    log.save(log_path)

    print_section("Summary")
    print_kv("Total entries ported", success)
    print_kv("Operations (created)", sum(1 for o in log.operations if o.get("status") == "created"))
    print_kv("Operations (dry_run)", sum(1 for o in log.operations if o.get("status") == "dry_run"))
    print_kv("Warnings", len(log.warnings))
    print_kv("Errors", len(log.errors))

    if log.errors:
        print("\n❌ Some operations failed — see log for details.")
        return 1
    if not args.dry_run and success > 0:
        print(f"\n✅ Ported {success} entries into Nexus.")
        if not args.archive_source:
            print("   The original Potentiator entries were NOT archived.")
            print("   After verifying the ported entries in Nexus, run this script again")
            print("   with --archive-source to archive the originals, OR archive them")
            print("   manually in the Notion UI.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
