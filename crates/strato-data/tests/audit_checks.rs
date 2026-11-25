//! One test per check of the Python audit scripts: each runs the whole check set once on the real
//! register files and asserts that its own check is present and passes. `CHECKS.md` maps every id to
//! the Python source line it reproduces and to the test below.

mod common;

use common::real_report;

fn passes(id: &str) {
    let r = real_report();
    let c = r
        .get(id)
        .unwrap_or_else(|| panic!("check {id} did not run"));
    assert!(c.passed, "{c}");
}

/// Every id the run produces, in order. A new or removed check must change this list and `CHECKS.md`.
const ALL_IDS: &[&str] = &[
    "D5.S.header.geometry",
    "D5.S.status.geometry",
    "D5.S.ids.geometry",
    "D5.V.evidence.geometry",
    "D5.S.rect.geometry",
    "D5.S.header.arrhenius",
    "D5.S.status.arrhenius",
    "D5.S.ids.arrhenius",
    "D5.V.evidence.arrhenius",
    "D5.S.rect.arrhenius",
    "D5.S.header.temperature",
    "D5.S.status.temperature",
    "D5.S.ids.temperature",
    "D5.V.evidence.temperature",
    "D5.S.rect.temperature",
    "D5.S.header.release",
    "D5.S.status.release",
    "D5.S.ids.release",
    "D5.V.evidence.release",
    "D5.S.rect.release",
    "D5.S.header.decay",
    "D5.S.status.decay",
    "D5.S.ids.decay",
    "D5.V.evidence.decay",
    "D5.S.rect.decay",
    "D5.S.header.yields",
    "D5.S.status.yields",
    "D5.S.ids.yields",
    "D5.V.evidence.yields",
    "D5.S.rect.yields",
    "LOAD.register",
    "LOAD.rows.geometry",
    "LOAD.rows.arrhenius",
    "LOAD.rows.decay",
    "LOAD.rows.yields",
    "LOAD.rows.temperature",
    "LOAD.rows.release",
    "D5.G.identity",
    "D5.G.layers",
    "D5.G.radii",
    "D5.G.crp6",
    "D5.G.agr1",
    "D5.L.uncertainty-formula",
    "D5.L.stable",
    "D5.L.lambda",
    "D5.L.unc-lambda",
    "D5.L.iaea.Cs-137",
    "D5.L.iaea.Sr-90",
    "D5.L.iaea.Ag-110m",
    "D5.F.values",
    "D5.F.limit",
    "D5.F.eq109",
    "D5.K.T71-01",
    "D5.K.T71-02",
    "D5.K.T71-03",
    "D5.K.T71-04",
    "D5.K.T71-05",
    "D5.X.equals",
    "D5.X.hales.H21-04",
    "D5.X.hales.H21-06",
    "D5.X.hales.H21-09",
    "D5.A.frg-values",
    "D5.A.r-sensitivity",
    "D5.B.booth-zero",
    "D5.B.case-1200",
    "D5.B.case-1600",
    "D5.B.single-branch",
    "D5.Y.steady-state",
    "D5.Y.overcount",
    "D5.Y.initial",
    "D5.Y.ode-solution",
    "D5.T.ramps",
    "D5.T.cumulative",
    "D5.T.total.AGR1-ST-C16-1600",
    "D5.T.total.AGR1-ST-C16-1700",
    "D5.T.total.AGR1-ST-C16-1800",
    "D5.T.total.CRP6-6a",
    "D5.T.total.CRP6-7a",
    "D5.T.total.CRP6-7b",
    "D5.T.total.CRP6-8a",
    "D5.T.total.CRP6-8b",
    "D5.T.total.CRP6-9",
    "D5.T.total.CRP6-10",
    "D5.T.total.CRP6-11",
    "D5.E.range01",
    "D5.E.minmax",
    "D5.E.fraction.E-114",
    "D5.E.fraction.E-115",
    "D5.E.fraction.E-116",
    "D5.E.failures-1800",
    "D5.E.one-particle",
    "D5.E.jaeri-count",
    "D5.A.sanity",
    "D5.V.b6-coverage",
    "D5.V.b6-values",
    "D5.V.rg",
    "D5.V.applicability",
    "D5.V.bounds",
    "D5.V.activation-yields",
    "D5.V.fission-sources",
    "B4D.A.stable",
    "B4D.A.seconds",
    "B4D.A.lambda",
    "B4D.A.product",
    "B4D.A.cs137",
    "B4D.A.mean-life",
    "B4D.A.negative",
    "B4D.B.yields",
    "B4D.B.source-term",
    "B4D.B.weighted-yield",
    "B4P.nuclide.Cs-137",
    "B4P.nuclide.Sr-90",
    "B4P.nuclide.Ag-110m",
    "B4P.nuclide.I-131",
    "B4P.source-example",
    "B4P.step-sizes",
    "B6.activation.decay",
    "B6.activation.yields",
    "B6.activation.no-source-term",
    "GEN.geometry.rows",
    "GEN.geometry.radii",
    "GEN.geometry.default",
    "CAT.nuclides",
];

