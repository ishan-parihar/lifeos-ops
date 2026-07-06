#!/usr/bin/env python3
"""
LifeOS R&D v3 — The Causal Amplification Architecture Visualization
====================================================================
Shows:
  - The User Cycle (Trajectory → Agency → Reflection → Embodiment → loop)
  - LifeOS as the INTERVENTION LAYER at each interface (not a coupled system)
  - The 4 intervention modes: ALIGN / SUPPORT / AMPLIFY / SCAFFOLD
  - The teleological pull (Vision as the attractor)
  - The prosthetic mechanism (Capture=sensory, Synthesis=digestive, Engines=motor)

Layout: circular diagram with the User Cycle at center, LifeOS intervention
ring around it, Vision (the attractor) at top.
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
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch, Circle, Wedge
import numpy as np
from pathlib import Path

OUT_DIR = Path("/home/z/my-project/repos/lifeos-ops/architecture/legacy_mapping/visualizations")
OUT_DIR.mkdir(parents=True, exist_ok=True)

C = {
    'bg':           '#FAFAF7',
    'user_cycle':   '#FEF3C7',  # warm amber — the user's lived cycle
    'user_edge':    '#C6866A',
    'trajectory':   '#DBEAFE',  # ice blue
    'agency':       '#D1FAE5',  # mint
    'reflection':   '#EDE9FE',  # lavender
    'embodiment':   '#FCE7F3',  # light pink
    'intervention': '#EFF6FF',  # pale blue — the prosthetic layer
    'int_edge':     '#4C6EF5',
    'vision':       '#FEF3C7',  # gold — the attractor
    'vision_edge':  '#D97706',
    'capture':      '#F1F5F9',
    'synthesis':    '#DBEAFE',
    'engines':      '#D1FAE5',
    'text':         '#243447',
    'text_muted':   '#64748B',
    'flow':         '#C6866A',
    'accent':       '#4C6EF5',
    'positive':     '#10B981',
    'negative':     '#EF4444',
}

def box(ax, x, y, w, h, fc, ec, lw=1.5, alpha=0.85, radius=0.15, z=2):
    ax.add_patch(FancyBboxPatch((x, y), w, h,
        boxstyle=f"round,pad=0.02,rounding_size={radius}",
        facecolor=fc, edgecolor=ec, lw=lw, alpha=alpha, zorder=z))

def label(ax, x, y, text, fs=9, color='#243447', weight='bold', ha='center', va='center', style='normal', z=5, alpha=None):
    kwargs = dict(ha=ha, va=va, fontsize=fs, color=color, fontweight=weight, style=style, zorder=z)
    if alpha is not None: kwargs['alpha'] = alpha
    ax.text(x, y, text, **kwargs)

def arrow(ax, x1, y1, x2, y2, color='#475569', lw=1.5, style='->', rad=0.0, alpha=0.7, z=3):
    ax.annotate('', xy=(x2, y2), xytext=(x1, y1),
                arrowprops=dict(arrowstyle=f'{style},head_width=0.4,head_length=0.5',
                                color=color, lw=lw, alpha=alpha,
                                connectionstyle=f'arc3,rad={rad}'),
                zorder=z)


def main():
    fig, ax = plt.subplots(figsize=(22, 24), constrained_layout=False)
    ax.set_xlim(0, 100)
    ax.set_ylim(0, 110)
    ax.axis('off')
    ax.set_facecolor(C['bg'])
    fig.patch.set_facecolor(C['bg'])

    # ─── Title ───────────────────────────────────────────────────────────
    label(ax, 50, 107, 'LifeOS R&D v3 — The Causal Amplification Architecture',
          fs=18, weight='bold')
    label(ax, 50, 103.5,
          'LifeOS is not a coupled system. It is the INTERVENTION LAYER at each interface of the user\'s causal amplification cycle.',
          fs=10, color=C['text_muted'], style='italic')
    label(ax, 50, 101,
          'The cycle is PULLED toward the ideal future (Vision). Each loop AMPLIFIES the user\'s causal capacity.',
          fs=10, color=C['text_muted'], style='italic')

    # ─── VISION (the teleological attractor) — top center ────────────────
    box(ax, 35, 92, 30, 7, C['vision'], C['vision_edge'], lw=3, alpha=0.9, radius=0.3, z=6)
    label(ax, 50, 97, 'THE IDEAL FUTURE', fs=11, weight='bold', color=C['vision_edge'], z=7)
    label(ax, 50, 94.5, 'Vision — the teleological attractor', fs=8, color=C['text_muted'], style='italic', z=7)

    # Teleological pull arrows (downward from Vision to the cycle)
    for angle in [-30, 0, 30]:
        x_end = 50 + 25 * np.sin(np.radians(angle))
        y_end = 88 - 5 * np.cos(np.radians(angle))
        arrow(ax, 50, 92, x_end, y_end, color=C['vision_edge'], lw=2, alpha=0.4, rad=angle*0.01, z=4)
    label(ax, 50, 89.5, 'teleological pull', fs=8, color=C['vision_edge'], style='italic', weight='bold', alpha=0.6)

    # ─── THE USER CYCLE (center) — 4 nodes in a circle ───────────────────
    cx, cy = 50, 55  # center of the cycle
    r = 18  # radius of the cycle

    # 4 cycle positions (top=Trajectory, right=Agency, bottom=Reflection, left=Embodiment)
    positions = {
        'Trajectory':  (cx, cy + r),           # top
        'Agency':      (cx + r, cy),           # right
        'Reflection':  (cx, cy - r),           # bottom
        'Embodiment':  (cx - r, cy),           # left
    }

    # Draw the cycle arrows (curved, showing amplification)
    cycle_order = ['Trajectory', 'Agency', 'Reflection', 'Embodiment']
    for i in range(4):
        src = cycle_order[i]
        dst = cycle_order[(i + 1) % 4]
        x1, y1 = positions[src]
        x2, y2 = positions[dst]
        # Curved arrow (clockwise)
        rad = -0.3 if i % 2 == 0 else -0.3
        arrow(ax, x1, y1, x2, y2, color=C['user_edge'], lw=3, alpha=0.6, rad=rad, z=4)

    # Draw the 4 cycle nodes
    node_colors = {
        'Trajectory': C['trajectory'],
        'Agency': C['agency'],
        'Reflection': C['reflection'],
        'Embodiment': C['embodiment'],
    }
    node_edges = {
        'Trajectory': '#3B82F6',
        'Agency': '#10B981',
        'Reflection': '#8B5CF6',
        'Embodiment': '#EC4899',
    }
    node_descs = {
        'Trajectory': 'past causation\n→ present\nLifeOS ALIGNS',
        'Agency': 'decisions + actions\ngenerates data\n+ gets aligned\nLifeOS SUPPORTS',
        'Reflection': 'processing catalysts\nENHANCED by\nthe prosthetic\nLifeOS AMPLIFIES',
        'Embodiment': 'insights → behavioral\npatterns → new causation\nLifeOS SCAFFOLDS',
    }

    for name, (x, y) in positions.items():
        # Node circle
        circle = Circle((x, y), 7, facecolor=node_colors[name],
                        edgecolor=node_edges[name], lw=2.5, alpha=0.85, zorder=5)
        ax.add_patch(circle)
        label(ax, x, y + 2, name.upper(), fs=10, weight='bold', color=node_edges[name], z=6)
        label(ax, x, y - 1.5, node_descs[name], fs=6.5, color=C['text'], z=6)

    # Center label
    label(ax, cx, cy + 2, 'THE CAUSAL', fs=9, weight='bold', color=C['user_edge'], z=6)
    label(ax, cx, cy, 'AMPLIFICATION', fs=9, weight='bold', color=C['user_edge'], z=6)
    label(ax, cx, cy - 2, 'CYCLE', fs=9, weight='bold', color=C['user_edge'], z=6)

    # ─── LifeOS INTERVENTION LAYER (outer ring) ──────────────────────────
    # 4 intervention boxes, one at each interface (between cycle nodes)
    intervention_positions = {
        'ALIGN':     (cx + r * 0.7, cy + r * 0.7),    # top-right (between Trajectory & Agency)
        'SUPPORT':   (cx + r * 0.7, cy - r * 0.7),    # bottom-right (between Agency & Reflection)
        'AMPLIFY':   (cx - r * 0.7, cy - r * 0.7),    # bottom-left (between Reflection & Embodiment)
        'SCAFFOLD':  (cx - r * 0.7, cy + r * 0.7),    # top-left (between Embodiment & Trajectory)
    }
    intervention_descs = {
        'ALIGN':    'Alignment Vector\nVision, Values,\nGoals',
        'SUPPORT':  'Decision Support\nProjects, Tasks,\nDirectives',
        'AMPLIFY':  'Cognitive Enhancement\nSynthesis Pipeline\nOpp (+) / Dir (−)',
        'SCAFFOLD': 'Digestion Scaffolding\nStats, 6 Logs\n(pattern mirror)',
    }

    for name, (x, y) in intervention_positions.items():
        box(ax, x - 6, y - 4, 12, 8, C['intervention'], C['int_edge'], lw=2, alpha=0.7, radius=0.2, z=3)
        label(ax, x, y + 2.5, name, fs=8, weight='bold', color=C['int_edge'], z=4)
        label(ax, x, y - 0.5, intervention_descs[name], fs=6, color=C['text'], z=4)

    # Intervention arrows (from intervention boxes to cycle nodes — showing intervention)
    for name, (ix, iy) in intervention_positions.items():
        # Each intervention box sits between 2 cycle nodes — draw arrows to both
        # Find the 2 nearest cycle nodes
        dists = {n: ((ix - px)**2 + (iy - py)**2)**0.5 for n, (px, py) in positions.items()}
        nearest = sorted(dists, key=dists.get)[:2]
        for n in nearest:
            px, py = positions[n]
            arrow(ax, ix, iy, px, py, color=C['int_edge'], lw=1, alpha=0.3, style='->', rad=0.1, z=2)

    # ─── THE PROSTHETIC MECHANISM (bottom) ───────────────────────────────
    box(ax, 5, 5, 90, 18, 'white', C['int_edge'], lw=2, alpha=0.5, radius=0.3, z=2)
    label(ax, 50, 21, 'THE PROSTHETIC MECHANISM (3 functional modes running simultaneously at every interface)',
          fs=10, weight='bold', color=C['int_edge'], z=3)
    label(ax, 50, 19, 'Not sequential stages — the same prosthetic operating in 3 modes at once',
          fs=8, color=C['text_muted'], style='italic', z=3)

    # 3 modes
    modes = [
        (20, 'CAPTURE\n(sensory)', C['capture'], '6 Logs:\nActivity, Diet, Financial,\nSubjective, Relational, Systemic\n\nPerceives the user\'s cycle\nat each stage'),
        (50, 'SYNTHESIS\n(digestive)', C['synthesis'], 'Notes-Management →\nOpportunities-and-Strengths (+)\nDirectives-and-Risks (−)\n\nTransforms raw signals\ninto actionable insight'),
        (80, 'ENGINES\n(motor)', C['engines'], 'Definition: Vision, Values, Stats\nAction: Goals, Projects, Tasks\nContext: People, Communities,\nFinancial-Accounts\n\nEnacts the intervention\nat each interface'),
    ]
    for x, title, fc, desc in modes:
        box(ax, x - 11, 7, 22, 10, fc, C['int_edge'], lw=1.5, alpha=0.7, radius=0.2, z=3)
        label(ax, x, 15.5, title, fs=9, weight='bold', color=C['int_edge'], z=4)
        label(ax, x, 11, desc, fs=6.5, color=C['text'], z=4)

    # ─── Connection from mechanism to intervention layer ─────────────────
    arrow(ax, 50, 17, 50, 35, color=C['int_edge'], lw=2, alpha=0.4, rad=0, z=1)
    label(ax, 51.5, 26, 'enacts\nintervention', fs=7, color=C['int_edge'], style='italic', ha='left', alpha=0.6, z=3)

    # ─── Legend ──────────────────────────────────────────────────────────
    legend_y = 0.5
    label(ax, 5, legend_y + 1.5, 'LEGEND:', fs=8, weight='bold', color=C['text'], ha='left', z=5)
    items = [
        (C['user_cycle'], C['user_edge'], 'User Cycle'),
        (C['intervention'], C['int_edge'], 'LifeOS Intervention'),
        (C['vision'], C['vision_edge'], 'Teleological Attractor'),
        (C['capture'], C['int_edge'], 'Capture (sensory)'),
        (C['synthesis'], C['int_edge'], 'Synthesis (digestive)'),
        (C['engines'], C['int_edge'], 'Engines (motor)'),
    ]
    for i, (fc, ec, name) in enumerate(items):
        x = 15 + i * 14
        box(ax, x, legend_y, 1, 1, fc, ec, lw=1.5, alpha=0.8, radius=0.1, z=5)
        label(ax, x + 2, legend_y + 0.5, name, fs=7, ha='left', z=5)

    plt.savefig(OUT_DIR / "r_and_d_v3_causal_amplification.png", dpi=180,
                bbox_inches='tight', facecolor=C['bg'])
    plt.close()
    print(f"  ✓ {OUT_DIR / 'r_and_d_v3_causal_amplification.png'}")

if __name__ == "__main__":
    main()
