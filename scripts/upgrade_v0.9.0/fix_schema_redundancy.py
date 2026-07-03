#!/usr/bin/env python3
"""
LifeOS v0.9.0 — Schema Redundancy Fixer
=========================================

Auto-fixes the redundancy issues detected by audit_schema_redundancy.py.

FIX STRATEGY:
  1. Remove entry-type-specific properties FROM per_db/*.yaml
  2. Ensure those properties ARE declared in the relevant per_entry_type/*.yaml
  3. Remove properties from per_entry_type/*.yaml that duplicate per_db or universal
  4. Unify name mismatches (knowledge_synthesis_state → synthesis_state)
  5. Re-run self-test to verify

USAGE:
    python3 fix_schema_redundancy.py
    python3 fix_schema_redundancy.py --dry-run
"""

from __future__ import annotations

import argparse
import copy
import re
import sys
from dataclasses import dataclass
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]
SCHEMAS_DIR = REPO_ROOT / "schemas"

DB_KEYS = ("matrix", "potentiator", "nexus", "significator", "greatway")


# ─────────────────────────────────────────────────────────────────────────────
# Define the moves: (db, property_name) → spec
# ─────────────────────────────────────────────────────────────────────────────

MOVES = {
    # ── nexus: entry-type-specific properties to move out ──
    ("nexus", "crucible_intensity"): {
        "targets": [("nexus", "Pattern"), ("nexus", "Crisis"), ("nexus", "Transformation-Event")],
        "reason": "Transformation-kind only",
    },
    ("nexus", "choice_polarity"): {
        "targets": [("nexus", "Directive"), ("nexus", "Decision")],
        "reason": "Choice-kind only",
    },
    ("nexus", "crystallization_ratio"): {
        "targets": [("nexus", "Directive"), ("nexus", "Decision")],
        "reason": "Choice-kind only",
    },
    ("nexus", "source_url"): {
        "targets": [("nexus", "Note")],
        "reason": "Note-kind only (relocated from Potentiator)",
    },
    ("nexus", "knowledge_synthesis_state"): {
        "targets": [("nexus", "Note"), ("nexus", "Knowledge-Category"), ("nexus", "Knowledge-Atom")],
        "reason": "Note/Knowledge entries only",
        "rename_to": "synthesis_state",
    },
    ("nexus", "transformation_threshold"): {
        "targets": [("nexus", "Pattern"), ("nexus", "Crisis"), ("nexus", "Transformation-Event")],
        "reason": "Transformation-kind only (threshold for the contact-boundary event to fire)",
    },
    # ── significator ──
    ("significator", "principle_sub_type"): {
        "targets": [("significator", "Purpose"), ("significator", "Value"), ("significator", "Principle")],
        "reason": "Principle-family entries only (per Flag 3 consolidation)",
    },
    # ── greatway: external-holon-specific properties to move out ──
    ("greatway", "quadrant"): {
        "targets": [
            ("greatway", "Person"), ("greatway", "Group"), ("greatway", "Community"),
            ("greatway", "Organization"), ("greatway", "Network"), ("greatway", "Movement"),
            ("greatway", "Place"),
        ],
        "reason": "External-holon entry-types only (per holon_type_placement.yaml)",
    },
    ("greatway", "dimension"): {
        "targets": [
            ("greatway", "Person"), ("greatway", "Group"), ("greatway", "Community"),
            ("greatway", "Organization"), ("greatway", "Network"), ("greatway", "Movement"),
            ("greatway", "Place"),
        ],
        "reason": "External-holon entry-types only",
    },
    ("greatway", "bonding_disposition"): {
        "targets": [
            ("greatway", "Person"), ("greatway", "Group"), ("greatway", "Community"),
            ("greatway", "Organization"), ("greatway", "Network"), ("greatway", "Movement"),
        ],
        "reason": "External-holon entry-types only (HoloOS doc 02.3 §4 bonding)",
    },
    ("greatway", "bonding_partner_count"): {
        "targets": [
            ("greatway", "Person"), ("greatway", "Group"), ("greatway", "Community"),
            ("greatway", "Organization"), ("greatway", "Network"), ("greatway", "Movement"),
        ],
        "reason": "External-holon entry-types only (metallic-bonding lattice density)",
    },
}


