"""Generate the web demo's data files with a provenance header.

This is the single generator for data that the web demo and other readers
(for example presentations) use. It writes JSON and CSV only; no number in
the web demo is typed by hand into a component.

Subcommands
-----------
case     Read a STRATO input case (TOML) and write its parameters as
         web/src/generated/<stem>.case.json (imported by the web build) and
         web/public/data/<stem>.case.csv (downloadable). The TOML is parsed
         with the Python standard library (tomllib); every value is copied
         unchanged, so the floating-point values are the ones the TOML text
         denotes. No physics is computed here.

catalogue
         Read the research catalogue tables in data/research/ (mechanisms,
         species, references, gallery; prompt/TECHNICAL_ARCHITECTURE.md
         Section 5.1) and the entry files under docs/research/, and write
         web/src/generated/research_catalogue.json, the only source of the
         research statuses the web demo shows. A table that does not exist
         yet is listed under ``tables_absent``. Values are copied; nothing is
         computed.

precomputed
         Run the native release binary (cargo run --locked --release) and the
         native wasm wrapper (crates/strato-wasm, example native_run) on
         inputs/synthetic/synthetic_five_layer.toml, and write their outputs
         to web/public/data/ as JSON and CSV with a PRECOMPUTED provenance
         header (prompt/TECHNICAL_ARCHITECTURE.md Section 5): the time series
         and final profile, Code B against Code C, the convergence study, a
         copy of tests/golden/v1.0/ for the browser parity check, and the
         native wrapper response, the reference of the browser parity test.
         Values are copied from the program outputs; nothing is computed
         here. scripts/web/precompute.sh calls this subcommand.

colormaps
         Read the cividis and viridis tables in data/colormaps/ (copied from
         matplotlib, see data/colormaps/README.md) and write
         web/src/generated/colormaps.json, the colour tables of the particle
         colouring. Display data only.

geometry
         Read the AGR-1 rows of data/geometry/geometry.csv and write
         web/src/generated/agr1_reference.json and
         web/public/data/agr1_reference_geometry.csv: the kernel diameter and
         coating thicknesses of each AGR-1 set with row ids, register status and
         source, and each layer's outer radius (sums of the registered values,
         checked against the register's own radius column where it has one).
         The reference particle is geometry only (rule E2).

Options
-------
--check  Regenerate in memory and compare with the committed files. Exit 1
         if they differ. The field ``created_utc`` is ignored, because it
         records when a file was written. For the catalogue the field
         ``commit`` (the tree the generator ran on) is ignored as well: a
         file cannot record the commit that contains it. The SHA-256 of every
         input file is compared and pins the content. For the precomputed
         files ``commit`` and ``rustc`` (the toolchain) are ignored as well,
         and in the native wrapper response its stamped ``engine.commit``;
         every value of the data is compared exactly.
--root   (catalogue only) Repository root to read from and write to; the
         default is this repository. Used by the fixture tests of
         scripts/web/check_catalogue.py.

Run from the repository root, with the standard library only:

    python -I -X utf8 scripts/web/generate_data.py case inputs/synthetic/synthetic_five_layer.toml
    python -I -X utf8 scripts/web/generate_data.py case inputs/synthetic/synthetic_five_layer.toml --check
    python -I -X utf8 scripts/web/generate_data.py catalogue
    python -I -X utf8 scripts/web/generate_data.py catalogue --check
"""

from __future__ import annotations

import argparse
import csv
import datetime as _dt
import decimal
import hashlib
import io
import json
import pathlib
import re
import subprocess
import sys
import tomllib

GENERATOR = "scripts/web/generate_data.py"
GENERATOR_VERSION = 1
SCHEMA = 1

ROOT = pathlib.Path(__file__).resolve().parents[2]
JSON_DIR = ROOT / "web" / "src" / "generated"
CSV_DIR = ROOT / "web" / "public" / "data"

# Units of the fields of strato::config::Config (src/config.rs at v1.0.1).
UNITS = {
    "temperature_k": "K",
    "outer_radius_m": "m",
    "diffusivity_m2_s": "m^2 s^-1",
    "cells": "1",
    "kernel_concentration_mol_m3": "mol m^-3",
    "dt_s": "s",
    "end_time_s": "s",
    "output_every_steps": "1",
}


def _git(*args: str, cwd: pathlib.Path = ROOT) -> str:
    out = subprocess.run(
        ["git", *args], cwd=cwd, check=True, capture_output=True, text=True
    )
    return out.stdout.strip()


def _strato_version(root: pathlib.Path = ROOT) -> str:
    path = root / "Cargo.toml"
    if not path.exists():
        return "unknown"
    with open(path, "rb") as fh:
        return tomllib.load(fh)["package"]["version"]


def _leading_comments(text: str) -> list[str]:
    notes = []
    for line in text.splitlines():
        s = line.strip()
        if not s.startswith("#"):
            break
        notes.append(s.lstrip("#").strip())
    return notes


