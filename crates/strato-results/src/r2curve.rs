//! The R2 records of one curve (`metric-r2.md` Sections 1.2 to 3 and 7): the point classes, the
//! early-bound rule, the decay checks, the seventeen ordered checks, the display, the label, the
//! premises, the independent-check fields and the caption record.
//!
//! The functions take plain inputs (rows, the curve assessment, the model values) and know nothing
//! about the registers or the solver, so that each check can be driven on a constructed curve by a
//! test. `r2run` builds the inputs from the registers and the model runs.

use strato_compare::json::J;
use strato_data::curve::{BoundClass, DecayCorrection, SpecimenLevel};

use crate::r2::{
    self, budget, decay_gate_delta, interval, interval_display, label, point_sums,
    systematic_interval, systematic_interval_display, zero_variance, Check, NoInterval, Point,
    ReasonCode, BUDGET_MAX, DEFINITION_ID, N_MIN,
};

pub use crate::labels::{COLOUR_A, COLOUR_B, COLOUR_EXP};

/// The two methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Method A: BTCS.
    A,
    /// Method B: Sturm-Liouville.
    B,
}

impl Method {
    /// `A` or `B`.
    pub fn letter(self) -> char {
        match self {
            Method::A => 'A',
            Method::B => 'B',
        }
    }

    /// The name of the method as the graphs write it.
    pub fn name(self) -> &'static str {
        match self {
            Method::A => "Method A: BTCS",
            Method::B => "Method B: Sturm\u{2013}Liouville",
        }
    }

    /// The colour of the method.
    pub fn colour(self) -> &'static str {
        match self {
            Method::A => COLOUR_A,
            Method::B => COLOUR_B,
        }
    }
}

/// Whether the source states that release and inventory are corrected to one common reference time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecayReference {
    /// Stated.
    Common,
    /// Not stated.
    NotStated,
}

/// The reference time of the denominator inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryReference {
    /// The time origin of the test.
    TestTimeOrigin,
    /// An earlier time (for example the end of irradiation); the interval is
    /// `inventory_reference_offset_h`.
    Earlier,
    /// Not stated.
    NotStated,
}

/// The class of one point (Section 2.4 and decision 74, ruling 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointClass {
    /// A measured point.
    Measured,
    /// A detection limit (`<`, M-ND, non-detect).
    DetectionBound,
    /// Any other upper bound.
    OtherUpperBound,
    /// A window upper bound.
    WindowBound,
    /// A compact or sphere total (`F_tot`).
    CompactTotalBound,
    /// The bound class is unset.
    Unclassified,
}

/// The class of a row. The precedence is decision 74, ruling 1: unclassified first, then window and
/// compact-total bounds, then detection bounds, then other upper bounds.
pub fn classify(r: &RowInput) -> PointClass {
    match r.bound_class {
        None => PointClass::Unclassified,
        Some(BoundClass::WindowUpperBound) => PointClass::WindowBound,
        Some(BoundClass::CompactTotal) => PointClass::CompactTotalBound,
        Some(bc) => {
            if r.qualifier == "<" || r.value_class == "M-ND" || bc == BoundClass::NonDetect {
                PointClass::DetectionBound
            } else if r.value_class == "M-UB" || bc == BoundClass::UpperBound {
                PointClass::OtherUpperBound
            } else if r.qualifier == ">" || r.qualifier == "range" {
                // Neither a measured point nor an upper bound of the particle release: it is not
                // a point of the curve.
                PointClass::OtherUpperBound
            } else {
                PointClass::Measured
            }
        }
    }
}

/// One row of a curve.
#[derive(Debug, Clone)]
pub struct RowInput {
    /// Register id.
    pub id: String,
    /// The time as registered, h, in the time basis of the row.
    pub time_h: f64,
    /// The value used for R2: the registered value, or, where the register gives particle
    /// equivalents, the registered value divided by the registered particle count.
    pub value: f64,
    /// The registered value, unchanged.
    pub registered_value: f64,
    /// Qualifier of the register (`=`, `~`, `<`, ...).
    pub qualifier: String,
    /// Value class of the register (`R-dig`, `M-ND`, ...).
    pub value_class: String,
    /// The bound class; `None` where the curve assessment leaves it unset.
    pub bound_class: Option<BoundClass>,
    /// Register status (`verified`, `digitised`, `NOT VERIFIED`).
    pub status: String,
    /// True if the value was read from a figure.
    pub digitised: bool,
    /// Where the independent coordinate verification of a digitised row is recorded.
    pub digitisation_verification_ref: Option<String>,
    /// Half-width of the value reading uncertainty, absolute, in the unit of `value`.
    pub u_value: Option<f64>,
    /// Half-width of the time reading uncertainty, h.
    pub u_time_h: Option<f64>,
    /// Delay from the time of the point to the counting of its latest-counted contribution, h;
    /// `None` where the source does not state it (decision 74, ruling 3).
    pub counting_delay_h: Option<f64>,
    /// The printed row of which this row is the digitised duplicate.
    pub duplicate_of: Option<String>,
    /// True if a negative value is presented by the source as a cumulative release fraction.
    pub negative_presented_as_release: bool,
    /// Time of the point measured from the time origin of the test, h.
    pub time_el_h: f64,
}

/// What the page of a curve says about its source.
#[derive(Debug, Clone, Default)]
pub struct SourceInfo {
    /// The APA 7 reference.
    pub apa: String,
    /// The DOI or URL.
    pub link: String,
    /// `verified` or the statement that no check is recorded.
    pub link_status: String,
    /// Figure, table or page as the register writes it.
    pub location: String,
    /// A limitation that the register states about the reference itself.
    pub note: Option<String>,
}

/// The words of the caption record.
#[derive(Debug, Clone, Default)]
pub struct CaptionWords {
    /// `time at temperature (h)` and the like.
    pub time_axis_label: String,
    /// `compact 6-4-3` and the like, with `, source mean` where aggregated.
    pub specimen_label: String,
    /// `linear` or `log`.
    pub y_axis_scale: String,
}

/// Everything about one curve that the checks read.
#[derive(Debug, Clone)]
pub struct CurveInput {
    /// Curve id (the dataset id of the assessment).
    pub curve_id: String,
    /// Nuclide as registered.
    pub nuclide: String,
    /// Programme as registered.
    pub program: String,
    /// The specimen as registered (`compact_or_sample`).
    pub specimen_id: String,
    /// Level of the specimen; `None` where unset.
    pub specimen_level: Option<SpecimenLevel>,
    /// True where the source prints one curve over several specimens; `None` where unset.
    pub source_aggregated: Option<bool>,
    /// Hold temperature, degC.
    pub temperature_c: Option<f64>,
    /// The label derived from the registered history (`1600 degC`, `stepped 1000 to 1600 degC`).
    pub temperature_label: String,
    /// The registered history ids used.
    pub temperature_history_id: Option<String>,
    /// The release definition as registered.
    pub release_definition: String,
    /// The denominator as registered.
    pub denominator: String,
    /// The time basis as registered.
    pub time_basis: String,
    /// Geometry, diffusivity rows, initial condition and solver settings.
    pub model_config_id: String,
    /// `independent`, `calibration_check` or `not_established`.
    pub independence: String,
    /// The source.
    pub source: SourceInfo,
    /// `measured_tabulated`, `digitised`, `calculated` or `mixed`.
    pub data_kind: String,
    /// The unit of the registered values.
    pub registered_unit: String,
    /// Statement of any conversion of the registered values to the quantity of the model.
    pub conversion: Option<String>,
    /// The rows in time order.
    pub rows: Vec<RowInput>,
    /// The basis of a known or suspected coating failure; `None` where unset, `none` where the
    /// reviewer found none.
    pub coating_failure_basis: Option<String>,
    /// True if the register marks the specimen as failed (group, status or applicability).
    pub register_marks_failure: bool,
    /// True if the source itself separates the matrix inventory from the particle release.
    pub matrix_separated_by_source: bool,
    /// The decay correction of the source (an unset field is read as not stated).
    pub decay_correction: DecayCorrection,
    /// Common reference time of release and inventory; `None` where unset.
    pub decay_reference: Option<DecayReference>,
    /// Reference time of the denominator inventory; `None` where unset.
    pub inventory_reference: Option<InventoryReference>,
    /// The registered interval from the inventory reference time to the time origin, h.
    pub inventory_reference_offset_h: Option<f64>,
    /// Decay constant of the nuclide, per hour.
    pub lambda_per_h: Option<f64>,
    /// Where the independent verification of the curve assessment is recorded.
    pub assessment_verification_ref: Option<String>,
    /// Where the independent recomputation of R2 is recorded (check V1).
    pub r2_verification_ref: Option<String>,
    /// True if the nuclide has a registered diffusivity row (check M0).
    pub nuclide_configurable: bool,
    /// True if the denominator is established and equals the particle inventory (check C4).
    pub denominator_established: bool,
    /// True if the release definition and time origin are those of Section 2.1 (check C4).
    pub definition_matches: bool,
    /// True if a registered temperature history covers the time origin to the last point (C5).
    pub history_covers: bool,
    /// True if every registered history row used is verified (C0).
    pub history_rows_verified: bool,
    /// Essential conditions that the source does not state, one phrase each (check C6).
    pub missing_conditions: Vec<String>,
    /// Premises of the model configuration, `(code, text)`.
    pub premises: Vec<(String, String)>,
    /// A systematic uncertainty common to the dataset that the source states (fraction), if any.
    pub u_systematic: Option<f64>,
    /// The kind and size of the value reading uncertainty: `rel_pct` or `abs`, and the number.
    pub u_value_summary: Option<(String, f64)>,
    /// The largest time reading uncertainty, h.
    pub u_time_summary_h: Option<f64>,
    /// The words of the caption record.
    pub words: CaptionWords,
}

