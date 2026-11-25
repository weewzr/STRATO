//! The `strato-btcs` command-line entry: reads the existing TOML input format, writes the CSV and
//! JSON exports with a provenance header, and agrees with the library run bit for bit.

use std::{fs, path::PathBuf, process::Command};
use strato::{
    config::Config,
    numerical::btcs::{run, Limits},
};

const INPUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/inputs/synthetic/synthetic_five_layer.toml"
);

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("strato-btcs-test-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn cli_writes_exports_that_match_the_library() {
    let out = scratch("ok");
    let status = Command::new(env!("CARGO_BIN_EXE_strato-btcs"))
        .args([INPUT, "--out"])
        .arg(&out)
        .args(["--commit", "abc1234", "--snapshots", "0,1e5,2e5"])
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    let stdout = String::from_utf8_lossy(&status.stdout).to_string();
    assert!(stdout.contains("commit=abc1234"));
    assert!(!stdout.contains("unknown") || stdout.contains("commit=abc1234"));

    let ts = fs::read_to_string(out.join("btcs_timeseries.csv")).unwrap();
    assert!(ts.contains("# commit: abc1234"));
    assert!(ts.contains("# input_file: synthetic_five_layer.toml"));
    assert!(ts.contains("# label: numerical solution of the stated model"));
    let last = ts.lines().last().unwrap();
    let f_cli: f64 = last.split(',').nth(3).unwrap().parse().unwrap();

    let cfg = Config::load(INPUT).unwrap();
    let lib = run(
        &cfg,
        &Limits {
            max_intervals_total: 1_000_000,
            max_steps: 10_000_000,
        },
        &[],
    )
    .unwrap();
    let f_lib = lib.series.last().unwrap().1.release_fraction;
    assert_eq!(f_cli.to_bits(), f_lib.to_bits());

    let profiles = fs::read_to_string(out.join("btcs_profiles.csv")).unwrap();
    let rows = profiles.lines().filter(|l| !l.starts_with('#')).count() - 1;
    assert_eq!(rows, 3 * (lib.grid.n + 1));
    let json = fs::read_to_string(out.join("btcs_run.json")).unwrap();
    assert!(json.contains("\"commit\":\"abc1234\"") && json.contains("\"snapshots\":["));
    // No absolute path of the machine in any export.
    let dir_text = out.to_string_lossy().to_string();
    for text in [&ts, &profiles, &json] {
        assert!(!text.contains(&dir_text) && !text.contains(INPUT));
    }
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn cli_refuses_bad_input() {
    let bin = env!("CARGO_BIN_EXE_strato-btcs");
    let missing = Command::new(bin)
        .arg("does-not-exist.toml")
        .output()
        .unwrap();
    assert!(!missing.status.success());
    let too_big = Command::new(bin)
        .args([INPUT, "--max-nodes", "10", "--out"])
        .arg(scratch("big"))
        .output()
        .unwrap();
    assert!(!too_big.status.success());
    assert!(String::from_utf8_lossy(&too_big.stderr).contains("exceeds"));
    let no_args = Command::new(bin).output().unwrap();
    assert!(!no_args.status.success());
}
