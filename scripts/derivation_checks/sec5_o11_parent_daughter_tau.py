"""STRATO Phase 2, open item O11: effective time for a parent-daughter pair whose
diffusivities differ.  Checks every key step of the derivation section (P2X.O11.k).

Symbolic part (SymPy):
  S1  substitution c_k = g_k(t) chat_k(r, tau(t)) gives the transformed coefficients
      alpha_k/tau', (lambda_k + g_k'/g_k)/tau' and b lambda_P g_P / (g_D tau')
  S2  a constant matrix commuting with diag(k1, k2), k1 != k2, is diagonal
  S3  condition N2 in integral form is the Bernoulli ODE alpha' = delta alpha^2 - dl alpha,
      and the closed-form Bernoulli solution satisfies it
  S4  a linear ramp of an Arrhenius law cannot satisfy N2 (polynomial against exponential in 1/T)
  S5  identical operators: c = exp(M t) chat(tau(t)) solves the coupled modal system for any alpha
  S6  Duhamel cascade formula for the daughter mode solves its modal ODE (two clocks)
  S7  d/dt ln(D_D/D_P) for one-branch Arrhenius laws is (Q_D - Q_P) T' / (R_g T^2)
  S8  (D - Dt)^2 / D = Dt (1 - rho)^2 / rho with rho = Dt / D
  S9  the two time integrals of the a priori daughter bound
  S10 d/dT [T E_2(theta/T)] = exp(-theta/T) (antiderivative used for tau on a ramp)
Numerical part (synthetic data only; no literature value is used):
  N1  ramp integral of D in closed form against adaptive quadrature
  N2  joint constant-coefficient form under N1 and N2 (Bernoulli history) against a stiff
      real-time integration of the modal system
  N3  identical operators, linear ramp: exp(M t) form against the stiff integration
  N4  N2 fit residual: zero for the Bernoulli history, large for a ramp with equal Q
  N5  counterexample, different Q: the clocks implied by parent and daughter modes differ
  N6  two-clock cascade (exact, semi-analytical) against the stiff integration, different Q
  N7  equal Q, ramp: freezing the coupling (pretending N2) gives a visible error
  N8  two-layer sphere, species-wise shared Q, Q_P != Q_D: cascade with the overlap matrix
      against a two-species finite-volume solution (spatial convergence)
  N9  single sphere: a priori and a posteriori L2 error bounds for surrogate diffusivities
  N10 two-layer finite volume with layer-specific Q: discrete a priori bounds hold
  N11 range of the diffusivity ratio for which the a priori bound is informative

Run from the repository root:
    .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec5_o11_parent_daughter_tau.py
Writes nothing to disk.  Prints one PASS/FAIL line per check and ALL PASS when all pass.
"""
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import mpmath as mp
import numpy as np
import scipy.sparse as sps
import sympy as sp
from scipy.integrate import quad, solve_ivp
from scipy.special import expn

from sec3_p21_common import Multilayer, SphereFV

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


# =====================================================================================
# Symbolic part
# =====================================================================================
t, s, x = sp.symbols("t s x", real=True)
lP, lD, b = sp.symbols("lambda_P lambda_D b", positive=True)
aP_, aD_ = sp.Function("alpha_P")(t), sp.Function("alpha_D")(t)
tau = sp.Function("tau")(t)
gP, gD = sp.Function("g_P")(t), sp.Function("g_D")(t)
chP, chD, dchP, dchD, LchP, LchD = sp.symbols("chP chD dchP dchD LchP LchD")

# S1: c_k = g_k chat_k(tau(t)); L*_k commutes with the scalar g_k(t), so L_k c_k = alpha_k g_k L*_k chat_k
dtcP = sp.diff(gP, t) * chP + gP * sp.diff(tau, t) * dchP
dtcD = sp.diff(gD, t) * chD + gD * sp.diff(tau, t) * dchD
resP = dtcP - (aP_ * gP * LchP - lP * gP * chP)
resD = dtcD - (aD_ * gD * LchD - lD * gD * chD + b * lP * gP * chP)
tp = sp.diff(tau, t)
claimP = dchP - ((aP_ / tp) * LchP - (lP + sp.diff(gP, t) / gP) / tp * chP)
claimD = dchD - ((aD_ / tp) * LchD - (lD + sp.diff(gD, t) / gD) / tp * chD + b * lP * gP / (gD * tp) * chP)
okS1 = sp.simplify(resP - gP * tp * claimP) == 0 and sp.simplify(resD - gD * tp * claimD) == 0
check("S1 transformed coefficients alpha_k/tau', (lambda_k + g_k'/g_k)/tau', b lambda_P g_P/(g_D tau') (P2X.O11.5)", okS1)

# S2: W diag(k1,k2) = diag(k1,k2) W forces W12 = W21 = 0 when k1 != k2
k1, k2 = sp.symbols("k1 k2")
W = sp.Matrix(2, 2, sp.symbols("w11 w12 w21 w22"))
K = sp.diag(k1, k2)
comm = sp.simplify(W * K - K * W)
okS2 = comm[0, 0] == 0 and comm[1, 1] == 0 and sp.factor(comm[0, 1]) == sp.factor(W[0, 1] * (k2 - k1)) \
    and sp.factor(comm[1, 0]) == sp.factor(W[1, 0] * (k1 - k2))
check("S2 commutator [W, diag(k1,k2)] = (k2-k1)[[0, w12], [-w21, 0]] (step 2 of the proof)", okS2, f"{comm.tolist()}")

