"""Section 5 (Phase 2.2) SymPy proof of the effective-time transform and the decay
factorisation, equations (P22.24)-(P22.38), plus the Galerkin reduction (P22.50) and the
backward-Euler time-level error (P22.18).

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec5_effective_time.py
The script writes no files. Every check prints a PASS or FAIL line.
"""
import math

import numpy as np
import sympy as sp

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


r, t, s = sp.symbols("r t s", positive=True)
lam, Lam, Dst, D1s, D2s, R = sp.symbols("lambda Lambda D_star D1_star D2_star R", positive=True)
tau = sp.Function("tau")(t)            # effective time, tau(0) = 0
alpha = sp.diff(tau, t)                # alpha(t) = d tau / dt  (P22.26)
X = sp.Symbol("X")                     # placeholder for the second argument of chat


def Lstar(expr, D):
    """Spherical operator with layer constant D*: (1/r^2) d/dr (r^2 D* d expr/dr)."""
    return sp.diff(r**2 * D * sp.diff(expr, r), r) / r**2


# ---------------------------------------------------------------------------
# 1. Main transform (P22.27)-(P22.30), arbitrary chat and arbitrary tau(t)
# ---------------------------------------------------------------------------
# chat(r, X) is an arbitrary smooth function. Define the tau-space source so that chat
# solves the tau-space problem exactly:  Shat = d chat/dX - L* chat.
chat = sp.Function("chat")
chat_rX = chat(r, X)
Shat_rX = sp.diff(chat_rX, X) - Lstar(chat_rX, Dst)

# Real-time field and source built from the transform rules
c = sp.exp(-lam * t) * chat_rX.subs(X, tau)                       # (P22.27)
S = alpha * sp.exp(-lam * t) * Shat_rX.subs(X, tau)               # (P22.30) inverted
D_real = alpha * Dst                                              # (P22.24)
residual = sp.diff(c, t) - Lstar(c, D_real) + lam * c - S          # real-time PDE (P22.11)
check("real-time PDE residual = 0 for arbitrary chat, tau (P22.29)",
      sp.simplify(sp.expand(residual.doit())) == 0)

# Explicit sanity case (no undefined functions): chat = exp(-X) r^2 + sin(r) X^2
chat_e = sp.exp(-X) * r**2 + sp.sin(r) * X**2
tau_e = t + t**3 / 3                       # alpha = 1 + t^2
alpha_e = sp.diff(tau_e, t)
Shat_e = sp.diff(chat_e, X) - Lstar(chat_e, Dst)
c_e = sp.exp(-lam * t) * chat_e.subs(X, tau_e)
S_e = alpha_e * sp.exp(-lam * t) * Shat_e.subs(X, tau_e)
res_e = sp.diff(c_e, t) - Lstar(c_e, alpha_e * Dst) + lam * c_e - S_e
check("explicit case chat = e^-X r^2 + X^2 sin r, tau = t + t^3/3", sp.simplify(res_e) == 0)

# Single mode of a homogeneous sphere: Gelbard (2003) Eq. (II.16)-(II.18) structure
n = sp.symbols("n", positive=True, integer=True)
mode = sp.exp(-lam * t) * sp.sin(n * sp.pi * r / R) / r * sp.exp(-n**2 * sp.pi**2 * Dst * tau / R**2)
res_mode = sp.diff(mode, t) - Lstar(mode, alpha * Dst) + lam * mode
check("single-sphere mode e^{-lam t} sin(n pi r/R)/r exp(-n^2 pi^2 D* tau/R^2) solves PDE",
      sp.simplify(res_mode.doit()) == 0)

# ---------------------------------------------------------------------------
# 2. Interface, boundary and initial conditions (P22.31)
# ---------------------------------------------------------------------------
ca = sp.Function("chat_a")(r, X)       # layer i
cb = sp.Function("chat_b")(r, X)       # layer i+1
flux_real = (alpha * D1s * sp.diff(sp.exp(-lam * t) * ca, r)
             - alpha * D2s * sp.diff(sp.exp(-lam * t) * cb, r))
flux_hat = D1s * sp.diff(ca, r) - D2s * sp.diff(cb, r)
check("real flux jump = alpha e^{-lam t} x (tau-space flux jump)",
      sp.simplify(flux_real - alpha * sp.exp(-lam * t) * flux_hat) == 0)
