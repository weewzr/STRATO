"""STRATO v1.0.0, Full Mathematical Derivation, Part IV: symbolic checks.

Checks the algebra of Part IV (Eqs. 4.1 to 4.31) with SymPy and small numerical
tests. Writes nothing. Prints one PASS/FAIL line per check and a summary.

Run:
    C:/Y3S1/Nuclear/STRATO/.venv/Scripts/python.exe -I -X utf8 \
        scripts/derivation_checks/v1_0_0_p4_separation.py
"""

import math
import random

import numpy as np
import sympy as sp

results = []


def check(name, ok, info=""):
    results.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}" + (f"  ({info})" if info else ""))


r, t, s = sp.symbols("r t s", positive=True)
D, Lam, k, L, R, R1, Rm, c0 = sp.symbols("D Lambda k L R R_1 R_m c_0", positive=True)
a, b, q, u0 = sp.symbols("a b q u_0", real=True)
phi = sp.Function("phi")
u = sp.Function("u")

# 4.1, 4.2: product form in the layer equation (2.7)
c = phi(r) * sp.exp(-Lam * t)
pde = sp.diff(c, t) - D / r**2 * sp.diff(r**2 * sp.diff(c, r), r)
ode = D / r**2 * sp.diff(r**2 * sp.diff(phi(r), r), r) + Lam * phi(r)
check("4.1-4.2 c = phi e^{-Lambda t} turns (2.7) into -e^{-Lambda t} x (4.2)",
      sp.simplify(pde + sp.exp(-Lam * t) * ode) == 0)

# 4.3: derivative of phi = u/r
ph = u(r) / r
check("4.3 phi' = (u' - u/r)/r", sp.simplify(sp.diff(ph, r) - (sp.diff(u(r), r) - u(r) / r) / r) == 0)
check("4.3 step: r^2 phi' = r u' - u", sp.simplify(r**2 * sp.diff(ph, r) - (r * sp.diff(u(r), r) - u(r))) == 0)
check("4.3 step: (r^2 phi')' = r u''", sp.simplify(sp.diff(r**2 * sp.diff(ph, r), r) - r * sp.diff(u(r), r, 2)) == 0)
check("4.4 (1/r^2)(r^2 phi')' = u''/r",
      sp.simplify(sp.diff(r**2 * sp.diff(ph, r), r) / r**2 - sp.diff(u(r), r, 2) / r) == 0)
check("4.4 expanded form phi'' + (2/r) phi' = u''/r",
      sp.simplify(sp.diff(ph, r, 2) + 2 / r * sp.diff(ph, r) - sp.diff(u(r), r, 2) / r) == 0)
lhs44 = sp.simplify(r * (D / r**2 * sp.diff(r**2 * sp.diff(ph, r), r) + Lam * ph) / D)
check("4.4 (4.2) times r/D equals u'' + (Lambda/D) u",
      sp.simplify(lhs44 - (sp.diff(u(r), r, 2) + Lam / D * u(r))) == 0)

# 4.5, 4.6: one layer
kk = sp.sqrt(Lam / D)
us = a * sp.cos(k * s) + b * sp.sin(k * s)
check("4.5 a cos ks + b sin ks solves u'' + k^2 u = 0", sp.simplify(sp.diff(us, s, 2) + k**2 * us) == 0)
bb = q / (D * k)
us2 = u0 * sp.cos(k * s) + bb * sp.sin(k * s)
check("4.5 a = u_in and b = q_in/(D k) give u(0) = u_in, D u'(0) = q_in",
      sp.simplify(us2.subs(s, 0) - u0) == 0 and sp.simplify(D * sp.diff(us2, s).subs(s, 0) - q) == 0)
