"""Build the STRATO WebAssembly wrapper for the web demo and check it.

Pattern of Outram Park crates/dhoby-ghaut/web/dispersion/build.sh and NUS-H2
scripts/build_wasm.py (prompt/TECHNICAL_ARCHITECTURE.md Section 3, "Build"):

1. The installed wasm-bindgen CLI must have exactly the wasm-bindgen version
   recorded in crates/strato-wasm/Cargo.lock.
2. cargo build --release --locked --target wasm32-unknown-unknown for the
   wrapper crate, with the build commit stamped in (STRATO_BUILD_COMMIT, the
   first seven characters of GITHUB_SHA or of git rev-parse HEAD, as the web
   build stamp in web/vite.config.ts).
3. wasm-bindgen --target web into web/src/solver/pkg.
4. Export and size check of the module: the six exported functions of the
   architecture (abi_version, build_info, validate_config, run_v1,
   analytical_release, compare_v1) are present, nothing else is exported apart
   from memory and the wasm-bindgen runtime (names starting with __wbindgen or
   __wbg), and the module is under 1 MB after gzip (Section 6 budget).

Usage, from the repository root (standard library only):

    python -I -X utf8 scripts/web/build_wasm.py           # build and check
    python -I -X utf8 scripts/web/build_wasm.py --check   # check the existing build only
"""

from __future__ import annotations

import argparse
import gzip
import os
import pathlib
import re
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
CRATE = ROOT / "crates" / "strato-wasm"
OUT = ROOT / "web" / "src" / "solver" / "pkg"
WASM_NAME = "strato_wasm"
TARGET = "wasm32-unknown-unknown"

REQUIRED_EXPORTS = {"abi_version", "build_info", "validate_config", "run_v1", "analytical_release", "compare_v1"}
# memory, and the wrapper's start function (#[wasm_bindgen(start)]), which wasm-bindgen also
# exports and calls once through __wbindgen_start when the module is instantiated.
ALLOWED_EXTRA = {"memory", "install_panic_hook"}
# Runtime exports of wasm-bindgen 0.2.130: __wbindgen_* and __wbg_* helpers, and the abort
# handling of a panic="abort" module (__abort_handler, __instance_terminated). Every name the
# runtime adds begins with two underscores; a name without them is the crate's own and must
# be one of the six above.
ALLOWED_PREFIXES = ("__",)
GZIP_LIMIT_BYTES = 1_000_000


def lock_version() -> str:
    text = (CRATE / "Cargo.lock").read_text(encoding="utf-8")
    m = re.search(r'name = "wasm-bindgen"\nversion = "([^"]+)"', text)
    if not m:
        raise SystemExit("FAIL wasm-bindgen is not in crates/strato-wasm/Cargo.lock")
    return m.group(1)


def cli_version() -> str:
    exe = shutil.which("wasm-bindgen")
    if not exe:
        raise SystemExit("FAIL the wasm-bindgen CLI is not on PATH")
    out = subprocess.run([exe, "--version"], check=True, capture_output=True, text=True).stdout
    m = re.search(r"wasm-bindgen\s+(\S+)", out)
    return m.group(1) if m else out.strip()


def git(*args: str) -> str:
    try:
        return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return ""


def stamp() -> tuple[str, bool]:
    ci = os.environ.get("GITHUB_SHA", "")
    commit = (ci or git("rev-parse", "HEAD"))[:7] or "unknown"
    dirty = False if ci else git("status", "--porcelain") != ""
    return commit, dirty


def _uleb(data: bytes, pos: int) -> tuple[int, int]:
    result = shift = 0
    while True:
        b = data[pos]
        pos += 1
        result |= (b & 0x7F) << shift
        if not b & 0x80:
            return result, pos
        shift += 7


def exports(wasm: bytes) -> list[str]:
    if wasm[:4] != b"\0asm":
        raise SystemExit("FAIL not a WebAssembly module")
    pos, names = 8, []
    while pos < len(wasm):
        sid = wasm[pos]
        size, pos = _uleb(wasm, pos + 1)
        end = pos + size
        if sid == 7:
            count, p = _uleb(wasm, pos)
            for _ in range(count):
                n, p = _uleb(wasm, p)
                names.append(wasm[p:p + n].decode("utf-8"))
                p += n + 1  # name, then the kind byte
                _, p = _uleb(wasm, p)  # index
        pos = end
    return names


def check() -> int:
    wasm_path = OUT / f"{WASM_NAME}_bg.wasm"
    js_path = OUT / f"{WASM_NAME}.js"
    if not wasm_path.exists() or not js_path.exists():
        print(f"FAIL missing {wasm_path.relative_to(ROOT).as_posix()} or {js_path.name}; run scripts/web/build_wasm.py")
        return 1
    data = wasm_path.read_bytes()
    names = exports(data)
    bad = 0
    missing = sorted(REQUIRED_EXPORTS - set(names))
    print(("FAIL" if missing else "PASS") + f" required exports present ({', '.join(sorted(REQUIRED_EXPORTS))})" + (f"; missing: {', '.join(missing)}" if missing else ""))
    bad += bool(missing)
    extra = sorted(n for n in names if n not in REQUIRED_EXPORTS and n not in ALLOWED_EXTRA and not n.startswith(ALLOWED_PREFIXES))
    print(("FAIL" if extra else "PASS") + " no unexpected exports" + (f": {', '.join(extra)}" if extra else f" ({len(names)} exports)"))
    bad += bool(extra)
    gz = len(gzip.compress(data, compresslevel=9))
    ok = gz < GZIP_LIMIT_BYTES
    print(("PASS" if ok else "FAIL") + f" size: {len(data)} bytes, {gz} bytes gzip (limit {GZIP_LIMIT_BYTES})")
    bad += not ok
    print(f"RESULT {'PASS' if bad == 0 else 'FAIL'}")
    return 1 if bad else 0


def build() -> int:
    want, have = lock_version(), cli_version()
    if want != have:
        print(f"FAIL wasm-bindgen CLI {have} differs from the crate's Cargo.lock version {want}")
        return 1
    print(f"PASS wasm-bindgen CLI {have} equals Cargo.lock")
    commit, dirty = stamp()
    env = {**os.environ, "STRATO_BUILD_COMMIT": commit, "STRATO_BUILD_DIRTY": "1" if dirty else "0"}
    cargo = ["cargo", "build", "--release", "--locked", "--lib", "--target", TARGET, "--manifest-path", str(CRATE / "Cargo.toml")]
    print("RUN  " + " ".join(cargo[:-1] + ["crates/strato-wasm/Cargo.toml"]) + f"  (STRATO_BUILD_COMMIT={commit})")
    subprocess.run(cargo, check=True, env=env, cwd=ROOT)
    wasm = CRATE / "target" / TARGET / "release" / f"{WASM_NAME}.wasm"
    if OUT.exists():
        shutil.rmtree(OUT)
    bindgen = ["wasm-bindgen", "--target", "web", "--out-dir", str(OUT), str(wasm)]
    print("RUN  wasm-bindgen --target web --out-dir web/src/solver/pkg " + wasm.relative_to(ROOT).as_posix())
    subprocess.run(bindgen, check=True, cwd=ROOT)
    (OUT / ".gitignore").write_text("*\n", encoding="utf-8")
    return check()


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--check", action="store_true", help="check the existing build only")
    args = p.parse_args(argv)
    return check() if args.check else build()


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