#[test]
fn the_list_of_ids_is_the_list_of_checks_that_run() {
    let got: Vec<&str> = real_report()
        .results
        .iter()
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(got, ALL_IDS);
}

#[test]
fn the_whole_check_set_passes() {
    let r = real_report();
    let failed: Vec<String> = r
        .results
        .iter()
        .filter(|c| !c.passed)
        .map(|c| c.to_string())
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
    assert!(r.all_passed());
}

#[test]
fn d5_s_header_geometry() {
    passes("D5.S.header.geometry");
}

#[test]
fn d5_s_status_geometry() {
    passes("D5.S.status.geometry");
}

#[test]
fn d5_s_ids_geometry() {
    passes("D5.S.ids.geometry");
}

#[test]
fn d5_v_evidence_geometry() {
    passes("D5.V.evidence.geometry");
}

#[test]
fn d5_s_rect_geometry() {
    passes("D5.S.rect.geometry");
}

#[test]
fn d5_s_header_arrhenius() {
    passes("D5.S.header.arrhenius");
}

#[test]
fn d5_s_status_arrhenius() {
    passes("D5.S.status.arrhenius");
}

#[test]
fn d5_s_ids_arrhenius() {
    passes("D5.S.ids.arrhenius");
}

#[test]
fn d5_v_evidence_arrhenius() {
    passes("D5.V.evidence.arrhenius");
}

#[test]
fn d5_s_rect_arrhenius() {
    passes("D5.S.rect.arrhenius");
}

#[test]
fn d5_s_header_temperature() {
    passes("D5.S.header.temperature");
}

#[test]
fn d5_s_status_temperature() {
    passes("D5.S.status.temperature");
}

#[test]
fn d5_s_ids_temperature() {
    passes("D5.S.ids.temperature");
}

#[test]
fn d5_v_evidence_temperature() {
    passes("D5.V.evidence.temperature");
}

#[test]
fn d5_s_rect_temperature() {
    passes("D5.S.rect.temperature");
}

#[test]
fn d5_s_header_release() {
    passes("D5.S.header.release");
}

#[test]
fn d5_s_status_release() {
    passes("D5.S.status.release");
}

#[test]
fn d5_s_ids_release() {
    passes("D5.S.ids.release");
}

#[test]
fn d5_v_evidence_release() {
    passes("D5.V.evidence.release");
}

#[test]
fn d5_s_rect_release() {
    passes("D5.S.rect.release");
}

#[test]
fn d5_s_header_decay() {
    passes("D5.S.header.decay");
}

#[test]
fn d5_s_status_decay() {
    passes("D5.S.status.decay");
}

#[test]
fn d5_s_ids_decay() {
    passes("D5.S.ids.decay");
}

#[test]
fn d5_v_evidence_decay() {
    passes("D5.V.evidence.decay");
}

#[test]
fn d5_s_rect_decay() {
    passes("D5.S.rect.decay");
}

#[test]
fn d5_s_header_yields() {
    passes("D5.S.header.yields");
}

#[test]
fn d5_s_status_yields() {
    passes("D5.S.status.yields");
}

#[test]
fn d5_s_ids_yields() {
    passes("D5.S.ids.yields");
}

#[test]
fn d5_v_evidence_yields() {
    passes("D5.V.evidence.yields");
}

#[test]
fn d5_s_rect_yields() {
    passes("D5.S.rect.yields");
}

#[test]
fn load_register() {
    passes("LOAD.register");
}

#[test]
fn load_rows_geometry() {
    passes("LOAD.rows.geometry");
}

#[test]
fn load_rows_arrhenius() {
    passes("LOAD.rows.arrhenius");
}

