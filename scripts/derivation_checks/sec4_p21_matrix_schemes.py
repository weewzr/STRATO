"""Phase 2.1 (section 4): symbolic checks of the discrete algebra.

Checks (SymPy):
  X1. (P21.61) Gaussian-elimination pivots of the BE matrix with decay equal d_j = e_j + g^E_j
      with e_1 = v_1, e_j = v_j + g^W_j e_{j-1}/(e_{j-1} + g^W_j), v_j = V_j (1/dt + lam).
  X2. (P21.60) row excess a_jj - sum |a_jk| = v_j (j < N) and v_N + G_R (last row).
  X3. (P21.63) (V/dt + lam V + K)^-1 V/dt = (1+z)^-1 (V/dt' + K)^-1 V/dt', dt' = dt/(1+z).
  X4. (P21.68) fixed point of the integrating-factor scheme (P21.66).
  X5. (P21.77) BE decay factor: (1+z)^-n e^{lam t} - 1 = lam^2 t dt / 2 + higher order.
  X6. (P21.73)-(P21.76) M_out updates: fixed points J/lam, exactness of the exponential
      update for a step-constant J, BE local error O(dt^2).
  X7. (P21.79) rectangle-at-(n+1) source: generation error per step -dt^2 S'/2 + O(dt^3).
  X8. (P21.54) cell average of a uniform kernel source for a cell that straddles R_1.

Run: .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec4_p21_matrix_schemes.py
Writes nothing to disk. Prints PASS/FAIL lines.
"""
import sympy as sp

ok_all = True


