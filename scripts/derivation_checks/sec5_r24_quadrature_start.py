"""Task R24: start of the release quadrature (P22.35) in pseudocode block R2.4 step 3.

Checks the derivation of the R24 section (equations tagged P2X.R24.k):
 (C1) the quadrature route (P22.35) and the balance route M0 - M - M_dec are the same
      expression when Mhat_rel = M0 - Mhat (SymPy), and the balance integrand M(s) has the
      same sqrt(s) start as Mhat_rel (bare kernel).
 (C2) image (Booth-type) short-time form of the bare-kernel release against the modal series,
      and the remainder bound pi^{3/2} u^{3/2} exp(-1/u).
 (C3) energy identity <f,(-L)^{-1} f> = int Q_f^2/(r^2 D) dr (SymPy, two layers) and the
      sphere tail sums W_N, V_N against their integral-test bounds.
 (C4) bare kernel, constant alpha: start-piece truncation error of the modal route against the
      bound lam*min(t0*W_N, V_N/alpha); comparison with the B7 estimate.
 (C5) bare kernel, constant alpha: plain composite Gauss-Legendre from s = 0 (slow, order
      about 1.5) against the proposed start piece plus tau-graded pieces; Bernstein-ellipse
      bound for each graded piece.
 (C6) bare kernel, exponential ramp alpha = exp(beta s): Booth start piece with s = v^2 and
      graded pieces (route B), and the modal start piece with the alpha bracket (route M),
      both against a 30-digit reference.
 (C7) two-layer particle (kernel-only start): Bessel and energy tails, computable bounds
      W_bar_N, V_bar_N, start-piece truncation error against the bound, end-to-end release.
 (C8) cold start: the standard series form carries a spurious release W_N while tau ~ 0.

All parameters are SYNTHETIC and dimensionless (R = 1, time unit R^2/D_ref); no material data.
Run from the repository root:
    .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec5_r24_quadrature_start.py
The script writes no files.
"""
import math

import mpmath as mp
import numpy as np
import sympy as sp
from scipy.optimize import brentq
from scipy.special import gammainc, gamma

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(("PASS  " if ok else "FAIL  ") + name + (("  " + detail) if detail else ""))


GL8 = np.polynomial.legendre.leggauss(8)
GL16 = np.polynomial.legendre.leggauss(16)


def gl(f, a, b, rule=GL8):
    x, w = rule
    s = 0.5 * (b - a) * x + 0.5 * (b + a)
    return 0.5 * (b - a) * float(np.dot(w, f(s)))


# ----------------------------------------------------------------------------------------
# C1  route identity (P2X.R24.1)
# ----------------------------------------------------------------------------------------
t, s, lam, M0 = sp.symbols("t s lambda M_0", positive=True)
G = sp.Function("G")  # G(s) = Mhat(tau(s)), arbitrary
IG = sp.Integral(sp.exp(-lam * s) * G(s), (s, 0, t))
quad_route = sp.exp(-lam * t) * (M0 - G(t)) + lam * (
    M0 * sp.integrate(sp.exp(-lam * s), (s, 0, t)) - IG)
balance_route = M0 - sp.exp(-lam * t) * G(t) - lam * IG
check("C1 (P22.35) with Mhat_rel = M0 - Mhat equals M0 - M - M_dec (SymPy)",
      sp.simplify(quad_route - balance_route) == 0)

mp.mp.dps = 40


def ierfc(x):
    return mp.exp(-x * x) / mp.sqrt(mp.pi) - x * mp.erfc(x)


def F_booth(u, kmax=8):
    u = mp.mpf(u)
    if u <= 0:
        return mp.mpf(0)
    sq = mp.sqrt(u)
    rem = 2 * mp.fsum(ierfc(k / sq) for k in range(1, kmax + 1))
    return 6 * sq * (1 / mp.sqrt(mp.pi) + rem) - 3 * u


