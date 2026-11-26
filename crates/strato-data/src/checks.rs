//! The consistency checks of the Python audit scripts, reproduced in Rust.
//!
//! Every check that `scripts/derivation_checks/D5_parameter_register.py`,
//! `b4_decay_source_pseudocode.py`, `b4_phase2_pseudocode_integration.py`,
//! `scripts/data_register/b6_nuclear_crosscheck.py` (the part that does not need the network),
//! `scripts/web/generate_data.py` (geometry extraction) and `scripts/web/check_catalogue.py`
//! (rule 6) make on the six register files has a counterpart here with the same tolerance. Where the
//! Python script used symbolic algebra (SymPy) the check is numerical or exact-decimal, and says so
//! in its description. `CHECKS.md` in the crate folder lists each check with its Python source line
//! and its Rust test.
//!
//! A check id has the form `<script>.<family>.<name>`: `D5` is D5_parameter_register.py, `B4D` is
//! b4_decay_source_pseudocode.py, `B4P` is b4_phase2_pseudocode_integration.py, `B6` is
//! b6_nuclear_crosscheck.py, `GEN` is generate_data.py, `CAT` is check_catalogue.py, `LOAD` is a
//! check that is new in Rust (the typed reader keeps every row).

use std::collections::{BTreeMap, HashSet};
use std::fmt;

use crate::arrhenius::{Arrhenius, ArrheniusRow, FluenceFactor, GasConstant, Law};
use crate::common::{EvidenceClass, Status};
use crate::csv;
use crate::decay::{decay_constant, Decay, Stability, TimeUnit};
use crate::decimal::Decimal;
use crate::geometry::{Geometry, GeometryQuantity, GeometryRow, GeometryValue};
use crate::register::{Register, RegisterTexts};
use crate::release::{Program, Quantity, ReleaseUnit, ReleaseValue, TestTemperature};
use crate::table::Vocabulary;
use crate::temperature::TemperaturePoint;
use crate::yields::{Parent, YieldRow};

const LN2: f64 = std::f64::consts::LN_2;
/// Avogadro constant, mol-1 (SI defining value), as in `b4_decay_source_pseudocode.py`.
pub const AVOGADRO: f64 = 6.022_140_76e23;

/// The result of one check.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    /// Stable id, `<script>.<family>.<name>`.
    pub id: String,
    /// True when the check passed.
    pub passed: bool,
    /// What the check asserts.
    pub description: String,
    /// Numbers or names that explain the outcome.
    pub detail: String,
}

impl fmt::Display for CheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] {}",
            if self.passed { "PASS" } else { "FAIL" },
            self.id,
            self.description
        )?;
        if !self.detail.is_empty() {
            write!(f, " ({})", self.detail)?;
        }
        Ok(())
    }
}

/// All results of a run.
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// The results in the order the checks ran.
    pub results: Vec<CheckResult>,
}

impl Report {
    fn add(
        &mut self,
        id: impl Into<String>,
        passed: bool,
        description: impl Into<String>,
        detail: impl Into<String>,
    ) {
        self.results.push(CheckResult {
            id: id.into(),
            passed,
            description: description.into(),
            detail: detail.into(),
        });
    }

    /// The result with a given id.
    pub fn get(&self, id: &str) -> Option<&CheckResult> {
        self.results.iter().find(|r| r.id == id)
    }

    /// Number of failed checks.
    pub fn failures(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }

    /// True when every check passed.
    pub fn all_passed(&self) -> bool {
        self.failures() == 0 && !self.results.is_empty()
    }

    /// The final line.
    pub fn summary(&self) -> String {
        if self.all_passed() {
            format!("SUMMARY: ALL PASS ({} checks)", self.results.len())
        } else {
            format!(
                "SUMMARY: {} FAIL of {} checks",
                self.failures(),
                self.results.len()
            )
        }
    }
}

/// Run every check. `species_csv` is the text of `data/research/species.csv` when the file exists
/// (rule 6 of `check_catalogue.py` skips itself when it does not).
pub fn run_all(texts: &RegisterTexts, species_csv: Option<&str>) -> Report {
    let mut rep = Report::default();
    schema_checks(&mut rep, texts);
    match Register::from_texts(texts) {
        Err(e) => {
            rep.add(
                "LOAD.register",
                false,
                "the six files are read by the typed readers",
                e.to_string(),
            );
            return rep;
        }
        Ok(reg) => {
            rep.add(
                "LOAD.register",
                true,
                "the six files are read by the typed readers",
                "",
            );
            load_checks(&mut rep, texts, &reg);
            geometry_checks(&mut rep, &reg.geometry);
            decay_checks(&mut rep, &reg.decay);
            fluence_and_reduced_checks(&mut rep, &reg.arrhenius);
            cross_set_checks(&mut rep, &reg.arrhenius);
            frg_checks(&mut rep, &reg.arrhenius);
            booth_checks(&mut rep, &reg.arrhenius);
            yield_rule_checks(&mut rep);
            temperature_checks(&mut rep, &reg);
            release_checks(&mut rep, &reg);
            sanity_checks(&mut rep, &reg.arrhenius);
            b6_checks(&mut rep, &reg);
            b4_decay_checks(&mut rep, &reg);
            b4_phase2_checks(&mut rep, &reg);
            b6_offline_checks(&mut rep, &reg);
            generate_data_checks(&mut rep, &reg.geometry);
            catalogue_checks(&mut rep, &reg.decay, species_csv);
        }
    }
    rep
}

// ---------------------------------------------------------------- helpers

fn rel(a: f64, b: f64) -> f64 {
    (a / b - 1.0).abs()
}

fn first_n(v: &[String], n: usize) -> String {
    let mut s: Vec<&str> = v.iter().take(n).map(String::as_str).collect();
    if v.len() > n {
        s.push("...");
    }
    s.join(", ")
}

/// `(D0, Q)` of a row where they exist.
fn d0_q(r: &ArrheniusRow) -> (Option<f64>, Option<f64>) {
    match &r.law {
        Law::Arrhenius { d0, q_kj_mol, .. } => (Some(d0.value), Some(q_kj_mol.value)),
        Law::Constant { d, q_kj_mol } => (Some(d.value), q_kj_mol.as_ref().map(|q| q.value)),
        Law::Instantaneous => (None, None),
    }
}

// ------------------------------------------------------- D5 family S (schema)

struct FileSpec<'a> {
    tag: &'static str,
    file: &'static str,
    text: &'a str,
    id_col: Option<&'static str>,
    source_col: &'static str,
}

fn schema_checks(rep: &mut Report, t: &RegisterTexts) {
    let specs = [
        FileSpec {
            tag: "geometry",
            file: crate::geometry::FILE,
            text: &t.geometry,
            id_col: Some("id"),
            source_col: "source",
        },
        FileSpec {
            tag: "arrhenius",
            file: crate::arrhenius::FILE,
            text: &t.arrhenius,
            id_col: Some("id"),
            source_col: "source",
        },
        FileSpec {
            tag: "temperature",
            file: crate::temperature::FILE,
            text: &t.temperature,
            id_col: Some("id"),
            source_col: "source",
        },
        FileSpec {
            tag: "release",
            file: crate::release::FILE,
            text: &t.release,
            id_col: Some("id"),
            source_col: "source",
        },
        FileSpec {
            tag: "decay",
            file: crate::decay::FILE,
            text: &t.decay,
            id_col: Some("nuclide"),
            source_col: "source",
        },
        FileSpec {
            tag: "yields",
            file: crate::yields::FILE,
            text: &t.yields,
            id_col: None,
            source_col: "source_JEFF",
        },
    ];
    for s in &specs {
        let recs = match csv::parse(s.file, s.text) {
            Ok(r) => r,
            Err(e) => {
                for n in ["header", "status", "ids", "rect"] {
                    rep.add(
                        format!("D5.S.{n}.{}", s.tag),
                        false,
                        format!("{}: file is readable", s.file),
                        e.to_string(),
                    );
                }
                continue;
            }
        };
        let header = &recs[0].cells;
        let col = |name: &str| header.iter().position(|h| h == name);
        let has = col(s.source_col).is_some() && col("status").is_some();
        rep.add(
            format!("D5.S.header.{}", s.tag),
            has,
            format!(
                "{}: source column '{}' and status column present",
                s.file, s.source_col
            ),
            "",
        );
        let rows = &recs[1..];
        let bad: Vec<String> = match col("status") {
            Some(i) => rows
                .iter()
                .filter(|r| {
                    r.cells
                        .get(i)
                        .is_none_or(|c| Status::from_cell(c).is_none())
                })
                .map(|r| r.cells.get(i).cloned().unwrap_or_default())
                .collect(),
            None => vec!["no status column".to_string()],
        };
        rep.add(
            format!("D5.S.status.{}", s.tag),
            bad.is_empty(),
            format!(
                "{}: {} rows, status values in {:?} (bad: {})",
                s.file,
                rows.len(),
                Status::ALL,
                first_n(&bad, 3)
            ),
            "",
        );
        let (ok, what) = match s.id_col {
            Some(c) => match col(c) {
                Some(i) => {
                    let ids: Vec<&str> = rows
                        .iter()
                        .filter_map(|r| r.cells.get(i).map(String::as_str))
                        .collect();
                    let uniq: HashSet<&str> = ids.iter().copied().collect();
                    (ids.len() == uniq.len(), "ids unique")
                }
                None => (false, "id column present"),
            },
            None => {
                // fission_yields.csv has no id column: the key (parent, product, level index) must be unique.
                let k = ["parent", "product", "LiveChart_level_idx"].map(col);
                match k {
                    [Some(a), Some(b), Some(c)] => {
                        let keys: HashSet<(&str, &str, &str)> = rows
                            .iter()
                            .map(|r| {
                                (
                                    r.cells[a].as_str(),
                                    r.cells[b].as_str(),
                                    r.cells[c].as_str(),
                                )
                            })
                            .collect();
                        (
                            keys.len() == rows.len(),
                            "key (parent, product, level index) unique (no id column)",
                        )
                    }
                    _ => (false, "key columns present"),
                }
            }
        };
        rep.add(
            format!("D5.S.ids.{}", s.tag),
            ok,
            format!("{}: {what}", s.file),
            "",
        );
        let bad_ev: Vec<String> = match col("evidence_class") {
            Some(i) => rows
                .iter()
                .filter(|r| {
                    r.cells
                        .get(i)
                        .is_none_or(|c| EvidenceClass::from_cell(c).is_none())
                })
                .map(|r| r.cells.get(i).cloned().unwrap_or_default())
                .collect(),
            None => vec!["no evidence_class column".to_string()],
        };
        rep.add(
            format!("D5.V.evidence.{}", s.tag),
            bad_ev.is_empty(),
            format!(
                "{}: evidence_class present and in the vocabulary of {} classes (bad: {})",
                s.file,
                EvidenceClass::ALL.len(),
                first_n(&bad_ev, 3)
            ),
            "",
        );
        let rect = rows.iter().all(|r| r.cells.len() == header.len());
        rep.add(
            format!("D5.S.rect.{}", s.tag),
            rect,
            format!("{}: rectangular, no overflow cells", s.file),
            "",
        );
    }
}