def report(name, ok, detail=""):
    global ok_all
    ok_all &= bool(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


N = 5
dt, lam = sp.symbols("Delta_t lambda", positive=True)
V = sp.symbols("V1:%d" % (N + 1), positive=True)
g = sp.symbols("g1:%d" % N, positive=True)        # internal face conductances, g_j between cells j and j+1
GR = sp.symbols("G_R", positive=True)
gw = [0] + list(g)                                 # g^W_j
ge = list(g) + [GR]                                # g^E_j
v = [Vj * (1 / dt + lam) for Vj in V]

A = sp.zeros(N, N)
for j in range(N):
    A[j, j] = v[j] + gw[j] + ge[j]
    if j > 0:
        A[j, j - 1] = -gw[j]
    if j + 1 < N:
        A[j, j + 1] = -ge[j]

# X1 pivots by elimination: d_1 = a_11, d_j = a_jj - a_{j,j-1} a_{j-1,j} / d_{j-1}
d = [A[0, 0]]
for j in range(1, N):
    d.append(A[j, j] - A[j, j - 1] * A[j - 1, j] / d[j - 1])
e = [v[0]]
for j in range(1, N):
    e.append(v[j] + gw[j] * e[j - 1] / (e[j - 1] + gw[j]))
# induction step (exact for any N): if d_{j-1} = e_{j-1} + g^W_j then d_j = e_j + g^E_j
ep, gW, gE, vj = sp.symbols("e_prev gW gE v_j", positive=True)
d_j = (vj + gW + gE) - gW**2 / (ep + gW)
e_j = vj + gW * ep / (ep + gW)
x1_step = sp.cancel(d_j - (e_j + gE)) == 0
x1 = all(sp.cancel(sp.together(d[j] - (e[j] + ge[j]))) == 0 for j in range(N))
report("X1 elimination pivots d_j = e_j + g^E_j with the subtraction-free e_j (P21.61)", x1 and x1_step,
       "induction step and explicit N = 5")
# e_j >= v_j > 0: e_j - v_j = g^W e_{j-1}/(e_{j-1}+g^W) is a ratio of positive quantities
report("X1b e_j - v_j = g^W_j e_{j-1}/(e_{j-1}+g^W_j) > 0 (no subtraction anywhere)",
       sp.cancel(e_j - vj - gW * ep / (ep + gW)) == 0)

# X2 row excess
exc = [sp.expand(A[j, j] - sum(-A[j, k] for k in range(N) if k != j)) for j in range(N)]   # off-diagonals are <= 0
report("X2 row excess = v_j (j < N), v_N + G_R (j = N) (P21.60)",
       all(sp.simplify(exc[j] - v[j]) == 0 for j in range(N - 1)) and sp.simplify(exc[-1] - v[-1] - GR) == 0,
       f"v_j = V_j (1/dt + lam)")

# X3 equivalence with a v1.0 step at dt' = dt/(1+z); use N = 3 to keep the inverse small
n3 = 3
V3 = sp.symbols("W1:4", positive=True)
g3 = sp.symbols("h1:3", positive=True)
K3 = sp.zeros(n3, n3)
gw3 = [0] + list(g3)
ge3 = list(g3) + [GR]
for j in range(n3):
    K3[j, j] = gw3[j] + ge3[j]
    if j > 0:
        K3[j, j - 1] = -gw3[j]
    if j + 1 < n3:
        K3[j, j + 1] = -ge3[j]
Vd = sp.diag(*V3)
z = lam * dt
dtp = dt / (1 + z)
# matrices are identical: V/dt + lam V + K = V/dt' + K, and V/dt = (V/dt')/(1+z)
report("X3a V/dt + lam V + K = V/dt' + K, dt' = dt/(1+z)",
       (Vd / dt + lam * Vd + K3 - (Vd / dtp + K3)).applyfunc(sp.cancel) == sp.zeros(n3, n3))
report("X3b V/dt = (1+z)^-1 V/dt', hence A_lam^-1 V/dt = (1+z)^-1 (V/dt'+K)^-1 V/dt' (P21.63)",
       (Vd / dt - Vd / dtp / (1 + z)).applyfunc(sp.cancel) == sp.zeros(n3, n3))

# X4 integrating-factor scheme fixed point: (V/dt + K) e^z C = V/dt C + V S (e^z - 1)/(lam dt)
S3 = sp.Matrix(sp.symbols("s1:4", positive=True))
C3 = sp.Matrix(sp.symbols("C1:4", positive=True))
zz = sp.symbols("z", positive=True)
fac = zz * sp.exp(zz) / (sp.exp(zz) - 1)
eq_scheme = (Vd / dt + K3) * sp.exp(zz) * C3 - Vd / dt * C3 - Vd * S3 * (sp.exp(zz) - 1) / zz   # lam dt = z
eq_claim = fac * K3 * C3 + (zz / dt) * Vd * C3 - Vd * S3                                        # lam = z/dt
report("X4 IF-scheme fixed point (z e^z/(e^z-1) K + lam V) C = V S (P21.68)",
       (eq_scheme * zz / (sp.exp(zz) - 1) - eq_claim).applyfunc(sp.simplify) == sp.zeros(n3, 1))
report("X4b factor z e^z/(e^z-1) = 1 + z/2 + z^2/12 + O(z^4)", sp.simplify(sp.series(fac, zz, 0, 4).removeO() - (1 + zz / 2 + zz**2 / 12)) == 0)

# X5 BE decay error growth: log of ratio (1+z)^-n / e^{-n z} = n (z^2/2 - z^3/3 + ...)
n_, zs = sp.symbols("n z", positive=True)
logratio = -n_ * sp.log(1 + zs) + n_ * zs
ser = sp.series(logratio, zs, 0, 4).removeO()
report("X5 ln[(1+z)^-n e^{nz}] = n z^2/2 - n z^3/3 + O(z^4); with n z = lam t: ~ lam^2 t dt/2 (P21.77)",
       sp.simplify(ser - (n_ * zs**2 / 2 - n_ * zs**3 / 3)) == 0)

# X6 M_out updates
Mo, J, h = sp.symbols("M_o J h", positive=True)
be_fix = sp.solve(sp.Eq(Mo, (Mo + h * J) / (1 + lam * h)), Mo)[0]
ex_fix = sp.solve(sp.Eq(Mo, sp.exp(-lam * h) * Mo + J * (1 - sp.exp(-lam * h)) / lam), Mo)[0]
report("X6a fixed points of both M_out updates are J/lam (P21.76)", sp.simplify(be_fix - J / lam) == 0 and sp.simplify(ex_fix - J / lam) == 0)
tt = sp.symbols("t", positive=True)
m_exact = sp.exp(-lam * tt) * Mo + J * (1 - sp.exp(-lam * tt)) / lam   # solves dM/dt = J - lam M exactly
report("X6b exponential update is the exact solution of dM_out/dt = J - lam M_out, J step-constant (P21.74)",
       sp.simplify(sp.diff(m_exact, tt) - (J - lam * m_exact)) == 0 and sp.simplify(m_exact.subs(tt, 0) - Mo) == 0)
loc = sp.series((Mo + h * J) / (1 + lam * h) - m_exact.subs(tt, h), h, 0, 3).removeO()
report("X6c BE update local error = h^2 lam (lam M_o - J)/2 + O(h^3)",
       sp.simplify(loc - h**2 * lam * (lam * Mo - J) / 2) == 0, str(sp.factor(loc)))

# X7 rectangle source error
t0 = sp.symbols("t_0")
Sfun = sp.Function("S")
s = sp.symbols("s")
a0, a1, a2 = sp.symbols("a0 a1 a2")
Spoly = a0 + a1 * (s - t0) + a2 * (s - t0)**2               # Taylor model of a smooth source
err = sp.integrate(Spoly, (s, t0, t0 + h)) - h * Spoly.subs(s, t0 + h)
report("X7 int_step S - dt S(t^{n+1}) = -dt^2 S'/2 - (2/3) dt^3 S''/2 (Taylor, P21.79)",
       sp.simplify(sp.expand(err - (-h**2 * a1 / 2 - sp.Rational(2, 3) * h**3 * a2))) == 0)

# X8 cell average of a uniform kernel source over a straddling cell [a, b], a < R1 < b
a, b, R1, SK, r = sp.symbols("a b R_1 S_K r", positive=True)
num = sp.integrate(4 * sp.pi * r**2 * SK, (r, a, R1))
Vc = sp.Rational(4, 3) * sp.pi * (b**3 - a**3)
report("X8 straddling cell: Sbar = S_K (R_1^3 - a^3)/(b^3 - a^3) (P21.54)", sp.simplify(num / Vc - SK * (R1**3 - a**3) / (b**3 - a**3)) == 0)

print("ALL PASS" if ok_all else "SOME CHECKS FAILED")
