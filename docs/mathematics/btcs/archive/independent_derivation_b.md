# STRATO BTCS scheme: independent derivation B

Status: working document for the Phase A review of the spherical BTCS finite-difference solver. It is one of two
independent derivations. A third agent compares them. This derivation was written from first principles. It does not
use the owner's original PDF documents or the other derivation. Wherever it says something about the owner's
handwritten treatment, it states the checks that the comparison should apply and does not claim what the notes contain.

Scope: the baseline model only (five layers, constant diffusivity in each layer, no source, no decay). Section 9
describes how generation (G1) and decay (G2) will enter the discrete balance later. Every number in Section 10 is a
numerical verification of the discrete mathematics on synthetic or analytical test problems. None of it is
experimental validation.

Classification labels used below:

- **PROVED**: a complete proof is given here, under the stated hypotheses.
- **PROOF OUTLINE**: every step of the argument is given, but constants are not tracked and the argument has not
  been checked independently.
- **OBSERVED**: shown only by the scratch computations of Section 10.
- **NOT PROVEN**: stated as open.

---

## 0. Notation and units

| Symbol | Meaning | Unit |
|---|---|---|
| $r$ | radial distance from the particle centre | $\mathrm{m}$ |
| $t$ | time | $\mathrm{s}$ |
| $C(r,t)$ | concentration of the diffusing species | $\mathrm{mol\ m^{-3}}$ |
| $D_{\mathrm{kernel}}, D_{\mathrm{buffer}}, D_{\mathrm{IPyC}}, D_{\mathrm{SiC}}, D_{\mathrm{OPyC}}$ | diffusivity of each material, constant within the material | $\mathrm{m^2\ s^{-1}}$ |
| $R_1,\dots,R_5$ | outer radii of kernel, buffer, IPyC, SiC, OPyC; $R_5$ is the particle surface | $\mathrm{m}$ |
| $C_0$ | initial kernel concentration | $\mathrm{mol\ m^{-3}}$ |
| $i$ | global radial node index, $i = 0,1,\dots,N$ | none |
| $j$ | time-step index, $t_j = j\,\Delta t$ | none |
| $r_i$ | radius of node $i$; $r_0 = 0$, $r_N = R_5$ | $\mathrm{m}$ |
| $C_i^j$ | discrete concentration at node $i$ and time step $j$ | $\mathrm{mol\ m^{-3}}$ |
| $\Delta r$ | uniform spacing, where the grid is uniform | $\mathrm{m}$ |
| $\Delta r_{i,\mathrm{inner}} = r_i - r_{i-1}$ | spacing on the inner side of node $i$ | $\mathrm{m}$ |
| $\Delta r_{i,\mathrm{outer}} = r_{i+1} - r_i$ | spacing on the outer side of node $i$ | $\mathrm{m}$ |
| $\overline{\Delta r}_i = \tfrac12\left(\Delta r_{i,\mathrm{inner}} + \Delta r_{i,\mathrm{outer}}\right)$ | mean spacing at node $i$ | $\mathrm{m}$ |
| $D_{i,\mathrm{inner}}$, $D_{i,\mathrm{outer}}$ | diffusivity of the material that fills $(r_{i-1}, r_i)$ and $(r_i, r_{i+1})$ | $\mathrm{m^2\ s^{-1}}$ |
| $\Delta t$ | time step | $\mathrm{s}$ |
| $\mu = D\,\Delta t/\left(\Delta r\right)^2$ | mesh ratio of a uniform region with diffusivity $D$ | none |
| $I_1, I_2, I_3, I_4$ | node indices of the interfaces: $r_{I_1} = R_1$, ..., $r_{I_4} = R_4$ | none |
| $V_i$ | nodal weight (volume) of node $i$, Eq. (A.8) | $\mathrm{m^3}$ |
| $g_{i,\mathrm{outer}}$ | connection coefficient between nodes $i$ and $i+1$, Eq. (A.7) | $\mathrm{m^3\ s^{-1}}$ |
| $P_{i,\mathrm{outer}}^{j}$ | discrete inward flow across the sphere between $r_i$ and $r_{i+1}$ | $\mathrm{mol\ s^{-1}}$ |
| $M_h^j$ | discrete inventory at time step $j$ | $\mathrm{mol}$ |
| $\Phi^j$ | discrete release rate through $r = R_5$ | $\mathrm{mol\ s^{-1}}$ |
| $M_{\mathrm{rel}}^j$ | discrete cumulative release | $\mathrm{mol}$ |
| $G$ | volumetric source (Section 9 only) | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $\lambda$ | radioactive decay constant (Section 9 only) | $\mathrm{s^{-1}}$ |

Conventions. A superscript on $C$ is a time index, never a power. A power always encloses the whole variable in
parentheses, for example $\left(r_i\right)^2$ or $\left(\Delta r\right)^2$. In $\left(\Delta r\right)^2$ the symbol $\Delta r$ is
one variable. Derivatives of the exact solution are written $\partial C/\partial r$, $\partial^2 C/\partial r^2$, and so on.
Where a derivative is evaluated at a point, the point is given, for example $\partial^4 C/\partial r^4(0)$. The letter
$\lambda$ is reserved for decay. For this reason the mesh ratio is $\mu$.

The index $i$ counts nodes only. Materials are always named. The interface indices $I_1,\dots,I_4$ are node indices.

---

## 1. The continuous model (restated, not re-derived)

This derivation uses the physical model that the repository already states in
`docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md`, Part III. The radii come from the inputs (synthetic
benchmark `inputs/synthetic/synthetic_five_layer.toml`, AGR-1 radii in `data/geometry/geometry.csv`).

Inside each material, with constant $D$, the governing equation is

$$
\frac{\partial C}{\partial t} = \frac{1}{\left(r\right)^2}\frac{\partial}{\partial r}\left[\left(r\right)^2 D\,\frac{\partial C}{\partial r}\right].
$$

$D$ is constant in the material, so it comes out of the derivative. The product rule gives
$\frac{\partial}{\partial r}\left[\left(r\right)^2 \frac{\partial C}{\partial r}\right] = 2r\,\frac{\partial C}{\partial r} + \left(r\right)^2\frac{\partial^2 C}{\partial r^2}$.
Dividing by $\left(r\right)^2$ then gives

$$
\frac{\partial C}{\partial t} = D\left[\frac{\partial^2 C}{\partial r^2} + \frac{2}{r}\frac{\partial C}{\partial r}\right]. \qquad \text{(1.1)}
$$

The conditions are as follows.

- Centre, $r = 0$: $\partial C/\partial r = 0$, and $C$ is finite.
- Interfaces $R_1,\dots,R_4$ (perfect contact): $C$ is continuous, and the radial flux $-D\,\partial C/\partial r$ is
  continuous. For example, at $R_1$ the condition is
  $D_{\mathrm{kernel}}\,\partial C/\partial r(R_1^-) = D_{\mathrm{buffer}}\,\partial C/\partial r(R_1^+)$.
- Surface: $C(R_5,t) = 0$ (absorbing).
- Start: $C(r,0) = C_0$ for $0 \le r < R_1$ and $0$ for $R_1 < r < R_5$.

Two consequences of these conditions are used repeatedly below.

**(1.2) The time derivative is continuous at an interface.** $C(R_m,t)$ has a single value for every $t$. Its time
derivative is therefore the same from both sides. Eq. (1.1) holds on each side, so

$$
D_{\mathrm{left}}\left[\frac{\partial^2 C}{\partial r^2} + \frac{2}{R_m}\frac{\partial C}{\partial r}\right]_{R_m^-}
= D_{\mathrm{right}}\left[\frac{\partial^2 C}{\partial r^2} + \frac{2}{R_m}\frac{\partial C}{\partial r}\right]_{R_m^+}.
$$

**(1.3) The continuous balance.** Write $Q(r,t) = \left(r\right)^2 D\,\partial C/\partial r$ (unit $\mathrm{mol\ s^{-1}}$).
$4\pi Q$ is the inward flow through the sphere of radius $r$. $Q$ is continuous across the interfaces and satisfies
$\partial Q/\partial r = \left(r\right)^2\,\partial C/\partial t$ in every material. With the inventory
$M(t) = 4\pi\int_0^{R_5} \left(r\right)^2 C\,dr$, these give

$$
\frac{dM}{dt} = -\Phi_{\mathrm{exact}}, \qquad \Phi_{\mathrm{exact}}(t) = -4\pi\left(R_5\right)^2 D_{\mathrm{OPyC}}\,\frac{\partial C}{\partial r}(R_5,t).
$$

---

## 2. (a) Interior nodes

### 2.1 Uniform spacing, one material

Apply central differences in space and Backward Euler in time at the new level $j+1$, at a node $i \ge 1$ whose two
neighbours lie in the same material, at the same spacing $\Delta r$:

$$
\frac{C_i^{j+1}-C_i^j}{\Delta t}
= D\left[\frac{C_{i+1}^{j+1}-2C_i^{j+1}+C_{i-1}^{j+1}}{\left(\Delta r\right)^2}
+ \frac{2}{r_i}\,\frac{C_{i+1}^{j+1}-C_{i-1}^{j+1}}{2\Delta r}\right]. \qquad \text{(A.1)}
$$

Step 1. Multiply by $\Delta t$. In the first term this gives $D\Delta t/\left(\Delta r\right)^2 = \mu$. In the second
term it gives $D\Delta t\cdot\frac{2}{r_i}\cdot\frac{1}{2\Delta r} = \frac{D\Delta t}{r_i\,\Delta r} = \mu\,\frac{\Delta r}{r_i}$:

$$
C_i^{j+1}-C_i^j = \mu\left(C_{i+1}^{j+1}-2C_i^{j+1}+C_{i-1}^{j+1}\right) + \mu\,\frac{\Delta r}{r_i}\left(C_{i+1}^{j+1}-C_{i-1}^{j+1}\right).
$$

Step 2. Collect the terms by unknown. For $C_{i-1}^{j+1}$ the coefficient is $\mu - \mu\Delta r/r_i$. For $C_i^{j+1}$ it is
$-2\mu$. For $C_{i+1}^{j+1}$ it is $\mu + \mu\Delta r/r_i$.

Step 3. Move every unknown at level $j+1$ to the left side:

$$
a_i\,C_{i-1}^{j+1} + b_i\,C_i^{j+1} + c_i\,C_{i+1}^{j+1} = d_i, \qquad \text{(A.2)}
$$

$$
a_i = -\mu\left(1 - \frac{\Delta r}{r_i}\right),\qquad
b_i = 1 + 2\mu,\qquad
c_i = -\mu\left(1 + \frac{\Delta r}{r_i}\right),\qquad
d_i = C_i^j. \qquad \text{(A.3)}
$$

On the grid $r_i = i\,\Delta r$ this becomes $a_i = -\mu\left(1 - 1/i\right)$ and $c_i = -\mu\left(1 + 1/i\right)$.
Every row sums to $a_i + b_i + c_i = 1$.

Signs. $c_i < 0$ always. $a_i \le 0$ exactly when $r_i \ge \Delta r$, which holds for every $i \ge 1$ on a grid that
starts at $r_0 = 0$. At $i = 1$, $a_1 = 0$ (Section 3).

### 2.2 The interior stencil is a difference of two flows (coordinator note 1)

**Proposition A.1 (PROVED).** Let node $i$ have neighbours at $r_{i-1} = r_i - \Delta r$ and $r_{i+1} = r_i + \Delta r$.
The value $r_i$ can be anything; the grid does not need $r_i = i\,\Delta r$. Then

$$
\left(r_i\right)^2\left[\frac{C_{i+1}-2C_i+C_{i-1}}{\left(\Delta r\right)^2} + \frac{2}{r_i}\frac{C_{i+1}-C_{i-1}}{2\Delta r}\right]
= \frac{r_i r_{i+1}\left(C_{i+1}-C_i\right) - r_{i-1} r_i\left(C_i - C_{i-1}\right)}{\left(\Delta r\right)^2}. \qquad \text{(A.4)}
$$

*Proof.* Use $r_i r_{i+1} = \left(r_i\right)^2 + r_i\Delta r$ and $r_{i-1}r_i = \left(r_i\right)^2 - r_i\Delta r$. The
numerator on the right is then

$$
\left(r_i\right)^2\left[\left(C_{i+1}-C_i\right) - \left(C_i-C_{i-1}\right)\right] + r_i\Delta r\left[\left(C_{i+1}-C_i\right) + \left(C_i - C_{i-1}\right)\right]
= \left(r_i\right)^2\left(C_{i+1}-2C_i+C_{i-1}\right) + r_i\Delta r\left(C_{i+1}-C_{i-1}\right).
$$

