#!/usr/bin/env python3
"""
LifeOS v0.9.0 — Schema Redundancy Auditor
==========================================

Detects ALL redundancy violations across the 3-tier YAML schema hierarchy:

  1. Properties in per_entry_type/*.yaml that are ALSO in per_db/<db>.yaml
  2. Properties in per_entry_type/*.yaml that are ALSO in universal/holon_coordinate.yaml
  3. Properties in per_db/*.yaml that are entry-type-specific (description says
     "For X entries" or "Used only for Y-kind") and should be MOVED to
     per_entry_type/*.yaml

Also detects:
  4. Properties in per_db/*.yaml that duplicate universal properties
  5. Properties whose names differ but mean the same thing (e.g.
     `synthesis_state` vs `knowledge_synthesis_state`)

USAGE:
    python3 audit_schema_redundancy.py
    python3 audit_schema_redundancy.py --fix   # auto-fix where safe
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]
SCHEMAS_DIR = REPO_ROOT / "schemas"

DB_KEYS = ("matrix", "potentiator", "nexus", "significator", "greatway")


@dataclass
class RedundancyIssue:
    issue_type: str  # duplicate_in_per_db | duplicate_in_universal | entry_type_specific_in_per_db | name_mismatch
    severity: str    # error | warning
    db: Optional[str]
    entry_type: Optional[str]
    property_name: str
    details: str
    fix_action: str = ""


@dataclass
class AuditReport:
    issues: list[RedundancyIssue] = field(default_factory=list)

    def add(self, issue: RedundancyIssue):
        self.issues.append(issue)

    def summary(self) -> str:
        errors = [i for i in self.issues if i.severity == "error"]
        warnings = [i for i in self.issues if i.severity == "warning"]
        lines = [
            "=" * 70,
            "  Schema Redundancy Audit Report",
            "=" * 70,
            f"  Total issues:   {len(self.issues)}",
            f"  Errors:         {len(errors)}",
            f"  Warnings:       {len(warnings)}",
            "",
        ]
        if errors:
            lines.append("ERRORS (must fix):")
            for i in errors:
                lines.append(f"  ❌ [{i.issue_type}] {i.db or '?'}/{i.entry_type or '?'}: {i.property_name}")
                lines.append(f"     {i.details}")
                if i.fix_action:
                    lines.append(f"     FIX: {i.fix_action}")
                lines.append("")
        if warnings:
            lines.append("WARNINGS (review):")
            for i in warnings:
                lines.append(f"  ⚠️  [{i.issue_type}] {i.db or '?'}/{i.entry_type or '?'}: {i.property_name}")
                lines.append(f"     {i.details}")
                if i.fix_action:
                    lines.append(f"     SUGGEST: {i.fix_action}")
                lines.append("")
        return "\n".join(lines)


def load_yaml(path: Path) -> dict:
    with open(path) as f:
        return yaml.safe_load(f) or {}


def get_property_names(schema: dict) -> set[str]:
    return set((schema.get("properties") or {}).keys())


def get_properties(schema: dict) -> dict[str, dict]:
    return schema.get("properties") or {}


# ── Heuristic: detect entry-type-specific properties in per_db files ──────

ENTRY_TYPE_SPECIFIC_PATTERNS = [
    re.compile(r"\bfor\s+(?:Person|Group|Community|Organization|Network|Movement|Place)\b", re.I),
    re.compile(r"\bfor\s+(?:external-holon|relocated)\b", re.I),
    re.compile(r"\bfor\s+(?:Note|Knowledge|Decision|Crisis|Transformation-Event)\b", re.I),
    re.compile(r"\bfor\s+(?:Principle|Purpose|Value)\s+entries\b", re.I),
    re.compile(r"\bused\s+only\s+for\s+(?:Transformation|Choice|Catalyst|Experience)-kind\b", re.I),
    re.compile(r"\bfor\s+(?:Transformation|Choice|Catalyst|Experience)-kind\s+entries\b", re.I),
    re.compile(r"\bfor\s+(?:Transformation|Choice|Catalyst|Experience)-kind\s+Nexus\b", re.I),
    re.compile(r"\bonly\s+for\s+(?:Transformation|Choice|Catalyst|Experience)-kind\b", re.I),
    re.compile(r"\brequired\s+for\s+(?:external-holon|Person|Group|Community)\b", re.I),
    re.compile(r"\bfor\s+Note-kind\s+entries\b", re.I),
    re.compile(r"\bfor\s+Knowledge-Category\s+entries\b", re.I),
    re.compile(r"\bfor\s+Principle\s+entries\s+only\b", re.I),
]


def is_entry_type_specific(prop_name: str, prop_schema: dict) -> tuple[bool, str]:
    """Check if a property's description indicates it's entry-type-specific.

    Returns (is_specific, matched_pattern_description).
    """
    desc = prop_schema.get("description", "")
    for pattern in ENTRY_TYPE_SPECIFIC_PATTERNS:
        m = pattern.search(desc)
        if m:
            return True, m.group(0)
    return False, ""


# ── Heuristic: detect property name mismatches (same concept, different name) ──

KNOWN_NAME_MISMATCHES = {
    # (per_db name, per_entry_type name) — same concept, different names
    ("knowledge_synthesis_state", "synthesis_state"): "Both refer to the note/knowledge synthesis state",
}


# ── Main audit ────────────────────────────────────────────────────────────

def audit() -> AuditReport:
    report = AuditReport()

    # Load universal
    uni_path = SCHEMAS_DIR / "universal" / "holon_coordinate.yaml"
    uni_schema = load_yaml(uni_path)
    uni_props = get_properties(uni_schema)

    # Load all per_db
    per_db_schemas: dict[str, dict] = {}
    per_db_props: dict[str, dict[str, dict]] = {}
    for db in DB_KEYS:
        path = SCHEMAS_DIR / "per_db" / f"{db}.yaml"
        schema = load_yaml(path)
        per_db_schemas[db] = schema
        per_db_props[db] = get_properties(schema)

    # Load all per_entry_type
    pet_dir = SCHEMAS_DIR / "per_entry_type"
    pet_schemas: dict[tuple[str, str], dict] = {}
    for path in sorted(pet_dir.glob("*.yaml")):
        schema = load_yaml(path)
        db = schema.get("applies_to_db")
        et = schema.get("applies_to_entry_type")
        if db and et:
            pet_schemas[(db, et)] = schema

    # ── Check 1: per_db properties that duplicate universal ─────────────
    for db, props in per_db_props.items():
        for pname, pschema in props.items():
            if pname in uni_props:
                report.add(RedundancyIssue(
                    issue_type="duplicate_in_universal",
                    severity="error",
                    db=db, entry_type=None,
                    property_name=pname,
                    details=f"Property '{pname}' is declared in BOTH universal/holon_coordinate.yaml AND per_db/{db}.yaml. Universal properties apply to all entries — per_db should only declare DB-specific extensions.",
                    fix_action=f"Remove '{pname}' from per_db/{db}.yaml (it's already in universal).",
                ))

    # ── Check 2: per_entry_type properties that duplicate universal ─────
    for (db, et), schema in pet_schemas.items():
        props = get_properties(schema)
        for pname in props:
            if pname in uni_props:
                report.add(RedundancyIssue(
                    issue_type="duplicate_in_universal",
                    severity="error",
                    db=db, entry_type=et,
                    property_name=pname,
                    details=f"Property '{pname}' is in BOTH universal and per_entry_type/{db}__{et}.yaml. Per-entry-type should only declare entry-type-specific extensions.",
                    fix_action=f"Remove '{pname}' from per_entry_type/{db}__{et}.yaml.",
                ))

    # ── Check 3: per_entry_type properties that duplicate per_db ────────
    for (db, et), schema in pet_schemas.items():
        props = get_properties(schema)
        db_props = per_db_props.get(db, {})
        for pname in props:
            if pname in db_props:
                report.add(RedundancyIssue(
                    issue_type="duplicate_in_per_db",
                    severity="error",
                    db=db, entry_type=et,
                    property_name=pname,
                    details=f"Property '{pname}' is in BOTH per_db/{db}.yaml AND per_entry_type/{db}__{et}.yaml. Per-entry-type should only declare entry-type-specific extensions not in per_db.",
                    fix_action=f"Remove '{pname}' from per_entry_type/{db}__{et}.yaml (already in per_db/{db}.yaml).",
                ))

    # ── Check 4: per_db properties that are entry-type-specific ─────────
    for db, props in per_db_props.items():
        for pname, pschema in props.items():
            is_specific, matched = is_entry_type_specific(pname, pschema)
            if is_specific:
                report.add(RedundancyIssue(
                    issue_type="entry_type_specific_in_per_db",
                    severity="error",
                    db=db, entry_type=None,
                    property_name=pname,
                    details=f"Property '{pname}' in per_db/{db}.yaml has description matching '{matched}' — it's entry-type-specific and should be MOVED to the relevant per_entry_type files.",
                    fix_action=f"Remove '{pname}' from per_db/{db}.yaml and ensure it's declared in the relevant per_entry_type/{db}__*.yaml files.",
                ))

    # ── Check 5: known name mismatches ─────────────────────────────────
    for (db_name, pet_name), reason in KNOWN_NAME_MISMATCHES.items():
        # Find which per_db has db_name
        for db, props in per_db_props.items():
            if db_name in props:
                # Find which per_entry_type has pet_name
                for (db2, et), schema in pet_schemas.items():
                    pet_props = get_properties(schema)
                    if pet_name in pet_props:
                        report.add(RedundancyIssue(
                            issue_type="name_mismatch",
                            severity="warning",
                            db=db, entry_type=et,
                            property_name=f"{db_name} vs {pet_name}",
                            details=f"{reason}. per_db/{db}.yaml uses '{db_name}', per_entry_type/{db}__{et}.yaml uses '{pet_name}'. Unify on one name.",
                            fix_action=f"Pick one name (suggest '{pet_name}') and use it consistently. Remove the other.",
                        ))

    return report


def main() -> int:
    parser = argparse.ArgumentParser(description="LifeOS v0.9.0 — Schema Redundancy Auditor")
    parser.add_argument("--fix", action="store_true", help="Auto-fix where safe (not yet implemented)")
    args = parser.parse_args()

    report = audit()
    print(report.summary())

    if args.fix:
        print("\n⚠️  Auto-fix not yet implemented. Please fix manually based on the report above.")

    errors = [i for i in report.issues if i.severity == "error"]
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
