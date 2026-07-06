#!/usr/bin/env python3
"""
LifeOS Legacy Architecture — Visualizer
=========================================
Reads architecture/legacy_mapping/{systems,databases,relations}.yaml
and generates 3 visualizations:

  1. network_graph.png       — all 26 DBs as nodes, color-coded by system,
                                with hierarchy + synthesis + cross-reference edges
  2. hierarchy_tree.png      — the parent-child hierarchy chains
  3. synthesis_pipeline.png  — the upward synthesis flow (logs → 1st → 2nd pipeline)

Also generates a Mermaid diagram (mermaid_graph.mmd) for GitHub rendering.
"""
import json
import yaml
import os
import sys
from pathlib import Path
from collections import defaultdict

import matplotlib
matplotlib.use('Agg')
import matplotlib.font_manager as fm
# Try to load Noto Sans SC if available, fall back to DejaVu Sans
import os
font_paths = [
    '/usr/share/fonts/truetype/chinese/NotoSansSC-Regular.ttf',
    '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf',
    '/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf',
]
for fp in font_paths:
    if os.path.exists(fp):
        try:
            fm.fontManager.addfont(fp)
        except Exception:
            pass

import matplotlib.pyplot as plt
plt.rcParams['font.sans-serif'] = ['Noto Sans SC', 'DejaVu Sans', 'Liberation Sans']
plt.rcParams['axes.unicode_minus'] = False

import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch
import numpy as np

# ─── Load YAML files ─────────────────────────────────────────────────────────
ARCH_DIR = Path("/home/z/my-project/repos/lifeos-ops/architecture/legacy_mapping")
OUT_DIR = Path("/home/z/my-project/repos/lifeos-ops/architecture/legacy_mapping/visualizations")
OUT_DIR.mkdir(parents=True, exist_ok=True)

with open(ARCH_DIR / "systems.yaml") as f:
    systems_data = yaml.safe_load(f)
with open(ARCH_DIR / "databases.yaml") as f:
    databases_data = yaml.safe_load(f)
with open(ARCH_DIR / "relations.yaml") as f:
    relations_data = yaml.safe_load(f)

# ─── Build node + edge inventory ─────────────────────────────────────────────
# System color map (low-saturation palette per charts skill)
SYSTEM_COLORS = {
    "foundational":         "#C6866A",  # warm terracotta — the "who I am" layer
    "strategic":            "#4C6EF5",  # blue — synthesis + goals
    "execution":            "#3AAFA9",  # teal — action
    "relational":           "#8B5CF6",  # purple — people/communities
    "content_creation":     "#F59E0B",  # amber — production
    "financial_system":     "#10B981",  # green — money
    "logging_system":       "#64748B",  # slate gray — cross-cutting logs
    "temporal_aggregation": "#94A3B8",  # light gray — deprecated
}

SYSTEM_LABELS = {
    "foundational":         "Foundational (2nd Synthesis)",
    "strategic":            "Strategic (1st Synthesis + Goals)",
    "execution":            "Execution",
    "relational":           "Relational",
    "content_creation":     "Content-Creation",
    "financial_system":     "Financial-System",
    "logging_system":       "Logging-System",
    "temporal_aggregation": "Temporal (DEPRECATED)",
}

# Determine each DB's primary system (first listed)
db_primary_system = {}
db_all_systems = {}
for db_id, db_info in databases_data.items():
    if db_id in ("version",):
        continue
    systems = db_info.get("systems", [])
    db_all_systems[db_id] = systems
    db_primary_system[db_id] = systems[0] if systems else "misc"

# Collect all edges
all_edges = []
for edge in relations_data.get("hierarchy_edges", []):
    all_edges.append({**edge, "category": "hierarchy"})
for edge in relations_data.get("synthesis_edges", []):
    all_edges.append({**edge, "category": "synthesis"})
for edge in relations_data.get("temporal_edges", []):
    all_edges.append({**edge, "category": "temporal"})
for edge in relations_data.get("cross_reference_edges", []):
    all_edges.append({**edge, "category": "cross_reference"})
for edge in relations_data.get("stats_edges", []):
    all_edges.append({**edge, "category": "synthesis"})  # stats_edges are synthesis flows

