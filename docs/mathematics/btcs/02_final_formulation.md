# STRATO: Final Five-Layer BTCS Formulation

**Status.** This is Document 2 of directive Section 9: the complete five-layer BTCS system, with nothing left to choose. It states the grid, every matrix row, the right-hand side, the initial vector, the solution of the linear system, the discrete inventory, the release rate, the cumulative release, the balance identity and its residual, and each proven property with its hypotheses. Section 11 adds generation, decay and temperature-dependent diffusivity (directive Section 17) as (F32) to (F42), and Section 11.7 the rigorous rounding bounds (F43) to (F46) of the balance residual; their derivations and proofs are in [Document 5](05_generation_decay_temperature.md), cited as "D5".

Every equation is numbered (F1), (F2), and so on. The pseudocode (Document 3) and the Rust code cite these numbers. The derivation of every equation is in [Document 1](01_from_ftcs_to_five_layer_btcs.md), cited as "D1" with its equation or section number. The comparison of the two independent derivations that led to this formulation is [reconciliation.md](reconciliation.md).

This document states mathematics. It contains no numerical results except where a property is labelled OBSERVED, and those come from scratch programs that were used only to check the algebra. Nothing here is experimental validation.

**Labels.** PROVED: a complete proof is in D1. PROOF OUTLINE: every step is in D1 and was checked, but the constants are not tracked. OBSERVED: shown only by scratch computations. NOT PROVEN: open.

---

## 1. Scope and model

- One species diffuses radially in a sphere of five concentric materials: kernel, buffer, IPyC, SiC and OPyC.
- Each material has one constant diffusivity.
- Perfect contact at the four interfaces: concentration and diffusive flux are continuous.
- Absorbing outer surface: $C(R_5, t) = 0$.
- Kernel-only start: $C(r, 0) = C_{\mathrm{init}}$ for $0 \le r < R_1$ and $0$ for $R_1 < r < R_5$.
- Sections 3 to 8 and 10: no source, no decay, constant temperature (the baseline). Section 11 adds the three approved mechanisms.

The continuous equation is $\partial C/\partial t = \left(r\right)^{-2}\,\partial_r\left[\left(r\right)^2 D(r)\,\partial_r C\right]$ (D1, Eq. (4)).

---

## 2. Symbols and units

The binding notation of directive Section 5 applies: $i$ is the global radial node index, $j$ the time-step index, $C_i^j$ the concentration at node $i$ and time step $j$. A time superscript is an index, not a power. A power encloses the whole variable in parentheses.

**Inputs.**

| Symbol | Meaning | Unit | Condition |
|---|---|---|---|
| $R_1, R_2, R_3, R_4, R_5$ | outer radii of kernel, buffer, IPyC, SiC, OPyC; $R_0 = 0$ | $\mathrm{m}$ | $0 < R_1 < R_2 < R_3 < R_4 < R_5$ |
| $D_{\mathrm{kernel}}, D_{\mathrm{buffer}}, D_{\mathrm{IPyC}}, D_{\mathrm{SiC}}, D_{\mathrm{OPyC}}$ | diffusivity of each material | $\mathrm{m^2\ s^{-1}}$ | each $> 0$ and finite |
| $n_{\mathrm{kernel}}, n_{\mathrm{buffer}}, n_{\mathrm{IPyC}}, n_{\mathrm{SiC}}, n_{\mathrm{OPyC}}$ | number of grid intervals in each material | dimensionless (integer) | each $\ge 1$ |
| $C_{\mathrm{init}}$ | initial concentration in the kernel | $\mathrm{mol\ m^{-3}}$ | $\ge 0$ |
| $\Delta t$ | time step | $\mathrm{s}$ | $> 0$ |
| $J_{\mathrm{steps}}$ | number of time steps | dimensionless (integer) | $\ge 1$ |

$C_{\mathrm{init}}$ is not written $C_0$, because $C_0^j$ is the concentration at the centre node (D1, Section 0.2).

**Derived quantities.**

| Symbol | Meaning | Unit |
|---|---|---|
| $N$ | index of the outer node, $N = n_{\mathrm{kernel}} + n_{\mathrm{buffer}} + n_{\mathrm{IPyC}} + n_{\mathrm{SiC}} + n_{\mathrm{OPyC}}$ | dimensionless |
| $N_{\mathrm{kernel}}, N_{\mathrm{buffer}}, N_{\mathrm{IPyC}}, N_{\mathrm{SiC}}$ | node index of $R_1, R_2, R_3, R_4$ | dimensionless |
| $r_i$ | radius of node $i$, $i = 0, \dots, N$ | $\mathrm{m}$ |
| $\Delta r_{\mathrm{kernel}}, \dots, \Delta r_{\mathrm{OPyC}}$ | uniform spacing inside each material | $\mathrm{m}$ |
| $\Delta r_{i,\mathrm{inner}}, \Delta r_{i,\mathrm{outer}}$ | spacing on the inner and outer side of node $i$ | $\mathrm{m}$ |
| $D_{i,\mathrm{inner}}, D_{i,\mathrm{outer}}$ | diffusivity of the material on the inner and outer side of node $i$ | $\mathrm{m^2\ s^{-1}}$ |
| $\mathrm{Fo}_{\mathrm{kernel}}, \dots, \mathrm{Fo}_{\mathrm{OPyC}}$ | mesh Fourier number of each material | dimensionless |
| $K_{i,\mathrm{outer}}$ | conductance of the interval $(r_i, r_{i+1})$, divided by $4\pi$ | $\mathrm{m^3\ s^{-1}}$ |
| $V_i$ | storage weight of node $i$ (nodal volume divided by $4\pi$) | $\mathrm{m^3}$ |
| $L_i, B_i, U_i$ | lower, diagonal and upper coefficient of matrix row $i$ | dimensionless |
| $b_i^j$ | right-hand side of row $i$ in the step from $j$ to $j+1$ | $\mathrm{mol\ m^{-3}}$ |
| $\theta$ | initial value at node $N_{\mathrm{kernel}}$ as a fraction of $C_{\mathrm{init}}$ | dimensionless |
| $t^j$ | time at step $j$, $t^j = j\,\Delta t$ | $\mathrm{s}$ |
| $M^j$ | discrete inventory | $\mathrm{mol}$ |
| $\Phi^j$ | discrete release rate through $R_5$ | $\mathrm{mol\ s^{-1}}$ |
| $M_{\mathrm{rel}}^j$ | cumulative release | $\mathrm{mol}$ |
| $F^j$ | release fraction | dimensionless |
| $\varepsilon^j$ | balance residual | $\mathrm{mol}$ |

The symbols of the additional physics (Section 11) are defined with their units where they first appear there and in D5, Section 0.2.

---

## 3. Grid

**Interface node indices.**

$$
N_{\mathrm{kernel}} = n_{\mathrm{kernel}}, \quad N_{\mathrm{buffer}} = N_{\mathrm{kernel}} + n_{\mathrm{buffer}}, \quad N_{\mathrm{IPyC}} = N_{\mathrm{buffer}} + n_{\mathrm{IPyC}}, \quad N_{\mathrm{SiC}} = N_{\mathrm{IPyC}} + n_{\mathrm{SiC}}, \quad N = N_{\mathrm{SiC}} + n_{\mathrm{OPyC}}. \tag{F1}
$$

**Spacing in each material** (piecewise-uniform grid, D1 Section 8.1):

$$
\Delta r_{\mathrm{kernel}} = \frac{R_1}{n_{\mathrm{kernel}}}, \quad \Delta r_{\mathrm{buffer}} = \frac{R_2 - R_1}{n_{\mathrm{buffer}}}, \quad \Delta r_{\mathrm{IPyC}} = \frac{R_3 - R_2}{n_{\mathrm{IPyC}}}, \quad \Delta r_{\mathrm{SiC}} = \frac{R_4 - R_3}{n_{\mathrm{SiC}}}, \quad \Delta r_{\mathrm{OPyC}} = \frac{R_5 - R_4}{n_{\mathrm{OPyC}}}. \tag{F2}
$$

**Node radii.** For a material with inner radius $R_{\mathrm{prev}}$, inner node index $N_{\mathrm{prev}}$, outer radius $R_{\mathrm{next}}$ and outer node index $N_{\mathrm{next}}$ (for the kernel: $R_{\mathrm{prev}} = 0$, $N_{\mathrm{prev}} = 0$, $R_{\mathrm{next}} = R_1$, $N_{\mathrm{next}} = N_{\mathrm{kernel}}$):

