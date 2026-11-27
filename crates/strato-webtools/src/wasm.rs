//! `build-wasm`: build the STRATO WebAssembly wrapper for the web demo and check it
//! (prompt/TECHNICAL_ARCHITECTURE.md Section 3, "Build").
//!
//! 1. The installed wasm-bindgen CLI must have exactly the wasm-bindgen version recorded in
//!    crates/strato-wasm/Cargo.lock.
//! 2. `cargo build --release --locked --target wasm32-unknown-unknown` for the wrapper crate,
//!    with the build commit stamped in (`STRATO_BUILD_COMMIT`, the first seven characters of
//!    `GITHUB_SHA` or of `git rev-parse HEAD`, as the web build stamp in web/vite.config.ts).
//! 3. `wasm-bindgen --target web` into web/src/solver/pkg.
//! 4. Export and size check of the module: the exported functions of the architecture are
//!    present, nothing else is exported apart from memory and the wasm-bindgen runtime (names
//!    starting with two underscores), and the module is under 1 MB after gzip (Section 6 budget).
//!
//! `build-wasm --check` runs step 4 on the existing build only.

use flate2::{write::GzEncoder, Compression};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::bail;
use crate::util::{self, Error, Result};

const CRATE: &str = "crates/strato-wasm";
const OUT: &str = "web/src/solver/pkg";
const WASM_NAME: &str = "strato_wasm";
const TARGET: &str = "wasm32-unknown-unknown";

/// The functions of the wrapper (BTCS interface, ABI 4) and the module's own status functions:
/// the six of ABI 3, which the web worker calls today, and the five of ABI 4 (equation stages,
/// scenarios, the assembled system, the analytical method and the options of the selectors).
pub const WEB_EXPORTS: [&str; 11] = [
    "abi_version",
    "build_info",
    "validate",
    "run",
    "compare",
    "export_run",
    "run_stage",
    "run_scenario",
    "btcs_system",
    "analytical",
    "options",
];
/// Every export the module may have: the web contract. The finite-volume functions of ABI 1 and 2
/// (`validate_config`, `run_v1`, `analytical_release`, `compare_v1`) are retired; one of them in
/// the module fails the check "no unexpected exports".
pub const REQUIRED_EXPORTS: [&str; 11] = WEB_EXPORTS;
/// The names of the retired finite-volume exports, which the tests keep out of the contract.
#[cfg(test)]
const RETIRED_EXPORTS: [&str; 4] = [
    "validate_config",
    "run_v1",
    "analytical_release",
    "compare_v1",
];
/// memory, and the wrapper's start function (`#[wasm_bindgen(start)]`), which wasm-bindgen also
/// exports and calls once through `__wbindgen_start` when the module is instantiated.
const ALLOWED_EXTRA: [&str; 2] = ["memory", "install_panic_hook"];
/// Every name the wasm-bindgen runtime adds begins with two underscores; a name without them is
/// the crate's own and must be one of the required exports.
const ALLOWED_PREFIX: &str = "__";
const GZIP_LIMIT_BYTES: usize = 1_000_000;

fn lock_version(root: &Path) -> Result<String> {
    let text = util::read_text(&root.join(CRATE).join("Cargo.lock"))?;
    let marker = "name = \"wasm-bindgen\"\nversion = \"";
    match text.find(marker) {
        Some(k) => {
            let rest = &text[k + marker.len()..];
            Ok(rest.split('"').next().unwrap_or("").to_string())
        }
        None => bail!("FAIL wasm-bindgen is not in crates/strato-wasm/Cargo.lock"),
    }
}

fn cli_version() -> Result<String> {
    let out = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .map_err(|_| Error("FAIL the wasm-bindgen CLI is not on PATH".into()))?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(text
        .strip_prefix("wasm-bindgen")
        .and_then(|r| r.split_whitespace().next())
        .unwrap_or(&text)
        .to_string())
}

fn stamp(root: &Path) -> (String, bool) {
    let ci = std::env::var("GITHUB_SHA").unwrap_or_default();
    let full = if ci.is_empty() {
        util::git(root, &["rev-parse", "HEAD"]).unwrap_or_default()
    } else {
        ci.clone()
    };
    let commit: String = full.chars().take(7).collect();
    let commit = if commit.is_empty() {
        "unknown".to_string()
    } else {
        commit
    };
    let dirty = if ci.is_empty() {
        !util::git(root, &["status", "--porcelain"])
            .unwrap_or_default()
            .is_empty()
    } else {
        false
    };
    (commit, dirty)
}

fn uleb(data: &[u8], mut pos: usize) -> Result<(usize, usize)> {
    let (mut result, mut shift) = (0usize, 0u32);
    loop {
        let Some(&b) = data.get(pos) else {
            bail!("FAIL truncated WebAssembly module");
        };
        pos += 1;
        result |= usize::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            return Ok((result, pos));
        }
        shift += 7;
    }
}

/// The names in the export section of a WebAssembly module.
pub fn exports(wasm: &[u8]) -> Result<Vec<String>> {
    if wasm.len() < 8 || &wasm[..4] != b"\0asm" {
        bail!("FAIL not a WebAssembly module");
    }
    let mut pos = 8;
    let mut names = Vec::new();
    while pos < wasm.len() {
        let id = wasm[pos];
        let (size, p) = uleb(wasm, pos + 1)?;
        let end = p + size;
        if id == 7 {
            let (count, mut q) = uleb(wasm, p)?;
            for _ in 0..count {
                let (n, r) = uleb(wasm, q)?;
                let name = wasm
                    .get(r..r + n)
                    .ok_or_else(|| Error("FAIL truncated export name".into()))?;
                names.push(String::from_utf8_lossy(name).into_owned());
                // name, then the kind byte, then the index
                let (_, r2) = uleb(wasm, r + n + 1)?;
                q = r2;
            }
        }
        pos = end;
    }
    Ok(names)
}