Divide by $\left(\Delta r\right)^2$. The second part becomes $\left(r_i\right)^2\cdot\frac{1}{r_i\Delta r}\left(C_{i+1}-C_{i-1}\right)$,
and $\frac{1}{r_i\Delta r} = \frac{2}{r_i}\cdot\frac{1}{2\Delta r}$. This is the left side. $\square$

Multiply (A.4) by $4\pi D\,\Delta r$. Then $4\pi\left(r_i\right)^2\Delta r$ times the owner's operator equals
$P_{i,\mathrm{outer}} - P_{i-1,\mathrm{outer}}$, with

$$
P_{i,\mathrm{outer}} = 4\pi\,g_{i,\mathrm{outer}}\left(C_{i+1}-C_i\right), \qquad g_{i,\mathrm{outer}} = \frac{D\,r_i\,r_{i+1}}{\Delta r}. \qquad \text{(A.5)}
$$

The same number $P_{i,\mathrm{outer}}$ appears in row $i$, with a plus sign, and in row $i+1$, with a minus sign. In row
$i+1$ the inner flow is $4\pi D r_{i+1} r_i \left(C_{i+1}-C_i\right)/\Delta r$, which is the same quantity. The
coordinator's note (1) is therefore correct, and it holds more generally than stated: the grid only needs to be
uniform around each node, not to satisfy $r_i = i\,\Delta r$. The face factor $r_i r_{i+1}$ is not the exact sphere
area at the midpoint. It equals $\left(\tfrac{r_i+r_{i+1}}{2}\right)^2 - \tfrac14\left(\Delta r\right)^2$. The difference is
$O\!\left(\left(\Delta r\right)^2\right)$ and is part of the truncation error (Section 8.1).

### 2.3 Flux form on a general grid (used everywhere below)

Proposition A.1 suggests one general scheme for every node $i = 1,\dots,N-1$, on any grid and across interfaces:

$$
V_i\,\frac{C_i^{j+1}-C_i^j}{\Delta t} = P_{i,\mathrm{outer}}^{j+1} - P_{i-1,\mathrm{outer}}^{j+1}, \qquad \text{(A.6)}
$$

$$
P_{i,\mathrm{outer}}^{j+1} = 4\pi\,g_{i,\mathrm{outer}}\left(C_{i+1}^{j+1}-C_i^{j+1}\right),\qquad
g_{i,\mathrm{outer}} = \frac{D_{i,\mathrm{outer}}\;r_i\,r_{i+1}}{\Delta r_{i,\mathrm{outer}}}, \qquad \text{(A.7)}
$$

$$
V_i = 4\pi\left(r_i\right)^2\,\overline{\Delta r}_i. \qquad \text{(A.8)}
$$

$D_{i,\mathrm{outer}}$ is the diffusivity of the single material that fills $(r_i, r_{i+1})$. This is well defined
because every interface is a node (Section 4). Dividing (A.6) by $V_i/\Delta t$ gives the tridiagonal row (A.2) with

$$
a_i = -\frac{\Delta t\,D_{i,\mathrm{inner}}\;r_{i-1}}{r_i\,\Delta r_{i,\mathrm{inner}}\;\overline{\Delta r}_i},\qquad
c_i = -\frac{\Delta t\,D_{i,\mathrm{outer}}\;r_{i+1}}{r_i\,\Delta r_{i,\mathrm{outer}}\;\overline{\Delta r}_i},\qquad
b_i = 1 - a_i - c_i,\qquad d_i = C_i^j. \qquad \text{(A.9)}
$$

Here $D_{i,\mathrm{inner}} = D_{i-1,\mathrm{outer}}$. The derivation of $a_i$ is as follows:
$a_i = -\Delta t\cdot 4\pi g_{i-1,\mathrm{outer}}/V_i = -\Delta t\,D_{i,\mathrm{inner}}\,r_{i-1}r_i/\left(\Delta r_{i,\mathrm{inner}}\left(r_i\right)^2\overline{\Delta r}_i\right)$,
and one factor $r_i$ cancels. The derivation of $c_i$ is the same.

Check. With equal spacing $\Delta r$ on both sides and one material, $\overline{\Delta r}_i = \Delta r$ and
$r_{i\mp1}/r_i = 1 \mp \Delta r/r_i$. Then (A.9) gives back (A.3) exactly. On a uniform single-material grid the
general scheme is therefore the owner's stencil itself.

**Recommended assembly (implementation note).** Assemble (A.6) multiplied out, that is, row $i$ as

$$
\left(V_i + 4\pi\Delta t\left(g_{i-1,\mathrm{outer}} + g_{i,\mathrm{outer}}\right)\right)C_i^{j+1} - 4\pi\Delta t\,g_{i-1,\mathrm{outer}}\,C_{i-1}^{j+1} - 4\pi\Delta t\,g_{i,\mathrm{outer}}\,C_{i+1}^{j+1} = V_i\,C_i^j.
$$

For $i \ge 1$ this matrix is symmetric. The flows it uses are exactly the flows used for the release and the
inventory, so the discrete balance holds to a smaller rounding residual (Section 10.1).

---

## 3. (b) The centre node, $i = 0$

### 3.1 The centre-node equation

At $r = 0$ the term $\frac{2}{r}\frac{\partial C}{\partial r}$ has the form $0/0$. A smooth radially symmetric field is
even in $r$: extended to negative $r$ along a line through the centre, it is an even function. Its Taylor series about
$0$ therefore has only even powers,
$C(r) = C(0) + \tfrac12 \left(r\right)^2 \tfrac{\partial^2 C}{\partial r^2}(0) + \tfrac1{24}\left(r\right)^4 \tfrac{\partial^4 C}{\partial r^4}(0) + \dots$

From this, $\frac{\partial C}{\partial r}(r) = r\,\frac{\partial^2 C}{\partial r^2}(0) + O\!\left(\left(r\right)^3\right)$, so
$\frac{2}{r}\frac{\partial C}{\partial r} \to 2\,\frac{\partial^2 C}{\partial r^2}(0)$. In the limit, Eq. (1.1) becomes

$$
\frac{\partial C}{\partial t}(0,t) = 3\,D_{\mathrm{kernel}}\,\frac{\partial^2 C}{\partial r^2}(0,t). \qquad \text{(B.1)}
$$

Discretise $\partial^2 C/\partial r^2(0)$ with a ghost node $C_{-1}$ at $r = -\Delta r$. By evenness, $C_{-1} = C_1$, so

$$
\frac{\partial^2 C}{\partial r^2}(0) \approx \frac{C_1 - 2C_0 + C_{-1}}{\left(\Delta r\right)^2} = \frac{2\left(C_1 - C_0\right)}{\left(\Delta r\right)^2}.
$$

Here $\Delta r = r_1 - r_0 = r_1$ is the first kernel spacing. Put this into (B.1) with Backward Euler, and write
$\mu_0 = D_{\mathrm{kernel}}\Delta t/\left(\Delta r\right)^2$:

$$
\frac{C_0^{j+1}-C_0^j}{\Delta t} = \frac{6D_{\mathrm{kernel}}}{\left(\Delta r\right)^2}\left(C_1^{j+1}-C_0^{j+1}\right)
\;\Longrightarrow\;
\left(1+6\mu_0\right)C_0^{j+1} - 6\mu_0\,C_1^{j+1} = C_0^j. \qquad \text{(B.2)}
$$

The row-0 coefficients are $b_0 = 1 + 6\mu_0$ and $c_0 = -6\mu_0$. This row also follows from a balance on the small
ball $0 \le r \le \Delta r/2$. The ball's volume is $\frac{4\pi}{3}\left(\Delta r/2\right)^3 = \frac{\pi}{6}\left(\Delta r\right)^3$.
The flow through its surface is approximately
$4\pi\left(\Delta r/2\right)^2 D_{\mathrm{kernel}}\left(C_1 - C_0\right)/\Delta r = \pi D_{\mathrm{kernel}}\Delta r\left(C_1-C_0\right)$.
Dividing the flow by the volume gives $6D_{\mathrm{kernel}}\left(C_1 - C_0\right)/\left(\Delta r\right)^2$, the same row.

**Truncation error of (B.2) (PROVED, for a field that is smooth and even near the centre).** Insert
$C_1 = C(0) + \tfrac12\left(\Delta r\right)^2\tfrac{\partial^2 C}{\partial r^2}(0) + \tfrac1{24}\left(\Delta r\right)^4\tfrac{\partial^4 C}{\partial r^4}(0) + O\!\left(\left(\Delta r\right)^6\right)$:

$$
\frac{6D_{\mathrm{kernel}}\left(C_1 - C(0)\right)}{\left(\Delta r\right)^2} = 3D_{\mathrm{kernel}}\frac{\partial^2 C}{\partial r^2}(0) + \frac{D_{\mathrm{kernel}}\left(\Delta r\right)^2}{4}\frac{\partial^4 C}{\partial r^4}(0) + O\!\left(\left(\Delta r\right)^4\right).
$$

Together with Backward Euler, the residual is $O(\Delta t) + O\!\left(\left(\Delta r\right)^2\right)$.

### 3.2 What the centre value does and does not influence (coordinator note 2)

Row $i = 1$ of (A.3) has $a_1 = -\mu\left(1 - \Delta r/r_1\right) = 0$, because $r_1 = \Delta r$. In the general form
(A.9), $a_1$ contains the factor $r_0 = 0$. Equivalently, $g_{0,\mathrm{outer}} = D_{\mathrm{kernel}}\,r_0 r_1/\Delta r = 0$:
the flow between node 0 and node 1 is identically zero. Row 1 then reads

$$
\frac{C_1^{j+1}-C_1^j}{\Delta t} = \frac{2D_{\mathrm{kernel}}}{\left(\Delta r\right)^2}\left(C_2^{j+1}-C_1^{j+1}\right).
$$

**Proposition B.1 (PROVED).** In the scheme (A.6)–(A.9) with centre row (B.2):

1. The rows $i = 1,\dots,N-1$ do not contain $C_0^{j+1}$. They form a closed tridiagonal system. The centre row then
   gives $C_0^{j+1} = \left(C_0^j + 6\mu_0 C_1^{j+1}\right)/\left(1+6\mu_0\right)$. The coupling runs one way only: from node 1
   to node 0.
2. $C_0$ has no influence on $C_i^{j}$ for any $i \ge 1$ or any $j$. It also has no influence on the inventory $M_h$,
   because $V_0 = 4\pi\left(r_0\right)^2\overline{\Delta r}_0 = 0$, and none on the release $\Phi$ or the cumulative
   release.
3. $C_0$ influences only itself: the reported profile value at $r = 0$, and any diagnostic that includes node 0, such
   as a maximum-norm error.
4. A wrong centre row therefore cannot spoil conservation, the release, or the solution at $i \ge 1$. It spoils
   only $C_0$. For example, dropping the factor 3 in (B.1) gives $-2\mu_0$ instead of $-6\mu_0$. Imposing $C_0 = C_1$
   gives a first-order Neumann row.

*Proof.* Items 1 and 2 follow from $a_1 = 0$ and $V_0 = 0$, by inspection of (A.6) and of the definitions of $M_h$ and
$\Phi$ (Section 5). Item 3 follows from item 1. $\square$

What the wrong centre rows do to $C_0$. Row 0 is a fast relaxation of $C_0$ towards $C_1$, with relaxation time of order
$\left(\Delta r\right)^2/D_{\mathrm{kernel}}$. For a smooth solution, (B.2) gives
$C_1 - C_0 \approx \left(\Delta r\right)^2\,\partial C/\partial t\,/\left(6D_{\mathrm{kernel}}\right)$, which is correct to
leading order. The row with the factor 3 missing gives three times that value. $C_0 = C_1$ gives zero. In both cases
the centre value is still $O\!\left(\left(\Delta r\right)^2\right)$ from the exact value. These rows are inconsistent at
$r = 0$, but their damage is limited to an $O\!\left(\left(\Delta r\right)^2\right)$ error at a node of zero weight.
Section 10.4 shows this: the error at $i \ge 1$ is identical to all printed digits for the three centre rows, and the
centre error converges at second order for all three. (B.2) is still recommended, because it is the only one of the
three that is consistent with (B.1).

**Consistency of the inventory weight.** The pair "face factor $r_0r_1 = 0$, weight $V_0 = 0$" is self-consistent. A
vertex-centred finite-volume method would give node 0 the ball volume $\frac{\pi}{6}\left(\Delta r\right)^3$ and the face
area $4\pi\left(\Delta r/2\right)^2$. If that weight were used in the inventory while the owner's stencil keeps
$g_{0,\mathrm{outer}} = 0$, the balance would not be exact. Node 0 would gain or lose inventory without a matching flow.
The scheme must use one pair consistently. With the owner's stencil, the consistent choice is $V_0 = 0$.

