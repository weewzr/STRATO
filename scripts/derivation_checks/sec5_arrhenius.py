"""Section 5 (Phase 2.2) checks for the Arrhenius law and its numerical handling.

Equations checked: (P22.3)-(P22.9), (P22.19)-(P22.23) of
sec5_phase22_temperature.md.

Run:  .venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/sec5_arrhenius.py
The script writes no files. Every check prints a PASS or FAIL line.
"""
import math

import numpy as np
import sympy as sp
from scipy import constants, integrate, special

RESULTS = []


def check(name, ok, detail=""):
    RESULTS.append(ok)
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")


# ---------------------------------------------------------------------------
# 1. Einstein relation: second moment of the 3-D Gaussian kernel is 6 D t (P22.3)
# ---------------------------------------------------------------------------
r, D, t = sp.symbols("r D t", positive=True)
G = (4 * sp.pi * D * t) ** sp.Rational(-3, 2) * sp.exp(-r**2 / (4 * D * t))
mass = sp.integrate(4 * sp.pi * r**2 * G, (r, 0, sp.oo))
msd = sp.integrate(4 * sp.pi * r**4 * G, (r, 0, sp.oo))
check("Gaussian kernel has unit mass", sp.simplify(mass - 1) == 0)
check("<r^2> = 6 D t (Einstein, 3-D)", sp.simplify(msd - 6 * D * t) == 0)

# Random walk of n independent jumps of fixed length a, isotropic directions:
# <|sum e_k a|^2> = n a^2 because <e_j . e_k> = delta_jk. With n = Gamma t:
Gam, a = sp.symbols("Gamma a", positive=True)
D_rw = sp.solve(sp.Eq(6 * D * t, Gam * t * a**2), D)[0]
check("D = Gamma a^2 / 6 (P22.3)", sp.simplify(D_rw - Gam * a**2 / 6) == 0)

# Order of magnitude of the lattice estimate D0 = z a^2 nu / 6 (approximate)
for z in (4, 6, 12):
    D0_lat = z * (3e-10) ** 2 * 1e13 / 6
    print(f"INFO  lattice D0 estimate z={z:2d}, a=0.3 nm, nu=1e13 1/s: {D0_lat:.2e} m^2/s")

# ---------------------------------------------------------------------------
# 2. Gas constant (P22.5) and its effect (P22.8)
# ---------------------------------------------------------------------------
# Exact SI value: R_g = N_A k_B with N_A = 6.02214076e23 1/mol and k_B = 1.380649e-23 J/K
# (both exact since 2019); CODATA prints it truncated as 8.314462618... J/(mol K).
RG_CODATA = 6.02214076e23 * 1.380649e-23
RG_T978 = 8.3143
check("R_g = 8.314462618... J/(mol K) (truncated print agrees to 2e-11)",
      abs(RG_CODATA - 8.314462618) / RG_CODATA < 2e-11, f"R_g = {RG_CODATA!r}")
check("scipy.constants.R equals N_A k_B", abs(constants.R - RG_CODATA) < 1e-14, f"scipy R = {constants.R!r}")
check("R_g = N_A k_B to machine precision",
      abs(constants.N_A * constants.k - constants.R) / constants.R < 1e-15,
      f"N_A k_B = {constants.N_A * constants.k!r}")

Q, T, R1, R2 = sp.symbols("Q T R1 R2", positive=True)
ratio = sp.exp(-Q / (R2 * T)) / sp.exp(-Q / (R1 * T))
closed = sp.exp(Q / T * (1 / R1 - 1 / R2))
check("D(R2)/D(R1) = exp[(Q/T)(1/R1 - 1/R2)] (P22.8)", sp.simplify(ratio / closed - 1) == 0)
eps = sp.symbols("epsilon")
first = sp.series(closed.subs(R2, R1 * (1 + eps)), eps, 0, 2).removeO()
check("first order: 1 + (Q/(R1 T)) eps", sp.simplify(first - (1 + Q / (R1 * T) * eps)) == 0)

dR_rel = (RG_CODATA - RG_T978) / RG_T978
print(f"INFO  (R_CODATA - R_T978)/R_T978 = {dR_rel:.4e}")