def F_series(u):
    u = mp.mpf(u)
    nmax = int(mp.sqrt(110 / (mp.pi ** 2 * u))) + 5
    return 1 - 6 / mp.pi ** 2 * mp.fsum(mp.exp(-n * n * mp.pi ** 2 * u) / n ** 2
                                        for n in range(1, nmax + 1))


def F_exact(u):
    return F_booth(u) if u < 0.05 else F_series(u)


# balance integrand M(s)/M0 = e^{-lam s}(1 - F(s)), alpha = 1, lam = 1: (e^{-s} - M)/sqrt(s) -> 6/sqrt(pi)
ratios = [float((mp.exp(-x) - mp.exp(-x) * (1 - F_exact(x))) / mp.sqrt(x)) for x in (1e-6, 1e-8, 1e-10)]
check("C1 balance integrand has the same sqrt start: (M0 e^{-lam s} - M)/(M0 sqrt s) -> 6/sqrt(pi)",
      abs(ratios[-1] - 6 / math.sqrt(math.pi)) < 1e-4,
      "values %.6f %.6f %.6f, 6/sqrt(pi) = %.6f" % (*ratios, 6 / math.sqrt(math.pi)))

# ----------------------------------------------------------------------------------------
# C2  image (Booth-type) form and its remainder bound (P2X.R24.4), (P2X.R24.5)
# ----------------------------------------------------------------------------------------
dmax = max(abs(F_booth(u) - F_series(u)) for u in (0.002, 0.01, 0.025, 0.05, 0.1, 0.3, 1.0))
check("C2 image form equals the modal series for u in [0.002, 1]", dmax < mp.mpf(10) ** -30,
      "max abs diff %.1e" % float(dmax))
okb, worst = True, 0.0
for u in (0.005, 0.01, 0.02, 0.025, 0.04, 0.0873):
    u_m = mp.mpf(u)
    rem = 12 * mp.sqrt(u_m) * mp.fsum(ierfc(k / mp.sqrt(u_m)) for k in range(1, 9))  # direct, no cancellation
    bnd = mp.pi ** 1.5 * mp.mpf(u) ** 1.5 * mp.exp(-1 / mp.mpf(u))
    okb &= (rem >= 0) and (rem <= bnd)
    worst = max(worst, float(rem / bnd))
check("C2 0 <= F - (6 sqrt(u/pi) - 3u) <= pi^{3/2} u^{3/2} e^{-1/u}", okb,
      "max ratio remainder/bound %.3f" % worst)
rem40 = float(mp.pi ** 1.5 * mp.mpf(1) / 40 ** 1.5 * mp.exp(-40))
check("C2 remainder bound at u_B = 1/40", rem40 < 1e-19, "%.2e" % rem40)

# ----------------------------------------------------------------------------------------
# C3  energy identity (P2X.R24.9) and sphere tails (P2X.R24.12)
# ----------------------------------------------------------------------------------------
r, rr = sp.symbols("r rho", positive=True)
R1s, D1s, D2s = sp.Rational(1, 2), sp.Integer(1), sp.Rational(1, 4)
for name, f1, f2 in (("1", 1, 1), ("c0 = kernel indicator", 1, 0)):
    Q1 = sp.integrate(rr ** 2 * f1, (rr, 0, r))
    Q2 = Q1.subs(r, R1s) + sp.integrate(rr ** 2 * f2, (rr, R1s, r))
    # u from u' = -Q/(r^2 D), u(1) = 0, continuity at R1
    u2 = sp.integrate(Q2.subs(r, rr) / (rr ** 2 * D2s), (rr, r, 1))
    u1 = u2.subs(r, R1s) + sp.integrate(Q1.subs(r, rr) / (rr ** 2 * D1s), (rr, r, R1s))
    lhs = sp.integrate(r ** 2 * f1 * u1, (r, 0, R1s)) + sp.integrate(r ** 2 * f2 * u2, (r, R1s, 1))
    rhs = sp.integrate(Q1 ** 2 / (r ** 2 * D1s), (r, 0, R1s)) + sp.integrate(Q2 ** 2 / (r ** 2 * D2s), (r, R1s, 1))
    resid = sp.simplify(sp.diff(u2, r, 2) + 2 / r * sp.diff(u2, r) + f2 / D2s)
    check("C3 <f,(-L)^{-1}f> = int Q^2/(r^2 D) dr, two layers, f = " + name,
          sp.simplify(lhs - rhs) == 0 and resid == 0, "value " + str(sp.nsimplify(rhs)))

