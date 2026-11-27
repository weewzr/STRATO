//! The generated tables of `data/reference/STRATO_Parameter_Register.md`.
//!
//! The register document is partly generated from the CSV files and partly written by hand. A
//! generated table sits between two marker lines
//!
//! ```text
//! <!-- strato-data:table KIND ARGS... -->
//! | header | ... |
//! |---|---| ...
//! | row | ... |
//! <!-- strato-data:end -->
//! ```
//!
//! and everything outside the markers is hand-written and never touched: `strato-data register
//! --check` regenerates every marked table from the CSV files and fails if a table differs;
//! `strato-data register --write` replaces the lines between the markers and leaves every other
//! byte of the document as it is. The kinds are `geom`, `radii`, `arr`, `decay`, `fy`, `temp` and
//! `rel`; their arguments (design ids, set ids, nuclides, history prefixes, program and kind) are
//! those of the former template. This is the port of `scripts/data_register/render_register.py`;
//! the output is the same text the script wrote for the same files.

use crate::register::RegisterTexts;
use crate::table::{Row, Table};
use crate::{arrhenius, decay, geometry, release, temperature, yields};

/// The first marker line starts with this.
pub const BEGIN: &str = "<!-- strato-data:table ";
/// The line that closes a generated table.
pub const END: &str = "<!-- strato-data:end -->";

/// The result for one generated table.
#[derive(Debug, Clone)]
pub struct BlockReport {
    /// Line (1-based) of the opening marker.
    pub line: usize,
    /// Kind and arguments, as in the marker.
    pub spec: String,
    /// Number of lines of the generated table.
    pub lines: usize,
    /// Whether the document holds exactly the generated table.
    pub current: bool,
    /// The first line that differs, for a message.
    pub first_difference: Option<String>,
}

/// The register files as tables of raw cells.
pub struct Tables {
    geo: Table,
    arr: Table,
    dec: Table,
    fy: Table,
    th: Table,
    rel: Table,
}

impl Tables {
    /// Parse the six files of `texts` against their exact headers.
    ///
    /// # Errors
    ///
    /// A file that does not parse.
    pub fn read(texts: &RegisterTexts) -> Result<Tables, String> {
        let p = |file: &'static str, text: &str, header: &[&str]| {
            Table::parse(file, text, header).map_err(|e| e.to_string())
        };
        Ok(Tables {
            geo: p(geometry::FILE, &texts.geometry, &geometry::HEADER)?,
            arr: p(arrhenius::FILE, &texts.arrhenius, &arrhenius::HEADER)?,
            dec: p(decay::FILE, &texts.decay, &decay::HEADER)?,
            fy: p(yields::FILE, &texts.yields, &yields::HEADER)?,
            th: p(temperature::FILE, &texts.temperature, &temperature::HEADER)?,
            rel: p(release::FILE, &texts.release, &release::HEADER)?,
        })
    }
}

fn unit(u: &str) -> &str {
    match u {
        "um" => r"$\mathrm{\mu m}$",
        "mm" => r"$\mathrm{mm}$",
        "%" => r"$\%$",
        "1" => "1",
        "Mg m^-3" => r"$\mathrm{Mg\ m^{-3}}$",
        "g" => r"$\mathrm{g}$",
        "wt%" => r"$\mathrm{wt\%}$",
        "particles" => "particles",
        "h^-1" => r"$\mathrm{h^{-1}}$",
        other => other,
    }
}

