"""STRATO Phase 2.1 checks, part 2: modal solution, release, inventory closure
and limiting cases (equations P21.24 to P21.49), symbolic (SymPy) plus a few
high-precision series sums (mpmath).

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec3_p21_modal_ode.py
Writes nothing to disk. Prints one PASS/FAIL line per check.
"""
import sys

sys.dont_write_bytecode = True
import mpmath as mp
import sympy as sp

results = []


def check(name, ok, detail=""):
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")
    results.append(bool(ok))


def zero(expr):
    return sp.simplify(expr) == 0


r, t, s, tau = sp.symbols("r t s tau", positive=True)
R, Dc, lam, k, a0, sn, S0, R1 = sp.symbols("R D lambda k a_0 s_n S_0 R_1", positive=True)
n, m = sp.symbols("n m", positive=True, integer=True)

# ---------------------------------------------------------------------------
# (P21.25) Green identity behind self-adjointness (generic D(r), phi, psi)
# ---------------------------------------------------------------------------
Df, ph, ps = sp.Function("D")(r), sp.Function("phi")(r), sp.Function("psi")(r)
lhs = ps * sp.diff(r ** 2 * Df * ph.diff(r), r) - ph * sp.diff(r ** 2 * Df * ps.diff(r), r)
rhs = sp.diff(r ** 2 * Df * (ps * ph.diff(r) - ph * ps.diff(r)), r)
check("P21.25 psi(r^2 D phi')' - phi(r^2 D psi')' = [r^2 D (psi phi' - phi psi')]'", zero(lhs - rhs))

# ---------------------------------------------------------------------------
# (P21.24), (P21.26), (P21.36) homogeneous sphere eigenpairs as a concrete case
# ---------------------------------------------------------------------------
phi = lambda j: sp.sin(j * sp.pi * r / R) / r
Lam = lambda j: j ** 2 * sp.pi ** 2 * Dc / R ** 2
Lop = lambda f: sp.diff(r ** 2 * Dc * sp.diff(f, r), r) / r ** 2
check("P21.24 L phi_n = -Lambda_n phi_n (homogeneous sphere, symbolic n)", zero(Lop(phi(n)) + Lam(n) * phi(n)))
orth = [sp.integrate(r ** 2 * phi(i) * phi(j), (r, 0, R)) for i, j in [(1, 2), (2, 3), (1, 4)]]
check("P21.26 orthogonality int r^2 phi_m phi_n = 0 (m != n samples)", all(zero(o) for o in orth))
Nn = sp.integrate(r ** 2 * phi(n) ** 2, (r, 0, R))
check("P21.26 N_n = R/2 for the homogeneous sphere", zero(Nn - R / 2), f"N_n = {sp.simplify(Nn)}")
In = sp.integrate(r ** 2 * phi(n), (r, 0, R))
check("P21.35 I_n = (-1)^(n+1) R^2/(n pi)", zero(In - (-1) ** (n + 1) * R ** 2 / (n * sp.pi)))
flux_id = R ** 2 * Dc * sp.diff(phi(n), r).subs(r, R) + Lam(n) * In
check("P21.36 R^2 D phi_n'(R) = -Lambda_n I_n", zero(flux_id))

# ---------------------------------------------------------------------------
# (P21.31)-(P21.34) modal ODE solutions
# ---------------------------------------------------------------------------
sf = sp.Function("s")
a_if = a0 * sp.exp(-k * t) + sp.Integral(sp.exp(-k * (t - tau)) * sf(tau), (tau, 0, t))
check("P21.31 integrating-factor solution satisfies a' = -k a + s(t)",
      zero(sp.diff(a_if, t).doit() + k * a_if.doit() - sf(t)) and zero(a_if.doit().subs(t, 0) - a0))
a_c = a0 * sp.exp(-k * t) + sn / k * (1 - sp.exp(-k * t))
check("P21.32 constant-source closed form", zero(a_c.diff(t) + k * a_c - sn) and zero(a_c.subs(t, 0) - a0))
check("P21.32 steady amplitude s_n/k_n", zero(sp.limit(a_c, t, sp.oo) - sn / k))
# piecewise-constant source: two intervals, recursion vs integrating factor
t1, dt2, s1, s2 = sp.symbols("t_1 Delta_2 s_1 s_2", positive=True)
step = lambda aprev, sj, d: aprev * sp.exp(-k * d) + sj / k * (1 - sp.exp(-k * d))
rec = step(step(a0, s1, t1), s2, dt2)
direct = a0 * sp.exp(-k * (t1 + dt2)) + sp.integrate(sp.exp(-k * (t1 + dt2 - tau)) * s1, (tau, 0, t1)) \
    + sp.integrate(sp.exp(-k * (t1 + dt2 - tau)) * s2, (tau, t1, t1 + dt2))
