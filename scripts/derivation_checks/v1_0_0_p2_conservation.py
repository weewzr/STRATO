"""STRATO v1.0.0, Full Mathematical Derivation, Part II: governing conservation equation.

Checks every formula of Part II (Eqs. 2.1 to 2.9) and the worked numbers of Sections 2.5 and 2.8.

  P2.1  units of D from Fick's first law (Section 2.0.3)
  P2.2  thin-shell algebra: outer minus inner flow, expanded form
  P2.3  thin-shell limit of the flow term gives 4 pi d(r^2 J_r)/dr
  P2.4  thin-shell limit of the storage term gives 4 pi r^2 dc/dt
  P2.5  product rule (1/r^2) d(r^2 J)/dr = dJ/dr + 2 J / r
  P2.6  Cartesian divergence of J_r(r) e_r equals (1/r^2) d(r^2 J_r)/dr (Eq. 2.4)
  P2.7  small-box limit: net outflow / volume -> divergence (numerical, Gauss quadrature)
  P2.8  divergence theorem on a shell a < r < b (Eq. 2.1 against Eq. 2.5)
  P2.9  Fick substitution keeps D inside: d(r^2 D c')/dr = D (r^2 c')' + r^2 D' c'
  P2.10 expanded layer operator (Eq. 2.8)
  P2.11 an exact solution of Eq. (2.7) satisfies the shell balance (Eq. 2.2) on a finite shell
  P2.12 units of every term of Eq. (2.6)
  P2.13 two-layer steady state: D inside conserves, D pulled outside creates/destroys amount
  P2.14 diffusion time scales L^2/D and R^2/D for the synthetic input (read from the TOML)

Run:  C:/Y3S1/Nuclear/STRATO/.venv/Scripts/python.exe -I -X utf8 \
      C:/Y3S1/Nuclear/STRATO/scripts/derivation_checks/v1_0_0_p2_conservation.py
Writes nothing. Prints one PASS/FAIL line per check; exit code 1 on any FAIL.
"""

import math
import re
import sys
from pathlib import Path

import numpy as np
import sympy as sp

results = []


def check(label, ok, detail=""):
    ok = bool(ok)
    results.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {label}" + (f"  [{detail}]" if detail else ""), flush=True)


# ---------------------------------------------------------------- P2.1 units of D
mol, m, s = sp.symbols("mol m s", positive=True)
unit_J = mol / (m**2 * s)          # flux
unit_c = mol / m**3                # concentration
unit_dcdr = unit_c / m             # concentration gradient
unit_D = sp.simplify(unit_J / unit_dcdr)
check("P2.1 Fick: [D] = [J_r]/[dc/dr] = m^2 s^-1", sp.simplify(unit_D - m**2 / s) == 0, f"[D] = {unit_D}")

# ---------------------------------------------------------------- P2.2 to P2.5 thin shell
r, h, t, rho = sp.symbols("r h t rho", positive=True)
J = sp.Function("J")
Jr = J(r)
Jrh = J(r + h)
outflow_minus_inflow = 4 * sp.pi * (r + h) ** 2 * Jrh - 4 * sp.pi * r**2 * Jr
expanded = 4 * sp.pi * (r**2 * (Jrh - Jr) + (2 * r * h + h**2) * Jrh)
check("P2.2 4pi(r+h)^2 J(r+h) - 4pi r^2 J(r) = 4pi[r^2 (J(r+h)-J(r)) + (2rh + h^2) J(r+h)]",
      sp.simplify(sp.expand(outflow_minus_inflow - expanded)) == 0)

# limit h -> 0 of (outflow - inflow)/h, using a Taylor expansion of J(r+h) to second order
J0, J1, J2 = sp.symbols("J0 J1 J2")
taylor = J0 + J1 * h + J2 * h**2 / 2
quot = (4 * sp.pi * (r + h) ** 2 * taylor - 4 * sp.pi * r**2 * J0) / h
lim = sp.limit(quot, h, 0)
target = 4 * sp.pi * (r**2 * J1 + 2 * r * J0)  # 4 pi d(r^2 J)/dr with J = J0, J' = J1
check("P2.3 lim_{h->0} [A(r+h)J(r+h) - A(r)J(r)]/h = 4pi (r^2 J' + 2 r J) = 4pi d(r^2 J)/dr",
      sp.simplify(lim - target) == 0, f"limit = {sp.factor(lim)}")
