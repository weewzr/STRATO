"""
b4_phase2_pseudocode_integration.py

Integration check for docs/mathematics/STRATO_Phase2_Pseudocode.md.

The pseudocode joins the Phase 2.1 blocks (decay, source, inventory) and the
Phase 2.2 blocks (Arrhenius diffusivity, temperature history) into one v2.0 run.
Each part was checked on its own by earlier scripts (b4_decay_source_pseudocode.py
and the Phase 2.2 check script of the work folder). This script checks what only
the joined run can show:

  1. the merged step planner (events from the source and from the temperature
     history, caps from decay, diffusion and Arrhenius slope) lands on every event
     by exact float equality,
  2. one run with a temperature ramp, a decay constant and a switched source
     closes the inventory inside and outside the particle to round-off, keeps the
     concentrations nonnegative, and the closure fails when the surface
     conductance of the first step is used for the whole run (negative test),
  3. a constant-temperature run with single-term laws of Q = 0, lambda = 0 and
     S = 0 reproduces a literal transcription of the v1.0 run loop bit for bit
     (Python floats; the Rust compiler is not tested here),
  4. option A of the step diffusivity equals Backward Euler in the effective time
     for a shared scaling factor and lambda = 0,
  5. the structural test of the effective-time shortcut (Eq. H22.2) on the three
     inputs of the pseudocode,
  6. the closed form with the exponential integral E2 (Eq. P22.22) against
     adaptive quadrature on a long ramp,
  7. the numbers quoted as examples in the pseudocode document.

It is a transcription in Python and does not test any Rust code. Diffusivities of
the buffer and the pyrocarbon layers and the source strength are synthetic test
values. The Cs laws of the kernel and of SiC are the TECDOC-978 rows A1-01, A1-02,
A3-01 and A3-02 of data/diffusivity/arrhenius.csv (fluence factor not applied).
It writes no files.

Run:
  .venv/Scripts/python.exe -I -X utf8 b4_phase2_pseudocode_integration.py
"""

import csv
import math
import os

import numpy as np
from scipy.integrate import quad
from scipy.special import exp1

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
RESULTS = []


def check(label, ok, detail=""):
    RESULTS.append(bool(ok))
    tail = f"  {detail}" if detail else ""
    print(f"{'PASS' if ok else 'FAIL'}  {label}{tail}")


NA = 6.02214076e23
RG = 8.31446261815324
LN2 = math.log(2.0)
GLX, GLW = np.polynomial.legendre.leggauss(8)

# ---------------------------------------------------------------------------
# Block T1: diffusivity laws
# ---------------------------------------------------------------------------


def diffusivity(law, T):
    """T1.4: law = list of (D0, Q in J/mol)."""
    if len(law) == 1 and law[0][1] == 0.0:
        return law[0][0]
    logs = [math.log(d0) - q / (RG * T) for d0, q in law]
    top = max(logs)
    return math.exp(top) * sum(math.exp(v - top) for v in logs)


def q_eff(law, T):
    """T1.5, Eq. P22.6."""
    logs = [math.log(d0) - q / (RG * T) for d0, q in law]
    top = max(logs)
    w = [math.exp(v - top) for v in logs]
    return sum(q * wi for (_, q), wi in zip(law, w)) / sum(w)


# ---------------------------------------------------------------------------
# Block T2: linear temperature history
# ---------------------------------------------------------------------------


class History:
    """Linear kind (continuous, piecewise linear) or constant kind (right-continuous steps)."""

    def __init__(self, times, temps, kind="linear", end=None):
        assert times[0] == 0.0 and all(b > a for a, b in zip(times, times[1:]))
        self.kind = kind
        if kind == "constant":
            assert end is not None and end > times[-1]
            times = list(times) + [end]
            temps = list(temps) + [temps[-1]]
        self.t, self.T = list(times), list(temps)

    def end(self):
        return self.t[-1]

    def temperature_end(self, t_n, t_next):
        """Temperature at the END of a step that starts at t_n: the value of the segment of t_n.
        For the linear kind this is T(t_next); for the constant kind it is the left limit."""
        if self.kind == "linear":
            return self.temperature_at(t_next)
        k = max(i for i, x in enumerate(self.t) if x <= t_n)
        return self.T[k]

    def seg(self, t):
        k = max(i for i, x in enumerate(self.t) if x <= t)
        return min(k, len(self.t) - 2)

    def temperature_at(self, t):
        assert 0.0 <= t <= self.end()
        k = max(i for i, x in enumerate(self.t) if x <= t)
        if t == self.t[k] or self.kind == "constant":
            return self.T[k]
        return self.T[k] +(self.T[k + 1] - self.T[k]) * (t - self.t[k]) / (self.t[k + 1] - self.t[k])

    def slope_after(self, t):
        if t >= self.end() or self.kind == "constant":
            return 0.0
        k = max(i for i, x in enumerate(self.t) if x <= t)
        return (self.T[k + 1] - self.T[k]) / (self.t[k + 1] - self.t[k])


