//! The output files: CSV with a provenance header of comment lines, and one JSON file for the
//! website. Column headers are full physical names with their unit (decision 30).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use strato::postprocessing::export::{sha256_hex, LABEL, SOLVER_ID};
use strato_data::release::ReleaseValue;
use strato_data::table::Vocabulary;

use crate::compare::{reading_text, Dataset, Eval, RowRecord};
use crate::data::Species;
use crate::json::J;
use crate::model::Curve;
use crate::status::{measured_text, Status, CATEGORIES, MODEL_LIMITATIONS};
use crate::studies::{AnalyticSeries, ConvergenceRow, ProfileRow, SensitivityRow};

/// The statement made wherever a row labelled with the element only is compared (decision 21).
pub const ELEMENT_ONLY_STATEMENT: &str = "the row is labelled with the element only (\"Cs\"): it is compared as elemental caesium with the caesium coefficient rows and without decay (decision 21)";

/// Where every file came from.
#[derive(Debug, Clone)]
pub struct Prov {
    /// Commit of the source tree the numbers were computed from.
    pub commit: String,
    /// Version of the solver package.
    pub solver_version: String,
    /// SHA-256 of `release_measurements.csv`.
    pub release_sha256: String,
    /// SHA-256 of `arrhenius.csv`.
    pub arrhenius_sha256: String,
    /// SHA-256 of `geometry.csv`.
    pub geometry_sha256: String,
    /// SHA-256 of `temperature_histories.csv`.
    pub temperature_sha256: String,
    /// Description of the registered inputs of the model.
    pub parameter_source: String,
}

impl Prov {
    /// Build from the register texts.
    pub fn new(
        commit: &str,
        solver_version: &str,
        texts: &strato_data::register::RegisterTexts,
        parameter_source: String,
    ) -> Prov {
        Prov {
            commit: if commit.trim().is_empty() {
                "unknown".into()
            } else {
                commit.trim().into()
            },
            solver_version: solver_version.to_string(),
            release_sha256: sha256_hex(texts.release.as_bytes()),
            arrhenius_sha256: sha256_hex(texts.arrhenius.as_bytes()),
            geometry_sha256: sha256_hex(texts.geometry.as_bytes()),
            temperature_sha256: sha256_hex(texts.temperature.as_bytes()),
            parameter_source,
        }
    }

    /// The comment header of a CSV file.
    pub fn header(&self, what: &str) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "# STRATO experimental comparison: {what}");
        let _ = writeln!(
            s,
            "# label: {LABEL}; the comparison with the register is a comparison of release behaviour, not a validation"
        );
        let _ = writeln!(
            s,
            "# solver: {SOLVER_ID}, package version {}",
            self.solver_version
        );
        let _ = writeln!(s, "# commit: {}", self.commit);
        let _ = writeln!(s, "# model_limitations: {MODEL_LIMITATIONS}");
        let _ = writeln!(s, "# parameter_source: {}", self.parameter_source);
        let _ = writeln!(
            s,
            "# parameter_class: coefficients are the published heating-test correlation of the register (class and phase per row in the sensitivity file); radii are the registered measured baseline geometry; every measurement keeps its register row id"
        );
        let _ = writeln!(s, "# release_measurements_sha256: {}", self.release_sha256);
        let _ = writeln!(s, "# arrhenius_sha256: {}", self.arrhenius_sha256);
        let _ = writeln!(s, "# geometry_sha256: {}", self.geometry_sha256);
        let _ = writeln!(
            s,
            "# temperature_histories_sha256: {}",
            self.temperature_sha256
        );
        s
    }
}

/// A CSV text under construction.
pub struct Csv {
    text: String,
}

fn cell(x: &str) -> String {
    if x.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", x.replace('"', "\"\"").replace(['\n', '\r'], " "))
    } else {
        x.to_string()
    }
}

/// A number in the shortest round-trip scientific form, or empty.
pub fn num(x: f64) -> String {
    if x.is_finite() {
        format!("{x:e}")
    } else {
        String::new()
    }
}

/// An optional number.
pub fn onum(x: Option<f64>) -> String {
    x.map_or(String::new(), num)
}

impl Csv {
    /// Start with a comment header and the column names.
    pub fn new(header: &str, columns: &[&str]) -> Csv {
        let mut text = header.to_string();
        text.push_str(
            &columns
                .iter()
                .map(|c| cell(c))
                .collect::<Vec<_>>()
                .join(","),
        );
        text.push('\n');
        Csv { text }
    }