fn gzip_len(data: &[u8]) -> Result<usize> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::best());
    enc.write_all(data)?;
    Ok(enc.finish()?.len())
}

/// The checks of step 4; prints one line per check and returns 1 on any failure.
pub fn check(root: &Path) -> Result<i32> {
    let out = root.join(OUT);
    let wasm_path = out.join(format!("{WASM_NAME}_bg.wasm"));
    let js_path = out.join(format!("{WASM_NAME}.js"));
    if !wasm_path.exists() || !js_path.exists() {
        println!(
            "FAIL missing {} or {WASM_NAME}.js; run strato-webtools build-wasm",
            util::rel_posix(root, &wasm_path)
        );
        return Ok(1);
    }
    let data = util::read_bytes(&wasm_path)?;
    let names = exports(&data)?;
    let mut bad = 0;
    let mut required: Vec<&str> = REQUIRED_EXPORTS.to_vec();
    required.sort();
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|r| !names.iter().any(|n| n == r))
        .collect();
    println!(
        "{} required exports present ({}){}",
        if missing.is_empty() { "PASS" } else { "FAIL" },
        required.join(", "),
        if missing.is_empty() {
            String::new()
        } else {
            format!("; missing: {}", missing.join(", "))
        }
    );
    bad += i32::from(!missing.is_empty());
    let mut extra: Vec<&String> = names
        .iter()
        .filter(|n| {
            !REQUIRED_EXPORTS.contains(&n.as_str())
                && !ALLOWED_EXTRA.contains(&n.as_str())
                && !n.starts_with(ALLOWED_PREFIX)
        })
        .collect();
    extra.sort();
    println!(
        "{} no unexpected exports{}",
        if extra.is_empty() { "PASS" } else { "FAIL" },
        if extra.is_empty() {
            format!(" ({} exports)", names.len())
        } else {
            format!(
                ": {}",
                extra
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    );
    bad += i32::from(!extra.is_empty());
    let gz = gzip_len(&data)?;
    let ok = gz < GZIP_LIMIT_BYTES;
    println!(
        "{} size: {} bytes, {gz} bytes gzip (limit {GZIP_LIMIT_BYTES})",
        if ok { "PASS" } else { "FAIL" },
        data.len()
    );
    bad += i32::from(!ok);
    println!("RESULT {}", if bad == 0 { "PASS" } else { "FAIL" });
    Ok(i32::from(bad > 0))
}

fn build(root: &Path) -> Result<i32> {
    let (want, have) = (lock_version(root)?, cli_version()?);
    if want != have {
        println!("FAIL wasm-bindgen CLI {have} differs from the crate's Cargo.lock version {want}");
        return Ok(1);
    }
    println!("PASS wasm-bindgen CLI {have} equals Cargo.lock");
    let (commit, dirty) = stamp(root);
    let manifest = root.join(CRATE).join("Cargo.toml");
    println!(
        "RUN  cargo build --release --locked --lib --target {TARGET} --manifest-path {CRATE}/Cargo.toml  (STRATO_BUILD_COMMIT={commit})"
    );
    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--locked",
            "--lib",
            "--target",
            TARGET,
            "--manifest-path",
        ])
        .arg(&manifest)
        .env("STRATO_BUILD_COMMIT", &commit)
        .env("STRATO_BUILD_DIRTY", if dirty { "1" } else { "0" })
        .current_dir(root)
        .status()
        .map_err(|e| Error(format!("cargo: {e}")))?;
    if !status.success() {
        bail!("cargo build failed ({status})");
    }
    let wasm = root
        .join(CRATE)
        .join("target")
        .join(TARGET)
        .join("release")
        .join(format!("{WASM_NAME}.wasm"));
    let out = root.join(OUT);
    if out.exists() {
        std::fs::remove_dir_all(&out)?;
    }
    println!(
        "RUN  wasm-bindgen --target web --out-dir {OUT} {}",
        util::rel_posix(root, &wasm)
    );
    let status = Command::new("wasm-bindgen")
        .args(["--target", "web", "--out-dir"])
        .arg(&out)
        .arg(&wasm)
        .current_dir(root)
        .status()
        .map_err(|e| Error(format!("wasm-bindgen: {e}")))?;
    if !status.success() {
        bail!("wasm-bindgen failed ({status})");
    }
    util::write_text(&out.join(".gitignore"), "*\n")?;
    check(root)
}

pub fn run(root: &Path, check_only: bool) -> Result<i32> {
    if check_only {
        check(root)
    } else {
        build(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_allowed_exports_are_the_web_functions_only() {
        assert_eq!(REQUIRED_EXPORTS.to_vec(), WEB_EXPORTS.to_vec());
        for n in RETIRED_EXPORTS {
            assert!(!REQUIRED_EXPORTS.contains(&n), "{n}");
        }
        // The web contract is the BTCS interface and the data export, by method-neutral names.
        for n in WEB_EXPORTS {
            assert!(!n.ends_with("_v1") && !n.contains("config"), "{n}");
        }
    }

    #[test]
    fn export_section_is_read() {
        // Header, then an export section with two exports: "memory" (memory 0) and "run" (func 3).
        let mut m = b"\0asm\x01\0\0\0".to_vec();
        let body: Vec<u8> = [&[2u8][..], &[6], b"memory", &[2, 0], &[3], b"run", &[0, 3]].concat();
        m.push(7);
        m.push(body.len() as u8);
        m.extend(body);
        assert_eq!(exports(&m).unwrap(), ["memory", "run"]);
    }

    #[test]
    fn not_a_module_is_refused() {
        assert!(exports(b"nope").is_err());
    }
}
