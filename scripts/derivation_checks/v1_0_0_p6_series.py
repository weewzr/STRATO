"""STRATO v1.0.0, Full Mathematical Derivation, Part VI: accuracy of the Code C series.

Replica of src/analytical/sturm_liouville.rs (v1.0.0) in double precision for the synthetic
five-layer input: Sturm count (SL 164-192), root isolation by bisection on the count
(SL 197-264), weights w_n = 3 I1 I / (R1^3 N) (SL 352) with closed-form layer integrals, and
an independent 40-digit mpmath refinement of every root and weight (oracle). Checks the
truncation and validity-time statements, the weight-sum deficit, the propagation of
weight and eigenvalue errors into F, and the early-time cancellation.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/v1_0_0_p6_series.py
Writes nothing. One PASS/FAIL line per check; exit code 1 on any FAIL.
"""

import math
import sys

import mpmath as mp
import numpy as np
import sympy as sp

ok_all = []


def check(label, ok, detail=""):
    ok = bool(ok)
    ok_all.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {label}" + (f"  [{detail}]" if detail else ""), flush=True)


Rl = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4]
Dl = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13]


def count(lam):
    n, u, q, left, prev = 0, 0.0, 1.0, 0.0, None
    for Ro, d in zip(Rl, Dl):
        if prev is not None:
            q = q + (d - prev) * u / left
        k = math.sqrt(lam / d)
        th = math.atan2(u, q / (d * k))
        L = Ro - left
        n += math.floor((th + k * L) / math.pi) - math.floor(th / math.pi)
        u, q = math.cos(k * L) * u + math.sin(k * L) * q / (d * k), -d * k * math.sin(k * L) * u + math.cos(k * L) * q
        m = max(abs(u), abs(q / (d * k)))
        if m > 0 and math.isfinite(m):
            u, q = u / m, q / m
        left, prev = Ro, d
    return max(n, 0)


def eigenvalues(lam_max):
    hi = lam_max
    nhi = count(hi)
    rate = sum((Ro - l0) / math.sqrt(d) for Ro, l0, d in zip(Rl, [0.0] + Rl[:-1], Dl))
    lo = 1e-6 * (1.0 / rate) ** 2
    while count(lo) != 0:
        lo *= 0.01
    roots, stack = [], [(lo, hi, 0, nhi)]
    while stack:
        a, b, na, nb = stack.pop()
        if nb <= na:
            continue
        if b - a <= 4e-15 * b:
            assert nb - na == 1
            roots.append(0.5 * (a + b))
            continue
        m = math.sqrt(a * b) if b > 4 * a else 0.5 * (a + b)
        nm = min(max(count(m), na), nb)
        stack.append((a, m, na, nm))
        stack.append((m, b, nm, nb))
    return sorted(roots)


def starts_of(lam, lib=math):
    u, q, left, prev, st = lib.mpf(0) if lib is mp else 0.0, lib.mpf(1) if lib is mp else 1.0, 0, None, []
    for Ro, d in zip(Rl, Dl):
        Ro_, d_ = (mp.mpf(Ro), mp.mpf(d)) if lib is mp else (Ro, d)
        if prev is not None:
            q = q + (d_ - prev) * u / left
        k = lib.sqrt(lam / d_)
        st.append((u, q, k, d_, left, Ro_))
        L = Ro_ - left
        u, q = lib.cos(k * L) * u + lib.sin(k * L) * q / (d_ * k), -d_ * k * lib.sin(k * L) * u + lib.cos(k * L) * q
        left, prev = Ro_, d_
    return u, st


def weight(lam, lib=math):
    _, st = starts_of(lam, lib)
    N = Mo = I1 = 0
    for idx, (u, q, k, d, left, Ro) in enumerate(st):
        a0, b0, L = u, q / (d * k), Ro - left
        c2, s2 = lib.cos(2 * k * L), lib.sin(2 * k * L)
        nn = a0 * a0 * (L / 2 + s2 / (4 * k)) + b0 * b0 * (L / 2 - s2 / (4 * k)) + a0 * b0 * (1 - c2) / (2 * k)
        c1, s1 = lib.cos(k * L), lib.sin(k * L)
        iu = a0 * s1 / k + b0 * (1 - c1) / k
        isu = a0 * (L * s1 / k + (c1 - 1) / k**2) + b0 * (-L * c1 / k + s1 / k**2)
        mm = left * iu + isu
        N += nn
        Mo += mm
        if idx == 0:
            I1 = mm
    r1 = mp.mpf(Rl[0]) if lib is mp else Rl[0]
    return 3 * I1 * Mo / (r1**3 * N)


t_min = 2000.0
lam_max = 45.0 / t_min
lam = eigenvalues(lam_max)
w = np.array([weight(l) for l in lam])
lam = np.array(lam)
check("6.16 replica: 32 modes below Lambda_max = 45/t_min = 0.0225 s^-1 (main run: analytical_modes=32)",
      len(lam) == 32 and count(lam_max) == 32)
check("6.16 mode cap not reached (32 <= K_max = 20000), so Lambda_cov = Lambda_max", count(lam_max) <= 20000)
t_valid = 40.0 / lam_max
check("6.16 t_valid = 40/Lambda_cov = 1777.78 s < t_min = 2000 s (= (40/45) t_min)",
      abs(t_valid - 1777.7777777777778) < 1e-9 and t_valid < t_min, f"{t_valid:.6f}")
