#!/usr/bin/env python3
"""
Step 5 (Phase 3): Finalise editable PowerPoint presentation
`deliverables/powerpoint/STRATO_Presentation_Phase3_Final.pptx`
from the 105-slide `STRATO_Presentation_Final.pptx` baseline:

- Preserves core TRISO geometry, governing equations, Method A BTCS 10-step derivation,
  Method B Sturm-Liouville 10-step derivation, Stage A-D equation/pseudocode slides,
  operating-scenario slides (64-93), AGR-1 irradiation conditions (94-95), and
  discrepancy/limitations/future research slides.
- Prunes outdated/redundant Ag/Cs/Kr slides and duplicate summaries.
- Integrates ALL 39 canonical Graph B figures (A01-A24, B01-B06, C01-C03, D01-D06)
  with experimental overlays and one primary final-endpoint Delta per model curve.
- Adds a concise Option B eight-configuration (P01-P08) website explanation slide.
- Enforces the strict final four slide sequence:
    (N-3) Experimental Deviation Summary
    (N-2) Scientific Conclusions
    (N-1) Limitations and Possible Mechanisms
    (N)   Future Research Directions
"""

import json
import shutil
from pathlib import Path

import pandas as pd
from pptx import Presentation
from pptx.dml.color import RGBColor
from pptx.enum.text import PP_ALIGN
from pptx.util import Inches, Pt

REPO = Path(__file__).resolve().parent.parent
BASELINE_PPTX = Path(
    r"C:\Users\Alf\Downloads\STRATO_Archive\STRATO_Archive\deliverables\powerpoint\STRATO_Presentation_Final.pptx"
)
OUT_PPTX = REPO / "deliverables" / "powerpoint" / "STRATO_Presentation_Phase3_Final.pptx"
DATA = REPO / "data" / "study39"
GRAPHS = REPO / "graphs" / "study39"

WHITE = RGBColor(0xFF, 0xFF, 0xFF)
NAVY = RGBColor(0x1E, 0x3A, 0x8A)
SLATE = RGBColor(0x47, 0x55, 0x69)
DARK = RGBColor(0x11, 0x18, 0x27)
RED_A = RGBColor(0xDC, 0x26, 0x26)


def style_header(slide, title_text: str, breadcrumb: str, num_placeholder: str = ""):
    tb = slide.shapes.add_textbox(Inches(0.68), Inches(0.10), Inches(10.80), Inches(0.30))
    r = tb.text_frame.paragraphs[0].add_run()
    r.text = breadcrumb
    r.font.name = "Calibri"
    r.font.size = Pt(9)
    r.font.bold = True
    r.font.color.rgb = SLATE

    tt = slide.shapes.add_textbox(Inches(0.68), Inches(0.42), Inches(11.30), Inches(0.75))
    tt.text_frame.word_wrap = True
    tr = tt.text_frame.paragraphs[0].add_run()
    tr.text = title_text
    tr.font.name = "Calibri"
    tr.font.size = Pt(22)
    tr.font.bold = True
    tr.font.color.rgb = DARK

    nb = slide.shapes.add_textbox(Inches(12.15), Inches(0.70), Inches(0.62), Inches(0.40))
    np_ = nb.text_frame.paragraphs[0]
    np_.alignment = PP_ALIGN.RIGHT
    nr = np_.add_run()
    nr.text = num_placeholder
    nr.font.name = "Calibri"
    nr.font.size = Pt(12)
    nr.font.color.rgb = SLATE


def set_notes(slide, text: str):
    slide.notes_slide.notes_text_frame.text = text


def delete_slide(prs: Presentation, index: int):
    rId = prs.slides._sldIdLst[index].rId
    prs.part.drop_rel(rId)
    del prs.slides._sldIdLst[index]


def reorder_slides(prs: Presentation, ordered_orig_indices: list[int]):
    sldIdLst = prs.slides._sldIdLst
    elems = list(sldIdLst)
    new_elems = [elems[i] for i in ordered_orig_indices]
    for ch in list(sldIdLst):
        sldIdLst.remove(ch)
    for ch in new_elems:
        sldIdLst.append(ch)


