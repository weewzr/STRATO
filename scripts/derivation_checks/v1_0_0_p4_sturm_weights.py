"""STRATO v1.0.0, Full Mathematical Derivation, Part IV: independent numerical checks.

Independent re-implementation of the five-layer eigenproblem of Code C for the
synthetic v1.0 input, using a DIFFERENT formulation from the Rust code:
u = alpha sin(k r) + beta cos(k r) in absolute r per layer, 2x2 matching of u and
D(u' - u/r) at each interface, sign-change scan in sqrt(Lambda) (float64) and
bisection in mpmath at 40 digits, closed-form antiderivatives for N, I_1, I.
Then checks the code-specific rules (Sturm count, lower bracket, panels, flux
ratio, floor, validity time) with a float64 replica of the Rust formulas.

Writes nothing. Prints PASS/FAIL and INFO lines.

Run:
    C:/Y3S1/Nuclear/STRATO/.venv/Scripts/python.exe -I -X utf8 \
        scripts/derivation_checks/v1_0_0_p4_sturm_weights.py
"""

import math

import mpmath as mp
import numpy as np

mp.mp.dps = 40
results = []


def check(name, ok, info=""):
    results.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}" + (f"  ({info})" if info else ""))


def info(msg):
    print(f"INFO  {msg}")


# synthetic input, inputs/synthetic/synthetic_five_layer.toml (not literature data)
RAD = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4]
DIF = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13]
NAMES = ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"]
RADm = [mp.mpf(x) for x in RAD]
DIFm = [mp.mpf(x) for x in DIF]


# ---------------- independent formulation ----------------
def coeffs_mp(lam, rad=RADm, dif=DIFm):
    """(alpha, beta, k) per layer, u = alpha sin(k r) + beta cos(k r), kernel alpha = 1."""
    out = []
    al, be = mp.mpf(1), mp.mpf(0)
    for i, (Ro, Di) in enumerate(zip(rad, dif)):
        ki = mp.sqrt(lam / Di)
        out.append((al, be, ki))
        if i + 1 < len(rad):
            sn, cs = mp.sin(ki * Ro), mp.cos(ki * Ro)
            u0 = al * sn + be * cs
            f0 = Di * (ki * (al * cs - be * sn) - u0 / Ro)  # D (u' - u/r) on the inner side
            kn = mp.sqrt(lam / dif[i + 1])
            sn2, cs2 = mp.sin(kn * Ro), mp.cos(kn * Ro)
            v0 = f0 / dif[i + 1] + u0 / Ro  # u' on the outer side
            al, be = sn2 * u0 + cs2 * v0 / kn, cs2 * u0 - sn2 * v0 / kn
    return out


def g_mp(lam, rad=RADm, dif=DIFm):
    al, be, kk = coeffs_mp(lam, rad, dif)[-1]
    return al * mp.sin(kk * rad[-1]) + be * mp.cos(kk * rad[-1])


def g_np(lam):
    lam = np.asarray(lam, dtype=float)
    al, be = np.ones_like(lam), np.zeros_like(lam)
    for i, (Ro, Di) in enumerate(zip(RAD, DIF)):
        ki = np.sqrt(lam / Di)
        sn, cs = np.sin(ki * Ro), np.cos(ki * Ro)
        u0 = al * sn + be * cs
        if i + 1 == len(RAD):
            return u0
        f0 = Di * (ki * (al * cs - be * sn) - u0 / Ro)
        kn = np.sqrt(lam / DIF[i + 1])
        sn2, cs2 = np.sin(kn * Ro), np.cos(kn * Ro)
        v0 = f0 / DIF[i + 1] + u0 / Ro
        al, be = sn2 * u0 + cs2 * v0 / kn, cs2 * u0 - sn2 * v0 / kn


