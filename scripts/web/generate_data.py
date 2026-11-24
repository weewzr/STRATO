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

Options
-------
--check  Regenerate in memory and compare with the committed files. Exit 1
         if they differ. The field ``created_utc`` is ignored, because it
         records when a file was written. For the catalogue the field
         ``commit`` (the tree the generator ran on) is ignored as well: a
         file cannot record the commit that contains it. The SHA-256 of every
         input file is compared and pins the content.
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


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("case", help="write the parameters of a TOML input case")
    c.add_argument("input")
    c.add_argument("--check", action="store_true")
    c.set_defaults(func=cmd_case)
    k = sub.add_parser("catalogue", help="write the research catalogue JSON")
    k.add_argument("--check", action="store_true")
    k.add_argument("--root", default="")
    k.set_defaults(func=cmd_catalogue)
    args = p.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
