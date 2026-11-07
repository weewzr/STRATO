"""Truncation-error and observed-order checks for Section 2 (Phase 1, items 15 to 20).

Part A, space (steady manufactured case). The Code B spatial operator K (conductances
G_f, outer conductance G_R, zero centre conductance) is applied to the steady problem
    0 = r^-2 d/dr(r^2 D_i dc/dr) + S,   c(R) = 0,   uniform S,
on the synthetic five-layer geometry (inputs/synthetic/synthetic_five_layer.toml). The
exact solution is piecewise quadratic with c'' jumping at every interface, so it probes
the interface treatment. The source term is a verification device only: Code B v1.0 has
S = 0, the operator is identical. Expected (Section 2.7): local flux error O(h) at the
four interface faces and at r = R, O(h^2) at interior faces, yet global cell-average and
inventory errors O(h^2) (supraconvergence), and the discrete outer flux equal to the
total source to round-off (conservation).

Part B, time. Backward Euler on the 200-cell synthetic mesh compared with the
semi-discrete solution on the same mesh (float64 generalised eigen-decomposition), so the
spatial error cancels and the observed order is the pure time order. The float64 eigen
route is exact in time but only accurate to about 1e-9 in F here: the absolute error of
each computed eigenvalue is about u*mu_max (mu_max = 256 s^-1), which perturbs
exp(-mu_min t) at t = 2e5 s by about u*mu_max*t = 6e-9 relative. This is checked below
against an independent order-4 Richardson extrapolation of the subtraction-free BE
solutions (independent checker's correction; the 40-digit value is 0.43255016318742).

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec2_fv_truncation_order.py
Writes nothing. Prints PASS/FAIL lines and exits non-zero on any FAIL.
"""

import math
import sys

import numpy as np

PI = math.pi
results = []


def check(label, ok, detail=""):
    ok = bool(ok)
    results.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {label}" + (f"  [{detail}]" if detail else ""), flush=True)


RADII = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4]  # m, synthetic
DIFFS = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13]  # m^2 s^-1, synthetic


def build_mesh(cells):
    """Same operations as finite_volume.rs build_mesh (lines 50-99)."""
    lo_, hi_, cent, vol, dif, mat = [], [], [], [], [], []
    inner = 0.0
    for i, (R_i, D_i) in enumerate(zip(RADII, DIFFS)):
        width = (R_i - inner) / cells
        for k in range(cells):
            a = inner + k * width
            b = R_i if k + 1 == cells else inner + (k + 1) * width
            lo_.append(a)
            hi_.append(b)
            cent.append(0.75 * b if a == 0.0 else 0.75 * (b**4 - a**4) / (b**3 - a**3))
            vol.append(4.0 * PI / 3.0 * (b**3 - a**3))
            dif.append(D_i)
            mat.append(i)
        inner = R_i
    n = len(vol)
    g = np.array([4.0 * PI * hi_[j] ** 2 / ((hi_[j] - cent[j]) / dif[j] + (cent[j + 1] - hi_[j]) / dif[j + 1])
                  for j in range(n - 1)])
    R = RADII[-1]
    gR = 4.0 * PI * R * R * dif[-1] / (R - cent[-1])
    return dict(lo=np.array(lo_), hi=np.array(hi_), cent=np.array(cent), vol=np.array(vol), mat=np.array(mat), g=g, gR=gR)


def solve_sf(v, gw, ge, rhs):
    """Subtraction-free elimination (finite_volume.rs lines 132-160)."""
    n = len(v)
    den, dp = np.zeros(n), np.zeros(n)
    e_prev = 0.0
    for i in range(n):
        e = v[0] if i == 0 else v[i] + gw[i] * e_prev / (e_prev + gw[i])
        den[i] = e + ge[i]
        dp[i] = (rhs[i] + (gw[i] * dp[i - 1] if i else 0.0)) / den[i]
        e_prev = e
    x = np.zeros(n)
    x[-1] = dp[-1]
    for i in range(n - 2, -1, -1):
        x[i] = dp[i] + ge[i] / den[i] * x[i + 1]
    return x


# ---------------------------------------------------------------------------
# Part A: steady manufactured case, spatial orders
# ---------------------------------------------------------------------------
S = 1.0  # mol m^-3 s^-1 (verification source)
# c_out[i] = exact c at the outer radius of layer i (0-based). c_out[4] = c(R) = 0 and, going inward,
# c_out[i] = c_out[i+1] + S/(6 D_{i+1}) (R_{i+1}^2 - R_i^2): continuity of c with D_i c' = -S r/3.
c_out = [0.0] * 5
for i in range(3, -1, -1):
    c_out[i] = c_out[i + 1] + S / (6.0 * DIFFS[i + 1]) * (RADII[i + 1] ** 2 - RADII[i] ** 2)


def exact_cell_average(m):
    a, b, i = m["lo"], m["hi"], m["mat"]
    r2 = 0.6 * (b**4 + b**3 * a + b**2 * a**2 + b * a**3 + a**4) / (b**2 + a * b + a**2)  # <r^2>, cancellation-free
    Ri = np.array(RADII)[i]
    Di = np.array(DIFFS)[i]
    return np.array(c_out)[i] + S / (6.0 * Di) * (Ri**2 - r2)