def build_case(input_path: pathlib.Path) -> tuple[dict, str]:
    rel = input_path.resolve().relative_to(ROOT).as_posix()
    raw = input_path.read_bytes()
    text = raw.decode("utf-8")
    cfg = tomllib.loads(text)

    # The keys mirror strato::config::Config exactly, so the object can later
    # be sent to the wasm wrapper unchanged. Missing keys raise KeyError here.
    config = {
        "case_name": cfg["case_name"],
        "species": cfg["species"],
        "temperature_k": cfg["temperature_k"],
        "regions": [
            {
                "name": r["name"],
                "outer_radius_m": r["outer_radius_m"],
                "diffusivity_m2_s": r["diffusivity_m2_s"],
                "cells": r["cells"],
            }
            for r in cfg["regions"]
        ],
        "initial": {
            "kernel_concentration_mol_m3": cfg["initial"]["kernel_concentration_mol_m3"]
        },
        "time": {
            "dt_s": cfg["time"]["dt_s"],
            "end_time_s": cfg["time"]["end_time_s"],
            "output_every_steps": cfg["time"]["output_every_steps"],
        },
        "output": {"directory": cfg["output"]["directory"]},
    }

    header = {
        "schema": SCHEMA,
        "label": "INPUT",
        "kind": "input_case",
        "generator": f"{GENERATOR} v{GENERATOR_VERSION}",
        "strato_version": _strato_version(),
        "input_file": rel,
        "input_sha256": hashlib.sha256(raw).hexdigest(),
        "input_commit": _git("log", "-1", "--format=%H", "--", rel),
        "command": f"python -I -X utf8 {GENERATOR} case {rel}",
        "created_utc": _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "notes": _leading_comments(text),
        "units": UNITS,
    }
    doc = {**header, "data": {"config": config}}

    # CSV: provenance header as '# key: value' lines, then one row per value.
    buf = io.StringIO()
    for key in ("label", "kind", "generator", "strato_version", "input_file",
                "input_sha256", "input_commit", "command", "created_utc"):
        buf.write(f"# {key}: {header[key]}\n")
    for note in header["notes"]:
        buf.write(f"# note: {note}\n")
    w = csv.writer(buf, lineterminator="\n")
    w.writerow(["group", "region_index", "region_name", "key", "value", "unit"])
    w.writerow(["case", "", "", "case_name", config["case_name"], ""])
    w.writerow(["case", "", "", "species", config["species"], ""])
    w.writerow(["case", "", "", "temperature_k", repr(config["temperature_k"]), UNITS["temperature_k"]])
    for i, r in enumerate(config["regions"]):
        for key in ("outer_radius_m", "diffusivity_m2_s", "cells"):
            w.writerow(["region", i, r["name"], key, repr(r[key]), UNITS[key]])
    w.writerow(["initial", "", "", "kernel_concentration_mol_m3",
                repr(config["initial"]["kernel_concentration_mol_m3"]),
                UNITS["kernel_concentration_mol_m3"]])
    for key in ("dt_s", "end_time_s", "output_every_steps"):
        w.writerow(["time", "", "", key, repr(config["time"][key]), UNITS[key]])
    return doc, buf.getvalue()


def _json_text(doc: dict) -> str:
    return json.dumps(doc, indent=2, ensure_ascii=False) + "\n"


def _strip_created(text: str, is_json: bool) -> str:
    if is_json:
        d = json.loads(text)
        d.pop("created_utc", None)
        return json.dumps(d, sort_keys=True)
    return "\n".join(l for l in text.splitlines() if not l.startswith("# created_utc:"))


def cmd_case(args: argparse.Namespace) -> int:
    input_path = (ROOT / args.input).resolve() if not pathlib.Path(args.input).is_absolute() else pathlib.Path(args.input)
    stem = input_path.stem
    doc, csv_text = build_case(input_path)
    targets = [
        (JSON_DIR / f"{stem}.case.json", _json_text(doc), True),
        (CSV_DIR / f"{stem}.case.csv", csv_text, False),
    ]
    if args.check:
        bad = 0
        for path, text, is_json in targets:
            rel = path.relative_to(ROOT).as_posix()
            if not path.exists():
                print(f"MISSING {rel}")
                bad += 1
                continue
            old = path.read_text(encoding="utf-8")
            if _strip_created(old, is_json) != _strip_created(text, is_json):
                print(f"DIFFERS {rel}")
                bad += 1
            else:
                print(f"SAME    {rel}")
        return 1 if bad else 0
    for path, text, _ in targets:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8", newline="\n")
        print(f"WROTE   {path.relative_to(ROOT).as_posix()}")
    return 0


# ---------------------------------------------------------------------------
# Research catalogue (prompt/TECHNICAL_ARCHITECTURE.md Section 5.1)
# ---------------------------------------------------------------------------

CATALOGUE_KIND = "research_catalogue"
RESEARCH_DIR = "data/research"
ENTRY_DIR = "docs/research"
GALLERY_DIR = "docs/research/gallery"
CATALOGUE_JSON = "web/src/generated/research_catalogue.json"

