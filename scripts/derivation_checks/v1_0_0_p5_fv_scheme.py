"""STRATO v1.0.0, Full Mathematical Derivation, Part V: symbolic checks of the
finite-volume scheme (Code B, src/numerical/finite_volume.rs at tag v1.0.0).

Each check reproduces one derivation step of Part V with SymPy (exact algebra) or
with exact rational arithmetic (fractions.Fraction) where floating point is the topic.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/v1_0_0_p5_fv_scheme.py
Writes nothing. Prints one PASS/FAIL line per check; exit code 1 on any FAIL.
"""

import math
import sys
from fractions import Fraction

import sympy as sp

results = []


def check(label, ok, detail=""):
    ok = bool(ok)
    results.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {label}" + (f"  [{detail}]" if detail else ""), flush=True)


r, a, b, rho = sp.symbols("r a b rho", positive=True)
alpha, beta = sp.symbols("alpha beta", real=True)

# ---------------------------------------------------------------- 5.3 volume
V = sp.integrate(4 * sp.pi * rho**2, (rho, a, b))
check("P5-01 Eq. (5.3): integral of 4 pi r^2 from a to b equals (4 pi/3)(b^3 - a^3)",
      sp.simplify(V - sp.Rational(4, 3) * sp.pi * (b**3 - a**3)) == 0)

# ---------------------------------------------------------------- 5.4 centroid
first_moment = sp.integrate(rho * 4 * sp.pi * rho**2, (rho, a, b))
rj = sp.simplify(first_moment / V)
code_rj = sp.Rational(3, 4) * (b**4 - a**4) / (b**3 - a**3)
check("P5-02 Eq. (5.4): int r dV / int dV equals 0.75 (b^4 - a^4)/(b^3 - a^3) (volume_centroid, line 46)",
      sp.simplify(rj - code_rj) == 0)
factored = sp.Rational(3, 4) * (a + b) * (a**2 + b**2) / (a**2 + a * b + b**2)
check("P5-03 cancelled form r_j = 3(a+b)(a^2+b^2)/(4(a^2+ab+b^2))",
      sp.simplify(code_rj - factored) == 0)
check("P5-04 a -> 0 limit of the centroid is 3b/4 (branch at line 44)",
      sp.simplify(sp.limit(factored, a, 0) - sp.Rational(3, 4) * b) == 0)
shift = sp.simplify(factored - (a + b) / 2)
target_shift = (a + b) * (a - b) ** 2 / (4 * (a**2 + a * b + b**2))
check("P5-05 r_j - (a+b)/2 = (a+b)(b-a)^2/(4(a^2+ab+b^2)) >= 0 (centroid lies outward of the midpoint)",
      sp.simplify(shift - target_shift) == 0)
lower = sp.factor(sp.simplify(factored - a))
upper = sp.factor(sp.simplify(b - factored))
check("P5-06 a < r_j < b: r_j - a and b - r_j have the closed forms below",
      sp.simplify(lower - (b - a) * (3 * b**2 + 2 * a * b + a**2) / (4 * (a**2 + a * b + b**2))) == 0
      and sp.simplify(upper - (b - a) * (b**2 + 2 * a * b + 3 * a**2) / (4 * (a**2 + a * b + b**2))) == 0,
      f"r_j - a = {lower}; b - r_j = {upper}")
m, h = sp.symbols("m h", positive=True)
shift_mh = sp.simplify(target_shift.subs({a: m - h / 2, b: m + h / 2}))
lead = sp.series(shift_mh, h, 0, 4).removeO()
check("P5-07 for a thin cell (width h, midpoint m) the centroid shift is h^2/(6m) + O(h^4)",
      sp.simplify(lead - h**2 / (6 * m)) == 0, f"series {sp.simplify(lead)}")

# linear profile: cell average equals value at the centroid
avg_lin = sp.simplify(sp.integrate((alpha + beta * rho) * 4 * sp.pi * rho**2, (rho, a, b)) / V)
check("P5-08 for c = alpha + beta r the cell average (5.1) equals c(r_j) exactly",
      sp.simplify(avg_lin - (alpha + beta * code_rj)) == 0)