levels = [10, 20, 40, 80, 160, 320]
rows = []
for nc in levels:
    m = build_mesh(nc)
    n = len(m["vol"])
    gw = np.concatenate(([0.0], m["g"]))
    ge = np.concatenate((m["g"], [m["gR"]]))
    C = solve_sf(np.zeros(n), gw, ge, S * m["vol"])  # steady: v = 0, rhs = S_j V_j
    Cx = exact_cell_average(m)
    flow_exact = S * 4.0 * PI / 3.0 * m["hi"] ** 3  # exact outward flow through each face (mol s^-1)
    tau = np.concatenate((m["g"] * (Cx[:-1] - Cx[1:]), [m["gR"] * Cx[-1]])) - flow_exact
    is_iface = np.zeros(n, dtype=bool)
    is_iface[[nc * k - 1 for k in range(1, 5)]] = True
    is_outer = np.zeros(n, dtype=bool)
    is_outer[-1] = True
    interior = ~(is_iface | is_outer)
    # Flux errors are scaled by the total outward flow S V_total (a fixed scale). Scaling by the
    # local flow would give h^2/(6 r_f^2) at interior faces, which is O(1) at the first faces
    # next to the centre because the local flow there vanishes like r_f^3.
    rel = np.abs(tau) / flow_exact[-1]
    rows.append(dict(
        n=n,
        nc=nc,
        e_c=float(np.max(np.abs(C - Cx)) / np.max(Cx)),
        e_L1=float(np.dot(m["vol"], np.abs(C - Cx)) / np.dot(m["vol"], Cx)),
        j_max=int(np.argmax(np.abs(C - Cx))),
        e_M=float(abs(np.dot(m["vol"], C) - np.dot(m["vol"], Cx)) / np.dot(m["vol"], Cx)),
        t_if=float(np.max(rel[is_iface])),
        t_in=float(np.max(rel[interior])),
        t_R=float(rel[-1]),
        cons=float(abs(m["gR"] * C[-1] - S * m["vol"].sum()) / (S * m["vol"].sum())),
    ))

print("INFO  cells | max-norm err (cell) | L1 err | inventory err | interface tau | interior tau | outer tau "
      "(tau scaled by S V_total)")
for r in rows:
    print(f"INFO  {r['n']:5d} | {r['e_c']:.3e} ({r['j_max']}) | {r['e_L1']:.3e} | {r['e_M']:.3e} | {r['t_if']:.3e} | "
          f"{r['t_in']:.3e} | {r['t_R']:.3e}")


def orders(key):
    return [math.log2(rows[k][key] / rows[k + 1][key]) for k in range(len(rows) - 1)]


fmt = lambda xs: ", ".join(f"{x:.2f}" for x in xs)
o_c, o_M, o_if, o_in, o_R = orders("e_c"), orders("e_M"), orders("t_if"), orders("t_in"), orders("t_R")
o_L1 = orders("e_L1")
check("(P1.79) volume-weighted L1 cell-average error is second order across interfaces (last order in [1.9, 2.1])",
      1.9 <= o_L1[-1] <= 2.1, fmt(o_L1))
# Max-norm error sits in the centre cell from 40 cells/region on; model e = K ln(N_reg)/N_reg^2.
fine = [r for r in rows if r["j_max"] == 0]
kfit = [r["e_c"] * r["nc"] ** 2 / math.log(r["nc"]) for r in fine]
check("(P1.79) max-norm error is at the centre cell and behaves like h^2 ln(1/h) (K spread < 3 %)",
      len(fine) >= 3 and (max(kfit) - min(kfit)) / min(kfit) < 0.03,
      f"orders {fmt(o_c)}; K = " + ", ".join(f"{k:.4e}" for k in kfit))
# Heuristic of Section 2.7: e_centre ~ sum_f tau_f/G_f ~ S h^2/(18 D_1) * sum_k 1/k ~ S R_1^2 ln(N)/(18 D_1 N^2)
K_pred = S * RADII[0] ** 2 / (18.0 * DIFFS[0] * float(np.max(exact_cell_average(build_mesh(levels[-1])))))
check("(P1.79) centre-cell constant K matches the face-sum prediction S R_1^2/(18 D_1 max Cbar) within 3 %",
      abs(kfit[-1] / K_pred - 1) < 0.03, f"predicted {K_pred:.4e}, observed {kfit[-1]:.4e}")
check("(P1.79) inventory error is second order (last order in [1.9, 2.1])", 1.9 <= o_M[-1] <= 2.1, fmt(o_M))
check("(P1.78) interface-face flux error is first order (last order in [0.9, 1.1])", 0.9 <= o_if[-1] <= 1.1, fmt(o_if))
check("(P1.78) outer-face flux error is first order (last order in [0.9, 1.1])", 0.9 <= o_R[-1] <= 1.1, fmt(o_R))
check("(P1.78) interior-face flux error is second order (last order in [1.9, 2.1])", 1.9 <= o_in[-1] <= 2.1, fmt(o_in))
check("(P1.75) discrete outer flux G_R C_N equals total source to round-off (all meshes < 1e-12)",
      max(r["cons"] for r in rows) < 1e-12, f"max {max(r['cons'] for r in rows):.2e}")