okt = True
for N in (10, 100, 1000):
    WN = 6 / mp.pi ** 2 * mp.zeta(2, N + 1)
    VN = 6 / mp.pi ** 4 * mp.zeta(4, N + 1)
    okt &= WN <= 6 / (mp.pi ** 2 * N) and VN <= 2 / (mp.pi ** 4 * N ** 3)
check("C3 sphere tails W_N <= 6/(pi^2 N), V_N <= 2/(pi^4 N^3) (R = D* = 1), N = 10, 100, 1000", okt)
check("C3 sphere sum w_n/Lambda_n = R^2/(15 D*)",
      abs(6 / mp.pi ** 4 * mp.zeta(4) - mp.mpf(1) / 15) < mp.mpf(10) ** -35)

# ----------------------------------------------------------------------------------------
# C4  sphere, constant alpha: start-piece truncation against the bound (P2X.R24.13)
# ----------------------------------------------------------------------------------------
lamv = 1.0
ok4, rows = True, []
for alpha in (1.0, 0.1):
    for N in (10, 100, 1000):
        L_next = (N + 1) ** 2 * math.pi ** 2
        for t0 in (40.0 / (alpha * L_next), 0.5):
            kap = lambda n: lamv + alpha * n ** 2 * mp.pi ** 2
            act = lamv * mp.nsum(lambda n: 6 / (n ** 2 * mp.pi ** 2) * (1 - mp.exp(-kap(n) * t0)) / kap(n),
                                 [N + 1, mp.inf])
            WN = float(6 / mp.pi ** 2 * mp.zeta(2, N + 1))
            VN = float(6 / mp.pi ** 4 * mp.zeta(4, N + 1))
            bnd = lamv * min(t0 * WN, VN / alpha)
            ok4 &= float(act) <= bnd
            rows.append((alpha, N, t0, float(act), bnd, lamv * t0 * WN))
check("C4 sphere start piece: truncation error <= lam min(t0 W_N, V_N/alpha) in 12 cases", ok4)
for a_, N, t0, act, bnd, b7 in rows:
    if t0 != 0.5:
        print("      alpha=%-4g N=%-5d t0=%.3e  error %.3e  bound %.3e  ratio %.3f  B7 estimate %.3e"
              % (a_, N, t0, act, bnd, act / bnd, b7))
ratio_b7 = min(b7 / bnd for a_, N, t0, act, bnd, b7 in rows if t0 != 0.5)
check("C4 at t0 = 40/(alpha Lambda_{N+1}) the new bound is at least 100 times below lam t0 W_N",
      ratio_b7 > 100, "smallest ratio %.1f" % ratio_b7)

# ----------------------------------------------------------------------------------------
# C5  sphere, constant alpha = 1, lam = 1, t = 1: plain GL from 0 versus graded pieces
# ----------------------------------------------------------------------------------------
tend = 1.0
exact_total = 1 - mp.exp(-tend) * (1 - F_exact(tend)) - lamv * mp.nsum(
    lambda n: 6 / (n ** 2 * mp.pi ** 2) * (1 - mp.exp(-(1 + n ** 2 * mp.pi ** 2) * tend)) / (1 + n ** 2 * mp.pi ** 2),
    [1, mp.inf])
check("C5 reference: closed form equals (P22.35) evaluated in mpmath",
      abs(exact_total - (mp.exp(-tend) * F_exact(tend)
                         + mp.quad(lambda x: mp.exp(-x) * F_exact(x), [0, 1e-4, 1e-2, 0.1, 1]))) < 1e-25)
