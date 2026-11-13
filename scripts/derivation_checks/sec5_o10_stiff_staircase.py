"""Open item O10: convergence order of the staircase (P22.53)-(P22.56) in the stiff regime.

What this script checks (labels refer to the O10 derivation section, equations (P2X.O10.k)):

  S0  symbolic identities used in the proofs (SymPy): the exact remainder of the harmonic
      face conductance under the per-layer average rule, the first and second time
      derivatives of ln G_f, the sharpness constant of the left rule, and two integrals.
  S1  face lemma: for random diffusivity vectors D, D' the generalised eigenvalues of
      (K(D) - K(D'), K(D')) lie in [min r - 1, max r - 1], r_i = D_i / D'_i, and the
      mixed Cauchy-Schwarz bound holds.
  S2  the duality (error) representation phi^T e(T) = -int w^T Delta z ds, on a small mesh.
  S3  Theorem A (order 1, uniform in stiffness): ||e(T)||_V <= (1/2) eta sqrt(||z0||^2 - ||z(T)||^2)
      for the left, midpoint and average rules on the stiff five-layer FV matrix
      (20 and 40 cells per layer, h||A|| from about 70 to 4.6e3).
  S4  Theorem C (commuting case, shared scaling): midpoint error <= |tau - tau~| / (e min tau)
      and <= h^2 sup|alpha''| / (24 e inf alpha); observed order 2; average rule exact.
  S5  Theorem B (temperature jumps): grid-aligned jump gives zero error for every rule;
      a misaligned jump obeys the O(h) bound of Theorem B.
  S6  smoothing lemma: ||z(t)||^2_K <= e^{Lt} ||z0||^2 / (2t) along the reference solution.
  S7  operator-relative constant Lambda_t = ||dK/dt K^{-1}||_2 / L against mesh refinement:
      grows like N^{1/2} for layer-specific Q, equals 1 for shared scaling.
  S8  observed orders in the V-norm and for the release fraction F in the window
      h||A|| in [1e2, 1.2e3] (Radau IIA reference), and reference-free (Cauchy) orders on
      a finer mesh (80 cells per layer).

The finite-volume matrix is a Python transcription of v1.0 Code B (build_mesh in
src/numerical/finite_volume.rs: equal-width cells per layer, volume centroids, two
half-cell resistances per face, absorbing outer face). All diffusion parameters are the
SYNTHETIC test values of sec5_staircase.py, not material data.

Run from the repository root:
    .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec5_o10_stiff_staircase.py
Runtime: a few minutes. The script writes no files.
"""
import math

import mpmath as mp
import numpy as np
import sympy as sp
from scipy.integrate import solve_ivp
from scipy.linalg import eigh, eigh_tridiagonal

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


# ---------------------------------------------------------------------------
# S0 symbolic identities
# ---------------------------------------------------------------------------
th, um, up = sp.symbols("theta u_m u_p", real=True)
f_face = 1 / (th / (1 + um) + (1 - th) / (1 + up))
ubar = th * um + (1 - th) * up
uhat = th * up + (1 - th) * um
rem = -th * (1 - th) * (um - up) ** 2 / (1 + uhat)
check("S0.1 harmonic face: f - 1 - ubar = -theta(1-theta)(u_- - u_+)^2/(1+uhat) exactly",
      sp.simplify(f_face - 1 - ubar - rem) == 0)

t = sp.symbols("t", real=True)
a, b = sp.symbols("a b", positive=True)
x, y = sp.Function("x")(t), sp.Function("y")(t)
lnG = -sp.log(a * sp.exp(-x) + b * sp.exp(-y))
thet = a * sp.exp(-x) / (a * sp.exp(-x) + b * sp.exp(-y))
d1 = sp.diff(lnG, t) - (thet * x.diff(t) + (1 - thet) * y.diff(t))
d2 = sp.diff(lnG, t, 2) - (thet * x.diff(t, 2) + (1 - thet) * y.diff(t, 2)
                           - thet * (1 - thet) * (x.diff(t) - y.diff(t)) ** 2)
