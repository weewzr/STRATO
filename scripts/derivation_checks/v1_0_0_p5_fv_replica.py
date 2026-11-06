"""STRATO v1.0.0, Full Mathematical Derivation, Part V: numerical replica of Code B
(src/numerical/finite_volume.rs at tag v1.0.0) on the synthetic five-layer input
(inputs/synthetic/synthetic_five_layer.toml; synthetic values, not literature data).

The replica follows the Rust arithmetic line by line (powi as repeated products,
sequential sums, the same order of operations) and checks the numbers quoted in Part V.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/v1_0_0_p5_fv_replica.py
Writes nothing. Prints one PASS/FAIL line per check; exit code 1 on any FAIL.
"""

import math
import sys

import numpy as np

results = []


def check(label, ok, detail=""):
    ok = bool(ok)
    results.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {label}" + (f"  [{detail}]" if detail else ""), flush=True)


PI = math.pi
# synthetic_five_layer.toml
regions = [  # (outer_radius_m, diffusivity_m2_s, cells)
    (2.0e-4, 1.0e-12, 40),
    (3.0e-4, 4.0e-10, 40),
    (3.5e-4, 8.0e-13, 40),
    (4.0e-4, 2.0e-14, 40),
    (4.5e-4, 7.0e-13, 40),
]
c0, dt_in, t_end, n_out = 1.0, 2.0e2, 2.0e5, 10


def p3(x):
    return x * x * x


def p4(x):
    x2 = x * x
    return x2 * x2


def shell_volume(a, b):            # lines 38 to 40
    return 4.0 * PI / 3.0 * (p3(b) - p3(a))


def volume_centroid(a, b):         # lines 42 to 48
    if a == 0.0:
        return 0.75 * b
    return 0.75 * (p4(b) - p4(a)) / (p3(b) - p3(a))


# build_mesh, lines 50 to 99
cells = []
inner = 0.0
snap_gap = []
for mat, (R, D, n) in enumerate(regions):
    width = (R - inner) / n
    snap_gap.append(abs((inner + n * width) - R))
    for k in range(n):
        left = inner + k * width
        right = R if k + 1 == n else inner + (k + 1) * width
        cells.append(dict(left=left, right=right, centroid=volume_centroid(left, right),
                          volume=shell_volume(left, right), mat=mat, D=D))
    inner = R
N = len(cells)
cond = []
dls, drs = [], []
for L, Rc in zip(cells[:-1], cells[1:]):
    face = L["right"]
    area = 4.0 * PI * face * face
    dl = face - L["centroid"]
    dr = Rc["centroid"] - face
    dls.append(dl)
    drs.append(dr)
    cond.append(area / (dl / L["D"] + dr / Rc["D"]))
Rout = regions[-1][0]
GR = 4.0 * PI * Rout * Rout * cells[-1]["D"] / (Rout - cells[-1]["centroid"])

check("P5R-01 Eq. (5.2): N_c = 200 cells, widths w_l = L_l/40",
      N == 200, "w = " + ", ".join(f"{(R - (regions[i-1][0] if i else 0.0)) / 40:.4e}" for i, (R, _, _) in enumerate(regions)) + " m")
check("P5R-02 faces are contiguous: right face of each cell equals left face of the next, bit for bit",
      all(cells[i]["right"] == cells[i + 1]["left"] for i in range(N - 1)))
check("P5R-03 interface faces equal R_l exactly (snapping, lines 58 to 59)",
      all(cells[40 * l - 1]["right"] == regions[l - 1][0] for l in range(1, 6)),
      "max |inner + n w - R_l| without snapping = %.1e m" % max(snap_gap))
check("P5R-04 every centroid lies strictly inside its cell and outward of the midpoint",
      all(c["left"] < c["centroid"] < c["right"] and c["centroid"] > 0.5 * (c["left"] + c["right"]) for c in cells))
w13 = 2.0e-4 / 13
check("P5R-03b without snapping the last face can miss R_l: 13 cells in the kernel give 13 w = 1.9999999999999998e-4 m",
      13 * w13 != 2.0e-4, f"13 w = {13 * w13!r}")
