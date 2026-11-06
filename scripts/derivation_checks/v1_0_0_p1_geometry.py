"""STRATO v1.0.0 Full Mathematical Derivation, Part I checks (geometry, Fick's law units, inventory).

Run:  C:/Y3S1/Nuclear/STRATO/.venv/Scripts/python.exe -I -X utf8 v1_0_0_p1_geometry.py
Prints PASS/FAIL lines. Reads inputs/synthetic/synthetic_five_layer.toml (read-only). Writes nothing.
"""
import math
import pathlib
import re
import sympy as sp

REPO = pathlib.Path("C:/Y3S1/Nuclear/STRATO")
fails = 0


def check(name, ok, detail=""):
    global fails
    print(("PASS" if ok else "FAIL"), name, detail)
    if not ok:
        fails += 1


r, a, b, dr, R, th, ph = sp.symbols("r a b Delta_r R theta varphi", positive=True)

# P1-1 Eq. (1.5): sphere area from the surface integral and from d/dr of the ball volume.
area_int = sp.integrate(sp.integrate(r**2 * sp.sin(th), (th, 0, sp.pi)), (ph, 0, 2 * sp.pi))
area_dv = sp.diff(sp.Rational(4, 3) * sp.pi * r**3, r)
check("P1-1 area 4 pi r^2 (surface integral and dV/dr)",
      sp.simplify(area_int - 4 * sp.pi * r**2) == 0 and sp.simplify(area_dv - 4 * sp.pi * r**2) == 0)

# P1-2 Eq. (1.6): shell volume as integral of area = difference of two ball volumes.
V = sp.integrate(4 * sp.pi * r**2, (r, a, b))
check("P1-2 V(a,b) = 4 pi/3 (b^3 - a^3)", sp.simplify(V - sp.Rational(4, 3) * sp.pi * (b**3 - a**3)) == 0)

# P1-3 thin-shell expansion and limit V(r, r+dr)/dr -> 4 pi r^2.
Vthin = sp.expand(sp.Rational(4, 3) * sp.pi * ((r + dr) ** 3 - r**3))
expected = 4 * sp.pi * r**2 * dr + 4 * sp.pi * r * dr**2 + sp.Rational(4, 3) * sp.pi * dr**3
check("P1-3 V(r,r+dr) = 4 pi r^2 dr + 4 pi r dr^2 + 4 pi/3 dr^3", sp.simplify(Vthin - expected) == 0)
check("P1-3b lim V(r,r+dr)/dr = 4 pi r^2", sp.limit(Vthin / dr, dr, 0) == 4 * sp.pi * r**2)

# P1-4 layer volumes telescope to the ball volume (Eq. (1.1) ordering, Eq. (1.6)).
Rs = sp.symbols("R0:6", positive=True)
tot = sum(sp.Rational(4, 3) * sp.pi * (Rs[l] ** 3 - Rs[l - 1] ** 3) for l in range(1, 6))
check("P1-4 sum of five shell volumes = 4 pi/3 (R5^3 - R0^3)",
      sp.simplify(tot - sp.Rational(4, 3) * sp.pi * (Rs[5] ** 3 - Rs[0] ** 3)) == 0)

# P1-5 AGR-1 radii from Morris et al. (2016) Table 2 (kernel diameter, layer thicknesses, um).
diam, thick = 350.0, [103.5, 39.4, 35.3, 41.0]
radii = [diam / 2]
for t in thick:
    radii.append(radii[-1] + t)
check("P1-5 AGR-1 baseline radii (um)", all(abs(x - y) < 1e-9 for x, y in zip(radii, [175.0, 278.5, 317.9, 353.2, 394.2])),
      "R = " + ", ".join(f"{x:.1f}" for x in radii) + f"; diameter {2 * radii[-1]:.1f} um")
nom = [175.0]
for t in [100.0, 40.0, 35.0, 40.0]:
    nom.append(nom[-1] + t)
check("P1-5b Collin et al. (2016) nominal radii (um)", nom == [175.0, 275.0, 315.0, 350.0, 390.0],
      "R = " + ", ".join(f"{x:.0f}" for x in nom))
fk_agr = (radii[0] / radii[-1]) ** 3
check("P1-5c AGR-1 kernel volume fraction", abs(fk_agr - 0.08750) < 5e-5, f"{fk_agr:.5f}")
ar_agr = (radii[-1] / radii[0]) ** 2
check("P1-5d AGR-1 outer/kernel surface area ratio", abs(ar_agr - 5.074) < 5e-4, f"{ar_agr:.4f}")

