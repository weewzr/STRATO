"""STRATO v1.0.0, Full Mathematical Derivation, Part III: five-layer model and conditions.

Checks every formula and worked number of Part III (Eqs. 3.1 to 3.10). P3.16 and P3.17 were added by the
adversarial check of Part III.

  P3.1  synthetic input: five increasing radii, thicknesses, diffusivities (read from the TOML)
  P3.2  condition count: 5 second-order layer equations need 10 conditions = 1 + 2*4 + 1
  P3.3  symmetry: a nonzero centre slope makes c(|s|) kinked along a line through the centre
  P3.4  small-ball balance: bounded dc/dt forces J_r(eps) = O(eps) -> 0 (Eq. 3.1)
  P3.5  regular branch sin(kr)/r: zero slope, zero centre flow; singular branch cos(kr)/r
        and B/r: centre flow 4 pi D (point source), so boundedness excludes them (Eq. 3.1)
  P3.6  M^0 = (4/3) pi R_1^3 c_0 by integration, value for the synthetic input, and the
        FV replica sum of 40 kernel shell volumes with the snapped last face (Eq. 3.6)
  P3.7  thin interface shell: volume (4pi/3)(6 R^2 delta + 2 delta^3) -> 0, areas -> 4 pi R^2 (Eq. 3.3)
  P3.8  two-layer steady shell: with (3.2), (3.3) the flow 4 pi r^2 J_r is the same in both
        layers; with slope continuity instead, the flow jumps (interface would create amount)
  P3.9  five-layer synthetic slowest mode by an independent ODE shooting in (phi, P = r^2 D phi'):
        Lambda_1 vs audit value, and the global balance -dM/dt = J_R (Eq. 3.8)
  P3.10 homogeneous sphere mode: dM/dt + J_R = 0 and M(0) - M(t) = int_0^t J_R (Eqs. 3.8, 3.9)
  P3.11 units of J_R, G_R C_Nc, M^0, F (Eqs. 3.6, 3.7, 3.9)
  P3.12 Robin condition: h -> infinity gives c_5(R) = c_inf; dimensionless form gives Bi (Eq. 3.10)
  P3.13 homogeneous sphere, uniform start: Robin release below Dirichlet release, increasing with Bi
  P3.14 partition factor (FUTURE): gamma = 1 gives Eq. (3.2); gamma_SiC = 0.5 under (IV.3) doubles c
  P3.15 slope ratio at each synthetic interface from Eq. (3.3): D_l / D_(l+1) (arithmetic on the TOML)
  P3.16 independent closed-form five-layer mode (A sin + B cos)/r with 2x2 matching: Lambda_1, the slope
        ratio measured on the mode, and -dM/dt = J_R with M by quadrature (P3.9b is built into its ODE)
  P3.17 Table 3.2 values by an independent finite-volume solve of the homogeneous Robin sphere
  P3.18 Section 3.5.5 (added by the A4 re-derivation): integration by parts across interfaces for the
        negative part, the pointwise and integral bounds behind 1 - F <= (R/R_1)^(3/2) exp(-6 D_min t/R^2),
        and that bound against the homogeneous-sphere series

Run:  C:/Y3S1/Nuclear/STRATO/.venv/Scripts/python.exe -I -X utf8 \
      C:/Y3S1/Nuclear/STRATO/scripts/derivation_checks/v1_0_0_p3_conditions.py
Writes nothing. Prints one PASS/FAIL line per check; exit code 1 on any FAIL.
"""

import math
import re
import sys
from pathlib import Path

import numpy as np
import sympy as sp
from scipy.integrate import solve_ivp
from scipy.optimize import brentq

results = []


def check(label, ok, detail=""):
    ok = bool(ok)
    results.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {label}" + (f"  [{detail}]" if detail else ""), flush=True)


