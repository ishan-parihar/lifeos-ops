# LifeOS v4.1 — Deep Property Audit: Content-Type Perception + YAML Formula Architecture
# =============================================================================
# **AUDIT DATE:** 2026-07-07
# **METHOD:** Perceive each DB by the TYPE OF CONTENT it holds, not by its
#            property list. Then ask: which properties are SELF-INFERENTIAL from
#            the Item Type (redundant), and which could be replaced by a single
#            YAML-generating formula that expands per-item-type context?
#
# **CORE INSIGHT (from user):** A generalized property table for all item-types
# is limiting because different item-types need different property spaces. A
# complex formula can generate YAML output that EXPANDS the context per
# item-type — replacing multiple redundant properties with one formula property.

---

## 1. The Problem with Generalized Property Tables

Trajectory has 16 item-types across 3 layers (Reference / Strategic / Execution). Each layer has DIFFERENT property needs:

| Layer | Example item-types | Properties they actually need | Properties they DON'T need |
|-------|-------------------|-------------------------------|---------------------------|
| Reference | Purpose, Value, Principle, Vision-Statement, Identity-Statement | Description, Source, Timeframe, Last Reviewed, Status, Serves Value | Priority, Progress, Target, Start Date, End Date, Tier |
| Strategic | Annual-Goal, Quarterly-Goal, Milestone | Description, Status, Progress, Target, Start Date, End Date, Parent, Serves Value | Source, Timeframe, Last Reviewed, Priority, Tier |
| Execution | Project, Task, Campaign, Content | Description, Status, Priority, Progress, Start Date, End Date, Parent, Involves, Generates Logs | Source, Timeframe, Last Reviewed, Target, Tier |

**The current 19-property Trajectory table forces ALL 16 item-types to share the same property surface.** A Purpose entry shows empty Priority/Progress/Target/Start Date/End Date/Tier fields. A Task shows empty Source/Timeframe/Last Reviewed fields. This is noise.

The same problem applies to Logbook (6 entry-types sharing 16 properties) and Context (5 entry-types sharing 26 properties).

---

## 2. The YAML Formula Solution

**Instead of having separate properties for each dimension, have ONE formula property that generates a YAML string per item-type.** The formula reads the Item Type and outputs different YAML based on which type it is.

### Example: Trajectory `Context YAML` formula

```notion-formula
ifs(
  prop("Item Type") == "Purpose",
    "type: purpose\ntimeline: lifetime\nreview: annual\nsource: " + prop("Source"),
  prop("Item Type") == "Value",
    "type: value\nconstraint: non-negotiable\nsource: " + prop("Source"),
  prop("Item Type") == "Principle",
    "type: principle\ndomain: decision-rule\nsource: " + prop("Source"),
  prop("Item Type") == "Vision-Statement",
    "type: vision\ntimeframe: " + prop("Timeframe") + "\nreview: " + formatDate(prop("Last Reviewed"), "yyyy-MM-dd"),
  prop("Item Type") == "Annual-Goal",
    "type: annual-goal\nyear: " + prop("Status") + "\nprogress: " + prop("Progress") + "%\ntarget: " + prop("Target"),
  prop("Item Type") == "Quarterly-Goal",
    "type: quarterly-goal\nprogress: " + prop("Progress") + "%\ntarget: " + prop("Target"),
  prop("Item Type") == "Project",
    "type: project\nstatus: " + prop("Status") + "\npriority: " + prop("Priority") + "\nprogress: " + prop("Progress") + "%",
  prop("Item Type") == "Task",
    "type: task\nstatus: " + prop("Status") + "\npriority: " + prop("Priority") + "\ndue: " + formatDate(prop("End Date"), "yyyy-MM-dd"),
  prop("Item Type") == "Campaign",
    "type: campaign\nstatus: " + prop("Status") + "\nprogress: " + prop("Progress") + "%",
  prop("Item Type") == "Content",
    "type: content\nstatus: " + prop("Status") + "\npriority: " + prop("Priority"),
  prop("Item Type") == "Milestone",
    "type: milestone\nstatus: " + prop("Status"),
  prop("Item Type") == "System" or prop("Item Type") == "Resource" or prop("Item Type") == "Sprint" or prop("Item Type") == "Budget",
    "type: " + prop("Item Type") + "\nstatus: " + prop("Status"),
  ""
)
```