T = sp.Matrix([[sp.cos(k * L), sp.sin(k * L) / (D * k)], [-D * k * sp.sin(k * L), sp.cos(k * L)]])
out = T * sp.Matrix([u0, q])
check("4.6 T row 1 equals u(L)", sp.simplify(out[0] - us2.subs(s, L)) == 0)
check("4.6 T row 2 equals D u'(L)", sp.simplify(out[1] - D * sp.diff(us2, s).subs(s, L)) == 0)
check("4.6 det T = 1", sp.simplify(T.det()) == 1)
L1, L2 = sp.symbols("L_1 L_2", positive=True)
T1, T2, T12 = (T.subs(L, x) for x in (L1, L2, L1 + L2))
check("4.6 T(L1 + L2) = T(L2) T(L1) (split a layer)", sp.simplify(sp.expand_trig(T2 * T1 - T12)) == sp.zeros(2, 2))
check("4.4 k = sqrt(Lambda/D) has units 1/m (Lambda 1/s, D m^2/s)",
      sp.simplify((sp.Symbol("sec", positive=True)**-1 / (sp.Symbol("m", positive=True)**2 / sp.Symbol("sec", positive=True)))**sp.Rational(1, 2)
                  - sp.Symbol("m", positive=True)**-1) == 0)

# 4.7, 4.8: interface
Dm, Dp, qm = sp.symbols("D_minus D_plus q_minus", positive=True)
qp = qm + (Dp - Dm) * u0 / Rm
check("4.8 jump keeps D(u' - u/r): q+ - D+ u/R = q- - D- u/R", sp.simplify((qp - Dp * u0 / Rm) - (qm - Dm * u0 / Rm)) == 0)
Qm = sp.Matrix([[1, 0], [(Dp - Dm) / Rm, 1]])
check("4.8 interface matrix Q has det 1 and reduces to I when D+ = D-",
      sp.simplify(Qm.det()) == 1 and Qm.subs(Dp, Dm) == sp.eye(2))
# physical meaning: D(u' - u/r) = r D phi' = -r J_r e^{Lambda t}
check("4.7 D(u' - u/r) = r D phi'", sp.simplify(D * (sp.diff(u(r), r) - u(r) / r) - r * D * sp.diff(ph, r)) == 0)

# 4.9: centre
kb = sp.Symbol("k_1", positive=True)
phi_s = b * sp.sin(kb * r) / r
check("4.9 sin branch: phi(0) = b k_1 (finite)", sp.limit(phi_s, r, 0) == b * kb)
check("4.9 sin branch: phi'(0) = 0 (symmetry, (3.1))", sp.limit(sp.diff(phi_s, r), r, 0) == 0)
check("4.9 cos branch: cos(k_1 r)/r unbounded at r = 0", sp.limit(sp.cos(kb * r) / r, r, 0, "+") == sp.oo)

# 4.11: phase form
A = sp.sqrt(a**2 + b**2)
th = sp.Symbol("theta")
expr = A * (sp.sin(k * s) * sp.cos(th) + sp.cos(k * s) * sp.sin(th))
expr = expr.subs({sp.sin(th): a / A, sp.cos(th): b / A})
check("4.11 A sin(ks + theta0) = a cos ks + b sin ks with sin theta0 = a/A, cos theta0 = b/A",
      sp.simplify(expr - us) == 0)
rng = random.Random(11)
ok = True
for _ in range(2000):
    aa, bv = rng.uniform(-1, 1), rng.uniform(-1, 1)
    th0 = math.atan2(aa, bv)
    ss = rng.uniform(0, 10)
    ok &= abs(math.hypot(aa, bv) * math.sin(ss + th0) - (aa * math.cos(ss) + bv * math.sin(ss))) < 1e-14
check("4.11 atan2(a, b) gives the phase (2000 random a, b, ks)", ok)

# 4.12: zero count by phase against dense sign-change count
ok, worst = True, 0
for _ in range(2000):
    aa, bv = rng.uniform(-1, 1), rng.uniform(-1, 1)
    kv, Lv = rng.uniform(0.1, 30.0), rng.uniform(0.05, 2.0)
    th0 = math.atan2(aa, bv)
    formula = math.floor((th0 + kv * Lv) / math.pi) - math.floor(th0 / math.pi)
    sg = np.linspace(0.0, Lv, 200001)[1:]
    uu = aa * np.cos(kv * sg) + bv * np.sin(kv * sg)
    brute = int(np.sum(np.sign(uu[1:]) * np.sign(uu[:-1]) < 0)) + int(uu[0] * aa < 0 and aa != 0)
    # zero exactly at s = L is counted by the formula; a sign test needs it explicitly
    if abs(uu[-1]) < 1e-12:
        brute += 1
    ok &= formula == brute
