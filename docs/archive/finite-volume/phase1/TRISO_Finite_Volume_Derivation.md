# Finite-Volume Derivation of TRISO Diffusion and Release

A TRISO particle is a small sphere with five layers. This document explains how a substance spreading through those layers is described by one equation, how a computer solves that equation by cutting the sphere into thin shells, and how the answer can be compared with an exact formula.

## 0. Symbols and units

| Symbol | Meaning | Units |
|---|---|---|
| $r,\ t$ | distance from the centre of the particle, time | $\mathrm{m}$, $\mathrm{s}$ |
| $R_i$ | outer radius of layer $i$ (Kernel, Buffer, IPyC, SiC, OPyC). The centre is $R_0=0$ and the particle surface is $R_5=R$ | $\mathrm{m}$ |
| $D_i$ | diffusivity of layer $i$: how fast the substance spreads there | $\mathrm{m^2\,s^{-1}}$ |
| $c(r,t)$ | concentration: amount of substance per cubic metre | $\mathrm{mol\,m^{-3}}$ |
| $J_r$ | flux: amount crossing one square metre per second, positive when moving outward | $\mathrm{mol\,m^{-2}\,s^{-1}}$ |
| $S$ | source: amount created per cubic metre per second | $\mathrm{mol\,m^{-3}\,s^{-1}}$ |
| $c_0$ | concentration in the kernel at the start | $\mathrm{mol\,m^{-3}}$ |
| $h,\ c_\infty$ | how quickly the surroundings take substance away, and the concentration of the surroundings (Case B only) | $\mathrm{m\,s^{-1}}$, $\mathrm{mol\,m^{-3}}$ |
| $\Lambda_n,\ \phi_n$ | $n$-th fade rate and its concentration shape (Section 4) | $\mathrm{s^{-1}}$, $\mathrm{mol\,m^{-3}}$ |
| $k_i=\sqrt{\Lambda/D_i}$ | how fast the shape wiggles inside layer $i$ | $\mathrm{m^{-1}}$ |
| $u=r\phi,\ q=Du'$ | helper quantities that make the sphere equation simpler (Section 4) | $\mathrm{mol\,m^{-2}}$, $\mathrm{mol\,m^{-1}\,s^{-1}}$ |
| $V_j,\ A_f$ | volume of shell $j$, area of a shell wall | $\mathrm{m^3}$, $\mathrm{m^2}$ |
| $G_f$ | how easily substance flows through a wall | $\mathrm{m^3\,s^{-1}}$ |
| $M,\ M_{\mathrm{rel}},\ F$ | amount still inside, amount released so far, fraction released $M_{\mathrm{rel}}/M^0$ | $\mathrm{mol}$, $\mathrm{mol}$, $1$ |

## 1. Scope and assumptions

Think of the particle as an onion with five layers. Layer $i$ lies between radius $R_{i-1}$ and radius $R_i$.

1. The particle is a ball of five nested layers: Kernel, Buffer, IPyC, SiC and OPyC.
2. Everything depends only on the distance $r$ from the centre (spherical symmetry). Each layer spreads the substance at its own constant speed $D_i$.
3. The layers touch perfectly. At a boundary the concentration is the same on both sides, and everything that leaves one layer enters the next. Nothing is lost or created at a boundary.
4. **Case A (release).** At the start all the substance is in the kernel: $c=c_0$ for $r<R_1$ and $c=0$ everywhere else. Nothing new is made afterwards ($S=0$). The outer surface acts like a perfect drain: an atom that reaches it is gone, so $c(R,t)=0$. The question is how much has escaped after time $t$. The finite-volume solver and the exact formula of Section 4 both solve Case A with the same inputs, so their answers can be compared directly.
5. **Case B (a separate test problem).** The substance is made continuously and the surroundings remove it at a limited speed (a Robin condition). Only the equation for the fade rates of this case is in the code (Section 4.6). Case A and Case B start from different situations, so their results are never compared.

