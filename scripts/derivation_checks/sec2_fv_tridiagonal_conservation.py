"""Numerical checks for Section 2 (Phase 1 items 16 to 20): a float64 re-implementation
of Code B (src/numerical/finite_volume.rs) written operation by operation, used to check

  1. the mesh, explicit bound, inventory and final release of the synthetic five-layer case
     against the Rust run recorded in audit note 01 (Section 1.7);
  2. that the subtraction-free elimination equals a dense solve and a 50-digit solve on
     random five-layer meshes, and that every computed concentration is nonnegative;
  3. conservation to round-off (balance residual) over many Backward Euler steps;
  4. the cancellation loss of the textbook Thomas pivot at large Fourier number, against
     the subtraction-free form (the Code B unit-test case with D dt/dr^2 = 2.5e5);
  5. the explicit-Euler positivity bound against the L2 stability limit 2/mu_max.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec2_fv_tridiagonal_conservation.py
Writes nothing. Prints PASS/FAIL lines and exits non-zero on any FAIL.
"""

import math
import sys

import mpmath as mp
import numpy as np

PI = math.pi
U = 2.0**-53  # unit roundoff of IEEE double
results = []


def check(label, ok, detail=""):
    ok = bool(ok)
    results.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {label}" + (f"  [{detail}]" if detail else ""), flush=True)


# ---------------------------------------------------------------------------
# Code B, line by line (finite_volume.rs lines 38-99, 132-182, 184-252)
# ---------------------------------------------------------------------------
def shell_volume(a, b):
    return 4.0 * PI / 3.0 * (b**3 - a**3)


def volume_centroid(a, b):
    return 0.75 * b if a == 0.0 else 0.75 * (b**4 - a**4) / (b**3 - a**3)


def build_mesh(radii, diffs, cells):
    """Faces uniform in r inside each region; the last face of a region is snapped to R_i."""
    left_, right_, cent, vol, dif = [], [], [], [], []
    inner = 0.0
    for R_i, D_i, n_i in zip(radii, diffs, cells):
        width = (R_i - inner) / n_i
        for k in range(n_i):
            lo = inner + k * width
            hi = R_i if k + 1 == n_i else inner + (k + 1) * width
            left_.append(lo)
            right_.append(hi)
            cent.append(volume_centroid(lo, hi))
            vol.append(shell_volume(lo, hi))
            dif.append(D_i)
        inner = R_i
    g = []
    for j in range(len(cent) - 1):
        face = right_[j]
        area = 4.0 * PI * face * face
        dl = face - cent[j]
        dr = cent[j + 1] - face
        g.append(area / (dl / dif[j] + dr / dif[j + 1]))
    R = radii[-1]
    gR = 4.0 * PI * R * R * dif[-1] / (R - cent[-1])
    return dict(left=left_, right=right_, cent=cent, vol=vol, D=dif, g=g, gR=gR, R=R)


def coefficients(mesh, dt):
    n = len(mesh["vol"])
    gw = [0.0] + mesh["g"]
    ge = mesh["g"] + [mesh["gR"]]
    v = [mesh["vol"][j] / dt for j in range(n)]
    return v, gw, ge


def solve_subtraction_free(v, gw, ge, rhs):
    """solve_diffusion_system: e_j = v_j + gw_j e_{j-1}/(e_{j-1}+gw_j), d_j = e_j + ge_j."""
    n = len(v)
    den, dp = [0.0] * n, [0.0] * n
    e_prev = 0.0
    for i in range(n):
        e = v[0] if i == 0 else v[i] + gw[i] * e_prev / (e_prev + gw[i])
        den[i] = e + ge[i]
        carry = 0.0 if i == 0 else gw[i] * dp[i - 1]
        dp[i] = (rhs[i] + carry) / den[i]
        e_prev = e
    x = [0.0] * n
    x[n - 1] = dp[n - 1]
    for i in range(n - 2, -1, -1):
        x[i] = dp[i] + ge[i] / den[i] * x[i + 1]
    return x


def solve_textbook(v, gw, ge, rhs):
    """Textbook Thomas: pivot b_j - a_j c'_{j-1} with b = v+gw+ge, a = -gw, c = -ge."""
    n = len(v)
    cp, dp = [0.0] * n, [0.0] * n
    for i in range(n):
        b = v[i] + gw[i] + ge[i]
        den = b if i == 0 else b - (-gw[i]) * cp[i - 1]
        cp[i] = -ge[i] / den
        dp[i] = rhs[i] / den if i == 0 else (rhs[i] - (-gw[i]) * dp[i - 1]) / den
    x = [0.0] * n
    x[n - 1] = dp[n - 1]
    for i in range(n - 2, -1, -1):
        x[i] = dp[i] - cp[i] * x[i + 1]
    return x