ref_int = mp.quad(lambda x: mp.exp(-x) * F_exact(x), [0, 1e-6, 1e-4, 1e-2, 0.1, 1])
errs = []
for m in (1, 2, 4, 8, 16, 32):
    h = tend / m
    x, w = GL8
    acc = mp.mpf(0)
    for j in range(m):
        a = j * h
        for xi, wi in zip(x, w):
            sv = a + 0.5 * h * (xi + 1)
            acc += 0.5 * h * wi * mp.exp(-sv) * F_exact(sv)
    errs.append(float(abs(acc - ref_int)))
order = math.log2(errs[-2] / errs[-1])
check("C5 plain composite 8-point GL from s = 0 converges slowly (order about 1.5 in pieces)",
      1.3 < order < 1.7 and errs[-1] > 1e-9,
      "errors m=1..32: " + " ".join("%.1e" % e for e in errs) + "; order %.2f" % order)

N5 = 200
nn = np.arange(1, N5 + 1, dtype=float)
w5 = 6 / (nn ** 2 * math.pi ** 2)
L5 = nn ** 2 * math.pi ** 2
W5 = float(6 / mp.pi ** 2 * mp.zeta(2, N5 + 1))
V5 = float(6 / mp.pi ** 4 * mp.zeta(4, N5 + 1))


def rel_hat(tau, w, L):
    tau = np.atleast_1d(tau)
    return 1.0 - np.exp(-np.outer(tau, L)) @ w


def modal_start(t0, a, w, L, lam_=1.0):
    kap = lam_ + a * L
    return (1 - math.exp(-lam_ * t0)) - lam_ * float(np.dot(w, -np.expm1(-kap * t0) / kap))


def graded_edges(t_a, t_b, tau_of, s_of, q=2.0):
    edges = [t_a]
    while True:
        nxt = s_of(q * tau_of(edges[-1]))
        if nxt >= t_b:
            edges.append(t_b)
            return edges
        edges.append(nxt)


t0 = 40.0 / ((N5 + 1) ** 2 * math.pi ** 2)
I0 = modal_start(t0, 1.0, w5, L5)
edges = graded_edges(t0, tend, lambda x: x, lambda x: x)
I1 = sum(gl(lambda sv: np.exp(-sv) * rel_hat(sv, w5, L5), a, b) for a, b in zip(edges[:-1], edges[1:]))
total = math.exp(-tend) * rel_hat(tend, w5, L5)[0] + I0 + I1
err5 = abs(total - float(exact_total))
bnd5 = lamv * V5 + math.exp(-(N5 + 1) ** 2 * math.pi ** 2 * t0) * W5 + 1e-13
check("C5 proposed route (modal start piece, ratio-2 graded GL) error <= lam V_N + e^{-40} W_N + 1e-13",
      err5 <= bnd5, "N = %d, %d graded pieces, error %.3e, bound %.3e, lam V_N = %.3e"
      % (N5, len(edges) - 1, err5, bnd5, lamv * V5))

# Bernstein-ellipse bound per graded piece, same N modes (truncation cancels); mpmath arithmetic
okB, worstB = True, 0.0
Wall = float(np.sum(np.abs(w5)))
wm = [mp.mpf(6) / (n * n * mp.pi ** 2) for n in range(1, N5 + 1)]
Lm = [n * n * mp.pi ** 2 for n in range(1, N5 + 1)]


def mp_gauss_legendre(n):
    """Gauss-Legendre nodes and weights refined by Newton's method in mpmath."""
    xs, ws = [], []
    for x in np.polynomial.legendre.leggauss(n)[0]:
        x = mp.mpf(x)
        for _ in range(6):
            p, dp = mp.legendre(n, x), n * (x * mp.legendre(n, x) - mp.legendre(n - 1, x)) / (x * x - 1)
            x = x - p / dp
        dp = n * (x * mp.legendre(n, x) - mp.legendre(n - 1, x)) / (x * x - 1)
        xs.append(x)
        ws.append(2 / ((1 - x * x) * dp * dp))
    return xs, ws