#[test]
fn load_rows_decay() {
    passes("LOAD.rows.decay");
}

#[test]
fn load_rows_yields() {
    passes("LOAD.rows.yields");
}

#[test]
fn load_rows_temperature() {
    passes("LOAD.rows.temperature");
}

#[test]
fn load_rows_release() {
    passes("LOAD.rows.release");
}

#[test]
fn d5_g_identity() {
    passes("D5.G.identity");
}

#[test]
fn d5_g_layers() {
    passes("D5.G.layers");
}

#[test]
fn d5_g_radii() {
    passes("D5.G.radii");
}

#[test]
fn d5_g_crp6() {
    passes("D5.G.crp6");
}

#[test]
fn d5_g_agr1() {
    passes("D5.G.agr1");
}

#[test]
fn d5_l_uncertainty_formula() {
    passes("D5.L.uncertainty-formula");
}

#[test]
fn d5_l_stable() {
    passes("D5.L.stable");
}

#[test]
fn d5_l_lambda() {
    passes("D5.L.lambda");
}

#[test]
fn d5_l_unc_lambda() {
    passes("D5.L.unc-lambda");
}

#[test]
fn d5_l_iaea_cs_137() {
    passes("D5.L.iaea.Cs-137");
}

#[test]
fn d5_l_iaea_sr_90() {
    passes("D5.L.iaea.Sr-90");
}

#[test]
fn d5_l_iaea_ag_110m() {
    passes("D5.L.iaea.Ag-110m");
}

#[test]
fn d5_f_values() {
    passes("D5.F.values");
}

#[test]
fn d5_f_limit() {
    passes("D5.F.limit");
}

#[test]
fn d5_f_eq109() {
    passes("D5.F.eq109");
}

#[test]
fn d5_k_t71_01() {
    passes("D5.K.T71-01");
}

#[test]
fn d5_k_t71_02() {
    passes("D5.K.T71-02");
}

#[test]
fn d5_k_t71_03() {
    passes("D5.K.T71-03");
}

#[test]
fn d5_k_t71_04() {
    passes("D5.K.T71-04");
}

#[test]
fn d5_k_t71_05() {
    passes("D5.K.T71-05");
}

#[test]
fn d5_x_equals() {
    passes("D5.X.equals");
}

#[test]
fn d5_x_hales_h21_04() {
    passes("D5.X.hales.H21-04");
}

#[test]
fn d5_x_hales_h21_06() {
    passes("D5.X.hales.H21-06");
}

#[test]
fn d5_x_hales_h21_09() {
    passes("D5.X.hales.H21-09");
}

#[test]
fn d5_a_frg_values() {
    passes("D5.A.frg-values");
}

#[test]
fn d5_a_r_sensitivity() {
    passes("D5.A.r-sensitivity");
}

#[test]
fn d5_b_booth_zero() {
    passes("D5.B.booth-zero");
}

#[test]
fn d5_b_case_1200() {
    passes("D5.B.case-1200");
}

#[test]
fn d5_b_case_1600() {
    passes("D5.B.case-1600");
}

#[test]
fn d5_b_single_branch() {
    passes("D5.B.single-branch");
}

#[test]
fn d5_y_steady_state() {
    passes("D5.Y.steady-state");
}

#[test]
fn d5_y_overcount() {
    passes("D5.Y.overcount");
}

#[test]
fn d5_y_initial() {
    passes("D5.Y.initial");
}

#[test]
fn d5_y_ode_solution() {
    passes("D5.Y.ode-solution");
}

#[test]
fn d5_t_ramps() {
    passes("D5.T.ramps");
}

#[test]
fn d5_t_cumulative() {
    passes("D5.T.cumulative");
}

#[test]
fn d5_t_total_agr1_st_c16_1600() {
    passes("D5.T.total.AGR1-ST-C16-1600");
}

#[test]
fn d5_t_total_agr1_st_c16_1700() {
    passes("D5.T.total.AGR1-ST-C16-1700");
}

#[test]
fn d5_t_total_agr1_st_c16_1800() {
    passes("D5.T.total.AGR1-ST-C16-1800");
}

#[test]
fn d5_t_total_crp6_6a() {
    passes("D5.T.total.CRP6-6a");
}

#[test]
fn d5_t_total_crp6_7a() {
    passes("D5.T.total.CRP6-7a");
}

