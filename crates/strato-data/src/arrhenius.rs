//! `data/diffusivity/arrhenius.csv`: Arrhenius terms of the diffusion coefficient.
//!
//! One row is one term `D0 exp(-Q / (R T))`; a coefficient of two terms is two rows with the same
//! `set_id`, species, layer and origin and `term` 1 and 2 (the coefficient is their sum). `D0` is in
//! m2 s-1 and `Q` in kJ mol-1 (multiply by 1e3 before use with R in J mol-1 K-1).
//!
//! The reader keeps the distinctions the register makes: a law is an Arrhenius term, a constant
//! or an instantaneous release; a `D0` is a point value or an upper bound; the fluence factor of
//! caesium in SiC is documented, documented but not used, or absent; a reduced coefficient `D'0`
//! is a rate in s-1 that must be multiplied by the kernel radius squared.

use crate::common::{EvidenceClass, Status};
use crate::error::{DataError, Result};
use crate::string_enum;
use crate::table::{Printed, Row, Table, Vocabulary};

/// Path of the file below the data folder.
pub const FILE: &str = "diffusivity/arrhenius.csv";

/// The exact header.
pub const HEADER: [&str; 29] = [
    "id",
    "set_id",
    "nuclide",
    "layer",
    "material",
    "origin",
    "term",
    "D0_m2_s",
    "D0_expression",
    "Q_kJ_mol",
    "form",
    "T_min_C",
    "T_max_C",
    "T_range_note",
    "R_J_mol_K",
    "data_class",
    "evidence_class",
    "ref_in_source",
    "source",
    "source_location",
    "link",
    "status",
    "notes",
    "coefficient_class",
    "phase_of_use",
    "reference_radius_um",
    "fluence_definition",
    "range_kind",
    "fitted_to",
];

/// Kernel radius of the reduced coefficients of TECDOC-1674 Table 7.1, metre (250 um, register note of T71-01).
pub const REDUCED_COEFFICIENT_KERNEL_RADIUS_M: f64 = 250e-6;

string_enum! {
    /// The `layer` column.
    pub enum DiffusionLayer ("diffusion layer") {
        /// Kernel.
        Kernel => "kernel",
        /// Buffer.
        Buffer => "buffer",
        /// One pyrocarbon set for IPyC and OPyC (no split in the register, GAP-02).
        PyC => "IPyC/OPyC",
        /// Silicon carbide.
        SiC => "SiC",
        /// Compact matrix graphite.
        Matrix => "matrix",
        /// Structural graphite outside the particle.
        StructuralGraphite => "structural graphite",
        /// A failed layer (convention).
        FailedLayer => "failed layer",
    }
}

string_enum! {
    /// The `form` column: how the register writes the coefficient.
    pub enum Form ("arrhenius form") {
        /// One term.
        OneTerm => "1T",
        /// First or second of two terms.
        TwoTerm => "2T",
        /// Two terms printed as a sum (Eq. 10.9).
        TwoTermSum => "2T (sum)",
        /// First of two terms, with a fluence factor.
        TwoTermFluenceDependent => "2T (term 1 fluence dependent)",
        /// Reduced coefficient D' (rate in s-1).
        Reduced => "1T (reduced D')",
        /// Only one term of the source is kept (BISON).
        SingleTerm => "1T (single term)",
        /// Low-temperature branch.
        LowTemperatureBranch => "1T (low-T branch)",
        /// High-temperature branch.
        HighTemperatureBranch => "1T (high-T branch)",
        /// The printed value is an upper bound.
        UpperBound => "1T (upper bound)",
        /// A constant (no activation energy).
        Constant => "constant",
        /// A constant with Q = 0 printed.
        ConstantQZero => "constant (Q = 0)",
        /// A temperature-independent single term (Q = 0).
        TemperatureIndependent => "1T (T-independent)",
        /// Instantaneous release (infinite D).
        Instantaneous => "instantaneous",
        /// One term of the AGR-1 irradiation-phase PARFUME set (Collin 2015 Table 2).
        IrradiationPhaseSingleTerm => "1T (single term; irradiation-phase set)",
        /// A single term fitted to AGR-1 heating-test data.
        Fitted => "1T (fitted)",
        /// A hypothesis of the source, not a fit to a measured coefficient.
        Hypothesis => "1T (hypothesis)",
    }
}

