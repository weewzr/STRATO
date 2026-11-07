"""Section 5 (Phase 2.2) numerical demonstration 3: staircase approximation of a ramp with
layer-specific activation energies, (P22.54)-(P22.57), and backward-Euler time-level choice
(P22.15)-(P22.18) when no shared alpha exists.

The spatial discretisation is fixed (independent FV, same scheme as v1.0 Code B, 20 cells per
layer), so every error reported here is a pure time/temperature-representation error.
Reference: the semi-discrete system with the smooth ramp T(t), integrated with Radau IIA
(rtol 1e-12) and cross-checked against a Richardson-extrapolated midpoint staircase.
Staircase solutions are propagated exactly on each constant-T interval (eigendecomposition),
which is what the eigenfunction-projection method does with the series.

All diffusion parameters are SYNTHETIC test values, not material data.
Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec5_staircase.py
The script writes no files.
"""
import math

import numpy as np
import sympy as sp
from scipy.integrate import solve_ivp
from scipy.linalg import eigh, solve_banded

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


# ---------------------------------------------------------------------------
# 0. Magnus terms behind the local error orders (P22.55)
# ---------------------------------------------------------------------------
h, s1, s2, s_ = sp.symbols("h s1 s2 s", positive=True)
A0 = sp.Matrix(2, 2, sp.symbols("p0:4"))
A1 = sp.Matrix(2, 2, sp.symbols("q0:4"))
A2 = sp.Matrix(2, 2, sp.symbols("w0:4"))
Aof = lambda x: A0 + x * A1 + x**2 * A2
comm = lambda X, Y: X * Y - Y * X
Omega2 = sp.Rational(1, 2) * (comm(Aof(s1), Aof(s2))).applyfunc(
    lambda e: sp.integrate(sp.integrate(e, (s2, 0, s1)), (s1, 0, h)))
lead = -(h**3 / 12) * comm(A0, A1)
diff2 = (Omega2 - lead).applyfunc(lambda e: sp.expand(e))
check("Magnus Omega_2 = -(h^3/12)[A0, A1] + O(h^4)",
      all(sp.Poly(e, h).monoms() == [] or min(m[0] for m in sp.Poly(e, h).monoms()) >= 4 for e in diff2 if e != 0))
Omega1 = Aof(s_).applyfunc(lambda e: sp.integrate(e, (s_, 0, h)))
mid_err = (Omega1 - h * Aof(h / 2)).applyfunc(sp.expand)
left_err = (Omega1 - h * Aof(0)).applyfunc(sp.expand)
check("midpoint: int_0^h A ds - h A(h/2) = h^3 A2/12 (O(h^3))", sp.simplify(mid_err - h**3 * A2 / 12) == sp.zeros(2, 2))
check("left end: int_0^h A ds - h A(0) = h^2 A1/2 + h^3 A2/3 (O(h^2))",
      sp.simplify(left_err - (h**2 * A1 / 2 + h**3 * A2 / 3)) == sp.zeros(2, 2))

# ---------------------------------------------------------------------------
# Problem set-up
# ---------------------------------------------------------------------------
RG = 6.02214076e23 * 1.380649e-23
RADII = np.cumsum([175.0, 103.5, 39.4, 35.3, 41.0]) * 1e-6
T_REF = 1873.15
D_REF = np.array([2e-12, 1e-10, 5e-13, 1e-13, 5e-13])             # synthetic
Q_LAYER = np.array([209e3, 50e3, 222e3, 514e3, 222e3])           # synthetic, layer specific
T_START, T_END, T_RAMP = 1873.15, 2073.15, 4 * 3600.0           # 1600 -> 1800 C at 50 K/h
T_HOLD_END = 6 * 3600.0


def T_of_t(t):
    return T_START + (T_END - T_START) * min(t, T_RAMP) / T_RAMP


def D_of_T(T):
    return D_REF * np.exp(-Q_LAYER / RG * (1.0 / T - 1.0 / T_REF))


