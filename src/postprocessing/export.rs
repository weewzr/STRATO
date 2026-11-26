//! Data export of a BTCS run as CSV and JSON text, with a provenance header (pseudocode PS-18).
//!
//! The functions build strings and touch no file, clock, host name or environment variable, so
//! the same code serves the native command line and the WebAssembly wrapper. Numbers are
//! written with Rust's `{:e}` formatting, the shortest decimal form that reads back to the same
//! `f64`, so a file read back in Rust is bit-identical to the run that wrote it. A non-finite
//! value is never written (the export refuses it). The only empty CSV cell, and the only JSON
//! `null` in an array, is the release rate at `t = 0`, where it is not defined (Document 2
//! Section 7.2).
//!
//! Every output carries the label "numerical solution of the stated model": a file records
//! what the solver computed, not an experimental measurement and not a validated prediction.
//! No absolute path, user name or host name is written; the input file is recorded by its file
//! name and by the SHA-256 digest of the canonical text of the validated input.

use crate::{
    config::Config,
    numerical::{
        btcs::RunResult,
        diagnostics::{check_series, TimeSeries},
        grid::{Grid, NodeClass, Region, ValidatedInput},
        inventory::inventory_by_region,
    },
};
use anyhow::{bail, Result};
use std::fmt::Write as _;

/// The label every export carries.
pub const LABEL: &str = "numerical solution of the stated model; not experimental validation";

/// The solver identifier written in every header.
pub const SOLVER_ID: &str = "STRATO BTCS five-layer spherical finite difference";

/// Commit recorded when the build was given one through the environment variable
/// `STRATO_BUILD_COMMIT` at compile time, else `"unknown"`.
pub fn build_commit() -> &'static str {
    match option_env!("STRATO_BUILD_COMMIT") {
        Some(c) if !c.trim().is_empty() => c,
        _ => "unknown",
    }
}

/// Where a run came from: solver version, commit and inputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provenance {
    /// Solver identifier ([`SOLVER_ID`]).
    pub solver: String,
    /// Version of the `strato` package that was compiled.
    pub package_version: String,
    /// Git commit of the source tree, as given by the caller (`"unknown"` if not known).
    pub commit: String,
    /// Case name of the input.
    pub case_name: String,
    /// Species label of the input.
    pub species: String,
    /// Name of the input file (no directory), or an empty string when there is none.
    pub input_file: String,
    /// SHA-256 digest (hex) of [`canonical_input`].
    pub input_sha256: String,
    /// Source row of the physical parameters (radii, diffusivities, initial concentration). The
    /// TOML input format carries no source field, so the default says exactly that; a caller
    /// that knows better (a synthetic benchmark, a register row) sets it with
    /// [`Provenance::with_parameter_source`].
    pub parameter_source: String,
    /// Class of the physical parameters (decision 31), set together with the source.
    pub parameter_class: String,
}

/// Default of [`Provenance::parameter_source`].
pub const DEFAULT_PARAMETER_SOURCE: &str =
    "input file as supplied; the input format records no source row";
/// Default of [`Provenance::parameter_class`].
pub const DEFAULT_PARAMETER_CLASS: &str = "user-supplied value, no source class recorded";
/// Source and class of the synthetic benchmark inputs of this repository.
pub const SYNTHETIC_SOURCE: &str = "synthetic benchmark, not literature data";

impl Provenance {
    /// Builds the provenance of a run. `input_file` is reduced to its last path component.
    pub fn new(cfg: &Config, input: &ValidatedInput, input_file: &str, commit: &str) -> Self {
        let file = input_file
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("")
            .to_string();
        Provenance {
            solver: SOLVER_ID.to_string(),
            package_version: env!("CARGO_PKG_VERSION").to_string(),
            commit: if commit.trim().is_empty() {
                "unknown".to_string()
            } else {
                commit.trim().to_string()
            },
            case_name: cfg.case_name.clone(),
            species: cfg.species.clone(),
            input_file: file,
            input_sha256: sha256_hex(canonical_input(cfg, input).as_bytes()),
            parameter_source: DEFAULT_PARAMETER_SOURCE.to_string(),
            parameter_class: DEFAULT_PARAMETER_CLASS.to_string(),
        }
    }

