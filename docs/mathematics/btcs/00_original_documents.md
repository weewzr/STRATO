# STRATO: The Owner's Original FTCS Documents, an Account and an Algebra Check

**Status.** Working document for the BTCS reconstruction of STRATO (directive Phase A). It gives a faithful account of the owner's two original documents and checks every algebra step in them. It does not change them. Where a step is wrong or an index is off, this document says so, shows the correct step and gives the reason. The continuation into the five-layer BTCS scheme is in [01_from_ftcs_to_five_layer_btcs.md](01_from_ftcs_to_five_layer_btcs.md).

**The two documents.** The owner provided two documents. They are not stored in the repository. This document does not reproduce the names, affiliations or other personal details printed in them.

- **Document T** (typed manuscript, 11 pages, dated June 2026). It is titled "A Spherical Finite-Difference Model of Fission-Product Diffusion in TRISO Fuel Kernels for Fluoride-Salt-Cooled High-Temperature Reactors". It has an abstract, a nomenclature table, Sections 1 to 8 and an empty reference list. Citations in the text appear as unresolved placeholders "[?]". Its equations are cited here as (T1) to (T30), with the manuscript's own numbers.
- **Document H** (handwritten notes, 10 pages; page 6 is blank). These notes are the derivation from which Document T was written. They include annotations in blue and red ink, two sketches, a question the owner wrote to himself, and a to-do list. Their equations carry no numbers in the original. They are cited here as (H1), (H2), and so on, in page order.

**How to read the checks.** Every transcribed equation is followed by a verdict:

- **Correct**: the step follows from the previous one.
- **Index slip**: a node or time index is wrong, but the intended equation is clear. The corrected form is given.
- **Error**: the step is mathematically wrong. The correct step and the reason are given.
- **Wording**: the equation is right, but the sentence that justifies it is not.

**Notation in this account.** Directive Section 5 is binding here as well:

- $i$ is the radial node index and $j$ the time-step index.
- $C_i^j$ is the concentration at node $i$ and time step $j$.
- $r_i$ is the radius of node $i$, $\Delta r$ the uniform spacing and $\Delta t$ the time step.
- A power encloses the whole variable in parentheses, for example $\left(\Delta r\right)^2$.

Both documents already follow this notation closely. Document H writes the continuous concentration as a lower-case $c$, and so does Document T. This account keeps the owner's lower-case $c(r,t)$ when it quotes the owner, and uses the same symbols as the owner everywhere else.

