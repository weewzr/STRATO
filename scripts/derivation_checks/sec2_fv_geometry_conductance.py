"""Symbolic checks for Section 2 (Phase 1 items 16 to 19): spherical control-volume
geometry, interface conductance, Backward Euler truncation and the subtraction-free
tridiagonal elimination used in Code B (src/numerical/finite_volume.rs).

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec2_fv_geometry_conductance.py
Writes nothing. Prints one PASS/FAIL line per check and exits non-zero on any FAIL.
Equation labels refer to sec2_phase1_finite_volume.md, equations (P1.40) to (P1.79).
"""

import sys
import time

import sympy as sp

results = []
T0 = time.perf_counter()


def check(label, ok, detail=""):
    """Record and print one PASS/FAIL line (flushed, with elapsed time)."""
    ok = bool(ok)
    results.append(ok)
    tail = f"  [{detail}]" if detail else ""
    print(f"{'PASS' if ok else 'FAIL'}  {label}{tail}  ({time.perf_counter() - T0:.1f} s)", flush=True)


def is_zero(expr):
    """Rational-function zero test: cancel is exact and much faster than simplify."""
    return sp.cancel(sp.together(expr)) == 0


def taylor_coeffs(expr, var, order):
    """Taylor coefficients [c0, ..., c_order] of a rational function analytic at var = 0."""
    num, den = sp.fraction(sp.cancel(sp.together(expr)))
    out = []
    f = num / den
    for k in range(order + 1):
        out.append(sp.cancel(f.subs(var, 0) / sp.factorial(k)))
        f = sp.cancel(sp.diff(f, var))
    return out


# ---------------------------------------------------------------------------
# Geometry (P1.46) to (P1.52)
# ---------------------------------------------------------------------------
r, a, b, h, m = sp.symbols("r a b h m", positive=True)
alpha, beta, gamma = sp.symbols("alpha beta gamma", real=True)

V = sp.integrate(4 * sp.pi * r**2, (r, a, b))
check("(P1.46) shell volume = 4 pi/3 (b^3 - a^3)", is_zero(V - sp.Rational(4, 3) * sp.pi * (b**3 - a**3)))
check("(P1.46) factored form (b-a)(a^2+ab+b^2)", is_zero(V - sp.Rational(4, 3) * sp.pi * (b - a) * (a**2 + a * b + b**2)))

rbar_def = sp.integrate(r * 4 * sp.pi * r**2, (r, a, b)) / V
rbar = sp.Rational(3, 4) * (b**4 - a**4) / (b**3 - a**3)
check("(P1.49) centroid int r dV / V = 3/4 (b^4-a^4)/(b^3-a^3)", is_zero(rbar_def - rbar))
check("(P1.50) centre cell a -> 0 gives 3b/4 (code branch a == 0)", sp.limit(rbar, a, 0) == sp.Rational(3, 4) * b)

# Thin-shell expansion about the midpoint m with width h
rbar_mh = rbar.subs({a: m - h / 2, b: m + h / 2})
ser = sp.series(rbar_mh, h, 0, 5).removeO()
check("(P1.50) thin shell: centroid = m + h^2/(6m) + O(h^4)",
      is_zero(sp.expand(ser).coeff(h, 1)) and is_zero(sp.expand(ser).coeff(h, 2) - 1 / (6 * m))
      and is_zero(sp.expand(ser).coeff(h, 0) - m) and is_zero(sp.expand(ser).coeff(h, 3)),
      f"series = {sp.simplify(ser)}")

# a < rbar < b : numerators as polynomials in (a, h) with b = a + h have nonnegative coefficients
num_lo = sp.expand((3 * (b**4 - a**4) - 4 * a * (b**3 - a**3)).subs(b, a + h))
num_hi = sp.expand((4 * b * (b**3 - a**3) - 3 * (b**4 - a**4)).subs(b, a + h))
den = sp.expand((4 * (b**3 - a**3)).subs(b, a + h))
coeffs_ok = all(c >= 0 for p in (num_lo, num_hi, den) for c in sp.Poly(p, a, h).coeffs())
check("(P1.52) a < rbar < b for every shell (positive-coefficient numerators)", coeffs_ok,
      f"rbar-a ~ {sp.factor(num_lo)}/{sp.factor(den)}")