    /// Sets the source row and class written beside every physical parameter.
    pub fn with_parameter_source(mut self, source: &str, class: &str) -> Self {
        self.parameter_source = one_line(source);
        self.parameter_class = one_line(class);
        self
    }
}

/// One exported column: the stable machine key and the descriptive header (full physical name
/// with its unit, decision 30). Headers contain no comma, so they need no CSV quoting.
pub type Column = (&'static str, &'static str);

/// Columns of the time-series CSV.
pub const SERIES_COLUMNS: [Column; 10] = [
    ("t_s", "Time (s)"),
    ("inventory_mol", "Remaining inventory (mol)"),
    ("cumulative_release_mol", "Cumulative release (mol)"),
    ("release_fraction", "Release fraction (dimensionless)"),
    ("release_rate_mol_s", "Release rate (mol/s)"),
    ("balance_residual_mol", "Mass-balance residual (mol)"),
    (
        "balance_residual_relative",
        "Relative mass-balance residual (dimensionless)",
    ),
    (
        "balance_residual_bound_mol",
        "Derived a-posteriori rounding bound of the mass-balance residual (mol)",
    ),
    (
        "min_concentration_mol_m3",
        "Minimum concentration over all nodes (mol/m^3)",
    ),
    (
        "max_concentration_mol_m3",
        "Maximum concentration over all nodes (mol/m^3)",
    ),
];

/// Columns of the profile CSV.
pub const PROFILE_COLUMNS: [Column; 6] = [
    ("t_s", "Time (s)"),
    ("node_index", "Node index (dimensionless)"),
    ("r_m", "Radius (m)"),
    ("region", "Material region"),
    ("node_class", "Node class"),
    ("concentration_mol_m3", "Concentration (mol/m^3)"),
];

/// Columns of the independent-check CSV of the regression suite.
pub const CHECK_COLUMNS: [Column; 17] = [
    ("t_s", "Time (s)"),
    (
        "f_btcs",
        "Release fraction computed by BTCS (dimensionless)",
    ),
    (
        "f_reference",
        "Release fraction of the independent reference (dimensionless)",
    ),
    (
        "abs_difference",
        "Absolute difference of the release fractions (dimensionless)",
    ),
    (
        "time_error",
        "Estimated time-discretisation error of the BTCS run (dimensionless)",
    ),
    ("time_order", "Observed order in time (dimensionless)"),
    (
        "time_uncertainty",
        "Uncertainty of the time-error estimate (dimensionless)",
    ),
    (
        "space_error",
        "Estimated space-discretisation error of the BTCS run (dimensionless)",
    ),
    ("space_order", "Observed order in space (dimensionless)"),
    (
        "space_uncertainty",
        "Uncertainty of the space-error estimate (dimensionless)",
    ),
    (
        "cross_error",
        "Estimated cross term of the time and space errors (dimensionless)",
    ),
    (
        "f_limit",
        "Estimated numerical limit of the release fraction (dimensionless)",
    ),
    (
        "limit_difference",
        "Absolute difference of the reference and the estimated limit (dimensionless)",
    ),
    (
        "reference_floor",
        "Accuracy floor of the independent reference (dimensionless)",
    ),
    (
        "tolerance",
        "Convention fraction of the difference plus reference floor (dimensionless)",
    ),
    (
        "limit_difference_over_tolerance",
        "Limit difference divided by tolerance (dimensionless)",
    ),
    ("passed", "Check passed (true or false)"),
];

/// The CSV header line of a column table.
pub fn header_line(columns: &[Column]) -> String {
    let mut s = columns.iter().map(|c| c.1).collect::<Vec<_>>().join(",");
    s.push('\n');
    s
}

