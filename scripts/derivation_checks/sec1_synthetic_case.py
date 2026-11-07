"""Section 1 (Phase 1, items 9 to 14): independent numerical reproduction of Code C.

Run with:
    .venv/Scripts/python.exe -I -X utf8 sec1_synthetic_case.py

What it does
1. Independent eigen-solver (formulation different from Code C):
   phi_i = (A_i sin(k_i r) + B_i cos(k_i r)) / r in ABSOLUTE r, 2x2 matching of
   [phi, D phi'] at each interface (P1.14, P1.15), start A_1 = 1, B_1 = 0 (P1.13),
   characteristic value g_phi(Lambda) = u_5(R) (P1.28). Arithmetic in mpmath, 40 digits.
   Roots: float64 sign scan in sqrt(Lambda), then mpmath bisection refinement.
   Integrals: closed forms (P1.33, P1.35), not quadrature.
2. Re-implementation of Code C's own algorithm (transfer of (u, q = D u'), jump rule
   P1.26, Sturm phase count P1.29, min_flux_ratio) to check it line by line.
3. Checks: target eigenvalues and F(2e5 s) of the v1.0 synthetic case, Sturm count,
   orthogonality with weight 1 in u (P1.32), modal closure, sum of weights (P1.38),
   homogeneous limit, Gelbard (2003) Section V geometry with all gamma = 1.
Nothing is written to disk. Every check prints PASS or FAIL.
"""

import math
import numpy as np
import mpmath as mp

mp.mp.dps = 40
results = []


def report(tag, ok, detail=""):
    results.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {tag}  {detail}")


# ---------------------------------------------------------------- inputs (synthetic)
RADII = ["2.0e-4", "3.0e-4", "3.5e-4", "4.0e-4", "4.5e-4"]      # m
DIFFS = ["1.0e-12", "4.0e-10", "8.0e-13", "2.0e-14", "7.0e-13"]  # m^2 s^-1
T_MIN = 2.0e3        # s, first FV output time (10 steps x 200 s) in the v1.0 main run
LAMBDA_MAX = 45.0 / T_MIN   # s^-1, as in comparison.rs line 59
T_EVAL = 2.0e5       # s
TARGET_L1 = 3.273255671175805e-6
TARGET_L2 = 7.791893309400181e-5
# F(2e5 s): audit 01 Sec. 1.10 item 2. The 50-digit oracle and the Rust v1.0 value differ
# by 1.9e-13 (Code C round-off, below its floor 3.997e-12); test each against its own role.
ORACLE_F = "0.43255563909304174"     # independent 50-digit Python oracle (audit 01)
RUST_F = 0.43255563909285666         # Rust v1.0 Code C output (audit 01)
RUST_FLOOR = 3.997e-12               # analytical_accuracy_floor_abs of the v1.0 run


# ---------------------------------------------------------------- 1. phi-form solver
def coeffs_phi(lam, radii, diffs):
    """Return per-layer (A_i, B_i, k_i) with u_i = A_i sin(k_i r) + B_i cos(k_i r)."""
    out = []
    A, B = mp.mpf(1), mp.mpf(0)
    for i, (Ro, Dv) in enumerate(zip(radii, diffs)):
        k = mp.sqrt(lam / Dv)
        out.append((A, B, k))
        if i + 1 == len(radii):
            break
        Dn = diffs[i + 1]
        kn = mp.sqrt(lam / Dn)
        sn, cn = mp.sin(k * Ro), mp.cos(k * Ro)
        uval = A * sn + B * cn                         # u continuous (P1.25)
        flux = Dv * (k * (A * cn - B * sn) - uval / Ro)  # D (u' - u/r) continuous
        s2, c2 = mp.sin(kn * Ro), mp.cos(kn * Ro)
        M = mp.matrix([[s2, c2], [Dn * kn * c2, -Dn * kn * s2]])
        rhs = mp.matrix([uval, flux + Dn * uval / Ro])
        A, B = mp.lu_solve(M, rhs)
    return out


def g_phi(lam, radii, diffs):
    A, B, k = coeffs_phi(lam, radii, diffs)[-1]
    R = radii[-1]
    return A * mp.sin(k * R) + B * mp.cos(k * R)