/// The model values of one method at the rows of a curve (one entry per row of the curve).
#[derive(Debug, Clone)]
pub struct MethodInput {
    /// The method.
    pub method: Method,
    /// `t_mod,i`, h, where evaluated.
    pub model_times_h: Vec<Option<f64>>,
    /// `F_M,i` at the exact step boundary.
    pub f_model: Vec<Option<f64>>,
    /// `F_M` at the lowest shifted time.
    pub f_minus: Vec<Option<f64>>,
    /// `F_M` at the highest shifted time.
    pub f_plus: Vec<Option<f64>>,
    /// The lowest shifted time, h.
    pub t_minus_h: Vec<Option<f64>>,
    /// The highest shifted time, h.
    pub t_plus_h: Vec<Option<f64>>,
    /// The discretisation-error estimate `e_i`.
    pub e_num: Vec<Option<f64>>,
    /// The 5 % rule of science design Section 3.5 at the row.
    pub resolved: Vec<bool>,
    /// True if the run reaches the last point plus its time uncertainty.
    pub reaches_last_point: bool,
    /// True if every diffusivity row is inside its printed range or documented application (M1).
    pub validity_ok: bool,
    /// B only: the applicability function accepts the configuration (B1).
    pub applicable: bool,
    /// B only: the drawing mask passes at every matched time (B2).
    pub mask_passed: bool,
    /// True if the geometry, diffusivity and decay rows of the configuration are verified (C0).
    pub model_rows_verified: bool,
    /// Solver commit, configuration id and output hashes, or the Method B commit and `alpha_max`.
    pub run_ref: Option<J>,
}

impl MethodInput {
    /// A method input with no value at any row (the model could not be run).
    pub fn empty(method: Method, rows: usize) -> MethodInput {
        MethodInput {
            method,
            model_times_h: vec![None; rows],
            f_model: vec![None; rows],
            f_minus: vec![None; rows],
            f_plus: vec![None; rows],
            t_minus_h: vec![None; rows],
            t_plus_h: vec![None; rows],
            e_num: vec![None; rows],
            resolved: vec![false; rows],
            reaches_last_point: false,
            validity_ok: true,
            applicable: method == Method::A,
            mask_passed: method == Method::A,
            model_rows_verified: false,
            run_ref: None,
        }
    }
}

/// One record of `r2.json`.
#[derive(Debug, Clone)]
pub struct R2Record {
    /// `r2.<curve_id>.<A|B>`.
    pub record_id: String,
    /// The curve.
    pub curve_id: String,
    /// The method.
    pub method: Method,
    /// True for `status = available`.
    pub available: bool,
    /// The first failing check.
    pub first_failing: Option<Check>,
    /// The reason code, the first failing check's.
    pub reason: Option<ReasonCode>,
    /// Every failing check, in the order of Section 3.1.
    pub failed_checks: Vec<Check>,
    /// The unrounded R2; `None` unless available.
    pub r2: Option<f64>,
    /// The display string.
    pub display: Option<String>,
    /// LaTeX label.
    pub label_latex: Option<String>,
    /// Plain label.
    pub label_plain: Option<String>,
    /// N, the length of `row_ids`.
    pub n: usize,
    /// The rows after the exclusions (every row for an unavailable curve).
    pub row_ids: Vec<String>,
    /// `E-id:early_bound` and `E-id:duplicate_of:<E-id>`.
    pub rows_excluded: Vec<String>,
    /// The rows counted for the sums (after the exclusions).
    pub counted_rows: Vec<String>,
    /// The premises.
    pub premises: Vec<(String, String)>,
    /// `decay_gate_delta`.
    pub decay_gate_delta: Option<f64>,
    /// The record as JSON.
    pub json: J,
}

fn jnum(x: Option<f64>) -> J {
    J::opt(x)
}

fn jnums(v: &[Option<f64>]) -> J {
    J::Arr(v.iter().map(|x| J::opt(*x)).collect())
}

fn jstrs(v: &[String]) -> J {
    J::Arr(v.iter().map(J::s).collect())
}

fn jopts(v: Option<&str>) -> J {
    v.map_or(J::Null, J::s)
}

/// What is known of the rows before any method is looked at.
struct Prepared {
    /// Classes of the rows, by index.
    classes: Vec<PointClass>,
    /// Indices of the rows that are duplicates of a printed row.
    duplicates: Vec<usize>,
    /// Indices of the early-bound rows that are excluded.
    early: Vec<usize>,
    /// Indices counted for the sums.
    used: Vec<usize>,
    /// Failing checks of scope curve, with the C0 part for the rows and the assessment.
    failed: Vec<Check>,
    /// Premises added by the preparation.
    premises: Vec<(String, String)>,
    /// `t_d,i` for the used rows, where D1 passes and the correction is not stated.
    t_d: Option<Vec<f64>>,
}

const PREMISE_EARLY: &str = "early points are bounds; R\u{b2} covers the measured points from t_1";
const PREMISE_NO_UNCERTAINTY: &str =
    "no per-point uncertainty stated; zero-variance test at floating-point level only";
const PREMISE_DECAY: &str = "decay not modelled: release assumed decay-corrected";

/// Add a failing check once.
fn add(failed: &mut Vec<Check>, ch: Check, cond: bool) {
    if cond && !failed.contains(&ch) {
        failed.push(ch);
    }
}

