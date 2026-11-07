"""Deliverable 5 (STRATO parameter register): consistency and derivation checks.

Run:
    .venv/Scripts/python.exe -I -X utf8 D5_parameter_register.py

Reads the CSV files under STRATO/data (read-only) and prints PASS/FAIL lines.
Writes nothing to disk.

Checks
  S  schema: one header row, unique ids, source and status columns, status vocabulary
  G  geometry: R_i = d_k/2 + sum_{j<=i} t_j (SymPy) against outer_radius_um_calc
  L  decay: lambda = ln2 / t_half and u(lambda) = lambda u(t)/t (SymPy derivative)
  F  Cs-in-SiC fluence factor exp(Gamma/5) and the printed check value of IAEA (2012) Eq. (10.9)
  K  reduced coefficient conversion D0 = D'0 r_k^2 (exact rationals)
  X  Collin (2016) and Hales (2021) rows equal the TECDOC-978 rows they cite
  A  FRG diffusivities at 1000/1600/1800 C reproduce the audit table (transcription check)
  B  CRP-6 case 1a/1b analytical values from the two-branch Cs kernel D (Booth series)
  Y  daughter source rule (R.6b): no double counting of the parent (SymPy ODE solution)
  T  temperature histories: ramp time = |dT|/|rate|, cumulative sums, printed totals
  E  release data: ranges, bounds, particle-failure fractions
"""
import csv
import math
import pathlib
import re

import sympy as sp

ROOT = pathlib.Path(__file__).resolve().parents[2]  # .../STRATO
DATA = ROOT / "data"
STATUS_OK = {"verified", "digitised", "NOT VERIFIED"}
n_fail = 0


def report(tag, ok, msg):
    """Print one PASS/FAIL line and count failures."""
    global n_fail
    if not ok:
        n_fail += 1
    print(f"{'PASS' if ok else 'FAIL'} [{tag}] {msg}")


def load(rel):
    with open(DATA / rel, newline="", encoding="utf-8") as f:
        text = f.read()
    rows = list(csv.DictReader(text.splitlines()))
    return text, rows


def fnum(s):
    """Parse a float, return None for blank or non-numeric cells."""
    try:
        return float(s)
    except (TypeError, ValueError):
        return None


# ---------------------------------------------------------------- S: schema
FILES = {
    "geometry/geometry.csv": ("id", "source", "status"),
    "diffusivity/arrhenius.csv": ("id", "source", "status"),
    "temperature/temperature_histories.csv": ("id", "source", "status"),
    "experimental/release_measurements.csv": ("id", "source", "status"),
    "nuclear/decay.csv": ("nuclide", "source", "status"),
    "nuclear/fission_yields.csv": (None, "source_JEFF", "status"),
}
tables = {}
for rel, (idcol, srccol, stcol) in FILES.items():
    text, rows = load(rel)
    tables[rel] = rows
    hdr = text.splitlines()[0].split(",")
    report("S", srccol in hdr and stcol in hdr, f"{rel}: source column '{srccol}' and status column present")
    bad = [r.get(stcol) for r in rows if r.get(stcol) not in STATUS_OK]
    report("S", not bad, f"{rel}: {len(rows)} rows, status values in {sorted(STATUS_OK)} (bad: {bad[:3]})")
    if idcol:
        ids = [r[idcol] for r in rows]
        report("S", len(ids) == len(set(ids)), f"{rel}: ids unique")
    ncols = {len(r) for r in rows}
    report("S", len(ncols) == 1 and None not in rows[0], f"{rel}: rectangular, no overflow cells")

# ------------------------------------------------------------- G: geometry
# Symbolic statement: outer radius of layer i = kernel radius + sum of thicknesses up to i.
dk, t2, t3, t4, t5 = sp.symbols("d_k t_2 t_3 t_4 t_5", positive=True)
Rsym = [dk / 2, dk / 2 + t2, dk / 2 + t2 + t3, dk / 2 + t2 + t3 + t4, dk / 2 + t2 + t3 + t4 + t5]
# thickness recovered from consecutive radii must be exact
recov = [sp.simplify(Rsym[i] - Rsym[i - 1]) for i in range(1, 5)]
report("G", recov == [t2, t3, t4, t5], "R_i - R_(i-1) = t_i symbolically (residual 0)")

