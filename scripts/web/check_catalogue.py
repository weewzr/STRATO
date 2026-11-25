"""Check the research catalogue files (prompt/TECHNICAL_ARCHITECTURE.md Section 5.2).

Standard library only. Reads data/research/*.csv, docs/research/*.md,
data/nuclear/decay.csv and web/src/generated/research_catalogue.json, writes
nothing, prints one PASS, FAIL or SKIP line per rule and exits 1 if any rule
fails. It checks form and references, not scientific truth: a status change
still needs the evidence and the independent review of the phase that
delivers it.

SKIP: species.csv and gallery.csv arrive with the catalogue content of web
phase 5.5. While a table is absent, the rules that need it print SKIP; a
table that is present is always checked in full.

Run from the repository root:

    python -I -X utf8 scripts/web/check_catalogue.py
    python -I -X utf8 scripts/web/check_catalogue.py --root <folder>   (fixture tests)
"""

from __future__ import annotations

import argparse
import csv
import importlib.util
import io
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent

_spec = importlib.util.spec_from_file_location("strato_generate_data", HERE / "generate_data.py")
assert _spec and _spec.loader
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)

STATUSES = ("PENDING", "RESEARCHED", "IMPLEMENTED", "VERIFIED", "VALIDATED")
SPECIES = ("Ag", "Pd", "Cs", "Sr", "Eu", "I", "Ru", "Xe", "Kr")
REGIMES = ("baseline", "A", "B", "C")
GALLERY_KINDS = ("screenshot", "reconstruction", "placeholder")
NOT_IN_REGISTER = "(not in register)"
# Overview pages of docs/research/ (Section 5.1). They describe several rows at
# once, so no single row can name them in its entry column. Rule 4 accepts
# exactly these names without a row; every other file must still be named by a
# row, and rule 3 checks the [ref:...] ids of these pages like any other file.
OVERVIEW_PAGES = (
    "docs/research/regime-a.md",
    "docs/research/regime-b.md",
    "docs/research/regime-c.md",
    "docs/research/materials_studio_gallery.md",
    "docs/research/roadmap.md",
)
REF_IN_TEXT = re.compile(r"\[ref:([^\]\s]+)\]")


class Report:
    def __init__(self) -> None:
        self.lines: list[str] = []
        self.failed = False

    def rule(self, number: int, name: str, problems: list[str], skipped: str = "") -> None:
        if skipped:
            self.lines.append(f"SKIP {number} {name}: {skipped}")
        elif problems:
            self.failed = True
            for p in problems:
                self.lines.append(f"FAIL {number} {name}: {p}")
        else:
            self.lines.append(f"PASS {number} {name}")


def load(root: pathlib.Path) -> dict[str, tuple[list[str], list[dict], bytes] | None]:
    out: dict[str, tuple[list[str], list[dict], bytes] | None] = {}
    for name in gen.TABLES:
        t = gen.read_table(root, name)
        if t is None:
            out[name] = None
            continue
        header, rows, raw = t
        out[name] = (header, [{"__line": i + 2, "__cells": len(r), **dict(zip(header, r))} for i, r in enumerate(rows)], raw)
    return out


def _path_part(item: str) -> str:
    """'data/diffusivity/arrhenius.csv#A1-01' -> 'data/diffusivity/arrhenius.csv'."""
    return item.split("#", 1)[0].strip()