# same with concrete smooth flux functions, exact limit
ok_concrete = True
for Jf in (sp.exp(-r) * r, sp.sin(r) + r**3, 1 / (1 + r**2)):
    q = (4 * sp.pi * (r + h) ** 2 * Jf.subs(r, r + h) - 4 * sp.pi * r**2 * Jf) / h
    ok_concrete &= sp.simplify(sp.limit(q, h, 0) - 4 * sp.pi * sp.diff(r**2 * Jf, r)) == 0
check("P2.3b same limit for three concrete fluxes (r e^-r, sin r + r^3, 1/(1+r^2))", ok_concrete)

# storage term: (1/h) d/dt int_r^{r+h} c 4 pi rho^2 drho -> 4 pi r^2 dc/dt (concrete c)
ok_store = True
for cf in (sp.exp(-rho) * sp.exp(-t), sp.sin(rho) / rho * sp.exp(-2 * t), rho**2 * t):
    stored = sp.integrate(cf * 4 * sp.pi * rho**2, (rho, r, r + h))
    rate = sp.diff(stored, t)
    lim_s = sp.limit(rate / h, h, 0)
    ok_store &= sp.simplify(lim_s - 4 * sp.pi * r**2 * sp.diff(cf, t).subs(rho, r)) == 0
check("P2.4 lim_{h->0} (1/h) d/dt int_r^{r+h} c 4pi rho^2 drho = 4pi r^2 dc/dt (three concrete c)", ok_store)

check("P2.5 (1/r^2) d(r^2 J)/dr = J' + 2J/r",
      sp.simplify(sp.diff(r**2 * Jr, r) / r**2 - (sp.diff(Jr, r) + 2 * Jr / r)) == 0)

# ---------------------------------------------------------------- P2.6 Cartesian divergence
x, y, z = sp.symbols("x y z", real=True)
rr = sp.sqrt(x**2 + y**2 + z**2)
G = sp.Function("G")
field = [G(rr) * x / rr, G(rr) * y / rr, G(rr) * z / rr]
div_cart = sum(sp.diff(fi, vi) for fi, vi in zip(field, (x, y, z)))
R_ = sp.symbols("R_", positive=True)
div_sph = (sp.diff(R_**2 * G(R_), R_) / R_**2)
# evaluate at a generic point by substituting the radius symbol
diff_expr = sp.simplify(div_cart - div_sph.subs(R_, rr))
check("P2.6 div(J_r(r) e_r) in Cartesian x,y,z equals (1/r^2) d(r^2 J_r)/dr", diff_expr == 0)

# ---------------------------------------------------------------- P2.7 small-box limit
def radial_field(px, py, pz):
    rad = np.sqrt(px**2 + py**2 + pz**2)
    jr = rad**2 * np.exp(-rad)  # J_r(r) = r^2 e^{-r}
    return jr * px / rad, jr * py / rad, jr * pz / rad


def box_outflow(x0, y0, z0, hh, n=12):
    g, w = np.polynomial.legendre.leggauss(n)
    u = 0.5 * hh * g
    wu = 0.5 * hh * w
    U, V = np.meshgrid(u, u, indexing="ij")
    W = np.outer(wu, wu)
    tot = 0.0
    for sign in (+1, -1):
        fx, _, _ = radial_field(x0 + sign * hh / 2, y0 + U, z0 + V)
        tot += sign * np.sum(W * fx)
        _, fy, _ = radial_field(x0 + U, y0 + sign * hh / 2, z0 + V)
        tot += sign * np.sum(W * fy)
        _, _, fz = radial_field(x0 + U, y0 + V, z0 + sign * hh / 2)
        tot += sign * np.sum(W * fz)
    return tot