fn load_checks(rep: &mut Report, t: &RegisterTexts, reg: &Register) {
    let raw = |text: &str, file: &str| {
        csv::parse(file, text)
            .map(|r| r.len() - 1)
            .unwrap_or(usize::MAX)
    };
    let items = [
        (
            "geometry",
            crate::geometry::FILE,
            &t.geometry,
            reg.geometry.rows.len(),
        ),
        (
            "arrhenius",
            crate::arrhenius::FILE,
            &t.arrhenius,
            reg.arrhenius.rows.len(),
        ),
        ("decay", crate::decay::FILE, &t.decay, reg.decay.rows.len()),
        (
            "yields",
            crate::yields::FILE,
            &t.yields,
            reg.yields.rows.len(),
        ),
        (
            "temperature",
            crate::temperature::FILE,
            &t.temperature,
            reg.temperature.rows.len(),
        ),
        (
            "release",
            crate::release::FILE,
            &t.release,
            reg.release.rows.len(),
        ),
    ];
    for (tag, file, text, typed) in items {
        let n = raw(text, file);
        rep.add(
            format!("LOAD.rows.{tag}"),
            n == typed,
            format!("{file}: the typed reader holds every data row of the file (none dropped)"),
            format!("{typed} typed rows, {n} data rows in the file"),
        );
    }
}

// ------------------------------------------------------------- D5 family G

struct DesignLayers<'a> {
    by_index: BTreeMap<u32, Vec<&'a GeometryRow>>,
}

fn design_layers(g: &Geometry) -> BTreeMap<&str, DesignLayers<'_>> {
    let mut designs: BTreeMap<&str, DesignLayers<'_>> = BTreeMap::new();
    for r in &g.rows {
        if r.outer_radius_um_calc.is_some() {
            designs
                .entry(r.design_id.as_str())
                .or_insert_with(|| DesignLayers {
                    by_index: BTreeMap::new(),
                })
                .by_index
                .entry(u32::from(r.layer_index))
                .or_default()
                .push(r);
        }
    }
    designs
}

fn point_decimal(r: &GeometryRow) -> Option<Decimal> {
    match &r.value {
        GeometryValue::Point(p) | GeometryValue::PointWithRange { value: p, .. } => {
            Decimal::parse(&p.text).ok()
        }
        _ => None,
    }
}

fn geometry_checks(rep: &mut Report, g: &Geometry) {
    let designs = design_layers(g);
    // identity R_i - R_(i-1) = t_i, exact decimal arithmetic instead of SymPy
    let mut identity_ok = true;
    let mut incomplete = Vec::new();
    let mut n_checked = 0usize;
    let mut worst = 0.0f64;
    let mut radii_ok = true;
    let mut radii_msgs = Vec::new();
    for (did, d) in &designs {
        let complete = d.by_index.keys().copied().collect::<Vec<_>>() == vec![1, 2, 3, 4, 5]
            && d.by_index.values().all(|v| v.len() == 1);
        if !complete {
            incomplete.push(format!(
                "{did}: layer set {:?}",
                d.by_index.keys().collect::<Vec<_>>()
            ));
            continue;
        }
        let vals: Option<Vec<Decimal>> =
            (1..=5).map(|i| point_decimal(d.by_index[&i][0])).collect();
        let Some(vals) = vals else {
            incomplete.push(format!("{did}: a layer value is not a plain decimal"));
            continue;
        };
        let mut radii = vec![vals[0].half()];
        for t in &vals[1..] {
            let prev = radii.last().copied().unwrap_or(Decimal::from_int(0));
            radii.push(prev + *t);
        }
        for i in 1..5 {
            if radii[i] - radii[i - 1] != vals[i] {
                identity_ok = false;
            }
        }
        for (i, rad) in radii.iter().enumerate() {
            let row = d.by_index[&(i as u32 + 1)][0];
            let tab = row
                .outer_radius_um_calc
                .as_ref()
                .map(|p| p.value)
                .unwrap_or(f64::NAN);
            n_checked += 1;
            let diff = (rad.to_f64() - tab).abs();
            worst = worst.max(diff);
            if diff.is_nan() || diff > 1e-9 {
                radii_ok = false;
                radii_msgs.push(format!(
                    "{did} layer {}: table {tab} vs calc {}",
                    i + 1,
                    rad.to_f64()
                ));
            }
        }
    }
    rep.add(
        "D5.G.identity",
        identity_ok && incomplete.is_empty(),
        "R_i - R_(i-1) = t_i for every design (exact decimal arithmetic in place of SymPy)",
        "",
    );
    rep.add(
        "D5.G.layers",
        incomplete.is_empty(),
        "every design with outer radii has exactly the five layers 1 to 5, one row each",
        first_n(&incomplete, 3),
    );
    rep.add(
        "D5.G.radii",
        radii_ok && n_checked == 5 * designs.len() && incomplete.is_empty(),
        format!(
            "{} designs, {n_checked} outer radii recomputed, all |diff| < 1e-9 um",
            designs.len()
        ),
        format!("worst diff {worst:.2e} um; {}", first_n(&radii_msgs, 3)),
    );
    let radii_of = |did: &str| -> Option<Vec<f64>> {
        let d = designs.get(did)?;
        (1..=5u32)
            .map(|i| {
                d.by_index
                    .get(&i)
                    .and_then(|v| v.first())
                    .and_then(|r| r.outer_radius_um_calc.as_ref())
                    .map(|p| p.value)
            })
            .collect()
    };
    let crp = radii_of("CRP6-SENS");
    rep.add(
        "D5.G.crp6",
        crp.as_deref() == Some(&[250.0, 350.0, 390.0, 425.0, 465.0][..]),
        "CRP-6 sensitivity radii equal the audited 250/350/390/425/465 um",
        format!("{crp:?}"),
    );
    let agr = radii_of("AGR1-B-M16");
    let d2 = agr.as_ref().map(|a| 2.0 * a[4]);
    rep.add(
        "D5.G.agr1",
        d2.is_some_and(|d| (d - 788.4).abs() < 1e-9),
        "AGR-1 Baseline particle diameter 2R = 788.4 um (nominal 780 um, IAEA 2012 p. 395)",
        format!("2R = {d2:?}"),
    );
}

// ------------------------------------------------------------- D5 family L

fn radioactive_rows(
    d: &Decay,
) -> impl Iterator<Item = (&crate::decay::DecayRow, &crate::decay::Radioactive)> {
    d.rows.iter().filter_map(|r| match &r.stability {
        Stability::Radioactive(x) => Some((r, x.as_ref())),
        Stability::Stable { .. } => None,
    })
}