# S3: alpha = C exp(delta tau - dl t), tau' = alpha  =>  alpha' = delta alpha^2 - dl alpha
C, dl, de, a0 = sp.symbols("C Delta_lambda delta alpha_0", real=True)
alpha_n2 = C * sp.exp(de * tau - dl * t)
lhs = sp.diff(alpha_n2, t).subs(sp.Derivative(tau, t), alpha_n2)
okS3a = sp.simplify(lhs - (de * alpha_n2**2 - dl * alpha_n2)) == 0
alpha_b = dl * a0 / (de * a0 + (dl - de * a0) * sp.exp(dl * t))
okS3b = sp.simplify(sp.diff(alpha_b, t) - (de * alpha_b**2 - dl * alpha_b)) == 0 and sp.simplify(alpha_b.subs(t, 0) - a0) == 0
check("S3 N2 <=> alpha' = delta alpha^2 - dl alpha; Bernoulli solution (P2X.O11.9)", okS3a and okS3b)

# S4: theta r x^2 + dl = d' exp(-theta x) on an x-interval forces d' = 0 and then r = dl = 0
th, rr, dp = sp.symbols("theta r dprime", real=True)
F4 = th * rr * x**2 + dl - dp * sp.exp(-th * x)
third = sp.simplify(sp.diff(F4, x, 3))
okS4 = sp.simplify(third - th**3 * dp * sp.exp(-th * x)) == 0
poly = sp.Poly(th * rr * x**2 + dl, x)
okS4 = okS4 and poly.coeffs() == [th * rr, dl]
check("S4 ramp: third x-derivative is theta^3 d' exp(-theta x), so d' = 0, then theta r = dl = 0 (P2X.O11.11)",
      okS4, f"d3/dx3 = {third}")

# S5: identical operators, per mode x' = (-tau' Lam I + M) x, x = exp(M t) exp(-Lam tau) x0
Lam = sp.symbols("Lambda", positive=True)
M = sp.Matrix([[-lP, 0], [b * lP, -lD]])
expMt = sp.Matrix([[sp.exp(-lP * t), 0], [b * lP * (sp.exp(-lP * t) - sp.exp(-lD * t)) / (lD - lP), sp.exp(-lD * t)]])
okS5a = sp.simplify(sp.diff(expMt, t) - M * expMt) == sp.zeros(2, 2) and expMt.subs(t, 0) == sp.eye(2)
x0 = sp.Matrix(sp.symbols("x0P x0D"))
xs = expMt * sp.exp(-Lam * tau) * x0
okS5 = okS5a and sp.simplify(sp.diff(xs, t) - (-sp.diff(tau, t) * Lam * sp.eye(2) + M) * xs) == sp.zeros(2, 1)
check("S5 identical operators: c = exp(M t) chat(tau(t)) is exact for any alpha (P2X.O11.12)", okS5)

# S6: cascade.  a_D(t) = int_0^t exp(-lD (t-s) - LD (tauD(t) - tauD(s))) f(s) ds solves
#     a_D' = -(lD + LD tauD'(t)) a_D + f(t)
LD_ = sp.symbols("Lambda_D", positive=True)
tauD = sp.Function("tau_D")
f = sp.Function("f")
aD_int = sp.Integral(sp.exp(-lD * (t - s) - LD_ * (tauD(t) - tauD(s))) * f(s), (s, 0, t))
resS6 = sp.diff(aD_int, t) + (lD + LD_ * sp.diff(tauD(t), t)) * aD_int - f(t)
resS6 = sp.simplify(sp.expand(resS6.doit()))
check("S6 two-clock Duhamel formula solves the daughter modal ODE (P2X.O11.14)", resS6 == 0, f"residual {resS6}")

# S7: Arrhenius ratio
QP, QD, Rg, D0P, D0D = sp.symbols("Q_P Q_D R_g D0P D0D", positive=True)
T = sp.Function("T")(t)
ratio = (D0D * sp.exp(-QD / (Rg * T))) / (D0P * sp.exp(-QP / (Rg * T)))
dln = sp.simplify(sp.diff(sp.log(ratio), t) - (QD - QP) * sp.diff(T, t) / (Rg * T**2))
check("S7 d/dt ln(D_D/D_P) = (Q_D - Q_P) T'/(R_g T^2) (P2X.O11.8)", dln == 0)

# S8
Dv, Dt_ = sp.symbols("D Dtilde", positive=True)
rho = Dt_ / Dv
check("S8 (D - Dt)^2/D = Dt (1 - rho)^2/rho (P2X.O11.17)", sp.simplify((Dv - Dt_)**2 / Dv - Dt_ * (1 - rho)**2 / rho) == 0)

# S9: int_0^t b lP e^{-lP s} b (1 - e^{-lP s}) ds and int_0^t b lP e^{-lP s} ds
tt = sp.symbols("t", positive=True)
i1 = sp.integrate(b * lP * sp.exp(-lP * s) * b * (1 - sp.exp(-lP * s)), (s, 0, tt))
i2 = sp.integrate(b * lP * sp.exp(-lP * s), (s, 0, tt))
okS9 = sp.simplify(i1 - b**2 / 2 * (1 - sp.exp(-lP * tt))**2) == 0 and sp.simplify(i2 - b * (1 - sp.exp(-lP * tt))) == 0
check("S9 integrals of the a priori daughter bound (P2X.O11.20)", okS9)