fn prepare(c: &CurveInput) -> Prepared {
    let classes: Vec<PointClass> = c.rows.iter().map(classify).collect();
    let mut duplicates = Vec::new();
    for (i, r) in c.rows.iter().enumerate() {
        if r.duplicate_of.is_some() {
            duplicates.push(i);
        }
    }
    let active: Vec<usize> = (0..c.rows.len())
        .filter(|i| !duplicates.contains(i))
        .collect();
    // Early-bound exclusion (Section 2.4): only detection bounds, only when at least one measured
    // point exists and every detection bound lies before the first measured point.
    let first_measured = active
        .iter()
        .copied()
        .find(|i| classes[*i] == PointClass::Measured);
    let mut early = Vec::new();
    if let Some(fm) = first_measured {
        let detection: Vec<usize> = active
            .iter()
            .copied()
            .filter(|i| classes[*i] == PointClass::DetectionBound)
            .collect();
        let pos = |i: usize| active.iter().position(|x| *x == i).unwrap_or(usize::MAX);
        if !detection.is_empty() && detection.iter().all(|d| pos(*d) < pos(fm)) {
            early = detection;
        }
    }
    let used: Vec<usize> = active
        .iter()
        .copied()
        .filter(|i| !early.contains(i))
        .collect();
    let mut premises = Vec::new();
    if !early.is_empty() {
        premises.push(("early_bound".to_string(), PREMISE_EARLY.to_string()));
    }
    let mut failed: Vec<Check> = Vec::new();
    let unclassified = used.iter().any(|i| classes[*i] == PointClass::Unclassified);
    let known_bad = used.iter().any(|i| {
        matches!(
            classes[*i],
            PointClass::WindowBound
                | PointClass::CompactTotalBound
                | PointClass::OtherUpperBound
                | PointClass::DetectionBound
        )
    });
    let no_measured = !used.iter().any(|i| classes[*i] == PointClass::Measured);
    // M0.
    add(&mut failed, Check::M0, !c.nuclide_configurable);
    // C1: a known non-measured point that the exclusion did not remove; a curve with no measured
    // point. An unclassified row is not known to be a bound (decision 74, ruling 2): it gives C6.
    add(
        &mut failed,
        Check::C1,
        known_bad || (no_measured && !unclassified),
    );
    // C2.
    let basis_fails = c
        .coating_failure_basis
        .as_ref()
        .is_some_and(|b| !b.trim().eq_ignore_ascii_case("none"));
    add(
        &mut failed,
        Check::C2,
        basis_fails || c.register_marks_failure,
    );
    // C3: the specimen holds inventory outside the particles that the source does not separate.
    let matrix = matches!(
        c.specimen_level,
        Some(SpecimenLevel::Compact) | Some(SpecimenLevel::Sphere)
    );
    add(
        &mut failed,
        Check::C3,
        matrix && !c.matrix_separated_by_source,
    );
    // C4.
    let negative_bad = used.iter().any(|i| {
        let r = &c.rows[*i];
        r.value < 0.0
            && !(r.negative_presented_as_release && r.u_value.is_some_and(|u| r.value.abs() > u))
    });
    add(
        &mut failed,
        Check::C4,
        !c.denominator_established || !c.definition_matches || negative_bad,
    );
    // C5.
    add(&mut failed, Check::C5, !c.history_covers);
    // C6: an unset assessment field or a condition the source does not state (decision 74,
    // ruling 2: never a reason that states an unestablished fact).
    let decay_unset = match c.decay_correction {
        DecayCorrection::Corrected => c.decay_reference.is_none(),
        DecayCorrection::NotStated => c.inventory_reference.is_none(),
        DecayCorrection::NotCorrected => false,
    };
    add(
        &mut failed,
        Check::C6,
        !c.missing_conditions.is_empty()
            || c.specimen_level.is_none()
            || c.source_aggregated.is_none()
            || c.coating_failure_basis.is_none()
            || unclassified
            || decay_unset,
    );
    // D1 (Section 2.5).
    let delays_missing = used.iter().any(|i| c.rows[*i].counting_delay_h.is_none());
    let d1 = match c.decay_correction {
        DecayCorrection::NotCorrected => true,
        DecayCorrection::Corrected => c.decay_reference == Some(DecayReference::NotStated),
        DecayCorrection::NotStated => {
            c.inventory_reference == Some(InventoryReference::NotStated)
                || (c.inventory_reference == Some(InventoryReference::Earlier)
                    && c.inventory_reference_offset_h.is_none())
                || delays_missing
        }
    };
    add(&mut failed, Check::D1, d1);
    if c.decay_correction == DecayCorrection::NotStated {
        premises.push(("decay_not_modelled".to_string(), PREMISE_DECAY.to_string()));
    }
    // C7 and C8.
    add(&mut failed, Check::C7, used.len() < N_MIN);
    let bare_points = zero_points(c, &used);
    if !used.is_empty() {
        add(&mut failed, Check::C8, zero_variance(&bare_points));
        if !bare_points
            .iter()
            .all(|p| p.u_value.is_some_and(|u| u > 0.0))
        {
            premises.push((
                "zero_variance_floating_point".to_string(),
                PREMISE_NO_UNCERTAINTY.to_string(),
            ));
        }
    }
    // C0 (curve part): the rows used, the registered history and the curve assessment.
    let rows_bad = used.iter().any(|i| {
        let r = &c.rows[*i];
        r.status == "NOT VERIFIED"
            || (r.status == "digitised" && r.digitisation_verification_ref.is_none())
            || (r.status != "verified" && r.status != "digitised")
    });
    add(
        &mut failed,
        Check::C0,
        rows_bad || !c.history_rows_verified || c.assessment_verification_ref.is_none(),
    );
    // The age of the counting: `t_d,i = Delta_ref + t_el,i + t_cnt,i`, only where it is defined.
    let t_d = if c.decay_correction == DecayCorrection::NotStated && !failed.contains(&Check::D1) {
        let delta_ref = match c.inventory_reference {
            Some(InventoryReference::TestTimeOrigin) => Some(0.0),
            Some(InventoryReference::Earlier) => c.inventory_reference_offset_h,
            _ => None,
        };
        delta_ref.map(|d| {
            used.iter()
                .map(|i| d + c.rows[*i].time_el_h + c.rows[*i].counting_delay_h.unwrap_or(0.0))
                .collect()
        })
    } else {
        None
    };
    Prepared {
        classes,
        duplicates,
        early,
        used,
        failed,
        premises,
        t_d,
    }
}

/// The points of the used rows with no model value (for the tests that read the experiment only).
fn zero_points(c: &CurveInput, used: &[usize]) -> Vec<Point> {
    used.iter()
        .map(|i| {
            let r = &c.rows[*i];
            Point {
                id: r.id.clone(),
                f_exp: r.value,
                f_model: 0.0,
                u_value: r.u_value,
                u_time_h: r.u_time_h,
                f_model_t_minus: None,
                f_model_t_plus: None,
                t_minus_h: None,
                t_plus_h: None,
                e_num: 0.0,
                resolved: true,
                digitised: r.digitised,
            }
        })
        .collect()
}

/// The points of the used rows with the model values of a method, or `None` where a value is
/// missing.
fn model_points(c: &CurveInput, used: &[usize], m: &MethodInput) -> Option<Vec<Point>> {
    let mut v = Vec::new();
    for i in used {
        let r = &c.rows[*i];
        let f_model = (*m.f_model.get(*i)?)?;
        let e_num = (*m.e_num.get(*i)?)?;
        v.push(Point {
            id: r.id.clone(),
            f_exp: r.value,
            f_model,
            u_value: r.u_value,
            u_time_h: r.u_time_h,
            f_model_t_minus: m.f_minus[*i],
            f_model_t_plus: m.f_plus[*i],
            t_minus_h: m.t_minus_h[*i],
            t_plus_h: m.t_plus_h[*i],
            e_num,
            resolved: m.resolved[*i],
            digitised: r.digitised,
        });
    }
    Some(v)
}

/// The data checks, those that decide whether the sums of a curve are formed at all.
const DATA_CHECKS: [Check; 9] = [
    Check::C1,
    Check::C2,
    Check::C3,
    Check::C4,
    Check::C5,
    Check::C6,
    Check::D1,
    Check::C7,
    Check::C8,
];

fn method_checks(
    c: &CurveInput,
    prep: &Prepared,
    m: Option<&MethodInput>,
) -> (Vec<Check>, Option<Vec<Point>>, Option<f64>) {
    let mut failed: Vec<Check> = prep.failed.clone();
    let Some(m) = m else {
        // No model exists (M0): nothing else of the method can be evaluated.
        failed.sort();
        failed.dedup();
        return (failed, None, None);
    };
    add(&mut failed, Check::M1, !m.validity_ok);
    let pts = model_points(c, &prep.used, m);
    let data_ok = !prep.failed.iter().any(|ch| DATA_CHECKS.contains(ch));
    // Method B has no values to resolve where it does not apply or the mask withholds a value:
    // B1 or B2 says so, and M2 is not evaluated.
    let no_b_run = m.method == Method::B && !(m.applicable && m.mask_passed);
    let mut m2 = !no_b_run && (!m.reaches_last_point || pts.is_none());
    if let Some(p) = pts.as_ref().filter(|_| !no_b_run) {
        if p.iter().any(|q| !q.resolved) {
            m2 = true;
        } else if data_ok {
            let s = point_sums(p);
            if s.ss_tot > 0.0 && s.ss_tot.is_finite() {
                m2 = m2 || budget(p, s.ss_tot) > BUDGET_MAX || !budget(p, s.ss_tot).is_finite();
            }
        }
    }
    add(&mut failed, Check::M2, m2);
    if m.method == Method::B {
        add(&mut failed, Check::B1, !m.applicable);
        add(&mut failed, Check::B2, m.applicable && !m.mask_passed);
    }
    // D2 only after the model checks of the method have passed and where the age of the counting
    // is defined.
    let mut gate = None;
    let model_ok = !failed
        .iter()
        .any(|ch| matches!(ch, Check::M1 | Check::M2 | Check::B1 | Check::B2));
    if model_ok && data_ok {
        if let (Some(p), Some(td), Some(l)) = (&pts, &prep.t_d, c.lambda_per_h) {
            let delta = decay_gate_delta(p, l, td);
            gate = Some(delta);
            add(&mut failed, Check::D2, delta > BUDGET_MAX);
        }
    }
    add(&mut failed, Check::C0, !m.model_rows_verified);
    add(&mut failed, Check::V1, c.r2_verification_ref.is_none());
    failed.sort();
    failed.dedup();
    (failed, pts, gate)
}

