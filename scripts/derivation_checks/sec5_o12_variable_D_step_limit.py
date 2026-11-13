"""Open item O12: Backward Euler step-size accuracy criterion with time-dependent diffusivity.

Checks the derivation of the variable-D generalisation of the diffusion step limit (P21.78)
(placeholder equations (P2X.O12.k) of the O12 derivation section). The Backward Euler scheme is
(P22.15) with conductances recomputed every step, option A (step-averaged D_i) or option E
(D_i at the end of the step). A Python transcription of the v1.0 finite-volume mesh
(interface-aligned cells, volume centroids, series face conductance, half-cell absorbing outer
conductance; same construction as sec5_fv_shared_alpha.py) is used throughout.

What is checked:
 S1  bounds 0 <= z^2/2 - z^3/3 <= g(z) = z - ln(1+z) <= z^2/2 for z >= 0 (symbolic derivatives).
 S2  constant coefficients: the variable-D criterion reduces to (P21.78) and its step rule (F3).
 S3  equidistribution: sum Z_k^2 >= Theta^2 / N (fewest steps at equal Z_k).
 S4  smoothing constant c_n of the equal-step absolute error bound, n >= 3.
 N0  homogeneity and monotonicity of Lambda_1 in the diffusivities; |d ln Lambda_1/dt| bound.
 N1  shared scaling, option A, lambda = 0: exact modal identity on variable steps; the step rule
     Z_k <= zeta = 2 eps / Theta controls the relative error of 1 - F; comparison with the
     constant-step warning of step F3 (Lambda_1 at the highest and at the lowest temperature);
     absolute-error bound for F over the whole history.
 N2  shared scaling, option E: exact identity with the coefficient-lag term, its sign on heating
     and cooling, and the rigorous lag bounds (variation of Lambda_1 and of ln Lambda_1).
 N3  layer-specific activation energies (no shared scaling): the frozen-coefficient prediction of
     the relative error of 1 - F against a Radau reference, options A and E, with and without the
     coefficient-resolution cap (P22.20); the remainder is first order in eps_D.
 N4  decay lambda > 0, shared scaling, option A: exact modal form of M and F, the split of the
     release-fraction error into amplitude, decay-amplitude and rectangle parts with their bounds,
     and first-order scaling of the release-fraction error in zeta.

All diffusivities, activation energies, the temperature history and the half-life are SYNTHETIC
test values; they are not material data. Geometry: AGR-1 Baseline nominal dimensions as used in
sec5_fv_shared_alpha.py.
Run (from the repository root):
    .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec5_o12_variable_D_step_limit.py
The script writes no files. It ends with ALL PASS only if every check passes.
"""
import math

import numpy as np
import sympy as sp
from scipy.integrate import quad, solve_ivp
from scipy.linalg import eigh, eigh_tridiagonal, solve_banded
from scipy.special import expn

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


RG = 6.02214076e23 * 1.380649e-23          # J/(mol K), exact SI value (P22.5)

# ---------------------------------------------------------------------------
# S1 to S4: scalar and symbolic statements
# ---------------------------------------------------------------------------
z = sp.symbols("z", nonnegative=True)
g = z - sp.log(1 + z)
h_up = sp.Rational(1, 2) * z**2 - g                       # >= 0 ?
h_lo = g - sp.Rational(1, 2) * z**2 + sp.Rational(1, 3) * z**3
ok1 = sp.simplify(sp.diff(h_up, z) - z**2 / (1 + z)) == 0 and h_up.subs(z, 0) == 0
ok2 = sp.simplify(sp.diff(h_lo, z) - z**3 / (1 + z)) == 0 and h_lo.subs(z, 0) == 0
zz = np.logspace(-3, 3, 2001)                 # z - log1p(z) loses digits below 1e-3 in floats
gz = zz - np.log1p(zz)
ok3 = np.all(gz <= 0.5 * zz**2 * (1 + 1e-9)) and np.all(gz >= (0.5 * zz**2 - zz**3 / 3) - 1e-9 * zz**2)
check("S1 g(z)=z-ln(1+z): (z^2/2-g)'=z^2/(1+z), (g-z^2/2+z^3/3)'=z^3/(1+z), both 0 at z=0",
      ok1 and ok2 and ok3)

Lam, lam, h, t, eps = sp.symbols("Lambda lambda h t epsilon", positive=True)
n = t / h
delta_const = n * sp.Rational(1, 2) * ((Lam + lam) * h) ** 2      # (1/2) sum Z_k^2, equal steps
theta_const = (Lam + lam) * t
zeta = 2 * eps / theta_const
h_rule = zeta / (Lam + lam)
ok = (sp.simplify(delta_const - sp.Rational(1, 2) * (Lam + lam) ** 2 * t * h) == 0
      and sp.simplify(h_rule - 2 * eps / ((Lam + lam) ** 2 * t)) == 0)
