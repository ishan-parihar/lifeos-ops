#!/usr/bin/env python3
"""
LifeOS Blueprint v4 — The 7-DB Structure Visualization
========================================================
Shows:
  - 3 functional layers (Teleological Pull / Historical Record / Action Interface)
  - 7 DBs with their entry-types
  - The 3 flows: Teleological Pull (downward), Ground-Truth (upward), Feedback (loop)
  - The morning view slice

Layout: 3 horizontal layers, with DBs in each layer, and flow arrows connecting them.
"""
import matplotlib
matplotlib.use('Agg')
import matplotlib.font_manager as fm
import os
font_paths = [
    '/usr/share/fonts/truetype/chinese/NotoSansSC-Regular.ttf',
    '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf',
    '/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf',
]
for fp in font_paths:
    if os.path.exists(fp):
        try: fm.fontManager.addfont(fp)
        except: pass

import matplotlib.pyplot as plt
plt.rcParams['font.sans-serif'] = ['Noto Sans SC', 'DejaVu Sans', 'Liberation Sans']
plt.rcParams['axes.unicode_minus'] = False
from matplotlib.patches import FancyBboxPatch
from pathlib import Path

OUT_DIR = Path("/home/z/my-project/repos/lifeos-ops/architecture/legacy_mapping/visualizations")

C = {
    'bg':           '#FAFAF7',
    'layer_a':      '#FEF3C7',  # gold — teleological pull
    'layer_a_edge': '#D97706',
    'layer_b':      '#EFF6FF',  # ice blue — historical record
    'layer_b_edge': '#3B82F6',
    'layer_c':      '#D1FAE5',  # mint — action interface
    'layer_c_edge': '#10B981',
    'vision':       '#FEF3C7',
    'compass':      '#FED7AA',
    'logbook':      '#DBEAFE',
    'synthesis':    '#BFDBFE',
    'profile':      '#93C5FD',
    'action':       '#A7F3D0',
    'context':      '#6EE7B7',
    'text':         '#243447',
    'text_muted':   '#64748B',
    'pull':         '#D97706',  # gold — teleological pull
    'ground':       '#3B82F6',  # blue — ground-truth flow
    'feedback':     '#8B5CF6',  # purple — feedback loop
    'positive':     '#10B981',
    'negative':     '#EF4444',
}

def box(ax, x, y, w, h, fc, ec, lw=1.5, alpha=0.85, radius=0.15, z=2):
    ax.add_patch(FancyBboxPatch((x, y), w, h,
        boxstyle=f"round,pad=0.02,rounding_size={radius}",
        facecolor=fc, edgecolor=ec, lw=lw, alpha=alpha, zorder=z))

def label(ax, x, y, text, fs=9, color='#243447', weight='bold', ha='center', va='center', style='normal', z=5, alpha=None, family=None):
    kwargs = dict(ha=ha, va=va, fontsize=fs, color=color, fontweight=weight, style=style, zorder=z)
    if alpha is not None: kwargs['alpha'] = alpha
    if family is not None: kwargs['family'] = family
    ax.text(x, y, text, **kwargs)

def leaf(ax, x, y, text, fs=7, color=None, z=5):
    c = color or C['text']
    ax.text(x, y, '• ' + text, ha='left', va='center', fontsize=fs, color=c, zorder=z, family='monospace')

def arrow(ax, x1, y1, x2, y2, color='#475569', lw=1.5, style='->', rad=0.0, alpha=0.7, z=3):
    ax.annotate('', xy=(x2, y2), xytext=(x1, y1),
                arrowprops=dict(arrowstyle=f'{style},head_width=0.4,head_length=0.5',
                                color=color, lw=lw, alpha=alpha,
                                connectionstyle=f'arc3,rad={rad}'),
                zorder=z)