p = np.array([0.7, -0.4, 1.1])
rp = float(np.linalg.norm(p))
div_exact = (4 * rp - rp**2) * math.exp(-rp)  # (1/r^2) d(r^4 e^-r)/dr
errs = []
for hh in (0.2, 0.1, 0.05, 0.025):
    est = box_outflow(*p, hh) / hh**3
    errs.append(abs(est - div_exact))
order = math.log2(errs[-2] / errs[-1])
check("P2.7 net outflow of a cube / its volume -> divergence as side h -> 0 (error O(h^2))",
      errs[-1] < 1e-3 * abs(div_exact) and 1.8 < order < 2.2,
      f"div = {div_exact:.10f}, errors {', '.join(f'{e:.2e}' for e in errs)}, observed order {order:.2f}")

# ---------------------------------------------------------------- P2.8 divergence theorem on a shell
a, b = sp.symbols("a b", positive=True)
ok_dt = True
for Jf in (sp.exp(-r) * r, r**3 - 2 * r, sp.cos(r)):
    lhs = sp.integrate(sp.diff(r**2 * Jf, r) / r**2 * 4 * sp.pi * r**2, (r, a, b))
    rhs = 4 * sp.pi * (b**2 * Jf.subs(r, b) - a**2 * Jf.subs(r, a))
    ok_dt &= sp.simplify(lhs - rhs) == 0
check("P2.8 int_shell div J dV = 4pi b^2 J(b) - 4pi a^2 J(a) (three concrete J_r)", ok_dt)

# ---------------------------------------------------------------- P2.9, P2.10 Fick substitution
Df = sp.Function("D")
c = sp.Function("c")
cr = c(r)
full = sp.diff(r**2 * Df(r) * sp.diff(cr, r), r)
split = Df(r) * sp.diff(r**2 * sp.diff(cr, r), r) + r**2 * sp.diff(Df(r), r) * sp.diff(cr, r)
check("P2.9 d(r^2 D c')/dr = D d(r^2 c')/dr + r^2 D' c' (D outside loses r^2 D' c')",
      sp.simplify(full - split) == 0)

Dc = sp.symbols("D_l", positive=True)
lhs = Dc / r**2 * sp.diff(r**2 * sp.diff(cr, r), r)
rhs = Dc * (sp.diff(cr, r, 2) + 2 / r * sp.diff(cr, r))
check("P2.10 (D/r^2) d(r^2 c')/dr = D (c'' + 2c'/r)", sp.simplify(lhs - rhs) == 0)

# ---------------------------------------------------------------- P2.11 exact solution and shell balance
k = sp.symbols("k", positive=True)
cex = sp.sin(k * r) / r * sp.exp(-Dc * k**2 * t)
res = sp.diff(cex, t) - Dc / r**2 * sp.diff(r**2 * sp.diff(cex, r), r)
check("P2.11a c = sin(kr)/r exp(-D k^2 t) satisfies Eq. (2.7) (SymPy residual)", sp.simplify(res) == 0)
vals = {Dc: sp.Rational(3, 10), k: sp.Rational(7, 5), t: sp.Rational(1, 2), a: sp.Rational(3, 10), b: sp.Rational(17, 10)}
storage = sp.diff(sp.integrate(cex * 4 * sp.pi * r**2, (r, a, b)), t)
Jex = -Dc * sp.diff(cex, r)
netin = 4 * sp.pi * a**2 * Jex.subs(r, a) - 4 * sp.pi * b**2 * Jex.subs(r, b)
gap = sp.N((storage - netin).subs(vals), 30)
check("P2.11b same c satisfies the finite-shell balance Eq. (2.2) on [0.3, 1.7]", abs(gap) < 1e-25, f"residual {gap:.1e}")