check("S2 constant D: (1/2) sum Z_k^2 = (1/2)(Lam+lam)^2 t dt (P21.78); rule Z<=2eps/Theta gives F3 dt", ok)

rng = np.random.default_rng(12)
ok = True
for _ in range(200):
    Zs = rng.random(rng.integers(2, 60)) ** 3
    ok &= np.sum(Zs**2) >= np.sum(Zs) ** 2 / len(Zs) * (1 - 1e-14)
check("S3 sum Z_k^2 >= Theta^2/N (Cauchy-Schwarz), equality at equal Z_k", ok)

# c_n: (1+x/n)^{-n} - e^{-x} <= (1+x/n)^{-n} x^2/(2n) <= c_n / n, c_n = (1/2) x*^2 (1+x*/n)^{-n}
nn = np.arange(3, 10**6 + 1, dtype=float)
xs = 2 * nn / (nn - 2)
cn = 0.5 * xs**2 * np.exp(-nn * np.log1p(xs / nn))
ok_mono = np.all(np.diff(cn) < 0)
ok_val = abs(cn[0] - 2.0 / 3.0) < 1e-15 and abs(cn[-1] - 2 * math.exp(-2)) < 1e-5
ok_grid = True
xg_ = np.logspace(-6, 4, 4001)
for nint in (3, 4, 5, 10, 30, 100, 1000):
    En = np.exp(-nint * np.log1p(xg_ / nint)) - np.exp(-xg_)
    ok_grid &= np.max(En) * nint <= cn[nint - 3] * (1 + 1e-12)
check("S4 equal steps: n[(1+x/n)^-n - e^-x] <= c_n, c_3 = 2/3, c_n decreasing to 2e^-2 (n<=1e6)",
      ok_mono and ok_val and ok_grid, f"c_3={cn[0]:.6f}, c_1e6={cn[-1]:.6f}, 2e^-2={2*math.exp(-2):.6f}")
NN = sp.symbols("N", positive=True, integer=True)
zc = (Lam + lam) * h
F_be = Lam * h * (1 - (1 + zc) ** (-NN)) / zc                # sum_{n=1}^N Lam h (1+z)^-n
F_ex_c = Lam / (Lam + lam) * (1 - sp.exp(-(Lam + lam) * NN * h))
err_c = sp.simplify(F_be - F_ex_c + Lam / (Lam + lam) * ((1 + zc) ** (-NN) - sp.exp(-(Lam + lam) * NN * h)))
geo = sp.simplify(sp.summation((1 + zc) ** (-sp.Symbol("k", integer=True, positive=True)),
                               (sp.Symbol("k", integer=True, positive=True), 1, 5)) - (1 - (1 + zc) ** -5) / zc)
check("S5 constant D with decay: F_h - F = -p (a_h - a), p = Lam/(Lam+lam) (geometric series, N = 5 spot check)",
      err_c == 0 and geo == 0)
print(f"INFO  CRP-6 1b reduction (constant T): (1/2) Lambda_1 (Lambda_1 t) dt = "
      f"{0.5*1.9745e-5*14.2*360:.4f} at 360 s, {0.5*1.9745e-5*14.2*36:.5f} at 36 s (Section 4.6 values)")

# ---------------------------------------------------------------------------
# Geometry and finite-volume transcription
# ---------------------------------------------------------------------------
RADII = np.cumsum([175.0, 103.5, 39.4, 35.3, 41.0]) * 1e-6        # m
T_REF = 1873.15
D_REF = np.array([2e-12, 1e-10, 5e-13, 1e-13, 5e-13])             # m^2/s, synthetic
Q_SHARED = 300e3                                                  # J/mol, synthetic
Q_LAYER = np.array([209e3, 50e3, 222e3, 514e3, 222e3])           # J/mol, synthetic, layer specific
C0 = 1.0


class FV:
    def __init__(self, radii, ncell):
        edges, mat, inner = [0.0], [], 0.0
        for i, Ro in enumerate(radii):
            e = np.linspace(inner, Ro, ncell + 1)[1:]
            e[-1] = Ro
            edges.extend(e)
            mat += [i] * ncell
            inner = Ro
        e = np.array(edges)
        a, b = e[:-1], e[1:]
        self.V = 4 * np.pi / 3 * (b**3 - a**3)
        self.xc = 0.75 * (b**4 - a**4) / (b**3 - a**3)
        self.mat = np.array(mat)
        f = b[:-1]
        self.Af = 4 * np.pi * f**2
        self.dl = f - self.xc[:-1]
        self.dr = self.xc[1:] - f
        R = radii[-1]
        self.gout = 4 * np.pi * R**2 / (R - self.xc[-1])
        self.n = len(self.V)
        self.sV = np.sqrt(self.V)

    def cond(self, D):
        Dc = D[self.mat]
        return self.Af / (self.dl / Dc[:-1] + self.dr / Dc[1:]), self.gout * Dc[-1]   # (P22.14)

    def diag_off(self, D):
        G, Go = self.cond(D)
        d = np.zeros(self.n)
        d[:-1] += G
        d[1:] += G
        d[-1] += Go
        return d, G, Go

    def lam1(self, D):
        d, G, _ = self.diag_off(D)
        return eigh_tridiagonal(d / self.V, -G / np.sqrt(self.V[:-1] * self.V[1:]),
                                eigvals_only=True, select="i", select_range=(0, 0))[0]

    def modes(self, D):
        d, G, _ = self.diag_off(D)
        A = np.diag(d) - np.diag(G, 1) - np.diag(G, -1)
        return eigh(A, np.diag(self.V))                         # Phi^T V Phi = I

    def be_step(self, C, dt, D, lam=0.0):
        """(V/dt + A + lam V) C^{n+1} = (V/dt) C^n, (P22.15); returns C^{n+1} and G_out."""
        d, G, Go = self.diag_off(D)
        ab = np.zeros((3, self.n))
        ab[0, 1:] = -G
        ab[1] = d + self.V / dt + lam * self.V
        ab[2, :-1] = -G
        return solve_banded((1, 1), ab, self.V / dt * C), Go


