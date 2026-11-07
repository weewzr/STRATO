"""Section 5 (Phase 2.2) numerical demonstration 1: shared-alpha temperature ramp.

An independent Python finite-volume (FV) solver, built with the same discretisation as
STRATO v1.0 Code B (interface-aligned cells, volume centroids, series face conductance,
half-cell absorbing outer conductance), is used in two ways. (1a) evaluates the semi-discrete
FV solution at the effective time tau(t) of (P22.26) through its eigen-decomposition (no real-time
stepping; this tests spatial convergence to an independent multilayer Sturm-Liouville series,
shooting on u = r c). (1b) and (1c) run in REAL time with D_i(T(t)) recomputed every step
(backward Euler) or continuously (Radau), and are compared with the tau-space solution.

Checks:
 (1a) semi-discrete FV (exact in time) converges to the series at tau(t): FV(tau) -> series.
 (1b) backward Euler with D(T^{n+1}) and with the step-averaged D are both first order in dt.
 (1c) decay factorisation (P22.27), (P22.33), (P22.35): a stiff real-time ODE integration with
      alpha(t) and decay agrees with e^{-lam t} x (tau-space solution).

All parameters are SYNTHETIC test values (labelled below); they are not material data.
Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec5_fv_shared_alpha.py
The script writes no files.
"""
import math

import numpy as np
from scipy.integrate import solve_ivp
from scipy.linalg import eigh, solve_banded
from scipy.optimize import brentq
from scipy.special import expn

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


RG = 6.02214076e23 * 1.380649e-23          # J/(mol K), exact SI value

# ---------------------------------------------------------------------------
# Geometry: AGR-1 Baseline nominal dimensions (Morris et al., 2016, Table 2):
# kernel diameter 350 um, buffer 103.5, IPyC 39.4, SiC 35.3, OPyC 41.0 um.
# ---------------------------------------------------------------------------
RADII = np.cumsum([175.0, 103.5, 39.4, 35.3, 41.0]) * 1e-6        # m
# SYNTHETIC diffusivities at T_ref and ONE shared activation energy (shared alpha case)
T_REF = 1873.15
D_REF = np.array([2e-12, 1e-10, 5e-13, 1e-13, 5e-13])             # m^2/s, synthetic
Q_SHARED = 300e3                                                  # J/mol, synthetic
C0 = 1.0                                                          # mol/m^3 in the kernel


def T_of_t(t):
    """Synthetic schedule: 1250 C at t = 0, 50 C/h to 1600 C (7 h), then hold."""
    return np.minimum(1523.15 + 50.0 / 3600.0 * t, 1873.15)


def alpha_of_t(t):
    return np.exp(-Q_SHARED / RG * (1.0 / T_of_t(t) - 1.0 / T_REF))


def tau_of_t(t):
    """Exact effective time (P22.22)-(P22.26): E2 closed form on the ramp, linear on the hold."""
    a = Q_SHARED / RG
    t_ramp = 7 * 3600.0
    beta = 50.0 / 3600.0
    prim = lambda T: T * expn(2, a / T)
    if t <= t_ramp:
        return math.exp(a / T_REF) * (prim(T_of_t(t)) - prim(1523.15)) / beta
    return tau_of_t(t_ramp) + (t - t_ramp) * 1.0


# ---------------------------------------------------------------------------
# Independent FV (same scheme as v1.0 Code B)
# ---------------------------------------------------------------------------
class FV:
    def __init__(self, radii, ncell):
        edges = [0.0]
        mat = []
        inner = 0.0
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
        self.face = b[:-1]
        self.Af = 4 * np.pi * self.face**2
        self.dl = self.face - self.xc[:-1]
        self.dr = self.xc[1:] - self.face
        self.R = radii[-1]
        self.AR = 4 * np.pi * self.R**2
        self.dout = self.R - self.xc[-1]
        self.n = len(self.V)

    def conductances(self, D_layer):
        Dc = D_layer[self.mat]
        G = self.Af / (self.dl / Dc[:-1] + self.dr / Dc[1:])          # (P22.14)
        Gout = self.AR * Dc[-1] / self.dout
        return G, Gout

    def stiffness(self, D_layer):
        G, Gout = self.conductances(D_layer)
        diag = np.zeros(self.n)
        diag[:-1] += G
        diag[1:] += G
        diag[-1] += Gout
        A = np.diag(diag) - np.diag(G, 1) - np.diag(G, -1)
        return A, G, Gout

    def c_init(self):
        return np.where(self.mat == 0, C0, 0.0)

    def be_step(self, C, dt, D_layer, lam=0.0):
        """(V/dt + A + lam V) C^{n+1} = (V/dt) C^n  (P22.15)."""
        G, Gout = self.conductances(D_layer)
        diag = self.V / dt + lam * self.V
        diag[:-1] += G
        diag[1:] += G
        diag[-1] += Gout
        ab = np.zeros((3, self.n))
        ab[0, 1:] = -G
        ab[1] = diag
        ab[2, :-1] = -G
        Cn = solve_banded((1, 1), ab, self.V / dt * C)
        return Cn, Gout


