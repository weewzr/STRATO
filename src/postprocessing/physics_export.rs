//! Data export of a run with generation, decay and temperature-dependent diffusion, as CSV
//! and JSON text with the provenance header of [`super::export`] (pseudocode PS-18, extended
//! by the hooks of PS-19).
//!
//! Like the baseline export, the functions build strings and touch no file, clock, host name or
//! environment variable. Every column header is the full physical name with its unit
//! (decision 30). The residual columns follow the balance (F41): the relative residual is
//! relative to the initial inventory plus the generated amount, the bound beside it is the one
//! the run used (the baseline bound, or (F46)), and the generated and decayed amounts are
//! separate columns that are never added to the release. A fission-rate density is exported as
//! "user-supplied" unless the input records another origin, and never as measured or
//! validated (decisions 14 and 42). The result is a numerical solution of the stated model;
//! it is not experimental validation.

use super::export::{
    canonical_input, comment_header, dictionary_json, header_line, json_arr, json_str, num,
    param_json, profile_csv, sha256_hex, Column, Provenance, LABEL,
};
use crate::{
    config::Config,
    numerical::{
        grid::Region,
        inventory::inventory_by_region,
        physics_config::PhysicsLabels,
        physics_run::{
            BoundKind, DiffusivityModel, Interpolation, PhysicsInput, PhysicsRun, TimeFunction,
        },
    },
};
use anyhow::Result;
use std::fmt::Write as _;

