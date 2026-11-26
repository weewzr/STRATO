//! `data/irradiation/*.csv`: AGR-1 irradiation conditions, burnup after each ATR cycle, and the
//! digitised capsule heat history, and the fission-rate density derived from them.
//!
//! The AGR-1 documents publish no fission rate. What they publish is the burnup of every compact
//! after every ATR cycle (Collin 2015, Appendix D), the length of every cycle in effective
//! full-power days and the heavy-metal inventory of one compact. [`Irradiation::derive_series`]
//! turns these into a fission-rate density per cycle, in fissions per cubic metre of kernel per
//! second. The result is always labelled [`DERIVED_LABEL`]; it is a calculation from published
//! inputs, never a measurement and never a published fission rate.

use crate::common::{EvidenceClass, Status};
use crate::error::{DataError, Result};
use crate::string_enum;
use crate::table::{Printed, Row, Table};

/// Path of the conditions file below the data folder.
pub const CONDITIONS_FILE: &str = "irradiation/agr1_irradiation_conditions.csv";
/// Path of the per-cycle burnup file.
pub const BURNUP_FILE: &str = "irradiation/agr1_burnup_by_cycle.csv";
/// Path of the digitised power history file.
pub const POWER_FILE: &str = "irradiation/agr1_power_history_digitised.csv";

/// The label every derived fission-rate density carries wherever it is shown or exported.
pub const DERIVED_LABEL: &str = "derived from published AGR-1 burnup (calculated)";

/// Avogadro constant, mol-1 (exact SI value; not from an AGR-1 document).
pub const AVOGADRO_PER_MOL: f64 = 6.022_140_76e23;

/// Seconds in one day (an effective full-power day is taken as exactly 86 400 s).
pub const SECONDS_PER_DAY: f64 = 86_400.0;

/// Number of ATR cycles of AGR-1.
pub const CYCLES: u8 = 13;

/// Header of the conditions file.
pub const CONDITIONS_HEADER: [&str; 20] = [
    "id",
    "condition_id",
    "condition",
    "scope",
    "capsule",
    "compact",
    "quantity",
    "symbol",
    "value",
    "value_min",
    "value_max",
    "unit",
    "uncertainty",
    "data_class",
    "evidence_class",
    "source",
    "source_location",
    "link",
    "status",
    "notes",
];

/// Header of the burnup file.
pub const BURNUP_HEADER: [&str; 21] = [
    "id",
    "history_id",
    "history",
    "scope",
    "capsule",
    "compact",
    "cycle",
    "atr_cycle",
    "cycle_end_cumulative_efpd",
    "quantity",
    "symbol",
    "value",
    "unit",
    "uncertainty",
    "data_class",
    "evidence_class",
    "source",
    "source_location",
    "link",
    "status",
    "notes",
];

/// Header of the power history file.
pub const POWER_HEADER: [&str; 16] = [
    "id",
    "history_id",
    "history",
    "capsule",
    "efpd_window_centre",
    "quantity",
    "value",
    "unit",
    "uncertainty",
    "data_class",
    "evidence_class",
    "source",
    "source_location",
    "link",
    "status",
    "notes",
];

string_enum! {
    /// The `scope` column of the conditions file.
    pub enum ConditionScope ("condition scope") {
        /// The whole experiment.
        Experiment => "experiment",
        /// One capsule.
        Capsule => "capsule",
        /// One compact.
        Compact => "compact",
        /// The kernel.
        Kernel => "kernel",
        /// One particle.
        Particle => "particle",
    }
}

string_enum! {
    /// The `scope` column of the burnup file.
    pub enum BurnupScope ("burnup scope") {
        /// One compact.
        Compact => "compact",
        /// The average over the compacts of one capsule.
        Capsule => "capsule",
    }
}

string_enum! {
    /// The `symbol` column of the burnup file: which quantity the row holds.
    pub enum BurnupQuantity ("burnup quantity symbol") {
        /// Cumulative burnup, % FIMA (fissions per initial heavy-metal atom times 100).
        Burnup => "Bu",
        /// Cumulative fast neutron fluence, 1e25 n m-2 (E > 0.18 MeV).
        Fluence => "Phi",
    }
}