def main():
    OUT_PPTX.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(BASELINE_PPTX, OUT_PPTX)
    prs = Presentation(OUT_PPTX)

    c01 = json.load(open(DATA / "diagnostics" / "c01_convergence.json", encoding="utf-8"))
    c02 = json.load(open(DATA / "diagnostics" / "c02_method_a_vs_b.json", encoding="utf-8"))
    mb = pd.read_csv(DATA / "diagnostics" / "mass_balance_audit.csv")
    ep_df = pd.read_csv(DATA / "endpoint_deviations.csv")
    sum_df = pd.read_csv(DATA / "deviations_exact_summary.csv")
    imp_df = pd.read_csv(DATA / "stage_improvement.csv")
    manifest = json.load(open(DATA / "study39_manifest.json", encoding="utf-8"))
    g_map = {g["graph_id"]: g for g in manifest["graphs"]}

    layout_blank = prs.slide_layouts[1]

    def add_clean_slide(title_text: str, breadcrumb: str, notes: str):
        s = prs.slides.add_slide(layout_blank)
        for ph in list(s.placeholders):
            sp = ph._element
            sp.getparent().remove(sp)
        style_header(s, title_text, breadcrumb, "?")
        set_notes(s, notes)
        return s

    def add_pair_graph_slide(gid_left: str, gid_right: str, title: str, crumb: str):
        gl = g_map[gid_left]
        gr = g_map[gid_right]
        notes = (
            f"Canonical Graph B figures {gid_left} & {gid_right} from graphs/study39/ & data/study39/endpoint_deviations.csv. "
            f"Sources: Bullock (1984); IAEA-TECDOC-978 (1997); Minato et al. (2000); IAEA-TECDOC-2090 (2025)."
        )
        s = add_clean_slide(title, crumb, notes)
        s.shapes.add_picture(str(GRAPHS / f"{gid_left}.png"), Inches(0.55), Inches(1.66), height=Inches(3.75))
        s.shapes.add_picture(str(GRAPHS / f"{gid_right}.png"), Inches(6.85), Inches(1.66), height=Inches(3.75))

        tb = s.shapes.add_textbox(Inches(0.55), Inches(5.55), Inches(12.22), Inches(1.45))
        tb.text_frame.word_wrap = True
        p = tb.text_frame.paragraphs[0]
        r = p.add_run()
        def ep_summary(gid_x, gx):
            if gid_x in ("D01", "D02"):
                tc_x = int(gx["temperature_c"])
                row_x = ep_df[(ep_df["temperature_c"] == tc_x) & (ep_df["config_id"] == "P01") & (ep_df["method"] == "Method A")].iloc[0]
                row_d = ep_df[(ep_df["temperature_c"] == tc_x) & (ep_df["config_id"] == "P04") & (ep_df["method"] == "Method A")].iloc[0]
                return f"Stage A {row_x['label_text']} | Stage D {row_d['delta_pp']:+.3f} pp"
            return gx.get("endpoint_label") or gx.get("method_b_endpoint_label") or "Deviation N/A"

        r.text = (
            f"• {gid_left} ({gl['temperature_c']} °C): {ep_summary(gid_left, gl)}  —  {gl.get('experimental_reference') or gl.get('experimental_endpoint', gl.get('physics_summary', ''))}\n"
            f"• {gid_right} ({gr['temperature_c']} °C): {ep_summary(gid_right, gr)}  —  {gr.get('experimental_reference') or gr.get('experimental_endpoint', gr.get('physics_summary', ''))}"
        )
        r.font.name = "Calibri"
        r.font.size = Pt(10.5)
        r.font.color.rgb = DARK
        return s

    # Build new Phase 3 slides (recording their indices after prs.slides baseline 105)
    # 1. Stage A A01-A06 (3 slides x 2)
    idx_a_slides = [
        len(prs.slides) + k
        for k in range(3)
    ]
    add_pair_graph_slide("A01", "A02", "Stage A (P01 Baseline): 1200 °C & 1350 °C vs Upper Bounds", "GROUP A  ·  METHOD A BTCS  ·  CS-137  ·  GRAPH B OVERLAY")
    add_pair_graph_slide("A03", "A04", "Stage A (P01 Baseline): 1500 °C & 1600 °C vs Bounds", "GROUP A  ·  METHOD A BTCS  ·  CS-137  ·  GRAPH B OVERLAY")
    add_pair_graph_slide("A05", "A06", "Stage A (P01 Baseline): 1700 °C ACT3 & 1800 °C ACT4", "GROUP A  ·  METHOD A BTCS  ·  CS-137  ·  GRAPH B OVERLAY")

    # 2. Stage B A07-A12 (3 slides x 2)
    idx_b_slides = [len(prs.slides) + k for k in range(3)]
    add_pair_graph_slide("A07", "A08", "Stage B (P02 +G, Fresh F_gen): 1200 °C & 1350 °C", "GROUP A  ·  STAGE B (+G)  ·  RAW DIFFERENCE VS G=0 BOUND")
    add_pair_graph_slide("A09", "A10", "Stage B (P02 +G, Fresh F_gen): 1500 °C & 1600 °C", "GROUP A  ·  STAGE B (+G)  ·  RAW DIFFERENCE VS G=0 BOUND")
    add_pair_graph_slide("A11", "A12", "Stage B (P02 +G, Fresh F_gen): 1700 °C & 1800 °C", "GROUP A  ·  STAGE B (+G)  ·  NON-EQUIVALENT F_GEN DENOMINATOR")

    # 3. Stage C A13-A18 (3 slides x 2)
    idx_c_slides = [len(prs.slides) + k for k in range(3)]
    add_pair_graph_slide("A13", "A14", "Stage C (P05 +G, +λ, Fresh F_gen): 1200 °C & 1350 °C", "GROUP A  ·  STAGE C (+G, +λ)  ·  CS-137  ·  GRAPH B OVERLAY")
    add_pair_graph_slide("A15", "A16", "Stage C (P05 +G, +λ, Fresh F_gen): 1500 °C & 1600 °C", "GROUP A  ·  STAGE C (+G, +λ)  ·  CS-137  ·  GRAPH B OVERLAY")
    add_pair_graph_slide("A17", "A18", "Stage C (P05 +G, +λ, Fresh F_gen): 1700 °C & 1800 °C", "GROUP A  ·  STAGE C (+G, +λ)  ·  CS-137  ·  GRAPH B OVERLAY")

    # 4. Stage D A19-A24 (3 slides x 2)
    idx_d_slides = [len(prs.slides) + k for k in range(3)]
    add_pair_graph_slide("A19", "A20", "Stage D (P04 Heating Ramp D(r, T(t)), G=0): 1200 °C & 1350 °C", "GROUP A  ·  STAGE D (RAMP)  ·  CS-137  ·  GRAPH B OVERLAY")
    add_pair_graph_slide("A21", "A22", "Stage D (P04 Heating Ramp D(r, T(t)), G=0): 1500 °C & 1600 °C", "GROUP A  ·  STAGE D (RAMP)  ·  CS-137  ·  GRAPH B OVERLAY")
    add_pair_graph_slide("A23", "A24", "Stage D (P04 Heating Ramp D(r, T(t)), G=0): 1700 °C & 1800 °C", "GROUP A  ·  STAGE D (RAMP)  ·  PHYSICALLY COMPARABLE VALIDATION")

    # 5. Method B B01-B06 (3 slides x 2) + C01 & C02 Verification slides
    idx_mb_slides = [len(prs.slides) + k for k in range(3)]
    add_pair_graph_slide("B01", "B02", "Method B (Sturm–Liouville) vs Method A & Experiment: 1200 & 1350 °C", "GROUP B  ·  ANALYTICAL VERIFICATION + EXPERIMENTAL OVERLAY")
    add_pair_graph_slide("B03", "B04", "Method B (Sturm–Liouville) vs Method A & Experiment: 1500 & 1600 °C", "GROUP B  ·  ANALYTICAL VERIFICATION + EXPERIMENTAL OVERLAY")
    add_pair_graph_slide("B05", "B06", "Method B (Sturm–Liouville) vs Method A & Experiment: 1700 & 1800 °C", "GROUP B  ·  ANALYTICAL VERIFICATION + EXPERIMENTAL OVERLAY")

    idx_c01_slide = len(prs.slides)
    s_c01 = add_clean_slide(
        "C01: BTCS Spatial & Temporal Convergence and 66-Run Mass Balance",
        "GROUP C  ·  NUMERICAL VERIFICATION  ·  CONVERGENCE & CONSERVATION",
        "Phase 3 C01 outputs: spatial order 2.014, temporal order 1.077, max |eps|/bound = 0.0498 <= 1.0 across 66 runs.",
    )
    s_c01.shapes.add_picture(str(GRAPHS / "C01.png"), Inches(0.55), Inches(1.66), width=Inches(12.20))
    tb_c01 = s_c01.shapes.add_textbox(Inches(0.68), Inches(5.88), Inches(11.95), Inches(1.10))
    tb_c01.text_frame.word_wrap = True
    r_c01 = tb_c01.text_frame.paragraphs[0].add_run()
    r_c01.text = (
        f"• Observed Spatial Order = {c01['last_spatial_order']:.3f} (~O(Δr²)) | Observed Temporal Order = {c01['last_temporal_order']:.3f} (~O(Δt)).\n"
        f"• All {len(mb)} audited Cs-137 runs (Stages A–D, P06, Controlled matched-inventory) satisfy tridiagonal dominance (min margin = 1.0) and close mass conservation (max |ε|/bound = {mb['max_residual_over_bound'].max():.4f} ≤ 1.0)."
    )
    r_c01.font.size = Pt(11.0)
    r_c01.font.color.rgb = DARK

    idx_c02_slide = len(prs.slides)
    s_c02 = add_clean_slide(
        "C02: Method A (BTCS) vs Method B (Sturm–Liouville) Numerical Agreement",
        "GROUP C  ·  NUMERICAL VS ANALYTICAL VERIFICATION  ·  1200–1800 °C",
        "Stage A (P01) max |F_A - F_B| <= 1.51e-3 pp; Method B applicability strictly enforced by drawing mask & TIME_VARYING_TEMPERATURE.",
    )
    s_c02.shapes.add_picture(str(GRAPHS / "C02.png"), Inches(1.35), Inches(1.64), height=Inches(4.15))
    tb_c02 = s_c02.shapes.add_textbox(Inches(0.68), Inches(5.95), Inches(11.95), Inches(1.00))
    tb_c02.text_frame.word_wrap = True
    r_c02 = tb_c02.text_frame.paragraphs[0].add_run()
    r_c02.text = (
        f"Stage A agreement across all 6 temperatures: max |F_A − F_B| ≤ {max(x['max_abs_diff_pp'] for x in c02['comparisons']):.2e} pp. "
        "Across all 24 constant-temperature (P01/P02/P03/P05) slots, 9 are drawn (all 6 P01 + 1800 °C P02/P03/P05) while 15 extension cases are honestly withheld by the analytical drawing mask; ramp configs P04/P06–P08 are mathematically inapplicable (TIME_VARYING_TEMPERATURE)."
    )
    r_c02.font.size = Pt(10.8)
    r_c02.font.color.rgb = DARK

    # 6. D06 slide replacement + Option B slide + D01-D05 experimental comparison slides
    idx_d06_slide = len(prs.slides)
    s_d06 = add_clean_slide(
        "D06: AGR-1 Irradiation Exploration (~955–1136 °C, 620.2 EFPD)",
        "GROUP D  ·  AGR-1 IRRADIATION  ·  DEVIATION UNAVAILABLE",
        "Collin (2015); Parsons et al. (2024). Published AGR-1 compact PIE lacks comparable intact-particle Cs-137 time series.",
    )
    s_d06.shapes.add_picture(str(GRAPHS / "D06.png"), Inches(0.65), Inches(1.70), width=Inches(12.00))

    idx_optb_slide = len(prs.slides)
    s_optb = add_clean_slide(
        "Website Option B: Eight Independent Physics Configurations (P01–P08)",
        "INTERACTIVE MODELLING  ·  INDEPENDENT CHECKBOXES & OVERLAYS",
        "Maps website configurations P01-P08 to Stages A-D and summarizes 81 planned entries (48 MA + 24 MB + 6 Exp + 3 Verif).",
    )
    optb_rows = [
        ["ID", "Gen (+G)", "Decay (-λC)", "T(t) Dep.", "Research Stage Equivalent", "Method B Status (1200–1800 °C)"],
        ["P01", "Off", "Off", "Off", "Stage A (Baseline)", "Drawn at all 6 T (<= 1.51e-3 pp)"],
        ["P02", "On", "Off", "Off", "Stage B (+G, Fresh F_gen)", "Drawn 1800 °C; 5 T masked"],
        ["P03", "Off", "On", "Off", "Stage C-decay (G=0, +λ, F_init)", "Drawn 1800 °C; 5 T masked"],
        ["P04", "Off", "Off", "On", "Stage D (Heating ramp G=0)", "Withheld (TIME_VARYING_TEMP)"],
        ["P05", "On", "On", "Off", "Stage C (+G, +λ, Fresh F_gen)", "Drawn 1800 °C; 5 T masked"],
        ["P06", "On", "Off", "On", "Transient +G (no decay)", "Withheld (TIME_VARYING_TEMP)"],
        ["P07", "Off", "On", "On", "Stage D (+λ, ramp G=0)", "Withheld (TIME_VARYING_TEMP)"],
        ["P08", "On", "On", "On", "Stage D Combined (All active)", "Withheld (TIME_VARYING_TEMP)"],
    ]
    tbl_opt = s_optb.shapes.add_table(len(optb_rows), len(optb_rows[0]), Inches(0.68), Inches(1.66), Inches(11.95), Inches(3.60)).table
    for i, row in enumerate(optb_rows):
        for j, v in enumerate(row):
            c = tbl_opt.cell(i, j)
            c.text = v
            r = c.text_frame.paragraphs[0].runs[0]
            r.font.size = Pt(10.0 if i == 0 else 9.5)
            r.font.bold = (i == 0)
            r.font.color.rgb = WHITE if i == 0 else DARK
    tb_opt = s_optb.shapes.add_textbox(Inches(0.68), Inches(5.45), Inches(11.95), Inches(1.45))
    tb_opt.text_frame.word_wrap = True
    ro = tb_opt.text_frame.paragraphs[0].add_run()
    ro.text = (
        "• Explore Controls: 'Select a Modelling Stage' locks Baseline Diffusion On and lets users toggle Generation, Decay, and Temperature dependence freely, paired with Species-Specific Physics ('None — Future development').\n"
        "• Independent Overlays & Scenarios: Three independent checkboxes (Method A — BTCS, Method B — Sturm–Liouville, Experimental data) + 5 operating scenarios (81 entries + prescribed-input Cs-137/Sr-90 S1–S5 trajectories)."
    )
    ro.font.size = Pt(10.8)
    ro.font.color.rgb = DARK

    idx_d12_slide = len(prs.slides)
    add_pair_graph_slide("D01", "D02", "Experimental Comparison D01 & D02: 1200 °C and 1350 °C (Upper Bounds)", "GROUP D  ·  STAGES A–D OVERLAY  ·  BULLOCK (1984) / TE-2090 TABLE 22")

    idx_d3_slide = len(prs.slides)
    s_d3 = add_clean_slide(
        "D03: 1500 °C Cs-137 Release vs Experiment (Bullock / TE-2090 Fig. 124a)",
        "GROUP D  ·  1500 °C EXPERIMENTAL COMPARISON  ·  NO AGR-1 1500 °C CS DATA",
        "Bullock (1984); IAEA-TECDOC-2090 (2025) Fig. 124a & Table 22. Degraded UO2/UC2 grey curves are non-comparable (coating failure).",
    )
    s_d3.shapes.add_picture(str(GRAPHS / "D03.png"), Inches(0.55), Inches(1.66), width=Inches(8.15))
    tb_d3 = s_d3.shapes.add_textbox(Inches(8.90), Inches(1.68), Inches(3.95), Inches(5.10))
    tb_d3.text_frame.word_wrap = True
    rd3 = tb_d3.text_frame.paragraphs[0].add_run()
    rd3.text = (
        "1500 °C Endpoint (t_final = 11,866 h)\n"
        "Reference: Bullock UCO 5% bound.\n\n"
        "• Stage A (P01): 18.68%\n  Δ vs bound = +13.682 pp\n"
        "• Stage C-dec (P03): 18.35%\n  Δ vs bound = +13.348 pp\n"
        "• Stage D ramp (P04): 18.69%\n  Δ vs bound = +13.685 pp\n"
        "• Stage B* (P02, F_gen): 7.86%\n  Δ vs bound = +2.861 pp\n"
        "• Stage C* (P05, F_gen): 7.75%\n  Δ vs bound = +2.755 pp\n\n"
        "All intact G=0 stages exceed both UCO (5%) and UO2*(1) (0.01%) bounds; grey UO2/UC2 points experienced coating failure (Bullock p. 311)."
    )
    rd3.font.size = Pt(10.6)
    rd3.font.color.rgb = DARK

    idx_d4_slide = len(prs.slides)
    s_d4 = add_clean_slide(
        "D04: 1700 °C Cs-137 Release vs Minato et al. (2000) HRB-22 ACT3",
        "GROUP D  ·  1700 °C EXPERIMENTAL VALIDATION (INTACT UO2; NOT AGR-1)",
        "Minato et al. (2000) Nucl. Technol. 131(1), 36–47 (Fig. 2 / TE-2090 Fig. 125). t_final = 271 h, F_exp = 5.456%.",
    )
    s_d4.shapes.add_picture(str(GRAPHS / "D04.png"), Inches(0.55), Inches(1.64), height=Inches(5.30))
    tb_d4 = s_d4.shapes.add_textbox(Inches(7.70), Inches(1.68), Inches(5.15), Inches(5.10))
    tb_d4.text_frame.word_wrap = True
    rd4 = tb_d4.text_frame.paragraphs[0].add_run()
    rd4.text = (
        "ACT3 (N=8, Intact; t_final = 271 h, F_exp = 5.456%)\n\n"
        "Comparable G=0 Anneal Models:\n"
        "• Stage A (P01): Δ = −0.120 pp (RMSE 0.240 pp)\n"
        "• Stage C-dec (P03): Δ = −0.123 pp (RMSE 0.241 pp, −0.23%)\n"
        "• Stage D ramp (P04): Δ = −0.035 pp (RMSE 0.226 pp, +6.08% better)\n\n"
        "Controlled Matched-Inventory Isolation:\n"
        "• Controlled +G (common C_init & F_tot): RMSE 0.255 pp (Δ = −0.185 pp)\n\n"
        "Non-Equivalent Fresh-Start Generation (*F_gen):\n"
        "• Stage B (P02): Δ = −3.775 pp (RMSE 1.970 pp)\n"
        "• Stage C (P05): Δ = −3.776 pp (RMSE 1.971 pp)"
    )
    rd4.font.size = Pt(10.8)
    rd4.font.color.rgb = DARK

    idx_d5_slide = len(prs.slides)
    s_d5 = add_clean_slide(
        "D05: 1800 °C Cs-137 Release vs Minato et al. (2000) HRB-22 ACT4",
        "GROUP D  ·  1800 °C BOUNDED COMPARISON (w = 1/25 FAILED SiC; NOT AGR-1)",
        "Minato et al. (2000) Nucl. Technol. 131(1), 36–47 (Fig. 3 / TE-2090 Fig. 126). t_final = 222 h, F_exp = 14.710%.",
    )
    s_d5.shapes.add_picture(str(GRAPHS / "D05.png"), Inches(0.55), Inches(1.64), height=Inches(5.30))
    tb_d5 = s_d5.shapes.add_textbox(Inches(7.70), Inches(1.68), Inches(5.15), Inches(5.10))
    tb_d5.text_frame.word_wrap = True
    rd5 = tb_d5.text_frame.paragraphs[0].add_run()
    rd5.text = (
        "ACT4 (N=11, w=1/25 failed SiC; t_final = 222 h, F_exp = 14.710%)\n\n"
        "Comparable G=0 Anneal Models:\n"
        "• Stage A (P01): Δ = +12.847 pp (RMSE 7.519 pp)\n"
        "• Method B (Stage A): Δ = +12.849 pp\n"
        "• Stage C-dec (P03): Δ = +12.838 pp (RMSE 7.516 pp, +0.05%)\n"
        "• Stage D ramp (P04): Δ = +13.132 pp (RMSE 7.706 pp, −2.48% worse)\n\n"
        "Controlled Matched-Inventory Isolation:\n"
        "• Controlled +G: Δ = +12.624 pp (RMSE 7.423 pp)\n\n"
        "Non-Equivalent Fresh-Start Generation (*F_gen):\n"
        "• Stage B / C (P02 / P05): Δ = −2.324 / −2.327 pp\n"
        "(Lower RMSE arises from Fresh start + F_gen, not true anneal accuracy.)"
    )
    rd5.font.size = Pt(10.6)
    rd5.font.color.rgb = DARK

    # 7. Final Four Slides: (N-3) Experimental Deviation Summary, (N-2) Scientific Conclusions, (N-1) 104, (N) 105
    idx_dev_sum = len(prs.slides)
    s_dev = add_clean_slide(
        "Experimental Deviation Summary",
        "CONCLUSIONS  ·  PHASE 3 CANONICAL ENDPOINT DEVIATIONS (1200–1800 °C)",
        "Generated from data/study39/endpoint_deviations.csv, deviations_exact_summary.csv, and C03.png.",
    )
    ep_a = ep_df[(ep_df["method"] == "Method A") & (ep_df["config_id"].isin(["P01", "P04", "P05"]))]
    dev_rows = [["Temp", "Dataset / Kind", "P01 (Stage A) Δ", "P04 (Stage D) Δ", "P05 (Stage C*) Δ", "Comparability"]]
    for tc, dshort, cls in [
        (1200, "Bullock UCO <5% (10,138 h)", "Upper bound (G=0)"),
        (1350, "Bullock UCO <5% (10,685 h)", "Upper bound (G=0)"),
        (1500, "Bullock UCO <5% (11,866 h)", "Exceeds bound (+13.7 pp)"),
        (1600, "HFR-K3/1 <0.0016% (378 h)", "Exceeds bound (+0.24 pp)"),
        (1700, "Minato ACT3 N=8 (271 h)", "Comparable (+6.08% RMSE)"),
        (1800, "Minato ACT4 w=1/25 (222 h)", "Bounded (−2.48% RMSE)"),
    ]:
        d1 = float(ep_a[(ep_a["temperature_c"] == tc) & (ep_a["config_id"] == "P01")]["delta_pp"].iloc[0])
        d4 = float(ep_a[(ep_a["temperature_c"] == tc) & (ep_a["config_id"] == "P04")]["delta_pp"].iloc[0])
        d5 = float(ep_a[(ep_a["temperature_c"] == tc) & (ep_a["config_id"] == "P05")]["delta_pp"].iloc[0])
        dev_rows.append([f"{tc} °C", dshort, f"{d1:+.3f} pp", f"{d4:+.3f} pp", f"{d5:+.3f} pp", cls])
    tbl_d = s_dev.shapes.add_table(len(dev_rows), len(dev_rows[0]), Inches(0.45), Inches(1.66), Inches(7.15), Inches(3.25)).table
    for i, row in enumerate(dev_rows):
        for j, v in enumerate(row):
            c = tbl_d.cell(i, j)
            c.text = v
            r = c.text_frame.paragraphs[0].runs[0]
            r.font.size = Pt(8.8 if i == 0 else 8.4)
            r.font.bold = (i == 0)
            r.font.color.rgb = WHITE if i == 0 else DARK
    s_dev.shapes.add_picture(str(GRAPHS / "C03.png"), Inches(7.72), Inches(1.75), width=Inches(5.18))
    tb_dev = s_dev.shapes.add_textbox(Inches(0.45), Inches(5.15), Inches(12.40), Inches(1.80))
    tb_dev.text_frame.word_wrap = True
    rdv = tb_dev.text_frame.paragraphs[0].add_run()
    rdv.text = (
        "• Graph C03 (right) & Endpoint Summary: Progressive physics (Stages A → D) produces non-monotonic accuracy shifts across intact post-irradiation tests.\n"
        "• D06 (955–1136 °C, AGR-1 620.2 EFPD): Experimental deviation UNAVAILABLE (no comparable intact-particle Cs-137 irradiation series).\n"
        "• Controlled Matched-Inventory Isolation: Switching +G on under identical C_init and F_tot leaves 1700 °C RMSE at 0.255 pp vs 0.240 pp, confirming Stage B/C shifts reflect Fresh-start F_gen."
    )
    rdv.font.size = Pt(10.5)
    rdv.font.color.rgb = DARK

    idx_conc = len(prs.slides)
    s_conc = add_clean_slide(
        "Scientific Conclusions",
        "CONCLUSIONS  ·  SOLVER VERIFICATION & CENTRAL RESEARCH QUESTION",
        "Phase 3 unified conclusion integrating verification, endpoint deviation analysis, and controlled isolation.",
    )
    tb_cc = s_conc.shapes.add_textbox(Inches(0.68), Inches(1.72), Inches(11.95), Inches(5.20))
    tb_cc.text_frame.word_wrap = True
    cc_bullets = [
        f"1. Numerical Verification Established (Distinct from Experimental Validation): Method A BTCS achieves observed spatial order {c01['last_spatial_order']:.3f} (~2) and temporal order {c01['last_temporal_order']:.3f} (~1), satisfies strict mass conservation across all {len(mb)} audited runs (max |ε|/bound = {mb['max_residual_over_bound'].max():.4f} ≤ 1), and agrees with Method B Sturm–Liouville within ≤ {max(x['max_abs_diff_pp'] for x in c02['comparisons']):.2e} pp.",
        "2. Answer to Central Research Question: Progressive physics incorporation produces modest, non-monotonic accuracy changes on comparable post-irradiation data: Stage D (ramp D(r, T(t))) improves 1700 °C HRB-22 ACT3 RMSE by +6.08% (0.240 → 0.226 pp; final Δ = −0.035 pp) yet worsens 1800 °C HRB-22 ACT4 overprediction by −2.48% (7.519 → 7.706 pp; final Δ = +13.132 pp).",
        "3. Controlled Comparability Matters: Radioactive decay alters furnace-anneal Cs-137 fractions by < 0.09% at 1700–1800 °C; Fresh-start Stage B/C (+G, F_gen) must not be misread as physical improvement over G=0 annealing data.",
        "4. Dominant Discrepancy Drivers: Unmodelled UO2 surrogate kernel diffusivities, specimen geometry/burnup mismatch, unit interface partition coefficients (k = 1), SiC microstructural/irradiation degradation, and discrete coating failure dominate remaining deviations.",
    ]
    for k, btxt in enumerate(cc_bullets):
        p = tb_cc.text_frame.paragraphs[0] if k == 0 else tb_cc.text_frame.add_paragraph()
        p.space_after = Pt(14)
        run = p.add_run()
        run.text = btxt
        run.font.size = Pt(13.0)
        run.font.color.rgb = DARK

    # Assemble clean final slide order using 0-based indices of original 105-slide deck
    # Keep: 0..19 (orig 1..20), A01..A06, 31..34 (orig 32..35 Stage B theory), A07..A12,
    #       36..38 (orig 37..39 Stage C theory), A13..A18, 41..43 (orig 42..44 Stage D theory), A19..A24,
    #       46..51 (orig 47..52 Method B theory), idx_c01, 58 (orig 59 refinement), idx_c02, B01..B06,
    #       63..94 (orig 64..95 scenarios + AGR1 conditions/measurements), idx_d06, idx_optb,
    #       idx_d12, idx_d3, idx_d4, idx_d5, 102 (orig 103 What Explains?),
    #       [FINAL 4]: idx_dev_sum, idx_conc, 103 (orig 104 Limitations), 104 (orig 105 Future Research)
    kept_order = (
        list(range(0, 20))
        + idx_a_slides
        + list(range(31, 35))
        + idx_b_slides
        + list(range(36, 39))
        + idx_c_slides
        + list(range(41, 44))
        + idx_d_slides
        + list(range(46, 52))
        + [idx_c01_slide, 58, idx_c02_slide]
        + idx_mb_slides
        + list(range(63, 95))
        + [idx_d06_slide, idx_optb_slide, idx_d12_slide, idx_d3_slide, idx_d4_slide, idx_d5_slide, 102]
        + [idx_dev_sum, idx_conc, 103, 104]
    )
    keep_set = set(kept_order)
    all_total = len(prs.slides)
    # First reorder putting kept_order first, then drop unwanted tail slides safely
    drop_indices = [i for i in range(all_total) if i not in keep_set]
    reorder_slides(prs, kept_order + drop_indices)
    for _ in drop_indices:
        delete_slide(prs, len(prs.slides) - 1)

    # Renumber slide number badges 2..N sequentially
    for num, sl in enumerate(prs.slides, start=1):
        if num == 1:
            continue
        for sh in sl.shapes:
            if sh.has_text_frame and sh.left.inches > 11.8 and sh.top.inches < 1.2:
                sh.text_frame.text = str(num)
                for p in sh.text_frame.paragraphs:
                    p.alignment = PP_ALIGN.RIGHT
                    for r in p.runs:
                        r.font.size = Pt(12)
                        r.font.color.rgb = SLATE

    prs.save(OUT_PPTX)

    # QA verification
    prs_v = Presentation(OUT_PPTX)
    n = len(prs_v.slides)
    all_text = "\n".join(
        sh.text
        for sl in prs_v.slides
        for sh in sl.shapes
        if sh.has_text_frame
    )
    missing_ids = [gid for gid in [f"A{i:02d}" for i in range(1, 25)] + [f"B{i:02d}" for i in range(1, 7)] + ["C01", "C02", "C03", "D01", "D02", "D03", "D04", "D05", "D06"] if gid not in all_text]
    reg_hits = sum(1 for sl in prs_v.slides for sh in sl.shapes if sh.has_text_frame and "REGISTERED" in sh.text)
    empty_notes = sum(1 for sl in prs_v.slides if not sl.has_notes_slide or not sl.notes_slide.notes_text_frame.text.strip())
    tail = []
    for k in range(n - 5, n):
        sl = prs_v.slides[k]
        t = next((sh.text.strip().replace("\n", " ") for sh in sl.shapes if sh.has_text_frame and sh.text.strip() and sh.top.inches > 0.35), "")
        tail.append((k + 1, t))
    print(f"Saved {OUT_PPTX} ({n} slides, {OUT_PPTX.stat().st_size:,} B)")
    print(f"Missing 39-graph IDs: {missing_ids} | REGISTERED tags: {reg_hits} | Empty notes: {empty_notes}")
    print("Last 5 slides:")
    for num, t in tail:
        print(f"  Slide {num}: {t}")


if __name__ == "__main__":
    main()