xg, wg = mp_gauss_legendre(8)
for q in (2.0, 8.0):
    e_q = graded_edges(t0, tend, lambda x: x, lambda x: x, q)
    tot_err = 0.0
    for a, b in zip(e_q[:-1], e_q[1:]):
        a, b = mp.mpf(a), mp.mpf(b)
        exact_p = (mp.exp(-a) - mp.exp(-b)) - mp.fsum(
            wn * (mp.exp(-(1 + Ln) * a) - mp.exp(-(1 + Ln) * b)) / (1 + Ln) for wn, Ln in zip(wm, Lm))
        num = mp.mpf(0)
        for xi, wi in zip(xg, wg):
            sv = (b - a) / 2 * xi + (b + a) / 2
            num += (b - a) / 2 * wi * mp.exp(-sv) * (1 - mp.fsum(wn * mp.exp(-Ln * sv) for wn, Ln in zip(wm, Lm)))
        x0 = (b + a) / (b - a)
        rho = x0 + mp.sqrt(x0 * x0 - 1)
        bound = (b - a) / 2 * 8 * (1 + Wall) * rho ** (1 - 16) / (rho - 1)  # (P2X.R24.17), n = 8
        err = abs(num - exact_p)
        tot_err += float(err)
        if q == 2.0:
            okB &= err <= bound
            worstB = max(worstB, float(err / bound))
    print("      q = %g: %d pieces, summed GL error %.2e" % (q, len(e_q) - 1, tot_err))
rho2 = 3 + 2 * math.sqrt(2)
tot_bound = 0.5 * (tend - t0) * 8 * (1 + Wall) * rho2 ** (-15) / (rho2 - 1)
check("C5 each ratio-2 piece obeys the Bernstein-ellipse bound (b-a)/2 * 8 M rho^(1-2n)/(rho-1), n = 8",
      okB, "largest error/bound %.2e; summed bound over [t0, t] %.2e" % (worstB, tot_bound))

# ----------------------------------------------------------------------------------------
# C6  bare kernel, ramp alpha(s) = exp(beta s): routes B and M against a 30-digit reference
# ----------------------------------------------------------------------------------------
beta, t6 = 5.0, 0.3
tau6 = lambda x: np.expm1(beta * np.asarray(x)) / beta
s_of6 = lambda y: math.log1p(beta * y) / beta
mp.mp.dps = 30
ref6 = mp.exp(-t6) * F_exact(mp.expm1(beta * t6) / beta) + mp.quad(
    lambda x: mp.exp(-x) * F_exact(mp.expm1(beta * x) / beta),
    [0, 1e-8, 1e-6, 1e-4, 1e-3, 1e-2, 0.05, 0.1, 0.2, t6])
ref6 = float(ref6)

# route B: Booth start on [0, t_B] with s = v^2, then N = 12 modes on graded pieces
uB = 1.0 / 40
tB = s_of6(uB)
nB = np.arange(1, 13, dtype=float)
wB, LB = 6 / (nB ** 2 * math.pi ** 2), nB ** 2 * math.pi ** 2
booth2 = lambda u: 6 * np.sqrt(u / math.pi) - 3 * u
startB = gl(lambda v: 2 * v * np.exp(-v * v) * booth2(tau6(v * v)), 0.0, math.sqrt(tB), GL16)
startB8 = gl(lambda v: 2 * v * np.exp(-v * v) * booth2(tau6(v * v)), 0.0, math.sqrt(tB), GL8)
eB = graded_edges(tB, t6, lambda x: float(tau6(x)), s_of6)
lateB = sum(gl(lambda sv: np.exp(-sv) * rel_hat(tau6(sv), wB, LB), a, b) for a, b in zip(eB[:-1], eB[1:]))
totB = math.exp(-t6) * rel_hat(float(tau6(t6)), wB, LB)[0] + startB + lateB
errB = abs(totB - ref6)
WB = float(6 / mp.pi ** 2 * mp.zeta(2, 13))
bndB = rem40 + math.exp(-169 * math.pi ** 2 * uB) * WB
check("C6 route B (Booth start, 12 modes) error at the rounding level",
      errB < 1e-14 and bndB < 1e-18,
      "error %.2e, analytic part of the bound %.1e, start GL8 vs GL16 %.1e, %d graded pieces"
      % (errB, bndB, abs(startB - startB8), len(eB) - 1))

