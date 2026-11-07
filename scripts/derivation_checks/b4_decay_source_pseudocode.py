"""
b4_decay_source_pseudocode.py

Checks for the Phase 2.1 pseudocode (decay, source and inventory): blocks A to E
of the working note decay_source_inventory.md (not in the repository).

The pseudocode introduces no new physics. It re-arranges equations P21.x that
were derived in Sections 3 and 4 and reconciled in B3. This script

  1. proves the helper identities H21.1 to H21.3 (stable evaluation of the
     interval factors) with SymPy and mpmath,
  2. runs a literal Python transcription of the pseudocode blocks
       A  decay constant from half-life,
       B  source from fission rate density and yield (cell and step averages),
       C  time grid and one Backward Euler step with decay and source,
       D  inventory, release and decay bookkeeping including M_out,
       E  modal solution with decay and piecewise-constant kernel source,
  3. checks the pass criteria of the verification cases (block F) that can be
     tested without Rust.

It is independent of the Rust code in src/. The mesh follows the build_mesh rules
of v1.0 (volume centroids, series face conductance, surface conductance with
R - r_N). All diffusivities, source strengths and times are synthetic test values
unless a source is named in a comment. It writes no files.

Run:
  .venv/Scripts/python.exe -I -X utf8 b4_decay_source_pseudocode.py
"""

import csv
import math
import os
import sys

import mpmath as mp
import numpy as np
import sympy as sp
from scipy.integrate import quad, solve_ivp

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from sec3_p21_common import Multilayer  # noqa: E402  (independent of src/)

REPO = os.path.dirname(os.path.dirname(HERE))
RESULTS = []


def check(label, ok, detail=""):
    RESULTS.append(bool(ok))
    tail = f"  {detail}" if detail else ""
    print(f"{'PASS' if ok else 'FAIL'}  {label}{tail}")


NA = 6.02214076e23  # mol^-1, SI defining constant
LN2 = math.log(2.0)

# ---------------------------------------------------------------------------
# Part 1. Helper identities H21.1 to H21.3
# ---------------------------------------------------------------------------
x_s, d_s, m0_s, g_s, k_s = sp.symbols("x Delta m0 g kappa", positive=True)
t_s = sp.symbols("t", positive=True)
phi1_s = (1 - sp.exp(-x_s)) / x_s
phi2_s = (x_s - 1 + sp.exp(-x_s)) / x_s**2

# H21.1: end of interval, written with phi1, equals (P21.33).
m_end = m0_s * sp.exp(-k_s * d_s) + g_s / k_s * (1 - sp.exp(-k_s * d_s))
m_h = m0_s * sp.exp(-k_s * d_s) + g_s * d_s * phi1_s.subs(x_s, k_s * d_s)
check("H21.1 m(t_j) = m e^{-kappa Delta} + g Delta phi1(kappa Delta) equals (P21.33)",
      sp.simplify(m_end - m_h) == 0)

# H21.2: time integral of the mode over the interval (per-interval form of P21.38).
mt = m0_s * sp.exp(-k_s * t_s) + g_s / k_s * (1 - sp.exp(-k_s * t_s))
int_mode = sp.integrate(mt, (t_s, 0, d_s))
int_h = d_s * (m0_s * phi1_s.subs(x_s, k_s * d_s) + g_s * d_s * phi2_s.subs(x_s, k_s * d_s))
check("H21.2 int_0^Delta m dt = Delta [m phi1 + g Delta phi2] (P21.38 per interval)",
      sp.simplify(int_mode - int_h) == 0)

# H21.3: series for small arguments; accuracy of the switch used in the pseudocode.
mp.mp.dps = 50


def phi1(x):
    """(1 - e^{-x})/x, evaluated without cancellation."""
    if x < 0.5:
        return sum((-x) ** k / math.factorial(k + 1) for k in range(14))
    return -math.expm1(-x) / x


def phi2(x):
    """(x - 1 + e^{-x})/x^2, evaluated without cancellation."""
    if x < 0.5:
        return sum((-x) ** k / math.factorial(k + 2) for k in range(14))
    return (x - 1.0 + math.exp(-x)) / (x * x)


worst1 = worst2 = 0.0
for xv in np.logspace(-12, 3, 400):
    r1 = mp.mpf(1) - mp.e ** (-mp.mpf(xv))
    e1 = r1 / mp.mpf(xv)
    e2 = (mp.mpf(xv) - 1 + mp.e ** (-mp.mpf(xv))) / mp.mpf(xv) ** 2
    worst1 = max(worst1, float(abs(phi1(xv) - e1) / e1))
    worst2 = max(worst2, float(abs(phi2(xv) - e2) / e2))
check("H21.3 phi1, phi2 with the series switch at 0.5 against 50-digit values (400 points, 1e-12 to 1e3)",
      worst1 < 1e-14 and worst2 < 1e-14, f"max rel error {worst1:.1e}, {worst2:.1e}")
x0 = 1e-14
check("H21.3 limit x -> 0: phi1 -> 1, phi2 -> 1/2", abs(phi1(x0) - 1) < 1e-13 and abs(phi2(x0) - 0.5) < 1e-13)

# ---------------------------------------------------------------------------
# Block A: decay constant from half-life
# ---------------------------------------------------------------------------
SECONDS = {"s": 1.0, "m": 60.0, "h": 3600.0, "d": 86400.0, "Y": 31556925.97}


def block_a_lambda(half_life_value, unit):
    """A1 to A4: convert to seconds, lambda = ln2 / t_half (P21.7), guard the result."""
    if half_life_value == "STABLE":
        return 0.0
    t_half_s = float(half_life_value) * SECONDS[unit]
    if not (math.isfinite(t_half_s) and t_half_s > 0.0):
        raise ValueError("half-life must be positive and finite")
    return LN2 / t_half_s


with open(os.path.join(REPO, "data", "nuclear", "decay.csv"), newline="", encoding="utf-8") as fh:
    decay_rows = list(csv.DictReader(fh))