### 3.3 Checks for the owner's handwritten ghost-node treatment

I have not seen the owner's notes. The comparison agent should check the following points against them.

- The interior formula (A.1) must not be applied at $i = 0$ with $r_0 = 0$, because $2/r_0$ is undefined. A correct
  ghost treatment uses the limit (B.1). In the limit the factor is $3$, not $1$.
- The symmetry ghost is $C_{-1} = C_1$. It sits at $r = -\Delta r$, the mirror image of node 1. It is not
  $C_{-1} = C_0$.
- Indexing: if the centre is labelled $i = 1$ instead of $i = 0$, then $r_i = (i-1)\Delta r$. In that labelling
  $1 \pm \Delta r/r_i$ becomes $1 \pm 1/(i-1)$, not $1 \pm 1/i$. Mixing the two labellings is an off-by-one error. It
  makes $a_1 \ne 0$ at the first node off the centre. In this document the centre is $i = 0$.
- The original notes use an insulated outer boundary. The new model replaces it with the absorbing row of Section 5.

---

## 4. (c) The four material interfaces

### 4.1 Setting

The grid has a node at every interface: $r_{I_1} = R_1$ (kernel and buffer), $r_{I_2} = R_2$ (buffer and IPyC),
$r_{I_3} = R_3$ (IPyC and SiC), $r_{I_4} = R_4$ (SiC and OPyC). At a generic interface node $I$, write $D_L$ for the
diffusivity of the inner material and $D_R$ for the outer material. For example, at $I_3$, $D_L = D_{\mathrm{IPyC}}$ and
$D_R = D_{\mathrm{SiC}}$. The spacings are $\Delta r_{I,\mathrm{inner}}$ (in the inner material) and
$\Delta r_{I,\mathrm{outer}}$ (in the outer material), and $r_I = R_m$.

Why the single-material stencil must not be used here. The left and right one-sided slopes differ by the factor
$D_L/D_R$. That ratio is $2.5\times10^{-3}$ at $R_1$ and $500$ at $R_2$ for the synthetic input. A single central
difference cannot represent such a kink. Applying (A.3) with $D_L$ or with $D_R$ ignores the other material's
diffusivity. The resulting flow across the interface is wrong by a factor that does not tend to 1 as
$\Delta r \to 0$.

One-sided Taylor expansions at the node. Write $q = D_L\,\partial C/\partial r(R_m^-) = D_R\,\partial C/\partial r(R_m^+)$
(flux continuity), and $\dot C = \partial C/\partial t(R_m)$, which has one value by (1.2). On each side, (1.1) gives the
second derivative:

$$
\frac{\partial^2 C}{\partial r^2}(R_m^\mp) = \frac{\dot C}{D_{L/R}} - \frac{2}{r_I}\frac{q}{D_{L/R}}. \qquad \text{(C.1)}
$$

Hence, with $h_L = \Delta r_{I,\mathrm{inner}}$ and $h_R = \Delta r_{I,\mathrm{outer}}$ used inside expansions only:

$$
D_R\,\frac{C_{I+1}-C_I}{h_R} = q + \frac{h_R}{2}\left(\dot C - \frac{2q}{r_I}\right) + \frac{\left(h_R\right)^2}{6}D_R\frac{\partial^3 C}{\partial r^3}(R_m^+) + O\!\left(\left(h_R\right)^3\right), \qquad \text{(C.2)}
$$

$$
D_L\,\frac{C_{I-1}-C_I}{h_L} = -q + \frac{h_L}{2}\left(\dot C - \frac{2q}{r_I}\right) - \frac{\left(h_L\right)^2}{6}D_L\frac{\partial^3 C}{\partial r^3}(R_m^-) + O\!\left(\left(h_L\right)^3\right). \qquad \text{(C.3)}
$$

### 4.2 Candidate I: interface node with a flux balance over the two half-intervals (recommended)

Use (A.6)–(A.9) unchanged at $i = I$. The inner flow uses $D_L$ and the outer flow uses $D_R$:

$$
4\pi\left(r_I\right)^2\overline{\Delta r}_I\,\frac{C_I^{j+1}-C_I^j}{\Delta t}
= 4\pi\frac{D_R\,r_I r_{I+1}}{h_R}\left(C_{I+1}^{j+1}-C_I^{j+1}\right) - 4\pi\frac{D_L\,r_{I-1} r_I}{h_L}\left(C_I^{j+1}-C_{I-1}^{j+1}\right). \qquad \text{(C.4)}
$$

Its coefficients are (A.9) with $D_{I,\mathrm{inner}} = D_L$ and $D_{I,\mathrm{outer}} = D_R$.

Derivation as a finite-difference consistency statement. Divide the right side of (C.4) by $4\pi\left(r_I\right)^2$.
Use $r_{I+1}/r_I = 1 + h_R/r_I$ and $r_{I-1}/r_I = 1 - h_L/r_I$, and then (C.2) and (C.3):

$$
\left(1+\frac{h_R}{r_I}\right)\left[q + \frac{h_R}{2}\dot C - \frac{h_R\,q}{r_I}\right] + \left(1-\frac{h_L}{r_I}\right)\left[-q + \frac{h_L}{2}\dot C - \frac{h_L\,q}{r_I}\right] + O\!\left(h^2\right).
$$

Multiply out to first order. The first bracket gives $q + \frac{h_R}{2}\dot C - \frac{h_R q}{r_I} + \frac{h_R q}{r_I} = q + \frac{h_R}{2}\dot C$.
The second gives $-q + \frac{h_L}{2}\dot C - \frac{h_L q}{r_I} + \frac{h_L q}{r_I} = -q + \frac{h_L}{2}\dot C$. The sum is
$\overline{\Delta r}_I\,\dot C + O\!\left(h^2\right)$, where $h = \max(h_L,h_R)$. Two cancellations occur: the flux $q$
cancels between the two sides, which is flux continuity; and the geometric terms $\pm h q/r_I$ cancel against the face
factors $r_I r_{I\pm1}$. Dividing by $\overline{\Delta r}_I$ shows that (C.4) approximates $\partial C/\partial t(R_m)$
with a spatial residual of size $O\!\left(h^2/\overline{\Delta r}_I\right) = O(h)$. The surviving $O\!\left(h^2\right)$
terms contain $\partial^3 C/\partial r^3$ on each side and the second-order Taylor terms. They do not cancel in general.
The local truncation error at an interface node is therefore **first order**. Section 8.7 explains why the global
error is still second order.

Derivation as a balance (the same equation by a second route). Integrate
$\left(r\right)^2\partial C/\partial t = \partial Q/\partial r$ over the two half-intervals
$\left[r_I - h_L/2,\; r_I + h_R/2\right]$. $Q$ is continuous at $R_m$, so the interface flow cancels inside the
interval. What remains is $Q(r_I + h_R/2) - Q(r_I - h_L/2)$. Each of these is a flow inside one material. The face
approximations (A.7) represent them with the material's own $D$. The storage is approximated by
$\left(r_I\right)^2\overline{\Delta r}_I\,\dot C$. This is (C.4). Concentration continuity is exact, because the node
carries one value $C_I$ that both materials share.

Matrix structure. The row has three entries, so the matrix stays tridiagonal. $a_I \le 0$ and $c_I < 0$ for any
spacing, and $b_I = 1 - a_I - c_I$. The flow $P_{I,\mathrm{outer}}$ is the same number in rows $I$ and $I+1$, and
$P_{I-1,\mathrm{outer}}$ is the same number in rows $I-1$ and $I$. The scheme is therefore conservative (Section 8.6).

### 4.3 Candidate II: ghost nodes on both sides

Extend each material's solution across the interface with a fictitious value. $\gamma_L$ is the smooth extension of
the inner material's solution, evaluated at the mirror point $r_I + h_L$. $\gamma_R$ is the extension of the outer
material's solution at $r_I - h_R$. The mirror points coincide with $r_{I+1}$ and $r_{I-1}$ only when $h_L = h_R$.
Apply each material's centred stencil at $I$ on its own symmetric spacing, and impose a centred flux equality. Write $a = C_{I-1}$, $b = C_I$, $c = C_{I+1}$ and $\dot b$
for the time derivative at $I$. To keep the algebra short, take the semi-discrete form; Backward Euler replaces
$\dot b$ by $\left(C_I^{j+1}-C_I^j\right)/\Delta t$ and evaluates everything else at $j+1$.

The left stencil, on the symmetric spacing $h_L$ of the inner material:

$$
\dot b = D_L\left[\frac{\gamma_L - 2b + a}{\left(h_L\right)^2} + \frac{1}{r_I}\frac{\gamma_L - a}{h_L}\right].
$$

The right stencil:

$$
\dot b = D_R\left[\frac{c - 2b + \gamma_R}{\left(h_R\right)^2} + \frac{1}{r_I}\frac{c - \gamma_R}{h_R}\right].
$$

The flux equality, with central one-material derivatives:

$$
q = D_L\,\frac{\gamma_L - a}{2h_L} = D_R\,\frac{c - \gamma_R}{2h_R}.
$$

Elimination. From the flux equality, $\gamma_L = a + 2h_L q/D_L$ and $\gamma_R = c - 2h_R q/D_R$. Insert these into the
two stencils:

$$
\dot b = \frac{2D_L\left(a-b\right)}{\left(h_L\right)^2} + \frac{2q}{h_L} + \frac{2q}{r_I}, \qquad
\dot b = \frac{2D_R\left(c-b\right)}{\left(h_R\right)^2} - \frac{2q}{h_R} + \frac{2q}{r_I}.
$$

Equate the two right sides:

$$
2q\left(\frac{1}{h_L}+\frac{1}{h_R}\right) = \frac{2D_R\left(c-b\right)}{\left(h_R\right)^2} + \frac{2D_L\left(b-a\right)}{\left(h_L\right)^2}
\;\Rightarrow\;
q = \frac{h_Lh_R}{h_L+h_R}\left[\frac{D_R\left(c-b\right)}{\left(h_R\right)^2} + \frac{D_L\left(b-a\right)}{\left(h_L\right)^2}\right].
$$

Put $q$ back into the first stencil. Use $\frac{2q}{h_L} = \frac{2h_R}{h_L+h_R}\left[\dots\right]$ and $h_L + h_R = 2\overline{\Delta r}_I$:

$$
\dot b = \frac{1}{\overline{\Delta r}_I}\left[\frac{D_R\left(c-b\right)}{h_R} - \frac{D_L\left(b-a\right)}{h_L}\right]
+ \frac{1}{r_I\overline{\Delta r}_I}\left[D_R\frac{h_L}{h_R}\left(c-b\right) + D_L\frac{h_R}{h_L}\left(b-a\right)\right]. \qquad \text{(C.5)}
$$

Compare with Candidate I. Divide (C.4) by $4\pi\left(r_I\right)^2\overline{\Delta r}_I$ and use $r_{I\pm1}/r_I = 1 \pm h_{R/L}/r_I$:

$$
\dot b = \frac{1}{\overline{\Delta r}_I}\left[\frac{D_R\left(c-b\right)}{h_R} - \frac{D_L\left(b-a\right)}{h_L}\right]
+ \frac{1}{r_I\overline{\Delta r}_I}\left[D_R\left(c-b\right) + D_L\left(b-a\right)\right]. \qquad \text{(C.6)}
$$

**Result.** When $h_L = h_R$, (C.5) and (C.6) are identical. The two-sided ghost treatment with centred flux matching is
then exactly Candidate I. When $h_L \ne h_R$ they differ: the geometric term carries the weights $h_L/h_R$ and $h_R/h_L$.
Candidate II is then **not conservative**. Multiplied by $V_I$, the coefficient of $\left(c - b\right)$ in row $I$
becomes $4\pi D_R\,r_I\left(r_I + h_L\right)/h_R$. Row $I+1$ uses $4\pi D_R\,r_I\left(r_I + h_R\right)/h_R$ for the same
flow. No choice of nodal weights makes both interfaces telescope in general. Its local truncation error is $O(h)$,
like Candidate I. This is exact on equal spacing, where it is Candidate I. For unequal spacing the reasoning is as
follows, but I have not written it out in full: each one-material stencil is second-order accurate for its smooth
extension, the eliminated ghost values differ from the true extensions by $O\!\left(h^3\right)$, and dividing by
$\left(h\right)^2$ leaves $O(h)$. Its
off-diagonal coefficient of $a$ is $\frac{D_L}{h_L\overline{\Delta r}_I}\left(1 - \frac{h_R}{r_I}\right)$, which is
non-positive only if $h_R \le r_I$. That holds on any reasonable grid.

### 4.4 Candidate III: algebraic interface row with one-sided second-order differences

Give the interface node no storage. Impose flux continuity with second-order one-sided differences:

