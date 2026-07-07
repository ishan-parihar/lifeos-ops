# AGENTS.md — LifeOS-Ops Development Protocol

> **Read this FIRST.** This document is the operating manual for any AI agent
> working in the `lifeos-ops` repository. It defines what the project IS, how
> to navigate it, what the rules are, and the EXACT workflow every iteration
> must follow. Skipping sections of this document is the #1 cause of
> regressions.

---

## 0. The Two Non-Negotiables

These two rules apply to **every iteration, no exceptions**:

1. **ALWAYS PUSH COMMITS ON THE `main` BRANCH.** No feature branches, no
   long-lived PRs. Commit logically-grouped changes to `main` and push
   immediately. The user's standing instruction.

2. **ALWAYS RUN `workspace-lint` AFTER EACH ITERATION.** Before declaring an
   iteration done, run:
   ```bash
   python3 skill/workspace-lint/scripts/workspace_lint.py
   ```
   If exit code ≠ 0, fix violations before pushing. If new violations are
   unavoidable (e.g. a new canonical dir is added but not yet populated),
   document the exception in the commit message.

These two rules exist because:
- Push-to-main keeps the worklog + repo state in sync — no orphan local commits.
- workspace-lint catches directory drift early. Once drift compounds, cleanup
  becomes a multi-hour task instead of a 30-second fix.

---

## 1. Project Overview

**LifeOS** is a **consciousness-prosthetic** built as a Rust CLI + MCP server
on top of Notion. It shapes the causal chain of the user's life toward an
ideal-future by running one causal amplification cycle through 5 databases
across 3 functional layers.

The architecture is **v4.1** — the merged 5-DB structure where the
teleological pull IS the parent/child hierarchy within Trajectory.

### The 5 DBs (v4.1 canon)

| # | DB | Layer | Purpose | Entry-Type discriminator |
|---|-----|-------|---------|--------------------------|
| 1 | **Trajectory** | A (Pull) | Teleological hierarchy | `Type` (12 entry-types across 3 internal layers) |
| 2 | **Logbook** | B (Record) | Ground-reality capture (6 channels) | `Entry Type` |
| 3 | **Synthesis** | B (Record) | Logs → insights (polar ±) | `Type` + `Polarity` |
| 4 | **Profile** | B (Record) | Cumulative state mirror | `Type` |
| 5 | **Context** | C (Action) | Environment CRM | `Type` |

The 3 functional layers + 3 flows + 1 cycle define the entire architecture:

```
Trajectory → Logbook → Synthesis → Profile → Trajectory
  (pull)      (capture)  (process)   (condense) (feedback)
```

Read [ONTOLOGY.md](ONTOLOGY.md) for the distilled architectural foundation.
Read [architecture/legacy_mapping/FORMAL_SPEC_v4.1.md](architecture/legacy_mapping/FORMAL_SPEC_v4.1.md)
for the machine-readable DB schema spec.

---

## 2. Architecture Quick Reference

### 2.1 Codebase Structure

```
lifeos-ops/
├── lifeos-core/                  # Shared library crate
│   └── src/
│       ├── notion/               # Notion API client (v2025-09-03) + types
│       ├── config.rs             # LifeOSConfig, DbConfig, HolonicConfig
│       ├── util/
│       │   ├── schema_engine.rs  # Auto-discovering schema cache
│       │   ├── yaml_schemas.rs   # 3-tier YAML schema validator
│       │   ├── id_resolver.rs    # Fuzzy title → page ID resolution
│       │   └── date_filter.rs    # Date-range filter construction
│       ├── tools/                # 31 MCP/CLI tools (mod.rs = registry)
│       ├── transform/            # Notion blocks ↔ markdown
│       ├── sync/                 # Bidirectional Notion ↔ vault sync
│       ├── vault/                # Local markdown vault index
│       └── server.rs             # MCP JSON-RPC server (async, v4.1)
├── lifeos/                       # Binary crate
│   └── src/{main.rs, mcp.rs}     # CLI dispatch + MCP runner
├── schemas/                      # 3-tier YAML schema hierarchy (v4.1 prefixes)
│   ├── universal/                # 6 properties, 3 validation rules
│   ├── per_db/                   # Per-DB relations + entry-types
│   └── per_entry_type/           # Per-entry-type specializations
├── scripts/                      # Python migration/audit scripts (one-time)
├── skill/                        # Bundled AI-agent skills
│   ├── notion-cli/SKILL.md       # Notion CLI (ntn) usage reference
│   └── workspace-lint/           # Directory structure validator
│       ├── SKILL.md
│       ├── scripts/workspace_lint.py
│       └── references/
├── workspace-lint.yaml           # This project's lint config
├── lifeos.config.default.json    # Embedded default config (compiled in)
├── .env                          # NOTION_API_TOKEN (gitignored)
├── ONTOLOGY.md, README.md, AGENTS.md
└── Cargo.toml                    # Workspace manifest
```