fn one_record(
    c: &CurveInput,
    prep: &Prepared,
    m: Option<&MethodInput>,
    method: Method,
) -> R2Record {
    let (failed, pts, gate) = method_checks(c, prep, m);
    let available = failed.is_empty();
    let first = failed.first().copied();
    let reason = first.map(Check::reason);
    let data_ok = !prep.failed.iter().any(|ch| DATA_CHECKS.contains(ch));
    // The sums are formed only where every data check passes and the model values exist; the
    // value is stored only for an available record.
    let sums = match (&pts, data_ok, failed.contains(&Check::M0)) {
        (Some(p), true, false) if !p.is_empty() => Some(point_sums(p)),
        _ => None,
    };
    let budget_value = match (&pts, &sums) {
        (Some(p), Some(s)) if s.ss_tot > 0.0 => Some(budget(p, s.ss_tot)),
        _ => None,
    };
    let r2_value = if available {
        sums.as_ref().map(r2::Sums::r2)
    } else {
        None
    };
    let display = match (&sums, r2_value) {
        (Some(s), Some(v)) => Some(r2::display(v, s.ss_res)),
        _ => None,
    };
    let (label_latex, label_plain) = match &display {
        Some(d) => {
            let (a, b) = label(method.letter(), d);
            (Some(a), Some(b))
        }
        None => (None, None),
    };
    // Rows of the record: after the exclusions for an available curve, every row otherwise.
    let keep: Vec<usize> = if available {
        prep.used.clone()
    } else {
        (0..c.rows.len()).collect()
    };
    let row_ids: Vec<String> = keep.iter().map(|i| c.rows[*i].id.clone()).collect();
    let mut excluded: Vec<String> = prep
        .early
        .iter()
        .map(|i| format!("{}:early_bound", c.rows[*i].id))
        .collect();
    for i in &prep.duplicates {
        excluded.push(format!(
            "{}:duplicate_of:{}",
            c.rows[*i].id,
            c.rows[*i].duplicate_of.clone().unwrap_or_default()
        ));
    }
    let counted: Vec<String> = prep.used.iter().map(|i| c.rows[*i].id.clone()).collect();
    // The reading-uncertainty interval, only for an available record.
    let mut premises = c.premises.clone();
    premises.extend(prep.premises.clone());
    match c.independence.as_str() {
        "calibration_check" => premises.push((
            "calibration_check".to_string(),
            "parameter fitted to these data: calibration check, not an independent comparison"
                .to_string(),
        )),
        "not_established" => premises.push((
            "independence_not_established".to_string(),
            "independence of the parameters from these data not established".to_string(),
        )),
        _ => {}
    }
    let scope = if c.rows.iter().any(|r| r.digitised) {
        "reading uncertainty only"
    } else {
        "stated measurement uncertainty"
    };
    let iv = match (&pts, available) {
        (Some(p), true) => Some(interval(p)),
        _ => None,
    };
    let digitisation = {
        let (status, lo, hi, half, corner, disp) = match &iv {
            Some(Ok(v)) => (
                "computed",
                J::Num(v.r2_low),
                J::Num(v.r2_high),
                J::Num(v.first_order_half_width),
                J::s(v.corner_method.word()),
                J::s(interval_display(v, scope)),
            ),
            Some(Err(NoInterval::NotApplicable)) => (
                "not applicable",
                J::Null,
                J::Null,
                J::Null,
                J::Null,
                J::Null,
            ),
            _ => (
                if c.rows.iter().any(|r| r.digitised) {
                    "not available"
                } else {
                    "not applicable"
                },
                J::Null,
                J::Null,
                J::Null,
                J::Null,
                J::Null,
            ),
        };
        J::obj(vec![
            (
                "u_value",
                match &c.u_value_summary {
                    Some((k, v)) => J::obj(vec![("kind", J::s(k.clone())), ("value", J::Num(*v))]),
                    None => J::Null,
                },
            ),
            ("u_time_h", jnum(c.u_time_summary_h)),
            ("status", J::s(status)),
            ("r2_low", lo),
            ("r2_high", hi),
            ("first_order_half_width", half),
            ("corner_method", corner),
            ("scope", J::s(scope)),
            ("interval_display", disp),
        ])
    };
    let systematic = match (&pts, available, c.u_systematic) {
        (Some(p), true, Some(u)) => {
            let (lo, hi) = systematic_interval(p, u);
            Some((lo, hi, systematic_interval_display(u, lo, hi)))
        }
        _ => None,
    };
    // Decay fields.
    let t_d_h: Vec<Option<f64>> = (0..c.rows.len())
        .map(|i| {
            prep.t_d.as_ref().and_then(|td| {
                prep.used
                    .iter()
                    .position(|u| *u == i)
                    .and_then(|k| td.get(k).copied())
            })
        })
        .collect();
    let lambda_t_n = match (c.lambda_per_h, prep.used.last()) {
        (Some(l), Some(last)) => t_d_h[*last].map(|t| l * t),
        _ => None,
    };
    let at = |v: &[Option<f64>]| -> Vec<Option<f64>> { keep.iter().map(|i| v[*i]).collect() };
    let empty_f = vec![None; c.rows.len()];
    let (mt, fm, fmm, fmp, en) = match m {
        Some(m) => (
            at(&m.model_times_h),
            at(&m.f_model),
            at(&m.f_minus),
            at(&m.f_plus),
            at(&m.e_num),
        ),
        None => (
            at(&empty_f),
            at(&empty_f),
            at(&empty_f),
            at(&empty_f),
            at(&empty_f),
        ),
    };
    let rows_of =
        |f: &dyn Fn(&RowInput) -> J| J::Arr(keep.iter().map(|i| f(&c.rows[*i])).collect());
    let early_points: Vec<J> = prep
        .early
        .iter()
        .map(|i| {
            let r = &c.rows[*i];
            J::obj(vec![
                ("id", J::s(r.id.clone())),
                ("time_h", J::Num(r.time_h)),
                ("value", J::Num(r.value)),
                ("qualifier", J::s(r.qualifier.clone())),
            ])
        })
        .collect();
    let json = J::obj(vec![
        (
            "record_id",
            J::s(format!("r2.{}.{}", c.curve_id, method.letter())),
        ),
        ("definition_id", J::s(DEFINITION_ID)),
        ("curve_id", J::s(c.curve_id.clone())),
        ("method", J::s(method.letter().to_string())),
        ("method_name", J::s(method.name())),
        ("nuclide", J::s(c.nuclide.clone())),
        ("program", J::s(c.program.clone())),
        ("specimen_id", J::s(c.specimen_id.clone())),
        (
            "specimen_level",
            jopts(c.specimen_level.map(|l| {
                use strato_data::table::Vocabulary;
                l.as_str()
            })),
        ),
        (
            "source_aggregated",
            c.source_aggregated.map_or(J::Null, J::Bool),
        ),
        ("temperature_C", jnum(c.temperature_c)),
        ("temperature_label", J::s(c.temperature_label.clone())),
        (
            "temperature_history_id",
            jopts(c.temperature_history_id.as_deref()),
        ),
        ("release_definition", J::s(c.release_definition.clone())),
        ("denominator", J::s(c.denominator.clone())),
        ("time_basis", J::s(c.time_basis.clone())),
        ("model_config_id", J::s(c.model_config_id.clone())),
        ("independence", J::s(c.independence.clone())),
        ("source_citation_apa", J::s(c.source.apa.clone())),
        ("source_citation_note", jopts(c.source.note.as_deref())),
        ("source_link", J::s(c.source.link.clone())),
        ("link_status", J::s(c.source.link_status.clone())),
        ("source_location", J::s(c.source.location.clone())),
        ("row_ids", jstrs(&row_ids)),
        ("rows_excluded", jstrs(&excluded)),
        ("early_bound_points", J::Arr(early_points)),
        ("qualifiers", rows_of(&|r| J::s(r.qualifier.clone()))),
        (
            "bound_classes",
            rows_of(&|r| {
                use strato_data::table::Vocabulary;
                jopts(r.bound_class.map(|b| b.as_str()))
            }),
        ),
        ("register_status", rows_of(&|r| J::s(r.status.clone()))),
        (
            "digitisation_verification_ref",
            rows_of(&|r| jopts(r.digitisation_verification_ref.as_deref())),
        ),
        ("data_kind", J::s(c.data_kind.clone())),
        ("n", J::Int(row_ids.len() as i64)),
        ("matched_times_h", rows_of(&|r| J::Num(r.time_h))),
        ("model_times_h", jnums(&mt)),
        ("f_exp", rows_of(&|r| J::Num(r.value))),
        ("f_exp_registered", rows_of(&|r| J::Num(r.registered_value))),
        ("f_exp_registered_unit", J::s(c.registered_unit.clone())),
        ("f_exp_conversion", jopts(c.conversion.as_deref())),
        ("f_model", jnums(&fm)),
        ("f_model_t_minus", jnums(&fmm)),
        ("f_model_t_plus", jnums(&fmp)),
        (
            "model_run_ref",
            m.and_then(|m| m.run_ref.clone()).unwrap_or(J::Null),
        ),
        (
            "curve_assessment_ref",
            jopts(c.assessment_verification_ref.as_deref()),
        ),
        (
            "evaluation",
            J::Arr(keep.iter().map(|_| J::s("exact_step")).collect()),
        ),
        ("e_num", jnums(&en)),
        ("numerical_budget", jnum(budget_value)),
        ("mean_f_exp", jnum(sums.map(|s| s.mean))),
        ("ss_tot", jnum(sums.map(|s| s.ss_tot))),
        ("ss_res", jnum(sums.map(|s| s.ss_res))),
        ("r2", jnum(r2_value)),
        ("display", jopts(display.as_deref())),
        (
            "status",
            J::s(if available {
                "available"
            } else {
                "unavailable"
            }),
        ),
        ("reason_code", jopts(reason.map(ReasonCode::code))),
        ("reason_caption", jopts(reason.map(ReasonCode::caption))),
        (
            "failed_checks",
            J::Arr(failed.iter().map(|ch| J::s(ch.id())).collect()),
        ),
        (
            "coating_failure_basis",
            jopts(c.coating_failure_basis.as_deref()),
        ),
        (
            "decay_correction",
            J::s({
                use strato_data::table::Vocabulary;
                c.decay_correction.as_str()
            }),
        ),
        (
            "decay_reference",
            jopts(c.decay_reference.map(|d| match d {
                DecayReference::Common => "common",
                DecayReference::NotStated => "not_stated",
            })),
        ),
        (
            "inventory_reference",
            jopts(c.inventory_reference.map(|d| match d {
                InventoryReference::TestTimeOrigin => "test_time_origin",
                InventoryReference::Earlier => "earlier",
                InventoryReference::NotStated => "not_stated",
            })),
        ),
        (
            "inventory_reference_offset_h",
            jnum(c.inventory_reference_offset_h),
        ),
        ("counting_delay_h", rows_of(&|r| jnum(r.counting_delay_h))),
        ("t_d_h", jnums(&at(&t_d_h))),
        ("lambda_t_N", jnum(lambda_t_n)),
        ("decay_gate_delta", jnum(gate)),
        ("verification_ref", jopts(c.r2_verification_ref.as_deref())),
        ("label_latex", jopts(label_latex.as_deref())),
        ("label_plain", jopts(label_plain.as_deref())),
        (
            "premises",
            J::Arr(
                premises
                    .iter()
                    .map(|(k, t)| {
                        J::obj(vec![("code", J::s(k.clone())), ("text", J::s(t.clone()))])
                    })
                    .collect(),
            ),
        ),
        ("digitisation", digitisation),
        (
            "systematic_r2_interval",
            match &systematic {
                Some((lo, hi, _)) => J::Arr(vec![J::Num(*lo), J::Num(*hi)]),
                None => J::Null,
            },
        ),
        (
            "systematic_interval_display",
            match &systematic {
                Some((_, _, d)) => J::s(d.clone()),
                None => J::Null,
            },
        ),
        (
            "method_b_applicable",
            match (method, m) {
                (Method::B, Some(m)) => J::Bool(m.applicable),
                _ => J::Null,
            },
        ),
        (
            "method_b_mask_passed",
            match (method, m) {
                (Method::B, Some(m)) if m.applicable => J::Bool(m.mask_passed),
                _ => J::Null,
            },
        ),
    ]);
    R2Record {
        record_id: format!("r2.{}.{}", c.curve_id, method.letter()),
        curve_id: c.curve_id.clone(),
        method,
        available,
        first_failing: first,
        reason,
        failed_checks: failed,
        r2: r2_value,
        display,
        label_latex,
        label_plain,
        n: row_ids.len(),
        row_ids,
        rows_excluded: excluded,
        counted_rows: counted,
        premises,
        decay_gate_delta: gate,
        json,
    }
}

