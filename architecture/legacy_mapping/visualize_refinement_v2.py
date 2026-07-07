#!/usr/bin/env python3
"""
LifeOS Refinement v2 — The Coupled System Visualization
=========================================================
Shows:
  - User System ⇌ LifeOS System (coupled symbiotic system)
  - Capture System (one system, 6 channels)
  - Synthesis Pipeline (the bloodstream)
  - 3 Engines (Definition / Action / Context) — dynamized by synthesis + trajectory
  - The coupling loop (live → capture → synthesize → operationalize → feed back → reflect)

Layout: large horizontal diagram showing the flow.
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
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch
from pathlib import Path

OUT_DIR = Path("/home/z/my-project/repos/lifeos-ops/architecture/legacy_mapping/visualizations")

# Color palette
C = {
    'bg':           '#FAFAF7',
    'user':         '#FEF3C7',  # warm amber — the lived life
    'user_edge':    '#C6866A',
    'lifeos':       '#EFF6FF',  # ice blue — the prosthetic
    'lifeos_edge':  '#4C6EF5',
    'capture':      '#F1F5F9',  # light gray — perception
    'capture_edge': '#64748B',
    'synthesis':    '#DBEAFE',  # blue — bloodstream
    'synth_edge':   '#3B82F6',
    'def_engine':   '#D1FAE5',  # mint — definition
    'def_edge':     '#10B981',
    'act_engine':   '#FEF3C7',  # amber — action
    'act_edge':     '#F59E0B',
    'ctx_engine':   '#EDE9FE',  # lavender — context
    'ctx_edge':     '#8B5CF6',
    'positive':     '#10B981',
    'negative':     '#EF4444',
    'text':         '#243447',
    'text_muted':   '#64748B',
    'accent':       '#4C6EF5',
    'flow':         '#C6866A',  # terracotta — the coupling flow
}

def box(ax, x, y, w, h, fc, ec, lw=1.5, alpha=0.85, radius=0.2, z=2):
    ax.add_patch(FancyBboxPatch((x, y), w, h,
        boxstyle=f"round,pad=0.02,rounding_size={radius}",
        facecolor=fc, edgecolor=ec, lw=lw, alpha=alpha, zorder=z))

def label(ax, x, y, text, fs=9, color='#243447', weight='bold', ha='center', va='center', style='normal', z=5, alpha=None):
    kwargs = dict(ha=ha, va=va, fontsize=fs, color=color,
                  fontweight=weight, style=style, zorder=z)
    if alpha is not None:
        kwargs['alpha'] = alpha
    ax.text(x, y, text, **kwargs)

def leaf(ax, x, y, text, fs=7, color=None):
    c = color or C['text']
    ax.text(x, y, '• ' + text, ha='left', va='center', fontsize=fs,
            color=c, zorder=5, family='monospace')

def arrow(ax, x1, y1, x2, y2, color='#475569', lw=1.5, style='->', rad=0.0, alpha=0.7, z=3):
    ax.annotate('', xy=(x2, y2), xytext=(x1, y1),
                arrowprops=dict(arrowstyle=f'{style},head_width=0.3,head_length=0.4',
                                color=color, lw=lw, alpha=alpha,
                                connectionstyle=f'arc3,rad={rad}'),
                zorder=z)


def main():
    fig, ax = plt.subplots(figsize=(26, 18), constrained_layout=False)
    ax.set_xlim(0, 100)
    ax.set_ylim(0, 100)
    ax.axis('off')
    ax.set_facecolor(C['bg'])
    fig.patch.set_facecolor(C['bg'])

    # ─── Title ───────────────────────────────────────────────────────────
    label(ax, 50, 97, 'LifeOS Refinement v2 — The Consciousness-Prosthetic (User ⇌ LifeOS Coupled System)',
          fs=17, weight='bold')
    label(ax, 50, 94, 'Engines operationalize themselves through synthesis-pipeline (bloodstream) + real-life user-trajectory (spine). User + LifeOS are coupled systems, not user-uses-tool.',
          fs=10, color=C['text_muted'], style='italic')

    # ─── USER SYSTEM (left column) ───────────────────────────────────────
    box(ax, 2, 12, 22, 78, C['user'], C['user_edge'], lw=2.5, alpha=0.4, radius=0.5)
    label(ax, 13, 87, 'USER SYSTEM', fs=13, weight='bold', color=C['user_edge'])
    label(ax, 13, 84, '(the lived life)', fs=9, color=C['text_muted'], style='italic')

    # Trajectory
    box(ax, 4, 68, 18, 12, 'white', C['user_edge'], lw=1.5, alpha=0.9, radius=0.2)
    label(ax, 13, 78, 'Trajectory', fs=10, weight='bold', color=C['user_edge'])
    label(ax, 13, 75.5, '(past → present → future arc)', fs=7.5, color=C['text_muted'], style='italic')
    label(ax, 13, 73, 'The SPINE the engines\nflow along', fs=7.5, color=C['text'], style='italic')

    # Agency
    box(ax, 4, 50, 18, 12, 'white', C['user_edge'], lw=1.5, alpha=0.9, radius=0.2)
    label(ax, 13, 60, 'Agency', fs=10, weight='bold', color=C['user_edge'])
    label(ax, 13, 57.5, '(decisions + actions)', fs=7.5, color=C['text_muted'], style='italic')
    label(ax, 13, 55, 'Generates the raw data\nthat Capture receives', fs=7.5, color=C['text'], style='italic')

    # Reflection
    box(ax, 4, 32, 18, 12, 'white', C['user_edge'], lw=1.5, alpha=0.9, radius=0.2)
    label(ax, 13, 42, 'Reflection', fs=10, weight='bold', color=C['user_edge'])
    label(ax, 13, 39.5, '(interpretation of experience)', fs=7.5, color=C['text_muted'], style='italic')
    label(ax, 13, 37, 'Informed by Synthesis\nfeeding back', fs=7.5, color=C['text'], style='italic')

    # Embodiment (the body that generates logs)
    box(ax, 4, 14, 18, 12, 'white', C['user_edge'], lw=1.5, alpha=0.9, radius=0.2)
    label(ax, 13, 24, 'Embodiment', fs=10, weight='bold', color=C['user_edge'])
    label(ax, 13, 21.5, '(the physical body)', fs=7.5, color=C['text_muted'], style='italic')
    label(ax, 13, 19, 'Generates Activity/Diet\ndata by living', fs=7.5, color=C['text'], style='italic')

    # ─── LifeOS SYSTEM (right ~75% of canvas) ────────────────────────────
    box(ax, 28, 8, 70, 82, C['lifeos'], C['lifeos_edge'], lw=2.5, alpha=0.4, radius=0.5)
    label(ax, 63, 87, 'LifeOS SYSTEM', fs=13, weight='bold', color=C['lifeos_edge'])
    label(ax, 63, 84, '(the prosthetic extension)', fs=9, color=C['text_muted'], style='italic')

    # ─── CAPTURE SYSTEM (one system, 6 channels) ─────────────────────────
    box(ax, 30, 70, 66, 10, C['capture'], C['capture_edge'], lw=2, alpha=0.7, radius=0.3)
    label(ax, 33, 78, 'CAPTURE', fs=10, weight='bold', color=C['capture_edge'], ha='left')
    label(ax, 33, 75.5, 'SYSTEM', fs=10, weight='bold', color=C['capture_edge'], ha='left')
    label(ax, 33, 73, '(one system,\n6 channels)', fs=7, color=C['text_muted'], style='italic', ha='left')

    # 6 channels
    channels = [
        (40, 'Body:\nActivity-Log\nDiet-Log'),
        (50, 'Mind:\nSubjective-J\nSystemic-J'),
        (60, 'Relational:\nRelational-J'),
        (70, 'Resource:\nFinancial-Log'),
    ]
    for x, txt in channels:
        box(ax, x - 4, 71, 8, 8, 'white', C['capture_edge'], lw=1, alpha=0.9, radius=0.15)
        label(ax, x, 75, txt, fs=7, color=C['text'])

    label(ax, 85, 75, 'Objective\nground-reality\ndata', fs=7.5, color=C['text_muted'], style='italic')

    # ─── SYNTHESIS PIPELINE (the bloodstream) ────────────────────────────
    box(ax, 30, 56, 66, 10, C['synthesis'], C['synth_edge'], lw=2, alpha=0.7, radius=0.3)
    label(ax, 33, 64, 'SYNTHESIS', fs=10, weight='bold', color=C['synth_edge'], ha='left')
    label(ax, 33, 61.5, 'PIPELINE', fs=10, weight='bold', color=C['synth_edge'], ha='left')
    label(ax, 33, 59, '(the bloodstream —\ndynamizes engines)', fs=7, color=C['text_muted'], style='italic', ha='left')

    # Notes (raw) + Polar pair
    box(ax, 42, 57, 14, 8, 'white', C['synth_edge'], lw=1, alpha=0.9, radius=0.15)
    label(ax, 49, 62.5, 'Notes-Management', fs=7.5, weight='bold', color=C['text'])
    label(ax, 49, 60, '(raw synthesis)', fs=7, color=C['text_muted'], style='italic')

    box(ax, 60, 57, 14, 8, 'white', C['positive'], lw=1.5, alpha=0.9, radius=0.15)
    label(ax, 67, 62.5, 'Opportunities', fs=7.5, weight='bold', color=C['positive'])
    label(ax, 67, 60.5, '& Strengths (+)', fs=7, color=C['positive'])

    box(ax, 76, 57, 14, 8, 'white', C['negative'], lw=1.5, alpha=0.9, radius=0.15)
    label(ax, 83, 62.5, 'Directives', fs=7.5, weight='bold', color=C['negative'])
    label(ax, 83, 60.5, '& Risks (−)', fs=7, color=C['negative'])

    # Polar pair arrows (showing they're dialectical)
    arrow(ax, 74, 61, 76, 61, color=C['text_muted'], lw=1, style='<->', alpha=0.5)

    # ─── 3 ENGINES (operationalized by synthesis + trajectory) ───────────
    label(ax, 63, 52, '3 ENGINES — operationalized by synthesis (flowing in) + trajectory (flowing through)',
          fs=10, weight='bold', color=C['lifeos_edge'])
    label(ax, 63, 49.5, 'Each engine is a PROCESS at the intersection of two flows — not a static container.',
          fs=8, color=C['text_muted'], style='italic')

    # Definition Engine (left)
    box(ax, 30, 28, 20, 18, C['def_engine'], C['def_edge'], lw=2, alpha=0.6, radius=0.3)
    label(ax, 40, 44, 'DEFINITION', fs=10, weight='bold', color=C['def_edge'])
    label(ax, 40, 41.5, 'ENGINE', fs=10, weight='bold', color=C['def_edge'])
    label(ax, 40, 39, '"Who am I?\nWhere am I going?"', fs=7.5, color=C['text_muted'], style='italic')
    leaf(ax, 32, 36, 'Compass:', fs=7, color=C['def_edge'])
    leaf(ax, 33, 34.5, 'Values & Principles', fs=7)
    leaf(ax, 33, 33, 'Vision', fs=7)
    leaf(ax, 32, 31, 'Mirror:', fs=7, color=C['def_edge'])
    leaf(ax, 33, 29.5, 'Stats (RPG profile)', fs=7)

    # Action Engine (center)
    box(ax, 52, 28, 22, 18, C['act_engine'], C['act_edge'], lw=2, alpha=0.6, radius=0.3)
    label(ax, 63, 44, 'ACTION', fs=10, weight='bold', color=C['act_edge'])
    label(ax, 63, 41.5, 'ENGINE', fs=10, weight='bold', color=C['act_edge'])
    label(ax, 63, 39, '"What should I do?"', fs=7.5, color=C['text_muted'], style='italic')
    leaf(ax, 54, 36, 'Compass:', fs=7, color=C['act_edge'])
    leaf(ax, 55, 34.5, 'Annual → Quarterly Goals', fs=7)
    leaf(ax, 54, 33, 'Engine:', fs=7, color=C['act_edge'])
    leaf(ax, 55, 31.5, 'Projects → Tasks', fs=7)
    leaf(ax, 55, 30, 'Campaigns → Content', fs=7)

    # Context Engine (right)
    box(ax, 76, 28, 20, 18, C['ctx_engine'], C['ctx_edge'], lw=2, alpha=0.6, radius=0.3)
    label(ax, 86, 44, 'CONTEXT', fs=10, weight='bold', color=C['ctx_edge'])
    label(ax, 86, 41.5, 'ENGINE', fs=10, weight='bold', color=C['ctx_edge'])
    label(ax, 86, 39, '"What\'s around me?"', fs=7.5, color=C['text_muted'], style='italic')
    leaf(ax, 78, 36, 'Ecosystem:', fs=7, color=C['ctx_edge'])
    leaf(ax, 79, 34.5, 'People, Communities', fs=7)
    leaf(ax, 79, 33, 'Financial-Accounts', fs=7)

    # ─── Flow arrows: Capture → Synthesis → Engines ──────────────────────
    # Capture → Synthesis
    arrow(ax, 63, 70, 63, 66, color=C['flow'], lw=2, alpha=0.7)
    label(ax, 64.5, 68, 'feeds', fs=7, color=C['flow'], style='italic', ha='left')

    # Synthesis → 3 engines (the bloodstream dynamizing)
    arrow(ax, 40, 56, 40, 46, color=C['synth_edge'], lw=2, alpha=0.7)
    arrow(ax, 63, 56, 63, 46, color=C['synth_edge'], lw=2, alpha=0.7)
    arrow(ax, 86, 56, 86, 46, color=C['synth_edge'], lw=2, alpha=0.7)
    label(ax, 63, 51.5, 'synthesis flows IN (dynamizes)', fs=7.5, color=C['synth_edge'],
          style='italic', weight='bold')

    # ─── Coupling loop: User ⇌ LifeOS ────────────────────────────────────
    # User lives → Capture (top arrow, left to right)
    arrow(ax, 24, 75, 30, 75, color=C['flow'], lw=2.5, alpha=0.8)
    label(ax, 27, 76.5, 'lives →\ncaptures', fs=7.5, color=C['flow'], weight='bold', style='italic')

    # LifeOS feeds back → User reflects (bottom arrow, right to left)
    arrow(ax, 30, 20, 24, 20, color=C['flow'], lw=2.5, alpha=0.8, style='->')
    label(ax, 27, 21.5, 'feeds back ←\nreflects', fs=7.5, color=C['flow'], weight='bold', style='italic')

    # Trajectory flowing through (dashed line from User Trajectory through all 3 engines)
    for eng_x in [40, 63, 86]:
        arrow(ax, 13, 68, eng_x - 6, 46, color=C['user_edge'], lw=1, alpha=0.3, style='->', rad=0.2)
    label(ax, 18, 58, 'trajectory flows\nTHROUGH', fs=7.5, color=C['user_edge'],
          style='italic', weight='bold', alpha=0.7)

    # ─── The coupling label ──────────────────────────────────────────────
    label(ax, 26, 47, '⇌', fs=24, weight='bold', color=C['flow'])
    label(ax, 26, 43, 'tight\ncoupling', fs=7.5, color=C['flow'], style='italic', weight='bold')

    # ─── Bottom: the coupling loop described ─────────────────────────────
    box(ax, 30, 10, 66, 14, 'white', C['lifeos_edge'], lw=1.5, alpha=0.7, radius=0.2)
    label(ax, 63, 22, 'THE COUPLING LOOP (User ⇌ LifeOS symbiotic system)',
          fs=9, weight='bold', color=C['lifeos_edge'])
    label(ax, 63, 19.5,
          'User lives → Capture receives → Synthesis processes → Engines operationalize →',
          fs=7.5, color=C['text'])
    label(ax, 63, 17.5,
          'LifeOS feeds back (insights, directives, stats) → User reflects + decides → User acts → (loop)',
          fs=7.5, color=C['text'])
    label(ax, 63, 14.5,
          'Design criteria: EFFICIENCY (10-sec logging) · ADAPTABILITY (engines reorganize with trajectory) · EFFICACY (synthesis informs decisions + identity)',
          fs=7.5, color=C['text_muted'], style='italic', weight='bold')

    # ─── Legend ──────────────────────────────────────────────────────────
    legend_x = 4
    legend_y = 6
    label(ax, legend_x, legend_y + 2, 'LEGEND', fs=9, weight='bold', color=C['text'], ha='left')
    items = [
        (C['user'], C['user_edge'], 'User System'),
        (C['capture'], C['capture_edge'], 'Capture (one system)'),
        (C['synthesis'], C['synth_edge'], 'Synthesis (bloodstream)'),
        (C['def_engine'], C['def_edge'], 'Definition Engine'),
        (C['act_engine'], C['act_edge'], 'Action Engine'),
        (C['ctx_engine'], C['ctx_edge'], 'Context Engine'),
    ]
    for i, (fc, ec, name) in enumerate(items):
        x = legend_x + i * 3.5
        box(ax, x, legend_y, 0.8, 0.8, fc, ec, lw=1.5, alpha=0.8, radius=0.1)
        label(ax, x + 1.3, legend_y + 0.4, name, fs=6.5, ha='left')

    plt.savefig(OUT_DIR / "refinement_v2_coupled_system.png", dpi=180,
                bbox_inches='tight', facecolor=C['bg'])
    plt.close()
    print(f"  ✓ {OUT_DIR / 'refinement_v2_coupled_system.png'}")

if __name__ == "__main__":
    main()