# ─── Visualization 1: Network Graph (all DBs + all edges) ────────────────────
def draw_network_graph():
    """Draw all 26 DBs as nodes, color-coded by system, with all edges."""
    fig, ax = plt.subplots(figsize=(24, 18), constrained_layout=True)
    ax.set_xlim(0, 100)
    ax.set_ylim(0, 100)
    ax.axis('off')
    ax.set_facecolor('#FAFAF7')
    fig.patch.set_facecolor('#FAFAF7')

    # Position nodes by system (clustered layout)
    # Define cluster centers (x, y) for each system
    cluster_centers = {
        "foundational":         (50, 90),   # top center — the apex
        "strategic":            (50, 65),   # upper middle
        "execution":            (20, 40),   # lower left
        "relational":           (80, 40),   # lower right
        "content_creation":     (20, 15),   # bottom left
        "financial_system":     (50, 15),   # bottom center
        "logging_system":       (80, 15),   # bottom right
        "temporal_aggregation": (90, 75),   # far right (deprecated, off to the side)
    }

    # Position each DB within its cluster
    db_positions = {}
    for db_id, db_info in databases_data.items():
        if db_id == "version":
            continue
        primary_sys = db_primary_system.get(db_id, "misc")
        cx, cy = cluster_centers.get(primary_sys, (50, 50))
        # Spread nodes within cluster in a small circle
        cluster_dbs = [d for d, s in db_primary_system.items() if s == primary_sys and d != "version"]
        idx = cluster_dbs.index(db_id) if db_id in cluster_dbs else 0
        n = max(len(cluster_dbs), 1)
        angle = 2 * np.pi * idx / n
        radius = 8 if n > 1 else 0
        db_positions[db_id] = (cx + radius * np.cos(angle), cy + radius * np.sin(angle))

    # Draw edges first (so nodes overlay them)
    edge_colors = {
        "hierarchy":      "#475569",  # slate
        "synthesis":      "#C6866A",  # terracotta (upward flow)
        "temporal":       "#CBD5E1",  # light gray (deprecated)
        "cross_reference":"#94A3B8",  # medium gray
    }
    edge_styles = {
        "hierarchy":      "-",
        "synthesis":      "-",
        "temporal":       ":",
        "cross_reference":"--",
    }
    edge_widths = {
        "hierarchy":      1.5,
        "synthesis":      2.0,
        "temporal":       0.8,
        "cross_reference":1.0,
    }

    drawn_edges = set()
    for edge in all_edges:
        src, dst = edge["from"], edge["to"]
        if src not in db_positions or dst not in db_positions:
            continue
        key = (src, dst, edge["category"])
        if key in drawn_edges:
            continue
        drawn_edges.add(key)
        x1, y1 = db_positions[src]
        x2, y2 = db_positions[dst]
        cat = edge["category"]
        ax.annotate("",
                    xy=(x2, y2), xytext=(x1, y1),
                    arrowprops=dict(
                        arrowstyle="->,head_width=0.3,head_length=0.4",
                        color=edge_colors[cat],
                        lw=edge_widths[cat],
                        linestyle=edge_styles[cat],
                        alpha=0.6,
                        connectionstyle="arc3,rad=0.1",
                    ))

    # Draw nodes
    for db_id, (x, y) in db_positions.items():
        db_info = databases_data.get(db_id, {})
        primary_sys = db_primary_system.get(db_id, "misc")
        color = SYSTEM_COLORS.get(primary_sys, "#94A3B8")
        is_deprecated = db_info.get("status") == "deprecated"
        alpha = 0.4 if is_deprecated else 1.0

        # Node circle
        circle = plt.Circle((x, y), 2.5, color=color, alpha=alpha, zorder=5,
                             ec='white', lw=2)
        ax.add_patch(circle)

        # Node label
        label = db_info.get("name", db_id).replace("-and-", " &\n").replace("-", "\n")
        fontsize = 8 if is_deprecated else 9
        ax.text(x, y - 4, label, ha='center', va='top', fontsize=fontsize,
                color='#243447', fontweight='bold' if not is_deprecated else 'normal',
                alpha=alpha, zorder=6)

    # Draw cluster labels
    for sys_id, (cx, cy) in cluster_centers.items():
        label = SYSTEM_LABELS.get(sys_id, sys_id)
        ax.text(cx, cy + 12, label, ha='center', va='bottom', fontsize=11,
                color=SYSTEM_COLORS.get(sys_id, '#243447'),
                fontweight='bold', alpha=0.8,
                bbox=dict(boxstyle='round,pad=0.3', facecolor='white',
                          edgecolor=SYSTEM_COLORS.get(sys_id, '#243447'),
                          alpha=0.7, lw=1.5))

    # Title
    ax.text(50, 98, "LifeOS Legacy Architecture — DB Network Graph",
            ha='center', va='top', fontsize=20, fontweight='bold', color='#243447')
    ax.text(50, 95.5, "26 DBs across 7 functional systems + Logging-System. Edges: hierarchy, synthesis flow, cross-reference, temporal (deprecated).",
            ha='center', va='top', fontsize=11, color='#64748B', style='italic')

    # Legend
    legend_items = [
        mpatches.Patch(color=SYSTEM_COLORS["foundational"], label=SYSTEM_LABELS["foundational"]),
        mpatches.Patch(color=SYSTEM_COLORS["strategic"], label=SYSTEM_LABELS["strategic"]),
        mpatches.Patch(color=SYSTEM_COLORS["execution"], label=SYSTEM_LABELS["execution"]),
        mpatches.Patch(color=SYSTEM_COLORS["relational"], label=SYSTEM_LABELS["relational"]),
        mpatches.Patch(color=SYSTEM_COLORS["content_creation"], label=SYSTEM_LABELS["content_creation"]),
        mpatches.Patch(color=SYSTEM_COLORS["financial_system"], label=SYSTEM_LABELS["financial_system"]),
        mpatches.Patch(color=SYSTEM_COLORS["logging_system"], label=SYSTEM_LABELS["logging_system"]),
        mpatches.Patch(color=SYSTEM_COLORS["temporal_aggregation"], label=SYSTEM_LABELS["temporal_aggregation"]),
    ]
    leg1 = ax.legend(handles=legend_items, loc='upper left', bbox_to_anchor=(0.0, 0.98),
                     fontsize=9, title="Systems", title_fontsize=10, framealpha=0.9)
    ax.add_artist(leg1)

    # Edge type legend
    edge_legend_items = [
        plt.Line2D([0], [0], color=edge_colors["hierarchy"], lw=edge_widths["hierarchy"],
                   label='Hierarchy (parent → child)', linestyle=edge_styles["hierarchy"]),
        plt.Line2D([0], [0], color=edge_colors["synthesis"], lw=edge_widths["synthesis"],
                   label='Synthesis flow (upward)', linestyle=edge_styles["synthesis"]),
        plt.Line2D([0], [0], color=edge_colors["cross_reference"], lw=edge_widths["cross_reference"],
                   label='Cross-reference', linestyle=edge_styles["cross_reference"]),
        plt.Line2D([0], [0], color=edge_colors["temporal"], lw=edge_widths["temporal"],
                   label='Temporal (DEPRECATED)', linestyle=edge_styles["temporal"]),
    ]
    ax.legend(handles=edge_legend_items, loc='lower right', bbox_to_anchor=(1.0, 0.02),
              fontsize=9, title="Edge types", title_fontsize=10, framealpha=0.9)

    plt.savefig(OUT_DIR / "network_graph.png", dpi=200, bbox_inches='tight',
                facecolor='#FAFAF7')
    plt.close()
    print(f"  ✓ {OUT_DIR / 'network_graph.png'}")