$$
r_i = R_{\mathrm{prev}} + \left(i - N_{\mathrm{prev}}\right)\Delta r_{\mathrm{material}} \quad \left(N_{\mathrm{prev}} < i < N_{\mathrm{next}}\right), \qquad r_0 = 0, \quad r_{N_{\mathrm{kernel}}} = R_1, \quad r_{N_{\mathrm{buffer}}} = R_2, \quad r_{N_{\mathrm{IPyC}}} = R_3, \quad r_{N_{\mathrm{SiC}}} = R_4, \quad r_N = R_5. \tag{F3}
$$

The interface and boundary radii are assigned exactly, not accumulated by addition, so that rounding cannot move an interface.

**Material of each interval.** Every interval $(r_i, r_{i+1})$, $0 \le i \le N-1$, lies inside exactly one material, because every interface radius is a node. Its diffusivity is

$$
D_{i,\mathrm{outer}} = \begin{cases}
D_{\mathrm{kernel}}, & 0 \le i < N_{\mathrm{kernel}},\\
D_{\mathrm{buffer}}, & N_{\mathrm{kernel}} \le i < N_{\mathrm{buffer}},\\
D_{\mathrm{IPyC}}, & N_{\mathrm{buffer}} \le i < N_{\mathrm{IPyC}},\\
D_{\mathrm{SiC}}, & N_{\mathrm{IPyC}} \le i < N_{\mathrm{SiC}},\\
D_{\mathrm{OPyC}}, & N_{\mathrm{SiC}} \le i < N,
\end{cases}
\qquad D_{i,\mathrm{inner}} = D_{i-1,\mathrm{outer}} \quad (1 \le i \le N). \tag{F4}
$$

**Node classes.** Node $0$ is the centre. Nodes $N_{\mathrm{kernel}}, N_{\mathrm{buffer}}, N_{\mathrm{IPyC}}, N_{\mathrm{SiC}}$ are the four interface nodes. Node $N$ is the outer boundary node. Every other node is an interior node of the material that contains it.

---

## 4. Geometric coefficients

**Spacings.**

$$
\Delta r_{i,\mathrm{outer}} = r_{i+1} - r_i \quad (0 \le i \le N-1), \qquad \Delta r_{i,\mathrm{inner}} = r_i - r_{i-1} = \Delta r_{i-1,\mathrm{outer}} \quad (1 \le i \le N). \tag{F5}
$$

**Interval conductance** (exact steady conductance of a spherical shell, divided by $4\pi$; D1, Eq. (19)):

$$
K_{i,\mathrm{outer}} = \frac{D_{i,\mathrm{outer}}\,r_i\,r_{i+1}}{\Delta r_{i,\mathrm{outer}}} \quad (0 \le i \le N-1), \qquad K_{i,\mathrm{inner}} = K_{i-1,\mathrm{outer}} \quad (1 \le i \le N). \tag{F6}
$$

Because $r_0 = 0$, $K_{0,\mathrm{outer}} = K_{1,\mathrm{inner}} = 0$ exactly. The implementation sets it to zero, not to a computed product.

**Storage weight** (half of each adjacent interval, the trapezoidal weight; D1, Eqs. (29) and (43)):

$$
V_i = \left(r_i\right)^2\,\frac{\Delta r_{i,\mathrm{inner}} + \Delta r_{i,\mathrm{outer}}}{2} \quad (1 \le i \le N-1), \qquad V_0 = 0. \tag{F7}
$$

$V_0 = 0$ is part of the scheme. No centre weight such as $\frac{1}{3}\left(\Delta r_{\mathrm{kernel}}/2\right)^3$ may be used anywhere (D1, Section 4.3, item 5).

**Mesh Fourier numbers.**

$$
\mathrm{Fo}_{\mathrm{material}} = \frac{D_{\mathrm{material}}\,\Delta t}{\left(\Delta r_{\mathrm{material}}\right)^2}, \qquad \text{material} \in \left\{\mathrm{kernel}, \mathrm{buffer}, \mathrm{IPyC}, \mathrm{SiC}, \mathrm{OPyC}\right\}. \tag{F8}
$$

---

## 5. The matrix rows

The unknowns of the step from $t^j$ to $t^{j+1}$ are $C_0^{j+1}, C_1^{j+1}, \dots, C_{N-1}^{j+1}$: $N$ unknowns, $N$ rows. $C_N^{j+1}$ is not an unknown (F18). Row $i$ reads

$$
L_i\,C_{i-1}^{j+1} + B_i\,C_i^{j+1} + U_i\,C_{i+1}^{j+1} = b_i^j, \tag{F9}
$$

with the term $L_0\,C_{-1}^{j+1}$ absent in row 0 and the term $U_{N-1}\,C_N^{j+1}$ absent in row $N-1$.

### 5.1 The general row, $1 \le i \le N-1$ (normative)

Every row $1 \le i \le N-1$, whatever the class of node $i$, is the backward-Euler flux balance (D1, Eqs. (30), (31), (43)):

$$
V_i\,\frac{C_i^{j+1} - C_i^j}{\Delta t} = K_{i,\mathrm{outer}}\left(C_{i+1}^{j+1} - C_i^{j+1}\right) - K_{i,\mathrm{inner}}\left(C_i^{j+1} - C_{i-1}^{j+1}\right), \tag{F10}
$$

and its coefficients are

$$
L_i = -\frac{\Delta t\,K_{i,\mathrm{inner}}}{V_i}, \qquad U_i = -\frac{\Delta t\,K_{i,\mathrm{outer}}}{V_i}, \qquad B_i = 1 - L_i - U_i = 1 + \left|L_i\right| + \left|U_i\right|. \tag{F11}
$$

**Assembly rule.** The implementation computes $L_i$, $B_i$ and $U_i$ for $1 \le i \le N-1$ from (F11), with (F5) to (F7). The explicit forms of Sections 5.3 to 5.6 are the same numbers written out by material. They are checks: in exact arithmetic they equal (F11), and in floating point they agree with it to rounding. In row $N-1$, $U_{N-1}$ is computed by (F11) and enters $B_{N-1}$, but it is not a matrix entry (Section 5.6).

### 5.2 The centre row, $i = 0$

From the limit $3D_{\mathrm{kernel}}\,\partial^2 C/\partial r^2$ at $r = 0$ and the symmetric ghost node $C_{-1} = C_1$ (D1, Eqs. (23) to (25)):

$$
\left(1 + 6\,\mathrm{Fo}_{\mathrm{kernel}}\right)C_0^{j+1} - 6\,\mathrm{Fo}_{\mathrm{kernel}}\,C_1^{j+1} = b_0^j, \qquad B_0 = 1 + 6\,\mathrm{Fo}_{\mathrm{kernel}}, \quad U_0 = -6\,\mathrm{Fo}_{\mathrm{kernel}}. \tag{F12}
$$

Row 0 is not a flux balance; $V_0 = 0$, so it carries no inventory.

### 5.3 Node 1 and the one-way coupling

By (F6), $K_{1,\mathrm{inner}} = 0$, so by (F11)

$$
L_1 = 0. \tag{F13}
$$

Rows $1$ to $N-1$ do not contain $C_0^{j+1}$. The centre value is driven by node 1 and drives nothing (D1, Section 4.3).

### 5.4 Interior rows of each material

Let node $i$ lie strictly inside one material, so that both adjacent intervals belong to it and have its spacing. Then (F11) equals (D1, Eq. (15)):

$$
L_i = -\mathrm{Fo}_{\mathrm{material}}\left(1 - \frac{\Delta r_{\mathrm{material}}}{r_i}\right), \qquad B_i = 1 + 2\,\mathrm{Fo}_{\mathrm{material}}, \qquad U_i = -\mathrm{Fo}_{\mathrm{material}}\left(1 + \frac{\Delta r_{\mathrm{material}}}{r_i}\right). \tag{F14}
$$

The five ranges of interior nodes, with the radius of each node:

| Material | Interior nodes | $r_i$ | Form of the factor |
|---|---|---|---|
| kernel | $1 \le i \le N_{\mathrm{kernel}} - 1$ | $i\,\Delta r_{\mathrm{kernel}}$ | $\Delta r_{\mathrm{kernel}}/r_i = 1/i$, the owner's factor (F15) |
| buffer | $N_{\mathrm{kernel}} < i < N_{\mathrm{buffer}}$ | $R_1 + \left(i - N_{\mathrm{kernel}}\right)\Delta r_{\mathrm{buffer}}$ | $\Delta r_{\mathrm{buffer}}/r_i$ |
| IPyC | $N_{\mathrm{buffer}} < i < N_{\mathrm{IPyC}}$ | $R_2 + \left(i - N_{\mathrm{buffer}}\right)\Delta r_{\mathrm{IPyC}}$ | $\Delta r_{\mathrm{IPyC}}/r_i$ |
| SiC | $N_{\mathrm{IPyC}} < i < N_{\mathrm{SiC}}$ | $R_3 + \left(i - N_{\mathrm{IPyC}}\right)\Delta r_{\mathrm{SiC}}$ | $\Delta r_{\mathrm{SiC}}/r_i$ |
| OPyC | $N_{\mathrm{SiC}} < i < N$ | $R_4 + \left(i - N_{\mathrm{SiC}}\right)\Delta r_{\mathrm{OPyC}}$ | $\Delta r_{\mathrm{OPyC}}/r_i$ |