# ---------------------------------------------------------------------------
# Block T4.2: step diffusivity, option A (step average) or E (end of step)
# ---------------------------------------------------------------------------


def e2(x):
    return math.exp(-x) - x * exp1(x)


def branch_average(d0, q, Ta, Tb, ta, tb):
    s = (Tb - Ta) / (tb - ta)
    theta = q / RG
    x = theta * abs(1.0 / Tb - 1.0 / Ta)
    if x <= 1.0:
        mid, half = 0.5 * (ta + tb), 0.5 * (tb - ta)
        hist = HIST_FOR_AVG[0]
        total = 0.0
        for xi, wi in zip(GLX, GLW):
            T = hist.temperature_at(mid + half * xi) if hist is not None else Ta + s * (mid + half * xi - ta)
            total += wi * d0 * math.exp(-q / (RG * T))
        return 0.5 * total
    return d0 / (tb - ta) / s * (Tb * e2(theta / Tb) - Ta * e2(theta / Ta))


HIST_FOR_AVG = [None]


def step_diffusivity(laws, hist, option, ta, tb):
    if option == "E":
        T = hist.temperature_end(ta, tb)
        return [diffusivity(l, T) for l in laws]
    HIST_FOR_AVG[0] = hist
    Ta, Tb = hist.temperature_at(ta), hist.temperature_at(tb)
    if hist.slope_after(ta) == 0.0:
        return [diffusivity(l, Ta) for l in laws]
    out = []
    for l in laws:
        if len(l) == 1 and l[0][1] == 0.0:
            out.append(l[0][0])
        else:
            out.append(sum(branch_average(d0, q, Ta, Tb, ta, tb) for d0, q in l))
    return out


# ---------------------------------------------------------------------------
# Mesh (v1.0 build rules), conductances (T4.3), solver (C2.3)
# ---------------------------------------------------------------------------


def shell_volume(a, b):
    return 4.0 * math.pi / 3.0 * (b ** 3 - a ** 3)


def volume_centroid(a, b):
    if a == 0.0:
        return 0.75 * b
    return 0.75 * (b ** 4 - a ** 4) / (b ** 3 - a ** 3)


def geometry(radii, cells):
    """T4.1: everything that does not depend on D."""
    cl = []
    inner = 0.0
    for mat, (r, n) in enumerate(zip(radii, cells)):
        width = (r - inner) / n
        for k in range(n):
            left = inner + k * width
            right = r if k + 1 == n else inner + (k + 1) * width
            cl.append(dict(left=left, right=right, c=volume_centroid(left, right), V=shell_volume(left, right), mat=mat))
        inner = r
    faces = []
    for a, b in zip(cl, cl[1:]):
        f = a["right"]
        faces.append((4.0 * math.pi * f * f, f - a["c"], b["c"] - f, a["mat"], b["mat"]))
    R = radii[-1]
    outer = (4.0 * math.pi * R * R, R - cl[-1]["c"], cl[-1]["mat"])
    return cl, faces, outer


def conductances(faces, outer, D):
    g = [A / (dl / D[ma] + dr / D[mb]) for (A, dl, dr, ma, mb) in faces]
    GR = outer[0] * D[outer[2]] / outer[1]
    return g, GR


def solve_system(v, gw, ge, rhs):
    n = len(v)
    den = [0.0] * n
    dp = [0.0] * n
    e_prev = 0.0
    for i in range(n):
        e = v[0] if i == 0 else v[i] + gw[i] * e_prev / (e_prev + gw[i])
        den[i] = e + ge[i]
        carry = 0.0 if i == 0 else gw[i] * dp[i - 1]
        dp[i] = (rhs[i] + carry) / den[i]
        e_prev = e
    x = [0.0] * n
    x[-1] = dp[-1]
    for i in range(n - 2, -1, -1):
        x[i] = dp[i] + ge[i] / den[i] * x[i + 1]
    return x