geo = tables["geometry/geometry.csv"]
designs = {}
for r in geo:
    if r["outer_radius_um_calc"]:
        designs.setdefault(r["design_id"], {})[int(r["layer_index"])] = r
n_checked = 0
for did, layers in designs.items():
    if set(layers) != {1, 2, 3, 4, 5}:
        report("G", False, f"{did}: incomplete layer set {sorted(layers)}")
        continue
    vals = {dk: float(layers[1]["value"]), t2: float(layers[2]["value"]), t3: float(layers[3]["value"]),
            t4: float(layers[4]["value"]), t5: float(layers[5]["value"])}
    for i in range(1, 6):
        calc = float(Rsym[i - 1].subs(vals))
        tab = float(layers[i]["outer_radius_um_calc"])
        n_checked += 1
        if abs(calc - tab) > 1e-9:
            report("G", False, f"{did} layer {i}: table {tab} vs calc {calc}")
report("G", n_checked == 5 * len(designs), f"{len(designs)} designs, {n_checked} outer radii recomputed, all |diff| < 1e-9 um")
sens = [float(designs["CRP6-SENS"][i]["outer_radius_um_calc"]) for i in range(1, 6)]
report("G", sens == [250, 350, 390, 425, 465], f"CRP-6 sensitivity radii {sens} um equal audited 250/350/390/425/465")
agr = [float(designs["AGR1-B-M16"][i]["outer_radius_um_calc"]) for i in range(1, 6)]
report("G", abs(2 * agr[-1] - 788.4) < 1e-9, f"AGR-1 Baseline particle diameter 2R = {2*agr[-1]:.1f} um (nominal 780 um, IAEA 2012 p. 395)")

# ---------------------------------------------------------------- L: decay
T_s, uT = sp.symbols("T_half u_T", positive=True)
lam = sp.log(2) / T_s
u_lam = sp.Abs(sp.diff(lam, T_s)) * uT          # first-order propagation
report("L", sp.simplify(u_lam - lam * uT / T_s) == 0, "u(lambda) = |d lambda/dT| u(T) = lambda u(T)/T (residual 0)")
worst_l, worst_u, n_l = 0.0, 0.0, 0
for r in tables["nuclear/decay.csv"]:
    Ts = fnum(r["half_life_s"])
    if Ts is None:
        report("L", r["stable"] == "yes" and fnum(r["lambda_s-1"]) == 0.0, f"{r['nuclide']}: stable, lambda = 0")
        continue
    lam_tab = float(r["lambda_s-1"])
    lam_calc = math.log(2) / Ts
    ul_calc = lam_calc * float(r["unc_half_life_s"]) / Ts
    worst_l = max(worst_l, abs(lam_tab / lam_calc - 1))
    worst_u = max(worst_u, abs(float(r["unc_lambda_s-1"]) / ul_calc - 1))
    n_l += 1
report("L", worst_l < 1e-6, f"{n_l} radioactive rows: max |lambda_tab/lambda_calc - 1| = {worst_l:.2e} (< 1e-6)")
report("L", worst_u < 1e-3, f"{n_l} radioactive rows: max rel. diff of u(lambda) = {worst_u:.2e} (< 1e-3, 4-digit rounding)")
# printed IAEA (2012) Table 8.1 half-lives vs ENSDF (cross-check only)
yr = 365.25 * 86400.0
for nuc, t_printed in (("Cs-137", 30.07 * yr), ("Sr-90", 28.78 * yr), ("Ag-110m", 249.8 * 86400.0)):
    Ts = float(next(r for r in tables["nuclear/decay.csv"] if r["nuclide"] == nuc)["half_life_s"])
    d = t_printed / Ts - 1
    report("L", abs(d) < 5e-3, f"{nuc}: IAEA (2012) Table 8.1 half-life differs from ENSDF by {d:+.2e} (cross-check, < 0.5 %)")