string_enum! {
    /// The `coefficient_class` column: the four classes of owner ruling 16 (decision 16 of the
    /// coordinator), exactly one per row. A row whose class could not be established would carry
    /// `NOT VERIFIED`, the register's existing word; no row does at present.
    pub enum CoefficientClass ("coefficient class") {
        /// Measured on AGR-1 fuel. No row has this class: no AGR-1 diffusion coefficient was measured.
        Agr1ExperimentalMeasurement => "AGR-1 experimental measurement",
        /// Fitted to AGR-1 data (Schappel 2020).
        Agr1SpecificFittedCorrelation => "AGR-1-specific fitted correlation",
        /// The correlation PARFUME (or BISON) uses for AGR-1.
        PublishedParfumeCorrelation => "published PARFUME correlation",
        /// A published correlation for another fuel or material, transferred to AGR-1.
        TransferredMaterialCorrelation => "transferred material correlation",
        /// The class could not be established.
        Unverified => "NOT VERIFIED",
    }
}

string_enum! {
    /// The `phase_of_use` column: the phase for which the source prints or fits the coefficient.
    pub enum PhaseOfUse ("phase of use") {
        /// The irradiation phase.
        Irradiation => "irradiation",
        /// The post-irradiation heating tests (safety tests).
        HeatingTest => "heating test",
        /// The source ties the row to no phase.
        NotSpecified => "not specified",
    }
}

string_enum! {
    /// The `range_kind` column: what the temperature range of a row is.
    pub enum RangeKind ("range kind") {
        /// The source prints the range with the correlation.
        Printed => "printed",
        /// The source prints no range; the range is the one the source fitted the correlation to
        /// (the temperatures of its heating tests). It is not a validity range printed by the source.
        Fitted => "fitted",
    }
}

string_enum! {
    /// The `T_range_note` column.
    pub enum RangeNote ("temperature range note") {
        /// No range printed.
        NonePrinted => "none printed",
        /// No range printed (short form).
        None => "none",
        /// Printed range.
        PrintedRange => "printed range",
        /// Accident conditions.
        AccidentConditions => "accident conditions",
        /// Accident conditions (abbreviation).
        AC => "AC (accident conditions)",
        /// Normal operating conditions.
        NOC => "NOC (normal operating conditions)",
        /// Printed as an upper limit of 2000 C.
        PrintedUpTo2000 => "printed as <= 2000 C",
        /// Printed as an upper limit of 1250 C.
        PrintedUpTo1250 => "printed as <= 1250 C",
        /// A single temperature.
        SingleTemperature => "single temperature",
        /// Isothermal heating tests.
        IsothermalHeatingTests => "isothermal heating tests (p. 162)",
        /// Fitted to the 1600 and 1800 C AGR-1 heating tests (Schappel 2020).
        FittedToAgr1HeatingTests => "none printed; fitted to the 1600 and 1800 C post-irradiation heating tests (printed p. 13-14)",
        /// The source prints no range; the fit used the 1600 and 1800 C heating tests and the typed range is that fitted range (Schappel 2020, rows S20-01 to S20-03).
        FittedRange1600To1800 => "no range is printed; fitted range 1600 to 1800 C (the 1600 and 1800 C post-irradiation heating tests, printed p. 13-14)",
        /// A hypothesis, not a fit (Schappel 2020, row S20-04).
        HypothesisNotFitted => "none printed; no fit: a hypothesis (pre-factor and activation energy follow from two assumed values, see the notes), not fitted to a measured temperature range",
    }
}

/// The gas constant a set of coefficients was printed with.
#[derive(Debug, Clone, PartialEq)]
pub enum GasConstant {
    /// A printed value, J mol-1 K-1 (8.3143 or 8.3145 in the register).
    Printed(Printed),
    /// The source prints none (`not printed`).
    NotPrinted,
}

/// Whether a number is a point value or an upper bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundKind {
    /// A value.
    PointValue,
    /// An upper bound (`<=` printed).
    UpperBound,
}