# ---------------------------------------------------------------------------
# Independent Sturm-Liouville series (u = r c, transfer through layers)
# ---------------------------------------------------------------------------
def shoot(lam, radii, D):
    left, a, b = 0.0, 0.0, 1.0
    segs = []
    for i, Ro in enumerate(radii):
        k = math.sqrt(lam / D[i])
        L = Ro - left
        segs.append((left, Ro, k, a, b))
        u = a * math.cos(k * L) + b * math.sin(k * L)
        up = k * (-a * math.sin(k * L) + b * math.cos(k * L))
        if i + 1 < len(radii):
            upn = u / Ro + D[i] / D[i + 1] * (up - u / Ro)   # D (u' - u/r) continuous
            kn = math.sqrt(lam / D[i + 1])
            a, b = u, upn / kn
        left = Ro
    return u, segs


def eigen_series(radii, D, lam_max, scan_per_root=40):
    theta = sum((Ro - Ri) / math.sqrt(d) for Ro, Ri, d in zip(radii, np.r_[0.0, radii[:-1]], D))
    zmax = math.sqrt(lam_max)
    nz = int(zmax * theta / math.pi * scan_per_root) + 200
    z = np.linspace(1e-9, zmax, nz)
    f = np.array([shoot(zz**2, radii, D)[0] for zz in z])
    roots = []
    for i in np.nonzero(np.sign(f[:-1]) * np.sign(f[1:]) < 0)[0]:
        zr = brentq(lambda zz: shoot(zz**2, radii, D)[0], z[i], z[i + 1], xtol=1e-15, rtol=1e-15)
        roots.append(zr**2)
    return np.array(roots)


def gl_integral(f, a, b, npts):
    x, w = np.polynomial.legendre.leggauss(npts)
    xm, xh = 0.5 * (a + b), 0.5 * (b - a)
    return xh * np.sum(w * f(xm + xh * x))


def mode_data(lam, radii, D):
    _, segs = shoot(lam, radii, D)
    N = I1 = In = 0.0
    for idx, (l, rr, k, a, b) in enumerate(segs):
        npts = 64 + 4 * int(k * (rr - l))
        u = lambda x, l=l, k=k, a=a, b=b: a * np.cos(k * (x - l)) + b * np.sin(k * (x - l))
        N += gl_integral(lambda x: u(x) ** 2, l, rr, npts)
        m = gl_integral(lambda x: x * u(x), l, rr, npts)
        In += m
        if idx == 0:
            I1 = m
    return N, I1, In


def series_weights(radii, D, lams):
    R1 = radii[0]
    w = []
    for lam in lams:
        N, I1, In = mode_data(lam, radii, D)
        w.append(3 * I1 * In / (R1**3 * N))
    return np.array(w)


# ---------------------------------------------------------------------------
# Build the tau-space (Phase 1) series at T_ref
# ---------------------------------------------------------------------------
t_eval = np.array([4.0, 7.0, 10.0, 20.0]) * 3600.0
tau_eval = np.array([tau_of_t(t) for t in t_eval])
print("INFO  t (h)   : " + "  ".join(f"{t / 3600:10.3f}" for t in t_eval))
print("INFO  tau (h) : " + "  ".join(f"{x / 3600:10.5f}" for x in tau_eval))
lam_max = 45.0 / tau_eval.min()
lams = eigen_series(RADII, D_REF, lam_max)
w = series_weights(RADII, D_REF, lams)
print(f"INFO  series: {len(lams)} modes up to Lambda_max = {lam_max:.3e} 1/s, sum w_n = {w.sum():.6f}")
F_series = 1.0 - np.array([np.sum(w * np.exp(-lams * x)) for x in tau_eval])
print("INFO  F_series(tau(t)): " + "  ".join(f"{f:.8f}" for f in F_series))

