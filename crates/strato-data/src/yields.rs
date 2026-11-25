//! `data/nuclear/fission_yields.csv`: JEFF-3.1.1 independent and cumulative thermal yields.
//!
//! Independent and cumulative yields are different types ([`YieldValue<Independent>`] and
//! [`YieldValue<Cumulative>`]) and cannot be exchanged by accident. A yield the library leaves
//! blank is [`JeffYield::BlankInApi`], never zero. The only way to get a source-term yield out of
//! a row is [`YieldRow::fission_source_yield`], which refuses every row that is not a direct
//! fission product, in particular the activation products Ag-110m, Cs-134 and Eu-154.

use std::marker::PhantomData;

use crate::common::{EvidenceClass, NuclideId, ProductClass, Status};
use crate::error::{DataError, Result};
use crate::string_enum;
use crate::table::{Printed, Row, Table, Vocabulary};

/// Path of the file below the data folder.
pub const FILE: &str = "nuclear/fission_yields.csv";

/// The exact header.
pub const HEADER: [&str; 24] = [
    "parent",
    "neutron_energy",
    "product",
    "Z_element",
    "A",
    "LiveChart_level_idx",
    "IFY_JEFF311",
    "unc_IFY_JEFF311",
    "CFY_JEFF311",
    "unc_CFY_JEFF311",
    "IFY_ENDFB_NuDat3",
    "unc_IFY_ENDFB_NuDat3",
    "CFY_ENDFB_NuDat3",
    "unc_CFY_ENDFB_NuDat3",
    "units",
    "STRATO_use_note",
    "data_type",
    "evidence_class",
    "source_JEFF",
    "source_ENDF",
    "product_class",
    "source_location",
    "status",
    "status_ENDFB_crosscheck",
];

/// Marker: an independent yield (the product made directly in fission).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Independent;

/// Marker: a cumulative yield (the product made directly plus all decay precursors).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cumulative;

/// A yield in atoms per fission with its uncertainty; `K` says whether it is independent or cumulative.
#[derive(Debug, Clone, PartialEq)]
pub struct YieldValue<K> {
    /// The yield.
    pub value: Printed,
    /// Its uncertainty.
    pub unc: Printed,
    kind: PhantomData<fn() -> K>,
}

impl<K> YieldValue<K> {
    fn new(value: Printed, unc: Printed) -> Self {
        YieldValue {
            value,
            unc,
            kind: PhantomData,
        }
    }
}

/// A JEFF-3.1.1 yield cell pair.
#[derive(Debug, Clone, PartialEq)]
pub enum JeffYield<K> {
    /// A value with its uncertainty.
    Reported(YieldValue<K>),
    /// The LiveChart API returned no value (`BLANK IN API`): unknown, not zero.
    BlankInApi,
}

impl<K> JeffYield<K> {
    /// The yield, if the library has one.
    pub fn reported(&self) -> Option<&YieldValue<K>> {
        match self {
            JeffYield::Reported(y) => Some(y),
            JeffYield::BlankInApi => None,
        }
    }
}

/// An ENDF/B-VIII (NuDat 3) cross-check cell pair.
#[derive(Debug, Clone, PartialEq)]
pub enum EndfYield<K> {
    /// A value with its uncertainty.
    Reported(YieldValue<K>),
    /// The cross-check has no entry for this row.
    NotInCrossCheck,
}

string_enum! {
    /// The fissioning nuclide.
    pub enum Parent ("fission parent") {
        /// U-235.
        U235 => "235U",
        /// Pu-239.
        Pu239 => "239Pu",
    }
}

string_enum! {
    /// The neutron energy of the yield set.
    pub enum NeutronEnergy ("neutron energy") {
        /// Thermal.
        Thermal => "thermal",
    }
}

string_enum! {
    /// The `units` column.
    pub enum YieldUnits ("yield units") {
        /// Atoms per fission; the independent yields of a parent sum to 2.
        AtomsPerFission => "atoms per fission (sum of IFY = 2)",
    }
}