def roots_up_to(lam_top, npts=600001):
    x = np.linspace(1e-9, math.sqrt(lam_top) * 1.02, npts)
    g = g_np(x * x)
    idx = np.where(np.sign(g[1:]) * np.sign(g[:-1]) < 0)[0]
    roots = []
    for i in idx:
        a, b = mp.mpf(x[i]) ** 2, mp.mpf(x[i + 1]) ** 2
        ga = g_mp(a)
        for _ in range(140):
            m = (a + b) / 2
            gm = g_mp(m)
            if ga * gm <= 0:
                b = m
            else:
                a, ga = m, gm
        roots.append((a + b) / 2)
    return [lr for lr in roots if lr <= lam_top]


def u_eval_np(lam, rr):
    """u(r) for a root, float64 values from the mp coefficients."""
    cf = coeffs_mp(lam)
    out = np.zeros_like(rr)
    left = 0.0
    for (al, be, kk), Ro in zip(cf, RAD):
        msk = (rr > left) & (rr <= Ro) if left > 0 else (rr >= 0) & (rr <= Ro)
        out[msk] = float(al) * np.sin(float(kk) * rr[msk]) + float(be) * np.cos(float(kk) * rr[msk])
        left = Ro
    return out


def integrals_mp(lam):
    """N = int u^2, I1 = int_0^R1 r u, I = int_0^R r u, closed-form antiderivatives."""
    cf = coeffs_mp(lam)
    N = I = I1 = mp.mpf(0)
    left = mp.mpf(0)
    for idx, ((al, be, kk), Ro) in enumerate(zip(cf, RADm)):
        def Fu2(x):
            return (al**2 * (x / 2 - mp.sin(2 * kk * x) / (4 * kk))
                    + be**2 * (x / 2 + mp.sin(2 * kk * x) / (4 * kk))
                    + 2 * al * be * mp.sin(kk * x) ** 2 / (2 * kk))

        def Fru(x):
            return (al * (mp.sin(kk * x) / kk**2 - x * mp.cos(kk * x) / kk)
                    + be * (mp.cos(kk * x) / kk**2 + x * mp.sin(kk * x) / kk))

        N += Fu2(Ro) - Fu2(left)
        m = Fru(Ro) - Fru(left)
        I += m
        if idx == 0:
            I1 = m
        left = Ro
    return N, I1, I


# ---------------- float64 replica of the Rust count and quadrature ----------------
def transfer(u, q, d, k, length):
    c, s = math.cos(k * length), math.sin(k * length)
    return c * u + s * q / (d * k), -d * k * s * u + c * q


def count_code(lam):
    n, u, q, left, prev = 0, 0.0, 1.0, 0.0, None
    for Ro, Di in zip(RAD, DIF):
        if prev is not None:
            q = q + (Di - prev) * u / left
        k = math.sqrt(lam / Di)
        th = math.atan2(u, q / (Di * k))
        L = Ro - left
        n += math.floor((th + k * L) / math.pi) - math.floor(th / math.pi)
        u, q = transfer(u, q, Di, k, L)
        m = max(abs(u), abs(q / (Di * k)))
        if m > 0 and math.isfinite(m):
            u, q = u / m, q / m
        left, prev = Ro, Di
    return max(n, 0)


def starts_code(lam):
    out, u, q, left, prev = [], 0.0, 1.0, 0.0, None
    for Ro, Di in zip(RAD, DIF):
        if prev is not None:
            q = q + (Di - prev) * u / left
        k = math.sqrt(lam / Di)
        out.append((u, q, k, Di, left, Ro))
        u, q = transfer(u, q, Di, k, Ro - left)
        left, prev = Ro, Di
    return out


XG, WG = np.polynomial.legendre.leggauss(16)


def weight_code(lam):
    st = starts_code(lam)
    N = I = I1 = 0.0
    for idx, (u, q, k, d, left, right) in enumerate(st):
        a, b = u, q / (d * k)
        P = math.ceil(k * (right - left) / (0.5 * math.pi)) + 1
        h = (right - left) / P
        for p in range(P):
            mid, half = left + p * h + 0.5 * h, 0.5 * h
            rr = mid + half * XG
            uu = a * np.cos(k * (rr - left)) + b * np.sin(k * (rr - left))
            N += half * np.sum(WG * uu * uu)
            mm = half * np.sum(WG * rr * uu)
            I += mm
            if idx == 0:
                I1 += mm
    return 3 * I1 * I / (RAD[0] ** 3 * N)


