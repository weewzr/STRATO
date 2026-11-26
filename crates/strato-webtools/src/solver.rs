//! The one place that knows how the numerical solver is called.
//!
//! The web tools never link the solver. They run its command line and its native wrapper example
//! and convert what those print and write. The solver is the spherical BTCS finite-difference
//! scheme (`strato-btcs`); the command lines, the names of the files the command line writes,
//! the file that holds the independent check and the names of the converted files are here, and
//! nowhere else. A later change of solver changes this file and the conversion in
//! `precomputed.rs`.

/// The input case that the precomputed results are computed from.
pub const INPUT: &str = "inputs/synthetic/synthetic_five_layer.toml";

/// The folder (under the repository root, ignored by Git) where the command line writes its files.
pub const CLI_OUT_DIR: &str = "results/web-btcs";

/// Name of the solver that the headers of the converted files carry.
pub const SOLVER_ID: &str = "STRATO BTCS five-layer spherical finite difference";

/// Source row and class that the exported files carry for the parameters (decision 31). The
/// input is the synthetic benchmark of the repository: it is not literature data.
pub const PARAMETER_SOURCE: &str = "synthetic benchmark, not literature data";
pub const PARAMETER_CLASS: &str = "synthetic benchmark";

/// Fractions of the end time at which the command line writes concentration profiles. A choice
/// of the display (which instants the downloadable profiles show), not a parameter of the model;
/// the solver serves each from the nearest step and never interpolates.
pub const SNAPSHOT_FRACTIONS: [f64; 5] = [0.0, 0.1, 0.25, 0.5, 1.0];

/// Command line of the native release binary of the root crate. `snapshots` is the
/// comma-separated list of snapshot times in seconds, `commit` the commit written into the headers.
pub fn cli_command(snapshots: &str, commit: &str) -> Vec<String> {
    let mut v: Vec<String> = [
        "cargo",
        "run",
        "--locked",
        "--release",
        "--quiet",
        "--bin",
        "strato-btcs",
        "--",
        INPUT,
        "--out",
        CLI_OUT_DIR,
        "--snapshots",
    ]
    .map(String::from)
    .to_vec();
    v.push(snapshots.to_string());
    v.extend(["--commit".to_string(), commit.to_string()]);
    v.extend([
        "--parameter-source".to_string(),
        PARAMETER_SOURCE.to_string(),
    ]);
    v.extend(["--parameter-class".to_string(), PARAMETER_CLASS.to_string()]);
    v
}

/// The same command as the person would type it, recorded in the provenance headers.
pub const CLI_TEXT: &str = "cargo run --locked --release --bin strato-btcs -- inputs/synthetic/synthetic_five_layer.toml --out results/web-btcs --snapshots <fractions of the end time> --commit <commit> --parameter-source <source> --parameter-class <class>";

/// Command line of the native run of the WebAssembly wrapper, which prints the `run` reply as
/// JSON on standard output.
pub fn native_command() -> Vec<String> {
    [
        "cargo",
        "run",
        "--locked",
        "--release",
        "--quiet",
        "--manifest-path",
        "crates/strato-wasm/Cargo.toml",
        "--example",
        "native_btcs_run",
        "--",
        INPUT,
    ]
    .map(String::from)
    .to_vec()
}

/// Command line of the battery of the parity test (tcs_parity example of the wrapper): `requests`
/// prints the requests of the named cases, `native` the native replies. The cases are the ones small
/// enough to keep in a file; the others (large grids) are compared by the Rust example against the
/// WebAssembly module run in Node.
pub fn battery_command(what: &str) -> Vec<String> {
    let mut v: Vec<String> = [
        "cargo",
        "run",
        "--locked",
        "--release",
        "--quiet",
        "--manifest-path",
        "crates/strato-wasm/Cargo.toml",
        "--example",
        "btcs_parity",
        "--",
    ]
    .map(String::from)
    .to_vec();
    v.push(what.to_string());
    v.extend(BATTERY_CASES.map(String::from));
    v
}

pub const BATTERY_CASES: [&str; 9] = [
    "validate_toml",
    "validate_rejected_dt",
    "run_single_step",
    "run_profile_times",
    "run_rejected_steps",
    "compare_coarse_sic_diffusivity",
    "export_timeseries_csv",
    "export_comparison_csv",
    "build_info",
];

