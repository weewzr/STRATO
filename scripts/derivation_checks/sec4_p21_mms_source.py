"""Phase 2.1 (section 4): method of manufactured solutions (MMS) in a two-layer sphere.

Manufactured solution (P21.88), layers 1 (0 < r < R1) and 2 (R1 < r < R):
    P(r)    = cos(pi r / (2 R))
    psi_1   = D_ref [ (P - P(R1)) / D_1 + P(R1) / D_2 ]
    psi_2   = D_ref P / D_2
    c_i     = c_ref g(t) psi_i(r),   g(t) = 1 + t / tau
It satisfies c'(0) = 0, continuity of c and of D dc/dr at R1, and c(R) = 0 by construction.
Source (P21.89): S_i = dc/dt - r^-2 (r^2 D_i c')' + lam c.

Checks:
  M1. conditions (centre, continuity, flux continuity, surface) exactly (SymPy).
  M2. the diffusion part of S is the same expression in both layers; S is continuous at R1.
  M3. S >= 0 on a fine grid for the chosen data (so positivity tests remain applicable).
  M4. exact cell average of S from a closed-form antiderivative (P21.90) versus quadrature.
  M5. FV steady variant (g = 1, dc/dt = 0): observed spatial order ~ 2.
  M6. FV transient variant with Backward Euler and exact step-averaged source:
      observed temporal order ~ 1 on a fine mesh.

Run: .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec4_p21_mms_source.py
Writes nothing to disk. Prints PASS/FAIL lines.
"""
import math

import numpy as np
import sympy as sp
from scipy.integrate import quad

ok_all = True


