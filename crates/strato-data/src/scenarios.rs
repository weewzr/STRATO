//! The scenario definitions of `data/scenarios/` (science design Section 5, decision 58): typed
//! reader and consistency checks.
//!
//! One CSV file per scenario holds its histories as breakpoint tables, its step plan and its
//! metadata. Every history row carries its source row ids or the label
//! "prescribed input (assumed)" with a reason; the checks resolve every id against the registers.
//! A scenario is a prescribed-input simulation, never a validated reactor prediction.

use std::collections::HashSet;
use std::path::Path;

use crate::checks::Report;
use crate::decay::Stability;
use crate::error::{DataError, Result};
use crate::irradiation::SeriesTarget;
use crate::register::Register;
use crate::table::{Printed, Row, Table};
use crate::yields::Parent;

/// The five files, in the order of the scenarios.
pub const FILES: [&str; 5] = [
    "scenarios/s1_normal_operation.csv",
    "scenarios/s2_shutdown_cooldown.csv",
    "scenarios/s3_startup.csv",
    "scenarios/s4_control_rod_insertion.csv",
    "scenarios/s5_fuel_removal.csv",
];

/// The scenario ids of [`FILES`].
pub const IDS: [&str; 5] = ["S1", "S2", "S3", "S4", "S5"];

/// The columns of every scenario file.
pub const HEADER: [&str; 12] = [
    "row_id",
    "record",
    "variant",
    "quantity",
    "t_value",
    "t_unit",
    "value",
    "unit",
    "interpolation",
    "source_ids",
    "basis",
    "text",
];

/// The label of a number that no register row gives.
pub const ASSUMED: &str = "prescribed input (assumed)";
/// The label of a number derived from published AGR-1 burnup.
pub const DERIVED: &str = "derived from published AGR-1 burnup (calculated)";
/// The label of a number taken from a registered row.
pub const REGISTERED: &str = "registered source row";
/// The label of a numerical setting.
pub const NUMERICAL: &str = "numerical setting (chosen; refinement-checked per Section 3.5)";
/// The variant that carries the exploration at the registered compact peak temperature.
pub const EXPLORATION_VARIANT: &str = "exploration-1197C";
/// The reference time-average peak temperature of the exploration variant, degC (IRR-097).
pub const EXPLORATION_TEMPERATURE_C: f64 = 1197.0;
/// The nuclides for which generation is meaningful (decision 15 and science design 5.1).
pub const MEANINGFUL: [&str; 2] = ["Sr-90", "Cs-137"];
/// The nuclides that get no direct fission-yield source term (decision 15).
pub const NOT_MEANINGFUL: [&str; 3] = ["Ag-110m", "Cs-134", "Eu-154"];

/// Kind of a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Record {
    /// Metadata or a single stated number.
    Meta,
    /// A breakpoint of a history.
    History,
    /// A segment of the step plan.
    Step,
}

/// One row of a scenario file.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioRow {
    /// Line in the file.
    pub line: usize,
    /// Row id.
    pub row_id: String,
    /// Kind.
    pub record: Record,
    /// Variant id (empty for the base scenario).
    pub variant: String,
    /// Quantity or metadata key.
    pub quantity: String,
    /// Time or segment end, with its printed text.
    pub t_value: Option<Printed>,
    /// Unit of the time.
    pub t_unit: String,
    /// The value, with its printed text.
    pub value: Option<Printed>,
    /// Unit of the value.
    pub unit: String,
    /// `linear` or `step` (history rows).
    pub interpolation: String,
    /// Register row ids the number comes from.
    pub source_ids: Vec<String>,
    /// How the number is labelled.
    pub basis: String,
    /// Text: the reason or the statement.
    pub text: String,
}

/// One scenario file.
#[derive(Debug, Clone)]
pub struct ScenarioFile {
    /// `S1` to `S5`.
    pub id: String,
    /// File path below `data/`.
    pub file: &'static str,
    /// The rows in file order.
    pub rows: Vec<ScenarioRow>,
}

