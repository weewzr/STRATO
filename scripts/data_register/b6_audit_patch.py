"""B6 audit patch (2026-11-06): apply the corrections of data/reference/B6_audit_log.md to the data CSVs.

usage: python -I -X utf8 b6_audit_patch.py <STRATO root>

Idempotent. Changes made:
  1. every CSV gets a column `evidence_class` (controlled vocabulary EVIDENCE below) that separates
     measured values, measured bounds, literature correlations, modelling conventions, specifications,
     synthetic benchmarks, calculated values, estimates and evaluated nuclear data;
  2. release_measurements.csv gets a column `particle_applicability` (compact vs particle level and what
     the value says about a single intact particle);
  3. location and note corrections listed in the audit log (Collin 2016 Fig. 6 folio, Fig. 12 label
     check, A4-02 graphite grade, explicit "no fission-yield source" for activation products).
No numerical value is changed.
"""
import csv
import io
import pathlib
import sys

ROOT = pathlib.Path(sys.argv[1]) / "data"

EVIDENCE = {
    "measured",
    "measured, digitised",
    "measured bound",
    "literature correlation",
    "literature correlation (upper bound)",
    "literature value (single temperature)",
    "modelling convention",
    "specification",
    "synthetic benchmark",
    "calculated",
    "estimate",
    "evaluated nuclear data",
    "basis not stated",
}


def rw(rel, fn, new_cols):
    p = ROOT / rel
    raw = p.read_text(encoding="utf-8")
    nl = "\r\n" if "\r\n" in raw else "\n"
    rd = csv.DictReader(io.StringIO(raw))
    hdr = list(rd.fieldnames)
    rows = list(rd)
    for c, after in new_cols:
        if c not in hdr:
            hdr.insert(hdr.index(after) + 1, c)
            for r in rows:
                r[c] = ""
    for r in rows:
        fn(r)
        assert r.get("evidence_class", "x") in EVIDENCE, (rel, r)
    out = io.StringIO()
    w = csv.DictWriter(out, fieldnames=hdr, lineterminator=nl)
    w.writeheader()
    w.writerows(rows)
    p.write_text(out.getvalue(), encoding="utf-8", newline="")
    print("patched", rel, len(rows), "rows")


# ------------------------------------------------------------------ arrhenius
ARR = {
    "recommended/fitted correlation": "literature correlation",
    "recommended/fitted correlation (upper bound)": "literature correlation (upper bound)",
    "measured/fitted single value": "literature value (single temperature)",
    "fitted to measured heating tests": "literature correlation",
    "modelling convention": "modelling convention",
    "model input": "modelling convention",
    "model input set": "literature correlation",
    "recommended (reduced coefficient)": "literature correlation",
    "benchmark specification": "literature correlation",
}


def arr(r):
    r["evidence_class"] = ARR[r["data_class"]]
    if r["id"] == "A4-02" and "irradiated A3-3" not in r["notes"]:
        r["notes"] = ("text p. 448 (idx 452): KFA metallic data of Table A-4 refer to irradiated A3-3 "
                      "matrix graphite; grade not printed in the table")
    if r["id"] in ("H21-04", "H21-06", "H21-09") and "Hales p. 3" not in r["notes"]:
        r["notes"] += ("; Hales p. 3 (idx 2) prints only D0,1 and Q1 and says additional high-temperature "
                       "terms are often used")


rw("diffusivity/arrhenius.csv", arr, [("evidence_class", "data_class")])

# ------------------------------------------------------------------ geometry


def geo(r):
    dc = r["data_class"]
    if dc == "measured upper bound":
        r["evidence_class"] = "measured bound"
    elif dc.startswith("measured"):
        r["evidence_class"] = "measured"
    elif dc in ("nominal", "nominal (rounded)", "specification", "design specification"):
        r["evidence_class"] = "specification"
    elif dc == "benchmark specification":
        assert r["design_id"] in ("CRP6-SENS", "CRP6-UOUT"), r["id"]
        r["evidence_class"] = "synthetic benchmark"
    else:
        raise KeyError(dc)


rw("geometry/geometry.csv", geo, [("evidence_class", "data_class")])

# ------------------------------------------------------------------ temperature
TH = {
    "specified": "specification",
    "specified (test conditions)": "specification",
    "specified (benchmark)": "specification",
    "specified (synthetic benchmark)": "synthetic benchmark",
    "specified (as modelled)": "modelling convention",
    "measured (text)": "measured",
    "measured (text) and reconstructed (duration)": "measured, digitised",
    "reconstructed": "measured, digitised",
    "specified/calculated (as-run)": "calculated",
    "calculated (as-run thermal analysis)": "calculated",
    "as printed (preliminary; basis not stated in notes)": "basis not stated",
    "measured/specified (as printed)": "basis not stated",
}


def temp(r):
    ec = TH[r["history_class"]]
    # first ramp of each modelled schedule starts from an assumed 30 C
    if r["history_class"] == "specified (as modelled)" and r["status"] == "NOT VERIFIED":
        ec = "estimate"
    r["evidence_class"] = ec


rw("temperature/temperature_histories.csv", temp, [("evidence_class", "history_class")])

