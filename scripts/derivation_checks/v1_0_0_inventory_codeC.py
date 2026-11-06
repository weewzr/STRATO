"""STRATO v1.0.0 implementation inventory, Code C (src/analytical/sturm_liouville.rs).

Symbolic checks (SymPy) of the layer transfer, the interface jump, the Sturm phase count,
the weight formula and the Robin root equation; a numerical replica (closed-form layer
integrals instead of the code's Gauss-Legendre quadrature) that reproduces the v1.0 main-run
numbers for the synthetic five-layer case.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/v1_0_0_inventory_codeC.py
Writes nothing. Prints one PASS/FAIL line per check; exit code 1 on any FAIL.
"""

import math
import sys

import numpy as np
import sympy as sp

ok_all = []


def check(label, ok, detail=""):
    ok = bool(ok)
    ok_all.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {label}" + (f"  [{detail}]" if detail else ""), flush=True)


r, s, k, D, L, Lam, R, R1 = sp.symbols("r s k D L Lambda R R_1", positive=True)
u0, q0 = sp.symbols("u0 q0", real=True)
phi = sp.Function("phi")

# C1 u = r phi turns (1/r^2)(r^2 D phi')' + Lambda phi = 0 into D u'' + Lambda u = 0
u = sp.Function("u")
expr = (sp.diff(r**2 * D * sp.diff(u(r) / r, r), r) / r**2 + Lam * u(r) / r)
check("C1 (1/r^2)(r^2 D (u/r)')' + Lambda u/r = (D u'' + Lambda u)/r",
      sp.simplify(expr - (D * sp.diff(u(r), r, 2) + Lam * u(r)) / r) == 0)

# C2 transfer_layer: u(s) = cos(ks) u0 + sin(ks) q0/(D k), q = D u'
us = sp.cos(k * s) * u0 + sp.sin(k * s) * q0 / (D * k)
qs = D * sp.diff(us, s)
check("C2 transfer u(s) solves u'' + k^2 u = 0", sp.simplify(sp.diff(us, s, 2) + k**2 * us) == 0)
check("C2 transfer q(s) = -D k sin(ks) u0 + cos(ks) q0",
      sp.simplify(qs - (-D * k * sp.sin(k * s) * u0 + sp.cos(k * s) * q0)) == 0)
check("C2 transfer at s = 0 is the identity", us.subs(s, 0) == u0 and sp.simplify(qs.subs(s, 0) - q0) == 0)
T = sp.Matrix([[sp.cos(k * L), sp.sin(k * L) / (D * k)], [-D * k * sp.sin(k * L), sp.cos(k * L)]])
check("C2 transfer matrix has determinant 1", sp.simplify(T.det() - 1) == 0)

# C3 interface: phi' = (u' - u/r)/r; D phi continuous flux => q_out = q_in + (D_out - D_in) u / r
Din, Dout, uu, qin, rr = sp.symbols("D_in D_out u_I q_in r_I", positive=True)
up = sp.symbols("up")
check("C3 d/dr (u/r) = (u' - u/r)/r",
      sp.simplify(sp.diff(u(r) / r, r) - (sp.diff(u(r), r) - u(r) / r) / r) == 0)
qout = qin + (Dout - Din) * uu / rr
check("C3 interface_state keeps D(u' - u/r) = q - D u/r equal on both sides",
      sp.simplify((qin - Din * uu / rr) - (qout - Dout * uu / rr)) == 0)

# C4 Sturm phase: u(s) = a cos ks + b sin ks = A sin(ks + theta0), theta0 = atan2(a, b)
a_, b_ = sp.symbols("a b", real=True)
Aamp = sp.sqrt(a_**2 + b_**2)
th = sp.atan2(a_, b_)
check("C4 A sin(theta0) = a and A cos(theta0) = b with theta0 = atan2(a, b)",
      all(abs(float((Aamp * sp.sin(th) - a_).subs({a_: x, b_: y}))) < 1e-14
          and abs(float((Aamp * sp.cos(th) - b_).subs({a_: x, b_: y}))) < 1e-14
          for x, y in [(0.3, 0.4), (-0.7, 0.2), (0.5, -1.1), (-0.2, -0.9)]))


def zeros_formula(theta0, kl):
    return math.floor((theta0 + kl) / math.pi) - math.floor(theta0 / math.pi)


rng = np.random.default_rng(3)
good = True
for _ in range(2000):
    a0, b0, kk, LL = rng.normal(), rng.normal(), rng.uniform(0.1, 5), rng.uniform(0.1, 7)
    t0 = math.atan2(a0, b0)
    ss = np.linspace(0, LL, 200001)[1:]
    vals = a0 * np.cos(kk * ss) + b0 * np.sin(kk * ss)
    brute = int(np.sum(np.sign(vals[1:]) != np.sign(vals[:-1])))
    brute += 1 if np.sign(vals[0]) != np.sign(a0) and a0 != 0 else 0
    good &= brute == zeros_formula(t0, kk * LL)