A range is empty when the material has a single interval. In the kernel, (F14) with $\Delta r_{\mathrm{kernel}}/r_i = 1/i$ is

$$
L_i = -\mathrm{Fo}_{\mathrm{kernel}}\left(1 - \frac{1}{i}\right), \qquad B_i = 1 + 2\,\mathrm{Fo}_{\mathrm{kernel}}, \qquad U_i = -\mathrm{Fo}_{\mathrm{kernel}}\left(1 + \frac{1}{i}\right), \qquad 1 \le i \le N_{\mathrm{kernel}} - 1. \tag{F15}
$$

The factor $1/i$ is valid only in the kernel. In the coatings $r_i \ne i\,\Delta r_{\mathrm{material}}$.

### 5.5 The four interface rows

Let node $i$ lie on an interface at radius $R$, with inner material "in" and outer material "out". Then (F11) equals (D1, Eqs. (32a) and (32b)):

$$
L_i = -\mathrm{Fo}_{\mathrm{in}}\,\frac{2\,\Delta r_{\mathrm{in}}}{\Delta r_{\mathrm{in}} + \Delta r_{\mathrm{out}}}\left(1 - \frac{\Delta r_{\mathrm{in}}}{R}\right), \qquad U_i = -\mathrm{Fo}_{\mathrm{out}}\,\frac{2\,\Delta r_{\mathrm{out}}}{\Delta r_{\mathrm{in}} + \Delta r_{\mathrm{out}}}\left(1 + \frac{\Delta r_{\mathrm{out}}}{R}\right), \qquad B_i = 1 - L_i - U_i. \tag{F16}
$$

Here $\mathrm{Fo}_{\mathrm{in}}$ and $\mathrm{Fo}_{\mathrm{out}}$ are the mesh Fourier numbers (F8) of the inner and outer materials, and $\Delta r_{\mathrm{in}}$ and $\Delta r_{\mathrm{out}}$ are their spacings (F2), so that $\Delta r_{\mathrm{in}} = \Delta r_{i,\mathrm{inner}}$ and $\Delta r_{\mathrm{out}} = \Delta r_{i,\mathrm{outer}}$. The four interface nodes:

| Row | $R$ | in | out | $\Delta r_{\mathrm{in}}$ | $\Delta r_{\mathrm{out}}$ | Eq. |
|---|---|---|---|---|---|---|
| $i = N_{\mathrm{kernel}}$ | $R_1$ | kernel | buffer | $\Delta r_{\mathrm{kernel}}$ | $\Delta r_{\mathrm{buffer}}$ | (F16a) |
| $i = N_{\mathrm{buffer}}$ | $R_2$ | buffer | IPyC | $\Delta r_{\mathrm{buffer}}$ | $\Delta r_{\mathrm{IPyC}}$ | (F16b) |
| $i = N_{\mathrm{IPyC}}$ | $R_3$ | IPyC | SiC | $\Delta r_{\mathrm{IPyC}}$ | $\Delta r_{\mathrm{SiC}}$ | (F16c) |
| $i = N_{\mathrm{SiC}}$ | $R_4$ | SiC | OPyC | $\Delta r_{\mathrm{SiC}}$ | $\Delta r_{\mathrm{OPyC}}$ | (F16d) |

Written out, the kernel-buffer row (F16a) is

$$
L_{N_{\mathrm{kernel}}} = -\frac{2D_{\mathrm{kernel}}\,\Delta t\left(R_1 - \Delta r_{\mathrm{kernel}}\right)}{R_1\,\Delta r_{\mathrm{kernel}}\left(\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}\right)}, \qquad U_{N_{\mathrm{kernel}}} = -\frac{2D_{\mathrm{buffer}}\,\Delta t\left(R_1 + \Delta r_{\mathrm{buffer}}\right)}{R_1\,\Delta r_{\mathrm{buffer}}\left(\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}\right)},
$$

and (F16b) to (F16d) follow by the substitutions of the table. The interface node carries one value, shared by both materials: concentration continuity is exact. The inner flow uses only the inner diffusivity and the outer flow only the outer diffusivity; no diffusivity is averaged, and no constant-diffusivity stencil is applied across the jump. If $n_{\mathrm{kernel}} = 1$, then $N_{\mathrm{kernel}} = 1$ and (F16a) gives $L_1 = 0$, in agreement with (F13).

### 5.6 The row next to the outer boundary, $i = N-1$

Row $N-1$ is computed by (F11) like every other row. Its upper neighbour is the boundary value $C_N^{j+1} = 0$ (F18), so the term $U_{N-1}\,C_N^{j+1}$ is zero and is dropped. The diagonal keeps the full value:

$$
L_{N-1}\,C_{N-2}^{j+1} + B_{N-1}\,C_{N-1}^{j+1} = b_{N-1}^j, \qquad B_{N-1} = 1 + \left|L_{N-1}\right| + \left|U_{N-1}\right|. \tag{F17}
$$

- If $n_{\mathrm{OPyC}} \ge 2$, node $N-1$ is an OPyC interior node and (F14) gives $L_{N-1} = -\mathrm{Fo}_{\mathrm{OPyC}}\left(1 - \Delta r_{\mathrm{OPyC}}/r_{N-1}\right)$ and $B_{N-1} = 1 + 2\,\mathrm{Fo}_{\mathrm{OPyC}}$.
- If $n_{\mathrm{OPyC}} = 1$, node $N-1$ is the interface node $N_{\mathrm{SiC}}$, and (F16d) gives $L_{N-1}$ and $B_{N-1}$.

No boundary value enters the right-hand side, because the boundary value is zero.

### 5.7 The outer boundary

$$
C_N^j = 0 \qquad \text{for every } j \ge 0. \tag{F18}
$$

This is the absorbing (Dirichlet) condition $C(R_5, t) = 0$. Node $N$ has no row and no ghost node. The owner's insulated outer row ((T23) of document 00) is not used.

### 5.8 The assembled system

$$
\mathbf{A}\,\mathbf{C}^{j+1} = \mathbf{b}^j, \qquad
\mathbf{A} =
\begin{pmatrix}
B_0 & U_0 & & & & \\
0 & B_1 & U_1 & & & \\
 & L_2 & B_2 & U_2 & & \\
 & & \ddots & \ddots & \ddots & \\
 & & & L_{N-2} & B_{N-2} & U_{N-2} \\
 & & & & L_{N-1} & B_{N-1}
\end{pmatrix},
\qquad
\mathbf{C}^{j+1} = \begin{pmatrix} C_0^{j+1} \\ C_1^{j+1} \\ \vdots \\ C_{N-1}^{j+1} \end{pmatrix}. \tag{F19}
$$

$\mathbf{A}$ is $N \times N$ and tridiagonal. The zero in position $(1, 0)$ is (F13). With a constant $\Delta t$ and constant diffusivities, $\mathbf{A}$ is the same in every step and is assembled once. If $\Delta t$ or any diffusivity changes, $\mathbf{A}$ is assembled again from (F8) to (F17) with the new values.

---

## 6. Right-hand side, initial vector and solution

### 6.1 Right-hand side

$$
b_i^j = C_i^j, \qquad 0 \le i \le N-1. \tag{F20}
$$

(With a source, (F38) of Section 11.3 replaces (F20).)

### 6.2 Initial vector

$$
C_i^0 = C_{\mathrm{init}} \quad \left(0 \le i \le N_{\mathrm{kernel}} - 1\right), \qquad C_{N_{\mathrm{kernel}}}^0 = \theta\,C_{\mathrm{init}}, \qquad C_i^0 = 0 \quad \left(N_{\mathrm{kernel}} + 1 \le i \le N\right), \tag{F21}
$$

with the inventory-matched kernel-surface value (D1, Eqs. (40), (41), (41a))

$$
\theta = \frac{\Delta r_{\mathrm{kernel}}}{\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}}\left(1 - \frac{1}{3\,n_{\mathrm{kernel}}}\right). \tag{F22}
$$

(F22) is the solution of

$$
\sum_{i=1}^{N_{\mathrm{kernel}}-1}V_i + \theta\,V_{N_{\mathrm{kernel}}} = \frac{\left(R_1\right)^3}{3}, \tag{F23}
$$

