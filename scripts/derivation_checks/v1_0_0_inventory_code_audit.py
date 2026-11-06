"""STRATO v1.0.0 implementation inventory: symbolic and numerical checks.

Checks every formula listed in the v1.0.0 implementation inventory against an
independent derivation (SymPy) or an independent float64 re-implementation
(NumPy / plain Python). Prints one PASS or FAIL line per check. Writes nothing.

Run:
  C:/Y3S1/Nuclear/STRATO/.venv/Scripts/python.exe -I -X utf8 v1_0_0_inventory_code_audit.py
"""
import math

import numpy as np
import sympy as sp

FAILS = []


def check(tag, ok, info=""):
    print(("PASS" if ok else "FAIL") + f" [{tag}] {info}")
    if not ok:
        FAILS.append(tag)


# ---------------------------------------------------------------- Code B, symbolic
r, a, b = sp.symbols("r a b", positive=True)
V = sp.integrate(4 * sp.pi * r**2, (r, a, b))
check("B01 shell volume (4 pi/3)(b^3-a^3) = int 4 pi r^2 dr",
      sp.simplify(V - sp.Rational(4, 3) * sp.pi * (b**3 - a**3)) == 0)

rc = sp.integrate(r * 4 * sp.pi * r**2, (r, a, b)) / V
check("B02 volume centroid (3/4)(b^4-a^4)/(b^3-a^3)",
      sp.simplify(rc - sp.Rational(3, 4) * (b**4 - a**4) / (b**3 - a**3)) == 0)
check("B02 centroid of the centre cell (a=0) is 3b/4",
      sp.simplify(sp.limit(rc, a, 0) - sp.Rational(3, 4) * b) == 0)

CP, CE, CI, dP, dE, DP, DE, A = sp.symbols("C_P C_E C_I delta_P delta_E D_P D_E A", positive=True)
CIsol = sp.solve(sp.Eq(DP * (CP - CI) / dP, DE * (CI - CE) / dE), CI)[0]
flow = A * DP * (CP - CIsol) / dP
G = A / (dP / DP + dE / DE)
check("B03 series-resistance conductance A/(dP/DP + dE/DE)",
      sp.simplify(flow - G * (CP - CE)) == 0)
check("B03 equal D reduces to A D/(dP+dE)",
      sp.simplify(G.subs(DE, DP) - A * DP / (dP + dE)) == 0)

R, rN, DN, CN = sp.symbols("R r_N D_N C_N", positive=True)
check("B04 outer conductance 4 pi R^2 D_N/(R - r_N) from half-cell drop to c=0",
      sp.simplify(4 * sp.pi * R**2 * DN * (CN - 0) / (R - rN)
                  - (4 * sp.pi * R**2 * DN / (R - rN)) * CN) == 0)

# B07: subtraction-free pivot equals the textbook Thomas pivot when gw_i = ge_{i-1}
v_i, gw_i, ge_i, e_p = sp.symbols("v_i gw_i ge_i e_prev", positive=True)
den_prev = e_p + gw_i  # den_{i-1} = e_{i-1} + ge_{i-1}, ge_{i-1} = gw_i
textbook = (v_i + gw_i + ge_i) - (-gw_i) * (-gw_i / den_prev)  # b_i - a_i * c'_{i-1}
subfree = v_i + gw_i * e_p / (e_p + gw_i) + ge_i
check("B07 pivot: b_i - a_i c'_{i-1} == e_i + ge_i, e_i = v_i + gw_i e_{i-1}/(e_{i-1}+gw_i)",
      sp.simplify(textbook - subfree) == 0)

# ---------------------------------------------------------------- Code B, numerical
rng = np.random.default_rng(20261106)