## 2. From conservation to the spherical equation

**Atoms are not created or destroyed.** Pick any region $V$ of space. The amount of substance inside changes for two reasons only: substance flows through the surface of the region, or substance is created inside it. In words, the change inside equals minus the flow out plus what is created:

**Equation TRISO-GOV-020**

```math
\frac{d}{dt}\int_V c\,dV=-\int_{\partial V}\mathbf J\cdot\mathbf n\,dA+\int_V S\,dV
```

A result from calculus (the divergence theorem) turns the flow through the surface into a sum over the volume. Moving every term under one integral gives:

```math
\int_V\left(\frac{\partial c}{\partial t}+\nabla\cdot\mathbf J-S\right)dV=0
```

This holds for every region $V$, even a tiny one, so the bracket must be zero everywhere:

**Equation TRISO-GOV-030**

```math
\frac{\partial c}{\partial t}+\nabla\cdot\mathbf J=S
```

**Fick's law.** Substance spreads from crowded places to empty places, and the steeper the drop in concentration, the faster it flows. The minus sign says the flow points downhill:

**Equation TRISO-GOV-034**

```math
\mathbf J=-D\nabla c
```

Putting Fick's law into the conservation law gives the diffusion equation:

**Equation TRISO-GOV-040**

```math
\frac{\partial c}{\partial t}=\nabla\cdot(D\nabla c)+S
```

**The sphere.** In a ball everything depends on $r$ only, and the flow points along the radius: $\mathbf J=J_r(r,t)\,\mathbf e_r$. Look at a thin shell between $r$ and $r+dr$. Its inner wall has area $4\pi r^2$ and its outer wall has area $4\pi(r+dr)^2$. Counting what flows in, what flows out and what is made inside gives $\partial_t(4\pi r^2c\,dr)=-\left[4\pi(r+dr)^2J_r(r+dr)-4\pi r^2J_r(r)\right]+4\pi r^2S\,dr$. Dividing by the shell volume gives $\nabla\cdot\mathbf J=r^{-2}\partial_r(r^2J_r)$, and with Fick's law the equation for layer $i$ is

**Equation TRISO-FV-100**

```math
\frac{\partial c_i}{\partial t}=\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2D_i\frac{\partial c_i}{\partial r}\right)+S_i
```

## 3. Conditions

The equation above needs extra information: what happens at the centre, between the layers, at the outer surface and at the start.

**Centre.** By symmetry no substance flows through the exact centre point, so $J_r(0,t)=0$:

```math
\left.\frac{\partial c_1}{\partial r}\right|_{r=0}=0
```

**Between layers** ($r=R_i$, $i=1,\dots,4$). Two rules: the concentration is the same on both sides, and the flow leaving one layer equals the flow entering the next:

```math
c_i(R_i,t)=c_{i+1}(R_i,t),\qquad D_i\left.\frac{\partial c_i}{\partial r}\right|_{R_i^-}=D_{i+1}\left.\frac{\partial c_{i+1}}{\partial r}\right|_{R_i^+}
```

**Outer surface.** Case A is the perfect drain. Case B lets the surroundings remove substance at a rate proportional to the gap $c-c_\infty$:

```math
c_5(R,t)=0\qquad\text{or}\qquad -D_5\left.\frac{\partial c_5}{\partial r}\right|_R=h\,[c_5(R,t)-c_\infty]
```

**Start (Case A).** The kernel is full, the coatings are empty and nothing is added later:

```math
c(r,0)=\begin{cases}c_0,&0\le r<R_1\\0,&R_1<r<R\end{cases}\qquad S_i=0\ (t>0)
```

## 4. Exact solution of Case A

Case A can be solved exactly by writing the answer as a sum of simple shapes that each fade away at their own rate. This kind of problem is called a Sturm-Liouville problem. The code uses the exact answer as a second, independent way to get the release fraction.

### 4.1 Separation and transformation