which makes the discrete initial inventory (F25) equal to the exact one, $M^0 = \frac{4\pi}{3}\left(R_1\right)^3C_{\mathrm{init}}$. The implementation uses (F22); (F23) is its check. Always $0 < \theta < 1$. $\theta = \frac{1}{2}$, $0$ or $1$ are not used: on unequal spacings at $R_1$ they reduce the spatial order of every later release fraction to one (D1, Section 7).

### 6.3 Solution of the linear system

The system (F19) is solved by the Thomas algorithm, coded in this form (D1, Sections 9.2 (vi) and 9.3):

$$
\begin{aligned}
&m_0 = B_0, \qquad b'_0 = b_0^j,\\
&w_i = L_i/m_{i-1}, \quad m_i = B_i - w_i\,U_{i-1}, \quad b'_i = b_i^j - w_i\,b'_{i-1}, \qquad i = 1, \dots, N-1,\\
&C_{N-1}^{j+1} = b'_{N-1}/m_{N-1}, \qquad C_i^{j+1} = \left(b'_i - U_i\,C_{i+1}^{j+1}\right)/m_i, \qquad i = N-2, \dots, 0.
\end{aligned} \tag{F24}
$$

$L_i$ and $U_i$ are stored with their signs ($\le 0$). No pivoting is used or needed (Property P5). After the solve, $C_N^{j+1} = 0$ by (F18).

---

## 7. Inventory, release and balance

### 7.1 Discrete inventory

$$
M^j = 4\pi\sum_{i=1}^{N-1}V_i\,C_i^j. \tag{F25}
$$

The sum may equally run from $i = 0$ to $i = N$, because $V_0 = 0$ and $C_N^j = 0$. It is the composite trapezoidal rule for $4\pi\int_0^{R_5}\left(r\right)^2C\,dr$ on the nodes (D1, Section 3.5).

### 7.2 Release rate

The outward flow through the last interval, at the new time level (D1, Eq. (36)):

$$
\Phi^{j+1} = 4\pi\,K_{N-1,\mathrm{outer}}\,C_{N-1}^{j+1}, \qquad j \ge 0. \tag{F26}
$$

**Normative rule.** $K_{N-1,\mathrm{outer}}$ in (F26) is the stored value computed by (F6) with the spacing $\Delta r_{N-1,\mathrm{outer}} = r_N - r_{N-1}$ of (F5): the same number that enters $U_{N-1}$ in (F11) and hence $B_{N-1}$ in (F17). It is not recomputed. Written out, $K_{N-1,\mathrm{outer}} = D_{\mathrm{OPyC}}\,r_{N-1}\,R_5/\Delta r_{N-1,\mathrm{outer}}$, and in exact arithmetic $\Delta r_{N-1,\mathrm{outer}} = \Delta r_{\mathrm{OPyC}}$ of (F2). That second form is explanatory only: in floating point $r_N - r_{N-1}$ and $\left(R_5 - R_4\right)/n_{\mathrm{OPyC}}$ can differ by rounding, and a release computed with a different number from the one in row $N-1$ would add a residual to (F29).

$\Phi^0$ is not defined and not needed. (F26) is the only release formula for which the balance (F29) is exact; it uses the same $K_{N-1,\mathrm{outer}}$ as row $N-1$ and the same time level as the implicit flows.

### 7.3 Cumulative release and release fraction

$$
M_{\mathrm{rel}}^0 = 0, \qquad M_{\mathrm{rel}}^{j+1} = M_{\mathrm{rel}}^j + \Delta t\,\Phi^{j+1}, \qquad\text{so}\qquad M_{\mathrm{rel}}^j = \sum_{k=1}^{j}\Delta t\,\Phi^k. \tag{F27}
$$

If the step length changes, $\Delta t$ in (F27) is the length of the step that produced $\Phi^{k}$.

$$
F^j = \frac{M_{\mathrm{rel}}^j}{M^0}. \tag{F28}
$$

$F^j$ is the reported release fraction. The complementary value $1 - M^j/M^0$ is a diagnostic only; it differs from $F^j$ by the relative residual of (F30).

### 7.4 Balance identity and residual

Summing (F10) over $i = 1, \dots, N-1$ telescopes (D1, Eqs. (21), (35), (37)):

$$
M^{j+1} - M^j = -\Delta t\,\Phi^{j+1}, \qquad M^j + M_{\mathrm{rel}}^j = M^0 \quad \text{for every } j \ge 0 \ \text{(exact arithmetic)}. \tag{F29}
$$

In floating point the identity holds only to rounding. The residual reported at every output step is

$$
\varepsilon^j = M^j + M_{\mathrm{rel}}^j - M^0, \qquad \frac{\varepsilon^j}{M^0} = F^j - \left(1 - \frac{M^j}{M^0}\right). \tag{F30}
$$

The tolerance that a test applies to $\left|\varepsilon^j\right|$ is the rigorous rounding bound (F44), evaluated at every step, with the a-priori form (F45), and, for a run with source, decay or $T(t)$, the bound (F46) (Section 11.7; derivation in D5, Section 5). Both contain the largest mesh Fourier number and the number of steps.

### 7.5 Diagnostics at the interfaces and the centre

**Interface flow.** At an interface node $i$ with radius $R$, the outward flow at $r = R$ is reported as (D1, Eq. (30a))

$$
\Phi_{R}^{j+1} = 4\pi K_{i,\mathrm{inner}}\left(C_{i-1}^{j+1} - C_i^{j+1}\right) - 4\pi\left(R\right)^2\frac{\Delta r_{i,\mathrm{inner}}}{2}\,\frac{C_i^{j+1} - C_i^j}{\Delta t} = 4\pi K_{i,\mathrm{outer}}\left(C_i^{j+1} - C_{i+1}^{j+1}\right) + 4\pi\left(R\right)^2\frac{\Delta r_{i,\mathrm{outer}}}{2}\,\frac{C_i^{j+1} - C_i^j}{\Delta t}. \tag{F31}
$$

The two expressions are equal in exact arithmetic (Property P8). Their difference in floating point is a diagnostic. (F31) is consistent to second order (P8a); the second-order convergence of the computed value is observed only (P8b).

**Centre value.** $C_0^j$ from (F12) is reported as the concentration at $r = 0$. It does not enter (F25) to (F30).

---

## 8. Proven properties and their hypotheses

**Standing hypotheses (H).** The grid is (F1) to (F3) with every $n_{\mathrm{material}} \ge 1$; every diffusivity is positive and finite; $\Delta t > 0$; the rows are (F11), (F12) and (F17). No condition relates $\Delta t$ to the spacing, and no condition is placed on the ratios of spacings or of diffusivities.

**Error quantities used in the table.** $e_i^j = C(r_i, t^j) - C_i^j$ is the nodal error (in $\mathrm{mol\ m^{-3}}$), with $C(r, t)$ the exact solution; $\tau_i^j$ is the residual of row $i$ when the exact solution is inserted, written so that the row reads "time difference minus spatial operator" (in $\mathrm{mol\ m^{-3}\ s^{-1}}$); $\left\|\mathbf{e}^j\right\|_\infty = \max_i\left|e_i^j\right|$, and $\left\|\mathbf{e}^j\right\|_V = \left(\sum_{i=1}^{N-1}V_i\left(e_i^j\right)^2\right)^{1/2}$ (in $\mathrm{mol\ m^{-3/2}}$). $T$ is the end time and $t_* \ge 0$ a start time of the convergence statement (both in $\mathrm{s}$); $\Delta r$ in an order statement means the largest spacing of the grid.