check("P21.33 piecewise-constant recursion equals integrating factor", zero(rec - direct))
ti = sp.symbols("t_irr", positive=True)
a_off = a_c.subs(t, ti) * sp.exp(-k * (t - ti))
check("P21.34 source switched off: a(t) = a(t_irr) exp(-k (t - t_irr))", zero(a_off.diff(t) + k * a_off))

# ---------------------------------------------------------------------------
# (P21.38)-(P21.42) release: direct modal vs inventory balance, per mode
# ---------------------------------------------------------------------------
In_s, Lam_s = sp.symbols("I_n Lambda_n", positive=True)
kk = Lam_s + lam
a_m = a0 * sp.exp(-kk * t) + sn / kk * (1 - sp.exp(-kk * t))
A_int = sp.integrate(a_m.subs(t, s), (s, 0, t))
A_closed = a0 * (1 - sp.exp(-kk * t)) / kk + sn / kk * (t - (1 - sp.exp(-kk * t)) / kk)
check("P21.38 closed form of int_0^t a_n ds (constant source)", zero(A_int - A_closed))
Mrel_direct = 4 * sp.pi * Lam_s * In_s * A_closed
M0_n, Mgen_n = 4 * sp.pi * In_s * a0, 4 * sp.pi * In_s * sn * t
M_n, Mdec_n = 4 * sp.pi * In_s * a_m, lam * 4 * sp.pi * In_s * A_closed
check("P21.39 per-mode identity: direct release = M0 + Mgen - M - Mdec",
      zero(Mrel_direct - (M0_n + Mgen_n - M_n - Mdec_n)))
check("P21.42 per-mode closure M0 + Mgen = M + Mdec + Mrel",
      zero(M0_n + Mgen_n - (M_n + Mdec_n + Mrel_direct)))
# global balance dM/dt = G - lambda M - J_R, per mode (G_n = 4 pi I_n s_n, J_n = 4 pi Lambda_n I_n a_n)
check("P21.40 dM/dt = G - lambda M - J_R (per mode)",
      zero(M_n.diff(t) - (4 * sp.pi * In_s * sn - lam * M_n - 4 * sp.pi * Lam_s * In_s * a_m)))

# (P21.43)-(P21.44) M_out and total surviving inventory
# (P21.45) source-free case: ultimate release fraction and decay-corrected release
Mrel_inf = sp.limit(Mrel_direct.subs(sn, 0), t, sp.oo)
check("P21.45 source-free ultimate release 4 pi I_n a_n(0) Lambda_n/k_n (per mode)",
      zero(Mrel_inf - 4 * sp.pi * In_s * a0 * Lam_s / kk))
JR_free = 4 * sp.pi * Lam_s * In_s * a0 * sp.exp(-kk * t)
check("P21.45 decay-corrected flux J_R e^(lambda t) equals the lambda = 0 flux (per mode)",
      zero(JR_free * sp.exp(lam * t) - 4 * sp.pi * Lam_s * In_s * a0 * sp.exp(-Lam_s * t)))
Jf, Gf = sp.Function("J"), sp.Function("G")
Mout = sp.Integral(Jf(tau) * sp.exp(-lam * (t - tau)), (tau, 0, t))
check("P21.43 M_out = int J exp(-lambda (t-s)) ds solves M_out' = J - lambda M_out",
      zero(sp.diff(Mout, t).doit() - (Jf(t) - lam * Mout.doit())))
M0 = sp.symbols("M_0", positive=True)
surv = M0 * sp.exp(-lam * t) + sp.Integral(Gf(tau) * sp.exp(-lam * (t - tau)), (tau, 0, t))
check("P21.44 M + M_out = M0 e^(-lambda t) + int G e^(-lambda (t-s)) ds solves (M+M_out)' = G - lambda (M+M_out)",
      zero(sp.diff(surv, t).doit() - (Gf(t) - lam * surv.doit())))

# ---------------------------------------------------------------------------
# (P21.46) lambda = 0 recovers the Phase 1 weights w_n = 3 I1_n I_n/(R1^3 N_n)
# ---------------------------------------------------------------------------
I1, Nn_s, c0 = sp.symbols("I1_n N_n c_0", positive=True)
an0 = c0 * I1 / Nn_s                                  # projection of kernel-uniform c0
w_from_modal = 4 * sp.pi * an0 * In_s / (sp.Rational(4, 3) * sp.pi * c0 * R1 ** 3)
check("P21.46 4 pi a_n(0) I_n / M_0 = 3 I1_n I_n /(R1^3 N_n)", zero(w_from_modal - 3 * I1 * In_s / (R1 ** 3 * Nn_s)))
# kernel-uniform constant source gives the same weights for R/B
sn_k = S0 * I1 / Nn_s
G_k = sp.Rational(4, 3) * sp.pi * R1 ** 3 * S0
check("P21.47 4 pi s_n I_n / G = w_n (source weights equal v1.0 weights)",
      zero(4 * sp.pi * sn_k * In_s / G_k - 3 * I1 * In_s / (R1 ** 3 * Nn_s)))