# ---------------------------------------------------------------- P3.1 synthetic input
toml = Path("C:/Y3S1/Nuclear/STRATO/inputs/synthetic/synthetic_five_layer.toml").read_text()
radii = [float(x) for x in re.findall(r"outer_radius_m\s*=\s*([0-9.eE+-]+)", toml)]
diffs = [float(x) for x in re.findall(r"diffusivity_m2_s\s*=\s*([0-9.eE+-]+)", toml)]
c0_in = float(re.search(r"kernel_concentration_mol_m3\s*=\s*([0-9.eE+-]+)", toml).group(1))
names = re.findall(r'\[\[regions\]\]\s*name\s*=\s*"([^"]+)"', toml)
L = [radii[0]] + [radii[i] - radii[i - 1] for i in range(1, 5)]
check("P3.1 five layers, increasing radii, R = R_5 = 4.5e-4 m",
      len(radii) == 5 and all(b > a for a, b in zip([0.0] + radii, radii)) and radii[-1] == 4.5e-4,
      f"names {names}, R_l {radii}, L_l {['%.1e' % x for x in L]}, D_l {diffs}, c0 {c0_in}")

# ---------------------------------------------------------------- P3.2 condition count
layers = 5
needed = 2 * layers
supplied = 1 + 2 * (layers - 1) + 1
check("P3.2 10 conditions needed (2 per layer), 1 centre + 8 interface + 1 outer supplied",
      needed == supplied == 10, f"needed {needed}, supplied {supplied}")

# ---------------------------------------------------------------- P3.3 symmetry
s_, a_, b_ = sp.symbols("s a b", real=True)
# profile with a nonzero centre slope b: c(r) = a + b r, along a line f(s) = a + b |s|
f = a_ + b_ * sp.Abs(s_)
right = sp.limit((f.subs(s_, sp.Symbol("h", positive=True)) - a_) / sp.Symbol("h", positive=True),
                 sp.Symbol("h", positive=True), 0)
hneg = sp.Symbol("h", positive=True)
left = sp.limit((f.subs(s_, -hneg) - a_) / (-hneg), hneg, 0)
check("P3.3 c = a + b r seen along a line: right slope b, left slope -b, equal only if b = 0",
      sp.simplify(right - b_) == 0 and sp.simplify(left + b_) == 0, f"right {right}, left {left}")

# ---------------------------------------------------------------- P3.4 small-ball balance
eps, K, D = sp.symbols("epsilon K D", positive=True)
storage_bound = K * sp.Rational(4, 3) * sp.pi * eps**3           # |d/dt int c dV| <= K V
J_bound = sp.simplify(storage_bound / (4 * sp.pi * eps**2))
check("P3.4 |J_r(eps)| <= K eps / 3 -> 0 as eps -> 0",
      sp.simplify(J_bound - K * eps / 3) == 0 and sp.limit(J_bound, eps, 0) == 0, f"bound {J_bound}")

# ---------------------------------------------------------------- P3.5 regular and singular branches
r, k, B, t = sp.symbols("r k B t", positive=True)
reg = sp.sin(k * r) / r
sing = sp.cos(k * r) / r
flow = lambda c: -4 * sp.pi * r**2 * D * sp.diff(c, r)   # outward flow through sphere of radius r
lim_reg_slope = sp.limit(sp.diff(reg, r), r, 0)
lim_reg_flow = sp.limit(flow(reg), r, 0)
lim_reg_val = sp.limit(reg, r, 0)
lim_sing_flow = sp.limit(flow(sing), r, 0)
flow_B = sp.simplify(flow(B / r))
steady_B = sp.simplify(sp.diff(r**2 * sp.diff(B / r, r), r) / r**2)
amount_B = sp.integrate(B / r * 4 * sp.pi * r**2, (r, 0, eps))
pde_reg = sp.simplify(sp.diff(reg * sp.exp(-D * k**2 * t), t)
                      - D / r**2 * sp.diff(r**2 * sp.diff(reg * sp.exp(-D * k**2 * t), r), r))
check("P3.5a sin(kr)/r e^{-Dk^2 t} solves Eq. (2.7); value k, slope 0, centre flow 0",
      pde_reg == 0 and lim_reg_val == k and lim_reg_slope == 0 and lim_reg_flow == 0)
