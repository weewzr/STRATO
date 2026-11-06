"""STRATO v1.0.0, Full Mathematical Derivation, Part VI: consistency of Code B.

Truncation error of the half-cell (series-resistance) face flux of
src/numerical/finite_volume.rs lines 78 to 91, evaluated with exact cell averages
(weight 4 pi r^2) and exact volume centroids, for
  (a) an interior face of one material with equal cell widths w,
  (b) a material interface with widths w_L = alpha w, w_R = beta w and diffusivities D_L, D_R,
  (c) the outer face (half cell to c = 0 at r = R),
plus the local error of one Backward Euler step and the cell residual at an interface.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/v1_0_0_p6_truncation.py
Writes nothing. One PASS/FAIL line per check; exit code 1 on any FAIL.
"""

import sys

import sympy as sp

ok_all = []


def check(label, ok, detail=""):
    ok = bool(ok)
    ok_all.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {label}" + (f"  [{detail}]" if detail else ""), flush=True)


r, s, w, rf = sp.symbols("r s w r_f", positive=True)
al, be = sp.symbols("alpha beta", positive=True)
D, DL, DR = sp.symbols("D D_L D_R", positive=True)
c0, c1, c2, c3, c4 = sp.symbols("c0 c1 c2 c3 c4")
c2L, c3L, c2R, c3R = sp.symbols("c2L c3L c2R c3R")
ORDER = 4


def avg_and_centroid(f, a, b):
    """Cell average of f(r) with weight r^2 and the volume centroid of [a, b]."""
    Vol = sp.integrate(r**2, (r, a, b))
    avg = sp.integrate(f * r**2, (r, a, b)) / Vol
    cen = sp.integrate(r**3, (r, a, b)) / Vol
    return avg, cen


def series_w(expr, n=ORDER):
    return sp.series(expr, w, 0, n).removeO()


# (a) interior face, one material, equal widths
f = c0 + c1 * (r - rf) + c2 * (r - rf) ** 2 / 2 + c3 * (r - rf) ** 3 / 6 + c4 * (r - rf) ** 4 / 24
aL, cenL = avg_and_centroid(f, rf - w, rf)
aR, cenR = avg_and_centroid(f, rf, rf + w)
Af = 4 * sp.pi * rf**2
Gf = Af / ((rf - cenL) / D + (cenR - rf) / D)
tau_a = series_w(sp.simplify(Gf * (aL - aR) + Af * D * c1))
lead_a = sp.simplify(tau_a.coeff(w, 2))
check("6.5a interior face: O(1) and O(w) terms of tau vanish",
      sp.simplify(tau_a.coeff(w, 0)) == 0 and sp.simplify(tau_a.coeff(w, 1)) == 0)
target_a = -sp.Rational(2, 3) * sp.pi * D * rf * (c2 + rf * c3 / 2)
check("6.5a interior face: tau = -(2 pi/3) D w^2 r_f (c'' + r_f c'''/2) + O(w^3)",
      sp.simplify(lead_a - target_a) == 0, f"{sp.factor(lead_a)}")

# cell residual in the interior: tau_{j-1/2} - tau_{j+1/2} over V_j is O(w^2)
# (tau is a smooth function of r_f times w^2, so its difference across one cell is O(w^3))
tau_fun = lambda x: target_a.subs(rf, x) * w**2
# with c2, c3 constants of a cubic profile, d tau/d r_f is c-independent of w
res_cell = sp.series((tau_fun(rf) - tau_fun(rf + w)) / (sp.Rational(4, 3) * sp.pi * ((rf + w) ** 3 - rf**3)), w, 0, 3).removeO()
check("6.5a interior cell residual (tau_W - tau_E)/V_j = O(w^2)",
      sp.simplify(res_cell.coeff(w, 0)) == 0 and sp.simplify(res_cell.coeff(w, 1)) == 0 and sp.simplify(res_cell.coeff(w, 2)) != 0)

# (b) material interface at r_f: c continuous, D c' continuous, c'' and c''' free on each side
c1R = DL * c1 / DR
fL = c0 + c1 * (r - rf) + c2L * (r - rf) ** 2 / 2 + c3L * (r - rf) ** 3 / 6
fR = c0 + c1R * (r - rf) + c2R * (r - rf) ** 2 / 2 + c3R * (r - rf) ** 3 / 6
aL, cenL = avg_and_centroid(fL, rf - al * w, rf)
aR, cenR = avg_and_centroid(fR, rf, rf + be * w)
Gi = Af / ((rf - cenL) / DL + (cenR - rf) / DR)
tau_b = series_w(sp.simplify(Gi * (aL - aR) + Af * DL * c1), 3)
check("6.5b interface face: O(1) term of tau vanishes (consistency)", sp.simplify(tau_b.coeff(w, 0)) == 0)
lead_b = sp.simplify(tau_b.coeff(w, 1))
target_b = Af * DL * DR / 3 * (al**2 * c2L - be**2 * c2R) / (DR * al + DL * be)
check("6.5b interface face: tau = (4 pi r_f^2 D_L D_R/3) (w_L^2 c''_L - w_R^2 c''_R)/(D_R w_L + D_L w_R) + O(w^2)",
      sp.simplify(lead_b - target_b) == 0)