fn stable_lambda_zero(d: &Decay) -> (usize, Vec<String>) {
    let mut n = 0;
    let mut bad = Vec::new();
    for r in &d.rows {
        if let Stability::Stable { lambda_per_s } = &r.stability {
            n += 1;
            if lambda_per_s.value != 0.0 {
                bad.push(r.nuclide.name());
            }
        }
    }
    (n, bad)
}

fn decay_checks(rep: &mut Report, d: &Decay) {
    // u(lambda) = |d lambda / dT| u(T) = lambda u(T)/T: numerical derivative in place of SymPy
    let mut worst_formula = 0.0f64;
    for t in [1.0, 24.56, 1.5e5, 9.49e8] {
        let h = t * 1e-6;
        let dl = ((LN2 / (t + h)) - (LN2 / (t - h))) / (2.0 * h);
        worst_formula = worst_formula.max(rel(dl.abs(), LN2 / t / t));
    }
    rep.add(
        "D5.L.uncertainty-formula",
        worst_formula < 1e-8,
        "u(lambda) = |d lambda/dT| u(T) = lambda u(T)/T (central difference of ln2/T against the closed form)",
        format!("worst relative difference {worst_formula:.1e}"),
    );
    let (n_stable, bad_stable) = stable_lambda_zero(d);
    rep.add(
        "D5.L.stable",
        bad_stable.is_empty() && n_stable > 0,
        "stable rows have lambda = 0",
        format!("{n_stable} stable rows"),
    );
    let mut wl = 0.0f64;
    let mut wu = 0.0f64;
    let mut n = 0;
    for (_, x) in radioactive_rows(d) {
        let ts = x.half_life_s.value;
        let lam_calc = LN2 / ts;
        let ul_calc = lam_calc * x.unc_half_life_s.value / ts;
        wl = wl.max(rel(x.lambda_per_s.value, lam_calc));
        wu = wu.max(rel(x.unc_lambda_per_s.value, ul_calc));
        n += 1;
    }
    rep.add(
        "D5.L.lambda",
        wl < 1e-6,
        format!("{n} radioactive rows: max |lambda_tab/lambda_calc - 1| < 1e-6"),
        format!("{wl:.2e}"),
    );
    rep.add(
        "D5.L.unc-lambda",
        wu < 1e-3,
        format!(
            "{n} radioactive rows: max relative difference of u(lambda) < 1e-3 (4-digit rounding)"
        ),
        format!("{wu:.2e}"),
    );
    let yr = 365.25 * 86400.0;
    for (nuc, printed) in [
        ("Cs-137", 30.07 * yr),
        ("Sr-90", 28.78 * yr),
        ("Ag-110m", 249.8 * 86400.0),
    ] {
        let ts = d.get(nuc).and_then(|r| match &r.stability {
            Stability::Radioactive(x) => Some(x.half_life_s.value),
            Stability::Stable { .. } => None,
        });
        let dd = ts.map(|t| printed / t - 1.0);
        rep.add(
            format!("D5.L.iaea.{nuc}"),
            dd.is_some_and(|x| x.abs() < 5e-3),
            format!("{nuc}: IAEA (2012) Table 8.1 half-life differs from ENSDF by less than 0.5 % (cross-check)"),
            format!("{dd:?}"),
        );
    }
}

// ------------------------------------------------- D5 families F and K

fn fluence_and_reduced_checks(rep: &mut Report, a: &Arrhenius) {
    // F: the divisor of exp(Gamma/divisor) is the one the register documents on A3-01
    let divisor = a.get("A3-01").and_then(|r| match r.fluence {
        FluenceFactor::Documented { divisor } => Some(divisor),
        _ => None,
    });
    let want = [
        (0.0, 1.000),
        (1.0, 1.221),
        (2.0, 1.492),
        (3.0, 1.822),
        (4.5, 2.460),
    ];
    let ok = divisor.is_some_and(|dv| want.iter().all(|(g, v)| ((g / dv).exp() - v).abs() < 6e-4));
    rep.add(
        "D5.F.values",
        ok,
        "exp(Gamma/5) at Gamma = 0, 1, 2, 3, 4.5 equals 1.000, 1.221, 1.492, 1.822, 2.460 (divisor taken from the documented factor of A3-01)",
        format!("divisor {divisor:?}"),
    );
    let lim = divisor
        .is_some_and(|dv| (0.0f64 / dv).exp() == 1.0 && ((1e-10f64 / dv).exp() - 1.0).abs() < 1e-9);
    rep.add(
        "D5.F.limit",
        lim,
        "fluence factor -> 1 as Gamma -> 0 (numerical limit in place of SymPy)",
        "",
    );
    let r = 8.3145;
    let tk = 1600.0 + 273.15;
    let d1 = a.get("E109-01").and_then(|x| x.diffusivity(tk, r, 2.0));
    let d2 = a.get("E109-02").and_then(|x| x.diffusivity(tk, r, 2.0));
    let (ok, detail) = match (d1, d2) {
        (Some(d1), Some(d2)) => {
            let s = d1.m2_per_s + d2.m2_per_s;
            (
                rel(s, 1.011e-16) < 1e-3,
                format!(
                    "{:.4e} + {:.4e} = {:.4e} m2/s vs printed 1.011e-16",
                    d1.m2_per_s, d2.m2_per_s, s
                ),
            )
        }
        _ => (false, "E109-01 or E109-02 is missing".to_string()),
    };
    rep.add(
        "D5.F.eq109",
        ok,
        "Eq. (10.9) at 1600 C, Gamma = 2 reproduces the printed 1.011e-16 m2/s within 1e-3",
        detail,
    );
    // K: D0 = D'0 r_k^2
    let rk2 = crate::arrhenius::REDUCED_COEFFICIENT_KERNEL_RADIUS_M.powi(2);
    for id in ["T71-01", "T71-02", "T71-03", "T71-04", "T71-05"] {
        let row = a.get(id);
        let (ok, detail) = match row.and_then(|r| r.reduced.as_ref().map(|x| (x, d0_q(r).0))) {
            Some((red, Some(d0))) => {
                let calc = red.d_prime_per_s.value * rk2;
                (
                    rel(calc, d0) < 1e-12,
                    format!(
                        "D'0 = {} s^-1 x (250 um)^2 = {calc:.5e} m2/s, table {d0:.5e}",
                        red.d_prime_per_s.text
                    ),
                )
            }
            _ => (false, "row or reduced coefficient missing".to_string()),
        };
        rep.add(
            format!("D5.K.{id}"),
            ok,
            format!("{id}: D0 = D'0 r_k^2 reproduces the table to 1e-12"),
            detail,
        );
    }
}

// ------------------------------------------------------------- D5 family X

/// `equals A1-01` or `equals C16-05` in a note: the cited id, first match only (as the Python regex).
fn equals_claim(note: &str) -> Option<String> {
    let mut rest = note;
    while let Some(p) = rest.find("equals ") {
        let tail = &rest[p + 7..];
        let b = tail.as_bytes();
        let a_form = b.len() >= 5
            && b[0] == b'A'
            && b[1].is_ascii_digit()
            && b[2] == b'-'
            && b[3].is_ascii_digit()
            && b[4].is_ascii_digit();
        let c_form = b.len() >= 6
            && tail.starts_with("C16-")
            && b[4].is_ascii_digit()
            && b[5].is_ascii_digit();
        if a_form {
            return Some(tail[..5].to_string());
        }
        if c_form {
            return Some(tail[..6].to_string());
        }
        rest = tail;
    }
    None
}

fn cross_set_checks(rep: &mut Report, a: &Arrhenius) {
    let mut n = 0;
    let mut bad = Vec::new();
    for r in &a.rows {
        let Some(cited) = equals_claim(&r.notes) else {
            continue;
        };
        n += 1;
        let same = match a.get(&cited) {
            Some(c) => {
                let (d, q) = d0_q(r);
                let (cd, cq) = d0_q(c);
                d.is_some() && d == cd && q.is_some() && q == cq
            }
            None => false,
        };
        if !same {
            bad.push(format!("{} != {cited}", r.id));
        }
    }
    rep.add(
        "D5.X.equals",
        n > 0 && bad.is_empty(),
        format!("{n} 'equals' claims (Collin 2016 Table 2, Hales 2021 Table 1) match the cited TECDOC-978 rows"),
        first_n(&bad, 3),
    );
    for (hid, first, second) in [
        ("H21-04", "A1-01", "A1-02"),
        ("H21-06", "A3-01", "A3-02"),
        ("H21-09", "A3-03", "A3-04"),
    ] {
        let text_q = |id: &str| match a.get(id).map(|r| &r.law) {
            Some(Law::Arrhenius { q_kj_mol, .. }) => Some(q_kj_mol.text.clone()),
            _ => None,
        };
        let ok = a
            .get(hid)
            .zip(a.get(first))
            .is_some_and(|(h, f)| d0_q(h).0 == d0_q(f).0 && d0_q(h).0.is_some())
            && text_q(hid).is_some()
            && text_q(hid) == text_q(first);
        rep.add(
            format!("D5.X.hales.{hid}"),
            ok,
            format!(
                "{hid}: Hales single term = first branch {first}; second branch {second} omitted"
            ),
            "",
        );
    }
}