def g_phi_float(z, radii, diffs):
    """Vectorised float64 version (z = sqrt(Lambda)) used only for sign scanning."""
    rr = [float(v) for v in radii]
    dd = [float(v) for v in diffs]
    A = np.ones_like(z)
    B = np.zeros_like(z)
    for i in range(len(rr) - 1):
        k = z / math.sqrt(dd[i])
        kn = z / math.sqrt(dd[i + 1])
        Ro = rr[i]
        sn, cn = np.sin(k * Ro), np.cos(k * Ro)
        uval = A * sn + B * cn
        flux = dd[i] * (k * (A * cn - B * sn) - uval / Ro) + dd[i + 1] * uval / Ro
        s2, c2 = np.sin(kn * Ro), np.cos(kn * Ro)
        # solve [[s2, c2], [Dn kn c2, -Dn kn s2]] [A, B] = [uval, flux]; det = -Dn kn
        Dk = dd[i + 1] * kn
        A = (uval * Dk * s2 + flux * c2) / Dk
        B = (uval * Dk * c2 - flux * s2) / Dk
        # scale to avoid overflow; zeros of g are scale free
        m = np.maximum(np.abs(A), np.abs(B))
        A, B = A / m, B / m
    k = z / math.sqrt(dd[-1])
    return A * np.sin(k * rr[-1]) + B * np.cos(k * rr[-1])


def find_eigs(radii, diffs, lam_max, npts=400_000):
    z = np.linspace(1e-9, math.sqrt(lam_max), npts)
    g = g_phi_float(z, radii, diffs)
    idx = np.nonzero(np.sign(g[:-1]) * np.sign(g[1:]) < 0)[0]
    eigs = []
    for i in idx:
        lo, hi = mp.mpf(z[i]) ** 2, mp.mpf(z[i + 1]) ** 2
        glo = g_phi(lo, radii, diffs)
        for _ in range(140):  # bisection to ~40 digits
            mid = (lo + hi) / 2
            gm = g_phi(mid, radii, diffs)
            if glo * gm <= 0:
                hi = mid
            else:
                lo, glo = mid, gm
        eigs.append((lo + hi) / 2)
    return eigs


def mode_integrals(lam, radii, diffs):
    """Closed-form I_1, I_n = int_0^R r u dr, N_n = int_0^R u^2 dr (P1.33, P1.35)."""
    cf = coeffs_phi(lam, radii, diffs)
    I1 = In = Nn = mp.mpf(0)
    left = mp.mpf(0)
    for i, ((A, B, k), Ro) in enumerate(zip(cf, radii)):
        def Pr(rv):   # antiderivative of r (A sin kr + B cos kr)
            return (A * (mp.sin(k * rv) / k**2 - rv * mp.cos(k * rv) / k)
                    + B * (mp.cos(k * rv) / k**2 + rv * mp.sin(k * rv) / k))

        def Pu2(rv):  # antiderivative of (A sin kr + B cos kr)^2
            return (A**2 * (rv / 2 - mp.sin(2 * k * rv) / (4 * k))
                    + B**2 * (rv / 2 + mp.sin(2 * k * rv) / (4 * k))
                    - A * B * mp.cos(2 * k * rv) / (2 * k))
        m = Pr(Ro) - Pr(left)
        In += m
        if i == 0:
            I1 = m
        Nn += Pu2(Ro) - Pu2(left)
        left = Ro
    return I1, In, Nn, cf


def series(radii, diffs, lam_max):
    radii = [mp.mpf(v) for v in radii]
    diffs = [mp.mpf(v) for v in diffs]
    eigs = find_eigs(radii, diffs, lam_max)
    R1 = radii[0]
    data = []
    for lam in eigs:
        I1, In, Nn, cf = mode_integrals(lam, radii, diffs)
        w = 3 * I1 * In / (R1**3 * Nn)
        data.append(dict(lam=lam, w=w, I1=I1, In=In, N=Nn, cf=cf))
    return radii, diffs, data


def F_of_t(data, tv):
    return 1 - mp.fsum(d["w"] * mp.exp(-d["lam"] * tv) for d in data)