# the O(w) term vanishes only in special cases, e.g. equal D and equal widths and equal c''
check("6.5b interface O(w) term vanishes when D_L = D_R, alpha = beta and c''_L = c''_R (one material)",
      sp.simplify(target_b.subs({DR: DL, be: al, c2R: c2L})) == 0)
# cell residual at an interface cell is O(1): tau = O(w), V_j = O(w)
Vint = sp.Rational(4, 3) * sp.pi * (rf**3 - (rf - al * w) ** 3)
lim = sp.limit(target_b * w / Vint, w, 0)
check("6.5b interface cell residual tau/V_j tends to a non-zero O(1) constant",
      sp.simplify(lim) != 0, f"limit {sp.factor(lim)}")
# Physical c'' jump: for the layer equation, D c'' + 2 D c'/r = dc/dt is continuous at the interface,
# so D_L c''_L = D_R c''_R in exact arithmetic (c and dc/dt and D c' continuous).
check("6.5b with D_L c''_L = D_R c''_R and equal widths the O(w) term does not vanish when D_L != D_R",
      sp.simplify(target_b.subs({c2R: DL * c2L / DR, be: al}).subs({DL: 1, DR: 2, c2L: 1, al: 1})) != 0)

tb_phys = sp.factor(target_b.subs(c2R, DL * c2L / DR))
check("6.5b with D_L c''_L = D_R c''_R: tau = (4 pi r_f^2 D_L c''_L/3)(D_R w_L^2 - D_L w_R^2)/(D_R w_L + D_L w_R) + O(w^2)",
      sp.simplify(tb_phys - Af * DL * c2L / 3 * (DR * al**2 - DL * be**2) / (DR * al + DL * be)) == 0)
check("6.5b that O(w) term vanishes exactly when w_L/w_R = sqrt(D_L/D_R) (equal w^2/D on both sides)",
      sp.simplify(tb_phys.subs(al, be * sp.sqrt(DL / DR))) == 0)
check("6.5b synthetic kernel/buffer interface: w_L/w_R = 2 against sqrt(D_L/D_R) = 0.05, so the O(w) term is present",
      abs((2.0e-4 / 40) / (1.0e-4 / 40) - 2) < 1e-12 and abs((1e-12 / 4e-10) ** 0.5 - 0.05) < 1e-15)

# (c) outer face: c(R) = 0
R = sp.symbols("R", positive=True)
fo = c1 * (r - R) + c2 * (r - R) ** 2 / 2 + c3 * (r - R) ** 3 / 6
aN, cenN = avg_and_centroid(fo, R - w, R)
AR = 4 * sp.pi * R**2
GR = AR * D / (R - cenN)
tau_c = series_w(sp.simplify(GR * aN + AR * D * c1), 3)
check("6.5c outer face: O(1) term of tau vanishes", sp.simplify(tau_c.coeff(w, 0)) == 0)
lead_c = sp.simplify(tau_c.coeff(w, 1))
check("6.5c outer face: tau = (4 pi R^2 D w/3) c''(R^-) + O(w^2)",
      sp.simplify(lead_c - AR * D * w**0 * c2 / 3) == 0, f"{sp.factor(lead_c)}")

# centre: zero area, the discrete and exact flows are both zero
check("6.5 centre face: A(0) = 0, so tau_{1/2} = 0 exactly", (4 * sp.pi * r**2).subs(r, 0) == 0)

# Backward Euler local error
t, h = sp.symbols("t h", positive=True)
y = sp.Function("y")
taylor = lambda k: sum(sp.Derivative(y(t), t, i) * h**i / sp.factorial(i) for i in range(k)) if k else 0
yph = sum(sp.Symbol(f"y{i}") * h**i / sp.factorial(i) for i in range(5))           # y(t+h)
ypph = sum(sp.Symbol(f"y{i+1}") * h**i / sp.factorial(i) for i in range(4))        # y'(t+h)
local = sp.expand(yph - h * ypph - sp.Symbol("y0"))
check("6.5 BE local error: y(t+h) - h y'(t+h) - y(t) = -h^2 y''/2 + O(h^3)",
      local.coeff(h, 1) == 0 and local.coeff(h, 2) == -sp.Symbol("y2") / 2, f"{sp.series(local, h, 0, 4)}")
check("6.5 BE truncation per unit time [y(t+h) - y(t)]/h - y'(t+h) = -h y''/2 + O(h^2): first order",
      sp.expand(local / h).coeff(h, 1) == -sp.Symbol("y2") / 2 and sp.expand(local / h).coeff(h, 0) == 0)

print("ALL PASS" if all(ok_all) else "SOME FAIL")
sys.exit(0 if all(ok_all) else 1)