check("P3.5b cos(kr)/r: centre flow -> 4 pi D (point source)", sp.simplify(lim_sing_flow - 4 * sp.pi * D) == 0,
      f"limit {lim_sing_flow}")
check("P3.5c B/r: steady solution, flow 4 pi D B for every r, finite amount 2 pi B eps^2",
      steady_B == 0 and sp.simplify(flow_B - 4 * sp.pi * D * B) == 0
      and sp.simplify(amount_B - 2 * sp.pi * B * eps**2) == 0, f"flow {flow_B}, amount {amount_B}")

# ---------------------------------------------------------------- P3.6 initial inventory
R1, c0, R = sp.symbols("R_1 c_0 R", positive=True)
M0 = sp.integrate(c0 * 4 * sp.pi * r**2, (r, 0, R1)) + sp.integrate(0 * r, (r, R1, R))
check("P3.6a int_0^R1 c0 4 pi r^2 dr + int_R1^R 0 dr = (4/3) pi R1^3 c0",
      sp.simplify(M0 - sp.Rational(4, 3) * sp.pi * R1**3 * c0) == 0, f"{M0}")
M0_val = 4.0 / 3.0 * math.pi * radii[0] ** 3 * c0_in
# FV replica: 40 kernel cells, uniform width, last face snapped to R_1 (finite_volume.rs 55-62, 38-40)
n1 = 40
w = radii[0] / n1
faces = [k_ * w for k_ in range(n1)] + [radii[0]]
M0_fv = 0.0
for a, b in zip(faces[:-1], faces[1:]):
    M0_fv += 4.0 * math.pi / 3.0 * (b * b * b - a * a * a)
inv_B11 = 3.351032163829113e-11
check("P3.6b synthetic M^0 = 3.351032163829e-11 mol; FV telescoping sum equals closed form",
      abs(M0_val - inv_B11) / inv_B11 < 1e-15 and abs(M0_fv - M0_val) / M0_val < 1e-14,
      f"closed {M0_val:.16e}, FV sum {M0_fv:.16e}, rel diff {abs(M0_fv - M0_val) / M0_val:.1e}")

# ---------------------------------------------------------------- P3.7 thin interface shell
Rl, dl = sp.symbols("R_l delta", positive=True)
Vshell = sp.Rational(4, 3) * sp.pi * ((Rl + dl) ** 3 - (Rl - dl) ** 3)
check("P3.7 V(R-d, R+d) = (4pi/3)(6R^2 d + 2 d^3) -> 0; A(R +- d) -> 4 pi R^2",
      sp.simplify(Vshell - sp.Rational(4, 3) * sp.pi * (6 * Rl**2 * dl + 2 * dl**3)) == 0
      and sp.limit(Vshell, dl, 0) == 0
      and sp.limit(4 * sp.pi * (Rl - dl) ** 2, dl, 0) == 4 * sp.pi * Rl**2
      and sp.limit(4 * sp.pi * (Rl + dl) ** 2, dl, 0) == 4 * sp.pi * Rl**2)

# ---------------------------------------------------------------- P3.8 two-layer steady shell
ra, rb, rc, D1s, D2s = sp.symbols("r_a r_b r_c D_1 D_2", positive=True)
A1, B1, A2, B2 = sp.symbols("A1 B1 A2 B2")
c1 = A1 + B1 / r
c2 = A2 + B2 / r
base = [sp.Eq(c1.subs(r, ra), 1), sp.Eq(c2.subs(r, rc), 0), sp.Eq(c1.subs(r, rb), c2.subs(r, rb))]
sol_ok = sp.solve(base + [sp.Eq(D1s * sp.diff(c1, r).subs(r, rb), D2s * sp.diff(c2, r).subs(r, rb))],
                  [A1, B1, A2, B2], dict=True)[0]
Q1 = sp.simplify((-4 * sp.pi * r**2 * D1s * sp.diff(c1, r)).subs(sol_ok))
Q2 = sp.simplify((-4 * sp.pi * r**2 * D2s * sp.diff(c2, r)).subs(sol_ok))
sol_bad = sp.solve(base + [sp.Eq(sp.diff(c1, r).subs(r, rb), sp.diff(c2, r).subs(r, rb))],
                   [A1, B1, A2, B2], dict=True)[0]