/// The law of one row.
#[derive(Debug, Clone, PartialEq)]
pub enum Law {
    /// `D0 exp(-Q/(R T))` with `D0` in m2 s-1 and `Q` in kJ mol-1.
    Arrhenius {
        /// Pre-exponential factor.
        d0: Printed,
        /// Whether `d0` is a value or an upper bound.
        d0_kind: BoundKind,
        /// Activation energy.
        q_kj_mol: Printed,
    },
    /// A diffusion coefficient that does not depend on temperature.
    Constant {
        /// The coefficient, m2 s-1.
        d: Printed,
        /// The activation energy as printed (0), absent for the single-temperature value.
        q_kj_mol: Option<Printed>,
    },
    /// Release from the compact is instantaneous: no finite coefficient exists.
    Instantaneous,
}

/// How the row documents the fluence factor `exp(Gamma/5)` of caesium in SiC.
#[derive(Debug, Clone, PartialEq)]
pub enum FluenceFactor {
    /// The row has no fluence factor.
    Absent,
    /// The term is multiplied by `exp(Gamma / divisor)`, Gamma in 1e25 m-2.
    Documented {
        /// The divisor, 5 in the register.
        divisor: f64,
    },
    /// The source has the factor but does not use it (Collin 2016 Table 2 note).
    DocumentedNotUsed,
}

/// A reduced coefficient `D'0` in s-1 of TECDOC-1674 Table 7.1 (D0 = D'0 r_k^2).
#[derive(Debug, Clone, PartialEq)]
pub struct ReducedCoefficient {
    /// `D'0`, s-1.
    pub d_prime_per_s: Printed,
}

/// A species label as printed, for example `Cs`, `Kr/Xe`, `Cs (upper curve)`, `all species`.
#[derive(Debug, Clone, PartialEq)]
pub enum Species {
    /// `all species`.
    AllSpecies,
    /// `all metallic FP`.
    AllMetallicFp,
    /// One or more elements or nuclides, with an optional qualifier in parentheses.
    Named {
        /// The names split at `/`.
        items: Vec<String>,
        /// The text in parentheses, if any.
        qualifier: Option<String>,
    },
}

impl Species {
    /// Parse a label.
    pub fn parse(raw: &str) -> std::result::Result<Species, String> {
        match raw {
            "all species" => return Ok(Species::AllSpecies),
            "all metallic FP" => return Ok(Species::AllMetallicFp),
            _ => {}
        }
        let (names, qualifier) = match raw.split_once(" (") {
            Some((n, q)) => match q.strip_suffix(')') {
                Some(q) => (n, Some(q.to_string())),
                None => return Err(format!("`{raw}`: unbalanced parenthesis")),
            },
            None => (raw, None),
        };
        let items: Vec<String> = names.split('/').map(str::to_string).collect();
        for i in &items {
            let ok = !i.is_empty()
                && i.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && i.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
            if !ok {
                return Err(format!(
                    "`{i}` in `{raw}` is not an element or nuclide name"
                ));
            }
        }
        Ok(Species::Named { items, qualifier })
    }
}

/// One row of `arrhenius.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrheniusRow {
    /// Line in the file.
    pub line: usize,
    /// Row identifier (`A1-01`).
    pub id: String,
    /// Set identifier (`T978-A1`).
    pub set_id: String,
    /// Species label, parsed.
    pub species: Species,
    /// Species label as printed.
    pub species_text: String,
    /// Layer.
    pub layer: DiffusionLayer,
    /// Material as printed.
    pub material: String,
    /// Origin as printed.
    pub origin: String,
    /// Term number, 1 or 2.
    pub term: u8,
    /// The law with its `D0` and `Q`.
    pub law: Law,
    /// `D0_expression` as printed (empty when blank).
    pub expression: String,
    /// The fluence factor documented in the expression or form.
    pub fluence: FluenceFactor,
    /// The reduced coefficient, for rows of form `1T (reduced D')`.
    pub reduced: Option<ReducedCoefficient>,
    /// Form.
    pub form: Form,
    /// Lower end of the printed temperature range, degC.
    pub t_min_c: Option<Printed>,
    /// Upper end of the printed temperature range, degC.
    pub t_max_c: Option<Printed>,
    /// Whether the range is printed by the source or is the fitted range (None when the row has no typed range).
    pub range_kind: Option<RangeKind>,
    /// Note on the range.
    pub range_note: RangeNote,
    /// Gas constant printed with the set.
    pub gas_constant: GasConstant,
    /// Data class as printed.
    pub data_class: String,
    /// Evidence class.
    pub evidence_class: EvidenceClass,
    /// Reference number inside the source.
    pub ref_in_source: String,
    /// Short citation.
    pub source: String,
    /// Location in the source.
    pub source_location: String,
    /// Link.
    pub link: String,
    /// Status.
    pub status: Status,
    /// Notes.
    pub notes: String,
    /// The class of owner ruling 16.
    pub coefficient_class: CoefficientClass,
    /// The phase the source ties the coefficient to.
    pub phase_of_use: PhaseOfUse,
    /// The kernel radius, um, to which the original publication refers the coefficient (kernel rows only).
    pub reference_radius_um: Option<Printed>,
    /// How the fluence of the `exp(Gamma/5)` factor is defined, or that the factor is not applied (empty when the row has no factor).
    pub fluence_definition: String,
    /// The datasets this row was fitted to, `<source> | <location prefix>` separated by `;`
    /// (empty when the register does not record that the row was fitted to a dataset). Set only
    /// where the comparability assessment establishes it; the audit log has one entry per row.
    pub fitted_to: String,
}

