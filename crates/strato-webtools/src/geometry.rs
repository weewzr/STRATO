//! `geometry`: the AGR-1 rows of data/geometry/geometry.csv as `web/src/generated/agr1_reference.json`
//! and `web/public/data/agr1_reference_geometry.csv`: the kernel diameter and coating thicknesses of
//! each AGR-1 set with row ids, register status and source, and each layer's outer radius (sums of
//! the registered values, checked against the register's own radius column where it has one).
//! The reference particle is geometry only (rule E2). Its header records the HEAD commit (with
//! "+modified" if the generator or the input file differed from it) and generator_sha256, the
//! SHA-256 of the generator's sources, so the generator that wrote the files is identified even
//! when the tree was not clean.

use fancy_regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::Path;

use crate::colormaps::num;
use crate::pyfmt::{csv_row, json_file};
use crate::util::{self, Error, Result};
use crate::{
    bail, catalogue, csv_line, CSV_DIR, GENERATOR, GENERATOR_VERSION, JSON_DIR, SCHEMA, TOOL_DIR,
};

pub const GEOMETRY_CSV: &str = "data/geometry/geometry.csv";
pub const GEOMETRY_DEFAULT: &str = "AGR1-B-M16";
const LAYERS: [&str; 5] = ["kernel", "buffer", "IPyC", "SiC", "OPyC"];
const LINK_VERIFIED: &str = "public link in the register";
const LINK_NOT_VERIFIED: &str = "no verified public link in the register";

/// The AGR-1 sets drawn as the reference particle, in display order: (set id, the design_id of
/// each of the five layer rows). Every set gives the kernel diameter and four coating
/// thicknesses of one register design. The IAEA (2012) Table 7.14 variants take the kernel from
/// design AGR1-T714 (row G-20, whose register note reads "single kernel value printed for all
/// variants").
const GEOMETRY_SETS: [(&str, [&str; 5]); 7] = [
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

/// A decimal number as Python's `decimal.Decimal` holds it (coefficient and exponent), for the
/// sums of the register values: the exponent the register's own text has is kept, as in `str()`.
#[derive(Clone, Copy, Debug)]
struct Dec {
    coef: i128,
    exp: i32,
}

impl Dec {
    fn parse(text: &str) -> Result<Dec> {
        let t = text.trim();
        let bad = || Error(format!("'{text}' is not a plain decimal number"));
        let (neg, body) = match t.strip_prefix('-') {
            Some(b) => (true, b),
            None => (false, t.strip_prefix('+').unwrap_or(t)),
        };
        let (int, frac) = body.split_once('.').unwrap_or((body, ""));
        if (int.is_empty() && frac.is_empty())
            || !int.chars().chain(frac.chars()).all(|c| c.is_ascii_digit())
        {
            return Err(bad());
        }
        let digits = format!("{int}{frac}");
        let coef: i128 = digits.parse().map_err(|_| bad())?;
        Ok(Dec {
            coef: if neg { -coef } else { coef },
            exp: -(frac.len() as i32),
        })
    }

    /// `self / 2` with the exponent Python's exact division gives.
    fn half(self) -> Dec {
        if self.coef % 2 == 0 {
            Dec {
                coef: self.coef / 2,
                exp: self.exp,
            }
        } else {
            Dec {
                coef: self.coef * 5,
                exp: self.exp - 1,
            }
        }
    }

    fn aligned(a: Dec, b: Dec) -> (i128, i128, i32) {
        let exp = a.exp.min(b.exp);
        (
            a.coef * 10i128.pow((a.exp - exp) as u32),
            b.coef * 10i128.pow((b.exp - exp) as u32),
            exp,
        )
    }

    fn add(self, other: Dec) -> Dec {
        let (a, b, exp) = Dec::aligned(self, other);
        Dec { coef: a + b, exp }
    }

    fn equals(self, other: Dec) -> bool {
        let (a, b, _) = Dec::aligned(self, other);
        a == b
    }

    /// `str(Decimal)` for the plain forms (exponent not above zero, adjusted exponent not below -6).
    fn text(self) -> Result<String> {
        let neg = self.coef < 0;
        let digits = self.coef.abs().to_string();
        let adjusted = digits.len() as i32 + self.exp - 1;
        if self.exp > 0 || adjusted < -6 {
            bail!("{self:?} needs the scientific form of Decimal, which this tool does not write");
        }
        let body = if self.exp == 0 {
            digits
        } else {
            let k = (-self.exp) as usize;
            if k < digits.len() {
                format!(
                    "{}.{}",
                    &digits[..digits.len() - k],
                    &digits[digits.len() - k..]
                )
            } else {
                format!("0.{}{}", "0".repeat(k - digits.len()), digits)
            }
        };
        Ok(if neg { format!("-{body}") } else { body })
    }

    fn float(self) -> Result<f64> {
        self.text()?
            .parse()
            .map_err(|_| Error("decimal does not convert to a float".into()))
    }
}

type Row = HashMap<String, String>;

fn field<'a>(r: &'a Row, key: &str) -> Result<&'a str> {
    r.get(key)
        .map(String::as_str)
        .ok_or_else(|| Error(format!("{GEOMETRY_CSV}: a row has no column '{key}'")))
}