# S10: d/dT [T E2(theta/T)] = exp(-theta/T), checked to 30 digits at three points
mp.mp.dps = 40
errs = []
for thv, Tv in [(36081.0, 1273.0), (48108.0, 1600.0), (24055.0, 1500.0)]:
    thv, Tv = mp.mpf(thv), mp.mpf(Tv)
    d = mp.diff(lambda TT: TT * mp.expint(2, thv / TT), Tv)
    errs.append(abs(d / mp.e**(-thv / Tv) - 1))
check("S10 d/dT [T E2(theta/T)] = exp(-theta/T)", max(errs) < 1e-30, f"max rel err {float(max(errs)):.1e}")

# =====================================================================================
# Numerical part, single homogeneous sphere (synthetic data)
# =====================================================================================
RG = 8.314462618               # J mol^-1 K^-1 (SI value)
R_S = 250e-6                   # m
C0 = 1.0                       # mol m^-3, uniform start of the parent
T0, T1, RATE = 1273.0, 1873.0, 50.0 / 3600.0      # K, K, K s^-1
T_RAMP = (T1 - T0) / RATE      # 43200 s
TREF = T1
DP_REF, DD_REF = 1e-13, 3e-14  # m^2 s^-1 at T_REF
LAM_P, LAM_D, BR = 1e-6, 3e-6, 1.0
Q_P, Q_D_CE = 300e3, 400e3     # J mol^-1
TIMES_S = (2e4, 4.32e4, 1e5, 3e5)
NMODE = 400
KN = np.arange(1, NMODE + 1) * math.pi / R_S
A0N = 2 * C0 * R_S * (-1.0) ** np.arange(0, NMODE) / (np.arange(1, NMODE + 1) * math.pi)
NORM_C0 = C0 * math.sqrt(R_S**3 / 3)
GLX, GLW = np.polynomial.legendre.leggauss(16)


def T_H1(t):
    return np.where(np.asarray(t) < T_RAMP, T0 + RATE * np.asarray(t), T1)


def D_arr(Dref, Q, t):
    return Dref * np.exp(-(Q / RG) * (1.0 / T_H1(t) - 1.0 / TREF))


def Theta(Dref, Q, t):
    """int_0^t D(T(s)) ds on history H1 (m^2), closed form (P22.22 type)."""
    th = Q / RG
    t = np.asarray(t, dtype=float)
    tr = np.minimum(t, T_RAMP)
    Tt = T0 + RATE * tr
    F = lambda TT: TT * expn(2, th / TT)
    ramp = Dref * math.exp(th / TREF) / RATE * (F(Tt) - F(T0))
    hold = Dref * math.exp(-th * (1 / T1 - 1 / TREF)) * np.maximum(t - T_RAMP, 0.0)
    return ramp + hold


def graded_nodes(t, breaks=(), K=36):
    pts = [0.0] + sorted(bk for bk in breaks if 0.0 < bk < t) + [t]
    S, Wt = [], []
    for a, c in zip(pts[:-1], pts[1:]):
        L = c - a
        edges = sorted(set([a, c] + [a + L * 2.0**-k for k in range(1, K)] + [c - L * 2.0**-k for k in range(1, K)]))
        for lo, hi in zip(edges[:-1], edges[1:]):
            S.append(0.5 * (hi - lo) * GLX + 0.5 * (hi + lo))
            Wt.append(0.5 * (hi - lo) * GLW)
    return np.concatenate(S), np.concatenate(Wt)


def radau_modes(n_list, DPf, DDf, b, times, breaks):
    """Stiff real-time integration of the 2x2 modal system for modes n (1-based)."""
    out = {}
    for n in n_list:
        k2 = KN[n - 1] ** 2
        f = lambda t, y: [-(LAM_P + k2 * DPf(t)) * y[0], b * LAM_P * y[0] - (LAM_D + k2 * DDf(t)) * y[1]]
        J = lambda t, y: [[-(LAM_P + k2 * DPf(t)), 0.0], [b * LAM_P, -(LAM_D + k2 * DDf(t))]]
        pts = [0.0] + [bk for bk in breaks if bk < max(times)] + [max(times)]
        y = np.array([A0N[n - 1], 0.0])
        vals = {}
        for a, c in zip(pts[:-1], pts[1:]):
            te = [tq for tq in times if a < tq <= c]
            sol = solve_ivp(f, (a, c), y, method="Radau", jac=J, rtol=1e-12, atol=1e-26,
                            t_eval=te if te else None, dense_output=False)
            for j, tq in enumerate(te):
                vals[tq] = sol.y[:, j].copy()
            y = sol.y[:, -1].copy()
        out[n] = vals
    return out


def rel(a, y, n, floor=1e-6):
    """Relative difference, skipped (0) when the reference amplitude is below floor * |a_n(0)|."""
    return 0.0 if abs(y) < floor * abs(A0N[n - 1]) else abs(a - y) / abs(y)


def cascade_sphere(ThP, ThD, b, t, breaks, nmax=NMODE):
    """Daughter modal amplitudes by the two-clock Duhamel formula (P2X.O11.14), O = I."""
    s_, w_ = graded_nodes(t, breaks)
    ThPs, ThDs, ThDt = ThP(s_), ThD(s_), float(ThD(np.array([t]))[0])
    k2 = KN[:nmax, None] ** 2
    ex = -LAM_D * (t - s_)[None, :] - k2 * (ThDt - ThDs)[None, :] - LAM_P * s_[None, :] - k2 * ThPs[None, :]
    return b * LAM_P * A0N[:nmax] * np.sum(w_[None, :] * np.exp(ex), axis=1)


