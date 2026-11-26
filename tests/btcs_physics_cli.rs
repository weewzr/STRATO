//! The `strato-btcs` command line and the export for runs with generation, decay and
//! temperature-dependent diffusion (the `[physics]` section of the input).
//!
//! All inputs here are synthetic (the benchmark of `inputs/synthetic`) with synthetic source
//! strengths, half-lives and activation energies written in the test; none is literature data.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use strato::{
    config::Config,
    numerical::{
        btcs::{validate_input, Limits},
        physics_config::load_physics,
        physics_run::run_physics,
    },
};

const INPUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/inputs/synthetic/synthetic_five_layer.toml"
);

const LIMITS: Limits = Limits {
    max_intervals_total: 1_000_000,
    max_steps: 10_000_000,
};

fn scratch(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("strato-physics-test-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// The synthetic baseline input with a smaller grid and a `[physics]` text appended.
fn input_text(physics: &str) -> String {
    let base = fs::read_to_string(INPUT).unwrap();
    let small = base
        .replace("cells = 40", "cells = 8")
        .replace("output_every_steps = 10", "output_every_steps = 25")
        .replace("dt_s = 2.0e2", "dt_s = 1.0e3");
    format!("{small}\n{physics}\n")
}

fn laws_text() -> String {
    let d1000 = [1.0e-12f64, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13];
    let theta = [1.0e4f64, 2.0e4, 1.5e4, 3.0e4, 1.5e4];
    let names = ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"];
    let r = 8.314_462_618_153_24;
    let mut s = String::new();
    for k in 0..5 {
        let d0 = d1000[k] * (theta[k] / 1000.0).exp();
        let ea_kj = theta[k] * r / 1000.0;
        s.push_str(&format!(
            "[[physics.arrhenius]]\nregion = \"{}\"\nsource = \"synthetic row {}\"\nclass = \"synthetic, not literature data\"\n[[physics.arrhenius.terms]]\nd0_m2_s = {d0:e}\nactivation_energy_kj_per_mol = {ea_kj:e}\n",
            names[k],
            k + 1
        ));
    }
    s
}

fn full_physics(product: &str, temperature: bool) -> String {
    let mut s = format!(
        "[physics.decay]\nhalf_life_s = 1.0e5\n\n[physics.source]\nmode = \"kernel_fission_yield\"\nproduct = \"{product}\"\nyield_kind = \"cumulative\"\nyield_atoms_per_fission = 0.06\nfission_rate_origin = \"user-supplied\"\n[physics.source.fission_rate_density]\nvalue = 3.0e19\n\n"
    );
    if temperature {
        s.push_str("[physics.temperature]\nunit = \"K\"\ninterpolation = \"linear\"\ntimes_s = [0.0, 2.0e5]\nvalues = [1000.0, 1100.0]\n\n");
        s.push_str(&laws_text());
    }
    s
}

struct Exported {
    ts: String,
    profiles: String,
    json: String,
    rows: Vec<Vec<f64>>,
    stdout: String,
}

fn run_cli(file: &Path, out: &Path) -> Exported {
    let res = Command::new(env!("CARGO_BIN_EXE_strato-btcs"))
        .arg(file)
        .arg("--out")
        .arg(out)
        .args(["--commit", "abc1234", "--snapshots", "0,1e5,2e5"])
        .output()
        .unwrap();
    assert!(
        res.status.success(),
        "{}",
        String::from_utf8_lossy(&res.stderr)
    );
    let ts = fs::read_to_string(out.join("btcs_timeseries.csv")).unwrap();
    let rows = ts
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            l.split(',')
                .map(|c| {
                    if c.is_empty() {
                        f64::NAN
                    } else {
                        c.parse().unwrap()
                    }
                })
                .collect()
        })
        .collect();
    Exported {
        profiles: fs::read_to_string(out.join("btcs_profiles.csv")).unwrap(),
        json: fs::read_to_string(out.join("btcs_run.json")).unwrap(),
        stdout: String::from_utf8_lossy(&res.stdout).to_string(),
        ts,
        rows,
    }
}

