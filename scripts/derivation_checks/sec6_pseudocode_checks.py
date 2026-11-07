"""
sec6_pseudocode_checks.py

Checks for docs/mathematics/STRATO_Phase2_Pseudocode.md.

The pseudocode document introduces no new physics. It re-arranges results that
were derived and checked in Sections 3 to 5 (equations P21.x and P22.x) into
algorithm blocks, and it adds a few helper relations (H1 to H6) that make the
algorithms numerically safe. This script

  1. proves the helper relations with SymPy (residual = 0, limits), and
  2. runs a literal Python transcription of the pseudocode blocks
     (B3 Arrhenius, B4 temperature history and step control, B5 the Backward
     Euler step, B6 bookkeeping, B7 the time loop, B8 modal solution,
     B9 effective time, B10 projection) and checks the pass criteria of the
     verification blocks that can be tested without Rust.

It is independent of the Rust code in src/. All data are synthetic test
values except where a source is named in a comment. It writes no files.

Run:
  .venv/Scripts/python.exe -I -X utf8 sec6_pseudocode_checks.py
"""

import math

import numpy as np
import sympy as sp
from scipy.integrate import quad
from scipy.linalg import eigh, expm
from scipy.special import expn

RESULTS = []


def check(label, ok, detail=""):
    """Print one PASS/FAIL line and remember the outcome."""
    RESULTS.append(bool(ok))
    tail = f"  {detail}" if detail else ""
    print(f"{'PASS' if ok else 'FAIL'}  {label}{tail}")


# ---------------------------------------------------------------------------
# Part 1. SymPy proofs of the helper relations H1 to H6
# ---------------------------------------------------------------------------
x, dt, a0, s, kap, t, lam, G = sp.symbols("x Delta a0 s kappa t lambda G", positive=True)


def phi1(z):
    """(1 - e^{-z}) / z, the exact 'fraction of a step' factor (H1)."""
    return (1 - sp.exp(-z)) / z


def phi2(z):
    """(z - 1 + e^{-z}) / z^2, used for the time integral of a mode (H2)."""
    return (z - 1 + sp.exp(-z)) / z**2


# Modal ODE with constant source over one interval: a' = -kappa a + s, a(0) = a0.
a_t = a0 * sp.exp(-kap * t) + s * (1 - sp.exp(-kap * t)) / kap
check("H0 constant-source mode solves a' = -kappa a + s (P21.32)",
      sp.simplify(sp.diff(a_t, t) + kap * a_t - s) == 0)

# H1: end-of-interval amplitude written with phi1 (same as P21.33).
h1 = a0 * sp.exp(-kap * dt) + s * dt * phi1(kap * dt)
check("H1 a(t_j) = a0 e^{-kappa Delta} + s Delta phi1(kappa Delta) equals (P21.33)",
      sp.simplify(a_t.subs(t, dt) - h1) == 0)

# H2: time integral of the mode over the interval (per-interval form of P21.38).
int_a = sp.integrate(a_t, (t, 0, dt))
h2 = dt * (a0 * phi1(kap * dt) + s * dt * phi2(kap * dt))
check("H2 int_0^Delta a dt = Delta [a0 phi1 + s Delta phi2] (P21.38 per interval)",
      sp.simplify(int_a - h2) == 0)

# Small-argument series used when kappa Delta is tiny (avoids cancellation).
ser1 = sp.series(phi1(x), x, 0, 4).removeO()
ser2 = sp.series(phi2(x), x, 0, 4).removeO()
check("H3 phi1 = 1 - x/2 + x^2/6 - x^3/24 + O(x^4)",
      sp.simplify(ser1 - (1 - x / 2 + x**2 / 6 - x**3 / 24)) == 0)
check("H3 phi2 = 1/2 - x/6 + x^2/24 - x^3/120 + O(x^4)",
      sp.simplify(ser2 - (sp.Rational(1, 2) - x / 6 + x**2 / 24 - x**3 / 120)) == 0)
check("H1/H2 limit kappa -> 0: a = a0 + s Delta, int a = a0 Delta + s Delta^2/2",
      sp.limit(h1, kap, 0) == a0 + s * dt
      and sp.simplify(sp.limit(h2, kap, 0) - (a0 * dt + s * dt**2 / 2)) == 0)

# H4: surviving total X = M + M_out obeys X' = G - lambda X (P21.44); recursion.
X0 = sp.symbols("X0", positive=True)
X_t = X0 * sp.exp(-lam * t) + G * (1 - sp.exp(-lam * t)) / lam
check("H4 X = M + M_out solves X' = G - lambda X (P21.44)",
      sp.simplify(sp.diff(X_t, t) - (G - lam * X_t)) == 0)
