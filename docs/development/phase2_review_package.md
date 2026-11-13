# STRATO v2.0 Phase 2 review package

Date: 2026-11-06. Status: READY FOR THE USER'S REVIEW. Work stops here.

**Nothing in Phase 2 is implemented.** No Rust has been written, and no file under `src/`, `tests/` or `examples/` differs from the tag v1.0.0. Nothing has been committed. Every Phase 2 equation is DERIVED NOT IMPLEMENTED or FUTURE. Implementation starts only after the decisions of Section 7 are made.

**Note of 2026-11-13: open mathematics items worked on.** The open items O4, O10, O11 and O12, the start of the release quadrature in block R2.4 step 3 (R24) and open issue 5 of Section 4.10 of the derivation (second-order integrators, P2) were worked on 2026-11-13. Each was derived with its own check script and then checked by an independent adversarial verification; every verdict was CONFIRMED WITH FIXES. The results are now Sections 3.5.3, 4.6.1, 5.2.5, 5.3.7, 5.3.8 and 5.4.6 of the derivation, with the new equations (P21.93) to (P21.142) and (P22.60) to (P22.142), the script output in its Appendix A.2 and the verification records in its Appendix B.5. The citations for O13 and the spectral-theorem citation of O14 were read on the same day; the compactness citation of O14 is still NOT VERIFIED. The pseudocode gained notes, status rows, open items O16 and O17, decisions 15 and 16 and the list of proposed step changes in its Section 9.5; no algorithm step was changed. Since the status line above was written, the 2026-11-06 versions of the Phase 2 documents have been committed to the repository; the open-item edits listed here were added on 2026-11-13, after that commit; no file under `src/`, `tests/` or `examples/` has changed. Sections 8 and 9, row 47 of Table 3.1, row 2 of Table 2.1 (16 decisions) and items D15 and D16 of Section 7 below are updated; the other sections keep their 2026-11-06 text.

## 1 What this package covers

Phase 2.1 adds radioactive decay, a fission-product source and a full inventory account to the v1.0 finite-volume solver (Code B) and the modal series (Code C). Phase 2.2 adds Arrhenius diffusivities $D_i(T)$ and prescribed temperature histories $T(t)$. The mathematics was derived, checked independently, reconciled (B3), turned into pseudocode (B4 to B6) and reviewed as a whole (B7). This page tells you what to read, lists the final equations and their status, summarises the evidence behind the parameters, and collects the decisions that only you can make.

## 2 What to read, in which order

| Order | Document | Path | Read first | Purpose of the reading |
|---|---|---|---|---|
| 1 | Phase 2 derivations | `docs/mathematics/STRATO_Phase2_Derivations.md` | Sections 1 and 2; 3.1, 3.3.2, 3.6; 4.3, 4.4, 4.5; 5.1, 5.2.3, 5.3.6, 5.4.5; open issues 3.10, 4.10, 5.7 | check the physics, the one change to the v1.0 step, the inventory closure and the limits of the effective time |
| 2 | Phase 2 pseudocode | `docs/mathematics/STRATO_Phase2_Pseudocode.md` | Sections 2 and 3 (one run, one step side by side with v1.0), then Section 9 (classification, open items, the 16 decisions) | check that the plain-language steps match the equations and that the defaults are acceptable |
| 3 | Change document | `docs/development/v1_to_v2_changes.md` | Sections 1 to 3 (physics, assumptions, limitations), 6 and 7 (modules, inputs, outputs) | see every v1.0 to v2.0 difference in one table per topic |
| 4 | Parameter register | `data/reference/STRATO_Parameter_Register.md` and `data/*/*.csv` | Section 0 (status vocabulary), 5 (Arrhenius), 6 (decay), 7 (yields), 8 (temperature histories), 11 (gaps) | choose the data sets of decisions D1, D4 and D5 |
| 5 | Verification record | Appendices A and B of the derivations | only where a result needs its evidence | script output, check logs, B3 reconciliation, B7 notes |

