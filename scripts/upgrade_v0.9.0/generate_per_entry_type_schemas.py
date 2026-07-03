#!/usr/bin/env python3
"""
Generate per-entry-type YAML schema files for the LifeOS v0.9.0 upgrade.

For each (DB, Entry-Type) pair, writes a YAML file at:
    schemas/per_entry_type/<db>__<entry_type_slug>.yaml

Each file:
  - inherits from universal/holon_coordinate.yaml + per_db/<db>.yaml
  - adds entry-type-specific extended properties (grounded in HoloOS docs)
  - declares which entry-type it specializes
  - lists the entry-type-specific validation rules

Run:
    python3 /home/z/my-project/repos/lifeos-ops/scripts/upgrade_v0.9.0/generate_per_entry_type_schemas.py
"""

from pathlib import Path
import yaml

REPO_ROOT = Path("/home/z/my-project/repos/lifeos-ops")
OUT_DIR   = REPO_ROOT / "schemas" / "per_entry_type"
OUT_DIR.mkdir(parents=True, exist_ok=True)


def slug(s: str) -> str:
    return s.lower().replace(" ", "_").replace("-", "_").replace("/", "_")


# ─────────────────────────────────────────────────────────────────────────────
# Load universal + per_db property names to detect redundancy at generation time
# ─────────────────────────────────────────────────────────────────────────────

def _load_layer_property_names(layer: str, db: str = None) -> set[str]:
    """Load property names from a schema layer (universal or per_db)."""
    if layer == "universal":
        path = REPO_ROOT / "schemas" / "universal" / "holon_coordinate.yaml"
    else:
        path = REPO_ROOT / "schemas" / "per_db" / f"{db}.yaml"
    if not path.exists():
        return set()
    with open(path) as f:
        schema = yaml.safe_load(f) or {}
    return set((schema.get("properties") or {}).keys())


# Cache these at module load time so the generator can detect redundancy
_UNIVERSAL_PROPS = _load_layer_property_names("universal")
_PER_DB_PROPS = {db: _load_layer_property_names("per_db", db) for db in
                 ("matrix", "potentiator", "nexus", "significator", "greatway")}


def filter_redundant_props(db: str, props: dict) -> dict:
    """Remove any property from `props` that's already in universal or per_db for this DB.

    This prevents the generator from reintroducing redundancy that was cleaned up
    by fix_schema_redundancy.py.
    """
    redundant = _UNIVERSAL_PROPS | _PER_DB_PROPS.get(db, set())
    return {k: v for k, v in props.items() if k not in redundant}


# ─────────────────────────────────────────────────────────────────────────────
# Shared property blocks for entry-types that share common properties.
# These are properties that were MOVED OUT of per_db into per_entry_type by
# fix_schema_redundancy.py — they must be declared here so the generator
# produces complete per_entry_type files on re-run.
# ─────────────────────────────────────────────────────────────────────────────

# Properties required by ALL external-holon entry-types in GreatWay
# (Person, Group, Community, Organization, Network, Movement, Place).
# Per HoloOS doc holon_type_placement.yaml + 02.3 §4 (bonding at Significator⇄Great-Way surface).
EXTERNAL_HOLON_PROPS = {
    "quadrant": {
        "notion_type": "select", "required": False,
        "description": "AQAL quadrant (HoloOS doc holon_type_placement.yaml).",
        "options": ["UL", "UR", "LL", "LR"],
    },
    "dimension": {
        "notion_type": "select", "required": False,
        "description": "Interior/Exterior dimension (HoloOS doc holon_type_placement.yaml).",
        "options": ["Interior", "Exterior"],
    },
}

# Bonding properties — only for entry-types that can bond with a Significator
# (Person, Group, Community, Organization, Network, Movement — NOT Place, which
# is a geographic environment, not a bonding partner).
BONDING_PROPS = {
    "bonding_disposition": {
        "notion_type": "select", "required": False,
        "description": "Per HoloOS doc 02.3 §4: Bonding occurs at the Significator⇄Great-Way surface. 4 bonding types: ionic, covalent, dative, metallic.",
        "options": ["ionic", "covalent", "dative", "metallic"],
    },
    "bonding_partner_count": {
        "notion_type": "number", "required": False,
        "description": "Number of active Significator-side bonds this GreatWay entry maintains. Used to compute metallic-bonding lattice density.",
    },
}

# Properties required by Transformation-kind Nexus entry-types
# (Pattern, Crisis, Transformation-Event).
TRANSFORMATION_KIND_PROPS = {
    "crucible_intensity": {
        "notion_type": "select", "required": False,
        "description": "Per HoloOS doc holon_states.yaml greater_cycle.preconditions.",
        "options": ["moderate", "acute"],
    },
    "transformation_threshold": {
        "notion_type": "number", "required": False,
        "description": "T_thresh — the threshold the Significator must reach for this Transformation-kind Nexus entry to fire (HoloOS doc 03.1 §3 stage 8). Default: 110.0 (from lifeos.config.default.json nexus_firing.pressure_threshold).",
    },
}

# Properties required by Choice-kind Nexus entry-types (Directive, Decision).
CHOICE_KIND_PROPS = {
    "choice_polarity": {
        "notion_type": "select", "required": False,
        "description": "STO/STS polarity of an emitted Choice (HoloOS doc 02.4 §2.3).",
        "options": ["STO", "STS", "neutral"],
    },
    "crystallization_ratio": {
        "notion_type": "number", "required": False,
        "description": "Per HoloOS doc holon_states.yaml greater_cycle.preconditions. Min 0.7 required for `choice-locked` greater-cycle phase. Range: 0.0-1.0.",
    },
}

# Properties required by Note/Knowledge Nexus entry-types
# (Note, Knowledge-Category, Knowledge-Atom).
KNOWLEDGE_FLOW_PROPS = {
    "synthesis_state": {
        "notion_type": "select", "required": False,
        "description": "Tracks the 9-stage digestion state of a note/knowledge atom as it crystallizes from Catalyst-class Note into Experience-class Knowledge into refined Catalyst.",
        "options": ["raw_note", "annotated", "synthesized", "applied"],
    },
}