# ================= eigenvalues =================
lam_main = mp.mpf(45) / 2000  # t_min = 10 steps x 200 s, CMP 58-59
roots_all = roots_up_to(0.2)
roots = [x for x in roots_all if x <= lam_main]
info(f"independent roots: {len(roots)} with Lambda <= 0.0225 s^-1, {len(roots_all)} with Lambda <= 0.2 s^-1")
for i in range(6):
    info(f"Lambda_{i + 1} = {mp.nstr(roots[i], 20)} s^-1")
check("32 eigenvalues with Lambda <= 45/2000 = 0.0225 s^-1 (main run 'modes = 32')", len(roots) == 32)
rust1, rust2 = 3.273255671175805e-6, 7.791893309400181e-5
rel1 = abs(float(roots[0]) - rust1) / rust1
rel2 = abs(float(roots[1]) - rust2) / rust2
check("Lambda_1 = 3.273255671175805e-6 s^-1 (Rust) within rel 1e-12", rel1 < 1e-12, f"independent {mp.nstr(roots[0], 16)}, rel diff {rel1:.1e}")
check("Lambda_2 = 7.791893309400181e-5 s^-1 (Rust) within rel 1e-12", rel2 < 1e-12, f"independent {mp.nstr(roots[1], 16)}, rel diff {rel2:.1e}")
check("g(Lambda_n) = 0 to 40-digit working precision (relative to max |u|)",
      all(abs(g_mp(x)) < mp.mpf(10) ** -30 * max(abs(c[0]) + abs(c[1]) for c in coeffs_mp(x)) for x in roots))

# oscillation: mode n has n - 1 zeros in (0, R)
rr = np.linspace(0.0, RAD[-1] * (1 - 1e-9), 2_000_001)[1:]
ok, bad = True, []
for n, lam in enumerate(roots_all, start=1):
    uu = u_eval_np(lam, rr)
    z = int(np.sum(np.sign(uu[1:]) * np.sign(uu[:-1]) < 0))
    if z != n - 1:
        ok = False
        bad.append((n, z))
check(f"mode n has exactly n - 1 interior zeros, n = 1..{len(roots_all)} (no root skipped by the scan)", ok, f"bad {bad[:3]}")

# ================= Sturm count (code replica) =================
ok = True
for n, lam in enumerate(roots_all, start=1):
    lf = float(lam)
    ok &= count_code(lf * (1 - 1e-9)) == n - 1 and count_code(lf * (1 + 1e-9)) == n
check(f"4.13 code count N(Lambda) steps from n-1 to n across each of {len(roots_all)} eigenvalues", ok)
rng = np.random.default_rng(4)
ok = True
for lam in np.exp(rng.uniform(math.log(1e-7), math.log(0.2), 300)):
    ok &= count_code(lam) == sum(1 for x in roots_all if x <= lam)
check("4.13 N(Lambda) = #{Lambda_n <= Lambda} at 300 random Lambda in [1e-7, 0.2]", ok)
info(f"N(0.0225) = {count_code(0.0225)}, N(2.25e-4) = {count_code(2.25e-4)}")

# lower bracket 4.14
phase_rate = sum((Ro - Lf) / math.sqrt(Di) for Ro, Lf, Di in zip(RAD, [0.0] + RAD[:-1], DIF))
lam_lo = 1e-6 * (1 / phase_rate) ** 2
check("4.14 Lambda_lo = 1e-6 (sum L/sqrt(D))^-2 has N = 0 and lies below Lambda_1",
      count_code(lam_lo) == 0 and lam_lo < float(roots[0]),
      f"sum L/sqrt(D) = {phase_rate:.6g} s^1/2, Lambda_lo = {lam_lo:.4e} s^-1, phase at Lambda_lo = {phase_rate * math.sqrt(lam_lo):.1e} rad")