# ------------------------------------------------------------------ release measurements
FIG6 = "6.pdf Fig. 6; printed p. 384; idx 6"
MATRIX = "Morris et al. (2016) highlights: Ag, Eu and Sr release dominated by release during irradiation"


def rel(r):
    vc, q = r["value_class"], r["qualifier"]
    if vc == "M printed table" or vc == "M (vector figure)":
        ec = "measured"
    elif vc in ("R-dig", "R-vis"):
        ec = "measured, digitised"
    elif vc == "M (PIE estimate)":
        ec = "estimate"
    elif vc == "M-UB":
        ec = "measured bound"
    elif vc == "M text":
        ec = "measured bound" if q == "<" else ("estimate" if q == ">" else "measured")
    else:
        raise KeyError(vc)
    r["evidence_class"] = ec

    if r["source_location"].startswith("6.pdf Fig. 6 (page not recorded"):
        r["source_location"] = FIG6
    if r["id"] == "E-077" and "render" not in r["notes"]:
        r["notes"] = ("label 4-4-2 confirmed as printed in Fig. 12 (render at 4x, 2026-11-06); compact 4-4-2 "
                      "appears in no table of either paper (safety-tested Capsule 4 level 4 compacts are 4-4-1 and "
                      "4-4-3); possibly a mislabel")
    if r["nuclide"] == "Sr-90" and r["source_location"].startswith("6.pdf Fig. 12") and "legend" not in r["notes"]:
        r["notes"] = (r["notes"] + "; " if r["notes"] else "") + "Fig. 12 legend labels the measured bars 'PIE'"

    prog, grp, nuc, qty = r["program"], r["group"], r["nuclide"], r["quantity"]
    if prog == "JAERI":
        pa = ("loose-particle batch (about 100 particles, no matrix); mean particle release; nearest analogue of a "
              "single intact particle, but the end-of-irradiation profile is not given (GAP-24)")
    elif prog == "CRP-6":
        pa = "fuel-sphere level (particles plus matrix); not a single-particle value"
    elif grp == "failure count":
        pa = "count per compact (or per test series); number of failed-particle sources, not a release value"
    elif grp == "failed SiC indicator":
        pa = "compact-level interpretive criterion (one particle = 2.4e-4 of a compact)"
    elif grp == "failed SiC" or grp == "between failures":
        pa = ("compact (about 4145 particles); dominated by particles with failed SiC; not applicable to a "
              "single intact particle")
    elif nuc == "Kr-85" and grp == "all":
        pa = ("compact-level bound over all particles of the test series (gas retained while at least one PyC "
              "layer is intact); background-limited")
    elif nuc == "Ce-144":
        pa = "post-test matrix inventory bound at compact level; not a release from a single particle"
    elif ec == "measured bound":
        pa = ("compact-level bound on the mean intact-particle release; includes background and matrix "
              "contamination, so it bounds but does not measure single intact-particle release")
    elif grp == "intact" and "matrix-corrected" in r["notes"] + r["time_basis"] + r["quantity"]:
        pa = ("compact with intact particles only, matrix-corrected (release after the first 24 h); compact "
              "release fraction equals the mean particle release fraction; nearest AGR-1 analogue of a single "
              "intact particle")
    elif grp == "intact" and nuc in ("Ag-110m", "Sr-90", "Eu-154"):
        pa = ("compact with intact particles only; includes in-pile inventory already in the matrix (" + MATRIX +
              "); not attributable to diffusion through a single intact particle during the test")
    elif grp == "intact":
        pa = ("compact with intact particles only; not matrix-corrected; mean particle release plus matrix and "
              "background contribution")
    elif grp == "intact phase":
        pa = "compact 4-3-2 before its first particle failure; compact level"
    else:
        pa = "compact-level statement; not a single-particle value"
    r["particle_applicability"] = pa


rw("experimental/release_measurements.csv", rel,
   [("evidence_class", "value_class"), ("particle_applicability", "evidence_class")])

# ------------------------------------------------------------------ nuclear data
ACT = {"Ag-110m", "Ag-110", "Cs-134", "Cs-134m", "Eu-154", "Eu-154m"}
ROUTE = {"Ag": "Ag-109(n,gamma)", "Cs": "Cs-133(n,gamma)", "Eu": "Eu-153(n,gamma)"}


def dec(r):
    r["evidence_class"] = "evaluated nuclear data"
    if r["nuclide"] in ("Ag-110m", "Cs-134", "Eu-154") and "NO fission-yield source" not in r["product_class"]:
        r["product_class"] += "; NO fission-yield source in STRATO"


def fyr(r):
    r["evidence_class"] = "evaluated nuclear data"
    if r["product"] in ACT and not r["STRATO_use_note"].startswith("do NOT"):
        el = r["product"].split("-")[0]
        r["STRATO_use_note"] = (f"do NOT use as a fission-yield source (activation product); source via "
                                f"{ROUTE[el]}, cross section NOT VERIFIED; was: {r['STRATO_use_note']}")


rw("nuclear/decay.csv", dec, [("evidence_class", "data_type")])
rw("nuclear/fission_yields.csv", fyr, [("evidence_class", "data_type")])