/// The two records of one curve. `a` and `b` are the model values of the methods, or `None` where
/// no model exists (check M0).
pub fn assess(c: &CurveInput, a: Option<&MethodInput>, b: Option<&MethodInput>) -> [R2Record; 2] {
    let prep = prepare(c);
    [
        one_record(c, &prep, a, Method::A),
        one_record(c, &prep, b, Method::B),
    ]
}

/// The class of every row of a curve, in the order of the rows.
pub fn classes_of(c: &CurveInput) -> Vec<PointClass> {
    prepare(c).classes
}

/// The reason lines of a curve (Section 7.4): none where both methods are available; one line
/// where both fail the same check; otherwise one line per unavailable method, prefixed by the
/// method where the outcomes differ, never for B1 and B2 whose sentences name Method B.
pub fn reason_lines(a: &R2Record, b: &R2Record) -> Vec<String> {
    let prefix = |m: Method, code: ReasonCode| -> String {
        if matches!(
            code,
            ReasonCode::MethodBNotApplicable | ReasonCode::MethodBMasked
        ) {
            code.caption().to_string()
        } else {
            format!(
                "For {}, {}",
                match m {
                    Method::A => "Method A",
                    Method::B => "Method B",
                },
                code.caption()
            )
        }
    };
    match (a.reason, b.reason) {
        (None, None) => vec![],
        (Some(x), Some(y)) if x == y => vec![x.caption().to_string()],
        (Some(x), Some(y)) => vec![prefix(Method::A, x), prefix(Method::B, y)],
        (Some(x), None) => vec![prefix(Method::A, x)],
        (None, Some(y)) => vec![prefix(Method::B, y)],
    }
}

/// The figure or table and printed page of a `source_location`, without file names or extraction
/// indices (`Fig. 4; printed p. 3 (PDF p. 3)` becomes `Fig. 4, p. 3`).
pub fn source_figure_table(location: &str) -> String {
    let head = location.split(';').next().unwrap_or(location).trim();
    let page = location
        .split("printed p. ")
        .nth(1)
        .map(|rest| {
            rest.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
        })
        .filter(|p| !p.is_empty());
    match page {
        Some(p) => format!("{head}, p. {p}"),
        None => head.to_string(),
    }
}