worst_hl = worst_lam = 0.0
n_rad = 0
for row in decay_rows:
    if row["half_life"] == "STABLE":
        check(f"A stable nuclide {row['nuclide']} has lambda = 0", float(row["lambda_s-1"]) == 0.0)
        continue
    n_rad += 1
    lam = block_a_lambda(row["half_life"], row["unit"])
    worst_hl = max(worst_hl, abs(float(row["half_life"]) * SECONDS[row["unit"]] - float(row["half_life_s"])) / float(row["half_life_s"]))
    worst_lam = max(worst_lam, abs(lam - float(row["lambda_s-1"])) / float(row["lambda_s-1"]))
    assert abs(lam * float(row["half_life"]) * SECONDS[row["unit"]] - LN2) < 1e-12
check(f"A seconds conversion reproduces the half_life_s column ({n_rad} radioactive rows)", worst_hl < 5e-9,
      f"max rel diff {worst_hl:.1e}")
check("A lambda = ln2/t_half reproduces the lambda column (printed to 7 digits)", worst_lam < 5e-7,
      f"max rel diff {worst_lam:.1e}")
check("A lambda * t_half = ln 2 (dimensionless) for every row", True)
lam_cs137 = block_a_lambda("30.08", "Y")
check("A Cs-137 lambda = 7.302187e-10 s^-1 (Section 3.2)", abs(lam_cs137 - 7.302187e-10) / 7.302187e-10 < 1e-6,
      f"{lam_cs137:.6e}")
check("A mean life 1/lambda = t_half/ln2 (P21.7)", abs(1 / lam_cs137 - 30.08 * SECONDS["Y"] / LN2) < 1.0)
try:
    block_a_lambda("-1", "s")
    check("A negative half-life is rejected", False)
except ValueError:
    check("A negative half-life is rejected", True)

# ---------------------------------------------------------------------------
# Block B: source
# ---------------------------------------------------------------------------
with open(os.path.join(REPO, "data", "nuclear", "fission_yields.csv"), newline="", encoding="utf-8") as fh:
    yrows = list(csv.DictReader(fh))


def yld(parent, product, key):
    for r in yrows:
        if r["parent"] == parent and r["neutron_energy"] == "thermal" and r["product"] == product:
            return float(r[key])
    raise KeyError(product)


y_ifc = {p: (yld("235U", p, "IFY_JEFF311"), yld("235U", p, "CFY_JEFF311")) for p in ("Cs-137", "Sr-90")}
check("B yields read from the register CSV agree with Section 3.3.2 (Cs-137, Sr-90, U-235 thermal)",
      abs(y_ifc["Cs-137"][1] - 6.2208e-2) < 1e-6 and abs(y_ifc["Cs-137"][0] - 7.2248e-4) < 1e-8
      and abs(y_ifc["Sr-90"][1] - 5.7290e-2) < 1e-6 and abs(y_ifc["Sr-90"][0] - 3.1343e-4) < 1e-8,
      f"Cs-137 {y_ifc['Cs-137']}, Sr-90 {y_ifc['Sr-90']}")


def block_b_source(rate_density, yield_eff):
    """B2 (P21.8, P21.52): mol m^-3 s^-1 from fissions m^-3 s^-1 and atoms per fission."""
    return yield_eff * rate_density / NA


def t99(half_life_precursor_s):
    """B1 (P21.12): time for the precursor feed to reach 99 % of equilibrium."""
    return math.log(100.0) * half_life_precursor_s / LN2


check("B t99(Xe-137, 3.818 min) = 0.423 h (Section 3.3.2)", abs(t99(3.818 * 60) / 3600 - 0.4228) < 5e-4,
      f"{t99(3.818 * 60) / 3600:.4f} h")
check("B t99 = 6.644 t_half", abs(t99(1.0) - 6.6439) < 1e-3)
F_dot = 5.0e19  # fissions m^-3 s^-1, SYNTHETIC
S_K = block_b_source(F_dot, y_ifc["Cs-137"][1])
check("B S_K = y F/N_A units: mol m^-3 s^-1; Cs-137 value for the synthetic rate density",
      abs(S_K - 6.2208e-2 * 5e19 / NA) < 1e-18, f"S_K = {S_K:.4e} mol m^-3 s^-1")
# fission-weighted mean yield (P21.10), two fissile nuclides
yq = 0.7 * yld("235U", "Cs-137", "CFY_JEFF311") + 0.3 * yld("239Pu", "Cs-137", "CFY_JEFF311")
check("B P21.10 fission-weighted yield is bracketed by the two pure yields",
      min(yld("235U", "Cs-137", "CFY_JEFF311"), yld("239Pu", "Cs-137", "CFY_JEFF311")) <= yq
      <= max(yld("235U", "Cs-137", "CFY_JEFF311"), yld("239Pu", "Cs-137", "CFY_JEFF311")), f"y_eff = {yq:.5f}")


def cell_average_factor(left, right, r1):
    """B3 (P21.54): fraction of the cell volume that lies in the kernel."""
    if right <= r1:
        return 1.0
    if left >= r1:
        return 0.0
    return (r1**3 - left**3) / (right**3 - left**3)


rng = np.random.default_rng(7)
worst = 0.0
for _ in range(50):
    a, b = sorted(rng.uniform(0.0, 1.0, 2))
    r1 = rng.uniform(0.0, 1.0)
    num = quad(lambda r: r * r * (1.0 if r < r1 else 0.0), a, b, points=[r1] if a < r1 < b else None)[0]
    f_num = num / ((b**3 - a**3) / 3.0)
    worst = max(worst, abs(f_num - cell_average_factor(a, b, r1)))
check("B P21.54 cell-average factor against quadrature (50 random cells, including straddling)", worst < 1e-10,
      f"max abs diff {worst:.1e}")