$$
D_L\,\frac{3C_I - 4C_{I-1} + C_{I-2}}{2h_L} = D_R\,\frac{-3C_I + 4C_{I+1} - C_{I+2}}{2h_R}. \qquad \text{(C.7)}
$$

Rearranged as a matrix row:

$$
\frac{D_L}{2h_L}C_{I-2} - \frac{2D_L}{h_L}C_{I-1} + \left(\frac{3D_L}{2h_L}+\frac{3D_R}{2h_R}\right)C_I - \frac{2D_R}{h_R}C_{I+1} + \frac{D_R}{2h_R}C_{I+2} = 0.
$$

The neighbouring rows $I\pm1$ keep the interior scheme. Properties:

- *Accuracy:* the flux condition is second-order accurate.
- *Structure:* the row has five nonzero entries, so the matrix is no longer tridiagonal. The extra entries
  $C_{I\pm2}$ can be eliminated with rows $I\pm1$, but then the coefficients depend on $\Delta t$ and the sign pattern
  is lost.
- *Sign pattern:* the entries of $C_{I\pm2}$ are **positive**, so the matrix is not a Z-matrix. Row $I$ is not
  diagonally dominant: the sum of the off-diagonal magnitudes is $\tfrac52\left(D_L/h_L + D_R/h_R\right)$, against the
  diagonal $\tfrac32\left(D_L/h_L + D_R/h_R\right)$. The M-matrix arguments of Section 8 fail, and positivity is not
  guaranteed.
- *Conservation:* the interface node has zero weight. The flows that rows $I-1$ and $I+1$ use at their interface-side
  faces, $P_{I-1,\mathrm{outer}}$ and $P_{I,\mathrm{outer}}$, are not equal, and (C.7) does not make them equal. Their
  difference is approximately the storage of the two half-intervals, which the scheme omits. The inventory drifts by
  an amount that tends to zero as $h \to 0$ but is not zero (Section 10.3).

### 4.5 Candidate IV: interface midway between two nodes (series conductance)

Place no node at $R_m$. Put $R_m$ inside the interval $(r_i, r_{i+1})$. The flow through that interval is set by two
resistances in series, $(R_m - r_i)/D_L$ and $(r_{i+1}-R_m)/D_R$:

$$
g_{i,\mathrm{outer}} = \frac{r_i\,r_{i+1}}{\dfrac{R_m - r_i}{D_L} + \dfrac{r_{i+1}-R_m}{D_R}}.
$$

When $R_m$ is the midpoint of the interval, this is the harmonic mean of $D_L$ and $D_R$ times $r_ir_{i+1}/\Delta r$.
The scheme is tridiagonal, conservative and a Z-matrix. Its drawbacks for STRATO:

- $C(R_m)$ is not an unknown and has to be reconstructed.
- The perfect-contact condition is enforced only implicitly.
- The grid must place each $R_m$ between two nodes and still have nodes at $r = 0$ and $r = R_5$.
- The method is close to the retired cell-based finite-volume method, which the directive asks to move away from.

Its advantage is that the jump of the initial profile falls between nodes (compare Section 6).

### 4.6 Comparison and recommendation

| Property | I: interface node, half-interval balance | II: two-sided ghost | III: algebraic one-sided | IV: interface between nodes |
|---|---|---|---|---|
| Local truncation error at the interface node | $O(h)$ (Section 4.2) | $O(h)$ | flux condition $O(h^2)$, no storage | interface not a node |
| Observed global order, five-layer $\Lambda_1$ (Section 10.3) | 2.00 | 2.00 | 2.00 | 2.0 |
| Exact discrete conservation | yes (PROVED, Section 8.6) | only if $h_L = h_R$ (then identical to I) | no | yes |
| Observed inventory drift (Section 10.3) | rounding only, $\sim10^{-11}$ | $1.2\times10^{-3}$ ($n=20$) | $3.0\times10^{-2}$ ($n=20$) | not run |
| Matrix | tridiagonal, symmetric after scaling | tridiagonal, not symmetric | five entries in interface rows | tridiagonal, symmetric after scaling |
| Z-matrix / M-matrix | yes, for every grid (Section 8.2) | yes if $h_R \le r_I$; not symmetric | no (positive entries) | yes |
| Interface value $C(R_m)$ | the unknown $C_I$ | the unknown $C_I$ | the unknown $C_I$ | reconstructed |
| Same formula as the interior rows | yes | no | no | yes |

**Recommendation: Candidate I.** It is the only nodal candidate that has all of the following at once: exact discrete
conservation, a tridiagonal M-matrix on every grid, concentration continuity built in, and the interior formula itself
as its formula. The $O(h)$ local error at four nodes does not lower the global order (Section 8.7; observed in
Section 10.3). The coordinator's note (3) is confirmed, with one refinement: on equal spacing the two-sided ghost
treatment is not an alternative, because it is the same scheme.

### 4.7 The interface flow seen from both sides

**Proposition C.1 (PROVED).** For Candidate I, define the interface flow reconstructed from the left and from the right
at step $j+1$:

$$
\mathcal Q_{I,L}^{j+1} = P_{I-1,\mathrm{outer}}^{j+1} + 4\pi\left(r_I\right)^2\frac{\Delta r_{I,\mathrm{inner}}}{2}\,\frac{C_I^{j+1}-C_I^j}{\Delta t},
\qquad
\mathcal Q_{I,R}^{j+1} = P_{I,\mathrm{outer}}^{j+1} - 4\pi\left(r_I\right)^2\frac{\Delta r_{I,\mathrm{outer}}}{2}\,\frac{C_I^{j+1}-C_I^j}{\Delta t}.
$$

Then $\mathcal Q_{I,L}^{j+1} = \mathcal Q_{I,R}^{j+1}$ exactly.

*Proof.* $\mathcal Q_{I,L} - \mathcal Q_{I,R} = P_{I-1,\mathrm{outer}} - P_{I,\mathrm{outer}} + V_I\left(C_I^{j+1}-C_I^j\right)/\Delta t$,
which is zero by row (C.4). $\square$

Each of the two quantities approximates the true inward flow $4\pi Q(R_m)$ to $O\!\left(h^2\right) + O(\Delta t)$. The
reason: $P_{I-1,\mathrm{outer}}$ approximates $4\pi Q$ at the midpoint $r_I - h_L/2$ to $O\!\left(h^2\right)$, and
$Q(R_m) = Q(r_I - h_L/2) + \frac{h_L}{2}\left(r_I\right)^2\dot C + O\!\left(h^2\right)$ because
$\partial Q/\partial r = \left(r\right)^2\partial C/\partial t$. The radial flux density at the interface is
$J_I = -\mathcal Q_{I,L}/\left(4\pi\left(r_I\right)^2\right)$ (unit $\mathrm{mol\ m^{-2}\ s^{-1}}$, positive outwards).
Section 10.2 gives the observed two-sided agreement. Phase D (test E) can use exactly this quantity.

---

## 5. (d) Absorbing outer boundary, release rate and cumulative release

### 5.1 The matrix row

Node $N$ sits at $r_N = R_5$, and $C_N^{j} = 0$ for every $j \ge 1$. The unknowns are $C_0,\dots,C_{N-1}$, which makes
$N$ unknowns. In row $N-1$ the term $c_{N-1}C_N^{j+1}$ is zero, so it is removed:

$$
a_{N-1}C_{N-2}^{j+1} + b_{N-1}C_{N-1}^{j+1} = C_{N-1}^j, \qquad b_{N-1} = 1 - a_{N-1} - c_{N-1}. \qquad \text{(D.1)}
$$

$c_{N-1}$ is computed from (A.9) with $D_{N-1,\mathrm{outer}} = D_{\mathrm{OPyC}}$ and stays in $b_{N-1}$, but it has no
matrix entry. On a uniform OPyC grid: $a_{N-1} = -\mu\left(1 - \Delta r/r_{N-1}\right)$ and $b_{N-1} = 1 + 2\mu$. Row
$N-1$ is strictly diagonally dominant by $1 + |c_{N-1}|$.

### 5.2 The release rate

The release rate is the outward flow through the last connection:

$$
\Phi^{j+1} = -P_{N-1,\mathrm{outer}}^{j+1} = 4\pi\,g_{N-1,\mathrm{outer}}\left(C_{N-1}^{j+1} - C_N^{j+1}\right) = 4\pi\,\frac{D_{\mathrm{OPyC}}\,r_{N-1}\,R_5}{\Delta r_{N-1,\mathrm{outer}}}\,C_{N-1}^{j+1}. \qquad \text{(D.2)}
$$

This definition is not a choice made for accuracy. It is the only definition for which the discrete balance (D.4)
below is exact.

**Accuracy (PROVED for a solution that is smooth in the OPyC near $R_5$, without source).** Write $h = \Delta r_{N-1,\mathrm{outer}}$
and expand around $R_5$ using $C(R_5) = 0$:
$C_{N-1} = -h\,\tfrac{\partial C}{\partial r} + \tfrac{\left(h\right)^2}{2}\tfrac{\partial^2 C}{\partial r^2} + O\!\left(\left(h\right)^3\right)$.
Then

$$
\frac{\Phi}{4\pi D_{\mathrm{OPyC}}} = R_5\left(R_5 - h\right)\left[-\frac{\partial C}{\partial r} + \frac{h}{2}\frac{\partial^2 C}{\partial r^2}\right] + O\!\left(\left(h\right)^2\right)
= \left(R_5\right)^2\left[-\frac{\partial C}{\partial r} + h\left(\frac12\frac{\partial^2 C}{\partial r^2} + \frac{1}{R_5}\frac{\partial C}{\partial r}\right)\right] + O\!\left(\left(h\right)^2\right).
$$

$C(R_5,t) = 0$ for all $t$, so $\partial C/\partial t(R_5) = 0$. Eq. (1.1) at $R_5$ then gives
$\partial^2 C/\partial r^2 = -\frac{2}{R_5}\partial C/\partial r$, and the $O(h)$ bracket vanishes. Therefore
$\Phi = \Phi_{\mathrm{exact}} + O\!\left(\left(h\right)^2\right)$, plus the $O(\Delta t)$ of the time level. With a
source that is nonzero at $R_5$ this cancellation fails. Section 9 gives the correction.

### 5.3 Inventory, cumulative release and the exact balance

$$
M_h^j = \sum_{i=1}^{N-1} V_i\,C_i^j, \qquad
M_{\mathrm{rel}}^j = \Delta t\sum_{k=1}^{j}\Phi^{k}, \quad M_{\mathrm{rel}}^0 = 0. \qquad \text{(D.3)}
$$

The sum may start at $i = 0$ or end at $i = N$ without change, because $V_0 = 0$ and $C_N = 0$.

**Proposition D.1 (PROVED, exact arithmetic).** For every $j \ge 0$,

$$
M_h^{j+1} + \Delta t\,\Phi^{j+1} = M_h^{j}, \qquad\text{hence}\qquad M_h^{j} + M_{\mathrm{rel}}^{j} = M_h^{0}. \qquad \text{(D.4)}
$$

*Proof.* Sum (A.6) over $i = 1,\dots,N-1$:
$\sum_i V_i\left(C_i^{j+1}-C_i^j\right) = \Delta t\sum_{i=1}^{N-1}\left(P_{i,\mathrm{outer}} - P_{i-1,\mathrm{outer}}\right)$.
The right side telescopes to $\Delta t\left(P_{N-1,\mathrm{outer}} - P_{0,\mathrm{outer}}\right)$.
$P_{0,\mathrm{outer}} = 0$ because $g_{0,\mathrm{outer}} = 0$, and $P_{N-1,\mathrm{outer}} = -\Phi^{j+1}$ by (D.2). So
$M_h^{j+1} - M_h^j = -\Delta t\,\Phi^{j+1}$. Summing over the steps gives the second identity. $\square$

Note that (D.4) uses neither the centre row nor the value of $C_0$ (Proposition B.1). The release fraction is
$F^j = M_{\mathrm{rel}}^j/M_h^0 = 1 - M_h^j/M_h^0$. The two forms are equal to rounding. With the start of Section 6,
$M_h^0$ equals the exact $M^0 = \frac{4\pi}{3}\left(R_1\right)^3 C_0$, so the denominator carries no discretisation
error.

The discrete inventory is a quadrature of $4\pi\int_0^{R_5}\left(r\right)^2C\,dr$. Its weights $V_i$ are those of the
composite trapezoidal rule on the nodes: the end values $\left(r_0\right)^2C_0 = 0$ and $C_N = 0$ vanish, and interior
nodes get the mean of the adjacent spacings. The rule is applied to a function that is smooth on every interval,
because the interfaces are nodes. It is therefore second-order accurate for $t > 0$.

---

## 6. (e) The initial condition on the grid