# ---------------------------------------------------------------- P2.12 units of every term
unit_t = s
unit_r = m
unit_S = mol / (m**3 * s)
term_dcdt = unit_c / unit_t
term_div = (1 / unit_r**2) * (1 / unit_r) * unit_r**2 * unit_D * unit_c / unit_r
check("P2.12 Eq. (2.6): [dc/dt] = [(1/r^2) d/dr (r^2 D dc/dr)] = [S] = mol m^-3 s^-1",
      sp.simplify(term_dcdt - unit_S) == 0 and sp.simplify(term_div - unit_S) == 0,
      f"{term_dcdt}, {sp.simplify(term_div)}, {unit_S}")
unit_shell = [unit_c * m**3 / s, m**2 * unit_J, unit_S * m**3]
check("P2.12b Eq. (2.2): every term in mol s^-1",
      all(sp.simplify(u_ - mol / s) == 0 for u_ in unit_shell))

# ---------------------------------------------------------------- P2.13 why D stays inside
# Two-layer steady shell a < r < b, interface at Ri, c(a) = 1, c(b) = 0, S = 0.
aa, Ri, bb, D1, D2 = 1.0e-4, 2.0e-4, 3.0e-4, 1.0e-12, 4.0e-10   # synthetic numbers, SI
# Correct (D inside): r^2 D c' = -Q/(4 pi), the same Q in both layers.
#   c(a) - c(b) = Q/(4 pi) [ (1/a - 1/Ri)/D1 + (1/Ri - 1/b)/D2 ]
Qc = 4 * math.pi * 1.0 / ((1 / aa - 1 / Ri) / D1 + (1 / Ri - 1 / bb) / D2)
# Build the piecewise profile from Q and check it independently: end values, continuity of c at Ri,
# the flow -4 pi r^2 D c' evaluated on each side of Ri (SymPy derivatives), and the steady equation in each layer.
rs_ = sp.symbols("rs", positive=True)
aS, RiS, bS = sp.Rational(1, 10**4), sp.Rational(2, 10**4), sp.Rational(3, 10**4)
D1S, D2S = sp.Rational(1, 10**12), sp.Rational(4, 10**10)
QS = 4 * sp.pi / ((1 / aS - 1 / RiS) / D1S + (1 / RiS - 1 / bS) / D2S)
c1p = 1 - QS / (4 * sp.pi * D1S) * (1 / aS - 1 / rs_)
c2p = QS / (4 * sp.pi * D2S) * (1 / rs_ - 1 / bS)
flow_in_c = (-4 * sp.pi * rs_**2 * D1S * sp.diff(c1p, rs_)).subs(rs_, RiS)
flow_out_c = (-4 * sp.pi * rs_**2 * D2S * sp.diff(c2p, rs_)).subs(rs_, RiS)
ok13a = (c1p.subs(rs_, aS) == 1 and c2p.subs(rs_, bS) == 0
         and sp.simplify(c1p.subs(rs_, RiS) - c2p.subs(rs_, RiS)) == 0
         and sp.simplify(flow_in_c - flow_out_c) == 0
         and sp.simplify(sp.diff(rs_**2 * D1S * sp.diff(c1p, rs_), rs_)) == 0
         and sp.simplify(sp.diff(rs_**2 * D2S * sp.diff(c2p, rs_), rs_)) == 0
         and abs(float(QS) - Qc) <= 1e-12 * Qc)
check("P2.13a D inside: piecewise steady profile has c(a)=1, c(b)=0, c continuous and equal flow on both sides of Ri",
      ok13a, f"Q = {Qc:.6e} mol s^-1, flow in {float(flow_in_c):.6e}, flow out {float(flow_out_c):.6e}")
# Wrong (D pulled out of the derivative): r^2 c' is then the same constant K in both layers.
#   c(a) - c(b) = K' (1/a - 1/b) with K' = -r^2 c'
Kp = 1.0 / (1 / aa - 1 / bb)
flow_layer1 = 4 * math.pi * D1 * Kp   # 4 pi r^2 (-D1 c') just inside Ri
flow_layer2 = 4 * math.pi * D2 * Kp   # 4 pi r^2 (-D2 c') just outside Ri
created = flow_layer2 - flow_layer1
check("P2.13b D outside: flow leaving the interface exceeds flow arriving by 4pi K'(D2 - D1) != 0",
      abs(created) > 100 * flow_layer1,
      f"arriving {flow_layer1:.3e}, leaving {flow_layer2:.3e} mol s^-1, ratio D2/D1 = {D2 / D1:.0f}")