check("S0.2 d/dt ln G_f = theta x' + (1-theta) y'", sp.simplify(d1) == 0)
check("S0.3 d2/dt2 ln G_f = theta x'' + (1-theta) y'' - theta(1-theta)(x'-y')^2", sp.simplify(d2) == 0)

X, beta, eps = sp.symbols("X beta epsilon", positive=True)
gX = sp.exp(-X / beta) - sp.exp(-X)
Xs = sp.log(beta) / (1 - 1 / beta)
gstar = beta ** (-1 / (beta - 1)) * (1 - 1 / beta)
mp.mp.dps = 40
ok04 = True
for bv in (mp.mpf("1.001"), mp.mpf("1.1"), mp.mpf(2), mp.mpf(5)):
    gnum = lambda xx: mp.exp(-xx / bv) - mp.exp(-xx)
    xs_num = mp.findroot(lambda xx: mp.diff(gnum, xx), mp.log(bv) / (1 - 1 / bv))
    xs_formula = mp.log(bv) / (1 - 1 / bv)
    ok04 &= abs(xs_num - xs_formula) < mp.mpf("1e-25") and \
        abs(gnum(xs_formula) - bv ** (-1 / (bv - 1)) * (1 - 1 / bv)) < mp.mpf("1e-30") and \
        all(gnum(xs_formula) >= gnum(xs_formula * fct) for fct in (mp.mpf("0.5"), mp.mpf("0.9"), mp.mpf("1.1"), 2))
mp.mp.dps = 15
check("S0.4 left-rule sharpness: argmax X* = ln(beta)/(1-1/beta), max = beta^(-1/(beta-1)) (1 - 1/beta) (mpmath, 4 betas)",
      ok04)
beta_left = (sp.exp(eps) - 1) / eps                     # eps = L h
beta_mid = sp.sinh(eps / 2) / (eps / 2)
lead_left = sp.limit(gstar.subs(beta, beta_left) / eps, eps, 0)
lead_mid = sp.limit(gstar.subs(beta, beta_mid) / eps**2, eps, 0)
check("S0.5 sup_mu error / (Lh) -> 1/(2e) (left), / (Lh)^2 -> 1/(24e) (midpoint)",
      sp.simplify(lead_left - 1 / (2 * sp.E)) == 0 and sp.simplify(lead_mid - 1 / (24 * sp.E)) == 0,
      f"limits {sp.nsimplify(lead_left)}, {sp.nsimplify(lead_mid)}")
s_, T_, h_ = sp.symbols("s T h", positive=True)
i1 = sp.integrate(1 / (s_ * (T_ - s_)), (s_, h_, T_ - h_))
i2 = sp.integrate(1 / sp.sqrt(s_ * (T_ - s_)), (s_, 0, T_))
check("S0.6 int_h^{T-h} ds/(s(T-s)) = (2/T) ln((T-h)/h); int_0^T ds/sqrt(s(T-s)) = pi",
      abs(float((i1 - 2 / T_ * sp.log((T_ - h_) / h_)).subs({T_: 7.0, h_: 0.3}))) < 1e-12
      and sp.simplify(i2 - sp.pi) == 0)

# ---------------------------------------------------------------------------
# Problem set-up (same synthetic data as sec5_staircase.py)
# ---------------------------------------------------------------------------
RG = 6.02214076e23 * 1.380649e-23
RADII = np.cumsum([175.0, 103.5, 39.4, 35.3, 41.0]) * 1e-6
T_REF = 1873.15
D_REF = np.array([2e-12, 1e-10, 5e-13, 1e-13, 5e-13])            # synthetic
Q_LAYER = np.array([209e3, 50e3, 222e3, 514e3, 222e3])          # synthetic, layer specific
Q_SHARED = np.full(5, 222e3)                                     # synthetic, shared scaling
T0, T1, TR = 1873.15, 2073.15, 4 * 3600.0                        # 1600 -> 1800 C in 4 h
TDOT = (T1 - T0) / TR


def T_ramp(t):
    return T0 + TDOT * t


def D_of_T(T, Q):
    return D_REF * np.exp(-Q / RG * (1.0 / T - 1.0 / T_REF))


def L_of(Q):
    """L = max_i sup_t |d/dt ln D_i(T(t))| on the ramp (attained at T0)."""
    return Q.max() * TDOT / (RG * T0**2)