fn symbol(s: &str) -> &str {
    match s {
        "d_k" => r"$d_k = 2R_1$",
        "t_2" => r"$t_2$",
        "t_3" => r"$t_3$",
        "t_4" => r"$t_4$",
        "t_5" => r"$t_5$",
        "rho_1" => r"$\rho_1$",
        "rho_2" => r"$\rho_2$",
        "rho_3" => r"$\rho_3$",
        "rho_4" => r"$\rho_4$",
        "rho_5" => r"$\rho_5$",
        "e_U5" => r"$\varepsilon_5$",
        "N_p" => r"$N_p$",
        "f_pack" => r"$f_{\mathrm{pack}}$",
        "f_U" => r"$f_U$",
        "L_c" => r"$L_c$",
        "D_c" => r"$D_c$",
        "g_c" => r"$g_c$",
        "w_mat" => r"$w_{\mathrm{mat}}$",
        "m_U" => r"$m_U$",
        "f_dSiC" => r"$f_{\mathrm{dSiC}}$",
        "f_dOPyC" => r"$f_{\mathrm{dOPyC}}$",
        "2R" => r"$2R$",
        "m_U5" => r"$m_{U5}$",
        "f_U2" => r"$f_{U,2}$",
        "f_U3" => r"$f_{U,3}$",
        "f_U4" => r"$f_{U,4}$",
        "f_U5" => r"$f_{U,5}$",
        "f_Um" => r"$f_{U,m}$",
        other => other,
    }
}

/// `5.6e-8` as `$5.6\times10^{-8}$`, `1e-6` as `$10^{-6}$`, plain numbers unchanged.
fn sci(s: &str) -> String {
    let s = s.trim();
    let Some(pos) = s.find(['e', 'E']) else {
        return s.to_string();
    };
    let (mant, ex) = (&s[..pos], &s[pos + 1..]);
    // `[-+]?\d*\.?\d+` for the mantissa, `[-+]?\d+` for the exponent
    let digits = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
    let unsigned = |t: &str| t.strip_prefix(['-', '+']).unwrap_or(t).to_string();
    let m = unsigned(mant);
    let mant_ok = match m.split_once('.') {
        Some((a, b)) => (a.is_empty() || digits(a)) && digits(b),
        None => digits(&m),
    };
    let ex_u = unsigned(ex);
    if !mant_ok || !digits(&ex_u) {
        return s.to_string();
    }
    let Ok(exp) = ex.trim_start_matches('+').parse::<i64>() else {
        return s.to_string();
    };
    if exp == 0 {
        return mant.to_string();
    }
    if mant == "1" || mant == "1.0" {
        return format!(r"$10^{{{exp}}}$");
    }
    format!(r"${mant}\times10^{{{exp}}}$")
}

fn link(u: &str) -> String {
    if u.starts_with("http") {
        if u.contains("doi.org") {
            format!("[doi]({u})")
        } else {
            format!("[link]({u})")
        }
    } else {
        u.replace("file:", "local ")
    }
}

fn table(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = vec![
        format!("| {} |", header.join(" | ")),
        format!("|{}", "---|".repeat(header.len())),
    ];
    for r in rows {
        let cells: Vec<String> = r.iter().map(|c| c.replace('|', "/")).collect();
        out.push(format!("| {} |", cells.join(" | ")));
    }
    out.join("\n")
}

type Res<T> = Result<T, String>;

fn t<'a>(r: &Row<'a>, c: &str) -> Res<&'a str> {
    r.text(c).map_err(|e| e.to_string())
}

fn geom(tb: &Tables, args: &[&str]) -> Res<String> {
    let mut rows = Vec::new();
    for r in tb.geo.rows() {
        if !args.contains(&t(&r, "design_id")?) {
            continue;
        }
        let v = if !t(&r, "value")?.is_empty() {
            sci(t(&r, "value")?)
        } else if !t(&r, "value_min")?.is_empty() && !t(&r, "value_max")?.is_empty() {
            format!(
                "{} to {}",
                sci(t(&r, "value_min")?),
                sci(t(&r, "value_max")?)
            )
        } else {
            format!("< {}", sci(t(&r, "value_max")?))
        };
        let var = if t(&r, "layer_index")? != "0" {
            format!("{} {}", t(&r, "layer")?, t(&r, "quantity")?)
        } else {
            t(&r, "quantity")?.to_string()
        };
        let note = if t(&r, "notes")?.is_empty() {
            String::new()
        } else {
            format!("; {}", t(&r, "notes")?)
        };
        rows.push(vec![
            t(&r, "id")?.to_string(),
            format!("{var} ({})", t(&r, "design")?),
            symbol(t(&r, "symbol")?).to_string(),
            v,
            unit(t(&r, "unit")?).to_string(),
            t(&r, "material")?.to_string(),
            "n/a".to_string(),
            format!("{}; {}{note}", t(&r, "uncertainty")?, t(&r, "data_class")?),
            t(&r, "source")?.to_string(),
            t(&r, "source_location")?.to_string(),
            link(t(&r, "link")?),
            t(&r, "evidence_class")?.to_string(),
            t(&r, "status")?.to_string(),
        ]);
    }
    Ok(table(
        &[
            "No.",
            "Variable",
            "Symbol",
            "Value",
            "Unit",
            "Material / Nuclide",
            "Temperature range",
            "Uncertainty / class / note",
            "Source / APA citation",
            "Location",
            "Link",
            "Evidence class",
            "Status",
        ],
        &rows,
    ))
}