# Exact headers, in order (Section 5.1). check_catalogue.py imports these.
TABLES: dict[str, list[str]] = {
    "mechanisms.csv": [
        "mechanism_id", "name", "regime", "brick", "builder_stage", "species",
        "material_domain", "status", "status_basis", "evidence", "validation_ids",
        "equation_ids", "coupled", "solver_capability", "required_parameters",
        "missing_work", "reference_ids", "entry", "status_date",
    ],
    "species.csv": [
        "species", "element", "nuclides", "layers", "regimes", "summary",
        "reference_ids", "entry",
    ],
    "references.csv": [
        "reference_id", "citation", "doi_or_url", "link_checked", "register_link",
        "notes",
    ],
    "gallery.csv": [
        "item_id", "kind", "title", "caption", "alt_text", "material", "species",
        "method", "conditions", "provenance", "file", "status", "mechanism_ids",
        "reference_ids",
    ],
}
# The two tables a catalogue cannot do without; species and gallery arrive with
# the catalogue content (web phase 5.5).
REQUIRED_TABLES = ("mechanisms.csv", "references.csv")

# Cells that hold ';'-separated lists.
LIST_COLUMNS = {
    "regime", "evidence", "validation_ids", "equation_ids", "required_parameters",
    "reference_ids", "nuclides", "layers", "regimes", "mechanism_ids",
}
INT_COLUMNS = {"brick", "builder_stage"}

_DISPLAY_MATH = re.compile(r"\$\$(.+?)\$\$", re.S)
_HEADING = re.compile(r"^(#{1,6})\s+(.*?)\s*$", re.M)


def read_table(root: pathlib.Path, name: str) -> tuple[list[str], list[list[str]], bytes] | None:
    """Return (header, rows, raw bytes) of data/research/<name>, or None if absent."""
    path = root / RESEARCH_DIR / name
    if not path.exists():
        return None
    raw = path.read_bytes()
    text = raw.decode("utf-8")
    rows = list(csv.reader(io.StringIO(text, newline="")))
    rows = [r for r in rows if r != []]
    if not rows:
        return [], [], raw
    return rows[0], rows[1:], raw


def split_list(cell: str) -> list[str]:
    return [p.strip() for p in cell.split(";") if p.strip()]


def _record(header: list[str], row: list[str]) -> dict:
    out: dict = {}
    for key, cell in zip(header, row):
        cell = cell.strip()
        if key in LIST_COLUMNS:
            out[key] = split_list(cell)
        elif key in INT_COLUMNS:
            out[key] = int(cell) if cell else None
        elif key == "coupled":
            out[key] = cell == "yes"
        else:
            out[key] = cell
    return out


def _entry_info(root: pathlib.Path, rel: str) -> dict | None:
    """Title and the research-concept equations of one entry file (display data only)."""
    if not rel:
        return None
    path = root / rel
    if not path.exists():
        return None
    text = path.read_text(encoding="utf-8")
    title = ""
    m = _HEADING.search(text)
    if m:
        title = m.group(2)
    # Equations between $$ under a heading that names candidate governing equations.
    concepts: list[str] = []
    headings = list(_HEADING.finditer(text))
    for i, h in enumerate(headings):
        if "candidate governing equations" not in h.group(2).lower():
            continue
        end = headings[i + 1].start() if i + 1 < len(headings) else len(text)
        concepts += [e.strip() for e in _DISPLAY_MATH.findall(text[h.end():end])]
    return {"path": rel, "title": title, "concept_equations": concepts}


def build_catalogue(root: pathlib.Path = ROOT) -> dict:
    inputs: list[dict] = []
    absent: list[str] = []
    data: dict = {}
    for name, header in TABLES.items():
        table = read_table(root, name)
        key = name.removesuffix(".csv")
        if table is None:
            absent.append(name)
            data[key] = []
            continue
        head, rows, raw = table
        inputs.append({"path": f"{RESEARCH_DIR}/{name}", "sha256": hashlib.sha256(raw).hexdigest()})
        data[key] = [_record(head, r) for r in rows]

    entries: dict[str, dict] = {}
    entry_dir = root / ENTRY_DIR
    if entry_dir.exists():
        for md in sorted(entry_dir.glob("*.md")):
            rel = md.relative_to(root).as_posix()
            inputs.append({"path": rel, "sha256": hashlib.sha256(md.read_bytes()).hexdigest()})
            info = _entry_info(root, rel)
            if info:
                entries[rel] = info
    data["entries"] = entries

    try:
        commit = _git("rev-parse", "HEAD", cwd=root) if (root / ".git").exists() else "unknown"
    except (OSError, subprocess.CalledProcessError):
        commit = "unknown"
    header = {
        "schema": SCHEMA,
        "label": "CATALOGUE",
        "kind": CATALOGUE_KIND,
        "generator": f"{GENERATOR} v{GENERATOR_VERSION}",
        "strato_version": _strato_version(root),
        "commit": commit,
        "command": f"python -I -X utf8 {GENERATOR} catalogue",
        "created_utc": _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "note": "CATALOGUE marks a data file of research statuses and references; it is not a result label.",
        "inputs": sorted(inputs, key=lambda d: d["path"]),
        "tables_absent": absent,
    }
    return {**header, "data": data}


