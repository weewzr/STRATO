"""Figure 1.1 for the STRATO v1.0.0 derivation document: TRISO cross section.

Draws a to-scale cross section of the five-layer TRISO particle and a radial
axis panel showing the five intervals of r between 0 and R = R_5.

Source of dimensions (measured values, AGR-1 Baseline fuel, average values):
    Morris, R. N., Baldwin, C. A., Demkowicz, P. A., Hunn, J. D., Reber, E. L.
    (2016). Performance of AGR-1 high-temperature reactor fuel during
    post-irradiation heating tests. Nuclear Engineering and Design 306, 24-35.
    https://doi.org/10.1016/j.nucengdes.2016.04.031 , Table 2 (p. 27).
    Kernel diameter 350 um; buffer 103.5 um; IPyC 39.4 um; SiC 35.3 um;
    OPyC 41.0 um. AGR-1 kernels are UCO; the label "UCO/UO2" names both common
    TRISO kernel types.

Outputs (200 dpi, white background, sized for 15 cm print width):
    docs/mathematics/v1.0.0/figures/triso_cross_section.png
    docs/mathematics/v1.0.0/figures/radial_domains.png

Run:
    C:/Y3S1/Nuclear/STRATO/.venv/Scripts/python.exe -I -X utf8 \
        scripts/figures/v1_0_0_triso_cross_section.py
"""

from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.patches import Circle, Rectangle  # noqa: E402

REPO = Path(__file__).resolve().parents[2]
OUT_DIR = REPO / "docs" / "mathematics" / "v1.0.0" / "figures"

# --- Data (Morris et al. 2016, Table 2), micrometres -------------------------
KERNEL_DIAMETER_UM = 350.0
THICKNESS_UM = {"Buffer": 103.5, "IPyC": 39.4, "SiC": 35.3, "OPyC": 41.0}

NAMES = ["Kernel", "Buffer", "IPyC", "SiC", "OPyC"]
LONG_LABELS = [
    "Kernel (UCO/UO$_2$)",
    "Buffer (porous carbon)",
    "IPyC (inner pyrolytic carbon)",
    "SiC (silicon carbide)",
    "OPyC (outer pyrolytic carbon)",
]
# Okabe-Ito colourblind-safe palette.
COLOURS = ["#D55E00", "#F0E442", "#56B4E9", "#009E73", "#CC79A7"]

CAPTION = (
    "Figure 1.1: Cross section of a TRISO particle drawn to scale with the "
    "nominal AGR-1 Baseline dimensions (kernel diameter 350 um; buffer, IPyC, "
    "SiC and OPyC thicknesses 103.5, 39.4, 35.3 and 41.0 um) reported by "
    "Morris et al. (2016), Table 2. R_1 to R_5 are the outer radii of the five "
    "layers, so R = R_5 = 394.2 um. The lower panel shows the radial "
    "coordinate r from 0 to R divided into the five layer intervals that the "
    "diffusion model treats as separate materials."
)


def outer_radii():
    """Return [R_1, ..., R_5] in micrometres (cumulative sums)."""
    radii = [KERNEL_DIAMETER_UM / 2.0]
    for name in NAMES[1:]:
        radii.append(radii[-1] + THICKNESS_UM[name])
    return radii


RADII = outer_radii()
R_TOTAL = RADII[-1]
FS = 9  # base font size in points


def draw_cross_section(ax):
    # Concentric discs, outermost first so inner discs sit on top.
    for i in reversed(range(5)):
        ax.add_patch(
            Circle((0, 0), RADII[i], facecolor=COLOURS[i], edgecolor="black",
                   linewidth=0.8, zorder=1 + (4 - i) * 0.0 + (5 - i))
        )

    # Radial line along the negative x axis with boundary ticks.
    ax.plot([0, -R_TOTAL], [0, 0], color="black", linewidth=1.0, zorder=20)
    ax.plot([0], [0], marker="o", markersize=3, color="black", zorder=21)
    ax.annotate("$r=0$", xy=(0, 0), xytext=(-40, -62), fontsize=FS - 1,
                ha="center", va="top",
                bbox=dict(boxstyle="round,pad=0.15", fc="white", ec="none",
                          alpha=0.9),
                arrowprops=dict(arrowstyle="-", linewidth=0.6), zorder=22)
    tick = 9.0
    for k, rk in enumerate(RADII):
        ax.plot([-rk, -rk], [-tick, tick], color="black", linewidth=1.0,
                zorder=20)
        above = (k % 2 == 0)
        y = 14.0 if above else -14.0
        ax.text(-rk, y, f"$R_{k + 1}$", fontsize=FS, ha="center",
                va="bottom" if above else "top", zorder=22,
                bbox=dict(boxstyle="round,pad=0.12", fc="white", ec="none",
                          alpha=0.9))

    # Layer labels with leader lines, in a column to the right.
    mids = [0.6 * RADII[0]] + [
        0.5 * (RADII[i - 1] + RADII[i]) for i in range(1, 5)
    ]
    angles_deg = [-70.0, -28.0, -2.0, 22.0, 46.0]
    text_y = [-310.0, -165.0, -10.0, 150.0, 310.0]
    text_x = 470.0
    import math
    for i in range(5):
        a = math.radians(angles_deg[i])
        px, py = mids[i] * math.cos(a), mids[i] * math.sin(a)
        ax.annotate(
            LONG_LABELS[i], xy=(px, py), xytext=(text_x, text_y[i]),
            fontsize=FS, ha="left", va="center", zorder=30,
            arrowprops=dict(arrowstyle="-", color="black", linewidth=0.7,
                            shrinkA=2, shrinkB=0,
                            connectionstyle="arc3,rad=0"),
        )
        ax.plot([px], [py], marker="o", markersize=2.5, color="black",
                zorder=31)

    # Scale bar: 100 um.
    x0, y0 = text_x, -R_TOTAL + 10.0
    ax.add_patch(Rectangle((x0, y0), 100.0, 8.0, facecolor="black",
                           edgecolor="black"))
    ax.text(x0 + 50.0, y0 - 10.0, r"$100\ \mu\mathrm{m}$", fontsize=FS,
            ha="center", va="top")

    ax.set_xlim(-R_TOTAL - 30.0, text_x + 560.0)
    ax.set_ylim(-R_TOTAL - 45.0, R_TOTAL + 25.0)
    ax.set_aspect("equal")
    ax.axis("off")