NC = 20
edges, mat, inner = [0.0], [], 0.0
for i, Ro in enumerate(RADII):
    e = np.linspace(inner, Ro, NC + 1)[1:]
    e[-1] = Ro
    edges.extend(e)
    mat += [i] * NC
    inner = Ro
e = np.array(edges)
a_, b_ = e[:-1], e[1:]
V = 4 * np.pi / 3 * (b_**3 - a_**3)
xc = 0.75 * (b_**4 - a_**4) / (b_**3 - a_**3)
mat = np.array(mat)
face = b_[:-1]
Af = 4 * np.pi * face**2
dl, dr = face - xc[:-1], xc[1:] - face
R = RADII[-1]
N = len(V)
C0 = np.where(mat == 0, 1.0, 0.0)
M0 = V @ C0


def conductances(D_layer):
    Dc = D_layer[mat]
    return Af / (dl / Dc[:-1] + dr / Dc[1:]), 4 * np.pi * R**2 * Dc[-1] / (R - xc[-1])


def stiffness(D_layer):
    G, Gout = conductances(D_layer)
    d = np.zeros(N)
    d[:-1] += G
    d[1:] += G
    d[-1] += Gout
    return np.diag(d) - np.diag(G, 1) - np.diag(G, -1)


def propagate(C, D_layer, dt):
    w, Phi = eigh(stiffness(D_layer), np.diag(V))
    return Phi @ (np.exp(-w * dt) * (Phi.T @ (V * C)))


# ---------------------------------------------------------------------------
# Reference: smooth ramp, Radau IIA
# ---------------------------------------------------------------------------
def rhs(t, C):
    return -(stiffness(D_of_T(T_of_t(t))) @ C) / V


def jac(t, C):
    return -stiffness(D_of_T(T_of_t(t))) / V[:, None]


sol = solve_ivp(rhs, (0, T_RAMP), C0, method="Radau", rtol=1e-12, atol=1e-18, jac=jac, dense_output=False)
C_ramp_ref = sol.y[:, -1]
F_ref_ramp = 1 - V @ C_ramp_ref / M0
C_hold_ref = propagate(C_ramp_ref, D_of_T(T_END), T_HOLD_END - T_RAMP)
F_ref_hold = 1 - V @ C_hold_ref / M0
print(f"INFO  reference (Radau): F(4 h) = {F_ref_ramp:.12f}, F(6 h) = {F_ref_hold:.12f}")

# ---------------------------------------------------------------------------
# Staircase variants
# ---------------------------------------------------------------------------
xg, wg = np.polynomial.legendre.leggauss(8)


def staircase(K, variant):
    h_ = T_RAMP / K
    C = C0.copy()
    for k in range(K):
        t0 = k * h_
        if variant == "left":
            Dk = D_of_T(T_of_t(t0))
        elif variant == "mid":
            Dk = D_of_T(T_of_t(t0 + h_ / 2))
        else:   # per-layer interval average of D (P22.57)
            Dk = 0.5 * sum(w * D_of_T(T_of_t(t0 + h_ / 2 * (1 + x))) for x, w in zip(xg, wg))
        C = propagate(C, Dk, h_)
    F_ramp = 1 - V @ C / M0
    C = propagate(C, D_of_T(T_END), T_HOLD_END - T_RAMP)
    return F_ramp, 1 - V @ C / M0


Ks = [2, 4, 8, 16, 32, 64, 128, 256]
res = {v: np.array([staircase(K, v) for K in Ks]) for v in ("left", "mid", "avg")}

# Cross-check of the reference with a different stiff integrator (BDF). A 4th-order Magnus
# step was tried and rejected: with h ||A|| ~ 1e2-1e3 the commutator series is far outside
# its convergence region for this stiff system, so it is no reference here.
sol_b = solve_ivp(rhs, (0, T_RAMP), C0, method="BDF", rtol=1e-12, atol=1e-18, jac=jac)
F_bdf = 1 - V @ sol_b.y[:, -1] / M0
check("reference cross-check: Radau vs BDF at F(4 h)", abs(F_bdf - F_ref_ramp) < 1e-9,
      f"BDF {F_bdf:.12f}, diff {abs(F_bdf - F_ref_ramp):.1e}")