# ------------------------------------------------------ F: fluence factor
G = sp.symbols("Gamma", nonnegative=True)
fac = sp.exp(G / 5)
want = {0: 1.000, 1: 1.221, 2: 1.492, 3: 1.822, 4.5: 2.460}
ok = all(abs(float(fac.subs(G, g)) - v) < 6e-4 for g, v in want.items())
report("F", ok, "exp(Gamma/5) at Gamma = 0, 1, 2, 3, 4.5 equals 1.000, 1.221, 1.492, 1.822, 2.460")
report("F", sp.limit(fac, G, 0) == 1, "fluence factor -> 1 as Gamma -> 0 (limit)")
arr = {r["id"]: r for r in tables["diffusivity/arrhenius.csv"]}
Rg = 8.3145
Tk = 1600 + 273.15
d1 = float(arr["E109-01"]["D0_m2_s"]) * math.exp(2 / 5) * math.exp(-float(arr["E109-01"]["Q_kJ_mol"]) * 1e3 / (Rg * Tk))
d2 = float(arr["E109-02"]["D0_m2_s"]) * math.exp(-float(arr["E109-02"]["Q_kJ_mol"]) * 1e3 / (Rg * Tk))
report("F", abs((d1 + d2) / 1.011e-16 - 1) < 1e-3,
       f"Eq. (10.9) at 1600 C, Gamma = 2: {d1:.4e} + {d2:.4e} = {d1+d2:.4e} m^2/s vs printed 1.011e-16")

# --------------------------------------------- K: reduced coefficient D'0
rk = sp.Rational(250, 10**6)
for rid in ("T71-01", "T71-02", "T71-03", "T71-04", "T71-05"):
    expr = arr[rid]["D0_expression"]
    dprime = sp.nsimplify(re.search(r"D'0 = ([0-9.eE+-]+)", expr).group(1))
    d0 = dprime * rk**2
    report("K", abs(float(d0) / float(arr[rid]["D0_m2_s"]) - 1) < 1e-12,
           f"{rid}: D0 = {dprime} s^-1 x (250 um)^2 = {float(d0):.5e} m^2/s matches table")

# ------------------------------------------- X: cross-set equality claims
n_x = 0
for r in tables["diffusivity/arrhenius.csv"]:
    m = re.search(r"equals (A\d-\d\d|C16-\d\d)", r["notes"])
    if not m:
        continue
    ref = arr[m.group(1)]
    same = (float(r["D0_m2_s"]) == float(ref["D0_m2_s"]) and
            float(r["Q_kJ_mol"]) == float(ref["Q_kJ_mol"]))
    n_x += 1
    if not same:
        report("X", False, f"{r['id']} != {ref['id']}")
report("X", n_x > 0, f"{n_x} 'equals' claims (Collin 2016 Table 2, Hales 2021 Table 1) match the cited TECDOC-978 rows")
for hid, (a, b) in {"H21-04": ("A1-01", "A1-02"), "H21-06": ("A3-01", "A3-02"), "H21-09": ("A3-03", "A3-04")}.items():
    ok = float(arr[hid]["D0_m2_s"]) == float(arr[a]["D0_m2_s"]) and arr[hid]["Q_kJ_mol"] == arr[a]["Q_kJ_mol"]
    report("X", ok, f"{hid}: Hales single term = first branch {a}; second branch {b} omitted")

# ------------------------------------------- A: FRG diffusivity reproduction


def D_of(ids, Tc, R=8.3143):
    T = Tc + 273.15
    return sum(float(arr[i]["D0_m2_s"]) * math.exp(-float(arr[i]["Q_kJ_mol"]) * 1e3 / (R * T)) for i in ids)


audit = {  # 03_tecdoc978_data.md, "Calculated values" table (R = 8.3143, Gamma = 0)
    "Cs UO2": (("A1-01", "A1-02"), (1.498e-16, 1.250e-13, 6.972e-13)),
    "Sr UO2": (("A1-03",), (2.093e-23, 5.421e-17, 1.114e-15)),
    "Ag UO2": (("A1-04",), (1.139e-15, 1.678e-13, 4.664e-13)),
    "Cs PyC": (("A2-01",), (4.911e-17, 4.061e-14, 1.606e-13)),
    "Sr PyC": (("A2-02",), (1.902e-14, 7.382e-12, 2.501e-11)),
    "Ag PyC": (("A2-03",), (2.547e-15, 2.691e-13, 6.984e-13)),
    "Cs SiC": (("A3-01", "A3-02"), (4.092e-19, 9.223e-17, 1.832e-15)),
    "Sr SiC": (("A3-03", "A3-04"), (4.661e-18, 2.462e-15, 2.936e-14)),
    "Ag SiC": (("A3-05",), (5.436e-18, 3.638e-15, 1.378e-14)),
}
worst = 0.0
for name, (ids, ref) in audit.items():
    for Tc, v in zip((1000, 1600, 1800), ref):
        worst = max(worst, abs(D_of(ids, Tc) / v - 1))
