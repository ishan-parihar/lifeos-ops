#!/usr/bin/env python3
"""
LifeOS v0.9.0 — Shared utilities for the Notion migration scripts.

Common functionality used by:
  - 01_add_dual_property_relations.py
  - 02_relocate_entry_types.py
  - 03_port_auxiliary_people_community_to_greatway.py
  - 04_port_notes_knowledge_to_nexus.py
  - 05_auto_tag_existing_entries.py
  - 06_apply_yaml_schemas_to_notion.py

This module is NOT standalone — import from the upgrade scripts.
"""

from __future__ import annotations

import json
import os
import sys
import time
import urllib.request
import urllib.error
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Optional

NOTION_API = "https://api.notion.com"
NOTION_VERSION = "2025-09-03"

DB_KEYS = ("matrix", "potentiator", "nexus", "significator", "greatway")

# Canonical DB names (per lifeos.config.default.json)
DB_NAMES = {
    "matrix": "Matrix",
    "potentiator": "Potentiator",
    "nexus": "Nexus",
    "significator": "Significator",
    "greatway": "GreatWay",
}

# Auxiliary DBs to discover for migration
AUX_DB_NAMES = ["People", "Community", "Group", "Notes", "Knowledge", "Knowledge Base"]


# ─────────────────────────────────────────────────────────────────────────────
# Notion HTTP client (lightweight, retry-aware)
# ─────────────────────────────────────────────────────────────────────────────

class NotionClient:
    def __init__(self, token: Optional[str] = None):
        self.token = token or os.environ.get("NOTION_API_TOKEN")
        if not self.token:
            print("ERROR: NOTION_API_TOKEN environment variable required.", file=sys.stderr)
            sys.exit(2)

    def _headers(self) -> dict:
        return {
            "Authorization": f"Bearer {self.token}",
            "Notion-Version": NOTION_VERSION,
            "Content-Type": "application/json",
        }

    def request(self, method: str, path: str, body: Optional[dict] = None, max_retries: int = 5) -> dict:
        url = f"{NOTION_API}{path}"
        data = json.dumps(body).encode() if body else None
        for attempt in range(max_retries):
            req = urllib.request.Request(url, data=data, headers=self._headers(), method=method)
            try:
                with urllib.request.urlopen(req) as resp:
                    body_bytes = resp.read()
                    return json.loads(body_bytes.decode()) if body_bytes else {}
            except urllib.error.HTTPError as e:
                if e.code == 429 and attempt < max_retries - 1:
                    delay = (2 ** attempt) + 0.5
                    print(f"  ⏳ Rate-limited, retrying in {delay:.1f}s (attempt {attempt + 1}/{max_retries})")
                    time.sleep(delay)
                    continue
                # For other errors, capture the response body
                err_body = e.read().decode() if e.fp else ""
                print(f"  ❌ Notion API {e.code} {method} {path}: {err_body[:500]}", file=sys.stderr)
                raise
            except Exception as e:
                if attempt < max_retries - 1:
                    delay = (2 ** attempt) + 0.5
                    print(f"  ⚠️  Error ({e}), retrying in {delay:.1f}s")
                    time.sleep(delay)
                    continue
                raise
        return {}

    # ── Data source / database endpoints ──
    def get_data_source(self, ds_id: str) -> dict:
        return self.request("GET", f"/v1/data_sources/{ds_id}")

    def get_database(self, db_id: str) -> dict:
        return self.request("GET", f"/v1/databases/{db_id}")

    def update_database(self, db_id: str, body: dict) -> dict:
        """PATCH /v1/databases/{id} — used to add properties and select options."""
        return self.request("PATCH", f"/v1/databases/{db_id}", body)

    def update_data_source(self, ds_id: str, body: dict) -> dict:
        """PATCH /v1/data_sources/{id} — Notion 2025-09-03 supports this for some operations."""
        return self.request("PATCH", f"/v1/data_sources/{ds_id}", body)

    def query_data_source(self, ds_id: str, body: Optional[dict] = None) -> dict:
        return self.request("POST", f"/v1/data_sources/{ds_id}/query", body or {"page_size": 100})

    def query_all_pages(self, ds_id: str, filter_body: Optional[dict] = None) -> list[dict]:
        """Query a data source with pagination; returns all pages."""
        results = []
        cursor = None
        while True:
            body = {"page_size": 100}
            if filter_body:
                body["filter"] = filter_body
            if cursor:
                body["start_cursor"] = cursor
            resp = self.query_data_source(ds_id, body)
            results.extend(resp.get("results", []))
            if resp.get("has_more"):
                cursor = resp.get("next_cursor")
            else:
                break
        return results

    # ── Page endpoints ──
    def get_page(self, page_id: str) -> dict:
        return self.request("GET", f"/v1/pages/{page_id}")

    def create_page(self, body: dict) -> dict:
        return self.request("POST", "/v1/pages", body)

    def update_page(self, page_id: str, properties: dict) -> dict:
        return self.request("PATCH", f"/v1/pages/{page_id}", {"properties": properties})

    def archive_page(self, page_id: str) -> dict:
        return self.request("PATCH", f"/v1/pages/{page_id}", {"archived": True})

    # ── Search ──
    def search(self, query: Optional[str] = None, filter_type: Optional[str] = None) -> list[dict]:
        body = {"page_size": 100}
        if query:
            body["query"] = query
        if filter_type:
            body["filter"] = {"value": filter_type, "property": "object"}
        results = []
        cursor = None
        while True:
            if cursor:
                body["start_cursor"] = cursor
            resp = self.request("POST", "/v1/search", body)
            results.extend(resp.get("results", []))
            if resp.get("has_more"):
                cursor = resp.get("next_cursor")
            else:
                break
        return results