check("continuity: c_a - c_b = e^{-lam t}(chat_a - chat_b)",
      sp.simplify(sp.exp(-lam * t) * ca - sp.exp(-lam * t) * cb - sp.exp(-lam * t) * (ca - cb)) == 0)
check("initial condition: c(r,0) = chat(r,0) because tau(0) = 0 and e^0 = 1",
      sp.simplify((sp.exp(-lam * t) * chat(r, X)).subs(t, 0).subs(X, 0) - chat(r, 0)) == 0)

# Robin surface condition -D5 dc/dr = h c (P22.38)
h = sp.Function("h")(t)
cR = sp.Function("chat_R")(r, X)
robin_real = -alpha * Dst * sp.diff(sp.exp(-lam * t) * cR, r) - h * sp.exp(-lam * t) * cR
robin_hat = -Dst * sp.diff(cR, r) - (h / alpha) * cR
check("Robin: real condition = alpha e^{-lam t} x (tau condition with h/alpha) (P22.38)",
      sp.simplify(robin_real - alpha * sp.exp(-lam * t) * robin_hat) == 0)

# ---------------------------------------------------------------------------
# 3. Without the decay factor, decay becomes lambda/alpha in tau (P22.32)
# ---------------------------------------------------------------------------
ct = sp.Function("ctil")
ct_rX = ct(r, X)
# Suppose ctil solves d ctil/dX = L* ctil - (lam/alpha(t(X))) ctil + S/alpha. With
# c = ctil(r, tau(t)) the chain rule gives dc/dt = alpha d ctil/dX; check this is the PDE.
lam_over_alpha = lam / alpha
c2 = ct_rX.subs(X, tau)
# d c2/dt = alpha * (d ctil/dX at X=tau); substituting the tau-equation must reproduce the PDE
dctil_dX = (Lstar(ct_rX, Dst)).subs(X, tau) - lam_over_alpha * c2 + sp.Function("Sreal")(r, t) / alpha
res2b = alpha * dctil_dX - (Lstar(c2, alpha * Dst) - lam * c2 + sp.Function("Sreal")(r, t))
check("unfactored form: alpha (L* - lam/alpha) + S = alpha L* - lam + S (P22.32)",
      sp.simplify(res2b.doit()) == 0)

# ---------------------------------------------------------------------------
# 4. Modal solution with source (P22.36) and the constant-coefficient limit (P22.37)
# ---------------------------------------------------------------------------
a0 = sp.symbols("a0")
sn = sp.Function("s_n")
tau_s = tau.subs(t, s)
a_t = sp.exp(-lam * t - Lam * tau) * (a0 + sp.Integral(sp.exp(lam * s + Lam * tau_s) * sn(s), (s, 0, t)))
res_a = sp.diff(a_t, t) + (lam + Lam * alpha) * a_t - sn(t)
check("a_n(t) of (P22.36) solves da/dt = -(lam + alpha Lambda*) a + s_n",
      sp.simplify(res_a.doit()) == 0)
a_at0 = a_t.subs(t, 0).doit().subs(sp.Function("tau")(0), 0)    # tau(0) = 0 by definition
check("a_n(0) = a0", sp.simplify(a_at0 - a0) == 0)

al0, s0 = sp.symbols("alpha0 s0", positive=True)
a_const = sp.exp(-(lam + al0 * Lam) * t) * sp.integrate(sp.exp((lam + al0 * Lam) * s) * s0, (s, 0, t))
check("constant alpha and source: a_n = s0 (1 - e^{-(lam+alpha Lambda) t})/(lam + alpha Lambda) (P22.37)",
      sp.simplify(a_const - s0 * (1 - sp.exp(-(lam + al0 * Lam) * t)) / (lam + al0 * Lam)) == 0)