def be_step(vols, g, GR, old, dt, lam, src):
    """C2.2 and C2.3 (Eqs. P21.58, P21.59)."""
    n = len(vols)
    gw = [0.0] + list(g)
    ge = list(g) + [GR]
    v = [vols[i] / dt + lam * vols[i] for i in range(n)]
    rhs = [(vols[i] / dt) * old[i] + vols[i] * src[i] for i in range(n)]
    return solve_system(v, gw, ge, rhs)


class Kahan:
    def __init__(self):
        self.s = 0.0
        self.c = 0.0

    def add(self, x):
        y = x - self.c
        t = self.s + y
        self.c = (t - self.s) - y
        self.s = t


# ---------------------------------------------------------------------------
# Source schedule (block B): fission rate density history, kernel only
# ---------------------------------------------------------------------------


def source_step_kernel(schedule, yield_, t0, t1):
    """B4: exact step average of S_K = y F / N_A over [t0, t1] (option A)."""
    acc = 0.0
    for (a, b, rate) in schedule:
        ov = min(b, t1) - max(a, t0)
        if ov > 0.0:
            acc += (yield_ * rate / NA) * ov
    return acc / (t1 - t0)


# ---------------------------------------------------------------------------
# Merged step planner (block S)
# ---------------------------------------------------------------------------


def plan_events(hist, outputs, switches, t_end):
    ev = {}
    for t in hist.t[1:]:
        ev[t] = "JUMP" if hist.kind == "constant" else "BREAK"
    for t in outputs:
        ev.setdefault(t, "OUTPUT")
    for t in switches:
        ev[t] = "SWITCH"
    ev[t_end] = ev.get(t_end, "OUTPUT")
    assert all(0.0 < t <= t_end for t in ev)
    return sorted(ev.items())


def plan_steps(hist, laws, events, dt_max, dt_first, dt_decay, dt_diff, eps_D=0.1, growth=2.0, naive_end=False):
    """Returns list of (t_n, t_next, dt, kind-or-None)."""
    out = []
    t = 0.0
    idx = 0
    grow_cap = dt_first
    reset_count = 0
    while idx < len(events):
        if len(out) > 100000:
            raise RuntimeError("planner produced more than 100000 steps")
        t_event, kind = events[idx]
        T = hist.temperature_at(t)
        s = hist.slope_after(t)
        if s == 0.0:
            bound_T = math.inf
        else:
            qmax = max(q_eff(l, T) for l in laws)
            bound_T = eps_D * RG * T * T / (qmax * abs(s))
        dt = min(dt_max, dt_decay, dt_diff, grow_cap, bound_T)
        for _ in range(30):
            rem = t_event - t
            if rem <= dt:
                t_next = t_event
            elif rem <= 2.0 * dt:
                t_next = t + rem / 2.0
            else:
                t_next = t + dt
            lnD0 = [math.log(diffusivity(l, T)) for l in laws]
            T1 = hist.temperature_at(t_next) if naive_end else hist.temperature_end(t, t_next)
            lnD1 = [math.log(diffusivity(l, T1)) for l in laws]
            if max(abs(a - b) for a, b in zip(lnD0, lnD1)) <= 2.0 * eps_D or t_next - t < 1e-6:
                break
            dt = (t_next - t) / 2.0
        else:
            raise RuntimeError("guard loop did not finish")
        dtn = t_next - t
        landed = t_next == t_event
        out.append((t, t_next, dtn, kind if landed else None))
        t = t_next
        grow_cap = growth * dtn
        if landed:
            idx += 1
            if kind in ("SWITCH", "JUMP"):
                grow_cap = dt_first
                reset_count += 1
    return out


# ---------------------------------------------------------------------------
# Blocks C2, D with the merged loop
# ---------------------------------------------------------------------------


