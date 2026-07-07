#!/usr/bin/env python3
"""
LifeOS Formal Spec v4 — Visualization
=======================================
Shows the formalized 7-DB structure with:
  - All 7 DBs with their entry-types + key properties
  - All inter-DB relations (with cardinality + flow color)
  - The 3 flows (Teleological Pull / Ground-Truth / Feedback)
  - The cycle
  - Implementation order
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
    'bg': '#FAFAF7',
    'layer_a': '#FEF3C7', 'layer_a_edge': '#D97706',
    'layer_b': '#EFF6FF', 'layer_b_edge': '#3B82F6',
    'layer_c': '#D1FAE5', 'layer_c_edge': '#10B981',
    'vision': '#FEF3C7', 'compass': '#FED7AA',
    'logbook': '#DBEAFE', 'synthesis': '#BFDBFE', 'profile': '#93C5FD',
    'action': '#A7F3D0', 'context': '#6EE7B7',
    'text': '#243447', 'text_muted': '#64748B',
    'pull': '#D97706', 'ground': '#3B82F6', 'feedback': '#8B5CF6',
    'positive': '#10B981', 'negative': '#EF4444',
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

def leaf(ax, x, y, text, fs=6.5, color=None, z=5):
    c = color or C['text']
    ax.text(x, y, '• ' + text, ha='left', va='center', fontsize=fs, color=c, zorder=z, family='monospace')

def arrow(ax, x1, y1, x2, y2, color='#475569', lw=1.5, style='->', rad=0.0, alpha=0.7, z=3):
    ax.annotate('', xy=(x2, y2), xytext=(x1, y1),
                arrowprops=dict(arrowstyle=f'{style},head_width=0.4,head_length=0.5',
                                color=color, lw=lw, alpha=alpha,
                                connectionstyle=f'arc3,rad={rad}'),
                zorder=z)


def main():
    fig, ax = plt.subplots(figsize=(28, 22), constrained_layout=False)
    ax.set_xlim(0, 100)
    ax.set_ylim(0, 100)
    ax.axis('off')
    ax.set_facecolor(C['bg'])
    fig.patch.set_facecolor(C['bg'])

    # Title
    label(ax, 50, 97, 'LifeOS Formal Spec v4 — The 7-DB Structure (Formalized)',
          fs=18, weight='bold')
    label(ax, 50, 94,
          '7 DBs · 3 layers · 18 inter-DB relations · 4 intra-DB relations · 3 flows · 1 cycle',
          fs=10, color=C['text_muted'], style='italic')

    # ─── Layer backgrounds ───────────────────────────────────────────────
    box(ax, 2, 70, 96, 22, C['layer_a'], C['layer_a_edge'], lw=2, alpha=0.2, radius=0.3, z=1)
    label(ax, 50, 89, 'LAYER A — TELEOLOGICAL PULL  (articulate + simulate the drive toward the ideal-future)',
          fs=11, weight='bold', color=C['layer_a_edge'], z=2)

    box(ax, 2, 30, 96, 36, C['layer_b'], C['layer_b_edge'], lw=2, alpha=0.2, radius=0.3, z=1)
    label(ax, 50, 63, 'LAYER B — HISTORICAL RECORD  (keep tabs on history + current-trajectory/trends)',
          fs=11, weight='bold', color=C['layer_b_edge'], z=2)

    box(ax, 2, 4, 96, 22, C['layer_c'], C['layer_c_edge'], lw=2, alpha=0.2, radius=0.3, z=1)
    label(ax, 50, 23, 'LAYER C — ACTION INTERFACE  (tell the user what to do + how)',
          fs=11, weight='bold', color=C['layer_c_edge'], z=2)

    # ─── Layer A: Vision + Compass ───────────────────────────────────────
    # DB 1: Vision
    box(ax, 5, 71, 40, 16, C['vision'], C['layer_a_edge'], lw=2, alpha=0.75, radius=0.2, z=3)
    label(ax, 25, 85, 'DB 1: VISION', fs=12, weight='bold', color=C['layer_a_edge'], z=4)
    label(ax, 25, 82.5, '(articulate the ideal-future · timeless)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 7, 80, 'Purpose · Value · Principle', fs=7, z=4)
    leaf(ax, 7, 78.5, 'Vision-Statement · Identity-Statement', fs=7, z=4)
    leaf(ax, 7, 76.5, 'Key props: Type, Source, Timeframe, Status', fs=6.5, z=4)
    leaf(ax, 7, 75, 'Relations: → Compass, → Action', fs=6.5, color=C['pull'], z=4)

    # DB 2: Compass
    box(ax, 50, 71, 45, 16, C['compass'], C['layer_a_edge'], lw=2, alpha=0.75, radius=0.2, z=3)
    label(ax, 72.5, 85, 'DB 2: COMPASS', fs=12, weight='bold', color=C['layer_a_edge'], z=4)
    label(ax, 72.5, 82.5, '(time-bound trajectory decomposition)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 52, 80, 'Annual-Goal · Quarterly-Goal · Milestone', fs=7, z=4)
    leaf(ax, 52, 78.5, 'Key props: Type, Year, Quarter, Status, Progress', fs=6.5, z=4)
    leaf(ax, 52, 76.5, 'Relations: ← Vision, → Action, → Profile', fs=6.5, color=C['pull'], z=4)
    leaf(ax, 52, 75, 'Intra-DB: Parent Goal (Annual→Quarterly)', fs=6.5, color=C['text_muted'], z=4)

    # Vision → Compass
    arrow(ax, 45, 79, 50, 79, color=C['pull'], lw=2.5, alpha=0.8, z=4)
    label(ax, 47.5, 80.5, 'Decomposes', fs=6.5, color=C['pull'], style='italic', z=4)

    # ─── Layer B: Logbook + Synthesis + Profile ──────────────────────────
    # DB 3: Logbook
    box(ax, 5, 45, 28, 16, C['logbook'], C['layer_b_edge'], lw=2, alpha=0.75, radius=0.2, z=3)
    label(ax, 19, 59, 'DB 3: LOGBOOK', fs=11, weight='bold', color=C['layer_b_edge'], z=4)
    label(ax, 19, 56.5, '(6 logs, 1 DB · objective capture)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 7, 54, 'Activity · Diet · Financial', fs=6.5, z=4)
    leaf(ax, 7, 52.5, 'Subjective · Relational · Systemic', fs=6.5, z=4)
    leaf(ax, 7, 50.5, 'Key props: Date, Entry Type, Channel', fs=6.5, z=4)
    leaf(ax, 7, 49, 'Relations: ← Action, → Synthesis', fs=6.5, color=C['ground'], z=4)

    # DB 4: Synthesis
    box(ax, 36, 45, 28, 16, C['synthesis'], C['layer_b_edge'], lw=2, alpha=0.75, radius=0.2, z=3)
    label(ax, 50, 59, 'DB 4: SYNTHESIS', fs=11, weight='bold', color=C['layer_b_edge'], z=4)
    label(ax, 50, 56.5, '(logs → insights · polar pair)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 38, 54, 'Note (neutral)', fs=6.5, z=4)
    leaf(ax, 38, 52.5, 'Opportunity · Strength (+)', fs=6.5, color=C['positive'], z=4)
    leaf(ax, 38, 51, 'Directive · Risk (−)', fs=6.5, color=C['negative'], z=4)
    leaf(ax, 38, 49, 'Relations: ← Logbook, → Action/Profile/Vision', fs=6.5, color=C['ground'], z=4)

    # DB 5: Profile
    box(ax, 67, 45, 28, 16, C['profile'], C['layer_b_edge'], lw=2, alpha=0.75, radius=0.2, z=3)
    label(ax, 81, 59, 'DB 5: PROFILE', fs=11, weight='bold', color=C['layer_b_edge'], z=4)
    label(ax, 81, 56.5, '(cumulative state mirror · RPG)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 69, 54, 'Trait · Metric · Capacity · Asset', fs=6.5, z=4)
    leaf(ax, 69, 52.5, 'Key props: Category, Current/Target', fs=6.5, z=4)
    leaf(ax, 69, 51, 'Key props: Trend, Unit, Frequency', fs=6.5, z=4)
    leaf(ax, 69, 49, 'Relations: ← Synthesis, → Vision/Compass', fs=6.5, color=C['feedback'], z=4)

    # Logbook → Synthesis → Profile
    arrow(ax, 33, 53, 36, 53, color=C['ground'], lw=2.5, alpha=0.8, z=4)
    label(ax, 34.5, 54.5, 'synthesizes', fs=6.5, color=C['ground'], style='italic', z=4)

    arrow(ax, 64, 53, 67, 53, color=C['ground'], lw=2.5, alpha=0.8, z=4)
    label(ax, 65.5, 54.5, 'condenses', fs=6.5, color=C['ground'], style='italic', z=4)

    # ─── Layer C: Action + Context ───────────────────────────────────────
    # DB 6: Action
    box(ax, 5, 5, 50, 16, C['action'], C['layer_c_edge'], lw=2, alpha=0.75, radius=0.2, z=3)
    label(ax, 30, 19, 'DB 6: ACTION', fs=12, weight='bold', color=C['layer_c_edge'], z=4)
    label(ax, 30, 16.5, '(actionable hierarchy · what to do + how)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 7, 14, 'Project · Task · Campaign · Content', fs=7, z=4)
    leaf(ax, 7, 12, 'Key props: Type, Status, Priority, Progress', fs=6.5, z=4)
    leaf(ax, 7, 10, 'Relations: ← Compass, ← Synthesis, → Logbook', fs=6.5, color=C['ground'], z=4)
    leaf(ax, 7, 8.5, 'Intra-DB: Parent Project, Blocked By', fs=6.5, color=C['text_muted'], z=4)
    leaf(ax, 7, 7, 'Relations: → Context (Involves)', fs=6.5, color=C['text_muted'], z=4)

    # DB 7: Context
    box(ax, 60, 5, 35, 16, C['context'], C['layer_c_edge'], lw=2, alpha=0.75, radius=0.2, z=3)
    label(ax, 77.5, 19, 'DB 7: CONTEXT', fs=12, weight='bold', color=C['layer_c_edge'], z=4)
    label(ax, 77.5, 16.5, '(the environment · who/what is around)', fs=7.5, color=C['text_muted'], style='italic', z=4)
    leaf(ax, 62, 14, 'Person (14 CRM props)', fs=7, z=4)
    leaf(ax, 62, 12, 'Community · Organization', fs=7, z=4)
    leaf(ax, 62, 10, 'Financial-Account · Place', fs=7, z=4)
    leaf(ax, 62, 8, 'Relations: → Action, → Logbook', fs=6.5, color=C['text_muted'], z=4)
    leaf(ax, 62, 6.5, 'Relations: → Synthesis', fs=6.5, color=C['text_muted'], z=4)

    # Action ↔ Context
    arrow(ax, 55, 13, 60, 13, color=C['layer_c_edge'], lw=2, alpha=0.7, style='<->', z=4)
    label(ax, 57.5, 14.5, 'involves', fs=6.5, color=C['layer_c_edge'], style='italic', z=4)

    # ─── The 3 Flows (layer-spanning arrows) ─────────────────────────────
    # Flow 1: Teleological Pull (Layer A → Layer C)
    arrow(ax, 72, 71, 40, 21, color=C['pull'], lw=3.5, alpha=0.6, rad=0.15, z=2)
    label(ax, 50, 45, 'FLOW 1: TELEOLOGICAL PULL', fs=9, weight='bold', color=C['pull'], z=3, alpha=0.8)
    label(ax, 50, 43, 'Vision → Compass → Action', fs=7.5, color=C['pull'], style='italic', z=3, alpha=0.8)

    # Flow 2: Ground-Truth (Layer C → Layer B)
    arrow(ax, 20, 21, 19, 45, color=C['ground'], lw=3.5, alpha=0.6, rad=-0.1, z=2)
    label(ax, 12, 35, 'FLOW 2: GROUND-TRUTH', fs=9, weight='bold', color=C['ground'], z=3, alpha=0.8)
    label(ax, 12, 33, 'Action → Logbook →', fs=7.5, color=C['ground'], style='italic', z=3, alpha=0.8)
    label(ax, 12, 31.5, 'Synthesis → Profile', fs=7.5, color=C['ground'], style='italic', z=3, alpha=0.8)

    # Flow 3: Feedback (Layer B → Layer A)
    arrow(ax, 81, 61, 25, 71, color=C['feedback'], lw=3.5, alpha=0.5, rad=-0.25, z=2)
    label(ax, 92, 40, 'FLOW 3: FEEDBACK', fs=9, weight='bold', color=C['feedback'], z=3, alpha=0.8)
    label(ax, 92, 38, 'Profile → Vision', fs=7.5, color=C['feedback'], style='italic', z=3, alpha=0.8)
    label(ax, 92, 36.5, '(gap informs pull)', fs=7.5, color=C['feedback'], style='italic', z=3, alpha=0.8)

    # ─── Cycle summary (bottom) ──────────────────────────────────────────
    box(ax, 5, 0.5, 90, 3, 'white', C['text_muted'], lw=1, alpha=0.7, radius=0.1, z=2)
    label(ax, 50, 2,
          'CYCLE:  Vision → Compass → Action → Logbook → Synthesis → Profile → Vision  (causal amplification loop)',
          fs=8.5, color=C['text'], z=3, family='monospace', weight='bold')

    plt.savefig(OUT_DIR / "formal_spec_v4_structure.png", dpi=180,
                bbox_inches='tight', facecolor=C['bg'])
    plt.close()
    print(f"  ✓ {OUT_DIR / 'formal_spec_v4_structure.png'}")

if __name__ == "__main__":
    main()