avg_quad = sp.simplify(sp.integrate(rho**2 * 4 * sp.pi * rho**2, (rho, a, b)) / V)
check("P5-09 for c = r^2 the cell average differs from c(r_j) (difference is O(h^2), so the match is exact only for straight lines)",
      sp.simplify(avg_quad - code_rj**2) != 0)

# ---------------------------------------------------------------- 5.5 face flux
Cj, Ce, Cf, J, Dj, De, dL, dR, Af = sp.symbols("C_j C_e C_f J D_j D_e delta_L delta_R A_f", positive=True)
sol = sp.solve([sp.Eq(J, Dj * (Cj - Cf) / dL), sp.Eq(J, De * (Cf - Ce) / dR)], [J, Cf], dict=True)[0]
G = Af / (dL / Dj + dR / De)
check("P5-10 Eq. (5.9): eliminating C_f gives A_f J = G (C_j - C_{j+1}), G = A_f/(dL/D_j + dR/D_{j+1}) (line 86)",
      sp.simplify(Af * sol[J] - G * (Cj - Ce)) == 0)
Cf_expected = (Dj * Cj / dL + De * Ce / dR) / (Dj / dL + De / dR)
check("P5-11 eliminated face value C_f = (D_j C_j/dL + D_e C_e/dR)/(D_j/dL + D_e/dR), a weighted average",
      sp.simplify(sol[Cf] - Cf_expected) == 0)
dd = sp.symbols("delta", positive=True)
Dh = 2 * Dj * De / (Dj + De)
check("P5-12 equal half distances: G = A_f D_h/(2 delta), D_h harmonic mean 2 D_j D_e/(D_j + D_e)",
      sp.simplify(G.subs({dL: dd, dR: dd}) - Af * Dh / (2 * dd)) == 0)
check("P5-13 equal diffusivities: G = A_f D/(dL + dR) = A_f D/(r_{j+1} - r_j)",
      sp.simplify(G.subs(De, Dj) - Af * Dj / (dL + dR)) == 0)
eps = sp.symbols("epsilon", positive=True)
G_small = G.subs(De, eps * Dj)
check("P5-14 if D_{j+1} -> 0 the conductance tends to the low-D half cell alone, A_f D_{j+1}/dR (bottleneck)",
      sp.limit(G_small / (Af * eps * Dj / dR), eps, 0) == 1)
check("P5-15 Eq. (5.11): G_R is Eq. (5.9) with dR = 0 (sink on the face): A D_5/(R - r_N) (line 91)",
      sp.simplify(G.subs(dR, 0) - Af * Dj / dL) == 0)

# spherical half-cell resistance remark (exact steady resistance vs code)
rP, rf, rE, Dsym = sp.symbols("r_P r_f r_E D", positive=True)
R_exact = (1 / rP - 1 / rE) / (4 * sp.pi * Dsym)
R_code = ((rf - rP) + (rE - rf)) / (4 * sp.pi * Dsym * rf**2)
ratio = sp.simplify(R_code / R_exact)
dLs, dRs = sp.symbols("dL dR", positive=True)
ratio_d = sp.expand(ratio.subs({rP: rf - dLs, rE: rf + dRs}))
check("P5-16 one material: code resistance / exact spherical resistance = r_P r_E / r_f^2 = 1 + (dR - dL)/r_f - dL dR/r_f^2",
      sp.simplify(ratio - rP * rE / rf**2) == 0
      and sp.simplify(ratio_d - (1 + (dRs - dLs) / rf - dLs * dRs / rf**2)) == 0)

# ---------------------------------------------------------------- 5.7 / 5.8 rows
Cm, C0, Cp, gW, gE, Vj, dt, Cold = sp.symbols("C_m C_0 C_p g_W g_E V_j Delta_t C_old", real=True)
semi = gW * (Cm - C0) + gE * (Cp - C0)
fluxform = (gW * (Cm - C0)) - (gE * (C0 - Cp))
check("P5-17 Eq. (5.12): A_w J_w - A_e J_e with A J = G (C_left - C_right) gives g^W(C_{j-1}-C_j) + g^E(C_{j+1}-C_j)",
      sp.simplify(semi - fluxform) == 0)