def solve_subfree(v, gw, ge, rhs):
    n = len(v)
    den = [0.0] * n
    dp = [0.0] * n
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
    n = len(v)
    cp = [0.0] * n
    dp = [0.0] * n
    for i in range(n):
        bi = v[i] + gw[i] + ge[i]
        ai = -gw[i]
        ci = -ge[i]
        piv = bi if i == 0 else bi - ai * cp[i - 1]
        cp[i] = ci / piv
        dp[i] = (rhs[i] if i == 0 else rhs[i] - ai * dp[i - 1]) / piv
    x = [0.0] * n
    x[n - 1] = dp[n - 1]
    for i in range(n - 2, -1, -1):
        x[i] = dp[i] - cp[i] * x[i + 1]
    return x


n = 9
g_int = rng.uniform(0.1, 3.0, n - 1)
gw = np.concatenate([[0.0], g_int])
ge = np.concatenate([g_int, [rng.uniform(0.1, 3.0)]])
v = rng.uniform(0.05, 2.0, n)
old = rng.uniform(0.0, 1.0, n)
Amat = np.diag(v + gw + ge) - np.diag(g_int, -1) - np.diag(g_int, 1)
x_ref = np.linalg.solve(Amat, v * old)
x_sf = np.array(solve_subfree(list(v), list(gw), list(ge), list(v * old)))
check("B07 subtraction-free elimination equals dense solve (random 9x9)",
      np.max(np.abs(x_sf - x_ref) / np.abs(x_ref)) < 1e-13,
      f"max rel diff {np.max(np.abs(x_sf - x_ref) / np.abs(x_ref)):.2e}")
check("B06 BE matrix is a nonsingular M-matrix: inverse entrywise >= 0",
      np.all(np.linalg.inv(Amat) >= -1e-15) and np.all(np.diag(Amat) > np.abs(Amat).sum(1) - np.diag(Amat)))
# Column sums of A minus diag(v): every column sums to zero except the last (outer drain)
colsum = Amat.sum(0) - v
check("B10 column sums of (A - V/dt) vanish except the outer column = G_out (telescoping balance)",
      np.allclose(colsum[:-1], 0.0, atol=1e-14) and abs(colsum[-1] - ge[-1]) < 1e-14,
      f"last col {colsum[-1]:.6f} vs G_out {ge[-1]:.6f}")

# Synthetic five-layer mesh exactly as build_mesh does it
regions = [(2.0e-4, 1.0e-12, 40), (3.0e-4, 4.0e-10, 40), (3.5e-4, 8.0e-13, 40),
           (4.0e-4, 2.0e-14, 40), (4.5e-4, 7.0e-13, 40)]


def build(regions):
    cells = []
    inner = 0.0
    for mat, (ro, d, nc) in enumerate(regions):
        width = (ro - inner) / nc
        for k in range(nc):
            left = inner + k * width
            right = ro if k + 1 == nc else inner + (k + 1) * width
            cen = 0.75 * right if left == 0.0 else 0.75 * (right**4 - left**4) / (right**3 - left**3)
            vol = 4.0 * math.pi / 3.0 * (right**3 - left**3)
            cells.append((left, right, cen, vol, mat, d))
        inner = ro
    cond = []
    for L, Rc in zip(cells[:-1], cells[1:]):
        face = L[1]
        area = 4.0 * math.pi * face * face
        cond.append(area / ((face - L[2]) / L[5] + (Rc[2] - face) / Rc[5]))
    Rout = regions[-1][0]
    gout = 4.0 * math.pi * Rout * Rout * cells[-1][5] / (Rout - cells[-1][2])
    return cells, cond, gout


cells, cond, gout = build(regions)
N = len(cells)
GW = [0.0] + cond
GE = cond + [gout]
vols = [c[3] for c in cells]
bound = min(vols[i] / (GW[i] + GE[i]) for i in range(N))
check("B15 explicit_dt_bound = min V_i/(gw_i+ge_i) reproduces golden 7.812107958751e-3 s",
      abs(bound / 7.812107958751e-3 - 1) < 1e-11, f"python {bound:.12e}")