Q1b = sp.simplify((-4 * sp.pi * r**2 * D1s * sp.diff(c1, r)).subs(sol_bad))
Q2b = sp.simplify((-4 * sp.pi * r**2 * D2s * sp.diff(c2, r)).subs(sol_bad))
check("P3.8 with Eq. (3.3) flow equal in both layers; with slope continuity it jumps when D_1 != D_2",
      sp.simplify(Q1 - Q2) == 0 and sp.simplify((Q1b - Q2b).subs({D1s: 1, D2s: 2, ra: 1, rb: 2, rc: 3})) != 0,
      f"Q = {sp.factor(Q1)}")

# ---------------------------------------------------------------- P3.9 five-layer slowest mode
R_out = radii[-1]


def shoot(lam, dense=False):
    """Integrate y = (phi, P = r^2 D phi', m = int phi r^2 dr) from r0 to R; phi and P continuous."""
    r0 = 1e-9
    y = [1.0 - lam * r0**2 / (6 * diffs[0]), -lam * r0**3 / 3.0, r0**3 / 3.0]
    left = r0
    for Rl_, Dl_ in zip(radii, diffs):
        f_ = lambda rr, yy, Dl_=Dl_: [yy[1] / (rr * rr * Dl_), -lam * rr * rr * yy[0], rr * rr * yy[0]]
        out = solve_ivp(f_, (left, Rl_), y, method="DOP853", rtol=1e-13, atol=1e-30)
        y = out.y[:, -1].tolist()
        left = Rl_
    return y


grid = np.geomspace(1e-8, 1e-4, 200)
vals = [shoot(g)[0] for g in grid]
i0 = next(i for i in range(len(vals) - 1) if vals[i] > 0 and vals[i + 1] <= 0)
lam1 = brentq(lambda x: shoot(x)[0], grid[i0], grid[i0 + 1], xtol=1e-22, rtol=1e-15)
# Reference: 40-digit phi-form evaluation (A4 re-derivation; Part IV quotes 3.273255671177022e-6).
# The earlier reference 3.273255671175805e-6 was the Rust Code C value (printed by an audit-only example).
lam_audit = 3.273255671177023e-6
phi_R, P_R, m_R = shoot(lam1)
JR_over_4pi = -P_R                       # J_R / (4 pi) = -R^2 D_5 phi'(R) = -P(R)
dMdt_over_4pi = -lam1 * m_R              # dM/dt / (4 pi) = -Lambda int phi r^2 dr
check("P3.9a slowest synthetic eigenvalue from (phi, P) shooting matches the 40-digit reference",
      abs(lam1 - lam_audit) / lam_audit < 1e-9, f"Lambda_1 = {lam1:.15e} s^-1, rel diff {abs(lam1 - lam_audit) / lam_audit:.1e}")
check("P3.9b global balance -dM/dt = J_R for the five-layer mode (interface terms cancel)",
      abs(dMdt_over_4pi + JR_over_4pi) / abs(JR_over_4pi) < 1e-9,
      f"-dM/dt/4pi {-dMdt_over_4pi:.12e}, J_R/4pi {JR_over_4pi:.12e}, "
      f"rel diff {abs(dMdt_over_4pi + JR_over_4pi) / abs(JR_over_4pi):.1e}, J_R > 0: {JR_over_4pi > 0}")

# ---------------------------------------------------------------- P3.10 homogeneous release identities
n_ = sp.symbols("n", positive=True, integer=True)
kk = n_ * sp.pi / R
lam = D * kk**2
cm = sp.sin(kk * r) / r * sp.exp(-lam * t)
Mt = sp.integrate(cm * 4 * sp.pi * r**2, (r, 0, R))
JR = sp.simplify(-4 * sp.pi * R**2 * D * sp.diff(cm, r).subs(r, R))
tp = sp.symbols("tp", positive=True)
Mrel = sp.integrate(JR.subs(t, tp), (tp, 0, t))
check("P3.10 one mode: dM/dt + J_R = 0 and M(0) - M(t) = int_0^t J_R dt'",
      sp.simplify(sp.diff(Mt, t) + JR) == 0 and sp.simplify(Mt.subs(t, 0) - Mt - Mrel) == 0,
      f"J_R = {JR}")