# ---------------------------------------------------------------- 2. Code C re-implementation
def codec_shoot(lam, radii, diffs, rescale=False, prec=None):
    """Literal transcription of sturm_liouville.rs::shoot (lines 132-162)."""
    u, q = mp.mpf(0), mp.mpf(1)
    left = mp.mpf(0)
    prev = None
    starts = []
    for Ro, Dv in zip(radii, diffs):
        if prev is not None:
            q = q + (Dv - prev) * u / left          # interface_state, line 98-100
        k = mp.sqrt(lam / Dv)
        starts.append((u, q, k, Dv, left, Ro))
        L = Ro - left
        c, s = mp.cos(k * L), mp.sin(k * L)          # transfer_layer, lines 86-92
        u, q = c * u + s * q / (Dv * k), -Dv * k * s * u + c * q
        if rescale:
            m = max(abs(u), abs(q / (Dv * k)))
            if m > 0:
                u, q = u / m, q / m
        left = Ro
        prev = Dv
    return u, starts


def codec_count(lam, radii, diffs):
    """Literal transcription of eigenvalue_count (lines 166-192), float64."""
    n = 0
    u, q = 0.0, 1.0
    left = 0.0
    prev = None
    for Ro, Dv in zip(radii, diffs):
        if prev is not None:
            q = q + (Dv - prev) * u / left
        k = math.sqrt(lam / Dv)
        a_, b_ = u, q / (Dv * k)
        th0 = math.atan2(a_, b_)
        L = Ro - left
        n += math.floor((th0 + k * L) / math.pi) - math.floor(th0 / math.pi)
        c, s = math.cos(k * L), math.sin(k * L)
        u, q = c * u + s * q / (Dv * k), -Dv * k * s * u + c * q
        m = max(abs(u), abs(q / (Dv * k)))
        if m > 0 and math.isfinite(m):
            u, q = u / m, q / m
        left = Ro
        prev = Dv
    return max(n, 0)


def codec_min_flux_ratio_f64(lams, radii, diffs):
    """Literal float64 transcription of ModalRelease::new lines 329-336."""
    best = math.inf
    for lam in lams:
        u, q = 0.0, 1.0
        left = 0.0
        prev = None
        for Ro, Dv in zip(radii, diffs):
            if prev is not None:
                q = q + (Dv - prev) * u / left
                scale = abs(q) + abs(Dv * u / left)
                if scale > 0:
                    best = min(best, abs(q - Dv * u / left) / scale)
            k = math.sqrt(lam / Dv)
            L = Ro - left
            c, s = math.cos(k * L), math.sin(k * L)
            u, q = c * u + s * q / (Dv * k), -Dv * k * s * u + c * q
            left = Ro
            prev = Dv
    return best


# ================================================================ run: synthetic case
radii, diffs, data = series(RADII, DIFFS, LAMBDA_MAX)
lams = [d["lam"] for d in data]
nm = len(lams)
print(f"synthetic case: {nm} modes with Lambda <= {LAMBDA_MAX:.4e} s^-1")
for i in range(6):
    print(f"  Lambda_{i+1} = {mp.nstr(lams[i], 16)} s^-1   w = {mp.nstr(data[i]['w'], 16)}")

rel1 = abs(lams[0] / TARGET_L1 - 1)
rel2 = abs(lams[1] / TARGET_L2 - 1)
report("Lambda_1 matches v1.0 (rel < 1e-11)", rel1 < 1e-11, f"(rel diff {float(rel1):.2e})")
report("Lambda_2 matches v1.0 (rel < 1e-11)", rel2 < 1e-11, f"(rel diff {float(rel2):.2e})")
Fv = F_of_t(data, mp.mpf(T_EVAL))
report("F(2e5 s) matches audit 50-digit oracle (abs < 1e-15)",
       abs(Fv - mp.mpf(ORACLE_F)) < 1e-15,
       f"(F = {mp.nstr(Fv, 17)}, diff {float(Fv - mp.mpf(ORACLE_F)):.2e})")
report("Rust v1.0 F(2e5 s) lies within its accuracy floor of the exact value",
       abs(float(Fv) - RUST_F) < RUST_FLOOR,
       f"(exact - Rust = {float(Fv) - RUST_F:.2e}, floor {RUST_FLOOR:.3e})")