def solve_mp(v, gw, ge, rhs, dps=50):
    """Reference: subtraction-free recurrences in 50-digit arithmetic (inputs taken as exact)."""
    with mp.workdps(dps):
        V = [mp.mpf(t) for t in v]
        W = [mp.mpf(t) for t in gw]
        E = [mp.mpf(t) for t in ge]
        B = [mp.mpf(t) for t in rhs]
        n = len(V)
        den, dp = [0] * n, [0] * n
        e_prev = mp.mpf(0)
        for i in range(n):
            e = V[0] if i == 0 else V[i] + W[i] * e_prev / (e_prev + W[i])
            den[i] = e + E[i]
            dp[i] = (B[i] + (W[i] * dp[i - 1] if i else 0)) / den[i]
            e_prev = e
        x = [0] * n
        x[n - 1] = dp[n - 1]
        for i in range(n - 2, -1, -1):
            x[i] = dp[i] + E[i] / den[i] * x[i + 1]
        return x


def dense_matrix(v, gw, ge):
    n = len(v)
    A = np.zeros((n, n))
    for i in range(n):
        A[i, i] = v[i] + gw[i] + ge[i]
        if i > 0:
            A[i, i - 1] = -gw[i]
        if i + 1 < n:
            A[i, i + 1] = -ge[i]
    return A


def run(mesh, c_kernel, n_kernel, dt_s, end_s, solver):
    """fv::run: uniform steps, Kahan-compensated release, residual = M + M_rel - M0."""
    n = len(mesh["vol"])
    c = [c_kernel if j < n_kernel else 0.0 for j in range(n)]
    inv = lambda cc: sum(V * x for V, x in zip(mesh["vol"], cc))
    M0 = inv(c)
    steps = math.ceil(end_s / dt_s)
    dt = end_s / steps
    v, gw, ge = coefficients(mesh, dt)
    released, comp, worst, cmin = 0.0, 0.0, 0.0, min(c)
    for _ in range(steps):
        c = solver(v, gw, ge, [v[j] * c[j] for j in range(n)])
        inc = dt * (mesh["gR"] * c[-1])
        y = inc - comp
        t = released + y
        comp = (t - released) - y
        released = t
        worst = max(worst, abs(inv(c) + released - M0) / M0)
        cmin = min(cmin, min(c))
    return dict(M0=M0, M=inv(c), F=released / M0, worst=worst, cmin=cmin, steps=steps, dt=dt, c=c)


def explicit_dt_bound(mesh):
    n = len(mesh["vol"])
    gw = [0.0] + mesh["g"]
    ge = mesh["g"] + [mesh["gR"]]
    return min(mesh["vol"][j] / (gw[j] + ge[j]) for j in range(n))


# ---------------------------------------------------------------------------
# 1. Synthetic five-layer case against the recorded Rust output (audit 01, Section 1.7)
# ---------------------------------------------------------------------------
radii = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4]
diffs = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13]
mesh = build_mesh(radii, diffs, [40] * 5)
rust_bound, rust_F, rust_M = 7.812107958751e-3, 4.324286068785e-1, 1.901949993620e-11
rust_M0 = 3.351032163829113e-11
check("(P1.67) explicit_dt_bound reproduces Rust 7.812107958751e-3 s",
      abs(explicit_dt_bound(mesh) / rust_bound - 1) < 1e-12, f"python {explicit_dt_bound(mesh):.12e}")
check("(P1.46) faces sit on all four interfaces and R", all(any(abs(f - R_i) < 1e-18 for f in mesh["right"]) for R_i in radii))
check("(P1.52) every half distance positive (centroid inside its cell)",
      all(mesh["left"][j] < mesh["cent"][j] < mesh["right"][j] for j in range(200)))
syn = run(mesh, 1.0, 40, 2.0e2, 2.0e5, solve_subtraction_free)
check("(P1.76) M0 = c0 (4/3) pi R1^3 matches Rust 3.351032163829113e-11 mol",
      abs(syn["M0"] / rust_M0 - 1) < 1e-14, f"python {syn['M0']:.15e}")
check("(P1.76) final F reproduces Rust 4.324286068785e-1 (1000 steps, dt = 200 s)",
      abs(syn["F"] - rust_F) < 5e-13, f"python {syn['F']:.13e}")
check("(P1.76) final inventory reproduces Rust 1.901949993620e-11 mol",
      abs(syn["M"] / rust_M - 1) < 1e-9, f"python {syn['M']:.12e}")