fv = FV(RADII, 20)
Cin = np.where(fv.mat == 0, C0, 0.0)
M0 = fv.V @ Cin
XG, WG = np.polynomial.legendre.leggauss(8)
XG4, WG4 = np.polynomial.legendre.leggauss(4)

# ---------------------------------------------------------------------------
# Temperature histories (synthetic): linear in T between knots (P22.21)
# ---------------------------------------------------------------------------
H = 3600.0
KN_UP = [(0.0, 1523.15), (7 * H, 1873.15), (100 * H, 1873.15)]                 # ramp 50 K/h, hold
KN_FULL = KN_UP + [(100 * H + 350 / 600 * H, 1523.15), (110 * H, 1523.15)]     # + cool -600 K/h, hold


def T_at(kn, t):
    for (ta, Ta), (tb, Tb) in zip(kn[:-1], kn[1:]):
        if t <= tb:
            return Ta + (Tb - Ta) * (t - ta) / (tb - ta)
    return kn[-1][1]


def D_shared(T):
    return D_REF * math.exp(-Q_SHARED / RG * (1 / T - 1 / T_REF))


def D_layer(T):
    return D_REF * np.exp(-Q_LAYER / RG * (1 / T - 1 / T_REF))


def alpha_T(T):
    return math.exp(-Q_SHARED / RG * (1 / T - 1 / T_REF))


def tau_exact(kn, t):
    """Effective time from (P22.22): E2 closed form on ramps, linear on holds."""
    th = Q_SHARED / RG
    prim = lambda T: T * expn(2, th / T)
    tau = 0.0
    for (ta, Ta), (tb, Tb) in zip(kn[:-1], kn[1:]):
        te = min(t, tb)
        if te <= ta:
            break
        Te = Ta + (Tb - Ta) * (te - ta) / (tb - ta)
        if Tb == Ta:
            tau += (te - ta) * alpha_T(Ta)
        else:
            tau += math.exp(th / T_REF) * (prim(Te) - prim(Ta)) * (tb - ta) / (Tb - Ta)
    return tau


def step_avg(fun, kn, t0, t1):
    s = 0.5 * (t0 + t1) + 0.5 * (t1 - t0) * XG
    vals = np.array([fun(T_at(kn, x)) for x in s])
    return 0.5 * np.tensordot(WG, vals, axes=(0, 0))


def make_steps(kn, lam1_t, lam, zeta, dt_uniform=None, eps_D=None, Qmax=None):
    """Rule (P2X.O12) on each linear segment: Z_k = int (Lambda_1+lam) <= zeta, monotone segments.
    With eps_D, also the coefficient-resolution cap (P22.20) evaluated at t_old (step F5.3).
    With dt_uniform, constant steps that land on every knot instead."""
    ts = [0.0]
    for (ta, Ta), (tb, Tb) in zip(kn[:-1], kn[1:]):
        t0 = ta
        slope = (Tb - Ta) / (tb - ta)
        while t0 < tb * (1 - 1e-15):
            if dt_uniform is not None:
                t1 = min(t0 + dt_uniform, tb)
            else:
                l0 = lam1_t(t0) + lam
                ttry = min(t0 + zeta / l0, tb)
                dt1 = zeta / (max(l0, lam1_t(ttry) + lam))
                if eps_D is not None and slope != 0:
                    T0 = T_at(kn, t0)
                    dt1 = min(dt1, eps_D * RG * T0**2 / (Qmax * abs(slope)))
                t1 = min(t0 + dt1, tb)
            if tb - t1 < 1e-9 * (tb - ta):
                t1 = tb
            ts.append(t1)
            t0 = t1
    return np.array(ts)