fn radii(tb: &Tables, args: &[&str]) -> Res<String> {
    let mut rows = Vec::new();
    for did in args {
        let mut lay: Vec<(i64, Row<'_>)> = Vec::new();
        for r in tb.geo.rows() {
            if t(&r, "design_id")? == *did && !t(&r, "outer_radius_um_calc")?.is_empty() {
                let i = t(&r, "layer_index")?
                    .parse::<i64>()
                    .map_err(|e| format!("layer_index of {did}: {e}"))?;
                lay.push((i, r));
            }
        }
        let get = |i: i64| -> Res<&Row<'_>> {
            lay.iter()
                .rfind(|(k, _)| *k == i)
                .map(|(_, r)| r)
                .ok_or_else(|| format!("design {did} has no layer {i} with a calculated radius"))
        };
        let mut row = vec![did.to_string(), t(get(1)?, "design")?.to_string()];
        for i in 1..=5 {
            row.push(t(get(i)?, "outer_radius_um_calc")?.to_string());
        }
        let ids: Res<Vec<&str>> = (1..=5).map(|i| t(get(i)?, "id")).collect();
        row.push(ids?.join(", "));
        rows.push(row);
    }
    Ok(table(
        &[
            "Design id",
            "Design",
            r"$R_1$ ($\mathrm{\mu m}$)",
            "$R_2$",
            "$R_3$",
            "$R_4$",
            "$R_5 = R$",
            "Input rows",
        ],
        &rows,
    ))
}

fn trange(r: &Row<'_>) -> Res<String> {
    let (a, b, note) = (t(r, "T_min_C")?, t(r, "T_max_C")?, t(r, "T_range_note")?);
    Ok(if !a.is_empty() && !b.is_empty() && a == b {
        format!("{a} \u{b0}C only")
    } else if !a.is_empty() && !b.is_empty() {
        format!("{a} to {b} \u{b0}C")
    } else if !b.is_empty() {
        format!("$\\le {b}$ \u{b0}C")
    } else {
        note.to_string()
    })
}