# ================= worked interface chain, mode 1, code scale q(0) = 1 =================
lam1 = roots[0]
cf = coeffs_mp(lam1)
scale = 1 / (DIFm[0] * cf[0][2])  # alpha_kernel such that q(0) = D_1 u'(0) = 1
print("CHAIN mode 1, Lambda_1 = %s s^-1, code scale q(0) = 1" % mp.nstr(lam1, 16))
print("CHAIN layer  k (1/m)       panels")
for i, ((al, be, kk), Ro) in enumerate(zip(cf, RAD)):
    Lw = Ro - ([0.0] + RAD)[i]
    print(f"CHAIN {NAMES[i]:7s} {mp.nstr(kk, 6):>12s}  {math.ceil(float(kk) * Lw / (0.5 * math.pi)) + 1}")
print("CHAIN interface   R (m)    u(R)           q_minus        q_plus         D(u'-u/r) inner   D(u'-u/r) outer   rho(outer side)")
ok = True
for i in range(4):
    Ro = RADm[i]
    al, be, kk = cf[i]
    al2, be2, kk2 = cf[i + 1]
    u_in = scale * (al * mp.sin(kk * Ro) + be * mp.cos(kk * Ro))
    up_in = scale * kk * (al * mp.cos(kk * Ro) - be * mp.sin(kk * Ro))
    u_out = scale * (al2 * mp.sin(kk2 * Ro) + be2 * mp.cos(kk2 * Ro))
    up_out = scale * kk2 * (al2 * mp.cos(kk2 * Ro) - be2 * mp.sin(kk2 * Ro))
    qm, qp = DIFm[i] * up_in, DIFm[i + 1] * up_out
    fin, fout = qm - DIFm[i] * u_in / Ro, qp - DIFm[i + 1] * u_out / Ro
    rho = abs(fout) / (abs(qp) + abs(DIFm[i + 1] * u_out / Ro))
    ok &= abs(u_in - u_out) <= mp.mpf(10) ** -30 * abs(u_in)
    ok &= abs(qp - (qm + (DIFm[i + 1] - DIFm[i]) * u_in / Ro)) <= mp.mpf(10) ** -30 * (abs(qm) + abs(qp))
    ok &= abs(fin - fout) <= mp.mpf(10) ** -28 * (abs(qm) + abs(qp))
    print(f"CHAIN {NAMES[i]}/{NAMES[i + 1]:6s} {RAD[i]:.2e} {mp.nstr(u_in, 8):>14s} {mp.nstr(qm, 8):>14s} "
          f"{mp.nstr(qp, 8):>14s} {mp.nstr(fin, 8):>16s} {mp.nstr(fout, 8):>16s} {mp.nstr(rho, 4):>10s}")
alR, beR, kR = cf[-1]
gR = scale * (alR * mp.sin(kR * RADm[-1]) + beR * mp.cos(kR * RADm[-1]))
print(f"CHAIN u(R) = g(Lambda_1) = {mp.nstr(gR, 3)}")
check("4.7-4.8 mode 1: at all four interfaces u is continuous, q+ = q- + (D+ - D-) u/R, D(u'-u/r) continuous", ok)
# code replica gives the same chain
st = starts_code(float(lam1))
okc = True
for i in range(1, 5):
    u_c, q_c = st[i][0], st[i][1]
    Ro = RADm[i - 1]
    al2, be2, kk2 = cf[i]
    u_ind = float(scale * (al2 * mp.sin(kk2 * Ro) + be2 * mp.cos(kk2 * Ro)))
    q_ind = float(DIFm[i] * scale * kk2 * (al2 * mp.cos(kk2 * Ro) - be2 * mp.sin(kk2 * Ro)))
    okc &= abs(u_c - u_ind) <= 1e-9 * abs(u_ind) and abs(q_c - q_ind) <= 1e-9 * abs(q_ind)
check("4.6-4.9 float64 replica of shoot (transfer_layer + interface_state) reproduces the chain to rel 1e-9", okc)

# ================= weights, F, completeness =================
W, Ns = [], []
for lam in roots_all:
    N, I1, I = integrals_mp(lam)
    W.append(3 * I1 * I / (RADm[0] ** 3 * N))