/// Columns of the time-series CSV of a run with the additional mechanisms.
pub const PHYSICS_SERIES_COLUMNS: [Column; 15] = [
    ("t_s", "Time (s)"),
    ("inventory_mol", "Remaining inventory (mol)"),
    ("cumulative_release_mol", "Cumulative release (mol)"),
    (
        "generated_mol",
        "Cumulative generated amount excluding the initial inventory (mol)",
    ),
    (
        "decayed_mol",
        "Cumulative amount decayed inside the particle and not part of the release (mol)",
    ),
    (
        "release_fraction_of_initial_inventory",
        "Release fraction of the initial inventory (dimensionless)",
    ),
    (
        "release_fraction_of_total",
        "Release fraction of the initial inventory plus the generated amount (dimensionless)",
    ),
    (
        "release_to_generation_ratio",
        "Release rate divided by generation rate (dimensionless)",
    ),
    ("release_rate_mol_s", "Release rate (mol/s)"),
    ("balance_residual_mol", "Mass-balance residual (mol)"),
    (
        "balance_residual_relative_to_total",
        "Mass-balance residual relative to the initial inventory plus the generated amount (dimensionless)",
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
    ("temperature_k", "Particle temperature (K)"),
];

fn opt(x: Option<f64>) -> Result<String> {
    match x {
        Some(v) => num(v),
        None => Ok(String::new()),
    }
}

/// Canonical text of the input of a run with the additional mechanisms: the baseline canonical
/// text followed by one line per physics quantity, numbers in the shortest round-trip form. Its
/// SHA-256 digest replaces the baseline digest in the provenance of such a run.
pub fn canonical_physics_input(cfg: &Config, input: &PhysicsInput) -> String {
    let mut s = canonical_input(cfg, &input.base);
    let _ = writeln!(s, "decay_constant_per_s={:e}", input.decay_constant_per_s);
    let names = ["kernel", "buffer", "ipyc", "sic", "opyc"];
    for (k, g) in input.source.per_material.iter().enumerate() {
        let _ = writeln!(s, "source_{}_mol_m3_s={}", names[k], history_text(g));
    }
    match &input.diffusivity {
        DiffusivityModel::Constant(d) => {
            let text: Vec<String> = d.iter().map(|x| format!("{x:e}")).collect();
            let _ = writeln!(s, "diffusivity_m2_s=constant:{}", text.join(";"));
        }
        DiffusivityModel::Arrhenius {
            laws,
            gas,
            temperature_k,
        } => {
            let _ = writeln!(
                s,
                "gas_constant_j_mol_k={:e}\ntemperature_k={}",
                gas.value(),
                history_text(temperature_k)
            );
            for (k, law) in laws.iter().enumerate() {
                let terms: Vec<String> = law
                    .terms()
                    .iter()
                    .map(|t| format!("{:e}/{:e}", t.d0(), t.ea_j_per_mol()))
                    .collect();
                let _ = writeln!(s, "arrhenius_{}={}", names[k], terms.join("+"));
            }
        }
    }
    s
}

fn history_text(f: &TimeFunction) -> String {
    match f {
        TimeFunction::Constant(v) => format!("constant:{v:e}"),
        TimeFunction::Table {
            times,
            values,
            mode,
        } => {
            let m = match mode {
                Interpolation::Step => "step",
                Interpolation::Linear => "linear",
            };
            let pairs: Vec<String> = times
                .iter()
                .zip(values)
                .map(|(t, v)| format!("{t:e}:{v:e}"))
                .collect();
            format!("{m}:{}", pairs.join(";"))
        }
    }
}

/// Provenance of a run with the additional mechanisms: as [`Provenance::new`], with the digest
/// of [`canonical_physics_input`].
pub fn physics_provenance(
    cfg: &Config,
    input: &PhysicsInput,
    input_file: &str,
    commit: &str,
) -> Provenance {
    let mut p = Provenance::new(cfg, &input.base, input_file, commit);
    p.input_sha256 = sha256_hex(canonical_physics_input(cfg, input).as_bytes());
    p
}

/// Time series as CSV. The header row is [`PHYSICS_SERIES_COLUMNS`]. Empty cells: the release
/// rate and the ratio at `t = 0` (not defined), the ratio while the generation rate is zero,
/// and the temperature for a constant diffusivity.
///
/// # Errors
///
/// A non-finite value.
pub fn physics_timeseries_csv(prov: &Provenance, run: &PhysicsRun) -> Result<String> {
    let mut s = comment_header(prov, "time series with generation, decay and temperature");
    let _ = writeln!(
        s,
        "# balance: initial + generated = remaining + released + decayed (Document 2, F41)"
    );
    let _ = writeln!(
        s,
        "# residual bound: {}",
        match run.meta.bound_kind {
            BoundKind::Baseline =>
                "baseline derived bound (no source, no decay, constant diffusivity)",
            BoundKind::Combined => "combined-scheme rounding bound (F46)",
        }
    );
    s.push_str(&header_line(&PHYSICS_SERIES_COLUMNS));
    for (t, d, p) in &run.series {
        let _ = writeln!(
            s,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            num(*t)?,
            num(d.inventory)?,
            num(d.cumulative_release)?,
            num(p.generated)?,
            num(p.decayed)?,
            num(d.release_fraction)?,
            num(p.release_fraction_of_total)?,
            opt(p.release_to_generation_ratio)?,
            opt(d.release_rate)?,
            num(d.balance_residual)?,
            num(p.balance_residual_relative)?,
            num(d.balance_residual_bound)?,
            num(d.min_concentration)?,
            num(d.max_concentration)?,
            opt(p.temperature_k)?
        );
    }
    Ok(s)
}

/// Radial profiles as CSV (same layout as the baseline export).
///
/// # Errors
///
/// A non-finite value, or a profile of the wrong length.
pub fn physics_profile_csv(prov: &Provenance, run: &PhysicsRun) -> Result<String> {
    profile_csv(prov, &run.grid, &run.snapshots)
}

fn history_json(f: &TimeFunction) -> Result<String> {
    Ok(match f {
        TimeFunction::Constant(v) => {
            format!("{{\"interpolation\":\"constant\",\"value\":{}}}", num(*v)?)
        }
        TimeFunction::Table {
            times,
            values,
            mode,
        } => format!(
            "{{\"interpolation\":{},\"times_s\":{},\"values\":{}}}",
            json_str(match mode {
                Interpolation::Step => "step",
                Interpolation::Linear => "linear",
            }),
            json_arr(times)?,
            json_arr(values)?
        ),
    })
}

/// The whole run as one JSON object: `provenance`, `label`, `input`, `physics` (decay, source,
/// diffusivity laws with their source rows and classes, temperature history, range report,
/// which rounding bound was used), `parameters` (each with full name, unit, source row and
/// class), `column_dictionary`, `grid`, `run`, `diagnostics`, `series` (column arrays, with
/// `null` where a value is not defined) and `snapshots`.
///
/// # Errors
///
/// A non-finite value.
pub fn physics_run_json(
    prov: &Provenance,
    input: &PhysicsInput,
    labels: &PhysicsLabels,
    run: &PhysicsRun,
) -> Result<String> {
    let g = &run.grid;
    let m = &run.meta;
    let b = &input.base;
    let names: Vec<&str> = Region::ALL.iter().map(|r| r.name()).collect();
    let mut s = String::from("{\n");
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
    let _ = writeln!(
        s,
        "\"input\":{{\"materials\":[{}],\"outer_radius_m\":{},\"intervals\":[{}],\"c_init_mol_m3\":{},\"dt_s\":{},\"steps\":{},\"output_every_steps\":{},\"end_time_s\":{}}},",
        names.iter().map(|n| json_str(n)).collect::<Vec<_>>().join(","),
        json_arr(&b.radii)?,
        b.n_intervals
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(","),
        num(b.c_init)?,
        num(b.dt)?,
        b.steps,
        b.output_every,
        num(input.end_time_s)?
    );

    // Physics block.
    let mut ph = String::new();
    let _ = write!(
        ph,
        "\"decay\":{{\"decay_constant_per_s\":{},\"decay_number_lambda_dt\":{},\"label\":{}}},",
        num(input.decay_constant_per_s)?,
        num(m.decay_number)?,
        json_str(&labels.decay)
    );
    let src: Vec<String> = input
        .source
        .per_material
        .iter()
        .enumerate()
        .map(|(k, f)| {
            Ok(format!(
                "{{\"material\":{},\"source_mol_m3_s\":{}}}",
                json_str(names[k]),
                history_json(f)?
            ))
        })
        .collect::<Result<_>>()?;
    let _ = write!(
        ph,
        "\"source\":{{\"label\":{},\"fission_rate_density_origin\":{},\"per_material\":[{}]}},",
        json_str(&labels.source),
        match &labels.fission_rate_origin {
            Some(o) => json_str(o),
            None => "null".to_string(),
        },
        src.join(",")
    );
    match &input.diffusivity {
        DiffusivityModel::Constant(d) => {
            let _ = write!(
                ph,
                "\"diffusivity\":{{\"model\":\"constant\",\"diffusivity_m2_s\":{},\"label\":{}}},",
                json_arr(d)?,
                json_str(&labels.diffusivity.join(" | "))
            );
        }
        DiffusivityModel::Arrhenius {
            laws,
            gas,
            temperature_k,
        } => {
            let mut rows = Vec::new();
            for (k, law) in laws.iter().enumerate() {
                let terms: Vec<String> = law
                    .terms()
                    .iter()
                    .map(|t| {
                        Ok(format!(
                            "{{\"d0_m2_s\":{},\"activation_energy_j_per_mol\":{}}}",
                            num(t.d0())?,
                            num(t.ea_j_per_mol())?
                        ))
                    })
                    .collect::<Result<_>>()?;
                let r = &m.ranges[k];
                rows.push(format!(
                    "{{\"material\":{},\"form\":{},\"terms\":[{}],\"row\":{},\"steps_within_range\":{},\"steps_below_range\":{},\"steps_above_range\":{},\"steps_range_not_stated\":{}}}",
                    json_str(names[k]),
                    json_str(if law.terms().len() == 1 { "single term" } else { "sum of terms" }),
                    terms.join(","),
                    json_str(labels.diffusivity.get(k).map_or("", String::as_str)),
                    r.within,
                    r.below,
                    r.above,
                    r.not_stated
                ));
            }
            let _ = write!(
                ph,
                "\"diffusivity\":{{\"model\":\"arrhenius\",\"gas_constant_j_mol_k\":{},\"temperature_k\":{},\"laws\":[{}]}},",
                num(gas.value())?,
                history_json(temperature_k)?,
                rows.join(",")
            );
        }
    }
    let _ = write!(
        ph,
        "\"balance_bound\":{{\"kind\":{},\"description\":{}}},\"matrix_assemblies\":{}",
        json_str(match m.bound_kind {
            BoundKind::Baseline => "baseline",
            BoundKind::Combined => "combined",
        }),
        json_str(match m.bound_kind {
            BoundKind::Baseline =>
                "derived baseline bound; no source, no decay, constant diffusivity",
            BoundKind::Combined =>
                "rounding bound (F46) of Document 2 Section 11.7 for source, decay or temperature history",
        }),
        m.assemblies
    );
    let _ = writeln!(s, "\"physics\":{{{ph}}},");

    // Parameters.
    let mut params = Vec::new();
    for (k, name) in names.iter().enumerate() {
        params.push(param_json(
            &format!("Outer radius of the {name} region (m)"),
            &num(b.radii[k])?,
            &prov.parameter_source,
            &prov.parameter_class,
        ));
    }
    params.push(param_json(
        "Initial kernel concentration (mol/m^3)",
        &num(b.c_init)?,
        &prov.parameter_source,
        &prov.parameter_class,
    ));
    params.push(param_json(
        "Decay constant (1/s)",
        &num(input.decay_constant_per_s)?,
        &prov.parameter_source,
        &prov.parameter_class,
    ));
    if let Some(o) = &labels.fission_rate_origin {
        params.push(param_json(
            "Fission-rate density, an explicit input with no default (fissions/m^3/s)",
            &json_str("see physics.source"),
            o,
            "user-supplied; not an experimentally validated irradiation condition",
        ));
    }
    params.push(param_json(
        "Time step (s)",
        &num(b.dt)?,
        "numerical setting chosen by the user",
        "numerical setting",
    ));
    let _ = writeln!(s, "\"parameters\":[{}],", params.join(","));
    let _ = writeln!(
        s,
        "\"column_dictionary\":{{\"series\":{}}},",
        dictionary_json(&PHYSICS_SERIES_COLUMNS)
    );
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
    let _ = writeln!(
        s,
        "\"run\":{{\"dt_s\":{},\"steps\":{},\"theta\":{},\"initial_inventory_mol\":{},\"matrix_min_row_margin\":{},\"matrix_max_interior_margin_deviation\":{},\"matrix_norm_inf\":{},\"min_pivot_margin\":{},\"max_multiplier\":{}}},",
        num(m.dt)?,
        m.steps,
        num(m.theta)?,
        num(m.initial_inventory)?,
        num(m.matrix_min_row_margin)?,
        num(m.matrix_max_interior_margin_deviation)?,
        num(m.matrix_norm_inf)?,
        num(m.min_pivot_margin)?,
        num(m.max_multiplier)?
    );

    // Diagnostics: worst residual against its bound, non-negativity, monotone release.
    let mut worst_rel = 0.0f64;
    let mut worst_ratio = 0.0f64;
    let mut cmin = f64::INFINITY;
    let mut monotone = true;
    let mut fgen_max = 0.0f64;
    for (k, (_, d, p)) in run.series.iter().enumerate() {
        worst_rel = worst_rel.max(p.balance_residual_relative.abs());
        if d.balance_residual_bound > 0.0 {
            worst_ratio = worst_ratio.max(d.balance_residual.abs() / d.balance_residual_bound);
        }
        cmin = cmin.min(d.min_concentration);
        fgen_max = fgen_max.max(p.release_fraction_of_total);
        if k > 0 && d.cumulative_release < run.series[k - 1].1.cumulative_release {
            monotone = false;
        }
    }
    let last = &run.series[run.series.len() - 1];
    let _ = writeln!(
        s,
        "\"diagnostics\":{{\"worst_balance_residual_relative_to_total\":{},\"worst_residual_over_bound\":{},\"final_balance_residual_bound_relative_to_total\":{},\"release_monotone\":{},\"max_release_fraction_of_total\":{},\"min_concentration_mol_m3\":{}}},",
        num(worst_rel)?,
        num(worst_ratio)?,
        num(last.1.balance_residual_bound / (m.initial_inventory + last.2.generated))?,
        monotone,
        num(fgen_max)?,
        num(cmin)?
    );

    // Series.
    let col = |f: &dyn Fn(
        &crate::numerical::diagnostics::StepDiagnostics,
        &crate::numerical::physics_run::PhysicsRow,
    ) -> f64|
     -> Result<String> {
        json_arr(
            &run.series
                .iter()
                .map(|(_, d, p)| f(d, p))
                .collect::<Vec<_>>(),
        )
    };
    let opt_col = |f: &dyn Fn(
        &crate::numerical::diagnostics::StepDiagnostics,
        &crate::numerical::physics_run::PhysicsRow,
    ) -> Option<f64>|
     -> Result<String> {
        let mut out = String::from("[");
        for (k, (_, d, p)) in run.series.iter().enumerate() {
            if k > 0 {
                out.push(',');
            }
            match f(d, p) {
                Some(v) => out.push_str(&num(v)?),
                None => out.push_str("null"),
            }
        }
        out.push(']');
        Ok(out)
    };
    let times: Vec<f64> = run.series.iter().map(|(t, _, _)| *t).collect();
    let _ = writeln!(
        s,
        "\"series\":{{\"t_s\":{},\"inventory_mol\":{},\"cumulative_release_mol\":{},\"generated_mol\":{},\"decayed_mol\":{},\"release_fraction_of_initial_inventory\":{},\"release_fraction_of_total\":{},\"release_to_generation_ratio\":{},\"release_rate_mol_s\":{},\"balance_residual_mol\":{},\"balance_residual_relative_to_total\":{},\"balance_residual_bound_mol\":{},\"min_concentration_mol_m3\":{},\"max_concentration_mol_m3\":{},\"temperature_k\":{}}},",
        json_arr(&times)?,
        col(&|d, _| d.inventory)?,
        col(&|d, _| d.cumulative_release)?,
        col(&|_, p| p.generated)?,
        col(&|_, p| p.decayed)?,
        col(&|d, _| d.release_fraction)?,
        col(&|_, p| p.release_fraction_of_total)?,
        opt_col(&|_, p| p.release_to_generation_ratio)?,
        opt_col(&|d, _| d.release_rate)?,
        col(&|d, _| d.balance_residual)?,
        col(&|_, p| p.balance_residual_relative)?,
        col(&|d, _| d.balance_residual_bound)?,
        col(&|d, _| d.min_concentration)?,
        col(&|d, _| d.max_concentration)?,
        opt_col(&|_, p| p.temperature_k)?
    );
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
