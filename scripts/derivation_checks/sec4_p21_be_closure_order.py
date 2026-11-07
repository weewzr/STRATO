"""Phase 2.1 (section 4): numerical checks of the Backward-Euler scheme with decay and source.

A Python transcription of the v1.0 mesh and subtraction-free solver (src/numerical/finite_volume.rs)
with the two Phase 2.1 changes of (P21.59): diagonal V/dt + lam V + G_W + G_E and right-hand
side (V/dt) C^n + V Sbar^{n+1}. Five-layer TECDOC-1674 Table 10.2 geometry, synthetic D.

Checks:
  N1. per-step identity (P21.70) and cumulative closure (P21.72) to round-off, with a
      switching kernel source, decay and variable aligned steps.
  N2. positivity: min C >= 0.0 exactly in floating point, including huge steps.
  N3. semi-discrete identity exp(-(V^-1 K + lam I) t) = exp(-lam t) exp(-V^-1 K t) and
      BE convergence at order 1 in time to it (S = 0).
  N4. (P21.63) BE with decay at dt equals (1 + lam dt)^-n times v1.0 BE at dt/(1 + lam dt).
  N5. regression: lam = 0, S = 0 reproduces the v1.0 step bit for bit.
  N6. decay only (all G = 0): C^n = (1 + lam dt)^-n C^0, order 1 to exp(-lam t).
  N7. M_out updates (P21.73), (P21.74): M_rel = M_out + M_dec,out, steady limit J_R/lam.
  N8. steady state: BE fixed point equals the semi-discrete steady state for any dt;
      the decay-exact integrating-factor scheme (P21.66) has the shifted fixed point (P21.68).
  N9. switch not aligned with a step: rectangle source at n+1 versus exact step average.

Run: .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec4_p21_be_closure_order.py
Writes nothing to disk. Prints PASS/FAIL lines.
"""
import math

import numpy as np
from scipy.linalg import expm

ok_all = True