| Property | Statement | Hypotheses | Status | Source |
|---|---|---|---|---|
| P1 Sign pattern | $L_i \le 0$, $U_i < 0$, $B_i > 0$ in every row; $B_i - \left\lvert L_i\right\rvert  - \left\lvert U_i\right\rvert _{\text{in matrix}} \ge 1$ | (H) | PROVED | D1, Eq. (45) |
| P2 Solvability | $\mathbf{A}$ is a non-singular M-matrix; $\mathbf{A}^{-1} \ge 0$ entrywise; $\left\lVert \mathbf{A}^{-1}\right\rVert _\infty \le 1$; every step has exactly one solution | (H) | PROVED | D1, Eqs. (46), (46a) |
| P3 Stability | $\max_i\left\lvert C_i^{j+1}\right\rvert  \le \max_i\left\lvert C_i^j\right\rvert$; $\sum_{i\ge1}V_i\left(C_i^{j+1}\right)^2 \le \sum_{i\ge1}V_i\left(C_i^j\right)^2$ | (H) | PROVED | D1, Eqs. (46), (46b) |
| P4 Positivity, maximum principle | if $0 \le C_i^j \le C_{\max}$ for all $i$, then $0 \le C_i^{j+1} \le C_{\max}$; with (F21), $0 \le C_i^j \le C_{\mathrm{init}}$ for all $i, j$; hence $\Phi^j \ge 0$, $F^j$ non-decreasing, $0 \le F^j \le 1$ | (H), $C_{\mathrm{init}} \ge 0$ | PROVED | D1, Section 9.2 (ii), (iii); Section 6 |
| P5 Thomas algorithm | pivots $m_i \ge 1 + \left\lvert U_i\right\rvert$; multipliers $\left\lvert U_i\right\rvert /m_i < 1$; no pivoting needed; cost proportional to $N$ | (H) | PROVED | D1, Section 9.3 |
| P6 Positivity in floating point | (F24) with $\mathbf{b} \ge 0$ returns a computed solution $\ge 0$ | (H), (F24) coded as written, every computed pivot $m_i > 0$ | PROVED | D1, Section 9.2 (vi) |
| P7 One-way centre coupling | $L_1 = 0$; rows $1$ to $N-1$, the inventory, the release and (F29) do not depend on row 0 or on $C_0^j$; $\left\lvert e_0^j\right\rvert  \le \max\left(\left\lvert e_0^0\right\rvert , \max_{k\le j}\left\lvert e_1^k\right\rvert \right) + t^j\max\left\lvert \tau_0\right\rvert$ | (H) | PROVED | D1, Section 4.3 |
| P8 Interface continuity | one shared value per interface node (concentration continuity exact); the two expressions of (F31) are equal | (H) | PROVED | D1, Section 5.4 |
| P8a Interface flux formula (consistency) | (F31) evaluated on the exact solution equals the exact interface flow $\Phi(R, t^{j+1})$ to $O\left(\left(\Delta r\right)^2 + \Delta r\,\Delta t\right)$ | (H); solution three times continuously differentiable in $r$ on each closed layer and twice continuously differentiable in $t$ | PROVED | D1, Section 5.4 |
| P8b Computed interface flux | (F31) evaluated on the computed solution: the differences between successive grids fall at second order in space (self-convergence) | as P8a | OBSERVED (orders $1.98$ to $2.00$, synthetic case). Convergence to the exact interface flow is not shown: no exact five-layer interface flow was computed, so the limit of the sequence was not compared with a reference. NOT PROVEN: no order is proved, because the proved nodal error bounds, divided by $\Delta r$, give none | D1, Section 5.4 |
| P9 Conservation | (F29) holds exactly in exact arithmetic | (H); the same stored $K$ in rows $i$, $i+1$ and in (F26), in particular the stored $K_{N-1,\mathrm{outer}}$ of (F6) with (F5) in row $N-1$ and in (F26); $V_i$ the weight by which row $i$ was divided; $V_0 = 0$ with $K_{0,\mathrm{outer}} = 0$; release at level $j+1$ | PROVED | D1, Eqs. (21), (35), (37) |
| P10 Exact initial inventory | $M^0 = \frac{4\pi}{3}\left(R_1\right)^3C_{\mathrm{init}}$ | (F21), (F22) | PROVED | D1, Section 7 |
| P11 Consistency | every row residual tends to zero; local spatial order 2 at the centre, node 1, interior nodes and row $N-1$; order 1 at the four interface nodes; first order in time | solution four times continuously differentiable in $r$ on each closed layer, twice continuously differentiable in $t$ | PROVED | D1, Section 9.5 |
| P12 Release formula | (F26) evaluated on the exact solution equals the exact release $-4\pi\left(R_5\right)^2D_{\mathrm{OPyC}}\,\partial_rC(R_5,t)$ to $O\left(\left(\Delta r_{\mathrm{OPyC}}\right)^2\right)$ | solution smooth in the OPyC near $R_5$; no source at $R_5$ | PROVED | D1, Eq. (36a) |
| P13 Convergence, one material | $\max_j\left\lVert \mathbf{e}^j\right\rVert _\infty = O\left(\Delta t + \left(\Delta r\right)^2\right)$ | one material, uniform grid, smooth solution on $[0,T]$ | PROVED | D1, Eq. (46c) |
| P14 Convergence, five layers | $\max_j\left\lVert \mathbf{e}^j\right\rVert _V = O\left(\left(\Delta r\right)^2 + \Delta t\right)$ on $[t_*, T]$ | solution four times continuously differentiable in $r$ on each closed layer and twice continuously differentiable in $t$ on $[t_*,T]$; $\left\lVert \mathbf{e}(t_*)\right\rVert _V = O\left(\left(\Delta r\right)^2\right)$ | PROOF OUTLINE | D1, Section 9.5 |
| P15 Five-layer, maximum norm | $\max_j\left\lVert \mathbf{e}^j\right\rVert _\infty = O\left(\Delta t + \Delta r\right)$ on $[t_*, T]$ | (H); solution four times continuously differentiable in $r$ on each closed layer and twice continuously differentiable in $t$ on $[t_*,T]$; $\left\lVert \mathbf{e}(t_*)\right\rVert _\infty = O(\Delta r)$ | PROVED (pessimistic) | D1, Section 9.5 |
| P16 Discontinuous start | second order in space for $F^j$ and the profile at fixed $t > 0$; first order in time | (F21), (F22) | OBSERVED; NOT PROVEN | D1, Sections 7 and 9.5 |
| P17 Computed release rate | second order in space | as P12 | OBSERVED; NOT PROVEN | D1, Section 6 |
| P18 Conservation in floating point | $\left\lvert \hat\varepsilon^J\right\rvert$ is bounded by (F44) a posteriori and by (F45) a priori | IEEE double, rounding to nearest, no overflow, no subnormal products or quotients, no fused multiply-add, positive computed pivots; (F24) or the textbook elimination order | PROVED (the constant of compensated summation is quoted, and $s_J = J$ covers it). The residual is not confined to a fixed range: by (F44) and (F45) it scales with the number of steps $J$ times $1 + 2\Lambda_{\max}$, about $J$ times four times the largest mesh Fourier number. Observed: relative residuals $6.9\times10^{-12}$ to $1.19\times10^{-9}$ on the synthetic case with $\Delta t = 200\ \mathrm{s}$ and 1000 steps (20 to 320 intervals per layer), at most $9.2\times10^{-3}$ of (F44) at any step; up to $2.3\times10^{-9}$ in the Rust measurement at very large mesh Fourier numbers recorded by coordinator decision 36; and $1.1\times10^{-8}$ to $6.7\times10^{-8}$ in a synthetic scratch case with largest $\mathrm{Fo}$ from $2.8\times10^6$ to $1.1\times10^7$ over 3000 steps, at most $1.5\times10^{-2}$ of (F44) at any step (D5, Section 5.7) | D5, Section 5 |

Nothing in this table claims exact conservation in floating point, positivity for a solver other than (F24), or any accuracy for experimental data.

---

## 9. Where the approved additional physics enters (directive Section 17)

The placeholders of the first version of this section are now derived (D5) and stated in Section 11. The map from each earlier placeholder to its equation:

| Placeholder | Now | Derivation |
|---|---|---|
| G1 source term in the right-hand side (F20) | (F38) with (F32) to (F34) | D5, Sections 1.2, 1.3, 1.5 |
| G1 weighting at interface nodes and at the centre | (F33), (F34) | D5, Section 1.3 |
| G1 release when the source is non-zero at $R_5$ | (F39) | D5, Section 1.4 |
| G2 decay term in the diagonal | (F35), (F37) | D5, Section 2.1 |
| G1 and G2 amounts in the balance | (F40), (F41) | D5, Section 4.2 |
| G3 time level of $T$ and reassembly | (F36), (F37) | D5, Sections 3.1, 3.2 |

With every source zero, $\lambda = 0$ and constant temperature, Section 11 reduces to Sections 3 to 7 bit for bit (P19, P29 and the $\lambda = 0$ limit of P25).

---

## 10. Decisions fixed by this formulation