def run_v2(radii, cells, laws, hist, steps, lam, yield_, schedule, c0_kernel, option="A", stale_GR=False):
    cl, faces, outer = geometry(radii, cells)
    vols = [c["V"] for c in cl]
    n = len(cl)
    conc = [c0_kernel if c["mat"] == 0 else 0.0 for c in cl]
    inv0 = sum(vols[i] * conc[i] for i in range(n))
    gen, dec, rel, dec_out = Kahan(), Kahan(), Kahan(), Kahan()
    out_now = 0.0
    GR_first = None
    res_in_max = res_out_max = 0.0
    min_c = math.inf
    rows = []
    for (ta, tb, dt, _) in steps:
        D = step_diffusivity(laws, hist, option, ta, tb)
        g, GR = conductances(faces, outer, D)
        if GR_first is None:
            GR_first = GR
        GRb = GR_first if stale_GR else GR
        sk = source_step_kernel(schedule, yield_, ta, tb)
        src = [sk if c["mat"] == 0 else 0.0 for c in cl]
        conc = be_step(vols, g, GR, conc, dt, lam, src)
        inv = sum(vols[i] * conc[i] for i in range(n))
        rate = GRb * conc[-1]
        gen_rate = sum(vols[i] * src[i] for i in range(n))
        gen.add(dt * gen_rate)
        dec.add(dt * lam * inv)
        rel.add(dt * rate)
        out_now = (out_now + dt * rate) / (1.0 + lam * dt)
        dec_out.add(dt * lam * out_now)
        supply = inv0 + gen.s
        res_in = (supply - (inv + dec.s + rel.s)) / supply
        res_out = (supply - (inv + out_now + dec.s + dec_out.s)) / supply
        res_in_max = max(res_in_max, abs(res_in))
        res_out_max = max(res_out_max, abs(res_out))
        min_c = min(min_c, min(conc))
        rows.append((tb, inv, gen.s, dec.s, rel.s, out_now))
    return dict(rows=rows, res_in=res_in_max, res_out=res_out_max, min_c=min_c, conc=conc, inv0=inv0)


# ---------------------------------------------------------------------------
# Literal transcription of the v1.0 run loop (finite_volume.rs lines 162 to 236)
# ---------------------------------------------------------------------------


def run_v1(radii, cells, d_layers, c0_kernel, end_time, dt_s):
    cl, faces, outer = geometry(radii, cells)
    vols = [c["V"] for c in cl]
    n = len(cl)
    g, GR = conductances(faces, outer, d_layers)
    conc = [c0_kernel if c["mat"] == 0 else 0.0 for c in cl]
    initial = sum(vols[i] * conc[i] for i in range(n))
    steps = math.ceil(end_time / dt_s)
    dt = end_time / steps
    rel = Kahan()
    rows = []
    for k in range(1, steps + 1):
        v = [vols[i] / dt for i in range(n)]
        rhs = [v[i] * conc[i] for i in range(n)]
        gw = [0.0] + list(g)
        ge = list(g) + [GR]
        nxt = solve_system(v, gw, ge, rhs)
        rate = GR * nxt[-1]
        rel.add(dt * rate)
        conc = nxt
        inv = sum(vols[i] * conc[i] for i in range(n))
        rows.append((k * dt, inv, rel.s))
    return rows, conc, steps, dt


# ===========================================================================
# Test case data
# ===========================================================================
RADII = [250e-6, 350e-6, 390e-6, 425e-6, 465e-6]
CELLS = [8, 4, 2, 3, 3]
KERNEL = [(5.6e-8, 209e3), (5.2e-4, 362e3)]  # A1-01, A1-02
BUFFER = [(1e-10, 0.0)]  # synthetic constant layer
PYC = [(1e-6, 222e3)]  # synthetic pyrocarbon law
SIC = [(5.5e-14, 125e3), (1.6e-2, 514e3)]  # A3-01, A3-02 at fluence 0
LAWS = [KERNEL, BUFFER, PYC, SIC, PYC]

print("== 1. Merged planner and one joined run ==")
hours = 3600.0
hist = History([0.0, 4 * hours, 8 * hours], [1273.15, 2073.15, 2073.15])
t_end = hist.end()
switches = [3 * hours]
outputs = [k * hours for k in range(1, 9)]
events = plan_events(hist, outputs, switches, t_end)
lam = 1.0e-5
dt_decay = 2 * 1e-3 / (lam ** 2 * t_end)
steps = plan_steps(hist, LAWS, events, dt_max=600.0, dt_first=5.0, dt_decay=dt_decay, dt_diff=math.inf)
print(f"  events {len(events)}, steps {len(steps)}, dt_decay {dt_decay:.1f} s")
ev_times = [t for t, _ in events]
ends = {s[1] for s in steps}
check("every event time is the end of a step by float equality", all(t in ends for t in ev_times),
      f"{len(ev_times)} of {len(ev_times)}")