At $t = 0$: $C_i^0 = C_0$ for $r_i < R_1$, $C_i^0 = 0$ for $r_i > R_1$, and $C_{I_1}^0 = \theta\,C_0$ at the kernel
surface node. $\theta$ is to be chosen. The continuous profile has no value at $R_1$ (a set of zero volume), so $\theta$
is a discretisation choice.

**Inventory-matched value (coordinator note 5).** Take a uniform kernel spacing $\Delta r_{I_1,\mathrm{inner}} = R_1/K$,
with $K$ kernel intervals. Require $M_h^0 = M^0$, that is, $4\pi$ times

$$
\sum_{i=1}^{K-1}\left(r_i\right)^2\Delta r_{I_1,\mathrm{inner}}\,C_0 + \left(R_1\right)^2\overline{\Delta r}_{I_1}\,\theta C_0 = \frac{\left(R_1\right)^3}{3}C_0.
$$

Step 1. Use $r_i = i\,\Delta r_{I_1,\mathrm{inner}}$ and $\sum_{i=1}^{K-1} i^2 = \frac{(K-1)K(2K-1)}{6}$. The first sum is
$\left(\Delta r_{I_1,\mathrm{inner}}\right)^3\frac{2K^3 - 3K^2 + K}{6}C_0$.

Step 2. The right side is $\left(\Delta r_{I_1,\mathrm{inner}}\right)^3\frac{K^3}{3}C_0 = \left(\Delta r_{I_1,\mathrm{inner}}\right)^3\frac{2K^3}{6}C_0$.
The difference, which node $I_1$ must supply, is $\left(\Delta r_{I_1,\mathrm{inner}}\right)^3\frac{3K^2 - K}{6}C_0$.

Step 3. Divide by $\left(R_1\right)^2\overline{\Delta r}_{I_1}C_0 = K^2\left(\Delta r_{I_1,\mathrm{inner}}\right)^2\overline{\Delta r}_{I_1}C_0$:

$$
\theta = \frac{\Delta r_{I_1,\mathrm{inner}}\left(3K-1\right)}{6K\,\overline{\Delta r}_{I_1}}
= \frac{\Delta r_{I_1,\mathrm{inner}}}{\Delta r_{I_1,\mathrm{inner}} + \Delta r_{I_1,\mathrm{outer}}}\left(1 - \frac{1}{3K}\right). \qquad \text{(E.1)}
$$

With equal spacing on both sides, $\theta = \tfrac12 - \tfrac{1}{6K}$. For the synthetic grid with 40 intervals per layer
($\Delta r_{I_1,\mathrm{inner}} = 5\ \mu\mathrm m$, $\Delta r_{I_1,\mathrm{outer}} = 2.5\ \mu\mathrm m$, $K = 40$),
$\theta = \tfrac23\left(1 - \tfrac{1}{120}\right) = 0.66111\ldots$. Always $0 < \theta < 1$, so the start is
non-negative and bounded by $C_0$.

Interpretation. The factor $\Delta r_{I_1,\mathrm{inner}}/(\Delta r_{I_1,\mathrm{inner}} + \Delta r_{I_1,\mathrm{outer}})$
is the kernel's share of the node's two half-intervals. It is the value that a "half-interval average" of the start
would give. The factor $1 - 1/(3K)$ corrects the trapezoidal-rule error of the kernel sum, so that the discrete
initial inventory is exact.

Alternatives:

- $\theta = \tfrac12$ (equal spacing): the mean of the two one-sided limits. The initial inventory is then too large by
  the relative amount $1/\left(2K^2\right)$.
- $\theta = 1$ or $\theta = 0$: the node is treated as kernel or as buffer. The relative error is then
  $O(1/K)$.

**Effect on the observed order (OBSERVED, Section 10.5; a proof for discontinuous data is NOT given).**

- With $\theta = \tfrac12$ or the matched value (E.1), the release fraction and the weighted $L^2$ profile error converge
  at second order in space at every fixed time tested, from $t = 4\times10^2\ \mathrm{s}$ to $4\times10^5\ \mathrm{s}$
  for $D = 10^{-12}\ \mathrm{m^2\ s^{-1}}$ and $R_1 = 200\ \mu\mathrm m$. The error constant grows as $t$ decreases.
  The matched value gives a smaller error in $F$ than $\tfrac12$, by a factor of 4 to 30 at these times.
- With $\theta = 1$ or $\theta = 0$ the order is **one**, at all the times tested. The $O(1/K)$ error in the initial
  inventory is never forgotten: the scheme conserves exactly the wrong inventory.
- The pointwise maximum error is pre-asymptotic while the spacing exceeds the diffusion length
  $\sqrt{D_{\mathrm{kernel}}\,t}$. At $t = 40\ \mathrm{s}$ the observed order rises from 0.5 to 2.0 as $K$ goes from 8
  to 256.
- Backward Euler stays first order in time for the discontinuous start. Its error constant is larger at early times,
  relative to the size of the solution's change.

For non-smooth initial data there is a classical theory of parabolic discretisations, mainly for finite-element
methods, in which the errors at time $t$ behave like $\left(\Delta r\right)^2/t$ and $\Delta t/t$. I have not proved the
corresponding statement for this scheme. The observations above agree with it qualitatively: the order stays the same
and the constant grows at early times.

---

## 7. (f) Uniform, piecewise-uniform and graded grids

**Uniform grid, $r_i = i\,\Delta r$.** This needs every $R_m$ to be an integer multiple of $\Delta r$.

- Synthetic radii (200, 300, 350, 400, 450 µm): the largest common step is 50 µm, so $N$ must be a multiple of 9.
- AGR-1 baseline radii of `data/geometry/geometry.csv`, rows G-01 to G-05 (175, 278.5, 317.9, 353.2, 394.2 µm): the
  largest common step is 0.1 µm, so $N = 3942\,k$. This is affordable for a tridiagonal solve. It wastes resolution
  in the slow layers, and it does not let the resolution follow the very different diffusion lengths of the five
  materials.

**Piecewise-uniform grid (recommended).** Each material gets $n_{\mathrm{kernel}}, \dots, n_{\mathrm{OPyC}}$ intervals of
equal length. The interfaces are nodes. The spacing changes only at interface nodes. Coefficients: (A.9). Inside a
material they reduce to (A.3) with that material's $\Delta r$ and $\mu$. At an interface node they are (C.4). The
spacing jump falls on the nodes that already have an $O(h)$ local error from the kink. It therefore adds no new kind
of error, and the proof outline of Section 8.7 covers it.

**Graded grid inside a material.** A smooth map $r = \mathcal R(x)$ on uniform $x$ makes
$\Delta r_{i,\mathrm{outer}} - \Delta r_{i,\mathrm{inner}} = O\!\left(h^2\right)$. Section 8.1 shows that the local error
stays $O\!\left(h^2\right)$ in that case. Use (A.9), never (A.3), on such grids.

The common non-conservative alternative is the three-point Lagrange formula. With $h_- = \Delta r_{i,\mathrm{inner}}$
and $h_+ = \Delta r_{i,\mathrm{outer}}$:

$$
\frac{\partial^2 C}{\partial r^2} \approx \frac{2\left[h_-\,C_{i+1} - \left(h_-+h_+\right)C_i + h_+\,C_{i-1}\right]}{h_-h_+\left(h_-+h_+\right)},\qquad
\frac{\partial C}{\partial r} \approx \frac{\left(h_-\right)^2C_{i+1} + \left[\left(h_+\right)^2 - \left(h_-\right)^2\right]C_i - \left(h_+\right)^2C_{i-1}}{h_-h_+\left(h_-+h_+\right)}.
$$

This formula has a second-order local error on any smooth grid. It does not telescope, so it loses (D.4). It is not
recommended.

When grading pays. Two regions may need finer spacing: near $R_1$ at early times, where the start profile jumps and
the diffusion length is $\sqrt{D_{\mathrm{kernel}}t}$; and inside thin, slow layers, where the profile is steep. The
SiC slope is 40 times the IPyC slope at $R_3$ in the synthetic case. A piecewise-uniform grid already lets each layer
have its own $\Delta r$. Smooth grading inside a layer is optional. My recommendation is to support only piecewise-uniform
grids in the first production version and to choose $n$ per layer from the input. This is a recommendation, not a
proven optimum.

Observed: a smoothly graded grid and even a grid that alternates spacings $h, 2h$ at every node both keep second order
for the homogeneous eigenvalue (Section 10.6). The alternating grid has an $O(h)$ local error at every node, so this is
an instance of the superconvergence of conservative three-point schemes. I do not prove it here.

---

## 8. (g) Mathematical properties

The scheme is fully specified by: (A.6)–(A.9) for $i = 1,\dots,N-1$; the centre row (B.2); the boundary row (D.1);
$C_N = 0$; the release (D.2); the start of Section 6; and a grid with nodes at $0, R_1, \dots, R_5$ and all
$D_{\cdot} > 0$, $\Delta t > 0$. Write the system as $A\,\mathbf C^{j+1} = \mathbf C^j$ with
$\mathbf C = \left(C_0,\dots,C_{N-1}\right)$.

### 8.1 Consistency and local truncation error

Define the residual $\tau_i^{j+1}$ as the result of inserting the exact solution into row $i$, divided so that the row
reads "time difference minus spatial operator". With
$\frac{C(t_{j+1})-C(t_j)}{\Delta t} = \frac{\partial C}{\partial t}(t_{j+1}) - \frac{\Delta t}{2}\frac{\partial^2 C}{\partial t^2} + O\!\left(\left(\Delta t\right)^2\right)$,
the time part of every row contributes $-\frac{\Delta t}{2}\frac{\partial^2 C}{\partial t^2}$.

**Proposition G.1 (PROVED, for a solution that is $C^4$ in $r$ on each closed layer and $C^2$ in $t$).**

| Node | Spatial residual | Order |
|---|---|---|
| interior, uniform spacing | $-D\left(\Delta r\right)^2\left[\tfrac1{12}\tfrac{\partial^4 C}{\partial r^4} + \tfrac{1}{3r_i}\tfrac{\partial^3 C}{\partial r^3}\right] + O\!\left(\left(\Delta r\right)^4\right)$ | 2 |
| interior, $i = 1$ (uniform kernel) | $-\tfrac{5}{12}D_{\mathrm{kernel}}\left(\Delta r\right)^2\tfrac{\partial^4 C}{\partial r^4}(0) + O\!\left(\left(\Delta r\right)^4\right)$ | 2 |
| centre, $i = 0$ | $-\tfrac14 D_{\mathrm{kernel}}\left(\Delta r\right)^2\tfrac{\partial^4 C}{\partial r^4}(0)$ | 2 |
| interior, unequal spacing (one material) | $O\!\left(\lvert\Delta r_{i,\mathrm{outer}} - \Delta r_{i,\mathrm{inner}}\rvert\right) + O\!\left(h^2\right)$ | 1, or 2 on a smoothly graded grid |
| interface node (Candidate I) | $O(h)$, from the $O(h^2)$ remainder of Section 4.2 divided by $\overline{\Delta r}_I$ | 1 |
| boundary row $N-1$ | as interior; $C_N = 0$ is exact | 2 |
| release (D.2), as an approximation of $\Phi_{\mathrm{exact}}$ | $O\!\left(h^2\right)$ (Section 5.2) | 2 |

Derivations.

*Interior, uniform.* The central differences give
$\frac{C_{i+1}-2C_i+C_{i-1}}{\left(\Delta r\right)^2} = \frac{\partial^2 C}{\partial r^2} + \frac{\left(\Delta r\right)^2}{12}\frac{\partial^4 C}{\partial r^4} + \dots$
and $\frac{C_{i+1}-C_{i-1}}{2\Delta r} = \frac{\partial C}{\partial r} + \frac{\left(\Delta r\right)^2}{6}\frac{\partial^3 C}{\partial r^3} + \dots$
Multiply the second by $2/r_i$ and the sum by $D$. Near the centre, $\partial^3 C/\partial r^3 = r\,\partial^4 C/\partial r^4(0) + O\!\left(\left(r\right)^3\right)$
for an even field, so $\frac{1}{3r_i}\frac{\partial^3 C}{\partial r^3}$ stays bounded. The residual is uniformly
$O\!\left(\left(\Delta r\right)^2\right)$ up to $i = 1$.

*Node 1.* Use $C(r) = C(0) + \tfrac12\left(r\right)^2 C_2 + \tfrac1{24}\left(r\right)^4 C_4$, with
$C_2 = \partial^2 C/\partial r^2(0)$ and $C_4 = \partial^4 C/\partial r^4(0)$. The scheme gives
$2D\left(C(2\Delta r)-C(\Delta r)\right)/\left(\Delta r\right)^2 = 3DC_2 + \tfrac{15}{12}D\left(\Delta r\right)^2C_4$.
The operator at $r = \Delta r$ is $C'' + 2C'/r = 3C_2 + \tfrac56\left(\Delta r\right)^2C_4$. The difference is
$\left(\tfrac{15}{12}-\tfrac{10}{12}\right) = \tfrac5{12}$. This agrees with the general formula, since
$\tfrac1{12} + \tfrac13 = \tfrac5{12}$.

