//! `data/geometry/geometry.csv`: particle and compact geometry.
//!
//! One row is one printed quantity of one design (a layer thickness, a density, a count). The
//! typed reader keeps all 21 columns. A value is a point, a range, an upper bound or a text,
//! never a number that silently stands for another kind.

use crate::common::{EvidenceClass, Status};
use crate::error::{DataError, Result};
use crate::string_enum;
use crate::table::{Printed, Row, Table, Vocabulary};

/// Path of the file below the data folder.
pub const FILE: &str = "geometry/geometry.csv";

/// The exact header.
pub const HEADER: [&str; 21] = [
    "id",
    "design_id",
    "design",
    "layer_index",
    "layer",
    "quantity",
    "symbol",
    "value",
    "value_min",
    "value_max",
    "unit",
    "uncertainty",
    "material",
    "outer_radius_um_calc",
    "data_class",
    "evidence_class",
    "source",
    "source_location",
    "link",
    "status",
    "notes",
];

string_enum! {
    /// The `layer` column: a coating layer or the object the row describes.
    pub enum GeometryLayer ("geometry layer") {
        /// Kernel (layer index 1).
        Kernel => "kernel",
        /// Porous carbon buffer (2).
        Buffer => "buffer",
        /// Inner pyrocarbon (3).
        IPyC => "IPyC",
        /// Silicon carbide (4).
        SiC => "SiC",
        /// Outer pyrocarbon (5).
        OPyC => "OPyC",
        /// The compact (index 0).
        Compact => "compact",
        /// The whole particle (index 0).
        Particle => "particle",
        /// An element of the kernel composition (index 0).
        Element => "element",
        /// Matrix graphite (index 6).
        MatrixGraphite => "matrix graphite",
    }
}

impl GeometryLayer {
    /// The layer index the register prints for this layer.
    pub fn expected_index(&self) -> u8 {
        match self {
            GeometryLayer::Kernel => 1,
            GeometryLayer::Buffer => 2,
            GeometryLayer::IPyC => 3,
            GeometryLayer::SiC => 4,
            GeometryLayer::OPyC => 5,
            GeometryLayer::MatrixGraphite => 6,
            GeometryLayer::Compact | GeometryLayer::Particle | GeometryLayer::Element => 0,
        }
    }
}

string_enum! {
    /// The `quantity` column.
    pub enum GeometryQuantity ("geometry quantity") {
        /// Layer thickness.
        Thickness => "thickness",
        /// Specified layer thickness.
        ThicknessSpecified => "thickness (specified)",
        /// Kernel diameter.
        Diameter => "diameter",
        /// Specified kernel diameter.
        DiameterSpecified => "diameter (specified)",
        /// Density.
        Density => "density",
        /// U-235 enrichment.
        Enrichment => "U-235 enrichment",
        /// Particles per compact.
        ParticlesPerCompact => "particles per compact",
        /// Particles per sphere.
        ParticlesPerSphere => "particles per sphere",
        /// Defective SiC fraction.
        DefectiveSicFraction => "defective SiC fraction",
        /// Defective OPyC fraction.
        DefectiveOpycFraction => "defective OPyC fraction",
        /// Uranium fraction in a layer.
        UraniumFractionInLayer => "uranium fraction in layer",
        /// Uranium fraction in the matrix.
        UraniumFractionInMatrix => "uranium fraction in matrix",
        /// Nominal particle diameter.
        NominalParticleDiameter => "nominal particle diameter",
        /// U-235 content per particle.
        U235ContentPerParticle => "U-235 content per particle",
        /// Compact length.
        CompactLength => "compact length",
        /// Compact diameter.
        CompactDiameter => "compact diameter",
        /// Uranium contamination, an upper bound.
        UContaminationUpperBound => "U contamination (upper bound)",
        /// Packing fraction.
        PackingFraction => "packing fraction",
        /// Mean uranium loading per compact.
        MeanULoadingPerCompact => "mean U loading per compact",
        /// Matrix composition.
        MatrixComposition => "matrix composition natural/synthetic graphite/resin",
        /// Compact-to-holder diametral gap.
        CompactToHolderGap => "compact-to-holder diametral gap",
    }
}

string_enum! {
    /// The `unit` column.
    pub enum GeometryUnit ("geometry unit") {
        /// Micrometre.
        Micrometre => "um",
        /// Millimetre.
        Millimetre => "mm",
        /// Dimensionless.
        One => "1",
        /// Megagram per cubic metre.
        MgPerM3 => "Mg m^-3",
        /// Percent.
        Percent => "%",
        /// Percent by weight.
        WeightPercent => "wt%",
        /// Gram.
        Gram => "g",
    }
}

