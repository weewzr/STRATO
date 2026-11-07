"""Phase 2.1 (section 4): FV against the Booth single-sphere solutions of TECDOC-1674.

Checks:
  B1. steady R/B with source and decay, TECDOC-1674 Eq. (7.6): the FV steady state
      (equal to the BE fixed point) converges at order 2 in space.
  B2. transient R/B(t) Eq. (7.7) and F(t) Eq. (7.8) = M_out/(M + M_out): BE with source and
      decay, error falls at order 1 in dt on a fixed fine mesh.
  B3. CRP-6 cases 1a/1b (bare kernel, uniform initial Cs, no source, 200 h): Booth Eq. (10.7)
      with the two-branch TECDOC-978 FRG Cs-in-UO2 D reproduces the printed analytical
      values 0.4673 and 0.99999959 (TECDOC-1674 Table 10.6); the single branch of
      TECDOC-1674 Table 7.1 does not. FV values at two (N, dt) settings. Effect of Cs-137 decay.

Data are from audit notes 03, 04 and 07 (see section 4 text). Synthetic values are marked.
Run: .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec4_p21_booth_crp6.py
Writes nothing to disk. Prints PASS/FAIL lines.
"""
import math

import mpmath as mp

ok_all = True


def report(name, ok, detail=""):
    global ok_all
    ok_all &= bool(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}", flush=True)


def sphere_mesh(R, D, N):
    """One-material sphere, v1.0 geometry: uniform faces, volume centroids, two-point G."""
    f = [R * k / N for k in range(N + 1)]
    V = [4 * math.pi / 3 * (f[j + 1]**3 - f[j]**3) for j in range(N)]
    cen = [0.75 * f[1] if j == 0 else 0.75 * (f[j + 1]**4 - f[j]**4) / (f[j + 1]**3 - f[j]**3) for j in range(N)]
    G = [4 * math.pi * f[j + 1]**2 * D / (cen[j + 1] - cen[j]) for j in range(N - 1)]
    GR = 4 * math.pi * R**2 * D / (R - cen[-1])
    return V, [0.0] + G, G + [GR], GR


def solve_sf(v, gw, ge, rhs):
    n = len(v)
    den, dp = [0.0] * n, [0.0] * n
    e_prev = 0.0
    for i in range(n):
        e = v[i] if i == 0 else v[i] + gw[i] * e_prev / (e_prev + gw[i])
        den[i] = e + ge[i]
        dp[i] = (rhs[i] + (gw[i] * dp[i - 1] if i else 0.0)) / den[i]
        e_prev = e
    x = [0.0] * n
    x[-1] = dp[-1]
    for i in range(n - 2, -1, -1):
        x[i] = dp[i] + ge[i] / den[i] * x[i + 1]
    return x


# ---------------------------------------------------------------- B1, B2 Booth with source and decay
R = 250e-6
Dp = 1e-5                     # synthetic reduced diffusivity D' = D/R^2 (s^-1)
D = Dp * R**2
lam = 6.88e-5                 # Kr-88 decay constant, TECDOC-1674 Eq. (5.4) (audit 04 section 2.4)
a = lam / Dp
mp.mp.dps = 30
x = mp.sqrt(a)
RB76 = float(3 / x * (mp.coth(x) - 1 / x))


def RB77(t):
    return RB76 - float(6 * mp.exp(-lam * t) * mp.nsum(lambda n: mp.exp(-n**2 * mp.pi**2 * Dp * t) / (n**2 * mp.pi**2 + a), [1, mp.inf]))


def F78(t):
    tail = mp.nsum(lambda n: (1 - mp.exp(-n**2 * mp.pi**2 * Dp * t)) / (n**2 * (n**2 * mp.pi**2 + a)), [1, mp.inf])
    return RB76 - float(6 / mp.pi**2 * a / mp.expm1(lam * t) * tail)


errs = []
for N in (25, 50, 100, 200):
    V, gw, ge, GR = sphere_mesh(R, D, N)
    C = solve_sf([lam * Vj for Vj in V], gw, ge, [Vj * 1.0 for Vj in V])     # (K + lam V) C = V S, S = 1
    errs.append(abs(GR * C[-1] / sum(V) - RB76) / RB76)