w32 = W[:32]
info("w_1..w_6 = " + ", ".join(mp.nstr(x, 13) for x in w32[:6]))
check("4.22 w_1 = 1.092030273751353, w_2 = -0.1423582853238343 (audit 01 Rust) within 1e-12",
      abs(w32[0] - mp.mpf("1.092030273751353")) < 1e-12 and abs(w32[1] - mp.mpf("-0.1423582853238343")) < 1e-12)
S32 = mp.fsum(w32)
check("4.23 sum of 32 weights = 1.000007275592095 (audit 01 oracle) within 1e-12; Rust 1.000007275592173",
      abs(S32 - mp.mpf("1.000007275592095")) < 1e-12, f"sum = {mp.nstr(S32, 16)}")


def F(t, lams, ws):
    return 1 - mp.fsum(wn * mp.exp(-ln * t) for ln, wn in zip(lams, ws))


F2e5 = F(mp.mpf(2e5), roots, w32)
check("4.22 F(2e5 s) = 0.43255563909304 (abs < 5e-15)", abs(F2e5 - mp.mpf("0.43255563909304")) < 5e-15, f"F = {mp.nstr(F2e5, 17)}")
check("4.22 F(2e5 s) equals 50-digit oracle 0.43255563909304174 (abs < 1e-16)",
      abs(F2e5 - mp.mpf("0.43255563909304174")) < 1e-16)
check("4.22 F(2e5 s) is within 1e-10 of the repo test constant 0.4325556390930383 (TST1 22-30)",
      abs(F2e5 - mp.mpf("0.4325556390930383")) < 1e-10)
two = [x for x in roots if x <= mp.mpf(45) / 2e5]
F2 = F(mp.mpf(2e5), two, w32[:len(two)])
check("4.26 study reference (Lambda_max = 45/t_end): 2 modes give the same F(2e5 s) to 1e-20",
      len(two) == 2 and abs(F2 - F2e5) < 1e-20, f"modes {len(two)}, diff {mp.nstr(abs(F2 - F2e5), 3)}")
info(f"F(2000 s) with 32 modes = {mp.nstr(F(mp.mpf(2000), roots, w32), 4)} (Rust gives -8.55e-14, audit 01)")
for lm in [2e-3, 2e-2, 2.25e-2, 2e-1]:
    K = sum(1 for x in roots_all if x <= lm)
    info(f"Lambda_max = {lm:g}: K = {K}, 1 - sum w = {mp.nstr(1 - mp.fsum(W[:K]), 4)}")
gaps = [abs(1 - mp.fsum(W[:sum(1 for x in roots_all if x <= lm)])) for lm in [2e-3, 2e-2, 2e-1]]
check("4.23 |1 - sum w| decreases for Lambda_max = 2e-3, 2e-2, 2e-1 (VER 228-246 rule)",
      gaps[0] > gaps[1] > gaps[2] and gaps[2] < 5e-2)

# quadrature replica against closed form
ok, worst = True, 0.0
for lam, wv in zip(roots, w32):
    wc = weight_code(float(lam))
    worst = max(worst, abs(wc - float(wv)))
check("4.24 composite GL16 with ceil(kL/(pi/2)) + 1 panels reproduces the closed-form w_n (32 modes, abs < 1e-12)",
      worst < 1e-12, f"max abs diff {worst:.1e}")

# orthogonality (float64 high-order Gauss per layer, normalised u)
xg, wg = np.polynomial.legendre.leggauss(200)
nodes, wts = [], []
left = 0.0
for Ro in RAD:
    for p in range(20):
        lo, hi = left + (Ro - left) * p / 20, left + (Ro - left) * (p + 1) / 20
        nodes.append(0.5 * (lo + hi) + 0.5 * (hi - lo) * xg)
        wts.append(0.5 * (hi - lo) * wg)
    left = Ro
