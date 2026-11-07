"""STRATO Phase 2.1 checks, part 4: the multilayer modal solution with source and
decay (P21.28-P21.42) against an independent conservative finite-volume solver
(sec3_p21_common.SphereFV, Crank-Nicolson) for a two-layer sphere.

Case 1: zero initial inventory, uniform kernel source S_0 for 0 <= t < t_irr,
        source off afterwards (irradiation then post-irradiation), decay lambda.
Case 2: no source, smooth initial profile over both layers, decay lambda
        (general initial-profile projection, P21.29).

All data are synthetic (chosen only to give O(1) dimensionless groups):
R_1 = 175e-6 m, R_2 = 390e-6 m, D = (2e-15, 5e-16) m^2 s^-1,
lambda = 1e-8 s^-1, S_0 = 1e-6 mol m^-3 s^-1, t_irr = 1e8 s.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec3_p21_fv_two_layer.py
Writes nothing to disk. Prints one PASS/FAIL line per check.
"""
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np

from sec3_p21_common import Multilayer, SphereFV, report

results = []
R1, R2 = 175e-6, 390e-6
DIFF = (2e-15, 5e-16)
LAM = 1e-8
S0 = 1e-6
T_IRR = 1e8
TIMES = (2e7, 5e7, 1e8, 1.5e8, 3e8)
NMODES = 400

# ---------------- modal side ---------------------------------------------------
ml = Multilayer([R1, R2], DIFF)
Lam = ml.eigenvalues(NMODES)
ok = all(ml.zero_crossings(Lam[j - 1], 80 * j + 400) == j - 1 for j in (1, 2, 3, 10, 50, 200, 400))
results.append(report("P21.24 Sturm check on two-layer eigenfunctions (n = 1..400 samples)", ok,
                      f"Lambda_1 = {Lam[0]:.4e} s^-1"))
Nn, In, I1, dU = ml.mode_integrals(Lam, src_layers=[0])
w = 3 * I1 * In / (R1 ** 3 * Nn)
results.append(report("P21.46 sum of v1.0 weights -> 1", abs(w.sum() - 1) < 5e-3, f"1 - sum w = {1 - w.sum():.2e} (N = {NMODES})"))
# flux identity (P21.36) on the layered eigenfunctions
rel = np.abs(R2 * DIFF[1] * dU / (-Lam * In) - 1)
results.append(report("P21.36 R^2 D_L phi_n'(R) = -Lambda_n I_n (two layers)", rel.max() < 1e-7, f"max rel err {rel.max():.1e}"))

k = Lam + LAM
sn = S0 * I1 / Nn                                   # (P21.30) kernel-uniform source projection
G = 4 / 3 * math.pi * R1 ** 3 * S0                  # generation rate (mol s^-1)


def modal_case1(t):
    """M, M_dec, M_rel (balance), J_R at time t for the irradiation/post-irradiation case."""
    if t <= T_IRR:
        a = sn / k * (-np.expm1(-k * t))
        A = sn / k * (t + np.expm1(-k * t) / k)
        JR = G * (1 - LAM * np.sum(w / k)) - 4 * math.pi * np.sum(Lam * In * sn / k * np.exp(-k * t))
        Mgen = G * t
    else:
        ai = sn / k * (-np.expm1(-k * T_IRR))
        Ai = sn / k * (T_IRR + np.expm1(-k * T_IRR) / k)
        a = ai * np.exp(-k * (t - T_IRR))
        A = Ai + ai * (-np.expm1(-k * (t - T_IRR))) / k
        JR = 4 * math.pi * np.sum(Lam * In * a)
        Mgen = G * T_IRR
    M = 4 * math.pi * np.sum(In * a)
    Mdec = 4 * math.pi * LAM * np.sum(In * A)
    return {"M": M, "Mdec": Mdec, "Mrel": Mgen - M - Mdec, "JR": JR}


# ---------------- FV side --------------------------------------------------------
def fv_case1(ncell, steps_irr):
    fv = SphereFV([R1, R2], DIFF, [ncell, ncell])
    dt = T_IRR / steps_irr
    C = np.zeros(fv.rc.size)
    src = np.where(fv.layer == 0, S0, 0.0)
    Mrel = Mdec = Mgen = 0.0
    out, t, step = {}, 0.0, 0
    nsteps = int(round(TIMES[-1] / dt))
    closure = 0.0
    for step in range(1, nsteps + 1):
        on = (step - 1) * dt < T_IRR - 0.5 * dt
        S = src if on else 0.0 * src
        C, rel, dec = fv.step(C, dt, LAM, S)
        Mrel += rel
        Mdec += dec
        Mgen += dt * np.sum(fv.V * S)
        t = step * dt
        M = fv.inventory(C)
        closure = max(closure, abs(Mgen - M - Mrel - Mdec) / max(Mgen, 1e-300))
        for tt in TIMES:
            if abs(t - tt) < 0.25 * dt:
                out[tt] = {"M": M, "Mdec": Mdec, "Mrel": Mrel, "JR": fv.G[-1] * C[-1]}
    return out, closure