/// The `value` cell of a condition row.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionValue {
    /// A number with its printed text.
    Number(Printed),
    /// Text (a date, a range, a description).
    Text(String),
}

/// One row of `agr1_irradiation_conditions.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionRow {
    /// Line in the file.
    pub line: usize,
    /// Row id (`IRR-001`).
    pub id: String,
    /// Condition set id.
    pub condition_id: String,
    /// Scope.
    pub scope: ConditionScope,
    /// Capsule number (1 to 6), when the row has one.
    pub capsule: Option<u8>,
    /// Compact name (`3-4-3`), when the row has one.
    pub compact: Option<String>,
    /// Quantity as printed.
    pub quantity: String,
    /// Symbol as printed.
    pub symbol: String,
    /// Value.
    pub value: ConditionValue,
    /// Unit.
    pub unit: String,
    /// Uncertainty as printed (`none printed`, or a statement).
    pub uncertainty: String,
    /// Data class as printed.
    pub data_class: String,
    /// Evidence class.
    pub evidence_class: EvidenceClass,
    /// Source location.
    pub source_location: String,
    /// Status.
    pub status: Status,
    /// Notes.
    pub notes: String,
}

/// One row of `agr1_burnup_by_cycle.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct BurnupRow {
    /// Line in the file.
    pub line: usize,
    /// Row id (`IRRB-0001`).
    pub id: String,
    /// Compact or capsule average.
    pub scope: BurnupScope,
    /// Capsule number.
    pub capsule: u8,
    /// Compact name; absent for a capsule average.
    pub compact: Option<String>,
    /// ATR cycle number of AGR-1, 1 to 13.
    pub cycle: u8,
    /// ATR cycle name (`138B`).
    pub atr_cycle: String,
    /// Cumulative effective full-power days at the end of the cycle.
    pub cycle_end_efpd: Printed,
    /// Burnup or fluence.
    pub quantity: BurnupQuantity,
    /// The cumulative value after this cycle.
    pub value: Printed,
    /// Unit as printed.
    pub unit: String,
    /// Evidence class.
    pub evidence_class: EvidenceClass,
    /// Status.
    pub status: Status,
    /// Source location.
    pub source_location: String,
    /// Notes.
    pub notes: String,
}

/// One row of `agr1_power_history_digitised.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct PowerRow {
    /// Line in the file.
    pub line: usize,
    /// Row id (`IRRP-0001`).
    pub id: String,
    /// Capsule number.
    pub capsule: u8,
    /// Centre of the 5 EFPD window, EFPD.
    pub efpd_window_centre: Printed,
    /// Capsule-average heat generation rate, MW m-3 (digitised; calculated by INL, not measured).
    pub heat_mw_per_m3: Printed,
    /// Reading uncertainty statement as printed.
    pub uncertainty: String,
    /// Evidence class.
    pub evidence_class: EvidenceClass,
    /// Status (always digitised).
    pub status: Status,
}

fn opt_capsule(row: &Row<'_>) -> Result<Option<u8>> {
    match row.optional("capsule")? {
        None => Ok(None),
        Some(t) => match t.parse::<u8>() {
            Ok(c) if (1..=6).contains(&c) => Ok(Some(c)),
            _ => Err(row.err(Some("capsule"), format!("`{t}` is not a capsule 1 to 6"))),
        },
    }
}

fn read_condition(row: Row<'_>) -> Result<ConditionRow> {
    let value = match row.optional("value")? {
        Some(t) => match Printed::parse(t) {
            Ok(p) => ConditionValue::Number(p),
            Err(_) => ConditionValue::Text(t.to_string()),
        },
        None => ConditionValue::Text(String::new()),
    };
    Ok(ConditionRow {
        line: row.line(),
        id: row.required("id")?.to_string(),
        condition_id: row.required("condition_id")?.to_string(),
        scope: row.vocab("scope")?,
        capsule: opt_capsule(&row)?,
        compact: row.optional("compact")?.map(str::to_string),
        quantity: row.required("quantity")?.to_string(),
        symbol: row.text("symbol")?.to_string(),
        value,
        unit: row.required("unit")?.to_string(),
        uncertainty: row.required("uncertainty")?.to_string(),
        data_class: row.required("data_class")?.to_string(),
        evidence_class: row.vocab("evidence_class")?,
        source_location: row.required("source_location")?.to_string(),
        status: row.vocab("status")?,
        notes: row.text("notes")?.to_string(),
    })
}

