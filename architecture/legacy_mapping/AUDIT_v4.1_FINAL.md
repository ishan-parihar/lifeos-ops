# LifeOS v4.1 — Final Property Relevance Audit
# =============================================================================
# **AUDIT DATE:** 2026-07-07
# **SOURCE:** Live Notion API query AFTER user's UI updates (deletions + renames)
# **TOTAL:** 80 properties across 5 DBs, 8,554 entries
#
# **METHOD:** For each property, assess against the v4.1 functional architecture:
#   - Does it serve the teleological pull (Trajectory hierarchy)?
#   - Does it serve the ground-truth flow (capture → synthesize → condense)?
#   - Does it serve the feedback loop (Profile → Trajectory)?
#   - Does it serve the action interface (what to do + how)?
#   - Does it serve the context (who/what is around)?
#   If YES to any → NEEDED. If NO to all → DEAD WEIGHT.

---

## 1. Current State Summary

| DB | Properties | Entries | Status |
|----|-----------|---------|--------|
| Trajectory | 21 | 633 | ⚠ 2 issues (ghost relation + formula) |
| Logbook | 10 | 6,911 | ⚠ 1 issue (missing key relations) |
| Synthesis | 11 | 797 | ⚠ 1 issue (missing key relations) |
| Profile | 12 | 130 | ⚠ 2 issues (Entry Type options + missing relations) |
| Context | 26 | 83 | ⚠ 1 issue (ghost relation) |
| **TOTAL** | **80** | **8,554** | — |

Down from 149 → 80 (47% reduction). The user's UI cleanup was effective.

---

## 2. Per-DB Property Relevance Analysis

### 2.1 Trajectory (21 properties, 633 entries)

| # | Property | Type | Relevance to v4.1 | Verdict |
|---|----------|------|-------------------|---------|
| 1 | Name | title | Identity of the entry | ✅ NEEDED |
| 2 | ID | unique_id | Notion auto-ID | ✅ NEEDED (auto) |
| 3 | Item Type | select (16 opts) | THE discriminator — separates Purpose from Task | ✅ NEEDED (rename to `Type` via UI) |
| 4 | Status | status (6 opts) | Tracks lifecycle (Future→Active→Done) | ✅ NEEDED |
| 5 | Priority | select (5 opts) | Execution-layer prioritization | ✅ NEEDED |
| 6 | Progress | number | Tracks goal/project completion | ✅ NEEDED |
| 7 | Target | number | The measurable target | ✅ NEEDED |
| 8 | Start Date | date | When the entry begins | ✅ NEEDED |
| 9 | End Date | date | When the entry ends/due | ✅ NEEDED |
| 10 | Description | rich_text | What the entry IS (articulation) | ✅ NEEDED |
| 11 | Source | rich_text | Where a Value/Principle came from | ✅ NEEDED (for Reference layer) |
| 12 | Timeframe | select (5 opts) | For Vision-Statement (Lifetime/10yr/5yr/etc.) | ✅ NEEDED (for Reference layer) |
| 13 | Last Reviewed | date | When the user last reflected on a Reference entry | ✅ NEEDED (for Reference layer) |
| 14 | Parent item | relation → self (dual) | THE HIERARCHY: Task→Project→Goal→Vision | ✅ NEEDED (rename to `Parent` via UI) |
| 15 | Sub-item | relation → self (dual) | The child side of the hierarchy | ✅ NEEDED (rename to `Child` via UI) |
| 16 | Serves Value | relation → self (dual) | The constraining relation: Values constrain Actions | ✅ NEEDED |
| 17 | Blocks | relation → self | Dependency tracking | ✅ NEEDED (rename to `Blocked By` via UI) |
| 18 | Involves | relation → Context (dual) | Which People/Communities are involved | ✅ NEEDED |
| 19 | Quadrant | select (4 opts: UL/UR/LL/LR) | Wilber quadrant tagging | ⚠ OPTIONAL — useful for dimensional analysis but not core to v4.1. Keep if user uses it; delete if fill rate <5%. |
| 20 | Tier | select (3 opts: Strategic/Operational/Tactical) | Execution-layer classification | ⚠ OPTIONAL — useful but overlaps with Item Type (Annual-Goal is inherently Strategic, Task is inherently Tactical). Keep if used; delete if redundant. |
| 21 | Monitor | formula | Custom formula (status monitoring) | ⚠ OPTIONAL — check fill rate. If the formula produces useful data, keep. If it's broken or unused, delete. |
| 22 | Generates Logs | relation → **GHOST** | Should link to Logbook but points to 0baacff9 (deleted Potentiator) | ❌ **BROKEN** — needs re-pointing to Logbook data_source |

