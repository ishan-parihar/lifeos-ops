---
name: workspace-lint
description: Maintain a pristine, project-specific directory structure by enforcing rules declared in a root config file (e.g. `workspace-lint.yaml`). Use this skill proactively whenever an AI agent creates new files, scripts, docs, reports, logs, or artifacts in any project — every placement decision (where to put a `.py`, `.md`, `.mq5`, `.csv`, log file, or analysis report) should be checked against this skill's config before the file is written. Trigger on phrases like "organize files", "clean up directory", "where should I put this", "structure the workspace", "new script", "draft report", "save analysis to", "audit workspace", or any time you notice orphaned files at the project root, duplicate directories (e.g. `1. PHANTOM` and `PHANTOM`), stray build artifacts (`__pycache__`, `.pyc`, `.log`), or a project that looks "messy". Run the bundled validator (`scripts/workspace_lint.py`) after each iteration to catch drift early.
---

# Workspace Lint

Keep any project's directory structure pristine. The skill enforces rules declared in a root-level config file (`workspace-lint.yaml` by default). After every iteration, run the validator to detect drift before it compounds.

## When to use

- **Before creating any file**: Check the config to know where it belongs.
- **After each iteration**: Run the validator to catch misplaced files.
- **On any cleanup request**: Audit the workspace against the canonical layout.
- **When a project feels messy**: The validator will tell you exactly what violates rules.

## Two components work together

1. **Config** (`workspace-lint.yaml`) — declared in the project root. Defines the canonical structure, allowed files per directory, and forbidden patterns. Project-specific.
2. **Validator** (`scripts/workspace_lint.py`) — bundled with this skill. Audits the project against the config. Reports violations. Optionally fixes them with `--fix`.

Both must exist for the skill to function. The config is per-project; the validator is shared.

---

## 1. Setup: Author the Config

Place `workspace-lint.yaml` in the project root. Use the schema in `references/config-schema.md` as a reference. The minimum viable config:

```yaml
project:
  name: "MyProject"
  type: "quant-research"     # Free-form tag for grouping rules

structure:
  canonical:
    - path: "src"
      purpose: "All source code"
    - path: "tests"
      purpose: "All test files"
    - path: "docs"
      purpose: "Documentation and reports"

rules:
  root:
    forbidden_files:
      - "*.py"              # No Python at root
      - "*.js"
      - "*.md"              # Exception: README.md, AGENTS.md, CHANGELOG.md
      - "*.log"
    allowed_root_files:
      - "README.md"
      - "AGENTS.md"
      - "CHANGELOG.md"
      - ".gitignore"
      - "workspace-lint.yaml"

  files:
    "*.py":
      preferred_dir: "src"
      max_size_kb: 100
    "*.md":
      preferred_dir: "docs"

  directories:
    forbidden_patterns:
      - "^\\s"             # Leading whitespace
      - "\\s$"             # Trailing whitespace
      - "__pycache__"      # Build artifacts
      - "node_modules"
```

For fuller examples see `references/examples.md` (covers a single-purpose repo, a multi-package monorepo, and a research project).

## 2. Run the Validator

```bash
# Default: lint the current directory using ./workspace-lint.yaml
python3 scripts/workspace_lint.py

# Lint a specific directory
python3 scripts/workspace_lint.py --root /path/to/project

# Lint with a non-default config name
python3 scripts/workspace_lint.py --config my-lint.yaml

# Auto-fix safe violations (moves misplaced files, removes __pycache__)
python3 scripts/workspace_lint.py --fix

# Show only summary (no per-file report)
python3 scripts/workspace_lint.py --summary
```

Exit codes:
- `0` — no violations
- `1` — violations found (informational; CI may treat as failure)
- `2` — config missing or invalid

## 3. Interpret Validator Output

The validator emits one line per violation in this format:

```
<path>:<line>:<rule>: <message> [<severity>]
```

| Severity | Meaning | Should auto-fix? |
|---|---|---|
| `error` | Hard violation: file in wrong location, config-required path missing | Yes (move) |
| `warn` | Soft violation: large file, naming inconsistency, deviated from preferred dir | No |
| `info` | Hint: could improve but not required | No |

## 4. Apply the Skill to Your Workflow

When you are about to write a file (after analysis, after running a tool, after generating a report):

1. **Read the config first.** Locate `workspace-lint.yaml` in the project root. If it doesn't exist, ask the user whether to scaffold one (use the examples in `references/examples.md`).
2. **Match the file to a rule.** Most projects have rules like `*.py → src/` or `*.md → docs/reports/`. Place the file accordingly.
3. **If no rule matches:** Place the file in the closest canonical subdirectory, or ask the user. Don't dump files at the root.
4. **After writing the file**, run the validator:
   ```bash
   python3 scripts/workspace_lint.py --root .
   ```
   If violations appear, fix them before declaring the iteration done.
5. **Commit the config alongside the project.** The config is the single source of truth for structure. Any change to layout should update the config in the same commit.

## 5. Authoring Style

Author `workspace-lint.yaml` deliberately:

- **Whitelist root files**, don't only blacklist. Whitelisting is safer.
- **Use glob patterns over per-file entries.** `*.py` covers all Python files.
- **Avoid deeply nested `preferred_dir`s.** Three levels deep is the readable ceiling.
- **Make the config human-readable.** Think of it as a manifest, not a database.
- **Group rules by function.** Keep `structure` (what exists) separate from `rules` (what's allowed).

## 6. Common Pitfalls

- **Config drift.** When you reorganize directories, update `workspace-lint.yaml` in the same commit. Out-of-date configs produce false positives.
- **Whitelist `.gitignore`.** Forgetting it means the validator flags the config itself.
- **Empty directories.** The validator warns on empty canonical directories. Either commit a `.gitkeep` or remove the rule entry.
- **Mass moves via `--fix`.** Always review the diff before applying. Auto-fix only moves "obviously misplaced" files; ambiguity triggers a warn, not a fix.
- **Cross-platform paths.** Use forward slashes (`Scripts/Python/`) everywhere; Windows backslashes are not portable.

## 7. Bundled Resources

| Resource | Purpose |
|---|---|
| `scripts/workspace_lint.py` | The validator. Audit + optional fix. |
| `references/config-schema.md` | Full YAML schema with all supported keys. |
| `references/examples.md` | Three worked configs (single-purpose, monorepo, research). |
| `assets/template-config.yaml` | Drop-in starter config. Copy to project root and edit. |
| `evals/evals.json` | Reference test cases for the skill. |
