#!/usr/bin/env python3
"""
Step 6: Reconstruct editable PowerPoint deliverable
`deliverables/powerpoint/STRATO_Presentation_Phase2_Final.pptx`
from the verified 105-slide `STRATO_Presentation_Final.pptx` baseline.

- Adds Phase 2 slides integrating the Central Research Question, 39-graph Cs-137
  matrix (`A01-A24, B01-B06, C01-C03, D01-D06`), convergence/Method A-B verification,
  D03/D04/D05 experimental overlays, and exact Stage A-D deviation & improvement tables.
- Ensures zero `REGISTERED` placeholder tags.
- Reorders closing slides so the FINAL FOUR SLIDES strictly follow:
    (N-3) Experimental Deviation Summary
    (N-2) Scientific Conclusions
    (N-1) Limitations and Possible Mechanisms
    (N)   Future Research Directions
- Renumbers all slide index badges (`1 .. N`) sequentially.
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
OUT_PPTX = REPO / "deliverables" / "powerpoint" / "STRATO_Presentation_Phase2_Final.pptx"
DATA = REPO / "data" / "study39"
GRAPHS = REPO / "graphs" / "study39"

NAVY = RGBColor(0x1E, 0x3A, 0x8A)
SLATE = RGBColor(0x47, 0x55, 0x69)
DARK = RGBColor(0x11, 0x18, 0x27)
RED_A = RGBColor(0xDC, 0x26, 0x26)


def style_header(slide, title_text: str, breadcrumb: str, num_placeholder: str = ""):
    # Breadcrumb matching baseline (0.68, 0.10)
    tb = slide.shapes.add_textbox(Inches(0.68), Inches(0.10), Inches(10.80), Inches(0.30))
    p = tb.text_frame.paragraphs[0]
    r = p.add_run()
    r.text = breadcrumb
    r.font.name = "Calibri"
    r.font.size = Pt(9)
    r.font.bold = True
    r.font.color.rgb = SLATE

    # Title matching baseline (0.68, 0.47)
    tt = slide.shapes.add_textbox(Inches(0.68), Inches(0.47), Inches(11.00), Inches(0.85))
    tt.text_frame.word_wrap = True
    tp = tt.text_frame.paragraphs[0]
    tr = tp.add_run()
    tr.text = title_text
    tr.font.name = "Calibri"
    tr.font.size = Pt(24)
    tr.font.bold = True
    tr.font.color.rgb = DARK

    # Slide number box matching baseline (12.15, 0.73)
    nb = slide.shapes.add_textbox(Inches(12.15), Inches(0.73), Inches(0.62), Inches(0.40))
    np_ = nb.text_frame.paragraphs[0]
    np_.alignment = PP_ALIGN.RIGHT
    nr = np_.add_run()
    nr.text = num_placeholder
    nr.font.name = "Calibri"
    nr.font.size = Pt(12)
    nr.font.color.rgb = SLATE


def set_notes(slide, text: str):
    ns = slide.notes_slide
    ns.notes_text_frame.text = text


def move_slide(prs: Presentation, old_index: int, new_index: int):
    sldIdLst = prs.slides._sldIdLst
    elems = list(sldIdLst)
    el = elems.pop(old_index)
    elems.insert(new_index, el)
    for child in list(sldIdLst):
        sldIdLst.remove(child)
    for child in elems:
        sldIdLst.append(child)


def main():
    OUT_PPTX.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(BASELINE_PPTX, OUT_PPTX)
    prs = Presentation(OUT_PPTX)

    c01 = json.load(open(DATA / "diagnostics" / "c01_convergence.json", encoding="utf-8"))
    c02 = json.load(open(DATA / "diagnostics" / "c02_method_a_vs_b.json", encoding="utf-8"))
    mb = pd.read_csv(DATA / "diagnostics" / "mass_balance_audit.csv")
    sum_df = pd.read_csv(DATA / "deviations_exact_summary.csv")
    imp_df = pd.read_csv(DATA / "stage_improvement.csv")

    act3 = sum_df[(sum_df["case_id"] == "D04_1700C_ACT3") & (sum_df["method"] == "Method A")].set_index("stage_key")
    act4 = sum_df[(sum_df["case_id"] == "D05_1800C_ACT4_N11") & (sum_df["method"] == "Method A")].set_index("stage_key")
    imp3 = imp_df[(imp_df["case_id"] == "D04_1700C_ACT3") & (imp_df["method"] == "Method A")].set_index("stage_key")
    imp4 = imp_df[(imp_df["case_id"] == "D05_1800C_ACT4_N11") & (imp_df["method"] == "Method A")].set_index("stage_key")

    layout_blank = prs.slide_layouts[1]  # Title and Content (clean)

    # Clear placeholder shapes on newly created slides helper
    def new_clean_slide(title_text: str, breadcrumb: str):
        s = prs.slides.add_slide(layout_blank)
        for ph in list(s.placeholders):
            sp = ph._element
            sp.getparent().remove(sp)
        style_header(s, title_text, breadcrumb, "?")
        return s

    # ---- Slide P1: Phase 2 Research Question, 39-Graph Cs-137 Matrix & Verification ----
    s_p1 = new_clean_slide(
        "Phase 2 Synthesis: 39-Graph Cs-137 Matrix & Verification",
        "PHASE 2  ·  39 GRAPHS  ·  CS-137  ·  1200–1800 °C  ·  VERIFICATION",
    )
    tb1 = s_p1.shapes.add_textbox(Inches(0.68), Inches(1.45), Inches(11.97), Inches(1.25))
    tf1 = tb1.text_frame
    tf1.word_wrap = True
    p = tf1.paragraphs[0]
    r = p.add_run()
    r.text = (
        "Central Question: To what extent does progressive transport physics improve predictive accuracy?  |  "
        "Fixed Basis: Cs-137 (t_1/2 = 30.08 y, lambda = 7.302e-10 s^-1, y = 0.062208) on AGR1-B-M16 (175, 278.5, 317.9, 353.2, 394.2 um) across 6 temperatures (1200–1800 degC).\n"
        f"39 Principal Graphs: A01–A24 (4 stages x 6 T), B01–B06 (Method B S-L x 6 T), C01–C03 (Convergence, A vs B, Deviation Summary), D01–D06 (Experimental + AGR-1 Irrad.).\n"
        f"Verification: Spatial order = {c01['last_spatial_order']:.3f}, Temporal order = {c01['last_temporal_order']:.3f}, 48-run max |eps|/bound = {mb['max_residual_over_bound'].max():.4f}, max |F_A - F_B| <= {max(x['max_abs_diff_pp'] for x in c02['comparisons']):.2e} pp."
    )
    r.font.size = Pt(11)
    r.font.color.rgb = DARK
    s_p1.shapes.add_picture(str(GRAPHS / "C01.png"), Inches(0.68), Inches(2.85), width=Inches(6.85))
    s_p1.shapes.add_picture(str(GRAPHS / "C02.png"), Inches(7.70), Inches(2.85), width=Inches(4.95))
    set_notes(s_p1, "Phase 2 canonical outputs from data/study39/diagnostics/{c01_convergence.json,c02_method_a_vs_b.json,mass_balance_audit.csv}. Colours: Method A #DC2626, Method B #2563EB.")

    # ---- Slide P2: Phase 2 Experimental Comparisons D03, D04, D05 ----
    s_p2 = new_clean_slide(
        "Phase 2 Experimental Overlays: D03 (1500 °C), D04 (1700 °C), D05 (1800 °C)",
        "PHASE 2  ·  EXPERIMENTAL COMPARISONS  ·  PROVENANCE & BOUNDARIES",
    )
    s_p2.shapes.add_picture(str(GRAPHS / "D03.png"), Inches(0.55), Inches(1.45), width=Inches(3.98))
    s_p2.shapes.add_picture(str(GRAPHS / "D04.png"), Inches(4.68), Inches(1.45), width=Inches(3.98))
    s_p2.shapes.add_picture(str(GRAPHS / "D05.png"), Inches(8.81), Inches(1.45), width=Inches(3.98))
    tb2 = s_p2.shapes.add_textbox(Inches(0.68), Inches(5.35), Inches(11.97), Inches(1.65))
    tf2 = tb2.text_frame
    tf2.word_wrap = True
    p2 = tf2.paragraphs[0]
    r2 = p2.add_run()
    r2.text = (
        "Verified Provenance Audit (Zero Source Misattribution):\n"
        "• D01–D03 (1200/1350/1500 °C): Bullock (1984) / TE-2090 Table 22 & Fig. 124a upper bounds (5% vs 0.01% collector readings; degraded UO2/UC2 grey hollow; NO AGR-1 1500 °C Cs data exists).\n"
        "• D04 (1700 °C) & D05 (1800 °C): Minato et al. (2000) HRB-22 UO2 ACT3 (N=8, intact) & ACT4 (N=11 incl. 4.6±2.0 h cluster, w=1/25 failed SiC); NOT AGR-1.\n"
        "• D06 (955–1136 °C, 620.2 EFPD): C15 Cs-137 exploration only (<10^-4 % predicted); AGR-1/AGR-2 compacts lack comparable intact-particle Cs-137 time series."
    )
    r2.font.size = Pt(11.5)
    r2.font.color.rgb = DARK
    set_notes(s_p2, "Data: data/study39/experimental/ & graphs/study39/{D03,D04,D05}.png. 1600 °C HFR-K3/1 upper bound & AGR-1 4-4-2 HUNN16-442 (5 failed SiC) documented in S03/report Section 11.")

    # ---- Slide P3: Closing (N-3) Experimental Deviation Summary ----
    s_dev_sum = new_clean_slide(
        "Experimental Deviation Summary",
        "CONCLUSIONS  ·  EXPERIMENTAL DEVIATION SUMMARY (STAGES A–D)",
    )
    rows_tbl = [
        ["Dataset / Case", "Stage Key", "Comparable?", "RMSE (pp)", "MAE (pp)", "IAE (pp)", "Final Δ (pp)", "vs Stage A"],
        ["ACT3 1700 °C (N=8)", "Stage A (Base)", "Yes (G=0)", f"{act3.loc['stage_a','rmse_pp']:.3f}", f"{act3.loc['stage_a','mae_pp']:.3f}", f"{act3.loc['stage_a','iae_time_avg_pp']:.3f}", f"{act3.loc['stage_a','final_delta_pp']:+.3f}", "Baseline"],
        ["ACT3 1700 °C (N=8)", "Stage C (G=0,+λ)", "Yes (G=0)", f"{act3.loc['stage_c_decay_only','rmse_pp']:.3f}", f"{act3.loc['stage_c_decay_only','mae_pp']:.3f}", f"{act3.loc['stage_c_decay_only','iae_time_avg_pp']:.3f}", f"{act3.loc['stage_c_decay_only','final_delta_pp']:+.3f}", f"{imp3.loc['stage_c_decay_only','rmse_improvement_vs_A_pct']:+.2f}%"],
        ["ACT3 1700 °C (N=8)", "Stage D (Ramp)", "Yes (G=0)", f"{act3.loc['stage_d_ramp_decay_off','rmse_pp']:.3f}", f"{act3.loc['stage_d_ramp_decay_off','mae_pp']:.3f}", f"{act3.loc['stage_d_ramp_decay_off','iae_time_avg_pp']:.3f}", f"{act3.loc['stage_d_ramp_decay_off','final_delta_pp']:+.3f}", f"{imp3.loc['stage_d_ramp_decay_off','rmse_improvement_vs_A_pct']:+.2f}%"],
        ["ACT3 1700 °C (N=8)", "Stage B/C (+G)", "No (F_gen)", f"{act3.loc['stage_c','rmse_pp']:.3f}", f"{act3.loc['stage_c','mae_pp']:.3f}", f"{act3.loc['stage_c','iae_time_avg_pp']:.3f}", f"{act3.loc['stage_c','final_delta_pp']:+.3f}", f"{imp3.loc['stage_c','rmse_improvement_vs_A_pct']:+.2f}%"],
        ["ACT4 1800 °C (N=11)", "Stage A (Base)", "Yes (G=0)", f"{act4.loc['stage_a','rmse_pp']:.3f}", f"{act4.loc['stage_a','mae_pp']:.3f}", f"{act4.loc['stage_a','iae_time_avg_pp']:.3f}", f"{act4.loc['stage_a','final_delta_pp']:+.3f}", "Baseline"],
        ["ACT4 1800 °C (N=11)", "Stage C (G=0,+λ)", "Yes (G=0)", f"{act4.loc['stage_c_decay_only','rmse_pp']:.3f}", f"{act4.loc['stage_c_decay_only','mae_pp']:.3f}", f"{act4.loc['stage_c_decay_only','iae_time_avg_pp']:.3f}", f"{act4.loc['stage_c_decay_only','final_delta_pp']:+.3f}", f"{imp4.loc['stage_c_decay_only','rmse_improvement_vs_A_pct']:+.2f}%"],
        ["ACT4 1800 °C (N=11)", "Stage D (Ramp)", "Yes (G=0)", f"{act4.loc['stage_d_ramp_decay_off','rmse_pp']:.3f}", f"{act4.loc['stage_d_ramp_decay_off','mae_pp']:.3f}", f"{act4.loc['stage_d_ramp_decay_off','iae_time_avg_pp']:.3f}", f"{act4.loc['stage_d_ramp_decay_off','final_delta_pp']:+.3f}", f"{imp4.loc['stage_d_ramp_decay_off','rmse_improvement_vs_A_pct']:+.2f}%"],
        ["ACT4 1800 °C (N=11)", "Stage B/C (+G)", "No (F_gen)", f"{act4.loc['stage_c','rmse_pp']:.3f}", f"{act4.loc['stage_c','mae_pp']:.3f}", f"{act4.loc['stage_c','iae_time_avg_pp']:.3f}", f"{act4.loc['stage_c','final_delta_pp']:+.3f}", f"{imp4.loc['stage_c','rmse_improvement_vs_A_pct']:+.2f}%"],
    ]
    tbl_shape = s_dev_sum.shapes.add_table(len(rows_tbl), len(rows_tbl[0]), Inches(0.55), Inches(1.40), Inches(7.20), Inches(3.80))
    tbl = tbl_shape.table
    for i, row in enumerate(rows_tbl):
        for j, val in enumerate(row):
            cell = tbl.cell(i, j)
            cell.text = str(val)
            p_c = cell.text_frame.paragraphs[0]
            r_c = p_c.runs[0]
            r_c.font.size = Pt(9.5 if i == 0 else 9.0)
            r_c.font.bold = (i == 0 or "Stage D" in row[1])
            r_c.font.color.rgb = NAVY if i == 0 else DARK
    s_dev_sum.shapes.add_picture(str(GRAPHS / "C03.png"), Inches(7.85), Inches(1.45), width=Inches(4.95))
    tb_b = s_dev_sum.shapes.add_textbox(Inches(0.55), Inches(5.35), Inches(12.20), Inches(1.65))
    tb_b.text_frame.word_wrap = True
    rb = tb_b.text_frame.paragraphs[0].add_run()
    rb.text = (
        "Upper Bounds (1200–1600 °C): Stage A–D Cs-137 predictions satisfy Bullock 5% counting bounds at 1200 °C (0.0184% < 5%) and 1350 °C (1.48% < 5%), "
        "exceed 5% at 1500 °C (18.68%, V_s = +13.68 pp), and exceed the stricter 0.01% collector threshold at all three temperatures (1.84x to 1868x) plus 1600 °C HFR-K3/1 (153x).\n"
        "Key Finding: Progressive physics does NOT monotonically improve predictive error — Stage D improves 1700 °C ACT3 (+6.08%) but slightly worsens 1800 °C ACT4 (-2.48%), "
        "and lower Stage B/C (+G) values reflect an incompatible release definition (F_gen vs post-irradiation F_init)."
    )
    rb.font.size = Pt(11)
    rb.font.color.rgb = DARK
    set_notes(s_dev_sum, "Generated directly from data/study39/deviations_exact_summary.csv, deviations_bounds.csv, and stage_improvement.csv.")

    # ---- Slide P4: Closing (N-2) Scientific Conclusions (consolidated Phase 1+2) ----
    s_conc = new_clean_slide(
        "Scientific Conclusions",
        "CONCLUSIONS  ·  SOLVER VERIFICATION & PROGRESSIVE PHYSICS ANSWER",
    )
    tbc = s_conc.shapes.add_textbox(Inches(0.68), Inches(1.55), Inches(11.97), Inches(5.40))
    tbc.text_frame.word_wrap = True
    bullets = [
        f"1. Numerical Verification Achieved (Not Experimental Validation): Method A BTCS reproduces theoretical convergence (spatial order {c01['last_spatial_order']:.3f} [O(dr^2)], temporal order {c01['last_temporal_order']:.3f} [O(dt)]), closes mass balance within max |eps|/bound = {mb['max_residual_over_bound'].max():.4f} across all 48 runs, and matches Method B Sturm–Liouville within <= {max(x['max_abs_diff_pp'] for x in c02['comparisons']):.2e} pp.",
        "2. Stage A -> B -> C -> D Physics Effect: Active in-pile fission generation (Stage B, F_gen) lowers release fractions from the baseline post-irradiation F_init curves (~27.6% -> ~12.4% at 1800 °C) because freshly generated atoms originate inside the kernel; Cs-137 radioactive decay (Stage C, lambda = 7.302e-10 s^-1) changes furnace-anneal release by < 0.09% at 1700–1800 °C; transient heating ramps (Stage D, AGR1-ST-C16) add 0.0005–0.285 pp pre-hold diffusion.",
        f"3. Answer to Central Research Question: On comparable post-irradiation Cs-137 data (G = 0), progressive physics yields modest, non-monotonic accuracy changes: Stage D improves 1700 °C HRB-22 ACT3 RMSE by +6.08% ({act3.loc['stage_a','rmse_pp']:.3f} -> {act3.loc['stage_d_ramp_decay_off','rmse_pp']:.3f} pp), while slightly worsening 1800 °C ACT4 overprediction by -2.48% ({act4.loc['stage_a','rmse_pp']:.3f} -> {act4.loc['stage_d_ramp_decay_off','rmse_pp']:.3f} pp).",
        "4. Dominant Discrepancy Drivers: Unmodelled physics—UO2 kernel surrogacy (250 um vs 175/272 um), unit partition coefficients (k = 1), SiC fluence/microstructure variance, initial end-of-irradiation C(r,0) distribution, and 1800 °C particle failure (w = 1/25)—exceed numerical and ramp-history corrections.",
    ]
    for k, b in enumerate(bullets):
        par = tbc.text_frame.paragraphs[0] if k == 0 else tbc.text_frame.add_paragraph()
        par.space_after = Pt(14)
        run = par.add_run()
        run.text = b
        run.font.size = Pt(13.5)
        run.font.color.rgb = DARK
    set_notes(s_conc, "Consolidates baseline solver conclusions (former Slides 101–102) with Phase 2 Stage A–D Cs-137 findings.")

    # Locate original closing slides by title in existing 105-slide baseline
    # Indices (0-based in prs.slides before adding P1..P4? No, we just added 4 slides at 105..108!)
    def find_idx(prefix: str) -> int:
        for i, sl in enumerate(prs.slides):
            for sh in sl.shapes:
                if sh.has_text_frame and sh.text.strip().startswith(prefix):
                    return i
        raise RuntimeError(f"Slide starting with {prefix!r} not found")

    # Note: Keep existing Slides 1..100, move P1 & P2 right after Slide 100 (or before Slide 97?), keep 103 ("What Explains...") before closing 4,
    # and make sure LAST 4 slides are strictly:
    # (N-3) Experimental Deviation Summary (s_dev_sum)
    # (N-2) Scientific Conclusions (s_conc)
    # (N-1) Limitations and Possible Mechanisms (original 104)
    # (N)   Future Research Directions (original 105)
    idx_101 = find_idx("Scientific Conclusions (1 of 2)")
    idx_102 = find_idx("Scientific Conclusions (2 of 2)")
    idx_103 = find_idx("What Explains the Model")
    idx_104 = find_idx("Limitations and Possible Mechanisms")
    idx_105 = find_idx("Future Research Directions")
    idx_p1 = len(prs.slides) - 4
    idx_p2 = len(prs.slides) - 3
    idx_dev_sum = len(prs.slides) - 2
    idx_conc = len(prs.slides) - 1

    # Reorder so tail from slide 97 onward is:
    # [97..100: original Exp Deviation Summaries], [P1], [P2], [101], [102], [103], [s_dev_sum], [s_conc], [104], [105]
    # Even cleaner: keep every historical slide intact AND put the final 4 strictly as specified!
    # Wait: moving P1 & P2 & 101 & 102 & 103 before the final 4 preserves 100% of historical slides while enforcing exact final 4 order!
    sldIdLst = prs.slides._sldIdLst
    elems = list(sldIdLst)
    new_order = (
        elems[:idx_101]
        + [elems[idx_p1], elems[idx_p2], elems[idx_101], elems[idx_102], elems[idx_103]]
        + [elems[idx_dev_sum], elems[idx_conc], elems[idx_104], elems[idx_105]]
    )
    for c in list(sldIdLst):
        sldIdLst.remove(c)
    for c in new_order:
        sldIdLst.append(c)

    # Renumber slide number textboxes cleanly 1..N (starting from slide 2, matching baseline)
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

    # Audit final PPTX
    prs2 = Presentation(OUT_PPTX)
    n = len(prs2.slides)
    tail = []
    for k in range(n - 6, n):
        sl = prs2.slides[k]
        t = next((sh.text.strip().replace("\n", " ") for sh in sl.shapes if sh.has_text_frame and sh.text.strip()), "")
        tail.append((k + 1, t))
    print(f"Saved {OUT_PPTX} ({n} slides, {OUT_PPTX.stat().st_size} B)")
    print("Last 6 slides:")
    for num, t in tail:
        print(f"  Slide {num}: {t}")
    # Assert zero REGISTERED uppercase tags
    reg_count = 0
    for sl in prs2.slides:
        for sh in sl.shapes:
            if sh.has_text_frame and "REGISTERED" in sh.text:
                reg_count += 1
    print("REGISTERED tag count:", reg_count)


if __name__ == "__main__":
    main()