**What this gives you:** One `Context YAML` formula property that outputs structured YAML per item-type. An AI agent reading this gets the FULL context of the entry — including which fields are relevant — without needing to parse 19 separate properties.

### How this replaces redundant properties

| Current property | Replaced by YAML formula? | Why |
|-----------------|--------------------------|-----|
| Priority | ✅ YES for Reference/Strategic; KEEP for Execution | Reference entries don't have priority. The formula only includes Priority when Item Type is Project/Task/Campaign/Content. |
| Progress | ✅ YES for Reference; KEEP for Strategic/Execution | Purpose doesn't have progress. |
| Target | ✅ YES for Reference/Execution; KEEP for Strategic | Values don't have targets. |
| Start Date / End Date | ✅ YES for Reference; KEEP for Strategic/Execution | Purpose doesn't have dates. |
| Source | ✅ YES for Strategic/Execution; KEEP for Reference | Tasks don't have sources. |
| Timeframe | ✅ YES for everything except Vision-Statement; KEEP for Vision-Statement | Only Vision-Statement needs Timeframe. |
| Last Reviewed | ✅ YES for Strategic/Execution; KEEP for Reference | Only Reference entries need review tracking. |
| Tier | ✅ YES — fully inferable from Item Type | Annual-Goal is Strategic, Task is Tactical. The formula infers this. |
| Monitor | ✅ YES — replace with YAML formula | Current formula is broken anyway. |
| Description | ❌ KEEP — universal | All entries need a description. |
| Status | ❌ KEEP — universal | All entries need status. |
| Name | ❌ KEEP — universal | — |
| ID | ❌ KEEP — auto | — |

---

## 3. Per-DB Deep Audit: Content-Type Perception

### 3.1 Trajectory — "The teleological hierarchy"

**What content does it hold?** 3 fundamentally different content types:
1. **Articulations** (Reference layer) — timeless text expressing Purpose/Values/Principles/Vision/Identity
2. **Targets** (Strategic layer) — time-bound goals with progress + targets
3. **Actions** (Execution layer) — deliverables with priority + dependencies + people

**The problem:** All 3 content types share the same 19-property table. 60% of properties are empty for any given entry.

**Self-inferential properties (can be inferred from Item Type):**

| Property | Inferable from Item Type? | How |
|----------|--------------------------|-----|
| Tier (Strategic/Operational/Tactical) | ✅ YES | Annual-Goal → Strategic; Project → Operational; Task → Tactical. Fully redundant. |
| Timeframe (Lifetime/10yr/5yr/3yr/1yr) | ⚠ PARTIALLY | Only Vision-Statement needs this. For all other types, it's empty. Could be in YAML formula. |
| Monitor (formula) | ⚠ BROKEN | References deleted properties. Replace with YAML formula. |

**Properties that are low-priority / could be in YAML formula:**

| Property | Why low-priority | Move to YAML? |
|----------|-----------------|---------------|
| Source | Only used by Reference layer (5 of 16 types). 31 entries out of 633. | ✅ Move to YAML — formula includes Source only for Reference types |
| Last Reviewed | Only used by Reference layer. Same 31 entries. | ✅ Move to YAML — formula includes Last Reviewed only for Reference types |
| Timeframe | Only used by Vision-Statement (1 of 16 types). ~5 entries. | ✅ Move to YAML — formula includes Timeframe only for Vision-Statement |
| Quadrant | Already deleted by user. | ✅ Done |
| Priority | Only relevant for Execution layer (4 of 16 types). 434 entries. | ⚠ KEEP as property — used by enough entries to justify a real property. But the YAML formula references it for context. |
| Progress | Only relevant for Strategic + Execution (7 of 16 types). ~475 entries. | ⚠ KEEP as property — same reasoning. |
| Target | Only relevant for Strategic + some Execution. ~100 entries. | ⚠ KEEP as property — used by enough entries. |
| Start Date / End Date | Only relevant for Strategic + Execution. ~400 entries. | ⚠ KEEP as property — date filtering is primary use case. |

**Trajectory target after YAML formula optimization:**