sw = mp.fsum(d["w"] for d in data)
report("sum of 32 truncated weights equals audit oracle 1.000007275592095 (abs < 1e-12)",
       abs(sw - mp.mpf("1.000007275592095")) < 1e-12, f"(sum = {mp.nstr(sw, 16)})")
report("weights w_1, w_2 match audit (1.092030273751353, -0.1423582853238343; abs < 1e-12)",
       abs(data[0]["w"] - mp.mpf("1.092030273751353")) < 1e-12
       and abs(data[1]["w"] - mp.mpf("-0.1423582853238343")) < 1e-12)

# --- Sturm count (P1.29): literal Code C count vs independent root list
cnt_max = codec_count(LAMBDA_MAX, [float(v) for v in radii], [float(v) for v in diffs])
report("Code C Sturm count at Lambda_max equals number of roots found", cnt_max == nm,
       f"(count {cnt_max}, roots {nm})")
ok = True
for i, lam in enumerate(lams):
    lo = codec_count(float(lam) * (1 - 1e-9), [float(v) for v in radii], [float(v) for v in diffs])
    hi = codec_count(float(lam) * (1 + 1e-9), [float(v) for v in radii], [float(v) for v in diffs])
    ok = ok and lo == i and hi == i + 1
report("Sturm count steps by one at every eigenvalue (N(Lambda_n^-) = n-1, N(Lambda_n^+) = n)", ok)
# independent zero count of u(r) at the top eigenvalue: n-th mode has n-1 interior zeros
okz = True
for idx in (0, 1, 5, nm - 1):
    cf = data[idx]["cf"]
    left = 0.0
    zeros = 0
    prev_sign = None
    for (A, B, k), Ro in zip(cf, radii):
        rr = np.linspace(left, float(Ro), 40_001)[1:]
        uu = float(A) * np.sin(float(k) * rr) + float(B) * np.cos(float(k) * rr)
        uu = uu[:-1] if Ro == radii[-1] else uu      # exclude r = R where u = 0
        sg = np.sign(uu)
        if prev_sign is not None:
            sg = np.concatenate([[prev_sign], sg])
        zeros += int(np.sum(sg[:-1] * sg[1:] < 0))
        prev_sign = sg[-1]
        left = float(Ro)
    okz = okz and zeros == idx
report("mode n has n-1 zeros in (0,R) (checked n = 1, 2, 6, 32 on dense r grid)", okz)

# --- characteristic functions: Code C (u,q) shooting = phi matching / (D_1 k_1) (P1.26-P1.28)
okg = True
worst = 0
for lam in [mp.mpf("1.3e-6"), mp.mpf("4.4e-5"), mp.mpf("2.0e-4"), mp.mpf("7.7e-3"), mp.mpf("0.02")]:
    gc, _ = codec_shoot(lam, radii, diffs)
    gp = g_phi(lam, radii, diffs) / (diffs[0] * mp.sqrt(lam / diffs[0]))
    rel = abs(gc / gp - 1)
    worst = max(worst, rel)
    okg = okg and rel < mp.mpf("1e-30")
report("Code C g(Lambda) = u(R) equals independent phi-matching g / (D_1 k_1)", okg,
       f"(max rel diff {mp.nstr(worst, 3)})")

# --- min_flux_ratio and accuracy floor (P1.39)
mfr = codec_min_flux_ratio_f64([float(v) for v in lams], [float(v) for v in radii],
                               [float(v) for v in diffs])
floor = 2.2e-16 / min(mfr, 1.0)
report("min_flux_ratio reproduces v1.0 run value 5.504e-5 (rel < 1e-3)",
       abs(mfr / 5.504e-5 - 1) < 1e-3, f"(ratio {mfr:.4e}, floor {floor:.3e})")

# --- orthogonality (P1.32): Gram matrix, weight 1 in u, vs weight 1 in phi
nodes, wts = np.polynomial.legendre.leggauss(64)
rq, wq = [], []
left = 0.0
for Ro in radii:
    Ro = float(Ro)
    npan = 40
    for p in range(npan):
        a_ = left + (Ro - left) * p / npan
        b_ = left + (Ro - left) * (p + 1) / npan
        rq.append(0.5 * (b_ - a_) * nodes + 0.5 * (a_ + b_))
        wq.append(0.5 * (b_ - a_) * wts)
    left = Ro