/// The machine key of a descriptive header, over all column tables.
pub fn key_of_header(header: &str) -> Option<&'static str> {
    SERIES_COLUMNS
        .iter()
        .chain(PROFILE_COLUMNS.iter())
        .chain(CHECK_COLUMNS.iter())
        .find(|c| c.1 == header)
        .map(|c| c.0)
}

/// Canonical text of the validated input: one `key=value` line per quantity, numbers in the
/// shortest round-trip form. It does not depend on the layout, the comments or the line endings
/// of the TOML file, so the digest identifies the model input, not the file.
pub fn canonical_input(cfg: &Config, input: &ValidatedInput) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "species={}", cfg.species);
    let _ = writeln!(s, "temperature_k={:e}", cfg.temperature_k);
    for (k, region) in Region::ALL.iter().enumerate() {
        let name = region.name();
        let _ = writeln!(s, "outer_radius_{name}_m={:e}", input.radii[k]);
        let _ = writeln!(s, "diffusivity_{name}_m2_s={:e}", input.d[k]);
        let _ = writeln!(s, "intervals_{name}={}", input.n_intervals[k]);
    }
    let _ = writeln!(s, "c_init_mol_m3={:e}", input.c_init);
    let _ = writeln!(s, "dt_s={:e}", input.dt);
    let _ = writeln!(s, "steps={}", input.steps);
    let _ = writeln!(s, "output_every_steps={}", input.output_every);
    s
}

pub(crate) fn num(x: f64) -> Result<String> {
    if !x.is_finite() {
        bail!("export refuses a non-finite value");
    }
    Ok(format!("{x:e}"))
}

pub(crate) fn comment_header(prov: &Provenance, what: &str) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "# {SOLVER_ID}: {what}");
    let _ = writeln!(s, "# label: {LABEL}");
    let _ = writeln!(s, "# solver_package_version: {}", prov.package_version);
    let _ = writeln!(s, "# commit: {}", prov.commit);
    let _ = writeln!(s, "# case: {}", one_line(&prov.case_name));
    let _ = writeln!(s, "# species: {}", one_line(&prov.species));
    let _ = writeln!(s, "# input_file: {}", one_line(&prov.input_file));
    let _ = writeln!(s, "# input_sha256: {}", prov.input_sha256);
    let _ = writeln!(
        s,
        "# parameter_source: {}",
        one_line(&prov.parameter_source)
    );
    let _ = writeln!(s, "# parameter_class: {}", one_line(&prov.parameter_class));
    s
}

pub(crate) fn one_line(s: &str) -> String {
    s.replace(['\n', '\r'], " ")
}

/// Time series as CSV. The header row is [`SERIES_COLUMNS`]: the full physical name with its
/// unit of time, remaining inventory, cumulative release, release fraction, release rate
/// (empty at `t = 0`), mass-balance residual, its relative value, the derived a-posteriori
/// rounding bound reported beside the observed residual, and the minimum and maximum
/// concentration.
///
/// # Errors
///
/// A non-finite value.
pub fn timeseries_csv(
    prov: &Provenance,
    series: &TimeSeries,
    initial_inventory: f64,
) -> Result<String> {
    let mut s = comment_header(prov, "time series");
    s.push_str(&header_line(&SERIES_COLUMNS));
    for (t, row) in series {
        let rate = match row.release_rate {
            Some(r) => num(r)?,
            None => String::new(),
        };
        let _ = writeln!(
            s,
            "{},{},{},{},{},{},{},{},{},{}",
            num(*t)?,
            num(row.inventory)?,
            num(row.cumulative_release)?,
            num(row.release_fraction)?,
            rate,
            num(row.balance_residual)?,
            num(row.balance_residual / initial_inventory)?,
            num(row.balance_residual_bound)?,
            num(row.min_concentration)?,
            num(row.max_concentration)?
        );
    }
    Ok(s)
}

fn class_and_region(grid: &Grid, i: usize) -> (&'static str, String) {
    match grid.node_class(i) {
        NodeClass::Centre => ("centre", Region::Kernel.name().to_string()),
        NodeClass::Interior(r) => ("interior", r.name().to_string()),
        NodeClass::Interface { inner, outer } => {
            ("interface", format!("{}|{}", inner.name(), outer.name()))
        }
        NodeClass::Outer => ("outer", Region::Opyc.name().to_string()),
    }
}