// ------------------------------------------------------------- D5 family A

fn frg_checks(rep: &mut Report, a: &Arrhenius) {
    let audit: [(&str, &[&str], [f64; 3]); 9] = [
        (
            "Cs UO2",
            &["A1-01", "A1-02"],
            [1.498e-16, 1.250e-13, 6.972e-13],
        ),
        ("Sr UO2", &["A1-03"], [2.093e-23, 5.421e-17, 1.114e-15]),
        ("Ag UO2", &["A1-04"], [1.139e-15, 1.678e-13, 4.664e-13]),
        ("Cs PyC", &["A2-01"], [4.911e-17, 4.061e-14, 1.606e-13]),
        ("Sr PyC", &["A2-02"], [1.902e-14, 7.382e-12, 2.501e-11]),
        ("Ag PyC", &["A2-03"], [2.547e-15, 2.691e-13, 6.984e-13]),
        (
            "Cs SiC",
            &["A3-01", "A3-02"],
            [4.092e-19, 9.223e-17, 1.832e-15],
        ),
        (
            "Sr SiC",
            &["A3-03", "A3-04"],
            [4.661e-18, 2.462e-15, 2.936e-14],
        ),
        ("Ag SiC", &["A3-05"], [5.436e-18, 3.638e-15, 1.378e-14]),
    ];
    let mut worst = 0.0f64;
    let mut missing = false;
    for (_, ids, refs) in &audit {
        for (tc, v) in [1000.0, 1600.0, 1800.0].iter().zip(refs) {
            match a.sum_of_terms(ids, *tc, 8.3143, 0.0) {
                Some(d) => worst = worst.max(rel(d.m2_per_s, *v)),
                None => missing = true,
            }
        }
    }
    rep.add(
        "D5.A.frg-values",
        !missing && worst < 5e-4,
        "27 FRG D(T) values reproduce the audit table (R = 8.3143, Gamma = 0), max relative difference < 5e-4",
        format!("{worst:.1e}"),
    );
    let sens: Vec<f64> = audit
        .iter()
        .filter_map(|(_, ids, _)| {
            let lo = a.sum_of_terms(ids, 1600.0, 8.3142, 0.0)?;
            let hi = a.sum_of_terms(ids, 1600.0, 8.3143, 0.0)?;
            Some(lo.m2_per_s / hi.m2_per_s - 1.0)
        })
        .collect();
    let (mn, mx) = (
        sens.iter().copied().fold(f64::INFINITY, f64::min),
        sens.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    );
    rep.add(
        "D5.A.r-sensitivity",
        sens.len() == audit.len() && mn > -3.8e-4 && mx < -1.1e-4,
        "R = 8.3142 instead of 8.3143 changes D(1600 C) by between -3.8e-4 and -1.1e-4",
        format!("{mn:.2e} to {mx:.2e}"),
    );
}

// ------------------------------------------------------------- D5 family B

fn booth(dt: f64, nmax: u32) -> f64 {
    1.0 - 6.0 / std::f64::consts::PI.powi(2)
        * (1..=nmax)
            .map(|k| {
                let k = f64::from(k);
                (-(k * std::f64::consts::PI).powi(2) * dt).exp() / (k * k)
            })
            .sum::<f64>()
}

fn booth_checks(rep: &mut Report, a: &Arrhenius) {
    // F(0) = 1 - (6/pi^2) zeta(2) = 0: partial sum plus the integral tail, in place of SymPy's summation
    let n = 200_000u32;
    let nf = f64::from(n);
    let s: f64 = (1..=n)
        .map(|k| 1.0 / (f64::from(k) * f64::from(k)))
        .sum::<f64>()
        + (1.0 / nf - 1.0 / (2.0 * nf * nf) + 1.0 / (6.0 * nf.powi(3)));
    let zero = 1.0 - 6.0 / std::f64::consts::PI.powi(2) * s;
    rep.add(
        "D5.B.booth-zero",
        zero.abs() < 1e-12,
        "Booth F(D't = 0) = 1 - (6/pi^2) zeta(2) = 0 (partial sum with tail correction in place of SymPy)",
        format!("{zero:.2e}"),
    );
    let t = 200.0 * 3600.0;
    let rk = 250e-6;
    for (tc, printed, tol, name) in [
        (1200.0, 0.4673, 3e-4, "case-1200"),
        (1600.0, 0.999_999_59, 5e-9, "case-1600"),
    ] {
        let (ok, detail) = match a.sum_of_terms(&["A1-01", "A1-02"], tc, 8.3145, 0.0) {
            Some(d) => {
                let dt = d.m2_per_s * t / (rk * rk);
                let f = booth(dt, 20_000);
                (
                    (f - printed).abs() < tol,
                    format!("D't = {dt:.5}, F = {f:.8} vs printed {printed}"),
                )
            }
            None => (false, "A1-01 or A1-02 missing".to_string()),
        };
        rep.add(
            format!("D5.B.{name}"),
            ok,
            format!(
                "CRP-6 case 1 at {tc} C from the two-branch Cs kernel D reproduces the printed F"
            ),
            detail,
        );
    }
    let (ok, detail) = match a.sum_of_terms(&["T71-01"], 1200.0, 8.3145, 0.0) {
        Some(d) => {
            let f = booth(d.m2_per_s * t / (rk * rk), 20_000);
            ((f - 0.4614).abs() < 5e-4, format!("F = {f:.4}, not 0.4673"))
        }
        None => (false, "T71-01 missing".to_string()),
    };
    rep.add(
        "D5.B.single-branch",
        ok,
        "the single branch of IAEA (2012) Table 7.1 gives F within 5e-4 of 0.4614 at 1200 C",
        detail,
    );
}

// ------------------------------------------------------------- D5 family Y

fn yield_rule_checks(rep: &mut Report) {
    // parent N_p' = yI_p F - lambda_p N_p, N_p(0) = 0; daughter source rule (R.6b). Closed form in place of SymPy dsolve.
    let cases = [
        (5.0e19, 0.03, 0.002, 0.8, 1.0e-6),
        (1.0, 1.0e-3, 4.0e-2, 0.25, 3.5e-3),
        (2.5e10, 7.2e-4, 1.0e-9, 1.0, 7.3e-10),
    ];
    let np = |f: f64, yip: f64, lp: f64, t: f64| yip * f / lp * (1.0 - (-lp * t).exp());
    let mut r_steady = 0.0f64;
    let mut r_over = 0.0f64;
    let mut r_init = 0.0f64;
    let mut r_ode = 0.0f64;
    for (f, yip, yid, b, lp) in cases {
        let n_inf = np(f, yip, lp, 1e4 / lp);
        let ycd = yid + b * yip;
        let right = yid * f + b * lp * n_inf;
        let wrong = ycd * f + lp * n_inf;
        r_steady = r_steady.max(((right - ycd * f) / (ycd * f)).abs());
        r_over = r_over.max(((wrong - ycd * f - yip * f) / (yip * f)).abs());
        r_init = r_init.max(np(f, yip, lp, 0.0).abs());
        let t = 0.7 / lp;
        let h = t * 1e-5;
        let dn = (np(f, yip, lp, t + h) - np(f, yip, lp, t - h)) / (2.0 * h);
        let res = yip * f - lp * np(f, yip, lp, t);
        r_ode = r_ode.max(((dn - res) / res).abs());
    }
    rep.add(
        "D5.Y.steady-state",
        r_steady < 1e-12,
        "steady state: y_I(d) Fdot + b lambda_p N_p = y_C(d) Fdot (residual 0, three parameter sets)",
        format!("{r_steady:.1e}"),
    );
    rep.add(
        "D5.Y.overcount",
        r_over < 1e-9,
        "using y_C(d) Fdot + lambda_p N_p over-counts by exactly y_I(p) Fdot",
        format!("{r_over:.1e}"),
    );
    rep.add(
        "D5.Y.initial",
        r_init == 0.0,
        "parent inventory starts at zero (initial condition satisfied)",
        "",
    );
    rep.add(
        "D5.Y.ode-solution",
        r_ode < 1e-8,
        "the closed-form parent inventory satisfies N_p' = y_I(p) Fdot - lambda_p N_p (central difference, in place of SymPy dsolve)",
        format!("{r_ode:.1e}"),
    );
}

// ------------------------------------------------------------- D5 family T