# L2 (spectral) forward-Euler limit for comparison: 2 / max eig of V^-1 A0
A0 = np.diag(np.array(GW) + np.array(GE)) - np.diag(cond, -1) - np.diag(cond, 1)
Vh = np.diag(1.0 / np.sqrt(vols))
mu = np.linalg.eigvalsh(Vh @ A0 @ Vh)
check("B15 positivity bound <= spectral FE stability bound 2/mu_max (bound is sufficient, not necessary)",
      bound <= 2.0 / mu.max(), f"positivity {bound:.6e} s, spectral {2.0 / mu.max():.6e} s")


def plain_sum(values):
    # Left-to-right float64 sum, as Rust's Iterator::sum. Python >= 3.12 sum() is
    # compensated for floats and would hide the round-off the code actually has.
    total = 0.0
    for x in values:
        total += x
    return total


def run(cells, cond, gout, c0, dt_in, t_end, solver):
    N = len(cells)
    GW = [0.0] + cond
    GE = cond + [gout]
    vols = [c[3] for c in cells]
    c = [c0 if cl[4] == 0 else 0.0 for cl in cells]
    m0 = plain_sum(V_ * x for V_, x in zip(vols, c))
    steps = math.ceil(t_end / dt_in)
    dt = t_end / steps
    rel, comp = 0.0, 0.0
    worst = 0.0
    for _ in range(steps):
        vv = [V_ / dt for V_ in vols]
        rhs = [vv[i] * c[i] for i in range(N)]
        c = solver(vv, GW, GE, rhs)
        inc = dt * (gout * c[-1])
        y = inc - comp
        t = rel + y
        comp = (t - rel) - y
        rel = t
        inv = plain_sum(V_ * x for V_, x in zip(vols, c))
        last = abs(inv + rel - m0) / m0
        worst = max(worst, last)
    return rel / m0, worst, min(c), steps, dt, last


F, worst, cmin, steps, dt, last = run(cells, cond, gout, 1.0, 2.0e2, 2.0e5, solve_subfree)
check("B11-B14 re-implemented run reproduces golden final F = 4.324286068785e-1",
      abs(F - 4.324286068785e-1) < 5e-13, f"F = {F:.12e}, steps {steps}, dt {dt}")
check("B13 relative balance residual <= 1e-12 over the run (golden final 2.806e-13)",
      worst < 1e-12, f"worst {worst:.3e}, final {last:.3e}")
check("B06 positivity: min final concentration >= 0", cmin >= 0.0, f"min c {cmin:.6e}")

# Textbook Thomas against subtraction-free at the unit-test Fourier number 2.5e5
reg_t = [((i + 1) * 1e-4, 1e-10, 50) for i in range(5)]
cells_t, cond_t, gout_t = build(reg_t)
_, worst_sf, _, _, _, _ = run(cells_t, cond_t, gout_t, 1.0, 1.0e4, 1.0e6, solve_subfree)
_, worst_tb, _, _, _, _ = run(cells_t, cond_t, gout_t, 1.0, 1.0e4, 1.0e6, solve_textbook)
check("B07 textbook Thomas drift at Fo=2.5e5 is far below the documented 'about 1e-9' (claim NOT reproduced)",
      worst_tb < 1e-11,
      f"textbook worst rel residual {worst_tb:.2e}; subtraction-free {worst_sf:.2e}")
check("B07 unit test threshold 1e-12 would NOT distinguish the two solvers here",
      worst_tb < 1e-12, f"textbook {worst_tb:.2e} < 1e-12")

# ---------------------------------------------------------------- Code A
h, a1, a2, a3, F0 = sp.symbols("h a1 a2 a3 F0")
Fh = lambda s: F0 + a1 * s + a2 * s**2 + a3 * s**3
rich = sp.expand((8 * Fh(h / 4) - 6 * Fh(h / 2) + Fh(h)) / 3)
check("A02 Richardson (8F(h/4) - 6F(h/2) + F(h))/3 cancels O(h) and O(h^2)",
      sp.simplify(rich.coeff(a1)) == 0 and sp.simplify(rich.coeff(a2)) == 0,
      f"remainder {sp.factor(rich - F0)}")