    /// Append a row; the number of cells must equal the number of columns of the header.
    pub fn row(&mut self, cells: Vec<String>) {
        self.text
            .push_str(&cells.iter().map(|c| cell(c)).collect::<Vec<_>>().join(","));
        self.text.push('\n');
    }

    /// The text.
    pub fn finish(self) -> String {
        self.text
    }
}

/// Text of the value cell(s) of a row.
fn value_cells(v: &ReleaseValue) -> (String, String, String, &'static str) {
    match v {
        ReleaseValue::Point(p)
        | ReleaseValue::Digitised(p)
        | ReleaseValue::UpperBound(p)
        | ReleaseValue::LowerBound(p) => {
            (p.text.clone(), String::new(), String::new(), v.qualifier())
        }
        ReleaseValue::Range { min, max } => (
            String::new(),
            min.text.clone(),
            max.text.clone(),
            v.qualifier(),
        ),
        ReleaseValue::FailedOverTested { failed, tested } => (
            failed.text.clone(),
            String::new(),
            tested.text.clone(),
            v.qualifier(),
        ),
    }
}

fn temperature_text(r: &RowRecord) -> String {
    use strato_data::release::TestTemperature as T;
    match &r.row.temperature {
        T::Celsius(p) => p.text.clone(),
        T::Range { lo, hi } => format!("{}-{}", lo.text, hi.text),
        T::Varied { lo, hi } => format!("varied ({}-{})", lo.text, hi.text),
        T::Absent => String::new(),
    }
}

/// The kind of evaluation of a row, in words.
pub fn eval_kind(e: &Eval) -> &'static str {
    match e {
        Eval::None => "none",
        Eval::Bound { .. } => "upper bound (inequality)",
        Eval::Point { .. } => "point value",
        Eval::Provisional { .. } => {
            "measured and model value for scale only (denominator not established; no deviation formed)"
        }
    }
}