**Shapes that only fade.** Look for solutions whose shape $\phi(r)$ stays the same while the whole thing fades as $e^{-\Lambda t}$, that is $c=\phi(r)\,e^{-\Lambda t}$. Putting this into the equation with $S=0$ gives, layer by layer,

```math
\frac{1}{r^2}\frac{d}{dr}\left(r^2D_i\frac{d\phi}{dr}\right)+\Lambda\phi=0
```

**A trick for spheres.** Writing $u=r\phi$ (Equation TRISO-ML-431) turns this into the equation of a vibrating string, whose solutions are sines and cosines:

**Equation TRISO-ML-432**

```math
u''+k_i^2u=0,\qquad k_i^2=\frac{\Lambda}{D_i}
```

**Equation TRISO-ML-433**

```math
u_i(r)=A_i\sin(k_ir)+B_i\cos(k_ir),\qquad \phi_i=\frac{u_i}{r}
```

At the centre the term $\cos(k_1r)/r$ blows up, which a real concentration cannot do, so the cosine term of the kernel must be zero: $B_1=0$ (TRISO-ML-435).

### 4.2 Conditions in the transformed variable

**Boundary between layers.** Equal concentration on both sides means equal $\phi$. Multiplying by $R_i$ gives

**Equation TRISO-ML-439**

```math
u_i(R_i)=u_{i+1}(R_i)
```

Equal flow uses $\phi'=(u'-u/r)/r$ and gives

**Equation TRISO-ML-446**

```math
D_i\left[u_i'(R_i)-\frac{u_i(R_i)}{R_i}\right]=D_{i+1}\left[u_{i+1}'(R_i)-\frac{u_{i+1}(R_i)}{R_i}\right]
```

The outer surface (perfect drain) requires $u_5(R)=0$.

**Watch out.** The quantity $D\,u'$ does not stay the same across a boundary when the two layers have different $D$. Using $D\,u'$ instead of Equation TRISO-ML-446 gives wrong fade rates and wrong release curves.

### 4.3 Characteristic function

**Walking outward.** Start at the centre with $u=0$ and keep track of the pair $(u,q)$ with $q=D\,u'$. Inside a layer of thickness $L$, the sine and cosine solution moves the pair forward like this:

**Equation TRISO-DEL-010**

```math
\begin{pmatrix}u\\ q\end{pmatrix}_{\mathrm{out}}=\begin{pmatrix}\cos k L & \dfrac{\sin kL}{Dk}\\[2mm] -Dk\sin kL & \cos kL\end{pmatrix}\begin{pmatrix}u\\ q\end{pmatrix}_{\mathrm{in}}
```

At the boundary at $R_i$, $u$ stays the same (Equation TRISO-ML-439) and Equation TRISO-ML-446 makes $q$ jump:

**Equation TRISO-DEL-011**

```math
q_{i+1}(R_i)=q_i(R_i)+\left(D_{i+1}-D_i\right)\frac{u(R_i)}{R_i}
```

Start from $(u,q)=(0,1)$ at $r=0$, walk through all five layers and call the final value of $u$ at the surface $g(\Lambda)$:

**Equation TRISO-DEL-012**

```math
g(\Lambda)=u(R;\Lambda),\qquad g(\Lambda_n)=0
```

Only special values of $\Lambda$ make $u$ vanish at the surface, which is what the perfect drain requires. These values $\Lambda_n$ are the allowed fade rates. The same condition can be written as a determinant, $\det\mathbf M(\Lambda)=0$ (TRISO-ML-474).

**No fade rate is missed.** The number of allowed fade rates below a given $\Lambda$ equals the number of times the solution crosses zero between the centre and the surface. Hunting for roots by bisecting on this whole-number count cannot skip one.

### 4.4 Orthogonality and release series

**Splitting the start into shapes.** The shapes are orthogonal, like perpendicular directions. With weight $r^2$ for $\phi$, that is weight $1$ for $u$, the boundary terms cancel and (TRISO-ML-528, TRISO-ML-531)