pub const BATTERY_TEXT: &str = "cargo run --locked --release --manifest-path crates/strato-wasm/Cargo.toml --example btcs_parity -- native <cases>";

pub const NATIVE_TEXT: &str = "cargo run --locked --release --manifest-path crates/strato-wasm/Cargo.toml --example native_btcs_run -- inputs/synthetic/synthetic_five_layer.toml";

/// The three files the command line writes into [`CLI_OUT_DIR`].
pub const CLI_TIMESERIES: &str = "btcs_timeseries.csv";
pub const CLI_PROFILES: &str = "btcs_profiles.csv";
pub const CLI_RUN_JSON: &str = "btcs_run.json";

/// The independent check of the regression suite for this input (run by the library's
/// regression example, which writes it under tests/golden/btcs): the release fraction of the
/// BTCS run against a reference that contains no BTCS code, with the observed orders in time and
/// in space. The tool copies it; it computes nothing.
pub const CHECK_FILE: &str = "tests/golden/btcs/s5_n40_dt200/independent_check.csv";

/// Machine keys of the columns of the independent-check file, in file order
/// (`postprocessing::export::CHECK_COLUMNS`). The conversion checks the number of columns.
pub const CHECK_KEYS: [&str; 17] = [
    "t_s",
    "f_btcs",
    "f_reference",
    "abs_difference",
    "time_error",
    "time_order",
    "time_uncertainty",
    "space_error",
    "space_order",
    "space_uncertainty",
    "cross_error",
    "f_limit",
    "limit_difference",
    "reference_floor",
    "tolerance",
    "limit_difference_over_tolerance",
    "passed",
];

/// Lines of the command line's standard output that name files (their separators depend on the
/// platform) and are left out of the summary.
pub const PATH_KEYS: [&str; 3] = ["timeseries", "profiles", "run_json"];

/// Names written to web/public/data/, in the order of the files written and checked.
pub const PRECOMPUTED_FILES: [&str; 9] = [
    "btcs_run.json",
    "btcs_vs_analytical.json",
    "btcs_independent_check.json",
    "wrapper_native_battery.json",
    "wrapper_native_synthetic.json",
    "btcs_run.timeseries.csv",
    "btcs_run.profiles.csv",
    "btcs_vs_analytical.csv",
    "btcs_independent_check.csv",
];

/// Names of files of an earlier solver that no longer exist; `precomputed --check` reports one
/// that is still in the folder.
pub const RETIRED_FILES: [&str; 10] = [
    "synthetic_run.json",
    "comparison_fv_vs_analytical.json",
    "convergence_study.json",
    "golden_v1.json",
    "synthetic_run.timeseries.csv",
    "synthetic_run.final_profile.csv",
    "comparison_fv_vs_analytical.csv",
    "convergence_study.csv",
    "convergence_guides.json",
    "convergence_guides.csv",
];

pub const NOTE_RUN: &str = "Time series, profiles and diagnostics of the native BTCS run of the synthetic five-layer case, as the solver's command line wrote them (numerical solution of the stated model; not experimental validation).";
pub const NOTE_COMPARISON: &str = "Release fraction of the BTCS run and of the analytical (Sturm-Liouville series) solution at every output time; the absolute difference is computed in Rust from the two columns; empty cells are null where the series is not valid.";
pub const NOTE_CHECK: &str = "Independent check of the BTCS release fraction against a reference that contains no BTCS code, with the observed orders in time and in space, copied from the regression suite; a regression reference, not truth.";
pub const NOTE_BATTERY: &str = "Requests of the cases of the parity battery of the BTCS functions that are small enough to keep in a file (validate, run, compare, export, build info) and the replies of the native wrapper (x86_64), the reference of the browser parity test of the functions beyond the default run.";
pub const NOTE_WRAPPER: &str =
    "run reply of the native wrapper (x86_64), the reference of the browser parity test.";

/// Names of the summary entries of the command line's standard output that the page shows.
pub const SUMMARY_KEYS: [&str; 6] = [
    "nodes_N",
    "steps",
    "final_release_fraction",
    "worst_balance_residual_relative",
    "final_balance_residual_bound_relative",
    "min_concentration_mol_m3",
];