check("steps chain without gaps (t_n+1 of one step equals t_n of the next)",
      all(steps[i][1] == steps[i + 1][0] for i in range(len(steps) - 1)))
check("no step exceeds dt_max or dt_decay", max(s[2] for s in steps) <= min(600.0, dt_decay))
after_switch = [steps[i + 1][2] for i in range(len(steps) - 1) if steps[i][3] == "SWITCH"]
check("the step after the source switch restarts at dt_first", all(abs(d - 5.0) < 1e-9 or d <= 5.0 for d in after_switch),
      f"{after_switch}")
dlns = []
for (ta, tb, dt, _) in steps:
    for l in LAWS:
        dlns.append(abs(math.log(diffusivity(l, hist.temperature_at(tb))) - math.log(diffusivity(l, hist.temperature_at(ta)))))
check("largest change of ln D in a step is at most 2 eps_D = 0.2", max(dlns) <= 0.2 + 1e-12, f"{max(dlns):.3f}")
grow = max(steps[i + 1][2] / steps[i][2] for i in range(len(steps) - 1) if steps[i][3] != "SWITCH" and steps[i + 1][2] > steps[i][2])
check("growth between neighbouring steps is at most 2 (except after a switch)", grow <= 2.0 + 1e-12, f"{grow:.3f}")

yield_ = 6.2208e-2
schedule = [(0.0, 3 * hours, 5.0e19), (3 * hours, t_end, 0.0)]
res = run_v2(RADII, CELLS, LAWS, hist, steps, lam, yield_, schedule, 0.0, "A")
check("joined run, option A: inside closure below 1e-12", res["res_in"] < 1e-12, f"{res['res_in']:.2e}")
check("joined run, option A: outside closure below 1e-12", res["res_out"] < 1e-12, f"{res['res_out']:.2e}")
check("joined run: minimum concentration is at least 0.0", res["min_c"] >= 0.0, f"{res['min_c']:.2e}")
gen_exact = (4.0 * math.pi / 3.0 * RADII[0] ** 3) * (yield_ * 5.0e19 / NA) * 3 * hours
check("generated amount equals the analytic integral (switch lands on a step end)",
      abs(res["rows"][-1][2] - gen_exact) / gen_exact < 1e-13, f"{abs(res['rows'][-1][2] - gen_exact) / gen_exact:.1e}")
resE = run_v2(RADII, CELLS, LAWS, hist, steps, lam, yield_, schedule, 0.0, "E")
check("joined run, option E: closure below 1e-12", max(resE["res_in"], resE["res_out"]) < 1e-12,
      f"{max(resE['res_in'], resE['res_out']):.2e}")
rel_A, rel_E = res["rows"][-1][4], resE["rows"][-1][4]
print(f"  released amount, option A {rel_A:.6e} mol, option E {rel_E:.6e} mol, relative difference {abs(rel_A - rel_E) / rel_A:.2e}")
resS = run_v2(RADII, CELLS, LAWS, hist, steps, lam, yield_, schedule, 0.0, "A", stale_GR=True)
check("negative test: booking with the first-step surface conductance breaks the closure (above 1e-8)",
      resS["res_in"] > 1e-8, f"{resS['res_in']:.2e}")

print("== 2. Regression to v1.0 (lambda = 0, S = 0, constant T, Q = 0 laws) ==")
d_layers = [4e-14, 4e-13, 2e-14, 5e-15, 2e-14]
const_laws = [[(d, 0.0)] for d in d_layers]
hist_c = History([0.0, 1.0e6], [1273.15, 1273.15])
end_time, dt_req = 4.0e5, 2.0e3
rows1, conc1, nsteps, dtn = run_v1(RADII, CELLS, d_layers, 1.0, end_time, dt_req)
fixed = [((k - 1) * dtn if k > 1 else 0.0, k * dtn, dtn, None) for k in range(1, nsteps + 1)]
res2 = run_v2(RADII, CELLS, const_laws, hist_c, fixed, 0.0, 0.0, [], 1.0, "A")
same = all(a[1] == b[1] and a[4] == b[2] for a, b in zip(res2["rows"], rows1)) and res2["conc"] == conc1
check("v2 loop equals the v1.0 transcription bit for bit over every step", same, f"{nsteps} steps, dt {dtn:.6f} s")

