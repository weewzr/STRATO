# STRATO: Generation, Decay and Temperature-Dependent Diffusivity in the Five-Layer BTCS Scheme

**Status.** This is Document 5 of the BTCS mathematics. It derives the three approved additional mechanisms of directive Section 17, generation (G1), radioactive decay (G2) and temperature-dependent diffusivity (G3), and their combination (G4), on the grid and with the matrix rows of [Document 2](02_final_formulation.md). It also gives (R), a rounding analysis of the balance residual of the baseline scheme and of the combined scheme, which yields rigorous bounds with every rounding of the computation counted, including the evaluation of the bound itself; Section 5.7 compares them with the derived a-posteriori bound that the Rust tests now use. The normative equations are stated in Document 2, Section 11, as (F32) to (F46), and the new properties as P19 to P31. This document holds their derivations and proofs.

Citations: "D1" is [Document 1](01_from_ftcs_to_five_layer_btcs.md), "D2" is Document 2 with its equation numbers (F1) to (F46), "archive B" is [independent derivation B](archive/independent_derivation_b.md). Equations of this document are numbered (1), (2), and so on.

**Labels**, as in D2. PROVED: a complete proof is given here or in D1. PROOF OUTLINE: every step is given, but the constants are not tracked. OBSERVED: shown only by scratch computations. NOT PROVEN: open.

**Numbers.** Every number in this document comes from uncommitted scratch programs that implement D2 and the equations below row by row, written only to test the algebra (Section 7). The synthetic case is `inputs/synthetic/synthetic_five_layer.toml`, which states that it is not literature data. The activation temperatures, source strengths and decay constants of the scratch runs are synthetic test values chosen for the checks, not data. Nothing here is experimental validation.

---

## 0. Model, scope and symbols

### 0.1 The approved combined equation

Directive Section 17 (G4) approves

$$
\frac{\partial C}{\partial t} = \frac{1}{\left(r\right)^2}\frac{\partial}{\partial r}\left[\left(r\right)^2 D(r, T)\,\frac{\partial C}{\partial r}\right] + G - \lambda C, \tag{1}
$$

on $0 < r < R_5$, $t > 0$, with the regular centre, perfect contact at $R_1, \dots, R_4$ and the absorbing surface $C(R_5, t) = 0$ of D2 Section 1.

**Assumptions of this document.**

1. The source $G$ is constant in space inside each material and may vary in time: $G(r, t) = G_{\mathrm{material}}(t)$ for $r$ inside that material. The physically approved case is the kernel-only source; general per-material values are allowed, because verification needs them (Section 4.5). A source that varies inside one material is not covered.
2. $G_{\mathrm{material}}(t) \ge 0$, and each $G_{\mathrm{material}}$ is integrable over every time step.
3. The decay constant $\lambda \ge 0$ is the same in every material and constant in time.
4. The temperature $T$ is uniform in the particle, $T = T(t)$. Each material has its own law $D_{\mathrm{material}}(T) > 0$.
5. Everything else is as in D2: five materials, piecewise-uniform grid with nodes at $0, R_1, \dots, R_5$, kernel-only start (F21) and (F22).

### 0.2 New symbols and units

The notation of directive Section 5 is binding: $i$ is the global radial node index, $j$ the time-step index, $C_i^j$ the concentration at node $i$ and time step $j$, $r_i$ the radius of node $i$, $\Delta r$ a spacing, $\Delta t$ the time step. A time superscript is an index, not a power; a power encloses the whole variable in parentheses. Every symbol of D2 Section 2 keeps its meaning. The new ones:

| Symbol | Meaning | Unit |
|---|---|---|
| $G(r, t)$ | volumetric source (generation rate density) | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $G_{\mathrm{kernel}}(t), G_{\mathrm{buffer}}(t), G_{\mathrm{IPyC}}(t), G_{\mathrm{SiC}}(t), G_{\mathrm{OPyC}}(t)$ | source in each material | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $\bar G_{\mathrm{material}}^j$ | average of $G_{\mathrm{material}}$ over the step from $t^j$ to $t^{j+1}$ | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $\dot F$ | fission-rate density in the kernel | $\mathrm{m^{-3}\ s^{-1}}$ (fissions per cubic metre per second) |
| $y$ | fission yield of the nuclide | dimensionless (atoms per fission) |
| $N_{\mathrm{A}}$ | Avogadro constant, $6.02214076\times10^{23}$ (exact by the SI definition) | $\mathrm{mol^{-1}}$ |
| $\lambda$ | decay constant | $\mathrm{s^{-1}}$ |
| $t_{1/2}$ | half-life, $\lambda = \ln 2/t_{1/2}$ | $\mathrm{s}$ |
| $\kappa$ | decay number of a step, $\kappa = \lambda\,\Delta t$ | dimensionless |
| $T(t)$ | particle temperature | $\mathrm{K}$ |
| $D_{0,\mathrm{material}}$ | pre-exponential factor | $\mathrm{m^2\ s^{-1}}$ |
| $E_{a,\mathrm{material}}$ | activation energy | $\mathrm{J\ mol^{-1}}$ |
| $R_{\mathrm{gas}}$ | molar gas constant (the directive writes $R$; renamed here because $R_1, \dots, R_5$ are radii) | $\mathrm{J\ mol^{-1}\ K^{-1}}$ |
| $\bar D_{\mathrm{material}}^j$ | diffusivity of the material used in the step from $t^j$ to $t^{j+1}$ | $\mathrm{m^2\ s^{-1}}$ |
| $K_{i,\mathrm{outer}}^j, \mathrm{Fo}_{\mathrm{material}}^j, L_i^j, B_i^j, U_i^j$ | (F6), (F8) and the row coefficients evaluated with $\bar D^j$ | as in D2 |
| $Q_i^j$ | source weight of node $i$ in step $j$ (generation rate in the dual region of node $i$, divided by $4\pi$) | $\mathrm{mol\ s^{-1}}$ |
| $\omega$ | kernel closure factor, $\omega = 1 - 1/\left(3\,n_{\mathrm{kernel}}\right)$ | dimensionless |
| $g_i^j$ | nodal source density of row $i$ | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $V_N$ | weight of the inner half-interval of the boundary node, $V_N = \left(R_5\right)^2\Delta r_{N-1,\mathrm{outer}}/2$ | $\mathrm{m^3}$ |
| $M_{\mathrm{gen}}^j, M_{\mathrm{dec}}^j$ | generated amount and decayed amount up to $t^j$ | $\mathrm{mol}$ |
| $F_{\mathrm{gen}}^j$ | release fraction relative to all material that has been present | dimensionless |
| $\beta^j$ | ratio of release rate to generation rate | dimensionless |
| $u$ | unit roundoff of IEEE double precision, $u = 2^{-53}$ | dimensionless |
| $\gamma_\nu$ | $\nu u/\left(1 - \nu u\right)$ for a positive integer $\nu$ with $\nu u < 1$ | dimensionless |
| $\mu$, $w_\mu$, $\alpha_\mu$ | mode number of the kernel-step series, its weight, its decay rate $D\left(\mu\pi/R_5\right)^2$ (Section 2.3) | dimensionless, dimensionless, $\mathrm{s^{-1}}$ |
| $C_{\mathrm{unit}}(r, \tau)$ | solution of the homogeneous problem with unit diffusivity, in the variable $\tau = \int D\,dt$ (Section 3.3) | $\mathrm{mol\ m^{-3}}$ ($\tau$ in $\mathrm{m^2}$) |
| $\Theta$, $D_{1000}$ | activation temperature $E_a/R_{\mathrm{gas}}$ and diffusivity at $1000\ \mathrm{K}$ of the synthetic laws of Section 3.2 | $\mathrm{K}$, $\mathrm{m^2\ s^{-1}}$ |
| $\chi^k$ | $1$ if some nodal source density of step $k$ is positive, otherwise $0$ (Section 5.8) | dimensionless |

---

## 1. G1: generation

### 1.1 The physical source

For a direct fission product generated in the kernel,

$$
G_{\mathrm{kernel}}(t) = \frac{y\,\dot F(t)}{N_{\mathrm{A}}}, \qquad G_{\mathrm{buffer}} = G_{\mathrm{IPyC}} = G_{\mathrm{SiC}} = G_{\mathrm{OPyC}} = 0. \tag{2}
$$

Units: $\mathrm{m^{-3}\ s^{-1}} \times 1 / \mathrm{mol^{-1}} = \mathrm{mol\ m^{-3}\ s^{-1}}$. Three points are data decisions, not mathematics, and are not fixed here:

- **Yield type.** An independent yield counts only atoms born directly in fission; a cumulative yield also counts atoms that arrive by decay of short-lived precursors, and so assumes that the precursors decay where they are born. The register distinguishes the two (directive Section 7); the choice for each nuclide is stated with the data.
- **Fission-rate density.** $\dot F$ is not in the parameter register (GAP-28). Until it is registered with a citation, it is a plain user input (coordinator decision 14).
- **Activation products.** Ag-110m, Cs-134 and Eu-154 have no direct fission-yield source; their $G$ is zero until a production model is approved (decision 15).

### 1.2 Flux balance with a source

Integrate Eq. (1), with $\lambda = 0$, over the dual region of node $i$, exactly as D1 Section 3.5 does without a source. The source adds the generation rate in that region on the right of (F10):

$$
V_i\,\frac{C_i^{j+1} - C_i^j}{\Delta t} = K_{i,\mathrm{outer}}\left(C_{i+1}^{j+1} - C_i^{j+1}\right) - K_{i,\mathrm{inner}}\left(C_i^{j+1} - C_{i-1}^{j+1}\right) + \bar Q_i^j, \qquad 1 \le i \le N-1. \tag{3}
$$

Dividing by $V_i$ leaves the matrix of D2 unchanged and adds $\Delta t\,g_i^j$, with $g_i^j = \bar Q_i^j/V_i$, to the right-hand side (F20). Three things must be fixed: the weights $Q_i$, the centre row, and the time level. Each is fixed so that the generated amount is booked in the same discrete convention as the inventory (F25).

### 1.3 The weights at interior, interface, centre and outer nodes

**The convention of the inventory.** (F25) and (F7) give each node the trapezoidal weight of its two half-intervals: $\left(r_i\right)^2\Delta r_{i,\mathrm{inner}}/2$ on the inner side and $\left(r_i\right)^2\Delta r_{i,\mathrm{outer}}/2$ on the outer side. A field that is constant across a node is represented by its nodal value. The kernel-only start is the one exception: the field jumps at $R_1$, and (F22) gives node $N_{\mathrm{kernel}}$ the value $\theta\,C_{\mathrm{init}}$, chosen so that the discrete inventory of the kernel is exact, (F23).

**Decomposition of the source.** Write the per-material source as a field continuous across $R_1$ plus a kernel indicator:

$$
G = \underbrace{G^{(\mathrm{c})}}_{\text{equal to } G_{\mathrm{buffer}} \text{ in the kernel}} + \left(G_{\mathrm{kernel}} - G_{\mathrm{buffer}}\right)\chi_{\mathrm{kernel}}, \qquad \chi_{\mathrm{kernel}}(r) = \begin{cases}1, & r < R_1,\\ 0, & r > R_1,\end{cases} \tag{4}
$$

where $G^{(\mathrm{c})}$ equals $G_{\mathrm{buffer}}$ in kernel and buffer and $G_{\mathrm{material}}$ in the outer coatings. Each part is weighted as its kind of field is represented in the inventory:

- the part $G^{(\mathrm{c})}$, which has no jump at $R_1$, with the plain half-interval weights;
- the kernel indicator with the inventory-matched shape of (F21) and (F22).

The inventory-matched shape has weight $V_i$ at the kernel nodes $1 \le i \le N_{\mathrm{kernel}} - 1$ and weight $\theta\,V_{N_{\mathrm{kernel}}}$ at node $N_{\mathrm{kernel}}$. By (F22) and (F7),

$$
\theta\,V_{N_{\mathrm{kernel}}} = \frac{\Delta r_{\mathrm{kernel}}}{\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}}\,\omega\,\left(R_1\right)^2\frac{\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}}{2} = \omega\left(R_1\right)^2\frac{\Delta r_{\mathrm{kernel}}}{2}, \qquad \omega = 1 - \frac{1}{3\,n_{\mathrm{kernel}}}. \tag{5}
$$

So the kernel indicator takes the plain weight on the kernel side of node $N_{\mathrm{kernel}}$, reduced by the factor $\omega$. Adding the two parts gives the weights.

**Weights (normative form in D2, (F33)).** With $G_{i,\mathrm{inner}}$ and $G_{i,\mathrm{outer}}$ the step-averaged sources of the materials on the inner and outer side of node $i$ (the same assignment as (F4)):

$$
\bar Q_i^j = \frac{\left(r_i\right)^2}{2}\left[\Delta r_{i,\mathrm{inner}}\,\bar G_{i,\mathrm{inner}}^j + \Delta r_{i,\mathrm{outer}}\,\bar G_{i,\mathrm{outer}}^j\right], \qquad 1 \le i \le N-1,\ i \ne N_{\mathrm{kernel}}, \tag{6}
$$

$$
\bar Q_{N_{\mathrm{kernel}}}^j = \frac{\left(R_1\right)^2}{2}\left[\Delta r_{\mathrm{kernel}}\left(\omega\,\bar G_{\mathrm{kernel}}^j + \left(1 - \omega\right)\bar G_{\mathrm{buffer}}^j\right) + \Delta r_{\mathrm{buffer}}\,\bar G_{\mathrm{buffer}}^j\right], \tag{7}
$$

$$
\bar Q_N^j = \frac{\left(R_5\right)^2}{2}\,\Delta r_{N-1,\mathrm{outer}}\,\bar G_{\mathrm{OPyC}}^j = V_N\,\bar G_{\mathrm{OPyC}}^j. \tag{8}
$$

Check of (7): with $\bar G_{\mathrm{kernel}} = \bar G_{\mathrm{buffer}}$ it reduces to (6), and with $\bar G_{\mathrm{buffer}} = 0$ it is $\omega\left(R_1\right)^2\Delta r_{\mathrm{kernel}}\bar G_{\mathrm{kernel}}/2 = \theta\,V_{N_{\mathrm{kernel}}}\bar G_{\mathrm{kernel}}$, by (5). The bracket of (7) is a combination of $\bar G_{\mathrm{kernel}}$ and $\bar G_{\mathrm{buffer}}$ with non-negative coefficients, because $\frac{2}{3} \le \omega < 1$.

The node classes:

- **Interior nodes** of one material: (6) gives $\bar Q_i = V_i\,\bar G_{\mathrm{material}}$, so $g_i = \bar G_{\mathrm{material}}$.
- **Interface nodes** $N_{\mathrm{buffer}}, N_{\mathrm{IPyC}}, N_{\mathrm{SiC}}$: (6) is the half-interval split of archive B, Section 9. Each material's source enters only through the half-interval that lies in it. No source is averaged across a jump.
- **Interface node** $N_{\mathrm{kernel}}$: (7).
- **Centre.** $V_0 = 0$, so node 0 has no weight and the centre source enters neither the inventory nor the generated amount (consistent with P7). Row 0 is the pointwise equation at $r = 0$ (D1 Section 4.2) and gets the pointwise source: $g_0^j = \bar G_{\mathrm{kernel}}^j$.
- **Outer node** $N$: no unknown, no row. Its inner half-interval has weight $V_N$, and its source (8) is booked in Section 1.4.

**Nodal source densities (D2, (F34)).**

$$
g_0^j = \bar G_{\mathrm{kernel}}^j, \qquad g_i^j = \frac{\bar Q_i^j}{V_i} \quad \left(1 \le i \le N-1\right). \tag{9}
$$

By the remark after (7), $g_i^j$ lies between the smallest and the largest of the sources of the materials adjacent to node $i$.

**Why $\omega$ at node $N_{\mathrm{kernel}}$.** The plain weight would also be consistent: it changes $\bar Q_{N_{\mathrm{kernel}}}$ by $\left(R_1\right)^2\Delta r_{\mathrm{kernel}}\bar G_{\mathrm{kernel}}/\left(6\,n_{\mathrm{kernel}}\right)$, which is of second order in the total. The factor $\omega$ is chosen because it gives three exact properties that the plain weight does not have, each proved below:

1. **Exact generated amount (P20).** For a kernel-only source, $\sum_{i=1}^{N}\bar Q_i^j = \frac{1}{3}\left(R_1\right)^3\bar G_{\mathrm{kernel}}^j$ exactly: the kernel part of the weights is the inventory-matched shape, whose sum is $\frac{1}{3}\left(R_1\right)^3$ by (F23). With the plain weight the sum is $\frac{1}{3}\left(R_1\right)^3\left(1 + 1/\left(2\left(n_{\mathrm{kernel}}\right)^2\right)\right)$, because $\left(\Delta r_{\mathrm{kernel}}\right)^3\left[\sum_{i=1}^{n_{\mathrm{kernel}}-1}\left(i\right)^2 + \frac{1}{2}\left(n_{\mathrm{kernel}}\right)^2\right] = \frac{1}{3}\left(R_1\right)^3 + \frac{1}{6}R_1\left(\Delta r_{\mathrm{kernel}}\right)^2$.
2. **The no-diffusion limit is the discrete start representation (P21).** With $D \to 0$ the rows become $C_i^{j+1} = C_i^j + \Delta t\,g_i^j$. A kernel-only source then adds $\Delta t\,\bar G_{\mathrm{kernel}}$ at kernel nodes and $\theta\,\Delta t\,\bar G_{\mathrm{kernel}}$ at node $N_{\mathrm{kernel}}$: the field $C_{\mathrm{init}} + \int G_{\mathrm{kernel}}\,dt$ in the kernel keeps the shape (F21). With the plain weight node $N_{\mathrm{kernel}}$ would gain $\Delta t\,\bar G_{\mathrm{kernel}}\,\Delta r_{\mathrm{kernel}}/\left(\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}\right)$ instead.
3. **The discrete steady state is exact (P22).** Proved in Section 1.6.

### 1.4 The outer node: release correction

**Balance.** Summing (3) over $i = 1, \dots, N-1$ telescopes as in D1 Eq. (35):

$$
\frac{M^{j+1} - M^j}{\Delta t} = -4\pi K_{N-1,\mathrm{outer}}C_{N-1}^{j+1} + 4\pi\sum_{i=1}^{N-1}\bar Q_i^j. \tag{10}
$$

The source of the half-interval next to $R_5$, $\bar Q_N^j$, is in neither sum. It lies in a region whose concentration is held at zero, so whatever is generated there leaves at once.

**Consistency of the release (archive B, Section 9, re-derived).** Let $h = \Delta r_{N-1,\mathrm{outer}}$ and insert a solution that is smooth in the OPyC near $R_5$ into the shell-flow formula, as in D1 Eq. (36a). With all derivatives at $R_5^-$, Taylor's theorem and $C(R_5, t) = 0$ give $C(r_{N-1}) = -h\,C' + \frac{1}{2}\left(h\right)^2C'' + O\left(\left(h\right)^3\right)$ and