def step_average(history, t0, t1):
    """B4 (P21.57 A): exact average over [t0, t1] of a piecewise-constant history
    [(t_start, t_end, value), ...]; zero outside the listed intervals."""
    total = 0.0
    for (a, b, v) in history:
        lo, hi = max(a, t0), min(b, t1)
        if hi > lo:
            total += v * (hi - lo)
    return total / (t1 - t0)


hist = [(0.0, 1.0e7, 2.0e-7), (1.0e7, 3.0e7, 0.0)]
check("B step average across a switch inside the step equals the exact integral / dt",
      abs(step_average(hist, 0.9e7, 1.1e7) - 2.0e-7 * 0.1e7 / 0.2e7) < 1e-20)

# ---------------------------------------------------------------------------
# Mesh, solver and the v1.0 step (literal transcription of finite_volume.rs)
# ---------------------------------------------------------------------------


def shell_volume(a, b):
    return 4.0 * math.pi / 3.0 * (b**3 - a**3)


def volume_centroid(a, b):
    if a == 0.0:
        return 0.75 * b
    return 0.75 * (b**4 - a**4) / (b**3 - a**3)


class Mesh:
    def __init__(self, radii, diffs, ncells):
        self.left, self.right, self.centroid, self.volume, self.mat, self.D = [], [], [], [], [], []
        inner = 0.0
        for mat, (R, D, n) in enumerate(zip(radii, diffs, ncells)):
            width = (R - inner) / n
            for k in range(n):
                left = inner + k * width
                right = R if k + 1 == n else inner + (k + 1) * width
                self.left.append(left)
                self.right.append(right)
                self.centroid.append(volume_centroid(left, right))
                self.volume.append(shell_volume(left, right))
                self.mat.append(mat)
                self.D.append(D)
            inner = R
        n = len(self.volume)
        self.n = n
        self.cond = []
        for j in range(n - 1):
            face = self.right[j]
            area = 4.0 * math.pi * face * face
            dl = face - self.centroid[j]
            dr = self.centroid[j + 1] - face
            self.cond.append(area / (dl / self.D[j] + dr / self.D[j + 1]))
        self.R = radii[-1]
        self.outer_cond = 4.0 * math.pi * self.R**2 * self.D[-1] / (self.R - self.centroid[-1])
        self.radii = list(radii)


def solve_diffusion_system(v, gw, ge, rhs):
    """Subtraction-free Thomas elimination (finite_volume.rs lines 132 to 159)."""
    n = len(v)
    den = [0.0] * n
    dp = [0.0] * n
    e_prev = 0.0
    for i in range(n):
        e = v[0] if i == 0 else v[i] + gw[i] * e_prev / (e_prev + gw[i])
        den[i] = e + ge[i]
        if den[i] <= 0.0 or not math.isfinite(den[i]):
            raise ValueError("singular pivot")
        carry = 0.0 if i == 0 else gw[i] * dp[i - 1]
        dp[i] = (rhs[i] + carry) / den[i]
        e_prev = e
    xx = [0.0] * n
    xx[n - 1] = dp[n - 1]
    for i in range(n - 2, -1, -1):
        xx[i] = dp[i] + ge[i] / den[i] * xx[i + 1]
    return xx


def faces(mesh):
    n = mesh.n
    gw = [0.0] + list(mesh.cond)
    ge = list(mesh.cond) + [mesh.outer_cond]
    return gw, ge


def be_step_v1(mesh, old, dt):
    """v1.0 backward_euler_step (lines 162 to 182)."""
    gw, ge = faces(mesh)
    v = [mesh.volume[i] / dt for i in range(mesh.n)]
    rhs = [v[i] * old[i] for i in range(mesh.n)]
    return solve_diffusion_system(v, gw, ge, rhs)


def be_step_new(mesh, old, dt, lam, sbar):
    """Block C2: v_j = V/dt + lam V,  b_j = (V/dt) C_old + V sbar_j  (P21.59)."""
    gw, ge = faces(mesh)
    v = [mesh.volume[i] / dt + lam * mesh.volume[i] for i in range(mesh.n)]
    rhs = [(mesh.volume[i] / dt) * old[i] + mesh.volume[i] * sbar[i] for i in range(mesh.n)]
    return solve_diffusion_system(v, gw, ge, rhs)


def inventory(mesh, c):
    return sum(mesh.volume[i] * c[i] for i in range(mesh.n))


class Kahan:
    def __init__(self):
        self.s = 0.0
        self.c = 0.0

    def add(self, inc):
        y = inc - self.c
        t = self.s + y
        self.c = (t - self.s) - y
        self.s = t


# ---------------------------------------------------------------------------
# Block C1: time grid
# ---------------------------------------------------------------------------


def time_grid(breakpoints, dt_start, dt_max, growth=1.5):
    """Step sizes: every breakpoint is a step boundary; after each breakpoint start with dt_start
    and grow by `growth` up to dt_max; scale the sizes of one leg so that it lands exactly."""
    sizes = []
    for a, b in zip(breakpoints[:-1], breakpoints[1:]):
        length = b - a
        leg, total, h = [], 0.0, dt_start
        while total < length * (1 - 1e-14):
            leg.append(min(h, dt_max))
            total += leg[-1]
            h *= growth
        scale = length / total
        sizes.extend(s * scale for s in leg)
    return sizes


g = time_grid([0.0, 1.0e7, 3.0e7], 1.0e4, 5.0e5)
cum = np.cumsum(g)
check("C1 time grid lands on every breakpoint", np.min(np.abs(cum - 1.0e7)) < 1e-3 and abs(cum[-1] - 3.0e7) < 1e-3
      and max(g) <= 5.0e5 * (1 + 1e-12), f"{len(g)} steps")
check("C1 first step after a breakpoint is not larger than dt_start", g[0] <= 1.0e4 * (1 + 1e-12))

# ---------------------------------------------------------------------------
# Blocks C2 and D: the FV run with decay, source and bookkeeping
# ---------------------------------------------------------------------------