| Item | Fixed choice | Alternatives rejected, and why | Where |
|---|---|---|---|
| Node placement | a node at $r = 0$, at each of $R_1, \dots, R_4$, and at $R_5$ | nodes off the interfaces with series conductances: the retired finite-volume layout, interface value not an unknown | D1, Sections 5.6, 8.1 |
| Grid | piecewise uniform, $n_{\mathrm{material}}$ intervals per material | uniform grid: needs a common divisor of all thicknesses; graded grids: benefit not established | D1, Section 8.1 |
| Interior and interface rows | flux balance (F10) with shell conductances (F6) and trapezoidal weights (F7) | Lagrange three-point formulas and two-sided ghost nodes on unequal spacing: not conservative, sign condition; one-sided algebraic rows: not tridiagonal, not an M-matrix, not conservative | D1, Sections 5.6, 8.2 |
| Centre row | (F12), factor 6 | finite-volume centre cell: changes row 1; extrapolation: drops the owner's derivation | D1, Section 4.3 |
| Outer boundary | (F18), absorbing | the owner's insulated row: a different physical surface | D1, Section 6 |
| Kernel-surface start value | (F22) | $\theta = \frac{1}{2}, 0, 1$: first order on unequal spacing | D1, Section 7 |
| Release | (F26) at level $j+1$ | any other formula breaks (F29) | D1, Section 6 |
| Normalisation of $F$ | (F28) with the discrete $M^0$, which equals the exact inventory by P10 | none needed | D1, Section 7 |
| Row scaling | rows divided by $V_i$, (F11) | rows multiplied by $V_i$: equal in exact arithmetic; no rounding advantage was shown (D1, Section 9.4) | reconciliation |
| Linear solver | Thomas algorithm (F24) | marching for $C_{i+1}$: unstable | D1, Section 9.3 |
| Interface flux diagnostic | (F31) | raw one-sided flows: only first-order consistent | D1, Section 5.4 |

---

## 11. Generation, decay and temperature-dependent diffusivity (directive Section 17)

This section states the three approved mechanisms and their combination, with nothing left to choose. The derivations, proofs and scratch checks are in D5. The model is Eq. (1) of D5: $\partial C/\partial t = \left(r\right)^{-2}\partial_r\left[\left(r\right)^2D(r, T)\,\partial_rC\right] + G - \lambda C$, with the boundary and interface conditions of Section 1.

**Assumptions.** The source is constant in space inside each material, $G = G_{\mathrm{material}}(t) \ge 0$ (in $\mathrm{mol\ m^{-3}\ s^{-1}}$); for a direct fission product $G_{\mathrm{kernel}} = y\,\dot F/N_{\mathrm{A}}$ and the coating sources are zero (D5, Eq. (2); $y$ the yield, dimensionless; $\dot F$ the fission-rate density in $\mathrm{m^{-3}\ s^{-1}}$; $N_{\mathrm{A}} = 6.02214076\times10^{23}\ \mathrm{mol^{-1}}$). The decay constant $\lambda \ge 0$ (in $\mathrm{s^{-1}}$) is the same everywhere. The temperature $T(t)$ (in $\mathrm{K}$) is uniform in the particle. Ag-110m, Cs-134 and Eu-154 have $G = 0$ (decision 15).

### 11.1 Source (G1)

**Step average** of each material's source, for the step from $t^j$ to $t^{j+1}$, evaluated exactly for piecewise-constant or piecewise-linear histories:

$$
\bar G_{\mathrm{material}}^j = \frac{1}{\Delta t}\int_{t^j}^{t^{j+1}}G_{\mathrm{material}}(t)\,dt. \tag{F32}
$$

**Source weights** (in $\mathrm{mol\ s^{-1}}$, the generation rate of the dual region of node $i$ divided by $4\pi$). $\bar G_{i,\mathrm{inner}}^j$ and $\bar G_{i,\mathrm{outer}}^j$ are the values (F32) of the materials on the inner and outer side of node $i$, assigned as in (F4), and $\omega = 1 - 1/\left(3\,n_{\mathrm{kernel}}\right)$ (dimensionless):

$$
\begin{aligned}
\bar Q_i^j &= \frac{\left(r_i\right)^2}{2}\left[\Delta r_{i,\mathrm{inner}}\,\bar G_{i,\mathrm{inner}}^j + \Delta r_{i,\mathrm{outer}}\,\bar G_{i,\mathrm{outer}}^j\right], \qquad 1 \le i \le N-1,\ i \ne N_{\mathrm{kernel}},\\
\bar Q_{N_{\mathrm{kernel}}}^j &= \frac{\left(R_1\right)^2}{2}\left[\Delta r_{\mathrm{kernel}}\left(\omega\,\bar G_{\mathrm{kernel}}^j + \left(1 - \omega\right)\bar G_{\mathrm{buffer}}^j\right) + \Delta r_{\mathrm{buffer}}\,\bar G_{\mathrm{buffer}}^j\right],\\
\bar Q_N^j &= \frac{\left(R_5\right)^2}{2}\,\Delta r_{N-1,\mathrm{outer}}\,\bar G_{\mathrm{OPyC}}^j.
\end{aligned} \tag{F33}
$$

The factor $\omega$ gives the kernel part of the source the inventory-matched shape of (F21) and (F22): for a kernel-only source $\bar Q_{N_{\mathrm{kernel}}}^j = \theta\,V_{N_{\mathrm{kernel}}}\bar G_{\mathrm{kernel}}^j$, and $\sum_{i=1}^{N}\bar Q_i^j = \frac{1}{3}\left(R_1\right)^3\bar G_{\mathrm{kernel}}^j$ exactly (D5, Section 1.3). Node 0 has no weight.

**Nodal source densities** (in $\mathrm{mol\ m^{-3}\ s^{-1}}$):

$$
g_0^j = \bar G_{\mathrm{kernel}}^j, \qquad g_i^j = \frac{\bar Q_i^j}{V_i} \quad \left(1 \le i \le N-1\right). \tag{F34}
$$

### 11.2 Decay (G2) and temperature (G3)

**Decay number** of a step and its place in the matrix:

$$
\kappa = \lambda\,\Delta t \quad \text{(dimensionless)}, \qquad \text{added to every diagonal entry } B_0, \dots, B_{N-1}. \tag{F35}
$$

The implementation stores $\kappa$ once per step length and uses the same stored number in the diagonal and in (F40).

**Diffusivity laws and the step value.** $D_{\mathrm{material}}(T) = D_{0,\mathrm{material}}\exp\left(-E_{a,\mathrm{material}}/\left(R_{\mathrm{gas}}T\right)\right)$, or the sum of two such terms where the register documents a row so (decision 13); $D_0$ in $\mathrm{m^2\ s^{-1}}$, $E_a$ in $\mathrm{J\ mol^{-1}}$, $T$ in $\mathrm{K}$ ($T = T_{\mathrm{C}} + 273.15$), $R_{\mathrm{gas}}$ in $\mathrm{J\ mol^{-1}\ K^{-1}}$, the value documented with the row where the register gives one, otherwise $R_{\mathrm{gas}} = N_{\mathrm{A}}k_{\mathrm{B}} = 8.31446261815324\ \mathrm{J\ mol^{-1}\ K^{-1}}$, the exact product of the SI values $N_{\mathrm{A}} = 6.02214076\times10^{23}\ \mathrm{mol^{-1}}$ and $k_{\mathrm{B}} = 1.380649\times10^{-23}\ \mathrm{J\ K^{-1}}$. A temperature outside the documented range of a correlation is reported. The step uses the value at the middle of the step:

$$
\bar D_{\mathrm{material}}^j = D_{\mathrm{material}}\left(T\left(t^j + \tfrac{1}{2}\Delta t\right)\right) \quad \left(\mathrm{m^2\ s^{-1}}\right). \tag{F36}
$$

$K_{i,\mathrm{outer}}^j$ and $\mathrm{Fo}_{\mathrm{material}}^j$ are (F6) and (F8) with $\bar D^j$. The matrix and its Thomas factors are reassembled from (F37) whenever $\bar D^j$ or $\Delta t$ differs from the previous step. $V_i$, (F21), (F22) and (F33) do not depend on $D$.

### 11.3 The combined rows (G4)

For the step from $t^j$ to $t^{j+1}$:

$$
\begin{aligned}
&L_i^j = -\frac{\Delta t\,K_{i,\mathrm{inner}}^j}{V_i}, \qquad U_i^j = -\frac{\Delta t\,K_{i,\mathrm{outer}}^j}{V_i}, \qquad B_i^j = 1 + \left|L_i^j\right| + \left|U_i^j\right| + \kappa, \qquad 1 \le i \le N-1,\\
&B_0^j = 1 + 6\,\mathrm{Fo}_{\mathrm{kernel}}^j + \kappa, \qquad U_0^j = -6\,\mathrm{Fo}_{\mathrm{kernel}}^j, \qquad L_1^j = 0,
\end{aligned} \tag{F37}
$$

with $U_{N-1}^j$ in $B_{N-1}^j$ but not in the matrix, as in (F17). The right-hand side is

$$
b_i^j = C_i^j + \Delta t\,g_i^j, \qquad 0 \le i \le N-1, \tag{F38}
$$