# ---------------------------------------------------------------------------
# Part B: Backward Euler order against the exact semi-discrete solution (same mesh)
# ---------------------------------------------------------------------------
m = build_mesh(40)
n = len(m["vol"])
V = m["vol"]
K = np.zeros((n, n))
gw = np.concatenate(([0.0], m["g"]))
ge = np.concatenate((m["g"], [m["gR"]]))
for j in range(n):
    K[j, j] = gw[j] + ge[j]
    if j > 0:
        K[j, j - 1] = -gw[j]
    if j + 1 < n:
        K[j, j + 1] = -ge[j]
C0 = np.where(m["mat"] == 0, 1.0, 0.0)
M0 = float(np.dot(V, C0))
s = 1.0 / np.sqrt(V)
lam, Q = np.linalg.eigh(s[:, None] * K * s[None, :])  # V^{-1/2} K V^{-1/2} = Q diag(lam) Q^T
T_END = 2.0e5


def semi_discrete_F(t):
    """F(t) = 1 - M(t)/M0 with C(t) = V^{-1/2} Q exp(-lam t) Q^T V^{1/2} C0 (exact in time, float64 eigenpairs)."""
    Ct = s * (Q @ (np.exp(-lam * t) * (Q.T @ (C0 / s))))
    return 1.0 - float(np.dot(V, Ct)) / M0


F_ref = semi_discrete_F(T_END)
errs = []
for dt in [800.0, 400.0, 200.0, 100.0, 50.0]:
    steps = math.ceil(T_END / dt)
    dt = T_END / steps
    c = C0.copy()
    released = 0.0
    for _ in range(steps):
        c = solve_sf(V / dt, gw, ge, V / dt * c)
        released += dt * m["gR"] * c[-1]
    errs.append(abs(released / M0 - F_ref))
o_t = [math.log2(errs[k] / errs[k + 1]) for k in range(len(errs) - 1)]
print(f"INFO  semi-discrete F(2e5 s), 200 cells = {F_ref:.13f}; BE errors " + ", ".join(f"{e:.3e}" for e in errs))
check("(P1.65) Backward Euler observed time order in [0.95, 1.05] against the same-mesh reference",
      all(0.95 <= o <= 1.05 for o in o_t), fmt(o_t))
audit = [4.857e-4, 2.430e-4, 1.216e-4, 6.079e-5, 3.040e-5]  # audit 01, Section 1.10 item 5 (Rust, time-converged FV reference)
check("(P1.65) errors agree with the audit's same-mesh Rust errors to 1 %",
      all(abs(e / a - 1) < 0.01 for e, a in zip(errs, audit)), "audit 4.857e-4 ... 3.040e-5")
# Independent reference: BE with the subtraction-free solver at dt = 200, 100, 50, 25 s, F from the
# fresh inventory, then Richardson to order 4 (BE error has a full power series in dt).
def be_inventory_F(dt):
    c = C0.copy()
    for _ in range(int(round(T_END / dt))):
        c = solve_sf(V / dt, gw, ge, V / dt * c)
    return 1.0 - float(np.dot(V, c)) / M0


Fl = [be_inventory_F(d) for d in (200.0, 100.0, 50.0, 25.0)]
r1 = [2 * Fl[k + 1] - Fl[k] for k in range(3)]
r2 = [(4 * r1[k + 1] - r1[k]) / 3 for k in range(2)]
F_rich = (8 * r2[1] - r2[0]) / 7
check("(P1.79) float64 eigen reference agrees with the BE-Richardson reference only to ~1e-9 (u mu_max t limit)",
      abs(F_ref - F_rich) < 2e-9, f"eigen {F_ref:.13f}, Richardson {F_rich:.13f}, diff {F_ref - F_rich:.2e}")
check("(P1.79) BE-Richardson reference is within 1e-11 of the audit's time-extrapolated Rust value",
      abs(F_rich - 0.4325501631858728) < 1e-11, f"diff {F_rich - 0.4325501631858728:.2e}")
check("(P1.79) semi-discrete F(2e5 s) on 200 cells is within 1e-8 of the audit's time-extrapolated Rust value",
      abs(F_ref - 0.4325501631858728) < 1e-8, f"{F_ref:.13f} vs 0.4325501631858728, diff {F_ref - 0.4325501631858728:.2e}")
# The temporal error of BE is O(dt); its leading constant follows from the semi-discrete solution:
# error(dt)/dt should tend to a constant.
check("(P1.65) BE error / dt tends to a constant (last two ratios agree to 1 %)",
      abs((errs[-1] / 50.0) / (errs[-2] / 100.0) - 1) < 0.01, f"err/dt = {errs[-2] / 100.0:.4e}, {errs[-1] / 50.0:.4e} s^-1")

print(f"SUMMARY {sum(results)}/{len(results)} checks passed")
print("ALL PASS" if all(results) else "SOME CHECKS FAILED")
sys.exit(0 if all(results) else 1)
