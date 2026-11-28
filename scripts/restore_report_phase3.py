#!/usr/bin/env python3
"""
Step 4 (Phase 3): Restore original pre-Phase-2 STRATO Research Report formatting,
typography, native Word OMML equations, 21 algorithm blocks, Rust implementation details,
and Sections 1-10 + Appendices A-B while surgically integrating:
  - Section 1.5 Literature Review (BISON, PARFUME, FV, FD, Analytical, Mechanistic models)
  - Updated Section 6 Verification Figures C01 & C02 + 66-run Cs-137 audit Table 6a
  - Replaced Section 7.4 Figures 11-18 using verified Phase 3 Graph B figures
    (D01, D02, D03, S03 1600C, D04, D05, D06, C03) with experimental overlays & single Delta
  - Section 7.6 Phase 3 Tables 11a, 11b, 11c (all 6 T endpoint deviations, exact metrics, controlled runs)
  - Updated Section 10 Conclusion & IAEA-TECDOC-2090 reference
  - Appendix C: Full Canonical 39-Graph Cs-137 Research Inventory (A01-A24, B01-B06 plates)

Input master (read-only):
  C:\\Users\\Alf\\Downloads\\STRATO_Archive\\STRATO_Archive\\deliverables\\report\\STRATO_Research_Report_Final.docx
Output:
  deliverables/report/STRATO_Research_Report_Phase3_Final.docx
"""

import json
import shutil
from pathlib import Path

import pandas as pd
from docx import Document
from docx.oxml import OxmlElement
from docx.oxml.ns import qn
from docx.shared import Inches, Pt, RGBColor
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.enum.table import WD_TABLE_ALIGNMENT

REPO = Path(__file__).resolve().parent.parent
ORIG_DOCX = Path(
    r"C:\Users\Alf\Downloads\STRATO_Archive\STRATO_Archive\deliverables\report\STRATO_Research_Report_Final.docx"
)
OUT_DOCX = REPO / "deliverables" / "report" / "STRATO_Research_Report_Phase3_Final.docx"
DATA = REPO / "data" / "study39"
GRAPHS = REPO / "graphs" / "study39"


def set_cell_shading(cell, fill_hex: str):
    tcPr = cell._tc.get_or_add_tcPr()
    shd = OxmlElement("w:shd")
    shd.set(qn("w:val"), "clear")
    shd.set(qn("w:color"), "auto")
    shd.set(qn("w:fill"), fill_hex)
    tcPr.append(shd)


def set_row_props(row, is_header: bool = False):
    trPr = row._tr.get_or_add_trPr()
    cant = OxmlElement("w:cantSplit")
    trPr.append(cant)
    if is_header:
        hdr = OxmlElement("w:tblHeader")
        trPr.append(hdr)


def set_table_borders(table):
    tblPr = table._tbl.tblPr
    borders = OxmlElement("w:tblBorders")
    for b_name, val, sz, col in [
        ("top", "single", "8", "1E3A8A"),
        ("bottom", "single", "8", "1E3A8A"),
        ("insideH", "single", "4", "CBD5E1"),
        ("left", "none", "0", "auto"),
        ("right", "none", "0", "auto"),
        ("insideV", "none", "0", "auto"),
    ]:
        b = OxmlElement(f"w:{b_name}")
        b.set(qn("w:val"), val)
        if val != "none":
            b.set(qn("w:sz"), sz)
            b.set(qn("w:space"), "0")
            b.set(qn("w:color"), col)
        borders.append(b)
    tblPr.append(borders)


def insert_para_before(anchor_p, text: str = "", style_name: str = "Body Text"):
    p = anchor_p.insert_paragraph_before(text)
    try:
        p.style = anchor_p.part.document.styles[style_name]
    except KeyError:
        pass
    return p


def insert_figure_before(anchor_p, img_path: Path, caption_text: str, width_in: float = 5.85, citation_text: str | None = None):
    p_img = insert_para_before(anchor_p, "", "Captioned Figure")
    p_img.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.CENTER
    p_img.paragraph_format.keep_with_next = True
    run = p_img.add_run()
    run.add_picture(str(img_path), width=Inches(width_in))

    p_cap = insert_para_before(anchor_p, caption_text, "Image Caption")
    if citation_text:
        p_cap.paragraph_format.keep_with_next = True
        insert_para_before(anchor_p, citation_text, "Image Caption")
    return p_img