def mesh(nc):
    edges, mat, inner = [0.0], [], 0.0
    for i, ro in enumerate(RADII):
        w = (ro - inner) / nc
        e = [inner + (k + 1) * w for k in range(nc)]
        e[-1] = ro
        edges += e
        mat += [i] * nc
        inner = ro
    e = np.array(edges)
    lo, hi = e[:-1], e[1:]
    V = 4 * np.pi / 3 * (hi**3 - lo**3)
    xc = 0.75 * (hi**4 - lo**4) / (hi**3 - lo**3)
    face = hi[:-1]
    return dict(V=V, sV=np.sqrt(V), Af=4 * np.pi * face**2, dl=face - xc[:-1], dr=xc[1:] - face,
                mat=np.array(mat), Gout=4 * np.pi * RADII[-1]**2 / (RADII[-1] - xc[-1]), N=len(V))


def faces(m, D):
    Dc = D[m["mat"]]
    return m["Af"] / (m["dl"] / Dc[:-1] + m["dr"] / Dc[1:]), m["Gout"] * Dc[-1]


def hat_tri(m, D):
    """Symmetric tridiagonal V^{-1/2} K(D) V^{-1/2}: (diagonal, off-diagonal)."""
    G, go = faces(m, D)
    d = np.zeros(m["N"])
    d[:-1] += G
    d[1:] += G
    d[-1] += go
    return d / m["V"], -G / (m["sV"][:-1] * m["sV"][1:])


def dense(tri):
    d, o = tri
    return np.diag(d) + np.diag(o, 1) + np.diag(o, -1)


def prop(m, z, D, dt):
    w, P = eigh_tridiagonal(*hat_tri(m, D))
    return P @ (np.exp(-w * dt) * (P.T @ z))


XG, WG = np.polynomial.legendre.leggauss(8)


def rule_D(rule, t0, h, Q, Tfun=T_ramp):
    if rule == "left":
        return D_of_T(Tfun(t0), Q)
    if rule == "mid":
        return D_of_T(Tfun(t0 + h / 2), Q)
    return 0.5 * sum(wq * D_of_T(Tfun(t0 + h / 2 * (1 + xq)), Q) for xq, wq in zip(XG, WG))


def staircase(m, z0, K, rule, Q, Tend=TR, Tfun=T_ramp, Dfun=None):
    h = Tend / K
    z = z0.copy()
    for k in range(K):
        Dk = Dfun(rule, k * h, h) if Dfun else rule_D(rule, k * h, h, Q, Tfun)
        z = prop(m, z, Dk, h)
    return z


def eta_ramp(K, rule, Q):
    """max_k sup_{s in I_k} sqrt(eps eps') for the ramp (D_i monotone, extremes at the ends)."""
    h = TR / K
    worst = 0.0
    for k in range(K):
        Dk = rule_D(rule, k * h, h, Q)
        r = np.concatenate([D_of_T(T_ramp(k * h), Q) / Dk, D_of_T(T_ramp((k + 1) * h), Q) / Dk])
        e1, e2 = np.max(np.abs(r - 1)), np.max(np.abs(1 / r - 1))
        worst = max(worst, math.sqrt(e1 * e2))
    return worst


def reference(m, z0, Q, Tend=TR, rtol=1e-13, dense_out=False):
    f = lambda t, z: -dense(hat_tri(m, D_of_T(T_ramp(t), Q))) @ z
    jac = lambda t, z: -dense(hat_tri(m, D_of_T(T_ramp(t), Q)))
    return solve_ivp(f, (0, Tend), z0, method="Radau", rtol=rtol, atol=1e-17, jac=jac,
                     dense_output=dense_out)


def init_indicator(m):
    z0 = m["sV"] * np.where(m["mat"] == 0, 1.0, 0.0)
    return z0 / np.linalg.norm(z0)


def normA(m, Q):
    return max(eigh_tridiagonal(*hat_tri(m, D_of_T(T, Q)), eigvals_only=True).max() for T in (T0, T1))