# N1: closed-form ramp integral against quadrature
errN1 = 0.0
for Q in (Q_P, Q_D_CE, 330e3):
    for tq in TIMES_S:
        ref = quad(lambda u: float(D_arr(1.0, Q, u)), 0, min(tq, T_RAMP), epsabs=0, epsrel=1e-13, limit=200)[0]
        ref += float(D_arr(1.0, Q, T_RAMP + 1.0)) * max(tq - T_RAMP, 0.0)
        errN1 = max(errN1, abs(float(Theta(1.0, Q, tq)) / ref - 1))
check("N1 ramp integral of D, closed form (E2) against adaptive quadrature", errN1 < 1e-11, f"max rel diff {errN1:.1e}")

# N2: Bernoulli history satisfying N2 (delta != 0), D_D = 0.3 D_P (case a, operators differ)
DL = LAM_P - LAM_D             # s^-1
A_B, ALPHA0 = 5.0, 0.05        # A = delta/DL, alpha(0)
DELTA = A_B * DL
B_B = 1.0 / ALPHA0 - A_B
alpha_B = lambda t: 1.0 / (A_B + B_B * np.exp(DL * np.asarray(t)))
tau_B = lambda t: (np.asarray(t) - np.log((A_B + B_B * np.exp(DL * np.asarray(t))) / (A_B + B_B)) / DL) / A_B
TIMES_B = (1e5, 3e5, 1e6)
errq = max(abs(float(tau_B(tq)) / quad(lambda u: float(alpha_B(u)), 0, tq, epsrel=1e-14)[0] - 1) for tq in TIMES_B)
rad = radau_modes([1, 2, 3, 5], lambda t: DP_REF * float(alpha_B(t)), lambda t: DD_REF * float(alpha_B(t)), BR, TIMES_B, [])
errN2 = 0.0
for n, vals in rad.items():
    LPn, LDn = KN[n - 1] ** 2 * DP_REF, KN[n - 1] ** 2 * DD_REF
    beta0 = BR * LAM_P / ALPHA0
    for tq, y in vals.items():
        ta = float(tau_B(tq))
        aPc = A0N[n - 1] * math.exp(-LAM_P * tq - LPn * ta)
        aDc = math.exp(-LAM_D * tq) * beta0 * A0N[n - 1] * (math.exp(-(LPn + DELTA) * ta) - math.exp(-LDn * ta)) / (LDn - LPn - DELTA)
        errN2 = max(errN2, rel(aPc, y[0], n), rel(aDc, y[1], n))
check("N2 N1 and N2 hold: joint constant-coefficient form in tau = stiff modal integration (P2X.O11.10)",
      errN2 < 1e-8 and errq < 1e-12, f"max rel diff {errN2:.1e} (modes 1,2,3,5); tau closed form vs quad {errq:.1e}")

# N3: identical operators (D_D = D_P, same Q), linear ramp H1
ThI = lambda t: Theta(DP_REF, Q_P, t)
rad = radau_modes([1, 2, 4], lambda t: float(D_arr(DP_REF, Q_P, t)), lambda t: float(D_arr(DP_REF, Q_P, t)), BR, TIMES_S, [T_RAMP])
errN3 = 0.0
for n, vals in rad.items():
    for tq, y in vals.items():
        e = math.exp(-KN[n - 1] ** 2 * float(ThI(tq)))
        aP_c = A0N[n - 1] * math.exp(-LAM_P * tq) * e
        aD_c = A0N[n - 1] * BR * LAM_P * (math.exp(-LAM_P * tq) - math.exp(-LAM_D * tq)) / (LAM_D - LAM_P) * e
        errN3 = max(errN3, rel(aP_c, y[0], n), rel(aD_c, y[1], n))
check("N3 identical operators on a ramp: exp(M t) form = stiff modal integration (P2X.O11.12)", errN3 < 1e-8,
      f"max rel diff {errN3:.1e}")

# N4: fit ln(alpha) + DL t = ln C + delta tau
def fit_residual(alpha_f, tau_f, tgrid):
    y = np.log(alpha_f(tgrid)) + DL * tgrid
    A = np.vstack([np.ones_like(tgrid), tau_f(tgrid)]).T
    coef, *_ = np.linalg.lstsq(A, y, rcond=None)
    return np.max(np.abs(A @ coef - y)), coef


tg = np.linspace(0, 1e6, 2001)
resB, coefB = fit_residual(alpha_B, tau_B, tg)
tg1 = np.linspace(0, 3e5, 2001)
resR, _ = fit_residual(lambda t: D_arr(1.0, Q_P, t), lambda t: Theta(1.0, Q_P, t), tg1)
check("N4 N2 residual: Bernoulli history fits ln C + delta tau exactly, ramp + hold does not",
      resB < 1e-9 and abs(coefB[1] / DELTA - 1) < 1e-8 and resR > 0.5,
      f"Bernoulli residual {resB:.1e}, fitted delta/delta = {coefB[1] / DELTA:.12f}; ramp residual {resR:.2f} (in ln alpha)")