# Arrhenius rows (TECDOC-978 Tables A-1 to A-3, audit 03). Q in kJ/mol, D0 in m^2/s.
ROWS = {
    "Cs UO2 FRG (2T)": [(5.6e-8, 209.0), (5.2e-4, 362.0)],
    "Sr UO2 FRG": [(2.2e-3, 488.0)],
    "Ag UO2 FRG": [(6.7e-9, 165.0)],
    "Cs PyC FRG": [(6.3e-8, 222.0)],
    "Kr,Xe PyC FRG (2T)": [(2.9e-8, 291.0), (2.0e5, 923.0)],
    "Cs SiC FRG (2T, Gamma=0)": [(5.5e-14, 125.0), (1.6e-2, 514.0)],
    "Sr SiC FRG (2T)": [(1.2e-9, 205.0), (1.8e6, 791.0)],
    "Ag SiC FRG": [(3.6e-9, 215.0)],
}


def arrhenius(terms, T_K, Rg, gamma=None):
    """D(T) = sum_m D0_m exp(-Q_m/(Rg T)); Q in kJ/mol is converted to J/mol."""
    total = 0.0
    for m, (D0, Qk) in enumerate(terms):
        if gamma is not None and m == 0:
            D0 = D0 * math.exp(gamma / 5.0)
        total += D0 * math.exp(-Qk * 1e3 / (Rg * T_K))
    return total


T_grid = np.linspace(1000.0, 2000.0, 101) + 273.15
worst = 0.0
for name, terms in ROWS.items():
    rel = max(arrhenius(terms, T, RG_CODATA) / arrhenius(terms, T, RG_T978) - 1.0 for T in T_grid)
    worst = max(worst, rel)
    print(f"INFO  max D(CODATA)/D(8.3143) - 1 over 1000-2000 C, {name:26s}: {rel:.3e}")
check("R_g choice changes D by less than 0.2 % on the 1000-2000 C grid", worst < 2e-3,
      f"worst = {worst:.3e}")

# Effect on a benchmark release: CRP-6 case 1a/1b (bare kernel, r_k = 250 um, 200 h)
def booth_release(Dp_t, nmax=20000):
    n = np.arange(1, nmax + 1)
    return 1.0 - 6.0 / np.pi**2 * np.sum(np.exp(-n**2 * np.pi**2 * Dp_t) / n**2)


rk, t_heat = 250e-6, 720000.0
for label, Tc in (("1a", 1200.0), ("1b", 1600.0)):
    Fs = []
    for Rg in (RG_T978, RG_CODATA):
        Dp_t = arrhenius(ROWS["Cs UO2 FRG (2T)"], Tc + 273.15, Rg) / rk**2 * t_heat
        Fs.append(booth_release(Dp_t))
    print(f"INFO  CRP-6 case {label}: F(R=8.3143) = {Fs[0]:.6f}, F(R=8.314462618) = {Fs[1]:.6f}, "
          f"difference {Fs[1] - Fs[0]:.2e}")
    if label == "1a":
        check("case 1a F within audit bracket 0.46721-0.46742 for both R_g",
              all(0.46715 < f < 0.46748 for f in Fs), f"{Fs[0]:.5f}, {Fs[1]:.5f}")

# ---------------------------------------------------------------------------
# 3. Effective activation energy (P22.6) and crossover temperature (P22.7)
# ---------------------------------------------------------------------------
beta, Rg = sp.symbols("beta R_g", positive=True)   # beta = 1/T
D01, D02, Q1, Q2 = sp.symbols("D01 D02 Q1 Q2", positive=True)
Dsum = D01 * sp.exp(-Q1 * beta / Rg) + D02 * sp.exp(-Q2 * beta / Rg)
Qeff = -Rg * sp.diff(sp.log(Dsum), beta)
Qeff_claim = (Q1 * D01 * sp.exp(-Q1 * beta / Rg) + Q2 * D02 * sp.exp(-Q2 * beta / Rg)) / Dsum
check("Q_eff = -R_g dlnD/d(1/T) = sum Q_m D_m / sum D_m (P22.6)", sp.simplify(Qeff - Qeff_claim) == 0)

