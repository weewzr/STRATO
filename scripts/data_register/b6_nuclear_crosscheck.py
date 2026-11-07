"""B6 audit: fresh IAEA LiveChart API download (ENSDF decay data, JEFF-3.1.1 thermal yields) compared cell by cell
with data/nuclear/decay.csv and data/nuclear/fission_yields.csv.

usage: python -I -X utf8 b6_nuclear_crosscheck.py <STRATO root> <raw download dir>
Raw API responses are cached in <raw download dir> (outside the repository); delete them to re-download.
Prints PASS/FAIL lines; writes nothing into the repository."""
import sys
import csv
import io
import math
import pathlib
import time
import urllib.request

ROOT = pathlib.Path(sys.argv[1]) / "data" / "nuclear"
RAW = pathlib.Path(sys.argv[2])
RAW.mkdir(parents=True, exist_ok=True)
API = "https://www-nds.iaea.org/relnsd/v1/data?"


def get(q, name):
    p = RAW / name
    if not p.exists():
        req = urllib.request.Request(API + q, headers={"User-Agent": "Mozilla/5.0"})
        p.write_bytes(urllib.request.urlopen(req, timeout=60).read())
        time.sleep(0.5)
    rd = csv.reader(io.StringIO(p.read_text(encoding="utf-8")))
    hdr = next(rd)
    # the levels endpoint repeats the header 'unc_hl' (first: in unit_hl, second: in seconds);
    # rename the second occurrence so DictReader semantics do not overwrite the first
    seen = set()
    for i, h in enumerate(hdr):
        if h in seen:
            hdr[i] = h + "_sec"
        seen.add(h)
    return [dict(zip(hdr, row)) for row in rd if row]


def rel(a, b):
    return abs(a - b) / max(abs(b), 1e-300)


fails = 0
checks = 0


def check(ok, msg):
    global fails, checks
    checks += 1
    if not ok:
        fails += 1
    print(("PASS " if ok else "FAIL ") + msg)


# ---------- decay data ----------
dec = list(csv.DictReader(open(ROOT / "decay.csv", encoding="utf-8")))
for r in dec:
    nuc = r["nuclide"]
    el, rest = nuc.split("-")
    A = rest.rstrip("m")
    iso = rest.endswith("m")
    key = f"{A}{el.lower()}"
    if iso:
        rows = get(f"fields=levels&nuclides={key}", f"lv_{key}.csv")
        e = float(r["level_keV"])
        cand = [x for x in rows if x["energy"] and abs(float(x["energy"]) - e) < 0.01]
        if not cand:
            check(False, f"{nuc}: level {e} keV not found")
            continue
        x = cand[0]
    else:
        rows = get(f"fields=ground_states&nuclides={key}", f"gs_{key}.csv")
        x = rows[0]
    if r["stable"].strip().upper() in ("TRUE", "YES", "1") or r["half_life"].upper() == "STABLE":
        check(x["half_life"].upper() == "STABLE", f"{nuc}: stable in API ({x['half_life']})")
        continue
    check(x["half_life"] == r["half_life"] and x["unit_hl"] == r["unit"] and x["unc_hl"] == r["unc_half_life"].split()[0],
          f"{nuc}: T1/2 {r['half_life']}({r['unc_half_life'].split()[0]}) {r['unit']} vs API {x['half_life']}({x['unc_hl']}) {x['unit_hl']}")
    hs = float(x["half_life_sec"])
    check(rel(float(r["half_life_s"]), hs) < 1e-9, f"{nuc}: T1/2 s {r['half_life_s']} vs API {hs:.10g}")
    lam = math.log(2) / hs
    check(rel(float(r["lambda_s-1"]), lam) < 1e-6, f"{nuc}: lambda {r['lambda_s-1']} vs ln2/T {lam:.7e}")
    check(x["decay_1"] == r["decay_mode_1"] and abs(float(x["decay_1_%"]) - float(r["branch_1_pct"])) < 1e-9,
          f"{nuc}: mode1 {r['decay_mode_1']} {r['branch_1_pct']} vs API {x['decay_1']} {x['decay_1_%']}")
    if r["decay_mode_2"]:
        check(x["decay_2"] == r["decay_mode_2"] and abs(float(x["decay_2_%"]) - float(r["branch_2_pct"])) < 1e-9,
              f"{nuc}: mode2 {r['decay_mode_2']} {r['branch_2_pct']} vs API {x['decay_2']} {x['decay_2_%']}")
    cut = x.get("ENSDFpublicationcut-off") or x.get("ENSDF_publication_cut-off")
    check(cut == r["ENSDF_cutoff"], f"{nuc}: ENSDF cut-off {r['ENSDF_cutoff']} vs API {cut}")

# year convention
cs = [r for r in dec if r["nuclide"] == "Cs-137"][0]
print("INFO seconds per year used by API:", float(cs["half_life_s"]) / float(cs["half_life"]))

# ---------- fission yields ----------
fy = list(csv.DictReader(open(ROOT / "fission_yields.csv", encoding="utf-8")))
api = {}
for par in ("235u", "239pu"):
    for kind in ("independent_fy", "cumulative_fy"):
        rows = get(f"fields={kind}&parents={par}", f"{kind}_{par}.csv")
        col, unc = ("independent_thermal_fy", "unc_it") if kind == "independent_fy" else ("cumulative_thermal_fy", "unc_ct")
        for x in rows:
            api[(par, kind[0], x["element_daughter"], int(x["a_daughter"]), int(x["daughter_level_idx"]))] = (x[col], x[unc])
sums = {}
for (par, k, el, A, idx), (v, u) in api.items():
    if k == "i" and v:
        sums[par] = sums.get(par, 0.0) + float(v)
print("INFO IFY sums:", {k: round(v, 5) for k, v in sums.items()})
for r in fy:
    par = r["parent"].lower().replace("235u", "235u").replace("239pu", "239pu")
    par = {"235u": "235u", "239pu": "239pu"}[par]
    el, A, idx = r["Z_element"], int(r["A"]), int(r["LiveChart_level_idx"])
    for k, c, cu in (("i", "IFY_JEFF311", "unc_IFY_JEFF311"), ("c", "CFY_JEFF311", "unc_CFY_JEFF311")):
        a = api.get((par, k, el, A, idx))
        tag = f"{r['parent']} {r['product']} idx{idx} {c}"
        if r[c] == "BLANK IN API":
            check(a is None or a[0] == "", f"{tag}: blank in CSV, API {a}")
            continue
        if a is None:
            check(False, f"{tag}: CSV {r[c]} but no API row")
            continue
        check(rel(float(r[c]), float(a[0])) < 1e-9 and rel(float(r[cu]), float(a[1])) < 1e-9,
              f"{tag}: {r[c]} +/- {r[cu]} vs API {a[0]} +/- {a[1]}")

# ---------- activation products must not get a fission source ----------
for r in dec:
    pc = r["product_class"]
    if r["nuclide"] in ("Ag-110m", "Cs-134", "Eu-154"):
        check(pc.startswith("activation product"), f"{r['nuclide']}: class '{pc}'")
for r in fy:
    if r["product"] in ("Ag-110m", "Cs-134", "Eu-154", "Ag-110", "Cs-134m", "Eu-154m"):
        note = r["STRATO_use_note"].lower()
        check(r["product_class"].startswith("activation product") and ("not" in note or "no " in note),
              f"{r['parent']} {r['product']}: class '{r['product_class']}', use note '{r['STRATO_use_note']}'")
print(f"SUMMARY {checks - fails}/{checks} PASS, {fails} FAIL")