*Unequal spacing.* Write $m_\pm$ for the midpoints of the two adjacent intervals and $Q = \left(r\right)^2D\,\partial C/\partial r$.
From $r_ir_{i+1} = \left(m_+\right)^2 - \tfrac14\left(h_+\right)^2$ and
$\frac{C_{i+1}-C_i}{h_+} = \frac{\partial C}{\partial r}(m_+) + \frac{\left(h_+\right)^2}{24}\frac{\partial^3 C}{\partial r^3}(m_+) + \dots$,
the flow $P_{i,\mathrm{outer}}/(4\pi) = Q(m_+) + O\!\left(\left(h_+\right)^2\right)$. The difference of the two flows is
$Q(m_+) - Q(m_-) = \overline{\Delta r}_i\,\partial Q/\partial r(r_i) + \tfrac18\left[\left(h_+\right)^2 - \left(h_-\right)^2\right]\partial^2 Q/\partial r^2(r_i) + O\!\left(h^3\right)$.
The $O\!\left(h^2\right)$ flow errors also differ by $O\!\left(h\,\lvert h_+-h_-\rvert\right) + O\!\left(h^3\right)$. Divide
by $V_i/(4\pi) = \left(r_i\right)^2\overline{\Delta r}_i$ and use $\partial Q/\partial r = \left(r\right)^2\partial C/\partial t$.
The residual is $O\!\left(\lvert h_+-h_-\rvert\right) + O\!\left(h^2\right)$.

*Interface node.* See Section 4.2.

Consistency. Every residual tends to zero as $\Delta r, \Delta t \to 0$, so the scheme is consistent. In time it is
first order everywhere. The $O(h)$ residuals are confined to the four interface nodes, which on a piecewise-uniform grid
are also the only places where the spacing changes.

### 8.2 Solvability (coordinator note 4)

**Proposition G.2 (PROVED).** For every grid $0 = r_0 < r_1 < \dots < r_N = R_5$ with the interfaces at nodes, every
choice of positive diffusivities and every $\Delta t > 0$, the matrix $A$ satisfies:

1. every off-diagonal entry is $\le 0$, so $A$ is a Z-matrix;
2. every row is strictly diagonally dominant, with $b_i - |a_i| - |c_i| \ge 1$;
3. $A$ is a non-singular M-matrix, and $A^{-1} \ge 0$ entry by entry;
4. the Thomas algorithm runs without pivoting, and every pivot satisfies $\beta_i > |c_i|$.

*Proof.*

1. In (A.9), every factor of $a_i$ and $c_i$ is non-negative ($\Delta t$, $D$, $r_{i\pm1} \ge 0$, the spacings), and there
   is a leading minus sign. In row 0, $c_0 = -6\mu_0 < 0$.
2. Interior rows: $b_i = 1 + |a_i| + |c_i|$. Row $N-1$: $b_{N-1} = 1 + |a_{N-1}| + |c_{N-1}|$, but its only off-diagonal
   entry is $a_{N-1}$. Row 0: $b_0 = 1 + |c_0|$. The margin is $\ge 1$ in every row.
3. Write $A = \mathcal D\left(\mathbb I - B\right)$, where $\mathcal D = \mathrm{diag}(b_i) > 0$ and
   $B = \mathbb I - \mathcal D^{-1}A$. $B$ has zero diagonal and off-diagonal entries $-a_i/b_i \ge 0$ and $-c_i/b_i \ge 0$.
   Its row sums are $\left(|a_i|+|c_i|\right)/b_i < 1$, so $\|B\|_\infty < 1$. The Neumann series
   $\left(\mathbb I - B\right)^{-1} = \sum_{k\ge0}B^k$ converges and is entrywise $\ge 0$. Hence
   $A^{-1} = \left(\mathbb I - B\right)^{-1}\mathcal D^{-1} \ge 0$ exists.
4. Thomas forward sweep: $\beta_0 = b_0$, and $\beta_i = b_i - a_i c_{i-1}/\beta_{i-1}$. Claim: $\beta_i \ge 1 + |c_i|$.
   For $i = 0$, $\beta_0 = 1 + |c_0|$. Assume the claim for $i-1$. Then
   $\lvert a_ic_{i-1}/\beta_{i-1}\rvert \le |a_i|\,|c_{i-1}|/\left(1+|c_{i-1}|\right) < |a_i|$, so
   $\beta_i > b_i - |a_i| \ge 1 + |c_i|$. Every multiplier satisfies $|c_i/\beta_i| < 1$, so the elimination is
   stable. $\square$

The coordinator's note (4) is confirmed for the scheme as specified. Its grid condition turns out to be empty: in the
flux form, the sign condition $r_{i-1} \ge 0$ holds on every grid that starts at $r = 0$. In the owner's uniform form
(A.3), the same condition reads $1 - \Delta r/r_i \ge 0$, that is $r_i \ge \Delta r$, which is the same fact. The
conditions fail for Candidate III, and for Candidate II if $h_R > r_I$.

### 8.3 Stability

**Proposition G.3 (PROVED): maximum norm.** $\|A^{-1}\|_\infty \le 1$, so
$\|\mathbf C^{j+1}\|_\infty \le \|\mathbf C^j\|_\infty$ for every $\Delta t > 0$. The scheme is unconditionally stable in
the maximum norm.

*Proof.* Let $A\mathbf x = \mathbf y$ and choose $k$ with $|x_k| = \|\mathbf x\|_\infty$. Then
$|y_k| \ge b_k|x_k| - |a_k||x_{k-1}| - |c_k||x_{k+1}| \ge \left(b_k - |a_k| - |c_k|\right)|x_k| \ge |x_k|$. $\square$

**Proposition G.4 (PROVED): weighted energy.** For $i \ge 1$ the sequence
$\|\mathbf C^j\|_V^2 = \sum_{i=1}^{N-1}V_i\left(C_i^j\right)^2$ does not increase.

*Proof.* Multiply (A.6) by $C_i^{j+1}$ and sum over $i$. Summation by parts uses $P_{0,\mathrm{outer}} = 0$ and $C_N = 0$:

$$
\sum_i V_i\left(C_i^{j+1}-C_i^j\right)C_i^{j+1} = -\Delta t\sum_{i=0}^{N-1}4\pi g_{i,\mathrm{outer}}\left(C_{i+1}^{j+1}-C_i^{j+1}\right)^2 \le 0.
$$

The $i = N-1$ term of the sum is $4\pi g_{N-1,\mathrm{outer}}\left(C_{N-1}^{j+1}\right)^2$. Next, use
$\left(x-y\right)x = \tfrac12\left(x^2 - y^2\right) + \tfrac12\left(x-y\right)^2 \ge \tfrac12\left(x^2-y^2\right)$. $\square$

### 8.4 Positivity and the discrete maximum principle

**Proposition G.5 (PROVED).** If $0 \le C_i^j \le C_{\max}$ for all $i$, then $0 \le C_i^{j+1} \le C_{\max}$, for every
$\Delta t > 0$ and every admissible grid.

*Proof.* $\mathbf C^{j+1} = A^{-1}\mathbf C^j \ge 0$ because $A^{-1} \ge 0$. Every row of $A$ sums to at least 1 (it
sums to exactly 1, except row $N-1$, which sums to $1 + |c_{N-1}|$), so $A\mathbf 1 \ge \mathbf 1$. Then
$\mathbf 1 - A^{-1}\mathbf 1 = A^{-1}\left(A\mathbf 1 - \mathbf 1\right) \ge 0$. Hence
$\mathbf C^{j+1} \le C_{\max}A^{-1}\mathbf 1 \le C_{\max}\mathbf 1$. $\square$