string_enum! {
    /// The `status_ENDFB_crosscheck` column.
    pub enum EndfStatus ("ENDF cross-check status") {
        /// Verified as a cross-check only.
        VerifiedCrossCheckOnly => "verified (cross-check only)",
        /// No ENDF value given.
        NotGiven => "not given",
        /// The isomer state assignment of the ENDF value is unclear.
        NotVerifiedIsomerStateUnclear => "NOT VERIFIED (isomer state assignment unclear)",
    }
}

/// Why a row cannot give a fission source term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceTermRefusal {
    /// The product is an activation product (Ag-110m, Cs-134, Eu-154 and their partners).
    ActivationProduct,
    /// The use note of the register forbids it.
    UseNoteForbids,
    /// The product class is not a direct fission product.
    NotADirectFissionProduct(String),
    /// JEFF has no cumulative yield for the row.
    NoCumulativeYield,
}

/// One row of `fission_yields.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct YieldRow {
    /// Line in the file.
    pub line: usize,
    /// Fissioning nuclide.
    pub parent: Parent,
    /// Neutron energy.
    pub energy: NeutronEnergy,
    /// Product.
    pub product: NuclideId,
    /// Element symbol of the product.
    pub z_element: String,
    /// Mass number of the product.
    pub a: u32,
    /// The LiveChart level index (0 ground state; 1, 2, 3, 13 isomers; -2 an unresolved level, GAP-27).
    pub level_idx: i32,
    /// JEFF-3.1.1 independent yield.
    pub jeff_independent: JeffYield<Independent>,
    /// JEFF-3.1.1 cumulative yield.
    pub jeff_cumulative: JeffYield<Cumulative>,
    /// ENDF/B independent yield.
    pub endf_independent: EndfYield<Independent>,
    /// ENDF/B cumulative yield.
    pub endf_cumulative: EndfYield<Cumulative>,
    /// Units.
    pub units: YieldUnits,
    /// The use note as printed.
    pub use_note: String,
    /// The note begins `do NOT use as a fission-yield source`.
    pub use_note_forbids_source: bool,
    /// Data type as printed.
    pub data_type: String,
    /// Evidence class.
    pub evidence_class: EvidenceClass,
    /// JEFF source.
    pub source_jeff: String,
    /// ENDF source (empty when there is no cross-check).
    pub source_endf: String,
    /// Product class.
    pub product_class: ProductClass,
    /// Location in the source.
    pub source_location: String,
    /// Status of the JEFF value.
    pub status: Status,
    /// Status of the ENDF cross-check.
    pub endf_status: EndfStatus,
}

impl YieldRow {
    /// The cumulative yield that may drive a source term for this product, or the reason it may not.
    pub fn fission_source_yield(
        &self,
    ) -> std::result::Result<&YieldValue<Cumulative>, SourceTermRefusal> {
        if self.product_class.is_activation_product() {
            return Err(SourceTermRefusal::ActivationProduct);
        }
        if self.use_note_forbids_source {
            return Err(SourceTermRefusal::UseNoteForbids);
        }
        if !self.product_class.is_direct_fission_product() {
            return Err(SourceTermRefusal::NotADirectFissionProduct(
                self.product_class.raw.clone(),
            ));
        }
        self.jeff_cumulative
            .reported()
            .ok_or(SourceTermRefusal::NoCumulativeYield)
    }
}

const BLANK: &str = "BLANK IN API";

fn jeff<K>(row: &Row<'_>, v: &str, u: &str) -> Result<JeffYield<K>> {
    let (vt, ut) = (row.required(v)?, row.required(u)?);
    if vt == BLANK || ut == BLANK {
        if vt != ut {
            return Err(row.err(
                Some(v),
                "the value and its uncertainty must both be `BLANK IN API` or both numbers",
            ));
        }
        return Ok(JeffYield::BlankInApi);
    }
    Ok(JeffYield::Reported(YieldValue::new(
        row.number(v)?,
        row.number(u)?,
    )))
}