### 2.2 The 5 DBs (v4.1)

| DB | Layer | Entry-Type property | Layer labels (Trajectory only) |
|----|-------|---------------------|--------------------------------|
| **Trajectory** | A (Pull) | `Type` (select, 12 options) | Reference / Strategic / Execution |
| **Logbook** | B (Record) | `Entry Type` (select, 6 options) | — |
| **Synthesis** | B (Record) | `Type` (select, 5 options) + `Polarity` | — |
| **Profile** | B (Record) | `Type` (select, 4 options) | — |
| **Context** | C (Action) | `Type` (select, 5 options) | — |

### 2.3 The 3 Flows + Cycle

| Flow | Path | Direction | Tool to check |
|------|------|-----------|---------------|
| **Pull** | Vision → Annual-Goal → QG → Project → Task (within Trajectory) | Downward | `cycle_health` |
| **Ground** | Trajectory → Logbook → Synthesis → Profile | Forward | `cycle_health` |
| **Feedback** | Profile + Synthesis → Trajectory | Loop back | `cycle_health` |

### 2.4 Notion API Version: 2025-09-03

**CRITICAL:** In this API version:
- The Search API filter value is `data_source` (NOT `database`)
- Properties live on the **data_source**, not the database container
- READ properties: `GET /v1/data_sources/{id}`
- MODIFY properties: `PATCH /v1/data_sources/{id}`
- Relation configs use `data_source_id` (NOT `database_id`)
- Status options CANNOT be renamed via API — only via Notion UI
- Saved views: `POST /v1/views` (Notion-Version 2026-03-11)

---

## 3. The Iteration Workflow (MANDATORY)

Every iteration — feature, fix, refactor, doc update — MUST follow these 6
steps in order. Skipping steps is the #1 cause of regressions.

### Step 1: Orient

```bash
cd /home/z/my-project/repos/lifeos-ops
git status                              # What's uncommitted?
git log --oneline -5                    # What was the last work?
cat /home/z/my-project/worklog.md       # What did previous agents do?
```

Read the relevant section of this AGENTS.md if you're touching unfamiliar code.

### Step 2: Implement

Write the code. Follow these rules:

- **Rust code:** One tool per file under `lifeos-core/src/tools/`. Register
  the tool in `lifeos-core/src/tools/mod.rs` (both `get_tool_definitions`
  AND `call_tool` dispatch). Add a CLI command in `lifeos-core/src/cli/mod.rs`
  + wire it in `lifeos/src/main.rs`.
- **Python scripts:** Go in `scripts/`. Mark migration scripts as one-time in the docstring.
- **YAML schemas:** Go in `schemas/` with v4.1 prefixes
  (`trajectory__*.yaml`, `logbook__*.yaml`, etc.). Run
  `lifeos validate-yaml --self-test` after adding/editing schemas.
- **Skill files:** Go ONLY under `skill/<skill-name>/`. Never loose at `skill/` root.
- **Config changes:** Update BOTH `lifeos.config.default.json` (committed)
  AND the local `lifeos.config.json`. The default config is `include_str!`'d
  at compile time — rebuild after changing it.

### Step 3: Build + Test

```bash
. "$HOME/.cargo/env"
cargo build --target x86_64-unknown-linux-gnu         # Must compile clean
./target/x86_64-unknown-linux-gnu/debug/lifeos validate-yaml --self-test
```

