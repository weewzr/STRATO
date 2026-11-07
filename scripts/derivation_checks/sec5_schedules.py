"""Section 5 (Phase 2.2) input-definition checks for temperature schedules, (P22.58)-(P22.59).

1. AGR-1 safety-test PLAN as described by Collin et al. (2016, p. 379): 120 C/h ramps, 2 h at
   400 C, 12 h at 1250 C, 50 C/h to T_test, 300 h at T_test, cooling at -600 C/h to 30 C.
   The start temperature (30 C) is an ASSUMPTION; the plan is not the as-run furnace record.
   The printed modelled durations 334, 336, 338 h (Collin et al., 2016, p. 380) are the check.
2. CRP-6 heating schedules (IAEA, 2012, Table 10.11, format (T C, time to reach T h, time at T h));
   the printed totals 333, 333, 277, 229, 541, 187, 699 h are the check. Linear ramps between
   the listed temperatures are an ASSUMPTION of this conversion.
3. Effective-time bookkeeping on the AGR-1 plan: share of int D dt contributed by the ramps,
   with the TECDOC-978 FRG Cs-in-SiC and Cs-in-UO2 correlations (calculated, illustration of
   the input pipeline only).

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec5_schedules.py
The script writes no files.
"""
import math

import numpy as np
from scipy.special import expn

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


RG = 6.02214076e23 * 1.380649e-23


def agr1_plan(T_test, T_start=30.0):
    """Breakpoints (t h, T C) of the nominal AGR-1 safety-test plan (P22.58)."""
    pts = [(0.0, T_start)]

    def ramp_to(T, rate):
        t, T0 = pts[-1]
        pts.append((t + abs(T - T0) / rate, T))

    def hold(dt):
        t, T0 = pts[-1]
        pts.append((t + dt, T0))

    ramp_to(400.0, 120.0)
    hold(2.0)
    ramp_to(1250.0, 120.0)
    hold(12.0)
    ramp_to(T_test, 50.0)
    hold(300.0)
    ramp_to(30.0, 600.0)
    return pts


printed = {1600.0: 334.0, 1700.0: 336.0, 1800.0: 338.0}
for Tt, tot in printed.items():
    pts = agr1_plan(Tt)
    total = pts[-1][0]
    plateau_start = pts[5][0]
    print(f"INFO  AGR-1 plan {Tt:.0f} C breakpoints (h, C): " + ", ".join(f"({t:.4g}, {T:.0f})" for t, T in pts))
    check(f"AGR-1 plan {Tt:.0f} C total {total:.2f} h rounds to printed {tot:.0f} h", abs(total - tot) < 0.5,
          f"plateau starts at {plateau_start:.2f} h")

# ---------------------------------------------------------------------------
# CRP-6 Table 10.11 schedules (P22.59)
# ---------------------------------------------------------------------------
CRP6 = {
    "6a/6b": ([(300, None, 0.5), (1050, 1.5, 5.5), (1250, 0.5, 13.5), (1600, 7.5, 304)], 333),
    "7a": ([(20, None, 0.8), (1650, 5.4, None), (1700, 0.8, 270)], 277),
    "7b": ([(20, None, 0.4), (1750, 5.8, None), (1800, 0.8, 222)], 229),
    "8a": ([(300, None, 0.5), (1050, 1.5, 5.5), (1250, 0.5, 16.5), (1550, 6.5, None), (300, 1, None),
            (1600, 9, 500)], 541),
    "8b": ([(300, None, 0.5), (1050, 1.5, 5.5), (1250, 0.5, 13.5), (1800, 12, 25.5), (300, 1, None),
            (1050, 1.5, 19.5), (1250, 0.5, 19), (1800, 12, 74.5)], 187),
    "9": ([(300, None, 7), (1050, 2, 13.5), (1600, 11, 99), (20, 17, None), (1700, 5.5, 100), (20, 17, None),
           (1800, 2, 100), (20, 17, None), (300, 7, None), (1800, 1, 300)], 699),
    "10": ([(300, None, 6), (1250, 19, 200), (1600, 7, 200), (1700, 2, 200), (1800, 2, 200)], 836),
    "11": ([(300, None, 0.5), (1250, 2, 10), (1600, 7.5, 200), (1650, 1, 200), (1700, 1, 200), (1800, 2, 200)], 824),
}


