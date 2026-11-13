"""STRATO Phase 2 open item O4: rigorous truncation bound for the modal balance form
in the layered particle (Phase 2 derivations, Section 3.5, Eq. (P21.39); new
equations (P2X.O4.k)).

What it checks
  1. Symbolic: the closed-form layer integrals used for N_n, I_n, I1_n; the Green-function
     identities sum alpha^2/Lambda, sum beta^2/Lambda, sum alpha*beta/Lambda for the
     homogeneous sphere; the antiderivative Phi(b, M) of exp(-b x^2)/x^2; the identity
     h_n = (Lambda_n mu_n + lambda (M0 + Mgen)) / kappa_n.
  2. A Python transcription of the layered shooting method (exact transfer in each layer,
     scaled Pruefer angle psi, bisection on psi(R) = n pi) against the v1.0 golden
     five-layer value F(2e5 s) in tests/golden/v1.0/verify_stdout.txt, against 40-digit
     mpmath roots, against a sign-change root count, and the n - 1 interior zeros.
  3. Eigenvalue bounds: min-max with D_min and D_max, and the travel-time (Weyl-type)
     bound (n - J) pi < sqrt(Lambda_n) tau < (n + J) pi; every interface jump below pi.
  4. Amplitude bounds |alpha_n| <= C_alpha/sqrt(Lambda_n), |beta_n| <= C_beta/sqrt(Lambda_n),
     |w_n| <= C_w/Lambda_n on every computed mode, and the closed-form Green identities
     and Bessel sums against the mode sums.
  5. Bound A (a priori, inputs only; its exponential part optionally sharpened with the
     cut-off Lambda_(N+1)) and Bound B (a posteriori, Bessel/Green) on the balance-form
     error, the inventory error, the steady R/B error and the L2(r^2) profile error in
     four scenarios; the tightness ratio |error| / bound is printed. The mode-count rule
     derived from Bound A is checked for two tolerances.
  The reference is the same series with NREF modes (40000 and 20000); its own truncation
  uncertainty is Bound A at NREF and is printed.
Cases: inputs/synthetic/synthetic_five_layer.toml (main case) and the five-layer case of
scripts/derivation_checks/b4_decay_source_pseudocode.py (152 negative weights in 300 modes).

Run from the repository root:
  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec3_o4_layered_truncation_bound.py
Writes nothing to disk. Prints PASS/FAIL lines (and INFO lines) and ends with ALL PASS
only when every check passes.
"""
import math
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
import mpmath as mp
import numpy as np
import sympy as sp
from scipy.integrate import quad

REPO = Path(__file__).resolve().parents[2]
RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


def info(text):
    print(f"INFO  {text}")


# ---------------------------------------------------------------------------
# 1 Symbolic checks
# ---------------------------------------------------------------------------
a_, b_, k_, L_, r0_, s_ = sp.symbols("a b k L r0 s", real=True)
k_, L_ = sp.symbols("k L", positive=True)
u_s = a_ * sp.cos(k_ * s_) + b_ * sp.sin(k_ * s_)
Iuu_cf = (a_**2 * (L_ / 2 + sp.sin(2 * k_ * L_) / (4 * k_)) + b_**2 * (L_ / 2 - sp.sin(2 * k_ * L_) / (4 * k_))
          + a_ * b_ * (1 - sp.cos(2 * k_ * L_)) / (2 * k_))
Iru_cf = (a_ * (r0_ * sp.sin(k_ * L_) / k_ + L_ * sp.sin(k_ * L_) / k_ - (1 - sp.cos(k_ * L_)) / k_**2)
          + b_ * (r0_ * (1 - sp.cos(k_ * L_)) / k_ - L_ * sp.cos(k_ * L_) / k_ + sp.sin(k_ * L_) / k_**2))
d1 = sp.simplify(sp.integrate(u_s**2, (s_, 0, L_)) - Iuu_cf)
d2 = sp.simplify(sp.integrate((r0_ + s_) * u_s, (s_, 0, L_)) - Iru_cf)
check("S1 closed-form layer integrals of u^2 and r u", d1 == 0 and d2 == 0)

n_, R_, D_ = sp.symbols("n R D", positive=True)
beta2 = 2 * R_**3 / (n_**2 * sp.pi**2)          # homogeneous sphere, normalised phi
lamn = n_**2 * sp.pi**2 * D_ / R_**2
q_bb = sp.simplify(sp.summation(beta2 / lamn, (n_, 1, sp.oo)) - R_**5 / (45 * D_))
check("S2 Green identity sum beta_n^2/Lambda_n = (1/9) int x^4/D dx (homogeneous sphere)", q_bb == 0)

x_, bb_, M_ = sp.symbols("x b M", positive=True)
Phi_sym = sp.exp(-bb_ * M_**2) / M_ - sp.sqrt(sp.pi * bb_) * sp.erfc(sp.sqrt(bb_) * M_)
check("S3 d/dM Phi(b, M) = -exp(-b M^2)/M^2", sp.simplify(sp.diff(Phi_sym, M_) + sp.exp(-bb_ * M_**2) / M_**2) == 0)

