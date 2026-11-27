//! Material parameters, diffusivity laws, yields and the fission-rate density of the registers.
//!
//! The selections, the nuclide data and the rate density live in `strato-scenarios` (shared with
//! the WebAssembly wrapper) and are re-exported here; this file keeps the description of a
//! selection for the parameters file.

use strato_compare::json::J;
use strato_data::select::RangeStatus;
use strato_data::table::Vocabulary;

pub use strato_scenarios::laws::{
    condition, decay_constant, generation_nuclides, rate_density, reference_generation, Laws,
    Nuclide, RateDensity, LAYERS,
};

/// The description of the selections of `laws` for the parameters file, with the diffusivity at
/// each temperature of `probe_c`.
pub fn laws_json(laws: &Laws<'_>, probe_c: &[f64]) -> J {
    J::Arr(
        laws.sel
            .iter()
            .zip(LAYERS)
            .map(|(s, (name, _))| {
                J::obj(vec![
                    ("layer", J::s(name)),
                    (
                        "register_rows",
                        J::Arr(s.row_ids().iter().map(|i| J::s(*i)).collect()),
                    ),
                    ("terms", J::Int(s.terms() as i64)),
                    ("coefficient_class", J::s(s.class.as_str())),
                    ("phase_of_use", J::s(s.rows[0].phase_of_use.as_str())),
                    ("source", J::s(s.source.clone())),
                    ("source_location", J::s(s.source_location.clone())),
                    (
                        "published_range_c",
                        match s.published_range_c {
                            Some((a, b)) => J::Arr(vec![J::Num(a), J::Num(b)]),
                            None => J::Null,
                        },
                    ),
                    (
                        "range_kind",
                        s.range_kind.map_or(J::Null, |k| J::s(k.as_str())),
                    ),
                    (
                        "labels",
                        J::Arr(s.labels.iter().map(|l| J::s(l.clone())).collect()),
                    ),
                    (
                        "diffusivity_m2_s_at_c",
                        J::Arr(
                            probe_c
                                .iter()
                                .map(|t| {
                                    let e = s.diffusivity_extrapolated(*t);
                                    J::obj(vec![
                                        ("temperature_c", J::Num(*t)),
                                        ("diffusivity_m2_s", J::Num(e.diffusivity.m2_per_s)),
                                        (
                                            "range",
                                            J::s(match e.range {
                                                RangeStatus::NoRangePrinted => "no range printed",
                                                RangeStatus::Inside => "inside the printed range",
                                                RangeStatus::Extrapolated { .. } => {
                                                    "outside the printed range"
                                                }
                                            }),
                                        ),
                                    ])
                                })
                                .collect(),
                        ),
                    ),
                ])
            })
            .collect(),
    )
}
