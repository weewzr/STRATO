# Changes from STRATO v1.0 to v2.0

> **Status (v2.0, Phase 2):** mathematics and pseudocode are drafted and independently checked, and the implementation waits for the user's review. No Rust has been written for Phase 2. Every v2.0 item below is DERIVED NOT IMPLEMENTED or FUTURE, and every "now" or "observed" number comes from a Python transcription. Every v1.0 item marked IMPLEMENTED is pinned to the tag v1.0.0. The design is in [the Phase 2 pseudocode](../mathematics/STRATO_Phase2_Pseudocode.md).

## 1 Modelling and physics changes

Class column: IMPLEMENTED (v1.0, file and lines), DERIVED NOT IMPLEMENTED (mathematics and pseudocode only), FUTURE (named, not planned for v2.0). Equation numbers (P21.x) and (P22.x) refer to Sections 3 to 5 of the Phase 2 derivation in [docs/mathematics/STRATO_Phase2_Derivations.md](../mathematics/STRATO_Phase2_Derivations.md).

| No. | Topic | v1.0 | v2.0 | Eq. | Class of the v2.0 item |
|---|---|---|---|---|---|
| 1.1 | Radioactive decay | None | First-order loss $\lambda c$, one $\lambda$ for all layers, $\lambda=\ln 2/t_{1/2}$ | (P21.1), (P21.7) | DERIVED NOT IMPLEMENTED |
| 1.2 | Fission-product source | None; inventory is the initial kernel concentration only | Volumetric source in the kernel, $S_K=y\dot F/N_A$ | (P21.8), (P21.52) | DERIVED NOT IMPLEMENTED |
| 1.3 | Yield used | Not applicable | Cumulative yield by default, fission-weighted mean for U-235 and Pu-239 | (P21.10) to (P21.12) | DERIVED NOT IMPLEMENTED |
| 1.4 | Initial state | Uniform kernel value; run stops if the initial inventory is not positive ([finite_volume.rs, lines 186 to 189](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L186-L189)) | Zero start allowed when a source is present | (P21.4) | DERIVED NOT IMPLEMENTED |
| 1.5 | Atoms outside the particle | Not tracked | Tracked; they keep decaying after release | (P21.43), (P21.73) | DERIVED NOT IMPLEMENTED |
| 1.6 | Inventory account | $M+M_\mathrm{rel}=M_0$ ([finite_volume.rs, lines 219 to 220](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L219-L220)) | $M_0+M_\mathrm{gen}=M+M_\mathrm{dec}+M_\mathrm{rel}$, plus an outside account | (P21.42), (P21.72), (P21.75) | DERIVED NOT IMPLEMENTED |
| 1.7 | Release measures | One fraction, $F=M_\mathrm{rel}/M_0$ | One fraction per phase of the history; release-to-birth ratio $R/B$ | (P21.45) | DERIVED NOT IMPLEMENTED |
| 1.8 | Diffusivity | One constant input per layer ([config.rs, line 11](https://github.com/weewzr/STRATO/blob/v1.0.0/src/config.rs#L11)) | Arrhenius law per layer, one or two branches | (P22.1) | DERIVED NOT IMPLEMENTED |
| 1.9 | Apparent activation energy | Not applicable | $Q_\mathrm{eff}$ from the branch weights; crossover temperature of two branches | (P22.6), (P22.7) | DERIVED NOT IMPLEMENTED |
| 1.10 | Temperature | One constant value, validated and recorded only ([config.rs, lines 86 to 88](https://github.com/weewzr/STRATO/blob/v1.0.0/src/config.rs#L86-L88)) | History $T(t)$: linear ramps and constant holds | (P22.21), (P22.39), (P22.58), (P22.59) | DERIVED NOT IMPLEMENTED |
| 1.11 | Face conductances | Built once from constant $D$ ([finite_volume.rs, lines 78 to 91](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L78-L91)) | Rebuilt every step from the layer diffusivities of that step | (P22.14) | DERIVED NOT IMPLEMENTED |
| 1.12 | Diffusivity inside a step | Not applicable | Step average (default) or end-of-step value, both selectable | (P22.18), (P22.22) | DERIVED NOT IMPLEMENTED |
| 1.13 | Time grid | One constant step ([finite_volume.rs, lines 190 and 191](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L190-L191)) | Fixed mode keeps this rule; planned mode lands on every event and obeys the step limits of row 4.28 | (P21.77), (P21.78), (P22.20) | DERIVED NOT IMPLEMENTED |
| 1.14 | Gas constant | Not applicable | SI value by default; $8.3143$ selectable for the CRP-6 comparison | (P22.5) | DERIVED NOT IMPLEMENTED |
| 1.15 | Semi-analytical reference | Series $F=1-\sum_nw_ne^{-\Lambda_nt}$ ([sturm_liouville.rs, lines 382 to 393](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L382-L393)) | Modal ODE with decay and source; kernel-uniform source and start profile only | (P21.28) to (P21.39) | DERIVED NOT IMPLEMENTED |
| 1.16 | Reference for $T(t)$ | Not applicable | Effective time when all layers share one scaling factor; ordered projection when $T$ is piecewise constant | (P22.24) to (P22.37), (P22.40) to (P22.43) | DERIVED NOT IMPLEMENTED |
| 1.17 | Staircase of a ramp | Not applicable | Verification tool for ramps, order 1 for the left rule; order 2 for the midpoint and average rules proved for shared scaling, and up to a factor $1+\ln(T/h)$ on a fixed mesh for layer-specific $Q$ (Theorem D); $V$-norm order about 1.75 for layer-specific $Q$ in the stiff regime (DEMONSTRATED NUMERICALLY, Section 5.4.6) | (P22.53) to (P22.57) | DERIVED NOT IMPLEMENTED |
| 1.18 | Fluence factor on one SiC branch | Not applicable | Formula approved; conversion of the fluence cutoff open | (P22.9) | DERIVED NOT IMPLEMENTED |
| 1.19 | Activation products | Not applicable | Capture source with burn-out; data not verified | (P21.13) to (P21.16) | FUTURE |
| 1.20 | Parent and daughter chains | Not applicable | Overlap matrix for a daughter with a different diffusivity | (P21.17) to (P21.22) | FUTURE |
| 1.21 | Robin outer surface | Absorbing surface $c(R,t)=0$ | Unchanged; Robin form not planned | (P22.38) | FUTURE |
| 1.22 | Interfaces, centre, surface | Perfect contact, regular centre, absorbing surface | Unchanged | (P21.2), (P21.3) | IMPLEMENTED (v1.0): series face conductance [finite_volume.rs, line 86](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L86); modal interface state [sturm_liouville.rs, lines 98 to 100](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L98-L100); zero west conductance in cell 1 [finite_volume.rs, line 172](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L172); regular start of the shooting [sturm_liouville.rs, line 134](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L134); outer conductance to $c(R)=0$ [finite_volume.rs, line 91](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L91) |
| 1.23 | Geometry and species | Five layers, one species per run | Unchanged | none | IMPLEMENTED (v1.0): five regions required [config.rs, lines 80 to 82](https://github.com/weewzr/STRATO/blob/v1.0.0/src/config.rs#L80-L82); one `species` field [config.rs, line 35](https://github.com/weewzr/STRATO/blob/v1.0.0/src/config.rs#L35) |
| 1.24 | Partition coefficients, matrix, populations | Not modelled | Not modelled; v3.0 scope | none | FUTURE |
| 1.25 | Regression with v1.0 | Delivered outputs | $\lambda=0$, $S=0$ and constant $T$ must reproduce v1.0 | (P21.63) | DERIVED NOT IMPLEMENTED |

Table: 1.1 Modelling and physics changes from v1.0 to v2.0.

Open data items behind rows 1.3, 1.18 and 1.19: capture and burn-out cross sections, the Ag-110m isomeric fraction, flux histories, the kernel heavy-metal density and the fluence cutoff conversion are NOT VERIFIED. No value for them appears in this table.

## 2 Assumptions

Each row compares the modelling assumption of v1.0 with the one planned for v2.0 and states what follows for the user. The v1.0 column follows the [README of the v1.0.0 tag](https://github.com/weewzr/STRATO/blob/v1.0.0/README.md) and the [v1.0 input check](https://github.com/weewzr/STRATO/blob/v1.0.0/src/config.rs#L80-L96). The v2.0 column follows Sections 4 and 5 of the reconciled derivation and Sections 2 to 4 of the pseudocode.

| No. | Topic | v1.0 | v2.0 | Consequence |
|---|---|---|---|---|
| 2.1 | Temperature | constant; the input value is recorded only | uniform in the particle at each instant, $T(t)$ from a history (linear or constant intervals) | holds for post-irradiation heating tests; fails in pile, where fission heating sets a radial gradient |
| 2.2 | Diffusivity law | one constant $D_i$ per layer | $D_i(T)$ by one or more Arrhenius branches per layer | an apparent activation energy $Q_\mathrm{eff}$ and a crossover temperature follow from the data |
| 2.3 | Fluence dependence | none | frozen during a heating test (Cs in SiC correlation) | no burnup or fluence evolution inside a run |
| 2.4 | Diffusivity inside a step | constant in time | constant in a step; the planner bounds $\lvert\Delta\ln D\rvert$ by $2\varepsilon_D$ | first-order time error remains; the factor 2 is a design choice |
| 2.5 | Decay | none | one nuclide, one constant $\lambda$ in all layers | no parent-daughter chain; chains are FUTURE |
| 2.6 | Feeding by precursors | none | none; a cumulative yield replaces the precursors | adequate when the precursor delay is short; otherwise approximate |
| 2.7 | Source | none | uniform in the kernel, piecewise constant in time, fission rate density as input | the burnup and power routes need data that are NOT VERIFIED |
| 2.8 | Initial state | uniform $c_0$ in the kernel, zero elsewhere, inventory must be positive | same, or all zero when a source is present | an arbitrary start profile is FUTURE |
| 2.9 | Interfaces | perfect contact, no partition coefficient | unchanged | measured partition ratios of about 0.3 to 3 are not modelled |
| 2.10 | Outer surface | absorbing, $c(R,t)=0$ | unchanged; released atoms are counted outside as the amount still undecayed (it keeps decaying, Eq. (P21.43)), with no transport | no transport outside the particle; a Robin surface is FUTURE and not planned |
| 2.11 | Number of regions | exactly five, enforced in `Config::validate` | five; a single region only if decision 1 allows it | the Booth cases V6a, V6b and V19 need that decision |
| 2.12 | Inventory account | initial, inside, released | initial plus generated equals inside plus decayed plus released, also outside | closure is a check on every step |
| 2.13 | Time integration | Backward Euler, first order | unchanged; decay enters the diagonal only | the step must satisfy the decay and diffusion limits (P21.77), (P21.78) |
| 2.14 | Analytical reference | modal series, no source, no decay | modal series with decay and a kernel-uniform source for constant $D$ | with a temperature history only special cases have a closed reference (Table 2.2) |
| 2.15 | Gas constant | not used | SI value by default; 8.3143 as an option (decision 11) | CRP-6 case 1a gives 0.467337 with 8.3143 and 0.467404 with the SI value, and only the first rounds to the printed 0.4673 (the constant used by the participants is NOT VERIFIED) |
| 2.16 | Data | synthetic values in `inputs/` | literature values in `data/` with source, page and status | the program is not yet wired to `data/`; no value enters without a printed source |

Table: 2.1 Assumptions of v1.0 and v2.0.

Table 2.2 states when an exact reference exists for a temperature history. The finite-volume path serves every input; only the reference changes.

| No. | Temperature history | Layers share one $Q$ | Reference solution | Status |
|---|---|---|---|---|
| 2.17 | constant | not needed | modal series (block R1) | DERIVED NOT IMPLEMENTED |
| 2.18 | any $T(t)$ | yes | effective time (block R2) | DERIVED NOT IMPLEMENTED |
| 2.19 | constant on each interval | no | projection between eigenbases (block R3) | DERIVED NOT IMPLEMENTED |
| 2.20 | ramp | no | staircase (block R4), a slow test tool; no general closed form is available | DERIVED NOT IMPLEMENTED |

Table: 2.2 Reference solutions for a temperature history.

The TECDOC-978 Cs data for the FRG particle fail the shared-$Q$ test for five layers (Table 5.4 of the pseudocode), so the effective-time route applies to synthetic shared-$Q$ cases and not to those data.

## 3 Limitations

Status words: REMOVED means the design has no such limit any more, REDUCED means part of it is gone, UNCHANGED means v1.0 and v2.0 agree, NEW marks a limit that v1.0 did not have. The words describe the design. None of them is true of a Rust release before the review is finished and the code is written.

| No. | v1.0 limitation | v2.0 status | Remaining limitation |
|---|---|---|---|
| 3.1 | inputs are synthetic; no literature parameters bundled | REDUCED | register of literature values exists in `data/`; the program does not read it yet; the README plans a comparison with AGR-1 release data for v2.0, and Table 8.1 contains none |
| 3.2 | temperature is constant | REMOVED | one temperature for the whole particle; as-run AGR-1 histories are not in the corpus; the CRP-6 ramp is assumed linear |
| 3.3 | diffusivity does not depend on temperature | REMOVED | no fluence or burnup evolution; layer-specific $Q$ gives no closed-form reference |
| 3.4 | no radioactive decay | REMOVED | one nuclide with one $\lambda$; chains are FUTURE |
| 3.5 | no fission-product generation | REDUCED | kernel-uniform source with a cumulative yield; cross sections, isomeric fraction and flux histories are NOT VERIFIED |
| 3.6 | all inventory starts in the kernel | REDUCED | a zero start with a source is allowed; projection of an arbitrary profile is FUTURE |
| 3.7 | perfect interfaces | UNCHANGED | no partition coefficients (v3.0 scope) |
| 3.8 | outer surface is a perfect sink | UNCHANGED | atoms outside are counted but not transported |
| 3.9 | single intact particle | UNCHANGED | no matrix, failed particles or populations (v3.0 scope) |
| 3.10 | Backward Euler is first order in time | UNCHANGED | the joined scheme (decay, source and $D(T)$ together) has not been convergence-tested; accuracy near complete release needs $\Lambda_1\Delta t\lesssim10^{-3}$ |
| 3.11 | exactly five regions | UNCHANGED | relaxed only if decision 1 is taken |
| 3.12 | verification only, no validation | UNCHANGED | agreement between two methods does not test the physics or the inputs |
| 3.13 | the exact series has a round-off floor and a validity time | UNCHANGED | the modal block R1 keeps the v1.0 mode rule and validity time; updated 2026-11-13: rigorous truncation bounds for a layered particle with a kernel-uniform start and source now exist (Bound A (P21.106) and Bound B (P21.112) of Section 3.5.3 of the derivation, PROVED under completeness and the classical min-max and Sturm oscillation theorems, so they rest on O14), but the steps of block R1 do not use them yet (proposal R1.0 of pseudocode Section 9.5); a bound that uses the sign changes of the weights is OPEN (O4) |
| 3.14 | not applicable (no step planner) | NEW | the step limit (P21.78) is derived for constant $D$; updated 2026-11-13: for a history its F3 value with $\Lambda_1$ at the highest temperature is a PROVED sufficient cap under shared scaling for option A (Section 5.2.5 of the derivation; for option E the lag $L_1$ enters and $\varepsilon_1$ is not proved for that value), while for layer-specific $Q$ the non-adiabatic remainder has no proved bound (OPEN, O12), so there it serves as a warning only; hard cap or warning is decision 6 |
| 3.15 | not applicable (no parent-daughter physics) | NEW | updated 2026-11-13 (Section 5.3.8 of the derivation): a parent and daughter with different diffusivities have a joint effective time only under N1 and N2 (Theorem 1, PROVED within the class (P22.101)); an exact two-clock cascade (P22.113), (P22.114) exists when each species passes R2.1, but its truncation bound is OPEN (O11); the finite-volume path remains the only route in the pseudocode steps |

Table: 3.1 Limitations of v1.0 and their v2.0 status.

## 4 Mathematics

Notation: $\mathbf V$ is the diagonal matrix of cell volumes $V_j$. $G_W$ and $G_E$ are the series face conductances of cell $j$. $z=\lambda\Delta t$. $\Lambda_n$ and $\phi_n$ are the layered eigenpairs of v1.0, $\kappa_n=\Lambda_n+\lambda$. $\alpha(t)$ is the temperature scaling factor and $\tau(t)=\int_0^t\alpha\,ds$ the effective time. Concentrations are in $\mathrm{mol\ m^{-3}}$ and inventories in $\mathrm{mol}$, with time in $\mathrm{s}$.

| No. | Item | v1.0 equation | v2.0 equation | v2.0 Eq. | Class of the v2.0 item |
|---|---|---|---|---|---|
| 4.1 | Governing PDE in layer $i$ | $\partial_tc=r^{-2}\partial_r\!\left(r^2D_i\partial_rc\right)+S_i$ with $S_i=0$ in the delivered runs (TRISO-FV-100) | $\partial_tc=r^{-2}\partial_r\!\left(r^2D_i\partial_rc\right)-\lambda c+S$ | (P21.1) | DERIVED NOT IMPLEMENTED |
| 4.2 | Interface conditions at $R_i$ | $c$ and $D\partial_rc$ continuous | Unchanged | (P21.2) | IMPLEMENTED (v1.0): series face conductance [finite_volume.rs, line 86](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L86); modal interface state [sturm_liouville.rs, lines 98 to 100](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L98-L100) |
| 4.3 | Centre and surface | $\partial_rc(0,t)=0$, $c(R,t)=0$ | Unchanged | (P21.3) | IMPLEMENTED (v1.0): zero west conductance in cell 1 [finite_volume.rs, line 172](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L172); regular start of the shooting [sturm_liouville.rs, line 134](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L134); outer conductance to $c(R)=0$ [finite_volume.rs, line 91](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L91) |
| 4.4 | Initial condition | $c(r,0)=c_0$ for $r<R_1$, zero elsewhere | $c(r,0)=c_{0}(r)$, may be zero | (P21.4) | DERIVED NOT IMPLEMENTED |
| 4.5 | Decay constant | none | $\lambda=\ln 2/t_{1/2}$ | (P21.7) | DERIVED NOT IMPLEMENTED |
| 4.6 | Source density | none | $S=\dot Fy\,\chi_1(r)/N_A$ and $S_K=y\dot F/N_A$ | (P21.8), (P21.52) | DERIVED NOT IMPLEMENTED |
| 4.7 | Fission-weighted yield | none | $y_\mathrm{eff}=\sum_qf_qy_q$ | (P21.10) | DERIVED NOT IMPLEMENTED |
| 4.8 | Modal ODE for amplitude $a_n$ | $a_n^\prime=-\Lambda_na_n$ (implied by TRISO-DEL-021) | $a_n^\prime=-(\Lambda_n+\lambda)a_n+s_n(t)$ | (P21.28) | DERIVED NOT IMPLEMENTED |
| 4.9 | Modal solution | $a_n=a_n(0)e^{-\Lambda_nt}$ | $a_n=a_n(0)e^{-\kappa_nt}+\int_0^te^{-\kappa_n(t-s)}s_n\,ds$ | (P21.31) | DERIVED NOT IMPLEMENTED |
| 4.10 | Modal solution, constant source | none | $a_n=a_n(0)e^{-\kappa_nt}+(s_n/\kappa_n)(1-e^{-\kappa_nt})$ | (P21.32) | DERIVED NOT IMPLEMENTED |
| 4.11 | Series for the release | $F=1-\sum_nw_ne^{-\Lambda_nt}$ (TRISO-DEL-021) | $M/M_0=\sum_nw_ne^{-\kappa_nt}$ and $M_\mathrm{rel}/M_0=\sum_nw_n\frac{\Lambda_n}{\kappa_n}\left(1-e^{-\kappa_nt}\right)$ | (P21.81) | DERIVED NOT IMPLEMENTED |
| 4.12 | Release rate without a derivative | none | $J_R=4\pi\sum_n\Lambda_nI_na_n$ from $R^2D_5\phi_n^\prime(R)=-\Lambda_nI_n$ | (P21.36), (P21.37) | DERIVED NOT IMPLEMENTED |
| 4.13 | Cell balance | $V_j\dot C_j=G_W(C_{j-1}-C_j)+G_E(C_{j+1}-C_j)+S_jV_j$ (TRISO-FV-155) | $V_j\dot C_j=G_W(C_{j-1}-C_j)+G_E(C_{j+1}-C_j)-\lambda V_jC_j+V_j\bar S_j$ | (P21.55) | DERIVED NOT IMPLEMENTED |
| 4.14 | Backward Euler row $j$ | $-G_WC_{j-1}^{n+1}+\left(\frac{V_j}{\Delta t}+G_W+G_E\right)C_j^{n+1}-G_EC_{j+1}^{n+1}=\frac{V_j}{\Delta t}C_j^n$ (TRISO-DEL-040) | same row with the new diagonal and right-hand side of rows 4.15 and 4.16 | (P21.58) | DERIVED NOT IMPLEMENTED |
| 4.15 | FV diagonal | $a_{jj}=\frac{V_j}{\Delta t}+G_W+G_E$ ([finite_volume.rs, line 178](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L178)) | $a_{jj}=V_j\left(\frac{1}{\Delta t}+\lambda\right)+G_W+G_E$ | (P21.59) | DERIVED NOT IMPLEMENTED |
| 4.16 | FV right-hand side | $b_j=\frac{V_j}{\Delta t}C_j^n$ ([finite_volume.rs, line 179](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L179)) | $b_j=\frac{V_j}{\Delta t}C_j^n+V_j\bar S_j^{n+1}$ | (P21.59) | DERIVED NOT IMPLEMENTED |
| 4.17 | Cell-averaged source | none | $\bar S_j=S_K$ in the kernel, $S_K(R_1^3-r_{j-1/2}^3)/(r_{j+1/2}^3-r_{j-1/2}^3)$ in the cell holding $R_1$, zero outside | (P21.54) | DERIVED NOT IMPLEMENTED |
| 4.18 | Source over a step | none | $\bar S_j^{n+1}=\frac{1}{\Delta t}\int_{t^n}^{t^{n+1}}\bar S_j\,dt$ (option A) | (P21.57) | DERIVED NOT IMPLEMENTED |
| 4.19 | Pivots of the solve | $d_j=e_j+g^E_j$, $e_j=v_j+\frac{g^W_je_{j-1}}{e_{j-1}+g^W_j}$ with $v_j=V_j/\Delta t$ ([finite_volume.rs, lines 132 to 160](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L132-L160)) | same recurrence with $v_j=V_j(1/\Delta t+\lambda)$ | (P21.61) | v1.0 solver IMPLEMENTED; v2.0 input DERIVED NOT IMPLEMENTED |
| 4.20 | Link to the v1.0 step | $\mathbf B_h=(\mathbf V/h+\mathbf K)^{-1}\mathbf V/h$ | $\mathbf C^{n+1}=(1+z)^{-1}\mathbf B_{\Delta t^\prime}\mathbf C^n$ with $\Delta t^\prime=\Delta t/(1+z)$ | (P21.63) | DERIVED NOT IMPLEMENTED |
| 4.21 | Positivity | $C^n\ge0\Rightarrow C^{n+1}\ge0$ | Same for every $\Delta t>0$, $\lambda\ge0$, $S\ge0$ and every temperature | (P21.62), (P22.17) | DERIVED NOT IMPLEMENTED |
| 4.22 | Inventory identity, continuous | $M+M_\mathrm{rel}=M_0$ | $M_0+M_\mathrm{gen}=M+M_\mathrm{dec}+M_\mathrm{rel}$ | (P21.42) | DERIVED NOT IMPLEMENTED |
| 4.23 | Rate form of the identity | $\dot M=-J_R$ | $\dot M=G-\lambda M-J_R$ | (P21.40) | DERIVED NOT IMPLEMENTED |
| 4.24 | Inventory step, discrete | $M^{n+1}+M_\mathrm{rel}^{n+1}=M^0$ | $M^{n+1}-M^n=\Delta t\left(\mathrm{Gen}^{n+1}-\lambda M^{n+1}-J_R^{n+1}\right)$ | (P21.70) | DERIVED NOT IMPLEMENTED |
| 4.25 | Inventory sums, discrete | $M_\mathrm{rel}^{n+1}=M_\mathrm{rel}^n+\Delta tG_RC_N^{n+1}$ | $M_0+M_\mathrm{gen}^n=M^n+M_\mathrm{dec}^n+M_\mathrm{rel}^n$ with $M_\mathrm{dec}^n=\sum_k\Delta t_k\lambda M^k$ | (P21.72) | DERIVED NOT IMPLEMENTED |
| 4.26 | Outside inventory | none | $M_\mathrm{out}^{n+1}=\frac{M_\mathrm{out}^n+\Delta tJ_R^{n+1}}{1+\lambda\Delta t}$ | (P21.73) | DERIVED NOT IMPLEMENTED |
| 4.27 | Outside closure | none | $M_0+M_\mathrm{gen}^n=M^n+M_\mathrm{out}^n+M_\mathrm{dec}^n+M_\mathrm{dec,out}^n$ | (P21.75) | DERIVED NOT IMPLEMENTED |
| 4.28 | Step limits | Explicit bound $\Delta t\le\min_jV_j/(G_W+G_E)$ for checks only (TRISO-FV-300) | Decay: $\Delta t\le2\varepsilon_\lambda/(\lambda^2t_\mathrm{end})$; diffusion: error $\tfrac12(\Lambda_1+\lambda)^2t\Delta t$ | (P21.77), (P21.78) | DERIVED NOT IMPLEMENTED |
| 4.29 | Steady state with source | none | $J_R=\mathrm{Gen}=\tfrac{4\pi}{3}R_1^3S_K$, $\tau_\mathrm{res}=\frac{R_1^2}{15D_1}+\sum_{i=2}^5\frac{R_i^2-R_{i-1}^2}{6D_i}$ | (P21.83), (P21.84) | DERIVED NOT IMPLEMENTED |
| 4.30 | Arrhenius diffusivity | $D_i$ constant input | $D_i(T)=\sum_{m=1}^{m_i}D_{0,i,m}\exp\!\left(-Q_{i,m}/(R_gT)\right)$, $m_i\in\{1,2\}$ | (P22.1) | DERIVED NOT IMPLEMENTED |
| 4.31 | Apparent activation energy | none | $Q_\mathrm{eff}=\sum_mQ_mD_m(T)/\sum_mD_m(T)$ | (P22.6) | DERIVED NOT IMPLEMENTED |
| 4.32 | Crossover temperature | none | $T_x=(Q_2-Q_1)/\left(R_g\ln(D_{0,2}/D_{0,1})\right)$ | (P22.7) | DERIVED NOT IMPLEMENTED |
| 4.33 | Cell balance with $D(T)$ | constant $G_W$, $G_E$ | $V_j\dot C_j=G_{j-1/2}(t)(C_{j-1}-C_j)-G_{j+1/2}(t)(C_j-C_{j+1})-\lambda V_jC_j+V_jS_j$ | (P22.13) | DERIVED NOT IMPLEMENTED |
| 4.34 | Series face conductance | $G=A/(\delta^-/D_a+\delta^+/D_b)$ with constant $D$ | same form with $D_a=D_a(T(t))$, $D_b=D_b(T(t))$; outer face $4\pi R^2D_5/(R-\bar r_N)$ | (P22.14) | DERIVED NOT IMPLEMENTED |
| 4.35 | Step with $D(T)$ | One matrix for all steps | Matrix rebuilt each step with $G^{(n)}$ | (P22.15) | DERIVED NOT IMPLEMENTED |
| 4.36 | Inventory step with $D(T)$ | $\sum_jV_j(C_j^{n+1}-C_j^n)=-\Delta tG_RC_N^{n+1}$ | $\sum_jV_j(C_j^{n+1}-C_j^n)=-\Delta t_nG_{N+1/2}^{(n)}C_N^{n+1}-\lambda\Delta t_n\sum_jV_jC_j^{n+1}+\Delta t_n\sum_jV_jS_j^{(n)}$ | (P22.16) | DERIVED NOT IMPLEMENTED |
| 4.37 | Ramp integral for the step average | none | $\int_{t_a}^{t_b}e^{-\theta/T}dt=\frac{1}{\dot T}\left[TE_2(\theta/T)\right]_{T_a}^{T_b}$ | (P22.22) | DERIVED NOT IMPLEMENTED |
| 4.38 | Shared scaling factor | none | $D_i(T(t))=\alpha(t)D_i^\ast$, $\alpha=\exp\!\left[-\frac{Q}{R_g}\left(\frac{1}{T}-\frac{1}{T_\mathrm{ref}}\right)\right]$ | (P22.24), (P22.25) | DERIVED NOT IMPLEMENTED |
| 4.39 | Effective time | none | $\tau(t)=\int_0^t\alpha(s)\,ds$ | (P22.26) | DERIVED NOT IMPLEMENTED |
| 4.40 | Solution in effective time | $F(t)=F_{v1}(t)$ | $c(r,t)=e^{-\lambda t}\hat c(r,\tau(t))$ with $\hat S=e^{\lambda t}S/\alpha$; for $\lambda=0$, $S=0$: $F(t)=F_{v1}(\tau(t))$ | (P22.27), (P22.30), (P22.33) | DERIVED NOT IMPLEMENTED |
| 4.41 | Modal solution with $\alpha(t)$ | $a_n=a_n(0)e^{-\Lambda_nt}$ | $a_n=a_n(0)e^{-\lambda t-\Lambda_n^\ast\tau(t)}+\int_0^te^{-\lambda(t-s)-\Lambda_n^\ast[\tau(t)-\tau(s)]}s_n\,ds$ | (P22.36) | DERIVED NOT IMPLEMENTED |
| 4.42 | Projection at a temperature switch | none | $a_m^{(k)}(t_k)=\sum_nP_{mn}^{(k)}a_n^{(k-1)}(t_k)$, $P_{mn}^{(k)}=\frac{1}{N_m^{(k)}}\int_0^Rr^2\phi_m^{(k)}\phi_n^{(k-1)}dr$ | (P22.42), (P22.43) | DERIVED NOT IMPLEMENTED |
| 4.43 | Staircase errors for a ramp | none | $e\approx C_\mathrm{L}N_T^{-1}$ (left), $e\approx C_\mathrm{M}N_T^{-2}$ (midpoint and average) for small $h\lVert B\rVert$; corrected in Section 5.4.6: in the stiff regime with layer-specific $Q$ order 2 is not uniform in the stiffness ($V$-norm order about 1.75, DEMONSTRATED NUMERICALLY) | (P22.56) | DERIVED NOT IMPLEMENTED |

Table: 4.1 Mathematics of v1.0 and v2.0 side by side.

The effective-time forms in rows 4.38 to 4.41 hold only when all five layers share one scaling factor $\alpha(t)$ (a common activation energy) and the decay constant is uniform. When the layers have different activation energies, the ratios $D_i/D_j$ change with temperature, no single effective time exists, and the finite-volume path of rows 4.33 to 4.36 is the reference. A piecewise-constant temperature keeps an exact ordered product through row 4.42.

| No. | Property | Statement | Class | Basis |
|---|---|---|---|---|
| 4.P1 | Positivity of one Backward Euler step | $C^{n+1}\ge0$ for $C^n\ge0$, $S\ge0$, any $\Delta t>0$, $\lambda\ge0$, any temperature | PROVED | Eqs. (P21.62), (P22.17); operations on nonnegative numbers only |
| 4.P2 | Inventory closure per step | Eqs. (P21.70), (P21.72) hold with a changing $G_R$ | PROVED for the exact linear solve; DEMONSTRATED NUMERICALLY at round-off ($10^{-15}$) | Pseudocode checks V9 and VI2 |
| 4.P3 | Steady state independent of $\Delta t$ | Fixed point of the unsplit step equals the semi-discrete steady state | PROVED | Eq. (P21.68) discussion |
| 4.P4 | Order in space | Second order | DEMONSTRATED NUMERICALLY | Observed orders 1.97 to 2.00 |
| 4.P5 | Order in time, joined scheme | First order | CLAIMED ONLY | Each part was shown separately; the joined run was checked for closure and positivity only |
| 4.P6 | Effective-time solution | Exact under the shared-$\alpha$ conditions | PROVED (symbolic); DEMONSTRATED NUMERICALLY | Eqs. (P22.24) to (P22.33) |
| 4.P7 | Staircase orders 1, 2, 2 | Left rule order 1; midpoint and average rules order 2 | updated 2026-11-13: order 1 PROVED uniformly in the stiffness; order 2 PROVED for shared scaling (uniformly) and, up to a factor $1+\ln(T/h)$, on a fixed mesh (constant grows like $N^{1/2}$, DEMONSTRATED NUMERICALLY); orders near 2 for $F$ DEMONSTRATED NUMERICALLY; $V$-norm order 7/4 CLAIMED ONLY; an order above 1 uniform in the mesh stiffness for layer-specific $Q$ OPEN | Eqs. (P22.54) to (P22.56), (P22.130), (P22.140); Section 5.4.6 of the derivation |
| 4.P8 | Bit-identical regression with v1.0 | Adding exactly 0.0 leaves every float unchanged | DEMONSTRATED NUMERICALLY in Python; CLAIMED ONLY for Rust | Pseudocode check VI4 |

Table: 4.2 Numerical properties tied to the equations of Table 4.1.

## 5 Pseudocode: v1.0 step and v2.0 step

Source of the v2.0 column: `docs/mathematics/STRATO_Phase2_Pseudocode.md` (block codes A to M, R1 to R4, steps such as H4.2). Equation numbers (P21.x) and (P22.x) are those of the reconciled derivation. The marks KEPT, CHANGED and NEW say how a step relates to v1.0. A step marked DESIGN rests on a choice that no equation fixes, and the review may overrule it.

Rows 1 to 9 run once per run. Rows 10 to 17 run once per time step. Rows 18 to 26 are bookkeeping and output. Rows 27 and 28 are reference solutions outside the loop.

| No. | Step | v1.0 (IMPLEMENTED) | v2.0 (planned) | Mark | Eq. or rule |
|---|---|---|---|---|---|
| 1 | Read and check the run file | [src/config.rs, lines 62 to 113](https://github.com/weewzr/STRATO/blob/v1.0.0/src/config.rs#L62-L113): five regions, positive radii and $D$, positive $T$, nonnegative $C_0$ | Block A: same checks, plus unit tags converted to SI and an error that names the field | CHANGED | (P22.10) |
| 2 | Cross-key consistency | none | A9: any $Q>0$ needs a temperature history; FIXED grid refuses events | NEW | DESIGN |
| 3 | Decay constant | none | Steps B1 to B6 of the pseudocode: $\lambda=\ln 2/t_{1/2}$ with the unit table; stable gives $\lambda=0$ | NEW | (P21.7) |
| 4 | Layer diffusivity | one constant $D$ per region, read from the file ([src/config.rs, line 11](https://github.com/weewzr/STRATO/blob/v1.0.0/src/config.rs#L11)) | C1 to C3: sum of Arrhenius branches by log-sum-exp; one branch with $Q=0$ returns $D_0$ exactly | NEW | (P22.1) |
| 5 | Temperature | `temperature_k` is checked and then unused by the solver ([src/config.rs, lines 86 to 88](https://github.com/weewzr/STRATO/blob/v1.0.0/src/config.rs#L86-L88)) | Block D: $T(t)$ from a linear table or from constant intervals | NEW | (P22.21), (P22.39) |
| 6 | Source per interval | none | E1 to E2: $S_K=y\dot F/N_A$ with the mixed cumulative yield | NEW | (P21.8), (P21.10) |
| 7 | Geometry | [src/numerical/finite_volume.rs, lines 50 to 99](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L50-L99): volumes, centroids and series conductances together | Block G: volumes, centroids, areas and distances once; conductances move to row 12; cell factor of the source added | CHANGED | (P21.54), (P22.14) |
| 8 | Initial state and start check | kernel cells hold $C_0$ ([lines 101 to 112](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L101-L112)); stop if the inventory is not positive ([lines 186 to 189](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L186-L189)) | same state; stop only if the inventory and the source are both zero (J0) | CHANGED | DESIGN |
| 9 | Time grid | `steps = ceil(end/dt)`, constant `dt` ([lines 190 and 191](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L190-L191)) | FIXED mode: same rule, bit for bit. PLANNED mode: steps land on every event within decay, diffusion and temperature caps | KEPT or NEW | (P21.77) to (P21.79), (P22.20) |
| 10 | Diffusivity used by one step | constant | H1: end-of-step value (option E) or exact step average (option A, default); Gauss-Legendre or $E_2$ closed form | NEW | (P22.22), (P22.23) |
| 11 | Source used by one step | none | H3: step-averaged source times the cell factor; `gen_rate` is the volume sum | NEW | (P21.57), (P21.54) |
| 12 | Conductances | fixed once in `build_mesh` ([lines 78 to 91](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L78-L91)) | H2: two half-cell resistances in series, rebuilt each step from that step's $D$; outer $G_R$ kept for the bookkeeping | CHANGED | (P22.14) |
| 13 | Matrix diagonal excess | `v[i] = volume / dt` ([line 178](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L178)) | `volume / dt + decay_constant * volume`; the order of terms keeps $\lambda=0$ bit-identical | CHANGED | (P21.59) |
| 14 | Right-hand side | `v[i] * old[i]` ([line 179](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L179)) | `(volume / dt) * old[i] + volume * source_cell[i]`; the enlarged diagonal must not be reused | CHANGED | (P21.59) |
| 15 | Linear solve | subtraction-free tridiagonal elimination ([lines 132 to 160](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L132-L160)) | unchanged code; the proof of positivity extends to decay and to changing conductances | KEPT | (P21.61), (P21.62), (P22.17) |
| 16 | Release rate | `mesh.outer_conductance * next[last]` ([line 210](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L210)) | `G_R * conc_new[N]` with the $G_R$ of this step | CHANGED | (P21.71), (P22.16) |
| 17 | Next step | loop over `1..=steps` ([line 208](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L208)) | loop until `t_end`; the same `dt` enters the matrix, the source average and the sums | CHANGED | DESIGN |
| 18 | Release sum | Kahan sum of `dt * release_rate` ([lines 211 to 217](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L211-L217)) | same Kahan sum, plus sums for generated and decayed amounts | KEPT and NEW | (P21.72) |
| 19 | Inventory | `inventory(mesh, c)` ([lines 114 to 120](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L114-L120)) | same function | KEPT | none |
| 20 | Atoms outside the particle | not tracked | `out_now` by Backward Euler; decayed-outside sum; exponential form optional | NEW | (P21.73), (P21.75), (P21.74) |
| 21 | Closure residual | `inv + released - initial` ([line 220](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L220)) | two residuals, inside and outside, tested at $10^{-12}$ relative to $M_0+M_\mathrm{gen}$ | CHANGED | (P21.72), (P21.75) |
| 22 | Positivity report | minimum concentration reported; the unit test tolerates $-10^{-13}$ ([line 310](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L310)) | any value below 0.0 is a failure | CHANGED | (P21.62), (P22.17) |
| 23 | Output rule | every `output_every_steps` steps and the last ([line 221](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L221)) | FIXED mode keeps this rule; PLANNED mode writes at the requested output times | CHANGED | DESIGN |
| 24 | Release fractions | one fraction, released over initial ([line 227](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L227)) | six fractions chosen by the scenario; empty when the denominator is 0 | CHANGED | (P21.45) |
| 25 | Output table | seven columns ([src/main.rs, lines 45 to 53](https://github.com/weewzr/STRATO/blob/v1.0.0/src/main.rs#L45-L53)) | wider table of Table 7.3 | CHANGED | K1 |
| 26 | End-of-run report | console lines ([src/main.rs, lines 75 to 87](https://github.com/weewzr/STRATO/blob/v1.0.0/src/main.rs#L75-L87)) | adds worst residuals, controlling step cap, floor and window counts, warnings | CHANGED | K2 |
| 27 | Exact release series | `ModalRelease` and `exact_release` ([src/analytical/sturm_liouville.rs, lines 308 to 395](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L308-L395) and [399 to 418](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L399-L418)) | R1: modal ODE with decay and a piecewise-constant source, exact in time | NEW | (P21.24) to (P21.49) |
| 28 | Reference for temperature histories | none | R2 effective time (shared scaling only), R3 projection at a temperature step, R4 staircase of a ramp (test tool) | NEW | (P22.24) to (P22.57) |

Table: 5.1 Pseudocode, v1.0 step against v2.0 step.

Four rules in the v2.0 column carry most of the risk and are worth stating once more.

| No. | Rule | Why |
|---|---|---|
| 1 | Use the same `t_old`, `t_new`, `dt` and $G_R$ in the matrix, the source average and the sums | The inventory balance then closes to round-off. A fixed first-step $G_R$ gave a residual of $1.2\times10^{-5}$ in the check run |
| 2 | Keep decay inside the implicit step (unsplit) | The unsplit step keeps the exact steady state. The decay-then-diffuse split shifts it by up to +127 % in the checked cases |
| 3 | Use the left-limit temperature (D6) on a step that ends at a jump | The right-continuous value belongs to the next segment and halves the last step before every jump |
| 4 | Add $\lambda V$ as a separate term | $V(1/\Delta t+\lambda)$ differs from $V/\Delta t$ in the last bit for 13 of 40 cells and would break the regression test V1 |

Table: 5.2 Rules of the v2.0 step that must not be changed without a new check.

Scope limits of the claims above. The v2.0 column was checked by Python transcriptions only (`b4_phase2_pseudocode_integration.py`, 32 of 32 PASS). Bit identity with v1.0 is DEMONSTRATED NUMERICALLY in Python and CLAIMED ONLY for Rust until the tests V1 and VT16 run. The joined scheme (decay, source and changing $D$ together) has been checked for closure and positivity and has not been convergence-tested, so its first-order time accuracy is CLAIMED ONLY.

## 6 Code and modules

All v1.0.0 files stay as released until the review is finished. The table lists what is planned. Every row has the status "Planned, not yet implemented".

| No. | File | New or extended | Planned content | Blocks | Status |
|---|---|---|---|---|---|
| 1 | `src/physics/decay.rs` | new | unit table, $\lambda$ and mean life, consistency checks | B | Planned, not yet implemented |
| 2 | `src/physics/diffusivity.rs` | new | branch records, log-sum-exp $D(T)$, $Q_\mathrm{eff}$, crossover, fluence factor, step average | C, H1 | Planned, not yet implemented |
| 3 | `src/physics/fission_source.rs` | new | yield choice, mixed yield, source per interval, step average, precursor warning | E, G4 | Planned, not yet implemented |
| 4 | `src/physics/inventory.rs` | new | compensated sums, outside inventory, residuals, fractions, report | J, K | Planned, not yet implemented |
| 5 | `src/thermal/temperature_history.rs` | new | linear and constant-interval histories, builders, lookups, events, slope | D, F, R2 | Planned, not yet implemented |
| 6 | `src/analytical/modal_source.rs` | new | modal ODE with decay and source on the Phase 1 eigenpairs | R1, R3 | Planned, not yet implemented |
| 7 | `src/analytical/thermal_intervals.rs` | new | effective-time test and solution, interval projection, staircase tool | R2, R3, R4 | Planned, not yet implemented |
| 8 | `src/numerical/finite_volume.rs` | extended | geometry split from conductances, step planner, step with decay and source, `run` loop | F, G, H, M | Planned, not yet implemented |
| 9 | `src/config.rs` | extended | new keys of Table 7.1, unit tags, cross-key checks | A | Planned, not yet implemented |
| 10 | `src/main.rs` | extended | wider output table, end-of-run report | K | Planned, not yet implemented |
| 11 | `src/lib.rs`, `src/analytical/mod.rs`, new `mod.rs` files | extended or new | module declarations for the files above; implied by rows 1 to 7, not named on the pseudocode page | none | Planned, not yet implemented |
| 12 | `src/analytical/sturm_liouville.rs` | extended, if decision 12 says so | public access to the layer states of the shooting solution, needed by R3 | R3 | Planned, not yet implemented (decision 12 open) |
| 13 | `tests/` | new files | regression, closure, positivity and reference cases of Section 7 of the pseudocode page; file names not fixed | V1 to V21, VT1 to VT18, VI1 to VI6 | Planned, not yet implemented |
| 14 | `examples/verify.rs` | extended | the verification cases that need a fine mesh | Section 7 | Planned, not yet implemented |

Table: 6.1 Code and modules.

The v1.0 files `src/postprocessing/comparison.rs` and `src/postprocessing/plots.rs` are not named in the pseudocode page. Whether they change is a review item.

| No. | Review item | Decision |
|---|---|---|
| 1 | Single-region particle or per-region source flag | Decision 1 |
| 2 | Source from fission rate only, or also burnup or power | Decision 2 |
| 3 | Step average (option A) as the only mode, or option R kept | Decision 3 |
| 4 | Backward Euler outside inventory only, or the exponential form too | Decision 4 |
| 5 | Precursor warning threshold | Decision 5 |
| 6 | Decay and diffusion step limits as hard caps or warnings | Decision 6 |
| 7 | Modal block limited to a kernel-uniform source and start | Decision 7 |
| 8 | Default conductance time level, option A or E | Decision 8 |
| 9 | Two history kinds or one kind with repeated times | Decision 9 |
| 10 | Merged planner of block F | Decision 10 |
| 11 | Gas constant default and option | Decision 11 |
| 12 | Eigenfunction access, accessors or copied shooting code | Decision 12 |
| 13 | Reference temperature of the effective time | Decision 13 |
| 14 | Staircase code in the library or in tests only | Decision 14 |

Table: 6.2 Decisions requested before any Rust is written.

## 7 Inputs and outputs

Every key and column below is planned. The pseudocode page fixes the quantities, units and defaults. It does not fix the TOML spelling, so the key names are proposals (DESIGN) that the review may rename. Keys that v1.0 already reads keep their meaning.

### 7.1 New and changed input keys

| No. | Key (proposed) | Unit | Default or rule | Used by | Status |
|---|---|---|---|---|---|
| 1 | `nuclide.name` | none | name found in `data/nuclear/decay.csv`, or `none` for $\lambda=0$; half-life comes from the table | B | Planned |
| 2 | `source.fissile_mix` (name, fraction) | none, fractions | fractions sum to 1 within $10^{-9}$ | E1 | Planned |
| 3 | `source.rate_history` (start, end, rate) | s, s, $\mathrm{fissions\ m^{-3}\ s^{-1}}$ | constant on each interval; rate nonnegative and finite | E2 | Planned |
| 4 | `source.precursor_half_life_s` | s | optional; gives $t_{99}$ | E1 | Planned |
| 5 | `source.precursor_warning_factor` | none | 0.1 (no source, decision 5) | E1 | Planned |
| 6 | `regions.law.branches` (`d0`, `q`) | $\mathrm{m^2\ s^{-1}}$, $\mathrm{J\ mol^{-1}}$ or $\mathrm{kJ\ mol^{-1}}$ | $D_0>0$, $Q\ge0$; replaces `diffusivity_m2_s` for an Arrhenius layer | C1 | Planned |
| 7 | `regions.law.q_unit` | tag | `J/mol` or `kJ/mol`; missing tag is an error | C1 | Planned |
| 8 | `regions.law.window` (`t_min`, `t_max`) | K or degC by tag | optional; no default window is stored | C3.5 | Planned |
| 9 | `regions.law.fluence` (branch index, value, cutoff tag) | $10^{25}\ \mathrm{m^{-2}}$ | cutoff tag must be `E>16fJ`; conversion from $E>0.18\ \mathrm{MeV}$ is NOT VERIFIED | C1.4 | Planned, data OPEN |
| 10 | `regions.diffusivity_m2_s` | $\mathrm{m^2\ s^{-1}}$ | kept for a constant layer; treated as one branch with $Q=0$ | C, A4 | Kept |
| 11 | `settings.gas_constant` | none | `SI` (default) or `FIXED_8_3143` (decision 11) | C2 | Planned |
| 12 | `settings.d_floor_m2_s` | $\mathrm{m^2\ s^{-1}}$ | positive; floor events are counted and reported | C3.6 | Planned |
| 13 | `temperature.kind` | none | `LINEAR` or `CONSTANT` (decision 9) | D | Planned |
| 14 | `temperature.times`, `temperature.values` | s or h, K or degC by tag | LINEAR: first time 0, strictly increasing; CONSTANT: start times and `end` | D1 to D4 | Planned |
| 15 | `temperature.segments` (RAMP target and rate, HOLD duration) | K or degC; K/h or degC/h; h | rate positive; alternative to row 14 | D1 | Planned |
| 16 | `temperature.triples` | degC, h, h | CRP-6 style; `-` read as 0 | D2 | Planned, ramp shape OPEN |
| 17 | `temperature_k` | K | v1.0 key; role in a v2.0 run not fixed (constant run, or one-interval history) | A | Open for review |
| 18 | `time.grid_mode` | none | `FIXED` (v1.0 rule) or `PLANNED` | F0 | Planned |
| 19 | `time.dt_s`, `time.end_time_s` | s | kept; `dt_s` is the request in FIXED mode | F0 | Kept |
| 20 | `time.output_every_steps` | steps | kept; FIXED mode only | F0, K | Kept |
| 21 | `time.output_times_s` | s | PLANNED mode; each in $(0,t_\mathrm{end}]$ | F1 | Planned |
| 22 | `time.dt_max_s`, `time.dt_first_s` | s | PLANNED mode caps; `dt_first_s` restarts after a switch or a jump | F4, F5 | Planned |
| 23 | `time.growth` | none | 2.0 | F5 | Planned |
| 24 | `time.eps_d` | none | 0.1; bound on the change of $\ln D$ per step | F5, (H22.1) | Planned |
| 25 | `time.eps_lambda` | none | $10^{-3}$; decay error target | F2 | Planned |
| 26 | `time.lambda1_per_s`, `time.eps_1` | $\mathrm{s^{-1}}$, none | optional; gives the diffusion cap or warning | F3 | Planned |
| 27 | `time.step_limit` | steps | stop and name the controlling cap if exceeded | F4 | Planned |
| 28 | `time.conductance_option` | none | `A` (step average) or `E` (end of step); decision 8 | H1 | Planned |
| 29 | `initial.kernel_concentration_mol_m3` | $\mathrm{mol\ m^{-3}}$ | kept; zero is allowed when a source exists | J0 | Changed |
| 30 | `output.outside_inventory_exponential` | none | off; optional exponential form (decision 4) | J1.4 | Planned |
| 31 | irradiation end time $t_\mathrm{irr}$ for $F_\mathrm{post}$ | s | pseudocode uses $t_\mathrm{irr}$ without saying where it comes from; taking it from the end of the source history is an assumption for review | J2 | Open for review |

Table: 7.1 New and changed input keys (names are proposals).

Data files read by v2.0, all already in the repository under `data/`.

| No. | File | Columns used | Block |
|---|---|---|---|
| 1 | `data/nuclear/decay.csv` | `nuclide`, `half_life`, `unit`, `half_life_s`, `lambda_s-1`, `stable` | B |
| 2 | `data/nuclear/fission_yields.csv` | `parent`, `product`, cumulative yield column of the chosen library (JEFF-3.1.1 in the worked example) | E1 |
| 3 | `data/diffusivity/arrhenius.csv` | `D0_m2_s`, `Q_kJ_mol`, `T_min_C`, `T_max_C`, `R_J_mol_K` | C |
| 4 | `data/temperature/temperature_histories.csv` | `T_start_C`, `T_end_C`, `rate_C_per_h`, `ramp_h`, `hold_h` | D1, D2 |

Table: 7.2 Data files read by v2.0.

The pseudocode does not say whether the solver reads these files directly or whether their values are copied into the run file. This is a review item. The page also records that the library choice for the cumulative yield (JEFF-3.1.1 or ENDF/B) is not fixed.

### 7.2 New and changed output columns

The v1.0 table `timeseries.csv` has seven columns ([src/main.rs, lines 45 to 53](https://github.com/weewzr/STRATO/blob/v1.0.0/src/main.rs#L45-L53)). Block K1 asks for the quantities below. The column names are proposals. An entry is left empty when its denominator is 0.

| No. | Column (proposed) | Unit | Meaning | v1.0 counterpart | Status |
|---|---|---|---|---|---|
| 1 | `time_s` | s | output time | `time_s` | Kept |
| 2 | `temperature_k` | K | $T$ at the output time; only with a history | none | Planned |
| 3 | `inventory_mol` | mol | atoms inside the particle | `inventory_mol` | Kept |
| 4 | `generated_mol` | mol | $M_\mathrm{gen}$, summed source | none | Planned |
| 5 | `decayed_mol` | mol | $M_\mathrm{dec}$, decayed inside | none | Planned |
| 6 | `cumulative_release_mol` | mol | $M_\mathrm{rel}$, crossed the surface | `cumulative_release_mol` | Kept |
| 7 | `outside_mol` | mol | $M_\mathrm{out}$, undecayed outside | none | Planned |
| 8 | `decayed_outside_mol` | mol | $M_\mathrm{dec,out}$ | none | Planned |
| 9 | `release_rate_mol_s` | $\mathrm{mol\ s^{-1}}$ | $J_R$ with the $G_R$ of the step | `release_rate_mol_s` | Kept |
| 10 | `generation_rate_mol_s` | $\mathrm{mol\ s^{-1}}$ | `gen_rate` | none | Planned |
| 11 | `residual_inside_mol` | mol | $(M_0+M_\mathrm{gen})-(M+M_\mathrm{dec}+M_\mathrm{rel})$ | `balance_residual_mol` (no generation term) | Changed |
| 12 | `residual_outside_mol` | mol | closure with $M_\mathrm{out}$ and $M_\mathrm{dec,out}$ | none | Planned |
| 13 | `min_concentration_mol_m3` | $\mathrm{mol\ m^{-3}}$ | smallest cell value; below 0.0 is a failure | `min_concentration_mol_m3` | Kept |
| 14 | `release_fraction` | none | $F_\mathrm{rel}=M_\mathrm{rel}/M_0$, source-free heating | `release_fraction` | Kept |
| 15 | `fraction_decay_corrected` | none | $F_\mathrm{corr}=e^{\lambda t}M_\mathrm{out}/M_0$; skipped if $\lambda t>700$ | none | Planned |
| 16 | `fraction_generated` | none | $F_\mathrm{gen}=M_\mathrm{rel}/M_\mathrm{gen}$, irradiation from zero | none | Planned |
| 17 | `release_to_birth` | none | $R/B=J_R/\mathrm{Gen}$ | none | Planned |
| 18 | `fraction_booth` | none | $F_B=M_\mathrm{out}/(M+M_\mathrm{out})$, TECDOC-1674 Eq. (7.8) | none | Planned |
| 19 | `fraction_post_irradiation` | none | $F_\mathrm{post}$ from the stored values at $t_\mathrm{irr}$ | none | Planned |

Table: 7.3 Output columns of the time series (names are proposals).

The residual of the v1.0 run is $M+M_\mathrm{rel}-M_0$ ([line 220](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L220)). With $M_\mathrm{gen}=0$ the first v2.0 residual is the same number with the opposite sign, so a comparison with v1.0 files must account for the sign. Whether FIXED mode keeps the v1.0 column set and order byte for byte is not stated on the pseudocode page. That is a review item, because the regression tests V1 and VT16 ask for bit-identical values, which says nothing about headers.

The end-of-run report (K2) adds the following items to the console lines of v1.0.

| No. | Item | Unit |
|---|---|---|
| 1 | largest inside and outside residual and the step where each occurred | mol, step number |
| 2 | smallest concentration over the run | $\mathrm{mol\ m^{-3}}$ |
| 3 | number of steps; cap that controlled the step size; shortest and longest step | count, name, s |
| 4 | number of $D$-floor events and each window warning | count, text |
| 5 | precursor warning with $t_{99}$, if raised | text, s |
| 6 | covered eigenvalue and first valid time of a modal or effective-time reference | $\mathrm{s^{-1}}$, s |

Table: 7.4 Additional end-of-run report items.

The files `final_profile.csv`, the comparison files and the SVG figures of v1.0 are not mentioned in the pseudocode page for Phase 2. Their treatment is a review item.

## 8 Verification

Classes: PROVED, DEMONSTRATED NUMERICALLY or CLAIMED ONLY, as in Section 9.2 of the pseudocode. Rows K1 to K5 are v1.0 tests that stay. Rows 8.1 onward are the planned v2.0 tests. Quantities in the last column marked "observed" belong to a Python transcription. Each must be repeated on the Rust code before it counts. Tolerances of rows 8.8 (V7) and 8.6 (V6a) come from the first run of that transcription (open item O3 of the pseudocode). The test names follow Section 7 of the pseudocode (V, VT and VI series).

| No. | Test | Exact reference | Quantity | Expected order | Tolerance and justification |
|---|---|---|---|---|---|
| K1 | five-layer FV against series, extrapolated | Code C series | $F(t)$ | none measured; the 200-cell error is about $5.5\times10^{-6}$ (second order) | absolute difference below $2\times10^{-5}$; value fixed in [tests/finite_volume_vs_analytical.rs, lines 14 to 21](https://github.com/weewzr/STRATO/blob/v1.0.0/tests/finite_volume_vs_analytical.rs#L14-L21) |
| K2 | series against Python oracle | independent value 0.4325556390930383 | $F(t)$ | exact | $10^{-10}$ ([lines 24 to 30](https://github.com/weewzr/STRATO/blob/v1.0.0/tests/finite_volume_vs_analytical.rs#L24-L30)); two codes agree to round-off |
| K3 | regression against delivered v1.0 output | `tests/golden/v1.0/*.csv` | inventory, release, $F$, profile | exact | round-off, scaled by $M_0$ ([tests/phase1_regression.rs, lines 41 to 87](https://github.com/weewzr/STRATO/blob/v1.0.0/tests/phase1_regression.rs#L41-L87)); valid for constant data only |
| K4 | refinement study | exact series | observed orders | 2.0 space, 1.0 time | spatial 1.8 to 2.2 and temporal 0.8 to 1.2 ([examples/verify.rs, lines 186 to 191](https://github.com/weewzr/STRATO/blob/v1.0.0/examples/verify.rs#L186-L191)) |
| K5 | balance and positivity | $M_0 = M + M_\mathrm{rel}$ | relative residual, minimum $C$ | exact | residual below $10^{-12}$ ([lines 213 to 223](https://github.com/weewzr/STRATO/blob/v1.0.0/examples/verify.rs#L213-L223)); the v1.0 unit test accepts $-10^{-13}$ for $C$ ([finite_volume.rs, line 310](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L310)), v2.0 accepts none |
| 8.1 | regression of v2.0 to v1.0 (V1, VT16, VI4) | v1.0 run, $\lambda=0$, $S=0$, constant $T$, FIXED grid | every row and the profile | exact | 0 ulp; adding exact 0.0 leaves floats unchanged; CLAIMED ONLY for Rust until run |
| 8.2 | decay without diffusion (V2) | $(1+z)^{-n}C^0$ (P21.63), $e^{-\lambda t}$ | $C$ | 1 in $\Delta t$ | identity $10^{-12}$ over at most 300 steps; error within 2 % of $\tfrac12\lambda^2t\Delta t$; observed $1.2\times10^{-14}$ and 0.08 % |
| 8.3 | decay with diffusion, no source (V3) | v1.0 step with $\Delta t'=\Delta t/(1+z)$ (P21.63) | cells, corrected release fraction | 1 in $\Delta t$ | identity $10^{-11}$; order 0.8 to 1.2; observed $4.6\times10^{-15}$, order 1.00 |
| 8.4 | constant kernel source, no decay (V4) | steady state (P21.82) to (P21.84) | $J_R/\mathrm{Gen}$, $M_\mathrm{ss}/\mathrm{Gen}$ | 2 in space | $\lvert J_R/\mathrm{Gen}-1\rvert<10^{-12}$; order 1.9 to 2.1; observed $1.3\times10^{-15}$, 1.97 and 1.99 |
| 8.5 | source with decay, fixed point (V5) | $R/B=0.9684291531$ (P21.85), own transfer walk | $R/B$ | 2 in space | order 1.9 to 2.1; value within $2\times10^{-6}$ on the (20, 8, 4, 4, 4) mesh; observed 0.968428661, order 1.99 |
| 8.6 | Booth sphere, steady (V6a) | (P21.86) | $R/B$ | 2 in space | order 1.9 to 2.1 for $N=100$, 200, 400; the criterion is the order because $x=100$ misses $10^{-5}$ at $N=200$ (thin boundary layer) |
| 8.7 | Booth sphere, transient (V6b) | TECDOC-1674 Eq. (7.8), $x=1$, (P21.87) | $F_B=M_\mathrm{out}/(M+M_\mathrm{out})$ | 1 in $\Delta t$ | grid and step halved together; observed 0.94; band not fixed in the pseudocode, proposal 0.8 to 1.2 as in K4 |
| 8.8 | modal series against FV, five layers (V7) | block R1 series | $\lvert\Delta\rvert/M_\mathrm{gen}(t_\mathrm{irr})$ | 2 in space | below $2\times10^{-3}$ on the finest mesh, factor 1.8 or more per halving; observed worst $9.7\times10^{-4}$ (O3) |
| 8.9 | manufactured solution, two layers (V8) | (P21.88) to (P21.90) | error norm | 2 steady, 1 transient | within 0.1 of the order; passed in `sec4_p21_mms_source.py`, absent from the transcription |
| 8.10 | closure with switching source, variable steps (V9) | exact linear identity (P21.72), (P21.75) | relative residual | exact | $10^{-12}$; observed $9.5\times10^{-16}$ and $6.3\times10^{-16}$ |
| 8.11 | positivity (V10, VT14) | proof (P21.62), (P22.17) | minimum $C$ | exact | at least 0.0 with no tolerance; the proof uses nonnegative operations only; $\Delta t=10^{-3}$ to $10^{12}\ \mathrm{s}$ |
| 8.12 | CRP-6 case 1a (V11) | printed 0.4673; bracket 0.46721 to 0.46742 | $F$ | 2 in space | $\lvert F-0.46742\rvert<2\times10^{-4}$; wide because the $R_g$ of the participants is NOT VERIFIED (O5); observed 0.46737 with 8.3145, a constant that step C2 does not offer; the selectable SI value and 8.3143 give 0.467404 and 0.467337, both inside |
| 8.13 | CRP-6 case 1b (V12) | series $1-F=4.0703\times10^{-7}$ | $1-F$ | 1 in $\Delta t$ | within 5 % at $\Delta t\le36\ \mathrm{s}$; at 360 s the error follows (P21.78): observed 5.14 % against 5.05 % predicted |
| 8.14 | source switch inside a step (V13) | analytic integral of the source | $M_\mathrm{gen}$ | exact | $10^{-13}$ with option A; option R misses by $3\times10^{-3}$ |
| 8.15 | steady state kept by Backward Euler (V14) | semi-discrete steady state (P21.68) | profile change | exact | $10^{-11}$ for any step size; observed $9.1\times10^{-16}$ |
| 8.16 | decay-exact split as cross-check (V15) | $e^{-\lambda t}\times$ v1.0 run (P21.67) | cells | exact | $10^{-12}$; observed $3.3\times10^{-15}$ |
| 8.17 | decay constants (V16) | `decay.csv` (ENSDF) | seconds, $\lambda$ | exact | seconds $10^{-8}$, $\lambda$ relative $5\times10^{-7}$; stable row gives 0; negative half-life rejected |
| 8.18 | cell and step averages, helper identities (V17, V21) | (P21.54), (H21.1) to (H21.3) | averages | exact | $10^{-12}$ and $10^{-14}$ relative; observed $4.4\times10^{-16}$ and $2.9\times10^{-16}$ |
| 8.19 | modal recursion, Booth sphere, $\lambda=0$ (V18 to V20) | ODE integration; Eq. (7.8); $M_\mathrm{out}=M_\mathrm{rel}$ | modal amplitudes, $F_B$ | exact | $10^{-8}$ (25 modes), $5\times10^{-9}$, $10^{-12}$ relative |
| 8.20 | input validation (VT1) | field-by-field rejection list | error text | exact | each bad input names its field; no panic; $Q=0$ accepted |
| 8.21 | $D(T)$ by log-sum-exp (VT2) | 40-digit evaluation, 4 register laws, 5 temperatures | $D$, $\ln D$ | exact | $10^{-14}$ relative; observed worst $8.4\times10^{-15}$, a margin of only 1.2; $\ln D$ finite where $e^{-Q/(R_gT)}$ underflows |
| 8.22 | $Q_\mathrm{eff}$ and crossover (VT3) | (P22.6), (P22.7) | 130.1 and 438.2 $\mathrm{kJ\ mol^{-1}}$, 1772.4 K | exact | 1772.4 K within 0.1 K (pseudocode); the 0.1 $\mathrm{kJ\ mol^{-1}}$ for the two $Q$ values is a proposal; observed 130.10, 438.22, 1772.44 |
| 8.23 | gas constant on CRP-6 1a (VT4) | 0.467337 (8.3143), 0.467404 (SI) | $F$ | exact | $2\times10^{-6}$; the two values round differently, so the test separates them |
| 8.24 | history construction and interpolation (VT5, VT6) | register totals 333.78, 335.95, 338.12 h and nine CRP-6 totals | durations, $T(t_k)$ | exact | 0.01 h (printed precision); node values exact; no extrapolation |
| 8.25 | event landing and planner (VT7, VI1, VI5) | float equality of event and step end | step sequence | exact | every event is a step end; no step below 1 s apart from the first (VI5); a duplicate event changes nothing; observed 41 of 41 and 8 of 8 events |
| 8.26 | step bound (VT8) | (P22.20) | largest $\lvert\Delta\ln D\rvert$ | exact | at most $2\varepsilon_D$ in every step; no step below 1 s next to a step of 100 s; observed 0.102 for $\varepsilon_D=0.1$, shortest step 1.24 s |
| 8.27 | effective-time integral (VT9) | closed form with $E_2$; Gauss-Legendre | $\tau$ | exact | $10^{-12}$ closed form, $10^{-13}$ quadrature; observed $2.4\times10^{-15}$ |
| 8.28 | FV against series at $\tau(t)$, shared $Q$ (VT10) | block R2 series | $F$ | 2 space, 1 time | each within 0.1; observed 2.00 and a difference of $5.4\times10^{-7}$ |
| 8.29 | option A against Backward Euler in $\tau$ (VT11, VI6) | substitution (P22.57), $\lambda=0$ | cells | exact | $10^{-11}$; for $\lambda>0$ a first-order difference is expected (observed $2.7\times10^{-3}$) |
| 8.30 | exactness check refuses unshared data (VT12) | order-swap demonstration | $F$ of two histories | exact | NotExact for layer-specific $Q$; observed 0.011413 against 0.009919; shared $Q$ agrees to $8.2\times10^{-15}$ |
| 8.31 | closure with a changing $G_R$ (VT13, VI2) | (P22.16), (P21.72), (P21.75) | relative residual | exact | $10^{-12}$ every step; a negative test (VI3) with the first-step $G_R$ must exceed $10^{-8}$ (observed $1.2\times10^{-5}$) |
| 8.32 | decay with effective time (VT15) | stiff real-time integration | inventory, release | exact | $10^{-11}$ relative; observed $1.6\times10^{-12}$ and $2.8\times10^{-12}$ |
| 8.33 | projection at a temperature step (VT17) | Richardson-extrapolated FV | $F$ | 2 (FV reference) | $10^{-6}$ for a 1 h step; $P=I$ when diffusivities are unchanged |
| 8.34 | staircase of a ramp (VT18) | stiff ODE reference | last-pair orders | 1, 2, 2 | within 0.15 for $N_T\ge32$ (left, midpoint, average); observed 1.00, 2.03, 1.95 |
| 8.35 | convergence of the joined scheme (PROPOSED) | refinement in $\Delta t$ and $h$ with decay, source and a ramp together | $F$, $M_\mathrm{dec}$ | 1 time, 2 space | not in the pseudocode; band to be fixed after the first run; today the joined scheme is CLAIMED ONLY |

Table: 8.1 Verification tests, references and tolerances.

The pseudocode assigns the V cases only: the algebraic ones (V1 to V3, V9, V10, V13 to V17, V21; rows 8.1 to 8.3, 8.10, 8.11, 8.14 to 8.18) fit `cargo test --release`, and the accuracy studies (V4 to V8, V11, V12, V18 to V20; rows 8.4 to 8.9, 8.12, 8.13, 8.19) fit `examples/verify.rs` or a longer test file. For the VT and VI rows the split is a proposal: short checks (8.20, 8.22 to 8.27, 8.29, 8.31) in `cargo test`, order and reference studies (8.21, 8.28, 8.30, 8.32 to 8.35) in the example. Neither split has been tried in Rust.

| No. | What the Python scripts do not establish | Consequence |
|---|---|---|
| 8.36 | bit identity (8.1) is checked in Python floating point only | claim stays open for Rust until the test runs |
| 8.37 | behaviour and speed of the compiled code | not tested |
| 8.38 | case V8 and the transient Booth tolerance were not run in the transcription | rows 8.7 and 8.9 have no observed value in that script |
| 8.39 | tolerances come from one run | revisit after the Rust run (O3) |

Table: 8.2 Limits of the present evidence.

## 9 Documentation

Documents that v2.0 must carry, what they hold and where they stand today. "In tree" means a file in the working tree of the repository. Update 2026-11-13: `data/`, `scripts/`, `docs/pandoc/`, `.github/workflows/docs-pdf.yml`, `docs/mathematics/v1.0.0/`, the `CHANGELOG.md` edit and the Phase 2 documents, including the reconciled derivation `docs/mathematics/STRATO_Phase2_Derivations.md`, are committed to the repository. The earlier statement that they were untracked described the working tree before those commits. The six check scripts of the open-item work in `scripts/derivation_checks/`, the open-item edits of the four Phase 2 documents listed here (rows 9.4, 9.5, 9.18 and this change list) and the Phase 2 entries of `.github/workflows/docs-pdf.yml` were added on 2026-11-13, after those commits.

| No. | Document | Location | Content | Status |
|---|---|---|---|---|
| 9.1 | README | `README.md` | status banner, assumptions table, outputs, verification table, known limitations | v1.0 text; to be rewritten at release (its development-status table already lists v2.0 as "In development") |
| 9.2 | Changelog | `CHANGELOG.md` | added, changed and removed items per version | `[Unreleased]` entry exists; to be filled at release |
| 9.3 | Change list v1.0 to v2.0 | `docs/development/v1_to_v2_changes.md` | numbered tables of changes, assumptions, limitations, verification | this file |
| 9.4 | Phase 2 derivation | `docs/mathematics/STRATO_Phase2_Derivations.md` | Sections 3 to 5: decay, source, modal solution, FV scheme, Arrhenius, effective time; appendices A (check scripts) and B (independent verification record) | in the repository (earlier version committed on 2026-11-07); DRAFT FOR REVIEW; the v1.0 line reference of Section 4.3 was clarified in the final review of the pseudocode (open item O6 of the pseudocode, resolved); open-item results added 2026-11-13 (Sections 3.5.3, 4.6.1, 5.2.5, 5.3.7, 5.3.8 and 5.4.6, Appendices A.2 and B.5) |
| 9.5 | Phase 2 pseudocode | `docs/mathematics/STRATO_Phase2_Pseudocode.md` | blocks A to M and R1 to R4, tables of status, 16 decisions, proposed step changes (Section 9.5) | in the repository (earlier version committed on 2026-11-07); DRAFT FOR REVIEW; updated 2026-11-13 |
| 9.6 | Phase 1 derivation | `docs/mathematics/phase1/` | v1.0 derivation and pseudocode as delivered | kept unchanged as the historical record |
| 9.7 | Parameter register | `data/reference/STRATO_Parameter_Register.md` and `data/*/*.csv` | literature parameters with source, page and status; gap list | in the repository (committed); draft; gaps listed in Section 9.3 of the pseudocode |
| 9.8 | Derivation check scripts | `scripts/derivation_checks/` | sympy and numeric checks of Sections 3 to 5 and the pseudocode | in the repository; the six open-item scripts (`sec3_o4_*`, `sec4_p2_*`, `sec5_o10_*`, `sec5_o11_*`, `sec5_o12_*`, `sec5_r24_*`) were added on 2026-11-13; pinned environment `requirements.lock.txt` |
| 9.9 | Input reference | to be written | new TOML fields for decay, source, Arrhenius laws and histories, with units | PLANNED after the decisions of Section 9.4 of the pseudocode |
| 9.10 | Output reference | to be written | new columns for generated, decayed and outside amounts, two residuals, $T$ and $D$ | PLANNED |
| 9.11 | Input template | `inputs/literature/literature_inputs.template.toml` | placeholders for literature runs | v1.0 content; must gain the new fields |
| 9.12 | Provenance | `docs/development/provenance.md` | origin of each file and the evidence | v1.0 only; needs a v2.0 section for new files |
| 9.13 | Verification report | output of `examples/verify.rs` and Table 8.1 | observed values in Rust beside the tolerances | PLANNED; no Rust result exists |
| 9.14 | Open items and decisions | Sections 9.3 and 9.4 of the pseudocode | O1 to O17 and decisions 1 to 16 | open; the work stops for the user's review; O4, O10 to O14 updated on 2026-11-13 |
| 9.15 | Version metadata | `Cargo.toml`, `CITATION.cff` | version 1.0.1 in both (v1.0.1 release, licence AGPL-3.0-only) | change again at the v2.0 release |
| 9.16 | PDF build | `docs/pandoc/`, `.github/workflows/docs-pdf.yml` | pandoc and XeLaTeX with a table-caption filter | in the repository (committed); the Markdown rules (tags, "Table:" captions, no raw pipes in math) serve this build; the Phase 2 derivation and pseudocode were added to the workflow's list on 2026-11-13, and their PDF build has not been run |
| 9.17 | Full v1.0.0 derivation | `docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md` | every v1.0 equation with the code lines that compute it | in the repository (committed); describes v1.0 only |
| 9.18 | Phase 2 review package | `docs/development/phase2_review_package.md` | reading order, status and the list of what the user must decide | in the repository (earlier version committed on 2026-11-07; updated 2026-11-13); READY FOR THE USER'S REVIEW |

Table: 9.1 Documentation for v2.0.