/// Radial profiles as CSV, one block of `N + 1` rows per snapshot. The header row is
/// [`PROFILE_COLUMNS`]: time, node index, radius, material region (the material of an interior
/// node; `inner|outer` at an interface node), node class (`centre`, `interior`, `interface`,
/// `outer`) and concentration.
/// The centre value is reported from its own row and carries no inventory; the outer node is
/// zero by (F18).
///
/// # Errors
///
/// A non-finite value, or a profile of the wrong length.
pub fn profile_csv(
    prov: &Provenance,
    grid: &Grid,
    snapshots: &[(f64, Vec<f64>)],
) -> Result<String> {
    let mut s = comment_header(prov, "radial concentration profiles");
    s.push_str(&header_line(&PROFILE_COLUMNS));
    for (t, c) in snapshots {
        if c.len() != grid.n + 1 {
            bail!(
                "profile length {} does not match the grid ({})",
                c.len(),
                grid.n + 1
            );
        }
        for i in 0..=grid.n {
            let (class, region) = class_and_region(grid, i);
            let _ = writeln!(
                s,
                "{},{},{},{},{},{}",
                num(*t)?,
                i,
                num(grid.r[i])?,
                region,
                class,
                num(c[i])?
            );
        }
    }
    Ok(s)
}

pub(crate) fn json_str(x: &str) -> String {
    let mut s = String::from("\"");
    for ch in x.chars() {
        match ch {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(s, "\\u{:04x}", c as u32);
            }
            c => s.push(c),
        }
    }
    s.push('"');
    s
}

pub(crate) fn json_arr(values: &[f64]) -> Result<String> {
    let mut s = String::from("[");
    for (k, v) in values.iter().enumerate() {
        if k > 0 {
            s.push(',');
        }
        s.push_str(&num(*v)?);
    }
    s.push(']');
    Ok(s)
}