def draw_radial_axis(ax):
    """Radial axis r in [0, R] split into the five layer intervals."""
    edges = [0.0] + RADII
    h = 1.0
    for i in range(5):
        a, b = edges[i], edges[i + 1]
        ax.add_patch(Rectangle((a, 0.0), b - a, h, facecolor=COLOURS[i],
                               edgecolor="black", linewidth=0.8))
        name = NAMES[i]
        ax.text(0.5 * (a + b), 0.5 * h, name, ha="center", va="center",
                fontsize=FS - 1 if b - a < 60 else FS)
        ax.text(0.5 * (a + b), h + 0.12, f"$i={i + 1}$", ha="center",
                va="bottom", fontsize=FS - 1)
    # Boundary ticks with symbolic and numeric labels.
    for k, rk in enumerate(edges):
        ax.plot([rk, rk], [-0.12, 0.0], color="black", linewidth=0.8)
        sym = "$0$" if k == 0 else (
            f"$R_{k}$" if k < 5 else "$R_5=R$")
        ax.text(rk, -0.18, sym, ha="center", va="top", fontsize=FS)
        ax.text(rk, -0.62 - (0.38 if k in (3, 5) else 0.0), f"{rk:.1f}",
                ha="center", va="top",
                fontsize=FS - 1.5, color="0.25")
    ax.annotate("", xy=(R_TOTAL + 38.0, 0.0), xytext=(0.0, 0.0),
                arrowprops=dict(arrowstyle="-|>", linewidth=0.9,
                                color="black"))
    ax.text(R_TOTAL + 42.0, 0.0, "$r$", ha="left", va="center", fontsize=FS + 1)
    ax.text(-12.0, -0.80, r"$r\ (\mu\mathrm{m})$:", ha="right", va="center",
            fontsize=FS - 1.5, color="0.25")
    ax.set_xlim(-95.0, R_TOTAL + 70.0)
    ax.set_ylim(-1.45, h + 0.55)
    ax.axis("off")


def main():
    # Arithmetic check of the cumulative radii against hand values.
    expected = [175.0, 278.5, 317.9, 353.2, 394.2]
    ok = all(abs(a - b) < 1e-9 for a, b in zip(RADII, expected))
    print(("PASS" if ok else "FAIL") + " radii R_1..R_5 (um) = "
          + ", ".join(f"{r:.1f}" for r in RADII))

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    plt.rcParams.update({"font.size": FS, "savefig.facecolor": "white"})

    # Combined figure: cross section above, radial axis below.
    fig = plt.figure(figsize=(6.3, 6.0), facecolor="white")
    gs = fig.add_gridspec(2, 1, height_ratios=[3.4, 1.0], hspace=0.02)
    draw_cross_section(fig.add_subplot(gs[0]))
    draw_radial_axis(fig.add_subplot(gs[1]))
    p1 = OUT_DIR / "triso_cross_section.png"
    fig.savefig(p1, dpi=200, facecolor="white", bbox_inches="tight",
                pad_inches=0.05)
    plt.close(fig)

    # Radial-axis panel alone.
    fig = plt.figure(figsize=(6.3, 1.6), facecolor="white")
    draw_radial_axis(fig.add_subplot(1, 1, 1))
    p2 = OUT_DIR / "radial_domains.png"
    fig.savefig(p2, dpi=200, facecolor="white", bbox_inches="tight",
                pad_inches=0.05)
    plt.close(fig)

    print(f"wrote {p1}")
    print(f"wrote {p2}")
    print("CAPTION: " + CAPTION)


if __name__ == "__main__":
    main()