# ---------------------------------------------------------------- P3.11 units
mol, m, s = sp.symbols("mol m s", positive=True)
u_c = mol / m**3
u_JR = sp.simplify(m**2 * (m**2 / s) * (u_c / m))
u_GR = sp.simplify(m**2 * (m**2 / s) / m)
check("P3.11 [4 pi R^2 D dc/dr] = mol/s, [G_R] = m^3/s, [G_R C] = mol/s, [M^0] = mol, [F] = 1",
      u_JR == mol / s and u_GR == m**3 / s and sp.simplify(u_GR * u_c) == mol / s
      and sp.simplify(m**3 * u_c) == mol and sp.simplify((mol / s * s) / mol) == 1)

# ---------------------------------------------------------------- P3.12 Robin limit and Biot number
h, cinf, g = sp.symbols("h c_inf g", positive=True)
cR = sp.symbols("c_R")
# -D g = h (c_R - c_inf) with g = dc/dr at R  ->  c_R = c_inf - D g / h
cR_sol = sp.solve(sp.Eq(-D * g, h * (cR - cinf)), cR)[0]
check("P3.12 Robin: c_R = c_inf - D g/h -> c_inf as h -> inf; times R/D gives -R g = Bi (c_R - c_inf)",
      sp.limit(cR_sol, h, sp.oo) == cinf
      and sp.simplify((-D * g * R / D) - (h * R / D) * (cR_sol - cinf)) == 0, f"c_R = {cR_sol}")

# ---------------------------------------------------------------- P3.13 Robin vs Dirichlet, homogeneous sphere


def robin_release(Bi, tau, nmodes=400):
    """Uniform start in a homogeneous sphere; -D c_r = h c at R; F = 1 - sum w_n exp(-z_n^2 tau)."""
    F = 1.0
    for n in range(nmodes):
        if Bi is None:
            z = (n + 1) * math.pi
        else:
            fz = lambda z: z * math.cos(z) - (1.0 - Bi) * math.sin(z)
            lo = 1e-9 if n == 0 else n * math.pi
            z = brentq(fz, lo, (n + 1) * math.pi, xtol=1e-15)
        I = (math.sin(z) - z * math.cos(z)) / z**2          # int_0^1 x sin(z x) dx (R = 1)
        N = 0.5 * (1.0 - math.sin(2 * z) / (2 * z))           # int_0^1 sin^2(z x) dx
        F -= 3.0 * I * I / N * math.exp(-z * z * tau)
    return F


taus = [0.02, 0.05, 0.1, 0.3]
table = {Bi: [robin_release(Bi, ta) for ta in taus] for Bi in (1.0, 10.0, 100.0, 1e6, None)}
crank = [1 - 6 / math.pi**2 * sum(math.exp(-n * n * math.pi**2 * ta) / n**2 for n in range(1, 4000)) for ta in taus]
order_ok = all(table[1.0][i] < table[10.0][i] < table[100.0][i] < table[None][i] for i in range(len(taus)))
check("P3.13 homogeneous sphere: F(Bi=1) < F(Bi=10) < F(Bi=100) < F(Dirichlet) at tau = 0.02 to 0.3",
      order_ok and max(abs(a - b) for a, b in zip(table[None], crank)) < 1e-12
      and max(abs(a - b) for a, b in zip(table[1e6], table[None])) < 1e-4,
      "tau=0.1: " + ", ".join(f"Bi={k_ if k_ else 'inf'}: {v[2]:.6f}" for k_, v in table.items()))