def run_fv(mesh, lam, c0, history, sizes, record_times=()):
    """Time loop. history: piecewise-constant S_K(t) list; the source acts in material 0 only
    unless `mesh.source_all` is True. Returns final state and the bookkeeping dictionary."""
    r1 = mesh.radii[0]
    n = mesh.n
    src_all = getattr(mesh, "source_all", False)
    # P21.54: cells straddling R1 are not allowed on the v1.0 mesh (every R_i is a face)
    for j in range(n):
        if mesh.left[j] < r1 < mesh.right[j] and not src_all:
            raise ValueError("kernel radius is not a cell face")
    c = list(c0)
    M0 = inventory(mesh, c)
    kg, kd, kr, kdo = Kahan(), Kahan(), Kahan(), Kahan()
    Mout = 0.0
    t = 0.0
    out = {"t": [], "M": [], "Mgen": [], "Mdec": [], "Mrel": [], "Mout": [], "Mdecout": [], "JR": [],
           "res1": [], "res2": [], "minC": [], "Gen": []}
    worst1 = worst2 = 0.0
    for dt in sizes:
        t1 = t + dt
        s_avg = step_average(history, t, t1)  # B4 (A)
        fac = [1.0 if (src_all or mesh.mat[j] == 0) else 0.0 for j in range(n)]
        sbar = [s_avg * fac[j] for j in range(n)]
        c = be_step_new(mesh, c, dt, lam, sbar)
        Mnew = inventory(mesh, c)
        gen = sum(mesh.volume[j] * sbar[j] for j in range(n))
        JR = mesh.outer_cond * c[-1]
        kg.add(dt * gen)
        kd.add(dt * lam * Mnew)
        kr.add(dt * JR)
        Mout = (Mout + dt * JR) / (1.0 + lam * dt)
        kdo.add(dt * lam * Mout)
        res1 = (M0 + kg.s) - (Mnew + kd.s + kr.s)
        res2 = (M0 + kg.s) - (Mnew + Mout + kd.s + kdo.s)
        scale = max(M0 + kg.s, 1e-300)
        worst1 = max(worst1, abs(res1) / scale)
        worst2 = max(worst2, abs(res2) / scale)
        t = t1
        out["t"].append(t)
        out["M"].append(Mnew)
        out["Mgen"].append(kg.s)
        out["Mdec"].append(kd.s)
        out["Mrel"].append(kr.s)
        out["Mout"].append(Mout)
        out["Mdecout"].append(kdo.s)
        out["JR"].append(JR)
        out["Gen"].append(gen)
        out["minC"].append(min(c))
    out["M0"] = M0
    out["worst_res1"] = worst1
    out["worst_res2"] = worst2
    return c, out


def at(out, key, t):
    i = int(np.argmin(np.abs(np.array(out["t"]) - t)))
    assert abs(out["t"][i] - t) <= 1e-6 * max(t, 1.0), (out["t"][i], t)
    return out[key][i]


# five-layer test particle: radii are those used by sec4_p21_steady_state.py; diffusivities synthetic
RADII = [250e-6, 350e-6, 390e-6, 425e-6, 465e-6]
DIFFS = [4e-14, 4e-13, 2e-14, 5e-15, 2e-14]
CELLS = [20, 8, 4, 4, 4]
V_K = 4.0 / 3.0 * math.pi * RADII[0] ** 3

# ---- F1: bitwise regression with the v1.0 step -------------------------------
mesh = Mesh(RADII, DIFFS, CELLS)
c_a = [1.0 if m == 0 else 0.0 for m in mesh.mat]
c_b = list(c_a)
same = True
for k in range(200):
    dt = 50.0 * 1.05**k
    c_a = be_step_v1(mesh, c_a, dt)
    c_b = be_step_new(mesh, c_b, dt, 0.0, [0.0] * mesh.n)
    same = same and (c_a == c_b)
check("F1 lambda = 0, S = 0: new step equals the v1.0 step bit for bit (200 steps, Python floats)", same)

# the unsafe variant: V*(1/dt + lam) is not bit-identical to V/dt
unsafe_diff = sum(1 for i in range(mesh.n) if mesh.volume[i] * (1.0 / 37.0 + 0.0) != mesh.volume[i] / 37.0)
check("F1 detail: V*(1/dt) differs from V/dt in the last bit for some cells (the reason for the written form)",
      unsafe_diff > 0, f"{unsafe_diff} of {mesh.n} cells differ")

# ---- F2: decay only (no diffusion) -----------------------------------------------
mesh0 = Mesh(RADII, DIFFS, CELLS)
mesh0.cond = [0.0] * len(mesh0.cond)
mesh0.outer_cond = 0.0
lam_t, dt_t, nst = 1.0e-6, 1.0e4, 300  # z = 0.01
c = [1.0] * mesh0.n
for _ in range(nst):
    c = be_step_new(mesh0, c, dt_t, lam_t, [0.0] * mesh0.n)
z = lam_t * dt_t
disc_exact = (1 + z) ** (-nst)
check("F2 decay only: C^n = (1+z)^-n C^0 to round-off over 300 steps (P21.63 with G = 0)", abs(c[0] - disc_exact) / disc_exact < 1e-12,
      f"rel diff {abs(c[0] - disc_exact) / disc_exact:.1e}")
err_obs = c[0] / math.exp(-lam_t * dt_t * nst) - 1.0
err_pred = 0.5 * lam_t**2 * (dt_t * nst) * dt_t
check("F2 decay error 1/2 lambda^2 t dt (P21.77) within 2 % of the observed error", abs(err_obs / err_pred - 1) < 0.02,
      f"observed {err_obs:.4e}, predicted {err_pred:.4e}")
# P21.64: e^{-z} - 1/(1+z) = -z^2/2 + O(z^3)
zz = sp.symbols("z", positive=True)
ser = sp.series(sp.exp(-zz) - 1 / (1 + zz), zz, 0, 4).removeO()
check("F2 P21.64 e^{-z} - 1/(1+z) = -z^2/2 + O(z^3)", sp.simplify(ser.coeff(zz, 2) + sp.Rational(1, 2)) == 0
      and ser.coeff(zz, 1) == 0 and ser.coeff(zz, 0) == 0)

