"""Phase 2.1 (section 4): the exact decay identity and the inventory partition.

Checks:
  D1. (P21.80) c = exp(-lam t) c0 turns the decay-free PDE into the PDE with decay, for an
      arbitrary function c0(r,t) and layer-wise D_i(t); interface, centre and surface
      conditions are multiplied by exp(-lam t) and therefore preserved.
  D2. Same identity with a source: c = exp(-lam t) ct, where ct solves the decay-free PDE
      with source exp(lam t) S.
  D3. (P21.81) modal inventory partition for decay plus diffusion, no source:
      M, M_rel, M_dec per mode; M + M_rel + M_dec = M_0 exactly; lam -> 0 limit.
  D4. (P21.87) Booth transient with source and decay: the modal release rate equals
      TECDOC-1674 Eq. (7.7) and the quantity M_out/(M + M_out) equals Eq. (7.8), per mode
      (symbolic) and summed (numeric, 1e-12).
  D5. lam -> 0 of Eq. (7.8) equals Eq. (7.1).

Run: .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec4_p21_decay_identity.py
Writes nothing to disk. Prints PASS/FAIL lines.
"""
import sympy as sp
import mpmath as mp

ok_all = True


def report(name, ok, detail=""):
    global ok_all
    ok_all &= bool(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


r, t, s = sp.symbols("r t s", positive=True)
lam, Lam = sp.symbols("lambda Lambda", positive=True)
c0 = sp.Function("c0")(r, t)
Dt = sp.Function("D")(t)          # layer diffusivity, may depend on time (temperature history)
S = sp.Function("S")(r, t)


def L_op(c, Di):
    return sp.diff(r**2 * Di * sp.diff(c, r), r) / r**2


# D1: residual of the decay PDE for c = e^{-lam t} c0 equals e^{-lam t} times the decay-free residual
c = sp.exp(-lam * t) * c0
res_decay = sp.diff(c, t) - L_op(c, Dt) + lam * c
res_free = sp.diff(c0, t) - L_op(c0, Dt)
report("D1 PDE: res_decay[e^{-lam t} c0] = e^{-lam t} res_free[c0] (P21.80)",
       sp.simplify(res_decay - sp.exp(-lam * t) * res_free) == 0)
# conditions: linear homogeneous functionals B[c] (centre derivative, interface jumps, surface value)
c0a, c0b = sp.Function("c0a")(r, t), sp.Function("c0b")(r, t)
Da, Db, Ri, Rs = sp.symbols("D_a D_b R_i R", positive=True)
conds = [
    lambda f: sp.diff(f[0], r).subs(r, 0),                                   # centre symmetry
    lambda f: (f[0] - f[1]).subs(r, Ri),                                     # continuity
    lambda f: (Da * sp.diff(f[0], r) - Db * sp.diff(f[1], r)).subs(r, Ri),   # flux continuity
    lambda f: f[1].subs(r, Rs),                                              # absorbing surface
]
e = sp.exp(-lam * t)
okc = all(sp.simplify(B([e * c0a, e * c0b]) - e * B([c0a, c0b])) == 0 for B in conds)
report("D1b conditions scale by e^{-lam t} (homogeneous, linear)", okc)

# D2: with a source
ct = sp.Function("ct")(r, t)
c = sp.exp(-lam * t) * ct
res_decay_src = sp.diff(c, t) - L_op(c, Dt) + lam * c - S
res_free_src = sp.diff(ct, t) - L_op(ct, Dt) - sp.exp(lam * t) * S
report("D2 with source: c = e^{-lam t} ct, ct driven by e^{lam t} S",
       sp.simplify(res_decay_src - sp.exp(-lam * t) * res_free_src) == 0)

# D3: modal partition, one mode with weight w (the sum over modes is linear)
w, M0 = sp.symbols("w M_0", positive=True)
k = Lam + lam
Mn = M0 * w * sp.exp(-k * t)                          # inventory carried by mode n
Jn = M0 * w * Lam * sp.exp(-k * t)                    # its contribution to J_R (Lambda_n times inventory, see text)
Mrel = sp.integrate(Jn.subs(t, s), (s, 0, t))
Mdec = sp.integrate(lam * Mn.subs(t, s), (s, 0, t))
Mrel_cl = M0 * w * Lam / k * (1 - sp.exp(-k * t))
Mdec_cl = M0 * w * lam / k * (1 - sp.exp(-k * t))
report("D3a M_rel per mode = M0 w Lam/(Lam+lam) (1 - e^{-(Lam+lam) t}) (P21.81)", sp.simplify(Mrel - Mrel_cl) == 0)
report("D3b M_dec per mode = M0 w lam/(Lam+lam) (1 - e^{-(Lam+lam) t})", sp.simplify(Mdec - Mdec_cl) == 0)
report("D3c M + M_rel + M_dec = M0 w per mode (closure)", sp.simplify(Mn + Mrel_cl + Mdec_cl - M0 * w) == 0)
report("D3d lam -> 0: M_rel -> M0 w (1 - e^{-Lam t}) (Phase 1)",
       sp.simplify(sp.limit(Mrel_cl, lam, 0) - M0 * w * (1 - sp.exp(-Lam * t))) == 0)

# D4: Booth with constant source B (atoms born per second in the whole sphere), zero initial inventory.
# Single sphere: Lambda_n = n^2 pi^2 D', weight of the uniform source in mode n is 6/(n^2 pi^2).
# Modal inventory: dm/dt = -(Lam+lam) m + B w  ->  m = B w (1 - e^{-k t})/k ; release rate Lam m.
B = sp.symbols("B", positive=True)
mn = B * w * (1 - sp.exp(-k * t)) / k
Jn = Lam * mn
Dp, n = sp.symbols("Dprime n", positive=True)
a = lam / Dp
subs_n = {Lam: n**2 * sp.pi**2 * Dp, w: 6 / (n**2 * sp.pi**2)}
RB_mode = sp.simplify((Jn / B).subs(subs_n))
RB_77_mode = 6 / (n**2 * sp.pi**2 + a) - 6 * sp.exp(-lam * t) * sp.exp(-n**2 * sp.pi**2 * Dp * t) / (n**2 * sp.pi**2 + a)
report("D4a mode-wise R/B(t) = TECDOC-1674 Eq. (7.7) term (P21.87)", sp.simplify(RB_mode - RB_77_mode) == 0)
# M_out: released atoms surviving outside, dM_out/dt = J - lam M_out, M_out(0) = 0
Mout_n = sp.integrate(sp.exp(-lam * (t - s)) * Jn.subs(t, s), (s, 0, t))
Mtot = B * (1 - sp.exp(-lam * t)) / lam                # all surviving atoms (inside + outside), per unit weight sum
F_mode = sp.simplify((Mout_n / Mtot).subs(subs_n))
F_78_mode = 6 / (n**2 * sp.pi**2 + a) - (6 / sp.pi**2) * a / (sp.exp(lam * t) - 1) * (1 - sp.exp(-n**2 * sp.pi**2 * Dp * t)) / (n**2 * (n**2 * sp.pi**2 + a))
report("D4b mode-wise M_out/(M + M_out) = TECDOC-1674 Eq. (7.8) term", sp.simplify(sp.expand(F_mode - F_78_mode)) == 0)
# sum of weights of all modes is 1, so the total surviving amount is B(1 - e^{-lam t})/lam: check M + M_out per mode
Min_n = mn
Mout_plus = sp.simplify(Min_n + Mout_n - w * Mtot)
report("D4c M_n + M_out,n = w B (1 - e^{-lam t})/lam (inventory identity behind the denominator)", Mout_plus == 0)

# numeric check of the summed series, D' = 1e-5 s^-1, lam = 6.88e-5 s^-1 (Kr-88 value of TECDOC-1674 Eq. 5.4)
mp.mp.dps = 30
Dpv, lv = mp.mpf("1e-5"), mp.mpf("6.88e-5")
av = lv / Dpv
x = mp.sqrt(av)
closed = 3 / x * (mp.coth(x) - 1 / x)


def F78(tt):
    tail = mp.nsum(lambda nn: (1 - mp.exp(-nn**2 * mp.pi**2 * Dpv * tt)) / (nn**2 * (nn**2 * mp.pi**2 + av)), [1, mp.inf])
    return closed - 6 / mp.pi**2 * av / (mp.exp(lv * tt) - 1) * tail


def Fmodal(tt):
    def term(nn):
        L = nn**2 * mp.pi**2 * Dpv
        kk = L + lv
        wn = 6 / (nn**2 * mp.pi**2)
        mout = wn * L * ((1 - mp.exp(-lv * tt)) / (lv * kk) - mp.exp(-lv * tt) * (1 - mp.exp(-L * tt)) / (kk * L))
        return mout
    return mp.nsum(term, [1, mp.inf]) / ((1 - mp.exp(-lv * tt)) / lv)


worst = max(abs(F78(tt) - Fmodal(tt)) for tt in (mp.mpf(1e3), mp.mpf(1e4), mp.mpf(1e5)))
report("D4d summed: Eq. (7.8) = M_out/(M + M_out) from modal sums", worst < 1e-12, f"max |diff| {mp.nstr(worst, 3)}; F(1e4 s) = {mp.nstr(F78(mp.mpf(1e4)), 10)}")


# D5: lam -> 0 of (7.8) equals (7.1)
def F71(Dt_):
    return 1 - 6 / Dt_ * mp.nsum(lambda nn: (1 - mp.exp(-nn**2 * mp.pi**2 * Dt_)) / (nn**4 * mp.pi**4), [1, mp.inf])


def F78_general(Dp_, l_, tt):
    a_ = l_ / Dp_
    x_ = mp.sqrt(a_)
    cl = 3 / x_ * (mp.coth(x_) - 1 / x_)
    tail = mp.nsum(lambda nn: (1 - mp.exp(-nn**2 * mp.pi**2 * Dp_ * tt)) / (nn**2 * (nn**2 * mp.pi**2 + a_)), [1, mp.inf])
    return cl - 6 / mp.pi**2 * a_ / mp.expm1(l_ * tt) * tail


mp.mp.dps = 40
d5 = max(abs(F78_general(mp.mpf(1), mp.mpf("1e-14"), mp.mpf(Dt_)) - F71(mp.mpf(Dt_))) for Dt_ in ("0.01", "0.1", "1"))
report("D5 lam -> 0 of Eq. (7.8) equals Eq. (7.1) at D't = 0.01, 0.1, 1", d5 < 1e-10, f"max |diff| {mp.nstr(d5, 3)}")

print("ALL PASS" if ok_all else "SOME CHECKS FAILED")