report("A", worst < 5e-4, f"27 FRG D(T) values reproduce the audit table, max rel. diff {worst:.1e} (4 significant figures)")
# R sensitivity quoted in the audit (8.3142 vs 8.3143)
sens_R = [D_of(ids, 1600, 8.3142) / D_of(ids, 1600) - 1 for ids, _ in audit.values()]
report("A", -3.8e-4 < min(sens_R) and max(sens_R) < -1.1e-4,
       f"R = 8.3142 instead of 8.3143 changes D(1600 C) by {min(sens_R):.2e} to {max(sens_R):.2e}")

# ----------------------------------------- B: CRP-6 case 1 (Booth series)
x, n = sp.symbols("x n", positive=True)
# F(x) = 1 - (6/pi^2) sum exp(-n^2 pi^2 x)/n^2 ; limit x -> 0 uses sum 1/n^2 = pi^2/6
s0 = sp.summation(1 / n**2, (n, 1, sp.oo))
report("B", sp.simplify(1 - 6 / sp.pi**2 * s0) == 0, "Booth F(D't = 0) = 1 - (6/pi^2) zeta(2) = 0 exactly")


def booth(Dt, nmax=20000):
    return 1.0 - 6.0 / math.pi**2 * sum(math.exp(-(k * math.pi) ** 2 * Dt) / k**2 for k in range(1, nmax + 1))


t = 200 * 3600.0
for Tc, printed, tol in ((1200, 0.4673, 3e-4), (1600, 0.99999959, 5e-9)):
    D = D_of(("A1-01", "A1-02"), Tc, R=8.3145)
    Dt = D * t / (250e-6) ** 2
    Fv = booth(Dt)
    report("B", abs(Fv - printed) < tol, f"case 1 at {Tc} C: D't = {Dt:.5f}, F = {Fv:.8f} vs printed {printed}")
Dt1 = D_of(("T71-01",), 1200, R=8.3145) * t / (250e-6) ** 2  # Table 7.1 Cs row, D'0 = 0.90 s^-1
report("B", abs(booth(Dt1) - 0.4614) < 5e-4, f"single branch (IAEA 2012 Table 7.1) gives F = {booth(Dt1):.4f}, not 0.4673")

# ------------------------------------- Y: yield source rule (no double count)
# Parent p (number density N_p) fed by fission, decaying to daughter d with branching b.
Fd, yIp, yIdau, b, lp = sp.symbols("Fdot y_I_p y_I_d b lambda_p", positive=True)
Np = sp.Function("N_p")
tt = sp.symbols("t", positive=True)
sol = sp.dsolve(sp.Eq(Np(tt).diff(tt), yIp * Fd - lp * Np(tt)), Np(tt), ics={Np(0): 0}).rhs
Np_inf = sp.limit(sol, tt, sp.oo)                       # steady parent inventory
yCd = yIdau + b * yIp                                   # cumulative-yield closure (one precursor)
right = yIdau * Fd + b * lp * Np_inf                    # (R.6b) daughter source
wrong = yCd * Fd + lp * Np_inf                          # CFY plus parent decay
report("Y", sp.simplify(right - yCd * Fd) == 0, "steady state: y_I(d) Fdot + b lambda_p N_p = y_C(d) Fdot (residual 0)")
report("Y", sp.simplify(wrong - yCd * Fd - yIp * Fd) == 0, "using y_C(d) Fdot + lambda_p N_p over-counts by exactly y_I(p) Fdot")
report("Y", sp.simplify(sol.subs(tt, 0)) == 0, "parent inventory starts at zero (initial condition satisfied)")

# -------------------------------------------------- T: temperature histories
hist = {}
for r in tables["temperature/temperature_histories.csv"]:
    hist.setdefault(r["history_id"], []).append(r)
n_rate = 0
for r in tables["temperature/temperature_histories.csv"]:
    rate, ramp = fnum(r["rate_C_per_h"]), fnum(r["ramp_h"])
    T0, T1 = fnum(r["T_start_C"]), fnum(r["T_end_C"])
    if rate is not None and ramp is not None and T0 is not None and T1 is not None:
        n_rate += 1
        if abs(abs(T1 - T0) / abs(rate) - ramp) > 6e-5:
            report("T", False, f"{r['id']}: ramp {ramp} h vs |dT|/rate {abs(T1-T0)/abs(rate):.5f} h")