# ---------------------------------------------------------------- P3.14 partition factor (FUTURE)
gl, gr_, cl = sp.symbols("gamma_l gamma_r c_l", positive=True)
c_right = sp.solve(sp.Eq(gl * cl, gr_ * sp.Symbol("x")), sp.Symbol("x"))[0]
check("P3.14 Gelbard (IV.3): c_(l+1) = (gamma_l/gamma_(l+1)) c_l; gamma = 1 gives (3.2); "
      "gamma_IPyC = 1, gamma_SiC = 1/2 gives c_SiC = 2 c_IPyC",
      sp.simplify(c_right.subs({gl: 1, gr_: 1}) - cl) == 0
      and sp.simplify(c_right.subs({gl: 1, gr_: sp.Rational(1, 2)}) - 2 * cl) == 0, f"c_(l+1) = {c_right}")

# ---------------------------------------------------------------- P3.15 slope ratios at the interfaces
ratios = [diffs[i] / diffs[i + 1] for i in range(4)]
expect = [1e-12 / 4e-10, 4e-10 / 8e-13, 8e-13 / 2e-14, 2e-14 / 7e-13]
check("P3.15 slope ratio (dc/dr)_(l+1)/(dc/dr)_l = D_l/D_(l+1) at R_1..R_4 (synthetic)",
      all(abs(a - b) <= 1e-12 * b for a, b in zip(ratios, expect)),
      ", ".join(f"R_{i + 1}: {x:.4g}" for i, x in enumerate(ratios)))

# ---------------------------------------------------------------- P3.16 closed-form five-layer mode
# Independent of P3.9: phi_l = (A_l sin(k_l r) + B_l cos(k_l r))/r, A_1 = 1, B_1 = 0 (regular centre),
# 2x2 matching of phi and D phi' at each interface, root of u(R) = R phi(R). The slope ratio and the
# balance are then evaluated on this mode (M by adaptive quadrature, J_R by the closed-form slope),
# so neither is built into the construction the way P stays continuous in the (phi, P) shooting.
from scipy.integrate import quad


def cf_coeffs(lam_):
    A_, B_ = 1.0, 0.0
    out_ = []
    for l_ in range(5):
        k_ = math.sqrt(lam_ / diffs[l_])
        out_.append((A_, B_, k_))
        if l_ < 4:
            rr = radii[l_]
            k2 = math.sqrt(lam_ / diffs[l_ + 1])
            u_ = A_ * math.sin(k_ * rr) + B_ * math.cos(k_ * rr)
            du_ = k_ * (A_ * math.cos(k_ * rr) - B_ * math.sin(k_ * rr))
            flux_ = diffs[l_] * (du_ - u_ / rr)
            s2, c2 = math.sin(k2 * rr), math.cos(k2 * rr)
            A_, B_ = np.linalg.solve(np.array([[s2, c2], [k2 * c2, -k2 * s2]]),
                                     np.array([u_, flux_ / diffs[l_ + 1] + u_ / rr]))
    return out_


def cf_dphi(rr, l_, cf_):
    A_, B_, k_ = cf_[l_]
    u_ = A_ * math.sin(k_ * rr) + B_ * math.cos(k_ * rr)
    du_ = k_ * (A_ * math.cos(k_ * rr) - B_ * math.sin(k_ * rr))
    return (du_ - u_ / rr) / rr


def cf_uR(lam_):
    A_, B_, k_ = cf_coeffs(lam_)[4]
    return A_ * math.sin(k_ * R_out) + B_ * math.cos(k_ * R_out)


g2 = np.geomspace(1e-8, 1e-4, 400)
v2 = [cf_uR(x_) for x_ in g2]
j0 = next(i for i in range(len(v2) - 1) if v2[i] * v2[i + 1] < 0)
lam_cf = brentq(cf_uR, g2[j0], g2[j0 + 1], xtol=1e-24, rtol=1e-15)
cfm = cf_coeffs(lam_cf)
slope_ratio = [cf_dphi(radii[l_], l_ + 1, cfm) / cf_dphi(radii[l_], l_, cfm) for l_ in range(4)]
m_cf = 0.0
for l_ in range(5):
    A_, B_, k_ = cfm[l_]
    m_cf += quad(lambda rr, A_=A_, B_=B_, k_=k_: rr * (A_ * math.sin(k_ * rr) + B_ * math.cos(k_ * rr)),
                 0.0 if l_ == 0 else radii[l_ - 1], radii[l_], epsabs=0, epsrel=1e-13, limit=200)[0]