and the system $\mathbf{A}^j\mathbf{C}^{j+1} = \mathbf{b}^j$ is solved by (F24) with the coefficients of step $j$. $C_N^{j+1} = 0$ (F18). With $G = 0$, $\lambda = 0$ and constant $T$, (F37) and (F38) are (F11), (F12), (F17) and (F20).

### 11.4 Release, generated and decayed amounts, balance

$$
\Phi^{j+1} = 4\pi K_{N-1,\mathrm{outer}}^j\,C_{N-1}^{j+1} + 4\pi\bar Q_N^j \quad \left(\mathrm{mol\ s^{-1}}\right), \tag{F39}
$$

with the stored $K_{N-1,\mathrm{outer}}^j$ of row $N-1$ of the same step. The second term is zero for the approved kernel-only source; when the OPyC source is not zero, it restores second-order consistency of the release (D5, Section 1.4). $M_{\mathrm{rel}}^j$ is (F27) with this $\Phi$.

$$
M_{\mathrm{gen}}^0 = 0, \quad M_{\mathrm{gen}}^{j+1} = M_{\mathrm{gen}}^j + 4\pi\,\Delta t\sum_{i=1}^{N}\bar Q_i^j; \qquad M_{\mathrm{dec}}^0 = 0, \quad M_{\mathrm{dec}}^{j+1} = M_{\mathrm{dec}}^j + \kappa\,M^{j+1} \qquad \left(\mathrm{mol}\right). \tag{F40}
$$

The sum for $M_{\mathrm{gen}}$ includes node $N$; $M^{j+1}$ is (F25). $M_{\mathrm{dec}}$ is never added to $M_{\mathrm{rel}}$.

$$
M^0 + M_{\mathrm{gen}}^j = M^j + M_{\mathrm{rel}}^j + M_{\mathrm{dec}}^j \ \ \text{(exact arithmetic)}, \qquad \varepsilon^j = M^j + M_{\mathrm{rel}}^j + M_{\mathrm{dec}}^j - M^0 - M_{\mathrm{gen}}^j. \tag{F41}
$$

(F41) replaces (F29) and (F30); it reduces to them when $G = 0$ and $\lambda = 0$. The residual is reported relative to $M^0 + M_{\mathrm{gen}}^j$.

### 11.5 Release fractions

$$
F^j = \frac{M_{\mathrm{rel}}^j}{M^0}\ \ \left(C_{\mathrm{init}} > 0\right), \qquad F_{\mathrm{gen}}^j = \frac{M_{\mathrm{rel}}^j}{M^0 + M_{\mathrm{gen}}^j}\ \ \left(M^0 + M_{\mathrm{gen}}^j > 0\right), \qquad \beta^{j+1} = \frac{\Phi^{j+1}}{4\pi\sum_{i=1}^{N}\bar Q_i^j}\ \ \left(\text{denominator} > 0\right). \tag{F42}
$$

All three are dimensionless. $F$ is (F28); with a source it can exceed $1$. $0 \le F_{\mathrm{gen}}^j \le 1$ always (P31); it equals $F^j$ when $G = 0$. $\beta$ is the ratio of release rate to generation rate. **Proposed rule for $C_{\mathrm{init}} = 0$ (decision 8, for the coordinator to confirm):** such a run is accepted when some $\bar G^j > 0$; $F_{\mathrm{gen}}^j$ is then reported from the first step with $M_{\mathrm{gen}}^j > 0$ and is absent before it; $F^j$ is not reported. A run with $C_{\mathrm{init}} = 0$ and no source remains rejected. Which normalisation is compared with a measured fraction is decided by the experimental comparison.

### 11.6 Properties of the extended scheme

Standing hypotheses (H) of Section 8, plus: $G \ge 0$, $\lambda \ge 0$, every $\bar D_{\mathrm{material}}^j > 0$ and finite.

| Property | Statement | Hypotheses | Status | Source |
|---|---|---|---|---|
| P19 Zero-source limit | $G = 0$ reproduces the baseline bit for bit | (F38) coded as $C + \Delta t\,g$ | PROVED | D5, Section 1.6 |
| P20 Generated amount | $M_{\mathrm{gen}}^j$ is the exact time integral of the discrete source; for a kernel-only source $M_{\mathrm{gen}}^j = \frac{4\pi}{3}\left(R_1\right)^3\int_0^{t^j}G_{\mathrm{kernel}}\,dt$ | (F32), (F33) | PROVED | D5, Sections 1.3, 1.5 |
| P21 No-diffusion limit of the source | the kernel field keeps the shape (F21) | kernel-only source | PROVED | D5, Section 1.3 |
| P22 Steady state, kernel-only source | the discrete steady state equals the exact steady profile at every node, for any grid and any diffusivities; the steady release equals $\frac{4\pi}{3}\left(R_1\right)^3G_{\mathrm{kernel}}$ | constant $G_{\mathrm{kernel}}$, $\lambda = 0$, constant $D$ | PROVED (observed in 50-digit arithmetic: nodal error $\le 3.6\times10^{-46}$ of $C(0)$, release error $\le 4.1\times10^{-46}$) | D5, Section 1.6 |
| P22a Steady state, uniform source, one material | exact on a grid with one spacing; on unequal spacings a local first-order row residual at the nodes where the spacing changes; global second order on unequal spacings | equal $G$ and equal $D$ in all materials | exact on one spacing: PROVED; local first-order residual: PROVED; global second order on unequal spacings: OBSERVED (orders $2.00$), NOT PROVEN | D5, Section 1.6 |
| P23 Release with an OPyC source | (F39) is second-order consistent; (F26) alone is first order | smooth solution near $R_5$ | PROVED; OBSERVED orders $2.00$ and $1.00$ | D5, Section 1.4 |
| P24 Decay matrix | non-singular M-matrix; margin $1 + \kappa$; $\left\lVert \left(\mathbf{A}^j\right)^{-1}\right\rVert _\infty \le 1/\left(1 + \kappa\right)$; pivots $m_i \ge 1 + \kappa + \left\lvert U_i^j\right\rvert$; P6 unchanged | (H) | PROVED | D5, Section 2.1 |
| P25 Decay relation | without source, $\mathbf{C}_\lambda^j = \left(1 + \kappa\right)^{-j}\mathbf{C}_{\mathrm{base}}^j$ computed with step $\Delta t/\left(1 + \kappa\right)$; $\lambda = 0$ gives the baseline bit for bit | constant $D$ | PROVED | D5, Section 2.3 |
| P26 Stability with $T(t)$ | the maximum norm and the weighted norm of P3 are non-increasing (no source, no decay) for every history $T(t)$ and every $\Delta t$ | (H) per step | PROVED | D5, Section 3.4 |
| P27 Time accuracy with $T(t)$ | first order with (F36); the change of $D$ within a step adds only second-order terms to the local residual; end-of-step values would add a first-order term proportional to $\mathrm{d}D/\mathrm{d}t$ | each $D_{\mathrm{material}}(T)$ twice continuously differentiable in $T$, and $T(t)$ twice continuously differentiable on each step | PROVED for a fixed grid; PROOF OUTLINE with grid-independent constants; OBSERVED | D5, Section 3.2 |
| P28 Homogeneous sphere with $T(t)$ | the scheme is constant-coefficient Backward Euler in $\tau = \int D\,dt$ with steps $\Delta t\,\bar D^j$; the exact solution is $C_{\mathrm{unit}}\left(r, \tau(t)\right)$, with $C_{\mathrm{unit}}$ the solution for unit diffusivity | equal $D$ in all materials at every time | PROVED | D5, Section 3.3 |
| P29 Constant temperature | identical to the constant-$D$ run, bit for bit | same double-precision $D(T)$ and the same assembly code | PROVED | D5, Section 3.5 |
| P30 Combined balance | (F41) holds exactly in exact arithmetic | the P9 hypotheses per step; the same stored $\kappa$ in the diagonal and (F40); the same $\bar Q$ in (F38), (F39) and (F40) | PROVED | D5, Section 4.2 |
| P31 Combined solution | $\mathbf{A}^j$ is a non-singular M-matrix for every $j$; $\mathbf{C}^j \ge \mathbf{0}$ (also in floating point with (F24)); $\left\lVert \mathbf{C}^{j+1}\right\rVert _\infty \le \left(\left\lVert \mathbf{C}^j\right\rVert _\infty + \Delta t\max_ig_i^j\right)/\left(1 + \kappa\right)$, hence $\left\lVert \mathbf{C}^j\right\rVert _\infty \le \max\left(C_{\mathrm{init}}, g_*/\lambda\right)$ for $g_i^j \le g_*$ and $\lambda > 0$; the same bound in the weighted norm; $M_{\mathrm{rel}}$ non-decreasing; $0 \le F_{\mathrm{gen}}^j \le 1$ | (H), $G \ge 0$ | PROVED | D5, Sections 4.3, 4.4 |
| Convergence with $G$, $\lambda$, $T(t)$ | second order in space, first order in time for release fraction, inventory and decayed amount | synthetic cases | OBSERVED; NOT PROVEN beyond P13 to P15 | D5, Sections 1.6, 2.3, 3.2 |