$$
\frac{r_{N-1}R_5\,C(r_{N-1})}{h} = \left(R_5\right)^2\left[-C' + h\left(\frac{1}{2}C'' + \frac{1}{R_5}C'\right)\right] + O\left(\left(h\right)^2\right).
$$

At $R_5$ the concentration is zero for every $t$, so $\partial_t C = 0$ and $\lambda C = 0$ there, and Eq. (1) gives $D_{\mathrm{OPyC}}\left(C'' + \frac{2}{R_5}C'\right) + G_{\mathrm{OPyC}} = 0$, that is $\frac{1}{2}C'' + \frac{1}{R_5}C' = -G_{\mathrm{OPyC}}/\left(2D_{\mathrm{OPyC}}\right)$. Hence

$$
4\pi K_{N-1,\mathrm{outer}}\,C(r_{N-1}, t) = \Phi_R(t) - 4\pi\,\frac{\left(R_5\right)^2h}{2}\,G_{\mathrm{OPyC}}(t) + O\left(\left(h\right)^2\right) = \Phi_R(t) - 4\pi V_N\,G_{\mathrm{OPyC}}(t) + O\left(\left(h\right)^2\right), \tag{11}
$$

with $\Phi_R = -4\pi\left(R_5\right)^2D_{\mathrm{OPyC}}\,\partial_rC(R_5, t)$ the exact release rate. The plain formula (F26) is therefore only first-order consistent when $G_{\mathrm{OPyC}} \ne 0$.

**Release with a source (D2, (F39)).** Define

$$
\Phi^{j+1} = 4\pi K_{N-1,\mathrm{outer}}^j\,C_{N-1}^{j+1} + 4\pi\bar Q_N^j, \qquad M_{\mathrm{gen}}^{j+1} = M_{\mathrm{gen}}^j + 4\pi\,\Delta t\sum_{i=1}^{N}\bar Q_i^j. \tag{12}
$$

Both sums now include node $N$. Adding and subtracting $4\pi\,\Delta t\,\bar Q_N^j$ in (10) gives $M^{j+1} - M^j = -\Delta t\,\Phi^{j+1} + \left(M_{\mathrm{gen}}^{j+1} - M_{\mathrm{gen}}^j\right)$: the balance stays exact, and by (11) the release is second-order consistent again (P23). For the approved kernel-only source $\bar Q_N^j = 0$ and (12) is (F26).

### 1.5 Time treatment of the source

The step uses the average of the source over the step,

$$
\bar G_{\mathrm{material}}^j = \frac{1}{\Delta t}\int_{t^j}^{t^{j+1}}G_{\mathrm{material}}(t)\,dt. \tag{13}
$$

- **Exact generated amount over a step (P20).** By (12) and (13), $M_{\mathrm{gen}}^j = 4\pi\int_0^{t^j}\sum_{i=1}^{N}Q_i(t)\,dt$ exactly, where $Q_i(t)$ is (6) to (8) with $G(t)$ in place of $\bar G^j$. For a kernel-only source this is $\frac{4\pi}{3}\left(R_1\right)^3\int_0^{t^j}G_{\mathrm{kernel}}(t)\,dt$, the exact amount generated in the kernel.
- **Computation.** For a source history that is piecewise constant or piecewise linear in time, (13) is evaluated exactly: the sum over the pieces inside the step of length times mean value. If the source is constant over the step, $\bar G^j = G$.
- **Consistency (PROVED).** Insert the exact solution. $\left(C(t^{j+1}) - C(t^j)\right)/\Delta t = \partial_t C(t^{j+1}) + O(\Delta t)$, and $\bar G^j = G(t^{j+1}) + O(\Delta t)$ when $G$ is continuously differentiable on the step, so the row residual is first order in time, as without a source. More precisely, with (13) and $\mathcal{T}_DC = \left(r\right)^{-2}\partial_r\left(\left(r\right)^2D\,\partial_rC\right)$ the continuous diffusion operator (unit of $\mathcal{T}_DC$: $\mathrm{mol\ m^{-3}\ s^{-1}}$), the time residual is $\frac{1}{\Delta t}\int_{t^j}^{t^{j+1}}\left[\mathcal{T}_DC(t) - \mathcal{T}_DC(t^{j+1})\right]dt$: the source contributes no time error of its own, also when $G$ jumps inside the step.
- **No diffusion, no decay.** $C_i^{j+1} = C_i^j + \Delta t\,g_i^j = C_i^j + \int_{t^j}^{t^{j+1}}g_i(t)\,dt$: Backward Euler with (13) is exact for pure accumulation.

### 1.6 Properties of G1

**Zero-source limit (P19, PROVED).** If every $\bar G^j = 0$, then $\Delta t\,g_i^j = 0$, the right-hand side is $C_i^j + 0 = C_i^j$ exactly in IEEE arithmetic, $\bar Q_N^j = 0$ adds nothing to the release, and the run is bit for bit the baseline run. (Observed: identical arrays in a scratch run.)

**Positivity (PROVED).** $G \ge 0$ gives $\bar Q_i^j \ge 0$ and $g_i^j \ge 0$ (the coefficients of (7) are non-negative), so $\mathbf{b}^j \ge \mathbf{0}$ whenever $\mathbf{C}^j \ge \mathbf{0}$, and P4 (and P6 in floating point, because $C + \Delta t\,g$ is a sum of non-negative numbers) gives $\mathbf{C}^{j+1} \ge \mathbf{0}$. The upper bound of P4 is replaced by (28) of Section 4.3. $\Phi^{j+1} \ge 0$ and $M_{\mathrm{rel}}^j$ is non-decreasing.

**Exact steady state for a kernel-only source (P22, PROVED).** Let $G_{\mathrm{kernel}} > 0$ be constant, the other sources zero, and the diffusivities any positive piecewise-constant values. The exact steady solution of Eq. (1) is: the outward flow through radius $r$, divided by $4\pi$, is $\frac{1}{3}\left(r\right)^3G_{\mathrm{kernel}}$ for $r \le R_1$ and $\frac{1}{3}\left(R_1\right)^3G_{\mathrm{kernel}}$ for $r \ge R_1$; so

$$
C(r) = C(R_1) + \frac{G_{\mathrm{kernel}}\left(\left(R_1\right)^2 - \left(r\right)^2\right)}{6D_{\mathrm{kernel}}} \ \ (r \le R_1), \qquad C(r) = C(R_{\mathrm{next}}) + \frac{G_{\mathrm{kernel}}\left(R_1\right)^3}{3D_{\mathrm{material}}}\left(\frac{1}{r} - \frac{1}{R_{\mathrm{next}}}\right) \ \ \text{(coating material)}, \tag{14}
$$

with $R_{\mathrm{next}}$ the outer radius of the material and $C(R_5) = 0$. The claim is that the nodal values of (14) solve the steady discrete rows (3) exactly. Proof, row by row:

- *Coatings.* On an interval inside a coating, $C = a/r + b$ with $a = G_{\mathrm{kernel}}\left(R_1\right)^3/\left(3D_{\mathrm{material}}\right)$, and $K_{i,\mathrm{outer}}\left(C_i - C_{i+1}\right) = \frac{D_{\mathrm{material}}r_ir_{i+1}}{\Delta r}\,a\,\frac{r_{i+1} - r_i}{r_ir_{i+1}} = D_{\mathrm{material}}\,a = \frac{1}{3}\left(R_1\right)^3G_{\mathrm{kernel}}$: the shell conductance is exact for this profile, every outer interval carries the exact flow, and rows $N_{\mathrm{kernel}} < i \le N-1$, which have no source, balance. $C_N = 0$ is exact.
- *Kernel interior, node 1 and the centre.* For $C = c - b\left(r\right)^2$, $b = G_{\mathrm{kernel}}/\left(6D_{\mathrm{kernel}}\right)$, and spacing $h = \Delta r_{\mathrm{kernel}}$: $K_{i,\mathrm{outer}}\left(C_{i+1} - C_i\right) = -D_{\mathrm{kernel}}b\,r_ir_{i+1}\left(r_i + r_{i+1}\right)$. With $r_{i\pm1} = r_i \pm h$ the difference of the outer and inner flows is $-D_{\mathrm{kernel}}b\,r_i\left[r_{i+1}\left(r_i + r_{i+1}\right) - r_{i-1}\left(r_{i-1} + r_i\right)\right] = -6D_{\mathrm{kernel}}b\left(r_i\right)^2h = -V_iG_{\mathrm{kernel}}$, which cancels $\bar Q_i = V_iG_{\mathrm{kernel}}$. This includes $i = 1$, where $K_{1,\mathrm{inner}} = 0$ and $r_0 = 0$ make the formula hold unchanged. The centre row: $6D_{\mathrm{kernel}}\left(C_1 - C_0\right)/\left(h\right)^2 + G_{\mathrm{kernel}} = -6D_{\mathrm{kernel}}b + G_{\mathrm{kernel}} = 0$.
- *Node $N_{\mathrm{kernel}}$.* The inner flow is $D_{\mathrm{kernel}}b\,R_1\left(R_1 - h\right)\left(2R_1 - h\right) = G_{\mathrm{kernel}}\left[\frac{1}{3}\left(R_1\right)^3 - \frac{1}{2}\left(R_1\right)^2h + \frac{1}{6}R_1\left(h\right)^2\right]$, the outer flow is $\frac{1}{3}\left(R_1\right)^3G_{\mathrm{kernel}}$. The row balances if and only if $\bar Q_{N_{\mathrm{kernel}}} = G_{\mathrm{kernel}}\left[\frac{1}{2}\left(R_1\right)^2h - \frac{1}{6}R_1\left(h\right)^2\right] = \omega\left(R_1\right)^2\frac{h}{2}\,G_{\mathrm{kernel}}$, which is (7). $\square$

So the discrete steady state equals the exact one at every node, for any grid and any diffusivities, and only with $\omega$. At steady state $M^{j+1} = M^j$, so by (10) $\Phi = 4\pi\sum\bar Q_i = \frac{4\pi}{3}\left(R_1\right)^3G_{\mathrm{kernel}}$, the exact generation rate. Observed, in 50-digit arithmetic for counts $(10,10,10,10,10)$, $(7,11,3,5,13)$, $(3,1,2,1,4)$, $(1,1,1,1,1)$ with the synthetic diffusivities, two metrics kept apart. Nodal error, $\max_{0 \le i \le N-1}\left|C_i - C(r_i)\right|/C(0)$ with $C$ of (14): $2.2\times10^{-48}$ to $3.6\times10^{-46}$ with $\omega$; $0.0044$, $0.0090$, $0.049$, $0.44$ with the plain weight. Release error, $\left|\Phi/\left(\frac{4\pi}{3}\left(R_1\right)^3G_{\mathrm{kernel}}\right) - 1\right|$: at most $4.1\times10^{-46}$ with $\omega$; $0.0050$, $0.0102$, $0.0556$, $0.5$ with the plain weight, which is exactly $1/\left(2\left(n_{\mathrm{kernel}}\right)^2\right)$, as it must be, because at steady state $\Phi = 4\pi\sum_i\bar Q_i$ and item 1 of Section 1.3 gives the plain-weight sum. (Both plain-weight errors fall at second order in $1/n_{\mathrm{kernel}}$.)

**Uniform source in one material (P22a: exactness on one spacing and the local residual PROVED; global second order on unequal spacings OBSERVED, NOT PROVEN).** If all five $G$ are equal and all five $D$ are equal, (6) and (7) give $\bar Q_i = V_iG$ at every node. On a grid with one spacing $h$ throughout, the interior computation above applies at every node, including the interface nodes (where now both flows come from the same quadratic), so $C(r) = G\left(\left(R_5\right)^2 - \left(r\right)^2\right)/\left(6D\right)$ is the exact discrete steady state. On unequal spacings the interface rows leave a residual $-D\,b\,r_i\left(\left(\Delta r_{i,\mathrm{outer}}\right)^2 - \left(\Delta r_{i,\mathrm{inner}}\right)^2\right)$ (same computation with $r_{i+1} = r_i + \Delta r_{i,\mathrm{outer}}$, $r_{i-1} = r_i - \Delta r_{i,\mathrm{inner}}$), a local error of first order at isolated nodes. Observed: nodal errors at rounding level ($10^{-14}$ to $4\times10^{-12}$) with counts proportional to the thicknesses, and second order ($3.8\times10^{-3}$ down to $1.5\times10^{-5}$, orders $2.00$) with equal counts.

**Release consistency with an OPyC source (P23).** Consistency to second order is PROVED by (11) and (12). Observed in the steady state of the uniform source on the uniform grid: $\left|\Phi/\left(\frac{4\pi}{3}\left(R_5\right)^3G\right) - 1\right| = 2.5\times10^{-4}, 6.2\times10^{-5}, 1.5\times10^{-5}, 3.9\times10^{-6}, 9.6\times10^{-7}$ (order $2.00$) with the term $4\pi\bar Q_N$, against $3.3\times10^{-2}$ to $2.1\times10^{-3}$ (order $1.00$) without it.

---

## 2. G2: radioactive decay

### 2.1 Implicit decay and the matrix

Integrate $-\lambda C$ over the dual region of node $i$ with the trapezoidal weight and evaluate it at the new level, as Backward Euler does with every other term:

$$
V_i\,\frac{C_i^{j+1} - C_i^j}{\Delta t} = K_{i,\mathrm{outer}}\left(C_{i+1}^{j+1} - C_i^{j+1}\right) - K_{i,\mathrm{inner}}\left(C_i^{j+1} - C_{i-1}^{j+1}\right) + \bar Q_i^j - \lambda V_i\,C_i^{j+1}. \tag{15}
$$

Divided by $V_i$, only the diagonal changes, by $\kappa = \lambda\,\Delta t$, in every row including the centre row (where the pointwise equation gains $-\lambda C_0$) and row $N-1$:

$$
B_i = 1 + \left|L_i\right| + \left|U_i\right| + \kappa \quad \left(1 \le i \le N-1\right), \qquad B_0 = 1 + 6\,\mathrm{Fo}_{\mathrm{kernel}} + \kappa. \tag{16}
$$

$L_i$ and $U_i$ are those of (F11) and (F12), unchanged.

**The matrix is still a non-singular M-matrix (P24, PROVED).** The off-diagonal entries are those of the baseline, so $L_i \le 0$ and $U_i \le 0$. The diagonal is larger by $\kappa \ge 0$. Every row therefore satisfies

$$
B_i - \left|L_i\right| - \left|U_i\right|_{\text{in matrix}} \ge 1 + \kappa \ge 1, \tag{17}
$$

the margin (D1 Eq. (45)) raised by $\kappa$. The proofs of D1 Section 9.2 use only the sign pattern and the margin, so they apply with $1$ replaced by $1 + \kappa$:

- (i) If $\mathbf{A}\mathbf{x} = \mathbf{b}$ and $\left|x_k\right| = \max_i\left|x_i\right|$, row $k$ gives $\left(1 + \kappa\right)\left|x_k\right| \le \left(B_k - \left|L_k\right| - \left|U_k\right|\right)\left|x_k\right| \le \left|b_k\right|$, so $\left\|\mathbf{x}\right\|_\infty \le \left\|\mathbf{b}\right\|_\infty/\left(1 + \kappa\right)$. With $\mathbf{b} = \mathbf{0}$ this forces $\mathbf{x} = \mathbf{0}$: $\mathbf{A}$ is non-singular.
- (ii) Write $\mathbf{A} = \mathbf{W}\left(\mathbf{I} - \mathbf{H}\right)$ with the diagonal matrix $\mathbf{W} = \mathrm{diag}\left(B_i\right)$ (dimensionless). $\mathbf{H}$ is entrywise non-negative with row sums $\left(\left|L_i\right| + \left|U_i\right|_{\text{in matrix}}\right)/B_i \le 1 - \left(1 + \kappa\right)/B_i < 1$, so the Neumann series converges, $\mathbf{A}^{-1} = \sum_{\ell \ge 0}\left(\mathbf{H}\right)^{\ell}\left(\mathbf{W}\right)^{-1} \ge 0$ entrywise ($\ell$ is the power of the series), and

$$
\left\|\mathbf{A}^{-1}\right\|_\infty \le \frac{1}{1 + \kappa}. \tag{18}
$$

  A matrix with non-positive off-diagonal entries and a non-negative inverse is a non-singular M-matrix.
- (iii) Thomas pivots: $m_0 = B_0 = 1 + \kappa + \left|U_0\right|$; if $m_{i-1} \ge 1 + \kappa + \left|U_{i-1}\right|$ then $\left|U_{i-1}\right|/m_{i-1} < 1$ and $m_i = B_i - \left|L_i\right|\left|U_{i-1}\right|/m_{i-1} \ge 1 + \kappa + \left|U_i\right|$. No pivoting is needed, and P6 (non-negativity in floating point for (F24)) holds unchanged, because the sign pattern is unchanged.

Observed: for a grid with counts $(6,4,3,5,4)$, $\Delta t \in \{10^{-3}, 200, 10^6\}\ \mathrm{s}$ and $\lambda \in \{0, 10^{-3}, 10\}\ \mathrm{s^{-1}}$, the smallest margin divided by $1 + \kappa$ was $1.000000$, $\left(1 + \kappa\right)\left\|\mathbf{A}^{-1}\right\|_\infty \le 1.000000$, and no entry of the computed inverse was negative.

### 2.2 Decay booked separately from release

Multiply (15) by $4\pi\,\Delta t$ and sum over $i = 1, \dots, N-1$. The decay term gives $\lambda\,\Delta t\cdot4\pi\sum_iV_iC_i^{j+1} = \kappa\,M^{j+1}$. Define

$$
M_{\mathrm{dec}}^0 = 0, \qquad M_{\mathrm{dec}}^{j+1} = M_{\mathrm{dec}}^j + \kappa\,M^{j+1}. \tag{19}
$$

$M_{\mathrm{dec}}$ is the amount that has decayed inside the particle. It is never added to $M_{\mathrm{rel}}$: the release (12) is the flow through $R_5$ only, and the decayed amount is the volume integral of $\lambda C$ at the implicit level. Nothing decays at node $N$, where $C_N = 0$, and nothing decays at node 0, which has no weight. What happens to material after it has left the particle (including its decay outside) is not modelled.

### 2.3 Exact relation to the decay-free scheme (P25, PROVED)

Let $\mathcal{K}$ be the matrix defined by $\mathbf{A}(\Delta t) = \mathbf{I} + \Delta t\,\mathcal{K}$ for the baseline matrix: $\Delta t\,\mathcal{K}$ has the entries $L_i$, $\left|L_i\right| + \left|U_i\right|$ and $U_i$ (and $6\,\mathrm{Fo}_{\mathrm{kernel}}$, $-6\,\mathrm{Fo}_{\mathrm{kernel}}$ in row 0), each proportional to $\Delta t$, so the entries of $\mathcal{K}$ (unit $\mathrm{s^{-1}}$) do not depend on $\Delta t$. With decay,

$$
\mathbf{I} + \Delta t\,\mathcal{K} + \kappa\,\mathbf{I} = \left(1 + \kappa\right)\left(\mathbf{I} + \Delta t'\,\mathcal{K}\right), \qquad \Delta t' = \frac{\Delta t}{1 + \kappa}. \tag{20}
$$

So, without a source, $\mathbf{C}^{j}_{\lambda} = \left(1 + \kappa\right)^{-j}\,\mathbf{C}^{j}_{\mathrm{base}}\left(\Delta t'\right)$: the decaying solution is the decay-free solution computed with the shorter step $\Delta t'$, multiplied by the discrete decay factor. This is the discrete form of the continuous identity $C = e^{-\lambda t}v$, where $v$ solves the decay-free problem. Consequences:

- **Decay-only limit.** With $D \to 0$, $C_i^j = \left(1 + \lambda\,\Delta t\right)^{-j}C_i^0$, which converges to $e^{-\lambda t^j}C_i^0$ at first order in $\Delta t$ (archive B, Section 9). Observed: $C_1^{30} = 0.0573085533011679$ for $\lambda\,\Delta t = 0.1$, equal to $\left(1.1\right)^{-30}$ to all printed digits; $e^{-3} = 0.0498$.
- **$\lambda = 0$ limit (PROVED).** $\kappa = 0$ adds the exact zero to each diagonal, and $B + 0 = B$ exactly: bit for bit the baseline.
- Observed: the relation (20) held in a scratch run to a relative $2.9\times10^{-13}$, rounding.

**Analytical limit with diffusion.** For the homogeneous sphere with the kernel step, the substitution $C = e^{-\lambda t}v$ and the series of the decay-free problem, $M_v(t)/M^0 = \sum_\mu w_\mu e^{-\alpha_\mu t}$ with the mode decay rate $\alpha_\mu = D\left(\mu\pi/R_5\right)^2$ (unit $\mathrm{s^{-1}}$) and the dimensionless mode weight $w_\mu = 6\left(-1\right)^{\mu+1}\left[\sin\left(\mu\pi R_1/R_5\right) - \left(\mu\pi R_1/R_5\right)\cos\left(\mu\pi R_1/R_5\right)\right]/\left(\mu\pi R_1/R_5\right)^3$, give

$$
\frac{M(t)}{M^0} = \sum_\mu w_\mu e^{-\left(\alpha_\mu + \lambda\right)t}, \qquad \frac{M_{\mathrm{rel}}(t)}{M^0} = \sum_\mu w_\mu\alpha_\mu\,\frac{1 - e^{-\left(\alpha_\mu + \lambda\right)t}}{\alpha_\mu + \lambda}, \qquad \frac{M_{\mathrm{dec}}(t)}{M^0} = \sum_\mu w_\mu\lambda\,\frac{1 - e^{-\left(\alpha_\mu + \lambda\right)t}}{\alpha_\mu + \lambda}. \tag{21}
$$

($\mu = 1, 2, \dots$ is the mode number; it is not the integer $\nu$ of $\gamma_\nu$ in Section 5.) The three add up to $\sum_\mu w_\mu = 1$, the continuous balance. Observed (five equal layers $D = 10^{-10}\ \mathrm{m^2\ s^{-1}}$, synthetic radii, $\lambda = 1/400\ \mathrm{s^{-1}}$, $t = 405\ \mathrm{s}$, where (21) gives remaining $0.0825$, released $0.475$, decayed $0.443$): with 40 intervals per layer and 50 to 800 steps the errors of all three halve with each halving of $\Delta t$ (decayed: $-2.5\times10^{-3}$ to $-1.7\times10^{-4}$; Cauchy orders $1.003$, $1.002$); with the time error removed by Richardson extrapolation, the decayed-fraction error falls from $-1.6\times10^{-4}$ ($10$ intervals per layer) to $-2.5\times10^{-6}$ ($80$), orders $1.99$, $2.00$, $2.00$. Largest balance residual $3.4\times10^{-13}$ of $M^0$.

---

## 3. G3: temperature-dependent diffusivity

### 3.1 The laws, units and range

The directive's law, with the gas constant renamed,

$$
D_{\mathrm{material}}(T) = D_{0,\mathrm{material}}\exp\left(-\frac{E_{a,\mathrm{material}}}{R_{\mathrm{gas}}T}\right), \tag{22}
$$

is the general form (decision 13). Where the register documents a row as a sum of two Arrhenius terms, the law is the sum $D_{0,1}\exp\left(-E_{a,1}/\left(R_{\mathrm{gas}}T\right)\right) + D_{0,2}\exp\left(-E_{a,2}/\left(R_{\mathrm{gas}}T\right)\right)$, exactly as documented. The mathematics below needs that each $D_{\mathrm{material}}(T)$ is positive and twice continuously differentiable in $T$ (both forms are), and that $T(t)$ is twice continuously differentiable on each step; the midpoint argument for P27 in Section 3.2 uses the second time derivative of the coefficients. Stability (P26) and the balance (P30) need only positive values. A history with a kink or a jump inside a step is treated at the end of Section 3.2.

- **Units.** $T$ in kelvin: $T = T_{\mathrm{C}} + 273.15$ when a history is given in degrees Celsius. $E_a$ in $\mathrm{J\ mol^{-1}}$: a value in $\mathrm{kJ\ mol^{-1}}$ is multiplied by $1000$. $E_a/\left(R_{\mathrm{gas}}T\right)$ is dimensionless, so $D$ has the unit of $D_0$, $\mathrm{m^2\ s^{-1}}$. If a source gives an activation energy per atom in electronvolts, $E_a/\left(R_{\mathrm{gas}}T\right)$ is replaced by $E/\left(k_{\mathrm{B}}T\right)$ with $k_{\mathrm{B}} = \left(1.380649\times10^{-23}\right)/\left(1.602176634\times10^{-19}\right)\ \mathrm{eV\ K^{-1}} = 8.617333262\ldots\times10^{-5}\ \mathrm{eV\ K^{-1}}$ (the quotient is exact by the SI definitions of the Boltzmann constant and the elementary charge; the decimal is truncated).
- **Gas constant.** A fitted pair $\left(D_0, E_a\right)$ is consistent with the value of $R_{\mathrm{gas}}$ used in the fit. Where the register documents that value with the row, the code uses it; otherwise $R_{\mathrm{gas}} = N_{\mathrm{A}}k_{\mathrm{B}}$ with $N_{\mathrm{A}} = 6.02214076\times10^{23}\ \mathrm{mol^{-1}}$ and $k_{\mathrm{B}} = 1.380649\times10^{-23}\ \mathrm{J\ K^{-1}}$, both exact by the SI definition, so $R_{\mathrm{gas}} = 8.31446261815324\ \mathrm{J\ mol^{-1}\ K^{-1}}$ exactly (this decimal is the exact product; in double precision it is rounded once, like every input).
- **Range.** A correlation is used only inside the temperature range that the register documents for it; a temperature outside it is reported, not silently extrapolated. Where the register documents no range, the result says so.

### 3.2 Time level and reassembly

The scheme needs, for each step, one diffusivity per material. The step value is the diffusivity at the temperature of the middle of the step:

$$
\bar D_{\mathrm{material}}^j = D_{\mathrm{material}}\left(T\left(t^j + \tfrac{1}{2}\Delta t\right)\right). \tag{23}
$$

With these values every coefficient that depends on $D$ is evaluated for the step: $K_{i,\mathrm{outer}}^j$ by (F6), $\mathrm{Fo}_{\mathrm{material}}^j$ by (F8), the rows (F11), (F12), (F17) with (16), and the release (12) with the same stored $K_{N-1,\mathrm{outer}}^j$. The matrix $\mathbf{A}^j$ is reassembled, and its Thomas factors recomputed, whenever $\bar D^j$ or $\Delta t$ differs from the previous step. The weights $V_i$, the initial vector (F21), (F22) and the source weights (6) to (8) do not depend on $D$ and are not recomputed. Because each step's rows are (F10) with that step's $K^j$, the telescoping of D1 Eq. (35) holds step by step with the $K^j$ of the same step in row $N-1$ and in the release (P9 per step).

**Why the middle of the step.** Write the semi-discrete system (rows divided by $V_i$, centre row included) as $\dot{\mathbf{c}} = -\mathcal{K}(t)\mathbf{c} + \mathbf{g}(t)$, where $\mathcal{K}(t)$ is linear in the diffusivities at time $t$. One step with matrix $\mathbf{I} + \Delta t\,\mathcal{K}_j$ and source $\bar{\mathbf{g}}^j$ of (13), applied to the exact semi-discrete solution, leaves the residual

$$
\boldsymbol{\rho}^j = \frac{\mathbf{c}(t^{j+1}) - \mathbf{c}(t^j)}{\Delta t} + \mathcal{K}_j\mathbf{c}(t^{j+1}) - \bar{\mathbf{g}}^j = \underbrace{\frac{1}{\Delta t}\int_{t^j}^{t^{j+1}}\left(\mathcal{K}_j - \mathcal{K}(s)\right)\mathbf{c}(s)\,ds}_{\text{coefficient term}} + \underbrace{\mathcal{K}_j\,\frac{1}{\Delta t}\int_{t^j}^{t^{j+1}}\left(\mathbf{c}(t^{j+1}) - \mathbf{c}(s)\right)ds}_{\text{Backward Euler term}}. \tag{24}
$$

The second term is the usual first-order error of Backward Euler. The first term depends on the choice of $\mathcal{K}_j$:

- **Step average** $\mathcal{K}_j = \frac{1}{\Delta t}\int\mathcal{K}(s)\,ds$: since $\int\left(\mathcal{K}_j - \mathcal{K}(s)\right)ds = 0$, the term equals $\frac{1}{\Delta t}\int\left(\mathcal{K}_j - \mathcal{K}(s)\right)\left(\mathbf{c}(s) - \mathbf{c}(t^{j+1})\right)ds$; both factors of the integrand are at most $\Delta t$ times a derivative, so the term is $O\left(\left(\Delta t\right)^2\left\|\dot{\mathcal{K}}\right\|\left\|\dot{\mathbf{c}}\right\|\right)$, of second order.
- **Middle of the step**, (23): $\mathcal{K}_j$ differs from the step average by $O\left(\left(\Delta t\right)^2\left\|\ddot{\mathcal{K}}\right\|\right)$ (midpoint rule), so the term is again of second order.
- **End of the step** $\mathcal{K}_j = \mathcal{K}(t^{j+1})$: the term is $\frac{1}{2}\Delta t\,\dot{\mathcal{K}}\,\mathbf{c} + O\left(\left(\Delta t\right)^2\right)$, an additional first-order error proportional to the rate of change of the diffusivities, $\dot D = D\,\frac{E_a}{R_{\mathrm{gas}}\left(T\right)^2}\,\dot T$ for (22).

So with (23) the change of $D$ within a step adds no first-order error; with end-of-step values it does. This, and not the size of the error in any one case, is the reason for (23): the first-order coefficient term of end-of-step values can have either sign relative to the Backward Euler term, so it can partly cancel it, and the measured ratio of the two total errors depends on the case (from $0.45$ to about $22$ in the runs below). (23) is chosen over the exact step average because the average of (22) over a linear temperature ramp is not an elementary function, and the midpoint value has the same order. Where the history holds the temperature constant over a step, (23) is $D(T)$ itself.

**Accuracy statement (P27).** Backward Euler with (23) is first order in time also when $D$ changes within a step: PROVED for a fixed grid (the residual (24) is $O(\Delta t)$ with constants that depend on the grid, on $\left\|\mathcal{K}\right\|$ and on the first two time derivatives of $T$ on the step; the error then follows from $\left\|\left(\mathbf{A}^j\right)^{-1}\right\|_\infty \le 1$ for every $j$, as in D1 Eq. (46c), $\left\|\mathbf{e}^J\right\|_\infty \le \sum_j\Delta t\left\|\boldsymbol{\rho}^j\right\|_\infty$); with constants independent of the grid, PROOF OUTLINE (it needs bounds on the time derivatives of the semi-discrete solution that are uniform in $\Delta r$, which are not proved). A temperature history with a kink or a jump inside a step makes that step's coefficient term first order instead of second, which leaves the global error first order.

Observed, case 1 (synthetic; one material on the synthetic radii, kernel step, 40 intervals per layer, $J = 25, 50, 100, 200, 400$ steps over $400\ \mathrm{s}$). The law is $D(T) = D_{1000}\exp\left(-\Theta\left(1/T - 1/\left(1000\ \mathrm{K}\right)\right)\right)$ with $\Theta = E_a/R_{\mathrm{gas}} = 10^4\ \mathrm{K}$, that is (22) with $D_0 = D_{1000}\,e^{10}$, where $D_{1000}$ (in $\mathrm{m^2\ s^{-1}}$) is the value at $1000\ \mathrm{K}$. $T$ rises linearly from $1000$ to $1400\ \mathrm{K}$, so $D$ rises by a factor $17.4$. The errors of the release fraction are taken against the exact (25), so they contain the spatial error of the grid as well:

| $D_{1000}$ ($\mathrm{m^2\ s^{-1}}$) | exact $F(400\ \mathrm{s})$ | error with (23), $J = 25 \to 400$ | Cauchy orders, (23) | error with end-of-step values, $J = 25 \to 400$ | Cauchy orders, end of step | $\lvert$end-of-step error$\rvert$ / $\lvert$(23) error$\rvert$, $J = 25$ to $400$ |
|---|---|---|---|---|---|---|
| $2.5\times10^{-11}$ | $0.93423$ | $-2.04\times10^{-2} \to -1.29\times10^{-3}$ | $0.995$, $0.999$, $1.000$ | $-9.27\times10^{-3} \to -6.25\times10^{-4}$ | $0.925$, $0.962$, $0.981$ | $0.45$ to $0.49$ |
| $1\times10^{-11}$ | $0.55236$ | $-1.76\times10^{-2} \to -1.16\times10^{-3}$ | $0.933$, $0.965$, $0.982$ | $+8.75\times10^{-3} \to +5.65\times10^{-4}$ | $0.967$, $0.984$, $0.992$ | $0.49$ to $0.50$ |
| $3\times10^{-12}$ | $0.058606$ | $+5.52\times10^{-3} \to +4.13\times10^{-4}$ | $0.898$, $0.949$, $0.974$ | $+1.45\times10^{-2} \to +9.52\times10^{-4}$ | $0.988$, $0.994$, $0.997$ | $2.6$ to $2.3$ |
| $1\times10^{-12}$ | $3.2888\times10^{-4}$ | $+4.34\times10^{-4} \to +2.68\times10^{-5}$ | $1.111$, $1.070$, $1.039$ | $+6.75\times10^{-4} \to +3.47\times10^{-5}$ | $1.272$, $1.159$, $1.087$ | $1.6$ to $1.3$ |

With the exact step average (20-point Gauss-Legendre quadrature in each step) the errors were within $1.1\,\%$ of those with (23) in all four cases. Case 2 (synthetic; five layers, 20 intervals per layer, $D_{\mathrm{material}}(1000\ \mathrm{K})$ equal to the five synthetic diffusivities $10^{-12}$, $4\times10^{-10}$, $8\times10^{-13}$, $2\times10^{-14}$, $7\times10^{-13}\ \mathrm{m^2\ s^{-1}}$, $\Theta = 10^4$, $2\times10^4$, $1.5\times10^4$, $3\times10^4$, $1.5\times10^4\ \mathrm{K}$, $T$ rising linearly from $900$ to $1000\ \mathrm{K}$ over $2\times10^5\ \mathrm{s}$, $J = 100$ to $1600$): Cauchy orders $1.011$, $1.005$, $1.002$ with (23) and $1.000$, $1.000$, $1.000$ with end-of-step values; the Richardson-extrapolated release fractions of the two sequences, $0.1175241$ and $0.1175240$, agree, and against them the time error at $J = 800$ is $-1.46\times10^{-5}$ with (23) and $+3.18\times10^{-4}$ with end-of-step values, a ratio of about $22$. Both levels are first order in every case; which one has the smaller error depends on the case.

### 3.3 The homogeneous sphere: exact change of time variable (P28, PROVED)

If all five diffusivities are equal at every time, $D(r, T(t)) = D(t)$, the substitution $\tau(t) = \int_0^tD(s)\,ds$ (unit $\mathrm{m^2}$) turns Eq. (1) without source and decay into the equation with unit diffusivity in the variable $\tau$, so

$$
C(r, t) = C_{\mathrm{unit}}\left(r, \tau(t)\right), \tag{25}
$$

with $C_{\mathrm{unit}}(r, \tau)$ (in $\mathrm{mol\ m^{-3}}$) the solution for unit diffusivity, for which the kernel-step series of Section 2.3 is known ($\alpha_\mu$ with $D = 1$, unit $\mathrm{m^{-2}}$). Discretely, $\mathbf{A}^j = \mathbf{I} + \Delta t\,\bar D^j\mathcal{K}_{\mathrm{unit}}$, where $\mathcal{K}_{\mathrm{unit}}$ (unit $\mathrm{m^{-2}}$) is the spatial operator for unit diffusivity: the step is exactly the constant-coefficient Backward Euler step for $C_{\mathrm{unit}}$ with step $\Delta\tau^j = \Delta t\,\bar D^j$. Hence the scheme computes the unit-diffusivity scheme on the variable steps $\Delta\tau^j$. With the exact step average, $\sum_{j<J}\Delta\tau^j = \tau(t^J)$, and the temporal error is that of constant-coefficient Backward Euler on those steps, whatever the rate of change of $T$. With (23), $\sum\Delta\tau^j$ differs from $\tau(t^J)$ by the error of the composite midpoint rule, $O\left(\left(\Delta t\right)^2\right)$. Observed: the step-average run and the unit-diffusivity run on the steps $\Delta t\,\bar D^j$ agreed to $4.4\times10^{-16}$ (absolute, rounding). With unequal diffusivities whose ratios change in time (different activation energies) no such change of variable exists, and no series reference is available (verification plan, Section 8).

### 3.4 Stability with time-dependent coefficients (P26, PROVED)

Each $\mathbf{A}^j$ is a matrix of the form of D2 with positive diffusivities $\bar D^j$, so P1, P2, P5, P6 and (17), (18) hold for every $j$. The two norms of P3 do not depend on $D$: the maximum norm trivially, and $\left\|\mathbf{C}\right\|_V^2 = \sum_{i=1}^{N-1}V_i\left(C_i\right)^2$ because $V_i$ depends only on the grid. Hence the bounds of P3 hold step by step and chain over any sequence of temperatures: $\left\|\mathbf{C}^{j+1}\right\|_\infty \le \left\|\mathbf{C}^j\right\|_\infty$ and $\left\|\mathbf{C}^{j+1}\right\|_V \le \left\|\mathbf{C}^j\right\|_V$ without source and decay, for every $\Delta t$ and every history $T(t)$. A norm that depended on $D$ (for example the energy $\left|\cdot\right|_K$) would not chain; none is used.

### 3.5 Equivalence with the constant case (P29, PROVED)

If $T$ is constant, (23) gives $\bar D^j = D_{\mathrm{material}}(T)$ for every $j$, every coefficient equals the constant-case coefficient computed from that value, and the run is the constant-diffusivity run with $D_{\mathrm{material}} = D_{\mathrm{material}}(T)$, in exact arithmetic. In floating point the two runs are bit for bit identical provided (a) the constant-case run is given the same double-precision numbers $D_{\mathrm{material}}(T)$, computed by the same function, and (b) both runs assemble the matrix by the same code with the same operations; then each step performs the same IEEE operations on the same operands. The temperature path must not, for example, compute $D$ through a different but mathematically equal expression. Observed: identical arrays in a scratch run. Between native code and WebAssembly, $\exp$ may differ in the last bit between libraries (verification plan, test H), so bit identity across platforms is not claimed for G3.

---

## 4. G4: the combined scheme

### 4.1 Rows

For $1 \le i \le N-1$ the row of the step from $t^j$ to $t^{j+1}$ is (15) with the coefficients of step $j$:

$$
V_i\,\frac{C_i^{j+1} - C_i^j}{\Delta t} = K_{i,\mathrm{outer}}^j\left(C_{i+1}^{j+1} - C_i^{j+1}\right) - K_{i,\mathrm{inner}}^j\left(C_i^{j+1} - C_{i-1}^{j+1}\right) + \bar Q_i^j - \lambda V_i\,C_i^{j+1}, \tag{26}
$$

that is $L_i^jC_{i-1}^{j+1} + B_i^jC_i^{j+1} + U_i^jC_{i+1}^{j+1} = C_i^j + \Delta t\,g_i^j$ with $L_i^j = -\Delta t\,K_{i,\mathrm{inner}}^j/V_i$, $U_i^j = -\Delta t\,K_{i,\mathrm{outer}}^j/V_i$, $B_i^j = 1 + \left|L_i^j\right| + \left|U_i^j\right| + \kappa$. Row 0: $\left(1 + 6\,\mathrm{Fo}_{\mathrm{kernel}}^j + \kappa\right)C_0^{j+1} - 6\,\mathrm{Fo}_{\mathrm{kernel}}^jC_1^{j+1} = C_0^j + \Delta t\,\bar G_{\mathrm{kernel}}^j$. $C_N^{j+1} = 0$. These are (F37) and (F38) of D2.

### 4.2 Discrete balance (P30, PROVED)

**Definitions, all in the discrete convention.**

| Term | Definition | Unit |
|---|---|---|
| initial $M^0$ | (F25) of the start vector (F21); $= \frac{4\pi}{3}\left(R_1\right)^3C_{\mathrm{init}}$ by P10 | $\mathrm{mol}$ |
| remaining $M^j$ | (F25), $4\pi\sum_{i=1}^{N-1}V_iC_i^j$ | $\mathrm{mol}$ |
| generated $M_{\mathrm{gen}}^j$ | $\sum_{k=0}^{j-1}4\pi\,\Delta t\sum_{i=1}^{N}\bar Q_i^k$, (12) | $\mathrm{mol}$ |
| released $M_{\mathrm{rel}}^j$ | $\sum_{k=1}^{j}\Delta t\,\Phi^k$, with $\Phi^k$ of (12) | $\mathrm{mol}$ |
| decayed $M_{\mathrm{dec}}^j$ | $\sum_{k=1}^{j}\kappa\,M^k$, (19) | $\mathrm{mol}$ |
| residual $\varepsilon^j$ | $M^j + M_{\mathrm{rel}}^j + M_{\mathrm{dec}}^j - M^0 - M_{\mathrm{gen}}^j$ | $\mathrm{mol}$ |

Here $k$ is a step index of a sum. With a variable step, $\Delta t$ and $\kappa = \lambda\,\Delta t$ in each term are those of the step that produced it.

**Claim.** $M^0 + M_{\mathrm{gen}}^j = M^j + M_{\mathrm{rel}}^j + M_{\mathrm{dec}}^j + \varepsilon^j$ with $\varepsilon^j = 0$ for every $j$, in exact arithmetic, for every grid, step, temperature history, source and decay constant.

**Proof.** Multiply (26) by $4\pi\,\Delta t$ and sum over $i = 1, \dots, N-1$. On the left: $M^{j+1} - M^j$. The flows telescope (D1 Eq. (21)): the term $K_{i,\mathrm{outer}}^j\left(C_{i+1} - C_i\right)$ of row $i$ cancels $-K_{i+1,\mathrm{inner}}^j\left(C_{i+1} - C_i\right)$ of row $i+1$, because $K_{i+1,\mathrm{inner}}^j = K_{i,\mathrm{outer}}^j$ is one stored number; $K_{1,\mathrm{inner}}^j = 0$; what remains is $-4\pi\,\Delta t\,K_{N-1,\mathrm{outer}}^jC_{N-1}^{j+1}$, since $C_N^{j+1} = 0$. The source gives $4\pi\,\Delta t\sum_{i=1}^{N-1}\bar Q_i^j$ and the decay $-\kappa M^{j+1}$. Adding and subtracting $4\pi\,\Delta t\,\bar Q_N^j$:

$$
M^{j+1} - M^j = -\Delta t\,\Phi^{j+1} + \left(M_{\mathrm{gen}}^{j+1} - M_{\mathrm{gen}}^j\right) - \left(M_{\mathrm{dec}}^{j+1} - M_{\mathrm{dec}}^j\right). \tag{27}
$$

Summing (27) over $j = 0, \dots, J-1$ telescopes the left side to $M^J - M^0$ and the right side to $-M_{\mathrm{rel}}^J + M_{\mathrm{gen}}^J - M_{\mathrm{dec}}^J$, with $M_{\mathrm{rel}}^0 = M_{\mathrm{gen}}^0 = M_{\mathrm{dec}}^0 = 0$. $\square$

The hypotheses are those of P9, per step: the same stored $K^j$ in rows $i$ and $i+1$ and in the release; $V_i$ the weight by which row $i$ was divided; the decay amount computed with the same $\kappa$ as the diagonal and with the inventory at level $j+1$; the generated amount computed from the same $\bar Q_i^j$ as the right-hand side, including $\bar Q_N^j$ in both (12) sums. The centre row does not enter. In floating point $\varepsilon^j$ is rounding; Section 5.8 bounds it. Observed: $\left|\varepsilon^j\right|/\left(M^0 + M_{\mathrm{gen}}^j\right) \le 4.2\times10^{-10}$ in the combined scratch runs of Section 5.8 (synthetic case, 40 and 80 intervals per layer, source, decay and a temperature ramp, 1000 steps), always below the bound (41).

### 4.3 Positivity, bounds, solvability and stability of the combined matrix (P31, PROVED)

- **Solvability.** $\mathbf{A}^j$ is a non-singular M-matrix for every $j$, with $\left\|\left(\mathbf{A}^j\right)^{-1}\right\|_\infty \le 1/\left(1 + \kappa\right)$ and Thomas pivots $m_i \ge 1 + \kappa + \left|U_i^j\right|$ (Section 2.1, applied to the coefficients of step $j$). No condition on $\Delta t$, the grid, the diffusivity ratios, $\lambda$ or $T(t)$.
- **Positivity.** $\mathbf{C}^j \ge \mathbf{0}$ and $G \ge 0$ give $\mathbf{b}^j \ge \mathbf{0}$ and $\mathbf{C}^{j+1} = \left(\mathbf{A}^j\right)^{-1}\mathbf{b}^j \ge \mathbf{0}$. In floating point P6 applies to (F24): $C + \Delta t\,g$ and every operation of (F24) act on non-negative operands, provided every computed pivot is positive.
- **Maximum norm.** With $g_{\max}^j = \max_{0 \le i \le N-1}g_i^j \le \max_{\mathrm{material}}\bar G_{\mathrm{material}}^j$ (remark after (9)), (i) of Section 2.1 gives

$$
\left\|\mathbf{C}^{j+1}\right\|_\infty \le \frac{\left\|\mathbf{C}^j\right\|_\infty + \Delta t\,g_{\max}^j}{1 + \kappa}. \tag{28}
$$

  Without decay, $\left\|\mathbf{C}^j\right\|_\infty \le C_{\mathrm{init}} + \sum_{k<j}\Delta t\,g_{\max}^k$. With decay and a bounded source, $g_{\max}^j \le g_*$ for all $j$, induction on (28) gives $\left\|\mathbf{C}^j\right\|_\infty \le \max\left(C_{\mathrm{init}}, g_*/\lambda\right)$: if $\left\|\mathbf{C}^j\right\|_\infty \le X = \max\left(C_{\mathrm{init}}, g_*/\lambda\right)$, then $\left(X + \Delta t\,g_*\right)/\left(1 + \lambda\,\Delta t\right) \le \left(X + \lambda\,\Delta t\,X\right)/\left(1 + \lambda\,\Delta t\right) = X$. This is the discrete counterpart of the bound $G/\lambda$ of the decay-source equilibrium.
- **Weighted norm.** Multiply (26) by $C_i^{j+1}$ and sum over $i = 1, \dots, N-1$. Summation by parts as in D1 Eq. (46b), with $g_i = \bar Q_i/V_i$:

$$
\sum_iV_i\left(C_i^{j+1} - C_i^j\right)C_i^{j+1} + \Delta t\sum_{i=0}^{N-1}K_{i,\mathrm{outer}}^j\left(C_{i+1}^{j+1} - C_i^{j+1}\right)^2 + \kappa\left\|\mathbf{C}^{j+1}\right\|_V^2 = \Delta t\sum_iV_ig_i^jC_i^{j+1}.
$$

  Dropping the non-negative middle term and using the Cauchy-Schwarz inequality in the weighted inner product, $\left(1 + \kappa\right)\left\|\mathbf{C}^{j+1}\right\|_V^2 \le \left\langle\mathbf{C}^j + \Delta t\,\mathbf{g}^j, \mathbf{C}^{j+1}\right\rangle_V \le \left(\left\|\mathbf{C}^j\right\|_V + \Delta t\left\|\mathbf{g}^j\right\|_V\right)\left\|\mathbf{C}^{j+1}\right\|_V$, so

$$
\left\|\mathbf{C}^{j+1}\right\|_V \le \frac{\left\|\mathbf{C}^j\right\|_V + \Delta t\left\|\mathbf{g}^j\right\|_V}{1 + \kappa}. \tag{29}
$$

- **Release and inventory.** $\Phi^{j+1} \ge 0$, so $M_{\mathrm{rel}}^j$ is non-decreasing; $M^j \ge 0$ and $M_{\mathrm{dec}}^j \ge 0$, so by (27) $M_{\mathrm{rel}}^j \le M^0 + M_{\mathrm{gen}}^j$. $M^j$ is not monotone when $G > 0$.

Observed: minimum concentration exactly $0$ and $M_{\mathrm{rel}}$ non-decreasing in every combined scratch run.

### 4.4 The release fraction, and the case $C_{\mathrm{init}} = 0$ (decision 8)

(F28), $F^j = M_{\mathrm{rel}}^j/M^0$, needs $M^0 > 0$, that is $C_{\mathrm{init}} > 0$. With a source it can exceed $1$, because more material than $M^0$ can be released. With $C_{\mathrm{init}} = 0$ it is undefined. Two quantities are defined instead (D2, (F42)):

$$
F_{\mathrm{gen}}^j = \frac{M_{\mathrm{rel}}^j}{M^0 + M_{\mathrm{gen}}^j} \quad \text{where } M^0 + M_{\mathrm{gen}}^j > 0, \qquad \beta^{j+1} = \frac{\Phi^{j+1}}{4\pi\sum_{i=1}^{N}\bar Q_i^j} \quad \text{where the denominator is } > 0. \tag{30}
$$

- $F_{\mathrm{gen}}^j$ is the fraction of all material ever present in the particle (initial plus generated) that has left it by $t^j$. By Section 4.3, $0 \le F_{\mathrm{gen}}^j \le 1$ (PROVED). It equals $F^j$ when $G = 0$. It is not necessarily monotone, because its denominator grows. Observed: $F_{\mathrm{gen}}$ within $[0, 1]$ in the combined runs.
- $\beta^{j+1}$ is the ratio of the release rate to the generation rate of the step. At a steady state without decay it is $1$, by (27) with $M^{j+1} = M^j$. It is defined only while the source is positive.
- **Rule for $C_{\mathrm{init}} = 0$.** A run with $C_{\mathrm{init}} = 0$ is meaningful when some $\bar G^j > 0$. Then $F_{\mathrm{gen}}^j$ is reported from the first step at which $M_{\mathrm{gen}}^j > 0$ and is absent before it (like $\Phi^0$, decision 8). A run with $C_{\mathrm{init}} = 0$ and no source has the solution zero and nothing to normalise; it stays rejected. The residual is reported relative to $M^0 + M_{\mathrm{gen}}^j$.
- Which normalisation a measured quantity must be compared with (for example a fraction of the end-of-irradiation inventory, which the kernel-only start represents, or a release-to-birth ratio) is a decision of the experimental comparison, not of this document.

### 4.5 Analytical limits each mechanism must reproduce

| Mechanism | Limit | Reference | Status of the statement | Scratch result |
|---|---|---|---|---|
| G1 | zero source | baseline run bit for bit | PROVED (P19) | identical |
| G1 | kernel-only source, total generated amount | $\frac{4\pi}{3}\left(R_1\right)^3\int_0^tG_{\mathrm{kernel}}\,dt$ exactly | PROVED (P20) | $0$ to $2.2\times10^{-16}$ relative, four grids |
| G1 | no diffusion | $C_{\mathrm{init}} + \int G\,dt$ in the kernel, shape (F21) | PROVED (P21) | $2.2\times10^{-16}$ at node $N_{\mathrm{kernel}}$ |
| G1 | steady state, kernel-only source, five layers | (14) at every node; $\Phi = \frac{4\pi}{3}\left(R_1\right)^3G_{\mathrm{kernel}}$ | PROVED (P22) | nodal error $\le 3.6\times10^{-46}$ of $C(0)$, release error $\le 4.1\times10^{-46}$, in 50 digits |
| G1 | steady state, uniform source, one material | $G\left(\left(R_5\right)^2 - \left(r\right)^2\right)/\left(6D\right)$ | exact on a grid with one spacing: PROVED (P22a); local first-order row residual at spacing changes: PROVED (P22a); global second order on unequal spacings: OBSERVED, NOT PROVEN | rounding on one spacing; orders $2.00$ on unequal spacings |
| G1 | transient kernel-only source, homogeneous sphere | Duhamel: $M(t) = \frac{4\pi}{3}\left(R_1\right)^3G_{\mathrm{kernel}}\sum_\mu w_\mu\left(1 - e^{-\alpha_\mu t}\right)/\alpha_\mu$ | reference PROVED (superposition of the kernel-step series); convergence OBSERVED | space orders $1.99$, $2.00$, $2.00$; time ratio $1.998$ (order 1); residual $\le 5.3\times10^{-13}$ of $M_{\mathrm{gen}}$ |
| G1 | OPyC source, release | second order with $4\pi\bar Q_N$, first without | consistency PROVED (P23) | orders $2.00$ and $1.00$ |
| G2 | $\lambda = 0$ | baseline bit for bit | PROVED | |
| G2 | no diffusion | $\left(1 + \lambda\,\Delta t\right)^{-j} \to e^{-\lambda t}$, first order | PROVED | exact to printed digits |
| G2 | exact discrete relation | (20) | PROVED (P25) | $2.9\times10^{-13}$ |
| G2 | homogeneous sphere, kernel step | (21) | reference PROVED; convergence OBSERVED | time order 1, space order 2 |
| G2 + G1 | steady state, uniform source and decay, one material | $\frac{G}{\lambda}\left(1 - \frac{R_5\sinh\left(br\right)}{r\sinh\left(bR_5\right)}\right)$, $b = \left(\lambda/D\right)^{1/2}$ (unit $\mathrm{m^{-1}}$) | reference PROVED (it solves $D\left(C'' + 2C'/r\right) - \lambda C + G = 0$, is regular at $0$ and vanishes at $R_5$); convergence OBSERVED | orders $2.00$ ($bR_5 = 4.5$, one spacing) |
| G3 | constant temperature | constant-$D$ run bit for bit | PROVED (P29) | identical |
| G3 | homogeneous sphere, $T(t)$ | $C_{\mathrm{unit}}\left(r, \tau(t)\right)$, (25), with the kernel-step series | PROVED (P28) | first order, errors in Section 3.2 |
| G4 | balance | (27) | PROVED (P30) | $\le 4.2\times10^{-10}$ of $M^0 + M_{\mathrm{gen}}$, below (41) |

"Verify each mechanism independently before coupling" (directive G4) is served by the rows G1, G2 and G3, each of which switches the other two mechanisms off.

---

## 5. (R) Rounding analysis of the balance residual of the baseline scheme

D2 Section 7.4 left the tolerance on $\left|\varepsilon^j\right|/M^0$ to a rounding analysis in which the largest mesh Fourier number and the number of steps enter. The first Rust tests used an a-posteriori bound with the constant $16u$ per step, which had no derivation. The tolerance task on `feat/btcs-solver` (coordinator decision 35) replaced it by a derived a-posteriori bound with the per-step constant $c_{\mathrm{step}} = \kappa_t + 3u/\left(1 - 3u\right)$, about $7u$, where $\kappa_t = 4u + O\left(\left(u\right)^2\right)$ is the backward-error constant of the Thomas solve, and with no safety factor; the production time loop computes that bound in every output row. This section derives the per-step constant independently, states two rigorous bounds for the baseline, one a posteriori and one a priori, compares the a-posteriori bound with the Rust bound, and checks both against the measured residuals; Section 5.8 states the a-posteriori bound (41) of the combined scheme. The per-step constant derived here, $c_{\mathbf{A}}$ of (35), equals $c_{\mathrm{step}}$ to first order in $u$; the difference is of order $\left(u\right)^2$. Hats denote computed floating-point numbers.

### 5.1 Model and hypotheses

- **(FP)** IEEE double precision with rounding to nearest: for $\circ \in \{+, -, \times, /\}$, $\mathrm{fl}\left(a \circ b\right) = \left(a \circ b\right)\left(1 + \delta\right)$ with $\left|\delta\right| \le u = 2^{-53}$.
- **(U)** No operation overflows, and no multiplication or division has a subnormal non-zero result. (Additions and subtractions obey (FP) also with subnormal results.) Under gradual underflow a product may carry an absolute error of up to $2^{-1075}$ instead; that case is not bounded here. Observed: no subnormal value occurred in the synthetic runs checked (20 and 40 intervals per layer over 1000 steps, 320 over 60 steps, one large step); the smallest positive concentration was $2.2\times10^{-18}$.
- **(C)** No fused multiply-add: each $a \times b + c$ is two rounded operations. Rust does not contract floating-point expressions unless `mul_add` is called.
- **(P)** Every computed Thomas pivot $\hat m_i$ is positive. The code checks this (P6 needs it too); it is not proved a priori for large $\mathrm{Fo}$.

**Lemma 1.** If $\left|\delta_k\right| \le u$ and $\sigma_k = \pm1$ for $k = 1, \dots, \nu$, and $\nu u < 1$, then $\prod_k\left(1 + \delta_k\right)^{\sigma_k} = 1 + \theta_\nu$ with $\left|\theta_\nu\right| \le \gamma_\nu$. *Proof.* The product lies between $\left(1 - u\right)^\nu \ge 1 - \nu u$ and $\left(1 - u\right)^{-\nu} \le 1/\left(1 - \nu u\right)$ (Bernoulli's inequality), and $1 - \nu u \ge 1 - \gamma_\nu$, $1/\left(1 - \nu u\right) = 1 + \gamma_\nu$. $\square$

**Lemma 2.** $\gamma_a + \gamma_b + \gamma_a\gamma_b \le \gamma_{a+b}$, hence $\gamma_a + \gamma_b \le \gamma_{a+b}$. *Proof.* The left side is $\left(1 - au\right)^{-1}\left(1 - bu\right)^{-1} - 1 \le \left(1 - \left(a + b\right)u\right)^{-1} - 1$. $\square$

**Lemma 3.** A sum of $\nu$ non-negative numbers, each of which carries a relative error of at most $\gamma_\nu$, carries a relative error of at most $\gamma_\nu$. (Immediate, because no cancellation occurs.)

### 5.2 Stored coefficients and the exact reference matrix

The code computes $\hat K_{i,\mathrm{outer}}$ and $\hat V_i$ once; they are the stored numbers, and $\hat K_{i+1,\mathrm{inner}}$ is the same stored number as $\hat K_{i,\mathrm{outer}}$ (D2, P9). The assembly forms the product $\hat P_i = \mathrm{fl}\left(-\Delta t\,\hat K_{i,\mathrm{outer}}\right)$ (unit $\mathrm{m^3}$, $\hat P_i \le 0$), then $\hat L_i = \mathrm{fl}\left(\hat P_{i-1}/\hat V_i\right)$, $\hat U_i = \mathrm{fl}\left(\hat P_i/\hat V_i\right)$ and $\hat B_i = \mathrm{fl}\left(\mathrm{fl}\left(1 - \hat L_i\right) - \hat U_i\right)$. The product $-\Delta t\,\hat K_{i,\mathrm{outer}}$ has the same operands in row $i$ (where it gives $\hat U_i$) and in row $i+1$ (where it gives $\hat L_{i+1}$), so it is the same rounded number $\hat P_i$ in both rows, whether the code stores it or computes it twice; $\hat P_0 = 0$ because $\hat K_{0,\mathrm{outer}} = 0$. Define the real numbers $K_i' = -\hat P_i/\Delta t$ (unit $\mathrm{m^3\ s^{-1}}$); by (FP), $K_i' = \hat K_{i,\mathrm{outer}}\left(1 + \delta\right)$ with $\left|\delta\right| \le u$. Define the reference matrix $\mathbf{A}^*$ from the stored numbers in exact arithmetic: $L_i^* = \hat P_{i-1}/\hat V_i$, $U_i^* = \hat P_i/\hat V_i$, $B_i^* = 1 + \left|L_i^*\right| + \left|U_i^*\right|$ for $1 \le i \le N-1$. For $\mathbf{A}^*$ the telescoping of D1 Eq. (21) is an exact identity: for every vector $\mathbf{x}$ with $x_N = 0$,

$$
\sum_{i=1}^{N-1}\hat V_i\left(\mathbf{A}^*\mathbf{x}\right)_i = \sum_{i=1}^{N-1}\hat V_ix_i + \Delta t\,K_{N-1}'\,x_{N-1}, \tag{31}
$$

because $\hat V_iL_i^* = \hat P_{i-1}$ and $\hat V_iU_i^* = \hat P_i$ exactly, and $\hat P_0 = 0$. Rows $1$ to $N-1$ do not involve $x_0$.

Against $\mathbf{A}^*$, each stored off-diagonal entry carries one rounding: $\hat L_i = L_i^*\left(1 + \delta\right)$, $\hat U_i = U_i^*\left(1 + \delta'\right)$. Both additions in $\hat B_i$ act on non-negative numbers, so $\hat B_i = \left(1 + \left|\hat L_i\right| + \left|\hat U_i\right|\right)\left(1 + \theta_2\right)$ (Lemmas 1 and 3), and $1 + \left|\hat L_i\right| + \left|\hat U_i\right| = B_i^*\left(1 + \theta_1\right)$ (a sum of non-negative terms, each with a factor in $\left[1 - u, 1 + u\right]$); by Lemma 2, $\hat B_i = B_i^*\left(1 + \theta_3\right)$. So, entrywise,

$$
\hat{\mathbf{A}} = \mathbf{A}^*\circ\left(\mathbf{1} + \boldsymbol{\Theta}\right), \qquad \left|\boldsymbol{\Theta}\right| \le \gamma_3, \qquad \left|\hat{\mathbf{A}} - \mathbf{A}^*\right| \le \gamma_3\left|\mathbf{A}^*\right|, \qquad \left|\mathbf{A}^*\right| \le \frac{\left|\hat{\mathbf{A}}\right|}{1 - \gamma_3}. \tag{32}
$$

**Correction of the first version.** The first version of this section built the reference from $\hat K$ itself, $L_i^* = -\Delta t\,\hat K_{i,\mathrm{inner}}/\hat V_i$. Against that reference $\hat L_i$ and $\hat U_i$ carry two roundings and $\hat B_i$ four, which gave $\gamma_4$ in (32) and the constant $\gamma_{10}/\left(1 - \gamma_4\right) \approx 10u$ in (35). That constant was valid but not sharp: it counted the rounding of the product $-\Delta t\,\hat K_{i,\mathrm{outer}}$ in every entry, although it is one shared number in the two rows that use it. With the reference built from $\hat P_i$ that rounding leaves the matrix and enters once, in the release term, where it costs $u$ on $\hat M_{\mathrm{rel}}$ (Section 5.5). The derivation of the Rust bound uses the same reference (the exact-form matrix with the data $K' = P/\left(-\Delta t\right)$, item 2 of `balance_bound_history` in `tests/btcs_verification.rs`).

### 5.3 Backward error of the Thomas solve

Write the elimination of (F24) as a factorisation $\hat{\mathbf{A}} \approx \mathcal{L}\,\mathcal{U}$: $\mathcal{L}$ is unit lower bidiagonal with sub-diagonal $\hat w_i$, $\mathcal{U}$ is upper bidiagonal with diagonal $\hat m_i$ and super-diagonal the stored $\hat U_i$.

- *Factorisation.* $\hat w_i = \left(\hat L_i/\hat m_{i-1}\right)\left(1 + \delta\right)$, so $\left(\mathcal{L}\mathcal{U}\right)_{i,i-1} = \hat w_i\hat m_{i-1} = \hat L_i + \hat L_i\delta$, an error of at most $u\left|\hat L_i\right| \le \gamma_1\left|\hat w_i\hat m_{i-1}\right|$. $\hat m_i = \left(\hat B_i - \hat w_i\hat U_{i-1}\left(1 + \delta_1\right)\right)\left(1 + \delta_2\right)$, so $\left(\mathcal{L}\mathcal{U}\right)_{ii} = \hat w_i\hat U_{i-1} + \hat m_i = \hat B_i + \hat m_i\left(1 - \left(1 + \delta_2\right)^{-1}\right) - \hat w_i\hat U_{i-1}\delta_1$, an error of at most $\gamma_1\left(\left|\hat m_i\right| + \left|\hat w_i\hat U_{i-1}\right|\right)$. The super-diagonal is exact. Hence $\mathcal{L}\mathcal{U} = \hat{\mathbf{A}} + \boldsymbol{\Delta}_1$ with $\left|\boldsymbol{\Delta}_1\right| \le \gamma_1\left|\mathcal{L}\right|\left|\mathcal{U}\right|$.
- *Forward sweep* $b'_i = \mathrm{fl}\left(b_i - \mathrm{fl}\left(\hat w_ib'_{i-1}\right)\right)$: $b_i = b'_i\left(1 + \delta_2\right)^{-1} + \hat w_ib'_{i-1}\left(1 + \delta_1\right)$, so $\left(\mathcal{L} + \boldsymbol{\Delta}_{\mathcal{L}}\right)\mathbf{b}' = \mathbf{b}$ with $\left|\boldsymbol{\Delta}_{\mathcal{L}}\right| \le \gamma_1\left|\mathcal{L}\right|$.
- *Back substitution* $\hat x_i = \mathrm{fl}\left(\mathrm{fl}\left(b'_i - \mathrm{fl}\left(\hat U_i\hat x_{i+1}\right)\right)/\hat m_i\right)$: $b'_i = \hat m_i\hat x_i\left(1 + \delta_2\right)^{-1}\left(1 + \delta_3\right)^{-1} + \hat U_i\hat x_{i+1}\left(1 + \delta_1\right)$, so $\left(\mathcal{U} + \boldsymbol{\Delta}_{\mathcal{U}}\right)\hat{\mathbf{x}} = \mathbf{b}'$ with $\left|\boldsymbol{\Delta}_{\mathcal{U}}\right| \le \gamma_2\left|\mathcal{U}\right|$.

Multiplying, $\mathbf{b} = \left(\hat{\mathbf{A}} + \boldsymbol{\Delta}_1 + \boldsymbol{\Delta}_{\mathcal{L}}\mathcal{U} + \mathcal{L}\boldsymbol{\Delta}_{\mathcal{U}} + \boldsymbol{\Delta}_{\mathcal{L}}\boldsymbol{\Delta}_{\mathcal{U}}\right)\hat{\mathbf{x}}$, and by Lemma 2

$$
\left(\hat{\mathbf{A}} + \boldsymbol{\Delta}\mathbf{A}\right)\hat{\mathbf{x}} = \mathbf{b}, \qquad \left|\boldsymbol{\Delta}\mathbf{A}\right| \le \left(\gamma_1 + \gamma_1 + \gamma_2 + \gamma_1\gamma_2\right)\left|\mathcal{L}\right|\left|\mathcal{U}\right| \le \gamma_4\left|\mathcal{L}\right|\left|\mathcal{U}\right|. \tag{33}
$$

**The sign pattern removes $\left|\mathcal{L}\right|\left|\mathcal{U}\right|$.** For a general matrix $\left|\mathcal{L}\right|\left|\mathcal{U}\right|$ can be much larger than $\left|\hat{\mathbf{A}}\right|$. Here, with (P), $\hat w_i \le 0$ (rounding keeps the sign of $\hat L_i/\hat m_{i-1}$), $\hat U_i \le 0$ and $\hat m_i > 0$. The entries of $\left|\mathcal{L}\right|\left|\mathcal{U}\right|$ are $\left|\hat w_i\hat m_{i-1}\right|$, $\left|\hat w_i\right|\left|\hat U_{i-1}\right| + \hat m_i = \hat w_i\hat U_{i-1} + \hat m_i$ (two non-negative terms) and $\left|\hat U_i\right|$, which are the moduli of the entries of $\mathcal{L}\mathcal{U}$. So $\left|\mathcal{L}\right|\left|\mathcal{U}\right| = \left|\mathcal{L}\mathcal{U}\right| = \left|\hat{\mathbf{A}} + \boldsymbol{\Delta}_1\right| \le \left|\hat{\mathbf{A}}\right| + \gamma_1\left|\mathcal{L}\right|\left|\mathcal{U}\right|$, that is

$$
\left|\mathcal{L}\right|\left|\mathcal{U}\right| \le \frac{\left|\hat{\mathbf{A}}\right|}{1 - \gamma_1}, \qquad \left|\boldsymbol{\Delta}\mathbf{A}\right| \le \frac{\gamma_4}{1 - \gamma_1}\left|\hat{\mathbf{A}}\right|. \tag{34}
$$

The same bound, with the same constant, holds for the textbook order of elimination (pivots $\hat m_i = \hat B_i - \hat L_i\hat c_{i-1}$, $\hat c_i = \hat U_i/\hat m_i$, forward $\hat y_i = \left(b_i - \hat L_i\hat y_{i-1}\right)/\hat m_i$, back $\hat x_i = \hat y_i - \hat c_i\hat x_{i+1}$): the factorisation then has the stored $\hat L_i$ exactly in $\mathcal{L}$, the errors are $\gamma_1$ (factorisation), $\gamma_2$ (forward) and $\gamma_1$ (back), and the signs again give $\left|\mathcal{L}\right|\left|\mathcal{U}\right| = \left|\mathcal{L}\mathcal{U}\right|$. This matters because the residuals measured by the test `btcs_verification` on `feat/btcs-solver` (commit 92dac9f) were produced with that order.

**Combined perturbation.** Let $\mathbf{E} = \left(\hat{\mathbf{A}} - \mathbf{A}^*\right) + \boldsymbol{\Delta}\mathbf{A}$, so that $\mathbf{A}^*\hat{\mathbf{x}} = \mathbf{b} - \mathbf{E}\hat{\mathbf{x}}$. By (32) and (34),

$$
\left|\mathbf{E}\right| \le \gamma_3\left|\mathbf{A}^*\right| + \frac{\gamma_4}{1 - \gamma_1}\left|\hat{\mathbf{A}}\right| \le c_{\mathbf{A}}\left|\hat{\mathbf{A}}\right|, \qquad c_{\mathbf{A}} = \frac{\gamma_3}{1 - \gamma_3} + \frac{\gamma_4}{1 - \gamma_1} = 7u\left(1 + O(u)\right). \tag{35}
$$

**Comparison with the Rust constant.** The Rust tests use $c_{\mathrm{step}} = \kappa_t + 3u/\left(1 - 3u\right)$ with $\kappa_t = \left(2a + \gamma_2 + a\gamma_2\right)\left(1 - u\right)/\left(1 - 2u\right)$ and $a = u/\left(1 - u\right) = \gamma_1$, so $\kappa_t = \left(\gamma_1 + \gamma_1 + \gamma_2 + \gamma_1\gamma_2\right)/\left(1 - \gamma_1\right)$: the backward-error constant of (33) and (34) before Lemma 2 rounds it up to $\gamma_4$. The second term is $\gamma_3$ in place of $\gamma_3/\left(1 - \gamma_3\right)$. Both differences are of order $\left(u\right)^2$ and of the same sign, so $c_{\mathbf{A}} \ge c_{\mathrm{step}}$, and in exact rational arithmetic $c_{\mathbf{A}} - c_{\mathrm{step}} = 17\left(u\right)^2\left(1 + O(u)\right)$, a relative difference of $2.7\times10^{-16}$. The two derivations give the same per-step constant, $7u$ to first order. **$c_{\mathrm{step}}$ is rigorous by itself.** The surplus $17\left(u\right)^2$ is slack in two steps of this document, not a term that $c_{\mathrm{step}}$ lacks. (a) Every factor of $\hat B_i/B_i^*$, $\hat L_i/L_i^*$ and $\hat U_i/U_i^*$ is a product of at most three factors $1 + \delta$, so $\left|\mathbf{A}^*/\hat{\mathbf{A}} - 1\right| \le \left(1 - u\right)^{-3} - 1 \le \gamma_3$ entrywise, because $\left(1 - u\right)^3 \ge 1 - 3u$. Hence $\left|\hat{\mathbf{A}} - \mathbf{A}^*\right| \le \gamma_3\left|\hat{\mathbf{A}}\right|$ holds directly; (35) takes the detour $\gamma_3\left|\mathbf{A}^*\right| \le \gamma_3/\left(1 - \gamma_3\right)\left|\hat{\mathbf{A}}\right|$, which costs $9\left(u\right)^2$. (b) Lemma 2 rounds $2\gamma_1 + \gamma_2 + \gamma_1\gamma_2 = 4u + 8\left(u\right)^2 + O\left(\left(u\right)^3\right)$ up to $\gamma_4 = 4u + 16\left(u\right)^2 + O\left(\left(u\right)^3\right)$, which costs $8\left(u\right)^2$. Without the two detours, $\left|\mathbf{E}\right| \le \left(\gamma_3 + \kappa_t\right)\left|\hat{\mathbf{A}}\right| = c_{\mathrm{step}}\left|\hat{\mathbf{A}}\right|$ follows from (32) to (34) with every factor kept, so the Rust constant is not $17\left(u\right)^2$ short.

### 5.4 The exact balance of the computed iterates

Let $\hat{\mathbf{x}}^j$ be the computed vectors, $\hat{\mathbf{x}}^0$ the start vector and $\hat x_N^j = 0$. In the baseline the right-hand side of step $j$ is $\hat{\mathbf{x}}^j$ itself, without rounding. Let

$$
\mathcal{M}^j = \sum_{i=1}^{N-1}\hat V_i\hat x_i^j \quad \left(\mathrm{mol}\right), \qquad S^j = \sum_{i=1}^{N-1}\hat V_i\left(\left|\hat{\mathbf{A}}\right|\hat{\mathbf{x}}^j\right)_i \quad \left(\mathrm{mol}\right), \tag{36}
$$

exact sums of stored numbers ($\mathcal{M}^j$ is the exact inventory of the computed vector divided by $4\pi$). Apply (31) to $\hat{\mathbf{x}}^{j+1}$ and use $\mathbf{A}^*\hat{\mathbf{x}}^{j+1} = \hat{\mathbf{x}}^j - \mathbf{E}\hat{\mathbf{x}}^{j+1}$ in rows $1$ to $N-1$ (row 1 of $\mathbf{E}$ has a zero in column 0, because $\hat L_1 = 0$):

$$
\left|\mathcal{M}^{j+1} - \mathcal{M}^j + \Delta t\,K_{N-1}'\hat x_{N-1}^{j+1}\right| = \left|\sum_{i=1}^{N-1}\hat V_i\left(\mathbf{E}\hat{\mathbf{x}}^{j+1}\right)_i\right| \le c_{\mathbf{A}}\,S^{j+1}, \tag{37}
$$

using $\hat{\mathbf{x}}^{j+1} \ge \mathbf{0}$ (P6 under (P)). Summed over the steps and multiplied by the constant $\hat p$ of Section 5.5, the exact-arithmetic residual of the computed iterates, $\hat p\left(\mathcal{M}^J + \sum_k\Delta t\,K_{N-1}'\hat x_{N-1}^k - \mathcal{M}^0\right)$, is bounded by $\hat p\,c_{\mathbf{A}}\sum_{k=1}^{J}S^k$.

**What $S^j$ measures.** In terms of $\mathbf{A}^*$, $\sum_i\hat V_i\left(\left|\mathbf{A}^*\right|\mathbf{x}\right)_i = \mathcal{M} + 2\Delta t\sum_{i=1}^{N-2}K_i'\left(x_i + x_{i+1}\right) + \Delta t\,K_{N-1}'x_{N-1}$ for $\mathbf{x} \ge \mathbf{0}$ (each interval enters through the outer coefficient of its inner node and the inner coefficient of its outer node). The net flow through an interval is $K'\left(x_i - x_{i+1}\right)$; the rounding sees the gross exchange $K'\left(x_i + x_{i+1}\right)$. Where the mesh Fourier number is large, the gross exchange per step is many times the inventory, and so is the rounding. This is why the residual grows with $\mathrm{Fo}$ and is not at the $10^{-13}$ level of runs with small $\mathrm{Fo}$.

### 5.5 Rounding of the reported quantities

With $\hat p = 4\,\mathrm{fl}(\pi)$ (the factor $4$ is exact) as the common constant of inventory and release:

- $\hat M^j = \mathrm{fl}\left(\hat p\,\mathrm{fl}\left(\sum\hat V_i\hat x_i^j\right)\right)$ by recursive summation of $N-1$ non-negative products: $\hat M^j = \hat p\,\mathcal{M}^j\left(1 + \theta_N\right)$ (Lemmas 1 and 3). The same holds for $\hat M^0$.
- The increment $\hat q^j = \mathrm{fl}\left(\Delta t\,\mathrm{fl}\left(\mathrm{fl}\left(\hat p\,\hat K_{N-1,\mathrm{outer}}\right)\hat x_{N-1}^j\right)\right) = \hat p\,\Delta t\,\hat K_{N-1,\mathrm{outer}}\hat x_{N-1}^j\left(1 + \theta_3\right) = \hat p\,\Delta t\,K_{N-1}'\hat x_{N-1}^j\left(1 + \theta_4\right)$, because $\hat K_{N-1,\mathrm{outer}} = K_{N-1}'\left(1 + \delta\right)^{-1}$ (Lemma 1). The release uses the stored $\hat K_{N-1,\mathrm{outer}}$, not $\hat P_{N-1}$; this is the one place where the rounding of the product appears.
- $\hat M_{\mathrm{rel}}^J = \sum_{j=1}^{J}\hat q^j + \sigma_J$ with the summation error $\left|\sigma_J\right| \le s_Ju\sum\hat q^j$. For recursive summation of non-negative terms $s_Ju = \gamma_{J-1}$ (Lemmas 1 and 3, PROVED). For the compensated (Kahan) accumulator of the code, the classical bound $s_J = 2 + O\left(Ju\right)$ is quoted, not re-proved here.
- $\hat\varepsilon^J = \mathrm{fl}\left(\mathrm{fl}\left(\hat M^J + \hat M_{\mathrm{rel}}^J\right) - \hat M^0\right)$, so $\left|\hat\varepsilon^J\right| \le \left(1 + u\right)\left[\left|X\right| + u\left(\hat M^J + \hat M_{\mathrm{rel}}^J\right)\right]$ with $X = \hat M^J + \hat M_{\mathrm{rel}}^J - \hat M^0$.

### 5.6 The two bounds

**Theorem R-post (a posteriori, PROVED under (FP), (U), (C), (P)).** For every $J \ge 1$,

$$
\left|\hat\varepsilon^J\right| \le \left(1 + \eta\right)\left[\hat p\,c_{\mathbf{A}}\sum_{k=1}^{J}S^k + \gamma_N\left(\hat M^J + \hat M^0\right) + \left(\gamma_4 + s_Ju + u\right)\hat M_{\mathrm{rel}}^J + u\,\hat M^J\right], \tag{38}
$$

with $c_{\mathbf{A}} = \gamma_3/\left(1 - \gamma_3\right) + \gamma_4/\left(1 - \gamma_1\right)$ of (35), $s_J \le J$, and the factor $1 + \eta$, $\eta = 4\left(3N + J + 10\right)u$ (dimensionless), under the condition $\eta \le 1$. The bound holds also when the right side is evaluated in floating point: if $\hat T^J$ is the computed value of the right side of (38), in the operation order of step (c) below, then $\left|\hat\varepsilon^J\right| \le \hat T^J$.

*Proof.* (a) *Decomposition.* $X = \hat p\left(\mathcal{M}^J + \sum_k\Delta t\,K'\hat x_{N-1}^k - \mathcal{M}^0\right) + \left(\hat M^J - \hat p\mathcal{M}^J\right) - \left(\hat M^0 - \hat p\mathcal{M}^0\right) + \sum_k\left(\hat q^k - \hat p\,\Delta t\,K'\hat x_{N-1}^k\right) + \sigma_J$, with $K' = K_{N-1}'$. The first term is bounded by the sum of (37) over the steps, times $\hat p$; the others by Section 5.5:

$$
\left|X\right| \le \hat p\,c_{\mathbf{A}}\sum_{k=1}^{J}S^k + \gamma_N\,\hat p\left(\mathcal{M}^J + \mathcal{M}^0\right) + \gamma_4\,\hat p\sum_{k=1}^{J}\Delta t\,K'\hat x_{N-1}^k + s_Ju\sum_{k=1}^{J}\hat q^k.
$$

(b) *The factors, written out.* By Section 5.5, $\hat p\mathcal{M} \le \hat M/\left(1 - \gamma_N\right)$ (for $\mathcal{M}^J$ and $\mathcal{M}^0$), $\hat p\sum_k\Delta t\,K'\hat x_{N-1}^k \le \sum_k\hat q^k/\left(1 - \gamma_4\right)$ and $\sum_k\hat q^k \le \hat M_{\mathrm{rel}}^J/\left(1 - s_Ju\right)$. Insert these into the last item of Section 5.5, $\left|\hat\varepsilon^J\right| \le \left(1 + u\right)\left[\left|X\right| + u\left(\hat M^J + \hat M_{\mathrm{rel}}^J\right)\right]$, and collect the coefficient of each reported amount:

- of $\hat p\,c_{\mathbf{A}}\sum_kS^k$: $1 + u$;
- of $\gamma_N\left(\hat M^J + \hat M^0\right)$: $\left(1 + u\right)/\left(1 - \gamma_N\right)$;
- of $\hat M_{\mathrm{rel}}^J$: $\left(1 + u\right)\left[\frac{\gamma_4}{\left(1 - \gamma_4\right)\left(1 - s_Ju\right)} + \frac{s_Ju}{1 - s_Ju} + u\right] \le \frac{1 + u}{\left(1 - \gamma_4\right)\left(1 - s_Ju\right)}\left(\gamma_4 + s_Ju + u\right)$;
- of $u\,\hat M^J$: $1 + u$.

Every coefficient is at most $f$ times the corresponding coefficient in the bracket of (38), with

$$
f = \frac{1 + u}{\left(1 - \gamma_N\right)\left(1 - \gamma_4\right)\left(1 - s_Ju\right)}.
$$

So $\left|\hat\varepsilon^J\right| \le f\,R^J$, where $R^J$ (in $\mathrm{mol}$) is the exact value of the bracket of (38).

(c) *Evaluation.* The code evaluates $R^J$ in floating point: $S^k$ as the sum over $i = 1, \dots, N-1$ of $\hat V_i\left(\left|\hat L_i\right|\hat x_{i-1}^k + \hat B_i\hat x_i^k + \left|\hat U_i\right|\hat x_{i+1}^k\right)$, accumulated over $k$, multiplied by $c_{\mathbf{A}}$ and $\hat p$; the constants $\gamma_\nu$ and $c_{\mathbf{A}}$ from $u$; and the four terms added. Every operand is a non-negative floating-point number and every operation is a product, a quotient or a sum of non-negative numbers, except the subtractions $1 - \nu u$, $1 - \hat\gamma_3$ and $1 - \hat\gamma_1$ in the constants. Their results are positive; $1 - \nu u$ carries at most one rounding, and the computed $1 - \hat\gamma_\nu$ ($\nu = 1, 3$) differs from $1 - \gamma_\nu$ by one rounding and by the error already in $\hat\gamma_\nu$, which is below $\gamma_\nu\gamma_2/\left(1 - \gamma_\nu\right) < u$ relative and is counted as one more factor $1 + \delta$. $c_{\mathbf{A}}$ is computed as $\mathrm{fl}\left(\mathrm{fl}\left(\hat\gamma_3/\mathrm{fl}\left(1 - \hat\gamma_3\right)\right) + \mathrm{fl}\left(\hat\gamma_4/\mathrm{fl}\left(1 - \hat\gamma_1\right)\right)\right)$ with $\hat\gamma_\nu = \mathrm{fl}\left(\nu u/\mathrm{fl}\left(1 - \nu u\right)\right)$: along either of its two terms, two roundings in the numerator $\hat\gamma_\nu$, two factors in the denominator, one quotient and the final addition of two non-negative terms, at most six. Counting the roundings along the longest chain of any term of the bracket: three in $\left(\left|\hat{\mathbf{A}}\right|\hat{\mathbf{x}}\right)_i$, one for $\hat V_i$, $N - 2$ in the sum over $i$, $J - 1$ in the sum over $k$, two for $c_{\mathbf{A}}$ and $\hat p$, at most six inside $c_{\mathbf{A}}$, and three for the four terms of the bracket: at most $m = N + J + 12$. By Lemma 1, each term is computed with a factor in $\left[\left(1 - u\right)^m, \left(1 - u\right)^{-m}\right]$, so the computed bracket satisfies $\hat R^J \ge \left(1 - mu\right)R^J$. $\eta$ is an integer multiple of $u = 2^{-53}$ and is computed exactly; the tolerance is $\hat T^J = \mathrm{fl}\left(\mathrm{fl}\left(1 + \eta\right)\hat R^J\right) \ge \left(1 + \eta\right)\left(1 - u\right)^2\left(1 - mu\right)R^J$.

(d) *The factor $1 + \eta$ covers $f$ and the evaluation.* It remains to show $\left(1 + \eta\right)\left(1 - u\right)^2\left(1 - mu\right) \ge f$, that is

$$
\left(1 + \eta\right)\left(1 - u\right)^2\left(1 - mu\right)\left(1 - \gamma_N\right)\left(1 - \gamma_4\right)\left(1 - s_Ju\right) \ge 1 + u.
$$

Use $1 - \gamma_\nu = \left(1 - 2\nu u\right)/\left(1 - \nu u\right) \ge 1 - 2\nu u$, $s_J \le J$, and $\prod_k\left(1 - a_k\right) \ge 1 - \sum_ka_k$ for $0 \le a_k \le 1$. The left side is at least $\left(1 + \eta\right)\left(1 - a\right)$ with $a = \left(2 + m + 2N + 8 + J\right)u = \left(3N + 2J + 22\right)u$. Now $\left(1 + \eta\right)\left(1 - a\right) \ge 1 + u$ is equivalent to $\eta\left(1 - a\right) \ge a + u$. Since $\eta = \left(12N + 4J + 40\right)u \ge 2\left(a + u\right) = \left(6N + 4J + 46\right)u$ for $N \ge 1$, and $\eta \le 1$ gives $a \le \eta/2 \le \frac{1}{2}$, we have $\eta\left(1 - a\right) \ge \eta/2 \ge a + u$. Hence $\left|\hat\varepsilon^J\right| \le f\,R^J \le \hat T^J$. $\square$

The proof uses $m = N + J + 12$ for the operation order of (c); an evaluation in a different order with up to $N + J + 22$ roundings per term is still covered, because the inequality $\eta \ge 2\left(a + u\right)$ keeps a margin of $\left(6N - 6\right)u$, ten more roundings cost $20u$, and $N \ge 5$.

To first order in $u$, (38) reads $\left|\hat\varepsilon^J\right| \lesssim 7u\cdot4\pi\sum_kS^k + u\left[N\left(\hat M^J + \hat M^0\right) + \left(5 + s_J\right)\hat M_{\mathrm{rel}}^J + \hat M^J\right]$. The first term dominates whenever $\mathrm{Fo}$ is not small.

**Relation to the Rust bound.** The Rust bound (`balance_bound_history` in the test file, and the production `diagnostics::BalanceBound`) has the same structure: $4\pi c_{\mathrm{step}}$ times the same sum of $S^k$, plus the rounding of the two inventories, of the increments and of $\hat M + \hat M_{\mathrm{rel}}$. It differs from (38) in three places, none of which is a missing term. (i) Its per-step constant $c_{\mathrm{step}}$ is smaller than $c_{\mathbf{A}}$ by $17\left(u\right)^2$; it is rigorous by itself, because the difference is slack in two steps of (35) (Section 5.3). (ii) In place of $s_Ju\,\hat M_{\mathrm{rel}}^J$ it adds the accumulation error of the compensated sum, measured exactly with expansion arithmetic; this is smaller than $s_Ju\,\hat M_{\mathrm{rel}}^J$ with $s_J = J$ and needs no quoted constant. (iii) It takes the floating-point evaluation of the bound as exact, which its test header states and justifies by the observed ratios of residual to bound. The header's figure for them, "at most $6\times10^{-2}$", is not true in general: coordinator decision 37 records that the largest ratio over all bounds evaluated in that test file is about $0.4$, and the figure is to be corrected on `feat/btcs-solver`. For the balance bound alone the largest ratio is $1.3\times10^{-2}$, over the seven runs of test F (the 20-per-layer case; `solver-core-results.md`, section F, and the table of Section 5.7), and $3.1\times10^{-2}$ over the three large-$\mathrm{Fo}$ runs of Section 5.7, which only the scratch program evaluated. In place of the evaluation error it keeps the slack factors $1 + \gamma_{N+6}$, $1 + \gamma_{J+8}$ and $\gamma_{N+1}/\left(1 - \gamma_{N+1}\right)$ in place of $\gamma_N$; (38) covers the evaluation by the factor $1 + \eta$. ($J$ in the factor $1 + \gamma_{J+8}$ is the total step count of the run.) In the cases of Section 5.7 the final values of the two bounds differ by at most $1.7\times10^{-5}$ of the bound, almost all of it from item (ii), so they agree to the printed digits.

**Theorem R-prior (a priori, PROVED under (FP), (U), (C), (P)).** Let

$$
\Lambda_{\max} = \max_{1 \le i \le N-1}\left(\left|\hat L_i\right| + \left|\hat U_i\right|\right), \qquad \kappa_R = c_{\mathbf{A}}\left(1 + \gamma_2\right)\left(1 + 2\Lambda_{\max}\right), \qquad \rho_J = \left(1 - \kappa_R\right)^{-J}, \tag{39}
$$

with $\hat U_{N-1}$ (the coefficient that enters $\hat B_{N-1}$) included; $\Lambda_{\max}$, $\kappa_R$ and $\rho_J$ are dimensionless. ($\kappa_R$ is a rounding number, not the decay number $\kappa$ of Section 2.) If $\kappa_R < 1$ and the condition $\eta \le 1$ of (38) holds, then for every $J \ge 1$

$$
\frac{\left|\hat\varepsilon^J\right|}{\hat M^0} \le \frac{1 + \eta}{1 - \gamma_N}\left[\left(\rho_J - 1\right) + b_J\,\rho_J\right], \qquad b_J = 2\gamma_N\left(1 + \gamma_N\right) + \left(\gamma_4 + s_Ju + u\right)\left(1 + \gamma_4\right)\left(1 + s_Ju\right) + u\left(1 + \gamma_N\right), \tag{40}
$$

with $b_J$ dimensionless and $b_J = \left(2N + 6 + s_J\right)u\left(1 + O\left(\left(N + J\right)u\right)\right)$ (the $6$ is $4$ from $\gamma_4$, which now carries the rounding of the product $-\Delta t\,\hat K_{N-1,\mathrm{outer}}$, and $1 + 1$ from the two terms in $u$). When $J\kappa_R \ll 1$ and $\Lambda_{\max} \gg N + J$, (40) is about $7u\,J\left(1 + 2\Lambda_{\max}\right)$. (40) is a statement about real numbers, meant for planning a run; unlike (38), its own floating-point evaluation is not part of the claim.

*Proof.* (a) *Column sums.* $S^j = \sum_i\hat x_i^j\,c_i$ with $c_i = \hat V_i\hat B_i + \hat V_{i+1}\left|\hat L_{i+1}\right| + \hat V_{i-1}\left|\hat U_{i-1}\right|$ (terms outside rows $1$ to $N-1$ omitted). By Section 5.2, $\hat V_{i+1}\left|\hat L_{i+1}\right| = \left|\hat P_i\right|\left(1 + \delta\right)$ and $\hat V_i\left|\hat U_i\right| = \left|\hat P_i\right|\left(1 + \delta'\right)$, so their ratio is at most $\left(1 + u\right)/\left(1 - u\right) \le 1 + \gamma_2$; likewise $\hat V_{i-1}\left|\hat U_{i-1}\right| \le \left(1 + \gamma_2\right)\hat V_i\left|\hat L_i\right|$; and $\hat B_i \le \left(1 + \left|\hat L_i\right| + \left|\hat U_i\right|\right)\left(1 + \gamma_2\right)$. So $c_i \le \left(1 + \gamma_2\right)\hat V_i\left(1 + 2\left|\hat L_i\right| + 2\left|\hat U_i\right|\right) \le \left(1 + \gamma_2\right)\left(1 + 2\Lambda_{\max}\right)\hat V_i$, and $c_{\mathbf{A}}S^j \le \kappa_R\mathcal{M}^j$.

(b) *Growth of the exact inventory.* By (37) and $\Delta t\,K_{N-1}'\hat x_{N-1}^j \ge 0$, $\mathcal{M}^j \le \mathcal{M}^{j-1} + c_{\mathbf{A}}S^j \le \mathcal{M}^{j-1} + \kappa_R\mathcal{M}^j$, so $\mathcal{M}^j \le \left(1 - \kappa_R\right)^{-1}\mathcal{M}^{j-1}$ and $\mathcal{M}^j \le \rho_j\,\mathcal{M}^0$.

(c) *The first term of (38).* $c_{\mathbf{A}}\sum_{k=1}^{J}S^k \le \kappa_R\mathcal{M}^0\sum_{k=1}^{J}\left(1 - \kappa_R\right)^{-k} = \mathcal{M}^0\left(\rho_J - 1\right)$.

(d) *The other terms of (38), with every factor.* Summing (37) over the steps and using (c) and $\mathcal{M}^J \ge 0$: $\sum_{k=1}^{J}\Delta t\,K_{N-1}'\hat x_{N-1}^k \le \mathcal{M}^0 - \mathcal{M}^J + c_{\mathbf{A}}\sum_kS^k \le \rho_J\,\mathcal{M}^0$. With Section 5.5 and (b):

- $\hat M^J = \hat p\,\mathcal{M}^J\left(1 + \theta_N\right) \le \left(1 + \gamma_N\right)\rho_J\,\hat p\,\mathcal{M}^0$;
- $\hat M_{\mathrm{rel}}^J \le \left(1 + s_Ju\right)\sum_k\hat q^k \le \left(1 + s_Ju\right)\left(1 + \gamma_4\right)\hat p\sum_k\Delta t\,K_{N-1}'\hat x_{N-1}^k \le \left(1 + s_Ju\right)\left(1 + \gamma_4\right)\rho_J\,\hat p\,\mathcal{M}^0$;
- $\hat M^0 \le \left(1 + \gamma_N\right)\hat p\,\mathcal{M}^0 \le \left(1 + \gamma_N\right)\rho_J\,\hat p\,\mathcal{M}^0$ (since $\rho_J \ge 1$), and $\hat p\,\mathcal{M}^0 \le \hat M^0/\left(1 - \gamma_N\right)$.

(38) read as an inequality between real numbers, $\left|\hat\varepsilon^J\right| \le \left(1 + \eta\right)R^J$, follows from step (d) of R-post, because $f \le \left(1 + \eta\right)\left(1 - u\right)^2\left(1 - mu\right) \le 1 + \eta$. Divide $R^J$, the bracket of (38), by $\hat p\,\mathcal{M}^0 > 0$ and bound its five terms one by one with (c) and the three items above:

- $\hat p\,c_{\mathbf{A}}\sum_kS^k/\left(\hat p\,\mathcal{M}^0\right) \le \rho_J - 1$;
- $\gamma_N\hat M^J/\left(\hat p\,\mathcal{M}^0\right) \le \gamma_N\left(1 + \gamma_N\right)\rho_J$;
- $\gamma_N\hat M^0/\left(\hat p\,\mathcal{M}^0\right) \le \gamma_N\left(1 + \gamma_N\right)\rho_J$;
- $\left(\gamma_4 + s_Ju + u\right)\hat M_{\mathrm{rel}}^J/\left(\hat p\,\mathcal{M}^0\right) \le \left(\gamma_4 + s_Ju + u\right)\left(1 + \gamma_4\right)\left(1 + s_Ju\right)\rho_J$;
- $u\,\hat M^J/\left(\hat p\,\mathcal{M}^0\right) \le u\left(1 + \gamma_N\right)\rho_J$.

The two inventory terms together give $2\gamma_N\left(1 + \gamma_N\right)\rho_J$, and the coefficients of $\rho_J$ in the last four lines add up to $b_J$ of (40). So

$$
\frac{\left|\hat\varepsilon^J\right|}{1 + \eta} \le R^J \le \hat p\,\mathcal{M}^0\left[\left(\rho_J - 1\right) + b_J\,\rho_J\right],
$$

and $\hat p\,\mathcal{M}^0 \le \hat M^0/\left(1 - \gamma_N\right)$ gives (40). No step uses more than the hypotheses of (38) and $\kappa_R < 1$. $\square$

**Correction of the second version.** The second version of (40), and D2 (F45), wrote the first term of $b_J$ as $\gamma_N\left(2 + \gamma_N\right)$. The two inventory terms give $2\gamma_N\left(1 + \gamma_N\right)$, which is larger by $\left(\gamma_N\right)^2$, so the step "the bracket is at most $\left(\rho_J - 1\right) + b_J\rho_J$" was false as written. The coefficient is now the one the proof yields. The scratch program of this revision (Section 7) evaluated (40) with both coefficients, in 60-digit arithmetic and in double precision, for the seven cases of the synthetic table and the three cases of the large-$\mathrm{Fo}$ table of Section 5.7. The relative change is at most $1.3\times10^{-20}$, and the double-precision values are identical, so every printed (40) value is unchanged.

The first version of (40) wrote the factor as $\left(1 + \eta\right)$ and the coefficient as $\left(2N + 6 + s_J\right)u$, absorbing the factors $1/\left(1 - \gamma_N\right)$, $1 + \gamma_N$, $1 + \gamma_4$ and $1 + s_Ju$ into one spare $u$; that is valid only while about $\left(2N + J\right)\left(\eta + 2Nu\right) \le 1$, which holds on every practical grid but does not follow from $\eta \le 1$. (40) above carries every factor and needs no such condition.

For an interior row $\left|L_i\right| + \left|U_i\right| = 2\,\mathrm{Fo}_{\mathrm{material}}$, so $\Lambda_{\max}$ is about twice the largest mesh Fourier number (slightly more at interface rows, by the factors of (F16)). (40) is the rule the verification plan asked for: the step count and the largest $\mathrm{Fo}$ enter, linearly.

### 5.7 Check against the measured residuals

Synthetic case, $\Delta t = 200\ \mathrm{s}$, 1000 steps unless stated. There are two Rust measurements of the worst $\left|\varepsilon^j\right|/M^0$, both from the test `btcs_verification` on `feat/btcs-solver`: the first run, at commit 92dac9f, which solved with the textbook elimination order, and the current one, which solves with the elimination order of (F24), as recorded in `solver-core-results.md` (section F) after the tolerance task. The scratch program of this section (Section 7) repeats the Rust operations in the Rust order, for both elimination orders. With the textbook order it reproduces every 92dac9f value to all printed digits, and with the (F24) order it reproduces every current Rust value to all printed digits; the second agreement is the stronger check, because it concerns the code as it now is. The program also evaluates the Rust bound of the tolerance task with the operations of `balance_bound_history` and reproduces its final values and the Rust ratios of observed residual to bound ($1.3\times10^{-2}$, $6.0\times10^{-4}$, $3.8\times10^{-3}$, $4.5\times10^{-3}$, $5.5\times10^{-4}$, $1.0\times10^{-3}$, $3.2\times10^{-3}$ in the order of the table) to all printed digits. The bounds below are final values divided by $M^0$; (38) is evaluated with the factor $1 + \eta$ in the operation order of step (c) of R-post; a ratio is the largest over all steps of the observed residual divided by the bound at that step.

| Case | $N$ | $\Lambda_{\max}$ | Rust, 92dac9f (textbook order) | Rust, current ((F24) order) | scratch, textbook | scratch, (F24) | (38) final | Rust bound final | (40) final | largest ratio to (38), textbook / (F24) |
|---|---|---|---|---|---|---|---|---|---|---|
| 20 per layer | 100 | $6.40\times10^{3}$ | $2.30\times10^{-11}$ | $3.77\times10^{-11}$ | $2.300\times10^{-11}$ | $3.766\times10^{-11}$ | $2.87\times10^{-9}$ | $2.87\times10^{-9}$ | $9.95\times10^{-9}$ | $8.1\times10^{-3}$ / $1.3\times10^{-2}$ |
| 40 per layer | 200 | $2.56\times10^{4}$ | $9.45\times10^{-11}$ | $6.91\times10^{-12}$ | $9.449\times10^{-11}$ | $6.912\times10^{-12}$ | $1.15\times10^{-8}$ | $1.15\times10^{-8}$ | $3.98\times10^{-8}$ | $8.2\times10^{-3}$ / $6.0\times10^{-4}$ |
| 80 per layer | 400 | $1.02\times10^{5}$ | $5.66\times10^{-11}$ | $1.70\times10^{-10}$ | $5.656\times10^{-11}$ | $1.705\times10^{-10}$ | $4.59\times10^{-8}$ | $4.59\times10^{-8}$ | $1.59\times10^{-7}$ | $1.5\times10^{-3}$ / $3.8\times10^{-3}$ |
| 160 per layer | 800 | $4.10\times10^{5}$ | $1.19\times10^{-9}$ | $8.25\times10^{-10}$ | $1.187\times10^{-9}$ | $8.252\times10^{-10}$ | $1.84\times10^{-7}$ | $1.84\times10^{-7}$ | $6.37\times10^{-7}$ | $6.5\times10^{-3}$ / $4.5\times10^{-3}$ |
| 320 per layer | 1600 | $1.64\times10^{6}$ | $1.13\times10^{-9}$ | $3.99\times10^{-10}$ | $1.126\times10^{-9}$ | $3.987\times10^{-10}$ | $7.35\times10^{-7}$ | $7.35\times10^{-7}$ | $2.55\times10^{-6}$ | $1.5\times10^{-3}$ / $5.5\times10^{-4}$ |
| 40 per layer, one step of $2\times10^5\ \mathrm{s}$ | 200 | $2.56\times10^{7}$ | $1.37\times10^{-11}$ | $9.69\times10^{-12}$ | $1.373\times10^{-11}$ | $9.691\times10^{-12}$ | $9.45\times10^{-9}$ | $9.45\times10^{-9}$ | $3.98\times10^{-8}$ | $1.5\times10^{-3}$ / $1.0\times10^{-3}$ |
| 40 per layer, contrast $10^4$ | 200 | $2.56\times10^{4}$ | $2.40\times10^{-11}$ | $1.09\times10^{-11}$ | $2.397\times10^{-11}$ | $1.094\times10^{-11}$ | $3.49\times10^{-9}$ | $3.49\times10^{-9}$ | $3.98\times10^{-8}$ | $9.4\times10^{-3}$ / $3.2\times10^{-3}$ |

(The contrast case alternates $10^{-14}$ and $10^{-10}\ \mathrm{m^2\ s^{-1}}$, starting with $10^{-14}$ in the kernel, as in the Rust test.) The final values of (38) and of the Rust bound differ by $1.7\times10^{-5}$ of the bound at 20 per layer and by less at the finer grids ($4.2\times10^{-6}$, $1.0\times10^{-6}$, $2.6\times10^{-7}$, $6.5\times10^{-8}$, $-1.5\times10^{-8}$ for the single step, $3.8\times10^{-7}$ for the contrast case; positive where (38) is the larger), for the reasons listed after R-post.

**Very large mesh Fourier numbers.** The residual grows with $\Lambda_{\max}$ and with $J$, as (38) and (40) say, so the range of the table above belongs to the synthetic case with $\Delta t = 200\ \mathrm{s}$ and 1000 steps only. Coordinator decision 36 records a measured residual of up to $2.3\times10^{-9}$ at very large mesh Fourier numbers. The same scratch program, with the (F24) order, ran a synthetic case chosen for large $\mathrm{Fo}$: radii $100$, $110$, $150$, $151$, $170\ \mathrm{\mu m}$, diffusivities $10^{-10}$, $10^{-18}$, $10^{-10}$, $10^{-18}$, $10^{-10}\ \mathrm{m^2\ s^{-1}}$ from kernel to OPyC, $C_{\mathrm{init}} = 1\ \mathrm{mol\ m^{-3}}$, $\Delta t = 10^5\ \mathrm{s}$, 3000 steps (synthetic test values, not data):

| Counts | $N$ | largest $\mathrm{Fo}$ | $\Lambda_{\max}$ | worst $\left\lvert\varepsilon^j\right\rvert/M^0$ | (38) final | (40) final | largest ratio to (38) |
|---|---|---|---|---|---|---|---|
| $(10,10,10,10,10)$ | 50 | $2.77\times10^{6}$ | $5.54\times10^{6}$ | $1.05\times10^{-8}$ | $8.56\times10^{-7}$ | $2.58\times10^{-5}$ | $3.1\times10^{-2}$ |
| $(20,10,30,2,14)$ | 76 | $5.63\times10^{6}$ | $1.13\times10^{7}$ | $3.71\times10^{-8}$ | $4.94\times10^{-6}$ | $5.25\times10^{-5}$ | $1.3\times10^{-2}$ |
| $(40,20,40,4,20)$ | 124 | $1.11\times10^{7}$ | $2.22\times10^{7}$ | $5.03\times10^{-8}$ | $1.38\times10^{-5}$ | $1.03\times10^{-4}$ | $5.9\times10^{-3}$ |

The Rust bound, evaluated by the same program, agrees with (38) to $2\times10^{-7}$ of the bound in these three cases, with the same ratios. Every computed concentration was non-negative, and the smallest positive one was $7.4\times10^{-24}\ \mathrm{mol\ m^{-3}}$, so no concentration was subnormal. The first version of this section ran these cases with another program, whose operation order was not the Rust order; its residuals were $1.14\times10^{-8}$, $3.03\times10^{-8}$ and $6.72\times10^{-8}$, of the same size as but not equal to the values above, which shows again that the residual is a rounding quantity. Its bounds, which used the first-version constants, are reproduced by the present program ((38) of the first version: $1.22\times10^{-6}$, $7.05\times10^{-6}$, $1.97\times10^{-5}$; (40) of the first version: $3.69\times10^{-5}$, $7.49\times10^{-5}$, $1.48\times10^{-4}$).

Reading:

- Every observed residual lies below (38), (40) and the Rust bound at every step; the largest ratio to (38) is $1.3\times10^{-2}$ in the synthetic table and $3.1\times10^{-2}$ in the large-$\mathrm{Fo}$ table. The relative residual itself ranges from about $7\times10^{-12}$ to $1.2\times10^{-9}$ in the synthetic table and reaches $5.0\times10^{-8}$ in the large-$\mathrm{Fo}$ table; no fixed range is a property of the scheme. The bound is not tightened from these numbers (decision 6), and no number here was used to choose a constant: $c_{\mathbf{A}}$ comes from Sections 5.2 and 5.3.
- The bounds grow like $\left(\Delta r\right)^{-2}$ through $\Lambda_{\max}$; the observed residuals grow more slowly. The gap (the ratios lie between $5.5\times10^{-4}$ and $3.1\times10^{-2}$) is consistent with rounding errors of varying sign, which a worst-case bound adds in modulus; no probabilistic statement is claimed.
- The two elimination orders give different residuals of the same size, in either order of ranking depending on the grid, as D1 Section 9.4 found for other implementation details.
- The first version of (38), with $c_{\mathbf{A}} = \gamma_{10}/\left(1 - \gamma_4\right)$ and $\gamma_3$ on $\hat M_{\mathrm{rel}}^J$, gave final values larger by the factor $1.43$ in every case ($4.10\times10^{-9}$, $1.64\times10^{-8}$, $6.56\times10^{-8}$, $2.63\times10^{-7}$, $1.05\times10^{-6}$, $1.35\times10^{-8}$, $4.99\times10^{-9}$ in the order of the table; the replica reproduces them). It was valid but not sharp (Section 5.2). The bound with $16u$ of the first Rust tests was larger again, by a factor of about $1.6$ over that first version; it had no derivation and is superseded on `feat/btcs-solver`.

**Recommendation for the tests (decision 6 allows only a derived tolerance).** A derived tolerance is never replaced by a looser one without a stated reason. The Rust bound of the tolerance task therefore stays the tolerance of test F and the bound reported in every output row: it is derived, its per-step constant equals $c_{\mathbf{A}}$ of (35) to first order (Section 5.3), it measures the accumulation error of the release sum exactly instead of using $s_Ju$, and its final values agree with (38) to the printed digits. This section confirms its constant by an independent count of the roundings; it does not replace it. (38) is the form to use where a test must also cover the floating-point evaluation of the bound itself, which the Rust bound takes as exact: use it at every step with $c_{\mathbf{A}}$ of (35), $s_J = J$ (which covers both the recursive and the compensated accumulator) and the factor $1 + \eta$, and report the observed residual and the ratio beside it. The first version of this recommendation proposed (38) with the constant $\gamma_{10}/\left(1 - \gamma_4\right)$ for test F; that bound is $1.43$ times looser than the Rust bound in every case, the recommendation gave no reason for the change, and it is withdrawn. Use (40) where a tolerance must be known before a run (for example to choose $\Delta t$ and the grid so that the balance stays below a stated level). All of them need (P), which the code checks. For runs with a source, decay or $T(t)$, use (41) of Section 5.8, evaluated in the operation order stated there, or extend the Rust bound by the additional terms of (41) and re-derive its constants for that order.

### 5.8 Extension to the combined scheme

The same argument, with the matrix and the sums of each step, gives the bound (41) for G1 to G4. The changes, each of which enters (41):

- **Decay.** $\hat\kappa = \mathrm{fl}\left(\lambda\,\Delta t\right)$ is stored and used both in $\hat B_i = \mathrm{fl}\left(\mathrm{fl}\left(\mathrm{fl}\left(1 - \hat L_i\right) - \hat U_i\right) + \hat\kappa\right)$ and in the decay booking; $\mathbf{A}^*$ contains $\hat\kappa$ exactly ($B_i^* = 1 + \left|L_i^*\right| + \left|U_i^*\right| + \hat\kappa$), so (31) gains $\hat\kappa\sum\hat V_ix_i$ on the right. The diagonal now carries three additions of non-negative numbers, so $\hat B_i = B_i^*\left(1 + \theta_4\right)$ and $\left|\hat{\mathbf{A}} - \mathbf{A}^*\right| \le \gamma_4\left|\mathbf{A}^*\right|$, and (35) becomes $\left|\mathbf{E}\right| \le c_{\mathbf{A}}^{\mathrm{G}}\left|\hat{\mathbf{A}}\right|$ with $c_{\mathbf{A}}^{\mathrm{G}} = \gamma_4/\left(1 - \gamma_4\right) + \gamma_4/\left(1 - \gamma_1\right) = 8u\left(1 + O(u)\right)$. The booking $\mathrm{fl}\left(\hat\kappa\hat M^{j+1}\right)$ differs from $\hat\kappa\hat p\mathcal{M}^{j+1}$ by at most $\gamma_{N+1}\hat\kappa\hat p\mathcal{M}^{j+1}$.
- **Source.** The right-hand side $\hat b_i = \mathrm{fl}\left(\hat x_i^j + \mathrm{fl}\left(\Delta t\,\hat g_i^j\right)\right)$ with $\hat g_i = \mathrm{fl}\left(\hat Q_i/\hat V_i\right)$ now carries rounding: $\left|\hat V_i\left(\hat b_i - \hat x_i^j\right) - \Delta t\,\hat Q_i\right| \le u\,\hat V_i\hat x_i^j + \gamma_3\,\Delta t\,\hat Q_i$. The booked generation $\mathrm{fl}\left(\Delta t\,\mathrm{fl}\left(\hat p\,\mathrm{fl}\left(\sum_{i=1}^{N}\hat Q_i\right)\right)\right)$ differs from $\hat p\,\Delta t\sum\hat Q_i$ by at most $\gamma_{N+1}\hat p\,\Delta t\sum\hat Q_i$. The release correction $\hat p\hat Q_N$ adds one rounded addition to $\hat q^j$ ($\gamma_5$ in place of the $\gamma_4$ of Section 5.5).
- **Temperature.** Each step uses its own $\hat{\mathbf{A}}^j$ in $S^{j+1}$; nothing else changes.
**Operation orders** for which the bound below is derived (the code follows them): $\hat B_i^j = \mathrm{fl}\left(\mathrm{fl}\left(\mathrm{fl}\left(1 - \hat L_i^j\right) - \hat U_i^j\right) + \hat\kappa\right)$; $\hat b_i^j = \mathrm{fl}\left(\hat x_i^j + \mathrm{fl}\left(\Delta t\,\hat g_i^j\right)\right)$ with $\hat g_i^j = \mathrm{fl}\left(\hat Q_i^j/\hat V_i\right)$ and the stored source weights $\hat Q_i^j \ge 0$; the release increment $\hat q^{j+1} = \mathrm{fl}\left(\Delta t\,\mathrm{fl}\left(\mathrm{fl}\left(\mathrm{fl}\left(\hat p\,\hat K_{N-1,\mathrm{outer}}^j\right)\hat x_{N-1}^{j+1}\right) + \mathrm{fl}\left(\hat p\,\hat Q_N^j\right)\right)\right)$; the generation increment $\mathrm{fl}\left(\Delta t\,\mathrm{fl}\left(\hat p\,\mathrm{fl}\left(\sum_{i=1}^{N}\hat Q_i^j\right)\right)\right)$; the decay increment $\mathrm{fl}\left(\hat\kappa\,\hat M^{j+1}\right)$; each of $\hat M_{\mathrm{rel}}$, $\hat M_{\mathrm{gen}}$, $\hat M_{\mathrm{dec}}$ accumulated with summation constant $s_J \le J$; and $\hat\varepsilon^J = \mathrm{fl}\left(\mathrm{fl}\left(\mathrm{fl}\left(\hat M^J + \hat M_{\mathrm{rel}}^J\right) - \mathrm{fl}\left(\hat M^0 + \hat M_{\mathrm{gen}}^J\right)\right) + \hat M_{\mathrm{dec}}^J\right)$. $S^{k}$ is (36) with the stored matrix $\hat{\mathbf{A}}^{k-1}$ of the step that produced $\hat{\mathbf{x}}^{k}$. $\chi^k = 1$ if some $\hat g_i^k > 0$ in step $k$, otherwise $\chi^k = 0$ (dimensionless).

**Theorem R-comb (a posteriori, PROVED under (FP), (U), (C), (P)).** For the combined scheme, for every $J \ge 1$ and with the same $\eta = 4\left(3N + J + 10\right)u \le 1$ as in (38),

$$
\begin{aligned}
\left|\hat\varepsilon^J\right| \le \left(1 + \eta\right)\Big[\,&\hat p\,c_{\mathbf{A}}^{\mathrm{G}}\sum_{k=1}^{J}S^k + u\sum_{k=0}^{J-1}\chi^k\hat M^k + \gamma_N\left(\hat M^J + \hat M^0\right) + \left(\gamma_5 + s_Ju\right)\hat M_{\mathrm{rel}}^J\\
&+ \left(\gamma_{N+1} + \gamma_3 + s_Ju\right)\hat M_{\mathrm{gen}}^J + \left(\gamma_{N+1} + s_Ju\right)\hat M_{\mathrm{dec}}^J + \gamma_2\left(\hat M^J + \hat M_{\mathrm{rel}}^J + \hat M^0 + \hat M_{\mathrm{gen}}^J\right)\Big], \qquad c_{\mathbf{A}}^{\mathrm{G}} = \frac{\gamma_4}{1 - \gamma_4} + \frac{\gamma_4}{1 - \gamma_1}.
\end{aligned} \tag{41}
$$

The terms, in order: the backward error of the solves; the rounding of the right-hand side $\hat x + \Delta t\,\hat g$ at steps with a source; the rounding of the two inventories; the release increments ($\gamma_5$) and their accumulator; the generation increments ($\gamma_{N+1}$), the source part of the right-hand side ($\gamma_3$) and their accumulator; the decay increments ($\gamma_{N+1}$) and their accumulator; the evaluation of $\hat\varepsilon^J$ ($\gamma_2$).

*Proof.* (a) *Per step, in exact arithmetic on stored numbers.* With $\hat\kappa$ in $\mathbf{A}^{*j}$, (31) becomes $\sum_{i=1}^{N-1}\hat V_i\left(\mathbf{A}^{*j}\mathbf{x}\right)_i = \left(1 + \hat\kappa\right)\sum_{i=1}^{N-1}\hat V_ix_i + \Delta t\,K_{N-1}^{\prime\,j}x_{N-1}$, with $K_{N-1}^{\prime\,j} = -\hat P_{N-1}^j/\Delta t$ formed from the product of step $j$ as in Section 5.2. By the Decay item, $\mathbf{A}^{*j}\hat{\mathbf{x}}^{j+1} = \hat{\mathbf{b}}^j - \mathbf{E}^j\hat{\mathbf{x}}^{j+1}$ with $\left|\mathbf{E}^j\right| \le c_{\mathbf{A}}^{\mathrm{G}}\left|\hat{\mathbf{A}}^j\right|$. By the Source item, $\sum_{i=1}^{N-1}\hat V_i\hat b_i^j = \mathcal{M}^j + \Delta t\sum_{i=1}^{N-1}\hat Q_i^j + \rho^j$ with $\left|\rho^j\right| \le \chi^j u\,\mathcal{M}^j + \gamma_3\,\Delta t\sum_{i=1}^{N-1}\hat Q_i^j$; where $\hat g_i^j = 0$ the sum $\hat x_i^j + 0$ is exact, which gives the factor $\chi^j$. Define the exact increments of stored numbers $r^{j+1} = \Delta t\left(K_{N-1}^{\prime\,j}\hat x_{N-1}^{j+1} + \hat Q_N^j\right)$, $g^j = \Delta t\sum_{i=1}^{N}\hat Q_i^j$, $d^{j+1} = \hat\kappa\,\mathcal{M}^{j+1}$ (all in $\mathrm{mol}$). Adding and subtracting $\Delta t\,\hat Q_N^j$, as in (27),

$$
\left|\mathcal{M}^{j+1} - \mathcal{M}^j + r^{j+1} - g^j + d^{j+1}\right| \le c_{\mathbf{A}}^{\mathrm{G}}S^{j+1} + \chi^ju\,\mathcal{M}^j + \gamma_3\,g^j.
$$

(b) *Reported amounts.* $\hat M^j = \hat p\,\mathcal{M}^j\left(1 + \theta_N\right)$; $\hat q^{j+1} = \hat p\,r^{j+1}\left(1 + \theta_5\right)$ (both summands of $\hat q$ are non-negative; the first carries the four roundings of its chain and the difference between $\hat K_{N-1,\mathrm{outer}}^j$ and $K_{N-1}^{\prime\,j}$, five factors, the second three); the generation increment is $\hat p\,g^j\left(1 + \theta_{N+1}\right)$; the decay increment is $\hat\kappa\,\hat M^{j+1}\left(1 + \delta\right) = \hat p\,d^{j+1}\left(1 + \theta_{N+1}\right)$; each accumulator adds at most $s_Ju$ times the sum of its computed increments. (c) *Evaluation of $\hat\varepsilon^J$.* With $X = \hat M^J + \hat M_{\mathrm{rel}}^J + \hat M_{\mathrm{dec}}^J - \hat M^0 - \hat M_{\mathrm{gen}}^J$ and the order above, the two inner sums carry $\delta_1$, $\delta_2$, the difference $\delta_3$ and the last sum $\delta_4$, so $\left|\hat\varepsilon^J\right| \le \left(1 + u\right)\left[\left|X\right| + \left(2u + \left(u\right)^2\right)\left(\hat M^J + \hat M_{\mathrm{rel}}^J + \hat M^0 + \hat M_{\mathrm{gen}}^J\right)\right]$, and $2u + \left(u\right)^2 \le \gamma_2$. (d) *Collection.* $X$ is $\hat p$ times the sum over the steps of the left side of (a), plus the differences between the reported amounts and $\hat p$ times their exact counterparts. Bound the first by (a) and the others by (b), and convert every exact quantity into a reported one: $\hat p\,\mathcal{M} \le \hat M/\left(1 - \gamma_N\right)$, $\hat p\sum r \le \hat M_{\mathrm{rel}}^J/\left(\left(1 - \gamma_5\right)\left(1 - s_Ju\right)\right)$, $\hat p\sum g \le \hat M_{\mathrm{gen}}^J/\left(\left(1 - \gamma_{N+1}\right)\left(1 - s_Ju\right)\right)$, $\hat p\sum d \le \hat M_{\mathrm{dec}}^J/\left(\left(1 - \gamma_{N+1}\right)\left(1 - s_Ju\right)\right)$. As in step (b) of R-post, every coefficient is then at most $f_{\mathrm{G}}$ times its coefficient in the bracket of (41), with $f_{\mathrm{G}} = \left(1 + u\right)/\left(\left(1 - \gamma_{N+1}\right)\left(1 - \gamma_5\right)\left(1 - s_Ju\right)\right)$. (e) *Evaluation of the bound and the factor.* Counted as in step (c) of R-post, with seven terms in the bracket and at most six roundings inside $c_{\mathbf{A}}^{\mathrm{G}}$ (computed as $\mathrm{fl}\left(\mathrm{fl}\left(\hat\gamma_4/\mathrm{fl}\left(1 - \hat\gamma_4\right)\right) + \mathrm{fl}\left(\hat\gamma_4/\mathrm{fl}\left(1 - \hat\gamma_1\right)\right)\right)$, counted as for $c_{\mathbf{A}}$), each term carries at most $m = N + J + 15$ roundings. As in step (d) of R-post, the left side of $\left(1 + \eta\right)\left(1 - u\right)^2\left(1 - mu\right)\left(1 - \gamma_{N+1}\right)\left(1 - \gamma_5\right)\left(1 - s_Ju\right) \ge 1 + u$ is at least $\left(1 + \eta\right)\left(1 - a\right)$ with $a = \left(2 + m + 2\left(N + 1\right) + 10 + J\right)u = \left(3N + 2J + 29\right)u$, and $\eta = \left(12N + 4J + 40\right)u \ge 2\left(a + u\right) = \left(6N + 4J + 60\right)u$ holds for $N \ge 4$ (the grid has $N \ge 5$); with $\eta \le 1$ this gives the inequality. Hence the computed value of the right side of (41) bounds $\left|\hat\varepsilon^J\right|$. $\square$

With $G = 0$ and $\lambda = 0$, (41) is a valid but slightly weaker bound than (38) (larger constants $c_{\mathbf{A}}^{\mathrm{G}} \approx 8u$ against $c_{\mathbf{A}} \approx 7u$, $\gamma_5$ against $\gamma_4$, and the $\gamma_2$ evaluation term); for the baseline, (38) or the Rust bound is used (Section 5.7). **Correction of the first version.** The first version of (41) had $c_{\mathbf{A}}^{\mathrm{G}} = \gamma_{11}/\left(1 - \gamma_5\right)$ and $\gamma_4$ on $\hat M_{\mathrm{rel}}^J$, from the reference matrix built from $\hat K$; valid, but not sharp, for the reason given in Section 5.2. The constants are derived for the operation orders written above; a different order changes them by a few units, and the bound must then be re-derived, not assumed. Observed (scratch program of this revision, which evaluates (41) at every step with $J$ the current step count). Synthetic case, $D_{\mathrm{material}}(T)$ as in case 2 of Section 3.2 with $T$ rising linearly from $1000$ to $1100\ \mathrm{K}$ over $2\times10^5\ \mathrm{s}$, $\lambda = 10^{-5}\ \mathrm{s^{-1}}$, $\Delta t = 200\ \mathrm{s}$, 1000 steps, 40 and 80 intervals per layer, and two synthetic sources: kernel only ($G_{\mathrm{kernel}} = 10^{-5}\ \mathrm{mol\ m^{-3}\ s^{-1}}$), and all five materials ($10^{-5}$, $2\times10^{-6}$, $3\times10^{-6}$, $10^{-6}$, $4\times10^{-6}\ \mathrm{mol\ m^{-3}\ s^{-1}}$, so $\bar Q_N \ne 0$). The worst $\left|\varepsilon^j\right|/\left(M^0 + M_{\mathrm{gen}}^j\right)$ was $1.5\times10^{-10}$, $1.1\times10^{-10}$ (40 per layer, kernel and all sources) and $4.2\times10^{-10}$, $3.2\times10^{-10}$ (80 per layer), against final values of (41) of $8.7\times10^{-9}$, $5.7\times10^{-9}$, $3.5\times10^{-8}$, $2.3\times10^{-8}$ of $M^0 + M_{\mathrm{gen}}^J$; the residual stayed below (41) at every step, the largest ratio being $4.0\times10^{-2}$. The smallest concentration was exactly $0$, and $F_{\mathrm{gen}}^J = 0.465$ (kernel source) and $0.664$ (all five sources) on both grids. The first version of this section ran the same cases with another program and reported worst residuals $1.5\times10^{-10}$, $1.1\times10^{-10}$, $4.0\times10^{-10}$, $3.1\times10^{-10}$ against first-version final values $1.2\times10^{-8}$, $7.8\times10^{-9}$, $4.8\times10^{-8}$, $3.1\times10^{-8}$. The present program reproduces those first-version bounds to the printed digits when it uses the first-version constants, and the $F_{\mathrm{gen}}^J$ values; its residuals at 80 per layer differ slightly. (41) leaves the operation order of $D_{\mathrm{material}}(T)$ and of the stored weights $\hat Q_i$ free, and the earlier program was not kept, so the cause of that difference was not traced; both sets are rounding values below their bounds.

---

## 6. Summary of properties

| Property | Statement | Status | Section |
|---|---|---|---|
| P19 Zero-source limit | $G = 0$ reproduces the baseline bit for bit | PROVED | 1.6 |
| P20 Generated amount | (12), (13) give the exact time integral of the discrete source; for a kernel-only source exactly $\frac{4\pi}{3}\left(R_1\right)^3\int G_{\mathrm{kernel}}\,dt$ | PROVED | 1.3, 1.5 |
| P21 No-diffusion limit of the source | the kernel field keeps the start shape (F21) | PROVED | 1.3 |
| P22 Steady state, kernel-only source | discrete steady state equals (14) at every node, any grid and diffusivities; release equals the generation rate | PROVED | 1.6 |
| P22a Steady state, uniform source, one material | exact on a grid with one spacing; local first-order row residual at spacing changes; global second order on unequal spacings | PROVED (exactness, local residual); OBSERVED, NOT PROVEN (global second order) | 1.6 |
| P23 Release with an OPyC source | (12) second-order consistent; (F26) alone first order | PROVED (consistency); OBSERVED (orders $2.00$ and $1.00$) | 1.4 |
| P24 Decay matrix | M-matrix, margin $1 + \kappa$, $\left\|\mathbf{A}^{-1}\right\|_\infty \le 1/\left(1 + \kappa\right)$, pivots $\ge 1 + \kappa + \left|U_i\right|$, P6 unchanged | PROVED | 2.1 |
| P25 Decay relation | (20) | PROVED | 2.3 |
| P26 Stability with $T(t)$ | maximum and weighted norms non-increasing for any history | PROVED | 3.4 |
| P27 Time accuracy with $T(t)$ | first order with (23); end-of-step values add a first-order term $\propto \dot D$ | PROVED (fixed grid); PROOF OUTLINE (grid-uniform constants); OBSERVED | 3.2 |
| P28 Homogeneous sphere with $T(t)$ | exact change of time variable (25) | PROVED | 3.3 |
| P29 Constant temperature | bit for bit equal to the constant case, under the conditions of Section 3.5 | PROVED | 3.5 |
| P30 Combined balance | (27): initial plus generated equals remaining plus released plus decayed, exactly in exact arithmetic | PROVED | 4.2 |
| P31 Combined matrix and solution | solvability, positivity, bounds (28), (29), $0 \le F_{\mathrm{gen}} \le 1$ | PROVED | 4.3, 4.4 |
| R-post, R-prior, R-comb | rigorous rounding bounds of the balance residual: (38) and (40) for the baseline, (41) for the combined scheme | PROVED under (FP), (U), (C), (P); the Kahan constant quoted | 5 |
| Spatial order 2, temporal order 1 with source, decay, $T(t)$ | for the release fraction, inventory and decayed amount | OBSERVED; NOT PROVEN beyond the baseline results | 1.6, 2.3, 3.2 |

Not proven: second-order convergence in space of the combined scheme in any norm beyond what D2 P14 and P15 give for the baseline (the error equation of the combined scheme has the same structure, and the weighted-norm outline of D1 Section 9.5 extends with $\kappa \ge 0$ only helping; that extension is not written out); a priori positivity of the computed pivots for very large $\mathrm{Fo}$; the case of gradual underflow in (R).

---

## 7. Scratch checks

All numbers above come from uncommitted scratch programs (Python with NumPy and mpmath), not part of the repository and not a verification of the Rust solver. They implement D2 row by row, in the operation order of the Rust code on `feat/btcs-solver` (assembly $-\Delta t\,K/V$, diagonal $1 - L - U$, Thomas in the order of (F24) and in the textbook order, inventory by increasing $i$, compensated summation of the release), plus the terms (6) to (9), (12), (16), (19) and (23). The reproduction of all seven measured Rust residuals to the printed digits (Section 5.7) is the check that the scratch program performs the same operations. The 50-digit check of P22 solved the steady rows with mpmath. The revision after the first independent review added three further uncommitted programs, written anew: the time-level comparison of Section 3.2 (both cases), the large-$\mathrm{Fo}$ residuals of Section 5.7, and the combined-scheme residuals against (41) of Section 5.8. They implemented D2 with the elimination order of (F24) but not the exact Rust operation order; the 50-digit steady-state check of P22 was also repeated. The revision after the second independent review (the sharper constant of Sections 5.2 and 5.3) replaced the large-$\mathrm{Fo}$ and combined-scheme programs by one new uncommitted Python program that follows the Rust code of `feat/btcs-solver` operation by operation (grid (F1) to (F7), conductances, assembly, both elimination orders, inventory, release increments, compensated sum, residual) and also evaluates the Rust bound of `balance_bound_history`, with the exact accumulation error computed in rational arithmetic. With the (F24) order it reproduces the current Rust residuals of `solver-core-results.md` (section F) to all printed digits, with the textbook order the 92dac9f residuals, and it reproduces the Rust bound and the Rust ratios of observed residual to bound to all printed digits; it reproduces the first-version values of (38), (40) and (41) when given the first-version constants. The difference $c_{\mathbf{A}} - c_{\mathrm{step}}$ of Section 5.3 was computed in exact rational arithmetic. The revision after the third independent review (the coefficient of $b_J$ in (40)) added one more uncommitted Python program. It rebuilds the grid (F1) to (F7) of the ten cases of Section 5.7, computes $\Lambda_{\max}$ from $\Delta t\left(K_{i,\mathrm{inner}} + K_{i,\mathrm{outer}}\right)/V_i$, and evaluates (40) with the old and the corrected $b_J$, in 60-digit arithmetic (mpmath) and in double precision. It reproduces the $\Lambda_{\max}$ and (40) columns of both tables to the printed digits. With the corrected $b_J$ the relative change is at most $1.3\times10^{-20}$, and the double-precision values are identical. $b_J$ enters no number of Section 5.8. The combined-scheme part implements the operation orders of Section 5.8; for $D_{\mathrm{material}}(T)$ and $\hat Q_i$, which (41) leaves free, it uses the formulas (22), (23) and (6) to (8) as written. The Rust verification suite must re-implement every check it relies on (Rust-first rule).

## 8. Open items for the following tasks

1. Implement (F32) to (F46) of D2 in Rust (pseudocode PS-19, Document 3, gives the hooks), with tests for every row of the table in Section 4.5, keep the derived Rust balance bound for the baseline (its constant equals that of (38) to first order, Section 5.7), and use (41), or the Rust bound extended by the additional terms of (41) with constants re-derived for the code's operation order, for runs with source, decay or $T(t)$ (decision 6).
2. Document 3 (pseudocode) and Document 4 (traceability) do not yet carry (F32) to (F46); they need rows for them.
3. Decide (coordinator or owner) whether generation runs report $F_{\mathrm{gen}}$ of (30) and accept $C_{\mathrm{init}} = 0$ with a source, as Section 4.4 proposes.
4. Register a cited fission-rate density (GAP-28) and, per nuclide, the yield type; until then $G$ is a user input (decision 14).
5. A grid-uniform proof of second-order convergence for the combined scheme, and an a priori proof of (P) for large $\mathrm{Fo}$, remain open.