def run_be(kn, ts, Dfun, option, lam=0.0):
    C = Cin.copy()
    rel = 0.0
    Fh = [0.0]
    for t0, t1 in zip(ts[:-1], ts[1:]):
        D = step_avg(Dfun, kn, t0, t1) if option == "A" else Dfun(T_at(kn, t1))
        C, Go = fv.be_step(C, t1 - t0, D, lam)
        rel += (t1 - t0) * Go * C[-1]                   # same G_out as in the matrix (P22.16)
        Fh.append(rel / M0)
    return C, np.array(Fh)


# ---------------------------------------------------------------------------
# N0 homogeneity, monotonicity and the bound |d ln Lambda_1/dt| <= max_i |d ln D_i/dt|
# ---------------------------------------------------------------------------
Dt = D_layer(1700.0)
l1 = fv.lam1(Dt)
# eigh_tridiagonal resolves Lambda_1 to about 1e-16 Lambda_max / Lambda_1 ~ 1e-10 relative here
hom = abs(fv.lam1(3.7 * Dt) / (3.7 * l1) - 1)
ok_m = True
for _ in range(50):
    fac = 1 + rng.random(5)
    ok_m &= fv.lam1(Dt * fac) >= l1 * (1 - 1e-8)
    ok_m &= min(fac) * l1 * (1 - 1e-8) <= fv.lam1(Dt * fac) <= max(fac) * l1 * (1 + 1e-8)
check("N0a Lambda_1(sD) = s Lambda_1(D); Lambda_1 non-decreasing in each D_i; ratio within [min,max]",
      hom < 1e-8 and ok_m, f"homogeneity residual {hom:.1e} (eigensolver accuracy)")
ok = True
for T in np.linspace(1530, 1870, 18):
    dT = 0.01
    dl = (math.log(fv.lam1(D_layer(T + dT))) - math.log(fv.lam1(D_layer(T - dT)))) / (2 * dT)
    ok &= dl <= np.max(Q_LAYER / (RG * T**2)) * (1 + 1e-6) and dl >= np.min(Q_LAYER / (RG * T**2)) * (1 - 1e-6)
check("N0b d ln Lambda_1/dT lies in [min_i, max_i] Q_i/(Rg T^2) (layer-specific Q)", ok)

# ---------------------------------------------------------------------------
# N1 shared scaling, option A, lambda = 0
# ---------------------------------------------------------------------------
wv, Phi = fv.modes(D_REF)                         # Lambda_m^* at T_ref
cm = Phi.T @ (fv.V * Cin)
wm = (fv.V @ Phi) * cm / M0                       # 1 - F = sum w_m a_m
W = np.sum(np.abs(wm))
check("N1a sum w_m = 1 and sum |w_m| <= sqrt(V_tot/V_K) (Cauchy-Schwarz)",
      abs(np.sum(wm) - 1) < 1e-12 and W <= math.sqrt(fv.V.sum() / fv.V[fv.mat == 0].sum()),
      f"sum|w_m| = {W:.4f}, bound {math.sqrt(fv.V.sum()/fv.V[fv.mat==0].sum()):.4f}, w_1 = {wm[0]:.4f}")
lam1_ref = wv[0]
kn = KN_UP
t_end = kn[-1][0]
Theta = lam1_ref * tau_exact(kn, t_end)
lam1_t = lambda x: lam1_ref * alpha_T(T_at(kn, x))
print(f"INFO  Lambda_1(1600 C) = {lam1_ref:.5e} 1/s, Lambda_1(1250 C) = {lam1_t(0):.4e} 1/s, "
      f"Theta(t_end) = {Theta:.4f}, Lambda_1,max t_end = {lam1_ref*t_end:.4f}")
oneF_exact = np.sum(wm * np.exp(-wv * tau_exact(kn, t_end)))


def modal_check(ts, eps_name):
    dtau = np.array([step_avg(alpha_T, kn, a, b) * (b - a) for a, b in zip(ts[:-1], ts[1:])])
    Zm = np.outer(wv, dtau)                                   # Z_{m,k}
    ah = np.exp(-np.sum(np.log1p(Zm), axis=1))
    return dtau, Zm, ah