Table: 2.1 Reading order.

## 3 Final equations

The table lists the equations that the implementation would use or test, with the label the derivation gives them. "exact" means exact for the stated model; a series remark means the terms are exact and the sum is truncated. "approximate" means a leading-order or modelling approximation. "numerical" means a discretisation with an observed order. The full set is (P21.1) to (P21.92) and (P22.1) to (P22.59).

| No. | Equation | Number | Status |
|---|---|---|---|
| 1 | transport with decay and source, $\partial_tc=r^{-2}\partial_r(r^2D_i\partial_rc)-\lambda c+S$ | (P21.1) | exact (model A1 to A6) |
| 2 | decay constant, $\lambda=\ln2/t_{1/2}$ | (P21.7) | exact |
| 3 | kernel fission source, $S=\dot F\,y\,\chi_1/N_A$ | (P21.8) | exact for the model; the choice of $y^C$ is approximate |
| 4 | precursor delay, $t_{99}=\ln100/\lambda_p$ | (P21.12) | exact for one precursor |
| 5 | activation product, two-step modal solution | (P21.16) | exact for constant flux; data NOT VERIFIED; FUTURE |
| 6 | parent-daughter modal coupling through $O_{nm}$ | (P21.20) | exact equations, series truncated in $n$ and $m$; FUTURE |
| 7 | modal ODE, $\dot a_n=-(\Lambda_n+\lambda)a_n+s_n$ | (P21.28) | exact |
| 8 | interval recursion for a piecewise-constant source | (P21.33) | exact (no time error) |
| 9 | release rate series, $J_R=4\pi\sum_n\Lambda_nI_na_n$ | (P21.37) | exact terms, series error $O(N^{-1})$; diagnostic only |
| 10 | balance form of the release | (P21.39) | exact identity, series error $O(N^{-3})$ worst case |
| 11 | inventory closure, $M_0+M_\mathrm{gen}=M+M_\mathrm{dec}+M_\mathrm{rel}$ | (P21.42) | exact |
| 12 | outside inventory, $M+M_\mathrm{out}=M_0e^{-\lambda t}+\int_0^tGe^{-\lambda(t-s)}ds$ | (P21.44) | exact |
| 13 | layered $R/B(t)$ with kernel source | (P21.47) | exact (series) |
| 14 | Booth sphere with source and decay, $R/B$ and $F_B$ | (P21.48) | exact (closed form and series) |
| 15 | cell average of the kernel source, straddling cell | (P21.54) | exact |
| 16 | step source: rectangle (R) or exact step average (A) | (P21.57) | (A) exact in amount; (R) approximate, $O(\Delta t)$ |
| 17 | Backward Euler with implicit decay | (P21.58) | numerical, first order in time, second order in space |
| 18 | subtraction-free pivots, $d_j=e_j+g^E_j$ | (P21.61) | exact (proof by induction) |
| 19 | positivity, $C^n\ge0$, $\bar S\ge0$ give $C^{n+1}\ge0$ | (P21.62) | exact, also in floating point |
| 20 | link to the v1.0 step, $\mathbf C^{n+1}=(1+z)^{-1}\mathbf B_{\Delta t'}\mathbf C^n$ | (P21.63) | exact discrete identity |
| 21 | fixed point of the decay-exact split scheme | (P21.68) | exact; shows a $\Delta t$-dependent steady-state bias |
| 22 | discrete closure summed over steps | (P21.72) | exact for the exact linear solve |
| 23 | outside inventory update (BE) | (P21.73) | numerical, first order |
| 24 | outside closure with $M_\mathrm{dec,out}$ | (P21.75) | exact |
| 25 | decay step limit, $\Delta t\le2\varepsilon_\lambda/(\lambda^2t_\mathrm{end})$ | (P21.77) | approximate (leading order) |
| 26 | diffusion error of the slowest mode | (P21.78) | approximate (leading order) |
| 27 | decay identity, $c_\lambda=e^{-\lambda t}c_0$ | (P21.80) | exact |
| 28 | multilayer steady state and residence time | (P21.84) | exact |
| 29 | CRP-6 cases 1a and 1b reference | (P21.91) | exact series; benchmark values numerical |
| 30 | Arrhenius sum, $D_i=\sum_mD_{0,i,m}e^{-Q_{i,m}/(R_gT)}$ | (P22.1) | empirical correlation (approximate model) |
| 31 | apparent activation energy $Q_\mathrm{eff}$ | (P22.6) | exact |
| 32 | crossover temperature $T_x$ | (P22.7) | exact |
| 33 | sensitivity to the gas constant | (P22.8) | exact; last form approximate to first order |
| 34 | Cs-in-SiC fluence factor $e^{\Phi_f/5}$ | (P22.9) | correlation as printed; fluence cutoff conversion NOT VERIFIED |
| 35 | Backward Euler with recomputed conductances | (P22.15) | numerical, first order in time |
| 36 | closure with a changing surface conductance | (P22.16) | exact |
| 37 | positivity for every $T(t)$ | (P22.17) | exact |
| 38 | time-level error of option E | (P22.18) | exact expansion to the stated order |
| 39 | step bound for fast temperature changes | (P22.20) | approximate (first-order change of $\ln D$) |
| 40 | step integral of $D$ on a linear ramp with $E_2$ | (P22.22) | exact |
| 41 | effective time $\tau=\int_0^t\alpha\,ds$ and transform | (P22.26) to (P22.31) | exact under E1 to E5 |
| 42 | release with decay in effective time | (P22.35) | exact; the integral needs quadrature |
| 43 | modal solution with source and variable $\alpha$ | (P22.36) | exact per mode; semi-analytical when quadrature is needed |
| 44 | projection at a temperature switch, $P_{mn}$ | (P22.42), (P22.43) | exact for the untruncated series |
| 45 | mode-count rule after a switch | (P22.44) | approximate (design criterion) |
| 46 | Galerkin system on a fixed basis | (P22.48), (P22.49) | numerical; not planned |
| 47 | staircase error orders 1 and 2 | (P22.56) | numerical; updated 2026-11-13: order 1 proved uniformly; order 2 uniform only with shared scaling; $V$-norm order about 1.75 observed with layer-specific $Q$ in the test case, and down to 1.51 in a second case (Section 5.4.6) |
| 48 | option A equals BE in $\tau$ for a shared $\alpha$ | (P22.57) | exact |
| 49 | AGR-1 and CRP-6 schedule definitions | (P22.58), (P22.59) | exact as input definitions; AGR-1 start temperature assumed |

Table: 3.1 Final equations and their status.

## 4 Key assumptions

Phase 2.1 (Section 3.1 of the derivations):

- A1. Spherical symmetry and Fickian diffusion with constant $D_i$ in each layer during a step.
- A2. Perfect interfaces: concentration and flux are continuous, no partition coefficient.
- A3. Absorbing outer surface $c(R,t)=0$ and a regular centre.
- A4. First-order decay with the same $\lambda$ in every layer.
- A5. The fission source sits in the kernel and is uniform there; recoil into the buffer and noble-gas precursor migration are neglected.
- A6. Released atoms never return and decay outside with the same $\lambda$.
- The cumulative yield $y^C$ treats the precursor feed as instantaneous. This is adequate for Cs-137 and Sr-90 ($t_{99}$ of 0.42 h and 0.48 h) and approximate for Kr-85 (29.8 h) and I-131 (9.2 d).

Phase 2.2 (Sections 5.0 and 5.3.6):

- The temperature is uniform across the particle, $T=T(t)$. This holds approximately in heating tests and fails in pile.
- $D_i$ depends on time only through $T(t)$; a fluence factor is frozen during a heating test.
- The effective-time solution is exact when conditions E1 to E5 hold. These are sufficient conditions; Section 5.3.6 states that E1 is also necessary, while E2 to E5 are not proven necessary. E1, one shared factor $\alpha(t)$ for every layer, fails for every TECDOC-978 multilayer set, so the general case needs the finite-volume path or the projection of Section 5.4.1.
- The ramp shape between CRP-6 schedule points is assumed linear, and the AGR-1 plans start at an assumed 30 $^\circ$C.

## 5 Parameter evidence summary

Counts are taken from the CSV files with the register's `status` column. "verified" means read from the printed page or library and re-checked; "digitised" means read from a raster figure with the uncertainty of register Section 0.2; "NOT VERIFIED" means not confirmed against a primary source.

| File | Rows | verified | digitised | NOT VERIFIED | NOT VERIFIED rows |
|---|---|---|---|---|---|
| `data/diffusivity/arrhenius.csv` | 118 | 115 | 0 | 3 | A3-18, A3-19 (Xe label inferred), T71-05 (cited source looks mis-cited) |
| `data/nuclear/decay.csv` | 16 | 16 | 0 | 0 | none |
| `data/nuclear/fission_yields.csv` | 56 | 54 | 0 | 2 | U-235 thermal yields of Ag-110 and Ag-110m |
| `data/temperature/temperature_histories.csv` | 122 | 110 | 7 | 5 | TH-007, TH-014, TH-021 (30 $^\circ$C start assumed), TH-048 (TAVA not printed), TH-117 (cycle shape not drawn) |
| `data/geometry/geometry.csv` | 149 | 149 | 0 | 0 | none |
| `data/experimental/release_measurements.csv` | 156 | 82 | 73 | 1 | E-077 (compact label 4-4-2 possibly a mislabel) |

Table: 5.1 Status of the register rows.

Further points that matter for Phase 2:

- Diffusivity sets in `arrhenius.csv`: TECDOC-978 FRG Tables A-1 to A-5 (`T978-*`), Collin et al. (2016) Table 2 as used by PARFUME (`C16-T2`), Hales et al. (2021) Table 1 as used by BISON (`H21-T1`), TECDOC-1674 Table 7.1 kernel values (`T1674-T71`) and Eq. 10.9 (`T1674-EQ109`). Cs, Sr and Ag have kernel, PyC and SiC rows in `T978-*`, `C16-T2` and `H21-T1`; the buffer is a modelling convention (`T978-BUF`, `C16-BUF`). For noble gases in SiC there are the TECDOC-978 Xe rows A3-17 (verified) and A3-18, A3-19 (nuclide label inferred, NOT VERIFIED) and one Kr row in `C16-T2` (C16-20). Iodine has kernel rows (A1-05 to A1-07, shared with Kr and Xe) and matrix rows only, with no PyC or SiC row in any set.
- The activation products Ag-110m, Cs-134 and Eu-154 are marked in `fission_yields.csv` as not to be used as a fission-yield source.
- Synthetic benchmark rows (CRP-6 cases 1 to 5 and 11; 33 rows, G-114 to G-124 in `geometry.csv` and TH-098 to TH-119 in `temperature_histories.csv`) are for code verification only. They are IAEA (2012) benchmark specifications checked against the printed pages, not data made up for STRATO.
- Not in the register at all, and needed only for parts outside the proposed v2.0 core: capture and burn-out cross sections, the Ag-110m isomeric fraction, flux histories, the kernel heavy-metal density, the energy per fission, daughter diffusivities, the conversion between fluence cutoffs, and as-run AGR-1 furnace histories.
- The gas constant differs between sources: 8.3143 in TECDOC-978, 8.3145 in TECDOC-1674 Eq. (10.5a) and in BISON as described by Collin et al. (2021), register RG-5. The Hales et al. (2021) BISON rows print no value. In `arrhenius.csv`, 8.3145 appears only in the Collin et al. (2016) and IAEA (2012) rows. CRP-6 case 1a against the printed 0.4673 gives $F=0.467337$ (8.3143), 0.467404 (SI value) and 0.467419 (8.3145); case 1b gives $1-F=4.0936\times10^{-7}$, $4.0746\times10^{-7}$ and $4.0703\times10^{-7}$ in the same order. Only 8.3143 rounds to the printed four digits. Step C2 of the pseudocode offers the SI value and 8.3143; both stay inside the test tolerances of cases V11 and V12. The value the CRP-6 participants used is NOT VERIFIED.

## 6 B7 verdict

**Verdict: pass. Blockers: none.** Reviewer model: claude-opus-5-5, 2026-11-06. B7 ran twice. The first pass corrected the three part drafts and appended a B7 note to each check log (Appendix B of the derivations). The second pass (the verdict given here) re-checked that state and corrected the pseudocode and the change document.

What B7 reported:

- All 17 section scripts exit with code 0 and print no FAIL line (251 PASS). The pseudocode scripts give 30, 55 and 32 PASS, the register script 91 PASS, and the register re-renders from the CSVs with no differing line.
- Independent spot checks with new code. Booth limits: order 2.00 to Eqs. (P21.48) and (P21.49), and $F_B$ differs from $M_\mathrm{rel}/M_\mathrm{gen}$ when $\lambda>0$. Section 4: $R/B=0.9684291531$, the residence time and the split-scheme fixed point (P21.68). Section 5: CRP-6 case 1a 0.467337 (8.3143) and 0.467419 (8.3145), case 1b $1-F=4.0703\times10^{-7}$, crossover 1772.4 K, $Q_\mathrm{eff}$ 130.10 and 438.22 $\mathrm{kJ\ mol^{-1}}$, Eq. (P22.22) at 40 digits, and the effective time with decay to $1.1\times10^{-15}$ for a shared $Q$, failing for layer-specific $Q$ as condition E1 requires. Further checks: the yield factors 86 and 183, the (P21.12) delay, (P21.84) by quadrature, the outside closure (P21.75), (P22.36) with variable $\alpha$ against an ODE solve, and the H4 worked example.
- One spot check failed on its first run and exposed a wrong sentence in the pseudocode: the ratio $D_\mathrm{SiC}/D_\mathrm{kernel}$ is not monotonic in temperature. It is $7.2\times10^{-4}$ at 1250 $^\circ$C, has a minimum of $4.2\times10^{-4}$ near 1438 $^\circ$C and reaches $2.6\times10^{-3}$ at 1800 $^\circ$C. The factor 6.3 quoted there is maximum over minimum; the text was corrected.
- Provenance: 75 code links pinned to v1.0.0 point to the intended code. Ag-110m, Cs-134 and Eu-154 are tagged in `decay.csv` as having no fission-yield source in STRATO, and their 12 yield rows say not to use them as one; capture data stay NOT VERIFIED (GAP-26). Every IMPLEMENTED claim refers to v1.0 code.
- Fixes of the second pass: pinned links for rows 1.22, 1.23, 4.2 and 4.3 of the change document; the $D_\mathrm{SiC}/D_\mathrm{kernel}$ paragraph; the gas-constant note (where 8.3145 appears, Section 5 above); cases V11, V12 and item O5 now record the values for the two constants that step C2 can select; the law behind the underflow example; the $\sqrt\tau$ start of the R2.4 release limited to a bare kernel with a uniform start; row 2.10 (released atoms are counted as the amount still undecayed, which keeps decaying by Eq. (P21.43)); row 8.12 (8.3145 is not offered by step C2). No equation, step or verification result changed.
- Left OPEN, not blocking: the gas constant of the CRP-6 participants (O5), the quadrature or balance route for $M_\mathrm{rel}$ in block R2.4, and the convergence of the joined scheme (decay, source and changing $D(T)$ together), which is CLAIMED ONLY.

What this B8 run re-checked (Section 9): every script was re-run and its output is byte-identical to the output used in Appendix A of the derivations; a fresh assembly of the derivations from the reconciled drafts reproduces the file in the repository byte for byte, with all 151 tagged display equations unchanged; the CRP-6 gas-constant values above were recomputed with independent code. B8 did not repeat the other B7 spot checks.

## 7 Decisions needed before implementation

Each decision lists the options and what the documents propose, so that the choice is yours. The pseudocode decision numbers are given in brackets.

1. **D1, nuclides first.** Section 3.3.4 recommends independent single nuclides: Cs-137, Sr-90, Kr-85 and I-131, with Ag-110m once its activation data exist. Cs-137 and Sr-90 have verified decay, yield and diffusivity data for kernel, PyC and SiC. Kr-85 and I-131 carry the precursor-delay caveat, and iodine has no PyC or SiC diffusivity in the register (Section 5 above). Which nuclides should v2.0 support?
2. **D2, cumulative or independent yields.** Section 3.3.2 uses $y^C$; $y^I$ alone would underestimate the Cs-137 and Sr-90 sources by factors of 86 and 183. Accept $y^C$ as the only option, and keep the precursor warning with factor 0.1, which has no source [5]?
3. **D3, activation products.** Ag-110m, Cs-134 and Eu-154 need cross sections, the isomeric fraction and flux histories that are NOT VERIFIED. Keep them FUTURE (as proposed), or include the equations of Section 3.3.3 without data?
4. **D4, diffusivity set and gas constant.** Default set: TECDOC-978 FRG, Collin et al. (2016) or Hales et al. (2021)? Gas constant: SI default with a `FIXED_8_3143` option (proposed), or the value of each register row [11]?
5. **D5, temperature histories.** Which histories ship as inputs: the reconstructed AGR-1 safety-test plans (30 $^\circ$C start assumed), the CRP-6 schedules (row 8a must be re-read), or both? One history kind with repeated times or two kinds [9]? Reference temperature of the effective time: highest temperature of the history (proposed) or end of the main hold [13]?
6. **D6, time integration of decay.** Backward Euler with implicit decay, Eq. (P21.58) (recommended in Section 4.4: positive, reproduces v1.0 bit for bit when $\lambda=0$ and $S=0$ in the Python transcription, steady state independent of $\Delta t$), or the exponential split of Eq. (P21.66), whose steady $R/B$ is biased by +0.15 % to +127 % in the tests? Outside inventory by the BE update (P21.73) only, or also the exponential form (P21.74) [4]?
7. **D7, modal projection under piecewise temperature.** Is block R3 (Section 5.4.1) in v2.0 scope? It needs the layer states of the shooting solution, which are private in `sturm_liouville.rs`: public accessors or a copy of the shooting code [12]? Keep the staircase (R4) behind a test-only switch or as a test helper [14]?
8. **D8, conductance time level.** Option A (step-average diffusivity, recommended in Section 5.2.3) or option E (end of step, as in the original task brief) as the default [8]?
9. **D9, source region.** Allow a single-region particle in `Config::validate`, or add a per-region source flag with kernel only as default [1]? The Booth and CRP-6 cases need one of the two.
10. **D10, source input.** Fission rate density only (proposed), or also burnup or power, which need the kernel heavy-metal density or the energy per fission [2]?
11. **D11, step average of the source.** Exact step average (A) only, or keep the rectangle rule (R) for comparison [3]?
12. **D12, step limits.** Make the decay limit (P21.77) and the diffusion limit (P21.78) hard caps, or warnings only [6]?
13. **D13, scope of the modal block R1.** Kernel-uniform source and start profile only for v2.0, with the general projection as FUTURE [7]?
14. **D14, step planner.** Accept the merged planner of pseudocode block F (growth 2.0, restart after a switch or jump, landing by halving, $\varepsilon_D=0.1$, a FIXED mode that reproduces the v1.0 grid) [10]?
15. **D15, integrator (added 2026-11-13).** PENDING, the owner decides. Backward Euler only, Backward Euler with TR-BDF2 as an option (with or without the positivity cap $h\beta_B\le1+\sqrt2$), or Backward Euler with the exponential integrator for constant-temperature segments (Section 4.6.1 of the derivation; open item O16 of the pseudocode) [15]?
16. **D16, proposed step changes (added 2026-11-13).** PENDING, the owner decides. Which of the proposed step changes of Section 9.5 of the pseudocode (from O4, O10, O11, O12, P2 and R24) should enter the pseudocode before any Rust is written [16]?

## 8 Unresolved questions

- Data: items O1, O7, O8 and O9 of the pseudocode (cross sections, isomeric fraction, flux histories, heavy-metal density, fluence cutoff conversion, CRP-6 ramp shape and row 8a, as-run AGR-1 histories).
- CRP-6: the gas constant and the decay treatment used by the participants (O5).
- Mathematics (updated 2026-11-13; labels as in the derivation):
  - Staircase (O10): order 1 proved uniformly; order 2 uniform only with shared scaling; $V$-norm order about 1.75 observed with layer-specific $Q$ in the test case, and down to 1.51 in a second case. An order above 1 that is uniform in the mesh stiffness is OPEN.
  - Truncation of the layered modal balance form (O4): rigorous bounds now exist for a kernel-uniform start and source (Bound A and Bound B, Section 3.5.3, PROVED under completeness and the classical min-max and Sturm oscillation theorems, and conservative). A bound that uses the sign changes of the weights is OPEN.
  - Completeness rests on the spectral theorem for compact self-adjoint operators (cited: Melrose, 2009, Lecture 20, Theorem 15) and on the compactness of the Green operator, for which no citation has been opened (O14).
  - Parent and daughter (O11): they share a joint effective time only under N1 and N2 of Section 5.3.8 (Theorem 1, PROVED within the class (P22.101); transformations outside the class OPEN), which real histories rarely meet; an exact two-clock cascade exists when each species passes R2.1, and its truncation bound is open.
  - Diffusion step limit with $D(T(t))$ (O12): proved only for shared scaling: below $\varepsilon_1$ for option A, below $\varepsilon_1+\tfrac12\zeta_E^2\,\mathrm{TV}(\ln\Lambda_1)$ for option E with $\zeta_E$; the present F3 value is a proved cap for option A only; with layer-specific $Q$ the non-adiabatic remainder has no proved bound.
- Accuracy (updated 2026-11-13): in block R2.4 the two routes coincide ((P22.81)); for $S = 0$ and an absorbing surface the start of the quadrature is bounded by (P22.93) or, for a bare kernel with a uniform start, by the image form (P22.86), both PROVED for constant $\alpha$ on the start piece; for varying $\alpha$ the bracket (P22.94) is PROVED when $c_0\ge0$; the image route on a ramp (P22.87) adds a 16-point Gauss-Legendre error with no proved bound (DEMONSTRATED NUMERICALLY); the layered cold start is OPEN (O17). Second-order integration was analysed (Section 4.6.1): no rational second-order scheme keeps positivity for every step over the structural class $\mathcal S$ that contains every STRATO matrix (barrier P2.T1, PROVED; for one fixed size $N\ge3$ OPEN, O16), the exponential integrator is positive for every step (PROVED), and the integrator choice is open (decision 15 of the pseudocode, O16). The proposed step changes of Section 9.5 of the pseudocode need a decision (decision 16). The joined scheme (decay, source and changing $D(T)$ together) has closure and positivity checks; its convergence is CLAIMED ONLY.
- Physics left out: temperature-dependent partition coefficients, radial temperature gradients in pile, recoil and precursor migration.
- Documentation: in the state of 2026-11-06 the documentation PDF workflow `.github/workflows/docs-pdf.yml` was untracked and built only `docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md` and `docs/mathematics/v1.0.0/STRATO_Final_Deliverable.md`; the Phase 2 documents were not listed, and the workflow was not changed in B8. Update 2026-11-13: the workflow file and both v1.0.0 documents are in the repository, and `docs/mathematics/STRATO_Phase2_Derivations.md` and `docs/mathematics/STRATO_Phase2_Pseudocode.md` were added to the workflow's list on 2026-11-13; its `[ -f "$md" ]` guard skips any listed file that is missing. The PDF build of the two Phase 2 documents has not been run.
- Rust-level claims stay open until Rust exists: bit identity with v1.0 (cases V1, VT16, VI4) is shown in Python only.

## 9 Evidence for this package

All commands were run from the repository root with `.venv/Scripts/python.exe -I -X utf8`.

| Check | Result |
|---|---|
| 17 derivation scripts `scripts/derivation_checks/sec3_*`, `sec4_*`, `sec5_*` | all exit 0; 251 PASS lines, 0 FAIL lines; output byte-identical to the run quoted in Appendix A; re-run on 2026-11-13 with the same 251 PASS and 0 FAIL, and PASS, FAIL, SUMMARY and ALL PASS lines equal to Appendix A.1 |
| 6 open-item scripts of 2026-11-13: `sec3_o4_layered_truncation_bound.py`, `sec4_p2_positive_second_order.py`, `sec5_o12_variable_D_step_limit.py`, `sec5_r24_quadrature_start.py`, `sec5_o11_parent_daughter_tau.py`, `sec5_o10_stiff_staircase.py` | all exit 0; 91, 41, 38, 25, 24 and 22 PASS lines (241 in total), 0 FAIL, each ending with ALL PASS; output line for line equal to the output quoted in the drafts and reproduced in Appendix A.2 of the derivations |
| independent verification of the six open items (2026-11-13) | verdict CONFIRMED WITH FIXES for each; verifier scripts in the work folder (227, 41, 28, 69, 26 and 40 PASS lines, each ALL PASS); records in Appendix B.5 of the derivations |
| `b4_decay_source_pseudocode.py`, `b4_phase2_pseudocode_integration.py`, `sec6_pseudocode_checks.py`, `D5_parameter_register.py` | 55, 32, 30 and 91 PASS lines, 0 FAIL; output byte-identical to the earlier B8 run |
| fresh assembly of the derivations from the reconciled drafts | byte-identical to `docs/mathematics/STRATO_Phase2_Derivations.md`; 151 tagged display equations, (P21.1) to (P21.92) and (P22.1) to (P22.59) |
| open-item integration of 2026-11-13 | 284 tagged display equations: the 151 earlier ones byte-identical and with unchanged numbers, plus (P21.93) to (P21.142) and (P22.60) to (P22.142); every tag unique |
| equation numbers of Table 3.1 | all 49 rows name tags that exist in the derivations |
| CRP-6 cases 1a and 1b for three gas constants, independent series in mpmath | 0.467337, 0.467404, 0.467419 and $4.0936\times10^{-7}$, $4.0746\times10^{-7}$, $4.0703\times10^{-7}$, as in Section 5 |
| house-style lint of both documents | 0 problems (no em dash, no forbidden construct, display math on separate lines, every table captioned, every code link pinned to v1.0.0 and inside the file) |
| Rust source against the tag | `git diff --quiet v1.0.0 -- src tests examples Cargo.toml` succeeds; on 2026-11-13 `git diff --quiet v1.0.0 -- src tests examples` succeeds, and `Cargo.toml` differs from v1.0.0 only by the committed v1.0.1 release (version and licence lines) |
| house-style lint of the four edited documents (2026-11-13) | no em dash, no local absolute path, no display maths sharing a line with text |
| register counts of Table 5.1 | read from the CSV `status` column and re-counted |

Table: 9.1 Checks run for this package.
