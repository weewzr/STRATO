"""Section 1 (Phase 1, items 1 to 14): symbolic checks of every derived formula.

Run with:
    .venv/Scripts/python.exe -I -X utf8 sec1_symbolic_checks.py

Each check prints one PASS or FAIL line tagged with the equation number(s) it covers
in sec1_phase1_physics_and_analytical.md. Nothing is written to disk.

Notation (SI): r radius (m), t time (s), D diffusivity (m^2 s^-1), c concentration
(mol m^-3), u = r c (mol m^-2), Lambda eigenvalue (s^-1), k = sqrt(Lambda/D) (m^-1),
q = D u' (state variable used by Code C), R_1 kernel radius, R outer radius.
"""

import sympy as sp
import mpmath as mp

results = []


def report(tag, ok, detail=""):
    results.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {tag}  {detail}")


r, t, s, x, y, z = sp.symbols("r t s x y z", real=True)
D, Di, Dj, k, L, Lam = sp.symbols("D D_i D_j k L Lambda", positive=True)
lam_n, lam_m, R, R1, c0, a, b, u0, q0 = sp.symbols(
    "Lambda_n Lambda_m R R_1 c_0 a b u_0 q_0", positive=True
)

# --------------------------------------------------------------------------
# (P1.3) radial divergence: Cartesian Laplacian of c(|x|) equals r^-2 (r^2 c')'
c = sp.Function("c")
rho = sp.sqrt(x**2 + y**2 + z**2)
lap_cart = sum(sp.diff(c(rho), v, 2) for v in (x, y, z))
lap_sph = (sp.diff(r**2 * sp.diff(c(r), r), r) / r**2).subs(r, rho)
report("P1.3 spherical divergence", sp.simplify(lap_cart - lap_sph.doit()) == 0)

# (P1.4) area of sphere and volume element: integral of r^2 sin(theta) over angles
th, ph = sp.symbols("theta varphi", real=True)
area = sp.integrate(sp.integrate(r**2 * sp.sin(th), (th, 0, sp.pi)), (ph, 0, 2 * sp.pi))
report("P1.4 surface area 4 pi r^2", sp.simplify(area - 4 * sp.pi * r**2) == 0)

# (P1.2) shell volume from P1.4
V = sp.integrate(4 * sp.pi * r**2, (r, a, b))
report("P1.2 shell volume", sp.simplify(V - sp.Rational(4, 3) * sp.pi * (b**3 - a**3)) == 0)

# --------------------------------------------------------------------------
# (P1.7)-(P1.8) shell balance: d/dt int_a^b 4 pi r^2 c dr equals the face-flux
# difference for an exact separable solution c = sin(k r)/r exp(-D k^2 t)
cex = sp.sin(k * r) / r * sp.exp(-D * k**2 * t)
pde_res = sp.diff(cex, t) - D * sp.diff(r**2 * sp.diff(cex, r), r) / r**2
report("P1.8 PDE residual for sin(kr)/r e^{-Dk^2 t}", sp.simplify(pde_res) == 0)
lhs = sp.diff(sp.integrate(4 * sp.pi * r**2 * cex, (r, a, b)), t)
face = lambda rr: (4 * sp.pi * rr**2 * D * sp.diff(cex, r)).subs(r, rr)
report("P1.7 shell balance (integral form)", sp.simplify(lhs - (face(b) - face(a))) == 0)

# (P1.9) u = r c turns the radial operator into u''
u = sp.Function("u")
cu = u(r) / r
res = sp.diff(r**2 * sp.diff(cu, r), r) / r**2 - sp.diff(u(r), r, 2) / r
report("P1.9 r^-2 (r^2 (u/r)')' = u''/r", sp.simplify(res) == 0)

# --------------------------------------------------------------------------
# (P1.12)-(P1.13) centre: cos(kr)/r carries a point flux, sin(kr)/r does not
flux_cos = sp.limit(4 * sp.pi * r**2 * D * sp.diff(sp.cos(k * r) / r, r), r, 0)
flux_sin = sp.limit(4 * sp.pi * r**2 * D * sp.diff(sp.sin(k * r) / r, r), r, 0)
dsin0 = sp.limit(sp.diff(sp.sin(k * r) / r, r), r, 0)
report(
    "P1.12 regular branch: zero centre flux and zero slope",
    flux_sin == 0 and dsin0 == 0 and sp.simplify(flux_cos + 4 * sp.pi * D) == 0,
    f"(cos branch flux -> {flux_cos})",
)

# --------------------------------------------------------------------------
# (P1.25)-(P1.26) interface: D(u' - u/r) continuous, q = D u'  =>  jump rule
qi, qj, uu, Ri = sp.symbols("q_i q_j u R_i", real=True)
sol = sp.solve(sp.Eq(qi - Di * uu / Ri, qj - Dj * uu / Ri), qj)[0]
report("P1.26 jump rule q_{i+1} = q_i + (D_{i+1}-D_i) u/R_i",
       sp.simplify(sol - (qi + (Dj - Di) * uu / Ri)) == 0)