**Trajectory verdict:** 18 NEEDED + 3 OPTIONAL + 1 BROKEN. The broken `Generates Logs` relation must be re-pointed to the Logbook data_source ID (`a1769af1-...`).

---

### 2.2 Logbook (10 properties, 6,911 entries)

| # | Property | Type | Relevance to v4.1 | Verdict |
|---|----------|------|-------------------|---------|
| 1 | Name | title | Identity of the log entry | ✅ NEEDED |
| 2 | ID | unique_id | Notion auto-ID | ✅ NEEDED (auto) |
| 3 | Date | date | THE primary time index — enables date-range filtering (the replacement for Days/Weeks/Months DBs) | ✅ NEEDED |
| 4 | Entry Type | select (6 opts) | THE discriminator: Activity/Diet/Financial/Subjective/Relational/Systemic | ✅ NEEDED |
| 5 | Distillation | rich_text | The log content | ✅ NEEDED (rename to `Content` via UI) |
| 6 | Amount | formula | Auto-calculates from sub-properties (financial amount, calories, etc.) | ⚠ OPTIONAL — if the formula works and produces useful data, keep. Check if it references deleted properties. |
| 7 | Duration | formula | Auto-calculates duration from date range | ⚠ OPTIONAL — same check. |
| 8 | Month Label | formula | Auto-derives month from Date for grouping | ✅ NEEDED (replaces the legacy Months DB — the user explicitly said "filters within existing DBs can do the work") |
| 9 | Quarter Label | formula | Auto-derives quarter from Date | ✅ NEEDED (replaces the legacy Quarters DB) |
| 10 | Week Label | formula | Auto-derives week from Date | ✅ NEEDED (replaces the legacy Weeks DB) |

**MISSING properties (were deleted in Phase A but are needed for v4.1 cycle):**

| Missing property | Type | Target | Purpose | Verdict |
|-----------------|------|--------|---------|---------|
| Channel | select (Body/Mind/Resource/Relational) | — | Derived from Entry Type; enables channel-based filtering | ⚠ Was added in Phase B but appears deleted — re-add |
| Sentiment | select (Positive/Neutral/Negative) | — | For Subjective/Relational logs | ⚠ Was added in Phase B but appears deleted — re-add |
| Source Project | relation → Trajectory | many-to-one | Which Project/Task this log belongs to | ❌ MISSING — critical for the Ground-Truth flow (Action → Logbook) |
| Subject Person | relation → Context | many-to-one | For Relational logs — who was the interaction with | ❌ MISSING — critical for Context linking |
| Subject Account | relation → Context | many-to-one | For Financial logs — which account | ❌ MISSING — critical for Context linking |
| Synthesized Into | relation → Synthesis | many-to-many | Which Synthesis entries this log fed into | ❌ MISSING — critical for the Ground-Truth flow (Logbook → Synthesis) |

**Logbook verdict:** 8 NEEDED + 2 OPTIONAL (formulas) + **6 MISSING** (2 props + 4 relations). The missing relations are critical — without them, the Ground-Truth flow (Action → Logbook → Synthesis) is broken.

---

### 2.3 Synthesis (11 properties, 797 entries)

