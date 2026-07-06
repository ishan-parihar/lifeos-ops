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

**LifeOS** is a holonic operating system built as a Rust CLI + MCP server on
top of Notion. It operationalizes the [HoloOS](https://github.com/ishanparihar/HoloOS)
ontological architecture into **5 Notion databases** (renamed in v0.10.1):

| Old name | New name | Holonic role | Currency in → out |
|----------|----------|--------------|-------------------|
| Matrix | **State** | Current-state organizer | Catalyst → Experience |
| Potentiator | **Possibility** | Latent-state generator | Experience → Catalyst |
| Nexus | **Process** | Contact-boundary (4 currencies) | All |
| Significator | **Identity** | Persistent identity-pattern | Transformation → Choice |
| GreatWay | **World** | Operating environment | Choice → Transformation |

The two metabolic cycles:
- **Lesser cycle** (State ⇌ Possibility): regulated by Eros↔Agape, health = G_z
- **Greater cycle** (Identity ⇌ World): regulated by Agency↔Communion, health = P_z
- **Process** (was Nexus) is the shared contact-boundary between them; the
  `Kind` select tags which currency a Process entry currently carries
  (Catalyst / Experience / Transformation / Choice).

Read [ONTOLOGY.md](ONTOLOGY.md) for the distilled ontological foundation.
Read [README.md](README.md) for the user-facing overview.

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
│       └── server.rs             # MCP JSON-RPC server (async, v0.10.1)
├── lifeos/                       # Binary crate
│   └── src/{main.rs, mcp.rs}     # CLI dispatch + MCP runner
├── schemas/                      # 3-tier YAML schema hierarchy
│   ├── universal/                # 6 properties, 3 validation rules
│   ├── per_db/                   # Per-DB relations + entry-types
│   └── per_entry_type/           # 31 per-entry-type specializations
├── scripts/                      # Python migration/audit scripts
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

### 2.2 The 5 DBs (v0.10.1 renamed)

| DB | Holonic role | Entry-Type property | Currency property |
|----|------|---------------------|-------------------|
| **State** | Current-state organizer | `Entry Type` (multi_select) | — |
| **Possibility** | Latent-state generator | `Entry Type` (select) | — |
| **Process** | Contact-boundary | `Category` (select) | `Kind` (select: Catalyst/Experience/Transformation/Choice) |
| **Identity** | Persistent identity | `Entry Type` (multi_select) | — |
| **World** | Operating environment | `Item Type` (select) | — |

### 2.3 Universal Properties (6, on all 5 DBs)

1. `Archetype Role` (select, 8 options) — Matrix/Potentiator/Catalyst/Experience/Significator/Transformation/Great Way/Choice
2. `Complex` (select, 4 options) — Mind/Body/Spirit/None
3. `Drive Activation` (multi_select, 4 options) — Agency/Communion/Eros/Agape
4. `Shadow Pattern` (select, 6 options) — None/Dark-Addiction/Dark-Allergy/Golden-Addiction/Golden-Allergy/Sinkhole of Indifference
5. `Digestion Stage` (select, 9 options) — on Process + Possibility only
6. `Holon Type` (select, 5 options) — on Identity only

### 2.4 Notion API Version: 2025-09-03

**CRITICAL:** In this API version:
- The Search API filter value is `data_source` (NOT `database`)
- Properties live on the **data_source**, not the database container
- READ properties: `GET /v1/data_sources/{id}`
- MODIFY properties: `PATCH /v1/data_sources/{id}`
- Relation configs use `data_source_id` (NOT `database_id`)
- Status options CANNOT be renamed via API — only via Notion UI

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
- **Python scripts:** Go in `scripts/`. Reuse `scripts/upgrade_v0.9.0/common.py`
  for Notion API access. Mark migration scripts as one-time in the docstring.
- **YAML schemas:** Go in `schemas/` (universal / per_db / per_entry_type).
  Run `lifeos validate-yaml --self-test` after adding/editing schemas.
- **Skill files:** Go ONLY under `skill/<skill-name>/`. Never loose at `skill/` root.
- **Config changes:** Update BOTH `lifeos.config.default.json` (committed)
  AND the local `lifeos.config.json` (gitignored at runtime but tracked in
  git history). The default config is `include_str!`'d at compile time —
  rebuild after changing it.

### Step 3: Build + Test

```bash
. "$HOME/.cargo/env"
cargo build --target x86_64-unknown-linux-gnu         # Must compile clean
./target/x86_64-unknown-linux-gnu/debug/lifeos validate-yaml --self-test
```

If adding MCP behavior, also run:
```bash
python3 /home/z/my-project/scripts/mcp_smoke_test.py   # 14-test regression suite
```

If touching Notion data, run `lifeos discover` to confirm DBs resolve.

### Step 4: Run workspace-lint

```bash
python3 skill/workspace-lint/scripts/workspace_lint.py
```

- Exit 0 → OK to commit.
- Exit 1 → fix violations (warnings are OK; errors are not).
- Exit 2 → config missing or invalid.

If you added a new file type or directory, update `workspace-lint.yaml` in
the same commit.

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
feat: add auto_enrich tool for entry-type→property inference
fix: repoint 8 ghost-database relations from 0baacff9 to a1769af1
refactor: rewrite server.rs with async tokio::io (B1 fix)
docs: update AGENTS.md with v0.10.1 development protocol
chore: add workspace-lint config + skill bundle
```

---

## 5. Worklog Protocol

The worklog lives at `/home/z/my-project/worklog.md` (shared across agents).
**Read it before starting work.** Append your record after finishing.

Template:
```markdown
---
Task ID: <next sequential number, e.g. 4>
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
  via the `auto_enrich` tool, which is opt-in (`apply=true`) and never
  overwrites existing values.

### 6.2 Schema tracks what IS used, not what COULD be
- If a property has 0% fill rate after 30 days, delete it.
- Don't add speculative properties "for the future."
- The YAML schema hierarchy (universal → per_db → per_entry_type) validates,
  not stores.

### 6.3 The 5 DBs are the ONLY DBs
- No auxiliary DBs. People/Group/Community belong in World. Notes/Knowledge
  belong in Process.
- If you need a new entry-type, add it to an existing DB's select options.
- Currencies (Catalyst, Experience, Choice) live as `Kind` options in Process,
  not separate DBs.

### 6.4 Type ⊥ Stage
- `Holon Type` (Donor/Acceptor/Sharer/Multivalent/Noble) is STABLE.
- `Digestion Stage` (1-9) is DYNAMIC.
- They are independent — never collapse them.

### 6.5 YAGNI — cut deadweight aggressively
- Don't add a property "for future use." Add it when the user needs it.
- Don't keep two tools that do the same thing. Consolidate.
- If a script has been executed and is no longer needed, archive it.
- The v0.10.0→v0.10.1 consolidation (34→28→31 tools, removed double-wrap
  bugs, removed phantom tool references) is the model.

### 6.6 Renamed DBs are the new canon
- The DBs in Notion are now: State, Possibility, Process, Identity, World.
- The config keys (`matrix`, `potentiator`, `nexus`, `significator`,
  `greatway`) are UNCHANGED for backward compatibility — they're internal
  identifiers, not user-facing labels.
- When writing user-facing text (descriptions, instructions, error messages),
  use the new names. When writing code that indexes into `config.databases`,
  use the old keys.

---

## 7. Common Tasks

### "I need to query entries in a DB"
```bash
lifeos query <db_key> --limit 20
lifeos query <db_key> --entry-type "Activity" --limit 50
lifeos query <db_key> --filter-property "Status" --filter-value "Active"
```
Or via MCP: `query` tool with `database`, `entry_type`, `limit` args.

### "I need to create a new entry"
```bash
lifeos mutate --operation create --database <db_key> --properties '{"Name":"...","Entry Type":"..."}'
```
Or via MCP: `mutate` tool.

### "I need to link two entries"
```bash
# By ID:
lifeos link --source <page_id> --target <page_id> --property "Generated From"

# By title (auto-resolves IDs):
lifeos quick-link --source-db matrix --source-title "..." --target-db potentiator --target-title "..." --property "Generated From"
```

### "I need to bulk-tag entries with universal properties"
```bash
# Dry-run first (preview what would change):
lifeos auto-enrich --mode tag --database matrix --limit 50

# Apply:
lifeos auto-enrich --mode tag --database matrix --limit 50 --apply
```

### "I need to see what's orphaned"
```bash
lifeos relational-gaps
lifeos orphans --database matrix
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
lifeos validate-yaml --db matrix     # Validate one DB
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
1. Create `lifeos-core/src/tools/my_tool.rs` with:
   - `pub struct MyToolParams { ... }` (derive `Deserialize`)
   - `pub fn schema() -> serde_json::Value { ... }`
   - `pub async fn execute(params, config, notion, schema_cache) -> Result<String, String>`
2. Register in `lifeos-core/src/tools/mod.rs`:
   - Add `pub mod my_tool;`
   - Add `tool_def("my_tool", "description", my_tool::schema())` to `get_tool_definitions`
   - Add dispatch arm to `call_tool`
3. Add CLI command in `lifeos-core/src/cli/mod.rs` + wire in `lifeos/src/main.rs`
4. Build + test: `cargo build --target x86_64-unknown-linux-gnu && ./target/.../lifeos my-tool --help`
5. Run `python3 /home/z/my-project/scripts/mcp_smoke_test.py` to verify tools/list includes it
6. Run `python3 skill/workspace-lint/scripts/workspace_lint.py`

### "I need to debug a Notion API issue"
- Check `lifeos-core/src/notion/client.rs` for the request shape.
- Use `ntn api <path> --docs` to read the official endpoint docs.
- Use `ntn api <path> --spec` to see the OpenAPI fragment.
- The Notion API 2025-09-03 has quirks — see §2.4 above.

---

## 8. Testing

### 8.1 Build verification
```bash
. "$HOME/.cargo/env"
cargo build --target x86_64-unknown-linux-gnu
```
Must complete with 0 errors. Warnings should be fixed in the same commit
that introduced them.

### 8.2 Schema self-test
```bash
./target/x86_64-unknown-linux-gnu/debug/lifeos validate-yaml --self-test
```
Loads 1 universal + 5 per_db + 31 per_entry_type schemas. Must report
"✅ All schemas passed self-test."

### 8.3 Discover smoke test
```bash
./target/x86_64-unknown-linux-gnu/debug/lifeos discover
```
Must resolve all 5 DBs (State, Possibility, Process, Identity, World),
sync ~297 property mappings, and discover ~63 relation edges.

### 8.4 MCP smoke test
```bash
python3 /home/z/my-project/scripts/mcp_smoke_test.py
```
14-test regression suite. Must report "14 passed, 0 failed." Covers:
- initialize (instructions include renamed DBs + correct currency flow)
- ping
- tools/list (≥28 tools, no phantom tools, expand + graph_metrics + auto_enrich visible)
- tools/call: get_schema
- tools/call: query matrix
- tools/call: auto_enrich dry-run
- batch JSON-RPC

### 8.5 workspace-lint
```bash
python3 skill/workspace-lint/scripts/workspace_lint.py
```
Must exit 0. Warnings are OK; errors are not.

---

## 9. Known Issues + Pending Items

See [AUDIT_v0.10.1_ACTION_TRACKER.md](AUDIT_v0.10.1_ACTION_TRACKER.md) for the
full prioritized list of fix / refine / upgrade / refactor / YAGNI items.

Top 4 high-priority items for the next sprint:

1. **U-9** — Demote `auto_enrich` to suggestion-only (remove `--apply` for tag mode). Aligns with user's manual-curation preference.
2. **D-ID-2** — Re-tag the 1 Identity entry using `Complex=Soul`, then remove `Soul` option via Notion UI.
3. **U-7** — Add `shadow_pattern_db_consistency` validation rule (Sinkhole only on World, Dark-* only on State, Golden-* only on Possibility).
4. **D-ID-1** — Rename `Identity.Stage` → `Life-Era` + remove `Active/Evolving/Archived` options (conflated with `Status`).

See also:
- [AUDIT_v0.10.1_DB_SCHEMA.md](AUDIT_v0.10.1_DB_SCHEMA.md) — per-DB schema inventory, relation topology, 14 drift items
- [AUDIT_v0.10.1_ARCHITECTURE.md](AUDIT_v0.10.1_ARCHITECTURE.md) — code-level audit, 10 YAGNI + 10 upgrade candidates
- [AUDIT_ponytail_ontology.md](AUDIT_ponytail_ontology.md) — pre-v0.10.1 audit (historical)

---

## 10. Contact

- Repo: [github.com/ishan-parihar/lifeos-ops](https://github.com/ishan-parihar/lifeos-ops)
- Ontology: [github.com/ishan-parihar/HoloOS](https://github.com/ishan-parihar/HoloOS)
- Author: [Ishan Parihar](https://github.com/ishan-parihar)

---

## 11. Quick Reference

| I want to... | Use this |
|---|---|
| Build the binary | `cargo build --target x86_64-unknown-linux-gnu` |
| Run schema self-test | `lifeos validate-yaml --self-test` |
| Discover DBs | `lifeos discover` |
| See the dashboard | `lifeos dashboard` |
| Run MCP smoke tests | `python3 /home/z/my-project/scripts/mcp_smoke_test.py` |
| Run workspace-lint | `python3 skill/workspace-lint/scripts/workspace_lint.py` |
| Use the Notion CLI directly | `ntn api ...` (see `skill/notion-cli/SKILL.md`) |
| Read the ontology | `ONTOLOGY.md` |
| Read the previous agent's work | `/home/z/my-project/worklog.md` |
| Commit + push | `git add -A && git commit -m "..." && git push origin main` |

---

*You are an AI agent working on LifeOS. The structure is the map. The
ontology is the territory. The 5 DBs are the only DBs. Push to main.
Run workspace-lint. Don't break the spiral.*
