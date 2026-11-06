"""STRATO v1.0.0, Full Mathematical Derivation, Part VI: properties of Code B.

Checks the conservation, M-matrix, positivity, maximum-principle, symmetry, positive
definiteness, Backward Euler stability and forward Euler bound statements of Part VI.
Symbolic parts use SymPy; numerical parts re-implement the Rust arithmetic of
src/numerical/finite_volume.rs (v1.0.0) in plain Python/NumPy on the synthetic five-layer
input (inputs/synthetic/synthetic_five_layer.toml) and on the unit-test case of
FV lines 375 to 389.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/v1_0_0_p6_properties.py
Writes nothing. One PASS/FAIL line per check; exit code 1 on any FAIL.
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


# ---------------------------------------------------------------- symbolic checks
N = 5
V = sp.symbols(f"V1:{N+1}", positive=True)
G = sp.symbols(f"G1:{N}", positive=True)          # G[j-1] = G_{j+1/2}, j = 1..N-1
GR, dt = sp.symbols("G_R Delta_t", positive=True)
Cn = sp.symbols(f"Cn1:{N+1}")
Cp = sp.symbols(f"Cp1:{N+1}")
gw = [0] + [G[j - 1] for j in range(1, N)]
ge = [G[j] for j in range(N - 1)] + [GR]
rows = []
for j in range(N):
    lhs = (V[j] / dt + gw[j] + ge[j]) * Cp[j]
    if j > 0:
        lhs -= gw[j] * Cp[j - 1]
    if j < N - 1:
        lhs -= ge[j] * Cp[j + 1]
    rows.append(lhs - V[j] / dt * Cn[j])
Mn = sum(V[j] * Cn[j] for j in range(N))
Mp = sum(V[j] * Cp[j] for j in range(N))
check("6.1 dt * (sum of the N = 5 BE rows) = M^{n+1} + dt G_R C_N^{n+1} - M^n (telescoping)",
      sp.expand(dt * sum(rows) - (Mp + dt * GR * Cp[-1] - Mn)) == 0)

A = sp.zeros(N, N)
for j in range(N):
    A[j, j] = gw[j] + ge[j]
    if j > 0:
        A[j, j - 1] = -gw[j]
    if j < N - 1:
        A[j, j + 1] = -ge[j]
check("6.1 column sums of A are (0, ..., 0, G_R)", list(sp.ones(1, N) * A) == [0] * (N - 1) + [GR])
check("6.4 A is symmetric because g^W_{j+1} = g^E_j = G_{j+1/2}", A == A.T)
x = sp.symbols(f"x1:{N+1}")
xv = sp.Matrix(x)
qf = sum(G[j] * (x[j] - x[j + 1]) ** 2 for j in range(N - 1)) + GR * x[-1] ** 2
check("6.4 x^T A x = sum_f G_f (x_j - x_{j+1})^2 + G_R x_N^2 (so A is positive definite)",
      sp.expand((xv.T * A * xv)[0] - qf) == 0)

# scalar test equation
z = sp.symbols("z")
xr, yi = sp.symbols("x y", real=True)
R_be = 1 / (1 - z)
R_fe = 1 + z
check("6.4 BE applied to y' = -mu y gives y^{n+1} = y^n/(1 + mu dt) = R(z) y^n, R(z) = 1/(1 - z), z = -mu dt",
      sp.simplify(R_be.subs(z, -sp.Symbol("mu") * dt) - 1 / (1 + sp.Symbol("mu") * dt)) == 0)
expr = sp.expand((1 - xr) ** 2 + yi ** 2 - 1)
check("6.4 |1 - z|^2 - 1 = x^2 - 2x + y^2 >= 0 for Re z = x <= 0 (A-stability of BE)",
      expr == xr ** 2 - 2 * xr + yi ** 2)
check("6.4 BE is L-stable: R(z) -> 0 as z -> -infinity", sp.limit(R_be, z, -sp.oo) == 0)
check("6.4 FE: |1 + z| <= 1 on the real axis only for -2 <= z <= 0",
      sp.solve_univariate_inequality(sp.Abs(1 + xr) <= 1, xr, relational=False) == sp.Interval(-2, 0))

# forward Euler positivity: diagonal coefficient
Vj, gwj, gej = sp.symbols("V_j g_W g_E", positive=True)
coef = 1 - dt * (gwj + gej) / Vj
check("6.6 FE update C_j^{n+1} = (1 - dt (g^W+g^E)/V_j) C_j^n + ...: coefficient >= 0 iff dt <= V_j/(g^W+g^E)",
      sp.solve(sp.Eq(coef, 0), dt) == [Vj / (gwj + gej)])

# ---------------------------------------------------------------- Rust replica
Rl = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4]
Dl = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13]


def build_mesh(radii, diffs, cells_per):
    cells = []
    inner = 0.0
    for mat, (Ro, d, nc) in enumerate(zip(radii, diffs, cells_per)):
        width = (Ro - inner) / nc
        for k in range(nc):
            left = inner + k * width
            right = Ro if k + 1 == nc else inner + (k + 1) * width
            cen = 0.75 * right if left == 0.0 else 0.75 * (right**4 - left**4) / (right**3 - left**3)
            vol = 4.0 * math.pi / 3.0 * (right * right * right - left * left * left)
            cells.append((left, right, cen, vol, mat, d))
        inner = Ro
    cond = []
    for L, Rr in zip(cells[:-1], cells[1:]):
        face = L[1]
        area = 4.0 * math.pi * face * face
        cond.append(area / ((face - L[2]) / L[5] + (Rr[2] - face) / Rr[5]))
    last = cells[-1]
    Rout = radii[-1]
    gR = 4.0 * math.pi * Rout * Rout * last[5] / (Rout - last[2])
    return cells, cond, gR


def coeffs(cells, cond, gR):
    n = len(cells)
    gwv = [0.0] + cond
    gev = cond + [gR]
    return gwv, gev


def solve_sf(v, gwv, gev, rhs):
    n = len(v)
    den, dp, e_prev, es = [0.0] * n, [0.0] * n, 0.0, [0.0] * n
    for i in range(n):
        e = v[0] if i == 0 else v[i] + gwv[i] * e_prev / (e_prev + gwv[i])
        es[i] = e
        den[i] = e + gev[i]
        carry = 0.0 if i == 0 else gwv[i] * dp[i - 1]
        dp[i] = (rhs[i] + carry) / den[i]
        e_prev = e
    x = [0.0] * n
    x[-1] = dp[-1]
    for i in range(n - 2, -1, -1):
        x[i] = dp[i] + gev[i] / den[i] * x[i + 1]
    return x, es, den


def solve_textbook(v, gwv, gev, rhs):
    n = len(v)
    cp, dp = [0.0] * n, [0.0] * n
    for i in range(n):
        b = v[i] + gwv[i] + gev[i]
        m = b if i == 0 else b - gwv[i] * cp[i - 1]      # b_i - a_i c'_{i-1}, a_i = -gw, c'_{i-1} = -ge/m
        cp[i] = gev[i] / m
        dp[i] = (rhs[i] + (0.0 if i == 0 else gwv[i] * dp[i - 1])) / m
    x = [0.0] * n
    x[-1] = dp[-1]
    for i in range(n - 2, -1, -1):
        x[i] = dp[i] + cp[i] * x[i + 1]
    return x


def run(cells, cond, gR, c0_kernel, dt_in, t_end, solver=solve_sf, track=False):
    gwv, gev = coeffs(cells, cond, gR)
    c = [c0_kernel if cl[4] == 0 else 0.0 for cl in cells]
    vol = [cl[3] for cl in cells]
    inv = lambda cc: sum(V_ * x_ for V_, x_ in zip(vol, cc))
    initial = inv(c)
    steps = math.ceil(t_end / dt_in)
    dtu = t_end / steps
    released, comp = 0.0, 0.0
    worst_res, minc, maxp_ok, vnorm_ok, inv_ok = 0.0, math.inf, True, True, True
    vn_prev = math.sqrt(sum(V_ * x_ * x_ for V_, x_ in zip(vol, c)))
    inv_prev = initial
    for _ in range(steps):
        v = [V_ / dtu for V_ in vol]
        rhs = [vv * x_ for vv, x_ in zip(v, c)]
        nxt = solver(v, gwv, gev, rhs)
        if solver is solve_sf:
            nxt = nxt[0]
        incr = dtu * (gR * nxt[-1])
        y = incr - comp
        t = released + y
        comp = (t - released) - y
        released = t
        if track:
            maxp_ok &= max(nxt) <= max(c) * (1 + 1e-14)
            vn = math.sqrt(sum(V_ * x_ * x_ for V_, x_ in zip(vol, nxt)))
            vnorm_ok &= vn <= vn_prev * (1 + 1e-14)
            vn_prev = vn
        c = nxt
        minc = min(minc, min(c))
        im = inv(c)
        inv_ok &= im <= inv_prev * (1 + 1e-15)
        inv_prev = im
        worst_res = max(worst_res, abs(im + released - initial) / initial)
    return dict(F=released / initial, worst=worst_res, minc=minc, maxp=maxp_ok, vnorm=vnorm_ok,
                inv=inv_ok, dt=dtu, steps=steps, M0=initial)


cells, cond, gR = build_mesh(Rl, Dl, [40] * 5)
n = len(cells)
gwv, gev = coeffs(cells, cond, gR)
vol = np.array([cl[3] for cl in cells])
check("6.4 interface symmetry g^W_{j+1} = g^E_j holds bit for bit in the replica (199 faces)",
      all(gwv[j + 1] == gev[j] for j in range(n - 1)))

# explicit bound and L2 bound
fe = min(vol[i] / (gwv[i] + gev[i]) for i in range(n))
check("6.6 Delta t_FE = min_j V_j/(g^W_j + g^E_j) = 7.812107958751e-3 s (main run prints 7.812107958751e-3)",
      abs(fe / 7.812107958751e-3 - 1) < 1e-11, f"{fe:.12e}")
Am = np.zeros((n, n))
for i in range(n):
    Am[i, i] = gwv[i] + gev[i]
    if i > 0:
        Am[i, i - 1] = -gwv[i]
    if i < n - 1:
        Am[i, i + 1] = -gev[i]
Vmh = np.diag(vol ** -0.5)
S = Vmh @ Am @ Vmh
mu = np.linalg.eigvalsh(S)
check("6.4 S = V^{-1/2} A V^{-1/2} is symmetric positive definite (all 200 eigenvalues > 0)",
      np.allclose(S, S.T, rtol=1e-14, atol=0) and mu.min() > 0, f"mu_min {mu.min():.4e}, mu_max {mu.max():.4e} s^-1")
check("6.7 forward Euler L2 limit 2/mu_max = 7.824293e-3 s > Delta t_FE (inventory B15)",
      abs(2 / mu.max() / 7.824293e-3 - 1) < 1e-6 and 2 / mu.max() > fe, f"{2/mu.max():.7e}")
dt = 200.0
amp = 1 / (1 + dt * mu)
check("6.5 BE amplification factors 1/(1 + dt mu_k) lie in (0, 1) at dt = 200 s",
      amp.min() > 0 and amp.max() < 1, f"range [{amp.min():.3e}, {amp.max():.6f}]")
check("6.4 dt/Delta t_FE = 2.560e4 for dt = 200 s", abs(dt / fe / 2.5601e4 - 1) < 1e-4, f"{dt/fe:.4e}")
w = [(Rl[l] - (Rl[l - 1] if l else 0.0)) / 40 for l in range(5)]
fo = [Dl[l] * dt / w[l] ** 2 for l in range(5)]
check("6.13 layer Fourier numbers D dt/w^2 = 8, 1.28e4, 102.4, 2.56, 89.6 at dt = 200 s (max in the buffer)",
      np.allclose(fo, [8.0, 12800.0, 102.4, 2.56, 89.6], rtol=1e-9), ", ".join(f"{f_:.4g}" for f_ in fo))

# M-matrix: B = V/dt + A
B = np.diag(vol / dt) + Am
off = B - np.diag(np.diag(B))
check("6.2 B = V/dt + A is a Z-matrix with positive diagonal", (off <= 0).all() and (np.diag(B) > 0).all())
margin = np.diag(B) - np.abs(off).sum(axis=1)
expected = vol / dt + np.r_[np.zeros(n - 1), gR]
check("6.2 row margin a_jj - sum |a_jk| = v_j (j < N), v_N + G_R (j = N): strictly diagonally dominant",
      np.allclose(margin, expected, rtol=1e-9) and (margin > 0).all(), f"min margin {margin.min():.3e} m^3/s")
cols_ok, resid = True, 0.0
for k in range(n):
    ek = [0.0] * n
    ek[k] = 1.0
    xk, _, _ = solve_sf(list(vol / dt), gwv, gev, ek)
    cols_ok &= min(xk) >= 0.0
    resid = max(resid, np.abs(B @ np.array(xk) - np.array(ek)).max())
check("6.2 every column of B^{-1} computed by the subtraction-free sweep is >= 0 (exactly) and solves B x = e_k",
      cols_ok and resid < 1e-12, f"max |B x - e_k| = {resid:.2e}")
Binv = np.linalg.inv(B)
check("6.2 B^{-1} from LAPACK is >= 0 up to rounding (min entry >= -1e-12 max entry)",
      Binv.min() >= -1e-12 * Binv.max(), f"min {Binv.min():.2e}, max {Binv.max():.2e}")

# pivots
_, es, den = solve_sf(list(vol / dt), gwv, gev, [1.0] * n)
vv = vol / dt
check("6.4b pivots: v_j <= e_j <= v_j + g^W_j and d_j = e_j + g^E_j > 0 for all j (no pivoting needed)",
      all(vv[j] <= es[j] <= vv[j] + gwv[j] for j in range(n)) and min(den) > 0)
check("6.4b back-substitution multipliers g^E_j/d_j lie in (0, 1)",
      all(0 < gev[j] / den[j] < 1 for j in range(n)), f"max {max(gev[j]/den[j] for j in range(n)):.6f}")

# full run
res = run(cells, cond, gR, 1.0, 200.0, 2.0e5, track=True)
check("6.3 replica of the main run: F(2e5 s) = 0.43242860687845 (main run prints 4.324286068785e-1)",
      abs(res["F"] - 0.43242860687845) < 1e-12, f"{res['F']:.14f}")
check("6.4 replica: every computed concentration is >= 0 exactly (1000 steps, dt = 200 s)", res["minc"] >= 0.0,
      f"min {res['minc']:.3e}")
check("6.3 replica: max_j C^{n+1}_j <= max_j C^n_j at every step (maximum principle)", res["maxp"])
check("6.5 replica: volume-weighted norm ||C^{n+1}||_V <= ||C^n||_V at every step", res["vnorm"])
check("6.1 replica: inventory M^n is non-increasing at every step", res["inv"])
check("6.1 replica: worst relative balance residual |eps^n|/M^0 < 1e-12 (Rust: 2.806e-13 on output rows)",
      res["worst"] < 1e-12, f"{res['worst']:.2e} over all 1000 steps")
check("6.1 M^0 = 3.351032163829113e-11 mol", abs(res["M0"] / 3.351032163829113e-11 - 1) < 1e-14)

# unit-test case of FV 375-389: 5 x 50 cells, D = 1e-10, dt = 1e4 s, t_end = 1e6 s
cells2, cond2, gR2 = build_mesh([(i + 1) * 1e-4 for i in range(5)], [1e-10] * 5, [50] * 5)
fo2 = 1e-10 * 1e4 / (2e-6) ** 2
r_sf = run(cells2, cond2, gR2, 1.0, 1.0e4, 1.0e6, solver=solve_sf)
r_tb = run(cells2, cond2, gR2, 1.0, 1.0e4, 1.0e6, solver=solve_textbook)
check("6.13 unit-test case has Fourier number D dt/w^2 = 2.5e5", abs(fo2 - 2.5e5) < 1e-6)
check("6.13 balance drift at Fo = 2.5e5: both pivots stay far below 1e-12 (no 'six digits lost')",
      r_sf["worst"] < 1e-12 and r_tb["worst"] < 1e-12,
      f"subtraction-free {r_sf['worst']:.2e}, textbook {r_tb['worst']:.2e}")
check("6.13 subtraction-free min C >= 0 at Fo = 2.5e5 (textbook value shown for comparison)",
      r_sf["minc"] >= 0.0, f"min C: subtraction-free {r_sf['minc']:.3e}, textbook {r_tb['minc']:.3e}")

print("ALL PASS" if all(ok_all) else "SOME FAIL")
sys.exit(0 if all(ok_all) else 1)
