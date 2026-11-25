# STRATO: Final Five-Layer BTCS Formulation

**Status.** This is Document 2 of directive Section 9: the complete five-layer BTCS system, with nothing left to choose. It states the grid, every matrix row, the right-hand side, the initial vector, the solution of the linear system, the discrete inventory, the release rate, the cumulative release, the balance identity and its residual, and each proven property with its hypotheses. It marks the places where generation, decay and temperature-dependent diffusivity (directive Section 17) will enter, without deriving them.

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
- No source, no decay, constant temperature. Section 9 marks where these enter later.

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

The unknowns of the step from $t^j$ to $t^{j+1}$ are $C_0^{j+1}, C_1^{j+1}, \dots, C_{N-1}^{j+1}$: $N$ unknowns, $N$ rows. $C_N^{j+1}$ is not an unknown (F15). Row $i$ reads

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

(Section 9 marks where a source term will be added.)

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

The tolerance that a test applies to $\left|\varepsilon^j\right|/M^0$ is not fixed here. It must come from a rounding analysis of the Rust implementation, in which the largest $\mathrm{Fo}$ and the number of steps enter (D1, Section 9.4).

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
| P8b Computed interface flux | (F31) evaluated on the computed solution converges to the exact interface flow at second order in space | as P8a | OBSERVED (orders $1.98$ to $2.00$, self-convergence, synthetic case); NOT PROVEN: no order is proved, because the proved nodal error bounds, divided by $\Delta r$, give none | D1, Section 5.4 |
| P9 Conservation | (F29) holds exactly in exact arithmetic | (H); the same stored $K$ in rows $i$, $i+1$ and in (F26), in particular the stored $K_{N-1,\mathrm{outer}}$ of (F6) with (F5) in row $N-1$ and in (F26); $V_i$ the weight by which row $i$ was divided; $V_0 = 0$ with $K_{0,\mathrm{outer}} = 0$; release at level $j+1$ | PROVED | D1, Eqs. (21), (35), (37) |
| P10 Exact initial inventory | $M^0 = \frac{4\pi}{3}\left(R_1\right)^3C_{\mathrm{init}}$ | (F21), (F22) | PROVED | D1, Section 7 |
| P11 Consistency | every row residual tends to zero; local spatial order 2 at the centre, node 1, interior nodes and row $N-1$; order 1 at the four interface nodes; first order in time | solution four times continuously differentiable in $r$ on each closed layer, twice continuously differentiable in $t$ | PROVED | D1, Section 9.5 |
| P12 Release formula | (F26) evaluated on the exact solution equals the exact release $-4\pi\left(R_5\right)^2D_{\mathrm{OPyC}}\,\partial_rC(R_5,t)$ to $O\left(\left(\Delta r_{\mathrm{OPyC}}\right)^2\right)$ | solution smooth in the OPyC near $R_5$; no source at $R_5$ | PROVED | D1, Eq. (36a) |
| P13 Convergence, one material | $\max_j\left\lVert \mathbf{e}^j\right\rVert _\infty = O\left(\Delta t + \left(\Delta r\right)^2\right)$ | one material, uniform grid, smooth solution on $[0,T]$ | PROVED | D1, Eq. (46c) |
| P14 Convergence, five layers | $\max_j\left\lVert \mathbf{e}^j\right\rVert _V = O\left(\left(\Delta r\right)^2 + \Delta t\right)$ on $[t_*, T]$ | solution four times continuously differentiable in $r$ on each closed layer and twice continuously differentiable in $t$ on $[t_*,T]$; $\left\lVert \mathbf{e}(t_*)\right\rVert _V = O\left(\left(\Delta r\right)^2\right)$ | PROOF OUTLINE | D1, Section 9.5 |
| P15 Five-layer, maximum norm | $\max_j\left\lVert \mathbf{e}^j\right\rVert _\infty = O\left(\Delta t + \Delta r\right)$ on $[t_*, T]$ | (H); solution four times continuously differentiable in $r$ on each closed layer and twice continuously differentiable in $t$ on $[t_*,T]$; $\left\lVert \mathbf{e}(t_*)\right\rVert _\infty = O(\Delta r)$ | PROVED (pessimistic) | D1, Section 9.5 |
| P16 Discontinuous start | second order in space for $F^j$ and the profile at fixed $t > 0$; first order in time | (F21), (F22) | OBSERVED; NOT PROVEN | D1, Sections 7 and 9.5 |
| P17 Computed release rate | second order in space | as P12 | OBSERVED; NOT PROVEN | D1, Section 6 |
| P18 Conservation in floating point | relative residual $\left\lvert \varepsilon^j\right\rvert /M^0$ of $10^{-13}$ to $10^{-10}$ in scratch runs | none | OBSERVED | D1, Section 9.4 |

Nothing in this table claims exact conservation in floating point, positivity for a solver other than (F24), or any accuracy for experimental data.

---

## 9. Placeholders for the approved additional physics (directive Section 17)

The terms below are **not derived** here. Each needs its own derivation, review and verification (directive Phase G) before it is implemented. This section only marks where each one enters the formulation above, so that the pseudocode and the Rust structure leave room for it. Until then each placeholder is zero, and the formulation above is complete without it.

**G1, generation.** A volumetric source $G$ (in $\mathrm{mol\ m^{-3}\ s^{-1}}$) enters

- the right-hand side (F20): $b_i^j = C_i^j + \left[\text{G1 source term}\right]$;
- the balance (F29) and residual (F30), through a generated amount $M_{\mathrm{gen}}^j$ (in $\mathrm{mol}$), so that the identity becomes $M^0 + M_{\mathrm{gen}}^j = M^j + M_{\mathrm{rel}}^j + \left[\text{G2 decayed amount}\right]$;
- possibly the release (F26): derivation B found that a source that is non-zero at $R_5$ breaks the cancellation behind P12, so the G1 derivation must decide the release formula in that case (D1, Section 11, item 5).

The G1 derivation must also fix the time level of the source, its weighting at interface nodes where the source differs between materials, and at the centre node, where $V_0 = 0$.

**G2, radioactive decay.** A decay constant $\lambda$ (in $\mathrm{s^{-1}}$) enters

- the diagonal (F11), (F12), (F17): $B_i = 1 + \left|L_i\right| + \left|U_i\right| + \left[\text{G2 decay term}\right]$;
- the balance (F29), through a decayed amount $M_{\mathrm{dec}}^j$ (in $\mathrm{mol}$), booked separately from the physical release $M_{\mathrm{rel}}^j$.

**G3, temperature-dependent diffusivity.** The diffusivities in (F4) become $D_{\mathrm{material}}(T)$ (in $\mathrm{m^2\ s^{-1}}$). Every quantity that depends on them, (F6), (F8), (F11), (F12), (F14), (F16) and (F26), is evaluated with the same values, and $\mathbf{A}$ is reassembled whenever $T$ changes (Section 5.8). The G3 derivation must fix the time level at which $T$ is evaluated in a step.

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