be = Vj * (C0 - Cold) / dt - semi
row = (-gW * Cm + (Vj / dt + gW + gE) * C0 - gE * Cp) - (Vj / dt) * Cold
check("P5-18 Eq. (5.13) rearranged is the matrix row (5.14) (doc comment lines 122 to 124)",
      sp.simplify(be - row) == 0)

# ---------------------------------------------------------------- 5.10 elimination
n = 5
v = sp.symbols("v1:%d" % (n + 1), positive=True)
g = sp.symbols("g1:%d" % (n + 1), positive=True)   # g[k] = G at face k+1/2, g[n-1] = G_R
s = sp.symbols("s1:%d" % (n + 1), real=True)
gw = [0] + [g[k - 1] for k in range(1, n)]
ge = [g[k] for k in range(n)]
Amat = sp.zeros(n, n)
for k in range(n):
    Amat[k, k] = v[k] + gw[k] + ge[k]
    if k > 0:
        Amat[k, k - 1] = -gw[k]
    if k < n - 1:
        Amat[k, k + 1] = -ge[k]
check("P5-19 Eq. (5.15): assembled matrix is symmetric (g^W_{j+1} = g^E_j)", Amat == Amat.T)
check("P5-20 each row sum equals v_j, except the last, which is v_N + G_R",
      all(sp.simplify(sum(Amat[k, :]) - v[k]) == 0 for k in range(n - 1))
      and sp.simplify(sum(Amat[n - 1, :]) - v[n - 1] - g[n - 1]) == 0)
# textbook Thomas
d_tb = [Amat[0, 0]]
for k in range(1, n):
    d_tb.append(sp.simplify(Amat[k, k] - Amat[k, k - 1] * Amat[k - 1, k] / d_tb[k - 1]))
# subtraction-free
e = [v[0]]
for k in range(1, n):
    e.append(v[k] + gw[k] * e[k - 1] / (e[k - 1] + gw[k]))
d_sf = [e[k] + ge[k] for k in range(n)]
check("P5-21 Eq. (5.17) = Eq. (5.18): textbook pivot b_j - a_j c_{j-1}/d_{j-1} equals e_j + g^E_j for all rows (N = 5)",
      all(sp.simplify(d_tb[k] - d_sf[k]) == 0 for k in range(n)))
check("P5-22 Eq. (5.17) in conductances: d_j = v_j + g^W_j + g^E_j - (g^W_j)^2/d_{j-1}",
      all(sp.simplify(d_tb[k] - (v[k] + gw[k] + ge[k] - gw[k] ** 2 / d_tb[k - 1])) == 0 for k in range(1, n)))
y = [s[0] / d_sf[0]]
for k in range(1, n):
    y.append((s[k] + gw[k] * y[k - 1]) / d_sf[k])
x = [None] * n
x[n - 1] = y[n - 1]
for k in range(n - 2, -1, -1):
    x[k] = y[k] + ge[k] / d_sf[k] * x[k + 1]
res = [sp.cancel(sp.together(sum(Amat[k, mm] * x[mm] for mm in range(n)) - s[k])) for k in range(n)]
check("P5-23 Eq. (5.19): forward sweep y_j and back substitution solve the system exactly (N = 5, all symbols free)",
      all(rr == 0 for rr in res))
check("P5-24 every operation in (5.18), (5.19) is +, x or / of the symbols (no minus sign appears)",
      all("-" not in str(expr) for expr in e + d_sf + y + x))
# e_1 branch: the general formula is 0/0 at i = 0 in floating point
e_prev, gw0 = 0.0, 0.0
try:
    val = 1.0 + gw0 * e_prev / (e_prev + gw0)
except ZeroDivisionError:
    val = float("nan")
check("P5-25 general recurrence at the first cell would evaluate 0 * 0 / (0 + 0) (undefined), hence the i == 0 branch (lines 141 to 142)",
      math.isnan(val))

# ---------------------------------------------------------------- cancellation demo
# Exact rationals vs float for v = 1, g = 1e6 (cell Fourier number 1e6): 30 rows.
vv, gg, N = 1.0, 1.0e6, 30
e_ex = [Fraction(vv)]
for _ in range(1, N):
    e_ex.append(Fraction(vv) + Fraction(gg) * e_ex[-1] / (e_ex[-1] + Fraction(gg)))