/// The five scenario texts.
#[derive(Debug, Clone)]
pub struct ScenarioTexts {
    /// Text of each file, in the order of [`FILES`].
    pub files: Vec<String>,
}

impl ScenarioTexts {
    /// Read the five files below `data_dir`.
    ///
    /// # Errors
    ///
    /// A file that cannot be read.
    pub fn read(data_dir: &Path) -> Result<ScenarioTexts> {
        let mut files = Vec::new();
        for f in FILES {
            let text = std::fs::read_to_string(data_dir.join(f))
                .map_err(|e| DataError::file(f, format!("cannot read the file: {e}")))?;
            files.push(text);
        }
        Ok(ScenarioTexts { files })
    }
}

/// Seconds in a unit of time (`s`, `h`, `d`).
///
/// # Errors
///
/// An unknown unit.
pub fn unit_seconds(unit: &str) -> std::result::Result<f64, String> {
    match unit {
        "s" => Ok(1.0),
        "h" => Ok(3600.0),
        "d" => Ok(86_400.0),
        other => Err(format!("`{other}` is not a time unit (s, h or d)")),
    }
}

fn split_ids(cell: &str) -> Vec<String> {
    cell.split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn read_row(row: Row<'_>) -> Result<ScenarioRow> {
    let record = match row.required("record")? {
        "meta" => Record::Meta,
        "history" => Record::History,
        "step" => Record::Step,
        other => return Err(row.err(Some("record"), format!("`{other}` is not meta, history or step"))),
    };
    Ok(ScenarioRow {
        line: row.line(),
        row_id: row.required("row_id")?.to_string(),
        record,
        variant: row.text("variant")?.to_string(),
        quantity: row.required("quantity")?.to_string(),
        t_value: row.opt_number("t_value")?,
        t_unit: row.text("t_unit")?.to_string(),
        value: row.opt_number("value")?,
        unit: row.text("unit")?.to_string(),
        interpolation: row.text("interpolation")?.to_string(),
        source_ids: split_ids(row.text("source_ids")?),
        basis: row.text("basis")?.to_string(),
        text: row.text("text")?.to_string(),
    })
}

/// All five scenarios.
#[derive(Debug, Clone)]
pub struct Scenarios {
    /// The files in the order S1 to S5.
    pub files: Vec<ScenarioFile>,
}

impl Scenarios {
    /// Type the five texts.
    ///
    /// # Errors
    ///
    /// A file whose header differs from [`HEADER`], a row that is not rectangular, a number
    /// cell that is not a number, or an unknown record kind.
    pub fn parse(texts: &ScenarioTexts) -> Result<Scenarios> {
        let mut files = Vec::new();
        for (k, text) in texts.files.iter().enumerate() {
            let table = Table::parse(FILES[k], text, &HEADER)?;
            let mut rows = Vec::new();
            for r in table.rows() {
                rows.push(read_row(r)?);
            }
            files.push(ScenarioFile {
                id: IDS[k].to_string(),
                file: FILES[k],
                rows,
            });
        }
        Ok(Scenarios { files })
    }

    /// The scenario with an id (`S1` to `S5`).
    pub fn get(&self, id: &str) -> Option<&ScenarioFile> {
        self.files.iter().find(|f| f.id == id)
    }
}

impl ScenarioFile {
    /// The ids of the variants of this scenario (not the base).
    pub fn variants(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for r in &self.rows {
            if !r.variant.is_empty() && !v.contains(&r.variant) {
                v.push(r.variant.clone());
            }
        }
        v
    }

    /// The metadata row `key` for a variant: the variant's own row if it has one, else the base
    /// row.
    pub fn meta(&self, variant: &str, key: &str) -> Option<&ScenarioRow> {
        let own = self
            .rows
            .iter()
            .find(|r| r.record == Record::Meta && r.quantity == key && r.variant == variant);
        own.or_else(|| {
            self.rows
                .iter()
                .find(|r| r.record == Record::Meta && r.quantity == key && r.variant.is_empty())
        })
    }

    /// The rows of one history quantity for a variant. The rows of the variant replace all base
    /// rows of the same quantity. The quantity `fission_rate_density` of the variant `cycles`
    /// replaces the base `G_kernel_factor`.
    pub fn history(&self, variant: &str, quantity: &str) -> Vec<&ScenarioRow> {
        let pick = |v: &str, q: &str| -> Vec<&ScenarioRow> {
            self.rows
                .iter()
                .filter(|r| r.record == Record::History && r.quantity == q && r.variant == v)
                .collect()
        };
        if !variant.is_empty() {
            let own = pick(variant, quantity);
            if !own.is_empty() {
                return own;
            }
            if quantity == "G_kernel_factor" {
                let rate = pick(variant, "fission_rate_density");
                if !rate.is_empty() {
                    return rate;
                }
            }
        }
        pick("", quantity)
    }

    /// The step-plan rows for a variant (the variant's own rows replace the base rows).
    pub fn steps(&self, variant: &str) -> Vec<&ScenarioRow> {
        let pick = |v: &str| -> Vec<&ScenarioRow> {
            self.rows
                .iter()
                .filter(|r| r.record == Record::Step && r.variant == v)
                .collect()
        };
        if !variant.is_empty() {
            let own = pick(variant);
            if !own.is_empty() {
                return own;
            }
        }
        pick("")
    }
}

impl ScenarioRow {
    /// `t_value` in seconds.
    ///
    /// # Errors
    ///
    /// A missing time or an unknown unit.
    pub fn t_seconds(&self) -> std::result::Result<f64, String> {
        let t = self
            .t_value
            .as_ref()
            .ok_or_else(|| format!("{}: no time", self.row_id))?;
        Ok(t.value * unit_seconds(&self.t_unit).map_err(|e| format!("{}: {e}", self.row_id))?)
    }
}

/// Every row id of every register file.
fn register_ids(reg: &Register) -> HashSet<&str> {
    let mut ids: HashSet<&str> = HashSet::new();
    ids.extend(reg.irradiation.conditions.iter().map(|r| r.id.as_str()));
    ids.extend(reg.irradiation.burnup.iter().map(|r| r.id.as_str()));
    ids.extend(reg.irradiation.power.iter().map(|r| r.id.as_str()));
    ids.extend(reg.temperature.rows.iter().map(|r| r.id.as_str()));
    ids.extend(reg.geometry.rows.iter().map(|r| r.id.as_str()));
    ids.extend(reg.arrhenius.rows.iter().map(|r| r.id.as_str()));
    ids.extend(reg.release.rows.iter().map(|r| r.id.as_str()));
    ids
}

fn condition_number(reg: &Register, id: &str) -> Option<f64> {
    use crate::irradiation::ConditionValue;
    reg.irradiation
        .conditions
        .iter()
        .find(|r| r.id == id)
        .and_then(|r| match &r.value {
            ConditionValue::Number(p) => Some(p.value),
            ConditionValue::Text(_) => None,
        })
}

/// The check results of the scenario files against the registers.
pub fn run_checks(reg: &Register, sc: &Scenarios) -> Report {
    let mut rep = Report::default();
    let ids = register_ids(reg);
    let verdict = |rep: &mut Report, id: &str, desc: &str, bad: Vec<String>, n: usize| {
        let detail = if bad.is_empty() {
            format!("{n} items")
        } else {
            bad.join("; ")
        };
        rep.add(format!("SC.{id}"), bad.is_empty(), desc, detail);
    };

    // Row ids: unique, prefixed with the scenario id.
    let mut seen: HashSet<String> = HashSet::new();
    let mut bad = Vec::new();
    for f in &sc.files {
        for r in &f.rows {
            if !r.row_id.starts_with(&format!("{}-", f.id)) {
                bad.push(format!("{} does not carry the id {}", r.row_id, f.id));
            }
            if !seen.insert(r.row_id.clone()) {
                bad.push(format!("{} is not unique", r.row_id));
            }
        }
    }
    verdict(
        &mut rep,
        "ids",
        "row ids are unique and carry the id of their scenario",
        bad,
        seen.len(),
    );

    // Units and vocabularies.
    let mut bad = Vec::new();
    let mut n = 0;
    for f in &sc.files {
        for r in f.rows.iter().filter(|r| r.record != Record::Meta) {
            n += 1;
            if unit_seconds(&r.t_unit).is_err() {
                bad.push(format!("{}: time unit `{}`", r.row_id, r.t_unit));
            }
            if r.t_value.is_none() {
                bad.push(format!("{}: no time", r.row_id));
            }
            match (r.record, r.quantity.as_str()) {
                (Record::History, "G_kernel_factor") => {
                    if r.unit != "1" {
                        bad.push(format!("{}: unit `{}` (expected 1)", r.row_id, r.unit));
                    }
                }
                (Record::History, "T_C") => {
                    if r.unit != "degC" {
                        bad.push(format!("{}: unit `{}` (expected degC)", r.row_id, r.unit));
                    }
                }
                (Record::History, "fission_rate_density") => {
                    if r.unit != "1e19 m^-3 s^-1" {
                        bad.push(format!("{}: unit `{}`", r.row_id, r.unit));
                    }
                }
                (Record::Step, "dt_max") => {
                    if unit_seconds(&r.unit).is_err() {
                        bad.push(format!("{}: step unit `{}`", r.row_id, r.unit));
                    }
                }
                _ => bad.push(format!("{}: unknown quantity `{}`", r.row_id, r.quantity)),
            }
            if r.record == Record::History {
                if r.interpolation != "linear" && r.interpolation != "step" {
                    bad.push(format!("{}: interpolation `{}`", r.row_id, r.interpolation));
                }
                match r.value.as_ref().map(|v| v.value) {
                    Some(v) if v.is_finite() => {
                        if r.quantity == "G_kernel_factor" && v < 0.0 {
                            bad.push(format!("{}: negative generation factor", r.row_id));
                        }
                        if r.quantity == "T_C" && v <= -273.15 {
                            bad.push(format!("{}: temperature below absolute zero", r.row_id));
                        }
                    }
                    _ => bad.push(format!("{}: no value", r.row_id)),
                }
            }
            if r.record == Record::Step && !r.value.as_ref().is_some_and(|v| v.value > 0.0) {
                bad.push(format!("{}: the largest step must be positive", r.row_id));
            }
        }
    }
    verdict(
        &mut rep,
        "units",
        "times are in s, h or d; the factor is dimensionless, the temperature in degC, the steps positive",
        bad,
        n,
    );

    // Monotone times.
    let mut bad = Vec::new();
    let mut n = 0;
    for f in &sc.files {
        let mut keys: Vec<(String, String, Record)> = Vec::new();
        for r in f.rows.iter().filter(|r| r.record != Record::Meta) {
            let k = (r.variant.clone(), r.quantity.clone(), r.record);
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        for (variant, quantity, record) in keys {
            let times: Vec<f64> = f
                .rows
                .iter()
                .filter(|r| r.record == record && r.variant == variant && r.quantity == quantity)
                .filter_map(|r| r.t_seconds().ok())
                .collect();
            n += 1;
            if times.windows(2).any(|w| w[1] <= w[0]) {
                bad.push(format!("{} {variant}/{quantity}: times are not increasing", f.id));
            }
            if record == Record::History && times.first() != Some(&0.0) {
                bad.push(format!("{} {variant}/{quantity}: a history must start at 0", f.id));
            }
        }
    }
    verdict(
        &mut rep,
        "times",
        "the times of every history and of every step plan are strictly increasing, histories start at 0",
        bad,
        n,
    );

    // Source ids resolve.
    let mut bad = Vec::new();
    let mut n = 0;
    for f in &sc.files {
        for r in &f.rows {
            for id in &r.source_ids {
                n += 1;
                if !ids.contains(id.as_str()) {
                    bad.push(format!("{}: `{id}` is not a register row id", r.row_id));
                }
            }
        }
    }
    verdict(
        &mut rep,
        "sources",
        "every source row id resolves in the registers",
        bad,
        n,
    );

    // Labels.
    let mut bad = Vec::new();
    let mut n = 0;
    for f in &sc.files {
        for r in f.rows.iter().filter(|r| r.record == Record::History) {
            n += 1;
            match r.basis.as_str() {
                REGISTERED | DERIVED => {
                    if r.source_ids.is_empty() {
                        bad.push(format!("{}: `{}` without a source id", r.row_id, r.basis));
                    }
                }
                ASSUMED => {
                    if r.text.trim().len() < 12 {
                        bad.push(format!("{}: an assumed number needs a reason", r.row_id));
                    }
                }
                other => bad.push(format!("{}: unknown basis `{other}`", r.row_id)),
            }
        }
        for r in f.rows.iter().filter(|r| r.record == Record::Step) {
            n += 1;
            if r.basis != NUMERICAL {
                bad.push(format!("{}: a step must be labelled as a numerical setting", r.row_id));
            }
        }
        for r in f.rows.iter().filter(|r| {
            r.record == Record::Meta && r.t_value.is_some() && r.quantity != "heating_dt_max"
        }) {
            n += 1;
            if r.basis != ASSUMED && r.basis != REGISTERED {
                bad.push(format!("{}: a stated time needs a label", r.row_id));
            }
            if r.basis == ASSUMED && r.text.trim().len() < 12 {
                bad.push(format!("{}: an assumed number needs a reason", r.row_id));
            }
        }
    }
    verdict(
        &mut rep,
        "labels",
        "every history number carries a source row id or the label \"prescribed input (assumed)\" with a reason; every step is a labelled numerical setting",
        bad,
        n,
    );

    // Coverage: both histories exist, start at 0, end where the step plan ends.
    let mut bad = Vec::new();
    let mut n = 0;
    for f in &sc.files {
        let mut variants = vec![String::new()];
        variants.extend(f.variants());
        for v in variants {
            n += 1;
            // The end of a history: a linear table ends where the run ends; a step table
            // (a cycle variant) holds its last value from its last time to the end.
            let end_of = |rows: Vec<&ScenarioRow>| {
                let last = rows.last()?;
                Some((last.t_seconds().ok()?, last.interpolation == "step"))
            };
            let g = end_of(f.history(&v, "G_kernel_factor"));
            let t = end_of(f.history(&v, "T_C"));
            let s = f.steps(&v).last().and_then(|r| r.t_seconds().ok());
            match (g, t, s) {
                (Some((g, g_step)), Some((t, t_step)), Some(s)) => {
                    let tol = 1e-9 * s.max(1.0);
                    let miss = |end: f64, step: bool| {
                        if step {
                            end > s + tol
                        } else {
                            (end - s).abs() > tol
                        }
                    };
                    if miss(g, g_step) || miss(t, t_step) {
                        bad.push(format!(
                            "{} `{v}`: histories end at {g} s and {t} s, the step plan at {s} s",
                            f.id
                        ));
                    }
                }
                _ => bad.push(format!("{} `{v}`: a history or the step plan is missing", f.id)),
            }
        }
    }
    verdict(
        &mut rep,
        "coverage",
        "every variant has a generation history, a temperature history and a step plan that end at the same time",
        bad,
        n,
    );

    // Nuclides.
    let mut bad = Vec::new();
    let mut n = 0;
    for f in &sc.files {
        n += 1;
        let list = |key: &str| -> Vec<String> {
            f.meta("", key)
                .map(|r| {
                    r.text
                        .split(':')
                        .next()
                        .unwrap_or("")
                        .split(';')
                        .map(|s| s.trim().to_string())
                        .collect()
                })
                .unwrap_or_default()
        };
        let good = list("nuclides_meaningful");
        let none = list("nuclides_not_meaningful");
        if good != MEANINGFUL {
            bad.push(format!("{}: meaningful nuclides {good:?}", f.id));
        }
        if none != NOT_MEANINGFUL {
            bad.push(format!("{}: nuclides without a source {none:?}", f.id));
        }
        for name in &good {
            let y = reg.yields.get(Parent::U235, name);
            if !y.is_some_and(|y| y.fission_source_yield().is_ok()) {
                bad.push(format!("{}: {name} has no usable cumulative yield", f.id));
            }
            let lambda = reg.decay.get(name).map(|d| &d.stability);
            if !matches!(lambda, Some(Stability::Radioactive(_))) {
                bad.push(format!("{}: {name} has no decay constant", f.id));
            }
        }
        for name in &none {
            let refused = reg
                .yields
                .get(Parent::U235, name)
                .is_some_and(|y| y.fission_source_yield().is_err());
            if !refused {
                bad.push(format!("{}: {name} is not refused a fission-yield source", f.id));
            }
        }
    }
    verdict(
        &mut rep,
        "nuclides",
        "generation is meaningful only for nuclides with a registered cumulative yield; activation products are refused a source (decision 15)",
        bad,
        n,
    );

    // The exploration variant.
    let mut bad = Vec::new();
    let mut n = 0;
    let peak = condition_number(reg, "IRR-097");
    for f in &sc.files {
        n += 1;
        if !f.variants().iter().any(|v| v == EXPLORATION_VARIANT) {
            bad.push(format!("{}: no exploration variant", f.id));
            continue;
        }
        let t = f.history(EXPLORATION_VARIANT, "T_C");
        let at_peak = t.iter().any(|r| {
            r.value.as_ref().map(|v| v.value) == Some(EXPLORATION_TEMPERATURE_C)
                && r.source_ids.iter().any(|i| i == "IRR-097")
        });
        let highest = t
            .iter()
            .filter_map(|r| r.value.as_ref().map(|v| v.value))
            .fold(f64::NEG_INFINITY, f64::max);
        if !at_peak || highest != EXPLORATION_TEMPERATURE_C || peak != Some(EXPLORATION_TEMPERATURE_C)
        {
            bad.push(format!("{}: the exploration temperature is not IRR-097 = 1197", f.id));
        }
        let rate = f.meta(EXPLORATION_VARIANT, "rate_density_reference");
        if rate.map(|r| r.text.as_str()) != Some("compact:6-2-1") {
            bad.push(format!("{}: the exploration rate is not that of compact 6-2-1", f.id));
        }
    }
    verdict(
        &mut rep,
        "exploration",
        "each scenario has the registered exploration variant at the compact time-average peak temperature IRR-097 with the rate of the same compact (decision 66)",
        bad,
        n,
    );

    // Values that repeat a register number.
    let mut bad = Vec::new();
    let mut n = 0;
    let expect = |id: &str| condition_number(reg, id);
    for f in &sc.files {
        for r in f.rows.iter().filter(|r| r.record == Record::History) {
            let Some(v) = r.value.as_ref().map(|v| v.value) else {
                continue;
            };
            if r.quantity == "T_C" && r.source_ids.iter().any(|i| i == "IRR-059") {
                n += 1;
                if expect("IRR-059") != Some(v) {
                    bad.push(format!("{}: {v} is not IRR-059", r.row_id));
                }
            }
            if r.quantity == "T_C" && r.source_ids.iter().any(|i| i == "IRR-097") {
                n += 1;
                if expect("IRR-097") != Some(v) {
                    bad.push(format!("{}: {v} is not IRR-097", r.row_id));
                }
            }
            if r.source_ids.iter().any(|i| i == "IRR-002") && r.quantity != "fission_rate_density"
            {
                // A time that ends the irradiation, 620.2 EFPD, measured from the start of the
                // file's own clock: a tail that follows a lead-in starts at the lead-in time.
                n += 1;
                let end = expect("IRR-002").unwrap_or(f64::NAN) * 86_400.0;
                let lead = f
                    .rows
                    .iter()
                    .filter(|_| r.quantity != "lead_in")
                    .find(|x| x.quantity == "lead_in")
                    .and_then(|x| x.t_seconds().ok())
                    .unwrap_or(0.0);
                let s = r.t_seconds().unwrap_or(f64::NAN);
                // A row at time 0 cites IRR-002 for its value or its label, not for its time.
                if s > 0.0 && (s - (end - lead)).abs() > 1e-6 {
                    bad.push(format!("{}: time {s} s is not the end of irradiation", r.row_id));
                }
            }
        }
        for r in f.rows.iter().filter(|r| r.quantity == "lead_in") {
            n += 1;
            let lead = r.text.split(':').next().unwrap_or("").trim();
            let t = r.t_seconds().unwrap_or(f64::NAN);
            let s1_end = sc
                .get("S1")
                .and_then(|s| s.steps("").last().and_then(|x| x.t_seconds().ok()))
                .unwrap_or(f64::NAN);
            if lead != "S1" || !(t > 0.0 && t <= s1_end * (1.0 + 1e-12)) {
                bad.push(format!("{}: the lead-in must be S1 at a time inside its run", r.row_id));
            }
        }
    }
    verdict(
        &mut rep,
        "values",
        "numbers that repeat a register value equal it (IRR-059, IRR-097, IRR-002); lead-in times lie inside Scenario 1",
        bad,
        n,
    );

    // The cycle variant against the derived series.
    let mut bad = Vec::new();
    let mut n = 0;
    if let Some(s1) = sc.get("S1") {
        let rows = s1.history("cycles", "fission_rate_density");
        match reg.irradiation.derive_series(SeriesTarget::Capsule(3)) {
            Err(e) => bad.push(format!("the Capsule 3 series cannot be derived: {e}")),
            Ok(series) => {
                if rows.len() != series.cycles.len() {
                    bad.push(format!(
                        "{} cycle rows against {} derived cycles",
                        rows.len(),
                        series.cycles.len()
                    ));
                }
                let mut start = 0.0;
                for (row, cyc) in rows.iter().zip(&series.cycles) {
                    n += 1;
                    let printed = row.value.as_ref().map_or(f64::NAN, |v| v.value) * 1e19;
                    let rel = (printed - cyc.fissions_per_m3_per_s).abs()
                        / cyc.fissions_per_m3_per_s;
                    // The number is printed with three significant digits: half a unit of the
                    // last digit is at most 0.005 / 5.09 = 9.8e-4 of the smallest value.
                    if rel > 1.0e-3 {
                        bad.push(format!(
                            "{}: {printed:e} against the derived {:e} (relative {rel:.2e})",
                            row.row_id, cyc.fissions_per_m3_per_s
                        ));
                    }
                    let t = row.t_value.as_ref().map_or(f64::NAN, |t| t.value);
                    if (t - start).abs() > 0.05 {
                        bad.push(format!("{}: cycle starts at {t} d, expected {start} d", row.row_id));
                    }
                    start += cyc.efpd;
                }
                if let Some(end) = expect("IRR-002") {
                    if (start - end).abs() > 0.05 {
                        bad.push(format!("the cycle lengths add to {start} d, not IRR-002 = {end} d"));
                    }
                }
            }
        }
    }
    verdict(
        &mut rep,
        "cycles",
        "the printed per-cycle rates and start times of the cycle variant equal the series derived from the registered burnup to their printed digits",
        bad,
        n,
    );

    // The heating-history references of Scenario 5.
    let mut bad = Vec::new();
    let mut n = 0;
    if let Some(s5) = sc.get("S5") {
        for r in s5.rows.iter().filter(|r| r.quantity == "heating_history") {
            n += 1;
            let hist = r.text.trim();
            let rows_of: Vec<&str> = reg
                .temperature
                .rows
                .iter()
                .filter(|t| t.history_id == hist)
                .map(|t| t.id.as_str())
                .collect();
            let listed: Vec<&str> = r.source_ids.iter().map(String::as_str).collect();
            if rows_of.is_empty() || rows_of != listed {
                bad.push(format!("{}: {hist} has rows {rows_of:?}, the file lists {listed:?}", r.row_id));
            }
        }
    }
    verdict(
        &mut rep,
        "heating",
        "the heating histories of Scenario 5 are registered histories and the listed source rows are exactly their rows",
        bad,
        n,
    );
    rep
}