CATALOGUE_VOLATILE = ("created_utc", "commit")


def catalogue_text(doc: dict) -> str:
    return _json_text(doc)


def comparable_catalogue(text: str) -> str:
    d = json.loads(text)
    for k in CATALOGUE_VOLATILE:
        d.pop(k, None)
    return json.dumps(d, sort_keys=True)


def cmd_catalogue(args: argparse.Namespace) -> int:
    root = pathlib.Path(args.root).resolve() if args.root else ROOT
    doc = build_catalogue(root)
    target = root / CATALOGUE_JSON
    text = catalogue_text(doc)
    rel = target.relative_to(root).as_posix()
    if args.check:
        if not target.exists():
            print(f"MISSING {rel}")
            return 1
        same = comparable_catalogue(target.read_text(encoding="utf-8")) == comparable_catalogue(text)
        print(f"{'SAME   ' if same else 'DIFFERS'} {rel}")
        return 0 if same else 1
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8", newline="\n")
    print(f"WROTE   {rel}")
    return 0


# ---------------------------------------------------------------------------
# Precomputed results (prompt/TECHNICAL_ARCHITECTURE.md Section 5)
# ---------------------------------------------------------------------------

PRECOMPUTED_INPUT = "inputs/synthetic/synthetic_five_layer.toml"
GOLDEN_DIR = "tests/golden/v1.0"
CLI_OUT_DIR = "results/synthetic-five-layer"
CLI_COMMAND = ["cargo", "run", "--locked", "--release", "--quiet", "--", PRECOMPUTED_INPUT]
NATIVE_COMMAND = [
    "cargo", "run", "--locked", "--release", "--quiet",
    "--manifest-path", "crates/strato-wasm/Cargo.toml", "--example", "native_run", "--", PRECOMPUTED_INPUT,
]
# Lines of the CLI's standard output that name files (their separators depend on the platform).
_PATH_KEYS = {"timeseries", "profile", "figure", "comparison", "convergence_study"}
# Header fields that record when, where and with which tools a file was written; --check
# ignores them (the data and the input hashes pin the content).
PRECOMPUTED_VOLATILE = ("created_utc", "commit", "rustc")
PRECOMPUTED_FILES = (
    "synthetic_run.json",
    "comparison_fv_vs_analytical.json",
    "convergence_study.json",
    "golden_v1.json",
    "wrapper_native_synthetic.json",
    "synthetic_run.timeseries.csv",
    "synthetic_run.final_profile.csv",
    "comparison_fv_vs_analytical.csv",
    "convergence_study.csv",
)


def _sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _columns(text: str) -> dict:
    """CSV text to column arrays. Numbers are read with float(), which returns the double
    that the shortest-round-trip text of the Rust csv writer denotes; an empty cell is None."""
    rows = list(csv.reader(io.StringIO(text)))
    head, body = rows[0], rows[1:]
    out: dict = {h: [] for h in head}
    for r in body:
        for h, cell in zip(head, r):
            if cell == "":
                out[h].append(None)
            else:
                try:
                    v: object = float(cell)
                except ValueError:
                    v = cell
                if h in ("cell", "material", "total_cells"):
                    v = int(cell)
                out[h].append(v)
    return out


def _stdout_summary(text: str) -> dict:
    out = {}
    for line in text.splitlines():
        if "=" not in line or line.startswith("WARNING"):
            continue
        k, v = line.split("=", 1)
        if k in _PATH_KEYS:
            continue
        out[k] = v
    return out


def _run(cmd: list[str], env: dict | None = None) -> str:
    res = subprocess.run(cmd, cwd=ROOT, check=True, capture_output=True, text=True, env=env)
    return res.stdout