# identity D phi' = D (u' - u/r)/r
phi_from_u = u(r) / r
report("P1.25 D phi' = D (u' - u/r)/r",
       sp.simplify(D * sp.diff(phi_from_u, r) - D * (sp.diff(u(r), r) - u(r) / r) / r) == 0)
# Pruefer pair: p phi' with p = r^2 D equals r (q - D u / r)
qf = D * sp.diff(u(r), r)
report("P1.29 r^2 D phi' = r (q - D u/r)",
       sp.simplify(r**2 * D * sp.diff(phi_from_u, r) - r * (qf - D * u(r) / r)) == 0)

# --------------------------------------------------------------------------
# (P1.27) layer transfer map
us = u0 * sp.cos(k * s) + q0 * sp.sin(k * s) / (D * k)
qs = -D * k * sp.sin(k * s) * u0 + sp.cos(k * s) * q0
ok = (sp.simplify(sp.diff(us, s, 2) + k**2 * us) == 0
      and sp.simplify(us.subs(s, 0) - u0) == 0
      and sp.simplify(D * sp.diff(us, s) - qs) == 0)
T = lambda l: sp.Matrix([[sp.cos(k * l), sp.sin(k * l) / (D * k)],
                         [-D * k * sp.sin(k * l), sp.cos(k * l)]])
L1, L2 = sp.symbols("L_1 L_2", positive=True)
comp = sp.simplify((T(L2) * T(L1) - T(L1 + L2)).applyfunc(sp.expand_trig))
ok = ok and comp == sp.zeros(2, 2) and sp.simplify(T(L).det() - 1) == 0
report("P1.27 transfer map solves ODE, det = 1, T(L2)T(L1) = T(L1+L2)", ok)

# (P1.28) layer-1 start (u, q) = (0, 1) gives u_1 = sin(k_1 r)/(D_1 k_1)
u1 = (0 * sp.cos(k * r) + 1 * sp.sin(k * r) / (D * k))
report("P1.28 kernel start state", sp.simplify(u1 - sp.sin(k * r) / (D * k)) == 0)

# (P1.29) phase form used by the Sturm count: a cos + b sin = A sin(ks + theta0) with
# theta0 = atan2(a, b). The earlier version of this check expanded A sin(ks + theta0) with
# sin(theta0) = a/A, cos(theta0) = b/A already substituted, which is a tautology and never
# tested atan2. Here sp.atan2 is used explicitly and evaluated in all four quadrants of (a, b).
aa, bb = sp.symbols("aa bb", real=True)
th0 = sp.atan2(aa, bb)
A = sp.sqrt(aa**2 + bb**2)
expr = sp.expand_trig(A * sp.sin(k * s + th0)) - (aa * sp.cos(k * s) + bb * sp.sin(k * s))
ok29 = True
for av, bv in [(1, 2), (-1, 2), (1, -2), (-1, -2), (0, 1), (0, -1), (3, 0), (-3, 0)]:
    ok29 = ok29 and sp.simplify(expr.subs({aa: av, bb: bv})) == 0
report("P1.29 a cos(ks) + b sin(ks) = A sin(ks + atan2(a,b)), all quadrants", ok29)

# --------------------------------------------------------------------------
# (P1.31) Lagrange identity in u and in phi
un, um = sp.Function("u_n")(r), sp.Function("u_m")(r)
subs2 = {sp.diff(un, r, 2): -lam_n / D * un, sp.diff(um, r, 2): -lam_m / D * um}
W_u = D * (sp.diff(un, r) * um - sp.diff(um, r) * un)
lhs = sp.diff(W_u, r).subs(subs2)
report("P1.31 d/dr[D(u_n'u_m - u_m'u_n)] = (Lambda_m - Lambda_n) u_n u_m",
       sp.simplify(lhs - (lam_m - lam_n) * un * um) == 0)
phn, phm = un / r, um / r
W_phi = r**2 * D * (sp.diff(phn, r) * phm - sp.diff(phm, r) * phn)
report("P1.31 r^2 D(phi_n'phi_m - phi_m'phi_n) = D(u_n'u_m - u_m'u_n)",
       sp.simplify(W_phi - W_u) == 0)
# interface term rewritten through continuous quantities only
cont = (D * (sp.diff(un, r) - un / r)) * um - (D * (sp.diff(um, r) - um / r)) * un
report("P1.31 W = [D(u_n'-u_n/r)] u_m - [D(u_m'-u_m/r)] u_n (continuous)",
       sp.simplify(cont - W_u) == 0)
report("P1.24 r^2 phi_n phi_m = u_n u_m", sp.simplify(r**2 * phn * phm - un * um) == 0)

# --------------------------------------------------------------------------
# Layer integrals (local coordinate s = r - r0, used by Code C and section text)
r0 = sp.symbols("r_0", positive=True)
ul = a * sp.cos(k * s) + b * sp.sin(k * s)
I_closed = ((a * sp.sin(k * s) - b * sp.cos(k * s)) * (r0 + s) / k
            + (a * sp.cos(k * s) + b * sp.sin(k * s)) / k**2)