/// The caption record of one curve (Section 7.4). `drawn` says which methods have a curve on the
/// graph.
pub fn caption_json(
    c: &CurveInput,
    a: &R2Record,
    b: &R2Record,
    drawn: [bool; 2],
    source_note: Option<&str>,
) -> J {
    let source_kind = match c.data_kind.as_str() {
        "measured_tabulated" => "tabulated",
        "digitised" => "digitised",
        "calculated" => "calculated from registered values",
        _ => "tabulated and digitised",
    };
    let upper = a.failed_checks.contains(&Check::C1) || b.failed_checks.contains(&Check::C1);
    let legend_exp = if upper {
        format!("Experiment, upper bound ({source_kind})")
    } else {
        format!("Experiment ({source_kind})")
    };
    let caption_text = format!(
        "Cumulative release fraction of {} versus {}, {}, {}.",
        c.nuclide, c.words.time_axis_label, c.words.specimen_label, c.temperature_label
    );
    let mut methods = Vec::new();
    for (m, d) in [(Method::A, drawn[0]), (Method::B, drawn[1])] {
        if d {
            methods.push(J::obj(vec![
                ("label", J::s(m.name())),
                ("colour", J::s(m.colour())),
                ("style", J::s("solid")),
            ]));
        }
    }
    let unverified = c.source.link_status != "verified";
    J::obj(vec![
        ("curve_id", J::s(c.curve_id.clone())),
        ("caption_text", J::s(caption_text)),
        (
            "reason_lines",
            J::Arr(reason_lines(a, b).into_iter().map(J::s).collect()),
        ),
        ("source_citation_apa", J::s(c.source.apa.clone())),
        ("source_citation_note", jopts(c.source.note.as_deref())),
        ("source_link", J::s(c.source.link.clone())),
        (
            "source_link_note",
            match source_note {
                Some(n) => J::s(n),
                None if unverified => J::s(format!(
                    "no check of the link {} is recorded in the register",
                    c.source.link
                )),
                None => J::Null,
            },
        ),
        (
            "source_figure_table",
            J::s(source_figure_table(&c.source.location)),
        ),
        ("source_kind", J::s(source_kind)),
        ("time_axis_label", J::s(c.words.time_axis_label.clone())),
        ("specimen_label", J::s(c.words.specimen_label.clone())),
        ("temperature_label", J::s(c.temperature_label.clone())),
        ("y_axis_label", J::s("Cumulative release fraction")),
        ("y_axis_scale", J::s(c.words.y_axis_scale.clone())),
        ("legend_exp_label", J::s(legend_exp)),
        ("marker_colour_exp", J::s(COLOUR_EXP)),
        ("legend_methods", J::Arr(methods)),
    ])
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;

    /// A curve of six measured points of one single particle that passes every check of the
    /// definition. The numbers are illustrative; they never reach production output.
    pub fn good_curve() -> CurveInput {
        let t = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let f = [0.010, 0.030, 0.060, 0.100, 0.150, 0.210];
        let rows = t
            .iter()
            .zip(f)
            .enumerate()
            .map(|(k, (t, f))| RowInput {
                id: format!("E-{:03}", k + 1),
                time_h: *t,
                value: f,
                registered_value: f,
                qualifier: "~".into(),
                value_class: "R-dig".into(),
                bound_class: Some(BoundClass::NoBound),
                status: "digitised".into(),
                digitised: true,
                digitisation_verification_ref: Some("log:12.8".into()),
                u_value: Some(0.001),
                u_time_h: None,
                counting_delay_h: Some(0.0),
                duplicate_of: None,
                negative_presented_as_release: false,
                time_el_h: *t,
            })
            .collect();
        CurveInput {
            curve_id: "TEST-1".into(),
            nuclide: "Cs-134".into(),
            program: "test".into(),
            specimen_id: "particle 1".into(),
            specimen_level: Some(SpecimenLevel::SingleParticle),
            source_aggregated: Some(false),
            temperature_c: Some(1600.0),
            temperature_label: "1600 \u{b0}C".into(),
            temperature_history_id: Some("H".into()),
            release_definition: "cumulative release fraction".into(),
            denominator: "particle inventory".into(),
            time_basis: "time at temperature".into(),
            model_config_id: "cfg".into(),
            independence: "independent".into(),
            source: SourceInfo {
                apa: "Author, A. (2000). Title. Publisher.".into(),
                link: "https://doi.org/10.0000/x".into(),
                link_status: "verified".into(),
                location: "Fig. 2; printed p. 12 (PDF p. 14)".into(),
                note: None,
            },
            data_kind: "digitised".into(),
            registered_unit: "1".into(),
            conversion: None,
            rows,
            coating_failure_basis: Some("none".into()),
            register_marks_failure: false,
            matrix_separated_by_source: false,
            decay_correction: DecayCorrection::Corrected,
            decay_reference: Some(DecayReference::Common),
            inventory_reference: Some(InventoryReference::TestTimeOrigin),
            inventory_reference_offset_h: None,
            lambda_per_h: Some(1.0e-4),
            assessment_verification_ref: Some("log:assessment".into()),
            r2_verification_ref: Some("log:r2".into()),
            nuclide_configurable: true,
            denominator_established: true,
            definition_matches: true,
            history_covers: true,
            history_rows_verified: true,
            missing_conditions: vec![],
            premises: vec![],
            u_systematic: None,
            u_value_summary: Some(("abs".into(), 0.001)),
            u_time_summary_h: None,
            words: CaptionWords {
                time_axis_label: "time at temperature (h)".into(),
                specimen_label: "particle 1".into(),
                y_axis_scale: "linear".into(),
            },
        }
    }

    /// Model values close to the experiment, resolved and verified.
    pub fn good_method(c: &CurveInput, method: Method) -> MethodInput {
        let n = c.rows.len();
        let mut m = MethodInput::empty(method, n);
        for (i, r) in c.rows.iter().enumerate() {
            m.model_times_h[i] = Some(r.time_h);
            m.f_model[i] = Some(r.value * 1.03);
            m.e_num[i] = Some(1.0e-7);
            m.resolved[i] = true;
        }
        m.reaches_last_point = true;
        m.validity_ok = true;
        m.applicable = true;
        m.mask_passed = true;
        m.model_rows_verified = true;
        m
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;

    fn both(c: &CurveInput) -> [R2Record; 2] {
        let a = good_method(c, Method::A);
        let b = good_method(c, Method::B);
        assess(c, Some(&a), Some(&b))
    }

    fn failing(c: &CurveInput) -> Vec<&'static str> {
        both(c)[0].failed_checks.iter().map(|x| x.id()).collect()
    }

    #[test]
    fn the_fixture_curve_passes_every_check_and_gets_a_value() {
        let c = good_curve();
        let [a, b] = both(&c);
        assert!(a.available && b.available, "{:?}", a.failed_checks);
        assert!(a.failed_checks.is_empty() && a.reason.is_none());
        let v = a.r2.expect("a value");
        assert!(v < 1.0 && v > 0.9);
        assert!(
            a.display.is_some()
                && a.label_plain
                    .as_deref()
                    .unwrap()
                    .starts_with("R\u{b2}_A = ")
        );
        assert!(b
            .label_latex
            .as_deref()
            .unwrap()
            .starts_with("R^{2}_{\\mathrm{B}} = "));
        assert!(a.json.render().contains("\"status\": \"available\""));
        // The same points, hence the same SS_tot, for both methods.
        assert_eq!(
            a.json.render().matches("\"ss_tot\"").count(),
            b.json.render().matches("\"ss_tot\"").count()
        );
    }

    // Each check on a constructed curve that fails exactly that check.

    #[test]
    fn m0_a_nuclide_without_a_diffusivity_row_has_no_model() {
        let mut c = good_curve();
        c.nuclide_configurable = false;
        let [a, b] = assess(&c, None, None);
        assert_eq!(a.failed_checks.first(), Some(&Check::M0));
        assert_eq!(a.reason, Some(ReasonCode::ModelNotConfigurable));
        assert!(!a.available && !b.available);
        assert!(a.r2.is_none() && a.label_plain.is_none());
        let j = caption_json(&c, &a, &b, [false, false], None).render();
        assert!(j.contains("\"legend_methods\": []"), "{j}");
    }

    #[test]
    fn c1_a_compact_total_curve_is_upper_bounds_never_too_few_points() {
        let mut c = good_curve();
        for r in &mut c.rows {
            r.bound_class = Some(BoundClass::CompactTotal);
        }
        c.rows.truncate(3);
        let f = both(&c);
        assert_eq!(f[0].reason, Some(ReasonCode::UpperBound));
        assert_eq!(f[0].first_failing, Some(Check::C1));
        assert!(f[0].failed_checks.contains(&Check::C7));
        assert!(f[0].rows_excluded.is_empty());
        let cap = caption_json(&c, &f[0], &f[1], [true, false], None).render();
        assert!(cap.contains("Experiment, upper bound (digitised)"));
    }

    #[test]
    fn c1_a_curve_of_window_bounds_is_upper_bounds() {
        let mut c = good_curve();
        for r in &mut c.rows {
            r.bound_class = Some(BoundClass::WindowUpperBound);
        }
        assert_eq!(both(&c)[0].reason, Some(ReasonCode::UpperBound));
        assert!(both(&c)[0].rows_excluded.is_empty());
    }

    #[test]
    fn c1_early_bounds_are_excluded_only_before_the_first_measured_point() {
        // Two detection limits followed by five measured points: N = 5, C1 passes.
        let mut c = good_curve();
        let extra = c.rows[0].clone();
        c.rows.truncate(5);
        for (k, (q, id)) in [("<", "E-100"), ("<", "E-101")].into_iter().enumerate() {
            let mut r = extra.clone();
            r.id = id.into();
            r.qualifier = q.into();
            r.value_class = "M-ND".into();
            r.time_h = -2.0 + k as f64;
            r.time_el_h = r.time_h;
            c.rows.insert(k, r);
        }
        let rec = &both(&c)[0];
        assert_eq!(rec.n, 5, "{:?}", rec.failed_checks);
        assert!(rec.available, "{:?}", rec.failed_checks);
        assert_eq!(
            rec.rows_excluded,
            vec![
                "E-100:early_bound".to_string(),
                "E-101:early_bound".to_string()
            ]
        );
        assert!(rec.premises.iter().any(|(k, _)| k == "early_bound"));
        assert!(rec.json.render().contains("\"early_bound_points\": [\n"));
        // The same two limits followed by four measured points: too few points.
        let mut c4 = c.clone();
        c4.rows.pop();
        let rec4 = &both(&c4)[0];
        assert_eq!(rec4.reason, Some(ReasonCode::TooFewPoints));
        // A detection limit after the first measured point is not excluded: upper bound.
        let mut c3 = good_curve();
        c3.rows[3].qualifier = "<".into();
        c3.rows[3].value_class = "M-ND".into();
        assert_eq!(both(&c3)[0].reason, Some(ReasonCode::UpperBound));
        // Five detection limits and no measured point: upper bound.
        let mut c5 = good_curve();
        c5.rows.truncate(5);
        for r in &mut c5.rows {
            r.qualifier = "<".into();
        }
        assert_eq!(both(&c5)[0].reason, Some(ReasonCode::UpperBound));
    }

    #[test]
    fn c1_point_class_precedence_follows_decision_74() {
        let mut r = good_curve().rows[0].clone();
        // Unclassified first, even for a detection limit.
        r.bound_class = None;
        r.qualifier = "<".into();
        assert_eq!(classify(&r), PointClass::Unclassified);
        // A compact total with a `<` qualifier and an M-ND class is a compact-total bound.
        r.bound_class = Some(BoundClass::CompactTotal);
        r.value_class = "M-ND".into();
        assert_eq!(classify(&r), PointClass::CompactTotalBound);
        r.bound_class = Some(BoundClass::WindowUpperBound);
        assert_eq!(classify(&r), PointClass::WindowBound);
        // A detection limit that is also an upper bound is a detection bound.
        r.bound_class = Some(BoundClass::UpperBound);
        r.value_class = "M-UB".into();
        r.qualifier = "<".into();
        assert_eq!(classify(&r), PointClass::DetectionBound);
        r.qualifier = "=".into();
        assert_eq!(classify(&r), PointClass::OtherUpperBound);
        r.bound_class = Some(BoundClass::NoBound);
        r.value_class = "R-dig".into();
        assert_eq!(classify(&r), PointClass::Measured);
    }

    #[test]
    fn an_unset_assessment_field_gives_missing_conditions_with_its_neutral_caption() {
        let mut c = good_curve();
        c.specimen_level = None;
        assert_eq!(failing(&c), vec!["C6"]);
        assert_eq!(both(&c)[0].reason, Some(ReasonCode::MissingConditions));
        let caption = ReasonCode::MissingConditions.caption();
        assert!(!caption.contains("compact") && !caption.contains("bound"));
        // An unclassified row is not read as an upper bound.
        let mut c = good_curve();
        c.rows[2].bound_class = None;
        assert_eq!(both(&c)[0].failed_checks, vec![Check::C6]);
        // Unset basis of a coating failure, source_aggregated, and an unset decay field.
        let mut c = good_curve();
        c.coating_failure_basis = None;
        assert_eq!(failing(&c), vec!["C6"]);
        let mut c = good_curve();
        c.source_aggregated = None;
        assert_eq!(failing(&c), vec!["C6"]);
        let mut c = good_curve();
        c.decay_reference = None;
        assert_eq!(failing(&c), vec!["C6"]);
    }

    #[test]
    fn c2_coating_failure_known_or_suspected_on_a_basis_other_than_the_model() {
        let mut c = good_curve();
        c.coating_failure_basis = Some("five failed particles in the specimen".into());
        assert_eq!(failing(&c), vec!["C2"]);
        assert_eq!(both(&c)[0].reason, Some(ReasonCode::CoatingFailure));
        let mut c = good_curve();
        c.register_marks_failure = true;
        assert_eq!(failing(&c), vec!["C2"]);
    }

    #[test]
    fn c3_a_compact_or_sphere_is_refused_unless_the_source_separates_the_matrix() {
        for lvl in [SpecimenLevel::Compact, SpecimenLevel::Sphere] {
            let mut c = good_curve();
            c.specimen_level = Some(lvl);
            assert_eq!(failing(&c), vec!["C3"]);
            assert_eq!(both(&c)[0].reason, Some(ReasonCode::CompactNotRelatable));
            // An aggregated curve of a compact still runs C3.
            c.source_aggregated = Some(true);
            assert_eq!(failing(&c), vec!["C3"]);
            c.matrix_separated_by_source = true;
            assert!(failing(&c).is_empty());
        }
        // Loose particles without matrix pass.
        let mut c = good_curve();
        c.specimen_level = Some(SpecimenLevel::ParticleBatch);
        assert!(failing(&c).is_empty());
    }

    #[test]
    fn c4_the_definition_the_denominator_and_negative_values() {
        let mut c = good_curve();
        c.denominator_established = false;
        assert_eq!(failing(&c), vec!["C4"]);
        assert_eq!(both(&c)[0].reason, Some(ReasonCode::IncompatibleDefinition));
        let mut c = good_curve();
        c.definition_matches = false;
        assert_eq!(failing(&c), vec!["C4"]);
        // A negative value that the source does not present as a release fraction.
        let mut c = good_curve();
        c.rows[0].value = -0.002;
        c.rows[0].registered_value = -0.002;
        assert!(failing(&c).contains(&"C4"));
        // Presented as a release fraction, with a stated uncertainty smaller than its size.
        c.rows[0].negative_presented_as_release = true;
        c.rows[0].u_value = Some(0.001);
        assert!(!failing(&c).contains(&"C4"));
    }

    #[test]
    fn c5_a_missing_temperature_history() {
        let mut c = good_curve();
        c.history_covers = false;
        assert_eq!(failing(&c), vec!["C5"]);
        assert_eq!(
            both(&c)[0].reason,
            Some(ReasonCode::MissingTemperatureHistory)
        );
    }

    #[test]
    fn c6_a_condition_the_source_does_not_state() {
        let mut c = good_curve();
        c.missing_conditions = vec!["time unit".into()];
        assert_eq!(failing(&c), vec!["C6"]);
        assert_eq!(both(&c)[0].reason, Some(ReasonCode::MissingConditions));
    }

    #[test]
    fn d1_the_decay_rules() {
        // Not corrected: no comparison is defined.
        let mut c = good_curve();
        c.decay_correction = DecayCorrection::NotCorrected;
        assert_eq!(failing(&c), vec!["D1"]);
        assert_eq!(both(&c)[0].reason, Some(ReasonCode::DecayNotComparable));
        // Corrected, but the common reference is not stated.
        let mut c = good_curve();
        c.decay_reference = Some(DecayReference::NotStated);
        assert_eq!(failing(&c), vec!["D1"]);
        // Not stated, inventory referred to the time origin, delays stated: the gate decides.
        let mut c = good_curve();
        c.decay_correction = DecayCorrection::NotStated;
        c.decay_reference = None;
        c.inventory_reference = Some(InventoryReference::TestTimeOrigin);
        c.lambda_per_h = Some(1.0e-7);
        let rec = both(&c);
        assert!(rec[0].available, "{:?}", rec[0].failed_checks);
        assert!(rec[0].decay_gate_delta.is_some());
        assert!(rec[0]
            .premises
            .iter()
            .any(|(_, t)| t.contains("decay not modelled")));
        // A null counting delay fails D1.
        c.rows[3].counting_delay_h = None;
        assert_eq!(failing(&c), vec!["D1"]);
    }

    #[test]
    fn d1_an_end_of_irradiation_inventory_with_an_unknown_storage_time_is_refused() {
        let mut c = good_curve();
        c.decay_correction = DecayCorrection::NotStated;
        c.decay_reference = None;
        c.inventory_reference = Some(InventoryReference::Earlier);
        c.inventory_reference_offset_h = None;
        let f = both(&c);
        assert_eq!(f[0].reason, Some(ReasonCode::DecayNotComparable));
        assert_eq!(f[0].first_failing, Some(Check::D1));
        assert!(f[0].r2.is_none());
        // With the storage time registered, the bound exists and the gate is evaluated.
        c.inventory_reference_offset_h = Some(250.0 * 24.0);
        c.lambda_per_h = Some(1.156e-4);
        let f = both(&c);
        assert!(!f[0].failed_checks.contains(&Check::D1));
        // lambda t_d is about 0.7: the gate fails, as the definition says (Ag-110m, 250 d).
        assert!(
            f[0].failed_checks.contains(&Check::D2),
            "{:?}",
            f[0].failed_checks
        );
        assert_eq!(f[0].reason, Some(ReasonCode::DecayNotComparable));
        assert!(f[0].decay_gate_delta.unwrap() > BUDGET_MAX);
    }

    #[test]
    fn the_genuine_single_particle_decay_corrected_series_passes() {
        let mut c = good_curve();
        c.specimen_level = Some(SpecimenLevel::SingleParticle);
        c.decay_correction = DecayCorrection::Corrected;
        c.decay_reference = Some(DecayReference::Common);
        let f = both(&c);
        assert!(f[0].available && f[1].available);
        assert!(f[0].r2.is_some() && f[0].decay_gate_delta.is_none());
        assert!(f[0].json.render().contains("\"decay_gate_delta\": null"));
    }

    #[test]
    fn c7_fewer_than_five_points() {
        let mut c = good_curve();
        c.rows.truncate(4);
        assert_eq!(failing(&c), vec!["C7"]);
        assert_eq!(both(&c)[0].reason, Some(ReasonCode::TooFewPoints));
    }

    #[test]
    fn c8_zero_variance() {
        let mut c = good_curve();
        let v = [0.300, 0.301, 0.300, 0.302, 0.301, 0.300];
        for (r, x) in c.rows.iter_mut().zip(v) {
            r.value = x;
            r.registered_value = x;
            r.u_value = Some(0.01 * x);
        }
        assert_eq!(failing(&c), vec!["C8"]);
        let f = both(&c);
        assert_eq!(f[0].reason, Some(ReasonCode::ZeroVariance));
        assert!(f[0].r2.is_none());
    }

    #[test]
    fn m1_a_diffusivity_row_outside_its_validity() {
        let c = good_curve();
        let mut a = good_method(&c, Method::A);
        a.validity_ok = false;
        let b = good_method(&c, Method::B);
        let f = assess(&c, Some(&a), Some(&b));
        assert_eq!(f[0].failed_checks, vec![Check::M1]);
        assert_eq!(f[0].reason, Some(ReasonCode::OutsideCorrelationValidity));
        assert!(f[1].available, "the other method is not affected");
        let lines = reason_lines(&f[0], &f[1]);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("For Method A, "));
    }

    #[test]
    fn m2_an_unresolved_point_empties_the_record_and_is_never_dropped() {
        let c = good_curve();
        let mut a = good_method(&c, Method::A);
        a.resolved[2] = false;
        let f = assess(&c, Some(&a), None);
        assert_eq!(f[0].failed_checks, vec![Check::M2]);
        assert_eq!(f[0].n, 6, "every row stays in the record");
        // A budget above the limit fails the same check.
        let mut a = good_method(&c, Method::A);
        a.e_num.fill(Some(0.01));
        assert_eq!(assess(&c, Some(&a), None)[0].failed_checks, vec![Check::M2]);
        // A run that does not reach the last point.
        let mut a = good_method(&c, Method::A);
        a.reaches_last_point = false;
        assert_eq!(assess(&c, Some(&a), None)[0].failed_checks, vec![Check::M2]);
    }

    #[test]
    fn b1_and_b2_name_method_b_and_leave_method_a_alone() {
        let c = good_curve();
        let a = good_method(&c, Method::A);
        let mut b = good_method(&c, Method::B);
        b.applicable = false;
        let f = assess(&c, Some(&a), Some(&b));
        assert!(f[0].available);
        assert_eq!(f[1].failed_checks, vec![Check::B1]);
        assert_eq!(f[1].reason, Some(ReasonCode::MethodBNotApplicable));
        assert_eq!(f[1].decay_gate_delta, None);
        assert_eq!(
            reason_lines(&f[0], &f[1]),
            vec![ReasonCode::MethodBNotApplicable.caption().to_string()]
        );
        let mut b = good_method(&c, Method::B);
        b.mask_passed = false;
        let f = assess(&c, Some(&a), Some(&b));
        assert_eq!(f[1].failed_checks, vec![Check::B2]);
        assert_eq!(f[1].reason, Some(ReasonCode::MethodBMasked));
    }

    #[test]
    fn d2_the_decay_gate_is_reached_only_after_the_model_checks() {
        let mut c = good_curve();
        c.decay_correction = DecayCorrection::NotStated;
        c.decay_reference = None;
        c.inventory_reference = Some(InventoryReference::Earlier);
        c.inventory_reference_offset_h = Some(6000.0);
        c.lambda_per_h = Some(1.0e-4);
        let a = good_method(&c, Method::A);
        let mut b = good_method(&c, Method::B);
        b.applicable = false;
        let f = assess(&c, Some(&a), Some(&b));
        assert_eq!(f[0].failed_checks, vec![Check::D2]);
        assert!(f[0].decay_gate_delta.is_some());
        assert_eq!(f[1].failed_checks, vec![Check::B1]);
        assert!(f[1].decay_gate_delta.is_none());
    }

    #[test]
    fn c0_a_row_that_is_not_verified() {
        let mut c = good_curve();
        c.rows[1].status = "NOT VERIFIED".into();
        assert_eq!(failing(&c), vec!["C0"]);
        assert_eq!(both(&c)[0].reason, Some(ReasonCode::InputsNotVerified));
        // A digitised row without a recorded coordinate verification.
        let mut c = good_curve();
        c.rows[1].digitisation_verification_ref = None;
        assert_eq!(failing(&c), vec!["C0"]);
        // No recorded verification of the curve assessment, or an unverified model row.
        let mut c = good_curve();
        c.assessment_verification_ref = None;
        assert_eq!(failing(&c), vec!["C0"]);
        let c = good_curve();
        let mut a = good_method(&c, Method::A);
        a.model_rows_verified = false;
        assert_eq!(assess(&c, Some(&a), None)[0].failed_checks, vec![Check::C0]);
    }

    #[test]
    fn v1_a_computed_value_without_a_recorded_recomputation_is_not_shown() {
        let mut c = good_curve();
        c.r2_verification_ref = None;
        let f = both(&c);
        assert_eq!(f[0].failed_checks, vec![Check::V1]);
        assert_eq!(f[0].reason, Some(ReasonCode::NotIndependentlyVerified));
        assert!(f[0].r2.is_none() && f[0].display.is_none() && f[0].label_plain.is_none());
        // The sums are formed but the value is stored for an available record only.
        let j = f[0].json.render();
        assert!(j.contains("\"r2\": null") && !j.contains("\"ss_tot\": null"));
    }

    #[test]
    fn the_first_failing_check_follows_the_order_and_every_failure_is_listed() {
        // C1 and C0 both fail: the reason is upper_bound, both are listed.
        let mut c = good_curve();
        for r in &mut c.rows {
            r.bound_class = Some(BoundClass::CompactTotal);
        }
        c.assessment_verification_ref = None;
        let f = &both(&c)[0];
        assert_eq!(f.reason, Some(ReasonCode::UpperBound));
        assert!(f.failed_checks.contains(&Check::C1) && f.failed_checks.contains(&Check::C0));
        // C2 outranks C3 and C4.
        let mut c = good_curve();
        c.coating_failure_basis = Some("failed".into());
        c.specimen_level = Some(SpecimenLevel::Compact);
        c.denominator_established = false;
        assert_eq!(both(&c)[0].reason, Some(ReasonCode::CoatingFailure));
        assert_eq!(
            both(&c)[0].failed_checks,
            vec![Check::C2, Check::C3, Check::C4]
        );
    }

    #[test]
    fn the_caption_record_has_the_fields_of_section_7_4() {
        let c = good_curve();
        let [a, b] = both(&c);
        let j = caption_json(&c, &a, &b, [true, true], None).render();
        assert!(j.contains(
            "\"caption_text\": \"Cumulative release fraction of Cs-134 versus time at temperature (h), particle 1, 1600 \u{b0}C.\""
        ));
        assert!(j.contains("\"reason_lines\": []"));
        assert!(j.contains("\"source_figure_table\": \"Fig. 2, p. 12\""));
        assert!(j.contains("\"legend_exp_label\": \"Experiment (digitised)\""));
        assert!(j.contains("\"marker_colour_exp\": \"#111111\""));
        assert!(j.contains("\"colour\": \"#2563EB\"") && j.contains("\"colour\": \"#DC2626\""));
        assert_eq!(
            source_figure_table("Table 4-9; printed p. 164 (0-based PDF index 170)"),
            "Table 4-9, p. 164"
        );
        assert_eq!(
            source_figure_table("Fig. 4-7 left; printed p. 137 (PDF p. 144)"),
            "Fig. 4-7 left, p. 137"
        );
    }

    #[test]
    fn the_interval_and_the_label_are_in_the_record_of_an_available_curve() {
        let c = good_curve();
        let f = both(&c);
        let j = f[0].json.render();
        assert!(j.contains("\"status\": \"computed\""), "{j}");
        assert!(j.contains("R\u{b2} reading-uncertainty interval ["));
        assert!(j.contains("(reading uncertainty only)"));
    }
}