/// A diffusion coefficient with the kind of number it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Diffusivity {
    /// m2 s-1.
    pub m2_per_s: f64,
    /// Value or upper bound.
    pub kind: BoundKind,
}

impl ArrheniusRow {
    /// Whether this row was fitted to the dataset that has this source and location (the
    /// register field `fitted_to`: an entry is `<source> | <location prefix>`, the source equal to
    /// `source` and the location starting with the prefix).
    pub fn fitted_to_dataset(&self, source: &str, location: &str) -> bool {
        self.fitted_to.split(';').any(|entry| {
            entry.split_once(" | ").is_some_and(|(s, prefix)| {
                !prefix.trim().is_empty()
                    && s.trim() == source.trim()
                    && location.trim().starts_with(prefix.trim())
            })
        })
    }

    /// The class as printed in the file.
    pub fn coefficient_class_text(&self) -> &'static str {
        use crate::table::Vocabulary;
        self.coefficient_class.as_str()
    }

    /// The species label as printed.
    pub fn nuclide_text(&self) -> &str {
        &self.species_text
    }

    /// `D0` (or the constant) as printed, when the row has one.
    pub fn d0_value(&self) -> Option<&str> {
        match &self.law {
            Law::Arrhenius { d0, .. } => Some(d0.text.as_str()),
            Law::Constant { d, .. } => Some(d.text.as_str()),
            Law::Instantaneous => None,
        }
    }

    /// `Q` as printed, when the row has one.
    pub fn q_value(&self) -> Option<&str> {
        match &self.law {
            Law::Arrhenius { q_kj_mol, .. } => Some(q_kj_mol.text.as_str()),
            Law::Constant { q_kj_mol, .. } => q_kj_mol.as_ref().map(|q| q.text.as_str()),
            Law::Instantaneous => None,
        }
    }

    /// The coefficient of this one term at `t_kelvin`, for gas constant `r_gas` (J mol-1 K-1) and
    /// fluence `gamma` in 1e25 m-2. The fluence factor multiplies the term only when the row
    /// documents it as used; pass 0 to evaluate without fluence. An instantaneous row has no
    /// finite coefficient and returns `None`.
    pub fn diffusivity(&self, t_kelvin: f64, r_gas: f64, gamma: f64) -> Option<Diffusivity> {
        match &self.law {
            Law::Instantaneous => None,
            Law::Constant { d, .. } => Some(Diffusivity {
                m2_per_s: d.value,
                kind: BoundKind::PointValue,
            }),
            Law::Arrhenius {
                d0,
                d0_kind,
                q_kj_mol,
            } => {
                let factor = match self.fluence {
                    FluenceFactor::Documented { divisor } => (gamma / divisor).exp(),
                    _ => 1.0,
                };
                Some(Diffusivity {
                    m2_per_s: d0.value
                        * factor
                        * (-q_kj_mol.value * 1e3 / (r_gas * t_kelvin)).exp(),
                    kind: *d0_kind,
                })
            }
        }
    }
}