# ---------------------------------------------------------------------------
# S1 face lemma
# ---------------------------------------------------------------------------
rng = np.random.default_rng(7)
m20 = mesh(20)
ok_g, ok_cs = True, True
for _ in range(20):
    Dp = D_REF * np.exp(rng.uniform(-3, 3, 5))
    r = np.exp(rng.uniform(-0.7, 0.7, 5))
    D = Dp * r
    Kp, Kd = dense(hat_tri(m20, Dp)), dense(hat_tri(m20, D))
    lam = eigh(Kd - Kp, Kp, eigvals_only=True)
    ok_g &= lam.min() >= r.min() - 1 - 1e-9 and lam.max() <= r.max() - 1 + 1e-9
    e1, e2 = np.max(np.abs(r - 1)), np.max(np.abs(1 / r - 1))
    for _ in range(5):
        xv, yv = rng.standard_normal(m20["N"]), rng.standard_normal(m20["N"])
        lhs = abs(xv @ (Kd - Kp) @ yv)
        rhs = math.sqrt(e1 * e2) * math.sqrt(xv @ Kp @ xv) * math.sqrt(yv @ Kd @ yv)
        ok_cs &= lhs <= rhs * (1 + 1e-12)
check("S1.1 face lemma: spec(K(D)-K(D'), K(D')) in [min r - 1, max r - 1] (20 random pairs)", ok_g)
check("S1.2 mixed bound |x^T (K(D)-K(D')) y| <= sqrt(eps eps') |x|_{K(D')} |y|_{K(D)} (100 cases)", ok_cs)

# ---------------------------------------------------------------------------
# S2 duality representation of the error
# ---------------------------------------------------------------------------
m4 = mesh(4)
# smooth initial datum (slowest mode at T0) so that z(s) is smooth on the quadrature scale
z0s = eigh_tridiagonal(*hat_tri(m4, D_of_T(T0, Q_LAYER)))[1][:, 0].copy()
Ks2, rule2 = 8, "left"
h2 = TR / Ks2
xq, wq = np.polynomial.legendre.leggauss(30)
nodes = np.array([k * h2 + h2 / 2 * (1 + xx) for k in range(Ks2) for xx in xq])
f2 = lambda t, z: -dense(hat_tri(m4, D_of_T(T_ramp(t), Q_LAYER))) @ z
j2 = lambda t, z: -dense(hat_tri(m4, D_of_T(T_ramp(t), Q_LAYER)))
sol = solve_ivp(f2, (0, TR), z0s, method="Radau", rtol=1e-13, atol=1e-17, jac=j2,
                t_eval=np.append(nodes, TR))
zt = staircase(m4, z0s, Ks2, rule2, Q_LAYER)
phi = z0s + 0.3 * eigh_tridiagonal(*hat_tri(m4, D_of_T(T1, Q_LAYER)))[1][:, 1]   # smooth test vector
lhs = phi @ (sol.y[:, -1] - zt)
integral, wv = 0.0, phi.copy()
for k in reversed(range(Ks2)):
    Dk = rule_D(rule2, k * h2, h2, Q_LAYER)
    Kt = dense(hat_tri(m4, Dk))
    for iq, (xx, ww) in enumerate(zip(xq, wq)):
        s = k * h2 + h2 / 2 * (1 + xx)
        wloc = prop(m4, wv, Dk, (k + 1) * h2 - s)           # w(s) = exp(-(t_{k+1}-s) K~_k) w(t_{k+1})
        Delta = dense(hat_tri(m4, D_of_T(T_ramp(s), Q_LAYER))) - Kt
        integral += h2 / 2 * ww * (wloc @ Delta @ sol.y[:, k * len(xq) + iq])
    wv = prop(m4, wv, Dk, h2)
check("S2 duality: phi^T e(T) = -int_0^T w^T Delta z ds (relative difference < 1e-8)",
      abs(lhs + integral) <= 1e-8 * abs(lhs), f"lhs {lhs:.10e}, -integral {-integral:.10e}")

