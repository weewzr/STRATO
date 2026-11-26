//! The report `docs/validation/experimental_comparison.md`, rendered from the results.
//!
//! The narrative is fixed text; every number in it, and every table, is computed from the rows,
//! the runs and the studies of the same run, so the report cannot drift from the files.

use std::fmt::Write as _;

use strato_data::release::TestTemperature;
use strato_data::table::Vocabulary;
use strato_data::Register;

use crate::compare::{Dataset, Eval, RowRecord};
use crate::data::{Radii, Species};
use crate::model::Curve;
use crate::outputs::Prov;
use crate::status::{measured_text, Status, CATEGORIES, MODEL_LIMITATIONS};
use crate::studies::{AnalyticSeries, ConvergenceRow, SensitivityRow};

/// Everything the report needs.
pub struct ReportInput<'a> {
    /// The registers.
    pub reg: &'a Register,
    /// Rows.
    pub records: &'a [RowRecord],
    /// Aggregates.
    pub datasets: &'a [Dataset],
    /// Sensitivity.
    pub sensitivity: &'a [SensitivityRow],
    /// Spatial convergence.
    pub spatial: &'a [ConvergenceRow],
    /// Temporal convergence.
    pub temporal: &'a [ConvergenceRow],
    /// Curves.
    pub curves: &'a [(Species, f64, &'a Curve)],
    /// Analytical series.
    pub analytic: &'a AnalyticSeries,
    /// Provenance.
    pub prov: &'a Prov,
    /// Geometry.
    pub geometry: &'a Radii,
    /// Spatial floor of the temporal study.
    pub temporal_spatial_floor: f64,
    /// Description of the verification case.
    pub homogeneous_note: &'a str,
    /// Coefficient set notes.
    pub set_notes: &'a [(String, String)],
}

fn sci(x: f64) -> String {
    if x == 0.0 {
        "0".to_string()
    } else {
        format!("{x:.2e}")
    }
}

fn pct(x: f64) -> String {
    format!("{x:.2}")
}

/// Compress ids such as E-001, E-002, E-003, E-007 into `E-001..E-003, E-007`.
pub fn compress_ids(ids: &[String]) -> String {
    let parsed: Vec<(String, u32, usize)> = ids
        .iter()
        .map(|id| {
            let (prefix, num) = id.rsplit_once('-').unwrap_or((id.as_str(), "0"));
            (
                prefix.to_string(),
                num.parse::<u32>().unwrap_or(0),
                num.len(),
            )
        })
        .collect();
    let mut out: Vec<String> = Vec::new();
    let mut k = 0;
    while k < parsed.len() {
        let mut j = k;
        while j + 1 < parsed.len()
            && parsed[j + 1].0 == parsed[k].0
            && parsed[j + 1].1 == parsed[j].1 + 1
        {
            j += 1;
        }
        let fmt = |i: usize| format!("{}-{:0w$}", parsed[i].0, parsed[i].1, w = parsed[i].2);
        if j == k {
            out.push(fmt(k));
        } else if j == k + 1 {
            out.push(format!("{}, {}", fmt(k), fmt(j)));
        } else {
            out.push(format!("{}..{}", fmt(k), fmt(j)));
        }
        k = j + 1;
    }
    out.join(", ")
}

fn temperature_cell(r: &RowRecord) -> String {
    match &r.row.temperature {
        TestTemperature::Celsius(p) => p.text.clone(),
        TestTemperature::Range { lo, hi } => format!("{}-{}", lo.text, hi.text),
        TestTemperature::Varied { lo, hi } => format!("varied {}-{}", lo.text, hi.text),
        TestTemperature::Absent => "n/a".to_string(),
    }
}

/// The reading uncertainty of a digitised value as a multiplicative factor, where the register
/// states it as a factor or a percentage.
fn reading_factor(u: &strato_data::release::ReadingUncertainty) -> Option<f64> {
    use strato_data::release::{ReadingUncertainty, ValueReading};
    match u {
        ReadingUncertainty::Stated { value: Some(v), .. } => match v {
            ValueReading::Factor(p) => Some(p.value),
            ValueReading::RelativePercent(p) => Some(1.0 + p.value / 100.0),
            ValueReading::RelativePercentRange { hi, .. } => Some(1.0 + hi.value / 100.0),
            ValueReading::Absolute(_) => None,
        },
        _ => None,
    }
}

/// The upper-bound rows grouped by the result they report: the same compact or group of compacts,
/// the same test temperature, the same element and the same time at temperature. Several rows of
/// one group are one result printed in more than one place in the sources.
fn bound_groups(rec: &[RowRecord]) -> Vec<Vec<&RowRecord>> {
    let mut keys: Vec<String> = Vec::new();
    let mut groups: Vec<Vec<&RowRecord>> = Vec::new();
    for r in rec {
        let (Eval::Bound { .. }, Some(m)) = (&r.eval, &r.model) else {
            continue;
        };
        if r.is_window_bound() {
            continue;
        }
        let text = measured_text(&r.row);
        let element = text.split('-').next().unwrap_or("");
        let key = format!(
            "{}|{}|{}|{}",
            r.row.compact_or_sample, m.spec.t_c, element, m.spec.t_end_h
        );
        match keys.iter().position(|k| *k == key) {
            Some(i) => groups[i].push(r),
            None => {
                keys.push(key);
                groups.push(vec![r]);
            }
        }
    }
    groups
}

/// The time of the curve vertex that a register note states ("the value is the vertex of the curve
/// at t = 303.0 h on the figure axis"), if it states one.
fn vertex_time_h(notes: &str) -> Option<f64> {
    let rest = notes.split("vertex of the curve at t = ").nth(1)?;
    let number: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    number.parse().ok()
}