check("H4 recursion X_j = X_{j-1} e^{-lambda Delta} + G Delta phi1(lambda Delta)",
      sp.simplify(X_t.subs(t, dt) - (X0 * sp.exp(-lam * dt) + G * dt * phi1(lam * dt))) == 0)

# H5: structural shared-scaling test (E1). Same Q set, D0 ratio independent of m
# gives a T-independent ratio D_2/D_1; a different Q gives a T-dependent ratio.
T, Rg, Q1, Q2, Q3, d1, d2, k1, k2 = sp.symbols("T R_g Q1 Q2 Q3 d1 d2 k1 k2", positive=True)
law = d1 * sp.exp(-Q1 / (Rg * T)) + d2 * sp.exp(-Q2 / (Rg * T))
ratio_shared = (k2 * law) / (k1 * law)
check("H5 same Q set and m-independent D0 ratio => D_2/D_1 independent of T",
      sp.simplify(sp.diff(ratio_shared, T)) == 0)
ratio_diff = (k2 * d1 * sp.exp(-Q3 / (Rg * T))) / (k1 * d1 * sp.exp(-Q1 / (Rg * T)))
dr = sp.simplify(sp.diff(ratio_diff, T))
check("H5 different Q => d(D_2/D_1)/dT = ratio (Q3 - Q1)/(R_g T^2), nonzero for Q3 != Q1",
      sp.simplify(dr - ratio_diff * (Q3 - Q1) / (Rg * T**2)) == 0)

# H6: antiderivative used for tau(t) on a ramp, d/dT [T E2(theta/T)] = e^{-theta/T} (P22.22).
th = sp.symbols("theta", positive=True)
expr = sp.diff(T * sp.expint(2, th / T), T)
expr = expr.subs(sp.expint(2, th / T), sp.exp(-th / T) - (th / T) * sp.expint(1, th / T))
check("H6 d/dT [T E2(theta/T)] = exp(-theta/T) (P22.22, used by B9)",
      sp.simplify(expr - sp.exp(-th / T)) == 0)


# ---------------------------------------------------------------------------
# Part 2. Python transcription of the pseudocode blocks
# ---------------------------------------------------------------------------
RG = 8.31446261815324          # J mol^-1 K^-1, N_A k_B (exact SI value)
YEAR_S = 31556925.97            # s per year used by the LiveChart API (audit 07)
GL8_X, GL8_W = np.polynomial.legendre.leggauss(8)


# ---- B1 decay constant ----------------------------------------------------
def decay_constant(t_half_s):
    """B1: lambda = ln 2 / t_half (P21.7)."""
    if not (math.isfinite(t_half_s) and t_half_s > 0.0):
        raise ValueError("half-life must be finite and > 0")
    return math.log(2.0) / t_half_s


lam_cs = decay_constant(30.08 * YEAR_S)
check("B1 lambda(Cs-137, 30.08 y) reproduces the Section 3 value 7.302187e-10 s^-1",
      abs(lam_cs / 7.302187e-10 - 1.0) < 1e-6, f"lambda = {lam_cs:.7e}")


# ---- B2 source cell average (P21.54) ----------------------------------------
def cell_average_kernel_source(left, right, R1, SK):
    """B2: exact cell average of a source uniform in r < R1 (P21.54)."""
    if right <= R1:
        return SK
    if left >= R1:
        return 0.0
    return SK * (R1**3 - left**3) / (right**3 - left**3)


rng = np.random.default_rng(7)
faces = np.sort(np.concatenate(([0.0, 400e-6], rng.uniform(0, 400e-6, 30))))
R1t, SKt = 173.3e-6, 2.5e-7
gen = sum(4 * math.pi / 3 * (b**3 - a**3) * cell_average_kernel_source(a, b, R1t, SKt)
          for a, b in zip(faces[:-1], faces[1:]))
check("B2 sum V_j Sbar_j = (4 pi/3) R1^3 S_K on a mesh with a straddling cell",
      abs(gen / (4 * math.pi / 3 * R1t**3 * SKt) - 1) < 1e-13)


# ---- B3 Arrhenius diffusivity ----------------------------------------------
def diffusivity(branches, T_K):
    """B3: D(T) = sum_m D0 exp(-Q/(R_g T)) by log-sum-exp (P22.1).

    branches: list of (D0 [m^2/s], Q [J/mol]); a one-item list with Q = 0 is
    a constant diffusivity (the v1.0 case).
    """
    if not (math.isfinite(T_K) and T_K > 0.0):
        raise ValueError("temperature must be finite and > 0 K")
    if len(branches) == 1 and branches[0][1] == 0.0:
        return branches[0][0]                       # constant D: no exp, bit-exact
    logs = [math.log(d0) - q / (RG * T_K) for d0, q in branches]
    top = max(logs)
    return math.exp(top) * sum(math.exp(v - top) for v in logs)