# Linear exactness and quadratic remainder (P1.51)
avg = lambda f: sp.integrate(f * 4 * sp.pi * r**2, (r, a, b)) / V
check("(P1.51) linear profile: cell average = c(rbar) exactly", is_zero(avg(alpha + beta * r) - (alpha + beta * rbar)))
sigma2 = avg((r - rbar) ** 2)
check("(P1.51) quadratic profile: average - c(rbar) = gamma sigma^2 exactly",
      is_zero(avg(alpha + beta * r + gamma * r**2) - (alpha + beta * rbar + gamma * rbar**2) - gamma * sigma2))
s2 = sp.series(sigma2.subs({a: m - h / 2, b: m + h / 2}), h, 0, 4).removeO()
check("(P1.51) sigma^2 = h^2/12 + O(h^4) for a thin shell", is_zero(s2 - h**2 / 12), f"series = {sp.simplify(s2)}")
r2avg = avg(r**2)
check("(P1.51) <r^2> = 3/5 (b^5-a^5)/(b^3-a^3)", is_zero(r2avg - sp.Rational(3, 5) * (b**5 - a**5) / (b**3 - a**3)))

# ---------------------------------------------------------------------------
# Interface conductance (P1.54) to (P1.61)
# ---------------------------------------------------------------------------
A, dP, dE, DP, DE, CP, CE, CI, F = sp.symbols("A_f delta_P delta_E D_P D_E C_P C_E C_I F", positive=True)
sol = sp.solve([sp.Eq(F, A * DP * (CP - CI) / dP), sp.Eq(F, A * DE * (CI - CE) / dE)], [F, CI], dict=True)[0]
G = A / (dP / DP + dE / DE)
check("(P1.56) series-resistance conductance: F = G_f (C_P - C_E)", is_zero(sol[F] - G * (CP - CE)))
CI_formula = (DP * CP / dP + DE * CE / dE) / (DP / dP + DE / dE)
check("(P1.55) interface value is the conductance-weighted mean", is_zero(sol[CI] - CI_formula))
w = (DP / dP) / (DP / dP + DE / dE)
check("(P1.55) weight in (0,1): C_I lies between C_P and C_E", is_zero(CI_formula - (w * CP + (1 - w) * CE)))

dl = sp.symbols("delta", positive=True)
Gh = G.subs({dP: dl / 2, dE: dl / 2})
check("(P1.57) equal half distances: G = A D_h / delta, D_h = 2 D_P D_E/(D_P+D_E)",
      is_zero(Gh - A * (2 * DP * DE / (DP + DE)) / dl))
check("(P1.57) equal diffusivity: G = A D/(delta_P+delta_E)", is_zero(G.subs(DE, DP) - A * DP / (dP + dE)))
check("(P1.58) D_E -> infinity: G -> A D_P/delta_P", is_zero(sp.limit(G, DE, sp.oo) - A * DP / dP))
check("(P1.58) D_E -> 0: G ~ A D_E/delta_E", is_zero(sp.limit(G / DE, DE, 0) - A / dE))
check("(P1.59) delta_E -> 0 (Dirichlet face): G -> A D_P/delta_P = G_R", is_zero(sp.limit(G, dE, 0) - A * DP / dP))

# Spherical versus planar half-cell resistance inside one material, equal widths
rf, hh = sp.symbols("r_f h", positive=True)
cent = lambda lo, hi: sp.Rational(3, 4) * (hi**4 - lo**4) / (hi**3 - lo**3)
rP, rE = cent(rf - hh, rf), cent(rf, rf + hh)
ratio = rf**2 / (rP * rE)  # G_code / G_exact_shell
ser = sp.series(ratio, hh, 0, 3).removeO()
check("(P1.61) code G vs exact steady shell conductance: ratio = 1 + O(h^2) (no O(h) term)",
      is_zero(sp.expand(ser).coeff(hh, 0) - 1) and is_zero(sp.expand(ser).coeff(hh, 1)),
      f"ratio = 1 + ({sp.simplify(sp.expand(ser).coeff(hh, 2))}) h^2 + ...")

# ---------------------------------------------------------------------------
# Flux truncation error at a face (P1.78), Taylor analysis with exact cell averages
# ---------------------------------------------------------------------------
eps, hP, hE, cf, qf = sp.symbols("epsilon h_P h_E c_f q_f", positive=True)
p2, p3, q2, q3 = sp.symbols("p2 p3 q2 q3", real=True)
x = sp.symbols("x", real=True)
cL = cf - qf / DP * x + p2 * x**2 + p3 * x**3  # left material, D_P c' = -q_f at x = 0
cR = cf - qf / DE * x + q2 * x**2 + q3 * x**3  # right material, D_E c' = -q_f at x = 0