for eps_ in (1e-2, 1e-3):
    zeta_ = 2 * eps_ / Theta
    ts = make_steps(kn, lam1_t, 0.0, zeta_)
    C, Fh = run_be(kn, ts, D_shared, "A")
    dtau, Zm, ah = modal_check(ts, eps_)
    oneF_h = fv.V @ C / M0
    ok_id = abs(oneF_h - np.sum(wm * ah)) / oneF_h < 1e-9
    ok_tau = abs(dtau.sum() / tau_exact(kn, t_end) - 1) < 1e-12
    Z1 = Zm[0]
    delta1 = np.sum(Z1 - np.log1p(Z1))
    obs = oneF_h / oneF_exact - 1
    check(f"N1b eps={eps_:g}: nodal BE 1-F = sum_m w_m prod(1+Lambda_m dtau_k)^-1; sum dtau = tau (P22.22)",
          ok_id and ok_tau, f"rel {abs(oneF_h - np.sum(wm*ah))/oneF_h:.1e}, steps {len(ts)-1}")
    check(f"N1c eps={eps_:g}: rule Z_k <= zeta holds, delta_1 <= zeta Theta/2 = eps",
          np.max(Z1) <= zeta_ * (1 + 1e-9) and delta1 <= 0.5 * np.sum(Z1**2) <= eps_ * (1 + 1e-9),
          f"max Z/zeta {np.max(Z1)/zeta_:.6f}, delta_1 {delta1:.4e}, (1/2)sum Z^2 {0.5*np.sum(Z1**2):.4e}")
    check(f"N1d eps={eps_:g}: relative error of 1-F at t_end = e^delta_1 - 1 (late time) and <= e^eps - 1",
          abs(obs / math.expm1(delta1) - 1) < 1e-3 and 0 < obs <= math.expm1(eps_),
          f"observed {obs:.5e}, predicted {math.expm1(delta1):.5e}, steps {len(ts)-1}, "
          f"Cauchy-Schwarz minimum Theta^2/sum Z^2 = {Theta**2/np.sum(Z1**2):.1f}")
    # absolute error of F at every step: identity and rigorous bound
    ts_all = ts
    F_ex_all = np.array([1 - np.sum(wm * np.exp(-wv * tau_exact(kn, x))) for x in ts_all[1:]])
    logah = -np.cumsum(np.log1p(Zm), axis=1)
    dm = np.cumsum(Zm - np.log1p(Zm), axis=1)
    ident = -np.sum(wm[:, None] * np.exp(logah) * (-np.expm1(-dm)), axis=0)
    bound = np.sum(np.abs(wm)[:, None] * np.exp(logah) * np.minimum(1.0, 0.5 * np.cumsum(Zm**2, axis=1)), axis=0)
    err = Fh[1:] - F_ex_all
    ok_ident = np.max(np.abs(err - ident)) < 1e-9
    ok_bound = np.all(np.abs(err) <= bound + 1e-12)
    k_late = np.searchsorted(ts_all[1:], 20 * H)
    check(f"N1e eps={eps_:g}: F_h - F = -sum w_m a_m^h (1 - e^-delta_m) at all steps; |err| <= bound",
          ok_ident and ok_bound,
          f"max|err| {np.max(np.abs(err)):.2e} (first step {abs(err[0]):.2e}), "
          f"max|err| after 20 h {np.max(np.abs(err[k_late:])):.2e}")
    if eps_ == 1e-3:
        n_rule = len(ts) - 1
        # equal-step absolute bound c_n/n * W in tau: compare at the end of the run
        print(f"INFO  eps=1e-3: n={n_rule}, W c_n / n = {W * 0.5*(2*n_rule/(n_rule-2))**2*(1-2/n_rule)**n_rule / n_rule:.2e}"
              f" (equal-Z bound, holds only for equal Z_k), observed |err| at t_end {abs(err[-1]):.2e}")

# Comparison with the constant-step warning of step F3
eps_ = 1e-3
for lab, L in (("Lambda_1 at the highest T", lam1_ref), ("Lambda_1 at the lowest T", lam1_t(0.0))):
    dt_u = 2 * eps_ / (L**2 * t_end)
    if dt_u > t_end:
        dt_u = t_end
    ts = make_steps(kn, lam1_t, 0.0, None, dt_uniform=dt_u)
    C, _ = run_be(kn, ts, D_shared, "A")
    obs = (fv.V @ C / M0) / oneF_exact - 1
    if lab.endswith("highest T"):
        check("N1f F3 warning value (Lambda_1 at T_max) is a sufficient cap: error <= e^eps - 1",
              0 < obs <= math.expm1(eps_), f"observed {obs:.4e}, steps {len(ts)-1} vs rule {n_rule}")
    else:
        check("N1g Lambda_1 at T_min (not a bound on Lambda_1(t)) gives an error above eps",
              obs > eps_, f"observed {obs:.4e}, steps {len(ts)-1}")