def q_eff(branches, T_K):
    """B3: apparent activation energy (P22.6)."""
    ds = [d0 * math.exp(-q / (RG * T_K)) for d0, q in branches]
    return sum(q * d for (_, q), d in zip(branches, ds)) / sum(ds)


# ---- B4 temperature history --------------------------------------------------
class TemperatureHistory:
    """B4: breakpoints (t_k, T_k); a repeated time is a jump; linear in between (P22.21)."""

    def __init__(self, points):
        self.p = points
        for (ta, _), (tb, _) in zip(points[:-1], points[1:]):
            if tb < ta:
                raise ValueError("breakpoint times must not decrease")
        # segments with positive length
        self.seg = [(i, i + 1) for i in range(len(points) - 1) if points[i + 1][0] > points[i][0]]

    def segment(self, ta, tb):
        """Return the segment that contains the whole step [ta, tb]."""
        for i, j in self.seg:
            if self.p[i][0] <= ta and tb <= self.p[j][0]:
                return i, j
        raise ValueError("step crosses a breakpoint")

    def T_in(self, seg, tt):
        i, j = seg
        (t0, T0), (t1, T1) = self.p[i], self.p[j]
        return T0 + (T1 - T0) * (tt - t0) / (t1 - t0)

    def slope(self, seg):
        i, j = seg
        return (self.p[j][1] - self.p[i][1]) / (self.p[j][0] - self.p[i][0])

    def events(self):
        return sorted({tt for tt, _ in self.p if tt > 0.0})


def step_diffusivity(branches, hist, ta, tb, option):
    """B4/B5: diffusivity used for one step, option 'A' (step average) or 'E' (end of step)."""
    seg = hist.segment(ta, tb)
    Ta, Tb = hist.T_in(seg, ta), hist.T_in(seg, tb)
    if option == "E" or Ta == Tb:
        return diffusivity(branches, Tb)            # constant T: no quadrature, bit-exact
    mid, half = 0.5 * (ta + tb), 0.5 * (tb - ta)
    vals = [diffusivity(branches, hist.T_in(seg, mid + half * xx)) for xx in GL8_X]
    return 0.5 * float(np.dot(GL8_W, vals))


# ---- mesh geometry and conductances (v1.0 build_mesh, split in two) ---------
def build_geometry(radii, cells):
    out, inner = [], 0.0
    for mat, (Rout, n) in enumerate(zip(radii, cells)):
        width = (Rout - inner) / n
        for k in range(n):
            left = inner + k * width
            right = Rout if k + 1 == n else inner + (k + 1) * width
            cen = 0.75 * right if left == 0.0 else 0.75 * (right**4 - left**4) / (right**3 - left**3)
            out.append(dict(left=left, right=right, centroid=cen,
                            volume=4.0 * math.pi / 3.0 * (right**3 - left**3), material=mat))
        inner = Rout
    return out


def conductances(geom, D_layer):
    """G_f for every internal face and G_R at r = R from per-layer D (P22.14)."""
    G = []
    for L, Rc in zip(geom[:-1], geom[1:]):
        face = L["right"]
        area = 4.0 * math.pi * face * face
        dl, drr = face - L["centroid"], Rc["centroid"] - face
        G.append(area / (dl / D_layer[L["material"]] + drr / D_layer[Rc["material"]]))
    last, R = geom[-1], geom[-1]["right"]
    GR = 4.0 * math.pi * R * R * D_layer[last["material"]] / (R - last["centroid"])
    return G, GR


def solve_diffusion_system(v, gw, ge, rhs):
    """v1.0 subtraction-free elimination (P1.71 to P1.73, P21.61)."""
    n = len(v)
    den, dp, e_prev = [0.0] * n, [0.0] * n, 0.0
    for i in range(n):
        e = v[0] if i == 0 else v[i] + gw[i] * e_prev / (e_prev + gw[i])
        den[i] = e + ge[i]
        carry = 0.0 if i == 0 else gw[i] * dp[i - 1]
        dp[i] = (rhs[i] + carry) / den[i]
        e_prev = e
    xs = [0.0] * n
    xs[-1] = dp[-1]
    for i in range(n - 2, -1, -1):
        xs[i] = dp[i] + ge[i] / den[i] * xs[i + 1]
    return xs


def west_east(G, GR):
    gw = [0.0] + G
    ge = G + [GR]
    return gw, ge


def v1_step(geom, G, GR, old, dtt):
    """Literal transcription of v1.0 backward_euler_step."""
    gw, ge = west_east(G, GR)
    v = [c["volume"] / dtt for c in geom]
    rhs = [vi * o for vi, o in zip(v, old)]
    return solve_diffusion_system(v, gw, ge, rhs)


