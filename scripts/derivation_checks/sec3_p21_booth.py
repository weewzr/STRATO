"""STRATO Phase 2.1 checks, part 3: numerical comparison of the multilayer modal
formulas (P21.35-P21.49) against the single-sphere Booth solutions with source and
decay, IAEA-TECDOC-1674 Eqs (7.6)-(7.8) (printed p. 319), and a convergence
comparison of the direct flux series against the inventory-balance form.

The sphere is entered as two layers with the same D, so the interface jump rule
and the layer walk are exercised even though the answer is the homogeneous one.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec3_p21_booth.py
Writes nothing to disk. Prints one PASS/FAIL line per check.
"""
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from scipy.integrate import quad
from scipy.special import polygamma

from sec3_p21_common import Multilayer, report

results = []

# ---------------- problem data (synthetic, SI) ------------------------------
R = 175e-6            # sphere radius (m), AGR-1 kernel radius used only as a size
D = 1.0e-15           # diffusivity (m^2 s^-1), synthetic
Dp = D / R ** 2       # reduced diffusivity D' (s^-1)
NMODES = 1000

ml = Multilayer([R / 2, R], [D, D])
Lam = ml.eigenvalues(NMODES)
nidx = np.arange(1, NMODES + 1)
exact_Lam = nidx ** 2 * math.pi ** 2 * Dp
results.append(report("P21.24 eigenvalues = n^2 pi^2 D/R^2 (n <= 1000)",
                      np.max(np.abs(Lam / exact_Lam - 1)) < 1e-10,
                      f"max rel err {np.max(np.abs(Lam / exact_Lam - 1)):.1e}"))
ok = all(ml.zero_crossings(Lam[j - 1], 60 * j + 200) == j - 1 for j in (1, 2, 10, 100, 1000))
results.append(report("P21.24 Sturm check: mode n has n-1 interior zeros (n = 1,2,10,100,1000)", ok))

Nn, In, Is, dU = ml.mode_integrals(Lam, src_layers=[0, 1])
w = 3.0 * Is * In / (R ** 3 * Nn)          # source (and v1.0) weights with source radius R
results.append(report("P21.47 weights w_n = 6/(n^2 pi^2)", np.max(np.abs(w * nidx ** 2 * math.pi ** 2 / 6 - 1)) < 1e-9,
                      f"max rel err {np.max(np.abs(w * nidx ** 2 * math.pi ** 2 / 6 - 1)):.1e}"))
tail = 6 / math.pi ** 2 * polygamma(1, NMODES + 1)
results.append(report("P21.47 sum w_n = 1 - exact tail", abs(1 - w.sum() - tail) < 1e-11,
                      f"1 - sum = {1 - w.sum():.6e}, tail = {tail:.6e}"))
# flux identity R^2 D phi_n'(R) = -Lambda_n I_n, with phi' (R) = u'(R)/R since u(R) = 0
rel = np.abs(R * D * dU / (-Lam * In) - 1)
results.append(report("P21.36 R^2 D_5 phi_n'(R) = -Lambda_n I_n (numerical eigenfunctions)", rel.max() < 1e-8,
                      f"max rel err {rel.max():.1e}"))


# ---------------- Booth closed forms (TECDOC-1674 7.6-7.8) --------------------
NB = 200000
nb = np.arange(1, NB + 1, dtype=float)


def booth_ss(x):
    return 3 * math.sqrt(1 / x) * (1 / math.tanh(math.sqrt(x)) - math.sqrt(1 / x))


def booth_RB(x, Dt):            # (7.7), Dt = D' t
    lamt = x * Dt
    return booth_ss(x) - 6 * math.exp(-lamt) * np.sum(np.exp(-nb ** 2 * math.pi ** 2 * Dt) / (nb ** 2 * math.pi ** 2 + x))


def booth_F(x, Dt):             # (7.8)
    lamt = x * Dt
    ssum = np.sum(-np.expm1(-nb ** 2 * math.pi ** 2 * Dt) / (nb ** 2 * (nb ** 2 * math.pi ** 2 + x)))
    return booth_ss(x) - 6 / math.pi ** 2 * x * math.exp(-lamt) / -math.expm1(-lamt) * ssum  # x/(e^{lt}-1), overflow-safe


def booth_Mrel_over_P(x, Dt):   # integral of (7.7) in time, divided by D' (so in units of 1/D')
    lamt = x * Dt
    kk = nb ** 2 * math.pi ** 2 + x
    return booth_ss(x) * Dt - 6 * np.sum(-np.expm1(-kk * Dt) / kk ** 2)


# ---------------- modal formulas -----------------------------------------------
def modal(x, t, N=NMODES):
    lam = x * Dp
    L, ww = Lam[:N], w[:N]
    k = L + lam
    RB = 1 - lam * np.sum(ww / k) - np.sum(ww * L / k * np.exp(-k * t))      # (P21.47) accelerated
    MoverP = np.sum(ww * (-np.expm1(-k * t)) / k)                           # M/P from (P21.35)
    FB = 1 - lam * MoverP / (-math.expm1(-lam * t))                         # (P21.48)
    return RB, MoverP, FB