@dataclass
class FixAction:
    action: str
    file: str
    property_name: str
    details: str


def load_yaml(path: Path) -> dict:
    with open(path) as f:
        return yaml.safe_load(f) or {}


def clean_description(desc: str) -> str:
    """Remove 'For X entries' / 'Used only for Y-kind' prefixes from a description."""
    patterns = [
        r"^For (?:Person|Group|Community|Organization|Network|Movement|Place|Note|Knowledge-Category|Knowledge-Atom|Decision|Crisis|Transformation-Event|Principle|Purpose|Value)(?:-kind)? entries[^:]*:\s*\n",
        r"^For (?:external-holon|relocated)[^:]*:\s*\n",
        r"^Used only for (?:Transformation|Choice|Catalyst|Experience)-kind Nexus entries[^:]*:\s*\n",
        r"^Required for (?:external-holon|Person|Group|Community)[^:]*:\s*\n",
        r"^For (?:Transformation|Choice|Catalyst|Experience)-kind Nexus entries[^:]*:\s*\n",
        r"^For (?:Transformation|Choice|Catalyst|Experience)-kind entries[^:]*:\s*\n",
        r"^For Principle entries only[^:]*:\s*\n",
    ]
    cleaned = desc
    for p in patterns:
        cleaned = re.sub(p, "", cleaned, flags=re.MULTILINE)
    return cleaned.strip()


def save_with_header(path: Path, data: dict) -> None:
    """Save YAML, preserving ONLY the comment header (lines starting with #).

    The header is everything from the start of the file up to (but not including)
    the first non-comment, non-blank line. This avoids duplicating YAML keys
    that are already in `data`.
    """
    original = path.read_text()
    header_lines: list[str] = []
    for line in original.splitlines():
        stripped = line.strip()
        if not stripped:
            header_lines.append("")
            continue
        if stripped.startswith("#"):
            header_lines.append(line)
            continue
        # First non-comment, non-blank line — stop
        break
    # Strip trailing blank lines from header
    while header_lines and not header_lines[-1].strip():
        header_lines.pop()
    header = "\n".join(header_lines)

    with open(path, "w") as f:
        if header:
            f.write(header + "\n\n")
        yaml.safe_dump(data, f, sort_keys=False, default_flow_style=False, allow_unicode=True, width=120)