fn check_export(e: &Exported, dir: &Path, grid_n: usize) {
    assert!(e.stdout.contains("matrix_assemblies=") && e.stdout.contains("final_generated_mol="));
    assert!(e.ts.contains("# commit: abc1234"));
    assert!(e
        .ts
        .contains("# label: numerical solution of the stated model; not experimental validation"));
    assert!(e.ts.contains("combined-scheme rounding bound (F46)"));
    let header = e.ts.lines().find(|l| !l.starts_with('#')).unwrap();
    for name in [
        "Time (s)",
        "Cumulative generated amount excluding the initial inventory (mol)",
        "Cumulative amount decayed inside the particle and not part of the release (mol)",
        "Release fraction of the initial inventory plus the generated amount (dimensionless)",
        "Particle temperature (K)",
        "Derived a-posteriori rounding bound of the mass-balance residual (mol)",
    ] {
        assert!(
            header.split(',').any(|c| c == name),
            "missing column {name}"
        );
    }
    assert_eq!(e.rows[0].len(), 15);
    // Residual (column 9) within its bound (column 11) in every row of the file.
    for r in &e.rows {
        assert!(r[9].abs() <= r[11], "residual {} bound {}", r[9], r[11]);
        assert!(r[12] >= 0.0, "minimum concentration");
    }
    assert!(e.json.contains("\"commit\":\"abc1234\""));
    assert!(e.json.contains("\"physics\":{"));
    assert!(e.json.contains("user-supplied"));
    assert!(e
        .json
        .contains("not an experimentally validated irradiation condition"));
    assert!(e.json.contains("\"kind\":\"combined\""));
    assert!(e.json.contains("\"snapshots\":["));
    let n_rows = e.profiles.lines().filter(|l| !l.starts_with('#')).count() - 1;
    assert_eq!(n_rows, 3 * (grid_n + 1));
    // No absolute path of the machine in any export.
    let dir_text = dir.to_string_lossy().to_string();
    for text in [&e.ts, &e.profiles, &e.json] {
        assert!(!text.contains(&dir_text));
    }
}

