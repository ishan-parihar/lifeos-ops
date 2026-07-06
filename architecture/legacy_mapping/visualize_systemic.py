#!/usr/bin/env python3
"""
LifeOS Systemic Structure — Nested Systems Visualization
=========================================================
Visualizes "systems within systems within systems" for all 4 framings
from BRAINSTORM_systemic_structures.md.

Layout: 2x2 grid, each cell showing one framing as nested boxes.
Outer box = macro-system (LifeOS)
Middle boxes = meso-systems (engines/layers/cycles/pillars)
Inner boxes = micro-systems (functional clusters)
Leaves = DBs (tools, shown as small text)

Style: low-saturation palette, clean nesting, no overlap.
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
from matplotlib.patches import FancyBboxPatch, Rectangle
from pathlib import Path

OUT_DIR = Path("/home/z/my-project/repos/lifeos-ops/architecture/legacy_mapping/visualizations")
OUT_DIR.mkdir(parents=True, exist_ok=True)

# Color palette (low-saturation, per charts skill)
COLORS = {
    'bg':           '#FAFAF7',
    'macro':        '#E9EEF3',  # outermost — pale blue-gray
    'macro_edge':   '#243447',
    'meso_a':       '#DBEAFE',  # engine A — ice blue
    'meso_b':       '#D1FAE5',  # engine B — mint
    'meso_c':       '#FEF3C7',  # engine C — light amber
    'meso_d':       '#EDE9FE',  # engine D — lavender
    'meso_e':       '#FCE7F3',  # engine E — light pink
    'micro':        '#F1F5F9',  # inner — light gray
    'leaf':         '#243447',  # DB text — dark slate
    'accent':       '#4C6EF5',  # accent — blue
    'positive':     '#10B981',  # green (+)
    'negative':     '#EF4444',  # red (−)
    'text':         '#243447',
    'text_muted':   '#64748B',
}

def draw_box(ax, x, y, w, h, facecolor, edgecolor='#243447', lw=1.5, alpha=0.85, radius=0.3):
    """Draw a rounded box."""
    box = FancyBboxPatch((x, y), w, h,
                          boxstyle=f"round,pad=0.02,rounding_size={radius}",
                          facecolor=facecolor, edgecolor=edgecolor, lw=lw, alpha=alpha,
                          zorder=2)
    ax.add_patch(box)

def draw_label(ax, x, y, text, fontsize=9, color='#243447', weight='bold', ha='center', va='center', style='normal'):
    ax.text(x, y, text, ha=ha, va=va, fontsize=fontsize, color=color,
            fontweight=weight, style=style, zorder=5)

def draw_leaf(ax, x, y, text, fontsize=7, color=None):
    """Draw a DB leaf (small text, no box)."""
    c = color or COLORS['leaf']
    ax.text(x, y, '• ' + text, ha='left', va='center', fontsize=fontsize,
            color=c, zorder=5, family='monospace')


# ─── Framing A: 3-Engine Model ───────────────────────────────────────────────
def draw_framing_a(ax, x0, y0, w, h):
    """3-Engine Model: Definition / Action / Context."""
    ax.set_xlim(x0, x0 + w)
    ax.set_ylim(y0, y0 + h)
    ax.axis('off')

    # Macro box (LifeOS)
    draw_box(ax, x0 + 0.5, y0 + 0.5, w - 1, h - 1, COLORS['macro'], COLORS['macro_edge'], 2, 0.4)
    draw_label(ax, x0 + w/2, y0 + h - 1.2, 'LifeOS — The Consciousness Prosthetic',
               fontsize=11, weight='bold')
    draw_label(ax, x0 + w/2, y0 + h - 2.0, 'Framing A: 3-Engine Model',
               fontsize=9, color=COLORS['text_muted'], style='italic')

    # 3 meso boxes (engines) side by side
    engine_w = (w - 3) / 3
    engine_h = h - 4
    engine_y = y0 + 1.5

    engines = [
        ('Definition Engine', COLORS['meso_a'], 'Who am I?\nWhere am I going?\nHow am I doing?',
         ['Identity Core:', '  Values & Principles', '  Vision', '', 'State Mirror:', '  Stats']),
        ('Action Engine', COLORS['meso_b'], 'What should I do?\nWhat\'s happening?\nWhat does it mean?',
         ['Direction Chain:', '  Annual → Quarterly', '  → Projects → Tasks', '',
          'Capture Mesh:', '  6 Logs', '',
          'Synthesis Lens:', '  Notes-Management', '  Opportunities (+)', '  Directives (−)']),
        ('Context Engine', COLORS['meso_c'], 'What\'s around me?',
         ['Relational Field:', '  People', '  Communities', '',
          'Financial Field:', '  Financial-Accounts', '',
          'Creative Field:', '  Campaigns', '  Content-Pipeline']),
    ]

    for i, (name, color, purpose, leaves) in enumerate(engines):
        ex = x0 + 1 + i * (engine_w + 0.5)
        draw_box(ax, ex, engine_y, engine_w, engine_h, color, alpha=0.6, lw=1.5)
        draw_label(ax, ex + engine_w/2, engine_y + engine_h - 0.8, name,
                   fontsize=9, weight='bold')
        draw_label(ax, ex + engine_w/2, engine_y + engine_h - 1.8, purpose,
                   fontsize=7, color=COLORS['text_muted'], style='italic')
        # Leaves
        for j, leaf in enumerate(leaves):
            draw_leaf(ax, ex + 0.3, engine_y + engine_h - 3.0 - j * 0.5, leaf, fontsize=6.5)


# ─── Framing B: 4-Layer Cognitive Architecture ───────────────────────────────
def draw_framing_b(ax, x0, y0, w, h):
    """4-Layer: Perceive / Process / Project / Preserve."""
    ax.set_xlim(x0, x0 + w)
    ax.set_ylim(y0, y0 + h)
    ax.axis('off')

    # Macro box
    draw_box(ax, x0 + 0.5, y0 + 0.5, w - 1, h - 1, COLORS['macro'], COLORS['macro_edge'], 2, 0.4)
    draw_label(ax, x0 + w/2, y0 + h - 1.2, 'LifeOS — The Consciousness Prosthetic',
               fontsize=11, weight='bold')
    draw_label(ax, x0 + w/2, y0 + h - 2.0, 'Framing B: 4-Layer Cognitive Architecture',
               fontsize=9, color=COLORS['text_muted'], style='italic')

    # 4 layers stacked vertically
    layer_h = (h - 4) / 4
    layer_y_start = y0 + 1.5

    layers = [
        ('PERCEIVE Layer', COLORS['meso_a'], 'Capture reality as it happens',
         ['Body: Activity-Log, Diet-Log', 'Mind: Subjective-Journal, Systemic-Journal',
          'Relational: Relational-Journal', 'Resource: Financial-Log']),
        ('PROCESS Layer', COLORS['meso_b'], 'Make sense of what was perceived',
         ['Raw: Notes-Management', 'Polar: Opportunities-and-Strengths (+)',
          'Polar: Directives-and-Risks (−)']),
        ('PROJECT Layer', COLORS['meso_c'], 'Direct action back into reality',
         ['Strategic: Annual-Goals, Quarterly-Goals', 'Execution: Projects, Tasks',
          'Creative: Campaigns, Content-Pipeline']),
        ('PRESERVE Layer', COLORS['meso_d'], 'Maintain identity + context across time',
         ['Identity: Values & Principles, Vision', 'State: Stats',
          'Relational: People, Communities', 'Financial: Financial-Accounts']),
    ]

    for i, (name, color, purpose, leaves) in enumerate(layers):
        ly = layer_y_start + (3 - i) * layer_h
        draw_box(ax, x0 + 1, ly, w - 2, layer_h - 0.3, color, alpha=0.6, lw=1.5)
        draw_label(ax, x0 + 2, ly + layer_h - 0.7, name, fontsize=9, weight='bold', ha='left')
        draw_label(ax, x0 + 2, ly + layer_h - 1.3, purpose, fontsize=7,
                   color=COLORS['text_muted'], style='italic', ha='left')
        # Leaves in 2 columns
        for j, leaf in enumerate(leaves):
            col = j % 2
            row = j // 2
            lx = x0 + 2 + col * (w - 4) / 2
            ly_leaf = ly + layer_h - 2.0 - row * 0.5
            draw_leaf(ax, lx, ly_leaf, leaf, fontsize=6.5)


# ─── Framing C: 2-Cycle Holonic Model ────────────────────────────────────────
def draw_framing_c(ax, x0, y0, w, h):
    """2-Cycle: Inner (self-maintenance) + Outer (self-transcendence)."""
    ax.set_xlim(x0, x0 + w)
    ax.set_ylim(y0, y0 + h)
    ax.axis('off')

    # Macro box
    draw_box(ax, x0 + 0.5, y0 + 0.5, w - 1, h - 1, COLORS['macro'], COLORS['macro_edge'], 2, 0.4)
    draw_label(ax, x0 + w/2, y0 + h - 1.2, 'LifeOS — The Consciousness Prosthetic',
               fontsize=11, weight='bold')
    draw_label(ax, x0 + w/2, y0 + h - 2.0, 'Framing C: 2-Cycle Holonic Model',
               fontsize=9, color=COLORS['text_muted'], style='italic')

    # 2 cycles side by side
    cycle_w = (w - 3) / 2
    cycle_h = h - 4
    cycle_y = y0 + 1.5

    # Inner Cycle (left)
    draw_box(ax, x0 + 1, cycle_y, cycle_w, cycle_h, COLORS['meso_a'], alpha=0.5, lw=2)
    draw_label(ax, x0 + 1 + cycle_w/2, cycle_y + cycle_h - 0.8, 'INNER CYCLE',
               fontsize=10, weight='bold', color=COLORS['accent'])
    draw_label(ax, x0 + 1 + cycle_w/2, cycle_y + cycle_h - 1.5, 'Self-Maintenance',
               fontsize=8, color=COLORS['text_muted'], style='italic')
    draw_label(ax, x0 + 1 + cycle_w/2, cycle_y + cycle_h - 2.2, '(the metabolic engine — daily)',
               fontsize=7, color=COLORS['text_muted'], style='italic')

    inner_stages = [
        ('Capture', COLORS['micro'], ['6 Logs:', '  Activity, Diet, Financial', '  Subjective, Relational, Systemic']),
        ('Digest', COLORS['micro'], ['Notes-Management', '→ Opportunities (+)', '→ Directives (−)']),
        ('Integrate', COLORS['micro'], ['Stats', '(current state across', ' all dimensions)']),
    ]
    stage_h = (cycle_h - 3.5) / 3
    for i, (name, color, leaves) in enumerate(inner_stages):
        sy = cycle_y + cycle_h - 3.5 - (i + 1) * stage_h + 0.3
        draw_box(ax, x0 + 1.3, sy, cycle_w - 0.6, stage_h - 0.2, color, alpha=0.8, lw=1)
        draw_label(ax, x0 + 1.5, sy + stage_h - 0.5, name, fontsize=8, weight='bold', ha='left')
        for j, leaf in enumerate(leaves):
            draw_leaf(ax, x0 + 1.5, sy + stage_h - 1.1 - j * 0.4, leaf, fontsize=6.5)

    # Outer Cycle (right)
    draw_box(ax, x0 + 2 + cycle_w, cycle_y, cycle_w, cycle_h, COLORS['meso_c'], alpha=0.5, lw=2)
    draw_label(ax, x0 + 2 + cycle_w + cycle_w/2, cycle_y + cycle_h - 0.8, 'OUTER CYCLE',
               fontsize=10, weight='bold', color='#C6866A')
    draw_label(ax, x0 + 2 + cycle_w + cycle_w/2, cycle_y + cycle_h - 1.5, 'Self-Transcendence',
               fontsize=8, color=COLORS['text_muted'], style='italic')
    draw_label(ax, x0 + 2 + cycle_w + cycle_w/2, cycle_y + cycle_h - 2.2, '(the evolutionary engine — periodic)',
               fontsize=7, color=COLORS['text_muted'], style='italic')

    outer_stages = [
        ('Envision', COLORS['micro'], ['Values & Principles', 'Vision']),
        ('Strategize', COLORS['micro'], ['Annual-Goals', 'Quarterly-Goals']),
        ('Execute', COLORS['micro'], ['Projects, Tasks', 'Campaigns, Content-Pipeline']),
        ('Contextualize', COLORS['micro'], ['People, Communities', 'Financial-Accounts']),
    ]
    stage_h_o = (cycle_h - 3.5) / 4
    for i, (name, color, leaves) in enumerate(outer_stages):
        sy = cycle_y + cycle_h - 3.5 - (i + 1) * stage_h_o + 0.3
        draw_box(ax, x0 + 2.3 + cycle_w, sy, cycle_w - 0.6, stage_h_o - 0.15, color, alpha=0.8, lw=1)
        draw_label(ax, x0 + 2.5 + cycle_w, sy + stage_h_o - 0.4, name, fontsize=8, weight='bold', ha='left')
        for j, leaf in enumerate(leaves):
            draw_leaf(ax, x0 + 2.5 + cycle_w, sy + stage_h_o - 0.9 - j * 0.4, leaf, fontsize=6.5)

    # Contact-boundary arrow between cycles
    ax.annotate('', xy=(x0 + 1 + cycle_w, cycle_y + cycle_h/2),
                xytext=(x0 + 2 + cycle_w, cycle_y + cycle_h/2),
                arrowprops=dict(arrowstyle='<->', color=COLORS['accent'], lw=2))
    draw_label(ax, x0 + 1.5 + cycle_w, cycle_y + cycle_h/2 + 0.5, 'Contact-Boundary',
               fontsize=7, color=COLORS['accent'], weight='bold')
    draw_label(ax, x0 + 1.5 + cycle_w, cycle_y + cycle_h/2 - 0.5, '(Synthesis Lens)',
               fontsize=6.5, color=COLORS['text_muted'], style='italic')


# ─── Framing D: 5-Pillar Model ───────────────────────────────────────────────
def draw_framing_d(ax, x0, y0, w, h):
    """5-Pillar: Mirror / Compass / Engine / Synthesizer / Ecosystem."""
    ax.set_xlim(x0, x0 + w)
    ax.set_ylim(y0, y0 + h)
    ax.axis('off')

    # Macro box
    draw_box(ax, x0 + 0.5, y0 + 0.5, w - 1, h - 1, COLORS['macro'], COLORS['macro_edge'], 2, 0.4)
    draw_label(ax, x0 + w/2, y0 + h - 1.2, 'LifeOS — The Consciousness Prosthetic',
               fontsize=11, weight='bold')
    draw_label(ax, x0 + w/2, y0 + h - 2.0, 'Framing D: 5-Pillar Model',
               fontsize=9, color=COLORS['text_muted'], style='italic')

    # 5 pillars side by side
    pillar_w = (w - 3.5) / 5
    pillar_h = h - 4
    pillar_y = y0 + 1.5

    pillars = [
        ('MIRROR', COLORS['meso_a'], 'Reflect current\nstate', ['Stats', '6 Logs']),
        ('COMPASS', COLORS['meso_b'], 'Give direction', ['Values & Principles', 'Vision', 'Annual-Goals', 'Quarterly-Goals']),
        ('ENGINE', COLORS['meso_c'], 'Drive action', ['Projects', 'Tasks', 'Campaigns', 'Content-Pipeline']),
        ('SYNTHESIZER', COLORS['meso_d'], 'Make meaning', ['Notes-Management', 'Opportunities (+)', 'Directives (−)']),
        ('ECOSYSTEM', COLORS['meso_e'], 'Track the\nenvironment', ['People', 'Communities', 'Financial-Accounts']),
    ]

    for i, (name, color, purpose, leaves) in enumerate(pillars):
        px = x0 + 1.25 + i * (pillar_w + 0.25)
        draw_box(ax, px, pillar_y, pillar_w, pillar_h, color, alpha=0.6, lw=1.5)
        draw_label(ax, px + pillar_w/2, pillar_y + pillar_h - 0.8, name,
                   fontsize=9, weight='bold')
        draw_label(ax, px + pillar_w/2, pillar_y + pillar_h - 1.8, purpose,
                   fontsize=7, color=COLORS['text_muted'], style='italic')
        for j, leaf in enumerate(leaves):
            draw_leaf(ax, px + 0.2, pillar_y + pillar_h - 2.8 - j * 0.5, leaf, fontsize=6.5)


# ─── Main: 2x2 grid ──────────────────────────────────────────────────────────
def main():
    fig, axes = plt.subplots(2, 2, figsize=(28, 20), constrained_layout=False)
    fig.patch.set_facecolor(COLORS['bg'])

    # Title
    fig.suptitle('LifeOS Systemic Structure — 4 Framings (Systems Within Systems Within Systems)',
                 fontsize=18, fontweight='bold', color=COLORS['text'], y=0.98)
    fig.text(0.5, 0.955, '"DB have a purpose, DB is not the purpose." Each framing pools the 21 legacy DBs by PURPOSE, not by what they ARE.',
             ha='center', fontsize=11, color=COLORS['text_muted'], style='italic')

    # Draw each framing
    draw_framing_a(axes[0, 0], 0, 0, 14, 10)
    draw_framing_b(axes[0, 1], 0, 0, 14, 10)
    draw_framing_c(axes[1, 0], 0, 0, 14, 10)
    draw_framing_d(axes[1, 1], 0, 0, 14, 10)

    plt.subplots_adjust(left=0.02, right=0.98, top=0.93, bottom=0.02, wspace=0.05, hspace=0.08)
    plt.savefig(OUT_DIR / "systemic_structures_4_framings.png", dpi=180,
                bbox_inches='tight', facecolor=COLORS['bg'])
    plt.close()
    print(f"  ✓ {OUT_DIR / 'systemic_structures_4_framings.png'}")

if __name__ == "__main__":
    main()