def _rustc() -> str:
    try:
        return _run(["rustc", "-V"]).strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def build_precomputed() -> tuple[dict[str, str], dict[str, str]]:
    """Run the native release binary and the native wrapper on the synthetic case and return
    {file name: text} for web/public/data/. Values are copied from the program outputs;
    nothing is computed here."""
    import os

    inp = ROOT / PRECOMPUTED_INPUT
    try:
        commit_full = _git("rev-parse", "HEAD")
    except (OSError, subprocess.CalledProcessError):
        commit_full = "unknown"
    version = _strato_version()
    created = _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    rustc = _rustc()

    stdout = _run(CLI_COMMAND)
    out_dir = ROOT / CLI_OUT_DIR
    texts = {name: (out_dir / name).read_text(encoding="utf-8") for name in (
        "timeseries.csv", "final_profile.csv", "comparison_fv_vs_analytical.csv", "convergence_study.csv")}

    def header(kind: str, command: str, extra_inputs: list[dict] | None = None, note: str = "") -> dict:
        h = {
            "schema": SCHEMA,
            "label": "PRECOMPUTED",
            "kind": kind,
            "generator": f"strato {version}",
            "converter": f"{GENERATOR} v{GENERATOR_VERSION}",
            "strato_version": version,
            "commit": commit_full,
            "input_file": PRECOMPUTED_INPUT,
            "input_sha256": _sha256(inp),
            "command": command,
            "rustc": rustc,
            "created_utc": created,
        }
        if extra_inputs:
            h["inputs"] = extra_inputs
        if note:
            h["note"] = note
        return h

    cli = "cargo run --locked --release -- " + PRECOMPUTED_INPUT
    files: dict[str, str] = {}
    files["synthetic_run.json"] = _json_text({
        **header("synthetic_run", cli, note="Time series and final profile of the native STRATO v1.0 run (Code B)."),
        "data": {
            "timeseries": _columns(texts["timeseries.csv"]),
            "final_profile": _columns(texts["final_profile.csv"]),
            "summary": _stdout_summary(stdout),
        },
    })
    files["comparison_fv_vs_analytical.json"] = _json_text({
        **header("comparison_fv_vs_analytical", cli, note="Code B against Code C at every output time, as the CLI writes it; empty series cells are null."),
        "data": _columns(texts["comparison_fv_vs_analytical.csv"]),
    })
    summary = _stdout_summary(stdout)
    files["convergence_study.json"] = _json_text({
        **header("convergence_study", cli, note="Spatial and temporal refinement study of the native run; observed orders as the CLI prints them."),
        "data": {
            "points": _columns(texts["convergence_study.csv"]),
            "exact_final_release_fraction": summary.get("exact_final_release_fraction"),
            "spatial_observed_orders": summary.get("spatial_observed_orders"),
            "temporal_observed_orders": summary.get("temporal_observed_orders"),
        },
    })
    golden_names = ("timeseries.csv", "final_profile.csv", "comparison_fv_vs_analytical.csv", "convergence_study.csv")
    golden_inputs = [{"path": f"{GOLDEN_DIR}/{n}", "sha256": _sha256(ROOT / GOLDEN_DIR / n)} for n in golden_names]
    files["golden_v1.json"] = _json_text({
        **header("golden_v1", f"copy of {GOLDEN_DIR}/", golden_inputs,
                 "Values of the golden v1.0 files, copied for the browser parity check; not regenerated by any run."),
        "data": {n.removesuffix(".csv"): _columns((ROOT / GOLDEN_DIR / n).read_text(encoding="utf-8")) for n in golden_names},
    })
    env = {**os.environ, "STRATO_BUILD_COMMIT": commit_full[:7], "STRATO_BUILD_DIRTY": "0"}
    native = json.loads(_run(NATIVE_COMMAND, env=env))
    files["wrapper_native_synthetic.json"] = _json_text({
        **header("wrapper_native_run", "cargo run --locked --release --manifest-path crates/strato-wasm/Cargo.toml --example native_run -- " + PRECOMPUTED_INPUT,
                 note="run_v1 response of the native wrapper (x86_64), the reference of the browser parity test."),
        "data": native,
    })

    def csv_with_header(kind: str, body: str) -> str:
        h = header(kind, cli)
        lines = [f"# {k}: {h[k]}" for k in ("label", "kind", "generator", "converter", "strato_version", "commit",
                                             "input_file", "input_sha256", "command", "rustc", "created_utc")]
        return "\n".join(lines) + "\n" + body.replace("\r\n", "\n")

    files["synthetic_run.timeseries.csv"] = csv_with_header("synthetic_run_timeseries", texts["timeseries.csv"])
    files["synthetic_run.final_profile.csv"] = csv_with_header("synthetic_run_final_profile", texts["final_profile.csv"])
    files["comparison_fv_vs_analytical.csv"] = csv_with_header("comparison_fv_vs_analytical", texts["comparison_fv_vs_analytical.csv"])
    files["convergence_study.csv"] = csv_with_header("convergence_study", texts["convergence_study.csv"])
    return files, {"stdout": stdout, **texts}


def golden_comparison(raw: dict[str, str]) -> int:
    """The comparison of docs/development/provenance.md Section 3, automated: the four CSV
    files byte for byte, the standard output after normalising path separators."""
    bad = 0
    for name in ("timeseries.csv", "final_profile.csv", "comparison_fv_vs_analytical.csv", "convergence_study.csv"):
        golden = (ROOT / GOLDEN_DIR / name).read_bytes()
        same = raw[name].encode("utf-8") == golden
        print(f"{'SAME   ' if same else 'DIFFERS'} {CLI_OUT_DIR}/{name} against {GOLDEN_DIR}/{name} (byte for byte)")
        bad += not same
    want = (ROOT / GOLDEN_DIR / "main_run_stdout.txt").read_text(encoding="utf-8").splitlines()
    have = raw["stdout"].replace("\\", "/").splitlines()
    same = want == have
    print(f"{'SAME   ' if same else 'DIFFERS'} main-run standard output against {GOLDEN_DIR}/main_run_stdout.txt (path separators normalised, {len(have)} lines)")
    return bad + (not same)