# Properties required by Principle-family Significator entry-types
# (Purpose, Value, Principle — per Flag 3 consolidation).
PRINCIPLE_FAMILY_PROPS = {
    "principle_sub_type": {
        "notion_type": "select", "required": False,
        "description": "For Principle-family entries only (per Flag 3 — consolidates Purpose/Value/Principle into one entry-type with a sub-type discriminator).",
        "options": ["Purpose", "Value", "Principle"],
    },
}


# ─────────────────────────────────────────────────────────────────────────────
# Per-entry-type specializations.
# Each entry: (db, entry_type, archetype_num_or_None, extra_props, extra_rules, doc_ref)
# ─────────────────────────────────────────────────────────────────────────────
SPECIALIZATIONS = [

    # ───────── MATRIX ENTRY-TYPES (archetypes 1, 8, 15) ─────────

    ("matrix", "Pattern", 1, {
        "attention_gating_pattern": {
            "notion_type": "select",
            "required": False,
            "description": "HoloOS doc 04.2.1 §4 archetype 1 row. How this Matrix-of-Mind pattern gates attention.",
            "options": ["narrow_focus", "broad_scan", "alternating", "default_mode"],
        },
        "silence_depth": {
            "notion_type": "number", "required": False,
            "description": "HoloOS doc 04.2.1 §4. Capacity to sustain mental silence.",
        },
        "self_knowing_clarity": {
            "notion_type": "select", "required": False,
            "description": "HoloOS doc 04.2.1 §4. Reflective self-awareness quality.",
            "options": ["opaque", "translucent", "transparent", "luminous"],
        },
    }, [], "04.2.1_Mind_Complex_Architecture.md §4 (archetype 1)"),

    ("matrix", "Practice", 8, {
        "activity_baseline": {
            "notion_type": "number", "required": False,
            "description": "HoloOS doc 04.2.2 §4 archetype 8 row. Baseline activity level for this body-complex practice.",
        },
        "even_functioning_score": {
            "notion_type": "select", "required": False,
            "description": "HoloOS doc 04.2.2 §4. Evenness of physiological functioning.",
            "options": ["erratic", "uneven", "steady", "harmonized"],
        },
        "substrate_set_point": {
            "notion_type": "rich_text", "required": False,
            "description": "HoloOS doc 04.2.2 §4. Homeostatic set-point the practice maintains.",
        },
    }, [], "04.2.2_Body_Complex_Architecture.md §4 (archetype 8)"),

    ("matrix", "Foundation", 15, {
        "receptive_depth_score": {
            "notion_type": "number", "required": False,
            "description": "HoloOS doc 04.2.3 §5 archetype 15 row. Receptive depth to primeval darkness.",
        },
        "darkness_discernment": {
            "notion_type": "select", "required": False,
            "description": "HoloOS doc 04.2.3 §5. Capacity to discern signal in the Night of the Soul.",
            "options": ["undifferentiated", "differentiating", "lucid", "integrated"],
        },
        "depth_field_stability": {
            "notion_type": "number", "required": False,
            "description": "HoloOS doc 04.2.3 §5. Stability of the depth-field under perturbation.",
        },
    }, [], "04.2.3_Spirit_Complex_Architecture.md §5 (archetype 15)"),

    ("matrix", "Threshold", None, {
        "trigger_threshold_value": {
            "notion_type": "number", "required": False,
            "description": "HoloOS doc 03.1 §3 stage 8. Value the Matrix-state must reach for this Threshold to fire.",
        },
        "trigger_catalyst_class": {
            "notion_type": "select", "required": False,
            "description": "Class of Catalyst that triggers this Threshold entry.",
            "options": ["informational", "emotional", "somatic", "relational", "environmental", "systemic"],
        },
    }, [
        {"id": "threshold_must_have_trigger",
         "description": "Threshold entries must specify either trigger_threshold_value OR trigger_catalyst_class.",
         "rule": "assert entry.trigger_threshold_value is not None or entry.trigger_catalyst_class is not None"},
    ], "03.1_Universal_Archetype_Anatomy.md §3 stage 8"),

    ("matrix", "Inventory", None, {
        "inventory_class": {
            "notion_type": "select", "required": False,
            "description": "Class of inventory (resource / health / asset / capability).",
            "options": ["resource", "health", "asset", "capability", "social_capital"],
        },
        "current_count": {"notion_type": "number", "required": False, "description": "Current count/level."},
        "unit": {"notion_type": "select", "required": False, "description": "Unit of measure.",
                 "options": ["count", "hours", "currency", "kg", "percent", "score"]},
    }, [], "Operational inventory entry-type"),

    ("matrix", "Habit", None, {
        "habit_frequency": {
            "notion_type": "select", "required": False,
            "options": ["daily", "weekly", "biweekly", "monthly", "quarterly", "ad_hoc"],
        },
        "habit_streak": {"notion_type": "number", "required": False, "description": "Current consecutive-completion streak."},
        "habit_cue": {"notion_type": "rich_text", "required": False, "description": "The cue that triggers this habit."},
        "habit_reward": {"notion_type": "rich_text", "required": False, "description": "The reward that completes the habit loop."},
    }, [], "Operational habit entry-type (HoloOS doc 04.2.2 §1 Body complex activity)"),

    ("matrix", "Routine", None, {
        "routine_window": {
            "notion_type": "select", "required": False,
            "options": ["morning", "midday", "afternoon", "evening", "night", "weekly_review", "monthly_review"],
        },
        "routine_step_count": {"notion_type": "number", "required": False, "description": "Number of sequential steps in this routine."},
        "routine_anchor": {"notion_type": "rich_text", "required": False, "description": "What this routine anchors to (time, event, location)."},
    }, [], "Operational routine entry-type (HoloOS doc 04.2.2 §1 Body complex activity)"),

    ("matrix", "Active Project", None, {
        "project_health": {
            "notion_type": "select", "required": False,
            "options": ["green", "yellow", "red", "blocked"],
        },
        "progress_percent": {"notion_type": "number", "required": False, "description": "Estimated completion percent."},
        "deadline_date": {"notion_type": "date", "required": False, "description": "Project deadline."},
        "project_lead": {"notion_type": "relation", "required": False, "description": "Person (GreatWay entry) leading the project."},
    }, [], "Operational project entry-type"),

    # ───────── POTENTIATOR ENTRY-TYPES (archetypes 2, 9, 16) ─────────

    ("potentiator", "Subjective", 2, {
        "interior_resonance": {
            "notion_type": "select", "required": False,
            "description": "HoloOS doc 04.2.1 §4 archetype 2. Resonance quality of the inner-model possibility-space.",
            "options": ["discordant", "neutral", "harmonic", "transcendent"],
        },
        "latency_horizon": {
            "notion_type": "select", "required": False,
            "description": "How far into the future this latent possibility extends.",
            "options": ["immediate", "days", "weeks", "months", "years", "decades"],
        },
    }, [], "04.2.1_Mind_Complex_Architecture.md §4 (archetype 2)"),

    ("potentiator", "Activity", 9, {
        "action_modality": {
            "notion_type": "select", "required": False,
            "description": "HoloOS doc 04.2.2 §4 archetype 9. Modality of possible action.",
            "options": ["exploratory", "exploitative", "restorative", "transformative"],
        },
        "energy_envelope": {
            "notion_type": "number", "required": False,
            "description": "Energy cost / envelope required to manifest this possibility.",
        },
    }, [], "04.2.2_Body_Complex_Architecture.md §4 (archetype 9)"),

    ("potentiator", "Relational", 16, {
        "relational_field_type": {
            "notion_type": "select", "required": False,
            "description": "HoloOS doc 04.2.3 §5 archetype 16. Type of relational-field possibility.",
            "options": ["intimate", "familial", "professional", "communal", "transpersonal"],
        },
        "depth_horizon": {
            "notion_type": "select", "required": False,
            "description": "Vertical reach of this relational possibility.",
            "options": ["surface", "mid", "deep", "foundational"],
        },
    }, [], "04.2.3_Spirit_Complex_Architecture.md §5 (archetype 16)"),

    ("potentiator", "Systemic", None, {
        "system_layer": {
            "notion_type": "select", "required": False,
            "options": ["technosphere", "biosphere", "noosphere", "sociosphere", "econosphere"],
        },
        "leverage_score": {
            "notion_type": "number", "required": False,
            "description": "Leverage score — how much system-level change this possibility could drive.",
        },
    }, [], "Systemic-entry-type (HoloOS doc 04.2.2 §4 systemic substrates)"),

    ("potentiator", "Diet", None, {
        "diet_class": {
            "notion_type": "select", "required": False,
            "options": ["food", "information", "media", "social", "somatic"],
        },
        "caloric_load": {"notion_type": "number", "required": False},
        "digestibility_score": {"notion_type": "number", "required": False, "description": "0-1 ease of metabolic integration."},
    }, [
        {"id": "diet_must_be_catalyst_role",
         "description": "Per UPGRADE_FLAGS.md Flag 1: Diet entries must have archetype_role=Catalyst.",
         "rule": "assert entry.archetype_role == 'Catalyst'"},
    ], "Per Flag 1: Diet is Catalyst-class currency"),

    ("potentiator", "Financial", None, {
        "financial_class": {
            "notion_type": "select", "required": False,
            "options": ["income", "expense", "investment", "debt", "asset", "flow"],
        },
        "amount": {"notion_type": "number", "required": False},
        "currency_code": {"notion_type": "select", "required": False,
                          "options": ["USD", "EUR", "INR", "GBP", "JPY", "BTC", "ETH"]},
    }, [
        {"id": "financial_must_be_catalyst_role",
         "description": "Per UPGRADE_FLAGS.md Flag 1: Financial entries must have archetype_role=Catalyst.",
         "rule": "assert entry.archetype_role == 'Catalyst'"},
    ], "Per Flag 1: Financial is Catalyst-class currency"),

    ("potentiator", "Observation", None, {
        "observation_class": {
            "notion_type": "select", "required": False,
            "options": ["empirical", "intuitive", "reported", "inferred", "remembered"],
        },
        "confidence_score": {"notion_type": "number", "required": False, "description": "0-1 confidence in this observation."},
        "source_attribution": {"notion_type": "rich_text", "required": False, "description": "Where the observation came from."},
    }, [
        {"id": "observation_must_be_catalyst_role",
         "description": "Per UPGRADE_FLAGS.md Flag 1: Observation entries must have archetype_role=Catalyst.",
         "rule": "assert entry.archetype_role == 'Catalyst'"},
    ], "Per Flag 1: Observation is Catalyst-class currency"),

    ("potentiator", "Goal", None, {
        "goal_horizon": {
            "notion_type": "select", "required": False,
            "options": ["30d", "90d", "180d", "365d", "3y", "5y", "10y", "lifetime"],
        },
        "goal_specificity": {
            "notion_type": "select", "required": False,
            "options": ["vision", "aspiration", "objective", "smart_goal", "milestone"],
        },
        "achievement_probability": {"notion_type": "number", "required": False, "description": "0-1 estimated probability of achievement."},
    }, [], "Relocated FROM GreatWay per Flag 3 (Goal = latent future-input)"),

    ("potentiator", "Vision", None, {
        "vision_clarity": {
            "notion_type": "select", "required": False,
            "options": ["germinal", "emerging", "crystallizing", "crystallized"],
        },
        "vision_resonance_score": {"notion_type": "number", "required": False, "description": "0-1 resonance with identity (Significator)."},
    }, [], "Relocated FROM GreatWay per Flag 3 (Vision = latent future-input)"),

    ("potentiator", "Aspiration", None, {
        "aspiration_horizon": {
            "notion_type": "select", "required": False,
            "options": ["near", "mid", "long", "lifetime", "transpersonal"],
        },
        "aspiration_pull_strength": {"notion_type": "number", "required": False,
            "description": "0-1 strength of pull this aspiration exerts on the Significator."},
        "aspiration_alignment_with_purpose": {"notion_type": "number", "required": False,
            "description": "0-1 alignment with the Significator's Purpose entry."},
    }, [], "Relocated FROM GreatWay per Flag 3 (Aspiration = latent future-input)"),

    # ───────── NEXUS ENTRY-TYPES (archetypes 3, 4, 6, 10, 11, 13, 17, 18, 20, 22) ─────────

    ("nexus", "Opportunity", 3, {
        "opportunity_horizon": {
            "notion_type": "select", "required": False,
            "description": "Time horizon of this opportunity-window.",
            "options": ["closing_now", "days", "weeks", "months", "years", "indefinite"],
        },
        "opportunity_cost": {"notion_type": "number", "required": False, "description": "Estimated opportunity cost of not pursuing."},
    }, [
        {"id": "opportunity_kind_constraint",
         "description": "Opportunity entries must have Kind=Catalyst.",
         "rule": "assert entry.kind == 'Catalyst'"},
    ], "04.2.1_Mind_Complex_Architecture.md §4 (archetype 3 Catalyst of Mind)"),

    ("nexus", "Insight", 10, {
        "insight_class": {
            "notion_type": "select", "required": False,
            "options": ["perceptual", "conceptual", "experiential", "revelatory"],
        },
        "insight_magnitude": {"notion_type": "number", "required": False, "description": "0-10 magnitude of cognitive restructuring."},
    }, [
        {"id": "insight_kind_constraint",
         "description": "Insight entries must have Kind=Catalyst.",
         "rule": "assert entry.kind == 'Catalyst'"},
    ], "04.2.2_Body_Complex_Architecture.md §4 (archetype 10 Catalyst of Body)"),

    ("nexus", "Reflection", 4, {
        "reflection_depth": {
            "notion_type": "select", "required": False,
            "options": ["descriptive", "analytical", "interpretive", "integrative", "transformative"],
        },
        "reflection_period": {
            "notion_type": "select", "required": False,
            "options": ["daily", "weekly", "monthly", "quarterly", "annual", "event_specific"],
        },
    }, [
        {"id": "reflection_kind_constraint",
         "description": "Reflection entries must have Kind=Experience.",
         "rule": "assert entry.kind == 'Experience'"},
    ], "04.2.1_Mind_Complex_Architecture.md §4 (archetype 4 Experience of Mind)"),

    ("nexus", "Integration", 11, {
        "integration_scope": {
            "notion_type": "select", "required": False,
            "options": ["intra_complex", "cross_complex", "cross_cycle", "cross_octave"],
        },
        "integration_completeness": {"notion_type": "number", "required": False, "description": "0-1 how complete the integration is."},
    }, [
        {"id": "integration_kind_constraint",
         "description": "Integration entries must have Kind=Experience.",
         "rule": "assert entry.kind == 'Experience'"},
    ], "04.2.2_Body_Complex_Architecture.md §4 (archetype 11 Experience of Body)"),

    ("nexus", "Pattern", 6, {
        "pattern_recurrence_count": {"notion_type": "number", "required": False, "description": "Number of times this pattern has recurred."},
        "pattern_recurrence_interval": {
            "notion_type": "select", "required": False,
            "options": ["daily", "weekly", "monthly", "quarterly", "annual", "irregular"],
        },
        "pattern_archetypal_resonance": {
            "notion_type": "rich_text", "required": False,
            "description": "Which of the 22 named archetypes this pattern most resonates with.",
        },
    }, [
        {"id": "pattern_kind_constraint",
         "description": "Pattern entries must have Kind=Transformation.",
         "rule": "assert entry.kind == 'Transformation'"},
    ], "04.2.1_Mind_Complex_Architecture.md §4 (archetype 6 Transformation of Mind)"),

    ("nexus", "Risk", None, {
        "risk_probability": {"notion_type": "number", "required": False, "description": "0-1 probability of materialization."},
        "risk_impact": {"notion_type": "number", "required": False, "description": "0-10 impact if materialized."},
        "risk_horizon": {
            "notion_type": "select", "required": False,
            "options": ["immediate", "days", "weeks", "months", "quarters", "years"],
        },
        "mitigation_strategy": {"notion_type": "rich_text", "required": False},
    }, [
        {"id": "risk_kind_constraint",
         "description": "Risk entries must have Kind=Catalyst (anticipated perturbation).",
         "rule": "assert entry.kind == 'Catalyst'"},
    ], "Operational risk entry-type"),

    ("nexus", "Directive", 22, {
        "directive_polarity": {
            "notion_type": "select", "required": False,
            "description": "STO / STS polarity of the emitted directive (HoloOS doc 02.4 §2.3).",
            "options": ["STO", "STS", "neutral"],
        },
        "directive_scope": {
            "notion_type": "select", "required": False,
            "options": ["personal", "team", "organizational", "civilizational"],
        },
        "directive_horizon": {
            "notion_type": "select", "required": False,
            "options": ["immediate", "tactical", "strategic", "long_term", "epochal"],
        },
    }, [
        {"id": "directive_kind_constraint",
         "description": "Directive entries must have Kind=Choice.",
         "rule": "assert entry.kind == 'Choice'"},
    ], "03.2_22_Named_Archetypes_Index.md (archetype 22 — Choice meta-pivot)"),

    # ───────── NEXUS: RELOCATED FROM POTENTIATOR ─────────

    ("nexus", "Note", None, {
        "source_url": {"notion_type": "url", "required": False, "description": "Original URL where this note was sourced."},
        "capture_method": {
            "notion_type": "select", "required": False,
            "options": ["manual", "web_clipper", "api_ingest", "voice_memo", "import"],
        },
        "raw_content": {"notion_type": "rich_text", "required": False, "description": "Original raw note content (before processing)."},
        "highlight_count": {"notion_type": "number", "required": False, "description": "Number of highlights extracted."},
        "synthesis_state": {
            "notion_type": "select", "required": False,
            "options": ["raw_note", "annotated", "synthesized", "applied"],
            "description": "Tracks the 9-stage digestion state of this note.",
        },
    }, [
        {"id": "note_kind_constraint",
         "description": "Note entries must have Kind=Catalyst (raw notes are unprocessed perturbations).",
         "rule": "assert entry.kind == 'Catalyst'"},
    ], "RELOCATED FROM Potentiator (raw notes are Catalyst-class currency per HoloOS doc 02.1 §1)"),

    ("nexus", "Knowledge-Category", None, {
        "category_scope": {
            "notion_type": "select", "required": False,
            "options": ["domain", "discipline", "methodology", "framework", "toolkit"],
        },
        "atom_count": {"notion_type": "number", "required": False, "description": "Number of Knowledge-Atoms in this category."},
        "maturity_score": {
            "notion_type": "number", "required": False,
            "description": "0-1 maturity of the category (per HoloOS doc 03.1 §3 stages 5-6).",
        },
        "applied_count": {"notion_type": "number", "required": False, "description": "Number of times this category has been applied in practice."},
    }, [
        {"id": "knowledge_category_kind_constraint",
         "description": "Knowledge-Category entries must have Kind=Experience (digested, categorized knowledge).",
         "rule": "assert entry.kind == 'Experience'"},
    ], "RELOCATED FROM Potentiator (digested notes are Experience-class currency per HoloOS doc 02.1 §1)"),

    ("nexus", "Knowledge-Atom", None, {
        "atom_statement": {"notion_type": "rich_text", "required": True, "description": "The single synthesized knowledge statement."},
        "evidence_strength": {
            "notion_type": "select", "required": False,
            "options": ["anecdotal", "case_study", "correlational", "causal", "axiomatic"],
        },
        "confidence_score": {"notion_type": "number", "required": False, "description": "0-1 confidence in this knowledge atom."},
        "source_count": {"notion_type": "number", "required": False, "description": "Number of independent sources corroborating this atom."},
    }, [
        {"id": "knowledge_atom_kind_constraint",
         "description": "Knowledge-Atom entries must have Kind=Experience.",
         "rule": "assert entry.kind == 'Experience'"},
    ], "RELOCATED FROM Potentiator (single synthesized knowledge units)"),

    ("nexus", "Decision", None, {
        "decision_polarity": {
            "notion_type": "select", "required": False,
            "options": ["STO", "STS", "neutral"],
        },
        "decision_scope": {
            "notion_type": "select", "required": False,
            "options": ["personal", "relational", "operational", "strategic", "epochal"],
        },
        "decision_irreversibility": {
            "notion_type": "select", "required": False,
            "options": ["reversible", "partially_reversible", "irreversible", "existential"],
        },
        "decided_at": {"notion_type": "date", "required": False},
    }, [
        {"id": "decision_kind_constraint",
         "description": "Decision entries must have Kind=Choice (decisions are micro-Choices emitted through the contact boundary).",
         "rule": "assert entry.kind == 'Choice'"},
    ], "RELOCATED FROM GreatWay per Flag 3 (Decisions are Choice-class currency)"),

    ("nexus", "Crisis", None, {
        "crisis_intensity": {
            "notion_type": "select", "required": False,
            "options": ["low", "moderate", "acute", "existential"],
        },
        "crisis_domain": {
            "notion_type": "select", "required": False,
            "options": ["health", "relational", "financial", "professional", "existential", "civilizational"],
        },
        "crisis_started_at": {"notion_type": "date", "required": False},
        "crisis_resolved_at": {"notion_type": "date", "required": False},
        "resolution_path": {"notion_type": "rich_text", "required": False},
    }, [
        {"id": "crisis_kind_constraint",
         "description": "Crisis entries must have Kind=Transformation (acute threshold events).",
         "rule": "assert entry.kind == 'Transformation'"},
    ], "Operational crisis entry-type (HoloOS doc 03.1 §3 stage 8)"),

    ("nexus", "Transformation-Event", None, {
        "transformation_magnitude": {
            "notion_type": "select", "required": False,
            "options": ["micro", "minor", "moderate", "major", "epochal"],
        },
        "transformation_axis": {
            "notion_type": "select", "required": False,
            "options": ["mind", "body", "spirit", "relational", "environmental", "systemic"],
        },
        "transformation_triggered_at": {"notion_type": "date", "required": False},
        "transformation_completed_at": {"notion_type": "date", "required": False},
        "pre_transformation_signature": {"notion_type": "rich_text", "required": False, "description": "Valence Signature YAML BEFORE transformation."},
        "post_transformation_signature": {"notion_type": "rich_text", "required": False, "description": "Valence Signature YAML AFTER transformation."},
    }, [
        {"id": "transformation_event_kind_constraint",
         "description": "Transformation-Event entries must have Kind=Transformation.",
         "rule": "assert entry.kind == 'Transformation'"},
    ], "Operational transformation-event entry-type"),

    # ───────── SIGNIFICATOR ENTRY-TYPES (archetypes 5, 12, 19) ─────────

    ("significator", "Identity-Statement", 5, {
        "statement_clarity": {
            "notion_type": "select", "required": False,
            "options": ["germinal", "emerging", "articulated", "consolidated", "luminous"],
        },
        "statement_resonance_score": {"notion_type": "number", "required": False, "description": "0-1 resonance score."},
    }, [], "04.2.1_Mind_Complex_Architecture.md §4 (archetype 5 Significator of Mind)"),

    ("significator", "Pillar", 12, {
        "pillar_load_bearing": {
            "notion_type": "select", "required": False,
            "options": ["primary", "secondary", "tertiary"],
        },
        "pillar_substrate": {
            "notion_type": "select", "required": False,
            "options": ["somatic", "cognitive", "affective", "relational", "vocational", "spiritual"],
        },
    }, [], "04.2.2_Body_Complex_Architecture.md §4 (archetype 12 Significator of Body)"),

    ("significator", "Strategic-Ideal", 19, {
        "ideal_horizon": {
            "notion_type": "select", "required": False,
            "options": ["5y", "10y", "25y", "lifetime", "transpersonal"],
        },
        "ideal_resonance_with_significator": {"notion_type": "number", "required": False, "description": "0-1 alignment with current identity."},
    }, [], "04.2.3_Spirit_Complex_Architecture.md §5 (archetype 19 Significator of Spirit)"),

    ("significator", "Purpose", None, {
        "purpose_horizon": {
            "notion_type": "select", "required": False,
            "options": ["vocation", "calling", "dharma", "destiny"],
        },
    }, [
        {"id": "purpose_principle_sub_type",
         "description": "Purpose entries must have principle_sub_type=Purpose.",
         "rule": "assert entry.principle_sub_type == 'Purpose'"},
    ], "Significator Purpose entry-type (per Flag 3 consolidation)"),

    ("significator", "Value", None, {
        "value_polarity": {
            "notion_type": "select", "required": False,
            "options": ["STO", "STS", "transcendent"],
        },
    }, [
        {"id": "value_principle_sub_type",
         "description": "Value entries must have principle_sub_type=Value.",
         "rule": "assert entry.principle_sub_type == 'Value'"},
    ], "Significator Value entry-type (per Flag 3 consolidation)"),

    ("significator", "Principle", None, {
        "principle_domain": {
            "notion_type": "select", "required": False,
            "options": ["ethical", "operational", "epistemic", "aesthetic", "spiritual"],
        },
    }, [
        {"id": "principle_principle_sub_type",
         "description": "Principle entries must have principle_sub_type=Principle.",
         "rule": "assert entry.principle_sub_type == 'Principle'"},
    ], "Significator Principle entry-type (per Flag 3 consolidation)"),

    ("significator", "Archetype", None, {
        "archetype_number": {"notion_type": "number", "required": False, "description": "1-22 per HoloOS doc 03.2."},
        "tarot_correspondence": {"notion_type": "select", "required": False,
            "options": ["Magician", "High Priestess", "Empress", "Emperor", "Hierophant", "Lovers",
                        "Chariot", "Strength", "Hermit", "Wheel of Fortune", "Justice", "Hanged Man",
                        "Death", "Temperance", "Devil", "Tower", "Star", "Moon", "Sun", "Judgement",
                        "World", "Fool"]},
    }, [], "22 named archetypes as identity configurations (HoloOS doc 03.2)"),

    # ───────── GREATWAY ENTRY-TYPES (archetypes 7, 14, 21 + relocated) ─────────

    ("greatway", "Annual Goal", 7, {
        "year": {"notion_type": "number", "required": False, "description": "Year this annual goal targets."},
        "annual_theme": {"notion_type": "rich_text", "required": False},
        "key_result_count": {"notion_type": "number", "required": False},
    }, [], "04.2.1_Mind_Complex_Architecture.md §4 (archetype 7 Great Way of Mind)"),

    ("greatway", "Project", 14, {
        "project_status": {
            "notion_type": "status", "required": False,
            "options": ["Future", "Ideation", "Paused", "Active", "Done", "Cancelled"],
        },
        "project_health": {
            "notion_type": "select", "required": False,
            "options": ["green", "yellow", "red", "blocked"],
        },
        "progress_percent": {"notion_type": "number", "required": False},
    }, [], "04.2.2_Body_Complex_Architecture.md §4 (archetype 14 Great Way of Body)"),

    ("greatway", "Resource", 21, {
        "resource_class": {
            "notion_type": "select", "required": False,
            "options": ["tangible", "intangible", "relational", "financial", "informational"],
        },
        "resource_renewability": {
            "notion_type": "select", "required": False,
            "options": ["non_renewable", "renewable", "flow", "stock"],
        },
    }, [], "04.2.3_Spirit_Complex_Architecture.md §5 (archetype 21 Great Way of Spirit)"),

    ("greatway", "Quarterly Goal", None, {
        "year": {"notion_type": "number", "required": False},
        "quarter": {"notion_type": "select", "required": False, "options": ["Q1", "Q2", "Q3", "Q4"]},
    }, [], "Operational quarterly-goal entry-type"),

    ("greatway", "Goal", None, {
        "goal_horizon": {
            "notion_type": "select", "required": False,
            "options": ["30d", "90d", "180d", "365d"],
        },
        "goal_status": {
            "notion_type": "status", "required": False,
            "options": ["Future", "Ideation", "Paused", "Active", "Done", "Cancelled"],
        },
    }, [], "Operational goal entry-type"),

    ("greatway", "Task", None, {
        "task_status": {
            "notion_type": "status", "required": False,
            "options": ["Todo", "In Progress", "Done", "Cancelled"],
        },
        "due_date": {"notion_type": "date", "required": False},
        "priority": {
            "notion_type": "select", "required": False,
            "options": ["P0_critical", "P1_high", "P2_medium", "P3_low", "P4_backlog"],
        },
        "estimate_hours": {"notion_type": "number", "required": False},
        "actual_hours": {"notion_type": "number", "required": False},
    }, [], "Operational task entry-type"),

    ("greatway", "System", None, {
        "system_class": {
            "notion_type": "select", "required": False,
            "options": ["tooling", "process", "infrastructure", "automation", "protocol"],
        },
        "system_maturity": {
            "notion_type": "select", "required": False,
            "options": ["experimental", "pilot", "production", "deprecated"],
        },
    }, [], "Operational system entry-type"),

    ("greatway", "Sprint", None, {
        "sprint_number": {"notion_type": "number", "required": False},
        "sprint_start_date": {"notion_type": "date", "required": False},
        "sprint_end_date": {"notion_type": "date", "required": False},
        "sprint_velocity": {"notion_type": "number", "required": False},
    }, [], "Operational sprint entry-type"),

    ("greatway", "Milestone", None, {
        "milestone_target_date": {"notion_type": "date", "required": False},
        "milestone_status": {
            "notion_type": "status", "required": False,
            "options": ["Upcoming", "At Risk", "On Track", "Slipped", "Achieved", "Missed"],
        },
    }, [], "Operational milestone entry-type"),

    ("greatway", "Budget", None, {
        "budget_period": {
            "notion_type": "select", "required": False,
            "options": ["monthly", "quarterly", "annual", "multi_year"],
        },
        "budget_allocated": {"notion_type": "number", "required": False},
        "budget_spent": {"notion_type": "number", "required": False},
        "budget_remaining": {"notion_type": "number", "required": False},
    }, [], "Operational budget entry-type"),

    ("greatway", "Campaign", None, {
        "campaign_status": {
            "notion_type": "status", "required": False,
            "options": ["Drafting", "Planning", "Active", "Paused", "Completed", "Cancelled"],
        },
        "campaign_start_date": {"notion_type": "date", "required": False},
        "campaign_end_date": {"notion_type": "date", "required": False},
    }, [], "Operational campaign entry-type"),

    ("greatway", "Content", None, {
        "content_status": {
            "notion_type": "status", "required": False,
            "options": ["Idea", "Drafting", "In Review", "Published", "Archived"],
        },
        "content_type": {
            "notion_type": "select", "required": False,
            "options": ["article", "video", "podcast", "social_post", "newsletter", "course", "book"],
        },
        "publish_date": {"notion_type": "date", "required": False},
    }, [], "Operational content entry-type"),

    # ───────── GREATWAY: RELOCATED FROM SIGNIFICATOR (external holons) ─────────

    ("greatway", "Person", None, {
        "person_type": {
            "notion_type": "select", "required": False,
            "description": "Relationship type to the focal holon.",
            "options": ["self", "family", "friend", "colleague", "mentor", "mentee",
                        "collaborator", "client", "vendor", "public_figure", "ancestor"],
        },
        "bonding_disposition": {
            "notion_type": "select", "required": False,
            "description": "HoloOS doc 02.3 §4 — bonding type at Significator⇄Great-Way surface.",
            "options": ["ionic", "covalent", "dative", "metallic"],
        },
        "valence_complementarity": {
            "notion_type": "rich_text", "required": False,
            "description": "How this person's Valence Signature complements the focal Significator's.",
        },
        "contact_frequency": {
            "notion_type": "select", "required": False,
            "options": ["daily", "weekly", "monthly", "quarterly", "annual", "rare"],
        },
        "last_contact_date": {"notion_type": "date", "required": False},
    }, [
        {"id": "person_quadrant_required",
         "description": "Per HoloOS doc holon_type_placement.yaml: Person entries must have quadrant set.",
         "rule": "assert entry.quadrant in ['UL', 'UR']"},
        {"id": "person_archetype_role",
         "description": "Person entries must have archetype_role=Great Way.",
         "rule": "assert entry.archetype_role == 'Great Way'"},
    ], "RELOCATED FROM Significator (external holons belong in the Great Way per HoloOS doc 08.5)"),

    ("greatway", "Group", None, {
        "group_size": {"notion_type": "number", "required": False, "description": "Number of members in the group."},
        "group_cohesion": {
            "notion_type": "select", "required": False,
            "options": ["fragmented", "loose", "coherent", "tight", "monolithic"],
        },
        "group_bonding_type": {
            "notion_type": "select", "required": False,
            "description": "Dominant bonding type within the group (HoloOS doc 02.3 §4).",
            "options": ["ionic", "covalent", "dative", "metallic", "mixed"],
        },
        "group_purpose": {"notion_type": "rich_text", "required": False},
    }, [
        {"id": "group_quadrant_required",
         "description": "Per HoloOS doc holon_type_placement.yaml: Group entries must have quadrant=LL.",
         "rule": "assert entry.quadrant == 'LL'"},
    ], "RELOCATED FROM Significator (small collective holons)"),

    ("greatway", "Community", None, {
        "community_size": {"notion_type": "number", "required": False, "description": "Approximate community size."},
        "community_bonding_type": {
            "notion_type": "select", "required": False,
            "options": ["ionic", "covalent", "dative", "metallic", "mixed"],
        },
        "community_polarity": {
            "notion_type": "select", "required": False,
            "options": ["STO-collective", "STS-collective", "uncommitted"],
        },
        "community_culture": {"notion_type": "rich_text", "required": False, "description": "Brief description of community culture."},
        "community_stage": {
            "notion_type": "select", "required": False,
            "description": "Spiral Dynamics stage (HoloOS doc stage_codes.yaml).",
            "options": ["L1", "L2", "L3", "L4", "L5", "L6", "L7", "L8"],
        },
    }, [
        {"id": "community_quadrant_required",
         "description": "Per HoloOS doc holon_type_placement.yaml: Community entries must have quadrant=LL.",
         "rule": "assert entry.quadrant == 'LL'"},
    ], "RELOCATED FROM Significator (larger collective holons)"),

    ("greatway", "Organization", None, {
        "organization_type": {
            "notion_type": "select", "required": False,
            "options": ["corporation", "nonprofit", "government", "academic", "religious", "cooperative", "informal"],
        },
        "org_size": {"notion_type": "number", "required": False, "description": "Number of employees/members."},
        "org_founded_date": {"notion_type": "date", "required": False},
        "org_industry": {"notion_type": "select", "required": False,
            "options": ["technology", "finance", "healthcare", "education", "manufacturing",
                        "retail", "media", "government", "nonprofit", "other"]},
    }, [
        {"id": "organization_quadrant_required",
         "description": "Per HoloOS doc holon_type_placement.yaml: Organization entries must have quadrant=LR.",
         "rule": "assert entry.quadrant == 'LR'"},
    ], "NEW entry-type (formal collective holons per HoloOS doc 08.5 §1.2)"),

    ("greatway", "Network", None, {
        "network_density": {
            "notion_type": "select", "required": False,
            "options": ["sparse", "loose", "moderate", "dense", "saturated"],
        },
        "network_node_count": {"notion_type": "number", "required": False},
        "network_edge_count": {"notion_type": "number", "required": False},
        "network_purpose": {"notion_type": "rich_text", "required": False},
    }, [
        {"id": "network_quadrant_required",
         "description": "Per HoloOS doc holon_type_placement.yaml: Network entries must have quadrant=LR.",
         "rule": "assert entry.quadrant == 'LR'"},
    ], "NEW entry-type (informal collective holons)"),

    ("greatway", "Movement", None, {
        "movement_stage": {
            "notion_type": "select", "required": False,
            "options": ["emergent", "growth", "peak", "decline", "legacy"],
        },
        "movement_polarity": {
            "notion_type": "select", "required": False,
            "options": ["STO-collective", "STS-collective", "uncommitted"],
        },
        "movement_ideology": {"notion_type": "rich_text", "required": False},
        "movement_stage_code": {
            "notion_type": "select", "required": False,
            "options": ["L1", "L2", "L3", "L4", "L5", "L6", "L7", "L8"],
        },
    }, [
        {"id": "movement_quadrant_required",
         "description": "Per HoloOS doc holon_type_placement.yaml: Movement entries must have quadrant=LL.",
         "rule": "assert entry.quadrant == 'LL'"},
    ], "NEW entry-type (ideological collective holons per HoloOS doc type_codes T15)"),

    ("greatway", "Place", None, {
        "place_type": {
            "notion_type": "select", "required": False,
            "options": ["home", "workplace", "city", "region", "country", "biome", "sacred_site", "virtual"],
        },
        "place_latitude": {"notion_type": "number", "required": False},
        "place_longitude": {"notion_type": "number", "required": False},
        "place_resonance_score": {
            "notion_type": "number", "required": False,
            "description": "0-1 resonance with the focal holon's Significator.",
        },
    }, [
        {"id": "place_quadrant_required",
         "description": "Per HoloOS doc holon_type_placement.yaml: Place entries must have quadrant=LR.",
         "rule": "assert entry.quadrant == 'LR'"},
    ], "NEW entry-type (geographic operating environment per HoloOS doc type_codes T12)"),
]