### 11.7 Rounding bounds of the balance residual

**Rounding bounds (D5, Section 5).** The tolerance that a test applies to $\left|\varepsilon^j\right|$ is the rigorous bound below, derived for IEEE double precision with rounding to nearest ($u = 2^{-53}$, dimensionless), no overflow and no subnormal products or quotients, no fused multiply-add, and positive computed Thomas pivots (checked by the code). Hats denote computed numbers, $\gamma_\nu = \nu u/\left(1 - \nu u\right)$, $\hat p = 4\,\mathrm{fl}(\pi)$, and, for the computed vector $\hat{\mathbf{C}}^j \ge \mathbf{0}$ and the stored matrix $\hat{\mathbf{A}}$,

$$
S^j = \sum_{i=1}^{N-1}V_i\left(\left|\hat{\mathbf{A}}\right|\hat{\mathbf{C}}^j\right)_i \quad \left(\mathrm{mol}\right), \qquad c_{\mathbf{A}} = \frac{\gamma_{10}}{1 - \gamma_4}, \qquad \eta = 4\left(3N + J + 10\right)u. \tag{F43}
$$

A posteriori, for every $J \ge 1$ (with $s_J = J$, which covers recursive and compensated summation of the release increments):

$$
\left|\hat\varepsilon^J\right| \le \left(1 + \eta\right)\left[\hat p\,c_{\mathbf{A}}\sum_{k=1}^{J}S^k + \gamma_N\left(\hat M^J + \hat M^0\right) + \left(\gamma_3 + \left(s_J + 1\right)u\right)\hat M_{\mathrm{rel}}^J + u\,\hat M^J\right]. \tag{F44}
$$

A priori, with $\Lambda_{\max} = \max_{1 \le i \le N-1}\left(\left|L_i\right| + \left|U_i\right|\right)$ (dimensionless; $U_{N-1}$ included) and $\kappa_R = c_{\mathbf{A}}\left(1 + \gamma_4\right)\left(1 + 2\Lambda_{\max}\right) < 1$:

$$
\frac{\left|\hat\varepsilon^J\right|}{\hat M^0} \le \left(1 + \eta\right)\left[\left(1 - \kappa_R\right)^{-J} - 1 + \left(2N + 6 + s_J\right)u\left(1 - \kappa_R\right)^{-J}\right] \approx 10\,u\,J\left(1 + 2\Lambda_{\max}\right). \tag{F45}
$$

$\Lambda_{\max}$ is about $2\max\mathrm{Fo}$. The per-step rounding is proportional to the gross exchange $\Delta t\,K\left(C_i + C_{i+1}\right)$ of every interval, not to the net flow, which is why the residual grows with $\mathrm{Fo}$ (D5, Section 5.4). (F44) is the test tolerance at every step of a baseline run (coordinator decision 6); (F45) is the bound known before a run.

**Combined scheme.** For a run with a source, decay or $T(t)$ (Section 11), the test tolerance at every step is (F46), derived in D5, Section 5.8 (Eq. (41) there), under the same hypotheses and for the operation orders stated there: the diagonal $\mathrm{fl}\left(\mathrm{fl}\left(\mathrm{fl}\left(1 - L_i\right) - U_i\right) + \kappa\right)$; the right-hand side $\mathrm{fl}\left(C_i^j + \mathrm{fl}\left(\Delta t\,g_i^j\right)\right)$; the release increment $\mathrm{fl}\left(\Delta t\,\mathrm{fl}\left(\mathrm{fl}\left(\mathrm{fl}\left(\hat p\,K_{N-1,\mathrm{outer}}^j\right)C_{N-1}^{j+1}\right) + \mathrm{fl}\left(\hat p\,\bar Q_N^j\right)\right)\right)$; the generation increment $\mathrm{fl}\left(\Delta t\,\mathrm{fl}\left(\hat p\,\mathrm{fl}\left(\sum_{i=1}^{N}\bar Q_i^j\right)\right)\right)$; the decay increment $\mathrm{fl}\left(\kappa\,\hat M^{j+1}\right)$; and $\hat\varepsilon^J = \mathrm{fl}\left(\mathrm{fl}\left(\mathrm{fl}\left(\hat M^J + \hat M_{\mathrm{rel}}^J\right) - \mathrm{fl}\left(\hat M^0 + \hat M_{\mathrm{gen}}^J\right)\right) + \hat M_{\mathrm{dec}}^J\right)$. $S^k$ is (F43) with the stored matrix $\hat{\mathbf{A}}^{k-1}$ of the step that produced $\hat{\mathbf{C}}^k$, and $\chi^k = 1$ if some $g_i^k > 0$ in step $k$, otherwise $0$ (dimensionless). With the same $\eta$ as in (F43) and $s_J = J$:

$$
\begin{aligned}
\left|\hat\varepsilon^J\right| \le \left(1 + \eta\right)\Big[\,&\hat p\,\frac{\gamma_{11}}{1 - \gamma_5}\sum_{k=1}^{J}S^k + u\sum_{k=0}^{J-1}\chi^k\hat M^k + \gamma_N\left(\hat M^J + \hat M^0\right) + \left(\gamma_4 + s_Ju\right)\hat M_{\mathrm{rel}}^J\\
&+ \left(\gamma_{N+1} + \gamma_3 + s_Ju\right)\hat M_{\mathrm{gen}}^J + \left(\gamma_{N+1} + s_Ju\right)\hat M_{\mathrm{dec}}^J + \gamma_2\left(\hat M^J + \hat M_{\mathrm{rel}}^J + \hat M^0 + \hat M_{\mathrm{gen}}^J\right)\Big] \quad \left(\mathrm{mol}\right).
\end{aligned} \tag{F46}
$$

The terms are, in order: the backward error of the solves; the rounding of the right-hand side at steps with a source; the two inventories; the release increments and their accumulator; the generation increments, the source part of the right-hand side and their accumulator; the decay increments and their accumulator; the evaluation of $\hat\varepsilon^J$. The computed value of the right side of (F46) is itself a rigorous bound (D5, Section 5.8). The residual is reported relative to $M^0 + M_{\mathrm{gen}}^j$. With $G = 0$ and $\lambda = 0$, (F46) is valid but weaker than (F44), and (F44) is used.

### 11.8 Decisions fixed by Section 11

| Item | Fixed choice | Alternatives rejected, and why | Where |
|---|---|---|---|
| Time level of the source | step average (F32) | value at $t^{j+1}$: same order, but the generated amount would not be the exact integral of the history | D5, Section 1.5 |
| Source weight at $N_{\mathrm{kernel}}$ | $\omega$-split (F33) | plain half-interval weight: second order, but the generated amount, the no-diffusion limit and the steady state are then not exact | D5, Sections 1.3, 1.6 |
| Source at the centre | pointwise $g_0 = \bar G_{\mathrm{kernel}}$ in row 0; no inventory weight | a weight at node 0: contradicts $V_0 = 0$ (F7) | D5, Section 1.3 |
| Source next to $R_5$ | released at once, added to $\Phi$ and $M_{\mathrm{gen}}$ (F39), (F40) | dropping it: first-order release when the OPyC source is not zero | D5, Section 1.4 |
| Decay | implicit, $\kappa$ on the diagonal (F35), booked separately (F40) | explicit decay: loses the M-matrix margin argument and positivity for large $\lambda\,\Delta t$; exponential factor per step: not the approved Backward Euler form | D5, Section 2 |
| Time level of $D$ | middle of the step (F36) | end of the step: an additional first-order coefficient term $\propto \mathrm{d}D/\mathrm{d}t$ in the local residual (the reason for rejecting it); its total error was $0.45$ to about $22$ times that of (F36) in the synthetic ramps of D5, Section 3.2, so the size comparison is case-dependent; exact step average: same order, not elementary for a temperature ramp | D5, Section 3.2 |
| Normalisation with a source | $F_{\mathrm{gen}}$ and $\beta$ (F42) beside $F$ | $F$ alone: exceeds 1 with a source, undefined for $C_{\mathrm{init}} = 0$ | D5, Section 4.4 |
| Balance tolerance | (F44) at every step of a baseline run, (F46) at every step with source, decay or $T(t)$, (F45) a priori | a constant chosen from observed residuals: excluded by decision 6 | D5, Section 5 |