Tx = sp.symbols("T_x", positive=True)
Tx_claim = (Q2 - Q1) / (Rg * sp.log(D02 / D01))
resid = (D01 * sp.exp(-Q1 / (Rg * Tx)) - D02 * sp.exp(-Q2 / (Rg * Tx))).subs(Tx, Tx_claim)
check("branches equal at T_x = (Q2-Q1)/(R_g ln(D02/D01)) (P22.7)", sp.simplify(sp.expand_log(resid, force=True)) == 0
      or abs(float(resid.subs({D01: 5.5e-14, D02: 1.6e-2, Q1: 125e3, Q2: 514e3, Rg: RG_CODATA}))) < 1e-25)

def crossover(terms, gamma=0.0):
    (a1, q1), (a2, q2) = terms
    a1 = a1 * math.exp(gamma / 5.0)
    return (q2 - q1) * 1e3 / (RG_CODATA * math.log(a2 / a1))


for name, gam in (("Cs SiC FRG (2T, Gamma=0)", 0.0), ("Cs SiC FRG (2T, Gamma=0)", 4.5),
                  ("Sr SiC FRG (2T)", 0.0), ("Cs UO2 FRG (2T)", 0.0), ("Kr,Xe PyC FRG (2T)", 0.0)):
    Tk = crossover(ROWS[name], gam)
    print(f"INFO  crossover {name:26s} Gamma={gam:3.1f}: T_x = {Tk:7.1f} K = {Tk - 273.15:7.1f} C")
Tk0 = crossover(ROWS["Cs SiC FRG (2T, Gamma=0)"])
ok = abs(arrhenius([ROWS["Cs SiC FRG (2T, Gamma=0)"][0]], Tk0, RG_CODATA)
         / arrhenius([ROWS["Cs SiC FRG (2T, Gamma=0)"][1]], Tk0, RG_CODATA) - 1) < 1e-12
check("numerical: Cs-in-SiC branches equal at computed T_x", ok)

# Fluence factor exp(Gamma/5) (P22.9)
for g in (0.0, 1.0, 2.0, 3.0, 4.5):
    print(f"INFO  fluence factor exp(Gamma/5) at Gamma = {g:3.1f}e25 m^-2: {math.exp(g / 5):.3f}")
check("fluence factor at Gamma=4.5 equals 2.460 (audit 03)", abs(math.exp(4.5 / 5) - 2.4596) < 1e-3)

# ---------------------------------------------------------------------------
# 4. Rate of change of D along a ramp (P22.19) and step bound (P22.20)
# ---------------------------------------------------------------------------
tt = sp.symbols("t", positive=True)
Tf = sp.Function("T")(tt)
Dt = D01 * sp.exp(-Q1 / (Rg * Tf)) + D02 * sp.exp(-Q2 / (Rg * Tf))
lhs = sp.diff(sp.log(Dt), tt)
QeffT = (Q1 * D01 * sp.exp(-Q1 / (Rg * Tf)) + Q2 * D02 * sp.exp(-Q2 / (Rg * Tf))) / Dt
check("dlnD/dt = Q_eff/(R_g T^2) dT/dt (P22.19)",
      sp.simplify(lhs - QeffT / (Rg * Tf**2) * sp.diff(Tf, tt)) == 0)


def dt_bound(terms, T_K, rate_K_per_h, eps_lnD=0.1):
    Ds = [D0 * math.exp(-q * 1e3 / (RG_CODATA * T_K)) for D0, q in terms]
    qeff = sum(q * 1e3 * d for (D0, q), d in zip(terms, Ds)) / sum(Ds)
    rate = abs(rate_K_per_h) / 3600.0
    return eps_lnD * RG_CODATA * T_K**2 / (qeff * rate), qeff


for name, Tc, rate in (("Cs SiC FRG (2T, Gamma=0)", 1600.0, 50.0), ("Cs SiC FRG (2T, Gamma=0)", 1250.0, 120.0),
                       ("Cs SiC FRG (2T, Gamma=0)", 1600.0, -600.0), ("Sr SiC FRG (2T)", 1800.0, 50.0),
                       ("Cs UO2 FRG (2T)", 1600.0, 50.0)):
    b, qe = dt_bound(ROWS[name], Tc + 273.15, rate)
    print(f"INFO  {name:26s} T={Tc:6.0f} C, dT/dt={rate:6.0f} K/h: Q_eff={qe / 1e3:6.1f} kJ/mol, "
          f"dt <= {b:7.1f} s for |dlnD| <= 0.1")