fn geometry_row<'a>(rows: &'a [Row], design: &str, layer: &str) -> Result<&'a Row> {
    let want: [&str; 2] = if layer == "kernel" {
        ["diameter", "diameter (specified)"]
    } else {
        ["thickness", "thickness (specified)"]
    };
    let mut hits = Vec::new();
    for r in rows {
        if field(r, "design_id")? == design
            && field(r, "layer")? == layer
            && want.contains(&field(r, "quantity")?)
        {
            hits.push(r);
        }
    }
    if hits.len() != 1 {
        bail!(
            "{GEOMETRY_CSV}: expected one {} row for {design} {layer}, found {}",
            want[0],
            hits.len()
        );
    }
    let r = hits[0];
    if field(r, "unit")? != "um" || field(r, "value")?.is_empty() {
        bail!("{GEOMETRY_CSV}: row {} has no value in um", field(r, "id")?);
    }
    Ok(r)
}

/// The register's source location without its local file name and text-extract index.
fn reader_location(text: &str) -> Result<String> {
    let lead = Regex::new(r"^\d+\.pdf\s*")?;
    let mut parts = Vec::new();
    for p in text
        .split(';')
        .map(str::trim)
        .filter(|p| !p.starts_with("idx"))
    {
        let q = lead.replace(p, "").into_owned();
        if !q.is_empty() {
            parts.push(q);
        }
    }
    Ok(parts.join("; "))
}

/// The register's link if it is a public http(s) URL, else `None`; and a status in words. The
/// register also records local file notes such as "file:Paper/3.pdf (IAEA URL NOT VERIFIED)";
/// those are not links a reader can follow and are not shipped.
fn public_link(text: &str) -> Result<(Option<String>, &'static str)> {
    let link = text.trim();
    if Regex::new(r"^https?://\S+$")?.is_match(link)? {
        return Ok((Some(link.to_string()), LINK_VERIFIED));
    }
    Ok((None, LINK_NOT_VERIFIED))
}

