"""Section 5 (Phase 2.2) numerical demonstration 2: piecewise-constant temperature with a
step change and layer-specific activation energies (no shared alpha).

Method under test: eigenfunction projection (P22.39)-(P22.46). On each constant-T interval
the multilayer series is exact; at the switch the field is re-expanded in the new
eigenfunctions with P_mn = int r^2 phi_m^new phi_n^old dr / N_m^new = int u_m^new u_n^old dr / N_m^new.
Reference: independent FV, semi-discrete and exact in time (eigendecomposition on each interval),
on 40/80/160 cells per layer with Richardson extrapolation.
Also checked: the Galerkin stiffness K (P22.49) is diagonal in its own eigenbasis and full in a
foreign one (mode coupling), and the truncation criterion (P22.44).

All diffusion parameters are SYNTHETIC test values, not material data.
Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec5_projection_step.py
The script writes no files.
"""
import math

import numpy as np
from scipy.linalg import eigh
from scipy.optimize import brentq

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


RG = 6.02214076e23 * 1.380649e-23
RADII = np.cumsum([175.0, 103.5, 39.4, 35.3, 41.0]) * 1e-6        # AGR-1 nominal (Morris et al., 2016)
T_REF = 1873.15
D_REF = np.array([2e-12, 1e-10, 5e-13, 1e-13, 5e-13])             # synthetic, m^2/s at T_REF
Q_LAYER = np.array([209e3, 50e3, 222e3, 514e3, 222e3])           # synthetic, J/mol, layer specific
C0 = 1.0
T1, T2 = 1873.15, 2073.15                                         # 1600 C then 1800 C (step)


def D_of_T(T):
    return D_REF * np.exp(-Q_LAYER / RG * (1.0 / T - 1.0 / T_REF))


# ----------------------------- FV (same scheme as v1.0) ------------------------------
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
        self.face = b[:-1]
        self.Af = 4 * np.pi * self.face**2
        self.dl, self.dr = self.face - self.xc[:-1], self.xc[1:] - self.face
        self.R = radii[-1]
        self.n = len(self.V)

    def stiffness(self, D_layer):
        Dc = D_layer[self.mat]
        G = self.Af / (self.dl / Dc[:-1] + self.dr / Dc[1:])
        Gout = 4 * np.pi * self.R**2 * Dc[-1] / (self.R - self.xc[-1])
        diag = np.zeros(self.n)
        diag[:-1] += G
        diag[1:] += G
        diag[-1] += Gout
        return np.diag(diag) - np.diag(G, 1) - np.diag(G, -1)

    def propagate(self, C, D_layer, dt):
        w, Phi = eigh(self.stiffness(D_layer), np.diag(self.V))
        return Phi @ (np.exp(-w * dt) * (Phi.T @ (self.V * C)))


def fv_release(nc, schedule, t_out):
    """schedule: list of (t_start, T); exact-in-time propagation of the semi-discrete system."""
    fv = FV(RADII, nc)
    C = np.where(fv.mat == 0, C0, 0.0)
    M0 = fv.V @ C
    out, t_now = [], 0.0
    bounds = [s[0] for s in schedule[1:]] + [np.inf]
    for te in t_out:
        while t_now < te:
            k = max(i for i, s in enumerate(schedule) if s[0] <= t_now + 1e-9)
            t_next = min(te, bounds[k])
            C = fv.propagate(C, D_of_T(schedule[k][1]), t_next - t_now)
            t_now = t_next
        out.append(1.0 - fv.V @ C / M0)
    return np.array(out)


# ----------------------------- series pieces -----------------------------------------
def shoot(lam, D):
    left, a, b, segs = 0.0, 0.0, 1.0, []
    for i, Ro in enumerate(RADII):
        k = math.sqrt(lam / D[i])
        L = Ro - left
        segs.append((left, Ro, k, a, b))
        u = a * math.cos(k * L) + b * math.sin(k * L)
        up = k * (-a * math.sin(k * L) + b * math.cos(k * L))
        if i + 1 < len(RADII):
            upn = u / Ro + D[i] / D[i + 1] * (up - u / Ro)
            a, b = u, upn / math.sqrt(lam / D[i + 1])
        left = Ro
    return u, segs


def eigenvalues(D, lam_max, per_root=40):
    theta = sum((Ro - Ri) / math.sqrt(d) for Ro, Ri, d in zip(RADII, np.r_[0.0, RADII[:-1]], D))
    zmax = math.sqrt(lam_max)
    z = np.linspace(1e-9, zmax, int(zmax * theta / math.pi * per_root) + 200)
    f = np.array([shoot(x**2, D)[0] for x in z])
    roots = [brentq(lambda x: shoot(x**2, D)[0], z[i], z[i + 1], xtol=1e-15, rtol=1e-15) ** 2
             for i in np.nonzero(np.sign(f[:-1]) * np.sign(f[1:]) < 0)[0]]
    return np.array(roots)


GL = {}


def gl(npts):
    if npts not in GL:
        GL[npts] = np.polynomial.legendre.leggauss(npts)
    return GL[npts]