import random

random.seed(1)
n_pairs, n_diff, max_rel = 20000, 0, 0.0
for _ in range(n_pairs):
    end_r = random.uniform(1e3, 1e8)
    dt_r = end_r / random.uniform(5, 500) * random.uniform(0.7, 1.0)
    d_fixed = end_r / math.ceil(end_r / dt_r)  # v1.0 rule (lines 190 and 191)
    tot = 0.0
    while tot < end_r * (1 - 1e-14):
        tot += dt_r
    d_geo = dt_r * (end_r / tot)  # geometric rule with growth 1 and the common scale factor
    if d_geo != d_fixed:
        n_diff += 1
        max_rel = max(max_rel, abs(d_geo - d_fixed) / d_fixed)
print(f"  growth-1 geometric grid against the v1.0 constant step: {n_diff} of {n_pairs} pairs differ, largest relative difference {max_rel:.2e}")
check("the geometric rule with growth 1 does not reproduce the v1.0 step bit for bit (motivates FIXED mode)",
      n_diff > 0 and max_rel < 1e-12)

print("== 3. Option A equals Backward Euler in the effective time (shared Q, lambda = 0) ==")
Q_sh = 300e3
sh_laws = [[(d0, Q_sh)] for d0 in (1e-5, 3e-6, 1e-7, 5e-8, 1e-7)]
hist_sh = History([0.0, 4 * hours, 6 * hours], [1273.15, 1873.15, 1873.15])
ev_sh = plan_events(hist_sh, [6 * hours], [], hist_sh.end())
st_sh = plan_steps(hist_sh, sh_laws, ev_sh, dt_max=900.0, dt_first=20.0, dt_decay=math.inf, dt_diff=math.inf)
rA = run_v2(RADII, CELLS, sh_laws, hist_sh, st_sh, 0.0, 0.0, [], 1.0, "A")
Tref = 1873.15
cl, faces, outer = geometry(RADII, CELLS)
vols = [c["V"] for c in cl]
Dstar = [diffusivity(l, Tref) for l in sh_laws]
gS, GRS = conductances(faces, outer, Dstar)
conc = [1.0 if c["mat"] == 0 else 0.0 for c in cl]
zero = [0.0] * len(cl)
for (ta, tb, dt, _) in st_sh:
    alpha_int = quad(lambda t: diffusivity(sh_laws[0], hist_sh.temperature_at(t)) / diffusivity(sh_laws[0], Tref), ta, tb,
                     epsabs=0, epsrel=1e-13, limit=200)[0]
    conc = be_step(vols, gS, GRS, conc, alpha_int, 0.0, zero)
inv_tau = sum(vols[i] * conc[i] for i in range(len(cl)))
inv_A = rA["rows"][-1][1]
check("inventory after the run: option A against BE in tau agrees to 1e-11", abs(inv_A - inv_tau) / inv_tau < 1e-11,
      f"{abs(inv_A - inv_tau) / inv_tau:.2e}")

print("== 4. Structural test of the effective-time shortcut (Eq. H22.2) ==")


def effective_time_check(laws):
    reasons = []
    if len(laws) == 1:
        return True, reasons
    ref = sorted(laws[0], key=lambda b: b[1])
    for i, l in enumerate(laws[1:], start=2):
        cur = sorted(l, key=lambda b: b[1])
        if len(cur) != len(ref):
            reasons.append(f"layer {i}: different number of terms")
            continue
        ratios = []
        for (d0c, qc), (d0r, qr) in zip(cur, ref):
            if abs(qc - qr) > 1e-12 * max(qr, 1.0):
                reasons.append(f"layer {i}: activation energies differ")
            ratios.append(d0c / d0r)
        if max(ratios) - min(ratios) > 1e-12 * max(ratios):
            reasons.append(f"layer {i}: D0 ratio depends on the term")
    return (not reasons), reasons