report("T", n_rate == 12, f"{n_rate} ramp rows (3 Collin 2016 histories x 4 rate-specified ramps): ramp_h = |T_end - T_start| / |rate| within 6e-5 h")
n_cum = 0
for hid, rows in hist.items():
    run = 0.0
    for r in rows:
        if r["cumulative_end_h_calc"] == "":
            continue
        run += (fnum(r["ramp_h"]) or 0.0) + (fnum(r["hold_h"]) or 0.0)
        n_cum += 1
        if abs(run - float(r["cumulative_end_h_calc"])) > 1e-3:
            report("T", False, f"{r['id']}: cumulative {r['cumulative_end_h_calc']} vs running sum {run:.4f}")
report("T", True, f"{n_cum} cumulative-time entries equal the running sum of ramp + hold (no FAIL above)")
printed_tot = {"AGR1-ST-C16-1600": 334, "AGR1-ST-C16-1700": 336, "AGR1-ST-C16-1800": 338, "CRP6-6a": 333,
               "CRP6-7a": 277, "CRP6-7b": 229, "CRP6-8a": 541, "CRP6-8b": 187, "CRP6-9": 699,
               "CRP6-10": 836, "CRP6-11": 824}
for hid, tot in printed_tot.items():
    last = float(hist[hid][-1]["cumulative_end_h_calc"])
    report("T", abs(last - tot) < 0.5, f"{hid}: schedule sums to {last:.2f} h, printed total {tot} h")

# ---------------------------------------------------------- E: release data
rel = tables["experimental/release_measurements.csv"]
bad = [r["id"] for r in rel if r["unit"] == "1" and fnum(r["value"]) is not None and not 0 <= float(r["value"]) <= 1]
report("E", not bad, f"all {sum(r['unit']=='1' for r in rel)} dimensionless values lie in [0, 1] (bad: {bad})")
bad = [r["id"] for r in rel if fnum(r["value_min"]) is not None and fnum(r["value_max"]) is not None
       and float(r["value_min"]) > float(r["value_max"])]
report("E", not bad, f"value_min <= value_max in every range row (bad: {bad})")
for rid, frac in (("E-114", 9.1e-5), ("E-115", 4.8e-4), ("E-116", 1.4e-3)):
    r = next(q for q in rel if q["id"] == rid)
    f_calc = float(r["value"]) / float(r["value_max"])
    report("E", abs(f_calc / frac - 1) < 0.02, f"{rid}: failed-particle fraction {f_calc:.2e} vs audited {frac:.1e}")
s1800 = sum(float(r["value"]) for r in rel if r["quantity"] == "particles with failed IPyC and SiC" and r["T_C"] == "1800")
report("E", s1800 == 23, f"sum of Collin (2016) Table 3 failures at 1800 C = {s1800:.0f} = 23 (Morris 2016 p. 34)")
report("E", abs(1 / 4145 - 2.41e-4) < 5e-7, f"one particle = 1/4145 = {1/4145:.3e} of a compact (printed 2.4e-4)")
n_t49 = sum(r["program"] == "JAERI" for r in rel)
report("E", n_t49 == 33, f"TECDOC-978 Table 4-9: {n_t49} non-blank cells transcribed (expected 33)")

# ------------------------------------------------------------- A: sanity
bad = [r["id"] for r in tables["diffusivity/arrhenius.csv"]
       if (fnum(r["D0_m2_s"]) is not None and fnum(r["D0_m2_s"]) <= 0)
       or (fnum(r["Q_kJ_mol"]) is not None and fnum(r["Q_kJ_mol"]) < 0)
       or (fnum(r["T_min_C"]) is not None and fnum(r["T_max_C"]) is not None and fnum(r["T_min_C"]) > fnum(r["T_max_C"]))]
report("A", not bad, f"all D0 > 0, Q >= 0, T_min <= T_max in arrhenius.csv (bad: {bad})")