lay_r = rq
rq, wq = np.concatenate(rq), np.concatenate(wq)
U = np.zeros((nm, rq.size))
bounds = [0.0] + [float(v) for v in radii]
for j, d in enumerate(data):
    for li, (A, B, k) in enumerate(d["cf"]):
        msk = (rq >= bounds[li]) & (rq <= bounds[li + 1])
        U[j, msk] = float(A) * np.sin(float(k) * rq[msk]) + float(B) * np.cos(float(k) * rq[msk])
G = (U * wq) @ U.T
Gn = G / np.sqrt(np.outer(np.diag(G), np.diag(G)))
off = np.max(np.abs(Gn - np.eye(nm)))
Gphi = (U / rq * wq) @ (U / rq).T
Gphin = Gphi / np.sqrt(np.outer(np.diag(Gphi), np.diag(Gphi)))
offphi = np.max(np.abs(Gphin - np.eye(nm)))
report("orthogonality with weight r^2 for phi (= 1 for u): max |offdiag| < 1e-9", off < 1e-9,
       f"(max {off:.2e}; weight 1 for phi gives {offphi:.2e})")
nerr = max(abs(G[j, j] / float(d["N"]) - 1) for j, d in enumerate(data))
report("closed-form N_n agrees with 64-pt Gauss quadrature (rel < 1e-10)", nerr < 1e-10,
       f"(max rel {nerr:.1e})")

# --- modal inventory closure (P1.19, P1.36): Lambda_n I_n = -R D_5 u_n'(R)
okc, worst = True, 0
for d in data:
    A, B, k = d["cf"][-1]
    R = radii[-1]
    du = k * (A * mp.cos(k * R) - B * mp.sin(k * R))
    rel = abs(d["lam"] * d["In"] / (-R * diffs[-1] * du) - 1)
    worst = max(worst, rel)
    okc = okc and rel < 1e-25
report("modal closure Lambda_n I_n = -R D_5 u_n'(R) for all 32 modes", okc,
       f"(max rel {mp.nstr(worst, 3)})")

# --- Rayleigh quotient (positivity of Lambda_n): Lambda_n N_n = int D (u' - u/r)^2 dr
okr, worst = True, 0.0
for j in range(8):
    d = data[j]
    dU = np.zeros(rq.size)
    Dq = np.zeros(rq.size)
    for li, (A, B, k) in enumerate(d["cf"]):
        msk = (rq >= bounds[li]) & (rq <= bounds[li + 1])
        kf = float(k)
        dU[msk] = kf * (float(A) * np.cos(kf * rq[msk]) - float(B) * np.sin(kf * rq[msk]))
        Dq[msk] = float(diffs[li])
    ray = np.sum(wq * Dq * (dU - U[j] / rq) ** 2) / np.sum(wq * U[j] ** 2)
    rel = abs(ray / float(d["lam"]) - 1)
    worst = max(worst, rel)
    okr = okr and rel < 1e-9
report("Rayleigh quotient int D (u'-u/r)^2 / int u^2 = Lambda_n > 0 (modes 1-8)", okr,
       f"(max rel {worst:.1e})")

# --- completeness trend (P1.38) and early-time behaviour (P1.39)
_, _, data4 = series(RADII, DIFFS, 16 * LAMBDA_MAX)
_, _, data16 = series(RADII, DIFFS, 256 * LAMBDA_MAX)
gaps = [1 - mp.fsum(d["w"] for d in dd) for dd in (data, data4, data16)]
print(f"  1 - sum w_n: {len(data)} modes {mp.nstr(gaps[0], 4)}, {len(data4)} modes "
      f"{mp.nstr(gaps[1], 4)}, {len(data16)} modes {mp.nstr(gaps[2], 4)}")
# Partial sums oscillate (w_n alternate in sign, |w_n| ~ n^-2 because the initial
# condition jumps at R_1), so test the envelope of |1 - S_N| over blocks of N.
cnt16 = codec_count(256 * LAMBDA_MAX, [float(v) for v in radii], [float(v) for v in diffs])
report("Code C Sturm count equals roots found at 256 Lambda_max", cnt16 == len(data16),
       f"(count {cnt16}, roots {len(data16)})")