# ─────────────────────────────────────────────────────────────────────────────
# DB discovery helpers
# ─────────────────────────────────────────────────────────────────────────────

def discover_db_ids(client: NotionClient) -> dict[str, str]:
    """Discover all 5 LifeOS DBs (and any auxiliary DBs) by name."""
    found: dict[str, str] = {}
    all_items = client.search(filter_type="data_source")
    for item in all_items:
        title_parts = item.get("title", [])
        title = "".join(p.get("plain_text", "") for p in title_parts).strip()
        # Check against canonical names
        for db_key, expected_name in DB_NAMES.items():
            if title == expected_name and db_key not in found:
                found[db_key] = item["id"]
        # Check against auxiliary names
        for aux_name in AUX_DB_NAMES:
            if title == aux_name:
                found[f"aux:{aux_name}"] = item["id"]
    return found


# ─────────────────────────────────────────────────────────────────────────────
# API 2025-09-03 note:
# In Notion API version 2025-09-03, properties live on the DATA_SOURCE, not the
# database container. The database container is a metadata wrapper; the
# data_source is the actual queryable table with properties.
#
# To READ properties:    GET /v1/data_sources/{id}
# To MODIFY properties:  PATCH /v1/data_sources/{id}
#
# The old approach (PATCH /v1/databases/{container_id}) silently fails to
# update properties in the new API version — it returns 200 but doesn't change
# the data_source's property schema.
#
# For relations, the `database_id` in the relation config should be the DATABASE
# CONTAINER ID (not the data_source ID), because Notion links relations at the
# database level. Use `get_database_container_id` to resolve it.
# ─────────────────────────────────────────────────────────────────────────────

def get_database_container_id(client: NotionClient, data_source_id: str) -> str:
    """Resolve a data_source_id to its parent database container ID.

    In Notion API 2025-09-03, a data_source has `parent.database_id` pointing
    to the database container. This is needed for relation properties, which
    link at the database level.
    """
    ds = client.get_data_source(data_source_id)
    db_id = ds.get("parent", {}).get("database_id") or ds.get("database_id")
    if not db_id:
        db_id = data_source_id
    return db_id


def get_data_source_schema(client: NotionClient, data_source_id: str) -> dict:
    """Fetch the current property schema from a data_source.

    In API 2025-09-03, properties live on the data_source, not the database
    container. Use this instead of `client.get_database()` when you need to
    read the current property definitions.
    """
    ds = client.get_data_source(data_source_id)
    return ds.get("properties", {})


# ─────────────────────────────────────────────────────────────────────────────
# Property mutation helpers (for adding properties + select options + relations)
# ─────────────────────────────────────────────────────────────────────────────

def add_select_property(
    client: NotionClient,
    data_source_id: str,
    prop_name: str,
    options: list[str],
    prop_type: str = "select",
    description: str = "",
) -> dict:
    """Add a select/multi_select/status property to a Notion data_source.

    Uses PATCH /v1/data_sources/{id} (API 2025-09-03).
    """
    if prop_type not in ("select", "multi_select", "status"):
        raise ValueError(f"add_select_property only supports select/multi_select/status, got {prop_type}")

    options_payload = [{"name": opt} for opt in options]
    body = {
        "properties": {
            prop_name: {
                prop_type: {"options": options_payload} if prop_type != "status" else {},
            }
        }
    }

    return client.update_data_source(data_source_id, body)