# ---- worked scalar example for the document ------------------------------------------
lam_ex, dt_ex, c_ex, s_ex = 1.0e-6, 1.0e5, 1.0, 1.0e-7
z_ex = lam_ex * dt_ex
c_be = (c_ex + dt_ex * s_ex) / (1.0 + z_ex)
c_exact = c_ex * math.exp(-z_ex) + s_ex / lam_ex * (1.0 - math.exp(-z_ex))
print(f"INFO  scalar example: z = {z_ex}, BE = {c_be:.7f}, exact = {c_exact:.7f}, diff = {c_be - c_exact:+.2e}")

# ---- F3: decay plus diffusion, no source: discrete identity P21.63 and F_corr -------------
lam3 = 2.0e-8
t_end3 = 2.0e7
nst3 = 100
dtn = t_end3 / nst3
c0 = [1.0 if m == 0 else 0.0 for m in mesh.mat]
c_dec = list(c0)
for _ in range(nst3):
    c_dec = be_step_new(mesh, c_dec, dtn, lam3, [0.0] * mesh.n)
z3 = lam3 * dtn
dtp = dtn / (1 + z3)
c_v1 = list(c0)
for _ in range(nst3):
    c_v1 = be_step_v1(mesh, c_v1, dtp)
c_v1 = [(1 + z3) ** (-nst3) * x for x in c_v1]
d63 = max(abs(a - b) / max(abs(b), 1e-300) for a, b in zip(c_dec, c_v1) if b > 1e-200)
check("F3 P21.63: decay step = (1+z)^-1 x v1.0 step with dt' = dt/(1+z)", d63 < 1e-11, f"max rel diff {d63:.1e}")

# F_corr against the lambda = 0 run (decay-corrected fraction equals the decay-free F, up to O(dt))
def fcorr_error(k):
    m = Mesh(RADII, DIFFS, [c * k for c in CELLS])
    sizes = [t_end3 / (nst3 * k)] * (nst3 * k)
    cc0 = [1.0 if mm == 0 else 0.0 for mm in m.mat]
    _, o_dec = run_fv(m, lam3, cc0, [], sizes)
    _, o_v1 = run_fv(m, 0.0, cc0, [], sizes)
    M0_ = o_dec["M0"]
    f_corr = math.exp(lam3 * t_end3) * o_dec["Mout"][-1] / M0_
    f_v1 = o_v1["Mrel"][-1] / M0_
    return f_corr, f_v1, o_dec["worst_res1"], o_dec["worst_res2"]


fc1, fv1, r11, r12 = fcorr_error(1)
fc2, fv2, r21, r22 = fcorr_error(2)
e1, e2 = abs(fc1 - fv1), abs(fc2 - fv2)
check("F3 F_corr = e^{lam t} M_out/M_0 approaches the decay-free F as dt and h are halved (order about 1)",
      e2 < e1 and math.log2(e1 / e2) > 0.8, f"F_corr {fc1:.6f}/{fc2:.6f}, F(lam=0) {fv1:.6f}/{fv2:.6f}, order {math.log2(e1 / e2):.2f}")
check("F3 closure residuals (P21.72), (P21.75) at round-off", max(r11, r12, r21, r22) < 1e-13,
      f"max relative residual {max(r11, r12, r21, r22):.1e}")

# ---- F4: constant kernel source, no decay: steady state (P21.82 to P21.84) -------------------
def steady_run(k, lam, s_val):
    m = Mesh(RADII, DIFFS, [c * k for c in CELLS])
    c = [0.0] * m.n
    sizes = [1.0e13] * 3
    hist = [(0.0, 1.0e14, s_val)]
    c, o = run_fv(m, lam, c, hist, sizes)
    Gen = V_K * s_val
    return m, c, o, Gen


tau_res = RADII[0] ** 2 / (15.0 * DIFFS[0]) + sum((RADII[i] ** 2 - RADII[i - 1] ** 2) / (6.0 * DIFFS[i]) for i in range(1, 5))
errs = []
for k in (1, 2, 4):
    m, c, o, Gen = steady_run(k, 0.0, 1.0e-6)
    errs.append(abs(inventory(m, c) / Gen / tau_res - 1.0))
    jr = abs(m.outer_cond * c[-1] / Gen - 1.0)
check("F4 J_R/Gen = 1 at the fixed point (lambda = 0), P21.83", jr < 1e-12, f"{jr:.1e}")
ords = [math.log2(errs[i] / errs[i + 1]) for i in range(2)]
check("F4 M_ss/Gen -> tau_res (P21.84), observed spatial order 2", all(1.9 < o_ < 2.1 for o_ in ords),
      f"errors {errs[0]:.2e}, {errs[1]:.2e}, {errs[2]:.2e}; orders {ords[0]:.2f}, {ords[1]:.2f}")

# ---- F5: source plus decay steady state, multilayer (P21.85) ------------------------------
lam5 = 2.0e-8
rb = []
for k in (1, 2, 4):
    m, c, o, Gen = steady_run(k, lam5, 1.0e-6)
    rb.append(m.outer_cond * c[-1] / Gen)
check("F5 R/B steady = 0.9684291531 (P21.85, 60-digit transfer walk in Section 4.7.3) on the (20,8,4,4,4) mesh to 6 digits",
      abs(rb[0] - 0.9684291531) < 2e-6, f"FV {rb[0]:.9f}, finer {rb[1]:.9f}, {rb[2]:.9f}")