r1 = cells[0]["centroid"]
check("P5R-05 first cell: r_1 = 0.75 r_{3/2} = 3.75e-6 m; dL at face 3/2 = r_{3/2}/4 = 1.25e-6 m",
      abs(r1 - 3.75e-6) < 1e-20 and abs(dls[0] - 1.25e-6) < 1e-20, f"r_1 = {r1!r}")
c2 = cells[1]
check("P5R-06 second cell (a = 5e-6, b = 1e-5 m): centroid 8.0357e-6 m vs midpoint 7.5e-6 m",
      abs(c2["centroid"] - 0.75 * (1e-20 - 6.25e-22) / (1e-15 - 1.25e-16)) < 1e-18,
      f"r_2 = {c2['centroid']:.6e}")
sumV = 0.0
for c in cells:
    sumV += c["volume"]
check("P5R-07 cell volumes add up to (4 pi/3) R^3 (telescoping of Eq. (5.3))",
      abs(sumV - 4.0 * PI / 3.0 * p3(Rout)) / sumV < 1e-14, f"rel diff {abs(sumV - 4*PI/3*p3(Rout))/sumV:.1e}")

# initial condition and inventory (lines 101 to 120, 186)
C = [c0 if c["mat"] == 0 else 0.0 for c in cells]


def inventory(Cv):
    s = 0.0
    for c, x in zip(cells, Cv):
        s += c["volume"] * x
    return s


M0 = inventory(C)
check("P5R-08 Eq. (5.20): M^0 = sum V_j C_j^0 = 3.351032163829113e-11 mol",
      abs(M0 - 3.351032163829113e-11) <= 2e-26, f"M0 = {M0!r}; (4 pi/3) R_1^3 c_0 = {4*PI/3*p3(2e-4)*c0!r}")

# interface conductances: show the asymmetric distances at the kernel/buffer face
f1 = 39  # face between cell 40 (last kernel) and 41 (first buffer), 0-based pair index
check("P5R-09 kernel/buffer face (j = 40): dL != dR and G uses both D values (line 86)",
      dls[f1] != drs[f1] and abs(cond[f1] - 4*PI*2e-4**2 / (dls[f1]/1e-12 + drs[f1]/4e-10)) < 1e-30,
      f"dL = {dls[f1]:.6e} m, dR = {drs[f1]:.6e} m, G = {cond[f1]:.6e} m^3/s, kernel-only half would give {4*PI*2e-4**2*1e-12/dls[f1]:.6e}")
A40 = 4 * PI * 2e-4**2
G_arith = A40 * 0.5 * (1e-12 + 4e-10) / (dls[f1] + drs[f1])
check("P5R-09b same face: an arithmetic mean of D_1, D_2 would overstate G by a factor of order 100",
      50 < G_arith / cond[f1] < 500, f"G_arith/G = {G_arith / cond[f1]:.1f}")
v_first = shell_volume(0.0, 5e-6)
v_edge = shell_volume(1.95e-4, 2e-4)
check("P5R-09c first kernel cell volume 5.24e-16 m^3; kernel-edge cell of equal width is about 4700 times larger",
      abs(v_first - 5.236e-16) < 1e-19 and 4600 < v_edge / v_first < 4750, f"V_1 = {v_first:.4e}, ratio = {v_edge / v_first:.1f}")
check("P5R-10 Eq. (5.11): G_R = 4 pi R^2 D_5/(R - r_200)",
      GR > 0, f"G_R = {GR:.6e} m^3/s, R - r_200 = {Rout - cells[-1]['centroid']:.6e} m")

# explicit bound (lines 238 to 252), only to quote the stiffness ratio
gwv = [0.0] + cond
gev = cond + [GR]
dtfe = min(c["volume"] / (gwv[i] + gev[i]) for i, c in enumerate(cells))
check("P5R-11 Delta t_FE = 7.812107958751e-3 s; ratio Delta t / Delta t_FE about 2.56e4",
      abs(dtfe - 7.812107958751e-3) < 1e-15,
      f"dt_FE = {dtfe!r}, ratio = {200.0/dtfe:.4e}, explicit steps to 2e5 s = {2e5/dtfe:.3e}")