def basis(D, lams):
    """Per-layer coefficients of u_n = a cos(k (r-l)) + b sin(k (r-l)) for each eigenvalue."""
    return [shoot(lam, D)[1] for lam in lams]


def u_eval(seg, x):
    l, rr, k, a, b = seg
    return a * np.cos(k * (x - l)) + b * np.sin(k * (x - l))


def du_eval(seg, x):
    l, rr, k, a, b = seg
    return k * (-a * np.sin(k * (x - l)) + b * np.cos(k * (x - l)))


def layer_nodes(i, kmax):
    l = 0.0 if i == 0 else RADII[i - 1]
    rr = RADII[i]
    npts = 80 + 6 * int(kmax * (rr - l))
    x, w = gl(npts)
    return 0.5 * (l + rr) + 0.5 * (rr - l) * x, 0.5 * (rr - l) * w


def gram(BA, BB, deriv=False, Dlayer=None):
    """Matrix G_mn = sum_layers int f_m^A f_n^B dr; f = u, or with deriv: D (u'-u/r) products."""
    G = np.zeros((len(BA), len(BB)))
    for i in range(len(RADII)):
        kmax = max(max(s[i][2] for s in BA), max(s[i][2] for s in BB))
        x, w = layer_nodes(i, kmax)
        if deriv:
            FA = np.array([du_eval(s[i], x) - u_eval(s[i], x) / x for s in BA])
            FB = np.array([du_eval(s[i], x) - u_eval(s[i], x) / x for s in BB])
            G += Dlayer[i] * (FA * w) @ FB.T
        else:
            FA = np.array([u_eval(s[i], x) for s in BA])
            FB = np.array([u_eval(s[i], x) for s in BB])
            G += (FA * w) @ FB.T
    return G


def moments(B):
    I1 = np.zeros(len(B))
    In = np.zeros(len(B))
    for i in range(len(RADII)):
        kmax = max(s[i][2] for s in B)
        x, w = layer_nodes(i, kmax)
        m = np.array([np.sum(w * x * u_eval(s[i], x)) for s in B])
        In += m
        if i == 0:
            I1 = m.copy()
    return I1, In


