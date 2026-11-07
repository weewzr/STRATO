"""STRATO Phase 2.1 checks, part 5: parent-daughter chain with diffusion
(P21.17-P21.22). Shows that the overlap matrix O_nm is the identity when the
daughter diffusivities are the parent's (or a common multiple of them in every
layer), that it is a full matrix otherwise, and that the coupled modal solution
(P21.21) matches an independent two-species finite-volume run.

Synthetic data: R_1 = 175e-6 m, R_2 = 390e-6 m; parent D = (2e-15, 5e-16),
daughter D = (1e-15, 4e-15) m^2 s^-1; lambda_P = 3e-8, lambda_D = 1e-8 s^-1,
branching b = 1; parent starts uniform in the kernel (c* = 1 mol m^-3), daughter
starts at zero, no external source.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec3_p21_chain_overlap.py
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
DP = np.array([2e-15, 5e-16])
DD = np.array([1e-15, 4e-15])
LP, LD, BR = 3e-8, 1e-8, 1.0
CSTAR = 1.0
NM = 80
TIMES = (1e7, 5e7, 1e8, 2e8)


def modes(diffs):
    ml = Multilayer([R1, R2], diffs)
    lam = ml.eigenvalues(NM)
    N, I, I1, _ = ml.mode_integrals(lam, src_layers=[0])
    return ml, lam, N, I, I1


def overlap(mlA, lamA, NA, mlB, lamB):
    """O_nm = int r^2 phi_n^A phi_m^B dr / N_n^A = int u_n^A u_m^B dr / N_n^A."""
    kmax = np.maximum(np.sqrt(lamA[-1] / mlA.D), np.sqrt(lamB[-1] / mlB.D))
    r, wq, _ = mlA.grid(2 * kmax)
    UA = np.array([mlA.u_eval(l, r) for l in lamA])
    UB = np.array([mlB.u_eval(l, r) for l in lamB])
    return (UA * wq) @ UB.T / NA[:, None]


mlP, lamP, NP, IP, I1P = modes(DP)
mlD, lamD, ND, ID, I1D = modes(DD)
mlS, lamS, NS, _, _ = modes(3.0 * DP)       # same D ratios as the parent (scaled by 3)

O_same = overlap(mlP, lamP, NP, mlP, lamP)
O_scaled = overlap(mlS, lamS, NS, mlP, lamP)
O_diff = overlap(mlD, lamD, ND, mlP, lamP)
eye = np.eye(NM)
# sign of each eigenfunction is arbitrary, so compare |O| with I for the scaled case
results.append(report("P21.22 O = I when daughter D equals parent D (same element)",
                      np.max(np.abs(O_same - eye)) < 1e-9, f"max |O - I| = {np.max(np.abs(O_same - eye)):.1e}"))
results.append(report("P21.22 |O| = I when D_D = 3 D_P in every layer (Lambda scales by 3)",
                      np.max(np.abs(np.abs(O_scaled) - eye)) < 1e-9 and np.allclose(lamS / lamP, 3.0, rtol=1e-12),
                      f"max ||O| - I| = {np.max(np.abs(np.abs(O_scaled) - eye)):.1e}"))
off = np.max(np.abs(O_diff - np.diag(np.diag(O_diff))))
results.append(report("P21.19 O is a full matrix when the D ratios differ", off > 0.05,
                      f"max off-diagonal |O_nm| = {off:.3f}, O_11 = {O_diff[0, 0]:.4f}, O_21 = {O_diff[1, 0]:.4f}"))

# ---------------- coupled modal solution (P21.21) ------------------------------
kP, kD = lamP + LP, lamD + LD
aP0 = CSTAR * I1P / NP                       # kernel-uniform parent projection


def daughter_inventory(t, O):
    E = (np.exp(-kP[None, :] * t) - np.exp(-kD[:, None] * t)) / (kD[:, None] - kP[None, :])
    aD = BR * LP * np.sum(O * aP0[None, :] * E, axis=1)
    return 4 * math.pi * np.sum(ID * aD)


def parent_inventory(t):
    return 4 * math.pi * np.sum(IP * aP0 * np.exp(-kP * t))


# ---------------- two-species FV (Crank-Nicolson, time-centred coupling) ---------
def fv_chain(ncell, dt):
    fP = SphereFV([R1, R2], DP, [ncell, ncell])
    fD = SphereFV([R1, R2], DD, [ncell, ncell])
    CP = np.where(fP.layer == 0, CSTAR, 0.0)
    CD = np.zeros_like(CP)
    out = {}
    for step in range(1, int(round(TIMES[-1] / dt)) + 1):
        CPn, _, _ = fP.step(CP, dt, LP, 0.0)
        S = BR * LP * 0.5 * (CP + CPn)          # daughter source b lambda_P c_P, time-centred
        CD, _, _ = fD.step(CD, dt, LD, S)
        CP = CPn
        for tt in TIMES:
            if abs(step * dt - tt) < 0.25 * dt:
                out[tt] = (fP.inventory(CP), fD.inventory(CD))
    return out


errs, errs_diag = [], []
O_diag = np.diag(np.diag(O_diff))
for ncell, dt in [(40, 2e5), (80, 1e5), (160, 5e4)]:
    o = fv_chain(ncell, dt)
    errs.append(max(abs(o[tt][1] / daughter_inventory(tt, O_diff) - 1) for tt in TIMES))
    errs_diag.append(max(abs(o[tt][1] / daughter_inventory(tt, O_diag) - 1) for tt in TIMES))
    eP = max(abs(o[tt][0] / parent_inventory(tt) - 1) for tt in TIMES)
    print(f"cells/layer {ncell:>4}: parent rel diff {eP:.2e}, daughter rel diff (full O) {errs[-1]:.2e}, "
          f"(diagonal O only) {errs_diag[-1]:.2e}")
p = math.log2(errs[-2] / errs[-1])
results.append(report("P21.21 coupled modal daughter inventory = two-species FV (converging)", errs[-1] < 1e-3 and p > 1.5,
                      f"finest rel diff {errs[-1]:.1e}, order {p:.2f}"))
results.append(report("P21.20 dropping the off-diagonal overlap terms gives a visible error",
                      errs_diag[-1] > 10 * errs[-1], f"{errs_diag[-1]:.1e} vs {errs[-1]:.1e}"))

print(f"\nSUMMARY: {sum(results)}/{len(results)} PASS")
sys.exit(0 if all(results) else 1)