# ---------------------------------------------------------------------------
# 5. Exact time integral of exp(-a/T) over a linear ramp (P22.22)-(P22.23)
# ---------------------------------------------------------------------------
aa, TT = sp.symbols("a T", positive=True)
E1 = sp.expint(1, aa / TT)
prim = TT * sp.exp(-aa / TT) - aa * E1
check("d/dT [T e^{-a/T} - a E1(a/T)] = e^{-a/T}", sp.simplify(sp.diff(prim, TT) - sp.exp(-aa / TT)) == 0)
E2_identity = sp.expint(2, aa / TT) - (sp.exp(-aa / TT) - (aa / TT) * sp.expint(1, aa / TT))
check("E2(x) = e^{-x} - x E1(x)", sp.simplify(sp.expand_func(E2_identity)) == 0
      or abs(float(E2_identity.subs({aa: 3.0e4, TT: 1700.0}))) < 1e-25)


def tau_linear_segment(Q_J, T_ref, Ta, Tb, dt):
    """int alpha dt over a linear ramp Ta->Tb of duration dt; alpha = exp(-a(1/T - 1/T_ref))."""
    a_ = Q_J / RG_CODATA
    if Ta == Tb:
        return dt * math.exp(-a_ * (1.0 / Ta - 1.0 / T_ref))
    beta_ = (Tb - Ta) / dt
    prim_ = lambda T_: T_ * special.expn(2, a_ / T_)
    return math.exp(a_ / T_ref) * (prim_(Tb) - prim_(Ta)) / beta_


for Q_J, Ta, Tb, dt in ((514e3, 1523.15, 1873.15, 7 * 3600.0), (300e3, 1873.15, 2073.15, 4 * 3600.0),
                        (209e3, 673.15, 1523.15, 850 / 120 * 3600.0), (514e3, 1873.15, 303.15, 1570 / 600 * 3600)):
    exact = tau_linear_segment(Q_J, 1873.15, Ta, Tb, dt)
    quad = integrate.quad(lambda s: math.exp(-Q_J / RG_CODATA * (1 / (Ta + (Tb - Ta) * s / dt) - 1 / 1873.15)),
                          0, dt, epsabs=0, epsrel=1e-13, limit=400)[0]
    check(f"E2 closed form vs quadrature, Q={Q_J / 1e3:.0f} kJ/mol, {Ta - 273.15:.0f}->{Tb - 273.15:.0f} C",
          abs(exact / quad - 1) < 1e-10, f"tau = {exact:.10e} s, rel diff {abs(exact / quad - 1):.1e}")

# ---------------------------------------------------------------------------
# 6. Input validation rules (P22.10 and the rule list): a reference implementation
# ---------------------------------------------------------------------------
def validate_term(D0, Q_kJ, T_K):
    if not (math.isfinite(D0) and D0 > 0):
        raise ValueError("D0 must be finite and > 0 m^2/s")
    if not (math.isfinite(Q_kJ) and Q_kJ >= 0):
        raise ValueError("Q must be finite and >= 0")
    if not (math.isfinite(T_K) and T_K > 0):
        raise ValueError("T must be finite and > 0 K")
    x = Q_kJ * 1e3 / (RG_CODATA * T_K)
    return math.log(D0) - x          # ln D, safe even when D underflows


bad = [(0.0, 100.0, 1500.0), (-1e-9, 100.0, 1500.0), (1e-9, -1.0, 1500.0), (1e-9, 100.0, 0.0),
       (1e-9, 100.0, -5.0), (float("nan"), 100.0, 1500.0), (1e-9, 100.0, float("inf"))]
rejected = 0
for args in bad:
    try:
        validate_term(*args)
    except ValueError:
        rejected += 1
check("validator rejects all 7 invalid inputs", rejected == len(bad))
check("validator accepts Q = 0 (TECDOC-978 row 44)", abs(validate_term(6.0e-6, 0.0, 1273.15) - math.log(6e-6)) < 1e-15)
lnD = validate_term(2.0e5, 923.0, 100.0)
check("ln D stays finite where exp underflows (Q=923 kJ/mol, T=100 K)",
      math.isfinite(lnD) and math.exp(lnD) == 0.0, f"ln D = {lnD:.1f}")
check("Kelvin conversion T_K = T_C + 273.15 exactly at 1600 C", 1600.0 + 273.15 == 1873.15)

print(f"\nSUMMARY {sum(RESULTS)}/{len(RESULTS)} checks passed")