/// The whole run as one JSON object: `provenance` (with the source row and class of the
/// physical parameters), `label`, `input`, `parameters` (each with full name, unit, source row
/// and class), `column_dictionary` (the descriptive header of every machine key used in
/// `series` and in the CSV files), `grid`, `run`
/// (including the matrix and pivot diagnostics), `diagnostics` (series-level properties),
/// `series` (column arrays, `release_rate_mol_s` has `null` at `t = 0`) and `snapshots`
/// (profiles with the inventory of each material).
///
/// # Errors
///
/// A non-finite value.
pub fn run_json(prov: &Provenance, input: &ValidatedInput, run: &RunResult) -> Result<String> {
    let g = &run.grid;
    let m = &run.meta;
    let names: Vec<&str> = Region::ALL.iter().map(|r| r.name()).collect();
    let mut s = String::from("{\n");
    // Provenance and label.
    let _ = writeln!(
        s,
        "\"provenance\":{{\"solver\":{},\"package_version\":{},\"commit\":{},\"case\":{},\"species\":{},\"input_file\":{},\"input_sha256\":{},\"parameter_source\":{},\"parameter_class\":{}}},",
        json_str(&prov.solver),
        json_str(&prov.package_version),
        json_str(&prov.commit),
        json_str(&prov.case_name),
        json_str(&prov.species),
        json_str(&prov.input_file),
        json_str(&prov.input_sha256),
        json_str(&prov.parameter_source),
        json_str(&prov.parameter_class)
    );
    let _ = writeln!(s, "\"label\":{},", json_str(LABEL));
    // Input.
    let _ = writeln!(
        s,
        "\"input\":{{\"materials\":{},\"outer_radius_m\":{},\"diffusivity_m2_s\":{},\"intervals\":[{}],\"c_init_mol_m3\":{},\"dt_s\":{},\"steps\":{},\"output_every_steps\":{}}},",
        json_arr_str(&names),
        json_arr(&input.radii)?,
        json_arr(&input.d)?,
        input
            .n_intervals
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(","),
        num(input.c_init)?,
        num(input.dt)?,
        input.steps,
        input.output_every
    );
    // Parameters with full names, units, source row and class (decision 31).
    let mut params = Vec::new();
    for (k, region) in Region::ALL.iter().enumerate() {
        let name = region.name();
        params.push(param_json(
            &format!("Outer radius of the {name} region (m)"),
            &num(input.radii[k])?,
            &prov.parameter_source,
            &prov.parameter_class,
        ));
        params.push(param_json(
            &format!("Diffusion coefficient in the {name} region (m^2/s)"),
            &num(input.d[k])?,
            &prov.parameter_source,
            &prov.parameter_class,
        ));
    }
    params.push(param_json(
        "Initial kernel concentration (mol/m^3)",
        &num(input.c_init)?,
        &prov.parameter_source,
        &prov.parameter_class,
    ));
    let setting = "numerical setting chosen by the user";
    params.push(param_json(
        "Time step (s)",
        &num(input.dt)?,
        setting,
        "numerical setting",
    ));
    let _ = writeln!(s, "\"parameters\":[{}],", params.join(","));
    let _ = writeln!(
        s,
        "\"column_dictionary\":{{\"series\":{},\"profile\":{}}},",
        dictionary_json(&SERIES_COLUMNS),
        dictionary_json(&PROFILE_COLUMNS)
    );
    // Grid.
    let _ = writeln!(
        s,
        "\"grid\":{{\"n\":{},\"interface_nodes\":[{}],\"dr_material_m\":{},\"r_m\":{}}},",
        g.n,
        g.interface_nodes
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(","),
        json_arr(&g.dr_region)?,
        json_arr(&g.r)?
    );
    // Run metadata and matrix diagnostics.
    let _ = writeln!(
        s,
        "\"run\":{{\"dt_s\":{},\"steps\":{},\"theta\":{},\"initial_inventory_mol\":{},\"fourier_number_material\":{},\"matrix_min_row_margin\":{},\"matrix_max_interior_margin_deviation\":{},\"matrix_norm_inf\":{},\"min_pivot_margin\":{},\"max_multiplier\":{}}},",
        num(m.dt)?,
        m.steps,
        num(m.theta)?,
        num(m.initial_inventory)?,
        json_arr(&m.fo)?,
        num(m.matrix.min_row_margin)?,
        num(m.matrix.max_interior_margin_deviation)?,
        num(m.matrix.norm_inf)?,
        num(m.min_pivot_margin)?,
        num(m.max_multiplier)?
    );
    // Series-level diagnostics.
    let rep = check_series(&run.series, m.initial_inventory);
    let last = &run.series[run.series.len() - 1].1;
    let _ = writeln!(
        s,
        "\"diagnostics\":{{\"worst_balance_residual_relative\":{},\"worst_residual_row\":{},\"final_balance_residual_bound_relative\":{},\"release_monotone\":{},\"release_in_range\":{},\"max_inventory_increase_relative\":{},\"min_concentration_mol_m3\":{},\"max_concentration_mol_m3\":{}}},",
        num(rep.worst_relative_residual)?,
        rep.worst_residual_row,
        num(last.balance_residual_bound / m.initial_inventory)?,
        rep.release_monotone,
        rep.release_in_range,
        num(rep.max_inventory_increase_rel)?,
        num(rep.min_concentration)?,
        num(rep.max_concentration)?
    );
    // Series.
    let col =
        |f: &dyn Fn(&crate::numerical::diagnostics::StepDiagnostics) -> f64| -> Result<String> {
            json_arr(&run.series.iter().map(|(_, r)| f(r)).collect::<Vec<_>>())
        };
    let times: Vec<f64> = run.series.iter().map(|(t, _)| *t).collect();
    let mut rates = String::from("[");
    for (k, (_, r)) in run.series.iter().enumerate() {
        if k > 0 {
            rates.push(',');
        }
        match r.release_rate {
            Some(v) => rates.push_str(&num(v)?),
            None => rates.push_str("null"),
        }
    }
    rates.push(']');
    let rel_resid: Vec<f64> = run
        .series
        .iter()
        .map(|(_, r)| r.balance_residual / m.initial_inventory)
        .collect();
    let _ = writeln!(
        s,
        "\"series\":{{\"t_s\":{},\"inventory_mol\":{},\"cumulative_release_mol\":{},\"release_fraction\":{},\"release_rate_mol_s\":{},\"balance_residual_mol\":{},\"balance_residual_relative\":{},\"balance_residual_bound_mol\":{},\"min_concentration_mol_m3\":{},\"max_concentration_mol_m3\":{}}},",
        json_arr(&times)?,
        col(&|r| r.inventory)?,
        col(&|r| r.cumulative_release)?,
        col(&|r| r.release_fraction)?,
        rates,
        col(&|r| r.balance_residual)?,
        json_arr(&rel_resid)?,
        col(&|r| r.balance_residual_bound)?,
        col(&|r| r.min_concentration)?,
        col(&|r| r.max_concentration)?
    );
    // Snapshots.
    s.push_str("\"snapshots\":[");
    for (k, (t, c)) in run.snapshots.iter().enumerate() {
        if k > 0 {
            s.push(',');
        }
        let parts = inventory_by_region(g, c);
        let _ = write!(
            s,
            "\n{{\"t_s\":{},\"concentration_mol_m3\":{},\"material_inventory_mol\":{}}}",
            num(*t)?,
            json_arr(c)?,
            json_arr(&parts)?
        );
    }
    s.push_str("\n]\n}\n");
    Ok(s)
}