# solve_diffusion_system and backward_euler_step, lines 132 to 182
def solve(v, gw, ge, rhs):
    n = len(v)
    den = [0.0] * n
    dp = [0.0] * n
    e_prev = 0.0
    for i in range(n):
        e = v[0] if i == 0 else v[i] + gw[i] * e_prev / (e_prev + gw[i])
        den[i] = e + ge[i]
        if den[i] <= 0.0 or not math.isfinite(den[i]):
            raise RuntimeError("pivot")
        carry = 0.0 if i == 0 else gw[i] * dp[i - 1]
        dp[i] = (rhs[i] + carry) / den[i]
        e_prev = e
    x = [0.0] * n
    x[n - 1] = dp[n - 1]
    for i in range(n - 2, -1, -1):
        x[i] = dp[i] + ge[i] / den[i] * x[i + 1]
    return x


def be_step(old, dt):
    v = [c["volume"] / dt for c in cells]
    rhs = [v[i] * old[i] for i in range(N)]
    return solve(v, gwv, gev, rhs), v


# dense comparison for one step
x1, v1 = be_step(C, 200.0)
Ad = np.zeros((N, N))
for i in range(N):
    Ad[i, i] = v1[i] + gwv[i] + gev[i]
    if i > 0:
        Ad[i, i - 1] = -gwv[i]
    if i < N - 1:
        Ad[i, i + 1] = -gev[i]
xd = np.linalg.solve(Ad, np.array(v1) * np.array(C))
check("P5R-12 Eq. (5.15): matrix is symmetric and the sweep (5.18), (5.19) equals a dense solve",
      np.allclose(Ad, Ad.T, rtol=0, atol=0) and np.max(np.abs(np.array(x1) - xd)) < 1e-12,
      f"max abs diff {np.max(np.abs(np.array(x1) - xd)):.1e} mol/m^3")
check("P5R-13 strict diagonal dominance margin of every row equals v_j > 0 (rows 1..N-1) or v_N + G_R",
      all(abs((Ad[i, i] - np.sum(np.abs(Ad[i])) + Ad[i, i]) - (v1[i] + (GR if i == N - 1 else 0.0))) <= 1e-12 * Ad[i, i] for i in range(N)))

# run, lines 184 to 236
steps = math.ceil(t_end / dt_in)
dt = t_end / steps
released, comp = 0.0, 0.0
naive = 0.0
rows = [(0.0, M0, 0.0, GR * C[-1], 0.0, min(C))]
worst_rel = 0.0
for step in range(1, steps + 1):
    nxt, _ = be_step(C, dt)
    rate = GR * nxt[-1]
    inc = dt * rate
    naive += inc
    y = inc - comp
    t = released + y
    comp = (t - released) - y
    released = t
    C = nxt
    inv = inventory(C)
    resid = inv + released - M0
    worst_rel = max(worst_rel, abs(resid) / M0)
    if step % n_out == 0 or step == steps:
        rows.append((step * dt, inv, released, rate, resid, min(C)))
F_end = rows[-1][2] / M0
check("P5R-14 Eq. (5.16): 1000 steps of exactly 200 s; output rows at t = 0 and every 10 steps: 101 rows",
      steps == 1000 and dt == 200.0 and len(rows) == 101 and rows[-1][0] == 2.0e5)
check("P5R-15 Eq. (5.24): F(2e5 s) = 0.4324286068785 (main run, 200 cells)",
      abs(F_end - 0.4324286068785) < 5e-13, f"F = {F_end:.14f}")
check("P5R-16 Eq. (5.24): worst relative balance residual over all 1000 steps below 1e-12",
      worst_rel < 1e-12, f"max |eps|/M0 = {worst_rel:.2e}")
check("P5R-17 t = 0 row: J_R^0 = G_R C^0_200 = 0 (kernel-only start)", rows[0][3] == 0.0)
check("P5R-18 min_j C_j >= 0 on every output row (exact, no tolerance)", all(rw[5] >= 0.0 for rw in rows))
check("P5R-19 Kahan and plain release sums agree to round-off for this run (difference reported)",
      abs(naive - released) / M0 < 1e-13, f"|plain - Kahan|/M0 = {abs(naive - released)/M0:.1e}")

print("ALL PASS" if all(results) else "SOME FAIL")
sys.exit(0 if all(results) else 1)