| Property | Type | KEEP | MOVE TO YAML | DELETE |
|----------|------|------|-------------|--------|
| Name | title | ✅ | | |
| ID | unique_id | ✅ | | |
| Item Type | select | ✅ | | |
| Description | rich_text | ✅ | | |
| Status | status | ✅ | | |
| Priority | select | ✅ | | |
| Progress | number | ✅ | | |
| Target | number | ✅ | | |
| Start Date | date | ✅ | | |
| End Date | date | ✅ | | |
| Parent item | relation | ✅ | | |
| Sub-item | relation | ✅ | | |
| Serves Value | relation | ✅ | | |
| Blocks | relation | ✅ | | |
| Involves | relation | ✅ | | |
| Generates Logs | relation | ✅ | | |
| Source | rich_text | | ✅ | |
| Timeframe | select | | ✅ | |
| Last Reviewed | date | | ✅ | |
| Tier | select | | | ✅ (fully redundant — infer from Item Type) |
| Monitor | formula | | ✅ (replace with YAML formula) | |
| **Context YAML** | formula (NEW) | ✅ (NEW) | | |
| **TOTAL** | | **16** | **4** | **1** |

**Result: 17 properties (16 real + 1 YAML formula), down from 19.** The 4 moved-to-YAML properties (Source, Timeframe, Last Reviewed, Monitor) are still accessible — they're encoded in the YAML formula output, visible to AI agents, but don't clutter the property table for non-Reference entries.

---

### 3.2 Logbook — "The objective capture surface"

**What content does it hold?** 6 channels of objective ground-reality data:
1. **Body data** (Activity, Diet) — physical metrics + duration + amount
2. **Mind data** (Subjective, Systemic) — reflections + observations
3. **Resource data** (Financial) — transactions + amounts
4. **Relational data** (Relational) — interactions + sentiment

**The problem:** The 4 formula properties (Amount, Duration, Month/Quarter/Week Label) are all derived from Date — they're auto-computed. Channel is derivable from Entry Type.

**Self-inferential properties:**

| Property | Inferable from Entry Type? | How |
|----------|---------------------------|-----|
| Channel (Body/Mind/Resource/Relational) | ✅ YES | Activity/Diet → Body; Subjective/Systemic → Mind; Financial → Resource; Relational → Relational. Fully redundant. |
| Amount (formula) | ⚠ CHECK | Formula may be broken (references sub-properties). If it works, it's useful for Financial/Diet. |
| Duration (formula) | ⚠ CHECK | Same — may be broken. |
| Month/Quarter/Week Label | ❌ KEEP | These REPLACE the legacy Days/Weeks/Months/Quarters DBs. The user explicitly said "filters within existing DBs can do the work." These formulas enable date-range grouping. |

**Properties that could be in YAML formula:**

| Property | Why | Move to YAML? |
|----------|-----|---------------|
| Channel | Fully inferable from Entry Type. | ✅ Move to YAML — formula outputs channel based on Entry Type |
| Sentiment | Only relevant for Subjective/Relational (2 of 6 types). ~158 entries of 6,911. | ⚠ KEEP as property — it's a curated user input, not auto-derivable |

**Logbook target after YAML formula optimization:**

| Property | Type | KEEP | MOVE TO YAML | DELETE |
|----------|------|------|-------------|--------|
| Name | title | ✅ | | |
| ID | unique_id | ✅ | | |
| Date | date | ✅ | | |
| Entry Type | select | ✅ | | |
| Content | rich_text | ✅ | | |
| Sentiment | select | ✅ | | |
| Source Project | relation | ✅ | | |
| Subject Person | relation | ✅ | | |
| Subject Account | relation | ✅ | | |
| Synthesized Into | relation | ✅ | | |
| Month Label | formula | ✅ | | |
| Quarter Label | formula | ✅ | | |
| Week Label | formula | ✅ | | |
| Amount | formula | ✅ (if working) | | |
| Duration | formula | ✅ (if working) | | |
| Channel | select | | ✅ | |
| **Context YAML** | formula (NEW) | ✅ (NEW) | | |
| **TOTAL** | | **15** | **1** | **0** |

**Result: 16 properties (15 real + 1 YAML formula), down from 16.** Marginal change — Logbook is already lean. The main win is removing Channel (redundant) and adding the YAML formula for AI-agent context.

---

### 3.3 Synthesis — "The insight engine"

**What content does it hold?** 5 types of insights:
1. **Raw captures** (Note) — meeting notes, web clips, voice memos
2. **Positive signals** (Opportunity, Strength) — capitalize
3. **Negative signals** (Directive, Risk) — correct

