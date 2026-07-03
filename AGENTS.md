# AGENTS.md — Guide for AI Agents Working on LifeOS

> **Read this first.** This document gives AI agents the complete context needed to work effectively on the LifeOS codebase.

---

## Project Overview

LifeOS is a **holonic operating system** built as a Rust CLI + MCP server on top of Notion. It operationalizes the [HoloOS](https://github.com/ishan-parihar/HoloOS) ontological architecture into 5 Notion databases.

**You are working on the `lifeos-ops` repo.** The `HoloOS` repo (sibling) contains the theoretical ontology docs. Read [ONTOLOGY.md](ONTOLOGY.md) for the distilled ontological foundation.

---

## Architecture Quick Reference

### 5 Databases (the only DBs — no auxiliary DBs)

| DB | Role | Entry-Type Property | Currency Property |
|----|------|---------------------|-------------------|
| Matrix | Current-state organizer | `Entry Type` (multi_select) | — |
| Potentiator | Latent-state generator | `Entry Type` (select) | — |
| Nexus | Contact boundary | `Category` (select) | `Kind` (select: Catalyst/Experience/Transformation/Choice) |
| Significator | Persistent identity | `Entry Type` (multi_select) | — |
| GreatWay | Operating environment | `Item Type` (select) | — |

### 6 Universal Properties (on all 5 DBs)

1. `Archetype Role` (select, 8 options) — Matrix/Potentiator/Catalyst/Experience/Significator/Transformation/Great Way/Choice
2. `Complex` (select, 4 options) — Mind/Body/Spirit/None
3. `Drive Activation` (multi_select, 4 options) — Agency/Communion/Eros/Agape
4. `Shadow Pattern` (select, 6 options) — None/Dark-Addiction/Dark-Allergy/Golden-Addiction/Golden-Allergy/Sinkhole of Indifference
5. `Digestion Stage` (select, 9 options) — on Nexus + Potentiator only
6. `Holon Type` (select, 5 options) — on Significator only

### Notion API Version: 2025-09-03

**CRITICAL:** In this API version, properties live on the **data_source**, not the database container.
- **READ** properties: `GET /v1/data_sources/{id}`
- **MODIFY** properties: `PATCH /v1/data_sources/{id}` (NOT `/v1/databases/{container_id}`)
- **Relation configs** use `data_source_id` (NOT `database_id`)
- The `common.py` script in `scripts/upgrade_v0.9.0/` has the correct helpers

---

## Codebase Structure

```
lifeos-core/src/
├── notion/           # Notion API client + types
├── config.rs         # LifeOSConfig, DbConfig, HolonicConfig
├── util/
│   ├── schema_engine.rs   # SchemaCache — auto-discovers properties from Notion
│   ├── yaml_schemas.rs    # YamlSchemaRegistry — 3-tier schema validator
│   ├── id_resolver.rs     # Fuzzy name → page ID resolution
│   └── date_filter.rs     # Date-range filter construction
├── tools/            # 29 MCP/CLI tools (one file per tool or tool group)
│   ├── mod.rs        # Tool registration + dispatch (THE registry)
│   ├── query.rs      # Query + query_override
│   ├── mutate.rs     # Create/update/delete (with Nexus Kind validation)
│   ├── relations.rs  # get_page, expand, trace, ancestors, backlinks, link
│   ├── relational_gaps.rs    # Surface orphaned entries
│   ├── build_context.rs      # Assemble relational neighborhood
│   ├── holonic_synthesis.rs  # Trace currency flow + health metrics
│   ├── suggest_categorization.rs  # Suggest entry-types (read-only)
│   ├── relational_graph.rs   # High-level relation tree
│   ├── relation_ops.rs       # unlink + batch_link
│   ├── workflows.rs          # daily + dashboard commands
│   ├── validate_yaml.rs      # YAML schema validator
│   ├── ontology.rs           # archetype_index, derive_type, valence_signature
│   └── ...
├── transform/        # Notion blocks ↔ markdown
├── sync/             # Bidirectional Notion ↔ vault sync
└── server.rs         # MCP JSON-RPC server
```

### How to Add a New Tool

1. Create `lifeos-core/src/tools/my_tool.rs` with:
   - `pub struct MyToolParams { ... }` (derive `Deserialize`)
   - `pub fn schema() -> serde_json::Value { ... }`
   - `pub async fn execute(params, config, notion, schema_cache) -> Result<String, String> { ... }`

2. Register in `lifeos-core/src/tools/mod.rs`:
   - Add `pub mod my_tool;`
   - Add `tool_def("my_tool", "description", my_tool::schema())` to `get_tool_definitions`
   - Add dispatch arm to `call_tool`

3. Add CLI command in `lifeos-core/src/cli/mod.rs` + wire in `lifeos/src/main.rs`

4. Build + test: `cargo build --target x86_64-unknown-linux-gnu && ./target/.../lifeos my-tool --help`

---

## Design Principles (NON-NEGOTIABLE)

### 1. Every relation is a deliberate choice
- Tools **surface** gaps and **suggest** connections
- The user (or AI agent acting explicitly) must approve each link
- **NO auto-population** of relations, entry-types, or properties
- Write tools accept explicit parameters only

### 2. Schema tracks what IS used, not what COULD be
- If a property has 0% fill rate after 30 days, delete it
- Don't add speculative properties "for the future"
- The YAML schema hierarchy (universal → per_db → per_entry_type) validates, not stores

### 3. The 5 DBs are the ONLY DBs
- No auxiliary DBs (People, Community, etc. were ported INTO GreatWay)
- If you need a new entry-type, add it to an existing DB's select options
- Currencies (Catalyst, Experience, Choice) live as entry-types in Nexus, not separate DBs

### 4. Type ⊥ Stage
- `Holon Type` (Donor/Acceptor/Sharer/Multivalent/Noble) is STABLE
- `Digestion Stage` (1-9) is DYNAMIC
- They are independent — never collapse them

---

## Common Tasks

### "I need to query entries in a DB"
Use `query` tool or `lifeos query --database <db> --filter-property <prop> --filter-value <val>`

### "I need to create a new entry"
Use `mutate` tool or `lifeos mutate --operation create --database <db> --properties '{"Name": "...", "Entry Type": "..."}'`

### "I need to link two entries"
- By ID: `lifeos link --source <id> --target <id> --property <prop>`
- By title: `lifeos quick-link --source-db <db> --source-title "..." --target-db <db> --target-title "..." --property <prop>`

### "I need to see what's orphaned"
`lifeos relational-gaps` or `lifeos orphans`

### "I need to understand an entry's context"
`lifeos build-context --page-id <id> --depth 2`

### "I need to see the overall relation structure"
`lifeos relational-graph`

### "I need to validate entries against schemas"
`lifeos validate-yaml --self-test` (schemas only) or `lifeos validate-yaml --all` (entries)

---

## Testing

```bash
# Build
cargo build --target x86_64-unknown-linux-gnu

# Schema self-test (no Notion API needed)
./target/x86_64-unknown-linux-gnu/debug/lifeos validate-yaml --self-test

# Verify CLI commands
./target/x86_64-unknown-linux-gnu/debug/lifeos --help

# Verify MCP tools count
grep 'tool_def(' lifeos-core/src/tools/mod.rs | wc -l
```

---

## Ontology Reference

Read [ONTOLOGY.md](ONTOLOGY.md) for the complete ontological foundation. Key concepts:

- **8 functional roles** → 5 DBs (3 currencies live in Nexus)
- **Two cycles**: Lesser (Matrix⇌Potentiator) + Greater (Significator⇌GreatWay)
- **Nexus = contact boundary** — shared between both cycles, `Kind` discriminates currency
- **4 drives**: Agency, Communion, Eros, Agape
- **G_z × P_z** = Total Metabolic Health (balance × commitment)
- **5 holon types**: Donor, Acceptor, Sharer, Multivalent, Noble
- **22 archetypes** = 7 roles × 3 complexes + Choice

---

## Commit Conventions

- `feat:` new feature
- `fix:` bug fix
- `refactor:` code restructure (no behavior change)
- `audit:` audit/analysis document
- `plan:` planning document
- `docs:` documentation only

Always push to `upgrade/v0.9.0` branch (the active development branch).

---

## Known Issues

1. **musl build requires musl-tools** — not available in all environments. GNU build works everywhere.
2. **Potentiator has 6,876 entries** — some operations are slow due to Notion API rate limits.
3. **3 relations need manual conversion** (single_property → dual_property) — see audit docs.
4. **77% of Matrix entries uncategorized** — use `lifeos suggest-categorization --db matrix` to get suggestions.

---

## Contact

- Repo: [github.com/ishan-parihar/lifeos-ops](https://github.com/ishan-parihar/lifeos-ops)
- Ontology: [github.com/ishan-parihar/HoloOS](https://github.com/ishan-parihar/HoloOS)
- Author: [Ishan Parihar](https://github.com/ishan-parihar)