def comparable_precomputed(name: str, text: str) -> str:
    if name.endswith(".json"):
        d = json.loads(text)
        for k in PRECOMPUTED_VOLATILE:
            d.pop(k, None)
        if isinstance(d.get("data"), dict) and isinstance(d["data"].get("engine"), dict):
            d["data"]["engine"].pop("commit", None)
        return json.dumps(d, sort_keys=True)
    keep = [l for l in text.splitlines() if not any(l.startswith(f"# {k}:") for k in PRECOMPUTED_VOLATILE)]
    return "\n".join(keep)


def cmd_precomputed(args: argparse.Namespace) -> int:
    files, raw = build_precomputed()
    golden_bad = golden_comparison(raw)
    if args.check:
        bad = golden_bad
        for name in PRECOMPUTED_FILES:
            path = CSV_DIR / name
            rel = path.relative_to(ROOT).as_posix()
            if not path.exists():
                print(f"MISSING {rel}")
                bad += 1
            elif comparable_precomputed(name, path.read_text(encoding="utf-8")) != comparable_precomputed(name, files[name]):
                print(f"DIFFERS {rel}")
                bad += 1
            else:
                print(f"SAME    {rel}")
        return 1 if bad else 0
    for name in PRECOMPUTED_FILES:
        path = CSV_DIR / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(files[name], encoding="utf-8", newline="\n")
        print(f"WROTE   {path.relative_to(ROOT).as_posix()}")
    return 0


# ---------------------------------------------------------------------------
# Colour maps of the particle colouring (web phase 4)
# ---------------------------------------------------------------------------

COLORMAP_DIR = "data/colormaps"
COLORMAP_NAMES = ("cividis", "viridis")
COLORMAP_JSON = "web/src/generated/colormaps.json"
VOLATILE = ("created_utc", "commit")


def _head_commit(root: pathlib.Path = ROOT) -> str:
    try:
        return _git("rev-parse", "HEAD", cwd=root) if (root / ".git").exists() else "unknown"
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def _now() -> str:
    return _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _comparable(text: str, is_json: bool) -> str:
    """Text of a generated file without the fields that record when and where it was written."""
    if is_json:
        d = json.loads(text)
        for k in VOLATILE:
            d.pop(k, None)
        return json.dumps(d, sort_keys=True)
    return "\n".join(
        l for l in text.splitlines()
        if not any(l.startswith(f"# {k}:") for k in VOLATILE)
    )


def _write_or_check(targets: list[tuple[pathlib.Path, str, bool]], check: bool) -> int:
    if check:
        bad = 0
        for path, text, is_json in targets:
            rel = path.relative_to(ROOT).as_posix()
            if not path.exists():
                print(f"MISSING {rel}")
                bad += 1
            elif _comparable(path.read_text(encoding="utf-8"), is_json) != _comparable(text, is_json):
                print(f"DIFFERS {rel}")
                bad += 1
            else:
                print(f"SAME    {rel}")
        return 1 if bad else 0
    for path, text, _ in targets:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8", newline="\n")
        print(f"WROTE   {path.relative_to(ROOT).as_posix()}")
    return 0


def build_colormaps() -> dict:
    """Copy the 256-entry tables of data/colormaps/ and add their 8-bit form.

    The 8-bit form is matplotlib's own conversion (``Colormap.__call__`` with
    ``bytes=True``: ``(lut * 255).astype(uint8)``, which truncates), so a colour
    on the page equals ``matplotlib.colormaps[name](i, bytes=True)``.
    """
    inputs = []
    maps: dict = {}
    for name in COLORMAP_NAMES:
        rel = f"{COLORMAP_DIR}/{name}.csv"
        raw = (ROOT / rel).read_bytes()
        inputs.append({"path": rel, "sha256": hashlib.sha256(raw).hexdigest()})
        rows = list(csv.reader(io.StringIO(raw.decode("utf-8"), newline="")))
        if rows[0] != ["index", "r", "g", "b"]:
            raise SystemExit(f"{rel}: unexpected header {rows[0]}")
        body = [r for r in rows[1:] if r]
        if [int(r[0]) for r in body] != list(range(256)):
            raise SystemExit(f"{rel}: expected the indices 0 to 255 in order")
        rgb = [[float(r[1]), float(r[2]), float(r[3])] for r in body]
        for triple in rgb:
            if not all(0.0 <= v <= 1.0 for v in triple):
                raise SystemExit(f"{rel}: component outside [0, 1]")
        maps[name] = {
            "entries": len(rgb),
            "rgb": rgb,
            "rgb8": [[int(v * 255) for v in t] for t in rgb],
        }
    readme = f"{COLORMAP_DIR}/README.md"
    inputs.append({"path": readme, "sha256": hashlib.sha256((ROOT / readme).read_bytes()).hexdigest()})
    header = {
        "schema": SCHEMA,
        "label": "DISPLAY",
        "kind": "colormaps",
        "generator": f"{GENERATOR} v{GENERATOR_VERSION}",
        "strato_version": _strato_version(),
        "commit": _head_commit(),
        "command": f"python -I -X utf8 {GENERATOR} colormaps",
        "created_utc": _now(),
        "note": (
            "Colour tables for displaying solver values; not data and not a result label. "
            "Copied from matplotlib 3.11.2 lib/matplotlib/_cm_listed.py (see data/colormaps/README.md). "
            "rgb8 = int(255 * rgb), matplotlib's bytes conversion."
        ),
        "inputs": inputs,
    }
    return {**header, "data": maps}