| # | Property | Type | Relevance to v4.1 | Verdict |
|---|----------|------|-------------------|---------|
| 1 | Name | title | Identity of the synthesis entry | ✅ NEEDED |
| 2 | ID | unique_id | Notion auto-ID | ✅ NEEDED (auto) |
| 3 | Date | date | When the insight was synthesized | ✅ NEEDED |
| 4 | Category | select (5 opts) | THE discriminator: Note/Opportunity/Strength/Directive/Risk | ✅ NEEDED |
| 5 | Polarity | select (3 opts) | +/−/neutral — the polar structure | ✅ NEEDED |
| 6 | Priority | select (4 opts) | Critical/High/Medium/Low | ✅ NEEDED |
| 7 | Status | status (4 emoji opts) | 💡 Identified → ✅ Activated → 🏆 Capitalized → 🧊 Archived | ✅ NEEDED |
| 8 | Capture Method | select (5 opts) | How the insight was captured | ✅ NEEDED |
| 9 | Source URL | url | For web clips | ✅ NEEDED |
| 10 | Raw Content | rich_text | The raw synthesis content | ✅ NEEDED |
| 11 | Synthesis State | select (4 opts) | raw_note → annotated → synthesized → applied | ✅ NEEDED |

**MISSING properties (were added in Phase B but appear deleted):**

| Missing property | Type | Target | Purpose | Verdict |
|-----------------|------|--------|---------|---------|
| Source Logs | relation → Logbook | many-to-many | Which logs this synthesized from | ❌ MISSING — critical for the Ground-Truth flow |
| Spawns | relation → Trajectory | one-to-many | Directives spawn corrective Actions | ❌ MISSING — critical for the Ground-Truth flow (Synthesis → Trajectory) |
| Revises | relation → Trajectory | many-to-many | Long-term insights reaffirm/revise Vision | ❌ MISSING — critical for the Feedback flow |
| Condenses Into | relation → Profile | many-to-many | Long-term insights condense into Profile traits | ❌ MISSING — critical for the Ground-Truth flow (Synthesis → Profile) |

**Synthesis verdict:** 11 NEEDED + **4 MISSING relations**. All 11 existing properties are relevant. But the 4 inter-DB relations that wire Synthesis into the cycle are missing — they were added in Phase B but appear to have been deleted during the user's UI cleanup (possibly accidentally).

---

### 2.4 Profile (12 properties, 130 entries)

| # | Property | Type | Relevance to v4.1 | Verdict |
|---|----------|------|-------------------|---------|
| 1 | Name | title | Identity of the profile entry | ✅ NEEDED |
| 2 | ID | unique_id | Notion auto-ID | ✅ NEEDED (auto) |
| 3 | Entry Type | multi_select (10 opts) | THE discriminator — BUT has 6 stale options | ✅ NEEDED (reduce to 4: Trait/Metric/Capacity/Asset) |
| 4 | Status | status (4 opts) | Draft/Active/Evolving/Archived | ✅ NEEDED |
| 5 | Category | select (8 opts) | Health/Financial/Relational/Cognitive/Spiritual/Execution/Content/Strategic | ✅ NEEDED |
| 6 | Current Value | rich_text | The current state | ✅ NEEDED |
| 7 | Target Value | rich_text | The ideal-future target (for gap calculation) | ✅ NEEDED |
| 8 | Trend | select (3 opts) | ↑/↓/→ | ✅ NEEDED |
| 9 | Unit | select (8 opts) | count/percentage/hours/rupees/etc. | ✅ NEEDED |
| 10 | Frequency | select (5 opts) | Daily/Weekly/Monthly/Quarterly/Annual | ✅ NEEDED |
| 11 | Last Reviewed | date | When last updated | ✅ NEEDED (rename to `Last Updated` via UI) |
| 12 | Informs Goal | relation → Trajectory | Profile informs which Goals are needed | ✅ NEEDED |