cost_factor = sum(sp.Rational(1) / sp.Rational(m) for m in ("1/4", "1/2", "1"))
check("A05 study_cost spatial multiplier: true cell-step factor of the 3 extrapolation runs is 7, code uses 1.75",
      cost_factor == 7, f"true factor {cost_factor}, code 1.75 -> underestimate by {cost_factor / sp.Rational(7, 4)}")
check("A03 observed order log2(e_k/e_k+1) reproduces golden spatial 2.000 (2.190e-5 -> 5.476e-6)",
      abs(math.log2(2.190e-5 / 5.476e-6) - 2.0) < 1e-3, f"{math.log2(2.190e-5 / 5.476e-6):.4f}")

# ---------------------------------------------------------------- Code C, symbolic
s, k, D, u0, q0, Rr, D1, D2 = sp.symbols("s k D u0 q0 R_i D_1 D_2", positive=True)
us = u0 * sp.cos(k * s) + q0 / (D * k) * sp.sin(k * s)
check("C04 transfer: u(s) solves u'' + k^2 u = 0 with u(0)=u0, D u'(0)=q0",
      sp.simplify(sp.diff(us, s, 2) + k**2 * us) == 0
      and sp.simplify(us.subs(s, 0) - u0) == 0
      and sp.simplify(D * sp.diff(us, s).subs(s, 0) - q0) == 0)
qs = sp.simplify(D * sp.diff(us, s))
check("C04 transfer: q(L) = -D k sin(kL) u0 + cos(kL) q0",
      sp.simplify(qs - (-D * k * sp.sin(k * s) * u0 + sp.cos(k * s) * q0)) == 0)
uI, qin = sp.symbols("u_I q_in")
qout = qin + (D2 - D1) * uI / Rr
check("C03 interface: q_out = q_in + (D_out - D_in) u/R keeps D(u' - u/r) continuous",
      sp.simplify((qin - D1 * uI / Rr) - (qout - D2 * uI / Rr)) == 0)

rr, z, Rs, Dd, hh = sp.symbols("r z R D h", positive=True)
phi = sp.sin(z * rr / Rs) / rr
robin = sp.simplify((-Dd * sp.diff(phi, rr) - hh * phi).subs(rr, Rs) * Rs**2 / (Dd * sp.sin(z)))
Bi = hh * Rs / Dd
check("C14 Robin: -D phi'(R) = h phi(R) with phi = sin(z r/R)/r  <=>  z cot z = 1 - Bi",
      sp.simplify(robin - (1 - z * sp.cot(z) - Bi)) == 0)

kk, R1 = sp.symbols("k R_1", positive=True)
I1 = sp.integrate(rr * sp.sin(kk * rr), (rr, 0, R1))
check("C08 closed form I1 = (sin kR1 - kR1 cos kR1)/k^2 (DERIVED, code uses quadrature)",
      sp.simplify(I1 - (sp.sin(kk * R1) - kk * R1 * sp.cos(kk * R1)) / kk**2) == 0)

# Homogeneous sphere, R1 = R: w_n = 6/(n^2 pi^2)
nn = sp.symbols("n", positive=True, integer=True)
kn = nn * sp.pi / Rs
In = sp.integrate(rr * sp.sin(kn * rr), (rr, 0, Rs))
Nn = sp.integrate(sp.sin(kn * rr) ** 2, (rr, 0, Rs))
w = sp.simplify(3 * In * In / (Rs**3 * Nn))
check("C10 weight 3 I1 In/(R1^3 Nn) reduces to 6/(n^2 pi^2) when R1 = R (Crank limit)",
      sp.simplify(w - 6 / (nn**2 * sp.pi**2)) == 0, f"w_n = {w}")