check("C4 floor((theta0 + kL)/pi) - floor(theta0/pi) counts zeros in (0, L] (2000 random cases)", good)

# C5 weights, homogeneous sphere: u_n = sin(n pi r/R); closed forms used in the unit test
n = sp.symbols("n", integer=True, positive=True)
al = n * sp.pi / R
un = sp.sin(al * r)
I1 = sp.integrate(r * un, (r, 0, R1))
In = sp.integrate(r * un, (r, 0, R))
Nn = sp.integrate(un**2, (r, 0, R))
check("C5 I1 = (sin(a R1) - a R1 cos(a R1))/a^2", sp.simplify(I1 - (sp.sin(al * R1) - al * R1 * sp.cos(al * R1)) / al**2) == 0)
check("C5 I_n = R^2 (-1)^(n+1)/(n pi)", sp.simplify(In - R**2 * (-1) ** (n + 1) / (n * sp.pi)) == 0)
check("C5 N_n = R/2", sp.simplify(Nn - R / 2) == 0)
w_full = sp.simplify((3 * I1 * In / (R1**3 * Nn)).subs(R1, R))
check("C5 R1 = R gives w_n = 6/(n^2 pi^2) (Crank uniform sphere)", sp.simplify(w_full - 6 / (n**2 * sp.pi**2)) == 0)

# C5b weight formula from projection: a_n = c0 I1/N, M(t) = 4 pi sum a_n I_n e^{-Lt}, M0 = 4 pi c0 R1^3/3
c0 = sp.symbols("c0", positive=True)
I1s, Ins, Ns = sp.symbols("I1 I_n N_n", positive=True)
w = (4 * sp.pi * (c0 * I1s / Ns) * Ins) / (sp.Rational(4, 3) * sp.pi * c0 * R1**3)
check("C5 w_n = 3 I1 I_n/(R1^3 N_n) from projection with weight 1 for u",
      sp.simplify(w - 3 * I1s * Ins / (R1**3 * Ns)) == 0)

# C6 Robin: phi = sin(z r/R)/r, -D phi'(R) = h phi(R), Bi = hR/D  =>  z cot z = 1 - Bi
z, h = sp.symbols("z h", positive=True)
ph = sp.sin(z * r / R) / r
resid = (-D * sp.diff(ph, r) - h * ph).subs(r, R)
target = (sp.sin(z) / R**2) * D * ((1 - h * R / D) - z * sp.cot(z))
check("C6 -D phi'(R) - h phi(R) = (D sin z/R^2)[(1 - Bi) - z cot z]", sp.simplify(resid - target) == 0)

# C7 Gauss-Legendre 16 points: exact to degree 31, not 32
x, wts = np.polynomial.legendre.leggauss(16)
exact31 = all(abs(np.sum(wts * x**p) - (2 / (p + 1) if p % 2 == 0 else 0)) < 1e-14 for p in range(32))
err32 = abs(np.sum(wts * x**32) - 2 / 33)
check("C7 16-point Gauss-Legendre integrates degree <= 31 exactly, degree 32 not", exact31 and err32 > 1e-12, f"deg-32 error {err32:.2e}")

# ---------------- numerical replica of ModalRelease for the synthetic case ----------------
Rl = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4]
Dl = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13]


def shoot(lam, rescale):
    u, q, left, prev, starts = 0.0, 1.0, 0.0, None, []
    for Ro, d in zip(Rl, Dl):
        if prev is not None:
            q = q + (d - prev) * u / left
        kk = math.sqrt(lam / d)
        starts.append((u, q, kk, d, left, Ro))
        LL = Ro - left
        u, q = math.cos(kk * LL) * u + math.sin(kk * LL) * q / (d * kk), -d * kk * math.sin(kk * LL) * u + math.cos(kk * LL) * q
        if rescale:
            m = max(abs(u), abs(q / (d * kk)))
            u, q = u / m, q / m
        left, prev = Ro, d
    return u, starts


def count(lam):
    nz, u, q, left, prev = 0, 0.0, 1.0, 0.0, None
    for Ro, d in zip(Rl, Dl):
        if prev is not None:
            q = q + (d - prev) * u / left
        kk = math.sqrt(lam / d)
        t0 = math.atan2(u, q / (d * kk))
        nz += zeros_formula(t0, kk * (Ro - left))
        LL = Ro - left
        u, q = math.cos(kk * LL) * u + math.sin(kk * LL) * q / (d * kk), -d * kk * math.sin(kk * LL) * u + math.cos(kk * LL) * q
        m = max(abs(u), abs(q / (d * kk)))
        u, q = u / m, q / m
        left, prev = Ro, d
    return max(nz, 0)