def execute_fix(dry_run: bool = False) -> list[FixAction]:
    actions: list[FixAction] = []

    # Load all schemas
    per_db_schemas: dict[str, dict] = {}
    for db in DB_KEYS:
        path = SCHEMAS_DIR / "per_db" / f"{db}.yaml"
        per_db_schemas[db] = load_yaml(path)

    pet_schemas: dict[tuple[str, str], tuple[Path, dict]] = {}
    pet_dir = SCHEMAS_DIR / "per_entry_type"
    for path in sorted(pet_dir.glob("*.yaml")):
        schema = load_yaml(path)
        db = schema.get("applies_to_db")
        et = schema.get("applies_to_entry_type")
        if db and et:
            pet_schemas[(db, et)] = (path, schema)

    # Execute each move
    for (source_db, prop_name), spec in MOVES.items():
        rename_to = spec.get("rename_to")
        final_name = rename_to or prop_name

        # Step 1: Remove from per_db (capture the removed property schema for reuse)
        db_schema = per_db_schemas.get(source_db, {})
        db_props = db_schema.get("properties", {})
        removed_prop: dict | None = None
        if prop_name in db_props:
            removed_prop = db_props.pop(prop_name)
            actions.append(FixAction(
                action="remove_from_per_db",
                file=f"per_db/{source_db}.yaml",
                property_name=prop_name,
                details=f"Removed '{prop_name}' from per_db/{source_db}.yaml ({spec['reason']})",
            ))
        else:
            print(f"  ⚠️  Property '{prop_name}' not found in per_db/{source_db}.yaml — already removed?")

        # Step 2: Add to each target per_entry_type file
        for (target_db, target_et) in spec["targets"]:
            key = (target_db, target_et)
            if key not in pet_schemas:
                print(f"  ⚠️  Target per_entry_type file not found: {target_db}__{target_et}.yaml — skipping")
                continue
            path, pet_schema = pet_schemas[key]
            pet_props = pet_schema.setdefault("properties", {})

            # If renaming, check if the old name is already there and remove it
            if rename_to and prop_name in pet_props:
                pet_props.pop(prop_name)
                actions.append(FixAction(
                    action="rename_in_per_entry_type",
                    file=path.name,
                    property_name=prop_name,
                    details=f"Renamed '{prop_name}' → '{rename_to}' in {path.name}",
                ))

            # Add the property under its final name (if not already present)
            if final_name not in pet_props:
                if removed_prop is not None:
                    prop_copy = copy.deepcopy(removed_prop)
                else:
                    prop_copy = {
                        "notion_type": "select",
                        "required": False,
                        "description": f"{spec['reason']}.",
                    }
                # Clean the description
                prop_copy["description"] = clean_description(prop_copy.get("description", ""))
                pet_props[final_name] = prop_copy
                actions.append(FixAction(
                    action="add_to_per_entry_type",
                    file=path.name,
                    property_name=final_name,
                    details=f"Added '{final_name}' to {path.name} ({spec['reason']})",
                ))
            else:
                # Already present — update from the moved property if available
                existing = pet_props[final_name]
                if removed_prop is not None:
                    existing["description"] = clean_description(removed_prop.get("description", ""))
                    if "options" in removed_prop:
                        existing["options"] = removed_prop["options"]
                    if "notion_type" in removed_prop:
                        existing["notion_type"] = removed_prop["notion_type"]
                    actions.append(FixAction(
                        action="update_in_per_entry_type",
                        file=path.name,
                        property_name=final_name,
                        details=f"Updated '{final_name}' in {path.name} to match moved property from per_db",
                    ))

    # Step 3: Remove any remaining duplicates from per_entry_type files
    # (properties that are still in per_db or universal after the moves)
    uni_schema = load_yaml(SCHEMAS_DIR / "universal" / "holon_coordinate.yaml")
    uni_props = set((uni_schema.get("properties") or {}).keys())

    for (db, et), (path, pet_schema) in pet_schemas.items():
        pet_props = pet_schema.get("properties", {})
        db_props = set((per_db_schemas.get(db, {}).get("properties") or {}).keys())
        to_remove = []
        for pname in list(pet_props.keys()):
            if pname in uni_props:
                to_remove.append((pname, "universal"))
            elif pname in db_props:
                to_remove.append((pname, f"per_db/{db}.yaml"))
        for pname, layer in to_remove:
            pet_props.pop(pname)
            actions.append(FixAction(
                action="remove_duplicate_from_per_entry_type",
                file=path.name,
                property_name=pname,
                details=f"Removed duplicate '{pname}' from {path.name} (already in {layer})",
            ))

    # Step 4: Save all modified schemas (unless dry-run)
    if dry_run:
        print("\n🔍 DRY RUN — no files modified.")
        return actions

    for db in DB_KEYS:
        path = SCHEMAS_DIR / "per_db" / f"{db}.yaml"
        save_with_header(path, per_db_schemas[db])

    for (db, et), (path, pet_schema) in pet_schemas.items():
        save_with_header(path, pet_schema)

    return actions


def main() -> int:
    parser = argparse.ArgumentParser(description="LifeOS v0.9.0 — Schema Redundancy Fixer")
    parser.add_argument("--dry-run", action="store_true", help="Show what would change without modifying files")
    args = parser.parse_args()

    print("=" * 70)
    print("  Schema Redundancy Fixer")
    print("=" * 70)
    print(f"  Mode: {'DRY RUN' if args.dry_run else 'LIVE EXECUTION'}")
    print()

    actions = execute_fix(dry_run=args.dry_run)

    print(f"\n{len(actions)} fix actions:\n")
    by_action: dict[str, list[FixAction]] = {}
    for a in actions:
        by_action.setdefault(a.action, []).append(a)
    for action_type, acts in by_action.items():
        print(f"  {action_type}: {len(acts)}")
        for a in acts:
            print(f"    - {a.file}: {a.details}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