fn temperature_checks(rep: &mut Report, reg: &Register) {
    let mut n_rate = 0;
    let mut bad = Vec::new();
    for r in &reg.temperature.rows {
        let t0 = r.t_start.as_ref().and_then(TemperaturePoint::celsius);
        let t1 = r.t_end.as_ref().and_then(TemperaturePoint::celsius);
        if let (Some(rate), Some(ramp), Some(t0), Some(t1)) = (&r.rate_c_per_h, &r.ramp_h, t0, t1) {
            n_rate += 1;
            let calc = (t1 - t0).abs() / rate.value.abs();
            if (calc - ramp.value).abs() > 6e-5 {
                bad.push(format!(
                    "{}: ramp {} h vs |dT|/rate {calc:.5} h",
                    r.id, ramp.text
                ));
            }
        }
    }
    rep.add(
        "D5.T.ramps",
        n_rate == 12 && bad.is_empty(),
        format!(
            "{n_rate} ramp rows (expected 12): ramp_h = |T_end - T_start| / |rate| within 6e-5 h"
        ),
        first_n(&bad, 3),
    );
    let mut n_cum = 0;
    let mut bad = Vec::new();
    let hist = reg.temperature.histories();
    for (_, rows) in &hist {
        let mut run = 0.0;
        for r in rows {
            let Some(c) = &r.cumulative_end_h_calc else {
                continue;
            };
            run += r.ramp_h.as_ref().map_or(0.0, |p| p.value)
                + r.hold_h.as_ref().map_or(0.0, |p| p.value);
            n_cum += 1;
            if (run - c.value).abs() > 1e-3 {
                bad.push(format!(
                    "{}: cumulative {} vs running sum {run:.4}",
                    r.id, c.text
                ));
            }
        }
    }
    rep.add(
        "D5.T.cumulative",
        bad.is_empty(),
        format!("{n_cum} cumulative-time entries equal the running sum of ramp + hold"),
        first_n(&bad, 3),
    );
    let printed = [
        ("AGR1-ST-C16-1600", 334.0),
        ("AGR1-ST-C16-1700", 336.0),
        ("AGR1-ST-C16-1800", 338.0),
        ("CRP6-6a", 333.0),
        ("CRP6-7a", 277.0),
        ("CRP6-7b", 229.0),
        ("CRP6-8a", 541.0),
        ("CRP6-8b", 187.0),
        ("CRP6-9", 699.0),
        ("CRP6-10", 836.0),
        ("CRP6-11", 824.0),
    ];
    for (hid, tot) in printed {
        let last = hist.iter().find(|(h, _)| *h == hid).and_then(|(_, rows)| {
            rows.last()
                .and_then(|r| r.cumulative_end_h_calc.as_ref().map(|p| p.value))
        });
        rep.add(
            format!("D5.T.total.{hid}"),
            last.is_some_and(|l| (l - tot).abs() < 0.5),
            format!("{hid}: schedule sums to the printed total {tot} h within 0.5 h"),
            format!("{last:?}"),
        );
    }
}

// ------------------------------------------------------------- D5 family E

fn release_checks(rep: &mut Report, reg: &Register) {
    let rows = &reg.release.rows;
    let single = |v: &ReleaseValue| match v {
        ReleaseValue::Point(p)
        | ReleaseValue::Digitised(p)
        | ReleaseValue::UpperBound(p)
        | ReleaseValue::LowerBound(p) => Some(p.value),
        _ => None,
    };
    let n1 = rows
        .iter()
        .filter(|r| r.unit == ReleaseUnit::Fraction)
        .count();
    let bad: Vec<String> = rows
        .iter()
        .filter(|r| r.unit == ReleaseUnit::Fraction)
        .filter(|r| single(&r.value).is_some_and(|v| !(0.0..=1.0).contains(&v)))
        .map(|r| r.id.clone())
        .collect();
    rep.add(
        "D5.E.range01",
        bad.is_empty(),
        format!("all {n1} dimensionless values lie in [0, 1]"),
        first_n(&bad, 5),
    );
    let bad: Vec<String> = rows
        .iter()
        .filter(|r| matches!(&r.value, ReleaseValue::Range { min, max } if min.value > max.value))
        .map(|r| r.id.clone())
        .collect();
    rep.add(
        "D5.E.minmax",
        bad.is_empty(),
        "value_min <= value_max in every range row",
        first_n(&bad, 5),
    );
    for (id, frac) in [("E-114", 9.1e-5), ("E-115", 4.8e-4), ("E-116", 1.4e-3)] {
        let calc = reg.release.get(id).and_then(|r| match &r.value {
            ReleaseValue::FailedOverTested { failed, tested } => Some(failed.value / tested.value),
            _ => None,
        });
        rep.add(
            format!("D5.E.fraction.{id}"),
            calc.is_some_and(|c| (c / frac - 1.0).abs() < 0.02),
            format!("{id}: failed-particle fraction within 2 % of the audited {frac:.1e}"),
            format!("{calc:?}"),
        );
    }
    let s1800: f64 = rows
        .iter()
        .filter(|r| r.quantity == Quantity::ParticlesWithFailedIpycAndSic)
        .filter(|r| matches!(&r.temperature, TestTemperature::Celsius(p) if p.text == "1800"))
        .filter_map(|r| single(&r.value))
        .sum();
    rep.add(
        "D5.E.failures-1800",
        s1800 == 23.0,
        "sum of Collin (2016) Table 3 failures at 1800 C = 23 (Morris 2016 p. 34)",
        format!("{s1800}"),
    );
    rep.add(
        "D5.E.one-particle",
        (1.0f64 / 4145.0 - 2.41e-4).abs() < 5e-7,
        "one particle = 1/4145 of a compact = 2.4e-4 (printed 2.4e-4)",
        format!("{:.3e}", 1.0 / 4145.0),
    );
    let n = rows.iter().filter(|r| r.program == Program::Jaeri).count();
    rep.add(
        "D5.E.jaeri-count",
        n == 33,
        format!("TECDOC-978 Table 4-9: {n} non-blank cells transcribed (expected 33)"),
        "",
    );
}

fn sanity_checks(rep: &mut Report, a: &Arrhenius) {
    let mut bad = Vec::new();
    for r in &a.rows {
        let (d0, q) = d0_q(r);
        let tmin = r.t_min_c.as_ref().map(|p| p.value);
        let tmax = r.t_max_c.as_ref().map(|p| p.value);
        if d0.is_some_and(|d| d <= 0.0)
            || q.is_some_and(|q| q < 0.0)
            || matches!((tmin, tmax), (Some(a), Some(b)) if a > b)
        {
            bad.push(r.id.clone());
        }
    }
    rep.add(
        "D5.A.sanity",
        bad.is_empty(),
        "all D0 > 0, Q >= 0, T_min <= T_max in arrhenius.csv",
        first_n(&bad, 5),
    );
}

// ------------------------------------------------------------- D5 family V