fn endf<K>(row: &Row<'_>, v: &str, u: &str) -> Result<EndfYield<K>> {
    match (row.optional(v)?, row.optional(u)?) {
        (None, None) => Ok(EndfYield::NotInCrossCheck),
        (Some(_), Some(_)) => Ok(EndfYield::Reported(YieldValue::new(
            row.number(v)?,
            row.number(u)?,
        ))),
        _ => Err(row.err(
            Some(v),
            "the ENDF value and its uncertainty must be given together or not at all",
        )),
    }
}

fn read_row(row: Row<'_>) -> Result<YieldRow> {
    let product =
        NuclideId::parse(row.required("product")?).map_err(|m| row.err(Some("product"), m))?;
    let a = row.uint("A")?;
    if a != product.mass {
        return Err(row.err(
            Some("A"),
            format!("A = {a} but the product name says mass {}", product.mass),
        ));
    }
    let z_element = row.required("Z_element")?.to_string();
    if z_element != product.element {
        return Err(row.err(
            Some("Z_element"),
            "the element does not match the product name",
        ));
    }
    let note = row.required("STRATO_use_note")?;
    let source_endf = row.text("source_ENDF")?.to_string();
    let endf_status: EndfStatus = row.vocab("status_ENDFB_crosscheck")?;
    let r = YieldRow {
        line: row.line(),
        parent: row.vocab("parent")?,
        energy: row.vocab("neutron_energy")?,
        product,
        z_element,
        a,
        level_idx: row.int("LiveChart_level_idx")?,
        jeff_independent: jeff(&row, "IFY_JEFF311", "unc_IFY_JEFF311")?,
        jeff_cumulative: jeff(&row, "CFY_JEFF311", "unc_CFY_JEFF311")?,
        endf_independent: endf(&row, "IFY_ENDFB_NuDat3", "unc_IFY_ENDFB_NuDat3")?,
        endf_cumulative: endf(&row, "CFY_ENDFB_NuDat3", "unc_CFY_ENDFB_NuDat3")?,
        units: row.vocab("units")?,
        use_note: note.to_string(),
        use_note_forbids_source: note.starts_with("do NOT use as a fission-yield source"),
        data_type: row.required("data_type")?.to_string(),
        evidence_class: row.vocab("evidence_class")?,
        source_jeff: row.required("source_JEFF")?.to_string(),
        source_endf,
        product_class: ProductClass::parse(row.required("product_class")?)
            .map_err(|m| row.err(Some("product_class"), m))?,
        source_location: row.required("source_location")?.to_string(),
        status: row.vocab("status")?,
        endf_status,
    };
    Ok(r)
}

/// All rows of `fission_yields.csv`.
#[derive(Debug, Clone)]
pub struct Yields {
    /// The rows in file order.
    pub rows: Vec<YieldRow>,
}

impl Yields {
    /// Read and validate the file text. The key (parent, energy, product, level index) must be unique.
    pub fn parse(text: &str) -> Result<Yields> {
        let table = Table::parse(FILE, text, &HEADER)?;
        let rows = table.rows().map(read_row).collect::<Result<Vec<_>>>()?;
        let mut seen = std::collections::HashSet::new();
        for r in &rows {
            let key = (r.parent, r.energy, r.product.name(), r.level_idx);
            if !seen.insert(key) {
                return Err(DataError::line(
                    FILE,
                    r.line,
                    format!(
                        "the key ({}, {}, level {}) appears twice",
                        r.parent.as_str(),
                        r.product.name(),
                        r.level_idx
                    ),
                ));
            }
        }
        Ok(Yields { rows })
    }

    /// The row of a parent and product (first level index in file order).
    pub fn get(&self, parent: Parent, product: &str) -> Option<&YieldRow> {
        self.rows
            .iter()
            .find(|r| r.parent == parent && r.product.name() == product)
    }
}
