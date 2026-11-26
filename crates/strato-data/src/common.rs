//! Types shared by several register files: status, evidence class, nuclide names, and the
//! product class that separates a direct fission product from an activation product.

use crate::string_enum;

string_enum! {
    /// Whether a row was checked against its source (register Section 0.2). It says nothing
    /// about what kind of value the row holds: that is the [`EvidenceClass`].
    pub enum Status ("status") {
        /// Checked against the cited page or library entry.
        Verified => "verified",
        /// Read from a raster figure.
        Digitised => "digitised",
        /// Not checked; the register says so.
        NotVerified => "NOT VERIFIED",
    }
}

string_enum! {
    /// What kind of value a row holds (register Section 0.2; column `evidence_class`).
    pub enum EvidenceClass ("evidence_class") {
        /// A measurement.
        Measured => "measured",
        /// A measurement read from a figure.
        MeasuredDigitised => "measured, digitised",
        /// A measured bound, for example a detection limit or an upper limit.
        MeasuredBound => "measured bound",
        /// A published correlation or fit.
        LiteratureCorrelation => "literature correlation",
        /// A published correlation that is only an upper bound.
        LiteratureCorrelationUpperBound => "literature correlation (upper bound)",
        /// A published value at one temperature only.
        LiteratureValueSingleTemperature => "literature value (single temperature)",
        /// A convention chosen for modelling, not a measurement or an evaluation.
        ModellingConvention => "modelling convention",
        /// A design or test specification.
        Specification => "specification",
        /// A benchmark case that is synthetic by construction.
        SyntheticBenchmark => "synthetic benchmark",
        /// Computed from other register values.
        Calculated => "calculated",
        /// An estimate (for example an inference from post-irradiation examination).
        Estimate => "estimate",
        /// An evaluated nuclear data library value.
        EvaluatedNuclearData => "evaluated nuclear data",
        /// The register does not state the basis.
        BasisNotStated => "basis not stated",
    }
}

impl EvidenceClass {
    /// True for the one class that is evaluated library data, as opposed to a modelling
    /// assumption (`ModellingConvention`) or a calculation (`Calculated`).
    pub fn is_evaluated_data(&self) -> bool {
        matches!(self, EvidenceClass::EvaluatedNuclearData)
    }

    /// True for the class that is a modelling assumption and not data.
    pub fn is_modelling_assumption(&self) -> bool {
        matches!(self, EvidenceClass::ModellingConvention)
    }
}

/// A nuclide name such as `Cs-137` or `Ag-110m`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NuclideId {
    /// Element symbol.
    pub element: String,
    /// Mass number.
    pub mass: u32,
    /// Metastable suffix as printed (`m`), empty for the ground state.
    pub isomer: String,
}

impl NuclideId {
    /// Parse `El-A` or `El-Am`.
    pub fn parse(text: &str) -> Result<NuclideId, String> {
        let (el, rest) = text
            .split_once('-')
            .ok_or_else(|| format!("`{text}` is not of the form Element-Mass"))?;
        let mut ec = el.chars();
        let ok_el = match (ec.next(), ec.next(), ec.next()) {
            (Some(a), None, _) => a.is_ascii_uppercase(),
            (Some(a), Some(b), None) => a.is_ascii_uppercase() && b.is_ascii_lowercase(),
            _ => false,
        };
        if !ok_el {
            return Err(format!("`{el}` is not an element symbol"));
        }
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        let isomer: String = rest.chars().skip(digits.len()).collect();
        let mass = digits
            .parse::<u32>()
            .map_err(|_| format!("`{text}` has no mass number"))?;
        if !(isomer.is_empty() || isomer.chars().all(|c| c.is_ascii_lowercase())) {
            return Err(format!("`{isomer}` is not an isomer suffix"));
        }
        Ok(NuclideId {
            element: el.to_string(),
            mass,
            isomer,
        })
    }

    /// The name as printed.
    pub fn name(&self) -> String {
        format!("{}-{}{}", self.element, self.mass, self.isomer)
    }
}

/// What a nuclide is with respect to fission, from the `product_class` column of
/// `decay.csv` and `fission_yields.csv`. This is the type that stops an activation product
/// from receiving a fission-yield source term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductKind {
    /// A fission product that may be fed by its cumulative yield (`fission product (cumulative-yield source)`).
    DirectFissionProduct,
    /// A short-lived fission precursor of another product.
    ShortLivedFissionPrecursor,
    /// A short-lived isomer of a fission product.
    ShortLivedFissionIsomer,
    /// A stable fission product that is also the target of an activation reaction.
    StableFissionProduct,
    /// Made by neutron capture (or decay of such a nuclide) and fission-shielded: no fission-yield source.
    ActivationProduct {
        /// The row is an isomer or ground-state partner (`(isomer)`).
        isomer: bool,
    },
    /// A decay daughter of another nuclide.
    DecayDaughter {
        /// The parent named in the text, when it names one.
        parent: Option<String>,
        /// The daughter is stable.
        stable: bool,
        /// The daughter is an isomer.
        isomer: bool,
    },
}