def v2_step(geom, G, GR, old, dtt, lam_, sbar):
    """B5: one Backward Euler step with decay and source (P21.58, P21.59, P22.15)."""
    gw, ge = west_east(G, GR)
    v = [c["volume"] / dtt + lam_ * c["volume"] for c in geom]
    rhs = [(c["volume"] / dtt) * o + c["volume"] * sb for c, o, sb in zip(geom, old, sbar)]
    return solve_diffusion_system(v, gw, ge, rhs)


class Kahan:
    def __init__(self):
        self.s, self.c = 0.0, 0.0

    def add(self, val):
        y = val - self.c
        tt = self.s + y
        self.c = (tt - self.s) - y
        self.s = tt


def source_step_average(schedule, ta, tb):
    """B2: exact step average of a piecewise-constant S_K(t) (P21.57 option A).

    schedule: list of (t_start, S_K), sorted, first t_start = 0.
    """
    total = 0.0
    for k, (t0, sk) in enumerate(schedule):
        t1 = schedule[k + 1][0] if k + 1 < len(schedule) else math.inf
        lo, hi = max(ta, t0), min(tb, t1)
        if hi > lo:
            total += sk * (hi - lo)
    return total / (tb - ta)


# ---- B7 the FV time loop ---------------------------------------------------
def run_fv(case, option="A", align_source=True, outputs=None, fixed_GR=None):
    """B7: v2.0 time loop. Returns final state and diagnostics.

    fixed_GR: if given, release is booked with this constant surface conductance
    (the v1.0 behaviour) instead of the step's G_R; used only to show that the
    v1.0 booking breaks the balance when D changes in time (V19).
    """
    geom = build_geometry(case["radii"], case["cells"])
    hist, laws, lam_ = case["hist"], case["laws"], case["lam"]
    sched = case["source"]
    R1 = case["radii"][0]
    c = [case["c0"] if g["material"] == 0 else 0.0 for g in geom]
    V = [g["volume"] for g in geom]
    M0 = sum(vv * cc for vv, cc in zip(V, c))
    t_end = case["t_end"]
    outputs = outputs if outputs is not None else [t_end]
    jump_events = set(hist.events()) | ({tt for tt, _ in sched if tt > 0} if align_source else set())
    events = sorted(jump_events | set(outputs) | {t_end})
    acc = {k: Kahan() for k in ("gen", "dec", "rel", "decout")}
    M_out, tnow, dt_prev = 0.0, 0.0, case["dt_restart"]
    visited, max_res, max_res_out, cmin = [], 0.0, 0.0, min(c)
    while tnow < t_end:
        # --- choose the step (B4) ---
        e_next = min(e for e in events if e > tnow)
        seg = hist.segment(tnow, e_next)
        dt_t = min(case["dt_max"], case["growth"] * dt_prev)
        slope = abs(hist.slope(seg))
        if slope > 0.0:                             # D-change limit (P22.20)
            Tn = hist.T_in(seg, tnow)
            qmax = max(q_eff(br, Tn) for br in laws)
            if qmax > 0.0:
                dt_t = min(dt_t, case["eps_D"] * RG * Tn**2 / (qmax * slope))
        rem = e_next - tnow
        if rem <= dt_t * (1.0 + 1e-12):
            tnext = e_next                         # land exactly on the event
        elif rem < 2.0 * dt_t:
            tnext = tnow + 0.5 * rem               # avoid a sliver step
        else:
            tnext = tnow + dt_t
        h = tnext - tnow
        # --- step diffusivities and conductances (B3, B4) ---
        D_layer = [step_diffusivity(br, hist, tnow, tnext, option) for br in laws]
        G, GR = conductances(geom, D_layer)
        # --- step-averaged source, exact cell average (B2) ---
        SK = source_step_average(sched, tnow, tnext)
        sbar = [cell_average_kernel_source(g["left"], g["right"], R1, SK) for g in geom]
        # --- solve (B5) ---
        c = v2_step(geom, G, GR, c, h, lam_, sbar)
        # --- bookkeeping (B6) ---
        M = sum(vv * cc for vv, cc in zip(V, c))
        Gen = sum(vv * sb for vv, sb in zip(V, sbar))
        JR = (GR if fixed_GR is None else fixed_GR) * c[-1]
        acc["gen"].add(h * Gen)
        acc["dec"].add(h * lam_ * M)
        acc["rel"].add(h * JR)
        M_out = (M_out + h * JR) / (1.0 + lam_ * h)
        acc["decout"].add(h * lam_ * M_out)
        scale = M0 + acc["gen"].s
        max_res = max(max_res, abs(M + acc["dec"].s + acc["rel"].s - scale) / scale)
        if acc["rel"].s > 0:
            max_res_out = max(max_res_out,
                              abs(acc["rel"].s - (M_out + acc["decout"].s)) / acc["rel"].s)
        cmin = min(cmin, min(c))
        visited.append(tnext)
        if tnext in jump_events:
            dt_prev = case["dt_restart"]           # restart small after a kink or switch
        else:
            dt_prev = h
        tnow = tnext
    return dict(c=c, M=M, M0=M0, Mgen=acc["gen"].s, Mdec=acc["dec"].s, Mrel=acc["rel"].s,
                Mout=M_out, res=max_res, res_out=max_res_out, cmin=cmin,
                visited=visited, events=events, geom=geom)