/// Per-row comparison file.
pub fn rows_csv(prov: &Prov, records: &[RowRecord]) -> String {
    let cols = [
        "Row identifier",
        "Experimental programme",
        "Compact or sample",
        "Fuel type",
        "Group of particles",
        "Nuclide or element as registered",
        "Test temperature (°C)",
        "Time at temperature stated by the row (h)",
        "Time basis",
        "Quantity as registered",
        "Qualifier",
        "Measured value (dimensionless unless the unit column says otherwise)",
        "Range minimum or failed count",
        "Range maximum or tested count",
        "Unit of the measured value",
        "Value class",
        "Reading uncertainty of a digitised value",
        "Denominator of the fraction",
        "Registered row status",
        "Source",
        "Source location",
        "Comparison status",
        "Status rule",
        "Reason for the status",
        "Model element",
        "Model diffusion coefficient set",
        "Window start (h at temperature)",
        "Window end (h at temperature)",
        "Origin of the window end",
        "Model release fraction at window end (dimensionless)",
        "Model release fraction at window start (dimensionless)",
        "Model value compared (dimensionless)",
        "Numerical difference of the model value between base and refined run (dimensionless)",
        "Kind of evaluation",
        "How a bound is read",
        "Signed deviation, model minus measurement (dimensionless)",
        "Absolute deviation (dimensionless)",
        "Relative deviation (percent)",
        "Signed log10 ratio, model over measurement (dimensionless)",
        "Model value over bound (dimensionless)",
        "Excess of the model over the bound (dimensionless)",
        "Uncertainty-normalised residual (dimensionless)",
        "Statement for a row that names the element only (decision 21)",
        "Model limitations",
    ];
    let mut csv = Csv::new(
        &prov.header("per-row comparison of the release register with the BTCS model"),
        &cols,
    );
    for r in records {
        let (value, lo, hi, qual) = value_cells(&r.row.value);
        let mut c = vec![
            r.row.id.clone(),
            r.row.program.as_str().to_string(),
            crate::data::typography(&r.row.compact_or_sample),
            r.row.fuel_type.as_str().to_string(),
            r.row.group.as_str().to_string(),
            measured_text(&r.row),
            temperature_text(r),
            r.row
                .time_h
                .as_ref()
                .map_or(String::new(), |p| p.text.clone()),
            crate::data::typography(&r.row.time_basis),
            r.row.quantity.as_str().to_string(),
            qual.to_string(),
            value,
            lo,
            hi,
            r.row.unit.as_str().to_string(),
            r.row.value_class.as_str().to_string(),
            reading_text(&r.row.reading_uncertainty),
            r.row.denominator.as_str().to_string(),
            r.row.status.as_str().to_string(),
            r.row.source.clone(),
            r.row.source_location.clone(),
            r.status.label().to_string(),
            r.verdict.category.to_string(),
            r.verdict.reason.clone(),
        ];
        match &r.model {
            Some(m) => {
                c.push(m.spec.species.label().to_string());
                c.push(m.set_id.clone());
                c.push(num(m.spec.window_start_h));
                c.push(num(m.spec.t_end_h));
                c.push(m.spec.time_source.clone());
                c.push(num(m.f_end));
                c.push(num(m.f_start));
                c.push(num(m.f_window));
                c.push(num(m.numerical_difference));
            }
            None => c.extend(std::iter::repeat_n(String::new(), 9)),
        }
        c.push(eval_kind(&r.eval).to_string());
        // Eight cells: how a bound is read, signed, absolute, relative, log10 ratio, model over
        // bound, excess over bound, normalised residual.
        match &r.eval {
            Eval::None => c.extend(std::iter::repeat_n(String::new(), 8)),
            Eval::Bound { source, margin, .. } => {
                c.push(source.text().to_string());
                c.extend(std::iter::repeat_n(String::new(), 4));
                c.push(num(margin.ratio));
                c.push(num(margin.excess));
                c.push(String::new());
            }
            Eval::Provisional { log10_ratio, .. } => {
                // for scale only: no signed, absolute, relative deviation and no residual
                c.push(String::new());
                c.extend(std::iter::repeat_n(String::new(), 3));
                c.push(onum(*log10_ratio));
                c.extend(std::iter::repeat_n(String::new(), 3));
            }
            Eval::Point { dev, .. } => {
                c.push(String::new());
                c.push(num(dev.signed));
                c.push(num(dev.absolute));
                c.push(onum(dev.relative_percent));
                c.push(onum(dev.log10_ratio));
                c.push(String::new());
                c.push(String::new());
                c.push(onum(dev.normalised_residual));
            }
        }
        c.push(match &r.model {
            Some(m) if m.spec.element_label_only => ELEMENT_ONLY_STATEMENT.to_string(),
            _ => String::new(),
        });
        c.push(if r.model.is_some() {
            MODEL_LIMITATIONS.to_string()
        } else {
            String::new()
        });
        debug_assert_eq!(c.len(), cols.len());
        csv.row(c);
    }
    csv.finish()
}
/// Per-dataset aggregate file.
pub fn datasets_csv(prov: &Prov, sets: &[Dataset]) -> String {
    let mut cols: Vec<String> = vec![
        "Experimental programme".into(),
        "Nuclide or element as registered".into(),
        "Group of particles".into(),
        "Quantity as registered".into(),
        "Source".into(),
        "Number of register rows".into(),
    ];
    for s in Status::ALL {
        cols.push(format!("Rows with status {}", s.label()));
    }
    cols.extend(
        [
            "Test temperatures (°C)",
            "Longest compared test duration (h)",
            "Point comparisons: number",
            "Point comparisons: RMSE (dimensionless)",
            "Point comparisons: MAE (dimensionless)",
            "Point comparisons: maximum absolute deviation (dimensionless)",
            "Point comparisons: RMSE of log10 ratio (dimensionless)",
            "Point comparisons: MAE of log10 ratio (dimensionless)",
            "Point comparisons: number with a defined relative deviation",
            "Upper bounds: number",
            "Upper bounds: satisfied",
            "Upper bounds: exceeded",
            "Upper bounds: smallest model value over bound (dimensionless)",
            "Upper bounds: largest model value over bound (dimensionless)",
            "Row identifiers",
        ]
        .map(String::from),
    );
    let colrefs: Vec<&str> = cols.iter().map(String::as_str).collect();
    let mut csv = Csv::new(
        &prov.header("per-dataset aggregates; groups never mix nuclides or quantity definitions"),
        &colrefs,
    );
    for d in sets {
        let mut c = vec![
            d.program.clone(),
            d.nuclide.clone(),
            d.group.clone(),
            d.quantity.clone(),
            d.source.clone(),
            d.row_ids.len().to_string(),
        ];
        for s in Status::ALL {
            c.push(d.counts.get(&s).copied().unwrap_or(0).to_string());
        }
        c.push(
            d.temperatures_c
                .iter()
                .map(|t| format!("{t}"))
                .collect::<Vec<_>>()
                .join(" "),
        );
        c.push(onum(d.max_duration_h));
        match &d.point {
            Some(p) => {
                c.push(p.n.to_string());
                c.push(num(p.rmse));
                c.push(num(p.mae));
                c.push(num(p.max_absolute));
                c.push(onum(p.rmse_log10));
                c.push(onum(p.mae_log10));
                c.push(p.n_relative.to_string());
            }
            None => {
                c.push("0".into());
                c.extend(std::iter::repeat_n(String::new(), 5));
                c.push(String::new());
            }
        }
        match &d.bound {
            Some(b) => {
                c.push(b.n.to_string());
                c.push(b.satisfied.to_string());
                c.push(b.exceeded.to_string());
                c.push(num(b.ratio_min));
                c.push(num(b.ratio_max));
            }
            None => {
                c.push("0".into());
                c.extend(std::iter::repeat_n(String::new(), 4));
            }
        }

        c.push(d.row_ids.join(" "));
        csv.row(c);
    }
    csv.finish()
}

