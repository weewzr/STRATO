"""Phase 2.1 (section 4): exact steady states used as verification references.

Checks (SymPy, exact or high-precision):
  A. (P21.82)-(P21.84) multilayer steady state for a uniform kernel source, no decay,
     absorbing surface: ODE residual, centre regularity, interface continuity of c and of
     the flux, c(R) = 0, steady release = generation, closed form of the steady inventory
     and of the mean residence time tau_res.
  B. (P21.86) single-sphere steady state with decay (Booth): ODE, BC, R/B equals
     TECDOC-1674 Eq. (7.6) and the eigen-series sum 6/(n^2 pi^2 + a).
  C. (P21.85) multilayer steady state with source and decay, solved exactly as a 9x9
     linear system at 60 digits for synthetic data: interface conditions, steady balance
     Gen = lambda*M_ss + J_R, and the lambda -> 0 limit against part A.

Run: .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec4_p21_steady_state.py
Writes nothing to disk. Prints PASS/FAIL lines.
"""
import sympy as sp
import mpmath as mp

ok_all = True


def report(name, ok, detail=""):
    global ok_all
    ok_all &= bool(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


r = sp.symbols("r", positive=True)
S = sp.symbols("S", positive=True)            # kernel source, mol m^-3 s^-1
R = sp.symbols("R1:6", positive=True)         # R1..R5 (m), R[0] = R_1
D = sp.symbols("D1:6", positive=True)         # D1..D5 (m^2 s^-1)
lam = sp.symbols("lambda", positive=True)


def L_op(c, Di):
    """Spherical diffusion operator r^-2 d/dr (r^2 D dc/dr)."""
    return sp.diff(r**2 * Di * sp.diff(c, r), r) / r**2


# ---------------------------------------------------------------- A. no decay
K = S * R[0]**3 / 3           # steady flux constant: r^2 J = S R_1^3 / 3 for r > R_1


def Phi(i):
    """Sum_{m=i+1}^{5} (1/D_m)(1/R_{m-1} - 1/R_m), i = 1..5 (1-based)."""
    return sum((1 / D[m - 1]) * (1 / R[m - 2] - 1 / R[m - 1]) for m in range(i + 1, 6))


c = {}
for i in range(2, 6):
    c[i] = K * ((1 / D[i - 1]) * (1 / r - 1 / R[i - 1]) + Phi(i))
c[1] = S * (R[0]**2 - r**2) / (6 * D[0]) + K * Phi(1)

res = [sp.simplify(L_op(c[1], D[0]) + S)] + [sp.simplify(L_op(c[i], D[i - 1])) for i in range(2, 6)]
report("A1 ODE residual in all five layers (P21.82)", all(x == 0 for x in res), str(res))
report("A2 centre regularity dc/dr(0) = 0", sp.simplify(sp.diff(c[1], r).subs(r, 0)) == 0)
cont = [sp.simplify(c[i].subs(r, R[i - 1]) - c[i + 1].subs(r, R[i - 1])) for i in range(1, 5)]
report("A3 continuity of c at R_1..R_4", all(x == 0 for x in cont), str(cont))
flux = [sp.simplify(D[i - 1] * sp.diff(c[i], r).subs(r, R[i - 1])
                    - D[i] * sp.diff(c[i + 1], r).subs(r, R[i - 1])) for i in range(1, 5)]
report("A4 continuity of D dc/dr at R_1..R_4", all(x == 0 for x in flux), str(flux))
report("A5 absorbing surface c(R) = 0", sp.simplify(c[5].subs(r, R[4])) == 0)
JR = sp.simplify(-4 * sp.pi * R[4]**2 * D[4] * sp.diff(c[5], r).subs(r, R[4]))
Gen = sp.Rational(4, 3) * sp.pi * R[0]**3 * S
report("A6 steady release J_R = Gen = (4/3) pi R_1^3 S (P21.83)", sp.simplify(JR - Gen) == 0)

# steady inventory, direct integration versus the closed form (P21.84)
Rb = [0] + list(R)
Mss = sp.integrate(4 * sp.pi * r**2 * c[1], (r, 0, R[0]))
for i in range(2, 6):
    Mss += sp.integrate(4 * sp.pi * r**2 * c[i], (r, Rb[i - 1], Rb[i]))
Mss_closed = sp.Rational(4, 9) * sp.pi * S * (R[0]**5 / (5 * D[0])
                                              + R[0]**3 * sum((Rb[i]**2 - Rb[i - 1]**2) / (2 * D[i - 1]) for i in range(2, 6)))
report("A7 steady inventory closed form (P21.84)", sp.simplify(sp.expand(Mss - Mss_closed)) == 0)
tau = sp.simplify(Mss_closed / Gen)
tau_closed = R[0]**2 / (15 * D[0]) + sum((Rb[i]**2 - Rb[i - 1]**2) / (6 * D[i - 1]) for i in range(2, 6))
report("A8 tau_res = M_ss/Gen = R_1^2/(15 D_1) + sum (R_i^2-R_{i-1}^2)/(6 D_i)", sp.simplify(tau - tau_closed) == 0)
# single-material limit: all D equal, R_1 = R  -> R^2/(15 D) (TECDOC-1674 Eq. 7.3 asymptote)
Dh, Rh = sp.symbols("D R", positive=True)
tau1 = tau_closed.subs({D[i]: Dh for i in range(5)}).subs({R[i]: Rh for i in range(5)})
report("A9 one material, R_1 = R: tau_res = R^2/(15 D)", sp.simplify(tau1 - Rh**2 / (15 * Dh)) == 0)

# ---------------------------------------------------------------- B. Booth, single sphere with decay
Dd, Rs = sp.symbols("D R", positive=True)
k = sp.sqrt(lam / Dd)
cB = (S / lam) * (1 - Rs * sp.sinh(k * r) / (r * sp.sinh(k * Rs)))
resB = sp.simplify(L_op(cB, Dd) - lam * cB + S)
report("B1 Booth steady ODE D r^-2 (r^2 c')' - lam c + S = 0 (TECDOC-1674 Eq. 7.5)", resB == 0)
report("B2 c(R) = 0", sp.simplify(cB.subs(r, Rs)) == 0)
report("B3 regular at r = 0 (finite limit)", sp.limit(cB, r, 0).is_finite)
RB = sp.simplify(-4 * sp.pi * Rs**2 * Dd * sp.diff(cB, r).subs(r, Rs) / (sp.Rational(4, 3) * sp.pi * Rs**3 * S))
x = sp.sqrt(lam * Rs**2 / Dd)                  # sqrt(lambda/D'), D' = D/R^2
RB76 = 3 / x * (sp.coth(x) - 1 / x)
vals = {Dd: sp.Rational(3, 10**15), Rs: sp.Rational(25, 10**5), lam: sp.Rational(688, 10**7)}
d76 = abs(sp.N((RB - RB76).subs(vals), 40))
report("B4 R/B equals TECDOC-1674 Eq. (7.6) (numeric, 40 digits)", d76 < 1e-35, f"|diff| = {float(d76):.2e}")
a = float((lam * Rs**2 / Dd).subs(vals))
mp.mp.dps = 30
series = mp.nsum(lambda n: 6 / (n**2 * mp.pi**2 + a), [1, mp.inf])
closed = 3 / mp.sqrt(a) * (mp.coth(mp.sqrt(a)) - 1 / mp.sqrt(a))
report("B5 sum 6/(n^2 pi^2 + a) = (7.6) closed form", abs(series - closed) < 1e-25, f"a = {a:.6g}, R/B = {mp.nstr(closed, 12)}")

# ---------------------------------------------------------------- C. multilayer, source and decay
mp.mp.dps = 60
radii = [mp.mpf(x) * mp.mpf("1e-6") for x in (250, 350, 390, 425, 465)]   # TECDOC-1674 Table 10.2 sensitivity geometry
Dv = [mp.mpf(x) for x in ("4e-14", "4e-13", "2e-14", "5e-15", "2e-14")]   # synthetic, m^2 s^-1
Sv = mp.mpf(1)


def ml_profile(lmb):
    """Exact steady state of D_i u'' - lam u + S r H(R_1 - r) = 0 with u = r c.
    Kernel: u = S r/lam + A1 sinh(k1 r); layer i>=2: u = A_i sinh(k_i s) + B_i cosh(k_i s),
    s = r - R_{i-1}. Unknowns [A1, A2, B2, ..., A5, B5] (9)."""
    k = [mp.sqrt(lmb / d) for d in Dv]

    def ufun(i, coef, rr):
        if i == 0:
            return Sv * rr / lmb + coef[0] * mp.sinh(k[0] * rr)
        A, B = coef[2 * i - 1], coef[2 * i]
        s = rr - radii[i - 1]
        return A * mp.sinh(k[i] * s) + B * mp.cosh(k[i] * s)

    def dufun(i, coef, rr):
        if i == 0:
            return Sv / lmb + coef[0] * k[0] * mp.cosh(k[0] * rr)
        A, B = coef[2 * i - 1], coef[2 * i]
        s = rr - radii[i - 1]
        return k[i] * (A * mp.cosh(k[i] * s) + B * mp.sinh(k[i] * s))

    # the conditions are linear in the 9 coefficients: build the matrix column by column
    def conds(coef):
        eqs = []
        for i in range(4):
            Ri = radii[i]
            eqs.append(ufun(i, coef, Ri) - ufun(i + 1, coef, Ri))
            eqs.append(Dv[i] * (dufun(i, coef, Ri) - ufun(i, coef, Ri) / Ri)
                       - Dv[i + 1] * (dufun(i + 1, coef, Ri) - ufun(i + 1, coef, Ri) / Ri))
        eqs.append(ufun(4, coef, radii[4]))
        return eqs

    zero = [mp.mpf(0)] * 9
    b = [-e for e in conds(zero)]
    M = mp.matrix(9, 9)
    for j in range(9):
        e = [mp.mpf(0)] * 9
        e[j] = mp.mpf(1)
        col = conds(e)
        for i in range(9):
            M[i, j] = col[i] + b[i]
    coef = mp.lu_solve(M, mp.matrix(b))
    coef = [coef[i] for i in range(9)]
    return k, coef, ufun, dufun, conds


lmb = mp.mpf("2e-8")
k, coef, ufun, dufun, conds = ml_profile(lmb)
resid = max(abs(e) for e in conds(coef))
report("C1 interface and surface conditions, multilayer with decay (P21.85)", resid < mp.mpf("1e-45"), f"max residual {mp.nstr(resid, 3)}")
# ODE residual of the layer forms: D u'' - lam u + S r H = 0 holds identically (check numerically by finite precision derivative)
rr = radii[2] - mp.mpf("1e-6")
ode = Dv[2] * mp.diff(lambda q: ufun(2, coef, q), rr, 2) - lmb * ufun(2, coef, rr)
report("C2 ODE residual inside IPyC (mp.diff)", abs(ode) < mp.mpf("1e-40") * abs(lmb * ufun(2, coef, rr)) + mp.mpf("1e-50"), mp.nstr(ode, 3))
# steady balance: Gen = lam * M_ss + J_R
Rb2 = [mp.mpf(0)] + radii
Mss = sum(mp.quad(lambda q: 4 * mp.pi * q * ufun(i, coef, q), [Rb2[i], Rb2[i + 1]]) for i in range(5))
JRv = -4 * mp.pi * radii[4]**2 * Dv[4] * (dufun(4, coef, radii[4]) - ufun(4, coef, radii[4]) / radii[4]) / radii[4]
Genv = 4 * mp.pi / 3 * radii[0]**3 * Sv
bal = abs(Genv - lmb * Mss - JRv) / Genv
report("C3 steady balance Gen = lam M_ss + J_R", bal < mp.mpf("1e-40"), f"rel {mp.nstr(bal, 3)}, R/B = {mp.nstr(JRv / Genv, 10)}")


# lambda -> 0: compare with part A evaluated at the same data
subsA = {S: 1}
subsA.update({R[i]: sp.Float(str(radii[i]), 60) for i in range(5)})
subsA.update({D[i]: sp.Float(str(Dv[i]), 60) for i in range(5)})
cA_mid = [sp.N(c[i + 1].subs(subsA).subs(r, sp.Float(str((Rb2[i] + Rb2[i + 1]) / 2), 60)), 50) for i in range(5)]
errs = []
for lsmall in (mp.mpf("1e-12"), mp.mpf("1e-14"), mp.mpf("1e-16")):
    k2, coef2, uf2, _, _ = ml_profile(lsmall)
    e = max(abs(uf2(i, coef2, (Rb2[i] + Rb2[i + 1]) / 2) / ((Rb2[i] + Rb2[i + 1]) / 2) - mp.mpf(str(cA_mid[i]))) / abs(mp.mpf(str(cA_mid[i])))
            for i in range(5))
    errs.append(e)
ratio1 = errs[0] / errs[1]
ratio2 = errs[1] / errs[2]
report("C4 lambda -> 0 limit equals part A, error O(lambda)", abs(ratio1 - 100) < 1 and abs(ratio2 - 100) < 1,
       f"rel errors {[mp.nstr(e, 3) for e in errs]}")

print("ALL PASS" if ok_all else "SOME CHECKS FAILED")