# N5: counterexample, Q_D != Q_P, b = 0: clocks implied by modes 1 and 2
rad = radau_modes([1, 2], lambda t: float(D_arr(DP_REF, Q_P, t)), lambda t: float(D_arr(DD_REF, Q_D_CE, t)), 0.0, TIMES_S, [T_RAMP])
radD = radau_modes([1, 2], lambda t: float(D_arr(DD_REF, Q_D_CE, t)), lambda t: float(D_arr(DD_REF, Q_D_CE, t)), 0.0, TIMES_S, [T_RAMP])
dk2 = KN[1] ** 2 - KN[0] ** 2
ratios, errimp = [], 0.0
for tq in TIMES_S:
    thP = -(math.log(abs(rad[2][tq][0] / rad[1][tq][0])) - math.log(abs(A0N[1] / A0N[0]))) / dk2
    thD = -(math.log(abs(radD[2][tq][0] / radD[1][tq][0])) - math.log(abs(A0N[1] / A0N[0]))) / dk2
    errimp = max(errimp, abs(thP / float(Theta(DP_REF, Q_P, tq)) - 1), abs(thD / float(Theta(DD_REF, Q_D_CE, tq)) - 1))
    ratios.append(thP / thD)
spread = max(ratios) / min(ratios)
check("N5 counterexample Q_P = 300, Q_D = 400 kJ/mol: implied clock ratio Theta_P/Theta_D is not constant",
      spread > 1.5 and errimp < 1e-6,
      f"Theta_P/Theta_D at t = {', '.join(f'{v:g}' for v in TIMES_S)} s: {', '.join(f'{v:.4f}' for v in ratios)}; "
      f"implied vs closed form {errimp:.1e}")

# N6: cascade (two clocks) against the stiff integration, Q_D != Q_P, b = 1
ThP_ = lambda t: Theta(DP_REF, Q_P, t)
ThD_ = lambda t: Theta(DD_REF, Q_D_CE, t)
rad = radau_modes([1, 2, 3, 5, 8], lambda t: float(D_arr(DP_REF, Q_P, t)), lambda t: float(D_arr(DD_REF, Q_D_CE, t)), BR,
                  TIMES_S, [T_RAMP])
errN6 = 0.0
for tq in TIMES_S:
    casc = cascade_sphere(ThP_, ThD_, BR, tq, [T_RAMP], nmax=8)
    for n in rad:
        errN6 = max(errN6, rel(casc[n - 1], rad[n][tq][1], n))
check("N6 two-clock cascade = stiff modal integration, Q_P != Q_D on ramp + hold (P2X.O11.14)", errN6 < 1e-8,
      f"max rel diff {errN6:.1e} (modes 1,2,3,5,8)")

# N7: equal Q, D_D = 0.3 D_P, ramp + hold: freezing the tau-coupling at its value at the output time
ThPe = lambda t: Theta(DP_REF, Q_P, t)
ThDe = lambda t: Theta(DD_REF, Q_P, t)
errN7 = []
for tq in (1e5, 3e5):
    exact = cascade_sphere(ThPe, ThDe, BR, tq, [T_RAMP], nmax=1)[0]
    ta = float(Theta(1.0, Q_P, tq))
    LPn, LDn = KN[0] ** 2 * DP_REF, KN[0] ** 2 * DD_REF
    beta = BR * LAM_P * math.exp(-DL * tq) / float(D_arr(1.0, Q_P, tq))
    naive = math.exp(-LAM_D * tq) * beta * A0N[0] * (math.exp(-LPn * ta) - math.exp(-LDn * ta)) / (LDn - LPn)
    errN7.append(abs(naive / exact - 1))
check("N7 equal Q but N2 fails: a frozen tau-coupling gives a visible error (illustration)", min(errN7) > 1e-2,
      f"mode-1 daughter rel error {errN7[0]:.2e} at 1e5 s, {errN7[1]:.2e} at 3e5 s")

# =====================================================================================
# Two-layer sphere: cascade with the overlap matrix against finite volumes (synthetic)
# =====================================================================================
R1M, R2M = 175e-6, 390e-6
DPS = np.array([2e-15, 5e-16])     # parent D* at TREF_M, m^2 s^-1
DDS = np.array([1e-15, 4e-15])     # daughter D* at TREF_M
LPM, LDM, BM = 3e-8, 1e-8, 1.0
QPM, QDM = 200e3, 350e3            # shared within each species, different between species
T0M, T1M, TRM = 1300.0, 1500.0, 1e8
TREF_M = T1M
RATE_M = (T1M - T0M) / TRM
TIMES_M = (5e7, 1e8, 2e8)
NM = 80


def T_M(t):
    return np.where(np.asarray(t) < TRM, T0M + RATE_M * np.asarray(t), T1M)


def alpha_M(Q, t):
    return np.exp(-(Q / RG) * (1.0 / T_M(t) - 1.0 / TREF_M))


def tau_M(Q, t):
    th = Q / RG
    t = np.asarray(t, dtype=float)
    tr = np.minimum(t, TRM)
    F = lambda TT: TT * expn(2, th / TT)
    return math.exp(th / TREF_M) / RATE_M * (F(T0M + RATE_M * tr) - F(T0M)) + np.maximum(t - TRM, 0.0)


def modes(diffs):
    ml = Multilayer([R1M, R2M], diffs)
    lam = ml.eigenvalues(NM)
    N, I, Is, _ = ml.mode_integrals(lam, src_layers=[0])
    return ml, lam, N, I, Is


def overlap(mlA, lamA, NA, mlB, lamB):
    kmax = np.maximum(np.sqrt(lamA[-1] / mlA.D), np.sqrt(lamB[-1] / mlB.D))
    r, wq, _ = mlA.grid(2 * kmax)
    UA = np.array([mlA.u_eval(l, r) for l in lamA])
    UB = np.array([mlB.u_eval(l, r) for l in lamB])
    return (UA * wq) @ UB.T / NA[:, None]


