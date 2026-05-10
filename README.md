# LifeOS

Unified CLI + MCP server for Notion-based personal operating system.

## Features

### CLI Commands
```bash
lifeos init       # Initialize vault directory
lifeos pull        # Pull pages from Notion to vault
lifeos push        # Push vault changes to Notion
lifeos watch      # Watch vault and sync in real-time
lifeos page new   # Create a new page
lifeos page edit   # Edit a page with 3-way merge
lifeos page diff  # Show diff between vault and Notion
lifeos page merge # Resolve merge conflicts
lifeos mcp        # Run as MCP server (AI agent integration)
```

### MCP Tools (for AI agents)
- `query` — Unified database queries with filters, sorts, presets
- `mutate` — Create, update, delete, upsert entries
- `intelligence_briefing` — Role-based analysis (CEO, COO, CMO, etc.)
- `data_science` — Temporal patterns, trajectories, correlations
- `review_pipeline` — Daily, weekly, monthly, quarterly reviews
- `strategic_simulator` — Cross-database OKR and project analysis
- `sync_note` — Bidirectional Notion ↔ markdown sync

## Quick Start

1. **Install** — Download the latest release binary for your platform
2. **Configure** — Create `lifeos.config.json`:
```json
{
  "api_version": "2025-09-03",
  "rate_limit": { "requests_per_second": 3.0, "cache_ttl_seconds": 300 },
  "databases": {
    "tasks": {
      "name": "Tasks",
      "data_source_id": "your-notion-database-id",
      "agent": "default",
      "properties": { "status": "Status", "date": "Date" }
    }
  }
}
```
3. **Set token** — `export NOTION_API_TOKEN=your_token`
4. **Init vault** — `lifeos init`
5. **Pull data** — `lifeos pull`

## Build from Source

```bash
# Requires Rust 1.70+
cargo build --release

# Cross-compile for musl (static binary)
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

## Architecture

```
lifeos-ops/
├── lifeos-core/     # Shared library (notion client, sync, transform, tools, MCP)
├── lifeos/          # Binary crate (CLI + MCP runner)
├── Cargo.toml       # Workspace root
└── lifeos.config.json  # Your configuration (not in repo)
```

### MCP Server Integration
Connect to AI agents via stdio JSON-RPC:
```bash
lifeos mcp
```
Compatible with Claude Desktop, Cursor, and any MCP-compatible AI client.

## License

MIT