fn read_burnup(row: Row<'_>) -> Result<BurnupRow> {
    let cycle = row.uint("cycle")?;
    if !(1..=u32::from(CYCLES)).contains(&cycle) {
        return Err(row.err(Some("cycle"), "the cycle must be 1 to 13"));
    }
    let capsule =
        opt_capsule(&row)?.ok_or_else(|| row.err(Some("capsule"), "a capsule is required"))?;
    Ok(BurnupRow {
        line: row.line(),
        id: row.required("id")?.to_string(),
        scope: row.vocab("scope")?,
        capsule,
        compact: row.optional("compact")?.map(str::to_string),
        cycle: cycle as u8,
        atr_cycle: row.required("atr_cycle")?.to_string(),
        cycle_end_efpd: row.number("cycle_end_cumulative_efpd")?,
        quantity: row.vocab("symbol")?,
        value: row.number("value")?,
        unit: row.required("unit")?.to_string(),
        evidence_class: row.vocab("evidence_class")?,
        status: row.vocab("status")?,
        source_location: row.required("source_location")?.to_string(),
        notes: row.text("notes")?.to_string(),
    })
}

fn read_power(row: Row<'_>) -> Result<PowerRow> {
    let capsule =
        opt_capsule(&row)?.ok_or_else(|| row.err(Some("capsule"), "a capsule is required"))?;
    Ok(PowerRow {
        line: row.line(),
        id: row.required("id")?.to_string(),
        capsule,
        efpd_window_centre: row.number("efpd_window_centre")?,
        heat_mw_per_m3: row.number("value")?,
        uncertainty: row.required("uncertainty")?.to_string(),
        evidence_class: row.vocab("evidence_class")?,
        status: row.vocab("status")?,
    })
}

fn unique_ids<'a>(file: &'static str, ids: impl Iterator<Item = (usize, &'a str)>) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for (line, id) in ids {
        if !seen.insert(id) {
            return Err(DataError::line(
                file,
                line,
                format!("the id {id} appears twice"),
            ));
        }
    }
    Ok(())
}

/// What a fission-rate series is derived for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeriesTarget {
    /// The average over the compacts of a capsule (1 to 6).
    Capsule(u8),
    /// One compact, for example `3-4-3`.
    Compact(String),
}

/// The kernel heavy-metal inventory used to turn burnup into fissions.
#[derive(Debug, Clone, PartialEq)]
pub struct KernelInventory {
    /// Heavy-metal atoms in the compact of the published masses (compact 3-4-3), from mass
    /// numbers used as molar masses in g mol-1 (an approximation of the report, below 0.02 %).
    pub atoms_per_compact: f64,
    /// Particles per Baseline compact (a value the source derives from other properties).
    pub particles_per_compact: f64,
    /// Kernel volume, m3, from the mean diameter.
    pub kernel_volume_m3: f64,
    /// Heavy-metal atoms per cubic metre of kernel.
    pub atoms_per_m3: f64,
    /// Ids of the condition rows the inventory is built from.
    pub source_ids: Vec<String>,
}

/// One ATR cycle of a derived series.
#[derive(Debug, Clone, PartialEq)]
pub struct CycleFission {
    /// Cycle number 1 to 13.
    pub cycle: u8,
    /// ATR cycle name.
    pub atr_cycle: String,
    /// Length of the cycle, effective full-power days (Collin 2015 Table 2).
    pub efpd: f64,
    /// Burnup gained in the cycle, % FIMA (difference of two printed cumulative values).
    pub burnup_increment_pct_fima: f64,
    /// Cycle-average fission-rate density, fissions per cubic metre of kernel per second.
    pub fissions_per_m3_per_s: f64,
}

/// A fission-rate density series derived from published burnup. Always labelled.
#[derive(Debug, Clone, PartialEq)]
pub struct DerivedSeries {
    /// What it is derived for.
    pub target: SeriesTarget,
    /// Always [`DERIVED_LABEL`].
    pub label: &'static str,
    /// The thirteen cycles.
    pub cycles: Vec<CycleFission>,
    /// The kernel inventory used.
    pub inventory: KernelInventory,
    /// Ids of the burnup rows and cycle-length rows used.
    pub source_ids: Vec<String>,
}