def cmd_colormaps(args: argparse.Namespace) -> int:
    doc = build_colormaps()
    return _write_or_check([(ROOT / COLORMAP_JSON, _json_text(doc), True)], args.check)


# ---------------------------------------------------------------------------
# AGR-1 reference particle (rule E2; prompt/DESIGN_SPECIFICATION.md 4.1 and 7.7)
# ---------------------------------------------------------------------------

GEOMETRY_CSV = "data/geometry/geometry.csv"
GEOMETRY_JSON = "web/src/generated/agr1_reference.json"
GEOMETRY_OUT_CSV = "web/public/data/agr1_reference_geometry.csv"
GEOMETRY_DEFAULT = "AGR1-B-M16"
LAYERS = ("kernel", "buffer", "IPyC", "SiC", "OPyC")
# The AGR-1 sets drawn as the reference particle, in display order: (set id, the
# design_id of each of the five layer rows). Every set gives the kernel diameter
# and four coating thicknesses of one register design. The IAEA (2012) Table 7.14
# variants take the kernel from design AGR1-T714 (row G-20, whose register note
# reads "single kernel value printed for all variants").
GEOMETRY_SETS = (
    ("AGR1-B-M16", ("AGR1-B-M16",) * 5),
    ("AGR1-NOM-C16", ("AGR1-NOM-C16",) * 5),
    ("AGR1-SPEC", ("AGR1-SPEC",) * 5),
    ("AGR1-T714-B", ("AGR1-T714",) + ("AGR1-T714-B",) * 4),
    ("AGR1-T714-V1", ("AGR1-T714",) + ("AGR1-T714-V1",) * 4),
    ("AGR1-T714-V2", ("AGR1-T714",) + ("AGR1-T714-V2",) * 4),
    ("AGR1-T714-V3", ("AGR1-T714",) + ("AGR1-T714-V3",) * 4),
)


def _geometry_row(rows: list[dict], design: str, layer: str) -> dict:
    want = ("diameter", "diameter (specified)") if layer == "kernel" else ("thickness", "thickness (specified)")
    hits = [r for r in rows if r["design_id"] == design and r["layer"] == layer and r["quantity"] in want]
    if len(hits) != 1:
        raise SystemExit(f"{GEOMETRY_CSV}: expected one {want[0]} row for {design} {layer}, found {len(hits)}")
    r = hits[0]
    if r["unit"] != "um" or not r["value"]:
        raise SystemExit(f"{GEOMETRY_CSV}: row {r['id']} has no value in um")
    return r


def _reader_location(text: str) -> str:
    """The register's source location without its local file name and text-extract index."""
    parts = [p.strip() for p in text.split(";")]
    parts = [re.sub(r"^\d+\.pdf\s*", "", p) for p in parts if not p.startswith("idx")]
    return "; ".join(p for p in parts if p)


LINK_VERIFIED = "public link in the register"
LINK_NOT_VERIFIED = "no verified public link in the register"


def _public_link(text: str) -> tuple[str | None, str]:
    """The register's link if it is a public http(s) URL, else None; and a status in words.

    The register also records local file notes such as "file:Paper/3.pdf (IAEA URL
    NOT VERIFIED)"; those are not links a reader can follow and are not shipped.
    """
    link = text.strip()
    if re.fullmatch(r"https?://\S+", link):
        return link, LINK_VERIFIED
    return None, LINK_NOT_VERIFIED