**ISSUE:** Entry Type still has 10 options (Trait/Metric/Capacity/Asset + Purpose/Value/Principle/Identity-Statement/Pillar/Strategic-Ideal). The last 6 are stale — they belong in Trajectory now, not Profile.

**MISSING properties (were added in Phase B but appear deleted):**

| Missing property | Type | Target | Purpose | Verdict |
|-----------------|------|--------|---------|---------|
| Closes Gap For | relation → Trajectory | many-to-many | Shows the gap between current state and ideal-future | ❌ MISSING — critical for the Feedback flow |
| Source Synthesis | relation → Synthesis | many-to-many | Which Synthesis entries condensed into this trait | ❌ MISSING — critical for the Ground-Truth flow (Synthesis → Profile) |

**Profile verdict:** 12 NEEDED (1 needs option cleanup) + **2 MISSING relations**. All 12 properties are relevant. The Entry Type needs option reduction. The 2 missing relations wire Profile into the cycle.

---

### 2.5 Context (26 properties, 83 entries)

| # | Property | Type | Relevance to v4.1 | Verdict |
|---|----------|------|-------------------|---------|
| 1 | Name | title | Identity | ✅ NEEDED |
| 2 | Type | select (5 opts) | THE discriminator: Person/Community/Org/Account/Place | ✅ NEEDED |
| 3 | Status | select (3 opts) | Active/Inactive/Archived | ✅ NEEDED |
| 4-17 | Person-specific (14 props) | various | CRM properties (Aspirational Drive, Developmental Altitude, Networking Profile, etc.) | ✅ NEEDED (for Person entries) |
| 18-20 | Community-specific (3 props) | various | Community Type, Strategic Value, Covenant | ✅ NEEDED (for Community entries) |
| 21-23 | Financial-Account-specific (3 props) | various | Account Type, Balance, Institution | ✅ NEEDED (for Financial-Account entries) |
| 24 | Related to Trajectory (Involves) | relation → Trajectory (dual) | Person/Community involved in Projects | ✅ NEEDED |
| 25 | Referenced In | relation → Synthesis | Person/Account referenced in Synthesis entries | ✅ NEEDED |
| 26 | Subject Of | relation → **GHOST** | Should link to Logbook but points to 0baacff9 | ❌ **BROKEN** — needs re-pointing to Logbook |

**Context verdict:** 25 NEEDED + 1 BROKEN. The `Subject Of` relation points to GHOST — needs re-pointing to Logbook data_source.

---

## 3. The Broken + Missing Relations (Critical for the v4.1 Cycle)

The v4.1 cycle requires these inter-DB relations to be wired:

```
Trajectory →(Generates Logs)→ Logbook →(Synthesized Into)→ Synthesis
→(Condenses Into)→ Profile →(Closes Gap For / Informs Goal)→ Trajectory
```

### Currently BROKEN (point to GHOST):

| DB | Property | Current target | Should target | Action |
|----|----------|---------------|---------------|--------|
| Trajectory | Generates Logs | GHOST (0baacff9) | Logbook (a1769af1) | Re-point |
| Context | Subject Of | GHOST (0baacff9) | Logbook (a1769af1) | Re-point |

### Currently MISSING (deleted during UI cleanup — need re-creation):

| From DB | Property | To DB | Cardinality | Purpose |
|---------|----------|-------|-------------|---------|
| Logbook | Source Project | Trajectory | many-to-one | Action → Logbook link |
| Logbook | Subject Person | Context | many-to-one | Context → Logbook link |
| Logbook | Subject Account | Context | many-to-one | Context → Logbook link |
| Logbook | Synthesized Into | Synthesis | many-to-many | Logbook → Synthesis link |
| Synthesis | Source Logs | Logbook | many-to-many | Synthesis ← Logbook link (reciprocal) |
| Synthesis | Spawns | Trajectory | one-to-many | Synthesis → Trajectory link |
| Synthesis | Revises | Trajectory | many-to-many | Synthesis → Trajectory link |
| Synthesis | Condenses Into | Profile | many-to-many | Synthesis → Profile link |
| Profile | Closes Gap For | Trajectory | many-to-many | Profile → Trajectory link |
| Profile | Source Synthesis | Synthesis | many-to-many | Profile ← Synthesis link (reciprocal) |