impl DerivedSeries {
    /// Time average over the whole irradiation, weighted by cycle length (effective full-power days).
    pub fn time_average(&self) -> f64 {
        let total: f64 = self.cycles.iter().map(|c| c.efpd).sum();
        self.cycles
            .iter()
            .map(|c| c.fissions_per_m3_per_s * c.efpd)
            .sum::<f64>()
            / total
    }

    /// The caveats that go with the numbers, to be shown beside them.
    pub fn caveats(&self) -> [&'static str; 4] {
        [
            "calculated from published AGR-1 burnup, not a published fission rate and not a measurement",
            "cycle average only: the rate varies within a cycle and the time base is core-wide effective full-power days",
            "the heavy-metal inventory of compact 3-4-3 is applied to every compact and variant",
            "an AGR-1 irradiation condition as computed by INL, not an experimentally validated condition of this model",
        ]
    }
}

/// The three irradiation files, typed.
#[derive(Debug, Clone)]
pub struct Irradiation {
    /// Conditions.
    pub conditions: Vec<ConditionRow>,
    /// Burnup and fluence after each cycle.
    pub burnup: Vec<BurnupRow>,
    /// Digitised heat history.
    pub power: Vec<PowerRow>,
}

impl Irradiation {
    /// Read and validate the three file texts.
    pub fn parse(conditions: &str, burnup: &str, power: &str) -> Result<Irradiation> {
        let c = Table::parse(CONDITIONS_FILE, conditions, &CONDITIONS_HEADER)?;
        let b = Table::parse(BURNUP_FILE, burnup, &BURNUP_HEADER)?;
        let p = Table::parse(POWER_FILE, power, &POWER_HEADER)?;
        let conditions = c.rows().map(read_condition).collect::<Result<Vec<_>>>()?;
        let burnup = b.rows().map(read_burnup).collect::<Result<Vec<_>>>()?;
        let power = p.rows().map(read_power).collect::<Result<Vec<_>>>()?;
        unique_ids(
            CONDITIONS_FILE,
            conditions.iter().map(|r| (r.line, r.id.as_str())),
        )?;
        unique_ids(BURNUP_FILE, burnup.iter().map(|r| (r.line, r.id.as_str())))?;
        unique_ids(POWER_FILE, power.iter().map(|r| (r.line, r.id.as_str())))?;
        Ok(Irradiation {
            conditions,
            burnup,
            power,
        })
    }

    /// The condition row with this quantity text (exactly one must exist).
    pub fn condition(&self, quantity: &str) -> Result<&ConditionRow> {
        let mut it = self.conditions.iter().filter(|r| r.quantity == quantity);
        match (it.next(), it.next()) {
            (Some(r), None) => Ok(r),
            (None, _) => Err(DataError::file(
                CONDITIONS_FILE,
                format!("no row with the quantity `{quantity}`"),
            )),
            (Some(_), Some(_)) => Err(DataError::file(
                CONDITIONS_FILE,
                format!("more than one row with the quantity `{quantity}`"),
            )),
        }
    }

    fn condition_number(&self, quantity: &str) -> Result<(f64, &ConditionRow)> {
        let r = self.condition(quantity)?;
        match &r.value {
            ConditionValue::Number(p) => Ok((p.value, r)),
            ConditionValue::Text(_) => Err(DataError::line(
                CONDITIONS_FILE,
                r.line,
                format!("the value of `{quantity}` is not a number"),
            )),
        }
    }

    /// Length of ATR cycle `n` (1 to 13) in effective full-power days (Collin 2015 Table 2).
    pub fn cycle_efpd(&self, n: u8) -> Result<(f64, &ConditionRow)> {
        self.condition_number(&format!("AGR-1 cycle {n} length"))
    }