/// The independent B6 re-reading of every Ag, Cs and Sr row (D0 m2/s, Q kJ/mol, T_min, T_max), copied from
/// `D5_parameter_register.py` lines 302-330.
type B6Row = (&'static str, f64, Option<f64>, Option<f64>, Option<f64>);
#[rustfmt::skip]
pub const B6: &[B6Row] = &[
    ("A1-01", 5.6e-8, Some(209.0), None, None), ("A1-02", 5.2e-4, Some(362.0), None, None),
    ("A1-03", 2.2e-3, Some(488.0), None, None), ("A1-04", 6.7e-9, Some(165.0), None, None),
    ("A1-08", 5.6e-8, Some(209.0), None, None), ("A1-09", 5.2e-4, Some(362.0), None, None),
    ("A1-10", 2.2e-3, Some(488.0), None, None), ("A1-11", 6.7e-9, Some(165.0), None, None),
    ("A1-12", 4.7e-10, Some(177.0), Some(1400.0), Some(1720.0)),
    ("A2-01", 6.3e-8, Some(222.0), None, None), ("A2-02", 2.3e-6, Some(197.0), None, None),
    ("A2-03", 5.3e-9, Some(154.0), None, None), ("A2-06", 5.0e-5, Some(318.0), Some(1200.0), Some(1800.0)),
    ("A2-07", 2.3e-6, Some(197.0), Some(1000.0), Some(1650.0)), ("A2-08", 5.3e-9, Some(154.0), None, None),
    ("A2-11", 6.0e-9, Some(198.0), Some(1250.0), Some(1385.0)), ("A2-12", 1.2e-3, Some(412.0), Some(1600.0), Some(2300.0)),
    ("A2-13", 5.3e-4, Some(193.0), Some(1400.0), Some(1700.0)),
    ("A3-01", 5.5e-14, Some(125.0), None, None), ("A3-02", 1.6e-2, Some(514.0), None, None),
    ("A3-03", 1.2e-9, Some(205.0), None, None), ("A3-04", 1.8e6, Some(791.0), None, None),
    ("A3-05", 3.6e-9, Some(215.0), None, None), ("A3-06", 6.7e-14, Some(106.0), None, None),
    ("A3-07", 1.1e-4, Some(437.0), None, None), ("A3-08", 6.7e-14, Some(106.0), None, None),
    ("A3-09", 2.4e-2, Some(482.0), None, None), ("A3-10", 1.2e-9, Some(205.0), None, None),
    ("A3-11", 1.8e6, Some(791.0), None, None), ("A3-12", 3.6e-9, Some(215.0), None, None),
    ("A3-13", 6.8e-12, Some(177.0), Some(1200.0), Some(1400.0)), ("A3-14", 2.5e-2, Some(503.0), Some(1600.0), Some(1900.0)),
    ("A3-15", 1.2e-9, Some(205.0), Some(1600.0), Some(1850.0)), ("A3-16", 6.8e-11, Some(177.0), Some(1200.0), Some(1400.0)),
    ("A3-20", 2.3e-17, None, Some(1000.0), Some(1000.0)), ("A3-21", 3.5e-10, Some(213.0), Some(1200.0), Some(2300.0)),
    ("A4-01", 3.6e-4, Some(189.0), None, None), ("A4-02", 1.0e-2, Some(303.0), None, Some(2000.0)),
    ("A4-03", 6.8e1, Some(262.0), Some(800.0), Some(1300.0)), ("A4-04", 1.3, Some(246.0), Some(800.0), Some(1300.0)),
    ("A4-05", 1.6, Some(258.0), Some(800.0), Some(1300.0)), ("A4-10", 2.8e-4, Some(210.0), Some(1175.0), Some(1375.0)),
    ("A4-11", 3.5e-7, Some(116.0), Some(750.0), Some(1550.0)), ("A4-12", 5.6e-2, Some(312.0), Some(1200.0), Some(1600.0)),
    ("A4-13", 8.7e7, Some(414.0), Some(800.0), Some(1000.0)),
    ("A5-01", 1.7e-6, Some(149.0), Some(550.0), Some(1450.0)), ("A5-02", 1.7e-2, Some(268.0), Some(800.0), Some(2200.0)),
    ("A5-03", 1.6, Some(258.0), Some(490.0), Some(800.0)), ("A5-05", 1.7e-6, Some(149.0), Some(550.0), Some(1450.0)),
    ("A5-06", 1.7e-2, Some(268.0), Some(800.0), Some(2200.0)), ("A5-07", 9.0e-6, Some(157.0), Some(750.0), Some(1030.0)),
    ("A5-08", 6.3e-3, Some(264.0), Some(750.0), Some(1030.0)), ("A5-09", 1.2e-4, Some(112.0), Some(600.0), Some(1000.0)),
    ("A5-10", 1.7e-4, Some(95.0), Some(600.0), Some(1000.0)), ("A5-11", 5.8e-2, Some(151.0), Some(450.0), Some(750.0)),
    ("A5-12", 8.3e-1, Some(324.0), Some(850.0), Some(1400.0)), ("A5-13", 1.6e2, Some(364.0), Some(850.0), Some(1100.0)),
    ("C16-01", 6.7e-9, Some(165.0), None, None), ("C16-02", 5.3e-9, Some(154.0), None, None),
    ("C16-03", 3.6e-9, Some(215.0), None, None), ("C16-04", 1.6, Some(258.0), None, None),
    ("C16-05", 5.6e-8, Some(209.0), None, None), ("C16-06", 5.2e-4, Some(362.0), None, None),
    ("C16-07", 6.3e-8, Some(222.0), None, None), ("C16-08", 5.5e-14, Some(125.0), None, None),
    ("C16-09", 1.6e-2, Some(514.0), None, None), ("C16-10", 3.6e-4, Some(189.0), None, None),
    ("C16-11", 2.2e-3, Some(488.0), None, None), ("C16-12", 2.3e-6, Some(197.0), None, None),
    ("C16-13", 1.2e-9, Some(205.0), None, None), ("C16-14", 1.8e6, Some(791.0), None, None),
    ("C16-15", 1.0e-2, Some(303.0), None, None),
    ("H21-01", 6.7e-9, Some(165.0), None, None), ("H21-02", 5.3e-9, Some(154.0), None, None),
    ("H21-03", 3.6e-9, Some(215.0), None, None), ("H21-04", 5.6e-8, Some(209.0), None, None),
    ("H21-05", 6.3e-8, Some(222.0), None, None), ("H21-06", 5.5e-14, Some(125.0), None, None),
    ("H21-07", 2.2e-3, Some(488.0), None, None), ("H21-08", 2.3e-6, Some(197.0), None, None),
    ("H21-09", 1.2e-9, Some(205.0), None, None),
];

const B6_SETS: [&str; 7] = [
    "T978-A1", "T978-A2", "T978-A3", "T978-A4", "T978-A5", "C16-T2", "H21-T1",
];

fn b6_checks(rep: &mut Report, reg: &Register) {
    let a = &reg.arrhenius;
    // coverage: every Ag/Cs/Sr row of the seven sets
    let covered: Vec<&str> = a
        .rows
        .iter()
        .filter(|r| {
            let el = r
                .species_text
                .split_whitespace()
                .next()
                .unwrap_or("")
                .split('-')
                .next()
                .unwrap_or("");
            matches!(el, "Ag" | "Cs" | "Sr") && B6_SETS.contains(&r.set_id.as_str())
        })
        .map(|r| r.id.as_str())
        .collect();
    let have: HashSet<&str> = covered.iter().copied().collect();
    let want: HashSet<&str> = B6.iter().map(|x| x.0).collect();
    let mut missing: Vec<String> = have.difference(&want).map(|s| (*s).to_string()).collect();
    let mut extra: Vec<String> = want.difference(&have).map(|s| (*s).to_string()).collect();
    missing.sort();
    extra.sort();
    rep.add(
        "D5.V.b6-coverage",
        missing.is_empty() && extra.is_empty() && covered.len() == B6.len(),
        format!(
            "B6 re-read covers every Ag/Cs/Sr row of A1-A5, C16-T2, H21-T1 ({} rows; missing {missing:?}, extra {extra:?})",
            B6.len()
        ),
        "",
    );
    let mism: Vec<String> = B6
        .iter()
        .filter(|(id, d0, q, tmin, tmax)| match a.get(id) {
            None => true,
            Some(r) => {
                let (rd, rq) = d0_q(r);
                rd != Some(*d0)
                    || rq != *q
                    || r.t_min_c.as_ref().map(|p| p.value) != *tmin
                    || r.t_max_c.as_ref().map(|p| p.value) != *tmax
            }
        })
        .map(|x| x.0.to_string())
        .collect();
    rep.add(
        "D5.V.b6-values",
        mism.is_empty(),
        format!(
            "B6 re-read: D0, Q, Tmin, Tmax identical to arrhenius.csv for {}/{} rows",
            B6.len() - mism.len(),
            B6.len()
        ),
        format!("mismatch: {mism:?}"),
    );
    // R per set; the Python dictionary keeps the last row of each three-character set prefix
    let mut rg: BTreeMap<String, String> = BTreeMap::new();
    for r in &a.rows {
        let text = match &r.gas_constant {
            GasConstant::Printed(p) => p.text.clone(),
            GasConstant::NotPrinted => "not printed".to_string(),
        };
        rg.insert(r.set_id.chars().take(3).collect(), text);
    }
    let get = |k: &str| rg.get(k).map(String::as_str);
    rep.add(
        "D5.V.rg",
        get("T97") == Some("8.3143")
            && get("C16") == Some("8.3145")
            && get("H21") == Some("not printed"),
        "R convention per set: TECDOC-978 8.3143, Collin 2016 8.3145, Hales 2021 'not printed'",
        format!("{rg:?}"),
    );
    let empty: Vec<String> = reg
        .release
        .rows
        .iter()
        .filter(|r| r.particle_applicability.is_empty())
        .map(|r| r.id.clone())
        .collect();
    rep.add(
        "D5.V.applicability",
        empty.is_empty(),
        "release_measurements.csv: particle_applicability filled for every row",
        first_n(&empty, 5),
    );
    let ub: Vec<String> = reg
        .release
        .rows
        .iter()
        .filter(|r| {
            matches!(r.value, ReleaseValue::UpperBound(_))
                && r.evidence_class != EvidenceClass::MeasuredBound
        })
        .map(|r| r.id.clone())
        .collect();
    rep.add(
        "D5.V.bounds",
        ub.is_empty(),
        "every '<' release value is classed 'measured bound'",
        first_n(&ub, 5),
    );
    let act: Vec<&YieldRow> = reg
        .yields
        .rows
        .iter()
        .filter(|r| {
            [
                "Ag-110m", "Ag-110", "Cs-134", "Cs-134m", "Eu-154", "Eu-154m",
            ]
            .contains(&r.product.name().as_str())
        })
        .collect();
    rep.add(
        "D5.V.activation-yields",
        act.len() == 12 && act.iter().all(|r| r.use_note_forbids_source),
        format!(
            "activation products (Ag-110m, Cs-134, Eu-154 and their isomer/ground partners): {} yield rows all flagged 'do NOT use as a fission-yield source'",
            act.len()
        ),
        "",
    );
    let src: Vec<String> = reg
        .decay
        .rows
        .iter()
        .filter(|r| r.product_class.is_direct_fission_product())
        .map(|r| r.nuclide.name())
        .collect();
    rep.add(
        "D5.V.fission-sources",
        !src.iter()
            .any(|n| ["Ag-110m", "Cs-134", "Eu-154"].contains(&n.as_str())),
        "decay.csv fission-source nuclides exclude Ag-110m, Cs-134, Eu-154",
        format!("{src:?}"),
    );
}

// ------------------------------------------------ b4_decay_source_pseudocode

fn yield_of(reg: &Register, parent: Parent, product: &str) -> Option<(f64, f64)> {
    let r = reg.yields.get(parent, product)?;
    Some((
        r.jeff_independent.reported()?.value.value,
        r.jeff_cumulative.reported()?.value.value,
    ))
}

fn b4_decay_checks(rep: &mut Report, reg: &Register) {
    let d = &reg.decay;
    let (n_stable, bad) = stable_lambda_zero(d);
    rep.add(
        "B4D.A.stable",
        bad.is_empty() && n_stable > 0,
        format!("stable nuclides have lambda = 0 ({n_stable} rows)"),
        "",
    );
    let mut w_hl = 0.0f64;
    let mut w_lam = 0.0f64;
    let mut w_prod = 0.0f64;
    let mut n = 0;
    for (_, x) in radioactive_rows(d) {
        n += 1;
        let t = x.half_life.value * x.unit.seconds();
        w_hl = w_hl.max(
            (x.half_life.value * x.unit.seconds() - x.half_life_s.value).abs()
                / x.half_life_s.value,
        );
        match decay_constant(x.half_life.value, x.unit) {
            Ok(lam) => {
                w_lam = w_lam.max((lam - x.lambda_per_s.value).abs() / x.lambda_per_s.value);
                w_prod = w_prod.max((lam * t - LN2).abs());
            }
            Err(_) => w_lam = f64::INFINITY,
        }
    }
    rep.add(
        "B4D.A.seconds",
        w_hl < 5e-9,
        format!("seconds conversion reproduces the half_life_s column ({n} radioactive rows)"),
        format!("max rel diff {w_hl:.1e}"),
    );
    rep.add(
        "B4D.A.lambda",
        w_lam < 5e-7,
        "lambda = ln2/t_half reproduces the lambda column (printed to 7 digits)",
        format!("max rel diff {w_lam:.1e}"),
    );
    rep.add(
        "B4D.A.product",
        w_prod < 1e-12,
        "lambda * t_half = ln 2 (dimensionless) for every row",
        format!("max abs diff {w_prod:.1e}"),
    );
    let lam = decay_constant(30.08, TimeUnit::Years);
    rep.add(
        "B4D.A.cs137",
        lam.as_ref().is_ok_and(|l| rel(*l, 7.302187e-10) < 1e-6),
        "Cs-137 lambda = 7.302187e-10 s^-1 from 30.08 y (Section 3.2)",
        format!("{lam:?}"),
    );
    rep.add(
        "B4D.A.mean-life",
        lam.as_ref()
            .is_ok_and(|l| (1.0 / l - 30.08 * TimeUnit::Years.seconds() / LN2).abs() < 1.0),
        "mean life 1/lambda = t_half/ln2 (P21.7) to 1 s",
        "",
    );
    rep.add(
        "B4D.A.negative",
        decay_constant(-1.0, TimeUnit::Seconds).is_err(),
        "a negative half-life is rejected",
        "",
    );
    let cs = yield_of(reg, Parent::U235, "Cs-137");
    let sr = yield_of(reg, Parent::U235, "Sr-90");
    let ok = match (cs, sr) {
        (Some((ci, cc)), Some((si, sc))) => {
            (cc - 6.2208e-2).abs() < 1e-6
                && (ci - 7.2248e-4).abs() < 1e-8
                && (sc - 5.7290e-2).abs() < 1e-6
                && (si - 3.1343e-4).abs() < 1e-8
        }
        _ => false,
    };
    rep.add(
        "B4D.B.yields",
        ok,
        "yields read from the register CSV agree with Section 3.3.2 (Cs-137, Sr-90, U-235 thermal; IFY, CFY)",
        format!("Cs-137 (IFY, CFY) {cs:?}, Sr-90 {sr:?}"),
    );
    let s_k = cs.map(|(_, c)| c * 5.0e19 / AVOGADRO);
    rep.add(
        "B4D.B.source-term",
        s_k.is_some_and(|s| (s - 6.2208e-2 * 5e19 / AVOGADRO).abs() < 1e-18),
        "S_K = y F/N_A (mol m^-3 s^-1) for Cs-137 and the synthetic rate density 5e19 fissions m^-3 s^-1",
        format!("{s_k:?}"),
    );
    let c235 = yield_of(reg, Parent::U235, "Cs-137").map(|y| y.1);
    let c239 = yield_of(reg, Parent::Pu239, "Cs-137").map(|y| y.1);
    let ok = match (c235, c239) {
        (Some(a), Some(b)) => {
            let yq = 0.7 * a + 0.3 * b;
            a.min(b) <= yq && yq <= a.max(b)
        }
        _ => false,
    };
    rep.add(
        "B4D.B.weighted-yield",
        ok,
        "P21.10 fission-weighted Cs-137 yield (0.7 U-235 + 0.3 Pu-239) is bracketed by the two pure yields",
        format!("{c235:?} {c239:?}"),
    );
}

// ------------------------------------------- b4_phase2_pseudocode_integration

fn b4_phase2_checks(rep: &mut Report, reg: &Register) {
    let table = [
        ("Cs-137", 7.302187e-10, 9.492323e8),
        ("Sr-90", 7.597709e-10, 9.123107e8),
        ("Ag-110m", 3.211198e-8, 2.158531e7),
        ("I-131", 9.996681e-7, 6.933773e5),
    ];
    for (nuc, lam_doc, t_doc) in table {
        let ts = reg.decay.get(nuc).and_then(|r| match &r.stability {
            Stability::Radioactive(x) => Some(x.half_life_s.value),
            Stability::Stable { .. } => None,
        });
        let lam = ts.map(|t| LN2 / t);
        rep.add(
            format!("B4P.nuclide.{nuc}"),
            matches!((lam, ts), (Some(l), Some(t)) if rel(l, lam_doc) < 5e-7 && rel(t, t_doc) < 1e-6),
            format!("{nuc}: lambda = ln 2 / t_half matches the pseudocode table (5e-7) and t_half matches (1e-6)"),
            format!("lambda {lam:?}, t_half {ts:?}"),
        );
    }
    let y = yield_of(reg, Parent::U235, "Cs-137").map(|y| y.1);
    let sk = y.map(|y| y * 5.0e19 / AVOGADRO);
    rep.add(
        "B4P.source-example",
        matches!((y, sk), (Some(y), Some(s)) if (y - 6.2208e-2).abs() < 1e-9 && (s - 5.1649e-6).abs() < 5e-11),
        "Cs-137 source example: y = 6.2208e-2, S_K = 5.1649e-6 mol m^-3 s^-1",
        format!("y {y:?}, S_K {sk:?}"),
    );
    let lam_of = |n: &str| {
        reg.decay.get(n).and_then(|r| match &r.stability {
            Stability::Radioactive(x) => Some(x.lambda_per_s.value),
            Stability::Stable { .. } => None,
        })
    };
    let dt_ag = lam_of("Ag-110m").map(|l| 2.0 * 1e-3 / (l * l * 620.0 * 86400.0));
    let dt_i = lam_of("I-131").map(|l| 2.0 * 1e-3 / (l * l * 5.0 / l));
    rep.add(
        "B4P.step-sizes",
        matches!((dt_ag, dt_i), (Some(a), Some(i)) if rel(a, 3.62e4) < 5e-3 && (i - 400.0).abs() < 1.0),
        "C1.2 examples: Ag-110m 3.62e4 s and I-131 400 s from lambda of decay.csv",
        format!("{dt_ag:?} s, {dt_i:?} s"),
    );
}

// ------------------------------------------- b6_nuclear_crosscheck (offline part)

fn b6_offline_checks(rep: &mut Report, reg: &Register) {
    let bad: Vec<String> = reg
        .decay
        .rows
        .iter()
        .filter(|r| ["Ag-110m", "Cs-134", "Eu-154"].contains(&r.nuclide.name().as_str()))
        .filter(|r| !r.product_class.is_activation_product())
        .map(|r| r.nuclide.name())
        .collect();
    rep.add(
        "B6.activation.decay",
        bad.is_empty(),
        "decay.csv: Ag-110m, Cs-134 and Eu-154 are classed as activation products",
        first_n(&bad, 3),
    );
    let bad: Vec<String> = reg
        .yields
        .rows
        .iter()
        .filter(|r| {
            [
                "Ag-110m", "Cs-134", "Eu-154", "Ag-110", "Cs-134m", "Eu-154m",
            ]
            .contains(&r.product.name().as_str())
        })
        .filter(|r| {
            !(r.product_class.is_activation_product()
                && (r.use_note.to_lowercase().contains("not")
                    || r.use_note.to_lowercase().contains("no ")))
        })
        .map(|r| format!("{} {}", r.parent.as_str(), r.product.name()))
        .collect();
    rep.add(
        "B6.activation.yields",
        bad.is_empty(),
        "fission_yields.csv: the six activation-product rows per parent are classed as activation products and their use note says not to use them",
        first_n(&bad, 3),
    );
    // the Rust type refuses a source term for them (new in Rust)
    let refused = reg
        .yields
        .rows
        .iter()
        .filter(|r| r.product_class.is_activation_product())
        .all(|r| r.fission_source_yield().is_err());
    rep.add(
        "B6.activation.no-source-term",
        refused,
        "no fission source-term yield can be built for any activation-product row (hard error in the type)",
        "",
    );
}

// ------------------------------------------------------------ generate_data.py

const GEN_LAYERS: [&str; 5] = ["kernel", "buffer", "IPyC", "SiC", "OPyC"];
const GEN_DEFAULT: &str = "AGR1-B-M16";

/// The AGR-1 sets drawn as the reference particle and the design of each layer row, from
/// `scripts/web/generate_data.py` (GEOMETRY_SETS).
pub const GEN_SETS: [(&str, [&str; 5]); 7] = [
    ("AGR1-B-M16", ["AGR1-B-M16"; 5]),
    ("AGR1-NOM-C16", ["AGR1-NOM-C16"; 5]),
    ("AGR1-SPEC", ["AGR1-SPEC"; 5]),
    (
        "AGR1-T714-B",
        [
            "AGR1-T714",
            "AGR1-T714-B",
            "AGR1-T714-B",
            "AGR1-T714-B",
            "AGR1-T714-B",
        ],
    ),
    (
        "AGR1-T714-V1",
        [
            "AGR1-T714",
            "AGR1-T714-V1",
            "AGR1-T714-V1",
            "AGR1-T714-V1",
            "AGR1-T714-V1",
        ],
    ),
    (
        "AGR1-T714-V2",
        [
            "AGR1-T714",
            "AGR1-T714-V2",
            "AGR1-T714-V2",
            "AGR1-T714-V2",
            "AGR1-T714-V2",
        ],
    ),
    (
        "AGR1-T714-V3",
        [
            "AGR1-T714",
            "AGR1-T714-V3",
            "AGR1-T714-V3",
            "AGR1-T714-V3",
            "AGR1-T714-V3",
        ],
    ),
];

/// One layer row of a reference-particle set, as `generate_data.py` selects it.
pub fn geometry_layer_row<'a>(
    g: &'a Geometry,
    design: &str,
    layer: &str,
) -> std::result::Result<&'a GeometryRow, String> {
    let want: [GeometryQuantity; 2] = if layer == "kernel" {
        [
            GeometryQuantity::Diameter,
            GeometryQuantity::DiameterSpecified,
        ]
    } else {
        [
            GeometryQuantity::Thickness,
            GeometryQuantity::ThicknessSpecified,
        ]
    };
    let hits: Vec<&GeometryRow> = g
        .rows
        .iter()
        .filter(|r| {
            r.design_id == design && r.layer.as_str() == layer && want.contains(&r.quantity)
        })
        .collect();
    if hits.len() != 1 {
        return Err(format!(
            "expected one {} row for {design} {layer}, found {}",
            want[0].as_str(),
            hits.len()
        ));
    }
    let r = hits[0];
    if r.unit != crate::geometry::GeometryUnit::Micrometre
        || !matches!(
            r.value,
            GeometryValue::Point(_) | GeometryValue::PointWithRange { .. }
        )
    {
        return Err(format!("row {} has no value in um", r.id));
    }
    Ok(r)
}