/// What the `value`, `value_min` and `value_max` cells hold together.
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryValue {
    /// A single printed number.
    Point(Printed),
    /// A printed value with the range the specification allows around it (`value`, `value_min`, `value_max`).
    PointWithRange {
        /// The specified value.
        value: Printed,
        /// Lower end of the specified range.
        min: Printed,
        /// Upper end of the specified range.
        max: Printed,
    },
    /// A printed range (`value_min` and `value_max`, no `value`).
    Range {
        /// Lower end.
        min: Printed,
        /// Upper end.
        max: Printed,
    },
    /// An upper bound (`value_max` only, with the uncertainty `upper bound`).
    UpperBound(Printed),
    /// A printed text that is not a number (the matrix composition `64/16/20`).
    Text(String),
}

/// The `uncertainty` column.
#[derive(Debug, Clone, PartialEq)]
pub enum Uncertainty {
    /// `none printed`.
    NonePrinted,
    /// `none`.
    NoneStated,
    /// `none (benchmark definition)`.
    NoneBenchmarkDefinition,
    /// `not in audit notes`.
    NotInAuditNotes,
    /// `SD not in audit notes`.
    SdNotInAuditNotes,
    /// `approximate (printed with ~)`.
    ApproximatePrintedWithTilde,
    /// `approximate (printed 'about')`.
    ApproximatePrintedAbout,
    /// `nominal`.
    Nominal,
    /// `range`.
    Range,
    /// `range over variants`.
    RangeOverVariants,
    /// `upper bound`.
    UpperBound,
    /// `SD x`.
    StandardDeviation(f64),
    /// `+/- x` or `+/- x (specified range)`.
    PlusMinus {
        /// The half width.
        half_width: f64,
        /// The text says `(specified range)`.
        specified_range: bool,
    },
    /// A footnote of the source, kept as printed.
    Footnote(String),
}

impl Uncertainty {
    /// Parse the cell; text outside the known forms is an error.
    pub fn parse(text: &str) -> std::result::Result<Uncertainty, String> {
        let fixed = match text {
            "none printed" => Some(Uncertainty::NonePrinted),
            "none" => Some(Uncertainty::NoneStated),
            "none (benchmark definition)" => Some(Uncertainty::NoneBenchmarkDefinition),
            "not in audit notes" => Some(Uncertainty::NotInAuditNotes),
            "SD not in audit notes" => Some(Uncertainty::SdNotInAuditNotes),
            "approximate (printed with ~)" => Some(Uncertainty::ApproximatePrintedWithTilde),
            "approximate (printed 'about')" => Some(Uncertainty::ApproximatePrintedAbout),
            "nominal" => Some(Uncertainty::Nominal),
            "range" => Some(Uncertainty::Range),
            "range over variants" => Some(Uncertainty::RangeOverVariants),
            "upper bound" => Some(Uncertainty::UpperBound),
            _ => None,
        };
        if let Some(u) = fixed {
            return Ok(u);
        }
        if let Some(rest) = text.strip_prefix("SD ") {
            return Printed::parse(rest).map(|p| Uncertainty::StandardDeviation(p.value));
        }
        if let Some(rest) = text.strip_prefix("+/- ") {
            let (num, specified_range) = match rest.strip_suffix(" (specified range)") {
                Some(n) => (n, true),
                None => (rest, false),
            };
            return Printed::parse(num).map(|p| Uncertainty::PlusMinus {
                half_width: p.value,
                specified_range,
            });
        }
        if text.starts_with("footnote (") {
            return Ok(Uncertainty::Footnote(text.to_string()));
        }
        Err(format!("`{text}` is not a known form of uncertainty"))
    }
}