/// The parsed `product_class` cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductClass {
    /// The kind.
    pub kind: ProductKind,
    /// The text after the first `; `, as printed.
    pub note: Option<String>,
    /// The whole cell as printed.
    pub raw: String,
    /// The register states "NO fission-yield source in STRATO" in this cell.
    pub states_no_fission_yield_source: bool,
}

impl ProductClass {
    /// Parse a `product_class` cell; any text outside the known vocabulary is an error.
    pub fn parse(raw: &str) -> Result<ProductClass, String> {
        let (head, note) = match raw.split_once("; ") {
            Some((h, n)) => (h, Some(n.to_string())),
            None => (raw, None),
        };
        let kind = match head {
            "fission product (cumulative-yield source)" => ProductKind::DirectFissionProduct,
            "short-lived fission precursor" => ProductKind::ShortLivedFissionPrecursor,
            "short-lived isomer of a fission product" => ProductKind::ShortLivedFissionIsomer,
            "stable fission product (activation target)" => ProductKind::StableFissionProduct,
            "activation product (fission-shielded)" => {
                ProductKind::ActivationProduct { isomer: false }
            }
            "activation product (fission-shielded) (isomer)" => {
                ProductKind::ActivationProduct { isomer: true }
            }
            "decay daughter (stable)" => ProductKind::DecayDaughter {
                parent: None,
                stable: true,
                isomer: false,
            },
            "decay daughter (isomer)" => ProductKind::DecayDaughter {
                parent: None,
                stable: false,
                isomer: true,
            },
            other => match other.strip_prefix("decay daughter of ") {
                Some(p) if !p.is_empty() => ProductKind::DecayDaughter {
                    parent: Some(p.to_string()),
                    stable: false,
                    isomer: false,
                },
                _ => return Err(format!("`{raw}` is not a known product class")),
            },
        };
        let states_no_fission_yield_source = note
            .as_deref()
            .is_some_and(|n| n.contains("NO fission-yield source in STRATO"));
        Ok(ProductClass {
            kind,
            note,
            raw: raw.to_string(),
            states_no_fission_yield_source,
        })
    }

    /// True for an activation product: no fission-yield source term may be built for it.
    pub fn is_activation_product(&self) -> bool {
        matches!(self.kind, ProductKind::ActivationProduct { .. })
    }

    /// True for a direct fission product, the only class that may take a cumulative-yield source.
    pub fn is_direct_fission_product(&self) -> bool {
        matches!(self.kind, ProductKind::DirectFissionProduct)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::Vocabulary;

    #[test]
    fn nuclide_names() {
        let n = NuclideId::parse("Ag-110m").unwrap();
        assert_eq!(
            (n.element.as_str(), n.mass, n.isomer.as_str()),
            ("Ag", 110, "m")
        );
        assert_eq!(n.name(), "Ag-110m");
        assert!(NuclideId::parse("Cs137").is_err());
        assert!(NuclideId::parse("cs-137").is_err());
        assert!(NuclideId::parse("Cs-").is_err());
    }

    #[test]
    fn product_classes_separate_activation_from_fission() {
        let a = ProductClass::parse(
            "activation product (fission-shielded); NO fission-yield source in STRATO",
        )
        .unwrap();
        assert!(a.is_activation_product() && !a.is_direct_fission_product());
        assert!(a.states_no_fission_yield_source);
        let f =
            ProductClass::parse("fission product (cumulative-yield source); use ground-state CFY")
                .unwrap();
        assert!(f.is_direct_fission_product());
        assert_eq!(f.note.as_deref(), Some("use ground-state CFY"));
        let d = ProductClass::parse("decay daughter of Sr-90").unwrap();
        assert!(
            matches!(d.kind, ProductKind::DecayDaughter { parent: Some(ref p), .. } if p == "Sr-90")
        );
        assert!(ProductClass::parse("fission-ish").is_err());
    }

    #[test]
    fn status_and_evidence_vocabularies_are_closed() {
        assert_eq!(Status::from_cell("NOT VERIFIED"), Some(Status::NotVerified));
        assert_eq!(Status::from_cell("Verified"), None);
        assert_eq!(EvidenceClass::ALL.len(), 13);
        assert!(EvidenceClass::EvaluatedNuclearData.is_evaluated_data());
        assert!(EvidenceClass::ModellingConvention.is_modelling_assumption());
    }
}