fn generate_data_checks(rep: &mut Report, g: &Geometry) {
    let mut row_problems = Vec::new();
    let mut radius_problems = Vec::new();
    for (set, designs) in &GEN_SETS {
        let mut outer = Decimal::from_int(0);
        for (layer, design) in GEN_LAYERS.iter().zip(designs.iter()) {
            let r = match geometry_layer_row(g, design, layer) {
                Ok(r) => r,
                Err(m) => {
                    row_problems.push(format!("{set}: {m}"));
                    continue;
                }
            };
            let Some(v) = point_decimal(r) else {
                row_problems.push(format!("{set}: {} is not a plain decimal", r.id));
                continue;
            };
            outer = if *layer == "kernel" {
                v.half()
            } else {
                outer + v
            };
            if let Some(reg_r) = &r.outer_radius_um_calc {
                match Decimal::parse(&reg_r.text) {
                    Ok(d) if d == outer => {}
                    _ => radius_problems.push(format!(
                        "row {} outer radius {} um, summed {outer} um",
                        r.id, reg_r.text
                    )),
                }
            }
        }
    }
    rep.add(
        "GEN.geometry.rows",
        row_problems.is_empty(),
        "each of the 7 AGR-1 reference sets has exactly one kernel diameter and four thickness rows, in um, with a value",
        first_n(&row_problems, 3),
    );
    rep.add(
        "GEN.geometry.radii",
        radius_problems.is_empty() && row_problems.is_empty(),
        "where the register gives an outer radius it equals the sum of diameter/2 and thicknesses exactly (decimal arithmetic)",
        first_n(&radius_problems, 3),
    );
    rep.add(
        "GEN.geometry.default",
        GEN_SETS.iter().any(|(s, _)| *s == GEN_DEFAULT),
        "the default set AGR1-B-M16 is one of the sets",
        "",
    );
}