/// One row of `geometry.csv`.
#[derive(Debug, Clone, PartialEq)]
pub struct GeometryRow {
    /// Line in the file.
    pub line: usize,
    /// Row identifier (`G-01`).
    pub id: String,
    /// Design identifier (`AGR1-B-M16`).
    pub design_id: String,
    /// Design description.
    pub design: String,
    /// Layer index as printed (0 when the row is not a layer).
    pub layer_index: u8,
    /// Layer or object.
    pub layer: GeometryLayer,
    /// Quantity.
    pub quantity: GeometryQuantity,
    /// Symbol as printed.
    pub symbol: String,
    /// The value.
    pub value: GeometryValue,
    /// Unit.
    pub unit: GeometryUnit,
    /// Uncertainty, parsed.
    pub uncertainty: Uncertainty,
    /// Uncertainty, as printed.
    pub uncertainty_text: String,
    /// Material as printed.
    pub material: String,
    /// Outer radius calculated by the register, micrometre; present on layer rows.
    pub outer_radius_um_calc: Option<Printed>,
    /// Data class as printed.
    pub data_class: String,
    /// Evidence class.
    pub evidence_class: EvidenceClass,
    /// Short citation.
    pub source: String,
    /// Location in the source.
    pub source_location: String,
    /// Link as printed.
    pub link: String,
    /// Status.
    pub status: Status,
    /// Notes.
    pub notes: String,
}

fn read_row(row: Row<'_>) -> Result<GeometryRow> {
    let value = match (
        row.optional("value")?,
        row.optional("value_min")?,
        row.optional("value_max")?,
    ) {
        (Some(v), None, None) => match Printed::parse(v) {
            Ok(p) => GeometryValue::Point(p),
            Err(_) => GeometryValue::Text(v.to_string()),
        },
        (Some(_), Some(_), Some(_)) => GeometryValue::PointWithRange {
            value: row.number("value")?,
            min: row.number("value_min")?,
            max: row.number("value_max")?,
        },
        (None, Some(_), Some(_)) => GeometryValue::Range {
            min: row.number("value_min")?,
            max: row.number("value_max")?,
        },
        (None, None, Some(_)) => {
            if row.text("uncertainty")? != "upper bound" {
                return Err(row.err(
                    Some("value_max"),
                    "only value_max is given, which the register uses for an upper bound, but the uncertainty does not say `upper bound`",
                ));
            }
            GeometryValue::UpperBound(row.number("value_max")?)
        }
        other => {
            return Err(row.err(
                Some("value"),
                format!("the combination of value, value_min and value_max cells {other:?} is not a point, a range or an upper bound"),
            ))
        }
    };
    if let GeometryValue::Text(_) = value {
        let q: GeometryQuantity = row.vocab("quantity")?;
        if q != GeometryQuantity::MatrixComposition {
            return Err(row.err(
                Some("value"),
                "a non-numeric value is only allowed for the matrix composition",
            ));
        }
    }
    let layer: GeometryLayer = row.vocab("layer")?;
    let idx = row.uint("layer_index")?;
    if idx != u32::from(layer.expected_index()) {
        return Err(row.err(
            Some("layer_index"),
            format!(
                "layer `{}` has index {} in the register, the row says {idx}",
                layer.as_str(),
                layer.expected_index()
            ),
        ));
    }
    let uncertainty_text = row.required("uncertainty")?.to_string();
    let uncertainty =
        Uncertainty::parse(&uncertainty_text).map_err(|m| row.err(Some("uncertainty"), m))?;
    Ok(GeometryRow {
        line: row.line(),
        id: row.required("id")?.to_string(),
        design_id: row.required("design_id")?.to_string(),
        design: row.required("design")?.to_string(),
        layer_index: layer.expected_index(),
        layer,
        quantity: row.vocab("quantity")?,
        symbol: row.required("symbol")?.to_string(),
        value,
        unit: row.vocab("unit")?,
        uncertainty,
        uncertainty_text,
        material: row.text("material")?.to_string(),
        outer_radius_um_calc: row.opt_number("outer_radius_um_calc")?,
        data_class: row.required("data_class")?.to_string(),
        evidence_class: row.vocab("evidence_class")?,
        source: row.required("source")?.to_string(),
        source_location: row.required("source_location")?.to_string(),
        link: row.required("link")?.to_string(),
        status: row.vocab("status")?,
        notes: row.text("notes")?.to_string(),
    })
}

/// All rows of `geometry.csv`.
#[derive(Debug, Clone)]
pub struct Geometry {
    /// The rows in file order.
    pub rows: Vec<GeometryRow>,
}

impl Geometry {
    /// Read and validate the file text.
    pub fn parse(text: &str) -> Result<Geometry> {
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
        Ok(Geometry { rows })
    }

    /// The design identifiers in order of first appearance.
    pub fn design_ids(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for r in &self.rows {
            if !out.contains(&r.design_id.as_str()) {
                out.push(r.design_id.as_str());
            }
        }
        out
    }
}