ok_real, why_real = effective_time_check(LAWS)
check("real TECDOC-978 style laws (two-term kernel, constant buffer) are refused", (not ok_real) and len(why_real) > 0, f"{why_real[:2]}")
ok_sh, _ = effective_time_check(sh_laws)
check("synthetic shared-Q laws are accepted", ok_sh)
ok_one, _ = effective_time_check([KERNEL])
check("a single layer is accepted for any law", ok_one)
Tg = np.linspace(1523.15, 2073.15, 6)
ratio = [diffusivity(LAWS[3], T) / diffusivity(LAWS[0], T) for T in (1523.15, 2073.15)]
print(f"  D_SiC/D_kernel at 1250 and 1800 degC: {ratio[0]:.2e}, {ratio[1]:.2e}")

print("== 5. Closed form with E2 (Eq. P22.22) against adaptive quadrature ==")
ta_, tb_ = 0.0, 4 * hours
h_ramp = History([0.0, 4 * hours, 8 * hours], [1273.15, 2073.15, 2073.15])
q_ = 438e3
HIST_FOR_AVG[0] = h_ramp
Ta_, Tb_ = h_ramp.temperature_at(ta_), h_ramp.temperature_at(tb_)
cf = branch_average(1.0, q_, Ta_, Tb_, ta_, tb_)
qd = quad(lambda t: math.exp(-q_ / (RG * h_ramp.temperature_at(t))), ta_, tb_, epsabs=0, epsrel=1e-13, limit=400)[0] / (tb_ - ta_)
x_big = (q_ / RG) * abs(1 / Tb_ - 1 / Ta_)
check("closed form for a change of ln D far above 1 agrees with quadrature to 1e-12", abs(cf - qd) / qd < 1e-12,
      f"x = {x_big:.1f}, difference {abs(cf - qd) / qd:.1e}")

print("== 5b. Constant-interval history: the temperature at the end of a step is the left limit ==")
hist_j = History([0.0, 2 * hours], [1273.15, 1873.15], kind="constant", end=4 * hours)
ev_j = plan_events(hist_j, [k * 0.5 * hours for k in range(1, 9)], [], hist_j.end())
st_ok = plan_steps(hist_j, LAWS, ev_j, dt_max=600.0, dt_first=5.0, dt_decay=math.inf, dt_diff=math.inf)
st_naive = plan_steps(hist_j, LAWS, ev_j, dt_max=600.0, dt_first=5.0, dt_decay=math.inf, dt_diff=math.inf, naive_end=True)
check("left-limit rule: the jump at 2 h is a step end and no step is shorter than 1 s except the first",
      any(s[1] == 2 * hours for s in st_ok) and min(s[2] for s in st_ok[1:]) >= 1.0,
      f"{len(st_ok)} steps, shortest {min(s[2] for s in st_ok[1:]):.3g} s")
print(f"  naive rule (T at t_next, right-continuous): {len(st_naive)} steps, shortest {min(s[2] for s in st_naive):.3g} s")
check("naive rule is a trap: it produces steps below 1 ms next to the jump", min(s[2] for s in st_naive) < 1e-3)
res_j = {o: run_v2(RADII, CELLS, LAWS, hist_j, st_ok, 1e-5, 0.0, [], 1.0, o) for o in ("A", "E")}
check("constant-interval run, options A and E: closure below 1e-12 and minimum concentration 0 or above",
      all(max(r["res_in"], r["res_out"]) < 1e-12 and r["min_c"] >= 0.0 for r in res_j.values()),
      f"{max(max(r['res_in'], r['res_out']) for r in res_j.values()):.2e}")
D_before = step_diffusivity(LAWS, hist_j, "E", 2 * hours - 30.0, 2 * hours)
D_after = step_diffusivity(LAWS, hist_j, "E", 2 * hours, 2 * hours + 30.0)
D_naive = [diffusivity(l, hist_j.temperature_at(2 * hours)) for l in LAWS]
check("option E on the step that ends at the jump uses the old temperature",
      D_before == [diffusivity(l, 1273.15) for l in LAWS] and D_after == D_naive and D_before != D_naive)

print("== 6. Numbers quoted in the pseudocode document ==")
decay = {}
with open(os.path.join(REPO, "data", "nuclear", "decay.csv"), newline="", encoding="utf-8") as fh:
    for row in csv.DictReader(fh):
        decay[row["nuclide"]] = row