def report(name, ok, detail=""):
    global ok_all
    ok_all &= bool(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


# ---------------------------------------------------------------- v1.0 geometry and solver
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
    cen = np.array([0.75 * b if a == 0 else 0.75 * (b**4 - a**4) / (b**3 - a**3) for a, b in zip(left, right)])
    G = np.array([4 * math.pi * right[j]**2 / ((right[j] - cen[j]) / D[j] + (cen[j + 1] - right[j]) / D[j + 1])
                  for j in range(len(V) - 1)])
    GR = 4 * math.pi * radii[-1]**2 * D[-1] / (radii[-1] - cen[-1])
    gw = np.concatenate(([0.0], G))
    ge = np.concatenate((G, [GR]))
    return dict(V=V, gw=gw, ge=ge, GR=GR, mat=np.array(mat), n=len(V))


def solve_sf(v, gw, ge, rhs):
    """Subtraction-free elimination, transcribed from solve_diffusion_system (v1.0)."""
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


def step_v1(m, C, dt):
    """v1.0 backward_euler_step: v = V/dt, rhs = v * old."""
    v = m["V"] / dt
    return solve_sf(v, m["gw"], m["ge"], v * C)


def step_v2(m, C, dt, lam, Sbar):
    """Phase 2.1 step (P21.58): v = V/dt + lam V (added, so lam = 0 is bitwise v1.0),
    rhs = (V/dt) C + V Sbar (the old-value term must NOT use the decay-augmented v)."""
    vt = m["V"] / dt
    return solve_sf(vt + lam * m["V"], m["gw"], m["ge"], vt * C + m["V"] * Sbar)


def K_matrix(m):
    n = m["n"]
    K = np.zeros((n, n))
    for i in range(n):
        K[i, i] = m["gw"][i] + m["ge"][i]
        if i > 0:
            K[i, i - 1] = -m["gw"][i]
        if i + 1 < n:
            K[i, i + 1] = -m["ge"][i]
    return K


radii = [250e-6, 350e-6, 390e-6, 425e-6, 465e-6]       # TECDOC-1674 Table 10.2 sensitivity geometry (m)
Ds = [4e-14, 4e-13, 2e-14, 5e-15, 2e-14]               # synthetic (m^2 s^-1)
m = build_mesh(radii, Ds, [20, 8, 4, 4, 4])
kern = (m["mat"] == 0).astype(float)
C0 = kern * 1.0                                         # 1 mol m^-3 in the kernel
M0 = float(np.sum(m["V"] * C0))

# ---------------------------------------------------------------- N1 closure with switching source
lam = 2e-8
sched = [(0.0, 3e7, 1e-6), (3e7, 5e7, 0.0), (5e7, 8e7, 2.5e-6)]  # (start, end, kernel source mol m^-3 s^-1)
C = C0.copy()
Mgen, Mdec, Mrel, step_res = [], [], [], []
Mprev = M0
times = []
for (ta, tb, Sk) in sched:                 # steps aligned with switch times, geometric growth after each switch
    tt, dt = ta, 1e3
    while tt < tb - 1e-9:
        dt_k = min(dt, tb - tt)
        Sbar = kern * Sk                    # exact step average (source constant inside the step)
        Cn = step_v2(m, C, dt_k, lam, Sbar)
        Mn = float(np.sum(m["V"] * Cn))
        gen = dt_k * float(np.sum(m["V"] * Sbar))
        dec = dt_k * lam * Mn
        rel = dt_k * m["GR"] * Cn[-1]
        step_res.append(abs((Mn - Mprev) - (gen - dec - rel)) / max(Mprev, Mn, gen))
        Mgen.append(gen); Mdec.append(dec); Mrel.append(rel)
        C, Mprev, tt, dt = Cn, Mn, tt + dt_k, dt * 1.5
        times.append(tt)
closure = abs(M0 + math.fsum(Mgen) - Mprev - math.fsum(Mdec) - math.fsum(Mrel)) / (M0 + math.fsum(Mgen))
report("N1a per-step identity M^{n+1}-M^n = dt(Gen - lam M^{n+1} - G_R C_N^{n+1}) (P21.70)", max(step_res) < 1e-13,
       f"max rel {max(step_res):.1e} over {len(step_res)} steps")
report("N1b cumulative closure M0 + M_gen = M + M_dec + M_rel (P21.72)", closure < 1e-13,
       f"rel residual {closure:.1e}; M_gen {math.fsum(Mgen):.4e}, M_dec {math.fsum(Mdec):.4e}, M_rel {math.fsum(Mrel):.4e}, M {Mprev:.4e} mol")

# ---------------------------------------------------------------- N2 positivity, exact
mins = []
for dt in (1e-3, 1e2, 1e6, 1e12):
    C = C0.copy()
    for k in range(20):
        C = step_v2(m, C, dt, 1e-9, kern * 1e-7 * (k % 2))
        mins.append(C.min())
report("N2 min C >= 0.0 exactly in floating point for dt = 1e-3 .. 1e12 s", min(mins) >= 0.0, f"min {min(mins):.3e}")

# ---------------------------------------------------------------- N3 order 1 to the e^{-lam t} identity
K = K_matrix(m)
A = K / m["V"][:, None]                                    # V^-1 K
# accurate reference through the symmetric form V^-1/2 K V^-1/2 (eigh), not a stiff expm
sq = np.sqrt(m["V"])
mu, Q = np.linalg.eigh(K / np.outer(sq, sq))
Lam1 = mu.min()                                            # slowest semi-discrete decay rate (s^-1)
T = 1.0 / Lam1                                             # one slowest time constant
lam = Lam1                                                 # lam T = 1
def semi_discrete(tt, lam_):
    return (Q @ (np.exp(-(mu + lam_) * tt) * (Q.T @ (sq * C0)))) / sq
ref_full = semi_discrete(T, lam)
ref_split = math.exp(-lam * T) * semi_discrete(T, 0.0)
ref_expm = expm(-(A + lam * np.eye(m["n"])) * T) @ C0
sd = max(np.max(np.abs(ref_full - ref_split)), np.max(np.abs(ref_full - ref_expm))) / np.max(np.abs(ref_full))
print(f"      N3 data: Lambda_1(semi-discrete) = {Lam1:.4e} s^-1, T = {T:.4e} s, lam = Lambda_1, M(T)/M0 = {np.sum(m['V']*ref_full)/M0:.4f}")
report("N3a semi-discrete identity exp(-(A + lam I)T) C0 = e^{-lam T} exp(-A T) C0 (P21.67)", sd < 1e-12, f"rel {sd:.1e}")
errs, errs_if = [], []
Mref = float(np.sum(m["V"] * ref_full))
for n in (25, 50, 100, 200, 400, 800):
    dt = T / n
    C = C0.copy()
    Cif = C0.copy()
    for k in range(n):
        C = step_v2(m, C, dt, lam, 0.0 * kern)
        Cif = math.exp(-lam * dt) * step_v1(m, Cif, dt)    # decay exact, diffusion BE (P21.66 with S = 0)
    errs.append(abs(float(np.sum(m["V"] * C)) - Mref) / Mref)
    errs_if.append(abs(float(np.sum(m["V"] * Cif)) - Mref) / Mref)
ords = [math.log2(errs[i] / errs[i + 1]) for i in range(len(errs) - 1)]
ords_if = [math.log2(errs_if[i] / errs_if[i + 1]) for i in range(len(errs_if) - 1)]
report("N3b unsplit BE: error in M(T) vs semi-discrete reference, order -> 1", abs(ords[-1] - 1) < 0.05,
       f"errors {[f'{e:.2e}' for e in errs]}, orders {[f'{o:.3f}' for o in ords]}")
report("N3c decay-exact split: order -> 1, smaller constant", abs(ords_if[-1] - 1) < 0.05 and errs_if[-1] < errs[-1],
       f"errors {[f'{e:.2e}' for e in errs_if]}")

lam, T = 2e-8, 5e7                                         # back to the N1 data for N4 to N9 (lam T = 1)

# ---------------------------------------------------------------- N4 (1 + z)^-n identity
dt, n = 4e5, 60
z = lam * dt
C = C0.copy()
Cv1 = C0.copy()
for k in range(n):
    C = step_v2(m, C, dt, lam, 0.0 * kern)
    Cv1 = step_v1(m, Cv1, dt / (1 + z))
d4 = np.max(np.abs(C - Cv1 / (1 + z)**n)) / np.max(np.abs(C))
report("N4 BE(lam, dt)^n = (1 + lam dt)^-n BE_v1(dt/(1 + lam dt))^n (P21.63)", d4 < 1e-13, f"rel {d4:.1e}")

# ---------------------------------------------------------------- N5 bitwise regression lam = 0, S = 0
C = C0.copy()
Cv1 = C0.copy()
same = True
for k in range(200):
    C = step_v2(m, C, 3.3e4, 0.0, 0.0 * kern)
    Cv1 = step_v1(m, Cv1, 3.3e4)
    same &= np.array_equal(C, Cv1)
report("N5 lam = 0, S = 0: identical to v1.0 bit for bit over 200 steps", same)

# ---------------------------------------------------------------- N6 decay only
md = dict(m)
md["gw"] = np.zeros(m["n"])
md["ge"] = np.zeros(m["n"])
errs6 = []
for n in (10, 20, 40, 80, 160):
    dt = T / n
    C = C0.copy()
    for k in range(n):
        C = step_v2(md, C, dt, lam, 0.0 * kern)
    exact_disc = C0 * (1 + lam * dt)**(-n)
    assert np.max(np.abs(C - exact_disc)) <= 1e-14
    errs6.append(abs(C[0] - math.exp(-lam * T)) / math.exp(-lam * T))
ords6 = [math.log2(errs6[i] / errs6[i + 1]) for i in range(len(errs6) - 1)]
pred = 0.5 * lam * T * lam * (T / 160)                      # leading term lam^2 t dt / 2 (P21.77)
report("N6 decay only: C^n = (1+z)^-n C0 exactly; order 1 to e^{-lam t}; error ~ lam^2 t dt/2",
       abs(ords6[-1] - 1) < 0.02 and abs(errs6[-1] / pred - 1) < 0.02,
       f"errors {[f'{e:.2e}' for e in errs6]}, predicted last {pred:.2e}")

# ---------------------------------------------------------------- N7 M_out bookkeeping
C = 0.0 * C0
Mout_be, Mout_ex, Mrel, Mdec_out, Mdec_in, Mgen = 0.0, 0.0, [], [], [], []
dt, Sk = 2e5, 1e-6
for k in range(4000):                                      # 8e8 s = 16 mean lives
    C = step_v2(m, C, dt, lam, kern * Sk)
    J = m["GR"] * C[-1]
    Mrel.append(dt * J)
    Mout_be = (Mout_be + dt * J) / (1 + lam * dt)          # (P21.73)
    Mout_ex = math.exp(-lam * dt) * Mout_ex + J * (-math.expm1(-lam * dt)) / lam   # (P21.74)
    Mdec_out.append(dt * lam * Mout_be)
    Mdec_in.append(dt * lam * float(np.sum(m["V"] * C)))
    Mgen.append(dt * float(np.sum(m["V"] * kern * Sk)))
Min = float(np.sum(m["V"] * C))
r7a = abs(math.fsum(Mrel) - Mout_be - math.fsum(Mdec_out)) / math.fsum(Mrel)
r7b = abs(math.fsum(Mgen) - Min - Mout_be - math.fsum(Mdec_in) - math.fsum(Mdec_out)) / math.fsum(Mgen)
report("N7a BE M_out: M_rel = M_out + M_dec,out (P21.75)", r7a < 1e-13, f"rel {r7a:.1e}")
report("N7b whole system: M_gen = M + M_out + M_dec,in + M_dec,out", r7b < 1e-13, f"rel {r7b:.1e}")
r7c = abs(Mout_be - J / lam) / (J / lam)
r7d = abs(Mout_ex - J / lam) / (J / lam)
report("N7c both M_out updates approach J_R/lam at steady state (P21.76)", r7c < 1e-6 and r7d < 1e-6,
       f"BE {r7c:.1e}, exponential {r7d:.1e}, (M_out_BE - M_out_exp)/M_out = {(Mout_be-Mout_ex)/Mout_ex:.1e}")

# ---------------------------------------------------------------- N8 steady-state preservation
Sbar = kern * 1e-6
Gen = float(np.sum(m["V"] * Sbar))
out = []
for lam8 in (2e-8, 7e-6):                                  # long-lived (lam << Lambda_1) and short-lived (lam ~ 10 Lambda_1)
    Css = np.linalg.solve(K + lam8 * np.diag(m["V"]), m["V"] * Sbar)
    Mss = float(np.sum(m["V"] * Css))
    RBss = m["GR"] * Css[-1] / Gen
    for zz in (0.1, 1.0, 5.0):
        dt = zz / lam8
        C = 0.0 * C0
        Cif = 0.0 * C0
        for k in range(3000):
            C = step_v2(m, C, dt, lam8, Sbar)
            Ct = solve_sf(m["V"] / dt, m["gw"], m["ge"], (m["V"] / dt) * (Cif + Sbar * math.expm1(zz) / lam8))
            Cif = math.exp(-zz) * Ct                       # (P21.66)
        fac = zz * math.exp(zz) / math.expm1(zz)
        Cif_pred = np.linalg.solve(fac * K + lam8 * np.diag(m["V"]), m["V"] * Sbar)
        Mif = float(np.sum(m["V"] * Cif))
        RBif = 1 - lam8 * Mif / Gen                        # R/B consistent with the IF scheme's own balance
        out.append((lam8, zz, abs(m["GR"] * C[-1] / Gen - RBss) / RBss, (Mif - Mss) / Mss, (RBif - RBss) / RBss,
                    np.max(np.abs(Cif - Cif_pred)) / np.max(Cif_pred), RBss))
okbe = all(o[2] < 1e-12 for o in out)
okif = all(o[5] < 1e-12 for o in out) and all(abs(o[3]) > 1e-3 for o in out)
report("N8a BE fixed point = semi-discrete steady state, lam dt = 0.1, 1, 5, two lam (R/B exact)", okbe,
       f"max rel R/B diff {max(o[2] for o in out):.1e}; steady R/B = {out[0][6]:.6f} (lam 2e-8), {out[3][6]:.6f} (lam 7e-6)")
report("N8b integrating-factor fixed point is (z e^z/(e^z-1) K + lam V) C = V S (P21.68): biased M_ss and R/B",
       okif, "; ".join(f"lam={o[0]:.0e},z={o[1]}: dM/M {o[3]:+.2e}, dRB/RB {o[4]:+.2e}" for o in out))

# ---------------------------------------------------------------- N9 non-aligned switch
# source on [0, ts) only; uniform steps of 1e6 s; ts = 2.4e6 s lies inside the third step
dt, ts, Sk = 1e6, 2.4e6, 1e-6
true_gen = ts * Sk * float(np.sum(m["V"] * kern))
gen_rect = sum(dt * Sk * float(np.sum(m["V"] * kern)) for k in range(1, 6) if k * dt <= ts)     # S at t^{n+1}
gen_avg = sum(dt * (max(0.0, min(ts, k * dt) - (k - 1) * dt) / dt) * Sk * float(np.sum(m["V"] * kern)) for k in range(1, 6))
report("N9 exact step average reproduces the generated amount; rectangle at n+1 misses part of a step",
       abs(gen_avg - true_gen) / true_gen < 1e-15 and abs(gen_rect - true_gen) / true_gen > 0.1,
       f"rectangle error {(gen_rect-true_gen)/true_gen:+.3f}, step average error {(gen_avg-true_gen)/true_gen:+.1e}")

print("ALL PASS" if ok_all else "SOME CHECKS FAILED")