fn parse_law(row: &Row<'_>, form: Form, evidence: EvidenceClass) -> Result<(Law, BoundKind)> {
    let d0 = row.opt_number("D0_m2_s")?;
    let q = row.opt_number("Q_kJ_mol")?;
    let bound = match (
        form == Form::UpperBound,
        evidence == EvidenceClass::LiteratureCorrelationUpperBound,
    ) {
        (true, true) => BoundKind::UpperBound,
        (false, false) => BoundKind::PointValue,
        _ => {
            return Err(row.err(
                Some("form"),
                "the form and the evidence class disagree on whether D0 is an upper bound",
            ))
        }
    };
    let law = match form {
        Form::Instantaneous => {
            if d0.is_some() || q.is_some() {
                return Err(row.err(
                    Some("D0_m2_s"),
                    "an instantaneous release has no D0 and no Q",
                ));
            }
            Law::Instantaneous
        }
        Form::Constant | Form::ConstantQZero | Form::TemperatureIndependent => {
            let d = d0.ok_or_else(|| row.err(Some("D0_m2_s"), "a constant needs its value"))?;
            if q.as_ref().is_some_and(|q| q.value != 0.0) {
                return Err(row.err(
                    Some("Q_kJ_mol"),
                    "a constant coefficient has no activation energy (Q must be empty or 0)",
                ));
            }
            Law::Constant { d, q_kj_mol: q }
        }
        _ => {
            let d0 = d0.ok_or_else(|| row.err(Some("D0_m2_s"), "an Arrhenius term needs D0"))?;
            let q = q.ok_or_else(|| row.err(Some("Q_kJ_mol"), "an Arrhenius term needs Q"))?;
            Law::Arrhenius {
                d0,
                d0_kind: bound,
                q_kj_mol: q,
            }
        }
    };
    Ok((law, bound))
}

fn parse_fluence(row: &Row<'_>, form: Form, expression: &str) -> Result<FluenceFactor> {
    if expression.starts_with("fluence factor exp(Gamma/5) NOT used") {
        return Ok(FluenceFactor::DocumentedNotUsed);
    }
    let documented = expression.contains("*exp(Gamma/5)");
    if form == Form::TwoTermFluenceDependent && !documented {
        return Err(row.err(
            Some("D0_expression"),
            "the form says the term is fluence dependent but the expression has no exp(Gamma/5) factor",
        ));
    }
    Ok(if documented {
        FluenceFactor::Documented { divisor: 5.0 }
    } else {
        FluenceFactor::Absent
    })
}

fn parse_reduced(
    row: &Row<'_>,
    form: Form,
    expression: &str,
) -> Result<Option<ReducedCoefficient>> {
    if form != Form::Reduced {
        return Ok(None);
    }
    let rest = expression.strip_prefix("D'0 = ").ok_or_else(|| {
        row.err(
            Some("D0_expression"),
            "a reduced coefficient must print `D'0 = <value> s^-1`",
        )
    })?;
    let num = rest.split(' ').next().unwrap_or("");
    let p = Printed::parse(num).map_err(|m| row.err(Some("D0_expression"), m))?;
    if !rest[num.len()..].starts_with(" s^-1") {
        return Err(row.err(Some("D0_expression"), "the unit of D'0 must be s^-1"));
    }
    Ok(Some(ReducedCoefficient { d_prime_per_s: p }))
}