#[test]
fn d5_t_total_crp6_7b() {
    passes("D5.T.total.CRP6-7b");
}

#[test]
fn d5_t_total_crp6_8a() {
    passes("D5.T.total.CRP6-8a");
}

#[test]
fn d5_t_total_crp6_8b() {
    passes("D5.T.total.CRP6-8b");
}

#[test]
fn d5_t_total_crp6_9() {
    passes("D5.T.total.CRP6-9");
}

#[test]
fn d5_t_total_crp6_10() {
    passes("D5.T.total.CRP6-10");
}

#[test]
fn d5_t_total_crp6_11() {
    passes("D5.T.total.CRP6-11");
}

#[test]
fn d5_e_range01() {
    passes("D5.E.range01");
}

#[test]
fn d5_e_minmax() {
    passes("D5.E.minmax");
}

#[test]
fn d5_e_fraction_e_114() {
    passes("D5.E.fraction.E-114");
}

#[test]
fn d5_e_fraction_e_115() {
    passes("D5.E.fraction.E-115");
}

#[test]
fn d5_e_fraction_e_116() {
    passes("D5.E.fraction.E-116");
}

#[test]
fn d5_e_failures_1800() {
    passes("D5.E.failures-1800");
}

#[test]
fn d5_e_one_particle() {
    passes("D5.E.one-particle");
}

#[test]
fn d5_e_jaeri_count() {
    passes("D5.E.jaeri-count");
}

#[test]
fn d5_a_sanity() {
    passes("D5.A.sanity");
}

#[test]
fn d5_v_b6_coverage() {
    passes("D5.V.b6-coverage");
}

#[test]
fn d5_v_b6_values() {
    passes("D5.V.b6-values");
}

#[test]
fn d5_v_rg() {
    passes("D5.V.rg");
}

#[test]
fn d5_v_applicability() {
    passes("D5.V.applicability");
}

#[test]
fn d5_v_bounds() {
    passes("D5.V.bounds");
}

#[test]
fn d5_v_activation_yields() {
    passes("D5.V.activation-yields");
}

#[test]
fn d5_v_fission_sources() {
    passes("D5.V.fission-sources");
}

#[test]
fn b4d_a_stable() {
    passes("B4D.A.stable");
}

#[test]
fn b4d_a_seconds() {
    passes("B4D.A.seconds");
}

#[test]
fn b4d_a_lambda() {
    passes("B4D.A.lambda");
}

#[test]
fn b4d_a_product() {
    passes("B4D.A.product");
}

#[test]
fn b4d_a_cs137() {
    passes("B4D.A.cs137");
}

#[test]
fn b4d_a_mean_life() {
    passes("B4D.A.mean-life");
}

#[test]
fn b4d_a_negative() {
    passes("B4D.A.negative");
}

#[test]
fn b4d_b_yields() {
    passes("B4D.B.yields");
}

#[test]
fn b4d_b_source_term() {
    passes("B4D.B.source-term");
}

#[test]
fn b4d_b_weighted_yield() {
    passes("B4D.B.weighted-yield");
}

#[test]
fn b4p_nuclide_cs_137() {
    passes("B4P.nuclide.Cs-137");
}

#[test]
fn b4p_nuclide_sr_90() {
    passes("B4P.nuclide.Sr-90");
}

#[test]
fn b4p_nuclide_ag_110m() {
    passes("B4P.nuclide.Ag-110m");
}

#[test]
fn b4p_nuclide_i_131() {
    passes("B4P.nuclide.I-131");
}

#[test]
fn b4p_source_example() {
    passes("B4P.source-example");
}

#[test]
fn b4p_step_sizes() {
    passes("B4P.step-sizes");
}

#[test]
fn b6_activation_decay() {
    passes("B6.activation.decay");
}

#[test]
fn b6_activation_yields() {
    passes("B6.activation.yields");
}

#[test]
fn b6_activation_no_source_term() {
    passes("B6.activation.no-source-term");
}

#[test]
fn gen_geometry_rows() {
    passes("GEN.geometry.rows");
}

#[test]
fn gen_geometry_radii() {
    passes("GEN.geometry.radii");
}

#[test]
fn gen_geometry_default() {
    passes("GEN.geometry.default");
}

#[test]
fn cat_nuclides() {
    passes("CAT.nuclides");
}