def build_schema(db: str, entry_type: str, archetype_num, extra_props: dict, extra_rules: list, doc_ref: str,
                 shared_props: dict = None) -> dict:
    # Merge shared props (from MOVES) with extra_props (entry-type-specific).
    # Shared props come FIRST (they're the moved-out-from-per_db properties),
    # then extra_props (truly entry-type-specific).
    merged: dict = {}
    if shared_props:
        merged.update(shared_props)
    merged.update(extra_props)

    # Filter out any properties that are already in universal or per_db to prevent redundancy
    filtered_props = filter_redundant_props(db, merged)
    if len(filtered_props) < len(merged):
        removed = set(merged.keys()) - set(filtered_props.keys())
        print(f"  ⚠️  Filtered redundant props from {db}__{entry_type}: {removed}")

    schema = {
        "schema_version": "0.9.0",
        "schema_type": "per_entry_type",
        "applies_to_db": db,
        "applies_to_entry_type": entry_type,
        "inherits_from": [
            "universal/holon_coordinate.yaml",
            f"per_db/{db}.yaml",
        ],
        "documentation_ref": doc_ref,
        "properties": filtered_props,
    }
    if archetype_num is not None:
        schema["archetype_number"] = archetype_num
    if extra_rules:
        schema["validation_rules"] = extra_rules
    return schema