# Cross-check: no eigenvalue missed (compare with fine-mesh FV generalized eigenvalues)
fv_fine = FV(RADII, 160)
A_ref, _, _ = fv_fine.stiffness(D_REF)
ev = eigh(A_ref, np.diag(fv_fine.V), eigvals_only=True, subset_by_value=(0, lam_max))
nmatch = min(len(ev), len(lams))
relgap = np.max(np.abs(ev[:nmatch] / lams[:nmatch] - 1))
check("eigenvalue count: series roots = FV eigenvalues below Lambda_max", len(ev) == len(lams),
      f"{len(lams)} vs {len(ev)}, max rel diff {relgap:.1e}")

# Check of the tau integral itself against quadrature
from scipy.integrate import quad
tq = quad(alpha_of_t, 0, 7 * 3600.0, epsabs=0, epsrel=1e-13, limit=200)[0]
check("tau(7 h) closed form = quadrature of alpha", abs(tq / tau_of_t(7 * 3600.0) - 1) < 1e-11)

# ---------------------------------------------------------------------------
# (1a) semi-discrete FV in tau (exact in time) vs series, mesh refinement
# ---------------------------------------------------------------------------
F_sd = {}
for nc in (20, 40, 80, 160):
    fv = FV(RADII, nc)
    A, _, _ = fv.stiffness(D_REF)
    wv, Phi = eigh(A, np.diag(fv.V))                 # Phi^T V Phi = I
    C0v = fv.c_init()
    M0 = fv.V @ C0v
    coef = Phi.T @ (fv.V * C0v)
    F_sd[nc] = np.array([1.0 - (fv.V @ (Phi @ (np.exp(-wv * x) * coef))) / M0 for x in tau_eval])
errs = {nc: np.max(np.abs(F_sd[nc] - F_series)) for nc in F_sd}
for nc in errs:
    print(f"INFO  cells/layer {nc:4d}: max |F_FV(tau) - F_series(tau)| = {errs[nc]:.3e}")
p_obs = math.log2(errs[80] / errs[160])
check("FV(tau) converges to the series at tau(t)", errs[160] < 1e-5 and errs[160] < errs[20],
      f"error {errs[160]:.2e} at 160 cells/layer, observed order {p_obs:.2f}")

# ---------------------------------------------------------------------------
# (1b) backward Euler in REAL time with recomputed conductances (P22.15)-(P22.18)
# ---------------------------------------------------------------------------
fv = FV(RADII, 40)
C0v = fv.c_init()
M0 = fv.V @ C0v
xg, wg = np.polynomial.legendre.leggauss(8)


def run_be(dt, mode):
    nsteps = int(round(t_eval[-1] / dt))
    C = C0v.copy()
    out = []
    released = 0.0
    k_eval = set(int(round(te / dt)) for te in t_eval)
    for n in range(1, nsteps + 1):
        t0, t1 = (n - 1) * dt, n * dt
        if mode == "end":
            al = alpha_of_t(t1)
        else:   # step average of alpha (exact Delta tau / dt to quadrature precision)
            al = 0.5 * np.sum(wg * alpha_of_t(0.5 * (t0 + t1) + 0.5 * dt * xg))
        C, Gout = fv.be_step(C, dt, al * D_REF)
        released += dt * Gout * C[-1]                     # same G_out as in the matrix
        if n in k_eval:
            out.append((released, fv.V @ C))
    rel = np.array([o[0] for o in out]) / M0
    bal = max(abs(o[0] + o[1] - M0) / M0 for o in out)
    return rel, bal


A40, _, _ = fv.stiffness(D_REF)
wv, Phi = eigh(A40, np.diag(fv.V))
coef = Phi.T @ (fv.V * C0v)
F_exact40 = np.array([1.0 - (fv.V @ (Phi @ (np.exp(-wv * x) * coef))) / M0 for x in tau_eval])
for mode in ("end", "avg"):
    e = []
    for dt in (720.0, 360.0, 180.0, 90.0):
        F, bal = run_be(dt, mode)
        e.append(np.max(np.abs(F - F_exact40)))
    orders = [math.log2(e[i] / e[i + 1]) for i in range(len(e) - 1)]
    label = "D(T^{n+1})" if mode == "end" else "step-averaged D"
    print(f"INFO  BE {label:16s}: errors " + ", ".join(f"{x:.3e}" for x in e)
          + " | orders " + ", ".join(f"{o:.3f}" for o in orders) + f" | balance residual {bal:.1e}")
    check(f"BE with {label}: observed order in [0.9, 1.1]", 0.9 < orders[-1] < 1.1)
    # Roundoff level only: this script uses LAPACK banded LU and plain summation, whereas
    # v1.0 uses subtraction-free Thomas elimination and Kahan summation (residual ~1e-13).
    check(f"BE with {label}: discrete balance M + M_rel = M0 to 1e-10 (roundoff)", bal < 1e-10,
          f"{bal:.1e}")
    if mode == "end":
        e_end = e
    else:
        e_avg = e