| Symbol | Meaning | Unit |
|---|---|---|
| $c(r,t)$ | concentration of the diffusing species (owner's continuous variable) | $\mathrm{mol\ m^{-3}}$ |
| $C_i^j$ | discrete concentration at node $i$, time step $j$ | $\mathrm{mol\ m^{-3}}$ |
| $r$ | radial coordinate | $\mathrm{m}$ |
| $R$ | kernel (outer) radius in the original single-region model | $\mathrm{m}$ |
| $r_i$ | radius of node $i$ | $\mathrm{m}$ |
| $\Delta r$ | uniform radial spacing | $\mathrm{m}$ |
| $t$ | time | $\mathrm{s}$ |
| $\Delta t$ | time step | $\mathrm{s}$ |
| $D$ | diffusion coefficient (diffusivity) | $\mathrm{m^2\ s^{-1}}$ |
| $\mathrm{Fo} = D\,\Delta t/\left(\Delta r\right)^2$ | mesh Fourier number | dimensionless |
| $\Sigma_f$ | macroscopic fission cross section | $\mathrm{m^{-1}}$ |
| $\phi$ | neutron flux | $\mathrm{m^{-2}\ s^{-1}}$ |
| $Y$ (written $y$ in Document H) | fission-product yield | atoms per fission |
| $\lambda$ | radioactive decay constant, $\ln 2/t_{1/2}$ | $\mathrm{s^{-1}}$ |
| $\kappa$ | grain-boundary trapping rate | $\mathrm{s^{-1}}$ |
| $D_0$, $Q$, $R_g$, $T$ | Arrhenius pre-factor, activation energy, gas constant, temperature | $\mathrm{m^2\ s^{-1}}$, $\mathrm{J\ mol^{-1}}$, $\mathrm{J\ mol^{-1}\ K^{-1}}$, $\mathrm{K}$ |

**Two node numberings.** The two documents number the nodes differently, and most index slips come from mixing the two.

- Document H numbers five nodes from $i = 1$ (centre) to $i = 5$ (surface), so $r_i = (i-1)\,\Delta r$.
- Document T numbers them from $i = 0$ (centre) to $i = 4$ (surface), so $r_i = i\,\Delta r$.

The factor $1/i$ that both documents use in the curvature term is exact only in the second numbering.

---

## 1. Document H, page by page

### Page 1: governing equation and reduction to one dimension

The heading describes a TRISO particle as a spherical capsule in a fluoride-salt-cooled high-temperature reactor (FHR). The notes start from Fick's second law in three dimensions,

$$
\frac{\partial c}{\partial t} = D\,\nabla^2 c, \tag{H1}
$$

and expand the Laplacian in spherical coordinates $(r,\theta,\phi)$:

$$
\frac{\partial c}{\partial t} = D\left[\frac{1}{\left(r\right)^2}\frac{\partial}{\partial r}\left(\left(r\right)^2\frac{\partial c}{\partial r}\right) + \frac{1}{\left(r\right)^2\sin\theta}\frac{\partial}{\partial\theta}\left(\sin\theta\,\frac{\partial c}{\partial\theta}\right) + \frac{1}{\left(r\right)^2\left(\sin\theta\right)^2}\frac{\partial^2 c}{\partial\phi^2}\right]. \tag{H2}
$$

**Correct.** This is the standard spherical Laplacian. Eq. (H1) assumes that $D$ is the same everywhere. That is true inside one material and false across a TRISO coating interface, which is the first point at which the five-layer model must start earlier (document 01, Section 1).

Two assumptions follow, transcribed as written:

1. "Triso fuel is uniform throughout (reduces dependency on $\theta$, $\phi$)."
2. "Concentration of Triso fuel at any position is equals to the concentration at the surface (reduces the convection term)."

**Assumption 1: correct, and sufficient on its own.** If the material, the source, the initial profile and the boundary data are the same in every direction, the solution has no angular dependence. Then both angular terms in (H2) are zero.

**Assumption 2: Error (as worded).** Taken literally, it would make $c$ equal to its surface value everywhere. That would leave nothing to diffuse and would contradict the rest of the notes. Eq. (H2) also contains no convection term; every term in it is diffusive. The reduction to one dimension needs only Assumption 1. What Assumption 2 seems to be reaching for is a boundary condition at the surface, a separate statement. Document T rewords it as a "surface-coupled boundary" (Section 2.2 there), but it is still not needed for the reduction. STRATO does not use Assumption 2. Its boundary condition is the absorbing condition $C(R_5,t) = 0$ (document 01, Section 6).

The reduced equation and its product-rule expansion:

$$
\frac{\partial c}{\partial t} = D\left[\frac{1}{\left(r\right)^2}\frac{\partial}{\partial r}\left(\left(r\right)^2\frac{\partial c}{\partial r}\right)\right], \tag{H3}
$$

$$
\frac{1}{\left(r\right)^2}\left[\frac{\partial}{\partial r}\left(\left(r\right)^2\frac{\partial c}{\partial r}\right)\right] = \frac{1}{\left(r\right)^2}\left[2r\,\frac{\partial c}{\partial r} + \left(r\right)^2\frac{\partial^2 c}{\partial r^2}\right] = \frac{2}{r}\frac{\partial c}{\partial r} + \frac{\partial^2 c}{\partial r^2}, \tag{H4}
$$

$$
\frac{\partial c}{\partial t} = D\left(\frac{\partial^2 c}{\partial r^2} + \frac{2}{r}\frac{\partial c}{\partial r}\right). \tag{H5}
$$

**Correct.** In (H4), $\frac{\partial}{\partial r}\left(r\right)^2 = 2r$, and dividing each term by $\left(r\right)^2$ gives the result.

The notes name the method, "FTCS (Forward Time Centred Space)", and discretise the right-hand side with central differences:

$$
\frac{\partial^2 c}{\partial r^2} = \frac{C_{i-1}^j - 2C_i^j + C_{i+1}^j}{\left(\Delta r\right)^2}, \qquad \frac{\partial c}{\partial r} = \frac{C_{i+1}^j - C_{i-1}^j}{2\,\Delta r}. \tag{H6}
$$

**Correct as approximations.** The "$=$" should be "$\approx$". By Taylor's theorem the two quotients equal the derivatives plus $\frac{\left(\Delta r\right)^2}{12}\frac{\partial^4 c}{\partial r^4}$ and $\frac{\left(\Delta r\right)^2}{6}\frac{\partial^3 c}{\partial r^3}$ respectively, up to higher-order terms. Both are second order (document 01, Eqs. (9) and (10)).

### Page 2: forward Euler, the explicit update and the centre node

Forward Euler in time and the "final discretised equation":

$$
\frac{\partial c}{\partial t} = \frac{C_i^{j+1} - C_i^j}{\Delta t}, \tag{H7}
$$

$$
\frac{C_i^{j+1} - C_i^j}{\Delta t} = D\left(\frac{C_{i+1}^j - 2C_i^j + C_{i-1}^j}{\left(\Delta r\right)^2} + \frac{2}{r}\,\frac{C_{i+1}^j - C_{i-1}^j}{2\,\Delta r}\right). \tag{H8}
$$

**Correct.** Strictly, the $r$ in (H8) is the radius $r_i$ of node $i$, and Document T writes it so (T9).

"Making $C_i^{j+1}$ the subject":

$$
C_i^{j+1} = C_i^j + \frac{D\,\Delta t}{\left(\Delta r\right)^2}\left(C_{i+1}^j - 2C_i^j + C_{i-1}^j\right) + \frac{D\,\Delta t}{r\,\Delta r}\left(C_{i+1}^j - C_{i-1}^j\right) \tag{H9}
$$

$$
= C_i^j\left(1 - 2\frac{D\,\Delta t}{\left(\Delta r\right)^2}\right) + C_{i+1}^j\left(\frac{D\,\Delta t}{\left(\Delta r\right)^2} + \frac{D\,\Delta t}{\left(\Delta r\right)^2}\cdot\frac{1}{i}\right) + C_{j-1}^{i}\left(\frac{D\,\Delta t}{\left(\Delta r\right)^2} - \frac{D\,\Delta t}{\left(\Delta r\right)^2}\cdot\frac{1}{i}\right). \tag{H10}
$$

Check of (H9): multiply (H8) by $\Delta t$ and add $C_i^j$. The curvature term is $D\,\Delta t\,\frac{2}{r}\,\frac{1}{2\Delta r} = \frac{D\,\Delta t}{r\,\Delta r}$. **Correct.**

Check of (H10): the step from $\frac{D\,\Delta t}{r\,\Delta r}$ to $\frac{D\,\Delta t}{\left(\Delta r\right)^2}\cdot\frac{1}{i}$ uses $r = i\,\Delta r$. That holds only when the centre is node $0$. Two findings:

- **Index slip.** The last term is written $C_{j-1}^{i}$ in the notes: the time and space indices are swapped. It must be $C_{i-1}^j$.
- **Index slip (numbering).** A few lines further down the same page, the notes number the nodes $i = 1$ (centre) to $i = 5$. In that numbering $r_i = (i-1)\,\Delta r$ and the factor is $1/(i-1)$, not $1/i$. Page 4 shows the consequence. Document T removes the conflict by numbering from $0$.

The corrected collected form, valid for $r_i = i\,\Delta r$:

$$
C_i^{j+1} = C_i^j\left(1 - 2\,\mathrm{Fo}\right) + C_{i+1}^j\,\mathrm{Fo}\left(1 + \frac{1}{i}\right) + C_{i-1}^j\,\mathrm{Fo}\left(1 - \frac{1}{i}\right), \qquad \mathrm{Fo} = \frac{D\,\Delta t}{\left(\Delta r\right)^2}. \tag{C1}
$$

The notes then state an "initial condition":

$$
\frac{\partial c}{\partial t} = 0. \tag{H11}
$$

**Error.** (H11) is not an initial condition. An initial-value problem for (H5) needs the initial profile $c(r,0)$. The time derivative at $t = 0$ is not free: the equation itself fixes it as $D\left(\frac{\partial^2 c}{\partial r^2} + \frac{2}{r}\frac{\partial c}{\partial r}\right)$ evaluated on the initial profile. So (H11) holds only for special profiles, such as a uniform profile in an insulated sphere. For STRATO's kernel-only start, the initial profile jumps at the kernel surface. There $\partial c/\partial t$ at $t = 0$ is not zero, and in fact not even finite. Document 01, Section 7 replaces (H11) with the initial profile.

"Boundary conditions: assume $i = 1$ to $i = 5$ (4 shells, 5 nodes)." A sketch shows five concentric rings labelled 1 to 5 from the centre outwards, with a dashed radius through them. Node 1 is at the centre.

**Centre node** ($i = 1$ in this numbering): symmetry gives zero slope, and a ghost node $C_0$ is introduced:

$$
\left.\frac{\partial c}{\partial r}\right|_{0} = 0, \qquad \frac{\partial c}{\partial r} = \frac{C_2 - C_0}{2\,\Delta r}, \qquad 0 = \frac{C_2 - C_0}{2\,\Delta r}, \qquad C_0 = C_2, \tag{H12}
$$

$$
\frac{\partial^2 c}{\partial r^2} = \frac{C_2 - 2C_1 + C_0}{\left(\Delta r\right)^2} = \frac{C_2 - 2C_1 + C_2}{\left(\Delta r\right)^2} = \frac{2C_2 - 2C_1}{\left(\Delta r\right)^2}. \tag{H13}
$$

**Correct**, in the page's own numbering. The ghost value $C_0$ sits at $r = -\Delta r$, the mirror image of node 2. The central difference about the centre node 1 is $(C_2 - C_0)/(2\Delta r)$, and setting it to zero gives $C_0 = C_2$.

### Page 3: L'Hôpital's rule at the centre, the owner's question, and the outer node

$$
\frac{\partial^2 c}{\partial r^2} + \frac{2}{r}\frac{\partial c}{\partial r} = \frac{\partial^2 c}{\partial r^2} + \lim_{r\to0}\frac{2\,\frac{\partial c}{\partial r}}{r} = \frac{\partial^2 c}{\partial r^2} + 2\,\frac{\partial^2 c}{\partial r^2} = 3\,\frac{\partial^2 c}{\partial r^2}. \tag{H14}
$$

**Correct.** At $r = 0$ both the numerator $\partial c/\partial r$ and the denominator $r$ tend to zero, a $0/0$ form. L'Hôpital's rule replaces them by their derivatives: $\lim_{r\to0}\frac{\partial c/\partial r}{r} = \lim_{r\to0}\frac{\partial^2 c/\partial r^2}{1}$. Document 01, Section 4 confirms this with a Taylor series.

Next to (H14) the owner wrote, in blue:

> **Q1.** "Should this be final equation to be discretised or should I use previous form"

The answer is in Section 3 of this document and in document 01, Section 4. In short: at $r = 0$ the "previous form" cannot be evaluated, because $2/r$ is undefined there. So $3\,\partial^2 c/\partial r^2$ is the equation for the centre node. At every node with $r_i > 0$ the previous form is used.

The centre update as written:

$$
\frac{\partial c}{\partial t} = D\left(3\,\frac{\partial^2 c}{\partial r^2}\right), \qquad \frac{C_{i+1}^j - C_i^j}{\Delta t} = D(3)\left(\frac{2C_1 - 2C_0}{\left(\Delta r\right)^2}\right), \qquad C_i^{j+1} = C_i^j + 6\,D\,\Delta t\left(\frac{C_1 - C_0}{\left(\Delta r\right)^2}\right). \tag{H15}
$$

The last expression is labelled "Concentration @ Node 0 at next timestep". Three findings:

- **Index slip (time difference).** The middle expression has $(C_{i+1}^j - C_i^j)/\Delta t$, a difference between two neighbouring nodes at the same time. A forward time difference compares the same node at two time steps: $(C_i^{j+1} - C_i^j)/\Delta t$. The final expression in (H15) does use $C_i^{j+1}$ and $C_i^j$, so this slip does not reach the result.
- **Index slip (numbering).** Page 2 put the centre at node 1 and the ghost at node 0, so (H13) reads $(2C_2 - 2C_1)/\left(\Delta r\right)^2$. On page 3 the same quantity is written $(2C_1 - 2C_0)/\left(\Delta r\right)^2$, and the result is labelled "Node 0". The notes have switched to centre-at-$0$ numbering in mid-derivation.
- **Factor 6: correct.** $3 \times 2 = 6$: the three comes from (H14) and the two from the collapsed stencil (H13).

The two consistent forms of the centre update are:

$$
\text{centre at } i = 1:\quad C_1^{j+1} = C_1^j + 6\,D\,\Delta t\,\frac{C_2^j - C_1^j}{\left(\Delta r\right)^2}; \qquad \text{centre at } i = 0:\quad C_0^{j+1} = C_0^j + 6\,D\,\Delta t\,\frac{C_1^j - C_0^j}{\left(\Delta r\right)^2}. \tag{C2}
$$

Document T uses the second form, (T20).

**Outer node** ($i = 5$, "insulation"):

$$
\left.\frac{\partial c}{\partial r}\right|_{0} = 0, \qquad \frac{\partial c}{\partial r} = \frac{C_6 - C_4}{2\,\Delta r}, \qquad C_6 = C_4, \qquad \frac{\partial^2 c}{\partial r^2} = \frac{C_6 - 2C_5 + C_4}{\left(\Delta r\right)^2} = \frac{2C_4 - 2C_5}{\left(\Delta r\right)^2}, \tag{H16}
$$

$$
\frac{\partial c}{\partial t} = D\left(3\,\frac{\partial^2 c}{\partial r^2}\right), \qquad \frac{C_{i+1}^j - C_i^j}{\Delta t} = D(3)\left(\frac{2C_4 - 2C_5}{\left(\Delta r\right)^2}\right), \qquad C_i^{j+1} = C_i^j + 6\,D\,\Delta t\left(\frac{C_4 - C_5}{\left(\Delta r\right)^2}\right). \tag{H17}
$$

The last expression is labelled "Concentration @ Node 4 at next timestep".

- **Index slip.** In (H16), the evaluation point of the zero-slope condition is written "$|_0$". It must be $|_{r=R}$, the surface.
- **Correct.** The ghost node $C_6 = C_4$ and the collapsed stencil $(2C_4 - 2C_5)/\left(\Delta r\right)^2$ follow correctly.
- **Error.** The factor 3 is taken over from the centre. The limit (H14) exists only because $r \to 0$. At the surface $r = R > 0$, so $2/R$ is an ordinary finite number. With the insulated condition the curvature term is $\frac{2}{R}\cdot 0 = 0$, and the equation at the surface is $\partial c/\partial t = D\,\partial^2 c/\partial r^2$. The factor is therefore $2$, not $6$.
- **Index slip.** The time difference again has $C_{i+1}^j$ in place of $C_i^{j+1}$. The result is labelled "Node 4" although the derivation is for node 5.

The corrected insulated-surface update, in centre-at-$1$ numbering, is:

$$
C_5^{j+1} = C_5^j + 2\,D\,\Delta t\,\frac{C_4^j - C_5^j}{\left(\Delta r\right)^2}. \tag{C3}
$$

Document T makes this correction itself (T23) and explains it in its Section 4.3. STRATO does not use this boundary at all. Its outer boundary is absorbing (document 01, Section 6).

### Page 4: the interior nodes

"For concentration @ Node 2, 3, 4 at next timestep":

- Node 2: $\frac{1}{i} = \frac{1}{2}$ (blue note: "Use for loop in code").
- Node 3: $\frac{1}{i} = \frac{1}{3}$.
- Node 4: $\frac{1}{i} = \frac{1}{4}$.

$$
C_i^{j+1} = C_i^j\left(1 - 2\frac{D\,\Delta t}{\left(\Delta r\right)^2}\right) + C_{i+1}^j\left(\frac{D\,\Delta t}{\left(\Delta r\right)^2} + \frac{D\,\Delta t}{\left(\Delta r\right)^2}\cdot\frac{1}{i}\right) + C_{i-1}^j\left(\frac{D\,\Delta t}{\left(\Delta r\right)^2} - \frac{D\,\Delta t}{\left(\Delta r\right)^2}\cdot\frac{1}{i}\right), \tag{H18}
$$

with $1/i = 1/2$, $1/3$ and $1/4$ for nodes 2, 3 and 4.

**Index slip (numbering).** In the page-2 numbering (centre at node 1), node 2 lies at $r = \Delta r$. Its curvature weight is $\Delta r/r_2 = 1$, not $1/2$. In general the weight is $\Delta r/r_i = 1/(i-1)$, so the correct weights for nodes 2, 3, 4 are $1$, $1/2$ and $1/3$. Document T gets the same values by renumbering from zero: interior nodes $1, 2, 3$ with $1/i = 1, 1/2, 1/3$ (T11 and its Table 1). **The weights in Document T are correct; those on page 4 are each one step too small.**

Blue notes: "Plot the concentration from $i=1$ to $i=5$ at each timestep. Layer each graph together."

### Page 5: the regrouped equation, base cases and source terms

$$
\frac{\partial c}{\partial t} = D\left[\frac{C_{i-1} - 2C_i + C_{i+1}}{\left(\Delta r\right)^2} + \frac{2}{r}\,\frac{C_{i+1} - C_{i-1}}{2\,\Delta r}\right] = D\left[\frac{r\left(C_{i-1} - 2C_i + C_{i+1}\right) + \Delta r\left(C_{i+1} - C_{i-1}\right)}{r\left(\Delta r\right)^2}\right], \tag{H19}
$$

$$
\boxed{\frac{\partial c}{\partial t} = D\left[\frac{C_{i-1}\left(r - \Delta r\right) + C_i\left(-2r\right) + C_{i+1}\left(r + \Delta r\right)}{r\left(\Delta r\right)^2}\right]} \tag{H20}
$$

The page calls (H20) the "Final Triso Equation for simple diffusion".

**Correct.** In (H19), $\frac{2}{r}\cdot\frac{1}{2\Delta r} = \frac{1}{r\,\Delta r} = \frac{\Delta r}{r\left(\Delta r\right)^2}$, and the common denominator is $r\left(\Delta r\right)^2$. Collecting by node gives (H20): $C_{i-1}$ has $r - \Delta r$, $C_i$ has $-2r$, and $C_{i+1}$ has $r + \Delta r$.

(H20) is a semi-discrete equation: space is discretised and time is still continuous. It is the most important equation in the two documents for what follows. With $r = r_i$ on a uniform grid, $r_i - \Delta r = r_{i-1}$ and $r_i + \Delta r = r_{i+1}$. Multiplying (H20) by $\left(r_i\right)^2\Delta r$ turns it into a difference of two flows, $D\,r_i r_{i+1}(C_{i+1} - C_i)/\Delta r - D\,r_{i-1} r_i (C_i - C_{i-1})/\Delta r$, the basis of discrete conservation (document 01, Section 3.5).

Blue notes: "Convert into code to plot", "Base case 1: Diffusivity $D$ is constant", "Base case 2: Diffusivity $D$ varies with Temperature $T$."

Red notes, "Possible source terms to add":

1. Fission product: $\Sigma_f\,\phi\,y$, where $\Sigma_f$ is the macroscopic fission cross section, $\phi$ the neutron flux at $r$, and $y$ the fission-product yield.
2. Radioactive decay: $-\lambda C$ with $\lambda = \ln 2/t_{1/2}$.
3. Grain-boundary trapping: $-\kappa C$, where $\kappa$ is the "net rate of fission products being trapped & unable to diffuse".

"Additional considerations":

1. $D$ is also different for each layer of TRISO fuel for given components.
2. $D$ changes with irradiation damage over time.

**Correct as a list of model terms.** On units, see the check of (T25) in Section 2. Item 1 of "Additional considerations" is exactly the extension STRATO makes (five layers). Of the source terms, the directive approves generation and decay; trapping is outside the approved scope.

### Page 6

Blank.

### Pages 7 and 8: an attempt at an implicit form by making $C_{i+1}$ the subject

Page 7 starts from the boxed (H20) and rearranges it for $C_{i+1}$:

$$
\frac{dc}{dt}\cdot\frac{r\left(\Delta r\right)^2}{D} = C_{i-1}\left(r - \Delta r\right) + C_i\left(-2r\right) + C_{i+1}\left(r + \Delta r\right), \tag{H21}
$$

$$
\frac{\frac{dc}{dt}\cdot\frac{r\left(\Delta r\right)^2}{D} - C_{i-1}\left(r - \Delta r\right) - C_i\left(-2r\right)}{r + \Delta r} = C_{i+1}. \tag{H22}
$$

**Correct** as algebra.

The notes then write two candidate time derivatives. The first is crossed out with a large X:

$$
\frac{dc}{dt} = \frac{C_{i+1} - C_{i-1}}{2\,\Delta t}\quad(\text{crossed out}), \qquad \frac{dc}{dt} = \frac{C_{i+1} - C_i}{\Delta t}. \tag{H23}
$$

**Error.** Both expressions difference two different spatial nodes and divide by a time step. A time derivative at node $i$ must compare the same node at two time steps, for example $(C_i^{j+1} - C_i^j)/\Delta t$ (backward Euler, evaluated at $t^{j+1}$) or $(C_i^{j+1} - C_i^{j-1})/(2\Delta t)$ (centred in time). The subscript must stay $i$; only the superscript changes. This is the same slip as in (H15) and (H17), here at the heart of the method. The notes use no time superscripts on pages 7 and 8, so the time level of each value is undefined.

The rest of page 7 and page 8 substitute the crossed-out form anyway:

$$
\frac{C_{i+1} - C_{i-1}}{2\,\Delta t} = D\,\frac{C_{i-1}\left(r - \Delta r\right)}{r\left(\Delta r\right)^2} + D\,\frac{C_i\left(-2r\right)}{r\left(\Delta r\right)^2} + D\,\frac{C_{i+1}\left(r + \Delta r\right)}{r\left(\Delta r\right)^2}, \tag{H24}
$$

$$
C_{i+1}\left(\frac{1}{2\,\Delta t} - \frac{D\left(r + \Delta r\right)}{r\left(\Delta r\right)^2}\right) = C_{i-1}\left(\frac{1}{2\,\Delta t} + \frac{D\left(r - \Delta r\right)}{r\left(\Delta r\right)^2}\right) + C_i\left(\frac{D\left(-2r\right)}{r\left(\Delta r\right)^2}\right), \tag{H25}
$$

$$
C_{j+1} = \frac{1}{\frac{1}{2\Delta t} - \frac{D\left(r + \Delta r\right)}{r\left(\Delta r\right)^2}}\left[C_{i-1}\left(\frac{1}{2\Delta t} + \frac{D\left(r - \Delta r\right)}{r\left(\Delta r\right)^2}\right)\ \ C_i\left(\frac{D\left(-2r\right)}{r\left(\Delta r\right)^2}\right)\right]. \tag{H26}
$$

- **Correct** as algebra, given (H24). Moving the $C_{i+1}$ term to the left and the $-C_{i-1}/(2\Delta t)$ term to the right gives (H25), and dividing gives (H26).
- **Index slip.** In (H26) the left side is written $C_{j+1}$; it means $C_{i+1}$.
- **Index slip.** A "$+$" is missing between the two terms in the square bracket.
- **Error (premise).** (H24) rests on the wrong time difference (H23), so (H25) and (H26) do not discretise the diffusion equation.

**Error (method).** Suppose the time difference is corrected. Solving each row for $C_{i+1}^{j+1}$ and marching outwards from the centre is still not a usable way to solve the implicit equations, for three reasons:

1. **It amplifies errors.** The marching recursion is unstable. Its characteristic equation $\mathrm{Fo}\left(z\right)^2 - (1 + 2\mathrm{Fo})\,z + \mathrm{Fo} = 0$ has roots whose product is $1$, so one root is larger than $1$ and multiplies every error at each step outwards. In a scratch experiment (60 nodes, $\mathrm{Fo} = 3.6$), a relative perturbation of $10^{-15}$ in $C_1$ grew to an absolute error of $4.3\times10^{-4}$ by the outer nodes. The marched outer value was $7.1\times10^{-4}$ instead of the imposed $0$.
2. **It cannot use the outer boundary condition.** The marching starts from the centre and never uses the outer condition.
3. **It needs a second starting value** that the centre condition does not supply.

The stable way is to write all rows at once, with the three new-time unknowns of each row on the left and the old value on the right, and to solve the tridiagonal system by Gaussian elimination specialised to three diagonals (the Thomas algorithm). Formally that is the same elimination, organised so that errors shrink instead of grow (document 01, Section 9.3).

### Page 9: sketches and a to-do list

- "$r = i\,\Delta r$", next to a sketch of concentric circles with $\Delta r$ marked as the innermost radius. The labels read "$i$ = number of shells" and "$r$ = total radius". With $r = i\,\Delta r$, $r$ is the radius of node $i$. It equals the total radius only at the outer node. This is the centre-at-$0$ numbering.
- A sketch of a curve with tangent and secant lines, $\Delta x$, and $\frac{dy}{dx} \approx \frac{\Delta y}{\Delta x}$, illustrating a derivative approximated by a difference quotient.
- A numbered to-do list, transcribed as far as the handwriting allows (some words are uncertain):
  - (1) find out how [the scheme; the handwriting resembles "FEM"] can go wrong in graph + code;
  - (2) find out how to [code the] exponent form [uncertain reading];
  - (3) redo code;
  - (4) watch a video lecture on FTCS.
- "Crank Nicolson", in blue.

### Page 10: plan and the backward Euler formula

"Approach through: (1) Focus on base case + ignore source terms. (2) Find cases to benchmark solutions against."

$$
x^{n+1} = x^n + h\,f\left(x^{n+1}, t^{n+1}\right). \tag{H27}
$$

**Correct.** This is backward Euler for $dx/dt = f(x,t)$ with step $h$. Here $n$ is a time index and $h$ a time step. In STRATO notation, $n$ becomes $j$, $h$ becomes $\Delta t$, and $x$ becomes the vector of nodal values $C_i$. (H27) is the time discretisation the directive prescribes.

---

## 2. Document T, section by section

Document T is the typed version of Document H. It corrects several of the slips found above: node numbering from $0$, the outer-node factor $2$, and the interior weights $1, \frac{1}{2}, \frac{1}{3}$. This section lists each equation with its check. Equations identical in content to Document H are only cross-referenced.

**Abstract and Section 1.** The abstract states the method: FTCS on a uniform radial mesh, symmetry at the centre via the L'Hôpital limit $3D\,\partial^2 c/\partial r^2$, zero flux at the outer node, optional source terms, and constant or temperature-dependent $D$. It also says that numerical results and verification are reported separately. The introduction motivates kernel-scale release in FHRs and sets out the plan of the paper. Its literature citations are unresolved placeholders ("[?]"), and the reference list is empty. Nothing to check algebraically.

**Nomenclature.** It defines the symbols of the table above. The index $i$ runs from $0$ (centre) to $4$ (surface), and $j$ is the time level. **Consistent** with the rest of Document T.

**Section 2: model formulation.**

- (T1) $\partial c/\partial t = D\nabla^2 c$ and (T2), the spherical Laplacian: as (H1) and (H2), **correct**.
- The two assumptions: Assumption 1 as in Document H, **correct**. Assumption 2 is reworded as "surface-coupled boundary: the concentration at any radial position is referenced to the value imposed at the kernel surface". It is still **not needed**: the reduction follows from Assumption 1 (Section 1 above).
- (T3) to (T5): as (H3) to (H5), **correct**.
- The text after (T5) says the curvature term "becomes singular at $r = 0$". **Correct** for the coefficient $2/r$. The product $\frac{2}{r}\frac{\partial c}{\partial r}$ stays finite for a smooth symmetric profile, because of (T14).

**Section 3: discretisation.**

- 3.1, mesh: $C_i^j \approx c(r_i, t_j)$ with $r_i = i\,\Delta r$, five nodes $i = 0, \dots, 4$, four shells. **Correct**, and it is the numbering in which the factor $1/i$ is exact.
- (T6) and (T7): central differences, written with "$\approx$". **Correct.**
- (T8): forward Euler. **Correct.**
- (T9): as (H8), with $r_i$. **Correct.**
- (T10): as (H9). **Correct.**
- (T11): the collected form (C1) above, with the indices in the right places. **Correct.**
- The text says the interior weights are $1/i = 1, \frac{1}{2}, \frac{1}{3}$ at $i = 1, 2, 3$. **Correct.**

**Section 3.4: stability.**

$$
\mathrm{Fo} = \frac{D\,\Delta t}{\left(\Delta r\right)^2}. \tag{T12}
$$

The section says: the central coefficient is $1 - 2\mathrm{Fo}$; for a planar stencil von Neumann requires $\mathrm{Fo} \le \frac{1}{2}$; "the additional curvature term tightens this bound near the centre". The first two statements are **correct**. The third is **Wording**: the bound is tightened, but by the centre node, not by the curvature term at interior nodes. The precise statements are:

- **Interior nodes.** In (T11) the neighbour coefficients are $\mathrm{Fo}\left(1 \pm \frac{1}{i}\right) \ge 0$ for every $i \ge 1$. So all coefficients of an interior row are non-negative exactly when $1 - 2\mathrm{Fo} \ge 0$, that is $\mathrm{Fo} \le \frac{1}{2}$. The curvature term does not tighten the interior bound.
- **Centre node.** (T20) has central coefficient $1 - 6\mathrm{Fo}$. Non-negativity, and with it a discrete maximum principle, needs $\mathrm{Fo} \le \frac{1}{6}$. For stability, note that (T11) at $i = 1$ has coefficient $\mathrm{Fo}\left(1 - \frac{1}{1}\right) = 0$ for $C_0$. The centre value therefore never feeds back into the other nodes, and the update matrix is block triangular. One of its eigenvalues is $1 - 6\mathrm{Fo}$, and $\lvert 1 - 6\mathrm{Fo}\rvert \le 1$ requires $\mathrm{Fo} \le \frac{1}{3}$.
- **Scratch experiment.** For the five-node insulated scheme of Table 1, the spectral radius of the update matrix is exactly $1$ for $\mathrm{Fo} \le \frac{1}{3}$. It is $1.04$ at $\mathrm{Fo} = 0.34$ and $2.0$ at $\mathrm{Fo} = \frac{1}{2}$. Bisection gave the largest stable $\mathrm{Fo}$ as $0.333333$ for 5, 21 and 81 nodes. Without the centre row, the remaining block is stable up to $\mathrm{Fo} = \frac{1}{2}$.

So the explicit scheme of Document T is stable for $\mathrm{Fo} \le \frac{1}{3}$ and non-negative for $\mathrm{Fo} \le \frac{1}{6}$. The planned sweep "across $\mathrm{Fo} = \frac{1}{2}$" (Section 6.2, item 4) would find the instability at $\frac{1}{3}$, in the centre value. None of this applies to BTCS, which needs no bound on $\mathrm{Fo}$ (document 01, Section 9).

**Section 4.1: initial condition.**

$$
\left.\frac{\partial c}{\partial t}\right|_{t=0} = 0. \tag{T13}
$$

**Error**, as (H11). The sentence that follows ("the prescribed initial profile ... is held fixed until the source terms or boundary values drive the transient") does refer to a prescribed profile. That profile, not (T13), is the initial condition.

**Section 4.2: centre node.**

- (T14) and (T15): $\partial c/\partial r = 0$ at $r = 0$; ghost $C_{-1}$, $(C_1 - C_{-1})/(2\Delta r) = 0 \Rightarrow C_{-1} = C_1$. **Correct** in centre-at-$0$ numbering.
- (T16): $\partial^2 c/\partial r^2|_{i=0} = (C_1 - 2C_0 + C_{-1})/\left(\Delta r\right)^2 = (2C_1 - 2C_0)/\left(\Delta r\right)^2$. **Correct.**
- (T17): $\lim_{r\to0}\frac{2}{r}\frac{\partial c}{\partial r} = 2\,\partial^2 c/\partial r^2|_{r=0}$. The **result is correct**; the justification is **Wording**. The text says the form is indeterminate "because both numerator and the $1/r$ factor diverge in opposite senses". In fact the numerator $\partial c/\partial r$ tends to $0$ by (T14), and the denominator $r$ tends to $0$: a $0/0$ form, which is what L'Hôpital's rule handles.
- (T18) and (T19): the operator $\to 3\,\partial^2 c/\partial r^2$ at the centre. **Correct.**
- (T20): $C_0^{j+1} = C_0^j + 6D\,\Delta t\,(C_1^j - C_0^j)/\left(\Delta r\right)^2$. **Correct** (factor $3 \times 2$).

**Section 4.3: outer node, insulated.**

- (T21) and (T22): $\partial c/\partial r|_{r=R} = 0$; ghost $C_5 = C_3$; $\partial^2 c/\partial r^2|_{i=4} = (2C_3 - 2C_4)/\left(\Delta r\right)^2$. **Correct.**
- The text explains that the L'Hôpital argument does not apply at $r = R > 0$, so the curvature term is $\frac{2}{R}\times 0 = 0$. **Correct**, and it fixes the factor-3 error of (H17).
- (T23): $C_4^{j+1} = C_4^j + 2D\,\Delta t\,(C_3^j - C_4^j)/\left(\Delta r\right)^2$. **Correct.**

The section ends with two statements:

- **"The insulated surface is the natural choice for verifying conservation, since ... the total quantity of species must then remain constant."** Correct for the continuous problem. For the discrete scheme it holds exactly only for one particular set of weights. Combining (T11) multiplied by $\left(r_i\right)^2\Delta r$ (which telescopes, Section 1, page 5 above) with (T20) and (T23), the exactly conserved sum is

  $$
  \sum_{i=1}^{3}\left(r_i\right)^2\Delta r\,C_i^j + \frac{r_3\,r_4\,\Delta r}{2}\,C_4^j, \qquad\text{with weight } 0 \text{ for the centre node } C_0^j. \tag{C4}
  $$

  The reason: the flow out of node 3 towards node 4 is $D\,r_3 r_4\,(C_3 - C_4)/\Delta r$. By (T23) node 4 gains $2D\,(C_3 - C_4)/\left(\Delta r\right)^2$ per unit time, so it must carry the weight $r_3 r_4\,\Delta r/2$ for the two to balance. In a scratch experiment (five nodes, $\mathrm{Fo} = 0.1$, 2000 steps), the sum (C4) changed by a relative $1.4\times10^{-14}$, round-off. With trapezoidal weights ($\left(r_4\right)^2\Delta r/2$ for the surface node) it drifted by $+10\%$. With exact shell volumes it drifted by $+1.7\%$. So the conservation check of Section 6.2, item 1, passes only with the weights (C4).
- **A Dirichlet condition fixing $C_4$ "(Assumption 2)" is to be used "when coupling to the coating layers is required".** In STRATO the coating layers are part of the domain, and the Dirichlet condition (zero) sits at the outer OPyC surface $R_5$, not at the kernel surface (document 01, Section 6).

**Section 4.4 and Table 1.** The summary: centre (T20) with factor 6, interior (T11) with $1/i = 1, \frac{1}{2}, \frac{1}{3}$, surface (T23) with factor 2. **Correct.**

**Section 5: source terms and diffusion-coefficient models.**

- (T24): $\partial c/\partial t = D\left(\partial^2 c/\partial r^2 + \frac{2}{r}\partial c/\partial r\right) + S(c,r,t)$, with $S$ evaluated explicitly at level $j$. **Correct.**
- (T25): $S_{\mathrm{gen}} = \Sigma_f\,\phi\,Y$. **Correct in form, inconsistent in units.** $\Sigma_f\,\phi$ has units $\mathrm{m^{-1}}\times\mathrm{m^{-2}\ s^{-1}} = \mathrm{m^{-3}\ s^{-1}}$ (fissions per cubic metre per second). Multiplied by the yield in atoms per fission, it gives atoms $\mathrm{m^{-3}\ s^{-1}}$. The nomenclature gives $S$ in $\mathrm{mol\ m^{-3}\ s^{-1}}$, which requires dividing by the Avogadro constant: $S_{\mathrm{gen}} = \Sigma_f\,\phi\,Y/N_A$. The directive also requires distinguishing independent from cumulative yields; that is a modelling choice for Phase G, outside this account.
- (T26): $S_{\mathrm{decay}} = -\lambda c$, $\lambda = \ln 2/t_{1/2}$. **Correct.**
- (T27): $S_{\mathrm{trap}} = -\kappa c$. **Correct** as a model statement. It is outside the approved STRATO scope (directive Section 17).
- (T28): $D(T) = D_0\exp\left(-Q/(R_g T)\right)$. **Correct.**
- Section 5.5 names the multilayer extension with flux continuity at the interfaces. That is the subject of document 01.

**Section 6: verification strategy.**

- **Item 1 (conservation).** See (C4).
- **Item 2 (relaxation to the volume average of the initial state).** Discretely, the insulated scheme relaxes to a uniform value, because every row of the update matrix sums to $1$. That value is the average weighted with (C4), not the exact volume average. In the five-node scratch run the uniform value was $0.1450$, while the shell-volume average of the same nodal data was $0.1426$. The two agree only as $\Delta r \to 0$.
- **Item 3 (analytical limit).** Sound.
- **Item 4 (stability sweep across $\frac{1}{2}$).** See Section 3.4 above: the threshold is $\frac{1}{3}$.

**Section 7: discussion.**

- (T29): as (H20). **Correct.** The text calls it "algebraically identical to eq. (10)". That is right once $\partial c/\partial t$ is replaced by the forward difference. As printed, (T29) is the semi-discrete form.
- (T30): backward Euler, as (H27). **Correct.**
- The sentence "the unknown $C_i^{j+1}$ appears on both sides and is obtained by making $C_{i+1}^{j+1}$ the subject and solving the resulting tridiagonal system across the mesh" mixes two procedures. **The second half is correct; the first half is the unstable marching of pages 7 and 8** (Section 1 above). In BTCS no unknown is "made the subject". All rows are solved together.
- The Crank-Nicolson remarks (second order in time, unconditionally stable for linear diffusion) are **correct**. Crank-Nicolson is A-stable but not L-stable: it damps the fastest modes only weakly. With STRATO's discontinuous start, those modes appear as slowly decaying oscillations near the kernel surface, unless the first steps are taken with backward Euler. The directive prescribes BTCS, which damps them strongly.
- Section 7.3 says the explicit scheme is first order in time and second order in space, "so accuracy is set by the smaller of the two truncation errors". **Wording**: the total error is the sum of the two, so the larger of the two terms sets the accuracy. The intended meaning was probably the lower order.

**Section 8: conclusion.** It summarises the paper and calls the model "the verified baseline". Document T contains no numerical results; it states that they are reported separately. Within the two documents, therefore, nothing has been verified numerically yet.

---

## 3. The owner's open questions and planned steps, with answers

| Where | Question or plan, as written (paraphrased where noted) | Answer | Where it is worked out |
|---|---|---|---|
| H p. 3 (Q1) | "Should this be final equation to be discretised or should I use previous form" | Use $3D\,\partial^2 C/\partial r^2$ at the centre node only, where $2/r$ is undefined; everywhere else use the "previous form" (H5), in its conservative variant (H20). The ghost-node discretisation of the centre form is correct (factor 6). Under the owner's interior stencil, the centre value receives information from node 1 but sends none back, and it carries zero weight in the conserved inventory. | Document 01, Sections 3.5 and 4 |
| H p. 5 | Base case 1, constant $D$; base case 2, $D(T)$ | STRATO starts from base case 1, with a different constant $D$ in each of the five layers. $D(T)$ is directive Phase G3. The scheme of document 01 takes any positive $D$ per interval, so $D(T)$ changes only the coefficients. | Document 01, Sections 5 and 8 |
| H p. 5 | Source terms: generation, decay, trapping | Generation and decay are approved (directive Section 17). Trapping is not. Generation needs the unit correction $1/N_A$ above. | Directive Phase G |
| H p. 5 | "$D$ is also different for each layer" | This is the five-layer extension. It requires starting from the conservation law, not from $D\nabla^2 c$, and treating interfaces with flux continuity. | Document 01, Sections 1 and 5 |
| H p. 5 | "$D$ changes with irradiation damage" | Not in the approved STRATO scope. | n/a |
| H pp. 7 and 8 | Implicit form by making $C_{i+1}$ the subject | Use backward Euler: all new values on the left side, solved simultaneously as a tridiagonal system (Thomas algorithm). Marching for $C_{i+1}$ is unstable. | Document 01, Sections 3 and 9 |
| H p. 9 (1) | Find out how the scheme can go wrong (in plots and in code) | For FTCS: instability for $\mathrm{Fo} > \frac{1}{3}$ (centre node), negative values for $\mathrm{Fo} > \frac{1}{6}$, apparent loss of mass if the wrong inventory weights are used, wrong node-numbering conventions. For BTCS: no time-step limit; the remaining pitfalls are interface treatment and the initial value at the kernel-surface node. | This document, Section 2; document 01, Sections 5, 7 and 9 |
| H p. 9 | "Crank Nicolson" | Second order in time, but not L-stable (see Section 2). The directive prescribes BTCS. | This document, Section 2 |
| H p. 10 | Focus on the base case without sources; find benchmark cases | Benchmarks already available in STRATO: the homogeneous-sphere series solution and the existing five-layer Sturm-Liouville analytical solver (directive Phase D). | Document 01, Sections 7 and 11 |
| T Sect. 7.2 and 8 | Next steps: implicit integrator, mesh refinement, multilayer coefficients | All three are carried out in document 01: BTCS, a piecewise-uniform grid with any number of nodes per layer, and five layers. | Document 01 |

---

## 4. What carries over to STRATO, and what does not

| Element of the original work | Status in the five-layer BTCS model |
|---|---|
| Reduction to one radial dimension (Assumption 1) | Kept. |
| Assumption 2 ("concentration equals the surface value") | Dropped; not needed and not correct as worded. |
| Fick's second law with one constant $D$ | Replaced by the conservation law with Fick's first law, because $D$ jumps at four interfaces. Inside each layer the owner's constant-$D$ form (H5) still holds. |
| Central differences in space (H6) | Kept inside each layer. Across an interface, replaced by a flux balance. |
| Forward Euler in time (H7) | Replaced by backward Euler (H27), as the owner anticipated on page 10. |
| Regrouped stencil (H20) | Kept. It is the key to discrete conservation and generalises directly to five layers and non-uniform spacing. |
| $1/i$ curvature weight | Kept in the kernel, where $r_i = i\,\Delta r$. In the coatings $r_i \ne i\,\Delta r$, so the general form $\left(r_i \pm \Delta r\right)/r_i$ is used. |
| Centre node from $3D\,\partial^2 c/\partial r^2$ with a ghost node (factor 6) | Kept, with consistent indices and in implicit form. |
| Insulated outer node (factor 2 in (T23); factor 6 in (H17) is wrong) | Not used. Replaced by the absorbing condition $C_N^{j} = 0$ at $R_5$. |
| "Initial condition" $\partial c/\partial t = 0$ | Replaced by the kernel-only initial profile, with an inventory-matched value at the kernel-surface node. |
| Five-node illustrative mesh | Replaced by a piecewise-uniform mesh with any number of intervals per layer. |
| Explicit nodal update | Replaced by a tridiagonal linear system per time step. |

**On the scratch experiments quoted above.** They were run as short uncommitted programs, solely to check the algebraic claims in this document. They are numerical checks of the algebra; they are not experimental validation.