# ─── Visualization 2: Hierarchy Tree ─────────────────────────────────────────
def draw_hierarchy_tree():
    """Draw the parent-child hierarchy chains as a tree."""
    fig, ax = plt.subplots(figsize=(22, 14), constrained_layout=True)
    ax.set_xlim(0, 100)
    ax.set_ylim(0, 100)
    ax.axis('off')
    ax.set_facecolor('#FAFAF7')
    fig.patch.set_facecolor('#FAFAF7')

    # Define hierarchy layers (top to bottom)
    # Layer 0: Foundational (Values/Principles, Vision)
    # Layer 1: Strategic goals (Annual-Goals) + Stats
    # Layer 2: Quarterly-Goals + 1st synthesis pipeline
    # Layer 3: Projects
    # Layer 4: Tasks + Campaigns + Financial-Accounts
    # Layer 5: Logs (Activity, Diet, Financial, Subjective, Relational, Systemic)
    # Layer 6: Temporal (DEPRECATED — Days/Weeks/Months/Quarters/Years)

    layers = {
        0: [("values_and_principles", "Values &\nPrinciples"),
            ("vision", "Vision"),
            ("stats", "Stats")],
        1: [("annual_goals", "Annual-Goals")],
        2: [("quarterly_goals", "Quarterly-Goals"),
            ("opportunities_and_strengths", "Opportunities &\nStrengths"),
            ("directives_and_risks", "Directives &\nRisks"),
            ("notes_management", "Notes-\nManagement")],
        3: [("projects", "Projects")],
        4: [("tasks", "Tasks"),
            ("campaigns", "Campaigns"),
            ("financial_accounts", "Financial-\nAccounts"),
            ("communities", "Communities")],
        5: [("activity_log", "Activity-Log"),
            ("diet_log", "Diet-Log"),
            ("financial_log", "Financial-Log"),
            ("subjective_journal", "Subjective-\nJournal"),
            ("relational_journal", "Relational-\nJournal"),
            ("systemic_journal", "Systemic-\nJournal"),
            ("content_pipeline", "Content-\nPipeline"),
            ("people", "People")],
        6: [("days", "Days"), ("weeks", "Weeks"), ("months", "Months"),
            ("quarters", "Quarters"), ("years", "Years")],
    }

    layer_y = {0: 90, 1: 75, 2: 60, 3: 45, 4: 32, 5: 18, 6: 6}
    layer_labels = {
        0: "Layer 0 — Foundational (2nd synthesis)",
        1: "Layer 1 — Annual Goals",
        2: "Layer 2 — Quarterly Goals + 1st Synthesis Pipeline",
        3: "Layer 3 — Projects",
        4: "Layer 4 — Tasks + Campaigns + Accounts + Communities",
        5: "Layer 5 — Logs + Content + People",
        6: "Layer 6 — Temporal (DEPRECATED)",
    }

    # Position nodes
    node_pos = {}
    for layer_idx, nodes in layers.items():
        n = len(nodes)
        spacing = 80 / max(n, 1)
        for i, (db_id, label) in enumerate(nodes):
            x = 10 + spacing * (i + 0.5)
            y = layer_y[layer_idx]
            node_pos[db_id] = (x, y, label)

    # Draw layer labels (left side)
    for layer_idx, label in layer_labels.items():
        y = layer_y[layer_idx]
        ax.text(2, y, label, ha='left', va='center', fontsize=9,
                color='#64748B', fontweight='bold', style='italic')

    # Draw edges (hierarchy only)
    hierarchy_edges = relations_data.get("hierarchy_edges", [])
    drawn = set()
    for edge in hierarchy_edges:
        src, dst = edge["from"], edge["to"]
        if src not in node_pos or dst not in node_pos:
            continue
        key = (src, dst)
        if key in drawn:
            continue
        drawn.add(key)
        x1, y1, _ = node_pos[src]
        x2, y2, _ = node_pos[dst]
        ax.annotate("",
                    xy=(x2, y2 + 2.5), xytext=(x1, y1 - 2.5),
                    arrowprops=dict(
                        arrowstyle="->,head_width=0.4,head_length=0.5",
                        color='#475569', lw=1.5, alpha=0.7,
                        connectionstyle="arc3,rad=0.0",
                    ))

    # Draw synthesis edges (terracotta)
    synthesis_edges = relations_data.get("synthesis_edges", []) + relations_data.get("stats_edges", [])
    drawn_syn = set()
    for edge in synthesis_edges:
        src, dst = edge["from"], edge["to"]
        if src not in node_pos or dst not in node_pos:
            continue
        key = (src, dst)
        if key in drawn_syn:
            continue
        drawn_syn.add(key)
        x1, y1, _ = node_pos[src]
        x2, y2, _ = node_pos[dst]
        ax.annotate("",
                    xy=(x2, y2 + 2.5), xytext=(x1, y1 - 2.5),
                    arrowprops=dict(
                        arrowstyle="->,head_width=0.3,head_length=0.4",
                        color='#C6866A', lw=1.2, alpha=0.5,
                        linestyle='-',
                        connectionstyle="arc3,rad=0.2",
                    ))

    # Draw nodes
    for db_id, (x, y, label) in node_pos.items():
        db_info = databases_data.get(db_id, {})
        primary_sys = db_primary_system.get(db_id, "misc")
        color = SYSTEM_COLORS.get(primary_sys, "#94A3B8")
        is_deprecated = db_info.get("status") == "deprecated"
        alpha = 0.4 if is_deprecated else 1.0

        box = FancyBboxPatch((x - 4, y - 2), 8, 4,
                              boxstyle="round,pad=0.2",
                              facecolor=color, edgecolor='white', lw=1.5,
                              alpha=alpha, zorder=5)
        ax.add_patch(box)
        ax.text(x, y, label, ha='center', va='center', fontsize=8,
                color='white', fontweight='bold', alpha=alpha, zorder=6)

    # Title
    ax.text(50, 98, "LifeOS Legacy Architecture — Hierarchy Tree",
            ha='center', va='top', fontsize=20, fontweight='bold', color='#243447')
    ax.text(50, 95.5, "Parent-child chains (slate) + upward synthesis flow (terracotta). 7 layers from Foundational to Temporal.",
            ha='center', va='top', fontsize=11, color='#64748B', style='italic')

    # Legend
    legend_items = [
        plt.Line2D([0], [0], color='#475569', lw=1.5, label='Hierarchy (parent → child)'),
        plt.Line2D([0], [0], color='#C6866A', lw=1.2, label='Synthesis flow (upward)', alpha=0.7),
    ]
    ax.legend(handles=legend_items, loc='lower right', fontsize=10, framealpha=0.9)

    plt.savefig(OUT_DIR / "hierarchy_tree.png", dpi=200, bbox_inches='tight',
                facecolor='#FAFAF7')
    plt.close()
    print(f"  ✓ {OUT_DIR / 'hierarchy_tree.png'}")