# ---------------------------------------------------------------------------
# S3 Theorem A and S8 observed orders, layer-specific Q (stiff)
# ---------------------------------------------------------------------------
L_lay = L_of(Q_LAYER)
print(f"INFO  L (layer-specific Q) = {L_lay:.4e} 1/s; L*T = {L_lay * TR:.3f}")
runs = {20: [64, 128, 256, 512, 1024, 2048, 4096], 40: [256, 512, 1024, 2048, 4096]}
okA, ratios = True, []
window_orders = {"left": [], "mid": [], "avg": []}
F_orders = {"left": [], "mid": [], "avg": []}
for nc, Ks in runs.items():
    m = mesh(nc)
    z0 = init_indicator(m)
    nA = normA(m, Q_LAYER)
    sol = reference(m, z0, Q_LAYER)
    zr = sol.y[:, -1]
    if nc == 20:
        zr2 = reference(m, z0, Q_LAYER, rtol=1e-11).y[:, -1]
        check("S8.0 reference: Radau rtol 1e-13 vs 1e-11 (V-norm difference)",
              np.linalg.norm(zr - zr2) < 1e-10, f"{np.linalg.norm(zr - zr2):.1e}")
    loss = math.sqrt(1 - zr @ zr)
    C0 = z0 / m["sV"]
    M0 = m["V"] @ C0
    Fr = 1 - m["V"] @ (zr / m["sV"]) / M0
    hA = [TR / K * nA for K in Ks]
    print(f"INFO  {nc} cells/layer: ||A||_V = {nA:.3e} 1/s; h||A|| = " + ", ".join(f"{v:.0f}" for v in hA))
    for rule in ("left", "mid", "avg"):
        errs, ferrs = [], []
        for K in Ks:
            zt = staircase(m, z0, K, rule, Q_LAYER)
            err = np.linalg.norm(zt - zr)
            bound = 0.5 * eta_ramp(K, rule, Q_LAYER) * loss
            okA &= err <= bound
            ratios.append(err / bound)
            errs.append(err)
            ferrs.append(abs(1 - m["V"] @ (zt / m["sV"]) / M0 - Fr))
        errs, ferrs = np.array(errs), np.array(ferrs)
        p = np.log2(errs[:-1] / errs[1:])
        pF = np.log2(ferrs[:-1] / ferrs[1:])
        print(f"INFO  {nc:2d} {rule:4s} ||e||_V: " + ", ".join(f"{e:.2e}" for e in errs))
        print(f"        orders V-norm: " + ", ".join(f"{v:.3f}" for v in p))
        print(f"        orders F(4 h): " + ", ".join(f"{v:.3f}" for v in pF))
        for i in range(len(Ks) - 1):
            if 100 <= hA[i + 1] and hA[i] <= 1200:
                window_orders[rule].append(p[i])
                F_orders[rule].append(pF[i])
check("S3 Theorem A: ||e(T)||_V <= (1/2) eta sqrt(||z0||^2-||z(T)||^2), all rules, both meshes, all K",
      okA, f"max err/bound {max(ratios):.3f}, min {min(ratios):.3f}")
for rule in ("left", "mid", "avg"):
    print(f"INFO  window orders h||A|| in [1e2, 1.2e3], {rule}: V-norm "
          + ", ".join(f"{v:.3f}" for v in window_orders[rule]) + " | F(4 h) "
          + ", ".join(f"{v:.3f}" for v in F_orders[rule]))
check("S8.1 left rule: V-norm order 1 in the stiff window (|p-1| < 0.02)",
      all(abs(v - 1) < 0.02 for v in window_orders["left"]))
check("S8.2 midpoint and average: V-norm orders in the stiff window lie in [1.70, 1.85], below 2",
      all(1.70 <= v <= 1.85 for r_ in ("mid", "avg") for v in window_orders[r_]))
check("S8.3 midpoint and average: release-fraction F(4 h) orders in the window lie in [1.7, 2.3]",
      all(1.7 <= v <= 2.3 for r_ in ("mid", "avg") for v in F_orders[r_]))

# reference-free orders on a finer mesh (80 cells per layer), midpoint rule
m80 = mesh(80)
nA80 = normA(m80, Q_LAYER)
z80 = init_indicator(m80)
K80 = [512, 1024, 2048, 4096]
zs80 = [staircase(m80, z80, K, "mid", Q_LAYER) for K in K80]
d80 = np.array([np.linalg.norm(zs80[i] - zs80[i + 1]) for i in range(len(K80) - 1)])
p80 = np.log2(d80[:-1] / d80[1:])
print(f"INFO  80 cells/layer, midpoint, h||A|| = " + ", ".join(f"{TR / K * nA80:.0f}" for K in K80)
      + "; Cauchy differences " + ", ".join(f"{v:.2e}" for v in d80)
      + "; orders " + ", ".join(f"{v:.3f}" for v in p80))