mlP, lamP, NP_, IP_, IsP = modes(DPS)
mlD, lamD, ND_, ID_, _ = modes(DDS)
O = overlap(mlD, lamD, ND_, mlP, lamP)
aP0 = 1.0 * IsP / NP_


def cascade_multi(t):
    s_, w_ = graded_nodes(t, [TRM])
    tPs, tDs, tDt = tau_M(QPM, s_), tau_M(QDM, s_), float(tau_M(QDM, np.array([t]))[0])
    Bm = np.exp(-LPM * s_[None, :] - lamP[:, None] * tPs[None, :])           # m x s
    aD = np.zeros(NM)
    for n in range(NM):
        An = np.exp(-LDM * (t - s_) - lamD[n] * (tDt - tDs))                  # s
        J = Bm @ (w_ * An)                                                     # m
        aD[n] = BM * LPM * np.sum(O[n, :] * aP0 * J)
    MD = 4 * math.pi * np.sum(ID_ * aD)
    MP = 4 * math.pi * np.sum(IP_ * aP0 * np.exp(-LPM * t - lamP * float(tau_M(QPM, np.array([t]))[0])))
    return MP, MD


def fv_semi(ncell, DPl_star, DDl_star, alphaP_l, alphaD_l, times, breaks, need_C=False):
    """Two-species semi-discrete FV with D_{i,k}(t) = alpha_{i,k}(t) D*_{i,k}; Radau in time.

    alphaP_l, alphaD_l: functions t -> array of per-layer scaling factors.  Face conductances are
    recomputed from the current cell diffusivities (harmonic resistance, absorbing surface)."""
    fP = SphereFV([R1M, R2M], DPl_star, [ncell, ncell])
    nC = fP.rc.size
    V, lay, rf, rc = fP.V, fP.layer, fP.rf, fP.rc
    A = 4.0 * np.pi * rf ** 2
    dm, dp = rf[1:nC] - rc[:-1], rc[1:] - rf[1:nC]          # centroid-to-face distances (m)
    dR = rf[nC] - rc[-1]

    def cond(Dstar_l, al):
        Dc = np.asarray(Dstar_l)[lay] * np.asarray(al)[lay]
        G = np.zeros(nC + 1)
        G[1:nC] = A[1:nC] / (dm / Dc[:-1] + dp / Dc[1:])
        G[nC] = A[nC] * Dc[-1] / dR
        return G

    def Kapply(G, C):
        out = (G[:-1] + G[1:]) * C
        out[:-1] -= G[1:-1] * C[1:]
        out[1:] -= G[1:-1] * C[:-1]
        return out

    def Kmat(G):
        return sps.diags([G[:-1] + G[1:], -G[1:-1], -G[1:-1]], [0, 1, -1], format="csr")

    Vinv = sps.diags(1.0 / V)
    I_ = sps.identity(nC, format="csr")

    def rhs(t, y):
        GP, GD = cond(DPl_star, alphaP_l(t)), cond(DDl_star, alphaD_l(t))
        CP, CD = y[:nC], y[nC:]
        return np.concatenate([-Kapply(GP, CP) / V - LPM * CP,
                               -Kapply(GD, CD) / V - LDM * CD + BM * LPM * CP])

    def jac(t, y):
        GP, GD = cond(DPl_star, alphaP_l(t)), cond(DDl_star, alphaD_l(t))
        return sps.bmat([[-Vinv @ Kmat(GP) - LPM * I_, None], [BM * LPM * I_, -Vinv @ Kmat(GD) - LDM * I_]],
                        format="csc")

    y = np.concatenate([np.where(lay == 0, 1.0, 0.0), np.zeros(nC)])
    pts = [0.0] + [bk for bk in breaks if bk < max(times)] + [max(times)]
    out = {}
    for a, c in zip(pts[:-1], pts[1:]):
        te = [tq for tq in times if a < tq <= c]
        sol = solve_ivp(rhs, (a, c), y, method="Radau", jac=jac, rtol=1e-9, atol=1e-14, t_eval=te if te else None)
        for j, tq in enumerate(te):
            CP, CD = sol.y[:nC, j], sol.y[nC:, j]
            out[tq] = (float(V @ CP), float(V @ CD), CP.copy(), CD.copy()) if need_C else (float(V @ CP), float(V @ CD))
        y = sol.y[:, -1].copy()
    return out, V


ref = {tq: cascade_multi(tq) for tq in TIMES_M}
errsD, errsP = [], []
for nc in (40, 80, 160):
    o, _ = fv_semi(nc, DPS, DDS, lambda t: np.full(2, alpha_M(QPM, t)), lambda t: np.full(2, alpha_M(QDM, t)),
                   TIMES_M, [TRM])
    errsD.append(max(abs(o[tq][1] / ref[tq][1] - 1) for tq in TIMES_M))
    errsP.append(max(abs(o[tq][0] / ref[tq][0] - 1) for tq in TIMES_M))
    print(f"      FV {nc:>3} cells/layer: parent rel diff {errsP[-1]:.2e}, daughter rel diff {errsD[-1]:.2e}", flush=True)
pD = math.log2(errsD[-2] / errsD[-1])
check("N8 two-layer cascade with overlap matrix O = two-species FV, Q_P = 200, Q_D = 350 kJ/mol (P2X.O11.15)",
      errsD[-1] < 1e-3 and errsP[-1] < 1e-3 and 1.8 < pD < 2.2,
      f"finest daughter rel diff {errsD[-1]:.1e}, parent {errsP[-1]:.1e}, observed order {pD:.2f}; "
      f"M_D/M_P0 at 2e8 s = {ref[2e8][1] / (4 * math.pi * R1M**3 / 3):.5f}")