e_sf = [vv]
for _ in range(1, N):
    e_sf.append(vv + gg * e_sf[-1] / (e_sf[-1] + gg))
d_t = [vv + gg]                       # textbook pivot, first row has no west face
for _ in range(1, N):
    d_t.append(vv + gg + gg - gg * gg / d_t[-1])
e_tb = [d_t[k] - gg for k in range(N)]  # e recovered from the textbook pivot
err_sf = max(abs(Fraction(e_sf[k]) - e_ex[k]) / e_ex[k] for k in range(N))
err_tb = max(abs(Fraction(e_tb[k]) - e_ex[k]) / e_ex[k] for k in range(N))
estar = 0.5 * (vv + math.sqrt(vv * vv + 4 * vv * gg))
check("P5-26 cancellation demo (v = 1, g = 1e6, 30 rows): subtraction-free e_j error is at round-off level",
      float(err_sf) < 1e-14, f"max rel error {float(err_sf):.2e}")
check("P5-27 same rows: e_j recovered from the textbook pivot loses digits (relative error grows with g/e_j, here g/e_j ~ 3e4 to 1e6)",
      float(err_tb) > 1e-13, f"max rel error {float(err_tb):.2e}; e_30 = {float(e_ex[-1]):.6g}; fixed point {estar:.6g}")

# ---------------------------------------------------------------- 5.8 decay example
k_, dt_ = 1.0, 3.0
fe = [1.0]
be_ = [1.0]
for _ in range(3):
    fe.append(fe[-1] * (1 - k_ * dt_))
    be_.append(be_[-1] / (1 + k_ * dt_))
check("P5-28 decay y' = -k y, k = 1 s^-1, dt = 3 s: forward Euler gives 1, -2, 4, -8",
      fe == [1.0, -2.0, 4.0, -8.0])
check("P5-29 same: Backward Euler gives 1, 0.25, 0.0625, 0.015625; exact e^{-3} per step = 0.049787",
      be_ == [1.0, 0.25, 0.0625, 0.015625], f"exp(-3) = {math.exp(-3):.6f}")
xx = sp.symbols("x", positive=True)
lte = sp.series(1 / (1 + xx) - sp.exp(-xx), xx, 0, 3).removeO()
check("P5-30 one-step error of Backward Euler: 1/(1+x) - e^{-x} = x^2/2 + O(x^3), x = k dt",
      sp.simplify(lte - xx**2 / 2) == 0)

# ---------------------------------------------------------------- 5.9 time step rule
steps = math.ceil(2.0e5 / 2.0e2)
check("P5-31 Eq. (5.16) synthetic input: N_t = ceil(2e5/2e2) = 1000, dt = 200 s exactly",
      steps == 1000 and 2.0e5 / steps == 200.0)
q = 2.1 / 0.3
steps2 = math.ceil(q)
check("P5-32 Eq. (5.16) floating-point caveat: t_end = 2.1 s, dt_in = 0.3 s gives 2.1/0.3 = 7.000000000000001, N_t = 8, dt = 0.2625 s",
      steps2 == 8 and q > 7.0, f"quotient {q!r}, dt used {2.1 / steps2!r}")

# ---------------------------------------------------------------- 5.11 Kahan
# add 1e-16 ten thousand times to 1.0
inc = 1.0e-16
naive = 1.0
s_k, comp = 1.0, 0.0
for _ in range(10000):
    naive += inc
    yk = inc - comp
    tk = s_k + yk
    comp = (tk - s_k) - yk
    s_k = tk
exact = Fraction(1) + 10000 * Fraction(inc)
check("P5-33 Kahan recurrence (5.23): 1 + 10^4 x 1e-16; plain sum stays 1.0, compensated sum keeps the increments",
      naive == 1.0 and abs(Fraction(s_k) - exact) / exact < 2.3e-16,
      f"plain {naive!r}, Kahan {s_k!r}, exact {float(exact)!r}")

print("ALL PASS" if all(results) else "SOME FAIL")
sys.exit(0 if all(results) else 1)