report("P1.35 antiderivative of r u (local form)",
       sp.simplify(sp.diff(I_closed, s) - (r0 + s) * ul) == 0)
N_closed = ((a**2 + b**2) * s / 2 + (a**2 - b**2) * sp.sin(2 * k * s) / (4 * k)
            + a * b * sp.sin(k * s)**2 / k)
report("P1.33 antiderivative of u^2 (local form)",
       sp.simplify(sp.diff(N_closed, s) - ul**2) == 0)
# Gelbard (2003) Eq. IV.11 B-term check: d/dr of (cos x + x sin x)/k^2 vs printed sign
good = (sp.cos(k * r) + k * r * sp.sin(k * r)) / k**2
bad = (sp.cos(k * r) - k * r * sp.sin(k * r)) / k**2
report("Gelbard IV.11 erratum: (cos x + x sin x) is correct, (cos x - x sin x) is not",
       sp.simplify(sp.diff(good, r) - r * sp.cos(k * r)) == 0
       and sp.simplify(sp.diff(bad, r) - r * sp.cos(k * r)) != 0)

# --------------------------------------------------------------------------
# (P1.35)-(P1.37) projection, inventory and weights for one mode (algebra)
I1, In, Nn = sp.symbols("I_1 I_n N_n", real=True)
an0 = c0 * I1 / Nn
M_t = 4 * sp.pi * an0 * In * sp.exp(-Lam * t)
M0 = sp.Rational(4, 3) * sp.pi * R1**3 * c0
w = 3 * I1 * In / (R1**3 * Nn)
report("P1.37 1 - M/M_0 = 1 - w_n e^{-Lambda t}",
       sp.simplify((1 - M_t / M0) - (1 - w * sp.exp(-Lam * t))) == 0)
al = sp.symbols("alpha", positive=True)
w_scaled = 3 * (al * I1) * (al * In) / (R1**3 * al**2 * Nn)
report("P1.37 w_n invariant under u -> alpha u", sp.simplify(w_scaled - w) == 0)

# --------------------------------------------------------------------------
# Limiting case: homogeneous sphere (all D_i = D)
n = sp.symbols("n", integer=True, positive=True)
un_h = sp.sin(n * sp.pi * r / R)
N_h = sp.integrate(un_h**2, (r, 0, R))
In_h = sp.integrate(r * un_h, (r, 0, R))
I1_h = sp.integrate(r * un_h, (r, 0, R1))
ok = (sp.simplify(N_h - R / 2) == 0
      and sp.simplify(In_h - (-1)**(n + 1) * R**2 / (n * sp.pi)) == 0)
report("limit: homogeneous N_n = R/2, I_n = (-1)^(n+1) R^2/(n pi)", ok)
w_booth = sp.simplify((3 * In_h * In_h / (R**3 * N_h)))
report("limit: R_1 = R gives Booth weight 6/(n^2 pi^2)",
       sp.simplify(w_booth - 6 / (n**2 * sp.pi**2)) == 0)
report("limit: sum of Booth weights = 1 (P1.38)",
       sp.simplify(sp.summation(6 / (n**2 * sp.pi**2), (n, 1, sp.oo)) - 1) == 0)
# kernel-only homogeneous: I_1 closed form used by the Code C unit test
a_n = n * sp.pi / R
I1_test = (sp.sin(a_n * R1) - a_n * R1 * sp.cos(a_n * R1)) / a_n**2
report("limit: kernel-only I_1 closed form (Code C test helper)",
       sp.simplify(I1_h - I1_test) == 0)
# completeness of kernel-only homogeneous weights, numerically (mpmath nsum)
mp.mp.dps = 30
Rv, R1v = mp.mpf("4.5e-4"), mp.mpf("2.0e-4")


def w_kernel(nn):
    an = nn * mp.pi / Rv
    i1 = (mp.sin(an * R1v) - an * R1v * mp.cos(an * R1v)) / an**2
    i_n = Rv**2 * (-1)**(int(nn) + 1) / (nn * mp.pi)
    return 3 * i1 * i_n / (R1v**3 * Rv / 2)


tot = mp.nsum(w_kernel, [1, mp.inf])
report("P1.38 kernel-only homogeneous: sum_n w_n = 1", abs(tot - 1) < 1e-20,
       f"(1 - sum = {mp.nstr(1 - tot, 3)})")

# (P1.36)-(P1.19) modal inventory closure: Lambda_n I_n = -R D_5 u_n'(R) when u_n(R)=0
# homogeneous check: Lambda_n = D (n pi / R)^2
lam_h = D * (n * sp.pi / R)**2
lhs = lam_h * In_h
rhs = -R * D * sp.diff(un_h, r).subs(r, R)
report("P1.19/P1.36 modal closure Lambda_n I_n = -R D u_n'(R) (homogeneous)",
       sp.simplify(lhs - rhs) == 0)

print()
print(f"SUMMARY {sum(results)}/{len(results)} PASS" + ("" if all(results) else "  (FAILURES)"))