**The problem:** Polarity is self-inferential from Category. Status overlaps with Synthesis State.

**Self-inferential properties:**

| Property | Inferable from Category? | How |
|----------|------------------------|-----|
| Polarity (+/−/neutral) | ✅ YES | Note → neutral; Opportunity/Strength → +; Directive/Risk → −. Fully redundant. |
| Synthesis State (raw_note/annotated/synthesized/applied) | ⚠ OVERLAPS with Status | Status (💡 Identified → ✅ Activated → 🏆 Capitalized → 🧊 Archived) and Synthesis State (raw_note → annotated → synthesized → applied) track the SAME lifecycle from different angles. One should be deleted. |

**Properties that could be in YAML formula:**

| Property | Why | Move to YAML? |
|----------|-----|---------------|
| Polarity | Fully inferable from Category. | ✅ Move to YAML — formula outputs polarity based on Category |
| Capture Method | Only relevant for Note type. ~794 of 797 entries are Notes. | ⚠ KEEP as property — nearly universal |
| Priority | Useful for all types. | ❌ KEEP |
| Source URL | Only relevant for web clips. | ⚠ KEEP as property — useful when populated |

**Synthesis target after YAML formula optimization:**

| Property | Type | KEEP | MOVE TO YAML | DELETE |
|----------|------|------|-------------|--------|
| Name | title | ✅ | | |
| ID | unique_id | ✅ | | |
| Date | date | ✅ | | |
| Category | select | ✅ | | |
| Priority | select | ✅ | | |
| Status | status | ✅ | | |
| Capture Method | select | ✅ | | |
| Source URL | url | ✅ | | |
| Raw Content | rich_text | ✅ | | |
| Source Logs | relation (auto-dual) | ✅ | | |
| Spawns | relation | ✅ | | |
| Revises | relation | ✅ | | |
| Condenses Into | relation (auto-dual) | ✅ | | |
| Polarity | select | | ✅ | |
| Synthesis State | select | | | ✅ (overlaps with Status — delete) |
| **Context YAML** | formula (NEW) | ✅ (NEW) | | |
| **TOTAL** | | **14** | **1** | **1** |

**Result: 14 properties (13 real + 1 YAML formula), down from 15.** Remove Polarity (redundant) + Synthesis State (overlaps Status). Add YAML formula.

---

### 3.4 Profile — "The cumulative state mirror"

**What content does it hold?** 4 types of state:
1. **Traits** — enduring characteristics (discipline, communication style)
2. **Metrics** — measurable indicators (net worth, weight, sleep quality)
3. **Capacities** — skill levels (systems thinking, writing)
4. **Assets** — built resources (LifeOS, portfolio, network)

**The problem:** All 4 types share the same property surface, but they have different needs:
- Traits don't have a Unit or Frequency
- Metrics need Unit + Frequency + Current Value + Target Value + Trend
- Capacities need a level system (not Unit)
- Assets don't have Trend or Frequency

**Self-inferential properties:**

| Property | Inferable from Entry Type? | How |
|----------|---------------------------|-----|
| Frequency | ⚠ Only relevant for Metrics | Traits/Capacities/Assets don't have a measurement frequency. |
| Unit | ⚠ Only relevant for Metrics | Same — only Metrics need a unit. |
| Trend | ⚠ Only relevant for Metrics | Same — only Metrics have a trend direction. |
| Target Value | ⚠ Less relevant for Assets | Assets don't have a "target" — they exist or don't. |

**Properties that could be in YAML formula:**

| Property | Why | Move to YAML? |
|----------|-----|---------------|
| Frequency | Only Metrics need it. ~34 of 130 entries. | ✅ Move to YAML — formula includes Frequency only for Metrics |
| Unit | Only Metrics need it. ~34 entries. | ✅ Move to YAML — formula includes Unit only for Metrics |
| Trend | Only Metrics need it. ~34 entries. | ✅ Move to YAML — formula includes Trend only for Metrics |
| Target Value | Metrics + some Capacities. ~40 entries. | ⚠ KEEP as property — used by enough entries + useful for gap visualization |

**Profile target after YAML formula optimization:**

