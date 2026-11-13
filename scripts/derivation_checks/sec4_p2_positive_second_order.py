"""Phase 2.1 (section 4), open issue 5: second-order time integrators and positivity.

Semi-discrete system (P21.55) written as dC/dt = -B C + s(t), B = V^-1 K + lam I.
The stability functions r(x), x = h mu >= 0, of the candidate schemes are analysed
symbolically, the positivity barrier is checked on the structural matrix family used
in its proof, and every candidate is run on the synthetic five-layer case of
sec4_p21_be_closure_order.py (TECDOC-1674 Table 10.2 geometry, synthetic D, kernel
loaded with 1 mol m^-3; mesh and subtraction-free solver transcribed from that script).

Checks:
  S1. symbolic: order and error constants of CN, TR-BDF2 (gamma = 2 - sqrt 2), local
      Richardson extrapolation of BE, BDF2; TR-BDF2 uses one matrix in both stages;
      zeros, minima and limits at infinity of r; radii of total monotonicity
      (CN 2, TR-BDF2 1 + sqrt 2, Richardson 0, BE and exp infinite); BDF2 roots and the
      amplitude 1 + (3/4) x^2 of the principal root after a Backward Euler start.
  S2. proof mechanics: Taylor weights at x = a (lemma P2.L2) and the Pringsheim bound
      |r(a - w)| <= r(a - |w|) for a totally monotone rational r.
  S3. barrier construction (lemma P2.L1): STRATO-structured B_delta with constant
      diagonal x tends to x I - eps N; entry (1, k+1) of r(B_delta) tends to
      (-1)^k r^(k)(x) eps^k / k!; negative entries for CN (x > 2), TR-BDF2
      (x > 1 + sqrt 2) and Richardson (every x >= 0); an order-2 r that is positive on
      every 2 x 2 member fails for N = 5.
  S4. synthetic mesh: CN exact positivity threshold from diag (I + h/2 B)^-1 >= 1/2,
      harmonic-fraction formula, sufficient bound h max B_jj <= M_r for CN and TR-BDF2,
      TR-BDF2 threshold of this matrix by bisection, local Richardson negative for every
      step, global Richardson negative at late times, BDF2 counterexample, exp(-hB) >= 0
      exactly in floating point by uniformisation with squaring.
  S5. near complete release (Lambda_1 t = 14, lam = 0.1 Lambda_1, S = 0): errors,
      observed orders, predicted leading error c mu_1^3 t h^2, negative values, one-step
      images of the kernel-loaded start, steps needed for 1e-3, CRP-6 1b step arithmetic
      with the Section 4.6 inputs, N^2 growth of max B_jj / Lambda_1 (homogeneous sphere).
  S6. inventory closure with decay and a switching source for CN, TR-BDF2, Richardson,
      BDF2 (modified inventory), ETD; steady state preserved by every scheme.

Run: .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec4_p2_positive_second_order.py
Writes nothing to disk. Prints PASS/FAIL lines and ALL PASS when every check passes.
"""
import math

import mpmath as mp
import numpy as np
import sympy as sp

ok_all = True