JR_cf = -4 * math.pi * R_out**2 * diffs[4] * cf_dphi(R_out, 4, cfm)
dMdt_cf = -lam_cf * 4 * math.pi * m_cf
check("P3.16 closed-form mode: Lambda_1 vs 40-digit reference, slope ratio D_l/D_(l+1) on the mode, -dM/dt = J_R by quadrature",
      abs(lam_cf / lam_audit - 1) < 1e-9
      and all(abs(a / b - 1) < 1e-8 for a, b in zip(slope_ratio, ratios))
      and abs(-dMdt_cf / JR_cf - 1) < 1e-8 and JR_cf > 0,
      f"Lambda_1 = {lam_cf:.15e} s^-1 (rel {abs(lam_cf / lam_audit - 1):.1e}), "
      f"ratios {', '.join(f'{x:.6g}' for x in slope_ratio)}, balance rel {abs(-dMdt_cf / JR_cf - 1):.1e}")

# ---------------------------------------------------------------- P3.17 Table 3.2 by finite volume
# Independent of the series of P3.13: homogeneous sphere R = D = 1, uniform start, 800 equal cells,
# Crank-Nicolson after four backward-Euler half steps; Robin face = half-cell resistance + 1/Bi.
from scipy.linalg import solve_banded


def fv_robin(bi_, tau_=0.1, n=800, steps=4000):
    dr_ = 1.0 / n
    f_ = np.arange(n + 1) * dr_
    V_ = (f_[1:] ** 3 - f_[:-1] ** 3) / 3.0
    rc_ = 0.5 * (f_[1:] + f_[:-1])
    G_ = f_[1:-1] ** 2 / dr_
    Gout = 1.0 / ((1.0 - rc_[-1]) + (0.0 if bi_ is None else 1.0 / bi_))
    dt_ = tau_ / steps
    diag = np.zeros(n)
    diag[:-1] += G_
    diag[1:] += G_
    diag[-1] += Gout
    ab = np.zeros((3, n))
    ab[0, 1:] = -0.5 * dt_ * G_
    ab[1, :] = V_ + 0.5 * dt_ * diag
    ab[2, :-1] = -0.5 * dt_ * G_
    c_ = np.ones(n)
    for _ in range(4):                      # backward Euler, step dt/2 (same matrix as CN's left side)
        c_ = solve_banded((1, 1), ab, V_ * c_)
    for _ in range(steps - 2):
        Ac = diag * c_
        Ac[:-1] -= G_ * c_[1:]
        Ac[1:] -= G_ * c_[:-1]
        c_ = solve_banded((1, 1), ab, V_ * c_ - 0.5 * dt_ * Ac)
    return 1.0 - (V_ * c_).sum() / V_.sum()


fv_tab = {b_: fv_robin(b_) for b_ in (1.0, 10.0, 100.0, None)}
check("P3.17 Table 3.2 (tau = 0.1) reproduced by an independent finite-volume solve to 2e-5",
      all(abs(fv_tab[b_] - table[b_][2]) < 2e-5 for b_ in fv_tab),
      ", ".join(f"Bi={b_ if b_ else 'inf'}: {v_:.6f}" for b_, v_ in fv_tab.items()))

# ---------------------------------------------------------------- P3.18 energy proofs of Section 3.5.5
# (a) integration by parts across an interface: sum_l int w dP/dr dr = -int w' 4 pi r^2 D c' dr when w is
#     continuous, w(R) = 0, P = 4 pi r^2 D c' is continuous and P(0) = 0. Two layers, D = 1 and 3, c < 0 on
#     (0, 2) so c^- = c; c and D c' continuous at r = 1; c'(0) = 0; c(2) = 0.
ca = -2 + r**2
cb = -1 + sp.Rational(2, 3) * (r - 1) + sp.Rational(1, 3) * (r - 1)**2
Pa, Pb = 4 * sp.pi * r**2 * 1 * sp.diff(ca, r), 4 * sp.pi * r**2 * 3 * sp.diff(cb, r)
conds = (ca.subs(r, 1) == cb.subs(r, 1) and Pa.subs(r, 1) == Pb.subs(r, 1) and cb.subs(r, 2) == 0
         and sp.diff(ca, r).subs(r, 0) == 0)