# ─── Visualization 3: Synthesis Pipeline Flow ────────────────────────────────
def draw_synthesis_pipeline():
    """Draw the upward synthesis flow: Logs → 1st pipeline → 2nd pipeline."""
    fig, ax = plt.subplots(figsize=(18, 12), constrained_layout=True)
    ax.set_xlim(0, 100)
    ax.set_ylim(0, 100)
    ax.axis('off')
    ax.set_facecolor('#FAFAF7')
    fig.patch.set_facecolor('#FAFAF7')

    # Three columns: Logs (left), 1st synthesis (middle), 2nd synthesis (right)
    col_x = {"logs": 15, "first": 50, "second": 85}
    col_labels = {
        "logs": "LOGGING-SYSTEM\n(Objective ground-reality data)",
        "first": "1ST SYNTHESIS PIPELINE\n(Logs → Insights)",
        "second": "2ND SYNTHESIS PIPELINE\n(Insights → Traits)",
    }

    # Draw column headers
    for col, x in col_x.items():
        ax.text(x, 92, col_labels[col], ha='center', va='center', fontsize=12,
                fontweight='bold', color='#243447',
                bbox=dict(boxstyle='round,pad=0.5', facecolor='white',
                          edgecolor='#475569', lw=2))

    # Logs (left column)
    logs = [
        ("activity_log", "Activity-Log"),
        ("diet_log", "Diet-Log"),
        ("financial_log", "Financial-Log"),
        ("subjective_journal", "Subjective-Journal"),
        ("relational_journal", "Relational-Journal"),
        ("systemic_journal", "Systemic-Journal"),
    ]
    log_y_positions = np.linspace(75, 15, len(logs))
    for (db_id, label), y in zip(logs, log_y_positions):
        box = FancyBboxPatch((col_x["logs"] - 8, y - 2.5), 16, 5,
                              boxstyle="round,pad=0.2",
                              facecolor=SYSTEM_COLORS["logging_system"],
                              edgecolor='white', lw=1.5, alpha=0.85)
        ax.add_patch(box)
        ax.text(col_x["logs"], y, label, ha='center', va='center',
                fontsize=9, color='white', fontweight='bold')

    # 1st synthesis (middle column)
    first_pipeline = [
        ("notes_management", "Notes-Management\n(raw synthesis)"),
        ("opportunities_and_strengths", "Opportunities &\nStrengths\n(positive trends)"),
        ("directives_and_risks", "Directives &\nRisks\n(negative trends +\ncorrective actions)"),
    ]
    first_y_positions = [65, 40, 15]
    for (db_id, label), y in zip(first_pipeline, first_y_positions):
        box = FancyBboxPatch((col_x["first"] - 9, y - 4), 18, 8,
                              boxstyle="round,pad=0.2",
                              facecolor=SYSTEM_COLORS["strategic"],
                              edgecolor='white', lw=1.5, alpha=0.85)
        ax.add_patch(box)
        ax.text(col_x["first"], y, label, ha='center', va='center',
                fontsize=9, color='white', fontweight='bold')

    # 2nd synthesis (right column)
    second_pipeline = [
        ("stats", "Stats\n(RPG status report\nacross all dimensions)"),
        ("values_and_principles", "Values &\nPrinciples\n(enduring\ncommitments)"),
        ("vision", "Vision\n(long-term\ndirection)"),
    ]
    second_y_positions = [70, 40, 15]
    for (db_id, label), y in zip(second_pipeline, second_y_positions):
        box = FancyBboxPatch((col_x["second"] - 9, y - 5), 18, 10,
                              boxstyle="round,pad=0.2",
                              facecolor=SYSTEM_COLORS["foundational"],
                              edgecolor='white', lw=1.5, alpha=0.85)
        ax.add_patch(box)
        ax.text(col_x["second"], y, label, ha='center', va='center',
                fontsize=9, color='white', fontweight='bold')

    # Draw synthesis edges (logs → 1st pipeline)
    log_to_first = {
        "activity_log": ["notes_management", "opportunities_and_strengths", "directives_and_risks"],
        "diet_log": ["opportunities_and_strengths", "directives_and_risks"],
        "financial_log": ["opportunities_and_strengths", "directives_and_risks"],
        "subjective_journal": ["notes_management", "opportunities_and_strengths", "directives_and_risks"],
        "relational_journal": ["opportunities_and_strengths", "directives_and_risks"],
        "systemic_journal": ["notes_management", "opportunities_and_strengths", "directives_and_risks"],
    }

    for log_id, targets in log_to_first.items():
        log_y = log_y_positions[logs.index((log_id, dict(logs)[log_id]))]
        for target_id in targets:
            target_y = first_y_positions[[t[0] for t in first_pipeline].index(target_id)]
            ax.annotate("",
                        xy=(col_x["first"] - 9, target_y),
                        xytext=(col_x["logs"] + 8, log_y),
                        arrowprops=dict(
                            arrowstyle="->,head_width=0.3,head_length=0.4",
                            color='#C6866A', lw=0.8, alpha=0.4,
                            connectionstyle="arc3,rad=0.1",
                        ))

    # Draw synthesis edges (1st pipeline → 2nd pipeline)
    first_to_second = {
        "opportunities_and_strengths": ["stats", "values_and_principles"],
        "directives_and_risks": ["stats", "values_and_principles"],
        "notes_management": ["stats"],
    }
    for src_id, targets in first_to_second.items():
        src_y = first_y_positions[[t[0] for t in first_pipeline].index(src_id)]
        for target_id in targets:
            target_y = second_y_positions[[t[0] for t in second_pipeline].index(target_id)]
            ax.annotate("",
                        xy=(col_x["second"] - 9, target_y),
                        xytext=(col_x["first"] + 9, src_y),
                        arrowprops=dict(
                            arrowstyle="->,head_width=0.4,head_length=0.5",
                            color='#C6866A', lw=1.5, alpha=0.7,
                            connectionstyle="arc3,rad=0.1",
                        ))

    # Downward causation arrow (Directives → Execution)
    ax.annotate("",
                xy=(50, 5), xytext=(50, 11),
                arrowprops=dict(
                    arrowstyle="->,head_width=0.5,head_length=0.6",
                    color='#475569', lw=2, alpha=0.8,
                ))
    ax.text(50, 3, "↓  Downward causation: Directives spawn corrective Projects/Tasks in the Execution-Pipeline",
            ha='center', va='center', fontsize=9, color='#475569', style='italic')

    # Title
    ax.text(50, 98, "LifeOS Legacy — Synthesis Pipeline (Upward Flow)",
            ha='center', va='top', fontsize=20, fontweight='bold', color='#243447')
    ax.text(50, 95.5, "Objective logs → 1st synthesis (insights) → 2nd synthesis (traits). Synthesis flows UPWARD only.",
            ha='center', va='top', fontsize=11, color='#64748B', style='italic')

    plt.savefig(OUT_DIR / "synthesis_pipeline.png", dpi=200, bbox_inches='tight',
                facecolor='#FAFAF7')
    plt.close()
    print(f"  ✓ {OUT_DIR / 'synthesis_pipeline.png'}")


