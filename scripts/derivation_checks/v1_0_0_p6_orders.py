"""STRATO v1.0.0, Full Mathematical Derivation, Part VI: Richardson extrapolation, observed
orders, the energy (Lax-type) error estimate and the study-cost formula.

Inputs are the printed v1.0.0 values of convergence_study.csv for the synthetic five-layer
case (audit note 01, Section 1.7; reproduced against CI evidence there), the Rust value of the
exact series at t_end (0.43255563909285666, audit 01 Section 1.10 item 2) and the same-mesh
time-converged FV value 0.4325501631858728 (audit 01 Section 1.10 item 5).

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/v1_0_0_p6_orders.py
Writes nothing. One PASS/FAIL line per check; exit code 1 on any FAIL.
"""

import math
import sys

import sympy as sp

ok_all = []


def check(label, ok, detail=""):
    ok = bool(ok)
    ok_all.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {label}" + (f"  [{detail}]" if detail else ""), flush=True)


# ---------------- Richardson (CMP 150-157)
h, F0, a1, a2, a3, a4 = sp.symbols("h F0 a1 a2 a3 a4")
Fh = lambda x: F0 + a1 * x + a2 * x**2 + a3 * x**3 + a4 * x**4
FRE = sp.expand((8 * Fh(h / 4) - 6 * Fh(h / 2) + Fh(h)) / 3)
check("6.10 (8F(h/4) - 6F(h/2) + F(h))/3 = F0 + a3 h^3/8 + 7 a4 h^4/32",
      sp.simplify(FRE - (F0 + a3 * h**3 / 8 + sp.Rational(7, 32) * a4 * h**4)) == 0, str(FRE))
wts = sp.Matrix([[1, 1, 1], [sp.Rational(1, 4), sp.Rational(1, 2), 1], [sp.Rational(1, 16), sp.Rational(1, 4), 1]])
sol = wts.solve(sp.Matrix([1, 0, 0]))
check("6.10 weights (8/3, -2, 1/3) are the unique ones that keep F0 and cancel h and h^2",
      list(sol) == [sp.Rational(8, 3), -2, sp.Rational(1, 3)])

mu, tt = sp.symbols("mu t", positive=True)
modal = sp.exp(-(tt / h) * sp.log(1 + mu * h))
ser = sp.series(modal, h, 0, 2).removeO()
check("6.9 one BE mode after t/h steps: (1 + mu h)^(-t/h) = e^{-mu t}(1 + mu^2 t h/2 + O(h^2)), smooth in h",
      sp.simplify(ser - sp.exp(-mu * tt) * (1 + mu**2 * tt * h / 2)) == 0)
xx = sp.symbols("x", positive=True)
check("6.4 per-step amplification error 1/(1+x) - e^{-x} = x^2/2 + O(x^3), x = mu dt",
      sp.simplify(sp.series(1 / (1 + xx) - sp.exp(-xx), xx, 0, 3).removeO() - xx**2 / 2) == 0)
m_, w_ = sp.symbols("m w", positive=True)
cen = sp.Rational(3, 4) * ((m_ + w_ / 2) ** 4 - (m_ - w_ / 2) ** 4) / ((m_ + w_ / 2) ** 3 - (m_ - w_ / 2) ** 3)
check("6.5 volume centroid of a cell of width w and midpoint m: r_j = m + w^2/(6m) + O(w^4)",
      sp.simplify(sp.series(cen, w_, 0, 4).removeO() - (m_ + w_**2 / (6 * m_))) == 0)
check("6.11 machine epsilon 2^-52 = 2.22e-16 is the literal 2.2e-16 of SL 372 (unit roundoff is half of it)",
      abs(2.0**-52 - 2.220446e-16) < 1e-21)
e_time_200 = 0.4325501631858728 - 0.4324286068785298
e_space_200 = 0.43255563909285666 - 0.4325501631858728
check("6.4 final error 1.270322e-4 = temporal 1.2156e-4 + spatial 5.476e-6 (temporal part is 96 %)",
      abs(e_time_200 + e_space_200 - 1.270322e-4) < 1e-9 and e_time_200 / 1.270322e-4 > 0.95,
      f"{e_time_200:.5e} + {e_space_200:.4e}")

# ---------------- orders (CMP 112-118)
Fex = 0.43255563909285666
sp_F = [0.4325337375009147, 0.4325501631858728, 0.4325542700813807, 0.4325552968363947, 0.4325555535278729]
tm_F = [0.4320644195613644, 0.4323071308829620, 0.4324286068785298, 0.4324893749894793, 0.4325197665769955]
dts = [800.0, 400.0, 200.0, 100.0, 50.0]
order = lambda e: [math.log2(e[k] / e[k + 1]) for k in range(len(e) - 1)]
e_s = [abs(f - Fex) for f in sp_F]
e_t = [abs(f - Fex) for f in tm_F]
o_s, o_t = order(e_s), order(e_t)
check("6.11 spatial orders from convergence_study.csv round to 2.00, 2.00, 2.00, 2.00",
      [round(o, 2) for o in o_s] == [2.0] * 4, ", ".join(f"{o:.4f}" for o in o_s))