def crp6_breakpoints(entries):
    """(T, time to reach, time at) -> breakpoints; '-' entries are zero duration (P22.59)."""
    t = 0.0
    pts = [(0.0, float(entries[0][0]))]
    for k, (T, reach, at) in enumerate(entries):
        if k > 0:
            t += reach or 0.0
            pts.append((t, float(T)))
        if at:
            t += at
            pts.append((t, float(T)))
    return pts


for case, (entries, tot) in CRP6.items():
    pts = crp6_breakpoints(entries)
    total = pts[-1][0]
    check(f"CRP-6 case {case:5s} summed duration {total:6.1f} h vs printed {tot} h", abs(total - tot) <= 1.0)

# ---------------------------------------------------------------------------
# Ramp share of int D dt on the AGR-1 1600 C plan (exact per linear segment, E2 form)
# ---------------------------------------------------------------------------
def seg_integral(terms, t0, T0, t1, T1):
    """int_{t0}^{t1} sum D0 exp(-Q/(RG T(t))) dt for linear T(t) in K, times in s."""
    total = 0.0
    for D0, Q in terms:
        a = Q / RG
        if abs(T1 - T0) < 1e-12:
            total += D0 * math.exp(-a / T0) * (t1 - t0)
        else:
            beta = (T1 - T0) / (t1 - t0)
            total += D0 * (T1 * expn(2, a / T1) - T0 * expn(2, a / T0)) / beta
    return total


SETS = {"Cs in SiC, FRG, Gamma=0": [(5.5e-14, 125e3), (1.6e-2, 514e3)],
        "Cs in UO2, FRG": [(5.6e-8, 209e3), (5.2e-4, 362e3)]}
pts = [(t * 3600.0, T + 273.15) for t, T in agr1_plan(1600.0)]
for name, terms in SETS.items():
    parts = [seg_integral(terms, *pts[i], *pts[i + 1]) for i in range(len(pts) - 1)]
    total = sum(parts)
    hold = parts[5]
    ramps = parts[4] + parts[6]
    print(f"INFO  AGR-1 1600 C plan, {name}: int D dt = {total:.3e} m^2; 1600 C hold {hold / total:.4f}, "
          f"50 C/h up-ramp {parts[4] / total:.4f}, cool-down {parts[6] / total:.2e}, start to end of 1250 C hold {sum(parts[:4]) / total:.2e}")
    check(f"segment shares sum to 1 ({name})", abs(sum(p / total for p in parts) - 1) < 1e-14)

# ---------------------------------------------------------------------------
# Thermocouple uncertainty (+-1 % of reading, Morris et al., 2016, p. 25) mapped to D via (P22.19)
# ---------------------------------------------------------------------------
for name, terms in SETS.items():
    for Tc in (1600.0, 1800.0):
        T = Tc + 273.15
        dT = 0.01 * Tc                      # 1 % of the reading taken in degrees C (assumption)
        Ds = [D0 * math.exp(-Q / (RG * T)) for D0, Q in terms]
        qeff = sum(Q * d for (D0, Q), d in zip(terms, Ds)) / sum(Ds)
        lin = qeff * dT / (RG * T**2)
        Dp = sum(D0 * math.exp(-Q / (RG * (T + dT))) for D0, Q in terms)
        exact = math.log(Dp / sum(Ds))
        print(f"INFO  {name:24s} {Tc:.0f} C, dT = {dT:.0f} K: dlnD linear {lin:.4f}, exact {exact:.4f}, "
              f"factor {math.exp(exact):.3f}")
        check(f"linearised dlnD within 5 % of exact ({name}, {Tc:.0f} C)", abs(lin / exact - 1) < 0.05)

print(f"\nSUMMARY {sum(RESULTS)}/{len(RESULTS)} checks passed")
