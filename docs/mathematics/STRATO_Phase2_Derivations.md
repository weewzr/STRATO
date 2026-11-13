# STRATO v2.0 Phase 2 derivations: decay, sources and inventory (Phase 2.1), temperature-dependent diffusivity (Phase 2.2)

Date: 2026-11-06; open-item results added 2026-11-13 (Sections 3.5.3, 4.6.1, 5.2.5, 5.3.7, 5.3.8 and 5.4.6, Appendix B.5). Status: DRAFT FOR REVIEW.

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
| Part 2.1, analytical | 3 | transport equation with decay and source, decay law, yields, activation products, chains, modal solution, release, rigorous truncation bound for the layered particle, inventory closure, Booth limits | (P21.1) to (P21.49); (P21.93) to (P21.112) in Section 3.5.3 |
| Part 2.1, finite volume | 4 | cell balance with decay and source, Backward Euler with implicit decay, decay-exact split, discrete closure, $M_\mathrm{out}$, step-size rules, second-order integrators and positivity, exact references, verification matrix | (P21.50) to (P21.92); (P21.113) to (P21.142) in Section 4.6.1 |
| Part 2.2, temperature | 5 | Arrhenius law, finite volume with $D(T(t))$, effective time, step limit with $D_i(T(t))$, start of the release quadrature, parent and daughter with different diffusivities, layer-specific activation energies, stiff-regime staircase, temperature histories | (P22.1) to (P22.59); (P22.60) to (P22.142) in Sections 5.2.5, 5.3.7, 5.3.8 and 5.4.6 |
| References | after 5 | all works cited in Sections 1 to 5 and Appendix B, APA 7 | none |
| Appendix A | A | verification scripts and their result lines from the B8 re-run (A.1) and from the open-item run of 2026-11-13 (A.2) | none |
| Appendix B | B | independent verification record: the three check logs, the B3 reconciliation and the open-item resolution of 2026-11-13 (B.5) | none |

Table: 1.1 Structure of the document.

Section numbers 3, 4 and 5 and the equation numbers are those of the reviewed drafts, so that every cross-reference in the pseudocode, the change document `docs/development/v1_to_v2_changes.md` and the parameter register stays valid. The open-item work of 2026-11-13 added the subsections 3.5.3, 4.6.1, 5.2.5, 5.3.7, 5.3.8 and 5.4.6 with the new equations (P21.93) to (P21.142) and (P22.60) to (P22.142), numbered after the last existing numbers; no earlier section or equation number changed. Phase 1 equations are cited by their tags in `docs/mathematics/phase1/TRISO_Finite_Volume_Derivation.md` (TRISO-ML-xxx, TRISO-FV-xxx, TRISO-DEL-xxx).

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

Operations are classified as IMPLEMENTED (with file and line in the v1.0.0 tag), DERIVED NOT IMPLEMENTED or FUTURE. Numerical properties are classified as PROVED, DEMONSTRATED NUMERICALLY or CLAIMED ONLY. The full classification tables are in Section 9 of the pseudocode; the review package `docs/development/phase2_review_package.md` gives a summary. The subsections added on 2026-11-13 label each mathematical statement PROVED (complete argument given), DEMONSTRATED NUMERICALLY (script output only), CLAIMED ONLY or OPEN, and say in addition whether a formula is exact, a series, approximate or numerical.

### 1.4 Sources and provenance

Page numbers are printed page numbers. A phrase such as "audit 07, Section 2" points to the Stage 1 audit notes, which were checked against the PDFs of the papers in the local corpus and are kept in the work folder, not in the repository. Numerical values of decay constants, yields and diffusivity correlations are collected, with their sources and status, in the parameter register `data/reference/STRATO_Parameter_Register.md` and the CSV files under `data/`. Code links point to the tag v1.0.0, for example [src/numerical/finite_volume.rs, lines 162 to 182](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L162-L182) is the v1.0 Backward Euler step.

### 1.5 Review history

Each part was drafted, then checked by an independent re-derivation whose findings are recorded in that part's check log. A reconciliation pass (B3) re-derived the key results again with new scripts that import nothing from the repository, corrected the drafts where needed and recorded each item as CONFIRMED, CORRECTED or OPEN. A final review (B7) re-ran every script, ran further independent checks and corrected wording and formatting errors; its notes close each check log. All of this is in Appendix B. The assembly of this document (B8) moved the text into one file, changed heading levels, split one-line display equations into the three-line form, added table captions and code links and merged the reference lists. Absolute local paths were shortened: repository paths are now relative to the repository root, and paths in the work folder, which is not part of the repository, start with `<work folder>/`. Three phrases were made explicit for the joined document: "the FV section" now reads "Section 4", the completeness remark in Section 3.10 names the Phase 1 derivation file, and the single-branch law of Section 4.7.6 is named as TECDOC-1674 Table 7.1 (audit 04). No equation, number or status was changed; the assembly script checks that all 151 tagged display equations are identical before and after.

**Open-item work (2026-11-13).** Six open points of the pseudocode were worked on: O4 (truncation bound for the layered particle), open issue 5 of Section 4.10 (second-order integrators and positivity, item P2), O12 (diffusion step limit with a temperature history), the accuracy note of block R2.4 step 3 (start of the release quadrature with decay, item R24), O11 (parent and daughter with different diffusivities) and O10 (staircase in the stiff regime). Each item was derived in a separate draft with its own check script in `scripts/derivation_checks/`, then checked by an adversarial verification with a new script that reuses none of the author's code. Every verdict was CONFIRMED WITH FIXES, and the fixes were applied to the drafts before they were moved here as Sections 3.5.3, 4.6.1, 5.2.5, 5.3.7, 5.3.8 and 5.4.6. Their placeholder tags were renumbered to (P21.93) to (P21.142) and (P22.60) to (P22.142), after the last existing numbers; Table B.7 lists the correspondence. The output of the six new scripts is in Appendix A.2 and the verification records are in Appendix B.5. The citations for O13 ($N_A$ and $k_B$) and O14 (the spectral theorem used for completeness) were read in the documents named in the References (Appendix B.5.8). No earlier equation number was changed; earlier labels changed only in the sentences that these citations and results now support, and each such sentence says so.

### 1.6 Suggested reading order

1. Section 2 for notation, then the symbol table at the start of each part (Sections 3.0, 4.0 and 5.0).
2. Section 3.1 (the extended transport equation) and Section 3.6 (inventory closure). These two fix the physics of Phase 2.1.
3. Sections 4.3 and 4.5: the only change to the v1.0 step and the discrete closure that the code will check.
4. Sections 5.1, 5.2.3 and 5.3.6: the Arrhenius law, the time level of the conductances and the conditions under which the effective time is exact.
5. The status tables (Table 3.5, Table 4.3 and Table 5.6) and the open issues (Sections 3.10, 4.10 and 5.7).
6. For the open items worked on 2026-11-13: the status tables 3.8 (Section 3.5.3), 4.7 (Section 4.6.1) and 5.10 (Section 5.2.5), the status lists of Sections 5.3.7, 5.3.8 and 5.4.6, and the summary Table B.6.
7. Appendix B only where a result needs its evidence.

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

Equation numbers (P21.1) to (P21.49), and (P21.93) to (P21.112) in Section 3.5.3. Phase 1 equations are cited by their tags in `docs/mathematics/phase1/TRISO_Finite_Volume_Derivation.md` (TRISO-ML-xxx, TRISO-DEL-xxx).

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
| $N_A$ | Avogadro constant, $6.02214076\times10^{23}$ (SI defining constant; BIPM, 2019, Table 1, p. 124) | $\mathrm{mol^{-1}}$ |
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
|----------|---------------|----------------------|-----------------------|------------------------------|
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
\begin{aligned}
B(t) &= y^C\dot F\,t - \frac{y_p\dot F}{\lambda_p}\left(1 - e^{-\lambda_p t}\right)\ \xrightarrow{t\gg 1/\lambda_p}\ y^C\dot F\left(t - \frac{y_p/y^C}{\lambda_p}\right),\\
t_{99} &= \frac{\ln 100}{\lambda_p} = 6.644\,t_{1/2,p}
\end{aligned}
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
\begin{aligned}
&\frac{da^P_n}{dt} = -\kappa^P_n a^P_n + s^P_n,\qquad \frac{da^A_n}{dt} = -\kappa^A_n a^A_n + f_m\sigma_P\phi\,a^P_n,\\
&\kappa^{P}_n = \Lambda_n + \lambda_P + \sigma_P\phi,\ \ \kappa^{A}_n = \Lambda_n + \lambda_A + \sigma_A\phi
\end{aligned}
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
\begin{aligned}
&\frac{1}{r^2}\frac{d}{dr}\left(r^2D_i\frac{d\phi_n}{dr}\right) + \Lambda_n\phi_n = 0,\quad \phi_n,\ D_i\phi_n'\ \text{continuous at } R_i,\\
&\phi_n \text{ regular at } 0,\quad \phi_n(R) = 0
\end{aligned}
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
\begin{aligned}
M_\mathrm{rel}(t) &= 4\pi\sum_n \Lambda_nI_n\int_0^t a_n\,ds,\\
\int_0^t a_n\,ds &= a_n(0)\frac{1-e^{-\kappa_nt}}{\kappa_n}\\
&\quad + \frac{s_n}{\kappa_n}\left(t - \frac{1-e^{-\kappa_nt}}{\kappa_n}\right)\ (\text{constant } s_n)
\end{aligned}
\tag{P21.38}
$$

#### 3.5.2 Slow convergence and the balance form

For a steady kernel source, $a_n \approx s_n/\kappa_n$ and the terms of (P21.37) behave like $G\,w_n\Lambda_n/\kappa_n \to G\,w_n$, which decay only as $n^{-2}$ for the homogeneous sphere ($w_n = 6/(n^2\pi^2)$). The truncation error is then $O(N^{-1})$. The terms of $M$ behave like $w_n/\kappa_n \sim n^{-4}$, error $O(N^{-3})$. These are worst-case rates for sign-definite $w_n$; Section 3.5.3 proves an explicit bound that needs no sign information. In the two-layer kernel-source case of Section 3.9, $|w_n|n^2$ stays of order one but $w_n$ changes sign 284 times in 400 modes, the tails partly cancel, and the independent check observed $N^{-2.1}$ for the direct $M_\mathrm{rel}$ and $N^{-4.0}$ for the balance form (`ind_p21_layered.py`, Section 3.9).

The cure is algebraic. From (P21.28), $\kappa_n\int_0^ta_n\,ds = a_n(0) - a_n(t) + \int_0^ts_n\,ds$. Write $\Lambda_n = \kappa_n - \lambda$ in (P21.38):

$$
\begin{aligned}
4\pi\sum_n^N\Lambda_nI_n\!\int_0^t\!a_n &= \underbrace{4\pi\sum_n^NI_na_n(0)}_{\to M_0} + \underbrace{4\pi\sum_n^NI_n\!\int_0^t\!s_n}_{\to M_\mathrm{gen}}\\
&\quad - \underbrace{4\pi\sum_n^NI_na_n(t)}_{M^{(N)}} - \underbrace{4\pi\lambda\sum_n^NI_n\!\int_0^t\!a_n}_{M_\mathrm{dec}^{(N)}}
\end{aligned}
$$

The two slow sums are truncated completeness sums whose limits are known exactly ($M_0$ from $c_0$, $M_\mathrm{gen}$ from $S$). Replacing them by those limits gives the **balance form** (the steady $R/B$ expression assumes the kernel-uniform source, for which $4\pi s_nI_n/G = w_n$, Section 3.7.2; for another source shape replace $w_n$ by $4\pi s_nI_n/G$):

$$
\begin{aligned}
&M_\mathrm{rel}(t) = M_0 + M_\mathrm{gen}(t) - M^{(N)}(t) - M^{(N)}_\mathrm{dec}(t),\\
&\frac{J_R}{G}\Big|_{\text{steady}} = 1 - \lambda\sum_n\frac{w_n}{\kappa_n}
\end{aligned}
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

#### 3.5.3 Rigorous truncation bound for the layered particle (open item O4)

**Result in one paragraph.** For piecewise-constant $D_i > 0$ on $L \ge 2$ layers with an absorbing surface, a kernel-uniform initial inventory $M_0$ and a kernel-uniform source $0 \le G(t) \le G_\mathrm{max}$, the error of the balance form (P21.39) after $N$ modes obeys the explicit bound (P21.106) below (Bound A). Its constants come from the radii and diffusivities alone: the travel time $\tau = \sum_i L_i/\sqrt{D_i}$, the minimum diffusivity, the kernel and outer-layer data $R_1, D_1, L_L, D_L$, and the number of interfaces $J = L-1$. The source and decay part falls like $(N-J)^{-3}$; the start-inventory part falls like $e^{-\Lambda_{N+1}t}$. The bound needs no sign information on the weights, so the 152 negative weights of the check case do not affect it. A second, sharper bound (Bound B, (P21.112)) uses the first $N$ computed modes and three closed-form Green integrals. **Label: PROVED** (given the completeness of the eigenfunctions, Section 3.10 and open item O14, and two classical results, the min-max principle and the Sturm oscillation theorem, whose proofs are sketched here; no textbook was opened), **and DEMONSTRATED NUMERICALLY** on the synthetic five-layer case and on the five-layer check case of the decay-part script, with every comparison holding. Bound A exceeds the true error by a factor of at least 33 and typically $10^3$ to $10^4$; Bound B by a factor of at least 3.4 in the scenarios of the check script and at least 1.9 in the wider tests of the verification record (Appendix B.5.2), which add stiff decay, more output times and eleven further layer configurations and find no counterexample.

##### 3.5.3.1 Assumptions and notation

- T1. Layers $i = 1,\dots,L$ with $L \ge 2$, outer radii $R_1 < \dots < R_L = R$, $R_0 = 0$, thicknesses $L_i = R_i - R_{i-1}$, constant $D_i > 0$ (one temperature, block R1). Absorbing surface $c(R,t) = 0$, regular centre, interface conditions (P21.2).
- T2. Kernel-uniform initial concentration ($M_0 \ge 0$) and kernel-uniform source $G(t)$ with $0 \le G(t) \le G_\mathrm{max}$, piecewise continuous; decay constant $\lambda \ge 0$ (A4).
- T3. The eigenfunctions of (P21.24) are complete in $L^2(r^2dr)$ (Green-function argument, Section 3.10; O14).
- T4. Where $\Lambda_{N+1}$ or the first $N$ modes enter (the cut-off sharpening of (P21.105) and Bound B), the computed modes are the first $N$ in increasing order, with none skipped.
- T5. Exact arithmetic. Round-off is discussed in Section 3.5.3.8.

Write $\langle f,g\rangle = \int_0^R r^2fg\,dr$, $\hat\phi_n = \phi_n/\sqrt{N_n}$, $u_n = r\hat\phi_n$ (so $\int_0^Ru_n^2\,dr = 1$), $\chi_1$ the indicator of the kernel and $V = R_1^3/3 = \langle\chi_1,1\rangle$. The two projections of the v1.0 weight are

$$
\alpha_n = \langle\chi_1,\hat\phi_n\rangle = \frac{I^{(1)}_n}{\sqrt{N_n}},\qquad \beta_n = \langle 1,\hat\phi_n\rangle = \frac{I_n}{\sqrt{N_n}},\qquad w_n = \frac{\alpha_n\beta_n}{V}
\tag{P21.93}
$$

which is (P21.46), $w_n = 3I^{(1)}_nI_n/(R_1^3N_n)$, and the weight of the v1.0 code (`src/analytical/sturm_liouville.rs`, line 352 at tag v1.0.0). The weights change sign because $\alpha_n$ and $\beta_n$ are projections of two different functions.

| Symbol | Meaning | Units |
|---|---|---|
| $\alpha_n$, $\beta_n$ | projections of $\chi_1$ and of 1 on $\hat\phi_n$ | $\mathrm{m^{3/2}}$ |
| $\tau$ | travel time $\sum_iL_i/\sqrt{D_i}$ | $\mathrm{s^{1/2}}$ |
| $a$, $c$ | $\pi^2/\tau^2$ and $D_\mathrm{min}\pi^2/R^2$ | $\mathrm{s^{-1}}$ |
| $\ell_n$ | proved lower bound of $\Lambda_n$, (P21.101) | $\mathrm{s^{-1}}$ |
| $\Lambda^*$ | validity threshold of the amplitude lemma, (P21.102) | $\mathrm{s^{-1}}$ |
| $C_\alpha$, $C_\beta$ | amplitude constants | $\mathrm{m^{3/2}\ s^{-1/2}}$ |
| $C_w$ | weight constant, $\lvert w_n\rvert \le C_w/\Lambda_n$ | $\mathrm{s^{-1}}$ |
| $\mu_n$ | modal inventory per unit weight, $m_n = w_n\mu_n$ | $\mathrm{mol}$ |
| $h_n$ | per-mode contribution to the balance-form error | $\mathrm{mol}$ |
| $K(t)$ | $G_\mathrm{max} + \lambda\,(M_0 + M_\mathrm{gen}(t))$ | $\mathrm{mol\ s^{-1}}$ |
| $S^{(p)}_N$ | upper bound of $\sum_{n>N}\ell_n^{-p}$ | $\mathrm{s^{p}}$ |
| $X_N(t)$ | upper bound of $\sum_{n>N}e^{-\Lambda_nt}/\Lambda_n$ | $\mathrm{s}$ |
| $Q_{\alpha\alpha}$, $Q_{\beta\beta}$, $Q_{\alpha\beta}$ | Green quadratic forms (P21.111) | $\mathrm{m^3\ s}$ |

Table: 3.6 Symbols of Section 3.5.3.

##### 3.5.3.2 Step 1: exact representation of the truncation error

The modal inventory of mode $n$ is $m_n = 4\pi I_na_n$. With the kernel-uniform start and source, $4\pi I_na_n(0) = M_0w_n$ and $4\pi I_ns_n = G\,w_n$ (P21.30), so (P21.28) gives $m_n = w_n\mu_n$ with

$$
\frac{d\mu_n}{dt} = -\kappa_n\mu_n + G(t),\qquad \mu_n(0) = M_0,\qquad \kappa_n = \Lambda_n + \lambda
\tag{P21.94}
$$

The concentration lies in $L^2(r^2dr)$ and so does the constant 1, so by T3 and Parseval's identity $M(t) = 4\pi\langle c,1\rangle = \sum_nw_n\mu_n(t)$, and the series converges absolutely by the Cauchy-Schwarz inequality. Since $\sum_n\lvert w_n\rvert < \infty$ (from (P21.102)) and $0 \le \mu_n \le M_0 + G_\mathrm{max}t$, the sum and the time integral may be exchanged, so $M_\mathrm{dec}(t) = \sum_nw_n\lambda\int_0^t\mu_n\,ds$. The exact release is $M_\mathrm{rel} = M_0 + M_\mathrm{gen} - M - M_\mathrm{dec}$ (P21.42), and the balance form (P21.39) keeps the same expression with the first $N$ modes in $M$ and $M_\mathrm{dec}$. Subtracting,

$$
E_N(t) := M^{(N)}_\mathrm{rel}(t) - M_\mathrm{rel}(t) = \sum_{n>N}w_n\,h_n(t),\qquad h_n(t) = \mu_n(t) + \lambda\int_0^t\mu_n\,ds
\tag{P21.95}
$$

**Exact.** Integrating (P21.94) gives $\kappa_n\int_0^t\mu_n\,ds = M_0 + M_\mathrm{gen}(t) - \mu_n(t)$, hence

$$
h_n(t) = \frac{\Lambda_n\,\mu_n(t) + \lambda\,\big(M_0 + M_\mathrm{gen}(t)\big)}{\kappa_n}
\tag{P21.96}
$$

**Exact** (symbolic check S4 for a constant source; numerical check T7 for the on-off source). From (P21.94) and $0 \le G \le G_\mathrm{max}$, $0 \le \mu_n(t) \le M_0e^{-\kappa_nt} + G_\mathrm{max}(1 - e^{-\kappa_nt})/\kappa_n$. With $\Lambda_n/\kappa_n \le 1$ and $1/\kappa_n \le 1/\Lambda_n$,

$$
0 \le h_n(t) \le M_0\,e^{-\Lambda_nt} + \frac{K(t)}{\Lambda_n},\qquad K(t) = G_\mathrm{max} + \lambda\,\big(M_0 + M_\mathrm{gen}(t)\big)
\tag{P21.97}
$$

and therefore, with the two weighted tails $T_0$ and $T_1$,

$$
\lvert E_N(t)\rvert \le M_0\,T_0(N,t) + K(t)\,T_1(N),\qquad T_0 = \sum_{n>N}\lvert w_n\rvert e^{-\Lambda_nt},\qquad T_1 = \sum_{n>N}\frac{\lvert w_n\rvert}{\Lambda_n}
\tag{P21.98}
$$

The absolute values are where the cancellation between weights of opposite sign is given up; this is the main source of pessimism (Section 3.5.3.7). The same steps give $\lvert M - M^{(N)}\rvert = \lvert\sum_{n>N}w_n\mu_n\rvert \le M_0T_0 + G_\mathrm{max}T_1$ for the inventory, and $\lambda\sum_{n>N}\lvert w_n\rvert/\kappa_n \le \lambda T_1$ for the steady $R/B$ of (P21.39).

##### 3.5.3.3 Step 2: lower bounds on the eigenvalues

**(a) Min-max comparison.** $\Lambda_n$ are the eigenvalues of the self-adjoint operator of the closed form $\mathfrak a[f] = \int_0^Rr^2D\,f'^2\,dr$ on $\{f: \int r^2(f^2+f'^2)\,dr < \infty,\ f(R) = 0\}$ in $L^2(r^2dr)$; the resolvent is compact (Section 3.10). The form domain does not depend on $D$ because $0 < D_\mathrm{min} \le D \le D_\mathrm{max}$. By the Courant-Fischer principle $\Lambda_n = \min_{\dim W = n}\max_{f\in W}\mathfrak a[f]/\langle f,f\rangle$, and $D_\mathrm{min}\int r^2f'^2 \le \mathfrak a[f] \le D_\mathrm{max}\int r^2f'^2$ with the unit-diffusivity sphere eigenvalues $(n\pi/R)^2$ gives

$$
D_\mathrm{min}\Big(\frac{n\pi}{R}\Big)^2 \le \Lambda_n \le D_\mathrm{max}\Big(\frac{n\pi}{R}\Big)^2
\tag{P21.99}
$$

**PROVED** (classical min-max principle). This bound is weak when one layer has a small diffusivity: in the synthetic case $D_\mathrm{max}/D_\mathrm{min} = 2\times10^4$ and $\Lambda_n/(D_\mathrm{min}n^2\pi^2/R^2) \ge 3.35$ (minimum 3.358).

**(b) Travel-time (Weyl-type) bound.** In layer $i$ put $k_i = \sqrt{\Lambda/D_i}$. With $u = r\phi$ each layer has $u'' + k_i^2u = 0$, and the interface conditions become $u$ continuous and $D_i(u'_- - u/R_i) = D_{i+1}(u'_+ - u/R_i)$. Define the scaled Pruefer angle $\psi$ by $u = \rho\sin\psi$, $u'/k_i = \rho\cos\psi$, $\rho > 0$. Inside a layer $\psi' = k_i(u'^2 - uu'')/(u'^2 + k_i^2u^2) = k_i$ exactly, and $\psi(0) = 0$ because $u = A\sin(k_1r)$ in the kernel. At an interface $u$ keeps its value, so $\sin\psi$ keeps its sign, and the new angle is chosen in the same open interval $(m\pi,(m+1)\pi)$ as the old one (if $u(R_i) = 0$ both angles equal $m\pi$, since $u'_+ = (D_i/D_{i+1})u'_-$). The jump $\delta_j$ at interface $j$ therefore satisfies $\lvert\delta_j\rvert < \pi$, and

$$
\psi(R;\Lambda) = \sqrt\Lambda\,\tau + \sum_{j=1}^J\delta_j,\qquad \tau = \sum_{i=1}^L\frac{L_i}{\sqrt{D_i}},\qquad \lvert\delta_j\rvert < \pi
$$

Inside layers $\psi$ increases and interfaces never move it across a multiple of $\pi$, so the zeros of $u$ in $(0,r]$ number $\lfloor\psi(r)/\pi\rfloor$. By the oscillation lemma below, $u_n$ has $n-1$ zeros in $(0,R)$ and $u_n(R) = 0$, so $\psi(R;\Lambda_n) = n\pi$ and

$$
(n - J)\,\pi < \sqrt{\Lambda_n}\,\tau < (n + J)\,\pi
\tag{P21.100}
$$

*Oscillation lemma (classical, sketch).* Use the Pruefer angle $\theta$ of the $\phi$ form, $\phi = \varrho\sin\theta$, $r^2D\phi' = \varrho\cos\theta$. Both $\phi$ and $r^2D\phi'$ are continuous, so $\theta$ is continuous and obeys $\theta' = \cos^2\theta/(r^2D) + \Lambda r^2\sin^2\theta$. Near the centre $\phi = \sin(k_1r)/(k_1r)$ gives $\cot\theta = -D_1k_1^2r^3/3 + O(r^5)$, so $\theta(0^+) = \pi/2$ and $\theta(\varepsilon;\Lambda)$ increases with $\Lambda$ for small $\varepsilon$. The right-hand side is Lipschitz in $\theta$ on $[\varepsilon,R]$ and strictly increasing in $\Lambda$ wherever $\sin\theta \ne 0$, so the comparison theorem for first-order equations makes $\theta(R;\Lambda)$ continuous and strictly increasing in $\Lambda$. At $\Lambda = 0$, $\phi \equiv 1$ and $\theta \equiv \pi/2$. At a zero of $\phi$, $\theta' = 1/(r^2D) > 0$, so $\theta$ crosses multiples of $\pi$ only upwards. The eigenvalues are the solutions of $\theta(R;\Lambda) = m\pi$, $m = 1,2,\dots$, each taken once, so the $n$-th eigenfunction has $\theta(R) = n\pi$ and exactly $n-1$ zeros in $(0,R)$; $u_n = r\phi_n$ has the same zeros.

**PROVED** (the lemma is the classical Sturm oscillation theorem; the sketch above follows the usual Pruefer argument and was not checked against a textbook). Numerically, the transcription finds exactly 200 sign changes of $u(R;\Lambda)$ below $(\Lambda_{200}+\Lambda_{201})/2$ and $n-1$ interior zeros for the modes tested (E6, E7), and $\sqrt{\Lambda_n}\tau/\pi - n$ stays in $[-1.22, 1.22]$ for all 40000 modes of the synthetic case, well inside $\pm J = \pm4$ (B2).

**(c) Combined bound.** With $c = D_\mathrm{min}\pi^2/R^2$ and $a = \pi^2/\tau^2$,

$$
\Lambda_n \ge \ell_n := \max\big(c\,n^2,\ a\,(n-J)_+^2\big)
\tag{P21.101}
$$

**PROVED.** For large $n$ the travel-time part dominates and is asymptotically sharp: $\Lambda_n/\ell_n = 1.0002$ at $n = 40000$ in the synthetic case (B4).

##### 3.5.3.4 Step 3: amplitude lemma

In the kernel $u_n = A\sin(k_1r)$ (regular centre) and in the outer layer $u_n = B\sin(k_L(R - r))$ (absorbing surface), with $k_1 = \sqrt{\Lambda_n/D_1}$, $k_L = \sqrt{\Lambda_n/D_L}$. Direct integration gives

$$
\alpha_n = \int_0^{R_1}r\,u_n\,dr = \frac{A}{k_1}\Big(\frac{\sin k_1R_1}{k_1} - R_1\cos k_1R_1\Big),\qquad \lvert\alpha_n\rvert \le \frac{\lvert A\rvert}{k_1}\sqrt{R_1^2 + k_1^{-2}}
$$

by the Cauchy-Schwarz inequality in $\mathbb R^2$. The flux identity (P21.36) with $\phi_n'(R) = u_n'(R)/R$ (because $u_n(R) = 0$) gives $\beta_n = -RD_Lu_n'(R)/\Lambda_n = RD_LBk_L/\Lambda_n$. The normalisation $\int_0^Ru_n^2 = 1$ over the two disjoint layers gives $A^2a_1 + B^2b_L \le 1$ with

$$
\begin{aligned}
a_1 &= \int_0^{R_1}\sin^2(k_1r)\,dr \ge a_1^- := \frac{R_1}{2} - \frac{1}{4k_1},\\
b_L &= \int_{R_{L-1}}^R\sin^2(k_L(R-r))\,dr \ge b_L^- := \frac{L_L}{2} - \frac{1}{4k_L}
\end{aligned}
$$

so $\lvert A\rvert \le 1/\sqrt{a_1^-}$, $\lvert B\rvert \le 1/\sqrt{b_L^-}$, and by $2\sqrt{a_1b_L}\,\lvert AB\rvert \le A^2a_1 + B^2b_L$ also $\lvert AB\rvert \le 1/(2\sqrt{a_1^-b_L^-})$. For $\Lambda_n > \Lambda^*$:

$$
\begin{aligned}
&\lvert\alpha_n\rvert \le \frac{C_\alpha(\Lambda_n)}{\sqrt{\Lambda_n}},\qquad C_\alpha(\Lambda) = \frac{\sqrt{D_1}\sqrt{R_1^2 + D_1/\Lambda}}{\sqrt{a_1^-}},\\
&\lvert\beta_n\rvert \le \frac{C_\beta(\Lambda_n)}{\sqrt{\Lambda_n}},\qquad C_\beta(\Lambda) = \frac{R\sqrt{D_L}}{\sqrt{b_L^-}}\\
&\lvert w_n\rvert \le \frac{C_w(\Lambda_n)}{\Lambda_n},\qquad C_w(\Lambda) = \frac{R\sqrt{D_1D_L}\,\sqrt{R_1^2 + D_1/\Lambda}}{2V\sqrt{a_1^-\,b_L^-}},\\
&\Lambda^* = \max\Big(\frac{D_1}{4R_1^2},\ \frac{D_L}{4L_L^2}\Big)
\end{aligned}
\tag{P21.102}
$$

with $a_1^-$, $b_L^-$ evaluated at $k_1 = \sqrt{\Lambda/D_1}$, $k_L = \sqrt{\Lambda/D_L}$. **PROVED.** All three constants decrease with $\Lambda$ (each factor does), so for $n > N$ they may be evaluated at $\ell_{N+1}$, or at $\Lambda_{N+1}$ when it is known. The limit is $C_w(\infty) = 3R\sqrt{D_1D_L}/(R_1^{5/2}L_L^{1/2})$.

Check of sharpness. For one layer ($L = 1$) the kernel and outer layer coincide, the factor $1/2$ must be dropped, and $C_w(\infty)/\Lambda_n = 6D/(R^2\Lambda_n) = 6/(n^2\pi^2)$, which is the exact homogeneous weight. In the synthetic case $\lvert w_n\rvert\Lambda_n/C_w(\Lambda_n)$ has maximum 0.69 and median 0.034 over 39999 modes (B7); $\lvert\alpha_n\rvert$ reaches 0.999 of its bound (B5).

##### 3.5.3.5 Step 4: tail sums in closed form

For a decreasing $f$, $\sum_{m>M}f(m) \le \int_M^\infty f(x)\,dx$. Applied to $\ell_n \ge cn^2$ and to $\ell_n \ge a(n-J)^2$ (with $m = n - J$), for $N > J$:

$$
S^{(p)}_N = \min\Big(\frac{1}{(2p-1)\,c^p\,N^{2p-1}},\ \frac{1}{(2p-1)\,a^p\,(N-J)^{2p-1}}\Big) \ge \sum_{n>N}\ell_n^{-p},\qquad p = 1,2,3
\tag{P21.103}
$$

$$
\begin{aligned}
&Y_N(s) = \min\Big(\frac{\Phi(cs,N)}{c},\ \frac{\Phi(as,N-J)}{a}\Big) \ge \sum_{n>N}\frac{e^{-\ell_ns}}{\ell_n},\\
&\Phi(b,M) = \int_M^\infty\frac{e^{-bx^2}}{x^2}\,dx = \frac{e^{-bM^2}}{M} - \sqrt{\pi b}\,\operatorname{erfc}(\sqrt b\,M)
\end{aligned}
\tag{P21.104}
$$

**Exact** for $\Phi$ (symbolic check S3), with $\Phi(0,M) = 1/M$. The second form cancels badly for large $bM^2$; the check script evaluates it with 60 digits, and a code should use $\Phi = e^{-bM^2}\big(1/M - \sqrt{\pi b}\,\operatorname{erfcx}(\sqrt bM)\big)$ with an asymptotic series for large $\sqrt bM$. For $n > N$ both $\Lambda_n \ge \ell_n$ and (under T4) $\Lambda_n \ge \Lambda_{N+1}$ hold, and $x \mapsto e^{-sx}/x$ decreases, so $e^{-\Lambda_nt}/\Lambda_n \le e^{-\theta\Lambda_{N+1}t}\,e^{-(1-\theta)\ell_nt}/\ell_n$ for every $\theta\in[0,1]$:

$$
\sum_{n>N}\frac{e^{-\Lambda_nt}}{\Lambda_n} \le X_N(t) := \min_{\theta\in[0,1]}e^{-\theta\Lambda_{N+1}t}\,Y_N\big((1-\theta)\,t\big)
\tag{P21.105}
$$

**PROVED.** $\theta = 0$ needs only the inputs; $\theta = 1$ gives $e^{-\Lambda_{N+1}t}S^{(1)}_N$, which has the correct exponential rate at late times. Any proved lower bound $\Lambda_\mathrm{cut} \le \Lambda_{N+1}$ may replace $\Lambda_{N+1}$, for example the v1.0 `lambda_covered` when every mode below it is kept. The script minimises over $\theta \in \{0, 0.5, 0.9, 1\}$.

##### 3.5.3.6 Step 5: main result (Bound A) and corollaries

**Theorem (Bound A).** Under T1 to T3, for $N > J$ with $\ell_{N+1} > \Lambda^*$ and all $t \ge 0$:

$$
\big\lvert M^{(N)}_\mathrm{rel}(t) - M_\mathrm{rel}(t)\big\rvert \le C_w(\ell_{N+1})\Big[M_0\,X_N(t) + \big(G_\mathrm{max} + \lambda\,(M_0 + M_\mathrm{gen}(t))\big)\,S^{(2)}_N\Big]
\tag{P21.106}
$$

*Proof.* Insert (P21.102) into (P21.98): $\lvert w_n\rvert e^{-\Lambda_nt} \le C_w(\ell_{N+1})e^{-\Lambda_nt}/\Lambda_n$ and $\lvert w_n\rvert/\Lambda_n \le C_w(\ell_{N+1})/\Lambda_n^2 \le C_w(\ell_{N+1})/\ell_n^2$, then sum with (P21.105) and (P21.103). With $\theta = 0$ in $X_N$ only T1 to T3 are used; with $\theta > 0$ also T4. **PROVED; DEMONSTRATED NUMERICALLY** (T1, T3, T5, T8).

Corollaries, by the same insertion:

$$
\big\lvert M(t) - M^{(N)}(t)\big\rvert \le C_w(\ell_{N+1})\Big[M_0X_N(t) + G_\mathrm{max}S^{(2)}_N\Big],\qquad \lvert\Delta F_B\rvert \le \frac{\lvert M - M^{(N)}\rvert}{M + M_\mathrm{out}}
\tag{P21.107}
$$

$$
\Big\lvert\frac{J_R}{G}\Big\vert^{(N)}_\mathrm{steady} - \frac{J_R}{G}\Big\vert_\mathrm{steady}\Big\rvert = \lambda\,\Big\lvert\sum_{n>N}\frac{w_n}{\kappa_n}\Big\rvert \le \lambda\,C_w(\ell_{N+1})\,S^{(2)}_N
\tag{P21.108}
$$

The denominator of $F_B$ is exact by (P21.44). For the concentration itself, with $c_0$ and $s_\mathrm{max}$ the kernel start concentration and the largest kernel source density, $\hat a_n = \alpha_n\nu_n$ with $\lvert\nu_n\rvert \le c_0e^{-\Lambda_nt} + s_\mathrm{max}/\Lambda_n$, and Parseval's identity gives

$$
\big\lVert c - c^{(N)}\big\rVert^2_{L^2(r^2dr)} = \sum_{n>N}\alpha_n^2\nu_n^2 \le 2\,C_\alpha(\ell_{N+1})^2\Big[c_0^2\,X_N(2t) + s_\mathrm{max}^2\,S^{(3)}_N\Big]
\tag{P21.109}
$$

with the factor 2 dropped when $s_\mathrm{max} = 0$. **PROVED**; DEMONSTRATED NUMERICALLY for $s_\mathrm{max} = 0$ only (T11, including $t = 0$ where the left side is the exact Bessel defect).

**Mode-count rule.** Since $S^{(2)}_N \le \tau^4/(3\pi^4(N-J)^3)$, the source and decay part of (P21.106) is at most $C_w(\ell_{N+1})K\tau^4/(3\pi^4(N-J)^3)$. For a tolerance $\varepsilon$ on this source and decay part (or on the steady $R/B$, with $\lambda$ in place of $K$),

$$
N \ge J + \Big(\frac{C_w(\ell_{N+1})\,K\,\tau^4}{3\pi^4\,\varepsilon}\Big)^{1/3}
\tag{P21.110}
$$

is sufficient, together with the standing conditions $N > J$ and $\ell_{N+1} > \Lambda^*$ of the theorem, solved by one or two fixed-point steps from $C_w(\infty)$, or by scanning $N$ as the script does. **PROVED** (direct consequence of (P21.106)). The rule does not control the start-inventory part $C_w(\ell_{N+1})M_0X_N(t)$ of (P21.106). For a tolerance on the release itself, $N$ must also satisfy $C_w(\ell_{N+1})M_0X_N(t_1) \le \varepsilon$ at the earliest output time $t_1 > 0$. At $t = 0$ the start-inventory part has no exponential factor and falls only like $1/N$ ($X_N(0) = S^{(1)}_N$), so it is the start-inventory part, and not the source part, that fixes $N$ for early output: the balance-form release error at $t = 0$ is exactly $M_0\big(1 - \sum_{n\le N}w_n\big)$, which is $2.8\times10^{-2}M_0$ in the synthetic case at the $N = 5$ that the rule returns for $K = 0$ (verification record in Appendix B.5.2, M1). In the synthetic case with $\lambda = \Lambda_1$ the rule gives $N = 92$ for $\varepsilon = 10^{-6}$ on the steady $R/B$ (true error $5.2\times10^{-12}$) and $N = 873$ for $\varepsilon = 10^{-9}$ (true error $1.1\times10^{-13}$) (T6b).

##### 3.5.3.7 Step 6: a posteriori bound (Bound B)

For $f \in L^2(r^2dr)$ let $F_f(x) = \int_0^xf(s)s^2\,ds$. The solution of $-x^{-2}(x^2Dv')' = f$ with $v(R) = 0$ and a regular centre has $x^2Dv' = -F_f$, so for two such functions, integrating by parts once,

$$
\sum_n\frac{\langle f,\hat\phi_n\rangle\langle g,\hat\phi_n\rangle}{\Lambda_n} = \langle g,\mathcal A^{-1}f\rangle = \int_0^R\frac{F_f(x)\,F_g(x)}{x^2D(x)}\,dx
$$

where the first equality uses T3. With $F_1 = x^3/3$ and $F_{\chi_1} = \min(x,R_1)^3/3$, layer by layer,

$$
\begin{aligned}
Q_{\beta\beta} &= \sum_n\frac{\beta_n^2}{\Lambda_n} = \frac{1}{45}\sum_{i=1}^L\frac{R_i^5 - R_{i-1}^5}{D_i},\\
Q_{\alpha\alpha} &= \sum_n\frac{\alpha_n^2}{\Lambda_n} = \frac{R_1^5}{45D_1} + \frac{R_1^6}{9}\sum_{i=2}^L\frac{1}{D_i}\Big(\frac{1}{R_{i-1}} - \frac{1}{R_i}\Big)\\
Q_{\alpha\beta} &= \sum_n\frac{\alpha_n\beta_n}{\Lambda_n} = \frac{R_1^5}{45D_1} + \frac{R_1^3}{18}\sum_{i=2}^L\frac{R_i^2 - R_{i-1}^2}{D_i},\\
&\sum_n\alpha_n^2 = V,\qquad \sum_n\beta_n^2 = \frac{R^3}{3}
\end{aligned}
\tag{P21.111}
$$

**Exact** (symbolic check S2 for one layer, quadrature check G1 for five layers). $Q_{\alpha\beta}/V = \sum_nw_n/\Lambda_n$ is the steady inventory per unit kernel source for $\lambda = 0$. Write the four Bessel defects $A_N = Q_{\alpha\alpha} - \sum_{n\le N}\alpha_n^2/\Lambda_n$, $B_N = Q_{\beta\beta} - \sum_{n\le N}\beta_n^2/\Lambda_n$, $a_N = V - \sum_{n\le N}\alpha_n^2$, $b_N = R^3/3 - \sum_{n\le N}\beta_n^2$, all non-negative. The Cauchy-Schwarz inequality on $\sum_{n>N}\lvert\alpha_n\beta_n\rvert g_n$ with $g_n = 1/\Lambda_n$ or $g_n = e^{-\Lambda_nt}$ gives

$$
\begin{aligned}
&T_1 \le \frac{\sqrt{A_NB_N}}{V},\\
&T_0 \le \frac{1}{V}\min\Big(e^{-\Lambda_{N+1}t}\sqrt{a_Nb_N},\ \sigma(\Lambda_{N+1},t)\sqrt{A_NB_N}\Big),\\
&\sigma(\Lambda',t) = \sup_{\Lambda\ge \Lambda'}\Lambda e^{-\Lambda t}\\
&\big\lvert M^{(N)}_\mathrm{rel} - M_\mathrm{rel}\big\rvert \le M_0\,T_0 + K(t)\,T_1
\end{aligned}
\tag{P21.112}
$$

with $\sigma(\Lambda',t) = \Lambda'e^{-\Lambda't}$ for $\Lambda't \ge 1$ and $1/(et)$ otherwise. **PROVED** (under T1 to T5); **DEMONSTRATED NUMERICALLY** (T2, T4, T5, T10). In Table 3.7 Bound B is 2.6 to 20 times smaller than Bound A for $N \le 300$; at late times the cut-off form of Bound A can be the sharper of the two (S1, $N = 300$, $t = 305.5\ \mathrm{s}$ in the script output, Appendix A.2). Two properties matter for the code. If a mode below $\Lambda_{N+1}$ were skipped, $A_N$, $B_N$ would include it and the $T_1$ part would stay valid, while the exponential part would not (T4). The defects are computed by subtraction, so Bound B is limited by round-off once $A_N$ approaches $10^{-13}Q_{\alpha\alpha}$; the script uses it only while $A_N, B_N > 10^{-9}$ of their totals, which is $N \le 300$ in both cases.

Bound B extends to any start profile $c_0(r)$ or source shape: replace $\chi_1$ by that profile in $\alpha_n$, $Q_{\alpha\alpha}$ and $a_N$, with $Q_{ff} = \int_0^RF_f^2/(x^2D)\,dx$. **PROVED by the same argument; not demonstrated numerically.** Bound A does not extend this way because the amplitude lemma uses the exact kernel form of $u_n$.

##### 3.5.3.8 Numerical demonstration

The check script transcribes the layered shooting method (exact transfer in each layer, bisection on $\psi(R) = n\pi$ inside the min-max bracket (P21.99), closed-form layer integrals) and reproduces the v1.0 golden value $F(2\times10^5\ \mathrm{s}) = 0.4325556390928$ to $3\times10^{-13}$ with 40000 modes (E8), the 60-digit mpmath roots to $1.6\times10^{-15}$ (E5), and the flux identity (P21.36) to $1.2\times10^{-12}$ (E3). The reference for each truncation error is the same series with $N_\mathrm{ref} = 40000$ modes (20000 for the check case), whose own uncertainty is Bound A at $N_\mathrm{ref}$; for $\lambda = 0$ the steady inventory has the exact reference $Q_{\alpha\beta}/V$ of (P21.111). Every comparison holds in both cases (91 PASS lines).

Synthetic case (`inputs/synthetic/synthetic_five_layer.toml`): $R_i = 200, 300, 350, 400, 450\ \mu\mathrm{m}$, $D_i = 10^{-12}, 4\times10^{-10}, 8\times10^{-13}, 2\times10^{-14}, 7\times10^{-13}\ \mathrm{m^2\ s^{-1}}$. Constants: $\tau = 674.217\ \mathrm{s^{1/2}}$, $a = 2.17121\times10^{-5}\ \mathrm{s^{-1}}$, $c = 9.74776\times10^{-7}\ \mathrm{s^{-1}}$, $\Lambda^* = 7.0\times10^{-5}\ \mathrm{s^{-1}}$, $C_w(\infty) = 2.8237\times10^{-4}\ \mathrm{s^{-1}}$, $C_w(\ell_{11}) = 3.4478\times10^{-4}\ \mathrm{s^{-1}}$, $\Lambda_1 = 3.27326\times10^{-6}\ \mathrm{s^{-1}}$; 151 negative weights among the first 300 modes. Scenarios: S1 no source, no decay, $M_0 = 1\ \mathrm{mol}$; S2 steady $R/B$ for a kernel source; S3 $M_0 = 1\ \mathrm{mol}$, $G = \Lambda_1\ \mathrm{mol\ s^{-1}}$ on $[0, 1/\Lambda_1)$, then post-irradiation; S4 $\lambda = 0$ steady inventory per unit source.

| Quantity | $N$ | True error | Bound A | Bound B | error / A |
|------------------------------|-------|-----------------|----------------|----------------|--------------|
| S4, $\sum_{n>N}w_n/\Lambda_n$ ($\mathrm{s}$) | 10 | $2.24$ | $1.13\times10^{3}$ | $5.65\times10^{1}$ | $2.0\times10^{-3}$ |
| | 50 | $2.38\times10^{-2}$ | $2.10$ | $5.22\times10^{-1}$ | $1.1\times10^{-2}$ |
| | 100 | $3.40\times10^{-4}$ | $2.28\times10^{-1}$ | $6.17\times10^{-2}$ | $1.5\times10^{-3}$ |
| | 300 | $5.12\times10^{-6}$ | $7.73\times10^{-3}$ | $2.34\times10^{-3}$ | $6.6\times10^{-4}$ |
| | 1000 | $2.0\times10^{-9}$ (i) | $2.02\times10^{-4}$ | not used | $1\times10^{-5}$ (i) |
| S2, steady $R/B$, $\lambda = \Lambda_1$ | 10 | $7.31\times10^{-6}$ | $3.69\times10^{-3}$ | $1.85\times10^{-4}$ | $2.0\times10^{-3}$ |
| | 50 | $7.78\times10^{-8}$ | $6.89\times10^{-6}$ | $1.71\times10^{-6}$ | $1.1\times10^{-2}$ |
| | 300 | $1.68\times10^{-11}$ | $2.53\times10^{-8}$ | $7.65\times10^{-9}$ | $6.6\times10^{-4}$ |
| S3, $M_\mathrm{rel}$ ($\mathrm{mol}$), $\lambda = \Lambda_1$, $t = 4.583\times10^5\ \mathrm{s}$ | 10 | $1.46\times10^{-5}$ | $1.11\times10^{-2}$ | $5.54\times10^{-4}$ | $1.3\times10^{-3}$ |
| | 50 | $1.56\times10^{-7}$ | $2.07\times10^{-5}$ | $5.12\times10^{-6}$ | $7.5\times10^{-3}$ |
| | 300 | $3.35\times10^{-11}$ | $7.59\times10^{-8}$ | $2.30\times10^{-8}$ | $4.4\times10^{-4}$ |
| S1, $F(t)$, $t = 305.5\ \mathrm{s}$ | 10 | $2.53\times10^{-3}$ | $9.41\times10^{-1}$ | $6.80\times10^{-2}$ | $2.7\times10^{-3}$ |
| | 50 | $5.53\times10^{-11}$ | $3.14\times10^{-9}$ | $1.19\times10^{-9}$ | $1.8\times10^{-2}$ |

Table: 3.7 Synthetic five-layer case. (i) below the reference uncertainty $3.1\times10^{-9}\ \mathrm{s}$, indicative only. Bound B is not used above $N = 300$ (round-off, Section 3.5.3.7).

Over all tested $N$ (5 to 5000) and times ($10^{-4}/\Lambda_1$ to $3/\Lambda_1$, and $2\times10^5\ \mathrm{s}$), the largest ratio error/Bound A is $1.8\times10^{-2}$ (S1), $1.1\times10^{-2}$ (S2, S3, S4); the largest ratio error/Bound B is 0.26. The check case of the decay-part script ($R_i = 250, 350, 390, 425, 465\ \mu\mathrm{m}$, $D_i = 4\times10^{-14}, 4\times10^{-13}, 2\times10^{-14}, 5\times10^{-15}, 2\times10^{-14}\ \mathrm{m^2\ s^{-1}}$, 152 negative weights in 300 modes, confirmed) gives largest ratios $3.0\times10^{-2}$ (A) and 0.29 (B). The observed decay of the true S4 error over $50 \le N \le 300$ is irregular (fitted slopes 5.3 and 2.8 in the two cases) because of the sign changes, while Bound A falls with slope 3.1.

**What the demonstration shows.** The bound always holds in the tested range and has the predicted $N^{-3}$ slope. Bound A is pessimistic by a factor of at least 33, with median ratios error/Bound A between $5\times10^{-5}$ and $3\times10^{-3}$ over the scenarios of both cases, because the absolute values in (P21.98) discard the cancellation between weights of opposite sign and because $C_w$ assumes the worst phase in every mode (median $\lvert w_n\rvert\Lambda_n/C_w = 0.034$).

##### 3.5.3.9 Status and limitations

| Result | Statement | Label |
|---|---|---|
| (P21.95), (P21.96) | exact error representation of the balance form | exact; PROVED given T3 |
| (P21.99) | min-max bounds with $D_\mathrm{min}$, $D_\mathrm{max}$ | PROVED (classical); DEMONSTRATED NUMERICALLY (B1) |
| (P21.100), (P21.101) | travel-time bound $\lvert\sqrt{\Lambda_n}\tau - n\pi\rvert < J\pi$ | PROVED (Pruefer argument, oscillation lemma sketched); DEMONSTRATED NUMERICALLY (B2, B3, E6, E7) |
| (P21.102) | amplitude lemma, $\lvert w_n\rvert \le C_w/\Lambda_n$ | PROVED; DEMONSTRATED NUMERICALLY (B5 to B8) |
| (P21.103) to (P21.105) | closed-form tail sums | PROVED; $\Phi$ exact (S3) |
| (P21.106) | Bound A on the balance-form release | PROVED; DEMONSTRATED NUMERICALLY (T1, T3, T5, T8) |
| (P21.107), (P21.108) | inventory, $F_B$ and steady $R/B$ | PROVED; DEMONSTRATED NUMERICALLY (T5, T9) |
| (P21.109) | $L^2(r^2dr)$ profile error | PROVED; DEMONSTRATED NUMERICALLY without source (T11) |
| (P21.110) | mode-count rule for the source and decay part (the start-inventory part needs its own condition) | PROVED; DEMONSTRATED NUMERICALLY (T6b) |
| (P21.111), (P21.112) | Green identities and Bound B | exact and PROVED; DEMONSTRATED NUMERICALLY (G1 to G7, T2, T4, T10) |
| Bound B for other profiles | any $c_0(r)$ or source shape | PROVED; not demonstrated numerically |

Table: 3.8 Status of the results of Section 3.5.3.

Limitations.

- The bounds rest on completeness (T3), which rests on the spectral theorem for compact self-adjoint operators (read in Melrose, 2009, Lecture 20, Theorem 15, p. 116) and on the compactness of the Green operator, for which no citation has been opened (O14, updated 2026-11-13). The min-max principle and the oscillation theorem are classical and were also not checked against a textbook.
- Bound A covers constant diffusivities, an absorbing surface, and a kernel-uniform start and source (block R1). After a projection at a temperature switch (step R3.5) the start profile is not kernel-uniform, so only Bound B in its general form applies there, and it needs the moments of the projected profile.
- No cancellation between weights of opposite sign is used, so both bounds are conservative. A bound that exploits the sign pattern is OPEN.
- $\ell_{N+1} > \Lambda^*$ is required. In the synthetic case this holds from $N = 5$, and $C_w(\ell_{N+1})$ is large only for the smallest $N$. The condition can be restrictive when the kernel or the outer layer is thin and fast: for eight alternating layers with $D = 10^{-9}$ and $10^{-16}\ \mathrm{m^2\ s^{-1}}$ (case X5 of the verification record in Appendix B.5.2) $\Lambda^* = 0.1\ \mathrm{s^{-1}}$ and Bound A applies only from $N = 2020$.
- Bound B loses meaning when its defects reach round-off, about $N > 300$ at the threshold used here; Bound A has no such limit beyond the evaluation of $\Phi$ (P21.104).
- The bounds treat truncation only. Errors of the computed $\Lambda_n$, $w_n$ (root finding, quadrature, the cancellation measured by `min_flux_ratio` in v1.0) are separate. In the binary64 transcription the eigenvalues agree with a 45-digit recomputation to $7.5\times10^{-14}$ relative, but the weights of 5331 of the first 20000 synthetic modes differ from it by more than $10^{-12}$ relative, by up to $2.1\times10^{-8}$ (forward shooting is ill-conditioned for modes concentrated in inner layers; check E3 does not see this because both of its $\beta_n$ come from the same shot). The effect on the tabulated truncation errors is negligible, since these modes have $\lvert w_n\rvert/\Lambda_n$ far below the tails compared.

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
&\text{(I) source-free heating:}\\
&\qquad F_\mathrm{rel} = \frac{M_\mathrm{rel}}{M_0},\quad F_\mathrm{corr} = \frac{e^{\lambda t}M_\mathrm{out}}{M_0}\\
&\qquad\qquad = \frac{1}{M_0}\int_0^tJ_R(s)e^{\lambda s}ds,\\
&\qquad \text{and } F_\mathrm{rel}(\infty) = \sum_n w_n\frac{\Lambda_n}{\Lambda_n+\lambda}\ (\text{kernel-uniform } c_0)\\
&\text{(II) irradiation from zero:}\\
&\qquad F_\mathrm{gen} = \frac{M_\mathrm{rel}}{M_\mathrm{gen}},\quad \frac{R}{B}(t) = \frac{J_R(t)}{G(t)},\quad F_B = \frac{M_\mathrm{out}}{M + M_\mathrm{out}}\\
&\text{(III) heating after irradiation:}\\
&\qquad F_\mathrm{post}(t) = \frac{M_\mathrm{rel}(t) - M_\mathrm{rel}(t_\mathrm{irr})}{M(t_\mathrm{irr})}
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
\begin{aligned}
\frac{R}{B}(t) &= \sum_{n}\frac{6}{n^2\pi^2 + x} - 6e^{-\lambda t}\sum_{n}\frac{e^{-n^2\pi^2D't}}{n^2\pi^2 + x},\\
F_B(t) &= \sum_n\frac{6}{n^2\pi^2+x} - \frac{6}{\pi^2}\frac{x}{e^{\lambda t}-1}\sum_n\frac{1 - e^{-n^2\pi^2D't}}{n^2(n^2\pi^2 + x)}
\end{aligned}
\tag{P21.48}
$$

These are TECDOC-1674 Eqs (7.7) and (7.8) (IAEA, 2012, p. 319) once $\sum_n 6/(n^2\pi^2+x)$ is replaced by its closed form below. The derivation identifies what (7.8) measures: the fraction of the **surviving** produced atoms that are outside the particle, $F_B = M_\mathrm{out}/(M + M_\mathrm{out})$. Multiplying numerator and denominator by $e^{\lambda t}$ gives the equivalent form $F_B = \int_0^tJ_Re^{\lambda s}ds\big/\int_0^tG\,e^{\lambda s}ds$: decay-corrected release divided by decay-corrected production. It differs from $M_\mathrm{rel}/M_\mathrm{gen}$ whenever $\lambda > 0$, and it differs from $F_\mathrm{corr}$ of scenario (I), whose denominator is $M_0$.

#### 3.7.4 Steady-state $R/B$ with decay

As $t\to\infty$ in (P21.47):

$$
\begin{aligned}
\left(\frac{R}{B}\right)_\mathrm{ss} &= \sum_nw_n\frac{\Lambda_n}{\Lambda_n + \lambda} = \int_0^\infty\left(-\frac{d}{dt}\sum_nw_ne^{-\Lambda_nt}\right)e^{-\lambda t}\,dt\\
&\xrightarrow{\text{one sphere}}\ 3\sqrt{\frac{1}{x}}\left(\coth\sqrt{x} - \sqrt{\frac{1}{x}}\right)
\end{aligned}
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
| (P21.93) to (P21.112), Section 3.5.3 (added 2026-11-13) | (P21.95), (P21.96) exact error representation; Bound A (P21.106), its corollaries (P21.107) to (P21.110) and Bound B (P21.112) PROVED, conditional on completeness (Section 3.10) and on the classical min-max and Sturm oscillation theorems, and DEMONSTRATED NUMERICALLY; both bounds are conservative because they use no cancellation between weights of opposite sign; labels per equation in Table 3.8 |

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
\begin{aligned}
\frac{V_j}{\Delta t}\left(C_j^{n+1} - C_j^n\right) &= -\sum_fG_f\,\Delta_fC^{n+\theta} - \lambda V_jC_j^{n+\theta} + V_jS_j^{n+1/2},\\
C^{n+\theta} &= \theta C^{n+1} + (1-\theta)C^n
\end{aligned}
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

**`sec3_o4_layered_truncation_bound.py`** (added 2026-11-13; Section 3.5.3, (P21.93) to (P21.112)). Symbolic layer integrals and Green identities, then a Python transcription of the layered shooting method checked against the v1.0 golden value, 60-digit roots and the flux identity (P21.36), and Bound A and Bound B against 40000-mode references on the synthetic five-layer case and on the check case with 152 negative weights in 300 modes: `91 PASS` and `ALL PASS`. The full output is in Appendix A.2 and the independent verification in Appendix B.5.2.

**Further verification opportunities for the Rust v2.0 code.** (i) Reproduce the Booth table above with STRATO B and C in one-layer mode; (ii) run the two-layer case above in both codes and require second-order convergence of the Rust FV with Crank-Nicolson, or first order in time with Backward Euler; (iii) assert (P21.42) every step; (iv) assert $F_\mathrm{corr}(t)$ of a source-free run equals the v1.0 $F(t)$ for any $\lambda$; (v) compare $(R/B)_\mathrm{ss}$ from (P21.49) with a long FV run.

### 3.10 Open issues

- Capture cross sections, the Ag-110m isomeric fraction, product burn-out cross sections and flux histories are NOT VERIFIED (Section 3.3.3); activation products cannot be run quantitatively until they are sourced.
- Kernel heavy-metal density for (P21.9) is not taken from a verified source in this section.
- Diffusivities of daughter elements (Y, Ba) in each layer are not in the audited data; chains stay future work.
- Completeness of the layered eigenfunction set (assumed in Section 4.5 of the Phase 1 derivation, `docs/mathematics/phase1/TRISO_Finite_Volume_Derivation.md`) follows from a Green-function argument added in the B3 reconciliation. For $-r^{-2}(r^2Dc')' = f$ with $c(R) = 0$ and a regular centre, integrating twice gives $c(r) = \int_0^R G(r,s)f(s)s^2\,ds$ with $G(r,s) = g(\max(r,s))$ and $g(x) = \int_x^R d\rho/(\rho^2D(\rho))$. This kernel is symmetric, the operator is injective, and $\int_0^R\int_0^R G^2r^2s^2\,dr\,ds = \tfrac23\int_0^R g(x)^2x^5\,dx$ is finite because $g(x) \le C/x$ near the centre, so the operator is compact and self-adjoint on $L^2(r^2dr)$. Its eigenfunctions, which are those of (P21.24) with $\Lambda_n$ the reciprocal eigenvalues, therefore form a complete orthogonal set: the spectral theorem for compact self-adjoint operators (the Hilbert-Schmidt theorem) gives an orthonormal basis of eigenfunctions of the orthogonal complement of the null space, which is the whole space because the operator is injective (Melrose, 2009, Lecture 20, Theorem 15, p. 116). The step from the finite double integral of $G^2$ to compactness is not cited (NOT VERIFIED against a source). The script `b3_completeness.py` confirms the Green function against the exact profile (P21.82) in exact arithmetic and evaluates the Hilbert-Schmidt norm for five layers. Numerical support: $\sum w_n \to 1$, the projection of a smooth $c_0$ reproducing $M_0$ to $3\times10^{-9}$, and second-order FV agreement.
- Truncation of the modal series in the layered particle (open item O4, updated 2026-11-13). For a kernel-uniform start and source, Bound A (P21.106) and Bound B (P21.112) of Section 3.5.3 bound the error of the balance form explicitly (PROVED under the completeness argument of the previous bullet and the classical min-max and Sturm oscillation theorems; DEMONSTRATED NUMERICALLY with no counterexample in the verification of Appendix B.5.2). Still OPEN: a bound that uses the cancellation between weights of opposite sign, which is why both bounds are pessimistic (largest observed error over Bound A $3.03\times10^{-2}$, over Bound B 0.517). Bound A needs $\ell_{N+1} > \Lambda^*$, which can force many modes (smallest admissible $N$ = 2020 for eight alternating layers with $D = 10^{-9}$ and $10^{-16}\ \mathrm{m^2\ s^{-1}}$, 9550 in the most extreme verification case); Bound B has no such threshold. After a projection at a temperature switch only the general form of Bound B applies. With $S = 0$, Section 5.3.7 adds computable tail bounds that cover the release quadrature with decay.

- Fission-fragment recoil into the buffer and noble-gas precursor migration are neglected (A5, Section 3.3.2).
- Which decay constants the CRP-6 participants used is NOT VERIFIED (audit 04, Section 2.4).

## 4 Part 2.1, finite volume: formulation with decay and source, and its verification design

This section extends Code B of STRATO v1.0 (`src/numerical/finite_volume.rs`) to a single nuclide $k$ with radioactive decay and a volumetric source. It fixes the discrete equations, states exactly which entries of the v1.0 tridiagonal system change, proves that positivity, the subtraction-free elimination and inventory closure survive, compares the implicit scheme with a decay-exact alternative, gives step-size rules, and designs the verification matrix. Every derived formula below was checked by one of six scripts in `scripts/derivation_checks/` (Section 4.9). The Phase 1 equations are cited by the labels of the delivered derivation `docs/mathematics/phase1/TRISO_Finite_Volume_Derivation.md` (for example TRISO-DEL-040). Equations of this section are numbered (P21.50) to (P21.92), and (P21.113) to (P21.142) in Section 4.6.1.

Status labels used: **exact** (an identity, no truncation), **semi-analytical** (exact up to a finite linear solve or root find), **series** (exact infinite sum, truncated in practice), **approximate** (a stated modelling or discretisation error remains), **numerical** (a computed number).

### 4.0 Symbols and units

| Symbol | Meaning | Units |
|---|---|---|
| $r$, $t$ | radius, time | $\mathrm{m}$, $\mathrm{s}$ |
| $i=1,\dots,5$ | layer index: kernel, buffer, IPyC, SiC, OPyC | 1 |
| $R_i$ | outer radius of layer $i$; $R_0=0$, $R_5=R$ | $\mathrm{m}$ |
| $D_i$ | diffusivity of layer $i$ (constant in this section) | $\mathrm{m^2\ s^{-1}}$ |
| $c(r,t)$ | concentration of nuclide $k$ | $\mathrm{mol\ m^{-3}}$ |
| $\lambda_k$, written $\lambda$ | decay constant, $\lambda=\ln 2/t_{1/2}$ | $\mathrm{s^{-1}}$ |
| $S(r,t)$ | volumetric source | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $S_K(t)$ | uniform kernel source value | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $\dot F$ | fission rate density | $\mathrm{fissions\ m^{-3}\ s^{-1}}$ |
| $y^I$, $y^C$ | independent and cumulative fission yield | $\mathrm{per\ fission}$ |
| $N_A$ | Avogadro constant | $\mathrm{mol^{-1}}$ |
| $j=1,\dots,N$ | cell index, cell $j$ spans $[r_{j-1/2},r_{j+1/2}]$ | 1 |
| $r_j$ | volume centroid of cell $j$, $\tfrac34(r_{j+1/2}^4-r_{j-1/2}^4)/(r_{j+1/2}^3-r_{j-1/2}^3)$ (`volume_centroid` in v1.0) | $\mathrm{m}$ |
| $V_K$ | kernel volume $\tfrac{4\pi}{3}R_1^3$ | $\mathrm{m^3}$ |
| $V_j$ | shell volume $\tfrac{4\pi}{3}(r_{j+1/2}^3-r_{j-1/2}^3)$ | $\mathrm{m^3}$ |
| $A_f$ | face area $4\pi r_f^2$ | $\mathrm{m^2}$ |
| $G_f$; $G_W$, $G_E$ | face conductance (TRISO-FV-140); west and east face of a cell | $\mathrm{m^3\ s^{-1}}$ |
| $G_R$ | surface conductance $4\pi R^2D_5/(R-r_N)$ (TRISO-DEL-030) | $\mathrm{m^3\ s^{-1}}$ |
| $g^W_j$, $g^E_j$ | $G_W,\ G_E$ of cell $j$; $g^W_1=0$, $g^E_N=G_R$, $g^E_j=g^W_{j+1}$ | $\mathrm{m^3\ s^{-1}}$ |
| $C_j^n$ | cell-average concentration at $t^n$ | $\mathrm{mol\ m^{-3}}$ |
| $\bar S_j(t)$, $\bar S_j^{n+1}$ | cell-averaged source; its value used in step $n\to n+1$ | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $\Delta t$, $z=\lambda\Delta t$ | time step; dimensionless decay per step | $\mathrm{s}$, 1 |
| $v_j$ | diagonal excess of row $j$ (Section 4.3) | $\mathrm{m^3\ s^{-1}}$ |
| $\mathbf V$, $\mathbf K$ | $\mathrm{diag}(V_j)$; conductance (stiffness) matrix with row sums $0$ except $G_R$ in row $N$ | $\mathrm{m^3}$, $\mathrm{m^3\ s^{-1}}$ |
| $M$, $M_0$ | inventory inside the particle, initial inventory | $\mathrm{mol}$ |
| $M_{gen}$, $M_{rel}$, $M_{dec}$, $M_{out}$ | generated, released through $r=R$, decayed inside, released and not yet decayed outside (cumulative) | $\mathrm{mol}$ |
| $M_{dec,out}$ | released atoms that decayed outside | $\mathrm{mol}$ |
| $\mathrm{Gen}(t)$ | generation rate $\int S\,dV$ | $\mathrm{mol\ s^{-1}}$ |
| $J_R(t)$ | release rate $-4\pi R^2D_5\,\partial_rc\vert_{R}$ | $\mathrm{mol\ s^{-1}}$ |
| $\Lambda_n$, $\phi_n$, $N_n$, $w_n$ | Phase 1 eigenvalues, eigenfunctions, norms $\int_0^Rr^2\phi_n^2dr$, release weights (TRISO-DEL-021) | $\mathrm{s^{-1}}$, $\mathrm{mol\ m^{-3}}$, $\mathrm{m^3\,(mol\ m^{-3})^2}$, 1 |
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
\begin{gathered}
\partial_rc_1(0,t)=0,\quad c_i(R_i,t)=c_{i+1}(R_i,t),\quad D_i\partial_rc_i\big\vert_{R_i^-}=D_{i+1}\partial_rc_{i+1}\big\vert_{R_i^+},\\
c_5(R,t)=0,\quad c(r,0)=c^0(r)
\end{gathered}\tag{P21.51}
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
\begin{aligned}
&V_j\frac{dC_j}{dt}=G_W(C_{j-1}-C_j)+G_E(C_{j+1}-C_j)-\lambda V_jC_j+V_j\bar S_j(t),\\
&\text{in matrix form}\quad \mathbf V\dot{\mathbf C}=-\mathbf K\mathbf C-\lambda\mathbf V\mathbf C+\mathbf V\bar{\mathbf S}(t).
\end{aligned} \tag{P21.55}
$$

Integrating (P21.55) over one step is still exact:

$$
\begin{aligned}
V_j\big[C_j(t^{n+1})-C_j(t^n)\big]&=\int_{t^n}^{t^{n+1}}\!\Big[G_W(C_{j-1}-C_j)+G_E(C_{j+1}-C_j)-\lambda V_jC_j\Big]dt\\
&\quad+V_j\int_{t^n}^{t^{n+1}}\bar S_j\,dt.
\end{aligned} \tag{P21.56}
$$

### 4.3 (b) Backward Euler with implicit decay

**Time quadrature.** Backward Euler (BE) replaces the first integral of (P21.56) by $\Delta t$ times its value at $t^{n+1}$ (first order). For the source integral two choices exist:

$$
\begin{aligned}
&\text{(R) rectangle: }\ \bar S_j^{n+1}=\bar S_j(t^{n+1-}),\\
&\text{(A) exact step average: }\ \bar S_j^{n+1}=\frac{1}{\Delta t}\int_{t^n}^{t^{n+1}}\bar S_j(t)\,dt.
\end{aligned} \tag{P21.57}
$$

In (R) the value is the left limit at $t^{n+1}$, so a source defined on half-open intervals $(t_a,t_b]$ is sampled inside the step. For a smooth source the generation error of (R) per step is $\int\bar S\,dt-\Delta t\,\bar S(t^{n+1})=-\tfrac12\Delta t^2\dot{\bar S}-\tfrac13\Delta t^3\ddot{\bar S}+O(\Delta t^4)$ with the derivatives taken at $t^n$ (X7; expanded about $t^{n+1}$ the cubic coefficient is $+\tfrac16$ instead), which sums to $O(\Delta t)$ over a run. (A) has no generation error at all; for a piecewise-constant or piecewise-linear $\dot F(t)$ it is a closed form. Both give a scheme of global order 1, because BE itself is first order. **Recommendation: (A).** It makes $M_{gen}$ equal to the true generated amount even when a switch falls inside a step (N9: $+1.6\times10^{-16}$ relative error against $-0.167$ for (R) in the test case), and it equals (R) when the source is constant on every step.

**The BE system.**

$$
\Big(\frac{V_j}{\Delta t}+\lambda V_j+G_W+G_E\Big)C_j^{n+1}-G_WC_{j-1}^{n+1}-G_EC_{j+1}^{n+1}=\frac{V_j}{\Delta t}C_j^n+V_j\bar S_j^{n+1}. \tag{P21.58}
$$

**What changes relative to v1.0.** Write $a_{jk}$ for the matrix entries and $b_j$ for the right-hand side.

$$
\begin{aligned}
&a_{jj}=v_j+g^W_j+g^E_j,\quad v_j=V_j\Big(\frac{1}{\Delta t}+\lambda\Big),\\
&a_{j,j-1}=-g^W_j,\quad a_{j,j+1}=-g^E_j,\qquad b_j=\frac{V_j}{\Delta t}C_j^n+V_j\bar S_j^{n+1}.
\end{aligned} \tag{P21.59}
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
\begin{aligned}
d_j&=v_j+g^W_j+g^E_j-\frac{(g^W_j)^2}{e_{j-1}+g^W_j}=v_j+g^E_j+g^W_j\,\frac{(e_{j-1}+g^W_j)-g^W_j}{e_{j-1}+g^W_j}\\
&=v_j+g^E_j+\frac{g^W_j\,e_{j-1}}{e_{j-1}+g^W_j}.
\end{aligned}
$$

The subtraction in the middle step is carried out analytically, so the recurrence that the code evaluates contains only sums, products and quotients of nonnegative numbers (exact; X1 checks the induction step and $N=5$ explicitly). Nothing in the proof uses the value of $v_j$ beyond $v_j>0$, so the v1.0 routine `solve_diffusion_system` applies unchanged with the new $v_j$. Moreover $e_j\ge v_j$ and $d_j\ge v_j+g^E_j>0$: decay can only make the pivots larger.

**Positivity.** With $y_j=(b_j+g^W_jy_{j-1})/d_j$ and $C_j^{n+1}=y_j+(g^E_j/d_j)\,C_{j+1}^{n+1}$, $C_N^{n+1}=y_N$:

$$
\begin{aligned}
&C^n\ge0,\ \bar S^{n+1}\ge0\ \Longrightarrow\ b\ge0\ \Longrightarrow\ y\ge0\ \Longrightarrow\ C^{n+1}\ge0\\
&\qquad\text{for every }\Delta t>0,\ \lambda\ge0.
\end{aligned} \tag{P21.62}
$$

(exact). Rounding a nonnegative sum, product or quotient of nonnegative floats gives a nonnegative float, so (P21.62) also holds in floating point: the minimum is $\ge0.0$ exactly, a sharper test than the v1.0 threshold of $-10^{-13}$ (N2: smallest value $1.1\times10^{-128}$ for $\Delta t=10^{-3}$ to $10^{12}\ \mathrm{s}$). A source with negative values (possible in a manufactured solution) voids (P21.62).

**Exact link to the v1.0 scheme.** Since $\mathbf V/\Delta t+\lambda\mathbf V=\mathbf V/\Delta t'$ with $\Delta t'=\Delta t/(1+z)$ and $\mathbf V/\Delta t=(1+z)^{-1}\mathbf V/\Delta t'$, a step of (P21.58) with $S=0$ is

$$
\begin{aligned}
\mathbf C^{n+1}&=(1+z)^{-1}\,\mathbf B_{\Delta t'}\mathbf C^n,\\
\mathbf B_{h}&=(\mathbf V/h+\mathbf K)^{-1}\mathbf V/h\ \ (\text{the v1.0 step}),\\
\mathbf C^n&=(1+z)^{-n}\mathbf B_{\Delta t'}^{\,n}\mathbf C^0.
\end{aligned} \tag{P21.63}
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
\begin{aligned}
\Big(\frac{\mathbf V}{\Delta t}+\mathbf K\Big)\tilde{\mathbf C}^{n+1}&=\frac{\mathbf V}{\Delta t}\mathbf C^n+\mathbf V\bar{\mathbf S}_\varphi^{n+1},\\
\bar{\mathbf S}_\varphi^{n+1}&=\frac1{\Delta t}\int_{t^n}^{t^{n+1}}e^{\lambda(t-t^n)}\bar{\mathbf S}\,dt\ \Big(=\bar{\mathbf S}\,\frac{e^z-1}{z}\ \text{if constant}\Big),\\
\mathbf C^{n+1}&=e^{-z}\tilde{\mathbf C}^{n+1}.
\end{aligned} \tag{P21.66}
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
\begin{aligned}
&\text{per mode, relative: unsplit BE }\ \varepsilon_n\approx\tfrac12(\Lambda_n+\lambda)^2t\,\Delta t;\\
&\text{decay-exact split }\ \varepsilon_n\approx\tfrac12\Lambda_n^2t\,\Delta t;\\
&\text{difference }\tfrac12(2\Lambda_n\lambda+\lambda^2)\,t\,\Delta t.
\end{aligned} \tag{P21.69}
$$

(approximate, leading order in $\Delta t$; from $n\ln[1+(\Lambda_n+\lambda)\Delta t]-(\Lambda_n+\lambda)t$ with $t=n\Delta t$, checker script S8c.) Both schemes are $O(\Delta t)$. The extra error of the unsplit scheme contains the cross term $\Lambda_n\lambda$, so it is not the pure-decay error $\tfrac12\lambda^2t\Delta t$ of (P21.77) added to the diffusion error. In N3 ($\lambda=\Lambda_1$, $t=1/\Lambda_1$) the relative error in $M(t)$ is $2.50\times10^{-3}$ (unsplit) against $6.2\times10^{-4}$ (split) at 800 steps, a ratio of 4 as (P21.69) predicts for the dominant first mode ($\tfrac12(2\Lambda_1)^2t\Delta t=2/800$ against $\tfrac12\Lambda_1^2t\Delta t=0.5/800$); both have observed order 0.999.

**Choice.** Use the unsplit BE (P21.58) in STRATO. Reasons: it is a two-line change to v1.0; it keeps positivity, the subtraction-free solver and bitwise regression; its closure (P21.72) has a decayed term $\Delta t\,\lambda M^{n+1}$ with a direct physical meaning; and it preserves the steady state exactly, which is what $R/B$ comparisons for short-lived nuclides need. Keep (P21.66) with $S=0$ as a cross-check, through (P21.67). Crank-Nicolson would raise the order but its mode factor $(1-x/2)/(1+x/2)$, $x=(\Lambda+\lambda)\Delta t$, is negative for $x>2$, so it gives up unconditional positivity; second-order alternatives are analysed in Section 4.6.1, which finds that no rational second-order scheme is positive for every step over the class $\mathcal S$ (P2.T1), that CN and TR-BDF2 have finite positivity thresholds on every matrix of $\mathcal S$ (P2.P1), and that the exponential integrator is positive for every step.

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
\begin{aligned}
&M_0+M_{gen}^n=M^n+M_{dec}^n+M_{rel}^n,\qquad M_{gen}^n=\sum_{k=1}^n\Delta t_k\,\mathrm{Gen}^k,\\
&M_{dec}^n=\sum_{k=1}^n\Delta t_k\,\lambda M^k,\quad M_{rel}^n=\sum_{k=1}^n\Delta t_k\,G_RC_N^k.
\end{aligned}
\tag{P21.72}
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
\begin{aligned}
&M_{rel}^n=M_{out}^n+M_{dec,out}^n,\quad M_{dec,out}^n=\sum_k\Delta t_k\,\lambda M_{out}^k,\\
&M_0+M_{gen}^n=M^n+M_{out}^n+M_{dec}^n+M_{dec,out}^n.
\end{aligned} \tag{P21.75}
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
\begin{aligned}
&\int_{t^n}^{t^{n+1}}\bar S\,dt-\Delta t\,\bar S(t^{n+1})=-\tfrac12\Delta t^2\dot{\bar S}+O(\Delta t^3)\ (\text{smooth}),\\
&\lvert\text{error}\rvert\le\lvert\Delta S_K\rvert V_K\Delta t\ (\text{jump in the step}).
\end{aligned} \tag{P21.79}
$$

Rules: (1) place a step boundary on every switch time of $S_K(t)$, on every output time and (Phase 2.2) on every temperature-history knot, as MOOSE does with `force_step_every_function_point` (idaholab, 2026; audit 08, Sec. 3); (2) after each switch restart from a small step and grow it geometrically (factor 1.5 in N1) because time derivatives jump there; BE is L-stable, so no oscillation follows; (3) choose $\Delta t_n=\min\{$(P21.77) limit, (P21.78) limit, time to the next switch or output, growth limit$\}$. There is no stability limit. The explicit bound TRISO-FV-300, used only for checks, becomes $\Delta t\le\min_jV_j/(G_W+G_E+\lambda V_j)$ with decay (non-negativity of the forward-Euler update). A step-doubling estimate of $\varepsilon$ is a cheap a-posteriori alternative.

#### 4.6.1 Second-order time integrators and positivity (open issue 5 of Section 4.10)

**Purpose.** Open issue 5 of Section 4.10 notes that no second-order positive integrator was analysed, while (P21.78) makes Backward Euler (BE) expensive near complete release. This section analyses Crank-Nicolson (CN), BDF2, TR-BDF2, Richardson extrapolation of BE and exponential integrators for the semi-discrete system (P21.55), proves a positivity barrier for the STRATO matrix structure, gives the exact positivity conditions of each candidate, proves the matching inventory-closure identities and checks everything on the synthetic five-layer case of `sec4_p21_be_closure_order.py`. The section recommends no scheme; Section 4.6.1.9 lists options with their trade-offs for the owner. Its equations are numbered (P21.113) to (P21.142).

Check script: `scripts/derivation_checks/sec4_p2_positive_second_order.py` (check labels S1a to S6b; result lines in Appendix A.2).

##### 4.6.1.0 Setting, symbols and assumptions

Divide (P21.55) by $\mathbf V$:

$$
\frac{d\mathbf C}{dt}=-\mathbf B\,\mathbf C+\mathbf s(t),\qquad \mathbf B=\mathbf V^{-1}\mathbf K+\lambda\mathbf I,\qquad \mathbf s=\bar{\mathbf S}. \tag{P21.113}
$$

| Symbol | Meaning | Units |
|---|---|---|
| $\mathbf B$, $B_{jk}$ | system matrix of (P21.113) and its entries | $\mathrm{s^{-1}}$ |
| $\mu_k$ | eigenvalues of $\mathbf B$, $\mu_k=\Lambda_k^{sd}+\lambda$ with $\Lambda_k^{sd}$ the semi-discrete diffusion rates; $\mu_1=\Lambda_1+\lambda$ the smallest | $\mathrm{s^{-1}}$ |
| $h$, $x=h\mu$ | time step; dimensionless step of one mode | $\mathrm{s}$, 1 |
| $r(x)$ | stability function of a one-step scheme: one step multiplies a mode by $r(h\mu)$, $r(x)\approx e^{-x}$ | 1 |
| $\mathbf Q_\tau$ | $(\mathbf I+\tau\mathbf B)^{-1}$, the BE step operator of length $\tau$ with $S=0$ | 1 |
| $M_r$ | radius of total monotonicity of $r$ (definition (P21.115)) | 1 |
| $\beta_B$ | $\max_jB_{jj}$, the largest diagonal entry of $\mathbf B$ | $\mathrm{s^{-1}}$ |
| $c_r$ | mode-error constant, $\ln r(x)=-x+c_rx^{p+1}+O(x^{p+2})$ | 1 |
| $\mathcal S$ | structural class defined below | |

Table: 4.4 Symbols of this section (others as Table 4.1).

**Properties of $\mathbf B$ used below (exact).** (i) $\mathbf B$ is tridiagonal with $B_{j,j\pm1}=-g_j^{W,E}/V_j<0$ and $B_{jj}=(g^W_j+g^E_j)/V_j+\lambda>0$, so $\mathbf B$ is a Z-matrix whose row sums $\lambda$ (rows $j<N$) and $\lambda+G_R/V_N$ (row $N$) are nonnegative. (ii) $\mathbf V^{1/2}\mathbf B\mathbf V^{-1/2}=\mathbf V^{-1/2}\mathbf K\mathbf V^{-1/2}+\lambda\mathbf I$ is symmetric positive definite, so the $\mu_k$ are real and positive. (iii) Gershgorin's theorem with (i) gives $0<\mu_k\le2\beta_B$, since the radius of row $j$ does not exceed its centre $B_{jj}$. (iv) $\mathbf Q_\tau\ge0$ entrywise for every $\tau>0$: it is the BE step of (P21.58) with $S=0$, and (P21.62) proves its output is nonnegative for every nonnegative input; moreover $(\mathbf Q_\tau)_{jj}\ge1/(1+\tau B_{jj})>0$ (from row $j$ of $\mathbf Q_\tau(\mathbf I+\tau\mathbf B)=\mathbf I$, whose off-diagonal products are $\le0$).

**Structural class $\mathcal S$.** All matrices $\mathbf B=\mathbf V^{-1}\mathbf K+\lambda\mathbf I$ of any size $N$, with positive diagonal $\mathbf V$, $\mathbf K$ symmetric tridiagonal with off-diagonals $-g_j<0$, zero row sums except row $N$, whose excess is $G_R>0$, and $\lambda\ge0$. Every STRATO matrix lies in $\mathcal S$. The converse is false, because a five-layer mesh fixes the cell volumes and allows only five diffusivities; statements "over $\mathcal S$" are therefore class statements, and the synthetic matrix is examined separately.

**Positivity.** A one-step linear scheme with source has the form $\mathbf C^{n+1}=r(h\mathbf B)\mathbf C^n+\mathbf q^n$, with $\mathbf q^n$ linear in the source. It is positive at step $h$ if $\mathbf C^n\ge0$ and $\mathbf s\ge0$ imply $\mathbf C^{n+1}\ge0$. Taking $\mathbf s=0$ and $\mathbf C^n=\mathbf e_j$ gives the first condition below, taking $\mathbf C^n=0$ gives the second, and together they are clearly sufficient:

$$
\text{positive at step }h\iff r(h\mathbf B)\ge0\ \text{entrywise and}\ \mathbf q^n\ge0\ \text{for every}\ \mathbf s\ge0. \tag{P21.114}
$$

$r$ is **totally monotone** on an interval $I\subset[0,\infty)$ if

$$
\begin{aligned}
&(-1)^k\,r^{(k)}(x)\ge0\quad\text{for all }x\in I,\ k=0,1,2,\dots;\\
&M_r=\sup\{M:\ r\ \text{totally monotone on}\ {}[0,M]\}.
\end{aligned} \tag{P21.115}
$$

A scheme has order $p$ if $r(x)=e^{-x}+O(x^{p+1})$. Assumptions: constant $\mathbf B$ within a step (Phase 2.1; Phase 2.2 is discussed in 4.6.1.6); a source that is constant on each step, which the planner rule of placing a step end on every switch guarantees for the piecewise-constant histories of block E; exact arithmetic unless floating point is stated.

##### 4.6.1.1 The positivity barrier for the class $\mathcal S$

**Literature check.** Bolley and Crouzeix (1978) state the barrier for one-step methods with rational $r$ (Théorème 1, printed p. 241) and for multistep methods (Théorème 3, printed p. 244). They characterise positivity for all M-matrices $\mathbf A$ with $\Delta t\,\sigma(\mathbf A)\le M$, where $\sigma(\mathbf A)$ is the largest diagonal entry, by total monotonicity of $r$ on $[0,M]$ (Lemme 3 and Théorème 2, printed p. 242), and give the CN condition $\Delta t\,\sigma(\mathbf A)\le2$ (printed p. 243). These statements were read in the Numdam scan of the article. Their necessity proof uses general M-matrices, which need not have the tridiagonal symmetrisable structure of $\mathcal S$, so the lemmas below re-prove the statements using members of $\mathcal S$ only. Godunov's theorem concerns advection schemes; its source was not opened and it is not cited.

**Lemma P2.L1 (necessity, constructed inside $\mathcal S$).** Let $r=p/q$ be rational without poles on $(0,\infty)$, and suppose $r(\mathbf B)\ge0$ for every $\mathbf B\in\mathcal S$ of size $N$ with $\beta_B\le M'$. Then $(-1)^kr^{(k)}(x)\ge0$ for $0<x<M'$ and $k\le N-1$.

*Proof.* Fix $0<\varepsilon<x<M'$ and $\delta\in(0,1)$, and take $V_j=\delta^{-(j-1)}$, $g_j=\varepsilon V_j$, $G_R=\varepsilon V_N$, $\lambda=x-\varepsilon$. Then $\mathbf K$ is symmetric with zero row sums except row $N$, so $\mathbf B_\delta\in\mathcal S$, and

$$
\begin{aligned}
&(\mathbf B_\delta)_{j,j+1}=-\varepsilon,\quad (\mathbf B_\delta)_{j+1,j}=-\varepsilon\delta,\quad (\mathbf B_\delta)_{11}=x,\\
&(\mathbf B_\delta)_{jj}=x+\varepsilon\delta\ (j\ge2),\qquad \mathbf B_\delta\to x\mathbf I-\varepsilon\mathbf N\ \ (\delta\to0),
\end{aligned} \tag{P21.116}
$$

with $\mathbf N$ the nilpotent upper shift. For $\delta$ small, $\beta_B=x+\varepsilon\delta\le M'$. The matrix $q(x\mathbf I-\varepsilon\mathbf N)$ is upper triangular with diagonal $q(x)\ne0$, so it is invertible, and $r(\mathbf B_\delta)=p(\mathbf B_\delta)q(\mathbf B_\delta)^{-1}$ is continuous at $\delta=0$. Taylor's formula is exact on a nilpotent perturbation:

$$
r(x\mathbf I-\varepsilon\mathbf N)=\sum_{k=0}^{N-1}\frac{r^{(k)}(x)}{k!}(-\varepsilon)^k\mathbf N^k,\qquad \big[r(x\mathbf I-\varepsilon\mathbf N)\big]_{1,k+1}=(-1)^k r^{(k)}(x)\,\frac{\varepsilon^k}{k!}. \tag{P21.117}
$$

Nonnegativity of $r(\mathbf B_\delta)$ for every $\delta$ passes to the limit, which gives the claim. $\square$ (PROVED; S3a, S3b reproduce (P21.117) to $10^{-6}$ relative at $\delta=10^{-7}$.)

**Lemma P2.L2 (Pringsheim property of a rational function).** Let $r$ be rational and $a$ real with $b_k=(-1)^kr^{(k)}(a)/k!\ge0$ for all $k$, so $r(x)=\sum_kb_k(a-x)^k$ on the disc $\lvert a-x\rvert<\rho$, $\rho$ the distance from $a$ to the nearest pole. Then $x=a-\rho$ is a pole.

*Proof.* For $\lvert w\rvert<\rho$,

$$
\lvert r(a-w)\rvert\le\sum_kb_k\lvert w\rvert^k=r(a-\lvert w\rvert). \tag{P21.118}
$$

If $a-\rho$ were not a pole, the right side would stay bounded as $\lvert w\rvert\to\rho$ by continuity of $r$ at $a-\rho$, while the left side is unbounded as $a-w$ approaches the pole on the circle. $\square$ (PROVED; S2b samples (P21.118).)

**Lemma P2.L3 (sufficiency).** If the rational $r$ is totally monotone on $[0,M]$, then $r(h\mathbf B)\ge0$ for every $\mathbf B\in\mathcal S$ with $h\beta_B\le M$.

*Proof.* If $r$ is a polynomial, (P21.119) below is a finite sum and the conclusion is immediate. Otherwise, by P2.L2 at $a=M$ the nearest pole is $M-\rho$, a real point outside $[0,M]$ (where $r$ is finite), hence $M-\rho<0$ and $\rho>M$. The matrix $M\mathbf I-h\mathbf B$ is entrywise nonnegative (diagonal $M-hB_{jj}\ge0$, off-diagonal $-hB_{jk}\ge0$) and, by property (iii), has real eigenvalues in $[-M,M)$, all inside the disc of convergence. Since $\mathbf B$ is diagonalisable with real spectrum,

$$
r(h\mathbf B)=\sum_{k\ge0}b_k\,(M\mathbf I-h\mathbf B)^k,\qquad b_k=\frac{(-1)^kr^{(k)}(M)}{k!}\ge0, \tag{P21.119}
$$

converges, and every partial sum is a nonnegative matrix. $\square$ (PROVED.)

Together, P2.L1 for all $N$ and P2.L3 show that positivity over $\mathcal S$ under the restriction $h\beta_B\le M$ holds exactly when $r$ is totally monotone on $[0,M]$; so $h\beta_B\le M_r$ is the sharp step bound over the class.

**Lemma P2.L4 (order barrier).** No rational $r$ is totally monotone on $[0,\infty)$ with $r(0)=1$, $r'(0)=-1$, $r''(0)=1$.

*Proof.* A polynomial with $(-1)^kr^{(k)}\ge0$ on $[0,\infty)$ for $k=d$ and $k=d-1$, $d$ its degree, has leading coefficient $a_d$ with $(-1)^da_d\ge0$ and $(-1)^{d-1}a_d\ge0$, so $d=0$ and $r'(0)=0$, a contradiction. Otherwise $r$ has poles, none in $[0,\infty)$. For any $a>0$, P2.L2 places the nearest pole at $a-\rho(a)<0$, so $\rho(a)>a$ and the Taylor series at $a$ converges, with its derivatives, at $x=0$. With $w_k=b_ka^k\ge0$ and $t_k=k/a$, the values at $0$ of $r=\sum b_k(a-x)^k$, $-r'$ and $r''$ give

$$
\sum_kw_k=1,\qquad \sum_kw_kt_k=1,\qquad \sum_kw_kt_k\Big(t_k-\frac1a\Big)=1, \tag{P21.120}
$$

hence

$$
\sum_kw_k(t_k-1)^2=\Big(1+\frac1a\Big)-2+1=\frac1a. \tag{P21.121}
$$

For fixed $x\in(0,a)$, $r(x)=\sum_kw_k\psi_a(t_k)$ with $\psi_a(t)=(1-x/a)^{at}$, which satisfies $\lvert\psi_a'(t)\rvert\le L_a=-a\ln(1-x/a)$ for $t\ge0$. By the Cauchy-Schwarz inequality and (P21.121),

$$
\big\lvert r(x)-\psi_a(1)\big\rvert\le L_a\sum_kw_k\lvert t_k-1\rvert\le L_a\,a^{-1/2}. \tag{P21.122}
$$

As $a\to\infty$, $L_a\to x$ and $\psi_a(1)=(1-x/a)^a\to e^{-x}$, so $r(x)=e^{-x}$ for every $x>0$. Then $p(x)=q(x)e^{-x}$ on an interval, hence on the whole real line (both sides are analytic), which fails as $x\to-\infty$ because $q\not\equiv0$. $\square$ (PROVED; S2a checks the weight identities on $r=1/(1+x)$.)

**Theorem P2.T1 (barrier for STRATO-structured matrices).** No one-step scheme with a rational stability function of order $p\ge2$ is positive for every step $h>0$ and every $\mathbf B\in\mathcal S$. *Proof.* $\mathcal S$ is closed under positive scaling, so unconditional positivity means $r(\mathbf B)\ge0$ on all of $\mathcal S$; P2.L1 for every $N$ and continuity at $x=0$ make $r$ totally monotone on $[0,\infty)$, and order 2 means $r(0)=1$, $r'(0)=-1$, $r''(0)=1$, which P2.L4 excludes. $\square$ (PROVED.) BE ($r=1/(1+x)$, $(-1)^kr^{(k)}=k!/(1+x)^{k+1}$) has $M_r=\infty$ at order 1, and the exponential $e^{-x}$ has $M_r=\infty$ because it is not rational; Section 4.6.1.6 uses this second exit.

**Remark (size matters, PROVED with S3d).** The proof needs matrices of every size. For $N=2$ the order-2 function $r=1/(1+x+x^2/2)$ is positive on all of $\mathcal S$: a $2\times2$ member has $r(h\mathbf B)$ off-diagonals equal to $hB_{12}$ times a divided difference of $r$, which is $\le0$ because $r$ decreases, and diagonals that are convex combinations of $r(h\mu_1)$, $r(h\mu_2)\ge0$ by eigenvalue interlacing. The same $r$ has $(-1)^4r^{(4)}(0)<0$ and fails for $N=5$ (entry $(1,5)=-9.0\times10^{-7}$ in S3d). Whether some order-2 rational $r$ is positive on all members of $\mathcal S$ of one fixed size $N\ge3$ is OPEN. This question is relevant to STRATO, because a STRATO run uses one fixed $N$ (40 on the synthetic mesh), so P2.T1 does not by itself exclude an order-2 scheme that is unconditionally positive on all matrices of that size. For the candidates of this section the gap does not matter, because each of them fails on every fixed matrix:

**Proposition P2.P1 (per-matrix failure of the candidates).** Let $\mathbf B\in\mathcal S$ be any single matrix. (a) CN has a finite positivity threshold: $(\mathbf Q_{h/2})_{jj}\to0$ as $h\to\infty$, so the diagonal of $r_{CN}(h\mathbf B)=2\mathbf Q_{h/2}-\mathbf I$ is negative for large $h$. (b) TR-BDF2 has a finite positivity threshold: $\mathbf B$ is a nonsingular irreducible M-matrix (tridiagonal with nonzero off-diagonals, eigenvalues $\mu_k>0$), so $\mathbf B^{-1}>0$ entrywise, $h\mathbf Q_{\beta h}\to(\beta\mathbf B)^{-1}$ and $h\mathbf Q_{\beta h}^2\to0$, hence $h\,r_{TR}(h\mathbf B)\to-\sqrt2\,(\beta\mathbf B)^{-1}<0$ entrywise and every entry of $r_{TR}(h\mathbf B)$ is negative for large $h$. (c) BDF2 fails for every $h$ by (P21.128). (d) Local Richardson fails for small $h$ when $N\ge5$ (Section 4.6.1.5). (PROVED; check V5 of the verification record in Appendix B.5.3 samples (a) and (b).)

##### 4.6.1.2 Crank-Nicolson

**Scheme.** With the exact step average (A) of the source,

$$
\Big(\mathbf I+\frac h2\mathbf B\Big)\mathbf C^{n+1}=\Big(\mathbf I-\frac h2\mathbf B\Big)\mathbf C^n+h\,\bar{\mathbf s}^{n+1}\quad\Longleftrightarrow\quad \mathbf C^{n+1}=2\,\mathbf Q_{h/2}\Big(\mathbf C^n+\frac h2\bar{\mathbf s}^{n+1}\Big)-\mathbf C^n. \tag{P21.123}
$$

The matrix $\mathbf I+\frac h2\mathbf B$ is the BE matrix of (P21.58) for a step $h/2$, so the subtraction-free solver applies; the final subtraction loses the floating-point guarantee of (P21.62). The step average differs from the trapezoid of the source by $O(h^3)$ per step, so the order stays 2.

**Stability function.** $r_{CN}=(1-x/2)/(1+x/2)=-1+2/(1+x/2)$: order 2, $c_{CN}=-1/12$ (S1d, S1e), $r_{CN}(\infty)=-1$, so stiff modes flip sign every step and lose only a factor $(x-2)/(x+2)\approx1-4/x$. For $k\ge1$, $(-1)^kr_{CN}^{(k)}=2^{1-k}k!/(1+x/2)^{k+1}>0$, while $r_{CN}\ge0$ only for $x\le2$: $M_{CN}=2$ exactly (S1f, S1h). By P2.L1 and P2.L3, CN is positive on all of $\mathcal S$ exactly when $h\beta_B\le2$, the bound printed by Bolley and Crouzeix (1978, p. 243).

**Theorem P2.T2 (exact CN threshold for a given matrix).** From (P21.123),

$$
r_{CN}(h\mathbf B)=2\,\mathbf Q_{h/2}-\mathbf I. \tag{P21.124}
$$

Off-diagonal entries are $2(\mathbf Q_{h/2})_{jk}\ge0$ for every $h$, and the source term $h\mathbf Q_{h/2}\bar{\mathbf s}$ is nonnegative. So CN is positive at step $h$ exactly when $(\mathbf Q_{h/2})_{jj}\ge\frac12$ for every $j$. With $\mathbf W=\mathbf V^{1/2}\mathbf Q_\tau\mathbf V^{-1/2}=(\mathbf I+\tau\mathbf V^{1/2}\mathbf B\mathbf V^{-1/2})^{-1}$ symmetric, $(\mathbf Q_\tau)_{jj}=W_{jj}=\sum_ku_{jk}^2/(1+\tau\mu_k)$ is continuous and strictly decreasing from 1 to 0, so CN is positive exactly for $0<h\le h^*_{CN}=\min_jh_j$, with $h_j$ the root of $(\mathbf Q_{h_j/2})_{jj}=\frac12$. A closed criterion follows from the Schur complement of the symmetric tridiagonal matrix $\tilde{\mathbf P}=\mathbf V+\tau(\mathbf K+\lambda\mathbf V)$, $\tau=h/2$, whose inverse gives $\mathbf Q_\tau=\tilde{\mathbf P}^{-1}\mathbf V$: with $H(a,b)=ab/(a+b)$, $v'_j=V_j(1+\tau\lambda)$, the forward excesses $e_1=v'_1$, $e_j=v'_j+H(e_{j-1},\tau g^W_j)$ of (P21.61) and the backward excesses $f_N=v'_N+\tau G_R$, $f_j=v'_j+H(f_{j+1},\tau g^E_j)$, the pivots of the leading and trailing blocks are $e_{j-1}+\tau g^W_j$ and $f_{j+1}+\tau g^E_j$, and

$$
\begin{aligned}
&\text{CN positive at step }h\iff{}\\
&\qquad\tau\lambda V_j+\tau G_R\,[j=N]+H\big(e_{j-1},\tau g^W_j\big)+H\big(f_{j+1},\tau g^E_j\big)\le V_j\quad\text{for every }j,\\
&\qquad\tau=\frac h2.
\end{aligned} \tag{P21.125}
$$

(PROVED; S4a checks the formula against the solver to $2.7\times10^{-16}$, S4b checks (P21.124), S4c the threshold.) Reading: over half a step, decay plus the two series conductances "half-step conductance in series with the neighbour's storage" must not remove more than the cell's own volume. Because $H(a,b)<b$, $h\beta_B\le2$ is sufficient; because $H$ increases in $a$ and $e_{j-1}\ge v'_{j-1}$, $f_{j+1}\ge v'_{j+1}$, replacing $e,f$ by $v'$ gives a necessary condition. Synthetic case ($N=40$, $\beta_B/\Lambda_1=7150$): $h^*_{CN}=586\ \mathrm{s}$, $h^*_{CN}\beta_B=3.003$, $\Lambda_1h^*_{CN}=4.2\times10^{-4}$, binding cell in the buffer (S4c).

**Closure and steady state.** Summing (P21.123) with weights $V_j$ and $\mathbf1^{\mathsf T}\mathbf K=G_R\mathbf e_N^{\mathsf T}$:

$$
M^{n+1}-M^n=h\,\mathrm{Gen}^{n+1}-\frac h2\lambda\big(M^n+M^{n+1}\big)-\frac h2G_R\big(C_N^n+C_N^{n+1}\big), \tag{P21.126}
$$

so (P21.72) holds with trapezoidal increments of $M_{dec}$ and $M_{rel}$ (PROVED; S6a $1.2\times10^{-16}$). The fixed point for a constant source is $\mathbf B^{-1}\mathbf s$ for every $h$ (S6b).

##### 4.6.1.3 BDF2

**Scheme (constant step, BE start).**

$$
\begin{aligned}
&\Big(\tfrac32\mathbf I+h\mathbf B\Big)\mathbf C^{n+1}=2\mathbf C^n-\tfrac12\mathbf C^{n-1}+h\bar{\mathbf s}^{n+1}\\
&\quad\Longleftrightarrow\quad \mathbf C^{n+1}=\mathbf Q_{2h/3}\Big[\tfrac13\big(4\mathbf C^n-\mathbf C^{n-1}\big)+\tfrac{2h}3\bar{\mathbf s}^{n+1}\Big].
\end{aligned} \tag{P21.127}
$$

**No positivity for any step (PROVED).** With $\mathbf C^n=0$, $\mathbf C^{n-1}=\mathbf e_j$, $\mathbf s=0$,

$$
C^{n+1}_j=-\tfrac13\big(\mathbf Q_{2h/3}\big)_{jj}<0\qquad\text{for every }h>0, \tag{P21.128}
$$

by property (iv) (S4g). Bolley and Crouzeix (1978, Théorème 3, p. 244) give the general multistep barrier. Per mode, the characteristic equation $(\tfrac32+x)\xi^2-2\xi+\tfrac12=0$ has roots

$$
\xi_{1,2}=\frac{2\pm\sqrt{1-2x}}{3+2x},\qquad \lvert\xi_{1,2}\rvert^2=\frac1{3+2x}\ \ (x>\tfrac12), \tag{P21.129}
$$

complex for $x>\frac12$; then the modal sequence is $A\rho^n\cos(n\theta+\phi)$ with $\theta\in(0,\pi)$ and changes sign infinitely often for every nonzero start (S1k, S1l). On the synthetic matrix of Section 4.6.1.8, $h\mu_{max}>\frac12$ for every $h>50.7\ \mathrm{s}$ ($\Lambda_1h>3.6\times10^{-5}$, check V9m), so such modes are present at every step of practical size there; for other meshes this is CLAIMED ONLY. The counterexample (P21.128) does not need complex roots. Order 2 with $c_{BDF2}=-1/3$; after one BE start the principal-root amplitude is $1+\frac34x^2+O(x^3)$ (S1d0), which adds $\frac34(\mu_1h)^2$ to the slowest-mode error. Roots tend to 0 as $x\to\infty$, so stiff modes are damped.

**Closure (constant step, PROVED).** With the modified inventory $\hat M^n=M^n+\frac12(M^n-M^{n-1})$ for $n\ge1$, summing (P21.127) gives $\hat M^{n+1}-\hat M^n=h(\mathrm{Gen}^{n+1}-\lambda M^{n+1}-G_RC_N^{n+1})$ and

$$
M_0+M_{gen}^n=\hat M^n-\tfrac12\big(M^1-M^0\big)+M_{dec}^n+M_{rel}^n, \tag{P21.130}
$$

with the rectangle sums of (P21.72) (S6a $1.4\times10^{-15}$). The plain identity (P21.72) has an $O(h)$ residual $\frac12[(M^1-M^0)-(M^n-M^{n-1})]$. Variable-step BDF2 has step-ratio dependent coefficients; its closure was not derived (OPEN).

##### 4.6.1.4 TR-BDF2

**Scheme.** With $\gamma=2-\sqrt2$, a trapezoid stage over $[t^n,t^n+\gamma h]$ is followed by BDF2 on the points $t^n$, $t^n+\gamma h$, $t^{n+1}$ (S1a confirms the variable-step BDF2 coefficients). For a source constant on the step:

$$
(\mathbf I+\beta h\mathbf B)\mathbf Y=(\mathbf I-\beta h\mathbf B)\mathbf C^n+2\beta h\,\bar{\mathbf s},\qquad (\mathbf I+\beta h\mathbf B)\mathbf C^{n+1}=a\mathbf Y-b\,\mathbf C^n+\beta h\,\bar{\mathbf s}, \tag{P21.131}
$$

$\beta=1-1/\sqrt2=\gamma/2$, $a=(\sqrt2+1)/2$, $b=(\sqrt2-1)/2$, $a-b=1$ (S1b). Both stages use the one matrix $\mathbf I+\beta h\mathbf B$, which is the BE matrix of a step $\beta h$. Writing $\mathbf Q=\mathbf Q_{\beta h}$,

$$
\begin{aligned}
&r_{TR}(x)=-\frac{\sqrt2}{1+\beta x}+\frac{1+\sqrt2}{(1+\beta x)^2},\\
&\mathbf C^{n+1}=\big[(1+\sqrt2)\mathbf Q^2-\sqrt2\,\mathbf Q\big]\mathbf C^n+\beta h\big[(1+\sqrt2)\mathbf Q^2+\mathbf Q\big]\bar{\mathbf s}.
\end{aligned} \tag{P21.132}
$$

(exact; S1c.) Order 2, $c_{TR}=\frac23-\frac{\sqrt2}2=-0.0404$, the smallest of the second-order candidates; $r_{TR}(\infty)=0$; $r_{TR}(1+\sqrt2)=0$ and $\min r_{TR}=-(\sqrt2-1)/2=-0.207$ at $x=4+3\sqrt2$ (S1f). The source part is always nonnegative.

**Exact radius (PROVED).** Differentiating (P21.132),

$$
(-1)^kr_{TR}^{(k)}(x)=\frac{k!\,\beta^k\big[(k+1)(1+\sqrt2)-\sqrt2\,(1+\beta x)\big]}{(1+\beta x)^{k+2}}, \tag{P21.133}
$$

which is nonnegative for every $k$ exactly when the $k=0$ bracket is, i.e. $\beta x\le1/\sqrt2$, $x\le1+\sqrt2$. So $M_{TR}=1+\sqrt2\approx2.414$ (S1i): TR-BDF2 is positive on all of $\mathcal S$ exactly when $h\beta_B\le1+\sqrt2$ (P2.L1 with $k=0$, P2.L3), about 21 % more than CN. The stage $\mathbf Y$ is a CN step of length $2\beta h$, positive under the weaker $h\beta_B\le1/\beta=3.41$. On the synthetic matrix the bound holds (S4d) and the actual threshold is $h^*_{TR}\beta_B=8.92$, $\Lambda_1h^*_{TR}=1.25\times10^{-3}$ (S4e, DEMONSTRATED NUMERICALLY); beyond it entries down to $-0.17$ appear.

**Closure (PROVED).** From (P21.131) with $\mathbf F(\mathbf Z)=\bar{\mathbf s}-\mathbf B\mathbf Z$: $\mathbf Y-\mathbf C^n=\beta h[\mathbf F(\mathbf C^n)+\mathbf F(\mathbf Y)]$ and, using $a-b=1$, $\mathbf C^{n+1}-\mathbf C^n=a(\mathbf Y-\mathbf C^n)+\beta h\mathbf F(\mathbf C^{n+1})$. With $\Phi(\mathbf Z)=\mathbf1^{\mathsf T}\mathbf V\mathbf F(\mathbf Z)=\mathrm{Gen}-\lambda M(\mathbf Z)-G_RZ_N$:

$$
M^{n+1}-M^n=h\Big[\frac{1}{2\sqrt2}\Big(\Phi(\mathbf C^n)+\Phi(\mathbf Y)\Big)+\Big(1-\frac1{\sqrt2}\Big)\Phi(\mathbf C^{n+1})\Big], \tag{P21.134}
$$

weights $a\beta=1/(2\sqrt2)$, $\beta$, summing to 1; $M_{dec}$ and $M_{rel}$ take the same stage-weighted increments (S6a $1.2\times10^{-16}$). The same argument holds for every Runge-Kutta method applied to the linear system, also to the augmented system $(\mathbf C,M_{out})$, so the outside closure (P21.75) carries over with stage-weighted $M_{dec,out}$.

##### 4.6.1.5 Richardson extrapolation of Backward Euler

**Local extrapolation.** With $\mathcal E_\tau$ the BE step (P21.58) of length $\tau$, source included,

$$
\mathbf C^{n+1}=2\,\mathcal E_{h/2}\mathcal E_{h/2}\mathbf C^n-\mathcal E_h\mathbf C^n,\qquad r_{RE}(x)=\frac2{(1+x/2)^2}-\frac1{1+x}. \tag{P21.135}
$$

Order 2, $c_{RE}=+1/6$, $r_{RE}(\infty)=0$, $r_{RE}<0$ for $x>2+2\sqrt2$, minimum $-0.0361$ near $x=11.76$ (S1g, numerical). Three tridiagonal solves per step with two different matrices.

**No positivity for any step (PROVED).**

$$
(-1)^kr_{RE}^{(k)}(x)=\frac{k!}{(1+x)^{k+1}}\Big[\frac{8(k+1)(1+x)^{k+1}}{(2+x)^{k+2}}-1\Big], \tag{P21.136}
$$

and $(1+x)/(2+x)<1$ makes the bracket tend to $-1$ as $k\to\infty$ for every $x\ge0$: $M_{RE}=0$ (S1j; at $x=0$ the first failing order is $k=4$, Taylor coefficient $-3/8$). By P2.L1, for every $h>0$ some member of $\mathcal S$ gives a negative entry. For any STRATO matrix with $N\ge5$ and small $h$, entry $(j,j+4)$ equals $-\frac38h^4\prod_{i=j}^{j+3}\lvert B_{i,i+1}\rvert+O(h^5)<0$, because lower powers of $\mathbf B$ vanish at distance 4 (S4f: $-4.35\times10^{-12}$ against $-4.44\times10^{-12}$ predicted). On the synthetic matrix every step tested from $10^{-3}/\beta_B$ to $20/\Lambda_1$ gives negative entries (S4f).

**Global extrapolation** $2\mathbf C_{h/2}(t)-\mathbf C_h(t)$ combines two positive BE runs. Per mode it is $2(1+x/2)^{-2n}-(1+x)^{-n}$, negative once $n>\ln2/\ln[(1+x/2)^2/(1+x)]$ (PROVED), i.e. for the slowest mode once $\mu_1^2th\gtrsim4\ln2$, which means the coarse BE run is already off by more than 100 %. S4i shows every entry negative at $\mu_1h=0.5$, $n=40$ (DEMONSTRATED NUMERICALLY). In the accurate regime it is a sound a-posteriori error estimate.

**Closure (PROVED).** Each BE run or sub-step satisfies (P21.70) with the same $M_0$ and, with exact step averages, the same generated amount; the weights $2$ and $-1$ sum to 1, so (P21.72) holds with $M_{dec}$ and $M_{rel}$ combined with the same weights (S6a $1.2\times10^{-16}$). The fixed point $\mathbf B^{-1}\mathbf s$ is kept (S6b).

##### 4.6.1.6 Exponential integrators

**Scheme (exponential Euler, exact for a step-constant source).**

$$
\mathbf C^{n+1}=e^{-h\mathbf B}\mathbf C^n+h\varphi_1(-h\mathbf B)\,\bar{\mathbf s},\qquad \varphi_1(z)=\frac{e^z-1}z,\qquad \varphi_2(z)=\frac{\varphi_1(z)-1}z. \tag{P21.137}
$$

For constant $\mathbf B$ and a source constant on each step this is the exact solution of (P21.113): no time error (S5d: $\le1.4\times10^{-12}$ relative in $M$ at $M/M_0=2.4\times10^{-7}$, any $n$). A source linear in time on the step adds $h^2\varphi_2(-h\mathbf B)\dot{\bar{\mathbf s}}$. It is the finite-volume analogue of the exact modal recursion (P21.33) of block R1.

**Positivity (PROVED, unconditional).** With $\beta\ge\beta_B$, $\beta\mathbf I-\mathbf B\ge0$ and

$$
e^{-h\mathbf B}=e^{-h\beta}\sum_{k\ge0}\frac{h^k(\beta\mathbf I-\mathbf B)^k}{k!}\ge0,\qquad h\varphi_1(-h\mathbf B)=\int_0^he^{-\sigma\mathbf B}d\sigma\ge0. \tag{P21.138}
$$

This does not contradict P2.T1, since $e^{-x}$ is not rational. In floating point, the eigen route $\mathbf V^{-1/2}\mathbf U\,\mathrm{diag}(e^{-h\mu_k})\,\mathbf U^{\mathsf T}\mathbf V^{1/2}$ returns round-off negatives ($-2.5\times10^{-16}$ in S4h, $-4.7\times10^{-17}\ \mathrm{mol\ m^{-3}}$ in S6a). Uniformisation with squaring uses only nonnegative numbers:

$$
e^{-h\mathbf B}=\Big(e^{-\tau\beta}\,T_q\big(\tau(\beta\mathbf I-\mathbf B)\big)\Big)^{2^m},\qquad \tau=h/2^m,\ \tau\beta\le\tfrac18,\ T_q(\mathbf X)=\sum_{k=0}^q\frac{\mathbf X^k}{k!}, \tag{P21.139}
$$

giving $\ge0.0$ exactly; with $q=14$ it agrees with the eigen route to $1.4\times10^{-11}$ for $h\beta$ up to $10^5$ (S4h). The truncation error of $T_q$ under squaring was not bounded (DEMONSTRATED NUMERICALLY only).

**Closure (PROVED).** Integrating $\mathbf C(t^n+\sigma)=e^{-\sigma\mathbf B}\mathbf C^n+\sigma\varphi_1(-\sigma\mathbf B)\bar{\mathbf s}$, with $\sigma\varphi_1(-\sigma\mathbf B)=\mathbf B^{-1}(\mathbf I-e^{-\sigma\mathbf B})$,

$$
\int_{t^n}^{t^{n+1}}\mathbf C\,dt=h\varphi_1(-h\mathbf B)\mathbf C^n+h^2\varphi_2(-h\mathbf B)\bar{\mathbf s}\ \ge0, \tag{P21.140}
$$

and the exact law $dM/dt=\mathrm{Gen}-\lambda M-G_RC_N$ gives (P21.72) with $\Delta M_{dec}=\lambda\mathbf1^{\mathsf T}\mathbf V\!\int\!\mathbf C\,dt$ and $\Delta M_{rel}=G_R(\int\mathbf C\,dt)_N$ (S6a $3\times10^{-13}$, eigen route). The fixed point is exact (S6b). The outside inventory can use the exact update (P21.74) consistently.

**Temperature histories (Phase 2.2).** Freezing $\mathbf B$ on each step at the midpoint temperature, or at the per-layer average diffusivities of block R4, gives a product of nonnegative exponentials: positive for every step (PROVED). This is the staircase of block R4 applied to the finite-volume matrix. For small $h\lVert\mathbf B\rVert$ its order 2 rests on the Magnus argument (P22.54) to (P22.56). The stiff regime is treated in Section 5.4.6 (open item O10): order 1 is PROVED uniformly in the stiffness (P22.130); order 2 is PROVED for shared scaling (P22.135) and, up to a factor $1+\ln(T/h)$, on a fixed mesh with a constant proportional to $\Lambda$ (Theorem D, (P22.140)); with layer-specific $Q$ the observed $V$-norm order is about 1.75 (DEMONSTRATED NUMERICALLY), and an order above 1 uniform in the mesh stiffness is OPEN. Closure per step holds with the $G_R$ of the step.

**Cost.** One symmetric tridiagonal eigenproblem of size $N$ per distinct $\mathbf B$ (once per constant-temperature segment, every step for a ramp), a dense $N\times N$ product per step, and safe evaluation of $\varphi_1,\varphi_2$ (with `expm1`); (P21.139) costs $O(N^3\log_2(8h\beta))$ per distinct $(\mathbf B,h)$. v1.0 has no such routine in the finite-volume code, so this is new Rust code or a new dependency.

##### 4.6.1.7 Accuracy near complete release: the second-order analogue of (P21.78)

For a one-step scheme of order $p$, $r(x)^n=e^{-nx}\exp(nc_rx^{p+1}+\dots)$, so with $t=nh$ the slowest mode carries the relative error

$$
\varepsilon_1(t)\approx c_r\,\mu_1^{p+1}\,t\,h^p\quad(\text{BDF2 with BE start: add }\tfrac34\mu_1^2h^2),\qquad \mu_1=\Lambda_1+\lambda, \tag{P21.141}
$$

(approximate, leading order). BE ($p=1$, $c=\frac12$) recovers (P21.78). The step rule becomes

$$
h\le\Big(\frac{\varepsilon_1}{\lvert c_r\rvert\,\mu_1^{p+1}\,t_{end}}\Big)^{1/p},\qquad c_{CN}=-\tfrac1{12},\ c_{TR}=\tfrac23-\tfrac{\sqrt2}2,\ c_{RE}=\tfrac16,\ c_{BDF2}=-\tfrac13. \tag{P21.142}
$$

S5c confirms (P21.141) with sign at $n=3200$ (observed over predicted 1.0000, 1.0004, 0.9929, 1.0037 and 1.0154 for CN, TR-BDF2, Richardson, BDF2 and BE). Calculated with the Section 4.6 inputs for CRP-6 case 1b ($\Lambda_1=1.97\times10^{-5}\ \mathrm{s^{-1}}$, $\Lambda_1t=14.2$, decay neglected) and $\varepsilon_1=10^{-3}$ in $1-F$: BE $7.1\ \mathrm{s}$, CN $1476\ \mathrm{s}$, TR-BDF2 $2118\ \mathrm{s}$, Richardson $1043\ \mathrm{s}$; so $\Lambda_1\Delta t\lesssim10^{-3}$ of open issue 5 relaxes to $\Lambda_1h\approx0.02$ (Richardson) to $0.04$ (TR-BDF2) at second order. The positivity bounds scale differently: for a homogeneous sphere on a uniform mesh $\beta_B/\Lambda_1$ grows like $N^2$ (S5g: 567 at $N=40$, 7979 at $N=150$), so the class-sharp CN bound $h\beta_B\le2$ is $\Lambda_1h\le2.5\times10^{-4}$ at $N=150$, about $12.7\ \mathrm{s}$ with the CRP-6 $\Lambda_1$ (arithmetic for a uniform mesh; the mesh of matrix row 12 was not checked for uniformity).

##### 4.6.1.8 Numerical verification on the synthetic case

Five-layer TECDOC-1674 Table 10.2 geometry, synthetic $D=(4\times10^{-14},\,4\times10^{-13},\,2\times10^{-14},\,5\times10^{-15},\,2\times10^{-14})\ \mathrm{m^2\ s^{-1}}$, cells $(20,8,4,4,4)$, kernel loaded with $1\ \mathrm{mol\ m^{-3}}$, $\Lambda_1=7.166\times10^{-7}\ \mathrm{s^{-1}}$, $\lambda=0.1\Lambda_1$, $S=0$, $t_{end}=14/\Lambda_1$, reference $M(t_{end})/M_0=2.4276\times10^{-7}$ (double eigen route, equal to a 30-digit mpmath value to $5\times10^{-12}$, S5a). Relative error of $M(t_{end})$:

| $n$ | BE | CN | TR-BDF2 | Richardson | BDF2 | ETD |
|---------------------|------------------------------|-----------------------------|------------------------------|------------------------------|-----------------------------|-------------------------------|
| 50 | $+6.2$ | $-23.2$ | $-5.9\times10^{-2}$ | $+1.8\times10^{-1}$ | $-4.5\times10^{-1}$ | $-2.0\times10^{-14}$ |
| 200 | $+7.6\times10^{-1}$ | $-7.6\times10^{-3}$ | $-3.7\times10^{-3}$ | $+1.4\times10^{-2}$ | $-2.7\times10^{-2}$ | $-1.0\times10^{-13}$ |
| 800 | $+1.6\times10^{-1}$ | $-4.8\times10^{-4}$ | $-2.3\times10^{-4}$ | $+9.3\times10^{-4}$ | $-1.7\times10^{-3}$ | $-3.2\times10^{-13}$ |
| 3200 | $+3.8\times10^{-2}$ | $-3.0\times10^{-5}$ | $-1.4\times10^{-5}$ | $+5.9\times10^{-5}$ | $-1.0\times10^{-4}$ | $-1.5\times10^{-12}$ |
| order (last pair) | 1.022 | 2.000 | 2.001 | 1.990 | 2.005 | round-off |
| negative cells in the run | none ($\ge0.0$ exactly) | $n\le1600$ (down to $-0.58$) | none | $n\ge800$ (down to $-8.5\times10^{-5}$) | none | none |

Table: 4.5 Near complete release on the synthetic case (S5b to S5e; $C_0=1\ \mathrm{mol\ m^{-3}}$). At $n=50$ CN returns a negative inventory, $M=-22\,M_{ref}$.

Steps needed for $\varepsilon_1=10^{-3}$ at $\mu_1t=15.4$ (leading order): BE 118580, CN 552, TR-BDF2 384, Richardson 780, BDF2 1020, ETD 1. Steps for guaranteed positivity on this matrix: CN 33334 ($h^*_{CN}$, PROVED threshold), TR-BDF2 41464 (class bound, PROVED) or 11224 ($h^*_{TR}$, positivity below it DEMONSTRATED NUMERICALLY on a grid only). One-step images of the kernel-loaded start stay nonnegative under TR-BDF2 at the sampled points up to $\Lambda_1h=0.42$, although $r_{TR}(h\mathbf B)$ already has negative entries, and are negative at $\Lambda_1h=1.4$ (S5f); on a 400-point logarithmic grid the first negative image appears at $\Lambda_1h=0.62$ (check V9h). The observed sign failures depend on the state. With a switching source and decay (closure run S6a, steps growing by 1.5 from $10^3\ \mathrm{s}$, BDF2 at $10^6\ \mathrm{s}$) the minima were CN $-9.9\times10^{-4}$, BDF2 $-4.0\times10^{-4}$, Richardson $-7.4\times10^{-5}$, TR-BDF2 $-1.1\times10^{-5}\ \mathrm{mol\ m^{-3}}$, while the generated amount per unit kernel volume reaches $S_Kt=30\ \mathrm{mol\ m^{-3}}$ in the first phase; every scheme closed the balance to $\le3\times10^{-13}$ relative with its own quadrature.

##### 4.6.1.9 Summary and options for the owner

| Scheme | Order, $c_r$ | Positive for | $r(\infty)$ | Closure | Solves per step |
|---|---|---|---|---|---|
| BE (P21.58) | 1, $\tfrac12$ | every $h$ (PROVED, also in floating point) | 0 | (P21.72) | 1 |
| CN (P21.123) | 2, $-\tfrac1{12}$ | $h\le h^*_{CN}$ from (P21.125); over $\mathcal S$: $h\beta_B\le2$ (PROVED) | $-1$ | trapezoid (P21.126) | 1 |
| BDF2 (P21.127) | 2, $-\tfrac13$ | no $h$ for general history (PROVED) | 0 | $\hat M$ (P21.130), constant step | 1, two levels stored |
| TR-BDF2 (P21.131) | 2, $-0.0404$ | over $\mathcal S$: $h\beta_B\le1+\sqrt2$, sharp (PROVED); synthetic matrix $h\beta_B\le8.92$ (numerical) | 0 | stage weights (P21.134) | 2, one matrix |
| Local Richardson (P21.135) | 2, $\tfrac16$ | no $h$ over $\mathcal S$; small $h$ fails for every $N\ge5$ (PROVED) | 0 | combined (PROVED) | 3, two matrices |
| ETD (P21.137) | exact for constant $\mathbf B$ and step-constant source | every $h$ (PROVED); floating point via (P21.139) (numerical) | 0 | exact integrals (P21.140) | dense, eigen or (P21.139) |

Table: 4.6 Candidates. All keep the semi-discrete steady state for every $h$ (PROVED: the steady state solves every stage equation; S6b $\le3.1\times10^{-13}$).

Options, each with its trade-off (no choice is made here):

- **Keep BE as the only integrator.** Positivity and closure stay exact, also in floating point, and the code change stays two lines. Cost: about $10^5$ steps for $10^{-3}$ in $1-F$ near complete release on the synthetic case (7 s steps for CRP-6 1b). Global Richardson of two BE runs can serve as an a-posteriori error estimate without affecting the reported solution.
- **Add TR-BDF2 as an option.** Smallest error constant, L-stable, one matrix in both stages, the existing solver serves both solves, closure with stage weights. Positivity is guaranteed only under $h\beta_B\le1+\sqrt2$ (or the per-matrix threshold), which on the synthetic mesh costs about 30 to 110 times the steps that accuracy needs; without the cap, small negative values appeared in the switching-source run, and the floating-point guarantee of (P21.62) is lost. Block J would then need a negativity tolerance instead of the exact test.
- **Add the exponential integrator for constant-temperature segments, and the frozen-$\mathbf B$ staircase for ramps.** Exact in time for Phase 2.1 histories, unconditionally positive, exact closure; the natural tool for $1-F$ near complete release. Cost: new dense linear-algebra code (eigen or uniformisation with squaring) and, for ramps, an order 2 that is PROVED only for shared scaling and, up to a factor $1+\ln(T/h)$, on a fixed mesh (Section 5.4.6); with layer-specific $Q$ an order above 1 uniform in the mesh stiffness is OPEN (O10).
- **Crank-Nicolson, BDF2 and local Richardson.** Analysed for completeness. CN needs $h\le h^*_{CN}$ and leaves stiff modes undamped; BDF2 has no positive regime for general data and complicates variable steps and restarts; local Richardson has no positive step at all. None of the three is attractive as a production integrator.

##### 4.6.1.10 Status of each result

| Result | Label |
|---|---|
| P2.L1 to P2.L4, Theorem P2.T1 (barrier over $\mathcal S$) | PROVED; mechanics DEMONSTRATED NUMERICALLY (S2, S3) |
| Fixed-$N$ remark: order-2 positive $r$ exists for $N=2$ | PROVED (interlacing argument), S3d; fixed $N\ge3$ OPEN (relevant: STRATO runs have fixed $N$) |
| Proposition P2.P1: CN and TR-BDF2 have finite thresholds on every matrix of $\mathcal S$ | PROVED; sampled by V5 |
| CN: $M_{CN}=2$, exact threshold (P21.125), monotone diagonal | PROVED; S4a to S4c |
| BDF2: counterexample (P21.128), complex roots, closure (P21.130) | PROVED; S1k, S1l, S4g, S6a; variable-step closure OPEN |
| TR-BDF2: (P21.132), (P21.133), $M_{TR}=1+\sqrt2$ sharp over $\mathcal S$, closure (P21.134) | PROVED; S1, S4d, S6a |
| TR-BDF2 threshold 8.92 of the synthetic matrix | DEMONSTRATED NUMERICALLY (S4e) |
| Local Richardson: $M_{RE}=0$, small-$h$ entry $-\frac38h^4\prod\lvert b\rvert$ | PROVED; S1j, S4f |
| Global Richardson: modal sign condition PROVED; whole vector negative | DEMONSTRATED NUMERICALLY (S4i) |
| ETD: exactness, positivity (P21.138), closure (P21.140) | PROVED; S5d, S6a |
| ETD floating-point positivity by (P21.139) | DEMONSTRATED NUMERICALLY (S4h) |
| Frozen-$\mathbf B$ ETD with $D(T(t))$: positivity; order in the stiff regime (Section 5.4.6) | positivity PROVED; order 1 PROVED uniformly (P22.130); order 2 PROVED for shared scaling (P22.135) and, up to a factor $1+\ln(T/h)$, on a fixed mesh (P22.140); $V$-norm order about 1.75 for layer-specific $Q$ DEMONSTRATED NUMERICALLY; an order above 1 uniform in the mesh stiffness OPEN (O10) |
| Orders and $c_r$ | PROVED (exact series, S1d, S1e); DEMONSTRATED NUMERICALLY (S5b) |
| Error rule (P21.141), (P21.142) | approximate, leading order; DEMONSTRATED NUMERICALLY to 1.5 % (S5c) |
| Steady state kept by every scheme | PROVED; S6b |
| Negativity magnitudes in runs | DEMONSTRATED NUMERICALLY for the synthetic case only |

Table: 4.7 Labels.

##### 4.6.1.11 Limitations

The class $\mathcal S$ is wider than the set of five-layer STRATO matrices, so P2.T1 does not by itself show that a given STRATO mesh fails; the per-matrix thresholds and the run results above are for one synthetic mesh and one synthetic set of diffusivities. All second-order statements assume a source constant on each step and constant $\mathbf B$ within the step; second-order CN, TR-BDF2 or BDF2 with $D(T(t))$ would need conductances at stage times, which was not analysed (OPEN). The leading-order rule (P21.141) ignores the stiff transient after a start or a switch, which for CN can dominate (Table 4.5, $n=50$). No Rust was written; the Python transcription in the check script is the only executed version.

### 4.7 Exact references used by the tests

#### 4.7.1 Decay identity (exact)

Let $c_0$ solve (P21.50)-(P21.51) with $\lambda=0$, $S=0$ and the same initial value. Then

$$
c_\lambda(r,t)=e^{-\lambda t}c_0(r,t). \tag{P21.80}
$$

Proof: $\partial_tc_\lambda=-\lambda c_\lambda+e^{-\lambda t}\partial_tc_0=-\lambda c_\lambda+e^{-\lambda t}\mathcal Lc_0=-\lambda c_\lambda+\mathcal Lc_\lambda$, with $\mathcal L$ the linear spatial operator (also when $D_i=D_i(t)$). The centre, interface and surface conditions are linear and homogeneous, so they are multiplied by $e^{-\lambda t}$ and still hold; the initial values agree. Uniqueness of the solution of the linear problem gives (P21.80) (D1, D1b). For one homogeneous sphere this is the factor $e^{-\lambda t}$ of Gelbard (2003, Eqs. II.6 and II.16 to II.18, pp. 11-14); (P21.80) extends it to five layers. With a source, $c=e^{-\lambda t}\tilde c$ where $\tilde c$ solves the decay-free problem with source $e^{\lambda t}S$ (D2). Conditions: $\lambda$ uniform in space and constant in time; it fails if a precursor feeds the nuclide or if $\lambda$ differed by layer.

**Inventory partition (series).** With the Phase 1 weights $w_n$ (TRISO-DEL-021), $M_{v1}(t)=M_0\sum_nw_ne^{-\Lambda_nt}$ and $J_{R,v1}=-dM_{v1}/dt$. By (P21.80), $J_R=e^{-\lambda t}J_{R,v1}$ and

$$
\begin{aligned}
\frac{M(t)}{M_0}&=\sum_nw_ne^{-(\Lambda_n+\lambda)t},\\
\frac{M_{rel}(t)}{M_0}&=\sum_nw_n\frac{\Lambda_n}{\Lambda_n+\lambda}\big(1-e^{-(\Lambda_n+\lambda)t}\big),\\
\frac{M_{dec}(t)}{M_0}&=\sum_nw_n\frac{\lambda}{\Lambda_n+\lambda}\big(1-e^{-(\Lambda_n+\lambda)t}\big).
\end{aligned} \tag{P21.81}
$$

(series; $M+M_{rel}+M_{dec}=M_0$ term by term, D3; $\lambda\to0$ returns Phase 1.) Truncation needs care. The Phase 1 rule ($t\ge40/\Lambda_{max}$, `min_valid_time`) controls only the series for $M$, whose omitted terms carry $e^{-40}$. The $M_{rel}$ series as written converges like the weight tail $\sum_{n>n_{max}}w_n$, which is $O(1/n_{max})$ for a uniformly loaded sphere ($w_n=6/(n^2\pi^2)$): with the 13 modes that satisfy the rule in CRP-6 case 1a, a term-by-term $M_{rel}/M_0$ is low by $0.045$ (checker N11). Evaluate instead $M_{rel}=M_0-M-M_{dec}$, using $\sum_nw_n=1$ exactly; the omitted part of $M_{dec}/M_0$ is then at most $\lambda t\sum_{n>n_{max}}w_n/(\Lambda_nt)$ ($1.7\times10^{-7}$ in case 1a with Cs-137 decay), so add modes until this bound is below the target accuracy. Two consequences for comparisons: $M_{rel}/M_0$ depends on $\lambda$, while the decay-corrected fraction $F_{corr}=1-e^{\lambda t}M(t)/M_0$ equals the decay-free $F$ exactly. For a uniformly loaded sphere at long times $M_{rel}(\infty)/M_0=\sum w_n\Lambda_n/(\Lambda_n+\lambda)\approx1-\lambda\tau_{res}$ with $\sum_nw_n/\Lambda_n=R^2/(15D)$.

#### 4.7.2 Constant kernel source, no decay: multilayer steady state (exact)

Steady flux from the integrated balance: $r^2J=S_Kr^3/3$ for $r<R_1$ and $r^2J=S_KR_1^3/3$ for $r>R_1$. Integrating $-D_ic'=J$ inward from $c(R)=0$ with continuity of $c$:

$$
\begin{aligned}
c(r)&=\frac{S_K R_1^3}{3}\left[\frac{1}{D_i}\Big(\frac1r-\frac1{R_i}\Big)+\Phi_i\right]\ (R_{i-1}\le r\le R_i,\ i\ge2),\\
c(r)&=\frac{S_K(R_1^2-r^2)}{6D_1}+\frac{S_KR_1^3}{3}\Phi_1\ (r\le R_1),
\end{aligned}
$$

$$
\Phi_i=\sum_{m=i+1}^{5}\frac{1}{D_m}\Big(\frac{1}{R_{m-1}}-\frac{1}{R_m}\Big). \tag{P21.82}
$$

$$
J_R=\mathrm{Gen}=\tfrac{4\pi}{3}R_1^3S_K. \tag{P21.83}
$$

Integrating by parts, $M_{ss}=-\int_0^R\tfrac{4\pi r^3}{3}c'\,dr$, gives

$$
\begin{aligned}
M_{ss}&=\frac{4\pi S_K}{9}\left[\frac{R_1^5}{5D_1}+R_1^3\sum_{i=2}^{5}\frac{R_i^2-R_{i-1}^2}{2D_i}\right],\\
\tau_{res}&=\frac{M_{ss}}{\mathrm{Gen}}=\frac{R_1^2}{15D_1}+\sum_{i=2}^{5}\frac{R_i^2-R_{i-1}^2}{6D_i}.
\end{aligned} \tag{P21.84}
$$

(exact; A1 to A9.) Each layer adds a series "residence resistance" $(R_i^2-R_{i-1}^2)/(6D_i)$. For zero initial inventory and constant source, $M_{rel}=\mathrm{Gen}\,t-M(t)$, so $F=M_{rel}/(\mathrm{Gen}\,t)=1-\tau_{res}/t$ plus exponentially decaying terms (approximate at finite $t$, exact as an asymptote). For one material with $R_1=R$, $\tau_{res}=R^2/(15D)$ reproduces the Booth asymptote $F\approx1-1/(15D't)$ of IAEA (2012, Eq. 7.3, printed p. 317) (A9). In the discrete scheme with $\lambda=0$, (P21.70) at a fixed point gives $G_RC_N=\mathrm{Gen}$ exactly, so $J_R/\mathrm{Gen}=1$ is a round-off test while the profile converges at order 2.

#### 4.7.3 Source and decay: multilayer steady state (semi-analytical)

With $u=rc$, (P21.50) at steady state reads $D_iu''-\lambda u+S_Kr\,H(R_1-r)=0$. With $s=r-R_{i-1}$:

$$
\begin{aligned}
u_1&=\frac{S_K}{\lambda}r+A_1\sinh(k_1r),\\
u_i&=A_i\sinh(k_is)+B_i\cosh(k_is)\ (i\ge2),\qquad k_i=\sqrt{\lambda/D_i},
\end{aligned} \tag{P21.85}
$$

with $u_i(R_i)=u_{i+1}(R_i)$, $D_i(u_i'-u_i/r)=D_{i+1}(u_{i+1}'-u_{i+1}/r)$ at $R_i$ (TRISO-ML-439, TRISO-ML-446) and $u_5(R)=0$: nine linear equations for nine coefficients. This is the Phase 1 transfer walk with $\Lambda\to-\lambda$ (cos, sin replaced by cosh, sinh) plus the particular solution in the kernel. Checked at 60 digits for the Table 10.2 geometry with synthetic $D_i=(4\times10^{-14},4\times10^{-13},2\times10^{-14},5\times10^{-15},2\times10^{-14})\ \mathrm{m^2\ s^{-1}}$ and $\lambda=2\times10^{-8}\ \mathrm{s^{-1}}$: conditions to $10^{-54}$, $\mathrm{Gen}=\lambda M_{ss}+J_R$ to $10^{-55}$, $R/B=0.9684291531$, and $\lambda\to0$ approaches (P21.82) with an $O(\lambda)$ error (C1 to C4). The FV fixed point on a 40-cell mesh with (20, 8, 4, 4, 4) cells per layer gives $R/B=0.968429$ for the same data (N8a; numerical, spatial error included). With 8 cells per layer the FV value is $0.968413$ (checker N2), so the agreement to six digits depends on that mesh.

#### 4.7.4 Booth single sphere with source and decay (exact and series)

For one material of radius $R$ with uniform source $S$ (IAEA, 2012, Eq. 7.5, printed p. 319):

$$
\begin{aligned}
c(r)&=\frac{S}{\lambda}\left[1-\frac{R\sinh(kr)}{r\sinh(kR)}\right],\\
\frac{R}{B}&=\frac{3}{\sqrt x}\Big(\coth\sqrt x-\frac1{\sqrt x}\Big)=\sum_{n\ge1}\frac{6}{n^2\pi^2+x},\qquad x=(kR)^2=\frac{\lambda}{D'},
\end{aligned} \tag{P21.86}
$$

which is Eq. (7.6) of the report (B1 to B5). The symbol $x=\lambda/D'$ is the one of Section 3, Eqs. (P21.48) and (P21.49). From the modal equations $\dot m_n=-(\Lambda_n+\lambda)m_n+Bw_n$ with $\Lambda_n=n^2\pi^2D'$, $w_n=6/(n^2\pi^2)$, zero initial inventory and birth rate $B=\mathrm{Gen}$:

$$
\begin{aligned}
&\frac{R}{B}(t)=\sum_n\frac{6}{n^2\pi^2+x}\Big(1-e^{-(n^2\pi^2D'+\lambda)t}\Big),\\
&F_{(7.8)}(t)=\frac{M_{out}(t)}{M(t)+M_{out}(t)},\qquad M+M_{out}=\frac{B(1-e^{-\lambda t})}{\lambda}.
\end{aligned} \tag{P21.87}
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
\begin{aligned}
\bar S_j^{n+1}&=\frac{1}{V_j\Delta t}\int_{t^n}^{t^{n+1}}\Big[\mathcal S_i(r_{j+1/2},t)-\mathcal S_i(r_{j-1/2},t)\Big]dt\\
&=\frac{\mathcal S_i(r_{j+1/2},t^{n+1/2})-\mathcal S_i(r_{j-1/2},t^{n+1/2})}{V_j},\quad \mathcal S_i'=4\pi r^2S_i,
\end{aligned} \tag{P21.90}
$$

where the step average equals the midpoint value because $S$ is linear in $t$ (exact). The closed-form antiderivative $\mathcal S_i$ is generated by SymPy (M4). A steady variant ($g=1$, no $\psi/\tau$ term) isolates the spatial order; the transient variant on a fine mesh isolates the temporal order.

#### 4.7.6 CRP-6 sensitivity cases 1a and 1b (series; benchmark data)

Bare UO2 kernel, $r_k=250\ \mu\mathrm{m}$, uniform initial Cs-137, no source, absorbing surface, $200\ \mathrm{h}=720\,000\ \mathrm{s}$ at $1200\ ^\circ\mathrm{C}$ (1a) and $1600\ ^\circ\mathrm{C}$ (1b) (IAEA, 2012, Table 10.5, printed p. 543; Table 10.2, printed p. 541). The benchmark diffusivity is the two-branch FRG Cs-in-UO2 law of IAEA (1997, Table A-1, printed p. 439), and Booth gives

$$
\begin{aligned}
&D(T)=5.6\times10^{-8}e^{-209\,000/(R_gT)}+5.2\times10^{-4}e^{-362\,000/(R_gT)}\ \mathrm{m^2\ s^{-1}},\\
&F=1-\frac{6}{\pi^2}\sum_{n\ge1}\frac{e^{-n^2\pi^2D't}}{n^2},\quad D'=D/r_k^2
\end{aligned} \tag{P21.91}
$$

(IAEA, 2012, Eq. 10.7, printed p. 544). Results (numerical, B3): with $R_g=8.3145$ (Eq. 10.5a), 1a: $D't=0.02594$, $F=0.46742$; with $R_g=8.314$, $F=0.46721$; the printed analytical value 0.4673 (Table 10.6, printed p. 544) lies inside this bracket. With the TECDOC-978 constant $R_g=8.3143$ (IAEA, 1997, p. 435) $F=0.467337$, which is the only one of these values that rounds to 0.4673 (B3 check `b3_sec4.py`, X11a; `b3_sec5.py`, T1f). 1b: $D't=1.4405$, $F=0.99999959$, equal to the printed value. The single-branch TECDOC-1674 Table 7.1 law ($D'_0=0.90\ \mathrm{s^{-1}}$, $Q=209\ \mathrm{kJ\ mol^{-1}}$) gives 0.46163 and 0.99995461 and does not reproduce the table, as found in audit 04 (Sec. 4.3). FV with $N=150$: 1a $F=0.46737$ ($\Delta t=360\ \mathrm{s}$) and $0.46740$ ($36\ \mathrm{s}$); 1b $1-F=4.28\times10^{-7}$ and $4.09\times10^{-7}$; closure $\le1.7\times10^{-14}$. The benchmark analytical values are decay-free. If Cs-137 decay is switched on, $M_{rel}/M_0$ drops by $7.5\times10^{-5}$ (1a) and $2.4\times10^{-5}$ (1b); the 1b shift equals $\lambda\tau_{res}=2.43\times10^{-5}$ of Section 4.7.1, which is 60 times the 1b target $1-F$. Compare decay runs through $F_{corr}$ of (P21.81), or run 1a/1b with $\lambda=0$.

**Observed order** (used in the matrix):

$$
p=\log_2\frac{e_h}{e_{h/2}},\qquad p_{diff}=\log_2\frac{\lvert X_h-X_{h/2}\rvert}{\lvert X_{h/2}-X_{h/4}\rvert}, \tag{P21.92}
$$

with $e$ a volume-weighted $L_2$ or scalar error against the exact reference, and $p_{diff}$ used when a fixed error of the other discretisation (for example a spatial floor in a time study) would mask $e$.

### 4.8 (f) Verification matrix

| No. | Test | Exact reference | Quantity | Expected behaviour / order | Tolerance |
|---|-----------------|-------------------|----------------------|-----------------|----------------------|
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

`scripts/derivation_checks/sec4_p2_positive_second_order.py` (added 2026-11-13; Section 4.6.1, (P21.113) to (P21.142)): symbolic orders and error constants of Crank-Nicolson, BDF2, TR-BDF2, local Richardson extrapolation and the exponential integrator, the positivity barrier on the matrix family of its proof, the exact Crank-Nicolson criterion (P21.125), and every candidate run on the synthetic five-layer case of `sec4_p21_be_closure_order.py`. Result: `41 PASS` and `ALL PASS`; the full output is in Appendix A.2 and the independent verification in Appendix B.5.3.

### 4.10 Assumptions, limiting cases and open issues

**Limiting cases verified.** $\lambda=0$, $S=0$: (P21.58) is TRISO-DEL-040 bit for bit (N5). $\lambda\to0$: (P21.85) tends to (P21.82) (C4), (P21.81) to Phase 1 (D3d), Eq. (7.8) to Eq. (7.1) (D5), $M_{out}\to M_{rel}$ (P21.76). $\Delta t\to\infty$ with constant source: the BE step returns the semi-discrete steady state (N8a). All $D_i$ equal and $R_1=R$: $\tau_{res}=R^2/(15D)$ (A9). $G=0$: pure decay (N6).

**Assumptions restated.** Uniform $\lambda$ per nuclide; no precursor feeding (otherwise (P21.80) fails and the coupled chain model of Section 3.3.4, FUTURE work, is needed); constant $D_i$ within a step (time dependence belongs to Phase 2.2); perfect interfaces without partition coefficients; absorbing surface; uniform kernel source with cumulative yield (approximate). The synthetic $D_i$ in the checks are test values, not material data.

**Open issues.**
1. The interpretation $F_{(7.8)}=M_{out}/(M+M_{out})$ is derived exactly. The B3 reconciliation read printed p. 319 of TECDOC-1674: Eq. (7.8) is introduced only as the cumulative fractional release, with no definition in terms of atom populations, so the identification rests on the derivation (Section 4.7.4), not on a printed statement.
2. CRP-6 case 1a matches the printed 0.4673 only inside the $R_g$ bracket (0.46721 to 0.46742); which $R_g$ the benchmark authors used is NOT VERIFIED. Whether participants included Cs-137 decay is NOT VERIFIED (audit 04, Sec. 2.4); decay shifts $M_{rel}/M_0$ in 1b by 60 times the target $1-F$.
3. Running cases 1a/1b in STRATO needs either a uniform initial concentration over all regions or a configurable region count; the v1.0 `initial_concentration` loads the kernel only and `Config::validate` requires exactly five regions ([src/config.rs, line 80](https://github.com/weewzr/STRATO/blob/v1.0.0/src/config.rs#L80)). Implementation choice left open.
4. Matrix row 7 depends on the Phase 2.1 modal solver of Section 3 (Code C extended with decay and source); its tolerance should be fixed once that solver reports an accuracy floor.
5. Updated 2026-11-13: second-order integrators are analysed in Section 4.6.1. No rational second-order scheme is positive for every step over the class $\mathcal S$ that contains the STRATO matrices (barrier P2.T1, PROVED). Over $\mathcal S$, Crank-Nicolson is positive for $h\beta_B\le2$ and TR-BDF2 for $h\beta_B\le1+\sqrt2$, both sharp (PROVED); on a given matrix Crank-Nicolson has the exact threshold $h^*_{CN}$ of (P21.125) (PROVED) and TR-BDF2 a threshold found by bisection (8.919 for the synthetic matrix, DEMONSTRATED NUMERICALLY). BDF2 has no positive step for a general history, and local Richardson extrapolation has none over $\mathcal S$ (PROVED), and the exponential integrator is positive for every step (PROVED). Still OPEN: whether a second-order scheme is positive on all matrices of one fixed size $N\ge3$, which is the question that matters for one STRATO run with its fixed mesh; the variable-step BDF2 closure; and second-order stages with $D(T(t))$. The choice of integrator is left to the review (pseudocode Sections 9.4 and 9.5). With BE alone, (P21.78) still shows that $1-F$ near complete release needs $\Lambda_1\Delta t\lesssim10^{-3}$.
6. The Rust changes of Section 4.3 are specified but not implemented; the Python transcription in `sec4_p21_be_closure_order.py` is the only executed version.

## 5 Part 2.2: temperature-dependent diffusivity

This section extends the STRATO v1.0 model (constant $D_i$ per layer, initial kernel inventory, absorbing surface) to diffusivities that follow the particle temperature $T(t)$. It covers the Arrhenius law and its input rules (5.1), the finite-volume (FV) treatment with time-dependent conductances (5.2), the effective-time transform and the exact conditions under which it reduces Phase 2.2 to Phase 1 (5.3), the general multilayer case where every layer has its own activation energy (5.4), and the definition of realistic temperature histories (5.5). Section 5.6 lists the verification scripts and quotes their output. Equations are numbered (P22.1) to (P22.59). The open-item subsections added on 2026-11-13 are 5.2.5 (step limit with $D_i(T(t))$), 5.3.7 (start of the release quadrature), 5.3.8 (parent and daughter with different diffusivities) and 5.4.6 (staircase in the stiff regime), with equations (P22.60) to (P22.142).

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
| $M$, $M_0$, $M_{\mathrm{gen}}$, $M_{\mathrm{rel}}$, $M_{\mathrm{dec}}$, $M_{\mathrm{out}}$ | inventories (Section 5.3.3) | $\mathrm{mol}$ |
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

Since the 2019 SI revision (BIPM, 2019, Section 2.2) $N_A = 6.02214076\times10^{23}\ \mathrm{mol^{-1}}$ and $k_B = 1.380649\times10^{-23}\ \mathrm{J\ K^{-1}}$ are exact, so (P22.5) gives $R_g = 8.31446261815324\ \mathrm{J\ mol^{-1}\ K^{-1}}$ exactly (printed in truncated form as 8.314462618). The script confirms that `scipy.constants.R` (SciPy 1.15.3) equals $N_Ak_B$ to machine precision; the defining values are printed in BIPM (2019, Table 1, p. 124) and were read there. TECDOC-978 prints $R = 8.3143\ \mathrm{J\ mol^{-1}\ K^{-1}}$ (IAEA, 1997, p. 435, p. 165 and p. 470). TECDOC-1674 Eq. (10.5a) uses $8.3145$ (IAEA, 2012, p. 532) and BISON uses $8.3145$ (Collin et al., 2021, p. 10, Eq. 71) (both via audit notes 04 and 06).

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

#### 5.2.5 Diffusion step limit for Backward Euler with a temperature history (open item O12)

This section resolves open item O12 as far as a proof allows. Eq. (P21.78) gives the time error of the slowest diffusion mode for constant diffusivities, and step F3 of the pseudocode uses it with a temperature history as a warning only. Below, the limit is generalised to $D_i(T(t))$ with conductances recomputed every step as in (P22.15), for option A (step average) and option E (end of step). Its equations are numbered (P22.60) to (P22.80).

##### 5.2.5.1 Setting and assumptions

The semi-discrete system is (P22.13) with $S=0$:

$$
V\dot C=-A(t)\,C-\lambda VC,\qquad K(t)=V^{-1}A(t),
$$

and Backward Euler is (P22.15). The analysis uses the following assumptions.

- O12-a. The mesh is fixed, so $V$ is constant; $A(t)=A(D(t))$ is assembled from (P22.14) with $D_i(t)>0$ continuous in $t$.
- O12-b. $T(t)$ is linear between knots (P22.21) and every step ends on every knot (rule F1). Each $D_i(T)$ is non-decreasing in $T$, which holds for Arrhenius laws with $Q_{\mathrm{eff},i}\ge0$ (P22.19).
- O12-c. The decay constant $\lambda$ is the same in every layer (assumption A4 of Part 2.2).
- O12-d. No source acts during the interval analysed ($S=0$, as in heating tests and CRP-6 case 1b). With a source, (P21.79) governs the switching error; the slow-mode analysis below with a source is OPEN.
- O12-e. The release is booked with the conductance that entered the matrix (P22.16), so $M^n+M_{\mathrm{rel}}^n+M_{\mathrm{dec}}^n=M_0$ to round-off.
- O12-f. The space discretisation is fixed. All errors below are time-discretisation errors against the semi-discrete solution exact in time; the spatial error is a separate matter (Section 4).

Notation. $\langle x,y\rangle_V=x^{\mathsf T}Vy$. $K(t)$ is self-adjoint in this inner product with eigenvalues $0<\Lambda_1(t)\le\Lambda_2(t)\le\dots$ and $V$-orthonormal eigenvectors $\varphi_m(t)$. The quantities that carry the result are

$$
\begin{aligned}
&\Theta(t)=\int_0^t\big(\Lambda_1(s)+\lambda\big)\,ds,\qquad Z_k=\int_{t_k}^{t_{k+1}}\big(\Lambda_1(t)+\lambda\big)\,dt,\\
&z_k=\Delta t_k\big(\Lambda_1^{(k)}+\lambda\big),\qquad g(z)=z-\ln(1+z),
\end{aligned}
$$

where $\Lambda_1^{(k)}$ is the smallest eigenvalue of $V^{-1}A^{(k)}$, the step matrix of (P22.15). $\Theta$ is the modal effective time of the slowest mode (dimensionless); for constant $T$ it is $(\Lambda_1+\lambda)t$.

##### 5.2.5.2 Two lemmas

**Lemma 1 (exact).** For $z\ge0$,

$$
\tfrac12z^2-\tfrac13z^3\;\le\;g(z)\;\le\;\tfrac12z^2. \tag{P22.60}
$$

Proof. Put $u(z)=\tfrac12z^2-g(z)$ and $v(z)=g(z)-\tfrac12z^2+\tfrac13z^3$. Both vanish at $z=0$, and $u'(z)=z-z/(1+z)=z^2/(1+z)\ge0$, $v'(z)=z/(1+z)-z+z^2=z^3/(1+z)\ge0$. Status: PROVED (script S1).

**Lemma 2 (exact).** Write $\Lambda_1(D)$ for the smallest eigenvalue of $V^{-1}A(D)$ as a function of the layer vector $D=(D_1,\dots,D_5)$. Then $\Lambda_1(sD)=s\Lambda_1(D)$ for $s>0$, $\Lambda_1$ is non-decreasing in each $D_i$, and for any two positive vectors $D,D'$

$$
\min_i\frac{D_i'}{D_i}\;\le\;\frac{\Lambda_1(D')}{\Lambda_1(D)}\;\le\;\max_i\frac{D_i'}{D_i},\qquad\left\lvert \frac{d\ln\Lambda_1}{dt}\right\rvert \le\max_i\left\lvert \frac{d\ln D_i}{dt}\right\rvert =\max_i\frac{Q_{\mathrm{eff},i}(T)\,\lvert \dot T\rvert }{R_gT^2}. \tag{P22.61}
$$

Proof. $x^{\mathsf T}A(D)x=\sum_fG_f(D)(x_j-x_{j+1})^2+G_{N+1/2}(D)x_N^2$. Each face conductance $G_f=A_f/(\delta_f^-/D^-+\delta_f^+/D^+)$ is positively homogeneous of degree one and non-decreasing in each argument, and $G_{N+1/2}\propto D_5$. So the quadratic form is homogeneous of degree one and non-decreasing in each $D_i$, and so is $\Lambda_1=\min_{x\ne0}x^{\mathsf T}Ax/x^{\mathsf T}Vx$. With $s_\pm$ the largest and smallest ratio $D_i'/D_i$, monotonicity gives $\Lambda_1(s_-D)\le\Lambda_1(D')\le\Lambda_1(s_+D)$, and homogeneity gives the first inequality. Taking $D'=D(t+h)$, the logarithm and $h\to0$ gives the second; the last equality is (P22.19). Status: PROVED (script N0a, N0b).

Consequences. On one linear segment $T(t)$ is monotone, so by O12-b and Lemma 2 $\Lambda_1(t)$ is monotone, and its maximum over a step lies at one end of the step. Over a monotone segment $[t_a,t_b]$,

$$
\left\lvert \ln\frac{\Lambda_1(t_b)}{\Lambda_1(t_a)}\right\rvert \le\max_i\left\lvert \ln\frac{D_i(T_b)}{D_i(T_a)}\right\rvert =\max_i\frac{Q_i}{R_g}\left\lvert \frac1{T_a}-\frac1{T_b}\right\rvert \quad(\text{single-branch Arrhenius}). \tag{P22.62}
$$

**Remark (option A eigenvalue, exact).** A weighted harmonic mean $1/(a/x+b/y)$ is concave on the positive quadrant, so by Jensen $G_f(\bar D^{(k)})\ge\frac1{\Delta t_k}\int G_f(D(t))\,dt$ for every face, and $\Lambda_1$ is concave in $A$ (a minimum of linear functionals). Hence for option A

$$
z_k^{A}\ \ge\ Z_k, \tag{P22.63}
$$

with equality in the shared-scaling case. Status: PROVED; the gap is third order in $\Delta t_k$ by Taylor expansion (approximate), and the script measures $\max_k(z_k^A-Z_k)/Z_k=2.3\times10^{-4}$ (N3c).

##### 5.2.5.3 Shared scaling: exact error of every mode

Assume (P22.24), $D_i(t)=\alpha(t)D_i^*$. Then $A(t)=\alpha(t)A^*$ by homogeneity, $K(t)=\alpha(t)K^*$, and the eigenvectors $\varphi_m^*$ of $K^*$ do not move; $\Lambda_m(t)=\alpha(t)\Lambda_m^*$.

*Option A.* The step matrix is $V/\Delta t_n+\bar\alpha_nA^*+\lambda V$ with $\bar\alpha_n\Delta t_n=\Delta\tau_n$. Multiplying (P22.15) by $\Delta t_nV^{-1}$ gives $(I+\Delta\tau_nK^*+\lambda\Delta t_nI)C^{n+1}=C^n$. With $C^n=\sum_mc_m^n\varphi_m^*$ each mode advances by $c_m^{n+1}=c_m^n/(1+Z_{m,n})$, where $Z_{m,n}=\Lambda_m^*\Delta\tau_n+\lambda\Delta t_n=\int_{t_n}^{t_{n+1}}(\Lambda_m+\lambda)\,dt$. The exact semi-discrete mode is $c_m(t)=c_m(0)\exp[-\Lambda_m^*\tau(t)-\lambda t]=c_m(0)\exp(-\sum_kZ_{m,k})$. Dividing,

$$
\ln\frac{c_m^{h}(t_n)}{c_m(t_n)}=\delta_m(t_n):=\sum_{k<n}g(Z_{m,k}),\qquad 0\le\delta_m\le\tfrac12\sum_{k<n}Z_{m,k}^2. \tag{P22.64}
$$

Status: exact identity, PROVED; the bounds follow from (P22.60). Script N1b: nodal Backward Euler against the modal product on variable steps, relative difference $3\times10^{-10}$, consistent with the eigensolver accuracy of about $10^{-10}$ seen in N0a.

*Option E.* Now $z_{m,n}=(\alpha(t_{n+1})\Lambda_m^*+\lambda)\Delta t_n$ and the same argument gives

$$
\delta_m^{E}=\sum_k g(z_{m,k})+L_m,\qquad L_m=\sum_k\big(Z_{m,k}-z_{m,k}\big)=\Lambda_m^*\sum_k\big(\Delta\tau_k-\alpha(t_{k+1})\Delta t_k\big). \tag{P22.65}
$$

$L_m$ is the coefficient lag of option E; its local term is (P22.18). Status: exact identity, PROVED (N2a, N2b agree to $10^{-9}$).

*Release fraction for $\lambda=0$.* By O12-e, $1-F^n=M^n/M_0$ for the scheme and for the exact semi-discrete solution. With $M=\langle\mathbf 1,C\rangle_V$,

$$
\begin{aligned}
&1-F(t)=\sum_mw_m\,a_m(t),\quad a_m=e^{-\Lambda_m^*\tau(t)},\quad w_m=\frac{\langle\mathbf 1,\varphi_m^*\rangle_V\langle\varphi_m^*,C_0\rangle_V}{M_0},\\
&F_h-F=-\sum_mw_m\,a_m^{h}\big(1-e^{-\delta_m}\big).
\end{aligned} \tag{P22.66}
$$

Completeness of the finite basis gives $\sum_mw_m=1$, and Cauchy-Schwarz gives

$$
W:=\sum_m\lvert w_m\rvert \le\frac{\lVert\mathbf 1\rVert_V\lVert C_0\rVert_V}{M_0}=\sqrt{V_{\mathrm{tot}}/V_K}\quad(\text{uniform start in the kernel}). \tag{P22.67}
$$

Status: PROVED. For the AGR-1 geometry of the script $W=2.18$ against the bound 3.38, and $w_1=1.145$ (N1a); the weights of a layered particle with a kernel-only start are not all positive.

*Late-time relative error.* $(1-F_h)/(1-F)=\sum_mw_ma_me^{\delta_m}/\sum_mw_ma_m$. Once the higher modes have decayed (and $Z_{1,k}$ is small enough that $a_m^h/a_1\to0$ as well), the slow mode dominates both sums and

$$
\frac{1-F_h}{1-F}-1\;\to\;e^{\delta_1}-1\;\approx\;\delta_1\;\le\;\tfrac12\sum_kZ_k^2. \tag{P22.68}
$$

Status: approximate (late-time, slow-mode dominance), DEMONSTRATED NUMERICALLY: observed $1.00167\times10^{-2}$ and $1.00018\times10^{-3}$ against $e^{\delta_1}-1$ equal to the same digits (N1d). For constant $T$, $Z_k=(\Lambda_1+\lambda)\Delta t$ and $\tfrac12\sum Z_k^2=\tfrac12(\Lambda_1+\lambda)^2t\,\Delta t$, which is (P21.78) (S2). So (P21.78) is the special case of (P22.68) with $\int(\Lambda_1+\lambda)\,dt$ over one step replaced by its constant-coefficient value.

##### 5.2.5.4 Step rule and its guarantee

**Theorem (shared scaling, option A).** Let $\varepsilon_1>0$ and $\Theta_{\mathrm{end}}=\Theta(t_{\mathrm{end}})$. If every step satisfies

$$
Z_k=\int_{t_k}^{t_{k+1}}\big(\Lambda_1(t)+\lambda\big)\,dt\;\le\;\zeta:=\frac{2\varepsilon_1}{\Theta_{\mathrm{end}}}, \tag{P22.69}
$$

then for every $t_n\le t_{\mathrm{end}}$

$$
0\le\delta_1(t_n)\le\tfrac12\zeta\,\Theta(t_n)\le\varepsilon_1,\qquad 0\le\frac{c_1^{h}(t_n)}{c_1(t_n)}-1\le e^{\varepsilon_1}-1. \tag{P22.70}
$$

Proof. By (P22.64) and (P22.60), $\delta_1\le\tfrac12\sum Z_k^2\le\tfrac12\max_kZ_k\sum Z_k\le\tfrac12\zeta\Theta(t_n)$, and $\Theta(t_n)\le\Theta_{\mathrm{end}}$. Status: PROVED. The bound holds for the retained slow-mode amplitude at all times, for $M$ and for the release rate in the slow-mode regime, and with (P22.68) for the relative error of $1-F$ at late times.

**Fewest steps (exact).** By Cauchy-Schwarz $\sum_kZ_k^2\ge\Theta_{\mathrm{end}}^2/N$ for $N$ steps, so $\tfrac12\sum Z_k^2\le\varepsilon_1$ needs

$$
N\ \ge\ \frac{\Theta_{\mathrm{end}}^2}{2\varepsilon_1}, \tag{P22.71}
$$

with equality for equal $Z_k=\zeta$. Rule (P22.69) therefore equidistributes the error and is optimal for this bound. Numerically (N1d) the rule used 2181 and 21747 steps at $\varepsilon_1=10^{-2}$ and $10^{-3}$, against the minima $\Theta_{\mathrm{end}}^2/(2\varepsilon_1)=2174.4$ and $21743.7$ (calculated with $\Theta_{\mathrm{end}}=6.5945$); the excess comes from the steps cut at knots.

**Implementation (exact).** On a linear segment $\Lambda_1(t)$ is monotone (Lemma 2), so two eigenvalue evaluations give a step that satisfies (P22.69):

$$
\begin{aligned}
&\Delta t^{0}=\frac{\zeta}{\Lambda_1(t_k)+\lambda},\\
&\Delta t_k=\min\left\{\frac{\zeta}{\max\big(\Lambda_1(t_k),\Lambda_1(t_k+\Delta t^0)\big)+\lambda},\ \text{time to the next event}\right\}.
\end{aligned} \tag{P22.72}
$$

Here $t_k+\Delta t^0$ is clipped to the end of the segment. On heating $\Delta t_k\le\Delta t^0$ and $\Lambda_1\le\Lambda_1(t_k+\Delta t^0)$ on the step; on cooling the maximum is at $t_k$; either way $Z_k\le\zeta$. Any further cap (P22.20, growth, events) only shortens the step and keeps the guarantee. The same step also gives $z_k\le\zeta$ for option E.

**Without eigenvalues (exact).** With $\Lambda_1^{\mathrm{ref}}=\Lambda_1(D(T_{\mathrm{ref}}))$, Lemma 2 gives the upper bound $\hat\Lambda_1(T)=\Lambda_1^{\mathrm{ref}}\max_iD_i(T)/D_i(T_{\mathrm{ref}})\ge\Lambda_1(T)$. Using $\hat\Lambda_1$ both in $\hat\Theta_{\mathrm{end}}\ge\Theta_{\mathrm{end}}$ and in the step keeps the guarantee, since $\tfrac12\sum Z_k^2\le\tfrac12\hat\zeta\Theta\le\tfrac12\hat\zeta\hat\Theta=\varepsilon_1$. In the shared case $\hat\Lambda_1=\Lambda_1$ and $\Theta_{\mathrm{end}}=\Lambda_1^*\tau(t_{\mathrm{end}})+\lambda t_{\mathrm{end}}$ follows exactly from (P22.22).

**The present F3 value.** With $\Lambda_{\max}=\max_t\Lambda_1(t)$, the constant step $\Delta t=2\varepsilon_1/[(\Lambda_{\max}+\lambda)^2t_{\mathrm{end}}]$ of step F3 gives $Z_k\le(\Lambda_{\max}+\lambda)\Delta t=:\zeta'$ and $\tfrac12\zeta'\Theta_{\mathrm{end}}\le\tfrac12\zeta'(\Lambda_{\max}+\lambda)t_{\mathrm{end}}=\varepsilon_1$. So the warning value of F3 is a sufficient cap, PROVED in the shared case for option A (the argument of (P22.70)); for option E the lag $L_1$ of (P22.73) enters $\delta_1^E$ through (P22.65), and $\delta_1^E\le\varepsilon_1$ is not proved for this value. Its step count is larger by about $[(\Lambda_{\max}+\lambda)t_{\mathrm{end}}/\Theta_{\mathrm{end}}]^2$. In the ramp-and-hold test this is 24219 steps against 21747 (ratio 1.1137; the estimate gives 1.1138), with error $9.40\times10^{-4}$ at $\varepsilon_1=10^{-3}$ (N1f). Taking $\Lambda_1$ at the lowest temperature is no bound: 5 steps and a relative error of 15.6 (N1g).

**Option E (shared scaling, exact bound).** For a monotone step the mean of $\Lambda_1$ lies between the end values, so $\lvert Z_k-z_k\rvert \le\Delta t_k\lvert \Lambda_1(t_{k+1})-\Lambda_1(t_k)\rvert$. With $\Delta t_k\le\zeta/\Lambda_{1,\max,k}$ from (P22.72) and $1-x\le-\ln x$ for $0<x\le1$,

$$
\begin{aligned}
&\lvert L_1\rvert \le\sum_k\Delta t_k\,\lvert \Delta\Lambda_{1,k}\rvert \le\zeta\,\mathrm{TV}_{[0,t]}(\ln\Lambda_1),\\
&L_1\approx-\tfrac12\zeta\ln\frac{\Lambda_1(t_b)}{\Lambda_1(t_a)}\ \text{on a monotone segment},
\end{aligned} \tag{P22.73}
$$

where TV is the total variation, bounded segment by segment by (P22.62). The leading-order form follows from $Z_k-z_k=-\tfrac12\Delta t_k^2\dot\Lambda_1+O(\Delta t_k^3)$ and $\Delta t_k\approx\zeta/\Lambda_1$ (approximate). Since $\sum z_k=\Theta-L_1$, (P22.65) gives $\lvert \delta_1^E\rvert \le\tfrac12\zeta(\Theta+\lvert L_1\rvert )+\lvert L_1\rvert$, and the option E rule

$$
\zeta_E=\frac{2\varepsilon_1}{\Theta_{\mathrm{end}}+2\,\mathrm{TV}(\ln\Lambda_1)}\quad\Longrightarrow\quad\lvert \delta_1^E\rvert \le\varepsilon_1+\tfrac12\zeta_E^2\,\mathrm{TV}(\ln\Lambda_1). \tag{P22.74}
$$

Status: PROVED (shared scaling). The lag is negative on heating, where option E partly cancels the Backward Euler under-release, and positive on cooling. In the ramp-and-hold test with $\zeta=2\times10^{-2}/\Theta_{\mathrm{end}}$ ($\Theta_{\mathrm{end}}$ of the history in question): $L_1=-4.78\times10^{-3}$ against the bound $1.34\times10^{-2}$ and the leading-order value $-6.71\times10^{-3}$; with the cool-down added, $L_1=-1.99\times10^{-3}$ (N2a, N2d). Option A has $L_1=0$ in the shared case, which supports the recommendation of Section 5.2.3.

##### 5.2.5.5 Absolute error of $F$ at all times ($\lambda=0$, shared scaling)

From (P22.66), with $1-e^{-\delta}\le\min(1,\delta)$,

$$
\lvert F_h(t_n)-F(t_n)\rvert \le\sum_m\lvert w_m\rvert \,a_m^{h}(t_n)\min\!\Big(1,\tfrac12\sum_{k<n}Z_{m,k}^2\Big). \tag{P22.75}
$$

Status: PROVED; checked at every step of both runs (N1e: largest error $1.07\times10^{-3}$ and $1.17\times10^{-4}$). For exactly equal effective steps ($Z_{1,k}=\zeta$, hence $Z_{m,k}=r_m\zeta$ with $r_m=\Lambda_m^*/\Lambda_1^*$), each mode error is $(1+x/n)^{-n}-e^{-x}\le(1+x/n)^{-n}x^2/(2n)$ with $x=r_mn\zeta$, and $x^2(1+x/n)^{-n}$ has its maximum at $x^*=2n/(n-2)$ for $n\ge3$:

$$
\lvert F_h(t_n)-F(t_n)\rvert \le\frac{W\,c_n}{n},\qquad c_n=\tfrac12(x^*)^2\Big(1+\frac{x^*}{n}\Big)^{-n},\quad c_3=\tfrac23,\quad c_n\to2e^{-2}. \tag{P22.76}
$$

Since $1+x^*/n=n/(n-2)$, the constant has the closed form $c_n=2(1-2/n)^{n-2}$ (exact), and $\frac{d}{dn}\ln c_n=\ln(1-2/n)+2/n=h(2/n)$ with $h(u)=\ln(1-u)+u<0$ on $(0,1)$ ($h(0)=0$, $h'(u)=-u/(1-u)$). Status: the bound for each $n\ge3$ is PROVED, and that $c_n$ decreases, so $c_n\le c_3=2/3$ for every $n\ge3$, is PROVED (verification record in Appendix B.5.4, A2; S4 had shown it numerically for $3\le n\le10^6$). This is the smoothing property of Backward Euler: the absolute error of $F$ after $n$ effective steps is $O(1/n)$ whatever the start, while the rule (P22.69) controls the relative error of $1-F$ late in the history. Early-time accuracy is set by the first steps (rule 2 under (P21.79), growth cap in F5), not by (P22.69).

##### 5.2.5.6 Layer-specific activation energies (no shared scaling)

Now the eigenvectors $\varphi_m(t)$ rotate along a ramp and the modes couple. Frozen-coefficient argument: once the solution lies in the instantaneous slow mode, one step multiplies its amplitude by $1/(1+z_k)$ while the exact semi-discrete flow multiplies it by $e^{-Z_k}$ plus coupling terms proportional to $\langle\dot\varphi_1,\varphi_m\rangle_V$. Neglecting the coupling, the per-step logarithmic error is $Z_k-\ln(1+z_k)=g(z_k)+(Z_k-z_k)$, and

$$
\frac{1-F_h}{1-F}-1\approx\exp\Big[\sum_kg(z_k)+\sum_k(Z_k-z_k)+R_{\mathrm{na}}\Big]-1, \tag{P22.77}
$$

with $z_k=z_k^A$ (option A, where $Z_k-z_k^A\le0$ by (P22.63) and is third order by the Taylor remark there) or $z_k=z_k^E$ (option E, lag as in (P22.73)).

Under the step (P22.72) with the instantaneous $\Lambda_1(t)$, the frozen-coefficient exponent $E=\sum_kg(z_k)+\sum_k(Z_k-z_k)=\sum_k\big[Z_k-\ln(1+z_k)\big]$ obeys exact bounds. Both options have $z_k\le\zeta$: for option E as stated under (P22.72), and for option A because $\bar D^{(k)}$ is componentwise at most $D$ at the hotter end of the step, so $\Lambda_1^{(k)}\le\max_{\mathrm{step}}\Lambda_1$ by Lemma 2. With $g(z)\le\tfrac12z^2\le\tfrac12\zeta z$ for $0\le z\le\zeta$, $\sum_kZ_k=\Theta\le\Theta_{\mathrm{end}}$ and $\zeta\le2$ (that is, $\varepsilon_1\le\Theta_{\mathrm{end}}$):

$$
\begin{aligned}
&E^A\le\tfrac12\zeta\Theta+\big(\tfrac12\zeta-1\big)\sum_k\big(z_k^A-Z_k\big)\le\varepsilon_1,\qquad E^E\le\varepsilon_1+\big(1-\tfrac12\zeta\big)L_1,\\
&\sum_kg(z_k)\le\varepsilon_1+\tfrac12\zeta^2\,\mathrm{TV}(\ln\Lambda_1),
\end{aligned} \tag{P22.78}
$$

where $L_1=\sum_k(Z_k-z_k^E)$ satisfies (P22.73) also without shared scaling, because the option E step matrix has $\Lambda_1^{(k)}=\Lambda_1(t_{k+1})$; the last inequality uses $0\le\sum_k(z_k^A-Z_k)\le\zeta\,\mathrm{TV}(\ln\Lambda_1)$ for option A and $\sum_kz_k^E=\Theta-L_1$ for option E. So for option A the frozen-coefficient exponent stays below $\varepsilon_1$ for every history, for option E it does on heating only ($L_1\le0$), and with cooling option E needs $\zeta_E$ of (P22.74). Status of (P22.78): PROVED (bounds on the frozen-coefficient exponent, which is itself only the leading-order model of the error). The first sum alone can exceed $\varepsilon_1$ by at most $\tfrac12\zeta^2\mathrm{TV}(\ln\Lambda_1)$; no case above $\varepsilon_1$ was found (largest $0.975\,\varepsilon_1$ in 15 option E histories, verification record in Appendix B.5.4, B5b). $R_{\mathrm{na}}$ is the non-adiabatic remainder: errors of the fast-mode amplitudes carried into the slow mode by the rotation of $\varphi_1$. It vanishes on holds and in the shared case, and its expected size is first order in the steps taken where $\varphi_1$ rotates, hence $O(\varepsilon_D)$ under (P22.20). Status of (P22.77): approximate, leading order; the structure of $R_{\mathrm{na}}$ is CLAIMED ONLY, and no bound for it is derived here (OPEN).

Numerical evidence (N3, synthetic layer-specific $Q$ of 209, 50, 222, 514 and 222 $\mathrm{kJ\ mol^{-1}}$, ramp 1250 to 1600 $^\circ$C at 50 $\mathrm{K\ h^{-1}}$ then hold to 100 h, $\Theta_{\mathrm{end}}=6.555$, Radau reference validated against the exact shared-scaling solution to $3\times10^{-10}$):

| $\varepsilon_1$ | $\varepsilon_D$ (P22.20) | option | observed | (P22.77) with $R_{\mathrm{na}}=0$ | $R_{\mathrm{na}}$ |
|------------|--------------|----------|----------------------|----------------------|--------------------|
| $10^{-2}$ | none | A | $1.053\times10^{-2}$ | $1.001\times10^{-2}$ | $5.2\times10^{-4}$ |
| $10^{-2}$ | none | E | $6.41\times10^{-3}$ | $5.71\times10^{-3}$ | $7.0\times10^{-4}$ |
| $10^{-2}$ | 0.1 | A | $1.021\times10^{-2}$ | $1.000\times10^{-2}$ | $2.1\times10^{-4}$ |
| $10^{-2}$ | 0.025 | A | $9.997\times10^{-3}$ | $9.942\times10^{-3}$ | $5.4\times10^{-5}$ |
| $10^{-2}$ | 0.025 | E | $9.080\times10^{-3}$ | $9.012\times10^{-3}$ | $6.8\times10^{-5}$ |
| $3\times10^{-3}$ | none | A | $3.341\times10^{-3}$ | $3.000\times10^{-3}$ | $3.4\times10^{-4}$ |
| $3\times10^{-3}$ | 0.1 | A | $3.176\times10^{-3}$ | $3.000\times10^{-3}$ | $1.8\times10^{-4}$ |
| $3\times10^{-3}$ | 0.025 | A | $3.048\times10^{-3}$ | $2.994\times10^{-3}$ | $5.4\times10^{-5}$ |
| $3\times10^{-3}$ | 0.025 | E | $2.304\times10^{-3}$ | $2.234\times10^{-3}$ | $7.0\times10^{-5}$ |

Table: 5.9 Relative error of $1-F$ at $t_{\mathrm{end}}$ with layer-specific activation energies (nine of the twelve runs of script N3; all twelve are in the script output, Appendix A.2).

$R_{\mathrm{na}}$ is positive in all twelve runs. Between $\varepsilon_D=0.1$ and $0.025$ it falls by factors of 2.9 to 4.2 (first order predicts 4), and at $\varepsilon_D=0.025$ it is the same for both $\varepsilon_1$ to within 2.3 % (N3d, N3e), so it is set by the ramp steps and not by the Z rule. In this history $R_{\mathrm{na}}/\varepsilon_D$ lies between $1.8\times10^{-3}$ and $2.9\times10^{-3}$; that constant depends on the history and the mesh and is not a general value. With $\varepsilon_D=0.025$, (P22.77) without $R_{\mathrm{na}}$ predicts the observed error to within 3.2 % (N3f).

Strongest rigorous statement in the general case. $K^{(k)}=V^{-1}A^{(k)}+\lambda I$ is self-adjoint and positive in $\langle\cdot,\cdot\rangle_V$, so $\lVert(I+\Delta t_kK^{(k)})^{-1}\rVert_V\le1$ and the global error obeys

$$
\lVert e^n\rVert_V\le\sum_{k<n}\lVert\rho_k\rVert_V,\qquad \lvert F_h-F\rvert \le\frac{\sqrt{V_{\mathrm{tot}}}}{M_0}\lVert e^n\rVert_V\quad(\lambda=0), \tag{P22.79}
$$

with $\rho_k=C(t_k)-(I+\Delta t_kK^{(k)})C(t_{k+1})$ the local residual of the exact semi-discrete solution. Status: PROVED, but it bounds the absolute error only, needs the exact solution, and does not reproduce the slow-mode structure, so it gives no computable relative criterion. A rigorous relative bound for layer-specific $Q$ is OPEN.

##### 5.2.5.7 Decay and the release fraction

With $\lambda>0$ (shared scaling, option A), (P22.64) holds with $Z_{m,k}=\Lambda_m^*\Delta\tau_k+\lambda\Delta t_k$, so the rule (P22.69) with $\Theta$ including $\lambda t$ controls the relative error of the retained inventory $M$ and of the slow-mode release rate: N4b observes $1.00325\times10^{-2}$ and $1.00032\times10^{-3}$, equal to $e^{\delta_1}-1$. This is the $(\Lambda_1+\lambda)$ of (P21.78).

The release fraction behaves differently. Per mode, $F_m+D_m+a_m=1$ holds for the exact solution and for the scheme ($D_m$ the decayed fraction), so

$$
\begin{aligned}
F_h-F&=-\sum_mw_m\big(e_{a,m}+e_{D,m}\big),\\
e_{D,m}&=\underbrace{\lambda\sum_n\Delta t_n\big(a_m^{h,n+1}-a_m^{n+1}\big)}_{\in[0,\ \lambda\sum_n\Delta t_na_m^{h,n+1}\min(1,\delta_m^{n+1})]}\\
&\quad+\underbrace{\lambda\Big(\sum_n\Delta t_na_m^{n+1}-\int_0^ta_m\,dt\Big)}_{\in[-\zeta(1-a_m(t)),\ 0]},
\end{aligned} \tag{P22.80}
$$

where $e_{a,m}=a_m^h-a_m\in[0,a_m^h\min(1,\delta_m)]$. The rectangle bound uses that $a_m$ decreases and that $\lambda\Delta t_n\le Z_{1,n}\le\zeta$ under the rule. Status: identity and the three bounds PROVED (N4c). For constant coefficients the two parts of $e_D$ cancel to leading order, and exactly $F_h-F=-p\,(a_h-a)$ with $p=\Lambda/(\Lambda+\lambda)$ (S5). On the ramp this cancellation does not carry over: with a synthetic half-life of 30 h the observed $F_h-F$ is $5.89\times10^{-5}$ and $5.91\times10^{-6}$ for $\varepsilon_1=10^{-2}$ and $10^{-3}$, about $-50$ times the constant-coefficient value $-p_1w_1a_1(e^{\delta_1}-1)$, first order in $\zeta$ (ratio 9.97, N4d), and below the a-posteriori bound from (P22.80), which equals $\zeta W$ to three digits here. The cause of the imbalance was not isolated. An a-priori bound of the release-fraction error with decay that is sharper than $O(\zeta W)$ is OPEN; the absolute error stays far below $\varepsilon_1$ in the test.

##### 5.2.5.8 Result

For Backward Euler with diffusivities $D_i(T(t))$ the diffusion step limit becomes a limit on the integral of the slowest rate over each step:

$$
\int_{t_k}^{t_{k+1}}\big(\Lambda_1(t)+\lambda\big)\,dt\le\frac{2\varepsilon_1}{\Theta_{\mathrm{end}}},\qquad\Theta_{\mathrm{end}}=\int_0^{t_{\mathrm{end}}}\big(\Lambda_1(t)+\lambda\big)\,dt,
$$

which reduces to (P21.78) for constant $T$ and needs the fewest steps for its bound.

| Statement | Label |
|---|---|
| (P22.60) to (P22.63): bounds of $g$, homogeneity and monotonicity of $\Lambda_1$, rate bound, $z^A\ge Z$ | PROVED |
| (P22.64), (P22.65), (P22.66): exact modal error, options A and E, release fraction ($\lambda=0$), shared scaling | PROVED |
| (P22.69), (P22.70), (P22.72): the step rule bounds $\delta_1\le\varepsilon_1$ at all times, shared scaling, option A | PROVED |
| (P22.71): equal $Z_k$ minimise the step count | PROVED |
| (P22.73), (P22.74): option E lag bound and rule | PROVED (shared); leading-order lag approximate |
| (P22.68): relative error of $1-F$ equals $e^{\delta_1}-1$ late in the history | approximate; DEMONSTRATED NUMERICALLY |
| (P22.75), (P22.76): absolute error of $F$ at all times, $c_n=2(1-2/n)^{n-2}\le2/3$ | PROVED |
| (P22.77): layer-specific $Q$, frozen-coefficient prediction | approximate (leading order); DEMONSTRATED NUMERICALLY on one history |
| (P22.78): frozen-coefficient exponent $\le\varepsilon_1$ (option A, any history; option E on heating) | PROVED |
| $R_{\mathrm{na}}=O(\varepsilon_D)$ | CLAIMED ONLY (structure); DEMONSTRATED NUMERICALLY on one history; bound OPEN |
| (P22.79): contractive absolute bound | PROVED, not computable a priori |
| (P22.80): release fraction with decay | identity and bounds PROVED; sharp a-priori bound OPEN |

Table: 5.10 Status of the O12 results.

##### 5.2.5.9 Limitations

- The guarantee holds for one quantity, the slow-mode amplitude, and through it for $M$ and late-time $1-F$. Early-time errors of $F$ come from the first steps and the fast modes; (P22.75) and (P22.76) bound them but the rule does not target them.
- The proof needs shared scaling. For layer-specific $Q$ the rule controls the frozen-coefficient part only, and $R_{\mathrm{na}}$ has to be kept small by (P22.20) and checked a posteriori, for example by step doubling on ramp segments (Section 5.2.4).
- $\Lambda_1(T)$ requires either one symmetric tridiagonal eigenvalue per evaluation or the conservative bound $\hat\Lambda_1$ of Lemma 2; with $\hat\Lambda_1$ the step count can be much larger when the layer activation energies differ strongly.
- $S\ne0$ is not covered (O12-d), and multi-branch laws need $Q_{\mathrm{eff},i}\ge0$ for the monotonicity used in (P22.72).
- All numbers come from synthetic parameters on the AGR-1 nominal geometry with 20 cells per layer; they illustrate the formulas and are not material predictions.

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

- E1. One shared factor $\alpha(t)$ multiplies every layer diffusivity (P22.24); the ratios $D_i/D_j$ never change. For a parent-daughter pair, E1 for one clock means N1 of Section 5.3.8 and is not sufficient; the decay coupling adds N2.
- E2. $T$ is uniform in $r$ (A1); a radial temperature profile makes $\alpha$ depend on $r$.
- E3. $\lambda$ is uniform (A4); a source $S$ of any shape is allowed and enters through (P22.30) or (P22.36).
- E4. The surface is absorbing, or a Robin coefficient obeys (P22.38).
- E5. No temperature-dependent partition coefficients at interfaces (A2).

With $S = 0$ and $\lambda = 0$ the result is v1.0 evaluated at $\tau(t)$, closed form apart from the series truncation. With $\lambda > 0$ the inventory stays closed form and the release needs the quadrature (P22.35). With $S \ne 0$ each mode needs the quadrature in (P22.36) unless $S$ and $\alpha$ make it elementary. If E1 fails, any single $\tau$ is an approximation of unknown accuracy and Section 5.4 applies. For TECDOC-978 data E1 fails in every multilayer case (for Cs, FRG: kernel 209 and 362, PyC 222, SiC 125 and 514 $\mathrm{kJ\ mol^{-1}}$; the FRG buffer uses a constant $10^{-8}\ \mathrm{m^2\ s^{-1}}$ with zero activation energy, IAEA, 1997, p. 440), so in STRATO the multilayer effective-time solution serves as an exact verification case with synthetic shared-$Q$ data, while the single-layer case (bare kernel) is exact with real data.

Numerical demonstration (`sec5_fv_shared_alpha.py`, synthetic shared $Q = 300\ \mathrm{kJ\ mol^{-1}}$, AGR-1 nominal radii, ramp 1250 to 1600 $^\circ$C at 50 $\mathrm{K\ h^{-1}}$ then hold): the semi-discrete FV solution evaluated at $\tau(t)$ (exact in time through its eigen-decomposition) converges to the independent series at $\tau(t)$ with observed spatial order 2.00 and a difference of $5.4\times10^{-7}$ at 160 cells per layer; backward Euler run in real time with recomputed conductances converges to that semi-discrete solution at first order (options E and A); with $t_{1/2} = 10\ \mathrm{h}$, a stiff real-time integration matches $e^{-\lambda t}\hat M(\tau(t))$ to $1.6\times10^{-12}$ and (P22.35) to $2.8\times10^{-12}$ (relative to $M_0$).

#### 5.3.7 Start of the release quadrature with decay (task R24)

This subsection answers the accuracy note left open by the B7 review in block R2.4 step 3 of the pseudocode: how to start the quadrature of (P22.35) near $t = 0$, where the truncated modal series is not valid and, for a bare kernel with a uniform start, $\hat M_\mathrm{rel}$ grows like $\sqrt\tau$. Its equations are numbered (P22.81) to (P22.99).

**Result in one paragraph.** The quadrature route (P22.35) and the balance route $M_\mathrm{rel} = M_0 - M - M_\mathrm{dec}$ are the same expression, so the balance route has the same start problem (PROVED, (P22.81)). Split the interval at $t_0$. On $[0, t_0]$ integrate each mode in closed form, or for a bare kernel use the image (Booth-type) short-time form; on $[t_0, t]$ use 8-point Gauss-Legendre on pieces whose effective-time ratio $\tau(b)/\tau(a)$ is at most 2. The error committed on $[0, t_0]$ is bounded by $\lambda M_0\min(t_0\bar W_N, \bar V_N/\alpha_0)$ for the modal start with constant $\alpha = \alpha_0$ (PROVED, (P22.93)); for varying $\alpha$ the same bound with $\alpha_-$ in place of $\alpha_0$ holds when each $E_n$ is integrated exactly, and otherwise the bracket (P22.94) is used for $c_0\ge0$. Here $\bar W_N$ and $\bar V_N$ are computable tail bounds that need only Bessel's inequality and an energy identity (PROVED, (P22.90)); for a homogeneous sphere $\bar V_N \le 2R^2/(\pi^4D^*N^3)$. For a bare kernel with constant $\alpha$ on the start piece the image form makes the start error at most $9.4\times10^{-20}M_0$ with 12 modes (PROVED; the image form itself is PROVED by a Laplace-transform argument in the verification record in Appendix B.5.5, F3); with a ramp on the start piece the extra Gauss-Legendre error of (P22.87) is DEMONSTRATED NUMERICALLY only. The graded Gauss-Legendre error is below $2.72\times10^{-12}\lambda(t - t_0)(1 + W)M_0$ on pieces of constant $\alpha$ (PROVED). One case stays OPEN: a layered particle with a long cold start, where the standard series carries a genuine spurious release.

##### Assumptions

- R24-A1. Conditions E1 to E5 of Section 5.3.6 hold with the absorbing surface $c(R) = 0$ (the Dirichlet branch of E4, which is the condition v1.0 and Section 5.3.5 use), $S = 0$, $\lambda > 0$, and $\alpha(s) > 0$ is piecewise continuous with $\tau(s) = \int_0^s\alpha\,ds'$. The Robin branch of E4 is excluded: the energy identity (P22.89) and the image form (P22.84) both use $c(R) = 0$ (verification record in Appendix B.5.5, F1).
- R24-A2. The eigenfunctions $\phi_n$ of $L^*$ are normalised in the weight $r^2$: $\langle f, g\rangle = \int_0^R r^2fg\,dr$, $\langle\phi_n, \phi_m\rangle = \delta_{nm}$. With $c_n = \langle c_0, \phi_n\rangle$ and $e_n = \langle 1, \phi_n\rangle$ the v1.0 weights are $w_n = (4\pi/M_0)\,c_ne_n$ (for a kernel-only start this is the v1.0 formula $w_n = 3I_{1,n}I_{n,n}/(R_1^3N_n)$ in `sturm_liouville.rs`).
- R24-A3. The computed modes form a set $\mathcal N$ of $N$ modes; $\mathcal N^c$ denotes the omitted modes. Where stated, the coverage assumption of v1.0 is used: every mode with $\Lambda_n^* < \Lambda_\mathrm{cov}$ is in $\mathcal N$ (R24-A3c). The tail bounds below do not need R24-A3c; the late-piece bound does.
- R24-A4. Only for the bracket (P22.94): $c_0 \ge 0$.
- Notation: $W_N = \sum_{\mathcal N^c}\lvert w_n\rvert$, $V_N = \sum_{\mathcal N^c}\lvert w_n\rvert/\Lambda_n^*$, $W = \sum_{\text{all }n}\lvert w_n\rvert$. Errors are quoted per $M_0$ unless $M_0$ is written.

##### Step 1. The two routes are one expression

With $S = 0$ and no decay in $\tau$-space, $\hat M_\mathrm{rel}(\tau) = M_0 - \hat M(\tau)$. Insert this in (P22.35) and use $M(s) = e^{-\lambda s}\hat M(\tau(s))$ from (P22.33):

$$
\begin{aligned}
e^{-\lambda t}\hat M_\mathrm{rel}(\tau(t)) + \lambda\int_0^te^{-\lambda s}\hat M_\mathrm{rel}(\tau(s))\,ds &= M_0 - M(t) - \lambda\int_0^tM(s)\,ds\\
&= M_0 - M(t) - M_\mathrm{dec}(t).
\end{aligned} \tag{P22.81}
$$

The middle step uses $e^{-\lambda t}M_0 + \lambda M_0\int_0^te^{-\lambda s}ds = M_0$. Status: exact, PROVED (SymPy, check C1). Consequence: when both routes use the same truncated $\hat M^{(N)}$ and the same quadrature they give the same number up to rounding. The balance integrand $M(s) = M_0e^{-\lambda s} - e^{-\lambda s}\hat M_\mathrm{rel}(\tau(s))$ differs from the release integrand by the smooth term $M_0e^{-\lambda s}$ only, so it has the same $\sqrt s$ start: for a bare kernel with $\alpha = 1$, $(M_0e^{-\lambda s} - M)/(M_0\sqrt s) \to 6/\sqrt\pi$ (check C1: 3.385108 at $s = 10^{-10}$ against 3.385138). The phrase "smooth integrand" in the B7 note of block R2.4 is therefore CORRECTED. The choice between the routes is therefore immaterial for accuracy, and the remaining steps treat the start of the quadrature for both.

##### Step 2. Error split

For $0 < t_0 \le t$:

$$
\begin{aligned}
M_\mathrm{rel}(t) &= e^{-\lambda t}\hat M_\mathrm{rel}(\tau(t)) + I_0 + I_1,\\
I_0 &= \lambda\int_0^{t_0}e^{-\lambda s}\hat M_\mathrm{rel}(\tau(s))\,ds,\qquad I_1 = \lambda\int_{t_0}^te^{-\lambda s}\hat M_\mathrm{rel}(\tau(s))\,ds.
\end{aligned} \tag{P22.82}
$$

The start piece $I_0$ carries a method error (truncation, image remainder or bracket width); the boundary term and $I_1$ carry series truncation, and $I_1$ also carries quadrature error. Steps 3 to 6 bound these separately and Step 7 adds them.

##### Step 3. Modal start piece

Insert $\hat M_\mathrm{rel}(\tau) = M_0\bigl(1 - \sum_nw_ne^{-\Lambda_n^*\tau}\bigr)$ (v1.0 series; the leading 1 is $\sum_nw_n$, which rests on the completeness proof of Section 3.10) and integrate each mode:

$$
\begin{aligned}
&I_0 = M_0\Bigl[(1 - e^{-\lambda t_0}) - \lambda\sum_nw_nE_n(t_0)\Bigr],\qquad E_n(t_0) = \int_0^{t_0}e^{-\lambda s - \Lambda_n^*\tau(s)}\,ds,\\
&\alpha = \alpha_0:\ E_n = \frac{1 - e^{-\kappa_nt_0}}{\kappa_n},\ \kappa_n = \lambda + \alpha_0\Lambda_n^*.
\end{aligned} \tag{P22.83}
$$

Status: exact for each mode. The modal sum of $E_n$ converges like $\sum\lvert w_n\rvert/\Lambda_n^*$, which is the reason the integrated start converges fast while the pointwise series at $\tau \to 0$ does not.

##### Step 4. Image (Booth-type) start for a bare kernel

For one homogeneous region with a uniform start, $u = D^*\tau/R^2$, $F = \hat M_\mathrm{rel}/M_0$, the image form of the sphere release is

$$
\begin{aligned}
F(u) &= 6\sqrt u\Bigl[\pi^{-1/2} + 2\sum_{k\ge1}\mathrm{ierfc}\bigl(k/\sqrt u\bigr)\Bigr] - 3u,\\
\mathrm{ierfc}(x) &= \int_x^\infty\mathrm{erfc}(y)\,dy = \frac{e^{-x^2}}{\sqrt\pi} - x\,\mathrm{erfc}(x).
\end{aligned} \tag{P22.84}
$$

Status: exact, PROVED (Laplace-transform argument in the verification record in Appendix B.5.5, F3; independent check V2). It equals the modal series $1 - (6/\pi^2)\sum n^{-2}e^{-n^2\pi^2u}$ to $2.7\times10^{-37}$ for $u \in [0.002, 1]$ (check C2) and to $2.7\times10^{-45}$ for $u \in [10^{-4}, 3]$ (check V2f). No printed source was opened for it.

Remainder bound. Let $\rho_B(u) = F(u) - (6\sqrt{u/\pi} - 3u) = 12\sqrt u\sum_k\mathrm{ierfc}(k/\sqrt u) \ge 0$. For $y > 0$, $\mathrm{erfc}(y) = (2/\sqrt\pi)\int_y^\infty e^{-z^2}dz \le (2/\sqrt\pi)\int_y^\infty(z/y)e^{-z^2}dz = e^{-y^2}/(y\sqrt\pi)$. Integrating once more, $\mathrm{ierfc}(x) \le \pi^{-1/2}\int_x^\infty e^{-y^2}y^{-1}dy \le \pi^{-1/2}x^{-2}\int_x^\infty ye^{-y^2}dy = e^{-x^2}/(2\sqrt\pi x^2)$. With $x = k/\sqrt u$ and $\sum_ke^{-k^2/u}k^{-2} \le e^{-1/u}\pi^2/6$:

$$
0 \le \rho_B(u) \le \pi^{3/2}u^{3/2}e^{-1/u},\qquad u_B = \tfrac1{40}:\ \rho_B \le 9.35\times10^{-20}. \tag{P22.85}
$$

Status: PROVED (with (P22.84)); check C2 finds the ratio remainder/bound at most 0.603, check V3a at most 0.607 on $u \in [10^{-3}, 100]$. Two corollaries: the bound is increasing in $u$, and since $\rho_B \le 3u$ for $u \le \pi/36$ while $F \le 1 \le 6\sqrt{u/\pi}$ for $u \ge \pi/36$, $F(u) \le 6\sqrt{u/\pi}$ for every $u > 0$ (PROVED).

Start-piece integral. Choose $t_B$ with $u(t_B) = u_B$. For constant $\alpha = \alpha_0$ on $[0, t_B]$, $u(s) = u_0s$ with $u_0 = \alpha_0D^*/R^2$, and

$$
\begin{aligned}
I_0^B &= M_0\Bigl[6\sqrt{\frac{u_0}{\pi\lambda}}\,\gamma\bigl(\tfrac32, \lambda t_B\bigr) - \frac{3u_0}{\lambda}\bigl(1 - e^{-\lambda t_B}(1 + \lambda t_B)\bigr)\Bigr],\\
\lvert I_0 - I_0^B\rvert &\le (1 - e^{-\lambda t_B})\,\pi^{3/2}u_B^{3/2}e^{-1/u_B}M_0,
\end{aligned} \tag{P22.86}
$$

with $\gamma$ the lower incomplete gamma function. The closed form is exact; the error bound is PROVED. For variable $\alpha$ on $[0, t_B]$ substitute $s = v^2$:

$$
I_0^B = \lambda M_0\int_0^{\sqrt{t_B}}2v\,e^{-\lambda v^2}\Bigl[6\sqrt{u(v^2)/\pi} - 3u(v^2)\Bigr]dv, \tag{P22.87}
$$

whose integrand is smooth when $\alpha(0) > 0$ and $\alpha$ is smooth on $[0, t_B]$, because $u(v^2)/v^2$ is then smooth and positive; a temperature breakpoint inside $[0, t_B]$ must be a panel edge in $v$. Gauss-Legendre with 16 points on the whole interval is used. The image remainder in (P22.87) obeys the bound of (P22.86) unchanged; the Gauss-Legendre error of the 16-point rule has no proved bound. Status: DEMONSTRATED NUMERICALLY (check C6, exponential ramp $\alpha = e^{5s}$: 8 and 16 points differ by $3.5\times10^{-18}$). The choice $u_B = 1/40$ also fixes the mode count: the late pieces need $\Lambda_\mathrm{cov}\tau(t_B) \ge 40$, so $\Lambda_\mathrm{cov} \ge 1600D^*/R^2$, which holds with $N = 12$ modes since $(N + 1)^2\pi^2 \ge 1600$ needs $N \ge 11.7$.

##### Step 5. Computable tail bounds for any layered particle

Bessel's inequality gives, for any $f \in L^2(r^2dr)$ and any orthonormal set,

$$
\sum_{\mathcal N^c}f_n^2 \le P_N(f) := \langle f, f\rangle - \sum_{\mathcal N}f_n^2. \tag{P22.88}
$$

Energy identity. Let $u = (-L^*)^{-1}f$, that is $-r^{-2}(r^2D^*u')' = f$ with $u(R) = 0$, a regular centre and continuity of $u$ and $D^*u'$ at interfaces. With $Q_f(r) = \int_0^r\rho^2f\,d\rho$ one integration gives $r^2D^*u' = -Q_f$, and one integration by parts ($Q_f(0) = 0$, $u(R) = 0$) gives

$$
\begin{aligned}
\langle f, (-L^*)^{-1}f\rangle &= \int_0^R\frac{Q_f(r)^2}{r^2D^*(r)}\,dr \ \ge\ \sum_{\text{all }n}\frac{f_n^2}{\Lambda_n^*},\\
H_N(f) &:= \int_0^R\frac{Q_f^2}{r^2D^*}\,dr - \sum_{\mathcal N}\frac{f_n^2}{\Lambda_n^*} \ \ge\ \sum_{\mathcal N^c}\frac{f_n^2}{\Lambda_n^*}.
\end{aligned} \tag{P22.89}
$$

Proof of the inequality: with the energy product $a(v, g) = \int_0^Rr^2D^*v'g'\,dr$, the functions $\phi_n/\sqrt{\Lambda_n^*}$ are orthonormal ($a(\phi_n, \phi_m) = \Lambda_n^*\delta_{nm}$) and $a(u, \phi_n/\sqrt{\Lambda_n^*}) = \langle f, \phi_n\rangle/\sqrt{\Lambda_n^*}$; Bessel's inequality in the energy product gives $\sum f_n^2/\Lambda_n^* \le a(u, u) = \langle f, u\rangle$. Status: PROVED; equality holds when the set is complete, but the bound does not need completeness. SymPy confirms the identity for two layers (check C3: $25/288$ for $f = 1$ and $11/1440$ for the kernel indicator, $R_1 = 1/2$, $D^* = (1, 1/4)$).

Cauchy-Schwarz on $\lvert w_n\rvert = (4\pi/M_0)\lvert c_ne_n\rvert$ then bounds the two tails that matter:

$$
\begin{aligned}
W_N &\le \bar W_N := \frac{4\pi}{M_0}\sqrt{P_N(c_0)\,P_N(1)},\\
V_N &\le \bar V_N := \frac{4\pi}{M_0}\sqrt{H_N(c_0)\,H_N(1)},\\
W &\le \frac{4\pi}{M_0}\sqrt{\langle c_0, c_0\rangle\langle1, 1\rangle}.
\end{aligned} \tag{P22.90}
$$

For a kernel-only unit start ($M_0 = 4\pi R_1^3/3$) in layers $R_0 = 0 < R_1 < \dots < R_5 = R$ the totals are closed form:

$$
\begin{aligned}
\langle c_0, c_0\rangle &= \frac{R_1^3}{3},\quad \langle1, 1\rangle = \frac{R^3}{3},\\
\int\frac{Q_{c_0}^2}{r^2D^*} &= \frac{R_1^5}{45D_1^*} + \frac{R_1^6}{9}\sum_{i=2}^5\frac{R_{i-1}^{-1} - R_i^{-1}}{D_i^*},\\
\int\frac{Q_1^2}{r^2D^*} &= \sum_{i=1}^5\frac{R_i^5 - R_{i-1}^5}{45D_i^*},\quad W \le \Bigl(\frac{R}{R_1}\Bigr)^{3/2}.
\end{aligned} \tag{P22.91}
$$

Status: exact. A missed root inflates $P_N$ and $H_N$, so the bounds stay valid and the size of $H_N$ doubles as a missed-root test. In floating point the subtraction in $H_N$ loses about $2.2\times10^{-16}$ times the total, which limits the usable bound to roughly $10^{-14}$ relative to $\int Q^2/(r^2D^*)$.

Homogeneous sphere ($c_0 = 1$, $w_n = 6/(n^2\pi^2)$, $\Lambda_n^* = n^2\pi^2D^*/R^2$): Cauchy-Schwarz is an equality and the integral test gives

$$
W_N \le \frac{6}{\pi^2N},\qquad V_N \le \frac{2R^2}{\pi^4D^*N^3},\qquad \sum_nw_n/\Lambda_n^* = \frac{R^2}{15D^*}. \tag{P22.92}
$$

Status: PROVED (check C3 confirms them against Hurwitz zeta values at $N = 10$, 100, 1000).

##### Step 6. Bounds on the start piece and on the late pieces

**Start piece, constant $\alpha$.** If $\alpha = \alpha_0$ on $[0, t_0]$ and $I_0^{(N)}$ is (P22.83) summed over $\mathcal N$, then $I_0 - I_0^{(N)} = -\lambda M_0\sum_{\mathcal N^c}w_nE_n$ and $E_n \le \min(t_0, 1/(\alpha_0\Lambda_n^*))$, so

$$
\lvert I_0 - I_0^{(N)}\rvert \le \lambda M_0\min\Bigl(t_0W_N,\ \frac{V_N}{\alpha_0}\Bigr) \le \lambda M_0\min\Bigl(t_0\bar W_N,\ \frac{\bar V_N}{\alpha_0}\Bigr). \tag{P22.93}
$$

Status: PROVED. For the sphere the bound is sharp: check C4 finds ratios error/bound from 0.995 to 1.000 for $N = 10$, 100, 1000 and $\alpha_0 = 1$, 0.1. At the v1.0 validity time $t_0 = 40/(\alpha_0\Lambda_{N+1}^*)$ the second entry wins, and the bound is about 120 times smaller than the B7 estimate $\lambda t_0W_N$ (ratio $240/2$ from (P22.92); check C4 finds at least 109.8). The B7 estimate remains a valid upper bound. The bound does not depend on $t_0$ once $\alpha_0\Lambda_{N+1}^*t_0 \gg 1$, so with constant $\alpha$ the start piece can run to the first breakpoint.

**Start piece, variable $\alpha$.** Let $0 < \alpha_- \le \alpha(s) \le \alpha_+$ on $[0, t_0]$ and $c_0 \ge 0$. The weak maximum principle (multiply the $\tau$-space equation by $r^2\hat c^-$, $\hat c^- = \max(-\hat c, 0)$, and integrate: $\tfrac{d}{d\tau}\int r^2(\hat c^-)^2 = -2\int r^2D^*\lvert\partial_r\hat c^-\rvert^2 \le 0$) gives $\hat c \ge 0$, hence $\partial_r\hat c(R) \le 0$ and $\hat M_\mathrm{rel}$ is non-decreasing in $\tau$. Since $\alpha_-s \le \tau(s) \le \alpha_+s$, $I_0$ lies between $J(\alpha_-)$ and $J(\alpha_+)$, where $J(a)$ is (P22.83) with constant $\alpha = a$, and $J$ is non-decreasing in $a$. Each $J(a)$ carries the truncation bound of (P22.93) with $\alpha_0$ replaced by $a \ge \alpha_-$. Taking the midpoint:

$$
\Bigl\lvert I_0 - \tfrac12\bigl[J^{(N)}(\alpha_-) + J^{(N)}(\alpha_+)\bigr]\Bigr\rvert \le \tfrac12\bigl\lvert J^{(N)}(\alpha_+) - J^{(N)}(\alpha_-)\bigr\rvert + 2\lambda M_0\min\Bigl(t_0\bar W_N,\ \frac{\bar V_N}{\alpha_-}\Bigr). \tag{P22.94}
$$

Status: PROVED under R24-A4. Check C6 (bare kernel, $\alpha = e^{5s}$, $N = 200$, $t_0 = \tau^{-1}(40/\Lambda_{201}^*)$): error $2.66\times10^{-9}$ against a bound of $5.37\times10^{-9}$, half-width $2.8\times10^{-10}$.

**Late pieces, truncation.** Under R24-A3c every omitted mode has $\Lambda_n^* \ge \Lambda_\mathrm{cov}$. For $s \ge t_0$ the pointwise error of $\hat M_\mathrm{rel}^{(N)}$ is at most $M_0e^{-\Lambda_\mathrm{cov}\tau(t_0)}W_N$. The boundary term carries the factor $e^{-\lambda t}$ and $I_1$ carries $\lambda\int_{t_0}^te^{-\lambda s}ds = e^{-\lambda t_0} - e^{-\lambda t}$, so together

$$
\begin{aligned}
\bigl\lvert\text{truncation of }e^{-\lambda t}\hat M_\mathrm{rel}(\tau(t)) + I_1\bigr\rvert &\le e^{-\lambda t_0}e^{-\Lambda_\mathrm{cov}\tau(t_0)}\,\bar W_NM_0\\
&\le e^{-40}\bar W_NM_0\quad\text{when }\Lambda_\mathrm{cov}\tau(t_0) \ge 40.
\end{aligned} \tag{P22.95}
$$

Status: PROVED under R24-A3c ($e^{-40} = 4.2\times10^{-18}$).

**Late pieces, quadrature.** On a piece $[a, b]$ with constant $\alpha_k$, $\tau$ is affine in $s$ and $\hat M_\mathrm{rel}(\tau)$ is analytic for $\mathrm{Re}\,\tau > 0$ with $\lvert\hat M_\mathrm{rel}\rvert \le M_0(1 + W)$ there, since $\sum\lvert w_n\rvert$ converges by (P22.90). The singular line $\mathrm{Re}\,\tau = 0$ sits at $\mathrm{Re}\,s = s^* := a - \tau(a)/\alpha_k$. Let $q = \tau(b)/\tau(a)$. The Bernstein ellipse with foci $a$, $b$ whose left vertex is $s^*$ has parameter

$$
\rho = \frac{\sqrt q + 1}{\sqrt q - 1}\qquad (q = 2:\ \rho = 3 + 2\sqrt2 = 5.83). \tag{P22.96}
$$

For $g$ analytic in the open ellipse $E_\rho$ (mapped to $[-1, 1]$) with $\lvert g\rvert \le M$ there, the Chebyshev coefficients satisfy $\lvert a_k\rvert \le 2M\rho^{-k}$ (Cauchy integral over $\lvert z\rvert = r < \rho$ after the Joukowski map $x = (z + z^{-1})/2$, then $r \to \rho$). The $n$-point Gauss-Legendre rule is exact up to degree $2n - 1$, its weights are positive with sum 2, and $\lvert T_k\rvert \le 1$ on $[-1, 1]$, so the rule error on $T_k$ is at most 4 and

$$
\lvert E_n^\mathrm{GL}[g; a, b]\rvert \le \frac{b - a}{2}\cdot\frac{8M\rho^{1-2n}}{\rho - 1},\qquad M = \lambda M_0(1 + W)\max(1, e^{-\lambda s^*}). \tag{P22.97}
$$

Status: PROVED (elementary argument above; no textbook constant is used). For a history that never cools before the piece, $s^* \ge 0$ and the last factor of $M$ is 1. With $n = 8$ and every piece graded to $q \le 2$, $8\rho^{-15}/(\rho - 1) = 5.45\times10^{-12}$, so summed over $[t_0, t]$ the quadrature error is at most $2.72\times10^{-12}\lambda(t - t_0)(1 + W)M_0$. Check C5 (sphere, $\alpha = 1$, $\lambda = 1$, $t = 1$, 200 modes) finds every piece below its bound (largest ratio $2.2\times10^{-3}$) and a summed error of $3.8\times10^{-15}$ on 14 pieces; with $q = 8$ (5 pieces) the error is $2.1\times10^{-8}$. Plain composite 8-point Gauss-Legendre from $s = 0$ converges only at order 1.50 in the number of pieces ($3.2\times10^{-6}$ with 32 pieces), which is the slow start the B7 note describes. On a ramp piece the analyticity of $\tau(s)$ and $\mathrm{Re}\,\tau(s) > 0$ in the ellipse are needed; this is CLAIMED ONLY, with check C6 as the numerical evidence (route B error below double rounding).

##### Step 7. Total error and the rule for $t_0$ and $N$

Adding Steps 3 to 6:

$$
\frac{\lvert M_\mathrm{rel}^\mathrm{comp}(t) - M_\mathrm{rel}(t)\rvert}{M_0} \le \varepsilon_\mathrm{start} + e^{-\Lambda_\mathrm{cov}\tau(t_0)}\bar W_N + 2.72\times10^{-12}\lambda(t - t_0)(1 + W), \tag{P22.98}
$$

with $\varepsilon_\mathrm{start}$ equal to $(1 - e^{-\lambda t_B})\pi^{3/2}u_B^{3/2}e^{-1/u_B}$ for the image start with constant $\alpha$ (P22.86), the same plus the unbounded 16-point quadrature error for the image start on a ramp (P22.87), $\lambda\min(t_0\bar W_N, \bar V_N/\alpha_0)$ for the modal start with constant $\alpha$ (P22.93), and the right side of (P22.94) with variable $\alpha$. The last term assumes $q \le 2$, $n = 8$, constant $\alpha$ on each graded piece and $s^* \ge 0$; for a piece with $s^* < 0$ (after cooling) the factor $e^{-\lambda s^*}$ of (P22.97) multiplies that piece's share. Status of (P22.98): PROVED when the start piece and every graded piece have constant $\alpha$ and under the coverage assumption R24-A3c (late truncation (P22.95)); otherwise it holds only up to the start-piece quadrature error (DEMONSTRATED NUMERICALLY) and the ramp-piece bound (CLAIMED ONLY).

Rule for a tolerance $\varepsilon$ per $M_0$ at a reported time $t$:

1. Bare kernel with a uniform start: $t_0 = \min(t_B, t)$ with $D^*\tau(t_B)/R^2 = 1/40$; image start by (P22.86) or (P22.87); $N = 12$ modes suffice for $\Lambda_\mathrm{cov}\tau(t_B) \ge 40$. For $t \le t_B$ the boundary term $e^{-\lambda t}\hat M_\mathrm{rel}(\tau(t))$ is also taken from the image form, and the total error is at most $\pi^{3/2}u(t)^{3/2}e^{-1/u(t)}$ (check V13). Start error below $9.4\times10^{-20}$ for constant $\alpha$, late truncation below $e^{-40}W_{12} = 2.1\times10^{-19}$.
2. Layered particle, $\alpha$ constant on the first piece $[0, t_1]$: $t_0 = \min(t_1, t)$ and modal start (P22.83). Choose the smallest $N$ with $\lambda\min(t_0\bar W_N, \bar V_N/\alpha_0) \le \varepsilon/2$ and $\Lambda_\mathrm{cov}\tau(t_0) \ge 40$; when $t_0 = t$ the second condition is the v1.0 validity condition at the reported time.
3. Layered particle with a ramp from $t = 0$: for trial $N$, put $t_0 = \tau^{-1}(40/\Lambda_\mathrm{cov})$ (the v1.0 validity time), evaluate (P22.94), and raise $N$ until it is at most $\varepsilon/2$.
4. Pieces on $[t_0, t]$: the temperature breakpoints, refined so that $\tau(b)/\tau(a) \le 2$ on every piece, 8 points each. The number of extra pieces is about $\log_2(\tau(t)/\tau(t_0))$.

For a homogeneous sphere with the modal route, (P22.92) turns step 2 into the explicit count

$$
N \ge \Bigl(\frac{4\lambda R^2}{\pi^4D^*\alpha_-\varepsilon}\Bigr)^{1/3}, \tag{P22.99}
$$

for example 161, 744 and 3451 modes for $\lambda R^2/(D^*\alpha_-) = 1$ and $\varepsilon = 10^{-8}$, $10^{-10}$, $10^{-12}$; the image start of rule 1 needs 12. For the layered particle check C7 (two layers, $R_1 = 1/2$, $D^* = (1, 1/4)$, kernel-only start, $\lambda = 1$, $\alpha = 1$) gives $\bar V_N = 1.36\times10^{-5}$, $8.91\times10^{-7}$, $1.11\times10^{-7}$, $1.41\times10^{-8}$ at $N = 20$, 50, 100, 200, an observed order of 3.0 (DEMONSTRATED NUMERICALLY for this case only). There $\bar V_N$ exceeds $\sum_{\mathcal N^c}\lvert w_n\rvert/\Lambda_n^*$ by less than 3 %, while the actual error is 9 to 1600 times smaller still because the 1333 sign changes of $w_n$ in 2000 modes make the tail cancel; the bound cannot use that cancellation.

##### Step 8. Cold start

If $\alpha$ is tiny on $[0, t_c]$ (a heat-up from near room temperature), $E_n \approx (1 - e^{-\lambda t_c})/\lambda$ for every omitted mode and the standard series has the error $\lambda\sum_{\mathcal N^c}w_nE_n \to (1 - e^{-\lambda t_c})W_N$. This is a genuine error of the truncated series ($\hat M_\mathrm{rel}^{(N)}(0) = M_0\sum_{\mathcal N^c}w_n \ne 0$), and (P22.93) is sharp there through its first entry. Check C8 ($\alpha = 10^{-8}$ on $[0, 1)$, $\lambda = 1$, $N = 100$) finds $3.697\times10^{-3}$ against $(1 - e^{-1})W_{100} = 3.824\times10^{-3}$. For a bare kernel the image start removes the problem (error at rounding level in C8); the direct form $\sum_{\mathcal N}w_n(1 - e^{-\Lambda_n^*\tau})$ has an error no larger than the true release, which is at most $6\sqrt{u/\pi}$ by Step 4 (PROVED for the bare kernel; C8: $1.27\times10^{-4}$). For a layered particle with sign-changing weights no bound on the direct form is proved, so a long cold start with a large $\lambda t_c$ is OPEN in the layered case.

##### Status of the results

| Result | Equation | Label | Evidence |
|---|---|---|---|
| Quadrature and balance routes coincide; same $\sqrt s$ start | (P22.81) | exact, PROVED | C1 (SymPy) |
| Modal start piece per mode | (P22.83) | exact per mode; series in $n$ | C4, C5 |
| Image short-time form of the sphere release | (P22.84) | exact, PROVED (verification record in Appendix B.5.5, F3) | C2, V2 |
| Image remainder bound; $F \le 6\sqrt{u/\pi}$ | (P22.85) | PROVED | C2 (ratio $\le 0.603$), V3 |
| Image start piece, constant $\alpha$ / ramp | (P22.86), (P22.87) | exact plus PROVED bound / DEMONSTRATED NUMERICALLY (quadrature error unbounded) | C6, C8, V4, V13 |
| Bessel and energy tails, $\bar W_N$, $\bar V_N$ | (P22.88) to (P22.91) | PROVED; closed-form totals exact | C3, C7 |
| Sphere tails | (P22.92) | PROVED | C3 |
| Start-piece bound, constant $\alpha$ | (P22.93) | PROVED | C4 (sharp for the sphere), C7 |
| Start-piece bracket, variable $\alpha$ | (P22.94) | PROVED for $c_0 \ge 0$ | C6 |
| Late truncation | (P22.95) | PROVED under R24-A3c | C5, C7 |
| Graded Gauss-Legendre bound | (P22.96), (P22.97) | PROVED for constant $\alpha$ per piece; CLAIMED ONLY on ramps | C5, C6 |
| Total bound and mode-count rule | (P22.98), (P22.99) | PROVED for constant $\alpha$ on the start piece and on every graded piece and under the coverage assumption R24-A3c (late truncation (P22.95)) (sum of the above); with ramps only up to the DEMONSTRATED NUMERICALLY and CLAIMED ONLY quadrature parts; explicit $N$ for the sphere | C5, C6, C7, V11 |
| $\bar V_N = O(N^{-3})$ in a layered particle | | DEMONSTRATED NUMERICALLY (two layers) | C7 |
| Cold start in a layered particle | | OPEN | C8 shows the size of the effect |

Table: 5.13 Status of the results of Section 5.3.7.

##### Limitations

- All bounds assume $S = 0$. With a source the start piece of (P22.36) needs the projections $s_n$ and a separate tail bound, which is not derived here.
- The modal bounds hold in exact arithmetic. With computed eigenpairs they hold to the accuracy of the eigenpairs (the v1.0 `accuracy_floor`), and $H_N$ cannot resolve tails below about $10^{-14}$ of its total.
- (P22.95) needs the coverage assumption R24-A3c, inherited from v1.0; the tail bounds of Step 5 detect a violation but do not repair it.
- The graded quadrature bound (P22.97) is proved for constant $\alpha$ on each piece and is pessimistic by about three orders of magnitude in the tests. On ramp pieces it is CLAIMED ONLY.
- The image form (P22.84) covers one homogeneous region with a uniform start and an absorbing surface only. This bare-kernel case is the one that Section 5.3.6 identifies as exact with real data.
- All results assume the absorbing surface $c(R) = 0$. Under the Robin branch of E4 the energy identity (P22.89) gains a surface term, so the Dirichlet totals (P22.91) understate $\langle f, (-L^*)^{-1}f\rangle$ and $H_N$ turns negative (check V14); the Robin case is not treated here.
- The multilayer numbers come from one synthetic two-layer case with a kernel-only start; the rate $O(N^{-3})$ of $\bar V_N$ is not proved for layered particles.
- A layered particle with a long cold phase and large $\lambda t_c$ remains OPEN (Step 8).

#### 5.3.8 Parent and daughter with different diffusivities (open item O11)

**Result in one paragraph.** A single effective time maps the coupled parent-daughter system to a constant-coefficient problem if and only if two conditions hold: every layer of both species shares one temperature factor (condition N1, which for one-branch Arrhenius laws and a non-constant temperature means one activation energy for all layers of both species), and the decay coupling becomes constant in that clock (condition N2). N2 holds automatically when the two species have identical diffusivities in every layer (an isomeric transition) or when there is no coupling ($b\lambda_P = 0$). Otherwise it is a strong restriction on the history itself, and no linear ramp with a non-zero activation energy satisfies it, so equal activation energies are necessary but not sufficient. The statement is PROVED within the class of transformations defined in (P22.101). Independently of that class, the daughter has an exact semi-analytical solution with two clocks whenever each species separately satisfies the existing condition E1: the parent field enters the daughter as a known source in (P22.36). When E1 fails for a species, a surrogate with shared scaling gives an approximate form with a computable a priori $L^2$ error bound. That bound is PROVED, and it is weak for the TECDOC-978 Cs layer ratios.

Its equations are numbered (P22.100) to (P22.120).

##### 5.3.8.1 Symbols

| Symbol | Meaning | SI unit |
|---|---|---|
| $k \in \{P, D\}$ | species index, parent or daughter | - |
| $c = (c_P, c_D)^{\mathsf T}$ | concentration pair | $\mathrm{mol\ m^{-3}}$ |
| $D_{i,k}(T)$ | diffusivity of species $k$ in layer $i$ | $\mathrm{m^2\ s^{-1}}$ |
| $\mathcal L_k(t)$ | $r^{-2}\partial_r(r^2D_{i,k}(T(t))\partial_r\,\cdot)$ with the interface and surface conditions of (P22.31) | $\mathrm{s^{-1}}$ |
| $b$, $\lambda_P$, $\lambda_D$ | branching ratio, decay constants | -, $\mathrm{s^{-1}}$ |
| $M$ | decay matrix $\begin{pmatrix}-\lambda_P & 0\\ b\lambda_P & -\lambda_D\end{pmatrix}$ | $\mathrm{s^{-1}}$ |
| $\Delta\lambda = \lambda_P - \lambda_D$ | decay-constant difference | $\mathrm{s^{-1}}$ |
| $\tau(t)$, $\hat c(r,\tau)$ | clock and transformed pair | $\mathrm{s}$, $\mathrm{mol\ m^{-3}}$ |
| $G(t)$ | $2\times2$ amplitude matrix of the transformation | - |
| $g_k(t)$ | diagonal entries of $G$ when $G$ is diagonal | - |
| $\alpha(t)$, $\alpha_k(t)$ | shared scaling factor, species scaling factor | - |
| $D^*_{i,k}$ | $D_{i,k}(T_{\mathrm{ref}})$ | $\mathrm{m^2\ s^{-1}}$ |
| $\mu_k$, $\delta = \mu_P - \mu_D$, $C$ | constants of the admissible amplitudes and of condition N2 | $\mathrm{s^{-1}}$ (rate in $\tau$), $\mathrm{s^{-1}}$, - |
| $\beta$ | coupling coefficient in the clock $\tau$ | $\mathrm{s^{-1}}$ |
| $\Lambda^P_m$, $\Lambda^D_n$, $\phi^P_m$, $\phi^D_n$ | eigenpairs of $\mathcal L^*_P$, $\mathcal L^*_D$ (built from $D^*_{i,k}$) | $\mathrm{s^{-1}}$ |
| $O_{nm}$ | overlap matrix (P21.19) between the two eigenbases | - |
| $\tilde D_{i,k}$, $\rho_{i,k} = \tilde D_{i,k}/D_{i,k}$ | surrogate diffusivity, ratio surrogate/true | $\mathrm{m^2\ s^{-1}}$, - |
| $h(\rho) = (1-\rho)^2/\rho$, $m_k$ | mismatch function and its maximum | - |
| $\lVert f\rVert$ | $\left(\int_0^R f^2r^2\,dr\right)^{1/2}$ | $\mathrm{mol\ m^{-3}}\ \mathrm{m^{3/2}}$ |

Table: 5.14 Symbols of Section 5.3.8.

##### 5.3.8.2 Problem and assumptions

The pair solves (P21.17) and (P21.18) with temperature-dependent diffusivities. In vector form,

$$
\partial_t c = \mathcal L(t)\,c + M c + S,\qquad \mathcal L(t) = \mathrm{diag}\big(\mathcal L_P(t), \mathcal L_D(t)\big), \tag{P22.100}
$$

with $S = (S_P, S_D)^{\mathsf T}$ (the daughter term uses $y^I_D$, Section 3.3.4). Assumptions A1 to A5 of Section 5.0 hold for both species. In addition:

- **B1.** $T(t)$ is continuous on the interval considered and each $D_{i,k}(T)$ is positive and continuous. A history with jumps (a staircase) is treated interval by interval.
- **B2.** For initial data that are smooth and supported inside one layer, the solution is differentiable in $t$ at the initial time and satisfies (P22.100) there. This is used only in step 1 of the proof of Theorem 1.
- **B3.** For Theorem 3, $c$ and the surrogate $\tilde c$ are weak solutions of energy class ($c \in L^2(0,t;H^1)$ with $\partial_tc \in L^2(0,t;H^{-1})$), so that the energy equality holds. This is the standard parabolic setting; no textbook was opened for it here. The semi-discrete finite-volume version of Theorem 3 needs no such assumption (Section 5.3.8.7).

**Class of transformations.** An effective-time transformation is a pair $(\tau, G)$ with $\tau \in C^1$, $\tau' > 0$, and $G(t)$ a real invertible $2\times2$ matrix of class $C^1$, both independent of $r$ and of the data:

$$
c(r,t) = G(t)\,\hat c\big(r,\tau(t)\big). \tag{P22.101}
$$

The single-species transform (P22.27) is the member $G = e^{-\lambda t}I$, $\tau = \int_0^t\alpha\,ds$. The transformation is called admissible when, for every restart time $t_0$ and every initial pair, $\hat c$ satisfies

$$
\partial_\tau\hat c = \mathcal B\,\hat c + \hat S \tag{P22.102}
$$

with a linear operator $\mathcal B$ that does not depend on $\tau$. Sources do not affect admissibility; they map to $\hat S = G^{-1}S/\tau'$, as in (P22.30).

**Step 0, transformed generator (exact).** Differentiating (P22.101), $\partial_tc = G'\hat c + \tau'G\,\partial_\tau\hat c$. Inserting into (P22.100) and multiplying by $G^{-1}/\tau'$:

$$
\partial_\tau\hat c = \frac{1}{\tau'(t)}\,G^{-1}\Big[\big(\mathcal L(t) + M\big)G - G'\Big]\hat c + \frac{G^{-1}S}{\tau'}. \tag{P22.103}
$$

For diagonal $G = \mathrm{diag}(g_P, g_D)$, and using that $\mathcal L_k$ commutes with the scalar $g_k(t)$, the two components are

$$
\begin{aligned}
\partial_\tau\hat c_P &= \frac{\mathcal L_P}{\tau'}\hat c_P - \frac{\lambda_P + g_P'/g_P}{\tau'}\hat c_P + \frac{S_P}{g_P\tau'},\\
\partial_\tau\hat c_D &= \frac{\mathcal L_D}{\tau'}\hat c_D - \frac{\lambda_D + g_D'/g_D}{\tau'}\hat c_D + \frac{b\lambda_P\,g_P}{g_D\,\tau'}\hat c_P + \frac{S_D}{g_D\tau'}.
\end{aligned} \tag{P22.104}
$$

Check S1 of the script confirms (P22.104) by substitution.

##### 5.3.8.3 Theorem 1: necessary and sufficient conditions

**Theorem 1.** Under A1 to A5, B1 and B2, an admissible transformation (P22.101) exists on an interval if and only if

- **N1** (one clock for every layer of both species): there are a function $\alpha(t) > 0$ and constants $D^*_{i,k}$ with
$$
D_{i,k}\big(T(t)\big) = \alpha(t)\,D^*_{i,k}\qquad\text{for all layers } i \text{ and } k \in \{P, D\}, \tag{P22.105}
$$
- **N2** (constant coupling in that clock): at least one of the following holds:
  - \(i) $b\lambda_P = 0$;
  - \(ii) the two species have identical diffusivities, $D^*_{i,P} = D^*_{i,D}$ in every layer;
  - \(iii) with $\tau = \int_0^t\alpha\,ds$ there are constants $C > 0$ and $\delta$ such that
$$
\alpha(t) = C\exp\big(\delta\,\tau(t) - \Delta\lambda\,t\big),\qquad\text{equivalently}\qquad \alpha' = \delta\alpha^2 - \Delta\lambda\,\alpha. \tag{P22.106}
$$

Status: PROVED (within the class (P22.101)).

**Proof, step 1 (N1 is necessary).** Fix $t_0$ and a layer $i$. Take initial pairs $\hat c_0$ that are smooth and supported inside layer $i$; they satisfy every interface and surface condition. By B2, (P22.102) and (P22.103) give, at $t_0$, the operator identity

$$
\mathcal B\,\hat c_0 = \frac{1}{\tau'}G^{-1}\big[(\mathcal L + M)G - G'\big]\hat c_0 .
$$

The right side is a differential operator. Inside layer $i$, $\mathcal L(t) = \mathrm{diag}\big(D_{i,P}(t), D_{i,D}(t)\big)(\partial_r^2 + 2r^{-1}\partial_r)$, so its $\partial_r^2$ coefficient is $P_i(t) = G^{-1}\mathrm{diag}(D_{i,P}, D_{i,D})G/\tau'$. Since $\mathcal B$ does not depend on $\tau$, the right side is the same operator at any two times $t_1$ and $t_2$. Two differential operators that agree on all smooth functions supported in an open interval have equal coefficients (apply them to functions equal to $1$, $r - r_0$ and $(r - r_0)^2/2$ near $r_0$). Hence $P_i(t)$ is a constant matrix. Its eigenvalues are $D_{i,P}(t)/\tau'(t)$ and $D_{i,D}(t)/\tau'(t)$, so this pair of continuous functions takes values in one fixed set of at most two numbers. A continuous function on an interval with values in a finite set is constant, so $D_{i,k}(T(t)) = d_{i,k}\,\tau'(t)$ with constants $d_{i,k}$ for every $i$ and $k$. This is (P22.105) with $\alpha = \tau'/\kappa$ for any constant $\kappa > 0$. Every admissible clock is therefore $\tau = \kappa\int_0^t\alpha\,ds + \text{const}$; we take $\kappa = 1$ and $\tau(0) = 0$, which only rescales $\mathcal B$.

*Remark.* Step 1 uses only the $\partial_r^2$ coefficient. It therefore also holds for amplitude matrices $G(r,t)$ that depend on $r$ (of class $C^2$ inside each layer), because the extra terms $\partial_rG$ and $\partial_r^2G$ are of lower order.

**Step 2 (reduction to diagonal $G$).** With N1, $\mathcal L(t) = \alpha(t)\mathcal L^*$ with $\mathcal L^* = \mathrm{diag}(\mathcal L^*_P, \mathcal L^*_D)$ built from $D^*_{i,k}$. The operator part of (P22.103) is $G^{-1}\mathcal L^*G$, which must be constant, so $W(t) = G(t)G(0)^{-1}$ commutes with $\mathcal L^*$. On functions supported in layer $i$ the $\partial_r^2$ coefficients give

$$
W\,\mathrm{diag}(D^*_{i,P}, D^*_{i,D}) - \mathrm{diag}(D^*_{i,P}, D^*_{i,D})\,W = (D^*_{i,D} - D^*_{i,P})\begin{pmatrix}0 & W_{12}\\ -W_{21} & 0\end{pmatrix} = 0
$$

(check S2). Case (a): if $D^*_{i,P} \ne D^*_{i,D}$ in some layer, then $W$ is diagonal and $G(t) = \mathrm{diag}\big(w_P(t), w_D(t)\big)\,E$ with the constant matrix $E = G(0)$. A constant invertible $E$ maps an autonomous system to an autonomous one ($\check c = E\hat c$ has generator $E\mathcal BE^{-1}$), so without loss of generality $G$ is diagonal. Case (b): if $D^*_{i,P} = D^*_{i,D}$ in every layer, the two species have the same operator and $W$ is unrestricted.

**Step 3 (case (a): the scalar parts give N2).** With diagonal $G$ and $\tau' = \alpha$, (P22.104) has the operator part $\mathcal L^*_k$, which is constant. The zero-order coefficient of $\hat c_k$ in its own equation, $-(\lambda_k + g_k'/g_k)/\alpha$, must be a constant, written $-\mu_k$. Then $g_k'/g_k = \mu_k\alpha - \lambda_k$, so

$$
\begin{aligned}
g_k(t) &= g_k(0)\exp\big(\mu_k\tau(t) - \lambda_kt\big),\\
\beta(t) &= \frac{b\lambda_P\,g_P}{g_D\,\alpha} = b\lambda_P\frac{g_P(0)}{g_D(0)}\,\frac{\exp\big((\mu_P - \mu_D)\tau - \Delta\lambda\,t\big)}{\alpha(t)}.
\end{aligned} \tag{P22.107}
$$

If $b\lambda_P = 0$ then $\beta = 0$ and nothing more is required (N2 (i)). If $b\lambda_P \ne 0$, the coupling $\beta$ must be a constant $\beta_0 \ne 0$; since $g_P$ and $g_D$ are continuous and never zero, $g_P/g_D$ keeps the sign of $g_P(0)/g_D(0)$, $\beta_0$ has that sign too and $\alpha > 0$, which is (P22.106) with $\delta = \mu_P - \mu_D$ and $C = b\lambda_P g_P(0)/(g_D(0)\beta_0) > 0$ (N2 (iii)). Differentiating $\ln\alpha = \ln C + \delta\tau - \Delta\lambda t$ with $\tau' = \alpha$ gives the Bernoulli form; its solution with $\alpha(0) = \alpha_0$ is

$$
\alpha(t) = \frac{\Delta\lambda\,\alpha_0}{\delta\alpha_0 + (\Delta\lambda - \delta\alpha_0)e^{\Delta\lambda t}},
$$

with the limits $\alpha = \alpha_0e^{-\Delta\lambda t}$ for $\delta = 0$ and $\alpha = \alpha_0/(1 - \delta\alpha_0t)$ for $\Delta\lambda = 0$ (check S3). A constant temperature always satisfies N2 (iii), with $\delta = \Delta\lambda/\alpha_0$.

**Step 4 (sufficiency).** Case (a) under N2 (iii): take $g_P = e^{\delta\tau - \lambda_Pt}$ and $g_D = e^{-\lambda_Dt}$. Then (P22.104) reads $\partial_\tau\hat c_P = (\mathcal L^*_P - \delta)\hat c_P$ and $\partial_\tau\hat c_D = \mathcal L^*_D\hat c_D + (b\lambda_P/C)\hat c_P$, which is autonomous. In the eigenbases of $\mathcal L^*_P$ and $\mathcal L^*_D$ (time independent under N1), with the overlap matrix $O$ of (P21.19), the modal solution for a source-free interval is

$$
\begin{aligned}
a^P_m(t) &= a^P_m(0)\,e^{-\lambda_Pt - \Lambda^P_m\tau},\\
a^D_n(t) &= e^{-\lambda_Dt}\Big[a^D_n(0)e^{-\Lambda^D_n\tau}\\
&\qquad + \frac{b\lambda_P}{C}\sum_m O_{nm}\,a^P_m(0)\,\frac{e^{-(\Lambda^P_m + \delta)\tau} - e^{-\Lambda^D_n\tau}}{\Lambda^D_n - \Lambda^P_m - \delta}\Big],
\end{aligned} \tag{P22.108}
$$

with $\tau = \tau(t)$, and the fraction replaced by $\tau e^{-\Lambda^D_n\tau}$ when its denominator vanishes. For constant $\alpha = \alpha_0$ (so $\tau = \alpha_0t$, $\delta = \Delta\lambda/\alpha_0$, $C = \alpha_0$), multiplying numerator and denominator by $\alpha_0$ turns (P22.108) into (P21.21), as it must. Case (b): with $\mathcal L_P = \mathcal L_D = \alpha(t)\mathcal L^*$, take $G = e^{Mt}$. Because $M$ is a constant matrix that acts on the species index only, and $\mathcal L^*$ acts identically on both species, the two commute, so (P22.103) gives $\partial_\tau\hat c = \mathcal L^*\hat c$ for any $\alpha$:

$$
c_P = e^{-\lambda_Pt}\,\hat c_P(r,\tau),\qquad c_D = e^{-\lambda_Dt}\,\hat c_D(r,\tau) + b\lambda_P\,\frac{e^{-\lambda_Pt} - e^{-\lambda_Dt}}{\lambda_D - \lambda_P}\,\hat c_P(r,\tau), \tag{P22.109}
$$

where $\hat c_P$ and $\hat c_D$ both solve the v1.0 problem in $\tau$ with the common $D^*_i$ (and $\lambda_D = \lambda_P$ gives the factor $b\lambda_Pte^{-\lambda_Pt}$). Check S5 confirms (P22.109) mode by mode. Case (i) is two copies of (P22.27) with a common clock. This completes the proof. $\square$

##### 5.3.8.4 Consequences for Arrhenius laws and two counterexamples

**Corollary 1 (N1 for Arrhenius laws).** For one-branch laws $D_{i,k} = D_{0,i,k}e^{-Q_{i,k}/(R_gT)}$,

$$
\frac{d}{dt}\ln\frac{D_{i,k}}{D_{j,l}} = \frac{(Q_{i,k} - Q_{j,l})\,\dot T}{R_g\,T^2} \tag{P22.110}
$$

(check S7), so on any interval where $T$ is not constant N1 holds if and only if all activation energies of all layers of both species are equal. For multi-branch laws, N1 on a history whose temperature range is a non-degenerate interval holds if and only if every layer of both species has the same set of $Q_m$ with pre-exponential ratios independent of $m$. Proof: with $x = 1/T$, a relation $\sum_m A_me^{-\theta_mx} = \text{const}\cdot\sum_m B_me^{-\theta'_mx}$ on an $x$-interval extends to all real $x$ (both sides are entire), and exponentials with distinct rates are linearly independent (multiply by $e^{\theta_{\min}x}$ and let $x \to \infty$, then induct). This is (H22.2) applied across both species, and under B1 it turns (H22.2) from a sufficient test into a necessary and sufficient one for histories that are not isothermal. Status: PROVED.

**Corollary 2 (a linear ramp never satisfies N2 (iii)).** Under N1 with one shared activation temperature $\theta = Q/R_g \ne 0$, $\alpha = \exp[-\theta(1/T - 1/T_{\mathrm{ref}})]$ and $\alpha'/\alpha = \theta\dot T/T^2$. On a ramp $T = T_a + rt$, the Bernoulli form (P22.106) becomes, with $x = 1/T$ and $\delta' = \delta e^{\theta/T_{\mathrm{ref}}}$,

$$
\theta r\,x^2 + \Delta\lambda = \delta'\,e^{-\theta x}\qquad\text{on an } x\text{-interval}. \tag{P22.111}
$$

The third $x$-derivative of the left side is zero and that of the right side is $-\theta^3\delta'e^{-\theta x}$, so $\delta' = 0$, and then the quadratic $\theta rx^2 + \Delta\lambda$ vanishes identically, so $r = 0$ (check S4). Hence for $r \ne 0$ no constants $C$, $\delta$ exist, whatever $\Delta\lambda$ is. For $\theta = 0$ the diffusivities do not depend on $T$, $\alpha$ is constant and N2 (iii) holds on any history, so the hypothesis $\theta \ne 0$ is needed. Status: PROVED.

**Counterexample A (different activation energies).** One homogeneous sphere, $R = 250\ \mu\mathrm{m}$, parent $Q_P = 300\ \mathrm{kJ\ mol^{-1}}$ with $D_P(T_{\mathrm{ref}}) = 10^{-13}\ \mathrm{m^2\ s^{-1}}$, daughter $Q_D = 400\ \mathrm{kJ\ mol^{-1}}$ with $D_D(T_{\mathrm{ref}}) = 3\times10^{-14}\ \mathrm{m^2\ s^{-1}}$, history: ramp from 1273 K to 1873 K at $50\ \mathrm{K\ h^{-1}}$, then hold, $T_{\mathrm{ref}} = 1873\ \mathrm{K}$ (synthetic data). By Corollary 1, N1 fails, so by Theorem 1 no admissible transformation exists. Numerically (check N5), the clock read off the ratio of modes 2 and 1 of each species, $\Theta_k(t) = \int_0^tD_k\,ds$, has $\Theta_P/\Theta_D = 16.50$, $4.35$, $3.41$ and $3.35$ at $t = 2\times10^4$, $4.32\times10^4$, $10^5$ and $3\times10^5\ \mathrm{s}$; one clock for both species would need a constant ratio. Status: PROVED (Theorem 1); the numbers are DEMONSTRATED NUMERICALLY.

**Counterexample B (equal activation energies, decay coupling).** Same sphere with $Q_D = Q_P = 300\ \mathrm{kJ\ mol^{-1}}$, $D^*_D = 0.3\,D^*_P$, $\lambda_P = 10^{-6}$, $\lambda_D = 3\times10^{-6}\ \mathrm{s^{-1}}$, $b = 1$ and the same ramp. N1 holds and the operators differ (case (a)), but by Corollary 2 N2 fails, so no admissible transformation exists. Check N4 fits $\ln\alpha + \Delta\lambda t$ against $\ln C + \delta\tau$: the maximum residual is $7.35$ in $\ln\alpha$ for the ramp and $1.8\times10^{-15}$ for a Bernoulli history built to satisfy (P22.106). Freezing the coupling $\beta$ at its value at the output time and using (P22.108) anyway gives a mode-1 daughter error of $38\ \%$ at $10^5\ \mathrm{s}$ and $6.8\ \%$ at $3\times10^5\ \mathrm{s}$ (check N7, illustration only). Status: PROVED; numbers DEMONSTRATED NUMERICALLY.

##### 5.3.8.5 Proposition 2: exact two-clock cascade

The coupling is one-way: the parent never sees the daughter. A joint constant-coefficient form is therefore not needed for an exact solution.

**Proposition 2.** Suppose each species separately satisfies E1 of Section 5.3.6:

$$
D_{i,k}\big(T(t)\big) = \alpha_k(t)\,D^*_{i,k},\qquad \tau_k(t) = \int_0^t\alpha_k(s)\,ds,\qquad k \in \{P, D\}, \tag{P22.112}
$$

with $\alpha_P$ and $\alpha_D$ allowed to differ. Then the parent modes follow (P22.36) in the clock $\tau_P$, and the daughter modes are

$$
\begin{aligned}
a^D_n(t) &= a^D_n(0)\,e^{-\lambda_Dt - \Lambda^D_n\tau_D(t)}\\
&\quad + \int_0^t e^{-\lambda_D(t-s) - \Lambda^D_n[\tau_D(t) - \tau_D(s)]}\Big[b\lambda_P\sum_mO_{nm}\,a^P_m(s) + s^D_n(s)\Big]\,ds,
\end{aligned} \tag{P22.113}
$$

where $s^D_n$ is the projection of $S_D$ on $\phi^D_n$ and $O$ is the time-independent overlap matrix (P21.19) of the eigenbases of $\mathcal L^*_D$ and $\mathcal L^*_P$. For a source-free parent and $a^D_n(0) = 0$:

$$
\begin{aligned}
a^D_n(t) &= b\lambda_P\sum_mO_{nm}\,a^P_m(0)\,J_{nm}(t),\\
J_{nm}(t) &= \int_0^t\exp\!\big(-\lambda_D(t-s) - \Lambda^D_n[\tau_D(t) - \tau_D(s)] - \lambda_Ps - \Lambda^P_m\tau_P(s)\big)\,ds,\\
M_D(t) &= 4\pi\sum_nI^D_na^D_n(t).
\end{aligned} \tag{P22.114}
$$

*Proof.* Under (P22.112) the daughter equation is the single-species problem (P22.11) with diffusivities $\alpha_D D^*_{i,D}$ and the source $S_D + b\lambda_Pc_P(r,t)$, a known function once the parent is solved. Section 5.3.4 allows a source of any shape, and the projection of $b\lambda_Pc_P = b\lambda_P\sum_ma^P_m\phi^P_m$ on $\phi^D_n$ (weight $r^2$, divided by $N^D_n$) is $b\lambda_P\sum_mO_{nm}a^P_m$. Inserting this modal source into (P22.36) with $\Lambda^*_n = \Lambda^D_n$ and $\tau = \tau_D$ gives (P22.113); check S6 verifies that it solves $da^D_n/dt = -(\lambda_D + \alpha_D\Lambda^D_n)a^D_n + b\lambda_P\sum_mO_{nm}a^P_m + s^D_n$. $\square$

The eigenfunctions depend only on the layer ratios, which E1 freezes, so neither eigenbasis nor $O$ depends on the choice of $T_{\mathrm{ref}}$ for either species. For a single layer (bare kernel) E1 holds for every law and $O = I$, so (P22.114) has one quadrature per mode. In a layered particle $O$ is full and the cost is one quadrature per pair $(n,m)$. When the conditions of Theorem 1 hold, the two clocks are proportional and the substitution $s \mapsto \tau(s)$ turns $J_{nm}$ into the elementary fraction of (P22.108), which is consistent with Section 5.3.8.3; the converse (that $J_{nm}$ has no elementary form otherwise) is not claimed. The daughter release follows from the integrated daughter balance $dM_D/dt = -J_{R,D} - \lambda_DM_D + b\lambda_PM_P + \dot M_{\mathrm{gen},D}$ (exact):

$$
M_{\mathrm{rel},D}(t) = M_D(0) + b\lambda_P\int_0^tM_P\,ds - M_D(t) - \lambda_D\int_0^tM_D\,ds + M_{\mathrm{gen},D}(t),
$$

where $M_D(0)$ vanishes for the source-free case of (P22.114) and $M_{\mathrm{gen},D}(t) = \int_0^t\int S_D\,dV\,ds$.

which uses inventories only, like the balance route of step R2.4.

Status: exact for each mode; semi-analytical (one quadrature per mode pair); series in $n$ and $m$. A truncation rule with a proved bound is OPEN (it inherits O4). Checks N6 (single sphere, $Q_P = 300$, $Q_D = 400\ \mathrm{kJ\ mol^{-1}}$, ramp and hold: cascade against a stiff real-time integration of the modal system, $1.6\times10^{-13}$) and N8 (two layers, species-wise shared $Q_P = 200$ and $Q_D = 350\ \mathrm{kJ\ mol^{-1}}$, 80 modes, full $O$ with largest off-diagonal entry 4.21: cascade against a two-species finite-volume solution, daughter inventory difference $4.7\times10^{-5}$ at 160 cells per layer, observed spatial order 2.00) are DEMONSTRATED NUMERICALLY.

##### 5.3.8.6 Theorem 3: approximate form with a computable error bound

When a species fails E1 (layer-specific activation energies, the TECDOC-978 case), replace its diffusivities by a surrogate $\tilde D_{i,k}(t) > 0$ that satisfies E1 (for example one fitted activation energy per species). The surrogate is then solved exactly by Proposition 2. Keep $\lambda_k$, $b$, $S$ and the initial data unchanged. Define

$$
\lVert f\rVert^2 = \int_0^Rf^2r^2\,dr, \tag{P22.115}
$$

$$
\begin{aligned}
\rho_{i,k}(t) &= \frac{\tilde D_{i,k}}{D_{i,k}},\qquad h(\rho) = \frac{(1-\rho)^2}{\rho},\\
m_k(t) &= \max_{i,\ s\le t}h\big(\rho_{i,k}(s)\big),\qquad \frac{(D - \tilde D)^2}{D} = \tilde D\,h(\rho),
\end{aligned} \tag{P22.116}
$$

$$
\eta_k(s) = \sum_i\int_{R_{i-1}}^{R_i}\frac{(D_{i,k} - \tilde D_{i,k})^2}{D_{i,k}}\,(\partial_r\tilde c_k)^2\,r^2\,dr,\qquad e_k = c_k - \tilde c_k. \tag{P22.117}
$$

**Theorem 3.** Under A1 to A5 and B3, with $e_k(0) = 0$:

$$
\lVert e_P(t)\rVert \le \Big(\tfrac12\int_0^t\eta_P\,ds\Big)^{1/2},\qquad
\lVert e_D(t)\rVert \le \Big(\tfrac12\int_0^t\eta_D\,ds\Big)^{1/2} + b\lambda_P\int_0^t\lVert e_P(s)\rVert\,ds, \tag{P22.118}
$$

and, for $S = 0$ and $c_D(0) = 0$,

$$
\lVert e_P(t)\rVert \le \tfrac12\sqrt{m_P}\,e^{-\lambda_Pt}\,\lVert c_P(0)\rVert,\qquad
\lVert e_D(t)\rVert \le \tfrac{b}{2}\big(1 - e^{-\lambda_Pt}\big)\big(\sqrt{m_P} + \sqrt{m_D}\big)\,\lVert c_P(0)\rVert. \tag{P22.119}
$$

The inventory error follows by the Cauchy-Schwarz inequality, $\lvert M_k - \tilde M_k\rvert \le 4\pi(R^3/3)^{1/2}\lVert e_k\rVert$. For the semi-discrete finite-volume system of Section 5.2 the same statements hold with $\lVert C\rVert_V^2 = \sum_jV_jC_j^2$, with $\eta$ replaced by $\sum_f(G_f - \tilde G_f)^2(\Delta\tilde C)_f^2/G_f$ over faces, and with $m_k$ unchanged:

$$
\lVert E_k(t)\rVert_V \le \text{(P22.119) with } \lVert C_P(0)\rVert_V,\qquad \lvert M_k - \tilde M_k\rvert \le \Big(\tfrac{4\pi R^3}{3}\Big)^{1/2}\lVert E_k\rVert_V. \tag{P22.120}
$$

Status: PROVED (continuous case under B3; finite-volume case without further assumption).

*Proof.* Both $c_k$ and $\tilde c_k$ satisfy the weak form $\int_0^R\partial_tc\,v\,r^2dr = -\int_0^RD\,\partial_rc\,\partial_rv\,r^2dr - \lambda\int_0^Rcv\,r^2dr + \int_0^RSv\,r^2dr$ for every $v \in H^1$ with $v(R) = 0$; flux continuity is natural in this form, so the different interface conditions of $c$ and $\tilde c$ need no separate treatment. Subtracting and taking $v = e_k$:

$$
\tfrac12\frac{d}{dt}\lVert e\rVert^2 + \int D(\partial_re)^2r^2 + \lambda\lVert e\rVert^2 = -\int(D - \tilde D)\,\partial_r\tilde c\,\partial_re\,r^2 + (\Delta S, e),
$$

where $\Delta S = 0$ for the parent and $\Delta S = b\lambda_Pe_P$ for the daughter. Young's inequality $\lvert(D - \tilde D)\partial_r\tilde c\,\partial_re\rvert \le D(\partial_re)^2 + \tfrac14(D - \tilde D)^2(\partial_r\tilde c)^2/D$ and $\lambda \ge 0$ give $\tfrac12\frac{d}{dt}\lVert e\rVert^2 \le \tfrac14\eta + \lVert\Delta S\rVert\lVert e\rVert$. Integrate from 0 to $t$: $\lVert e(t)\rVert^2 \le \tfrac12\int_0^t\eta + 2\int_0^t\lVert\Delta S\rVert\lVert e\rVert$. Fix the end time and set $c_*^2 = \tfrac12\int_0^t\eta$. The elementary lemma "if $u \ge 0$ is continuous and $u^2 \le c_*^2 + 2\int_0^tfu$ with $f \ge 0$, then $u \le c_* + \int_0^tf$" (put $w = c_*^2 + 2\int fu$, so $w' = 2fu \le 2f\sqrt w$ and $(\sqrt w)' \le f$; for $c_* = 0$ take the limit $c_* \downarrow 0$) gives (P22.118). For (P22.119): with uniform $\lambda$ and $S = 0$, $c_P = e^{-\lambda_Pt}c_P^0$ and $\tilde c_P = e^{-\lambda_Pt}\tilde c_P^0$, where the superscript 0 marks the solutions with $\lambda = 0$; the surrogate energy equality gives $\int_0^t\!\int\tilde D(\partial_r\tilde c^0_P)^2r^2 \le \tfrac12\lVert c_P(0)\rVert^2$, and with (P22.116), $\int_0^t\eta^0_P \le \tfrac12m_P\lVert c_P(0)\rVert^2$, hence the parent bound. For the daughter surrogate, $\tfrac12\frac{d}{dt}\lVert\tilde c_D\rVert^2 \le b\lambda_P\lVert\tilde c_P\rVert\lVert\tilde c_D\rVert$ with $\lVert\tilde c_P(s)\rVert \le e^{-\lambda_Ps}\lVert c_P(0)\rVert$, so the lemma gives $\lVert\tilde c_D(s)\rVert \le b(1 - e^{-\lambda_Ps})\lVert c_P(0)\rVert$ and

$$
\int_0^t\!\!\int\tilde D_D(\partial_r\tilde c_D)^2r^2 \le b\lambda_P\int_0^t\lVert\tilde c_P\rVert\lVert\tilde c_D\rVert\,ds \le \tfrac{b^2}{2}\big(1 - e^{-\lambda_Pt}\big)^2\lVert c_P(0)\rVert^2
$$

(check S9). Then $(\tfrac12\int_0^t\eta_D)^{1/2} \le \tfrac b2(1 - e^{-\lambda_Pt})\sqrt{m_D}\lVert c_P(0)\rVert$, and the parent term is $b\lambda_P\int_0^t\tfrac12\sqrt{m_P}e^{-\lambda_Ps}\lVert c_P(0)\rVert\,ds = \tfrac b2(1 - e^{-\lambda_Pt})\sqrt{m_P}\lVert c_P(0)\rVert$. In the finite-volume case the same steps hold with sums: $C^{\mathsf T}KC = \sum_fG_f(\Delta C)_f^2$, and since $\tilde G_f/G_f$ is a weighted harmonic mean of the two adjacent cell ratios $\rho$ and $h$ is decreasing on $(0,1]$ and increasing on $[1,\infty)$, $h(\tilde G_f/G_f) \le \max(h(\rho^-), h(\rho^+)) \le m_k$. $\square$

**How informative the bound is.** The a priori factor $\tfrac12\sqrt{h(\rho)} = \tfrac12\lvert1 - \rho\rvert/\sqrt\rho$ is below 0.1 only for $0.819 < \rho < 1.221$ and below 1 for $0.172 < \rho < 5.83$ (check N11). For a surrogate with fixed layer ratios, the ratio $\rho_1/\rho_2$ of two layers spans the same factor $q$ as the true ratio $D_1/D_2$ over the history. At every instant $\max(h(\rho_1), h(\rho_2)) \ge h(\sqrt{\rho_1/\rho_2})$, so $m \ge h(q^{1/4})$. For the TECDOC-978 Cs ratio $D_{\mathrm{SiC}}/D_{\mathrm{kernel}}$, which spans a factor 6.3 between 1250 and 1800 $^\circ$C (Section 5.2 of the pseudocode; the ratio is not monotonic, so the history must reach both its minimum near 1438 $^\circ$C and 1800 $^\circ$C), any shared-scaling surrogate therefore has an a priori factor of at least $0.23$ (check N12): a relative $L^2$ error bound of 23 % at best, before any other layer is counted. The a posteriori form (P22.118) can be much smaller because $\eta$ weights the mismatch by the surrogate gradient. Checks N9 (single sphere, true $Q_P = 300$ and $Q_D = 330$, surrogate $315\ \mathrm{kJ\ mol^{-1}}$ for both) and N10 (two layers, true $Q_P = (200, 230)$ and $Q_D = (330, 370)$, surrogates 215 and $350\ \mathrm{kJ\ mol^{-1}}$, finite volumes) confirm every inequality; the observed errors lie between about one and six orders of magnitude below the a priori bounds, and the a posteriori parent bound is tighter than the a priori one by a factor 4 to 7 (DEMONSTRATED NUMERICALLY).

##### 5.3.8.7 Summary and labels

| Case | Joint effective time (one clock, constant coefficients) | Exact solution available | Label |
|---|---|---|---|
| $b\lambda_P = 0$, N1 | yes, two copies of (P22.27) | yes | PROVED |
| identical diffusivities in every layer (isomeric pair), N1 | yes, for any $T(t)$, (P22.109) | yes, closed form in $\tau$ | PROVED |
| N1 with different operators, isothermal interval | yes, (P21.21) | yes, closed form | PROVED |
| N1 with different operators, history obeying (P22.106) | yes, (P22.108) | yes, closed form in $\tau$ | PROVED |
| N1 with different operators, $b\lambda_P \ne 0$, any linear ramp with $Q \ne 0$ | no (Corollary 2) | yes, cascade (P22.114) | PROVED |
| each species satisfies E1, $Q_P \ne Q_D$ (includes every bare-kernel pair) | no (Theorem 1, Counterexample A) | yes, cascade (P22.113), semi-analytical | PROVED; cascade DEMONSTRATED NUMERICALLY (N6, N8) |
| a species fails E1 (layer-specific $Q$) | no | no; surrogate plus bound (P22.119) or the finite-volume path | bound PROVED; weak for TECDOC-978 data |

Table: 5.11 Effective-time options for a parent-daughter pair.

**Limitations.**

- Theorem 1 is proved for the class (P22.101): one clock independent of $r$ and an amplitude matrix independent of $r$. Step 1 (necessity of N1) also covers amplitudes $G(r,t)$; the remaining steps do not, and transformations with $r$-dependent clocks or nonlocal maps are not covered (OPEN).
- B1 excludes temperature jumps inside an interval; a staircase is treated interval by interval.
- The cascade needs one quadrature per mode pair, and the truncation of the double series has no proved bound (OPEN, related to O4). The graded Gauss-Legendre rule of the script is a verification device; its accuracy was checked against stiff integrations only for the cases of N6 and N8.
- B3 rests on standard parabolic theory with no textbook opened here; the finite-volume form (P22.120) does not need it.
- The a priori bound (P22.119) assumes $S = 0$ and $c_D(0) = 0$; the a posteriori form (P22.118) holds with sources but needs the surrogate gradient, which for the daughter means the cascade at every quadrature node.
- All numbers in the checks are synthetic. Daughter diffusivities (Y, Ba) per layer are still not in the audited data (O1), so no real chain can be run yet.

### 5.4 (d) General multilayer case: layer-specific activation energies

When the ratios $D_i/D_j$ change with $T$, the eigenfunctions change with $T$: the interface condition $D_i u_i' - D_{i+1}u_{i+1}' = (D_i - D_{i+1})u/r$ (the v1.0 rule in $u = rc$) depends on the ratios. Four approaches follow.

#### 5.4.1 (i) Piecewise-constant temperature with eigenfunction projection

Let

$$
T(t) = T_k\quad\text{for}\quad t_k \le t < t_{k+1},\qquad k = 0,\dots,N_T-1. \tag{P22.39}
$$

On interval $k$ the operator $L^{(k)}$ has constant coefficients $D_i(T_k)$ and eigenpairs $(\Lambda_n^{(k)}, \phi_n^{(k)})$ found by the v1.0 shooting and Sturm-count machinery, so

$$
\begin{aligned}
c(r,t) &= \sum_n a_n^{(k)}(t)\phi_n^{(k)}(r),\\
a_n^{(k)}(t) &= e^{-(\Lambda_n^{(k)} + \lambda)(t - t_k)}a_n^{(k)}(t_k) + \int_{t_k}^t e^{-(\Lambda_n^{(k)} + \lambda)(t - s)}s_n^{(k)}(s)\,ds.
\end{aligned} \tag{P22.40}
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
\begin{aligned}
K_{mn}(t) &= \frac{1}{N_m}\int_0^R r^2D(r,T(t))\,\psi_m'\psi_n'\,dr = \sum_{i=1}^5 D_i(T(t))\,K^{(i)}_{mn},\\
K^{(i)}_{mn} &= \frac{1}{N_m}\int_{R_{i-1}}^{R_i}r^2\psi_m'\psi_n'\,dr.
\end{aligned} \tag{P22.49}
$$

The affine split in (P22.49) lets the five layer matrices $K^{(i)}$ be computed once; each step only forms a weighted sum. $N^{1/2}KN^{-1/2}$ is symmetric positive definite. If $\psi = \phi^*$ and $D = \alpha D^*$, integration by parts with the eigen-equation gives

$$
\int_0^R r^2D^*\phi_m^{*\prime}\phi_n^{*\prime}\,dr = \Lambda_n^*N_n\,\delta_{mn}\quad\Longrightarrow\quad K(t) = \alpha(t)\,\mathrm{diag}(\Lambda^*), \tag{P22.50}
$$

and (P22.48) decouples into (P22.36). In general

$$
\begin{aligned}
\mathbf a(t) &= U(t,0)\,\mathbf a(0) + \int_0^tU(t,s)\,\mathbf s(s)\,ds,\\
U(t,s) &= \mathcal T\exp\!\left(-\int_s^t[K(\sigma) + \lambda I]\,d\sigma\right),\\
{}[K(t_1),K(t_2)] &= \sum_{i,j}D_i(t_1)D_j(t_2)\,[K^{(i)},K^{(j)}],
\end{aligned} \tag{P22.51}
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
\begin{aligned}
&e_{N_T} \approx C_{\mathrm L}N_T^{-1}\ \ (\text{left}),\qquad e_{N_T} \approx C_{\mathrm M}N_T^{-2}\ \ (\text{midpoint, average}),\\
&p_{\mathrm{obs}} = \log_2(e_{N_T}/e_{2N_T}).
\end{aligned} \tag{P22.56}
$$

Caveat: the Magnus expansion assumes $h\lVert B\rVert$ small. For the stiff FV matrices of the test $h\lVert B\rVert$ is of order $10^2$ to $10^3$, where the argument is heuristic (a fourth-order Magnus step was tried as a reference and failed for this reason). The observed orders below confirm (P22.56) for $F(6\ \mathrm{h})$ in the test problem; at the end of the ramp, $F(4\ \mathrm{h})$, the observed midpoint orders scatter between 1.80 and 2.11 over the last pairs (script output). A proof for the stiff regime is not given here (update 2026-11-13: Section 5.4.6 proves order 1 uniformly in the stiffness, order 2 for shared scaling and, for layer-specific activation energies, a bound $h^2(1+\ln(T/h))$ on a fixed mesh with a constant proportional to $\Lambda$, and shows numerically that order 2 in the $V$-norm is not uniform under mesh refinement when the layers have different activation energies). With shared scaling, all three choices have $\Omega_2 = 0$ and the per-layer average is exact at the interval ends:

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

#### 5.4.6 Staircase convergence in the stiff regime (open item O10)

This section replaces the caveat after Eq. (P22.56). It proves error bounds for the staircase of Section 5.4.4 that do not assume $h\lVert B\rVert$ small, using energy and logarithmic-norm arguments in the $V$-weighted norm, and it tests the predicted orders on the stiff five-layer finite-volume (FV) matrix. Its equations are numbered (P22.121) to (P22.142).

Summary of the outcome:

| Statement | Assumption on $D_i(T(t))$ | Order, uniform in $h\lVert A\rVert$ | Label |
|---|---|---|---|
| Theorem A, all three rules | $\ln D_i$ Lipschitz in $t$ (constant $L$) | 1, explicit constant, any mesh | PROVED |
| Sharpness of order 1, left rule | same | order 1 cannot be improved | PROVED |
| Theorem B (i), jumps on the grid | piecewise constant, breakpoints at $t_k$ | exact | PROVED |
| Theorem B (ii), one jump inside an interval | piecewise constant | 1 | PROVED |
| Theorem C, shared scaling | $D_i = \alpha(t)D_i^*$, $\alpha\in C^2$ | 2 (midpoint), exact (average), 1 (left) | PROVED |
| Theorem D, layer-specific $Q_i$ | $\ln D_i \in C^2$ | $h^2(1+\ln(T/h))$ with a constant $\propto\Lambda$, uniform in $h$ on a fixed mesh (the constant grows under mesh refinement because $\Lambda\ge\Lambda_t$, next row) | PROVED |
| Growth $\Lambda \propto N^{1/2}$ | layer-specific $Q_i$ | - | DEMONSTRATED NUMERICALLY |
| $V$-norm order about 1.75 for midpoint and average when $h\lVert A\rVert \gg 1$ (test case) | layer-specific $Q_i$ | 1.76 to 1.81 in the window, 1.75 on 80 cells per layer | DEMONSTRATED NUMERICALLY |
| Limiting order $7/4$ as the mesh is refined | layer-specific $Q_i$ | $7/4$ | CLAIMED ONLY (heuristic) |
| A second synthetic case (three layers) gives mesh-converged $V$-norm orders 1.51 to 1.96 for midpoint | layer-specific $Q_i$ | case dependent | DEMONSTRATED NUMERICALLY (verification record, Appendix B.5.7) |

Table: 5.15 Summary of the results of Section 5.4.6.

The main consequence is a correction of (P22.56): order 2 of the midpoint and average rules is not uniform in the stiffness when the layers have different activation energies (DEMONSTRATED NUMERICALLY in two synthetic cases; no proof of the lower bound is given). In the $V$-norm of the concentration vector the observed order in the stiff window is between 1.76 and 1.81 (1.75 on the finest mesh); the release fraction, a smooth functional, still shows orders between 1.98 and 2.13.

##### 5.4.6.1 Setting and assumptions

Semi-discrete system (P22.13) with $S = 0$ and decay constant $\lambda\ge 0$. With $z = V^{1/2}C$ and the symmetric matrix $\hat K(D) = V^{-1/2}K(D)V^{-1/2}$,

$$
\lVert C\rVert_V = \lVert z\rVert_2,\qquad A = V^{-1}K = V^{-1/2}\hat K\,V^{1/2},\qquad \frac{dz}{dt} = -\big(\hat K(D(t)) + \lambda I\big)z. \tag{P22.121}
$$

$\hat K(D)$ is symmetric positive definite for every $D>0$ (absorbing outer face), so the logarithmic norm of $-A$ in the $V$-norm is $-\lambda_{\min}(\hat K)\le 0$ and $\lVert e^{-sA}\rVert_V\le 1$ for every $s\ge 0$, whatever the size of $s\lVert A\rVert$. The staircase uses $\tilde D_k$ (left, midpoint or per-layer average, Eq. (P22.53)) on $I_k = [t_k, t_{k+1})$, $t_k = kh$, $h = T/N_T$, and propagates exactly: $\dot{\tilde z} = -(\hat K(\tilde D_k) + \lambda I)\tilde z$ on $I_k$. The error is $e = z - \tilde z$, and $\hat\Delta(s) = \hat K(D(s)) - \hat K(\tilde D_k)$ for $s\in I_k$.

The FV matrix is a sum over faces,

$$
\begin{aligned}
K(D) &= \sum_f G_f(D)\,\mathbf a_f\mathbf a_f^{\mathsf T},\qquad \mathbf a_f = \mathbf u_j - \mathbf u_{j+1}\ \text{(face between cells } j, j+1\text{)},\\
\mathbf a_{\mathrm{out}} &= \mathbf u_N,
\end{aligned} \tag{P22.122}
$$

with $G_f$ from (P22.14): $G_f = A_f/(\delta_f^-/D_- + \delta_f^+/D_+)$ at a face between layers, $G_f = A_fD_i/(\delta_f^- + \delta_f^+)$ inside layer $i$, and $G_{\mathrm{out}} = 4\pi R^2D_5/(R-\bar r_N)$.

Assumptions, used as named below:

- (H0) $D_i(t)$ positive and bounded away from zero and infinity; $\lambda\ge0$; $S = 0$.
- (H1) $\ln D_i(T(t))$ is Lipschitz in $t$: $\lvert \tfrac{d}{dt}\ln D_i\rvert\le L$ almost everywhere. A continuous piecewise-linear history (ramps and holds with kinks) satisfies (H1). For one Arrhenius branch, $\tfrac{d}{dt}\ln D_i = Q_i\dot T/(R_gT^2)$, so

  $$
  L = \max_i\frac{Q_i\,\sup_t\lvert\dot T\rvert}{R_g\,T_{\min}^2}, \tag{P22.123}
  $$

  and for several parallel branches the logarithmic derivative is a weighted mean of the branch values, so $\max_b Q_b$ replaces $Q_i$.
- (H2) $\ln D_i\in C^2$ on $[0,T]$ (or on each $I_k$), with $\lvert\tfrac{d^2}{dt^2}\ln D_i\rvert\le L_2$.
- (H3) operator-relative bound, Eq. (P22.136) below.
- \(J) $T(t)$ piecewise constant with finitely many jumps.

In the test problem ($Q_{\max} = 514\ \mathrm{kJ\,mol^{-1}}$, $\dot T = 200\ \mathrm{K}/4\ \mathrm{h}$, $T_{\min} = 1873.15\ \mathrm{K}$), $L = 2.45\times10^{-4}\ \mathrm{s^{-1}}$ and $LT = 3.52$ over the 4 h ramp (script output, Appendix A.2).

##### 5.4.6.2 Three lemmas

**Lemma 1 (face lemma).** Let $r_i = D_i/D_i'$ and $\underline r\le r_i\le\bar r$ for all layers. Then for every face

$$
\underline r\,G_f(D')\le G_f(D)\le\bar r\,G_f(D'),\qquad \underline r\,\hat K(D')\preceq\hat K(D)\preceq\bar r\,\hat K(D'). \tag{P22.124}
$$

Proof. Each $G_f$ is non-decreasing in each of its (one or two) diffusivities and positively homogeneous of degree one, so $G_f(D)\le G_f(\bar rD') = \bar rG_f(D')$, and likewise from below. The matrix statement follows from (P22.122), because each $\mathbf a_f\mathbf a_f^{\mathsf T}\succeq 0$. $\square$

Write $\lVert\mathbf x\rVert_M^2 = \mathbf x^{\mathsf T}M\mathbf x$, $\epsilon = \max_i\lvert r_i - 1\rvert$ and $\epsilon' = \max_i\lvert r_i^{-1} - 1\rvert$. By (P22.124), $g_f = G_f(D) - G_f(D')$ obeys $\lvert g_f\rvert\le\epsilon G_f(D')$ and $\lvert g_f\rvert\le\epsilon' G_f(D)$, hence $\lvert g_f\rvert\le\sqrt{\epsilon G_f(D')}\sqrt{\epsilon' G_f(D)}$. The Cauchy-Schwarz inequality over the faces then gives the mixed bound

$$
\big\lvert\mathbf x^{\mathsf T}\big(\hat K(D) - \hat K(D')\big)\mathbf y\big\rvert\le\sqrt{\epsilon\,\epsilon'}\ \lVert\mathbf x\rVert_{\hat K(D')}\,\lVert\mathbf y\rVert_{\hat K(D)}. \tag{P22.125}
$$

Time derivatives. With $x = \ln D_-$, $y = \ln D_+$ and $\theta_f = (\delta_f^-/D_-)/(\delta_f^-/D_- + \delta_f^+/D_+)\in(0,1)$ (exact, SymPy check S0.2 and S0.3),

$$
\frac{d}{dt}\ln G_f = \theta_f\dot x + (1-\theta_f)\dot y,\qquad \frac{d^2}{dt^2}\ln G_f = \theta_f\ddot x + (1-\theta_f)\ddot y - \theta_f(1-\theta_f)(\dot x - \dot y)^2. \tag{P22.126}
$$

So under (H1) $\lvert\dot G_f\rvert\le LG_f$ and $\pm\dot{\hat K}\preceq L\hat K$; under (H2) $\lvert\tfrac{d^2}{dt^2}\ln G_f\rvert\le L_2 + L^2$ and $\lvert\ddot G_f\rvert\le(L_2 + 2L^2)G_f$, since $\ddot G/G = (\ln G)'' + ((\ln G)')^2$. Faces inside a layer and the outer face have $\theta_f = 1$.

**Lemma 2 (error representation).** Let $\boldsymbol\phi\in\mathbb R^N$ and let $\mathbf w$ solve the backward staircase problem $\dot{\mathbf w} = (\hat K(\tilde D_k) + \lambda I)\mathbf w$ on $I_k$, $\mathbf w(T) = \boldsymbol\phi$. Then

$$
\boldsymbol\phi^{\mathsf T}\mathbf e(T) = -\int_0^T\mathbf w(s)^{\mathsf T}\hat\Delta(s)\,\mathbf z(s)\,ds. \tag{P22.127}
$$

Proof. $\dot{\mathbf e} = -(\hat K(\tilde D_k) + \lambda I)\mathbf e - \hat\Delta\mathbf z$, so $\tfrac{d}{ds}(\mathbf w^{\mathsf T}\mathbf e) = -\mathbf w^{\mathsf T}\hat\Delta\mathbf z$ on each $I_k$; $\mathbf w^{\mathsf T}\mathbf e$ is continuous, $\mathbf e(0) = 0$. $\square$ (Numerical check S2: both sides agree to a relative $10^{-9}$ on a mesh of 4 cells per layer.)

**Lemma 3 (energy identities).** For the exact solution and the backward staircase solution,

$$
\int_0^T\lVert\mathbf z\rVert^2_{\hat K(D(s))}\,ds\le\tfrac12\big(\lVert\mathbf z_0\rVert^2 - \lVert\mathbf z(T)\rVert^2\big),\qquad \int_0^T\lVert\mathbf w\rVert^2_{\hat K(\tilde D(s))}\,ds\le\tfrac12\lVert\boldsymbol\phi\rVert^2. \tag{P22.128}
$$

Proof. $\tfrac{d}{dt}\lVert\mathbf z\rVert^2 = -2\lVert\mathbf z\rVert^2_{\hat K} - 2\lambda\lVert\mathbf z\rVert^2$ and $\tfrac{d}{ds}\lVert\mathbf w\rVert^2 = 2\lVert\mathbf w\rVert^2_{\hat K(\tilde D)} + 2\lambda\lVert\mathbf w\rVert^2$; integrate and drop the $\lambda$ terms, which have the favourable sign. Equality holds for $\lambda = 0$. $\square$

##### 5.4.6.3 Theorem A: order 1, uniform in the stiffness

**Theorem A.** Under (H0), let $\epsilon_k(s)$, $\epsilon_k'(s)$ be the quantities of (P22.125) for $D = D(s)$, $D' = \tilde D_k$, and $\bar\eta = \sup_k\sup_{s\in I_k}\sqrt{\epsilon_k(s)\epsilon_k'(s)}$. Then, for every mesh, every $h$ and every $\lambda\ge0$,

$$
\lVert C(T) - \tilde C(T)\rVert_V\le\tfrac12\,\bar\eta\,\sqrt{\lVert C_0\rVert_V^2 - \lVert C(T)\rVert_V^2}\le\tfrac12\,\bar\eta\,\lVert C_0\rVert_V. \tag{P22.129}
$$

Proof. In (P22.127) apply (P22.125) with $\mathbf x = \mathbf w$, $\mathbf y = \mathbf z$, then the Cauchy-Schwarz inequality in time and (P22.128):

$$
\lvert\boldsymbol\phi^{\mathsf T}\mathbf e(T)\rvert\le\bar\eta\Big(\int_0^T\lVert\mathbf w\rVert^2_{\hat K(\tilde D)}\Big)^{1/2}\Big(\int_0^T\lVert\mathbf z\rVert^2_{\hat K(D)}\Big)^{1/2}\le\tfrac12\,\bar\eta\,\lVert\boldsymbol\phi\rVert\sqrt{\lVert\mathbf z_0\rVert^2 - \lVert\mathbf z(T)\rVert^2}.
$$

Take the supremum over $\lVert\boldsymbol\phi\rVert = 1$. $\square$

No norm of $A$ appears, so the bound holds for $h\lVert A\rVert$ of any size. The same bound holds at every intermediate time.

**Corollary A1 (the three rules).** Under (H1), $r_i(s) = D_i(s)/\tilde D_{i,k}\in[e^{-L\delta},e^{L\delta}]$ with $\delta = h$ for the left rule, $\delta = h/2$ for the midpoint rule, and $\delta = h$ for the per-layer average (the average lies between the extreme values on $I_k$). Since $\lvert r - 1\rvert$ and $\lvert r^{-1} - 1\rvert$ are both at most $e^{L\delta} - 1$,

$$
\begin{aligned}
\lVert\mathbf e(T)\rVert_V&\le\tfrac12\big(e^{Lh} - 1\big)\lVert C_0\rVert_V\ \ (\text{left, average}),\\
\lVert\mathbf e(T)\rVert_V&\le\tfrac12\big(e^{Lh/2} - 1\big)\lVert C_0\rVert_V\ \ (\text{midpoint}).
\end{aligned} \tag{P22.130}
$$

Order 1 therefore needs only a Lipschitz temperature history (kinks allowed anywhere) and holds uniformly in the stiffness, the mesh, the layer data and $\lambda$. Check S3: the bound holds for all three rules on meshes of 20 and 40 cells per layer with $h\lVert A\rVert$ from 71 to $4.6\times10^3$; the largest observed ratio error/bound is 0.086, so the bound is safe but not tight for this smooth ramp (the worst case over the stiffness is (P22.131)).

**Sharpness (left rule).** Take $\hat K(t) = e^{Lt}\hat K^*$ (shared scaling, $\lvert\tfrac{d}{dt}\ln\alpha\rvert = L$) and an eigenvector of $\hat K^*$ with eigenvalue $\mu$ as initial datum. The exact and left-staircase exponents are $X = \mu(e^{LT} - 1)/L$ and $X/\beta$ with $\beta = (e^{Lh} - 1)/(Lh) > 1$, so the error is $e^{-X/\beta} - e^{-X}$. Its maximum over $X>0$ (that is, over the stiffness $\mu$) is exact:

$$
\begin{aligned}
&\sup_{\mu>0}\lVert\mathbf e(T)\rVert = \beta^{-1/(\beta-1)}\big(1 - \beta^{-1}\big) = \frac{Lh}{2e}\big(1 + O(Lh)\big),\\
&\text{midpoint: } \beta = \frac{\sinh(Lh/2)}{Lh/2},\ \ \sup_{\mu>0}\lVert\mathbf e(T)\rVert = \frac{(Lh)^2}{24e}\big(1 + O(Lh)\big).
\end{aligned} \tag{P22.131}
$$

(SymPy checks S0.4 and S0.5.) No bound of the form $o(h)$ uniform in the stiffness exists for the left rule, and the constant $\tfrac12$ of (P22.130) is within a factor $e$ of the best possible one.

##### 5.4.6.4 Smoothing lemma and Theorem B: temperature jumps

**Lemma 4 (energy smoothing).** (a) Under (H1), $\lVert\mathbf z(t)\rVert^2_{\hat K(D(t))}\le\dfrac{e^{Lt}}{2t}\lVert\mathbf z_0\rVert^2$. If $D$ is piecewise constant with jumps $D^-\to D^+$ at times before $t$, and $\rho_j = \max\big(1, \max_iD_i^+/D_i^-\big)\ge 1$ for jump $j$, then $\lVert\mathbf z(t)\rVert^2_{\hat K(D(t))}\le\dfrac{\prod_j\rho_j}{2t}\lVert\mathbf z_0\rVert^2$. The floor at 1 is needed: with a cooling jump and $\rho_j<1$ the product form is false (counterexample V4.1 of the verification record in Appendix B.5.7). (b) For the backward staircase solution, with $q$ the product, over the switch times $t_j\in(s, T)$, of $\max\big(1, \max_i\tilde D_{i,j-1}/\tilde D_{i,j}\big)$ (the value on the earlier interval in real time over the value on the later one; in reversed time this is the new value over the old one, because the energy $\lVert\mathbf w\rVert^2_{\hat K}$ is multiplied by at most this ratio at the switch, Lemma 1),

$$
\begin{aligned}
&\lVert\mathbf z(t)\rVert^2_{\hat K(D(t))}\le\frac{e^{Lt}}{2t}\lVert\mathbf z_0\rVert^2,\\
&\lVert\mathbf w(s)\rVert^2_{\hat K(\tilde D(s))}\le\frac{q}{2(T-s)}\lVert\boldsymbol\phi\rVert^2,\quad q\le e^{2L(T-s)}\ \text{under (H1)}.
\end{aligned} \tag{P22.132}
$$

Proof. Let $E = \mathbf z^{\mathsf T}\hat K\mathbf z$ and $m_0 = \lVert\mathbf z_0\rVert^2\ge\lVert\mathbf z(t)\rVert^2$. Then $\dot E = \mathbf z^{\mathsf T}\dot{\hat K}\mathbf z - 2\lVert\hat K\mathbf z\rVert^2 - 2\lambda E\le LE - 2E^2/m_0$, using $\pm\dot{\hat K}\preceq L\hat K$ and $E\le\lVert\mathbf z\rVert\,\lVert\hat K\mathbf z\rVert$. For $u = 1/E$, $\dot u\ge -Lu + 2/m_0$, so $u(t)\ge\tfrac{2}{m_0}(1 - e^{-Lt})/L\ge 2te^{-Lt}/m_0$. At a jump $E$ is multiplied by at most $\rho_j$ (Lemma 1), so $u$ is divided by at most $\rho_j$, and between jumps $u$ grows at least at the rate $2/m_0$ when $L = 0$; by induction over the jumps, using $\rho_j\ge1$, $u(t)\ge 2t/(m_0\prod_j\rho_j)$. Part (b) is the same argument in reversed time $\sigma = T - s$ with $L = 0$ inside each interval, the factor $\max(1,\max_i\tilde D_{i,j-1}/\tilde D_{i,j})$ at the switch $t_j$ (the orientation matters: with the inverse ratio the statement is false, counterexample V11.1 of the verification record in Appendix B.5.7), and, under (H1), a factor of at most $e^{2Lh}$ at each of the at most $\sigma/h$ switches (consecutive staircase values differ by at most $2Lh$ in $\ln D_i$). $\square$ (Check S6 for part (a).)

**Theorem B.** Under (J):

(i) If every jump time is a grid point $t_k$ (right-continuous history), all three rules give $\tilde D_k = D(s)$ on every $I_k$, so $\hat\Delta\equiv 0$ and $\mathbf e\equiv 0$ by (P22.127): the staircase is exact.

(ii) If one jump $D^a\to D^b$ lies inside $I_{k^*}$ with $1\le k^*\le N_T - 2$, and $\rho = \max_i\max\big(D^b_i/D^a_i,\ D^a_i/D^b_i\big)\ge1$ is its two-sided ratio (heating and cooling layers both allowed), then

$$
\lVert\mathbf e(T)\rVert_V\le\frac{\sqrt{\epsilon_{k^*}\epsilon'_{k^*}}\ \rho\ h}{2\sqrt{t_{k^*}\,(T - t_{k^*+1})}}\ \lVert C_0\rVert_V,\qquad \sqrt{\epsilon_{k^*}\epsilon'_{k^*}}\le\rho - 1. \tag{P22.133}
$$

Proof. Only $I_{k^*}$ contributes to (P22.127). Apply (P22.125), bound $\int_{I_{k^*}}\lVert\mathbf w\rVert\,\lVert\mathbf z\rVert\,ds$ by $h$ times the product of the suprema of the two energies from (P22.132), with one jump for $\mathbf z$ (inside $I_{k^*}$, factor at most $\rho$) and one switch for $\mathbf w$ (at $t_{k^*+1}$, from $D^b$ to $\tilde D_{k^*}$, which lies between $D^a$ and $D^b$ for every rule, so again a factor at most $\rho$). The two-sided $\rho$ also gives $r_i = D_i(s)/\tilde D_{i,k^*}\in[\rho^{-1},\rho]$, hence $\sqrt{\epsilon\epsilon'}\le\rho-1$. $\square$

So a misaligned jump costs order 1 for every rule, including the midpoint and average rules, with a constant that does not depend on the stiffness; several jumps add. For a history that is Lipschitz between jumps, the same steps give the sum of (P22.129) over the smooth intervals and (P22.133) over the jump intervals, with the smoothing factors multiplied by $e^{LT}\prod_j\rho_j$. Checks S5.1 (aligned jump, error below $10^{-13}$) and S5.2 (misaligned jump at $0.3T$, bound holds for every rule and $N_T$ from 16 to 4096).

##### 5.4.6.5 Theorem C: the commuting case

With shared scaling $D_i = \alpha(t)D_i^*$, homogeneity of $G_f$ gives $\hat K(D(t)) = \alpha(t)\hat K^*$ exactly, so all matrices commute. With $\tau = \int_0^T\alpha\,dt$ and $\tilde\tau = h\sum_k\tilde\alpha_k$, $\mathbf z(T) = e^{-\lambda T}e^{-\tau\hat K^*}\mathbf z_0$ and $\tilde{\mathbf z}(T) = e^{-\lambda T}e^{-\tilde\tau\hat K^*}\mathbf z_0$. For every eigenvalue $\mu>0$ of $\hat K^*$, $\lvert e^{-\mu\tau} - e^{-\mu\tilde\tau}\rvert\le\mu\lvert\tau - \tilde\tau\rvert e^{-\mu\min(\tau,\tilde\tau)}$ and $\sup_{\mu>0}\mu e^{-\mu m} = 1/(em)$, hence

$$
\lVert\mathbf e(T)\rVert_V\le\frac{\lvert\tau - \tilde\tau\rvert}{e\,\min(\tau,\tilde\tau)}\,\lVert C_0\rVert_V. \tag{P22.134}
$$

With the quadrature errors of the three rules and $\min(\tau,\tilde\tau)\ge T\inf\alpha$:

$$
\begin{aligned}
\lVert\mathbf e(T)\rVert_V&\le\frac{h^2}{24e}\,\frac{\sup\lvert\ddot\alpha\rvert}{\inf\alpha}\,\lVert C_0\rVert_V\ \ (\text{midpoint}),\\
&\le\frac{h}{2e}\,\frac{\sup\lvert\dot\alpha\rvert}{\inf\alpha}\,\lVert C_0\rVert_V\ \ (\text{left}),\qquad \mathbf e(T) = 0\ \ (\text{average}).
\end{aligned} \tag{P22.135}
$$

(The last statement is (P22.57).) Order 2 of the midpoint rule is therefore uniform in the stiffness whenever the matrices commute, which for the FV matrix means shared scaling (or a single layer). Check S4: bound holds, observed order 2.00 at $h\lVert A\rVert$ up to $4.5\times10^3$, average rule exact up to round-off ($1.3\times10^{-12}$).

##### 5.4.6.6 Theorem D: order 2 on a fixed mesh, layer-specific activation energies

Let $\mathcal B$ be the box of diffusivity vectors with $\ln D_i$ between the minimum and maximum of $\ln D_i(T(t))$ on $[0,T]$; all staircase values lie in $\mathcal B$. Define $\Lambda$ as the smallest constant with

$$
\big\lVert\big(\hat K(D) - \hat K(D')\big)\,\hat K(D'')^{-1}\big\rVert_2\le\Lambda\,\lVert\ln D - \ln D'\rVert_\infty\qquad\text{for all } D, D', D''\in\mathcal B. \tag{P22.136}
$$

(H3) is the statement $\Lambda<\infty$. For a fixed mesh it always holds: $\hat K$ is a smooth function of $\ln D$ on the compact box and is uniformly invertible there. It implies $\lVert\dot{\hat K}\hat K(D'')^{-1}\rVert\le\Lambda L$ and $\lVert\hat K(\tilde D_j)\hat K(\tilde D_{j+1})^{-1}\rVert\le 1 + 2\Lambda Lh$. Because $\lVert\ln D(t+\delta) - \ln D(t)\rVert_\infty\le L\delta$, $\Lambda$ is bounded below by $\Lambda_t = \lVert\dot{\hat K}(t)\hat K(t)^{-1}\rVert_2/L$, which the script computes.

**Lemma 5 (operator smoothing).** Under (H1) and (H3),

$$
\begin{aligned}
&\lVert\hat K(D(t))\,\mathbf z(t)\rVert\le\frac{c_z}{t}\lVert\mathbf z_0\rVert,\quad c_z = \tfrac{1}{\sqrt2}e^{(\Lambda L + L/2)T};\\
&\lVert\hat K(\tilde D(s))\,\mathbf w(s)\rVert\le\frac{c_w}{T-s}\lVert\boldsymbol\phi\rVert,\quad c_w = \tfrac{1}{\sqrt2}e^{(2\Lambda L + L)T}.
\end{aligned} \tag{P22.137}
$$

Proof. Let $F = \lVert\hat K\mathbf z\rVert^2$. Then $\dot F = 2(\hat K\mathbf z)^{\mathsf T}(\dot{\hat K}\hat K^{-1})(\hat K\mathbf z) - 2\lVert\hat K^{3/2}\mathbf z\rVert^2 - 2\lambda F\le 2\Lambda LF - 2F^2/E$, because $F^2\le E\,\lVert\hat K^{3/2}\mathbf z\rVert^2$. With Lemma 4, $F^2/E\ge 2te^{-Lt}F^2/m_0$. For $u = 1/F$, $\tfrac{d}{dt}(e^{2\Lambda Lt}u)\ge 4te^{-Lt}/m_0$, so $u(t)\ge e^{-(2\Lambda L + L)t}\,2t^2/m_0$. For $\mathbf w$ use reversed time, Lemma 4 (b), and at each switch the factor $(1 + 2\Lambda Lh)^2\le e^{4\Lambda Lh}$ on $F$. $\square$

Face quadrature defect. With $\tilde G_f = G_f(\tilde D_k)$ define

$$
\gamma_k = \max_f\Big\lvert\frac{1}{h\,\tilde G_f}\int_{I_k}G_f(D(s))\,ds - 1\Big\rvert;\qquad \text{midpoint: }\ \gamma_k\le\frac{h^2}{24}\big(L_2 + 2L^2\big)e^{Lh/2}. \tag{P22.138}
$$

The midpoint bound is the midpoint quadrature remainder with $\lvert\ddot G_f\rvert\le(L_2 + 2L^2)G_f$ from (P22.126). For the per-layer average only the interface faces contribute (all other $G_f$ are linear in one $D_i$), and with $D_\pm(s) = \bar D_\pm(1 + u_\pm(s))$, $\int_{I_k}u_\pm\,ds = 0$, $\hat u = \theta_fu_+ + (1-\theta_f)u_-$, the exact identity (SymPy check S0.1)

$$
\begin{aligned}
&\frac{G_f(D)}{G_f(\bar D)} = 1 + \theta_fu_- + (1-\theta_f)u_+ - \frac{\theta_f(1-\theta_f)(u_- - u_+)^2}{1 + \hat u}\\
&\qquad\ \Longrightarrow\ \gamma_k\le e^{Lh}\sinh^2(Lh)\ \ (\text{average, every } h)
\end{aligned} \tag{P22.139}
$$

shows a defect of order $h^2$ that is always of one sign (the harmonic conductance is concave). The bound uses $\theta_f(1-\theta_f)\le\tfrac14$, $u_\pm\in[e^{-Lh}-1,\,e^{Lh}-1]$ (both $D_\pm(s)$ and $\bar D_\pm$ lie in the range of $D_\pm$ on $I_k$), hence $\lvert u_- - u_+\rvert\le 2\sinh(Lh)$ and $1 + \hat u\ge e^{-Lh}$. (An earlier form, $\gamma_k\le(e^{Lh}-1)^2/(2 - e^{Lh})$, needs $e^{Lh}<2$; in the test problem $e^{LT/4} = 2.41$, so it did not cover $N_T = 4$ in Theorem D. The form above holds for every $h$ and has the same leading term $(Lh)^2$.) This is the rigorous form of the "local defect $O(h^3)$" statement after (P22.55).

**Theorem D.** Under (H0) to (H3) and $N_T\ge 4$, for the midpoint rule ($\bar\rho = Lh/2$) and the per-layer average rule ($\bar\rho = Lh$), with $\bar\gamma = \max_k\gamma_k$,

$$
\begin{aligned}
\lVert\mathbf e(T)\rVert_V&\le\Big[\Lambda\bar\rho\,\frac{h}{T-h}\,(c_w + c_z) + \frac{\pi}{2}\,e^{L(T + h/2)}\,\bar\gamma\\
&\qquad + \frac{4\Lambda\bar\rho\,c_wc_z}{T}\,h\,\ln\frac{T}{h}\Big]\lVert C_0\rVert_V\\
&\ \le\ C_D\,h^2\Big(1 + \ln\frac{T}{h}\Big)\lVert C_0\rVert_V,
\end{aligned} \tag{P22.140}
$$

where $C_D$ depends on $\Lambda$, $L$, $L_2$ and $T$ only, not on $\lVert A\rVert$.

Proof. Split (P22.127) into $J_k = \int_{I_k}\mathbf w^{\mathsf T}\hat\Delta\mathbf z\,ds$ and take $\lVert\boldsymbol\phi\rVert = 1$.

First interval. $\lvert J_0\rvert\le\int_{I_0}\lVert\hat\Delta\hat K(\tilde D_0)^{-1}\rVert\,\lVert\hat K(\tilde D_0)\mathbf w\rVert\,\lVert\mathbf z\rVert\,ds\le\Lambda\bar\rho\,c_w\ln\tfrac{T}{T-h}\le\Lambda\bar\rho\,c_w\tfrac{h}{T-h}$, by (P22.136), (P22.137) and $\lVert\mathbf z\rVert\le\lVert\mathbf z_0\rVert$.

Last interval. In the same way with the roles exchanged, $\lvert J_{N_T-1}\rvert\le\Lambda\bar\rho\,c_z\tfrac{h}{T-h}$, using $\lVert\mathbf w\rVert\le\lVert\boldsymbol\phi\rVert$.

Interior intervals, $1\le k\le N_T-2$. With $\hat\Phi_k(s) = \int_{t_k}^s\hat\Delta\,d\sigma$, integration by parts and the two differential equations give (the $\lambda$ terms cancel)

$$
J_k = \mathbf w(t_{k+1})^{\mathsf T}\hat\Phi_k(t_{k+1})\mathbf z(t_{k+1}) - \int_{I_k}\Big[\big(\hat K(\tilde D_k)\mathbf w\big)^{\mathsf T}\hat\Phi_k\mathbf z - \big(\hat\Phi_k\mathbf w\big)^{\mathsf T}\hat K(D(s))\mathbf z\Big]ds.
$$

The boundary term has face coefficients $\lvert\int_{I_k}(G_f - \tilde G_f)\rvert\le h\gamma_k\tilde G_f\le h\gamma_ke^{Lh/2}\sqrt{\tilde G_fG_f(t_{k+1})}$, so by the face Cauchy-Schwarz step and Lemma 4 it is at most $h\gamma_ke^{L(T+h/2)}/\big(2\sqrt{t_{k+1}(T - t_{k+1})}\big)$. The function $1/\sqrt{s(T-s)}$ is convex, so the sum of $h/\sqrt{t_{k+1}(T-t_{k+1})}$ is at most $\int_0^T ds/\sqrt{s(T-s)} = \pi$ (S0.6). In the integral, $\lVert\hat\Phi_k(s)\hat K(D(s))^{-1}\rVert$ and $\lVert\hat\Phi_k(s)\hat K(\tilde D_k)^{-1}\rVert$ are at most $h\Lambda\bar\rho$ by (P22.136), so with (P22.137) the integrand is at most $2h\Lambda\bar\rho\,c_wc_z/(s(T-s))$, and $\int_h^{T-h}ds/(s(T-s)) = \tfrac{2}{T}\ln\tfrac{T-h}{h}$ (S0.6). Adding the three parts gives (P22.140); each is $O(h^2)$ or $O(h^2\ln(T/h))$ because $\bar\rho = O(h)$ and $\bar\gamma = O(h^2)$. $\square$

##### 5.4.6.7 Why the order-2 bound is not uniform under mesh refinement

The constant of (P22.140) is proportional to $\Lambda$ (and to $\Lambda$ in the exponents of $c_z$, $c_w$). Applied to a discrete solution whose flux is continuous across an interface, $\dot{\hat K}\hat K^{-1}$ produces in each layer the multiple $\ell_i = Q_i\dot T/(R_gT^2)$ of the layer operator, plus a term $(\ell_{i+1} - \ell_i)J_f$ concentrated in the two cells next to the interface face $f$. Its $V$-norm is of size $\lvert J_f\rvert/\sqrt{V_j}\propto(\Delta r)^{-1/2}$, so $\Lambda_t\propto N^{1/2}$, which is $\lVert A\rVert^{1/4}$ because $\lVert A\rVert\propto N^2$. The script confirms this (S7.1): $\Lambda_t$ = 3.63, 5.06, 7.10, 10.00, 14.12 for 10 to 160 cells per layer, successive ratios 1.394 to 1.412 against $\sqrt2 = 1.414$; with shared scaling $\Lambda_t = 1$ on every mesh (S7.2). Hence Theorem D is uniform in $h$ for a given mesh, but its constant grows with the mesh stiffness, and Theorem A is the strongest bound PROVED here that is uniform in both.

Heuristic for the observed order (CLAIMED ONLY). For an eigenpair $(\mu_n,\boldsymbol\psi_n)$ of the staircase matrix on the last intervals, the midpoint rule creates per interval the error component $-\big(\int_0^he^{-\mu_nu}(\tfrac h2 - u)\,du\big)\,c_n$ with $c_n = \boldsymbol\psi_n^{\mathsf T}\dot{\hat K}\mathbf z$, and

$$
\int_0^he^{-\mu u}\big(\tfrac h2 - u\big)\,du = \frac h2\,\frac{1 - e^{-\mu h}}{\mu} - \frac{1 - e^{-\mu h}(1 + \mu h)}{\mu^2}\ \approx\ \frac{h}{2\mu}\quad(\mu h\gg1). \tag{P22.141}
$$

Stiff components are damped by $e^{-\mu h}$ per interval and do not accumulate, so $\lVert\mathbf e_{\mathrm{stiff}}\rVert^2\approx\tfrac{h^2}{4}\sum_{\mu_nh>1}\mu_n^{-2}c_n^2$. The interface term of $\dot{\hat K}\mathbf z$ is a point load, whose coefficients $c_n$ do not decay with $n$, while $\mu_n\propto n^2$; with $n_h\propto h^{-1/2}$,

$$
\lVert\mathbf e(T)\rVert_V\propto h\,\Big(\sum_{n>n_h}n^{-4}\Big)^{1/2}\propto h\cdot h^{3/4} = h^{7/4}\qquad(\text{layer-specific } Q_i,\ 1/\lVert A\rVert\ll h). \tag{P22.142}
$$

The non-stiff components give the same power (per-mode error about $\tfrac{h^2}{12}c_n$ summed over $n<n_h$). Without the interface term (shared scaling) the same sums give $h^2$, in line with Theorem C, and a smooth functional such as the release fraction weights the high modes weakly and keeps order 2. The heuristic is consistent with Theorem D: $h^{7/4}\le\Lambda h^2$ exactly when $h\gtrsim 1/\lVert A\rVert$. It is not a proof: the uniform size of $c_n$ and the neglect of interaction between modes are assumed. In the second synthetic case of the verification record in Appendix B.5.7 the orders at the finest steps are 1.51 to 1.57 ($N_T\le 8192$), so that case has not reached, and does not test, the conjectured limit.

##### 5.4.6.8 Numerical verification on the stiff five-layer matrix

Script `sec5_o10_stiff_staircase.py`: Python transcription of v1.0 Code B (`build_mesh`, equal-width cells per layer), synthetic data of `sec5_staircase.py`, ramp 1600 to 1800 $^\circ$C in 4 h, kernel indicator as initial datum normalised to $\lVert C_0\rVert_V = 1$, reference Radau IIA (rtol $10^{-13}$, cross-checked against rtol $10^{-11}$). Errors are $\lVert\mathbf e(4\ \mathrm h)\rVert_V$; orders are $\log_2(e_{N_T}/e_{2N_T})$.

| Mesh (cells per layer) | Rule | $h\lVert A\rVert$ of the pairs | Observed $V$-norm orders | Release fraction $F(4\ \mathrm h)$ orders |
|---|---|---|---|---|
| 20 | left | 1138 to 142 | 1.00 | 1.00 |
| 20 | midpoint | 1138 to 142 | 1.77 to 1.81 | 2.04 to 2.13 |
| 20 | average | 1138 to 142 | 1.76 to 1.80 | 1.98 to 1.99 |
| 40 | midpoint | 1143 to 286 | 1.77 to 1.78 | 2.05 to 2.06 |
| 40 | average | 1143 to 286 | 1.76 to 1.78 | 1.99 |
| 80 (Cauchy, no reference) | midpoint | 9158 to 1145 | 1.75 to 1.76 | not computed |

Table: 5.12 Observed staircase orders in the stiff regime (values from the script output in Appendix A.2).

A smooth initial datum (the slowest eigenvector of $\hat K(D(0))$) gives nearly the same orders on 20 cells per layer: 1.768 to 1.806 (midpoint) and 1.765 to 1.804 (average) for $N_T$ = 256 to 2048 (DEMONSTRATED NUMERICALLY, check V11.4 of the verification record in Appendix B.5.7; the author's preliminary run had reported 1.766 to 1.794). It indicates that the reduction comes from the interface term; the discontinuity of the initial datum would then play no part in it. The values in Table 5.5 for $F(6\ \mathrm h)$ remain correct; they test a functional.

##### 5.4.6.9 Result

- PROVED (Theorem A, (P22.129), (P22.130)): every staircase rule converges with order at least 1 in the $V$-norm, uniformly in $h\lVert A\rVert$, the mesh and $\lambda$, for any temperature history with Lipschitz $\ln D_i(T(t))$; constants $\tfrac12(e^{Lh} - 1)$ (left, average) and $\tfrac12(e^{Lh/2} - 1)$ (midpoint). For the left rule order 1 is sharp (P22.131).
- PROVED (Theorem B): breakpoints of a piecewise-constant history on the grid give the exact solution; a breakpoint inside an interval costs order 1 for every rule (P22.133).
- PROVED (Theorem C): with shared scaling the midpoint rule has order 2 uniformly in the stiffness (P22.135) and the average rule is exact.
- PROVED (Theorem D): with layer-specific activation energies and $\ln D_i\in C^2$, midpoint and average rules satisfy $\lVert\mathbf e\rVert_V\le C_Dh^2(1 + \ln(T/h))$ on a fixed mesh, with $C_D$ free of $\lVert A\rVert$ but proportional to the operator-relative constant $\Lambda$.
- DEMONSTRATED NUMERICALLY: $\Lambda\ge\Lambda_t\propto N^{1/2}$; in the window $h\lVert A\rVert\in[10^2, 1.2\times10^3]$ the $V$-norm orders of midpoint and average are 1.76 to 1.81, below 2, while the release-fraction orders stay between 1.98 and 2.13.
- CLAIMED ONLY: the limiting $V$-norm order under mesh refinement is $7/4$ (P22.142).
- OPEN: a proof of an order above 1 for midpoint and average that is uniform in the mesh stiffness with layer-specific $Q_i$ (for example $h^{7/4}$ or $h^{3/2}$); it needs regularity of the semi-discrete solution at the interfaces, which energy and logarithmic-norm arguments alone do not provide.

Statement (P22.56) should read: left order 1 (PROVED, uniform); midpoint and average order 2 for the release fraction (DEMONSTRATED NUMERICALLY) and for the $V$-norm on a fixed mesh once $h\lVert A\rVert$ is moderate, up to a factor $1+\ln(T/h)$ (PROVED, Theorem D), with an observed $V$-norm order of about 1.75 in the test case when $h\lVert A\rVert\gg1$ and layer-specific $Q_i$ (between 1.51 and 1.96 in a second synthetic case).

##### 5.4.6.10 Limitations

- $S = 0$ throughout. With a source, Lemma 3 gains the source work and Theorem A gains a term $\tfrac12\bar\eta$ times the $L^2(0,T;\hat K^{-1})$ size of the source; not written out.
- The proofs use the FV structure through Lemma 1 only (monotone, degree-one homogeneous face conductances). They apply to the v1.0 harmonic face and absorbing outer face; a Robin surface (P22.38) adds a face of the same kind and is covered, but partition coefficients or non-symmetric terms are not.
- Theorem D's constant contains $c_wc_z = \tfrac12e^{(3\Lambda L + 3L/2)T}$; with $\Lambda\ge\Lambda_t\approx 5$ (20 cells per layer) to 7 (40 cells per layer) and $LT\approx 3.5$ in the test it is far from sharp; it establishes the order and is too large to serve as an error estimate.
- The staircase is assumed to be propagated exactly on each interval (as in blocks R3 and R4). An additional per-interval eigen-series truncation error (open item O4) adds to the bounds.
- The author's script runs one synthetic parameter set and one ramp. The verification record (Appendix B.5.7) adds a second three-layer case, in which the midpoint $V$-norm orders are mesh-converged (the same on 15 and 30 cells per layer) and fall from 1.96 to 1.51 before rising again within $N_T\le 8192$. The order reduction therefore depends on the case and on $h$ relative to the physical layer time scales, and the value 1.75 is specific to the test case.

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
| `sec5_arrhenius.py` | (P22.3) to (P22.8), (P22.19), (P22.22), (P22.23); $R_g$; CRP-6 1a/1b; crossovers; fluence factor; step bounds; validator | 25/25 PASS |
| `sec5_effective_time.py` | SymPy proof of (P22.27) to (P22.38), closure, (P22.50) for a single sphere, (P22.18) | 16/16 PASS |
| `sec5_fv_shared_alpha.py` | independent FV vs independent series at $\tau(t)$; BE options; decay factorisation and (P22.35) | 10/10 PASS |
| `sec5_projection_step.py` | projection (P22.42) to (P22.45) vs extrapolated FV; Galerkin diagonal test (P22.50); coefficient decay rate implied by (P22.46) (the identity itself is tested only by the checker script `sec5chk_numeric.py`) | 9/9 PASS |
| `sec5_staircase.py` | Magnus terms (P22.54), (P22.55); staircase orders (P22.56); BE orders without shared $\alpha$ | 9/9 PASS |
| `sec5_schedules.py` | AGR-1 plan (P22.58), CRP-6 conversion (P22.59), ramp shares, thermocouple sensitivity | 17/17 PASS |
| `sec5_o12_variable_D_step_limit.py` (2026-11-13) | Section 5.2.5: step limit with $D_i(T(t))$, (P22.60) to (P22.80); exact modal errors, step rule, option E rule, layer-specific $Q$ runs | 38 PASS, ALL PASS |
| `sec5_r24_quadrature_start.py` (2026-11-13) | Section 5.3.7: start of the release quadrature (P22.35), (P22.81) to (P22.99); image form, tail bounds, start and late bounds, cold start | 25/25 PASS |
| `sec5_o11_parent_daughter_tau.py` (2026-11-13) | Section 5.3.8: parent-daughter effective time, (P22.100) to (P22.120); Theorem 1, Bateman mixture, two-clock cascade, surrogate bounds | 24/24 PASS |
| `sec5_o10_stiff_staircase.py` (2026-11-13) | Section 5.4.6: staircase in the stiff regime, (P22.121) to (P22.142); Theorems A to D and the observed orders | 22/22 PASS |

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
- Staircase in the stiff regime (O10, updated 2026-11-13, Section 5.4.6): order 1 uniform in the stiffness is PROVED (P22.130); order 2 is PROVED for shared scaling (P22.135) and, up to a factor $1+\ln(T/h)$ on a fixed mesh with a mesh-dependent constant, for layer-specific $Q$ (Theorem D, (P22.140)). OPEN: an order above 1 that is uniform in the mesh stiffness for layer-specific $Q$ (observed $V$-norm order about 1.75 in the test case and 1.51 to 1.96 in a second synthetic case; the limit 7/4 is CLAIMED ONLY).
- The two-path picture behind (P22.1) assumes parallel transport in local equilibrium; no source in the corpus justifies it for the fitted TECDOC-978 rows beyond Eq. (A-1) itself.
- The POD affine split (P22.52) is inexact at interface faces because of the harmonic conductance; not implemented or tested.
- Temperature-dependent partition coefficients (A2) would change the projection weight; out of scope until a data source exists.
- The values of $N_A$ and $k_B$ were read in the SI Brochure (BIPM, 2019, Section 2.2 and Table 1, p. 124); SciPy's constants table is only a numerical cross-check.
- Diffusion step limit with $D_i(T(t))$ (O12, added 2026-11-13, Section 5.2.5): the rule (P22.69) with the step (P22.72) is PROVED for shared scaling, option A; option E is bounded by (P22.74) with $\zeta_E$, which gives $\varepsilon_1+\tfrac12\zeta_E^2\,\mathrm{TV}(\ln\Lambda_1)$ (PROVED, shared scaling). For layer-specific activation energies only the frozen-coefficient part is bounded (P22.78; option A for any history, option E on heating only); the non-adiabatic remainder $R_\mathrm{na}$ is CLAIMED ONLY to be $O(\varepsilon_D)$, with numerical support, and a rigorous relative bound is OPEN. A sharp a priori bound for the release fraction with decay is OPEN, and a source term is not covered.
- Start of the release quadrature with decay (R24, added 2026-11-13, Section 5.3.7): for an absorbing surface and $S = 0$ the start bound (P22.93) is PROVED for constant $\alpha$ and the bracket (P22.94) is PROVED for varying $\alpha$ and $c_0\ge0$, and the total bound (P22.98) is PROVED when $\alpha$ is constant on the start piece and on every graded piece and under the coverage assumption R24-A3c (late truncation (P22.95)). OPEN: a layered particle with a long cold phase (Step 8 of Section 5.3.7). CLAIMED ONLY: the graded Gauss-Legendre bound (P22.97) on ramp pieces. Not treated: the Robin surface of E4 and a source.
- Parent and daughter with different diffusivities (O11, added 2026-11-13, Section 5.3.8): truncation of the two-clock cascade (P22.114) has no proved bound (OPEN). The a priori surrogate bound (P22.119) is at least 0.23 for the TECDOC-978 Cs layer ratios on a history that reaches both the minimum of the ratio near 1438 $^\circ$C and 1800 $^\circ$C, so the surrogate route is a diagnostic only. Transformations outside the class (P22.101) are OPEN.

## References

The reference lists of the three drafts were merged into one list. Notes in parentheses give the locations used in this document and the verification status of identifiers. Page numbers are printed page numbers.

Basu, S. K., & McCutchan, E. A. (2020). Nuclear data sheets for A = 90. *Nuclear Data Sheets, 165*, 1–329. https://doi.org/10.1016/j.nds.2020.04.001

Bolley, C., & Crouzeix, M. (1978). Conservation de la positivité lors de la discrétisation des problèmes d'évolution paraboliques. *RAIRO. Analyse numérique, 12*(3), 237–245. (Numdam item `M2AN_1978__12_3_237_0`. Théorème 1, p. 241; Lemme 3 and Théorème 2, p. 242; the Crank-Nicolson condition $\Delta t\,\sigma(A)\le2$, p. 243; Théorème 3, p. 244; checked against the Numdam scan in the verification of Section 4.6.1, Appendix B.5.3. DOI NOT VERIFIED.)

Browne, E., & Tuli, J. K. (2007). Nuclear data sheets for A = 137. *Nuclear Data Sheets, 108*, 2173–2318. https://doi.org/10.1016/j.nds.2007.09.002 (Cs-137 half-life 30.08 y, via audit 07.)

Bureau International des Poids et Mesures. (2019). *The International System of Units (SI)* (9th ed., version 4.01, June 2026). BIPM. https://doi.org/10.59161/AUEZ1291 (Section 2.2, pp. 123–124, and Table 1, p. 124; kelvin, p. 129, and mole, p. 130, both in Section 2.3.1. English text PDF opened from https://www.bipm.org/en/publications/si-brochure on 2026-11-13; earlier versions of the 9th edition were not opened.)

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

Melrose, R. (2009). *Lecture notes for 18.102 Introduction to functional analysis, Spring 2009* (Lecture 20, Theorem 15, p. 116) [Course notes]. MIT OpenCourseWare. https://ocw.mit.edu/courses/18-102-introduction-to-functional-analysis-spring-2009/ (The instructor is named on the course page; the lecture PDF prints no author. Read on 2026-11-13 for the spectral theorem used in Section 3.10; the compactness of an operator with a square-integrable kernel is not in this lecture.)

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

**Re-run of 2026-11-13.** For the open-item integration all 17 scripts of Table A.1 were run again from the repository root. Each exited with code 0, and their PASS, FAIL, SUMMARY and ALL PASS lines agree with Appendix A.1 line for line (251 PASS, 0 FAIL). The six scripts added for the open items were run in the same pass; their full output is in Appendix A.2.

| Script | Section | What it checks | PASS | FAIL |
|---|---|---|---|---|
| `sec3_o4_layered_truncation_bound.py` | 3.5.3 | truncation bounds for the layered particle, (P21.93) to (P21.112): layered shooting against the v1.0 golden value, Bound A and Bound B against 40000-mode references | 91 | 0 |
| `sec4_p2_positive_second_order.py` | 4.6.1 | second-order integrators and positivity, (P21.113) to (P21.142): orders, barrier, thresholds, closures, runs on the synthetic case | 41 | 0 |
| `sec5_o12_variable_D_step_limit.py` | 5.2.5 | step limit with $D_i(T(t))$, (P22.60) to (P22.80) | 38 | 0 |
| `sec5_r24_quadrature_start.py` | 5.3.7 | start of the release quadrature with decay, (P22.81) to (P22.99) | 25 | 0 |
| `sec5_o11_parent_daughter_tau.py` | 5.3.8 | parent and daughter with different diffusivities, (P22.100) to (P22.120) | 24 | 0 |
| `sec5_o10_stiff_staircase.py` | 5.4.6 | staircase in the stiff regime, (P22.121) to (P22.142) | 22 | 0 |
| total | | | 241 | 0 |

Table: A.3 Open-item check scripts, run of 2026-11-13. Each ends with ALL PASS.

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

### A.2 Result lines of the open-item scripts (2026-11-13)

The complete output of the six scripts added for the open items, from the run of 2026-11-13 made for this integration from the repository root with the command above. Lines are wrapped at 100 characters as in Appendix A.1. The scripts print the placeholder labels (P2X.O4.k and so on) of the drafts; Table B.7 gives the final equation number of each label. For every script this output is line for line identical to the output quoted in its draft.

`sec3_o4_layered_truncation_bound.py` (Section 3.5.3):

```text
PASS  S1 closed-form layer integrals of u^2 and r u
PASS  S2 Green identity sum beta_n^2/Lambda_n = (1/9) int x^4/D dx (homogeneous sphere)
PASS  S3 d/dM Phi(b, M) = -exp(-b M^2)/M^2
PASS  S4 h_n = mu_n + lambda int mu_n = (Lambda mu_n + lambda (M0 + G t)) / kappa (constant source)
PASS  I1 synthetic input read: five radii and five diffusivities  R = [0.0002, 0.0003, 0.00035,
    0.0004, 0.00045], D = [1e-12, 4e-10, 8e-13, 2e-14, 7e-13]
---- case SYN: 5 layers, NREF = 40000
INFO  SYN tau = 674.217 s^0.5, a = pi^2/tau^2 = 2.17121e-05 1/s, c_min = 9.74776e-07 1/s, threshold
    Lambda* = 7e-05 1/s, J = 4
PASS  SYN E1 bisection brackets from min-max are valid for all 40000 modes
PASS  SYN E2 psi(R) = n pi at every root  max |psi - n pi| = 9.0e-08
PASS  SYN E3 beta_n by moments equals the flux identity (P21.36)  max rel diff 1.2e-12
PASS  SYN E4 eigenvalues strictly increasing
INFO  SYN negative weights among the first 300 modes: 151; among 40000: 19995
PASS  SYN E5 eigenvalues agree with 60-digit mpmath roots (n = 1 ... 10000)  max rel diff 1.6e-15
PASS  SYN E6 sign changes of u(R; Lambda) below (Lambda_200 + Lambda_201)/2 equal 200  count 200
PASS  SYN E7 mode n has n - 1 interior zeros (n = 1..40, 100, 300, 1000)
PASS  SYN B1 min-max: D_min (n pi/R)^2 <= Lambda_n <= D_max (n pi/R)^2  min Lambda_n/(c n^2) = 3.358
PASS  SYN B2 travel time: n - J < sqrt(Lambda_n) tau/pi < n + J  range of sqrt(Lambda_n) tau/pi - n:
    [-1.218, 1.216]
PASS  SYN B3 every interface jump of psi is below pi  max |delta|/pi = 0.857
PASS  SYN B4 Lambda_n >= ell_n (combined lower bound)  Lambda_n/ell_n at n = 10, 100, 40000: 2.604,
    1.104, 1.0002
PASS  SYN B5 |alpha_n| <= C_alpha(Lambda_n)/sqrt(Lambda_n) for Lambda_n > Lambda*  max ratio 0.999
PASS  SYN B6 |beta_n| <= C_beta(Lambda_n)/sqrt(Lambda_n) for Lambda_n > Lambda*  max ratio 0.703
PASS  SYN B7 |w_n| <= C_w(Lambda_n)/Lambda_n for Lambda_n > Lambda*  max ratio 0.693, median 0.034,
    modes covered 39999/40000
PASS  SYN B8 C_w(Lambda) and C_alpha(Lambda) decrease above Lambda*
INFO  SYN C_w(inf) = 0.00028237 1/s, C_w(ell_11) = 0.00034478 1/s
PASS  SYN G1 closed forms Q_aa, Q_bb, Q_ab equal the quadrature of F_f F_g/(x^2 D)
PASS  SYN G5 Bessel: 0 <= R1^3/3 - sum alpha^2 <= C_alpha^2 S1(NREF)  defect rel 4.88e-05, bound rel
    1.73e-04
PASS  SYN G6 Bessel: 0 <= R^3/3 - sum beta^2 <= C_beta^2 S1(NREF)  defect rel 1.90e-05, bound rel
    2.15e-04
PASS  SYN G7 |1 - sum w_n| <= C_w S1(NREF)  1 - sum w = -1.50e-10, bound 3.25e-04
PASS  SYN E8 F(2e5 s) equals the v1.0 golden value  0.4325556390931 vs 0.4325556390928
INFO  SYN N values tested: [5, 10, 20, 50, 100, 200, 300, 500, 1000, 2000, 5000]; Lambda_1 =
    3.27326e-06 1/s
INFO  SYN Bound B evaluated (Green defects above 1e-9 of the totals) for N in [5, 10, 20, 50, 100,
    200, 300]
PASS  SYN G2 0 <= Q_aa - sum_(n<=N) alpha^2/Lambda <= C_alpha^2 S2(N), N in [5, 10, 20, 50, 100,
    200, 300]
PASS  SYN G3 0 <= Q_bb - sum_(n<=N) beta^2/Lambda <= C_beta^2 S2(N)
PASS  SYN G4 |Q_ab - sum_(n<=N) alpha beta/Lambda| <= V C_w S2(N)
PASS  SYN G4b at NREF the Green defects are at round-off (below 1e-12 relative)  2.7e-13
PASS  SYN T1 S1 (lambda = 0, no source): |sum_(n>N) w_n e^(-Lambda_n t)| <= Bound A (theta = 0 and
    theta-minimised), all N, t (scaled by e^(Lambda_(N+1) t))  tightness |err|/A: max 1.76e-02,
    median 6.56e-04
PASS  SYN T2 S1: |error| <= Bound B (a posteriori)  tightness |err|/B: max 2.60e-01, median 2.03e-02
PASS  SYN T3 S4 (lambda = 0 steady inventory): NREF tail and exact Green total Q_ab/V - sum_(n<=N)
    w_n/Lambda_n (+ 1000 eps Q_ab/V) <= Bound A  tightness max 1.13e-02, median 5.24e-05
PASS  SYN T4 S4: same error <= Bound B  tightness max 8.20e-02, median 1.52e-02
PASS  SYN T5 S2 steady R/B, lambda = 1 Lambda_1: |error| <= Bound A and <= Bound B  tightness max
    1.13e-02; closed form vs NREF sum differ by 5.1e-14 (Bound A at NREF 1.0e-14)
PASS  SYN T6 S2 the closed-form reference and the NREF sum agree within Bound A(NREF) + 1000 eps
    lambda Q_ab/V
PASS  SYN T5 S2 steady R/B, lambda = 100 Lambda_1: |error| <= Bound A and <= Bound B  tightness max
    1.12e-02; closed form vs NREF sum differ by 5.1e-12 (Bound A at NREF 1.0e-12)
PASS  SYN T6 S2 the closed-form reference and the NREF sum agree within Bound A(NREF) + 1000 eps
    lambda Q_ab/V
PASS  SYN T6b mode-count rule: N = 92 modes give steady R/B error <= 1e-06 (lambda = Lambda_1)  true
    error 5.24e-12
PASS  SYN T6b mode-count rule: N = 873 modes give steady R/B error <= 1e-09 (lambda = Lambda_1)
    true error 1.10e-13
PASS  SYN T7 S3 identity h_n = (Lambda mu + lambda (M0 + Mgen))/kappa, lambda = 1 Lambda_1
PASS  SYN T8 S3 balance-form release error <= Bound A, lambda = 1 Lambda_1  tightness max 1.13e-02,
    median 5.20e-05
PASS  SYN T9 S3 inventory error |M - M^(N)| <= C_w (M0 E_N + G S2), lambda = 1 Lambda_1
PASS  SYN T10 S3 balance-form release error <= Bound B, lambda = 1 Lambda_1  tightness max 8.15e-02,
    median 1.52e-02
PASS  SYN T7 S3 identity h_n = (Lambda mu + lambda (M0 + Mgen))/kappa, lambda = 100 Lambda_1
PASS  SYN T8 S3 balance-form release error <= Bound A, lambda = 100 Lambda_1  tightness max
    1.12e-02, median 4.69e-05
PASS  SYN T9 S3 inventory error |M - M^(N)| <= C_w (M0 E_N + G S2), lambda = 100 Lambda_1
PASS  SYN T10 S3 balance-form release error <= Bound B, lambda = 100 Lambda_1  tightness max
    5.46e-02, median 1.46e-02
PASS  SYN T11 L2(r^2) profile error^2 <= C_alpha^2 E_N(2t) (t = 0 by Bessel exactly)  tightness max
    2.82e-01, median 6.55e-02
INFO  SYN S4 observed order for 50 <= N <= 300: true error 5.31, Bound A 3.13; reference uncertainty
    (Bound A at NREF) 3.1e-09 s, so rows with |error| below it are indicative only
INFO  SYN table: scenario | parameter | N | |error| | Bound A | Bound B
INFO  SYN S1 | 305.5 | 10 | 5.095e-03 | 1.895e+00 | 1.369e-01 | values scaled by exp(0.7001)
INFO  SYN S1 | 305.5 | 50 | 1.348e-03 | 7.655e-02 | 2.904e-02 | values scaled by exp(17.01)
INFO  SYN S1 | 305.5 | 300 | 1.068e-05 | 1.866e-03 | 4.570e-03 | values scaled by exp(597.4)
INFO  SYN S1 | 305.5 | 1000 | 9.034e-07 | 9.959e-03 | nan | values scaled by exp(6649)
INFO  SYN S1 | 3.055e+04 | 10 | 5.691e-03 | 2.647e+00 | 1.294e-01 | values scaled by exp(70.01)
INFO  SYN S1 | 3.055e+04 | 50 | 1.374e-03 | 2.900e-01 | 2.904e-02 | values scaled by exp(1701)
INFO  SYN S1 | 3.055e+04 | 300 | 1.068e-05 | 4.411e-02 | 4.570e-03 | values scaled by exp(5.974e+04)
INFO  SYN S1 | 3.055e+04 | 1000 | 1.955e-06 | 1.307e-02 | nan | values scaled by exp(6.649e+05)
INFO  SYN S4 | inf | 10 | 2.238e+00 | 1.129e+03 | 5.646e+01
INFO  SYN S4 | inf | 50 | 2.377e-02 | 2.104e+00 | 5.216e-01
INFO  SYN S4 | inf | 100 | 3.405e-04 | 2.284e-01 | 6.171e-02
INFO  SYN S4 | inf | 300 | 5.120e-06 | 7.729e-03 | 2.337e-03
INFO  SYN S4 | inf | 1000 | 1.981e-09 | 2.023e-04 | nan
INFO  SYN S4 | inf | 2000 | 4.584e-11 | 2.512e-05 | nan
INFO  SYN S2 | 3.273e-06 | 10 | 7.314e-06 | 3.694e-03 | 1.848e-04
INFO  SYN S2 | 3.273e-06 | 50 | 7.779e-08 | 6.888e-06 | 1.707e-06
INFO  SYN S2 | 3.273e-06 | 100 | 1.115e-09 | 7.477e-07 | 2.020e-07
INFO  SYN S2 | 3.273e-06 | 300 | 1.676e-11 | 2.530e-08 | 7.651e-09
INFO  SYN S2 | 3.273e-06 | 1000 | 6.485e-15 | 6.622e-10 | nan
INFO  SYN S2 | 3.273e-06 | 2000 | 1.500e-16 | 8.223e-11 | nan
INFO  SYN S2 | 0.0003273 | 10 | 6.388e-04 | 3.694e-01 | 1.848e-02
INFO  SYN S2 | 0.0003273 | 50 | 7.734e-06 | 6.888e-04 | 1.707e-04
INFO  SYN S2 | 0.0003273 | 100 | 1.113e-07 | 7.477e-05 | 2.020e-05
INFO  SYN S2 | 0.0003273 | 300 | 1.676e-09 | 2.530e-06 | 7.651e-07
INFO  SYN S2 | 0.0003273 | 1000 | 6.485e-13 | 6.622e-08 | nan
INFO  SYN S2 | 0.0003273 | 2000 | 1.500e-14 | 8.223e-09 | nan
INFO  SYN S3 | lam=3.27e-06,t=4.583e+05 | 10 | 1.463e-05 | 1.108e-02 | 5.544e-04
INFO  SYN S3 | lam=3.27e-06,t=4.583e+05 | 50 | 1.556e-07 | 2.066e-05 | 5.122e-06
INFO  SYN S3 | lam=3.27e-06,t=4.583e+05 | 300 | 3.352e-11 | 7.590e-08 | 2.295e-08
INFO  SYN S3 | lam=3.27e-06,t=4.583e+05 | 1000 | 1.297e-14 | 1.987e-09 | nan
INFO  SYN S3 | lam=0.000327,t=4.583e+05 | 10 | 1.278e-03 | 7.426e-01 | 3.715e-02
INFO  SYN S3 | lam=0.000327,t=4.583e+05 | 50 | 1.547e-05 | 1.384e-03 | 3.432e-04
INFO  SYN S3 | lam=0.000327,t=4.583e+05 | 300 | 3.351e-09 | 5.085e-06 | 1.538e-06
INFO  SYN S3 | lam=0.000327,t=4.583e+05 | 1000 | 1.297e-12 | 1.331e-07 | nan
---- case B4: 5 layers, NREF = 20000
INFO  B4 tau = 2468.77 s^0.5, a = pi^2/tau^2 = 1.61934e-06 1/s, c_min = 2.28225e-07 1/s, threshold
    Lambda* = 3.125e-06 1/s, J = 4
PASS  B4 E1 bisection brackets from min-max are valid for all 20000 modes
PASS  B4 E2 psi(R) = n pi at every root  max |psi - n pi| = 2.5e-10
PASS  B4 E3 beta_n by moments equals the flux identity (P21.36)  max rel diff 6.0e-15
PASS  B4 E4 eigenvalues strictly increasing
INFO  B4 negative weights among the first 300 modes: 152; among 20000: 10000
PASS  B4 E5 eigenvalues agree with 60-digit mpmath roots (n = 1 ... 10000)  max rel diff 6.6e-16
PASS  B4 E6 sign changes of u(R; Lambda) below (Lambda_200 + Lambda_201)/2 equal 200  count 200
PASS  B4 E7 mode n has n - 1 interior zeros (n = 1..40, 100, 300, 1000)
PASS  B4 B1 min-max: D_min (n pi/R)^2 <= Lambda_n <= D_max (n pi/R)^2  min Lambda_n/(c n^2) = 3.138
PASS  B4 B2 travel time: n - J < sqrt(Lambda_n) tau/pi < n + J  range of sqrt(Lambda_n) tau/pi - n:
    [-0.590, 0.590]
PASS  B4 B3 every interface jump of psi is below pi  max |delta|/pi = 0.429
PASS  B4 B4 Lambda_n >= ell_n (combined lower bound)  Lambda_n/ell_n at n = 10, 100, 20000: 2.684,
    1.089, 1.0004
PASS  B4 B5 |alpha_n| <= C_alpha(Lambda_n)/sqrt(Lambda_n) for Lambda_n > Lambda*  max ratio 0.965
PASS  B4 B6 |beta_n| <= C_beta(Lambda_n)/sqrt(Lambda_n) for Lambda_n > Lambda*  max ratio 0.550
PASS  B4 B7 |w_n| <= C_w(Lambda_n)/Lambda_n for Lambda_n > Lambda*  max ratio 0.579, median 0.266,
    modes covered 19999/20000
PASS  B4 B8 C_w(Lambda) and C_alpha(Lambda) decrease above Lambda*
INFO  B4 C_w(inf) = 6.313e-06 1/s, C_w(ell_11) = 7.2443e-06 1/s
PASS  B4 G1 closed forms Q_aa, Q_bb, Q_ab equal the quadrature of F_f F_g/(x^2 D)
PASS  B4 G5 Bessel: 0 <= R1^3/3 - sum alpha^2 <= C_alpha^2 S1(NREF)  defect rel 4.56e-05, bound rel
    1.19e-04
PASS  B4 G6 Bessel: 0 <= R^3/3 - sum beta^2 <= C_beta^2 S1(NREF)  defect rel 2.28e-05, bound rel
    1.99e-04
PASS  B4 G7 |1 - sum w_n| <= C_w S1(NREF)  1 - sum w = 2.82e-09, bound 1.95e-04
INFO  B4 N values tested: [5, 10, 20, 50, 100, 200, 300, 500, 1000, 2000]; Lambda_1 = 7.16238e-07
    1/s
INFO  B4 Bound B evaluated (Green defects above 1e-9 of the totals) for N in [5, 10, 20, 50, 100,
    200, 300]
PASS  B4 G2 0 <= Q_aa - sum_(n<=N) alpha^2/Lambda <= C_alpha^2 S2(N), N in [5, 10, 20, 50, 100, 200,
    300]
PASS  B4 G3 0 <= Q_bb - sum_(n<=N) beta^2/Lambda <= C_beta^2 S2(N)
PASS  B4 G4 |Q_ab - sum_(n<=N) alpha beta/Lambda| <= V C_w S2(N)
PASS  B4 G4b at NREF the Green defects are at round-off (below 1e-12 relative)  2.2e-13
PASS  B4 T1 S1 (lambda = 0, no source): |sum_(n>N) w_n e^(-Lambda_n t)| <= Bound A (theta = 0 and
    theta-minimised), all N, t (scaled by e^(Lambda_(N+1) t))  tightness |err|/A: max 2.97e-02,
    median 2.90e-03
PASS  B4 T2 S1: |error| <= Bound B (a posteriori)  tightness |err|/B: max 2.92e-01, median 2.46e-02
PASS  B4 T3 S4 (lambda = 0 steady inventory): NREF tail and exact Green total Q_ab/V - sum_(n<=N)
    w_n/Lambda_n (+ 1000 eps Q_ab/V) <= Bound A  tightness max 8.64e-03, median 4.52e-04
PASS  B4 T4 S4: same error <= Bound B  tightness max 2.01e-01, median 8.84e-03
PASS  B4 T5 S2 steady R/B, lambda = 1 Lambda_1: |error| <= Bound A and <= Bound B  tightness max
    8.64e-03; closed form vs NREF sum differ by 1.0e-15 (Bound A at NREF 7.2e-14)
PASS  B4 T6 S2 the closed-form reference and the NREF sum agree within Bound A(NREF) + 1000 eps
    lambda Q_ab/V
PASS  B4 T5 S2 steady R/B, lambda = 100 Lambda_1: |error| <= Bound A and <= Bound B  tightness max
    8.60e-03; closed form vs NREF sum differ by 1.0e-13 (Bound A at NREF 7.2e-12)
PASS  B4 T6 S2 the closed-form reference and the NREF sum agree within Bound A(NREF) + 1000 eps
    lambda Q_ab/V
PASS  B4 T6b mode-count rule: N = 88 modes give steady R/B error <= 1e-06 (lambda = Lambda_1)  true
    error 1.11e-08
PASS  B4 T6b mode-count rule: N = 836 modes give steady R/B error <= 1e-09 (lambda = Lambda_1)  true
    error 2.69e-14
PASS  B4 T7 S3 identity h_n = (Lambda mu + lambda (M0 + Mgen))/kappa, lambda = 1 Lambda_1
PASS  B4 T8 S3 balance-form release error <= Bound A, lambda = 1 Lambda_1  tightness max 8.64e-03,
    median 4.50e-04
PASS  B4 T9 S3 inventory error |M - M^(N)| <= C_w (M0 E_N + G S2), lambda = 1 Lambda_1
PASS  B4 T10 S3 balance-form release error <= Bound B, lambda = 1 Lambda_1  tightness max 1.98e-01,
    median 7.26e-03
PASS  B4 T7 S3 identity h_n = (Lambda mu + lambda (M0 + Mgen))/kappa, lambda = 100 Lambda_1
PASS  B4 T8 S3 balance-form release error <= Bound A, lambda = 100 Lambda_1  tightness max 8.60e-03,
    median 4.32e-04
PASS  B4 T9 S3 inventory error |M - M^(N)| <= C_w (M0 E_N + G S2), lambda = 100 Lambda_1
PASS  B4 T10 S3 balance-form release error <= Bound B, lambda = 100 Lambda_1  tightness max
    8.23e-02, median 8.83e-03
PASS  B4 T11 L2(r^2) profile error^2 <= C_alpha^2 E_N(2t) (t = 0 by Bessel exactly)  tightness max
    3.84e-01, median 7.64e-02
INFO  B4 S4 observed order for 50 <= N <= 300: true error 2.77, Bound A 3.13; reference uncertainty
    (Bound A at NREF) 1.0e-07 s, so rows with |error| below it are indicative only
INFO  B4 table: scenario | parameter | N | |error| | Bound A | Bound B
INFO  B4 S1 | 1396 | 10 | 7.383e-04 | 5.782e-01 | 9.920e-02 | values scaled by exp(0.2997)
INFO  B4 S1 | 1396 | 50 | 4.878e-05 | 2.049e-02 | 1.111e-02 | values scaled by exp(5.856)
INFO  B4 S1 | 1396 | 300 | 1.474e-05 | 5.944e-04 | 1.830e-03 | values scaled by exp(204.6)
INFO  B4 S1 | 1396 | 1000 | 1.067e-06 | 7.491e-05 | nan | values scaled by exp(2264)
INFO  B4 S1 | 1.396e+05 | 10 | 9.179e-03 | 7.456e-01 | 5.988e-02 | values scaled by exp(29.97)
INFO  B4 S1 | 1.396e+05 | 50 | 3.507e-04 | 8.633e-02 | 1.111e-02 | values scaled by exp(585.6)
INFO  B4 S1 | 1.396e+05 | 300 | 1.238e-05 | 1.321e-02 | 1.830e-03 | values scaled by exp(2.046e+04)
INFO  B4 S1 | 1.396e+05 | 1000 | 1.079e-06 | 3.918e-03 | nan | values scaled by exp(2.264e+05)
INFO  B4 S4 | inf | 10 | 1.937e+00 | 4.263e+03 | 2.790e+02
INFO  B4 S4 | inf | 50 | 3.780e-03 | 8.398e+00 | 2.648e+00
INFO  B4 S4 | inf | 100 | 7.906e-03 | 9.151e-01 | 3.336e-01
INFO  B4 S4 | inf | 300 | 1.104e-04 | 3.103e-02 | 1.249e-02
INFO  B4 S4 | inf | 1000 | 6.599e-08 | 8.129e-04 | nan
INFO  B4 S4 | inf | 2000 | 2.833e-08 | 1.010e-04 | nan
INFO  B4 S2 | 7.162e-07 | 10 | 1.388e-06 | 3.054e-03 | 1.998e-04
INFO  B4 S2 | 7.162e-07 | 50 | 2.707e-09 | 6.015e-06 | 1.897e-06
INFO  B4 S2 | 7.162e-07 | 100 | 5.662e-09 | 6.554e-07 | 2.389e-07
INFO  B4 S2 | 7.162e-07 | 300 | 7.904e-11 | 2.223e-08 | 8.945e-09
INFO  B4 S2 | 7.162e-07 | 1000 | 4.726e-14 | 5.822e-10 | nan
INFO  B4 S2 | 7.162e-07 | 2000 | 2.029e-14 | 7.231e-11 | nan
INFO  B4 S2 | 7.162e-05 | 10 | 1.351e-04 | 3.054e-01 | 1.998e-02
INFO  B4 S2 | 7.162e-05 | 50 | 2.643e-07 | 6.015e-04 | 1.897e-04
INFO  B4 S2 | 7.162e-05 | 100 | 5.638e-07 | 6.554e-05 | 2.389e-05
INFO  B4 S2 | 7.162e-05 | 300 | 7.900e-09 | 2.223e-06 | 8.945e-07
INFO  B4 S2 | 7.162e-05 | 1000 | 4.726e-12 | 5.822e-08 | nan
INFO  B4 S2 | 7.162e-05 | 2000 | 2.029e-12 | 7.231e-09 | nan
INFO  B4 S3 | lam=7.16e-07,t=2.094e+06 | 10 | 2.777e-06 | 9.161e-03 | 5.995e-04
INFO  B4 S3 | lam=7.16e-07,t=2.094e+06 | 50 | 5.413e-09 | 1.805e-05 | 5.690e-06
INFO  B4 S3 | lam=7.16e-07,t=2.094e+06 | 300 | 1.581e-10 | 6.668e-08 | 2.684e-08
INFO  B4 S3 | lam=7.16e-07,t=2.094e+06 | 1000 | 9.452e-14 | 1.747e-09 | nan
INFO  B4 S3 | lam=7.16e-05,t=2.094e+06 | 10 | 2.702e-04 | 6.138e-01 | 4.017e-02
INFO  B4 S3 | lam=7.16e-05,t=2.094e+06 | 50 | 5.286e-07 | 1.209e-03 | 3.812e-04
INFO  B4 S3 | lam=7.16e-05,t=2.094e+06 | 300 | 1.580e-08 | 4.468e-06 | 1.798e-06
INFO  B4 S3 | lam=7.16e-05,t=2.094e+06 | 1000 | 9.452e-12 | 1.170e-07 | nan
PASS  B4 X1 the b4 check case has 152 negative weights in its first 300 modes  152

ALL PASS
```

`sec4_p2_positive_second_order.py` (Section 4.6.1):

```text
PASS  S1a TR-BDF2 stage 2 equals variable-step BDF2 with omega = (1-gamma)/gamma
PASS  S1b TR-BDF2: beta = gamma/2 = 1 - 1/sqrt2 (one matrix I + beta h B in both stages), a - b = 1
    beta = 0.292893, a = 1.207107, b = 0.207107
PASS  S1c TR-BDF2 r = -sqrt2/(1+beta x) + (1+sqrt2)/(1+beta x)^2 (exact partial fractions)
PASS  S1d0 BDF2 started by one BE step: principal-root amplitude alpha = 1 + (3/4) x^2 + O(x^3)
    coefficients [0, 0, 3/4]
PASS  S1d orders: BE 1, CN 2, TR-BDF2 2, Richardson 2, BDF2 2  {'BE': 1, 'CN': 2, 'TR-BDF2': 2,
    'Richardson': 2, 'BDF2': 2}
PASS  S1e mode-error constants c in ln r(x) = -x + c x^(p+1): BE 1/2, CN -1/12, TR-BDF2 2/3 -
    sqrt2/2, Richardson 1/6, BDF2 -1/3  BE: 1/2 = +0.500000; CN: -1/12 = -0.083333; TR-BDF2: 2/3 -
    sqrt(2)/2 = -0.040440; Richardson: 1/6 = +0.166667; BDF2: -1/3 = -0.333333
PASS  S1f CN r(2) = 0, r < 0 for x > 2, r -> -1; TR-BDF2 r(1+sqrt2) = 0, min r = -(sqrt2-1)/2 at x =
    4+3 sqrt2, r -> 0  TR-BDF2 min -0.207107 at x = 8.2426
PASS  S1g Richardson: r = 0 at x = 2 + 2 sqrt2, r < 0 beyond, r -> 0  min r on [0,400] = -0.036117
    at x = 11.759
PASS  S1h CN totally monotone on [0, 2): (-1)^k r^(k) >= 0, k <= 14
PASS  S1i TR-BDF2 totally monotone on [0, 1+sqrt2], k <= 14 (sign of -sqrt2 (1+beta x) +
    (k+1)(1+sqrt2))
PASS  S1j Richardson closed form of (-1)^k r^(k) holds (k <= 6); some k fails at every x (radius 0);
    x = 0 fails first at k = 4 (c_4 = -3/8)  first failing k: {0.0: 4, 0.1: 4, 1.0: 7, 3.0: 13,
    10.0: 0, 100.0: 0}
PASS  S1k BDF2 roots complex for x > 1/2 with |xi|^2 = 1/(3+2x); real and in (0,1] for 0 <= x <= 1/2
PASS  S1l BDF2 scalar x = 1 from y0 = y1 = 1 changes sign  first negative at n = 7
PASS  S2a Taylor weights at a: sum w = r(0), sum w t = -r'(0), sum w t (t - 1/a) = r''(0) (BE, a =
    5)  2.2e-16 6.7e-16 1.3e-15
PASS  S2b Pringsheim bound |r(z)| <= r(a - |z - a|) inside the disc of radius a + 1 (BE, 2000
    points)
PASS  S3a B_delta is STRATO-structured (K symmetric, zero row sums except the last, lam >= 0)
PASS  S3b entry (1,k+1) of r(B_delta) -> (-1)^k r^(k)(x) eps^k/k! as delta -> 0 (delta = 1e-7)  CN
    x=1.0: max rel 8.2e-09, entry(1,5) +1.029e-07; TR-BDF2 x=1.5: max rel 6.0e-09, entry(1,5)
    +5.191e-08; Richardson x=0.3: max rel 1.4e-06, entry(1,5) +5.473e-09
PASS  S3c negative entries in the class: Richardson at x = 0.3, CN at x = 2.5, TR-BDF2 at x = 2.6
    min entries -2.07e-08, -1.11e-01, -2.48e-02
PASS  S3d order-2 r = 1/(1+x+x^2/2): r(B) >= 0 on 3000 random 2 x 2 members; negative entry (1,5)
    for N = 5 (x = 0.1)  entry(1,5) = -8.96e-07
      data: N = 40, Lambda_1 = 7.16597e-07 s^-1, lam = 7.16597e-08 s^-1, max_j B_jj = 5.12376e-03
    s^-1, max_j B_jj / Lambda_1 = 7.1501e+03, mu_max/mu_1 = 1.2515e+04
PASS  S4a diag (I + h/2 B)^-1 from the harmonic-fraction formula equals the solver value  max rel
    2.7e-16
PASS  S4b r_CN(hB) = 2 (I + h/2 B)^-1 - I (exact identity; off-diagonal entries >= 0 for every h)
    max abs diff 6.6e-15
PASS  S4c CN positive exactly for h <= h*_CN; h*_CN >= 2/max B_jj (Bolley-Crouzeix bound)   h*_CN =
    5.86084e+02 s, h*_CN max B_jj = 3.0030, Lambda_1 h*_CN = 4.1999e-04, binding cell j = 24
    (material 1)
PASS  S4d TR-BDF2 positive at h max B_jj = 1 + sqrt2 (sufficient bound of lemma P2.L3)  min entry
    2.888e-55
PASS  S4e TR-BDF2 on the synthetic matrix: positive on every grid step below the first failure,
    which lies beyond the bound  first failure between grid points, bisected: h*_TR max B_jj =
    8.919, Lambda_1 h*_TR = 1.247e-03
PASS  S4f Richardson-BE has negative entries for every h on a 60-point grid (1e-3/max B_jj to
    20/Lambda_1); small-h entry (1,5) ~ -3/8 h^4 prod b  entry(1,5) -4.354e-12, predicted -4.442e-12
PASS  S4i global Richardson (whole runs) at mu_1 h = 0.5: entries not all negative at n = 10, all
    negative at n = 40 (crossing n = ln2/ln(a^2/b))  crossing n = 17.0
PASS  S4g BDF2: C^n = 0, C^{n-1} = e_j >= 0 gives C^{n+1} = -(1/3)(I + 2h/3 B)^-1 e_j < 0 for every
    h  max -5.75e-19
PASS  S4h exp(-hB) >= 0 exactly in floating point by uniformisation and squaring; agrees with the
    eigen route  m=10: eigen min -2.5e-16, rel 7.0e-14, M rel 8.0e-15; m=16: eigen min +2.4e-07, rel
    5.0e-12, M rel 5.1e-12; m=20: eigen min +1.5e-13, rel 1.4e-11, M rel 1.4e-11
PASS  S5a reference M(t_end) from the double eigen route equals the 30-digit mpmath value
    M(t_end)/M0 = 2.427556e-07, rel diff 5.1e-12
      BE         rel err M(t_end): +6.21e+00 +1.93e+00 +7.58e-01 +3.35e-01 +1.58e-01 +7.64e-02
    +3.76e-02 | order(last pair) 1.022 | min C over run +0.0e+00 +0.0e+00 +0.0e+00 +0.0e+00 +0.0e+00
    +0.0e+00 +0.0e+00
      CN         rel err M(t_end): -2.32e+01 -3.22e-02 -7.59e-03 -1.90e-03 -4.75e-04 -1.19e-04
    -2.97e-05 | order(last pair) 2.000 | min C over run -5.8e-01 -4.8e-01 -4.2e-01 -3.4e-01 -2.2e-01
    -5.5e-02 +0.0e+00
      TR-BDF2    rel err M(t_end): -5.94e-02 -1.49e-02 -3.71e-03 -9.26e-04 -2.31e-04 -5.77e-05
    -1.44e-05 | order(last pair) 2.001 | min C over run +0.0e+00 +0.0e+00 +0.0e+00 +0.0e+00 +0.0e+00
    +0.0e+00 +0.0e+00
      Richardson rel err M(t_end): +1.75e-01 +5.02e-02 +1.37e-02 +3.60e-03 +9.25e-04 +2.34e-04
    +5.90e-05 | order(last pair) 1.990 | min C over run +0.0e+00 +0.0e+00 +0.0e+00 +0.0e+00 -3.5e-05
    -7.6e-05 -8.5e-05
      BDF2       rel err M(t_end): -4.54e-01 -1.13e-01 -2.74e-02 -6.68e-03 -1.65e-03 -4.09e-04
    -1.02e-04 | order(last pair) 2.005 | min C over run +0.0e+00 +0.0e+00 +0.0e+00 +0.0e+00 +0.0e+00
    +0.0e+00 +0.0e+00
      ETD        rel err M(t_end): -2.00e-14 -3.57e-14 -1.02e-13 -1.72e-13 -3.23e-13 -6.06e-13
    -1.45e-12 | order(last pair) -1.259 | min C over run +0.0e+00 +0.0e+00 +0.0e+00 +0.0e+00
    +0.0e+00 +0.0e+00 +0.0e+00
PASS  S5b observed orders (last pair): BE 1, CN, TR-BDF2, Richardson, BDF2 2 (within 0.05)  BE
    1.022, CN 2.000, TR-BDF2 2.001, Richardson 1.990, BDF2 2.005
PASS  S5c leading-order prediction c mu_1^(p+1) t h^p (BDF2 plus (3/4)(mu_1 h)^2) of the relative
    error at n = 3200 (within 5 %, sign included)  CN obs/pred 1.0000, TR-BDF2 obs/pred 1.0004,
    Richardson obs/pred 0.9929, BDF2 obs/pred 1.0037, BE obs/pred 1.0154
PASS  S5d ETD (exact exponential) error at round-off for every n  max 1.4e-12
PASS  S5e negative cell values in these runs: BE and ETD none (BE >= 0.0 exactly); CN for n <= 1600
    and M(t_end) < 0 at n = 50; Richardson for n >= 800; TR-BDF2 and BDF2 none  BE: n = []; CN: n =
    [50, 100, 200, 400, 800, 1600]; TR-BDF2: n = []; Richardson: n = [800, 1600, 3200]; BDF2: n =
    []; ETD: n = []
      h max B_jj =        5, Lambda_1 h = 6.99e-04: TR-BDF2 min entry +8.50e-45, min r(hB)C0:
    TR-BDF2 +4.92e-21, Richardson -7.39e-05, CN +6.31e-19
      h max B_jj =      100, Lambda_1 h = 1.40e-02: TR-BDF2 min entry -1.73e-01, min r(hB)C0:
    TR-BDF2 +5.20e-07, Richardson -4.44e-05, CN -1.70e-01
      h max B_jj =     1000, Lambda_1 h = 1.40e-01: TR-BDF2 min entry -1.00e-01, min r(hB)C0:
    TR-BDF2 +1.64e-03, Richardson +1.81e-03, CN -4.82e-01
      h max B_jj =     3000, Lambda_1 h = 4.20e-01: TR-BDF2 min entry -8.04e-02, min r(hB)C0:
    TR-BDF2 +3.69e-03, Richardson +2.92e-03, CN -5.64e-01
      h max B_jj =    10000, Lambda_1 h = 1.40e+00: TR-BDF2 min entry -3.93e-02, min r(hB)C0:
    TR-BDF2 -1.31e-01, Richardson +1.49e-03, CN -6.97e-01
      h max B_jj =   100000, Lambda_1 h = 1.40e+01: TR-BDF2 min entry -1.31e-02, min r(hB)C0:
    TR-BDF2 -9.57e-02, Richardson -1.91e-02, CN -9.38e-01
PASS  S5f kernel-loaded start: TR-BDF2 image r(hB)C0 >= 0 up to Lambda_1 h = 0.42 although r(hB) has
    negative entries; negative at Lambda_1 h = 1.4
      steps for 1e-3 at Lambda_1 t = 14 (leading order): BE 118580, CN 552, TR-BDF2 384, Richardson
    780, BDF2 1020
      steps for guaranteed positivity: CN (exact h*) 33334, CN (2/max B_jj) 50051, TR-BDF2
    ((1+sqrt2)/max B_jj) 41464, TR-BDF2 (h*_TR of this matrix) 11224
      CRP-6 1b arithmetic, step for 1e-3 in 1 - F (leading order): BE 7.1 s, CN 1475.6 s, TR-BDF2
    2118.3 s, Richardson 1043.4 s
PASS  S5g homogeneous sphere, uniform mesh: Lambda_1 near pi^2 D/R^2 and max B_jj/Lambda_1 grows
    like N^2  N = 40: max B_jj/Lambda_1 = 5.6732e+02, CN cap Lambda_1 h <= 3.525e-03, N = 150: max
    B_jj/Lambda_1 = 7.9790e+03, CN cap Lambda_1 h <= 2.507e-04
PASS  S6a closure with decay and switching source, scheme-consistent quadrature: CN  max per-step
    residual / (M0 + M_gen) 1.0e-16, cumulative 1.2e-16, min C over run -9.9e-04 mol m^-3
PASS  S6a closure with decay and switching source, scheme-consistent quadrature: TR-BDF2  max
    per-step residual / (M0 + M_gen) 1.1e-16, cumulative 1.2e-16, min C over run -1.1e-05 mol m^-3
PASS  S6a closure with decay and switching source, scheme-consistent quadrature: Richardson  max
    per-step residual / (M0 + M_gen) 2.2e-16, cumulative 1.2e-16, min C over run -7.4e-05 mol m^-3
PASS  S6a closure with decay and switching source, scheme-consistent quadrature: BDF2  max per-step
    residual / (M0 + M_gen) 8.6e-17, cumulative 1.4e-15, min C over run -4.0e-04 mol m^-3
PASS  S6a closure with decay and switching source, scheme-consistent quadrature: ETD  max per-step
    residual / (M0 + M_gen) 5.5e-14, cumulative 3.0e-13, min C over run -4.7e-17 mol m^-3
PASS  S6b semi-discrete steady state B^-1 s is a fixed point of every scheme (h = 10/Lambda_1)  CN
    2.2e-13, TR-BDF2 1.7e-13, Richardson 1.4e-13, BDF2 1.2e-13, ETD 3.1e-13
ALL PASS
```

`sec5_o12_variable_D_step_limit.py` (Section 5.2.5):

```text
PASS  S1 g(z)=z-ln(1+z): (z^2/2-g)'=z^2/(1+z), (g-z^2/2+z^3/3)'=z^3/(1+z), both 0 at z=0
PASS  S2 constant D: (1/2) sum Z_k^2 = (1/2)(Lam+lam)^2 t dt (P21.78); rule Z<=2eps/Theta gives F3
    dt
PASS  S3 sum Z_k^2 >= Theta^2/N (Cauchy-Schwarz), equality at equal Z_k
PASS  S4 equal steps: n[(1+x/n)^-n - e^-x] <= c_n, c_3 = 2/3, c_n decreasing to 2e^-2 (n<=1e6)
    c_3=0.666667, c_1e6=0.270671, 2e^-2=0.270671
PASS  S5 constant D with decay: F_h - F = -p (a_h - a), p = Lam/(Lam+lam) (geometric series, N = 5
    spot check)
INFO  CRP-6 1b reduction (constant T): (1/2) Lambda_1 (Lambda_1 t) dt = 0.0505 at 360 s, 0.00505 at
    36 s (Section 4.6 values)
PASS  N0a Lambda_1(sD) = s Lambda_1(D); Lambda_1 non-decreasing in each D_i; ratio within [min,max]
    homogeneity residual 1.0e-10 (eigensolver accuracy)
PASS  N0b d ln Lambda_1/dT lies in [min_i, max_i] Q_i/(Rg T^2) (layer-specific Q)
PASS  N1a sum w_m = 1 and sum |w_m| <= sqrt(V_tot/V_K) (Cauchy-Schwarz)  sum|w_m| = 2.1798, bound
    3.3808, w_1 = 1.1453
INFO  Lambda_1(1600 C) = 1.93320e-05 1/s, Lambda_1(1250 C) = 2.3119e-07 1/s, Theta(t_end) = 6.5945,
    Lambda_1,max t_end = 6.9595
PASS  N1b eps=0.01: nodal BE 1-F = sum_m w_m prod(1+Lambda_m dtau_k)^-1; sum dtau = tau (P22.22)
    rel 2.9e-10, steps 2181
PASS  N1c eps=0.01: rule Z_k <= zeta holds, delta_1 <= zeta Theta/2 = eps  max Z/zeta 1.000000,
    delta_1 9.9668e-03, (1/2)sum Z^2 9.9870e-03
PASS  N1d eps=0.01: relative error of 1-F at t_end = e^delta_1 - 1 (late time) and <= e^eps - 1
    observed 1.00167e-02, predicted 1.00167e-02, steps 2181, Cauchy-Schwarz minimum Theta^2/sum Z^2
    = 2177.2
PASS  N1e eps=0.01: F_h - F = -sum w_m a_m^h (1 - e^-delta_m) at all steps; |err| <= bound  max|err|
    1.07e-03 (first step 8.88e-16), max|err| after 20 h 6.33e-04
PASS  N1b eps=0.001: nodal BE 1-F = sum_m w_m prod(1+Lambda_m dtau_k)^-1; sum dtau = tau (P22.22)
    rel 3.1e-10, steps 21747
PASS  N1c eps=0.001: rule Z_k <= zeta holds, delta_1 <= zeta Theta/2 = eps  max Z/zeta 1.000000,
    delta_1 9.9968e-04, (1/2)sum Z^2 9.9988e-04
PASS  N1d eps=0.001: relative error of 1-F at t_end = e^delta_1 - 1 (late time) and <= e^eps - 1
    observed 1.00018e-03, predicted 1.00018e-03, steps 21747, Cauchy-Schwarz minimum Theta^2/sum Z^2
    = 21746.2
PASS  N1e eps=0.001: F_h - F = -sum w_m a_m^h (1 - e^-delta_m) at all steps; |err| <= bound
    max|err| 1.17e-04 (first step 6.66e-16), max|err| after 20 h 6.38e-05
INFO  eps=1e-3: n=21747, W c_n / n = 2.71e-05 (equal-Z bound, holds only for equal Z_k), observed
    |err| at t_end 1.57e-06
PASS  N1f F3 warning value (Lambda_1 at T_max) is a sufficient cap: error <= e^eps - 1  observed
    9.3950e-04, steps 24219 vs rule 21747
PASS  N1g Lambda_1 at T_min (not a bound on Lambda_1(t)) gives an error above eps  observed
    1.5553e+01, steps 5
PASS  N2a option E, ramp+hold: modal identity; |lag| <= sum dt_k |dLambda_1,k| <= max dt
    TV(Lambda_1)  lag -4.784e-03, sum dt|dL| 9.343e-03, (1/2) sum dt|dL| 4.672e-03
PASS  N2b option E, ramp+hold: relative error of 1-F = e^delta_E - 1  observed 5.20876e-03,
    predicted 5.20876e-03, sum g(z) 9.9792e-03
PASS  N2d option E, ramp+hold: |lag| <= zeta TV(ln Lambda_1) <= zeta (Q/Rg) TV(1/T); max z_k <= zeta
    |lag| 4.784e-03, zeta TV(ln L1) 1.342e-02 (leading order half: 6.712e-03), TV(ln L1) 4.4263,
    (Q/Rg)TV(1/T) 4.4263
PASS  N2c heating: the lag term is negative (option E partly cancels the BE lag)
PASS  N2a option E, ramp+hold+cool+hold: modal identity; |lag| <= sum dt_k |dLambda_1,k| <= max dt
    TV(Lambda_1)  lag -1.989e-03, sum dt|dL| 1.590e-02, (1/2) sum dt|dL| 7.951e-03
PASS  N2b option E, ramp+hold+cool+hold: relative error of 1-F = e^delta_E - 1  observed
    8.01249e-03, predicted 8.01249e-03, sum g(z) 9.9699e-03
PASS  N2d option E, ramp+hold+cool+hold: |lag| <= zeta TV(ln Lambda_1) <= zeta (Q/Rg) TV(1/T); max
    z_k <= zeta  |lag| 1.989e-03, zeta TV(ln L1) 2.678e-02 (leading order half: 1.339e-02), TV(ln
    L1) 8.8526, (Q/Rg)TV(1/T) 8.8526
INFO  with cool-down the lag term is -1.989e-03 (heating part cancels the cooling part)
PASS  N3a Radau reference reproduces the exact shared-scaling 1-F (validates the reference)  rel
    2.9e-10
INFO  layer-specific Q: Lambda_1(1250 C) = 1.3501e-08, Lambda_1(1600 C) = 1.93320e-05, Theta =
    6.5554, 1-F_ref = 1.572490e-03 (rtol 1e-10 vs 1e-11: 1.5e-14)
INFO  N3 option A, eps=0.01, eps_D=None: observed 1.05273e-02, frozen-coefficient prediction
    1.00116e-02, ratio 1.0515, remainder +5.157e-04, steps 2208 (86 on the ramp)
INFO  N3 option E, eps=0.01, eps_D=None: observed 6.40566e-03, frozen-coefficient prediction
    5.70738e-03, ratio 1.1223, remainder +6.983e-04, steps 2208 (86 on the ramp)
PASS  N3c eps=0.01: option A step eigenvalue >= step mean of Lambda_1 (concavity), gap is tiny  max
    (z_A - Z)/Z = 2.32e-04
INFO  N3 option A, eps=0.01, eps_D=0.1: observed 1.02115e-02, frozen-coefficient prediction
    1.00006e-02, ratio 1.0211, remainder +2.109e-04, steps 2228 (106 on the ramp)
INFO  N3 option E, eps=0.01, eps_D=0.1: observed 7.51145e-03, frozen-coefficient prediction
    7.22618e-03, ratio 1.0395, remainder +2.853e-04, steps 2228 (106 on the ramp)
INFO  N3 option A, eps=0.01, eps_D=0.025: observed 9.99677e-03, frozen-coefficient prediction
    9.94233e-03, ratio 1.0055, remainder +5.444e-05, steps 2426 (304 on the ramp)
INFO  N3 option E, eps=0.01, eps_D=0.025: observed 9.08001e-03, frozen-coefficient prediction
    9.01157e-03, ratio 1.0076, remainder +6.843e-05, steps 2426 (304 on the ramp)
INFO  N3 option A, eps=0.003, eps_D=None: observed 3.34061e-03, frozen-coefficient prediction
    3.00037e-03, ratio 1.1134, remainder +3.402e-04, steps 7254 (182 on the ramp)
INFO  N3 option E, eps=0.003, eps_D=None: observed 1.53585e-03, frozen-coefficient prediction
    1.20690e-03, ratio 1.2726, remainder +3.289e-04, steps 7254 (182 on the ramp)
PASS  N3c eps=0.003: option A step eigenvalue >= step mean of Lambda_1 (concavity), gap is tiny  max
    (z_A - Z)/Z = 1.23e-04
INFO  N3 option A, eps=0.003, eps_D=0.1: observed 3.17627e-03, frozen-coefficient prediction
    2.99957e-03, ratio 1.0589, remainder +1.767e-04, steps 7273 (201 on the ramp)
INFO  N3 option E, eps=0.003, eps_D=0.1: observed 1.86208e-03, frozen-coefficient prediction
    1.65647e-03, ratio 1.1241, remainder +2.056e-04, steps 7273 (201 on the ramp)
INFO  N3 option A, eps=0.003, eps_D=0.025: observed 3.04816e-03, frozen-coefficient prediction
    2.99383e-03, ratio 1.0181, remainder +5.433e-05, steps 7416 (344 on the ramp)
INFO  N3 option E, eps=0.003, eps_D=0.025: observed 2.30435e-03, frozen-coefficient prediction
    2.23432e-03, ratio 1.0313, remainder +7.003e-05, steps 7416 (344 on the ramp)
PASS  N3d remainder R_na = observed - frozen prediction is positive and first order in eps_D (ratio
    for eps_D 0.1 : 0.025 in [2.5, 5.5], first order gives 4)  ratios 3.87, 4.17, 3.25, 2.94
PASS  N3e at eps_D = 0.025 R_na does not depend on the Z rule (eps 1e-2 vs 3e-3 within 10 %)  ratios
    1.002, 0.977
PASS  N3f with eps_D = 0.025 the frozen-coefficient prediction is within 3.5 % (options A and E)
    ratios 1.0055, 1.0076, 1.0181, 1.0313
PASS  N4a decay, eps=0.01: nodal M and F equal the modal sums (exact modal form)  steps 3967
PASS  N4b decay, eps=0.01: relative error of retained M at t_end = e^delta_1 - 1, <= e^eps - 1
    observed 1.00325e-02, predicted 1.00325e-02
PASS  N4c decay, eps=0.01: F_h - F = -sum w_m (e_a + amp_D + rect); each part within its bound; |F_h
    - F| <= B  F = 0.636306, F_h - F = 5.8884e-05: amplitude -1.559e-06, decay-amplitude -2.915e-04,
    rectangle 3.519e-04; B = 4.897e-03, zeta W = 4.896e-03
INFO  eps=0.01: constant-coefficient relation -p_1 w_1 a_1 (e^delta_1 - 1) = -1.171e-06 against F_h
    - F = 5.888e-05 (ratio -50.3); the constant-D relation does not carry over
PASS  N4a decay, eps=0.001: nodal M and F equal the modal sums (exact modal form)  steps 39651
PASS  N4b decay, eps=0.001: relative error of retained M at t_end = e^delta_1 - 1, <= e^eps - 1
    observed 1.00032e-03, predicted 1.00032e-03
PASS  N4c decay, eps=0.001: F_h - F = -sum w_m (e_a + amp_D + rect); each part within its bound;
    |F_h - F| <= B  F = 0.636306, F_h - F = 5.9088e-06: amplitude -1.555e-07, decay-amplitude
    -2.918e-05, rectangle 3.525e-05; B = 4.897e-04, zeta W = 4.896e-04
INFO  eps=0.001: constant-coefficient relation -p_1 w_1 a_1 (e^delta_1 - 1) = -1.167e-07 against F_h
    - F = 5.909e-06 (ratio -50.6); the constant-D relation does not carry over
PASS  N4d decay: release-fraction error is first order in zeta (ratio for eps 1e-2 : 1e-3 near 10)
    ratio 9.966
ALL PASS
```

`sec5_r24_quadrature_start.py` (Section 5.3.7):

```text
PASS  C1 (P22.35) with Mhat_rel = M0 - Mhat equals M0 - M - M_dec (SymPy)
PASS  C1 balance integrand has the same sqrt start: (M0 e^{-lam s} - M)/(M0 sqrt s) -> 6/sqrt(pi)
    values 3.382134 3.384837 3.385108, 6/sqrt(pi) = 3.385138
PASS  C2 image form equals the modal series for u in [0.002, 1]  max abs diff 2.7e-37
PASS  C2 0 <= F - (6 sqrt(u/pi) - 3u) <= pi^{3/2} u^{3/2} e^{-1/u}  max ratio remainder/bound 0.603
PASS  C2 remainder bound at u_B = 1/40  9.35e-20
PASS  C3 <f,(-L)^{-1}f> = int Q^2/(r^2 D) dr, two layers, f = 1  value 25/288
PASS  C3 <f,(-L)^{-1}f> = int Q^2/(r^2 D) dr, two layers, f = c0 = kernel indicator  value 11/1440
PASS  C3 sphere tails W_N <= 6/(pi^2 N), V_N <= 2/(pi^4 N^3) (R = D* = 1), N = 10, 100, 1000
PASS  C3 sphere sum w_n/Lambda_n = R^2/(15 D*)
PASS  C4 sphere start piece: truncation error <= lam min(t0 W_N, V_N/alpha) in 12 cases
      alpha=1    N=10    t0=3.349e-02  error 1.765e-05  bound 1.766e-05  ratio 0.999  B7 estimate
    1.938e-03
      alpha=1    N=100   t0=3.973e-04  error 2.023e-08  bound 2.023e-08  ratio 1.000  B7 estimate
    2.403e-06
      alpha=1    N=1000  t0=4.045e-06  error 2.050e-11  bound 2.050e-11  ratio 1.000  B7 estimate
    2.458e-09
      alpha=0.1  N=10    t0=3.349e-01  error 1.756e-04  bound 1.766e-04  ratio 0.995  B7 estimate
    1.938e-02
      alpha=0.1  N=100   t0=3.973e-03  error 2.022e-07  bound 2.023e-07  ratio 1.000  B7 estimate
    2.403e-05
      alpha=0.1  N=1000  t0=4.045e-05  error 2.050e-10  bound 2.050e-10  ratio 1.000  B7 estimate
    2.458e-08
PASS  C4 at t0 = 40/(alpha Lambda_{N+1}) the new bound is at least 100 times below lam t0 W_N
    smallest ratio 109.8
PASS  C5 reference: closed form equals (P22.35) evaluated in mpmath
PASS  C5 plain composite 8-point GL from s = 0 converges slowly (order about 1.5 in pieces)  errors
    m=1..32: 5.8e-04 2.0e-04 7.2e-05 2.5e-05 8.9e-06 3.2e-06; order 1.50
PASS  C5 proposed route (modal start piece, ratio-2 graded GL) error <= lam V_N + e^{-40} W_N +
    1e-13  N = 200, 14 graded pieces, error 2.547e-09, bound 2.547e-09, lam V_N = 2.547e-09
      q = 2: 14 pieces, summed GL error 3.77e-15
      q = 8: 5 pieces, summed GL error 2.06e-08
PASS  C5 each ratio-2 piece obeys the Bernstein-ellipse bound (b-a)/2 * 8 M rho^(1-2n)/(rho-1), n =
    8  largest error/bound 2.20e-03; summed bound over [t0, t] 5.44e-12
PASS  C6 route B (Booth start, 12 modes) error at the rounding level  error 0.00e+00, analytic part
    of the bound 1.3e-19, start GL8 vs GL16 3.5e-18, 5 graded pieces
PASS  C6 route M (modal start, alpha bracket, N = 200) error <= half-width + 2 lam min(t0 W,
    V/alpha_-) + e^-40 W + 1e-13  error 2.659e-09, bound 5.374e-09 (half-width 2.8e-10)
PASS  C7 2000 roots found, sum w_n -> 1 and the weights change sign  1 - sum w = 2.74e-07, sign
    changes 1333
PASS  C7 modal sum w_n/Lambda_n equals (4 pi/M0) int Q_c0 Q_1/(r^2 D) dr  diff 1.6e-14
PASS  C7 Bessel and energy tails are non-negative and shrink (no missed root)  H_N(1) at N = 100,
    1000, 2000: 1.14e-08 1.15e-11 1.44e-12
PASS  C7 sum |w_n| <= (R/R1)^{3/2} (Cauchy-Schwarz)  2.729 <= 2.828
PASS  C7 start-piece truncation (plus the bounded remainder beyond 2000 modes) <= lam min(t0 W_bar,
    V_bar)  largest ratio 0.108
      N=20: error 1.46e-06 bound 1.36e-05 (V_bar 1.36e-05, sum|w|/L over 2000 modes 1.33e-05)
      N=50: error 3.91e-08 bound 8.91e-07 (V_bar 8.91e-07, sum|w|/L over 2000 modes 8.73e-07)
      N=100: error 6.94e-11 bound 1.11e-07 (V_bar 1.11e-07, sum|w|/L over 2000 modes 1.09e-07)
      N=200: error 1.55e-10 bound 1.41e-08 (V_bar 1.41e-08, sum|w|/L over 2000 modes 1.38e-08)
PASS  C7 two-layer end-to-end release (N = 100) within the bound  error 6.945e-11, bound 1.115e-07
PASS  C8 cold piece: standard-form error equals lam sum_{n>N} w_n E_n and lies in [0, (1 - e^{-lam
    t_c}) W_N]  error 3.6970e-03, lam sum_{n>N} w_n E_n 3.6970e-03, (1-e^{-1}) W_N = 3.824e-03
PASS  C8 cold piece: direct-form error <= true start-piece release <= (1 - e^{-lam t_c}) 6 sqrt(a
    t_c/pi); Booth exact  direct error 1.27e-04, true 1.28e-04, Booth closed form error 0.0e+00
SUMMARY: 25/25 PASS
ALL PASS
```

`sec5_o11_parent_daughter_tau.py` (Section 5.3.8):

```text
PASS  S1 transformed coefficients alpha_k/tau', (lambda_k + g_k'/g_k)/tau', b lambda_P g_P/(g_D
    tau') (P2X.O11.5)
PASS  S2 commutator [W, diag(k1,k2)] = (k2-k1)[[0, w12], [-w21, 0]] (step 2 of the proof)  [[0,
    w12*(-k1 + k2)], [w21*(k1 - k2), 0]]
PASS  S3 N2 <=> alpha' = delta alpha^2 - dl alpha; Bernoulli solution (P2X.O11.9)
PASS  S4 ramp: third x-derivative is theta^3 d' exp(-theta x), so d' = 0, then theta r = dl = 0
    (P2X.O11.11)  d3/dx3 = dprime*theta**3*exp(-theta*x)
PASS  S5 identical operators: c = exp(M t) chat(tau(t)) is exact for any alpha (P2X.O11.12)
PASS  S6 two-clock Duhamel formula solves the daughter modal ODE (P2X.O11.14)  residual 0
PASS  S7 d/dt ln(D_D/D_P) = (Q_D - Q_P) T'/(R_g T^2) (P2X.O11.8)
PASS  S8 (D - Dt)^2/D = Dt (1 - rho)^2/rho (P2X.O11.17)
PASS  S9 integrals of the a priori daughter bound (P2X.O11.20)
PASS  S10 d/dT [T E2(theta/T)] = exp(-theta/T)  max rel err 2.5e-40
PASS  N1 ramp integral of D, closed form (E2) against adaptive quadrature  max rel diff 3.8e-15
PASS  N2 N1 and N2 hold: joint constant-coefficient form in tau = stiff modal integration
    (P2X.O11.10)  max rel diff 1.8e-13 (modes 1,2,3,5); tau closed form vs quad 4.4e-16
PASS  N3 identical operators on a ramp: exp(M t) form = stiff modal integration (P2X.O11.12)  max
    rel diff 1.5e-13
PASS  N4 N2 residual: Bernoulli history fits ln C + delta tau exactly, ramp + hold does not
    Bernoulli residual 1.8e-15, fitted delta/delta = 1.000000000000; ramp residual 7.35 (in ln
    alpha)
PASS  N5 counterexample Q_P = 300, Q_D = 400 kJ/mol: implied clock ratio Theta_P/Theta_D is not
    constant  Theta_P/Theta_D at t = 20000, 43200, 100000, 300000 s: 16.5041, 4.3468, 3.4136,
    3.3522; implied vs closed form 1.7e-10
PASS  N6 two-clock cascade = stiff modal integration, Q_P != Q_D on ramp + hold (P2X.O11.14)  max
    rel diff 1.6e-13 (modes 1,2,3,5,8)
PASS  N7 equal Q but N2 fails: a frozen tau-coupling gives a visible error (illustration)  mode-1
    daughter rel error 3.82e-01 at 1e5 s, 6.79e-02 at 3e5 s
      FV  40 cells/layer: parent rel diff 2.04e-04, daughter rel diff 7.58e-04
      FV  80 cells/layer: parent rel diff 5.09e-05, daughter rel diff 1.90e-04
      FV 160 cells/layer: parent rel diff 1.27e-05, daughter rel diff 4.74e-05
PASS  N8 two-layer cascade with overlap matrix O = two-species FV, Q_P = 200, Q_D = 350 kJ/mol
    (P2X.O11.15)  finest daughter rel diff 4.7e-05, parent 1.3e-05, observed order 2.00; M_D/M_P0 at
    2e8 s = 0.00001
PASS  N8b the overlap matrix is full (the daughter needs all parent modes)  max off-diagonal 4.214
PASS  N9a untruncated mode sum (Poisson transform) = long direct sum  max rel diff 2.2e-16
PASS  N9 single sphere: L2 errors of a shared-Qbar surrogate obey the a priori and a posteriori
    bounds (P2X.O11.18-20)  m_P = 0.2097, m_D = 0.2097
      t=20000: E_P/|c0| 1.35e-02 <= post 3.14e-02 <= prior 2.24e-01; E_D/|c0| 4.09e-05 <= prior
    9.07e-03
      t=43200: E_P/|c0| 7.38e-03 <= post 4.03e-02 <= prior 2.19e-01; E_D/|c0| 4.32e-05 <= prior
    1.94e-02
      t=100000: E_P/|c0| 1.15e-03 <= post 4.03e-02 <= prior 2.07e-01; E_D/|c0| 9.41e-05 <= prior
    4.36e-02
      t=300000: E_P/|c0| 3.98e-05 <= post 4.03e-02 <= prior 1.70e-01; E_D/|c0| 4.15e-05 <= prior
    1.19e-01
PASS  N10 two-layer FV, layer-specific Q: discrete a priori bounds hold for the shared-Q surrogate
    (P2X.O11.21)  m_P = 0.0343, m_D = 0.0612
      t=5e+07: E_P/|C0| 4.78e-03 <= 2.07e-02; E_D/|C0| 1.28e-02 <= 1.68e-01; |dM_D|/M_P0 2.04e-02
      t=1e+08: E_P/|C0| 4.74e-04 <= 4.61e-03; E_D/|C0| 2.59e-04 <= 2.06e-01; |dM_D|/M_P0 6.14e-04
      t=2e+08: E_P/|C0| 8.07e-07 <= 2.30e-04; E_D/|C0| 1.48e-07 <= 2.16e-01; |dM_D|/M_P0 3.52e-07
PASS  N11 a priori factor (1/2)|1-rho|/sqrt(rho) < 0.1 and < 1: ratio intervals  < 0.1 for rho in
    (0.8190, 1.2210); < 1 for rho in (0.1716, 5.8284)
PASS  N12 shared-scaling surrogate, layer ratio spread q = 6.3: a priori factor >= (1/2)
    sqrt(h(q^(1/4)))  factor >= 0.2321 (scan minimum 0.2322)

SUMMARY: 24/24 PASS
ALL PASS
```

`sec5_o10_stiff_staircase.py` (Section 5.4.6):

```text
PASS  S0.1 harmonic face: f - 1 - ubar = -theta(1-theta)(u_- - u_+)^2/(1+uhat) exactly
PASS  S0.2 d/dt ln G_f = theta x' + (1-theta) y'
PASS  S0.3 d2/dt2 ln G_f = theta x'' + (1-theta) y'' - theta(1-theta)(x'-y')^2
PASS  S0.4 left-rule sharpness: argmax X* = ln(beta)/(1-1/beta), max = beta^(-1/(beta-1)) (1 -
    1/beta) (mpmath, 4 betas)
PASS  S0.5 sup_mu error / (Lh) -> 1/(2e) (left), / (Lh)^2 -> 1/(24e) (midpoint)  limits exp(-1)/2,
    exp(-1)/24
PASS  S0.6 int_h^{T-h} ds/(s(T-s)) = (2/T) ln((T-h)/h); int_0^T ds/sqrt(s(T-s)) = pi
PASS  S1.1 face lemma: spec(K(D)-K(D'), K(D')) in [min r - 1, max r - 1] (20 random pairs)
PASS  S1.2 mixed bound |x^T (K(D)-K(D')) y| <= sqrt(eps eps') |x|_{K(D')} |y|_{K(D)} (100 cases)
PASS  S2 duality: phi^T e(T) = -int_0^T w^T Delta z ds (relative difference < 1e-8)  lhs
    -4.8671629369e-02, -integral -4.8671629415e-02
INFO  L (layer-specific Q) = 2.4471e-04 1/s; L*T = 3.524
PASS  S8.0 reference: Radau rtol 1e-13 vs 1e-11 (V-norm difference)  7.2e-16
INFO  20 cells/layer: ||A||_V = 2.024e+01 1/s; h||A|| = 4553, 2277, 1138, 569, 285, 142, 71
INFO  20 left ||e||_V: 2.34e-03, 1.16e-03, 5.81e-04, 2.90e-04, 1.45e-04, 7.26e-05, 3.63e-05
        orders V-norm: 1.004, 1.002, 1.001, 1.001, 1.000, 1.000
        orders F(4 h): 1.002, 1.001, 1.001, 1.000, 1.000, 1.000
INFO  20 mid  ||e||_V: 3.17e-05, 9.50e-06, 2.77e-06, 8.13e-07, 2.37e-07, 6.77e-08, 1.88e-08
        orders V-norm: 1.736, 1.779, 1.769, 1.779, 1.806, 1.846
        orders F(4 h): 1.805, 2.112, 2.125, 2.073, 2.040, 2.023
INFO  20 avg  ||e||_V: 3.10e-05, 9.35e-06, 2.74e-06, 8.06e-07, 2.35e-07, 6.74e-08, 1.88e-08
        orders V-norm: 1.727, 1.773, 1.764, 1.776, 1.804, 1.845
        orders F(4 h): 2.038, 1.977, 1.977, 1.988, 1.994, 1.996
INFO  40 cells/layer: ||A||_V = 8.131e+01 1/s; h||A|| = 4574, 2287, 1143, 572, 286
INFO  40 left ||e||_V: 5.81e-04, 2.90e-04, 1.45e-04, 7.26e-05, 3.63e-05
        orders V-norm: 1.001, 1.001, 1.000, 1.000
        orders F(4 h): 1.001, 1.000, 1.000, 1.000
INFO  40 mid  ||e||_V: 2.79e-06, 8.25e-07, 2.44e-07, 7.16e-08, 2.09e-08
        orders V-norm: 1.758, 1.759, 1.767, 1.780
        orders F(4 h): 2.113, 2.087, 2.063, 2.048
INFO  40 avg  ||e||_V: 2.76e-06, 8.18e-07, 2.42e-07, 7.13e-08, 2.08e-08
        orders V-norm: 1.753, 1.755, 1.764, 1.778
        orders F(4 h): 1.978, 1.985, 1.990, 1.992
PASS  S3 Theorem A: ||e(T)||_V <= (1/2) eta sqrt(||z0||^2-||z(T)||^2), all rules, both meshes, all K
    max err/bound 0.086, min 0.000
INFO  window orders h||A|| in [1e2, 1.2e3], left: V-norm 1.001, 1.001, 1.000, 1.000, 1.000 | F(4 h)
    1.001, 1.000, 1.000, 1.000, 1.000
INFO  window orders h||A|| in [1e2, 1.2e3], mid: V-norm 1.769, 1.779, 1.806, 1.767, 1.780 | F(4 h)
    2.125, 2.073, 2.040, 2.063, 2.048
INFO  window orders h||A|| in [1e2, 1.2e3], avg: V-norm 1.764, 1.776, 1.804, 1.764, 1.778 | F(4 h)
    1.977, 1.988, 1.994, 1.990, 1.992
PASS  S8.1 left rule: V-norm order 1 in the stiff window (|p-1| < 0.02)
PASS  S8.2 midpoint and average: V-norm orders in the stiff window lie in [1.70, 1.85], below 2
PASS  S8.3 midpoint and average: release-fraction F(4 h) orders in the window lie in [1.7, 2.3]
INFO  80 cells/layer, midpoint, h||A|| = 9158, 4579, 2289, 1145; Cauchy differences 5.90e-07,
    1.75e-07, 5.19e-08; orders 1.752, 1.755
PASS  S8.4 80 cells/layer midpoint Cauchy orders lie in [1.70, 1.80] (trend to 7/4, CLAIMED)
PASS  S4.1 Theorem C: midpoint error <= |tau-tau~|/(e min tau) <= h^2 sup|alpha''|/(24 e inf alpha);
    average exact  midpoint errors 2.11e-06, 5.27e-07, 1.32e-07, 3.29e-08, 8.23e-09, 2.06e-09;
    average max error 1.3e-12 (round-off)
PASS  S4.2 shared scaling, midpoint V-norm order 2 (|p-2| < 0.02) at h||A|| up to 4.5e3  orders
    2.000, 2.000, 2.000, 2.000, 1.999
PASS  S5.1 jump on the grid: staircase exact for every rule (< 1e-13)
PASS  S5.2 misaligned jump: ||e(T)||_V <= sqrt(eps eps') rho h / (2 sqrt(t_k* (T - t_k*+1)))  rho =
    24.140; err/bound in [3.90e-06, 8.57e-04]
PASS  S6 smoothing: ||z(t)||^2_{K(t)} <= e^{Lt} ||z0||^2 / (2t) at 60 times
INFO  Lambda_t (layer-specific Q), cells/layer 10..160: 3.627, 5.056, 7.097, 9.999, 14.115;
    successive ratios 1.394, 1.404, 1.409, 1.412
PASS  S7.1 layer-specific Q: Lambda_t grows like N^{1/2} (last ratio within 0.01 of sqrt 2)
PASS  S7.2 shared scaling: Lambda_t = 1 on every mesh

SUMMARY 22/22 checks passed
ALL PASS
```

## Appendix B Independent verification record

This appendix keeps the review record of the three drafts unchanged in content. In it, `sec3_phase21_decay_source_analytical.md`, `sec4_phase21_decay_source_fv.md` and `sec5_phase22_temperature.md` are the draft files of Sections 3, 4 and 5 of this document, and `B3_reconciliation.md` is the reconciliation record reproduced in Appendix B.4. Paths under the work folder (`derive_checks`, `b3_checks`, `b7_checks` and, for Appendix B.5, `open_maths`) refer to independent checker scripts that are not part of the repository. Row numbers such as "Section 3 row 14" refer to the numbered rows of the check-log tables below. Equation and section numbers are those of this document.

### B.1 Check log of Section 3 (Part 2.1, analytical)

Adversarial check of this section, 2026-11-06. Every key result was re-derived without reusing the author's algebra or helper code. Independent scripts (working files, not in the repo):

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

1. Staircase convergence in the stiff regime ($h\lVert B\rVert \sim 10^2$ to $10^3$) is shown numerically in two independent cases (draft and B3) but not proved; the Magnus argument is formally valid only for small $h\lVert B\rVert$. Update 2026-11-13: superseded by Section 5.4.6 (order 1 proved uniformly; order 2 proved for shared scaling; for layer-specific $Q$ a bound $h^2(1+\ln(T/h))$ proved on a fixed mesh, with a constant proportional to $\Lambda$ that grows under mesh refinement (Theorem D); an order above 1 uniform in the mesh stiffness for layer-specific $Q$ remains OPEN; Appendix B.5.7).
2. The completeness proof relies on the spectral theorem for compact self-adjoint operators, read in Melrose (2009, Lecture 20, Theorem 15, p. 116) on 2026-11-13. The compactness of a kernel operator with square-integrable kernel is not yet cited (Appendix B.5.8).
3. The earlier checker quoted a balance-reading bias of $+1.283$ for the short-lived nuclide at $z = 1$; B3 gives $+1.2740$ on the (20, 8, 4, 4, 4) mesh, which agrees with the draft's $+127$ %. The checker line in the Section 4 log was left verbatim; the difference most likely comes from a different mesh (not checked).
4. Which $R_g$ and which decay treatment the CRP-6 participants used remains NOT VERIFIED; the printed 0.4673 is reproduced at four digits with $R_g = 8.3142$ or $8.3143$ (0.467296, 0.467337), while 8.314 gives 0.4672 and 8.3144, 8.3145 and the SI value give 0.4674.
5. Data gaps unchanged: capture and burn-out cross sections, the Ag-110m isomeric fraction, flux histories, kernel heavy-metal density, daughter diffusivities, the fluence cutoff conversion, as-run AGR-1 temperature histories and the CRP-6 ramp shape.
6. Notation: Section 4 still uses $B$ both for the birth rate (TECDOC $R/B$) and for the v1.0 step matrix $\mathbf B_h$; not changed because both uses are local and defined.
7. All results remain DERIVED NOT IMPLEMENTED for v2.0: no Rust was written, so nothing here is IMPLEMENTED.

### B.5 Open-item resolution (2026-11-13)

On 2026-11-13 the open items O4, O10, O11 and O12 of the pseudocode, the accuracy note of block R2.4 step 3 (item R24) and open issue 5 of Section 4.10 (item P2) were worked on, and the citations needed by O13 and O14 were looked up. Each mathematical item was derived in a draft with its own check script. A second, adversarial verification then re-derived every key step by hand and wrote an independent script that imports nothing from the author's script; it also searched for counterexamples. Each verifier applied its fixes to the draft and appended a verification record. The records follow unchanged in content, apart from the numbering and the file locations, which were converted to those of this document outside the code blocks; code blocks are verbatim and keep the placeholder labels of Table B.7. The verifier scripts and their output files are kept in the work folder, outside the repository; the author scripts are in `scripts/derivation_checks/` (Appendix A.2).

A result is labelled PROVED only where a complete argument is given, DEMONSTRATED NUMERICALLY only where script output supports it, and CLAIMED ONLY or OPEN otherwise. No item was refuted or downgraded as a whole. The verification corrected individual statements, and those corrections are listed in each record.

| Item | Where | Equations | Verdict | Final label |
|---|---|---|---|---|
| O4, rigorous truncation bound for the layered particle | Section 3.5.3 | (P21.93) to (P21.112) | CONFIRMED WITH FIXES | Bound A (P21.106), corollaries (P21.107) to (P21.110) and Bound B (P21.112): PROVED, conditional on completeness (O14) and on the classical min-max and Sturm oscillation theorems, and DEMONSTRATED NUMERICALLY; no counterexample in 1687 Bound A and 1039 Bound B comparisons; a cancellation-aware bound is OPEN |
| P2, second-order integrators and positivity | Section 4.6.1 | (P21.113) to (P21.142) | CONFIRMED WITH FIXES | barrier P2.T1 over $\mathcal S$ with lemmas P2.L1 to P2.L4, $M_{CN}=2$ and $M_{TR}=1+\sqrt2$ sharp over $\mathcal S$, the exact CN criterion (P21.125), the BDF2 and Richardson failures, ETD positivity and every closure identity: PROVED; the TR-BDF2 threshold 8.919 of the synthetic matrix and the negativity magnitudes of the runs: DEMONSTRATED NUMERICALLY; positivity of an order-2 scheme over all matrices of one fixed size $N\ge3$: OPEN |
| O12, diffusion step limit with $D_i(T(t))$ | Section 5.2.5 | (P22.60) to (P22.80) | CONFIRMED WITH FIXES | PROVED for shared scaling: (P22.69) with (P22.72) for option A, and (P22.73), (P22.74) with $\zeta_E$ for option E (bound $\varepsilon_1+\tfrac12\zeta_E^2\,\mathrm{TV}(\ln\Lambda_1)$); for layer-specific $Q$ the frozen-coefficient part is PROVED (option A for any history, option E on heating only), the non-adiabatic remainder $R_\mathrm{na}$ is CLAIMED ONLY with numerical support and a rigorous relative bound is OPEN; the sharp bound for the release fraction with decay and the source case are OPEN |
| R24, start of the release quadrature with decay | Section 5.3.7 | (P22.81) to (P22.99) | CONFIRMED WITH FIXES | the two routes coincide (P22.81), exact and PROVED; tail bounds (P22.88) to (P22.91), start bound (P22.93) for constant $\alpha$ (for varying $\alpha$ only with $\alpha_-$ and exact $E_n$), bracket (P22.94) for $c_0\ge0$, late truncation (P22.95) under coverage R24-A3c and the image form (P22.84): PROVED; Gauss-Legendre bound (P22.97) PROVED for constant $\alpha$ per piece and CLAIMED ONLY on ramps; total (P22.98) PROVED only for constant $\alpha$ on the start piece and every graded piece, under the coverage assumption R24-A3c (late truncation (P22.95)) and only for $c(R)=0$; layered cold start OPEN |
| O11, parent and daughter with different diffusivities | Section 5.3.8 | (P22.100) to (P22.120) | CONFIRMED WITH FIXES | Theorem 1 PROVED within the class (P22.101); Corollary 1 PROVED; Corollary 2 PROVED with the added hypothesis $\theta=Q/R_g\ne0$; Proposition 2 exact and semi-analytical, the cascade DEMONSTRATED NUMERICALLY; Theorem 3 PROVED (continuous form under B3, finite-volume form unconditionally); truncation bound of the cascade OPEN |
| O10, staircase in the stiff regime | Section 5.4.6 | (P22.121) to (P22.142) | CONFIRMED WITH FIXES | Theorem A (order 1 uniform in $h\lVert A\rVert$, mesh and $\lambda$; left rule sharp), Theorem B (aligned jumps exact, misaligned jump order 1), Theorem C (shared scaling: midpoint order 2 uniform, average exact) and Theorem D (order $h^2(1+\ln(T/h))$ on a fixed mesh, constant proportional to $\Lambda$): PROVED; growth of $\Lambda_t$ like $N^{1/2}$ and $V$-norm orders 1.76 to 1.81 (test case) and 1.51 to 1.96 (second case): DEMONSTRATED NUMERICALLY; limit 7/4: CLAIMED ONLY; an order above 1 uniform in the mesh stiffness for layer-specific $Q$: OPEN |
| O13, values of $N_A$ and $k_B$ | Sections 3.0, 5.1.4, 5.7 | none | VERIFIED (document read) | exact by definition (BIPM, 2019, Table 1, p. 124) |
| O14, completeness of the layered eigenfunctions | Section 3.10 | none | partly VERIFIED | spectral theorem read (Melrose, 2009, Lecture 20, Theorem 15, p. 116); compactness of an operator with square-integrable kernel NOT VERIFIED |

Table: B.6 Results of the open-item work of 2026-11-13.

Counts of the independent verification scripts, as printed in the records below: O4 227/227 PASS, P2 41 PASS lines, O12 28 PASS lines, R24 69/69 PASS, O11 26/26 PASS, O10 40/40 PASS, each ending with ALL PASS; the citation check printed 12 PASS lines and ALL PASS. These scripts were not re-run for the integration; the six author scripts were (Appendix A.2).

#### B.5.1 Placeholder labels and final equation numbers

The drafts numbered their equations with placeholder labels of the form `P2X.O4.k`. The integration numbered them in the order in which they appear in this document, after the last existing numbers (P21.92) and (P22.59). The check scripts and the verification records print the placeholder labels inside their code blocks, which are quoted verbatim; the prose of this document uses the final numbers only. The draft O11 tagged its equations out of numerical order, so its placeholder numbers 7 to 12 do not map in sequence.

| Placeholder prefix | Section | Placeholder number = final equation |
|---|---|---|
| P2X.O4 | 3.5.3 | 1 = (P21.93); 2 = (P21.94); 3 = (P21.95); 4 = (P21.96); 5 = (P21.97); 6 = (P21.98); 7 = (P21.99); 8 = (P21.100); 9 = (P21.101); 10 = (P21.102); 11 = (P21.103); 12 = (P21.104); 13 = (P21.105); 14 = (P21.106); 15 = (P21.107); 16 = (P21.108); 17 = (P21.109); 18 = (P21.110); 19 = (P21.111); 20 = (P21.112) |
| P2X.P2 | 4.6.1 | 1 = (P21.113); 2 = (P21.114); 3 = (P21.115); 4 = (P21.116); 5 = (P21.117); 6 = (P21.118); 7 = (P21.119); 8 = (P21.120); 9 = (P21.121); 10 = (P21.122); 11 = (P21.123); 12 = (P21.124); 13 = (P21.125); 14 = (P21.126); 15 = (P21.127); 16 = (P21.128); 17 = (P21.129); 18 = (P21.130); 19 = (P21.131); 20 = (P21.132); 21 = (P21.133); 22 = (P21.134); 23 = (P21.135); 24 = (P21.136); 25 = (P21.137); 26 = (P21.138); 27 = (P21.139); 28 = (P21.140); 29 = (P21.141); 30 = (P21.142) |
| P2X.O12 | 5.2.5 | 1 = (P22.60); 2 = (P22.61); 3 = (P22.62); 4 = (P22.63); 5 = (P22.64); 6 = (P22.65); 7 = (P22.66); 8 = (P22.67); 9 = (P22.68); 10 = (P22.69); 11 = (P22.70); 12 = (P22.71); 13 = (P22.72); 14 = (P22.73); 15 = (P22.74); 16 = (P22.75); 17 = (P22.76); 18 = (P22.77); 18a = (P22.78); 19 = (P22.79); 20 = (P22.80) |
| P2X.R24 | 5.3.7 | 1 = (P22.81); 2 = (P22.82); 3 = (P22.83); 4 = (P22.84); 5 = (P22.85); 6 = (P22.86); 7 = (P22.87); 8 = (P22.88); 9 = (P22.89); 10 = (P22.90); 11 = (P22.91); 12 = (P22.92); 13 = (P22.93); 14 = (P22.94); 15 = (P22.95); 16 = (P22.96); 17 = (P22.97); 18 = (P22.98); 19 = (P22.99) |
| P2X.O11 | 5.3.8 | 1 = (P22.100); 2 = (P22.101); 3 = (P22.102); 4 = (P22.103); 5 = (P22.104); 6 = (P22.105); 9 = (P22.106); 7 = (P22.107); 10 = (P22.108); 12 = (P22.109); 8 = (P22.110); 11 = (P22.111); 13 = (P22.112); 14 = (P22.113); 15 = (P22.114); 16 = (P22.115); 17 = (P22.116); 18 = (P22.117); 19 = (P22.118); 20 = (P22.119); 21 = (P22.120) |
| P2X.O10 | 5.4.6 | 1 = (P22.121); 2 = (P22.122); 3 = (P22.123); 4 = (P22.124); 5 = (P22.125); 6 = (P22.126); 7 = (P22.127); 8 = (P22.128); 9 = (P22.129); 10 = (P22.130); 11 = (P22.131); 12 = (P22.132); 13 = (P22.133); 14 = (P22.134); 15 = (P22.135); 16 = (P22.136); 17 = (P22.137); 18 = (P22.138); 19 = (P22.139); 20 = (P22.140); 21 = (P22.141); 22 = (P22.142) |

Table: B.7 Placeholder labels of the open-item drafts and their final equation numbers.

#### B.5.2 Verification record of O4, truncation bound (Section 3.5.3)

**Verdict: CONFIRMED WITH FIXES.** Bound A (P21.106), its corollaries (P21.107) to (P21.110) and Bound B (P21.112) were re-derived step by step and survive an adversarial search with no counterexample. The fixes below are wording and number corrections. None changes a formula or a label of a main result.

**Method.** Every key step of Sections 3.5.3.2 to 3.5.3.7 was re-derived by hand. The independent script `<work folder>/open_maths/O4_verify.py` was then written from scratch, with no code, helper or number taken from the author's script or from any earlier check. It was run as `.venv/Scripts/python.exe -I -X utf8 O4_verify.py` (about 12 minutes) and printed 227/227 PASS and ALL PASS. Its routes differ from the author's in three places:

- eigenvalues come from bisection on a layer-local zero counter (no global Pruefer angle), refined in mpmath on $u(R) = 0$, and are cross-checked by P1 finite elements and by the sign alternation of $u(R)$ at the mid-points between consecutive roots (checks E0 to E4);
- the exact references are the resolvent $\langle\chi_1,(\mathcal A+s)^{-1}1\rangle$ solved in closed form layer by layer (exponential basis, mpmath, complex $s$), the Green total $Q_{\alpha\beta}/V$ for $\lambda = 0$, and Talbot inversion of the resolvent for $F(t)$ and for $M + M_\mathrm{dec}$ under the on-off source with decay; the first 320 modes (200 for X5, 100 for X8) are computed in mpmath, so the Bessel defects are exact to about 40 digits and Bound B is tested without the round-off cut-off of Section 3.5.3.7;
- the tail sums of $\ell_n$ are checked against Hurwitz zeta values, not against the integral test.

**Step-by-step re-derivation.**

- Step 1. $E_N = \sum_{n>N}w_nh_n$, the identity $\kappa_n\int_0^t\mu_n = M_0 + M_\mathrm{gen} - \mu_n$ for an arbitrary source, (P21.96) and the per-mode bound (P21.97) re-derived; exact (S0d), and the per-mode bound holds for $\lambda$ up to $10^4\Lambda_1$ (A4). The $t = 0$ error $M_0(1 - \sum_{n\le N}w_n)$ follows from $\sum_nw_n = \langle\chi_1,1\rangle/V = 1$.
- Step 2. The min-max sandwich, the jump argument $\lvert\delta_j\rvert < \pi$ (the sign of $u$ and, at a zero, the sign of $u'$ are kept at each interface) and $\psi(R;\Lambda_n) = n\pi$ re-derived. The oscillation lemma is the classical Sturm theorem; the sketch leaves out the details at the singular end $r = 0$, so (P21.100) is PROVED conditional on that classical theorem, as the section states. Numerically (L1 to L3) on all 13 configurations below, with $\sqrt{\Lambda_n}\tau/\pi - n$ reaching $-3.91$ for $J = 7$ (X5) and $-0.99$ for $J = 1$ (X7).
- Step 3. The kernel moment, the flux form $\beta_n = RD_LBk_L/\Lambda_n$, the split of the normalisation over two disjoint layers and the assembly of $C_w$ were re-derived; the assembly and $C_w(\infty)$ are exact (S0c, S0g). The largest observed ratios to the lemma constants are 0.9997 for $\alpha_n$ (X9), 0.9998 for $\beta_n$ (X7) and 0.908 for $w_n$ (X9), so the constants are nearly attained and cannot be lowered much without a new argument.
- Step 4. (P21.103) to (P21.105) re-derived; $\Phi$ exact (S0e); the bounds hold against Hurwitz zeta and direct sums of $\ell_n$ (T1).
- Steps 5 and 6. Bound A follows by insertion. The Green identity, the three closed forms (P21.111) (exact, three layers, S0h; double quadrature, S0i; $s \to 0$ limit of the resolvent, G1) and the Cauchy-Schwarz steps of Bound B, including $\sigma(\Lambda',t)$ (S0j), were re-derived.

**Counterexample search.** Thirteen layer configurations: HOM2 (homogeneous sphere split in two layers, exact eigenpairs), SYN and B4 (the two cases of the section), X5 (eight layers alternating $10^{-9}$ and $10^{-16}\ \mathrm{m^2\ s^{-1}}$), X7 (fast kernel with a very slow thin skin, ratio $10^8$), X8 (slow kernel with a fast thin skin, ratio $10^8$, lemma checks only), X9 (slow kernel, fast middle, slow skin), X10 (ten layers with ratios up to $10^6$), and RND0 to RND4 (random radii, two to seven layers, $\log_{10}D$ uniform on $[-16.5,-10]$, seed 20261113). Decay up to $\lambda = 10^7\Lambda_1$ for the steady $R/B$ and $10^4\Lambda_1$ for the on-off source; times $t = 0$, $10^{-6}/\Lambda_1$ to $10^3/\Lambda_1$, and exactly at the switch-off; $N$ from the smallest admissible value upward. In 1687 comparisons with Bound A and 1039 with Bound B no violation was found. The largest ratio error/Bound A is $3.03\times10^{-2}$ (B4, a factor of 33.0); the largest error/Bound B is 0.517 (SYN, a factor of 1.9). In the extreme cases Bound A is very pessimistic (largest ratios $6.3\times10^{-5}$ for X7, $5.0\times10^{-7}$ for X10, $5.8\times10^{-9}$ for X5), and its validity threshold can be large: the smallest admissible $N$ is 266 for X7, 326 for RND4, 2020 for X5 and 9550 for X8. Bound B has no such threshold.

**Fixes made in this section.**

- F1. "a factor of at least 34" (summary paragraph and Section 3.5.3.8) changed to 33: the author's own output gives the largest ratio $2.97\times10^{-2}$ (B4, T1), and this check $3.03\times10^{-2}$.
- F2. "$\ge 3.36$" in Step 2(a) changed to "$\ge 3.35$ (minimum 3.358)"; both scripts give the minimum 3.358.
- F3. In the mode-count paragraph, "At $t = 0$ no finite $N$ helps" was false: the $t = 0$ error is $M_0(1 - \sum_{n\le N}w_n) = O(1/N)$ and is bounded by $C_w(\ell_{N+1})M_0S^{(1)}_N$. Reworded to say that the start-inventory part falls only like $1/N$ at $t = 0$.
- F4. The mode-count rule (P21.110) now repeats the standing conditions $N > J$ and $\ell_{N+1} > \Lambda^*$ of the theorem.
- F5. The summary statement on Bound B now separates the check-script scenarios (factor at least 3.4) from the wider tests here (factor at least 1.9).
- F6. The section cited "Verification record, M1" and "case X5 of the Verification record", but no such record existed. This section supplies them: M1, $1 - \sum_{n\le5}w_n = 2.768\times10^{-2}$ for SYN with $N = 5$ the smallest admissible $N$; X5, $\Lambda^* = 0.1\ \mathrm{s^{-1}}$ and smallest admissible $N = 2020$. Both are confirmed.

The repository script `scripts/derivation_checks/sec3_o4_layered_truncation_bound.py` was re-run unchanged. Its output matches Appendix A.2 line for line (91 PASS lines and ALL PASS), and its checks test what their labels claim. It was not modified. The other numbers checked against the text also agree: $\tau$, $a$, $c$, $\Lambda^*$, $C_w(\infty)$, $C_w(\ell_{11})$, $\Lambda_1$, 151 and 152 negative weights, $N = 92$ and $873$ from the mode-count rule (true errors $5.24\times10^{-12}$ and $1.10\times10^{-13}$), and the v1.0 golden released fraction $F(2\times10^5\ \mathrm{s}) = 0.4325556390928$ (Talbot gives $0.43255563909304$).

**Labels.** The PROVED labels of Table 3.8 are justified as conditional results: they rest on completeness (T3, O14) and on the min-max and Sturm oscillation theorems, as the table and the limitations state. The DEMONSTRATED NUMERICALLY labels are backed by script output. Check A6 adds a tail-based test of (P21.109) with a kernel source. Its left side is truncated at $N_\mathrm{ref}$ and is therefore a lower estimate, so the label "DEMONSTRATED NUMERICALLY without source" in Table 3.8 is kept.

**Limits of this check.** The tail-based comparisons for $N \ge 320$ and the S1 and S3 checks at long times use Bound A at $N_\mathrm{ref}$ as the allowance for the modes beyond $N_\mathrm{ref}$. That is circular in principle, but the allowance is many orders below the compared errors. The comparisons against the resolvent, the Green totals and Talbot inversion are not circular. For X8 the modal-sum half of check G1 is vacuous, since Bound A is not admissible at $N_\mathrm{ref}$; the script prints the raw relative difference instead ($9.7\times10^{-12}$). The float64 weights of X5 differ from the mpmath ones by up to $2.7\times10^{-8}$ relative, which confirms the ill-conditioning noted in Section 3.5.3.9.

Output of `<work folder>/open_maths/O4_verify.py`, condensed: all PASS lines and the summary INFO lines; per-case parameter lines and timing lines omitted.

```text
---- part 0: symbolic identities
PASS  S0a kernel moment int_0^R1 r sin(k r) dr = (sin(k R1)/k - R1 cos(k R1))/k  0.0e+00
PASS  S0b layer integrals of u^2 and r u used by the solver  5.5e-194
PASS  S0c |alpha| bound: R1^2 + 1/k^2 - (sin/k - R1 cos)^2 = (R1 sin + cos/k)^2 >= 0
PASS  S0d kappa int mu = M0 + Mgen - mu (Psi' = 0) and h_n = (Lambda mu + lambda(M0+Mgen))/kappa
PASS  S0e Phi(b,M) = int_M^inf e^{-bx^2}/x^2 dx: derivative, limit and quadrature  quad diff 1.0e-51
PASS  S0f int_N^inf (c x^2)^-p dx = 1/((2p-1) c^p N^(2p-1)), p = 1, 2, 3
PASS  S0g C_w assembly, C_w(inf) = 3 R sqrt(D1 DL)/(R1^5/2 LL^1/2), one-layer limit 2 C_w(inf) = 6D/R^2
PASS  S0h Green forms Q_bb, Q_aa, Q_ab of (P2X.O4.19), three layers, symbolic
PASS  S0i Green identity <chi, A^-1 1> by double quadrature equals Q_ab  rel 0.0
PASS  S0j sigma(L',t) = L' e^{-L' t} if L' t >= 1 else 1/(e t)
PASS  H1 homogeneous sphere split in two layers: Lambda_n = D (n pi/R)^2 and beta_n^2 = 2 R^3/(n^2 pi^2) exactly  max rel eig 8.9e-16
PASS  H2 Talbot inversion of the exact resolvent equals the converged modal sum (homogeneous split, t = 0.05 R^2/D)  diff 3.2e-53
---- case SYN: L = 5, NREF = 20000, NM (mp modes) = 320
PASS  SYN E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  SYN E1 eigenvalues strictly increasing  min rel gap 3.09e-06
PASS  SYN E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  SYN E3 first 8 eigenvalues agree with P1 finite elements  max rel 3.9e-05
PASS  SYN E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 8.0e-48
INFO  SYN float64 vs mpmath, first 320 modes: eigenvalue rel 3.8e-14, weight rel 1.3e-10
INFO  SYN tau = 674.217 s^0.5, a = 2.17121e-5, c = 9.74776e-7, Lambda* = 7.0e-5, Lambda_1 = 3.27326e-6, C_w(inf) = 0.00028237, negative weights in first 300: 151
PASS  SYN L1 min-max (P2X.O4.7) on all 20000 modes  min Lambda/(c n^2) = 3.3580
PASS  SYN L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 4  range [-1.217, 1.209]
PASS  SYN L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0003
PASS  SYN L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 0.9968, 0.6935, 0.4887 over 319 mp modes; |w| float sample 0.6698
PASS  SYN G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 9.5e-14 (allowance 2.5e-8), s->0 rel 1.0e-20
PASS  SYN T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  SYN Bound A valid from N = 5; N tested exact [5, 6, 10, 20, 50, 100, 200, 300], tail-based [5, 6, 10, 20, 50, 100, 200, 300, 500, 1000, 2000, 2500, 3000, 5000, 8000]
PASS  SYN B0 Bessel defects A_N, B_N, a_N, b_N >= 0 for all N < 320 (mpmath)
PASS  SYN A1 S4 (lambda = 0): |Q_ab/V - sum_(n<=N) w_n/Lambda_n| <= Bound A and <= Bound B
PASS  SYN A2 S2 steady R/B, lambda/Lambda_1 in ['1.0', '1.0e+2', '1.0e+4', '1.0e+7']: exact resolvent error <= Bound A and Bound B
PASS  SYN A3 S1 (no source): exact t = 0, Talbot and NREF-tail errors <= Bound A (and Bound B where defined)
PASS  SYN A4 per-mode bound 0 <= h_n <= M0 e^(-Lambda t) + K/Lambda (P2X.O4.5), stiff lambda included
PASS  SYN A5 S3 balance-form release error (Talbot reference of M + M_dec) <= Bound A and Bound B
PASS  SYN A6 profile bound (P2X.O4.17) with a constant kernel source, NREF tail (lower estimate of the lhs)
INFO  SYN error/Bound A: max 2.111e-02 (S1 t=0.001/L1 N=50 tail), median 6.56e-04, 280 comparisons; error/Bound B: max 5.167e-01
PASS  SYN C1 released fraction F(200000 s) = 1 - M/M0 by Talbot inversion of the exact resolvent equals the v1.0 golden 0.4325556390928  Talbot 0.4325556390930421
INFO  SYN reproduction: C_w(ell_11) = 0.000344782, Lambda_1 = 3.273256e-6
PASS  SYN C2 constants of Section 3.5.3.8: tau, a, c, Lambda*, C_w(inf), C_w(ell_11), Lambda_1, 151 negative weights
PASS  SYN M1 t = 0 balance-form error M0 (1 - sum_(n<=5) w_n) = 2.8e-2 M0, and N = 5 is the smallest valid N  0.02768
PASS  SYN M2 rule (P2X.O4.18) at eps = 1.0e-6 gives N = 92 (derivation: 92) and true error <= eps  true error 5.24e-12
PASS  SYN M2 rule (P2X.O4.18) at eps = 1.0e-9 gives N = 873 (derivation: 873) and true error <= eps  true error 1.10e-13
---- case B4: L = 5, NREF = 20000, NM (mp modes) = 320
PASS  B4 E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  B4 E1 eigenvalues strictly increasing  min rel gap 3.12e-05
PASS  B4 E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  B4 E3 first 8 eigenvalues agree with P1 finite elements  max rel 1.4e-05
PASS  B4 E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 4.5e-50
INFO  B4 float64 vs mpmath, first 320 modes: eigenvalue rel 3.7e-16, weight rel 1.3e-11
INFO  B4 tau = 2468.77 s^0.5, a = 1.61934e-6, c = 2.28225e-7, Lambda* = 3.125e-6, Lambda_1 = 7.16238e-7, C_w(inf) = 6.313e-6, negative weights in first 300: 152
PASS  B4 L1 min-max (P2X.O4.7) on all 20000 modes  min Lambda/(c n^2) = 3.1383
PASS  B4 L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 4  range [-0.590, 0.590]
PASS  B4 L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0003
PASS  B4 L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 0.9619, 0.5471, 0.5735 over 319 mp modes; |w| float sample 0.5793
PASS  B4 G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 4.4e-12 (allowance 1.0e-7), s->0 rel 1.0e-20
PASS  B4 T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  B4 Bound A valid from N = 5; N tested exact [5, 6, 10, 20, 50, 100, 200, 300], tail-based [5, 6, 10, 20, 50, 100, 200, 300, 500, 1000, 2000, 2500, 3000, 5000, 8000]
PASS  B4 B0 Bessel defects A_N, B_N, a_N, b_N >= 0 for all N < 320 (mpmath)
PASS  B4 A1 S4 (lambda = 0): |Q_ab/V - sum_(n<=N) w_n/Lambda_n| <= Bound A and <= Bound B
PASS  B4 A2 S2 steady R/B, lambda/Lambda_1 in ['1.0', '1.0e+2', '1.0e+4', '1.0e+7']: exact resolvent error <= Bound A and Bound B
PASS  B4 A3 S1 (no source): exact t = 0, Talbot and NREF-tail errors <= Bound A (and Bound B where defined)
PASS  B4 A4 per-mode bound 0 <= h_n <= M0 e^(-Lambda t) + K/Lambda (P2X.O4.5), stiff lambda included
PASS  B4 A5 S3 balance-form release error (Talbot reference of M + M_dec) <= Bound A and Bound B
PASS  B4 A6 profile bound (P2X.O4.17) with a constant kernel source, NREF tail (lower estimate of the lhs)
INFO  B4 error/Bound A: max 3.026e-02 (S1 t=0.001/L1 N=100 tail), median 7.68e-04, 282 comparisons; error/Bound B: max 2.806e-01
PASS  B4 C3 152 negative weights among the first 300 modes
---- case X7 fast kernel, very slow thin skin: L = 2, NREF = 6000, NM (mp modes) = 320
PASS  X7 fast kernel, very slow thin skin E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  X7 fast kernel, very slow thin skin E1 eigenvalues strictly increasing  min rel gap 1.06e-04
PASS  X7 fast kernel, very slow thin skin E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  X7 fast kernel, very slow thin skin E3 first 8 eigenvalues agree with P1 finite elements  max rel 2.2e-04
PASS  X7 fast kernel, very slow thin skin E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 9.6e-45
INFO  X7 fast kernel, very slow thin skin float64 vs mpmath, first 320 modes: eigenvalue rel 1.4e-09, weight rel 4.8e-09
INFO  X7 fast kernel, very slow thin skin tau = 15820.9 s^0.5, a = 3.94311e-8, c = 8.05682e-10, Lambda* = 0.002778, Lambda_1 = 1.9901e-9, C_w(inf) = 9.5258e-6, negative weights in first 300: 150
PASS  X7 fast kernel, very slow thin skin L1 min-max (P2X.O4.7) on all 6000 modes  min Lambda/(c n^2) = 2.4701
PASS  X7 fast kernel, very slow thin skin L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 1  range [-0.989, 0.464]
PASS  X7 fast kernel, very slow thin skin L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0002
PASS  X7 fast kernel, very slow thin skin L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 5.181e-5, 0.9998, 0.0001036 over 54 mp modes; |w| float sample 0.0005
PASS  X7 fast kernel, very slow thin skin G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 1.2e-9 (allowance 0.0097), s->0 rel 1.0e-20
PASS  X7 fast kernel, very slow thin skin T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  X7 fast kernel, very slow thin skin Bound A valid from N = 266; N tested exact [300], tail-based [300, 500, 1000, 2000, 2500]
PASS  X7 fast kernel, very slow thin skin B0 Bessel defects A_N, B_N, a_N, b_N >= 0 for all N < 320 (mpmath)
PASS  X7 fast kernel, very slow thin skin A1 S4 (lambda = 0): |Q_ab/V - sum_(n<=N) w_n/Lambda_n| <= Bound A and <= Bound B
PASS  X7 fast kernel, very slow thin skin A2 S2 steady R/B, lambda/Lambda_1 in ['1.0', '1.0e+2', '1.0e+4', '1.0e+7']: exact resolvent error <= Bound A and Bound B
PASS  X7 fast kernel, very slow thin skin A3 S1 (no source): exact t = 0, Talbot and NREF-tail errors <= Bound A (and Bound B where defined)
PASS  X7 fast kernel, very slow thin skin A4 per-mode bound 0 <= h_n <= M0 e^(-Lambda t) + K/Lambda (P2X.O4.5), stiff lambda included
PASS  X7 fast kernel, very slow thin skin A5 S3 balance-form release error (Talbot reference of M + M_dec) <= Bound A and Bound B
PASS  X7 fast kernel, very slow thin skin A6 profile bound (P2X.O4.17) with a constant kernel source, NREF tail (lower estimate of the lhs)
INFO  X7 fast kernel, very slow thin skin error/Bound A: max 6.284e-05 (S1 t=0.001/L1 N=500 tail), median 4.04e-07, 56 comparisons; error/Bound B: max 5.578e-03
---- case X5 alternating 1e-9/1e-16: L = 8, NREF = 8000, NM (mp modes) = 200
PASS  X5 alternating 1e-9/1e-16 E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  X5 alternating 1e-9/1e-16 E1 eigenvalues strictly increasing  min rel gap 8.16e-08
PASS  X5 alternating 1e-9/1e-16 E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  X5 alternating 1e-9/1e-16 E3 first 8 eigenvalues agree with P1 finite elements  max rel 1.7e-05
PASS  X5 alternating 1e-9/1e-16 E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 2.7e-45
INFO  X5 alternating 1e-9/1e-16 float64 vs mpmath, first 200 modes: eigenvalue rel 5.5e-10, weight rel 2.7e-08
INFO  X5 alternating 1e-9/1e-16 tau = 20006.3 s^0.5, a = 2.46584e-8, c = 6.1685e-9, Lambda* = 0.1, Lambda_1 = 1.07295e-8, C_w(inf) = 0.0030358, negative weights in first 200: 100
PASS  X5 alternating 1e-9/1e-16 L1 min-max (P2X.O4.7) on all 8000 modes  min Lambda/(c n^2) = 1.4079
PASS  X5 alternating 1e-9/1e-16 L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 7  range [-3.907, 1.528]
PASS  X5 alternating 1e-9/1e-16 L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0014
PASS  X5 alternating 1e-9/1e-16 L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 0.0, 0.0, 0.0 over 0 mp modes; |w| float sample 0.0002
PASS  X5 alternating 1e-9/1e-16 G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 2.6e-11 (allowance 4.2), s->0 rel 1.1e-20
PASS  X5 alternating 1e-9/1e-16 T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  X5 alternating 1e-9/1e-16 Bound A valid from N = 2020; N tested exact [], tail-based [2500, 3000]
PASS  X5 alternating 1e-9/1e-16 B0 Bessel defects A_N, B_N, a_N, b_N >= 0 for all N < 200 (mpmath)
PASS  X5 alternating 1e-9/1e-16 A1 S4 (lambda = 0): |Q_ab/V - sum_(n<=N) w_n/Lambda_n| <= Bound A and <= Bound B
PASS  X5 alternating 1e-9/1e-16 A2 S2 steady R/B, lambda/Lambda_1 in ['1.0', '1.0e+2']: exact resolvent error <= Bound A and Bound B
PASS  X5 alternating 1e-9/1e-16 A3 S1 (no source): exact t = 0, Talbot and NREF-tail errors <= Bound A (and Bound B where defined)
PASS  X5 alternating 1e-9/1e-16 A4 per-mode bound 0 <= h_n <= M0 e^(-Lambda t) + K/Lambda (P2X.O4.5), stiff lambda included
PASS  X5 alternating 1e-9/1e-16 A5 S3 balance-form release error (Talbot reference of M + M_dec) <= Bound A and Bound B
PASS  X5 alternating 1e-9/1e-16 A6 profile bound (P2X.O4.17) with a constant kernel source, NREF tail (lower estimate of the lhs)
INFO  X5 alternating 1e-9/1e-16 error/Bound A: max 5.775e-09 (S1 t=10/L1 N=3000 tail), median 1.31e-09, 14 comparisons; error/Bound B: max nan
INFO  X5 smallest N with ell_(N+1) > Lambda*: 2020
---- case X9 slow kernel, fast middle, slow skin: L = 3, NREF = 6000, NM (mp modes) = 320
PASS  X9 slow kernel, fast middle, slow skin E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  X9 slow kernel, fast middle, slow skin E1 eigenvalues strictly increasing  min rel gap 5.47e-06
PASS  X9 slow kernel, fast middle, slow skin E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  X9 slow kernel, fast middle, slow skin E3 first 8 eigenvalues agree with P1 finite elements  max rel 7.1e-05
PASS  X9 slow kernel, fast middle, slow skin E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 1.6e-45
INFO  X9 slow kernel, fast middle, slow skin float64 vs mpmath, first 320 modes: eigenvalue rel 3.2e-14, weight rel 8.6e-08
INFO  X9 slow kernel, fast middle, slow skin tau = 10662.5 s^0.5, a = 8.68131e-8, c = 5.59501e-9, Lambda* = 6.25e-7, Lambda_1 = 9.83792e-8, C_w(inf) = 8.9095e-7, negative weights in first 300: 150
PASS  X9 slow kernel, fast middle, slow skin L1 min-max (P2X.O4.7) on all 6000 modes  min Lambda/(c n^2) = 8.1001
PASS  X9 slow kernel, fast middle, slow skin L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 2  range [-0.910, 1.100]
PASS  X9 slow kernel, fast middle, slow skin L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0004
PASS  X9 slow kernel, fast middle, slow skin L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 0.9997, 0.9956, 0.2867 over 317 mp modes; |w| float sample 0.9079
PASS  X9 slow kernel, fast middle, slow skin G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 3.4e-9 (allowance 0.00018), s->0 rel 9.3e-21
PASS  X9 slow kernel, fast middle, slow skin T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  X9 slow kernel, fast middle, slow skin Bound A valid from N = 4; N tested exact [4, 5, 10, 20, 50, 100, 200, 300], tail-based [4, 5, 10, 20, 50, 100, 200, 300, 500, 1000, 2000, 2500]
PASS  X9 slow kernel, fast middle, slow skin B0 Bessel defects A_N, B_N, a_N, b_N >= 0 for all N < 320 (mpmath)
PASS  X9 slow kernel, fast middle, slow skin A1 S4 (lambda = 0): |Q_ab/V - sum_(n<=N) w_n/Lambda_n| <= Bound A and <= Bound B
PASS  X9 slow kernel, fast middle, slow skin A2 S2 steady R/B, lambda/Lambda_1 in ['1.0', '1.0e+2', '1.0e+4', '1.0e+7']: exact resolvent error <= Bound A and Bound B
PASS  X9 slow kernel, fast middle, slow skin A3 S1 (no source): exact t = 0, Talbot and NREF-tail errors <= Bound A (and Bound B where defined)
PASS  X9 slow kernel, fast middle, slow skin A4 per-mode bound 0 <= h_n <= M0 e^(-Lambda t) + K/Lambda (P2X.O4.5), stiff lambda included
PASS  X9 slow kernel, fast middle, slow skin A5 S3 balance-form release error (Talbot reference of M + M_dec) <= Bound A and Bound B
PASS  X9 slow kernel, fast middle, slow skin A6 profile bound (P2X.O4.17) with a constant kernel source, NREF tail (lower estimate of the lhs)
INFO  X9 slow kernel, fast middle, slow skin error/Bound A: max 2.089e-02 (S1 t=0.001/L1 N=2500 tail), median 1.41e-04, 248 comparisons; error/Bound B: max 1.854e-01
---- case X10 ten layers, ratio 1e6 zig-zag: L = 10, NREF = 6000, NM (mp modes) = 320
PASS  X10 ten layers, ratio 1e6 zig-zag E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  X10 ten layers, ratio 1e6 zig-zag E1 eigenvalues strictly increasing  min rel gap 1.01e-07
PASS  X10 ten layers, ratio 1e6 zig-zag E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  X10 ten layers, ratio 1e6 zig-zag E3 first 8 eigenvalues agree with P1 finite elements  max rel 3.6e-05
PASS  X10 ten layers, ratio 1e6 zig-zag E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 1.2e-38
INFO  X10 ten layers, ratio 1e6 zig-zag float64 vs mpmath, first 320 modes: eigenvalue rel 2.0e-11, weight rel 2.2e-03
INFO  X10 ten layers, ratio 1e6 zig-zag tau = 9978.54 s^0.5, a = 9.9121e-8, c = 6.1685e-9, Lambda* = 1.562e-5, Lambda_1 = 2.88382e-8, C_w(inf) = 0.001875, negative weights in first 300: 150
PASS  X10 ten layers, ratio 1e6 zig-zag L1 min-max (P2X.O4.7) on all 6000 modes  min Lambda/(c n^2) = 4.0516
PASS  X10 ten layers, ratio 1e6 zig-zag L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 9  range [-2.901, 2.822]
PASS  X10 ten layers, ratio 1e6 zig-zag L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0023
PASS  X10 ten layers, ratio 1e6 zig-zag L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 0.9664, 0.6432, 0.0007919 over 307 mp modes; |w| float sample 0.0048
PASS  X10 ten layers, ratio 1e6 zig-zag G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 4.6e-7 (allowance 0.3), s->0 rel 1.1e-20
PASS  X10 ten layers, ratio 1e6 zig-zag T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  X10 ten layers, ratio 1e6 zig-zag Bound A valid from N = 21; N tested exact [50, 100, 200, 300], tail-based [50, 100, 200, 300, 500, 1000, 2000, 2500]
PASS  X10 ten layers, ratio 1e6 zig-zag B0 Bessel defects A_N, B_N, a_N, b_N >= 0 for all N < 320 (mpmath)
PASS  X10 ten layers, ratio 1e6 zig-zag A1 S4 (lambda = 0): |Q_ab/V - sum_(n<=N) w_n/Lambda_n| <= Bound A and <= Bound B
PASS  X10 ten layers, ratio 1e6 zig-zag A2 S2 steady R/B, lambda/Lambda_1 in ['1.0', '1.0e+2', '1.0e+4', '1.0e+7']: exact resolvent error <= Bound A and Bound B
PASS  X10 ten layers, ratio 1e6 zig-zag A3 S1 (no source): exact t = 0, Talbot and NREF-tail errors <= Bound A (and Bound B where defined)
PASS  X10 ten layers, ratio 1e6 zig-zag A4 per-mode bound 0 <= h_n <= M0 e^(-Lambda t) + K/Lambda (P2X.O4.5), stiff lambda included
PASS  X10 ten layers, ratio 1e6 zig-zag A5 S3 balance-form release error (Talbot reference of M + M_dec) <= Bound A and Bound B
PASS  X10 ten layers, ratio 1e6 zig-zag A6 profile bound (P2X.O4.17) with a constant kernel source, NREF tail (lower estimate of the lhs)
INFO  X10 ten layers, ratio 1e6 zig-zag error/Bound A: max 4.982e-07 (S1 t=1000/L1 N=50 tail), median 5.13e-10, 137 comparisons; error/Bound B: max 1.005e-06
---- case X8 slow kernel, fast thin skin (lemma only): L = 2, NREF = 4000, NM (mp modes) = 100
PASS  X8 slow kernel, fast thin skin (lemma only) E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  X8 slow kernel, fast thin skin (lemma only) E1 eigenvalues strictly increasing  min rel gap 5.00e-04
PASS  X8 slow kernel, fast thin skin (lemma only) E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  X8 slow kernel, fast thin skin (lemma only) E3 first 8 eigenvalues agree with P1 finite elements  max rel 7.3e-06
PASS  X8 slow kernel, fast thin skin (lemma only) E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 2.4e-43
INFO  X8 slow kernel, fast thin skin (lemma only) float64 vs mpmath, first 100 modes: eigenvalue rel 4.4e-16, weight rel 3.3e-12
INFO  X8 slow kernel, fast thin skin (lemma only) tau = 94869.9 s^0.5, a = 1.09659e-9, c = 8.05682e-10, Lambda* = 0.1, Lambda_1 = 1.09662e-9, C_w(inf) = 9.5258e-6, negative weights in first 100: 0
PASS  X8 slow kernel, fast thin skin (lemma only) L1 min-max (P2X.O4.7) on all 4000 modes  min Lambda/(c n^2) = 1.3611
PASS  X8 slow kernel, fast thin skin (lemma only) L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 1  range [0.000, 0.067]
PASS  X8 slow kernel, fast thin skin (lemma only) L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0005
PASS  X8 slow kernel, fast thin skin (lemma only) L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 0.0, 0.0, 0.0 over 0 mp modes; |w| float sample 0.0000
INFO  X8 slow kernel, fast thin skin (lemma only) G1 modal-sum comparison skipped (Bound A not valid at NREF); raw diff 9.7e-12 relative
PASS  X8 slow kernel, fast thin skin (lemma only) G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 0.0031 (allowance 0.0031), s->0 rel 9.4e-21
PASS  X8 slow kernel, fast thin skin (lemma only) T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  X8 smallest N with ell_(N+1) > Lambda*: 9550
---- case RND0: L = 7, NREF = 6000, NM (mp modes) = 320
PASS  RND0 E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  RND0 E1 eigenvalues strictly increasing  min rel gap 5.09e-06
PASS  RND0 E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  RND0 E3 first 8 eigenvalues agree with P1 finite elements  max rel 1.9e-04
PASS  RND0 E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 3.8e-44
INFO  RND0 float64 vs mpmath, first 320 modes: eigenvalue rel 9.6e-14, weight rel 1.2e-07
INFO  RND0 tau = 9385.47 s^0.5, a = 1.12044e-7, c = 1.82348e-9, Lambda* = 4.522e-5, Lambda_1 = 2.26889e-8, C_w(inf) = 0.00014298, negative weights in first 300: 147
PASS  RND0 L1 min-max (P2X.O4.7) on all 6000 modes  min Lambda/(c n^2) = 12.4426
PASS  RND0 L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 6  range [-1.502, 1.695]
PASS  RND0 L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0017
PASS  RND0 L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 0.4787, 0.7518, 0.1036 over 301 mp modes; |w| float sample 0.0896
PASS  RND0 G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 4.8e-9 (allowance 0.018), s->0 rel 1.0e-20
PASS  RND0 T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  RND0 Bound A valid from N = 26; N tested exact [50, 100, 200, 300], tail-based [50, 100, 200, 300, 500, 1000, 2000, 2500]
PASS  RND0 B0 Bessel defects A_N, B_N, a_N, b_N >= 0 for all N < 320 (mpmath)
PASS  RND0 A1 S4 (lambda = 0): |Q_ab/V - sum_(n<=N) w_n/Lambda_n| <= Bound A and <= Bound B
PASS  RND0 A2 S2 steady R/B, lambda/Lambda_1 in ['1.0', '1.0e+2', '1.0e+4', '1.0e+7']: exact resolvent error <= Bound A and Bound B
PASS  RND0 A3 S1 (no source): exact t = 0, Talbot and NREF-tail errors <= Bound A (and Bound B where defined)
PASS  RND0 A4 per-mode bound 0 <= h_n <= M0 e^(-Lambda t) + K/Lambda (P2X.O4.5), stiff lambda included
PASS  RND0 A5 S3 balance-form release error (Talbot reference of M + M_dec) <= Bound A and Bound B
PASS  RND0 A6 profile bound (P2X.O4.17) with a constant kernel source, NREF tail (lower estimate of the lhs)
INFO  RND0 error/Bound A: max 3.086e-04 (S1 t=1/L1 N=50 tail), median 1.76e-06, 136 comparisons; error/Bound B: max 2.788e-04
---- case RND1: L = 3, NREF = 6000, NM (mp modes) = 320
PASS  RND1 E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  RND1 E1 eigenvalues strictly increasing  min rel gap 8.30e-06
PASS  RND1 E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  RND1 E3 first 8 eigenvalues agree with P1 finite elements  max rel 8.8e-04
PASS  RND1 E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 3.3e-48
INFO  RND1 float64 vs mpmath, first 320 modes: eigenvalue rel 3.7e-14, weight rel 5.9e-11
INFO  RND1 tau = 1515.05 s^0.5, a = 4.29975e-6, c = 2.21899e-8, Lambda* = 5.646e-7, Lambda_1 = 1.76545e-7, C_w(inf) = 1.5319e-6, negative weights in first 300: 152
PASS  RND1 L1 min-max (P2X.O4.7) on all 6000 modes  min Lambda/(c n^2) = 7.9561
PASS  RND1 L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 2  range [-0.797, 0.775]
PASS  RND1 L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0004
PASS  RND1 L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 0.9982, 0.9993, 0.931 over 319 mp modes; |w| float sample 0.9913
PASS  RND1 G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 1.8e-13 (allowance 1.3e-7), s->0 rel 1.0e-20
PASS  RND1 T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  RND1 Bound A valid from N = 3; N tested exact [3, 4, 5, 10, 20, 50, 100, 200, 300], tail-based [3, 4, 5, 10, 20, 50, 100, 200, 300, 500, 1000, 2000, 2500]
PASS  RND1 B0 Bessel defects A_N, B_N, a_N, b_N >= 0 for all N < 320 (mpmath)
PASS  RND1 A1 S4 (lambda = 0): |Q_ab/V - sum_(n<=N) w_n/Lambda_n| <= Bound A and <= Bound B
PASS  RND1 A2 S2 steady R/B, lambda/Lambda_1 in ['1.0', '1.0e+2']: exact resolvent error <= Bound A and Bound B
PASS  RND1 A3 S1 (no source): exact t = 0, Talbot and NREF-tail errors <= Bound A (and Bound B where defined)
PASS  RND1 A4 per-mode bound 0 <= h_n <= M0 e^(-Lambda t) + K/Lambda (P2X.O4.5), stiff lambda included
PASS  RND1 A5 S3 balance-form release error (Talbot reference of M + M_dec) <= Bound A and Bound B
PASS  RND1 A6 profile bound (P2X.O4.17) with a constant kernel source, NREF tail (lower estimate of the lhs)
INFO  RND1 error/Bound A: max 1.696e-02 (S1 t=1/L1 N=4 tail), median 1.29e-03, 208 comparisons; error/Bound B: max 8.794e-02
---- case RND2: L = 7, NREF = 6000, NM (mp modes) = 320
PASS  RND2 E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  RND2 E1 eigenvalues strictly increasing  min rel gap 1.52e-06
PASS  RND2 E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  RND2 E3 first 8 eigenvalues agree with P1 finite elements  max rel 1.0e-04
PASS  RND2 E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 7.9e-46
INFO  RND2 float64 vs mpmath, first 320 modes: eigenvalue rel 4.4e-13, weight rel 7.1e-08
INFO  RND2 tau = 5939.74 s^0.5, a = 2.79747e-7, c = 8.28649e-9, Lambda* = 6.663e-6, Lambda_1 = 3.55273e-8, C_w(inf) = 2.1197e-5, negative weights in first 300: 148
PASS  RND2 L1 min-max (P2X.O4.7) on all 6000 modes  min Lambda/(c n^2) = 4.2874
PASS  RND2 L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 6  range [-1.754, 1.806]
PASS  RND2 L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0016
PASS  RND2 L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 0.1945, 0.9991, 0.05757 over 314 mp modes; |w| float sample 0.1244
PASS  RND2 G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 1.4e-10 (allowance 0.00042), s->0 rel 1.0e-20
PASS  RND2 T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  RND2 Bound A valid from N = 10; N tested exact [10, 20, 50, 100, 200, 300], tail-based [10, 20, 50, 100, 200, 300, 500, 1000, 2000, 2500]
PASS  RND2 B0 Bessel defects A_N, B_N, a_N, b_N >= 0 for all N < 320 (mpmath)
PASS  RND2 A1 S4 (lambda = 0): |Q_ab/V - sum_(n<=N) w_n/Lambda_n| <= Bound A and <= Bound B
PASS  RND2 A2 S2 steady R/B, lambda/Lambda_1 in ['1.0', '1.0e+2', '1.0e+4', '1.0e+7']: exact resolvent error <= Bound A and Bound B
PASS  RND2 A3 S1 (no source): exact t = 0, Talbot and NREF-tail errors <= Bound A (and Bound B where defined)
PASS  RND2 A4 per-mode bound 0 <= h_n <= M0 e^(-Lambda t) + K/Lambda (P2X.O4.5), stiff lambda included
PASS  RND2 A5 S3 balance-form release error (Talbot reference of M + M_dec) <= Bound A and Bound B
PASS  RND2 A6 profile bound (P2X.O4.17) with a constant kernel source, NREF tail (lower estimate of the lhs)
INFO  RND2 error/Bound A: max 7.763e-04 (S1 t=0.001/L1 N=20 tail), median 3.66e-06, 190 comparisons; error/Bound B: max 2.750e-02
---- case RND3: L = 3, NREF = 6000, NM (mp modes) = 320
PASS  RND3 E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  RND3 E1 eigenvalues strictly increasing  min rel gap 2.57e-04
PASS  RND3 E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  RND3 E3 first 8 eigenvalues agree with P1 finite elements  max rel 1.1e-05
PASS  RND3 E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 2.3e-47
INFO  RND3 float64 vs mpmath, first 320 modes: eigenvalue rel 4.3e-16, weight rel 9.3e-12
INFO  RND3 tau = 5794.46 s^0.5, a = 2.9395e-7, c = 1.38047e-7, Lambda* = 0.0003831, Lambda_1 = 3.23223e-7, C_w(inf) = 2.9316e-5, negative weights in first 300: 140
PASS  RND3 L1 min-max (P2X.O4.7) on all 6000 modes  min Lambda/(c n^2) = 2.1168
PASS  RND3 L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 2  range [-0.465, 0.540]
PASS  RND3 L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0005
PASS  RND3 L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 0.9353, 0.402, 0.5612 over 285 mp modes; |w| float sample 0.6898
PASS  RND3 G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 2.1e-9 (allowance 0.00053), s->0 rel 9.6e-21
PASS  RND3 T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  RND3 Bound A valid from N = 38; N tested exact [50, 100, 200, 300], tail-based [50, 100, 200, 300, 500, 1000, 2000, 2500]
PASS  RND3 B0 Bessel defects A_N, B_N, a_N, b_N >= 0 for all N < 320 (mpmath)
PASS  RND3 A1 S4 (lambda = 0): |Q_ab/V - sum_(n<=N) w_n/Lambda_n| <= Bound A and <= Bound B
PASS  RND3 A2 S2 steady R/B, lambda/Lambda_1 in ['1.0', '1.0e+2']: exact resolvent error <= Bound A and Bound B
PASS  RND3 A3 S1 (no source): exact t = 0, Talbot and NREF-tail errors <= Bound A (and Bound B where defined)
PASS  RND3 A4 per-mode bound 0 <= h_n <= M0 e^(-Lambda t) + K/Lambda (P2X.O4.5), stiff lambda included
PASS  RND3 A5 S3 balance-form release error (Talbot reference of M + M_dec) <= Bound A and Bound B
PASS  RND3 A6 profile bound (P2X.O4.17) with a constant kernel source, NREF tail (lower estimate of the lhs)
INFO  RND3 error/Bound A: max 5.184e-03 (S1 t=0.001/L1 N=100 tail), median 1.58e-04, 108 comparisons; error/Bound B: max 2.454e-02
---- case RND4: L = 4, NREF = 6000, NM (mp modes) = 320
PASS  RND4 E0 bisection brackets valid (Z(lo) < n <= Z(hi))
PASS  RND4 E1 eigenvalues strictly increasing  min rel gap 1.77e-04
PASS  RND4 E2 u(R) alternates in sign between consecutive roots and Z(mid_n) = n, n <= 2000
PASS  RND4 E3 first 8 eigenvalues agree with P1 finite elements  max rel 2.9e-05
PASS  RND4 E4 mpmath roots bracketed by the counter and flux identity for beta (mpmath)  max |beta_mom - beta_flux|/|beta| = 1.5e-45
INFO  RND4 float64 vs mpmath, first 320 modes: eigenvalue rel 5.1e-15, weight rel 1.9e-11
INFO  RND4 tau = 4564.55 s^0.5, a = 4.737e-7, c = 8.64756e-8, Lambda* = 0.04945, Lambda_1 = 1.11433e-7, C_w(inf) = 0.0038203, negative weights in first 300: 153
PASS  RND4 L1 min-max (P2X.O4.7) on all 6000 modes  min Lambda/(c n^2) = 1.2886
PASS  RND4 L2 travel time (P2X.O4.8) |sqrt(Lambda) tau/pi - n| < J = 3  range [-0.880, 0.890]
PASS  RND4 L3 Lambda_n >= ell_n (P2X.O4.9)  min ratio 1.0008
PASS  RND4 L4 amplitude lemma (P2X.O4.10): |alpha|, |beta|, |w| below C_alpha, C_beta, C_w  max ratios 0.0, 0.0, 0.0 over 0 mp modes; |w| float sample 0.0647
PASS  RND4 G1 resolvent: <chi,(A+s)^-1 1>/V equals the modal sum (s = Lambda_1) and Q_ab/V (s -> 0)  diff 3.9e-10 (allowance 0.027), s->0 rel 1.0e-20
PASS  RND4 T1 S^(p)_N (Hurwitz zeta) and Y_N(s) (direct sums) bound the ell_n tails
INFO  RND4 Bound A valid from N = 326; N tested exact [], tail-based [500, 1000, 2000, 2500]
PASS  RND4 B0 Bessel defects A_N, B_N, a_N, b_N >= 0 for all N < 320 (mpmath)
PASS  RND4 A1 S4 (lambda = 0): |Q_ab/V - sum_(n<=N) w_n/Lambda_n| <= Bound A and <= Bound B
PASS  RND4 A2 S2 steady R/B, lambda/Lambda_1 in ['1.0', '1.0e+2', '1.0e+4', '1.0e+7']: exact resolvent error <= Bound A and Bound B
PASS  RND4 A3 S1 (no source): exact t = 0, Talbot and NREF-tail errors <= Bound A (and Bound B where defined)
PASS  RND4 A4 per-mode bound 0 <= h_n <= M0 e^(-Lambda t) + K/Lambda (P2X.O4.5), stiff lambda included
PASS  RND4 A5 S3 balance-form release error (Talbot reference of M + M_dec) <= Bound A and Bound B
PASS  RND4 A6 profile bound (P2X.O4.17) with a constant kernel source, NREF tail (lower estimate of the lhs)
INFO  RND4 error/Bound A: max 5.076e-04 (S1 t=0.001/L1 N=1000 tail), median 9.99e-07, 28 comparisons; error/Bound B: max nan
INFO  all cases: 1687 Bound A comparisons, max error/A = 3.026e-02; 1039 Bound B comparisons, max error/B = 5.167e-01
INFO  SYN: max error/A = 2.111e-02 (factor 47.4), max error/B = 5.167e-01
INFO  B4: max error/A = 3.026e-02 (factor 33.0), max error/B = 2.806e-01
INFO  run time 699 s
227/227 PASS
ALL PASS
```

#### B.5.3 Verification record of P2, second-order integrators and positivity (Section 4.6.1)

An independent adversarial check re-derived every key step without reusing the author's algebra or helper code (verification script `<work folder>/open_maths/P2_verify.py`, kept in the work folder, outside the repository; run with `.venv/Scripts/python.exe -I -X utf8 P2_verify.py`). The script builds the TR-BDF2 stability function from the $\gamma$ stages, recomputes the orders and $c_r$, the BDF2 roots and start amplitude, the closed-form derivatives (P21.133) and (P21.136) for $k\le10$, the construction (P21.116) and limit (P21.117) at 60 digits, and re-implements the mesh, the CN pivot criterion (P21.125), every scheme and every closure quadrature (the ETD step integral is checked against a Van Loan block exponential). The citation was checked against the Numdam scan: Théorème 1 on printed p. 241, Lemme 3 and Théorème 2 on p. 242 (with $\sigma(\mathbf A)$ built from the diagonal coefficients), the Crank-Nicolson condition $\Delta t\,\sigma(\mathbf A)\le2$ on p. 243 and Théorème 3 on p. 244, as stated. The hand re-derivation of P2.L1 to P2.L4 and P2.T1 found no gap in the arguments.

Counterexample search. Lemma P2.L3 was attacked with 400 random members of $\mathcal S$ ($N\le20$, volumes, conductances, $G_R$ and $\lambda$ spread over up to 20 decades, $\lambda=0$ included) at $h\beta_B=M_r$, $0.999M_r$, $0.3M_r$ and $10^{-3}M_r$ for CN and TR-BDF2. At 60 digits 45 evaluations gave entries of order $-10^{-61}$; recomputed at 250 digits all were nonnegative, so they were round-off and no counterexample exists in the sample (V3a). The order-2 function $1/(1+x+x^2/2)$ stayed nonnegative on 3000 random $N=2$ members (V2c) and on 3000 random $N=3$ members (information only; the fixed-$N$ question stays OPEN). The CN criterion (P21.125) matched $(\mathbf Q_\tau)_{jj}\ge\frac12$ on 200 random members to $2.4\times10^{-60}$ relative (V4a). All numerical results of Section 4.6.1.8 that were re-run (matrix data, $h^*_{CN}$, $h^*_{TR}$ by bisection at 50 digits, Table 4.5, negative-cell pattern, (P21.141) ratios, step counts, CRP-6 arithmetic, the $N^2$ growth) were reproduced. The switching-source minima of S6a were not re-run with the author's schedule; V7a checks the same closure identities on an independent random system with variable steps, switching source and decay.

Fixes made in this file (none changes a theorem or a label of the main results):

1. (P21.114): the necessity argument named only the test $\mathbf s=0$, $\mathbf C^n=\mathbf e_j$; the test $\mathbf C^n=0$ for the source part was added.
2. P2.L3: the polynomial case (no pole, so P2.L2 does not apply) is now handled explicitly.
3. Remark after P2.T1: the sentence calling the fixed-$N$ question "irrelevant for STRATO, whose meshes have $N\gg5$" was wrong, since a STRATO run has one fixed $N$ and P2.T1 needs matrices of every size. It was replaced by the correct scope statement and by Proposition P2.P1 (PROVED), which shows that CN and TR-BDF2 have finite thresholds on every matrix, so the comparison of the candidates is unaffected.
4. Section 4.6.1.3: "Any practical STRATO step has $h\mu_{max}>\frac12$" was unsupported; it is now quantified on the synthetic matrix ($h>50.7\ \mathrm{s}$, V9m) and labelled CLAIMED ONLY elsewhere.
5. Section 4.6.1.7: the second-order range $\Lambda_1h\approx0.03$ to $0.04$ omitted Richardson ($0.021$); corrected to $0.02$ to $0.04$.
6. Section 4.6.1.8: "guaranteed positivity" for TR-BDF2 at 11224 steps rests on a grid scan, so it is now labelled DEMONSTRATED NUMERICALLY, and the state-dependent TR-BDF2 image statement now gives the first negative point on a 400-point grid ($\Lambda_1h=0.62$) instead of suggesting a threshold between the six sampled points.

The repository check script `sec4_p2_positive_second_order.py` produced the same numbers as the independent script wherever both compute a quantity; no error was found in it and it was not changed.

Verification script output:

```text
PASS  V1a TR-BDF2 r from the gamma stages equals (P2X.P2.20) partial fractions  
PASS  V1b a=(sqrt2+1)/2, b=(sqrt2-1)/2, beta=1-1/sqrt2=gamma/2, a-b=1  
PASS  V1c orders and c_r of ln r = -x + c x^(p+1) (BE 1/2, CN -1/12, TR 2/3-sqrt2/2, RE 1/6)  BE: c=1/2; CN: c=-1/12; TR: c=2/3 - sqrt(2)/2; RE: c=1/6
PASS  V1d BDF2: roots solve (3/2+x)xi^2-2xi+1/2=0; ln xi1 = -x - x^3/3 + ...; BE-start amplitude 1 + 3/4 x^2  ln xi1 = -xs**4/4 - xs**3/3 - xs, alpha = xs**3/2 + 3*xs**2/4 + 1
PASS  V1e BDF2 |xi|^2 = 1/(3+2x) for complex roots (x=1.7)  
PASS  V1f closed forms (P2X.P2.21), (P2X.P2.24) and CN derivative formula hold for k = 0..10  
PASS  V1g M_TR = 1+sqrt2: k=0 bracket zero there; brackets for k>=1 positive at x=1+sqrt2  
PASS  V1h TR min at x = 4+3sqrt2 with value -(sqrt2-1)/2 (unique stationary point, numeric root compare)  
PASS  V1i RE: r=0 at x = 2+2sqrt2; min on (0,inf) approx -0.0361 near 11.76  
PASS  V1j RE numeric minimum  min -0.03612 at 11.759
PASS  V1k RE has a failing k at every tested x; Taylor coefficient of x^4 at 0 is -3/8  first failing k {0.0: 4, 0.001: 4, 0.5: 5, 2.0: 10, 7.0: 0, 50.0: 0, 1000.0: 0}
PASS  V1l r=1/(1+x+x^2/2): order 2, r^(4)(0) < 0 and r^(4)(0.1) < 0  r''''(0)=-6, r''''(0.1)=-3.4412
PASS  V2a L1: B_delta in S with the stated entries; entry (1,k+1) -> (-1)^k r^(k)(x) eps^k/k! (delta=1e-12, N=6)  CN x=1.0: rel 1.6e-13; TR x=1.5: rel 1.2e-13; RE x=0.3: rel 2.5e-11
PASS  V2b order-2 r=1/(1+x+x^2/2) has a negative entry on an N=5 member of S  entry(1,5) = -8.96e-7
PASS  V2c N=2: 1/(1+x+x^2/2) gives r(hB) >= 0 on 3000 random 2x2 members (16 decades of data)  min entry 3.2e-47
      info: N=3 random search for 1/(1+x+x^2/2): min entry 8.19e-44 (fixed-N question stays OPEN)
      info: V3a entries rechecked at 250 digits: 45
PASS  V3a L3: r(hB) >= 0 on 400 random members (N <= 20, data spread up to 20 decades) at h beta_B in {M_r, 0.999 M_r, 0.3 M_r, 1e-3 M_r}  min CN 0.0, TR 0.0
PASS  V3b sharpness: N=1 member fails just above M_r for CN and TR  
PASS  V4a (P2X.P2.13): (Q_tau)_jj = V_j/(V_j + LHS_j) and the criterion matches Q_jj >= 1/2 (200 random members)  max rel 2.4e-60
PASS  V4b (Q_tau)_jj strictly decreasing in tau (sampled)  
PASS  V5 every tested member: for large h all entries of r_TR(hB) < 0 and every diagonal of r_CN(hB) < 0  
PASS  V6a BDF2: C^n=0, C^{n-1}=e_j gives C^{n+1}_j < 0 for every tested h and member  
PASS  V6b local Richardson: entry (j,j+4) = -3/8 h^4 prod|B_{i,i+1}| (1+O(h)) < 0  ratios 0.99999961, 0.99999794, 0.99999826
PASS  V6c global Richardson modal crossing at mu h = 0.5: n* = 16.98, sign flips between 16 and 17  n* = 16.980
PASS  V7a closure with scheme quadrature (variable steps, switching source, decay): CN trapezoid, TR stage weights, RE combined, ETD exact integral  CN 4.4e-16; TR 6.6e-16; RE 1.3e-15; ETD 1.6e-14
PASS  V7b BDF2 (BE start, constant step): identity (P2X.P2.18) with the modified inventory  max rel 2.9e-15
PASS  V7c TR source term beta h[(1+sqrt2)Q^2 + Q] s (P2X.P2.20)  
PASS  V7d steady state B^-1 s is a fixed point (CN, ETD; h from 1e-3 to 1e4)  
PASS  V8 uniformisation and squaring (q=14, tau bmax <= 1/8): every entry >= 0.0 and agrees with expm  h bmax=0.01: rel 2.2e-16; h bmax=1: rel 3.4e-15; h bmax=100: rel 3.1e-13; h bmax=100000: rel 1.6e-10
PASS  V9a synthetic matrix: N=40, Lambda_1 = 7.166e-7 s^-1, beta_B/Lambda_1 = 7150  Lambda_1 = 7.16597e-07, beta_B = 5.12376e-03, ratio 7150.1, mu_max/mu_1 = 12515
PASS  V9b CN threshold h*_CN = 586 s, h* beta_B = 3.003, Lambda_1 h* = 4.20e-4, binding cell 1-based 24 (material 1, buffer)  h* = 586.084 s, h* bB = 3.00296, Lambda_1 h* = 4.19986e-04, j = 23, material 1
PASS  V9c TR-BDF2 on the synthetic matrix: r(hB) >= 0 at 13 grid points up to h bB = 8.91, first failure at h bB = 8.919 (bisected, 50 digits)  threshold in [8.91879, 8.91879], Lambda_1 h* = 1.2474e-03, min entry at bound 2.414: 2.88e-55
PASS  V9d reference M(t_end)/M0 = 2.4276e-7 (double eigen route) agrees with 30-digit mpmath  M/M0 = 2.427556e-07, rel diff 3.3e-12
      BE    rel err: +6.21e+00 +1.93e+00 +7.58e-01 +3.35e-01 +1.58e-01 +7.64e-02 +3.76e-02 | order 1.022 | min C +0.00e+00
      CN    rel err: -2.32e+01 -3.22e-02 -7.59e-03 -1.90e-03 -4.75e-04 -1.19e-04 -2.97e-05 | order 2.000 | min C -5.77e-01
      TR    rel err: -5.94e-02 -1.49e-02 -3.71e-03 -9.26e-04 -2.31e-04 -5.77e-05 -1.44e-05 | order 2.001 | min C +0.00e+00
      RE    rel err: +1.75e-01 +5.02e-02 +1.37e-02 +3.60e-03 +9.25e-04 +2.34e-04 +5.90e-05 | order 1.990 | min C -8.45e-05
      BDF2  rel err: -4.54e-01 -1.13e-01 -2.74e-02 -6.68e-03 -1.65e-03 -4.09e-04 -1.02e-04 | order 2.005 | min C +0.00e+00
      ETD   rel err: +2.81e-14 +6.26e-14 +1.07e-13 +2.28e-13 +4.87e-13 +9.73e-13 +1.78e-12 | order -0.867 | min C +0.00e+00
PASS  V9e Table 4.X.2 relative errors reproduced at n = 50, 200, 800, 3200 (to the printed 2 digits)  
PASS  V9f negative cells: BE, TR, BDF2, ETD none; CN n <= 1600; RE n >= 800  {'BE': [], 'CN': [50, 100, 200, 400, 800, 1600], 'TR': [], 'RE': [800, 1600, 3200], 'BDF2': [], 'ETD': []}
PASS  V9g (P2X.P2.29) predicts the n=3200 error with sign (obs/pred)  BE 1.0154, CN 1.0000, TR 1.0004, RE 0.9929, BDF2 1.0037
PASS  V9h kernel-loaded start: TR image >= 0 for Lambda_1 h up to 0.42 and negative by 1.4 (400-point grid)  first negative image at Lambda_1 h = 0.6229
PASS  V9i step counts for 1e-3 at mu_1 t = 15.4: BE 118580, CN 552, TR 384, RE 780, BDF2 1020  {'BE': np.float64(118580.0), 'CN': np.float64(551.7), 'TR': np.float64(384.3), 'RE': np.float64(780.2), 'BDF2': np.float64(1019.6)}
PASS  V9j positivity step counts: CN 33334, TR class bound 41464  CN 33334.4, TR(class) 41463.6, TR(h*) 11223.7
PASS  V9k CRP-6 1b steps: BE 7.1 s, CN 1476 s, TR 2118 s, RE 1043 s; second-order Lambda_1 h spans 0.021 to 0.042  BE 7.1 s (Lambda_1 h 0.000), CN 1475.6 s (Lambda_1 h 0.029), TR 2118.3 s (Lambda_1 h 0.042), RE 1043.4 s (Lambda_1 h 0.021)
PASS  V9l homogeneous uniform sphere: beta_B/Lambda_1 = 567 (N=40), 7979 (N=150); CN cap 2/7979 = 2.51e-4, 12.7 s with CRP-6 Lambda_1  567.32, 7979.0, 12.72 s
PASS  V9m synthetic matrix: BDF2 has complex modal roots (h mu_max > 1/2) for every h > 50.7 s  0.5/mu_max = 50.69 s, Lambda_1 h there = 3.63e-05
ALL PASS
```

#### B.5.4 Verification record of O12, step limit with a temperature history (Section 5.2.5)

Adversarial check of this section, 2026-11-13, by an independent script (`<work folder>/open_maths/O12_verify.py`, kept in the work folder, outside the repository). The script imports nothing from the repository script (Appendix A.2) and shares none of its code: it has its own mesh assembly from (P22.14), its own effective time by Gauss-Legendre (checked against adaptive quadrature), its own step planner written from (P22.72), exact modal references for shared scaling, a Radau reference for layer-specific $Q$ and an ODE quadrature for the decayed fraction. The repository script was also rerun unchanged and again ends in ALL PASS.

Verdict: CONFIRMED WITH FIXES. The main result, the theorem (P22.69) and (P22.70) with the step (P22.72) for shared scaling and option A, holds. No counterexample was found in 80 adversarial histories (random, hold-and-ramp, violent cycling between 900 and 2100 K, monotone heating; layer diffusivity ratios up to $10^8$; $Q$ from 50 to 700 $\mathrm{kJ\ mol^{-1}}$; $\lambda t_{\mathrm{end}}$ up to 6; $\Theta_{\mathrm{end}}$ from 0.05 to 12; check at every step including the first), where the largest $\delta_1/\varepsilon_1$ was 0.99934 (B4). Every PROVED label was re-derived and holds. The numbers quoted from N1, N2, N3 (rows without $\varepsilon_D$), N4 and the minima of (P22.71) were reproduced by the independent code to the printed digits.

Steps re-derived independently.

- Lemma 1 and Lemma 2: symbolic derivatives (A1); homogeneity, monotonicity and the ratio bound at 40 digits on graded meshes with layer ratios up to $10^{12}$ (B1). The derivative form in (P22.61) follows from the ratio bound applied to the Dini quotients, so differentiability of $\Lambda_1$ is not needed.
- (P22.63): the weighted harmonic mean is concave (Hessian determinant 0, trace negative, A5); $z_k^A\ge Z_k$ and the observed order 3 of the gap (B8).
- (P22.64), (P22.65): checked for every resolvable mode, not only the slowest, on random stiff steps with $z$ up to about $10^3$ and $\lambda\ge0$ (B3).
- (P22.66), (P22.67): $\sum w_m=1$, $W=2.1798\le3.3808$, $w_1=1.1453$ (B2b).
- (P22.72): $Z_k\le\zeta$ in every case (B2c, B4), and the proof (monotone $\Lambda_1$ on each step, maximum at one end) is complete.
- (P22.73), (P22.74): algebra (A3), lag bound and the option E rule at every step on cycling histories with up to 11 ramps (B5). The cool-down values $L_1=-1.989\times10^{-3}$ and $8.01249\times10^{-3}$ are reproduced only when $\zeta$ uses $\Theta_{\mathrm{end}}$ of the full history; the text now says so.
- (P22.75), (P22.76): bound at every $n$ from 3 to 399 with uniform-random and near-surface starts (B6).
- (P22.79): $\lVert(I+\Delta t K)^{-1}\rVert_V\le1$ on 40 random meshes (B7).
- (P22.80): identity and the three bounds for every mode, and $F_h-F=5.8884\times10^{-5}$ and $5.9088\times10^{-6}$ reproduced with an exact decayed fraction (B10).

Fixes made in this section.

1. Section 5.2.5.6, the sentence that the rule "keeps the first sum below $\varepsilon_1$ exactly as in Section 5.2.5.4" was not proved. For option E, $\sum_kz_k^E=\Theta-L_1$ exceeds $\Theta$ on heating, and for option A $z_k^A\ge Z_k$, so the argument of Section 5.2.5.4 does not carry over unchanged. It is replaced by (P22.78), which is PROVED. Option A: $z_k^A\le\zeta$ (new, from Lemma 2 since $\bar D^{(k)}$ is componentwise at most $D$ at the hotter end) and the frozen-coefficient exponent is at most $\varepsilon_1$ for every history. Option E: the exponent is at most $\varepsilon_1+(1-\tfrac12\zeta)L_1$. For both, the first sum alone is at most $\varepsilon_1+\tfrac12\zeta^2\mathrm{TV}(\ln\Lambda_1)$. Numerically the frozen exponent of option A is $9.962\times10^{-3}\le10^{-2}$ and $2.996\times10^{-3}\le3\times10^{-3}$ (B9d), and the largest first sum for option E in 15 histories is $0.975\,\varepsilon_1$ (B5b), so the old sentence is unproved but no counterexample to it was found. A row for (P22.78) was added to Table 5.10, and the pseudocode O12 text now cites it.
2. Section 5.2.5.5, the label of $c_n\le2/3$ is raised from DEMONSTRATED NUMERICALLY to PROVED: $c_n=2(1-2/n)^{n-2}$ exactly, and $\frac{d}{dn}\ln c_n=\ln(1-2/n)+2/n<0$ (A2). Table 5.10 was updated to match.
3. Section 9.5 of the pseudocode, "R2.4 evaluates the effective-time solution exactly in time" is not accurate for $\lambda>0$, where step 3 of R2.4 uses a release quadrature. It now reads "without time stepping (its only time error is the release quadrature of step 3 when $\lambda>0$)". The conclusion (no change to R2.4) stands.
4. Section 5.2.5.4, option E paragraph: $\zeta=2\times10^{-2}/\Theta$ now names $\Theta_{\mathrm{end}}$ of the history in question (see the cool-down values above).

Labels left as they are. (P22.68), (P22.77) and the leading-order lag stay approximate. $R_{\mathrm{na}}=O(\varepsilon_D)$ stays CLAIMED ONLY with numerical support on one history. A relative bound for layer-specific $Q$, the sharp release-fraction bound with decay and the source case stay OPEN. The rows of Table 5.9 with $\varepsilon_D$ were not rerun independently; the four rows without $\varepsilon_D$ were (B9d). The repository script needs no change.

Output of `<work folder>/open_maths/O12_verify.py` (run with the project interpreter, `-I -X utf8`):

```text
PASS  A1 Lemma 1: (z^2/2-g)'=z^2/(1+z)>=0, (g-z^2/2+z^3/3)'=z^3/(1+z)>=0, both vanish at 0
PASS  A2 x*=2n/(n-2); c_n = 2(1-2/n)^(n-2) (closed form); d ln c_n/dn = ln(1-2/n)+2/n < 0 for all n>=3 (so c_n decreasing is PROVED, not only numerical); c_3 = 2/3, limit 2e^-2  c_3=2/3, limit=2*exp(-2)
PASS  A2b sup_x n[(1+x/n)^-n - e^-x] <= c_n on a dense grid, n in {3,4,5,10,100,1e4}
PASS  A3 (P2X.O12.15): (1/2)zeta_E(Theta+zeta_E TV)+zeta_E TV = eps + (1/2)zeta_E^2 TV
PASS  A4 constant D with decay: F_h = p(1-a_h) and F = p(1-a), so F_h - F = -p(a_h - a) exactly
PASS  A5 G(X,Y)=1/(a/X+b/Y): det Hessian = 0 and trace < 0 (concave, Jensen step of (P2X.O12.4))
PASS  A6 option E: Z_k - z_k = -(1/2) dt^2 Lambda_1' + O(dt^3) (Taylor, leading-order lag)
PASS  B1 Lemma 2 at 40 digits: homogeneity, monotonicity in each D_i, min ratio <= Lambda1'/Lambda1 <= max ratio (12 random graded meshes, layer ratios up to 1e12)
PASS  B2a own model: Lambda_1(1600 C), Lambda_1(1250 C), Theta_end reproduce 1.93320e-5, 2.3119e-7, 6.5945; tau by Gauss-Legendre agrees with adaptive quad  Lambda_1 = 1.933199e-05, 2.31187e-07, Theta = 6.59449, Lambda_max t_end = 6.9595
PASS  B2b sum w_m = 1, W = sum|w_m| <= sqrt(V_tot/V_K); W and w_1 reproduce 2.18, 1.145  W = 2.1798, bound 3.3808, w_1 = 1.1453
PASS  B2c eps=0.01: Z_k <= zeta, delta_1 = sum g(Z_k) at every step (exact identity), delta_1 <= zeta Theta(t_n)/2 <= eps, steps and error match the document  steps 2181, max|delta_1 - sum g| 6.3e-11, delta_1(end) 9.96683e-03, rel err 1-F 1.001666e-02
PASS  B2c eps=0.001: Z_k <= zeta, delta_1 = sum g(Z_k) at every step (exact identity), delta_1 <= zeta Theta(t_n)/2 <= eps, steps and error match the document  steps 21747, max|delta_1 - sum g| 6.5e-11, delta_1(end) 9.99680e-04, rel err 1-F 1.000180e-03
PASS  B2d F3 constant step with Lambda_1 at T_max: steps and error reproduce the document (24219, 0.0009395)  steps 24219, rel err 9.39502e-04
PASS  B2d F3 constant step with Lambda_1 at T_min: steps and error reproduce the document (5, 15.553)  steps 5, rel err 1.55528e+01
PASS  B2e option E (2 segments): delta_1^E = sum g(z) + L_1 exactly; |L_1| <= zeta TV(ln Lambda_1); lag and 1-F error reproduce -0.004784, 0.00520876  L_1 -4.7840e-03, zeta TV 1.3424e-02, rel err 5.20876e-03, sum g(z) 9.9792e-03
PASS  B2e option E (4 segments): delta_1^E = sum g(z) + L_1 exactly; |L_1| <= zeta TV(ln Lambda_1); lag and 1-F error reproduce -0.001989, 0.00801249  L_1 -1.9893e-03, zeta TV 2.6775e-02, rel err 8.01249e-03, sum g(z) 9.9699e-03
INFO  B2 done in 6 s
PASS  B3 random graded meshes, random stiff steps (z up to ~1e3), lambda >= 0, options A and E: ln(c_m^h/c_m) = sum g(z_m,k) + sum (Z_m,k - z_m,k) for every resolvable mode  max deviation 2.9e-07
PASS  B4 theorem, 80 adversarial shared-scaling cases (random/hold/cycling/monotone histories, ramps 0.03 to 30 h before rescaling, layer D ratios to 1e8, Q 50 to 700 kJ/mol, lambda t_end 0 or 0.05 to 6, Theta_end 0.05 to 12): Z_k <= zeta, 0 <= delta_1(t_n) <= zeta Theta(t_n)/2 <= eps at every step, N >= Theta^2/sum Z^2  max delta_1/eps = 0.999339; 11 s
PASS  B5 option E with zeta_E on thermal cycling (up to 11 ramps 1000 <-> 2100 K): z_k <= zeta_E, |L_1(t_n)| <= zeta_E TV, |delta_1^E(t_n)| <= eps + zeta_E^2 TV/2 at every step
INFO  B5b heating only Q=500 kJ/mol, Theta=0.5, eps=0.05: sum g(z^E)/eps = 0.7302, sum z^E/Theta = 1.1970, delta_1^E/eps = -1.2394, steps 25
INFO  B5b heating only Q=500 kJ/mol, Theta=1.0, eps=0.05: sum g(z^E)/eps = 0.8173, sum z^E/Theta = 1.0724, delta_1^E/eps = -0.6312, steps 90
INFO  B5b heating only Q=700 kJ/mol, Theta=0.5, eps=0.02: sum g(z^E)/eps = 0.7550, sum z^E/Theta = 1.0813, delta_1^E/eps = -1.2764, steps 70
INFO  B5b heating only Q=300 kJ/mol, Theta=2.0, eps=0.05: sum g(z^E)/eps = 0.9500, sum z^E/Theta = 1.0348, delta_1^E/eps = -0.4431, steps 210
INFO  B5b heating only Q=700 kJ/mol, Theta=1.0, eps=0.1: sum g(z^E)/eps = 0.6912, sum z^E/Theta = 1.0969, delta_1^E/eps = -0.2778, steps 56
PASS  B5b option E with zeta = 2 eps/Theta (15 cases, 5 heating only): proved bound sum g(z^E) <= zeta(Theta - L_1)/2 <= eps + zeta^2 TV/2 holds; on heating only L_1 <= 0 and delta_1^E <= eps  max sum g(z^E)/eps = 0.9752 (Theta_end 6.197, TV(ln Lambda_1) 57.4); 0 of 15 cases exceed eps (no counterexample to 'below eps' found)
PASS  B6 equal Z_1,k: |F_h - F| <= W c_n / n for all 3 <= n < 400, uniform-random and near-surface starts  max |err|/(W c_n/n) = 0.324
PASS  B7 ||(I + dt K)^-1||_V <= 1 for 40 random graded meshes, dt from 1e-2 to 1e8 s, lambda >= 0
PASS  B8 layer-specific Q: z^A >= Z (P2X.O12.4), z^A <= dt max Lambda_1 on the step, Lambda-hat >= Lambda_1; Jensen gap observed order near 3  gap orders [3.   3.   2.99 3.01 2.96 2.99 3.01 2.99]
PASS  B9a layer-specific Q: Lambda_1(1250 C) and Theta_end reproduce 1.3501e-8 and 6.5554  1.35014e-08, Theta 6.55544
PASS  B9b Radau reference validated on the shared case against the exact modal 1-F  rel 7.5e-11
PASS  B9c layer-specific reference 1-F reproduces 1.572490e-3 and is tolerance-converged  1-F_ref 1.5724899e-03, tol change 4.1e-13
INFO  B9 eps=0.01 option A: observed 1.05273e-02, frozen prediction 1.00116e-02, remainder +5.157e-04, frozen exponent 9.96178e-03, max z/zeta 1.000000, steps 2208
INFO  B9 eps=0.01 option E: observed 6.40566e-03, frozen prediction 5.70738e-03, remainder +6.983e-04, frozen exponent 5.69116e-03, max z/zeta 1.000000, steps 2208
INFO  B9 eps=0.003 option A: observed 3.34061e-03, frozen prediction 3.00037e-03, remainder +3.402e-04, frozen exponent 2.99588e-03, max z/zeta 1.000000, steps 7254
INFO  B9 eps=0.003 option E: observed 1.53585e-03, frozen prediction 1.20690e-03, remainder +3.289e-04, frozen exponent 1.20618e-03, max z/zeta 1.000000, steps 7254
PASS  B9d layer-specific Q, rule with instantaneous Lambda_1: observed errors reproduce the document (A and E); for option A z^A <= zeta and the frozen exponent sum[Z - ln(1+z^A)] <= eps (new bound)  94 s
INFO  B10 eps=0.01: steps 3967, retained-M rel err 1.00325e-02, F = 0.636306, F_h - F = 5.8884e-05 (tau check 4.3e-15), balance 1.3e-12
INFO  B10 eps=0.001: steps 39651, retained-M rel err 1.00032e-03, F = 0.636306, F_h - F = 5.9088e-06 (tau check 5.4e-15), balance 4.3e-13
PASS  B10 decay: retained M error reproduces 1.00325e-2 and 1.00032e-3; balance closes; identity and the three bounds of (P2X.O12.20) hold for every mode; F_h - F reproduces 5.888e-5 and 5.909e-6 (ratio near 10)  ratio 9.966
ALL PASS
```

#### B.5.5 Verification record of R24, start of the release quadrature (Section 5.3.7)

Independent adversarial check (second reviewer). Every key step was re-derived without the author's algebra or helper code, and checked by a separate script `<work folder>/open_maths/R24_verify.py` (own Sturm zero-count eigen-solver with bisection and Brent polishing, own analytic mode integrals checked against quadrature, own Gauss-Legendre nodes, mpmath references). Layer data are SYNTHETIC and dimensionless: L1 is the author's two-layer case; L2 has five layers with $D^* = (1, 10^2, 10^{-2}, 10^{-4}, 10^{-1})$; L3 has a slow kernel, $D^* = (10^{-4}, 1, 1, 1, 1)$; L4 has two barrier layers, $D^* = (1, 10^{-6}, 1, 10^{-6}, 1)$; 1500 eigenpairs each. Temperature histories: constant, heating $e^{5s}$, cooling $e^{-5s}$, oscillating $1 + 0.9\sin 30s$, and a hot, cool, hot staircase; $\lambda$ from $10^{-6}$ to $10^4$; edge times $t = t_0$, $t < t_B$ and $t \gg t_0$.

Verdict: the main results hold within their stated scope; no counterexample was found. Four defects of scope or labelling were fixed in the text above, and one comment in the repository script was corrected.

Fixes made:

- F1 (scope error). R24-A1 invoked E1 to E5 of Section 5.3.6, and E4 admits a Robin surface. The energy identity (P22.89), the totals (P22.91) and the image form (P22.84) all use $c(R) = 0$. With a Robin surface $-D^*\partial_r\hat c = h\hat c$ the identity becomes $\langle f, (-L^*)^{-1}f\rangle = \int_0^R Q_f^2/(r^2D^*)\,dr + Q_f(R)^2/(R^2h)$, so the Dirichlet totals are too small and $H_N$ goes negative. Check V14 (homogeneous sphere, $hR/D^* = 2$): $\sum e_n^2/\Lambda_n = 0.0777778 = 1/45 + 1/(9h)$ against the Dirichlet value $1/45 = 0.0222222$. Fixed: R24-A1 now requires $c(R) = 0$, and a Limitations bullet records the exclusion.
- F2 (edge times). The split (P22.82) needs $t_0 \le t$, but rules 1 and 2 set $t_0 = t_B$ or $t_0 = t_1$ without reference to the reported time. Fixed: $t_0 = \min(t_B, t)$ and $t_0 = \min(t_1, t)$; for $t \le t_B$ in rule 1 the boundary term also comes from the image form, and the total error is then at most $\pi^{3/2}u(t)^{3/2}e^{-1/u(t)}$ (V13, $\lambda = 1$ and 500).
- F3 (label upgrade, with proof). (P22.84) was labelled DEMONSTRATED NUMERICALLY. A complete argument exists, so it, (P22.85) and the bound in (P22.86) are now PROVED. Take $R = D^* = 1$. (i) The modal fraction $F(u) = 1 - \sum_n(6/n^2\pi^2)e^{-n^2\pi^2u}$ has non-negative terms, so its transform may be taken term by term (Tonelli): $\bar F(p) = 1/p - \sum_n 6/(n^2\pi^2(p + n^2\pi^2)) = \sum_n 6/(p(p + n^2\pi^2))$, using $\sum_n 6/(n^2\pi^2) = 1$. The Mittag-Leffler expansion $\coth z = 1/z + \sum_n 2z/(z^2 + n^2\pi^2)$ at $z = \sqrt p$ gives $\bar F(p) = 3(\sqrt p\coth\sqrt p - 1)/p^2$ (V2b, V2c). (ii) For the image side, $\partial_a[2\sqrt t\,\mathrm{ierfc}(a/2\sqrt t)] = -\mathrm{erfc}(a/2\sqrt t)$ (V2e), and the left side tends to 0 as $a \to \infty$, so the standard pair $\mathcal L\{\mathrm{erfc}(a/2\sqrt t)\} = e^{-a\sqrt p}/p$ integrated over $a$ gives $\mathcal L\{2\sqrt t\,\mathrm{ierfc}(a/2\sqrt t)\} = e^{-a\sqrt p}/p^{3/2}$ (V2d checks this pair by quadrature). With $a = 2k$, together with $\mathcal L\{6\sqrt{u/\pi}\} = 3p^{-3/2}$ and $\mathcal L\{3u\} = 3p^{-2}$, and term by term (all image terms are non-negative), the right side $G$ of (P22.84) has $\bar G(p) = 3p^{-3/2}(1 + 2\sum_k e^{-2k\sqrt p}) - 3p^{-2} = 3p^{-3/2}\coth\sqrt p - 3p^{-2} = \bar F(p)$ (V2a). (iii) $F$ and $G$ are continuous on $[0, \infty)$ (Weierstrass test with the bound of Step 4) and of polynomial growth, so Lerch's theorem gives $F = G$. The argument rests only on the standard erfc transform pair and the partial-fraction expansion of $\coth$, and both are checked numerically. V2f confirms the identity to $2.7\times10^{-45}$ on $u \in [10^{-4}, 3]$.
- F4 (labelling of the ramp start and of the total). The summary paragraph, rule 1 and the status of (P22.98) applied the $9.4\times10^{-20}$ start bound and the label PROVED to the ramp form (P22.87) as well. That form adds a 16-point Gauss-Legendre error with no proved bound, and its smoothness also needs $\alpha$ smooth on $[0, t_B]$. Fixed: the text now restricts the proved start bound to constant $\alpha$, labels the ramp quadrature error DEMONSTRATED NUMERICALLY, and states (P22.98) as PROVED only when the start piece and every graded piece have constant $\alpha$, with the factor $e^{-\lambda s^*}$ noted for pieces after cooling.
- F5 (repository script). The header comment of check C2 in `scripts/derivation_checks/sec5_r24_quadrature_start.py` cited (P22.86), (P22.87) for the image form and its remainder; corrected to (P22.84), (P22.85). No code changed; the script still gives 25/25 PASS.

Confirmed without change (independent derivation and check):

- (P22.81) for arbitrary $\hat M(\tau(s))$ (V1, SymPy residual 0).
- The remainder bound (P22.85), its value $9.35\times10^{-20}$ at $u_B = 1/40$, the corollary $F \le 6\sqrt{u/\pi}$ and the threshold $\pi/36$ (V3; largest remainder/bound 0.607 on $[10^{-3}, 100]$). The closed form (P22.86) for $\lambda$ from $10^{-6}$ to $10^4$ (V4).
- The energy identity and Bessel bounds (P22.88) to (P22.91) for all four layer sets: closed-form totals equal quadrature to $5\times10^{-16}$, $P_N, H_N \ge 0$ for every tested $N$, true tails below $\bar W_N$ and $\bar V_N$ (largest ratios 0.984 and 0.996), $\sum\lvert w_n\rvert \le (R/R_1)^{3/2}$ (V6). Sphere tails (P22.92) including $N = 1$ (V5).
- (P22.93) in 576 combinations of layer set, $N$, $t_0$ ($10^{-9}$ to 10), $\alpha_0$ ($10^{-3}$ to 30) and $\lambda$ ($10^{-4}$ to $10^3$); largest ratio 0.715 (V7).
- The monotone release for $c_0 \ge 0$ and the bracket (P22.94) under heating, cooling and oscillating $\alpha$, including the barrier case L4; largest ratio 0.996 (V8a, V8b).
- (P22.95) including $t = t_0$, $\Lambda_\mathrm{cov}\tau(t_0) = 0.5$ and $\lambda = 10^3$; largest ratio 0.216 (V9).
- (P22.96) and (P22.97): ellipse vertex, Gauss-Legendre exactness and the constant $5.45\times10^{-12}$; on 28 graded pieces with sign-changing five-layer weights, a cooling step ($s^* < 0$) and $\lambda = 20$ the largest error/bound is $9.3\times10^{-7}$ (V10).
- The numbers 161, 744, 3451, the ratio 109.8 and its limit 120, $e^{-40}W_{12} = 2.1\times10^{-19}$ and the cold-start values of Step 8 (V5c, V11, V12).

Remarks (no change needed):

- (P22.94) is valid as written, and it also holds with $\delta = \lambda M_0\min(t_0\bar W_N, \bar V_N/\alpha_-)$ in place of $2\delta$: from $J^{(N)}(\alpha_-) - \delta \le I_0 \le J^{(N)}(\alpha_+) + \delta$ the distance to the midpoint is at most half the width plus $\delta$ (V8b, largest ratio 0.998 with $\delta$).
- (P22.93) also holds for variable $\alpha$ with $\alpha_0$ replaced by $\alpha_-$ when each $E_n$ is integrated exactly, since $E_n \le \int_0^{t_0}e^{-\alpha_-\Lambda_n^*s}ds$ (V8c). The bracket is a way to avoid per-mode quadrature.
- With extreme diffusivity ratios the eigenpair accuracy, more than the $2.2\times10^{-16}$ rounding of Step 5, sets the floor of $H_N$. In L4 ($D^*$ ratio $10^{-6}$) this solver reaches a surface residual of $6.9\times10^{-10}$, and the truncated rate $d\hat M_\mathrm{rel}/d\tau$ at $\tau = 40/\Lambda_{1500}^*$ comes out at $-1.3\times10^{-10}$ of the summed term size, consistent with that floor (V6g). The Limitations bullet on the accuracy of the eigenpairs covers this.

Script output (`<work folder>/open_maths/R24_verify.py`, run with `.venv/Scripts/python.exe -I -X utf8 R24_verify.py`):

```text
PASS  V1 (P22.35) with Mhat_rel = M0 - Mhat equals M0 - M - M_dec for arbitrary h(s)  residual 0
PASS  V2a coth z = 1 + 2 sum_k exp(-2kz): geometric sum x/(1-x) with x = e^{-2z} < 1 (SymPy)
PASS  V2b sqrt(p)coth(sqrt p) - 1 = sum 2p/(p + n^2 pi^2), p in [0.01, 1000]  max rel diff 9.2e-40
PASS  V2c Laplace transform of the modal F equals 3(sqrt p coth sqrt p - 1)/p^2  max rel diff 7.0e-40
PASS  V2d L{2 sqrt(t) ierfc(a/(2 sqrt t))} = exp(-a sqrt p)/p^{3/2} (quadrature, 3 cases)  max rel diff 3.4e-26
PASS  V2e d/da[2 sqrt t ierfc(a/(2 sqrt t))] = -erfc(a/(2 sqrt t)) and ierfc' = -erfc (SymPy)
PASS  V2f image form equals modal series for u in [1e-4, 3] (45-digit arithmetic)  max abs diff 2.7e-45
PASS  V3a 0 <= rho_B(u) <= pi^{3/2} u^{3/2} e^{-1/u} for u in [1e-3, 100] (rho_B summed directly)  max ratio 0.607
PASS  V3b F(u) <= 6 sqrt(u/pi) for u in [1e-3, 100]
PASS  V3c bound at u_B = 1/40 equals 9.35e-20 and the bound is increasing (d log/du = 3/(2u) + 1/u^2 > 0)  9.3509e-20
PASS  V3d pi^{3/2} sqrt(u) e^{-1/u} <= 3 at u = pi/36 (max over (0, pi/36])  1.74e-05
PASS  V4a (P2X.R24.6) closed form equals the quadrature, lam in {1e-6, 1, 37, 1e4}  max rel diff 6.8e-17
PASS  V4b (P2X.R24.6) error bound holds against the exact start piece  max error/bound 0.014
PASS  V5a sphere W_N <= 6/(pi^2 N), V_N <= 2/(pi^4 N^3) for N = 1, 2, 12, 100, 1000
PASS  V5b sum w_n/Lambda_n = 1/15 (R = D* = 1)
PASS  V5c e^{-40} W_12 = 2.1e-19 and (N+1)^2 pi^2 >= 1600 first at N = 12  e^-40 W_12 = 2.065e-19
PASS  V6a L1 two layers R1=0.5, D=(1, 0.25): 1500 eigenvalues isolated by zero count, u(R) residual small, ordered  max |u(R)|/amp 6.8e-13, Lambda_1 2.63, Lambda_1500 9.87e+06
PASS  V6b L1 two layers R1=0.5, D=(1, 0.25): analytic mode integrals equal quadrature (modes 1, 7, 50)  max rel diff 4.1e-16
PASS  V6c L1 two layers R1=0.5, D=(1, 0.25): closed-form totals (P2X.R24.11) equal quadrature  rel diffs 0.0e+00 1.6e-16
PASS  V6d L1 two layers R1=0.5, D=(1, 0.25): P_N, H_N >= 0 and sum_{N<n<=Nref}|w_n| <= W_bar_N, sum|w_n|/Lambda_n <= V_bar_N  max ratios 0.984, 0.996
PASS  V6e L1 two layers R1=0.5, D=(1, 0.25): 1 - sum w_n within W_bar_Nref, sum|w| <= (R/R1)^{3/2}  1 - sum w -4.86e-07, W_bar_Nref 9.92e-04, sum|w| 2.729 <= 2.828, sign changes 1000
PASS  V6f L1 two layers R1=0.5, D=(1, 0.25): sum w_n/Lambda_n equals (4 pi/M0) int Q_c0 Q_1/(r^2 D) within V_bar_Nref  diff 4.9e-14, V_bar_Nref 3.3e-11
PASS  V6g L1 two layers R1=0.5, D=(1, 0.25): dMhat_rel/dtau >= 0 for tau >= 40/Lambda_Nref (maximum principle), up to 7e-11 of sum|terms|  min rate/scale -3.46e-16
PASS  V6a L2 five layers, D=(1, 1e2, 1e-2, 1e-4, 1e-1): 1500 eigenvalues isolated by zero count, u(R) residual small, ordered  max |u(R)|/amp 1.2e-11, Lambda_1 0.00371, Lambda_1500 1.584e+05
PASS  V6b L2 five layers, D=(1, 1e2, 1e-2, 1e-4, 1e-1): analytic mode integrals equal quadrature (modes 1, 7, 50)  max rel diff 2.3e-12
PASS  V6c L2 five layers, D=(1, 1e2, 1e-2, 1e-4, 1e-1): closed-form totals (P2X.R24.11) equal quadrature  rel diffs 1.8e-16 1.2e-16
PASS  V6d L2 five layers, D=(1, 1e2, 1e-2, 1e-4, 1e-1): P_N, H_N >= 0 and sum_{N<n<=Nref}|w_n| <= W_bar_N, sum|w_n|/Lambda_n <= V_bar_N  max ratios 0.266, 0.932
PASS  V6e L2 five layers, D=(1, 1e2, 1e-2, 1e-4, 1e-1): 1 - sum w_n within W_bar_Nref, sum|w| <= (R/R1)^{3/2}  1 - sum w 4.06e-09, W_bar_Nref 1.03e-02, sum|w| 1.456 <= 2.828, sign changes 1437
PASS  V6f L2 five layers, D=(1, 1e2, 1e-2, 1e-4, 1e-1): sum w_n/Lambda_n equals (4 pi/M0) int Q_c0 Q_1/(r^2 D) within V_bar_Nref  diff 1.9e-09, V_bar_Nref 1.0e-08
PASS  V6g L2 five layers, D=(1, 1e2, 1e-2, 1e-4, 1e-1): dMhat_rel/dtau >= 0 for tau >= 40/Lambda_Nref (maximum principle), up to 1e-09 of sum|terms|  min rate/scale -3.14e-13
PASS  V6a L3 slow kernel, D=(1e-4, 1, 1, 1, 1): 1500 eigenvalues isolated by zero count, u(R) residual small, ordered  max |u(R)|/amp 6.3e-13, Lambda_1 0.003947, Lambda_1500 8706
PASS  V6b L3 slow kernel, D=(1e-4, 1, 1, 1, 1): analytic mode integrals equal quadrature (modes 1, 7, 50)  max rel diff 6.7e-16
PASS  V6c L3 slow kernel, D=(1e-4, 1, 1, 1, 1): closed-form totals (P2X.R24.11) equal quadrature  rel diffs 0.0e+00 0.0e+00
PASS  V6d L3 slow kernel, D=(1e-4, 1, 1, 1, 1): P_N, H_N >= 0 and sum_{N<n<=Nref}|w_n| <= W_bar_N, sum|w_n|/Lambda_n <= V_bar_N  max ratios 0.302, 0.988
PASS  V6e L3 slow kernel, D=(1e-4, 1, 1, 1, 1): 1 - sum w_n within W_bar_Nref, sum|w| <= (R/R1)^{3/2}  1 - sum w -9.92e-06, W_bar_Nref 8.10e-03, sum|w| 1.096 <= 2.828, sign changes 15
PASS  V6f L3 slow kernel, D=(1e-4, 1, 1, 1, 1): sum w_n/Lambda_n equals (4 pi/M0) int Q_c0 Q_1/(r^2 D) within V_bar_Nref  diff 1.3e-09, V_bar_Nref 3.1e-07
PASS  V6g L3 slow kernel, D=(1e-4, 1, 1, 1, 1): dMhat_rel/dtau >= 0 for tau >= 40/Lambda_Nref (maximum principle), up to 6e-11 of sum|terms|  min rate/scale 8.28e-07
PASS  V6a L4 extreme, D=(1, 1e-6, 1, 1e-6, 1): 1500 eigenvalues isolated by zero count, u(R) residual small, ordered  max |u(R)|/amp 6.9e-10, Lambda_1 5.195e-05, Lambda_1500 977.1
PASS  V6b L4 extreme, D=(1, 1e-6, 1, 1e-6, 1): analytic mode integrals equal quadrature (modes 1, 7, 50)  max rel diff 8.4e-12
PASS  V6c L4 extreme, D=(1, 1e-6, 1, 1e-6, 1): closed-form totals (P2X.R24.11) equal quadrature  rel diffs 2.2e-16 4.5e-16
PASS  V6d L4 extreme, D=(1, 1e-6, 1, 1e-6, 1): P_N, H_N >= 0 and sum_{N<n<=Nref}|w_n| <= W_bar_N, sum|w_n|/Lambda_n <= V_bar_N  max ratios 0.500, 0.978
PASS  V6e L4 extreme, D=(1, 1e-6, 1, 1e-6, 1): 1 - sum w_n within W_bar_Nref, sum|w| <= (R/R1)^{3/2}  1 - sum w -4.77e-08, W_bar_Nref 1.41e-02, sum|w| 2.876 <= 3.953, sign changes 1496
PASS  V6f L4 extreme, D=(1, 1e-6, 1, 1e-6, 1): sum w_n/Lambda_n equals (4 pi/M0) int Q_c0 Q_1/(r^2 D) within V_bar_Nref  diff 2.4e-06, V_bar_Nref 2.6e-05
PASS  V6g L4 extreme, D=(1, 1e-6, 1, 1e-6, 1): dMhat_rel/dtau >= 0 for tau >= 40/Lambda_Nref (maximum principle), up to 7e-08 of sum|terms|  min rate/scale -1.34e-10
PASS  V7 L1 two layers R1=0.5, D=(1, 0.25): |I0 - I0^(N)| <= lam M0 min(t0 W_bar, V_bar/alpha0), 144 combinations  max ratio 0.370
PASS  V7 L2 five layers, D=(1, 1e2, 1e-2, 1e-4, 1e-1): |I0 - I0^(N)| <= lam M0 min(t0 W_bar, V_bar/alpha0), 144 combinations  max ratio 0.062
PASS  V7 L3 slow kernel, D=(1e-4, 1, 1, 1, 1): |I0 - I0^(N)| <= lam M0 min(t0 W_bar, V_bar/alpha0), 144 combinations  max ratio 0.715
PASS  V7 L4 extreme, D=(1, 1e-6, 1, 1e-6, 1): |I0 - I0^(N)| <= lam M0 min(t0 W_bar, V_bar/alpha0), 144 combinations  max ratio 0.021
PASS  V8a L1 two layers R1=0.5, D=(1, 0.25): I0 lies in [J(alpha_-), J(alpha_+)] for heating, cooling and oscillating alpha
PASS  V8b L1 two layers R1=0.5, D=(1, 0.25): bracket (P2X.R24.14) holds; it also holds with delta in place of 2 delta  max err/bound 0.996 (with 2 delta), 0.998 (with delta)
PASS  V8a L2 five layers, D=(1, 1e2, 1e-2, 1e-4, 1e-1): I0 lies in [J(alpha_-), J(alpha_+)] for heating, cooling and oscillating alpha
PASS  V8b L2 five layers, D=(1, 1e2, 1e-2, 1e-4, 1e-1): bracket (P2X.R24.14) holds; it also holds with delta in place of 2 delta  max err/bound 0.037 (with 2 delta), 0.071 (with delta)
PASS  V8a L4 extreme, D=(1, 1e-6, 1, 1e-6, 1): I0 lies in [J(alpha_-), J(alpha_+)] for heating, cooling and oscillating alpha
PASS  V8b L4 extreme, D=(1, 1e-6, 1, 1e-6, 1): bracket (P2X.R24.14) holds; it also holds with delta in place of 2 delta  max err/bound 0.001 (with 2 delta), 0.002 (with delta)
PASS  V8c L2: modal start with exact variable-alpha E_n obeys (P2X.R24.13) with alpha_- (no bracket needed)  max ratio 0.030
PASS  V9 L1 two layers R1=0.5, D=(1, 0.25): |truncation of boundary term + I1| <= e^{-lam t0} e^{-Lambda_cov tau(t0)} W_bar_N  max ratio 0.216
PASS  V9 L2 five layers, D=(1, 1e2, 1e-2, 1e-4, 1e-1): |truncation of boundary term + I1| <= e^{-lam t0} e^{-Lambda_cov tau(t0)} W_bar_N  max ratio 0.014
PASS  V9 L3 slow kernel, D=(1e-4, 1, 1, 1, 1): |truncation of boundary term + I1| <= e^{-lam t0} e^{-Lambda_cov tau(t0)} W_bar_N  max ratio 0.051
PASS  V9 L4 extreme, D=(1, 1e-6, 1, 1e-6, 1): |truncation of boundary term + I1| <= e^{-lam t0} e^{-Lambda_cov tau(t0)} W_bar_N  max ratio 0.002
PASS  V10a GL8: weights positive, sum 2, exact on x^14, inexact on x^16
PASS  V10b rho(q=2) = 3 + 2 sqrt 2 and 8 rho^{-15}/(rho-1) = 5.45e-12, half 2.72e-12  5.4452e-12
PASS  V10c Bernstein ellipse with foci a, b and parameter (sqrt q + 1)/(sqrt q - 1) has left vertex s*
PASS  V10d L2 weights (40 modes, sign changing), heating, cooling (s* < 0), lam = 1 and 20: GL8 error <= (P2X.R24.17) on every graded piece  28 pieces, max error/bound 9.34e-07
PASS  V11a (P2X.R24.19) gives 161, 744, 3451 modes  [161, 744, 3451]
PASS  V11b ratio t0 W_N/V_N at N = 10 is 109.8 and tends to 120  N=10: 109.75
PASS  V12a cold piece: standard-series error 3.697e-3 within [0, (1 - e^{-1}) W_100 = 3.824e-3]  error 3.6970e-03 (+ at most 2e-15), bound 3.8237e-03
PASS  V12b cold piece: direct-form error <= true start release <= (1 - e^{-1}) 6 sqrt(a/pi)  direct 1.266e-04, true 1.283e-04, bound 2.140e-04
PASS  V13 rule 1 at t < t_B (image form for boundary term and start piece): error <= rho_B bound at u(t)
PASS  V14 Robin surface (hR/D = 2): sum e_n^2/Lambda_n -> 1/45 + 1/(9h), above the Dirichlet value 1/45 of (P2X.R24.9)  modal 0.07777778, Robin value 0.07777778, Dirichlet formula 0.02222222 (H_N would go negative)
SUMMARY: 69/69 PASS
ALL PASS
```

#### B.5.6 Verification record of O11, parent and daughter (Section 5.3.8)

An independent adversarial check re-derived every key step without the author's algebra or helper code: own SymPy derivations, an own spherical two-point finite-volume operator with series face conductances, an own generalised eigen-decomposition (eigenvectors orthonormal in the volume-weighted product, so the overlap matrix is $O = \Phi_D^{\mathsf T}V\Phi_P$ and its entries are bounded by 1 in that scaling, unlike the $\phi(0) = 1$ scaling behind the quoted 4.21) and own stiff Radau reference integrations. The checking script is `<work folder>/open_maths/O11_verify.py`, kept in the work folder, outside the repository.

**Verdict.** The main results stand. Theorem 1 (PROVED within the class (P22.101)), Corollary 1 (PROVED), Corollary 2 (PROVED after the hypothesis $\theta \ne 0$ was added), Proposition 2 (exact, semi-analytical) and Theorem 3 (PROVED) were confirmed; no counterexample was found. The labels are justified. Corrections made in this file:

1. Corollary 2 and the opening paragraph: the hypothesis $\theta = Q/R_g \ne 0$ was missing. For $\theta = 0$ the diffusivities do not depend on $T$ and N2 (iii) holds on every history, so "no linear ramp satisfies N2" is false without it. The Wronskian of $\{x^2, 1, e^{-\theta x}\}$ is $-2\theta(\theta x + 1)e^{-\theta x}$, not identically zero exactly when $\theta \ne 0$ (check V4). The summary-table row now states $b\lambda_P \ne 0$ and $Q \ne 0$.
2. Step 3 of the proof of Theorem 1 asserted that the amplitudes $g_k$ are positive. A real invertible diagonal $G$ may have a negative entry. The conclusion $C > 0$ still holds because $g_P/g_D$ keeps one sign, and the text now says so.
3. Step 4, case (b): "$M$ is a constant scalar matrix" was wrong wording ($M$ is not a multiple of the identity). The argument needs only that $M$ acts on the species index and $\mathcal L^*$ acts identically on both species; reworded.
4. Daughter release balance of Section 5.3.8.5: the term $M_D(0)$ was missing and $M_{\mathrm{gen},D}$ was not defined; both added. The formula as printed held only for $M_D(0) = 0$.
5. The sentence "the integrand of $J_{nm}$ is elementary in one clock exactly when the conditions of Theorem 1 hold" claimed an unproved converse. It now states only the proved direction and says the converse is not claimed.
6. The TECDOC-978 factor 6.3 is the largest over the smallest value of a non-monotonic ratio (pseudocode Section 5.2), so the lower bound 0.23 on the a priori factor applies to a history that reaches both the minimum near 1438 $^\circ$C and 1800 $^\circ$C; this condition was added. The bound $m \ge h(q^{1/4})$ is attained by the best fixed-ratio surrogate (check V9b), so 0.23 is sharp for that class.

No change to the repository script was needed: it reruns with `SUMMARY: 24/24 PASS` and `ALL PASS`, and its quoted numbers for N4 (7.35), N5 (16.50, 4.35, 3.41, 3.35) and N7 (38 % and 6.8 %) were reproduced independently (checks N2, N5 and N6 below).

**What was tested beyond the author's checks.** Theorem 1 sufficiency with full $O$ in a two-layer particle on two Bernoulli histories (one with $\alpha$ falling to 0.08, one with $\alpha$ rising to 1.76 and $\Delta\lambda < 0$), field agreement $4.5\times10^{-13}$ (numerical). The isomeric case (P22.109) on a layered ramp, including $\lambda_D = \lambda_P$. The two-clock cascade in a two-layer particle and in a stiff case with $D_D/D_P$ up to 300, $Q_P = 150$, $Q_D = 450\ \mathrm{kJ\ mol^{-1}}$ and $\lambda_D = 2\times10^{-2}\ \mathrm{s^{-1}}$, agreement $1.5\times10^{-13}$ (a first run showed $3.3\times10^{-6}$ at $2\times10^5\ \mathrm{s}$; it was traced to the absolute tolerance of the reference integrator, since the fields there are of order $e^{-60}$, and vanished at a tighter tolerance). Theorem 3 in finite-volume form on 30 random adversarial cases (one to three layers, layer-specific activation energies up to $600\ \mathrm{kJ\ mol^{-1}}$, surrogates wrong by up to 2.5 decades with an unrelated activation energy, ramp, sinusoidal and cooling histories, single-cell spike data, decay constants with $\lambda\,t_{\mathrm{end}}$ from $10^{-3}$ to $10^2$, output times down to $10^{-6}t_{\mathrm{end}}$): no violation of (P22.118), (P22.119), (P22.120) or the inventory bound. The largest error-to-bound ratio was 0.982 for the a posteriori form and 0.248 for the a priori form, so the a posteriori bound can be nearly tight (DEMONSTRATED NUMERICALLY).

**Not re-checked independently.** The continuous single-sphere numbers of N9, the two-layer finite-volume convergence numbers of N8 and N10, and the value 4.214 of the largest off-diagonal entry of $O$ in the $\phi(0) = 1$ scaling (it agrees with (P21.19), confirmed in an earlier review). Assumption B3 is still standard theory with no textbook opened.

Output of `<work folder>/open_maths/O11_verify.py` (run with `.venv/Scripts/python.exe -I -X utf8`; integrator warnings and per-case progress lines removed):

```text
PASS  V1 (P2X.O11.5): c_k = g_k chat_k(tau) with the stated tau-equations satisfies (P2X.O11.1)
PASS  V1b (P2X.O11.4) general G: residual of (P2X.O11.1) is zero
PASS  V2 [W, diag(k1,k2)] = (k2-k1)[[0,W12],[-W21,0]]
PASS  V3 Bernoulli solution, alpha(0), tau = (ln(alpha/alpha0) + dl t)/delta, limits delta->0, dl->0, isothermal
PASS  V4 ramp: N2(iii) <=> theta r x^2 + dl = delta' e^{-theta x}; Wronskian of {x^2, 1, e^-theta x} nonzero for theta>0  Wronskian = -2*theta*(theta*x + 1)*exp(-theta*x) (vanishes identically only for theta = 0)
PASS  V5 (P2X.O11.10) solves the daughter modal ODE iff tau' = C exp(delta tau - dl t) (N2(iii))
PASS  V5b degenerate case Lambda_Dn = Lambda_Pm + delta: tau e^{-Lambda tau} term
PASS  V5c constant alpha: (P2X.O11.10) reduces to (P21.21)
PASS  V6 exp(Mt) entries of (P2X.O11.12) and the lambda_D = lambda_P limit
PASS  V7 (P2X.O11.14) solves da/dt = -(lambda_D + alpha_D Lambda) a + q for arbitrary tau_D(t), q(t)
PASS  V8 int b lP e^{-lP s} b(1-e^{-lP s}) = b^2/2 (1-e^{-lP t})^2; parent term b/2 (1-e^{-lP t}); lemma b(1-e^{-lP t})
PASS  V9 h = 4 sinh^2(ln rho / 2) = h(1/rho); (D-Dt)^2/D = Dt h(Dt/D); N11 thresholds  < 0.1 on (0.8190, 1.2210), < 1 on (0.1716, 5.8284)
PASS  V9b max(h(r1),h(r2)) >= h(sqrt(r1/r2)) (2e5 samples); N12 factor 1/2 sqrt(h(6.3^(1/4)))  factor 0.2321; best symmetric split gives 0.2322 (bound attained)
PASS  V9c weighted harmonic mean of cell ratios: h(Gt/G) <= max(h(rho-), h(rho+))
PASS  N0 own FV: K symmetric, C^T K C = sum G (dC)^2, eigenvectors V-orthonormal, O full  max off-diagonal |O| = 1.000
PASS  N1 Theorem 1 sufficiency, two layers, different operators, Bernoulli history (cooling-like, dl > 0): (P2X.O11.10) = stiff integration  max rel field diff 4.5e-13; alpha(2e5)/alpha0 = 0.080; tau closed form vs quad 8.4e-15
PASS  N1 Theorem 1 sufficiency, two layers, different operators, Bernoulli history (heating-like, dl < 0): (P2X.O11.10) = stiff integration  max rel field diff 3.2e-13; alpha(2e5)/alpha0 = 1.762; tau closed form vs quad 2.3e-14
PASS  N1c ramp clock closed form tau = [T E2(theta/T)] e^(theta/Tref) / rate against adaptive quadrature  max rel diff 7.1e-15
PASS  N2 ramp + hold (Q = 300 kJ/mol): no (C, delta) fits ln alpha + dl t = ln C + delta tau  max residual 7.35 in ln alpha (author quotes 7.35)
PASS  N3 identical operators, layered, ramp + hold, lambda_P = 1e-05, lambda_D = 4e-05: (P2X.O11.12) exact  max rel field diff 1.2e-13
PASS  N3 identical operators, layered, ramp + hold, lambda_P = 2e-05, lambda_D = 2e-05: (P2X.O11.12) exact  max rel field diff 3.8e-13
PASS  N4 two-clock cascade (P2X.O11.14), layered, full O, Q_P = 200, Q_D = 350 kJ/mol, ramp + hold = stiff FV integration  max rel diff 3.5e-13 (fields and M_D); clock interpolation 0.0e+00; max off-diag O 1.00
PASS  N4b stiff adversarial cascade: D_D/D_P up to 300, Q_P = 150, Q_D = 450 kJ/mol, lambda_D = 2e-2 s^-1  max rel diff 1.5e-13; max off-diag O 0.99
PASS  N5 counterexample A: Theta_P/Theta_D = 16.50, 4.35, 3.41, 3.35 at 2e4, 4.32e4, 1e5, 3e5 s  16.5041, 4.3468, 3.4136, 3.3522
PASS  N6 counterexample B illustration: frozen coupling error 38 % at 1e5 s and 6.8 % at 3e5 s  0.3817, 0.0679
PASS  N7 Theorem 3 discrete form: a posteriori (P2X.O11.19), a priori (P2X.O11.20/21) and inventory bounds, random adversarial cases (1 to 3 layers, rho in 1e-3..1e3 and beyond, spike data, stiff decay, edge times 1e-6 Tend)  30 cases run, 0 skipped (integrator), 0 with a violation; largest error/bound ratio: a posteriori 0.982, a priori 0.248

SUMMARY: 26/26 PASS
ALL PASS
```

#### B.5.7 Verification record of O10, staircase in the stiff regime (Section 5.4.6)

Adversarial verification with an independent script, `<work folder>/open_maths/O10_verify.py` (own FV assembly, own propagators by symmetric eigendecomposition, own Radau and BDF references, own SymPy algebra; nothing is imported from `sec5_o10_stiff_staircase.py`). Every proof step of Sections 5.4.6.2 to 5.4.6.6 was re-derived by hand before the checks were written.

Steps re-derived and found correct (exact algebra unless stated): the monotonicity and degree-one homogeneity argument of Lemma 1 and the mixed bound (P22.125); the duality identity (P22.127); the energy identities (P22.128); Theorem A and Corollary A1, including $\lvert r^{-1}-1\rvert\le e^{L\delta}-1$; the left and midpoint exponents $X/\beta$ and the exact supremum (P22.131); the Riccati comparison $u\ge 2(1-e^{-Lt})/(Lm_0)\ge 2te^{-Lt}/m_0$ of Lemma 4 (a); the ratio bound of Theorem B (ii) with $s\ge t_{k^*}$ and $T-s\ge T-t_{k^*+1}$; Theorem C and the composite quadrature errors; the bound of Lemma 5 (the stated exponent $2\Lambda L + L$ is valid whatever the sign of $2\Lambda - 1$, and for $\mathbf w$ the induction over switches goes through after multiplying $u$ by $e^{4\Lambda Lhn}$, $n$ the number of switches passed); the midpoint remainder with $\lvert\ddot G_f\rvert\le(L_2+2L^2)G_f$; the harmonic-face identity in (P22.139) and the bound $e^{Lh}\sinh^2(Lh)$; the integration by parts in Theorem D, the switch-energy count $q\le e^{2L(T-t_{k+1})}$ in its boundary term, the convexity bound by $\pi$ and $\int_h^{T-h}ds/(s(T-s)) = \tfrac2T\ln\tfrac{T-h}{h}$.

Fixes (the first three were applied to the body in the first verification pass, the others in this pass):

1. Lemma 4 (a), jump form. The product $\prod_j\rho_j$ with $\rho_j = \max_iD_i^+/D_i^-$ is false when a jump lowers every diffusivity ($\rho_j<1$): the induction needs $\prod_j\rho_j\ge1$. Counterexample V4.1 ($\sup E = 1.648\times10^{-6}$ against the claimed $5.0\times10^{-8}$). Fixed by $\rho_j = \max(1,\max_iD_i^+/D_i^-)$ (V4.2, V4.3).
2. Theorem B (ii). With a one-sided ratio $\max_iD^b_i/D^a_i$ a cooling layer makes $\epsilon'$ unbounded by $\rho - 1$, and the switch factor of $\mathbf w$ at $t_{k^*+1}$ is not controlled. Fixed by the two-sided $\rho = \max_i\max(D^b_i/D^a_i, D^a_i/D^b_i)$; checked with $k^* = 1$ and $N_T - 2$ (V5.2) and with $\rho$ up to $10^6$ and mixed heating and cooling layers (V11.3, worst ratio error/bound $8.4\times10^{-5}$).
3. (P22.139). The earlier bound $(e^{Lh}-1)^2/(2-e^{Lh})$ needs $e^{Lh}<2$, which fails at $N_T = 4$ in the test problem ($e^{LT/4} = 2.413$, V7.3), so Theorem D was not covered there. Replaced by $\gamma_k\le e^{Lh}\sinh^2(Lh)$, valid for every $h$ (V7.2).
4. Lemma 4 (b), orientation of $q$. The statement used $\max_i\tilde D_{i,\mathrm{before}}/\tilde D_{i,\mathrm{after}}$ with "before" and "after" in reversed time. At a switch in reversed time the energy $\lVert\mathbf w\rVert^2_{\hat K}$ is multiplied by up to the new value over the old one, which is the inverse ratio. Counterexample V11.1: real-time staircase $\tilde D_0 = 100\,\tilde D_1$ on a three-layer mesh, $\mu h = 1/2$ for one eigenvalue, $s$ just below $t_1$; the literal $q$ equals 1 and gives $1.448\times10^{-2}$, the computed supremum is $0.533$. Fixed to $q = \prod_{t_j\in(s,T)}\max(1,\max_i\tilde D_{i,j-1}/\tilde D_{i,j})$ and checked on 160 random layer-wise jump staircases (V11.2, worst ratio 0.368). The uses of Lemma 4 (b) in Theorem B (ii) (switch from $D^b$ to $\tilde D_{k^*}$, ratio at most $\rho$) and under (H1) ($q\le e^{2L(T-s)}$) were already consistent with the corrected form, so Theorems B and D and Lemma 5 are unaffected.
5. Labels. The summary row of Theorem D said "not uniform under mesh refinement" under the label PROVED, and the opening paragraph stated the loss of uniform order 2 without a label. What is proved is the bound with a constant proportional to $\Lambda$; the growth $\Lambda\ge\Lambda_t\propto N^{1/2}$ and the reduced orders are DEMONSTRATED NUMERICALLY, and no lower bound on the error is proved. The row, the opening paragraph and the heading of Section 5.4.6.7 were reworded.
6. Smooth initial datum. The statement was CLAIMED ONLY (preliminary run, 1.766 to 1.794). V11.4 computes it: midpoint 1.768, 1.779, 1.806 and average 1.765, 1.777, 1.804; now DEMONSTRATED NUMERICALLY with these values.
7. Second synthetic case (V9.4, V9.5): three layers ($Q$ = 100, 400, 250 $\mathrm{kJ\,mol^{-1}}$, synthetic), slowest-mode initial datum, midpoint rule, $N_T$ = 64 to 8192. The errors on 15 and 30 cells per layer agree to 0.8 %, and the orders fall from 1.96 to 1.51 and rise to 1.57. The band $1.75\pm0.1$ is therefore specific to the test case; the pseudocode recommendation in Section 9.5 of the pseudocode was adjusted to require only an order above 1 in the $V$-norm.

No counterexample was found to Theorems A, B, C or D in their corrected forms, to Corollary A1 or to the sharpness statement, including extreme diffusivity ratios ($10^{12}$ across layers), stiffness $h\lVert A\rVert$ up to $4\times10^5$, intermediate times inside an interval, $\lambda>0$, fast ramps with $LT = 17.6$, and jumps in the first and last interior intervals. The labels of Section 5.4.6.9 are justified after fixes 5 and 6: PROVED for Theorems A to D (complete arguments), DEMONSTRATED NUMERICALLY for the $\Lambda_t$ growth and the observed orders, CLAIMED ONLY for the limit $7/4$, OPEN for an order above 1 uniform in the mesh stiffness.

Repository script `sec5_o10_stiff_staircase.py`: not changed. Its checks do not use any of the faulty forms (S5.2 already uses the two-sided $\rho$, S6 tests only the Lipschitz form of Lemma 4 (a), and Lemma 4 (b) and (P22.139) are not checked there), and its 22 PASS lines remain valid.

Output of `<work folder>/open_maths/O10_verify.py` (run with the project interpreter, `-I -X utf8`):

```
PASS  V0.1 d/dt ln G = th x' + (1-th) y'
PASS  V0.2 d2/dt2 ln G = th x'' + (1-th) y'' - th(1-th)(x'-y')^2
PASS  V0.3 harmonic ratio identity (P2X.O10.19)
PASS  V0.4 sup_X (e^{-X/beta}-e^{-X}) = beta^{-1/(beta-1)}(1-1/beta) (stationary point + grid)
PASS  V0.5 limits: left sup/(Lh) -> 1/(2e), midpoint sup/(Lh)^2 -> 1/(24e)  0.183939720586 vs 0.183939720586; 0.015328310049 vs 0.015328310049
PASS  V0.6 staircase exponents X/beta with beta_left=(e^{Lh}-1)/(Lh), beta_mid=sinh(Lh/2)/(Lh/2)
PASS  V0.7 (P2X.O10.21) integral
PASS  V0.8 int 1/(s(T-s)), int 1/sqrt(s(T-s)) = pi, and the Riemann sum over k=1..N_T-2 <= pi
PASS  V1.1 face lemma spectrum within [min r - 1, max r - 1] (40 random meshes, D ratios to 1e12, r to e^6)
PASS  V1.2 mixed bound, worst case over x, y (generalised singular value) <= sqrt(eps eps')
PASS  V2.1 Theorem A in operator norm: ||P - P~||_2 <= eta/2 (24 problems x 3 N_T x 3 rules, ramps, sinusoids, kinks, lambda)  informative cases (eta/2 < 1): 60, worst ratio 0.316
PASS  V2.2 Theorem A, vector form with sqrt(1 - ||z(T)||^2) at the worst singular vector
PASS  V2.3 Theorem A at intermediate times inside an interval, D x 1000 (h||A|| ~ 4.0e+05)
PASS  V2.4 Corollary A1: eta <= e^{L delta} - 1 with delta = h, h/2, h (worst-case log-slopes +-L)
PASS  V3 left-rule worst case over the stiffness equals the closed form, between Lh/(2e)(1-O(Lh)) and (e^{Lh}-1)/2
PASS  V4.1 COUNTEREXAMPLE to the literal product form with rho_j < 1 (cooling jump): sup E(t) > rho/(2t)  sup E = 1.648e-06, rho/(2t) = 5.000e-08
PASS  V4.2 with rho_j replaced by max(rho_j, 1) the bound holds
PASS  V4.3 Lemma 4(a) jump form with max(rho_j,1): 30 random three-jump histories
PASS  V4.4 Lemma 4(a) Lipschitz form on heating, cooling and fast ramps (operator sup over z0)
PASS  V4.5 Lemma 4(b) backward staircase energy bound, all rules, L T = 17.62
PASS  V5.1 Theorem B(i): aligned jump, staircase exact for every rule (40 random layer-specific jumps)
PASS  V5.2 Theorem B(ii) with two-sided rho, operator norm, k* = 1 and N_T-2 included  max err/bound 0.001
INFO  misaligned jump at T/3, left: errors 6.545e-03, 1.576e-03, 3.902e-04, 9.733e-05, 2.432e-05, 6.079e-06; orders 1.027, 1.007, 1.002, 1.000, 1.000
INFO  misaligned jump at T/3, mid : errors 3.035e-03, 7.732e-04, 1.942e-04, 4.861e-05, 1.216e-05, 3.039e-06; orders 0.986, 0.997, 0.999, 1.000, 1.000
INFO  misaligned jump at T/3, avg : errors 1.883e-04, 3.205e-05, 4.702e-06, 7.512e-07, 1.462e-07, 3.329e-08; orders 1.277, 1.385, 1.323, 1.181, 1.067
PASS  V5.3 misaligned jump: last-pair order within 0.1 of 1 for left, midpoint and average (order 1 attained)
PASS  V6.1 Theorem C (P2X.O10.14) for three alpha histories (growth, oscillation, decay)
PASS  V6.2 Theorem C average rule exact (< 1e-11)
PASS  V7.1 midpoint defect (P2X.O10.18) and average defect (P2X.O10.19) where e^{Lh} < 2
PASS  V7.2 replacement average bound gamma <= e^{Lh} sinh^2(Lh), valid for every h  9 of 18 cases have e^{Lh} >= 2 where (P2X.O10.19) gives no bound; max gamma/new 0.082
PASS  V7.3 test problem: e^{L h} >= 2 at N_T = 4 (so (P2X.O10.19) alone does not cover N_T >= 4)  L = 2.4471e-04 1/s, L T = 3.5238, e^(LT/4) = 2.413
PASS  V8 Lemma 5 exact-solution bound with Lambda replaced by a lower estimate 1.79 (L T = 17.62)
PASS  V9.0 reference Radau vs BDF (rtol 1e-13)  3.5e-14
INFO  left h||A|| 1138, 569, 285, 142; V-norm orders 1.001, 1.001, 1.000; F orders 1.001, 1.000, 1.000
INFO  mid  h||A|| 1138, 569, 285, 142; V-norm orders 1.769, 1.779, 1.806; F orders 2.125, 2.073, 2.040
INFO  avg  h||A|| 1138, 569, 285, 142; V-norm orders 1.764, 1.776, 1.804; F orders 1.977, 1.988, 1.994
PASS  V9.1 left: V-norm order 1.00 +- 0.01
PASS  V9.2 midpoint and average: V-norm orders in [1.74, 1.82] for h||A|| from 1138 to 142 (author: 1.76 to 1.81)
PASS  V9.3 midpoint and average: F(4 h) orders in [1.95, 2.15]
INFO  second data set, 15 cells/layer, midpoint: h||A|| 44750, 22375, 11187, 5594, 2797, 1398, 699, 350
        errors 2.72e-06, 7.01e-07, 1.81e-07, 4.72e-08, 1.31e-08, 4.10e-09, 1.44e-09, 4.87e-10; V-norm orders 1.957, 1.958, 1.934, 1.851, 1.674, 1.508, 1.566
INFO  second data set, 30 cells/layer, midpoint: h||A|| 180457, 90229, 45114, 22557, 11279, 5639, 2820, 1410
        errors 2.72e-06, 6.99e-07, 1.80e-07, 4.69e-08, 1.30e-08, 4.08e-09, 1.43e-09, 4.83e-10; V-norm orders 1.959, 1.961, 1.938, 1.851, 1.670, 1.511, 1.567
PASS  V9.4 second data set: staircase error is mesh-converged (15 vs 30 cells per layer agree to 2 %)  max relative difference 0.008
PASS  V9.5 second data set: midpoint V-norm order leaves the band 1.75 +- 0.1 (minimum below 1.65)  minimum order 1.511
INFO  Lambda_t: 3.627, 5.056, 7.097, 9.999, 14.115; ratios 1.394, 1.404, 1.409, 1.412
PASS  V10 Lambda_t matches the author's values (3.63, 5.06, 7.10, 10.00, 14.12) to 0.5 % and ratios tend to sqrt 2
PASS  V11.1 COUNTEREXAMPLE to Lemma 4 (b) with q from D~_before/D~_after in reversed time  sup E_w = 5.3277e-01, literal q/(2(T-s)) = 1.4482e-02, corrected q/(2(T-s)) = 1.4482e+00
PASS  V11.2 corrected Lemma 4 (b): sup_phi ||w(s)||^2_K <= q/(2(T-s)), q = prod max(1, max_i D~_{i,j-1}/D~_{i,j})  160 cases, max ratio 0.368
PASS  V11.3 Theorem B (ii), rho up to 1e6, mixed heating and cooling, k* = 1 and N_T - 2, operator norm  max err/bound 8.363e-05
INFO  smooth datum, mid: errors 6.79e-06, 1.99e-06, 5.81e-07, 1.66e-07; V-norm orders 1.768, 1.779, 1.806
INFO  smooth datum, avg: errors 6.74e-06, 1.98e-06, 5.79e-07, 1.66e-07; V-norm orders 1.765, 1.777, 1.804
PASS  V11.4 smooth initial datum: midpoint and average V-norm orders in [1.70, 1.85] (author's preliminary 1.766 to 1.794)

SUMMARY 40/40 checks passed
ALL PASS
```

#### B.5.8 Citations for O13 and O14

The two documents were downloaded and their text was extracted; the quoted strings were checked again by a script (`<work folder>/open_maths/citations_check.py`, 12 PASS lines and ALL PASS). Page numbers are printed page numbers.

**O13, $N_A$ and $k_B$: VERIFIED.** The SI Brochure (BIPM, 2019), English text, title page "9th edition 2019, https://doi.org/10.59161/AUEZ1291, V4.01, June 2026". Section 2.2, p. 123, lists the defining constants, among them $k = 1.380\,649\times10^{-23}\ \mathrm{J\ K^{-1}}$ and $N_A = 6.022\,140\,76\times10^{23}\ \mathrm{mol^{-1}}$. Table 1 on p. 124 gives the same values, and the sentence above it says that the defining constants have no uncertainty. The definitions of the kelvin (p. 129) and the mole (p. 130) in Section 2.3.1 repeat them, as does Appendix 1 (pp. 194 and 196). Status: exact by definition. The product $N_Ak_B = 8.31446261815324\ \mathrm{J\ mol^{-1}\ K^{-1}}$ is the exact decimal product of the two printed values (script PASS), and `scipy.constants` agrees to $10^{-15}$ relative. Earlier versions of the 9th edition were not opened. The v1.0 documents carry an unrelated item also named O13; only the Phase 2 item is meant here.

**O14, completeness: spectral statement VERIFIED, one step NOT VERIFIED.** Lecture 20 of the MIT OpenCourseWare notes for 18.102 (Melrose, 2009), printed p. 116, Theorem 15: for a compact self-adjoint operator on a separable Hilbert space the orthogonal complement of the null space "has an orthonormal basis consisting of eigenfunctions of A" (Melrose, 2009, p. 116), with real nonzero eigenvalues that tend to zero. For the injective Green operator of Section 3.10 the null space is $\{0\}$, so the eigenfunctions form a basis of the whole space; this one-line inference is not a sentence of the source. Not in the pages read: the name "Hilbert-Schmidt theorem", the closure-of-the-range form, and the separability of $L^2(r^2dr)$ (a standard fact, CLAIMED ONLY here). NOT VERIFIED: the step from a square-integrable symmetric kernel to a compact operator. Lecture 20 does not treat it, and a second set of lecture notes that was opened gives only a continuous-kernel example without proof, so it is not recorded as a source. O14 therefore stays partly open.