```math
\int_0^R u_mu_n\,dr=0\qquad(m\ne n)
```

This lets us write the starting state as a sum of shapes, $c=\sum_nA_n\phi_ne^{-\Lambda_nt}$, with $A_n=c_0\,I^{(1)}_n/N_n$, where

**Equation TRISO-DEL-020**

```math
I^{(1)}_n=\int_0^{R_1}r\,u_n\,dr,\qquad I_n=\int_0^{R}r\,u_n\,dr,\qquad N_n=\int_0^{R}u_n^2\,dr
```

The amount inside is $M(t)=4\pi\sum_nA_nI_ne^{-\Lambda_nt}$ and at the start $M^0=\tfrac{4\pi}{3}c_0R_1^3$. So the fraction released is

**Equation TRISO-DEL-021**

```math
F(t)=1-\sum_{n}w_n\,e^{-\Lambda_nt},\qquad w_n=\frac{3\,I^{(1)}_nI_n}{R_1^3\,N_n},\qquad\sum_nw_n=1
```

In words: each shape fades at its own rate, and the weight $w_n$ says how much of the starting substance sits in shape $n$. The weights add up to 1 because at $t=0$ everything is still inside. The integrals are computed layer by layer from the walked-through values, using $u(r)=u\cos ks+(q/Dk)\sin ks$ with $s=r-R_{i-1}$.

**Quick check with one material.** If all layers have the same $D$, the shapes are $u_n=\sin(n\pi r/R)$, the fade rates are $\Lambda_n=n^2\pi^2D/R^2$ and $N_n=R/2$. If the substance fills the whole sphere at the start ($R_1=R$), then $w_n=6/(n^2\pi^2)$ and we recover the classical result $F=1-\tfrac{6}{\pi^2}\sum n^{-2}e^{-n^2\pi^2Dt/R^2}$.

### 4.5 Status and numerical limits