def check(root: pathlib.Path) -> Report:
    rep = Report()
    tables = load(root)
    mech = tables["mechanisms.csv"]
    refs = tables["references.csv"]
    species = tables["species.csv"]
    gallery = tables["gallery.csv"]

    # 1. Headers, cell counts, encoding and line ends.
    probs: list[str] = []
    for name in gen.REQUIRED_TABLES:
        if tables[name] is None:
            probs.append(f"{gen.RESEARCH_DIR}/{name} is missing")
    for name, t in tables.items():
        if t is None:
            continue
        header, rows, raw = t
        if raw.startswith(b"\xef\xbb\xbf"):
            probs.append(f"{name}: starts with a byte-order mark")
        if b"\r" in raw:
            probs.append(f"{name}: contains CR line ends (LF only)")
        if header != gen.TABLES[name]:
            probs.append(f"{name}: header {header} differs from {gen.TABLES[name]}")
        for r in rows:
            if r["__cells"] != len(header):
                probs.append(f"{name} line {r['__line']}: {r['__cells']} cells, header has {len(header)}")
    rep.rule(1, "headers", probs)

    mrows = mech[1] if mech else []
    rrows = refs[1] if refs else []
    srows = species[1] if species else []
    grows = gallery[1] if gallery else []

    # 2. Status words.
    probs = []
    for table, rows in (("mechanisms.csv", mrows), ("gallery.csv", grows)):
        for r in rows:
            if r.get("status", "") not in STATUSES:
                probs.append(f"{table} line {r['__line']}: status '{r.get('status', '')}' is not one of {', '.join(STATUSES)}")
    rep.rule(2, "status words", probs)

    # 3. References.
    probs = []
    ref_ids = [r.get("reference_id", "") for r in rrows]
    known = set(ref_ids)
    for rid in sorted({x for x in ref_ids if ref_ids.count(x) > 1}):
        probs.append(f"references.csv: reference_id '{rid}' is not unique")
    for table, rows in (("mechanisms.csv", mrows), ("species.csv", srows), ("gallery.csv", grows)):
        for r in rows:
            for rid in gen.split_list(r.get("reference_ids", "")):
                if rid not in known:
                    probs.append(f"{table} line {r['__line']}: reference '{rid}' is not in references.csv")
    entry_dir = root / gen.ENTRY_DIR
    md_files = sorted(entry_dir.glob("*.md")) if entry_dir.exists() else []
    for md in md_files:
        for rid in REF_IN_TEXT.findall(md.read_text(encoding="utf-8")):
            if rid not in known:
                probs.append(f"{md.relative_to(root).as_posix()}: [ref:{rid}] is not in references.csv")
    rep.rule(3, "references", probs)

    # 4. Entry files.
    probs = []
    referenced: set[str] = set()
    for table, rows in (("mechanisms.csv", mrows), ("species.csv", srows)):
        for r in rows:
            entry = r.get("entry", "").strip()
            if not entry:
                continue
            referenced.add(entry)
            if not entry.startswith(gen.ENTRY_DIR + "/"):
                probs.append(f"{table} line {r['__line']}: entry '{entry}' is not under {gen.ENTRY_DIR}/")
            if not (root / entry).is_file():
                probs.append(f"{table} line {r['__line']}: entry file '{entry}' does not exist")
    for md in md_files:
        rel = md.relative_to(root).as_posix()
        if rel not in referenced and rel not in OVERVIEW_PAGES:
            probs.append(f"{rel} is referenced by no catalogue row")
    rep.rule(4, "entry files", probs)

    # 5. Uniqueness, species and regimes.
    probs = []
    seen: dict[tuple[str, str, str], int] = {}
    for r in mrows:
        key = (r.get("mechanism_id", ""), r.get("species", ""), r.get("material_domain", ""))
        if key in seen:
            probs.append(f"mechanisms.csv line {r['__line']}: duplicate (mechanism_id, species, material_domain) {key} (first at line {seen[key]})")
        else:
            seen[key] = r["__line"]
        if r.get("species", "") not in (*SPECIES, "all"):
            probs.append(f"mechanisms.csv line {r['__line']}: species '{r.get('species', '')}' is not one of the nine or 'all'")
        regimes = gen.split_list(r.get("regime", ""))
        if not regimes:
            probs.append(f"mechanisms.csv line {r['__line']}: empty regime")
        for g in regimes:
            if g not in REGIMES:
                probs.append(f"mechanisms.csv line {r['__line']}: regime '{g}' is not one of {', '.join(REGIMES)}")
    if species is None:
        rep.rule(5, "uniqueness, species and regimes", probs)
        rep.rule(5, "species table", [], skipped="species.csv absent (catalogue content of web phase 5.5)")
    else:
        names = [r.get("species", "") for r in srows]
        for s in SPECIES:
            if names.count(s) == 0:
                probs.append(f"species.csv: species '{s}' is missing")
            elif names.count(s) > 1:
                probs.append(f"species.csv: species '{s}' appears {names.count(s)} times")
        for n in names:
            if n not in SPECIES:
                probs.append(f"species.csv: '{n}' is not one of the nine species")
        for r in srows:
            for g in gen.split_list(r.get("regimes", "")):
                if g not in ("A", "B", "C"):
                    probs.append(f"species.csv line {r['__line']}: regime '{g}' is not A, B or C")
        rep.rule(5, "uniqueness, species and regimes", probs)

    # 6. Nuclides against data/nuclear/decay.csv.
    if species is None:
        rep.rule(6, "nuclides", [], skipped="species.csv absent (catalogue content of web phase 5.5)")
    else:
        probs = []
        decay = root / "data" / "nuclear" / "decay.csv"
        registered: set[str] = set()
        if decay.exists():
            registered = {row.get("nuclide", "") for row in csv.DictReader(io.StringIO(decay.read_text(encoding="utf-8")))}
        else:
            probs.append("data/nuclear/decay.csv is missing")
        for r in srows:
            for n in gen.split_list(r.get("nuclides", "")):
                if n.endswith(NOT_IN_REGISTER):
                    continue
                if n not in registered:
                    probs.append(f"species.csv line {r['__line']}: nuclide '{n}' is not in data/nuclear/decay.csv and is not marked '{NOT_IN_REGISTER}'")
        rep.rule(6, "nuclides", probs)

    # 7. Evidence for the higher statuses.
    probs = []
    for r in mrows:
        status = r.get("status", "")
        line = r["__line"]
        evidence = gen.split_list(r.get("evidence", ""))
        for item in evidence:
            p = _path_part(item)
            if p and not (root / p).exists():
                probs.append(f"mechanisms.csv line {line}: evidence path '{p}' does not exist")
        if status in ("RESEARCHED", "IMPLEMENTED", "VERIFIED", "VALIDATED") and not evidence:
            probs.append(f"mechanisms.csv line {line}: status {status} needs evidence")
        if status in ("IMPLEMENTED", "VERIFIED", "VALIDATED") and not r.get("solver_capability", "").strip():
            probs.append(f"mechanisms.csv line {line}: status {status} needs a solver_capability")
        if status in ("VERIFIED", "VALIDATED"):
            if not any(_path_part(e).startswith(("tests/", "crates/strato-wasm/tests/")) for e in evidence):
                probs.append(f"mechanisms.csv line {line}: status {status} needs evidence under tests/ or crates/strato-wasm/tests/")
        if status == "VALIDATED" and not gen.split_list(r.get("validation_ids", "")):
            probs.append(f"mechanisms.csv line {line}: status VALIDATED needs validation_ids")
        if status != "VALIDATED" and gen.split_list(r.get("validation_ids", "")):
            probs.append(f"mechanisms.csv line {line}: validation_ids must be empty unless the status is VALIDATED")
    rep.rule(7, "evidence", probs)

    # 8. Gallery.
    gdir = root / gen.GALLERY_DIR
    gfiles = sorted(p for p in gdir.rglob("*") if p.is_file()) if gdir.exists() else []
    if gallery is None:
        if gfiles:
            rep.rule(8, "gallery", [f"{len(gfiles)} file(s) under {gen.GALLERY_DIR}/ but no gallery.csv"])
        else:
            rep.rule(8, "gallery", [], skipped="gallery.csv absent (catalogue content of web phase 5.5)")
    else:
        probs = []
        listed = set()
        for r in grows:
            kind = r.get("kind", "")
            f = r.get("file", "").strip()
            line = r["__line"]
            if kind not in GALLERY_KINDS:
                probs.append(f"gallery.csv line {line}: kind '{kind}' is not one of {', '.join(GALLERY_KINDS)}")
            if kind in ("screenshot", "reconstruction"):
                if not f or not (root / f).is_file():
                    probs.append(f"gallery.csv line {line}: a {kind} row needs an existing file")
                if not r.get("provenance", "").strip():
                    probs.append(f"gallery.csv line {line}: a {kind} row needs a provenance")
            if kind == "placeholder" and f:
                probs.append(f"gallery.csv line {line}: a placeholder row must have an empty file")
            if f:
                listed.add(f)
        for p in gfiles:
            rel = p.relative_to(root).as_posix()
            if rel not in listed:
                probs.append(f"{rel} is not listed in gallery.csv")
        rep.rule(8, "gallery", probs)

    # 9. The generated JSON equals a fresh regeneration.
    target = root / gen.CATALOGUE_JSON
    if not target.exists():
        rep.rule(9, "generated catalogue", [f"{gen.CATALOGUE_JSON} is missing; run generate_data.py catalogue"])
    else:
        fresh = gen.comparable_catalogue(gen.catalogue_text(gen.build_catalogue(root)))
        old = gen.comparable_catalogue(target.read_text(encoding="utf-8"))
        rep.rule(9, "generated catalogue", [] if fresh == old else [f"{gen.CATALOGUE_JSON} differs from a fresh regeneration (stale)"])
    return rep


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--root", default="", help="repository root (default: this repository)")
    args = p.parse_args(argv)
    root = pathlib.Path(args.root).resolve() if args.root else gen.ROOT
    rep = check(root)
    for line in rep.lines:
        print(line)
    print("RESULT FAIL" if rep.failed else "RESULT PASS")
    return 1 if rep.failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