expected = {"left": 1.0, "mid": 2.0, "avg": 2.0}
for v in ("left", "mid", "avg"):
    err = np.abs(res[v] - np.array([F_ref_ramp, F_ref_hold]))
    orders = np.log2(err[:-1] / err[1:])
    print(f"INFO  staircase {v:4s}: K = " + ", ".join(str(k) for k in Ks))
    print("        err F(4 h) = " + ", ".join(f"{x:.2e}" for x in err[:, 0]))
    print("        err F(6 h) = " + ", ".join(f"{x:.2e}" for x in err[:, 1]))
    print("        order(4 h) = " + ", ".join(f"{x:.2f}" for x in orders[:, 0]))
    print("        order(6 h) = " + ", ".join(f"{x:.2f}" for x in orders[:, 1]))
    p = orders[-2:, :].mean()
    check(f"staircase '{v}' observed order {p:.2f} ~ {expected[v]:.0f} (P22.56)", abs(p - expected[v]) < 0.15)
    if v == "mid":
        err_mid = err
    if v == "avg":
        err_avg = err
print(f"INFO  error ratio mid/avg at K = 256: {err_mid[-1, 0] / err_avg[-1, 0]:.2f} (4 h), "
      f"{err_mid[-1, 1] / err_avg[-1, 1]:.2f} (6 h)")
K_needed = next((k for k, e in zip(Ks, np.abs(res['mid'][:, 1] - F_ref_hold)) if e < 1e-6), None)
print(f"INFO  smallest K with |error F(6 h)| < 1e-6, midpoint staircase: {K_needed}")

# ---------------------------------------------------------------------------
# Backward Euler with recomputed conductances, no shared alpha (P22.15)-(P22.18)
# ---------------------------------------------------------------------------
def be_run(dt, mode):
    n = int(round(T_HOLD_END / dt))
    C = C0.copy()
    out = {}
    for k in range(1, n + 1):
        t0, t1 = (k - 1) * dt, k * dt
        if mode == "end":
            Dk = D_of_T(T_of_t(t1))
        else:
            Dk = 0.5 * sum(w * D_of_T(T_of_t(t0 + dt / 2 * (1 + x))) for x, w in zip(xg, wg))
        G, Gout = conductances(Dk)
        d = V / dt
        d = d.copy()
        d[:-1] += G
        d[1:] += G
        d[-1] += Gout
        ab = np.zeros((3, N))
        ab[0, 1:] = -G
        ab[1] = d
        ab[2, :-1] = -G
        C = solve_banded((1, 1), ab, V / dt * C)
        if abs(t1 - T_RAMP) < 1e-6 or abs(t1 - T_HOLD_END) < 1e-6:
            out[round(t1)] = 1 - V @ C / M0
    return np.array([out[round(T_RAMP)], out[round(T_HOLD_END)]])


for mode in ("end", "avg"):
    errs = []
    for dt in (600.0, 300.0, 150.0, 75.0):
        errs.append(np.abs(be_run(dt, mode) - np.array([F_ref_ramp, F_ref_hold])))
    errs = np.array(errs)
    orders = np.log2(errs[:-1] / errs[1:])
    lab = "D(T^{n+1})" if mode == "end" else "step-averaged D"
    print(f"INFO  BE {lab:16s} errors F(6 h): " + ", ".join(f"{x:.2e}" for x in errs[:, 1])
          + " | orders " + ", ".join(f"{x:.3f}" for x in orders[:, 1]))
    check(f"BE with {lab}, layer-specific Q: first order", 0.9 < orders[-1, 1] < 1.1)

print(f"\nSUMMARY {sum(RESULTS)}/{len(RESULTS)} checks passed")