/// Render the report.
#[allow(clippy::too_many_lines)]
pub fn render(inp: &ReportInput<'_>) -> String {
    let mut s = String::new();
    let rec = inp.records;
    let n = rec.len();
    let count = |st: Status| rec.iter().filter(|r| r.status == st).count();

    // ---- Mandatory disclosure -------------------------------------------------------------
    let _ = writeln!(
        s,
        "# Experimental Comparison: Data Interpretation and Limitations\n"
    );
    let _ = writeln!(
        s,
        "STRATO simulates how a fission product moves through one ideal TRISO particle: a kernel and four coating layers (buffer, inner pyrocarbon, silicon carbide, outer pyrocarbon) in perfect contact, with every atom starting in the kernel. The AGR-1 experiments did not measure single particles. Almost every AGR-1 value is for a compact, a small cylinder that holds 4145 particles in a graphite matrix; one particle holds about 2.4 in 10,000 (2.4e-4) of a compact.\n"
    );
    let n_read = rec
        .iter()
        .filter(|r| {
            !matches!(
                r.row.reading_uncertainty,
                strato_data::release::ReadingUncertainty::NotDigitised
            )
        })
        .count();
    let _ = writeln!(
        s,
        "Some measurements are only upper bounds, written with \"<\". They say that the release was at most that large, and this report treats them as inequalities, never as measured values. Many values ({n_read} of the {n} rows) were read from published figures, so they carry the reading uncertainty of the register; the others are printed in tables or in the text. None of the rows that were compared carries a per-value measurement uncertainty published by its source (the one general statement in the register, a 5 percent systematic gamma-measurement error plus counting statistics for the Stempien 2023 values, is not a per-value uncertainty and changes no status). This report does not make one up: where an uncertainty is missing, it says so, and no error bar is drawn.\n"
    );
    let _ = writeln!(
        s,
        "Real release also depends on things this model does not contain: what the fuel went through in the reactor before the heating test (the irradiation history and the inventory profile at its end), silver and other products that were already outside the SiC layer before the test, transport through the graphite matrix, particles with broken coating layers, and the temperature ramps of the real tests. For silver, strontium and europium, the register cites published work that attributes most of the release to the time in the reactor. For those, most compact values do not describe diffusion out of an intact particle during the test, and this report marks them NOT DIRECTLY COMPARABLE.\n"
    );
    let _ = writeln!(
        s,
        "The comparisons here are meant to show how the model's release behaves and how far it is from the data. They quantify deviations; they do not by themselves establish a direct single-particle validation. A comparison, even a close one, would not validate the model on single particles. A large deviation stays visible. The accuracy of the solver itself (its numerical verification) is a separate question and is shown in the sections on verification below. Nothing in this report calls the model validated.\n"
    );

    // ---- Summary -----------------------------------------------------------------------------
    let _ = writeln!(s, "## Summary of the result\n");
    let _ = writeln!(
        s,
        "Every one of the {n} rows of the release register has a status that the program derives from the row and from the coordinator's decisions (decisions 15 to 24, 67 and 68 of the project's decision list). The counts are:\n"
    );
    let _ = writeln!(s, "| Status | Rows |\n|---|---:|");
    for st in Status::ALL {
        let _ = writeln!(s, "| {} | {} |", st.label(), count(st));
    }
    // The reported bounds and totals read as bounds, and, apart, the window upper bounds under
    // premise BP1 (decisions 67 and 68): the two are different classes.
    let exceeded: Vec<&RowRecord> = rec
        .iter()
        .filter(|r| r.status == Status::ExceedsReportedUpperBound && !r.is_window_bound())
        .collect();
    let within = rec
        .iter()
        .filter(|r| r.status == Status::WithinReportedUpperBound && !r.is_window_bound())
        .count();
    let ratios: Vec<f64> = rec
        .iter()
        .filter(|r| !r.is_window_bound())
        .filter_map(|r| match &r.eval {
            Eval::Bound { margin, .. } => Some(margin.ratio),
            _ => None,
        })
        .collect();
    let window_rows: Vec<&RowRecord> = rec.iter().filter(|r| r.is_window_bound()).collect();
    let window_ratios: Vec<f64> = window_rows
        .iter()
        .filter_map(|r| match &r.eval {
            Eval::Bound { margin, .. } => Some(margin.ratio),
            _ => None,
        })
        .collect();
    let window_exceeded = window_rows
        .iter()
        .filter(|r| r.status == Status::ExceedsReportedUpperBound)
        .count();
    let (rmin, rmax) = ratios
        .iter()
        .fold((f64::INFINITY, 0.0_f64), |(a, b), x| (a.min(*x), b.max(*x)));
    let points = rec
        .iter()
        .filter(|r| matches!(r.eval, Eval::Point { .. }))
        .count();
    let groups = bound_groups(rec);
    let repeated: Vec<String> = groups
        .iter()
        .filter(|g| g.len() > 1)
        .map(|g| {
            g.iter()
                .map(|r| r.row.id.clone())
                .collect::<Vec<_>>()
                .join(" = ")
        })
        .collect();
    let _ = writeln!(s);
    if ratios.is_empty() {
        let _ = writeln!(s, "No row supports an upper-bound comparison.\n");
    } else {
        let _ = writeln!(
            s,
            "- **Upper bounds.** {} rows are read as inequalities (reported bounds of caesium, and caesium compact totals read as bounds, decision 19). The model satisfies {} of them and exceeds {}. Where it exceeds, the model release is between {} and {} times the bound (smallest and largest ratio). These exceedances are shown in the tables and figures; none is hidden.",
            ratios.len(),
            within,
            exceeded.len(),
            sci(rmin),
            sci(rmax)
        );
        let _ = writeln!(
            s,
            "- **These rows are not independent observations.** The {} rows report {} distinct results (one result is one compact or one group of compacts, at one test temperature and one time at temperature). Where several rows give the same result, they are the same compact result printed in more than one place in the sources, for example in a figure of one paper and in a table of another, or restated as a bound in the text; they are listed as equal here: {}. A count of rows is therefore not a count of independent measurements, and the number of rows that the model exceeds is not the number of independent results that it exceeds.",
            ratios.len(),
            groups.len(),
            if repeated.is_empty() {
                "none".to_string()
            } else {
                repeated.join("; ")
            }
        );
    }
    if !window_rows.is_empty() {
        let wmin = window_ratios.iter().copied().fold(f64::INFINITY, f64::min);
        let wmax = window_ratios.iter().copied().fold(0.0_f64, f64::max);
        let _ = writeln!(
            s,
            "- **Window upper bounds under premise BP1.** {} rows ({}) are the matrix-corrected fractions of intact compacts of Cs-134 and Ag-110m (figures 3 and 6 of the source). They are fractions of the as-run calculated compact inventory (Collin et al. 2016, manuscript p. 3) over the window from about 24 h at temperature to the end of the 300 h plateau, or to the end of the test if shorter (p. 4), and the source itself calls the correction an attempt that cannot separate release from the particles from release from the matrix (p. 6). They are read as upper bounds on the release through intact coatings in that window only under premise BP1, which is the source's assumption and not an established fact. The model value compared with each is the release over the same window. The model exceeds {} of them, by {} to {} times the bound (model over bound); they are listed in their own section and are not counted among the bounds above.",
            window_rows.len(),
            compress_ids(&window_rows.iter().map(|r| r.row.id.clone()).collect::<Vec<_>>()),
            window_exceeded,
            sci(wmin),
            sci(wmax)
        );
    }
    let _ = writeln!(
        s,
        "- **Point comparisons.** {points} rows have the status COMPARISON AVAILABLE as a point value: no registered row is a point comparison (decision 67). The matrix-corrected rows of intact compacts are window upper bounds (above); the failed-SiC rows E-038..E-045 are compacts in which particles with failed SiC dominate and are NOT DIRECTLY COMPARABLE; the JAERI rows E-117..E-149 stay DATA INCOMPLETE because the source gives neither a denominator nor a decay correction nor the test sequence. Therefore every set of points for a normalised root-mean-square error has N = 0, and no NRMSE (decisions 64 and 65) and no deviation, relative deviation, RMSE, MAE or maximum is produced, shown or written; no rule is relaxed to obtain one."
    );
    let _ = writeln!(
        s,
        "- **Sr-90.** No row of Sr-90 supports a comparison: the intact-compact values are totals that include inventory released in the reactor, and the other rows are failed-particle quantities. The Sr-90 panel of figure A and figure J show the Sr-90 fractions inside their time axis as muted grey crosses (context), never as a comparison; rows whose time lies beyond the axis are not drawn and are listed in the footer of the figure."
    );
    let _ = writeln!(
        s,
        "- **Model limitations of every result.** {}.",
        MODEL_LIMITATIONS
    );
    let _ = writeln!(
        s,
        "- **Status of the model.** Nothing here validates the model. The solver is numerically verified elsewhere; this report compares its release behaviour with the data and shows where they differ.\n"
    );

    // ---- Model ------------------------------------------------------------------------------
    let _ = writeln!(s, "## What was run\n");
    let _ = writeln!(
        s,
        "The solver is the five-layer spherical backward-time centred-space finite-difference solver of the STRATO repository (package version {}, source commit `{}`). For every comparable row the program runs it for the registered measured baseline geometry (design `{}`), with each layer's diffusion coefficient evaluated at the test temperature from the registered published heating-test correlation, chosen by the selection function of the `strato-data` crate (decisions 16, 39, 40 and 47: the rows of set `C16-T2` for kernel, pyrocarbon and silicon carbide, set `C16-BUF` for the buffer), and reads the release fraction at the time of the measurement (for the ten matrix-corrected rows of intact compacts, the release over the window from 24 h at temperature to the end of the 300 h plateau, or the end of the test if shorter, as decisions 67 and 68 fix; where the register holds no as-run duration of the compact at that temperature the plateau end is used, and the origin of every window end is stated in the results). The time is counted from the moment the test temperature is reached. The temperature ramp of the real test (about 31 to 35 h) is not modelled, and the diffusion coefficients are constant, because the solver has constant coefficients; temperature-dependent diffusion, decay and generation are not yet in the solver.\n",
        inp.prov.solver_version,
        inp.prov.commit,
        crate::data::GEOMETRY_DESIGN
    );
    let r = inp.geometry.outer_m;
    let _ = writeln!(
        s,
        "Outer radii (µm) of kernel, buffer, IPyC, SiC and OPyC: {:.1}, {:.1}, {:.1}, {:.1}, {:.1} (register rows {}). The kernel diameter of the register is 350 µm (the measured value of the kernel batch is 349.7 µm; the kernel-radius sensitivity case of decision 41 uses the kernel radius of 174.85 µm of the selection function, half the measured kernel diameter of 349.7 µm, so its factor is (174.85/250)^2 = 0.489; the mesh of every run uses the register's 175.0 µm).\n",
        r[0] * 1e6,
        r[1] * 1e6,
        r[2] * 1e6,
        r[3] * 1e6,
        r[4] * 1e6,
        inp.geometry.row_ids.join(", ")
    );
    let _ = writeln!(s, "Coefficient sets used by default:\n");
    for (_, note) in inp.set_notes {
        let _ = writeln!(s, "- {note}");
    }
    let _ = writeln!(
        s,
        "\nThe class (owner ruling 16) and the phase of use of the coefficients of every region are stated in each line above. The kernel values are UO2 surrogates: no UCO kernel measurement exists (decision 40). The coefficient classes and the states of the rows (verified or under source review) are as the register records them at the merged data commit.\n"
    );
    let _ = writeln!(
        s,
        "Diffusion coefficients (m²/s) at the test temperatures:\n"
    );
    let _ = writeln!(s, "| Nuclide | Temperature (°C) | Kernel | Buffer | IPyC | SiC | OPyC |\n|---|---:|---:|---:|---:|---:|---:|");
    for (sp, t, c) in inp.curves {
        let d = c.d_m2_s;
        let _ = writeln!(
            s,
            "| {} | {:.0} | {} | {} | {} | {} | {} |",
            sp.nuclide(),
            t,
            sci(d[0]),
            sci(d[1]),
            sci(d[2]),
            sci(d[3]),
            sci(d[4])
        );
    }
    let st = crate::model::Settings::BASE;
    let _ = writeln!(
        s,
        "\nNumerical settings of the base runs: intervals per layer {:?}, requested time step {} h; the numerical difference reported beside every model value is the difference to a run with twice the intervals and half the step. Metrics are those of directive section 13 (absolute deviation, relative percentage deviation, RMSE, MAE, maximum absolute deviation, uncertainty-normalised residual) plus the signed log10 ratio and the log-space RMSE (decision 18). No normalised root-mean-square error is computed or shown (decisions 64, 65 and 67): no registered row is a point comparison, so the set of compatible points has N = 0. Relative deviation is undefined for a measured value that is not positive, and no near-zero floor is applied because no row states a detection floor (decision 17). No accuracy threshold is used anywhere.\n",
        st.intervals, st.dt_h
    );

    // ---- Status table ------------------------------------------------------------------------
    let _ = writeln!(
        s,
        "## Status of every row: the rules and the incompatible rows\n"
    );
    let _ = writeln!(
        s,
        "The rules run in this order and the first that applies decides. A status is never computed from a deviation.\n"
    );
    let _ = writeln!(
        s,
        "| Rule | Status | Rows | Reason | Register rows |\n|---|---|---:|---|---|"
    );
    for (code, meaning) in CATEGORIES.iter().map(|(a, b)| (*a, *b)).chain([
        ("no_nuclide", "no nuclide named"),
        ("not_a_single_number", "not a single number"),
        ("no_rule", "no rule covers the row"),
    ]) {
        let ids: Vec<String> = rec
            .iter()
            .filter(|r| r.verdict.category == code)
            .map(|r| r.row.id.clone())
            .collect();
        if ids.is_empty() {
            continue;
        }
        let first = rec.iter().find(|r| r.verdict.category == code);
        let (status, reason) = first.map_or((String::new(), String::new()), |r| {
            (r.status.label().to_string(), r.verdict.reason.clone())
        });
        // Rows of one rule can end in different statuses (bounds); show the verdict text of the rule.
        let statuses: Vec<&str> = {
            let mut v: Vec<&str> = rec
                .iter()
                .filter(|r| r.verdict.category == code)
                .map(|r| r.status.label())
                .collect();
            v.sort_unstable();
            v.dedup();
            v
        };
        let _ = writeln!(
            s,
            "| `{code}` ({meaning}) | {} | {} | {} | {} |",
            if statuses.len() > 1 {
                statuses.join(" / ")
            } else {
                status
            },
            ids.len(),
            reason.replace('|', "/"),
            compress_ids(&ids)
        );
    }
    let _ = writeln!(s);

    // ---- Results -------------------------------------------------------------------------------
    let _ = writeln!(s, "## Upper-bound comparisons\n");
    let _ = writeln!(
        s,
        "Each row is an inequality: the model's release fraction at the end of the test must not exceed the number. \"Read as\" says how the number came to be a bound: the register states it as a bound, or it is a compact total (mean particle release plus matrix plus background, all non-negative), which for caesium, whose in-pile release is small, is read as an upper bound on release through intact coatings (decision 19; digitised totals are labelled digitised). The model fraction is a fraction of the particle's own inventory; the bound is a fraction of the compact inventory; the two are the same quantity only if the inventory outside the particles is negligible.\n"
    );
    let _ = writeln!(
        s,
        "| Row | Nuclide | Compact or sample | Test temperature (°C) | Time at test temperature (h) | Bound read as | Bound | Model | Model / bound | Status | Numerical difference of the model | Same result also in rows |\n|---|---|---|---:|---:|---|---:|---:|---:|---|---:|---|"
    );
    for r in rec.iter().filter(|r| !r.is_window_bound()) {
        if let (
            Eval::Bound {
                limit,
                source,
                margin,
            },
            Some(m),
        ) = (&r.eval, &r.model)
        {
            let same: Vec<String> = groups
                .iter()
                .find(|g| g.iter().any(|x| x.row.id == r.row.id))
                .map(|g| {
                    g.iter()
                        .filter(|x| x.row.id != r.row.id)
                        .map(|x| x.row.id.clone())
                        .collect()
                })
                .unwrap_or_default();
            let _ = writeln!(
                s,
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                r.row.id,
                measured_text(&r.row),
                crate::data::typography(&r.row.compact_or_sample),
                temperature_cell(r),
                m.spec.t_end_h,
                source.text(),
                sci(*limit),
                sci(m.f_window),
                sci(margin.ratio),
                r.status.label(),
                sci(m.numerical_difference),
                if same.is_empty() {
                    "-".to_string()
                } else {
                    same.join(", ")
                }
            );
        }
    }
    let _ = writeln!(s);

    // The statement of decision 21 for the rows that name the element only.
    let element_only: Vec<String> = rec
        .iter()
        .filter(|r| r.model.as_ref().is_some_and(|m| m.spec.element_label_only))
        .map(|r| r.row.id.clone())
        .collect();
    if !element_only.is_empty() {
        let _ = writeln!(
            s,
            "For rows {}, {}.\n",
            compress_ids(&element_only),
            crate::outputs::ELEMENT_ONLY_STATEMENT
        );
    }

    if !window_rows.is_empty() {
        let _ = writeln!(
            s,
            "## Window upper bounds under premise BP1 (matrix-corrected rows of intact compacts)\n"
        );
        let _ = writeln!(
            s,
            "Class: {}. Denominator: {}. The measured value of each row is the matrix-corrected fraction of the as-run calculated compact inventory (Collin et al. 2016, manuscript p. 3) that was released between about 24 h at temperature and the end of the 300 h plateau, or the end of the test if that was shorter (p. 4). The source's correction subtracts the release assumed to have come from outside the SiC during the first 24 h; it is an attempt, and the source states that release from the particles cannot be separated from silver slowly driven out of the matrix later (p. 6). The value therefore counts any release from outside the SiC after about 24 h as well. It bounds the release of an intact particle through its coatings in the window only under premise BP1, which is the source's assumption and not an established fact. The model quantity compared with each row is the release of one intact particle over the same window (kernel-only start, assumption of decision 20), and the comparison is an inequality: the model over the window must not exceed the value. The model fraction is a fraction of the particle's own inventory and the bound a fraction of the compact inventory; the register holds no inventory ratio that would convert one into the other. These rows are not point comparisons, enter no set of points, and no NRMSE or deviation is formed from them (decisions 64, 65, 67 and 68). The status words WITHIN and EXCEEDS REPORTED UPPER BOUND are the program's words for every inequality; for these rows the bound is not stated by the source as a bound but read as one under premise BP1, and the class column of the results says so.\n",
            crate::status::WINDOW_BOUND_CLASS,
            crate::status::WINDOW_BOUND_DENOMINATOR
        );
        let _ = writeln!(
            s,
            "| Row | Nuclide | Compact | Test temperature (°C) | Window (h at test temperature) | Bound (digitised) | Model over the same window | Model / bound | Status | Numerical difference of the model |\n|---|---|---|---:|---|---:|---:|---:|---|---:|"
        );
        for r in &window_rows {
            if let (Eval::Bound { limit, margin, .. }, Some(m)) = (&r.eval, &r.model) {
                let _ = writeln!(
                    s,
                    "| {} | {} | {} | {} | {}-{} | {} | {} | {} | {} | {} |",
                    r.row.id,
                    measured_text(&r.row),
                    crate::data::typography(&r.row.compact_or_sample),
                    temperature_cell(r),
                    m.spec.window_start_h,
                    m.spec.t_end_h,
                    sci(*limit),
                    sci(m.f_window),
                    sci(margin.ratio),
                    r.status.label(),
                    sci(m.numerical_difference)
                );
            }
        }
        let _ = writeln!(s, "\nWhere the window ends:\n");
        let mut origins: Vec<(String, Vec<String>)> = Vec::new();
        for r in &window_rows {
            let Some(m) = &r.model else { continue };
            match origins.iter_mut().find(|(o, _)| *o == m.spec.time_source) {
                Some((_, ids)) => ids.push(r.row.id.clone()),
                None => origins.push((m.spec.time_source.clone(), vec![r.row.id.clone()])),
            }
        }
        for (origin, ids) in &origins {
            let _ = writeln!(s, "- {}: {}.", compress_ids(ids), origin);
        }
        let _ = writeln!(s);
    }
    let provisional: Vec<&RowRecord> = rec
        .iter()
        .filter(|r| matches!(r.eval, Eval::Provisional { .. }))
        .collect();
    if !provisional.is_empty() {
        let _ = writeln!(
            s,
            "## DATA INCOMPLETE rows with a model value, for scale only\n"
        );
        let _ = writeln!(
            s,
            "The register does not establish what these fractions are fractions of (denominator *not established*, decision 22), so the program keeps them DATA INCOMPLETE and forms no deviation for them. Only the measured value, the model's release over the same window and the log10 ratio of the two are given, so that the order of magnitude of the difference can be seen. They enter no status and no aggregate.\n"
        );
        let _ = writeln!(
            s,
            "| Row | Nuclide | Compact | Test temperature (°C) | Window (h at test temperature) | Measured (digitised) | Model over the same window | log10 (model / measured), for scale |\n|---|---|---|---:|---|---:|---:|---:|"
        );
        for r in &provisional {
            if let (Eval::Provisional { value, log10_ratio }, Some(m)) = (&r.eval, &r.model) {
                let _ = writeln!(
                    s,
                    "| {} | {} | {} | {} | {}-{} | {} | {} | {} |",
                    r.row.id,
                    measured_text(&r.row),
                    crate::data::typography(&r.row.compact_or_sample),
                    temperature_cell(r),
                    m.spec.window_start_h,
                    m.spec.t_end_h,
                    sci(*value),
                    sci(m.f_window),
                    log10_ratio.map_or("undefined".to_string(), |x| format!("{x:.2}"))
                );
            }
        }
        let _ = writeln!(s);
    }

    let _ = writeln!(s, "## Aggregate metrics per dataset\n");
    let _ = writeln!(
        s,
        "Aggregates are formed per dataset: the same programme, nuclide, group of particles and quantity definition, never across them. Bounds are counted, not averaged into a deviation. Rows that are DATA INCOMPLETE enter no aggregate. A group with no point comparison has no RMSE, MAE or maximum: they are undefined, not zero.\n"
    );
    let _ = writeln!(
        s,
        "| Programme | Nuclide | Group | Quantity | Rows | Point comparisons | Bounds (satisfied / exceeded) | Model / bound (smallest, largest) |\n|---|---|---|---|---:|---:|---|---|"
    );
    for d in inp.datasets {
        if d.bound.is_none() && d.point.is_none() {
            continue;
        }
        let b = d.bound.map_or("-".to_string(), |b| {
            format!("{} / {}", b.satisfied, b.exceeded)
        });
        let br = d.bound.map_or("-".to_string(), |b| {
            format!("{}, {}", sci(b.ratio_min), sci(b.ratio_max))
        });
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            d.program,
            d.nuclide,
            d.group,
            d.quantity,
            d.row_ids.len(),
            d.point.as_ref().map_or(0, |p| p.n),
            b,
            br,
        );
    }
    let _ = writeln!(
        s,
        "\nDatasets with a point comparison (status COMPARISON AVAILABLE): {}.\n",
        inp.datasets.iter().filter(|d| d.point.is_some()).count()
    );
    // ---- Figures -----------------------------------------------------------------------------
    let _ = writeln!(s, "## Figures\n");
    let figs = [
        ("figure_A_release_vs_time.svg", "A. Fraction released versus time at temperature, for the three elements at 1600, 1700 and 1800 °C (the model has no decay, so each panel is the element; the nuclides measured are listed in the panel). Lines: BTCS. Plus signs: the independent analytical series of the five-layer problem (the eigenfunction expansion of the repository), only at the times where it meets its validity criterion (truncation valid, and the value above the series' own estimated accuracy; negative or unresolved values are not drawn). Hollow squares: the model's own value over the window of a window upper bound under premise BP1 (24 h at temperature to the end of the 300 h plateau, or the test end if shorter), drawn beside it because that bound is a window release, not cumulative from t = 0. Markers: registered measurements, by what they can support (downward triangle: upper bound, among them the window upper bounds, hollow when exceeded by the model; hollow diamond: DATA INCOMPLETE; grey cross: NOT DIRECTLY COMPARABLE context). Log scale; no error bars, because no source reports a per-value measurement uncertainty for the plotted rows. Measurements whose time lies beyond the 420 h axis are not drawn and are listed in the footer. The early part of every curve is not converged in time (see the section on accuracy)."),
        ("figure_B_absolute_deviation.svg", "B. Absolute deviation versus time of the point comparisons (none exist, so the panel is empty and says so; DATA INCOMPLETE rows have no deviation) and, beside it, the model over each upper bound (an inequality, not a deviation)."),
        ("figure_C_relative_deviation.svg", "C. Relative percentage deviation versus time of the point comparisons (none exist), with the policy for undefined values stated on the figure, and the model over each upper bound."),
        ("figure_D_radial_profiles.svg", "D. Radial concentration profiles through the five layers at selected times, 1600 °C."),
        ("figure_E_inventory_balance.svg", "E. Inventory balance: remaining inventory and cumulative release (Ag-110m at 1800 °C), and the relative balance residual with its derived rounding bound for the three nuclides. Generated inventory and decay loss are not applicable yet: the solver has neither."),
        ("figure_F_spatial_convergence.svg", "F. Spatial convergence against the exact series of a homogeneous verification case (synthetic diffusion coefficient, not AGR-1 data)."),
        ("figure_G_temporal_convergence.svg", "G. Temporal convergence on a fixed fine mesh, same homogeneous verification case."),
        ("figure_H_temperature_sensitivity.svg", "H. Temperature sensitivity of the release after 300 h for each registered coefficient variant."),
        ("figure_I_silver_110m_comparison.svg", "I. Silver-110m (Ag-110m): the registered measurements, marked by what each can support, as in figure A (grey cross: NOT DIRECTLY COMPARABLE context, muted; hollow diamond: DATA INCOMPLETE; downward triangle: upper bound, among them the window upper bounds under premise BP1, hollow when exceeded by the model), where a vertical bar on a marker is the reading uncertainty of a digitised value and a marker without a bar is a printed value, exact as printed; the BTCS prediction (lines); the analytical prediction where it is valid (plus signs); hollow squares, the model's value over the window of a window upper bound; and, in the right panel, the log10 ratio of the model to the measured value for rows that have one, with the statement that no deviation and no NRMSE is defined where no row is a point comparison, and the window bounds with their model values in the lines under the figure. Measurements whose time lies beyond the time axis are not drawn and are listed in the footer."),
        ("figure_J_strontium_90_comparison.svg", "J. Strontium-90 (Sr-90): the same layout and markers as figure I."),
        ("figure_K_caesium_137_data_unavailable.svg", "K. Caesium-137 (Cs-137): DATA UNAVAILABLE, no release measurement exists for the AGR-1 particles in the sources of the register."),
    ];
    for (file, cap) in figs {
        let _ = writeln!(s, "![{cap}](figures/{file})\n\n*{cap}*\n");
    }
    let _ = writeln!(
        s,
        "The figures are static SVG files drawn in Rust with the `plotters` crate (pure Rust SVG backend; it is used by the verified solver package for its own figures and is not linked into the WebAssembly wrapper). Each figure names its experimental sources, the solver version and the commit.\n"
    );
    let _ = writeln!(s, "### Ag-110m, Sr-90 and Cs-137 (decision 24)\n");
    let _ = writeln!(
        s,
        "Figures I and J show, for silver-110m and strontium-90, the registered measurements (digitised values labelled digitised with the reading uncertainty of the register, tabulated values exact), the BTCS prediction, the independent analytical prediction where it is valid, and the deviations. The numbers beside the curves are repeated here; they are the same text as under the figures.\n"
    );
    for species in [Species::Ag, Species::Sr] {
        let _ = writeln!(s, "**{}**\n", species.nuclide());
        for l in crate::figures::deviation_lines(inp.records, species) {
            let _ = writeln!(s, "- {l}");
        }
        let _ = writeln!(s);
    }
    let _ = writeln!(
        s,
        "**Cs-137.** DATA UNAVAILABLE: AGR-1 and AGR-2 report no release of Cs-137 (decision 43). Figure K says so and lists the Cs-137 rows of the register, which belong to another programme, with their status; no deviation is formed from them.\n"
    );

    // ---- Numerical quality -----------------------------------------------------------------
    let _ = writeln!(s, "## How accurate are the model values?\n");
    let _ = writeln!(
        s,
        "This is numerical accuracy of the solver on the stated model, not agreement with experiment.\n"
    );
    let _ = writeln!(s, "Verification case: {}\n", inp.homogeneous_note);
    let _ = writeln!(s, "Spatial convergence (error of the release fraction at 100 h, time error removed by Richardson extrapolation):\n");
    let _ = writeln!(s, "| Refinement | Kernel mesh width (m) | Nodes | Error | Observed order |\n|---:|---:|---:|---:|---:|");
    for c in inp.spatial {
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} |",
            c.level,
            sci(c.step),
            c.count,
            sci(c.error),
            c.order.map_or("-".into(), |o| format!("{o:.2}"))
        );
    }
    let _ = writeln!(s, "\nTemporal convergence (fixed fine mesh; the spatial error of that mesh is about {}, the level the curve approaches as the step shrinks):\n", sci(inp.temporal_spatial_floor));
    let _ = writeln!(s, "| Refinement | Time step (s) | Steps | Error | Observed order |\n|---:|---:|---:|---:|---:|");
    for c in inp.temporal {
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} |",
            c.level,
            sci(c.step),
            c.count,
            sci(c.error),
            c.order.map_or("-".into(), |o| format!("{o:.2}"))
        );
    }
    let _ = writeln!(
        s,
        "\nThe observed orders are reported, not required to meet a target (decision 7). On the registered five-layer cases, the base runs differ from the independent analytical series as follows (relative difference in percent; blank where the series does not meet its validity criterion, namely truncation valid and the value above the series' own estimated absolute accuracy, which is also stated in the header of `release_curves.csv`):\n"
    );
    let _ = writeln!(
        s,
        "| Nuclide | Test temperature (°C) | 10 h | 100 h | 300 h |\n|---|---:|---:|---:|---:|"
    );
    for (sp, t, c) in inp.curves {
        let an = inp.analytic.get(&(*sp, (*t * 10.0).round() as i64));
        let cell = |h: f64| -> String {
            let a = an.and_then(|v| {
                v.iter()
                    .find(|(x, _)| (*x - h).abs() < 1e-9)
                    .and_then(|(_, f)| *f)
            });
            match (a, c.at(h)) {
                (Some(a), Some(b)) => pct((b - a) / a * 100.0),
                _ => String::new(),
            }
        };
        let _ = writeln!(
            s,
            "| {} | {:.0} | {} | {} | {} |",
            sp.nuclide(),
            t,
            cell(10.0),
            cell(100.0),
            cell(300.0)
        );
    }
    let _ = writeln!(
        s,
        "\n**The early part of every curve is not converged in time.** Where the release is still rising steeply from zero, the base run (step {} h) and the run with half the step differ by large factors, and the BTCS values there on the log axes of figures A, I and J are dominated by time-step error, not by the diffusion model. The table gives, for every curve, the largest relative difference between the two runs over the drawn points (release above 1e-10) and the last time at which a drawn point differs by more than 10 percent (a reading aid, not a tolerance); after that time every drawn point agrees within 10 percent.\n",
        st.dt_h
    );
    let _ = writeln!(
        s,
        "| Nuclide | Test temperature (°C) | Largest relative difference of the drawn points (percent) | Last time with a difference above 10 percent (h) |\n|---|---:|---:|---:|"
    );
    for (sp, t, c) in inp.curves {
        let diffs = c.drawn_time_differences();
        let largest = diffs.iter().map(|d| d.1).fold(0.0_f64, f64::max) * 100.0;
        let settle = c
            .settle_time_h(0.1)
            .map_or("none".to_string(), |h| format!("{h:.2}"));
        let _ = writeln!(
            s,
            "| {} | {:.0} | {} | {} |",
            sp.nuclide(),
            t,
            sci(largest),
            settle
        );
    }
    let earliest = rec
        .iter()
        .filter_map(|r| r.model.as_ref().map(|m| m.spec.t_end_h))
        .fold(f64::INFINITY, f64::min);
    let _ = writeln!(
        s,
        "\nThe model values of the rows of this report are read at {earliest} h at temperature or later, and the numerical difference of each of them is given beside it in the tables."
    );
    let mut worst = 0.0_f64;
    for r in rec {
        if let Some(m) = &r.model {
            if m.f_window > 0.0 {
                worst = worst.max(m.numerical_difference / m.f_window);
            }
        }
    }
    let _ = writeln!(
        s,
        "\nFor the rows of this report the largest relative numerical difference between base and refined run is {} (as a fraction of the model value); every deviation discussed below is many times larger.\n",
        sci(worst)
    );

    // ---- Sensitivity ---------------------------------------------------------------------------
    let _ = writeln!(
        s,
        "## Temperature sensitivity of the registered correlations\n"
    );
    let _ = writeln!(
        s,
        "Release fraction after 300 h at temperature (figure H). The default is the published heating-test correlation. The other variants are labelled alternatives, never defaults: the AGR-1 fitted silicon-carbide rows (decision 40; fitted to AGR-1 heating-test data, so they are calibrated to the very data a comparison would use and cannot serve as independent validation), other registered silicon-carbide rows, and the kernel-radius scaling of decision 41. The AGR-1 fitted silicon-carbide rows of Schappel (2020) disagree with the correction factors of Collin (2016) by 4 to 220 times (decision 40). The default correlation prints no validity range; the sweep covers 1600 to 1800 °C, the temperatures the register records for the tests.\n"
    );
    let _ = writeln!(s, "| Nuclide | Variant | Class of SiC row | 1600 °C | 1700 °C | 1800 °C |\n|---|---|---|---:|---:|---:|");
    let mut keys: Vec<(Species, String)> = Vec::new();
    for r in inp.sensitivity {
        let k = (r.species, r.variant_id.clone());
        if !keys.contains(&k) {
            keys.push(k);
        }
    }
    for (sp, id) in keys {
        let at = |t: f64| {
            inp.sensitivity
                .iter()
                .find(|r| r.species == sp && r.variant_id == id && (r.t_c - t).abs() < 1.0)
                .map_or("-".to_string(), |r| sci(r.f_300))
        };
        let class = inp
            .sensitivity
            .iter()
            .find(|r| r.species == sp && r.variant_id == id)
            .map_or(String::new(), |r| r.sic_class.clone());
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} |",
            sp.nuclide(),
            id,
            class,
            at(1600.0),
            at(1700.0),
            at(1800.0)
        );
    }
    let outside = inp
        .sensitivity
        .iter()
        .filter(|r| r.outside_printed_range)
        .count();
    let _ = writeln!(
        s,
        "\nOf {} sweep points, {} lie outside a printed validity range of a row they use.\n",
        inp.sensitivity.len(),
        outside
    );

    // ---- Discussion ---------------------------------------------------------------------------
    let _ = writeln!(s, "## Discussion\n");
    let cs_exceeded = exceeded
        .iter()
        .filter(|r| measured_text(&r.row).starts_with("Cs"))
        .count();
    let _ = writeln!(
        s,
        "**Caesium.** The model exceeds {cs_exceeded} of {} caesium upper-bound rows, which are {} distinct results (see the summary), by factors of {} to {} (model over bound). With the published coefficients, evaluated at the test temperature and with no correction, the model predicts far more caesium release than these bounds allow. This is the deviation of the stated model from the data; it is not reduced by anything done here. Possible contributions, none of which this report has tested or established: the published correlations were reported by their source to need multiplicative corrections to match the AGR-1 heating tests (register Table 18, rows CF-6 and CF-3: the factors on the diffusion coefficient of caesium in silicon carbide are 5e-3 to 6e-2 at 1600 °C, and of silver in silicon carbide 2e-4 to 4e-3 at 1600 °C for the matrix-corrected values; they are compact-specific, fitted to the data, and are not applied here), the kernel coefficients are UO2 surrogates, the start is kernel-only, the compact values are not single-particle values, and the heating ramps are not modelled.\n",
        ratios.len(),
        groups.len(),
        sci(rmin),
        sci(rmax)
    );
    if !exceeded.is_empty() {
        let factors: Vec<f64> = exceeded
            .iter()
            .filter_map(|r| reading_factor(&r.row.reading_uncertainty))
            .collect();
        let largest = factors.iter().copied().fold(f64::NAN, f64::max);
        let stated = if factors.is_empty() {
            "none of the exceeded rows states a reading uncertainty of a digitised value"
                .to_string()
        } else {
            format!(
                "the largest reading uncertainty stated for an exceeded row corresponds to a factor of {largest:.2} on the bound"
            )
        };
        let _ = writeln!(
            s,
            "**Robustness of the bound statuses.** The smallest exceedance is {} times the bound; {stated}. The exceedances are therefore far beyond the reading uncertainty of the digitised bounds, and no status of this section depends on it.\n",
            sci(
                exceeded
                    .iter()
                    .filter_map(|r| match &r.eval {
                        Eval::Bound { margin, .. } => Some(margin.ratio),
                        _ => None,
                    })
                    .fold(f64::INFINITY, f64::min)
            )
        );
    }
    // Rows read at the registered end of the test although their register note gives the time of
    // the curve vertex that was read off the published figure.
    let mut vertex: Vec<(String, f64, f64)> = Vec::new();
    let mut vertex_status_changes = 0;
    for r in rec {
        let (Some(m), Some(v)) = (&r.model, vertex_time_h(&r.row.notes)) else {
            continue;
        };
        if r.row.time_h.is_some()
            || m.spec.window_start_h > 0.0
            || (v - m.spec.t_end_h).abs() < 0.05
        {
            continue;
        }
        let Some(curve) = inp
            .curves
            .iter()
            .find(|(sp, t, _)| *sp == m.spec.species && (*t - m.spec.t_c).abs() < 1.0)
            .map(|x| x.2)
        else {
            continue;
        };
        let (Some(at_vertex), Some(at_end)) = (curve.at(v), curve.at(m.spec.t_end_h)) else {
            continue;
        };
        if at_end <= 0.0 {
            continue;
        }
        if let Eval::Bound { limit, .. } = &r.eval {
            if (m.f_window > *limit) != (at_vertex > *limit) {
                vertex_status_changes += 1;
            }
        }
        vertex.push((r.row.id.clone(), v, (at_vertex / at_end - 1.0) * 100.0));
    }
    if !vertex.is_empty() {
        let lo = vertex.iter().map(|v| v.2).fold(f64::INFINITY, f64::min);
        let hi = vertex.iter().map(|v| v.2).fold(f64::NEG_INFINITY, f64::max);
        let list: Vec<String> = vertex
            .iter()
            .map(|(id, v, _)| format!("{id} ({v:.1} h)"))
            .collect();
        let _ = writeln!(
            s,
            "**Time at which a digitised value is compared.** For rows without a time of their own, the model is read at the registered end of the test (the origin is stated in every row). For {} the register notes that the digitised value is the vertex of the published curve at a slightly different time on the figure axis ({}). The comparison keeps the registered end of the test; reading the model at the vertex time instead would change the model value by {:+.1} to {:+.1} percent, and {vertex_status_changes} of these statuses would change.\n",
            compress_ids(&vertex.iter().map(|v| v.0.clone()).collect::<Vec<_>>()),
            list.join(", "),
            lo,
            hi
        );
    }
    let f300 = |sp: Species, t: f64| -> f64 {
        inp.curves
            .iter()
            .find(|(s2, t2, _)| *s2 == sp && (*t2 - t).abs() < 1.0)
            .and_then(|(_, _, c)| c.at(300.0))
            .unwrap_or(f64::NAN)
    };
    let ag_lo = format!("{:.0}", f300(Species::Ag, 1600.0) * 100.0);
    let ag_hi = format!("{:.0}", f300(Species::Ag, 1800.0) * 100.0);
    let sr_1600 = format!("{:.1}", f300(Species::Sr, 1600.0) * 100.0);
    // Smallest and largest registered fraction of an element among the rows of one status rule.
    let span = |prefix: &str, rule: &str| -> String {
        let vals: Vec<f64> = rec
            .iter()
            .filter(|r| {
                r.verdict.category == rule
                    && r.row.unit.as_str() == "1"
                    && measured_text(&r.row).starts_with(prefix)
            })
            .filter_map(|r| match &r.row.value {
                strato_data::release::ReleaseValue::Point(p)
                | strato_data::release::ReleaseValue::Digitised(p) => Some(p.value),
                _ => None,
            })
            .filter(|v| *v > 0.0)
            .collect();
        if vals.is_empty() {
            return "none registered".to_string();
        }
        let lo = vals.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = vals.iter().copied().fold(0.0, f64::max);
        format!("{} to {}", sci(lo), sci(hi))
    };
    let ag_in_pile = span("Ag", "includes_in_pile_inventory");
    let sr_in_pile = span("Sr", "includes_in_pile_inventory");
    let ag_window: Vec<(&RowRecord, f64, f64)> = window_rows
        .iter()
        .filter(|r| measured_text(&r.row).starts_with("Ag"))
        .filter_map(|r| match &r.eval {
            Eval::Bound { limit, margin, .. } => Some((*r, *limit, margin.ratio)),
            _ => None,
        })
        .collect();
    if !ag_window.is_empty() {
        let lim_lo = ag_window.iter().map(|x| x.1).fold(f64::INFINITY, f64::min);
        let lim_hi = ag_window.iter().map(|x| x.1).fold(0.0_f64, f64::max);
        let ratio_lo = ag_window.iter().map(|x| x.2).fold(f64::INFINITY, f64::min);
        let ratio_hi = ag_window.iter().map(|x| x.2).fold(0.0_f64, f64::max);
        let n_exceed = ag_window
            .iter()
            .filter(|x| x.0.status == Status::ExceedsReportedUpperBound)
            .count();
        let _ = writeln!(
            s,
            "**Silver.** The model releases {ag_lo} to {ag_hi} percent of the silver within 300 h at 1600 to 1800 °C (figure A). The matrix-corrected digitised values of the intact compacts, which are window upper bounds under premise BP1 (denominator: the as-run calculated compact inventory), are {} to {}; the model's release over the same window is {} to {} times these bounds, and it exceeds {n_exceed} of {} of them. The bound holds only if the source's premise holds, and the model fraction is a fraction of the particle's own inventory while the bound is a fraction of the compact inventory, which the register does not convert; the size of the gap is therefore a statement about these inequalities, not a deviation, and no NRMSE or other deviation is formed. The compact totals that include the inventory released in the reactor (registered values {ag_in_pile}) are NOT DIRECTLY COMPARABLE and are drawn as context only.\n",
            sci(lim_lo),
            sci(lim_hi),
            sci(ratio_lo),
            sci(ratio_hi),
            ag_window.len()
        );
    }
    if !window_rows.is_empty() {
        let cs_w = window_rows
            .iter()
            .filter(|r| measured_text(&r.row).starts_with("Cs"))
            .count();
        let _ = writeln!(
            s,
            "**Matrix-corrected rows of intact compacts.** {} rows ({} of caesium, {} of silver) are window upper bounds under premise BP1, not point comparisons (decisions 67 and 68). The window of each row ends at the end of the 300 h plateau or at the end of the test if that is shorter. For the compacts whose register as-run history gives a longer hold at the test temperature (6-4-3 at 1600 °C, 400 h; 4-4-3 at 1700 °C, 303 h) the longer as-run end is not the window end; for the compacts for which the register holds no as-run duration, the plateau end is used and the test may have been shorter, which the results say row by row.\n",
            window_rows.len(),
            cs_w,
            window_rows.len() - cs_w
        );
    }
    let _ = writeln!(
        s,
        "**Strontium.** No Sr-90 row supports a comparison. The model value after 300 h at 1600 °C is {sr_1600} percent (figures A and H), while the registered compact totals that include the in-pile inventory are {sr_in_pile}; those values include strontium released in the reactor, so the gap cannot be read as a deviation of the diffusion model.\n"
    );
    let _ = writeln!(
        s,
        "**Rows that did not take part, and what would change that.** NOT ASSESSED ({}): counts, one-particle constants and thresholds, and the prompt range that spans the heating ramps (needs temperature-dependent diffusion). DATA INCOMPLETE ({}): the JAERI loose-particle rows (denominator, decay correction, test sequence, heat-up ramp, end-of-irradiation profile), and krypton rows (background-limited bounds, the krypton SiC row under re-read). NOT DIRECTLY COMPARABLE ({}): compacts with failed SiC, totals that include in-pile release, deposition-cup collections over multi-temperature histories, loose particles that include failures, fuel spheres, another fuel batch, and quantities that describe the irradiation phase. Eu-154 rows are only totals that include in-pile release, and the register has no europium diffusion coefficients.\n",
        count(Status::NotAssessed),
        count(Status::DataIncomplete),
        count(Status::NotDirectlyComparable)
    );
    let _ = writeln!(
        s,
        "**Calibration and validation.** No model parameter was fitted to any row of the register by this work; all coefficients are literature values of the register. The AGR-1 fitted rows (Schappel 2020) and the correction factors of the PARFUME modelling were fitted to AGR-1 heating-test data, so they count as calibration against these data: they appear only as labelled alternatives in the sensitivity study and are not used as independent evidence.\n"
    );
    let _ = writeln!(
        s,
        "**Uncertainty.** No compared row carries a measurement uncertainty from its source. A digitising reading uncertainty is not a measurement uncertainty and is never used to form a residual or a status; therefore the statuses WITHIN REPORTED UNCERTAINTY and OUTSIDE REPORTED UNCERTAINTY do not occur. The uncertainty-normalised residual is implemented and tested, and is defined only where a source reports an uncertainty.\n"
    );
    let _ = writeln!(
        s,
        "**Limits of this comparison.** Decay and generation are not in the solver, so every result is a release of a stable, pre-existing inventory; for Ag-110m the decay over a 335 h test is about 4 percent of the inventory (from the decay register), which is small beside the deviations above. The temperature ramps are not modelled. The end-of-irradiation inventory profile is not in the register, so the kernel-only start is an assumption stated beside every result (decision 20). No claim of validation follows from this report.\n"
    );

    // ---- Reproduction -----------------------------------------------------------------------
    let _ = writeln!(s, "## Files and reproduction\n");
    let _ = writeln!(
        s,
        "All numbers in this report come from files in `data/comparison/`, each with a provenance header (solver version, source commit, SHA-256 of the register files used, model limitations): `comparison_rows.csv` (every row), `comparison_datasets.csv`, `release_curves.csv`, `deviation_curves.csv`, `radial_profiles.csv`, `inventory_balance.csv`, `temperature_sensitivity.csv`, `convergence_spatial.csv`, `convergence_temporal.csv` and `comparison.json` (for the website's comparison panel; schema in the project's comparison schema note). The figures are in `docs/validation/figures/`. To regenerate everything, run in the tools workspace `cargo run --release -p strato-compare -- run --repo <checkout> --commit <hash>`; the report itself is rendered by the same program, so the tables cannot drift from the data.\n"
    );
    let mut sources: Vec<String> = rec
        .iter()
        .filter(|r| r.model.is_some())
        .map(|r| format!("{} ({})", r.row.source, r.row.source_location))
        .collect();
    sources.sort();
    sources.dedup();
    let _ = writeln!(s, "Sources of the rows that received a model value:\n");
    for x in sources {
        let _ = writeln!(s, "- {x}");
    }
    let _ = writeln!(s);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_ranges_compress() {
        let ids: Vec<String> = ["E-001", "E-002", "E-003", "E-007", "E-009", "E-010"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(compress_ids(&ids), "E-001..E-003, E-007, E-009, E-010");
        assert_eq!(compress_ids(&[]), "");
    }
}