def add_rich_text_property(client: NotionClient, data_source_id: str, prop_name: str) -> dict:
    body = {"properties": {prop_name: {"rich_text": {}}}}
    return client.update_data_source(data_source_id, body)


def add_number_property(client: NotionClient, data_source_id: str, prop_name: str,
                         number_format: str = "number") -> dict:
    body = {"properties": {prop_name: {"number": {"format": number_format}}}}
    return client.update_data_source(data_source_id, body)


def add_relation_property(
    client: NotionClient,
    data_source_id: str,
    prop_name: str,
    target_database_id: str,
    dual_property: bool = False,
    dual_property_name: Optional[str] = None,
    single_directional: bool = False,
) -> dict:
    """Add a relation property to a Notion data_source.

    In API 2025-09-03, relations are PATCHed on the data_source. The
    `target_database_id` should be the target DB's DATA_SOURCE ID (Notion
    will resolve it to the database container internally).

    Args:
        data_source_id: The source DB's data_source ID.
        target_database_id: The target DB's data_source ID.
        dual_property: If True, create a dual_property (synced) relation.
        dual_property_name: Required if dual_property=True.
    """
    # In API 2025-09-03, relation configs use `data_source_id` (not `database_id`)
    rel_config: dict = {"data_source_id": target_database_id}
    if dual_property:
        rel_config["type"] = "dual_property"
        if not dual_property_name:
            raise ValueError("dual_property_name is required when dual_property=True")
        rel_config["dual_property"] = {"name": dual_property_name}
    else:
        rel_config["type"] = "single_property"
        rel_config["single_property"] = {}

    body = {"properties": {prop_name: {"relation": rel_config}}}
    return client.update_data_source(data_source_id, body)


def add_select_option_to_existing_property(
    client: NotionClient,
    data_source_id: str,
    prop_name: str,
    new_options: list[str],
    prop_type: str = "select",
) -> dict:
    """Add new select/multi_select options to an EXISTING property on a data_source.

    In API 2025-09-03, reads current options from the data_source and PATCHes
    the data_source with the merged options list.
    """
    if prop_type == "status":
        raise ValueError("Cannot add options to a status property via Notion API — must use Notion UI.")

    ds_schema = get_data_source_schema(client, data_source_id)
    existing_prop = ds_schema.get(prop_name)
    if not existing_prop:
        raise ValueError(f"Property '{prop_name}' not found on data_source {data_source_id}. Available: {list(ds_schema.keys())}")

    existing_options = [o.get("name") for o in existing_prop.get(prop_type, {}).get("options", [])]
    merged = list(dict.fromkeys(existing_options + new_options))  # dedupe, preserve order

    body = {
        "properties": {
            prop_name: {
                prop_type: {"options": [{"name": o} for o in merged]}
            }
        }
    }
    return client.update_data_source(data_source_id, body)


# ─────────────────────────────────────────────────────────────────────────────
# Pretty-printing helpers
# ─────────────────────────────────────────────────────────────────────────────

def print_section(title: str) -> None:
    print()
    print("─" * 70)
    print(f"  {title}")
    print("─" * 70)


def print_kv(k: str, v: Any, indent: int = 4) -> None:
    if isinstance(v, (dict, list)):
        v_str = json.dumps(v, indent=2, default=str)
        # Indent multi-line values
        v_str = v_str.replace("\n", "\n" + " " * indent)
        print(f"{' ' * indent}{k}: {v_str}")
    else:
        print(f"{' ' * indent}{k}: {v}")


# ─────────────────────────────────────────────────────────────────────────────
# Migration bookkeeping
# ─────────────────────────────────────────────────────────────────────────────

@dataclass
class MigrationLog:
    """Persistent record of what was migrated, for verification and rollback."""
    script_name: str
    started_at: str
    finished_at: Optional[str] = None
    operations: list[dict] = None
    errors: list[dict] = None
    warnings: list[dict] = None

    def __post_init__(self):
        if self.operations is None:
            self.operations = []
        if self.errors is None:
            self.errors = []
        if self.warnings is None:
            self.warnings = []

    def to_dict(self) -> dict:
        return {
            "script_name": self.script_name,
            "started_at": self.started_at,
            "finished_at": self.finished_at,
            "operations_count": len(self.operations),
            "errors_count": len(self.errors),
            "warnings_count": len(self.warnings),
            "operations": self.operations,
            "errors": self.errors,
            "warnings": self.warnings,
        }

    def save(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "w") as f:
            json.dump(self.to_dict(), f, indent=2, default=str)
        print(f"\n📝 Migration log saved to: {path}")