check("4.12 floor((theta0 + kL)/pi) - floor(theta0/pi) = zeros of u in (0, L] (2000 random layers)", ok)

# 4.17: Lagrange identity and Rayleigh identity
p = sp.Function("p")
fm, fn = sp.Function("phi_m"), sp.Function("phi_n")
lhs = fn(r) * sp.diff(p(r) * sp.diff(fm(r), r), r) - fm(r) * sp.diff(p(r) * sp.diff(fn(r), r), r)
rhs = sp.diff(p(r) * (fn(r) * sp.diff(fm(r), r) - fm(r) * sp.diff(fn(r), r)), r)
check("4.17 Lagrange identity phi_n (p phi_m')' - phi_m (p phi_n')' = [p (phi_n phi_m' - phi_m phi_n')]'",
      sp.simplify(sp.expand(lhs - rhs)) == 0)
check("4.17 Rayleigh step (p phi' phi)' = (p phi')' phi + p phi'^2",
      sp.simplify(sp.diff(p(r) * sp.diff(fm(r), r) * fm(r), r)
                  - (sp.diff(p(r) * sp.diff(fm(r), r), r) * fm(r) + p(r) * sp.diff(fm(r), r)**2)) == 0)
check("4.17 weight: r^2 phi_m phi_n = u_m u_n and r^2 phi'^2 = (u' - u/r)^2",
      sp.simplify(r**2 * (u(r) / r)**2 - u(r)**2) == 0
      and sp.simplify(r**2 * sp.diff(u(r) / r, r)**2 - (sp.diff(u(r), r) - u(r) / r)**2) == 0)

# 4.18 to 4.22: projection and weights
I1, In, Nn, sc = sp.symbols("I_1 I_n N_n sigma", positive=True)
An = c0 * I1 / Nn
M0 = sp.Rational(4, 3) * sp.pi * R1**3 * c0
w = sp.simplify(4 * sp.pi * An * In / M0)
check("4.22 4 pi A_n I_n / M^0 = 3 I_1 I_n / (R_1^3 N_n)", sp.simplify(w - 3 * I1 * In / (R1**3 * Nn)) == 0)
check("4.18 A_n u_n and w_n are unchanged when u_n -> sigma u_n",
      sp.simplify((c0 * sc * I1 / (sc**2 * Nn)) * sc - An) == 0
      and sp.simplify(3 * sc * I1 * sc * In / (R1**3 * sc**2 * Nn) - w) == 0)

# 4.25: closed-form layer integrals in the local coordinate
R0 = sp.Symbol("R_0", positive=True)
Nl = sp.integrate(us**2, (s, 0, L))
Nl_cf = (a**2 + b**2) * L / 2 + (a**2 - b**2) * sp.sin(2 * k * L) / (4 * k) + a * b * (1 - sp.cos(2 * k * L)) / (2 * k)
check("4.25 int_0^L u^2 ds closed form", sp.simplify(sp.expand_trig(Nl - Nl_cf)) == 0)
Ml = sp.integrate((R0 + s) * us, (s, 0, L))
Ml_cf = (R0 * (a * sp.sin(k * L) + b * (1 - sp.cos(k * L))) / k
         + a * (L * sp.sin(k * L) / k + (sp.cos(k * L) - 1) / k**2)
         + b * (-L * sp.cos(k * L) / k + sp.sin(k * L) / k**2))
check("4.25 int_0^L (R_{l-1} + s) u ds closed form", sp.simplify(Ml - Ml_cf) == 0)

# 4.29, 4.30: homogeneous sphere
n = sp.Symbol("n", integer=True, positive=True)
al = n * sp.pi / R
un = sp.sin(al * r)
check("4.29 u_n = sin(n pi r/R) vanishes at r = R", sp.simplify(un.subs(r, R)) == 0)
check("4.29 N_n = R/2", sp.simplify(sp.integrate(un**2, (r, 0, R)) - R / 2) == 0)
check("4.29 I_n = R^2 (-1)^(n+1)/(n pi)",
      sp.simplify(sp.integrate(r * un, (r, 0, R)) - R**2 * (-1)**(n + 1) / (n * sp.pi)) == 0)