check("S8.4 80 cells/layer midpoint Cauchy orders lie in [1.70, 1.80] (trend to 7/4, CLAIMED)",
      all(1.70 <= v <= 1.80 for v in p80))

# ---------------------------------------------------------------------------
# S4 Theorem C: commuting case (shared scaling)
# ---------------------------------------------------------------------------
m = m20
z0 = init_indicator(m)
alpha = lambda t: math.exp(-Q_SHARED[0] / RG * (1 / T_ramp(t) - 1 / T_REF))
Dstar = D_REF
tau = float(mp.quad(lambda tt: alpha(float(tt)), [0, TR]))
wK, PK = eigh_tridiagonal(*hat_tri(m, Dstar))
z_exact = PK @ (np.exp(-wK * tau) * (PK.T @ z0))
LsT = Q_SHARED[0] * TDOT / RG
# alpha''/alpha = (ln alpha)'' + ((ln alpha)')^2, (ln alpha)' = Q Tdot/(R T^2), (ln alpha)'' = -2 Q Tdot^2/(R T^3)
sup_ratio = max(abs(-2 * LsT * TDOT / T**3 + (LsT / T**2) ** 2) * alpha((T - T0) / TDOT)
                for T in np.linspace(T0, T1, 2001)) / alpha(0.0)
okC, errs_mid, err_avg_C = True, [], 0.0
for K in [64, 128, 256, 512, 1024, 2048]:
    h = TR / K
    zt = staircase(m, z0, K, "mid", Q_SHARED)
    tau_t = h * sum(alpha((k + 0.5) * h) for k in range(K))
    err = np.linalg.norm(zt - z_exact)
    b1 = abs(tau - tau_t) / (math.e * min(tau, tau_t))
    b2 = h**2 * sup_ratio / (24 * math.e)
    okC &= err <= b1 * (1 + 1e-9) and b1 <= b2
    errs_mid.append(err)
    za = staircase(m, z0, K, "avg", Q_SHARED)
    err_avg_C = max(err_avg_C, np.linalg.norm(za - z_exact))
okC &= err_avg_C < 1e-11                                       # exact up to round-off
pC = np.log2(np.array(errs_mid[:-1]) / np.array(errs_mid[1:]))
check("S4.1 Theorem C: midpoint error <= |tau-tau~|/(e min tau) <= h^2 sup|alpha''|/(24 e inf alpha); average exact",
      okC, f"midpoint errors " + ", ".join(f"{v:.2e}" for v in errs_mid)
      + f"; average max error {err_avg_C:.1e} (round-off)")
check("S4.2 shared scaling, midpoint V-norm order 2 (|p-2| < 0.02) at h||A|| up to 4.5e3",
      all(abs(v - 2) < 0.02 for v in pC), "orders " + ", ".join(f"{v:.3f}" for v in pC))

# ---------------------------------------------------------------------------
# S5 Theorem B: temperature jumps
# ---------------------------------------------------------------------------
DA, DB = D_of_T(T0, Q_LAYER), D_of_T(T1, Q_LAYER)
rho = float(np.max(np.maximum(DB / DA, DA / DB)))


def jump_D_factory(tstar):
    def Dfun(rule, t0, h):
        t1 = t0 + h
        if rule == "left":
            return DA if t0 < tstar else DB
        if rule == "mid":
            return DA if t0 + h / 2 < tstar else DB
        frac = min(max((tstar - t0) / h, 0.0), 1.0)
        return frac * DA + (1 - frac) * DB
    return Dfun


def jump_exact(z0, tstar):
    return prop(m, prop(m, z0, DA, tstar), DB, TR - tstar)