check("(P1.77) synthetic case: max |B^n|/M0 < 1e-12 over all steps", syn["worst"] < 1e-12, f"{syn['worst']:.3e}")
check("(P1.72)-(P1.73) synthetic case: every computed C_j^n >= 0", syn["cmin"] >= 0.0, f"min C = {syn['cmin']:.3e}")

# ---------------------------------------------------------------------------
# 2. Random five-layer meshes: elimination vs dense vs 50-digit, positivity, symmetry
# ---------------------------------------------------------------------------
rng = np.random.default_rng(20261106)
worst_sf, worst_dense, worst_col, worst_pos, n_trials = 0.0, 0.0, 0.0, 0.0, 25
fo_range = [math.inf, 0.0]
for trial in range(n_trials):
    R1 = rng.uniform(1.0e-4, 3.0e-4)
    radii_r = list(R1 + np.concatenate(([0.0], np.cumsum(rng.uniform(5e-6, 1e-4, 4)))))
    diffs_r = list(10.0 ** rng.uniform(-15.0, -9.0, 5))
    cells_r = [int(k) for k in rng.integers(3, 61, 5)]
    m = build_mesh(radii_r, diffs_r, cells_r)
    dt = 10.0 ** rng.uniform(-2.0, 6.0)
    v, gw, ge = coefficients(m, dt)
    n = len(v)
    fo = [(gw[j] + ge[j]) / v[j] for j in range(n)]  # cell Fourier-type number (G_W+G_E) dt / V
    fo_range = [min(fo_range[0], min(fo)), max(fo_range[1], max(fo))]
    rhs = list(rng.uniform(0.0, 1.0, n) * np.array(v))  # rhs = v_j C_j^n with C^n >= 0
    x = solve_subtraction_free(v, gw, ge, rhs)
    xr = solve_mp(v, gw, ge, rhs)
    worst_sf = max(worst_sf, max(abs(mp.mpf(x[j]) - xr[j]) / abs(xr[j]) for j in range(n)))
    A = dense_matrix(v, gw, ge)
    xd = np.linalg.solve(A, np.array(rhs))
    worst_dense = max(worst_dense, float(np.max(np.abs(xd - np.array(x))) / np.max(np.abs(x))))
    col = A.sum(axis=0)
    col_target = np.array(v)
    col_target[-1] += m["gR"]
    worst_col = max(worst_col, float(np.max(np.abs(col - col_target) / np.abs(A).sum(axis=0))))
    worst_pos = min(worst_pos, min(x))
    if trial == 0:
        check("(P1.69) A symmetric (random mesh)", np.array_equal(A, A.T))
        # Columns of A^{-1} by the subtraction-free solver are entrywise nonnegative (M-matrix)
        inv_min = min(min(solve_subtraction_free(v, gw, ge, [1.0 if i == k else 0.0 for i in range(n)])) for k in range(n))
        check("(P1.69) A^{-1} >= 0 entrywise (inverse-positive M-matrix)", inv_min >= 0.0, f"min entry {inv_min:.3e}")
check(f"(P1.71)-(P1.74) subtraction-free vs 50-digit: componentwise rel. error < 50 N u ({n_trials} random meshes)",
      worst_sf < 50 * 300 * U, f"max {float(worst_sf):.3e}; cell Fourier numbers {fo_range[0]:.1e} to {fo_range[1]:.1e}")
check("(P1.71)-(P1.73) subtraction-free equals dense LU solve (max-norm rel. diff < 1e-10)", worst_dense < 1e-10,
      f"max {worst_dense:.3e}")
check("(P1.69) floating column sums equal v_j (+G_R) to round-off", worst_col < 4 * U, f"max {worst_col:.3e}")
check("(P1.72)-(P1.73) computed solution nonnegative for nonnegative rhs (all trials)", worst_pos >= 0.0)

# Conservation over many steps on one random mesh with a large time step
m = build_mesh([2.5e-4, 3.4e-4, 3.8e-4, 4.15e-4, 4.55e-4], [3e-12, 2e-9, 5e-13, 1e-15, 4e-13], [37, 23, 11, 19, 7])
r_rand = run(m, 2.0, 37, 5.0e3, 5.0e6, solve_subtraction_free)
check("(P1.77) random 5-layer mesh, 1000 steps: max |B^n|/M0 at round-off level (< 1e-12)",
      r_rand["worst"] < 1e-12, f"max {r_rand['worst']:.3e}, F = {r_rand['F']:.6f}, min C = {r_rand['cmin']:.2e}")