fn arrh(tb: &Tables, args: &[&str]) -> Res<String> {
    let mut rows = Vec::new();
    for r in tb.arr.rows() {
        if !args.contains(&t(&r, "set_id")?) {
            continue;
        }
        let id = t(&r, "id")?;
        let mut d0 = if t(&r, "D0_m2_s")?.is_empty() {
            r"$\infty$".to_string()
        } else {
            sci(t(&r, "D0_m2_s")?)
        };
        if id == "A3-01" || id == "E109-01" {
            d0 = r"$5.5\times10^{-14}\,e^{\Gamma/5}$".to_string();
        }
        if id == "A3-13" {
            d0 = r"$\le 6.8\times10^{-12}$".to_string();
        }
        let q = if t(&r, "Q_kJ_mol")?.is_empty() {
            "none".to_string()
        } else {
            t(&r, "Q_kJ_mol")?.to_string()
        };
        let note = if !t(&r, "D0_expression")?.is_empty()
            && !["A3-01", "E109-01", "A3-13"].contains(&id)
        {
            format!(" {}.", t(&r, "D0_expression")?)
        } else {
            String::new()
        };
        let form = format!(
            "{}{}{note}",
            t(&r, "form")?,
            if t(&r, "notes")?.is_empty() {
                String::new()
            } else {
                format!("; {}", t(&r, "notes")?)
            }
        );
        rows.push(vec![
            id.to_string(),
            format!("{} ({})", t(&r, "nuclide")?, t(&r, "origin")?),
            format!("{} [{}]", t(&r, "material")?, t(&r, "layer")?),
            t(&r, "term")?.to_string(),
            d0,
            q,
            r"$\mathrm{m^2\ s^{-1}}$; $\mathrm{kJ\ mol^{-1}}$".to_string(),
            trange(&r)?,
            form,
            format!("{}; ref. {}", t(&r, "source")?, t(&r, "ref_in_source")?),
            t(&r, "source_location")?.to_string(),
            link(t(&r, "link")?),
            format!(
                "{}; $R_g$ = {}",
                t(&r, "evidence_class")?,
                t(&r, "R_J_mol_K")?
            ),
            t(&r, "status")?.to_string(),
        ]);
    }
    Ok(table(
        &[
            "No.",
            "Nuclide (origin)",
            "Layer/material [STRATO layer]",
            "Term",
            "D0",
            "Q",
            "Units",
            "Temperature range",
            "Form / note",
            "APA citation",
            "Location",
            "Link",
            "Evidence class; gas constant",
            "Status",
        ],
        &rows,
    ))
}

fn decay_table(tb: &Tables) -> Res<String> {
    let mut rows = Vec::new();
    for (i, r) in tb.dec.rows().enumerate() {
        let (hl, lam, ul, mode);
        if t(&r, "stable")? == "yes" {
            hl = "stable".to_string();
            lam = "0".to_string();
            ul = String::new();
            mode = "stable".to_string();
        } else {
            let first = t(&r, "unc_half_life")?
                .split_whitespace()
                .next()
                .ok_or_else(|| {
                    format!(
                        "unc_half_life of {} is empty",
                        t(&r, "nuclide").unwrap_or("")
                    )
                })?;
            hl = format!(
                "{} {} (ENSDF last digit {first})",
                t(&r, "half_life")?,
                t(&r, "unit")?
            );
            lam = sci(t(&r, "lambda_s-1")?);
            ul = sci(t(&r, "unc_lambda_s-1")?);
            mode = format!(
                "{} {} %{}",
                t(&r, "decay_mode_1")?,
                t(&r, "branch_1_pct")?,
                if t(&r, "decay_mode_2")?.is_empty() {
                    String::new()
                } else {
                    format!("; {} {} %", t(&r, "decay_mode_2")?, t(&r, "branch_2_pct")?)
                }
            );
        }
        rows.push(vec![
            format!("N-{:02}", i + 1),
            "decay constant".to_string(),
            r"$\lambda_k$".to_string(),
            lam,
            r"$\mathrm{s^{-1}}$".to_string(),
            t(&r, "nuclide")?.to_string(),
            "n/a".to_string(),
            ul,
            hl,
            mode,
            t(&r, "daughters")?.to_string(),
            t(&r, "product_class")?.to_string(),
            format!("{}; via IAEA LiveChart", t(&r, "ENSDF_evaluation")?),
            t(&r, "source_location")?
                .split(';')
                .next()
                .unwrap_or("")
                .to_string(),
            "[link](https://www-nds.iaea.org/relnsd/v1/data)".to_string(),
            t(&r, "evidence_class")?.to_string(),
            t(&r, "status")?.to_string(),
        ]);
    }
    Ok(table(
        &[
            "No.",
            "Variable",
            "Symbol",
            "Value",
            "Unit",
            "Material / Nuclide",
            "Temperature range",
            r"Uncertainty $u(\lambda_k)$",
            r"$t_{1/2}$",
            "Decay mode",
            "Daughter(s)",
            "Direct fission product or activation product",
            "Source / APA citation",
            "Location",
            "Link",
            "Evidence class",
            "Status",
        ],
        &rows,
    ))
}