# ---------------------------------------------------------------------------
# N2 shared scaling, option E: delta_E = sum g(z_k) + sum (Z_k - z_k)
# ---------------------------------------------------------------------------
for kname, knE in (("ramp+hold", KN_UP), ("ramp+hold+cool+hold", KN_FULL)):
    kn = knE
    t_endE = kn[-1][0]
    lam1_tE = lambda x: lam1_ref * alpha_T(T_at(kn, x))
    ThetaE = lam1_ref * tau_exact(kn, t_endE)
    ts = make_steps(kn, lam1_tE, 0.0, 2 * 1e-2 / ThetaE)
    C, _ = run_be(kn, ts, D_shared, "E")
    dtau = np.array([step_avg(alpha_T, kn, a, b) * (b - a) for a, b in zip(ts[:-1], ts[1:])])
    zE = np.array([lam1_tE(b) * (b - a) for a, b in zip(ts[:-1], ts[1:])])
    ZE = lam1_ref * dtau
    lag = np.sum(ZE - zE)
    dE = np.sum(zE - np.log1p(zE)) + lag
    oneF_ex = np.sum(wm * np.exp(-wv * tau_exact(kn, t_endE)))
    zEm = np.outer(wv / lam1_ref, zE)
    oneF_mod = np.sum(wm * np.exp(-np.sum(np.log1p(zEm), axis=1)))
    oneF_h = fv.V @ C / M0
    obs = oneF_h / oneF_ex - 1
    dts = np.diff(ts)
    dL = np.abs(np.array([lam1_tE(b) - lam1_tE(a) for a, b in zip(ts[:-1], ts[1:])]))
    TV = np.sum(dL)
    ok_id = abs(oneF_h / oneF_mod - 1) < 1e-9
    ok_lag = abs(lag) <= np.sum(dts * dL) <= np.max(dts[dL > 0]) * TV * (1 + 1e-12)
    check(f"N2a option E, {kname}: modal identity; |lag| <= sum dt_k |dLambda_1,k| <= max dt TV(Lambda_1)",
          ok_id and ok_lag, f"lag {lag:.3e}, sum dt|dL| {np.sum(dts*dL):.3e}, (1/2) sum dt|dL| {0.5*np.sum(dts*dL):.3e}")
    check(f"N2b option E, {kname}: relative error of 1-F = e^delta_E - 1", abs(obs / math.expm1(dE) - 1) < 1e-3,
          f"observed {obs:.5e}, predicted {math.expm1(dE):.5e}, sum g(z) {np.sum(zE-np.log1p(zE)):.4e}")
    zetaE = 2 * 1e-2 / ThetaE
    TVln = np.sum(np.abs(np.diff(np.log([lam1_tE(x) for x in ts]))))
    TVQ = sum(Q_SHARED / RG * abs(1 / Tb - 1 / Ta) for (_, Ta), (_, Tb) in zip(kn[:-1], kn[1:]))
    check(f"N2d option E, {kname}: |lag| <= zeta TV(ln Lambda_1) <= zeta (Q/Rg) TV(1/T); max z_k <= zeta",
          abs(lag) <= zetaE * TVln * (1 + 1e-12) and TVln <= TVQ * (1 + 1e-9) and np.max(zE) <= zetaE * (1 + 1e-9),
          f"|lag| {abs(lag):.3e}, zeta TV(ln L1) {zetaE*TVln:.3e} (leading order half: {0.5*zetaE*TVln:.3e}), "
          f"TV(ln L1) {TVln:.4f}, (Q/Rg)TV(1/T) {TVQ:.4f}")
    if kname == "ramp+hold":
        check("N2c heating: the lag term is negative (option E partly cancels the BE lag)", lag < 0)
    else:
        print(f"INFO  with cool-down the lag term is {lag:.3e} (heating part cancels the cooling part)")

# ---------------------------------------------------------------------------
# N3 layer-specific Q: frozen-coefficient prediction against a Radau reference
# ---------------------------------------------------------------------------
kn = KN_UP
t_end = kn[-1][0]


def ode_ref(Dfun, rtol):
    y = Cin.copy()
    for (ta, _), (tb, _) in zip(kn[:-1], kn[1:]):
        def rhs(tt, yy):
            d, G, Go = fv.diag_off(Dfun(T_at(kn, tt)))
            Ay = d * yy
            Ay[:-1] -= G * yy[1:]
            Ay[1:] -= G * yy[:-1]
            return -Ay / fv.V

        def jac(tt, yy):
            d, G, Go = fv.diag_off(Dfun(T_at(kn, tt)))
            A = np.diag(d) - np.diag(G, 1) - np.diag(G, -1)
            return -A / fv.V[:, None]
        y = solve_ivp(rhs, (ta, tb), y, method="Radau", rtol=rtol, atol=1e-16, jac=jac).y[:, -1]
    return fv.V @ y / M0


ref_shared = ode_ref(D_shared, 1e-11)
check("N3a Radau reference reproduces the exact shared-scaling 1-F (validates the reference)",
      abs(ref_shared / oneF_exact - 1) < 1e-6, f"rel {abs(ref_shared/oneF_exact-1):.1e}")
ref_L = ode_ref(D_layer, 1e-11)
ref_L2 = ode_ref(D_layer, 1e-10)
lam1_tL = lambda x: fv.lam1(D_layer(T_at(kn, x)))
# Theta by composite Gauss-Legendre on each segment
ThetaL = 0.0
for (ta, _), (tb, _) in zip(kn[:-1], kn[1:]):
    ed = np.linspace(ta, tb, 41)
    for a, b in zip(ed[:-1], ed[1:]):
        ThetaL += 0.5 * (b - a) * sum(w * lam1_tL(0.5 * (a + b) + 0.5 * (b - a) * x) for x, w in zip(XG4, WG4))