    /// The kernel heavy-metal inventory: published masses of compact 3-4-3 (Sterbentz 2015),
    /// particles per Baseline compact (Collin 2015 Table A-3) and kernel diameter (Table A-1).
    pub fn kernel_inventory(&self) -> Result<KernelInventory> {
        let mut atoms = 0.0;
        let mut ids = Vec::new();
        for (iso, mass_number) in [
            ("U-234", 234.0),
            ("U-235", 235.0),
            ("U-236", 236.0),
            ("U-238", 238.0),
        ] {
            let (m_g, row) =
                self.condition_number(&format!("initial {iso} mass, compact 3-4-3"))?;
            atoms += m_g / mass_number * AVOGADRO_PER_MOL;
            ids.push(row.id.clone());
        }
        let (n_p, row) = self.condition_number("particles per compact, Baseline")?;
        ids.push(row.id.clone());
        let (d_um, row) = self.condition_number("kernel diameter")?;
        ids.push(row.id.clone());
        let d_m = d_um * 1e-6;
        let volume = std::f64::consts::PI / 6.0 * d_m * d_m * d_m;
        Ok(KernelInventory {
            atoms_per_compact: atoms,
            particles_per_compact: n_p,
            kernel_volume_m3: volume,
            atoms_per_m3: atoms / n_p / volume,
            source_ids: ids,
        })
    }

    /// The cumulative burnup (% FIMA) after each cycle for a target, cycles 1 to 13 in order.
    pub fn cumulative_burnup(&self, target: &SeriesTarget) -> Result<Vec<&BurnupRow>> {
        let mut out: Vec<&BurnupRow> = self
            .burnup
            .iter()
            .filter(|r| {
                r.quantity == BurnupQuantity::Burnup
                    && match target {
                        SeriesTarget::Capsule(c) => {
                            r.scope == BurnupScope::Capsule && r.capsule == *c
                        }
                        SeriesTarget::Compact(name) => {
                            r.scope == BurnupScope::Compact
                                && r.compact.as_deref() == Some(name.as_str())
                        }
                    }
            })
            .collect();
        out.sort_by_key(|r| r.cycle);
        let complete = out.len() == usize::from(CYCLES)
            && out
                .iter()
                .enumerate()
                .all(|(i, r)| usize::from(r.cycle) == i + 1);
        if !complete {
            return Err(DataError::file(
                BURNUP_FILE,
                format!("{target:?}: the thirteen cumulative burnup values were not found"),
            ));
        }
        Ok(out)
    }

    /// Derive the cycle-average fission-rate density for a capsule average or one compact.
    ///
    /// For ATR cycle `n`: `dBu = Bu_n - Bu_(n-1)` (`Bu_0 = 0`, % FIMA), and
    /// `F_n = dBu / 100 * N_HM / (EFPD_n * 86 400 s)` with `N_HM` the heavy-metal atoms per cubic
    /// metre of kernel. Units: (fissions per initial atom) times (atoms per m3) per second.
    pub fn derive_series(&self, target: SeriesTarget) -> Result<DerivedSeries> {
        let inventory = self.kernel_inventory()?;
        let rows = self.cumulative_burnup(&target)?;
        let mut cycles = Vec::new();
        let mut source_ids = Vec::new();
        let mut previous = 0.0;
        for r in rows {
            let (efpd, len_row) = self.cycle_efpd(r.cycle)?;
            let increment = r.value.value - previous;
            previous = r.value.value;
            let f = increment / 100.0 * inventory.atoms_per_m3 / (efpd * SECONDS_PER_DAY);
            cycles.push(CycleFission {
                cycle: r.cycle,
                atr_cycle: r.atr_cycle.clone(),
                efpd,
                burnup_increment_pct_fima: increment,
                fissions_per_m3_per_s: f,
            });
            source_ids.push(r.id.clone());
            source_ids.push(len_row.id.clone());
        }
        Ok(DerivedSeries {
            target,
            label: DERIVED_LABEL,
            cycles,
            inventory,
            source_ids,
        })
    }

    /// The time-average fission-rate density over the whole irradiation from an end-of-irradiation
    /// burnup `bu_pct_fima` (% FIMA) and the irradiation length in effective full-power days:
    /// `Bu / 100 * N_HM / (t * 86 400 s)`. Labelled [`DERIVED_LABEL`] by the caller.
    pub fn time_average_from_end_burnup(&self, bu_pct_fima: f64, efpd: f64) -> Result<f64> {
        Ok(bu_pct_fima / 100.0 * self.kernel_inventory()?.atoms_per_m3 / (efpd * SECONDS_PER_DAY))
    }
}