okB_al = True
ts_al = 0.375 * TR
for K in [8, 64, 512]:
    for rule in ("left", "mid", "avg"):
        zt = staircase(m, z0, K, rule, None, Dfun=jump_D_factory(ts_al))
        okB_al &= np.linalg.norm(zt - jump_exact(z0, ts_al)) < 1e-13
check("S5.1 jump on the grid: staircase exact for every rule (< 1e-13)", okB_al)

okB, rB = True, []
ts = 0.3 * TR
for K in [16, 64, 256, 1024, 4096]:
    h = TR / K
    ks = int(ts // h)
    for rule in ("left", "mid", "avg"):
        Dk = jump_D_factory(ts)(rule, ks * h, h)
        rr = np.concatenate([DA / Dk, DB / Dk])
        ee = math.sqrt(np.max(np.abs(rr - 1)) * np.max(np.abs(1 / rr - 1)))
        bound = ee * rho * h / (2 * math.sqrt(ks * h * (TR - (ks + 1) * h)))
        err = np.linalg.norm(staircase(m, z0, K, rule, None, Dfun=jump_D_factory(ts)) - jump_exact(z0, ts))
        okB &= err <= bound
        rB.append(err / bound)
check("S5.2 misaligned jump: ||e(T)||_V <= sqrt(eps eps') rho h / (2 sqrt(t_k* (T - t_k*+1)))",
      okB, f"rho = {rho:.3f}; err/bound in [{min(rB):.2e}, {max(rB):.2e}]")

# ---------------------------------------------------------------------------
# S6 smoothing lemma along the reference solution
# ---------------------------------------------------------------------------
sol = reference(m, z0, Q_LAYER, dense_out=True)
okS = True
for tt in np.linspace(TR / 400, TR, 60):
    zz = sol.sol(tt)
    E = zz @ dense(hat_tri(m, D_of_T(T_ramp(tt), Q_LAYER))) @ zz
    okS &= E <= math.exp(L_lay * tt) / (2 * tt)
check("S6 smoothing: ||z(t)||^2_{K(t)} <= e^{Lt} ||z0||^2 / (2t) at 60 times", okS)

# ---------------------------------------------------------------------------
# S7 operator-relative constant against mesh refinement
# ---------------------------------------------------------------------------
def lambda_t(nc, Q):
    mm = mesh(nc)
    D = D_of_T(T0, Q)
    Dd = D * Q * TDOT / (RG * T0**2)
    Dc, Ddc = D[mm["mat"]], Dd[mm["mat"]]
    G = mm["Af"] / (mm["dl"] / Dc[:-1] + mm["dr"] / Dc[1:])
    Gd = G**2 / mm["Af"] * (mm["dl"] * Ddc[:-1] / Dc[:-1]**2 + mm["dr"] * Ddc[1:] / Dc[1:]**2)

    def asm(g, go):
        d = np.zeros(mm["N"])
        d[:-1] += g
        d[1:] += g
        d[-1] += go
        return dense((d / mm["V"], -g / (mm["sV"][:-1] * mm["sV"][1:])))
    Kh = asm(G, mm["Gout"] * Dc[-1])
    Kd = asm(Gd, mm["Gout"] * Ddc[-1])
    return np.linalg.norm(np.linalg.solve(Kh.T, Kd.T).T, 2) / L_of(Q)


ncs = [10, 20, 40, 80, 160]
lam_lay = np.array([lambda_t(n, Q_LAYER) for n in ncs])
lam_sh = np.array([lambda_t(n, Q_SHARED) for n in ncs])
rat = lam_lay[1:] / lam_lay[:-1]
print("INFO  Lambda_t (layer-specific Q), cells/layer 10..160: " + ", ".join(f"{v:.3f}" for v in lam_lay)
      + "; successive ratios " + ", ".join(f"{v:.3f}" for v in rat))
check("S7.1 layer-specific Q: Lambda_t grows like N^{1/2} (last ratio within 0.01 of sqrt 2)",
      abs(rat[-1] - math.sqrt(2)) < 0.01)
check("S7.2 shared scaling: Lambda_t = 1 on every mesh", np.all(np.abs(lam_sh - 1) < 1e-9))

print(f"\nSUMMARY {sum(RESULTS)}/{len(RESULTS)} checks passed")
if all(RESULTS):
    print("ALL PASS")