grids = [(20, 250), (40, 500), (80, 1000), (160, 2000)]
errs = []
print("\nCase 1: max relative difference FV vs modal over t in", TIMES)
print(f"{'cells/layer':>11} {'M':>10} {'M_rel':>10} {'M_dec':>10} {'J_R':>10} {'closure':>10}")
for ncell, steps in grids:
    fv_out, closure = fv_case1(ncell, steps)
    e = {q: max(abs(fv_out[tt][q] / modal_case1(tt)[q] - 1) for tt in TIMES) for q in ("M", "Mrel", "Mdec", "JR")}
    errs.append(e)
    print(f"{ncell:>11} {e['M']:>10.2e} {e['Mrel']:>10.2e} {e['Mdec']:>10.2e} {e['JR']:>10.2e} {closure:>10.1e}")
    results.append(report(f"P21.42 discrete FV inventory closure, {ncell} cells/layer", closure < 1e-12, f"{closure:.1e}"))
for q in ("M", "Mrel", "Mdec", "JR"):
    p = math.log2(errs[-2][q] / errs[-1][q])
    results.append(report(f"P21.28 FV -> modal for {q}, observed order", p > 1.5 and errs[-1][q] < 2e-4,
                          f"order {p:.2f}, finest rel diff {errs[-1][q]:.1e}"))
m = modal_case1(T_IRR)
print(f"\nModal values at t_irr: M = {m['M']:.6e} mol, M_rel = {m['Mrel']:.6e} mol, "
      f"M_dec = {m['Mdec']:.6e} mol, R/B = {m['JR'] / G:.6f}")
RBss = 1 - LAM * np.sum(w / k)
RBss_dir = np.sum(w * Lam / k)
print(f"Steady R/B (balance form) = {RBss:.8f}; direct sum over {NMODES} modes = {RBss_dir:.8f}")
results.append(report("P21.49 steady R/B: direct sum differs from balance form by the weight tail",
                      abs((RBss - RBss_dir) - (1 - w.sum())) < 1e-12,
                      f"difference {RBss - RBss_dir:.3e}, 1 - sum w = {1 - w.sum():.3e}"))

# ---------------- Case 2: general initial profile, no source ----------------------
CSTAR = 1.0  # mol m^-3


def c0(r):
    return CSTAR * (1 - (r / R2) ** 2)


a0 = np.zeros(NMODES)
for j, lam_n in enumerate(Lam):
    r, wq, _ = ml.grid(np.sqrt(lam_n / np.array(DIFF)))
    a0[j] = np.sum(wq * r * c0(r) * ml.u_eval(lam_n, r)) / Nn[j]       # (P21.29)
M0 = CSTAR * 8 * math.pi * R2 ** 3 / 15
M0_modal = 4 * math.pi * np.sum(a0 * In)
results.append(report("P21.29 projection of c0 reproduces M_0 (truncated sum)", abs(M0_modal / M0 - 1) < 1e-6,
                      f"rel diff {abs(M0_modal / M0 - 1):.1e}"))
T2 = (1e7, 5e7, 1e8)


def modal_case2(t):
    a = a0 * np.exp(-k * t)
    A = a0 * (-np.expm1(-k * t)) / k
    M = 4 * math.pi * np.sum(In * a)
    Mdec = 4 * math.pi * LAM * np.sum(In * A)
    return {"M": M, "Mrel": M0 - M - Mdec}


def fv_case2(ncell, steps_per_1e7):
    fv = SphereFV([R1, R2], DIFF, [ncell, ncell])
    rf = fv.rf
    # exact cell averages of c0 over spherical shells
    C = CSTAR * (1 - 0.6 * (rf[1:] ** 5 - rf[:-1] ** 5) / (R2 ** 2 * (rf[1:] ** 3 - rf[:-1] ** 3)))
    dt = 1e7 / steps_per_1e7
    Mrel = 0.0
    out = {}
    for step in range(1, int(round(T2[-1] / dt)) + 1):
        C, rel, _ = fv.step(C, dt, LAM, 0.0)
        Mrel += rel
        for tt in T2:
            if abs(step * dt - tt) < 0.25 * dt:
                out[tt] = {"M": fv.inventory(C), "Mrel": Mrel}
    return out


e2 = []
for ncell, steps in [(40, 100), (80, 200), (160, 400)]:
    o = fv_case2(ncell, steps)
    e2.append(max(abs(o[tt][q] / modal_case2(tt)[q] - 1) for tt in T2 for q in ("M", "Mrel")))
p2 = math.log2(e2[-2] / e2[-1])
results.append(report("P21.29 general initial profile with decay: FV -> modal", e2[-1] < 1e-4 and p2 > 1.5,
                      f"rel diff {e2[0]:.1e}, {e2[1]:.1e}, {e2[2]:.1e}; order {p2:.2f}"))

print(f"\nSUMMARY: {sum(results)}/{len(results)} PASS")
sys.exit(0 if all(results) else 1)