check("6.11 printed temporal orders 0.98, 0.97, 0.94, 0.89 are reproduced",
      [round(o, 2) for o in o_t] == [0.98, 0.97, 0.94, 0.89], ", ".join(f"{o:.4f}" for o in o_t))
Fref = 0.4325501631858728
e_ref = [abs(f - Fref) for f in tm_F]
o_ref = order(e_ref)
check("6.11 against the same-mesh reference the errors are 4.857e-4 ... 3.040e-5 and the orders 0.999 to 1.000",
      all(abs(o - 1) < 1.5e-3 for o in o_ref), ", ".join(f"{e:.4g}" for e in e_ref) + " / " + ", ".join(f"{o:.4f}" for o in o_ref))
Es = Fex - Fref
check("6.11 spatial error of the 200-cell mesh E_s = F_ex - F_ref = 5.476e-6", abs(Es - 5.476e-6) < 1e-9, f"{Es:.4e}")
# model: F(dt) - F_ex = -(K dt + Es), K from the finest same-mesh error
K = e_ref[-1] / dts[-1]
model = [K * d + Es for d in dts]
o_model = order(model)
check("6.11 model e_k = K dt_k + E_s with K = 6.08e-7 s^-1 reproduces the printed orders to 0.01",
      all(abs(a - b) < 0.01 for a, b in zip(o_model, o_t)), f"K = {K:.4e}; model orders " + ", ".join(f"{o:.3f}" for o in o_model))
x = sp.symbols("x", positive=True)  # x = E_s/(K dt)
pk = sp.log((2 + x) / (1 + x), 2)
check("6.11 apparent order log2((2K dt + E_s)/(K dt + E_s)) < 1 for E_s > 0 and -> 0 as dt -> 0",
      sp.limit(pk, x, 0) == 1 and sp.limit(pk, x, sp.oo) == 0 and sp.simplify(sp.diff(pk, x)) != 0)
check("6.12 resolution test: every printed error exceeds 3 eps_F = 1.199e-11",
      min(e_s + e_t) > 3 * 3.997e-12, f"smallest error {min(e_s + e_t):.3e}")

# ---------------- energy estimate (Lax-type argument)
N = 5
eps = sp.symbols(f"e1:{N+1}")
tau = sp.symbols(f"t1:{N}")       # tau_{j+1/2}, j = 1..N-1
tauR = sp.Symbol("tR")
tfull = [0] + list(tau) + [tauR]   # tau_{1/2} = 0 (zero area), ..., tau_{N+1/2} = tau_R
lhs = sum(eps[j] * (tfull[j + 1] - tfull[j]) for j in range(N))
rhs = sum(tau[j] * (eps[j] - eps[j + 1]) for j in range(N - 1)) + tauR * eps[-1]
check("6.15 summation by parts: sum_j e_j (tau_{j+1/2} - tau_{j-1/2}) = sum_f tau_f (e_j - e_{j+1}) + tau_R e_N",
      sp.expand(lhs - rhs) == 0)
Gs, ds, ts = sp.symbols("G delta tau", positive=True)
check("6.15 Young: |tau delta| <= G delta^2/2 + tau^2/(2G) since (sqrt(G) delta - tau/sqrt(G))^2 >= 0",
      sp.expand(Gs * ds**2 / 2 + ts**2 / (2 * Gs) - ts * ds - (sp.sqrt(Gs) * ds - ts / sp.sqrt(Gs)) ** 2 / 2) == 0)
w, rr, Dd, k1, k2, L = sp.symbols("w r D k1 k2 L", positive=True)
interior = (k1 * w**2 * rr) ** 2 / (4 * sp.pi * rr**2 * Dd / w) * (L / w)
interface = (k2 * w * rr**2) ** 2 / (4 * sp.pi * rr**2 * Dd / w)
check("6.15 interior faces contribute O(w^4) (about L/w faces, tau = O(w^2), G = O(1/w))",
      sp.degree(sp.simplify(interior), w) == 4)
check("6.15 each interface or outer face contributes O(w^3), so ||e||_V = O(w^(3/2)) at least",
      sp.degree(sp.simplify(interface), w) == 3)

# ---------------- cost (CMP 159-176)
cells, steps = 200, 1000
W = lambda f, m: (steps / m) * cells * f
Wsp_impl = sum(1.75 * W(f, 1) for f in [0.5, 1, 2, 4, 8])
Wtm = sum(W(1, m) for m in [4, 2, 1, 0.5, 0.25])
Wsp_true = sum((4 + 2 + 1) * W(f, 1) for f in [0.5, 1, 2, 4, 8])
check("6.14 implemented study cost 1.75 * 3.1e6 + 1.55e6 = 6.975e6 cell-steps",
      abs(Wsp_impl - 5.425e6) < 1 and abs(Wtm - 1.55e6) < 1, f"{Wsp_impl:.4e} + {Wtm:.4e}")
check("6.14 true spatial cost of three Richardson runs is 7 * 3.1e6 = 2.17e7 (factor 4 above the estimate)",
      abs(Wsp_true - 2.17e7) < 1 and abs(Wsp_true / Wsp_impl - 4) < 1e-12)

print("ALL PASS" if all(ok_all) else "SOME FAIL")
sys.exit(0 if all(ok_all) else 1)