t_, Lam_, lam_, M0_, G_ = sp.symbols("t Lambda lambda M0 G", positive=True)
kap = Lam_ + lam_
mu = M0_ * sp.exp(-kap * t_) + G_ * (1 - sp.exp(-kap * t_)) / kap
h = mu + lam_ * sp.integrate(mu.subs(t_, s_), (s_, 0, t_))
check("S4 h_n = mu_n + lambda int mu_n = (Lambda mu_n + lambda (M0 + G t)) / kappa (constant source)",
      sp.simplify(h - (Lam_ * mu + lam_ * (M0_ + G_ * t_)) / kap) == 0)

# ---------------------------------------------------------------------------
# 2 Layered sphere: shooting, eigenpairs, moments
# ---------------------------------------------------------------------------
mp.mp.dps = 60


def Phi(b, M, shift=0.0):
    """exp(shift) * int_M^inf exp(-b x^2) x^-2 dx, evaluated with 60 digits.
    The factor exp(shift) lets the caller scale out exp(-Lambda_(N+1) t) and avoid underflow."""
    b, M = mp.mpf(b), mp.mpf(M)
    if b == 0:
        return float(mp.e ** mp.mpf(shift) / M)
    v = mp.e ** (-b * M * M) / M - mp.sqrt(mp.pi * b) * mp.erfc(mp.sqrt(b) * M)
    return float(v * mp.e ** mp.mpf(shift))


class Layered:
    def __init__(self, R, D):
        self.R = np.array(R, float)
        self.D = np.array(D, float)
        self.Rl = np.concatenate([[0.0], self.R[:-1]])
        self.L = self.R - self.Rl
        self.nl = len(R)
        self.J = self.nl - 1
        self.Rout, self.R1 = self.R[-1], self.R[0]
        self.V = self.R1**3 / 3.0
        self.tau = float(np.sum(self.L / np.sqrt(self.D)))
        self.a = math.pi**2 / self.tau**2
        self.c = self.D.min() * math.pi**2 / self.Rout**2
        self.cmax = self.D.max() * math.pi**2 / self.Rout**2
        D1, Dn, R1, Rl, Rr = self.D[0], self.D[1:], self.R1, self.Rl[1:], self.R[1:]
        self.Qbb = float(np.sum((self.R**5 - self.Rl**5) / self.D) / 45.0)
        self.Qaa = float(R1**5 / (45 * D1) + R1**6 / 9 * np.sum((1 / Rl - 1 / Rr) / Dn))
        self.Qab = float(R1**5 / (45 * D1) + R1**3 / 9 * np.sum((Rr**2 - Rl**2) / (2 * Dn)))

    # transfer through all layers, vectorised over lam
    def shoot(self, lam):
        lam = np.asarray(lam, float)
        a, b, psi = np.zeros_like(lam), np.ones_like(lam), np.zeros_like(lam)
        coef, jumps = [], []
        for i in range(self.nl):
            k = np.sqrt(lam / self.D[i])
            kL = k * self.L[i]
            coef.append((a, b, k))
            cs, sn = np.cos(kL), np.sin(kL)
            u, upk = a * cs + b * sn, -a * sn + b * cs
            psi = psi + kL
            if i < self.nl - 1:
                rr = self.D[i] / self.D[i + 1]
                up2 = rr * upk * k + (1.0 - rr) * u / self.R[i]
                upk2 = up2 / np.sqrt(lam / self.D[i + 1])
                d = np.mod(np.arctan2(u, upk2) - psi + np.pi, 2 * np.pi) - np.pi
                jumps.append(d)
                psi = psi + d
                a, b = u, upk2
        return dict(psi=psi, uR=u, upR=upk * k, coef=coef, jumps=np.array(jumps))

    def eigen(self, N):
        n = np.arange(1, N + 1, dtype=float)
        lo, hi = self.c * n**2 * (1 - 1e-12), self.cmax * n**2 * (1 + 1e-12)
        ok = bool(np.all(self.shoot(lo)["psi"] < n * np.pi) and np.all(self.shoot(hi)["psi"] > n * np.pi))
        for _ in range(100):
            mid = np.sqrt(lo * hi)
            f = self.shoot(mid)["psi"] - n * np.pi
            lo, hi = np.where(f < 0, mid, lo), np.where(f < 0, hi, mid)
        return 0.5 * (lo + hi), ok

    def modes(self, lam):
        st = self.shoot(lam)
        norm = mom = m1 = 0.0
        for i, (a, b, k) in enumerate(st["coef"]):
            L, r0 = self.L[i], self.Rl[i]
            kL = k * L
            sn, cs, s2 = np.sin(kL), np.cos(kL), np.sin(2 * kL)
            omc, omc2 = 2 * np.sin(kL / 2) ** 2, 2 * np.sin(kL) ** 2
            Iuu = a * a * (L / 2 + s2 / (4 * k)) + b * b * (L / 2 - s2 / (4 * k)) + a * b * omc2 / (2 * k)
            Iru = (a * (r0 * sn / k + L * sn / k - omc / k**2)
                   + b * (r0 * omc / k - L * cs / k + sn / k**2))
            norm, mom = norm + Iuu, mom + Iru
            if i == 0:
                m1 = Iru
        sq = np.sqrt(norm)
        alpha, beta = m1 / sq, mom / sq
        beta_flux = -self.Rout * self.D[-1] * st["upR"] / lam / sq
        return alpha, beta, alpha * beta / self.V, beta_flux, st

    # amplitude constants (P2X.O4 amplitude lemma)
    def thr(self):
        return max(self.D[0] / (4 * self.R1**2), self.D[-1] / (4 * self.L[-1] ** 2))

    def amp(self, lam):
        lam = np.asarray(lam, float)
        k1, k5 = np.sqrt(lam / self.D[0]), np.sqrt(lam / self.D[-1])
        a1, b5 = self.R1 / 2 - 1 / (4 * k1), self.L[-1] / 2 - 1 / (4 * k5)
        geo = np.sqrt(self.R1**2 + 1 / k1**2)
        Ca = np.sqrt(self.D[0]) * geo / np.sqrt(a1)
        Cb = self.Rout * np.sqrt(self.D[-1]) / np.sqrt(b5)
        f = 0.5 if self.nl >= 2 else 1.0
        Cw = np.sqrt(self.D[0] * self.D[-1]) * self.Rout * geo * f / (self.V * np.sqrt(a1 * b5))
        return Ca, Cb, Cw

    def ell(self, n):
        n = np.asarray(n, float)
        return np.maximum(self.c * n**2, np.where(n > self.J, self.a * (n - self.J) ** 2, 0.0))

    def S1(self, N):  # >= sum_{n>N} 1/ell_n
        v = 1 / (self.c * N)
        return min(v, 1 / (self.a * (N - self.J))) if N > self.J else v

    def S2(self, N):  # >= sum_{n>N} 1/ell_n^2
        v = 1 / (3 * self.c**2 * N**3)
        return min(v, 1 / (3 * self.a**2 * (N - self.J) ** 3)) if N > self.J else v

    def S3(self, N):  # >= sum_{n>N} 1/ell_n^3
        v = 1 / (5 * self.c**3 * N**5)
        return min(v, 1 / (5 * self.a**3 * (N - self.J) ** 5)) if N > self.J else v

    def EN(self, N, t, shift=0.0):  # >= exp(shift) sum_{n>N} exp(-ell_n t)/ell_n
        v = Phi(self.c * t, N, shift) / self.c
        return min(v, Phi(self.a * t, N - self.J, shift) / self.a) if N > self.J else v

    def usable(self, N):
        return float(self.ell(N + 1)) > self.thr()