def main():
    fig, ax = plt.subplots(figsize=(24, 20), constrained_layout=False)
    ax.set_xlim(0, 100)
    ax.set_ylim(0, 100)
    ax.axis('off')
    ax.set_facecolor(C['bg'])
    fig.patch.set_facecolor(C['bg'])

    # ─── Title ───────────────────────────────────────────────────────────
    label(ax, 50, 97, 'LifeOS Blueprint v4 — The 7-DB Notion System Structure',
          fs=18, weight='bold')
    label(ax, 50, 94,
          '7 DBs across 3 functional layers. 3 flows: Teleological Pull (downward) · Ground-Truth (upward) · Feedback (loop).',
          fs=10, color=C['text_muted'], style='italic')

    # ─── Layer backgrounds ───────────────────────────────────────────────
    # Layer A (top) — Teleological Pull
    box(ax, 2, 72, 96, 18, C['layer_a'], C['layer_a_edge'], lw=2, alpha=0.25, radius=0.3, z=1)
    label(ax, 50, 88, 'LAYER A — TELEOLOGICAL PULL  (articulate + simulate the drive toward the ideal-future)',
          fs=11, weight='bold', color=C['layer_a_edge'], z=2)

    # Layer C (middle) — Action Interface
    box(ax, 2, 42, 96, 26, C['layer_c'], C['layer_c_edge'], lw=2, alpha=0.25, radius=0.3, z=1)
    label(ax, 50, 65, 'LAYER C — ACTION INTERFACE  (tell the user what to do + how)',
          fs=11, weight='bold', color=C['layer_c_edge'], z=2)

    # Layer B (bottom) — Historical Record
    box(ax, 2, 4, 96, 34, C['layer_b'], C['layer_b_edge'], lw=2, alpha=0.25, radius=0.3, z=1)
    label(ax, 50, 35, 'LAYER B — HISTORICAL RECORD + CURRENT TRAJECTORY  (keep tabs on history + trends)',
          fs=11, weight='bold', color=C['layer_b_edge'], z=2)

    # ─── Layer A DBs (2) ─────────────────────────────────────────────────
    # DB 1: Vision
    box(ax, 8, 74, 38, 12, C['vision'], C['layer_a_edge'], lw=2, alpha=0.7, radius=0.2, z=3)
    label(ax, 27, 84, 'DB 1: VISION', fs=11, weight='bold', color=C['layer_a_edge'], z=4)
    label(ax, 27, 81.5, '(the ideal-future articulation)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 10, 79, 'Purpose — the deepest why', fs=7, z=4)
    leaf(ax, 10, 77.5, 'Value — enduring commitments', fs=7, z=4)
    leaf(ax, 10, 76, 'Principle — decision rules', fs=7, z=4)
    leaf(ax, 30, 79, 'Vision-Statement — ideal-future', fs=7, z=4)
    leaf(ax, 30, 77.5, 'Identity-Statement — who I become', fs=7, z=4)

    # DB 2: Compass
    box(ax, 54, 74, 38, 12, C['compass'], C['layer_a_edge'], lw=2, alpha=0.7, radius=0.2, z=3)
    label(ax, 73, 84, 'DB 2: COMPASS', fs=11, weight='bold', color=C['layer_a_edge'], z=4)
    label(ax, 73, 81.5, '(time-bound trajectory decomposition)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 56, 79, 'Annual-Goal — yearly target', fs=7, z=4)
    leaf(ax, 56, 77.5, 'Quarterly-Goal — quarterly decomposition', fs=7, z=4)
    leaf(ax, 56, 76, 'Milestone — event-bound checkpoint', fs=7, z=4)

    # Vision → Compass arrow (decomposes into)
    arrow(ax, 46, 80, 54, 80, color=C['layer_a_edge'], lw=2, alpha=0.8, z=4)
    label(ax, 50, 81, 'decomposes', fs=7, color=C['layer_a_edge'], style='italic', z=4)

    # ─── Layer C DBs (2) ─────────────────────────────────────────────────
    # DB 6: Action
    box(ax, 8, 44, 48, 18, C['action'], C['layer_c_edge'], lw=2, alpha=0.7, radius=0.2, z=3)
    label(ax, 32, 59, 'DB 6: ACTION', fs=11, weight='bold', color=C['layer_c_edge'], z=4)
    label(ax, 32, 56.5, '(the actionable hierarchy — what to do + how)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 10, 54, 'Project — multi-step deliverable', fs=7, z=4)
    leaf(ax, 10, 52.5, 'Task — atomic unit of work', fs=7, z=4)
    leaf(ax, 10, 51, 'Campaign — coordinated content effort', fs=7, z=4)
    leaf(ax, 10, 49.5, 'Content — single content piece', fs=7, z=4)
    leaf(ax, 35, 54, 'Hierarchy: Project → Task', fs=7, z=4)
    leaf(ax, 35, 52.5, 'Traces to: Compass (Goal)', fs=7, z=4)
    leaf(ax, 35, 51, 'Generates: Logbook entries', fs=7, z=4)
    leaf(ax, 35, 49.5, 'Involves: Context (People)', fs=7, z=4)

    # DB 7: Context
    box(ax, 60, 44, 32, 18, C['context'], C['layer_c_edge'], lw=2, alpha=0.7, radius=0.2, z=3)
    label(ax, 76, 59, 'DB 7: CONTEXT', fs=11, weight='bold', color=C['layer_c_edge'], z=4)
    label(ax, 76, 56.5, '(the environment)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 62, 54, 'Person (rich CRM, 14 props)', fs=7, z=4)
    leaf(ax, 62, 52.5, 'Community', fs=7, z=4)
    leaf(ax, 62, 51, 'Organization', fs=7, z=4)
    leaf(ax, 62, 49.5, 'Financial-Account', fs=7, z=4)
    leaf(ax, 62, 48, 'Place', fs=7, z=4)

    # Action ↔ Context arrow (involves)
    arrow(ax, 56, 53, 60, 53, color=C['layer_c_edge'], lw=1.5, alpha=0.7, style='<->', z=4)
    label(ax, 58, 54, 'involves', fs=6.5, color=C['layer_c_edge'], style='italic', z=4)

    # ─── Layer B DBs (3) ─────────────────────────────────────────────────
    # DB 3: Logbook
    box(ax, 8, 22, 26, 10, C['logbook'], C['layer_b_edge'], lw=2, alpha=0.7, radius=0.2, z=3)
    label(ax, 21, 30, 'DB 3: LOGBOOK', fs=10, weight='bold', color=C['layer_b_edge'], z=4)
    label(ax, 21, 27.5, '(6 logs, 1 DB)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 10, 25.5, 'Activity · Diet · Financial', fs=6.5, z=4)
    leaf(ax, 10, 24, 'Subjective · Relational · Systemic', fs=6.5, z=4)

    # DB 4: Synthesis
    box(ax, 37, 22, 26, 10, C['synthesis'], C['layer_b_edge'], lw=2, alpha=0.7, radius=0.2, z=3)
    label(ax, 50, 30, 'DB 4: SYNTHESIS', fs=10, weight='bold', color=C['layer_b_edge'], z=4)
    label(ax, 50, 27.5, '(polar pair + raw)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 39, 25.5, 'Note (neutral)', fs=6.5, z=4)
    leaf(ax, 39, 24, 'Opportunity/Strength (+)', fs=6.5, color=C['positive'], z=4)
    leaf(ax, 39, 22.5, 'Directive/Risk (−)', fs=6.5, color=C['negative'], z=4)

    # DB 5: Profile
    box(ax, 66, 22, 26, 10, C['profile'], C['layer_b_edge'], lw=2, alpha=0.7, radius=0.2, z=3)
    label(ax, 79, 30, 'DB 5: PROFILE', fs=10, weight='bold', color=C['layer_b_edge'], z=4)
    label(ax, 79, 27.5, '(cumulative state mirror)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 68, 25.5, 'Trait · Metric · Capacity', fs=6.5, z=4)
    leaf(ax, 68, 24, 'Asset', fs=6.5, z=4)
    label(ax, 68, 22.5, '(the RPG status report)', fs=6.5, color=C['text_muted'], style='italic', ha='left', z=4)

    # Logbook → Synthesis → Profile arrows (ground-truth flow)
    arrow(ax, 34, 27, 37, 27, color=C['layer_b_edge'], lw=2, alpha=0.8, z=4)
    label(ax, 35.5, 28, 'synthesizes', fs=6.5, color=C['layer_b_edge'], style='italic', z=4)

    arrow(ax, 63, 27, 66, 27, color=C['layer_b_edge'], lw=2, alpha=0.8, z=4)
    label(ax, 64.5, 28, 'condenses', fs=6.5, color=C['layer_b_edge'], style='italic', z=4)

    # ─── The 3 Flows (layer-spanning arrows) ─────────────────────────────
    # Flow 1: Teleological Pull (Layer A → Layer C) — downward, gold
    arrow(ax, 73, 74, 50, 62, color=C['pull'], lw=3, alpha=0.7, rad=0.1, z=2)
    label(ax, 64, 70, 'FLOW 1: TELEOLOGICAL PULL', fs=8, weight='bold', color=C['pull'], z=3)
    label(ax, 64, 68, '(Vision → Compass → Action)', fs=7, color=C['pull'], style='italic', z=3)

    # Flow 2: Ground-Truth (Layer C → Layer B) — downward, blue
    arrow(ax, 32, 44, 21, 32, color=C['ground'], lw=3, alpha=0.7, rad=0.1, z=2)
    label(ax, 22, 40, 'FLOW 2: GROUND-TRUTH', fs=8, weight='bold', color=C['ground'], z=3)
    label(ax, 22, 38, '(Action → Logbook →', fs=7, color=C['ground'], style='italic', z=3)
    label(ax, 22, 36.5, ' Synthesis → Profile)', fs=7, color=C['ground'], style='italic', z=3)

    # Flow 3: Feedback (Layer B → Layer A) — upward, purple (the loop closing)
    arrow(ax, 79, 32, 27, 74, color=C['feedback'], lw=3, alpha=0.6, rad=-0.3, z=2)
    label(ax, 88, 55, 'FLOW 3: FEEDBACK', fs=8, weight='bold', color=C['feedback'], z=3)
    label(ax, 88, 53, '(Profile → Vision)', fs=7, color=C['feedback'], style='italic', z=3)
    label(ax, 88, 51.5, '(gap informs pull)', fs=7, color=C['feedback'], style='italic', z=3)

    # ─── Bottom: The cycle summary ───────────────────────────────────────
    box(ax, 8, 6, 84, 12, 'white', C['text_muted'], lw=1.5, alpha=0.7, radius=0.2, z=2)
    label(ax, 50, 16, 'THE CYCLE (causal amplification)',
          fs=10, weight='bold', color=C['text'], z=3)
    label(ax, 50, 13.5,
          'Vision → Compass → Action → Logbook → Synthesis → Profile → Vision',
          fs=9, color=C['text'], z=3, family='monospace')
    label(ax, 50, 11,
          'Each loop: pull becomes more precise · actions become more aligned · history becomes richer · self-understanding deepens',
          fs=8, color=C['text_muted'], style='italic', z=3)
    label(ax, 50, 8.5,
          'MORNING VIEW = panoramic slice across all 3 layers (pull + gap + trends + actions + accomplishments + capture)',
          fs=8, color=C['text_muted'], style='italic', weight='bold', z=3)

    # ─── Legend ──────────────────────────────────────────────────────────
    legend_items = [
        (C['layer_a'], C['layer_a_edge'], 'Layer A: Teleological Pull'),
        (C['layer_c'], C['layer_c_edge'], 'Layer C: Action Interface'),
        (C['layer_b'], C['layer_b_edge'], 'Layer B: Historical Record'),
    ]
    for i, (fc, ec, name) in enumerate(legend_items):
        x = 8 + i * 28
        box(ax, x, 1, 1.5, 1.5, fc, ec, lw=1.5, alpha=0.8, radius=0.1, z=5)
        label(ax, x + 2.5, 1.75, name, fs=7.5, ha='left', z=5)

    plt.savefig(OUT_DIR / "blueprint_v4_7_db_structure.png", dpi=180,
                bbox_inches='tight', facecolor=C['bg'])
    plt.close()
    print(f"  ✓ {OUT_DIR / 'blueprint_v4_7_db_structure.png'}")

if __name__ == "__main__":
    main()