**Proposition G.6 (PROVED, IEEE floating point, given $\beta_i > 0$ as computed).** If the Thomas algorithm is coded as
$\beta_i = b_i - (a_i/\beta_{i-1})c_{i-1}$, $d'_i = d_i - (a_i/\beta_{i-1})d'_{i-1}$ and
$x_i = (d'_i - c_ix_{i+1})/\beta_i$, with $a_i, c_i \le 0$ stored as such, then a right side $d \ge 0$ gives a
computed solution $x \ge 0$, including in rounded arithmetic.

*Proof.* $a_i/\beta_{i-1} \le 0$ and $d'_{i-1} \ge 0$, so the subtracted product is $\le 0$, and $d'_i$ is a sum of
non-negative numbers. Likewise $-c_ix_{i+1} \ge 0$. Every operation acts on non-negative operands, and rounding a
non-negative exact result gives a non-negative number. $\square$

The positivity is unconditional only for this scheme, in which Candidate I is used at the interfaces. Candidate III
gives up the proof. For a solver that is not the Thomas algorithm, positivity under rounding is not claimed. The
discrete maximum principle and positivity also hold for the centre value, since row 0 is part of $A$. Positivity of the
exact continuous solution is shown in the repository's v1.0.0 derivation (Section 3.5.5) and is not used here.

### 8.5 Interface continuity

Concentration continuity is exact, because there is one nodal value at each interface. Flux continuity holds exactly in
the sense of Proposition C.1, and to $O\!\left(h^2 + \Delta t\right)$ against the true flow.

### 8.6 Discrete inventory conservation

**Proposition G.7 (PROVED, exact arithmetic).** Identity (D.4): $M_h^j + M_{\mathrm{rel}}^j = M_h^0$ for every $j$. This
holds for any grid, any $\Delta t$, any positive diffusivities, and any centre row.

Requirements, stated precisely: the same $g_{i,\mathrm{outer}}$ is used in rows $i$ and $i+1$, in $\Phi$ and nowhere
else; $V_i$ is the weight by which row $i$ was divided; $V_0 = 0$ and $g_{0,\mathrm{outer}} = 0$ belong together;
the release is taken at the same time level as the implicit flows ($j+1$).

In floating point, $\varepsilon^j = M_h^j + M_{\mathrm{rel}}^j - M_h^0$ is not zero. It is a rounding residual. It
grows with the largest mesh ratio and with the number of steps, because each solve has a backward error proportional
to $u\,b_i$, where $u = 2^{-53}$. Observed (Section 10.1): $9.4\times10^{-11}$ relative in the divided form,
$4.3\times10^{-12}$ in the multiplied-out form of Section 2.3, and $3.3\times10^{-13}$ with a 100 times smaller mesh
ratio. **No exact conservation in floating point is claimed.** A test tolerance should be derived from the mesh ratio
and the step count, not chosen to pass.

### 8.7 Convergence: spatial and temporal order

**Proposition G.8 (PROVED): homogeneous sphere, maximum norm.** For one material, a uniform grid, and a solution that is
$C^4$ in $r$ and $C^2$ in $t$ on $[0,T]$:
$\max_j\|\mathbf C(t_j) - \mathbf C^j\|_\infty \le \|\mathbf e^0\|_\infty + T\,\max_j\|\boldsymbol\tau^j\|_\infty = O\!\left(\Delta t + \left(\Delta r\right)^2\right)$.

*Proof.* The error satisfies $A\mathbf e^{j+1} = \mathbf e^j + \Delta t\,\boldsymbol\tau^{j+1}$. By Proposition G.3,
$\|\mathbf e^{j+1}\|_\infty \le \|\mathbf e^j\|_\infty + \Delta t\|\boldsymbol\tau^{j+1}\|_\infty$. Summing over the steps
and applying Proposition G.1 (every row is second order, including rows 0 and 1) gives the result. $\square$

The same argument on the five-layer grid gives only $O\!\left(\Delta t + h\right)$ in the maximum norm, because of the
$O(h)$ interface residuals. That is a proven but pessimistic bound.

**Proposition G.9 (PROOF OUTLINE): five layers, weighted $L^2$ norm, second order.**

Hypotheses:

- (H1) On $[t_*, T]$ with $t_* \ge 0$, the solution is $C^4$ in $r$ on each closed layer and $C^2$ in $t$.
- (H2) The grid is piecewise uniform with the interfaces at nodes. Its largest spacing is $h$, and the ratio between
  the largest and smallest spacings is bounded as $h \to 0$.
- (H3) $\|\mathbf e(t_*)\|_V = O\!\left(h^2\right)$.

Then $\max_j\|\mathbf e^j\|_V = O\!\left(h^2 + \Delta t\right)$ on $[t_*, T]$, with $\|\mathbf e\|_V^2 = \sum_{i\ge1}V_i\left(e_i\right)^2$.

*Argument.*

1. Split the residual of row $i$, multiplied by $V_i$, into two kinds. At smooth nodes it is $V_i\tau_i$ with
   $|\tau_i| \le c\left(h^2 + \Delta t\right)$ (Proposition G.1). At the four interface nodes it is $\rho_I$, with
   $|\rho_I| \le c\left(r_I\right)^2\left(h^2 + h\Delta t\right)$, by the $O\!\left(h^2\right)$ remainder of Section 4.2
   before the division by $\overline{\Delta r}_I$.
2. Multiply the error equation $V_i\left(e_i^{j+1}-e_i^j\right) = \Delta t\left[P_{i,\mathrm{outer}}(\mathbf e^{j+1}) - P_{i-1,\mathrm{outer}}(\mathbf e^{j+1}) + \text{residual}_i\right]$
   by $e_i^{j+1}$ and sum, as in Proposition G.4. Write
   $|\mathbf e|_g^2 = \sum_{i=0}^{N-1}4\pi g_{i,\mathrm{outer}}\left(e_{i+1}-e_i\right)^2$, with $e_N = 0$. This gives
   $$\frac{\|\mathbf e^{j+1}\|_V^2 - \|\mathbf e^j\|_V^2}{2\Delta t} + |\mathbf e^{j+1}|_g^2 \le c\left(h^2+\Delta t\right)\|\mathbf e^{j+1}\|_V + \sum_{I}|\rho_I|\,|e_I^{j+1}|.$$
3. A discrete Sobolev bound at the interface nodes. $e_N = 0$, so
   $|e_I| = \left|\sum_{i\ge I}\left(e_i - e_{i+1}\right)\right| \le |\mathbf e|_g\left(\sum_{i\ge I}\frac{1}{4\pi g_{i,\mathrm{outer}}}\right)^{1/2}$.
   For $i \ge I_1$, $g_{i,\mathrm{outer}} \ge D_{\min}\left(R_1\right)^2/h$, so the bracket is at most
   $S = R_5/\left(4\pi D_{\min}\left(R_1\right)^2\right)$, independent of $h$. The centre is not involved, because
   $I \ge I_1$.
4. Young's inequality: $\sum_I|\rho_I||e_I| \le |\mathbf e|_g^2 + S\left(\sum_I|\rho_I|\right)^2/4$. The second term is
   $O\!\left(\left(h^2+h\Delta t\right)^2\right)$. The $|\mathbf e|_g^2$ term cancels against the left side.
5. Sum over the steps. Let $X = \max_{j}\|\mathbf e^j\|_V$. Then
   $X^2 \le \|\mathbf e^0\|_V^2 + 2Tc\left(h^2+\Delta t\right)X + T\,O\!\left(\left(h^2+h\Delta t\right)^2\right)$, so
   $X \le \|\mathbf e^0\|_V + 2Tc\left(h^2+\Delta t\right) + O\!\left(\sqrt T\left(h^2 + h\Delta t\right)\right)$.

The argument is complete in its steps, but its constants are not tracked and it has not been checked independently.
Hence the label PROOF OUTLINE.

What it does not give:

- a pointwise (maximum-norm) second-order bound on the five-layer grid;
- a bound on the centre value. Row 0 gives
  $|e_0^{J}| \le \max\left(|e_0^0|, \max_j|e_1^j|\right) + t_J\max|\tau_0|$, so the centre error follows the
  pointwise error at node 1;
- anything for the discontinuous start on $[0, T]$. (H1) fails at $t = 0$. For that case the observed second order
  (Section 10.5) is **OBSERVED** only.

**Temporal order.** First order. The time residual is $-\tfrac{\Delta t}{2}\partial^2 C/\partial t^2$, and
Propositions G.8 and G.9 give $O(\Delta t)$. Observed: 0.975 to 0.997 (Section 10.4).

---

## 9. How generation (G1) and decay (G2) will enter the balance

With a source $G_i^{j+1}$ (in $\mathrm{mol\ m^{-3}\ s^{-1}}$) and decay constant $\lambda$ (in $\mathrm{s^{-1}}$), both
treated at level $j+1$:

$$
V_i\,\frac{C_i^{j+1}-C_i^j}{\Delta t} = P_{i,\mathrm{outer}}^{j+1} - P_{i-1,\mathrm{outer}}^{j+1} + V_i\,G_i^{j+1} - \lambda V_i\,C_i^{j+1}. \qquad \text{(G.1)}
$$

Row coefficients: $b_i$ gains $+\lambda\Delta t$, and $d_i = C_i^j + \Delta t\,G_i^{j+1}$. Row 0 is (B.2) with the same
two additions. Consequences:

- **Solvability, stability.** The margin of diagonal dominance becomes $1 + \lambda\Delta t$, so
  $\|A^{-1}\|_\infty \le 1/(1+\lambda\Delta t)$. Propositions G.2 and G.3 hold unchanged (PROVED by the same proofs).
- **Positivity.** If $G \ge 0$, then $d \ge 0$ and Proposition G.5 gives $\mathbf C^{j+1} \ge 0$. The upper bound
  $C_{\max}$ no longer holds when $G > 0$.
- **Discrete balance (PROVED, exact arithmetic).** Summing (G.1) as in Proposition D.1 gives
  $M_h^{j+1} - M_h^j = -\Delta t\,\Phi^{j+1} + \Delta t\sum_{i=1}^{N-1}V_iG_i^{j+1} - \lambda\Delta t\,M_h^{j+1}$. With
  $$M_{\mathrm{gen}}^J = \Delta t\sum_{j=1}^{J}\sum_{i=1}^{N-1}V_iG_i^{j},\qquad M_{\mathrm{dec}}^J = \lambda\Delta t\sum_{j=1}^{J}M_h^{j},\qquad M_{\mathrm{rel}}^J = \Delta t\sum_{j=1}^{J}\Phi^{j},$$
  this is $M_h^0 + M_{\mathrm{gen}}^J = M_h^J + M_{\mathrm{rel}}^J + M_{\mathrm{dec}}^J$. This is the directive's test F
  with $\varepsilon_{\mathrm{balance}}$ equal to rounding only. Decay is integrated at the implicit level, consistent
  with Backward Euler, and is tracked separately from release.
- **Source at an interface node.** A source that differs between materials (for example, kernel only) is
  discontinuous at the interface node. Use the half-interval split
  $V_IG_I := 4\pi\left(r_I\right)^2\left[\tfrac12\Delta r_{I,\mathrm{inner}}G_L + \tfrac12\Delta r_{I,\mathrm{outer}}G_R\right]$.
  For the kernel-only source, add the same trapezoidal correction as in (E.1) if the generated amount must be exact.
- **Source at the centre.** $V_0 = 0$, so a source at node 0 changes only $C_0$ and not the inventory. This is
  consistent with Proposition B.1.
- **Source at the surface (a correction needed for G1).** The half-interval of node $N$ has weight
  $V_N = 4\pi\left(R_5\right)^2\Delta r_{N-1,\mathrm{outer}}/2$ and no storage, because $C_N = 0$. Material generated
  there is in neither $M_h$ nor $M_{\mathrm{gen}}$ as defined above. Repeat the expansion of Section 5.2 with
  $\partial^2 C/\partial r^2 = -\frac{2}{R_5}\partial C/\partial r - G(R_5)/D_{\mathrm{OPyC}}$. The $O(h)$ bracket no
  longer vanishes: it equals $-2\pi\left(R_5\right)^2h\,G(R_5) = -V_NG_N$. The fix is to define
  $\Phi^{j+1} := 4\pi g_{N-1,\mathrm{outer}}C_{N-1}^{j+1} + V_NG_N^{j+1}$ and to add $V_NG_N$ to $M_{\mathrm{gen}}$. Then
  the balance stays exact and $\Phi$ is again second order. If the source is zero in the OPyC, this changes nothing.
- **Decay-only limit.** Without diffusion, the row gives $C^{j} = C^0\left(1+\lambda\Delta t\right)^{-j}$, which is
  first-order accurate against $C^0e^{-\lambda t_j}$. This is the Phase G2 analytical-limit check.
- **Temperature-dependent $D$ (G3, for completeness).** If $D$ changes with time, evaluate every $g_{i,\mathrm{outer}}$
  at the level used in the matrix. Use the same values in $\Phi$. (D.4) then remains exact step by step.

---

## 10. Numerical checks (scratch program, not committed)

All numbers come from an uncommitted scratch program in Python with NumPy and mpmath. It was used only to test the
statements above. It is not a STRATO tool, and the Rust verification suite of Phase D must re-implement these checks.
Synthetic input: the five-layer case of `inputs/synthetic/synthetic_five_layer.toml`. The case is labelled synthetic and
is not literature data. It has 40 intervals per layer ($N = 200$), $\Delta t = 200\ \mathrm{s}$ and $C_0 = 1$. All
results are numerical verification, not experimental validation.

### 10.1 Conservation and sign pattern (synthetic five-layer case)

| Quantity | Observed |
|---|---|
| $\theta$ at $R_1$, Eq. (E.1) | 0.6611111111111099 |
| $M_h^0$ against $\frac{4\pi}{3}\left(R_1\right)^3$ | $3.3510321638291135\times10^{-11}$ against $3.351032163829113\times10^{-11}\ \mathrm{mol}$ (relative $2.2\times10^{-16}$) |
| $\max_j\lvert\varepsilon^j\rvert/M_h^0$, 1000 steps, divided form (A.9) | $9.45\times10^{-11}$ |
| same, with one step of iterative refinement per solve | $7.22\times10^{-11}$ |
| same, multiplied-out assembly of Section 2.3 and compensated summation of $M_{\mathrm{rel}}$ | $4.27\times10^{-12}$ |
| same, divided form, $\Delta t = 2\ \mathrm{s}$ (mesh ratio 100 times smaller) | $3.26\times10^{-13}$ |
| $\min_{i,j} C_i^j$ | $0.0$ (no negative value) |
| largest off-diagonal entry of $A$ | $0.0$ |
| smallest diagonal-dominance margin | $0.9999999999999964$ (1 up to rounding) |
| smallest entry of $A^{-1}$; $\|A^{-1}\|_\infty$ | $0.0$; $1.000000000000001$ |
| $a_1$ | $-0.0$ |
| $\|A^{-1}\|_\infty$ at $\Delta t = 10^9\ \mathrm{s}$; smallest entry | $3.36\times10^{-4}$; $0.0$ |
| $F(2\times10^5\ \mathrm{s})$ as $M_{\mathrm{rel}}/M_h^0$ and as $1 - M_h/M_h^0$ | 0.4324291045505565 and 0.43242910464505013 |

The exact zeros in $A^{-1}$ are its column 0 below the diagonal. They are Proposition B.1 seen in the matrix: $C_0$
feeds nothing. The smaller residual at smaller mesh ratio, and in the multiplied-out form, supports the reading of
Section 8.6: the residual is rounding.

Release fraction at $t = 2\times10^5\ \mathrm{s}$, with the time integration done exactly in the eigenbasis of the
semi-discrete system:

| per-layer $n$ | $\Delta t = 200$ | $\Delta t = 20$ | $\Delta t = 2$ | exact in time |
|---|---|---|---|---|
| 40 | 0.4324291048 | 0.4325384786 | 0.4325494196 | 0.4325506353 |
| 80 | 0.4324328458 | 0.4325422336 | 0.4325531759 | 0.4325543918 |

The reference in the repository's v1.0.0 derivation (Part III, Section 3.5.4; analytical code) is
0.43255563909286. The time-exact errors are $5.00\times10^{-6}$ and $1.25\times10^{-6}$, a ratio of 4.01 (order 2.00).
At $\Delta t = 200\ \mathrm{s}$ the temporal error ($1.3\times10^{-4}$) dominates.

### 10.2 Interface flow from both sides (Proposition C.1), synthetic case, step 1000

| Interface node | $\mathcal Q_{I,L}/(4\pi)$ ($\mathrm{mol\ s^{-1}}$) | $\mathcal Q_{I,R}/(4\pi)$ ($\mathrm{mol\ s^{-1}}$) | relative difference |
|---|---|---|---|
| $I_1$ (kernel and buffer) | $-7.534712326167\times10^{-19}$ | $-7.534712325613\times10^{-19}$ | $7.4\times10^{-11}$ |
| $I_2$ (buffer and IPyC) | $-2.526714598464\times10^{-18}$ | $-2.526714598361\times10^{-18}$ | $4.1\times10^{-11}$ |
| $I_3$ (IPyC and SiC) | $-3.991375132006\times10^{-18}$ | $-3.991375132005\times10^{-18}$ | $1.3\times10^{-13}$ |
| $I_4$ (SiC and OPyC) | $-4.926916741211\times10^{-18}$ | $-4.926916741211\times10^{-18}$ | $2.8\times10^{-15}$ |

The raw one-sided flows $P_{I-1,\mathrm{outer}}$ and $P_{I,\mathrm{outer}}$ differ by up to 6% at $I_1$, which is the
half-interval storage. The reconstructed values agree to rounding. The rounding is larger where the mesh ratio is large
(the buffer).

### 10.3 Interface candidates: slowest five-layer eigenvalue

Reference. I computed it independently by shooting with closed-form layer solutions $(\alpha\sin kr + \beta\cos kr)/r$
in 40-digit arithmetic, matching $C$ and $D\,\partial C/\partial r$ at each interface:
$\Lambda_1 = 3.273255671177023208\times10^{-6}\ \mathrm{s^{-1}}$. This agrees with the value in the repository's v1.0.0
derivation, $3.273255671177023\times10^{-6}$, to all 16 digits printed there. The discrete $\Lambda_1$ is the smallest
eigenvalue of the semi-discrete operator, so it measures the spatial error alone. Relative errors, with $n$ intervals
per layer:

| $n$ | I | II | III | IV |
|---|---|---|---|---|
| 10 | $-1.429\times10^{-3}$ | $-6.010\times10^{-4}$ | $+9.980\times10^{-4}$ | $-6.590\times10^{-4}$ |
| 20 | $-3.575\times10^{-4}$ | $-1.503\times10^{-4}$ | $+2.506\times10^{-4}$ | $-1.570\times10^{-4}$ |
| 40 | $-8.940\times10^{-5}$ | $-3.758\times10^{-5}$ | $+6.279\times10^{-5}$ | $-3.831\times10^{-5}$ |
| 80 | $-2.235\times10^{-5}$ | $-9.395\times10^{-6}$ | $+1.572\times10^{-5}$ | $-9.466\times10^{-6}$ |
| 160 | $-5.588\times10^{-6}$ | $-2.379\times10^{-6}$ | not run | $-2.357\times10^{-6}$ |
| observed order | 2.00 | 1.98 to 2.00 | 1.99 to 2.00 | 2.01 to 2.07 |

All four converge at second order. On this measure the errors of II and IV are about 2.4 times smaller than those of
I. Accuracy on $\Lambda_1$ does not separate the candidates. Conservation and matrix structure do. The table gives the
largest relative inventory residual $\lvert M_h + M_{\mathrm{rel}} - M_h^0\rvert/M_h^0$, with steps of 200 s: 200 steps
for I and II, 1000 steps for III.

| $n$ | I | II | III |
|---|---|---|---|
| 20 | $2.8\times10^{-11}$ | $1.25\times10^{-3}$ | $3.00\times10^{-2}$ |
| 40 | $2.2\times10^{-11}$ | $3.15\times10^{-4}$ | $8.76\times10^{-3}$ |
| 80 | | | $2.37\times10^{-3}$ |

Candidate III also has 8 positive off-diagonal entries, two in each interface row. Its solution stayed non-negative in
this run, but no proof guarantees that.

### 10.4 Homogeneous sphere, smooth mode $C = \frac{\sin(\pi r/R_5)}{\pi r/R_5}e^{-D\pi^2t/\left(R_5\right)^2}$

$D = 10^{-12}\ \mathrm{m^2\ s^{-1}}$, $R_5 = 450\ \mu\mathrm m$, uniform grid.

Spatial error of the discrete $\Lambda_1$ against $D\pi^2/\left(R_5\right)^2$:

| $N$ | 20 | 40 | 80 | 160 | 320 | 640 |
|---|---|---|---|---|---|---|
| error | $-2.0545\times10^{-3}$ | $-5.1394\times10^{-4}$ | $-1.2850\times10^{-4}$ | $-3.2127\times10^{-5}$ | $-8.0319\times10^{-6}$ | $-2.0080\times10^{-6}$ |
| order | | 1.999 | 2.000 | 2.000 | 2.000 | 2.000 |

Centre-row variants, at $t = 0.2\left(R_5\right)^2/D$ with $\Delta t = T/(4N^2)$:

| $N$ | max error, $i \ge 1$ (all three variants) | centre error, (B.2) | centre error, factor 3 dropped | centre error, $C_0 = C_1$ |
|---|---|---|---|---|
| 20 | $7.306\times10^{-4}$ | $7.331\times10^{-4}$ | $1.894\times10^{-3}$ | $1.600\times10^{-4}$ |
| 40 | $1.831\times10^{-4}$ | $1.832\times10^{-4}$ | $4.700\times10^{-4}$ | $4.032\times10^{-5}$ |
| 80 | $4.580\times10^{-5}$ | $4.581\times10^{-5}$ | $1.173\times10^{-4}$ | $1.010\times10^{-5}$ |
| 160 | $1.145\times10^{-5}$ | $1.145\times10^{-5}$ | $2.931\times10^{-5}$ | $2.526\times10^{-6}$ |

The error at $i \ge 1$ is identical to all printed digits for the three variants, as Proposition B.1 predicts. All
centre errors are second order. That $C_0 = C_1$ happens to be closer here is a property of this mode, not an
argument for an inconsistent row.

Temporal order ($N = 400$, $t = 0.2\left(R_5\right)^2/D$, measured against the time-exact semi-discrete solution):
the maximum errors for 10, 20, 40, 80 and 160 steps are $2.6146\times10^{-2}$, $1.3301\times10^{-2}$,
$6.7078\times10^{-3}$, $3.3683\times10^{-3}$ and $1.6878\times10^{-3}$. The observed orders are 0.975, 0.988, 0.994
and 0.997.

### 10.5 Discontinuous start in a homogeneous sphere

$D = 10^{-12}\ \mathrm{m^2\ s^{-1}}$, $R_1 = 200\ \mu\mathrm m$, $R_5 = 450\ \mu\mathrm m$, $C_0$ for $r < R_1$, node at
$R_1$, $K$ kernel intervals, uniform grid. The exact solution is the eigenfunction series with 40000 terms. Spatial
error only (time-exact semi-discrete solution). Error in $F$ at
$t = 4\times10^2,\ 4\times10^3,\ 4\times10^4,\ 4\times10^5\ \mathrm{s}$:

| $\theta$ | $K = 8$ | $K = 128$ | observed order, $K = 64 \to 128$ |
|---|---|---|---|
| $\tfrac12$ | $7.81\times10^{-3}$, $6.28\times10^{-3}$, $1.52\times10^{-3}$, $2.9\times10^{-10}$ | $3.05\times10^{-5}$, $2.46\times10^{-5}$, $5.93\times10^{-6}$, $1.1\times10^{-12}$ | 2.00, 2.00, 2.00, 2.00 |
| matched (E.1) | $4.6\times10^{-10}$, $1.42\times10^{-3}$, $5.17\times10^{-5}$, $2.5\times10^{-10}$ | $2\times10^{-14}$, $5.54\times10^{-6}$, $1.94\times10^{-7}$, $9.7\times10^{-13}$ | rounding, 2.00, 2.00, 2.00 |
| 1 | $1.95\times10^{-1}$, $1.91\times10^{-1}$, $3.92\times10^{-2}$, $1.2\times10^{-9}$ | $1.17\times10^{-2}$, $1.16\times10^{-2}$, $2.36\times10^{-3}$, $5.8\times10^{-11}$ | 1.00, 1.00, 1.00, 1.03 |
| 0 | $1.80\times10^{-1}$, $1.79\times10^{-1}$, $3.62\times10^{-2}$, $6.5\times10^{-10}$ | $1.17\times10^{-2}$, $1.16\times10^{-2}$, $2.35\times10^{-3}$, $5.5\times10^{-11}$ | 1.00, 1.00, 1.00, 0.97 |

At $t = 4\times10^2\ \mathrm{s}$ almost nothing has been released. The matched value's error there is the exactness of
$M_h^0$. The weighted $L^2$ profile error behaves in the same way: order 2.00 for $\tfrac12$ and the matched value at
every time, order 1.0 for $\theta = 0$ and $\theta = 1$. The maximum nodal error with the matched value, at
$t = 40\ \mathrm{s}$ and $t = 4\times10^4\ \mathrm{s}$, for $K = 8, 16, 32, 64, 128, 256$:

| $K$ | 8 | 16 | 32 | 64 | 128 | 256 |
|---|---|---|---|---|---|---|
| $t = 40\ \mathrm{s}$ | $3.30\times10^{-2}$ | $2.29\times10^{-2}$ | $6.69\times10^{-3}$ | $1.82\times10^{-3}$ | $4.71\times10^{-4}$ | $1.18\times10^{-4}$ |
| order | | 0.52 | 1.78 | 1.88 | 1.95 | 2.00 |
| $t = 4\times10^4\ \mathrm{s}$ | $4.55\times10^{-3}$ | $1.18\times10^{-3}$ | $2.97\times10^{-4}$ | $7.44\times10^{-5}$ | $1.86\times10^{-5}$ | $4.66\times10^{-6}$ |
| order | | 1.95 | 1.99 | 2.00 | 2.00 | 2.00 |

Backward Euler with this start ($K = 64$): the maximum nodal error at $t = 4\times10^2\ \mathrm{s}$ halves with every
halving of $\Delta t$ (order 1.00, from $1.57\times10^{-2}$ with 5 steps to $4.96\times10^{-4}$ with 160 steps). The
error in $F$ at $t = 4\times10^4\ \mathrm{s}$ has order 0.90 to 0.99.

### 10.6 Grids (homogeneous eigenvalue)

| Grid | errors for $N = 40, 80, 160, 320$ | observed order |
|---|---|---|
| uniform | $-5.139\times10^{-4}$, $-1.285\times10^{-4}$, $-3.213\times10^{-5}$, $-8.032\times10^{-6}$ | 2.00 |
| smooth exponential grading ($r = R_5\left(e^{ax}-1\right)/\left(e^a-1\right)$, $a = 0.792$) | $-6.441\times10^{-4}$, $-1.611\times10^{-4}$, $-4.028\times10^{-5}$, $-1.007\times10^{-5}$ | 2.00 |
| spacings alternating $h, 2h$ | $-6.856\times10^{-4}$, $-1.714\times10^{-4}$, $-4.284\times10^{-5}$, $-1.071\times10^{-5}$ | 2.00 |

---

## 11. Summary of conclusions

1. Interior rows: (A.3) on uniform spacing; (A.9) on any grid. The owner's stencil is exactly a flux difference with
   face factors $r_ir_{i+1}$ and weight $\left(r_i\right)^2\Delta r$. This needs only local uniformity (Proposition
   A.1).
2. Centre: (B.2), $\left(1+6\mu_0\right)C_0^{j+1} - 6\mu_0C_1^{j+1} = C_0^j$, second order. $a_1 = 0$, so $C_0$ feeds
   nothing: not the other nodes, not the inventory, not the release. A wrong centre row damages only $C_0$
   (Proposition B.1).
3. Interfaces: Candidate I, a node at each interface with $D_L$ on the inner half and $D_R$ on the outer half. It is
   the general formula (A.9) itself. It is tridiagonal, an M-matrix, exactly conservative, and has an $O(h)$ local
   error at four nodes, with second-order global convergence (PROOF OUTLINE; OBSERVED). On equal spacing, the
   two-sided ghost treatment is identical to it.
4. Outer boundary: $C_N = 0$ removed from row $N-1$. The release is
   $\Phi^{j+1} = 4\pi D_{\mathrm{OPyC}}r_{N-1}R_5C_{N-1}^{j+1}/\Delta r_{N-1,\mathrm{outer}}$, which is second order and
   makes $M_h^j + M_{\mathrm{rel}}^j = M_h^0$ exact in exact arithmetic.
5. Start: $\theta$ from (E.1), so that $M_h^0 = M^0$ exactly. Second order is observed at fixed $t > 0$, with $\theta = 0$
   or $1$ degrading the order to one.
6. Grids: piecewise uniform, with nodes at the interfaces.
7. Properties: unconditional solvability, maximum-norm stability, maximum principle and positivity, including in
   floating point for the Thomas algorithm (PROVED). Exact discrete conservation in exact arithmetic (PROVED); a rounding
   residual in floating point (OBSERVED $\sim10^{-12}$ to $10^{-10}$). First order in time. Second order in space:
   PROVED in the maximum norm for the homogeneous sphere; PROOF OUTLINE in the weighted $L^2$ norm for five layers;
   OBSERVED for the discontinuous start.
8. For G1, the release must include the generation in the outer half-interval, or $\Phi$ drops to first order
   (Section 9).

Open items for the comparison: check Section 3.3 against the owner's notes; confirm or refute the $O(h)$ interface
residual and the proof outline of Proposition G.9; decide whether the production solver should report $C_0$ from (B.2),
which this document recommends.
