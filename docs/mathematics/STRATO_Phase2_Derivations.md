# STRATO v2.0 Phase 2 derivations: decay, sources and inventory (Phase 2.1), temperature-dependent diffusivity (Phase 2.2)

Date: 2026-11-06. Status: DRAFT FOR REVIEW.

**Not implemented yet.** No Rust has been written for Phase 2, and no file of STRATO v1.0.0 under `src/`, `tests/` or `examples/` has been changed. Every Phase 2 equation in this document is DERIVED NOT IMPLEMENTED or FUTURE. The plain-language pseudocode that follows from it is `docs/mathematics/STRATO_Phase2_Pseudocode.md`. Work stops after both documents for the user's review.

## 1 How to read this document

### 1.1 Scope

STRATO v1.0 (Phase 1) solves Fickian diffusion of one fission product through the five layers of a TRISO particle (kernel, buffer, IPyC, SiC, OPyC) with constant layer diffusivities, an absorbing outer surface and an initial inventory in the kernel. Code B is the finite-volume solver and Code C is the eigenfunction (modal) series. Phase 2 extends both in two steps.

- Phase 2.1 (Sections 3 and 4) adds radioactive decay, a volumetric fission-product source and a complete inventory account: generated, decayed inside, released, still inside, and released but not yet decayed outside. Section 3 gives the continuous model and its modal solution. Section 4 gives the finite-volume scheme and the tests that will verify it.
- Phase 2.2 (Section 5) makes each layer diffusivity follow temperature by an Arrhenius law and lets the temperature follow a prescribed history $T(t)$.

### 1.2 Structure

| Part | Section | Content | Equations |
|---|---|---|---|
| Front matter | 1, 2 | reading guide, notation shared by the three parts | none |
| Part 2.1, analytical | 3 | transport equation with decay and source, decay law, yields, activation products, chains, modal solution, release, inventory closure, Booth limits | (P21.1) to (P21.49) |
| Part 2.1, finite volume | 4 | cell balance with decay and source, Backward Euler with implicit decay, decay-exact split, discrete closure, $M_\mathrm{out}$, step-size rules, exact references, verification matrix | (P21.50) to (P21.92) |
| Part 2.2, temperature | 5 | Arrhenius law, finite volume with $D(T(t))$, effective time, layer-specific activation energies, temperature histories | (P22.1) to (P22.59) |
| References | after 5 | all works cited in Sections 1 to 5 and Appendix B, APA 7 | none |
| Appendix A | A | verification scripts and their result lines from the B8 re-run | none |
| Appendix B | B | independent verification record: the three check logs and the B3 reconciliation | none |

Table: 1.1 Structure of the document.

Section numbers 3, 4 and 5 and the equation numbers are those of the reviewed drafts, so that every cross-reference in the pseudocode, the change document `docs/development/v1_to_v2_changes.md` and the parameter register stays valid. Phase 1 equations are cited by their tags in `docs/mathematics/phase1/TRISO_Finite_Volume_Derivation.md` (TRISO-ML-xxx, TRISO-FV-xxx, TRISO-DEL-xxx).

### 1.3 Labels

Each result carries a label that says how firm it is.

| Label | Meaning |
|---|---|
| exact | follows from the stated model without approximation |
| semi-analytical | an eigenfunction series; each term is exact once the eigenvalues and modal integrals are known (numerical roots and quadrature), and the series is truncated |
| approximate | a leading-order expansion or a model simplification; the text names the neglected term |
| numerical | a discretisation result whose error depends on mesh and step |
| NOT VERIFIED | a value or statement whose primary source was not opened or not found; it is not used as verified anywhere |
| OPEN | not resolvable from the available sources; listed in Sections 3.10, 4.10 and 5.7 |

Table: 1.2 Labels for results.

Operations are classified as IMPLEMENTED (with file and line in the v1.0.0 tag), DERIVED NOT IMPLEMENTED or FUTURE. Numerical properties are classified as PROVED, DEMONSTRATED NUMERICALLY or CLAIMED ONLY. The full classification tables are in Section 9 of the pseudocode; the review package `docs/development/phase2_review_package.md` gives a summary.

### 1.4 Sources and provenance

Page numbers are printed page numbers. A phrase such as "audit 07, Section 2" points to the Stage 1 audit notes, which were checked against the PDFs of the papers in the local corpus and are kept in the work folder, not in the repository. Numerical values of decay constants, yields and diffusivity correlations are collected, with their sources and status, in the parameter register `data/reference/STRATO_Parameter_Register.md` and the CSV files under `data/`. Code links point to the tag v1.0.0, for example [src/numerical/finite_volume.rs, lines 162 to 182](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L162-L182) is the v1.0 Backward Euler step.

### 1.5 Review history

Each part was drafted, then checked by an independent re-derivation whose findings are recorded in that part's check log. A reconciliation pass (B3) re-derived the key results again with new scripts that import nothing from the repository, corrected the drafts where needed and recorded each item as CONFIRMED, CORRECTED or OPEN. A final review (B7) re-ran every script, ran further independent checks and corrected wording and formatting errors; its notes close each check log. All of this is in Appendix B. The assembly of this document (B8) moved the text into one file, changed heading levels, split one-line display equations into the three-line form, added table captions and code links and merged the reference lists. Absolute local paths were shortened: repository paths are now relative to the repository root, and paths in the work folder, which is not part of the repository, start with `<work folder>/`. Three phrases were made explicit for the joined document: "the FV section" now reads "Section 4", the completeness remark in Section 3.10 names the Phase 1 derivation file, and the single-branch law of Section 4.7.6 is named as TECDOC-1674 Table 7.1 (audit 04). No equation, number or status was changed; the assembly script checks that all 151 tagged display equations are identical before and after.

### 1.6 Suggested reading order

1. Section 2 for notation, then the symbol table at the start of each part (Sections 3.0, 4.0 and 5.0).
2. Section 3.1 (the extended transport equation) and Section 3.6 (inventory closure). These two fix the physics of Phase 2.1.
3. Sections 4.3 and 4.5: the only change to the v1.0 step and the discrete closure that the code will check.
4. Sections 5.1, 5.2.3 and 5.3.6: the Arrhenius law, the time level of the conductances and the conditions under which the effective time is exact.
5. The status tables (Table 3.5, Table 4.3 and Table 5.6) and the open issues (Sections 3.10, 4.10 and 5.7).
6. Appendix B only where a result needs its evidence.

## 2 Notation and conventions

### 2.1 Conventions

- SI units throughout: metres, seconds, kelvin, moles, joules per mole. Temperatures in degrees Celsius appear only in inputs and examples and are converted by Eq. (P22.10).
- Layers are numbered $i = 1, \dots, 5$ from the kernel outwards, with outer radii $R_1 < \dots < R_5 = R$ and $R_0 = 0$.
- Finite-volume cells are numbered $j = 1, \dots, N$ from the centre. Time levels are written as superscripts, $C_j^n$.
- One nuclide is treated at a time unless a species index $k$ (Section 3) or a superscript $P$, $A$, $D$ (parent, activation product, daughter) is shown.
- Section 3 writes inventory subscripts upright ($M_\mathrm{gen}$) and Section 4 writes them in italics ($M_{gen}$). Both denote the same quantity.
- Display equations carry the numbers (P21.x) and (P22.x). Unnumbered displays are intermediate steps.

### 2.2 Symbols shared by the three parts

| Symbol | Meaning | SI unit |
|---|---|---|
| $r$, $t$ | radius, time | $\mathrm{m}$, $\mathrm{s}$ |
| $c(r,t)$ | concentration of the nuclide | $\mathrm{mol\ m^{-3}}$ |
| $D_i$ | diffusivity of layer $i$ (constant in Sections 3 and 4, $D_i(T(t))$ in Section 5) | $\mathrm{m^2\ s^{-1}}$ |
| $\lambda$, $t_{1/2}$ | decay constant, half-life, $\lambda = \ln 2/t_{1/2}$ | $\mathrm{s^{-1}}$, $\mathrm{s}$ |
| $S(r,t)$ | volumetric source | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $\dot F$, $y^I$, $y^C$, $N_A$ | fission rate density, independent and cumulative yield, Avogadro constant | $\mathrm{m^{-3}\ s^{-1}}$, none, none, $\mathrm{mol^{-1}}$ |
| $\Lambda_n$, $\phi_n$, $N_n$, $w_n$ | eigenvalues, eigenfunctions, norms and release weights of the layered sphere (Phase 1) | $\mathrm{s^{-1}}$, none, $\mathrm{m^3}$, none |
| $a_n(t)$, $s_n(t)$ | modal amplitude, projected source | $\mathrm{mol\ m^{-3}}$, $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $M$, $M_0$ | inventory inside the particle, initial inventory | $\mathrm{mol}$ |
| $M_\mathrm{gen}$, $M_\mathrm{dec}$, $M_\mathrm{rel}$, $M_\mathrm{out}$ | generated, decayed inside, released through $r = R$, released and not yet decayed outside | $\mathrm{mol}$ |
| $J_R(t)$ | release rate $-4\pi R^2D_5\,\partial_r c(R,t)$ | $\mathrm{mol\ s^{-1}}$ |
| $V_j$, $A_f$, $G_f$, $G_R$ | cell volume, face area, face conductance, surface conductance | $\mathrm{m^3}$, $\mathrm{m^2}$, $\mathrm{m^3\ s^{-1}}$, $\mathrm{m^3\ s^{-1}}$ |
| $T$, $R_g$ | temperature, molar gas constant | $\mathrm{K}$, $\mathrm{J\ mol^{-1}\ K^{-1}}$ |

Table: 2.1 Symbols shared by Sections 3, 4 and 5.

### 2.3 Symbols whose meaning depends on the part

The three parts were written separately, and a few letters carry different meanings. Each use is defined where it occurs. The pseudocode renames them so that one name means one thing (its Table 1.3).

| Symbol | Section 3 | Section 4 | Section 5 |
|---|---|---|---|
| $G$ | $G(t)$, total generation rate; $G_f$, face conductance of the check scheme in Section 3.9 only | $G_W$, $G_E$, $G_R$, face and surface conductances; generation is $\mathrm{Gen}$ | $G_f(t)$, face conductances |
| $B$ | $B(t)$, cumulative number of atoms born per unit volume; $B$ in $R/B$ is the birth rate | $\mathbf B_h$, the v1.0 step matrix; $B$ in $R/B$ is the birth rate | $B(t)$, Magnus generator $-K(t) - \lambda I$ |
| $k$ | species index; Phase 1 writes $k_i = \sqrt{\Lambda/D_i}$ | $k_i = \sqrt{\lambda/D_i}$, inverse decay length in layer $i$; also a step index in sums such as Eq. (P21.72) | index of a temperature interval or breakpoint ($t_k$, $T_k$) |
| $z$ | not used | $z = \lambda\Delta t$, decay per step | number of equivalent target sites of a jump |
| $x$ | $x = \lambda/D'$, decay-to-diffusion ratio of one sphere | same as Section 3 | generic argument of $E_n(x)$ |
| $R$ | particle radius | particle radius | particle radius; the gas constant is always $R_g$ |
| $f$ | $f_q$, fission fractions; $f_m$, isomeric fraction; face index in Section 3.9 | face index in $A_f$, $G_f$ | correlation factor of a jump; face index in $A_f$, $G_f$ |

Table: 2.2 Symbols with part-dependent meanings.

## 3 Part 2.1, analytical: continuous model with decay and source, and its modal solution

This section extends the Phase 1 model (STRATO v1.0, five layers, constant $D_i$, absorbing surface, initial kernel inventory) by radioactive decay and a volumetric source. Everything here is the **continuous** model and its **semi-analytical** (eigenfunction series) solution. The finite-volume discretisation of the same terms belongs to Section 4; the only discretised equations below are those of the independent Python check in Section 3.9, and they are marked as such.

Equation numbers (P21.1) to (P21.49). Phase 1 equations are cited by their tags in `docs/mathematics/phase1/TRISO_Finite_Volume_Derivation.md` (TRISO-ML-xxx, TRISO-DEL-xxx).

### 3.0 Symbols and units

| Symbol | Meaning | SI unit |
|---|---|---|
| $i = 1,\dots,5$ | layer index: kernel, buffer, IPyC, SiC, OPyC | none |
| $R_0 = 0 < R_1 < \dots < R_5 = R$ | layer outer radii; $R$ is the particle radius | $\mathrm{m}$ |
| $r$, $t$, $s$ | radius, time, dummy time | $\mathrm{m}$, $\mathrm{s}$, $\mathrm{s}$ |
| $k$ | species (nuclide) index when several nuclides appear | none |
| $c_k(r,t)$, $c$ | concentration of species $k$ (index dropped for one species) | $\mathrm{mol\ m^{-3}}$ |
| $D_{i,k}$, $D_i$ | diffusivity of species $k$ in layer $i$ | $\mathrm{m^2\ s^{-1}}$ |
| $\lambda_k$, $\lambda$ | decay constant | $\mathrm{s^{-1}}$ |
| $t_{1/2}$ | half-life | $\mathrm{s}$ |
| $S_{i,k}(r,t)$, $S$ | volumetric source | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $\dot F(r,t)$ | fission rate density (nonzero in the kernel only) | $\mathrm{fissions\ m^{-3}\ s^{-1}}$ |
| $y^I$, $y^C$ | independent and cumulative fission yield | atoms per fission |
| $N_A$ | Avogadro constant, $6.02214076\times10^{23}$ (SI defining constant) | $\mathrm{mol^{-1}}$ |
| $\chi_1(r)$ | indicator of the kernel: 1 for $r<R_1$, 0 otherwise | none |
| $\sigma$, $\phi$ | one-group capture cross section, neutron scalar flux | $\mathrm{m^2}$, $\mathrm{m^{-2}\ s^{-1}}$ |
| $f_m$ | fraction of captures that populate the product state (isomeric fraction) | none |
| $b$ | decay branching ratio parent to daughter | none |
| $u = r\,c$ (or $u_n = r\phi_n$) | transformed variable | $\mathrm{mol\ m^{-2}}$ (or $\mathrm{m}$) |
| $\Lambda_n$, $\phi_n(r)$ | eigenvalues and eigenfunctions of the layered sphere (Phase 1); $\phi_n$ taken dimensionless with arbitrary scale | $\mathrm{s^{-1}}$, none |
| $\kappa_n = \Lambda_n + \lambda$ | modal loss rate including radioactive decay (written $\kappa_n$, never $k_n$, because $k$ is the species index of the shared notation and $k_i = \sqrt{\Lambda/D_i}$ in Phase 1); superscripts $P$, $A$, $D$ mark the species, and $\kappa^P_n$, $\kappa^A_n$ also include capture $\sigma\phi$ (Section 3.3.3) | $\mathrm{s^{-1}}$ |
| $\psi$ | any second smooth function in the Green identity (P21.25) | as $\phi$ |
| $B(t)$ | cumulative number of product atoms born per unit volume (Section 3.3.2) | $\mathrm{m^{-3}}$ |
| $\lambda_p$, $y_p$, $N_p$ | precursor decay constant, yield into the precursor, precursor atom density | $\mathrm{s^{-1}}$, per fission, $\mathrm{m^{-3}}$ |
| $t_{99}$ | time for the precursor feed to reach 99 % of its equilibrium rate | $\mathrm{s}$ |
| $\beta$, $n_\mathrm{HM}$, $t_\mathrm{irr}$ | burnup (FIMA), initial heavy-metal atom density of the kernel, irradiation time | none, $\mathrm{m^{-3}}$, $\mathrm{s}$ |
| $f_q$ | fraction of fissions in fissile nuclide $q$ (U-235, Pu-239) | none |
| $O_{nm}$, $\gamma$ | overlap matrix (P21.19), constant ratio $D_{i,D}/D_{i,P}$ | none, none |
| $F_\mathrm{rel}$, $F_\mathrm{corr}$, $F_\mathrm{gen}$, $F_B$, $F_\mathrm{post}$, $R/B$ | release fractions of (P21.45) and release-rate-to-birth-rate ratio | none |
| $\delta_\pm$, $\delta$, $\theta$ | node-to-face distances, last node to $r=R$ distance, time weight (Section 3.9) | $\mathrm{m}$, $\mathrm{m}$, none |
| $N_n = \int_0^R r^2\phi_n^2\,dr$ | norm (weight $r^2$) | $\mathrm{m^3}$ |
| $I_n = \int_0^R r^2\phi_n\,dr$, $I^{(1)}_n = \int_0^{R_1} r^2\phi_n\,dr$ | whole-particle and kernel moments | $\mathrm{m^3}$ |
| $a_n(t)$ | modal amplitude | $\mathrm{mol\ m^{-3}}$ |
| $s_n(t)$ | projected source | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $w_n = 3I^{(1)}_nI_n/(R_1^3N_n)$ | v1.0 release weights (TRISO-DEL-021) | none |
| $M$, $M_0$, $M_\mathrm{gen}$, $M_\mathrm{rel}$, $M_\mathrm{dec}$, $M_\mathrm{out}$ | inventory inside, initial, generated, released through $r=R$, decayed inside, released and still undecayed outside | $\mathrm{mol}$ |
| $G(t) = 4\pi\int_0^R r^2 S\,dr$ | total generation rate | $\mathrm{mol\ s^{-1}}$ |
| $J_R(t) = -4\pi R^2D_5\,\partial_r c(R,t)$ | release rate | $\mathrm{mol\ s^{-1}}$ |
| $D' = D/R^2$ | reduced diffusivity (single sphere, Booth) | $\mathrm{s^{-1}}$ |
| $x = \lambda/D'$ | decay-to-diffusion ratio (single sphere) | none |
| $V_j$, $A_f$, $G_f$, $C_j^n$, $\Delta t$ | FV cell volume, face area, face conductance, cell value at step $n$, time step (Section 3.9 only) | $\mathrm{m^3}$, $\mathrm{m^2}$, $\mathrm{m^3\ s^{-1}}$, $\mathrm{mol\ m^{-3}}$, $\mathrm{s}$ |

Table: 3.1 Symbols and units of Part 2.1 (continuous model).

The temperature $T$ is held constant in this section, so every $D_i$ is a constant. Time-dependent temperature, $\alpha(t)$ and $\tau(t)$ belong to Phase 2.2.

**Assumptions used throughout.**

- A1. Spherical symmetry; Fickian diffusion with constant $D_i$ in each layer.
- A2. Perfect contact at interfaces: $c$ and the radial flux are continuous (no partition coefficient), as in FRESCO-II Eq. (10.4b) (IAEA, 2012, p. 531).
- A3. Absorbing outer surface $c(R,t) = 0$ and a regular centre.
- A4. Decay is first order with the same $\lambda_k$ in every layer (a nuclear property, independent of the host material).
- A5. The fission source is confined to the kernel and is uniform there unless stated otherwise. Fission-fragment recoil into the buffer is neglected.
- A6. Released atoms never return to the particle. Outside the particle they decay with the same $\lambda$.

### 3.1 (a) Extended transport equation

For species $k$ in layer $i$ ($R_{i-1} < r < R_i$):

$$
\frac{\partial c_k}{\partial t} = \frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_{i,k}\frac{\partial c_k}{\partial r}\right) - \lambda_k c_k + S_{i,k}(r,t)
\tag{P21.1}
$$

Units of each term: $\mathrm{mol\ m^{-3}\ s^{-1}}$. The diffusion term is $\mathrm{m^{-2}}\cdot\mathrm{m^{-1}}\cdot(\mathrm{m^2}\cdot\mathrm{m^2\ s^{-1}}\cdot\mathrm{mol\ m^{-4}})$, the decay term is $\mathrm{s^{-1}}\cdot\mathrm{mol\ m^{-3}}$. The form matches the participant equations in the CRP-6 benchmark: ATLAS Eq. (10.1) (sign typo on the divergence noted in audit 04), FRESCO-II Eq. (10.3b) and NRCDIF Eq. (10.6) (IAEA, 2012, pp. 529 to 536).

Interface conditions at $r = R_i$, $i = 1,\dots,4$ (A2):

$$
c_k(R_i^-,t) = c_k(R_i^+,t),\qquad D_{i,k}\,\partial_r c_k(R_i^-,t) = D_{i+1,k}\,\partial_r c_k(R_i^+,t)
\tag{P21.2}
$$

Centre and surface (A3):

$$
\partial_r c_k(0,t) = 0\ \ (\text{regular centre}),\qquad c_k(R,t) = 0
\tag{P21.3}
$$

Initial condition (zero for an irradiation started from fresh fuel, or the end-of-irradiation profile for a heating test, Section 3.3.5):

$$
c_k(r,0) = c_{k,0}(r)
\tag{P21.4}
$$

Neither the decay term nor the source changes the interface or boundary conditions, so the Phase 1 rules (TRISO-ML-439, TRISO-ML-446, $u_5(R) = 0$) carry over unchanged. **Status: exact model statement under A1 to A6.**

### 3.2 (b) Radioactive decay law

For $N$ atoms of one nuclide, each atom decays independently with a constant probability per unit time $\lambda$. In a short interval $dt$ the expected loss is $\lambda N\,dt$:

$$
\frac{dN}{dt} = -\lambda N
\tag{P21.5}
$$

Separate variables, $dN/N = -\lambda\,dt$, integrate from $N_0$ at $t=0$:

$$
N(t) = N_0\,e^{-\lambda t}
\tag{P21.6}
$$

Half-life: set $N(t_{1/2}) = N_0/2$, so $e^{-\lambda t_{1/2}} = 1/2$ and

$$
\lambda = \frac{\ln 2}{t_{1/2}},\qquad \bar t = \int_0^\infty t\,\lambda e^{-\lambda t}\,dt = \frac{1}{\lambda}
\tag{P21.7}
$$

where $\bar t$ is the mean life. In (P21.1) the same law appears per unit volume as $-\lambda c$. **Status: exact.**

ENSDF adopted values (via the IAEA LiveChart API, 1 y = 31 556 925.97 s; audit 07, Section 2):

| Nuclide | $t_{1/2}$ (ENSDF) | $t_{1/2}$ (s) | $\lambda$ ($\mathrm{s^{-1}}$) | Evaluation |
|---|---|---|---|---|
| Cs-137 | 30.08(9) y | $9.492323\times10^{8}$ | $7.302187\times10^{-10}$ | Browne and Tuli (2007) |
| Sr-90 | 28.91(3) y | $9.123107\times10^{8}$ | $7.597709\times10^{-10}$ | Basu and McCutchan (2020) |
| Ag-110m | 249.83(4) d | $2.158531\times10^{7}$ | $3.211198\times10^{-8}$ | Gürdal and Kondev (2012) |
| Cs-134 | 2.0652(4) y | $6.517136\times10^{7}$ | $1.063576\times10^{-8}$ | Sonzogni (2004) |
| Eu-154 | 8.592(5) y | $2.711371\times10^{8}$ | $2.556445\times10^{-9}$ | Nica (2025) |

Table: 3.2 ENSDF half-lives and decay constants of the Phase 2.1 nuclides.

Ag-110m decays by $\beta^-$ (98.67 %) and by isomeric transition to Ag-110 (1.33 %); the $\lambda$ above is the total, which is the right quantity for the $-\lambda c$ loss term. TECDOC-1674 Table 8.1 (IAEA, 2012, p. 435) prints slightly different half-lives (Cs-137 30.07 y, Sr-90 28.78 y, Ag-110m 249.8 d); STRATO should use the ENSDF adopted values. The script recomputes every $\lambda$ from $t_{1/2}$ and agrees with audit 07 to better than $3\times10^{-7}$ relative.

### 3.3 (c) Source terms

#### 3.3.1 Direct fission products

A fission rate density $\dot F$ multiplied by the atoms of nuclide $k$ produced per fission gives atoms $\mathrm{m^{-3}\ s^{-1}}$; dividing by $N_A$ converts to moles:

$$
S_k(r,t) = \frac{\dot F(r,t)\,y_k}{N_A}\,\chi_1(r)
\tag{P21.8}
$$

Units: $(\mathrm{m^{-3}\ s^{-1}})(\text{count})/(\mathrm{mol^{-1}}) = \mathrm{mol\ m^{-3}\ s^{-1}}$ (checked with `sympy.physics.units`).

If only the burnup $\beta$ (FIMA, fissions per initial heavy-metal atom) reached after an irradiation time $t_\mathrm{irr}$ is known, the time-averaged fission rate density follows from the definition of FIMA,

$$
\bar{\dot F} = \frac{\beta\,n_\mathrm{HM}}{t_\mathrm{irr}}
\tag{P21.9}
$$

with $n_\mathrm{HM}$ the initial heavy-metal atom density of the kernel ($\mathrm{m^{-3}}$). This is exact as a time average and approximate as a rate history (A5 also assumes no radial fission profile inside the kernel). Kernel heavy-metal densities are not part of this section's sources: NOT VERIFIED here.

When both U-235 and Pu-239 fission (fission fractions $f_q$, $\sum_q f_q = 1$), the yield is the fission-weighted mean (audit 07, Section 6):

$$
y_{\mathrm{eff},k} = \sum_q f_q\,y_{k,q}
\tag{P21.10}
$$

#### 3.3.2 Independent or cumulative yield

The independent yield $y^I$ counts atoms formed directly by fission; the cumulative yield $y^C$ adds every atom that arrives later by decay of precursors, $y^C(k) = y^I(k) + \sum_j b_{j\to k}\,y^C(j)$ (audit 07, Section 5.1).

To decide which one a single-nuclide model should use, take one precursor $p$ (decay constant $\lambda_p$, yield $y_p$ into the precursor) feeding the product with branching 1. Precursor balance and product birth rate:

$$
\frac{dN_p}{dt} = y_p\dot F - \lambda_p N_p,\qquad \text{birth rate} = y^I\dot F + \lambda_p N_p
\tag{P21.11}
$$

With $N_p(0) = 0$ and constant $\dot F$, $N_p = y_p\dot F(1-e^{-\lambda_p t})/\lambda_p$. Integrating the birth rate from 0 to $t$ and using $y^C = y^I + y_p$:

$$
B(t) = y^C\dot F\,t - \frac{y_p\dot F}{\lambda_p}\left(1 - e^{-\lambda_p t}\right)\ \xrightarrow{t\gg 1/\lambda_p}\ y^C\dot F\left(t - \frac{y_p/y^C}{\lambda_p}\right),\qquad t_{99} = \frac{\ln 100}{\lambda_p} = 6.644\,t_{1/2,p}
\tag{P21.12}
$$

So the true birth $B(t)$ lags the cumulative-yield model $y^C\dot F t$ (which runs ahead of it) by $(y_p/y^C)/\lambda_p \le 1/\lambda_p = t_{1/2,p}/\ln 2$, and after $t_{99}$ the precursor feed rate $\lambda_pN_p$ has reached 99 % of its equilibrium value $y_p\dot F$ (equivalently, 99 % of the precursor atoms born at any instant have decayed into the product). **Status: exact for a single precursor; the rule "use $y^C$" is an approximation valid when $t_{99}$ is short compared with the time step and with the diffusion time, and the precursor does not migrate before decaying.**

Numbers (JEFF-3.1.1 thermal yields, U-235, audit 07, Sections 3 and 5.4; $t_{99}$ recomputed by the script):

| Product | $y^I$ | $y^C$ | Longest precursor | $t_{99}$ | Choice |
|---|---|---|---|---|---|
| Cs-137 | $7.2248\times10^{-4}$ | $6.2208\times10^{-2}$ | Xe-137, 3.818 min | 0.423 h | $y^C$ |
| Sr-90 | $3.1343\times10^{-4}$ | $5.7290\times10^{-2}$ | Rb-90m, 258 s | 0.476 h | $y^C$ |

Table: 3.3 Independent and cumulative yields, longest precursor and choice of yield.

Here $y^I/y^C$ is 1.2 % for Cs-137 and 0.55 % for Sr-90: using $y^I$ alone would underestimate the source by factors of 86 and 183. The precursors Xe-137 and Kr-90 are noble gases that could migrate during their few minutes of life; the effect is sub-hour and is neglected (audit 07, Section 5.4). The independent yield becomes the correct choice only when the precursor is itself modelled as a separate species (Section 3.3.4), in which case $y^C$ would count the precursor twice (audit 07, Section 5.2).

#### 3.3.3 Activation products: Ag-110m, Cs-134, Eu-154

These three nuclides are shielded from the fission chains: the isobaric $\beta^-$ chains end at stable Pd-110, Xe-134 and Sm-154 (audit 07, Section 4). Their cumulative yields are negligible (Cs-134 $1.213\times10^{-7}$, Eu-154 $1.95\times10^{-9}$ for U-235; Ag-110m blank in the API for U-235 and $1.5503\times10^{-8}$ for Pu-239). They are produced by neutron capture on a stable target that is itself a fission product: Ag-109($n,\gamma$), Cs-133($n,\gamma$), Eu-153($n,\gamma$).

Target $P$ (stable, so $\lambda_P = 0$ unless it is radioactive) with concentration $c_P$, produced by fission and burned by capture:

$$
\frac{\partial c_P}{\partial t} = \frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2D_{i,P}\frac{\partial c_P}{\partial r}\right) - (\lambda_P + \sigma_P\phi)\,c_P + \frac{\dot F\,y^C_P}{N_A}\chi_1(r)
\tag{P21.13}
$$

Activation product $A$:

$$
\frac{\partial c_A}{\partial t} = \frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2D_{i,A}\frac{\partial c_A}{\partial r}\right) - (\lambda_A + \sigma_A\phi)\,c_A + S_A,\qquad S_A(r,t) = f_m\,\sigma_P\,\phi\,c_P(r,t)
\tag{P21.14}
$$

Two features follow from the physics.

1. The source $S_A$ sits wherever target atoms sit, including atoms that have already diffused into the coatings, so $S_A$ is not confined to the kernel.
2. Target and product are isotopes of the same element (Ag, Cs, Eu). Assuming isotope effects on diffusion are negligible next to the scatter of the Arrhenius data (assumption, not quantified here), $D_{i,A} = D_{i,P}$ in every layer. Then both species share the eigenfunctions $\phi_n$ (Section 3.4), and with a spatially uniform $\phi$ (a particle of under $1\ \mathrm{mm}$ sees a flat flux; assumption) the projection of $S_A$ is diagonal:

$$
\frac{da^P_n}{dt} = -\kappa^P_n a^P_n + s^P_n,\qquad \frac{da^A_n}{dt} = -\kappa^A_n a^A_n + f_m\sigma_P\phi\,a^P_n,\qquad \kappa^{P}_n = \Lambda_n + \lambda_P + \sigma_P\phi,\ \ \kappa^{A}_n = \Lambda_n + \lambda_A + \sigma_A\phi
\tag{P21.15}
$$

For constant $\phi$, constant $\dot F$ and zero initial inventories:

$$
a^A_n(t) = \frac{f_m\sigma_P\phi\,s^P_n}{\kappa^P_n}\left[\frac{1 - e^{-\kappa^A_n t}}{\kappa^A_n} - \frac{e^{-\kappa^P_n t} - e^{-\kappa^A_n t}}{\kappa^A_n - \kappa^P_n}\right]
\tag{P21.16}
$$

Checks (script): satisfies both ODEs, $a^A_n(0) = 0$, steady value $f_m\sigma_P\phi s^P_n/(\kappa^P_n\kappa^A_n)$, and early growth $f_m\sigma_P\phi s^P_n t^2/2$ (quadratic, a two-step process). **Status: exact within A1 to A6 and constant $\phi$.**

Data needed and not available in the audited sources (all **NOT VERIFIED**):

- one-group capture cross sections $\sigma_P$ for Ag-109, Cs-133 and Eu-153 in an HTGR spectrum;
- the isomeric fraction $f_m$ for Ag-109($n,\gamma$)Ag-110m (for Cs-134 the isomer Cs-134m decays 100 % by IT to the ground state with $t_{1/2} = 2.912$ h, so $f_m \approx 1$ for times beyond a day; for Eu-154 the isomer branch is not documented here);
- burn-out cross sections $\sigma_A$ of the products themselves;
- the neutron flux history $\phi(t)$ of each irradiation.

The target yields are known (U-235 / Pu-239 $y^C$): Ag-109 $2.8765\times10^{-4}$ / $1.6732\times10^{-2}$, Cs-133 $6.5963\times10^{-2}$ / $6.9948\times10^{-2}$, Eu-153 $1.4772\times10^{-3}$ / $3.8033\times10^{-3}$ (audit 07, Section 3). The factor 58 between the Pu-239 and U-235 Ag-109 yields means the Ag-110m source depends strongly on the Pu fission fraction $f_q$ in (P21.10). The Cs-133 precursor Xe-133 has $t_{99} = 34.9$ d, which delays the Cs-134 source in the first month (audit 07, Section 5.4).

#### 3.3.4 Parent-daughter chains with diffusion (Bateman with diffusion)

Parent $P$ decays to daughter $D$ with branching $b$; each diffuses with its own $D_{i,P}$, $D_{i,D}$:

$$
\frac{\partial c_P}{\partial t} = \mathcal L_P c_P - \lambda_P c_P + S_P
\tag{P21.17}
$$

$$
\frac{\partial c_D}{\partial t} = \mathcal L_D c_D - \lambda_D c_D + b\lambda_P c_P + \frac{\dot F\,y^I_D}{N_A}\chi_1
\tag{P21.18}
$$

with $\mathcal L_k c = r^{-2}\partial_r(r^2D_{i,k}\partial_r c)$. The daughter fission term uses $y^I_D$: $y^C_D$ already contains the parent, so $y^C_D\dot F + b\lambda_P N_P$ double counts (audit 07, Section 5.2).

Each species has its own eigenpairs $(\Lambda^P_m,\phi^P_m)$ and $(\Lambda^D_n,\phi^D_n)$. Expand $c_P = \sum_m a^P_m\phi^P_m$ and $c_D = \sum_n a^D_n\phi^D_n$, multiply (P21.18) by $r^2\phi^D_n$ and integrate (the steps are those of Section 3.4.2). The parent term projects as $b\lambda_P\sum_m O_{nm}a^P_m$ with the **overlap matrix**

$$
O_{nm} = \frac{1}{N^D_n}\int_0^R r^2\,\phi^D_n(r)\,\phi^P_m(r)\,dr
\tag{P21.19}
$$

so the modal system couples:

$$
\frac{da^D_n}{dt} = -(\Lambda^D_n + \lambda_D)\,a^D_n + b\lambda_P\sum_m O_{nm}\,a^P_m(t) + s^D_n(t)
\tag{P21.20}
$$

For a source-free interval with $a^P_m(t) = a^P_m(0)e^{-\kappa^P_mt}$, $\kappa^P_m = \Lambda^P_m+\lambda_P$, $\kappa^D_n = \Lambda^D_n+\lambda_D$ (and $\kappa^D_n \ne \kappa^P_m$):

$$
a^D_n(t) = a^D_n(0)\,e^{-\kappa^D_nt} + b\lambda_P\sum_m O_{nm}\,a^P_m(0)\,\frac{e^{-\kappa^P_mt} - e^{-\kappa^D_nt}}{\kappa^D_n - \kappa^P_m}
\tag{P21.21}
$$

When does $O$ reduce to the identity? The eigenfunctions depend only on the ratios $D_{i,k}/D_{j,k}$ between layers. If $D_{i,D} = \gamma\,D_{i,P}$ in every layer with one constant $\gamma$ (including $\gamma = 1$, same element), then $\phi^D_n = \phi^P_n$ up to scale, $\Lambda^D_n = \gamma\Lambda^P_n$, and

$$
O_{nm} = \delta_{nm}\quad\Rightarrow\quad a^D_n(t) = a^D_n(0)e^{-\kappa^D_nt} + b\lambda_P a^P_n(0)\frac{e^{-\kappa^P_nt} - e^{-\kappa^D_nt}}{\kappa^D_n - \kappa^P_n}
\tag{P21.22}
$$

which is the two-member Bateman solution mode by mode. For a single homogeneous sphere $O$ is always diagonal, since $\phi_n = \sin(n\pi r/R)/r$ for any $D$. For real chains in a layered particle (Sr-90 to Y-90, Cs-137 to Ba-137m) the element changes, the layer ratios differ, and $O$ is a full matrix. The individual entries depend on how each $\phi_n$ is scaled: rescaling $\phi^P_m \to c_m\phi^P_m$ and $\phi^D_n \to d_n\phi^D_n$ turns $O_{nm}$ into $O_{nm}c_m/d_n$ and $a^P_m$ into $a^P_m/c_m$, so $O_{nm}a^P_m$ still carries the factor $1/d_n$ (as $a^D_n$ does), and only the field contributions $O_{nm}a^P_m\phi^D_n$ and $c_D$ itself are scale-free. The statement $O_{nm} = \delta_{nm}$ in (P21.22) assumes $\phi^D_n$ is taken equal to $\phi^P_n$ (same scaling, for example $\phi_n(0) = 1$).

**Recommendation for v2.0.** Start with independent single nuclides (Cs-137, Sr-90, Kr-85 ground state, I-131 with the fission source of (P21.8), noting that for Kr-85 (precursor Kr-85m, $t_{99} = 29.8$ h) and I-131 (precursor Te-131m, $t_{99} = 9.2$ d, comparable to the I-131 half-life) the instantaneous $y^C$ source of Section 3.3.2 is only approximate (audit 07, Section 5.4); Ag-110m once its activation data exist) and document coupled chains as future work. Two reasons support this. The daughter diffusivities in each layer are not in the audited data, and the overlap coupling turns $N$ independent scalar ODEs into an $N\times N$ system. The activation pair target/product of Section 3.3.3 is the one chain that stays diagonal and therefore cheap.

#### 3.3.5 Irradiation and post-irradiation phases

A test such as the CRP-6 cases 4a to 5b (audit 04, Sections 4.2 and 6) is irradiation followed by heating. With a constant source during irradiation:

$$
S(r,t) = \begin{cases} S_\mathrm{irr}(r), & 0 \le t < t_\mathrm{irr}\\ 0, & t \ge t_\mathrm{irr}\end{cases},\qquad c(r,t_\mathrm{irr}^+) = c(r,t_\mathrm{irr}^-)
\tag{P21.23}
$$

The heating phase starts from the end-of-irradiation profile, which is not uniform in the kernel and already has atoms in the coatings. In modal form no new projection is needed: the amplitudes $a_n(t_\mathrm{irr})$ carry over (P21.34). FRESCO-II normalised its inventory to 100 % at the end of irradiation (IAEA, 2012, p. 532); the equivalent here is to divide by $M(t_\mathrm{irr})$ (Section 3.6).

### 3.4 (d) Modal solution

#### 3.4.1 Eigenproblem and orthogonality

The eigenproblem is the Phase 1 problem (TRISO-ML-432 to TRISO-ML-446, TRISO-DEL-010 to TRISO-DEL-012):

$$
\frac{1}{r^2}\frac{d}{dr}\left(r^2D_i\frac{d\phi_n}{dr}\right) + \Lambda_n\phi_n = 0,\quad \phi_n,\ D_i\phi_n'\ \text{continuous at } R_i,\quad \phi_n \text{ regular at } 0,\quad \phi_n(R) = 0
\tag{P21.24}
$$

Why decay does not change it: put $c = e^{-\lambda t}\tilde c$ into (P21.1) with $S = 0$. The $-\lambda c$ terms cancel and $\tilde c$ obeys the Phase 1 equation, because $\lambda$ is the same in every layer (A4). Gelbard (2003, p. 11, Eq. II.6) uses this transform for one sphere. If $\lambda$ varied between layers the cancellation would fail and the eigenfunctions would change.

For any two smooth functions $\phi,\psi$ on a layer,

$$
\psi\,(r^2D\phi')' - \phi\,(r^2D\psi')' = \left[r^2D\,(\psi\phi' - \phi\psi')\right]'
\tag{P21.25}
$$

Integrate (P21.25) over every layer and add. At each interface the bracket is continuous (both $\phi$ and $D\phi'$ are), so the interior contributions cancel in pairs; at $r=0$ the factor $r^2$ removes it, and at $r=R$ both functions vanish. Hence $\int_0^R r^2(\psi\mathcal L\phi - \phi\mathcal L\psi)\,dr = 0$: the operator is self-adjoint with weight $r^2$. Applied to two eigenfunctions, $(\Lambda_m - \Lambda_n)\int_0^R r^2\phi_m\phi_n\,dr = 0$, so

$$
\int_0^R r^2\phi_m\phi_n\,dr = N_n\,\delta_{mn},\qquad N_n = \int_0^R r^2\phi_n^2\,dr = \int_0^R u_n^2\,dr
\tag{P21.26}
$$

(weight $r^2$ for $\phi$ equals weight 1 for $u = r\phi$, as in the v1.0 code).

#### 3.4.2 Projection: the modal ODE

Expand the solution in the eigenfunctions:

$$
c(r,t) = \sum_{n\ge1} a_n(t)\,\phi_n(r),\qquad a_n(t) = \frac{1}{N_n}\int_0^R r^2\,c(r,t)\,\phi_n(r)\,dr
\tag{P21.27}
$$

Multiply (P21.1) by $r^2\phi_n$ and integrate over $[0,R]$:

$$
\frac{d}{dt}\int_0^R r^2c\,\phi_n\,dr = \int_0^R r^2\phi_n\,\mathcal Lc\,dr - \lambda\int_0^R r^2c\,\phi_n\,dr + \int_0^R r^2S\,\phi_n\,dr
$$

$c$ satisfies the same interface and boundary conditions as $\phi_n$, so self-adjointness gives $\int r^2\phi_n\mathcal Lc\,dr = \int r^2c\,\mathcal L\phi_n\,dr = -\Lambda_n N_na_n$. Dividing by $N_n$:

$$
\frac{da_n}{dt} = -(\Lambda_n + \lambda)\,a_n + s_n(t) = -\kappa_n\,a_n + s_n(t)
\tag{P21.28}
$$

$$
s_n(t) = \frac{1}{N_n}\int_0^R r^2\,S(r,t)\,\phi_n(r)\,dr,\qquad a_n(0) = \frac{1}{N_n}\int_0^R r^2\,c_0(r)\,\phi_n(r)\,dr
\tag{P21.29}
$$

This derivation never differentiates the series term by term; it only uses the projection (P21.27), so it holds even though the series for $\partial_rc$ converges slowly. For a source uniform in the kernel, $S = S_0(t)\chi_1$:

$$
s_n(t) = S_0(t)\,\frac{I^{(1)}_n}{N_n},\qquad I^{(1)}_n = \int_0^{R_1}r^2\phi_n\,dr = \int_0^{R_1}r\,u_n\,dr
\tag{P21.30}
$$

which has the same structure as the v1.0 initial-condition projection $A_n = c_0I^{(1)}_n/N_n$ (TRISO-DEL-020).

#### 3.4.3 Solutions in time

Multiply (P21.28) by the integrating factor $e^{\kappa_nt}$: $\frac{d}{dt}(e^{\kappa_nt}a_n) = e^{\kappa_nt}s_n$. Integrate from 0 to $t$:

$$
a_n(t) = a_n(0)\,e^{-\kappa_nt} + \int_0^t e^{-\kappa_n(t-s)}\,s_n(s)\,ds
\tag{P21.31}
$$

Constant source $s_n$:

$$
a_n(t) = a_n(0)\,e^{-\kappa_nt} + \frac{s_n}{\kappa_n}\left(1 - e^{-\kappa_nt}\right)\ \xrightarrow{t\to\infty}\ \frac{s_n}{\kappa_n}
\tag{P21.32}
$$

Piecewise-constant source, $s_n = s_n^{(j)}$ on $[t_{j-1},t_j)$, $\Delta_j = t_j - t_{j-1}$:

$$
a_n(t_j) = a_n(t_{j-1})\,e^{-\kappa_n\Delta_j} + \frac{s_n^{(j)}}{\kappa_n}\left(1 - e^{-\kappa_n\Delta_j}\right)
\tag{P21.33}
$$

The recursion is exact (it equals (P21.31) for that source), so a stepwise power history is integrated with no time-step error. Source switched off at $t_\mathrm{irr}$:

$$
a_n(t) = a_n(t_\mathrm{irr})\,e^{-\kappa_n(t - t_\mathrm{irr})},\qquad t \ge t_\mathrm{irr}
\tag{P21.34}
$$

**Status: semi-analytical (series). Each $a_n(t)$ is exact given $\Lambda_n$, $N_n$ and the source projection; the truncation at $N$ modes is the main approximation, and the eigenvalues are numerical roots and the modal integrals are evaluated by quadrature (Phase 1).**

#### 3.4.4 Inventory

Integrate (P21.27) over the sphere:

$$
M(t) = 4\pi\int_0^R r^2c\,dr = 4\pi\sum_n a_n(t)\,I_n,\qquad I_n = \int_0^R r^2\phi_n\,dr = \int_0^R r\,u_n\,dr
\tag{P21.35}
$$

For the homogeneous sphere $I_n = (-1)^{n+1}R^2/(n\pi)$ with $\phi_n = \sin(n\pi r/R)/r$ (script).

### 3.5 (e) Release rate and cumulative release

#### 3.5.1 Flux identity and the direct series

Integrate the eigen-equation (P21.24) times $r^2$ over $[0,R]$. The flux terms telescope across interfaces (continuity of $D\phi'$) and vanish at the centre:

$$
R^2D_5\,\phi_n'(R) = -\Lambda_n I_n
\tag{P21.36}
$$

Then $J_R = -4\pi R^2D_5\sum_n a_n\phi_n'(R)$ becomes, without any derivative,

$$
J_R(t) = 4\pi\sum_n \Lambda_n\,I_n\,a_n(t)
\tag{P21.37}
$$

and, integrating in time,

$$
M_\mathrm{rel}(t) = 4\pi\sum_n \Lambda_nI_n\int_0^t a_n\,ds,\qquad \int_0^t a_n\,ds = a_n(0)\frac{1-e^{-\kappa_nt}}{\kappa_n} + \frac{s_n}{\kappa_n}\left(t - \frac{1-e^{-\kappa_nt}}{\kappa_n}\right)\ (\text{constant } s_n)
\tag{P21.38}
$$

#### 3.5.2 Slow convergence and the balance form

For a steady kernel source, $a_n \approx s_n/\kappa_n$ and the terms of (P21.37) behave like $G\,w_n\Lambda_n/\kappa_n \to G\,w_n$, which decay only as $n^{-2}$ for the homogeneous sphere ($w_n = 6/(n^2\pi^2)$). The truncation error is then $O(N^{-1})$. The terms of $M$ behave like $w_n/\kappa_n \sim n^{-4}$, error $O(N^{-3})$. These are worst-case rates for sign-definite $w_n$. In the two-layer kernel-source case of Section 3.9, $|w_n|n^2$ stays of order one but $w_n$ changes sign 284 times in 400 modes, the tails partly cancel, and the independent check observed $N^{-2.1}$ for the direct $M_\mathrm{rel}$ and $N^{-4.0}$ for the balance form (`ind_p21_layered.py`, Section 3.9).

The cure is algebraic. From (P21.28), $\kappa_n\int_0^ta_n\,ds = a_n(0) - a_n(t) + \int_0^ts_n\,ds$. Write $\Lambda_n = \kappa_n - \lambda$ in (P21.38):

$$
4\pi\sum_n^N\Lambda_nI_n\!\int_0^t\!a_n = \underbrace{4\pi\sum_n^NI_na_n(0)}_{\to M_0} + \underbrace{4\pi\sum_n^NI_n\!\int_0^t\!s_n}_{\to M_\mathrm{gen}} - \underbrace{4\pi\sum_n^NI_na_n(t)}_{M^{(N)}} - \underbrace{4\pi\lambda\sum_n^NI_n\!\int_0^t\!a_n}_{M_\mathrm{dec}^{(N)}}
$$

The two slow sums are truncated completeness sums whose limits are known exactly ($M_0$ from $c_0$, $M_\mathrm{gen}$ from $S$). Replacing them by those limits gives the **balance form** (the steady $R/B$ expression assumes the kernel-uniform source, for which $4\pi s_nI_n/G = w_n$, Section 3.7.2; for another source shape replace $w_n$ by $4\pi s_nI_n/G$):

$$
M_\mathrm{rel}(t) = M_0 + M_\mathrm{gen}(t) - M^{(N)}(t) - M^{(N)}_\mathrm{dec}(t),\qquad \frac{J_R}{G}\Big|_{\text{steady}} = 1 - \lambda\sum_n\frac{w_n}{\kappa_n}
\tag{P21.39}
$$

Both $M^{(N)}$ and $M^{(N)}_\mathrm{dec}$ converge like $\sum w_n/\kappa_n$. For the transient flux during an irradiation that starts from zero inventory ($a_n(0) = 0$, kernel-uniform constant source) the same split gives $J_R = G\left(1 - \lambda\sum_n w_n/\kappa_n\right) - 4\pi\sum_n\Lambda_nI_n(s_n/\kappa_n)e^{-\kappa_nt}$, whose second sum converges through the exponential for $t>0$.

Script result (homogeneous sphere, $x = \lambda/D' = 1$, $D't = 0.1$; $R/B_\mathrm{ss}$ errors against TECDOC-1674 Eq. (7.6), $M_\mathrm{rel}$ errors against the time integral of Eq. (7.7), which is derived in the script and not printed in TECDOC-1674; $M_\mathrm{rel}$ errors in units of $G/D'$):

| $N$ | $R/B_\mathrm{ss}$ direct | $R/B_\mathrm{ss}$ balance | $M_\mathrm{rel}$ direct | $M_\mathrm{rel}$ balance |
|---|---|---|---|---|
| 10 | $5.78\times10^{-2}$ | $1.77\times10^{-5}$ | $5.77\times10^{-3}$ | $1.94\times10^{-5}$ |
| 100 | $6.05\times10^{-3}$ | $2.02\times10^{-8}$ | $6.05\times10^{-4}$ | $2.23\times10^{-8}$ |
| 1000 | $6.08\times10^{-4}$ | $2.05\times10^{-11}$ | $6.08\times10^{-5}$ | $2.26\times10^{-11}$ |

Table: 3.4 Errors of the direct and balance forms against the number of modes $N$.

Observed orders 0.99 (direct) and 2.94 (balance). STRATO should compute $M_\mathrm{rel}$ and $R/B$ with (P21.39) and keep (P21.37) only as a diagnostic.

### 3.6 (f) Inventory closure

Integrate (P21.1) over the sphere with weight $4\pi r^2$. The diffusion term gives $4\pi[r^2D\partial_rc]_0^R$ after the interface terms cancel by (P21.2), which is $-J_R$:

$$
\frac{dM}{dt} = G(t) - \lambda M(t) - J_R(t)
\tag{P21.40}
$$

Define the cumulative quantities

$$
M_\mathrm{gen}(t) = \int_0^tG\,ds,\qquad M_\mathrm{dec}(t) = \lambda\int_0^tM\,ds,\qquad M_\mathrm{rel}(t) = \int_0^tJ_R\,ds
\tag{P21.41}
$$

Integrating (P21.40) from 0 to $t$:

$$
M_0 + M_\mathrm{gen}(t) = M(t) + M_\mathrm{dec}(t) + M_\mathrm{rel}(t)
\tag{P21.42}
$$

**Status: exact for the continuous model**, and exact to round-off for a conservative FV scheme whose release and decay increments use the same time weighting as the update (Section 3.9).

Released atoms keep decaying outside (A6): $dM_\mathrm{out}/dt = J_R - \lambda M_\mathrm{out}$, $M_\mathrm{out}(0) = 0$, so

$$
M_\mathrm{out}(t) = \int_0^tJ_R(s)\,e^{-\lambda(t-s)}\,ds \le M_\mathrm{rel}(t)
\tag{P21.43}
$$

$M_\mathrm{rel}$ counts every atom that ever crossed $r = R$; $M_\mathrm{out}$ counts those still present outside at time $t$. They differ by the atoms that decayed after release, and they coincide only for $\lambda = 0$. Adding (P21.40) and the $M_\mathrm{out}$ equation, the surviving total obeys $d(M+M_\mathrm{out})/dt = G - \lambda(M+M_\mathrm{out})$:

$$
M(t) + M_\mathrm{out}(t) = M_0e^{-\lambda t} + \int_0^tG(s)\,e^{-\lambda(t-s)}\,ds
\tag{P21.44}
$$

which does not depend on any diffusivity.

**Release fractions by scenario.**

$$
\begin{aligned}
&\text{(I) source-free heating:} && F_\mathrm{rel} = \frac{M_\mathrm{rel}}{M_0},\quad F_\mathrm{corr} = \frac{e^{\lambda t}M_\mathrm{out}}{M_0} = \frac{1}{M_0}\int_0^tJ_R(s)e^{\lambda s}ds,\quad \text{and } F_\mathrm{rel}(\infty) = \sum_n w_n\frac{\Lambda_n}{\Lambda_n+\lambda}\ (\text{kernel-uniform } c_0)\\
&\text{(II) irradiation from zero:} && F_\mathrm{gen} = \frac{M_\mathrm{rel}}{M_\mathrm{gen}},\quad \frac{R}{B}(t) = \frac{J_R(t)}{G(t)},\quad F_B = \frac{M_\mathrm{out}}{M + M_\mathrm{out}}\\
&\text{(III) heating after irradiation:} && F_\mathrm{post}(t) = \frac{M_\mathrm{rel}(t) - M_\mathrm{rel}(t_\mathrm{irr})}{M(t_\mathrm{irr})}
\end{aligned}
\tag{P21.45}
$$

Remarks. In (I), $J_Re^{\lambda t}$ equals the flux of the $\lambda = 0$ problem (substitution $c = e^{-\lambda t}\tilde c$), so the decay-corrected fraction $F_\mathrm{corr}$ is exactly the Phase 1 $F(t)$, whatever $\lambda$ is. $F_\mathrm{rel}$ saturates below 1 because part of the inventory decays before it escapes. Gelbard's (II.18) is $1 - M/M_0 = F_\mathrm{rel} + M_\mathrm{dec}/M_0$, "released or decayed" (Gelbard, 2003, p. 14), and should not be compared with $F_\mathrm{rel}$. In (II), $R/B$ is the release-rate-to-birth-rate ratio of TECDOC-1674 and $F_B$ is the quantity in its Eq. (7.8) (shown in Section 3.7).

### 3.7 (g) Limiting cases

#### 3.7.1 $\lambda = 0$, $S = 0$: Phase 1

With $\kappa_n = \Lambda_n$ and kernel-uniform $c_0$, (P21.32) gives $a_n = c_0(I^{(1)}_n/N_n)e^{-\Lambda_nt}$; (P21.35) with $M_0 = \tfrac43\pi R_1^3c_0$:

$$
F(t) = 1 - \frac{M}{M_0} = 1 - \sum_nw_ne^{-\Lambda_nt},\qquad w_n = \frac{4\pi a_n(0)I_n}{M_0} = \frac{3I^{(1)}_nI_n}{R_1^3N_n}
\tag{P21.46}
$$

which is TRISO-DEL-021, the v1.0 release series. **Exact reduction.**

#### 3.7.2 Kernel source in the layered particle: $R/B(t)$

For a constant kernel-uniform source from zero inventory, $4\pi s_nI_n/G = w_n$: the source weights equal the v1.0 weights. From (P21.37) and (P21.32),

$$
\frac{R}{B}(t) = \sum_nw_n\frac{\Lambda_n}{\kappa_n}\left(1 - e^{-\kappa_nt}\right) = 1 - \lambda\sum_n\frac{w_n}{\kappa_n} - \sum_nw_n\frac{\Lambda_n}{\kappa_n}e^{-\kappa_nt}
\tag{P21.47}
$$

(the second form is the balance form, using $\sum_nw_n = 1$).

#### 3.7.3 Single homogeneous sphere: Booth with source and decay

One layer, $R_1 = R$, $\phi_n = \sin(n\pi r/R)/r$, $\Lambda_n = n^2\pi^2D'$, $N_n = R/2$, $w_n = 6/(n^2\pi^2)$, $\Lambda_n/\kappa_n = n^2\pi^2/(n^2\pi^2 + x)$. Substituting in (P21.47) term by term gives TECDOC-1674 Eq. (7.7). For the cumulative fraction, use (P21.44) with $M_0 = 0$ and constant production $G$: $M + M_\mathrm{out} = G(1-e^{-\lambda t})/\lambda$, hence $F_B = 1 - \lambda M/[G(1 - e^{-\lambda t})]$. With $M/G = \sum_nw_n(1-e^{-\kappa_nt})/\kappa_n$ and the split $1 - e^{-\kappa_nt} = (1 - e^{-\lambda t}) + e^{-\lambda t}(1 - e^{-n^2\pi^2D't})$:

$$
\frac{R}{B}(t) = \sum_{n}\frac{6}{n^2\pi^2 + x} - 6e^{-\lambda t}\sum_{n}\frac{e^{-n^2\pi^2D't}}{n^2\pi^2 + x},\qquad F_B(t) = \sum_n\frac{6}{n^2\pi^2+x} - \frac{6}{\pi^2}\frac{x}{e^{\lambda t}-1}\sum_n\frac{1 - e^{-n^2\pi^2D't}}{n^2(n^2\pi^2 + x)}
\tag{P21.48}
$$

These are TECDOC-1674 Eqs (7.7) and (7.8) (IAEA, 2012, p. 319) once $\sum_n 6/(n^2\pi^2+x)$ is replaced by its closed form below. The derivation identifies what (7.8) measures: the fraction of the **surviving** produced atoms that are outside the particle, $F_B = M_\mathrm{out}/(M + M_\mathrm{out})$. Multiplying numerator and denominator by $e^{\lambda t}$ gives the equivalent form $F_B = \int_0^tJ_Re^{\lambda s}ds\big/\int_0^tG\,e^{\lambda s}ds$: decay-corrected release divided by decay-corrected production. It differs from $M_\mathrm{rel}/M_\mathrm{gen}$ whenever $\lambda > 0$, and it differs from $F_\mathrm{corr}$ of scenario (I), whose denominator is $M_0$.

#### 3.7.4 Steady-state $R/B$ with decay

As $t\to\infty$ in (P21.47):

$$
\left(\frac{R}{B}\right)_\mathrm{ss} = \sum_nw_n\frac{\Lambda_n}{\Lambda_n + \lambda} = \int_0^\infty\left(-\frac{d}{dt}\sum_nw_ne^{-\Lambda_nt}\right)e^{-\lambda t}\,dt\ \xrightarrow{\text{one sphere}}\ 3\sqrt{\frac{1}{x}}\left(\coth\sqrt{x} - \sqrt{\frac{1}{x}}\right)
\tag{P21.49}
$$

The middle form reads: the steady $R/B$ is the Laplace transform, at $\lambda$, of the Phase 1 release-time density $dF/dt$. An atom born at a uniformly random kernel point escapes with the Phase 1 time distribution and survives that long with probability $e^{-\lambda t}$, so $(R/B)_\mathrm{ss}$ is its escape probability. The same number is the ultimate release fraction $F_\mathrm{rel}(\infty)$ of scenario (I) with a kernel-uniform $c_0$. The right-hand form is TECDOC-1674 Eq. (7.6), using $\sum_n6/(n^2\pi^2+x) = 3x^{-1/2}(\coth\sqrt x - x^{-1/2})$ (checked to $10^{-25}$ with mpmath). Limits of the single-sphere form: $1 - x/15 + 2x^2/315 + \dots$ for $x\to0$, and $3/\sqrt{x}$ for $x\to\infty$, which is TECDOC-1674 Eq. (7.25) (IAEA, 2012, p. 402). For the layered particle with a kernel source no short-lived closed form is derived here; (P21.49) with the layered $w_n$ and $\Lambda_n$ is the result to use.

### 3.8 Status of each result

| Result | Label |
|---|---|
| (P21.1) to (P21.4), (P21.40) to (P21.44) | exact (continuous model under A1 to A6) |
| (P21.5) to (P21.7), (P21.11), (P21.12) | exact; choice of $y^C$ is approximate (delay up to $1/\lambda_p$) |
| (P21.9) | exact as a time average, approximate as a rate history |
| (P21.13) to (P21.16) | exact for constant $\phi$; data NOT VERIFIED |
| (P21.17) to (P21.22) | exact equations; series with truncation in both $n$ and $m$ |
| (P21.27) to (P21.39), (P21.45) to (P21.47) | semi-analytical series; each modal amplitude exact given $\Lambda_n$ and the modal integrals (numerical roots and quadrature); worst-case truncation error (sign-definite $w_n$) $O(N^{-3})$ for $M$, $M_\mathrm{dec}$ and the balance form, $O(N^{-1})$ for the direct flux series; faster when $w_n$ change sign (Section 3.5.2) |
| (P21.48), (P21.49) single sphere | exact; (7.6) closed form; the (7.7) series converges exponentially for $t>0$; the (7.8) series converges algebraically, terms $\sim 1/(\pi^2n^4)$, tail $O(N^{-3})$ |

Table: 3.5 Status of each result of Section 3.

### 3.9 Verification

Scripts in `scripts/derivation_checks/` (run with `.venv/Scripts/python.exe -I -X utf8 <script>`; none writes files; the shared helper `sec3_p21_common.py` is independent of `src/`).

**`sec3_p21_decay_source.py`** (SymPy, P21.5 to P21.23): `SUMMARY: 24/24 PASS`. Selected lines:

```text
PASS  P21.7 lambda(Cs-137) = ln2/T1/2 reproduces audit 07  lambda = 7.302187e-10 s^-1, rel diff 2.9e-08
PASS  P21.7 lambda(Ag-110m) = ln2/T1/2 reproduces audit 07  lambda = 3.211198e-08 s^-1, rel diff 1.1e-07
PASS  P21.12 cumulative birth = yC F t - yp F (1-exp(-lp t))/lp
PASS  P21.12 t99 for Xe-137 (3.818 min) matches audit 07  t99 = 0.423 h
PASS  P21.16 activation mode solves a_A' = -kA a_A + f sigma phi a_P, a_A(0)=0
PASS  P21.21 Bateman-diffusion modal solution satisfies coupled ODE
PASS  P21.22 O = I reduces to two-member Bateman per mode
```

**`sec3_p21_modal_ode.py`** (SymPy and mpmath, P21.24 to P21.49): `SUMMARY: 32/32 PASS`. Selected lines:

```text
PASS  P21.25 psi(r^2 D phi')' - phi(r^2 D psi')' = [r^2 D (psi phi' - phi psi')]'
PASS  P21.36 R^2 D phi_n'(R) = -Lambda_n I_n
PASS  P21.31 integrating-factor solution satisfies a' = -k a + s(t)
PASS  P21.33 piecewise-constant recursion equals integrating factor
PASS  P21.39 per-mode identity: direct release = M0 + Mgen - M - Mdec
PASS  P21.42 per-mode closure M0 + Mgen = M + Mdec + Mrel
PASS  P21.44 M + M_out = M0 e^(-lambda t) + int G e^(-lambda (t-s)) ds solves (M+M_out)' = G - lambda (M+M_out)
PASS  P21.45 decay-corrected flux J_R e^(lambda t) equals the lambda = 0 flux (per mode)
PASS  P21.46 4 pi a_n(0) I_n / M_0 = 3 I1_n I_n /(R1^3 N_n)
PASS  P21.48 modal R/B(t) term = TECDOC-1674 (7.7) term
PASS  P21.48 F_B = M_out/(M + M_out) term = TECDOC-1674 (7.8) term (uses sum w_n = 1)
PASS  P21.49 (7.6) closed form = sum 6/(n^2 pi^2 + x), x = 100.0  R/B = 0.270000001237
PASS  P21.49 short-lived limit: R/B_ss * sqrt(x)/3 -> 1 (TECDOC-1674 7.25)
```

**`sec3_p21_booth.py`** (numerical, single sphere entered as two equal-$D$ layers, $R = 175\ \mu\mathrm{m}$, $D = 10^{-15}\ \mathrm{m^2\ s^{-1}}$, 1000 modes; synthetic): `SUMMARY: 20/20 PASS`.

```text
PASS  P21.24 eigenvalues = n^2 pi^2 D/R^2 (n <= 1000)  max rel err 1.3e-15
PASS  P21.47 weights w_n = 6/(n^2 pi^2)  max rel err 5.9e-11
PASS  P21.48 modal R/B(t) vs TECDOC-1674 (7.7), lambda/D' = 1.0  max abs diff 2.1e-11
PASS  P21.48 modal F_B(t) vs TECDOC-1674 (7.8), lambda/D' = 1.0  max abs diff 2.1e-08, within the N = 1000 truncation bound (ratio 1.00)
PASS  P21.49 steady R/B = 1 - lambda sum w_n/k_n vs (7.6), lambda/D' = 1.0  0.9391058565 vs 0.9391058565 (truncation bound 2.1e-11)
PASS  P21.48 F_B equals M_out/(M + M_out)  0.773329025 vs 0.773329025
PASS  P21.39 direct flux series error ~ N^-1  observed order 0.99
PASS  P21.39 balance form error ~ N^-3  observed order 2.94 (N <= 100, before round-off)
```

The $F_B$ differences of $2\times10^{-8}$ occur at $D't = 10^{-3}$ and equal the analytic truncation bound $2/(\pi^4N^3D't)$ of the $M/G$ series (the script calls it `M/P`), so they come from truncation alone. The check $M + M_\mathrm{out}$ is an identity for the truncated balance-form flux, so it tests the convolution (P21.43) and the quadrature rather than the series.

**`sec3_p21_fv_two_layer.py`** (independent FV against the modal solution, two layers). Data (synthetic): $R_1 = 175\ \mu\mathrm{m}$, $R_2 = 390\ \mu\mathrm{m}$, $D = (2\times10^{-15},\ 5\times10^{-16})\ \mathrm{m^2\ s^{-1}}$, $\lambda = 10^{-8}\ \mathrm{s^{-1}}$, $S_0 = 10^{-6}\ \mathrm{mol\ m^{-3}\ s^{-1}}$ in the kernel for $t < t_\mathrm{irr} = 10^8\ \mathrm{s}$, zero after; 400 modes. The FV check is **discretised** (cell-centred, Crank-Nicolson, $\theta = 1/2$), unlike the Rust Backward Euler solver:

$$
\frac{V_j}{\Delta t}\left(C_j^{n+1} - C_j^n\right) = -\sum_fG_f\,\Delta_fC^{n+\theta} - \lambda V_jC_j^{n+\theta} + V_jS_j^{n+1/2},\qquad C^{n+\theta} = \theta C^{n+1} + (1-\theta)C^n
$$

where $\Delta_fC = C_j - C_{j'}$ is the difference between cell $j$ and its neighbour $j'$ across face $f$ (with $C = 0$ beyond $r = R$), $G_f = A_f/(\delta_-/D_- + \delta_+/D_+)$ at internal faces with $\delta_\pm$ the distances from the two nodes to the face, $G_R = A_RD_5/\delta$ at the surface with $\delta = R - r_N$ ($r_N$ the radius of the last node). The Python nodes sit at cell midpoints, while the Rust code (`finite_volume.rs`) places them at shell volume centroids; both are second order in space, and the independent check below uses centroids. The scheme also uses release increment $\Delta tG_RC_N^{n+\theta}$ and decay increment $\lambda\Delta t\sum_jV_jC_j^{n+\theta}$. Output (`SUMMARY: 14/14 PASS`):

```text
cells/layer          M      M_rel      M_dec        J_R    closure
         20   9.77e-04   2.43e-03   1.96e-04   8.39e-04    2.4e-14
         40   2.44e-04   6.09e-04   4.92e-05   2.10e-04    1.5e-14
         80   6.11e-05   1.52e-04   1.23e-05   5.25e-05    5.6e-14
        160   1.53e-05   3.81e-05   3.08e-06   1.31e-05    5.7e-14
PASS  P21.28 FV -> modal for M, observed order  order 2.00, finest rel diff 1.5e-05
PASS  P21.28 FV -> modal for Mrel, observed order  order 2.00, finest rel diff 3.8e-05
PASS  P21.28 FV -> modal for JR, observed order  order 2.00, finest rel diff 1.3e-05
Modal values at t_irr: M = 6.941502e-10 mol, M_rel = 1.019337e-09 mol, M_dec = 5.314425e-10 mol, R/B = 0.671485
Steady R/B (balance form) = 0.68638073; direct sum over 400 modes = 0.68638922
PASS  P21.29 general initial profile with decay: FV -> modal  rel diff 9.4e-05, 2.3e-05, 5.8e-06; order 2.00
```

The table gives the maximum relative difference over $t \in \{2, 5, 10, 15, 30\}\times10^7\ \mathrm{s}$, which spans irradiation, the switch-off and the post-irradiation phase; the differences fall at second order, so they are discretisation error. The FV closure (P21.42) holds to $6\times10^{-14}$.

**`sec3_p21_chain_overlap.py`** (P21.19 to P21.22; parent $D = (2\times10^{-15},\ 5\times10^{-16})$, daughter $D = (10^{-15},\ 4\times10^{-15})\ \mathrm{m^2\ s^{-1}}$, $\lambda_P = 3\times10^{-8}$, $\lambda_D = 10^{-8}\ \mathrm{s^{-1}}$, 80 modes each; synthetic): `SUMMARY: 5/5 PASS`.

```text
PASS  P21.22 O = I when daughter D equals parent D (same element)  max |O - I| = 5.4e-14
PASS  P21.22 |O| = I when D_D = 3 D_P in every layer (Lambda scales by 3)  max ||O| - I| = 5.1e-14
PASS  P21.19 O is a full matrix when the D ratios differ  max off-diagonal |O_nm| = 4.214, O_11 = 2.3122, O_21 = -1.6006
PASS  P21.21 coupled modal daughter inventory = two-species FV (converging)  finest rel diff 2.2e-05, order 2.00
PASS  P21.20 dropping the off-diagonal overlap terms gives a visible error  3.7e-01 vs 2.2e-05
```

Keeping only the diagonal of $O$ gives a 37 % error in the daughter inventory, so the coupling cannot be dropped when the daughter's layer ratios differ.

**Further verification opportunities for the Rust v2.0 code.** (i) Reproduce the Booth table above with STRATO B and C in one-layer mode; (ii) run the two-layer case above in both codes and require second-order convergence of the Rust FV with Crank-Nicolson, or first order in time with Backward Euler; (iii) assert (P21.42) every step; (iv) assert $F_\mathrm{corr}(t)$ of a source-free run equals the v1.0 $F(t)$ for any $\lambda$; (v) compare $(R/B)_\mathrm{ss}$ from (P21.49) with a long FV run.

### 3.10 Open issues

- Capture cross sections, the Ag-110m isomeric fraction, product burn-out cross sections and flux histories are NOT VERIFIED (Section 3.3.3); activation products cannot be run quantitatively until they are sourced.
- Kernel heavy-metal density for (P21.9) is not taken from a verified source in this section.
- Diffusivities of daughter elements (Y, Ba) in each layer are not in the audited data; chains stay future work.
- Completeness of the layered eigenfunction set (assumed in Section 4.5 of the Phase 1 derivation, `docs/mathematics/phase1/TRISO_Finite_Volume_Derivation.md`) follows from a Green-function argument added in the B3 reconciliation. For $-r^{-2}(r^2Dc')' = f$ with $c(R) = 0$ and a regular centre, integrating twice gives $c(r) = \int_0^R G(r,s)f(s)s^2\,ds$ with $G(r,s) = g(\max(r,s))$ and $g(x) = \int_x^R d\rho/(\rho^2D(\rho))$. This kernel is symmetric, the operator is injective, and $\int_0^R\int_0^R G^2r^2s^2\,dr\,ds = \tfrac23\int_0^R g(x)^2x^5\,dx$ is finite because $g(x) \le C/x$ near the centre, so the operator is compact and self-adjoint on $L^2(r^2dr)$. Its eigenfunctions, which are those of (P21.24) with $\Lambda_n$ the reciprocal eigenvalues, therefore form a complete orthogonal set by the Hilbert-Schmidt theorem (derived; no textbook citation was checked). The script `b3_completeness.py` confirms the Green function against the exact profile (P21.82) in exact arithmetic and evaluates the Hilbert-Schmidt norm for five layers. Numerical support: $\sum w_n \to 1$, the projection of a smooth $c_0$ reproducing $M_0$ to $3\times10^{-9}$, and second-order FV agreement.
- Fission-fragment recoil into the buffer and noble-gas precursor migration are neglected (A5, Section 3.3.2).
- Which decay constants the CRP-6 participants used is NOT VERIFIED (audit 04, Section 2.4).

## 4 Part 2.1, finite volume: formulation with decay and source, and its verification design

This section extends Code B of STRATO v1.0 (`src/numerical/finite_volume.rs`) to a single nuclide $k$ with radioactive decay and a volumetric source. It fixes the discrete equations, states exactly which entries of the v1.0 tridiagonal system change, proves that positivity, the subtraction-free elimination and inventory closure survive, compares the implicit scheme with a decay-exact alternative, gives step-size rules, and designs the verification matrix. Every derived formula below was checked by one of six scripts in `scripts/derivation_checks/` (Section 4.9). The Phase 1 equations are cited by the labels of the delivered derivation `docs/mathematics/phase1/TRISO_Finite_Volume_Derivation.md` (for example TRISO-DEL-040). Equations of this section are numbered (P21.50) to (P21.92).

Status labels used: **exact** (an identity, no truncation), **semi-analytical** (exact up to a finite linear solve or root find), **series** (exact infinite sum, truncated in practice), **approximate** (a stated modelling or discretisation error remains), **numerical** (a computed number).

### 4.0 Symbols and units

| Symbol | Meaning | Units |
|---|---|---|
| $r,\ t$ | radius, time | $\mathrm{m}$, $\mathrm{s}$ |
| $i=1,\dots,5$ | layer index: kernel, buffer, IPyC, SiC, OPyC | 1 |
| $R_i$ | outer radius of layer $i$; $R_0=0$, $R_5=R$ | $\mathrm{m}$ |
| $D_i$ | diffusivity of layer $i$ (constant in this section) | $\mathrm{m^2\ s^{-1}}$ |
| $c(r,t)$ | concentration of nuclide $k$ | $\mathrm{mol\ m^{-3}}$ |
| $\lambda_k$, written $\lambda$ | decay constant, $\lambda=\ln 2/t_{1/2}$ | $\mathrm{s^{-1}}$ |
| $S(r,t)$ | volumetric source | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $S_K(t)$ | uniform kernel source value | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $\dot F$ | fission rate density | $\mathrm{fissions\ m^{-3}\ s^{-1}}$ |
| $y^I,\ y^C$ | independent and cumulative fission yield | $\mathrm{per\ fission}$ |
| $N_A$ | Avogadro constant | $\mathrm{mol^{-1}}$ |
| $j=1,\dots,N$ | cell index, cell $j$ spans $[r_{j-1/2},r_{j+1/2}]$ | 1 |
| $r_j$ | volume centroid of cell $j$, $\tfrac34(r_{j+1/2}^4-r_{j-1/2}^4)/(r_{j+1/2}^3-r_{j-1/2}^3)$ (`volume_centroid` in v1.0) | $\mathrm{m}$ |
| $V_K$ | kernel volume $\tfrac{4\pi}{3}R_1^3$ | $\mathrm{m^3}$ |
| $V_j$ | shell volume $\tfrac{4\pi}{3}(r_{j+1/2}^3-r_{j-1/2}^3)$ | $\mathrm{m^3}$ |
| $A_f$ | face area $4\pi r_f^2$ | $\mathrm{m^2}$ |
| $G_f$; $G_W,\ G_E$ | face conductance (TRISO-FV-140); west and east face of a cell | $\mathrm{m^3\ s^{-1}}$ |
| $G_R$ | surface conductance $4\pi R^2D_5/(R-r_N)$ (TRISO-DEL-030) | $\mathrm{m^3\ s^{-1}}$ |
| $g^W_j,\ g^E_j$ | $G_W,\ G_E$ of cell $j$; $g^W_1=0$, $g^E_N=G_R$, $g^E_j=g^W_{j+1}$ | $\mathrm{m^3\ s^{-1}}$ |
| $C_j^n$ | cell-average concentration at $t^n$ | $\mathrm{mol\ m^{-3}}$ |
| $\bar S_j(t)$, $\bar S_j^{n+1}$ | cell-averaged source; its value used in step $n\to n+1$ | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $\Delta t$, $z=\lambda\Delta t$ | time step; dimensionless decay per step | $\mathrm{s}$, 1 |
| $v_j$ | diagonal excess of row $j$ (Section 4.3) | $\mathrm{m^3\ s^{-1}}$ |
| $\mathbf V,\ \mathbf K$ | $\mathrm{diag}(V_j)$; conductance (stiffness) matrix with row sums $0$ except $G_R$ in row $N$ | $\mathrm{m^3}$, $\mathrm{m^3\ s^{-1}}$ |
| $M,\ M_0$ | inventory inside the particle, initial inventory | $\mathrm{mol}$ |
| $M_{gen},\ M_{rel},\ M_{dec},\ M_{out}$ | generated, released through $r=R$, decayed inside, released and not yet decayed outside (cumulative) | $\mathrm{mol}$ |
| $M_{dec,out}$ | released atoms that decayed outside | $\mathrm{mol}$ |
| $\mathrm{Gen}(t)$ | generation rate $\int S\,dV$ | $\mathrm{mol\ s^{-1}}$ |
| $J_R(t)$ | release rate $-4\pi R^2D_5\,\partial_rc\vert_{R}$ | $\mathrm{mol\ s^{-1}}$ |
| $\Lambda_n,\ \phi_n,\ N_n,\ w_n$ | Phase 1 eigenvalues, eigenfunctions, norms $\int_0^Rr^2\phi_n^2dr$, release weights (TRISO-DEL-021) | $\mathrm{s^{-1}}$, $\mathrm{mol\ m^{-3}}$, $\mathrm{m^3\,(mol\ m^{-3})^2}$, 1 |
| $u=rc$ | transformed concentration | $\mathrm{mol\ m^{-2}}$ |
| $k_i=\sqrt{\lambda/D_i}$ | decay length inverse in layer $i$ | $\mathrm{m^{-1}}$ |
| $D'=D/R^2$ | reduced diffusivity (Booth) | $\mathrm{s^{-1}}$ |
| $R/B$ | release rate over birth rate, $J_R/\mathrm{Gen}$ | 1 |
| $\tau_{res}$ | mean residence time $M_{ss}/\mathrm{Gen}$ | $\mathrm{s}$ |
| $T$, $R_g$ | temperature, gas constant | $\mathrm{K}$, $\mathrm{J\ mol^{-1}\ K^{-1}}$ |

Table: 4.1 Symbols and units of the finite-volume formulation.

### 4.1 Continuous problem

Assumptions for this section: spherical symmetry; constant $D_i$ per layer; one nuclide with uniform $\lambda\ge0$ (the decay constant is a nuclear property, so it is the same in every layer); perfect interfaces without partition coefficients (as v1.0, and as FRESCO-II Eq. 10.4b in IAEA, 2012, printed p. 531); absorbing outer surface (free evaporation, IAEA, 2012, Eq. 10.4d and Sec. 10.2.4, printed pp. 531, 538-539); precursors are not tracked.

$$
\frac{\partial c_i}{\partial t}=\frac{1}{r^2}\frac{\partial}{\partial r}\!\left(r^2D_i\frac{\partial c_i}{\partial r}\right)-\lambda c_i+S(r,t),\qquad R_{i-1}<r<R_i \tag{P21.50}
$$

$$
\partial_rc_1(0,t)=0,\quad c_i(R_i,t)=c_{i+1}(R_i,t),\quad D_i\partial_rc_i\big\vert_{R_i^-}=D_{i+1}\partial_rc_{i+1}\big\vert_{R_i^+},\quad c_5(R,t)=0,\quad c(r,0)=c^0(r)\tag{P21.51}
$$

(P21.50) is TRISO-FV-100 with the decay term added; it coincides with the particle equations of FRESCO-II (Eq. 10.3b) and NRCDIF (Eq. 10.6) in IAEA (2012, printed pp. 531, 535-536). Eq. (10.1) of the same report prints a minus sign in front of the divergence; that is a sign typo in the source (audit 04, Sec. 1).

**Kernel source (approximate model).**

$$
S(r,t)=S_K(t)\,H(R_1-r),\qquad S_K(t)=\frac{y\,\dot F(t)}{N_A}\tag{P21.52}
$$

Units: $(\mathrm{per\ fission})(\mathrm{fissions\ m^{-3}\ s^{-1}})/(\mathrm{mol^{-1}})=\mathrm{mol\ m^{-3}\ s^{-1}}$. Use $y=y^C$ when every precursor is short lived compared with the step and diffusion times and is assumed to decay where it was born; use $y=y^I$ only when the precursors are carried as separate species (parent-daughter chains, Section 3.3.4, FUTURE work beyond v2.0). For Cs-137 the chain precursors I-137 (24.5 s) and Xe-137 (3.818 min) are short (audit 07, Sec. 4), so $y^C$ is the natural choice; this neglects any migration of the gaseous Xe-137 before it decays, so the source location is approximate. Recoil and the radial burnup profile are neglected (uniform $S$ in the kernel).

### 4.2 (a) Cell balance with decay and source

Multiply (P21.50) by $4\pi r^2$ and integrate over cell $j$. The divergence term gives the two face flows exactly (as in TRISO-FV-123). Because $\lambda$ is uniform, $\int\lambda c\,4\pi r^2dr=\lambda V_jC_j$ with $C_j=V_j^{-1}\int c\,4\pi r^2dr$: the decay term needs no approximation. The source enters through its cell average

$$
\bar S_j(t)=\frac{1}{V_j}\int_{r_{j-1/2}}^{r_{j+1/2}}S(r,t)\,4\pi r^2\,dr. \tag{P21.53}
$$

**Exact cell average of the uniform kernel source.** For (P21.52),

$$
\bar S_j(t)=S_K(t)\times\begin{cases}1,& r_{j+1/2}\le R_1\\[1mm] \dfrac{R_1^3-r_{j-1/2}^3}{r_{j+1/2}^3-r_{j-1/2}^3},& r_{j-1/2}<R_1<r_{j+1/2}\\[2mm] 0,& r_{j-1/2}\ge R_1\end{cases}\tag{P21.54}
$$

(exact; middle case checked in X8). The v1.0 mesh puts every $R_i$ on a face (`build_mesh`, test `interfaces_are_faces`), so the straddling case never occurs: $\bar S_j=S_K$ for cells of material 0 and $0$ elsewhere, and $\mathrm{Gen}=\sum_jV_j\bar S_j=\tfrac{4\pi}{3}R_1^3S_K$ exactly because the kernel cell volumes telescope. The centroid point value $S(r_j)$ would give the same numbers for a uniform source but is only second-order accurate for a non-uniform one, so the cell average (P21.53) is the definition to implement.

**Semi-discrete equation (exact balance, approximate fluxes).** With the two-point face flows of TRISO-FV-140:

$$
V_j\frac{dC_j}{dt}=G_W(C_{j-1}-C_j)+G_E(C_{j+1}-C_j)-\lambda V_jC_j+V_j\bar S_j(t),\qquad\text{in matrix form}\quad \mathbf V\dot{\mathbf C}=-\mathbf K\mathbf C-\lambda\mathbf V\mathbf C+\mathbf V\bar{\mathbf S}(t). \tag{P21.55}
$$

Integrating (P21.55) over one step is still exact:

$$
V_j\big[C_j(t^{n+1})-C_j(t^n)\big]=\int_{t^n}^{t^{n+1}}\!\Big[G_W(C_{j-1}-C_j)+G_E(C_{j+1}-C_j)-\lambda V_jC_j\Big]dt+V_j\int_{t^n}^{t^{n+1}}\bar S_j\,dt. \tag{P21.56}
$$

### 4.3 (b) Backward Euler with implicit decay

**Time quadrature.** Backward Euler (BE) replaces the first integral of (P21.56) by $\Delta t$ times its value at $t^{n+1}$ (first order). For the source integral two choices exist:

$$
\text{(R) rectangle: }\ \bar S_j^{n+1}=\bar S_j(t^{n+1-}),\qquad \text{(A) exact step average: }\ \bar S_j^{n+1}=\frac{1}{\Delta t}\int_{t^n}^{t^{n+1}}\bar S_j(t)\,dt. \tag{P21.57}
$$

In (R) the value is the left limit at $t^{n+1}$, so a source defined on half-open intervals $(t_a,t_b]$ is sampled inside the step. For a smooth source the generation error of (R) per step is $\int\bar S\,dt-\Delta t\,\bar S(t^{n+1})=-\tfrac12\Delta t^2\dot{\bar S}-\tfrac13\Delta t^3\ddot{\bar S}+O(\Delta t^4)$ with the derivatives taken at $t^n$ (X7; expanded about $t^{n+1}$ the cubic coefficient is $+\tfrac16$ instead), which sums to $O(\Delta t)$ over a run. (A) has no generation error at all; for a piecewise-constant or piecewise-linear $\dot F(t)$ it is a closed form. Both give a scheme of global order 1, because BE itself is first order. **Recommendation: (A).** It makes $M_{gen}$ equal to the true generated amount even when a switch falls inside a step (N9: $+1.6\times10^{-16}$ relative error against $-0.167$ for (R) in the test case), and it equals (R) when the source is constant on every step.

**The BE system.**

$$
\Big(\frac{V_j}{\Delta t}+\lambda V_j+G_W+G_E\Big)C_j^{n+1}-G_WC_{j-1}^{n+1}-G_EC_{j+1}^{n+1}=\frac{V_j}{\Delta t}C_j^n+V_j\bar S_j^{n+1}. \tag{P21.58}
$$

**What changes relative to v1.0.** Write $a_{jk}$ for the matrix entries and $b_j$ for the right-hand side.

$$
a_{jj}=v_j+g^W_j+g^E_j,\quad v_j=V_j\Big(\frac{1}{\Delta t}+\lambda\Big),\qquad a_{j,j-1}=-g^W_j,\quad a_{j,j+1}=-g^E_j,\qquad b_j=\frac{V_j}{\Delta t}C_j^n+V_j\bar S_j^{n+1}. \tag{P21.59}
$$

| Item | v1.0 (`backward_euler_step`, [src/numerical/finite_volume.rs, lines 162 to 182](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L162-L182); assembly loop [src/numerical/finite_volume.rs, lines 171 to 180](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L171-L180)) | Phase 2.1 |
|---|---|---|
| $v_j$ passed to `solve_diffusion_system` | `v[i] = volume / dt` | `volume / dt + lambda * volume` |
| $g^W_j$, $g^E_j$ | `conductance`, `outer_conductance` | unchanged |
| $b_j$ | `rhs[i] = v[i] * old[i]` | `(volume / dt) * old[i] + volume * sbar[i]` |
| solver | subtraction-free elimination | unchanged |
| diagnostics in `run` | $M$, $M_{rel}$, residual $M+M_{rel}-M_0$ | add $M_{gen}$, $M_{dec}$, $M_{out}$; residual (P21.72) |

Table: 4.2 Changes to the v1.0 Backward Euler step.

Two implementation details matter. (i) After $v_j$ is redefined, the right-hand side must keep $V_j/\Delta t$: reusing the augmented `v[i]` as in v1.0 [src/numerical/finite_volume.rs, line 179](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L179) would silently multiply the old inventory by $1+z$. (ii) To make the $\lambda=0,\ S=0$ run reproduce v1.0 bit for bit, compute $v_j$ as `V/dt + lambda*V` and $b_j$ as `(V/dt)*old + V*sbar`: adding an exact $0.0$ leaves every float unchanged, while the algebraically equal `V*(1/dt + lambda)` differs from `V/dt` in the last bit (N5 confirms bitwise equality over 200 steps with the first form). The repository code was not edited; the table is a specification.

**Diagonal excess and M-matrix property.** Off-diagonal entries are $\le0$ and

$$
a_{jj}-\sum_{k\ne j}\lvert a_{jk}\rvert=v_j=V_j\Big(\frac1{\Delta t}+\lambda\Big)\ (j<N),\qquad a_{NN}-\lvert a_{N,N-1}\rvert=v_N+G_R. \tag{P21.60}
$$

(exact, X2). Decay therefore enters only as an increase of the diagonal excess $v_j$, from $V_j/\Delta t$ to $V_j(1/\Delta t+\lambda)$. Physically, $\lambda V_j$ acts like an extra conductance from cell $j$ to a reservoir held at zero concentration. The matrix stays a strictly row-diagonally dominant Z-matrix with positive diagonal, hence a nonsingular M-matrix; the direct proof below does not need that general theory.

**The subtraction-free elimination remains valid.** Gaussian elimination without pivoting has pivots $d_1=a_{11}$ and $d_j=a_{jj}-a_{j,j-1}a_{j-1,j}/d_{j-1}=a_{jj}-(g^W_j)^2/d_{j-1}$, since $g^E_{j-1}=g^W_j$. Claim:

$$
d_j=e_j+g^E_j,\qquad e_1=v_1,\qquad e_j=v_j+\frac{g^W_j\,e_{j-1}}{e_{j-1}+g^W_j}\quad(j\ge2). \tag{P21.61}
$$

Proof by induction. If $d_{j-1}=e_{j-1}+g^E_{j-1}=e_{j-1}+g^W_j$, then

$$
d_j=v_j+g^W_j+g^E_j-\frac{(g^W_j)^2}{e_{j-1}+g^W_j}=v_j+g^E_j+g^W_j\,\frac{(e_{j-1}+g^W_j)-g^W_j}{e_{j-1}+g^W_j}=v_j+g^E_j+\frac{g^W_j\,e_{j-1}}{e_{j-1}+g^W_j}.
$$

The subtraction in the middle step is carried out analytically, so the recurrence that the code evaluates contains only sums, products and quotients of nonnegative numbers (exact; X1 checks the induction step and $N=5$ explicitly). Nothing in the proof uses the value of $v_j$ beyond $v_j>0$, so the v1.0 routine `solve_diffusion_system` applies unchanged with the new $v_j$. Moreover $e_j\ge v_j$ and $d_j\ge v_j+g^E_j>0$: decay can only make the pivots larger.

**Positivity.** With $y_j=(b_j+g^W_jy_{j-1})/d_j$ and $C_j^{n+1}=y_j+(g^E_j/d_j)\,C_{j+1}^{n+1}$, $C_N^{n+1}=y_N$:

$$
C^n\ge0,\ \bar S^{n+1}\ge0\ \Longrightarrow\ b\ge0\ \Longrightarrow\ y\ge0\ \Longrightarrow\ C^{n+1}\ge0\quad\text{for every }\Delta t>0,\ \lambda\ge0. \tag{P21.62}
$$

(exact). Rounding a nonnegative sum, product or quotient of nonnegative floats gives a nonnegative float, so (P21.62) also holds in floating point: the minimum is $\ge0.0$ exactly, a sharper test than the v1.0 threshold of $-10^{-13}$ (N2: smallest value $1.1\times10^{-128}$ for $\Delta t=10^{-3}$ to $10^{12}\ \mathrm{s}$). A source with negative values (possible in a manufactured solution) voids (P21.62).

**Exact link to the v1.0 scheme.** Since $\mathbf V/\Delta t+\lambda\mathbf V=\mathbf V/\Delta t'$ with $\Delta t'=\Delta t/(1+z)$ and $\mathbf V/\Delta t=(1+z)^{-1}\mathbf V/\Delta t'$, a step of (P21.58) with $S=0$ is

$$
\mathbf C^{n+1}=(1+z)^{-1}\,\mathbf B_{\Delta t'}\mathbf C^n,\qquad \mathbf B_{h}=(\mathbf V/h+\mathbf K)^{-1}\mathbf V/h\ \ (\text{the v1.0 step}),\qquad \mathbf C^n=(1+z)^{-n}\mathbf B_{\Delta t'}^{\,n}\mathbf C^0. \tag{P21.63}
$$

(exact discrete identity; X3 symbolic, N4 numeric to $7.6\times10^{-16}$). It is the discrete analogue of the continuous identity (P21.80) and gives a regression test that ties a decay run to a v1.0 run with a modified step.

**Decay alone.** With all $G=0$, (P21.58) is $C^{n+1}=(C^n+\Delta t\,\bar S^{n+1})/(1+z)$. The factor $(1+z)^{-1}$ lies in $(0,1)$ for every $z>0$ and tends to $0$ as $z\to\infty$ (unconditionally stable, positive, L-stable). Against the exact factor:

$$
e^{-z}-\frac{1}{1+z}=-\frac{z^2}{2}+O(z^3). \tag{P21.64}
$$

### 4.4 (c) Decay-exact alternative (integrating factor, operator split)

Because $\lambda\mathbf I$ commutes with $\mathbf V^{-1}\mathbf K$, the substitution $\mathbf C(t)=e^{-\lambda(t-t^n)}\tilde{\mathbf C}(t)$ removes decay from (P21.55) exactly:

$$
\mathbf V\frac{d\tilde{\mathbf C}}{dt}=-\mathbf K\tilde{\mathbf C}+e^{\lambda(t-t^n)}\mathbf V\bar{\mathbf S}(t),\qquad t^n\le t\le t^{n+1}. \tag{P21.65}
$$

Applying BE to $\tilde{\mathbf C}$ with the exact weighted source integral gives the decay-exact scheme

$$
\Big(\frac{\mathbf V}{\Delta t}+\mathbf K\Big)\tilde{\mathbf C}^{n+1}=\frac{\mathbf V}{\Delta t}\mathbf C^n+\mathbf V\bar{\mathbf S}_\varphi^{n+1},\quad \bar{\mathbf S}_\varphi^{n+1}=\frac1{\Delta t}\int_{t^n}^{t^{n+1}}e^{\lambda(t-t^n)}\bar{\mathbf S}\,dt\ \Big(=\bar{\mathbf S}\,\frac{e^z-1}{z}\ \text{if constant}\Big),\quad \mathbf C^{n+1}=e^{-z}\tilde{\mathbf C}^{n+1}. \tag{P21.66}
$$

With $S=0$ this is the Lie split "v1.0 diffusion step, then exact decay", and because the two operators commute the split introduces no error:

$$
\mathbf C^n_{split}=e^{-\lambda t^n}\,\mathbf B_{\Delta t}^{\,n}\mathbf C^0,\qquad e^{-(\mathbf V^{-1}\mathbf K+\lambda\mathbf I)t}=e^{-\lambda t}e^{-\mathbf V^{-1}\mathbf K t}. \tag{P21.67}
$$

(exact; N3a checks the semi-discrete identity to $2.6\times10^{-13}$.)

**Steady state, the decisive difference.** For a constant source, the fixed point of the unsplit BE (P21.58) satisfies $(\mathbf K+\lambda\mathbf V)\mathbf C=\mathbf V\bar{\mathbf S}$, because the $\mathbf V/\Delta t$ terms cancel when $\mathbf C^{n+1}=\mathbf C^n$: it is exactly the semi-discrete steady state for every $\Delta t$. The fixed point of (P21.66) is different (X4):

$$
\big(\varphi(z)\,\mathbf K+\lambda\mathbf V\big)\mathbf C=\mathbf V\bar{\mathbf S},\qquad \varphi(z)=\frac{z\,e^z}{e^z-1}=1+\frac z2+\frac{z^2}{12}+O(z^4). \tag{P21.68}
$$

The decay-exact scheme therefore converges to a steady state whose diffusion is accelerated by $\varphi(z)$. In the five-layer test (N8) its steady $R/B$ (taken from its own balance, $1-\lambda M/\mathrm{Gen}$) is off by $+0.15\,\%$ for a long-lived nuclide at $z=0.1$, and by $+10\,\%$ ($z=0.1$) and $+127\,\%$ ($z=1$) for a short-lived nuclide with $\lambda\approx10\Lambda_1$, whereas the unsplit BE reproduces the semi-discrete $R/B$ to $5.6\times10^{-14}$ at $z=0.1$, $1$ and $5$. These percentages depend on how $R/B$ is read from the split scheme: taken as $G_RC_N/\mathrm{Gen}$ at its fixed point, the biases are $-4.7\,\%$ and $-36\,\%$ (long-lived, $z=0.1$ and $1$) and $+4.9\,\%$ and $+44\,\%$ (short-lived), and by (P21.68) the two readings differ by the factor $\varphi(z)$. Either way the split fixed point is wrong, which is the conclusion that matters. Any split that applies exact decay after a BE diffusion-plus-source step has a similar shifted fixed point.

**Error orders (numerical evidence N3, N6).**

$$
\text{per mode, relative: unsplit BE }\ \varepsilon_n\approx\tfrac12(\Lambda_n+\lambda)^2t\,\Delta t;\qquad \text{decay-exact split }\ \varepsilon_n\approx\tfrac12\Lambda_n^2t\,\Delta t;\qquad \text{difference }\tfrac12(2\Lambda_n\lambda+\lambda^2)\,t\,\Delta t. \tag{P21.69}
$$

(approximate, leading order in $\Delta t$; from $n\ln[1+(\Lambda_n+\lambda)\Delta t]-(\Lambda_n+\lambda)t$ with $t=n\Delta t$, checker script S8c.) Both schemes are $O(\Delta t)$. The extra error of the unsplit scheme contains the cross term $\Lambda_n\lambda$, so it is not the pure-decay error $\tfrac12\lambda^2t\Delta t$ of (P21.77) added to the diffusion error. In N3 ($\lambda=\Lambda_1$, $t=1/\Lambda_1$) the relative error in $M(t)$ is $2.50\times10^{-3}$ (unsplit) against $6.2\times10^{-4}$ (split) at 800 steps, a ratio of 4 as (P21.69) predicts for the dominant first mode ($\tfrac12(2\Lambda_1)^2t\Delta t=2/800$ against $\tfrac12\Lambda_1^2t\Delta t=0.5/800$); both have observed order 0.999.

**Choice.** Use the unsplit BE (P21.58) in STRATO. Reasons: it is a two-line change to v1.0; it keeps positivity, the subtraction-free solver and bitwise regression; its closure (P21.72) has a decayed term $\Delta t\,\lambda M^{n+1}$ with a direct physical meaning; and it preserves the steady state exactly, which is what $R/B$ comparisons for short-lived nuclides need. Keep (P21.66) with $S=0$ as a cross-check, through (P21.67). Crank-Nicolson would raise the order but its mode factor $(1-x/2)/(1+x/2)$, $x=(\Lambda+\lambda)\Delta t$, is negative for $x>2$, so it gives up unconditional positivity; second-order positive alternatives (for example TR-BDF2) are outside this section.

### 4.5 (d) Discrete inventory closure

**Per-step identity.** Sum (P21.58) over $j$. An internal face $f$ between cells $j$ and $j+1$ contributes $G_f(C_j-C_{j+1})$ to row $j$ and $G_f(C_{j+1}-C_j)$ to row $j+1$, which cancel; the centre face has $g^W_1=0$; the outer face leaves $G_RC_N$. With $M^n=\sum_jV_jC_j^n$:

$$
M^{n+1}-M^n=\Delta t\big(\mathrm{Gen}^{n+1}-\lambda M^{n+1}-J_R^{n+1}\big) \tag{P21.70}
$$

$$
\mathrm{Gen}^{n+1}=\sum_jV_j\bar S_j^{n+1},\qquad J_R^{n+1}=G_RC_N^{n+1}. \tag{P21.71}
$$

(exact for the exact solution of the linear system; it is the BE image of $dM/dt=\mathrm{Gen}-\lambda M-J_R$.) Summing over steps $k=1,\dots,n$ (step sizes may vary):

$$
M_0+M_{gen}^n=M^n+M_{dec}^n+M_{rel}^n,\qquad M_{gen}^n=\sum_{k=1}^n\Delta t_k\,\mathrm{Gen}^k,\quad M_{dec}^n=\sum_{k=1}^n\Delta t_k\,\lambda M^k,\quad M_{rel}^n=\sum_{k=1}^n\Delta t_k\,G_RC_N^k. \tag{P21.72}
$$

In floating point the residual of (P21.72) is the accumulated linear-solver residual plus summation rounding; compensated (Kahan) summation, already used in v1.0 for $M_{rel}$, should be used for all three sums. N1 gives $1.7\times10^{-15}$ per step and $1.2\times10^{-16}$ cumulative relative residual over 71 variable steps with a switching source. The residual tests assembly and solver; it stays at round-off for any $\Delta t$, so it says nothing about time accuracy.

**Atoms outside the particle.** Released atoms keep decaying: $dM_{out}/dt=J_R-\lambda M_{out}$. The BE update consistent with (P21.58) is

$$
M_{out}^{n+1}=\frac{M_{out}^n+\Delta t\,J_R^{n+1}}{1+\lambda\Delta t}, \tag{P21.73}
$$

and the exponential update, exact when $J_R$ is constant on the step at $J_R^{n+1}$, is

$$
M_{out}^{n+1}=e^{-\lambda\Delta t}M_{out}^n+J_R^{n+1}\,\frac{1-e^{-\lambda\Delta t}}{\lambda}. \tag{P21.74}
$$

With (P21.73), $M_{out}^{n+1}-M_{out}^n=\Delta t\,(J_R^{n+1}-\lambda M_{out}^{n+1})$, and summation gives

$$
M_{rel}^n=M_{out}^n+M_{dec,out}^n,\quad M_{dec,out}^n=\sum_k\Delta t_k\,\lambda M_{out}^k,\qquad M_0+M_{gen}^n=M^n+M_{out}^n+M_{dec}^n+M_{dec,out}^n. \tag{P21.75}
$$

(exact; N7: $1.3\times10^{-14}$ and $8.5\times10^{-15}$.) Both updates have the fixed point

$$
M_{out}\to J_R/\lambda\quad(\text{constant }J_R),\qquad \lambda\to0:\ M_{out}^n=M_{rel}^n. \tag{P21.76}
$$

(X6a.) The BE update has local error $\tfrac12\Delta t^2\lambda(\lambda M_{out}-J_R)$ (X6c); the exponential one has none for step-constant $J_R$. Consistency argument for (P21.73): it applies the same decay rule inside and outside, so the whole-system decayed amount is $\sum_k\Delta t_k\lambda(M^k+M_{out}^k)$ and (P21.75) closes with intrinsically defined terms. With (P21.74) the outside closure holds only if $M_{dec,out}$ is defined as the remainder $M_{out}^k+\Delta tJ_R^{k+1}-M_{out}^{k+1}$. Recommended: (P21.73) for the closure, (P21.74) as the reported $M_{out}$ only if an outside-decay accuracy better than $O(\Delta t)$ is wanted. In N7 the two differ by $3.8\times10^{-9}$ relative after 16 mean lives; this is a transient remainder, since at the exact fixed point $J_R/\lambda$ they coincide (X6a).

### 4.6 (e) Temporal accuracy and step-size guidance

**Decay error.** From (P21.64), $\ln[(1+z)^{-n}e^{nz}]=n(z^2/2-z^3/3+\dots)$ (X5); with $nz=\lambda t$ the relative error of the BE decay factor is

$$
\varepsilon_\lambda(t)\approx\tfrac12\lambda^2t\,\Delta t\quad\Longrightarrow\quad \Delta t\le\frac{2\varepsilon_\lambda}{\lambda^2t_{end}}. \tag{P21.77}
$$

(approximate, leading order; N6 observes $3.12\times10^{-3}$ against the predicted $3.13\times10^{-3}$.) Examples (calculated, $\varepsilon_\lambda=10^{-3}$): Ag-110m, $\lambda=3.2116\times10^{-8}\ \mathrm{s^{-1}}$ (from the 249.8 d half-life printed in IAEA, 2012, Table 8.1, printed p. 435; the ENSDF adopted 249.83 d recommended in Section 3.2 gives $3.2112\times10^{-8}\ \mathrm{s^{-1}}$ and the same limit), over 620 efpd ($\lambda t=1.72$): $\Delta t\le3.6\times10^4\ \mathrm{s}\approx10\ \mathrm{h}$. I-131, $\lambda=1.0002\times10^{-6}\ \mathrm{s^{-1}}$ (from the 8.021 d half-life in the same Table 8.1; the ENSDF adopted 8.0252 d of audit 07 gives $9.9967\times10^{-7}\ \mathrm{s^{-1}}$ and the same limit to three digits), over five mean lives after a source change: $\Delta t\le400\ \mathrm{s}$. For Cs-137 ($\lambda=7.302\times10^{-10}\ \mathrm{s^{-1}}$, Browne & Tuli, 2007, via audit 07) the decay error is negligible over heating tests. At a steady state (P21.77) does not apply, because the BE fixed point is exact (Section 4.4).

**Diffusion error.** Each mode of $\mathbf V^{-1}\mathbf K$ with rate $\Lambda$ is advanced by $[1+(\Lambda+\lambda)\Delta t]^{-1}$, so the slowest mode carries a relative error of about

$$
\varepsilon_1(t)\approx\tfrac12(\Lambda_1+\lambda)^2t\,\Delta t, \tag{P21.78}
$$

with $\Lambda_1$ the first Phase 1 eigenvalue (or the smallest semi-discrete one). CRP-6 case 1b (Section 4.7.6): $\Lambda_1=\pi^2D'=1.97\times10^{-5}\ \mathrm{s^{-1}}$, $\Lambda_1t=14.2$; at the FRESCO-II step of $360\ \mathrm{s}$ (IAEA, 2012, Table 10.4, printed p. 542) (P21.78) predicts $5.1\,\%$ error in $1-F$, and at $36\ \mathrm{s}$ $0.5\,\%$. The FV values $1-F=4.280\times10^{-7}$ and $4.090\times10^{-7}$ differ by $4.6\,\%$, matching the predicted difference. Quantities near complete release, such as $1-F$ in case 1b, are therefore the most step-sensitive.

**Source switching times.** The rectangle rule misplaces (adds or loses) up to $\lvert\Delta S_K\rvert V_K\Delta t$ of generated inventory at a switch inside a step, with $V_K=\tfrac{4\pi}{3}R_1^3$ the kernel volume; the step average (A) is exact in amount but smears the switch over one step, an $O(\Delta t)$ timing error in $M$ and $J_R$:

$$
\int_{t^n}^{t^{n+1}}\bar S\,dt-\Delta t\,\bar S(t^{n+1})=-\tfrac12\Delta t^2\dot{\bar S}+O(\Delta t^3)\ (\text{smooth}),\qquad \lvert\text{error}\rvert\le\lvert\Delta S_K\rvert V_K\Delta t\ (\text{jump in the step}). \tag{P21.79}
$$

Rules: (1) place a step boundary on every switch time of $S_K(t)$, on every output time and (Phase 2.2) on every temperature-history knot, as MOOSE does with `force_step_every_function_point` (idaholab, 2026; audit 08, Sec. 3); (2) after each switch restart from a small step and grow it geometrically (factor 1.5 in N1) because time derivatives jump there; BE is L-stable, so no oscillation follows; (3) choose $\Delta t_n=\min\{$(P21.77) limit, (P21.78) limit, time to the next switch or output, growth limit$\}$. There is no stability limit. The explicit bound TRISO-FV-300, used only for checks, becomes $\Delta t\le\min_jV_j/(G_W+G_E+\lambda V_j)$ with decay (non-negativity of the forward-Euler update). A step-doubling estimate of $\varepsilon$ is a cheap a-posteriori alternative.

### 4.7 Exact references used by the tests

#### 4.7.1 Decay identity (exact)

Let $c_0$ solve (P21.50)-(P21.51) with $\lambda=0$, $S=0$ and the same initial value. Then

$$
c_\lambda(r,t)=e^{-\lambda t}c_0(r,t). \tag{P21.80}
$$

Proof: $\partial_tc_\lambda=-\lambda c_\lambda+e^{-\lambda t}\partial_tc_0=-\lambda c_\lambda+e^{-\lambda t}\mathcal Lc_0=-\lambda c_\lambda+\mathcal Lc_\lambda$, with $\mathcal L$ the linear spatial operator (also when $D_i=D_i(t)$). The centre, interface and surface conditions are linear and homogeneous, so they are multiplied by $e^{-\lambda t}$ and still hold; the initial values agree. Uniqueness of the solution of the linear problem gives (P21.80) (D1, D1b). For one homogeneous sphere this is the factor $e^{-\lambda t}$ of Gelbard (2003, Eqs. II.6 and II.16 to II.18, pp. 11-14); (P21.80) extends it to five layers. With a source, $c=e^{-\lambda t}\tilde c$ where $\tilde c$ solves the decay-free problem with source $e^{\lambda t}S$ (D2). Conditions: $\lambda$ uniform in space and constant in time; it fails if a precursor feeds the nuclide or if $\lambda$ differed by layer.

**Inventory partition (series).** With the Phase 1 weights $w_n$ (TRISO-DEL-021), $M_{v1}(t)=M_0\sum_nw_ne^{-\Lambda_nt}$ and $J_{R,v1}=-dM_{v1}/dt$. By (P21.80), $J_R=e^{-\lambda t}J_{R,v1}$ and

$$
\frac{M(t)}{M_0}=\sum_nw_ne^{-(\Lambda_n+\lambda)t},\quad \frac{M_{rel}(t)}{M_0}=\sum_nw_n\frac{\Lambda_n}{\Lambda_n+\lambda}\big(1-e^{-(\Lambda_n+\lambda)t}\big),\quad \frac{M_{dec}(t)}{M_0}=\sum_nw_n\frac{\lambda}{\Lambda_n+\lambda}\big(1-e^{-(\Lambda_n+\lambda)t}\big). \tag{P21.81}
$$

(series; $M+M_{rel}+M_{dec}=M_0$ term by term, D3; $\lambda\to0$ returns Phase 1.) Truncation needs care. The Phase 1 rule ($t\ge40/\Lambda_{max}$, `min_valid_time`) controls only the series for $M$, whose omitted terms carry $e^{-40}$. The $M_{rel}$ series as written converges like the weight tail $\sum_{n>n_{max}}w_n$, which is $O(1/n_{max})$ for a uniformly loaded sphere ($w_n=6/(n^2\pi^2)$): with the 13 modes that satisfy the rule in CRP-6 case 1a, a term-by-term $M_{rel}/M_0$ is low by $0.045$ (checker N11). Evaluate instead $M_{rel}=M_0-M-M_{dec}$, using $\sum_nw_n=1$ exactly; the omitted part of $M_{dec}/M_0$ is then at most $\lambda t\sum_{n>n_{max}}w_n/(\Lambda_nt)$ ($1.7\times10^{-7}$ in case 1a with Cs-137 decay), so add modes until this bound is below the target accuracy. Two consequences for comparisons: $M_{rel}/M_0$ depends on $\lambda$, while the decay-corrected fraction $F_{corr}=1-e^{\lambda t}M(t)/M_0$ equals the decay-free $F$ exactly. For a uniformly loaded sphere at long times $M_{rel}(\infty)/M_0=\sum w_n\Lambda_n/(\Lambda_n+\lambda)\approx1-\lambda\tau_{res}$ with $\sum_nw_n/\Lambda_n=R^2/(15D)$.

#### 4.7.2 Constant kernel source, no decay: multilayer steady state (exact)

Steady flux from the integrated balance: $r^2J=S_Kr^3/3$ for $r<R_1$ and $r^2J=S_KR_1^3/3$ for $r>R_1$. Integrating $-D_ic'=J$ inward from $c(R)=0$ with continuity of $c$:

$$
c(r)=\frac{S_K R_1^3}{3}\left[\frac{1}{D_i}\Big(\frac1r-\frac1{R_i}\Big)+\Phi_i\right]\ (R_{i-1}\le r\le R_i,\ i\ge2),\qquad c(r)=\frac{S_K(R_1^2-r^2)}{6D_1}+\frac{S_KR_1^3}{3}\Phi_1\ (r\le R_1),
$$

$$
\Phi_i=\sum_{m=i+1}^{5}\frac{1}{D_m}\Big(\frac{1}{R_{m-1}}-\frac{1}{R_m}\Big). \tag{P21.82}
$$

$$
J_R=\mathrm{Gen}=\tfrac{4\pi}{3}R_1^3S_K. \tag{P21.83}
$$

Integrating by parts, $M_{ss}=-\int_0^R\tfrac{4\pi r^3}{3}c'\,dr$, gives

$$
M_{ss}=\frac{4\pi S_K}{9}\left[\frac{R_1^5}{5D_1}+R_1^3\sum_{i=2}^{5}\frac{R_i^2-R_{i-1}^2}{2D_i}\right],\qquad \tau_{res}=\frac{M_{ss}}{\mathrm{Gen}}=\frac{R_1^2}{15D_1}+\sum_{i=2}^{5}\frac{R_i^2-R_{i-1}^2}{6D_i}. \tag{P21.84}
$$

(exact; A1 to A9.) Each layer adds a series "residence resistance" $(R_i^2-R_{i-1}^2)/(6D_i)$. For zero initial inventory and constant source, $M_{rel}=\mathrm{Gen}\,t-M(t)$, so $F=M_{rel}/(\mathrm{Gen}\,t)=1-\tau_{res}/t$ plus exponentially decaying terms (approximate at finite $t$, exact as an asymptote). For one material with $R_1=R$, $\tau_{res}=R^2/(15D)$ reproduces the Booth asymptote $F\approx1-1/(15D't)$ of IAEA (2012, Eq. 7.3, printed p. 317) (A9). In the discrete scheme with $\lambda=0$, (P21.70) at a fixed point gives $G_RC_N=\mathrm{Gen}$ exactly, so $J_R/\mathrm{Gen}=1$ is a round-off test while the profile converges at order 2.

#### 4.7.3 Source and decay: multilayer steady state (semi-analytical)

With $u=rc$, (P21.50) at steady state reads $D_iu''-\lambda u+S_Kr\,H(R_1-r)=0$. With $s=r-R_{i-1}$:

$$
u_1=\frac{S_K}{\lambda}r+A_1\sinh(k_1r),\qquad u_i=A_i\sinh(k_is)+B_i\cosh(k_is)\ (i\ge2),\qquad k_i=\sqrt{\lambda/D_i}, \tag{P21.85}
$$

with $u_i(R_i)=u_{i+1}(R_i)$, $D_i(u_i'-u_i/r)=D_{i+1}(u_{i+1}'-u_{i+1}/r)$ at $R_i$ (TRISO-ML-439, TRISO-ML-446) and $u_5(R)=0$: nine linear equations for nine coefficients. This is the Phase 1 transfer walk with $\Lambda\to-\lambda$ (cos, sin replaced by cosh, sinh) plus the particular solution in the kernel. Checked at 60 digits for the Table 10.2 geometry with synthetic $D_i=(4\times10^{-14},4\times10^{-13},2\times10^{-14},5\times10^{-15},2\times10^{-14})\ \mathrm{m^2\ s^{-1}}$ and $\lambda=2\times10^{-8}\ \mathrm{s^{-1}}$: conditions to $10^{-54}$, $\mathrm{Gen}=\lambda M_{ss}+J_R$ to $10^{-55}$, $R/B=0.9684291531$, and $\lambda\to0$ approaches (P21.82) with an $O(\lambda)$ error (C1 to C4). The FV fixed point on a 40-cell mesh with (20, 8, 4, 4, 4) cells per layer gives $R/B=0.968429$ for the same data (N8a; numerical, spatial error included). With 8 cells per layer the FV value is $0.968413$ (checker N2), so the agreement to six digits depends on that mesh.

#### 4.7.4 Booth single sphere with source and decay (exact and series)

For one material of radius $R$ with uniform source $S$ (IAEA, 2012, Eq. 7.5, printed p. 319):

$$
c(r)=\frac{S}{\lambda}\left[1-\frac{R\sinh(kr)}{r\sinh(kR)}\right],\qquad \frac{R}{B}=\frac{3}{\sqrt x}\Big(\coth\sqrt x-\frac1{\sqrt x}\Big)=\sum_{n\ge1}\frac{6}{n^2\pi^2+x},\qquad x=(kR)^2=\frac{\lambda}{D'}, \tag{P21.86}
$$

which is Eq. (7.6) of the report (B1 to B5). The symbol $x=\lambda/D'$ is the one of Section 3, Eqs. (P21.48) and (P21.49). From the modal equations $\dot m_n=-(\Lambda_n+\lambda)m_n+Bw_n$ with $\Lambda_n=n^2\pi^2D'$, $w_n=6/(n^2\pi^2)$, zero initial inventory and birth rate $B=\mathrm{Gen}$:

$$
\frac{R}{B}(t)=\sum_n\frac{6}{n^2\pi^2+x}\Big(1-e^{-(n^2\pi^2D'+\lambda)t}\Big),\qquad F_{(7.8)}(t)=\frac{M_{out}(t)}{M(t)+M_{out}(t)},\qquad M+M_{out}=\frac{B(1-e^{-\lambda t})}{\lambda}. \tag{P21.87}
$$

The first expression is Eq. (7.7) term by term (D4a). As a sum it converges slowly (tail about $6/(\pi^2n_{max})$); compute it in the form of Eq. (7.7), the closed form (P21.86) minus $6e^{-\lambda t}\sum_n e^{-n^2\pi^2D't}/(n^2\pi^2+x)$, whose sum converges like the Phase 1 series. The second result is derived here: the "cumulative fractional release" of Eq. (7.8) equals the released atoms still present outside divided by all surviving atoms, inside and outside (symbolic per mode D4b, D4c; summed to $10^{-31}$ in D4d; $\lambda\to0$ gives Eq. 7.1, the in-reactor constant-production Booth formula of IAEA (2012, printed p. 317), D5). It does not equal $M_{rel}/M_{gen}$ when $\lambda>0$. The report does not state this definition in words: on printed p. 319 it introduces Eq. (7.8) only as the cumulative fractional release (checked in the PDF during the B3 reconciliation), so the identification is derived here and the algebra is exact. FV comparison quantities: $(R/B)^n=G_RC_N^n/\mathrm{Gen}^n$ and $F^n=M_{out}^n/(M^n+M_{out}^n)$ with (P21.73).

#### 4.7.5 Manufactured solution in a two-layer sphere (exact)

Layers $0<r<R_1$ and $R_1<r<R$; $P(r)=\cos(\pi r/2R)$; $D_{ref}$, $c_{ref}$, $\tau$ are scaling constants ($\mathrm{m^2\ s^{-1}}$, $\mathrm{mol\ m^{-3}}$, $\mathrm{s}$):

$$
c_i=c_{ref}\,g(t)\,\psi_i(r),\quad g=1+\frac t\tau,\quad \psi_1=D_{ref}\Big[\frac{P-P(R_1)}{D_1}+\frac{P(R_1)}{D_2}\Big],\quad \psi_2=\frac{D_{ref}P}{D_2}. \tag{P21.88}
$$

By construction $D_i\psi_i'=D_{ref}P'$ is continuous, $\psi$ is continuous at $R_1$, $P'(0)=0$ and $P(R)=0$, so all of (P21.51) hold (M1). The source follows from (P21.50):

$$
S_i=c_{ref}\Big[\frac{\psi_i}{\tau}+g\,D_{ref}\,Q(r)+\lambda g\psi_i\Big],\qquad Q(r)=\frac{\pi}{Rr}\sin\frac{\pi r}{2R}+\frac{\pi^2}{4R^2}\cos\frac{\pi r}{2R}. \tag{P21.89}
$$

The diffusion part $D_{ref}Q$ is the same in both layers, $S$ is continuous at $R_1$, finite at $r=0$ ($Q(0)=3\pi^2/4R^2$) and nonnegative (M2, M3; since $P\ge P(R_1)$ on $[0,R_1]$, $\psi_1\ge0$). The source must enter as exact cell and step averages, or quadrature error contaminates the observed order:

$$
\bar S_j^{n+1}=\frac{1}{V_j\Delta t}\int_{t^n}^{t^{n+1}}\Big[\mathcal S_i(r_{j+1/2},t)-\mathcal S_i(r_{j-1/2},t)\Big]dt=\frac{\mathcal S_i(r_{j+1/2},t^{n+1/2})-\mathcal S_i(r_{j-1/2},t^{n+1/2})}{V_j},\quad \mathcal S_i'=4\pi r^2S_i, \tag{P21.90}
$$

where the step average equals the midpoint value because $S$ is linear in $t$ (exact). The closed-form antiderivative $\mathcal S_i$ is generated by SymPy (M4). A steady variant ($g=1$, no $\psi/\tau$ term) isolates the spatial order; the transient variant on a fine mesh isolates the temporal order.

#### 4.7.6 CRP-6 sensitivity cases 1a and 1b (series; benchmark data)

Bare UO2 kernel, $r_k=250\ \mu\mathrm{m}$, uniform initial Cs-137, no source, absorbing surface, $200\ \mathrm{h}=720\,000\ \mathrm{s}$ at $1200\ ^\circ\mathrm{C}$ (1a) and $1600\ ^\circ\mathrm{C}$ (1b) (IAEA, 2012, Table 10.5, printed p. 543; Table 10.2, printed p. 541). The benchmark diffusivity is the two-branch FRG Cs-in-UO2 law of IAEA (1997, Table A-1, printed p. 439), and Booth gives

$$
D(T)=5.6\times10^{-8}e^{-209\,000/(R_gT)}+5.2\times10^{-4}e^{-362\,000/(R_gT)}\ \mathrm{m^2\ s^{-1}},\qquad F=1-\frac{6}{\pi^2}\sum_{n\ge1}\frac{e^{-n^2\pi^2D't}}{n^2},\quad D'=D/r_k^2 \tag{P21.91}
$$

(IAEA, 2012, Eq. 10.7, printed p. 544). Results (numerical, B3): with $R_g=8.3145$ (Eq. 10.5a), 1a: $D't=0.02594$, $F=0.46742$; with $R_g=8.314$, $F=0.46721$; the printed analytical value 0.4673 (Table 10.6, printed p. 544) lies inside this bracket. With the TECDOC-978 constant $R_g=8.3143$ (IAEA, 1997, p. 435) $F=0.467337$, which is the only one of these values that rounds to 0.4673 (B3 check `b3_sec4.py`, X11a; `b3_sec5.py`, T1f). 1b: $D't=1.4405$, $F=0.99999959$, equal to the printed value. The single-branch TECDOC-1674 Table 7.1 law ($D'_0=0.90\ \mathrm{s^{-1}}$, $Q=209\ \mathrm{kJ\ mol^{-1}}$) gives 0.46163 and 0.99995461 and does not reproduce the table, as found in audit 04 (Sec. 4.3). FV with $N=150$: 1a $F=0.46737$ ($\Delta t=360\ \mathrm{s}$) and $0.46740$ ($36\ \mathrm{s}$); 1b $1-F=4.28\times10^{-7}$ and $4.09\times10^{-7}$; closure $\le1.7\times10^{-14}$. The benchmark analytical values are decay-free. If Cs-137 decay is switched on, $M_{rel}/M_0$ drops by $7.5\times10^{-5}$ (1a) and $2.4\times10^{-5}$ (1b); the 1b shift equals $\lambda\tau_{res}=2.43\times10^{-5}$ of Section 4.7.1, which is 60 times the 1b target $1-F$. Compare decay runs through $F_{corr}$ of (P21.81), or run 1a/1b with $\lambda=0$.

**Observed order** (used in the matrix):

$$
p=\log_2\frac{e_h}{e_{h/2}},\qquad p_{diff}=\log_2\frac{\lvert X_h-X_{h/2}\rvert}{\lvert X_{h/2}-X_{h/4}\rvert}, \tag{P21.92}
$$

with $e$ a volume-weighted $L_2$ or scalar error against the exact reference, and $p_{diff}$ used when a fixed error of the other discretisation (for example a spatial floor in a time study) would mask $e$.

### 4.8 (f) Verification matrix

| No. | Test | Exact reference | Quantity | Expected behaviour / order | Tolerance |
|---|---|---|---|---|---|
| 1 | Phase 1 regression, $\lambda=0$, $S=0$ | v1.0 `backward_euler_step` and `run` | all $C_j^n$, $M$, $M_{rel}$ | bit-identical (Sec. 4.3 detail ii) | array equality, 0 ulp |
| 2 | Decay only, no diffusion (all $G=0$) | $c=c_0e^{-\lambda t}$; discrete $(1+z)^{-n}c_0$ | $C_j^n$, $M^n$ | discrete identity to round-off; order 1, error $\tfrac12\lambda^2t\Delta t$ (P21.77) | $10^{-14}$ (discrete); error/prediction within 2 % |
| 3 | Decay plus diffusion, no source | (P21.80), (P21.81): Phase 1 series times $e^{-\lambda t}$; discrete (P21.63), semi-discrete (P21.67) | $M$, $M_{rel}$, $M_{dec}$, $F_{corr}$ | (P21.63), (P21.67) to round-off; FV vs series $O(h^2)+O(\Delta t)$, order 1 in time | $10^{-13}$ for identities; series: Phase 1 tolerance above the reported accuracy floor |
| 4 | Constant kernel source, no decay, approach to steady state | (P21.82) to (P21.84) | profile, $J_R/\mathrm{Gen}$, $M_{ss}$, $F\to1-\tau_{res}/t$ | $J_R=\mathrm{Gen}$ at the fixed point exactly; profile and $M_{ss}$ order 2 in $h$ | $\lvert J_R/\mathrm{Gen}-1\rvert<10^{-12}$; observed order 1.9 to 2.1 |
| 5 | Source plus decay steady state, multilayer | (P21.85) (9x9 solve) | $R/B$, $M_{ss}$, profile | BE fixed point independent of $\Delta t$; order 2 in $h$ | $\Delta t$-independence $10^{-12}$; order 1.9 to 2.1 |
| 6 | Booth sphere with source and decay | IAEA (2012) Eqs. 7.6 to 7.8, (P21.86), (P21.87) | $R/B_{ss}$, $R/B(t)$, $F(t)=M_{out}/(M+M_{out})$ | order 2 in $h$ (steady); order 1 in $\Delta t$ (transient) | rel. $10^{-5}$ at $N=200$ (steady); abs. $10^{-4}$ at $N=400$, $\Delta t=50\ \mathrm{s}$ |
| 7 | Modal vs FV, multilayer with source and decay | Phase 2.1 modal solution of Section 3, (P21.31) to (P21.39) (truncated series) | $M$, $J_R$, $F$, $M_{dec}$ | difference $O(h^2)+O(\Delta t)$, both closures at round-off | $\lvert\Delta F\rvert<10^{-4}$ at production mesh, and above the modal accuracy floor |
| 8 | Manufactured solution, two layers | (P21.88) to (P21.90) | volume-weighted $L_2$ error | order 2 in $h$ (steady variant); order 1 in $\Delta t$ (transient variant) | observed order within $\pm0.1$ at the finest pair |
| 9 | Inventory closure | (P21.72), (P21.75) | relative residual | round-off for any $\Delta t$, switching source, variable steps | $<10^{-12}$ (v1.0 criterion) |
| 10 | Positivity | (P21.62) | $\min_jC_j^n$ | $\ge0.0$ exactly when $C^0\ge0$, $S\ge0$, any $\Delta t$ | $0$ (exact) |
| 11 | CRP-6 case 1a | (P21.91) with the two-branch FRG Cs law; printed 0.4673 | $F$ at $200\ \mathrm{h}$ | $O(h^2)+O(\Delta t)$ | $\lvert F-0.46742\rvert<2\times10^{-4}$ ($R_g$ bracket 0.46721 to 0.46742) |
| 12 | CRP-6 case 1b | as 11; printed 0.99999959 | $1-F$ | relative error of $1-F$ follows (P21.78) | within 5 % at $\Delta t\le36\ \mathrm{s}$, $N=150$ |
| 13 | Source switch inside a step | $\int S\,dt$ | $M_{gen}$ | (A) exact; (R) off by up to $\lvert\Delta S_K\rvert V_K\Delta t$ | $10^{-14}$ for (A) |
| 14 | Steady-state preservation | semi-discrete steady state | $R/B$ for $\lambda\Delta t=0.1,1,5$ | identical for every $\Delta t$ (unsplit BE only) | $10^{-12}$ |
| 15 | Decay-exact split as cross-check, $S=0$ | (P21.67) | $\mathbf C^n$ | $=e^{-\lambda t^n}\times$ v1.0 FV, round-off | $10^{-13}$ |

Table: 4.3 Verification matrix for Phase 2.1.

Tests 1, 2, 3 (identities), 9, 10, 13, 14 and 15 test the algebra and should run in `cargo test`; tests 4 to 8, 11 and 12 test accuracy and need a mesh and step study. Test 12 is the most step-sensitive item in the matrix.

### 4.9 Script evidence

All scripts run with `.venv/Scripts/python.exe -I -X utf8 <script>`, write nothing to disk and end with `ALL PASS`. Python with SymPy 1.14.0, NumPy 2.2.6, SciPy 1.15.3, mpmath.

`scripts/derivation_checks/sec4_p21_steady_state.py` (P21.82 to P21.86):
```text
PASS  A1 ODE residual in all five layers (P21.82)  [0, 0, 0, 0, 0]
PASS  A2 centre regularity dc/dr(0) = 0
PASS  A3 continuity of c at R_1..R_4  [0, 0, 0, 0]
PASS  A4 continuity of D dc/dr at R_1..R_4  [0, 0, 0, 0]
PASS  A5 absorbing surface c(R) = 0
PASS  A6 steady release J_R = Gen = (4/3) pi R_1^3 S (P21.83)
PASS  A7 steady inventory closed form (P21.84)
PASS  A8 tau_res = M_ss/Gen = R_1^2/(15 D_1) + sum (R_i^2-R_{i-1}^2)/(6 D_i)
PASS  A9 one material, R_1 = R: tau_res = R^2/(15 D)
PASS  B1 Booth steady ODE D r^-2 (r^2 c')' - lam c + S = 0 (TECDOC-1674 Eq. 7.5)
PASS  B2 c(R) = 0
PASS  B3 regular at r = 0 (finite limit)
PASS  B4 R/B equals TECDOC-1674 Eq. (7.6) (numeric, 40 digits)  |diff| = 2.53e-175
PASS  B5 sum 6/(n^2 pi^2 + a) = (7.6) closed form  a = 1433.33, R/B = 0.0771475583135
PASS  C1 interface and surface conditions, multilayer with decay (P21.85)  max residual 2.59e-54
PASS  C2 ODE residual inside IPyC (mp.diff)  -2.97e-67
PASS  C3 steady balance Gen = lam M_ss + J_R  rel 3.21e-55, R/B = 0.9684291531
PASS  C4 lambda -> 0 limit equals part A, error O(lambda)  rel errors ['1.62e-6', '1.62e-8', '1.62e-10']
ALL PASS
```

`scripts/derivation_checks/sec4_p21_decay_identity.py` (P21.80, P21.81, P21.87):
```text
PASS  D1 PDE: res_decay[e^{-lam t} c0] = e^{-lam t} res_free[c0] (P21.80)
PASS  D1b conditions scale by e^{-lam t} (homogeneous, linear)
PASS  D2 with source: c = e^{-lam t} ct, ct driven by e^{lam t} S
PASS  D3a M_rel per mode = M0 w Lam/(Lam+lam) (1 - e^{-(Lam+lam) t}) (P21.81)
PASS  D3b M_dec per mode = M0 w lam/(Lam+lam) (1 - e^{-(Lam+lam) t})
PASS  D3c M + M_rel + M_dec = M0 w per mode (closure)
PASS  D3d lam -> 0: M_rel -> M0 w (1 - e^{-Lam t}) (Phase 1)
PASS  D4a mode-wise R/B(t) = TECDOC-1674 Eq. (7.7) term (P21.87)
PASS  D4b mode-wise M_out/(M + M_out) = TECDOC-1674 Eq. (7.8) term
PASS  D4c M_n + M_out,n = w B (1 - e^{-lam t})/lam (inventory identity behind the denominator)
PASS  D4d summed: Eq. (7.8) = M_out/(M + M_out) from modal sums  max |diff| 9.86e-32; F(1e4 s) = 0.5311856865
PASS  D5 lam -> 0 of Eq. (7.8) equals Eq. (7.1) at D't = 0.01, 0.1, 1  max |diff| 2.7e-16
ALL PASS
```

`scripts/derivation_checks/sec4_p21_matrix_schemes.py` (P21.54, P21.60, P21.61, P21.63, P21.68, P21.73 to P21.79):
```text
PASS  X1 elimination pivots d_j = e_j + g^E_j with the subtraction-free e_j (P21.61)  induction step and explicit N = 5
PASS  X1b e_j - v_j = g^W_j e_{j-1}/(e_{j-1}+g^W_j) > 0 (no subtraction anywhere)
PASS  X2 row excess = v_j (j < N), v_N + G_R (j = N) (P21.60)  v_j = V_j (1/dt + lam)
PASS  X3a V/dt + lam V + K = V/dt' + K, dt' = dt/(1+z)
PASS  X3b V/dt = (1+z)^-1 V/dt', hence A_lam^-1 V/dt = (1+z)^-1 (V/dt'+K)^-1 V/dt' (P21.63)
PASS  X4 IF-scheme fixed point (z e^z/(e^z-1) K + lam V) C = V S (P21.68)
PASS  X4b factor z e^z/(e^z-1) = 1 + z/2 + z^2/12 + O(z^4)
PASS  X5 ln[(1+z)^-n e^{nz}] = n z^2/2 - n z^3/3 + O(z^4); with n z = lam t: ~ lam^2 t dt/2 (P21.77)
PASS  X6a fixed points of both M_out updates are J/lam (P21.76)
PASS  X6b exponential update is the exact solution of dM_out/dt = J - lam M_out, J step-constant (P21.74)
PASS  X6c BE update local error = h^2 lam (lam M_o - J)/2 + O(h^3)  -h**2*lambda*(J - M_o*lambda)/2
PASS  X7 int_step S - dt S(t^{n+1}) = -dt^2 S'/2 - (2/3) dt^3 S''/2 (Taylor, P21.79)
PASS  X8 straddling cell: Sbar = S_K (R_1^3 - a^3)/(b^3 - a^3) (P21.54)
ALL PASS
```

`scripts/derivation_checks/sec4_p21_mms_source.py` (P21.88 to P21.90):
```text
PASS  M1 centre, continuity, flux continuity, surface (P21.88)  [0, 0, 0, 0]
PASS  M2a diffusion part identical in both layers  -L c / (c_ref g) = pi*D_ref*(4*R*sin(pi*r/(2*R)) + pi*r*cos(pi*r/(2*R)))/(4*R**2*r)
PASS  M2b S continuous at R1
PASS  M3 S >= 0 everywhere for the chosen data  min S = 2.565e-07 mol m^-3 s^-1
PASS  M4a antiderivative derivative check
PASS  M4b cell integral of S: antiderivative = quadrature (P21.90)  rel 1.9e-15
PASS  M5 steady MMS, spatial order (volume-weighted L2) -> 2  errors ['2.28e-04', '5.64e-05', '1.41e-05', '3.51e-06', '8.77e-07'], orders ['2.014', '2.004', '2.001', '2.000']
PASS  M6 transient MMS, temporal order -> 1 (BE)  errors ['6.22e-02', '3.12e-02', '1.56e-02', '7.81e-03', '3.91e-03'], orders ['0.995', '0.999', '1.000', '1.000']
ALL PASS
```

`scripts/derivation_checks/sec4_p21_be_closure_order.py` (Python transcription of the v1.0 mesh and solver with (P21.59); P21.62, P21.63, P21.67, P21.68, P21.70 to P21.77):
```text
PASS  N1a per-step identity M^{n+1}-M^n = dt(Gen - lam M^{n+1} - G_R C_N^{n+1}) (P21.70)  max rel 1.7e-15 over 71 steps
PASS  N1b cumulative closure M0 + M_gen = M + M_dec + M_rel (P21.72)  rel residual 1.2e-16; M_gen 6.8722e-09, M_dec 2.1191e-10, M_rel 6.4675e-09, M 2.5828e-10 mol
PASS  N2 min C >= 0.0 exactly in floating point for dt = 1e-3 .. 1e12 s  min 1.095e-128
      N3 data: Lambda_1(semi-discrete) = 7.1660e-07 s^-1, T = 1.3955e+06 s, lam = Lambda_1, M(T)/M0 = 0.1602
PASS  N3a semi-discrete identity exp(-(A + lam I)T) C0 = e^{-lam T} exp(-A T) C0 (P21.67)  rel 2.6e-13
PASS  N3b unsplit BE: error in M(T) vs semi-discrete reference, order -> 1  errors ['7.88e-02', '3.97e-02', '1.99e-02', '9.98e-03', '4.99e-03', '2.50e-03'], orders ['0.990', '0.995', '0.997', '0.999', '0.999']
PASS  N3c decay-exact split: order -> 1, smaller constant  errors ['1.96e-02', '9.89e-03', '4.97e-03', '2.49e-03', '1.25e-03', '6.23e-04']
PASS  N4 BE(lam, dt)^n = (1 + lam dt)^-n BE_v1(dt/(1 + lam dt))^n (P21.63)  rel 7.6e-16
PASS  N5 lam = 0, S = 0: identical to v1.0 bit for bit over 200 steps
PASS  N6 decay only: C^n = (1+z)^-n C0 exactly; order 1 to e^{-lam t}; error ~ lam^2 t dt/2  errors ['4.80e-02', '2.45e-02', '1.24e-02', '6.22e-03', '3.12e-03'], predicted last 3.13e-03
PASS  N7a BE M_out: M_rel = M_out + M_dec,out (P21.75)  rel 1.3e-14
PASS  N7b whole system: M_gen = M + M_out + M_dec,in + M_dec,out  rel 8.5e-15
PASS  N7c both M_out updates approach J_R/lam at steady state (P21.76)  BE 1.2e-07, exponential 1.2e-07, (M_out_BE - M_out_exp)/M_out = -3.8e-09
PASS  N8a BE fixed point = semi-discrete steady state, lam dt = 0.1, 1, 5, two lam (R/B exact)  max rel R/B diff 5.6e-14; steady R/B = 0.968429 (lam 2e-8), 0.025830 (lam 7e-6)
PASS  N8b integrating-factor fixed point is (z e^z/(e^z-1) K + lam V) C = V S (P21.68): biased M_ss and R/B  lam=2e-08,z=0.1: dM/M -4.71e-02, dRB/RB +1.54e-03; lam=2e-08,z=1.0: dM/M -3.61e-01, dRB/RB +1.18e-02; lam=2e-08,z=5.0: dM/M -7.97e-01, dRB/RB +2.60e-02; lam=7e-06,z=0.1: dM/M -2.70e-03, dRB/RB +1.02e-01; lam=7e-06,z=1.0: dM/M -3.38e-02, dRB/RB +1.27e+00; lam=7e-06,z=5.0: dM/M -2.32e-01, dRB/RB +8.76e+00
PASS  N9 exact step average reproduces the generated amount; rectangle at n+1 misses part of a step  rectangle error -0.167, step average error +1.6e-16
ALL PASS
```
(N7c: the residual $1.2\times10^{-7}$ equals the $e^{-16}$ transient that remains after 16 mean lives; both updates have the exact fixed point, X6a.)

`scripts/derivation_checks/sec4_p21_booth_crp6.py` (P21.86, P21.87, P21.91, matrix rows 6, 11, 12):
```text
PASS  B1 steady R/B: FV (= BE fixed point) -> Eq. (7.6), spatial order 2  (7.6) R/B = 0.71980889; rel errors ['6.36e-04', '1.59e-04', '3.99e-05', '9.97e-06']; orders ['1.997', '1.999', '2.000']
PASS  B2 transient R/B(t) (7.7) and F(t) (7.8) at t = 5e4 s: BE temporal order ~ 1, errors < 1e-4 at the finest step dt = 50 s  (7.7) 0.719726, (7.8) 0.710587; |err R/B| ['2.7e-05', '1.4e-05', '7.7e-06', '4.7e-06']; |err F| ['4.4e-04', '2.2e-04', '1.1e-04', '5.7e-05']; difference orders ['1.10', '1.05'] / ['1.00', '1.00']
      case 1a: T = 1473.15 K, D(two-branch) = 2.2521e-15 m^2/s, D't = 0.02594; F(two-branch, R_g = 8.3145 / 8.314462618 / 8.314) = 0.46741912 / 0.46740380 / 0.46721432; F(single branch) = 0.46162937; printed 0.4673
      case 1b: T = 1873.15 K, D(two-branch) = 1.2504e-13 m^2/s, D't = 1.44045; F(two-branch, R_g = 8.3145 / 8.314462618 / 8.314) = 0.99999959 / 0.99999959 / 0.99999959; F(single branch) = 0.99995461; printed 0.99999959
PASS  B3a two-branch FRG Cs D reproduces Table 10.6 analytical 1a/1b, single branch does not
      case 1a: with Cs-137 decay, M_rel/M_0 = 0.46734403 (change -7.51e-05); decay-corrected 1 - e^(lam t) M/M_0 = 0.46741912
      case 1b: with Cs-137 decay, M_rel/M_0 = 0.99997526 (change -2.43e-05); decay-corrected 1 - e^(lam t) M/M_0 = 0.99999959
      FV case 1a, N = 150, dt = 360 s: F = 0.46736891, 1 - F = 5.326e-01, closure 3.9e-16
      FV case 1a, N = 150, dt = 36 s: F = 0.46739958, 1 - F = 5.326e-01, closure 1.7e-14
      FV case 1b, N = 150, dt = 360 s: F = 0.99999957, 1 - F = 4.280e-07, closure 1.6e-15
      FV case 1b, N = 150, dt = 36 s: F = 0.99999959, 1 - F = 4.090e-07, closure 9.3e-15
PASS  B3b FV (N = 150, dt = 36 s) matches Booth (10.7): |dF| < 2e-4 (1a), 1-F within 5 % (1b); closure < 1e-12
ALL PASS
```
(The "decay-corrected" lines of B3 restate (P21.80) analytically; the FV check of that identity is N3a and N4.)

### 4.10 Assumptions, limiting cases and open issues

**Limiting cases verified.** $\lambda=0$, $S=0$: (P21.58) is TRISO-DEL-040 bit for bit (N5). $\lambda\to0$: (P21.85) tends to (P21.82) (C4), (P21.81) to Phase 1 (D3d), Eq. (7.8) to Eq. (7.1) (D5), $M_{out}\to M_{rel}$ (P21.76). $\Delta t\to\infty$ with constant source: the BE step returns the semi-discrete steady state (N8a). All $D_i$ equal and $R_1=R$: $\tau_{res}=R^2/(15D)$ (A9). $G=0$: pure decay (N6).

**Assumptions restated.** Uniform $\lambda$ per nuclide; no precursor feeding (otherwise (P21.80) fails and the coupled chain model of Section 3.3.4, FUTURE work, is needed); constant $D_i$ within a step (time dependence belongs to Phase 2.2); perfect interfaces without partition coefficients; absorbing surface; uniform kernel source with cumulative yield (approximate). The synthetic $D_i$ in the checks are test values, not material data.

**Open issues.**
1. The interpretation $F_{(7.8)}=M_{out}/(M+M_{out})$ is derived exactly. The B3 reconciliation read printed p. 319 of TECDOC-1674: Eq. (7.8) is introduced only as the cumulative fractional release, with no definition in terms of atom populations, so the identification rests on the derivation (Section 4.7.4), not on a printed statement.
2. CRP-6 case 1a matches the printed 0.4673 only inside the $R_g$ bracket (0.46721 to 0.46742); which $R_g$ the benchmark authors used is NOT VERIFIED. Whether participants included Cs-137 decay is NOT VERIFIED (audit 04, Sec. 2.4); decay shifts $M_{rel}/M_0$ in 1b by 60 times the target $1-F$.
3. Running cases 1a/1b in STRATO needs either a uniform initial concentration over all regions or a configurable region count; the v1.0 `initial_concentration` loads the kernel only and `Config::validate` requires exactly five regions ([src/config.rs, line 80](https://github.com/weewzr/STRATO/blob/v1.0.0/src/config.rs#L80)). Implementation choice left open.
4. Matrix row 7 depends on the Phase 2.1 modal solver of Section 3 (Code C extended with decay and source); its tolerance should be fixed once that solver reports an accuracy floor.
5. A second-order positive time integrator was not analysed; (P21.78) shows that $1-F$ near complete release needs $\Lambda_1\Delta t\lesssim10^{-3}$ with BE.
6. The Rust changes of Section 4.3 are specified but not implemented; the Python transcription in `sec4_p21_be_closure_order.py` is the only executed version.

## 5 Part 2.2: temperature-dependent diffusivity

This section extends the STRATO v1.0 model (constant $D_i$ per layer, initial kernel inventory, absorbing surface) to diffusivities that follow the particle temperature $T(t)$. It covers the Arrhenius law and its input rules (5.1), the finite-volume (FV) treatment with time-dependent conductances (5.2), the effective-time transform and the exact conditions under which it reduces Phase 2.2 to Phase 1 (5.3), the general multilayer case where every layer has its own activation energy (5.4), and the definition of realistic temperature histories (5.5). Section 5.6 lists the verification scripts and quotes their output. Equations are numbered (P22.1) to (P22.59).

Status labels used throughout: **exact** (no approximation beyond the stated model), **semi-analytical** (exact formula that still needs a one-dimensional quadrature or root search), **series** (exact infinite series, truncated with a stated rule), **approximate** (a modelling or asymptotic approximation), **numerical** (a discretisation with an observed order of accuracy).

### 5.0 Symbols, units and assumptions

| Symbol | Meaning | SI unit |
|---|---|---|
| $i = 1..5$ | layer index (kernel, buffer, IPyC, SiC, OPyC) | - |
| $R_0 = 0 < R_1 < \dots < R_5 = R$ | layer outer radii | $\mathrm{m}$ |
| $c(r,t)$ | concentration of one nuclide | $\mathrm{mol\ m^{-3}}$ |
| $T(t)$, $T_C$ | particle temperature in kelvin, temperature in degrees Celsius | $\mathrm{K}$, $^\circ\mathrm{C}$ |
| $T_{\mathrm{ref}}$ | reference temperature of the effective-time transform | $\mathrm{K}$ |
| $D_i(T)$ | diffusivity of layer $i$ | $\mathrm{m^2\ s^{-1}}$ |
| $D_{0,i,m}$, $Q_{i,m}$ | pre-exponential factor and activation energy of Arrhenius branch $m$ in layer $i$ | $\mathrm{m^2\ s^{-1}}$, $\mathrm{J\ mol^{-1}}$ |
| $Q_{\mathrm{eff},i}(T)$ | apparent activation energy of a multi-branch law | $\mathrm{J\ mol^{-1}}$ |
| $T_x$ | crossover temperature where two branches are equal | $\mathrm{K}$ |
| $R_g$ | molar gas constant | $\mathrm{J\ mol^{-1}\ K^{-1}}$ |
| $k_B$, $N_A$ | Boltzmann constant, Avogadro constant | $\mathrm{J\ K^{-1}}$, $\mathrm{mol^{-1}}$ |
| $\nu$ | attempt frequency of an atomic jump | $\mathrm{s^{-1}}$ |
| $\Gamma_j$ | total jump frequency of one diffusing atom | $\mathrm{s^{-1}}$ |
| $a_j$ | jump length | $\mathrm{m}$ |
| $z$, $f$ | number of equivalent target sites, correlation factor | - |
| $\Delta H_m$, $\Delta S_m$ | migration enthalpy and entropy per atom | $\mathrm{J}$, $\mathrm{J\ K^{-1}}$ |
| $\Phi_f$ | fast-neutron fluence (TECDOC-978 writes $\Gamma$) | $10^{25}\ \mathrm{m^{-2}}$ |
| $\lambda$ | decay constant | $\mathrm{s^{-1}}$ |
| $S(r,t)$ | volumetric source | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $\alpha(t)$ | shared scaling factor, $D_i = \alpha D_i^{\ast}$ | - |
| $D_i^{\ast}$ | layer diffusivity at $T_{\mathrm{ref}}$ | $\mathrm{m^2\ s^{-1}}$ |
| $\tau(t)$ | effective time | $\mathrm{s}$ |
| $\hat c(r,\tau)$, $\hat S(r,\tau)$ | transformed concentration and source | $\mathrm{mol\ m^{-3}}$, $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $u = r c$ | reduced concentration | $\mathrm{mol\ m^{-2}}$ |
| $\Lambda_n$, $\phi_n(r)$ | eigenvalues and eigenfunctions ($u_n = r\phi_n$) | $\mathrm{s^{-1}}$; unit of $\phi_n$ set by its normalisation |
| $N_n = \int_0^R r^2\phi_n^2\,dr$ | norm (weight $r^2$) | $\mathrm{m^3}$ times $[\phi_n]^2$ |
| $a_n(t)$, $w_n$ | modal amplitudes (so that $a_n\phi_n$ is in $\mathrm{mol\ m^{-3}}$), v1.0 release weights | $\mathrm{mol\ m^{-3}}/[\phi_n]$, - |
| $q_i$, $\rho_i$ | interface flux density of the old field, diffusivity ratio new/old (Section 5.4.1) | $\mathrm{mol\ m^{-2}\ s^{-1}}$, - |
| $P_{mn}$ | projection matrix between two eigenbases | - |
| $K_{mn}(t)$ | Galerkin stiffness on a fixed basis | $\mathrm{s^{-1}}$ |
| $M, M_0, M_{\mathrm{gen}}, M_{\mathrm{rel}}, M_{\mathrm{dec}}, M_{\mathrm{out}}$ | inventories (Section 5.3.3) | $\mathrm{mol}$ |
| $F(t)$ | released fraction $M_{\mathrm{rel}}/M_0$ ($\lambda = 0$, $S = 0$) | - |
| $\dot T$, $\theta = Q/R_g$ | ramp rate of one linear segment, activation temperature (Section 5.2.4) | $\mathrm{K\ s^{-1}}$, $\mathrm{K}$ |
| $H(t)$ | surface mass-transfer coefficient of a Robin condition (Section 5.3.5 only) | $\mathrm{m\ s^{-1}}$ |
| $s_n(t)$, $I_n$ | modal source $N_n^{-1}\int_0^R r^2\phi_nS\,dr$, modal inventory integral $\int_0^R r^2\phi_n\,dr$ | $\mathrm{mol\ m^{-3}\ s^{-1}}/[\phi_n]$, $\mathrm{m^3}[\phi_n]$ |
| $n_{\max}$ | number of retained modes in a truncated series | - |
| $N_T$, $h$, $B(t)$ | number of constant-temperature intervals of a staircase, interval length, Magnus generator $-K(t) - \lambda I$ (Section 5.4.4) | -, $\mathrm{s}$, $\mathrm{s^{-1}}$ |
| $J_R(t) = -4\pi R^2 D_5\,\partial_r c\vert_R$ | release rate | $\mathrm{mol\ s^{-1}}$ |
| $j = 1..N$, $V_j$, $\bar r_j$ | FV cells, shell volume, volume centroid | -, $\mathrm{m^3}$, $\mathrm{m}$ |
| $A_f$, $\delta_f^{\pm}$, $G_f$ | face area, centroid-to-face distances, face conductance | $\mathrm{m^2}$, $\mathrm{m}$, $\mathrm{m^3\ s^{-1}}$ |
| $C_j^n$, $\Delta t_n$ | cell value at step $n$, time step | $\mathrm{mol\ m^{-3}}$, $\mathrm{s}$ |
| $E_1$, $E_2$ | exponential integrals | - |

Table: 5.1 Symbols and units of Part 2.2.

Assumptions used in every subsection unless stated otherwise:

- **A1.** The temperature is uniform across the particle at each instant, $T = T(t)$. This holds approximately in post-irradiation heating tests, where Hales et al. (2021, p. 9) use "the surface temperature throughout the particle" for CRP-6 case 8a; it fails in pile, where fission heating sets a radial gradient (Hales et al., 2021, pp. 10 to 11).
- **A2.** Concentration and flux are continuous at every interface (no partition coefficient), as in v1.0. TECDOC-978 reports measured partition ratios "from about 0.3 to 3" that "may change with temperature and fast fluence" (IAEA, 1997, p. 436); STRATO v1.0 does not model them.
- **A3.** Absorbing surface, $c(R,t) = 0$, regular centre.
- **A4.** One nuclide with one decay constant $\lambda$ in all layers.
- **A5.** $D_i$ depends on time only through $T(t)$ (and, for one Cs-in-SiC correlation, through a fluence $\Phi_f$ that is frozen during a heating test).

### 5.1 (a) The Arrhenius law

#### 5.1.1 Statement

TECDOC-978 Eq. (A-1) (IAEA, 1997, p. 435) writes the effective diffusivity of a fission product in a coating material as a sum of Arrhenius branches. With the notation of this report:

$$
D_i(T) = \sum_{m=1}^{m_i} D_{0,i,m}\,\exp\!\left(-\frac{Q_{i,m}}{R_g T}\right), \qquad m_i \in \{1, 2\}. \tag{P22.1}
$$

Units: $D_{0,i,m}$ in $\mathrm{m^2\ s^{-1}}$, $Q_{i,m}$ in $\mathrm{J\ mol^{-1}}$, $R_g$ in $\mathrm{J\ mol^{-1}\ K^{-1}}$, $T$ in $\mathrm{K}$; the exponent is dimensionless. Status: empirical correlation (approximate model), fitted to irradiation and heating data.

#### 5.1.2 Physical origin: the jump-frequency picture

The Arrhenius form follows from transition-state theory for thermally activated jumps. An atom vibrates in a potential well with attempt frequency $\nu$ (order $10^{13}\ \mathrm{s^{-1}}$). The probability per attempt of crossing a saddle point of free-energy height $\Delta G_m = \Delta H_m - T\Delta S_m$ is the Boltzmann factor, so with $z$ equivalent target sites the total jump frequency is

$$
\Gamma_j = z\,\nu\,\exp\!\left(\frac{\Delta S_m}{k_B}\right)\exp\!\left(-\frac{\Delta H_m}{k_B T}\right). \tag{P22.2}
$$

Next, relate jumps to $D$. After $n_j = \Gamma_j t$ independent jumps of length $a_j$ in random directions $\mathbf e_k$, the mean squared displacement is $\langle|\sum_k a_j\mathbf e_k|^2\rangle = a_j^2\sum_{k,l}\langle\mathbf e_k\cdot\mathbf e_l\rangle = n_j a_j^2$, because $\langle\mathbf e_k\cdot\mathbf e_l\rangle = \delta_{kl}$. For correlated successive jumps (vacancy mechanisms) the cross terms do not vanish and the result is $f n_j a_j^2$ with the correlation factor $f \le 1$; $f = 1$ for independent jumps. The continuum diffusion equation started from a point source gives the Gaussian $G = (4\pi Dt)^{-3/2}e^{-r^2/(4Dt)}$, whose second moment is $\int_0^\infty 4\pi r^4 G\,dr = 6Dt$ (Einstein relation in three dimensions; verified by SymPy in `sec5_arrhenius.py`). Equating the two:

$$
6Dt = f\,\Gamma_j t\,a_j^2 \quad\Longrightarrow\quad D = \frac{f\,\Gamma_j a_j^2}{6}. \tag{P22.3}
$$

Status: approximate (random-walk model). Inserting (P22.2) and multiplying numerator and denominator of the exponent by $N_A$:

$$
D = \underbrace{\frac{f z a_j^2\nu}{6}\,e^{\Delta S_m/k_B}}_{D_0}\;\exp\!\left(-\frac{N_A\Delta H_m}{N_A k_B T}\right), \qquad Q = N_A\,\Delta H_m, \tag{P22.4}
$$

$$
R_g = N_A\,k_B. \tag{P22.5}
$$

So $D_0$ is "attempt frequency times jump distance squared" with geometric and entropic factors, and $Q$ is the molar migration enthalpy. Order of magnitude (approximate, script output): $a_j = 0.3\ \mathrm{nm}$, $\nu = 10^{13}\ \mathrm{s^{-1}}$, $z = 4$ to $12$, $\Delta S_m = 0$ give $D_0 \approx 6\times10^{-7}$ to $1.8\times10^{-6}\ \mathrm{m^2\ s^{-1}}$.

Caveat: the pre-exponential factors of the Arrhenius rows in TECDOC-978 Tables A-1 to A-3 span $8.8\times10^{-15}$ (I, Kr, Xe in UO$_2$, FRG, accident-condition branch 1; IAEA, 1997, Table A-1, p. 439) to $1.8\times10^{6}\ \mathrm{m^2\ s^{-1}}$ (Sr in SiC, FRG, branch 2; Table A-3, p. 446), about 20.3 decades (calculated, `sec5chk_extra.py`). Twenty decades cannot be explained by (P22.4) with physical $\nu$ and $a_j$. The fitted parameters are effective quantities that lump lattice, grain-boundary and pore transport, nano-cracking and trapping, which is how BISON and PARFUME also treat them (Collin et al., 2021, p. 3, Sec. 2.3; Hales et al., 2021, p. 15, Sec. 6; via audit note 06). STRATO should therefore treat $D_0$ and $Q$ as fit parameters and avoid any physical interpretation of an individual pair.

#### 5.1.3 Several branches: parallel mechanisms

If two mechanisms carry flux in parallel through the same material and are driven by the same concentration gradient (local equilibrium between paths), their fluxes add, $J = -(D_a + D_b)\nabla c$, which gives the sum in (P22.1). (Mechanisms in series would combine harmonically instead; Eq. (A-1) does not cover that case.) TECDOC-978 uses two branches for Cs and Sr in SiC, Cs in UO$_2$, I, Kr, Xe in UO$_2$ under accident conditions, and Kr, Xe in PyC (IAEA, 1997, Tables A-1 to A-3, pp. 439, 442, 446).

The apparent activation energy measured from the local slope of an Arrhenius plot is, with $\beta = 1/T$ and $D_m(T) = D_{0,m}e^{-Q_m\beta/R_g}$,

$$
Q_{\mathrm{eff}}(T) = -R_g\frac{d\ln D}{d(1/T)} = -R_g\frac{\sum_m D_{0,m}(-Q_m/R_g)e^{-Q_m\beta/R_g}}{\sum_m D_m} = \frac{\sum_m Q_m D_m(T)}{\sum_m D_m(T)}. \tag{P22.6}
$$

So $Q_{\mathrm{eff}}$ is the $D$-weighted mean of the branch energies and moves from $Q_1$ to $Q_2$ as $T$ rises. The two branches are equal where $D_{0,1}e^{-Q_1/(R_gT_x)} = D_{0,2}e^{-Q_2/(R_gT_x)}$; taking logarithms, $\ln(D_{0,2}/D_{0,1}) = (Q_2 - Q_1)/(R_g T_x)$, so

$$
T_x = \frac{Q_2 - Q_1}{R_g\,\ln(D_{0,2}/D_{0,1})}. \tag{P22.7}
$$

Calculated crossover temperatures (exact evaluation of (P22.7), $R_g$ exact SI value):

| Correlation (IAEA, 1997) | $T_x$ (K) | $T_x$ ($^\circ$C) |
|---|---|---|
| Cs in SiC, FRG, $\Phi_f = 0$ (Table A-3, p. 446) | 1772.4 | 1499.3 |
| Cs in SiC, FRG, $\Phi_f = 4.5$ | 1835.0 | 1561.9 |
| Sr in SiC, FRG (Table A-3) | 2016.9 | 1743.8 |
| Cs in UO$_2$, FRG (Table A-1, p. 439) | 2014.1 | 1741.0 |
| Kr, Xe in PyC, FRG (Table A-2, p. 442) | 2571.3 | 2298.1 |

Table: 5.2 Crossover temperatures of the two-branch correlations.

The Cs-in-SiC crossover sits inside the 1600 to 1800 $^\circ$C range of the AGR-1 and CRP-6 heating tests. A heating test therefore samples both branches, and the effective activation energy of SiC changes during a ramp ($Q_{\mathrm{eff}} = 130\ \mathrm{kJ\ mol^{-1}}$ at 1250 $^\circ$C and $438\ \mathrm{kJ\ mol^{-1}}$ at 1600 $^\circ$C, script output). This alone breaks the shared-scaling condition of Section 5.3 for any multilayer case built on these data.

#### 5.1.4 Value of the gas constant

Since the 2019 SI revision $N_A = 6.02214076\times10^{23}\ \mathrm{mol^{-1}}$ and $k_B = 1.380649\times10^{-23}\ \mathrm{J\ K^{-1}}$ are exact, so (P22.5) gives $R_g = 8.31446261815324\ \mathrm{J\ mol^{-1}\ K^{-1}}$ exactly (printed in truncated form as 8.314462618). The script confirms that `scipy.constants.R` (SciPy 1.15.3) equals $N_Ak_B$ to machine precision; a document citation for the SI values was not opened in this session (NOT VERIFIED against a printed source). TECDOC-978 prints $R = 8.3143\ \mathrm{J\ mol^{-1}\ K^{-1}}$ (IAEA, 1997, p. 435, p. 165 and p. 470). TECDOC-1674 Eq. (10.5a) uses $8.3145$ (IAEA, 2012, p. 532) and BISON uses $8.3145$ (Collin et al., 2021, p. 10, Eq. 71) (both via audit notes 04 and 06).

Let $R_g' = R_g(1+\delta)$. For one branch,

$$
\frac{D(R_g')}{D(R_g)} = \exp\!\left[\frac{Q}{T}\left(\frac1{R_g} - \frac1{R_g'}\right)\right] = \exp\!\left[\frac{Q}{R_gT}\,\frac{\delta}{1+\delta}\right] \approx 1 + \frac{Q}{R_gT}\,\delta. \tag{P22.8}
$$

For a multi-branch law, $d\ln D/d\ln R_g = Q_{\mathrm{eff}}/(R_gT)$ by the same steps as (P22.6), so $Q$ is replaced by $Q_{\mathrm{eff}}$ to first order. Here $\delta = (8.31446261815 - 8.3143)/8.3143 = 1.956\times10^{-5}$. With $Q_{\mathrm{eff}}/(R_gT)$ up to 46.1 for the FRG rows between 1000 and 2000 $^\circ$C, the relative change of $D$ is at most $9.0\times10^{-4}$ (Sr in UO$_2$ at 1000 $^\circ$C; script table, confirmed by `sec5chk_extra.py`). An individual branch can have a larger $Q/(R_gT)$ (87 for the Kr, Xe in PyC branch with $Q = 923\ \mathrm{kJ\ mol^{-1}}$ at 1000 $^\circ$C), but its weight in $D$ is negligible there. On a release fraction the effect is smaller still: CRP-6 case 1a (bare kernel, two-branch FRG Cs, $r_k = 250\ \mu\mathrm{m}$, 200 h at 1200 $^\circ$C) gives $F = 0.467337$ with 8.3143 and $F = 0.467404$ with the SI value (8.3145 gives 0.467419). Only the 8.3143 value rounds to the printed analytical 0.4673 (IAEA, 2012, Table 10.6, p. 544); the SI value rounds to 0.4674, so a fourth-digit comparison with that table needs the TECDOC-978 constant (B3 check `b3_sec5.py`, T1f). Case 1b changes by $1.9\times10^{-9}$.

Recommendation: STRATO uses the exact SI value by default and offers $R_g = 8.3143$ as an input option when a TECDOC-978-based benchmark must be reproduced to four or more digits. The difference is far below the spread between national correlations for the same nuclide and layer (compare the Cs-in-SiC rows 22 to 30 of Table A-3) and below the effect of a 1 % temperature uncertainty (factor 1.27 on $D_{\mathrm{SiC}}$ for Cs at 1600 $^\circ$C, Section 5.5.1); it matters only for code-to-code comparison.

#### 5.1.5 Validity ranges and the fluence factor

- TECDOC-978 prints validity ranges only for some rows, for example Japan Cs in SiC 1600 to 1900 $^\circ$C and Japan Cs in UO$_2$ 1400 to 1720 $^\circ$C; the FRG and USA rows have blank range cells, except USA Cs and Sr in PyC (1200 to 1800 and 1000 to 1650 $^\circ$C, Table A-2) and the FRG I, Kr, Xe in UO$_2$ rows, which carry the labels "NOC" and "AC" instead of a range (IAEA, 1997, Tables A-1 to A-3; audit note 03).
- Model statements that override the Arrhenius rows: "rapid diffusive transport" for metallic fission products in LTI-PyC above 1900 $^\circ$C (IAEA, 1997, p. 437); German safety analyses assume rapid transport in matrix graphite above 2000 $^\circ$C for Cs and Sr (p. 448).
- Fluence factor for FRG Cs in SiC (IAEA, 1997, Table A-3 footnote 1, p. 446):

$$
D_{0,\mathrm{SiC},1}(\Phi_f) = 5.5\times10^{-14}\,\exp(\Phi_f/5)\ \mathrm{m^2\ s^{-1}},\qquad \Phi_f\ \text{in}\ 10^{25}\ \mathrm{m^{-2}},\ E > 16\ \mathrm{fJ}. \tag{P22.9}
$$

The factor is 1.000, 1.221, 1.492, 1.822 and 2.460 at $\Phi_f$ = 0, 1, 2, 3 and 4.5 (calculated). Because (P22.9) acts on one branch of one layer, it changes the ratio $D_{\mathrm{SiC}}/D_{\mathrm{kernel}}$ and moves the crossover (1499 to 1562 $^\circ$C between $\Phi_f = 0$ and 4.5); during irradiation $\Phi_f(t)$ grows, so $D$ varies in time even at constant $T$, while during a post-irradiation heating test $\Phi_f$ is frozen at its end-of-irradiation value (assumption A5). A separate problem is the energy cutoff: AGR-1 fluences are quoted for $E > 0.18\ \mathrm{MeV}$ (Collin et al., 2016, Table 1, p. 379), while (P22.9) uses $E > 16\ \mathrm{fJ}$ ($0.1\ \mathrm{MeV}$); the conversion factor between the two cutoffs is NOT VERIFIED and must not be guessed.

#### 5.1.6 Units, Kelvin conversion and input validation

All internal arithmetic uses kelvin:

$$
T = T_C + 273.15\ \mathrm{K}. \tag{P22.10}
$$

Input rules for the Phase 2.2 Arrhenius block (each tested by the reference validator in `sec5_arrhenius.py`):

| Quantity | Rule | Reason |
|---|---|---|
| $D_{0,i,m}$ | finite and $> 0$, in $\mathrm{m^2\ s^{-1}}$ | $\ln D_0$ must exist |
| $Q_{i,m}$ | finite and $\ge 0$; mandatory unit tag ($\mathrm{kJ\ mol^{-1}}$ or $\mathrm{J\ mol^{-1}}$) | TECDOC-978 text defines Q in J/mol while every table column is in kJ/mol (audit note 03); $Q = 0$ is a real row (I in matrix pores, row 44) |
| $T$ | finite and $> 0\ \mathrm{K}$ after conversion; mandatory unit tag | a missing tag turns 1600 $^\circ$C into 1600 K |
| $R_g$ | default exact SI value; optional 8.3143 | Section 5.1.4 |
| $\Phi_f$ | finite, $\ge 0$, with energy cutoff tag | (P22.9) |
| validity range | stored when printed; warning (never silent clamping) on extrapolation | Section 5.1.5 |
| $D$ evaluation | compute $\ln D_m = \ln D_0 - Q/(R_gT)$ and combine branches by log-sum-exp | $e^{-Q/(R_gT)}$ underflows to 0 for $Q = 923\ \mathrm{kJ\ mol^{-1}}$ at 100 K while $\ln D = -1097.9$ stays finite |
| floor | $D \ge D_{\min} > 0$ with a warning | the FV conductance divides by $D$; the series uses $k_i = \sqrt{\Lambda/D_i}$ |
| breakpoints | times strictly increasing, first time 0 | Section 5.2.4 |

Table: 5.3 Unit and input-validation rules.

### 5.2 (b) Finite volume with $D(T(t))$

#### 5.2.1 Continuous problem

$$
\frac{\partial c}{\partial t} = \frac{1}{r^2}\frac{\partial}{\partial r}\!\left(r^2 D(r,T(t))\frac{\partial c}{\partial r}\right) - \lambda c + S(r,t), \qquad D(r,T) = D_i(T)\ \text{for}\ R_{i-1} < r < R_i, \tag{P22.11}
$$

$$
c\ \text{and}\ D_i\,\partial_r c\ \text{continuous at}\ r = R_i,\quad \partial_r c(0,t) = 0,\quad c(R,t) = 0,\quad c(r,0) = c_{\mathrm{init}}(r). \tag{P22.12}
$$

The only change from v1.0 is that $D_i$ depends on $t$; the decay and source terms are the Phase 2.1 extensions and are carried here because Section 5.3 needs them.

#### 5.2.2 Semi-discrete equations

Integrating (P22.11) over cell $j$ (shell $[r_{j-1/2}, r_{j+1/2}]$, volume $V_j = \tfrac{4\pi}{3}(r_{j+1/2}^3 - r_{j-1/2}^3)$) and approximating each face flux by two half-cell resistances in series, exactly as in v1.0 Code B (`build_mesh`, [src/numerical/finite_volume.rs, lines 78 to 91](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L78-L91)), gives the semi-discrete system (numerical, continuous in time):

$$
V_j\frac{dC_j}{dt} = G_{j-1/2}(t)\,(C_{j-1} - C_j) - G_{j+1/2}(t)\,(C_j - C_{j+1}) - \lambda V_jC_j + V_jS_j(t), \tag{P22.13}
$$

with $G_{1/2} = 0$ (zero-area centre) and $C_{N+1} = 0$ (absorbing surface), and

$$
G_f(t) = \frac{A_f}{\delta_f^-/D_{i(f^-)}(T(t)) + \delta_f^+/D_{i(f^+)}(T(t))},\qquad G_{N+1/2}(t) = \frac{4\pi R^2 D_5(T(t))}{R - \bar r_N}, \tag{P22.14}
$$

where $\bar r_j = \tfrac34(b^4 - a^4)/(b^3 - a^3)$ is the volume centroid of the shell $[a,b]$, $\delta_f^- = r_f - \bar r_{j}$, $\delta_f^+ = \bar r_{j+1} - r_f$ and $A_f = 4\pi r_f^2$. Only $D$ changes with time. The geometric data $A_f$, $\delta_f^\pm$ are computed once; recomputing all $G_f$ costs $O(N)$ per step, the same order as the tridiagonal solve.

#### 5.2.3 Backward Euler with recomputed conductances

$$
V_j\frac{C_j^{n+1} - C_j^n}{\Delta t_n} = G^{(n)}_{j-1/2}(C^{n+1}_{j-1} - C^{n+1}_j) - G^{(n)}_{j+1/2}(C^{n+1}_j - C^{n+1}_{j+1}) - \lambda V_jC_j^{n+1} + V_jS_j^{(n)}, \tag{P22.15}
$$

with one of two time-level choices for the conductances:

- **Option E (end of step):** $G^{(n)}_f = G_f(T(t_{n+1}))$, the textbook fully implicit choice.
- **Option A (step average):** $\bar D_i^{(n)} = \frac{1}{\Delta t_n}\int_{t_n}^{t_{n+1}} D_i(T(t))\,dt$, then $G^{(n)}_f$ from (P22.14) with $\bar D_i^{(n)}$.

Both choices keep the two structural properties of v1.0. Summing (P22.15) over $j$, the interior face terms cancel in pairs (each $G_f$ appears once with each sign), so

$$
\sum_j V_j(C_j^{n+1} - C_j^n) = -\Delta t_n\,G^{(n)}_{N+1/2}C_N^{n+1} - \lambda\Delta t_n\sum_jV_jC_j^{n+1} + \Delta t_n\sum_jV_jS_j^{(n)}. \tag{P22.16}
$$

The release increment must therefore be booked as $\Delta M_{\mathrm{rel}} = \Delta t_n G^{(n)}_{N+1/2}C^{n+1}_N$ with the same $G^{(n)}_{N+1/2}$ that entered the matrix, and $\Delta M_{\mathrm{dec}} = \lambda\Delta t_n\sum_jV_jC^{n+1}_j$; then $M + M_{\mathrm{rel}} + M_{\mathrm{dec}} = M_0 + M_{\mathrm{gen}}$ holds to round-off for every temperature history. Implementation note: v1.0 `run()` uses the fixed `mesh.outer_conductance` for the release rate (line 210), which is correct only for constant $D$; Phase 2.2 must pass the step's $G_{N+1/2}^{(n)}$. Second, the matrix $V/\Delta t_n + A^{(n)} + \lambda V$ has a positive diagonal, non-positive off-diagonals and strict diagonal dominance for any $G_f \ge 0$, so it is an M-matrix:

$$
C^n \ge 0,\ S^{(n)} \ge 0 \;\Longrightarrow\; C^{n+1} \ge 0\quad\text{for every}\ T(t)\ \text{and every}\ \Delta t_n > 0. \tag{P22.17}
$$

The subtraction-free Thomas elimination of v1.0 applies unchanged because it needs only non-negative $G$.

**Accuracy of the time level.** In the shared-scaling case of Section 5.3 ($G_f(t) = \alpha(t)G_f^*$), option A gives $\Delta t_n G^{(n)} = \Delta\tau_n G^*$ with $\Delta\tau_n = \int_{t_n}^{t_{n+1}}\alpha\,dt$ (exact also at interface faces, because $G_f$ is homogeneous of degree one in the pair of diffusivities), so for $\lambda = 0$ option A is exactly backward Euler applied in the effective time $\tau$ with step $\Delta\tau_n$ (with $S \ne 0$ this needs the step source $\Delta t_nS^{(n)} = \Delta\tau_n\hat S^{(n)}$). For $\lambda > 0$ the decay term stays $\lambda\Delta t_n$ inside the implicit operator, and the iterates differ from $e^{-\lambda t_n}$ times backward Euler in $\tau$ by a first-order time-discretisation error (checker script `sec5chk_extra.py`). Without shared scaling, option A averages each $D_i$ and then forms $G_f$, which at an interface face differs from the step average of $G_f$ by $O(\Delta t_n^2)$. Option E uses $\alpha(t_{n+1})\Delta t_n$ instead. Expanding $\alpha(t_{n+1} - s) = \alpha - s\alpha' + \tfrac12 s^2\alpha'' + \dots$ (derivatives at $t_{n+1}$) and integrating over $0 \le s \le \Delta t_n$:

$$
\Delta\tau_n - \alpha(t_{n+1})\Delta t_n = -\tfrac12\alpha'(t_{n+1})\Delta t_n^2 + \tfrac16\alpha''(t_{n+1})\Delta t_n^3 + O(\Delta t_n^4). \tag{P22.18}
$$

The extra local error of option E is $O(\Delta t^2)$, the same order as the local truncation error of backward Euler itself, so both options are first order globally. Numerical evidence (Section 5.6): in the shared-$\alpha$ ramp test the observed orders are 1.00 to 1.01 (option E) and 0.98 to 1.00 (option A), with option A 2.24 times more accurate at $\Delta t = 90\ \mathrm{s}$; in the layer-specific-$Q$ test the orders are 0.97 to 0.99 (E) and 0.99 to 1.00 (A), and option E is 1.75 times more accurate. Neither option is uniformly better. Recommendation: option A, because it reproduces backward Euler in $\tau$ exactly whenever the effective-time reduction holds and $\lambda = 0$ (so the v1.0 convergence evidence carries over), and because its integral of each $D_i$ over a step is exact even when a step contains a kink of $T(t)$.

#### 5.2.4 Step control on fast temperature changes

Differentiating (P22.1) along $T(t)$ and using (P22.6):

$$
\frac{d\ln D_i}{dt} = \frac{Q_{\mathrm{eff},i}(T)}{R_gT^2}\,\frac{dT}{dt}. \tag{P22.19}
$$

Limiting the change of every layer diffusivity to $|\Delta\ln D_i| \le \varepsilon_D$ per step gives the step bound

$$
\Delta t_n \le \frac{\varepsilon_D\,R_g\,T^2}{\max_i Q_{\mathrm{eff},i}(T)\,|dT/dt|}. \tag{P22.20}
$$

Calculated examples with $\varepsilon_D = 0.1$ (about 10.5 % change of $D$ per step):

| Correlation | $T$ ($^\circ$C) | $dT/dt$ ($\mathrm{K\ h^{-1}}$) | $Q_{\mathrm{eff}}$ ($\mathrm{kJ\ mol^{-1}}$) | bound (s) |
|---|---|---|---|---|
| Cs in SiC, FRG | 1600 | 50 | 438.2 | 479 |
| Cs in SiC, FRG | 1250 | 120 | 130.1 | 445 |
| Cs in SiC, FRG | 1600 | $-600$ | 438.2 | 40 |
| Sr in SiC, FRG | 1800 | 50 | 627.3 | 410 |
| Cs in UO$_2$, FRG | 1600 | 50 | 260.2 | 807 |

Table: 5.4 Step bounds from Eq. (P22.20) for example ramps.

For reference, the CRP-6 participants used heating time steps from 86.4 s (lower end of the COPA range) to automatically adjusted steps below 300 000 s (ATLAS) (IAEA, 2012, Table 10.4, p. 542; audit note 04). Rule (P22.20) is a resolution criterion for the coefficients; the accuracy of the solution is controlled separately by step doubling, using $|C_{\Delta t} - C_{\Delta t/2}|$ as the first-order error estimate. v1.0 uses one uniform step (`run()`, line 191); Phase 2.2 needs a variable step.

**Tabulated $T(t)$.** A schedule given as breakpoints $(t_k, T_k)$ is interpolated linearly in $T$:

$$
T(t) = T_k + (T_{k+1} - T_k)\,\frac{t - t_k}{t_{k+1} - t_k}, \qquad t_k \le t \le t_{k+1}. \tag{P22.21}
$$

Linear interpolation of $T$ (rather than of $D$ or $\ln D$) reproduces furnace ramps specified in $^\circ\mathrm{C\ h^{-1}}$. The step sequence must land exactly on every breakpoint and every output time, $t_{n+1} = \min(t_n + \Delta t_n,\ \text{next breakpoint},\ \text{next output})$: $dT/dt$ jumps at a breakpoint (and $T$ itself jumps in step schedules such as CRP-6 case 3c), and a step straddling the jump loses the smoothness on which the truncation-error expansion rests.

**Exact integral of $D$ over a linear ramp.** For option A and for the effective time, the integral of one Arrhenius branch over a segment with constant $\dot T = dT/dt \ne 0$ is needed. Substituting $dt = dT/\dot T$ and writing $\theta = Q/R_g$ (unit K):

$$
\int_{t_a}^{t_b} e^{-\theta/T(t)}\,dt = \frac{1}{\dot T}\int_{T_a}^{T_b}e^{-\theta/T}\,dT = \frac{1}{\dot T}\Big[\,T\,E_2(\theta/T)\,\Big]_{T_a}^{T_b}, \tag{P22.22}
$$

$$
E_n(x) = \int_1^\infty e^{-xs}s^{-n}\,ds,\qquad E_2(x) = e^{-x} - xE_1(x). \tag{P22.23}
$$

Proof: with $x = \theta/T$, $dE_1/dT = (dE_1/dx)(dx/dT) = (-e^{-x}/x)(-\theta/T^2) = e^{-\theta/T}/T$, so $\frac{d}{dT}[Te^{-\theta/T} - \theta E_1(\theta/T)] = e^{-\theta/T}(1 + \theta/T) - \theta e^{-\theta/T}/T = e^{-\theta/T}$, and $Te^{-\theta/T} - \theta E_1(\theta/T) = TE_2(\theta/T)$ by (P22.23). For $\dot T = 0$ the integral is $(t_b - t_a)e^{-\theta/T}$. Status: exact. The closed form agrees with adaptive quadrature to $\le 2.4\times10^{-15}$ relative on four test ramps, including the AGR-1 cool-down. For very short steps the difference of two nearly equal $TE_2$ values cancels digits, so a step average uses 8-point Gauss-Legendre quadrature (relative error near $10^{-15}$ in the scripts) and whole segments use (P22.22).

### 5.3 (c) Effective time

#### 5.3.1 Shared scaling

Suppose every layer scales with the same factor:

$$
D_i(T(t)) = \alpha(t)\,D_i^*,\qquad i = 1,\dots,5,\qquad \alpha(t) > 0. \tag{P22.24}
$$

A sufficient condition is $D_i(T) = \kappa_i \chi(T)$ with one function $\chi$ for all layers. For single-branch Arrhenius laws this means one shared activation energy $Q$ (the $D_{0,i}$ may differ), and then

$$
\alpha(t) = \exp\!\left[-\frac{Q}{R_g}\left(\frac{1}{T(t)} - \frac{1}{T_{\mathrm{ref}}}\right)\right],\qquad D_i^* = D_i(T_{\mathrm{ref}}). \tag{P22.25}
$$

Multi-branch laws satisfy the condition only when every layer has the same set of $Q_m$ with $D_{0,i,m}/D_{0,j,m}$ independent of $m$. A single-layer problem (bare kernel, CRP-6 cases 1a and 1b, Gelbard's Section II) satisfies it for any $D(T)$, with $\alpha = D(T(t))/D(T_{\mathrm{ref}})$. Define the effective time

$$
\tau(t) = \int_0^t \alpha(s)\,ds. \tag{P22.26}
$$

Because $\alpha > 0$, $\tau$ is strictly increasing and has an inverse $t(\tau)$; $\tau$ has the unit of time and equals the time that would give the same diffusion at $T_{\mathrm{ref}}$. On a linear ramp $\tau$ follows from (P22.22).

#### 5.3.2 Derivation

Try

$$
c(r,t) = e^{-\lambda t}\,\hat c(r,\tau(t)). \tag{P22.27}
$$

By the chain rule, with $L^*\hat c = r^{-2}\partial_r(r^2D^*(r)\partial_r\hat c)$:

$$
\frac{\partial c}{\partial t} = -\lambda e^{-\lambda t}\hat c + e^{-\lambda t}\alpha(t)\,\partial_\tau\hat c,\qquad \frac{1}{r^2}\partial_r\!\left(r^2\alpha D^*\partial_rc\right) = \alpha(t)\,e^{-\lambda t}L^*\hat c. \tag{P22.28}
$$

Substituting both into (P22.11): $-\lambda e^{-\lambda t}\hat c + e^{-\lambda t}\alpha\,\partial_\tau\hat c = \alpha e^{-\lambda t}L^*\hat c - \lambda e^{-\lambda t}\hat c + S$. The two decay terms cancel identically. Dividing by $\alpha e^{-\lambda t} > 0$:

$$
\partial_\tau\hat c = L^*\hat c + \hat S(r,\tau), \tag{P22.29}
$$

$$
\hat S(r,\tau) = \frac{e^{\lambda t(\tau)}\,S(r,t(\tau))}{\alpha(t(\tau))}. \tag{P22.30}
$$

Conditions: continuity of $c$ gives continuity of $\hat c$ (common factor $e^{-\lambda t}$); the flux condition $D_i\partial_rc^- = D_{i+1}\partial_rc^+$ becomes $\alpha e^{-\lambda t}(D_i^*\partial_r\hat c^- - D_{i+1}^*\partial_r\hat c^+) = 0$; $c(R) = 0$ gives $\hat c(R) = 0$; and $\tau(0) = 0$ gives the same initial field:

$$
\hat c,\ D_i^*\partial_r\hat c\ \text{continuous at}\ R_i,\quad \hat c(R,\tau) = 0,\quad \partial_r\hat c(0,\tau) = 0,\quad \hat c(r,0) = c_{\mathrm{init}}(r). \tag{P22.31}
$$

So $\hat c$ solves the v1.0 problem (constant $D_i^*$) in the variable $\tau$, with an extra source $\hat S$. Status: exact; no step above is approximate.

**Why the decay factor matters.** Without the factor, set $\tilde c(r,\tau) = c(r,t(\tau))$; then $\partial_t c = \alpha\,\partial_\tau\tilde c$ and

$$
\partial_\tau\tilde c = L^*\tilde c - \frac{\lambda}{\alpha(t(\tau))}\,\tilde c + \frac{S}{\alpha}. \tag{P22.32}
$$

The decay rate in $\tau$ is $\lambda/\alpha$, which varies along a ramp, so (P22.32) is not of Phase 1 form unless $\lambda = 0$ or $\alpha$ is constant. The factor $e^{-\lambda t}$ removes the decay because $\lambda$ is the same at every radius (assumption A4), so multiplication by $e^{-\lambda t}$ commutes with $L^*$. If $\lambda$ differed between layers this would fail.

#### 5.3.3 Inventories and release

With $M(t) = 4\pi\int_0^R r^2c\,dr$ and the $\tau$-space inventory $\hat M(\tau)$:

$$
M(t) = e^{-\lambda t}\hat M(\tau(t));\qquad \lambda = 0,\ S = 0:\ \ F(t) = F_{\mathrm{v1}}(\tau(t)) = 1 - \sum_n w_n e^{-\Lambda_n^*\tau(t)}. \tag{P22.33}
$$

The release rate scales with $\alpha$ through $D_5$:

$$
J_R(t) = -4\pi R^2\alpha D_5^*\,\partial_r c\vert_R = \alpha(t)\,e^{-\lambda t}\,\hat J_R(\tau(t)),\qquad \hat J_R = \frac{d\hat M_{\mathrm{rel}}}{d\tau}. \tag{P22.34}
$$

The cumulative release is $M_{\mathrm{rel}}(t) = \int_0^t J_R\,ds = \int_0^t e^{-\lambda s}\,d\hat M_{\mathrm{rel}}(\tau(s))$. Integrating by parts with $\hat M_{\mathrm{rel}}(0) = 0$:

$$
M_{\mathrm{rel}}(t) = e^{-\lambda t}\hat M_{\mathrm{rel}}(\tau(t)) + \lambda\int_0^t e^{-\lambda s}\hat M_{\mathrm{rel}}(\tau(s))\,ds. \tag{P22.35}
$$

For $\lambda = 0$ this collapses to $M_{\mathrm{rel}}(t) = \hat M_{\mathrm{rel}}(\tau(t))$. For $\lambda > 0$ the release depends on the whole path $\tau(s)$, $s \le t$, and needs one quadrature (semi-analytical). The quadrature uses only inventories, which converge faster than the flux series. The first relation in (P22.33) holds for any source (then $\hat M$ comes from $\hat c$ with $\hat S$); with $S = 0$ and $\lambda > 0$ it reads $M(t)/M_0 = e^{-\lambda t}\sum_nw_ne^{-\Lambda_n^*\tau(t)}$. Closure holds with $M_{\mathrm{dec}}(t) = \lambda\int_0^t M\,ds$ (decay inside the particle): $M + M_{\mathrm{rel}} + M_{\mathrm{dec}} = M_0$ for $S = 0$, with $M_{\mathrm{gen}}(t)$ added on the right when $S \ne 0$ (SymPy: the time derivative of the sum vanishes and the sum equals $M_0$ at $t = 0$). Released atoms keep decaying outside: $dM_{\mathrm{out}}/dt = J_R - \lambda M_{\mathrm{out}}$, $M_{\mathrm{out}}(t) = \int_0^t e^{-\lambda(t-s)}J_R(s)\,ds$.

Single-sphere limit: with $w_n = 6/(n^2\pi^2)$ and $\Lambda_n^* = n^2\pi^2D^*/R^2$, $1 - M/M_0 = 1 - \frac{6e^{-\lambda t}}{\pi^2}\sum_n n^{-2}\exp(-n^2\pi^2\int_0^tD\,ds/R^2)$, which is Gelbard's Eq. (II.18) with his time-averaged diffusivity, Eqs. (II.13) and (II.14) (Gelbard, 2003, pp. 12 to 14). Gelbard calls this quantity "the fraction released or decayed"; in the notation here it is $(M_{\mathrm{rel}} + M_{\mathrm{dec}})/M_0$, and (P22.35) separates the two parts. Gelbard gives the transform for one homogeneous region only; (P22.24) to (P22.37) are its multilayer generalisation.

#### 5.3.4 Modal form with source

The eigenpairs $(\Lambda_n^*, \phi_n^*)$ of $L^*$ with conditions (P22.31) are time independent. Projecting (P22.29) on $\phi_n^*$ (weight $r^2$) gives $d\hat a_n/d\tau = -\Lambda_n^*\hat a_n + \hat s_n(\tau)$, with solution $\hat a_n(\tau) = \hat a_n(0)e^{-\Lambda_n^*\tau} + \int_0^\tau e^{-\Lambda_n^*(\tau - \sigma)}\hat s_n(\sigma)\,d\sigma$. Substitute $\sigma = \tau(s)$, $d\sigma = \alpha(s)\,ds$, $\hat s_n(\tau(s)) = e^{\lambda s}s_n(s)/\alpha(s)$ with $s_n(t) = N_n^{-1}\int_0^R r^2\phi_n^*S\,dr$, and multiply by $e^{-\lambda t}$ ($a_n = e^{-\lambda t}\hat a_n$). The factor $\alpha(s)$ cancels:

$$
a_n(t) = a_n(0)\,e^{-\lambda t - \Lambda_n^*\tau(t)} + \int_0^t e^{-\lambda(t-s) - \Lambda_n^*[\tau(t) - \tau(s)]}\,s_n(s)\,ds, \tag{P22.36}
$$

which solves $da_n/dt = -(\lambda + \alpha(t)\Lambda_n^*)a_n + s_n(t)$ (SymPy residual 0). Status: exact for each mode; semi-analytical when the integral needs quadrature; series in $n$ with the v1.0 truncation rule $\Lambda_{\max}^*\,\tau \ge 40$. Limiting case, constant $\alpha = \alpha_0$ and constant $s_n$:

$$
a_n(t) = a_n(0)e^{-(\lambda + \alpha_0\Lambda_n^*)t} + \frac{s_n\left(1 - e^{-(\lambda + \alpha_0\Lambda_n^*)t}\right)}{\lambda + \alpha_0\Lambda_n^*}, \tag{P22.37}
$$

with the steady state $s_n/(\lambda + \alpha_0\Lambda_n^*)$ as $t \to \infty$.

#### 5.3.5 Surface condition

If a Robin condition $-D_5\partial_rc = H(t)\,c$ replaced the absorbing surface, the transform gives $-D_5^*\partial_r\hat c = (H/\alpha)\,\hat c$:

$$
\text{Robin surface:}\quad \frac{H(t)}{\alpha(t)} = \text{const}\ \ \text{is required.} \tag{P22.38}
$$

v1.0 and this section use $c(R) = 0$, which carries no such constraint.

#### 5.3.6 When the effective-time solution is exact

The effective-time solution (P22.33) to (P22.37) is exact when all of the following hold (sufficient conditions; E1 is also necessary for one $\tau$ to serve every layer, while E2 to E5 are the conditions used in the derivation and are not proven necessary):

- E1. One shared factor $\alpha(t)$ multiplies every layer diffusivity (P22.24); the ratios $D_i/D_j$ never change.
- E2. $T$ is uniform in $r$ (A1); a radial temperature profile makes $\alpha$ depend on $r$.
- E3. $\lambda$ is uniform (A4); a source $S$ of any shape is allowed and enters through (P22.30) or (P22.36).
- E4. The surface is absorbing, or a Robin coefficient obeys (P22.38).
- E5. No temperature-dependent partition coefficients at interfaces (A2).

With $S = 0$ and $\lambda = 0$ the result is v1.0 evaluated at $\tau(t)$, closed form apart from the series truncation. With $\lambda > 0$ the inventory stays closed form and the release needs the quadrature (P22.35). With $S \ne 0$ each mode needs the quadrature in (P22.36) unless $S$ and $\alpha$ make it elementary. If E1 fails, any single $\tau$ is an approximation of unknown accuracy and Section 5.4 applies. For TECDOC-978 data E1 fails in every multilayer case (for Cs, FRG: kernel 209 and 362, PyC 222, SiC 125 and 514 $\mathrm{kJ\ mol^{-1}}$; the FRG buffer uses a constant $10^{-8}\ \mathrm{m^2\ s^{-1}}$ with zero activation energy, IAEA, 1997, p. 440), so in STRATO the multilayer effective-time solution serves as an exact verification case with synthetic shared-$Q$ data, while the single-layer case (bare kernel) is exact with real data.

Numerical demonstration (`sec5_fv_shared_alpha.py`, synthetic shared $Q = 300\ \mathrm{kJ\ mol^{-1}}$, AGR-1 nominal radii, ramp 1250 to 1600 $^\circ$C at 50 $\mathrm{K\ h^{-1}}$ then hold): the semi-discrete FV solution evaluated at $\tau(t)$ (exact in time through its eigen-decomposition) converges to the independent series at $\tau(t)$ with observed spatial order 2.00 and a difference of $5.4\times10^{-7}$ at 160 cells per layer; backward Euler run in real time with recomputed conductances converges to that semi-discrete solution at first order (options E and A); with $t_{1/2} = 10\ \mathrm{h}$, a stiff real-time integration matches $e^{-\lambda t}\hat M(\tau(t))$ to $1.6\times10^{-12}$ and (P22.35) to $2.8\times10^{-12}$ (relative to $M_0$).

### 5.4 (d) General multilayer case: layer-specific activation energies

When the ratios $D_i/D_j$ change with $T$, the eigenfunctions change with $T$: the interface condition $D_i u_i' - D_{i+1}u_{i+1}' = (D_i - D_{i+1})u/r$ (the v1.0 rule in $u = rc$) depends on the ratios. Four approaches follow.

#### 5.4.1 (i) Piecewise-constant temperature with eigenfunction projection

Let

$$
T(t) = T_k\quad\text{for}\quad t_k \le t < t_{k+1},\qquad k = 0,\dots,N_T-1. \tag{P22.39}
$$

On interval $k$ the operator $L^{(k)}$ has constant coefficients $D_i(T_k)$ and eigenpairs $(\Lambda_n^{(k)}, \phi_n^{(k)})$ found by the v1.0 shooting and Sturm-count machinery, so

$$
c(r,t) = \sum_n a_n^{(k)}(t)\phi_n^{(k)}(r),\qquad a_n^{(k)}(t) = e^{-(\Lambda_n^{(k)} + \lambda)(t - t_k)}a_n^{(k)}(t_k) + \int_{t_k}^t e^{-(\Lambda_n^{(k)} + \lambda)(t - s)}s_n^{(k)}(s)\,ds. \tag{P22.40}
$$

At a switch the field is continuous in time,

$$
c(r,t_k^-) = c(r,t_k^+), \tag{P22.41}
$$

because integrating (P22.11) over $(t_k - \epsilon, t_k + \epsilon)$ gives a change bounded by $2\epsilon$ times a bounded right-hand side (valid for $t_k > 0$, where the field is smooth inside each layer); a jump in $D$ changes the flux, never $c$ itself. Expanding the continuous field in the new basis with weight $r^2$:

$$
a_m^{(k)}(t_k) = \frac{1}{N_m^{(k)}}\int_0^R r^2\phi_m^{(k)}c(r,t_k)\,dr = \sum_n P_{mn}^{(k)}\,a_n^{(k-1)}(t_k), \tag{P22.42}
$$

$$
P_{mn}^{(k)} = \frac{1}{N_m^{(k)}}\int_0^R r^2\phi_m^{(k)}\phi_n^{(k-1)}\,dr = \frac{1}{N_m^{(k)}}\int_0^R u_m^{(k)}u_n^{(k-1)}\,dr. \tag{P22.43}
$$

Both bases are orthogonal under the same weight $r^2$ because, without partition coefficients, the weight does not depend on $D$. (With partition coefficients $\gamma_i$ the weight becomes $\gamma_i r^2$ and changes with $T$ if $\gamma_i$ does.) Status: exact for the infinite series under piecewise-constant $T$, since the eigenfunctions are complete in $L^2(r^2dr)$ and the old series converges in that norm. In the shared-$\alpha$ case the eigenfunctions do not change, $P = I$, and the method reduces to Section 5.3.

**Truncation.** With $n_{\max}$ modes per interval, two errors appear. The omitted old modes carry amplitudes damped by $e^{-\Lambda_n^{(k-1)}(t_k - t_{k-1})}$, which gives the criterion (the v1.0 `min_valid_time` rule, $\Lambda_{\mathrm{covered}}\,t \ge 40$ in `sturm_liouville.rs`, applied per interval)

$$
\Lambda_{n_{\max}+1}^{(k-1)}\,(t_k - t_{k-1}) \ge 40. \tag{P22.44}
$$

The omitted new modes affect only outputs with $t - t_k < 40/\Lambda_{n_{\max}+1}^{(k)}$. A cheap diagnostic is the inventory mismatch at the switch,

$$
\Delta M_k = 4\pi\Big(\sum_m a_m^{(k)}(t_k)I_m^{(k)} - \sum_n a_n^{(k-1)}(t_k)I_n^{(k-1)}\Big),\qquad I_n = \int_0^R r^2\phi_n\,dr, \tag{P22.45}
$$

which is zero for the infinite series.

**How fast do the new coefficients decay?** Let $g = c(\cdot,t_k)$: continuous, $g(R) = 0$, smooth inside layers, and with $D_i^{(k-1)}g'$ continuous, so $q_i = D_i^{(k-1)}g'(R_i^-) = D_{i+1}^{(k-1)}g'(R_i^+)$. Write $\rho_i = D_i^{(k)}/D_i^{(k-1)}$. Using $L^{(k)}\phi_m = -\Lambda_m\phi_m$ and integrating by parts twice on each layer, the terms in $g\,r^2D\phi_m'$ telescope (both factors continuous) and vanish at $r = 0$ and $r = R$; the terms in $\phi_m r^2D^{(k)}g'$ leave the jump $D_i^{(k)}g'(R_i^-) - D_{i+1}^{(k)}g'(R_i^+) = q_i(\rho_i - \rho_{i+1})$ at each interface. The result (the integral is taken layer by layer, so it contains no interface delta terms) is

$$
N_m a_m = \frac{1}{\Lambda_m}\sum_{i=1}^{4}R_i^2\,\phi_m(R_i)\,q_i\,(\rho_i - \rho_{i+1}) - \frac{1}{\Lambda_m}\sum_{i=1}^{5}\int_{R_{i-1}}^{R_i}\phi_m\,\partial_r\!\left(r^2D_i^{(k)}\partial_rg\right)dr. \tag{P22.46}
$$

Status: exact identity (checker: numerical residual $2.1\times10^{-11}$ over 12 modes, and flipping the sign of the jump term breaks it, `sec5chk_numeric.py`).

For normalised eigenfunctions, $\phi_m(R_i) = O(1)$ and $\Lambda_m = O(m^2)$, so $a_m = O(m^{-2})$ whenever some $\rho_i \ne \rho_{i+1}$ (approximate estimate). If all $\rho_i$ are equal (shared scaling) the interface term vanishes and the decay is faster. Observed: $|a_m|\sqrt{N_m} \propto m^{-2.03}$ for $m \ge 20$ (script); an independent case (projection of the first old mode, $20 \le m \le 60$, B3 check `b3_sec5.py`, T4b) gives a fitted exponent of $-1.74$, so $m^{-2}$ is an asymptotic estimate and the exponent seen at moderate $m$ depends on the case. The inventory sum converges faster than this bound because the $I_m$ alternate in sign: the observed envelope of $|\Delta M_k|/M_0$ falls as $n_{\max}^{-2.9}$ (numerical, one case only).

**Near-surface and Gibbs effects.** The derivative series $\sum_m a_m\phi_m'(R)$ has terms of order $m\cdot m^{-2}$ and converges only slowly and non-uniformly right after a switch; the reconstructed $c(r)$ also shows Gibbs-type oscillation near interfaces where the true $g'$ has the old jump ratio. Release should therefore be computed from the inventory balance $M_{\mathrm{rel}} = M_0 + M_{\mathrm{gen}} - M - M_{\mathrm{dec}}$, never from a truncated flux series near a switch. The kernel step initial condition at $t = 0$ is a separate, already known Gibbs case ($a_n = O(n^{-1})$), handled in v1.0 by `min_valid_time`.

Numerical demonstration (`sec5_projection_step.py`): synthetic layer-specific $Q$, step 1600 to 1800 $^\circ$C at $t_s = 1\ \mathrm{h}$. The full-basis projection agrees with Richardson-extrapolated FV (observed spatial order 2.00) to $6.9\times10^{-11}$ in $F$. With $t_s = 0.05\ \mathrm{h}$ the old basis must resolve a young field: $n_{\max} = 5$, 10, 20, 40 give errors $8.1\times10^{-3}$, $2.6\times10^{-4}$, $1.9\times10^{-6}$, $3.5\times10^{-13}$ at the switch, falling as $\Lambda_{n_{\max}+1}^{\mathrm{old}}t_s$ grows (0.48, 1.61, 6.12 for $n_{\max}$ = 5, 10, 20). Rule (P22.44) is conservative for this case: with $t_s = 1\ \mathrm{h}$, $n_{\max} = 10$ has $\Lambda_{11}^{\mathrm{old}}t_s = 32 < 40$ and already reaches the reference accuracy ($4.7\times10^{-11}$).

#### 5.4.2 (ii) Galerkin system on a fixed basis

Take a fixed basis $\psi_m$ (for example the eigenfunctions at $T_{\mathrm{ref}}$), continuous with $\psi_m(R) = 0$, orthogonal with weight $r^2$ and norms $N_m$. Multiply (P22.11) by $r^2\psi_m$ and integrate by parts on each layer. The boundary terms vanish at $r = 0$ (factor $r^2$) and at $r = R$ ($\psi_m(R) = 0$); the interface terms $R_i^2\psi_m(R_i)[D_i\partial_rc^- - D_{i+1}\partial_rc^+]$ vanish by flux continuity (P22.12). This leaves the weak form

$$
\int_0^R r^2\psi_m\,\partial_tc\,dr = -\int_0^R r^2D(r,T(t))\,\psi_m'\,\partial_rc\,dr - \lambda\int_0^R r^2\psi_mc\,dr + \int_0^R r^2\psi_mS\,dr. \tag{P22.47}
$$

Inserting $c \approx \sum_n a_n(t)\psi_n$ and dividing by $N_m$:

$$
\frac{d\mathbf a}{dt} = -K(t)\,\mathbf a - \lambda\,\mathbf a + \mathbf s(t), \tag{P22.48}
$$

$$
K_{mn}(t) = \frac{1}{N_m}\int_0^R r^2D(r,T(t))\,\psi_m'\psi_n'\,dr = \sum_{i=1}^5 D_i(T(t))\,K^{(i)}_{mn},\qquad K^{(i)}_{mn} = \frac{1}{N_m}\int_{R_{i-1}}^{R_i}r^2\psi_m'\psi_n'\,dr. \tag{P22.49}
$$

The affine split in (P22.49) lets the five layer matrices $K^{(i)}$ be computed once; each step only forms a weighted sum. $N^{1/2}KN^{-1/2}$ is symmetric positive definite. If $\psi = \phi^*$ and $D = \alpha D^*$, integration by parts with the eigen-equation gives

$$
\int_0^R r^2D^*\phi_m^{*\prime}\phi_n^{*\prime}\,dr = \Lambda_n^*N_n\,\delta_{mn}\quad\Longrightarrow\quad K(t) = \alpha(t)\,\mathrm{diag}(\Lambda^*), \tag{P22.50}
$$

and (P22.48) decouples into (P22.36). In general

$$
\mathbf a(t) = U(t,0)\,\mathbf a(0) + \int_0^tU(t,s)\,\mathbf s(s)\,ds,\quad U(t,s) = \mathcal T\exp\!\left(-\int_s^t[K(\sigma) + \lambda I]\,d\sigma\right),\quad [K(t_1),K(t_2)] = \sum_{i,j}D_i(t_1)D_j(t_2)\,[K^{(i)},K^{(j)}], \tag{P22.51}
$$

where $\mathcal T$ denotes time ordering and $U$ is the propagator of (P22.48). It reduces to the ordinary exponential of $\int K$ only when all $K(t)$ commute, which holds when the $D_i$ keep fixed ratios. Otherwise no general closed form is available. The solution is not even a function of the layer integrals $\int D_i\,dt$: two histories with the same $\int D_i\,dt$ in every layer, two 2 h plateaus at 1600 and 1800 $^\circ$C taken in opposite order, give $F = 0.011413$ and $0.009919$ for synthetic layer-specific $Q$ (B3 check `b3_sec5.py`, T3c; with a shared $Q$ the two orders agree to $8.2\times10^{-15}$), so no effective time can exist (numerical demonstration). The Magnus series ($\Omega_1 = \int K$, $\Omega_2 = \tfrac12\iint[K(s_1),K(s_2)]$, ...) is itself an approximation scheme. Equation (P22.48) is therefore a numerical method in two senses: the time integration needs a stiff integrator (backward Euler, or an exponential step with frozen $K$), and the fixed basis has the wrong interface derivative ratio for every other temperature, so convergence in the number of modes is algebraic, as in (P22.46). Script evidence: in its own basis $K(T_2)$ is diagonal to $2\times10^{-13}$ relative; in the $T_1$ basis its largest off-diagonal entry over the first 10 modes reaches 20.4 times the corresponding diagonal entry, $\max_{m\ne n}|K_{mn}|/K_{mm}$ (strong coupling).

#### 5.4.3 (iii) Reduced-order models

A truncated modal model (any of the above with small $n_{\max}$) and proper orthogonal decomposition (POD) are numerical approximations. For POD, collect FV snapshots $X = [\mathbf C(t_1),\dots,\mathbf C(t_s)]$, take the SVD of $V^{1/2}X = U\Sigma W^{\mathsf T}$ (so the inner product is the discrete $\int r^2\,\cdot\,\cdot\,dr$), keep $r$ columns, $\Psi = V^{-1/2}U_r$, and project:

$$
\Psi^{\mathsf T}V\Psi\,\frac{d\tilde{\mathbf a}}{dt} = -\sum_{i=1}^5D_i(T(t))\,\Psi^{\mathsf T}A^{(i)}\Psi\,\tilde{\mathbf a} - \lambda\,\Psi^{\mathsf T}V\Psi\,\tilde{\mathbf a} + \Psi^{\mathsf T}V\mathbf S, \tag{P22.52}
$$

where $A^{(i)}$ is the FV stiffness assembled with $D_i = 1$ in layer $i$ and zero elsewhere (the FV stiffness is not exactly affine in $D_i$ because of the harmonic face conductance at interfaces; the two interface faces need a separate nonlinear term or interpolation, a NOT VERIFIED implementation detail). The discarded singular values measure how well the snapshots are reproduced; they give no error bound for temperature histories outside the training set, so a POD model must be checked against FV for every new history class.

#### 5.4.4 (iv) Staircase approximation of a ramp

A ramp is replaced by $N_T$ constant-temperature intervals of length $h$, then solved exactly per interval by (i) (or by an exact semi-discrete propagator). Three choices of the interval value:

$$
T_k^{\mathrm L} = T(t_k),\qquad T_k^{\mathrm M} = T(t_k + h/2),\qquad \bar D_{i,k} = \frac1h\int_{t_k}^{t_k + h}D_i(T(t))\,dt. \tag{P22.53}
$$

Local error (Magnus argument for $\mathbf y' = B(t)\mathbf y$, $B = -K - \lambda I$, expanded as $B(t_k + s) = B_0 + sB_1 + s^2B_2$): the exact one-step exponent is $\Omega = \Omega_1 + \Omega_2 + \dots$ with

$$
\Omega_1 = \int_0^hB\,ds,\qquad \Omega_2 = \frac12\int_0^h\!\!\int_0^{s_1}[B(s_1),B(s_2)]\,ds_2\,ds_1 = -\frac{h^3}{12}[B_0,B_1] + O(h^4). \tag{P22.54}
$$

The staircase uses $hB(\cdot)$ at its chosen point, so

$$
\Omega_1 - hB_0 = \tfrac12h^2B_1 + \tfrac13h^3B_2,\qquad \Omega_1 - hB(h/2) = \tfrac1{12}h^3B_2. \tag{P22.55}
$$

The left-point staircase has local error $O(h^2)$ and global error $O(h) = O(N_T^{-1})$. The midpoint staircase has local error $O(h^3)$ (both the $\Omega_1$ defect and the neglected $\Omega_2$) and global error $O(N_T^{-2})$. For an operator that is affine in the $D_i$, such as the continuous operator and the Galerkin matrix (P22.49), the per-layer average reproduces $\Omega_1$ exactly; only $\Omega_2$ remains, so it is also $O(N_T^{-2})$. On the FV matrix the two interface faces use the harmonic conductance (P22.14), which is not affine in $D_i$: forming $G_f$ from $\bar D_{i,k}$ differs from the interval average of $G_f$ by $O(h^2)$, a local defect of $O(h^3)$ (checker, `sec5chk_extra.py`, observed order 2.05), so the global order stays 2:

$$
e_{N_T} \approx C_{\mathrm L}N_T^{-1}\ \ (\text{left}),\qquad e_{N_T} \approx C_{\mathrm M}N_T^{-2}\ \ (\text{midpoint, average}),\qquad p_{\mathrm{obs}} = \log_2(e_{N_T}/e_{2N_T}). \tag{P22.56}
$$

Caveat: the Magnus expansion assumes $h\lVert B\rVert$ small. For the stiff FV matrices of the test $h\lVert B\rVert$ is of order $10^2$ to $10^3$, where the argument is heuristic (a fourth-order Magnus step was tried as a reference and failed for this reason). The observed orders below confirm (P22.56) for $F(6\ \mathrm{h})$ in the test problem; at the end of the ramp, $F(4\ \mathrm{h})$, the observed midpoint orders scatter between 1.80 and 2.11 over the last pairs (script output). A proof for the stiff regime is not given here. With shared scaling, all three choices have $\Omega_2 = 0$ and the per-layer average is exact at the interval ends:

$$
D_i = \alpha D_i^*:\quad \bar D_{i,k}\,h = D_i^*\,\Delta\tau_k\ \Longrightarrow\ \text{staircase with}\ \bar D\ \text{is exact at}\ t_k. \tag{P22.57}
$$

Observed (`sec5_staircase.py`, synthetic layer-specific $Q$, ramp 1600 to 1800 $^\circ$C in 4 h, then 2 h hold; fixed FV mesh so only the temperature representation is tested; reference Radau IIA, cross-checked by BDF to $3.4\times10^{-13}$):

| $N_T$ | 2 | 8 | 32 | 128 | 256 | order (last pair) |
|---|---|---|---|---|---|---|
| left, error $F$(6 h) | $6.5\times10^{-2}$ | $1.5\times10^{-2}$ | $3.6\times10^{-3}$ | $8.9\times10^{-4}$ | $4.4\times10^{-4}$ | 1.00 |
| midpoint, error $F$(6 h) | $4.6\times10^{-3}$ | $2.4\times10^{-4}$ | $1.4\times10^{-5}$ | $8.0\times10^{-7}$ | $2.0\times10^{-7}$ | 2.03 |
| per-layer average, error $F$(6 h) | $5.9\times10^{-4}$ | $7.8\times10^{-5}$ | $6.7\times10^{-6}$ | $4.6\times10^{-7}$ | $1.2\times10^{-7}$ | 1.95 |

Table: 5.5 Staircase errors against the number of intervals $N_T$.

The midpoint staircase needs $N_T = 128$ intervals for an error below $10^{-6}$ on this 200 K ramp, that is 128 eigenvalue problems and 128 projections when used with (i).

#### 5.4.5 Comparison

| Method | Exactness | Cost | Accuracy control | Use in STRATO |
|---|---|---|---|---|
| Effective time (5.3) | exact under E1 to E5; series truncation only | one eigen-solve; $\tau$ by (P22.22) | $\Lambda_{\max}^{\ast}\tau \ge 40$ | verification of the variable-$D$ FV; bare-kernel benchmarks with real data |
| Projection, piecewise-constant $T$ (5.4.1) | exact for the infinite series under (P22.39) | one eigen-solve per distinct $T_k$, $n_{\max}^2$ quadratures per switch | (P22.44), (P22.45), mode count | CRP-6 case 3c (1600 then 1800 $^\circ$C step) and other step schedules |
| Projection with staircase (5.4.4) | approximate, $O(N_T^{-1})$ or $O(N_T^{-2})$ | $N_T$ eigen-solves and projections | Richardson in $N_T$ | independent check of FV on ramps; too costly as the production path |
| Galerkin on fixed basis (5.4.2) | numerical (basis and time integration) | $O(n_{\max}^2)$ per step after precomputing $K^{(i)}$ | $n_{\max}$ and $\Delta t$ refinement | not recommended; slow algebraic convergence when ratios change |
| POD / reduced order (5.4.3) | numerical, training-set dependent | offline FV snapshots, cheap online | validation against FV only | optional for sampling studies (uncertainty ensembles) |
| FV with recomputed $G_f$ (5.2) | numerical, first order in time, second order in space | $O(N)$ per step | step doubling, (P22.20), mesh refinement | production solver for arbitrary $T(t)$ |

Table: 5.6 Comparison of the solution methods for a temperature history.

### 5.5 (e) Realistic temperature histories as input definitions

The histories below are input definitions reconstructed from published descriptions. They are not as-run furnace records, and no illustrative profile in this section is data.

#### 5.5.1 AGR-1 safety tests

Morris et al. (2016, p. 25) describe the programme: ramp to 400 $^\circ$C and hold 2 h; ramp to 1250 $^\circ$C and hold 12 h; ramp at 50 $^\circ\mathrm{C\ h^{-1}}$ to the test temperature; hold at 1600, 1700 or 1800 $^\circ$C for "300 or more hours" (p. 34). Morris et al. do not state the first two ramp rates. Collin et al. (2016, p. 379) give the plan used for PARFUME: two ramps of 120 $^\circ\mathrm{C\ h^{-1}}$, the two plateaus, 50 $^\circ\mathrm{C\ h^{-1}}$ to $T_{\max}$, nominal 300 h at $T_{\max}$, cooling to 30 $^\circ$C at $-600\ ^\circ\mathrm{C\ h^{-1}}$, with modelled total durations of 334, 336 and 338 h (p. 380). Each segment of length $|\Delta T|/\text{rate}$ gives the breakpoints

$$
t_{k+1} = t_k + \frac{|T_{k+1} - T_k|}{\dot T_k}\ \ (\text{ramp}),\qquad t_{k+1} = t_k + \Delta t_{\mathrm{hold}}\ \ (\text{hold}). \tag{P22.58}
$$

Reconstructed nominal plan (start at 30 $^\circ$C is an ASSUMPTION; calculated breakpoints in h and $^\circ$C):

| Test | Breakpoints $(t, T)$ | Total (h) | printed (h) |
|---|---|---|---|
| 1600 $^\circ$C | (0, 30), (3.083, 400), (5.083, 400), (12.17, 1250), (24.17, 1250), (31.17, 1600), (331.2, 1600), (333.8, 30) | 333.78 | 334 |
| 1700 $^\circ$C | ... (33.17, 1700), (333.2, 1700), (336.0, 30) | 335.95 | 336 |
| 1800 $^\circ$C | ... (35.17, 1800), (335.2, 1800), (338.1, 30) | 338.12 | 338 |

Table: 5.7 Reconstructed AGR-1 safety-test plans.

The totals reproduce the printed durations, which supports the 30 $^\circ$C start but does not prove it. What is measured and what is not:

- Measured: the thermocouple uncertainty of $\pm1\%$ of reading with dual type C thermocouples (Morris et al., 2016, p. 25) and the reported deviations from the plan: furnace power-supply cycling at the start and intentional thermal cycling after 250 h for compact 3-3-2, two power interruptions during the 1250 $^\circ$C soak for 3-3-1 (Morris et al., 2016, pp. 31 to 32), unplanned cycles at the start for 3-2-2 (Collin et al., 2016, p. 382).
- Not available in the corpus: per-compact as-run furnace histories. Test lengths read from figures (about 385 h for 3-3-2, about 275 to 280 h for 4-3-2) are visual readings (audit note 05).
- Time origin: Morris et al. plot "elapsed time at temperature", so their $t = 0$ is the arrival at the test temperature, 31.2, 33.2 or 35.2 h into the reconstructed plan.

Consequences calculated with the FRG correlations on the 1600 $^\circ$C plan (exact segment integrals by (P22.22)): the 300 h hold contributes 99.3 % of $\int D_{\mathrm{SiC}}\,dt$ for Cs (99.1 % for Cs in UO$_2$), the 50 $^\circ\mathrm{C\ h^{-1}}$ up-ramp 0.50 % (0.69 %), everything up to the end of the 1250 $^\circ$C hold 0.13 % (0.13 %), the cool-down 0.04 % (0.06 %). The thermocouple uncertainty matters more than the ramps: by (P22.19), $\pm16\ \mathrm{K}$ at 1600 $^\circ$C changes $D_{\mathrm{SiC}}$ for Cs by a factor 1.27 ($\Delta\ln D = 0.24$) and $D_{\mathrm{UO_2}}$ by 1.15. (Reading "1 % of reading" in $^\circ$C is an assumption.)

#### 5.5.2 CRP-6 heating schedules

The sensitivity cases (IAEA, 2012, Table 10.5, p. 543) are isothermal (1a, 1b at 1200 and 1600 $^\circ$C, 200 h; 3a, 3b at 1600 and 1800 $^\circ$C) or a step (3c: 200 h at 1600 $^\circ$C then 200 h at 1800 $^\circ$C). Case 3c is exactly of type (P22.39) with one switch, so the projection method of 5.4.1 gives an exact series reference for it; the benchmark itself specifies the step without a heat-up ramp. The postcalculation and prediction cases (IAEA, 2012, Tables 10.11 and 10.12, pp. 554 to 555 and 568) list schedules as triples ($T$ in $^\circ$C, time to reach $T$ in h, time at $T$ in h). Converting a list of triples to breakpoints:

$$
t^{\mathrm{arr}}_k = t^{\mathrm{end}}_{k-1} + \Delta t^{\mathrm{reach}}_k,\qquad t^{\mathrm{end}}_k = t^{\mathrm{arr}}_k + \Delta t^{\mathrm{at}}_k,\qquad T(t^{\mathrm{arr}}_k) = T(t^{\mathrm{end}}_k) = T_k, \tag{P22.59}
$$

with "-" read as zero and the first temperature taken as the start value. The summed durations reproduce all printed totals (333, 333, 277, 229, 541, 187, 699, 836, 824 h; script). Assumptions in the conversion: linear ramps between listed temperatures (the shape is not specified, NOT VERIFIED); the particle is isothermal at the furnace temperature (supported by Hales et al., 2021, p. 9, who prescribe the heating as a temperature boundary condition). The 8a row "(1550, 6.5, -), (300, 1, -)" is printed as such and is internally consistent with the total; it should be re-read from the page image before use (audit note 04).

### 5.6 Verification scripts and output

All scripts are in `scripts/derivation_checks/`, run with `.venv/Scripts/python.exe -I -X utf8 <script>`, write no files and print PASS/FAIL lines.

| Script | Content | Result |
|---|---|---|
| `sec5_arrhenius.py` | (P22.3) to (P22.8), (P22.19), (P22.22), (P22.23); R_g; CRP-6 1a/1b; crossovers; fluence factor; step bounds; validator | 25/25 PASS |
| `sec5_effective_time.py` | SymPy proof of (P22.27) to (P22.38), closure, (P22.50) for a single sphere, (P22.18) | 16/16 PASS |
| `sec5_fv_shared_alpha.py` | independent FV vs independent series at $\tau(t)$; BE options; decay factorisation and (P22.35) | 10/10 PASS |
| `sec5_projection_step.py` | projection (P22.42) to (P22.45) vs extrapolated FV; Galerkin diagonal test (P22.50); coefficient decay rate implied by (P22.46) (the identity itself is tested only by the checker script `sec5chk_numeric.py`) | 9/9 PASS |
| `sec5_staircase.py` | Magnus terms (P22.54), (P22.55); staircase orders (P22.56); BE orders without shared $\alpha$ | 9/9 PASS |
| `sec5_schedules.py` | AGR-1 plan (P22.58), CRP-6 conversion (P22.59), ramp shares, thermocouple sensitivity | 17/17 PASS |

Table: 5.8 Verification scripts of Section 5.

Selected output lines (verbatim):

```text
PASS  <r^2> = 6 D t (Einstein, 3-D)
PASS  scipy.constants.R equals N_A k_B  scipy R = 8.31446261815324
PASS  Q_eff = -R_g dlnD/d(1/T) = sum Q_m D_m / sum D_m (P22.6)
PASS  case 1a F within audit bracket 0.46721-0.46742 for both R_g  0.46734, 0.46740
PASS  dlnD/dt = Q_eff/(R_g T^2) dT/dt (P22.19)
PASS  d/dT [T e^{-a/T} - a E1(a/T)] = e^{-a/T}
PASS  E2 closed form vs quadrature, Q=514 kJ/mol, 1600->30 C  tau = 3.2158049448e+02 s, rel diff 1.6e-15
PASS  ln D stays finite where exp underflows (Q=923 kJ/mol, T=100 K)  ln D = -1097.9
SUMMARY 25/25 checks passed

PASS  real-time PDE residual = 0 for arbitrary chat, tau (P22.29)
PASS  real flux jump = alpha e^{-lam t} x (tau-space flux jump)
PASS  Robin: real condition = alpha e^{-lam t} x (tau condition with h/alpha) (P22.38)
PASS  a_n(t) of (P22.36) solves da/dt = -(lam + alpha Lambda*) a + s_n
PASS  d/dt of (P22.35) equals J_R(t) of (P22.34)
PASS  closure M + M_rel + M_dec = M0: d/dt(sum) = 0 and sum(0) = M0
PASS  int r^2 D* psi_m' psi_n' dr = Lambda_n* N_n delta_mn for m,n = 1..3
PASS  numerical: (Delta tau - alpha^{n+1} dt)/(-alpha' dt^2/2) -> 1  ratios 0.98108, 0.99048, 0.99522
SUMMARY 16/16 checks passed

PASS  eigenvalue count: series roots = FV eigenvalues below Lambda_max  24 vs 24, max rel diff 1.8e-03
PASS  FV(tau) converges to the series at tau(t)  error 5.37e-07 at 160 cells/layer, observed order 2.00
PASS  BE with D(T^{n+1}): observed order in [0.9, 1.1]
PASS  BE with step-averaged D: observed order in [0.9, 1.1]
PASS  BE with step-averaged D: discrete balance M + M_rel = M0 to 1e-10 (roundoff)  1.1e-11
PASS  inventory: M(t) = e^{-lam t} Mhat(tau(t)) (P22.33)  max rel diff 1.6e-12
PASS  release: (P22.35) equals the integrated surface flux  max rel diff 2.8e-12
SUMMARY 10/10 checks passed

PASS  K(T2) in its own basis = diag(Lambda) (P22.49)-(P22.50)  diag rel err 9.3e-15, off-diag 2.0e-13
PASS  t_s=1.0 h: projection (all modes) matches extrapolated FV to 1e-6  max 6.9e-11
PASS  normalised projection coefficients decay like m^-2 (P22.46)  slope -2.03
PASS  t_s=0.05 h: projection (all modes) matches extrapolated FV to 1e-6  max 1.5e-10
SUMMARY 9/9 checks passed

PASS  Magnus Omega_2 = -(h^3/12)[A0, A1] + O(h^4)
PASS  reference cross-check: Radau vs BDF at F(4 h)  BDF 0.631011052637, diff 3.4e-13
PASS  staircase 'left' observed order 1.00 ~ 1 (P22.56)
PASS  staircase 'mid' observed order 1.99 ~ 2 (P22.56)
PASS  staircase 'avg' observed order 1.98 ~ 2 (P22.56)
SUMMARY 9/9 checks passed

PASS  AGR-1 plan 1600 C total 333.78 h rounds to printed 334 h  plateau starts at 31.17 h
PASS  CRP-6 case 8b    summed duration  187.0 h vs printed 187 h
PASS  linearised dlnD within 5 % of exact (Cs in SiC, FRG, Gamma=0, 1600 C)
SUMMARY 17/17 checks passed
```

Verification opportunities for the Phase 2.2 code (not yet implemented in STRATO):

1. Shared-$Q$ five-layer ramp: Rust FV against v1.0 `exact_release` evaluated at $\tau(t)$ from (P22.22); expected spatial order 2, temporal order 1, and with option A an exact match to BE in $\tau$.
2. Bare kernel with the two-branch Cs law on any $T(t)$: FV against the Booth series at $\tau(t)$, extending CRP-6 1a/1b (0.4673, 0.99999959) to non-isothermal histories.
3. CRP-6 case 3c: FV against the projection series with one switch.
4. Balance residual $M + M_{\mathrm{rel}} + M_{\mathrm{dec}} - M_0$ at round-off for every history, with $G_{N+1/2}^{(n)}$ booked at the matrix time level.
5. Breakpoint landing: a schedule with a jump must give identical results whether or not the output grid contains the jump time.

### 5.7 Open issues

- Fluence cutoff conversion (E > 0.1 MeV versus E > 0.18 MeV) for (P22.9) with AGR-1 data: NOT VERIFIED; no source in the corpus.
- AGR-1 as-run furnace histories per compact are not in the corpus; the plan in 5.5.1 is a reconstruction with an assumed 30 $^\circ$C start.
- The ramp shape between CRP-6 schedule points is not specified in TECDOC-1674; linear ramps are an assumption.
- The order-2 staircase result is shown numerically for one stiff test problem; a stiff-regime proof is not given.
- The two-path picture behind (P22.1) assumes parallel transport in local equilibrium; no source in the corpus justifies it for the fitted TECDOC-978 rows beyond Eq. (A-1) itself.
- The POD affine split (P22.52) is inexact at interface faces because of the harmonic conductance; not implemented or tested.
- Temperature-dependent partition coefficients (A2) would change the projection weight; out of scope until a data source exists.
- The SI document for $N_A$ and $k_B$ was not opened in this session; the value rests on SciPy's constants table and the definition (P22.5).

## References

The reference lists of the three drafts were merged into one list. Notes in parentheses give the locations used in this document and the verification status of identifiers. Page numbers are printed page numbers.

Basu, S. K., & McCutchan, E. A. (2020). Nuclear data sheets for A = 90. *Nuclear Data Sheets, 165*, 1–329. https://doi.org/10.1016/j.nds.2020.04.001

Browne, E., & Tuli, J. K. (2007). Nuclear data sheets for A = 137. *Nuclear Data Sheets, 108*, 2173–2318. https://doi.org/10.1016/j.nds.2007.09.002 (Cs-137 half-life 30.08 y, via audit 07.)

Collin, B., Jiang, W., Gamble, K., Gardner, R., Hales, J., Haugh, B., Novascone, S., Roche, J., Spencer, B., & Toptan, A. (2021). TRISO fuel performance modelling with BISON. *Journal of Physics: Conference Series, 2048*(1), 012012. https://doi.org/10.1088/1742-6596/2048/1/012012

Collin, B. P., Petti, D. A., Demkowicz, P. A., & Maki, J. T. (2016). Comparison of fission product release predictions using PARFUME with results from the AGR-1 safety tests. *Nuclear Engineering and Design, 301*, 378–390. https://doi.org/10.1016/j.nucengdes.2016.03.023

Gelbard, F. (2003). *Analytical modeling of fission product releases by diffusion from multicoated fuel particles* (Report SAND2002-3966). Sandia National Laboratories. (Eq. II.1, p. 10; Eq. II.6, p. 11; Eqs. II.16 to II.18 within pp. 10–14, Eq. II.18 on p. 14. These contain the single-sphere decay factor that Eq. (P21.80) generalises. DOI and URL NOT VERIFIED.)

Gürdal, G., & Kondev, F. G. (2012). Nuclear data sheets for A = 110. *Nuclear Data Sheets, 113*, 1315–1561. https://doi.org/10.1016/j.nds.2012.05.002

Hales, J. D., Jiang, W., Toptan, A., & Gamble, K. A. (2021). Modeling fission product diffusion in TRISO fuel particles with BISON. *Journal of Nuclear Materials, 548*, 152840. https://doi.org/10.1016/j.jnucmat.2021.152840

idaholab. (2026). *moose* [Computer software]. GitHub. https://github.com/idaholab/moose (`force_step_every_function_point`, via audit 08.)

International Atomic Energy Agency. (1997). *Fuel performance and fission product behaviour in gas cooled reactors* (IAEA-TECDOC-978; K. Verfondern, Ed.). IAEA. (Eq. A-1, p. 435; partition ratios, p. 436; Tables A-1 to A-3, including Table A-1 on p. 439, FRG Cs in UO2 with two branches. No DOI printed; URL NOT VERIFIED.)

International Atomic Energy Agency. (2012). *Advances in high temperature gas cooled reactor fuel technology* (IAEA-TECDOC-CD-1674) [CD-ROM]. IAEA. (Eqs. 7.1 to 7.8, pp. 317–319; Eq. 7.25, p. 402; Table 8.1, p. 435; Eqs. 10.1 to 10.7 and the FRESCO-II normalisation, pp. 529–544; Tables 10.2, 10.4, 10.5 and 10.6. ISBN 978-92-0-186810-7. URL from audit 04, not opened: https://www-pub.iaea.org/MTCD/Publications/PDF/TE_1674_CD_web.pdf)

International Atomic Energy Agency, Nuclear Data Section. (n.d.). *LiveChart of Nuclides, data download API v1* [Data set]. Retrieved November 5, 2026, from https://www-nds.iaea.org/relnsd/vcharthtml/api_v0_guide.html (Source of the ENSDF half-lives and of the JEFF-3.1.1 fission yields quoted here.)

Kellett, M., Bersillon, O., & Mills, R. (2009). *The JEFF-3.1/-3.1.1 radioactive decay data and fission yields sub-libraries* (JEFF Report 20, NEA No. 6287). OECD Nuclear Energy Agency. https://www.oecd-nea.org/jcms/pl_14322 (Report text not opened; cited as the yield library.)

Morris, R. N., Baldwin, C. A., Demkowicz, P. A., Hunn, J. D., & Reber, E. L. (2016). Performance of AGR-1 high-temperature reactor fuel during post-irradiation heating tests. *Nuclear Engineering and Design, 306*, 24–35. https://doi.org/10.1016/j.nucengdes.2016.04.031

Nica, N. (2025). Nuclear data sheets for A = 154. *Nuclear Data Sheets, 200*, 2–524. https://doi.org/10.1016/j.nds.2025.02.002

Sonzogni, A. A. (2004). Nuclear data sheets for A = 134. *Nuclear Data Sheets, 103*, 1–182. https://doi.org/10.1016/j.nds.2004.11.001

STRATO project. (2026a). *Finite-volume derivation of TRISO diffusion and release* (docs/mathematics/phase1/TRISO_Finite_Volume_Derivation.md, v1.0). (Tags cited: TRISO-ML-432 to TRISO-ML-446; TRISO-FV-100, TRISO-FV-123, TRISO-FV-140 and TRISO-FV-300; TRISO-DEL-010 to TRISO-DEL-040.)

STRATO project. (2026b). *STRATO v1.0.0* [Computer software]. https://github.com/weewzr/STRATO/tree/v1.0.0 (Files `src/numerical/finite_volume.rs`, `src/analytical/sturm_liouville.rs` and `src/config.rs`, quoted by line number.)

## Appendix A Verification scripts and result lines

All scripts are in `scripts/derivation_checks/` of the repository. Each was re-run for this assembly (B8, 2026-11-06) from the repository root with

```bash
.venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/<script>.py
```

The scripts write no files. Every script exited with code 0. The counts below are the lines that begin with PASS or FAIL in the B8 output; they agree with the B3 and B7 re-runs (251 PASS in total for the 17 section scripts). The helper module `sec3_p21_common.py` prints nothing and contains no checks. The scripts check Python transcriptions and symbolic derivations; they do not run any Rust.

| Script | Section | What it checks | PASS | FAIL |
|---|---|---|---|---|
| `sec3_p21_decay_source.py` | 3 | decay law, sources, precursors, activation products, chains, (P21.5) to (P21.23) | 24 | 0 |
| `sec3_p21_modal_ode.py` | 3 | modal solution, release, closure and limits, (P21.24) to (P21.49), SymPy and mpmath | 32 | 0 |
| `sec3_p21_booth.py` | 3 | multilayer modal formulas against the Booth single-sphere solutions, TECDOC-1674 Eqs. (7.6) to (7.8) | 20 | 0 |
| `sec3_p21_fv_two_layer.py` | 3 | modal solution with source and decay against an independent two-layer FV solver | 14 | 0 |
| `sec3_p21_chain_overlap.py` | 3 | parent-daughter overlap matrix, (P21.17) to (P21.22) | 5 | 0 |
| `sec4_p21_matrix_schemes.py` | 4 | symbolic algebra of the BE matrix, pivots, link to v1.0, split scheme, $M_\mathrm{out}$ | 13 | 0 |
| `sec4_p21_be_closure_order.py` | 4 | Python transcription of the v1.0 mesh and solver with decay and source: closure, positivity, orders | 15 | 0 |
| `sec4_p21_decay_identity.py` | 4 | decay identity (P21.80) and the inventory partition (P21.81) | 12 | 0 |
| `sec4_p21_steady_state.py` | 4 | exact steady states (P21.82) to (P21.85) | 18 | 0 |
| `sec4_p21_mms_source.py` | 4 | manufactured solution in a two-layer sphere, (P21.88) to (P21.90) | 8 | 0 |
| `sec4_p21_booth_crp6.py` | 4 | FV against the Booth solutions and CRP-6 cases 1a and 1b | 4 | 0 |
| `sec5_arrhenius.py` | 5 | Arrhenius forms, gas constant, crossovers, fluence factor, step bounds, (P22.3) to (P22.9), (P22.19) to (P22.23) | 25 | 0 |
| `sec5_effective_time.py` | 5 | effective-time transform and decay factorisation, (P22.24) to (P22.38), (P22.18), (P22.50) | 16 | 0 |
| `sec5_fv_shared_alpha.py` | 5 | FV with a shared scaling factor on a ramp against the series at $\tau(t)$ | 10 | 0 |
| `sec5_projection_step.py` | 5 | eigenfunction projection at a temperature step, (P22.39) to (P22.46) | 9 | 0 |
| `sec5_staircase.py` | 5 | staircase approximation of a ramp, (P22.53) to (P22.57) | 9 | 0 |
| `sec5_schedules.py` | 5 | AGR-1 and CRP-6 temperature schedules, (P22.58), (P22.59) | 17 | 0 |
| total | | | 251 | 0 |

Table: A.1 Derivation check scripts of Sections 3 to 5, B8 re-run.

Four further scripts in the same folder check the pseudocode and the parameter register rather than this document. They were re-run as well.

| Script | What it checks | PASS | FAIL |
|---|---|---|---|
| `b4_decay_source_pseudocode.py` | pseudocode blocks of Phase 2.1 (decay, source, bookkeeping) | 55 | 0 |
| `b4_phase2_pseudocode_integration.py` | joined Phase 2 pseudocode: planner, step, bookkeeping, v1.0 bit identity | 32 | 0 |
| `sec6_pseudocode_checks.py` | Phase 2.2 pseudocode blocks | 30 | 0 |
| `D5_parameter_register.py` | parameter register against its CSV files and sources | 91 | 0 |

Table: A.2 Related scripts, B8 re-run.

### A.1 Result lines

The PASS, FAIL and SUMMARY lines of each script are reproduced verbatim from the B8 run, wrapped at 100 characters (a continuation line starts with four spaces). Section 3.9, Section 4.9 and Section 5.6 explain what each check means.

`sec3_p21_decay_source.py` (Section 3):

```text
PASS  P21.6 dN/dt=-lambda N gives N0 exp(-lambda t)
PASS  P21.7 half-life T = ln2/lambda  T = log(2)/lambda
PASS  P21.7 mean life = 1/lambda
PASS  P21.7 lambda(Cs-137) = ln2/T1/2 reproduces audit 07  lambda = 7.302187e-10 s^-1, rel diff
    2.9e-08
PASS  P21.7 lambda(Sr-90) = ln2/T1/2 reproduces audit 07  lambda = 7.597709e-10 s^-1, rel diff
    5.2e-08
PASS  P21.7 lambda(Ag-110m) = ln2/T1/2 reproduces audit 07  lambda = 3.211198e-08 s^-1, rel diff
    1.1e-07
PASS  P21.7 lambda(Cs-134) = ln2/T1/2 reproduces audit 07  lambda = 1.063576e-08 s^-1, rel diff
    2.9e-07
PASS  P21.7 lambda(Eu-154) = ln2/T1/2 reproduces audit 07  lambda = 2.556445e-09 s^-1, rel diff
    1.1e-07
PASS  P21.8 S = Fdot y / N_A has units mol m^-3 s^-1
PASS  P21.12 cumulative birth = yC F t - yp F (1-exp(-lp t))/lp
PASS  P21.12 asymptotic lag = (yp/yC)/lambda_p <= 1/lambda_p  lag = y_p/(lambda_p*(y_I + y_p))
PASS  P21.12 t99 = log2(100) T1/2 = 6.644 T1/2  factor 6.6439
PASS  P21.12 t99 for Xe-137 (3.818 min) matches audit 07  t99 = 0.423 h
PASS  P21.12 t99 for Rb-90m (258 s) matches audit 07  t99 = 0.476 h
PASS  P21.12 t99 for Xe-133 (5.2475 d) matches audit 07  t99 = 836.727 h
PASS  P21.15 parent mode solves a_P' = -kP a_P + sP, a_P(0)=0
PASS  P21.16 activation mode solves a_A' = -kA a_A + f sigma phi a_P, a_A(0)=0
PASS  P21.16 activation mode steady state = f sigma phi sP/(kP kA)
PASS  P21.16 activation mode grows as f sigma phi sP t^2/2 at small t
PASS  P21.21 Bateman-diffusion modal solution satisfies coupled ODE
PASS  P21.21 initial condition a_D(0) recovered
PASS  P21.22 O = I reduces to two-member Bateman per mode
PASS  P21.23 post-irradiation amplitude continuous at t_irr
PASS  P21.23 post-irradiation amplitude solves source-free ODE
SUMMARY: 24/24 PASS
```

`sec3_p21_modal_ode.py` (Section 3):

```text
PASS  P21.25 psi(r^2 D phi')' - phi(r^2 D psi')' = [r^2 D (psi phi' - phi psi')]'
PASS  P21.24 L phi_n = -Lambda_n phi_n (homogeneous sphere, symbolic n)
PASS  P21.26 orthogonality int r^2 phi_m phi_n = 0 (m != n samples)
PASS  P21.26 N_n = R/2 for the homogeneous sphere  N_n = R/2
PASS  P21.35 I_n = (-1)^(n+1) R^2/(n pi)
PASS  P21.36 R^2 D phi_n'(R) = -Lambda_n I_n
PASS  P21.31 integrating-factor solution satisfies a' = -k a + s(t)
PASS  P21.32 constant-source closed form
PASS  P21.32 steady amplitude s_n/k_n
PASS  P21.33 piecewise-constant recursion equals integrating factor
PASS  P21.34 source switched off: a(t) = a(t_irr) exp(-k (t - t_irr))
PASS  P21.38 closed form of int_0^t a_n ds (constant source)
PASS  P21.39 per-mode identity: direct release = M0 + Mgen - M - Mdec
PASS  P21.42 per-mode closure M0 + Mgen = M + Mdec + Mrel
PASS  P21.40 dM/dt = G - lambda M - J_R (per mode)
PASS  P21.45 source-free ultimate release 4 pi I_n a_n(0) Lambda_n/k_n (per mode)
PASS  P21.45 decay-corrected flux J_R e^(lambda t) equals the lambda = 0 flux (per mode)
PASS  P21.43 M_out = int J exp(-lambda (t-s)) ds solves M_out' = J - lambda M_out
PASS  P21.44 M + M_out = M0 e^(-lambda t) + int G e^(-lambda (t-s)) ds solves (M+M_out)' = G -
    lambda (M+M_out)
PASS  P21.46 4 pi a_n(0) I_n / M_0 = 3 I1_n I_n /(R1^3 N_n)
PASS  P21.47 4 pi s_n I_n / G = w_n (source weights equal v1.0 weights)
PASS  P21.47 per-mode R/B(t) = w_n Lambda_n/k_n (1 - e^(-k_n t))
PASS  P21.47 accelerated split Lambda_n/k_n = 1 - lambda/k_n
PASS  P21.49 Laplace transform: int Lambda e^(-Lambda t) e^(-lambda t) dt = Lambda/(Lambda+lambda)
PASS  P21.48 modal R/B(t) term = TECDOC-1674 (7.7) term
PASS  P21.48 F_B = M_out/(M + M_out) term = TECDOC-1674 (7.8) term (uses sum w_n = 1)
PASS  P21.49 (7.6) closed form = sum 6/(n^2 pi^2 + x), x = 0.01  R/B = 0.99933396762
PASS  P21.49 (7.6) closed form = sum 6/(n^2 pi^2 + x), x = 1.0  R/B = 0.939105856498
PASS  P21.49 (7.6) closed form = sum 6/(n^2 pi^2 + x), x = 100.0  R/B = 0.270000001237
PASS  P21.49 (7.6) closed form = sum 6/(n^2 pi^2 + x), x = 2107.0  R/B = 0.0639327047985
PASS  P21.49 small-lambda limit: R/B_ss = 1 - x/15 + 2x^2/315 + ...
PASS  P21.49 short-lived limit: R/B_ss * sqrt(x)/3 -> 1 (TECDOC-1674 7.25)
SUMMARY: 32/32 PASS
```

`sec3_p21_booth.py` (Section 3):

```text
PASS  P21.24 eigenvalues = n^2 pi^2 D/R^2 (n <= 1000)  max rel err 1.3e-15
PASS  P21.24 Sturm check: mode n has n-1 interior zeros (n = 1,2,10,100,1000)
PASS  P21.47 weights w_n = 6/(n^2 pi^2)  max rel err 5.9e-11
PASS  P21.47 sum w_n = 1 - exact tail  1 - sum = 6.076232e-04, tail = 6.076232e-04
PASS  P21.36 R^2 D_5 phi_n'(R) = -Lambda_n I_n (numerical eigenfunctions)  max rel err 2.9e-11
PASS  P21.48 modal R/B(t) vs TECDOC-1674 (7.7), lambda/D' = 0.01  max abs diff 1.8e-13
PASS  P21.48 modal F_B(t) vs TECDOC-1674 (7.8), lambda/D' = 0.01  max abs diff 2.1e-08, within the N
    = 1000 truncation bound (ratio 1.00)
PASS  P21.48 modal R/B(t) vs TECDOC-1674 (7.7), lambda/D' = 1.0  max abs diff 2.1e-11
PASS  P21.48 modal F_B(t) vs TECDOC-1674 (7.8), lambda/D' = 1.0  max abs diff 2.1e-08, within the N
    = 1000 truncation bound (ratio 1.00)
PASS  P21.48 modal R/B(t) vs TECDOC-1674 (7.7), lambda/D' = 100.0  max abs diff 2.1e-09
PASS  P21.48 modal F_B(t) vs TECDOC-1674 (7.8), lambda/D' = 100.0  max abs diff 2.2e-08, within the
    N = 1000 truncation bound (ratio 1.00)
PASS  P21.49 steady R/B = 1 - lambda sum w_n/k_n vs (7.6), lambda/D' = 0.01  0.9993339676 vs
    0.9993339676 (truncation bound 2.1e-13)
PASS  P21.49 steady R/B = 1 - lambda sum w_n/k_n vs (7.6), lambda/D' = 1.0  0.9391058565 vs
    0.9391058565 (truncation bound 2.1e-11)
PASS  P21.49 steady R/B = 1 - lambda sum w_n/k_n vs (7.6), lambda/D' = 100.0  0.2700000033 vs
    0.2700000012 (truncation bound 2.1e-09)
PASS  P21.44 M + M_out = P (1 - e^(-lambda t))/lambda (quadrature of J_R)  rel diff 2.2e-16
PASS  P21.48 F_B equals M_out/(M + M_out)  0.773329025 vs 0.773329025
PASS  P21.45 M_rel > M_out when lambda > 0 (released atoms decay outside)  M_rel/P = 6.983546e+06 s,
    M_out/P = 6.138254e+06 s
PASS  P21.39 direct flux series error ~ N^-1  observed order 0.99
PASS  P21.39 balance form error ~ N^-3  observed order 2.94 (N <= 100, before round-off)
PASS  P21.39 balance form 1e4 times more accurate at N = 100  R/B 6.0e-03 vs 2.0e-08; M_rel 6.0e-04
    vs 2.2e-08
SUMMARY: 20/20 PASS
```

`sec3_p21_fv_two_layer.py` (Section 3):

```text
PASS  P21.24 Sturm check on two-layer eigenfunctions (n = 1..400 samples)  Lambda_1 = 3.3767e-08
    s^-1
PASS  P21.46 sum of v1.0 weights -> 1  1 - sum w = -8.49e-06 (N = 400)
PASS  P21.36 R^2 D_L phi_n'(R) = -Lambda_n I_n (two layers)  max rel err 9.2e-12
PASS  P21.42 discrete FV inventory closure, 20 cells/layer  2.4e-14
PASS  P21.42 discrete FV inventory closure, 40 cells/layer  1.5e-14
PASS  P21.42 discrete FV inventory closure, 80 cells/layer  5.6e-14
PASS  P21.42 discrete FV inventory closure, 160 cells/layer  5.7e-14
PASS  P21.28 FV -> modal for M, observed order  order 2.00, finest rel diff 1.5e-05
PASS  P21.28 FV -> modal for Mrel, observed order  order 2.00, finest rel diff 3.8e-05
PASS  P21.28 FV -> modal for Mdec, observed order  order 2.00, finest rel diff 3.1e-06
PASS  P21.28 FV -> modal for JR, observed order  order 2.00, finest rel diff 1.3e-05
PASS  P21.49 steady R/B: direct sum differs from balance form by the weight tail  difference
    -8.490e-06, 1 - sum w = -8.490e-06
PASS  P21.29 projection of c0 reproduces M_0 (truncated sum)  rel diff 3.1e-09
PASS  P21.29 general initial profile with decay: FV -> modal  rel diff 9.4e-05, 2.3e-05, 5.8e-06;
    order 2.00
SUMMARY: 14/14 PASS
```

`sec3_p21_chain_overlap.py` (Section 3):

```text
PASS  P21.22 O = I when daughter D equals parent D (same element)  max |O - I| = 5.4e-14
PASS  P21.22 |O| = I when D_D = 3 D_P in every layer (Lambda scales by 3)  max ||O| - I| = 5.1e-14
PASS  P21.19 O is a full matrix when the D ratios differ  max off-diagonal |O_nm| = 4.214, O_11 =
    2.3122, O_21 = -1.6006
PASS  P21.21 coupled modal daughter inventory = two-species FV (converging)  finest rel diff
    2.2e-05, order 2.00
PASS  P21.20 dropping the off-diagonal overlap terms gives a visible error  3.7e-01 vs 2.2e-05
SUMMARY: 5/5 PASS
```

`sec4_p21_matrix_schemes.py` (Section 4):

```text
PASS  X1 elimination pivots d_j = e_j + g^E_j with the subtraction-free e_j (P21.61)  induction step
    and explicit N = 5
PASS  X1b e_j - v_j = g^W_j e_{j-1}/(e_{j-1}+g^W_j) > 0 (no subtraction anywhere)
PASS  X2 row excess = v_j (j < N), v_N + G_R (j = N) (P21.60)  v_j = V_j (1/dt + lam)
PASS  X3a V/dt + lam V + K = V/dt' + K, dt' = dt/(1+z)
PASS  X3b V/dt = (1+z)^-1 V/dt', hence A_lam^-1 V/dt = (1+z)^-1 (V/dt'+K)^-1 V/dt' (P21.63)
PASS  X4 IF-scheme fixed point (z e^z/(e^z-1) K + lam V) C = V S (P21.68)
PASS  X4b factor z e^z/(e^z-1) = 1 + z/2 + z^2/12 + O(z^4)
PASS  X5 ln[(1+z)^-n e^{nz}] = n z^2/2 - n z^3/3 + O(z^4); with n z = lam t: ~ lam^2 t dt/2 (P21.77)
PASS  X6a fixed points of both M_out updates are J/lam (P21.76)
PASS  X6b exponential update is the exact solution of dM_out/dt = J - lam M_out, J step-constant
    (P21.74)
PASS  X6c BE update local error = h^2 lam (lam M_o - J)/2 + O(h^3)  -h**2*lambda*(J - M_o*lambda)/2
PASS  X7 int_step S - dt S(t^{n+1}) = -dt^2 S'/2 - (2/3) dt^3 S''/2 (Taylor, P21.79)
PASS  X8 straddling cell: Sbar = S_K (R_1^3 - a^3)/(b^3 - a^3) (P21.54)
ALL PASS
```

`sec4_p21_be_closure_order.py` (Section 4):

```text
PASS  N1a per-step identity M^{n+1}-M^n = dt(Gen - lam M^{n+1} - G_R C_N^{n+1}) (P21.70)  max rel
    1.7e-15 over 71 steps
PASS  N1b cumulative closure M0 + M_gen = M + M_dec + M_rel (P21.72)  rel residual 1.2e-16; M_gen
    6.8722e-09, M_dec 2.1191e-10, M_rel 6.4675e-09, M 2.5828e-10 mol
PASS  N2 min C >= 0.0 exactly in floating point for dt = 1e-3 .. 1e12 s  min 1.095e-128
PASS  N3a semi-discrete identity exp(-(A + lam I)T) C0 = e^{-lam T} exp(-A T) C0 (P21.67)  rel
    2.6e-13
PASS  N3b unsplit BE: error in M(T) vs semi-discrete reference, order -> 1  errors ['7.88e-02',
    '3.97e-02', '1.99e-02', '9.98e-03', '4.99e-03', '2.50e-03'], orders ['0.990', '0.995', '0.997',
    '0.999', '0.999']
PASS  N3c decay-exact split: order -> 1, smaller constant  errors ['1.96e-02', '9.89e-03',
    '4.97e-03', '2.49e-03', '1.25e-03', '6.23e-04']
PASS  N4 BE(lam, dt)^n = (1 + lam dt)^-n BE_v1(dt/(1 + lam dt))^n (P21.63)  rel 7.6e-16
PASS  N5 lam = 0, S = 0: identical to v1.0 bit for bit over 200 steps
PASS  N6 decay only: C^n = (1+z)^-n C0 exactly; order 1 to e^{-lam t}; error ~ lam^2 t dt/2  errors
    ['4.80e-02', '2.45e-02', '1.24e-02', '6.22e-03', '3.12e-03'], predicted last 3.13e-03
PASS  N7a BE M_out: M_rel = M_out + M_dec,out (P21.75)  rel 1.3e-14
PASS  N7b whole system: M_gen = M + M_out + M_dec,in + M_dec,out  rel 8.5e-15
PASS  N7c both M_out updates approach J_R/lam at steady state (P21.76)  BE 1.2e-07, exponential
    1.2e-07, (M_out_BE - M_out_exp)/M_out = -3.8e-09
PASS  N8a BE fixed point = semi-discrete steady state, lam dt = 0.1, 1, 5, two lam (R/B exact)  max
    rel R/B diff 5.6e-14; steady R/B = 0.968429 (lam 2e-8), 0.025830 (lam 7e-6)
PASS  N8b integrating-factor fixed point is (z e^z/(e^z-1) K + lam V) C = V S (P21.68): biased M_ss
    and R/B  lam=2e-08,z=0.1: dM/M -4.71e-02, dRB/RB +1.54e-03; lam=2e-08,z=1.0: dM/M -3.61e-01,
    dRB/RB +1.18e-02; lam=2e-08,z=5.0: dM/M -7.97e-01, dRB/RB +2.60e-02; lam=7e-06,z=0.1: dM/M
    -2.70e-03, dRB/RB +1.02e-01; lam=7e-06,z=1.0: dM/M -3.38e-02, dRB/RB +1.27e+00; lam=7e-06,z=5.0:
    dM/M -2.32e-01, dRB/RB +8.76e+00
PASS  N9 exact step average reproduces the generated amount; rectangle at n+1 misses part of a step
    rectangle error -0.167, step average error +1.6e-16
ALL PASS
```

`sec4_p21_decay_identity.py` (Section 4):

```text
PASS  D1 PDE: res_decay[e^{-lam t} c0] = e^{-lam t} res_free[c0] (P21.80)
PASS  D1b conditions scale by e^{-lam t} (homogeneous, linear)
PASS  D2 with source: c = e^{-lam t} ct, ct driven by e^{lam t} S
PASS  D3a M_rel per mode = M0 w Lam/(Lam+lam) (1 - e^{-(Lam+lam) t}) (P21.81)
PASS  D3b M_dec per mode = M0 w lam/(Lam+lam) (1 - e^{-(Lam+lam) t})
PASS  D3c M + M_rel + M_dec = M0 w per mode (closure)
PASS  D3d lam -> 0: M_rel -> M0 w (1 - e^{-Lam t}) (Phase 1)
PASS  D4a mode-wise R/B(t) = TECDOC-1674 Eq. (7.7) term (P21.87)
PASS  D4b mode-wise M_out/(M + M_out) = TECDOC-1674 Eq. (7.8) term
PASS  D4c M_n + M_out,n = w B (1 - e^{-lam t})/lam (inventory identity behind the denominator)
PASS  D4d summed: Eq. (7.8) = M_out/(M + M_out) from modal sums  max |diff| 9.86e-32; F(1e4 s) =
    0.5311856865
PASS  D5 lam -> 0 of Eq. (7.8) equals Eq. (7.1) at D't = 0.01, 0.1, 1  max |diff| 2.7e-16
ALL PASS
```

`sec4_p21_steady_state.py` (Section 4):

```text
PASS  A1 ODE residual in all five layers (P21.82)  [0, 0, 0, 0, 0]
PASS  A2 centre regularity dc/dr(0) = 0
PASS  A3 continuity of c at R_1..R_4  [0, 0, 0, 0]
PASS  A4 continuity of D dc/dr at R_1..R_4  [0, 0, 0, 0]
PASS  A5 absorbing surface c(R) = 0
PASS  A6 steady release J_R = Gen = (4/3) pi R_1^3 S (P21.83)
PASS  A7 steady inventory closed form (P21.84)
PASS  A8 tau_res = M_ss/Gen = R_1^2/(15 D_1) + sum (R_i^2-R_{i-1}^2)/(6 D_i)
PASS  A9 one material, R_1 = R: tau_res = R^2/(15 D)
PASS  B1 Booth steady ODE D r^-2 (r^2 c')' - lam c + S = 0 (TECDOC-1674 Eq. 7.5)
PASS  B2 c(R) = 0
PASS  B3 regular at r = 0 (finite limit)
PASS  B4 R/B equals TECDOC-1674 Eq. (7.6) (numeric, 40 digits)  |diff| = 2.53e-175
PASS  B5 sum 6/(n^2 pi^2 + a) = (7.6) closed form  a = 1433.33, R/B = 0.0771475583135
PASS  C1 interface and surface conditions, multilayer with decay (P21.85)  max residual 2.59e-54
PASS  C2 ODE residual inside IPyC (mp.diff)  -2.97e-67
PASS  C3 steady balance Gen = lam M_ss + J_R  rel 3.21e-55, R/B = 0.9684291531
PASS  C4 lambda -> 0 limit equals part A, error O(lambda)  rel errors ['1.62e-6', '1.62e-8',
    '1.62e-10']
ALL PASS
```

`sec4_p21_mms_source.py` (Section 4):

```text
PASS  M1 centre, continuity, flux continuity, surface (P21.88)  [0, 0, 0, 0]
PASS  M2a diffusion part identical in both layers  -L c / (c_ref g) = pi*D_ref*(4*R*sin(pi*r/(2*R))
    + pi*r*cos(pi*r/(2*R)))/(4*R**2*r)
PASS  M2b S continuous at R1
PASS  M3 S >= 0 everywhere for the chosen data  min S = 2.565e-07 mol m^-3 s^-1
PASS  M4a antiderivative derivative check
PASS  M4b cell integral of S: antiderivative = quadrature (P21.90)  rel 1.9e-15
PASS  M5 steady MMS, spatial order (volume-weighted L2) -> 2  errors ['2.28e-04', '5.64e-05',
    '1.41e-05', '3.51e-06', '8.77e-07'], orders ['2.014', '2.004', '2.001', '2.000']
PASS  M6 transient MMS, temporal order -> 1 (BE)  errors ['6.22e-02', '3.12e-02', '1.56e-02',
    '7.81e-03', '3.91e-03'], orders ['0.995', '0.999', '1.000', '1.000']
ALL PASS
```

`sec4_p21_booth_crp6.py` (Section 4):

```text
PASS  B1 steady R/B: FV (= BE fixed point) -> Eq. (7.6), spatial order 2  (7.6) R/B = 0.71980889;
    rel errors ['6.36e-04', '1.59e-04', '3.99e-05', '9.97e-06']; orders ['1.997', '1.999', '2.000']
PASS  B2 transient R/B(t) (7.7) and F(t) (7.8) at t = 5e4 s: BE temporal order ~ 1, errors < 1e-4 at
    the finest step dt = 50 s  (7.7) 0.719726, (7.8) 0.710587; |err R/B| ['2.7e-05', '1.4e-05',
    '7.7e-06', '4.7e-06']; |err F| ['4.4e-04', '2.2e-04', '1.1e-04', '5.7e-05']; difference orders
    ['1.10', '1.05'] / ['1.00', '1.00']
PASS  B3a two-branch FRG Cs D reproduces Table 10.6 analytical 1a/1b, single branch does not
PASS  B3b FV (N = 150, dt = 36 s) matches Booth (10.7): |dF| < 2e-4 (1a), 1-F within 5 % (1b);
    closure < 1e-12
ALL PASS
```

`sec5_arrhenius.py` (Section 5):

```text
PASS  Gaussian kernel has unit mass
PASS  <r^2> = 6 D t (Einstein, 3-D)
PASS  D = Gamma a^2 / 6 (P22.3)
PASS  R_g = 8.314462618... J/(mol K) (truncated print agrees to 2e-11)  R_g = 8.31446261815324
PASS  scipy.constants.R equals N_A k_B  scipy R = 8.31446261815324
PASS  R_g = N_A k_B to machine precision  N_A k_B = 8.31446261815324
PASS  D(R2)/D(R1) = exp[(Q/T)(1/R1 - 1/R2)] (P22.8)
PASS  first order: 1 + (Q/(R1 T)) eps
PASS  R_g choice changes D by less than 0.2 % on the 1000-2000 C grid  worst = 9.021e-04
PASS  case 1a F within audit bracket 0.46721-0.46742 for both R_g  0.46734, 0.46740
PASS  Q_eff = -R_g dlnD/d(1/T) = sum Q_m D_m / sum D_m (P22.6)
PASS  branches equal at T_x = (Q2-Q1)/(R_g ln(D02/D01)) (P22.7)
PASS  numerical: Cs-in-SiC branches equal at computed T_x
PASS  fluence factor at Gamma=4.5 equals 2.460 (audit 03)
PASS  dlnD/dt = Q_eff/(R_g T^2) dT/dt (P22.19)
PASS  d/dT [T e^{-a/T} - a E1(a/T)] = e^{-a/T}
PASS  E2(x) = e^{-x} - x E1(x)
PASS  E2 closed form vs quadrature, Q=514 kJ/mol, 1250->1600 C  tau = 3.8576546576e+03 s, rel diff
    3.3e-16
PASS  E2 closed form vs quadrature, Q=300 kJ/mol, 1600->1800 C  tau = 4.3202717562e+04 s, rel diff
    3.3e-16
PASS  E2 closed form vs quadrature, Q=209 kJ/mol, 400->1250 C  tau = 1.1369225879e+02 s, rel diff
    2.6e-15
PASS  E2 closed form vs quadrature, Q=514 kJ/mol, 1600->30 C  tau = 3.2158049448e+02 s, rel diff
    1.2e-15
PASS  validator rejects all 7 invalid inputs
PASS  validator accepts Q = 0 (TECDOC-978 row 44)
PASS  ln D stays finite where exp underflows (Q=923 kJ/mol, T=100 K)  ln D = -1097.9
PASS  Kelvin conversion T_K = T_C + 273.15 exactly at 1600 C
SUMMARY 25/25 checks passed
```

`sec5_effective_time.py` (Section 5):

```text
PASS  real-time PDE residual = 0 for arbitrary chat, tau (P22.29)
PASS  explicit case chat = e^-X r^2 + X^2 sin r, tau = t + t^3/3
PASS  single-sphere mode e^{-lam t} sin(n pi r/R)/r exp(-n^2 pi^2 D* tau/R^2) solves PDE
PASS  real flux jump = alpha e^{-lam t} x (tau-space flux jump)
PASS  continuity: c_a - c_b = e^{-lam t}(chat_a - chat_b)
PASS  initial condition: c(r,0) = chat(r,0) because tau(0) = 0 and e^0 = 1
PASS  Robin: real condition = alpha e^{-lam t} x (tau condition with h/alpha) (P22.38)
PASS  unfactored form: alpha (L* - lam/alpha) + S = alpha L* - lam + S (P22.32)
PASS  a_n(t) of (P22.36) solves da/dt = -(lam + alpha Lambda*) a + s_n
PASS  a_n(0) = a0
PASS  constant alpha and source: a_n = s0 (1 - e^{-(lam+alpha Lambda) t})/(lam + alpha Lambda)
    (P22.37)
PASS  d/dt of (P22.35) equals J_R(t) of (P22.34)
PASS  closure M + M_rel + M_dec = M0: d/dt(sum) = 0 and sum(0) = M0
PASS  int r^2 D* psi_m' psi_n' dr = Lambda_n* N_n delta_mn for m,n = 1..3
PASS  Delta tau - alpha(t_{n+1}) dt = -alpha' dt^2/2 + alpha'' dt^3/6
PASS  numerical: (Delta tau - alpha^{n+1} dt)/(-alpha' dt^2/2) -> 1  ratios 0.98108, 0.99048,
    0.99522
SUMMARY 16/16 checks passed
```

`sec5_fv_shared_alpha.py` (Section 5):

```text
PASS  eigenvalue count: series roots = FV eigenvalues below Lambda_max  24 vs 24, max rel diff
    1.8e-03
PASS  tau(7 h) closed form = quadrature of alpha
PASS  FV(tau) converges to the series at tau(t)  error 5.37e-07 at 160 cells/layer, observed order
    2.00
PASS  BE with D(T^{n+1}): observed order in [0.9, 1.1]
PASS  BE with D(T^{n+1}): discrete balance M + M_rel = M0 to 1e-10 (roundoff)  9.2e-12
PASS  BE with step-averaged D: observed order in [0.9, 1.1]
PASS  BE with step-averaged D: discrete balance M + M_rel = M0 to 1e-10 (roundoff)  1.1e-11
PASS  inventory: M(t) = e^{-lam t} Mhat(tau(t)) (P22.33)  max rel diff 1.6e-12
PASS  release: (P22.35) equals the integrated surface flux  max rel diff 2.8e-12
PASS  inventory vs series: |M/M0 - e^{-lam t}(1 - F_series(tau))| < 1e-4 at 40 cells/layer  4.3e-06
SUMMARY 10/10 checks passed
```

`sec5_projection_step.py` (Section 5):

```text
PASS  orthogonality of new eigenfunctions (weight r^2 for phi = weight 1 for u)  max normalised
    off-diagonal 3.5e-14
PASS  K(T2) in its own basis = diag(Lambda) (P22.49)-(P22.50)  diag rel err 9.3e-15, off-diag
    2.0e-13
PASS  K(T2) in the T1 basis is not diagonal: modes couple when layer ratios change
PASS  t_s=1.0 h: projection (all modes) matches extrapolated FV to 1e-6  max 6.9e-11
PASS  t_s=1.0 h: outputs at t - t_s >= 0.25 h unaffected by new-basis truncation  max 6.9e-11
PASS  normalised projection coefficients decay like m^-2 (P22.46)  slope -2.03
PASS  switch-time inventory error envelope decreases with M_new  4.6e-04 -> 2.8e-07
PASS  t_s=0.05 h: projection (all modes) matches extrapolated FV to 1e-6  max 1.5e-10
PASS  t_s=0.05 h: outputs at t - t_s >= 0.25 h unaffected by new-basis truncation  max 1.5e-10
SUMMARY 9/9 checks passed
```

`sec5_staircase.py` (Section 5):

```text
PASS  Magnus Omega_2 = -(h^3/12)[A0, A1] + O(h^4)
PASS  midpoint: int_0^h A ds - h A(h/2) = h^3 A2/12 (O(h^3))
PASS  left end: int_0^h A ds - h A(0) = h^2 A1/2 + h^3 A2/3 (O(h^2))
PASS  reference cross-check: Radau vs BDF at F(4 h)  BDF 0.631011052637, diff 3.4e-13
PASS  staircase 'left' observed order 1.00 ~ 1 (P22.56)
PASS  staircase 'mid' observed order 1.99 ~ 2 (P22.56)
PASS  staircase 'avg' observed order 1.98 ~ 2 (P22.56)
PASS  BE with D(T^{n+1}), layer-specific Q: first order
PASS  BE with step-averaged D, layer-specific Q: first order
SUMMARY 9/9 checks passed
```

`sec5_schedules.py` (Section 5):

```text
PASS  AGR-1 plan 1600 C total 333.78 h rounds to printed 334 h  plateau starts at 31.17 h
PASS  AGR-1 plan 1700 C total 335.95 h rounds to printed 336 h  plateau starts at 33.17 h
PASS  AGR-1 plan 1800 C total 338.12 h rounds to printed 338 h  plateau starts at 35.17 h
PASS  CRP-6 case 6a/6b summed duration  333.0 h vs printed 333 h
PASS  CRP-6 case 7a    summed duration  277.0 h vs printed 277 h
PASS  CRP-6 case 7b    summed duration  229.0 h vs printed 229 h
PASS  CRP-6 case 8a    summed duration  541.0 h vs printed 541 h
PASS  CRP-6 case 8b    summed duration  187.0 h vs printed 187 h
PASS  CRP-6 case 9     summed duration  699.0 h vs printed 699 h
PASS  CRP-6 case 10    summed duration  836.0 h vs printed 836 h
PASS  CRP-6 case 11    summed duration  824.0 h vs printed 824 h
PASS  segment shares sum to 1 (Cs in SiC, FRG, Gamma=0)
PASS  segment shares sum to 1 (Cs in UO2, FRG)
PASS  linearised dlnD within 5 % of exact (Cs in SiC, FRG, Gamma=0, 1600 C)
PASS  linearised dlnD within 5 % of exact (Cs in SiC, FRG, Gamma=0, 1800 C)
PASS  linearised dlnD within 5 % of exact (Cs in UO2, FRG, 1600 C)
PASS  linearised dlnD within 5 % of exact (Cs in UO2, FRG, 1800 C)
SUMMARY 17/17 checks passed
```

## Appendix B Independent verification record

This appendix keeps the review record of the three drafts unchanged in content. In it, `sec3_phase21_decay_source_analytical.md`, `sec4_phase21_decay_source_fv.md` and `sec5_phase22_temperature.md` are the draft files of Sections 3, 4 and 5 of this document, and `B3_reconciliation.md` is the reconciliation record reproduced in Appendix B.4. Paths under the work folder (`scratchpad`, `derive_checks`, `b3_checks`, `b7_checks`) refer to independent checker scripts that are not part of the repository. Row numbers such as "Section 3 row 14" refer to the numbered rows of the check-log tables below. Equation and section numbers are those of this document.

### B.1 Check log of Section 3 (Part 2.1, analytical)

Adversarial check of this section, 2026-11-06. Every key result was re-derived without reusing the author's algebra or helper code. Independent scripts (scratchpad, not in the repo):

- `<work folder>/derive_checks/ind_p21_symbolic.py`: closed forms re-derived from the ODEs with `sympy.dsolve` or from the defining integrals, the summed (not per-term) TECDOC-1674 (7.8) identity in mpmath, units with `sympy.physics.units`. Output `SUMMARY: 34/34 PASS`.
- `<work folder>/derive_checks/ind_p21_layered.py`: eigenfunctions in $\phi$ form with $\phi$ and $D\phi'$ matched directly (no jump rule), all modal integrals and overlaps in closed form (no quadrature), finite volumes with centroid nodes (as in `finite_volume.rs`) integrated exactly in time by an augmented matrix exponential, so only spatial error remains. Output `SUMMARY: 25/25 PASS`.

Selected independent output:

```text
PASS  P21.48 F_B = M_out/(M+M_out) equals TECDOC (7.8) (summed, not per term)  max diff 3.4e-15
PASS  3.7.3 F_B = int J e^{lam s} / int G e^{lam s} (decay-corrected release over decay-corrected production)  max diff 4.4e-16
PASS  3.8 (7.8) series tail decays algebraically, order 3 (not exponentially)  order 2.945
PASS  P21.24 first 20 eigenvalues = FV generalized eigenvalues, converging at O(h^2) (no root missed)  max rel 3.2e-03 -> 8.1e-04
PASS  P21.36 R^2 D_L phi_n'(R) = -Lambda_n I_n, two layers  max rel 7.1e-15
PASS  P21.36 five layers flux identity  max rel 3.5e-14
PASS  Sec 3.9 quoted modal values at t_irr reproduced  max rel 5.9e-08
PASS  Sec 3.9 quoted R/B(t_irr) and steady R/B  R/B(t_irr) = 0.671485, R/B_ss = 0.68638073
PASS  P21.42 and P21.44 exact closure in the semi-discrete FV (exact time integration)  2.2e-11
PASS  P21.28-38 FV (exact in time) -> modal for JR  order 2.00, finest 2.1e-05
PASS  P21.49 steady R/B (layered) = FV steady solve  FV 0.68638097 vs modal 0.68638073, order 2.00
PASS  P21.39 layered: direct M_rel converges at least like N^-1, balance at least like N^-3  observed orders 2.10 (direct), 4.05 (balance)
PASS  P21.19 quoted O_11, O_21, max off-diagonal (phi(0) = 1 scaling)  O11 2.3122, O21 -1.6006, max off 4.214
PASS  P21.21 coupled modal daughter = two-species FV (exact time)  finest 2.4e-05, order 2.00
PASS  P21.20 diagonal-only overlap error about 37 %  0.369
```

The author's five scripts were re-run unchanged and print `24/24`, `32/32`, `20/20`, `14/14` and `5/5 PASS`; each check was read and tests what its label claims. Two limits of those scripts: `sec3_p21_modal_ode.py` checks (P21.36) only for the homogeneous sphere (the layered case is in `sec3_p21_fv_two_layer.py` and in the independent script), and the modal $M_\mathrm{rel}$ in `sec3_p21_fv_two_layer.py` is the balance form, so the FV comparison tests (P21.39) and not the direct series (P21.38). No script needed a code change.

| # | Item | Verdict |
|---|---|---|
| 1 | (P21.1) to (P21.4): signs, units, interface and boundary conditions; Phase 1 tags TRISO-ML-439, TRISO-ML-446 and $u_5(R) = 0$ exist in the Phase 1 document | CONFIRMED |
| 2 | Citations: ATLAS (10.1) sign typo, FRESCO-II (10.3b), (10.4b) p. 531, NRCDIF (10.6), FRESCO-II normalisation p. 532, Table 8.1 p. 435, Eqs (7.5) to (7.8) p. 319, (7.25) p. 402, all against audit 04 | CONFIRMED |
| 3 | Gelbard (2003) Eq. II.1 p. 10, transform and Eq. II.6 p. 11, Eq. II.18 p. 14 "released or decayed", against audit 02 | CONFIRMED |
| 4 | Half-lives, $\lambda$ values, Ag-110m branching, Cs-134m IT 100 % with 2.912 h, ENSDF evaluation references and DOIs, LiveChart and JEFF references, against audit 07 | CONFIRMED (independent recomputation, max rel 2.8e-7) |
| 5 | (P21.5) to (P21.7) | CONFIRMED |
| 6 | (P21.8), (P21.9) units | CONFIRMED |
| 7 | (P21.11), (P21.12) formula, $t_{99}$ values 0.423 h, 0.476 h, 34.9 d, yield ratios 1.2 %, 0.55 %, factors 86 and 183, Ag-109 factor 58 | CONFIRMED |
| 8 | Direction of the precursor lag | CORRECTED ("the cumulative-yield model is late by at most $1/\lambda_p$" -> "the true birth lags the cumulative-yield model, which runs ahead, by $(y_p/y^C)/\lambda_p \le 1/\lambda_p$"; $B(t) < y^C\dot Ft$) |
| 9 | Meaning of $t_{99}$ | CORRECTED ("99 % of the precursor contribution has arrived" -> feed rate at 99 % of equilibrium, equivalently 99 % of the precursor atoms born at one instant have decayed; the old wording read as a cumulative fraction) |
| 10 | Section 3.3.3 yields and shielding statements | CONFIRMED |
| 11 | (P21.15), (P21.16) activation solution, steady value, $t^2$ growth (re-derived by dsolve) | CONFIRMED |
| 12 | (P21.19) to (P21.22) chain solution (dsolve), $O = I$ for a common layer ratio, quoted $O_{11}$, $O_{21}$, 4.214, 37 % diagonal-only error | CONFIRMED |
| 13 | Scale dependence of $O_{nm}a^P_m$ | CORRECTED ("the products $O_{nm}a^P_m$ do not [depend on scaling]" -> they carry $1/d_n$ of the daughter scaling; only $O_{nm}a^P_m\phi^D_n$ and $c_D$ are scale-free; $O = \delta$ in (P21.22) needs equal scaling), checked numerically |
| 14 | Recommendation list includes I-131 and Kr-85 with an instantaneous $y^C$ source | CORRECTED (caveat added: Te-131m $t_{99} = 9.2$ d, Kr-85m $t_{99} = 29.8$ h, audit 07 Section 5.4) |
| 15 | Audit cross-reference for CRP-6 cases 4a to 5b | CORRECTED ("audit 04, Section 3" -> "audit 04, Sections 4.2 and 6"; Section 3 of audit 04 is about diffusion coefficients) |
| 16 | (P21.24) to (P21.26) eigenproblem, Green identity, orthogonality (closed-form Gram matrices for two and five layers) | CONFIRMED |
| 17 | (P21.27) to (P21.30) projection and source weights; same structure as TRISO-DEL-020 | CONFIRMED |
| 18 | (P21.31) to (P21.34) time solutions; recursion exact over three intervals | CONFIRMED |
| 19 | (P21.35) $I_n$ of the homogeneous sphere | CONFIRMED |
| 20 | (P21.36), (P21.37) flux identity, factor $4\pi$ and $r^2$ (two and five layers, rel 1e-14) | CONFIRMED |
| 21 | (P21.38), (P21.39) balance identity per mode | CONFIRMED |
| 22 | Steady $R/B$ in (P21.39) and the transient flux split | CORRECTED (missing assumptions stated: kernel-uniform source, $a_n(0) = 0$; otherwise $w_n \to 4\pi s_nI_n/G$) |
| 23 | Truncation orders $O(N^{-1})$ and $O(N^{-3})$ | CORRECTED (labelled worst case for sign-definite $w_n$; the two-layer kernel-source case gives $N^{-2.1}$ and $N^{-4.0}$ because $w_n$ changes sign 284 times in 400 modes) |
| 24 | Convergence table: $M_\mathrm{rel}$ reference | CORRECTED ("against the TECDOC-1674 closed forms" -> $M_\mathrm{rel}$ is compared with the time integral of (7.7), derived in the script and not printed in TECDOC-1674); table values re-run and CONFIRMED |
| 25 | (P21.40) to (P21.44) inventory closure and surviving inventory; exact discrete closure in an FV with exact time integration (2.2e-11) | CONFIRMED |
| 26 | (P21.45) $F_\mathrm{corr}$ equals the Phase 1 $F$ (layered, modal) and $F_\mathrm{rel}(\infty) = (R/B)_\mathrm{ss}$ for kernel-uniform $c_0$ | CONFIRMED (qualifier "kernel-uniform $c_0$" added in Section 3.7.4) |
| 27 | (P21.46), (P21.47) reduction to TRISO-DEL-021; same formula as `sturm_liouville.rs` line 352 (`3.0 * i1 * moment / (r1.powi(3) * norm)`) | CONFIRMED |
| 28 | (P21.48) against TECDOC-1674 (7.7) and (7.8), summed numerically | CONFIRMED |
| 29 | "(7.8) is neither $M_\mathrm{rel}/M_\mathrm{gen}$ nor a decay-corrected release" | CORRECTED (it equals $\int J_Re^{\lambda s}ds/\int Ge^{\lambda s}ds$, a decay-corrected release over decay-corrected production; wording replaced) |
| 30 | Production symbol $P$ in Section 3.7.3 clashes with the parent label $P$ | CORRECTED ($P$ -> $G$) |
| 31 | (P21.49) closed form, $x \to 0$ and $x \to \infty$ limits, Laplace-transform reading; layered steady $R/B$ against a direct FV steady solve (order 2.00) | CONFIRMED |
| 32 | Status table: "(7.7), (7.8) with exponential convergence for $t > 0$" | CORRECTED (only (7.7) converges exponentially; the (7.8) terms tend to $1/(\pi^2n^4)$, observed tail order 2.95) |
| 33 | Status of Section 3.4.3 | CORRECTED ("truncation is the only approximation" -> also numerical roots and quadrature of the modal integrals) |
| 34 | Shared notation: modal rate $k_n$ clashes with the species index $k$ and with Phase 1 $k_i$ | CORRECTED ($k_n$, $k^P_m$, $k^D_n$ -> $\kappa_n$, $\kappa^P_m$, $\kappa^D_n$, 56 occurrences; quoted script output left verbatim) |
| 35 | Symbols used but not in the table ($B$, $\lambda_p$, $y_p$, $N_p$, $t_{99}$, $\beta$, $n_\mathrm{HM}$, $t_\mathrm{irr}$, $f_q$, $O_{nm}$, $\gamma$, release fractions, $\psi$, $\delta_\pm$, $\theta$) | CORRECTED (rows with SI units added) |
| 36 | FV equation of Section 3.9: $\Delta_fC$, $\delta_\pm$, $\delta$ undefined; node placement differs from Rust | CORRECTED (definitions added; note that the Python check uses midpoints and `finite_volume.rs` uses volume centroids) |
| 37 | Quoted script outputs in Section 3.9 (two-layer values, overlap values, convergence tables) | CONFIRMED by re-running and by independent reproduction |
| 38 | Completeness of the layered eigenfunction set | OPEN (only numerical support: $\sum w_n \to 1$, FV agreement, no missed root against FV generalized eigenvalues) |
| 39 | Cross sections, isomeric fraction, flux histories, kernel heavy-metal density, daughter diffusivities | OPEN (NOT VERIFIED, as the section states) |
| 40 | Decay constants used by CRP-6 participants | OPEN (NOT VERIFIED, audit 04 Section 2.4) |

Table: B.1 Independent check of Section 3.

#### B3 reconciliation note

A second independent re-derivation (B3, 2026-11-06; scripts `b3_sec3_symbolic.py` 36/36 PASS, `b3_sec3_layered.py` 10/10 PASS, `b3_completeness.py` 2/2 PASS in `<work folder>/b3_checks/`) confirmed every key result of this section, including the quoted two-layer values. One change: open item 38 (completeness of the layered eigenfunctions) is now closed by the Green-function argument in Section 3.10 (derived, Hilbert-Schmidt theorem; citation not checked). Items 39 and 40 stay OPEN. Details: `<work folder>/derive/B3_reconciliation.md`.

#### B7 review note

B7 final review, 2026-11-06. All five `sec3_*` scripts re-run: 20/20, 5/5, 24/24, 14/14 and 32/32 PASS. Independent checks in `<work folder>/b7_checks/` (`b7_c1_booth_fv.py`: own FV solved exactly in time converges at order 2.00 to (P21.48) and (P21.49) at $x = 0.01$, 1 and 100, and $F_B \ne M_\mathrm{rel}/M_\mathrm{gen}$; `b7_c5_symbolic.py`: (P21.12), $t_{99}$, (P21.16), (P21.21)). Text changes: the initial-condition sentence of Section 3.1 cited a Section 3.3.6 that does not exist and now cites Section 3.3.5; the symbol row of $J_R$ no longer has a literal bar inside table math. No equation changed.

### B.2 Check log of Section 4 (Part 2.1, finite volume)

Checker scripts (written from scratch; they do not import the author's scripts), run with `.venv/Scripts/python.exe -I -X utf8`:

- `<work folder>/derive_checks/sec4chk_symbolic.py`: 51 PASS lines (S1 to S28), `ALL PASS`.
- `<work folder>/derive_checks/sec4chk_numeric.py`: 22 PASS lines (N1 to N15), `ALL PASS`. It rebuilds the v1.0 mesh from `build_mesh` (volume centroids, series conductances, $G_R$ with $R-r_N$) and uses dense LU as an independent solver.

Selected checker output:
```text
PASS S8c log relative error per mode: unsplit (Lam+lam)^2 t h/2, split Lam^2 t h/2, so decay adds (2 Lam lam + lam^2) t h/2 (P21.69)
PASS S12 rectangle error, derivatives at t^n: -h^2 S'/2 - h^3 S''/3 + O(h^4)
PASS S20 M_ss closed form by direct volume integral (P21.84)
PASS S24 summed: Eq. 7.8 = M_out/(M+M_out) (two parameter sets)  max diff 4.24e-21; M_rel/M_gen = 0.90741667 vs F78 = 0.91912798 (differ, lam > 0)
PASS N1c R/B = 0.9684291531 (section value)  R/B = 0.968429153101
PASS N2 FV steady R/B: 0.968429 needs the (20,8,4,4,4) mesh; 8 cells per layer gives 0.968413  R/B_FV = 0.9684287 (20,8,4,4,4), 0.9684126 (8x5)
PASS N4 IF fixed point solves (P21.66); bias of R/B (balance def.) ... lam=2e-08 z=0.1 dRB(balance)=+1.536e-03 dRB(G_R C_N)=-4.691e-02; ... lam=7e-06 z=1 dRB(balance)=+1.283e+00 dRB(G_R C_N)=+4.434e-01; lam/Lambda_1 = 9.77
PASS N6b cumulative closure (P21.72) and outside closure (P21.75)  6.9e-14 2.8e-15 7.2e-14
PASS N7 min C >= 0.0 exactly, lam = 7e-6, dt = 1e-3..1e12 s
PASS N10 decay shift of M_rel/M0 ... 1a -7.508e-05, 1b -2.433e-05, lam tau_res 2.433e-05, ratio 59.8
PASS N11 truncation ... modes 13: 1-M-M_dec err 1.69e-07 (bound 1.69e-07), naive M_rel err -4.501e-02
PASS N13a FV 1a (N=150): F = 0.46737 (360 s), 0.46740 (36 s); |F - 0.46742| < 2e-4  F = 0.46737 / 0.46740
PASS N13 FV 1b (N=150, dense LU): 1-F = 4.28e-7 (360 s), 4.09e-7 (36 s)  1-F = 4.2796e-07 / 4.0903e-07, Booth 4.0703e-07, rel err 5.14 % / 0.49 %
PASS N15 single mode, lam = Lambda, 800 steps: unsplit 2.5e-3, split 6.25e-4 (ratio 4, not 2)  unsplit 2.499e-03 split 6.247e-04 ratio 4.00
```

Author scripts re-run: all six end with `ALL PASS` (18, 12, 13, 8, 15 and 4 PASS lines); every printed PASS line matches the quotation in Section 4.9 apart from trailing spaces. Each was read for what it actually tests (see items below).

| Item | Verdict |
|---|---|
| (P21.50), (P21.51): PDE, centre, interface, surface conditions; FRESCO-II Eqs. 10.3b, 10.4a,b,d and NRCDIF Eq. 10.6, printed pp. 531, 535-536; Eq. 10.1 sign typo | CONFIRMED (audit 04, Sec. 1) |
| (P21.52) units and $y^C$ choice; I-137 24.5 s, Xe-137 3.818 min | CONFIRMED (S28; audit 07) |
| (P21.53), (P21.54) cell average, straddling case; faces on every $R_i$ in `build_mesh` | CONFIRMED (S1; `finite_volume.rs` lines 54-73) |
| (P21.55), (P21.56) semi-discrete equation, decay term exact for uniform $\lambda$ | CONFIRMED |
| (P21.57) rectangle error coefficient $-\tfrac13\Delta t^3\ddot{\bar S}$ | CORRECTED (unqualified -> "derivatives at $t^n$", reason: about $t^{n+1}$ the coefficient is $+\tfrac16$; S12, S12b; author X7 does expand about $t^n$) |
| (P21.58), (P21.59) and the v1.0 change table: `backward_euler_step` lines 171-180, `v[i] = volume / dt`, `rhs[i] = v[i] * old[i]` at line 179 | CONFIRMED against the Rust source |
| Bitwise regression rule ($V/\Delta t+\lambda V$ form) | CONFIRMED (N8: adding $0.0$ is exact; $V\cdot(1/\Delta t)$ differs from $V/\Delta t$ in 502 of 2000 random cases) |
| (P21.60) row excess; (P21.61) subtraction-free pivots, induction and $N=5$ | CONFIRMED (S2, S3, S3b, S4; recurrence matches `solve_diffusion_system` lines 140-158) |
| (P21.62) positivity in exact and floating-point arithmetic; v1.0 test threshold $-10^{-13}$ (line 310) | CONFIRMED (N7) |
| (P21.63) link to the v1.0 step with $\Delta t'=\Delta t/(1+z)$ | CONFIRMED (S5 symbolic 3x3, N5 numeric) |
| (P21.64), (P21.77) BE decay factor error and step rule | CONFIRMED (S7, S8, S8b) |
| (P21.65) to (P21.68) integrating-factor scheme, fixed point, $\varphi(z)$ series | CONFIRMED (S6, S6b, S6c, N4, N5b) |
| N8 bias percentages of the split scheme | CORRECTED (added: values hold for $R/B=1-\lambda M/\mathrm{Gen}$; with $G_RC_N/\mathrm{Gen}$ they are $-4.7\,\%$, $-36\,\%$, $+4.9\,\%$, $+44\,\%$; reason: the two readings differ by $\varphi(z)$, N4) |
| (P21.69) "decay part $\approx\tfrac12\lambda^2t\Delta t$" | CORRECTED (-> per mode $\tfrac12(\Lambda_n+\lambda)^2t\Delta t$ unsplit, $\tfrac12\Lambda_n^2t\Delta t$ split; reason: the cross term $\Lambda_n\lambda$ is missing in the old statement, and N3's ratio 4 requires it; S8c, N15) |
| Crank-Nicolson factor sign | CONFIRMED (S14) |
| (P21.70) to (P21.72) per-step and cumulative closure, Kahan already in v1.0 `run` (lines 211-217) | CONFIRMED (N6, N6b) |
| (P21.73) to (P21.76) outside inventory, local error, fixed points, $\lambda\to0$ | CONFIRMED (S9, S10, S11, S11b, N6b) |
| "differ by $3.8\times10^{-9}$ relative at steady state" | CORRECTED (-> "after 16 mean lives, a transient remainder"; reason: both updates have the same exact fixed point) |
| Ag-110m $\lambda$, $\lambda t=1.72$, $\Delta t\le3.6\times10^4\ \mathrm{s}$ | CONFIRMED (N14; Table 8.1, p. 435 via audit 04) |
| I-131 $\lambda=1.0002\times10^{-6}\ \mathrm{s^{-1}}$ without source | CORRECTED (source added: Table 8.1 8.021 d; ENSDF 8.0252 d gives $9.9967\times10^{-7}$; limit 400 s unchanged; N14) |
| Cs-137 $\lambda=7.302\times10^{-10}\ \mathrm{s^{-1}}$, Browne & Tuli (2007) with DOI and pages | CONFIRMED (audit 07, lines 36, 233, 322) |
| (P21.78) and CRP-6 1b step prediction: $\Lambda_1=1.97\times10^{-5}$, $5.1\,\%$ and $0.5\,\%$; FRESCO-II 360 s (Table 10.4, p. 542) | CONFIRMED (N12, N13: independent FV gives $5.14\,\%$ and $0.49\,\%$) |
| Explicit bound with decay | CONFIRMED (S13) |
| (P21.79) jump bound used undefined $V_K$ and the word "loses" | CORRECTED ($V_K=\tfrac{4\pi}{3}R_1^3$ defined, in the symbol table and the text; "loses" -> "misplaces (adds or loses)") |
| Symbol $r_N$ in $G_R$ undefined in the table | CORRECTED (row for $r_j$ added: volume centroid, as `volume_centroid` and line 91 of `finite_volume.rs`) |
| (P21.80) decay identity, also with time-dependent $D$ and with source | CONFIRMED (S15, S15b) |
| Gelbard (2003) listed in References but never cited in the text | CORRECTED (in-text citation added in 4.7.1: Eqs. II.6, II.16 to II.18, pp. 11-14, per audit 02) |
| (P21.81) partition per mode, closure, $F_{corr}$, $\sum w_n/\Lambda_n=R^2/(15D)$ | CONFIRMED (S16 to S17) |
| (P21.81) truncation "with the Phase 1 rule" | CORRECTED (-> evaluate $M_{rel}=M_0-M-M_{dec}$; reason: a term-by-term $M_{rel}$ sum truncated by the Phase 1 rule is off by the weight tail, $-0.045$ in case 1a; the $M_{dec}$ tail bound is $\lambda t\sum w_n/(\Lambda_nt)$; N11) |
| (P21.82) to (P21.84) multilayer steady state, $J_R=\mathrm{Gen}$, $M_{ss}$, $\tau_{res}$, one-material limit; Booth Eq. 7.3 p. 317 | CONFIRMED (S18 to S20c) |
| (P21.85) particular solution, jump rule, 9 unknowns; $R/B=0.9684291531$ | CONFIRMED (S21, S21b, N1 to N1c at 50 digits) |
| FV value 0.968429 "on a 40-cell mesh" | CORRECTED (mesh stated: (20, 8, 4, 4, 4) cells; 8 per layer gives 0.968413; N2) |
| (P21.86) Booth steady solution and Eq. 7.6, series form | CONFIRMED (S22 to S22d) |
| (P21.87) Eq. 7.7 term by term; $F_{(7.8)}=M_{out}/(M+M_{out})$ and $\ne M_{rel}/M_{gen}$ | CONFIRMED (S23, S23b, S24: summed difference $4\times10^{-21}$; $M_{rel}/M_{gen}=0.9074$ vs $F_{(7.8)}=0.9191$) |
| (P21.87) first series evaluated as written | CORRECTED (note added: slow tail $6/(\pi^2n_{max})$; evaluate in the Eq. 7.7 form) |
| Whether TECDOC-1674 defines Eq. 7.8 in words as $M_{out}/(M+M_{out})$ | OPEN (NOT VERIFIED; the algebra is exact) |
| (P21.88) to (P21.90) manufactured solution, source, $Q(0)$, midpoint step average | CONFIRMED (S25 to S27b); author M5/M6 use exact cell and step averages (read in the script) |
| (P21.91) two-branch FRG Cs law (TECDOC-978 Table A-1, p. 439), 1a $D't=0.02594$, $F$ bracket 0.46721 to 0.46742, 1b $F=0.99999959$, single branch 0.46163 and 0.99995461 | CONFIRMED (N9, N9b; audit 03, audit 04 Sec. 4.3) |
| CRP-6 FV values 1a and 1b, decay shifts $-7.5\times10^{-5}$ and $-2.43\times10^{-5}=-\lambda\tau_{res}$, ratio 60 | CONFIRMED (N10, N13, N13a) |
| Author script B2 label "errors < 1e-4" | CORRECTED (label -> "errors < 1e-4 at the finest step dt = 50 s"; reason: the first three step sizes give $\lvert\Delta F\rvert$ of $4.4\times10^{-4}$ to $1.1\times10^{-4}$; the test only checks the finest; script edited and re-run, `ALL PASS`) |
| Author D5 ($\lambda\to0$ of Eq. 7.8) | CONFIRMED as a numerical limit (evaluated at $\lambda/D'=10^{-14}$, not a symbolic limit) |
| Author N5 bitwise test | CONFIRMED in scope: it compares two Python transcriptions, so it supports the IEEE argument but does not execute the Rust code (open issue 6 already says so) |
| MOOSE `force_step_every_function_point` | CORRECTED (in-text citation idaholab, 2026 added; audit 08 lines 23, 182) |
| IAEA (2012) page numbers for Eqs. 7.1 to 7.8, 10.3b to 10.7, Tables 8.1, 10.2, 10.4 to 10.6 | CONFIRMED (audit 04) |
| Which $R_g$ the CRP-6 authors used; whether participants included decay | OPEN (NOT VERIFIED, as stated in open issue 2) |
| Rust implementation of Section 4.3 | OPEN (specified only; no Rust code was edited) |

Table: B.2 Independent check of Section 4.

Counts: 30 CONFIRMED, 13 CORRECTED, 3 OPEN.

#### B3 reconciliation note

A second independent re-derivation (B3, 2026-11-06; `b3_sec4.py` 32/32 PASS, `b3_sec4_split.py`) confirmed every key result of this section. Changes: (P21.86) and (P21.87) now use $x = \lambda/D'$ as in Section 3 (the draft used $x = \sqrt{\lambda/D'}$ here and $x = \lambda/D'$ in Section 3); the open item on the wording of Eq. (7.8) is closed (the report names it only the cumulative fractional release, printed p. 319); Section 4.7.6 adds that $R_g = 8.3143$ gives $F = 0.467337$, the only tested constant that rounds to the printed 0.4673. Which $R_g$ the CRP-6 participants used and whether they included decay stays OPEN. Details: `<work folder>/derive/B3_reconciliation.md`.

#### B7 review note

B7 final review, 2026-11-06. All six `sec4_*` scripts re-run, each `ALL PASS`. Independent checks in `<work folder>/b7_checks/` (`b7_c2_multilayer_fv.py`: Richardson-extrapolated own FV gives $R/B = 0.9684291531$ and 0.968428661 on the (20, 8, 4, 4, 4) mesh; $\tau_\mathrm{res}$ of (P21.84) by quadrature of (P21.82); iterating the split scheme (P21.66) reaches the fixed point (P21.68) to $10^{-13}$ with the biases $+0.0015$, $+0.1018$, $+1.2740$ (balance) and $-0.0469$, $-0.3604$, $+0.0485$, $+0.4374$ (flux); `b7_c3_crp6_arrhenius.py`: CRP-6 1a and 1b values, decay shifts $-7.51\times10^{-5}$ and $-2.43\times10^{-5}$, the 0.045 truncation deficit of the term-by-term series). Text changes: Section 4.1 and Section 4.10 called parent-daughter chains a Phase 2.2 topic; Phase 2.2 is the temperature part and chains are FUTURE (Section 3.3.4), so both sentences were corrected; matrix row 7 and open issue 4 now name the Phase 2.1 modal solution of Section 3 as the reference; open issue 1 now records the B3 reading of printed p. 319; the v1.0 column header of the Section 4.3 table names `backward_euler_step` at lines 162 to 182 and its assembly loop at lines 171 to 180; the Ag-110m step example notes that the ENSDF half-life gives the same limit. No equation changed.

### B.3 Check log of Section 5 (Part 2.2)

Checker scripts (independent of the author's scripts; run with `.venv/Scripts/python.exe -I -X utf8 <script>`, no file output) are in `<work folder>/derive_checks/`: `sec5chk_symbolic.py` (25/25 PASS), `sec5chk_numeric.py` (29/29 PASS), `sec5chk_extra.py` (8/8 PASS). The six author scripts were re-run: 25/25, 16/16, 10/10, 9/9, 9/9, 17/17 PASS (outputs `sec5chk_author_*.out` in the same folder). Selected checker output:

```text
PASS  (P22.29)-(P22.30) residual of real-time PDE is 0 with Shat = e^{lam t} S/alpha
PASS  (P22.54) Omega_2 = -(h^3/12)[A0,A1] + O(h^4)
PASS  (P22.46) N_m a_m = [sum R_i^2 phi(R_i) q_i (rho_i - rho_i+1) - layerwise int phi (r^2 D f')' dr]/Lambda_m, 12 modes  max rel diff 2.1e-11
PASS  (P22.35) M_rel by inventory quadrature equals integrated surface flux (release is non-trivial)  3.1e-09, M_rel(20 h)/M0 = 0.6481
PASS  D0 span of Arrhenius rows in Tables A-1 to A-3 is 8.8e-15 to 1.8e6 (not 5.5e-14)  8.8e-15 to 1.8e+06, 20.3 decades
PASS  per-layer averaged D: interface G defect is O(h^2) per unit time (local O(h^3))  orders 2.20, 2.10, 2.05
PASS  option A differs from e^{-lam t} x (BE in tau) when lam > 0 (statement needs lam = 0)  8.77e-04
```

| # | Item | Verdict |
|---|---|---|
| 1 | (P22.1) form, units, TECDOC-978 Eq. (A-1) p. 435 | CONFIRMED |
| 2 | (P22.2) to (P22.5) jump picture; Einstein $\langle r^2\rangle = 6Dt$ (SymPy) | CORRECTED: (P22.3) wrote $6Dt = \Gamma_jta_j^2$ and then inserted $f$ without derivation; now $6Dt = f\Gamma_jta_j^2$ with $f$ introduced from the correlated cross terms; jump count renamed $n \to n_j$ (clash with mode index) |
| 3 | Lattice $D_0$ estimate $6\times10^{-7}$ to $1.8\times10^{-6}$ | CONFIRMED (numeric) |
| 4 | $D_0$ span in TECDOC-978 tables | CORRECTED: lower end $5.5\times10^{-14}$ (Cs in SiC) $\to$ $8.8\times10^{-15}$ (I, Kr, Xe in UO$_2$, FRG AC branch 1, Table A-1); 20.3 decades |
| 5 | (P22.6) $Q_{\mathrm{eff}}$, (P22.7) $T_x$ and the five crossover values | CONFIRMED (SymPy and numeric to 0.1 K) |
| 6 | List of two-branch correlations | CORRECTED: added I, Kr, Xe in UO$_2$ (FRG, accident conditions, Table A-1 row 5) |
| 7 | $Q_{\mathrm{eff}}$ 130.1 and 438.2 $\mathrm{kJ\ mol^{-1}}$ | CONFIRMED |
| 8 | $R_g$ exact SI value; TECDOC-978 8.3143 at pp. 435, 165, 470 | CONFIRMED; added that TECDOC-1674 Eq. (10.5a) and BISON (Collin et al., 2021, Eq. 71) use 8.3145 (audit notes 04, 06) |
| 9 | (P22.8) | CORRECTED: symbol $R$ (clashes with particle radius) $\to R_g$; "Q/(RT) up to about 46 for the tabulated rows" $\to$ $Q_{\mathrm{eff}}/(R_gT)$ up to 46.1 for the FRG rows (a single branch reaches 87); multi-branch generalisation stated. Value $9.0\times10^{-4}$ and Sr in UO$_2$ at 1000 $^\circ$C CONFIRMED |
| 10 | CRP-6 1a $F$ = 0.467337 / 0.467404; 1b change $1.9\times10^{-9}$; printed 0.4673, 0.99999959 (Table 10.6, p. 544) | CONFIRMED (independent Booth series) |
| 11 | Validity ranges | CORRECTED: "FRG and USA rows have blank range cells" now lists the exceptions (USA Cs, Sr in PyC; FRG NOC/AC labels) |
| 12 | (P22.9) fluence factor values, footnote, 16 fJ = 0.1 MeV, AGR-1 E > 0.18 MeV (Collin et al., 2016, Table 1, p. 379) | CONFIRMED |
| 13 | Input-validation table; $\ln D = -1097.9$ at 100 K; I in pores $Q = 0$ (row 44); FRG buffer $10^{-8}$, p. 440 | CONFIRMED |
| 14 | (P22.11) to (P22.14) against `finite_volume.rs` lines 78 to 91 (series face conductance, volume centroid, outer half-cell conductance) | CONFIRMED |
| 15 | (P22.16) discrete balance; v1.0 release booked with fixed `outer_conductance` (line 210), uniform step (line 191) | CONFIRMED (code read; numeric balance $1.3\times10^{-14}$) |
| 16 | (P22.17) M-matrix and positivity | CONFIRMED (strict row dominance from $V_j/\Delta t_n > 0$) |
| 17 | Option A "exactly backward Euler in $\tau$" | CORRECTED: holds for $\lambda = 0$ only (with $\lambda > 0$ the iterates differ, $8.8\times10^{-4}$ in a scalar test); added that option A without shared scaling forms $G_f$ from averaged $D_i$, which differs from the averaged $G_f$ by $O(\Delta t^2)$ at interface faces |
| 18 | (P22.18) | CONFIRMED (SymPy series) |
| 19 | Observed BE orders and accuracy ratios 2.24, 1.75 | CORRECTED: orders were quoted as 1.00 and 0.99 to 1.00; script output gives 1.00 to 1.01 / 0.98 to 1.00 (shared $\alpha$) and 0.97 to 0.99 / 0.99 to 1.00 (layer-specific $Q$); ratios CONFIRMED |
| 20 | (P22.19), (P22.20) and the step-bound table | CONFIRMED |
| 21 | CRP-6 heating time steps "120 s to more than $10^5$ s" | CORRECTED $\to$ 86.4 s (COPA) to automatic steps below 300 000 s (ATLAS), Table 10.4, p. 542 |
| 22 | (P22.22), (P22.23) $E_2$ closed form | CONFIRMED (SymPy derivative, 40-digit $E_2$ identity, quadrature to $2.9\times10^{-15}$ relative); symbols renamed $\beta \to \dot T$ (clash with $\beta = 1/T$ in (P22.6)) and $a \to \theta$ (clash with $a_j$, $a_n$) |
| 23 | (P22.24) to (P22.32) effective-time transform, decay factor, interface and boundary conditions | CONFIRMED (SymPy residual 0) |
| 24 | (P22.33) to (P22.35), closure, $M_{\mathrm{out}}$; consistency with v1.0 $F = 1 - \sum w_ne^{-\Lambda_nt}$ (`sturm_liouville.rs` header) | CONFIRMED |
| 25 | Gelbard Eqs. (II.13), (II.14), (II.18), pp. 12 to 14, "fraction released or decayed" | CONFIRMED against audit note 02 |
| 26 | (P22.36), (P22.37) and the steady-state limit | CONFIRMED (SymPy) |
| 27 | (P22.38) Robin condition | CONFIRMED; coefficient renamed $h \to H$ (clash with staircase interval $h$) |
| 28 | Section 5.3.6 "exact if and only if" | CORRECTED $\to$ sufficient conditions; necessity of E2 to E5 is not shown |
| 29 | Section 5.3.6 numerical demonstration | CORRECTED: the $5.4\times10^{-7}$, order 2.00 result is the semi-discrete FV evaluated at $\tau(t)$ by eigen-decomposition, not a real-time run; real-time evidence is the BE and Radau runs; script docstring of `sec5_fv_shared_alpha.py` clarified (re-run, 10/10 PASS) |
| 30 | Section 5.4 jump rule $D_iu_i' - D_{i+1}u_{i+1}' = (D_i - D_{i+1})u/r$ vs `interface_state` in `sturm_liouville.rs` | CONFIRMED |
| 31 | (P22.39) to (P22.45) projection, weight $r^2$ independent of $D$, $P = I$ under shared scaling | CONFIRMED (numeric: projection vs Richardson FV $1.5\times10^{-10}$; inventory mismatch $2.6\times10^{-7}$) |
| 32 | Mode-count symbol $M$ (clash with inventory $M(t)$ of the shared notation) | CORRECTED $\to n_{\max}$ throughout; truncation rule linked to `min_valid_time` ($\Lambda_{\mathrm{covered}}t \ge 40$) |
| 33 | (P22.46) interface-jump identity | CONFIRMED (numeric residual $2.1\times10^{-11}$, sign test); clarified that the integral is layer by layer; field renamed $f \to g$ |
| 34 | Decay rates $m^{-2.03}$ and envelope $n_{\max}^{-2.9}$ | CONFIRMED (author output; checker slope $-1.90$ in a different case) |
| 35 | "in line with (P22.44)" for $t_s = 0.05$ h | CORRECTED: errors fall as $\Lambda t_s$ grows; rule (P22.44) is conservative ($\Lambda t_s = 32$ already at reference accuracy for $t_s = 1$ h) |
| 36 | (P22.47) to (P22.50) Galerkin form, symmetry of $N^{1/2}KN^{-1/2}$ | CONFIRMED |
| 37 | (P22.51) commutator expansion and fixed-ratio limit | CONFIRMED (SymPy) |
| 38 | "off-diagonal reaches 20 times the diagonal" | CORRECTED: made precise, $\max_{m\ne n}\lvert K_{mn}\rvert/K_{mm} = 20.4$ over the first 10 modes |
| 39 | (P22.52) POD projection, $\Psi^{\mathsf T}V\Psi = I$ | CONFIRMED by algebra; untested numerically (OPEN, as stated by the author) |
| 40 | (P22.53) to (P22.55) Magnus terms | CONFIRMED (SymPy); generator renamed $A \to B$ (clash with $A_f$, $A^{(i)}$) and interval count $K \to N_T$ (clash with $K_{mn}$) |
| 41 | Per-layer average "reproduces $\Omega_1$ exactly" | CORRECTED: exact for operators affine in $D_i$; on the FV matrix the harmonic interface conductance leaves an $O(h^3)$ local defect (observed 2.05), order 2 unchanged |
| 42 | (P22.56) staircase orders and error table | CONFIRMED for $F(6\ \mathrm{h})$ (checker: left 0.99/1.00, mid 2.01/2.00, average 1.93/1.97); added that $F(4\ \mathrm{h})$ midpoint orders scatter 1.80 to 2.11 |
| 43 | (P22.57) shared-scaling exactness of the averaged staircase | CONFIRMED ($G_f$ homogeneous of degree one, checker $3\times10^{-16}$) |
| 44 | AGR-1 programme (Morris et al., 2016, pp. 25, 31, 32, 34; Collin et al., 2016, pp. 379, 380, 382), (P22.58) breakpoints and totals 333.78, 335.95, 338.12 h | CONFIRMED |
| 45 | Shares of $\int D\,dt$ on the 1600 $^\circ$C plan | CONFIRMED; added the UO$_2$ values for the pre-1250 and cool-down parts (0.13 %, 0.06 %) |
| 46 | Thermocouple factors 1.27 and 1.15 | CONFIRMED (exact two-branch ratios 1.2734, 1.1529) |
| 47 | (P22.59) CRP-6 conversion and nine totals (Tables 10.11, 10.12, pp. 554 to 555, 568); case 3c definition (Table 10.5, p. 543) | CONFIRMED |
| 48 | Author script coverage claims | CORRECTED: `sec5_projection_step.py` tests only the decay rate implied by (P22.46), not the identity; Section 5.6 table amended |
| 49 | References (APA 7, DOIs, editor of TECDOC-978) | CONFIRMED against audit notes 02 to 06; Gelbard DOI remains NOT VERIFIED |
| 50 | Fluence cutoff conversion, as-run AGR-1 histories, CRP-6 ramp shape, stiff-regime staircase proof, POD interface term | OPEN (as listed in Section 5.7) |

Table: B.3 Independent check of Section 5.

#### B3 reconciliation note

A second independent re-derivation (B3, 2026-11-06; `b3_sec5.py` 23/23 PASS) confirmed the Arrhenius forms, the effective-time transform with decay and source, the conditions for exactness, the projection method and the staircase orders. Changes: item 10 of the log above was wrong in one respect, since $F = 0.467404$ (SI constant) rounds to 0.4674, not 0.4673 (Section 5.1.4 corrected); Section 5.4.2 now shows that with layer-specific $Q$ the solution is not a function of the layer integrals $\int D_i\,dt$ (order-swap test), which replaces the bare statement that no closed form exists; Section 5.4.1 records a second fitted coefficient-decay exponent ($-1.74$). Item 50 stays OPEN. Details: `<work folder>/derive/B3_reconciliation.md`.

#### B7 review note

B7 final review, 2026-11-06. All six `sec5_*` scripts re-run: 25/25, 16/16, 10/10, 9/9, 17/17 and 9/9 PASS. Independent checks in `<work folder>/b7_checks/` (`b7_c3_crp6_arrhenius.py`: crossover temperatures, $Q_\mathrm{eff}$, the step-bound table, the $R_g$ sensitivity and the $\pm16\ \mathrm{K}$ factors; `b7_c4_temperature.py`: AGR-1 breakpoints and the shares of $\int D\,dt$, (P22.22) at 40 digits, and a two-layer FV run in real time with decay that matches $e^{-\lambda t}\hat M(\tau(t))$ to $1.1\times10^{-15}$ and the release by parts (P22.35) to $2.4\times10^{-13}$ for a shared $Q$, while a layer-specific $Q$ gives a visible error, as condition E1 requires). Text changes: a corrupted `\times` (a tab character had replaced the backslash and the letter t) in Section 5.4.2 now reads $8.2\times10^{-15}$; three table cells use `\ast` instead of a literal asterisk and one uses `\lvert`, `\rvert`. No equation changed.

### B.4 B3 reconciliation of Sections 3, 4 and 5

Date: 2026-11-06. Scope: the three Phase 2 derivation drafts `sec3_phase21_decay_source_analytical.md`, `sec4_phase21_decay_source_fv.md` and `sec5_phase22_temperature.md` in this folder. The drafts and their "Independent check log" sections were not assumed correct. Every key result listed in the B3 brief was re-derived with new scripts that import nothing from the repository, from the drafts' scripts or from the earlier checker scripts. No Rust file was read for editing and none was changed.

Scripts (all in `<work folder>/b3_checks/`, run with `.venv/Scripts/python.exe -I -X utf8 <script>`; saved output in the `.out` file of the same name):

| Script | What it does | Result |
|---|---|---|
| `b3_layers.py` | own layered eigen-solver in $u = rc$ form (analytic sine and cosine per layer, transfer across interfaces, bracketing in $\sqrt{\Lambda}$) and own FV assembly with the `build_mesh` rules (volume centroids, series face conductance, $G_R$ with $R - r_N$) | helper |
| `b3_sec3_symbolic.py` | SymPy and mpmath: projection, integrating factor, decay factor, closure, $M_\mathrm{out}$, Booth limits, precursor, activation and chain solutions | 36/36 PASS |
| `b3_sec3_layered.py` | two-layer case of Section 3.9: eigenpairs, orthogonality, flux identity, modal values against FV integrated exactly in time; five-layer flux identity | 10/10 PASS |
| `b3_completeness.py` | Green function of the layered operator against (P21.82), Hilbert-Schmidt norm | 2/2 PASS |
| `b3_sec4.py` | BE system with decay and source in symbols, pivots, link to v1.0, integrating-factor fixed point, error expansions, outside inventory, exact steady states, own transfer walk for (P21.85), MMS, CRP-6 1a and 1b, FV runs | 32/32 PASS |
| `b3_sec4_split.py` | fixed-point bias of the decay-exact split scheme, both readings of $R/B$ | values listed below |
| `b3_sec5.py` | Arrhenius forms and register values, effective-time transform in symbols, variable-$\alpha$ modal solution, real-time stiff FV against the $\tau$ solution, order-swap test, option A, continuous eigenfunction projection at a temperature step, staircase orders, Magnus terms | 23/23 PASS |

Table: B.4 Independent scripts and their results.

#### Reconciliation table

Status: CONFIRMED (independent result agrees with the draft), CORRECTED (draft text changed in place, or an OPEN item closed), OPEN (not resolvable from the available sources). Evidence cites the script and the tag of the printed line.

| Item | Draft result | Independent result | Status | Evidence |
|---|---|---|---|---|
| Modal ODE (P21.28) | projection with weight $r^2$ gives $da_n/dt = -(\Lambda_n+\lambda)a_n + s_n$ | self-adjointness checked on a test field for $m = 1, 2, 3$; Green identity (P21.25) for arbitrary $D(r)$ | CONFIRMED | `b3_sec3_symbolic.py` S1a, S1b, S1c |
| Integrating factor (P21.31) | $a_n = a_n(0)e^{-\kappa_nt} + \int_0^t e^{-\kappa_n(t-s)}s_n\,ds$ | residual zero for $s = s_0 + s_1t + s_2\sin\omega t$, initial value correct | CONFIRMED | S2a |
| Constant source (P21.32) and recursion (P21.33) | $a_n(0)e^{-\kappa_nt} + (s_n/\kappa_n)(1 - e^{-\kappa_nt})$, limit $s_n/\kappa_n$; interval recursion | dsolve reproduces (P21.32); recursion equals the integral over three intervals to $2.3\times10^{-41}$ | CONFIRMED | S2b, S2c |
| Decay factor (P21.80) and Section 3.6 remark | $c = e^{-\lambda t}\hat c$, $\hat c$ driven by $e^{\lambda t}S$ | residual zero, also with $D = D(r,t)$ | CONFIRMED | S3a |
| Inventory closure (P21.42) | $M_0 + M_\mathrm{gen} = M + M_\mathrm{dec} + M_\mathrm{rel}$ | per mode in symbols; semi-discrete FV exact in time $1.1\times10^{-12}$; BE over 70 variable steps $6.1\times10^{-15}$ | CONFIRMED | S4a; `b3_sec3_layered.py` L8; `b3_sec4.py` X6c |
| Completeness sum for the kernel source | $4\pi\sum_nI_ns_n = G$ | summed in closed form for the homogeneous sphere | CONFIRMED | S4d |
| $M_\mathrm{out}$ (P21.43), (P21.44) | $M_\mathrm{out}' = J_R - \lambda M_\mathrm{out}$; $M + M_\mathrm{out} = M_0e^{-\lambda t} + \int Ge^{-\lambda(t-s)}ds$ | Leibniz derivative of both convolutions | CONFIRMED | S4f, S4g |
| Flux identity (P21.36) | $R^2D_L\phi_n'(R) = -\Lambda_nI_n$ | homogeneous sphere in symbols; two layers, 400 modes, max rel $2.9\times10^{-11}$; five layers $6.5\times10^{-13}$ | CONFIRMED | S4e; L2; L10 |
| Booth $R/B(t)$, TECDOC-1674 (7.7) | $w_n\Lambda_n/\kappa_n = 6/(n^2\pi^2 + x)$, $x = \lambda/D'$ | identity in symbols | CONFIRMED | S5a |
| TECDOC-1674 (7.8) and (P21.48) | $F_B = M_\mathrm{out}/(M + M_\mathrm{out})$, not $M_\mathrm{rel}/M_\mathrm{gen}$ | closed form, definition and direct quadrature of $\int J_Re^{-\lambda(t-s)}ds$ agree at three $(x, D't)$ points (e.g. 0.275559830223 all three); $M_\mathrm{rel}/M_\mathrm{gen} = 0.26480$ against 0.27556 | CONFIRMED | S5e, S5f |
| (7.6) closed form and limits (P21.49) | $\sum 6/(n^2\pi^2+x) = 3x^{-1/2}(\coth\sqrt x - x^{-1/2})$; $1 - x/15 + 2x^2/315$; $3/\sqrt x$ | agreement below $10^{-25}$ at $x = 0.01, 1, 100$; series and limit in symbols | CONFIRMED | S5b, S5c, S5d |
| $\lambda \to 0$ of (7.8) | reduces to TECDOC-1674 (7.1) | 0.563649970714 for both at $D't = 0.1$; (7.1) is the in-reactor constant-production form (audit 04, printed p. 317) | CONFIRMED | S5g |
| Steady $R/B$ balance form (P21.39) | $1 - \lambda\sum w_n/\kappa_n$ | equals (7.6) at $x = 1$ | CONFIRMED | S5h |
| Two-layer values of Section 3.9 | $M = 6.941502\times10^{-10}$, $M_\mathrm{rel} = 1.019337\times10^{-9}$, $M_\mathrm{dec} = 5.314425\times10^{-10}$ mol, $R/B(t_\mathrm{irr}) = 0.671485$, $(R/B)_\mathrm{ss} = 0.68638073$ | same values from own eigen-solver | CONFIRMED | L5 |
| FV against modal, two layers | second order | orders 2.00 for $M$, $M_\mathrm{rel}$, $M_\mathrm{dec}$, $R/B(t_\mathrm{irr})$, $(R/B)_\mathrm{ss}$ (centroid nodes, exact time) | CONFIRMED | L7 |
| Truncation orders, Section 3.5.2 | worst case $N^{-1}$ and $N^{-3}$; two-layer case $N^{-2.1}$ and $N^{-4.0}$, 284 sign changes of $w_n$ | direct order 1.97, balance 4.06; 284 sign changes | CONFIRMED | L9 |
| Precursor lag (P21.12), $t_{99}$ | $B(t) = y^C\dot Ft - y_p\dot F(1 - e^{-\lambda_pt})/\lambda_p < y^C\dot Ft$; $t_{99} = 6.644\,t_{1/2}$; Xe-137 0.423 h | dsolve; 6.64386; 0.4228 h | CONFIRMED | S6a to S6d |
| Activation (P21.16) and chain (P21.21) | closed-form modal solutions | residuals zero | CONFIRMED | S6e, S6f |
| Completeness of the layered eigenfunctions (Section 3 log item 38) | OPEN, numerical support only | proof: the Green operator $G(r,s) = g(\max(r,s))$, $g(x) = \int_x^Rd\rho/(\rho^2D)$, is symmetric, injective and Hilbert-Schmidt on $L^2(r^2dr)$, so its eigenfunctions (those of (P21.24)) are complete; Green function reproduces (P21.82) exactly; norm finite | CORRECTED (OPEN closed; text added to Section 3.10) | `b3_completeness.py` C1, C2 |
| Cross sections, isomeric fraction, flux histories, kernel heavy-metal density, daughter diffusivities; CRP-6 decay constants (Section 3 items 39, 40) | NOT VERIFIED | no source available in the audit notes | OPEN | none |
| BE system with decay and source (P21.58) to (P21.60), per-step closure (P21.70) | $V/\Delta t + \lambda V$ on the diagonal; closure exact | solved 4-cell system in symbols, residual zero; $\mathbf 1^{\mathsf T}K = G_R\mathbf e_N^{\mathsf T}$ | CONFIRMED | `b3_sec4.py` X1, X1b |
| Subtraction-free pivots (P21.61), positivity (P21.62) | $d_j = e_j + g^E_j$; $C \ge 0$ | pivots of Gaussian elimination equal the recurrence for $N = 5$; min $C = 1.2\times10^{-26} \ge 0$ | CONFIRMED | X2, X6d |
| Link to v1.0 (P21.63) | step $= (1+z)^{-1}\times$ v1.0 step with $\Delta t' = \Delta t/(1+z)$ | matrix identity in symbols | CONFIRMED | X3 |
| Integrating-factor fixed point (P21.68) | $(\varphi(z)K + \lambda V)C = VS$, $\varphi = 1 + z/2 + z^2/12 + O(z^4)$; unsplit BE keeps the exact steady state | fixed point and series in symbols (the $z^3$ coefficient is zero) | CONFIRMED | X4a, X4b, X4c |
| Split-scheme bias percentages (Section 4.4) | balance reading $+0.15$, $+10$, $+127$ %; flux reading $-4.7$, $-36$, $+4.9$, $+44$ % | $+0.0015$, $+0.1018$, $+1.2740$; $-0.0469$, $-0.3604$, $+0.0485$, $+0.4374$; $\lambda/\Lambda_1 = 9.77$ | CONFIRMED | `b3_sec4_split.out` |
| Error expansions (P21.57), (P21.64), (P21.69) | rectangle $-\tfrac12h^2\dot S - \tfrac13h^3\ddot S$ at $t^n$; $-z^2/2$; per mode $\tfrac12\kappa^2t\Delta t$, ratio 4 | all in symbols | CONFIRMED | X5a to X5d |
| Step rules (P21.77), (P21.78) | Ag-110m $\Delta t \le 3.6\times10^4$ s; I-131 $\lambda$; CRP-6 1b 5.1 % at 360 s | $3.620\times10^4$ s; $1.0002\times10^{-6}$ and $9.9967\times10^{-7}$; $\Lambda_1 = 1.9745\times10^{-5}$, predicted 0.0505 | CONFIRMED | X5e, X5f, X11d |
| FV CRP-6 1b | $1 - F = 4.28\times10^{-7}$ (360 s), $4.09\times10^{-7}$ (36 s) | $4.2796\times10^{-7}$, $4.0903\times10^{-7}$; Booth $4.0703\times10^{-7}$ | CONFIRMED | X11e |
| Outside inventory (P21.73) to (P21.76) | BE local error $\tfrac12h^2\lambda(\lambda M_\mathrm{out} - J)$, fixed point $J/\lambda$, closure | series and fixed points in symbols; closure $1.1\times10^{-16}$ | CONFIRMED | X6a, X6b, X6c |
| Multilayer steady state with kernel source (P21.82) to (P21.84) | profile, $J_R = \mathrm{Gen}$, $M_\mathrm{ss}$, $\tau_\mathrm{res}$ | all conditions and the volume integral in exact rationals | CONFIRMED | X7a to X7d |
| Steady state with source and decay (P21.85) | $R/B = 0.9684291531$; FV 0.968429 on the (20, 8, 4, 4, 4) mesh | own transfer walk (one unknown, not the draft's nine-equation system): 0.96842915310057; FV order 2.00, first mesh 0.96842866 | CONFIRMED | X8a, X8b |
| Booth steady state (P21.86), (P21.87) | correct formula, but $x = \sqrt{\lambda/D'}$ and $a = \lambda/D'$ while Section 3 uses $x = \lambda/D'$ | formula confirmed; notation unified to $x = \lambda/D'$ | CORRECTED (notation) | X9a, X9b |
| Manufactured solution (P21.88), (P21.89) | conditions, source, $Q(0) = 3\pi^2/(4R^2)$ | in symbols | CONFIRMED | X10 |
| CRP-6 1a and 1b (P21.91) | 1a $F$ 0.46742 (8.3145), 0.46721 (8.314); 1b 0.99999959; single branch 0.46163, 0.99995461 | same; added 8.3143 gives 0.467337, the only value that rounds to the printed 0.4673 | CONFIRMED (sentence added) | X11a to X11c |
| Decay shift in 1b | $-\lambda\tau_\mathrm{res} = -2.43\times10^{-5}$ | FV shift $-2.4332\times10^{-5}$ | CONFIRMED | X11f |
| Wording of Eq. (7.8) in TECDOC-1674 (Section 4 log, OPEN) | NOT VERIFIED | printed p. 319 (PDF index 332) introduces it only as the cumulative fractional release; no definition by atom populations | CORRECTED (OPEN closed; text changed in Section 4.7.4) | PyMuPDF text of `Paper/3.pdf` page index 332 |
| $R_g$ used by CRP-6 participants; decay in their runs | NOT VERIFIED | Chapter 10 states $R = 8.3145$ for Eq. (10.5a) (printed p. 532) but not what each participant used; 8.3143 reproduces 0.4673 | OPEN | `b3_sec4.py` X11a; PDF search, printed pp. 530 to 536 |
| Arrhenius forms (P22.1), (P22.6), (P22.7) | $Q_\mathrm{eff}$ as $D$-weighted mean; $T_x$ formula; 1772.4, 1835.0, 2014.1 K; $Q_\mathrm{eff}$ 130.1 and 438.2 kJ/mol | in symbols; same values from the register CSV | CONFIRMED | `b3_sec5.py` T1a to T1d |
| $D_0$ span | $8.8\times10^{-15}$ to $1.8\times10^6$, 20.3 decades (Tables A-1 to A-3) | same; Table A-4 rows would extend it to $10^{-18}$ to $8.7\times10^7$, which the draft correctly excludes | CONFIRMED | T1e |
| CRP-6 1a and the gas constant (Section 5.1.4) | 0.467337 (8.3143) and 0.467404 (SI) "both round to 0.4673" | 0.467404 rounds to 0.4674; only 8.3143 gives 0.4673 | CORRECTED | T1f |
| Effective-time transform (P22.24) to (P22.31) | $c = e^{-\lambda t}\hat c(r, \tau(t))$, decay cancels, $\hat S = e^{\lambda t}S/\alpha$ | residual zero in symbols; real-time stiff FV against the $\tau$ solution $7.0\times10^{-13}$ ($\lambda > 0$) and $3.4\times10^{-14}$ (with source) | CONFIRMED | T2a, T3a, T3b |
| Without the decay factor (P22.32) | decay rate $\lambda/\alpha$ in $\tau$ | in symbols | CONFIRMED | T2b |
| Exactness conditions, shared $\alpha$ (Section 5.3.6) | E1 (one $\alpha$ for all layers) needed for a single $\tau$; E2 to E5 sufficient | with $\alpha_1 \ne \alpha_2$ the transformed interface condition carries $\alpha_1D_1^\ast/(\alpha_2D_2^\ast)$, which varies with $t$; decay must be uniform (T2a uses one $\lambda$) | CONFIRMED | T2c |
| Variable-$\alpha$ modal solution (P22.36) | solves $a' = -(\lambda + \alpha\Lambda^\ast)a + s$ | residual zero for arbitrary $\tau(t)$ and $s(t)$ | CONFIRMED | T2d |
| Release by parts (P22.35) and closure | $M_\mathrm{rel} = e^{-\lambda t}\hat M_\mathrm{rel} + \lambda\int e^{-\lambda s}\hat M_\mathrm{rel}ds$ | equals $\int J\,dt$ per mode, closure with $M_\mathrm{rel} = \int J\,dt$, max rel $1.4\times10^{-16}$ | CONFIRMED | T2e |
| Option A in effective time | exactly BE in $\tau$ for $\lambda = 0$ only | $2.0\times10^{-15}$ for $\lambda = 0$; $2.7\times10^{-3}$ for $\lambda > 0$ (30 steps) | CONFIRMED | T3e |
| Projection at a temperature switch (P22.40) to (P22.45) | exact for the full series | continuous projection with own eigenfunctions, 60 modes per basis: $F = 0.5640460376$ against Richardson FV 0.5640460370 (FV order 1.99); inventory mismatch at the switch $3.0\times10^{-6}$ | CONFIRMED | T4a |
| Coefficient decay (P22.46) | $a_m = O(m^{-2})$, observed $m^{-2.03}$ | fitted $-1.74$ for $20 \le m \le 60$ in another case | CONFIRMED as an approximate asymptotic estimate (sentence added) | T4b |
| Noncommuting operators, no closed form (Section 5.4.2) | "no closed form exists" when the ratios $D_i/D_j$ change | relative commutator $9.5\times10^{-3}$; two histories with equal $\int D_i\,dt$ in every layer give $F = 0.011413$ and $0.009919$; shared $Q$ gives agreement to $8.2\times10^{-15}$. So no effective time exists; a general closed form is not available, while piecewise-constant $T$ has the exact ordered product (P22.40) to (P22.43) | CORRECTED (wording made precise with the demonstration) | T3c, T3d |
| Magnus terms (P22.54), (P22.55) | $\Omega_2 = -(h^3/12)[B_0,B_1] + O(h^4)$; $\Omega_1$ defects | $\Omega_2 = -h^3C_{01}/12 - h^4C_{02}/12 - h^5C_{12}/60$; defects in symbols | CONFIRMED | T5d, T5e |
| Staircase orders (P22.56) | left 1, midpoint 2, per-layer average 2 | own case (synthetic layer-specific $Q$, 1600 to 1800 $^\circ$C in 4 h then 2 h hold, Radau and BDF reference agree to $2.6\times10^{-14}$): last-pair orders 1.00, 2.01, 2.04 | CONFIRMED (numerical) | T5a to T5c |
| Section 5 log item 50 (fluence cutoff conversion, as-run AGR-1 histories, CRP-6 ramp shape, stiff-regime staircase proof, POD interface term) | OPEN | unchanged | OPEN | none |

Table: B.5 Reconciliation of the key results of Sections 3, 4 and 5.

#### CORRECTED items from the earlier check logs

Each CORRECTED row of the three check logs was looked up in the current text. All were applied: Section 3 rows 8, 9, 13, 14, 15, 22 to 24, 29, 30, 32 to 36 (for example "runs ahead", the $t_{99}$ wording, the $1/d_n$ scaling, the Te-131m caveat, $G$ for production, $\kappa_n$ in all prose with two remaining $k_n$ only inside the explanatory symbol row and the log); Section 4 rows on (P21.57), (P21.69), the 16 mean lives, the I-131 source, $V_K$, $r_j$, the Gelbard citation, the truncation rule of (P21.81), the mesh of the FV value, the (P21.87) note, the script label "at the finest step" (present in `sec4_p21_booth_crp6.py`) and the MOOSE citation; Section 5 rows 2, 4, 6, 9, 11, 17, 19, 21, 22, 27 to 29, 32, 33, 35, 38, 40 to 42, 45, 48.

#### Changes made to the section files

- `sec3_phase21_decay_source_analytical.md`: Section 3.10, completeness bullet replaced by the Green-function argument; "B3 reconciliation note" appended to the check log.
- `sec4_phase21_decay_source_fv.md`: (P21.86) and (P21.87) rewritten with $x = \lambda/D'$; Eq. (7.1) identified as the in-reactor constant-production form; the Eq. (7.8) wording statement replaced by the PDF finding; 8.3143 result added in Section 4.7.6; note appended to the check log.
- `sec5_phase22_temperature.md`: Section 5.1.4 rounding claim corrected; Section 5.4.1 second fitted exponent added; Section 5.4.2 closed-form statement replaced by the order-swap demonstration; note appended to the check log.

No new em dashes; equation numbers unchanged.

#### Re-run of the section scripts after the edits

All 17 scripts in `scripts/derivation_checks/` named `sec3_*`, `sec4_*` and `sec5_*` were re-run after the edits (outputs in `b3_checks/rerun/`). Each exited with code 0 and printed no FAIL line: `sec3_p21_booth` 20/20, `sec3_p21_chain_overlap` 5/5, `sec3_p21_decay_source` 24/24, `sec3_p21_fv_two_layer` 14/14, `sec3_p21_modal_ode` 32/32, the six `sec4_p21_*` scripts `ALL PASS`, `sec5_arrhenius` 25/25, `sec5_effective_time` 16/16, `sec5_fv_shared_alpha` 10/10, `sec5_projection_step` 9/9, `sec5_schedules` 17/17, `sec5_staircase` 9/9. No script was changed by B3 (the edits touched prose only).

Two of my own first-draft checks failed for reasons in my scripts and were fixed before the final run: the $\lambda \to 0$ limit of (7.8) was first compared with the initial-inventory Booth series instead of the in-reactor form (7.1), and the one-material limit of $\tau_\mathrm{res}$ was first tested without setting $R_1 = R$. Neither affected a draft result.

#### Remaining scientific discrepancies and open points

1. Staircase convergence in the stiff regime ($h\lVert B\rVert \sim 10^2$ to $10^3$) is shown numerically in two independent cases (draft and B3) but not proved; the Magnus argument is formally valid only for small $h\lVert B\rVert$.
2. The completeness proof relies on the Hilbert-Schmidt theorem; no textbook citation was opened in this session.
3. The earlier checker quoted a balance-reading bias of $+1.283$ for the short-lived nuclide at $z = 1$; B3 gives $+1.2740$ on the (20, 8, 4, 4, 4) mesh, which agrees with the draft's $+127$ %. The checker line in the Section 4 log was left verbatim; the difference most likely comes from a different mesh (not checked).
4. Which $R_g$ and which decay treatment the CRP-6 participants used remains NOT VERIFIED; the printed 0.4673 is reproduced at four digits with $R_g = 8.3142$ or $8.3143$ (0.467296, 0.467337), while 8.314 gives 0.4672 and 8.3144, 8.3145 and the SI value give 0.4674.
5. Data gaps unchanged: capture and burn-out cross sections, the Ag-110m isomeric fraction, flux histories, kernel heavy-metal density, daughter diffusivities, the fluence cutoff conversion, as-run AGR-1 temperature histories and the CRP-6 ramp shape.
6. Notation: Section 4 still uses $B$ both for the birth rate (TECDOC $R/B$) and for the v1.0 step matrix $\mathbf B_h$; not changed because both uses are local and defined.
7. All results remain DERIVED NOT IMPLEMENTED for v2.0: no Rust was written, so nothing here is IMPLEMENTED.
