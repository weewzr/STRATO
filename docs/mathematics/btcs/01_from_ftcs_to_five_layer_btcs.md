# STRATO: From Original FTCS Derivation to Five-Layer BTCS

**Status.** This is Document 1 of directive Section 9: the full derivation of the five-layer BTCS scheme (directive items A1 to A7). Two agents derived the scheme independently: derivation A from the owner's original documents, and derivation B from first principles. The two were compared equation by equation, and every difference was resolved by algebra or by a scratch experiment. The record of that comparison is [reconciliation.md](reconciliation.md). This document is derivation A with the results of derivation B and of the reconciliation integrated. Derivation B is kept unchanged as a record in [archive/independent_derivation_b.md](archive/independent_derivation_b.md).

It is a mathematical derivation. The numbers quoted from scratch experiments are numerical checks of the algebra. They are not verification of a production solver, and they are not experimental validation.

**Companion documents.**

- [00_original_documents.md](00_original_documents.md) is the account and algebra check of the owner's two original documents. It is cited as "document 00". Its equation labels (H1) to (H27), (T1) to (T30) and (C1) to (C4) are used here unchanged.
- [02_final_formulation.md](02_final_formulation.md) is Document 2: the final system, row by row, with nothing left to choose. Its equations are numbered (F1), (F2), and so on. Where this document and Document 2 differ in presentation, Document 2 is the normative statement for the pseudocode and the Rust code.

**Classification of statements.** Each property in Section 9 carries one of these labels:

- **PROVED**: a complete proof is given here, under the stated hypotheses.
- **PROOF OUTLINE**: every step of the argument is given and was checked in the reconciliation, but the constants are not tracked.
- **OBSERVED**: shown only by scratch computations.
- **NOT PROVEN**: stated as open.

---

## 0. Scope, physical model and notation

### 0.1 The physical model

The model is the one STRATO already fixed in its v1.0.0 derivation (`docs/archive/finite-volume/v1.0.0/STRATO_Full_Mathematical_Derivation.md`, Part III). Only the numerical method changes.

- **Five layers.** One fission-product species diffuses radially through five concentric layers: kernel, buffer, IPyC, SiC and OPyC. The diffusivity is constant in each layer.
- **Perfect contact.** At each of the four internal interfaces the concentration is continuous, which means a partition coefficient of one.
- **Absorbing outer boundary.** The outer surface is absorbing: $C(R_5, t) = 0$.
- **Kernel-only start.** At $t = 0$ the species is uniformly distributed in the kernel, and the coatings are clean.
- **No source or decay.** Generation, decay and temperature-dependent diffusivity are directive Phase G. They are mentioned here only where they change a matrix coefficient.

### 0.2 Notation

Directive Section 5 is binding:

- $i$ is the radial node index and $j$ the time-step index. Neither is used for anything else.
- Materials are named, never numbered by $i$.
- A power encloses the whole variable in parentheses, for example $\left(r_i\right)^2$. A time superscript such as the $j$ in $C_i^j$ is an index, not a power.