fn read_row(row: Row<'_>) -> Result<ArrheniusRow> {
    let form: Form = row.vocab("form")?;
    let evidence: EvidenceClass = row.vocab("evidence_class")?;
    let (law, _) = parse_law(&row, form, evidence)?;
    let expression = row.text("D0_expression")?.to_string();
    let fluence = parse_fluence(&row, form, &expression)?;
    let reduced = parse_reduced(&row, form, &expression)?;
    let term = row.uint("term")?;
    if term != 1 && term != 2 {
        return Err(row.err(Some("term"), "the term number must be 1 or 2"));
    }
    let species_text = row.required("nuclide")?.to_string();
    let species = Species::parse(&species_text).map_err(|m| row.err(Some("nuclide"), m))?;
    let r = row.required("R_J_mol_K")?;
    let gas_constant = if r == "not printed" {
        GasConstant::NotPrinted
    } else {
        GasConstant::Printed(Printed::parse(r).map_err(|m| row.err(Some("R_J_mol_K"), m))?)
    };
    let t_min = row.opt_number("T_min_C")?;
    let t_max = row.opt_number("T_max_C")?;
    let range_kind = match row.text("range_kind")? {
        "" => None,
        s => Some(RangeKind::from_cell(s).ok_or_else(|| {
            row.err(
                Some("range_kind"),
                "the range kind must be `printed` or `fitted`, or empty",
            )
        })?),
    };
    if range_kind.is_some() != (t_min.is_some() || t_max.is_some()) {
        return Err(row.err(
            Some("range_kind"),
            "the range kind is given exactly when T_min_C or T_max_C is given",
        ));
    }
    Ok(ArrheniusRow {
        line: row.line(),
        id: row.required("id")?.to_string(),
        set_id: row.required("set_id")?.to_string(),
        species,
        species_text,
        layer: row.vocab("layer")?,
        material: row.required("material")?.to_string(),
        origin: row.required("origin")?.to_string(),
        term: term as u8,
        law,
        expression,
        fluence,
        reduced,
        form,
        t_min_c: t_min,
        t_max_c: t_max,
        range_kind,
        range_note: row.vocab("T_range_note")?,
        gas_constant,
        data_class: row.required("data_class")?.to_string(),
        evidence_class: evidence,
        ref_in_source: row.text("ref_in_source")?.to_string(),
        source: row.required("source")?.to_string(),
        source_location: row.required("source_location")?.to_string(),
        link: row.required("link")?.to_string(),
        status: row.vocab("status")?,
        notes: row.text("notes")?.to_string(),
        coefficient_class: row.vocab("coefficient_class")?,
        phase_of_use: row.vocab("phase_of_use")?,
        reference_radius_um: row.opt_number("reference_radius_um")?,
        fluence_definition: row.text("fluence_definition")?.to_string(),
        fitted_to: row.text("fitted_to")?.to_string(),
    })
}

/// All rows of `arrhenius.csv`.
#[derive(Debug, Clone)]
pub struct Arrhenius {
    /// The rows in file order.
    pub rows: Vec<ArrheniusRow>,
}

impl Arrhenius {
    /// Read and validate the file text.
    pub fn parse(text: &str) -> Result<Arrhenius> {
        let table = Table::parse(FILE, text, &HEADER)?;
        let rows = table.rows().map(read_row).collect::<Result<Vec<_>>>()?;
        let mut seen = std::collections::HashSet::new();
        for r in &rows {
            if !seen.insert(r.id.as_str()) {
                return Err(DataError::line(
                    FILE,
                    r.line,
                    format!("the id {} appears twice", r.id),
                ));
            }
        }
        Ok(Arrhenius { rows })
    }

    /// The row with identifier `id`.
    pub fn get(&self, id: &str) -> Option<&ArrheniusRow> {
        self.rows.iter().find(|r| r.id == id)
    }

    /// The coefficient of the rows `ids` added (a two-term coefficient is the sum of its terms),
    /// at `t_celsius`, gas constant `r_gas` and fluence `gamma`. It is an upper bound when any
    /// term is. `None` when an id is unknown or a term is instantaneous.
    pub fn sum_of_terms(
        &self,
        ids: &[&str],
        t_celsius: f64,
        r_gas: f64,
        gamma: f64,
    ) -> Option<Diffusivity> {
        let mut total = 0.0;
        let mut kind = BoundKind::PointValue;
        for id in ids {
            let d = self
                .get(id)?
                .diffusivity(t_celsius + 273.15, r_gas, gamma)?;
            total += d.m2_per_s;
            if d.kind == BoundKind::UpperBound {
                kind = BoundKind::UpperBound;
            }
        }
        Some(Diffusivity {
            m2_per_s: total,
            kind,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn species_labels() {
        assert_eq!(Species::parse("all species"), Ok(Species::AllSpecies));
        match Species::parse("Cs (upper curve)").unwrap() {
            Species::Named { items, qualifier } => {
                assert_eq!(items, vec!["Cs"]);
                assert_eq!(qualifier.as_deref(), Some("upper curve"));
            }
            other => panic!("{other:?}"),
        }
        match Species::parse("Kr/Xe/I").unwrap() {
            Species::Named { items, .. } => assert_eq!(items, vec!["Kr", "Xe", "I"]),
            other => panic!("{other:?}"),
        }
        assert!(Species::parse("cs").is_err());
        assert!(Species::parse("Cs (open").is_err());
    }
}
