"""Figure 5.1 for the STRATO v1.0.0 derivation document: one spherical control volume.

Left panel: a slice through the centre of the particle showing cell j (shaded shell
between the faces r_{j-1/2} and r_{j+1/2}) and its neighbours j-1 and j+1, with the
volume centroids marked on one radial ray.
Right panel: the same three cells laid out along the radius, with the face areas,
the centroid r_j (computed with the formula of src/numerical/finite_volume.rs,
volume_centroid, lines 42 to 48), the cell midpoints for comparison, the
centre-to-face distances used in the conductances (lines 84 to 85) and the two
face flows of the cell balance.

The radii are illustrative (dimensionless faces at 1, 2, 3, 4), chosen so that the
outward shift of the centroid is visible; they are not STRATO input values.

Output (150 dpi): docs/mathematics/v1.0.0/figures/control_volume.png

Run:
    C:/Y3S1/Nuclear/STRATO/.venv/Scripts/python.exe -I -X utf8 scripts/figures/v1_0_0_control_volume.py
"""

import math
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.patches import Circle, FancyArrowPatch, Rectangle, Wedge  # noqa: E402

REPO = Path(__file__).resolve().parents[2]
OUT = REPO / "docs" / "mathematics" / "v1.0.0" / "figures" / "control_volume.png"


def centroid(a, b):
    """Same formula as volume_centroid in finite_volume.rs."""
    if a == 0.0:
        return 0.75 * b
    return 0.75 * (b**4 - a**4) / (b**3 - a**3)


faces = [1.0, 2.0, 3.0, 4.0]          # r_{j-3/2}, r_{j-1/2}, r_{j+1/2}, r_{j+3/2}
cent = [centroid(faces[k], faces[k + 1]) for k in range(3)]   # r_{j-1}, r_j, r_{j+1}
mids = [0.5 * (faces[k] + faces[k + 1]) for k in range(3)]

SHADE = "#9ecae1"
NEIGH = "#e6eef5"
FACE = "#08306b"
CENT = "#d95f02"
FLOW = "#1b7837"

fig, (axL, axR) = plt.subplots(1, 2, figsize=(11.0, 5.2), gridspec_kw={"width_ratios": [1.0, 1.35]})

# ---------------------------------------------------------------- left: slice
axL.set_aspect("equal")
axL.add_patch(Circle((0, 0), faces[3], facecolor=NEIGH, edgecolor="none"))
axL.add_patch(Wedge((0, 0), faces[2], 0, 360, width=faces[2] - faces[1], facecolor=SHADE, edgecolor="none"))
axL.add_patch(Circle((0, 0), faces[0], facecolor="white", edgecolor="0.6", lw=0.8, ls=":"))
for rf in faces:
    lw = 2.0 if rf in (faces[1], faces[2]) else 0.9
    axL.add_patch(Circle((0, 0), rf, fill=False, edgecolor=FACE, lw=lw))
axL.add_patch(Circle((0, 0), cent[1], fill=False, edgecolor=CENT, lw=1.2, ls="--"))
axL.plot([0], [0], "k+", ms=8)
axL.text(0.08, -0.35, "centre $r=0$", fontsize=9)
ang = math.radians(35.0)
axL.plot([0, 4.3 * math.cos(ang)], [0, 4.3 * math.sin(ang)], color="0.3", lw=0.8)
for k, rc in enumerate(cent):
    axL.plot(rc * math.cos(ang), rc * math.sin(ang), "o", color=CENT, ms=6, zorder=5)
axL.text(4.35 * math.cos(ang), 4.35 * math.sin(ang), "$r$", fontsize=11)
axL.annotate("cell $j$\n(volume $V_j$)", xy=(-2.5 * math.cos(0.6), -2.5 * math.sin(0.6)),
             xytext=(-4.6, -4.6), fontsize=10, arrowprops=dict(arrowstyle="-", color="0.3"))
axL.annotate("inner face, area\n$A_{j-1/2}=4\\pi r_{j-1/2}^2$", xy=(faces[1] * math.cos(2.4), faces[1] * math.sin(2.4)),
             xytext=(-4.7, 3.5), fontsize=9, color=FACE, arrowprops=dict(arrowstyle="-", color=FACE))
axL.annotate("outer face, area\n$A_{j+1/2}=4\\pi r_{j+1/2}^2$", xy=(faces[2] * math.cos(1.25), faces[2] * math.sin(1.25)),
             xytext=(0.6, 4.6), fontsize=9, color=FACE, arrowprops=dict(arrowstyle="-", color=FACE))
axL.annotate("centroid sphere $r_j$", xy=(cent[1] * math.cos(-0.5), cent[1] * math.sin(-0.5)),
             xytext=(1.6, -4.7), fontsize=9, color=CENT, arrowprops=dict(arrowstyle="-", color=CENT))
