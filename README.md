# LifeOS

> A consciousness-prosthetic built as a Rust CLI + MCP server on top of Notion.
> Shapes the causal chain of your life toward an ideal-future by running one
> amplification cycle through 5 databases across 3 functional layers.

[![Release](https://img.shields.io/github/v/release/ishan-parihar/lifeos-ops)](https://github.com/ishan-parihar/lifeos-ops/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

---

## What is LifeOS?

Every entry — a task, a note, a person, a goal — finds its place in one of 5 Notion databases organized across 3 functional layers. The cycle is the unit of health:

```
Trajectory → Logbook → Synthesis → Profile → Trajectory
  (pull)      (capture)  (process)   (condense) (feedback)
```

**4 hops.** Tighter feedback = faster amplification.

## The 5 Databases

| DB | Layer | Purpose | Entry-Types |
|----|-------|---------|-------------|
| **Trajectory** | A — Pull | Teleological hierarchy — the pull IS the parent/child tree | Purpose, Value, Principle, Vision-Statement, Identity-Statement, Annual-Goal, Quarterly-Goal, Milestone, Project, Task, Campaign, Content |
| **Logbook** | B — Record | Ground-reality capture (6 channels) | Activity, Diet, Financial, Subjective, Relational, Systemic |
| **Synthesis** | B — Record | Logs → insights (polar ±) | Note, Opportunity, Strength, Directive, Risk |
| **Profile** | B — Record | Cumulative state mirror (RPG status) | Trait, Metric, Capacity, Asset |
| **Context** | C — Action | Environment CRM (who/what is around) | Person, Community, Organization, Financial-Account, Place |

## The 3 Functional Layers

```
┌─────────────────────────────────────────────────────┐
│  Layer A — Teleological Pull (Trajectory DB)        │
│                                                     │
│  Purpose → Value → Vision-Statement                 │
│    → Annual-Goal → Quarterly-Goal                   │
│    → Project → Task                                 │
└──────────────────────┬──────────────────────────────┘
                       │ Pull (downward)
                       ▼
┌─────────────────────────────────────────────────────┐
│  Layer B — Historical Record                        │
│                                                     │
│  Logbook ──── Ground ───▶ Synthesis ───▶ Profile    │
│  (capture)               (process)         (mirror) │
└──────────────────────┬──────────────────────────────┘
                       │ Feedback (loop)
                       ▼
┌─────────────────────────────────────────────────────┐
│  Layer C — Action Interface (Context DB)            │
│                                                     │
│  People / Communities / Orgs /                      │
│  Financial-Accounts / Places                        │
└─────────────────────────────────────────────────────┘
```

## The 3 Flows

| Flow | Path | Direction |
|------|------|-----------|
| **Pull** | Vision → Annual-Goal → QG → Project → Task (within Trajectory) | Downward |
| **Ground** | Trajectory → Logbook → Synthesis → Profile | Forward |
| **Feedback** | Profile + Synthesis → Trajectory | Loop back |

## Installation

### One-line install (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/ishan-parihar/lifeos-ops/main/install.sh | bash
```

### Manual install from releases

```bash
curl -fsSL -o /tmp/lifeos.tar.gz https://github.com/ishan-parihar/lifeos-ops/releases/latest/download/lifeos-x86_64-unknown-linux-gnu.tar.gz
tar xzf /tmp/lifeos.tar.gz -C /tmp
sudo install /tmp/lifeos-x86_64-unknown-linux-gnu /usr/local/bin/lifeos
lifeos --version
```

### Build from source

```bash
git clone https://github.com/ishan-parihar/lifeos-ops.git
cd lifeos-ops
cargo build --release
sudo install target/release/lifeos /usr/local/bin/
```

## Quick Start

```bash
export NOTION_API_TOKEN=ntn_xxx

lifeos discover          # Resolve your 5 DBs
lifeos dashboard         # Overview: orphans, gaps, health
lifeos daily             # Daily review
lifeos morning           # Active goals + today's tasks + recent logs
```

## CLI Commands (29 tools)

### Schema & Query
| Command | Purpose |
|---------|---------|
| `lifeos schema` | DB schemas with entry types |
| `lifeos query` | Query any DB with filters, sort, entry_type |
| `lifeos mutate` | Create/update/delete entries |

### Intelligence
| Command | Purpose |
|---------|---------|
| `lifeos intelligence` | Role or cycle briefing |
| `lifeos data-science` | Temporal patterns, trajectories, correlations |
| `lifeos review` | Daily/weekly/monthly/quarterly reviews |
| `lifeos strategic` | Cross-DB strategic analysis |
| `lifeos sync` | Bidirectional Notion ↔ markdown sync |

### Relational Navigation
| Command | Purpose |
|---------|---------|
| `lifeos get-page` | Fetch entry with relations resolved |
| `lifeos build-context` | Complete neighborhood for an entry |
| `lifeos trace` | Follow relations N levels deep |
| `lifeos ancestors` | Walk up hierarchy (returns layer labels) |
| `lifeos backlinks` | Find all entries referencing a page |
| `lifeos relational-graph` | High-level inter-DB relation tree |

### Relational Write
| Command | Purpose |
|---------|---------|
| `lifeos link` | Create a single relation |
| `lifeos unlink` | Remove a single relation |
| `lifeos batch-link` | Create multiple relations |

### Audit & Validation
| Command | Purpose |
|---------|---------|
| `lifeos orphans` | Entries with zero relations |
| `lifeos relational-gaps` | Sparse relations + missing expected links |
| `lifeos validate-yaml` | Validate entries against YAML schemas |
| `lifeos suggest-links` | Suggest cross-DB links for orphans |
| `lifeos suggest-categorization` | Suggest entry-types for uncategorized entries |

### Workflow
| Command | Purpose |
|---------|---------|
| `lifeos daily` | Daily review: gaps + synthesis + recent entries |
| `lifeos dashboard` | Orphan counts, top gaps, health metrics |

### Enrichment & Utilities
| Command | Purpose |
|---------|---------|
| `lifeos auto-enrich` | READ-ONLY advisor — suggests universal properties |
| `lifeos fill-rate` | Audit property fill rates per DB |
| `lifeos quick-link` | Link two entries by title (auto-resolves IDs) |
| `lifeos morning` | Aggregated view: goals + tasks + logs + synthesis |
| `lifeos cycle-health` | Check if the amplification cycle is running |

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
│   └── src/
│       ├── notion/           # Notion API client (v2025-09-03)
│       ├── config.rs         # LifeOSConfig, DbConfig
│       ├── util/
│       │   ├── schema_engine.rs   # Auto-discovering schema cache
│       │   ├── yaml_schemas.rs    # 3-tier YAML schema validator
│       │   ├── id_resolver.rs     # Fuzzy title → page ID
│       │   └── date_filter.rs     # Date-range filter construction
│       ├── tools/            # 29 MCP/CLI tools (mod.rs = registry)
│       ├── transform/        # Notion blocks ↔ markdown
│       ├── sync/             # Bidirectional Notion ↔ vault
│       ├── vault/            # Local markdown vault index
│       └── server.rs         # MCP JSON-RPC server
├── lifeos/                   # Binary crate (CLI + MCP runner)
├── schemas/                  # 3-tier YAML schema hierarchy
│   ├── universal/            # Properties + validation rules
│   ├── per_db/               # Per-DB relations + entry-types
│   └── per_entry_type/       # Per-entry-type specializations
├── skill/                    # Bundled AI-agent skills
│   ├── notion-cli/           # Notion CLI (ntn) reference
│   └── workspace-lint/       # Directory structure validator
├── ONTOLOGY.md               # Architectural foundation
├── AGENTS.md                 # AI agent development protocol
└── lifeos.config.default.json
```

## Design Principles

1. **5 DBs, not 22** — The v4.1 merger eliminated 6 cross-DB relations that are now intra-DB self-relations. The 5-DB structure is the minimum that separates the 3 functional layers.
2. **The pull IS the hierarchy** — The teleological pull is the parent/child tree within Trajectory, not a flow between DBs.
3. **Every relation is deliberate** — Tools surface gaps and suggest connections, but the user approves each link. No auto-population.
4. **The cycle is the unit of health** — `cycle_health` checks whether Pull, Ground, and Feedback flows all have active links.
5. **YAGNI aggressively** — Don't add a property "for future use." Don't keep two tools that do the same thing.
6. **Tools are for AI agents, not the user** — The user operates via Notion UI. Tools give AI agents operational access.

## Documentation

- [ONTOLOGY.md](ONTOLOGY.md) — Architectural foundation (the 3 layers, 3 flows, 1 cycle)
- [AGENTS.md](AGENTS.md) — AI agent development protocol
- [schemas/](schemas/) — YAML schema hierarchy reference

## Notion API Version

LifeOS targets **Notion API version `2025-09-03`**, which uses the **data source** abstraction:
- Properties live on the **data source** (not the database container)
- `PATCH /v1/data_sources/{id}` is used for property mutations
- Relation configs use `data_source_id` (not `database_id`)

## License

MIT — Developed by [Ishan Parihar](https://github.com/ishan-parihar)
