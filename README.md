# LifeOS

> A holonic operating system for personal and organizational life management, built on the [HoloOS](https://github.com/ishan-parihar/HoloOS) ontological architecture and implemented as a unified Rust CLI + MCP server on top of Notion.

[![Release](https://img.shields.io/github/v/release/ishan-parihar/lifeos-ops)](https://github.com/ishan-parihar/lifeos-ops/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

---

## What is LifeOS?

LifeOS operationalizes the HoloOS holonic architecture into **5 Notion databases** organized as 4 reservoirs + 1 contact-boundary. Every entry — a task, a note, a person, a goal — finds its ontological place in the dual-metabolic cycle (Catalyst → Experience → Transformation → Choice).

```
                    ┌─────────────────────────────────────────┐
                    │           GREATER CYCLE (ascent)         │
                    │                                         │
                    │   Significator ◄──────► Great Way       │
                    │   (identity)    T/Ch    (environment)   │
                    │       ▲                   ▲              │
                    │       │                   │              │
                    │   ┌───┴───────────────────┴───┐          │
                    │   │    NEXUS (contact boundary)│          │
                    │   │    Catalyst/Experience/    │          │
                    │   │    Transformation/Choice   │          │
                    │   └───┬───────────────────┬───┘          │
                    │       │                   │              │
                    │       ▼                   ▼              │
                    │   Matrix ◄──────► Potentiator            │
                    │   (state)    C/E     (possibility)       │
                    │                                         │
                    │           LESSER CYCLE (engine)          │
                    └─────────────────────────────────────────┘
```

## The 5 Databases

| DB | Role | Cycle | Currency In → Out |
|----|------|-------|-------------------|
| **Matrix** | Current-state organizer | Lesser | Catalyst → Experience |
| **Potentiator** | Latent-state generator | Lesser | Experience → Catalyst |
| **Nexus** | Contact boundary (transmutation) | Both | All 4 currencies |
| **Significator** | Persistent identity-pattern | Greater | Transformation → Choice |
| **GreatWay** | Operating environment | Greater | Choice → Transformation |

## Quick Start

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/ishan-parihar/lifeos-ops/main/install.sh | bash

# Set your Notion API token
export NOTION_API_TOKEN=ntn_xxx

# Discover your 5 LifeOS databases
lifeos discover

# View the dashboard
lifeos dashboard

# Daily review
lifeos daily
```

## CLI Commands

### Core
| Command | Purpose |
|---------|---------|
| `lifeos discover` | Scan Notion, resolve DB IDs + schema |
| `lifeos schema` | Show database schemas |
| `lifeos query` | Query any DB with filters |
| `lifeos mutate` | Create/update/delete entries |
| `lifeos get-page` | Fetch entry with relations resolved |

### Relational Intelligence
| Command | Purpose |
|---------|---------|
| `lifeos relational-gaps` | Surface orphaned entries + missing relations |
| `lifeos build-context` | Assemble complete neighborhood for an entry |
| `lifeos relational-graph` | High-level inter-DB relation tree |
| `lifeos holonic-synthesis` | Trace currency flow, identify bottlenecks |
| `lifeos quick-link` | Link entries by title (auto-resolves IDs) |

### Relational Write (deliberate, no auto-population)
| Command | Purpose |
|---------|---------|
| `lifeos link` | Create a single relation |
| `lifeos unlink` | Remove a single relation |
| `lifeos batch-link` | Create multiple relations (explicit specification) |

### Workflow
| Command | Purpose |
|---------|---------|
| `lifeos daily` | Daily review: gaps + synthesis + recent entries |
| `lifeos dashboard` | Overview: orphan counts, top gaps, health metrics |
| `lifeos review` | Daily/weekly/monthly/quarterly review pipeline |

### Ontology
| Command | Purpose |
|---------|---------|
| `lifeos archetype-index` | List all 22 HoloOS archetypes |
| `lifeos derive-type` | Derive Holon Type from Valence Signature |
| `lifeos valence-signature` | Generate Valence Signature YAML template |
| `lifeos validate-yaml` | Validate entries against YAML schemas |

### Sync
| Command | Purpose |
|---------|---------|
| `lifeos pull` | Pull pages from Notion to vault |
| `lifeos push` | Push vault changes to Notion |
| `lifeos watch` | Watch vault + sync in real-time |

## MCP Server

LifeOS runs as an MCP server for AI agent integration (Claude Desktop, Cursor, etc.):

```bash
lifeos mcp
```

**29 MCP tools** available — see [ONTOLOGY.md](ONTOLOGY.md) for the full architecture map.

## Architecture

```
lifeos-ops/
├── lifeos-core/              # Shared library
│   ├── src/
│   │   ├── notion/           # Notion API client (v2025-09-03)
│   │   ├── config.rs         # LifeOSConfig, HolonicConfig
│   │   ├── util/
│   │   │   ├── schema_engine.rs   # Auto-discovering schema cache
│   │   │   └── yaml_schemas.rs    # 3-tier YAML schema validator
│   │   ├── tools/            # 29 MCP/CLI tools
│   │   ├── transform/        # Notion blocks ↔ markdown
│   │   └── sync/             # Bidirectional Notion ↔ vault sync
│   └── Cargo.toml
├── lifeos/                   # Binary crate (CLI + MCP runner)
├── schemas/                  # 3-tier YAML schema hierarchy
│   ├── universal/            # 6 properties, 3 validation rules
│   ├── per_db/               # Per-DB relations + entry-types
│   └── per_entry_type/       # 31 per-entry-type specializations
├── scripts/upgrade_v0.9.0/   # One-time migration scripts (executed)
├── ONTOLOGY.md               # Native ontological foundation
├── AUDIT_ponytail_ontology.md # Architecture audit
└── lifeos.config.default.json
```

## Design Principles

1. **5 DBs, not 8** — Currencies (Catalyst, Experience, Choice) flow through the Nexus, they don't have their own reservoirs
2. **Every relation is deliberate** — Tools surface gaps and suggest connections, but the user approves each
3. **Schema tracks what IS used** — If a property has 0% fill rate after 30 days, it gets deleted
4. **Type ⊥ Stage** — Holon Type and Digestion Stage are independent properties
5. **G_z × P_z = Total Health** — Both integrative coherence AND transcendental tension required

## Documentation

- [ONTOLOGY.md](ONTOLOGY.md) — Native ontological foundation (distilled from HoloOS)
- [AUDIT_ponytail_ontology.md](AUDIT_ponytail_ontology.md) — Architecture audit + integration gaps
- [schemas/README.md](schemas/README.md) — YAML schema hierarchy reference
- [AGENTS.md](AGENTS.md) — Guide for AI agents working on this project

## Build from Source

```bash
git clone https://github.com/ishan-parihar/lifeos-ops.git
cd lifeos-ops
cargo build --release
# Binary at target/release/lifeos (or target/x86_64-unknown-linux-gnu/release/lifeos)
```

## Notion API Version

LifeOS targets **Notion API version `2025-09-03`**, which uses the **data source** abstraction:
- Properties live on the **data source** (not the database container)
- `PATCH /v1/data_sources/{id}` is used for property mutations
- Relation configs use `data_source_id` (not `database_id`)

## License

MIT — Developed by [Ishan Parihar](https://github.com/ishan-parihar)