offO = np.max(np.abs(O - np.diag(np.diag(O))))
check("N8b the overlap matrix is full (the daughter needs all parent modes)", offO > 0.05, f"max off-diagonal {offO:.3f}")

# =====================================================================================
# Error bounds (Theorem 3)
# =====================================================================================
def h_rho(r):
    return (1 - r) ** 2 / r


def theta_sum(a):
    """sum_{n>=1} exp(-a n^2) for a > 0, without truncation error: direct sum for a >= 1,
    Poisson (Jacobi) transform sqrt(pi/a)(1 + 2 sum exp(-pi^2 n^2/a)) for a < 1."""
    a = np.atleast_1d(np.asarray(a, dtype=float))
    out = np.empty_like(a)
    n = np.arange(1, 60)
    big = a >= 1.0
    out[big] = np.sum(np.exp(-a[big, None] * n[None, :] ** 2), axis=1)
    sm = ~big
    out[sm] = 0.5 * (np.sqrt(math.pi / a[sm]) * (1 + 2 * np.sum(np.exp(-math.pi**2 * n[None, :] ** 2 / a[sm, None]), axis=1)) - 1)
    return out


# N9: single sphere.  True Q_P = 300, Q_D = 330; surrogate shared Qbar = 315 for both species
QPt, QDt, QBAR = 300e3, 330e3, 315e3
rho0P = math.exp(-((QBAR - QPt) / RG) * (1 / T0 - 1 / TREF))
rho0D = math.exp(-((QBAR - QDt) / RG) * (1 / T0 - 1 / TREF))
mP, mD = h_rho(rho0P), h_rho(rho0D)
ThPt, ThDt_ = (lambda t: Theta(DP_REF, QPt, t)), (lambda t: Theta(DD_REF, QDt, t))
ThPs, ThDs = (lambda t: Theta(DP_REF, QBAR, t)), (lambda t: Theta(DD_REF, QBAR, t))
okN9, lines = True, []
for tq in TIMES_S:
    aPt = A0N * np.exp(-LAM_P * tq - KN**2 * float(ThPt(tq)))
    aPs = A0N * np.exp(-LAM_P * tq - KN**2 * float(ThPs(tq)))
    EP = math.sqrt(R_S / 2 * np.sum((aPt - aPs) ** 2))
    aDt = cascade_sphere(ThPt, ThDt_, BR, tq, [T_RAMP])
    aDs = cascade_sphere(ThPs, ThDs, BR, tq, [T_RAMP])
    ED = math.sqrt(R_S / 2 * np.sum((aDt - aDs) ** 2))
    bP = 0.5 * math.sqrt(mP) * math.exp(-LAM_P * tq) * NORM_C0
    bD = 0.5 * BR * (1 - math.exp(-LAM_P * tq)) * (math.sqrt(mP) + math.sqrt(mD)) * NORM_C0
    # a posteriori parent bound sqrt(1/2 int eta), eta = (D - Dt)^2/D * int Dt-free |d_r ct|^2 r^2 dr.
    # Uniform start: a_n(0)^2 k_n^2 N_n = 2 c0^2 R for every n, so the gradient norm is the full
    # (untruncated) theta-type sum 2 c0^2 R exp(-2 lambda s) sum_{n>=1} exp(-2 pi^2 n^2 Theta/R^2).
    post = []
    for Kg in (36, 46):
        s_, w_ = graded_nodes(tq, [T_RAMP], K=Kg)
        Dt_s, Ds_s = D_arr(DP_REF, QPt, s_), D_arr(DP_REF, QBAR, s_)
        # below 1 s the closed form loses digits to cancellation; the midpoint rule s D(s/2) is then
        # accurate to about 1e-9 relative (curvature of ln D on the ramp is theta r / T0^2 = 3e-4 s^-1)
        Th_s = np.where(s_ < 1.0, s_ * D_arr(DP_REF, QBAR, 0.5 * s_), ThPs(s_))
        grad2 = 2 * C0**2 * R_S * np.exp(-2 * LAM_P * s_) * theta_sum(2 * math.pi**2 * Th_s / R_S**2)
        eta = (Dt_s - Ds_s) ** 2 / Dt_s * grad2
        post.append(math.sqrt(0.5 * np.sum(w_ * eta)))
    okN9 = okN9 and EP <= bP and ED <= bD and EP <= post[1] * (1 + 1e-9) and abs(post[0] / post[1] - 1) < 1e-4
    lines.append(f"t={tq:g}: E_P/|c0| {EP / NORM_C0:.2e} <= post {post[1] / NORM_C0:.2e} <= prior {bP / NORM_C0:.2e}; "
                 f"E_D/|c0| {ED / NORM_C0:.2e} <= prior {bD / NORM_C0:.2e}")
tsx = np.array([0.3, 0.999, 1.0, 2.0])
tsd = np.array([np.sum(np.exp(-v * np.arange(1, 4000) ** 2)) for v in tsx])
check("N9a untruncated mode sum (Poisson transform) = long direct sum", np.max(np.abs(theta_sum(tsx) / tsd - 1)) < 1e-13,
      f"max rel diff {np.max(np.abs(theta_sum(tsx) / tsd - 1)):.1e}")