# ---------------------------------------------------------------- Code C, numerical
def gl16():
    nq = 16
    x = np.zeros(nq)
    wq = np.zeros(nq)
    for i in range(nq):
        zz = math.cos((i + 0.75) / (nq + 0.5) * math.pi)
        dp = 1.0
        for _ in range(100):
            p_prev, p = 1.0, zz
            for kq in range(2, nq + 1):
                pk = ((2 * kq - 1) * zz * p - (kq - 1) * p_prev) / kq
                p_prev, p = p, pk
            dp = nq * (zz * p - p_prev) / (zz * zz - 1.0)
            dz = p / dp
            zz -= dz
            if abs(dz) < 1e-15:
                break
        x[i] = zz
        wq[i] = 2.0 / ((1.0 - zz * zz) * dp * dp)
    return x, wq


xg, wg = gl16()
xr, wr = np.polynomial.legendre.leggauss(16)
check("C08 Gauss-Legendre 16 nodes/weights equal numpy leggauss",
      np.allclose(np.sort(xg), xr, atol=1e-14) and np.allclose(wg[np.argsort(xg)], wr, atol=1e-14))
e31 = abs(np.sum(wg * xg**30) - 2.0 / 31.0)
e32 = abs(np.sum(wg * xg**32) - 2.0 / 33.0)
check("C08 rule exact to degree 31 (x^30 integrated exactly, x^32 not)", e31 < 1e-14 and e32 > 1e-12,
      f"err(x^30) {e31:.1e}, err(x^32) {e32:.1e}")


def sturm_count(lam, layers):
    nz = 0
    u, q = 0.0, 1.0
    left = 0.0
    prev = None
    for ro, d in layers:
        if prev is not None:
            q = q + (d - prev) * u / left
        kq = math.sqrt(lam / d)
        aa, bb = u, q / (d * kq)
        th = math.atan2(aa, bb)
        L = ro - left
        nz += math.floor((th + kq * L) / math.pi) - math.floor(th / math.pi)
        c_, s_ = math.cos(kq * L), math.sin(kq * L)
        u, q = c_ * u + s_ * q / (d * kq), -d * kq * s_ * u + c_ * q
        m = max(abs(u), abs(q / (d * kq)))
        u, q = u / m, q / m
        left = ro
        prev = d
    return max(nz, 0)


lay_h = [(2e-4, 1e-10), (3e-4, 1e-10), (3.5e-4, 1e-10), (4e-4, 1e-10), (4.5e-4, 1e-10)]
ok = True
for lam in np.linspace(1e-4, 5.0, 400):
    exact_n = math.floor(math.sqrt(lam / 1e-10) * 4.5e-4 / math.pi)
    ok &= sturm_count(lam, lay_h) == exact_n
check("C05 Sturm count via atan2 phase equals floor(sqrt(Lambda/D) R/pi) for equal D (400 Lambdas)", ok)

lay_s = [(ro, d) for ro, d, _ in regions]
nmodes = sturm_count(45.0 / 2000.0, lay_s)
check("C06 Sturm count at Lambda_max = 45/t_min = 0.0225 1/s gives 32 modes (golden analytical_modes=32)",
      nmodes == 32, f"count {nmodes}")
check("C12 validity time 40/Lambda_covered = 40/0.0225 = 1.777778e3 s (golden analytical_valid_from_s)",
      abs(40.0 / 0.0225 - 1.777778e3) < 1e-3)
check("C11 accuracy floor 2.2e-16/rho_min with rho_min = 5.504e-5 gives 3.997e-12 (golden)",
      abs(2.2e-16 / 5.504e-5 - 3.997e-12) < 1e-15, f"{2.2e-16 / 5.504e-5:.4e}")

print()
print("ALL PASS" if not FAILS else f"FAILED: {FAILS}")