/// Release-versus-time curves of the default sets.
pub fn curves_csv(
    prov: &Prov,
    curves: &[(Species, f64, &Curve, String)],
    analytic: &AnalyticSeries,
    floors: &BTreeMap<(Species, i64), f64>,
    stride: usize,
) -> String {
    let cols = [
        "Nuclide",
        "Test temperature (°C)",
        "Diffusion coefficient set",
        "Time at temperature (h)",
        "Release fraction, BTCS base run (dimensionless)",
        "Release fraction, BTCS refined run (dimensionless)",
        "Release fraction, independent analytical series (dimensionless)",
        "Estimated absolute accuracy of the analytical series for this case (dimensionless)",
    ];
    let mut csv = Csv::new(
        &(prov.header("release fraction versus time, kernel-only start, constant temperature")
            + &format!(
                "# analytical series validity criterion: {}\n",
                crate::studies::ANALYTIC_CRITERION
            )),
        &cols,
    );
    for (s, t_c, c, set) in curves {
        let an = analytic.get(&(*s, (*t_c * 10.0).round() as i64));
        for k in (0..c.t_h.len()).step_by(stride) {
            let t = c.t_h[k];
            let a = an.and_then(|v| {
                v.iter()
                    .find(|(x, _)| (x - t).abs() < 1e-9)
                    .and_then(|(_, f)| *f)
            });
            csv.row(vec![
                s.nuclide().to_string(),
                format!("{t_c}"),
                set.clone(),
                num(t),
                num(c.f[k]),
                onum(c.at_refined(t)),
                onum(a),
                onum(floors.get(&(*s, (*t_c * 10.0).round() as i64)).copied()),
            ]);
        }
    }
    csv.finish()
}