def insert_table_before(anchor_p, doc, caption_text: str, headers: list[str], rows: list[list[str]], col_widths_in: list[float]):
    p_cap = insert_para_before(anchor_p, caption_text, "Table Caption")
    p_cap.paragraph_format.keep_with_next = True

    tbl = doc.add_table(rows=len(rows) + 1, cols=len(headers))
    tbl.alignment = WD_TABLE_ALIGNMENT.CENTER
    try:
        tbl.style = doc.styles["Table"]
    except KeyError:
        pass
    set_table_borders(tbl)

    # Header
    hdr_row = tbl.rows[0]
    set_row_props(hdr_row, is_header=True)
    for j, h in enumerate(headers):
        cell = hdr_row.cells[j]
        cell.width = Inches(col_widths_in[j])
        set_cell_shading(cell, "EFF6FF")
        p = cell.paragraphs[0]
        p.paragraph_format.space_before = Pt(2)
        p.paragraph_format.space_after = Pt(2)
        r = p.add_run(h)
        r.bold = True
        r.font.size = Pt(8.5)
        r.font.color.rgb = RGBColor(0x1E, 0x3A, 0x8A)

    for i, rdata in enumerate(rows):
        row = tbl.rows[i + 1]
        set_row_props(row, is_header=False)
        for j, val in enumerate(rdata):
            cell = row.cells[j]
            cell.width = Inches(col_widths_in[j])
            if i % 2 == 1:
                set_cell_shading(cell, "F8FAFC")
            p = cell.paragraphs[0]
            p.paragraph_format.space_before = Pt(1.5)
            p.paragraph_format.space_after = Pt(1.5)
            run = p.add_run(str(val))
            run.font.size = Pt(8.0)

    # Move table XML directly before anchor_p
    anchor_p._p.addprevious(tbl._tbl)
    spacer = insert_para_before(anchor_p, "", "Normal")
    spacer.paragraph_format.space_after = Pt(4)
    return tbl


def remove_paragraph(p):
    elem = p._element
    elem.getparent().remove(elem)