def mp_uR(case, lam):
    """u(R; Lambda) with mpmath (same transfer, 60 digits)."""
    a, b = mp.mpf(0), mp.mpf(1)
    for i in range(case.nl):
        k = mp.sqrt(lam / mp.mpf(case.D[i]))
        kL = k * mp.mpf(case.L[i])
        u, upk = a * mp.cos(kL) + b * mp.sin(kL), -a * mp.sin(kL) + b * mp.cos(kL)
        if i < case.nl - 1:
            rr = mp.mpf(case.D[i]) / mp.mpf(case.D[i + 1])
            up2 = rr * upk * k + (1 - rr) * u / mp.mpf(case.R[i])
            a, b = u, up2 / mp.sqrt(lam / mp.mpf(case.D[i + 1]))
    return u


def tail(x):
    """tail[N] = sum_{n > N} x_n over the computed modes (N = 0..len(x))."""
    rc = np.cumsum(x[::-1])[::-1]
    return np.concatenate([rc, [0.0]])


def phi1(x):
    x = np.asarray(x, float)
    small = x < 1e-4
    xs = np.where(small, 1.0, x)
    return np.where(small, 1 - x / 2 + x * x / 6, -np.expm1(-xs) / xs)


def phi2(x):
    x = np.asarray(x, float)
    small = x < 1e-3
    xs = np.where(small, 1.0, x)
    return np.where(small, 0.5 - x / 6 + x * x / 24, (xs + np.expm1(-xs)) / xs**2)


def read_synthetic():
    raw = (REPO / "inputs/synthetic/synthetic_five_layer.toml").read_text()
    R = [float(v) for v in re.findall(r"^outer_radius_m\s*=\s*([0-9.eE+-]+)", raw, re.M)]
    D = [float(v) for v in re.findall(r"^diffusivity_m2_s\s*=\s*([0-9.eE+-]+)", raw, re.M)]
    return R, D