# ----------------------------- the experiment ----------------------------------------
M0 = C0 * 4 * np.pi * RADII[0] ** 3 / 3
for t_s_h in (1.0, 0.05):
    t_s = t_s_h * 3600.0
    t_out = t_s + np.array([0.0, 0.25, 1.0, 4.0]) * 3600.0
    print(f"\n=== step 1600 C -> 1800 C at t_s = {t_s_h} h; outputs at t_s + 0, 0.25, 1, 4 h ===")
    D1, D2 = D_of_T(T1), D_of_T(T2)
    lam_max_old = 45.0 / t_s
    lam_max_new = 45.0 / (0.25 * 3600.0)
    l_old = eigenvalues(D1, lam_max_old)
    l_new = eigenvalues(D2, max(lam_max_new, lam_max_old))
    B_old, B_new = basis(D1, l_old), basis(D2, l_new)
    N_old = np.diag(gram(B_old, B_old))
    N_new = np.diag(gram(B_new, B_new))
    I1_old, In_old = moments(B_old)
    _, In_new = moments(B_new)
    a0 = C0 * I1_old / N_old                      # a_n(0) = c0 int_0^R1 r u_n dr / N_n
    a_ts = a0 * np.exp(-l_old * t_s)
    P = gram(B_new, B_old) / N_new[:, None]        # (P22.44)
    print(f"INFO  modes: old {len(l_old)} (Lambda <= {lam_max_old:.2e} 1/s), new {len(l_new)}")

    # orthogonality and Galerkin diagonal checks (only once)
    if t_s_h == 1.0:
        Gnn = gram(B_new, B_new)
        offo = np.max(np.abs(Gnn - np.diag(N_new)) / np.sqrt(np.outer(N_new, N_new)))
        check("orthogonality of new eigenfunctions (weight r^2 for phi = weight 1 for u)", offo < 1e-10,
              f"max normalised off-diagonal {offo:.1e}")
        K_own = gram(B_new, B_new, deriv=True, Dlayer=D2) / N_new[:, None]
        diag_err = np.max(np.abs(np.diag(K_own) / l_new - 1))
        off_own = np.max(np.abs(K_own - np.diag(np.diag(K_own))) / l_new[:, None])
        check("K(T2) in its own basis = diag(Lambda) (P22.49)-(P22.50)", diag_err < 1e-9 and off_own < 1e-9,
              f"diag rel err {diag_err:.1e}, off-diag {off_own:.1e}")
        m = min(10, len(l_old))
        K_for = gram(B_old[:m], B_old[:m], deriv=True, Dlayer=D2) / N_old[:m, None]
        coup = np.max(np.abs(K_for - np.diag(np.diag(K_for))) / np.abs(np.diag(K_for))[:, None])
        print(f"INFO  K(T2) in the T1 basis, first 10 modes: max |K_mn|/K_mm (m != n) = {coup:.3f}")
        check("K(T2) in the T1 basis is not diagonal: modes couple when layer ratios change", coup > 1e-2)

    F_proj = {}
    for M in (5, 10, 20, 40, None):
        mo = len(l_old) if M is None else min(M, len(l_old))
        mn = len(l_new) if M is None else min(M, len(l_new))
        a_new = P[:mn, :mo] @ a_ts[:mo]
        F_before = 1.0 - 4 * np.pi * np.dot(a_ts[:mo], In_old[:mo]) / M0
        res = [F_before]
        for te in t_out[1:]:
            res.append(1.0 - 4 * np.pi * np.dot(a_new * np.exp(-l_new[:mn] * (te - t_s)), In_new[:mn]) / M0)
        F_proj[M] = np.array(res)
        F_after0 = 1.0 - 4 * np.pi * np.dot(a_new, In_new[:mn]) / M0
        F_proj[(M, "jump")] = F_after0 - F_before

    sched = [(0.0, T1), (t_s, T2)]
    F40, F80, F160 = (fv_release(nc, sched, t_out) for nc in (40, 80, 160))
    p_sp = np.log2(np.abs(F80 - F40)[1:] / np.abs(F160 - F80)[1:])
    F_ref = F160 + (F160 - F80) / 3.0
    print("INFO  FV spatial order (80/160): " + ", ".join(f"{p:.2f}" for p in p_sp))
    print("INFO  F_ref (Richardson FV): " + ", ".join(f"{f:.9f}" for f in F_ref))
    for M in (5, 10, 20, 40, None):
        err = np.abs(F_proj[M] - F_ref)
        lab = "all" if M is None else str(M)
        print(f"INFO  projection M={lab:>3s}: |F - F_ref| = " + ", ".join(f"{e:.1e}" for e in err)
              + f" | inventory jump at switch {F_proj[(M, 'jump')]:+.1e}")
    errs_all = np.abs(F_proj[None] - F_ref)
    check(f"t_s={t_s_h} h: projection (all modes) matches extrapolated FV to 1e-6", np.max(errs_all) < 1e-6,
          f"max {np.max(errs_all):.1e}")
    # The inventory right after the switch, F(t_s^+), is evaluated with the truncated NEW basis.
    # Its mismatch with F(t_s^-) is a new-basis truncation error; it disappears from outputs at
    # t - t_s >= 40/Lambda_max^new because the omitted new modes are damped (min valid time).
    check(f"t_s={t_s_h} h: outputs at t - t_s >= 0.25 h unaffected by new-basis truncation",
          np.max(errs_all[1:]) < 1e-9, f"max {np.max(errs_all[1:]):.1e}")
    if t_s_h == 1.0:
        l_big = eigenvalues(D2, 400.0 * l_new[-1])
        B_big = basis(D2, l_big)
        N_big = np.diag(gram(B_big, B_big))
        _, In_big = moments(B_big)
        a_big = (gram(B_big, B_old) / N_big[:, None]) @ a_ts
        F_before = 1.0 - 4 * np.pi * np.dot(a_ts, In_old) / M0
        # partial sums oscillate in sign (localised layer modes), so use the tail envelope
        errM = np.abs(1.0 - 4 * np.pi * np.cumsum(a_big * In_big) / M0 - F_before)
        env = np.maximum.accumulate(errM[::-1])[::-1]          # env(M) = max_{M' >= M} err(M')
        Ms = [10, 20, 40, 80, 160]
        Ms = [m for m in Ms if m <= len(l_big)]
        print(f"INFO  new basis with {len(l_big)} modes; envelope of |F(t_s^+) - F(t_s^-)| for M = "
              + ", ".join(f"{m}: {env[m - 1]:.2e}" for m in Ms))
        mm = np.arange(20, 161)
        env_slope = np.polyfit(np.log(mm), np.log(env[mm - 1]), 1)[0]
        print(f"INFO  envelope slope (M = 20..160): {env_slope:.2f}")
        amag = np.abs(a_big) * np.sqrt(N_big)
        idx = np.arange(1, len(a_big) + 1)
        sel = idx >= 20
        slope = np.polyfit(np.log(idx[sel]), np.log(amag[sel] + 1e-300), 1)[0]
        check("normalised projection coefficients decay like m^-2 (P22.46)", -2.3 < slope < -1.7,
              f"slope {slope:.2f}")
        check("switch-time inventory error envelope decreases with M_new", env[159] < 1e-2 * env[9],
              f"{env[9]:.1e} -> {env[159]:.1e}")
        print(f"INFO  normalised coefficient |a_m| sqrt(N_m) decays like m^{slope:.2f} (m >= 20)")
    # truncation criterion (P22.44): Lambda_{M+1}^old t_s >= 40 makes the omitted old modes negligible
    for M in (5, 10, 20):
        if M < len(l_old):
            print(f"INFO  M={M:2d}: Lambda_(M+1)^old t_s = {l_old[M] * t_s:8.2f}")

print(f"\nSUMMARY {sum(RESULTS)}/{len(RESULTS)} checks passed")