def report(name, ok, detail=""):
    global ok_all
    ok_all &= bool(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


# =========================================================================== S1 symbolic
x = sp.symbols("x", nonnegative=True)
s2 = sp.sqrt(2)
gam = 2 - s2
beta = (1 - gam) / (2 - gam)
r_BE = 1 / (1 + x)
r_CN = (1 - x / 2) / (1 + x / 2)
r1 = (1 - gam * x / 2) / (1 + gam * x / 2)                    # trapezoid over gamma h
a_tr = 1 / (gam * (2 - gam))
b_tr = (1 - gam) ** 2 / (gam * (2 - gam))
r_TR = (a_tr * r1 - b_tr) / (1 + beta * x)                     # BDF2 on t^n, t^n + gamma h, t^n+1
r_RE = 2 / (1 + x / 2) ** 2 - 1 / (1 + x)                      # local Richardson of BE

# variable-step BDF2 coefficients on the nonuniform points (omega = (1 - gamma)/gamma)
om = (1 - gam) / gam
report("S1a TR-BDF2 stage 2 equals variable-step BDF2 with omega = (1-gamma)/gamma",
       sp.simplify((1 + om) ** 2 / (1 + 2 * om) - a_tr) == 0 and sp.simplify(om ** 2 / (1 + 2 * om) - b_tr) == 0
       and sp.simplify((1 - gam) * (1 + om) / (1 + 2 * om) - beta) == 0, "")
report("S1b TR-BDF2: beta = gamma/2 = 1 - 1/sqrt2 (one matrix I + beta h B in both stages), a - b = 1",
       sp.simplify(beta - gam / 2) == 0 and sp.simplify(beta - (1 - 1 / s2)) == 0 and sp.simplify(a_tr - b_tr - 1) == 0,
       f"beta = {float(beta):.6f}, a = {float(a_tr):.6f}, b = {float(b_tr):.6f}")
u = 1 + beta * x
A_pf, B_pf = -s2, 1 + s2
report("S1c TR-BDF2 r = -sqrt2/(1+beta x) + (1+sqrt2)/(1+beta x)^2 (exact partial fractions)",
       sp.simplify(sp.together(r_TR - (A_pf / u + B_pf / u ** 2)).subs(x, sp.Rational(7, 3))) == 0
       and sp.expand(sp.numer(sp.together(sp.radsimp(sp.together(r_TR - (A_pf / u + B_pf / u ** 2)))))) == 0, "")
r_TR = A_pf / u + B_pf / u ** 2                                # proved equal above; simpler form used below


def series_coeffs(expr, n=5):
    ser = sp.series(expr, x, 0, n).removeO()
    return [sp.radsimp(sp.expand(ser.coeff(x, k))) for k in range(n)]


orders, cons = {}, {}
for name, r in (("BE", r_BE), ("CN", r_CN), ("TR-BDF2", r_TR), ("Richardson", r_RE)):
    d = series_coeffs(r - sp.exp(-x), 5)
    p = next(k for k in range(5) if sp.simplify(d[k]) != 0) - 1
    lc = series_coeffs(sp.log(r) + x, 5)
    orders[name] = p
    cons[name] = sp.simplify(lc[p + 1])
# BDF2 principal root of (3/2 + x) xi^2 - 2 xi + 1/2 = 0
xi1 = (2 + sp.sqrt(1 - 2 * x)) / (3 + 2 * x)
dB = series_coeffs(xi1 - sp.exp(-x), 5)
orders["BDF2"] = next(k for k in range(5) if sp.simplify(dB[k]) != 0) - 1
cons["BDF2"] = sp.simplify(series_coeffs(sp.log(xi1) + x, 5)[3])
xi2 = (2 - sp.sqrt(1 - 2 * x)) / (3 + 2 * x)
alpha_st = (1 / (1 + x) - xi2) / (xi1 - xi2)                   # amplitude of the principal root after a BE start
al = series_coeffs(alpha_st - 1, 4)
report("S1d0 BDF2 started by one BE step: principal-root amplitude alpha = 1 + (3/4) x^2 + O(x^3)",
       al[0] == 0 and al[1] == 0 and sp.simplify(al[2] - sp.Rational(3, 4)) == 0, f"coefficients {al[:3]}")
report("S1d orders: BE 1, CN 2, TR-BDF2 2, Richardson 2, BDF2 2",
       orders == {"BE": 1, "CN": 2, "TR-BDF2": 2, "Richardson": 2, "BDF2": 2}, f"{orders}")
report("S1e mode-error constants c in ln r(x) = -x + c x^(p+1): BE 1/2, CN -1/12, TR-BDF2 2/3 - sqrt2/2, Richardson 1/6, BDF2 -1/3",
       sp.simplify(cons["BE"] - sp.Rational(1, 2)) == 0 and sp.simplify(cons["CN"] + sp.Rational(1, 12)) == 0
       and sp.simplify(cons["Richardson"] - sp.Rational(1, 6)) == 0 and sp.simplify(cons["TR-BDF2"] - (sp.Rational(2, 3) - s2 / 2)) == 0 and sp.simplify(cons["BDF2"] + sp.Rational(1, 3)) == 0,
       "; ".join(f"{k}: {sp.simplify(v)} = {float(v):+.6f}" for k, v in cons.items()))
c_num = {k: float(v) for k, v in cons.items()}

# zeros, minima, limits
zCN = sp.solve(sp.numer(sp.together(r_CN)), x)
zTR = sp.solve(sp.numer(sp.together(A_pf * u + B_pf)), x)
xmin_TR = (-2 * B_pf / A_pf - 1) / beta
rmin_TR = sp.simplify((A_pf / u + B_pf / u ** 2).subs(x, xmin_TR))
report("S1f CN r(2) = 0, r < 0 for x > 2, r -> -1; TR-BDF2 r(1+sqrt2) = 0, min r = -(sqrt2-1)/2 at x = 4+3 sqrt2, r -> 0",
       zCN == [2] and sp.simplify(zTR[0] - (1 + s2)) == 0 and sp.simplify(rmin_TR + (s2 - 1) / 2) == 0
       and sp.simplify(xmin_TR - (4 + 3 * s2)) == 0 and sp.limit(r_CN, x, sp.oo) == -1 and sp.limit(r_TR, x, sp.oo) == 0,
       f"TR-BDF2 min {float(rmin_TR):.6f} at x = {float(xmin_TR):.4f}")
zRE = [z for z in sp.solve(sp.numer(sp.together(r_RE)), x) if z.is_positive]
xs = np.linspace(0.0, 400.0, 400001)
fRE = sp.lambdify(x, r_RE, "numpy")
report("S1g Richardson: r = 0 at x = 2 + 2 sqrt2, r < 0 beyond, r -> 0",
       len(zRE) == 1 and sp.simplify(zRE[0] - (2 + 2 * s2)) == 0 and sp.limit(r_RE, x, sp.oo) == 0
       and np.all(fRE(xs[xs > 2 + 2 * math.sqrt(2) + 1e-9]) < 0),
       f"min r on [0,400] = {fRE(xs).min():.6f} at x = {xs[np.argmin(fRE(xs))]:.3f}")

# total monotonicity: sign of (-1)^k r^(k)(x), k = 0..K, on a grid
def tm_ok(r, xgrid, K=14):
    out = []
    for k in range(K + 1):
        f = sp.lambdify(x, (-1) ** k * sp.diff(r, x, k), "mpmath")
        out.append(all(f(mp.mpf(xv)) >= 0 for xv in xgrid))
    return all(out)


grid_in = [0, 0.25, 0.5, 1, 1.5, 1.99]
report("S1h CN totally monotone on [0, 2): (-1)^k r^(k) >= 0, k <= 14", tm_ok(r_CN, grid_in), "")
grid_tr = [0, 0.5, 1, 2, 2.4, float(1 + math.sqrt(2)) - 1e-9]
report("S1i TR-BDF2 totally monotone on [0, 1+sqrt2], k <= 14 (sign of -sqrt2 (1+beta x) + (k+1)(1+sqrt2))",
       tm_ok(r_TR, grid_tr), "")
# Richardson: (-1)^k r^(k)(x) = k! [8(k+1)(1+x)^(k+1)/(2+x)^(k+2) - 1]/(1+x)^(k+1)
kk = sp.symbols("k", integer=True, nonnegative=True)
ok_form = all(abs(float(((-1) ** k * sp.diff(r_RE, x, k)
              - sp.factorial(k) * (8 * (k + 1) * (1 + x) ** (k + 1) / (2 + x) ** (k + 2) - 1) / (1 + x) ** (k + 1)).subs(x, xv))) < 1e-12
              for k in range(7) for xv in (0, sp.Rational(1, 3), 2, 17))
first_bad = {}
for xv in (0.0, 0.1, 1.0, 3.0, 10.0, 100.0):
    k = 0
    while 8 * (k + 1) * (1 + xv) ** (k + 1) / (2 + xv) ** (k + 2) >= 1:
        k += 1
    first_bad[xv] = k
report("S1j Richardson closed form of (-1)^k r^(k) holds (k <= 6); some k fails at every x (radius 0); x = 0 fails first at k = 4 (c_4 = -3/8)",
       ok_form and first_bad[0.0] == 4 and sp.simplify(series_coeffs(r_RE, 6)[4] - sp.Rational(-3, 8)) == 0,
       f"first failing k: {first_bad}")
# BDF2: complex roots for x > 1/2, |xi|^2 = 1/(3 + 2x) < 1
xis = [np.roots([1.5 + xv, -2.0, 0.5]) for xv in (0.6, 1.0, 10.0, 1e4)]
report("S1k BDF2 roots complex for x > 1/2 with |xi|^2 = 1/(3+2x); real and in (0,1] for 0 <= x <= 1/2",
       all(np.iscomplex(z).all() for z in xis)
       and all(abs(abs(z[0]) ** 2 - 1 / (3 + 2 * xv)) < 1e-14 for z, xv in zip(xis, (0.6, 1.0, 10.0, 1e4)))
       and all(np.isreal(np.roots([1.5 + xv, -2.0, 0.5])).all() and np.all(np.roots([1.5 + xv, -2.0, 0.5]).real > 0)
               for xv in (0.0, 0.2, 0.49)), "")
seq = [1.0, 1.0]
for _ in range(12):
    seq.append((2 * seq[-1] - 0.5 * seq[-2]) / 2.5)          # x = 1, positive start
report("S1l BDF2 scalar x = 1 from y0 = y1 = 1 changes sign", min(seq) < 0, f"first negative at n = {next(i for i, v in enumerate(seq) if v < 0)}")

# =========================================================================== S2 proof mechanics (lemma P2.L2)
a0 = 5.0
f_be = lambda t: 1.0 / (1.0 + t)
w = np.array([(a0 / (1 + a0)) ** k / (1 + a0) for k in range(4000)])   # b_k a^k, b_k = (-1)^k r^(k)(a)/k! = (1+a)^-(k+1)
tk = np.arange(4000) / a0
id1 = abs(w.sum() - 1.0)
id2 = abs((w * tk).sum() - 1.0)                                # -r'(0) = 1
id3 = abs((w * tk * (tk - 1 / a0)).sum() - 2.0)                # r''(0) = 2 for BE
report("S2a Taylor weights at a: sum w = r(0), sum w t = -r'(0), sum w t (t - 1/a) = r''(0) (BE, a = 5)",
       max(id1, id2, id3) < 1e-12, f"{id1:.1e} {id2:.1e} {id3:.1e}")
rng = np.random.default_rng(1)
zs = a0 + 5.9 * np.sqrt(rng.random(2000)) * np.exp(2j * np.pi * rng.random(2000))
okp = all(abs(1 / (1 + z)) <= f_be(a0 - abs(z - a0)) * (1 + 1e-12) for z in zs)
report("S2b Pringsheim bound |r(z)| <= r(a - |z - a|) inside the disc of radius a + 1 (BE, 2000 points)", okp, "")

# =========================================================================== S3 barrier construction (lemma P2.L1)
def rat_matrix(r, M):
    num, den = sp.fraction(sp.together(r))
    pn = sp.Poly(sp.expand(num), x).all_coeffs()[::-1]
    pd = sp.Poly(sp.expand(den), x).all_coeffs()[::-1]
    n = M.shape[0]
    P = sum(float(c) * np.linalg.matrix_power(M, k) for k, c in enumerate(pn))
    Qm = sum(float(c) * np.linalg.matrix_power(M, k) for k, c in enumerate(pd))
    return np.linalg.solve(Qm, P) if np.ndim(P) else P * np.linalg.inv(Qm)


def strato_family(n, xv, eps, delta):
    """B = V^-1 K + lam I, V_j = delta^-(j-1), g_j = eps V_j, G_R = eps V_n, lam = x - eps (h = 1)."""
    Vd = np.array([delta ** (-j) for j in range(n)])
    g = eps * Vd[:-1]
    K = np.zeros((n, n))
    for j in range(n - 1):
        K[j, j] += g[j]; K[j + 1, j + 1] += g[j]; K[j, j + 1] -= g[j]; K[j + 1, j] -= g[j]
    GRf = eps * Vd[-1]
    K[-1, -1] += GRf
    lamf = xv - eps
    return K, Vd, lamf, K / Vd[:, None] + lamf * np.eye(n)


n6, eps6, del6 = 6, 0.05, 1e-7
structure_ok, lim_ok, details = True, True, []
for name, r, xv in (("CN", r_CN, 1.0), ("TR-BDF2", r_TR, 1.5), ("Richardson", r_RE, 0.3)):
    K6, V6, l6, B6 = strato_family(n6, xv, eps6, del6)
    rs = K6.sum(axis=1)
    structure_ok &= np.allclose(K6, K6.T) and np.all(np.abs(rs[:-1]) <= 1e-9 * np.abs(K6).max()) and rs[-1] > 0 and l6 >= 0
    RB = rat_matrix(r, B6)
    pred = [float(((-1) ** k * sp.diff(r, x, k)).subs(x, xv)) * eps6 ** k / math.factorial(k) for k in range(n6)]
    err = max(abs(RB[0, k] - pred[k]) / max(abs(pred[k]), 1e-300) for k in range(n6))
    lim_ok &= err < 1e-4
    details.append(f"{name} x={xv}: max rel {err:.1e}, entry(1,5) {RB[0, 4]:+.3e}")
report("S3a B_delta is STRATO-structured (K symmetric, zero row sums except the last, lam >= 0)", structure_ok, "")
report("S3b entry (1,k+1) of r(B_delta) -> (-1)^k r^(k)(x) eps^k/k! as delta -> 0 (delta = 1e-7)", lim_ok, "; ".join(details))
K6, V6, l6, B6 = strato_family(n6, 0.3, eps6, del6)
neg_RE = rat_matrix(r_RE, B6).min()
K6, V6, l6, B6 = strato_family(n6, 2.5, eps6, del6)
neg_CN = rat_matrix(r_CN, B6).min()
K6, V6, l6, B6 = strato_family(n6, 2.6, eps6, del6)
neg_TR = rat_matrix(r_TR, B6).min()
report("S3c negative entries in the class: Richardson at x = 0.3, CN at x = 2.5, TR-BDF2 at x = 2.6",
       neg_RE < 0 and neg_CN < 0 and neg_TR < 0, f"min entries {neg_RE:.2e}, {neg_CN:.2e}, {neg_TR:.2e}")

# fixed size matters: r = 1/(1 + x + x^2/2) has order 2, is positive on every 2 x 2 member, fails for N = 5
r_q = 1 / (1 + x + x ** 2 / 2)
okN2 = True
rng2 = np.random.default_rng(7)
for _ in range(3000):
    V2 = 10 ** rng2.uniform(-3, 3, 2); g2 = 10 ** rng2.uniform(-3, 3); G2 = 10 ** rng2.uniform(-3, 3)
    l2 = 10 ** rng2.uniform(-4, 2) * rng2.integers(0, 2)
    K2 = np.array([[g2, -g2], [-g2, g2 + G2]])
    B2 = (K2 / V2[:, None] + l2 * np.eye(2)) * 10 ** rng2.uniform(-3, 3)
    R2 = np.linalg.solve(np.eye(2) + B2 + B2 @ B2 / 2, np.eye(2))
    okN2 &= bool(np.all(R2 >= -1e-13 * np.abs(R2).max()))
K5, V5, l5, B5 = strato_family(5, 0.1, 0.05, 1e-7)
R5 = np.linalg.solve(np.eye(5) + B5 + B5 @ B5 / 2, np.eye(5))
report("S3d order-2 r = 1/(1+x+x^2/2): r(B) >= 0 on 3000 random 2 x 2 members; negative entry (1,5) for N = 5 (x = 0.1)",
       okN2 and R5[0, 4] < 0 and sp.series(r_q - sp.exp(-x), x, 0, 3).removeO() == 0, f"entry(1,5) = {R5[0, 4]:.2e}")

# =========================================================================== synthetic mesh (transcribed)
def build_mesh(radii, Ds, ncells):
    left, right, D, mat = [], [], [], []
    inner = 0.0
    for mm, (Ro, Dm, n) in enumerate(zip(radii, Ds, ncells)):
        wd = (Ro - inner) / n
        for k in range(n):
            left.append(inner + k * wd)
            right.append(Ro if k + 1 == n else inner + (k + 1) * wd)
            D.append(Dm)
            mat.append(mm)
        inner = Ro
    left, right, D = np.array(left), np.array(right), np.array(D)
    V = 4 * math.pi / 3 * (right ** 3 - left ** 3)
    cen = np.array([0.75 * b if a == 0 else 0.75 * (b ** 4 - a ** 4) / (b ** 3 - a ** 3) for a, b in zip(left, right)])
    G = np.array([4 * math.pi * right[j] ** 2 / ((right[j] - cen[j]) / D[j] + (cen[j + 1] - right[j]) / D[j + 1])
                  for j in range(len(V) - 1)])
    GR = 4 * math.pi * radii[-1] ** 2 * D[-1] / (radii[-1] - cen[-1])
    return dict(V=V, gw=np.concatenate(([0.0], G)), ge=np.concatenate((G, [GR])), GR=GR, mat=np.array(mat), n=len(V))


def solve_sf(v, gw, ge, rhs):
    n = len(v)
    den, dp = np.empty(n), np.empty(n)
    e_prev = 0.0
    for i in range(n):
        e = v[i] if i == 0 else v[i] + gw[i] * e_prev / (e_prev + gw[i])
        den[i] = e + ge[i]
        dp[i] = (rhs[i] + (gw[i] * dp[i - 1] if i else 0.0)) / den[i]
        e_prev = e
    out = np.empty(n)
    out[-1] = dp[-1]
    for i in range(n - 2, -1, -1):
        out[i] = dp[i] + ge[i] / den[i] * out[i + 1]
    return out


radii = [250e-6, 350e-6, 390e-6, 425e-6, 465e-6]
Ds = [4e-14, 4e-13, 2e-14, 5e-15, 2e-14]
m = build_mesh(radii, Ds, [20, 8, 4, 4, 4])
Vv, gw, ge, GR, N = m["V"], m["gw"], m["ge"], m["GR"], m["n"]
kern = (m["mat"] == 0).astype(float)
C0 = kern.copy()
M0 = float(Vv @ C0)
K = np.zeros((N, N))
for i in range(N):
    K[i, i] = gw[i] + ge[i]
    if i:
        K[i, i - 1] = -gw[i]
    if i + 1 < N:
        K[i, i + 1] = -ge[i]
sq = np.sqrt(Vv)
mu0, Qe = np.linalg.eigh(K / np.outer(sq, sq))
Lam1 = mu0.min()
lam = 0.1 * Lam1
B = K / Vv[:, None] + lam * np.eye(N)
Bjj = np.diag(B).copy()
bmax = Bjj.max()
mu = mu0 + lam
print(f"      data: N = {N}, Lambda_1 = {Lam1:.5e} s^-1, lam = {lam:.5e} s^-1, max_j B_jj = {bmax:.5e} s^-1, "
      f"max_j B_jj / Lambda_1 = {bmax / Lam1:.4e}, mu_max/mu_1 = {mu.max() / mu.min():.4e}")


def Qapply(tau, Cv):
    """(I + tau B)^-1 C by the subtraction-free solver (= Backward Euler step of length tau, S = 0)."""
    vt = Vv / tau
    return solve_sf(vt + lam * Vv, gw, ge, vt * Cv)


def Qmat(tau):
    return np.column_stack([Qapply(tau, np.eye(N)[:, j]) for j in range(N)])


# =========================================================================== S4 positivity on the synthetic mesh
def H(a, b):
    return a * b / (a + b) if (a > 0 and b > 0) else 0.0


def cn_diag_formula(h):
    """1/(V^-1-scaled) diag of (I + tau B)^-1, tau = h/2, by forward/backward excess recurrences."""
    tau = h / 2
    vp = Vv * (1 + tau * lam)
    e = np.empty(N); f = np.empty(N)
    e[0] = vp[0]
    for j in range(1, N):
        e[j] = vp[j] + H(e[j - 1], tau * gw[j])
    f[N - 1] = vp[N - 1] + tau * GR
    for j in range(N - 2, -1, -1):
        f[j] = vp[j] + H(f[j + 1], tau * ge[j])
    den = np.array([vp[j] + (tau * GR if j == N - 1 else 0.0) + (H(e[j - 1], tau * gw[j]) if j else 0.0)
                    + (H(f[j + 1], tau * ge[j]) if j < N - 1 else 0.0) for j in range(N)])
    return Vv / den                                            # = diag of (I + tau B)^-1


h_test = 3.0 / bmax
dq = np.diag(Qmat(h_test / 2))
report("S4a diag (I + h/2 B)^-1 from the harmonic-fraction formula equals the solver value",
       np.max(np.abs(cn_diag_formula(h_test) - dq) / dq) < 1e-13, f"max rel {np.max(np.abs(cn_diag_formula(h_test) - dq) / dq):.1e}")
Rcn = 2 * Qmat(h_test / 2) - np.eye(N)
Rcn_e = Qe @ np.diag((1 - h_test * mu / 2) / (1 + h_test * mu / 2)) @ Qe.T
Rcn_e = Rcn_e / sq[:, None] * sq[None, :]
report("S4b r_CN(hB) = 2 (I + h/2 B)^-1 - I (exact identity; off-diagonal entries >= 0 for every h)",
       np.max(np.abs(Rcn - Rcn_e)) < 1e-12 and np.all(Rcn - np.diag(np.diag(Rcn)) >= 0.0), f"max abs diff {np.max(np.abs(Rcn - Rcn_e)):.1e}")
# exact CN threshold by bisection on min_j diag = 1/2 (diag is strictly decreasing in h)
hs = np.logspace(-2, 2, 9) / bmax
mono = all(np.all(np.diff([cn_diag_formula(hh)[j] for hh in hs]) < 0) for j in range(N))
lo, hi = 0.5 / bmax, 50 / bmax
for _ in range(200):
    mid = math.sqrt(lo * hi)
    (lo, hi) = (mid, hi) if cn_diag_formula(mid).min() >= 0.5 else (lo, mid)
hstar_cn = lo
jstar = int(np.argmin(cn_diag_formula(hi)))
ok_th = np.all(2 * Qmat(hstar_cn * (1 - 1e-9) / 2) - np.eye(N) >= 0.0) and (2 * Qmat(hstar_cn * 1.001 / 2) - np.eye(N)).min() < 0
report("S4c CN positive exactly for h <= h*_CN; h*_CN >= 2/max B_jj (Bolley-Crouzeix bound) ",
       mono and ok_th and hstar_cn >= 2 / bmax,
       f"h*_CN = {hstar_cn:.5e} s, h*_CN max B_jj = {hstar_cn * bmax:.4f}, Lambda_1 h*_CN = {Lam1 * hstar_cn:.4e}, binding cell j = {jstar + 1} (material {m['mat'][jstar]})")


def R_TR(h):
    Qb = Qmat(beta_f * h)
    return (1 + math.sqrt(2)) * Qb @ Qb - math.sqrt(2) * Qb, (1 + math.sqrt(2)) * Qb @ Qb + math.sqrt(2) * Qb


def R_RE(h):
    Qh2 = Qmat(h / 2)
    return 2 * Qh2 @ Qh2 - Qmat(h), 2 * Qh2 @ Qh2 + Qmat(h)


beta_f = float(beta)
M_TR = 1 + math.sqrt(2)


def sig_neg(Rs):
    Rm, scale = Rs
    return bool(np.any(Rm < -1e-12 * scale)), float((Rm / np.where(scale > 0, scale, 1)).min())


Rt, St = R_TR(M_TR / bmax)
report("S4d TR-BDF2 positive at h max B_jj = 1 + sqrt2 (sufficient bound of lemma P2.L3)", not sig_neg((Rt, St))[0],
       f"min entry {Rt.min():.3e}")
hgrid = np.logspace(math.log10(0.5 / bmax), math.log10(20 / Lam1), 160)
ineg = next((i for i, hh in enumerate(hgrid) if sig_neg(R_TR(hh))[0]), None)
first_neg_tr = hgrid[ineg]
lo_t, hi_t = hgrid[ineg - 1], hgrid[ineg]
for _ in range(40):
    mid = math.sqrt(lo_t * hi_t)
    (lo_t, hi_t) = (lo_t, mid) if sig_neg(R_TR(mid))[0] else (mid, hi_t)
hstar_tr = lo_t
report("S4e TR-BDF2 on the synthetic matrix: positive on every grid step below the first failure, which lies beyond the bound",
       ineg > 0 and hstar_tr * bmax > M_TR,
       f"first failure between grid points, bisected: h*_TR max B_jj = {hstar_tr * bmax:.3f}, Lambda_1 h*_TR = {Lam1 * hstar_tr:.3e}")
neg_all_re = all(sig_neg(R_RE(hh))[0] for hh in np.logspace(math.log10(1e-3 / bmax), math.log10(20 / Lam1), 60))
hsm = 0.02 / bmax
Rr, Sr = R_RE(hsm)
prod4 = np.prod([-B[i, i + 1] for i in range(4)])
pred15 = -3 / 8 * hsm ** 4 * prod4
report("S4f Richardson-BE has negative entries for every h on a 60-point grid (1e-3/max B_jj to 20/Lambda_1); small-h entry (1,5) ~ -3/8 h^4 prod b",
       neg_all_re and Rr[0, 4] < 0 and abs(Rr[0, 4] / pred15 - 1) < 0.1, f"entry(1,5) {Rr[0, 4]:.3e}, predicted {pred15:.3e}")
# global Richardson 2 C_{h/2}(t) - C_h(t): at fixed h it turns negative everywhere once (a^2/b)^n > 2
hg = 0.5 / (Lam1 + lam)
Wg = {n: 2 * np.linalg.matrix_power(Qmat(hg / 2), 2 * n) - np.linalg.matrix_power(Qmat(hg), n) for n in (10, 40)}
n_cross = math.log(2) / math.log((1 + 0.25) ** 2 / 1.5)
report("S4i global Richardson (whole runs) at mu_1 h = 0.5: entries not all negative at n = 10, all negative at n = 40 (crossing n = ln2/ln(a^2/b))",
       not np.all(Wg[10] < 0) and np.all(Wg[40] < 0), f"crossing n = {n_cross:.1f}")
Cbd = -Qapply(2 * 1e3 / 3, np.eye(N)[:, 25]) / 3
report("S4g BDF2: C^n = 0, C^{n-1} = e_j >= 0 gives C^{n+1} = -(1/3)(I + 2h/3 B)^-1 e_j < 0 for every h", np.all(Cbd < 0), f"max {Cbd.max():.2e}")


def expm_eigh(h):
    E = Qe @ np.diag(np.exp(-h * mu)) @ Qe.T
    return E / sq[:, None] * sq[None, :]


def expm_unif(h, q=14):
    """exp(-hB) = (exp(-tau beta) T_q(tau P))^(2^m), P = beta I - B >= 0, tau beta <= 1/8: subtraction-free."""
    bt = bmax
    P = bt * np.eye(N) - B
    P[P < 0] = 0.0                                           # diagonal of P is >= 0 analytically; clears round-off
    msq = max(0, math.ceil(math.log2(8 * h * bt)))
    tau = h / 2 ** msq
    T = np.eye(N); term = np.eye(N)
    for k in range(1, q + 1):
        term = term @ (tau * P) / k
        T = T + term
    E = math.exp(-tau * bt) * T
    for _ in range(msq):
        E = E @ E
    return E, msq


res_e = []
for hh in (1e-2 / Lam1, 1 / Lam1, 14 / Lam1):
    Eu, msq = expm_unif(hh)
    Ee = expm_eigh(hh)
    res_e.append((Eu.min() >= 0.0, Ee.min(), np.linalg.norm(Eu - Ee) / np.linalg.norm(Ee), msq,
                  abs(Vv @ (Eu @ C0) / (Vv @ (Ee @ C0)) - 1)))
report("S4h exp(-hB) >= 0 exactly in floating point by uniformisation and squaring; agrees with the eigen route",
       all(r_[0] for r_ in res_e) and all(r_[2] < 1e-9 and r_[4] < 1e-9 for r_ in res_e),
       "; ".join(f"m={r_[3]}: eigen min {r_[1]:+.1e}, rel {r_[2]:.1e}, M rel {r_[4]:.1e}" for r_ in res_e))

# =========================================================================== S5 near complete release
T_end = 14.0 / Lam1
c_ref = (Qe @ (np.exp(-mu * T_end) * (Qe.T @ (sq * C0)))) / sq
Mref = float(Vv @ c_ref)
mp.mp.dps = 30
Sm = mp.matrix(N, N)
for i in range(N):
    for j in range(N):
        Sm[i, j] = mp.mpf(float(K[i, j])) / (mp.sqrt(mp.mpf(float(Vv[i]))) * mp.sqrt(mp.mpf(float(Vv[j]))))
Ev, Qm = mp.eigsy(Sm)
z0 = [mp.sqrt(mp.mpf(float(Vv[i]))) * mp.mpf(float(C0[i])) for i in range(N)]
Mmp = mp.mpf(0)
for k in range(N):
    proj = mp.fsum(Qm[i, k] * z0[i] for i in range(N))
    Mmp += mp.exp(-(Ev[k] + mp.mpf(lam)) * mp.mpf(T_end)) * proj * mp.fsum(Qm[i, k] * mp.sqrt(mp.mpf(float(Vv[i]))) for i in range(N))
report("S5a reference M(t_end) from the double eigen route equals the 30-digit mpmath value",
       abs(Mref / float(Mmp) - 1) < 1e-10, f"M(t_end)/M0 = {Mref / M0:.6e}, rel diff {abs(Mref / float(Mmp) - 1):.1e}")


def run(scheme, n):
    h = T_end / n
    C = C0.copy(); Cm = None; cmin = 0.0
    for k in range(n):
        if scheme == "BE":
            Cn = Qapply(h, C)
        elif scheme == "CN":
            Cn = 2 * Qapply(h / 2, C) - C
        elif scheme == "TR-BDF2":
            Y = 2 * Qapply(beta_f * h, C) - C
            Cn = Qapply(beta_f * h, a_f * Y - b_f * C)
        elif scheme == "Richardson":
            Cn = 2 * Qapply(h / 2, Qapply(h / 2, C)) - Qapply(h, C)
        elif scheme == "BDF2":
            Cn = Qapply(h, C) if Cm is None else Qapply(2 * h / 3, (4 * C - Cm) / 3)
            Cm = C
        elif scheme == "ETD":
            Cn = Eh @ C
        C = Cn
        cmin = min(cmin, C.min())
    return float(Vv @ C), cmin


a_f, b_f = float(a_tr), float(b_tr)
ns = [50, 100, 200, 400, 800, 1600, 3200]
tab = {}
for sch in ("BE", "CN", "TR-BDF2", "Richardson", "BDF2", "ETD"):
    rows = []
    for n in ns:
        if sch == "ETD":
            Eh = expm_eigh(T_end / n)
        Mn, cmin = run(sch, n)
        rows.append(((Mn - Mref) / Mref, cmin))
    tab[sch] = rows
    errs = [abs(e) for e, _ in rows]
    p_last = math.log2(errs[-2] / errs[-1]) if errs[-1] > 0 else float("nan")
    print(f"      {sch:10s} rel err M(t_end): " + " ".join(f"{e:+.2e}" for e, _ in rows)
          + f" | order(last pair) {p_last:.3f} | min C over run " + " ".join(f"{c:+.1e}" for _, c in rows))


def order_last(sch):
    e = [abs(v) for v, _ in tab[sch]]
    return math.log2(e[-2] / e[-1])


mu1 = Lam1 + lam
pred = {k: c_num[k] * mu1 ** 3 * T_end * (T_end / 3200) ** 2 for k in ("CN", "TR-BDF2", "Richardson", "BDF2")}
pred["BDF2"] += 0.75 * (mu1 * T_end / 3200) ** 2               # BE start, amplitude term of S1d0
pred["BE"] = 0.5 * mu1 ** 2 * T_end * (T_end / 3200)
report("S5b observed orders (last pair): BE 1, CN, TR-BDF2, Richardson, BDF2 2 (within 0.05)",
       abs(order_last("BE") - 1) < 0.05 and all(abs(order_last(s) - 2) < 0.05 for s in ("CN", "TR-BDF2", "Richardson", "BDF2")),
       ", ".join(f"{s} {order_last(s):.3f}" for s in ("BE", "CN", "TR-BDF2", "Richardson", "BDF2")))
report("S5c leading-order prediction c mu_1^(p+1) t h^p (BDF2 plus (3/4)(mu_1 h)^2) of the relative error at n = 3200 (within 5 %, sign included)",
       all(abs(tab[s][-1][0] / pred[s] - 1) < 0.05 for s in pred),
       ", ".join(f"{s} obs/pred {tab[s][-1][0] / pred[s]:.4f}" for s in pred))
report("S5d ETD (exact exponential) error at round-off for every n", all(abs(e) < 1e-9 for e, _ in tab["ETD"]),
       f"max {max(abs(e) for e, _ in tab['ETD']):.1e}")
neg_n = {s: [n for n, (_, c) in zip(ns, tab[s]) if c < 0] for s in tab}
report("S5e negative cell values in these runs: BE and ETD none (BE >= 0.0 exactly); CN for n <= 1600 and M(t_end) < 0 at n = 50; "
       "Richardson for n >= 800; TR-BDF2 and BDF2 none",
       neg_n["BE"] == [] and neg_n["ETD"] == [] and neg_n["CN"] == ns[:-1] and tab["CN"][0][0] < -1
       and neg_n["Richardson"] == [800, 1600, 3200] and neg_n["TR-BDF2"] == [] and neg_n["BDF2"] == [],
       "; ".join(f"{s}: n = {v}" for s, v in neg_n.items()))
# TR-BDF2, Richardson and CN applied to the kernel-loaded start: where the one-step image turns negative
rows_w = []
for hx in (5.0, 100.0, 1e3, 3e3, 1e4, 1e5):
    hh = hx / bmax
    rows_w.append((hx, Lam1 * hh, R_TR(hh)[0].min(), (R_TR(hh)[0] @ C0).min(), (R_RE(hh)[0] @ C0).min(),
                   ((2 * Qmat(hh / 2) - np.eye(N)) @ C0).min()))
for rw in rows_w:
    print(f"      h max B_jj = {rw[0]:8.0f}, Lambda_1 h = {rw[1]:.2e}: TR-BDF2 min entry {rw[2]:+.2e}, min r(hB)C0: "
          f"TR-BDF2 {rw[3]:+.2e}, Richardson {rw[4]:+.2e}, CN {rw[5]:+.2e}")
report("S5f kernel-loaded start: TR-BDF2 image r(hB)C0 >= 0 up to Lambda_1 h = 0.42 although r(hB) has negative entries; negative at Lambda_1 h = 1.4",
       all(rw[3] >= 0 for rw in rows_w[:4]) and rows_w[4][3] < 0 and all(rw[2] < 0 for rw in rows_w[1:]), "")
# steps needed for |relative error| = 1e-3 at t_end (leading order), and positivity caps
eps1 = 1e-3
n_need = {"BE": 0.5 * (mu1 * T_end) ** 2 / eps1}
for s in ("CN", "TR-BDF2", "Richardson", "BDF2"):
    n_need[s] = math.sqrt(abs(c_num[s] * (mu1 * T_end) ** 3 + (0.75 * (mu1 * T_end) ** 2 if s == "BDF2" else 0.0)) / eps1)
n_pos = {"CN (exact h*)": T_end / hstar_cn, "CN (2/max B_jj)": T_end * bmax / 2, "TR-BDF2 ((1+sqrt2)/max B_jj)": T_end * bmax / M_TR}
print("      steps for 1e-3 at Lambda_1 t = 14 (leading order): " + ", ".join(f"{k} {v:.0f}" for k, v in n_need.items()))
n_pos["TR-BDF2 (h*_TR of this matrix)"] = T_end / hstar_tr
print("      steps for guaranteed positivity: " + ", ".join(f"{k} {v:.0f}" for k, v in n_pos.items()))
# CRP-6 case 1b arithmetic with the Section 4.6 inputs Lambda_1 = 1.97e-5 s^-1, Lambda_1 t = 14.2 (lam neglected)
L1c, L1t = 1.97e-5, 14.2
hc = {"BE": 2 * eps1 / L1t / L1c}
for s_ in ("CN", "TR-BDF2", "Richardson"):
    hc[s_] = math.sqrt(eps1 / (abs(c_num[s_]) * L1t)) / L1c
print("      CRP-6 1b arithmetic, step for 1e-3 in 1 - F (leading order): " + ", ".join(f"{k} {v:.1f} s" for k, v in hc.items()))
# homogeneous sphere, uniform mesh: ratio max B_jj / Lambda_1 against N
ratios, l1s = {}, {}
for Nh in (40, 150):
    mh = build_mesh([1.0], [1.0], [Nh])
    Kh = np.diag(mh["gw"] + mh["ge"]) - np.diag(mh["gw"][1:], -1) - np.diag(mh["ge"][:-1], 1)
    sqh = np.sqrt(mh["V"])
    l1h = np.linalg.eigvalsh(Kh / np.outer(sqh, sqh)).min()
    ratios[Nh] = (np.diag(Kh) / mh["V"]).max() / l1h
    l1s[Nh] = l1h
report("S5g homogeneous sphere, uniform mesh: Lambda_1 near pi^2 D/R^2 and max B_jj/Lambda_1 grows like N^2",
       abs(ratios[150] / ratios[40] / (150 / 40) ** 2 - 1) < 0.05 and abs(l1s[150] / math.pi ** 2 - 1) < 0.01,
       ", ".join(f"N = {k}: max B_jj/Lambda_1 = {v:.4e}, CN cap Lambda_1 h <= {2 / v:.3e}" for k, v in ratios.items()))

# =========================================================================== S6 closure and steady state
lam_c = 2e-8
Bc = K / Vv[:, None] + lam_c * np.eye(N)
muc = mu0 + lam_c


def Qc(tau, Cv):
    vt = Vv / tau
    return solve_sf(vt + lam_c * Vv, gw, ge, vt * Cv)


sched = [(0.0, 3e7, 1e-6), (3e7, 5e7, 0.0), (5e7, 8e7, 2.5e-6)]


def phi_funcs(h):
    z = h * muc
    E = np.exp(-z)
    p1 = -np.expm1(-z) / muc                                  # h phi_1(-h mu)
    p2 = np.where(z > 1e-3, (z + np.expm1(-z)) / muc ** 2, h ** 2 * (0.5 - z / 6 + z ** 2 / 24))
    f = lambda d: (Qe @ np.diag(d) @ Qe.T) / sq[:, None] * sq[None, :]
    return f(E), f(p1), f(p2)


def closure(scheme):
    C = C0.copy(); Mgen = []; Mdec = []; Mrel = []
    Cm = None; resabs = []; M1 = None; cmin = 0.0
    for (ta, tb, Sk) in sched:
        tt = ta; dt = 1e3
        while tt < tb - 1e-6:
            hk = min(dt, tb - tt) if scheme != "BDF2" else 1e6
            s = kern * Sk
            Mold = float(Vv @ C)
            gen = hk * float(Vv @ s)
            if scheme == "CN":
                Cn = 2 * Qc(hk / 2, C + hk / 2 * s) - C           # (I + h/2 B) C1 = (I - h/2 B) C + h s
                dec = lam_c * hk / 2 * (float(Vv @ C) + float(Vv @ Cn))
                rel = GR * hk / 2 * (C[-1] + Cn[-1])
            elif scheme == "TR-BDF2":
                Y = 2 * Qc(beta_f * hk, C + beta_f * hk * s) - C   # (I + bh B) Y = (I - bh B) C + 2 bh s
                Cn = Qc(beta_f * hk, a_f * Y - b_f * C + beta_f * hk * s)
                wts = (a_f * beta_f, a_f * beta_f, beta_f)
                dec = lam_c * hk * sum(wi * float(Vv @ Z) for wi, Z in zip(wts, (C, Y, Cn)))
                rel = GR * hk * sum(wi * Z[-1] for wi, Z in zip(wts, (C, Y, Cn)))
            elif scheme == "Richardson":
                Z1 = Qc(hk / 2, C + hk / 2 * s); Z2 = Qc(hk / 2, Z1 + hk / 2 * s); Zf = Qc(hk, C + hk * s)
                Cn = 2 * Z2 - Zf
                dec = lam_c * (2 * hk / 2 * (float(Vv @ Z1) + float(Vv @ Z2)) - hk * float(Vv @ Zf))
                rel = GR * (2 * hk / 2 * (Z1[-1] + Z2[-1]) - hk * Zf[-1])
            elif scheme == "BDF2":
                if Cm is None:
                    Cn = Qc(hk, C + hk * s)
                else:
                    Cn = Qc(2 * hk / 3, (4 * C - Cm) / 3 + 2 * hk / 3 * s)
                dec = lam_c * hk * float(Vv @ Cn); rel = GR * hk * Cn[-1]
            elif scheme == "ETD":
                E, P1, P2 = phi_funcs(hk)
                Cn = E @ C + P1 @ s
                integ = P1 @ C + P2 @ s
                dec = lam_c * float(Vv @ integ); rel = GR * integ[-1]
            if scheme == "BDF2":
                if Cm is None:                                     # first step is BE: plain inventory
                    Mhat_new, Mhat_old, M1 = float(Vv @ Cn), Mold, float(Vv @ Cn)
                else:                                              # Mhat^k = (3 M^k - M^(k-1))/2
                    Mhat_new = 1.5 * float(Vv @ Cn) - 0.5 * float(Vv @ C)
                    Mhat_old = 1.5 * Mold - 0.5 * float(Vv @ Cm)
                resabs.append(abs((Mhat_new - Mhat_old) - (gen - dec - rel)))
                Cm = C
            else:
                resabs.append(abs((float(Vv @ Cn) - Mold) - (gen - dec - rel)))
            Mgen.append(gen); Mdec.append(dec); Mrel.append(rel)
            C = Cn; tt += hk; dt *= 1.5; cmin = min(cmin, C.min())
    Mend = float(Vv @ C)
    scale = M0 + math.fsum(Mgen)
    resmax = max(resabs) / scale
    if scheme == "BDF2":
        # cumulative closure with the modified inventory: Mhat^n = M^n + (M^n - M^{n-1})/2 after the BE start,
        # plus the start correction (M^1 - M^0)/2
        Mprev = float(Vv @ Cm)
        lhs = M0 + math.fsum(Mgen)
        rhs = Mend + 0.5 * (Mend - Mprev) - 0.5 * (M1 - M0) + math.fsum(Mdec) + math.fsum(Mrel)
        return resmax, abs(lhs - rhs) / lhs, cmin
    return resmax, abs(M0 + math.fsum(Mgen) - Mend - math.fsum(Mdec) - math.fsum(Mrel)) / scale, cmin


for sch in ("CN", "TR-BDF2", "Richardson", "BDF2", "ETD"):
    stp, cum, cmn = closure(sch)
    report(f"S6a closure with decay and switching source, scheme-consistent quadrature: {sch}",
           stp < 1e-12 and cum < 1e-12, f"max per-step residual / (M0 + M_gen) {stp:.1e}, cumulative {cum:.1e}, min C over run {cmn:+.1e} mol m^-3")
s_c = kern * 1e-6
Cstar = np.linalg.solve(Bc, s_c)
hbig = 10 / Lam1
E, P1, P2 = phi_funcs(hbig)
fp = {
    "CN": 2 * Qc(hbig / 2, Cstar + hbig / 2 * s_c) - Cstar,
    "TR-BDF2": Qc(beta_f * hbig, a_f * (2 * Qc(beta_f * hbig, Cstar + beta_f * hbig * s_c) - Cstar) - b_f * Cstar + beta_f * hbig * s_c),
    "Richardson": 2 * Qc(hbig / 2, Qc(hbig / 2, Cstar + hbig / 2 * s_c) + hbig / 2 * s_c) - Qc(hbig, Cstar + hbig * s_c),
    "BDF2": Qc(2 * hbig / 3, (4 * Cstar - Cstar) / 3 + 2 * hbig / 3 * s_c),
    "ETD": E @ Cstar + P1 @ s_c,
}
dev = {k: float(np.max(np.abs(v - Cstar)) / np.max(Cstar)) for k, v in fp.items()}
report("S6b semi-discrete steady state B^-1 s is a fixed point of every scheme (h = 10/Lambda_1)",
       all(v < 1e-10 for v in dev.values()), ", ".join(f"{k} {v:.1e}" for k, v in dev.items()))

print("ALL PASS" if ok_all else "SOME CHECKS FAILED")
