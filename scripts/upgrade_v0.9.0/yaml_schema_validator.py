#!/usr/bin/env python3
"""
LifeOS v0.9.0 — YAML Schema Validator (Python reference implementation)

Validates LifeOS Notion entries against the 3-tier YAML schema hierarchy:
    1. universal/holon_coordinate.yaml          (every entry)
    2. per_db/<db>.yaml                          (every entry in a given DB)
    3. per_entry_type/<db>__<entry_type>.yaml    (entries of one type in one DB)

A property is required for an entry IFF it is `required: true` at ANY level
that applies to that entry (universal → per_db → per_entry_type).

USAGE:
    # Validate a single Notion entry (by page ID) against its schema
    NOTION_API_TOKEN=xxx python3 yaml_schema_validator.py --page-id <id>

    # Validate all entries in a single DB
    NOTION_API_TOKEN=xxx python3 yaml_schema_validator.py --db matrix

    # Validate all entries in all 5 DBs
    NOTION_API_TOKEN=xxx python3 yaml_schema_validator.py --all

    # Dry-run: just load and self-validate the schema files
    python3 yaml_schema_validator.py --self-test

This is the reference implementation. The Rust port at
    lifeos-core/src/util/yaml_schemas.rs
mirrors these semantics and is invoked by the `lifeos validate-yaml` CLI command.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Optional

import yaml

# ─────────────────────────────────────────────────────────────────────────────
# Constants
# ─────────────────────────────────────────────────────────────────────────────

REPO_ROOT = Path(__file__).resolve().parents[2]  # lifeos-ops/
SCHEMAS_DIR = REPO_ROOT / "schemas"
NOTION_API = "https://api.notion.com/v1"
NOTION_VERSION = "2025-09-03"

DB_KEYS = ("matrix", "potentiator", "nexus", "significator", "greatway")

VALID_NOTION_TYPES = {
    "title", "rich_text", "select", "multi_select", "status", "date",
    "number", "checkbox", "people", "relation", "url", "email",
    "phone_number", "files", "formula", "rollup",
    "created_time", "last_edited_time", "created_by", "last_edited_by",
    "unique_id", "button",
}

# DB → entry-type property name (per lifeos.config.default.json)
ENTRY_TYPE_PROP = {
    "matrix": "Entry Type",
    "potentiator": "Entry Type",
    "significator": "Entry Type",
    "greatway": "Item Type",
    "nexus": "Category",
}

# DB → currency property name (Nexus only, per config)
CURRENCY_PROP = {"nexus": "Kind"}


# ─────────────────────────────────────────────────────────────────────────────
# Data structures
# ─────────────────────────────────────────────────────────────────────────────

@dataclass
class PropertySchema:
    name: str
    notion_type: str
    required: bool
    description: str = ""
    options: list[str] = field(default_factory=list)


@dataclass
class ValidationRule:
    rule_id: str
    description: str
    rule_expr: str
    applies_to_db: Optional[str] = None


@dataclass
class SchemaLayer:
    """One tier of the 3-tier schema hierarchy."""
    schema_type: str  # universal | per_db | per_entry_type
    applies_to_db: Optional[str]
    applies_to_entry_type: Optional[str]
    properties: dict[str, PropertySchema]
    validation_rules: list[ValidationRule]
    raw: dict


@dataclass
class ValidationError:
    db: str
    entry_type: Optional[str]
    page_id: Optional[str]
    page_title: Optional[str]
    layer: str  # universal | per_db | per_entry_type | cross-property
    property_name: Optional[str]
    rule_id: str
    severity: str  # error | warning
    message: str


@dataclass
class ValidationResult:
    valid: bool
    errors: list[ValidationError]
    warnings: list[ValidationError]
    entry_count: int = 0
    validated_count: int = 0

    def summary(self) -> str:
        lines = [
            f"Validation Result: {'✅ PASS' if self.valid else '❌ FAIL'}",
            f"  Entries scanned:    {self.entry_count}",
            f"  Entries validated:  {self.validated_count}",
            f"  Errors:             {len(self.errors)}",
            f"  Warnings:           {len(self.warnings)}",
        ]
        return "\n".join(lines)


# ─────────────────────────────────────────────────────────────────────────────
# Schema loading
# ─────────────────────────────────────────────────────────────────────────────

class SchemaRegistry:
    """Loads and caches all 3 tiers of YAML schemas."""

    def __init__(self, schemas_dir: Path = SCHEMAS_DIR):
        self.schemas_dir = schemas_dir
        self.universal: Optional[SchemaLayer] = None
        self.per_db: dict[str, SchemaLayer] = {}
        self.per_entry_type: dict[tuple[str, str], SchemaLayer] = {}
        self.load_errors: list[str] = []

    def load_all(self) -> None:
        # Universal
        uni_path = self.schemas_dir / "universal" / "holon_coordinate.yaml"
        if uni_path.exists():
            self.universal = self._load_layer(uni_path, "universal")
        else:
            self.load_errors.append(f"Missing universal schema: {uni_path}")

        # Per-DB
        for db in DB_KEYS:
            path = self.schemas_dir / "per_db" / f"{db}.yaml"
            if path.exists():
                layer = self._load_layer(path, "per_db")
                if layer.applies_to_db == db:
                    self.per_db[db] = layer
                else:
                    self.load_errors.append(
                        f"per_db schema {path} applies_to_db={layer.applies_to_db}, expected {db}"
                    )
            else:
                self.load_errors.append(f"Missing per_db schema: {path}")

        # Per-entry-type
        pet_dir = self.schemas_dir / "per_entry_type"
        if pet_dir.exists():
            for path in sorted(pet_dir.glob("*.yaml")):
                layer = self._load_layer(path, "per_entry_type")
                if layer.applies_to_db and layer.applies_to_entry_type:
                    self.per_entry_type[(layer.applies_to_db, layer.applies_to_entry_type)] = layer

    def _load_layer(self, path: Path, schema_type: str) -> SchemaLayer:
        with open(path) as f:
            raw = yaml.safe_load(f) or {}

        props: dict[str, PropertySchema] = {}
        for name, p in (raw.get("properties") or {}).items():
            props[name] = PropertySchema(
                name=name,
                notion_type=p.get("notion_type", "rich_text"),
                required=bool(p.get("required", False)),
                description=p.get("description", ""),
                options=list(p.get("options") or []),
            )

        rules: list[ValidationRule] = []
        for r in (raw.get("validation_rules") or []):
            rules.append(ValidationRule(
                rule_id=r["id"],
                description=r.get("description", ""),
                rule_expr=r.get("rule", ""),
                applies_to_db=r.get("applies_to_db"),
            ))

        return SchemaLayer(
            schema_type=schema_type,
            applies_to_db=raw.get("applies_to_db"),
            applies_to_entry_type=raw.get("applies_to_entry_type"),
            properties=props,
            validation_rules=rules,
            raw=raw,
        )

    def layers_for(self, db: str, entry_type: Optional[str]) -> list[SchemaLayer]:
        """Return the 3 applicable schema layers for a (db, entry_type) pair."""
        layers: list[SchemaLayer] = []
        if self.universal:
            layers.append(self.universal)
        if db in self.per_db:
            layers.append(self.per_db[db])
        if entry_type and (db, entry_type) in self.per_entry_type:
            layers.append(self.per_entry_type[(db, entry_type)])
        return layers

    # ── Self-test: validate the schema files themselves ──────────────
    def self_test(self) -> list[str]:
        issues: list[str] = []
        # 1. Universal must exist
        if not self.universal:
            issues.append("universal/holon_coordinate.yaml not loaded")
            return issues
        # 2. All 5 per_db must exist
        for db in DB_KEYS:
            if db not in self.per_db:
                issues.append(f"per_db/{db}.yaml not loaded")
        # 3. All properties must have valid Notion types
        for layer in [self.universal] + list(self.per_db.values()) + list(self.per_entry_type.values()):
            for pname, ps in layer.properties.items():
                if ps.notion_type not in VALID_NOTION_TYPES:
                    issues.append(
                        f"{layer.schema_type} {layer.applies_to_db}/{layer.applies_to_entry_type}: "
                        f"property '{pname}' has invalid notion_type '{ps.notion_type}'"
                    )
        # 4. Cross-check: every entry-type mentioned in per_db.<db>.entry_types must
        #    have a corresponding per_entry_type file
        for db, layer in self.per_db.items():
            declared = set(layer.raw.get("entry_types") or [])
            actual = {et for (d, et) in self.per_entry_type.keys() if d == db}
            missing = declared - actual
            for m in sorted(missing):
                issues.append(f"per_db/{db}.yaml declares entry-type '{m}' but no per_entry_type file exists")
        # 5. Every per_entry_type file's applies_to_db must be a valid DB
        for (db, et), layer in self.per_entry_type.items():
            if db not in DB_KEYS:
                issues.append(f"per_entry_type file has invalid applies_to_db: {db} (for entry-type {et})")
        return issues


# ─────────────────────────────────────────────────────────────────────────────
# Notion entry extraction
# ─────────────────────────────────────────────────────────────────────────────

class NotionFetcher:
    """Fetches Notion entries and extracts the property values needed for validation."""

    def __init__(self, token: str):
        import urllib.request
        self.token = token
        self._urllib = urllib.request

    def _headers(self) -> dict:
        return {
            "Authorization": f"Bearer {self.token}",
            "Notion-Version": NOTION_VERSION,
            "Content-Type": "application/json",
        }

    def _request(self, method: str, path: str, body: Optional[dict] = None) -> dict:
        url = f"{NOTION_API}{path}"
        data = json.dumps(body).encode() if body else None
        req = self._urllib.Request(url, data=data, headers=self._headers(), method=method)
        max_retries = 5
        for attempt in range(max_retries):
            try:
                with self._urllib.urlopen(req) as resp:
                    return json.loads(resp.read().decode())
            except Exception as e:
                if hasattr(e, "code") and e.code == 429 and attempt < max_retries - 1:
                    time.sleep(2 ** attempt + 0.5)
                    continue
                raise

    def discover_db_ids(self) -> dict[str, str]:
        """Search Notion for the 5 LifeOS databases by name."""
        body = {"filter": {"value": "data_source", "property": "object"}, "page_size": 100}
        found = {}
        cursor = None
        while True:
            if cursor:
                body["start_cursor"] = cursor
            resp = self._request("POST", "/v1/search", body)
            for item in resp.get("results", []):
                title_parts = item.get("title", [])
                title = "".join(p.get("plain_text", "") for p in title_parts).strip()
                # Match by canonical name (per lifeos.config.default.json)
                for db_key, expected_name in [("matrix", "Matrix"), ("potentiator", "Potentiator"),
                                              ("significator", "Significator"), ("greatway", "GreatWay"),
                                              ("nexus", "Nexus")]:
                    if title == expected_name and db_key not in found:
                        found[db_key] = item["id"]
            if resp.get("has_more"):
                cursor = resp.get("next_cursor")
            else:
                break
        return found

    def query_all(self, data_source_id: str, page_size: int = 100) -> list[dict]:
        """Query all pages from a data source (handles pagination)."""
        results = []
        cursor = None
        while True:
            body = {"page_size": page_size}
            if cursor:
                body["start_cursor"] = cursor
            resp = self._request("POST", f"/v1/data_sources/{data_source_id}/query", body)
            results.extend(resp.get("results", []))
            if resp.get("has_more"):
                cursor = resp.get("next_cursor")
            else:
                break
        return results

    def get_page(self, page_id: str) -> dict:
        return self._request("GET", f"/v1/pages/{page_id}")


# ─────────────────────────────────────────────────────────────────────────────
# Property extraction (Notion → flat dict for validation)
# ─────────────────────────────────────────────────────────────────────────────

def extract_property_value(prop_name: str, prop_value: dict) -> Any:
    """Convert a Notion property value into a Python value for validation."""
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
    if ptype == "people":
        return [p.get("name") or p.get("id") for p in prop_value.get("people", [])]
    if ptype == "relation":
        return [r.get("id") for r in prop_value.get("relation", [])]
    if ptype == "formula":
        f = prop_value.get("formula", {})
        return f.get("string") or f.get("number") or f.get("boolean")
    return None


def extract_entry_dict(page: dict, db_key: str) -> tuple[Optional[str], dict]:
    """Extract the entry_type and a flat dict of property values from a Notion page."""
    props = page.get("properties", {})
    et_prop_name = ENTRY_TYPE_PROP.get(db_key, "Entry Type")
    entry_type = None
    flat: dict[str, Any] = {}
    for name, value in props.items():
        pyval = extract_property_value(name, value)
        # The entry_type property is named differently per DB — normalize it to "entry_type"
        if name == et_prop_name:
            entry_type = pyval
            flat["entry_type"] = pyval
            # Also alias the Nexus currency property
        flat[name] = pyval
    # Normalize Nexus "Kind" property
    if db_key == "nexus":
        flat["kind"] = flat.get("Kind")
    # Title fallback
    flat["title"] = "".join(t.get("plain_text", "") for t in props.get("Name", {}).get("title", [])) or \
                    "".join(t.get("plain_text", "") for t in props.get("Title", {}).get("title", []))
    return entry_type, flat


# ─────────────────────────────────────────────────────────────────────────────
# Validation engine
# ─────────────────────────────────────────────────────────────────────────────

def _snake(s: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", s.lower()).strip("_")


def validate_entry(
    db_key: str,
    page: dict,
    registry: SchemaRegistry,
) -> tuple[list[ValidationError], list[ValidationError]]:
    """Validate a single Notion entry against its applicable schema layers."""
    errors: list[ValidationError] = []
    warnings: list[ValidationError] = []
    entry_type, flat = extract_entry_dict(page, db_key)
    page_id = page.get("id")
    page_title = flat.get("title") or "<untitled>"

    layers = registry.layers_for(db_key, entry_type)
    if not layers:
        warnings.append(ValidationError(db_key, entry_type, page_id, page_title,
                                        "universal", None, "no-schema", "warning",
                                        "No applicable schema layers found."))
        return errors, warnings

    # ── 1. Per-property type + required + options checks ──────────
    # Aggregate property schemas across all layers (later layers override earlier).
    merged_props: dict[str, PropertySchema] = {}
    for layer in layers:
        for pname, ps in layer.properties.items():
            # Snake-case alias matching: a config-key alias can match a Notion property name
            merged_props[pname] = ps
            merged_props[_snake(pname)] = ps

    for pname, ps in merged_props.items():
        # Skip if this is just a snake-case alias of an already-checked property
        if pname != _snake(pname) and _snake(pname) in merged_props and pname != ps.name:
            continue

        value = flat.get(pname) or flat.get(_snake(pname)) or flat.get(ps.name)
        # Required check
        if ps.required and (value is None or value == "" or value == []):
            errors.append(ValidationError(
                db_key, entry_type, page_id, page_title,
                "any", pname, "required-missing", "error",
                f"Required property '{pname}' is missing or empty.",
            ))
            continue
        # Skip further checks if value is empty
        if value is None or value == "" or value == []:
            continue
        # Options check (for select/multi_select/status)
        if ps.options and ps.notion_type in ("select", "multi_select", "status"):
            if ps.notion_type == "multi_select":
                bad = [v for v in value if v not in ps.options]
            else:
                bad = [value] if value not in ps.options else []
            if bad:
                errors.append(ValidationError(
                    db_key, entry_type, page_id, page_title,
                    "any", pname, "invalid-option", "error",
                    f"Property '{pname}' has value(s) {bad} not in allowed options: {ps.options}",
                ))

    # ── 2. Cross-property validation rules ────────────────────────
    # Aggregate rules across all layers.
    all_rules: list[tuple[SchemaLayer, ValidationRule]] = []
    for layer in layers:
        for r in layer.validation_rules:
            all_rules.append((layer, r))

    for layer, rule in all_rules:
        # Filter by applies_to_db if set
        if rule.applies_to_db and rule.applies_to_db != db_key:
            continue
        try:
            ok = _eval_rule(rule.rule_expr, flat)
            if not ok:
                errors.append(ValidationError(
                    db_key, entry_type, page_id, page_title,
                    "cross-property", None, rule.rule_id, "error",
                    f"Validation rule '{rule.rule_id}' failed: {rule.description}",
                ))
        except RuleEvalError as e:
            warnings.append(ValidationError(
                db_key, entry_type, page_id, page_title,
                "cross-property", None, rule.rule_id, "warning",
                f"Validation rule '{rule.rule_id}' could not be evaluated: {e}",
            ))

    return errors, warnings


# ─────────────────────────────────────────────────────────────────────────────
# Mini rule evaluator
# ─────────────────────────────────────────────────────────────────────────────

class RuleEvalError(Exception):
    pass


def _eval_rule(rule_expr: str, entry: dict) -> bool:
    """Evaluate a YAML-schema validation rule against an entry dict.

    Supported rule syntax (a small Python-like DSL):
        if entry.<prop> == "<value>":
            assert_no_relations(entry, ["<rel1>", "<rel2>"])
        if entry.<prop> is not None:
            assert entry.<prop> parses as valid YAML
            assert "<key>" in entry.<prop>
        assert (entry.<a> is None) == (entry.<b> is None)
        assert entry.<prop> in {"<v1>", "<v2>", ...}
        assert entry.<prop> == "<value>"
        assert entry.<prop> in ["<v1>", "<v2>"]
    """
    # Build a safe eval namespace with the entry's properties
    safe_globals = {"__builtins__": {}, "True": True, "False": False, "None": None}
    safe_locals = {"entry": _EntryWrapper(entry),
                   "assert_no_relations": _make_assert_no_relations(entry),
                   "assert": _make_assert(),
                   "parses_as_valid_yaml": _parses_as_valid_yaml}

    # Translate the rule_expr into something executable
    # We support two patterns:
    #   1. if/elif/else block ending in assert or assert_no_relations
    #   2. bare assert(...)
    # Strip comments and normalize whitespace
    expr = rule_expr.strip()
    try:
        exec(compile(expr, "<rule>", "exec"), safe_globals, safe_locals)
        return True
    except AssertionError:
        return False
    except Exception as e:
        raise RuleEvalError(f"Rule evaluation failed: {e}\nRule:\n{rule_expr}")


@dataclass
class _EntryWrapper:
    """Wraps an entry dict so you can use entry.prop syntax in rules."""
    _data: dict

    def __getattr__(self, name: str) -> Any:
        if name == "_data":
            return self._data
        return self._data.get(name) or self._data.get(_snake(name))

    def __getitem__(self, key: str) -> Any:
        return self._data.get(key)


def _make_assert_no_relations(entry: dict):
    def assert_no_relations(ent, prop_names: list[str]):
        for pname in prop_names:
            val = ent.get(pname) if isinstance(ent, dict) else getattr(ent, pname, None)
            if val:  # non-empty relation list
                raise AssertionError(f"Relation '{pname}' is populated: {val}")
    return assert_no_relations


def _make_assert():
    def _assert(condition, *args):
        if not condition:
            msg = " ".join(str(a) for a in args) if args else "assertion failed"
            raise AssertionError(msg)
    return _assert


def _parses_as_valid_yaml(s: str) -> bool:
    if not s:
        return False
    try:
        yaml.safe_load(s)
        return True
    except Exception:
        return False


# ─────────────────────────────────────────────────────────────────────────────
# Main entry points
# ─────────────────────────────────────────────────────────────────────────────

def self_test() -> int:
    """Load all schema files and validate the schemas themselves."""
    print(f"Loading schemas from {SCHEMAS_DIR}...")
    registry = SchemaRegistry()
    registry.load_all()

    if registry.load_errors:
        print("\n❌ Schema load errors:")
        for e in registry.load_errors:
            print(f"  - {e}")
        return 1

    issues = registry.self_test()
    if issues:
        print(f"\n❌ {len(issues)} schema self-test issues found:")
        for i in issues:
            print(f"  - {i}")
        return 1

    print(f"\n✅ All schemas passed self-test.")
    print(f"  Universal layer:        1 schema, {len(registry.universal.properties)} properties, {len(registry.universal.validation_rules)} rules")
    for db in DB_KEYS:
        layer = registry.per_db[db]
        pet_count = sum(1 for (d, _) in registry.per_entry_type if d == db)
        print(f"  per_db/{db}.yaml:        {len(layer.properties)} properties, "
              f"{len(layer.validation_rules)} rules, {len(layer.raw.get('entry_types', []))} entry-types declared, "
              f"{pet_count} per_entry_type files")
    print(f"  per_entry_type total:   {len(registry.per_entry_type)} schemas")
    return 0


def validate_db(db_key: str, token: str, registry: SchemaRegistry, limit: int = 0) -> ValidationResult:
    """Validate all entries in a single DB."""
    fetcher = NotionFetcher(token)
    db_ids = fetcher.discover_db_ids()
    if db_key not in db_ids:
        return ValidationResult(valid=False, errors=[ValidationError(
            db_key, None, None, None, "any", None, "db-not-found", "error",
            f"Could not discover DB '{db_key}' in Notion workspace. Found: {list(db_ids.keys())}",
        )], warnings=[], entry_count=0)

    print(f"Querying {db_key} ({db_ids[db_key]})...")
    pages = fetcher.query_all(db_ids[db_key])
    if limit:
        pages = pages[:limit]

    all_errors: list[ValidationError] = []
    all_warnings: list[ValidationError] = []
    for page in pages:
        errs, warns = validate_entry(db_key, page, registry)
        all_errors.extend(errs)
        all_warnings.extend(warns)

    return ValidationResult(
        valid=len(all_errors) == 0,
        errors=all_errors,
        warnings=all_warnings,
        entry_count=len(pages),
        validated_count=len(pages),
    )


def validate_all(token: str, registry: SchemaRegistry) -> ValidationResult:
    all_errors: list[ValidationError] = []
    all_warnings: list[ValidationError] = []
    total = 0
    for db in DB_KEYS:
        print(f"\n── Validating {db} ──────────────────")
        result = validate_db(db, token, registry)
        print(result.summary())
        all_errors.extend(result.errors)
        all_warnings.extend(result.warnings)
        total += result.entry_count

    return ValidationResult(
        valid=len(all_errors) == 0,
        errors=all_errors,
        warnings=all_warnings,
        entry_count=total,
        validated_count=total,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="LifeOS v0.9.0 YAML Schema Validator")
    parser.add_argument("--self-test", action="store_true", help="Validate the schema files themselves (no Notion API).")
    parser.add_argument("--db", choices=DB_KEYS, help="Validate all entries in a single DB.")
    parser.add_argument("--all", action="store_true", help="Validate all entries in all 5 DBs.")
    parser.add_argument("--page-id", help="Validate a single Notion entry by page ID.")
    parser.add_argument("--limit", type=int, default=0, help="Max entries per DB (0 = unlimited).")
    parser.add_argument("--json", action="store_true", help="Output JSON instead of human-readable text.")
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    token = os.environ.get("NOTION_API_TOKEN")
    if not token:
        print("ERROR: NOTION_API_TOKEN environment variable is required for --db, --all, or --page-id.", file=sys.stderr)
        print("Run --self-test to validate the schema files without a token.", file=sys.stderr)
        return 2

    registry = SchemaRegistry()
    registry.load_all()
    if registry.load_errors:
        print("Schema load errors:", file=sys.stderr)
        for e in registry.load_errors:
            print(f"  - {e}", file=sys.stderr)
        return 1

    if args.all:
        result = validate_all(token, registry)
    elif args.db:
        result = validate_db(args.db, token, registry, limit=args.limit)
    elif args.page_id:
        fetcher = NotionFetcher(token)
        page = fetcher.get_page(args.page_id)
        # Detect DB from page parent
        parent = page.get("parent", {})
        ds_id = parent.get("data_source_id") or parent.get("database_id")
        db_ids = fetcher.discover_db_ids()
        db_key = next((k for k, v in db_ids.items() if v == ds_id), None)
        if not db_key:
            print(f"ERROR: Could not determine DB for page {args.page_id} (data_source_id={ds_id})", file=sys.stderr)
            return 1
        errs, warns = validate_entry(db_key, page, registry)
        result = ValidationResult(valid=not errs, errors=errs, warnings=warns, entry_count=1, validated_count=1)
    else:
        parser.print_help()
        return 1

    print()
    print(result.summary())
    if args.json:
        print(json.dumps({
            "valid": result.valid,
            "entry_count": result.entry_count,
            "error_count": len(result.errors),
            "warning_count": len(result.warnings),
            "errors": [e.__dict__ for e in result.errors[:50]],
            "warnings": [w.__dict__ for w in result.warnings[:20]],
        }, indent=2, default=str))
    else:
        if result.errors:
            print(f"\nFirst 20 errors:")
            for e in result.errors[:20]:
                print(f"  ❌ [{e.db}/{e.entry_type or '?'}] {e.page_title or '?'}: {e.message}")
        if result.warnings:
            print(f"\nFirst 10 warnings:")
            for w in result.warnings[:10]:
                print(f"  ⚠️  [{w.db}/{w.entry_type or '?'}] {w.page_title or '?'}: {w.message}")

    return 0 if result.valid else 1


if __name__ == "__main__":
    sys.exit(main())