# ------------------------------------------- V: B6 audit (2026-11-06)
# Independent re-reading of every Ag, Cs and Sr row from page renders: IAEA (1997) Tables A-1 to A-5
# (2.pdf idx 443, 446, 450, 455, 457 at 2x), Collin et al. (2016) Table 2 (6.pdf idx 3 at 2.5x) and
# Hales et al. (2021) Table 1 (8.pdf idx 2, born-digital text layer). Tuple: (D0 m2/s, Q kJ/mol, Tmin C, Tmax C).
B6 = {
    "A1-01": (5.6e-8, 209, None, None), "A1-02": (5.2e-4, 362, None, None), "A1-03": (2.2e-3, 488, None, None),
    "A1-04": (6.7e-9, 165, None, None), "A1-08": (5.6e-8, 209, None, None), "A1-09": (5.2e-4, 362, None, None),
    "A1-10": (2.2e-3, 488, None, None), "A1-11": (6.7e-9, 165, None, None), "A1-12": (4.7e-10, 177, 1400, 1720),
    "A2-01": (6.3e-8, 222, None, None), "A2-02": (2.3e-6, 197, None, None), "A2-03": (5.3e-9, 154, None, None),
    "A2-06": (5.0e-5, 318, 1200, 1800), "A2-07": (2.3e-6, 197, 1000, 1650), "A2-08": (5.3e-9, 154, None, None),
    "A2-11": (6.0e-9, 198, 1250, 1385), "A2-12": (1.2e-3, 412, 1600, 2300), "A2-13": (5.3e-4, 193, 1400, 1700),
    "A3-01": (5.5e-14, 125, None, None), "A3-02": (1.6e-2, 514, None, None), "A3-03": (1.2e-9, 205, None, None),
    "A3-04": (1.8e6, 791, None, None), "A3-05": (3.6e-9, 215, None, None), "A3-06": (6.7e-14, 106, None, None),
    "A3-07": (1.1e-4, 437, None, None), "A3-08": (6.7e-14, 106, None, None), "A3-09": (2.4e-2, 482, None, None),
    "A3-10": (1.2e-9, 205, None, None), "A3-11": (1.8e6, 791, None, None), "A3-12": (3.6e-9, 215, None, None),
    "A3-13": (6.8e-12, 177, 1200, 1400), "A3-14": (2.5e-2, 503, 1600, 1900), "A3-15": (1.2e-9, 205, 1600, 1850),
    "A3-16": (6.8e-11, 177, 1200, 1400), "A3-20": (2.3e-17, None, 1000, 1000), "A3-21": (3.5e-10, 213, 1200, 2300),
    "A4-01": (3.6e-4, 189, None, None), "A4-02": (1.0e-2, 303, None, 2000), "A4-03": (6.8e1, 262, 800, 1300),
    "A4-04": (1.3, 246, 800, 1300), "A4-05": (1.6, 258, 800, 1300), "A4-10": (2.8e-4, 210, 1175, 1375),
    "A4-11": (3.5e-7, 116, 750, 1550), "A4-12": (5.6e-2, 312, 1200, 1600), "A4-13": (8.7e7, 414, 800, 1000),
    "A5-01": (1.7e-6, 149, 550, 1450), "A5-02": (1.7e-2, 268, 800, 2200), "A5-03": (1.6, 258, 490, 800),
    "A5-05": (1.7e-6, 149, 550, 1450), "A5-06": (1.7e-2, 268, 800, 2200), "A5-07": (9.0e-6, 157, 750, 1030),
    "A5-08": (6.3e-3, 264, 750, 1030), "A5-09": (1.2e-4, 112, 600, 1000), "A5-10": (1.7e-4, 95, 600, 1000),
    "A5-11": (5.8e-2, 151, 450, 750), "A5-12": (8.3e-1, 324, 850, 1400), "A5-13": (1.6e2, 364, 850, 1100),
    "C16-01": (6.7e-9, 165, None, None), "C16-02": (5.3e-9, 154, None, None), "C16-03": (3.6e-9, 215, None, None),
    "C16-04": (1.6, 258, None, None), "C16-05": (5.6e-8, 209, None, None), "C16-06": (5.2e-4, 362, None, None),
    "C16-07": (6.3e-8, 222, None, None), "C16-08": (5.5e-14, 125, None, None), "C16-09": (1.6e-2, 514, None, None),
    "C16-10": (3.6e-4, 189, None, None), "C16-11": (2.2e-3, 488, None, None), "C16-12": (2.3e-6, 197, None, None),
    "C16-13": (1.2e-9, 205, None, None), "C16-14": (1.8e6, 791, None, None), "C16-15": (1.0e-2, 303, None, None),
    "H21-01": (6.7e-9, 165, None, None), "H21-02": (5.3e-9, 154, None, None), "H21-03": (3.6e-9, 215, None, None),
    "H21-04": (5.6e-8, 209, None, None), "H21-05": (6.3e-8, 222, None, None), "H21-06": (5.5e-14, 125, None, None),
    "H21-07": (2.2e-3, 488, None, None), "H21-08": (2.3e-6, 197, None, None), "H21-09": (1.2e-9, 205, None, None),
}
arr_by_id = {r["id"]: r for r in tables["diffusivity/arrhenius.csv"]}
agcssr = [i for i, r in arr_by_id.items()
          if r["nuclide"].split()[0].split("-")[0] in ("Ag", "Cs", "Sr") and r["set_id"] in
          ("T978-A1", "T978-A2", "T978-A3", "T978-A4", "T978-A5", "C16-T2", "H21-T1")]