for x in (0.01, 1.0, 100.0):
    errRB, ratioF, errF = 0.0, 0.0, 0.0
    for Dt in (1e-3, 1e-2, 0.1, 1.0, 10.0):
        t = Dt / Dp
        RB, _, FB = modal(x, t)
        errRB = max(errRB, abs(RB - booth_RB(x, Dt)))
        e = abs(FB - booth_F(x, Dt))
        # truncation bound of M/P: sum_{n>N} w_n/k_n < 2/(pi^4 D' N^3), amplified by lambda/(1-e^{-lambda t})
        lamt = x * Dt
        bound = 2 / (math.pi ** 4 * NMODES ** 3) / Dt * (lamt / -math.expm1(-lamt)) + 1e-12
        ratioF, errF = max(ratioF, e / bound), max(errF, e)
    results.append(report(f"P21.48 modal R/B(t) vs TECDOC-1674 (7.7), lambda/D' = {x}", errRB < 1e-8, f"max abs diff {errRB:.1e}"))
    results.append(report(f"P21.48 modal F_B(t) vs TECDOC-1674 (7.8), lambda/D' = {x}", ratioF < 1.0,
                          f"max abs diff {errF:.1e}, within the N = {NMODES} truncation bound (ratio {ratioF:.2f})"))

# steady state R/B (P21.49) against (7.6)
for x in (0.01, 1.0, 100.0):
    lam = x * Dp
    k = Lam + lam
    ss_modal = 1 - lam * np.sum(w / k)
    results.append(report(f"P21.49 steady R/B = 1 - lambda sum w_n/k_n vs (7.6), lambda/D' = {x}",
                          abs(ss_modal - booth_ss(x)) < 1.1 * x * 2 / (math.pi ** 4 * NMODES ** 3) + 1e-12,
                          f"{ss_modal:.10f} vs {booth_ss(x):.10f} (truncation bound {x * 2 / (math.pi ** 4 * NMODES ** 3):.1e})"))

# M_out by direct convolution quadrature (P21.43); check M + M_out = P(1-e^{-lambda t})/lambda (P21.44)
x, Dt = 1.0, 0.3
lam, t = x * Dp, Dt / Dp


def JR_over_P(sv):
    k = Lam + lam
    return 1 - lam * np.sum(w / k) - np.sum(w * Lam / k * np.exp(-k * sv))


Mout_P = quad(lambda sv: JR_over_P(sv) * math.exp(-lam * (t - sv)), 0, t, limit=400, epsabs=0, epsrel=1e-11,
              points=[t * 1e-6, t * 1e-4, t * 1e-2])[0]
_, MoverP, FB = modal(x, t)
surv = -math.expm1(-lam * t) / lam
results.append(report("P21.44 M + M_out = P (1 - e^(-lambda t))/lambda (quadrature of J_R)",
                      abs((MoverP + Mout_P) / surv - 1) < 1e-7, f"rel diff {abs((MoverP + Mout_P) / surv - 1):.1e}"))
results.append(report("P21.48 F_B equals M_out/(M + M_out)", abs(Mout_P / (MoverP + Mout_P) - FB) < 1e-7,
                      f"{Mout_P / (MoverP + Mout_P):.9f} vs {FB:.9f}"))
Mrel_P = quad(JR_over_P, 0, t, limit=400, epsabs=0, epsrel=1e-11, points=[t * 1e-6, t * 1e-4, t * 1e-2])[0]
results.append(report("P21.45 M_rel > M_out when lambda > 0 (released atoms decay outside)", Mrel_P > Mout_P,
                      f"M_rel/P = {Mrel_P:.6e} s, M_out/P = {Mout_P:.6e} s"))

# ---------------- convergence: direct flux series vs balance --------------------
print("\nConvergence, lambda/D' = 1 (errors are absolute, dimensionless or in units of P/D')")
print(f"{'N':>6} {'R/B_ss direct':>15} {'R/B_ss balance':>15} {'M_rel direct':>14} {'M_rel balance':>14}")
x, Dt = 1.0, 0.1
lam, t = x * Dp, Dt / Dp
ref_ss = booth_ss(x)
ref_Mrel = booth_Mrel_over_P(x, Dt)           # in units of 1/D'
rows = []
for N in (10, 30, 100, 300, 1000):
    L, ww = Lam[:N], w[:N]
    k = L + lam
    ss_dir = np.sum(ww * L / k)                                   # sum w_n Lambda_n/k_n (P21.49 raw)
    ss_bal = 1 - lam * np.sum(ww / k)                             # balance form
    A = (t - (-np.expm1(-k * t)) / k) / k                         # int a_n ds / s_n
    Mrel_dir = np.sum(ww * L * A) * Dp                            # 4 pi sum Lambda I int a, over P, times D'
    Mrel_bal = (t - np.sum(ww * (-np.expm1(-k * t)) / k) - lam * np.sum(ww * A)) * Dp
    e = (abs(ss_dir - ref_ss), abs(ss_bal - ref_ss), abs(Mrel_dir - ref_Mrel), abs(Mrel_bal - ref_Mrel))
    rows.append((N,) + e)
    print(f"{N:>6} {e[0]:>15.3e} {e[1]:>15.3e} {e[2]:>14.3e} {e[3]:>14.3e}")
r = np.array(rows)
p_dir = -np.polyfit(np.log(r[:, 0]), np.log(r[:, 1]), 1)[0]
p_bal = -np.polyfit(np.log(r[:3, 0]), np.log(r[:3, 2]), 1)[0]
results.append(report("P21.39 direct flux series error ~ N^-1", abs(p_dir - 1) < 0.1, f"observed order {p_dir:.2f}"))
results.append(report("P21.39 balance form error ~ N^-3", abs(p_bal - 3) < 0.2, f"observed order {p_bal:.2f} (N <= 100, before round-off)"))
results.append(report("P21.39 balance form 1e4 times more accurate at N = 100",
                      r[2, 2] * 1e4 < r[2, 1] and r[2, 4] * 1e4 < r[2, 3],
                      f"R/B {r[2, 1]:.1e} vs {r[2, 2]:.1e}; M_rel {r[2, 3]:.1e} vs {r[2, 4]:.1e}"))

print(f"\nSUMMARY: {sum(results)}/{len(results)} PASS")
sys.exit(0 if all(results) else 1)