def cell(cfun, lo, hi):
    """Exact r^2-weighted average and centroid offset (x = r - r_f) on [lo, hi].

    The width factor eps is cancelled so that both are analytic at eps = 0."""
    wgt = sp.expand((rf + x) ** 2)
    vol = sp.integrate(wgt, (x, lo, hi))
    avg_c = sp.cancel(sp.integrate(sp.expand(cfun * wgt), (x, lo, hi)) / vol)
    off = sp.cancel(sp.integrate(x * wgt, (x, lo, hi)) / vol)
    return avg_c, off


Af = 4 * sp.pi * rf**2
CPx, xP = cell(cL, -eps * hP, 0)
CEx, xE = cell(cR, 0, eps * hE)
# Resistances delta/D are O(eps); write G = Af/(eps*rho) with rho analytic at eps = 0.
rho = sp.cancel((-xP / DP + xE / DE) / eps)
tau_over = (CPx - CEx) / eps / rho  # = G (C_P - C_E) / Af, analytic at eps = 0
tc = taylor_coeffs(tau_over * Af - Af * qf, eps, 1)
lead = 2 * Af * (p2 * hP**2 - q2 * hE**2) / (3 * (hP / DP + hE / DE))
check("(P1.78) interface face: O(1) term of flux error vanishes (consistency)", tc[0] == 0)
check("(P1.78) interface face: flux error = eps*2A(p2 hP^2 - q2 hE^2)/(3(hP/DP+hE/DE)) + O(eps^2)",
      is_zero(tc[1] - lead), f"leading coefficient = {sp.factor(lead)}")
# smooth single material with equal widths: O(eps) term vanishes, O(eps^2) term is generically nonzero
ts = taylor_coeffs((tau_over * Af - Af * qf).subs({DE: DP, q2: p2, q3: p3, hE: hP}), eps, 2)
check("(P1.78) interior face (same D, smooth c, equal widths): flux error is O(eps^2)",
      ts[0] == 0 and ts[1] == 0 and ts[2] != 0, f"eps^2 coefficient = {sp.factor(ts[2])}")
# outer Dirichlet face: c(R) = 0 on the cell [R - eps hP, R], G_R = Af D_P / delta_N
CNx, xN = cell(cL.subs(cf, 0), -eps * hP, 0)
tR = taylor_coeffs(Af * DP * sp.cancel(CNx / eps) / sp.cancel(-xN / eps) - Af * qf, eps, 1)
check("(P1.78) outer face c(R)=0: flux error = eps*2 A D_P p2 hP/3 + O(eps^2)",
      tR[0] == 0 and is_zero(tR[1] - 2 * Af * DP * p2 * hP / 3))

# ---------------------------------------------------------------------------
# Backward Euler (P1.65) to (P1.67)
# ---------------------------------------------------------------------------
dt, f0, f1, f2, f3 = sp.symbols("Delta_t f0 f1 f2 f3", real=True)
y_next = f0 + f1 * dt + f2 * dt**2 / 2 + f3 * dt**3 / 6  # Taylor of y(t+dt)
yp_next = f1 + f2 * dt + f3 * dt**2 / 2  # Taylor of y'(t+dt)
tauBE = sp.expand((y_next - f0) / dt - yp_next)
check("(P1.65) BE local truncation = -(dt/2) y'' - (dt^2/3) y''' (first order)",
      is_zero(tauBE - (-f2 * dt / 2 - f3 * dt**2 / 3)))
z = sp.symbols("z", positive=True)
gBE = 1 / (1 + z)
check("(P1.66) amplification 1/(1+z) - exp(-z) = z^2/2 + O(z^3)",
      is_zero(sp.series(gBE - sp.exp(-z), z, 0, 3).removeO() - z**2 / 2))
check("(P1.66) 0 < g < 1 for z > 0 and g -> 0 as z -> oo (L-stable)",
      sp.limit(gBE, z, sp.oo) == 0 and is_zero(1 - gBE - z / (1 + z)))
Vj, gW, gE = sp.symbols("V_j g_W g_E", positive=True)
dtmax = sp.solve(sp.Eq(1 - dt * (gW + gE) / Vj, 0), dt)[0]
check("(P1.67) forward-Euler self coefficient >= 0 iff dt <= V/(gW+gE)", is_zero(dtmax - Vj / (gW + gE)))