# route M: modal start with the alpha bracket, N = 200
t0M = s_of6(40.0 / ((N5 + 1) ** 2 * math.pi ** 2))
a_lo, a_hi = 1.0, math.exp(beta * t0M)
Jlo, Jhi = modal_start(t0M, a_lo, w5, L5), modal_start(t0M, a_hi, w5, L5)
startM = 0.5 * (Jlo + Jhi)
half = 0.5 * abs(Jhi - Jlo)
eM = graded_edges(t0M, t6, lambda x: float(tau6(x)), s_of6)
lateM = sum(gl(lambda sv: np.exp(-sv) * rel_hat(tau6(sv), w5, L5), a, b) for a, b in zip(eM[:-1], eM[1:]))
totM = math.exp(-t6) * rel_hat(float(tau6(t6)), w5, L5)[0] + startM + lateM
errM = abs(totM - ref6)
bndM = half + 2 * lamv * min(t0M * W5, V5 / a_lo) + math.exp(-40) * W5 + 1e-13
check("C6 route M (modal start, alpha bracket, N = 200) error <= half-width + 2 lam min(t0 W, V/alpha_-) + e^-40 W + 1e-13",
      errM <= bndM, "error %.3e, bound %.3e (half-width %.1e)" % (errM, bndM, half))

# ----------------------------------------------------------------------------------------
# C7  two-layer particle, kernel-only start: R1 = 0.5, D = (1, 0.25)
# ----------------------------------------------------------------------------------------
R1, D1, D2 = 0.5, 1.0, 0.25


def shoot(z):
    lam_ = z * z
    k1, k2 = z / math.sqrt(D1), z / math.sqrt(D2)
    u = np.sin(k1 * R1)
    up2 = (D1 * k1 * np.cos(k1 * R1) + (D2 - D1) * u / R1) / D2
    A2 = u * np.sin(k2 * R1) + up2 / k2 * np.cos(k2 * R1)
    B2 = u * np.cos(k2 * R1) - up2 / k2 * np.sin(k2 * R1)
    return A2 * np.sin(k2) + B2 * np.cos(k2), (k1, k2, A2, B2)


NREF = 2000
zs = np.arange(0.01, 2.2 * NREF + 50, 0.01)
fz = shoot(zs)[0]
idx = np.nonzero(np.sign(fz[:-1]) * np.sign(fz[1:]) < 0)[0]
roots = [brentq(lambda z: shoot(z)[0], zs[i], zs[i + 1], xtol=1e-13, rtol=1e-15) for i in idx[:NREF]]
roots = np.array(roots)


def Isin(k, a, b): return (np.sin(k * b) - np.sin(k * a)) / k ** 2 - (b * np.cos(k * b) - a * np.cos(k * a)) / k
def Icos(k, a, b): return (np.cos(k * b) - np.cos(k * a)) / k ** 2 + (b * np.sin(k * b) - a * np.sin(k * a)) / k
def Iss(k, a, b): return 0.5 * (b - a) - (np.sin(2 * k * b) - np.sin(2 * k * a)) / (4 * k)
def Icc(k, a, b): return 0.5 * (b - a) + (np.sin(2 * k * b) - np.sin(2 * k * a)) / (4 * k)
def Isc(k, a, b): return (np.sin(k * b) ** 2 - np.sin(k * a) ** 2) / (2 * k)


