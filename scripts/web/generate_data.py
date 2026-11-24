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

Options
-------
--check  Regenerate in memory and compare with the committed files. Exit 1
         if they differ. The field ``created_utc`` is ignored, because it
         records when a file was written.

Run from the repository root, with the standard library only:

    python -I -X utf8 scripts/web/generate_data.py case inputs/synthetic/synthetic_five_layer.toml
    python -I -X utf8 scripts/web/generate_data.py case inputs/synthetic/synthetic_five_layer.toml --check
"""

from __future__ import annotations

import argparse
import csv
import datetime as _dt
import hashlib
import io
import json
import pathlib
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


def _git(*args: str) -> str:
    out = subprocess.run(
        ["git", *args], cwd=ROOT, check=True, capture_output=True, text=True
    )
    return out.stdout.strip()


def _strato_version() -> str:
    with open(ROOT / "Cargo.toml", "rb") as fh:
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


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("case", help="write the parameters of a TOML input case")
    c.add_argument("input")
    c.add_argument("--check", action="store_true")
    c.set_defaults(func=cmd_case)
    args = p.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