check("6.16 per-term factor of every omitted mode: e^{-40} = 4.25e-18 at t_valid, e^{-45} = 2.86e-20 at t_min",
      abs(math.exp(-40) / 4.248354e-18 - 1) < 1e-6 and abs(math.exp(-45) / 2.862519e-20 - 1) < 1e-6)
deficit = 1 - w.sum()
check("6.16 truncated series at t = 0 gives F(0) = 1 - sum w = -7.28e-6 (not 0): why t < t_valid is excluded",
      abs(deficit + 7.275592e-6) < 1e-11, f"{deficit:.6e}")
sabs = np.abs(w).sum()
check("6.17 sum |w_n| over the 32 modes is about 2 (O(1))", 1 < sabs < 3, f"{sabs:.6f}")

# tail check with the modes up to 0.2 s^-1 (VER completeness run: 95 modes, 1 - sum w = 5.401e-6)
lam2 = np.array(eigenvalues(0.2))
w2 = np.array([weight(l) for l in lam2])
check("6.16 95 modes up to 0.2 s^-1 with 1 - sum w = 5.401e-6 (verify prints 95 and 5.401e-6)",
      len(lam2) == 95 and abs(abs(1 - w2.sum()) - 5.401e-6) < 5e-10, f"{len(lam2)}, {1 - w2.sum():.4e}")
om = lam2 > lam_max
tail_tv = float(np.sum(np.abs(w2[om]) * np.exp(-lam2[om] * t_valid)))
check("6.16 omitted modes 33 to 95 contribute < e^{-40} sum|w| at t_valid (observed below 1e-18)",
      tail_tv < math.exp(-40) * np.abs(w2[om]).sum() and tail_tv < 1e-18, f"{tail_tv:.3e}")

# oracle: refine every root and weight at 40 digits
mp.mp.dps = 40
lam_o, w_o = [], []
for l in lam:
    a, b = mp.mpf(l) * (1 - mp.mpf("1e-9")), mp.mpf(l) * (1 + mp.mpf("1e-9"))
    ga = starts_of(a, mp)[0]
    assert ga * starts_of(b, mp)[0] < 0
    for _ in range(110):
        m = (a + b) / 2
        gm = starts_of(m, mp)[0]
        if (gm > 0) == (ga > 0):
            a, ga = m, gm
        else:
            b = m
    lam_o.append((a + b) / 2)
    w_o.append(weight((a + b) / 2, mp))
rel_lam = max(abs(float((mp.mpf(x) - y) / y)) for x, y in zip(lam, lam_o))
abs_w = max(abs(float(mp.mpf(x) - y)) for x, y in zip(w, w_o))
check("6.17 replica eigenvalues agree with the 40-digit oracle (relative error < 1e-12; audit Rust vs 50-digit: 3.7e-13)",
      rel_lam < 1e-12, f"{rel_lam:.2e}")
check("6.17 replica weights agree with the oracle (absolute error < 1e-12; audit Rust: 9.0e-14)", abs_w < 1e-12,
      f"{abs_w:.2e}")
Fo = lambda t: 1 - mp.fsum(wo * mp.exp(-lo * t) for wo, lo in zip(w_o, lam_o))
Fr = lambda t: 1 - float(np.sum(w * np.exp(-lam * t)))
check("6.17 oracle F(2e5 s) = 0.43255563909304 (audit 50-digit: 0.43255563909304174)",
      abs(float(Fo(2e5)) - 0.43255563909304174) < 1e-16, mp.nstr(Fo(2e5), 17))
check("6.17 oracle F(2e3 s) = 1.6e-15 > 0 (audit: 1.6e-15); 32-mode series is positive at t_min",
      abs(float(Fo(2e3)) / 1.6e-15 - 1) < 0.05, mp.nstr(Fo(2e3), 6))

# error propagation bound: |dF| <= sum |dw| e^{-L t} + max|dL/L| e^{-1} sum|w|
x = sp.symbols("x", positive=True)
check("6.17 max over x > 0 of x e^{-x} is e^{-1} (at x = 1)",
      sp.solve(sp.diff(x * sp.exp(-x), x), x) == [1] and sp.simplify((x * sp.exp(-x)).subs(x, 1) - sp.exp(-1)) == 0)
for t in [2e3, 2e4, 2e5]:
    dF = abs(Fr(t) - float(Fo(t)))
    dw = [float(mp.mpf(a_) - b_) for a_, b_ in zip(w, w_o)]
    dl = [float((mp.mpf(a_) - b_) / b_) for a_, b_ in zip(lam, lam_o)]
    bound = sum(abs(d_) * math.exp(-l_ * t) for d_, l_ in zip(dw, lam)) + max(map(abs, dl)) * math.exp(-1) * sabs
    check(f"6.17 replica error at t = {t:.0e} s is within the propagation bound and below eps_F = 3.997e-12",
          dF <= bound * (1 + 1e-9) + 4.5e-16 and dF < 3.997e-12, f"|dF| {dF:.2e}, bound {bound:.2e}")
print(f"INFO  replica F(2e3 s) = {Fr(2e3):.3e}; Rust (audit 01) -8.55e-14; oracle {mp.nstr(Fo(2e3), 3)}")
print(f"INFO  sum over modes of w_n at t = 2e3 s: {float(np.sum(w*np.exp(-lam*2e3))):.17f} (the 1 - (nearly 1) cancellation)")

print("ALL PASS" if all(ok_all) else "SOME FAIL")
sys.exit(0 if all(ok_all) else 1)