# ---------------------------------------------------------------- P2.14 time scales of the synthetic input
toml = Path("C:/Y3S1/Nuclear/STRATO/inputs/synthetic/synthetic_five_layer.toml").read_text(encoding="utf-8")
names = re.findall(r'(?m)^name\s*=\s*"([^"]+)"', toml)
radii = [float(v) for v in re.findall(r"outer_radius_m\s*=\s*([0-9.eE+-]+)", toml)]
diffs = [float(v) for v in re.findall(r"diffusivity_m2_s\s*=\s*([0-9.eE+-]+)", toml)]
t_end = float(re.search(r"end_time_s\s*=\s*([0-9.eE+-]+)", toml).group(1))
check("P2.14a synthetic input parsed: five layers", len(radii) == 5 and len(diffs) == 5 and len(names) == 5,
      ", ".join(f"{n} R={R:.2e} D={D:.1e}" for n, R, D in zip(names, radii, diffs)))
inner = [0.0] + radii[:-1]
taus = [(R - Rin) ** 2 / D for R, Rin, D in zip(radii, inner, diffs)]
expected = [4.0e4, 25.0, 3125.0, 1.25e5, 2.5e-9 / 7.0e-13]
check("P2.14b L_l^2/D_l = 4.0e4, 25, 3125, 1.25e5, 3571.4 s",
      all(abs(a_ - e_) <= 1e-12 * e_ for a_, e_ in zip(taus, expected)),
      ", ".join(f"{v:.6g} s" for v in taus))
R = radii[-1]
tau_R_kernel = R**2 / diffs[0]
tau_R_sic = R**2 / diffs[3]
check("P2.14c R^2/D with D_1 and with D_4: 2.025e5 s and 1.0125e7 s",
      abs(tau_R_kernel - 2.025e5) < 1e-6 and abs(tau_R_sic - 1.0125e7) < 1e-3,
      f"{tau_R_kernel:.6g} s, {tau_R_sic:.6g} s; t_end = {t_end:.6g} s")
lam1_homog = math.pi**2 * diffs[0] / radii[0] ** 2
check("P2.14d bare kernel (radius R_1, D_1, c = 0 at R_1): 1/Lambda_1 = R_1^2/(pi^2 D_1) = 4053 s",
      abs(1 / lam1_homog - 4.0e4 / math.pi**2) < 1e-9, f"1/Lambda_1 = {1 / lam1_homog:.6g} s")
lam1_code = 3.273255671177023e-6   # Lambda_1 of the five-layer case (audit 01, sec1_synthetic_case.out)
check("P2.14e five-layer 1/Lambda_1 = 3.055e5 s is of the order of the largest layer time L_4^2/D_4",
      0.1 < (1 / lam1_code) / max(taus) < 10, f"1/Lambda_1 = {1 / lam1_code:.4g} s, max L^2/D = {max(taus):.4g} s")

conv = [taus[0] / 3600, taus[2] / 60, taus[3] / 3600, taus[4] / 60, t_end / 3600, (1 / lam1_code) / 3600]
want = [11.11, 52.08, 34.72, 59.52, 55.56, 84.86]
check("P2.14f unit conversions: 11.11 h, 52.08 min, 34.72 h, 59.52 min, t_end 55.56 h, 1/Lambda_1 84.86 h",
      all(abs(round(v_, 2) - w_) < 1e-9 for v_, w_ in zip(conv, want)), ", ".join(f"{v_:.4f}" for v_ in conv))

n_pass = sum(results)
print(f"SUMMARY {n_pass}/{len(results)} PASS" + ("  ALL PASS" if all(results) else ""))
sys.exit(0 if all(results) else 1)