def main():
    OUT_DOCX.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(ORIG_DOCX, OUT_DOCX)
    doc = Document(OUT_DOCX)

    # Load verified Phase 3 canonical data
    c01 = json.load(open(DATA / "diagnostics" / "c01_convergence.json", encoding="utf-8"))
    c02 = json.load(open(DATA / "diagnostics" / "c02_method_a_vs_b.json", encoding="utf-8"))
    mb = pd.read_csv(DATA / "diagnostics" / "mass_balance_audit.csv")
    ep_df = pd.read_csv(DATA / "endpoint_deviations.csv")
    sum_df = pd.read_csv(DATA / "deviations_exact_summary.csv")
    imp_df = pd.read_csv(DATA / "stage_improvement.csv")
    manifest = json.load(open(DATA / "study39_manifest.json", encoding="utf-8"))

    # Locate anchors by text prefix
    def find_p(prefix: str, start: int = 0):
        for p in doc.paragraphs[start:]:
            if p.text.strip().startswith(prefix):
                return p
        raise RuntimeError(f"Anchor paragraph starting with {prefix!r} not found")

    # 1. Update Abstract cleanly while keeping style & detail
    p_abs = find_p("ABSTRACT:")
    p_abs.text = (
        "ABSTRACT: Fission products formed in the kernel of a TRISO fuel particle must traverse four surrounding "
        "coating layers—porous carbon buffer, inner pyrolytic carbon (IPyC), silicon carbide (SiC), and outer pyrolytic "
        "carbon (OPyC)—before reaching the primary coolant. This report formulates a five-layer spherical diffusion model "
        "solved numerically using a backward-time, central-space (BTCS) finite-difference scheme (Method A) and verified "
        "against an independent analytical Sturm–Liouville eigenfunction expansion (Method B). Building on the validated "
        "five-layer Rust and WebAssembly codebase, a canonical 39-graph Cs-137 progressive-physics investigation (A01–D06) "
        "and an eight-configuration independent website study (P01–P08) examine Baseline diffusion, fission-product "
        "generation (+G), radioactive decay (−λC), and transient temperature-dependent diffusivity D(r, T(t)) across six "
        "heating temperatures (1200–1800 °C) and AGR-1 irradiation conditions (955–1136 °C, 620.2 EFPD). Numerical "
        "verification confirms second-order spatial convergence (observed order "
        f"{c01['last_spatial_order']:.3f}), first-order temporal convergence ({c01['last_temporal_order']:.3f}), "
        f"strict mass balance across all {len(mb)} audited runs (max |ε|/bound = {mb['max_residual_over_bound'].max():.4f} ≤ 1), "
        f"and analytical agreement within {max(x['max_abs_diff_pp'] for x in c02['comparisons']):.2e} pp for Stage A. "
        "Experimental comparisons evaluate Method A and Method B against a fixed final experimental endpoint per "
        "temperature using Δ = F_model(t_final) − F_exp(t_final) (or Δ vs upper bound), demonstrating that added "
        "physics yields modest, non-monotonic accuracy shifts (+6.08% RMSE improvement on 1700 °C HRB-22 ACT3 vs −2.48% "
        "change on 1800 °C HRB-22 ACT4) while unmodelled microstructural, chemical, and batch-failure effects remain dominant."
    )

    # 2. Update Section 1.4 organisation note & insert Section 1.5 Literature Review right before Section 2
    p_org = find_p("The report is organised as follows.")
    p_org.text = (
        p_org.text.rstrip(".")
        + "; Section 1.5 reviews the TRISO transport modelling literature (BISON, PARFUME, finite-volume, finite-difference, "
        "analytical, and mechanistic approaches); and Appendix C compiles the full 39-graph Cs-137 progressive-physics inventory (A01–D06)."
    )

    p_sec2 = find_p("2 Baseline Spherical Diffusion Model")
    insert_para_before(p_sec2, "1.5 Literature Review: TRISO Fission-Product Transport Codes and Modelling Approaches", "Heading 2")
    insert_para_before(
        p_sec2,
        "Quantitative safety assessment of High-Temperature Gas-Cooled Reactors (HTGRs) relies on multi-layer transport "
        "solvers capable of tracking volatile fission products across five distinct concentric material domains. Across the "
        "international literature, production and research codes fall into five complementary paradigms that motivate STRATO’s design.",
        "First Paragraph",
    )
    insert_para_before(
        p_sec2,
        "BISON (Idaho National Laboratory). BISON is a parallel, multidimensional finite-element nuclear fuel performance "
        "code built on the MOOSE framework (Hales et al., 2021; Parsons et al., 2024). It couples thermomechanical stress, "
        "pyrolytic carbon irradiation shrinkage, cracking, gap conductance, and species diffusion in 1D spherical, 2D axisymmetric, "
        "or 3D particle geometries using effective Arrhenius diffusivities. Unlike BISON, STRATO intentionally decouples "
        "transport from continuum mechanics so that the pure five-layer parabolic operator, its discrete conservation properties, "
        "and the exact contribution of each transport term (+G, −λC, D(r, T(t))) can be isolated, analytically benchmarked, "
        "and executed deterministically in microseconds to milliseconds via native Rust and browser WebAssembly.",
        "Body Text",
    )
    insert_para_before(
        p_sec2,
        "PARFUME (Idaho National Laboratory). PARFUME (Miller et al., 2018; Collin et al., 2015, 2016) serves as the standard "
        "statistical TRISO fuel performance code used to evaluate AGR-1 and AGR-2 irradiation and heating-test data. It combines "
        "closed-form thermo-mechanical stress solutions, Weibull batch failure statistics, and a 1D spherical finite-difference "
        "diffusion solver evaluated over IAEA-TECDOC-978 effective diffusivities (Bullock, 1984; IAEA, 1997; Collin, 2016). "
        "STRATO adopts the exact AGR-1 Baseline geometry (Morris et al., 2016) and Collin (2016) C16/C15 diffusivity registers, "
        "but extends verification rigour through provable rounding bounds on every time step, explicit exact-arithmetic checks "
        "against Sturm–Liouville eigenvalues, and strict separation between numerical verification and experimental comparability.",
        "Body Text",
    )
    insert_para_before(
        p_sec2,
        "Finite-Volume and Finite-Difference Spherical Transport Methods. Cell-centred finite-volume (FV) methods integrate Fick’s "
        "divergence theorem over spherical shells r_{i-1/2} to r_{i+1/2}, guaranteeing exact local and global flux conservation across "
        "steep diffusivity jumps (10³–10⁶× across buffer/IPyC/SiC interfaces) via harmonic diffusivity averaging (Schappel et al., 2020). "
        "Vertex-centred finite-difference schemes, such as STRATO Method A (BTCS), instead place discrete nodes directly on every "
        "material interface and enforce flux continuity over symmetric control volumes straddling the interface (Section 3.2). Both "
        "yield unconditionally stable, strictly diagonally dominant tridiagonal systems solved in O(N) operations via the Thomas "
        "algorithm; STRATO’s interface-node formulation preserves second-order spatial accuracy O(Δr²) without requiring half-cell boundary approximations.",
        "Body Text",
    )
    insert_para_before(
        p_sec2,
        "Analytical Multilayer Diffusion Methods. Closed-form analytical solvers such as Gelbard’s eigenfunction formulation "
        "(Gelbard, 2003) and Laplace-transform series resolve multi-region spherical diffusion free of spatial discretisation error. "
        "STRATO Method B implements the Sturm–Liouville eigenfunction series in orthogonal weighted inner-product space and extends "
        "it cleanly to first-order radioactive decay (B2) and piecewise-constant kernel generation (B3–B5). Crucially, STRATO guards "
        "every analytical output with a verified drawing mask (withholding early-time Gibbs ringing and ill-conditioned cover "
        "regions) and proves mathematically why separation of variables fails when D(r, T(t)) varies in time, demarcating "
        "the exact boundary where numerical solvers become indispensable.",
        "Body Text",
    )
    insert_para_before(
        p_sec2,
        "Mechanistic TRISO Transport Models. Recent atomistic, phase-field, and multiphysics investigations—particularly for "
        "Ag-110m—demonstrate that homogeneous Arrhenius diffusion neglects polycrystalline SiC grain-boundary percolation "
        "(Gerczak et al., 2016; Simon et al., 2022), trapping and detrapping at irradiation defects (Szlufarska et al., 2016), "
        "palladium-assisted SiC corrosion (Lillo & van Rooyen, 2015; Minato et al., 2000), non-unit interface partition coefficients "
        "(k ≠ 1), and discrete through-layer coating fracture (Demkowicz et al., 2016). In Section 9 and on the STRATO website, "
        "these mechanisms are explicitly catalogued as unimplemented future physics rather than parameter-fitted into effective diffusivities.",
        "Body Text",
    )

    # 3. Section 6.2: Add C01, C02 and Table 6a right before Section 6.3
    p_sec63 = find_p("6.3 Status of the Verified Properties")
    insert_para_before(
        p_sec63,
        "To verify the Phase 3 canonical Cs-137 study matrix before comparing against experimental datasets, Method A was executed "
        f"across all {len(mb)} runs (Stages A–D, P06, and controlled matched-inventory simulations over 1200–1800 °C). Every single run "
        f"satisfied all tridiagonal dominance bounds (min row margin = 1.0, min pivot margin = 1.0) and closed discrete mass conservation "
        f"with max |ε|/bound = {mb['max_residual_over_bound'].max():.4f} ≤ 1.0 (Figures 10a–10b, Table 6a).",
        "Body Text",
    )
    insert_figure_before(
        p_sec63,
        GRAPHS / "C01.png",
        f"Figure 10a (C01): BTCS spatial convergence (observed order {c01['last_spatial_order']:.3f}), temporal convergence ({c01['last_temporal_order']:.3f}), and {len(mb)}-run mass-balance audit (max |ε|/bound = {mb['max_residual_over_bound'].max():.4f}).",
        width_in=6.1,
    )
    insert_figure_before(
        p_sec63,
        GRAPHS / "C02.png",
        f"Figure 10b (C02): Method A (BTCS, red) vs Method B (Sturm–Liouville, blue) end-of-test Stage A agreement and maximum discrepancy across 1200–1800 °C (max |F_A − F_B| ≤ {max(x['max_abs_diff_pp'] for x in c02['comparisons']):.2e} pp).",
        width_in=6.0,
    )
    t6a_rows = []
    for r in c02["comparisons"]:
        t6a_rows.append([
            f"{int(r['temperature_c'])} °C",
            "P01 (Stage A)",
            f"{r['final_f_a']*100:.4g}%",
            f"{r['final_f_b']*100:.4g}%" if r.get("final_f_b") is not None else "N/A",
            f"{r['max_abs_diff_pp']:.2e}",
            f"≥ {r['valid_from_h']:.3g} h",
            f"{r['accuracy_floor']*100:.2e}%",
        ])
    insert_table_before(
        p_sec63,
        doc,
        "Table 6a: Phase 3 Cs-137 Stage A numerical-analytical agreement (Method A vs Method B across 1200–1800 °C; C02).",
        ["Temp", "Config", "Method A", "Method B", "Max |Δ| (pp)", "B Valid From", "B Floor"],
        t6a_rows,
        [0.65, 0.95, 0.90, 0.90, 0.95, 0.85, 0.85],
    )

    # 4. Section 7.4: Replace outdated Figures 11-18 cleanly with canonical Graph B figures D01, D02, D03, S03, D04, D05, D06, C03
    p_74_intro = find_p("Comparison figures. Figure")
    p_74_intro.text = (
        "Comparison figures (Phase 3 Graph B suite). Figures 11 to 18 present the verified Cs-137 experimental overlays "
        "from the canonical 39-graph matrix on a unified Graph B basis: Time (hours) against Cumulative Cs-137 release (%), "
        "Method A BTCS in red, Method B Sturm–Liouville in blue, experimental measurements as black circles, experimental "
        "upper bounds as black downward triangles, and non-comparable degraded batches as hollow grey circles. At each "
        "temperature, all modelling stages are compared against one immutable final experimental endpoint t_final using "
        "exactly one signed endpoint difference Δ = F_model(t_final) − F_exp(t_final) (or Δ vs upper bound) per curve."
    )

    # Remove old paragraphs 286..312 (old Figures 11-18 and intermediate notes inside 7.4) right up to "7.5 Bounds and Their Premises"
    p_75 = find_p("7.5 Bounds and Their Premises")
    idx_start = None
    idx_end = None
    for idx, p in enumerate(doc.paragraphs):
        if p._p is p_74_intro._p:
            idx_start = idx + 1
        elif p._p is p_75._p:
            idx_end = idx
            break
    to_remove = list(doc.paragraphs[idx_start:idx_end])
    for rp in to_remove:
        remove_paragraph(rp)

    phase3_sec74 = [
        ("D01.png",
         "Figure 11 (D01): Cs-137 cumulative release across Stages A–D at 1200 °C overlaid against Bullock (1984) / IAEA-TECDOC-2090 Table 22 upper bounds (t_final = 10,138 h; U_exp = 5% UCO counting bound, 0.01% collector threshold).",
         "Bullock, R. E. (1984). J. Nucl. Mater., 125(3), 304–319; IAEA-TECDOC-2090 (2025), Table 22, p. 82."),
        ("D02.png",
         "Figure 12 (D02): Cs-137 cumulative release across Stages A–D at 1350 °C overlaid against Bullock (1984) / IAEA-TECDOC-2090 Table 22 upper bounds (t_final = 10,685 h; U_exp = 5% UCO bound).",
         "Bullock, R. E. (1984). J. Nucl. Mater., 125(3), 304–319; IAEA-TECDOC-2090 (2025), Table 22, p. 82."),
        ("D03.png",
         "Figure 13 (D03): Cs-137 cumulative release across Stages A–D at 1500 °C overlaid against Bullock (1984) / IAEA-TECDOC-2090 Fig. 124a & Table 22 bounds (t_final = 11,866 h; degraded UO2/UC2 curves shown grey N/C; no AGR-1 1500 °C Cs data exist).",
         "Bullock, R. E. (1984). J. Nucl. Mater., 125(3), 304–319; IAEA-TECDOC-2090 (2025), Fig. 124a, p. 212."),
        ("S03.png",
         "Figure 14 (S03): Cs-137 release at 1600 °C (Stages A and D) overlaid against IAEA-TECDOC-978 HFR-K3/1 sphere upper bounds (t_final ≤ 400 h; U_exp = 0.0016%) and non-comparable JAERI Sample A context.",
         "International Atomic Energy Agency (1997). IAEA-TECDOC-978; no AGR-1 intact Cs-137 curve exists at 1600 °C."),
        ("D04.png",
         "Figure 15 (D04): Cs-137 release at 1700 °C across Stages A–D overlaid against Minato et al. (2000) HRB-22 ACT3 intact UO2 particles (N = 8, t_final = 271 h, F_exp = 5.456%; NOT AGR-1), with pointwise residuals Δ(t_i).",
         "Minato, K., et al. (2000). Nucl. Technol., 131(1), 36–47 (Fig. 2 / TE-2090 Fig. 125)."),
        ("D05.png",
         "Figure 16 (D05): Cs-137 release at 1800 °C across Stages A–D overlaid against Minato et al. (2000) HRB-22 ACT4 (N = 11 incl. 4.6 h cluster, w = 1/25 failed SiC, t_final = 222 h, F_exp = 14.710%; NOT AGR-1), with residuals Δ(t_i).",
         "Minato, K., et al. (2000). Nucl. Technol., 131(1), 36–47 (Fig. 3 / TE-2090 Fig. 126)."),
        ("D06.png",
         "Figure 17 (D06): AGR-1 irradiation-temperature exploration (~955–1136 °C, 620.2 EFPD = 14,884.8 h, C15 Cs diffusivities). Experimental deviation unavailable because published AGR-1 PIE records lack comparable intact-particle Cs-137 irradiation time series.",
         "Collin, B. P. (2015). INL/EXT-10-18097; Parsons et al. (2024). J. Nucl. Mater., 598, 155139."),
        ("C03.png",
         "Figure 18 (C03): Unified Cs-137 final-endpoint deviation summary across Stages A–D (P01–P05) for exact measurements (1700–1800 °C) and upper-bound differences (1200–1600 °C).",
         "Compiled from data/study39/endpoint_deviations.csv across all six canonical temperatures."),
    ]
    for fn, cap, cit in phase3_sec74:
        insert_figure_before(p_75, GRAPHS / fn, cap, width_in=5.8, citation_text=cit)

    # 5. Section 7.6 Result Analysis: Insert Tables 11a, 11b, 11c right before Heading 1: 8 Five Operating-Scenario Cases
    p_sec8 = find_p("8 Five Operating-Scenario Cases")
    insert_para_before(
        p_sec8,
        "Phase 3 Progressive-Physics Endpoint Deviation and Controlled Analysis. Table 11a compiles the signed final-endpoint "
        "differences Δ = F_model(t_final) − F_exp(t_final) (and Δ vs upper bound) evaluated at the single immutable experimental "
        "endpoint for each temperature 1200–1800 °C across Configurations P01–P08 (Stages A–D). Table 11b reports the full "
        "time-series metrics (MAE, RMSE, max |Δ|, IAE) on the two exact measurement sets (ACT3 1700 °C and ACT4 1800 °C), and "
        "Table 11c isolates +G and +λ cleanly under matched initial inventory C_init = g_ref × 620.2 EFPD and common total denominator.",
        "First Paragraph",
    )

    # Table 11a: Endpoint summary for all 6 T x P01..P08 (Method A)
    ep_a = ep_df[(ep_df["method"] == "Method A") & (ep_df["config_id"].str.startswith("P"))].copy()
    t11a_rows = []
    for _, r in ep_a.iterrows():
        v_txt = f"{r['violation_pp']:.3f}" if pd.notna(r["violation_pp"]) else "N/A"
        cls_short = (
            "Upper-bound (G=0)" if "Upper-bound comparison" in r["comparison_class"]
            else "UB raw (*F_gen)" if "Upper-bound raw" in r["comparison_class"]
            else "Comparable (ACT3)" if "validation (G=0" in r["comparison_class"]
            else "Bounded (ACT4 w=1/25)" if "bounded" in r["comparison_class"]
            else "Raw diff (*F_gen)"
        )
        t11a_rows.append([
            f"{int(r['temperature_c'])} °C",
            f"{r['config_id']} ({r['stage_alias'].split('(')[0].strip()})",
            f"{r['t_final_h']:.0f}" if r["t_final_h"] >= 100 else f"{r['t_final_h']:.1f}",
            f"{r['f_exp_or_u_pct']:.3f}%",
            f"{r['f_model_pct']:.4g}%",
            f"{r['delta_pp']:+.3f}",
            v_txt,
            cls_short,
        ])
    insert_table_before(
        p_sec8,
        doc,
        "Table 11a: Method A final-endpoint deviations Δ (pp) and upper-bound violations V = max(0, Δ_bound) across P01–P08 (1200–1800 °C).",
        ["Temp", "Config (Stage)", "t_final (h)", "Exp/Bound", "Model", "Δ (pp)", "V (pp)", "Class"],
        t11a_rows,
        [0.52, 1.35, 0.62, 0.72, 0.72, 0.68, 0.55, 1.34],
    )

    # Table 11b: Exact series metrics for 1700 C ACT3 and 1800 C ACT4
    t11b_rows = []
    for cid_case, lbl in [("D04_1700C_ACT3", "ACT3 1700 °C (N=8)"), ("D05_1800C_ACT4_N11", "ACT4 1800 °C (N=11)")]:
        sub_i = imp_df[(imp_df["case_id"] == cid_case) & (imp_df["method"] == "Method A")]
        sub_s = sum_df[(sum_df["case_id"] == cid_case) & (sum_df["method"] == "Method A")].set_index("stage_key")
        for sk_keep, sname in [
            ("stage_a", "P01 / Stage A (Base)"),
            ("stage_b", "P02 / Stage B* (+G, F_gen)"),
            ("stage_c_decay_only", "P03 / Stage C-dec (G=0, +λ)"),
            ("stage_d_ramp_decay_off", "P04 / Stage D (Ramp T(t))"),
            ("stage_c", "P05 / Stage C* (+G, +λ, F_gen)"),
            ("p06_gen_temp", "P06* (+G, T(t), F_gen)"),
            ("stage_d_ramp_decay_on", "P07 / Stage D (+λ, T(t))"),
            ("stage_d_combined", "P08 / Stage D* (All, F_gen)"),
        ]:
            ir = sub_i[sub_i["stage_key"] == sk_keep].iloc[0]
            sr = sub_s.loc[sk_keep]
            t11b_rows.append([
                lbl,
                sname,
                f"{ir['mae_pp']:.3f}",
                f"{ir['rmse_pp']:.3f}",
                f"{ir['max_abs_delta_pp']:.3f}",
                f"{ir['iae_time_avg_pp']:.3f}",
                f"{ir['final_delta_pp']:+.3f}",
                f"{ir['rmse_improvement_vs_A_pct']:+.2f}%",
            ])
    insert_table_before(
        p_sec8,
        doc,
        "Table 11b: Exact time-series deviation metrics on HRB-22 ACT3 (1700 °C, intact) and ACT4 (1800 °C, w=1/25 failed SiC); * denotes F_gen denominator.",
        ["Dataset", "Config / Stage", "MAE (pp)", "RMSE (pp)", "Max |Δ|", "IAE (pp)", "Final Δ", "RMSE vs A"],
        t11b_rows,
        [1.05, 1.55, 0.62, 0.62, 0.62, 0.62, 0.68, 0.74],
    )

    # Table 11c: Controlled matched-inventory isolation at 1700 & 1800 C
    t11c_rows = []
    for tc, cname in [(1700, "D04_1700C_ACT3"), (1800, "D05_1800C_ACT4_N11")]:
        sub_s = sum_df[(sum_df["case_id"] == cname) & (sum_df["method"] == "Method A")].set_index("stage_key")
        for sk, desc in [
            ("stage_a", "Baseline G=0, λ=0 (matched C_init, F=M_rel/(M0+0))"),
            ("ctrl_gen_on_decay_off", "Controlled +G (matched C_init=g_ref*620.2d, F_tot)"),
            ("ctrl_gen_on_decay_on", "Controlled +G, +λ (matched C_init=g_ref*620.2d, F_tot)"),
            ("stage_b", "Uncontrolled Stage B (Fresh C_init=0, F_gen)"),
        ]:
            sr = sub_s.loc[sk]
            t11c_rows.append([
                f"{tc} °C",
                desc,
                f"{sr['f_model_final']*100:.3f}%",
                f"{sr['f_exp_final']*100:.3f}%",
                f"{sr['final_delta_pp']:+.3f}",
                f"{sr['rmse_pp']:.3f}",
            ])
    insert_table_before(
        p_sec8,
        doc,
        "Table 11c: Controlled matched-inventory comparison isolating +G and +λ under identical C_init and total-inventory denominator.",
        ["Temp", "Configuration", "Model End", "Exp End", "Final Δ (pp)", "RMSE (pp)"],
        t11c_rows,
        [0.55, 2.75, 0.75, 0.75, 0.85, 0.85],
    )

    # 6. Update Section 10 Conclusion cleanly (preserving blank Statements immediately after!)
    p_conc = find_p("This work built a five-layer model of fission-product diffusion")
    p_conc.text = (
        "This work restored and extended a five-layer spherical diffusion model of intact TRISO fuel particles solved with "
        "Method A (BTCS finite differences) and verified against Method B (Sturm–Liouville eigenfunction series). Across all "
        f"{len(mb)} audited Cs-137 Phase 3 simulations (Stages A–D, P06, and controlled runs over 1200–1800 °C), Method A "
        f"reproduces second-order spatial convergence ({c01['last_spatial_order']:.3f}) and first-order temporal convergence "
        f"({c01['last_temporal_order']:.3f}), obeys derived rounding balance bounds (max |ε|/bound = {mb['max_residual_over_bound'].max():.4f} ≤ 1), "
        f"and agrees with Method B to within {max(x['max_abs_diff_pp'] for x in c02['comparisons']):.2e} pp for Stage A. "
        "Answering the central research question with the complete 39-graph matrix evaluated against one fixed experimental "
        "endpoint per temperature demonstrates that progressive incorporation of transport physics does not monotonically "
        "improve experimental agreement: transient ramp diffusion D(r, T(t)) improves 1700 °C HRB-22 ACT3 RMSE by +6.08% "
        "(0.240 → 0.226 pp, final Δ = −0.035 pp) while slightly worsening 1800 °C HRB-22 ACT4 overprediction by −2.48% "
        "(7.519 → 7.706 pp, final Δ = +13.132 pp). Radioactive decay changes 1700–1800 °C annealing predictions by < 0.09% "
        "relative, while Stage B/C source-inclusive curves yield lower apparent fractions solely due to the Fresh-start "
        "F_gen denominator—proven via controlled matched-inventory comparisons, where true +G shifts 1700 °C RMSE by only "
        "0.015 pp (0.240 → 0.255 pp). Upper-bound checks satisfy Bullock 5% limits at 1200–1350 °C (0.0184% and 1.48%) "
        "but exceed bounds at 1500 °C (18.68% > 5%, V = +13.685 pp) and 1600 °C HFR-K3/1 (0.310% vs 0.0016%). Model–experiment "
        "discrepancies therefore remain governed by unmodelled specimen mismatch, UO2 surrogate kernel diffusivities, unit "
        "interface partitioning, SiC microstructural/irradiation degradation, and discrete coating failure."
    )

    # 7. Append IAEA-TECDOC-2090 alphabetically to References (right before kurata / right before Appendix A)
    p_kurata = find_p("Kurata, Y.")
    insert_para_before(
        p_kurata,
        "International Atomic Energy Agency. (2025). Coated particle fuels for high temperature gas cooled small modular "
        "reactors: Progress in design, manufacturing, experimentation, modelling and analysis technologies (IAEA-TECDOC-2090). "
        "https://www-pub.iaea.org/MTCD/publications/PDF/TE-2090web.pdf",
        "Reference",
    )

    # 8. Append Appendix C at the very end of the document (after Table 14 of Appendix B)
    doc.add_page_break()
    h_appc = doc.add_paragraph("Appendix C Complete Canonical 39-Graph Cs-137 Research Inventory (A01–D06)")
    h_appc.style = doc.styles["Heading 1"]
    p_appc_intro = doc.add_paragraph(
        "Figures A01–A24 and B01–B06 below complete the canonical 39-graph inventory alongside C01–C02 (Section 6.2) "
        "and D01–D06, C03 (Section 7.4). All plots follow the Graph B standard: Time (hours) vs Cumulative Cs-137 "
        "release (%), Method A red, Method B blue, experimental measurements black circles, upper bounds black downward "
        "triangles, with one final-endpoint Δ annotation per model curve."
    )
    p_appc_intro.style = doc.styles["First Paragraph"]

    # Helper to append figure + caption at document end
    def append_end_figure(gid: str, cap: str):
        p_i = doc.add_paragraph("")
        p_i.style = doc.styles["Captioned Figure"]
        p_i.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.CENTER
        p_i.paragraph_format.keep_with_next = True
        p_i.add_run().add_picture(str(GRAPHS / f"{gid}.png"), width=Inches(5.35))
        p_c = doc.add_paragraph(cap)
        p_c.style = doc.styles["Image Caption"]

    h_ga = doc.add_paragraph("C.1 Group A: Method A BTCS Progressive Matrix (A01–A24)")
    h_ga.style = doc.styles["Heading 2"]
    for g in [x for x in manifest["graphs"] if x["group"] == "A"]:
        gid = g["graph_id"]
        lbl = g.get("endpoint_label") or "Endpoint Δ unavailable"
        append_end_figure(
            gid,
            f"Figure {gid}: {g['title']} ({g['config_id']}, Cs-137) overlaid with {g.get('experimental_endpoint')}. {lbl} [{g.get('comparison_class')}].",
        )

    h_gb = doc.add_paragraph("C.2 Group B: Method B Sturm–Liouville Analytical Comparisons (B01–B06)")
    h_gb.style = doc.styles["Heading 2"]
    for g in [x for x in manifest["graphs"] if x["group"] == "B"]:
        gid = g["graph_id"]
        blbl = g.get("method_b_endpoint_label") or "N/A"
        append_end_figure(
            gid,
            f"Figure {gid}: {g['title']} (Method B vs Method A and {g.get('experimental_endpoint')}). Method B {blbl}; Stage A max |F_A − F_B| = {g.get('method_a_vs_b_stage_a_max_pp', 0.0):.2e} pp.",
        )

    doc.save(OUT_DOCX)
    print(f"Saved restored & updated Phase 3 report to {OUT_DOCX} ({OUT_DOCX.stat().st_size:,} B)")


if __name__ == "__main__":
    main()