pub(crate) fn param_json(name: &str, value: &str, source: &str, class: &str) -> String {
    format!(
        "{{\"name\":{},\"value\":{},\"source\":{},\"class\":{}}}",
        json_str(name),
        value,
        json_str(source),
        json_str(class)
    )
}

pub(crate) fn dictionary_json(columns: &[Column]) -> String {
    let parts: Vec<String> = columns
        .iter()
        .map(|c| format!("{}:{}", json_str(c.0), json_str(c.1)))
        .collect();
    format!("{{{}}}", parts.join(","))
}

pub(crate) fn json_arr_str(items: &[&str]) -> String {
    let parts: Vec<String> = items.iter().map(|x| json_str(x)).collect();
    format!("[{}]", parts.join(","))
}

// ---------------------------------------------------------------------------------------
// SHA-256 (FIPS 180-4), for the digest of the canonical input.
// ---------------------------------------------------------------------------------------

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// SHA-256 digest of `data`, as 64 lowercase hexadecimal digits.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for block in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (t, word) in block.chunks(4).enumerate() {
            w[t] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for t in 16..64 {
            let s0 = w[t - 15].rotate_right(7) ^ w[t - 15].rotate_right(18) ^ (w[t - 15] >> 3);
            let s1 = w[t - 2].rotate_right(17) ^ w[t - 2].rotate_right(19) ^ (w[t - 2] >> 10);
            w[t] = w[t - 16]
                .wrapping_add(s0)
                .wrapping_add(w[t - 7])
                .wrapping_add(s1);
        }
        let mut v = h;
        for t in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[t])
                .wrapping_add(w[t]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v[7] = v[6];
            v[6] = v[5];
            v[5] = v[4];
            v[4] = v[3].wrapping_add(t1);
            v[3] = v[2];
            v[2] = v[1];
            v[1] = v[0];
            v[0] = t1.wrapping_add(t2);
        }
        for k in 0..8 {
            h[k] = h[k].wrapping_add(v[k]);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{InitialConfig, OutputConfig, RegionConfig, TimeConfig},
        numerical::btcs::{run_input, validate_input, Limits},
    };

    const LIMITS: Limits = Limits {
        max_intervals_total: 100_000,
        max_steps: 1_000_000,
    };

    fn config() -> Config {
        let radii = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4];
        let d = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13];
        let names = ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"];
        Config {
            case_name: "export \"test\", case".into(),
            species: "synthetic".into(),
            temperature_k: 1000.0,
            regions: (0..5)
                .map(|k| RegionConfig {
                    name: names[k].into(),
                    outer_radius_m: radii[k],
                    diffusivity_m2_s: d[k],
                    cells: 4 + k,
                })
                .collect(),
            initial: InitialConfig {
                kernel_concentration_mol_m3: 1.0,
            },
            time: TimeConfig {
                dt_s: 1000.0,
                end_time_s: 20_000.0,
                output_every_steps: 3,
            },
            output: OutputConfig {
                directory: "unused".into(),
            },
        }
    }

    fn data_rows(text: &str) -> Vec<Vec<String>> {
        let mut rows = text.lines().filter(|l| !l.starts_with('#'));
        let _header = rows.next();
        rows.map(|l| l.split(',').map(str::to_string).collect())
            .collect()
    }

    #[test]
    fn sha256_known_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        // 56 bytes: the padding spills into a second block.
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn roundtrip_is_bit_identical() {
        let cfg = config();
        let input = validate_input(&cfg, &LIMITS).unwrap();
        let run = run_input(&input, &[0.0, 7000.0, 20_000.0]).unwrap();
        let prov = Provenance::new(&cfg, &input, "some/dir\\case.toml", "abc1234");
        assert_eq!(prov.input_file, "case.toml");

        // CSV time series.
        let csv = timeseries_csv(&prov, &run.series, run.meta.initial_inventory).unwrap();
        assert!(csv.contains("# commit: abc1234"));
        assert!(csv.contains("# label: numerical solution of the stated model"));
        assert!(!csv.contains("some/dir") && !csv.contains('\\'));
        assert!(csv.contains("\nTime (s),Remaining inventory (mol),"));
        assert!(csv.contains("# parameter_source: input file as supplied"));
        let rows = data_rows(&csv);
        assert_eq!(rows.len(), run.series.len());
        for (row, (t, d)) in rows.iter().zip(&run.series) {
            let read = |k: usize| row[k].parse::<f64>().unwrap();
            assert_eq!(read(0).to_bits(), t.to_bits());
            assert_eq!(read(1).to_bits(), d.inventory.to_bits());
            assert_eq!(read(2).to_bits(), d.cumulative_release.to_bits());
            assert_eq!(read(3).to_bits(), d.release_fraction.to_bits());
            match d.release_rate {
                Some(r) => assert_eq!(read(4).to_bits(), r.to_bits()),
                None => assert!(row[4].is_empty()),
            }
            assert_eq!(read(5).to_bits(), d.balance_residual.to_bits());
            assert_eq!(read(7).to_bits(), d.balance_residual_bound.to_bits());
            assert_eq!(read(8).to_bits(), d.min_concentration.to_bits());
            assert_eq!(read(9).to_bits(), d.max_concentration.to_bits());
        }

        // CSV profiles.
        let pcsv = profile_csv(&prov, &run.grid, &run.snapshots).unwrap();
        let rows = data_rows(&pcsv);
        assert_eq!(rows.len(), run.snapshots.len() * (run.grid.n + 1));
        let mut k = 0;
        for (t, c) in &run.snapshots {
            for i in 0..=run.grid.n {
                let row = &rows[k];
                k += 1;
                assert_eq!(row[0].parse::<f64>().unwrap().to_bits(), t.to_bits());
                assert_eq!(row[1].parse::<usize>().unwrap(), i);
                assert_eq!(
                    row[2].parse::<f64>().unwrap().to_bits(),
                    run.grid.r[i].to_bits()
                );
                assert_eq!(row[5].parse::<f64>().unwrap().to_bits(), c[i].to_bits());
            }
        }
        assert!(pcsv.contains(",centre,") && pcsv.contains(",outer,"));
        assert!(pcsv.contains("kernel|buffer,interface,"));

        // JSON: arrays read back by a minimal extractor.
        let json = run_json(&prov, &input, &run).unwrap();
        let extract = |key: &str| -> Vec<Option<f64>> {
            let start = json.find(&format!("\"{key}\":[")).unwrap() + key.len() + 4;
            let end = start + json[start..].find(']').unwrap();
            json[start..end]
                .split(',')
                .map(|x| {
                    if x == "null" {
                        None
                    } else {
                        Some(x.parse().unwrap())
                    }
                })
                .collect()
        };
        let f = extract("release_fraction");
        assert_eq!(f.len(), run.series.len());
        for (v, (_, d)) in f.iter().zip(&run.series) {
            assert_eq!(v.unwrap().to_bits(), d.release_fraction.to_bits());
        }
        let rates = extract("release_rate_mol_s");
        assert_eq!(rates[0], None);
        assert_eq!(
            rates[1].unwrap().to_bits(),
            run.series[1].1.release_rate.unwrap().to_bits()
        );
        let r = extract("r_m");
        for (v, x) in r.iter().zip(&run.grid.r) {
            assert_eq!(v.unwrap().to_bits(), x.to_bits());
        }
        assert!(json.contains("\"commit\":\"abc1234\""));
        assert!(json.contains("\"t_s\":\"Time (s)\""));
        assert!(json.contains("\"name\":\"Diffusion coefficient in the kernel region (m^2/s)\""));
        assert!(json.contains("\"class\":\"user-supplied value, no source class recorded\""));
        let synthetic = prov
            .clone()
            .with_parameter_source(SYNTHETIC_SOURCE, "synthetic benchmark");
        let sj = run_json(&synthetic, &input, &run).unwrap();
        assert!(sj.contains("\"source\":\"synthetic benchmark, not literature data\""));
        for (key, header) in SERIES_COLUMNS.iter().chain(PROFILE_COLUMNS.iter()) {
            assert_eq!(key_of_header(header), Some(*key));
            assert!(!header.contains(','));
        }
        assert!(json.contains("export \\\"test\\\", case"));
        assert!(
            !json.contains("some/dir")
                && !json.contains("NaN")
                && !json.contains(":inf")
                && !json.contains(",inf")
        );
        // Structure: balanced braces and brackets outside strings.
        let (mut depth, mut in_str, mut esc) = (0i32, false, false);
        for ch in json.chars() {
            if in_str {
                match (esc, ch) {
                    (true, _) => esc = false,
                    (false, '\\') => esc = true,
                    (false, '"') => in_str = false,
                    _ => {}
                }
            } else {
                match ch {
                    '"' => in_str = true,
                    '{' | '[' => depth += 1,
                    '}' | ']' => depth -= 1,
                    _ => {}
                }
                assert!(depth >= 0);
            }
        }
        assert_eq!(depth, 0);
    }

    #[test]
    fn input_digest_ignores_layout_and_tracks_inputs() {
        let cfg = config();
        let input = validate_input(&cfg, &LIMITS).unwrap();
        let a = Provenance::new(&cfg, &input, "a.toml", "c").input_sha256;
        let b = Provenance::new(&cfg, &input, "other-name.toml", "c").input_sha256;
        assert_eq!(a, b);
        let mut cfg2 = cfg.clone();
        cfg2.regions[1].diffusivity_m2_s *= 1.0 + 1e-15;
        let input2 = validate_input(&cfg2, &LIMITS).unwrap();
        assert_ne!(
            a,
            Provenance::new(&cfg2, &input2, "a.toml", "c").input_sha256
        );
        assert_eq!(a.len(), 64);
        assert_eq!(
            Provenance::new(&cfg, &input, "a.toml", "  ").commit,
            "unknown"
        );
    }

    #[test]
    fn non_finite_values_are_refused() {
        assert!(num(f64::NAN).is_err());
        assert!(num(f64::INFINITY).is_err());
        assert_eq!(num(0.0).unwrap(), "0e0");
        assert_eq!(num(1.5e-12).unwrap().parse::<f64>().unwrap(), 1.5e-12);
    }
}