print(f"INFO  layer-specific Q: Lambda_1(1250 C) = {lam1_tL(0):.4e}, Lambda_1(1600 C) = {lam1_tL(t_end):.5e}, "
      f"Theta = {ThetaL:.4f}, 1-F_ref = {ref_L:.6e} (rtol 1e-10 vs 1e-11: {abs(ref_L2/ref_L-1):.1e})")
N3 = {}
for eps_, epsD in ((1e-2, None), (1e-2, 0.1), (1e-2, 0.025), (3e-3, None), (3e-3, 0.1), (3e-3, 0.025)):
    ts = make_steps(kn, lam1_tL, 0.0, 2 * eps_ / ThetaL, eps_D=epsD, Qmax=np.max(Q_LAYER))
    Zk = np.array([0.5 * (b - a) * sum(w * lam1_tL(0.5 * (a + b) + 0.5 * (b - a) * x) for x, w in zip(XG4, WG4))
                   for a, b in zip(ts[:-1], ts[1:])])
    zEk = np.array([lam1_tL(b) * (b - a) for a, b in zip(ts[:-1], ts[1:])])
    zAk = np.array([fv.lam1(step_avg(D_layer, kn, a, b)) * (b - a) for a, b in zip(ts[:-1], ts[1:])])
    nramp = int(np.sum(ts[1:] <= 7 * H))
    for opt in ("A", "E"):
        C, _ = run_be(kn, ts, D_layer, opt)
        obs = (fv.V @ C / M0) / ref_L - 1
        zk = zAk if opt == "A" else zEk
        pred = np.sum(zk - np.log1p(zk)) + np.sum(Zk - zk)
        N3[(eps_, epsD, opt)] = (obs, math.expm1(pred))
        print(f"INFO  N3 option {opt}, eps={eps_:g}, eps_D={epsD}: observed {obs:.5e}, frozen-coefficient "
              f"prediction {math.expm1(pred):.5e}, ratio {obs/math.expm1(pred):.4f}, "
              f"remainder {obs-math.expm1(pred):+.3e}, steps {len(ts)-1} ({nramp} on the ramp)")
    if epsD is None:
        check(f"N3c eps={eps_:g}: option A step eigenvalue >= step mean of Lambda_1 (concavity), gap is tiny",
              np.all(zAk >= Zk * (1 - 1e-8)), f"max (z_A - Z)/Z = {np.max((zAk - Zk)/Zk):.2e}")
rem = {k: v[0] - v[1] for k, v in N3.items()}
ok_pos = all(x > 0 for x in rem.values())
ratios = [rem[(e, 0.1, o)] / rem[(e, 0.025, o)] for e in (1e-2, 3e-3) for o in ("A", "E")]
check("N3d remainder R_na = observed - frozen prediction is positive and first order in eps_D "
      "(ratio for eps_D 0.1 : 0.025 in [2.5, 5.5], first order gives 4)",
      ok_pos and all(2.5 < x < 5.5 for x in ratios), "ratios " + ", ".join(f"{x:.2f}" for x in ratios))
ind = [rem[(1e-2, 0.025, o)] / rem[(3e-3, 0.025, o)] for o in ("A", "E")]
check("N3e at eps_D = 0.025 R_na does not depend on the Z rule (eps 1e-2 vs 3e-3 within 10 %)",
      all(abs(x - 1) < 0.1 for x in ind), "ratios " + ", ".join(f"{x:.3f}" for x in ind))
rat = [N3[(e, 0.025, o)][0] / N3[(e, 0.025, o)][1] for e in (1e-2, 3e-3) for o in ("A", "E")]
check("N3f with eps_D = 0.025 the frozen-coefficient prediction is within 3.5 % (options A and E)",
      all(abs(x - 1) < 0.035 for x in rat), "ratios " + ", ".join(f"{x:.4f}" for x in rat))

# ---------------------------------------------------------------------------
# N4 decay, shared scaling, option A
# ---------------------------------------------------------------------------
kn = KN_UP
t_end = kn[-1][0]
LAMD = math.log(2) / (30 * H)                     # synthetic half-life 30 h
ThetaD = lam1_ref * tau_exact(kn, t_end) + LAMD * t_end
tau_e = tau_exact(kn, t_end)
a_ex = np.exp(-wv * tau_e - LAMD * t_end)         # exact mode amplitudes at t_end
# exact per-mode release fraction F_m = int Lambda_m alpha exp(-Lambda_m tau - lam t) dt (adaptive quadrature)
F_m = np.zeros(fv.n)
for m in range(fv.n):
    for (ta, _), (tb, _) in zip(kn[:-1], kn[1:]):
        f = lambda x: wv[m] * alpha_T(T_at(kn, x)) * math.exp(-wv[m] * tau_exact(kn, x) - LAMD * x)
        F_m[m] += quad(f, ta, tb, epsabs=0, epsrel=1e-12, limit=400)[0]