o = [math.log2(errs[i] / errs[i + 1]) for i in range(3)]
report("B1 steady R/B: FV (= BE fixed point) -> Eq. (7.6), spatial order 2", abs(o[-1] - 2) < 0.1,
       f"(7.6) R/B = {RB76:.8f}; rel errors {[f'{e:.2e}' for e in errs]}; orders {[f'{q:.3f}' for q in o]}")

tE = 5e4
N = 400
V, gw, ge, GR = sphere_mesh(R, D, N)
Gen = sum(V)                     # S = 1 mol m^-3 s^-1 everywhere
eRB, eF = [], []
for dt in (400.0, 200.0, 100.0, 50.0):
    C = [0.0] * N
    Mout = 0.0
    for k in range(int(round(tE / dt))):
        C = solve_sf([Vj / dt + lam * Vj for Vj in V], gw, ge, [Vj / dt * c + Vj for Vj, c in zip(V, C)])
        Mout = (Mout + dt * GR * C[-1]) / (1 + lam * dt)
    M = sum(Vj * c for Vj, c in zip(V, C))
    eRB.append(GR * C[-1] / Gen)
    eF.append(Mout / (M + Mout))
# temporal order from successive differences (removes the fixed O(h^2) spatial error of the N = 400 mesh)
oRB = [math.log2(abs(eRB[i] - eRB[i + 1]) / abs(eRB[i + 1] - eRB[i + 2])) for i in range(2)]
oF = [math.log2(abs(eF[i] - eF[i + 1]) / abs(eF[i + 1] - eF[i + 2])) for i in range(2)]
errRB = [abs(q - RB77(tE)) for q in eRB]
errF = [abs(q - F78(tE)) for q in eF]
report("B2 transient R/B(t) (7.7) and F(t) (7.8) at t = 5e4 s: BE temporal order ~ 1, errors < 1e-4 at the finest step dt = 50 s",
       abs(oRB[-1] - 1) < 0.1 and abs(oF[-1] - 1) < 0.1 and errRB[-1] < 1e-4 and errF[-1] < 1e-4,
       f"(7.7) {RB77(tE):.6f}, (7.8) {F78(tE):.6f}; |err R/B| {[f'{e:.1e}' for e in errRB]}; |err F| {[f'{e:.1e}' for e in errF]}; "
       f"difference orders {[f'{q:.2f}' for q in oRB]} / {[f'{q:.2f}' for q in oF]}")

# ---------------------------------------------------------------- B3 CRP-6 cases 1a / 1b
Rk = 250e-6                               # kernel radius, TECDOC-1674 Table 10.2 (500 um diameter)
t200 = 200 * 3600.0                       # 200 h = 720 000 s
Rg_1674 = 8.3145                          # TECDOC-1674 Eq. (10.5a)
Rg_SI = 8.314462618                       # N_A k_B (SI 2019 exact product, truncated)


def D_cs_two(T, Rg):                      # TECDOC-978 Table A-1 FRG Cs in UO2, two branches (audit 03)
    return 5.6e-8 * math.exp(-209000 / (Rg * T)) + 5.2e-4 * math.exp(-362000 / (Rg * T))


def D_cs_one(T, Rg):                      # TECDOC-1674 Table 7.1 Cs, D'_0 = 0.90 s^-1 -> D_0 = D'_0 r_k^2
    return 0.90 * Rk**2 * math.exp(-209000 / (Rg * T))


def booth107(Dt, lam_=0.0, t=None):
    """Fractional release M_rel/M_0 of a uniformly loaded sphere; lam_ > 0 adds decay (P21.81)."""
    if lam_ == 0.0:
        return float(1 - 6 / mp.pi**2 * mp.nsum(lambda n: mp.exp(-n**2 * mp.pi**2 * Dt) / n**2, [1, mp.inf]))
    Dpr = Dt / t

    def term(n):
        L = n**2 * mp.pi**2 * Dpr
        return 6 / (n**2 * mp.pi**2) * L / (L + lam_) * (1 - mp.exp(-(L + lam_) * t))
    return float(mp.nsum(term, [1, mp.inf]))