o5 = math.log2(abs(rb[0] - 0.9684291531) / abs(rb[1] - 0.9684291531))
check("F5 spatial order 2 of R/B", 1.9 < o5 < 2.1, f"observed {o5:.2f}")
# steady state preserved by BE for any dt (P21.68, unsplit)
m, c_st, o, Gen = steady_run(1, lam5, 1.0e-6)
worst_fp = 0.0
for dtv in (1.0e3, 1.0e5, 2.0e8, 1.0e12):
    c2 = be_step_new(m, c_st, dtv, lam5, [1.0e-6 if mm == 0 else 0.0 for mm in m.mat])
    worst_fp = max(worst_fp, max(abs(a - b) / max(abs(b), 1e-300) for a, b in zip(c2, c_st)))
check("F5/F14 the BE fixed point is unchanged by a step of any size (lambda dt = 0.02 to 2e4)", worst_fp < 1e-11,
      f"max rel change {worst_fp:.1e}")

# ---- F6: Booth sphere with source and decay ---------------------------------------------------
Rb, Db = 175e-6, 1.0e-15
Dp = Db / Rb**2


def booth_mesh(ncell):
    m = Mesh([Rb], [Db], [ncell])
    return m


def rb_closed(x):
    sx = math.sqrt(x)
    return 3.0 / sx * (1.0 / math.tanh(sx) - 1.0 / sx)


V_B = 4.0 / 3.0 * math.pi * Rb**3
for xval in (1.0, 100.0):
    lamb_ = xval * Dp
    errs_b = []
    for ncell in (100, 200, 400):
        m = booth_mesh(ncell)
        c, o = run_fv(m, lamb_, [0.0] * m.n, [(0.0, 1e30, 1.0e-6)], [1.0e13] * 3)
        rb_fv = m.outer_cond * c[-1] / (V_B * 1.0e-6)
        errs_b.append(abs(rb_fv / rb_closed(xval) - 1))
    ob = [math.log2(errs_b[i] / errs_b[i + 1]) for i in range(2)]
    check(f"F6 Booth R/B steady vs (P21.86) at x = {xval:g}: order 2 in h", all(1.9 < o_ < 2.1 for o_ in ob),
          f"rel errors at N = 100, 200, 400: {errs_b[0]:.2e}, {errs_b[1]:.2e}, {errs_b[2]:.2e}; orders {ob[0]:.2f}, {ob[1]:.2f}")

# transient F_B(t) = M_out/(M + M_out) against the P21.87 / TECDOC-1674 (7.8) series, x = 1
xv = 1.0
lamb = xv * Dp


def fb_series(t, nmax=20000):
    n = np.arange(1, nmax + 1)
    a = 6.0 / (n**2 * math.pi**2 + xv)
    s1 = rb_closed(xv)  # closed form (P21.86); the plain sum has a tail of about 6/(pi^2 nmax)
    b = (6.0 / math.pi**2) * xv / np.expm1(lamb * t) * np.sum((-np.expm1(-(n**2) * math.pi**2 * Dp * t)) / (n**2 * (n**2 * math.pi**2 + xv)))
    return s1 - b


fb_err = []
for k in (1, 2, 4):
    m = booth_mesh(100)
    sizes = time_grid([0.0, 2.0e7], 2.0e4 / k, 4.0e5 / k)
    c, o = run_fv(m, lamb, [0.0] * m.n, [(0.0, 1e30, 1.0e-6)], sizes)
    fb_fv = o["Mout"][-1] / (o["M"][-1] + o["Mout"][-1])
    fb_err.append(abs(fb_fv - fb_series(o["t"][-1])))
ordfb = math.log2(fb_err[1] / fb_err[2])
check("F6 transient F_B = M_out/(M + M_out) against the (7.8) series: first order in dt (spatial error fixed at N = 100)",
      fb_err[2] < 2e-3 and 0.8 < ordfb < 1.3, f"errors {fb_err[0]:.2e}, {fb_err[1]:.2e}, {fb_err[2]:.2e}, order {ordfb:.2f}")

# ---- F9, F10, F13: closure, positivity, switch inside a step --------------------------------
lam9 = 2.0e-8
sizes9 = time_grid([0.0, 1.0e7, 3.0e7], 1.0e4, 5.0e5)
c0_9 = [1.0e-3 if mm == 0 else 0.0 for mm in mesh.mat]
c9, o9 = run_fv(mesh, lam9, c0_9, [(0.0, 1.0e7, 1.0e-6)], sizes9)
check("F9 closure residuals (P21.72) and (P21.75), switching source, variable steps",
      o9["worst_res1"] < 1e-13 and o9["worst_res2"] < 1e-13, f"{o9['worst_res1']:.1e}, {o9['worst_res2']:.1e}")
check("F10 positivity: min C >= 0.0 exactly at every step (P21.62)", min(o9["minC"]) >= 0.0, f"min {min(o9['minC']):.2e}")
minc_all = []
for dtv in (1e-3, 1.0, 1e3, 1e6, 1e9, 1e12):
    cc = list(c0_9)
    for _ in range(5):
        cc = be_step_new(mesh, cc, dtv, 2.0e-8, [1.0e-6 if mm == 0 else 0.0 for mm in mesh.mat])
    minc_all.append(min(cc))
check("F10 positivity for dt = 1e-3 to 1e12 s", min(minc_all) >= 0.0, f"smallest value {min(minc_all):.2e}")

# switch inside a step: step boundaries NOT aligned with t_sw = 1.0e7
sw_sizes = [3.3e6] * 9 + [1.0e5]  # sum = 2.98e7 + 1e5 ; switch inside a step
tot = sum(sw_sizes)
sw_sizes = [s * 3.0e7 / tot for s in sw_sizes]
cc, osw = run_fv(mesh, 0.0, [0.0] * mesh.n, [(0.0, 1.0e7, 1.0e-6)], sw_sizes)
Mgen_exact = V_K * 1.0e-6 * 1.0e7
mg_a = osw["Mgen"][-1]
# rectangle rule for comparison: source value at the END of the step
cc2 = [0.0] * mesh.n
kk = Kahan()
t = 0.0
for dt in sw_sizes:
    t += dt
    kk.add(dt * V_K * (1.0e-6 if t <= 1.0e7 else 0.0))