If adding MCP behavior, also run:
```bash
python3 /home/z/my-project/scripts/mcp_smoke_v4.1.py   # 14-test regression suite
```

If touching Notion data, run `lifeos discover` to confirm DBs resolve.

### Step 4: Run workspace-lint

```bash
python3 skill/workspace-lint/scripts/workspace_lint.py
```

- Exit 0 → OK to commit.
- Exit 1 → fix violations (warnings are OK; errors are not).
- Exit 2 → config missing or invalid.

### Step 5: Update the Worklog

Append to `/home/z/my-project/worklog.md` using the template in §5 below.
Every iteration gets its own `---` section with Task ID, Agent, Task, Work
Log, Stage Summary, and (if applicable) Pending Items.

### Step 6: Commit + Push to `main`

```bash
git add -A
git status                                  # Verify what's staged
git commit -m "<type>: <subject>"           # See commit conventions §4
git push origin main
```

**No feature branches. No PRs. Commit to `main` and push.**

If the push is rejected (remote has new commits), `git pull --rebase origin
main` first, resolve conflicts, then push again.

---

## 4. Commit Conventions

Use [Conventional Commits](https://www.conventionalcommits.org/) prefixes:

| Type | Use for |
|------|---------|
| `feat:` | New feature (new tool, new command, new schema) |
| `fix:` | Bug fix (no behavior change beyond the fix) |
| `refactor:` | Code restructure (no behavior change) |
| `perf:` | Performance improvement |
| `docs:` | Documentation only |
| `test:` | Test additions or fixes |
| `chore:` | Build, CI, config, lint — no production code |
| `audit:` | Audit/analysis document |
| `plan:` | Planning document |

Examples:
```
feat: add morning tool — aggregated view across 5 DBs
fix: repoint 8 ghost-database relations from 0baacff9 to a1769af1
refactor: YAGNI cleanup — remove 4 redundant v4.1 utility tools (35→31)
docs: rewrite ONTOLOGY.md + AGENTS.md for v4.1
```

---

## 5. Worklog Protocol

The worklog lives at `/home/z/my-project/worklog.md` (shared across agents).
**Read it before starting work.** Append your record after finishing.

Template:
```markdown
---
Task ID: <next sequential number, e.g. 18>
Agent: <agent name, e.g. LifeOS-Architect>
Task: <one-sentence description of what you were asked to do>

Work Log:
- <concrete step 1>
- <concrete step 2>
- ...

Stage Summary:
- <key results>
- <decisions made>
- <artifacts produced>

Pending Items (for next sprint):
1. <item>
2. <item>
```

---

## 6. Design Principles (NON-NEGOTIABLE)

### 6.1 Every relation is a deliberate choice
- Tools **surface** gaps and **suggest** connections.
- The user (or AI agent acting explicitly) must approve each link.
- **NO auto-population** of relations, entry-types, or properties — EXCEPT
  via the `auto_enrich` tool, which is suggestion-only (v0.10.2 demoted).

### 6.2 Schema tracks what IS used, not what COULD be
- If a property has 0% fill rate after 30 days, delete it.
- Don't add speculative properties "for the future."
- The YAML schema hierarchy (universal → per_db → per_entry_type) validates,
  not stores.

### 6.3 The 5 DBs are the ONLY DBs
- The v4.1 canon DBs: **Trajectory / Logbook / Synthesis / Profile / Context**.
- Config keys map 1:1 to these names (`trajectory`, `logbook`, `synthesis`,
  `profile`, `context`).
- Old holonic keys (matrix/potentiator/nexus/significator/greatway) and the
  v0.10.1 renamed keys (state/possibility/process/identity/world) are
  DEPRECATED. The codebase uses v4.1 keys throughout.

### 6.4 The pull IS the hierarchy
- The teleological pull is NOT a flow between DBs — it's the parent/child
  tree within Trajectory (via the `Parent` self-relation).
- Use `ancestors` to walk this hierarchy. Returns `entry_type` and `layer`
  (Reference/Strategic/Execution) for each chain node.

### 6.5 The cycle is the unit of health
- `cycle_health` is the canonical "is the system alive?" tool.
- It checks 3 flows: Pull (Trajectory hierarchy links), Ground (Logbook →
  Synthesis links), Feedback (Profile → Trajectory links).
- Dormant flow = stuck cycle. All 3 flows must have active links.

### 6.6 YAGNI — cut deadweight aggressively
- Don't add a property "for future use." Add it when the user needs it.
- Don't keep two tools that do the same thing. Consolidate.
- The v4.1 YAGNI cleanup (35 → 31 tools) is the model: 4 redundant utility
  tools killed because AI agents compose them from primitives.

### 6.7 MCP/CLI tools are for AI agents, not the user
- The user operates LifeOS via Notion UI directly.
- Tools exist to give AI agents operational access to the 5 DBs.
- A dedicated tool is justified ONLY when:
  1. It eliminates multiple round-trips an agent would otherwise make.
  2. It encodes architectural semantics the agent would miss.
  3. It writes state in a specific way the agent shouldn't second-guess.
- If an agent can compose the behavior from `query` + `mutate` + `link` +
  `ancestors` in ≤3 calls, you don't need a dedicated tool.

### 6.8 Lazy senior dev mode
Before writing any code:
1. **Does it need to exist at all?** (YAGNI)
2. **Does the stdlib do it?** (Don't reinvent.)
3. **Is there a native platform feature?** (Don't reinvent Notion's saved views, formulas, etc.)
4. **Can it be one line?** (Don't wrap if you don't have to.)
5. **No unrequested abstractions, no avoidable dependencies, no boilerplate.**
6. Mark intentional simplifications with `// ponytail:` comments.

---

## 7. The 31 Tools (v4.1)

### Schema/Query (3)
- `get_schema` — DB schemas with entry types. First call any agent makes.
- `query` — Query any DB with filters, sort, entry_type, cycle, AI filter override.
- `mutate` — Create/update/delete entries across all 5 DBs.

### Intelligence (5)
- `intelligence_briefing` — Role or cycle briefing. (Still references old holonic modes — pending v4.1 refactor.)
- `data_science` — Temporal patterns, trajectories, correlations.
- `review_pipeline` — Daily/weekly/monthly/quarterly reviews.
- `strategic_simulator` — Cross-DB strategic analysis.
- `sync_note` — Bidirectional Notion ↔ markdown sync.

### Relational Navigation (6)
- `get_page` — Fetch entry with all relations resolved to titles.
- `build_context` — Complete relational neighborhood for an entry. One call replaces 3+.
- `trace` — Follow relations N levels deep.
- `ancestors` — Walk up hierarchy. Returns entry_type + layer labels for Trajectory.
- `backlinks` — Find all entries that reference a given page.
- `relational_graph` — High-level relational graph overview with link counts.

### Relational Write (3)
- `link` — Create a relation between two entries.
- `unlink` — Remove a single relation.
- `batch_link` — Create multiple relations in one call.

### Audit & Validation (5)
- `orphans` — List entries with zero populated relations.
- `relational_gaps` — Surface entries with sparse relations + missing expected relations.
- `validate_yaml` — Validate entries against v4.1 YAML schema hierarchy.
- `suggest_links` — Suggest likely cross-DB links for orphan entries via title similarity.
- `suggest_categorization` — Suggest entry-types for uncategorized entries.

### Workflow (2)
- `daily` — Run daily review: relational gaps + recent entries in one call.
- `dashboard` — Orphan count per DB, recent entries, top gaps, health metrics.

### Auto-Enrichment & Audit (3)
- `auto_enrich` — READ-ONLY advisor. Suggests universal properties. Never writes.
- `fill_rate` — Audit property fill rates per DB. Flags <5% as YAGNI candidates.
- `quick_link` — Link two entries by title (auto-resolves page IDs via fuzzy match).

### v4.1 Utility Layer (2)
- `morning` — Aggregated morning view: active goals, today tasks, recent logs, recent synthesis. Primary AI-agent orient call. One call replaces 4+ queries.
- `cycle_health` — Check if the v4.1 causal amplification cycle is running. Reports pull/ground/feedback flow health + recommendations.

---

## 8. Common Tasks

### "I need to query entries in a DB"
```bash
lifeos query trajectory --limit 20
lifeos query logbook --entry-type "Activity" --limit 50
lifeos query trajectory --filter-property "Status" --filter-value "Active"
```
Or via MCP: `query` tool with `database`, `entry_type`, `limit` args.

### "I need to create a new entry"
```bash
lifeos mutate --operation create --database logbook --properties '{"Name":"...","Entry Type":"Activity","Date":{"start":"2026-07-08"}}'
```
Or via MCP: `mutate` tool.

### "I need to link two entries"
```bash
# By ID:
lifeos link --source <page_id> --target <page_id> --property "Source Project"

# By title (auto-resolves IDs):
lifeos quick-link --source-db trajectory --source-title "..." --target-db logbook --target-title "..." --property "Source Project"
```

### "I need to orient (morning view)"
```bash
lifeos morning
```
Or via MCP: `morning` tool. Returns active_goals + todays_tasks + recent_logs + recent_synthesis in one call.

### "I need to check if the cycle is alive"
```bash
lifeos cycle-health
```
Returns pull/ground/feedback flow health + recommendations.

### "I need to trace an entry up the hierarchy"
```bash
lifeos ancestors <page_id> --max-levels 5
```
Returns chain with entry_type + layer (Reference/Strategic/Execution) labels.

### "I need to see what's orphaned"
```bash
lifeos relational-gaps
lifeos orphans --database trajectory
lifeos dashboard
```

### "I need to understand an entry's context"
```bash
lifeos build-context --page-id <id> --depth 2
```

### "I need to validate entries against schemas"
```bash
lifeos validate-yaml --self-test     # Schema self-test (no Notion API)
lifeos validate-yaml --all           # Validate all entries (requires token)
lifeos validate-yaml --database trajectory
```

### "I need to use the Notion CLI directly (ntn)"
Read `skill/notion-cli/SKILL.md` for the full `ntn` CLI reference. The CLI
auto-uses `NOTION_API_TOKEN` from `.env`. Useful for one-off operations
that don't warrant a new `lifeos` tool:
```bash
ntn api v1/data_sources/<id>                          # Get a DB schema
ntn api v1/data_sources/<id>/query -d '{"page_size":5}'  # Query entries
ntn pages get <page-id>                               # Get a page as markdown
```

### "I need to add a new MCP tool"
1. **YAGNI check first** — can an AI agent compose this from `query` + `mutate` + `link` + `ancestors` in ≤3 calls? If yes, don't add the tool.
2. Create `lifeos-core/src/tools/my_tool.rs` with:
   - `pub struct MyToolParams { ... }` (derive `Deserialize`)
   - `pub fn schema() -> serde_json::Value { ... }`
   - `pub async fn execute(params, config, notion, schema_cache) -> Result<String, String>`
3. Register in `lifeos-core/src/tools/mod.rs`:
   - Add `pub mod my_tool;`
   - Add `tool_def("my_tool", "description", my_tool::schema())` to `get_tool_definitions`
   - Add dispatch arm to `call_tool`
4. Add CLI command in `lifeos-core/src/cli/mod.rs` + wire in `lifeos/src/main.rs`
5. Build + test: `cargo build --target x86_64-unknown-linux-gnu && ./target/.../lifeos my-tool --help`
6. Run `python3 /home/z/my-project/scripts/mcp_smoke_v4.1.py` to verify tools/list includes it
7. Run `python3 skill/workspace-lint/scripts/workspace_lint.py`
8. Update §7 of this AGENTS.md to include the new tool in the appropriate category.

### "I need to debug a Notion API issue"
- Check `lifeos-core/src/notion/client.rs` for the request shape.
- Use `ntn api <path> --docs` to read the official endpoint docs.
- Use `ntn api <path> --spec` to see the OpenAPI fragment.
- The Notion API 2025-09-03 has quirks — see §2.4 above.

---

## 9. Testing

### 9.1 Build verification
```bash
. "$HOME/.cargo/env"
cargo build --target x86_64-unknown-linux-gnu
```
Must complete with 0 errors. Warnings should be fixed in the same commit
that introduced them.

### 9.2 Schema self-test
```bash
./target/x86_64-unknown-linux-gnu/debug/lifeos validate-yaml --self-test
```
Loads universal + per_db + per_entry_type schemas. Must report
"✅ All schemas passed self-test."

### 9.3 Discover smoke test
```bash
./target/x86_64-unknown-linux-gnu/debug/lifeos discover
```
Must resolve all 5 DBs (Trajectory, Logbook, Synthesis, Profile, Context)
and discover relation edges.

### 9.4 MCP smoke test
```bash
python3 /home/z/my-project/scripts/mcp_smoke_v4.1.py
```
14-test regression suite. Must report "14 passed, 0 failed." Covers:
- initialize
- ping
- tools/list (≥25 tools, no phantom tools, morning + cycle_health visible)
- tools/list YAGNI checks (no capture, no trace_trajectory, no gap_analysis, no surface_synthesis)
- tools/call: get_schema
- tools/call: query trajectory
- batch JSON-RPC

### 9.5 workspace-lint
```bash
python3 skill/workspace-lint/scripts/workspace_lint.py
```
Must exit 0. Warnings are OK; errors are not.

---

## 10. Known Issues + Pending Items

Top pending items for future sprints:

1. **intelligence.rs dead modes** — `nexus_interpretation` and other
   dead-mode stubs still present. Evolve `intelligence_briefing` for v4.1
   3-layer architecture (Reference/Strategic/Execution briefings instead
   of CEO-CFO/etc. role briefings).
2. **data_science.rs old holonic params** — Still references old holonic
   cycle params (lesser/greater). Refactor for v4.1 3-flow architecture.
3. **validate_yaml old validation rules** — Validation rules reference
   old holonic entry-types. Update for v4.1 entry-types.
4. **suggest_categorization v4.1 entry-types** — Heuristics still use
   old holonic terms. Update for v4.1.
5. **CLI help text drift** — Several CLI commands' help text references
   old DB names (matrix/potentiator/etc.). Audit and update.
6. **Profile diff / snapshot system** — Profile is meant to track
   state-over-time but currently has no history mechanism. YAGNI for now;
   add when user requests trend visualization.

See also:
- [architecture/legacy_mapping/FORMAL_SPEC_v4.1.md](architecture/legacy_mapping/FORMAL_SPEC_v4.1.md) — the v4.1 DB schema spec
- [architecture/legacy_mapping/PONYTAIL_AUDIT_v4.1.md](architecture/legacy_mapping/PONYTAIL_AUDIT_v4.1.md) — the 34% dead-weight audit

---

## 11. Contact

- Repo: [github.com/ishan-parihar/lifeos-ops](https://github.com/ishan-parihar/lifeos-ops)
- Ontology: [github.com/ishanparihar/HoloOS](https://github.com/ishanparihar/HoloOS)
- Author: [Ishan Parihar](https://github.com/ishanparihar)

---

## 12. Quick Reference

| I want to... | Use this |
|---|---|
| Build the binary | `cargo build --target x86_64-unknown-linux-gnu` |
| Run schema self-test | `lifeos validate-yaml --self-test` |
| Discover DBs | `lifeos discover` |
| See the morning view | `lifeos morning` |
| Check cycle health | `lifeos cycle-health` |
| See the dashboard | `lifeos dashboard` |
| Walk an entry up the hierarchy | `lifeos ancestors <page_id>` |
| Run MCP smoke tests | `python3 /home/z/my-project/scripts/mcp_smoke_v4.1.py` |
| Run workspace-lint | `python3 skill/workspace-lint/scripts/workspace_lint.py` |
| Use the Notion CLI directly | `ntn api ...` (see `skill/notion-cli/SKILL.md`) |
| Read the architecture | `ONTOLOGY.md` + `architecture/legacy_mapping/FORMAL_SPEC_v4.1.md` |
| Read the previous agent's work | `/home/z/my-project/worklog.md` |
| Commit + push | `git add -A && git commit -m "..." && git push origin main` |

---

*You are an AI agent working on LifeOS. The 5 DBs are the only DBs.
The cycle is the unit of health. MCP/CLI tools are for AI agents, not the
user. Push to main. Run workspace-lint. Don't break the cycle.*