/// Python's `f"{x:.5e}"` split into mantissa and exponent, the mantissa without trailing zeros.
fn yield_number(v: &str) -> Res<String> {
    let x: f64 = v
        .trim()
        .parse()
        .map_err(|_| format!("`{v}` is not a number"))?;
    if x == 0.0 {
        return Ok("0".to_string());
    }
    let s = format!("{x:.5e}");
    let (m, e) = s.split_once('e').ok_or("exponent form")?;
    let m = m.trim_end_matches('0').trim_end_matches('.');
    let e: i64 = e.parse().map_err(|_| "exponent".to_string())?;
    Ok(sci(&format!("{m}e{e}")))
}

fn pm(v: &str, u: &str) -> Res<String> {
    if v.contains("BLANK") {
        return Ok("blank in API".to_string());
    }
    if u.is_empty() {
        yield_number(v)
    } else {
        Ok(format!(r"{} $\pm$ {}", yield_number(v)?, yield_number(u)?))
    }
}

fn fyt(tb: &Tables, args: &[&str]) -> Res<String> {
    let mut rows = Vec::new();
    let mut i = 0;
    for r in tb.fy.rows() {
        if !args.contains(&t(&r, "product")?) {
            continue;
        }
        i += 1;
        let par = if t(&r, "parent")? == "235U" {
            "U-235"
        } else {
            "Pu-239"
        };
        let endf = if !t(&r, "CFY_ENDFB_NuDat3")?.is_empty() {
            format!(
                "{} ({})",
                pm(t(&r, "CFY_ENDFB_NuDat3")?, t(&r, "unc_CFY_ENDFB_NuDat3")?)?,
                t(&r, "status_ENDFB_crosscheck")?
            )
        } else {
            "not given".to_string()
        };
        rows.push(vec![
            format!("Y-{i:02}"),
            format!("cumulative yield, {par} thermal fission"),
            r"$y^C_k$".to_string(),
            pm(t(&r, "CFY_JEFF311")?, t(&r, "unc_CFY_JEFF311")?)?,
            "per fission".to_string(),
            t(&r, "product")?.to_string(),
            "thermal".to_string(),
            pm(t(&r, "IFY_JEFF311")?, t(&r, "unc_IFY_JEFF311")?)?,
            endf,
            "JEFF-3.1.1".to_string(),
            format!("{}; {}", t(&r, "product_class")?, t(&r, "STRATO_use_note")?),
            "Kellett et al. (2009) via IAEA LiveChart".to_string(),
            t(&r, "source_location")?
                .split(';')
                .next()
                .unwrap_or("")
                .to_string(),
            "[link](https://www-nds.iaea.org/relnsd/v1/data)".to_string(),
            t(&r, "evidence_class")?.to_string(),
            t(&r, "status")?.to_string(),
        ]);
    }
    Ok(table(
        &[
            "No.",
            "Variable",
            "Symbol",
            r"Value (CFY $\pm 1\sigma$)",
            "Unit",
            "Material / Nuclide",
            "Neutron energy",
            r"Independent yield $y^I_k \pm 1\sigma$",
            "ENDF/B CFY via NuDat 3 (status)",
            "Library",
            "Direct fission product or activation product; STRATO use",
            "Source / APA citation",
            "Location",
            "Link",
            "Evidence class",
            "Status",
        ],
        &rows,
    ))
}