# ---- V1 bitwise regression ---------------------------------------------------
radii5 = [175e-6, 275e-6, 315e-6, 350e-6, 390e-6]          # synthetic
cells5 = [20, 8, 4, 4, 4]
Dconst = [4e-14, 4e-13, 2e-14, 5e-15, 2e-14]                # synthetic
geom = build_geometry(radii5, cells5)
G0, GR0 = conductances(geom, Dconst)
c1 = [1.0 if g["material"] == 0 else 0.0 for g in geom]
c2 = list(c1)
rel1, rel2 = Kahan(), Kahan()
same = True
for _ in range(200):
    c1 = v1_step(geom, G0, GR0, c1, 3600.0)
    c2 = v2_step(geom, G0, GR0, c2, 3600.0, 0.0, [0.0] * len(geom))
    rel1.add(3600.0 * GR0 * c1[-1])
    rel2.add(3600.0 * GR0 * c2[-1])
    same = same and all(a == b for a, b in zip(c1, c2))
check("V1 lambda = 0, S = 0: v2.0 step equals v1.0 step bit for bit over 200 steps",
      same and rel1.s == rel2.s)

# ---- V9, V10, V13, V19, V20 on a five-layer Arrhenius case -------------------
laws5 = [                                                   # synthetic test laws
    [(5.6e-8, 209e3), (5.2e-4, 362e3)],                     # shape of P21.91 (Cs in UO2)
    [(1e-8, 0.0)],                                          # constant
    [(1e-6, 250e3)],
    [(5e-12, 150e3), (1e-2, 500e3)],
    [(1e-6, 250e3)],
]
hist5 = TemperatureHistory([(0.0, 1523.15), (7200.0, 1873.15), (21600.0, 1873.15),
                            (21600.0, 2073.15), (36000.0, 2073.15)])  # ramp, hold, jump, hold
case5 = dict(radii=radii5, cells=cells5, laws=laws5, hist=hist5, lam=1e-5,
             source=[(0.0, 1e-6), (10800.0, 0.0), (25200.0, 5e-7)],
             c0=1.0, t_end=36000.0, dt_max=300.0, dt_restart=1.0, growth=1.5, eps_D=0.1)
out = run_fv(case5, option="A", outputs=[3600.0 * k for k in range(1, 11)])
check("V9/V19 closure M0 + Mgen = M + Mdec + Mrel at every step (ramp, jump, switching source)",
      out["res"] < 1e-12, f"max rel residual {out['res']:.1e}")
check("V9 outside closure Mrel = Mout + Mdec,out at every step (P21.75)",
      out["res_out"] < 1e-12, f"max rel residual {out['res_out']:.1e}")
check("V10 min C >= 0.0 exactly over the run (P21.62, P22.17)", out["cmin"] >= 0.0,
      f"min C = {out['cmin']:.3e}")
D_init = [diffusivity(br, hist5.p[0][1]) for br in laws5]
_, GR_init = conductances(build_geometry(radii5, cells5), D_init)
out_fixed = run_fv(case5, option="A", fixed_GR=GR_init)
check("V19 negative test: fixed initial G_R (v1.0 booking) fails the 1e-12 closure criterion",
      out_fixed["res"] > 1e-8, f"max rel residual {out_fixed['res']:.1e} (vs {out['res']:.1e})")
landed = all(e in out["visited"] for e in out["events"])
check("V20 every breakpoint, switch and output time is a step end (exact float match)", landed,
      f"{len(out['events'])} events, {len(out['visited'])} steps")
out_b = run_fv(case5, option="A", outputs=[21600.0, 36000.0])
out_c = run_fv(case5, option="A", outputs=[36000.0])
check("V20 adding the jump time 6 h to the output list leaves the final state bit-identical",
      all(a == b for a, b in zip(out_b["c"], out_c["c"])))
# V13: switch NOT aligned with a step end, exact step average keeps M_gen exact.
case13 = dict(case5)
case13["source"] = [(0.0, 1e-6), (10801.7, 0.0)]
out13 = run_fv(case13, option="A", align_source=False)
exact_gen = 4 * math.pi / 3 * radii5[0] ** 3 * 1e-6 * 10801.7
check("V13 switch inside a step: step-averaged source gives exact M_gen",
      abs(out13["Mgen"] / exact_gen - 1) < 1e-13,
      f"rel err {abs(out13['Mgen'] / exact_gen - 1):.1e}")