lam_max = 45.0 / 2000.0  # first positive output time of the main run: 10 steps x 200 s
roots = []
grid = np.geomspace(1e-8, lam_max, 400001)
g = np.array([shoot(l_, True)[0] for l_ in grid])
for i in np.nonzero(np.sign(g[1:]) != np.sign(g[:-1]))[0]:
    lo, hi = grid[i], grid[i + 1]
    flo = shoot(lo, True)[0]
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        fm = shoot(mid, True)[0]
        if (fm > 0) == (flo > 0):
            lo, flo = mid, fm
        else:
            hi = mid
    roots.append(0.5 * (lo + hi))
check("C8 sign-change scan of g(Lambda) = u(R) finds 32 roots below 45/2000 s^-1 (main run: 32 modes)", len(roots) == 32, f"{len(roots)}")
check("C8 Sturm count at Lambda_max equals the number of sign-change roots", count(lam_max) == len(roots), f"{count(lam_max)}")
check("C8 Sturm count steps by one across each root",
      all(count(x_ * (1 - 1e-9)) == i and count(x_ * (1 + 1e-9)) == i + 1 for i, x_ in enumerate(roots)))
check("C8 lowest eigenvalue 3.273255671175805e-6 s^-1 (audit, 50-digit oracle agreement 3.7e-13)",
      abs(roots[0] / 3.273255671175805e-6 - 1) < 1e-11, f"{roots[0]:.15e}")


def layer_integrals(a0, b0, kk, left, LL):
    # closed forms of int u^2 ds and int (left + s) u ds, u = a0 cos ks + b0 sin ks, s in [0, L]
    c2, s2 = math.cos(2 * kk * LL), math.sin(2 * kk * LL)
    N = a0 * a0 * (LL / 2 + s2 / (4 * kk)) + b0 * b0 * (LL / 2 - s2 / (4 * kk)) + a0 * b0 * (1 - c2) / (2 * kk)
    c1, s1 = math.cos(kk * LL), math.sin(kk * LL)
    int_u = a0 * s1 / kk + b0 * (1 - c1) / kk
    int_su = a0 * (LL * s1 / kk + (c1 - 1) / kk**2) + b0 * (-LL * c1 / kk + s1 / kk**2)
    return N, left * int_u + int_su


ws, ratio, ratio_pre = [], math.inf, math.inf
for lam in roots:
    _, starts = shoot(lam, False)
    for j, (u_, q_, kk, d, left, Ro) in enumerate(starts[1:], start=1):
        sc = abs(q_) + abs(d * u_ / left)
        ratio = min(ratio, abs(q_ - d * u_ / left) / sc)
        d_in = Dl[j - 1]
        q_in = q_ - (d - d_in) * u_ / left
        ratio_pre = min(ratio_pre, abs(q_in - d_in * u_ / left) / (abs(q_in) + abs(d_in * u_ / left)))
    N = Mo = I1v = 0.0
    for idx, (u_, q_, kk, d, left, Ro) in enumerate(starts):
        nn, mm = layer_integrals(u_, q_ / (d * kk), kk, left, Ro - left)
        N += nn
        Mo += mm
        if idx == 0:
            I1v = mm
    ws.append(3 * I1v * Mo / (Rl[0] ** 3 * N))
ws = np.array(ws)
lam = np.array(roots)
check("C11 sum of the 32 weights = 1.000007275592 (audit: Rust 1.000007275592173)",
      abs(ws.sum() - 1.000007275592173) < 1e-11, f"{ws.sum():.15f}")
F = lambda t: 1 - float(np.sum(ws * np.exp(-lam * t)))
check("C12 F(2e5 s) = 0.43255563909304 (50-digit oracle 0.43255563909304174)",
      abs(F(2.0e5) - 0.43255563909304174) < 1e-12, f"{F(2.0e5):.14f}")
check("C13 min_flux_ratio over modes and interfaces = 5.504e-5 (main run)", abs(ratio / 5.504e-5 - 1) < 1e-3, f"{ratio:.4e}")
check("C13 accuracy floor 2.2e-16/ratio = 3.997e-12 (main run)", abs(2.2e-16 / ratio / 3.997e-12 - 1) < 1e-3, f"{2.2e-16/ratio:.4e}")
check("C14 validity time 40/Lambda_covered = 1.777778e3 s with Lambda_covered = 45/2000",
      abs(40 / lam_max - 1.777778e3) < 1e-3)
check("C13 flux ratio is side dependent: inner-side (pre-jump) value differs from the code's outer-side value",
      abs(ratio_pre / ratio - 1) > 0.1, f"outer {ratio:.4e}, inner {ratio_pre:.4e}, floors {2.2e-16/ratio:.2e} vs {2.2e-16/ratio_pre:.2e}")
print(f"INFO  F(2e3 s) from 32-mode replica = {F(2.0e3):.3e} (Rust prints -8.55e-14; oracle 1.6e-15)")

print("ALL PASS" if all(ok_all) else "SOME FAIL")
sys.exit(0 if all(ok_all) else 1)