// ------------------------------------------------------- check_catalogue.py rule 6

/// Items of a `;`-separated cell; a `;` inside parentheses belongs to its item (`split_list` of generate_data.py).
pub fn split_list(cell: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut cur = String::new();
    let mut depth = 0usize;
    for ch in cell.chars() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' && depth > 0 {
            depth -= 1;
        }
        if ch == ';' && depth == 0 {
            items.push(std::mem::take(&mut cur));
        } else {
            cur.push(ch);
        }
    }
    items.push(cur);
    items
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

const NOT_IN_REGISTER: &str = "(not in register)";

fn catalogue_checks(rep: &mut Report, d: &Decay, species_csv: Option<&str>) {
    let Some(text) = species_csv else {
        rep.add(
            "CAT.nuclides",
            true,
            "species.csv nuclides are in decay.csv or marked '(not in register)'",
            "skipped: data/research/species.csv is absent, as in check_catalogue.py",
        );
        return;
    };
    let recs = match csv::parse("research/species.csv", text) {
        Ok(r) => r,
        Err(e) => {
            rep.add(
                "CAT.nuclides",
                false,
                "species.csv is readable",
                e.to_string(),
            );
            return;
        }
    };
    let Some(col) = recs[0].cells.iter().position(|h| h == "nuclides") else {
        rep.add(
            "CAT.nuclides",
            false,
            "species.csv has a nuclides column",
            "",
        );
        return;
    };
    let registered: HashSet<String> = d.rows.iter().map(|r| r.nuclide.name()).collect();
    let mut probs = Vec::new();
    for rec in &recs[1..] {
        for n in split_list(rec.cells.get(col).map_or("", String::as_str)) {
            if n.ends_with(NOT_IN_REGISTER) {
                continue;
            }
            if !registered.contains(&n) {
                probs.push(format!("species.csv line {}: nuclide '{n}' is not in decay.csv and is not marked '{NOT_IN_REGISTER}'", rec.line));
            }
        }
    }
    rep.add(
        "CAT.nuclides",
        probs.is_empty(),
        "every nuclide named in species.csv is in decay.csv or marked '(not in register)'",
        first_n(&probs, 3),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equals_claims_are_found() {
        assert_eq!(
            equals_claim("equals A1-01 of TECDOC"),
            Some("A1-01".to_string())
        );
        assert_eq!(
            equals_claim("x equals C16-05; y"),
            Some("C16-05".to_string())
        );
        assert_eq!(equals_claim("equals nothing"), None);
        assert_eq!(equals_claim("equals A12"), None);
    }

    #[test]
    fn split_list_keeps_semicolons_inside_parentheses() {
        assert_eq!(split_list("a; b (c; d);e"), vec!["a", "b (c; d)", "e"]);
        assert_eq!(split_list(""), Vec::<String>::new());
    }

    #[test]
    fn booth_series_limits() {
        assert!(booth(0.0, 200_000).abs() < 1e-5);
        assert!((booth(10.0, 100) - 1.0).abs() < 1e-12);
    }
}