lhs18 = sp.integrate(ca * sp.diff(Pa, r), (r, 0, 1)) + sp.integrate(cb * sp.diff(Pb, r), (r, 1, 2))
rhs18 = -sp.integrate(sp.diff(ca, r) * Pa, (r, 0, 1)) - sp.integrate(sp.diff(cb, r) * Pb, (r, 1, 2))
check("P3.18a dZ/dt step: sum int c^- dP/dr dr = -int 4 pi r^2 D (dc^-/dr)^2 dr (interface and end terms cancel)",
      conds and sp.simplify(lhs18 - rhs18) == 0 and rhs18 < 0, f"both = {sp.nsimplify(rhs18)}")
# (b) pointwise bound c(r)^2 <= (1/r - 1/R) Phi / D_min and E <= R^2/(6 D_min) * 4 pi Phi, random profiles
rng = np.random.default_rng(18)
rg = np.linspace(1e-6, 1.0, 40001)
Dg = np.where(rg < 0.4, 1.0, np.where(rg < 0.7, 0.02, 0.5))
ok_b, worst_b, worst_e = True, 0.0, 0.0
for _ in range(100):
    cc = np.polynomial.polynomial.polyval(rg, rng.normal(size=7))
    cc -= cc[-1]
    dc = np.gradient(cc, rg)
    integrand = rg**2 * Dg * dc**2
    Phi = np.trapezoid(integrand, rg)
    bound = (1 / rg - 1.0) * Phi / Dg.min()
    worst_b = max(worst_b, np.max(cc[:-1]**2 / bound[:-1]))
    E_ = np.trapezoid(4 * np.pi * rg**2 * cc**2, rg)
    worst_e = max(worst_e, E_ / (1.0 / (6 * Dg.min()) * 4 * np.pi * Phi))
ok_b = worst_b <= 1 + 1e-6 and worst_e <= 1 + 1e-6
check("P3.18b c^2 <= (1/r - 1/R) Phi / D_min pointwise and E <= R^2/(6 D_min) 4 pi Phi (100 random profiles, 3 layers)",
      ok_b, f"largest ratios {worst_b:.3f}, {worst_e:.3f}")
check("P3.18c int_0^R r^2 (1/r - 1/R) dr = R^2/6",
      sp.simplify(sp.integrate(r**2 * (1 / r - 1 / R), (r, 0, R)) - R**2 / 6) == 0)
# (c) decay bound 1 - F <= (R/R_1)^(3/2) exp(-6 D_min t / R^2): synthetic rate and homogeneous sphere series
rate = 6 * min(diffs) / R_out**2
tau_g = np.geomspace(1e-4, 3.0, 200)
one_minus_F = [6 / math.pi**2 * sum(math.exp(-n**2 * math.pi**2 * ta) / n**2 for n in range(1, 4000)) for ta in tau_g]
check("P3.18d 6 D_min/R^2 <= Lambda_1 (synthetic); homogeneous sphere, R_1 = R: 1 - F(tau) <= exp(-6 tau) for tau in [1e-4, 3]",
      rate <= lam1 and all(v <= math.exp(-6 * ta) for v, ta in zip(one_minus_F, tau_g)),
      f"6 D_min/R^2 = {rate:.4e} s^-1, Lambda_1 = {lam1:.4e} s^-1; bound on (R/R_1)^1.5 = {(R_out / radii[0])**1.5:.4f}")

n_pass = sum(results)
print(f"SUMMARY {n_pass}/{len(results)} PASS" + ("  ALL PASS" if all(results) else ""))
sys.exit(0 if all(results) else 1)