# ---- V16 shared alpha: option A equals BE in tau ------------------------------
lawsS = [[(d0, 300e3)] for d0 in (2e-5, 2e-4, 1e-5, 4e-6, 1e-5)]   # synthetic, shared Q
caseS = dict(case5, laws=lawsS, lam=0.0, source=[(0.0, 0.0)])
runS = run_fv(caseS, option="A")
Tref = 1873.15
Dstar = [diffusivity(br, Tref) for br in lawsS]
Gs, GRs = conductances(geom, Dstar)
cs = [1.0 if g["material"] == 0 else 0.0 for g in geom]
tprev = 0.0
for tn in runS["visited"]:
    seg = hist5.segment(tprev, tn)
    Ta, Tb = hist5.T_in(seg, tprev), hist5.T_in(seg, tn)
    if Ta == Tb:
        alpha_avg = math.exp(-300e3 / RG * (1 / Tb - 1 / Tref))
    else:
        mid, half = 0.5 * (tprev + tn), 0.5 * (tn - tprev)
        alpha_avg = 0.5 * sum(w * math.exp(-300e3 / RG * (1 / hist5.T_in(seg, mid + half * xx) - 1 / Tref))
                              for xx, w in zip(GL8_X, GL8_W))
    cs = v1_step(geom, Gs, GRs, cs, alpha_avg * (tn - tprev))   # v1.0 step in tau
    tprev = tn
diffS = max(abs(a - b) for a, b in zip(runS["c"], cs)) / max(cs)
check("V16 shared alpha, lambda = 0: option A in real time = v1.0 BE in tau with step Delta tau",
      diffS < 1e-11, f"max rel diff {diffS:.1e}")


# ---- B9 effective time on a bare-kernel-like homogeneous sphere --------------
def tau_segment(branches, Ta, Tb, ta, tb, Dref):
    """B9: int alpha dt over one linear segment by (P22.22); constant T if Ta == Tb."""
    if Ta == Tb:
        return (tb - ta) * diffusivity(branches, Ta) / Dref
    slope = (Tb - Ta) / (tb - ta)
    tot = 0.0
    for d0, q in branches:
        thv = q / RG
        tot += d0 * (Tb * expn(2, thv / Tb) - Ta * expn(2, thv / Ta)) / slope
    return tot / Dref


cs_law = [(5.6e-8, 209e3), (5.2e-4, 362e3)]          # FRG Cs in UO2 shape (P21.91)
histK = TemperatureHistory([(0.0, 1273.15), (18000.0, 1873.15), (180000.0, 1873.15)])
DrefK = diffusivity(cs_law, 1873.15)
tau_cf = tau_segment(cs_law, 1273.15, 1873.15, 0.0, 18000.0, DrefK)
tau_q = quad(lambda tt: diffusivity(cs_law, 1273.15 + 600.0 * tt / 18000.0) / DrefK,
             0.0, 18000.0, epsabs=0, epsrel=1e-13)[0]
check("B9 tau over a ramp: E2 closed form (P22.22) = adaptive quadrature",
      abs(tau_cf / tau_q - 1) < 1e-12, f"rel diff {abs(tau_cf / tau_q - 1):.1e}")
tau_end = tau_cf + (180000.0 - 18000.0)
Rk, R1k = 390e-6, 175e-6
nn = np.arange(1, 4001)
kn = nn * math.pi / Rk
In = (-1.0) ** (nn + 1) * Rk / kn                      # int_0^R r sin(kr) dr
I1n = (np.sin(kn * R1k) - kn * R1k * np.cos(kn * R1k)) / kn**2
Nn = Rk / 2.0
wn = 3.0 * I1n * In / (R1k**3 * Nn)                    # v1.0 weights (P1.37)
Lam = DrefK * kn**2
F_series = 1.0 - float(np.sum(wn * np.exp(-Lam * tau_end)))
check("B9 truncation rule Lambda*_max tau >= 40 holds for the series",
      Lam[-1] * tau_end >= 40.0, f"Lambda_max tau = {Lam[-1] * tau_end:.1e}")
caseK = dict(case5, laws=[cs_law] * 5, hist=histK, lam=0.0, source=[(0.0, 0.0)],
             cells=[60, 16, 8, 8, 8], t_end=180000.0, eps_D=1.0)
Fh = []
for dtm in (2400.0, 1200.0, 600.0):
    caseK["dt_max"] = dtm
    r = run_fv(caseK, option="A")
    Fh.append(r["Mrel"] / r["M0"])