fn temp(tb: &Tables, args: &[&str]) -> Res<String> {
    let mut rows = Vec::new();
    for r in tb.th.rows() {
        if !args
            .iter()
            .any(|p| t(&r, "history_id").is_ok_and(|h| h.starts_with(p)))
        {
            continue;
        }
        let ns = |c: &str| -> Res<String> {
            let v = t(&r, c)?;
            Ok(if v.is_empty() {
                "n/s".to_string()
            } else {
                v.to_string()
            })
        };
        rows.push(vec![
            t(&r, "id")?.to_string(),
            t(&r, "history")?.to_string(),
            t(&r, "step")?.to_string(),
            t(&r, "segment")?.to_string(),
            ns("T_start_C")?,
            ns("T_end_C")?,
            t(&r, "rate_C_per_h")?.to_string(),
            t(&r, "ramp_h")?.to_string(),
            t(&r, "hold_h")?.to_string(),
            t(&r, "cumulative_end_h_calc")?.to_string(),
            t(&r, "history_class")?.to_string(),
            t(&r, "notes")?.to_string(),
            t(&r, "source")?.to_string(),
            t(&r, "source_location")?.to_string(),
            link(t(&r, "link")?),
            t(&r, "evidence_class")?.to_string(),
            t(&r, "status")?.to_string(),
        ]);
    }
    Ok(table(
        &[
            "No.",
            "History",
            "Step",
            "Segment",
            "T start (\u{b0}C)",
            "T end (\u{b0}C)",
            "Rate ($\\mathrm{\u{b0}C\\ h^{-1}}$)",
            "Ramp (h)",
            "Hold (h)",
            "Cumulative end (h, calc.)",
            "Class (measured / reconstructed / specified)",
            "Note",
            "Source / APA citation",
            "Location",
            "Link",
            "Evidence class",
            "Status",
        ],
        &rows,
    ))
}

fn relt(tb: &Tables, args: &[&str]) -> Res<String> {
    let (prog, kind) = match args {
        [p, k, ..] => (*p, *k),
        _ => return Err("rel needs a program and a kind".to_string()),
    };
    let mut rows = Vec::new();
    for r in tb.rel.rows() {
        if t(&r, "program")? != prog {
            continue;
        }
        let failed = t(&r, "quantity")?.contains("failed");
        if kind == "counts" && !failed {
            continue;
        }
        if kind != "counts" && t(&r, "unit")? == "particles" && failed {
            continue;
        }
        if kind != "all" && kind != "counts" && t(&r, "nuclide")? != kind {
            continue;
        }
        let v = if !t(&r, "value")?.is_empty() {
            let q = if t(&r, "qualifier")? != "=" {
                t(&r, "qualifier")?
            } else {
                ""
            };
            let mut v = format!("{q} {}", sci(t(&r, "value")?)).trim().to_string();
            if !t(&r, "value_max")?.is_empty() && kind == "counts" {
                v.push_str(&format!(" of {}", t(&r, "value_max")?));
            }
            v
        } else {
            format!(
                "{} to {}",
                sci(t(&r, "value_min")?),
                sci(t(&r, "value_max")?)
            )
        };
        let tc = if t(&r, "T_C")?.is_empty() {
            "n/a".to_string()
        } else {
            format!("{} \u{b0}C", t(&r, "T_C")?)
        };
        let th = if t(&r, "time_h")?.is_empty() {
            "n/s"
        } else {
            t(&r, "time_h")?
        };
        rows.push(vec![
            t(&r, "id")?.to_string(),
            format!("{} ({})", t(&r, "quantity")?, t(&r, "group")?),
            if t(&r, "unit")? == "1" {
                r"$f_{\mathrm{rel}}$".to_string()
            } else {
                String::new()
            },
            v,
            unit(t(&r, "unit")?).to_string(),
            format!(
                "{}; {} ({})",
                t(&r, "nuclide")?,
                t(&r, "compact_or_sample")?,
                t(&r, "fuel_type")?
            ),
            tc,
            format!(
                "{}; t = {th} h ({})",
                t(&r, "value_class")?,
                t(&r, "time_basis")?
            ),
            if t(&r, "parfume_value_same_source")?.is_empty() {
                String::new()
            } else {
                sci(t(&r, "parfume_value_same_source")?)
            },
            t(&r, "notes")?.to_string(),
            t(&r, "particle_applicability")?.to_string(),
            t(&r, "source")?.to_string(),
            t(&r, "source_location")?.to_string(),
            link(t(&r, "link")?),
            t(&r, "evidence_class")?.to_string(),
            t(&r, "status")?.to_string(),
        ]);
    }
    Ok(table(
        &[
            "No.",
            "Variable",
            "Symbol",
            "Value",
            "Unit",
            "Material / Nuclide",
            "Temperature range",
            "Uncertainty / data class / time",
            "PARFUME, same source (calculated)",
            "Note",
            "Applicability to one intact particle",
            "Source / APA citation",
            "Location",
            "Link",
            "Evidence class",
            "Status",
        ],
        &rows,
    ))
}