# ---------------------------------------------------------------------------
# Tridiagonal system and subtraction-free elimination (P1.68) to (P1.74)
# ---------------------------------------------------------------------------
N = 4
v = sp.symbols(f"v1:{N + 1}", positive=True)
g = sp.symbols(f"g1:{N}", positive=True)  # internal face conductances, g[j] between cells j and j+1
GR = sp.symbols("G_R", positive=True)
rhs = sp.symbols(f"s1:{N + 1}", positive=True)
gw = [0] + list(g)
ge = list(g) + [GR]
Amat = sp.zeros(N, N)
for j in range(N):
    Amat[j, j] = v[j] + gw[j] + ge[j]
    if j > 0:
        Amat[j, j - 1] = -gw[j]
    if j < N - 1:
        Amat[j, j + 1] = -ge[j]
check("(P1.69) A is symmetric", Amat == Amat.T)
colsum = sp.ones(1, N) * Amat
target = sp.Matrix([[v[j] + (GR if j == N - 1 else 0) for j in range(N)]])
check("(P1.69) column sums: 1^T A = v^T + G_R e_N^T (discrete conservation)",
      all(is_zero(colsum[j] - target[j]) for j in range(N)))
check("(P1.69) row j: diagonal - |off-diagonals| = v_j > 0 (strict dominance)",
      all(is_zero(Amat[j, j] - sum(abs(Amat[j, k]) for k in range(N) if k != j) - v[j] - (GR if j == N - 1 else 0))
          for j in range(N)))

# Subtraction-free recurrences, exactly as in solve_diffusion_system
e, d, yv = [None] * N, [None] * N, [None] * N
for j in range(N):
    e[j] = v[0] if j == 0 else v[j] + gw[j] * e[j - 1] / (e[j - 1] + gw[j])
    d[j] = e[j] + ge[j]
    yv[j] = (rhs[j] + (gw[j] * yv[j - 1] if j > 0 else 0)) / d[j]
xs = [None] * N
xs[N - 1] = yv[N - 1]
for j in range(N - 2, -1, -1):
    xs[j] = yv[j] + ge[j] / d[j] * xs[j + 1]
res = Amat * sp.Matrix(xs) - sp.Matrix(rhs)
check("(P1.71)-(P1.73) subtraction-free elimination solves A x = rhs exactly (N=4, symbolic)",
      all(is_zero(res[j]) for j in range(N)))

# Textbook pivot d_j = b_j - a_j c_{j-1}/d_{j-1} equals e_j + g^E_j
dt_book = [None] * N
for j in range(N):
    bj = v[j] + gw[j] + ge[j]
    dt_book[j] = bj if j == 0 else bj - gw[j] * ge[j - 1] / dt_book[j - 1]
check("(P1.70)-(P1.71) textbook pivot equals e_j + g^E_j for every row",
      all(is_zero(dt_book[j] - d[j]) for j in range(N)))

# Relative sensitivity of e_j to e_{j-1} is below 1 (rounding errors are not amplified)
ev, gv, vv = sp.symbols("e g v", positive=True)
ej = vv + gv * ev / (ev + gv)
sens = sp.diff(ej, ev) * ev / ej
num = sp.factor(sp.together(1 - sens))
check("(P1.74) d ln e_j / d ln e_{j-1} = g^2 e/((e+g)^2 e_j) and 1 - it > 0",
      is_zero(sens - gv**2 * ev / ((ev + gv) ** 2 * ej))
      and is_zero(sp.numer(num) - (vv * (ev + gv) ** 2 + gv * ev**2)),
      f"1 - sens = {num}")
fixed = sp.solve(sp.Eq(ev, vv + gv * ev / (ev + gv)), ev)
fixed_pos = [s for s in fixed if s.is_positive is not False]
check("(P1.74) fixed point e* = (v + sqrt(v^2 + 4 v g))/2 ~ sqrt(v g) for v << g",
      any(is_zero(s - (vv + sp.sqrt(vv**2 + 4 * vv * gv)) / 2) for s in fixed_pos))

npass = sum(results)
print(f"SUMMARY {npass}/{len(results)} checks passed")
print("ALL PASS" if all(results) else "SOME CHECKS FAILED")
sys.exit(0 if all(results) else 1)