# P1-6 synthetic input: ordering (1.1), thicknesses (1.2), kernel fraction.
# Minimal reader (the venv Python has no tomllib): numeric keys only, file order kept.
raw = (REPO / "inputs/synthetic/synthetic_five_layer.toml").read_text()
Rsyn = [float(x) for x in re.findall(r"^outer_radius_m\s*=\s*([0-9.eE+-]+)", raw, re.M)]
c0_file = float(re.search(r"^kernel_concentration_mol_m3\s*=\s*([0-9.eE+-]+)", raw, re.M).group(1))
check("P1-6a five outer radii read from the synthetic input", len(Rsyn) == 5)
Lsyn = [Rsyn[0]] + [Rsyn[i] - Rsyn[i - 1] for i in range(1, 5)]
check("P1-6 synthetic radii strictly increasing from R0 = 0", all(x > y for x, y in zip(Rsyn, [0.0] + Rsyn[:-1])),
      "R (um) = " + ", ".join(f"{x * 1e6:.0f}" for x in Rsyn))
check("P1-6b synthetic thicknesses L (um)", all(abs(x * 1e6 - y) < 1e-6 for x, y in zip(Lsyn, [200, 100, 50, 50, 50])),
      "L (um) = " + ", ".join(f"{x * 1e6:.0f}" for x in Lsyn))
fk_syn = (Rsyn[0] / Rsyn[-1]) ** 3
check("P1-6c synthetic kernel volume fraction", abs(fk_syn - 0.08779) < 5e-5, f"{fk_syn:.5f}")

# P1-7 Eq. (1.4) units: D [m^2 s^-1] * dc/dr [mol m^-4] = [mol m^-2 s^-1] (exponents of m, s, mol).
def mul(*us):
    out = {}
    for u in us:
        for k, v in u.items():
            out[k] = out.get(k, 0) + v
    return {k: v for k, v in out.items() if v != 0}
D_u, dcdr_u, J_u = {"m": 2, "s": -1}, {"mol": 1, "m": -4}, {"mol": 1, "m": -2, "s": -1}
check("P1-7 units of -D dc/dr equal flux units", mul(D_u, dcdr_u) == J_u)

# P1-8 sign of Fick's law: concentration falling outward gives an outward (positive) flux.
c0, D = sp.symbols("c0 D", positive=True)
J = -D * sp.diff(c0 * (1 - r / R), r)
check("P1-8 c = c0 (1 - r/R) gives J_r = D c0/R > 0", sp.simplify(J - D * c0 / R) == 0)

# P1-9 Eq. (1.3) -> (1.7): sum of thin-shell amounts converges to the integral, error ~ dr^2.
Rn, cc = 4.5e-4, 1.0
f = lambda x: cc * math.exp(-x / Rn)
exact = float(sp.integrate(4 * sp.pi * r**2 * sp.exp(-r / R), (r, 0, R)).subs(R, Rn))
errs = []
for n in (50, 100, 200, 400):
    h = Rn / n
    s = sum(f((k + 0.5) * h) * 4 * math.pi * ((k + 0.5) * h) ** 2 * h for k in range(n))
    errs.append(abs(s - exact))
orders = [math.log2(errs[i] / errs[i + 1]) for i in range(3)]
check("P1-9 thin-shell sum converges to the inventory integral (order 2)", all(abs(p - 2) < 0.02 for p in orders),
      "orders " + ", ".join(f"{p:.4f}" for p in orders))

# P1-10 Eq. (1.7) for the kernel-only synthetic start: M = 4 pi/3 R1^3 c0 vs code value M^0 (inventory B11).
c0v = c0_file
M_int = float(sp.integrate(c0v * 4 * sp.pi * r**2, (r, 0, Rsyn[0])))
# Code value: t = 0 row of the regression output tests/golden/v1.0/timeseries.csv (column inventory_mol), read-only.
gold = (REPO / "tests/golden/v1.0/timeseries.csv").read_text().splitlines()
M_code = float(gold[1].split(",")[1]) if gold[1].startswith("0.0,") else float("nan")
check("P1-10 kernel-only inventory 4 pi/3 R1^3 c0 (mol) equals code M^0", abs(M_int - M_code) / M_code < 1e-14,
      f"{M_int:.15e} vs {M_code:.15e}")

print("ALL PASS" if fails == 0 else f"{fails} FAIL")