/// The AGR-1 layer dimensions of the register, with each layer's outer radius.
///
/// Outer radii: kernel diameter / 2, then the previous radius plus the layer thickness (sums of
/// register values, in micrometres). Where the register itself gives the radius (column
/// outer_radius_um_calc) the two must agree exactly (decimal arithmetic on the register text), or
/// the generator stops. This is geometry bookkeeping for a drawing; no transport quantity is
/// computed.
pub fn build(root: &Path) -> Result<(Value, String)> {
    let raw = util::read_bytes(&root.join(GEOMETRY_CSV))?;
    let rows = util::dict_rows(&util::decode_utf8(&raw, GEOMETRY_CSV)?).rows;
    let mut sets: Vec<Value> = Vec::new();
    let mut set_rows: Vec<Vec<Vec<String>>> = Vec::new();
    for (set_id, designs) in GEOMETRY_SETS {
        let mut layers: Vec<Value> = Vec::new();
        let mut csv_lines: Vec<Vec<String>> = Vec::new();
        let mut outer = Dec { coef: 0, exp: 0 };
        let mut statuses: Vec<String> = Vec::new();
        for (layer, design) in LAYERS.iter().zip(designs) {
            let r = geometry_row(&rows, design, layer)?;
            let value = Dec::parse(field(r, "value")?)?;
            outer = if *layer == "kernel" {
                value.half()
            } else {
                outer.add(value)
            };
            let registered = field(r, "outer_radius_um_calc")?.trim();
            if !registered.is_empty() && !Dec::parse(registered)?.equals(outer) {
                bail!(
                    "{GEOMETRY_CSV}: row {} outer radius {registered} um, summed Decimal('{}') um",
                    field(r, "id")?,
                    outer.text()?
                );
            }
            let (link, link_status) = public_link(field(r, "link")?)?;
            let source_kind = if registered.is_empty() {
                "generator"
            } else {
                "register"
            };
            let status = field(r, "status")?.to_string();
            statuses.push(status.clone());
            let location = reader_location(field(r, "source_location")?)?;
            layers.push(json!({
                "row_id": field(r, "id")?,
                "design_id": field(r, "design_id")?,
                "layer": layer,
                "material": field(r, "material")?,
                "quantity": field(r, "quantity")?,
                "value_um": num(value.float()?)?,
                "value_text": field(r, "value")?,
                "outer_radius_um": num(outer.float()?)?,
                "outer_radius_text": outer.text()?,
                "outer_radius_source": source_kind,
                "data_class": field(r, "data_class")?,
                "evidence_class": field(r, "evidence_class")?,
                "status": status,
                "source": field(r, "source")?,
                "location": location,
                "link": link,
                "link_status": link_status,
                "notes": field(r, "notes")?,
            }));
            csv_lines.push(vec![
                set_id.to_string(),
                layer.to_string(),
                field(r, "id")?.to_string(),
                field(r, "design_id")?.to_string(),
                field(r, "quantity")?.to_string(),
                field(r, "value")?.to_string(),
                outer.text()?,
                source_kind.to_string(),
                field(r, "evidence_class")?.to_string(),
                status,
                field(r, "source")?.to_string(),
                location,
                link.unwrap_or_else(|| link_status.to_string()),
            ]);
        }
        let main = rows
            .iter()
            .find(|r| {
                field(r, "design_id")
                    .map(|d| d == designs[4])
                    .unwrap_or(false)
            })
            .ok_or_else(|| Error(format!("{GEOMETRY_CSV}: no row of design {}", designs[4])))?;
        let (set_link, set_link_status) = public_link(field(main, "link")?)?;
        statuses.sort();
        statuses.dedup();
        sets.push(json!({
            "set_id": set_id,
            "design": field(main, "design")?,
            "source": field(main, "source")?,
            "link": set_link,
            "link_status": set_link_status,
            "evidence_class": field(main, "evidence_class")?,
            "statuses": statuses,
            "layers": layers,
        }));
        set_rows.push(csv_lines);
    }

    let generator = format!("{GENERATOR} v{GENERATOR_VERSION}");
    let version = catalogue::strato_version(root)?;
    let commit = util::source_commit(root, &[TOOL_DIR, GEOMETRY_CSV]);
    let gen_sha = crate::generator_sha256(root);
    let input_sha = util::sha256_hex(&raw);
    let command = format!("{GENERATOR} geometry");
    let created = util::now_utc();
    let note = concat!(
        "Registered geometry of the AGR-1 UCO TRISO particle (rule E2): geometry only, not a simulation and ",
        "never combined with the diffusivities of the synthetic case. REGISTER marks register data; it is not ",
        "a result label. Outer radii are sums of the registered diameter and thicknesses. location is the ",
        "register's source location without its local file names and text-extract indices; link is given only ",
        "where the register holds a public http(s) URL. commit is the HEAD commit the generator ran on; it ",
        "carries the suffix +modified when the generator or the input file differed from that commit. ",
        "generator_sha256 is the SHA-256 of the generator file that wrote this file."
    );
    let mut doc = Map::new();
    doc.insert("schema".into(), json!(SCHEMA));
    doc.insert("label".into(), json!("REGISTER"));
    doc.insert("kind".into(), json!("reference_geometry"));
    doc.insert("generator".into(), json!(generator));
    doc.insert("strato_version".into(), json!(version));
    doc.insert("commit".into(), json!(commit));
    doc.insert("generator_sha256".into(), json!(gen_sha));
    doc.insert("input_file".into(), json!(GEOMETRY_CSV));
    doc.insert("input_sha256".into(), json!(input_sha));
    doc.insert("command".into(), json!(command));
    doc.insert("created_utc".into(), json!(created));
    doc.insert("note".into(), json!(note));
    doc.insert(
        "data".into(),
        json!({"default_set": GEOMETRY_DEFAULT, "sets": sets}),
    );

    let mut buf = String::new();
    for (k, v) in [
        ("label", "REGISTER"),
        ("kind", "reference_geometry"),
        ("generator", generator.as_str()),
        ("strato_version", version.as_str()),
        ("commit", commit.as_str()),
        ("generator_sha256", gen_sha.as_str()),
        ("input_file", GEOMETRY_CSV),
        ("input_sha256", input_sha.as_str()),
        ("command", command.as_str()),
        ("created_utc", created.as_str()),
    ] {
        buf.push_str(&format!("# {k}: {v}\n"));
    }
    buf.push_str(&format!("# note: {note}\n"));
    buf.push_str(&csv_line!(
        "set_id",
        "layer",
        "row_id",
        "design_id",
        "quantity",
        "value_um",
        "outer_radius_um",
        "outer_radius_source",
        "evidence_class",
        "status",
        "source",
        "location",
        "link"
    ));
    for set in &set_rows {
        for line in set {
            buf.push_str(&csv_row(line));
        }
    }
    Ok((Value::Object(doc), buf))
}

pub fn run(root: &Path, check: bool) -> Result<i32> {
    let (doc, csv_text) = build(root)?;
    let targets = [
        crate::Target {
            path: root.join(JSON_DIR).join("agr1_reference.json"),
            text: json_file(&doc),
            is_json: true,
        },
        crate::Target {
            path: root.join(CSV_DIR).join("agr1_reference_geometry.csv"),
            text: csv_text,
            is_json: false,
        },
    ];
    crate::write_or_check(root, &targets, check, &crate::VOLATILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_text_follows_python() {
        let d = |s: &str| Dec::parse(s).unwrap();
        assert_eq!(d("425").half().text().unwrap(), "212.5");
        assert_eq!(d("350").half().text().unwrap(), "175");
        assert_eq!(d("350.0").half().text().unwrap(), "175.0");
        assert_eq!(d("212.5").add(d("100")).text().unwrap(), "312.5");
        assert_eq!(d("95").add(d("40.0")).text().unwrap(), "135.0");
        assert!(d("212.50").equals(d("212.5")));
        assert_eq!(d("0.5").text().unwrap(), "0.5");
        assert_eq!(d("0.05").text().unwrap(), "0.05");
    }
}