/// Deviations versus time of every row that has a model value and a measured number.
pub fn deviation_csv(prov: &Prov, records: &[RowRecord]) -> String {
    let cols = [
        "Row identifier",
        "Comparison status",
        "Kind of evaluation",
        "Nuclide",
        "Test temperature (°C)",
        "Time at temperature (h)",
        "Measured value or bound (dimensionless)",
        "Model value compared (dimensionless)",
        "Signed deviation, model minus measurement (dimensionless)",
        "Absolute deviation (dimensionless)",
        "Relative deviation (percent)",
        "Signed log10 ratio, model over measurement (dimensionless)",
        "Excess of the model over a bound (dimensionless)",
    ];
    let mut csv = Csv::new(
        &prov.header("deviation of the model from the measurement versus time; rows whose denominator is not established (DATA INCOMPLETE) carry the measured value, the model value and the log10 ratio for scale only, with no deviation"),
        &cols,
    );
    let mut items: Vec<&RowRecord> = records
        .iter()
        .filter(|r| r.model.is_some() && !matches!(r.eval, Eval::None))
        .collect();
    items.sort_by(|a, b| {
        let ka = (
            measured_text(&a.row),
            a.model.as_ref().map_or(0.0, |m| m.spec.t_c),
        );
        let kb = (
            measured_text(&b.row),
            b.model.as_ref().map_or(0.0, |m| m.spec.t_c),
        );
        ka.0.cmp(&kb.0)
            .then(ka.1.partial_cmp(&kb.1).unwrap_or(std::cmp::Ordering::Equal))
            .then(
                a.model
                    .as_ref()
                    .map_or(0.0, |m| m.spec.t_end_h)
                    .partial_cmp(&b.model.as_ref().map_or(0.0, |m| m.spec.t_end_h))
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });
    for r in items {
        let Some(m) = &r.model else { continue };
        let (measured, signed, abs, rel, lg, excess) = match &r.eval {
            Eval::Bound { limit, margin, .. } => (
                num(*limit),
                String::new(),
                String::new(),
                String::new(),
                onum(margin.log10_ratio),
                num(margin.excess),
            ),
            Eval::Point { value, dev } => (
                num(*value),
                num(dev.signed),
                num(dev.absolute),
                onum(dev.relative_percent),
                onum(dev.log10_ratio),
                String::new(),
            ),
            Eval::Provisional { value, log10_ratio } => (
                num(*value),
                String::new(),
                String::new(),
                String::new(),
                onum(*log10_ratio),
                String::new(),
            ),
            Eval::None => continue,
        };
        csv.row(vec![
            r.row.id.clone(),
            r.status.label().to_string(),
            eval_kind(&r.eval).to_string(),
            measured_text(&r.row),
            num(m.spec.t_c),
            num(m.spec.t_end_h),
            measured,
            num(m.f_window),
            signed,
            abs,
            rel,
            lg,
            excess,
        ]);
    }
    csv.finish()
}

/// Radial profiles.
pub fn profiles_csv(prov: &Prov, rows: &[ProfileRow]) -> String {
    let cols = [
        "Nuclide",
        "Test temperature (°C)",
        "Time at temperature (h)",
        "Node index (dimensionless)",
        "Radius (µm)",
        "Material region",
        "Concentration over initial kernel concentration (dimensionless)",
    ];
    let mut csv = Csv::new(
        &prov.header("radial concentration profiles through the five layers; the initial kernel concentration is the unit"),
        &cols,
    );
    for p in rows {
        csv.row(vec![
            p.species.nuclide().to_string(),
            format!("{}", p.t_c),
            num(p.t_h),
            p.node.to_string(),
            num(p.r_um),
            p.region.clone(),
            num(p.c_over_c0),
        ]);
    }
    csv.finish()
}

/// Inventory balance of the default curves.
pub fn balance_csv(
    prov: &Prov,
    curves: &[(Species, f64, &Curve, String)],
    stride: usize,
) -> String {
    let cols = [
        "Nuclide",
        "Test temperature (°C)",
        "Time at temperature (h)",
        "Remaining inventory over initial inventory (dimensionless)",
        "Cumulative release over initial inventory (dimensionless)",
        "Relative mass-balance residual (dimensionless)",
        "Derived a-posteriori bound of the relative residual (dimensionless)",
    ];
    let mut csv = Csv::new(
        &(prov.header("inventory balance: remaining inventory plus cumulative release minus initial inventory")
            + "# generated inventory and decay loss: not in the solver yet, so the file has no such columns (they are not zero, they are not computed)\n"),
        &cols,
    );
    for (s, t_c, c, _) in curves {
        for k in (0..c.t_h.len()).step_by(stride) {
            csv.row(vec![
                s.nuclide().to_string(),
                format!("{t_c}"),
                num(c.t_h[k]),
                num(c.remaining[k]),
                num(c.f[k]),
                num(c.residual_rel[k]),
                num(c.bound_rel[k]),
            ]);
        }
    }
    csv.finish()
}

/// Temperature sensitivity.
pub fn sensitivity_csv(prov: &Prov, rows: &[SensitivityRow]) -> String {
    let cols = [
        "Nuclide",
        "Coefficient variant",
        "Variant description",
        "Class of the kernel coefficient",
        "Phase of use of the kernel coefficient",
        "Class of the buffer coefficient",
        "Phase of use of the buffer coefficient",
        "Class of the inner pyrocarbon coefficient",
        "Phase of use of the inner pyrocarbon coefficient",
        "Class of the silicon carbide coefficient",
        "Phase of use of the silicon carbide coefficient",
        "Class of the outer pyrocarbon coefficient",
        "Phase of use of the outer pyrocarbon coefficient",
        "Test temperature (°C)",
        "Kernel diffusion coefficient (m²/s)",
        "Buffer diffusion coefficient (m²/s)",
        "IPyC diffusion coefficient (m²/s)",
        "SiC diffusion coefficient (m²/s)",
        "OPyC diffusion coefficient (m²/s)",
        "Release fraction at 24 h (dimensionless)",
        "Release fraction at 100 h (dimensionless)",
        "Release fraction at 300 h (dimensionless)",
        "A temperature range is printed for some row of the variant",
        "The temperature is outside a printed range",
    ];
    let mut csv = Csv::new(
        &prov.header("temperature sensitivity of the release of an intact particle over the registered heating-test temperatures"),
        &cols,
    );
    for r in rows {
        let mut c = vec![
            r.species.nuclide().to_string(),
            r.variant_id.clone(),
            r.variant_label.clone(),
        ];
        for (class, phase) in &r.classes {
            c.push(class.clone());
            c.push(phase.clone());
        }
        c.push(format!("{}", r.t_c));
        c.extend(r.d_m2_s.iter().map(|d| num(*d)));
        c.push(num(r.f_24));
        c.push(num(r.f_100));
        c.push(num(r.f_300));
        c.push(r.any_printed_range.to_string());
        c.push(r.outside_printed_range.to_string());
        csv.row(c);
    }
    csv.finish()
}

/// Convergence rows.
pub fn convergence_csv(
    prov: &Prov,
    what: &str,
    step_name: &str,
    rows: &[ConvergenceRow],
) -> String {
    let cols = [
        "Refinement factor (dimensionless)",
        step_name,
        "Number of nodes or steps (dimensionless)",
        "Release fraction, BTCS (dimensionless)",
        "Release fraction, analytical (dimensionless)",
        "Truncation bound of the analytical series (dimensionless)",
        "Absolute error of the release fraction (dimensionless)",
        "Observed order (dimensionless)",
    ];
    let mut csv = Csv::new(&prov.header(what), &cols);
    for r in rows {
        csv.row(vec![
            r.level.to_string(),
            num(r.step),
            r.count.to_string(),
            num(r.f_solver),
            num(r.f_analytic),
            num(r.f_analytic_bound),
            num(r.error),
            onum(r.order),
        ]);
    }
    csv.finish()
}

/// The assumptions that belong to one model value, in words.
fn row_assumptions(r: &RowRecord, m: &crate::compare::ModelValue) -> Vec<String> {
    let mut a = vec![
        "kernel-only start: all inventory in the kernel at the start of the window clock (decision 20)".to_string(),
        format!(
            "compared window {} to {} h at temperature; time counted from reaching the test temperature; origin of the end time: {}",
            m.spec.window_start_h, m.spec.t_end_h, m.spec.time_source
        ),
        "constant diffusion coefficients at the test temperature; heating ramps are not modelled".to_string(),
        "no radioactive decay and no fission-product generation in the solver".to_string(),
        "model fraction is a fraction of the particle's own inventory; measured fractions are fractions of the compact inventory unless the denominator column says otherwise".to_string(),
    ];
    if m.spec.element_label_only {
        a.push(ELEMENT_ONLY_STATEMENT.to_string());
    }
    if matches!(r.eval, Eval::Provisional { .. }) {
        a.push("provisional: the denominator is not established, so this model value is shown for scale only; no deviation, relative deviation or aggregate is formed from it".to_string());
    }
    a
}
/// The JSON file for the website panel.
pub fn comparison_json(
    prov: &Prov,
    records: &[RowRecord],
    sets: &[Dataset],
    geometry: &crate::data::Radii,
    set_notes: &[(String, String)],
    coefficient_regions: Vec<J>,
) -> String {
    let mut counts: BTreeMap<Status, usize> = BTreeMap::new();
    for r in records {
        *counts.entry(r.status).or_insert(0) += 1;
    }
    let status_counts: Vec<(String, J)> = Status::ALL
        .iter()
        .map(|s| {
            (
                s.label().to_string(),
                J::Int(counts.get(s).copied().unwrap_or(0) as i64),
            )
        })
        .collect();
    let rows: Vec<J> = records
        .iter()
        .map(|r| {
            let (value, lo, hi, qual) = value_cells(&r.row.value);
            let mut o = vec![
                ("id", J::s(&r.row.id)),
                ("programme", J::s(r.row.program.as_str())),
                (
                    "compact_or_sample",
                    J::s(crate::data::typography(&r.row.compact_or_sample)),
                ),
                ("group", J::s(r.row.group.as_str())),
                ("nuclide", J::s(measured_text(&r.row))),
                ("temperature_text", J::s(temperature_text(r))),
                ("quantity", J::s(r.row.quantity.as_str())),
                ("qualifier", J::s(qual)),
                (
                    "measured_value_text",
                    if value.is_empty() {
                        J::Null
                    } else {
                        J::s(value)
                    },
                ),
                (
                    "range_min_or_count_text",
                    if lo.is_empty() { J::Null } else { J::s(lo) },
                ),
                (
                    "range_max_or_count_text",
                    if hi.is_empty() { J::Null } else { J::s(hi) },
                ),
                ("value_class", J::s(r.row.value_class.as_str())),
                (
                    "reading_uncertainty",
                    J::s(reading_text(&r.row.reading_uncertainty)),
                ),
                (
                    "reading_interval",
                    match &r.row.value {
                        ReleaseValue::Point(p)
                        | ReleaseValue::Digitised(p)
                        | ReleaseValue::UpperBound(p) => {
                            crate::figures::reading_interval(&r.row.reading_uncertainty, p.value)
                                .map_or(J::Null, |(lo, hi)| J::Arr(vec![J::Num(lo), J::Num(hi)]))
                        }
                        _ => J::Null,
                    },
                ),
                ("reported_measurement_uncertainty", J::Null),
                ("denominator", J::s(r.row.denominator.as_str())),
                ("source", J::s(&r.row.source)),
                ("source_location", J::s(&r.row.source_location)),
                ("status", J::s(r.status.label())),
                ("status_rule", J::s(r.verdict.category)),
                ("status_reason", J::s(&r.verdict.reason)),
                ("evaluation_kind", J::s(eval_kind(&r.eval))),
            ];
            if let Some(m) = &r.model {
                o.push((
                    "model",
                    J::obj(vec![
                        ("element", J::s(m.spec.species.label())),
                        ("element_label_only", J::Bool(m.spec.element_label_only)),
                        ("temperature_c", J::Num(m.spec.t_c)),
                        ("window_start_h", J::Num(m.spec.window_start_h)),
                        ("window_end_h", J::Num(m.spec.t_end_h)),
                        ("window_end_origin", J::s(&m.spec.time_source)),
                        ("release_fraction_at_end", J::Num(m.f_end)),
                        ("release_fraction_at_start", J::Num(m.f_start)),
                        ("value_compared", J::Num(m.f_window)),
                        ("numerical_difference", J::Num(m.numerical_difference)),
                        ("coefficient_set", J::s(&m.set_id)),
                        (
                            "assumptions",
                            J::Arr(row_assumptions(r, m).iter().map(J::s).collect()),
                        ),
                        (
                            "diffusion_coefficients_m2_s",
                            J::Arr(m.d_m2_s.iter().map(|d| J::Num(*d)).collect()),
                        ),
                    ]),
                ));
            }
            match &r.eval {
                Eval::None => {}
                Eval::Bound {
                    limit,
                    source,
                    margin,
                } => o.push((
                    "bound",
                    J::obj(vec![
                        ("limit", J::Num(*limit)),
                        ("read_as", J::s(source.text())),
                        ("satisfied", J::Bool(margin.satisfied)),
                        ("model_over_limit", J::Num(margin.ratio)),
                        ("excess", J::Num(margin.excess)),
                        ("log10_ratio", J::opt(margin.log10_ratio)),
                    ]),
                )),
                Eval::Point { value, dev } => o.push((
                    "deviation",
                    J::obj(vec![
                        ("measured_value", J::Num(*value)),
                        ("signed", J::Num(dev.signed)),
                        ("absolute", J::Num(dev.absolute)),
                        ("relative_percent", J::opt(dev.relative_percent)),
                        ("log10_ratio", J::opt(dev.log10_ratio)),
                        ("normalised_residual", J::opt(dev.normalised_residual)),
                    ]),
                )),
                Eval::Provisional { value, log10_ratio } => o.push((
                    "for_scale_only",
                    J::obj(vec![
                        ("measured_value", J::Num(*value)),
                        ("log10_ratio", J::opt(*log10_ratio)),
                        (
                            "note",
                            J::s("denominator not established (decision 22): no deviation, relative deviation or aggregate is formed"),
                        ),
                    ]),
                )),
            }
            J::obj(o)
        })
        .collect();
    let datasets: Vec<J> = sets
        .iter()
        .map(|d| {
            let counts: Vec<(String, J)> = Status::ALL
                .iter()
                .map(|s| {
                    (
                        s.label().to_string(),
                        J::Int(d.counts.get(s).copied().unwrap_or(0) as i64),
                    )
                })
                .collect();
            let mut o = vec![
                ("programme", J::s(&d.program)),
                ("nuclide", J::s(&d.nuclide)),
                ("group", J::s(&d.group)),
                ("quantity", J::s(&d.quantity)),
                ("source", J::s(&d.source)),
                ("row_ids", J::Arr(d.row_ids.iter().map(J::s).collect())),
                (
                    "temperatures_c",
                    J::Arr(d.temperatures_c.iter().map(|t| J::Num(*t)).collect()),
                ),
                ("longest_compared_duration_h", J::opt(d.max_duration_h)),
                ("status_counts", J::Obj(counts)),
            ];
            let pa = |p: &crate::metrics::PointAggregate| {
                J::obj(vec![
                    ("n", J::Int(p.n as i64)),
                    ("rmse", J::Num(p.rmse)),
                    ("mae", J::Num(p.mae)),
                    ("max_absolute_deviation", J::Num(p.max_absolute)),
                    ("rmse_log10_ratio", J::opt(p.rmse_log10)),
                    ("mae_log10_ratio", J::opt(p.mae_log10)),
                    ("n_relative_deviation_defined", J::Int(p.n_relative as i64)),
                ])
            };
            o.push(("point_metrics", d.point.as_ref().map_or(J::Null, pa)));
            o.push((
                "bound_metrics",
                d.bound.as_ref().map_or(J::Null, |b| {
                    J::obj(vec![
                        ("n", J::Int(b.n as i64)),
                        ("satisfied", J::Int(b.satisfied as i64)),
                        ("exceeded", J::Int(b.exceeded as i64)),
                        ("model_over_bound_min", J::Num(b.ratio_min)),
                        ("model_over_bound_max", J::Num(b.ratio_max)),
                    ])
                }),
            ));

            J::obj(o)
        })
        .collect();
    let cats = J::Arr(
        CATEGORIES
            .iter()
            .map(|(k, v)| J::obj(vec![("code", J::s(*k)), ("meaning", J::s(*v))]))
            .collect(),
    );
    J::obj(vec![
        ("schema_version", J::s("1")),
        (
            "provenance",
            J::obj(vec![
                ("solver", J::s(SOLVER_ID)),
                ("solver_package_version", J::s(&prov.solver_version)),
                ("commit", J::s(&prov.commit)),
                ("label", J::s(LABEL)),
                ("release_measurements_sha256", J::s(&prov.release_sha256)),
                ("arrhenius_sha256", J::s(&prov.arrhenius_sha256)),
                ("geometry_sha256", J::s(&prov.geometry_sha256)),
                ("temperature_histories_sha256", J::s(&prov.temperature_sha256)),
                ("parameter_source", J::s(&prov.parameter_source)),
            ]),
        ),
        ("disclosure_title", J::s("Experimental Comparison: Data Interpretation and Limitations")),
        ("model_limitations", J::s(MODEL_LIMITATIONS)),
        (
            "decay_and_generation_in_solver",
            J::Bool(false),
        ),
        ("relative_deviation_floor", J::Null),
        (
            "relative_deviation_rule",
            J::s("undefined for a measured value <= 0; no floor is applied because the register states no detection floor (decision 17)"),
        ),
        (
            "geometry",
            J::obj(vec![
                ("design", J::s(crate::data::GEOMETRY_DESIGN)),
                ("outer_radii_m", J::Arr(geometry.outer_m.iter().map(|r| J::Num(*r)).collect())),
                ("geometry_row_ids", J::Arr(geometry.row_ids.iter().map(J::s).collect())),
            ]),
        ),
        (
            "coefficient_sets",
            J::Arr(
                set_notes
                    .iter()
                    .map(|(k, v)| J::obj(vec![("set", J::s(k)), ("description", J::s(v))]))
                    .collect(),
            ),
        ),
        ("coefficient_regions", J::Arr(coefficient_regions)),
        ("status_counts", J::Obj(status_counts)),
        ("status_rules", cats),
        ("datasets", J::Arr(datasets)),
        ("rows", J::Arr(rows)),
    ])
    .render()
}
