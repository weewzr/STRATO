//! The label texts of the graphs that are generated in Rust (LaTeX and plain), so that the
//! website, the report and the deck show the same words (decisions 30, 53, 54, 64).

use strato_compare::json::J;

use crate::experiment::{record_json, NrmseRecord, A1, BP1, BP2};
use crate::output::{json_file, Prov};

/// The colour of Method A.
pub const COLOUR_A: &str = "#2563EB";
/// The colour of Method B.
pub const COLOUR_B: &str = "#DC2626";
/// The colour of experimental observations.
pub const COLOUR_EXP: &str = "#111111";
/// The colour of the normal-operation reference curve.
pub const COLOUR_REFERENCE: &str = "#6B7280";

fn text(id: &str, plain: &str, latex: &str) -> J {
    J::obj(vec![
        ("id", J::s(id)),
        ("plain", J::s(plain)),
        ("latex", J::s(latex)),
    ])
}

/// The `labels.json` file.
pub fn labels_json(prov: &Prov, records: &[NrmseRecord]) -> String {
    let texts = vec![
        text("method_a", "Method A: BTCS", "\\text{Method A: BTCS}"),
        text(
            "method_b",
            "Method B: Sturm\u{2013}Liouville",
            "\\text{Method B: Sturm\u{2013}Liouville}",
        ),
        text("experiment", "Experimental observations", "\\text{Experimental observations}"),
        text("normal_operation", "Normal operation", "\\text{Normal operation}"),
        text(
            "axis_time_at_temperature",
            "Time at test temperature (h)",
            "\\text{Time at test temperature (h)}",
        ),
        text(
            "axis_fraction_released",
            "Fraction released (\u{2013})",
            "F\\ (\\text{fraction released, dimensionless})",
        ),
        text("axis_time_days", "Time (d)", "t\\ (\\mathrm{d})"),
        text(
            "axis_time_after_event_h",
            "Time after the event (h)",
            "t - t_{\\mathrm{e}}\\ (\\mathrm{h})",
        ),
        text(
            "axis_fraction_generated_released",
            "Fraction of generated inventory released (\u{2013})",
            "F_{\\mathrm{gen}}\\ (\\text{dimensionless})",
        ),
        text(
            "axis_fraction_post_event",
            "Fraction of the inventory at the event released (\u{2013})",
            "F_{\\mathrm{post}}\\ (\\text{dimensionless})",
        ),
        text(
            "axis_inventory",
            "Inventory in one particle (mol)",
            "M\\ (\\mathrm{mol})",
        ),
        text(
            "axis_temperature",
            "Particle temperature (\u{b0}C)",
            "T\\ (^{\\circ}\\mathrm{C})",
        ),
        text(
            "axis_generation",
            "Generation rate density in the kernel (mol m\u{207b}\u{b3} s\u{207b}\u{b9})",
            "G_{\\mathrm{kernel}}\\ (\\mathrm{mol\\,m^{-3}\\,s^{-1}})",
        ),
        text(
            "panel_diffusivity",
            "Diffusivity of each layer at T (m\u{b2} s\u{207b}\u{b9})",
            "D\\ (\\mathrm{m^{2}\\,s^{-1}})",
        ),
        text(
            "panel_decayed_fraction",
            "Decayed amount as a fraction of the initial inventory (\u{2013})",
            "M_{\\mathrm{dec}}/M^{0}\\ (\\text{dimensionless})",
        ),
        text(
            "panel_inventory_vs_normal",
            "Inventory: scenario against normal operation (mol)",
            "M\\ (\\mathrm{mol})",
        ),
        text(
            "unresolved_note",
            "not resolved on the grid",
            "\\text{not resolved on the grid}",
        ),
        text(
            "method_b_withheld",
            "not available: the drawing mask withholds the value",
            "\\text{not available: the drawing mask withholds the value}",
        ),
        text(
            "method_b_time_varying",
            "not available: the temperature changes in time",
            "\\text{not available: the temperature changes in time}",
        ),
        text("bound_marker", "Reported upper bound (calculated compact inventory)", "\\text{Reported upper bound}"),
        text("bp1", BP1, &format!("\\text{{{BP1}}}")),
        text("bp2", BP2, &format!("\\text{{{BP2}}}")),
        text("a1", A1, &format!("\\text{{{A1}}}")),
        text(
            "preview",
            "Preview: not computed by the solver",
            "\\text{Preview: not computed by the solver}",
        ),
        text(
            "scenario_label",
            "prescribed-input simulation; not a validated reactor prediction",
            "\\text{prescribed-input simulation; not a validated reactor prediction}",
        ),
        text(
            "numerical_label",
            "numerical solution of the stated model; not experimental validation",
            "\\text{numerical solution of the stated model; not experimental validation}",
        ),
        text(
            "c15_outside",
            "C15 evaluated outside its documented application (955 to 1197 \u{b0}C)",
            "\\text{C15 evaluated outside its documented application (955 to 1197 }^{\\circ}\\mathrm{C})",
        ),
    ];
    let nr: Vec<J> = records
        .iter()
        .map(|r| {
            J::obj(vec![
                ("set_id", J::s(format!("{}.{}", r.meta.set_id, r.method))),
                ("display", r.display.clone().map_or(J::Null, J::s)),
                (
                    "label_latex",
                    r.display
                        .as_ref()
                        .map_or(J::Null, |d| J::s(crate::nrmse::label(r.method, d).0)),
                ),
                (
                    "label_plain",
                    r.display
                        .as_ref()
                        .map_or(J::Null, |d| J::s(crate::nrmse::label(r.method, d).1)),
                ),
                ("shown_on_graph", J::Bool(r.display.is_some())),
            ])
        })
        .collect();
    let _ = record_json;
    json_file(
        prov,
        "label texts of the graphs generated in Rust",
        vec![
            (
                "colours",
                J::obj(vec![
                    ("method_a_blue", J::s(COLOUR_A)),
                    ("method_b_red", J::s(COLOUR_B)),
                    ("experiment_black", J::s(COLOUR_EXP)),
                    ("normal_operation_grey", J::s(COLOUR_REFERENCE)),
                    (
                        "rule",
                        J::s("Method A blue solid, Method B red solid, same thickness; experimental observations black markers; no dashed or dotted lines; plots use a fixed light surface in both themes"),
                    ),
                ]),
            ),
            (
                "nrmse_display_rule",
                J::s("NRMSE_A = x% in blue and NRMSE_B = y% in red, two significant figures from the stored display string; nothing else on the graph: no N, no uncertainty, no status word, no placeholder where no value exists (metric-nrmse.md Section 6)"),
            ),
            (
                "nrmse_label_templates",
                J::obj(vec![
                    ("latex_a", J::s("\\mathrm{NRMSE}_{\\mathrm{A}} = <display>\\,\\%")),
                    ("latex_b", J::s("\\mathrm{NRMSE}_{\\mathrm{B}} = <display>\\,\\%")),
                    ("plain_a", J::s("NRMSE_A = <display>%")),
                    ("plain_b", J::s("NRMSE_B = <display>%")),
                ]),
            ),
            ("texts", J::Arr(texts)),
            ("nrmse_labels", J::Arr(nr)),
        ],
    )
}