#[test]
fn cli_source_and_decay_run_agrees_with_the_library_bit_for_bit() {
    // No exponential is evaluated on this path (constant diffusivity), so every operation is
    // an IEEE operation and the command line and the library agree bit for bit.
    let dir = scratch("full");
    let file = dir.join("in.toml");
    fs::write(
        &file,
        input_text(&full_physics("direct_fission_product", false)),
    )
    .unwrap();
    let out = dir.join("out");
    let e = run_cli(&file, &out);
    let cfg = Config::load(&file).unwrap();
    let input = validate_input(&cfg, &LIMITS).unwrap();
    let raw = fs::read_to_string(&file).unwrap();
    let setup = load_physics(&raw, input, cfg.time.end_time_s)
        .unwrap()
        .unwrap();
    let run = run_physics(&setup.input, &[]).unwrap();
    check_export(&e, &dir, run.grid.n);
    assert_eq!(run.meta.assemblies, 1);
    assert_eq!(e.rows.len(), run.series.len());
    for (k, (t, d, p)) in run.series.iter().enumerate() {
        let r = &e.rows[k];
        assert_eq!(r[0].to_bits(), t.to_bits());
        assert_eq!(r[1].to_bits(), d.inventory.to_bits());
        assert_eq!(r[2].to_bits(), d.cumulative_release.to_bits());
        assert_eq!(r[3].to_bits(), p.generated.to_bits());
        assert_eq!(r[4].to_bits(), p.decayed.to_bits());
        assert_eq!(r[5].to_bits(), d.release_fraction.to_bits());
        assert_eq!(r[9].to_bits(), d.balance_residual.to_bits());
        assert_eq!(r[11].to_bits(), d.balance_residual_bound.to_bits());
    }
    let last = run.series.last().unwrap();
    assert!(last.2.generated > 0.0 && last.2.decayed > 0.0);
    // Constant diffusivity: the temperature column is empty.
    assert!(e.rows.iter().all(|r| r[14].is_nan()));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cli_temperature_history_run_exports_the_temperature_and_reassembles() {
    // The diffusivities come from exp and ln of the platform library, which can differ in the
    // last bit between two executables (observed here between a test thread and a command-line
    // process: one unit in the last place of one D), and the rounding noise of the solve then
    // differs at the 1e-12 level. Document 5, Section 3.5 does not claim bit identity across
    // platforms for this path, and none is asserted: the checks are structural, and the
    // residual bound holds in the file itself.
    let dir = scratch("temp");
    let file = dir.join("in.toml");
    fs::write(
        &file,
        input_text(&full_physics("direct_fission_product", true)),
    )
    .unwrap();
    let out = dir.join("out");
    let e = run_cli(&file, &out);
    let cfg = Config::load(&file).unwrap();
    let input = validate_input(&cfg, &LIMITS).unwrap();
    let n: usize = input.n_intervals.iter().sum();
    check_export(&e, &dir, n);
    let assemblies: usize = e
        .stdout
        .lines()
        .find_map(|l| l.strip_prefix("matrix_assemblies="))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(
        assemblies, input.steps,
        "the temperature changes the matrix at every step"
    );
    // Temperature column: 1000 K at t = 0 up to 1100 K at the end, increasing.
    assert_eq!(e.rows[0][14], 1000.0);
    assert_eq!(e.rows.last().unwrap()[14], 1100.0);
    assert!(e.rows.windows(2).all(|w| w[1][14] > w[0][14]));
    assert!(e.json.contains("synthetic row 3"));
    assert!(e.json.contains("\"model\":\"arrhenius\""));
    // Generation and decay are positive and separate from the release.
    let last = e.rows.last().unwrap();
    assert!(last[3] > 0.0 && last[4] > 0.0 && last[2] > 0.0);
    let _ = fs::remove_dir_all(&dir);
}
#[test]
fn cli_refuses_an_activation_product_and_a_missing_fission_rate() {
    let bin = env!("CARGO_BIN_EXE_strato-btcs");
    let dir = scratch("refuse");
    // Decision 15: a direct fission-yield source for an activation product is refused.
    let file = dir.join("activation.toml");
    fs::write(&file, input_text(&full_physics("activation_product", true))).unwrap();
    let r = Command::new(bin)
        .arg(&file)
        .arg("--out")
        .arg(dir.join("o1"))
        .output()
        .unwrap();
    assert!(!r.status.success());
    assert!(
        String::from_utf8_lossy(&r.stderr).contains("activation product"),
        "{}",
        String::from_utf8_lossy(&r.stderr)
    );
    assert!(!dir.join("o1").join("btcs_timeseries.csv").exists());
    // Decisions 14 and 42: no default for the fission-rate density.
    let no_rate = "[physics.source]\nmode = \"kernel_fission_yield\"\nproduct = \"direct_fission_product\"\nyield_kind = \"cumulative\"\nyield_atoms_per_fission = 0.06\n";
    let file = dir.join("norate.toml");
    fs::write(&file, input_text(no_rate)).unwrap();
    let r = Command::new(bin)
        .arg(&file)
        .arg("--out")
        .arg(dir.join("o2"))
        .output()
        .unwrap();
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("no default"));
    // A temperature history without the five laws, and laws without a history.
    let file = dir.join("nolaws.toml");
    fs::write(
        &file,
        input_text("[physics.temperature]\nunit = \"K\"\nvalue = 1000.0\n"),
    )
    .unwrap();
    let r = Command::new(bin)
        .arg(&file)
        .arg("--out")
        .arg(dir.join("o3"))
        .output()
        .unwrap();
    assert!(!r.status.success());
    let file = dir.join("notemp.toml");
    fs::write(&file, input_text(&laws_text())).unwrap();
    let r = Command::new(bin)
        .arg(&file)
        .arg("--out")
        .arg(dir.join("o4"))
        .output()
        .unwrap();
    assert!(!r.status.success());
    // An unknown key in the section is an error, not ignored.
    let file = dir.join("typo.toml");
    fs::write(&file, input_text("[physics.decay]\nhalf_life = 5.0\n")).unwrap();
    let r = Command::new(bin)
        .arg(&file)
        .arg("--out")
        .arg(dir.join("o5"))
        .output()
        .unwrap();
    assert!(!r.status.success());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn two_term_law_is_data_not_code_and_is_reported_as_a_sum() {
    // The published sum of two terms is supplied as two terms of one law; the code chooses
    // nothing (decisions 13 and 39). The export names the form.
    let dir = scratch("sum");
    let mut laws = String::new();
    for (k, name) in ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"]
        .iter()
        .enumerate()
    {
        laws.push_str(&format!(
            "[[physics.arrhenius]]\nregion = \"{name}\"\n[[physics.arrhenius.terms]]\nd0_m2_s = 1.0e-6\nactivation_energy_kj_per_mol = {}\n",
            100.0 + 10.0 * k as f64
        ));
        if k == 1 {
            laws.push_str(
                "[[physics.arrhenius.terms]]\nd0_m2_s = 1.0e-9\nactivation_energy_kj_per_mol = 50.0\nvalid_from_k = 900.0\nvalid_to_k = 1500.0\n",
            );
        }
    }
    let text = format!(
        "[physics.temperature]\nunit = \"degC\"\ninterpolation = \"step\"\ntimes_s = [0.0, 1.0e5]\nvalues = [800.0, 900.0]\n\n{laws}"
    );
    let file = dir.join("sum.toml");
    fs::write(&file, input_text(&text)).unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_strato-btcs"))
        .arg(&file)
        .arg("--out")
        .arg(dir.join("out"))
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let json = fs::read_to_string(dir.join("out").join("btcs_run.json")).unwrap();
    assert!(json.contains("\"form\":\"sum of terms\""));
    assert!(json.contains("\"form\":\"single term\""));
    // 800 degC = 1073.15 K and 900 degC = 1173.15 K lie inside the stated range of the sum.
    assert!(json.contains("\"steps_below_range\":0"));
    assert!(json.contains("\"steps_above_range\":0"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn baseline_input_without_physics_is_unchanged() {
    let dir = scratch("base");
    let out = dir.join("out");
    let r = Command::new(env!("CARGO_BIN_EXE_strato-btcs"))
        .arg(INPUT)
        .arg("--out")
        .arg(&out)
        .output()
        .unwrap();
    assert!(r.status.success());
    let ts = fs::read_to_string(out.join("btcs_timeseries.csv")).unwrap();
    assert!(ts.contains("Release fraction (dimensionless)"));
    assert!(!ts.contains("Cumulative generated amount"));
    let _ = fs::remove_dir_all(&dir);
}
