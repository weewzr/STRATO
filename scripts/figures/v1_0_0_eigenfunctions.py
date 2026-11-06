"""Figure 4.1 for the STRATO v1.0.0 derivation document: first three eigenfunctions.

Synthetic five-layer v1.0 input (inputs/synthetic/synthetic_five_layer.toml,
SYNTHETIC NUMERICAL BENCHMARK, NOT LITERATURE TRISO DATA). The eigenfunctions are
computed independently of the Rust code: u = alpha sin(k r) + beta cos(k r) in each
layer, u and D(u' - u/r) matched at each interface, roots of g(Lambda) = u(R) found
by a sign scan in sqrt(Lambda) and Brent's method.

Panel (a): concentration shape phi_n = u_n / r, scaled so that max |phi_n| = 1 and
phi_n(0) > 0 (dimensionless). Panel (b): u_n / sqrt(N_n), N_n = int_0^R u_n^2 dr,
which makes int_0^R (u_n / sqrt(N_n))^2 dr = 1 (unit m^-1/2).

Output (150 dpi): docs/mathematics/v1.0.0/figures/eigenfunctions.png

Run:
    C:/Y3S1/Nuclear/STRATO/.venv/Scripts/python.exe -I -X utf8 \
        scripts/figures/v1_0_0_eigenfunctions.py
"""

import math
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402
from scipy.integrate import quad  # noqa: E402
from scipy.optimize import brentq  # noqa: E402

REPO = Path(__file__).resolve().parents[2]
OUT = REPO / "docs" / "mathematics" / "v1.0.0" / "figures" / "eigenfunctions.png"

RAD = [2.0e-4, 3.0e-4, 3.5e-4, 4.0e-4, 4.5e-4]
DIF = [1.0e-12, 4.0e-10, 8.0e-13, 2.0e-14, 7.0e-13]
NAMES = ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"]


def coeffs(lam):
    out, al, be = [], 1.0, 0.0
    for i, (ro, di) in enumerate(zip(RAD, DIF)):
        k = math.sqrt(lam / di)
        out.append((al, be, k))
        if i + 1 < len(RAD):
            sn, cs = math.sin(k * ro), math.cos(k * ro)
            u0 = al * sn + be * cs
            f0 = di * (k * (al * cs - be * sn) - u0 / ro)
            kn = math.sqrt(lam / DIF[i + 1])
            sn2, cs2 = math.sin(kn * ro), math.cos(kn * ro)
            v0 = f0 / DIF[i + 1] + u0 / ro
            al, be = sn2 * u0 + cs2 * v0 / kn, cs2 * u0 - sn2 * v0 / kn
    return out


def g(lam):
    al, be, k = coeffs(lam)[-1]
    return al * math.sin(k * RAD[-1]) + be * math.cos(k * RAD[-1])


def u_of(lam, r):
    cf = coeffs(lam)
    out = np.zeros_like(r)
    edges = [0.0] + RAD
    for i, (al, be, k) in enumerate(cf):
        m = (r >= edges[i]) & (r <= edges[i + 1])
        out[m] = al * np.sin(k * r[m]) + be * np.cos(k * r[m])
    return out


def first_roots(n):
    x = np.linspace(1e-6, 0.03, 200001)
    gv = np.array([g(v * v) for v in x])
    idx = np.where(np.sign(gv[1:]) * np.sign(gv[:-1]) < 0)[0][:n]
    return [brentq(lambda v: g(v * v), x[i], x[i + 1], xtol=1e-18, rtol=1e-15) ** 2 for i in idx]


def main():
    lams = first_roots(3)
    ref = [3.273255671175805e-6, 7.791893309400181e-5]
    for got, want in zip(lams, ref):
        assert abs(got / want - 1) < 1e-9, (got, want)
    r = np.linspace(1e-9, RAD[-1], 6001)
    rum = r * 1e6
    styles = [("#0072B2", "-"), ("#D55E00", "--"), ("#009E73", "-.")]

    fig, axes = plt.subplots(2, 1, figsize=(7.0, 6.4), sharex=True)
    for n, (lam, (col, ls)) in enumerate(zip(lams, styles), start=1):
        u = u_of(lam, r)
        phi = u / r
        phi = phi / np.max(np.abs(phi))
        if phi[0] < 0:
            phi, u = -phi, -u
        norm = sum(quad(lambda x: float(u_of(lam, np.array([x]))[0]) ** 2, a, b, limit=200)[0]
                   for a, b in zip([0.0] + RAD[:-1], RAD))
        mant, expo = f"{lam:.4e}".split("e")
        lab = rf"$n = {n}$, $\Lambda_{n} = {mant}\times10^{{{int(expo)}}}\ \mathrm{{s^{{-1}}}}$"
        axes[0].plot(rum, phi, color=col, ls=ls, lw=1.6, label=lab)
        axes[1].plot(rum, u / math.sqrt(norm), color=col, ls=ls, lw=1.6, label=lab)

    for ax in axes:
        ax.axhline(0.0, color="0.3", lw=0.6)
        for ro in RAD[:-1]:
            ax.axvline(ro * 1e6, color="0.55", lw=0.8, ls=":")
        ax.set_xlim(0, RAD[-1] * 1e6)
        ax.grid(False)
    edges = [0.0] + RAD
    for i, name in enumerate(NAMES):
        axes[0].text(0.5 * (edges[i] + edges[i + 1]) * 1e6, 1.12, name, ha="center", va="bottom", fontsize=8.5)
    axes[0].set_ylim(-1.05, 1.25)
    axes[0].set_ylabel(r"$\phi_n(r)\,/\,\max|\phi_n|$ (1)")
    axes[1].set_ylabel(r"$u_n(r)\,/\,\sqrt{N_n}$ $(\mathrm{m^{-1/2}})$")
    axes[1].set_xlabel(r"radius $r$ ($\mathrm{\mu m}$)")
    axes[0].set_title("(a) concentration shape", fontsize=9.5, loc="left")
    axes[1].set_title(r"(b) transformed eigenfunction $u_n = r\,\phi_n$, unit norm", fontsize=9.5, loc="left")
    axes[0].legend(fontsize=8, loc="lower left", frameon=False)
    fig.text(0.99, 0.005, "Synthetic v1.0 input, not literature data. Dotted lines: interfaces R1 to R4.",
             ha="right", va="bottom", fontsize=7, color="0.35")
    fig.tight_layout(rect=(0, 0.02, 1, 1))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(OUT, dpi=150, facecolor="white")
    print(f"wrote {OUT}")
    for n, lam in enumerate(lams, start=1):
        print(f"Lambda_{n} = {lam:.15e} s^-1")


if __name__ == "__main__":
    main()