nodes, wts = np.concatenate(nodes), np.concatenate(wts)
Umat = np.array([u_eval_np(lam, nodes) for lam in roots])
G = (Umat * wts) @ Umat.T
Gn = G / np.sqrt(np.outer(np.diag(G), np.diag(G)))
off = np.max(np.abs(Gn - np.diag(np.diag(Gn))))
G2 = (Umat * wts / nodes**2) @ Umat.T
G2n = G2 / np.sqrt(np.outer(np.diag(G2), np.diag(G2)))
off2 = np.max(np.abs(G2n - np.diag(np.diag(G2n))))
check("4.17 int u_m u_n dr = 0 for m != n (32 modes, normalised Gram off-diagonal < 1e-10)",
      off < 1e-10 and off2 > 1e-2, f"max {off:.1e}; weight 1 for phi instead gives {off2:.2f}")

# Rayleigh quotient: Lambda = int D (u' - u/r)^2 dr / int u^2 dr  (> 0)
ok = True
for lam in roots[:4]:
    cfl = coeffs_mp(lam)
    num = den = mp.mpf(0)
    left = mp.mpf(0)
    for (al, be, kk), Ro, Di in zip(cfl, RADm, DIFm):
        uf = lambda x: al * mp.sin(kk * x) + be * mp.cos(kk * x)
        upf = lambda x: kk * (al * mp.cos(kk * x) - be * mp.sin(kk * x))
        pts = mp.linspace(left if left > 0 else mp.mpf(10) ** -30, Ro, 9)
        num += Di * mp.quad(lambda x: (upf(x) - uf(x) / x) ** 2, pts)
        den += mp.quad(lambda x: uf(x) ** 2, pts)
        left = Ro
    ok &= abs(num / den - lam) < mp.mpf(10) ** -25 * lam
check("4.17 Rayleigh quotient int D(u' - u/r)^2 / int u^2 = Lambda_n > 0 (modes 1 to 4)", ok)

# flux ratio and floor (code side: post-jump q, outer D)
rho_out = rho_in = math.inf
for lam in roots:
    st = starts_code(float(lam))
    for i in range(1, 5):
        u, q, k, d, left, right = st[i]
        dprev = st[i - 1][3]
        qin = q - (d - dprev) * u / left
        sc_o = abs(q) + abs(d * u / left)
        sc_i = abs(qin) + abs(dprev * u / left)
        flux = abs(q - d * u / left)
        rho_out = min(rho_out, flux / sc_o)
        rho_in = min(rho_in, abs(qin - dprev * u / left) / sc_i)
floor = 2.2e-16 / min(rho_out, 1.0)
check("4.27-4.28 rho_min = 5.504e-5 and eps_F = 3.997e-12 (main run) within rel 1e-3",
      abs(rho_out / 5.504e-5 - 1) < 1e-3 and abs(floor / 3.997e-12 - 1) < 1e-3,
      f"rho_min {rho_out:.4e}, eps_F {floor:.4e}; inner side would give {rho_in:.4e}, floor {2.2e-16 / rho_in:.2e}")
check("4.28 Rust F(2e5 s) = 0.43255563909285666 (audit 01) lies within eps_F of the independent value",
      abs(mp.mpf("0.43255563909285666") - F2e5) < floor, f"diff {mp.nstr(abs(mp.mpf('0.43255563909285666') - F2e5), 3)}")
check("4.26 t_valid = 40/0.0225 = 1777.78 s < first output 2000 s", abs(40 / 0.0225 - 1777.7777777) < 1e-6 and 40 / 0.0225 < 2000)

# homogeneous limit with the independent solver
Dh = [mp.mpf(1e-10)] * 5
okh = True
for n in range(1, 6):
    lam_ex = n**2 * mp.pi**2 * Dh[0] / RADm[-1] ** 2
    okh &= abs(g_mp(lam_ex, RADm, Dh)) < mp.mpf(10) ** -30
    okh &= abs(g_mp(lam_ex * (1 + mp.mpf(10) ** -6), RADm, Dh)) > mp.mpf(10) ** -12
check("4.29 equal D: g(n^2 pi^2 D/R^2) = 0 with the five-layer matching (n = 1..5)", okh)

print(f"SUMMARY {sum(results)}/{len(results)} PASS")
print("ALL PASS" if all(results) else "SOME FAIL")