check("4.29 I_1n = (sin(alpha R1) - alpha R1 cos(alpha R1))/alpha^2",
      sp.simplify(sp.integrate(r * un, (r, 0, R1)) - (sp.sin(al * R1) - al * R1 * sp.cos(al * R1)) / al**2) == 0)
wn_full = sp.simplify(3 * (R**2 * (-1)**(n + 1) / (n * sp.pi))**2 / (R**3 * R / 2))
check("4.30 R_1 = R gives w_n = 6/(n^2 pi^2)", sp.simplify(wn_full - 6 / (n**2 * sp.pi**2)) == 0)
check("4.30 sum_n 6/(n^2 pi^2) = 1 (completeness holds here)",
      sp.simplify(sp.summation(6 / (n**2 * sp.pi**2), (n, 1, sp.oo)) - 1) == 0)

# 4.31: Robin root equation
z, Bi, h = sp.symbols("z Bi h", positive=True)
phR = sp.sin(z * r / R) / r
robin = (-D * sp.diff(phR, r) - h * phR).subs(r, R)
target = (D / R**2) * sp.sin(z) * ((1 - z * sp.cot(z)) - h * R / D)
check("4.31 -D phi'(R) - h phi(R) = (D sin z / R^2)(1 - z cot z - Bi)", sp.simplify(robin - target) == 0)

# 4.24: Gauss-Legendre 16, code algorithm (SL 266-292)
def gl16_code():
    nn = 16
    xs, ws = [], []
    for i in range(nn):
        zz = math.cos((i + 0.75) / (nn + 0.5) * math.pi)
        dp = 1.0
        for _ in range(100):
            p_prev, pp = 1.0, zz
            for kk_ in range(2, nn + 1):
                pk = ((2 * kk_ - 1) * zz * pp - (kk_ - 1) * p_prev) / kk_
                p_prev, pp = pp, pk
            dp = nn * (zz * pp - p_prev) / (zz * zz - 1.0)
            dz = pp / dp
            zz -= dz
            if abs(dz) < 1e-15:
                break
        xs.append(zz)
        ws.append(2.0 / ((1.0 - zz * zz) * dp * dp))
    return np.array(xs), np.array(ws)


xc, wc = gl16_code()
xr, wr = np.polynomial.legendre.leggauss(16)
order = np.argsort(xc)
check("4.24 code nodes and weights equal numpy leggauss(16)",
      np.max(np.abs(xc[order] - xr)) < 1e-15 and np.max(np.abs(wc[order] - wr)) < 1e-14,
      f"max node diff {np.max(np.abs(xc[order] - xr)):.1e}, weight diff {np.max(np.abs(wc[order] - wr)):.1e}")
errs = [abs(np.sum(wc * xc**d) - (2.0 / (d + 1) if d % 2 == 0 else 0.0)) for d in range(32)]
e32 = abs(np.sum(wc * xc**32) - 2.0 / 33)
check("4.24 rule exact for degree <= 31, not for degree 32",
      max(errs) < 1e-14 and e32 > 1e-11, f"max error deg<=31 {max(errs):.1e}, deg 32 {e32:.1e}")
ok = True
for _ in range(10000):
    kl = rng.uniform(0.0, 500.0)
    panels = math.ceil(kl / (0.5 * math.pi)) + 1
    ok &= kl / panels < 0.5 * math.pi
check("4.24 panel rule gives phase k h < pi/2 per panel (10000 random kL)", ok)

# 4.26: truncation
check("4.26 t_valid = 40/Lambda_max = (40/45) t_min < t_min when uncapped; e^-40 = 4.25e-18",
      abs(math.exp(-40) - 4.248354255291589e-18) < 1e-30 and 40 / 45 < 1)

print(f"SUMMARY {sum(results)}/{len(results)} PASS")
print("ALL PASS" if all(results) else "SOME FAIL")