# ---------------------------------------------------------------------------
# 5. Inventories (P22.33)-(P22.35): release and decay bookkeeping
# ---------------------------------------------------------------------------
M0 = sp.symbols("M0", positive=True)
Mhat = sp.Function("Mhat")             # tau-space inventory, Mhat + Mhat_rel = M0
Mrel_hat = lambda x: M0 - Mhat(x)
M_t = sp.exp(-lam * t) * Mhat(tau)
J_t = alpha * sp.exp(-lam * t) * sp.diff(Mrel_hat(X), X).subs(X, tau)     # (P22.34)
Mrel_claim = sp.exp(-lam * t) * Mrel_hat(tau) + lam * sp.Integral(sp.exp(-lam * s) * Mrel_hat(tau_s), (s, 0, t))
check("d/dt of (P22.35) equals J_R(t) of (P22.34)",
      sp.simplify((sp.diff(Mrel_claim, t) - J_t).doit()) == 0)
Mdec = lam * sp.Integral(sp.exp(-lam * s) * Mhat(tau_s), (s, 0, t))
# d/dt of the sum vanishes identically and at t = 0 the sum is Mhat(0) = M0
dsum = sp.simplify(sp.diff(M_t + Mrel_claim + Mdec, t).doit())
at0 = (M_t + Mrel_claim + Mdec).subs(t, 0).doit().subs(sp.Function("tau")(0), 0)
check("closure M + M_rel + M_dec = M0: d/dt(sum) = 0 and sum(0) = M0",
      dsum == 0 and sp.simplify(at0 - M0) == 0)

# ---------------------------------------------------------------------------
# 6. Galerkin stiffness on the eigenbasis of a homogeneous sphere is diagonal (P22.50)
# ---------------------------------------------------------------------------
ok = True
for m_ in range(1, 4):
    for n_ in range(1, 4):
        psi_m = sp.sin(m_ * sp.pi * r / R) / r
        psi_n = sp.sin(n_ * sp.pi * r / R) / r
        Kmn = sp.integrate(sp.simplify(r**2 * Dst * sp.diff(psi_m, r) * sp.diff(psi_n, r)), (r, 0, R))
        Nn = sp.integrate(sp.simplify(r**2 * psi_n**2), (r, 0, R))
        target = Dst * (n_ * sp.pi / R) ** 2 * Nn if m_ == n_ else 0
        ok &= sp.simplify(Kmn - target) == 0
check("int r^2 D* psi_m' psi_n' dr = Lambda_n* N_n delta_mn for m,n = 1..3", ok)

# ---------------------------------------------------------------------------
# 7. Backward-Euler time level (P22.18): tau increment vs alpha(t_{n+1}) dt
# ---------------------------------------------------------------------------
h_, a0s, a1s, a2s = sp.symbols("h alpha0 alpha1 alpha2")
u = sp.symbols("u")
# Taylor expansion of alpha about t_{n+1}: alpha(t_{n+1} - u) = a0 - a1 u + a2 u^2/2
dtau = sp.integrate(a0s - a1s * u + a2s * u**2 / 2, (u, 0, h_))
check("Delta tau - alpha(t_{n+1}) dt = -alpha' dt^2/2 + alpha'' dt^3/6",
      sp.simplify(dtau - a0s * h_ - (-a1s * h_**2 / 2 + a2s * h_**3 / 6)) == 0)

# Numerical confirmation with an Arrhenius alpha along a 50 K/h ramp (Q = 300 kJ/mol)
Rg = 6.02214076e23 * 1.380649e-23
aQ = 300e3 / Rg
Tref = 1873.15
T_of = lambda tt: 1523.15 + 50.0 / 3600.0 * tt
al = lambda tt: math.exp(-aQ * (1.0 / T_of(tt) - 1.0 / Tref))
dal = lambda tt: al(tt) * aQ / T_of(tt) ** 2 * 50.0 / 3600.0
t1 = 5 * 3600.0
xg, wg = np.polynomial.legendre.leggauss(12)
ratios = []
for dt in (400.0, 200.0, 100.0):
    mid, half = t1 - dt / 2, dt / 2
    dtau_n = half * sum(w * al(mid + half * x) for x, w in zip(xg, wg))
    ratios.append((dtau_n - al(t1) * dt) / (-dal(t1) * dt**2 / 2))
check("numerical: (Delta tau - alpha^{n+1} dt)/(-alpha' dt^2/2) -> 1", abs(ratios[-1] - 1) < 1e-2,
      "ratios " + ", ".join(f"{q:.5f}" for q in ratios))

print(f"\nSUMMARY {sum(RESULTS)}/{len(RESULTS)} checks passed")