D_m = 1.0 - F_m - a_ex                             # exact per-mode balance F_m + D_m + a_m = 1
F_ex = np.sum(wm * F_m)
errs_F = {}
for eps_ in (1e-2, 1e-3):
    zeta_ = 2 * eps_ / ThetaD
    ts = make_steps(kn, lambda x: lam1_ref * alpha_T(T_at(kn, x)), LAMD, zeta_)
    C, Fh = run_be(kn, ts, D_shared, "A", lam=LAMD)
    dtau = np.array([step_avg(alpha_T, kn, a, b) * (b - a) for a, b in zip(ts[:-1], ts[1:])])
    dts = np.diff(ts)
    Zm = np.outer(wv, dtau) + LAMD * dts
    ahist = np.exp(-np.cumsum(np.log1p(Zm), axis=1))          # a_h^{n+1}, all modes and steps
    ah = ahist[:, -1]
    Mh = fv.V @ C / M0
    M_ex = np.sum(wm * a_ex)
    dm = np.sum(Zm - np.log1p(Zm), axis=1)
    d1 = dm[0]
    Fh_m = np.sum(np.outer(wv, dtau) * ahist, axis=1)          # sum_n Lambda_m dtau_n a_h^{n+1}
    Dh_m = LAMD * np.sum(dts * ahist, axis=1)
    check(f"N4a decay, eps={eps_:g}: nodal M and F equal the modal sums (exact modal form)",
          abs(Mh / np.sum(wm * ah) - 1) < 1e-9 and abs(Fh[-1] / np.sum(wm * Fh_m) - 1) < 1e-9,
          f"steps {len(ts)-1}")
    check(f"N4b decay, eps={eps_:g}: relative error of retained M at t_end = e^delta_1 - 1, <= e^eps - 1",
          abs((Mh / M_ex - 1) / math.expm1(d1) - 1) < 1e-3 and Mh / M_ex - 1 <= math.expm1(eps_),
          f"observed {Mh/M_ex-1:.5e}, predicted {math.expm1(d1):.5e}")
    errF = Fh[-1] - F_ex
    e_a = ah - a_ex
    e_D = Dh_m - D_m
    tauh = np.cumsum(dtau)
    aexh = np.exp(-np.outer(wv, tauh) - LAMD * ts[1:])          # exact a_m at the step ends
    aexh_prev = np.hstack([np.ones((fv.n, 1)), aexh[:, :-1]])
    amp_D = LAMD * np.sum(dts * (ahist - aexh), axis=1)       # >= 0, amplitude error inside the decay sum
    rect = e_D - amp_D                                        # right-endpoint rectangle error of lam int a
    dmh = np.cumsum(Zm - np.log1p(Zm), axis=1)
    up_amp = LAMD * np.sum(dts * ahist * np.minimum(1, dmh), axis=1)
    ok_split = abs(errF - np.sum(wm * (-e_a - e_D))) < 1e-10
    ok_ea = np.all(e_a >= -1e-15) and np.all(e_a <= ah * np.minimum(1, dm) + 1e-15)
    ok_amp = np.all(amp_D >= -1e-13) and np.all(amp_D <= up_amp + 1e-13)
    ok_rect = (np.all(rect <= 1e-12) and np.all(rect >= -LAMD * np.sum(dts * (aexh_prev - aexh), axis=1) - 1e-12)
               and np.all(rect >= -zeta_ * (1 - a_ex) - 1e-12))
    B = np.sum(np.abs(wm) * (ah * np.minimum(1, dm) + np.maximum(zeta_ * (1 - a_ex), up_amp)))
    check(f"N4c decay, eps={eps_:g}: F_h - F = -sum w_m (e_a + amp_D + rect); each part within its bound; |F_h - F| <= B",
          ok_split and ok_ea and ok_amp and ok_rect and abs(errF) <= B,
          f"F = {F_ex:.6f}, F_h - F = {errF:.4e}: amplitude {-np.sum(wm*e_a):.3e}, decay-amplitude "
          f"{-np.sum(wm*amp_D):.3e}, rectangle {-np.sum(wm*rect):.3e}; B = {B:.3e}, zeta W = {zeta_*W:.3e}")
    p1 = lam1_ref / (lam1_ref + LAMD)
    cc = -p1 * wm[0] * a_ex[0] * math.expm1(d1)
    print(f"INFO  eps={eps_:g}: constant-coefficient relation -p_1 w_1 a_1 (e^delta_1 - 1) = {cc:.3e} "
          f"against F_h - F = {errF:.3e} (ratio {errF/cc:.1f}); the constant-D relation does not carry over")
    errs_F[eps_] = errF
r = errs_F[1e-2] / errs_F[1e-3]
check("N4d decay: release-fraction error is first order in zeta (ratio for eps 1e-2 : 1e-3 near 10)",
      7 < r < 13, f"ratio {r:.3f}")

print("ALL PASS" if all(RESULTS) else f"FAILURES: {RESULTS.count(False)} of {len(RESULTS)}")
