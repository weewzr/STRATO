//! The label texts of the graphs that are generated in Rust (LaTeX and plain), so that the
//! website, the report and the deck show the same words (decisions 30, 53, 54, 64).

use strato_compare::json::J;

use crate::experiment::{A1, BP1, BP2};
use crate::output::{json_file, Prov};
use crate::r2curve::R2Record;

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

/// The `labels.json` file: the label texts, the R2 labels of the records and the caption record of
/// every curve (`metric-r2.md` Sections 7.3 and 7.4).
pub fn labels_json(prov: &Prov, records: &[R2Record], captions: &[J]) -> String {
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
    let r2: Vec<J> = records
        .iter()
        .map(|r| {
            J::obj(vec![
                ("record_id", J::s(r.record_id.clone())),
                (
                    "status",
                    J::s(if r.available {
                        "available"
                    } else {
                        "unavailable"
                    }),
                ),
                ("reason_code", r.reason.map_or(J::Null, |c| J::s(c.code()))),
                ("display", r.display.clone().map_or(J::Null, J::s)),
                ("label_latex", r.label_latex.clone().map_or(J::Null, J::s)),
                ("label_plain", r.label_plain.clone().map_or(J::Null, J::s)),
                ("shown_on_graph", J::Bool(r.display.is_some())),
            ])
        })
        .collect();
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
                "r2_display_rule",
                J::s("R²_A = x in blue and R²_B = y in red, from the stored display string, which is rounded toward minus infinity; nothing else inside the plot area: no N, no interval, no status word, no placeholder where no value exists; the reason, the citation and the link stand below the graph (metric-r2.md Sections 7.2 to 7.4)"),
            ),
            (
                "r2_label_templates",
                J::obj(vec![
                    ("latex_a", J::s("R^{2}_{\\mathrm{A}} = <display_latex>")),
                    ("latex_b", J::s("R^{2}_{\\mathrm{B}} = <display_latex>")),
                    ("plain_a", J::s("R²_A = <display>")),
                    ("plain_b", J::s("R²_B = <display>")),
                ]),
            ),
            ("texts", J::Arr(texts)),
            ("r2_labels", J::Arr(r2)),
            ("r2_captions", J::Arr(captions.to_vec())),
        ],
    )
}