| Property | Type | KEEP | MOVE TO YAML | DELETE |
|----------|------|------|-------------|--------|
| Name | title | ✅ | | |
| ID | unique_id | ✅ | | |
| Entry Type | multi_select | ✅ | | |
| Status | status | ✅ | | |
| Category | select | ✅ | | |
| Current Value | rich_text | ✅ | | |
| Target Value | rich_text | ✅ | | |
| Last Updated | date | ✅ | | |
| Closes Gap For | relation | ✅ | | |
| Informs Goal | relation | ✅ | | |
| Source Synthesis | relation (auto-dual) | ✅ | | |
| Frequency | select | | ✅ | |
| Unit | select | | ✅ | |
| Trend | select | | ✅ | |
| **Context YAML** | formula (NEW) | ✅ (NEW) | | |
| **TOTAL** | | **12** | **3** | **0** |

**Result: 12 properties (11 real + 1 YAML formula), down from 14.** Move Frequency/Unit/Trend to YAML — they're only relevant for Metrics (26% of entries).

---

### 3.5 Context — "The environment"

**What content does it hold?** 5 types:
1. **Person** (14 specific properties) — rich CRM
2. **Community** (3 specific properties) — group CRM
3. **Organization** (0 specific properties) — basic org tracking
4. **Financial-Account** (3 specific properties) — account state
5. **Place** (0 specific properties) — location

**The problem:** 14 Person-specific properties show as empty for Community/Org/Account/Place entries. 83 entries but 26 properties = significant noise for non-Person types.

**Self-inferential properties:**

| Property | Inferable from Type? | How |
|----------|---------------------|-----|
| All 14 Person-specific props | ✅ YES — only populated for Type=Person | A Person entry uses all 14; a Community entry uses 0. These are already self-inferential — the question is whether to keep them as properties or move to YAML. |

**The trade-off:** Person-specific properties (Aspirational Drive, Developmental Altitude, etc.) are SELECT properties that the USER actively sets. They're not auto-derivable. Moving them to YAML would mean the user can't set them via Notion's select UI — they'd have to type YAML manually or use the API.

**Recommendation:** KEEP Person-specific properties as real properties. The user needs to set them interactively. YAML formula doesn't help here — it would only DISPLAY them, not enable setting them.

**However:** Add a `Context YAML` formula that aggregates the relevant properties per type — so an AI agent gets a clean YAML summary without scanning 26 properties:

```notion-formula
ifs(
  prop("Type") == "Person",
    "type: person\naspirational_drive: " + prop("Aspirational Drive") + "\ndevelopmental_altitude: " + prop("Developmental Altitude") + "\nnetworking: " + prop("Networking Profile") + "\nrelationship: " + prop("Relationship Status") + "\ntrajectory: " + prop("Desired Trajectory") + "\nbalance: " + prop("Value Exchange Balance"),
  prop("Type") == "Community",
    "type: community\ntype_detail: " + prop("Community Type") + "\nstrategic_value: " + prop("Strategic Value") + "\ncovenant: " + prop("Covenant"),
  prop("Type") == "Financial-Account",
    "type: account\naccount_type: " + prop("Account Type") + "\nbalance: " + prop("Balance") + "\ninstitution: " + prop("Institution"),
  prop("Type") == "Organization" or prop("Type") == "Place",
    "type: " + prop("Type") + "\nstatus: " + prop("Status"),
  ""
)
```