_, (k1, k2, A2, B2) = shoot(roots)
Nn = Iss(k1, 0, R1) + A2 ** 2 * Iss(k2, R1, 1) + 2 * A2 * B2 * Isc(k2, R1, 1) + B2 ** 2 * Icc(k2, R1, 1)
I1 = Isin(k1, 0, R1)
In = I1 + A2 * Isin(k2, R1, 1) + B2 * Icos(k2, R1, 1)
c0n, onen = I1 / np.sqrt(Nn), In / np.sqrt(Nn)
L7 = roots ** 2
M0f = 4 * math.pi * R1 ** 3 / 3
w7 = 4 * math.pi / M0f * c0n * onen
Hc0 = R1 ** 5 / (45 * D1) + R1 ** 6 / 9 * (1 / R1 - 1) / D2
H1 = R1 ** 5 / (45 * D1) + (1 - R1 ** 5) / (45 * D2)
Hx = R1 ** 5 / (45 * D1) + R1 ** 3 / 9 * (1 - R1 ** 2) / (2 * D2)
Pc0, P1 = R1 ** 3 / 3, 1.0 / 3


def tails(N):
    P_c = Pc0 - math.fsum(c0n[:N] ** 2)
    P_1 = P1 - math.fsum(onen[:N] ** 2)
    H_c = Hc0 - math.fsum(c0n[:N] ** 2 / L7[:N])
    H_1 = H1 - math.fsum(onen[:N] ** 2 / L7[:N])
    return P_c, P_1, H_c, H_1


check("C7 2000 roots found, sum w_n -> 1 and the weights change sign",
      len(roots) == NREF and abs(math.fsum(w7) - 1) < 2e-3,
      "1 - sum w = %.2e, sign changes %d" % (1 - math.fsum(w7), int(np.sum(np.diff(np.sign(w7)) != 0))))
sig = math.fsum(w7 / L7)
check("C7 modal sum w_n/Lambda_n equals (4 pi/M0) int Q_c0 Q_1/(r^2 D) dr",
      abs(sig - 4 * math.pi / M0f * Hx) < 1e-9, "diff %.1e" % abs(sig - 4 * math.pi / M0f * Hx))
tl = [tails(N) for N in (1, 10, 100, 1000, 2000)]
check("C7 Bessel and energy tails are non-negative and shrink (no missed root)",
      all(min(x) > -1e-15 for x in tl) and tl[-1][3] < 1e-10 and tl[-1][2] < 1e-10,
      "H_N(1) at N = 100, 1000, 2000: %.2e %.2e %.2e" % (tl[2][3], tl[3][3], tl[4][3]))
check("C7 sum |w_n| <= (R/R1)^{3/2} (Cauchy-Schwarz)",
      math.fsum(np.abs(w7)) <= (1 / R1) ** 1.5, "%.3f <= %.3f" % (math.fsum(np.abs(w7)), (1 / R1) ** 1.5))
ok7, worst7, loose = True, 0.0, []
for N in (20, 50, 100, 200):
    P_c, P_1, H_c, H_1 = tails(N)
    Wbar = 4 * math.pi / M0f * math.sqrt(max(P_c, 0) * max(P_1, 0))
    Vbar = 4 * math.pi / M0f * math.sqrt(max(H_c, 0) * max(H_1, 0))
    Wtrue = math.fsum(np.abs(w7[N:]))
    t0 = 40.0 / L7[N]
    kap = lamv + L7[N:]
    act = lamv * abs(math.fsum(w7[N:] * (-np.expm1(-kap * t0)) / kap))
    Pc_r, P1_r, Hc_r, H1_r = tails(NREF)
    rest = lamv * 4 * math.pi / M0f * math.sqrt(max(Hc_r, 0) * max(H1_r, 0))
    bnd = lamv * min(t0 * Wbar, Vbar)
    ok7 &= (act + rest <= bnd) and (Wtrue <= Wbar)
    worst7 = max(worst7, (act + rest) / bnd)
    loose.append("N=%d: error %.2e bound %.2e (V_bar %.2e, sum|w|/L over 2000 modes %.2e)"
                 % (N, act, bnd, Vbar, math.fsum(np.abs(w7[N:]) / L7[N:])))
check("C7 start-piece truncation (plus the bounded remainder beyond 2000 modes) <= lam min(t0 W_bar, V_bar)",
      ok7, "largest ratio %.3f" % worst7)
for line in loose:
    print("      " + line)