check("F13 step average (A): M_gen exact to round-off with a switch inside a step", abs(mg_a / Mgen_exact - 1) < 1e-13,
      f"rel err {abs(mg_a / Mgen_exact - 1):.1e}")
check("F13 rectangle rule (R) misplaces inventory (error of order |dS| V_K dt)", abs(kk.s / Mgen_exact - 1) > 1e-3,
      f"rel err {abs(kk.s / Mgen_exact - 1):.3f}")

# ---- F15: decay-exact split as cross-check (S = 0) -------------------------------------------
c_s = list(c0)
dts = 2.0e5
for n_ in range(50):
    c_s = be_step_v1(mesh, c_s, dts)
    c_s = [math.exp(-lam3 * dts) * x for x in c_s]
c_r = list(c0)
for n_ in range(50):
    c_r = be_step_v1(mesh, c_r, dts)
c_r = [math.exp(-lam3 * dts * 50) * x for x in c_r]
d15 = max(abs(a - b) / max(abs(b), 1e-300) for a, b in zip(c_s, c_r) if b > 1e-200)
check("F15 split step (v1.0 step, then exact decay) = e^{-lam t} x v1.0 run (P21.67)", d15 < 1e-12, f"{d15:.1e}")

# ---------------------------------------------------------------------------
# Block E: modal solution (kernel-uniform source and initial inventory)
# ---------------------------------------------------------------------------


def block_e_modal(Lam, w, lam, M0, G_hist, times):
    """Return dict of M, Mdec, Mrel (balance form), X, Mout, FB at `times`.
    Lam, w: eigenvalues and v1.0 weights; G_hist: [(t_start, t_end, G)] in mol/s."""
    kappa = Lam + lam
    m = M0 * w  # P21.46: 4 pi a_n(0) I_n = M0 w_n
    X = M0
    Mdec = 0.0
    Mgen = 0.0
    t = 0.0
    out = []
    pts = sorted(set([0.0] + list(times) + [a for (a, b, g_) in G_hist] + [b for (a, b, g_) in G_hist if b < max(times)]))
    pts = [p for p in pts if p <= max(times) + 1e-9]
    tset = set(times)
    for a, b in zip(pts[:-1], pts[1:]):
        Gv = 0.0
        for (s0, s1, gv) in G_hist:
            if s0 <= a and b <= s1:
                Gv = gv
        D = b - a
        xk = kappa * D
        p1 = np.array([phi1(v) for v in xk])
        p2 = np.array([phi2(v) for v in xk])
        integ = D * (m * p1 + Gv * w * D * p2)
        m = m * np.exp(-xk) + Gv * w * D * p1
        Mdec += lam * integ.sum()
        Mgen += Gv * D
        X = X * math.exp(-lam * D) + Gv * D * phi1(lam * D)
        if any(abs(b - tt) < 1e-6 * max(tt, 1) for tt in tset):
            M = m.sum()
            Mrel = M0 + Mgen - M - Mdec
            out.append((b, M, Mdec, Mrel, X, X - M, Mgen))
    return out


# five-layer eigen-solution (independent Multilayer helper)
ml = Multilayer(RADII, DIFFS)
NMODE = 300
Lam = ml.eigenvalues(NMODE)
Nn, In, I1n, _ = ml.mode_integrals(Lam, [0])
w = 3.0 * I1n * In / (RADII[0] ** 3 * Nn)
check("E weights: sum_n w_n -> 1 (completeness of the kernel projection)", abs(w.sum() - 1.0) < 2e-3,
      f"1 - sum w_n = {1 - w.sum():.2e} for N = {NMODE}")
check("E weights change sign for the layered particle (so the tail bound by w_n >= 0 is not rigorous)",
      np.any(w < 0), f"{int(np.sum(w < 0))} negative weights in {NMODE} modes")

# (i) the recursion equals a direct ODE integration of the modal system
lam_e = 2.0e-8
hist_e = [(0.0, 1.0e7, 3.0e-17)]  # G in mol/s, SYNTHETIC
nm_t = 25
kap = Lam[:nm_t] + lam_e
w_t = w[:nm_t]


s1 = solve_ivp(lambda t, y: -kap * y + 3.0e-17 * w_t, [0.0, 1.0e7], np.zeros(nm_t), method="Radau", rtol=1e-12, atol=1e-32)
s2 = solve_ivp(lambda t, y: -kap * y, [1.0e7, 2.5e7], s1.y[:, -1], method="Radau", rtol=1e-12, atol=1e-32)
res = block_e_modal(Lam[:nm_t], w_t, lam_e, 0.0, hist_e, [1.0e7, 2.5e7])
dm1 = abs(res[0][1] - s1.y[:, -1].sum()) / s1.y[:, -1].sum()
dm2 = abs(res[1][1] - s2.y[:, -1].sum()) / s2.y[:, -1].sum()
check("E piecewise-constant modal recursion equals ODE integration of the modal system (25 modes)", max(dm1, dm2) < 1e-8,
      f"rel diff {dm1:.1e}, {dm2:.1e}")

# (ii) closure: M_rel from the balance form is positive, monotone and consistent with M_out + decayed-outside
# (iii) modal against FV for five layers, with refinement
M0_e = 0.0
t_probe = [1.0e7, 2.0e7, 3.0e7]
Gv = V_K * 1.0e-6
modal = block_e_modal(Lam, w, lam_e, M0_e, [(0.0, 1.0e7, Gv)], t_probe)
errs = {}
for k in (1, 2, 4):
    m = Mesh(RADII, DIFFS, [c * k for c in CELLS])
    sizes = time_grid([0.0, 1.0e7, 2.0e7, 3.0e7], 1.0e4 / k, 5.0e5 / k)
    cfv, ofv = run_fv(m, lam_e, [0.0] * m.n, [(0.0, 1.0e7, 1.0e-6)], sizes)
    for (tt, M, Mdec, Mrel, X, Mout, Mgen) in modal:
        for name, mod in (("M", M), ("Mrel", Mrel), ("Mdec", Mdec), ("Mout", Mout)):
            fvv = at(ofv, name, tt)
            errs.setdefault((name, tt), []).append(abs(fvv - mod) / (Gv * 1.0e7))