# ---------------------------------------------------------------------------
# 3. Textbook Thomas at large Fourier number (Code B test conservation_holds_at_very_large_fourier_number)
# ---------------------------------------------------------------------------
mt = build_mesh([1e-4, 2e-4, 3e-4, 4e-4, 5e-4], [1e-10] * 5, [50] * 5)
v, gw, ge = coefficients(mt, 1.0e4)
c0 = [1.0 if j < 50 else 0.0 for j in range(250)]
rhs = [v[j] * c0[j] for j in range(250)]
ref = solve_mp(v, gw, ge, rhs)
rel = lambda xx: max(abs(mp.mpf(xx[j]) - ref[j]) / ref[j] for j in range(250))
e_sf, e_tb = float(rel(solve_subtraction_free(v, gw, ge, rhs))), float(rel(solve_textbook(v, gw, ge, rhs)))
fo_max = max((gw[j] + ge[j]) / v[j] for j in range(250))
check("(P1.74) one step at D dt/dr^2 = 2.5e5: subtraction-free componentwise error stays O(N u)",
      e_sf < 50 * 250 * U, f"subtraction-free {e_sf:.2e}, textbook {e_tb:.2e}, max (G_W+G_E)dt/V = {fo_max:.2e}")
check("(P1.70) textbook pivot is less accurate than subtraction-free at this Fourier number", e_tb > e_sf)
r_sf = run(mt, 1.0, 50, 1.0e4, 1.0e6, solve_subtraction_free)
r_tb = run(mt, 1.0, 50, 1.0e4, 1.0e6, solve_textbook)
check("(P1.77) 100 steps at Fo 2.5e5: subtraction-free max |B|/M0 < 1e-12 (Code B test threshold)",
      r_sf["worst"] < 1e-12, f"subtraction-free {r_sf['worst']:.3e}, textbook {r_tb['worst']:.3e}")
print(f"INFO  textbook Thomas balance drift on this case: {r_tb['worst']:.3e} (Phase 1 doc claims about 1e-9)")

# Fourier-number sweep: one step plus 20 steps of balance, D dt/dr^2 = 2.5e1 ... 2.5e11
sf_err, tb_err = [], []
for dt_k in [1e0, 1e2, 1e4, 1e6, 1e8, 1e10]:
    v, gw, ge = coefficients(mt, dt_k)
    rhs = [v[j] * c0[j] for j in range(250)]
    ref = solve_mp(v, gw, ge, rhs)
    sf_err.append(float(rel(solve_subtraction_free(v, gw, ge, rhs))))
    tb_err.append(float(rel(solve_textbook(v, gw, ge, rhs))))
    b_sf = run(mt, 1.0, 50, dt_k, 20 * dt_k, solve_subtraction_free)["worst"]
    b_tb = run(mt, 1.0, 50, dt_k, 20 * dt_k, solve_textbook)["worst"]
    print(f"INFO  Fo = {1e-10 * dt_k / (2e-6) ** 2:.1e}: componentwise error subtraction-free {sf_err[-1]:.2e}, "
          f"textbook {tb_err[-1]:.2e}; 20-step max |B|/M0 subtraction-free {b_sf:.2e}, textbook {b_tb:.2e}")
check("(P1.74) subtraction-free componentwise error is flat in Fo (all < 50 N u, max/min < 10)",
      max(sf_err) < 50 * 250 * U and max(sf_err) / min(sf_err) < 10, f"range {min(sf_err):.2e} to {max(sf_err):.2e}")
check("(P1.70) textbook componentwise error grows with Fo (largest Fo error > 10x smallest)",
      tb_err[-1] > 10 * tb_err[0], f"range {tb_err[0]:.2e} to {tb_err[-1]:.2e}")

# ---------------------------------------------------------------------------
# 4. Explicit positivity bound versus L2 stability limit (reporting only)
# ---------------------------------------------------------------------------
v1, gw1, ge1 = coefficients(mesh, 1.0)  # dt = 1 so v = V
K = dense_matrix([0.0] * 200, gw1, ge1)
Vh = 1.0 / np.sqrt(np.array(mesh["vol"]))
mu_max = float(np.max(np.linalg.eigvalsh(Vh[:, None] * K * Vh[None, :])))
dt_pos = explicit_dt_bound(mesh)
check("(P1.67) positivity bound <= L2 stability limit 2/mu_max <= 2 x positivity bound",
      dt_pos <= 2.0 / mu_max <= 2.0 * dt_pos * (1 + 1e-12), f"dt_pos {dt_pos:.4e} s, 2/mu_max {2.0 / mu_max:.4e} s")

print(f"SUMMARY {sum(results)}/{len(results)} checks passed")
print("ALL PASS" if all(results) else "SOME CHECKS FAILED")
sys.exit(0 if all(results) else 1)