report("V", sorted(agcssr) == sorted(B6), f"B6 re-read covers every Ag/Cs/Sr row of A1-A5, C16-T2, H21-T1 "
       f"({len(B6)} rows; missing {sorted(set(agcssr) - set(B6))}, extra {sorted(set(B6) - set(agcssr))})")
mism = [i for i, (d0, q, a, b) in B6.items()
        if fnum(arr_by_id[i]["D0_m2_s"]) != d0 or fnum(arr_by_id[i]["Q_kJ_mol"]) != (None if q is None else float(q))
        or fnum(arr_by_id[i]["T_min_C"]) != (None if a is None else float(a))
        or fnum(arr_by_id[i]["T_max_C"]) != (None if b is None else float(b))]
report("V", not mism, f"B6 re-read: D0, Q, Tmin, Tmax identical to arrhenius.csv for {len(B6) - len(mism)}/{len(B6)} "
       f"rows (mismatch: {mism})")
rg = {r["set_id"][:3]: r["R_J_mol_K"] for r in tables["diffusivity/arrhenius.csv"]}
report("V", rg["T97"] == "8.3143" and rg["C16"] == "8.3145" and rg["H21"] == "not printed",
       f"R convention per set: TECDOC-978 {rg['T97']}, Collin 2016 {rg['C16']}, Hales 2021 '{rg['H21']}'")
EVID = {"measured", "measured, digitised", "measured bound", "literature correlation",
        "literature correlation (upper bound)", "literature value (single temperature)", "modelling convention",
        "specification", "synthetic benchmark", "calculated", "estimate", "evaluated nuclear data",
        "basis not stated"}
for relp, rows in tables.items():
    bad = [r.get("evidence_class") for r in rows if r.get("evidence_class") not in EVID]
    report("V", not bad, f"{relp}: evidence_class present and in vocabulary (bad: {bad[:3]})")
relrows = tables["experimental/release_measurements.csv"]
report("V", all(r.get("particle_applicability") for r in relrows),
       "release_measurements.csv: particle_applicability filled for every row")
ub = [r["id"] for r in relrows if r["qualifier"] == "<" and r["evidence_class"] != "measured bound"]
report("V", not ub, f"every '<' release value is classed 'measured bound' (bad: {ub})")
act = [r for r in tables["nuclear/fission_yields.csv"]
       if r["product"] in ("Ag-110m", "Ag-110", "Cs-134", "Cs-134m", "Eu-154", "Eu-154m")]
report("V", len(act) == 12 and all(r["STRATO_use_note"].startswith("do NOT use as a fission-yield source")
                                   for r in act),
       f"activation products (Ag-110m, Cs-134, Eu-154 and their isomer/ground partners): {len(act)} yield rows "
       f"all flagged 'do NOT use as a fission-yield source'")
src = [r["nuclide"] for r in tables["nuclear/decay.csv"] if r["product_class"].startswith("fission product")]
report("V", not set(src) & {"Ag-110m", "Cs-134", "Eu-154"},
       f"decay.csv fission-source nuclides {src} exclude Ag-110m, Cs-134, Eu-154")

print(f"SUMMARY: {'ALL PASS' if n_fail == 0 else f'{n_fail} FAIL'}")