- **Completeness and cut-off.** The proof that these shapes can describe every possible start for this layered sphere is not in the foundation document. So the series is checked numerically, against an independent implementation and through the weights adding up to 1. Shapes are included up to a largest fade rate $\Lambda_{\max}$, and the series is used only at times $t\ge40/\Lambda_{\max}$, when the omitted faster-fading shapes have shrunk by a factor $e^{-40}$.
- **Rounding errors.** When the flow $D(u'-u/r)$ at a boundary is tiny compared with $D\,u'$ and $D\,u/r$ (neighbouring layers with very different diffusivities), the calculation subtracts nearly equal numbers and loses digits. The code reports the smallest ratio $|D(u'-u/r)|/(|Du'|+|Du/r|)$ over all shapes and boundaries, and an accuracy floor of about $2.2\times10^{-16}$ divided by that ratio. Differences between the finite-volume answer and the series that are smaller than this floor cannot be trusted.

### 4.6 Homogeneous Robin sphere (Case B reference)

For a single material with a partly blocking surface ($\mathrm{Bi}=hR/D$), take $\phi=\sin(zr/R)/r$ and use $-D\phi'(R)=h\phi(R)$. This gives the equation that fixes the allowed values of $z$:

```math
z\cot z=1-\mathrm{Bi}
```

## 5. Conservative spherical finite volumes

**The idea.** Cut the particle into many thin spherical shells called cells, so that every boundary between two layers is exactly the wall between two cells. Track the average concentration in each cell. Substance only moves from cell to cell through walls, so what leaves one cell enters the next and the total is conserved exactly.

### 5.1 Geometry

Cell $j$ lies between $r_{j-1/2}$ and $r_{j+1/2}$. Its volume and the areas of its two walls are

**Equation TRISO-FV-101**

```math
V_j=\frac{4\pi}{3}\left(r_{j+1/2}^3-r_{j-1/2}^3\right),\qquad A_{j\pm1/2}=4\pi r_{j\pm1/2}^2
```

The unknown for each cell is its average concentration $C_j=V_j^{-1}\int c\,4\pi r^2dr$ (TRISO-FV-118). It is placed at the volume-weighted centre of the cell:

**Equation TRISO-FV-109A**

```math
r_j=\frac{3}{4}\,\frac{r_{j+1/2}^4-r_{j-1/2}^4}{r_{j+1/2}^3-r_{j-1/2}^3}
```

### 5.2 Exact cell balance

Multiply Equation TRISO-FV-100 by $4\pi r^2$ and add it up over the cell. The flow term adds up exactly to the difference between what crosses the two walls, $4\pi\left[r^2D\,\partial_rc\right]_{r_{j-1/2}}^{r_{j+1/2}}$. With $J_r=-D\,\partial_rc$ (TRISO-FV-114) this equals $A_{j-1/2}J_{j-1/2}-A_{j+1/2}J_{j+1/2}$, so

**Equation TRISO-FV-123**

```math
V_j\frac{dC_j}{dt}=A_{j-1/2}J_{j-1/2}-A_{j+1/2}J_{j+1/2}+S_jV_j
```

In words: the change in the cell equals the inflow through the inner wall, minus the outflow through the outer wall, plus what is created inside. This is exact. Approximations only enter when the flows through the walls are estimated.

### 5.3 Flow through a wall

**Inside one material.** The flow through a wall is proportional to the difference in concentration between the two neighbouring cells, divided by the distance between their centres, $\delta r_{PE}=r_E-r_P$ (TRISO-FV-124 to TRISO-FV-128):

```math
J_e\approx D\,\frac{C_P-C_E}{\delta r_{PE}}
```

**Wall between two materials.** Let $\delta_P=r_f-r_P$ and $\delta_E=r_E-r_f$ be the distances from each cell centre to the wall, and let $C_I$ be the concentration right at the wall (the same on both sides). Assume the concentration changes in a straight line across each half cell. The same flow crosses both halves:

```math
J_f=D_P\frac{C_P-C_I}{\delta_P}=D_E\frac{C_I-C_E}{\delta_E}
```

Eliminating $C_I$ (TRISO-FV-133 to TRISO-FV-138) gives $C_P-C_E=J_f\left(\delta_P/D_P+\delta_E/D_E\right)$. This is like two electrical resistors in series, where each half cell has a resistance of distance divided by diffusivity. The flow through the wall is therefore

**Equation TRISO-FV-140**

```math
A_fJ_f=G_f\,(C_P-C_E),\qquad G_f=\frac{A_f}{\delta_P/D_P+\delta_E/D_E}
```

If $D_P=D_E$ this is the ordinary two-point formula, and for equal half distances it contains the harmonic mean $2D_PD_E/(D_P+D_E)$. Treating each half cell as a straight-line drop, with the wall area $A_f$ on both halves, is the approximation made by the scheme. Comparison with the exact solution of Section 4 shows that the error shrinks with the square of the cell size, including across material boundaries.

### 5.4 Semi-discrete equation and conservation

Putting Equation TRISO-FV-140 into Equation TRISO-FV-123, with $G_W$ for the inner (west) wall and $G_E$ for the outer (east) wall, gives one equation per cell:

**Equation TRISO-FV-155**

```math
V_j\frac{dC_j}{dt}=G_W\,(C_{j-1}-C_j)+G_E\,(C_{j+1}-C_j)+S_jV_j
```

What leaves one cell through a wall enters its neighbour through the same wall, so all internal flows cancel when we add up $\sum_jV_jC_j$. The total amount in the particle changes only through the outer surface.

### 5.5 Centre and outer boundary

The first cell touches the centre, where the inner wall has zero area. There is no inflow there, so $G_W=0$, and no extra "ghost cell" is needed.

At the outer surface the perfect drain gives $c(R,t)=0$. The centre of the last cell is at $r_N$, so the drain is a distance $R-r_N$ away:

**Equation TRISO-DEL-030**

```math
G_R=\frac{4\pi R^2D_5}{R-r_N},\qquad V_N\frac{dC_N}{dt}=G_W(C_{N-1}-C_N)-G_RC_N,\qquad \dot M_{\mathrm{rel}}=G_RC_N
```

The last term is the amount leaving the particle per second.

### 5.6 Backward Euler

**Stepping in time.** Move forward in steps of $\Delta t$ and evaluate the flows with the new, still unknown, concentrations. For every cell this gives

```math
\frac{V_j}{\Delta t}\left(C_j^{n+1}-C_j^n\right)=G_W\left(C_{j-1}^{n+1}-C_j^{n+1}\right)+G_E\left(C_{j+1}^{n+1}-C_j^{n+1}\right)+S_j^{n+1}V_j
```

**Equation TRISO-DEL-040**

```math
-G_WC_{j-1}^{n+1}+\left(\frac{V_j}{\Delta t}+G_W+G_E\right)C_j^{n+1}-G_EC_{j+1}^{n+1}=\frac{V_j}{\Delta t}C_j^n+S_j^{n+1}V_j
```

Each equation links one cell to its two neighbours only, so the whole system is a tridiagonal one that a computer solves very quickly. Every number on the diagonal is positive and larger than the sum of the other entries in its row, which are zero or negative. This guarantees that the new concentrations are never negative when the old ones are not, for any time step.

**A solving recipe that wastes no digits.** Let $v_j=V_j/\Delta t$ and let $g_j^W=G_W$ of cell $j$ (so $g^W_{j+1}=g^E_j$). Define $e_1=v_1$ and $e_j=v_j+g^W_je_{j-1}/(e_{j-1}+g^W_j)$. Then the pivots are $d_j=e_j+g^E_j$, the forward sweep is $y_j=(\mathrm{rhs}_j+g^W_jy_{j-1})/d_j$, and the back substitution is $C_j=y_j+(g^E_j/d_j)\,C_{j+1}$ with $C_N=y_N$. Every step only adds, multiplies or divides positive numbers. The textbook recipe subtracts two nearly equal numbers when the Fourier number $D\Delta t/\Delta r^2$ is large, which wastes digits and lets the total-amount balance drift to about $10^{-9}$. The recipe above keeps the drift below $10^{-12}$ (observed at most $3\times10^{-13}$).

**A simpler stepping rule, for checks only.** The explicit (forward Euler) rule needs no equation solving but is safe only when

**Equation TRISO-FV-300**

```math
\Delta t\le\min_j\frac{V_j}{G_W+G_E}
```

Backward Euler is used for the real calculations and has no such limit.

### 5.7 Inventory, release, balance

The amount still inside is the sum of cell volume times concentration. The amount released adds up what leaves through the outer surface in each step:

```math
M^n=\sum_jV_jC_j^n,\qquad M_{\mathrm{rel}}^{n+1}=M_{\mathrm{rel}}^n+\Delta t\,G_RC_N^{n+1},\qquad F^n=\frac{M_{\mathrm{rel}}^n}{M^0},\qquad \dot F^n=\frac{G_RC_N^n}{M^0}
```

Adding Equation TRISO-DEL-040 over all cells makes every internal flow cancel and leaves

```math
M^{n+1}+M_{\mathrm{rel}}^{n+1}=M^0
```

up to solver and rounding error. The release uses the same flow through the outer surface as this balance, so the sum $M^n+M_{\mathrm{rel}}^n-M^0$ is an independent check that nothing is lost.

## 6. Limitations

- Results obtained with the synthetic test inputs supplied with the code are numerical checks, not physical validation.
- Temperature and fluence dependence of $D$, source schedules and a production Robin release mode are not implemented.
- The exact series is trustworthy only above the accuracy floor that the code reports for each run. The floor grows roughly in proportion to the diffusivity contrast between adjacent layers.
- Completeness of the eigenfunction set for the singular piecewise problem is checked numerically, not proved.