check("N9 single sphere: L2 errors of a shared-Qbar surrogate obey the a priori and a posteriori bounds (P2X.O11.18-20)",
      okN9, f"m_P = {mP:.4f}, m_D = {mD:.4f}")
for ln in lines:
    print("      " + ln)

# N10: two-layer FV, layer-specific true Q (E1 fails for each species), surrogate shared per species
QP_L, QD_L = np.array([200e3, 230e3]), np.array([330e3, 370e3])
QP_BAR, QD_BAR = 215e3, 350e3
aPl_true = lambda t: np.exp(-(QP_L / RG) * (1.0 / float(T_M(t)) - 1.0 / TREF_M))
aDl_true = lambda t: np.exp(-(QD_L / RG) * (1.0 / float(T_M(t)) - 1.0 / TREF_M))
aPl_sur = lambda t: np.full(2, math.exp(-(QP_BAR / RG) * (1.0 / float(T_M(t)) - 1.0 / TREF_M)))
aDl_sur = lambda t: np.full(2, math.exp(-(QD_BAR / RG) * (1.0 / float(T_M(t)) - 1.0 / TREF_M)))
x0 = 1 / T0M - 1 / TREF_M
mPf = max(h_rho(math.exp(-((QP_BAR - q) / RG) * x0)) for q in QP_L)
mDf = max(h_rho(math.exp(-((QD_BAR - q) / RG) * x0)) for q in QD_L)
ot, V = fv_semi(40, DPS, DDS, aPl_true, aDl_true, TIMES_M, [TRM], need_C=True)
os_, _ = fv_semi(40, DPS, DDS, aPl_sur, aDl_sur, TIMES_M, [TRM], need_C=True)
nC = V.size
C0v = np.where(np.arange(nC) < 40, 1.0, 0.0)
nrm0 = math.sqrt(np.sum(V * C0v**2))
okN10, lines = True, []
for tq in TIMES_M:
    EP = math.sqrt(np.sum(V * (ot[tq][2] - os_[tq][2]) ** 2))
    ED = math.sqrt(np.sum(V * (ot[tq][3] - os_[tq][3]) ** 2))
    bP = 0.5 * math.sqrt(mPf) * math.exp(-LPM * tq) * nrm0
    bD = 0.5 * BM * (1 - math.exp(-LPM * tq)) * (math.sqrt(mPf) + math.sqrt(mDf)) * nrm0
    dMD = abs(ot[tq][1] - os_[tq][1]) / (4 * math.pi * R1M**3 / 3)
    okN10 = okN10 and EP <= bP and ED <= bD
    lines.append(f"t={tq:g}: E_P/|C0| {EP / nrm0:.2e} <= {bP / nrm0:.2e}; E_D/|C0| {ED / nrm0:.2e} <= {bD / nrm0:.2e}; "
                 f"|dM_D|/M_P0 {dMD:.2e}")
check("N10 two-layer FV, layer-specific Q: discrete a priori bounds hold for the shared-Q surrogate (P2X.O11.21)",
      okN10, f"m_P = {mPf:.4f}, m_D = {mDf:.4f}")
for ln in lines:
    print("      " + ln)

# N11: ratio range in which the a priori factor (1/2) sqrt(h(rho)) is below 0.1 and below 1
from scipy.optimize import brentq as _bq
g = lambda r, lev: 0.5 * math.sqrt(h_rho(r)) - lev
r01 = (_bq(g, 1e-6, 1.0, args=(0.1,)), _bq(g, 1.0, 1e6, args=(0.1,)))
r1 = (_bq(g, 1e-9, 1.0, args=(1.0,)), _bq(g, 1.0, 1e9, args=(1.0,)))
check("N11 a priori factor (1/2)|1-rho|/sqrt(rho) < 0.1 and < 1: ratio intervals",
      abs(r01[0] * r01[1] - 1) < 1e-9 and abs(r1[0] * r1[1] - 1) < 1e-9,
      f"< 0.1 for rho in ({r01[0]:.4f}, {r01[1]:.4f}); < 1 for rho in ({r1[0]:.4f}, {r1[1]:.4f})")

# N12: two layers whose true diffusivity ratio spans a factor q over the history.  Any surrogate with
# a fixed layer ratio has rho_1/rho_2 spanning [a, q a]; at each instant max(h(rho_1), h(rho_2)) >=
# h(sqrt(rho_1/rho_2)), so m >= h(q^(1/4)).  q = 6.3 is the TECDOC-978 Cs SiC/kernel spread quoted
# in Section 5.2 of the pseudocode (1250 to 1800 C).  Brute-force scan over the offset a confirms it.
q = 6.3
fac = 0.5 * math.sqrt(h_rho(q ** 0.25))
best = np.inf
for a_ in np.exp(np.linspace(-3, 3, 6001)):
    path = a_ * np.exp(np.linspace(0, math.log(q), 201))
    best = min(best, np.max(h_rho(np.sqrt(np.maximum(path, 1 / path)))))
check("N12 shared-scaling surrogate, layer ratio spread q = 6.3: a priori factor >= (1/2) sqrt(h(q^(1/4)))",
      abs(0.5 * math.sqrt(best) / fac - 1) < 1e-3, f"factor >= {fac:.4f} (scan minimum {0.5 * math.sqrt(best):.4f})")

print(f"\nSUMMARY: {sum(RESULTS)}/{len(RESULTS)} PASS")
if all(RESULTS):
    print("ALL PASS")
sys.exit(0 if all(RESULTS) else 1)