/// The table of one marker: `spec` is `KIND ARGS...`.
///
/// # Errors
///
/// An unknown kind or a cell the generator cannot read.
pub fn render_table(tb: &Tables, spec: &str) -> Res<String> {
    let mut words = spec.split_whitespace();
    let kind = words.next().ok_or("an empty marker")?;
    let args: Vec<&str> = words.collect();
    match kind {
        "geom" => geom(tb, &args),
        "radii" => radii(tb, &args),
        "arr" => arrh(tb, &args),
        "decay" => decay_table(tb),
        "fy" => fyt(tb, &args),
        "temp" => temp(tb, &args),
        "rel" => relt(tb, &args),
        other => Err(format!("unknown kind of generated table `{other}`")),
    }
}

/// Checks (and with `write` regenerates) every marked table of `document`. Returns the new text
/// (equal to `document` where nothing changed, and always equal outside the markers) and a report
/// per table.
///
/// # Errors
///
/// A marker without its end, a nested marker, an unknown kind or a cell that cannot be read.
pub fn process(
    document: &str,
    tb: &Tables,
    write: bool,
) -> Result<(String, Vec<BlockReport>), String> {
    let eol = if document.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut out = String::with_capacity(document.len());
    let mut reports = Vec::new();
    let mut lines = document.split_inclusive('\n').peekable();
    let mut number = 0usize;
    while let Some(line) = lines.next() {
        number += 1;
        let trimmed = line.trim_end_matches(['\r', '\n']);
        let Some(rest) = trimmed.strip_prefix(BEGIN) else {
            if trimmed == END {
                return Err(format!(
                    "line {number}: a closing marker without an opening one"
                ));
            }
            out.push_str(line);
            continue;
        };
        let spec = rest
            .strip_suffix(" -->")
            .ok_or_else(|| format!("line {number}: the marker does not end with ` -->`"))?
            .trim()
            .to_string();
        let open = number;
        let mut old: Vec<&str> = Vec::new();
        let mut closed = false;
        let mut end_line = "";
        for l in lines.by_ref() {
            number += 1;
            if l.trim_end_matches(['\r', '\n']) == END {
                closed = true;
                end_line = l;
                break;
            }
            if l.trim_end_matches(['\r', '\n']).starts_with(BEGIN) {
                return Err(format!("line {number}: a marker inside a generated table"));
            }
            old.push(l);
        }
        if !closed {
            return Err(format!("line {open}: the marker has no closing line"));
        }
        let generated = render_table(tb, &spec).map_err(|e| format!("line {open}: {spec}: {e}"))?;
        let new_lines: Vec<&str> = generated.split('\n').collect();
        let old_trim: Vec<&str> = old
            .iter()
            .map(|l| l.trim_end_matches(['\r', '\n']))
            .collect();
        let first_difference = new_lines
            .iter()
            .zip(old_trim.iter())
            .find(|(a, b)| a != b)
            .map(|(a, b)| format!("generated `{}` against document `{}`", cut(a), cut(b)))
            .or_else(|| {
                (new_lines.len() != old_trim.len()).then(|| {
                    format!(
                        "the document has {} lines, the generated table {}",
                        old_trim.len(),
                        new_lines.len()
                    )
                })
            });
        reports.push(BlockReport {
            line: open,
            spec: spec.clone(),
            lines: new_lines.len(),
            current: first_difference.is_none(),
            first_difference,
        });
        out.push_str(line);
        if write {
            for l in &new_lines {
                out.push_str(l);
                out.push_str(eol);
            }
        } else {
            for l in &old {
                out.push_str(l);
            }
        }
        out.push_str(end_line);
    }
    Ok((out, reports))
}

fn cut(s: &str) -> String {
    s.chars().take(90).collect()
}