# end-to-end, constant alpha = 1, lam = 1, t = 1, N = 100 against N = 2000 per-mode closed form
N7 = 100
t0 = 40.0 / L7[N7]
I0 = modal_start(t0, 1.0, w7[:N7], L7[:N7])
e7 = graded_edges(t0, 1.0, lambda x: x, lambda x: x)
I1 = sum(gl(lambda sv: np.exp(-sv) * rel_hat(sv, w7[:N7], L7[:N7]), a, b) for a, b in zip(e7[:-1], e7[1:]))
tot7 = math.exp(-1.0) * rel_hat(1.0, w7[:N7], L7[:N7])[0] + I0 + I1
kapr = 1.0 + L7
ref7 = 1 - math.exp(-1.0) * math.fsum(w7 * np.exp(-L7)) - math.fsum(w7 * (-np.expm1(-kapr)) / kapr)
P_c, P_1, H_c, H_1 = tails(N7)
b7 = (lamv * 4 * math.pi / M0f * math.sqrt(H_c * H_1) + math.exp(-40) * 4 * math.pi / M0f * math.sqrt(P_c * P_1)
      + rest + 1e-13)
check("C7 two-layer end-to-end release (N = 100) within the bound", abs(tot7 - ref7) <= b7,
      "error %.3e, bound %.3e" % (abs(tot7 - ref7), b7))

# ----------------------------------------------------------------------------------------
# C8  cold start: alpha = 1e-8 on [0, 1), then 1; lam = 1; standard series at N = 100
# ----------------------------------------------------------------------------------------
N8, a_cold, tc = 100, 1e-8, 1.0
w8, L8 = w5[:N8], L5[:N8]
std = modal_start(tc, a_cold, w8, L8)  # standard form on the cold piece
kap = 1.0 + a_cold * L8
direct = float(np.dot(w8, (-np.expm1(-tc) - (-np.expm1(-kap * tc)) / kap * 1.0)))  # sum_{n<=N} w_n int e^{-s}(1-e^{-a L s})
true_c = float(mp.quad(lambda x: mp.exp(-x) * F_exact(a_cold * x), [0, 1e-6, 1e-3, 1]))
booth_c = float(6 * math.sqrt(a_cold / math.pi) * gammainc(1.5, tc) * gamma(1.5)
                - 3 * a_cold * (1 - math.exp(-tc) * (1 + tc)))
W8 = float(6 / mp.pi ** 2 * mp.zeta(2, N8 + 1))
n_big = np.arange(N8 + 1, 10 ** 7 + 1, dtype=float)
kap_big = 1.0 + a_cold * n_big ** 2 * math.pi ** 2
# sum over N < n <= 1e7 directly; the rest is below 6/(pi^4 a) / (3e21) = 2e-15
trunc_std = math.fsum(6 / (n_big ** 2 * math.pi ** 2) * (-np.expm1(-kap_big * tc)) / kap_big)
check("C8 cold piece: standard-form error equals lam sum_{n>N} w_n E_n and lies in [0, (1 - e^{-lam t_c}) W_N]",
      abs((std - true_c) - trunc_std) < 1e-11 and 0 <= trunc_std <= (1 - math.exp(-tc)) * W8,
      "error %.4e, lam sum_{n>N} w_n E_n %.4e, (1-e^{-1}) W_N = %.3e" % (std - true_c, trunc_std, (1 - math.exp(-tc)) * W8))
check("C8 cold piece: direct-form error <= true start-piece release <= (1 - e^{-lam t_c}) 6 sqrt(a t_c/pi); Booth exact",
      abs(direct - true_c) <= true_c <= (1 - math.exp(-tc)) * 6 * math.sqrt(a_cold * tc / math.pi)
      and abs(booth_c - true_c) < 1e-15,
      "direct error %.2e, true %.2e, Booth closed form error %.1e" % (abs(direct - true_c), true_c, abs(booth_c - true_c)))

print("SUMMARY: %d/%d PASS" % (sum(RESULTS), len(RESULTS)))
if all(RESULTS):
    print("ALL PASS")