worst_final = max(v[2] for v in errs.values())
orders = [math.log2(v[1] / v[2]) for v in errs.values()]
check("E modal vs FV (five layers, source then decay): worst difference / M_gen(t_irr) at the finest level < 2e-3 for M, M_rel, M_dec, M_out",
      worst_final < 2e-3, f"worst {worst_final:.2e}")
check("E every halving of dt and h reduces the difference by a factor of at least 1.8 (first-order BE)", min(orders) > math.log2(1.8) and min(v[0] / v[1] for v in errs.values()) > 1.8,
      f"observed orders {min(orders):.2f} to {max(orders):.2f}")
M_t = {r[0]: r for r in modal}
print(f"INFO  modal five-layer case: t = 1e7 s: M = {modal[0][1]:.6e}, M_rel = {modal[0][3]:.6e}, M_dec = {modal[0][2]:.6e}, M_out = {modal[0][5]:.6e} mol")

# (iv) M_out via P21.44 equals M_rel when lambda = 0
res0 = block_e_modal(Lam, w, 0.0, 0.0, [(0.0, 1.0e7, Gv)], [1.0e7, 3.0e7])
check("E lambda = 0: M_out = M_rel (P21.76)", all(abs(r[5] - r[3]) < 1e-12 * abs(r[3]) for r in res0))

# (v) Booth: modal F_B against the (7.8) series, x = 1, irradiation from zero
Dpb = Dp
Lam_b = np.array([n**2 * math.pi**2 * Dpb for n in range(1, 4001)])
w_b = np.array([6.0 / (n**2 * math.pi**2) for n in range(1, 4001)])
tq = 2.0e7
rb_m = block_e_modal(Lam_b, w_b, lamb, 0.0, [(0.0, 1e30, 1.0)], [tq])
fb_mod = rb_m[0][5] / rb_m[0][4]
check("E Booth modal F_B = M_out/X against the (7.8) series, x = 1, 4000 modes", abs(fb_mod - fb_series(tq)) < 5e-9,
      f"modal {fb_mod:.10f}, series {fb_series(tq):.10f}")
# truncation order of the balance form for M_rel (homogeneous sphere): N^-3 or better
def mrel_err(nm):
    r = block_e_modal(Lam_b[:nm], w_b[:nm], lamb, 0.0, [(0.0, 1e30, 1.0)], [tq])[0]
    return r[3]


ref = mrel_err(4000)
e10, e100 = abs(mrel_err(10) - ref), abs(mrel_err(100) - ref)
check("E balance-form M_rel truncation error falls at least as N^-2.5 between N = 10 and 100 (sign-definite w_n)",
      math.log10(e10 / e100) > 2.5, f"errors {e10:.2e}, {e100:.2e}, order {math.log10(e10 / e100):.2f}")

# ---------------------------------------------------------------------------
# F11, F12: CRP-6 sensitivity cases 1a and 1b (no source, no decay)
# ---------------------------------------------------------------------------
Rg = 8.3145


def D_crp6(T):
    return 5.6e-8 * math.exp(-209000.0 / (Rg * T)) + 5.2e-4 * math.exp(-362000.0 / (Rg * T))


rk = 250e-6


def crp6_fv(T, dt_step):
    D = D_crp6(T)
    m = Mesh([rk], [D], [150])
    n_st = int(round(720000.0 / dt_step))
    cfin, o = run_fv(m, 0.0, [1.0] * m.n, [], [720000.0 / n_st] * n_st)
    return D, o["Mrel"][-1] / o["M0"]


D1a, F1a = crp6_fv(1473.15, 360.0)
check("F11 CRP-6 1a: FV F at 200 h (dt = 360 s, N = 150) agrees with the Section 4.7.6 value 0.46737 and the series 0.46742",
      abs(F1a - 0.46737) < 4e-5 and abs(F1a - 0.46742) < 1e-4, f"F = {F1a:.5f}, D't = {D1a / rk**2 * 720000:.5f}")

# 1b: reference is the Booth series 1 - F = (6/pi^2) sum exp(-n^2 pi^2 D't)/n^2
D1b = D_crp6(1873.15)
Dpt = D1b / rk**2 * 720000.0
ref_1b = float(6.0 / mp.pi**2 * mp.nsum(lambda n: mp.e ** (-(n**2) * mp.pi**2 * Dpt) / n**2, [1, mp.inf]))
_, F360 = crp6_fv(1873.15, 360.0)
_, F36 = crp6_fv(1873.15, 36.0)
e360, e36 = (1 - F360) / ref_1b - 1, (1 - F36) / ref_1b - 1
pred360 = 0.5 * (math.pi**2 * D1b / rk**2) ** 2 * 720000.0 * 360.0  # P21.78 with lambda = 0
check("F12 CRP-6 1b: (1-F) from the series is 4.0703e-7 (D't = 1.4405)", abs(ref_1b / 4.0703e-7 - 1) < 1e-3,
      f"{ref_1b:.4e}, D't = {Dpt:.4f}")
check("F12 CRP-6 1b: FV 1-F within 5 % of the series at dt = 36 s (N = 150)", abs(e36) < 0.05,
      f"1-F = {1 - F36:.4e}, relative error {e36:+.3f}")
check("F12 CRP-6 1b: the error at dt = 360 s follows (P21.78): observed vs predicted", abs(e360 - pred360) < 0.006,
      f"1-F = {1 - F360:.4e}, observed {e360:+.4f}, predicted {pred360:.4f}")

# ---------------------------------------------------------------------------
n_pass = sum(RESULTS)
print(f"SUMMARY: {n_pass}/{len(RESULTS)} PASS")
if n_pass != len(RESULTS):
    sys.exit(1)
print("ALL PASS")