def report(name, ok, detail=""):
    global ok_all
    ok_all &= bool(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


r, t = sp.symbols("r t", positive=True)
R1, R, D1, D2, Dref, cref, tau, lam = sp.symbols("R_1 R D_1 D_2 D_ref c_ref tau lambda", positive=True)
P = sp.cos(sp.pi * r / (2 * R))
P1 = P.subs(r, R1)
g = 1 + t / tau
psi1 = Dref * ((P - P1) / D1 + P1 / D2)
psi2 = Dref * P / D2
c1, c2 = cref * g * psi1, cref * g * psi2


def L_op(c, Di):
    return sp.diff(r**2 * Di * sp.diff(c, r), r) / r**2


# M1 conditions
m1 = [
    sp.simplify(sp.diff(c1, r).subs(r, 0)),
    sp.simplify((c1 - c2).subs(r, R1)),
    sp.simplify((D1 * sp.diff(c1, r) - D2 * sp.diff(c2, r)).subs(r, R1)),
    sp.simplify(c2.subs(r, R)),
]
report("M1 centre, continuity, flux continuity, surface (P21.88)", all(x == 0 for x in m1), str(m1))

# M2 source
S1 = sp.simplify(sp.diff(c1, t) - L_op(c1, D1) + lam * c1)
S2 = sp.simplify(sp.diff(c2, t) - L_op(c2, D2) + lam * c2)
diff1 = sp.simplify(-L_op(c1, D1) / (cref * g))
diff2 = sp.simplify(-L_op(c2, D2) / (cref * g))
report("M2a diffusion part identical in both layers", sp.simplify(diff1 - diff2) == 0, f"-L c / (c_ref g) = {sp.simplify(diff2)}")
report("M2b S continuous at R1", sp.simplify((S1 - S2).subs(r, R1)) == 0)
print("      S_i = c_ref [ psi_i / tau + g(t) * D_ref * Q(r) + lam g(t) psi_i ],  Q(r) =", sp.simplify(diff2 / Dref))

# numerical data (synthetic): kernel + buffer of the TECDOC-1674 Table 10.2 geometry
vals = {R1: 250e-6, R: 350e-6, D1: 4e-14, D2: 1e-14, Dref: 1e-14, cref: 1.0, tau: 1e7, lam: 1e-7}
fS1 = sp.lambdify((r, t), S1.subs(vals), "math")
fS2 = sp.lambdify((r, t), S2.subs(vals), "math")
fc1 = sp.lambdify((r, t), c1.subs(vals), "math")
fc2 = sp.lambdify((r, t), c2.subs(vals), "math")
grid = np.linspace(1e-9, 350e-6, 4001)
Smin = min(min(fS1(x, tt) if x <= 250e-6 else fS2(x, tt) for x in grid) for tt in (0.0, 1e7))
report("M3 S >= 0 everywhere for the chosen data", Smin >= 0, f"min S = {Smin:.3e} mol m^-3 s^-1")

# M4 exact cell average via antiderivative of 4 pi r^2 S
a_, b_ = sp.symbols("a b", positive=True)
F1 = sp.integrate(4 * sp.pi * r**2 * S1, r)
F2 = sp.integrate(4 * sp.pi * r**2 * S2, r)
report("M4a antiderivative derivative check", sp.simplify(sp.diff(F1, r) - 4 * sp.pi * r**2 * S1) == 0
       and sp.simplify(sp.diff(F2, r) - 4 * sp.pi * r**2 * S2) == 0)
fF1 = sp.lambdify((r, t), F1.subs(vals), "math")
fF2 = sp.lambdify((r, t), F2.subs(vals), "math")
Fc1 = sp.lambdify((r, t), sp.integrate(4 * sp.pi * r**2 * c1, r).subs(vals), "math")
Fc2 = sp.lambdify((r, t), sp.integrate(4 * sp.pi * r**2 * c2, r).subs(vals), "math")
aa, bb, tt0 = 120e-6, 131e-6, 3e6
exact = fF1(bb, tt0) - fF1(aa, tt0)
num = quad(lambda x: 4 * math.pi * x * x * fS1(x, tt0), aa, bb, epsabs=0, epsrel=1e-13)[0]
report("M4b cell integral of S: antiderivative = quadrature (P21.90)", abs(exact - num) <= 1e-12 * abs(num), f"rel {abs(exact-num)/abs(num):.1e}")


# ---------------------------------------------------------------- FV machinery (same algebra as v1.0)
def build_mesh(radii, Ds, ncells):
    left, right, D, mat = [], [], [], []
    inner = 0.0
    for m, (Ro, Dm, n) in enumerate(zip(radii, Ds, ncells)):
        w = (Ro - inner) / n
        for k in range(n):
            left.append(inner + k * w)
            right.append(Ro if k + 1 == n else inner + (k + 1) * w)
            D.append(Dm)
            mat.append(m)
        inner = Ro
    left, right, D = np.array(left), np.array(right), np.array(D)
    V = 4 * math.pi / 3 * (right**3 - left**3)
    cen = np.where(left == 0, 0.75 * right, 0.75 * (right**4 - left**4) / np.where(right**3 - left**3 == 0, 1, right**3 - left**3))
    G = np.empty(len(V) - 1)
    for j in range(len(V) - 1):
        f = right[j]
        G[j] = 4 * math.pi * f * f / ((f - cen[j]) / D[j] + (cen[j + 1] - f) / D[j + 1])
    GR = 4 * math.pi * radii[-1]**2 * D[-1] / (radii[-1] - cen[-1])
    return dict(left=left, right=right, V=V, cen=cen, G=G, GR=GR, mat=np.array(mat))


def solve_sf(v, gw, ge, rhs):
    """Subtraction-free tridiagonal elimination (v1.0 algorithm, P21.61)."""
    n = len(v)
    den, dp = np.empty(n), np.empty(n)
    e_prev = 0.0
    for i in range(n):
        e = v[i] if i == 0 else v[i] + gw[i] * e_prev / (e_prev + gw[i])
        den[i] = e + ge[i]
        dp[i] = (rhs[i] + (gw[i] * dp[i - 1] if i else 0.0)) / den[i]
        e_prev = e
    x = np.empty(n)
    x[-1] = dp[-1]
    for i in range(n - 2, -1, -1):
        x[i] = dp[i] + ge[i] / den[i] * x[i + 1]
    return x


def couplings(m):
    n = len(m["V"])
    gw = np.concatenate(([0.0], m["G"]))
    ge = np.concatenate((m["G"], [m["GR"]]))
    return gw, ge


def cell_avg(m, Fa, Fb, tt):
    """Exact cell averages from antiderivatives Fa (layer 1) and Fb (layer 2)."""
    out = np.empty(len(m["V"]))
    for j, (a, b, mt) in enumerate(zip(m["left"], m["right"], m["mat"])):
        F = Fa if mt == 0 else Fb
        out[j] = (F(b, tt) - F(a, tt)) / m["V"][j]
    return out


radii = [250e-6, 350e-6]
Ds = [4e-14, 1e-14]
lamv = 1e-7

# M5 steady variant: g = 1 and dc/dt = 0, so S_ss = c_ref [D_ref Q + lam psi]; reuse S at t = 0 minus psi/tau
S1ss = (S1 - cref * psi1 / tau).subs(t, 0)
S2ss = (S2 - cref * psi2 / tau).subs(t, 0)
fFs1 = sp.lambdify(r, sp.integrate(4 * sp.pi * r**2 * S1ss, r).subs(vals), "math")
fFs2 = sp.lambdify(r, sp.integrate(4 * sp.pi * r**2 * S2ss, r).subs(vals), "math")
errs = []
hs = []
for N in (10, 20, 40, 80, 160):
    m = build_mesh(radii, Ds, [N, N])
    gw, ge = couplings(m)
    Sbar = cell_avg(m, lambda b, _t: fFs1(b), lambda b, _t: fFs2(b), 0.0)
    C = solve_sf(lamv * m["V"], gw, ge, m["V"] * Sbar)       # (K + lam V) C = V Sbar
    cex = cell_avg(m, Fc1, Fc2, 0.0)                         # exact cell averages at g = 1 (t = 0)
    e = math.sqrt(np.sum(m["V"] * (C - cex)**2) / np.sum(m["V"] * cex**2))
    errs.append(e)
    hs.append(N)
orders = [math.log(errs[i] / errs[i + 1], 2) for i in range(len(errs) - 1)]
report("M5 steady MMS, spatial order (volume-weighted L2) -> 2", abs(orders[-1] - 2) < 0.15,
       f"errors {[f'{e:.2e}' for e in errs]}, orders {[f'{o:.3f}' for o in orders]}")

# M6 transient variant: BE with exact step-averaged source, fine mesh, temporal order 1
N = 200
m = build_mesh(radii, Ds, [N, N])
gw, ge = couplings(m)
T = 1e7
# step-average of the cell-averaged source over [t0, t1]: S is linear in t, so the exact step
# average equals the value at the midpoint (exact, no quadrature error)
errsT = []
for nsteps in (4, 8, 16, 32, 64):
    dt = T / nsteps
    C = cell_avg(m, Fc1, Fc2, 0.0)
    for k in range(nsteps):
        tmid = (k + 0.5) * dt
        Sbar = cell_avg(m, fF1, fF2, tmid)
        C = solve_sf(m["V"] / dt + lamv * m["V"], gw, ge, (m["V"] / dt) * C + m["V"] * Sbar)
    cex = cell_avg(m, Fc1, Fc2, T)
    errsT.append(math.sqrt(np.sum(m["V"] * (C - cex)**2) / np.sum(m["V"] * cex**2)))
ordT = [math.log(errsT[i] / errsT[i + 1], 2) for i in range(len(errsT) - 1)]
report("M6 transient MMS, temporal order -> 1 (BE)", abs(ordT[-1] - 1) < 0.1,
       f"errors {[f'{e:.2e}' for e in errsT]}, orders {[f'{o:.3f}' for o in ordT]}")

print("ALL PASS" if ok_all else "SOME CHECKS FAILED")
