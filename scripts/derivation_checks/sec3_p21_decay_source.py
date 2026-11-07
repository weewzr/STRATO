"""STRATO Phase 2.1 checks, part 1: decay law, source terms, precursors,
activation products and parent-daughter chains (equations P21.5 to P21.23).

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec3_p21_decay_source.py
Writes nothing to disk. Prints one PASS/FAIL line per check.
"""
import math
import sys

sys.dont_write_bytecode = True
import sympy as sp

results = []


def check(name, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")
    results.append(bool(ok))


t, s, lam, N0, T12 = sp.symbols("t s lambda N_0 T_half", positive=True)

# ---------------------------------------------------------------------------
# (P21.5)-(P21.7) first-order decay law, half-life, mean life
# ---------------------------------------------------------------------------
N = sp.Function("N")
sol = sp.dsolve(sp.Eq(N(t).diff(t), -lam * N(t)), N(t), ics={N(0): N0}).rhs
check("P21.6 dN/dt=-lambda N gives N0 exp(-lambda t)", sp.simplify(sol - N0 * sp.exp(-lam * t)) == 0)
th = sp.solve(sp.Eq(N0 * sp.exp(-lam * T12), N0 / 2), T12)[0]
check("P21.7 half-life T = ln2/lambda", sp.simplify(th - sp.log(2) / lam) == 0, f"T = {th}")
mean_life = sp.integrate(t * lam * sp.exp(-lam * t), (t, 0, sp.oo))
check("P21.7 mean life = 1/lambda", sp.simplify(mean_life - 1 / lam) == 0)

# ENSDF adopted half-lives (audit 07, Section 2; LiveChart 1 y = 31 556 925.97 s)
YEAR = 31556925.97
ensdf = {  # nuclide: (T1/2 value, unit seconds per unit, lambda quoted in audit 07, s^-1)
    "Cs-137": (30.08, YEAR, 7.302187e-10),
    "Sr-90": (28.91, YEAR, 7.597709e-10),
    "Ag-110m": (249.83, 86400.0, 3.211198e-8),
    "Cs-134": (2.0652, YEAR, 1.063576e-8),
    "Eu-154": (8.592, YEAR, 2.556445e-9),
}
for nuc, (val, unit, lam_audit) in ensdf.items():
    lam_calc = math.log(2) / (val * unit)
    rel = abs(lam_calc / lam_audit - 1)
    check(f"P21.7 lambda({nuc}) = ln2/T1/2 reproduces audit 07", rel < 2e-6,
          f"lambda = {lam_calc:.6e} s^-1, rel diff {rel:.1e}")

# ---------------------------------------------------------------------------
# (P21.8) unit check of S = Fdot y / N_A
# ---------------------------------------------------------------------------
from sympy.physics import units as u

Fdot_u = 1 / (u.m ** 3 * u.s)          # fissions m^-3 s^-1 (fission is a count)
NA_u = 6.02214076e23 / u.mol           # SI defining constant (count per mol)
S_u = Fdot_u * 1 / NA_u                # yield is a count per fission
check("P21.8 S = Fdot y / N_A has units mol m^-3 s^-1",
      sp.simplify(S_u / (u.mol / (u.m ** 3 * u.s))).is_number)

# ---------------------------------------------------------------------------
# (P21.11)-(P21.12) precursor delay for a single short-lived precursor
# ---------------------------------------------------------------------------
lp, yI, yp, F = sp.symbols("lambda_p y_I y_p Fdot", positive=True)
Np = sp.Function("N_p")
Np_sol = sp.dsolve(sp.Eq(Np(t).diff(t), yp * F - lp * Np(t)), Np(t), ics={Np(0): 0}).rhs
birth = yI * F + lp * Np_sol                       # product birth rate
B = sp.integrate(birth.subs(t, s), (s, 0, t))      # cumulative product born
yC = yI + yp
target = yC * F * t - yp * F * (1 - sp.exp(-lp * t)) / lp
check("P21.12 cumulative birth = yC F t - yp F (1-exp(-lp t))/lp", sp.simplify(B - target) == 0)
lag = sp.limit(sp.simplify((yC * F * t - B) / (yC * F)), t, sp.oo)
check("P21.12 asymptotic lag = (yp/yC)/lambda_p <= 1/lambda_p", sp.simplify(lag - yp / (yC * lp)) == 0, f"lag = {lag}")
t99 = sp.solve(sp.Eq(1 - sp.exp(-lp * t), sp.Rational(99, 100)), t)[0]
check("P21.12 t99 = log2(100) T1/2 = 6.644 T1/2",
      abs(float(t99.subs(lp, math.log(2))) - math.log2(100)) < 1e-12,
      f"factor {math.log2(100):.4f}")
for name, th_s, quoted_h in [("Xe-137 (3.818 min)", 3.818 * 60, 0.42), ("Rb-90m (258 s)", 258.0, 0.48),
                             ("Xe-133 (5.2475 d)", 5.2475 * 86400, 34.9 * 24)]:
    t99_h = math.log2(100) * th_s / 3600
    check(f"P21.12 t99 for {name} matches audit 07", abs(t99_h / quoted_h - 1) < 0.02, f"t99 = {t99_h:.3f} h")

# ---------------------------------------------------------------------------
# (P21.13)-(P21.16) activation product from a fission-produced parent of the
# same element (same D in every layer, so same eigenfunctions, O = I)
# modal equations: a_P' = -kP a_P + sP ; a_A' = -kA a_A + f sigma phi a_P
# ---------------------------------------------------------------------------
kP, kA, sP, fsp = sp.symbols("kappa_P kappa_A s_P f_sigma_phi", positive=True)
aP = sP / kP * (1 - sp.exp(-kP * t))
aA = fsp * sP / kP * ((1 - sp.exp(-kA * t)) / kA - (sp.exp(-kP * t) - sp.exp(-kA * t)) / (kA - kP))
check("P21.15 parent mode solves a_P' = -kP a_P + sP, a_P(0)=0",
      sp.simplify(aP.diff(t) + kP * aP - sP) == 0 and aP.subs(t, 0) == 0)
check("P21.16 activation mode solves a_A' = -kA a_A + f sigma phi a_P, a_A(0)=0",
      sp.simplify(aA.diff(t) + kA * aA - fsp * aP) == 0 and sp.simplify(aA.subs(t, 0)) == 0)
check("P21.16 activation mode steady state = f sigma phi sP/(kP kA)",
      sp.simplify(sp.limit(aA, t, sp.oo) - fsp * sP / (kP * kA)) == 0)
# early-time growth is quadratic in t (two-step production)
ser = sp.series(aA, t, 0, 3).removeO()
check("P21.16 activation mode grows as f sigma phi sP t^2/2 at small t",
      sp.simplify(ser - fsp * sP * t ** 2 / 2) == 0)

# ---------------------------------------------------------------------------
# (P21.17)-(P21.22) parent-daughter chain with different D: overlap coupling
# a_D' = -K_D a_D + b lambda_p O a_P, a_P = exp(-K_P t) a_P(0)  (2 x 2 modes)
# ---------------------------------------------------------------------------
b, lpar = sp.symbols("b lambda_par", positive=True)
kP1, kP2, kD1, kD2 = sp.symbols("kP1 kP2 kD1 kD2", positive=True)
O = sp.Matrix(2, 2, sp.symbols("O11 O12 O21 O22"))
aP0 = sp.Matrix(sp.symbols("aP1 aP2"))
aD0 = sp.Matrix(sp.symbols("aD1 aD2"))
kPv, kDv = [kP1, kP2], [kD1, kD2]
aPt = sp.Matrix([aP0[m] * sp.exp(-kPv[m] * t) for m in range(2)])
aDt = sp.Matrix([
    aD0[n] * sp.exp(-kDv[n] * t)
    + b * lpar * sum(O[n, m] * aP0[m] * (sp.exp(-kPv[m] * t) - sp.exp(-kDv[n] * t)) / (kDv[n] - kPv[m])
                     for m in range(2))
    for n in range(2)])
res = (aDt.diff(t) + sp.diag(*kDv) * aDt - b * lpar * O * aPt).applyfunc(sp.simplify)
check("P21.21 Bateman-diffusion modal solution satisfies coupled ODE", res == sp.zeros(2, 1))
check("P21.21 initial condition a_D(0) recovered", (aDt.subs(t, 0) - aD0).applyfunc(sp.simplify) == sp.zeros(2, 1))
# with O = I and one mode this is the classical two-member Bateman solution
bat = aDt[0].subs({O[0, 0]: 1, O[0, 1]: 0, aD0[0]: 0})
classical = b * lpar * aP0[0] * (sp.exp(-kP1 * t) - sp.exp(-kD1 * t)) / (kD1 - kP1)
check("P21.22 O = I reduces to two-member Bateman per mode", sp.simplify(bat - classical) == 0)

# ---------------------------------------------------------------------------
# (P21.23) irradiation then post-irradiation: amplitudes carry over at t_irr
# ---------------------------------------------------------------------------
k, sn, ti = sp.symbols("k s_n t_irr", positive=True)
a_irr = sn / k * (1 - sp.exp(-k * t))
a_post = a_irr.subs(t, ti) * sp.exp(-k * (t - ti))
check("P21.23 post-irradiation amplitude continuous at t_irr", sp.simplify(a_irr.subs(t, ti) - a_post.subs(t, ti)) == 0)
check("P21.23 post-irradiation amplitude solves source-free ODE", sp.simplify(a_post.diff(t) + k * a_post) == 0)

print(f"\nSUMMARY: {sum(results)}/{len(results)} PASS")
sys.exit(0 if all(results) else 1)