**Total: 2 BROKEN + 10 MISSING = 12 relations to fix/create.** Without these, the v4.1 cycle is not wired.

---

## 4. Property Relevance Summary

### Properties that are NEEDED (all serve the v4.1 architecture):

| DB | NEEDED | Optional | Broken/Missing | Total |
|----|--------|----------|---------------|-------|
| Trajectory | 18 | 3 (Quadrant, Tier, Monitor) | 1 (Generates Logs → GHOST) | 21+1 |
| Logbook | 8 | 2 (Amount, Duration formulas) | 6 (Channel, Sentiment, Source Project, Subject Person/Account, Synthesized Into) | 10+6 |
| Synthesis | 11 | 0 | 4 (Source Logs, Spawns, Revises, Condenses Into) | 11+4 |
| Profile | 12 | 0 | 2 (Closes Gap For, Source Synthesis) + Entry Type cleanup | 12+2 |
| Context | 25 | 0 | 1 (Subject Of → GHOST) | 26+1 |
| **TOTAL** | **74** | **5** | **14** | **80+14** |

### Properties that are OPTIONAL (user decides based on fill rate):

1. **Trajectory.Quadrant** (UL/UR/LL/LR) — Wilber quadrant tagging. Useful for dimensional analysis but not core to v4.1. Run `lifeos fill-rate` to check.
2. **Trajectory.Tier** (Strategic/Operational/Tactical) — Overlaps with Item Type (Annual-Goal is inherently Strategic, Task is inherently Tactical). May be redundant.
3. **Trajectory.Monitor** (formula) — Check if the formula still works (may reference deleted properties).
4. **Logbook.Amount** (formula) — Check if the formula still works.
5. **Logbook.Duration** (formula) — Check if the formula still works.

### Properties that need UI cleanup:

1. **Profile.Entry Type** — reduce from 10 to 4 options (remove Purpose/Value/Principle/Identity-Statement/Pillar/Strategic-Ideal; keep Trait/Metric/Capacity/Asset)

### Renames still needed (via Notion UI):

1. Trajectory: `Item Type` → `Type`
2. Trajectory: `Parent item` → `Parent`
3. Trajectory: `Sub-item` → `Child`
4. Trajectory: `Blocks` → `Blocked By`
5. Logbook: `Distillation` → `Content`
6. Profile: `Last Reviewed` → `Last Updated`

---

## 5. Action Plan

### Immediate (API — I can do now):

1. **Re-point 2 broken relations** (Trajectory.Generates Logs + Context.Subject Of → Logbook)
2. **Re-create 10 missing relations** (Logbook ↔ Synthesis, Synthesis → Trajectory/Profile, Profile → Trajectory/Synthesis)
3. **Re-add 2 missing properties** (Logbook.Channel + Logbook.Sentiment — if they were deleted)

### UI-only (you must do in Notion):

1. **Reduce Profile.Entry Type** to 4 options (remove 6 stale Reference types)
2. **Rename 6 properties** (Item Type→Type, Parent item→Parent, etc.)
3. **Check 5 optional properties** (Quadrant, Tier, Monitor, Amount, Duration) — keep or delete based on usage

### Verification:

After the API fixes + UI cleanup, re-run the audit to confirm:
- 0 ghost relations
- 0 missing cycle relations
- 0 stale options
- ~80-85 properties total (optimal for the v4.1 consciousness-prosthetic)

---

*Final property relevance audit. 74 needed + 5 optional + 14 to fix = 93 target properties (up from current 80, because the 14 missing relations must be re-created).*