S = np.cumsum([float(d["w"]) for d in data16])
gap = np.abs(1 - S)
env_a = gap[31:128].max()
env_b = gap[384:].max()
print(f"  envelope max|1 - S_N|: N in [32,128] {env_a:.3e}; N in [385,{len(S)}] {env_b:.3e}")
report("envelope of |1 - S_N| shrinks as modes are added (completeness, P1.38)",
       env_b < 0.25 * env_a)
F_more = F_of_t(data16, mp.mpf(T_EVAL))
report("F(2e5 s) unchanged by adding modes (abs < 1e-15)", abs(F_more - Fv) < 1e-15,
       f"({len(data16)} modes: diff {float(F_more - Fv):.1e})")
Fe = F_of_t(data, mp.mpf(T_MIN))
Fe16 = F_of_t(data16, mp.mpf(T_MIN))
report(f"early time t = 2e3 s: 32-mode F agrees with {len(data16)}-mode F to 1e-14",
       abs(Fe - Fe16) < 1e-14,
       f"(F32 = {mp.nstr(Fe, 3)}, F_more = {mp.nstr(Fe16, 3)})")
F0 = F_of_t(data, mp.mpf(0))
report("t = 0: truncated series gives F = 1 - sum w_n != 0 (invalid region, as expected)",
       abs(F0) > 1e-7, f"(F(0) = {mp.nstr(F0, 4)}; valid from 40/Lambda_max = {40/LAMBDA_MAX:.4e} s)")

# ================================================================ limiting case: equal D
Dh = "1.0e-10"
_, _, datah = series(RADII, [Dh] * 5, 45.0 / 20.0)
Rv, R1v, Dv = mp.mpf("4.5e-4"), mp.mpf("2.0e-4"), mp.mpf(Dh)


def kernel_closed(tv, nmax=4000):
    rem = 0
    for n in range(1, nmax):
        an = n * mp.pi / Rv
        i1 = (mp.sin(an * R1v) - an * R1v * mp.cos(an * R1v)) / an**2
        i_n = Rv**2 * (-1)**(n + 1) / (n * mp.pi)
        rem += 3 * i1 * i_n / (R1v**3 * Rv / 2) * mp.exp(-an**2 * Dv * tv)
    return 1 - rem


mp.mp.dps = 25
worst = max(abs(F_of_t(datah, mp.mpf(tv)) - kernel_closed(mp.mpf(tv))) for tv in (20, 100, 500, 2000))
report("homogeneous limit: series = closed form (t = 20..2000 s, abs < 1e-12)", worst < 1e-12,
       f"(max diff {mp.nstr(worst, 3)})")
lam_h_err = max(abs(d["lam"] / (Dv * ((j + 1) * mp.pi / Rv)**2) - 1) for j, d in enumerate(datah[:20]))
report("homogeneous limit: Lambda_n = n^2 pi^2 D / R^2 (first 20, rel < 1e-20)", lam_h_err < 1e-20)
mp.mp.dps = 40

# ================================================================ Gelbard (2003) Section V, gamma = 1
RG = ["300e-6", "360e-6", "390e-6", "415e-6", "460e-6"]
DG = ["8e-13", "2e-11", "2e-12", "1.2e-13", "2e-12"]
_, _, dg = series(RG, DG, 45.0 / 360.0)
lg = dg[0]["lam"]
f7 = F_of_t(dg, mp.mpf(7 * 3600))
report("Gelbard Sec. V geometry, gamma = 1: Lambda_1 = 2.79968e-5 s^-1 (audit 02, 6 sig. fig.)",
       abs(lg / mp.mpf("2.79968e-5") - 1) < 5e-6, f"({mp.nstr(lg, 8)})")
report("Gelbard Sec. V geometry, gamma = 1: F(7 h) = 0.42400 (audit 02, 5 dp)",
       abs(f7 - mp.mpf("0.42400")) < 5e-6, f"({mp.nstr(f7, 8)})")

print()
print(f"SUMMARY {sum(results)}/{len(results)} PASS" + ("" if all(results) else "  (FAILURES)"))