# ─── Visualization 4: Mermaid diagram (for GitHub rendering) ─────────────────
def write_mermaid():
    """Write a Mermaid diagram for GitHub rendering."""
    out = ["```mermaid"]
    out.append("graph TD")
    out.append("    %% LifeOS Legacy Architecture — Hierarchy + Synthesis Flow")

    # Define subgraphs by system
    out.append("    subgraph Foundational['Foundational — 2nd Synthesis Pipeline']")
    for db_id in ["values_and_principles", "vision", "stats"]:
        label = databases_data[db_id]["name"]
        out.append(f"        {db_id}[\"{label}\"]")
    out.append("    end")

    out.append("    subgraph Strategic['Strategic — 1st Synthesis + Goals']")
    for db_id in ["annual_goals", "quarterly_goals", "opportunities_and_strengths", "directives_and_risks", "notes_management"]:
        label = databases_data[db_id]["name"]
        out.append(f"        {db_id}[\"{label}\"]")
    out.append("    end")

    out.append("    subgraph Execution['Execution']")
    for db_id in ["projects", "tasks", "systemic_journal", "activity_log", "diet_log"]:
        label = databases_data[db_id]["name"]
        out.append(f"        {db_id}[\"{label}\"]")
    out.append("    end")

    out.append("    subgraph Relational['Relational']")
    for db_id in ["people", "communities", "relational_journal"]:
        label = databases_data[db_id]["name"]
        out.append(f"        {db_id}[\"{label}\"]")
    out.append("    end")

    out.append("    subgraph Content['Content-Creation']")
    for db_id in ["campaigns", "content_pipeline"]:
        label = databases_data[db_id]["name"]
        out.append(f"        {db_id}[\"{label}\"]")
    out.append("    end")

    out.append("    subgraph Financial['Financial-System']")
    for db_id in ["financial_accounts", "financial_log"]:
        label = databases_data[db_id]["name"]
        out.append(f"        {db_id}[\"{label}\"]")
    out.append("    end")

    out.append("    subgraph Subjective['Logging-System (cross-cutting)']")
    out.append("        subjective_journal[\"Subjective-Journal\"]")
    out.append("    end")

    # Hierarchy edges
    out.append("    %% Hierarchy edges (parent → child)")
    for edge in relations_data.get("hierarchy_edges", []):
        out.append(f"    {edge['from']} --> {edge['to']}")

    # Synthesis edges (dotted)
    out.append("    %% Synthesis flow (upward, dotted)")
    for edge in relations_data.get("synthesis_edges", []) + relations_data.get("stats_edges", []):
        out.append(f"    {edge['from']} -.->|synthesis| {edge['to']}")

    out.append("```")

    mermaid_path = OUT_DIR / "mermaid_graph.mmd"
    with open(mermaid_path, "w") as f:
        f.write("\n".join(out))
    print(f"  ✓ {mermaid_path}")

    # Also write a markdown file with the mermaid embedded
    md_path = OUT_DIR / "mermaid_graph.md"
    with open(md_path, "w") as f:
        f.write("# LifeOS Legacy Architecture — Mermaid Diagram\n\n")
        f.write("\n".join(out))
        f.write("\n\n## Legend\n\n")
        f.write("- `-->` = hierarchy (parent → child)\n")
        f.write("- `-.->` = synthesis flow (upward)\n")
        f.write("- Subgraphs = the 7 functional systems + Logging-System\n")
    print(f"  ✓ {md_path}")


# ─── Main ────────────────────────────────────────────────────────────────────
def main():
    print("Generating LifeOS legacy architecture visualizations…")
    draw_network_graph()
    draw_hierarchy_tree()
    draw_synthesis_pipeline()
    write_mermaid()
    print(f"\nAll visualizations saved to: {OUT_DIR}")

if __name__ == "__main__":
    main()