printed = {"1a": 0.4673, "1b": 0.99999959}     # TECDOC-1674 Table 10.6, analytical row (audit 04)
res = {}
for case, Tc in (("1a", 1200.0), ("1b", 1600.0)):
    T = Tc + 273.15
    two = [booth107(D_cs_two(T, Rg) * t200 / Rk**2) for Rg in (Rg_1674, Rg_SI, 8.314)]
    one = booth107(D_cs_one(T, Rg_1674) * t200 / Rk**2)
    res[case] = (two, one)
    print(f"      case {case}: T = {T} K, D(two-branch) = {D_cs_two(T, Rg_1674):.4e} m^2/s, D't = {D_cs_two(T, Rg_1674)*t200/Rk**2:.5f}; "
          f"F(two-branch, R_g = 8.3145 / 8.314462618 / 8.314) = {two[0]:.8f} / {two[1]:.8f} / {two[2]:.8f}; F(single branch) = {one:.8f}; printed {printed[case]}")
# printed 0.4673 (4 digits, +-5e-5) must lie inside the R_g bracket [8.314, 8.3145] widened by the rounding
lo, hi = min(res["1a"][0]) - 5e-5, max(res["1a"][0]) + 5e-5
ok1a = lo <= 0.4673 <= hi and abs(res["1a"][1] - 0.4673) > 3e-3
ok1b = abs((1 - res["1b"][0][0]) / (1 - 0.99999959) - 1) < 0.03 and abs(res["1b"][1] - 0.99999959) > 1e-5
report("B3a two-branch FRG Cs D reproduces Table 10.6 analytical 1a/1b, single branch does not", ok1a and ok1b)

lam_cs = 7.302187e-10                     # Cs-137, ENSDF via LiveChart (audit 07)
for case, Tc in (("1a", 1200.0), ("1b", 1600.0)):
    T = Tc + 273.15
    Dt = D_cs_two(T, Rg_1674) * t200 / Rk**2
    Fd = booth107(Dt, lam_cs, t200)
    Mfrac = 1 - booth107(Dt)                                  # retained fraction without decay
    Fcorr = 1 - math.exp(lam_cs * t200) * (math.exp(-lam_cs * t200) * Mfrac)   # 1 - e^{lam t} M_lam/M_0 (P21.80)
    print(f"      case {case}: with Cs-137 decay, M_rel/M_0 = {Fd:.8f} (change {Fd - booth107(Dt):+.2e}); "
          f"decay-corrected 1 - e^(lam t) M/M_0 = {Fcorr:.8f}")

# FV for the bare kernel: uniform initial concentration, no source, no decay (the benchmark definition)
fv = {}
for case, Tc in (("1a", 1200.0), ("1b", 1600.0)):
    Dk = D_cs_two(Tc + 273.15, Rg_1674)
    for (N, dt) in ((150, 360.0), (150, 36.0)):          # (150, 360 s) is the FRESCO-II case-1 setting, TECDOC-1674 Table 10.4
        V, gw, ge, GR = sphere_mesh(Rk, Dk, N)
        C = [1.0] * N
        M0 = sum(V)
        rel = []
        for k in range(int(round(t200 / dt))):
            C = solve_sf([Vj / dt for Vj in V], gw, ge, [Vj / dt * c for Vj, c in zip(V, C)])
            rel.append(dt * GR * C[-1])
        F = math.fsum(rel) / M0
        clos = abs(sum(Vj * c for Vj, c in zip(V, C)) + math.fsum(rel) - M0) / M0
        fv[(case, N, dt)] = (F, clos)
        print(f"      FV case {case}, N = {N}, dt = {dt:.0f} s: F = {F:.8f}, 1 - F = {1-F:.3e}, closure {clos:.1e}", flush=True)
ok_fv = (abs(fv[("1a", 150, 36.0)][0] - res["1a"][0][0]) < 2e-4
         and abs(math.log((1 - fv[("1b", 150, 36.0)][0]) / (1 - res["1b"][0][0]))) < 0.05
         and all(v[1] < 1e-12 for v in fv.values()))
report("B3b FV (N = 150, dt = 36 s) matches Booth (10.7): |dF| < 2e-4 (1a), 1-F within 5 % (1b); closure < 1e-12", ok_fv)

print("ALL PASS" if ok_all else "SOME CHECKS FAILED")