axL.set_xlim(-5.0, 5.0)
axL.set_ylim(-5.0, 5.4)
axL.axis("off")
axL.set_title("(a) slice through the centre", fontsize=11)

# ---------------------------------------------------------------- right: radius line
y0, hgt = 0.0, 1.0
for k in range(3):
    axR.add_patch(Rectangle((faces[k], y0), faces[k + 1] - faces[k], hgt,
                            facecolor=SHADE if k == 1 else NEIGH, edgecolor="none"))
for rf in faces:
    axR.plot([rf, rf], [y0 - 0.05, y0 + hgt + 0.05], color=FACE, lw=2.0 if rf in faces[1:3] else 1.0)
names_f = ["$r_{j-3/2}$", "$r_{j-1/2}$", "$r_{j+1/2}$", "$r_{j+3/2}$"]
for rf, nm in zip(faces, names_f):
    axR.text(rf, y0 + hgt + 0.12, nm, ha="center", fontsize=10, color=FACE)
names_c = ["$r_{j-1}$", "$r_j$", "$r_{j+1}$"]
for k in range(3):
    axR.plot(cent[k], y0 + 0.5 * hgt, "o", color=CENT, ms=7, zorder=5)
    axR.text(cent[k], y0 + 0.62 * hgt, names_c[k], ha="center", fontsize=10, color=CENT)
    axR.plot([mids[k], mids[k]], [y0 + 0.40, y0 + 0.46], color="0.25", lw=1.2)
axR.text(mids[1], y0 + 0.28, "midpoint", ha="center", fontsize=8, color="0.25")
axR.text(2.5, y0 + 0.88, "cell $j$", ha="center", fontsize=10)
axR.text(1.5, y0 + 0.88, "cell $j-1$", ha="center", fontsize=9, color="0.35")
axR.text(3.5, y0 + 0.88, "cell $j+1$", ha="center", fontsize=9, color="0.35")


def dim(xa, xb, yy, label, color="0.15"):
    axR.add_patch(FancyArrowPatch((xa, yy), (xb, yy), arrowstyle="<->", mutation_scale=10, color=color, lw=1.0))
    axR.text(0.5 * (xa + xb), yy - 0.17, label, ha="center", va="top", fontsize=9, color=color)


dim(cent[0], faces[1], -0.35, "$\\delta^L_{j-1/2}$")
dim(faces[1], cent[1], -0.35, "$\\delta^R_{j-1/2}$")
dim(cent[1], faces[2], -0.35, "$\\delta^L_{j+1/2}$")
dim(faces[2], cent[2], -0.35, "$\\delta^R_{j+1/2}$")
for x in (cent[0], cent[1], cent[2]):
    axR.plot([x, x], [-0.45, y0 + 0.5], color="0.6", lw=0.6, ls=":")

# face flows
for rf, lab in ((faces[1], "$A_{j-1/2}J_{j-1/2}$\n$=G_{j-1/2}(C_{j-1}-C_j)$"),
                (faces[2], "$A_{j+1/2}J_{j+1/2}$\n$=G_{j+1/2}(C_j-C_{j+1})$")):
    axR.add_patch(FancyArrowPatch((rf - 0.32, 1.55), (rf + 0.32, 1.55), arrowstyle="-|>", mutation_scale=14,
                                  color=FLOW, lw=1.6))
    axR.text(rf, 1.72, lab, ha="center", va="bottom", fontsize=9, color=FLOW)
axR.text(2.5, -1.05,
         "$V_j\\,dC_j/dt = A_{j-1/2}J_{j-1/2} - A_{j+1/2}J_{j+1/2}$   (flows counted positive outward)",
         ha="center", fontsize=9.5)
axR.text(2.5, -1.35, "centroid $r_j=\\frac{3}{4}(r_{j+1/2}^4-r_{j-1/2}^4)/(r_{j+1/2}^3-r_{j-1/2}^3)$ lies outward of the midpoint",
         ha="center", fontsize=9, color=CENT)
axR.annotate("", xy=(4.35, -0.75), xytext=(0.75, -0.75),
             arrowprops=dict(arrowstyle="-|>", color="0.3", lw=0.9))
axR.text(4.38, -0.80, "$r$", fontsize=11)
axR.set_xlim(0.6, 4.6)
axR.set_ylim(-1.55, 2.45)
axR.axis("off")
axR.set_title("(b) the same cells along the radius", fontsize=11)

fig.tight_layout()
OUT.parent.mkdir(parents=True, exist_ok=True)
fig.savefig(OUT, dpi=150, facecolor="white")
print(f"wrote {OUT}")
