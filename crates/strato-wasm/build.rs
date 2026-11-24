//! Build stamp of the wrapper: the version of the `strato` package it wraps and the commit
//! of the tree it was built from. The commit is passed in by scripts/web/build_wasm.py (and by
//! scripts/web/generate_data.py for the native precomputed run) through STRATO_BUILD_COMMIT;
//! without it the stamp reads "unknown". Nothing here touches the numerics.

use std::{env, fs, path::Path};

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    let strato_toml = Path::new(&manifest).join("../../Cargo.toml");
    println!("cargo:rerun-if-changed={}", strato_toml.display());
    let text = fs::read_to_string(&strato_toml).expect("the strato package manifest is readable");
    let version = text
        .lines()
        .skip_while(|l| l.trim() != "[package]")
        .find_map(|l| {
            let l = l.trim();
            let rest = l.strip_prefix("version")?.trim_start().strip_prefix('=')?;
            Some(rest.trim().trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=STRATO_PACKAGE_VERSION={version}");

    println!("cargo:rerun-if-env-changed=STRATO_BUILD_COMMIT");
    println!("cargo:rerun-if-env-changed=STRATO_BUILD_DIRTY");
    let commit = env::var("STRATO_BUILD_COMMIT")
        .ok()
        .filter(|c| !c.trim().is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let dirty = env::var("STRATO_BUILD_DIRTY")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);
    println!("cargo:rustc-env=STRATO_WRAPPER_COMMIT={}", commit.trim());
    println!(
        "cargo:rustc-env=STRATO_WRAPPER_DIRTY={}",
        if dirty { "1" } else { "0" }
    );
}