**Context target:** Keep all 26 properties (they're all needed for their respective types). Add 1 `Context YAML` formula. Total: 27.

---

## 4. Summary: The YAML Formula Architecture

### What the YAML formula does

Each DB gets ONE `Context YAML` formula property that:
1. **Reads the Item Type** (or Entry Type / Category / Type)
2. **Outputs structured YAML** containing only the fields relevant to that type
3. **AI agents read the YAML** to get full context without parsing N properties
4. **Human users see clean property tables** (via saved views that hide irrelevant props per type)

### What gets replaced

| DB | Current props | Props moved to YAML | Props deleted | New YAML formula | Target total |
|----|--------------|--------------------|--------------|-----------------|-------------|
| Trajectory | 19 | 4 (Source, Timeframe, Last Reviewed, Monitor) | 1 (Tier) | 1 | 16 |
| Logbook | 16 | 1 (Channel) | 0 | 1 | 16 |
| Synthesis | 15 | 1 (Polarity) | 1 (Synthesis State) | 1 | 14 |
| Profile | 14 | 3 (Frequency, Unit, Trend) | 0 | 1 | 12 |
| Context | 26 | 0 (keep all — user-set selects) | 0 | 1 | 27 |
| **TOTAL** | **90** | **9** | **2** | **5** | **85** |

**Net: 90 → 85 properties (6% reduction) + 5 YAML formulas that expand context per item-type.**

But the real win isn't the property count — it's the CLARITY. Each entry now has a clean YAML context that an AI agent can parse in one read, without scanning 19 empty properties to find the 3 that matter.

### What gets DELETED (fully redundant — no YAML needed):

| DB | Property | Why deleted |
|----|----------|-------------|
| Trajectory | Tier | Fully inferable from Item Type (Annual-Goal→Strategic, Task→Tactical) |
| Synthesis | Synthesis State | Overlaps with Status (both track lifecycle) |
| Trajectory | Generates Logs (GHOST) | Needs re-pointing (not deletion) |
| Synthesis | Related to Logbook (Synthesized Into) → GHOST | Needs re-pointing (not deletion) |
| Context | Subject Of → GHOST | Needs re-pointing (not deletion) |

### What gets MOVED TO YAML (property deleted, value encoded in formula):

| DB | Property | Why moved |
|----|----------|-----------|
| Trajectory | Source | Only used by Reference layer (5 of 16 types, ~31 of 633 entries) |
| Trajectory | Timeframe | Only used by Vision-Statement (1 of 16 types, ~5 entries) |
| Trajectory | Last Reviewed | Only used by Reference layer (~31 entries) |
| Trajectory | Monitor | Broken formula anyway — replace with YAML |
| Logbook | Channel | Fully inferable from Entry Type |
| Synthesis | Polarity | Fully inferable from Category |
| Profile | Frequency | Only used by Metrics (~34 of 130 entries) |
| Profile | Unit | Only used by Metrics |
| Profile | Trend | Only used by Metrics |

---

## 5. Implementation Plan

### Phase 1: Create the 5 YAML formulas (via Notion UI — formulas are too complex for API)

For each DB, add a `Context YAML` formula property with the type-conditional YAML generation. I'll provide the exact formula text for each DB.

### Phase 2: Delete the 2 fully-redundant properties (via Notion UI)

| DB | Property | Action |
|----|----------|--------|
| Trajectory | Tier | Delete via Notion UI |
| Synthesis | Synthesis State | Delete via Notion UI |

### Phase 3: Move 9 properties to YAML (via Notion UI — delete the property after confirming the formula works)

For each of the 9 properties being moved to YAML:
1. Add the YAML formula that references the property
2. Verify the formula output includes the property's value
3. Delete the original property (its value is now encoded in the YAML)

### Phase 4: Fix 3 ghost relations (via Notion UI — API can't delete dual_property)

| DB | Property | Action |
|----|----------|--------|
| Trajectory | Generates Logs | Re-point to Logbook (or delete + recreate) |
| Synthesis | Related to Logbook (Synthesized Into) | Re-point to Logbook |
| Context | Subject Of | Re-point to Logbook |

### Phase 5: Verify

- [ ] Each DB has a `Context YAML` formula that outputs type-specific YAML
- [ ] 0 ghost relations remain
- [ ] 0 fully-redundant properties remain
- [ ] Total properties: ~85 (down from 90)
- [ ] AI agents can read `Context YAML` to get full entry context in one field

---

## 6. The Bigger Picture: Why This Matters

The YAML formula architecture solves a fundamental tension in the LifeOS design:

**Tension:** A generalized DB (one table for all item-types) is operationally simpler (fewer DBs, unified views) but forces all item-types to share the same property surface (noise from empty properties).

**Solution:** The YAML formula EXPANDS the property space per item-type without adding properties to the table. Each item-type gets its own virtual property space — encoded in the YAML output — while the physical table stays clean.

This is how the LifeOS consciousness-prosthetic achieves **maximum efficiency** (clean tables, minimal properties) AND **maximum efficacy** (rich per-type context via YAML) AND **maximum adaptability** (new item-types can be added by extending the formula, not by adding properties).

---

*Deep audit v4.1. Content-type-perceived analysis + YAML formula architecture proposal. Ready for implementation.*