# ---------------------------------------------------------------------------
# 3 Run one case
# ---------------------------------------------------------------------------
def run_case(tag, R, D, NREF, golden=None, full=True):
    cs = Layered(R, D)
    print(f"---- case {tag}: {cs.nl} layers, NREF = {NREF}")
    info(f"{tag} tau = {cs.tau:.6g} s^0.5, a = pi^2/tau^2 = {cs.a:.6g} 1/s, c_min = {cs.c:.6g} 1/s, "
         f"threshold Lambda* = {cs.thr():.4g} 1/s, J = {cs.J}")
    lam, ok = cs.eigen(NREF)
    check(f"{tag} E1 bisection brackets from min-max are valid for all {NREF} modes", ok)
    n = np.arange(1, NREF + 1, dtype=float)
    alpha, beta, w, beta_flux, st = cs.modes(lam)
    check(f"{tag} E2 psi(R) = n pi at every root", np.max(np.abs(st["psi"] - n * np.pi)) < 1e-9 * NREF,
          f"max |psi - n pi| = {np.max(np.abs(st['psi'] - n * np.pi)):.1e}")
    rel_flux = np.max(np.abs(beta - beta_flux) / np.abs(beta))
    check(f"{tag} E3 beta_n by moments equals the flux identity (P21.36)", rel_flux < 1e-8, f"max rel diff {rel_flux:.1e}")
    check(f"{tag} E4 eigenvalues strictly increasing", bool(np.all(np.diff(lam) > 0)))
    neg300 = int(np.sum(w[:300] < 0))
    info(f"{tag} negative weights among the first 300 modes: {neg300}; among {NREF}: {int(np.sum(w < 0))}")

    # mpmath roots
    worst = 0.0
    for m in [1, 2, 3, 10, 100, 1000, min(10000, NREF)]:
        lm = mp.findroot(lambda L: mp_uR(cs, L), mp.mpf(lam[m - 1]), tol=mp.mpf(10) ** -50)
        worst = max(worst, abs(float((mp.mpf(lam[m - 1]) - lm) / lm)))
    check(f"{tag} E5 eigenvalues agree with 60-digit mpmath roots (n = 1 ... 10000)", worst < 1e-13, f"max rel diff {worst:.1e}")

    # root count by sign changes of u(R) on a fine sqrt(Lambda) grid below the midpoint of modes 200, 201
    top = 0.5 * (lam[199] + lam[200])
    g = np.linspace(1e-9, math.sqrt(top), 400000) ** 2
    uR = cs.shoot(g)["uR"]
    nsc = int(np.sum(np.sign(uR[1:]) * np.sign(uR[:-1]) < 0))
    check(f"{tag} E6 sign changes of u(R; Lambda) below (Lambda_200 + Lambda_201)/2 equal 200", nsc == 200, f"count {nsc}")

    # interior zeros of u_n: n - 1 (oscillation theorem), checked on a fine grid
    zok = True
    for m in list(range(1, 41)) + [100, 300, 1000]:
        coef = cs.modes(np.array([lam[m - 1]]))[4]["coef"]
        vals = []
        for i, (a, b, k) in enumerate(coef):
            npts = int(60 * k[0] * cs.L[i] / math.pi) + 50
            s = np.linspace(0, cs.L[i], npts)[1:]
            if i == cs.nl - 1:
                s = s[:-1]
            vals.append(a[0] * np.cos(k[0] * s) + b[0] * np.sin(k[0] * s))
        v = np.concatenate(vals)
        zc = int(np.sum(np.sign(v[1:]) * np.sign(v[:-1]) < 0))
        zok &= (zc == m - 1)
    check(f"{tag} E7 mode n has n - 1 interior zeros (n = 1..40, 100, 300, 1000)", zok)

    # eigenvalue bounds
    sq = np.sqrt(lam) * cs.tau / math.pi
    check(f"{tag} B1 min-max: D_min (n pi/R)^2 <= Lambda_n <= D_max (n pi/R)^2",
          bool(np.all(lam >= cs.c * n**2) and np.all(lam <= cs.cmax * n**2)),
          f"min Lambda_n/(c n^2) = {np.min(lam / (cs.c * n**2)):.3f}")
    check(f"{tag} B2 travel time: n - J < sqrt(Lambda_n) tau/pi < n + J",
          bool(np.all(sq > n - cs.J) and np.all(sq < n + cs.J)),
          f"range of sqrt(Lambda_n) tau/pi - n: [{np.min(sq - n):.3f}, {np.max(sq - n):.3f}]")
    check(f"{tag} B3 every interface jump of psi is below pi", float(np.max(np.abs(st["jumps"]))) < math.pi,
          f"max |delta|/pi = {np.max(np.abs(st['jumps'])) / math.pi:.3f}")
    ell_n = cs.ell(n)
    check(f"{tag} B4 Lambda_n >= ell_n (combined lower bound)", bool(np.all(lam >= ell_n)),
          f"Lambda_n/ell_n at n = 10, 100, {NREF}: {lam[9] / ell_n[9]:.3f}, {lam[99] / ell_n[99]:.3f}, {lam[-1] / ell_n[-1]:.4f}")

    # amplitude lemma
    use = lam > cs.thr()
    Ca, Cb, Cw = cs.amp(np.where(use, lam, 2 * cs.thr()))
    ra = np.abs(alpha) * np.sqrt(lam) / Ca
    rb = np.abs(beta) * np.sqrt(lam) / Cb
    rw = np.abs(w) * lam / Cw
    check(f"{tag} B5 |alpha_n| <= C_alpha(Lambda_n)/sqrt(Lambda_n) for Lambda_n > Lambda*", bool(np.all(ra[use] <= 1)),
          f"max ratio {ra[use].max():.3f}")
    check(f"{tag} B6 |beta_n| <= C_beta(Lambda_n)/sqrt(Lambda_n) for Lambda_n > Lambda*", bool(np.all(rb[use] <= 1)),
          f"max ratio {rb[use].max():.3f}")
    check(f"{tag} B7 |w_n| <= C_w(Lambda_n)/Lambda_n for Lambda_n > Lambda*", bool(np.all(rw[use] <= 1)),
          f"max ratio {rw[use].max():.3f}, median {np.median(rw[use]):.3f}, modes covered {int(use.sum())}/{NREF}")
    lg = np.geomspace(cs.thr() * 1.0001, lam[-1] * 10, 2000)
    cw_g = cs.amp(lg)[2]
    check(f"{tag} B8 C_w(Lambda) and C_alpha(Lambda) decrease above Lambda*", bool(np.all(np.diff(cw_g) < 0) and np.all(np.diff(cs.amp(lg)[0]) < 0)))
    info(f"{tag} C_w(inf) = {float(cs.amp(1e30)[2]):.5g} 1/s, C_w(ell_11) = {float(cs.amp(cs.ell(11))[2]):.5g} 1/s")

    # Green identities and Bessel sums against the mode sums
    def qnum(fa, fb):
        tot = 0.0
        for i in range(cs.nl):
            tot += quad(lambda x: fa(x) * fb(x) / (x * x * cs.D[i]), max(cs.Rl[i], 1e-30), cs.R[i], limit=200, epsabs=0, epsrel=1e-13)[0]
        return tot
    F1 = lambda x: x**3 / 3
    Fc = lambda x: min(x, cs.R1) ** 3 / 3
    qa, qb, qab = qnum(Fc, Fc), qnum(F1, F1), qnum(Fc, F1)
    check(f"{tag} G1 closed forms Q_aa, Q_bb, Q_ab equal the quadrature of F_f F_g/(x^2 D)",
          max(abs(qa / cs.Qaa - 1), abs(qb / cs.Qbb - 1), abs(qab / cs.Qab - 1)) < 1e-11)
    Car, Cbr, Cwr = [float(v) for v in cs.amp(cs.ell(NREF + 1))]
    ea = cs.V - math.fsum(alpha**2)
    eb = cs.Rout**3 / 3 - math.fsum(beta**2)
    ew = 1 - math.fsum(w)
    S1r = cs.S1(NREF)
    check(f"{tag} G5 Bessel: 0 <= R1^3/3 - sum alpha^2 <= C_alpha^2 S1(NREF)", -1e-14 <= ea / cs.V <= Car**2 * S1r / cs.V,
          f"defect rel {ea / cs.V:.2e}, bound rel {Car**2 * S1r / cs.V:.2e}")
    check(f"{tag} G6 Bessel: 0 <= R^3/3 - sum beta^2 <= C_beta^2 S1(NREF)", -1e-14 <= eb <= Cbr**2 * S1r,
          f"defect rel {eb / (cs.Rout**3 / 3):.2e}, bound rel {Cbr**2 * S1r / (cs.Rout**3 / 3):.2e}")
    check(f"{tag} G7 |1 - sum w_n| <= C_w S1(NREF)", abs(ew) <= Cwr * S1r, f"1 - sum w = {ew:.2e}, bound {Cwr * S1r:.2e}")

    if golden is not None:
        Fend = 1 - math.fsum(w * np.exp(-lam * 2.0e5))
        check(f"{tag} E8 F(2e5 s) equals the v1.0 golden value", abs(Fend - golden) < 1e-12, f"{Fend:.13f} vs {golden:.13f}")

    # ------------------------------------------------------------------
    # bounds A and B on the scenarios
    # ------------------------------------------------------------------
    pa2 = np.concatenate([[0.0], np.cumsum(alpha**2)])
    pb2 = np.concatenate([[0.0], np.cumsum(beta**2)])
    pal = np.concatenate([[0.0], np.cumsum(alpha**2 / lam)])
    pbl = np.concatenate([[0.0], np.cumsum(beta**2 / lam)])

    def EX(N, t, shift=0.0):
        # >= exp(shift) sum_{n>N} exp(-Lambda_n t)/Lambda_n, using Lambda_n >= max(ell_n, Lambda_(N+1)):
        # minimum over theta of exp(-theta Lambda_(N+1) t) E_N((1 - theta) t)
        return min(cs.EN(N, (1 - th) * t, shift - th * lam[N] * t) for th in (0.0, 0.5, 0.9, 1.0))

    def boundA(N, t, M0, Gmax, lamd, Mgen):
        Cw_ = float(cs.amp(cs.ell(N + 1))[2])
        return Cw_ * (M0 * EX(N, t) + (Gmax + lamd * (M0 + Mgen)) * cs.S2(N))

    def boundB(N, t, M0, Gmax, lamd, Mgen):
        A_ = cs.Qaa - pal[N]
        B_ = cs.Qbb - pbl[N]
        a0 = cs.V - pa2[N]
        b0 = cs.Rout**3 / 3 - pb2[N]
        if min(A_, B_, a0, b0) <= 0:
            return math.inf
        L1 = lam[N]
        sig = L1 * math.exp(-L1 * t) if L1 * t >= 1 else (1 / (math.e * t) if t > 0 else math.inf)
        T1 = math.sqrt(A_ * B_) / cs.V
        T0 = min(math.exp(-L1 * t) * math.sqrt(a0 * b0), sig * math.sqrt(A_ * B_)) / cs.V
        return M0 * T0 + (Gmax + lamd * (M0 + Mgen)) * T1

    L1s = lam[0]
    Ns = [N for N in [5, 10, 20, 50, 100, 200, 300, 500, 1000, 2000, 5000] if N < NREF // 4 and cs.usable(N)]
    info(f"{tag} N values tested: {Ns}; Lambda_1 = {L1s:.6g} 1/s")
    # B is reliable while its Green defects are well above round-off of the partial sums
    reliableB = [N for N in Ns if (cs.Qaa - pal[N]) > 1e-9 * cs.Qaa and (cs.Qbb - pbl[N]) > 1e-9 * cs.Qbb]
    info(f"{tag} Bound B evaluated (Green defects above 1e-9 of the totals) for N in {reliableB}")

    # Green defects at the tested N (above round-off): 0 <= Q - partial_N <= bound
    gok = [True, True, True]
    for N in reliableB:
        Ca_, Cb_, Cw_ = [float(v) for v in cs.amp(cs.ell(N + 1))]
        da, db = cs.Qaa - pal[N], cs.Qbb - pbl[N]
        dab = cs.Qab - math.fsum(alpha[:N] * beta[:N] / lam[:N])
        gok[0] &= 0 <= da <= Ca_**2 * cs.S2(N)
        gok[1] &= 0 <= db <= Cb_**2 * cs.S2(N)
        gok[2] &= abs(dab) <= cs.V * Cw_ * cs.S2(N)
    check(f"{tag} G2 0 <= Q_aa - sum_(n<=N) alpha^2/Lambda <= C_alpha^2 S2(N), N in {reliableB}", gok[0])
    check(f"{tag} G3 0 <= Q_bb - sum_(n<=N) beta^2/Lambda <= C_beta^2 S2(N)", gok[1])
    check(f"{tag} G4 |Q_ab - sum_(n<=N) alpha beta/Lambda| <= V C_w S2(N)", gok[2])
    rd = max(abs(cs.Qaa - pal[NREF]) / cs.Qaa, abs(cs.Qbb - pbl[NREF]) / cs.Qbb)
    check(f"{tag} G4b at NREF the Green defects are at round-off (below 1e-12 relative)", rd < 1e-12, f"{rd:.1e}")

    rows = []
    # S1: no source, no decay: F tail, inventory tail
    times = [1e-4 / L1s, 1e-3 / L1s, 1e-2 / L1s, 0.1 / L1s, 1 / L1s, 3 / L1s] + ([2.0e5] if golden else [])
    okA = okB = True
    ratA, ratB = [], []
    for t in times:
        for N in Ns:
            # every quantity is multiplied by exp(Lambda_(N+1) t) so that nothing underflows
            sh = lam[N] * t
            tr = abs(math.fsum(w[N:] * np.exp(-(lam[N:] - lam[N]) * t)))
            bA0 = float(cs.amp(cs.ell(N + 1))[2]) * cs.EN(N, t, sh)
            bA = float(cs.amp(cs.ell(N + 1))[2]) * EX(N, t, sh)
            okA &= tr <= bA and tr <= bA0
            ratA.append(tr / bA)
            bB = math.nan
            if N in reliableB:
                a0, b0 = cs.V - pa2[N], cs.Rout**3 / 3 - pb2[N]
                A_, B_ = cs.Qaa - pal[N], cs.Qbb - pbl[N]
                sig = lam[N] if sh >= 1 else math.exp(sh) / (math.e * t)
                bB = min(math.sqrt(a0 * b0), sig * math.sqrt(A_ * B_)) / cs.V
                okB &= tr <= bB
                ratB.append(tr / bB)
            rows.append(("S1", t, N, tr, bA, bB, sh))
    check(f"{tag} T1 S1 (lambda = 0, no source): |sum_(n>N) w_n e^(-Lambda_n t)| <= Bound A (theta = 0 and theta-minimised), all N, t (scaled by e^(Lambda_(N+1) t))", okA,
          f"tightness |err|/A: max {max(ratA):.2e}, median {np.median(ratA):.2e}")
    check(f"{tag} T2 S1: |error| <= Bound B (a posteriori)", okB,
          f"tightness |err|/B: max {max(ratB):.2e}, median {np.median(ratB):.2e}")

    # S4 (exact closed form, lambda = 0): steady inventory per unit source, sum_{n>N} w_n/Lambda_n
    okA = okB = True
    ratA, ratB = [], []
    for N in Ns:
        ex = cs.Qab / cs.V - math.fsum(w[:N] / lam[:N])
        ref = tail(w / lam)[N]
        allow = 1000 * 2.2e-16 * cs.Qab / cs.V
        bA = boundA(N, 1.0, 0.0, 1.0, 0.0, 0.0)
        okA &= abs(ex) <= bA + allow and abs(ref) <= bA
        ratA.append(abs(ref) / bA)
        if N in reliableB:
            bB = boundB(N, 1.0, 0.0, 1.0, 0.0, 0.0)
            okB &= abs(ex) <= bB and abs(ref) <= bB
            ratB.append(abs(ref) / bB)
        rows.append(("S4", math.inf, N, abs(ref), bA, boundB(N, 1.0, 0.0, 1.0, 0.0, 0.0) if N in reliableB else math.nan, abs(ex)))
    check(f"{tag} T3 S4 (lambda = 0 steady inventory): NREF tail and exact Green total Q_ab/V - sum_(n<=N) w_n/Lambda_n (+ 1000 eps Q_ab/V) <= Bound A", okA,
          f"tightness max {max(ratA):.2e}, median {np.median(ratA):.2e}")
    check(f"{tag} T4 S4: same error <= Bound B", okB, f"tightness max {max(ratB):.2e}, median {np.median(ratB):.2e}")

    # S2: steady R/B with decay, constant kernel source from zero
    for lamd in [L1s, 100 * L1s]:
        kap = lam + lamd
        okA = okB = True
        ratA = []
        refgap = 0.0
        for N in Ns:
            ex = lamd * ((cs.Qab / cs.V - math.fsum(w[:N] / lam[:N])) - lamd * tail(w / (lam * kap))[N])
            ref = lamd * tail(w / kap)[N]
            refgap = max(refgap, abs(ex - ref))
            bA = lamd * float(cs.amp(cs.ell(N + 1))[2]) * cs.S2(N)
            allow = 1000 * 2.2e-16 * lamd * cs.Qab / cs.V
            okA &= abs(ref) <= bA and abs(ex) <= bA + allow
            ratA.append(abs(ref) / bA)
            if N in reliableB:
                okB &= abs(ref) <= lamd * math.sqrt((cs.Qaa - pal[N]) * (cs.Qbb - pbl[N])) / cs.V
            rows.append(("S2", lamd, N, abs(ref), bA, lamd * math.sqrt((cs.Qaa - pal[N]) * (cs.Qbb - pbl[N])) / cs.V if N in reliableB else math.nan))
        bref = lamd * float(cs.amp(cs.ell(NREF + 1))[2]) * cs.S2(NREF)
        check(f"{tag} T5 S2 steady R/B, lambda = {lamd / L1s:.0f} Lambda_1: |error| <= Bound A and <= Bound B", okA and okB,
              f"tightness max {max(ratA):.2e}; closed form vs NREF sum differ by {refgap:.1e} (Bound A at NREF {bref:.1e})")
        check(f"{tag} T6 S2 the closed-form reference and the NREF sum agree within Bound A(NREF) + 1000 eps lambda Q_ab/V",
              refgap <= bref + 1000 * 2.2e-16 * lamd * cs.Qab / cs.V)

    # mode-count rule from Bound A: smallest N with lambda C_w(ell_(N+1)) S2(N) <= tol (steady R/B)
    for tol in (1e-6, 1e-9):
        lamd = L1s
        Nq = next(N for N in range(cs.J + 1, 10**6) if cs.usable(N)
                  and lamd * float(cs.amp(cs.ell(N + 1))[2]) * cs.S2(N) <= tol)
        err = abs(lamd * tail(w / (lam + lamd))[Nq])
        check(f"{tag} T6b mode-count rule: N = {Nq} modes give steady R/B error <= {tol:.0e} (lambda = Lambda_1)",
              err <= tol, f"true error {err:.2e}")
    # S3: kernel inventory M0, source G on [0, t_off), decay lambda, then post-irradiation
    for lamd in [L1s, 100 * L1s]:
        M0, G, toff = 1.0, L1s, 1.0 / L1s
        kap = lam + lamd
        okA = okB = okM = okI = True
        ratA, ratB = [], []
        for t in [1e-3 / L1s, 0.1 / L1s, 0.5 / L1s, 1.0 / L1s, 1.5 / L1s, 3.0 / L1s]:
            t1 = min(t, toff)
            mu1 = M0 * np.exp(-kap * t1) + G * t1 * phi1(kap * t1)
            int1 = M0 * t1 * phi1(kap * t1) + G * t1**2 * phi2(kap * t1)
            if t > toff:
                dt = t - toff
                mu = mu1 * np.exp(-kap * dt)
                intm = int1 + mu1 * dt * phi1(kap * dt)
            else:
                mu, intm = mu1, int1
            Mgen = G * t1
            hn = mu + lamd * intm
            ident = np.max(np.abs(hn - (lam * mu + lamd * (M0 + Mgen)) / kap) / hn)
            okI &= ident < 1e-9
            tE = tail(w * hn)
            tM = tail(w * mu)
            for N in Ns:
                bA = boundA(N, t, M0, G, lamd, Mgen)
                okA &= abs(tE[N]) <= bA
                ratA.append(abs(tE[N]) / bA)
                bAM = float(cs.amp(cs.ell(N + 1))[2]) * (M0 * EX(N, t) + G * cs.S2(N))
                okM &= abs(tM[N]) <= bAM
                if N in reliableB:
                    bB = boundB(N, t, M0, G, lamd, Mgen)
                    okB &= abs(tE[N]) <= bB
                    ratB.append(abs(tE[N]) / bB)
                rows.append(("S3", (lamd, t), N, abs(tE[N]), bA, boundB(N, t, M0, G, lamd, Mgen) if N in reliableB else math.nan))
        check(f"{tag} T7 S3 identity h_n = (Lambda mu + lambda (M0 + Mgen))/kappa, lambda = {lamd / L1s:.0f} Lambda_1", okI)
        check(f"{tag} T8 S3 balance-form release error <= Bound A, lambda = {lamd / L1s:.0f} Lambda_1", okA,
              f"tightness max {max(ratA):.2e}, median {np.median(ratA):.2e}")
        check(f"{tag} T9 S3 inventory error |M - M^(N)| <= C_w (M0 E_N + G S2), lambda = {lamd / L1s:.0f} Lambda_1", okM)
        check(f"{tag} T10 S3 balance-form release error <= Bound B, lambda = {lamd / L1s:.0f} Lambda_1", okB,
              f"tightness max {max(ratB):.2e}, median {np.median(ratB):.2e}")

    # L2(r^2) profile error, kernel-uniform start c0 = 1, no source, no decay
    okL = True
    ratL = []
    for t in [0.0, 1e-3 / L1s, 1e-1 / L1s, 1.0 / L1s]:
        tl = tail(alpha**2 * np.exp(-2 * lam * t))
        for N in Ns:
            tr = (cs.V - pa2[N]) if t == 0 else tl[N]
            Ca_ = float(cs.amp(cs.ell(N + 1))[0])
            bL = Ca_**2 * cs.EN(N, 2 * t)
            okL &= tr <= bL
            if tr > 1e-280:
                ratL.append(tr / bL)
    check(f"{tag} T11 L2(r^2) profile error^2 <= C_alpha^2 E_N(2t) (t = 0 by Bessel exactly)", okL,
          f"tightness max {max(ratL):.2e}, median {np.median(ratL):.2e}")

    # observed orders at fixed t (steady S4 and S2)
    if full:
        sel = [r for r in rows if r[0] == "S4"]
        NN = np.array([r[2] for r in sel], float)
        er = np.array([r[3] for r in sel])
        bA = np.array([r[4] for r in sel])
        big = (NN >= 50) & (NN <= 300)
        pe = -np.polyfit(np.log(NN[big]), np.log(er[big]), 1)[0]
        pb = -np.polyfit(np.log(NN[big]), np.log(bA[big]), 1)[0]
        unc = float(cs.amp(cs.ell(NREF + 1))[2]) * cs.S2(NREF)
        info(f"{tag} S4 observed order for 50 <= N <= 300: true error {pe:.2f}, Bound A {pb:.2f}; "
             f"reference uncertainty (Bound A at NREF) {unc:.1e} s, so rows with |error| below it are indicative only")
        print(f"INFO  {tag} table: scenario | parameter | N | |error| | Bound A | Bound B")
        for r in rows:
            if (r[0] == "S1" and r[1] in (times[1], times[3]) and r[2] in (10, 50, 300, 1000)) or \
               (r[0] in ("S4", "S2") and r[2] in (10, 50, 100, 300, 1000, 2000)) or \
               (r[0] == "S3" and r[1][1] == 1.5 / L1s and r[2] in (10, 50, 300, 1000)):
                par = f"{r[1]:.4g}" if not isinstance(r[1], tuple) else f"lam={r[1][0]:.3g},t={r[1][1]:.4g}"
                extra = f" | values scaled by exp({r[6]:.4g})" if r[0] == "S1" else ""
                print(f"INFO  {tag} {r[0]} | {par} | {r[2]} | {r[3]:.3e} | {r[4]:.3e} | {r[5]:.3e}{extra}")
    return cs


R, D = read_synthetic()
check("I1 synthetic input read: five radii and five diffusivities", len(R) == 5 and len(D) == 5,
      f"R = {R}, D = {D}")
gold_txt = (REPO / "tests/golden/v1.0/verify_stdout.txt").read_text()
gold = float(re.search(r"FIVE_LAYER exact_F_end=([0-9.eE+-]+)", gold_txt).group(1))
run_case("SYN", R, D, 40000, golden=gold, full=True)

# b4 check case of the pseudocode (152 negative weights in 300 modes)
RB4 = [250e-6, 350e-6, 390e-6, 425e-6, 465e-6]
DB4 = [4e-14, 4e-13, 2e-14, 5e-15, 2e-14]
cb4 = run_case("B4", RB4, DB4, 20000, golden=None, full=True)
wb4 = cb4.modes(cb4.eigen(300)[0])[2]
check("B4 X1 the b4 check case has 152 negative weights in its first 300 modes", int(np.sum(wb4 < 0)) == 152,
      f"{int(np.sum(wb4 < 0))}")

print()
print("ALL PASS" if all(RESULTS) else f"FAILURES: {RESULTS.count(False)} of {len(RESULTS)}")