def build_geometry() -> tuple[dict, str]:
    """The AGR-1 layer dimensions of the register, with each layer's outer radius.

    Outer radii: kernel diameter / 2, then the previous radius plus the layer
    thickness (sums of register values, in micrometres). Where the register
    itself gives the radius (column outer_radius_um_calc) the two must agree to
    exactly (decimal arithmetic on the register text), or the generator stops. This is geometry bookkeeping for a drawing;
    no transport quantity is computed.
    """
    raw = (ROOT / GEOMETRY_CSV).read_bytes()
    rows = list(csv.DictReader(io.StringIO(raw.decode("utf-8"), newline="")))
    sets = []
    for set_id, designs in GEOMETRY_SETS:
        layers = []
        outer = decimal.Decimal(0)
        for layer, design in zip(LAYERS, designs):
            r = _geometry_row(rows, design, layer)
            value = decimal.Decimal(r["value"])
            outer = value / 2 if layer == "kernel" else outer + value
            registered = r["outer_radius_um_calc"].strip()
            if registered and decimal.Decimal(registered) != outer:
                raise SystemExit(f"{GEOMETRY_CSV}: row {r['id']} outer radius {registered} um, summed {outer!r} um")
            link, link_status = _public_link(r["link"])
            layers.append({
                "row_id": r["id"],
                "design_id": r["design_id"],
                "layer": layer,
                "material": r["material"],
                "quantity": r["quantity"],
                "value_um": float(value),
                "value_text": r["value"],
                "outer_radius_um": float(outer),
                "outer_radius_text": str(outer),
                "outer_radius_source": "register" if registered else "generator",
                "data_class": r["data_class"],
                "evidence_class": r["evidence_class"],
                "status": r["status"],
                "source": r["source"],
                "location": _reader_location(r["source_location"]),
                "link": link,
                "link_status": link_status,
                "notes": r["notes"],
            })
        main = next(r for r in rows if r["design_id"] == designs[-1])
        set_link, set_link_status = _public_link(main["link"])
        sets.append({
            "set_id": set_id,
            "design": main["design"],
            "source": main["source"],
            "link": set_link,
            "link_status": set_link_status,
            "evidence_class": main["evidence_class"],
            "statuses": sorted({l["status"] for l in layers}),
            "layers": layers,
        })
    header = {
        "schema": SCHEMA,
        "label": "REGISTER",
        "kind": "reference_geometry",
        "generator": f"{GENERATOR} v{GENERATOR_VERSION}",
        "strato_version": _strato_version(),
        "commit": _head_commit(),
        "input_file": GEOMETRY_CSV,
        "input_sha256": hashlib.sha256(raw).hexdigest(),
        "command": f"python -I -X utf8 {GENERATOR} geometry",
        "created_utc": _now(),
        "note": (
            "Registered geometry of the AGR-1 UCO TRISO particle (rule E2): geometry only, not a simulation and "
            "never combined with the diffusivities of the synthetic case. REGISTER marks register data; it is not "
            "a result label. Outer radii are sums of the registered diameter and thicknesses. location is the "
            "register's source location without its local file names and text-extract indices; link is given only "
            "where the register holds a public http(s) URL. commit is the source commit the file was generated from."
        ),
    }
    doc = {**header, "data": {"default_set": GEOMETRY_DEFAULT, "sets": sets}}

    buf = io.StringIO()
    for key in ("label", "kind", "generator", "strato_version", "commit", "input_file", "input_sha256", "command", "created_utc"):
        buf.write(f"# {key}: {header[key]}\n")
    buf.write(f"# note: {header['note']}\n")
    w = csv.writer(buf, lineterminator="\n")
    w.writerow(["set_id", "layer", "row_id", "design_id", "quantity", "value_um", "outer_radius_um",
                "outer_radius_source", "evidence_class", "status", "source", "location", "link"])
    for s in sets:
        for l in s["layers"]:
            w.writerow([s["set_id"], l["layer"], l["row_id"], l["design_id"], l["quantity"], l["value_text"],
                        l["outer_radius_text"], l["outer_radius_source"], l["evidence_class"], l["status"],
                        l["source"], l["location"], l["link"] or l["link_status"]])
    return doc, buf.getvalue()


def cmd_geometry(args: argparse.Namespace) -> int:
    doc, csv_text = build_geometry()
    return _write_or_check(
        [(ROOT / GEOMETRY_JSON, _json_text(doc), True), (ROOT / GEOMETRY_OUT_CSV, csv_text, False)],
        args.check,
    )


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    pc = sub.add_parser("precomputed", help="run the native binary and wrapper and write web/public/data/")
    pc.add_argument("--check", action="store_true")
    pc.set_defaults(func=cmd_precomputed)
    c = sub.add_parser("case", help="write the parameters of a TOML input case")
    c.add_argument("input")
    c.add_argument("--check", action="store_true")
    c.set_defaults(func=cmd_case)
    k = sub.add_parser("catalogue", help="write the research catalogue JSON")
    k.add_argument("--check", action="store_true")
    k.add_argument("--root", default="")
    k.set_defaults(func=cmd_catalogue)
    cm = sub.add_parser("colormaps", help="write the colour tables of the particle colouring")
    cm.add_argument("--check", action="store_true")
    cm.set_defaults(func=cmd_colormaps)
    g = sub.add_parser("geometry", help="write the AGR-1 reference-particle geometry from the register")
    g.add_argument("--check", action="store_true")
    g.set_defaults(func=cmd_geometry)
    args = p.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