| Symbol | Meaning | Unit |
|---|---|---|
| $r$ | radial coordinate | $\mathrm{m}$ |
| $t$ | time | $\mathrm{s}$ |
| $C(r,t)$ | concentration of the diffusing species (the owner's $c$) | $\mathrm{mol\ m^{-3}}$ |
| $J(r,t)$ | radial diffusive flux, outward positive | $\mathrm{mol\ m^{-2}\ s^{-1}}$ |
| $\Phi(r,t) = 4\pi\left(r\right)^2 J(r,t)$ | outward flow through the sphere of radius $r$ | $\mathrm{mol\ s^{-1}}$ |
| $D(r)$ | diffusivity at radius $r$ | $\mathrm{m^2\ s^{-1}}$ |
| $D_{\mathrm{kernel}}, D_{\mathrm{buffer}}, D_{\mathrm{IPyC}}, D_{\mathrm{SiC}}, D_{\mathrm{OPyC}}$ | diffusivity of each material, constant within the material | $\mathrm{m^2\ s^{-1}}$ |
| $R_1, R_2, R_3, R_4, R_5$ | outer radii of kernel, buffer, IPyC, SiC and OPyC; $R_0 = 0$ | $\mathrm{m}$ |
| $C_{\mathrm{init}}$ | initial concentration in the kernel | $\mathrm{mol\ m^{-3}}$ |
| $i = 0, 1, \dots, N$ | global radial node index; node $0$ at the centre, node $N$ at $R_5$ | dimensionless |
| $r_i$ | radius of node $i$, with $0 = r_0 < r_1 < \dots < r_N = R_5$ | $\mathrm{m}$ |
| $N_{\mathrm{kernel}}, N_{\mathrm{buffer}}, N_{\mathrm{IPyC}}, N_{\mathrm{SiC}}, N_{\mathrm{OPyC}} = N$ | index of the node on $R_1, R_2, R_3, R_4, R_5$ | dimensionless |
| $\Delta r_{i,\mathrm{outer}} = r_{i+1} - r_i$ | spacing on the outer side of node $i$: the length of the interval between nodes $i$ and $i+1$ | $\mathrm{m}$ |
| $\Delta r_{i,\mathrm{inner}} = r_i - r_{i-1}$ | spacing on the inner side of node $i$; $\Delta r_{i,\mathrm{inner}} = \Delta r_{i-1,\mathrm{outer}}$ | $\mathrm{m}$ |
| $\Delta r$ | uniform spacing, where the grid is uniform | $\mathrm{m}$ |
| $\Delta r_{\mathrm{kernel}}, \dots, \Delta r_{\mathrm{OPyC}}$ | uniform spacing inside each layer (Section 8) | $\mathrm{m}$ |
| $D_{i,\mathrm{outer}}$ | diffusivity of the material filling the interval $(r_i, r_{i+1})$ | $\mathrm{m^2\ s^{-1}}$ |
| $D_{i,\mathrm{inner}}$ | diffusivity of the material filling $(r_{i-1}, r_i)$; $D_{i,\mathrm{inner}} = D_{i-1,\mathrm{outer}}$ | $\mathrm{m^2\ s^{-1}}$ |
| $j = 0, 1, 2, \dots$ | time-step index; $t^j = j\,\Delta t$ | dimensionless |
| $\Delta t$ | time step | $\mathrm{s}$ |
| $C_i^j$ | concentration at node $i$ and time step $j$ | $\mathrm{mol\ m^{-3}}$ |
| $\mathrm{Fo} = D\,\Delta t/\left(\Delta r\right)^2$ | mesh Fourier number; $\mathrm{Fo}_{\mathrm{kernel}} = D_{\mathrm{kernel}}\Delta t/\left(\Delta r_{\mathrm{kernel}}\right)^2$, and likewise for the other layers | dimensionless |
| $K_{i,\mathrm{outer}}$ | conductance of the interval $(r_i, r_{i+1})$ divided by $4\pi$, Eq. (19); $K_{i,\mathrm{inner}} = K_{i-1,\mathrm{outer}}$ | $\mathrm{m^3\ s^{-1}}$ |
| $V_i$ | nodal storage weight divided by $4\pi$, Eqs. (29) and (43) | $\mathrm{m^3}$ |
| $L_i, B_i, U_i$ | lower, diagonal and upper coefficients of row $i$ of the tridiagonal matrix | dimensionless |
| $M^j$ | discrete inventory at time step $j$ | $\mathrm{mol}$ |
| $\Phi^{j}$ | discrete release rate through $R_5$ at time step $j$ | $\mathrm{mol\ s^{-1}}$ |
| $M_{\mathrm{rel}}^j$ | cumulative release up to time step $j$ | $\mathrm{mol}$ |
| $F^j$ | release fraction | dimensionless |
| $\theta$ | initial value at the kernel-surface node as a fraction of $C_{\mathrm{init}}$ | dimensionless |
| $\varepsilon^j = M^j + M_{\mathrm{rel}}^j - M^0$ | balance residual at time step $j$ | $\mathrm{mol}$ |

**A clash in the directive's notation.** Directive item A5 writes the initial kernel concentration as $C_0$. With the binding notation $C_i^j$, $C_0$ is also the concentration at the centre node $i = 0$. This document therefore writes the initial kernel concentration as $C_{\mathrm{init}}$.

**Inner and outer subscripts instead of half indices.** Directive Section 5 asks for full `inner` and `outer` subscripts where a geometric boundary must be named. The interval between nodes $i$ and $i+1$ is therefore named from node $i$ as its outer interval, and from node $i+1$ as its inner interval: $\Delta r_{i,\mathrm{outer}} = \Delta r_{i+1,\mathrm{inner}}$, $D_{i,\mathrm{outer}} = D_{i+1,\mathrm{inner}}$ and $K_{i,\mathrm{outer}} = K_{i+1,\mathrm{inner}}$. No half-integer node index and no finite-volume face is used.

### 0.3 Map of the transitions

| Step | Owner's original work (document 00) | Five-layer BTCS (this document) | Section |
|---|---|---|---|
| Starting law | Fick's second law $\partial c/\partial t = D\nabla^2 c$ (H1), one constant $D$ | Conservation of mass plus Fick's first law; $D$ stays inside the derivative | 1 |
| Equation solved | $D\left(\partial^2 C/\partial r^2 + \frac{2}{r}\partial C/\partial r\right)$ (H5) | The same, inside each layer only | 2 |
| Time discretisation | Forward Euler (FTCS), explicit | Backward Euler (BTCS), implicit | 3.3 |
| Interior stencil | (H9) to (H10), (T11) | The same stencil at the new time level; conservative flux form (H20) to Eq. (18) | 3.4, 3.5 |
| Centre | Ghost node, $3D\,\partial^2 C/\partial r^2$, factor 6 (T20) | The same, implicit; one-way coupling analysed | 4 |
| Interfaces | Not treated; flux continuity mentioned (T Section 5.5) | A node on each interface, half-interval flux balance | 5 |
| Outer boundary | Insulated, factor 2 (T23); factor 6 in (H17) is wrong | Absorbing $C_N^j = 0$; release from the last interval flow | 6 |
| Initial state | "$\partial c/\partial t = 0$" (H11), (T13) | Kernel-only step profile; inventory-matched value at $R_1$ | 7 |
| Grid | Uniform, five nodes, $r_i = i\,\Delta r$ | Piecewise uniform, interfaces on nodes; non-uniform coefficients | 8 |
| Solution | Explicit update node by node | Tridiagonal system per step, Thomas algorithm | 9, 10 |

---

## 1. From conservation of mass and Fick's law to the spherical diffusion equation

The owner starts from Fick's second law (H1), which already assumes one constant $D$. In a TRISO particle $D$ jumps by orders of magnitude at four radii. The derivation must therefore begin one step earlier, with conservation of mass and Fick's first law, and must keep $D$ inside the derivative.

**Step 1: the balance on a thin shell.** Consider the shell between radii $r$ and $r + \delta r$, with $0 < \delta r$. The amount of species in the shell is

$$
\int_r^{r+\delta r} C(s,t)\,4\pi\left(s\right)^2\,ds \qquad (\mathrm{mol}).
$$

With no source and no sink inside the shell, this amount changes only by what flows through its two surfaces. The outward flow through the inner surface, $\Phi(r,t)$, enters the shell; the outward flow through the outer surface, $\Phi(r+\delta r,t)$, leaves it:

$$
\frac{d}{dt}\int_r^{r+\delta r} C(s,t)\,4\pi\left(s\right)^2\,ds = \Phi(r,t) - \Phi(r+\delta r,t). \tag{1}
$$

**Step 2: divide by the thickness.** The shell does not move, so the time derivative can be taken inside the integral. The mean-value theorem for integrals gives a radius $\xi$ in $[r, r+\delta r]$ with

$$
\int_r^{r+\delta r} \frac{\partial C}{\partial t}(s,t)\,4\pi\left(s\right)^2\,ds = \frac{\partial C}{\partial t}(\xi,t)\,4\pi\left(\xi\right)^2\,\delta r.
$$

Dividing Eq. (1) by $\delta r$:

$$
\frac{\partial C}{\partial t}(\xi,t)\,4\pi\left(\xi\right)^2 = -\frac{\Phi(r+\delta r,t) - \Phi(r,t)}{\delta r}.
$$

**Step 3: shrink the shell.** As $\delta r \to 0$, $\xi \to r$. The right side becomes the definition of the derivative of $\Phi$ with respect to $r$:

$$
4\pi\left(r\right)^2\,\frac{\partial C}{\partial t} = -\frac{\partial \Phi}{\partial r} = -\frac{\partial}{\partial r}\left[4\pi\left(r\right)^2 J\right]. \tag{2}
$$

**Step 4: Fick's first law.** The flux runs down the concentration gradient:

$$
J = -D(r)\,\frac{\partial C}{\partial r}. \tag{3}
$$

**Step 5: combine and divide by $4\pi\left(r\right)^2$** (allowed for $r > 0$):

$$
\boxed{\frac{\partial C}{\partial t} = \frac{1}{\left(r\right)^2}\frac{\partial}{\partial r}\left[\left(r\right)^2 D(r)\,\frac{\partial C}{\partial r}\right]} \tag{4}
$$

Eq. (4) is the directive's spherical diffusion equation. When the temperature depends on time (Phase G3), $D(r)$ becomes $D(r,T)$ without any other change.

**Relation to the owner's route.** If $D$ is one constant, $D$ can be taken outside the derivative in Eq. (4). The result is $D$ times the radial part of the Laplacian (H2), which is exactly the owner's (H3). The owner's route (3D Laplacian, then Assumption 1) and this one (shell balance) agree wherever $D$ is constant. Only this route stays valid across a jump in $D$.

---

## 2. The constant-diffusivity form inside one layer

**Step 1: take $D$ out of the derivative.** Inside one material, for example the buffer ($R_1 < r < R_2$), $D(r) = D_{\mathrm{buffer}}$ is a constant:

$$
\frac{\partial C}{\partial t} = \frac{D_{\mathrm{buffer}}}{\left(r\right)^2}\frac{\partial}{\partial r}\left[\left(r\right)^2\frac{\partial C}{\partial r}\right].
$$

**Step 2: apply the product rule,** as the owner did in (H4). The derivative of $\left(r\right)^2$ is $2r$:

$$
\frac{\partial}{\partial r}\left[\left(r\right)^2\frac{\partial C}{\partial r}\right] = 2r\,\frac{\partial C}{\partial r} + \left(r\right)^2\frac{\partial^2 C}{\partial r^2}.
$$

**Step 3: divide by $\left(r\right)^2$:**

$$
\boxed{\frac{\partial C}{\partial t} = D\left[\frac{\partial^2 C}{\partial r^2} + \frac{2}{r}\frac{\partial C}{\partial r}\right]}, \qquad D \in \left\{D_{\mathrm{kernel}}, D_{\mathrm{buffer}}, D_{\mathrm{IPyC}}, D_{\mathrm{SiC}}, D_{\mathrm{OPyC}}\right\}. \tag{5}
$$

Eq. (5) is the owner's (H5), now restricted to the open intervals $0 < r < R_1$, $R_1 < r < R_2$, $R_2 < r < R_3$, $R_3 < r < R_4$ and $R_4 < r < R_5$.

**Why Eq. (5) cannot be used at an interface.** At $r = R_1$ the flux $-D\,\partial C/\partial r$ is continuous (Section 5.2), but $D$ jumps. The slope $\partial C/\partial r$ therefore jumps, and $\partial^2 C/\partial r^2$ does not exist there. Suppose a node $i$ lies on $R_1$ with spacing $\Delta r$ on both sides. Expanding $C_{i\pm1}$ about $r_i$ separately on each side gives

$$
\frac{C_{i+1} - 2C_i + C_{i-1}}{\left(\Delta r\right)^2} = \frac{1}{\Delta r}\left(\left.\frac{\partial C}{\partial r}\right|_{R_1^+} - \left.\frac{\partial C}{\partial r}\right|_{R_1^-}\right) + O(1) = \frac{1}{\Delta r}\left(\frac{D_{\mathrm{kernel}}}{D_{\mathrm{buffer}}} - 1\right)\left.\frac{\partial C}{\partial r}\right|_{R_1^-} + O(1).
$$

The second equality uses flux continuity. Whichever single value of $D$ multiplies it, the constant-$D$ stencil at an interface node reports a rate of change that grows like $1/\Delta r$ when the true rate is finite. It is not consistent. This is why the directive forbids applying it across a jump, and why Section 5 builds the interface row from fluxes instead.

---

## 3. A1: interior nodes, central differences and Backward Euler

### 3.1 Grid

The nodes are $0 = r_0 < r_1 < \dots < r_N = R_5$. Each interface radius $R_1, \dots, R_4$ is a node, so every interval $(r_i, r_{i+1})$ lies entirely inside one material (Section 8 explains the choice). In this section the grid is uniform with spacing $\Delta r$ around node $i$ ($r_{i\pm1} = r_i \pm \Delta r$), and node $i$ lies strictly inside one layer with constant $D$. Node $i$ is an *interior node* of that layer.

### 3.2 Central differences in space, from Taylor's theorem

Expand $C$ about $r_i$ at a fixed time, writing $C_i$ for $C(r_i,t)$ and primes for derivatives with respect to $r$:

$$
C_{i\pm1} = C_i \pm \Delta r\,C_i' + \frac{\left(\Delta r\right)^2}{2}C_i'' \pm \frac{\left(\Delta r\right)^3}{6}C_i''' + \frac{\left(\Delta r\right)^4}{24}C_i'''' + O\left(\left(\Delta r\right)^5\right). \tag{6}
$$

Adding the two expansions cancels the odd terms; subtracting them cancels the even terms:

$$
C_{i+1} + C_{i-1} = 2C_i + \left(\Delta r\right)^2 C_i'' + \frac{\left(\Delta r\right)^4}{12}C_i'''' + O\left(\left(\Delta r\right)^6\right), \tag{7}
$$

$$
C_{i+1} - C_{i-1} = 2\,\Delta r\,C_i' + \frac{\left(\Delta r\right)^3}{3}C_i''' + O\left(\left(\Delta r\right)^5\right). \tag{8}
$$

Solving for the derivatives gives the owner's stencils (H6), with their errors:

$$
\frac{\partial^2 C}{\partial r^2}(r_i) = \frac{C_{i+1} - 2C_i + C_{i-1}}{\left(\Delta r\right)^2} - \frac{\left(\Delta r\right)^2}{12}C_i'''' + O\left(\left(\Delta r\right)^4\right), \tag{9}
$$

$$
\frac{\partial C}{\partial r}(r_i) = \frac{C_{i+1} - C_{i-1}}{2\,\Delta r} - \frac{\left(\Delta r\right)^2}{6}C_i''' + O\left(\left(\Delta r\right)^4\right). \tag{10}
$$

Both stencils are second-order accurate.

### 3.3 Backward Euler in time, from Taylor's theorem

The owner used forward Euler, (H7), which evaluates the right-hand side at the old time $t^j$. Backward Euler evaluates it at the new time $t^{j+1}$, as in the owner's (H27) with $x \to C$, $h \to \Delta t$ and $n \to j$. Expanding about $t^{j+1}$ (dots denote time derivatives):

$$
C(r_i, t^j) = C(r_i, t^{j+1}) - \Delta t\,\dot C(r_i, t^{j+1}) + \frac{\left(\Delta t\right)^2}{2}\ddot C(r_i, t^{j+1}) + O\left(\left(\Delta t\right)^3\right),
$$

$$
\frac{\partial C}{\partial t}(r_i, t^{j+1}) = \frac{C_i^{j+1} - C_i^j}{\Delta t} + \frac{\Delta t}{2}\ddot C(r_i, t^{j+1}) + O\left(\left(\Delta t\right)^2\right). \tag{11}
$$

The quotient $(C_i^{j+1} - C_i^j)/\Delta t$ is a first-order approximation of $\partial C/\partial t$ at $t^{j+1}$. It is the same quotient as in forward Euler. The only difference is the time level at which the spatial terms are taken.

### 3.4 The BTCS interior equation and its tridiagonal coefficients

**Step 1: evaluate Eq. (5) at node $i$ and time $t^{j+1}$, and insert Eqs. (9) to (11) without their error terms:**

$$
\frac{C_i^{j+1} - C_i^j}{\Delta t} = D\left[\frac{C_{i+1}^{j+1} - 2C_i^{j+1} + C_{i-1}^{j+1}}{\left(\Delta r\right)^2} + \frac{2}{r_i}\,\frac{C_{i+1}^{j+1} - C_{i-1}^{j+1}}{2\,\Delta r}\right]. \tag{12}
$$

This is the owner's (H8) and (T9) with every value on the right moved from time step $j$ to time step $j+1$.

**Step 2: multiply by $\Delta t$.** The first term on the right gets $D\,\Delta t/\left(\Delta r\right)^2 = \mathrm{Fo}$. The second gets

$$
D\,\Delta t\cdot\frac{2}{r_i}\cdot\frac{1}{2\,\Delta r} = \frac{D\,\Delta t}{r_i\,\Delta r} = \frac{D\,\Delta t}{\left(\Delta r\right)^2}\cdot\frac{\Delta r}{r_i} = \mathrm{Fo}\,\frac{\Delta r}{r_i}.
$$

So

$$
C_i^{j+1} - C_i^j = \mathrm{Fo}\left(C_{i+1}^{j+1} - 2C_i^{j+1} + C_{i-1}^{j+1}\right) + \mathrm{Fo}\,\frac{\Delta r}{r_i}\left(C_{i+1}^{j+1} - C_{i-1}^{j+1}\right). \tag{13}
$$

**Step 3: collect the three unknowns on the left and leave the known old value on the right.** For each new-time value, add up its coefficients.

- $C_{i-1}^{j+1}$: $+\mathrm{Fo}$ from the second difference and $-\mathrm{Fo}\,\Delta r/r_i$ from the first difference. Moved to the left, the total is $-\mathrm{Fo}\left(1 - \frac{\Delta r}{r_i}\right)$.
- $C_i^{j+1}$: $1$ from the time difference and $-2\mathrm{Fo}$ on the right. Moved to the left, the total is $1 + 2\mathrm{Fo}$.
- $C_{i+1}^{j+1}$: $+\mathrm{Fo}$ and $+\mathrm{Fo}\,\Delta r/r_i$. Moved to the left, the total is $-\mathrm{Fo}\left(1 + \frac{\Delta r}{r_i}\right)$.

$$
\boxed{L_i\,C_{i-1}^{j+1} + B_i\,C_i^{j+1} + U_i\,C_{i+1}^{j+1} = C_i^j} \tag{14}
$$

$$
L_i = -\mathrm{Fo}\left(1 - \frac{\Delta r}{r_i}\right), \qquad B_i = 1 + 2\,\mathrm{Fo}, \qquad U_i = -\mathrm{Fo}\left(1 + \frac{\Delta r}{r_i}\right). \tag{15}
$$

**In the kernel**, $r_i = i\,\Delta r_{\mathrm{kernel}}$, so $\Delta r/r_i = 1/i$. Eq. (15) then takes the owner's form with the factor $1/i$:

$$
L_i = -\mathrm{Fo}_{\mathrm{kernel}}\left(1 - \frac{1}{i}\right), \qquad B_i = 1 + 2\,\mathrm{Fo}_{\mathrm{kernel}}, \qquad U_i = -\mathrm{Fo}_{\mathrm{kernel}}\left(1 + \frac{1}{i}\right), \qquad 1 \le i \le N_{\mathrm{kernel}} - 1. \tag{16}
$$

**In the coatings**, $r_i \ne i\,\Delta r$: for example $r_i = R_1 + (i - N_{\mathrm{kernel}})\,\Delta r_{\mathrm{buffer}}$ in the buffer. The factor $1/i$ is then wrong, and Eq. (15) with the actual $r_i$ must be used.

**FTCS and BTCS side by side.** The owner's explicit update (C1) and Eq. (14) carry the same three numbers with opposite roles:

$$
\text{FTCS:}\quad C_i^{j+1} = \mathrm{Fo}\left(1 - \tfrac{\Delta r}{r_i}\right)C_{i-1}^j + \left(1 - 2\mathrm{Fo}\right)C_i^j + \mathrm{Fo}\left(1 + \tfrac{\Delta r}{r_i}\right)C_{i+1}^j,
$$

$$
\text{BTCS:}\quad -\mathrm{Fo}\left(1 - \tfrac{\Delta r}{r_i}\right)C_{i-1}^{j+1} + \left(1 + 2\mathrm{Fo}\right)C_i^{j+1} - \mathrm{Fo}\left(1 + \tfrac{\Delta r}{r_i}\right)C_{i+1}^{j+1} = C_i^j.
$$

FTCS computes each new value directly. BTCS couples each new value to its two new neighbours, so all rows must be solved together (Section 9.3). In exchange, BTCS has no limit on $\mathrm{Fo}$ (Section 9). FTCS requires $\mathrm{Fo} \le \frac{1}{3}$ for stability and $\mathrm{Fo} \le \frac{1}{6}$ for non-negativity (document 00, Section 2).

**Truncation error.** Inserting the exact solution into Eq. (12) leaves, by Eqs. (9) to (11), the residual

$$
\tau_i^{j+1} = -\frac{\Delta t}{2}\ddot C - D\left(\Delta r\right)^2\left[\frac{C_i''''}{12} + \frac{C_i'''}{3\,r_i}\right] + O\left(\left(\Delta t\right)^2 + \left(\Delta r\right)^4\right). \tag{17}
$$

The scheme is therefore first order in time and second order in space at interior nodes. Near the centre, $C'''/r_i$ stays bounded, because $C'''$ is odd in $r$ and vanishes at $r = 0$.

**Node 1, the node next to the centre.** For an even field, $C''' = r\,C''''(0) + O\left(\left(r\right)^3\right)$ (Eq. (22) below), so at $r_1 = \Delta r_{\mathrm{kernel}}$ the bracket of Eq. (17) is $\frac{1}{12}C''''(0) + \frac{1}{3}C''''(0) = \frac{5}{12}C''''(0)$. A direct check: row 1 has $L_1 = 0$ (Section 4.3), so its spatial part is $2D_{\mathrm{kernel}}\left(C_2 - C_1\right)/\left(\Delta r_{\mathrm{kernel}}\right)^2$. Inserting Eq. (22) gives $3D_{\mathrm{kernel}}C''(0) + \frac{15}{12}D_{\mathrm{kernel}}\left(\Delta r_{\mathrm{kernel}}\right)^2C''''(0)$, while the exact operator at $r_1$ is $3D_{\mathrm{kernel}}C''(0) + \frac{10}{12}D_{\mathrm{kernel}}\left(\Delta r_{\mathrm{kernel}}\right)^2C''''(0)$. The residual of row 1 is therefore
$$
\tau_1^{j+1} = -\frac{\Delta t}{2}\ddot C - \frac{5}{12}D_{\mathrm{kernel}}\left(\Delta r_{\mathrm{kernel}}\right)^2C''''(0) + O\left(\left(\Delta t\right)^2 + \left(\Delta r_{\mathrm{kernel}}\right)^4\right), \tag{17a}
$$
second order, uniformly up to the centre (both derivations obtained this value independently).

### 3.5 The owner's regrouped form as a difference of flows (coordinator's hypothesis 1)

**Claim.** On a uniform grid inside one layer, multiplying the spatial operator of Eq. (12) by $\left(r_i\right)^2\Delta r$ gives a difference of two interval flows:

$$
\left(r_i\right)^2\Delta r\cdot D\left[\frac{C_{i+1} - 2C_i + C_{i-1}}{\left(\Delta r\right)^2} + \frac{2}{r_i}\frac{C_{i+1} - C_{i-1}}{2\,\Delta r}\right] = \frac{D\,r_i\,r_{i+1}}{\Delta r}\left(C_{i+1} - C_i\right) - \frac{D\,r_{i-1}\,r_i}{\Delta r}\left(C_i - C_{i-1}\right). \tag{18}
$$

**Proof.**

1. *Left side.* Multiply out:
   $$\frac{D}{\Delta r}\left[\left(r_i\right)^2\left(C_{i+1} - 2C_i + C_{i-1}\right) + r_i\,\Delta r\left(C_{i+1} - C_{i-1}\right)\right] = \frac{D}{\Delta r}\left[r_i\left(r_i + \Delta r\right)C_{i+1} - 2\left(r_i\right)^2 C_i + r_i\left(r_i - \Delta r\right)C_{i-1}\right].$$
   This is the owner's (H20) multiplied by $\left(r_i\right)^2\Delta r$.
2. *Right side.* Expand:
   $$\frac{D}{\Delta r}\left[r_i r_{i+1}C_{i+1} - \left(r_i r_{i+1} + r_{i-1}r_i\right)C_i + r_{i-1}r_i C_{i-1}\right].$$
3. *Compare.* On a uniform grid $r_{i\pm1} = r_i \pm \Delta r$. So $r_i r_{i+1} = r_i\left(r_i + \Delta r\right)$ and $r_{i-1}r_i = r_i\left(r_i - \Delta r\right)$, and the middle coefficient is $r_i\left(r_{i+1} + r_{i-1}\right) = r_i\cdot 2r_i = 2\left(r_i\right)^2$. All three coefficients agree. $\square$

A scratch check with random nodal values (12 intervals) gave agreement between the two sides to $1.4\times10^{-14}$, round-off.

**Physical meaning of the factor $r_i r_{i+1}$.** In a steady state with constant $D$, Eq. (4) reduces to $\frac{d}{dr}\left[\left(r\right)^2\frac{dC}{dr}\right] = 0$. Integrating twice gives $C = a + b/r$, with constants $a$ (in $\mathrm{mol\ m^{-3}}$) and $b$ (in $\mathrm{mol\ m^{-2}}$). The outward flow through any sphere in the shell $r_i \le r \le r_{i+1}$ is then

$$
\Phi = -4\pi\left(r\right)^2 D\,\frac{dC}{dr} = -4\pi\left(r\right)^2 D\left(-\frac{b}{\left(r\right)^2}\right) = 4\pi D\,b.
$$

The concentration difference across the shell is

$$
C_{i+1} - C_i = b\left(\frac{1}{r_{i+1}} - \frac{1}{r_i}\right) = -b\,\frac{r_{i+1} - r_i}{r_i\,r_{i+1}}.
$$

Eliminating $b$ gives the exact steady flow through the shell:

$$
\Phi = -4\pi\,\frac{D\,r_i\,r_{i+1}}{r_{i+1} - r_i}\left(C_{i+1} - C_i\right) = -4\pi K_{i,\mathrm{outer}}\left(C_{i+1} - C_i\right), \qquad K_{i,\mathrm{outer}} = \frac{D_{i,\mathrm{outer}}\,r_i\,r_{i+1}}{\Delta r_{i,\mathrm{outer}}}. \tag{19}
$$

So the face factor $r_i r_{i+1}$ in Eq. (18) is not an approximation of an area. It is the exact steady-state conductance of a spherical shell. This is why the owner's stencil reproduces $C = a + b/r$ exactly, and why Eq. (19) can be used unchanged on non-uniform grids and next to interfaces (Sections 5 and 8). $K_{i,\mathrm{outer}}$ has units $\mathrm{m^2\ s^{-1}}\times\mathrm{m}\times\mathrm{m}/\mathrm{m} = \mathrm{m^3\ s^{-1}}$.

**Discrete conservation.** With $V_i = \left(r_i\right)^2\Delta r$ on a uniform grid, Eqs. (12) and (18) give the BTCS step in balance form:

$$
V_i\,\frac{C_i^{j+1} - C_i^j}{\Delta t} = K_{i,\mathrm{outer}}\left(C_{i+1}^{j+1} - C_i^{j+1}\right) - K_{i,\mathrm{inner}}\left(C_i^{j+1} - C_{i-1}^{j+1}\right). \tag{20}
$$

The amount gained by node $i$ through its outer interval is exactly the amount lost by node $i+1$ through its inner interval, because both use the same $K_{i,\mathrm{outer}}$. Summing Eq. (20) over $i = 1, \dots, N-1$, every inner interval flow appears once with $+$ and once with $-$, and only the two end terms remain:

$$
\sum_{i=1}^{N-1}V_i\left(C_i^{j+1} - C_i^j\right) = \Delta t\left[K_{N-1,\mathrm{outer}}\left(C_N^{j+1} - C_{N-1}^{j+1}\right) - K_{0,\mathrm{outer}}\left(C_1^{j+1} - C_0^{j+1}\right)\right]. \tag{21}
$$

$K_{0,\mathrm{outer}} = D_{\mathrm{kernel}}\,r_0\,r_1/\Delta r_{0,\mathrm{outer}} = 0$ because $r_0 = 0$, so nothing flows through the centre. Eq. (21) holds for every $\Delta t > 0$, in exact arithmetic. Hypothesis (1) is therefore **proved**: the conserved quantity is $4\pi\sum_{i=1}^{N-1}V_i C_i^j$, the centre node has weight zero, and the only exchange with the outside is the flow through the last interval (Section 6).

**What the conserved inventory approximates.** The trapezoidal rule for $\int_0^{R_5}4\pi\left(r\right)^2 C\,dr$ on the nodes $r_0, \dots, r_N$ is

$$
\sum_{i=0}^{N-1}\frac{\Delta r_{i,\mathrm{outer}}}{2}\left[4\pi\left(r_i\right)^2 C_i + 4\pi\left(r_{i+1}\right)^2 C_{i+1}\right] = 4\pi\sum_{i=1}^{N-1}\left(r_i\right)^2\frac{\Delta r_{i,\mathrm{inner}} + \Delta r_{i,\mathrm{outer}}}{2}C_i + 2\pi\left[\Delta r_{0,\mathrm{outer}}\left(r_0\right)^2 C_0 + \Delta r_{N-1,\mathrm{outer}}\left(r_N\right)^2 C_N\right].
$$

The last bracket is zero, because $r_0 = 0$ and $C_N = 0$ (Section 6). The conserved sum is therefore exactly the trapezoidal rule: a second-order approximation of the true inventory for a smooth profile.

---

## 4. A2: the centre node

### 4.1 The limit $3\,\partial^2 C/\partial r^2$, checked with a Taylor series

The symmetry argument, given in full in v1.0.0 Part III, Section 3.1, shows that $\partial C/\partial r = 0$ at $r = 0$. It also shows that a smooth radially symmetric field is an even function of the signed distance through the centre. Its Taylor series about $r = 0$ therefore has only even powers:

$$
C(r,t) = C(0,t) + \frac{\left(r\right)^2}{2}C''(0,t) + \frac{\left(r\right)^4}{24}C''''(0,t) + O\left(\left(r\right)^6\right). \tag{22}
$$

Differentiating term by term:

$$
\frac{\partial C}{\partial r} = r\,C''(0) + \frac{\left(r\right)^3}{6}C''''(0) + \dots, \qquad \frac{2}{r}\frac{\partial C}{\partial r} = 2\,C''(0) + \frac{\left(r\right)^2}{3}C''''(0) + \dots, \qquad \frac{\partial^2 C}{\partial r^2} = C''(0) + \frac{\left(r\right)^2}{2}C''''(0) + \dots
$$

Adding the last two:

$$
\frac{\partial^2 C}{\partial r^2} + \frac{2}{r}\frac{\partial C}{\partial r} = 3\,C''(0,t) + \frac{5}{6}\left(r\right)^2 C''''(0,t) + \dots \xrightarrow{\ r\to0\ } 3\left.\frac{\partial^2 C}{\partial r^2}\right|_{r=0}. \tag{23}
$$

This confirms the owner's L'Hôpital result (H14) and (T18), and answers his question Q1 (document 00, Section 3). At the centre node the operator is $3D_{\mathrm{kernel}}\,\partial^2 C/\partial r^2$, evaluated with the kernel diffusivity.

### 4.2 Ghost node and the BTCS centre row

With the centre at $i = 0$ (the numbering of Document T), the ghost value $C_{-1}$ sits at $r = -\Delta r_{0,\mathrm{outer}}$. By the even symmetry of Eq. (22), $C_{-1} = C_1$, which is (T15). The central second difference collapses as in (T16):

$$
\left.\frac{\partial^2 C}{\partial r^2}\right|_{r=0} \approx \frac{C_1 - 2C_0 + C_{-1}}{\left(\Delta r_{0,\mathrm{outer}}\right)^2} = \frac{2\left(C_1 - C_0\right)}{\left(\Delta r_{0,\mathrm{outer}}\right)^2}. \tag{24}
$$

By Eq. (22), $2\left(C_1 - C_0\right)/\left(\Delta r_{0,\mathrm{outer}}\right)^2 = C''(0) + \frac{\left(\Delta r_{0,\mathrm{outer}}\right)^2}{12}C''''(0) + \dots$, so this is second order. Backward Euler with Eqs. (23) and (24), all at $t^{j+1}$:

$$
\frac{C_0^{j+1} - C_0^j}{\Delta t} = 3D_{\mathrm{kernel}}\cdot\frac{2\left(C_1^{j+1} - C_0^{j+1}\right)}{\left(\Delta r_{\mathrm{kernel}}\right)^2}.
$$

Multiplying by $\Delta t$ and collecting, as in Section 3.4:

$$
\boxed{\left(1 + 6\,\mathrm{Fo}_{\mathrm{kernel}}\right)C_0^{j+1} - 6\,\mathrm{Fo}_{\mathrm{kernel}}\,C_1^{j+1} = C_0^j}, \qquad B_0 = 1 + 6\,\mathrm{Fo}_{\mathrm{kernel}}, \quad U_0 = -6\,\mathrm{Fo}_{\mathrm{kernel}}. \tag{25}
$$

**Truncation error of the centre row.** Multiplying the expansion above by $3D_{\mathrm{kernel}}$:
$$
\frac{6D_{\mathrm{kernel}}\left(C_1 - C_0\right)}{\left(\Delta r_{\mathrm{kernel}}\right)^2} = 3D_{\mathrm{kernel}}C''(0) + \frac{D_{\mathrm{kernel}}\left(\Delta r_{\mathrm{kernel}}\right)^2}{4}C''''(0) + O\left(\left(\Delta r_{\mathrm{kernel}}\right)^4\right),
$$
so the residual of row 0 is $\tau_0^{j+1} = -\frac{\Delta t}{2}\ddot C(0) - \frac{1}{4}D_{\mathrm{kernel}}\left(\Delta r_{\mathrm{kernel}}\right)^2C''''(0) + O\left(\left(\Delta t\right)^2 + \left(\Delta r_{\mathrm{kernel}}\right)^4\right)$. **(PROVED** for a field that is smooth and even near the centre.)

**Check against the owner's treatment.** Eq. (25) is the implicit counterpart of (T20), $C_0^{j+1} = (1 - 6\mathrm{Fo})C_0^j + 6\mathrm{Fo}\,C_1^j$, with the same factor $6 = 3 \times 2$. In Document H the same row appears with inconsistent indices, (H15). The consistent forms are (C2) in document 00. Two of the owner's slips must not be carried over: the time difference across two spatial nodes, and the factor 3 at the outer surface (H17).

### 4.3 The centre value does not feed back (coordinator's hypothesis 2)

**Claim.** In the interior row of node 1, the coefficient of $C_0^{j+1}$ is zero.

**Proof.** By Eq. (16) with $i = 1$: $L_1 = -\mathrm{Fo}_{\mathrm{kernel}}\left(1 - \frac{1}{1}\right) = 0$. Equivalently, by Eq. (20), node 1 is coupled to node 0 through $K_{0,\mathrm{outer}} = D_{\mathrm{kernel}}\,r_0\,r_1/\Delta r_{0,\mathrm{outer}} = 0$. $\square$

**Consequences.**

1. **One-way coupling.** Rows $1, \dots, N-1$ contain only $C_1^{j+1}, \dots, C_{N-1}^{j+1}$ and form a closed system. $C_0^{j+1}$ is then fixed by Eq. (25) from $C_1^{j+1}$. Node 0 receives information from node 1 but sends none back. In the matrix this is the zero in position $(1,0)$. The matrix stays tridiagonal, and the Thomas algorithm (Section 9.3) needs no special case.
2. **Zero weight in the inventory.** Node 0 does not appear in the conserved sum (21); its trapezoidal weight would be $\left(r_0\right)^2\Delta r_{0,\mathrm{outer}}/2 = 0$. This is the trapezoidal rule's zero weight at $r = 0$, where the integrand $4\pi\left(r\right)^2 C$ vanishes.
3. **Conservation.** Eq. (25) is not part of the balance (21). It is a consistent diagnostic equation: $C_0^j$ converges to $C(0, t^j)$, at second order in space and first order in time, but no amount is stored in or released from node 0.
4. **Physical consistency.** The true flow through a sphere of radius $r$ is $4\pi\left(r\right)^2 J$, which tends to zero as $r \to 0$ (v1.0.0 Part III, Eq. (3.1)). A zero conductance between nodes 0 and 1 is the exact discrete statement of that condition.
5. **The ghost-node row and finite volumes.** The ghost-node row (25) is the same equation that a finite-volume balance gives on a small ball of radius $\Delta r/2$ around the centre. The ball's storage weight is $\frac{1}{3}\left(\frac{\Delta r}{2}\right)^3$ (volume divided by $4\pi$), its surface factor is $\left(\frac{\Delta r}{2}\right)^2$, and the flux is $D\,(C_1 - C_0)/\Delta r$:
   $$\frac{1}{3}\left(\frac{\Delta r}{2}\right)^3\frac{dC_0}{dt} = \left(\frac{\Delta r}{2}\right)^2 D\,\frac{C_1 - C_0}{\Delta r} \quad\Longrightarrow\quad \frac{dC_0}{dt} = \frac{6D\left(C_1 - C_0\right)}{\left(\Delta r\right)^2}.$$
   In that finite-volume picture, node 1 would lose what node 0 gains. In the owner's interior stencil it does not, because the factor there is $r_0 r_1 = 0$, not $\left(\Delta r/2\right)^2$. The pair (row 0, row 1) is therefore not mutually conservative. This is harmless only because node 0 has zero weight. **An implementation must not add a centre weight $\frac{1}{3}\left(\Delta r/2\right)^3$ to the inventory**: that would create an apparent imbalance of order $\Delta t\left(\Delta r/2\right)^2 D_{\mathrm{kernel}}\left|C_1 - C_0\right|/\Delta r$ per step.
6. **Alternatives considered.**
   - A fully finite-volume centre cell, with storage $\frac{1}{3}\left(\Delta r/2\right)^3$ and a face factor $\left(\Delta r/2\right)^2$ between nodes 0 and 1, is also conservative. It changes the owner's stencil at $i = 1$ and abandons the trapezoidal weights; it is the former finite-volume architecture that the directive retires.
   - Dropping row 0 and setting $C_0 = (4C_1 - C_2)/3$ (even quadratic extrapolation) is also second order, but it discards the owner's L'Hôpital derivation.

   **Decision:** keep Eq. (25).
7. **What a wrong centre row would damage (from derivation B).** Because of items 1 and 2, the rows $i \ge 1$, the inventory, the release and the balance identity (37) do not depend on row 0 at all. A wrong centre row therefore damages only the value $C_0^j$ itself and any diagnostic that includes node 0. Two wrong rows are instructive:
   - dropping the factor 3 of Eq. (23) gives $U_0 = -2\,\mathrm{Fo}_{\mathrm{kernel}}$ instead of $-6\,\mathrm{Fo}_{\mathrm{kernel}}$;
   - imposing $C_0 = C_1$ gives a first-order Neumann row.

   Row 0 relaxes $C_0$ towards $C_1$ on the time scale $\left(\Delta r_{\mathrm{kernel}}\right)^2/D_{\mathrm{kernel}}$. For a smooth solution, Eq. (25) gives $C_1 - C_0 \approx \left(\Delta r_{\mathrm{kernel}}\right)^2\,\partial_t C(0)/\left(6D_{\mathrm{kernel}}\right)$, correct to leading order. The row without the factor 3 gives three times that difference and $C_0 = C_1$ gives zero, so both wrong rows still leave $C_0$ within $O\left(\left(\Delta r_{\mathrm{kernel}}\right)^2\right)$ of the exact value. In derivation B's scratch runs on the homogeneous sphere (archive, Section 10.4), the error at $i \ge 1$ was identical to all printed digits for the three rows, and all three centre errors converged at second order. Eq. (25) is kept because it is the only one of the three that is consistent with Eq. (23); the observation shows only that a centre-row mistake cannot hide in the inventory or the release, so a test of $C_0$ itself is needed to detect it.
8. **Error bound at the centre.** Row 0 reads $C_0^{j+1} = \left(C_0^j + 6\,\mathrm{Fo}_{\mathrm{kernel}}C_1^{j+1}\right)/\left(1 + 6\,\mathrm{Fo}_{\mathrm{kernel}}\right)$, a convex combination. Its error therefore satisfies $\left|e_0^{j}\right| \le \max\left(\left|e_0^0\right|, \max_{k \le j}\left|e_1^k\right|\right) + t^j\max\left|\tau_0\right|$: the centre error follows the pointwise error at node 1.

---

## 5. A3: the four material interfaces

### 5.1 Continuity of concentration (assumption)

Under the approved perfect-contact assumption, with a partition coefficient of one (v1.0.0 Part III, Section 3.2.1):

$$
C(R_1^-, t) = C(R_1^+, t), \quad C(R_2^-, t) = C(R_2^+, t), \quad C(R_3^-, t) = C(R_3^+, t), \quad C(R_4^-, t) = C(R_4^+, t). \tag{26}
$$

This is a modelling assumption, not a consequence of conservation. In the scheme it is built in by placing a node on each interface, with one unknown shared by both materials.

### 5.2 Continuity of diffusive flux (derived)

Apply the balance (1) to the thin shell $R_1 - \delta < r < R_1 + \delta$, which straddles the kernel-buffer interface:

$$
\frac{d}{dt}\int_{R_1-\delta}^{R_1+\delta}C\,4\pi\left(r\right)^2\,dr = \Phi(R_1 - \delta, t) - \Phi(R_1 + \delta, t).
$$

If $\left|\partial C/\partial t\right| \le \mathcal{K}$ near the interface, for a finite bound $\mathcal{K}$ in $\mathrm{mol\ m^{-3}\ s^{-1}}$, the left side is at most $\mathcal{K}\cdot\frac{4\pi}{3}\left[\left(R_1 + \delta\right)^3 - \left(R_1 - \delta\right)^3\right] = \mathcal{K}\cdot\frac{4\pi}{3}\left[6\left(R_1\right)^2\delta + 2\left(\delta\right)^3\right]$, which tends to $0$ as $\delta \to 0$. Hence $\Phi(R_1^-, t) = \Phi(R_1^+, t)$. Dividing by $4\pi\left(R_1\right)^2 > 0$ and inserting Fick's law (3) on each side:

$$
D_{\mathrm{kernel}}\left.\frac{\partial C}{\partial r}\right|_{R_1^-} = D_{\mathrm{buffer}}\left.\frac{\partial C}{\partial r}\right|_{R_1^+}, \tag{27}
$$

and likewise $D_{\mathrm{buffer}}\,\partial_r C|_{R_2^-} = D_{\mathrm{IPyC}}\,\partial_r C|_{R_2^+}$, $D_{\mathrm{IPyC}}\,\partial_r C|_{R_3^-} = D_{\mathrm{SiC}}\,\partial_r C|_{R_3^+}$ and $D_{\mathrm{SiC}}\,\partial_r C|_{R_4^-} = D_{\mathrm{OPyC}}\,\partial_r C|_{R_4^+}$. The full version of this derivation is in v1.0.0 Part III, Section 3.2.2.

### 5.3 The interface row: a node on the interface, with a flux balance over its two half-intervals

Let node $i$ lie on an interface at radius $r_i = R$, with:

- the inner material, diffusivity $D_{\mathrm{in}}$, filling $(r_{i-1}, r_i)$, of length $\Delta r_- = \Delta r_{i,\mathrm{inner}}$;
- the outer material, diffusivity $D_{\mathrm{out}}$, filling $(r_i, r_{i+1})$, of length $\Delta r_+ = \Delta r_{i,\mathrm{outer}}$.

For the kernel-buffer interface, $D_{\mathrm{in}} = D_{\mathrm{kernel}}$, $D_{\mathrm{out}} = D_{\mathrm{buffer}}$, $\Delta r_- = \Delta r_{\mathrm{kernel}}$ and $\Delta r_+ = \Delta r_{\mathrm{buffer}}$.

The interface row is built from the two ingredients of Section 3.5, which do not need a single $D$ at the node.

**Flow through each adjacent interval.** Each interval lies inside one material, so the exact steady shell conductance (19) applies with that material's diffusivity, and no averaging of $D$ is needed:

$$
K_{i,\mathrm{inner}} = \frac{D_{\mathrm{in}}\,r_{i-1}\,r_i}{\Delta r_-}, \qquad K_{i,\mathrm{outer}} = \frac{D_{\mathrm{out}}\,r_i\,r_{i+1}}{\Delta r_+}. \tag{28}
$$

**Storage at the node.** By the trapezoidal rule (Section 3.5), node $i$ carries half of each adjacent interval:

$$
V_i = \left(r_i\right)^2\,\frac{\Delta r_- + \Delta r_+}{2}. \tag{29}
$$

This uses the inner material's half-interval on the left and the outer material's on the right, with the single nodal value $C_i$ shared by both. That shared value is the concentration continuity (26).

**The balance.** Storage change equals inflow from the inner interval minus outflow through the outer interval, at $t^{j+1}$:

$$
V_i\,\frac{C_i^{j+1} - C_i^j}{\Delta t} = K_{i,\mathrm{outer}}\left(C_{i+1}^{j+1} - C_i^{j+1}\right) - K_{i,\mathrm{inner}}\left(C_i^{j+1} - C_{i-1}^{j+1}\right). \tag{30}
$$

Eq. (30) has the same form as Eq. (20). The two differ only in where $V_i$ and $K$ come from. The interface row is therefore tridiagonal and fits into the telescoping sum (21) without change.

**Its coefficients.** Multiply by $\Delta t/V_i$ and collect as in Section 3.4:

$$
L_i = -\frac{\Delta t\,K_{i,\mathrm{inner}}}{V_i}, \qquad U_i = -\frac{\Delta t\,K_{i,\mathrm{outer}}}{V_i}, \qquad B_i = 1 - L_i - U_i. \tag{31}
$$

Inserting Eqs. (28) and (29), with $r_{i-1} = R - \Delta r_-$ and $r_{i+1} = R + \Delta r_+$:

$$
L_i = -\frac{2D_{\mathrm{in}}\Delta t\left(R - \Delta r_-\right)}{R\,\Delta r_-\left(\Delta r_- + \Delta r_+\right)} = -\mathrm{Fo}_{\mathrm{in}}\,\frac{2\,\Delta r_-}{\Delta r_- + \Delta r_+}\left(1 - \frac{\Delta r_-}{R}\right), \tag{32a}
$$

$$
U_i = -\frac{2D_{\mathrm{out}}\Delta t\left(R + \Delta r_+\right)}{R\,\Delta r_+\left(\Delta r_- + \Delta r_+\right)} = -\mathrm{Fo}_{\mathrm{out}}\,\frac{2\,\Delta r_+}{\Delta r_- + \Delta r_+}\left(1 + \frac{\Delta r_+}{R}\right), \tag{32b}
$$

where $\mathrm{Fo}_{\mathrm{in}} = D_{\mathrm{in}}\Delta t/\left(\Delta r_-\right)^2$ and $\mathrm{Fo}_{\mathrm{out}} = D_{\mathrm{out}}\Delta t/\left(\Delta r_+\right)^2$.

**Check.** With $D_{\mathrm{in}} = D_{\mathrm{out}}$ and $\Delta r_- = \Delta r_+$, the fraction $2\Delta r_\pm/(\Delta r_- + \Delta r_+)$ is $1$, and Eqs. (32a) and (32b) reduce to the interior coefficients (15). The interface row is the owner's row, generalised.

### 5.4 The discrete interface flux, seen from both sides

The discrete outward flows on the two sides of the interface node are

$$
\Phi_{i,\mathrm{inner}} = 4\pi K_{i,\mathrm{inner}}\left(C_{i-1} - C_i\right) \ \text{(arriving from the inner material)}, \qquad \Phi_{i,\mathrm{outer}} = 4\pi K_{i,\mathrm{outer}}\left(C_i - C_{i+1}\right) \ \text{(leaving into the outer material)}.
$$

By Eq. (30), $\Phi_{i,\mathrm{inner}}^{j+1} - \Phi_{i,\mathrm{outer}}^{j+1} = 4\pi V_i\,(C_i^{j+1} - C_i^j)/\Delta t$: they differ by exactly the storage term. As the grid is refined, $V_i \to 0$ while $\partial C/\partial t$ stays bounded, so the two flows become equal: the discrete form of Eq. (27). The difference is the amount stored in the two half-intervals during the step. It is not a loss, because Eq. (21) accounts for it.

**The flow at the interface radius itself (from derivation B).** Split the storage term between the two half-intervals, in the same proportions as $V_i$ in Eq. (29). The outward flow at $r = R$, reconstructed from the inner side and from the outer side, is

$$
\Phi_{R,\mathrm{inner}}^{j+1} = \Phi_{i,\mathrm{inner}}^{j+1} - 4\pi\left(R\right)^2\frac{\Delta r_-}{2}\,\frac{C_i^{j+1} - C_i^j}{\Delta t}, \qquad \Phi_{R,\mathrm{outer}}^{j+1} = \Phi_{i,\mathrm{outer}}^{j+1} + 4\pi\left(R\right)^2\frac{\Delta r_+}{2}\,\frac{C_i^{j+1} - C_i^j}{\Delta t}. \tag{30a}
$$

**Claim (PROVED).** $\Phi_{R,\mathrm{inner}}^{j+1} = \Phi_{R,\mathrm{outer}}^{j+1}$ exactly.

**Proof.** $\Phi_{R,\mathrm{inner}} - \Phi_{R,\mathrm{outer}} = \Phi_{i,\mathrm{inner}} - \Phi_{i,\mathrm{outer}} - 4\pi\left(R\right)^2\frac{\Delta r_- + \Delta r_+}{2}\,\frac{C_i^{j+1} - C_i^j}{\Delta t} = \Phi_{i,\mathrm{inner}} - \Phi_{i,\mathrm{outer}} - 4\pi V_i\,\frac{C_i^{j+1} - C_i^j}{\Delta t} = 0$, by the statement after Eq. (30). $\square$

**Accuracy.** Two statements must be kept apart, as for the release rate in Section 6: the consistency of formula (30a), which is proved, and the convergence of the computed interface flow, which is observed only.

*Consistency (PROVED, for a solution that is three times continuously differentiable in $r$ on each closed layer and twice continuously differentiable in $t$).* Insert the exact solution into Eq. (30a). Let $m_- = R - \Delta r_-/2$ be the midpoint of the inner interval. $\Phi_{i,\mathrm{inner}}$ approximates the exact outward flow at $m_-$ to second order: its conductance factor is $r_{i-1}r_i = \left(m_-\right)^2 - \frac{1}{4}\left(\Delta r_-\right)^2$, and the difference quotient is the derivative at $m_-$ up to $O\left(\left(\Delta r_-\right)^2\right)$. Integrating Eq. (2) from $m_-$ to $R$ gives $\Phi(R) = \Phi(m_-) - 4\pi\left(R\right)^2\frac{\Delta r_-}{2}\,\partial_t C(R) + O\left(\left(\Delta r_-\right)^2\right)$. The backward difference $\left(C(R, t^{j+1}) - C(R, t^j)\right)/\Delta t$ equals $\partial_t C(R, t^{j+1}) + O(\Delta t)$, and it is multiplied by $4\pi\left(R\right)^2\Delta r_-/2$. So Eq. (30a), evaluated on the exact solution, equals the exact interface flow $\Phi(R, t^{j+1})$ to $O\left(\left(\Delta r\right)^2 + \Delta r\,\Delta t\right)$, from either side. The raw one-sided values $\Phi_{i,\mathrm{inner}}$ and $\Phi_{i,\mathrm{outer}}$, evaluated on the exact solution, are only first-order approximations of it, because they omit the storage of a half-interval.

*Convergence of the computed interface flow (OBSERVED, NOT PROVEN).* Eq. (30a) contains $K_{i,\mathrm{inner}}\left(C_{i-1} - C_i\right)$ with $K_{i,\mathrm{inner}} \approx D_{\mathrm{in}}\left(R\right)^2/\Delta r_-$, and the storage term contains $\Delta r_-/\Delta t$ times a difference of computed values. A nodal error of order $\left(\Delta r\right)^2$ would therefore allow only first order in the computed flow. The nodal error bounds that are proved for five layers do not close this gap: the maximum-norm bound P15 of Document 2, $O\left(\Delta t + \Delta r\right)$, divided by $\Delta r$, gives no convergence at all, and the weighted $L^2$ bound of Section 9.5 is not pointwise. So no order of convergence of the computed interface flow is proved, not even first order. A reconciliation scratch run (synthetic case of Section 11, $m$ intervals in every layer, backward Euler extrapolated to $\Delta t \to 0$ by three-level Richardson extrapolation from $\Delta t = 100$, $50$ and $25\ \mathrm{s}$, flow divided by $M^0$, $t = 2\times10^5\ \mathrm{s}$) gave differences between successive grids $m = 10, 20, \dots, 320$ that fell by a factor of four at every refinement, at all four interfaces: observed orders $1.98$ to $2.00$ (at $R_1$: $1.997$, $1.999$, $2.000$, $1.999$). This is self-convergence; no independent reference for the interface flow was used.

*Equality of the two sides.* In derivation B's synthetic run (archive, Section 10.2) the raw values differed by up to 6% at $R_1$, and the two reconstructed values agreed to a relative $7.4\times10^{-11}$ or better, rounding.

**Decision:** a post-processor that reports the flux at an interface reports Eq. (30a). This is the quantity that directive Phase D, test E, can use, with the status above: consistent to second order (proved); successive-grid differences of the computed value fall at second order (observed only, self-convergence). Convergence of the computed value to the exact interface flow is not shown, because no exact five-layer interface flow was computed.

### 5.5 Consistency of the interface row

Expand $C_{i\pm1}$ about $r_i = R$ on each side separately. Write $C'_\pm$ and $C''_\pm$ for the one-sided derivatives at $R^\pm$:

$$
K_{i,\mathrm{outer}}\left(C_{i+1} - C_i\right) = D_{\mathrm{out}}\,R\left(R + \Delta r_+\right)\left[C'_+ + \frac{\Delta r_+}{2}C''_+ + O\left(\left(\Delta r_+\right)^2\right)\right],
$$

$$
K_{i,\mathrm{inner}}\left(C_i - C_{i-1}\right) = D_{\mathrm{in}}\,R\left(R - \Delta r_-\right)\left[C'_- - \frac{\Delta r_-}{2}C''_- + O\left(\left(\Delta r_-\right)^2\right)\right].
$$

Subtracting and ordering by powers of the spacing:

$$
\begin{aligned}
&\left(R\right)^2\left[D_{\mathrm{out}}C'_+ - D_{\mathrm{in}}C'_-\right] + \frac{\left(R\right)^2\Delta r_+}{2}D_{\mathrm{out}}\left[C''_+ + \frac{2}{R}C'_+\right] + \frac{\left(R\right)^2\Delta r_-}{2}D_{\mathrm{in}}\left[C''_- + \frac{2}{R}C'_-\right] + O\left(\left(\Delta r\right)^2\right).
\end{aligned}
$$

- The first bracket vanishes by flux continuity (27).
- By Eq. (5) on each side, $D_{\mathrm{out}}\left[C''_+ + \frac{2}{R}C'_+\right]$ and $D_{\mathrm{in}}\left[C''_- + \frac{2}{R}C'_-\right]$ are the one-sided limits of $\partial C/\partial t$ at $R$. They are equal, because $C$ is continuous at $R$ for all $t$ (Eq. (26)), and so is its time derivative.
- The difference of flows is therefore $\left(R\right)^2\frac{\Delta r_- + \Delta r_+}{2}\,\frac{\partial C}{\partial t} + O\left(\left(\Delta r\right)^2\right) = V_i\,\frac{\partial C}{\partial t} + O\left(\left(\Delta r\right)^2\right)$.

Dividing by $V_i$: the interface row approximates $\partial C/\partial t$ with a local truncation error of order $\Delta r$. The surviving $O\left(\left(\Delta r\right)^2\right)$ terms of the difference of flows contain $C'''_\pm$ and the second-order Taylor terms on each side; they do not cancel in general (both derivations reached this conclusion independently). A first-order local error confined to isolated nodes is the usual situation at interfaces. That it does not reduce the global order is shown in the weighted $L^2$ norm by the proof outline of Section 9.5, and observed: in scratch runs of the synthetic five-layer case (Section 11), the global error in the release fraction converged at order $1.99$ to $2.00$ (Section 7, with the note on the $m = 160$ entry).

### 5.6 Alternatives, and why the half-interval balance is chosen (coordinator's hypothesis 3)

**(a) Ghost nodes on both sides.** In the owner's ghost-node style, extend each material's solution one step past the interface:

- $G_{\mathrm{in}}$ at $R + \Delta r_-$, belonging to the inner material;
- $G_{\mathrm{out}}$ at $R - \Delta r_+$, belonging to the outer material.

Write the centred constant-$D$ stencil of Eq. (12) on each side, and require two things: both sides give the same $dC_i/dt$, and the centred fluxes $D_{\mathrm{in}}(G_{\mathrm{in}} - C_{i-1})/(2\Delta r_-)$ and $D_{\mathrm{out}}(C_{i+1} - G_{\mathrm{out}})/(2\Delta r_+)$ are equal. Call that common value $q$.

Write $F_+ = D_{\mathrm{out}}(C_{i+1} - C_i)/\Delta r_+$ and $F_- = D_{\mathrm{in}}(C_i - C_{i-1})/\Delta r_-$ for the one-sided gradient fluxes. Eliminating both ghosts gives $q = (\Delta r_- F_+ + \Delta r_+ F_-)/(\Delta r_- + \Delta r_+)$ and

$$
\frac{dC_i}{dt} = \frac{2\left(F_+ - F_-\right)}{\Delta r_- + \Delta r_+} + \frac{2}{R}\,\frac{\Delta r_-\,F_+ + \Delta r_+\,F_-}{\Delta r_- + \Delta r_+}. \tag{33a}
$$

The half-interval balance (30), divided by $V_i$, is

$$
\frac{dC_i}{dt} = \frac{2\left(F_+ - F_-\right)}{\Delta r_- + \Delta r_+} + \frac{2}{R}\,\frac{\Delta r_+\,F_+ + \Delta r_-\,F_-}{\Delta r_- + \Delta r_+}. \tag{33b}
$$

The two differ only in the weights of the curvature term:

- **Equal spacing.** For $\Delta r_- = \Delta r_+$ they are identical. The owner's ghost-node method and the flux balance give the same interface row.
- **Unequal spacing.** For $\Delta r_- \ne \Delta r_+$, (33a) is not a difference of interval flows, so it loses discrete conservation. Multiplied by $V_i$, its coefficient of $\left(C_{i+1} - C_i\right)$ is $D_{\mathrm{out}}\,R\left(R + \Delta r_-\right)/\Delta r_+$, while row $i+1$ uses $D_{\mathrm{out}}\,R\left(R + \Delta r_+\right)/\Delta r_+$ for the same interval, so the sum (21) does not telescope. Its local truncation error is also $O(\Delta r)$, like that of (33b). Its coefficient of $C_{i-1}$, $2D_{\mathrm{in}}\left(R - \Delta r_+\right)/\left(R\,\Delta r_-\left(\Delta r_- + \Delta r_+\right)\right)$, has the wrong sign when $\Delta r_+ > R$. Scratch runs:
  - balance residual (synthetic case, 40 intervals per layer, $\Delta t = 200\ \mathrm{s}$): the largest relative residual was $3.15\times10^{-4}$ with (33a) at the four interface nodes, against rounding level ($\sim10^{-11}$) with (33b). Derivation B obtained the same $3.15\times10^{-4}$ independently, with 200 steps, and $1.25\times10^{-3}$ at 20 intervals per layer;
  - accuracy (derivation B, archive Section 10.3): on the slowest five-layer eigenvalue both rows converge at second order, and the error of (33a) is about 2.4 times smaller than that of (33b). Accuracy on this measure does not separate the two rows; conservation and the sign pattern do.

**(b) One-sided second-order differences.** Replace the interface row by the algebraic flux-matching condition

$$
D_{\mathrm{in}}\frac{3C_i - 4C_{i-1} + C_{i-2}}{2\,\Delta r_-} = D_{\mathrm{out}}\frac{-3C_i + 4C_{i+1} - C_{i+2}}{2\,\Delta r_+}.
$$

Three defects:

- The row has five entries, so the matrix is no longer tridiagonal.
- $C_{i\pm2}$ enter with the sign opposite to that of $C_{i\pm1}$, so the off-diagonal sign pattern needed for an M-matrix (Section 9) is lost.
- The node has no storage. The system becomes differential-algebraic, and the inventory loses the interface node's weight (29), so the balance (21) no longer closes. Row $i$ is also not diagonally dominant: its off-diagonal magnitudes sum to $\frac{5}{2}\left(D_{\mathrm{in}}/\Delta r_- + D_{\mathrm{out}}/\Delta r_+\right)$, against the diagonal $\frac{3}{2}\left(D_{\mathrm{in}}/\Delta r_- + D_{\mathrm{out}}/\Delta r_+\right)$. In derivation B's scratch runs the relative inventory drift was $3.0\times10^{-2}$, $8.8\times10^{-3}$ and $2.4\times10^{-3}$ at 20, 40 and 80 intervals per layer.

**(c) One-sided first-order differences.** The condition $D_{\mathrm{in}}(C_i - C_{i-1})/\Delta r_- = D_{\mathrm{out}}(C_{i+1} - C_i)/\Delta r_+$ is tridiagonal and has the right signs. It is Eq. (30) with $V_i = 0$ and with planar instead of spherical conductances. Its flux is only first-order accurate. It also removes the storage of two half-intervals, so the trapezoidal inventory is not conserved.

**(d) Nodes off the interfaces, with harmonic-mean diffusivities.** This works, but it requires a series conductance for every interval that straddles an interface, $K = r_i r_{i+1}\big/\left[\left(R - r_i\right)/D_{\mathrm{in}} + \left(r_{i+1} - R\right)/D_{\mathrm{out}}\right]$. It is tridiagonal, conservative and second order (derivation B observed order 2.0 on the slowest eigenvalue). But $C(R)$ is not an unknown and must be reconstructed, and it is in substance the finite-volume layout the directive retires. With nodes on the interfaces, every interval lies in one material and no averaging is needed.

**Decision.** Use the half-interval flux balance (30) to (32). It keeps nodal unknowns and continuity of concentration. It keeps the tridiagonal structure and the M-matrix sign pattern (Section 9). It conserves the trapezoidal inventory exactly, gives the owner's ghost-node row on equal spacing, and never applies a constant-$D$ stencil across a jump. Hypothesis (3) is **proved**.

---

## 6. A4: the absorbing outer boundary and the release rate

**The condition.** The approved boundary is $C(R_5, t) = 0$. Node $N$ lies on $R_5$, so

$$
C_N^j = 0 \qquad \text{for every } j \ge 0. \tag{34a}
$$

This is a Dirichlet condition: it fixes the value, so node $N$ needs no equation and no ghost node. The owner's insulated treatment ((T21) to (T23), and the incorrect (H17)) is not used. It describes a different physical surface, one that releases nothing.

**The last matrix row.** Row $N-1$ is an interior OPyC row (15), or an interface row (31) if the OPyC has a single interval. Its upper neighbour is $C_N^{j+1} = 0$, so the term $U_{N-1}C_N^{j+1}$ vanishes. The right-hand side gains nothing, because the boundary value is zero:

$$
L_{N-1}C_{N-2}^{j+1} + B_{N-1}C_{N-1}^{j+1} = C_{N-1}^j, \qquad B_{N-1} = 1 - L_{N-1} - U_{N-1}. \tag{34b}
$$

$B_{N-1}$ keeps the full value $1 - L_{N-1} - U_{N-1}$. The coupling to the boundary remains in the diagonal; only the off-diagonal entry is removed. The unknowns are $C_0^{j+1}, \dots, C_{N-1}^{j+1}$: $N$ unknowns and $N$ rows.

**The release rate, from the discrete balance.** Insert $C_N^{j+1} = 0$ and $K_{0,\mathrm{outer}} = 0$ into Eq. (21), and multiply by $4\pi$. With the inventory

$$
M^j = 4\pi\sum_{i=1}^{N-1}V_i\,C_i^j, \tag{34c}
$$

the balance becomes

$$
\frac{M^{j+1} - M^j}{\Delta t} = -4\pi K_{N-1,\mathrm{outer}}\,C_{N-1}^{j+1} \equiv -\Phi^{j+1}. \tag{35}
$$

The release rate is therefore

$$
\boxed{\Phi^{j+1} = 4\pi\,\frac{D_{\mathrm{OPyC}}\,r_{N-1}\,R_5}{\Delta r_{N-1,\mathrm{outer}}}\,C_{N-1}^{j+1}} \qquad \left(\mathrm{m^3\ s^{-1}}\times\mathrm{mol\ m^{-3}} = \mathrm{mol\ s^{-1}}\right). \tag{36}
$$

It is the discrete counterpart of $\Phi_R = -4\pi\left(R_5\right)^2 D_{\mathrm{OPyC}}\,\partial_r C(R_5, t)$ (v1.0.0 Part III, Eq. (3.7)). The one-sided gradient $(0 - C_{N-1})/\Delta r_{N-1,\mathrm{outer}}$ and the factor $r_{N-1}R_5$ in place of $\left(R_5\right)^2$ come from the exact shell conductance (19).

**Cumulative release, balance and release fraction.**

$$
M_{\mathrm{rel}}^j = \sum_{k=1}^{j}\Delta t\,\Phi^k, \qquad M^j + M_{\mathrm{rel}}^j = M^0 \ \text{(exactly, in exact arithmetic)}, \qquad F^j = \frac{M_{\mathrm{rel}}^j}{M^0} = 1 - \frac{M^j}{M^0}. \tag{37}
$$

With a variable time step, $\Delta t$ in the sum is the length of step $k$. The balance follows by summing Eq. (35) over steps $1, \dots, j$. The two expressions for $F^j$ are equal in exact arithmetic. Their difference in floating point is the balance residual that directive Phase D, test F, asks to be reported.

**Sign and bounds.** Section 9 proves $C_{N-1}^{j+1} \ge 0$, so $\Phi^{j+1} \ge 0$: the release never runs backwards, and $F^j$ never decreases. Since $M^j \ge 0$, $F^j \le 1$.

**Accuracy of the release rate.** $\Phi^{j+1}$ is the flow through the last interval, not exactly at $R_5$. Two statements must be kept apart: the consistency of formula (36), which is proved, and the convergence of the computed release rate, which is observed.

*Consistency (PROVED, from derivation B, for a solution that is smooth in the OPyC near $R_5$, with no source).* Write $h = \Delta r_{N-1,\mathrm{outer}}$ and insert the exact solution into Eq. (36). Taylor's theorem about $R_5$, with $C(R_5, t) = 0$, gives $C(r_{N-1}) = -h\,C'(R_5) + \frac{\left(h\right)^2}{2}C''(R_5) + O\left(\left(h\right)^3\right)$, so

$$
\frac{r_{N-1}R_5\,C(r_{N-1})}{h} = R_5\left(R_5 - h\right)\left[-C' + \frac{h}{2}C''\right] + O\left(\left(h\right)^2\right) = \left(R_5\right)^2\left[-C' + h\left(\frac{1}{2}C'' + \frac{1}{R_5}C'\right)\right] + O\left(\left(h\right)^2\right),
$$

with all derivatives at $R_5^-$. Because $C(R_5, t) = 0$ for every $t$, $\partial C/\partial t = 0$ at $R_5$, and Eq. (5) there gives $C'' = -\frac{2}{R_5}C'$. The bracket multiplying $h$ is therefore zero, and

$$
4\pi D_{\mathrm{OPyC}}\,\frac{r_{N-1}R_5\,C(r_{N-1}, t)}{\Delta r_{N-1,\mathrm{outer}}} = \Phi_R(t) + O\left(\left(\Delta r_{N-1,\mathrm{outer}}\right)^2\right). \tag{36a}
$$

The cancellation uses $\partial_t C(R_5) = 0$ and the absence of a source at $R_5$. With a source that is non-zero at $R_5$ (Phase G1), the bracket is $-G(R_5)/\left(2D_{\mathrm{OPyC}}\right)$, not zero, and the release formula must be revisited in the G1 derivation (Section 11).

*Convergence of the computed release rate (OBSERVED, not proven).* Eq. (36) divides the computed $C_{N-1}^{j+1}$ by $\Delta r_{N-1,\mathrm{outer}}$. A nodal error of order $\left(\Delta r\right)^2$ at node $N-1$ would therefore allow only first order in $\Phi$. Second order requires the nodal error itself to vanish linearly towards the Dirichlet node. That is what scratch runs show, but it is not proved here.

A scratch check on the homogeneous sphere supports second order. Setup: $R_5 = 1$, $D = 1$, uniform start, semi-discrete solution integrated exactly in time, with the rate normalised by the exact initial inventory. The error in $\Phi/M^0$ fell from $-2.7\times10^{-1}$ to $-1.0\times10^{-3}$ at $t = 0.01$ as $N$ went from 20 to 320, and from $-1.7\times10^{-3}$ to $-6.5\times10^{-6}$ at $t = 0.2$. The observed orders were $2.00$ to $2.03$. A separate scratch program written for the reconciliation reproduced these numbers to the digits printed, and gave orders $2.00$ to $2.01$ at $t = 0.05$.

---

## 7. A5: the initial condition on the grid

**The continuous profile** is the directive's, with $C_{\mathrm{init}}$ in place of the clashing symbol $C_0$:

$$
C(r,0) = \begin{cases} C_{\mathrm{init}}, & 0 \le r < R_1,\\ 0, & R_1 < r < R_5. \end{cases} \tag{38}
$$

It replaces the owner's (H11) and (T13), which are not initial conditions (document 00). The profile jumps at $R_1$, which is the node $N_{\mathrm{kernel}}$.

**Nodal values.**

$$
C_i^0 = C_{\mathrm{init}} \ \ (0 \le i < N_{\mathrm{kernel}}), \qquad C_{N_{\mathrm{kernel}}}^0 = \theta\,C_{\mathrm{init}}, \qquad C_i^0 = 0 \ \ (N_{\mathrm{kernel}} < i \le N). \tag{39}
$$

Only $\theta$ has to be decided, because Eq. (38) leaves the value at $r = R_1$ open.

**Choice of $\theta$ from the discrete inventory (coordinator's hypothesis 5).** Require the discrete initial inventory (34c) to equal the exact initial inventory $\frac{4\pi}{3}\left(R_1\right)^3 C_{\mathrm{init}}$:

$$
\sum_{i=1}^{N_{\mathrm{kernel}}-1}V_i + \theta\,V_{N_{\mathrm{kernel}}} = \frac{\left(R_1\right)^3}{3}. \tag{40}
$$

On the kernel, $r_i = i\,\Delta r_{\mathrm{kernel}}$ and $V_i = \left(i\,\Delta r_{\mathrm{kernel}}\right)^2\Delta r_{\mathrm{kernel}}$. Writing $n = N_{\mathrm{kernel}}$ and using $\sum_{i=1}^{n-1}\left(i\right)^2 = \frac{(n-1)\,n\,(2n-1)}{6}$, with $n\,\Delta r_{\mathrm{kernel}} = R_1$:

$$
\sum_{i=1}^{n-1}V_i = \left(\Delta r_{\mathrm{kernel}}\right)^3\frac{2\left(n\right)^3 - 3\left(n\right)^2 + n}{6} = \frac{\left(R_1\right)^3}{3} - \frac{\left(R_1\right)^2\Delta r_{\mathrm{kernel}}}{2} + \frac{R_1\left(\Delta r_{\mathrm{kernel}}\right)^2}{6}.
$$

The interface weight is $V_{N_{\mathrm{kernel}}} = \left(R_1\right)^2\left(\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}\right)/2$, by Eq. (29). Solving Eq. (40) for $\theta$:

$$
\boxed{\theta = \frac{\Delta r_{\mathrm{kernel}} - \dfrac{\left(\Delta r_{\mathrm{kernel}}\right)^2}{3R_1}}{\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}}} \tag{41}
$$

Since $\Delta r_{\mathrm{kernel}}/R_1 = 1/N_{\mathrm{kernel}}$, Eq. (41) can also be written

$$
\theta = \frac{\Delta r_{\mathrm{kernel}}}{\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}}\left(1 - \frac{1}{3N_{\mathrm{kernel}}}\right), \tag{41a}
$$

which is the form derivation B reached independently: the kernel's share of the node's two half-intervals, times a factor that corrects the trapezoidal-rule error of the kernel sum. For the synthetic grid with 40 intervals per layer, $\theta = \frac{2}{3}\left(1 - \frac{1}{120}\right) = 0.66\overline{1}$.

On a general kernel grid, θ is computed directly from Eq. (40). Three properties:

- **Range.** $0 < \theta < 1$: the numerator is positive because $\Delta r_{\mathrm{kernel}} \le R_1$, and it is smaller than the denominator.
- **Exact inventory.** $M^0$ equals the exact initial inventory to round-off, so the two forms of $F^j$ in Eq. (37) need no further normalisation choice.
- **Limit.** As the grid is refined, $\theta \to \Delta r_{\mathrm{kernel}}/(\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}})$: the share of the interface node's storage that lies on the kernel side. This limit is $\frac{1}{2}$ only when the kernel and buffer spacings are equal. With equal spacings, Eq. (41) gives $\theta = \frac{1}{2} - \frac{\Delta r}{6R_1}$.

**Why it matters (scratch experiments).** Synthetic five-layer case, $m$ intervals in every layer, so that $\Delta r_{\mathrm{kernel}} = 2\Delta r_{\mathrm{buffer}}$. The semi-discrete system was integrated exactly in time, and the error is in $F$ at $t = 2\times10^5\ \mathrm{s}$ against the existing high-precision reference $F = 0.43255563909304$ (v1.0.0 Part IV). In this table $F$ is $1 - M^j/M^0_{\mathrm{exact}}$, normalised by the exact initial inventory $M^0_{\mathrm{exact}} = \frac{4\pi}{3}\left(R_1\right)^3C_{\mathrm{init}}$, so that the inventory error of each choice of $\theta$ is visible (the reconciliation reproduced every entry of the first three columns to the digits printed with this normalisation):

| $\theta$ | error, $m = 10$ | $m = 20$ | $m = 40$ | $m = 80$ | $m = 160$ | observed order |
|---|---|---|---|---|---|---|
| Eq. (41) (tends to $\frac{2}{3}$) | $-7.99\times10^{-5}$ | $-2.00\times10^{-5}$ | $-5.00\times10^{-6}$ | $-1.25\times10^{-6}$ | $-3.13\times10^{-7}$ (note) | $2.00$ |
| $\frac{1}{2}$ | $+1.82\times10^{-2}$ | $+9.82\times10^{-3}$ | $+5.09\times10^{-3}$ | $+2.59\times10^{-3}$ | $+1.31\times10^{-3}$ | $0.9$ to $1.0$ |
| $1$ (kernel value) | $-4.51\times10^{-2}$ | $-2.18\times10^{-2}$ | $-1.07\times10^{-2}$ | $-5.32\times10^{-3}$ | $-2.65\times10^{-3}$ | $1.0$ |
| $0$ (buffer value) | $+8.15\times10^{-2}$ | $+4.15\times10^{-2}$ | $+2.09\times10^{-2}$ | $+1.05\times10^{-2}$ | $+5.26\times10^{-3}$ | $1.0$ |

*Note on the $m = 160$ entry of the first row.* Derivation A's exact-time-integration program gave $-3.03\times10^{-7}$ there, and with it a last observed order of $2.05$. That value is attributed to rounding in the eigen-decomposition of the $800 \times 800$ symmetrised operator: an independent dense eigen-decomposition reproduced $-3.033\times10^{-7}$ at $m = 160$ and gave an implausible $-3.83\times10^{-8}$ at $m = 320$ (order $2.99$), while two independent backward-Euler programs with three-level Richardson extrapolation in $\Delta t$ (the verifier's, and a reconciliation run with four sequences of $\Delta t$ from $100$ down to $3.125\ \mathrm{s}$) gave $-3.129\times10^{-7}$ to $-3.132\times10^{-7}$ at $m = 160$ and $-1.2511\times10^{-6}$ to $-1.2512\times10^{-6}$ at $m = 80$: order $2.00$. The table shows the time-extrapolated value. The cause was not isolated further than this comparison.

On this grid only the inventory-matched value keeps second-order convergence. The reason is algebraic. A choice $\theta$ changes the initial inventory by $4\pi\left(\theta - \theta_{(41)}\right)V_{N_{\mathrm{kernel}}}C_{\mathrm{init}}$, a relative error of
$$
3\left(\theta - \theta_{(41)}\right)\frac{\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}}{2R_1}, \tag{41b}
$$
and the scheme conserves that wrong inventory for ever (Eq. (37)). The error is $O(\Delta r)$ whenever $\theta - \theta_{(41)}$ does not tend to zero, that is, whenever $\theta$ does not tend to the limit $\Delta r_{\mathrm{kernel}}/\left(\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}\right)$.

**Correction from the reconciliation: $\theta = \frac{1}{2}$ on equal spacing.** An earlier version of this section said that any fixed choice, including $\frac{1}{2}$, gives first order. That holds only when $\Delta r_{\mathrm{kernel}} \ne \Delta r_{\mathrm{buffer}}$. With equal spacings, $\theta_{(41)} = \frac{1}{2} - \frac{1}{6N_{\mathrm{kernel}}}$, so $\theta = \frac{1}{2}$ gives by Eq. (41b) a relative inventory error of $1/\left(2\left(N_{\mathrm{kernel}}\right)^2\right)$: second order. Derivation B observed this order (homogeneous sphere, archive Section 10.5), and a reconciliation run on the synthetic case with equal spacings at $R_1$ (kernel $2m$ intervals, the other layers $m$) gave orders $2.00$, $2.00$, $2.00$, $1.96$ for $\theta = \frac{1}{2}$ and $2.00$, $2.00$, $2.00$, $1.95$ for Eq. (41), against $1.10$ to $1.01$ for $\theta = 1$. Eq. (41) is the only choice that is second order on every grid, and it makes $M^0$ exact.

**Decision:** use Eq. (40), or Eq. (41) on a uniform kernel grid. The centre value is $C_0^0 = C_{\mathrm{init}}$.

**Early times.** The jump at $R_1$ makes the solution non-smooth at $t = 0$. Two effects are visible in the scratch runs (synthetic case, $\theta$ from Eq. (41)).

- **In space.** Maximum-norm differences of the nodal profile, between successive grids $m = 10, 20, \dots, 160$, measured against $m = 320$:
  - at $t = 200\ \mathrm{s}$: $2.5\times10^{-2}$, $7.2\times10^{-3}$, $1.8\times10^{-3}$, $4.3\times10^{-4}$, $8.7\times10^{-5}$ (orders $1.81$, $1.98$, $2.07$; the last order, $2.32$, is inflated by comparison with the reference grid);
  - from $t = 2000\ \mathrm{s}$ on: orders $1.95$ to $2.07$.
  - The second-order regime begins once $\Delta r_{\mathrm{kernel}}$ is small compared with the kernel diffusion length $\sqrt{D_{\mathrm{kernel}}\,t}$, about $14\ \mu\mathrm{m}$ at $t = 200\ \mathrm{s}$. Before that, the coarse grids do not resolve the front.
  - At such early times $F$ itself is of order $10^{-12}$ to $10^{-10}$ (nothing has reached $R_5$), so convergence of $F$ cannot be measured there. The profile, not $F$, is the right early-time test quantity.
- **In time.** For a fixed grid ($m = 40$), the maximum-norm difference between backward Euler and exact time integration at $t = 2000\ \mathrm{s}$ halved with each halving of $\Delta t$: orders $0.83$, $0.91$, $0.97$, $0.98$, $0.99$ for 1 to 32 steps. The same differences at $t = 2\times10^4\ \mathrm{s}$ gave orders between $0.92$ and $1.23$, pre-asymptotic.
- **Non-negativity.** All values stayed non-negative, as Section 9 proves. Backward Euler damps the high-frequency content of the jump strongly. (Crank-Nicolson would damp it only weakly, document 00, Section 2.)

**A note for the homogeneous-sphere benchmark (directive Phase D, test B).** There the uniform start reaches the Dirichlet node $N$, where $C_N^0 = 0$ is forced. The discrete $M^0$ is then smaller than the exact one by about $4\pi\left(R_5\right)^2\frac{\Delta r}{2}C_{\mathrm{init}}$. In a scratch check ($N = 20$ to $320$), $F = 1 - M^j/M^0$ converged at order $2.00$ when $M^0$ was the exact inventory, and only at order $1.00$ when it was the discrete one. The benchmark should normalise by the exact $M^0$. For the STRATO start (38) the question does not arise, because Eq. (40) makes the two equal.

---

## 8. A6: non-uniform grids

### 8.1 Is a non-uniform grid advantageous?

A uniform grid would need every interface radius to be a node, so every layer thickness must be an integer multiple of $\Delta r$.

- **AGR-1 baseline** (`data/geometry/geometry.csv`, rows G-01 to G-05): kernel radius $175\ \mu\mathrm{m}$, then thicknesses $103.5$, $39.4$, $35.3$ and $41.0\ \mu\mathrm{m}$. The largest common divisor is $0.1\ \mu\mathrm{m}$, so a uniform grid needs $3942$ intervals, with no freedom to refine one layer without the others.
- **Synthetic case** (`inputs/synthetic/synthetic_five_layer.toml`): the radii allow $\Delta r = 50\ \mu\mathrm{m}$ (only $9$ intervals), and finer grids are its integer fractions.
- **Diffusivities.** In the synthetic case they range from $2.0\times10^{-14}$ (SiC) to $4.0\times10^{-10}\ \mathrm{m^2\ s^{-1}}$ (buffer), a factor of $2\times10^4$. The slope ratios at the interfaces are $D_{\mathrm{in}}/D_{\mathrm{out}}$ (v1.0.0 Part III, Section 3.2.2). The layers therefore need different resolutions.

**Decision: piecewise-uniform grid.**

- Each layer gets its own number of intervals, uniform within the layer:

  $$
  \Delta r_{\mathrm{kernel}} = \frac{R_1}{N_{\mathrm{kernel}}},\quad \Delta r_{\mathrm{buffer}} = \frac{R_2 - R_1}{N_{\mathrm{buffer}} - N_{\mathrm{kernel}}},\quad \Delta r_{\mathrm{IPyC}} = \frac{R_3 - R_2}{N_{\mathrm{IPyC}} - N_{\mathrm{buffer}}},\quad \Delta r_{\mathrm{SiC}} = \frac{R_4 - R_3}{N_{\mathrm{SiC}} - N_{\mathrm{IPyC}}},\quad \Delta r_{\mathrm{OPyC}} = \frac{R_5 - R_4}{N - N_{\mathrm{SiC}}}. \tag{42}
  $$

- The node radii are $r_i = R_{\mathrm{prev}} + (i - N_{\mathrm{prev}})\,\Delta r_{\mathrm{layer}}$, where $R_{\mathrm{prev}}$ and $N_{\mathrm{prev}}$ are the inner radius and node index of the layer. The interface nodes are set to the exact $R_1, \dots, R_5$, not accumulated by addition, so that no round-off drift moves an interface.
- Each layer needs at least one interval.
- The spacing changes only at the four interface nodes, which need special rows anyway (Section 5).
- Each layer can be refined independently.
- The scratch runs above, with equal counts per layer and therefore spacing ratios of $2:1$ at $R_1$, show second-order convergence.

Fully graded grids, for example geometric refinement towards $R_1$ for early times, are allowed by the same formulas (Section 8.2). Their benefit has not been established here. They are left as an option for the Phase D convergence study.

### 8.2 Coefficients on a general non-uniform grid

The uniform formulas (15) must not be reused when $\Delta r_{i,\mathrm{inner}} \ne \Delta r_{i,\mathrm{outer}}$. The general row is the balance (30), with $D_{i,\mathrm{inner}}$ and $D_{i,\mathrm{outer}}$ the diffusivities of the two adjacent intervals:

$$
K_{i,\mathrm{inner}} = \frac{D_{i,\mathrm{inner}}\,r_{i-1}\,r_i}{\Delta r_{i,\mathrm{inner}}}, \qquad K_{i,\mathrm{outer}} = \frac{D_{i,\mathrm{outer}}\,r_i\,r_{i+1}}{\Delta r_{i,\mathrm{outer}}}, \qquad V_i = \left(r_i\right)^2\,\frac{\Delta r_{i,\mathrm{inner}} + \Delta r_{i,\mathrm{outer}}}{2}, \qquad L_i = -\frac{\Delta t\,K_{i,\mathrm{inner}}}{V_i},\quad U_i = -\frac{\Delta t\,K_{i,\mathrm{outer}}}{V_i},\quad B_i = 1 - L_i - U_i. \tag{43}
$$

Eq. (43) covers every node $1 \le i \le N-1$: interior nodes of a layer, interface nodes, and nodes of a graded grid. On a piecewise-uniform grid it reduces to Eq. (15) inside each layer (with $\Delta r = \Delta r_{\mathrm{layer}}$) and to Eqs. (32a) and (32b) at the interfaces.

**Consistency of Eq. (43) where the spacing changes inside one material.** Set $D_{i,\mathrm{inner}} = D_{i,\mathrm{outer}} = D$, $\Delta r_- = \Delta r_{i,\mathrm{inner}}$ and $\Delta r_+ = \Delta r_{i,\mathrm{outer}}$, and expand with Eq. (6) using the one-sided spacings:

$$
\frac{K_{i,\mathrm{outer}}\left(C_{i+1} - C_i\right) - K_{i,\mathrm{inner}}\left(C_i - C_{i-1}\right)}{V_i} = D\left[C_i'' + \frac{2}{r_i}C_i' + \frac{\Delta r_+ - \Delta r_-}{r_i}C_i'' + \frac{\Delta r_+ - \Delta r_-}{3}C_i'''\right] + O\left(\left(\Delta r\right)^2\right). \tag{44}
$$

The error is of order $\Delta r_+ - \Delta r_-$:

- On a uniform or smoothly graded grid, $\Delta r_+ - \Delta r_- = O\left(\left(\Delta r\right)^2\right)$, and the scheme is second order.
- At an isolated jump in spacing, the local error is first order. As at the interfaces, this did not reduce the global order in the scratch runs. On a piecewise-uniform grid (Section 8.1) the spacing jumps only at the interface nodes, which already carry a first-order local error from the kink, so the jump adds no new kind of error.
- Derivation B observed second order for the homogeneous eigenvalue even on a grid whose spacing alternates between $h$ and $2h$ at every node (archive, Section 10.6), where every node has a first-order local error. This is an instance of the known superconvergence of conservative three-point schemes; it is OBSERVED, not proved here, and production grids do not rely on it.

**Comparison with the standard non-uniform three-point formulas.** The Lagrange three-point approximations of $C''$ and $C'$ on unequal spacing give

$$
\frac{dC_i}{dt} = \frac{2\left(F_+ - F_-\right)}{\Delta r_- + \Delta r_+} + \frac{2}{r_i}\,\frac{\Delta r_-F_+ + \Delta r_+F_-}{\Delta r_- + \Delta r_+},
$$

with $F_+ = D\left(C_{i+1} - C_i\right)/\Delta r_+$ and $F_- = D\left(C_i - C_{i-1}\right)/\Delta r_-$, as in Section 5.6. This is formula (33a) again. It is formally more accurate at a spacing jump, but it is not conservative.

Its coefficient of $C_{i-1}$ is $\frac{2D\left(r_i - \Delta r_+\right)}{r_i\,\Delta r_-\left(\Delta r_- + \Delta r_+\right)}$, which becomes negative, giving a wrong-sign off-diagonal, whenever $\Delta r_{i,\mathrm{outer}} > r_i$. On a grid that grows outward from the centre this already happens at node 1, if $\Delta r_{1,\mathrm{outer}} > \Delta r_{0,\mathrm{outer}}$. The corresponding coefficient in Eq. (43) is $\frac{2D\left(r_i - \Delta r_-\right)}{r_i\,\Delta r_-\left(\Delta r_- + \Delta r_+\right)} = \frac{2D\,r_{i-1}}{r_i\,\Delta r_-\left(\Delta r_- + \Delta r_+\right)} \ge 0$ for every grid. This is one of the reasons Eq. (43) is chosen.

**Temperature-dependent diffusivity (Phase G3, forward pointer).** Eq. (43) needs one diffusivity per interval. With a temperature that is uniform within the particle, each interval takes $D_{\mathrm{layer}}(T)$ of its material. The matrix is reassembled whenever $T$ changes, and no structural change is needed.

---

## 9. Matrix properties used by the scheme (coordinator's hypothesis 4)

The system at each step is $\mathbf{A}\,\mathbf{C}^{j+1} = \mathbf{C}^j$, with $\mathbf{C}^j = \left(C_0^j, \dots, C_{N-1}^j\right)^{\mathsf T}$ and $\mathbf{A}$ tridiagonal with rows (25), (43) and (34b).

### 9.1 Sign pattern and diagonal dominance

For every $\Delta t > 0$, every grid with positive spacings, and every set of positive diffusivities:

1. **Off-diagonals are non-positive.**
   - Row 0: $U_0 = -6\,\mathrm{Fo}_{\mathrm{kernel}} < 0$.
   - Rows $1 \le i \le N-1$: $L_i = -\Delta t\,K_{i,\mathrm{inner}}/V_i \le 0$ and $U_i = -\Delta t\,K_{i,\mathrm{outer}}/V_i < 0$, because $K_{i,\mathrm{inner}} = D_{i,\mathrm{inner}}\,r_{i-1}\,r_i/\Delta r_{i,\mathrm{inner}} \ge 0$ and $K_{i,\mathrm{outer}} = D_{i,\mathrm{outer}}\,r_i\,r_{i+1}/\Delta r_{i,\mathrm{outer}} > 0$ (zero only for $K_{1,\mathrm{inner}} = K_{0,\mathrm{outer}}$, where $r_0 = 0$) and $V_i > 0$.
2. **Diagonals are positive, with excess at least one.** $B_i = 1 + |L_i| + |U_i|$ in rows $0$ to $N-2$; for row 0, read $|L_0| = 0$. In row $N-1$ the entry $U_{N-1}$ is removed but $B_{N-1}$ keeps it, so $B_{N-1} - |L_{N-1}| = 1 + |U_{N-1}|$. Every row therefore satisfies
   $$B_i - \left(|L_i| + |U_i|_{\text{in matrix}}\right) \ge 1. \tag{45}$$

These hold with no grid condition: at interface nodes, at the centre, on any non-uniform grid built with Eq. (43), and for any time step.

By contrast:

- FTCS needs $\mathrm{Fo} \le \frac{1}{6}$ for non-negative coefficients (document 00).
- The Lagrange form needs $\Delta r_{i,\mathrm{outer}} \le r_i$ (Section 8.2).
- Generation and decay (Phase G) keep both properties. Decay adds $\lambda\,\Delta t \ge 0$ to each $B_i$. A generation term $G_i \ge 0$ (in $\mathrm{mol\ m^{-3}\ s^{-1}}$) adds $\Delta t\,G_i \ge 0$ to the right-hand side.

### 9.2 Solvability, stability in the maximum norm, and non-negativity

A matrix with non-positive off-diagonals, positive diagonal and strict row diagonal dominance is a non-singular M-matrix, with an entrywise non-negative inverse. The three consequences used here can be proved directly from Eq. (45).

**(i) Maximum-norm bound.** Let $\mathbf{A}\mathbf{x} = \mathbf{b}$, and let $k$ be an index with $|x_k| = \max_i|x_i|$. Row $k$ gives $B_k x_k = b_k - L_k x_{k-1} - U_k x_{k+1}$, so

$$
B_k|x_k| \le |b_k| + \left(|L_k| + |U_k|\right)|x_k| \ \Longrightarrow\ \left(B_k - |L_k| - |U_k|\right)|x_k| \le |b_k| \ \Longrightarrow\ \max_i|x_i| \le \max_i|b_i|, \tag{46}
$$

using Eq. (45). Two consequences:

- **Solvability.** With $\mathbf{b} = \mathbf{0}$, Eq. (46) forces $\mathbf{x} = \mathbf{0}$, so $\mathbf{A}$ is non-singular and every step has exactly one solution.
- **Unconditional stability.** With $\mathbf{b} = \mathbf{C}^j$: $\max_i|C_i^{j+1}| \le \max_i|C_i^j|$ for every $\Delta t$.

**(ii) Non-negativity.** Let $\mathbf{b} \ge \mathbf{0}$, and suppose the smallest entry $x_k$ of the solution is negative. In row $k$, the neighbours satisfy $x_{k\pm1} \ge x_k$, and $-L_k = |L_k|$ and $-U_k = |U_k|$ are non-negative. So

$$
B_k x_k = b_k + |L_k|\,x_{k-1} + |U_k|\,x_{k+1} \ge b_k + \left(|L_k| + |U_k|\right)x_k \ \Longrightarrow\ \left(B_k - |L_k| - |U_k|\right)x_k \ge b_k \ge 0,
$$

which by Eq. (45) gives $x_k \ge 0$, a contradiction. Hence $\mathbf{C}^j \ge \mathbf{0}$ implies $\mathbf{C}^{j+1} \ge \mathbf{0}$. The initial data (39) are non-negative, so all concentrations stay non-negative for every $\Delta t$ and every grid.

**(iii) Discrete maximum principle.** Every row sums to $1$, except row $N-1$, whose sum $1 + |U_{N-1}|$ exceeds $1$. Combining (i) and (ii): for non-negative data, $0 \le C_i^{j+1} \le \max_k C_k^j$. No new maximum and no negative value can be created in a step.

**(iv) Inverse-positivity (from derivation B).** Write $\mathbf{A} = \mathcal{D}\left(\mathbf{I} - \mathbf{E}\right)$ with $\mathcal{D} = \mathrm{diag}(B_i) > 0$. The matrix $\mathbf{E} = \mathbf{I} - \mathcal{D}^{-1}\mathbf{A}$ has zero diagonal, off-diagonal entries $|L_i|/B_i \ge 0$ and $|U_i|/B_i \ge 0$, and row sums $\left(|L_i| + |U_i|_{\text{in matrix}}\right)/B_i < 1$ by Eq. (45). So $\left\|\mathbf{E}\right\|_\infty < 1$, the Neumann series $\left(\mathbf{I} - \mathbf{E}\right)^{-1} = \sum_{k \ge 0}\mathbf{E}^k$ converges and is entrywise non-negative, and
$$
\mathbf{A}^{-1} = \left(\mathbf{I} - \mathbf{E}\right)^{-1}\mathcal{D}^{-1} \ge 0 \ \text{(entrywise)}, \qquad \left\|\mathbf{A}^{-1}\right\|_\infty \le 1. \tag{46a}
$$
$\mathbf{A}$ is therefore a non-singular M-matrix. Column 0 of $\mathbf{A}^{-1}$ is zero below the diagonal: this is the one-way coupling of Section 4.3 seen in the inverse (derivation B observed these exact zeros).

**(v) Weighted energy (PROVED, from derivation B).** Let $\left\|\mathbf{C}^j\right\|_V^2 = \sum_{i=1}^{N-1}V_i\left(C_i^j\right)^2$. Multiply the balance (30) (in its general form (43)) by $C_i^{j+1}$ and sum over $i = 1, \dots, N-1$. Summation by parts, with $K_{0,\mathrm{outer}} = 0$ and $C_N^{j+1} = 0$, gives
$$
\sum_{i=1}^{N-1}V_i\left(C_i^{j+1} - C_i^j\right)C_i^{j+1} = -\Delta t\sum_{i=0}^{N-1}K_{i,\mathrm{outer}}\left(C_{i+1}^{j+1} - C_i^{j+1}\right)^2 \le 0. \tag{46b}
$$
With $\left(x - y\right)x = \frac{1}{2}\left(\left(x\right)^2 - \left(y\right)^2\right) + \frac{1}{2}\left(x - y\right)^2 \ge \frac{1}{2}\left(\left(x\right)^2 - \left(y\right)^2\right)$, this gives $\left\|\mathbf{C}^{j+1}\right\|_V \le \left\|\mathbf{C}^j\right\|_V$ for every $\Delta t > 0$.

**(vi) Non-negativity in floating point (PROVED, from derivation B).** Code the Thomas algorithm of Section 9.3 as $m_i = B_i - \left(L_i/m_{i-1}\right)U_{i-1}$, $b'_i = b_i - \left(L_i/m_{i-1}\right)b'_{i-1}$ and $x_i = \left(b'_i - U_i\,x_{i+1}\right)/m_i$, with $L_i \le 0$ and $U_i \le 0$ stored with their signs. If every computed pivot $m_i$ is positive, then $\mathbf{b} \ge \mathbf{0}$ gives a computed $\mathbf{x} \ge \mathbf{0}$ in IEEE arithmetic. Proof: $L_i/m_{i-1} \le 0$ and $b'_{i-1} \ge 0$, so $b'_i$ is a sum of non-negative numbers; likewise $-U_i\,x_{i+1} \ge 0$; each operation acts on non-negative operands, and rounding a non-negative exact result gives a non-negative number. $\square$ The hypothesis on the computed pivots holds whenever rounding does not destroy the margin of Section 9.3, which is at least $1$ in exact arithmetic. This statement is for the Thomas algorithm in this form only; for another linear solver, positivity under rounding is not claimed.

Hypothesis (4) is therefore **proved, with no grid condition**, for the scheme of Sections 3 to 8. "Unconditionally stable" here means stable in the maximum norm and in the weighted norm $\left\|\cdot\right\|_V$. The scratch runs also showed no negative values, including a single step of $\Delta t = 10^9\ \mathrm{s}$ on the synthetic grid.

**Generation and decay (forward pointer).** Decay adds $\lambda\,\Delta t$ to every $B_i$ and raises the margin of Eq. (45) to $1 + \lambda\,\Delta t$, so $\left\|\mathbf{A}^{-1}\right\|_\infty \le 1/\left(1 + \lambda\,\Delta t\right)$ and (i) to (vi) hold unchanged. A non-negative generation term keeps the right-hand side non-negative, so (ii) holds, but the upper bound of (iii) does not. These are statements about the matrix only; the derivation of the source and decay terms is Phase G (Document 2, Section 9).

### 9.3 The Thomas algorithm, and why it replaces "making $C_{i+1}$ the subject"

Gaussian elimination specialised to three diagonals runs in two sweeps. The forward sweep computes pivots $m_0 = B_0$ and $m_i = B_i - L_i\,U_{i-1}/m_{i-1}$. The backward sweep substitutes from row $N-1$ to row $0$.

**Claim.** $m_i \ge 1 + |U_i|$ for $0 \le i \le N-2$, and $m_{N-1} \ge 1$.

**Proof.** $m_0 = B_0 = 1 + |U_0|$. If $m_{i-1} \ge 1 + |U_{i-1}|$, then $|U_{i-1}|/m_{i-1} < 1$, and

$$
m_i = 1 + |L_i| + |U_i| - |L_i|\,\frac{|U_{i-1}|}{m_{i-1}} \ge 1 + |U_i|.
$$

For the last row the same bound holds with $|U_{N-1}|$ removed from the matrix. $\square$

Consequences:

- No pivot is zero, and no pivoting is needed.
- The multipliers $|U_i|/m_i$ are smaller than one, so errors are damped, not amplified, during the backward sweep.
- The cost is proportional to $N$ per time step.

The owner's attempt to make $C_{i+1}$ the subject (document 00, (H21) to (H26)) is the same elimination run in the unstable direction: it divides by $U_i$, whose magnitude can be smaller than the other terms of the row. In a scratch run of that marching on 60 nodes with $\mathrm{Fo} = 3.6$, a relative perturbation of $10^{-15}$ grew to $4.3\times10^{-4}$.

### 9.4 Conservation in floating point

Eqs. (21), (35) and (37) are exact in exact arithmetic. In floating point the residual $\varepsilon^j = M^j + M_{\mathrm{rel}}^j - M^0$ is not zero. Each solve has a backward error proportional to the unit roundoff $u = 2^{-53}$ times the matrix entries, and the entries grow with the largest $\mathrm{Fo}$; the computed margin $B_i - |L_i| - |U_i|$ itself differs from $1$ by rounding of order $u\,B_i$ (a scratch run with $\Delta t = 10^9\ \mathrm{s}$, where $B_i$ reaches about $10^{11}$, gave a smallest computed margin of $0.999992$). The three scratch programs gave, for the same case (synthetic, 40 intervals per layer, $\Delta t = 200\ \mathrm{s}$, 1000 steps), the following largest relative residuals $\max_j\left|\varepsilon^j\right|/M^0$:

| Implementation | Largest relative residual |
|---|---|
| derivation A, rows divided by $V_i$, plain Thomas algorithm | $3.75\times10^{-11}$ |
| derivation B, rows divided by $V_i$ | $9.45\times10^{-11}$ |
| derivation B, rows multiplied by $V_i$ (symmetric for $i \ge 1$), compensated summation of $M_{\mathrm{rel}}$ | $4.27\times10^{-12}$ |
| derivation B, rows divided by $V_i$, $\Delta t = 2\ \mathrm{s}$ | $3.26\times10^{-13}$ |
| reconciliation, rows divided by $V_i$ | $6.91\times10^{-12}$ |
| reconciliation, rows multiplied by $V_i$, plain summation | $1.27\times10^{-11}$ |

The spread over a factor of about 20 for the same equations, and the opposite ranking of the two row scalings in two programs, show that the residual is rounding and depends on summation order and implementation details. No row scaling is shown to be better. The residual is larger than the $2.8\times10^{-13}$ of the retired finite-volume code (v1.0.0 Part V, Section 5.10.3) and smaller at smaller mesh ratios. **No exact conservation in floating point is claimed.** The Phase D test tolerance must be derived from an analysis of the rounding in the Rust implementation (the mesh ratio and the step count enter), not chosen to pass and not taken from this table. That analysis is [Document 5](05_generation_decay_temperature.md), Section 5: the residual is bounded rigorously, a posteriori by Document 2 (F44) and a priori by (F45), about $7u$ per step times $1 + 2\Lambda_{\max}$ with $\Lambda_{\max}$ about twice the largest $\mathrm{Fo}$; the a-posteriori bound that the Rust tests compute has the same per-step constant to first order. The Rust measurements on the synthetic case with $\Delta t = 200\ \mathrm{s}$ and 1000 steps ($6.9\times10^{-12}$ to $8.3\times10^{-10}$ at 20 to 320 intervals per layer with the (F24) elimination of the current code; $2.3\times10^{-11}$ to $1.19\times10^{-9}$ with the textbook order of the first run) lie below (F44) by a factor of at least $75$. The residual has no fixed range: it scales with the number of steps times the largest $\mathrm{Fo}$; coordinator decision 36 records up to $2.3\times10^{-9}$ at very large $\mathrm{Fo}$, and a synthetic scratch case of Document 5, Section 5.7 (largest $\mathrm{Fo}$ up to $1.1\times10^7$, 3000 steps) reached $5.0\times10^{-8}$ in the Rust operation order, still below (F44).

### 9.5 Convergence

**Consistency (PROVED).** Every row residual tends to zero as $\Delta r, \Delta t \to 0$. For a solution that is four times continuously differentiable in $r$ on each closed layer and twice continuously differentiable in $t$:

| Row | Spatial residual | Order |
|---|---|---|
| interior, uniform spacing | Eq. (17) | 2 |
| node 1 | Eq. (17a), $-\frac{5}{12}D_{\mathrm{kernel}}\left(\Delta r_{\mathrm{kernel}}\right)^2C''''(0)$ | 2 |
| centre, row 0 | $-\frac{1}{4}D_{\mathrm{kernel}}\left(\Delta r_{\mathrm{kernel}}\right)^2C''''(0)$ (Section 4.2) | 2 |
| spacing change inside one material | Eq. (44): $O\left(\left\lvert \Delta r_{i,\mathrm{outer}} - \Delta r_{i,\mathrm{inner}}\right\rvert \right) + O\left(\left(\Delta r\right)^2\right)$ | 1; 2 on a smoothly graded grid |
| interface node | Section 5.5: $O(\Delta r)$ | 1 |
| row $N-1$ | as interior; $C_N = 0$ is exact | 2 |
| release formula (36) | Eq. (36a) | 2 |

In time every row is first order, with residual $-\frac{\Delta t}{2}\ddot C$.

**Homogeneous sphere, maximum norm (PROVED, from derivation B).** One material, uniform grid, solution four times continuously differentiable in $r$ and twice continuously differentiable in $t$ on $[0, T]$. The nodal error $\mathbf{e}^j = \mathbf{C}(t^j) - \mathbf{C}^j$ satisfies $\mathbf{A}\,\mathbf{e}^{j+1} = \mathbf{e}^j + \Delta t\,\boldsymbol{\tau}^{j+1}$. By Eq. (46a), $\left\|\mathbf{e}^{j+1}\right\|_\infty \le \left\|\mathbf{e}^j\right\|_\infty + \Delta t\left\|\boldsymbol{\tau}^{j+1}\right\|_\infty$, and summing over the steps,
$$
\max_j\left\|\mathbf{e}^j\right\|_\infty \le \left\|\mathbf{e}^0\right\|_\infty + T\max_j\left\|\boldsymbol{\tau}^j\right\|_\infty = O\left(\Delta t + \left(\Delta r\right)^2\right), \tag{46c}
$$
because every row, including rows 0 and 1, is second order in space. The same argument on the five-layer grid gives only $O\left(\Delta t + \Delta r\right)$ in the maximum norm, because of the first-order interface residuals: proved, but pessimistic. Its hypotheses are those of the argument itself: on $[t_*, T]$ the solution is four times continuously differentiable in $r$ on each closed layer and twice continuously differentiable in $t$; the grid is piecewise uniform with the interfaces at nodes; and the initial error is bounded in the maximum norm, $\left\|\mathbf{e}(t_*)\right\|_\infty = O(\Delta r)$. A weighted $L^2$ bound on the initial error, as in the next paragraph, is not sufficient for this maximum-norm statement.

**Five layers, weighted $L^2$ norm, second order (PROOF OUTLINE, from derivation B, steps checked in the reconciliation).** Hypotheses: (a) on $[t_*, T]$, $t_* \ge 0$, the solution is four times continuously differentiable in $r$ on each closed layer and twice continuously differentiable in $t$; (b) the grid is piecewise uniform with the interfaces at nodes, with largest spacing $h$; (c) $\left\|\mathbf{e}(t_*)\right\|_V = O\left(\left(h\right)^2\right)$. Claim: $\max_j\left\|\mathbf{e}^j\right\|_V = O\left(\left(h\right)^2 + \Delta t\right)$ on $[t_*, T]$. Steps:

1. Multiply the error equation of row $i \ge 1$ by $V_i$. At the non-interface nodes the residual is $V_i\tau_i$ with $\left|\tau_i\right| \le c\left(\left(h\right)^2 + \Delta t\right)$. At the four interface nodes it is $\rho_i$, with $\left|\rho_i\right| \le c\left(r_i\right)^2\left(\left(h\right)^2 + h\,\Delta t\right)$: the $O\left(\left(h\right)^2\right)$ remainder of Section 5.5 before the division by $V_i$, plus $V_i$ times the time residual.
2. Multiply by $e_i^{j+1}$ and sum as in Eq. (46b). With $\left|\mathbf{e}\right|_K^2 = \sum_{i=0}^{N-1}K_{i,\mathrm{outer}}\left(e_{i+1} - e_i\right)^2$ and $e_N = 0$, and the Cauchy-Schwarz inequality on the first kind of residual,
   $$\frac{\left\|\mathbf{e}^{j+1}\right\|_V^2 - \left\|\mathbf{e}^j\right\|_V^2}{2\Delta t} + \left|\mathbf{e}^{j+1}\right|_K^2 \le c\left(\left(h\right)^2 + \Delta t\right)\left\|\mathbf{e}^{j+1}\right\|_V + \sum_{\text{interfaces}}\left|\rho_i\right|\left|e_i^{j+1}\right|.$$
3. A discrete Sobolev bound at an interface node $i \ge N_{\mathrm{kernel}}$: since $e_N = 0$, $\left|e_i\right| = \left|\sum_{k \ge i}\left(e_k - e_{k+1}\right)\right| \le \left|\mathbf{e}\right|_K\left(\sum_{k \ge i}1/K_{k,\mathrm{outer}}\right)^{1/2}$. For $k \ge N_{\mathrm{kernel}}$, $1/K_{k,\mathrm{outer}} = \Delta r_{k,\mathrm{outer}}/\left(D_{k,\mathrm{outer}}r_kr_{k+1}\right) \le \Delta r_{k,\mathrm{outer}}/\left(D_{\min}\left(R_1\right)^2\right)$, so the sum is at most $S = R_5/\left(D_{\min}\left(R_1\right)^2\right)$, independent of $h$. (The reconciliation noted that this bound needs no condition on the ratio of spacings.) The centre is not involved.
4. Young's inequality: $\sum\left|\rho_i\right|\left|e_i\right| \le \left|\mathbf{e}\right|_K^2 + \frac{S}{4}\left(\sum\left|\rho_i\right|\right)^2$, and the $\left|\mathbf{e}\right|_K^2$ term cancels against the left side.
5. Summing over the steps with $X = \max_j\left\|\mathbf{e}^j\right\|_V$: $\left(X\right)^2 \le \left\|\mathbf{e}^0\right\|_V^2 + 2Tc\left(\left(h\right)^2 + \Delta t\right)X + T\,O\left(\left(\left(h\right)^2 + h\,\Delta t\right)^2\right)$, so $X = O\left(\left(h\right)^2 + \Delta t\right)$.

The constants are not tracked, hence the label PROOF OUTLINE. It gives no pointwise second-order bound on the five-layer grid, and it does not cover the discontinuous start on $[0, T]$, because hypothesis (a) fails at $t = 0$.

**Discontinuous start (OBSERVED).** With the start (39) and $\theta$ from Eq. (40), both derivations and the reconciliation observed second order in space for the release fraction at fixed $t > 0$ and for the nodal profile once $\Delta r_{\mathrm{kernel}}$ is small compared with $\sqrt{D_{\mathrm{kernel}}\,t}$ (Section 7), and first order in time. The classical theory of parabolic discretisations with non-smooth data, mainly for finite elements, predicts errors that behave like $\left(\Delta r\right)^2/t$ and $\Delta t/t$; the observations agree with it qualitatively. The corresponding statement for this scheme is **NOT PROVEN**.

### 9.6 Summary of properties

| Property | Result | Status |
|---|---|---|
| Solvability, M-matrix, $\mathbf{A}^{-1} \ge 0$ | every $\Delta t > 0$, every grid with nodes at $0, R_1, \dots, R_5$, every positive diffusivity | PROVED (Section 9.2) |
| Stability | $\left\lVert \mathbf{C}^{j+1}\right\rVert _\infty \le \left\lVert \mathbf{C}^j\right\rVert _\infty$ and $\left\lVert \mathbf{C}^{j+1}\right\rVert _V \le \left\lVert \mathbf{C}^j\right\rVert _V$, every $\Delta t > 0$ | PROVED (Eqs. (46), (46a), (46b)) |
| Positivity and maximum principle | $0 \le C_i^{j+1} \le \max_k C_k^j$, every $\Delta t > 0$; also in floating point for the Thomas algorithm of Section 9.2 (vi) | PROVED |
| Thomas algorithm | pivots $m_i \ge 1 + \vert U_i\vert$, multipliers below one, no pivoting | PROVED (Section 9.3) |
| Interface continuity | concentration exact (one shared node value); reconstructed flux (30a) equal from both sides | PROVED (Section 5.4) |
| Interface flux formula (30a) | evaluated on the exact solution, equals the exact interface flow to $O\left(\left(\Delta r\right)^2 + \Delta r\,\Delta t\right)$ | PROVED (Section 5.4, consistency) |
| Computed interface flux (30a) | successive-grid differences fall at second order in space (self-convergence); convergence to the exact interface flow not shown | OBSERVED (Section 5.4); NOT PROVEN (no order proved) |
| Conservation | $M^j + M_{\mathrm{rel}}^j = M^0$ exactly in exact arithmetic, any grid, any $\Delta t$, any centre row | PROVED (Eqs. (21), (35), (37)) |
| Conservation in floating point | rounding residual, bounded rigorously by Document 2 (F44) and (F45) (and (F46) for the combined scheme); observed $10^{-13}$ to $1.2\times10^{-9}$ relative on the synthetic case with $\Delta t$ from $2$ to $200\ \mathrm{s}$ (Section 9.4), up to $5.0\times10^{-8}$ at largest $\mathrm{Fo}$ of order $10^7$ over 3000 steps; the residual scales with the step count times the largest $\mathrm{Fo}$ | PROVED bound (Document 5, Section 5); residual values OBSERVED (Section 9.4) |
| Consistency | all rows; local order 2, except 1 at interface nodes and spacing jumps | PROVED (Section 9.5) |
| Release formula | consistent to second order | PROVED (Eq. (36a)) |
| Computed release rate | second order | OBSERVED (Section 6) |
| Temporal order | 1; observed $0.998$ to $1.000$ ($F$ at $2\times10^5\ \mathrm{s}$, $\Delta t = 2000$ to $125\ \mathrm{s}$, 40 intervals per layer; reproduced in the reconciliation) | PROVED for smooth solutions (Eq. (46c), Section 9.5); OBSERVED for the discontinuous start |
| Spatial order, homogeneous sphere | 2, maximum norm | PROVED (Eq. (46c)) |
| Spatial order, five layers | 2, weighted $L^2$ norm | PROOF OUTLINE (Section 9.5) |
| Spatial order, release fraction, synthetic case | observed $1.99$ to $2.00$ ($m = 5$ to $160$; an earlier $2.05$ for the last refinement is attributed to eigen-solver rounding, Section 7, note) | OBSERVED (Section 7) |
| Spatial order, discontinuous start | 2 at fixed $t > 0$ | OBSERVED; NOT PROVEN |

---

## 10. The assembled tridiagonal system, row by row

This section collects the rows in the owner's $\mathrm{Fo}$ form, as a summary of the derivation. The normative statement, with every quantity defined and every equation numbered for the pseudocode and the Rust code, is [Document 2](02_final_formulation.md).

**Unknowns:** $C_0^{j+1}, C_1^{j+1}, \dots, C_{N-1}^{j+1}$. The boundary value $C_N^{j+1} = 0$ is not an unknown. **Right-hand side:** $C_0^j, \dots, C_{N-1}^j$. All $\mathrm{Fo}$ use the layer's own diffusivity and spacing, $\mathrm{Fo}_{\mathrm{layer}} = D_{\mathrm{layer}}\Delta t/\left(\Delta r_{\mathrm{layer}}\right)^2$.

**Row 0 (centre, Eq. (25)):**

$$
\left(1 + 6\,\mathrm{Fo}_{\mathrm{kernel}}\right)C_0^{j+1} - 6\,\mathrm{Fo}_{\mathrm{kernel}}\,C_1^{j+1} = C_0^j.
$$

**Rows $1 \le i \le N_{\mathrm{kernel}} - 1$ (kernel interior, Eq. (16)):**

$$
-\mathrm{Fo}_{\mathrm{kernel}}\left(1 - \frac{1}{i}\right)C_{i-1}^{j+1} + \left(1 + 2\,\mathrm{Fo}_{\mathrm{kernel}}\right)C_i^{j+1} - \mathrm{Fo}_{\mathrm{kernel}}\left(1 + \frac{1}{i}\right)C_{i+1}^{j+1} = C_i^j.
$$

At $i = 1$ the first coefficient is zero (Section 4.3).

**Row $N_{\mathrm{kernel}}$ (kernel-buffer interface at $R_1$, Eqs. (32a) and (32b)):**

$$
-\mathrm{Fo}_{\mathrm{kernel}}\frac{2\Delta r_{\mathrm{kernel}}}{\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}}\left(1 - \frac{\Delta r_{\mathrm{kernel}}}{R_1}\right)C_{i-1}^{j+1} + B_i\,C_i^{j+1} - \mathrm{Fo}_{\mathrm{buffer}}\frac{2\Delta r_{\mathrm{buffer}}}{\Delta r_{\mathrm{kernel}} + \Delta r_{\mathrm{buffer}}}\left(1 + \frac{\Delta r_{\mathrm{buffer}}}{R_1}\right)C_{i+1}^{j+1} = C_i^j,
$$

with $i = N_{\mathrm{kernel}}$ and $B_i = 1 - L_i - U_i$.

**Rows $N_{\mathrm{kernel}} < i < N_{\mathrm{buffer}}$ (buffer interior, Eq. (15)):**

$$
-\mathrm{Fo}_{\mathrm{buffer}}\left(1 - \frac{\Delta r_{\mathrm{buffer}}}{r_i}\right)C_{i-1}^{j+1} + \left(1 + 2\,\mathrm{Fo}_{\mathrm{buffer}}\right)C_i^{j+1} - \mathrm{Fo}_{\mathrm{buffer}}\left(1 + \frac{\Delta r_{\mathrm{buffer}}}{r_i}\right)C_{i+1}^{j+1} = C_i^j, \qquad r_i = R_1 + (i - N_{\mathrm{kernel}})\Delta r_{\mathrm{buffer}}.
$$

**Row $N_{\mathrm{buffer}}$ (buffer-IPyC interface at $R_2$):** as the row at $R_1$, with (kernel, buffer, $R_1$) replaced by (buffer, IPyC, $R_2$).

**Rows $N_{\mathrm{buffer}} < i < N_{\mathrm{IPyC}}$ (IPyC interior):** as the buffer interior rows, with IPyC values and $r_i = R_2 + (i - N_{\mathrm{buffer}})\Delta r_{\mathrm{IPyC}}$.

**Row $N_{\mathrm{IPyC}}$ (IPyC-SiC interface at $R_3$):** with (IPyC, SiC, $R_3$).

**Rows $N_{\mathrm{IPyC}} < i < N_{\mathrm{SiC}}$ (SiC interior):** with SiC values and $r_i = R_3 + (i - N_{\mathrm{IPyC}})\Delta r_{\mathrm{SiC}}$.

**Row $N_{\mathrm{SiC}}$ (SiC-OPyC interface at $R_4$):** with (SiC, OPyC, $R_4$).

**Rows $N_{\mathrm{SiC}} < i \le N - 2$ (OPyC interior):** with OPyC values and $r_i = R_4 + (i - N_{\mathrm{SiC}})\Delta r_{\mathrm{OPyC}}$.

**Row $N-1$ (last unknown, next to the absorbing surface, Eq. (34b)):**

$$
-\mathrm{Fo}_{\mathrm{OPyC}}\left(1 - \frac{\Delta r_{\mathrm{OPyC}}}{r_{N-1}}\right)C_{N-2}^{j+1} + \left(1 + 2\,\mathrm{Fo}_{\mathrm{OPyC}}\right)C_{N-1}^{j+1} = C_{N-1}^j.
$$

If the OPyC has only one interval, row $N-1$ is the interface row at $R_4$ with its $C_N^{j+1}$ term removed. Its diagonal keeps the full value $1 - L_{N-1} - U_{N-1}$ in either case.

**Row $N$:** none, because $C_N^{j+1} = 0$ (Eq. (34a)).

**The matrix.**

$$
\begin{pmatrix}
B_0 & U_0 & & & & \\
0 & B_1 & U_1 & & & \\
 & L_2 & B_2 & U_2 & & \\
 & & \ddots & \ddots & \ddots & \\
 & & & L_{N-2} & B_{N-2} & U_{N-2} \\
 & & & & L_{N-1} & B_{N-1}
\end{pmatrix}
\begin{pmatrix} C_0^{j+1} \\ C_1^{j+1} \\ C_2^{j+1} \\ \vdots \\ C_{N-2}^{j+1} \\ C_{N-1}^{j+1} \end{pmatrix} =
\begin{pmatrix} C_0^{j} \\ C_1^{j} \\ C_2^{j} \\ \vdots \\ C_{N-2}^{j} \\ C_{N-1}^{j} \end{pmatrix}. \tag{47}
$$

The zero in position $(1,0)$ is $L_1 = 0$ (Section 4.3).

**After each solve:**

- the release rate $\Phi^{j+1} = 4\pi D_{\mathrm{OPyC}}\,r_{N-1}R_5\,C_{N-1}^{j+1}/\Delta r_{\mathrm{OPyC}}$ (Eq. (36));
- the inventory $M^{j+1} = 4\pi\sum_{i=1}^{N-1}V_i C_i^{j+1}$, with $V_i = \left(r_i\right)^2\Delta r_{\mathrm{layer}}$ inside a layer and $V_i = \left(R\right)^2\left(\Delta r_{\mathrm{in}} + \Delta r_{\mathrm{out}}\right)/2$ at an interface (Eqs. (29) and (34c));
- the cumulative release $M_{\mathrm{rel}}^{j+1} = M_{\mathrm{rel}}^j + \Delta t\,\Phi^{j+1}$;
- the balance residual $M^{j+1} + M_{\mathrm{rel}}^{j+1} - M^0$, which is zero in exact arithmetic;
- the release fraction $F^{j+1} = M_{\mathrm{rel}}^{j+1}/M^0$.

**Initial vector:** Eq. (39), with $\theta$ from Eq. (40).

---

## 11. Scratch experiments and open items

**Scratch experiments.** All numbers quoted in this document come from short uncommitted scratch programs, written only to test the algebra above. They solved the semi-discrete system exactly in time, by an eigen-decomposition of the symmetrised operator, or ran backward Euler with a plain Thomas algorithm. They used the synthetic five-layer input of `inputs/synthetic/synthetic_five_layer.toml`, which states that it is not literature data, and the homogeneous-sphere series solution. Three independent scratch programs were used: one for derivation A, one for derivation B (cited as "archive"), and one written for the reconciliation that implements Document 2 row by row. They are numerical checks of the derivation. They are not part of the repository, not a verification of the future Rust solver, and not experimental validation.

| Check | Result |
|---|---|
| Hypothesis (1), identity (18), random data | agreement to $1.4\times10^{-14}$ |
| Hypothesis (2), $L_1 = 0$ | exact by Eq. (16); one-way coupling confirmed structurally; column 0 of $\mathbf{A}^{-1}$ zero below the diagonal (archive) |
| Hypothesis (3), ghost (33a) vs. balance (33b) at unequal spacing | balance residual $3.15\times10^{-4}$ vs. rounding level, obtained independently by A and B |
| Hypothesis (4), positivity | no negative value in any run; proved in Section 9.2 |
| Hypothesis (5), $\theta$ | on unequal spacing only Eq. (40)/(41) gives order 2; on equal spacing $\theta = \frac{1}{2}$ is also second order (Section 7) |
| Spatial order, $F(2\times10^5\ \mathrm{s})$ vs. reference | $1.988$, $1.997$, $1.999$, $1.997$, and $2.050$ for the last refinement, which is attributed to eigen-solver rounding (Section 7, note); backward Euler with Richardson extrapolation in $\Delta t$ gives $2.00$ for that refinement. Reconciliation: $2.00$, $2.00$, $2.00$, $2.01$ |
| Computed interface flux (30a), $t = 2\times10^5\ \mathrm{s}$, self-convergence $m = 10$ to $320$ | observed orders $1.98$ to $2.00$ at all four interfaces (Section 5.4) |
| Temporal order, BE | $0.998$ to $1.000$; reproduced in the reconciliation |
| Release rate, homogeneous sphere | order $2.00$ to $2.03$ with exact $M^0$; reproduced in the reconciliation |
| $F(2\times10^5\ \mathrm{s})$, 40 intervals per layer, $\Delta t = 200\ \mathrm{s}$ | $0.43242910455$ (archive) and $0.432429104571$ (reconciliation), equal within the rounding residual of Section 9.4 |

**Open items for the following phases.**

1. Directive A7 asks for an independent review. The reconciliation compared two independent derivations and checked each difference; the coordinator decides whether that satisfies A7 or whether Document 2 needs a further review before Phase C.
2. Floating-point conservation (Section 9.4): done in [Document 5](05_generation_decay_temperature.md), Section 5; the tolerance is Document 2 (F44).
3. Grid counts per layer for production runs: a convergence study on the AGR-1 geometry with literature diffusivities (Phase D), including whether grading towards $R_1$ helps at early times.
4. A proof of second-order convergence of the computed release rate (Section 6), of the computed interface flux (30a) (Section 5.4), and of the discontinuous start (Section 9.5), are open.
5. Phase G hooks (Document 2, Section 9, gives the places; none is derived here). Now derived in [Document 5](05_generation_decay_temperature.md) and stated in Document 2, Section 11; Document 5 adopts the half-interval split and the surface release correction of derivation B, and adds the factor $\omega$ at $R_1$. The original list:
   - decay adds $\lambda\,\Delta t$ to every $B_i$, with the decayed amount $4\pi\lambda\,\Delta t\sum V_iC_i^{j+1}$ booked separately from the release;
   - generation adds $\Delta t\,G_i$ to the right-hand side, with the generated amount $4\pi\sum V_i\,\Delta t\,G_i$. Derivation B found two points that the G1 derivation must settle: a source that differs between two materials needs the half-interval split of $V_i$ at an interface node, and a source that is non-zero at $R_5$ breaks the cancellation in Eq. (36a), so the release then needs the generation in the outer half-interval of node $N$ (archive, Section 9). These findings are recorded, not adopted;
   - temperature changes $D_{i,\mathrm{inner}}$ and $D_{i,\mathrm{outer}}$ and triggers reassembly, with the same values used in the release.

   Each needs its own short derivation before implementation. None is claimed here.