pdiff = math.log2(abs(Fh[0] - Fh[1]) / abs(Fh[1] - Fh[2]))
check("V17 FV (option A, variable steps) -> series F(tau(t)): first order in Delta t",
      0.8 < pdiff < 1.2 and abs(Fh[2] - F_series) < 2e-3,
      f"F_series = {F_series:.6f}, F_FV = {Fh[2]:.6f}, p_diff = {pdiff:.2f}")


# ---- B8 modal solution with decay and piecewise-constant source -------------
def phi1n(z):
    z = np.asarray(z, dtype=float)
    small = z < 1e-4
    out_ = np.empty_like(z)
    out_[~small] = -np.expm1(-z[~small]) / z[~small]
    zs = z[small]
    out_[small] = 1 - zs / 2 + zs**2 / 6 - zs**3 / 24
    return out_


def phi2n(z):
    z = np.asarray(z, dtype=float)
    small = z < 1e-2
    out_ = np.empty_like(z)
    zl = z[~small]
    out_[~small] = (zl + np.expm1(-zl)) / zl**2
    zs = z[small]
    out_[small] = 0.5 - zs / 6 + zs**2 / 24 - zs**3 / 120
    return out_


def modal_run(Dm, lam_, sched, c0, times, nmodes):
    """B8: modal amplitudes, inventories and balance-form release (homogeneous sphere data)."""
    k = nn[:nmodes] * math.pi / Rk
    In_, I1_ = In[:nmodes], I1n[:nmodes]
    Lam_ = Dm * k**2
    kap_ = Lam_ + lam_
    a = c0 * I1_ / Nn
    M0 = 4 * math.pi / 3 * R1k**3 * c0
    Vk = 4 * math.pi / 3 * R1k**3
    events = sorted({tt for tt, _ in sched if tt > 0} | set(times))
    tnow, Mgen, Mdec, X, Mrel_direct = 0.0, 0.0, 0.0, M0, 0.0
    res = {}
    for te in events:
        h = te - tnow
        SK = source_step_average(sched, tnow, te)
        sn = SK * I1_ / Nn
        z = kap_ * h
        int_a = h * (a * phi1n(z) + sn * h * phi2n(z))      # H2
        a = a * np.exp(-z) + sn * h * phi1n(z)             # H1
        Gr = Vk * SK
        Mgen += Gr * h
        Mdec += 4 * math.pi * lam_ * float(np.sum(In_ * int_a))
        Mrel_direct += 4 * math.pi * float(np.sum(Lam_ * In_ * int_a))   # P21.38
        X = X * math.exp(-lam_ * h) + Gr * h * float(phi1n(np.array([lam_ * h]))[0])  # H4
        tnow = te
        if te in times:
            M = 4 * math.pi * float(np.sum(a * In_))                    # P21.35
            res[te] = dict(M=M, Mdec=Mdec, Mgen=Mgen, Mrel=M0 + Mgen - M - Mdec,
                           Mrel_direct=Mrel_direct, Mout=X - M)
    return res


Dm, lam_m = 1.5e-13, 1e-6
sched_m = [(0.0, 1e-6), (5e5, 0.0)]
tm = 1e6
mod = modal_run(Dm, lam_m, sched_m, 0.5, [tm], 4000)[tm]
# The direct flux series converges like O(1/N) at worst, with sign changes
# (Section 3.5.2), so test against the 1/N envelope fixed at N = 125.
Ns = (125, 250, 500, 1000, 2000, 4000)
errs = []
for nm in Ns:
    md = modal_run(Dm, lam_m, sched_m, 0.5, [tm], nm)[tm]
    errs.append(abs(md["Mrel_direct"] - md["Mrel"]) / md["Mrel"])
env_ok = all(e * nm_ <= errs[0] * Ns[0] * (1 + 1e-9) for e, nm_ in zip(errs, Ns))
check("B8 direct flux series M_rel -> balance form (P21.38 vs P21.39) inside the C/N envelope",
      env_ok and errs[-1] < errs[0] / 100,
      f"rel diff {errs[0]:.1e} (N=125) -> {errs[-1]:.1e} (N=4000)")
histC = TemperatureHistory([(0.0, 1500.0), (tm, 1500.0)])
caseM = dict(case5, laws=[[(Dm, 0.0)]] * 5, hist=histC, lam=lam_m, source=sched_m,
             c0=0.5, t_end=tm, cells=[70, 20, 8, 8, 8])
vals = []
for dtm in (4000.0, 2000.0, 1000.0):
    caseM["dt_max"] = dtm
    r = run_fv(caseM, option="A")
    vals.append((r["M"], r["Mout"], r["Mrel"]))