JR_mode = 4 * sp.pi * Lam_s * In_s * (sn_k / kk) * (1 - sp.exp(-kk * t)) / G_k
w_s = sp.symbols("w_n", positive=True)
check("P21.47 per-mode R/B(t) = w_n Lambda_n/k_n (1 - e^(-k_n t))",
      zero(JR_mode.subs(I1, w_s * R1 ** 3 * Nn_s / (3 * In_s)) - w_s * Lam_s / kk * (1 - sp.exp(-kk * t))))
check("P21.47 accelerated split Lambda_n/k_n = 1 - lambda/k_n", zero(Lam_s / kk - (1 - lam / kk)))
lap = sp.integrate(Lam_s * sp.exp(-Lam_s * t) * sp.exp(-lam * t), (t, 0, sp.oo))
check("P21.49 Laplace transform: int Lambda e^(-Lambda t) e^(-lambda t) dt = Lambda/(Lambda+lambda)",
      zero(lap - Lam_s / kk))

# ---------------------------------------------------------------------------
# (P21.48) Booth single sphere, TECDOC-1674 Eqs (7.6)-(7.8): per-term identities
# x = lambda/D', D' = D/R^2, w_n = 6/(n^2 pi^2), Lambda_n = n^2 pi^2 D'
# ---------------------------------------------------------------------------
Dp, x = sp.symbols("Dprime x", positive=True)
wn = 6 / (n ** 2 * sp.pi ** 2)
Ln = n ** 2 * sp.pi ** 2 * Dp
kn = Ln + x * Dp
mine_RB = wn * Ln / kn * (1 - sp.exp(-kn * t))
booth_RB = 6 / (n ** 2 * sp.pi ** 2 + x) - 6 * sp.exp(-x * Dp * t) * sp.exp(-n ** 2 * sp.pi ** 2 * Dp * t) / (n ** 2 * sp.pi ** 2 + x)
check("P21.48 modal R/B(t) term = TECDOC-1674 (7.7) term", zero(mine_RB - booth_RB))
lamv = x * Dp
mine_F_term = wn - lamv / (1 - sp.exp(-lamv * t)) * wn * (1 - sp.exp(-kn * t)) / kn   # F_B = 1 - lambda M /(P(1-e^-lambda t))
booth_F_term = 6 / (n ** 2 * sp.pi ** 2 + x) - 6 / sp.pi ** 2 * x / (sp.exp(lamv * t) - 1) \
    * (1 - sp.exp(-n ** 2 * sp.pi ** 2 * Dp * t)) / (n ** 2 * (n ** 2 * sp.pi ** 2 + x))
check("P21.48 F_B = M_out/(M + M_out) term = TECDOC-1674 (7.8) term (uses sum w_n = 1)",
      zero(sp.simplify(mine_F_term - booth_F_term).rewrite(sp.exp)))

mp.mp.dps = 30
for xv in [mp.mpf("0.01"), mp.mpf("1"), mp.mpf("100"), mp.mpf("2107")]:
    closed = 3 * mp.sqrt(1 / xv) * (mp.coth(mp.sqrt(xv)) - mp.sqrt(1 / xv))
    series = mp.nsum(lambda j: 6 / (j ** 2 * mp.pi ** 2 + xv), [1, mp.inf])
    check(f"P21.49 (7.6) closed form = sum 6/(n^2 pi^2 + x), x = {mp.nstr(xv, 5)}",
          abs(closed - series) < mp.mpf("1e-25"), f"R/B = {mp.nstr(closed, 12)}")

xs = sp.symbols("x", positive=True)
RBss = 3 / sp.sqrt(xs) * (sp.coth(sp.sqrt(xs)) - 1 / sp.sqrt(xs))
ser0 = sp.series(RBss, xs, 0, 3).removeO()
check("P21.49 small-lambda limit: R/B_ss = 1 - x/15 + 2x^2/315 + ...", zero(ser0 - (1 - xs / 15 + 2 * xs ** 2 / 315)))
check("P21.49 short-lived limit: R/B_ss * sqrt(x)/3 -> 1 (TECDOC-1674 7.25)",
      sp.limit(RBss * sp.sqrt(xs) / 3, xs, sp.oo) == 1)

print(f"\nSUMMARY: {sum(results)}/{len(results)} PASS")
sys.exit(0 if all(results) else 1)
