"""STRATO v1.0.0 implementation inventory, Code B (src/numerical/finite_volume.rs) and
the Code A helpers that wrap it (src/postprocessing/comparison.rs).

Checks that the formulas listed in 00_implementation_inventory.md are what the code
computes and that the claimed properties hold. Symbolic checks use SymPy; numerical
checks re-implement the Rust arithmetic in NumPy on the synthetic five-layer input.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/v1_0_0_inventory_codeB.py
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


r, a, b, D, DP, DE, A, dP, dE = sp.symbols("r a b D D_P D_E A delta_P delta_E", positive=True)

# B1 shell volume: integral of 4 pi r^2 dr from a to b
V = sp.integrate(4 * sp.pi * r**2, (r, a, b))
check("B1 shell volume = 4 pi/3 (b^3 - a^3)", sp.simplify(V - sp.Rational(4, 3) * sp.pi * (b**3 - a**3)) == 0)

# B2 volume centroid: int r dV / int dV
rc = sp.integrate(r * 4 * sp.pi * r**2, (r, a, b)) / V
check("B2 centroid = 3/4 (b^4 - a^4)/(b^3 - a^3)",
      sp.simplify(rc - sp.Rational(3, 4) * (b**4 - a**4) / (b**3 - a**3)) == 0)
check("B2 centroid a=0 limit = 3b/4", sp.simplify(rc.subs(a, 0) - sp.Rational(3, 4) * b) == 0)
check("B2 centroid lies strictly inside (a, b) (numeric spot check)",
      all(a_ < float(rc.subs({a: a_, b: b_})) < b_ for a_, b_ in [(1e-6, 2e-6), (0.1, 0.11), (1.0, 3.0)]))

# B3 series conductance: eliminate C_I from D_P (C_P - C_I)/dP = D_E (C_I - C_E)/dE
CP, CE, CI = sp.symbols("C_P C_E C_I")
CIs = sp.solve(sp.Eq(DP * (CP - CI) / dP, DE * (CI - CE) / dE), CI)[0]
J = DP * (CP - CIs) / dP
G = A / (dP / DP + dE / DE)
check("B3 A J = G (C_P - C_E), G = A/(dP/DP + dE/DE)", sp.simplify(A * J - G * (CP - CE)) == 0)
check("B3 equal D reduces to A D/(dP + dE)", sp.simplify(G.subs(DE, DP) - A * DP / (dP + dE)) == 0)

# B4 outer conductance: half-cell to Dirichlet c=0 at R: A D (C_N - 0)/(R - r_N)
# (definition, nothing to eliminate); checked numerically below against Rust output.

# B5 subtraction-free pivot equals textbook Thomas pivot
v, gw, gwp, gep, ge, eprev = sp.symbols("v g_W g_W_prev g_E_prev g_E e_prev", positive=True)
den_prev = eprev + gep  # den_{i-1} = e_{i-1} + ge_{i-1}
textbook = (v + gw + ge) - gw * gep / den_prev  # b_i - a_i c'_{i-1}, a_i = -gw, c'_{i-1} = -ge_{i-1}/den_{i-1}
subfree = v + gw * eprev / (eprev + gw) + ge
check("B5 subtraction-free pivot = textbook pivot when g_W(i) = g_E(i-1)",
      sp.simplify((textbook - subfree).subs(gep, gw)) == 0)
# forward sweep: textbook d'_i = (rhs_i - a_i d'_{i-1})/den_i = (rhs_i + gw d'_{i-1})/den_i  (identical form)
# back substitution: x_i = d'_i - c'_i x_{i+1} = d'_i + (ge_i/den_i) x_{i+1}


# ---------------- numerical replica on the synthetic input ----------------
R = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4]
Dl = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13]


def mesh(ncell):
    left, right, dif, mat = [], [], [], []
    inner = 0.0
    for m, (Ro, d) in enumerate(zip(R, Dl)):
        w = (Ro - inner) / ncell
        for k in range(ncell):
            left.append(inner + k * w)
            right.append(Ro if k + 1 == ncell else inner + (k + 1) * w)
            dif.append(d)
            mat.append(m)
        inner = Ro
    left, right, dif = map(np.array, (left, right, dif))
    vol = 4.0 * math.pi / 3.0 * (right**3 - left**3)
    cen = np.where(left == 0.0, 0.75 * right, 0.75 * (right**4 - left**4) / np.where(left == 0, 1, right**3 - left**3))
    face = right[:-1]
    g = 4.0 * math.pi * face**2 / ((face - cen[:-1]) / dif[:-1] + (cen[1:] - face) / dif[1:])
    gR = 4.0 * math.pi * R[-1] ** 2 * dif[-1] / (R[-1] - cen[-1])
    return vol, cen, g, gR, np.array(mat)


def solve(v, gw, ge, rhs):
    n = len(v)
    den = np.zeros(n)
    dp = np.zeros(n)
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


def run(ncell, dt_in, t_end):
    vol, cen, g, gR, mat = mesh(ncell)
    n = len(vol)
    c = np.where(mat == 0, 1.0, 0.0)
    m0 = float(np.sum(vol * c))
    steps = math.ceil(t_end / dt_in)
    dt = t_end / steps
    gw = np.concatenate(([0.0], g))
    ge = np.concatenate((g, [gR]))
    rel, comp, worst = 0.0, 0.0, 0.0
    for _ in range(steps):
        v = vol / dt
        c = solve(v, gw, ge, v * c)
        inc = dt * gR * c[-1]
        y = inc - comp
        t = rel + y
        comp = (t - rel) - y
        rel = t
        worst = max(worst, abs(np.sum(vol * c) + rel - m0) / m0)
    return rel / m0, m0, worst, c


vol, cen, g, gR, mat = mesh(40)
gw = np.concatenate(([0.0], g))
ge = np.concatenate((g, [gR]))
bound = float(np.min(vol / (gw + ge)))
check("B15 explicit_dt_bound = min V/(gW+gE) reproduces main-run 7.812107958751e-3 s",
      abs(bound - 7.812107958751e-3) / 7.812107958751e-3 < 1e-11, f"{bound:.12e}")
m0_formula = 4.0 * math.pi / 3.0 * R[0] ** 3 * 1.0
check("B11 M0 = sum V_j c0 over kernel = 4 pi R1^3 c0/3 = 3.351032163829113e-11 mol",
      abs(float(np.sum(vol[mat == 0])) - m0_formula) / m0_formula < 1e-14, f"{m0_formula:.15e}")

# M-matrix / positivity: explicit Forward Euler with dt <= bound keeps c >= 0, dt slightly above may not
n = len(vol)
Amat = np.diag(gw + ge) - np.diag(g, 1) - np.diag(g, -1)
Ms = np.diag(1 / np.sqrt(vol)) @ Amat @ np.diag(1 / np.sqrt(vol))
mu_max = float(np.max(np.linalg.eigvalsh(Ms)))
check("B15 positivity bound <= L2 stability limit 2/mu_max (bound is the stricter one)",
      bound <= 2.0 / mu_max, f"bound={bound:.6e}, 2/mu_max={2/mu_max:.6e}")
dt = 200.0
Mbe = np.diag(vol / dt) + Amat
offdiag_ok = np.all(Mbe - np.diag(np.diag(Mbe)) <= 0)
inv = np.linalg.inv(Mbe)
check("B8 BE matrix is a Z-matrix, strictly diagonally dominant, inverse >= 0 (M-matrix)",
      offdiag_ok and np.all(np.diag(Mbe) > np.sum(np.abs(Mbe), 1) - np.abs(np.diag(Mbe))) and inv.min() >= 0)

F, m0, worst, cfin = run(40, 200.0, 2.0e5)
check("B14 replica F(2e5 s) on 200 cells equals main-run 4.324286068785e-1",
      abs(F - 4.324286068785e-1) < 5e-13, f"{F:.13e}")
check("B13 replica relative balance residual <= 1e-12 (v1.0 reports <= 2.8e-13)", worst < 1e-12, f"{worst:.3e}")

# BE column sums: sum over rows of (V/dt + A) x = sum V/dt x_old - gR x_N  => exact discrete balance
x_old = np.random.default_rng(1).random(n)
x_new = np.linalg.solve(Mbe, vol / dt * x_old)
lhs = np.sum(vol * x_new) + dt * gR * x_new[-1]
check("B13 summed BE equations give M^{n+1} + dt G_R C_N^{n+1} = M^n",
      abs(lhs - np.sum(vol * x_old)) / np.sum(vol * x_old) < 1e-12)

# A2 Richardson: coefficients (8, -6, 1)/3 for dt/4, dt/2, dt cancel orders 1 and 2
h = sp.symbols("h")
a1, a2, a3, F0 = sp.symbols("a1 a2 a3 F0")
Fh = lambda s: F0 + a1 * s + a2 * s**2 + a3 * s**3
comb = sp.expand((8 * Fh(h / 4) - 6 * Fh(h / 2) + Fh(h)) / 3)
check("A2 (8F(dt/4) - 6F(dt/2) + F(dt))/3 = F0 + O(dt^3)",
      sp.simplify(comb - F0 - sp.Rational(1, 8) * a3 * h**3) == 0, f"remainder {sp.factor(comb - F0)}")

# A4 study cost: actual work of time_extrapolated_release is (4 + 2 + 1) run_cost, code uses 1.75
steps = lambda dt_: math.ceil(2.0e5 / dt_)
actual = sum(steps(200.0 * m) for m in (0.25, 0.5, 1.0)) / steps(200.0)
check("A4 actual cell-steps of time_extrapolated_release = 7 x run_cost; code assumes 1.75 x",
      actual == 7.0 and actual / 1.75 == 4.0, f"actual factor {actual}, ratio to code {actual/1.75}")

# B9 claim in code comment (finite_volume.rs lines 126 to 131, 377) and Phase 1 Section 5.6:
# the textbook Thomas pivot loses digits at large Fourier number. Re-run the unit-test case
# conservation_holds_at_very_large_fourier_number with both eliminations.
def thomas(v, gw, ge, rhs):
    n = len(v)
    cp = np.zeros(n)
    dp = np.zeros(n)
    for i in range(n):
        bi = v[i] + gw[i] + ge[i]
        den = bi - (gw[i] * cp[i - 1] if i else 0.0)
        cp[i] = ge[i] / den
        dp[i] = (rhs[i] + (gw[i] * dp[i - 1] if i else 0.0)) / den
    x = np.zeros(n)
    x[-1] = dp[-1]
    for i in range(n - 2, -1, -1):
        x[i] = dp[i] + cp[i] * x[i + 1]
    return x


def drift(solver):
    Rt = [1e-4, 2e-4, 3e-4, 4e-4, 5e-4]
    left, right = [], []
    inner = 0.0
    for Ro in Rt:
        w = (Ro - inner) / 50
        for k in range(50):
            left.append(inner + k * w)
            right.append(Ro if k == 49 else inner + (k + 1) * w)
        inner = Ro
    left, right = np.array(left), np.array(right)
    vol = 4 * math.pi / 3 * (right**3 - left**3)
    cen = np.where(left == 0, 0.75 * right, 0.75 * (right**4 - left**4) / np.where(left == 0, 1, right**3 - left**3))
    g = 4 * math.pi * right[:-1] ** 2 / ((right[:-1] - cen[:-1]) / 1e-10 + (cen[1:] - right[:-1]) / 1e-10)
    gR = 4 * math.pi * 5e-4**2 * 1e-10 / (5e-4 - cen[-1])
    gw_ = np.concatenate(([0.0], g))
    ge_ = np.concatenate((g, [gR]))
    c = np.where(np.arange(250) < 50, 1.0, 0.0)
    m0 = float(np.sum(vol * c))
    rel, comp, worst = 0.0, 0.0, 0.0
    dt_ = 1.0e4
    for _ in range(100):
        v_ = vol / dt_
        c = solver(v_, gw_, ge_, v_ * c)
        y = dt_ * gR * c[-1] - comp
        t = rel + y
        comp = (t - rel) - y
        rel = t
        worst = max(worst, abs(np.sum(vol * c) + rel - m0) / m0)
    return worst


d_sf, d_tb = drift(solve), drift(thomas)
check("B9 subtraction-free elimination keeps relative drift < 1e-12 at Fo = 2.5e5", d_sf < 1e-12, f"{d_sf:.2e}")
print(f"INFO  textbook Thomas drift on the same case = {d_tb:.2e} (claim in docs: about 1e-9; code comment: ~6 digits lost)")

# B10 dt actually used: t_end / ceil(t_end / dt_in)
check("B10 dt rule: dt_in = 300, t_end = 1000 gives 4 steps of 250 s",
      math.ceil(1000 / 300) == 4 and 1000 / math.ceil(1000 / 300) == 250.0)

print("ALL PASS" if all(ok_all) else "SOME FAIL")
sys.exit(0 if all(ok_all) else 1)