print(f"INFO  error ratio end/avg at dt = 90 s: {e_end[-1] / e_avg[-1]:.3f}")

# ---------------------------------------------------------------------------
# (1c) decay factorisation: real-time stiff ODE vs e^{-lam t} x tau-space solution
# ---------------------------------------------------------------------------
LAM = math.log(2) / (10 * 3600.0)        # synthetic half-life 10 h
G40, Gout40 = fv.conductances(D_REF)


def rhs(t, y):
    C = y[:-1]
    al = alpha_of_t(t)
    flux = np.zeros_like(C)
    q = al * G40 * (C[:-1] - C[1:])
    flux[:-1] -= q
    flux[1:] += q
    flux[-1] -= al * Gout40 * C[-1]
    dC = flux / fv.V - LAM * C
    dMrel = al * Gout40 * C[-1]                         # J_R(t), released atoms (no decay outside)
    return np.r_[dC, dMrel]


Ainv = -np.diag(1 / fv.V) @ A40
def jac(t, y):
    J = np.zeros((fv.n + 1, fv.n + 1))
    J[:-1, :-1] = alpha_of_t(t) * Ainv - LAM * np.eye(fv.n)
    J[-1, -2] = alpha_of_t(t) * Gout40
    return J


sol = solve_ivp(rhs, (0, t_eval[-1]), np.r_[C0v, 0.0], method="Radau", t_eval=t_eval,
                rtol=1e-11, atol=1e-16, jac=jac)
M_ode = fv.V @ sol.y[:-1]
Mrel_ode = sol.y[-1]
# tau-space (Phase 1) inventory and release, then the transform (P22.33), (P22.35)
Mhat = lambda x: fv.V @ (Phi @ (np.exp(-wv * x) * coef))
M_tr = np.exp(-LAM * t_eval) * np.array([Mhat(x) for x in tau_eval])
check("inventory: M(t) = e^{-lam t} Mhat(tau(t)) (P22.33)", np.max(np.abs(M_ode - M_tr) / M0) < 1e-8,
      f"max rel diff {np.max(np.abs(M_ode - M_tr) / M0):.1e}")
# (P22.35): M_rel = e^{-lam t} Mhat_rel(tau) + lam int_0^t e^{-lam s} Mhat_rel(tau(s)) ds
Mrel_tr = []
for te, xe in zip(t_eval, tau_eval):
    edges = np.unique(np.r_[np.linspace(0, te, 81), min(te, 7 * 3600.0)])
    integ = 0.0
    for a_, b_ in zip(edges[:-1], edges[1:]):
        ss = 0.5 * (a_ + b_) + 0.5 * (b_ - a_) * xg
        vals = [math.exp(-LAM * x) * (M0 - Mhat(tau_of_t(x))) for x in ss]
        integ += 0.5 * (b_ - a_) * np.dot(wg, vals)
    Mrel_tr.append(math.exp(-LAM * te) * (M0 - Mhat(xe)) + LAM * integ)
Mrel_tr = np.array(Mrel_tr)
check("release: (P22.35) equals the integrated surface flux", np.max(np.abs(Mrel_ode - Mrel_tr) / M0) < 1e-8,
      f"max rel diff {np.max(np.abs(Mrel_ode - Mrel_tr) / M0):.1e}")
Mdec = M0 - M_ode - Mrel_ode
print("INFO  with decay, fractions at t = 4,7,10,20 h: inside " + ", ".join(f"{x:.5f}" for x in M_ode / M0)
      + " | released " + ", ".join(f"{x:.5f}" for x in Mrel_ode / M0)
      + " | decayed inside " + ", ".join(f"{x:.5f}" for x in Mdec / M0))
# series version of the inventory (spatially exact) at the same tau
M_series = np.exp(-LAM * t_eval) * (1.0 - F_series)
check("inventory vs series: |M/M0 - e^{-lam t}(1 - F_series(tau))| < 1e-4 at 40 cells/layer",
      np.max(np.abs(M_ode / M0 - M_series)) < 1e-4, f"{np.max(np.abs(M_ode / M0 - M_series)):.1e}")

print(f"\nSUMMARY {sum(RESULTS)}/{len(RESULTS)} checks passed")