def get_shared_props(db: str, entry_type: str) -> dict:
    """Determine which shared property blocks apply to a (db, entry_type) pair.

    This mirrors the MOVES dict in fix_schema_redundancy.py — properties that
    were moved OUT of per_db into per_entry_type must be declared here so the
    generator produces complete per_entry_type files on re-run.
    """
    shared: dict = {}

    # External-holon entry-types in GreatWay get EXTERNAL_HOLON_PROPS
    external_holon_types = {"Person", "Group", "Community", "Organization", "Network", "Movement", "Place"}
    if db == "greatway" and entry_type in external_holon_types:
        shared.update(EXTERNAL_HOLON_PROPS)

    # Bonding-eligible external-holons (not Place) also get BONDING_PROPS
    bonding_types = {"Person", "Group", "Community", "Organization", "Network", "Movement"}
    if db == "greatway" and entry_type in bonding_types:
        shared.update(BONDING_PROPS)

    # Transformation-kind Nexus entry-types get TRANSFORMATION_KIND_PROPS
    transformation_kind_types = {"Pattern", "Crisis", "Transformation-Event"}
    if db == "nexus" and entry_type in transformation_kind_types:
        shared.update(TRANSFORMATION_KIND_PROPS)

    # Choice-kind Nexus entry-types get CHOICE_KIND_PROPS
    choice_kind_types = {"Directive", "Decision"}
    if db == "nexus" and entry_type in choice_kind_types:
        shared.update(CHOICE_KIND_PROPS)

    # Note/Knowledge Nexus entry-types get KNOWLEDGE_FLOW_PROPS
    knowledge_types = {"Note", "Knowledge-Category", "Knowledge-Atom"}
    if db == "nexus" and entry_type in knowledge_types:
        shared.update(KNOWLEDGE_FLOW_PROPS)

    # Principle-family Significator entry-types get PRINCIPLE_FAMILY_PROPS
    principle_family_types = {"Purpose", "Value", "Principle"}
    if db == "significator" and entry_type in principle_family_types:
        shared.update(PRINCIPLE_FAMILY_PROPS)

    return shared


def main() -> int:
    written = []
    for db, entry_type, archetype_num, extra_props, extra_rules, doc_ref in SPECIALIZATIONS:
        shared = get_shared_props(db, entry_type)
        schema = build_schema(db, entry_type, archetype_num, extra_props, extra_rules, doc_ref,
                              shared_props=shared)
        out_file = OUT_DIR / f"{db}__{slug(entry_type)}.yaml"
        with open(out_file, "w") as f:
            f.write(f"# Per-Entry-Type Schema: {db}.{entry_type}\n")
            f.write(f"# Auto-generated by scripts/upgrade_v0.9.0/generate_per_entry_type_schemas.py\n")
            f.write(f"# Documentation: {doc_ref}\n")
            f.write(f"# =============================================================================\n\n")
            yaml.safe_dump(schema, f, sort_keys=False, default_flow_style=False, allow_unicode=True, width=120)
        written.append(out_file.name)

    print(f"Wrote {len(written)} per-entry-type schema files to {OUT_DIR}")
    for name in sorted(written):
        print(f"  - {name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