for nuc, lam_doc, t_doc in (("Cs-137", 7.302187e-10, 9.492323e8), ("Sr-90", 7.597709e-10, 9.123107e8),
                            ("Ag-110m", 3.211198e-8, 2.158531e7), ("I-131", 9.996681e-7, 6.933773e5)):
    row = decay[nuc]
    t_half = float(row["half_life_s"])
    lam_n = LN2 / t_half
    check(f"{nuc}: lambda = ln 2 / t_half matches the table", abs(lam_n - lam_doc) / lam_doc < 5e-7 and abs(t_half - t_doc) / t_doc < 1e-6,
          f"lambda {lam_n:.6e}, t_half {t_half:.6e}, mean life {1 / lam_n:.4e}")
with open(os.path.join(REPO, "data", "nuclear", "fission_yields.csv"), newline="", encoding="utf-8") as fh:
    cfy = [r for r in csv.DictReader(fh) if r["parent"] == "235U" and r["neutron_energy"] == "thermal" and r["product"] == "Cs-137"]
y_cs = float(cfy[0]["CFY_JEFF311"])
sk = y_cs * 5.0e19 / NA
check("Cs-137 source example: y = 6.2208e-2, S_K = 5.1649e-6 mol m^-3 s^-1", abs(y_cs - 6.2208e-2) < 1e-9 and abs(sk - 5.1649e-6) < 5e-11,
      f"y {y_cs}, S_K {sk:.4e}")
lam_ag = float(decay["Ag-110m"]["lambda_s-1"])
dt_ag = 2 * 1e-3 / (lam_ag ** 2 * 620 * 86400.0)
lam_i = float(decay["I-131"]["lambda_s-1"])
dt_i = 2 * 1e-3 / (lam_i ** 2 * 5.0 / lam_i)
check("C1.2 examples: Ag-110m 3.62e4 s and I-131 400 s", abs(dt_ag - 3.62e4) / 3.62e4 < 5e-3 and abs(dt_i - 400.0) < 1.0,
      f"{dt_ag:.4e} s, {dt_i:.1f} s")
z, c_old, s_, lam_s, dt_s = 0.1, 1.0, 1e-7, 1e-6, 1e5
c_be = (c_old + dt_s * s_) / (1 + lam_s * dt_s)
c_ex = c_old * math.exp(-lam_s * dt_s) + (s_ / lam_s) * (1 - math.exp(-lam_s * dt_s))
check("scalar example: 0.9181818 against 0.9143537", abs(c_be - 0.9181818) < 5e-8 and abs(c_ex - 0.9143537) < 5e-8,
      f"{c_be:.7f}, {c_ex:.7f}, difference {c_be - c_ex:+.2e}")
t99_f = lambda half_s: math.log(100.0) * half_s / LN2
vals = {"Xe-137 3.818 min": t99_f(3.818 * 60) / 3600, "Rb-90m 258 s": t99_f(258.0) / 3600,
        "Kr-85m 4.48 h": t99_f(4.48 * 3600) / 3600, "Te-131m 33.25 h": t99_f(33.25 * 3600) / 86400,
        "Xe-133 5.2475 d": t99_f(5.2475 * 86400) / 86400}
print("  t99:", {k: round(v, 4) for k, v in vals.items()})
check("t99: 0.423 h, 0.476 h, 29.8 h, 9.2 d, 34.9 d", abs(vals["Xe-137 3.818 min"] - 0.423) < 5e-4 and abs(vals["Rb-90m 258 s"] - 0.476) < 5e-4
      and abs(vals["Kr-85m 4.48 h"] - 29.8) < 0.05 and abs(vals["Te-131m 33.25 h"] - 9.2) < 0.05 and abs(vals["Xe-133 5.2475 d"] - 34.9) < 0.05)
cs_sic = [(5.5e-14, 125e3), (1.6e-2, 514e3)]
bT = lambda Tc, rate_h: 0.1 * RG * (Tc + 273.15) ** 2 / (q_eff(cs_sic, Tc + 273.15) * abs(rate_h) / 3600.0)
b1, b2, b3 = bT(1600, 50), bT(1250, 120), bT(1600, -600)
check("T3 examples: 479 s, 445 s, 40 s", abs(b1 - 479) < 1 and abs(b2 - 445) < 1 and abs(b3 - 40) < 1, f"{b1:.1f}, {b2:.1f}, {b3:.1f}")

print()
n_ok = sum(RESULTS)
print(f"SUMMARY: {n_ok}/{len(RESULTS)} PASS")
print("ALL PASS" if n_ok == len(RESULTS) else "SOME FAILED")
raise SystemExit(0 if n_ok == len(RESULTS) else 1)