pM = math.log2(abs(vals[0][0] - vals[1][0]) / abs(vals[1][0] - vals[2][0]))
pO = math.log2(abs(vals[0][1] - vals[1][1]) / abs(vals[1][1] - vals[2][1]))
# At t = 1e6 s the inventory M has decayed to about 1e-3 of M_out, and the BE
# error of a nearly emptied particle is large (P21.78: Lambda_1 t ~ 5), so compare
# the Richardson-extrapolated FV value (first order in Delta t) with the modal one.
Mx = 2 * vals[2][0] - vals[1][0]
Ox = 2 * vals[2][1] - vals[1][1]
eM = abs(Mx / mod["M"] - 1)
eO = abs(Ox / mod["Mout"] - 1)
check("V7 FV -> modal M and M_out (decay, source switch): order 1 in Delta t, extrapolated match",
      0.8 < pM < 1.2 and 0.8 < pO < 1.2 and eM < 5e-3 and eO < 5e-3,
      f"rel diff (extrapolated) M {eM:.1e}, M_out {eO:.1e}; p_diff {pM:.2f}, {pO:.2f}")
check("B8 modal closure M0 + Mgen = M + Mdec + Mrel and Mout <= Mrel",
      mod["Mout"] <= mod["Mrel"] and abs(mod["M"] + mod["Mdec"] + mod["Mrel"]
                                         - (4 * math.pi / 3 * R1k**3 * 0.5 + mod["Mgen"])) < 1e-15)


# ---- B10 projection between bases (discrete transcription) ------------------
geomP = build_geometry(radii5, [8, 4, 3, 3, 3])
nP = len(geomP)
Vd = np.diag([g["volume"] for g in geomP])


def stiffness(D_layer):
    G, GR = conductances(geomP, D_layer)
    K = np.zeros((nP, nP))
    for i, g in enumerate(G):
        K[i, i] += g
        K[i + 1, i + 1] += g
        K[i, i + 1] -= g
        K[i + 1, i] -= g
    K[-1, -1] += GR
    return K


lamP = 2e-6
Tk = [1873.15, 2073.15]
Ks = [stiffness([diffusivity(br, T_) for br in laws5]) for T_ in Tk]
spans = [3600.0, 7200.0]
cP0 = np.array([1.0 if g["material"] == 0 else 0.0 for g in geomP])
Sb = np.array([1e-6 if g["material"] == 0 else 0.0 for g in geomP])
# reference: exact semi-discrete propagation with expm
cref = cP0.copy()
for K, h in zip(Ks, spans):
    A = np.linalg.solve(Vd, K) + lamP * np.eye(nP)
    E = expm(-A * h)
    cref = E @ cref + np.linalg.solve(A, (np.eye(nP) - E) @ Sb)


def projection_run(nmax):
    """B10: modal propagation on each interval, projection at each switch (P22.40 to P22.45)."""
    a_prev, Phi_prev, dM = None, None, []
    for idx, (K, h) in enumerate(zip(Ks, spans)):
        Lk, Phi = eigh(K, Vd)                      # Phi^T V Phi = I, so N_n = 1
        Lk, Phi = Lk[:nmax], Phi[:, :nmax]
        Ik = Phi.T @ Vd @ np.ones(nP)              # discrete I_n = sum_j V_j phi_jn
        if idx == 0:
            a = Phi.T @ Vd @ cP0                   # project the initial field
        else:
            a = (Phi.T @ Vd @ Phi_prev) @ a_prev   # P_mn (P22.43)
            dM.append(abs(Ik @ a - Iprev @ a_prev))  # inventory mismatch (P22.45)
        sn = Phi.T @ Vd @ Sb
        z = (Lk + lamP) * h
        a = a * np.exp(-z) + sn * h * phi1n(z)     # H1 with kappa_n = Lambda_n + lambda
        a_prev, Phi_prev, Iprev = a, Phi, Ik
    return Phi_prev @ a_prev, dM


cfull, dMfull = projection_run(nP)
efull = np.max(np.abs(cfull - cref)) / np.max(np.abs(cref))
check("V18 projection with the full basis = exact semi-discrete propagation",
      efull < 1e-10, f"max rel diff {efull:.1e}, Delta M_k {dMfull[0]:.1e}")
errs_p, dms = [], []
for nm in (4, 8, 12):
    cp, dM = projection_run(nm)
    errs_p.append(np.max(np.abs(cp - cref)) / np.max(np.abs(cref)))
    dms.append(dM[0])
check("B10 truncated projection: error shrinks as n_max grows, Delta M_k > 0 when truncated",
      errs_p[0] > errs_p[1] > errs_p[2] and dms[0] > 0.0,
      "errors " + ", ".join(f"{e:.1e}" for e in errs_p))

# ---------------------------------------------------------------------------
n_pass = sum(RESULTS)
print(f"SUMMARY {n_pass}/{len(RESULTS)} checks passed")
print("ALL PASS" if n_pass == len(RESULTS) else "SOME CHECKS FAILED")
