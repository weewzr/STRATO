# STRATO v2.0 Phase 2 pseudocode: decay, source, inventory, Arrhenius diffusivity and temperature histories

Date: 2026-11-06; open items updated 2026-11-13 (Sections 9.2, 9.3 and 9.5). Status: DRAFT FOR REVIEW.

**Not implemented yet: Rust follows after review.** No Rust has been written for Phase 2. No file of STRATO v1.0.0 under `src/`, `tests/` or `examples/` has been changed. This page is the plain-language pseudocode that comes after the mathematics (Sections 3 to 5 of the derivation, equations (P21.1) to (P21.92) and (P22.1) to (P22.59)) and before any code. Work stops here until the review is finished. The open-item work of 2026-11-13 added the derivation equations (P21.93) to (P21.142) and (P22.60) to (P22.142); this page cites them in notes and in the proposals of Section 9.5, and no algorithm step uses them yet.

## 1 Reading guide

### 1.1 What this page is

Phase 2 of STRATO adds two things to the v1.0 finite-volume solver (Code B) and to the modal series (Code C).

- Phase 2.1: radioactive decay, a fission-product source and a full inventory account (generated, decayed, released, still inside, still undecayed outside).
- Phase 2.2: a diffusivity that follows temperature by an Arrhenius law, and a temperature that follows a history in time.

The two parts were first written as separate drafts: "the decay part" (Phase 2.1, blocks A to F) and "the temperature part" (Phase 2.2, blocks T1 to T7). Their block codes appear in this page only in the lines that say where a block came from and in Section 8. This page joins them into one run of the program, in the order in which the program does things, and then checks every step against the equation it cites. Section 8 lists the places where the two drafts disagreed or where an equation was cited loosely. In each case the pseudocode was changed. No equation was changed.

### 1.2 Step format

Each block is a list of numbered short steps. A step has a code such as H4.2 and a plain-language action. A line that begins with "why:" gives the reason. A step that rests on an equation names it in parentheses, for example (P21.58). A step with no equation number is bookkeeping, an input check or a design rule.

Three marks show how a step relates to v1.0.

| Mark | Meaning |
|---|---|
| [KEPT] | the step is identical to STRATO v1.0.0 |
| [CHANGED] | a v1.0.0 step gets a different formula |
| [NEW] | the step does not exist in v1.0.0 |

Table: 1.1 Change marks.

Four words show how firm the mathematics behind a step is.

| Word | Meaning |
|---|---|
| APPROVED | the equation is in the reconciled Sections 3 to 5 and was re-derived in B3 or covered by the section check logs and scripts |
| DESIGN | a choice taken in this pseudocode that no equation fixes (a default, a data layout, a safety rule); the review may overrule it |
| OPEN | the equation or its input data is not settled; the step says what is missing |
| HELPER | a small identity used only to arrange a formula for stable arithmetic; proved in a check script and not part of the reconciled sections |

Table: 1.2 Status words for steps.

Every equation or operation is also classified as IMPLEMENTED (with file and line in the v1.0.0 tag), DERIVED NOT IMPLEMENTED (derived and checked, no Rust yet) or FUTURE (outside Phase 2, documented only). Every numerical property is classified as PROVED, DEMONSTRATED NUMERICALLY or CLAIMED ONLY. Section 9 gives the tables.

### 1.3 Equation numbers and names

Equations (P21.x) come from Sections 3 and 4 and equations (P22.x) from Section 5. Three small identities for stable arithmetic carry the numbers (H21.1) to (H21.3) and two design relations carry (H22.1) and (H22.2); they appear where they are used. Code links point to the tag v1.0.0, for example [src/numerical/finite_volume.rs, lines 162 to 182](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L162-L182) is the v1.0 Backward Euler step.

Units inside the program are SI: metres, seconds, kelvin, joules per mole, moles per cubic metre. A number that enters in other units carries a unit tag and is converted first (block A). Names in lower case with underscores are values. Names with a leading capital (`Law`, `History`) are records, that is, labelled bundles of numbers. A list is written `[a, b, c]` and is indexed from 1 in the text and from 0 in the planned code.

The two drafts used some symbols twice. The pseudocode uses the names in Table 1.3 so that one name means one thing. Section 6 gives the full glossary.

| Name in this page | Replaces | Reason |
|---|---|---|
| `gen`, `gen_rate` | $G$ (generation rate in $\mathrm{mol\ s^{-1}}$) | $G$ is also the face conductance of the finite-volume step |
| `west_g`, `east_g`, `G_R` | $g^W_j$, $g^E_j$, $G_R$ | unchanged; these are the conductances |
| `mode_kappa` | $\kappa_n=\Lambda_n+\lambda$ | `ratio_i` now names the layer ratio of Eq. (H22.2), formerly also $\kappa_i$ |
| `Rg` | $R$ in the Arrhenius code | $R$ is the outer radius of the particle |
| `t_event` | $e$ for an event time | `pivot_e[j]` keeps $e_j$ of Eq. (P21.61) |
| `arg_mode`, `arg_branch` | $x$ | $x$ was used for three different arguments |

Table: 1.3 Renamed symbols.

## 2 Overview of one v2.0 run

### 2.1 Flow of one run

```text
 RUN FILE (regions, laws, nuclide, source, temperature history, time settings)
      |
      v
 [A] check the input, convert units to SI
      |
      +--> [B] decay constant        (from the half-life)
      +--> [C] diffusivity laws      (D0 and Q of each layer)
      +--> [D] temperature history   (T as a function of time)
      +--> [E] source history        (fission rate, yield, source per interval)
      |
      v
 [G] geometry once                   (volumes, face areas, distances, cell factors)
 [F] list of events                  (breakpoints, source switches, output times)
      |
      v
 +-------------------------- TIME LOOP, one pass per step ---------------------------+
 |  [F] choose the next step: t_old --> t_new, dt, landing on every event            |
 |  [H1] diffusivity of each layer over this step                                    |
 |  [H2] conductances from those diffusivities                                       |
 |  [H3] source of each cell, averaged over the step                                 |
 |  [H4] one Backward Euler solve with decay (v1.0 elimination, new diagonal)        |
 |  [J]  bookkeeping: inside, generated, decayed, released, outside, residuals       |
 |  [K]  write a row if t_new is an output time                                      |
 +-----------------------------------------------------------------------------------+
      |
      v
 [K] end-of-run report (closure, positivity, warnings, counts)

 Reference solutions, outside the loop and used to test it:
 [R1] modal series with decay and source   [R2] effective time (shared scaling only)
 [R3] piecewise-constant temperature       [R4] staircase of a ramp (test tool)
```

Read the diagram from top to bottom. The set-up blocks run once. The loop runs once per time step. Blocks R1 to R4 never run inside a finite-volume run; they produce reference numbers to compare with it.

### 2.2 Blocks in run order

| Block | Content | Origin in the part drafts | Main equations | Planned Rust module |
|---|---|---|---|---|
| A | input and unit checks | new glue; $T$ and $Q$ unit rules from T1 | (P22.10) | `src/config.rs` (extended) |
| B | decay constant from half-life | decay part, block A | (P21.5) to (P21.7) | `src/physics/decay.rs` |
| C | Arrhenius diffusivity of a layer | temperature part, T1 | (P22.1), (P22.5) to (P22.7), (P22.9) | `src/physics/diffusivity.rs` |
| D | temperature history | temperature part, T2 | (P22.21), (P22.39), (P22.58), (P22.59) | `src/thermal/temperature_history.rs` |
| E | yield and source history | decay part, block B | (P21.8), (P21.10) to (P21.12), (P21.52), (P21.57) | `src/physics/fission_source.rs` |
| F | events and step planner | decay part C1 and temperature part T3, merged | (P21.77) to (P21.79), (P22.19), (P22.20) | `src/numerical/finite_volume.rs` (planner) and `src/thermal/temperature_history.rs` |
| G | geometry once, cell factors | temperature part T4.1 and decay part B3 | (P21.54), (P22.14) | `src/numerical/finite_volume.rs` (extended) |
| H | one time step | T4.2, T4.3, B4, B5, C2 | (P21.57) to (P21.62), (P22.13) to (P22.17), (P22.22) | `src/numerical/finite_volume.rs` (extended) |
| J | bookkeeping | decay part D and T4.5 | (P21.40) to (P21.45), (P21.70) to (P21.76), (P22.16) | `src/physics/inventory.rs` |
| K | output rows and end report | decay part D9, D10 | (P21.45) | `src/physics/inventory.rs`, `src/main.rs` |
| M | the main loop that calls A to K | new glue | none | `src/numerical/finite_volume.rs` (`run` extended) |
| R1 | modal series with decay and source | decay part, block E | (P21.24) to (P21.49) | `src/analytical/modal_source.rs` |
| R2 | effective time | temperature part T5 | (P22.24) to (P22.37) | `src/analytical/thermal_intervals.rs`, `src/thermal/temperature_history.rs` |
| R3 | piecewise-constant temperature with projection | temperature part T6 | (P22.39) to (P22.46) | `src/analytical/thermal_intervals.rs`, `src/analytical/modal_source.rs` |
| R4 | staircase of a ramp (test tool) | temperature part T7 | (P22.53) to (P22.57) | `src/analytical/thermal_intervals.rs` |

Table: 2.1 Blocks in run order.

### 2.3 Which solution path serves which input

| Input | Path | Notes |
|---|---|---|
| any allowed input | finite volume, blocks A to M | the production path; first order in time, second order in space |
| constant diffusivities, uniform kernel start and kernel source, absorbing surface | R1 | exact in time for a piecewise-constant source |
| one layer, or all layers sharing one temperature dependence, any decay, any source | R2, after the structural test of step R2.1 | refused with a reason when the test fails; real TECDOC-978 data fail it for five layers |
| temperature constant on each interval, layers do not share a dependence | R3 | exact for the untruncated series; the reference for a temperature step such as CRP-6 case 3c |
| temperature ramp, layers do not share a dependence | R4 (test tool) or the finite-volume path | no general closed form is available and no effective time exists (Section 5.4.2 of the derivation); R4 is slow and serves verification only |

Table: 2.2 Solution paths.

### 2.4 Planned Rust modules (proposal)

None of these exists yet. The v1.0 files stay as released until the review is finished.

| Planned module | New or extended | Blocks |
|---|---|---|
| `src/physics/decay.rs` | new | B |
| `src/physics/fission_source.rs` | new | E, and the cell factor of G |
| `src/physics/inventory.rs` | new | J, K |
| `src/physics/diffusivity.rs` | new | C, and the step average of H1 |
| `src/thermal/temperature_history.rs` | new | D, the temperature part of F, and R2 |
| `src/numerical/finite_volume.rs` | extended | planner F, G, H, M |
| `src/analytical/modal_source.rs` | new | R1, and the propagation of R3 |
| `src/analytical/thermal_intervals.rs` | new | R2, R3, R4 |
| `src/config.rs` | extended | A |
| `tests/`, `examples/verify.rs` | new files and extended | Section 7 |

Table: 2.3 Planned modules.

## 3 One time step, side by side: v1.0 and v2.0

The left column follows the v1.0.0 code: [build_mesh, lines 50 to 99](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L50-L99), [the solver, lines 132 to 160](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L132-L160), [the Backward Euler step, lines 162 to 182](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L162-L182) and [the run loop, lines 184 to 236](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L184-L236). The right column is the plan of this page. The line numbers were read from the tag.

| Item | v1.0 FV time-loop step | v2.0 step |
|---|---|---|
| mesh and conductances | `build_mesh` fixes volumes, centroids and series conductances once, from one constant diffusivity per cell (lines 78 to 91) | block G fixes everything that does not depend on $D$ once; block H2 recomputes the conductances in every step from the layer diffusivities of that step (P22.14) [CHANGED] |
| initial state | kernel cells hold the initial concentration, all others zero ([lines 114 to 120](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L114-L120) sum the inventory) | same; all zero is allowed when a source is present [CHANGED] |
| start check | stop if the initial inventory is not positive (lines 186 to 189) | stop only if the initial inventory is zero and the source is zero everywhere (J0) [CHANGED] |
| time grid | `steps = ceil(end_time / dt)`, `dt = end_time / steps`, one constant step (lines 190 and 191) | FIXED mode keeps this rule bit for bit; PLANNED mode lands on every event and respects the decay, diffusion and temperature limits (block F) [NEW] |
| diagonal excess of the matrix | `v[i] = volume / dt` (line 178) | `volume / dt + decay_constant * volume` (H4.2) (P21.59) [CHANGED] |
| right-hand side | `rhs[i] = v[i] * old[i]` (line 179) | `(volume / dt) * old[i] + volume * source_cell[i]` (H4.2) (P21.59) [CHANGED] |
| elimination | subtraction-free tridiagonal solve (lines 132 to 160) | unchanged (H4.3) (P21.61) [KEPT] |
| release rate | `mesh.outer_conductance * next[last]` (line 210) | `G_R * conc_new[N]` with the $G_R$ of this step (J1.2) (P22.16) [CHANGED] |
| release sum | Kahan sum of `dt * release_rate` (lines 211 to 217) | Kahan sums of the release, the generated amount, the decayed amount and the decayed-outside amount (J1.3) (P21.72) [CHANGED] |
| inventory | `inventory(mesh, c)` (line 219) | same function (J1.1) [KEPT] |
| residual | `inv + released - initial` (line 220) | two residuals, inside and outside, relative to $M_0+M_\mathrm{gen}$ (J1.5) (P21.72), (P21.75) [CHANGED] |
| atoms outside the particle | not tracked | `out_now` and the decayed-outside sum (J1.4) (P21.73), (P21.75) [NEW] |
| output rule | every `output_every_steps` steps and the last step (line 221) | at the output times (PLANNED) or every `output_every_steps` steps (FIXED) [CHANGED] |
| positivity | reported as the minimum concentration; the v1.0 unit test accepts $-10^{-13}$ (line 310) | reported; the proof says it is not below 0.0, so any negative value is a failure (J1.6) (P21.62), (P22.17) [CHANGED] |
| temperature | none; $D$ is constant | block D supplies $T(t)$; block C turns it into $D$ [NEW] |
| decay | none | $\lambda$ from block B enters only the diagonal [NEW] |
| source | none | block E and H3 [NEW] |

Table: 3.1 The v1.0 time-loop step and the v2.0 step.

## 4 Blocks in run order

### 4.1 Block M: the main loop

Block M only calls the other blocks in the right order. It adds no mathematics.

```text
BLOCK M  one complete run                       planned module: src/numerical/finite_volume.rs (run, extended)
INPUT   the checked run description of block A
OUTPUT  rows at the output times; the final concentration profile; the end-of-run report

M1  Set-up, once per run, in this order:
      decay_constant, mean_life_s       = block B
      laws, settings                    = block C
      history                           = block D          (none for a constant-temperature run)
      yield, source_history             = block E
      geometry, cell_factor             = block G
      events, step caps                 = block F, steps F1 to F4
M2  Start state: conc[j] = initial kernel concentration in the kernel cells, 0 in all others.
    Run block J, step J0 (totals set to zero, inv0 computed).
    Write the row for t = 0.
M3  WHILE the last event (t_end) has not been reached:
      (t_old, t_new, dt, flags)         = block F, step F5
      D_layer[1..5]                     = block H, step H1
      west_g[j], east_g[j], G_R         = block H, step H2
      source_cell[j], gen_rate          = block H, step H3
      conc_new[j]                       = block H, step H4
      block J, step J1, with conc_new, G_R, gen_rate and dt
      IF t_new is an output time THEN write a row (block K, step K1)
      conc = conc_new
M4  Write the end-of-run report (block K, step K2).
    why: every quantity that enters the matrix, the source integral and the bookkeeping of one
         step is computed from the same t_old, t_new, dt and G_R, so the balance can close.
```

### 4.2 Block A: input and unit checks

**Origin and status.** New glue. The rules for tags come from the temperature draft (steps T1.1 and T1.2). Equation used: (P22.10) for $T=T_C+273.15\ \mathrm{K}$, APPROVED.

```text
BLOCK A  input and unit checks                       planned module: src/config.rs (extended)
INPUT   the run file of v1.0 plus the keys named below
OUTPUT  a checked run description in SI units, or an error that names the field   [NEW]

A1  Regions: exactly five, outer radii strictly increasing, at least one cell each.             [KEPT]
    why: this is the v1.0 rule (src/config.rs lines 79 to 82). Decision 1 asks whether to
         allow a one-region particle for the Booth test cases.
A2  Nuclide: a name (block B reads its half-life) or "none" (decay_constant = 0).
A3  Source: a fissile mix with yields (block E) and a fission rate history, or "none".
A4  Layer laws: for each layer either a constant diffusivity (stored as one branch with Q = 0)
    or a list of branches with D0 and Q, with the unit tag of Q.
A5  Temperature history: kind LINEAR or CONSTANT with unit tags (block D), or "none".
A6  Time settings: t_end, grid mode FIXED or PLANNED, the request dt (FIXED) or the planner
    settings (PLANNED), output times or output_every_steps, step limit.
A7  Initial kernel concentration in mol per m3, nonnegative.
A8  Convert every value to SI using its unit tag. A missing or unknown tag is an error that names the field.
    degC to K: T = T_C + 273.15.                                                    (P22.10) APPROVED
    why: a missing tag would turn 1600 degC into 1600 K, or kJ per mol into J per mol.
A9  Consistency:
      IF any branch has Q > 0 THEN a temperature history must be given (it may be one constant interval).
      IF grid mode is FIXED THEN there must be no source switch time, no temperature knot and no output time list.
      IF the nuclide is an activation product THEN stop (block E, step E1).
    why: the fixed grid cannot land on an event; the planner can.
```

### 4.3 Block B: decay constant from half-life

**Origin.** Decay part, block A.

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P21.5), (P21.6) | $dN/dt=-\lambda N$, $N=N_0e^{-\lambda t}$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.7) | $\lambda=\ln 2/t_{1/2}$, mean life $1/\lambda$ | APPROVED | DERIVED NOT IMPLEMENTED |

Table: 4.1 Equations of block B.

The nuclear data table `data/nuclear/decay.csv` carries, per nuclide, the half-life as written in ENSDF (number and unit), the half-life in seconds and $\lambda$ to seven digits. Its year follows the IAEA LiveChart convention, 1 y = 31 556 925.97 s. A calendar year of 365.25 d (31 557 600 s) would change $\lambda$ by $2.1\times10^{-5}$ relative, so the unit table below must not be replaced by a rounder value. The half-lives are the ENSDF adopted values, not the half-lives printed in TECDOC-1674 Table 8.1.

```text
BLOCK B  decay constant                              planned module: src/physics/decay.rs
INPUT   nuclide name; half-life number and unit as in the nuclear data table
OUTPUT  decay_constant (1/s); mean_life_s (s)   [NEW]

B1  IF the half-life field is the word STABLE THEN
        decay_constant = 0 ; mean_life_s = "none" ; leave the block.
    why: a stable nuclide has no loss term. Blocks H and J then reduce to the v1.0 equations
         exactly, which is the regression test V1.
B2  Look up seconds_per_unit: s = 1, minute = 60, hour = 3600, day = 86400, year = 31 556 925.97.
    IF the unit is not in this list THEN stop with an error that names the unit.
    why: a silent default would give a wrong lambda by a factor such as 60.
B3  half_life_s = half_life_number * seconds_per_unit.
B4  IF half_life_s is not finite or not greater than 0 THEN stop with an error.
B5  decay_constant = ln(2) / half_life_s.                                    (P21.7) APPROVED
    why: setting N(t_half) = N0/2 in N = N0 exp(-lambda t) gives exp(-lambda t_half) = 1/2.
B6  mean_life_s = 1 / decay_constant.                                        (P21.7) APPROVED
B7  Consistency checks (report a warning, do not change the value):
      (a) decay_constant * half_life_s equals ln(2) to 1e-12 relative (a dimensionless product;
          any other result means a unit slip).
      (b) IF the table also gives half_life_s, B3 agrees with it to 1e-8 relative.
      (c) IF the table also gives lambda, decay_constant agrees with it to 5e-7 relative
          (the column has seven printed digits).
```

| Nuclide | Half-life (ENSDF) | $t_{1/2}$ in $\mathrm{s}$ | $\lambda$ in $\mathrm{s^{-1}}$ | Mean life in $\mathrm{s}$ |
|---|---|---|---|---|
| Cs-137 | 30.08 y | $9.492323\times10^{8}$ | $7.302187\times10^{-10}$ | $1.3695\times10^{9}$ |
| Sr-90 | 28.91 y | $9.123107\times10^{8}$ | $7.597709\times10^{-10}$ | $1.3162\times10^{9}$ |
| Ag-110m | 249.83 d | $2.158531\times10^{7}$ | $3.211198\times10^{-8}$ | $3.1141\times10^{7}$ |
| I-131 | 8.0252 d | $6.933773\times10^{5}$ | $9.996681\times10^{-7}$ | $1.0003\times10^{6}$ |

Table: 4.2 Decay constants that block B must reproduce (recomputed from `decay.csv` by `b4_phase2_pseudocode_integration.py`).

### 4.4 Block C: Arrhenius diffusivity of a layer

**Origin.** Temperature part, T1.

**In plain words.** A diffusivity that follows temperature is stored as a short list of terms. Each term is a pair, a pre-exponential factor $D_0$ and an activation energy $Q$. At a given temperature each term gives $D_0e^{-Q/(R_gT)}$ and the layer diffusivity is the sum of its terms, because two parallel transport paths add their fluxes. A layer that keeps the constant diffusivity of v1.0 is the special case of one term with $Q=0$.

The program never forms $e^{-Q/(R_gT)}$ alone for a large $Q$, because it can underflow to exactly zero: for the Kr and Xe law in PyC ($D_0=2.0\times10^{5}\ \mathrm{m^2\ s^{-1}}$, $Q=923\ \mathrm{kJ\ mol^{-1}}$, IAEA, 1997, Table A-2) at 100 K the logarithm of the term is $-1097.9$ while the exponential is 0 in double precision. It works with logarithms and adds the terms with the largest one factored out.

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P22.1) | $D_i(T)=\sum_mD_{0,i,m}\exp(-Q_{i,m}/(R_gT))$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.5) | $R_g=N_Ak_B$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.6) | apparent activation energy $Q_\mathrm{eff}$ as the $D$-weighted mean of the branch energies | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.7) | crossover temperature $T_x$ of a two-branch law | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.9) | fluence factor $\exp(\Phi_f/5)$ on one SiC branch, $\Phi_f$ in $10^{25}\ \mathrm{m^{-2}}$ with cutoff $E>16\ \mathrm{fJ}$ | formula APPROVED; conversion of the fluence cutoff OPEN (data) | DERIVED NOT IMPLEMENTED |
| (P22.10) | $T=T_C+273.15\ \mathrm{K}$ | APPROVED | DERIVED NOT IMPLEMENTED |

Table: 4.3 Equations of block C.

```text
Branch   = { D0 [m^2/s], Q [J/mol] }
Law      = { branches : list of Branch (length >= 1; the data use 1 or 2),
             fluence  : optional { branch_index, scale_1e25_per_m2 = 5.0, cutoff_tag } }
Window   = { T_min [K], T_max [K] }     -- optional validity range, only if printed in the source
Settings = { gas_constant : SI | FIXED_8_3143,   -- default SI
             D_floor [m^2/s] > 0 }
```

```text
BLOCK C  Arrhenius diffusivity                       planned module: src/physics/diffusivity.rs
INPUT   Law of each layer; temperature T in K; Settings; frozen fluence Phi_f (0 if none)
OUTPUT  D in m^2/s; apparent activation energy in J/mol                                  [NEW]

C1  load_law(raw_branches, q_unit, window, fluence)
      1. require q_unit in {"J/mol", "kJ/mol"}; otherwise stop with an error naming the field.
         why: TECDOC-978 defines Q in J/mol in the text but prints kJ/mol in every table column;
              a missing tag would be a silent factor 1000.
      2. for each branch: require D0 finite and D0 > 0; require Q finite and Q >= 0.
         why: ln(D0) must exist. Q = 0 is a real row (iodine in matrix pores), so zero is allowed.
      3. convert Q to J/mol (multiply by 1000 if the tag was kJ/mol).
      4. IF fluence is given THEN require cutoff_tag == "E>16fJ" (0.1 MeV).        (P22.9) [OPEN: data]
         why: AGR-1 fluences are quoted for E > 0.18 MeV; the conversion factor between the two
              cutoffs is NOT VERIFIED and must not be guessed. Any other tag is an error until a
              verified conversion exists.
      5. store the window if given. Never store a default window.

C2  ln_branch(law, m, T, settings, Phi_f)
      1. D0_eff = D0[m]; IF the fluence rule applies to branch m THEN D0_eff = D0[m] * exp(Phi_f / scale).   (P22.9)
         why: Phi_f is frozen during a post-irradiation heating test, so the factor is a constant
              of the run and not a function of time.
      2. Rg = 8.31446261815324 if settings.gas_constant == SI, else 8.3143.          (P22.5)
      3. return ln(D0_eff) - Q[m] / (Rg * T).                                        (P22.1)

C3  diffusivity(law, T, settings, Phi_f)
      1. IF the law has one branch and Q == 0 THEN return D0 exactly (no exp, no log).   [DESIGN]
         why: a constant layer then reproduces the v1.0 number bit for bit (regression test VT16).
      2. logs = [ln_branch(law, m, T, ...) for every branch m]
      3. top = maximum of logs
      4. D = exp(top) * sum over m of exp(logs[m] - top)                             (P22.1)
         why: log-sum-exp keeps every term finite; the largest term is exactly 1.
      5. IF a window exists and T is outside it THEN record a warning (once per run, with layer
         and temperature range). Do not clamp and do not stop.
      6. IF D < D_floor THEN D = D_floor and count one floor event; report the count at the end.
         why: the conductance divides by D, and the series uses sqrt(Lambda / D).
      7. return D.

C4  apparent_activation_energy(law, T, settings)
      1. logs as in C3 step 2; top = maximum; w[m] = exp(logs[m] - top).
      2. return sum over m of Q[m] * w[m] / sum over m of w[m].                      (P22.6)
         why: this is the D-weighted mean of the branch energies, the slope of ln D against 1/T.
              It moves from Q_1 to Q_2 as T rises, and it sets the step limit of block F.

C5  crossover(law)      -- diagnostic only, two-branch laws
      return (Q[2] - Q[1]) / (Rg * ln(D0[2] / D0[1])).                               (P22.7)
```

**Checks already made on these steps (Python).** Bad inputs (D0 equal to zero, D0 negative, Q negative, Q not a number, missing Q tag, $T=0\ \mathrm{K}$, $T=-300\ ^\circ\mathrm{C}$, missing temperature tag) are rejected and $Q=0$ is accepted (`b4_arrhenius_checks.py`). $D(T)$ by the log-sum-exp rule agrees with 40-digit arithmetic for four register laws at five temperatures, worst relative difference $8.4\times10^{-15}$. For Cs in SiC (FRG, $\Phi_f=0$, register rows A3-01 and A3-02) the crossover is 1772.4 K and $Q_\mathrm{eff}$ is 130.10 and 438.22 $\mathrm{kJ\ mol^{-1}}$ at 1250 and 1600 $^\circ$C.

**Note on the gas constant (DESIGN, decision 11).** The register stores the $R_g$ used by each source row (8.3143 for the TECDOC-978 rows, 8.3145 for the Collin et al. (2016) and IAEA (2012) rows; the BISON rows from Hales et al. (2021) print no value, while Collin et al. (2021) give 8.3145 for BISON, register RG-5). The default is the SI value for every row, with `FIXED_8_3143` as the option for four-digit comparison with TECDOC-1674. CRP-6 case 1a (bare kernel, $r=250\ \mu\mathrm{m}$, 200 h at 1200 $^\circ$C, two-branch Cs in UO$_2$) gives $F=0.467337$ with 8.3143 and $F=0.467404$ with the SI value; only the first rounds to the printed 0.4673. Which constant the CRP-6 participants used is NOT VERIFIED.

### 4.5 Block D: temperature history

**Origin.** Temperature part, T2, with one new step (D6) found in the cross-check of Section 8.

**In plain words.** The history says what temperature the whole particle has at each moment (assumption A1: the temperature is uniform in the particle). Two kinds are needed.

- **Linear kind.** A table of breakpoints $(t_k,T_k)$ with strictly increasing times. Between two breakpoints the temperature changes linearly, which reproduces furnace ramps quoted in degrees per hour. A hold is two breakpoints with the same temperature.
- **Constant-interval kind.** A list of intervals, each with one temperature. This is the form needed for a sudden temperature step (CRP-6 case 3c, 200 h at 1600 $^\circ$C and then 200 h at 1800 $^\circ$C) and for the exact projection of block R3.

Keeping the two kinds apart is a DESIGN choice (decision 9), because the approved text requires strictly increasing times and a jump cannot be written as two breakpoints at one time. A schedule that mixes ramps and jumps is not needed by any benchmark in the corpus and is left out.

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P22.21) | linear interpolation of $T$ between breakpoints | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.39) | $T(t)=T_k$ on the interval $[t_k,t_{k+1})$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.58) | segment length from a quoted ramp rate, or a hold time | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.59) | arrival and end times from a table of (temperature, time to reach, time at) | APPROVED | DERIVED NOT IMPLEMENTED |

Table: 4.4 Equations of block D.

```text
HistoryLinear   = { times_s : list, temps_K : list }          -- same length >= 2, times_s[1] = 0
HistoryConstant = { starts_s : list, temps_K : list, end_s }  -- starts_s[1] = 0, temps_K[k] holds on [starts_s[k], starts_s[k+1])
Schedule_item   = RAMP(T_target, rate)  |  HOLD(duration)     -- input of the segment builder
Triple          = { T_degC, time_to_reach_h, time_at_h }      -- CRP-6 table style
```

```text
BLOCK D  temperature history                         planned module: src/thermal/temperature_history.rs
INPUT   a schedule in one of the forms above, with unit tags
OUTPUT  a History with the functions D5 to D8                                            [NEW]

D1  build_linear_from_segments(T_start, items, units)
      1. t = 0; T = T_start (converted by step A8); points = [(0, T)].
      2. FOR each item in order:
           RAMP(T_target, rate): require rate > 0 (the sign is carried by T_target - T);
                                  t = t + |T_target - T| / rate;  T = T_target;
           HOLD(duration):        require duration > 0; t = t + duration.
           append (t, T) to points.                                           (P22.58) APPROVED
         why: the length of a segment is its temperature change divided by the quoted rate.
              Rates in degC per h and K per h are equal numbers, so only the time unit needs converting.
      3. convert all times to seconds and return HistoryLinear(points).
      Example: the AGR-1 plan at 1600 degC (ASSUMPTION: start at 30 degC; the safety-test procedure
      in register rows TH-001 onward does not state the first two ramp rates; the register gives
      120 degC per h from its other source) totals 333.78 h with the plateau starting at 31.17 h, as
      printed (334 h). The 1700 and 1800 plans total 335.95 h and 338.12 h.

D2  build_linear_from_triples(triples)
      1. read "-" as 0 for both durations; take triples[1].T as the start value at t = 0.
      2. t_arr[k] = t_end[k-1] + time_to_reach[k];  t_end[k] = t_arr[k] + time_at[k];
         append (t_arr[k], T[k]) and (t_end[k], T[k]).                          (P22.59) APPROVED
         why: the table lists the time to arrive at each temperature and the time spent there.
      3. IF two consecutive points would have equal times but different temperatures THEN stop with
         "instantaneous change: use the constant-interval kind".
      4. [OPEN] The ramp between listed temperatures is taken as linear. TECDOC-1674 does not state
         the shape (NOT VERIFIED). The summed durations reproduce all nine printed totals.
      5. [OPEN] Row 8a "(1550, 6.5, -), (300, 1, -)" must be re-read from the page image before use.

D3  build_constant(starts_h, temps, end_h, units)
      convert to seconds and kelvin; require starts strictly increasing, starts[1] = 0, end > last start;
      return HistoryConstant.                                                  (P22.39) APPROVED

D4  validate(h)
      LINEAR: at least two points, all values finite, times[1] = 0, times strictly increasing, every T > 0 K.
      CONSTANT: starts strictly increasing, every T > 0 K, end_s > last start.
      The end time of the history is times[last] (LINEAR) or end_s (CONSTANT).

D5  temperature_at(h, t)                    -- value at the START of a step or at an output time
      1. require 0 <= t <= end; never extrapolate.
      2. k = largest index with times[k] <= t (binary search).
      3. CONSTANT kind: return temps[k]  (right-continuous; at a jump the new value holds).
      4. LINEAR kind: IF t == times[k] THEN return temps[k] exactly.
         IF k is the last point THEN return temps[last].
         return temps[k] + (temps[k+1] - temps[k]) * (t - times[k]) / (times[k+1] - times[k]).   (P22.21) APPROVED
         why: the table value itself, not a recomputed one, so that T at a breakpoint is bit exact.

D6  temperature_at_step_end(h, t_old, t_new)   -- value at the END of a step that starts at t_old   [NEW, DESIGN]
      1. k = largest index with times[k] <= t_old.
      2. LINEAR kind: IF t_new == times[k+1] THEN return temps[k+1] exactly, ELSE return D5(h, t_new).
      3. CONSTANT kind: return temps[k]  (the value of the segment that contains the whole step).
      why: at a jump the right-continuous value of D5 at t_new belongs to the NEXT segment. A step that
           ends on the jump must be judged and computed with the old temperature. Using D5 there makes
           the step guard of block F see an impossible jump of ln D and halve the last step before every
           jump (the check script shows a shortest step of 5.6e-7 s and 66 steps instead of 36 steps).
           The LINEAR kind is continuous, so both functions agree there.

D7  slope_after(h, t)                       -- right-hand slope, used by block F
      LINEAR: k as in D5 (for t == end return 0); return (temps[k+1] - temps[k]) / (times[k+1] - times[k]).
      CONSTANT: return 0.
      why: at a breakpoint the slope jumps; the step that starts there lives on the new segment.

D8  events(h, other_flags)
      1. LINEAR: every breakpoint time > 0 gets the flag BREAK.
         CONSTANT: every start time > 0 gets the flag JUMP.
      2. block F adds the flags SWITCH (from block E), OUTPUT and END.
      3. require each time to lie in (0, end]; merge duplicates by exact float equality (flags are united); sort ascending.
         why: the event list is a set. Listing a breakpoint twice then cannot change the step sequence.
```

For the constant-interval kind, `temperature_at` returns `temps[k]` with `k` the largest index with `starts[k] <= t`, so the temperature is right-continuous and the slope is zero.

**Checks already made on these steps.** AGR-1 plans and ramps: totals 333.78 h, 335.95 h and 338.12 h; plateau start 31.17 h; `T(t)` reproduces every breakpoint value; the last segment ends at 30 $^\circ$C. The CRP-6 conversion of the nine printed lists reproduces the printed totals (`sec5_schedules.py`, 17 of 17 PASS).

### 4.6 Block E: yield and source history

**Origin.** Decay part, block B.

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P21.8), (P21.52) | $S_K=y\dot F/N_A$, kernel only | APPROVED (unit check) | DERIVED NOT IMPLEMENTED |
| (P21.10) | mixed yield $y=\sum_qf_qy_q$ for several fissile nuclides | APPROVED (a definition, no separate B3 row) | DERIVED NOT IMPLEMENTED |
| (P21.11), (P21.12) | precursor lag, $t_{99}=\ln 100/\lambda_p=6.644\,t_{1/2,p}$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.57), option A | exact step average of the source | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.9) | $\dot F=\beta n_\mathrm{HM}/t_\mathrm{irr}$ from burnup | time average APPROVED; $n_\mathrm{HM}$ NOT VERIFIED | OPEN |
| (P21.13) to (P21.16) | activation products | equations APPROVED; all data NOT VERIFIED | FUTURE |

Table: 4.5 Equations of block E.

**Independent or cumulative yield: the choice rule.** The independent yield $y^I$ counts atoms formed directly in the fission. The cumulative yield $y^C$ also counts the atoms that arrive later by decay of the precursors. STRATO v2.0 carries one species at a time, so the precursors are not tracked. Section 3.3.2 of the derivation gives the rule, written here as a decision list.

1. If the nuclide is produced by neutron capture on a stable fission product (Ag-110m, Cs-134, Eu-154), no fission-yield source exists for it. Stop with an error that names the missing activation data. This is the FUTURE route of step E5.
2. If the precursors are not carried as separate species (always true in v2.0), use $y^C$. The product is then born at the moment of fission instead of after the precursor chain has decayed.
3. If a later version carries the precursor as its own species, the daughter needs $y^I$ in its fission term. Using $y^C$ would count the precursor twice (Section 3.3.4, Eq. (P21.18)).
4. Report the precursor delay $t_{99}$ of Eq. (P21.12). If $t_{99}$ is longer than a set fraction of the time to the first output, warn that the source is early: the true birth $B(t)$ stays below $y^C\dot Ft$ by up to $1/\lambda_p$ in time.

| Product | Longest precursor | $t_{99}$ | Treatment in v2.0 | Warning if the first output is earlier than (factor 0.1) |
|---|---|---|---|---|
| Cs-137 | Xe-137, 3.818 min | 0.423 h | $y^C$ | 4.2 h |
| Sr-90 | Rb-90m, 258 s | 0.476 h | $y^C$ | 4.8 h |
| Kr-85 | Kr-85m, 4.48 h | 29.8 h | $y^C$ with a warning in most runs | 12.4 d |
| I-131 | Te-131m, 33.25 h | 9.2 d | $y^C$ with a warning in most runs; the delay is comparable to the I-131 half-life | 92 d |
| Cs-134 | Xe-133, 5.2475 d (precursor of the target Cs-133) | 34.9 d | refused, activation route (FUTURE) | not applicable |

Table: 4.6 Yield treatment by product. The $t_{99}$ values were recomputed from the half-lives by `b4_phase2_pseudocode_integration.py` (0.4228 h, 0.4761 h, 29.76 h, 9.20 d, 34.86 d); the last column is ten times $t_{99}$.

The warning threshold is a proposal (decision 5, open item O2). Section 3.3.2 says only that $y^C$ is valid when $t_{99}$ is short compared with the time step and the diffusion time.

```text
BLOCK E  yield and source history                    planned module: src/physics/fission_source.rs
INPUT   nuclide; fissile mix: list of (fissile nuclide, fraction f_q) with sum f_q = 1;
        fission rate density history: list of (start s, end s, rate in fissions per m3 per s),
          constant on each interval;
        yield table (atoms per fission) for each fissile nuclide;
        longest-precursor half-life (s) if known; warning factor (default 0.1)
OUTPUT  yield; source_kernel_i for each interval i (mol per m3 per s); the switch times; the function E3   [NEW]

E1  Choose the yield (decision list above).
      IF the nuclide is an activation product THEN stop (see E5).
      FOR each fissile nuclide q: read y_q = cumulative yield of the product.
      Check that the fractions f_q add to 1 within 1e-9.
      yield = sum over q of f_q * y_q.                                           (P21.10) APPROVED
      why: a reactor with U-235 and Pu-239 fissions has a yield between the two pure ones.
      IF a longest-precursor half-life is known THEN
          t99 = ln(100) * precursor_half_life_s / ln(2).                         (P21.12) APPROVED
          IF t99 > warning_factor * (time of the first output) THEN print a warning with t99.   [OPEN: factor]
E2  FOR each interval i of the fission rate history:
        source_kernel_i = yield * fission_rate_i / N_A.                          (P21.8, P21.52) APPROVED
    Units: (atoms per fission)(fissions m^-3 s^-1)/(atoms per mol) = mol m^-3 s^-1.
    IF any rate is negative or not finite THEN stop with an error.
    why: the source enters as a right-hand-side term; a negative value voids positivity (P21.62).
    The switch times are the starts and ends of the intervals; block F receives them with the flag SWITCH.
E3  Function source_step(t0, t1):
        source_step = ( sum over intervals i of source_kernel_i * overlap(i, [t0, t1]) ) / (t1 - t0),
        where overlap is the length of time in common.                           (P21.57, option A) APPROVED
    why: if a switch falls inside the step, this keeps the generated amount exact; taking the value at
         the end of the step (option R) would lose or add up to |jump| * kernel volume * dt (P21.79).
         In the PLANNED mode every switch is a step end, so no switch falls inside a step; the
         function stays exact for FIXED runs and for tests.
    dt in H3 is the t_new - t_old that block F computed once; E3 is called with the same two times.
E4  (Optional, for plots) record the precursor delay t99 from E1 in the output header.
E5  FUTURE, not part of v2.0.  Activation products (P21.13 to P21.16).
      A product A is made from a stable target P by neutron capture:
          source_A(r, t) = f_m * sigma_P * flux * C_P(r, t),
      so it exists wherever target atoms are, also in the coatings.
      Needed and not yet verified: capture cross section sigma_P, isomeric fraction f_m,
      burn-out cross section of A, flux history. The time stepping would carry two species,
      target first, then product with the target's new values as source. The time-discretisation
      choice for this coupling is OPEN. The closed modal form (P21.15, P21.16) applies when A and P
      share one diffusivity per layer.
```

**The burnup route.** Eq. (P21.9) turns burnup $\beta$ (FIMA) into a mean fission rate density only with the heavy-metal atom density $n_\mathrm{HM}$ of the kernel, which no audited source gives. A conversion from reactor power would need the energy released per fission, which the sections do not contain. Phase 2 therefore takes the fission rate density as a direct input (decision 2). Both conversions stay OPEN.

**Synthetic example.** For Cs-137 with $y^C=6.2208\times10^{-2}$ (JEFF-3.1.1, U-235, thermal; register Table 22) and a synthetic $\dot F=5.0\times10^{19}\ \mathrm{fissions\ m^{-3}\ s^{-1}}$, step E2 gives $S_K=5.1649\times10^{-6}\ \mathrm{mol\ m^{-3}\ s^{-1}}$ (recomputed from `fission_yields.csv` by the integration script). The rate density is a test value and not a STRATO input.

### 4.7 Block F: events and step planner

**Origin.** This block merges two drafts that each had their own step planner: block C1 of the decay part (a list of steps built from the decay and diffusion limits, growing geometrically after each boundary) and block T3 of the temperature part (steps chosen one at a time from the slope of $T(t)$, landing on each event). The two disagreed on the growth factor (1.5 against 2.0), on how the last step before an event is shaped (a common scale factor against halving the remainder), on whether a step restarts small after an event, and neither reproduced the v1.0 constant grid. The merged rules are DESIGN choices that keep the rules of both parts that the equations support (decision 10; mismatches X1, X2 and X7 in Section 8).

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P21.64), (P21.77) | Backward Euler decay error $e^{-z}-1/(1+z)=-z^2/2$; step rule $\Delta t\le2\varepsilon_\lambda/(\lambda^2t_\mathrm{end})$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.78) | slowest-mode error $\tfrac12(\Lambda_1+\lambda)^2t\Delta t$ | APPROVED for constant diffusivities; OPEN for a temperature history | DERIVED NOT IMPLEMENTED |
| (P21.79) and its rules | amount error at a switch inside a step; place a step end on every switch, output time and temperature knot; restart small after a switch; no stability limit | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.19), (P22.20) | $d\ln D/dt=Q_\mathrm{eff}\dot T/(R_gT^2)$; step bound $\varepsilon_DR_gT^2/(Q_\mathrm{eff}\lvert\dot T\rvert)$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.21) | linear $T(t)$ between breakpoints | APPROVED | DERIVED NOT IMPLEMENTED |

Table: 4.7 Equations of block F.

The temperature step bound is a resolution rule for the coefficients. Accuracy of the solution is checked separately, by comparing a run with steps $\Delta t$ and $\Delta t/2$. The guard of step F5.5 compares actual logarithms of the diffusivity:

$$
\max_{i}\,\bigl\lvert \ln D_i\bigl(T_\mathrm{end}\bigr) - \ln D_i\bigl(T_\mathrm{old}\bigr)\bigr\rvert \le 2\varepsilon_D
\tag{H22.1}
$$

Here $T_\mathrm{old}$ is the temperature at the start of the step and $T_\mathrm{end}$ the temperature at its end (step D6). Eq. (H22.1) is a DESIGN relation: (P22.20) uses the activation energy at the start of the step, a strongly curved law can change faster than predicted, and the factor 2 leaves room for that first-order estimate.

```text
BLOCK F  events and step planner                     planned modules: finite_volume.rs, temperature_history.rs
INPUT   t_end; history (block D) or none; switch times (block E); output times or output_every_steps;
        grid mode FIXED or PLANNED;
        FIXED: dt_request;
        PLANNED: dt_max; dt_first; growth (default 2.0); eps_D (default 0.1);
                 eps_lambda (default 1e-3); optional Lambda_1 (1/s) with eps_1; step_limit
OUTPUT  F1 to F4 once: the event list and the fixed step caps
        F5 for each step: t_old, t_new, dt, flags                                       [NEW]

F0  IF the grid mode is FIXED THEN
        n_steps = ceil(t_end / dt_request) ; dt = t_end / n_steps ;
        step k runs from (k-1)*dt to k*dt ; rows at every output_every_steps steps and at the last step.   [KEPT]
        Leave the block; steps F1 to F5 do not run.
    why: this is the v1.0 rule (lines 190, 191 and 221). Building the same grid by the geometric rule
         with growth 1 gives the same numbers only to round-off, and the regression cases V1 and VT16
         demand bit identity. Block A has already refused FIXED when an event time exists.
F1  Events.  Collect, each with its flags:
      BREAK or JUMP   from block D, step D8,
      SWITCH          every start and end of the source history (block E), where the rate changes,
      OUTPUT          every requested output time,
      END             t_end.
    Require every time in (0, t_end]. Merge equal times by exact float equality (union of the flags). Sort.
    why: a step that straddles a switch smears it over one step, and a step that straddles a kink of
         T(t) loses the smooth behaviour on which the error estimate rests. Ending steps on events
         costs nothing (rule 1 under P21.79; MOOSE does the same with force_step_every_function_point).
         The list is a set, so listing an event twice cannot change the step sequence.
F2  IF decay_constant > 0 THEN
        dt_decay = 2 * eps_lambda / (decay_constant^2 * t_end).                  (P21.77) APPROVED
    ELSE dt_decay = infinity.
    why: the Backward Euler decay factor 1/(1+z) is off from exp(-z) by z^2/2 per step, so the relative
         decay error after t_end is about decay_constant^2 * t_end * dt / 2. The limit does not apply
         at a steady state, where the fixed point is exact.
F3  IF Lambda_1 is given THEN
        dt_diff = 2 * eps_1 / ((Lambda_1 + decay_constant)^2 * t_end).           (P21.78, solved for dt)
    ELSE dt_diff = infinity.
    [Temperature history, note updated 2026-11-13 from Section 5.2.5 of the derivation; the step itself is
     unchanged. Use Lambda_1 at the highest temperature. With a shared scaling factor and option A this
     dt_diff is then a sufficient cap: the slow-mode error stays below eps_1 (PROVED, the F3 paragraph
     under (P22.72)), at about [(Lambda_max + decay_constant) * t_end / Theta_end]^2 times the fewest
     steps (1.11 in the ramp-and-hold test). For option E the lag L_1 enters ((P22.65), (P22.73)), so
     eps_1 is not proved for this value. Lambda_1 at the lowest temperature gives no bound. With
     layer-specific activation energies only the frozen-coefficient part of the error is bounded (P22.78;
     option A for any history, option E on heating only) and the remainder is OPEN, so there the value
     stays a warning. The cheaper rule (P22.69) with the step (P22.72) is proposed in
     Section 9.5 and is not applied here. Hard cap or warning: decision 6.]
    why: the slowest diffusion mode carries the largest time error; quantities near complete release
         (for example 1 - F) are the most sensitive.
F4  dt_cap = smallest of dt_max, dt_decay, dt_diff.
    IF t_end / dt_cap is larger than step_limit THEN stop and report which of the three limits controls it.
    why: Backward Euler has no stability limit; these are accuracy limits only (decision 6 asks
         whether the limits of F2 and F3 are hard caps or warnings).
    Set grow_cap = dt_first and event_pointer = 1.

F5  next_step(t_old, grow_cap, event_pointer)
      1. t_event = events[event_pointer].time          -- the next event strictly after t_old
      2. s = slope_after(history, t_old)                (0 if no history)                (block D, D7)
      3. IF s == 0 THEN dt_T = infinity
         ELSE T = temperature_at(history, t_old)
              Qmax = maximum over all layers i of apparent_activation_energy(law_i, T)           (block C, C4)
              dt_T = eps_D * Rg * T^2 / (Qmax * |s|).                               (P22.19, P22.20) APPROVED
         why: d(ln D)/dt = Q_eff * dT/dt / (Rg T^2); fixing the allowed change of ln D fixes the step.
              A hold has no limit from this rule, a steep cool-down has a short one.
              Example values (eps_D = 0.1; Cs in SiC): 1600 degC at 50 K/h gives 479 s;
              1250 degC at 120 K/h gives 445 s; 1600 degC at -600 K/h gives 40 s.
      4. dt = minimum of dt_cap, grow_cap, dt_T.
         why: the growth cap keeps the step from jumping by orders of magnitude when a slope changes.
      5. LOOP (at most 30 times):
           rem = t_event - t_old
           IF rem <= dt THEN t_new = t_event                     (this step lands on the event, by assignment)
           ELSE IF rem <= 2*dt THEN t_new = t_old + rem / 2      (halve the remainder)
           ELSE t_new = t_old + dt
           why: the middle case replaces "one full step plus a sliver" by two equal steps, so no step
                is much shorter than its neighbours and no step overshoots the event.
           Guard (H22.1): T_old = temperature_at(history, t_old);
                          T_end = temperature_at_step_end(history, t_old, t_new)             (block D, D6)
                          compare the layer logarithms of the diffusivity; skip the guard if every law is constant.
           IF the guard holds OR t_new - t_old < 1 microsecond THEN leave the loop
           ELSE dt = (t_new - t_old) / 2 and repeat; after 30 repeats stop with an error.
      6. dt_n = t_new - t_old, computed once by this single subtraction.
         why: the very same dt_n must enter the matrix, the source average and the release bookkeeping,
              otherwise the balance does not close.
      7. IF t_new == t_event (it was assigned, never computed by addition) THEN
             event_pointer = event_pointer + 1 ; flags = events[...].flags
             grow_cap = dt_first IF the flags contain SWITCH or JUMP, ELSE growth * dt_n
         ELSE flags = none ; grow_cap = growth * dt_n.
         why: after a switch or a jump the time derivatives jump (rule 2 under P21.79: restart small and
              grow geometrically); Backward Euler is L-stable, so no oscillation follows. A kink of a
              linear history (BREAK) and an output time do not restart the step.
      8. RETURN t_new, dt_n, flags, grow_cap, event_pointer.
```

**Examples (calculated).** With $\varepsilon_\lambda=10^{-3}$ and 620 effective full-power days, Ag-110m ($\lambda=3.211198\times10^{-8}\ \mathrm{s^{-1}}$) gives $\Delta t\le3.62\times10^4\ \mathrm{s}$, about 10 h. I-131 over five mean lives gives $\Delta t\le400\ \mathrm{s}$. For Cs-137 the limit is far beyond any heating test. These agree with Section 4.6 of the derivation.

**Checks already made on these steps (Python).** On the AGR-1 1600 $^\circ$C plan (41 events; layers Cs in UO$_2$, constant buffer, PyC, Cs in SiC, PyC; $\varepsilon_D=0.1$, $\Delta t_\mathrm{max}=3600\ \mathrm{s}$, growth 2) the temperature-part rule gave 1826 steps, every event a step end by exact float equality, a largest change of $\ln D$ of 0.102, no halving by the guard, a shortest step of 1.24 s, and a step sequence that does not change when a breakpoint is listed twice (`b4_arrhenius_checks.py`). The merged rule was run by `b4_phase2_pseudocode_integration.py` on a ramp from 1000 to 1800 $^\circ$C in 4 h with a 4 h hold, a source switched off at 3 h and $\lambda=10^{-5}\ \mathrm{s^{-1}}$: 139 steps, all 8 events are step ends by float equality, the step after the switch restarts at $\Delta t_\mathrm{first}=5\ \mathrm{s}$, the largest change of $\ln D$ is 0.102 and the growth between steps never exceeds 2.

### 4.8 Block G: geometry once

**Origin.** Temperature part T4.1, and step B3 of the decay part (the cell factor).

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P21.53), (P21.54) | cell average of the source over the cell volume; for the kernel source: factor 1 inside the kernel, 0 outside, $(R_1^3-a^3)/(b^3-a^3)$ for a cell cut by $R_1$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.14) | geometry factors $A_f$, $\delta^\pm_f$ of the conductance | APPROVED | DERIVED NOT IMPLEMENTED |

Table: 4.8 Equations of block G.

```text
BLOCK G  geometry                                    planned module: src/numerical/finite_volume.rs (build_mesh split)
INPUT   regions: outer radius, cells; kernel radius R1 = outer radius of region 1
OUTPUT  per cell: volume, centroid, layer index, cell_factor; per face: area, dist_left, dist_right   [CHANGED]

G1  FOR each cell j (edges left, right; layer index = region):                              [KEPT]
        volume[j]   = (4 pi / 3) * (right^3 - left^3)
        centroid[j] = 0.75 * right                                       IF left == 0
                      0.75 * (right^4 - left^4) / (right^3 - left^3)     otherwise
G2  FOR each internal face f between cell j and j+1 (radius r_f = right edge of cell j):
        area[f]       = 4 pi r_f^2
        dist_left[f]  = r_f - centroid[j]
        dist_right[f] = centroid[j+1] - r_f
G3  Outer face: area_R = 4 pi R^2 ; dist_R = R - centroid[N]
G4  FOR each cell j: cell_factor[j] =
        1                                      IF right <= R1
        0                                      IF left >= R1
        (R1^3 - left^3) / (right^3 - left^3)   otherwise.                          (P21.54) APPROVED
    On the v1.0 mesh every layer boundary is a cell face, so the third case never occurs.
    IF it occurs anyway THEN keep the formula (it is exact) and print a note.
    why: the cell average, not the value at the cell centre, is what the balance needs. For the v1.0 mesh
         the sum of volume * cell_factor equals (4 pi / 3) R1^3 exactly (P21.54).
G5  Nothing here depends on D, lambda or T, so nothing here is recomputed inside the loop.
    why: v1.0 build_mesh fixes the conductances while it builds the mesh (lines 78 to 91); only the
         part that does not depend on D is kept here, and block H2 does the rest.
```

### 4.9 Block H: one time step

This is the heart of the loop. Four steps prepare the numbers; the fourth solves the system.

#### 4.9.1 What changes with respect to v1.0

The v1.0 function `backward_euler_step` ([lines 162 to 182](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L162-L182)) builds a tridiagonal system from three arrays of face conductances and one array `v`. Two lines change; the others stay.

| Item | v1.0.0 (lines 178 and 179) | Phase 2 |
|---|---|---|
| diagonal excess `v[i]` | `volume / dt` | `volume / dt + lambda * volume` |
| right-hand side `rhs[i]` | `v[i] * old[i]` | `(volume / dt) * old[i] + volume * source_cell[i]` |
| face conductances | `conductance`, `outer_conductance` of the mesh | recomputed in H2 for each step |
| solver (subtraction-free elimination) | lines 132 to 160 | unchanged |
| layout of the full matrix row | $-g^W_jC_{j-1}+(v_j+g^W_j+g^E_j)C_j-g^E_jC_{j+1}$ | same form with the new $v_j$ |

Table: 4.9 What changes in one Backward Euler step.

Two traps follow from Section 4.3 of the derivation.

- After `v` is redefined, the right-hand side must still use $V_j/\Delta t$. Reusing the enlarged `v[i]` as in line 179 would multiply the old inventory by $1+z$ and the run would lose inventory without any decay having been asked for.
- Write the diagonal as `volume / dt + lambda * volume`. With $\lambda=0$ the second term is exactly 0.0 and the result is bit-identical to v1.0. The algebraically equal `volume * (1/dt + lambda)` differs from `volume / dt` in the last bit for 13 of the 40 cells of the test mesh and would break the regression test V1. Rust does not fuse a multiply and an add unless the program asks for it (`mul_add`); the Rust test V1 is what confirms this for the real compiler.

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P21.55) | semi-discrete cell balance with decay and source | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.58), (P21.59) | Backward Euler system; diagonal $V_j(1/\Delta t+\lambda)$, right-hand side $(V_j/\Delta t)C_j^n+V_j\bar S_j^{n+1}$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.60), (P21.61) | diagonal excess and subtraction-free pivots stay valid | APPROVED | v1.0 solver IMPLEMENTED, [lines 132 to 160](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L132-L160) |
| (P21.62), (P22.17) | positivity for every $\Delta t>0$, $\lambda\ge0$ and every temperature | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.63) | link to the v1.0 step: $(1+z)^{-1}$ times the v1.0 step with $\Delta t'=\Delta t/(1+z)$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.80) | decay factor $c_\lambda=e^{-\lambda t}c_0$, valid also for a diffusivity that changes in time | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.13) to (P22.15) | semi-discrete and Backward Euler form with time-dependent conductances | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.18) | difference between the end-of-step and step-average choices | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.22), (P22.23) | integral of an Arrhenius term over a linear ramp, exponential integral $E_2$ | APPROVED | DERIVED NOT IMPLEMENTED |

Table: 4.10 Equations of block H.

**Decay and a changing diffusivity.** Decay enters only the diagonal term $\lambda V_j$ of Eq. (P21.59), so block H needs no change when the conductances are rebuilt in every step. The reason is Eq. (P21.80): for a source-free problem the solution with decay is $e^{-\lambda t}$ times the solution without decay, and the proof of that statement uses only the linearity of the spatial operator, so it holds when $D_i=D_i(t)$. This is also why block R2 can split off the factor $e^{-\lambda t}$ before the clock change. The identity needs one decay constant for the whole particle, constant in time (the same restriction as condition E3 of block R2).

#### 4.9.2 Which diffusivity does a step use

Two ways exist to pick the diffusivity of a step.

- **Option E, end of step:** $D_i$ evaluated at the temperature at the end of the step (step D6). This is the textbook fully implicit choice and the one named in the task brief.
- **Option A, step average:** $\bar D_i=\frac{1}{\Delta t_n}\int_{t_n}^{t_{n+1}}D_i(T(t))\,dt$. The approved text (Section 5.2.3) recommends this option, because for a shared scaling factor and no decay it is exactly Backward Euler in the effective time, and because it stays exact across a kink of $T(t)$.

Both are first order in time. Neither is uniformly more accurate in the tests (A is 2.24 times better in the shared-$\alpha$ ramp, E is 1.75 times better in the layer-specific test). The default here is A and E stays selectable (decision 8). In the joined check run (ramp from 1000 to 1800 $^\circ$C, decay, switched source) the released amounts differ by 1.4 % between the two options at the planner's step sizes, which shows that the choice matters at these steps. That run has no reference solution, so it does not say which option is closer to the truth.

```text
BLOCK H  one time step                               planned module: src/numerical/finite_volume.rs (extended), diffusivity.rs
INPUT   t_old, t_new, dt (from block F); conc[j] (old); decay_constant; laws; history; option A or E;
        geometry and cell_factor (block G); source_history (block E)
OUTPUT  conc_new[j]; and for block J: G_R, gen_rate                                       [CHANGED]

H1  step_diffusivity(t_old, t_new)  ->  D_layer[1..5]
      1. IF no history is given THEN D_layer[i] = diffusivity(law_i, T_const)  and leave the step.
      2. require [t_old, t_new] to lie inside one history segment (block F guarantees it).
      3. Option E: T_end = temperature_at_step_end(history, t_old, t_new)                  (block D, D6)
                   D_layer[i] = diffusivity(law_i, T_end).
         why: the left-limit value; the right-continuous value belongs to the next segment.
      4. Option A:
           IF the segment slope is 0 THEN D_layer[i] = diffusivity(law_i, temperature_at(history, t_old)); no quadrature.
           ELSE FOR each layer i and each branch m:
                arg_branch = (Q[m] / Rg) * | 1/T_end - 1/T_old |
                IF arg_branch <= 1 THEN Dbar_m = 8-point Gauss-Legendre average of the branch over [t_old, t_new]
                ELSE Dbar_m = (1/(t_new - t_old)) * D0_eff * (1/s) * [ T * E2(theta/T) ] from T_old to T_end,
                              with theta = Q[m]/Rg, s = the slope                               (P22.22) APPROVED
                D_layer[i] = sum over m of Dbar_m ; then apply the floor of step C3.6.
         why: the average of a sum is the sum of the averages. Eight-point Gauss-Legendre has a relative
              error below 1e-15 while arg_branch <= 1 (checked: 4.8e-16 at a change of 2 in ln D, 1.9e-13
              at 4, 5e-9 at 8). The closed form keeps full accuracy for large changes (2e-15 or better from
              0.5 upward; 2.3e-15 at a change of 16 in the integration check) but loses digits on pieces
              below about 1e-3 K (relative error 2.6e-11 at 1e-3 K, 8e-9 at 1e-5 K). Steps from block F
              change ln D by at most 2 * eps_D, so Gauss-Legendre is the normal path.
         E2(x) = exp(-x) - x * E1(x), E1 the exponential integral of order 1.                (P22.23) APPROVED
      5. RETURN D_layer.

H2  conductances(D_layer)       -- O(N) per step
      FOR each internal face f between layer a (left) and layer b (right):
          g[f] = area[f] / ( dist_left[f] / D_layer[a] + dist_right[f] / D_layer[b] )        (P22.14) APPROVED
      G_R   = area_R * D_layer[5] / dist_R                                                  (P22.14) APPROVED
      west_g[j] = 0 for j = 1, g[j-1] otherwise ;  east_g[j] = g[j] for j < N, G_R for j = N.
      why: two half-cell resistances in series, as in v1.0. A face inside one layer uses the same D on both sides.
           A constant temperature needs this once; the planned code may reuse the arrays while D_layer is unchanged.

H3  source_for_step(t_old, t_new, dt)  ->  source_cell[j], gen_rate
      1. source_step = block E, function E3, called with t_old and t_new.                   (P21.57, option A)
      2. source_cell[j] = source_step * cell_factor[j].
      3. gen_rate = sum over j of volume[j] * source_cell[j].
      why: for the v1.0 mesh gen_rate equals (4 pi / 3) R1^3 * source_step exactly (P21.54).
           IF there is no source, source_cell[j] = 0 for every j.

H4  one Backward Euler step with decay and source
      1. Check: dt finite and > 0; decay_constant finite and >= 0; every array has N entries;
         every conc_old[j] and source_cell[j] >= 0.
         (A negative source is allowed only in a manufactured-solution test, with an explicit switch.)
         why: positivity (P21.62) needs a nonnegative old state and source.
      2. FOR each cell j:
             diag_excess[j] = volume[j] / dt + decay_constant * volume[j]        [CHANGED] (P21.59)
             rhs[j]         = (volume[j] / dt) * conc_old[j] + volume[j] * source_cell[j]   [CHANGED] (P21.59)
         why: the first term of rhs is the old inventory of the cell divided by dt, the second is the atoms
              born in the cell per second; decay acts like an extra conductance from every cell to a
              reservoir that stays at zero concentration, so it only enlarges the diagonal excess (P21.60).
      3. Run the v1.0 elimination on (diag_excess, west_g, east_g, rhs).             [KEPT] (P21.61)
         Forward sweep, for j = 1..N:
             pivot_e[j] = diag_excess[1]                       IF j = 1
             pivot_e[j] = diag_excess[j] + west_g[j]*pivot_e[j-1] / (pivot_e[j-1] + west_g[j])   otherwise
             den[j]     = pivot_e[j] + east_g[j]
             carry[j]   = west_g[j] * partial[j-1]             (0 for j = 1)
             partial[j] = (rhs[j] + carry[j]) / den[j]
         Back substitution: conc_new[N] = partial[N];
             conc_new[j] = partial[j] + east_g[j] / den[j] * conc_new[j+1], for j = N-1 down to 1.
         why: this form contains only sums, products and quotients of nonnegative numbers. The textbook
              pivot subtracts two nearly equal numbers when diffusion is fast. Decay only makes every pivot
              larger, so the proof of v1.0 holds (P21.61). With conductances that change from step to step
              the matrix is still an M-matrix (strictly dominant diagonal, non-positive off-diagonals).  (P22.17)
      4. RETURN conc_new.
         Property: conc_new[j] >= 0 for every dt > 0, decay_constant >= 0 and every temperature, in exact
         arithmetic and in floating point, because no step subtracts (P21.62, P22.17).
```

**Worked scalar example (calculated).** Take one cell without diffusion, $C^n=1$, $S=10^{-7}\ \mathrm{mol\ m^{-3}\ s^{-1}}$, $\lambda=10^{-6}\ \mathrm{s^{-1}}$ and $\Delta t=10^5\ \mathrm{s}$, so $z=0.1$. Step H4 gives $C^{n+1}=(C^n+\Delta tS)/(1+z)=0.9181818$. The exact solution of $dC/dt=-\lambda C+S$ is $C^ne^{-z}+(S/\lambda)(1-e^{-z})=0.9143537$. The difference of $+3.8\times10^{-3}$ is the local error of this single step. It is of order $z^2$, as Eq. (P21.64) predicts for the decay factor (to leading order it is $(C^n-S/\lambda)z^2/2$, here $4.5\times10^{-3}$, with the higher terms making up the rest), so halving $\Delta t$ divides the one-step difference by about 4 (3.7 in the B7 check). Over a fixed time interval the number of steps doubles as well, so the accumulated error falls in proportion to $\Delta t$: the scheme is first order.

**Why the unsplit step is kept.** The alternative of taking a v1.0 step and then multiplying by $e^{-z}$ (Eq. (P21.66)) is exact in the decay but its steady state is shifted by the factor $\varphi(z)=ze^z/(e^z-1)$ in the diffusion term (Eq. (P21.68)). Its steady release-to-birth ratio (balance reading) is wrong by up to +127 % in the test cases of Section 4.4 of the derivation, depending on $\lambda/\Lambda_1$ and $z$. The unsplit step keeps the exact semi-discrete steady state for any $\Delta t$ (checked: a step of any size from $10^3$ to $10^{12}\ \mathrm{s}$ changes the steady profile by less than $10^{-15}$ relative). The split form stays in the plan as a cross-check for $S=0$ (case V15).

**Relation to the effective time.** For a shared scaling factor and $\lambda=0$, option A equals Backward Euler in the effective time with step $\Delta\tau_n=\int_{t_n}^{t_{n+1}}\alpha\,dt$, because every conductance is then $\Delta\tau_n/\Delta t_n$ times its constant-temperature value. The integration check finds a relative difference of $4\times10^{-16}$ in the inventory after a ramp and hold with five layers. For $\lambda>0$ the decay stays inside the implicit operator and the two differ at first order ($2.7\times10^{-3}$ after 30 steps in the B3 check), so no test may demand equality there.

### 4.10 Block J: bookkeeping

**Origin.** Decay part, block D, and step T4.5 of the temperature part.

**Release is counted two ways.** $M_\mathrm{rel}$ is every atom that has ever crossed the surface. $M_\mathrm{out}$ is the part that is still undecayed outside. They agree only for $\lambda=0$ (Eq. (P21.76)). A comparison with a benchmark number has to say which one the benchmark means. TECDOC-1674 Eq. (7.8) is $F_B=M_\mathrm{out}/(M+M_\mathrm{out})$ (Section 3.7.3 of the derivation), and the decay-corrected heating-test fraction is $F_\mathrm{corr}=e^{\lambda t}M_\mathrm{out}/M_0$.

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P21.40) to (P21.42) | $dM/dt=\mathrm{Gen}-\lambda M-J_R$; closure $M_0+M_\mathrm{gen}=M+M_\mathrm{dec}+M_\mathrm{rel}$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.43), (P21.44) | atoms outside the particle: $M_\mathrm{out}'=J_R-\lambda M_\mathrm{out}$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.45) | release fractions $F_\mathrm{rel}$, $F_\mathrm{corr}$, $F_\mathrm{gen}$, $R/B$, $F_B$, $F_\mathrm{post}$ | APPROVED (definitions) | DERIVED NOT IMPLEMENTED |
| (P21.70), (P21.71) | per-step identity $M^{n+1}-M^n=\Delta t(\mathrm{Gen}^{n+1}-\lambda M^{n+1}-J_R^{n+1})$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.72) | summed closure with $M_\mathrm{gen}$, $M_\mathrm{dec}$, $M_\mathrm{rel}$ | APPROVED | DERIVED NOT IMPLEMENTED; v1.0 residual IMPLEMENTED ([line 220](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L220)) |
| (P21.73), (P21.75), (P21.76) | Backward Euler update of $M_\mathrm{out}$; closure with $M_\mathrm{dec,out}$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.74) | exponential update of $M_\mathrm{out}$ | APPROVED | optional (decision 4) |
| (P22.16) | summed step identity with the $G_R$ of the step | APPROVED | DERIVED NOT IMPLEMENTED |

Table: 4.11 Equations of block J.

v1.0 stores the inventory, one cumulative release sum with Kahan compensation ([lines 211 to 217](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L211-L217)) and the residual $M+M_\mathrm{rel}-M_0$. It rejects a run whose initial inventory is not positive ([lines 186 to 189](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L186-L189)); that check has to go, because irradiation from zero is a case with $M_0=0$, and `release_fraction = released / initial` has to be guarded against a zero denominator.

```text
BLOCK J  bookkeeping                                 planned module: src/physics/inventory.rs
INPUT   conc_new[j], G_R, gen_rate, dt (all from the same step); volumes; decay_constant
OUTPUT  running totals and residuals                                                      [CHANGED]

J0  Once, before the loop:
      inv0 = sum over j of volume[j] * conc[j].                                           [KEPT]
      gen_total = dec_total = rel_total = dec_out_total = 0   (each a Kahan compensated sum);
      out_now = 0.
      IF inv0 = 0 AND the source is zero everywhere THEN stop: there is nothing to follow.
      why: v1.0 stops whenever inv0 is not positive; v2.0 only stops when nothing is in the particle
           and nothing will be made.
J1  For each step:
      1. inv = sum over j of volume[j] * conc_new[j].                                     [KEPT]
      2. release_rate = G_R * conc_new[N], with the G_R that block H2 used in this step.   (P21.71, P22.16) [CHANGED]
         why: v1.0 line 210 reads the fixed mesh.outer_conductance. That is correct for a constant D only.
              With a changing D the release must use the G_R of the step, or the balance fails (the
              negative test in Section 7 gives a residual of 1.2e-5).
      3. Add to the compensated sums (Kahan, as v1.0 does for the release):
             gen_total += dt * gen_rate                                       (P21.72)  [NEW]
             dec_total += dt * decay_constant * inv                           (P21.72)  [NEW]
             rel_total += dt * release_rate                                   (P21.72, P22.16)  [KEPT]
         why: each increment is the term that appears in the per-step identity (P21.70), so the summed
              identity (P21.72) closes to round-off.
      4. Atoms outside the particle:
             out_now = (out_now + dt * release_rate) / (1 + decay_constant * dt)      (P21.73) [NEW]
             dec_out_total += dt * decay_constant * out_now                           (P21.75) [NEW]
         why: the outside inventory obeys M_out' = J_R - lambda M_out; this is the Backward Euler form of
              that law, the same rule that is used inside, so the closure below holds with intrinsically
              defined terms.
         Optional report value (only if outside decay needs better than first order; decision 4):
             out_exp = exp(-decay_constant*dt) * out_exp_old
                       + release_rate * dt * phi1(decay_constant*dt)                  (P21.74, H21.1)
         (phi1 is defined in Eq. (H21.1) of Section 5.1.) In that case dec_out_total must be taken as
         the remainder out_old + dt*release_rate - out_new, otherwise the closure does not hold.
      5. Residuals, relative to the total supply (inv0 + gen_total):
             res_inside  = (inv0 + gen_total) - (inv + dec_total + rel_total)                   (P21.72)
             res_outside = (inv0 + gen_total) - (inv + out_now + dec_total + dec_out_total)     (P21.75)
         IF the relative size of either exceeds 1e-12 THEN report a closure failure.   [CHANGED]
         why: the criterion is that of v1.0. The residual tests assembly and solver, and stays at
              round-off for any dt, so it does not measure time accuracy.
      6. Positivity: minimum of conc_new[j] over j. IF it is below 0.0 THEN report a failure.     (P21.62, P22.17)
         why: the proofs say the minimum is at least 0.0 exactly; the v1.0 unit test accepts -1e-13 (line 310), which is looser than needed.
      7. IF t_new is an output time THEN store inv, the totals, release_rate, gen_rate and the residuals for block K.
J2  Fractions for the output rows (P21.45), with t the output time:
      source-free heating:        F_rel  = rel_total / inv0
                                  F_corr = exp(decay_constant * t) * out_now / inv0
                                  (skip F_corr when decay_constant * t > 700, where the exponential overflows)
      irradiation from zero:      F_gen  = rel_total / gen_total
                                  R_over_B = release_rate / gen_rate
                                  F_B    = out_now / (inv + out_now)
      heating after irradiation:  F_post = (rel_total - rel_total_at_t_irr) / inv_at_t_irr
                                  (store rel_total and inv at the step that ends on t_irr)
      An entry is left empty when its denominator is 0.
    why: decay-corrected and uncorrected fractions answer different questions (Section 3.6 of the
         derivation). F_B equals M_out/(M + M_out) and is not M_rel/M_gen.
```

**Remarks on block J.**

- With a Backward Euler update the decay-corrected fraction $F_\mathrm{corr}$ is slightly off the decay-free fraction and can exceed 1 by a first-order amount. In the check it is 1.000796 on the first mesh and 1.000398 on the next for a decay-free value close to 1. This excess is the Backward Euler decay error of Eq. (P21.77); the sign of the scheme is correct.
- The residuals of step J1.5 are insensitive to the step size. A good closure is necessary and says nothing about accuracy in time. Accuracy comes from the comparisons of Section 7.
- When the source is switched off at $t_\mathrm{irr}$, the profile at the end of the step that ends on $t_\mathrm{irr}$ is the starting profile of the heating phase (Eq. (P21.23)). Nothing is re-projected; the run continues with `source_cell = 0`.
- Summing the step equations over $j$ cancels every interior face and leaves exactly the three terms of step J1.3 (Eq. (P22.16)). That is why $G_R$, $dt$ and $\bar S$ must be the ones used in the matrix.

### 4.11 Block K: output rows and the end-of-run report

```text
BLOCK K  output                                      planned modules: src/physics/inventory.rs, src/main.rs
INPUT   stored values from block J
OUTPUT  the output table and the report                                                  [CHANGED]

K1  At an output time write one row with: t, T (if a history exists), inv, gen_total, dec_total,
    rel_total, out_now, dec_out_total, release_rate, gen_rate, res_inside, res_outside,
    minimum of conc, and the fractions of J2.  Leave an entry empty when its denominator is 0.
    why: v1.0 writes six columns (inventory, cumulative release, release fraction, release rate,
         balance residual, minimum concentration); the new columns are the ones the new bookkeeping needs.
K2  End-of-run report:
      - largest |res_inside| and |res_outside| over the run and the step where they occurred;
      - smallest concentration over the run;
      - number of steps, the cap that controlled the step size (dt_max, dt_decay, dt_diff or dt_T),
        shortest and longest step;
      - number of D-floor events (step C3.6) and every window warning (step C3.5);
      - the precursor warning of step E1, if any;
      - for an effective-time or modal reference: the covered eigenvalue and the first valid time.
    why: a run that closes but was cut off by a floor or a window must say so.
```

## 5 Reference solutions (outside the finite-volume loop)

These blocks give numbers that the finite-volume run is compared with. None of them runs inside the loop of block M.

### 5.1 Block R1: modal solution with decay and a piecewise-constant source

**Origin.** Decay part, block E.

**Scope.** R1 gives the semi-analytical series for the case that v1.0 already treats: a source and an initial inventory that are uniform in the kernel, constant diffusivities, an absorbing surface. It needs only the eigenvalues $\Lambda_n$ and the weights $w_n$ that the v1.0 `ModalRelease` already computes ([src/analytical/sturm_liouville.rs, lines 308 to 395](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L308-L395); weight formula at [line 352](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L352)). For a general source or initial profile the block also needs the norms $N_n$ and the moments $I_n$, $I^{(1)}_n$; that is the FUTURE extension of step R1.1.

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P21.24) to (P21.26) | eigenproblem and orthogonality; decay does not change it | APPROVED | v1.0 eigenvalues IMPLEMENTED |
| (P21.28) to (P21.30) | modal ODE $da_n/dt=-(\Lambda_n+\lambda)a_n+s_n$; kernel-uniform source | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.31) to (P21.34) | solution for any, constant and piecewise-constant source | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.35), (P21.37), (P21.38) | inventory, direct flux series, direct release series | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.39) | balance form $M_\mathrm{rel}=M_0+M_\mathrm{gen}-M-M_\mathrm{dec}$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.44) | $M+M_\mathrm{out}=M_0e^{-\lambda t}+\int Ge^{-\lambda(t-s)}ds$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P21.30), (P21.46) | $4\pi a_n(0)I_n=M_0w_n$ and, for the kernel-uniform source, $4\pi s_nI_n=\mathrm{Gen}\,w_n$ | APPROVED | v1.0 weights IMPLEMENTED |
| (P21.48), (P21.49) | single-sphere Booth forms used as reference | APPROVED | reference only |
| (H21.1) to (H21.3) | stable evaluation of the interval factors | HELPER, proved in the check script | helper |

Table: 5.1 Equations of block R1.

**Helper identities.** Let $\kappa_n=\Lambda_n+\lambda$ (`mode_kappa`), $x=\kappa_n\Delta$ (`arg_mode`), and define the two interval factors

$$
\varphi_1(x)=\frac{1-e^{-x}}{x},\qquad \varphi_2(x)=\frac{x-1+e^{-x}}{x^2}
\tag{H21.1}
$$

For the modal inventory $m_n=4\pi I_na_n$ with the kernel source $\mathrm{Gen}$ constant on an interval of length $\Delta$, Eqs. (P21.33) and (P21.38) can be written

$$
\begin{aligned}
m_n(t_j)&=m_n(t_{j-1})e^{-\kappa_n\Delta}+\mathrm{Gen}\,w_n\Delta\,\varphi_1(\kappa_n\Delta),\\
\int_{t_{j-1}}^{t_j}m_n\,ds&=\Delta\Big[m_n(t_{j-1})\varphi_1(\kappa_n\Delta)+\mathrm{Gen}\,w_n\Delta\,\varphi_2(\kappa_n\Delta)\Big]
\end{aligned}
\tag{H21.2}
$$

and the surviving total $X=M+M_\mathrm{out}$ of Eq. (P21.44) obeys

$$
X(t_j)=X(t_{j-1})e^{-\lambda\Delta}+\mathrm{Gen}\,\Delta\,\varphi_1(\lambda\Delta)
\tag{H21.3}
$$

Both $\varphi_1$ and $\varphi_2$ tend to 1 and 1/2 for $x\to0$, so a run with $\lambda=0$ needs no special case. Written naively, $\varphi_2$ loses digits for $x<0.1$ because it subtracts nearly equal numbers. The pseudocode therefore uses the series $\sum_k(-x)^k/(k+1)!$ and $\sum_k(-x)^k/(k+2)!$ for $x<0.5$ (14 terms) and the closed forms above that. The decay-part script compares both against 50-digit values at 400 points between $10^{-12}$ and $10^{3}$ with a largest relative error of $2.9\times10^{-16}$.

The identities (H21.2) follow from multiplying the modal ODE (P21.28) by $4\pi I_n$ and using $4\pi s_nI_n=\mathrm{Gen}\,w_n$; the first is Eq. (P21.33) and the second is the interval integral of it. Eq. (H21.3) is the same recursion for the single mode $\Lambda=0$ applied to Eq. (P21.44). This was checked here by substitution as well as by the script. These three identities are not rows of the B3 reconciliation, which is why they carry the word HELPER.

```text
BLOCK R1  modal solution with decay and source        planned module: src/analytical/modal_source.rs
INPUT   Phase 1 series: mode_rate[n] = Lambda_n (1/s), weight[n] = w_n, n = 1..n_modes
        (from the existing ModalRelease);
        decay_constant; initial kernel-uniform inventory inv0 (mol, may be 0);
        source schedule: list of (start s, end s, gen in mol per s), constant on each interval
          [gen = kernel volume * source_kernel from block E];
        output times
OUTPUT  at each output time: inv, dec_total, rel_total, gen_total, out_now, F_B, R_over_B (diagnostic)

R1.1  Initial modal inventories: mode_inv[n] = inv0 * weight[n].                 (P21.46) APPROVED
      why: the kernel-uniform start profile projects onto the modes with the v1.0 weights.
      FUTURE: for another start profile, project it with N_n and I_n (P21.27, P21.29).
R1.2  total_survivors = inv0 ; dec_total = 0 ; gen_total = 0 ; t = 0.
R1.3  Boundaries = sorted list of 0, every start and end of the source schedule, and every output time,
      without repeats.
      FOR each interval (t, t+dt) between neighbouring boundaries:
R1.3a   gen = the schedule value on this interval (0 where none is listed).
R1.3b   FOR each mode n:
            mode_kappa = mode_rate[n] + decay_constant ; arg_mode = mode_kappa * dt
            f1 = phi1(arg_mode) ; f2 = phi2(arg_mode)      (series or closed form, see above)
            area[n] = dt * ( mode_inv[n] * f1 + gen * weight[n] * dt * f2 )          (H21.2)
            mode_inv[n] = mode_inv[n] * exp(-arg_mode) + gen * weight[n] * dt * f1   (H21.2, P21.33)
        why: this is exact for a constant gen, so a stepwise power history carries no time error.
             A source that is switched off is the case gen = 0 (P21.34).
R1.3c   dec_total = dec_total + decay_constant * (sum over n of area[n]).            (P21.38, P21.41)
        gen_total = gen_total + gen * dt.                                           (P21.41)
        total_survivors = total_survivors * exp(-decay_constant * dt)
                          + gen * dt * phi1(decay_constant * dt).                   (H21.3, P21.44)
R1.4  At an output time:
        inv       = sum over n of mode_inv[n].                                      (P21.35)
        rel_total = inv0 + gen_total - inv - dec_total.                             (P21.39)
        out_now   = total_survivors - inv.                                          (P21.44)
        F_B       = out_now / total_survivors.                                      (P21.45)
      why: the direct series for the release, sum of Lambda_n * area[n], converges only like 1/N because
           the weights decay like 1/n^2. The balance form replaces the two slow sums by their exact
           limits inv0 and gen_total (P21.39).
R1.5  Diagnostics, never used for the release totals:
        flux_direct = sum over n of mode_rate[n] * mode_inv[n]            (P21.37)
        completeness_defect = 1 - sum over n of weight[n]
        IF |completeness_defect| exceeds a user tolerance THEN advise more modes.
      why: the sum of w_n is 1 exactly; the missing part measures how many modes were dropped.
```

**Truncation.** The mode count follows two checks.

1. The v1.0 rule keeps modes up to $\Lambda_\mathrm{max}$ and trusts times $t\ge40/\Lambda_\mathrm{max}$ for the series of $M$ (`min_valid_time`, [line 377](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L377)). This controls only the inventory series. The release series needs the balance form of step R1.4.
2. For a homogeneous sphere the weights are positive and the balance form converges like $N^{-3}$, the direct flux series like $N^{-1}$ (Section 3.5.2 of the derivation). The check script finds order 2.94 for the balance form between 10 and 100 modes. For the layered particle the weights change sign (152 negative weights among the first 300 modes in the check case), so the rigorous tail bound by $(1-\sum w_n)/\Lambda_{N+1}$ is lost. Update 2026-11-13: Section 3.5.3 of the derivation gives explicit bounds that need no sign information, Bound A (P21.106) with the mode-count rule (P21.110) and Bound B (P21.112) (PROVED under completeness (O14) and the classical min-max and Sturm oscillation theorems; DEMONSTRATED NUMERICALLY; conservative); the steps of this block do not use them yet, and the diagnostic and mode-count steps that would are proposed in Section 9.5. The completeness defect of step R1.5 is a diagnostic and gives no bound. Section 3.9 of the derivation observed orders of 2.1 and 4.0 for the two-layer case.

**Limits.** R1 assumes constant diffusivities, so it covers a constant-temperature run. Temperature-dependent cases use R2 or R3. R1 also stops at a kernel-uniform source: the activation source of step E5 is spread over all layers and needs the general projection.

### 5.2 Block R2: effective time and its exactness check

**Origin.** Temperature part, T5.

**In plain words.** If every layer diffusivity is the same function of temperature up to a constant factor, one clock change turns the whole variable-temperature problem into the constant-temperature problem of v1.0. The new clock is the effective time $\tau(t)=\int_0^t\alpha(s)\,ds$, where $\alpha$ is the common scaling. Decay disappears from the clock change after a factor $e^{-\lambda t}$ is split off. The released fraction is then the v1.0 closed-form series evaluated at $\tau$ instead of $t$.

This is valid only when the layers really share their temperature dependence, and real data rarely do. The program therefore runs a check first and refuses the shortcut when the check fails. A refusal is an error message with the reason. The program does not fall back to the shortcut with a warning and does not quietly use a mean activation energy.

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P22.24), (P22.25) | shared scaling $D_i(T(t))=\alpha(t)D_i^\ast$; $\alpha$ for a single shared $Q$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.26) | $\tau(t)=\int_0^t\alpha(s)\,ds$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.27) to (P22.31) | $c=e^{-\lambda t}\hat c(r,\tau(t))$; decay cancels; transformed source $\hat S=e^{\lambda t}S/\alpha$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.32) | without the decay factor the decay rate in $\tau$ is $\lambda/\alpha$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.33) to (P22.35) | inventory, flux and release by parts | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.36), (P22.37) | modal solution with variable $\alpha$; constant $\alpha$ and source | APPROVED | DERIVED NOT IMPLEMENTED |
| (H22.2) | structural test for condition E1 | DESIGN (sufficient test, not in B3) | DERIVED NOT IMPLEMENTED |

Table: 5.2 Equations of block R2.

The conditions E1 to E5 of Section 5.3.6 of the derivation decide whether the shortcut is exact (they are conditions of the model and are not the steps E1 to E5 of block E). Only condition E1 depends on the data.

| Condition | Meaning | How the program decides | Outcome for the planned input |
|-------------|---------------------------|------------------------------|------------------------------|
| E1 | one factor $\alpha(t)$ multiplies every layer | structural test of the laws, Eq. (H22.2) | depends on the data |
| E2 | temperature uniform in $r$ | the history is one scalar function of $t$ | true by construction |
| E3 | one decay constant for the nuclide | one nuclide per solve | true by construction; a source of any shape is allowed |
| E4 | absorbing outer surface | the only surface condition of the model | true by construction |
| E5 | no partition coefficients at interfaces | none in the model | true by construction |

Table: 5.3 The conditions of the effective-time solution and how each is tested.

The structural test for E1 compares each layer with layer 1, term by term, after sorting the terms by $Q$:

$$
Q_{i,m}=Q_{1,m}\ \ \text{for all } m,\qquad \frac{D_{0,i,m}}{D_{0,1,m}}=\mathrm{ratio}_i\ \ \text{independent of } m
\tag{H22.2}
$$

If Eq. (H22.2) holds for every layer then $D_i(T)=\mathrm{ratio}_i\,D_1(T)$ for all $T$, which is Eq. (P22.24) with $\alpha(t)=D_1(T(t))/D_1(T_\mathrm{ref})$ and $D_i^\ast=D_i(T_\mathrm{ref})$. A single-layer problem passes with any law. The test is sufficient. It is also what E1 needs in practice: if the ratios $D_i/D_j$ change at all along the history, no single clock exists (B3: two histories with equal $\int D_i\,dt$ in every layer but a different order give $F=0.011413$ and $0.009919$).

```text
BLOCK R2  effective time                              planned modules: thermal_intervals.rs, temperature_history.rs
INPUT   laws of the layers; history; settings; decay_constant; output times
OUTPUT  Exact{ratio, T_ref, alpha_law} or NotExact{reasons}; then tau(t) and the solution       [NEW]

R2.1  check_effective_time(laws, history, settings)
      1. reasons = [].
      2. IF the number of layers is 1 THEN ratio = [1] and go to step 6.
      3. ref = layer 1 terms sorted by Q.
         FOR each layer i = 2..5:
             cur = layer i terms sorted by Q.
             IF len(cur) != len(ref) THEN add reason "layer i has a different number of terms".
             FOR each term m:
                 IF | Q_cur[m] - Q_ref[m] | > 1e-12 * max(Q_ref[m], 1) THEN add reason "layer i, term m: activation energies differ".
                 ratio_term[m] = D0_cur[m] / D0_ref[m]
             IF max(ratio_term) - min(ratio_term) > 1e-12 * max(ratio_term) THEN add reason "layer i: D0 ratio depends on the term".
             ratio[i] = ratio_term[1]
         why: equality of floats read from files needs a tolerance; 1e-12 is far below any difference
              between two published correlations.
      4. IF any fluence rule is present and Phi_f is not frozen for the whole history THEN add reason "D0 changes in time".
         why: the fluence factor of (P22.9) acts on one term of one layer.
      5. Require the surface to be absorbing, no partition coefficients and a single decay_constant.
      6. IF reasons is not empty THEN return NotExact{reasons}.
      7. T_ref = the highest temperature of the history.                                    [DESIGN, decision 13]
         why: then alpha <= 1 and tau <= t; any T_ref is exact, this one keeps the series cheap to truncate.
      8. D_star[i] = diffusivity(law_i, T_ref) ;  alpha(T) = D_1(T) / D_1(T_ref).          (P22.24) APPROVED
         (For a single shared Q and one branch this is exp[-Q/Rg (1/T - 1/T_ref)], Eq. (P22.25).)
      9. RETURN Exact{...}.
```

| Input | Layers | Result | Reason |
|---|---|---|---|
| Real TECDOC-978 Cs data, FRG: kernel (209 and 362 $\mathrm{kJ\ mol^{-1}}$), buffer (constant), PyC (222), SiC (125 and 514), PyC | 5 | NotExact | layer 2 has one term and layer 1 has two; the buffer has $Q=0$ next to $Q>0$ |
| Synthetic data with one shared $Q=300\ \mathrm{kJ\ mol^{-1}}$ and different $D_0$ per layer | 5 | Exact | Eq. (H22.2) holds |
| Bare kernel, two-term Cs law (CRP-6 cases 1a and 1b) | 1 | Exact | one layer |

Table: 5.4 Outcome of the check for three inputs.

The ratio $D_\mathrm{SiC}/D_\mathrm{kernel}$ of the real data varies by a factor 6.3 between 1250 and 1800 $^\circ$C (the ratio is not monotonic: $7.2\times10^{-4}$ at 1250 $^\circ$C, a minimum of $4.2\times10^{-4}$ near 1438 $^\circ$C and $2.6\times10^{-3}$ at 1800 $^\circ$C, so the factor 6.3 is the largest over the smallest value; the integration check, whose kernel and SiC laws are these TECDOC-978 laws and whose buffer and PyC are synthetic, prints only the two end values, a factor 3.7; B7 review), so the failure is not marginal. Consequence for the project: with real data the effective-time solution applies to single-layer cases only and serves as a verification case with synthetic shared-$Q$ data in the multilayer case (Section 5.3.6 of the derivation). The integration script reproduces the three outcomes of Table 5.4.

```text
R2.2  tau_increment(law_1, T_ref, history, t_a, t_b)
      -- the same piecewise rule as step H1.4, applied to alpha = D_1(T) / D_1(T_ref)
      1. require [t_a, t_b] inside one segment.
      2. IF the slope is 0 THEN return (t_b - t_a) * D_1(T) / D_1(T_ref).
      3. otherwise sum over terms m, with arg_branch = (Q_m / Rg) * |1/T_b - 1/T_a|:
            arg_branch <= 1: Gauss-Legendre, 8 points                              [DESIGN threshold, evidence below]
            arg_branch  > 1: closed form (1/s) * D0_m * [T E2(theta_m / T)] from T_a to T_b      (P22.22) APPROVED
         then divide by D_1(T_ref).
      why: on a long ramp the closed form is exact in one evaluation; on a short piece it subtracts
           two nearly equal numbers.
R2.3  tau_at(t) = tau at the last breakpoint before t, plus tau_increment from there to t.        (P22.26) APPROVED
      -- the running sum over breakpoints is stored once; tau is strictly increasing, so it can be inverted by bisection.
```

Evidence for the 8-point rule and its threshold (Cs in SiC, $\lvert\Delta\ln D\rvert$ over the piece against a 40-digit reference): Gauss-Legendre relative error $1.0\times10^{-15}$ at 0.05, $4.4\times10^{-16}$ at 0.1 and 0.5, $4.8\times10^{-16}$ at 2, $1.9\times10^{-13}$ at 4, $5.3\times10^{-9}$ at 8 and $1.4\times10^{-6}$ at 16. The closed form stays at $2\times10^{-15}$ or better from 0.5 upward, and on four whole ramps (including the $-600\ \mathrm{K\ h^{-1}}$ cool-down) it agrees with adaptive quadrature to $2.4\times10^{-15}$ at worst. The threshold of 1 per term is therefore conservative.

```text
R2.4  Evaluate the solution when R2.1 returned Exact
      1. layers = [(R_i, D_star[i])] ; times_tau = [tau_at(t) for t in output_times].
      2. decay_constant == 0 and no source:
            F(t) = F_v1(tau(t)), computed by the existing v1.0 routine exact_release on times_tau       (P22.33) APPROVED
            (src/analytical/sturm_liouville.rs, lines 399 to 418). Points earlier than the validity time
            return "not valid" and no number (the v1.0 behaviour is kept).
      3. decay_constant > 0, no source:
            inventory   M(t)/M0 = exp(-decay_constant t) * sum_n w_n exp(-Lambda_star[n] * tau(t))     (P22.33) APPROVED
            release     M_rel(t) = exp(-decay_constant t) * Mhat_rel(tau(t))
                          + decay_constant * integral_0^t exp(-decay_constant s) Mhat_rel(tau(s)) ds   (P22.35) APPROVED
            the integral by Gauss-Legendre on each piece between breakpoints, using tau_at at the nodes.
            why: with decay the release depends on the whole path tau(s), s <= t, so one quadrature is
                 unavoidable; it uses inventories only, which converge faster than the flux series.
            [Accuracy note, updated 2026-11-13 from Section 5.3.7 of the derivation; the step is unchanged.]
                 The quadrature also needs Mhat_rel(tau(s)) for s earlier than the validity time of step 5,
                 where the truncated series is not valid, and for a bare kernel with a uniform start Mhat_rel
                 grows like sqrt(tau) near s = 0, so Gauss-Legendre converges only slowly on the first piece.
                 Correction to the B7 note: the balance form M_rel = M0 - M - M_dec is the same expression
                 as (P22.35) (exact, PROVED, (P22.81)), so it has the same start problem and is not a separate
                 route; the earlier remark "(smooth integrand)" was wrong for the start piece.
                 For an absorbing surface and no source the start can be bounded (all PROVED):
                   modal start in closed form per mode on [0, t0], error at most
                   decay_constant * M0 * min(t0 * Wbar_N, Vbar_N / alpha_0) for constant alpha    (P22.93)
                   with the computable tail bounds Wbar_N, Vbar_N of (P22.90), (P22.91), and the bracket
                   (P22.94) for a varying alpha and c0 >= 0; for a bare kernel with a uniform start the image
                   form (P22.86) up to t_B, start error below 9.4e-20 M0 with 12 modes for constant alpha.
                 Late pieces with tau(b)/tau(a) <= 2 and 8 Gauss-Legendre points obey (P22.97) for constant
                 alpha per piece (CLAIMED ONLY on ramp pieces); the total (P22.98) is PROVED only when alpha is
                 constant on the start piece and on every graded piece and under the coverage assumption
                 R24-A3c (late truncation (P22.95)). A layered particle with a long cold
                 start is OPEN (O17). The sub-steps that would apply these results are proposed in Section 9.5.
      4. With a source: each mode a_n(t) from the quadrature of Eq. (P22.36); the special case of constant
         alpha and constant source is Eq. (P22.37).                                                 (P22.36, P22.37) APPROVED
         The modal source projection comes from block R1.
      5. Series validity: require Lambda_covered_star * tau(t) >= 40 at every reported time (the v1.0 rule
         min_valid_time applied to tau). Otherwise report "not valid".
```

Not covered by an approved equation (OPEN): a parent and daughter pair with different diffusivities, because the source of the daughter is then the parent field and the two fields do not share one clock. Such cases use the finite-volume path only.

Update 2026-11-13 (Section 5.3.8 of the derivation; the block is unchanged). A joint effective time for the pair exists only under conditions N1 and N2 of Theorem 1 (PROVED within the class (P22.101)): equal activation energies are necessary and not sufficient, and no linear ramp with a non-zero activation energy satisfies N2 when the decay coupling is present (Corollary 2). For identical diffusivities in every layer the pair has the closed form (P22.109) in $\tau$, and whenever each species passes R2.1 on its own the daughter has an exact two-clock cascade (P22.113), (P22.114) whose truncation bound is OPEN. A chain mode of R2.1, a step 6 of R2.4 and a new block R2.5 that would use these results are proposed in Section 9.5; until the review accepts them, the finite-volume path stays the only route for such pairs.

### 5.3 Block R3: piecewise-constant temperature with eigenfunction projection

**Origin.** Temperature part, T6.

**In plain words.** When the temperature is constant on each interval, the problem on each interval is the v1.0 problem with that interval's diffusivities and it has its own exact eigenfunction series. At a switch the concentration profile is continuous in time and only its coefficients change, because the new eigenfunctions differ from the old ones. The program expands the old profile in the new eigenfunctions (a projection) and carries on. For a series that is not truncated, this is exact. This block gives the reference answer for CRP-6 case 3c and for any step schedule. If the layers share their temperature dependence (R2.1 passes), the eigenfunctions do not change and the projection matrix is the identity, so the program skips the projection and uses block R2.

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P22.39), (P22.40) | constant $T$ on each interval; modal solution per interval | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.41) to (P22.43) | continuity of $c$ at a switch; projection matrix $P_{mn}$ | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.44) | mode-count rule $\Lambda_{n_\mathrm{max}+1}(t_k-t_{k-1})\ge40$ | APPROVED (conservative) | DERIVED NOT IMPLEMENTED |
| (P22.45) | inventory mismatch $\Delta M_k$ at a switch as a diagnostic | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.46) | decay of the new coefficients like $m^{-2}$ | approximate asymptotic estimate | DERIVED NOT IMPLEMENTED |

Table: 5.5 Equations of block R3.

```text
Interval  = { t_start, t_end, T, D[1..5], Lambda_cover, n_modes }
Basis     = { Lambda[n], N[n], I[n], I1[n], shape_of_eigenfunction_in_each_layer }
             -- N[n] = int r^2 phi^2 dr (norm), I[n] = int r^2 phi dr, I1[n] = int_0^{R_1} r^2 phi dr
```

```text
BLOCK R3  piecewise-constant temperature              planned modules: thermal_intervals.rs, modal_source.rs
INPUT   HistoryConstant; laws; radii; nuclide data; source schedule; output times
OUTPUT  M, M_dec, M_gen, M_rel, M_out at the output times; Delta_M_k at every switch              [NEW]

R3.1  prepare(history_constant, laws, radii, nuclide data)
      1. FOR each interval k: D_k[i] = diffusivity(law_i, T_k) (step C3); cache the basis by the exact
         value of the vector D_k.
         why: schedules often return to an earlier temperature; the eigen-solve is the costly part.
      2. Mode count per interval (DESIGN rule built from the approved criteria):
           the old-basis rule  Lambda[n_max + 1] * (t_end - t_start) >= 40                  (P22.44) APPROVED
           the new-basis rule  the series is valid from t_valid = t_start + 40 / Lambda_cover
         so Lambda_cover[k] = max( 40 / (t_end[k] - t_start[k]), 40 / (earliest requested output offset in interval k) ),
         capped by max_modes. Outputs earlier than t_valid are reported "not valid".
         why: this is the v1.0 'min_valid_time' idea applied per interval. Rule (P22.44) is conservative:
              the t_s = 1 h test with n_max = 10 had Lambda_11 * t_s = 32 < 40 and was already at reference accuracy.
R3.2  basis(D_k)            -- eigenpairs of one interval
      1. eigenvalues Lambda[n] up to Lambda_cover by the v1.0 shooting and Sturm-count machinery
         (eigenvalue_count and eigenvalues in sturm_liouville.rs).
      2. FOR each Lambda[n]: shoot from the centre to get u = r*phi in each layer as a cosine and sine pair,
         then N[n], I[n], I1[n] by the v1.0 panel rule (about one panel per quarter wavelength, 16-point Gauss-Legendre).
      NEEDS A DECISION (decision 12): the routine 'shoot' and the layer states are private in v1.0
         (line 132 of src/analytical/sturm_liouville.rs at tag v1.0.0). Either new
         public accessors are added to sturm_liouville.rs, or the shooting is copied into thermal_intervals.rs.
R3.3  start (k = 1)
      a[n] = c0 * I1[n] / N[n]   for a uniform kernel concentration c0.               (P21.29, P21.30)
      (this is the v1.0 weight ingredient: w_n = 3 I1[n] I[n] / (R_1^3 N[n]) per unit M_0)   (P21.46)
R3.4  propagate_inside_interval(a, Lambda, decay_constant, gen_k, t_from, t_to)
      FOR each mode n with mode_kappa = Lambda[n] + decay_constant:
          constant modal source s on the sub-interval of length Delta:
          a_new = a * exp(-mode_kappa * Delta) + (s / mode_kappa) * (1 - exp(-mode_kappa * Delta))    (P22.40, P21.33) APPROVED
      why: exact in time, so a long interval needs no time stepping.
      Also accumulate the time integral of a_n over the sub-interval, for the decayed inventory:
          int a dt = a * (1 - exp(-mode_kappa Delta)) / mode_kappa
                     + (s / mode_kappa) * (Delta - (1 - exp(-mode_kappa Delta)) / mode_kappa)         (P21.38) APPROVED
      (The same recursion in the stable form (H21.2) is used by block R1.)
R3.5  switch(a_old, basis_old, basis_new)             -- at t_k, k >= 2
      1. P[m][n] = ( 1 / N_new[m] ) * integral over r of u_new_m(r) * u_old_n(r) dr, layer by layer.   (P22.43) APPROVED
         why: both bases are orthogonal with weight r^2, which does not depend on D when there are no
              partition coefficients. The products of two sines or cosines are integrated with the same
              panel rule as R3.2, with the panel count set by the sum of the two wave numbers (this
              avoids the singular closed form when two wave numbers coincide).                [DESIGN]
      2. a_new[m] = sum over n of P[m][n] * a_old[n].                                         (P22.42) APPROVED
      3. IF D_k == D_(k-1) in every layer THEN P is the identity; skip steps 1 and 2.
      4. Diagnostic: Delta_M = 4 pi ( sum_m a_new[m] I_new[m] - sum_n a_old[n] I_old[n] )     (P22.45) APPROVED
         Report it; it is zero for the infinite series. It measures the error right after the switch,
         not later: in the B3 check it was 3.0e-6 of M_0 at the switch while F later agreed to 6e-10.
R3.6  outputs at time t in interval k
      1. M(t) = 4 pi sum_n a_n(t) I_n                                                          (P21.35) APPROVED
      2. M_dec(t) = decay_constant * (accumulated integrals of M over all earlier sub-intervals plus the current one)   (P21.38, P21.41)
      3. M_gen(t) from the source schedule (known in closed form per piece).
      4. M_rel(t) = M_0 + M_gen(t) - M(t) - M_dec(t)  and  F = M_rel / M_0  (decay_constant = 0, no source).   (P21.39)
         why: the balance form. The flux series 4 pi sum Lambda I a converges like 1/m near a switch
              because the new coefficients fall only like m^-2 (about; observed m^-1.74 to m^-2.03), so
              the flux series must never be used just after a switch.                          (P22.46)
      5. Atoms outside the particle (decay_constant > 0):
         M_out(t) = M_0 exp(-decay_constant t) + integral_0^t gen(s) exp(-decay_constant (t - s)) ds - M(t)   (P21.44) APPROVED
         with gen the generation rate of the source schedule. No flux series is needed.
```

**Truncation and accuracy notes.** Coefficients of the new series fall like $m^{-2}$ in order of magnitude when the layer diffusivities change by different factors (Eq. (P22.46), an approximate asymptotic estimate; the observed exponent depends on the case). The reconstructed profile shows Gibbs-type oscillation near interfaces right after a switch, so only integral quantities (inventories) are reported near a switch. Completeness of the layered eigenfunctions, which makes the projection exact for the infinite series, was closed in B3 by a Green-function argument (Hilbert-Schmidt); no textbook reference was opened for it.

### 5.4 Block R4: staircase approximation of a ramp (verification tool only)

**Origin.** Temperature part, T7.

To test the finite-volume solver on a ramp with layer-specific activation energies, the ramp can be replaced by many short constant-temperature intervals and solved with block R3. The error shrinks as the intervals get shorter. This is slow (128 eigenvalue problems for an error below $10^{-6}$ on a 200 K ramp) and is not a production path.

| Equation | Statement | Status | Class |
|---|---|---|---|
| (P22.53) | left, midpoint and per-layer average rules for the interval temperature | APPROVED | DERIVED NOT IMPLEMENTED |
| (P22.54), (P22.55) | Magnus terms and the defects of the three rules | APPROVED for small $h\lVert B\rVert$ | DERIVED NOT IMPLEMENTED |
| (P22.56) | error orders 1, 2 and 2; observed order $p=\log_2(e_{N_T}/e_{2N_T})$ | updated 2026-11-13 (Section 5.4.6 of the derivation): order 1 PROVED uniformly in the stiffness (P22.130); order 2 PROVED for shared scaling (P22.135) and, up to a factor $1+\ln(T/h)$, on a fixed mesh (P22.140); with layer-specific $Q$ and $h\lVert A\rVert\gg1$ the $V$-norm order is about 1.75 in the test case and 1.51 to 1.96 in a second case (DEMONSTRATED NUMERICALLY); an order above 1 uniform in the mesh stiffness is OPEN | DERIVED NOT IMPLEMENTED |
| (P22.57) | with a shared scaling the average rule is exact at the interval ends | APPROVED | DERIVED NOT IMPLEMENTED |

Table: 5.6 Equations of block R4.

```text
R4.1  staircase(linear segment [t_a, t_b], N_T, rule)       h = (t_b - t_a) / N_T
      FOR k = 0..N_T-1:
         rule LEFT:        T_k  = T(t_a + k h)                       -- error order 1
         rule MIDPOINT:    T_k  = T(t_a + (k + 1/2) h)               -- order 2 for F; on a fixed mesh up to 1+ln(T/h) (note below)
         rule AVERAGE:     Dbar_i,k = (1/h) * int D_i(T(t)) dt (per layer, as in step H1.4) -- order 2 as MIDPOINT      (P22.53) APPROVED
      -- the AVERAGE rule hands layer diffusivities, not a temperature, to block R3; so R3 must accept a list D[1..5] directly.
      Return a HistoryConstant (or a list of D vectors).
R4.2  observed order  p = log2( e(N_T) / e(2 N_T) )                                              (P22.56)
```

The orders 1, 2 and 2 are shown numerically for one stiff test problem (ramp 1600 to 1800 $^\circ$C in 4 h and then a 2 h hold, synthetic layer-specific $Q$; last-pair orders 1.00, 2.03 and 1.95; B3 found 1.00, 2.01 and 2.04). A proof for the stiff regime ($h\lVert B\rVert$ of order $10^2$ to $10^3$) is OPEN; the Magnus argument holds for small $h\lVert B\rVert$. With a shared scaling factor the AVERAGE rule is exact at the interval ends (Eq. (P22.57)).

Update 2026-11-13 (Section 5.4.6 of the derivation; the steps above are unchanged). Order 1 of all three rules is PROVED uniformly in the stiffness, the mesh and $\lambda$ (Theorem A, (P22.130)), and the LEFT rule cannot do better. Temperature jumps on the interval ends are exact and a jump inside an interval costs order 1 (Theorem B). With shared scaling MIDPOINT has order 2 uniformly and AVERAGE is exact (Theorem C). With layer-specific $Q$ order 2, up to a factor $1+\ln(T/h)$, is PROVED only on a fixed mesh, with a constant that grows under mesh refinement (Theorem D, (P22.140)); in the $V$-norm of the concentration vector the observed order is 1.76 to 1.81 in the test case (1.75 on the finest mesh) and 1.51 to 1.96 in a second synthetic case, while the release fraction $F$ keeps orders 1.98 to 2.13 (DEMONSTRATED NUMERICALLY). Consequence for step R4.2 and case VT18: measure the observed order on a smooth functional such as $F$, as VT18 does; a $V$-norm acceptance test can only require an order of at least 1, and a fixed band such as $1.75\pm0.1$ is not safe. A sub-step R4.0 that places interval ends on every kink and jump of $T(t)$ is proposed in Section 9.5; for a linear segment, as in R4.1, the segment ends already are the kinks.

### 5.5 Methods that are not planned

- **Galerkin system on a fixed basis** (Eqs. (P22.47) to (P22.51)): slow algebraic convergence when the diffusivity ratios change; not recommended in Section 5.4.5 of the derivation.
- **Proper orthogonal decomposition** (Eq. (P22.52)): the affine split in the layer diffusivities is inexact at the two interface faces because of the harmonic conductance; OPEN, not implemented, not tested.
- **Robin outer surface** (Eq. (P22.38)): v1.0 and this page use the absorbing surface.
- **Radial temperature profile or temperature-dependent partition coefficients**: outside the model (assumptions A1 and A2).

## 6 Glossary of variables

Units are SI. "Block" names the block where the name is first used. Local loop counters and temporary values inside one step are not listed.

### 6.1 Time and step control

| Name | Symbol | Unit | Meaning | Block |
|---|---|---|---|---|
| `t`, `t_old`, `t_new` | $t$ | $\mathrm{s}$ | time; start and end of the current step | M, F |
| `t_end` | $t_\mathrm{end}$ | $\mathrm{s}$ | end time of the run | A |
| `dt`, `dt_n` | $\Delta t$, $\Delta t_n$ | $\mathrm{s}$ | step length, computed once as `t_new - t_old` | F |
| `dt_request` | | $\mathrm{s}$ | requested step of the FIXED grid (v1.0 `dt_s`) | A, F |
| `n_steps` | | 1 | number of steps of the FIXED grid | F |
| `dt_max`, `dt_first` | | $\mathrm{s}$ | largest step; first step and first step after a switch or jump | F |
| `dt_decay`, `dt_diff`, `dt_T`, `dt_cap` | | $\mathrm{s}$ | step limits from decay (P21.77), slowest diffusion mode (P21.78), temperature slope (P22.20); smallest fixed cap | F |
| `growth` | | 1 | largest ratio of a step to the previous one (default 2.0) | F |
| `grow_cap` | | $\mathrm{s}$ | limit on the next step from the growth rule | F |
| `eps_lambda`, `eps_1`, `eps_D` | $\varepsilon_\lambda$, $\varepsilon_1$, $\varepsilon_D$ | 1 | allowed relative decay error, diffusion error and change of $\ln D$ per step | F |
| `events` | | | sorted list of (time, flags) with flags BREAK, JUMP, SWITCH, OUTPUT, END | F |
| `event_pointer`, `t_event` | | 1, $\mathrm{s}$ | position in the list and the next event time | F |
| `step_limit` | | 1 | largest number of steps the user accepts | A, F |
| `output_every_steps` | | 1 | v1.0 output interval in steps (FIXED grid) | A |

Table: 6.1 Time and step control.

### 6.2 Geometry and mesh

| Name | Symbol | Unit | Meaning | Block |
|---|---|---|---|---|
| `N` | $N$ | 1 | number of cells | G |
| `R1` | $R_1$ | $\mathrm{m}$ | outer radius of the kernel (region 1) | G |
| `R` | $R$ | $\mathrm{m}$ | outer radius of the particle | G |
| `left`, `right`, `centroid[j]` | $r_{j-1/2}$, $r_{j+1/2}$, $\bar r_j$ | $\mathrm{m}$ | edges and volume centroid of cell $j$ | G |
| `volume[j]` | $V_j$ | $\mathrm{m^3}$ | volume of shell cell $j$ | G |
| `area[f]`, `area_R` | $A_f$, $4\pi R^2$ | $\mathrm{m^2}$ | area of an internal face; area of the outer face | G |
| `dist_left[f]`, `dist_right[f]`, `dist_R` | $\delta^-_f$, $\delta^+_f$ | $\mathrm{m}$ | distances from the face to the centroids on each side; outer distance $R-\bar r_N$ | G |
| `cell_factor[j]` | | 1 | fraction of cell $j$ that lies inside the kernel (P21.54) | G |
| `layer[j]` | $i(j)$ | 1 | layer index of cell $j$ | G |
| `west_g[j]`, `east_g[j]` | $g^W_j$, $g^E_j$ | $\mathrm{m^3\ s^{-1}}$ | diffusion conductance of the inner and outer face of cell $j$ | H |
| `G_R` | $G_R$ | $\mathrm{m^3\ s^{-1}}$ | surface conductance, `east_g` of the last cell; changes with $D_5$ | H |

Table: 6.2 Geometry and mesh.

### 6.3 State, decay and source

| Name | Symbol | Unit | Meaning | Block |
|---|---|---|---|---|
| `conc[j]`, `conc_old`, `conc_new` | $C_j$ | $\mathrm{mol\ m^{-3}}$ | cell-average concentration | H |
| `decay_constant` | $\lambda$ | $\mathrm{s^{-1}}$ | probability per second that one atom decays | B |
| `half_life_s`, `mean_life_s` | $t_{1/2}$, $1/\lambda$ | $\mathrm{s}$ | half-life and mean life | B |
| `z` | $z=\lambda\Delta t$ | 1 | decay per step | H |
| `fission_rate_i` | $\dot F$ | $\mathrm{fissions\ m^{-3}\ s^{-1}}$ | fission rate density in the kernel on interval $i$ | E |
| `yield` | $y$ | atoms per fission | cumulative yield used for the source | E |
| `f_q`, `y_q` | $f_q$, $y_q$ | 1, atoms per fission | fraction of fissile nuclide $q$ and its yield | E |
| `N_A` | $N_A$ | $\mathrm{mol^{-1}}$ | Avogadro constant, $6.02214076\times10^{23}$; BIPM, 2019, Table 1, p. 124 | E |
| `source_kernel_i` | $S_K$ | $\mathrm{mol\ m^{-3}\ s^{-1}}$ | source in the kernel on interval $i$ | E |
| `source_step` | $\bar S$ | $\mathrm{mol\ m^{-3}\ s^{-1}}$ | kernel source averaged over one step | H |
| `source_cell[j]` | $\bar S_j$ | $\mathrm{mol\ m^{-3}\ s^{-1}}$ | cell-average source over one step | H |
| `gen_rate` | $\mathrm{Gen}$ | $\mathrm{mol\ s^{-1}}$ | generation rate in the particle, $\sum_jV_j\bar S_j$ | H |
| `t99` | $t_{99}$ | $\mathrm{s}$ | precursor delay $\ln100/\lambda_p$ (P21.12) | E |

Table: 6.3 State, decay and source.

### 6.4 Temperature and diffusivity

| Name | Symbol | Unit | Meaning | Block |
|---|---|---|---|---|
| `T`, `T_old`, `T_end` | $T$ | $\mathrm{K}$ | temperature; at the start and at the end of a step | D, F, H |
| `T_ref` | $T_\mathrm{ref}$ | $\mathrm{K}$ | reference temperature of the effective time | R2 |
| `D0` | $D_0$ | $\mathrm{m^2\ s^{-1}}$ | pre-exponential factor of one branch | C |
| `Q` | $Q$ | $\mathrm{J\ mol^{-1}}$ | activation energy of one branch | C |
| `Rg` | $R_g$ | $\mathrm{J\ mol^{-1}\ K^{-1}}$ | gas constant, 8.31446261815324 (SI) or 8.3143 | C |
| `Phi_f` | $\Phi_f$ | $10^{25}\ \mathrm{m^{-2}}$ | frozen fast fluence for the SiC fluence factor (P22.9) | C |
| `D_layer[i]` | $D_i$ | $\mathrm{m^2\ s^{-1}}$ | diffusivity of layer $i$ in the current step | H |
| `D_floor` | | $\mathrm{m^2\ s^{-1}}$ | smallest diffusivity the program accepts | C |
| `Q_eff`, `Qmax` | $Q_\mathrm{eff}$ | $\mathrm{J\ mol^{-1}}$ | apparent activation energy (P22.6); largest over layers | C, F |
| `times_s`, `temps_K` | $t_k$, $T_k$ | $\mathrm{s}$, $\mathrm{K}$ | breakpoints of a history | D |
| `s` | $\dot T$ | $\mathrm{K\ s^{-1}}$ | slope of $T(t)$ on a segment | D, F |
| `alpha` | $\alpha(t)$ | 1 | common scaling of all layer diffusivities (P22.24) | R2 |
| `tau` | $\tau(t)$ | $\mathrm{s}$ | effective time (P22.26) | R2 |
| `ratio_i` | | 1 | $D_{0,i,m}/D_{0,1,m}$ of Eq. (H22.2) | R2 |
| `D_star[i]` | $D_i^\ast$ | $\mathrm{m^2\ s^{-1}}$ | diffusivity of layer $i$ at `T_ref` | R2 |

Table: 6.4 Temperature and diffusivity.

### 6.5 Bookkeeping and fractions

| Name | Symbol | Unit | Meaning | Block |
|------------------|-------------|---------|----------------------------------------------------|--------|
| `inv`, `inv0` | $M$, $M_0$ | $\mathrm{mol}$ | inventory inside the particle now and at the start | J |
| `gen_total` | $M_\mathrm{gen}$ | $\mathrm{mol}$ | cumulative generated | J |
| `dec_total` | $M_\mathrm{dec}$ | $\mathrm{mol}$ | cumulative decayed inside | J |
| `rel_total` | $M_\mathrm{rel}$ | $\mathrm{mol}$ | cumulative released through the surface | J |
| `out_now` | $M_\mathrm{out}$ | $\mathrm{mol}$ | released and still undecayed outside | J |
| `dec_out_total` | $M_\mathrm{dec,out}$ | $\mathrm{mol}$ | released and decayed outside | J |
| `release_rate` | $J_R$ | $\mathrm{mol\ s^{-1}}$ | release rate $G_RC_N$ | J |
| `res_inside`, `res_outside` | | $\mathrm{mol}$ | closure residuals (P21.72), (P21.75) | J |
| `F_rel`, `F_corr`, `F_gen`, `R_over_B`, `F_B`, `F_post` | | 1 | release fractions of Eq. (P21.45) | J |
| `t_irr` | $t_\mathrm{irr}$ | $\mathrm{s}$ | switch-off time of the source | J |

Table: 6.5 Bookkeeping and fractions.

### 6.6 Modal, projection and staircase

| Name | Symbol | Unit | Meaning | Block |
|---|---|---|---|---|
| `mode_rate[n]` | $\Lambda_n$ | $\mathrm{s^{-1}}$ | eigenvalue of mode $n$ | R1 |
| `weight[n]` | $w_n$ | 1 | release weight of mode $n$ per unit initial inventory | R1 |
| `mode_kappa` | $\kappa_n=\Lambda_n+\lambda$ | $\mathrm{s^{-1}}$ | mode rate with decay | R1 |
| `arg_mode` | $x=\kappa_n\Delta$ | 1 | argument of the interval factors | R1 |
| `phi1(x)`, `phi2(x)` | $\varphi_1$, $\varphi_2$ | 1 | interval factors of Eq. (H21.1) | R1 |
| `mode_inv[n]` | $m_n=4\pi I_na_n$ | $\mathrm{mol}$ | inventory carried by mode $n$ | R1 |
| `gen` | $\mathrm{Gen}$ | $\mathrm{mol\ s^{-1}}$ | generation rate on one interval of the schedule | R1 |
| `total_survivors` | $X=M+M_\mathrm{out}$ | $\mathrm{mol}$ | all atoms of the nuclide that have not decayed | R1 |
| `N[n]`, `I[n]`, `I1[n]` | $N_n$, $I_n$, $I^{(1)}_n$ | $\mathrm{m^3}$ times eigenfunction scale | norm and moments of the eigenfunction | R3 |
| `P[m][n]` | $P_{mn}$ | 1 | projection matrix at a temperature switch (P22.43) | R3 |
| `Delta_M` | $\Delta M_k$ | $\mathrm{mol}$ | inventory mismatch at a switch (P22.45) | R3 |
| `N_T`, `h` | $N_T$, $h$ | 1, $\mathrm{s}$ | number and length of staircase intervals | R4 |
| `arg_branch` | | 1 | $(Q/R_g)\lvert1/T_\mathrm{end}-1/T_\mathrm{old}\rvert$, the change of $\ln D$ of one branch over a step | H, R2 |

Table: 6.6 Modal, projection and staircase names.

### 6.7 Input fields, solver internals and temporaries

These names appear in the numbered steps and are listed so that every quantity of the pseudocode has a unit. Functions (for example `diffusivity`, `temperature_at`, `tau_at`) are not variables and are described in the block that defines them.

| Name | Symbol | Unit | Meaning | Block |
|---|---|---|---|---|
| `half_life_number`, `seconds_per_unit` | | 1, $\mathrm{s}$ | half-life as written in the data table and the seconds in one table unit | B |
| `precursor_half_life_s`, `warning_factor` | $t_{1/2,p}$ | $\mathrm{s}$, 1 | half-life of the longest precursor; fraction of the time to the first output above which the precursor warning appears (default 0.1) | E |
| `source_history` | | $\mathrm{fissions\ m^{-3}\ s^{-1}}$ | list of (start, end, fission rate density), constant on each interval | E |
| `output_times` | | $\mathrm{s}$ | times at which a row is written | A, F |
| `q_unit`, `cutoff_tag`, `scale_1e25_per_m2` | | text, text, $10^{25}\ \mathrm{m^{-2}}$ | unit tag of $Q$; energy-cutoff tag of the fluence; fluence scale 5.0 of Eq. (P22.9) | C |
| `starts_s`, `end_s` | $t_k$ | $\mathrm{s}$ | start times and end time of a constant-interval history | D |
| `diag_excess[j]` | $v_j$ | $\mathrm{m^3\ s^{-1}}$ | volume term of the matrix diagonal, $V_j(1/\Delta t+\lambda)$ (P21.59) | H |
| `pivot_e[j]`, `den[j]`, `carry[j]`, `partial[j]` | $e_j$, $d_j$ | $\mathrm{m^3\ s^{-1}}$, $\mathrm{m^3\ s^{-1}}$, $\mathrm{mol\ s^{-1}}$, $\mathrm{mol\ m^{-3}}$ | elimination pivot; its sum with the east conductance; the term carried from the previous cell into the right-hand side; the forward-sweep value of the concentration (P21.61) | H |
| `rem` | | $\mathrm{s}$ | time left to the next event during the landing loop | F |
| `Dbar_m`, `logs`, `top` | $\bar D_m$ | $\mathrm{m^2\ s^{-1}}$, 1, 1 | step-average diffusivity of one branch; natural logarithms of the branch terms and their maximum | C, H |
| `out_exp`, `out_exp_old` | | $\mathrm{mol}$ | optional exponential-form $M_\mathrm{out}$ of Eq. (P21.74) | J |
| `rel_total_at_t_irr`, `inv_at_t_irr` | | $\mathrm{mol}$ | stored values at the step that ends on $t_\mathrm{irr}$, for $F_\mathrm{post}$ | J |
| `flux_direct`, `completeness_defect` | | $\mathrm{mol\ s^{-1}}$, 1 | diagnostic flux series (P21.37); $1-\sum_nw_n$ | R1 |
| `area[n]` (block R1) | | $\mathrm{mol\ s}$ | time integral of the modal inventory over one interval (H21.2) | R1 |
| `f1`, `f2` | $\varphi_1$, $\varphi_2$ | 1 | values of the interval factors at `arg_mode` | R1 |
| `max_modes`, `n_modes`, `Lambda_cover` | | 1, 1, $\mathrm{s^{-1}}$ | mode-count cap; modes used; eigenvalue up to which the series is trusted | R3 |
| `t_valid` | $t_\mathrm{valid}$ | $\mathrm{s}$ | first time at which a truncated series is trusted, $40/\Lambda_\mathrm{cover}$ after the start of the interval | R2, R3 |
| `t_a`, `t_b` | | $\mathrm{s}$ | start and end of one piece of a history segment | H, R2 |
| `times_tau` | | $\mathrm{s}$ | effective times of the output times | R2 |
| `reasons`, `ratio_term[m]` | | text, 1 | list of reasons for a refusal; $D_0$ ratio of one term in the structural test | R2 |

Table: 6.7 Input fields, solver internals and temporaries.

## 7 Verification cases and pass criteria

The cases below use the v1.0 five-layer particle of the check scripts (radii 250, 350, 390, 425 and 465 $\mu\mathrm{m}$; synthetic diffusivities $4\times10^{-14}$, $4\times10^{-13}$, $2\times10^{-14}$, $5\times10^{-15}$ and $2\times10^{-14}\ \mathrm{m^2\ s^{-1}}$; cells (20, 8, 4, 4, 4)) unless stated. The last column states what a Python transcription of the pseudocode showed. A value in that column belongs to the transcription and not to Rust; each case must be repeated on the Rust code. The prefix V marks the decay, source and inventory cases, VT the temperature cases and VI the cases that need the joined run.

### 7.1 Decay, source and inventory

| ID | Case and reference | Pass criterion | Result of the transcription |
|---|---|---|---|
| V1 | $\lambda=0$, $S=0$ against the v1.0 step | all cell values bit-identical over 200 steps (0 ulp) | PASS, 0 ulp |
| V2 | decay without diffusion; $C^n=(1+z)^{-n}C^0$ (P21.63), $e^{-\lambda t}$ | identity to $10^{-12}$ relative over at most 300 steps; error against $e^{-\lambda t}$ within 2 % of $\tfrac12\lambda^2t\Delta t$ (P21.77) | PASS, $1.2\times10^{-14}$; $1.5012\times10^{-2}$ against $1.5000\times10^{-2}$ |
| V3 | decay with diffusion, no source; (P21.63) | identity to $10^{-11}$; $F_\mathrm{corr}$ approaches the $\lambda=0$ fraction at observed order 0.8 to 1.2 | PASS, $4.6\times10^{-15}$; order 1.00 |
| V4 | constant kernel source, no decay; (P21.82) to (P21.84) | $\lvert J_R/\mathrm{Gen}-1\rvert<10^{-12}$; $M_\mathrm{ss}/\mathrm{Gen}\to\tau_\mathrm{res}$ at order 1.9 to 2.1 | PASS, $1.3\times10^{-15}$; errors $5.5\times10^{-6}$, $1.4\times10^{-6}$, $3.5\times10^{-7}$, orders 1.97, 1.99 |
| V5 | source with decay, fixed point; (P21.85): $R/B=0.9684291531$ (synthetic $D$, $\lambda=2\times10^{-8}\ \mathrm{s^{-1}}$) | order 1.9 to 2.1 in $h$; value on the (20,8,4,4,4) mesh within $2\times10^{-6}$ | PASS, 0.968428661; order 1.99 |
| V6a | Booth sphere, steady $R/B$; (P21.86) | order 1.9 to 2.1 in $h$ for $N=100$, 200, 400 | PASS; $x=1$: $4.1\times10^{-6}$, $1.0\times10^{-6}$, $2.6\times10^{-7}$; $x=100$: $9.9\times10^{-4}$, $2.5\times10^{-4}$, $6.2\times10^{-5}$ (orders 2.00; $x=100$ misses the $10^{-5}$ tolerance at $N=200$) |
| V6b | Booth sphere, transient $F_B=M_\mathrm{out}/(M+M_\mathrm{out})$; (P21.87), TECDOC-1674 (7.8), $x=1$ | first order in $\Delta t$ when $\Delta t$ and the grid are halved | PASS; errors $5.2\times10^{-4}$, $2.7\times10^{-4}$, $1.4\times10^{-4}$, order 0.94 |
| V7 | modal series (R1) against the finite-volume run, five layers, source then decay | difference over $M_\mathrm{gen}(t_\mathrm{irr})$ below $2\times10^{-3}$ at the finest mesh, factor 1.8 or more per halving | PASS, worst $9.7\times10^{-4}$ at the finest mesh (80, 32, 16, 16, 16) |
| V8 | manufactured solution, two layers; (P21.88) to (P21.90) | order within 0.1 of 2 (steady) and 1 (transient) | not in the transcription script; `sec4_p21_mms_source.py` passed in the B3 re-run |
| V9 | closure with switching source and variable steps; (P21.72), (P21.75) | relative residual below $10^{-12}$ | PASS, $9.5\times10^{-16}$ and $6.3\times10^{-16}$ |
| V10 | positivity; (P21.62) | minimum of $C$ is $\ge0.0$ for $\Delta t=10^{-3}$ to $10^{12}\ \mathrm{s}$ | PASS, smallest value $1.2\times10^{-127}$ |
| V11 | CRP-6 case 1a (no source, no decay, one region); (P21.91) with $R_g=8.3145$ in the transcription | $\lvert F-0.46742\rvert<2\times10^{-4}$ | PASS, $F=0.46737$ at $\Delta t=360\ \mathrm{s}$, $N=150$; step C2 offers only the SI value and 8.3143, whose series values 0.467404 and 0.467337 are also inside the tolerance (B7) |
| V12 | CRP-6 case 1b; $1-F$ from the series is $4.0703\times10^{-7}$ with $R_g=8.3145$ ($4.0746\times10^{-7}$ SI, $4.0936\times10^{-7}$ with 8.3143; the Rust test takes the series value for the $R_g$ it runs) | within 5 % at $\Delta t\le36\ \mathrm{s}$; the error at larger steps follows (P21.78) | PASS, $4.0903\times10^{-7}$ (+0.5 %); at 360 s +5.14 % against 5.05 % predicted |
| V13 | source switch inside a step | $M_\mathrm{gen}$ exact to $10^{-13}$ with option A | PASS, error 0; option R is off by $3\times10^{-3}$ |
| V14 | steady state preserved by Backward Euler | profile change below $10^{-11}$ for a step of any size | PASS, $9.1\times10^{-16}$ |
| V15 | decay-exact split as cross-check, $S=0$; (P21.67) | equals $e^{-\lambda t}\times$ v1.0 run to $10^{-12}$ | PASS, $3.3\times10^{-15}$ |
| V16 | block B against `decay.csv` | seconds to $10^{-8}$, $\lambda$ to $5\times10^{-7}$ relative; a stable row gives 0; a negative half-life is rejected | PASS, $2.4\times10^{-10}$ and $3.9\times10^{-7}$ |
| V17 | cell and step averages of block E and G; (P21.54) | agreement to $10^{-12}$ | PASS, $4.4\times10^{-16}$ |
| V18 | block R1 recursion against ODE integration | relative difference below $10^{-8}$ (25 modes) | PASS, $8.2\times10^{-16}$ and $6.4\times10^{-15}$ |
| V19 | block R1, Booth sphere | absolute difference below $5\times10^{-9}$ against (7.8) at $x=1$ | PASS, ten printed digits (0.8731144969) |
| V20 | block R1, $\lambda=0$; (P21.76) | $M_\mathrm{out}=M_\mathrm{rel}$ to $10^{-12}$ relative | PASS |
| V21 | helper identities (H21.1) to (H21.3) | symbolic residual 0; relative error below $10^{-14}$ | PASS, $2.9\times10^{-16}$ |

Table: 7.1 Verification cases for decay, source and inventory.

Which cases belong in `cargo test`: the algebraic cases V1, V2, V3 (identity part), V9, V10, V13, V14, V15, V16, V17 and V21 run in seconds on a small mesh. The accuracy cases V4 to V8, V11, V12 and V18 to V20 need a mesh and step study and fit `examples/verify.rs` or a longer test file.

Two differences between the transcription and Section 4.8 of the derivation need the reviewer's attention. Section 4.8 test 6 gives a steady Booth tolerance of $10^{-5}$ relative at $N=200$; the transcription meets it at $x=1$ and misses it at $x=100$ because the thin boundary layer needs more cells, so the criterion of V6a is the observed order. And Section 4.8 test 7 named its reference as the Phase 2.2 modal solution, while the reference is the modal series of block R1, which belongs to the constant-temperature part; the B7 review corrected the derivation text to "Phase 2.1 modal solution of Section 3".

### 7.2 Temperature and diffusivity

| ID | What is tested | Pass criterion | Evidence now |
|---|---|---|---|
| VT1 | input validation, steps A8, C1 | each bad input returns an error that names the field; no panic; $Q=0$ accepted | 9 of 9 reference checks pass |
| VT2 | $D(T)$ by log-sum-exp against a 40-digit evaluation (4 register laws, 5 temperatures) | relative difference $\le10^{-14}$; $\ln D$ finite where $e^{-Q/(R_gT)}$ underflows | worst $8.4\times10^{-15}$; $\ln D=-1097.91$ |
| VT3 | $Q_\mathrm{eff}$ and crossover | 130.1 and 438.2 $\mathrm{kJ\ mol^{-1}}$, 1772.4 K within 0.1 | 130.10, 438.22, 1772.44 |
| VT4 | gas constant, CRP-6 1a | 0.467337 (8.3143) and 0.467404 (SI) within $2\times10^{-6}$ | pass |
| VT5 | history construction, step D1 to D3 | AGR-1 totals 333.78, 335.95, 338.12 h and nine CRP-6 totals within 0.01 h; non-increasing times rejected | pass |
| VT6 | interpolation, step D5; (P22.21) | $T(t_k)$ equals the table value exactly; no extrapolation | pass |
| VT7 | landing, step F5; (H22.1) | every event is a step end by float equality; a duplicate event leaves the step sequence and the final state bit-identical | 41 of 41 events; sequence identical |
| VT8 | step bound; (P22.20) | largest $\lvert\Delta\ln D_i\rvert\le2\varepsilon_D$ in every step; no step below 1 s next to a step of 100 s | 0.102 for $\varepsilon_D=0.1$; shortest 1.24 s |
| VT9 | effective-time integral, step R2.2 | closed form $\le10^{-12}$; Gauss-Legendre $\le10^{-13}$ while the argument is at most 1 | $2.4\times10^{-15}$ worst; $\le1.4\times10^{-15}$ |
| VT10 | FV against the series at $\tau(t)$ (shared $Q$) | spatial order $2.0\pm0.1$; temporal order $1.0\pm0.1$ | order 2.00, difference $5.4\times10^{-7}$ at 160 cells per layer |
| VT11 | option A against Backward Euler in $\tau$ ($\lambda=0$, shared $\alpha$) | relative difference $\le10^{-11}$; for $\lambda>0$ a first-order difference is expected | $3.5\times10^{-15}$; $2.7\times10^{-3}$ (B3); $4\times10^{-16}$ in the integration check |
| VT12 | exactness check refuses unshared data, step R2.1 | NotExact for layer-specific $Q$; shared $Q$ passes and the two orders agree to $10^{-13}$ | $F=0.011413$ against $0.009919$; shared $Q$ $8.2\times10^{-15}$; the three outcomes of Table 5.4 reproduced |
| VT13 | closure with a changing $G_R$; (P22.16) | relative residual $\le10^{-12}$ at every step; the fixed-$G_R$ booking of v1.0 fails (above $10^{-8}$) | $4.8\times10^{-15}$; negative test $5.8\times10^{-5}$ (`sec6_pseudocode_checks.py`); $1.2\times10^{-5}$ in the integration check |
| VT14 | positivity; (P22.17) | minimum concentration $\ge0.0$ | 0.0 |
| VT15 | decay with effective time ($\lambda>0$); (P22.33), (P22.35) | inventory and release relative difference $\le10^{-11}$ to a stiff real-time integration | $1.6\times10^{-12}$ and $2.8\times10^{-12}$ |
| VT16 | regression to v1.0 (constant $T$, single-term $Q=0$ laws) | results bit-identical to v1.0 | pass for 200 steps in Python |
| VT17 | projection at a temperature step, block R3 | agreement $\le10^{-6}$ in $F$ with a Richardson-extrapolated FV for a 1 h step; $P=I$ when the diffusivities do not change; $\Delta M_k$ reported | $6.9\times10^{-11}$; $1.5\times10^{-10}$ for a 0.05 h step |
| VT18 | staircase orders, block R4 | last-pair orders $1.0\pm0.15$, $2.0\pm0.15$, $2.0\pm0.15$ (left, midpoint, average) for $N_T\ge32$ | 1.00, 2.03, 1.95 (B3: 1.00, 2.01, 2.04) |

Table: 7.2 Verification cases for the Arrhenius and temperature-history blocks.

### 7.3 Cases that need the joined run

These cases test what neither part could test alone. They are new in this page and have a Python transcription in `b4_phase2_pseudocode_integration.py`.

| ID | What is tested | Pass criterion | Result of the transcription |
|---|---|---|---|
| VI1 | planner landing with events from the source and from the history | every event time is a step end by float equality; the step after a switch is at most `dt_first`; no step above `dt_max` or `dt_decay`; largest change of $\ln D$ at most $2\varepsilon_D$ | PASS; 8 of 8 events, 139 steps, largest $\lvert\Delta\ln D\rvert=0.102$ |
| VI2 | closure of the joined run (ramp, decay, source off at 3 h), options A and E | both residuals below $10^{-12}$; minimum concentration at least 0.0; generated amount equal to its analytic integral to $10^{-13}$ | PASS; $1.3\times10^{-15}$ (A), $1.2\times10^{-15}$ (E); generated amount error 0 |
| VI3 | negative test, release booked with the first-step $G_R$ | residual above $10^{-8}$ | PASS, $1.2\times10^{-5}$ |
| VI4 | joined loop in FIXED mode against the v1.0 transcription | every row and the final profile bit-identical | PASS, 200 steps |
| VI5 | constant-interval history with a jump: left-limit rule of step D6 | the jump is a step end; no step shorter than 1 s apart from the first; closure below $10^{-12}$; option E on the step that ends at the jump uses the old temperature | PASS, 36 steps; the naive rule gives 66 steps and a shortest step of $5.6\times10^{-7}\ \mathrm{s}$ |
| VI6 | option A against Backward Euler in $\tau$ over five layers with shared $Q$ | relative difference $\le10^{-11}$ | PASS, $4\times10^{-16}$ |

Table: 7.3 Verification cases for the joined run.

## 8 Cross-check of every step against the reconciled equations

### 8.1 Method

Every step that carries an equation number was compared with the display equation of that number in `sec3_phase21_decay_source_analytical.md`, `sec4_phase21_decay_source_fv.md` and `sec5_phase22_temperature.md`, with the reconciliation record `B3_reconciliation.md` as the arbiter. A script (`extract_eqs.py` in the work folder) pulled the 151 tagged display equations (P21.1 to P21.92 and P22.1 to P22.59) out of the three sections so that each cited number could be read next to the step that cites it. The comparison of meaning was done by reading, with the following quantities recomputed:

- the elimination recurrence of step H4.3 against Gaussian elimination of the matrix of Eq. (P21.58), and against the v1.0 code of `solve_diffusion_system` (lines 132 to 160);
- the interval formulas (H21.1) to (H21.3) against Eqs. (P21.33), (P21.38) and (P21.44);
- the identity $\int e^{-\theta/T}dT=T\,E_2(\theta/T)$ behind Eq. (P22.22);
- the step limits of Eqs. (P21.77), (P21.78) and (P22.20) with the example values;
- the v1.0 line numbers quoted in this page, against the tag.

### 8.2 Mismatches found and how the pseudocode was changed

No equation was changed. Each row below names a place where the drafts disagreed with each other, with the v1.0 code, or with the equation they cite.

| ID | Where | Mismatch | Change in the pseudocode |
|---|---|---|---|
| X1 | decay part C1 against temperature part T3 | two step planners with different growth factors (1.5 against 2.0), different handling of the last step before an event (common scale factor against halving the remainder), and a restart after an event in one and not in the other | merged into block F: landing by assignment and halving of the remainder, restart at `dt_first` after SWITCH and JUMP only, one `growth` (default 2.0), caps from both parts; decision 10 |
| X2 | decay part C1.5 against v1.0 lines 190 and 191 | with growth 1 the geometric rule gives the v1.0 step only to round-off (the common scale factor is not the same arithmetic as `end_time / steps`; in 19314 of 20000 random pairs of end time and step the last bits differ, by at most $1.8\times10^{-14}$ relative), but cases V1 and VT16 demand bit identity | step F0: FIXED mode with the v1.0 rule; block A refuses FIXED when an event time exists |
| X3 | temperature part T2 (right-continuous constant kind) against T3 step 5 and T4.2 option E | `temperature_at(t_new)` at a jump returns the temperature of the next segment, so the step guard sees an impossible jump of $\ln D$ and halves the last step before every jump, and option E would use the new temperature for the old step. The check script reproduces it (66 steps, shortest step $5.6\times10^{-7}\ \mathrm{s}$, against 36 steps) | new step D6 (`temperature_at_step_end`, left limit); used in F5.5 and H1.3 |
| X4 | decay part D4 against temperature part T4.5 | D4 defined $G_R$ as the east conductance of the last cell without saying that it must be the one of the current step; v1.0 line 210 reads the fixed mesh constant, which is wrong once $D_5$ changes | J1.2 states that $G_R$ is the one used in the matrix of the same step; negative test VI3 |
| X5 | decay part Table 6A.11 | the relation $4\pi s_nI_n=G\,w_n$ was attributed to (P21.46), (P21.47); (P21.47) is the $R/B$ series | cited as (P21.30) with (P21.46) in Table 5.1 |
| X6 | temperature part T5.1 step 8 | $\alpha(T)=D_1(T)/D_1(T_\mathrm{ref})$ was cited to (P22.24), (P22.25); (P22.25) holds only for one branch with a shared $Q$, and the general statement is in the text of Section 5.3.1 | cited to (P22.24), with (P22.25) named as the single-branch case (R2.1 step 8) |
| X7 | decay part C1.3 | (P21.78) is derived for constant diffusivities, yet the step was written without a restriction | marked OPEN for a temperature history in F3 (use $\Lambda_1$ at the highest temperature as a warning only); update 2026-11-13: with shared scaling and option A that value is a PROVED sufficient cap (not proved for option E, whose lag $L_1$ enters), see the F3 note and (P22.69) to (P22.72) |
| X8 | both parts | symbols used twice: $G$ (generation and conductance), $\kappa$ (mode rate and layer ratio), $R$ (gas constant and radius), $e$ (event and pivot), $x$ (three arguments), $B$ (birth rate and step matrix) | renamed (Table 1.3); $B$ appears only in cited derivation text |
| X9 | temperature part equations (6.1) and (6.2) | numbered locally and not in the reconciled sections | renamed (H22.1) and (H22.2), status DESIGN; the helpers (H21.1) to (H21.3) are HELPER |
| X10 | both parts, verification | two ID series (V and VT) and an overlap between V9 and VT13 | kept as two series; the overlap is noted in Table 7.2; new series VI for the joined run |
| X11 | decay part, open item O6 | Section 4 of the derivation refers to the v1.0 step at lines 171 to 180; in the tag it is lines 162 to 182 with the changed lines at 178 and 179 | not a pseudocode change; carried as O6 in Section 9.3. Resolved in the B7 review: lines 171 to 180 are the assembly loop inside `backward_euler_step`, and Section 4.3 of the derivation now cites the function at lines 162 to 182 with that loop named |
| X12 | temperature part D1 example | named two literature sources for the missing ramp rates by author | replaced by the register rows (TH-001 onward); no reference is cited that this page cannot show |
| X13 | decay part, interaction paragraph and Table 6A.6, against this page | the decay part states that the decay factor of Eq. (P21.80) holds for a diffusivity that changes in time and cites (P21.53) for the cell average; this page cited neither, so nothing in block H said why decay and a changing $D$ may be combined | cited (P21.80) in Table 4.10 and in the new paragraph "Decay and a changing diffusivity" (restriction: one constant $\lambda$); cited (P21.53) in Table 4.8 |

Table: 8.1 Mismatches and changes.

### 8.3 Status of the equations used

The word APPROVED in the tables of Section 4 and 5 means that the equation is in the reconciled Sections 3 to 5 and that either B3 re-derived it or the section check logs and the 17 section scripts that B3 re-ran cover it. The distinction matters, so the equations cited in Sections 4 and 5 that are not mentioned anywhere in `B3_reconciliation.md` are listed here (found by a script that reads the B3 text and expands its ranges). Many are definitions or one-line consequences of an equation that B3 did confirm; the Phase 2.2 finite-volume relations (P22.13) to (P22.23) are the largest group, and for them the support is the Section 5 check log, the scripts `sec5_*.py` and the closure and positivity runs of Section 7 of this page, not a separate B3 row.

```text
P21.1 P21.5 P21.6 P21.7 P21.8 P21.9 P21.10 P21.11 P21.13 P21.15 P21.18 P21.23 P21.26 P21.27
P21.29 P21.30 P21.34 P21.35 P21.37 P21.38 P21.40 P21.41 P21.45 P21.46 P21.52 P21.54 P21.55
P21.66 P21.67 P21.71 P21.72 P21.79 P21.90 P21.92
P22.5 P22.9 P22.10 P22.13 P22.14 P22.15 P22.16 P22.17 P22.18 P22.19 P22.20 P22.21 P22.22
P22.23 P22.33 P22.37 P22.38 P22.39 P22.47 P22.51 P22.52 P22.53 P22.57 P22.58 P22.59
```

Steps that rest on OPEN items are marked in the text: E1 (precursor warning factor), E5 (activation products), C1.4 (fluence cutoff), D2.4 and D2.5 (ramp shape, row 8a), F3 (variable temperature), R3.2 (access to eigenfunctions), R4 (stiff regime), R2.4 (parent and daughter with different diffusivities) and R2.4 step 3 (start of the release quadrature, added in the B7 review). Update 2026-11-13: F3 rests on an unproved bound for layer-specific $Q$ (O12) and, under shared scaling, for option E; R4 only for an order above 1 that is uniform in the mesh stiffness (O10); the parent-daughter note after R2.4 for the truncation of the cascade and for transformations outside the class (P22.101) (O11); and R2.4 step 3 for a layered particle with a long cold start and for ramp pieces ((P22.87), (P22.97); O17).

**B7 final review (2026-11-06).** Changes to this page: the solution-path row for a ramp now says that no general closed form is available (Section 5.4.2 of the derivation); the worked scalar example of block H now calls its difference a one-step local error of order $z^2$; R2.4 step 3 carries the accuracy note on the start of the release quadrature; the remark under Table 7.1 and items X11 and O6 record the corrections made to Section 4 of the derivation; the closing sentence of Section 9.4 lists every new non-Rust file. No equation, step or verification result was changed.

**Second integration pass (2026-11-06, after the decay part was rewritten).** The rewritten decay part and the unchanged temperature part were compared with this page again. The numbers of the decay part (four decay constants, the Cs-137 source, the two step limits, the yield table, the scalar example, the verification cases V1 to V21) agree with this page, and the re-run of its script gives 55 of 55 PASS. Changes made on this page: item X13 above; the error values of cases V4, V6a and V6b and the mesh of V7 and V11 are now written in Table 7.1 instead of being left to the part draft; Table 6.7 lists the input fields, the solver internals and the temporaries of the numbered steps with their units. No equation, step or other verification result was changed. The pseudocode still carries the merged planner of block F (growth 2.0, landing by halving the remainder), not the growth factor 1.5 of the decay part; that is item X1 and decision 10.

## 9 Status, open items and decisions

### 9.1 Classification of the operations

| Operation | Class | Reference |
|---|---|---|
| mesh, volumes, series conductances | IMPLEMENTED | [src/numerical/finite_volume.rs, lines 50 to 99](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L50-L99) |
| subtraction-free solver | IMPLEMENTED | [lines 132 to 160](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L132-L160) |
| inventory sum | IMPLEMENTED | [lines 114 to 120](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L114-L120) |
| Kahan release sum and release rate | IMPLEMENTED | [lines 209 to 217](https://github.com/weewzr/STRATO/blob/v1.0.0/src/numerical/finite_volume.rs#L209-L217) |
| modal eigenvalues and weights $w_n$ | IMPLEMENTED | [src/analytical/sturm_liouville.rs, lines 308 to 395](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L308-L395) |
| exact release from the series at given times | IMPLEMENTED | [lines 399 to 418](https://github.com/weewzr/STRATO/blob/v1.0.0/src/analytical/sturm_liouville.rs#L399-L418) |
| decay constant, source, yield, cell and step averages (B, E, G4) | DERIVED NOT IMPLEMENTED | (P21.7), (P21.8), (P21.10), (P21.54), (P21.57) |
| step with decay and source (H4) | DERIVED NOT IMPLEMENTED | (P21.58), (P21.59) |
| events and planner (F) | DERIVED NOT IMPLEMENTED | (P21.77) to (P21.79), (P22.20); the landing and restart rules are DESIGN |
| bookkeeping with $M_\mathrm{gen}$, $M_\mathrm{dec}$, $M_\mathrm{out}$ and fractions (J) | DERIVED NOT IMPLEMENTED | (P21.45), (P21.72) to (P21.76), (P22.16) |
| Arrhenius diffusivity, apparent $Q$, crossover (C) | DERIVED NOT IMPLEMENTED | (P22.1), (P22.6), (P22.7) |
| histories (D) | DERIVED NOT IMPLEMENTED | (P22.21), (P22.39), (P22.58), (P22.59) |
| conductances at the step temperature, step average (H1, H2) | DERIVED NOT IMPLEMENTED | (P22.14), (P22.22) |
| modal solution with decay and source (R1) | DERIVED NOT IMPLEMENTED | (P21.28) to (P21.39), (P21.44) |
| effective time (R2) | DERIVED NOT IMPLEMENTED | (P22.24) to (P22.37) |
| projection at a temperature switch (R3) | DERIVED NOT IMPLEMENTED | (P22.40) to (P22.46) |
| staircase of a ramp (R4) | DERIVED NOT IMPLEMENTED | (P22.53) to (P22.57) |
| activation products (E5) | FUTURE | (P21.13) to (P21.16), data NOT VERIFIED |
| parent-daughter chains with overlap matrix | FUTURE | (P21.17) to (P21.22) |
| projection of an arbitrary start profile onto the modes | FUTURE | (P21.27), (P21.29) |
| precursors as separate species | FUTURE | Section 3.3.4 of the derivation |
| Galerkin on a fixed basis, POD, Robin surface | FUTURE (not planned) | (P22.47) to (P22.52), (P22.38) |
| truncation bounds and mode count for R1 (proposed steps R1.0 and R1.5) | DERIVED NOT IMPLEMENTED; proposed in Section 9.5, not yet a step | (P21.105) to (P21.112) |
| diffusion step rule with a temperature history (proposed change of F3 to F5) | DERIVED NOT IMPLEMENTED; proposed in Section 9.5, not yet a step | (P22.69) to (P22.74) |
| bounded start of the release quadrature (proposed R2.4 sub-steps 3a to 3e) | DERIVED NOT IMPLEMENTED; proposed in Section 9.5, not yet a step | (P22.81) to (P22.99) |
| parent-daughter closed form and two-clock cascade (proposed R2.1 chain mode, R2.4 step 6, block R2.5) | DERIVED NOT IMPLEMENTED; proposed in Section 9.5, not yet a step | (P22.105) to (P22.114) |
| second-order or exponential integrator (proposed step H4b, decision 15) | DERIVED NOT IMPLEMENTED; proposed in Section 9.5, not yet a step | (P21.113) to (P21.142) |

Table: 9.1 Classification of the operations. The last five rows were added on 2026-11-13; they classify results of the open-item work that no step uses yet.

### 9.2 Status of the numerical properties

| Property | Statement | Class | Basis |
|---|---|---|---|
| Positivity of the Backward Euler step | $C^{n+1}\ge0$ for every $\Delta t>0$, $\lambda\ge0$, every temperature, $C^n\ge0$, $S\ge0$ | PROVED | Eqs. (P21.62), (P22.17); sums, products and quotients of nonnegative numbers only; V10, VT14 and VI2 show no violation |
| Subtraction-free pivots stay valid with decay | $d_j=e_j+g^E_j$ | PROVED | Eq. (P21.61) by induction |
| Inventory closure | $M_0+M_\mathrm{gen}=M+M_\mathrm{dec}+M_\mathrm{rel}$ per step and summed, also with a changing $G_R$ | PROVED for the exact linear solve; DEMONSTRATED NUMERICALLY at round-off | Eqs. (P21.70), (P21.72), (P22.16); V9 gives $10^{-15}$, VI2 gives $1.3\times10^{-15}$ |
| Outside-inventory closure | $M_0+M_\mathrm{gen}=M+M_\mathrm{out}+M_\mathrm{dec}+M_\mathrm{dec,out}$ | PROVED; DEMONSTRATED NUMERICALLY | Eq. (P21.75); V9, VI2 |
| Steady state independent of $\Delta t$ | fixed point of the unsplit step equals the semi-discrete steady state | PROVED | Eq. (P21.68) discussion; V14 shows $10^{-15}$ |
| Time accuracy of Backward Euler for decay | first order; decay error $\tfrac12\lambda^2t\Delta t$ | PROVED as a leading-order expansion; DEMONSTRATED NUMERICALLY | Eqs. (P21.64), (P21.77); V2 gives a 0.1 % match |
| Space accuracy | second order in $h$ (steady and transient) | DEMONSTRATED NUMERICALLY | V4, V5, V6a give orders 1.97 to 2.00 |
| Time accuracy of the joined scheme (decay, source and changing $D(T)$ together) | first order | CLAIMED ONLY | each part was shown separately (V3, V12, VT10); the joined run has been checked for closure and positivity (VI2) and has not been convergence-tested |
| Option A equals Backward Euler in $\tau$ | for a shared scaling factor and $\lambda=0$ | PROVED by substitution (every conductance equals $\Delta\tau_n/\Delta t_n$ times its reference value); DEMONSTRATED NUMERICALLY | Eq. (P22.57) and the text of Section 5.2.3; VT11 ($3.5\times10^{-15}$), VI6 ($4\times10^{-16}$) |
| Effective-time solution exact | under E1 to E5 | PROVED (symbolic, B3); DEMONSTRATED NUMERICALLY | Eqs. (P22.24) to (P22.33); VT10, VT15 |
| Modal recursion exact in time | piecewise-constant source, no time error | PROVED | Eq. (P21.33); V18 |
| Projection at a temperature switch | exact for the untruncated series | PROVED (completeness by the Green-function argument of B3); DEMONSTRATED NUMERICALLY | Eqs. (P22.40) to (P22.43); VT17 |
| Staircase orders 1, 2, 2 | left, midpoint, average | updated 2026-11-13: order 1 PROVED uniformly in the stiffness; order 2 PROVED for shared scaling (uniformly) and, up to a factor $1+\ln(T/h)$, on a fixed mesh (constant grows like $N^{1/2}$, DEMONSTRATED NUMERICALLY); orders near 2 for $F$ DEMONSTRATED NUMERICALLY; $V$-norm order 7/4 CLAIMED ONLY; an order above 1 uniform in the mesh stiffness for layer-specific $Q$ OPEN | Eqs. (P22.54) to (P22.56), (P22.130), (P22.140); Section 5.4.6 of the derivation; VT18; `sec5_o10_stiff_staircase.py` |
| Gauss-Legendre average with 8 points | relative error below $10^{-15}$ while the change of $\ln D$ is at most 2 | DEMONSTRATED NUMERICALLY | `b4_gl8_range.py`; VT9 |
| Truncation of the balance form | $N^{-3}$ worst case for sign-definite weights; updated 2026-11-13: a rigorous bound for the layered particle whose source and decay part falls like $(N-J)^{-3}$, and for $S=0$ and an absorbing surface a computable a posteriori tail bound | PROVED asymptotically for the homogeneous sphere; DEMONSTRATED NUMERICALLY otherwise; layered bound PROVED under completeness (O14) and the classical min-max and Sturm oscillation theorems; tail bound for $S=0$ and an absorbing surface PROVED | Section 3.5.2 of the derivation; layered weights change sign; (P21.106); (P22.90), (P22.91) |
| Rigorous truncation bound for the layered particle | updated 2026-11-13: Bound A (P21.106) with corollaries (P21.107) to (P21.110), Bound B (P21.112), for a kernel-uniform start and source | PROVED (conditional on completeness, O14, and on the classical min-max and Sturm oscillation theorems); DEMONSTRATED NUMERICALLY; conservative | Section 3.5.3 of the derivation; `sec3_o4_layered_truncation_bound.py` |
| Diffusion step limit with a temperature history | rule (P22.69) with step (P22.72) keeps the slow-mode error below $\varepsilon_1$ for option A; for option E the rule (P22.74) with $\zeta_E$ bounds it by $\varepsilon_1+\tfrac12\zeta_E^2\,\mathrm{TV}(\ln\Lambda_1)$; the F3 value with $\Lambda_1$ at the highest temperature is a sufficient cap for option A | PROVED for shared scaling (option A by (P22.69) with (P22.72), option E by (P22.74); the F3 cap for option A only); frozen-coefficient part PROVED for layer-specific $Q$ (option A for any history, option E on heating only); remainder CLAIMED ONLY | Section 5.2.5 of the derivation; `sec5_o12_variable_D_step_limit.py` |
| Start of the release quadrature with decay | start bound (P22.93), bracket (P22.94) and total (P22.98), absorbing surface, $S=0$ | (P22.93) PROVED for constant $\alpha$; bracket (P22.94) PROVED for varying $\alpha$ and $c_0\ge0$; total (P22.98) PROVED for constant $\alpha$ on the start piece and on every graded piece and under the coverage assumption R24-A3c (late truncation (P22.95)); CLAIMED ONLY on ramp pieces; layered cold start OPEN | Section 5.3.7 of the derivation; `sec5_r24_quadrature_start.py` |
| Joint effective time of a parent and daughter | exists only under N1 and N2; otherwise a two-clock cascade when each species passes R2.1 (E1), and the surrogate or the finite-volume path when a species fails it | PROVED within the class (P22.101); cascade DEMONSTRATED NUMERICALLY; cascade truncation OPEN | Section 5.3.8 of the derivation; `sec5_o11_parent_daughter_tau.py` |
| Positivity of second-order integrators | no rational order-2 scheme is positive for every step over $\mathcal S$; TR-BDF2 for $h\beta_B\le1+\sqrt2$, CN for $h\le h^*_{CN}$ (P21.125); exponential integrator for every step | PROVED; per-matrix TR-BDF2 threshold DEMONSTRATED NUMERICALLY; a fixed matrix size $N\ge3$ OPEN | Section 4.6.1 of the derivation; `sec4_p2_positive_second_order.py` |
| Modal against finite volume agreement | difference falls by a factor of at least 1.8 per halving | DEMONSTRATED NUMERICALLY | V7 |
| Bit-identical regression with v1.0 | adding exactly 0.0 leaves every float unchanged | DEMONSTRATED NUMERICALLY in Python; CLAIMED ONLY for Rust until V1 and VT16 run | Section 4.3 detail (ii) of the derivation; VI4 |
| Choice of cumulative yield | adequate when the precursor delay is short | PROVED for a single precursor (Eq. (P21.12)); an approximation otherwise | Section 3.3.2 of the derivation |

Table: 9.2 Status of the numerical properties.

### 9.3 Open items

- O1. Capture and burn-out cross sections, the Ag-110m isomeric fraction and flux histories are NOT VERIFIED, as are the daughter diffusivities, the kernel heavy-metal density $n_\mathrm{HM}$ and the energy per fission. Together these block E5, the burnup route of Eq. (P21.9) and the chains.
- O2. The warning factor of step E1 (default 0.1) has no source; it is a design proposal.
- O3. The tolerances of V7 and V6a come from the first run of the transcription and should be revisited when the Rust code runs.
- O4. Resolved for a kernel-uniform start and source (2026-11-13): Bound A (P21.106) and Bound B (P21.112) of Section 3.5.3 of the derivation bound the truncation of the modal balance form in the layered particle (PROVED under completeness and the classical min-max and Sturm oscillation theorems, so the result rests on O14; DEMONSTRATED NUMERICALLY; both conservative). For $S = 0$ and an absorbing surface, with or without decay and in effective time, the computable tail bounds (P22.90) and (P22.91) of Section 5.3.7 also apply; the start bound (P22.93) is PROVED for constant $\alpha$ and holds in effective time only with $\alpha_0$ replaced by $\alpha_-$ and each $E_n$ integrated exactly; the late truncation bound (P22.95) needs the coverage assumption R24-A3c. Still OPEN: a bound that uses the cancellation between weights of opposite sign, and a bound after a projection at a temperature switch other than the general form of Bound B.
- O5. The gas constant and the decay treatment used by the CRP-6 participants remain NOT VERIFIED. Cases V11 and V12 were run in the transcription with $R_g=8.3145$ (the value printed in IAEA, 2012, Eq. (10.5a)) and no decay. Step C2 offers the SI value and 8.3143 only; with either one both cases stay inside their tolerances (B7 review), so a third option is needed only if the review wants the printed TECDOC-1674 constant itself.
- O6. Resolved in the B7 review. Section 4 of the derivation referred to the v1.0 step at "lines 171 to 180", which is the assembly loop inside `backward_euler_step` (lines 162 to 182 in the tag, changed lines 178 and 179). The table of Section 4.3 now names both ranges.
- O7. The conversion between the fluence cutoffs $E>0.18\ \mathrm{MeV}$ and $E>16\ \mathrm{fJ}$ is NOT VERIFIED. Step C1.4 refuses fluence input with any other cutoff tag.
- O8. The ramp shape of the CRP-6 lists is assumed linear, and row 8a must be re-read from the page image.
- O9. As-run AGR-1 furnace histories are not in the corpus. The 30 $^\circ$C start of the reconstructed plan is an assumption supported only by the printed total durations.
- O10. Updated 2026-11-13 (Section 5.4.6 of the derivation). Staircase: order 1 uniform in the stiffness PROVED; order 2 PROVED for shared scaling and, up to a factor $1+\ln(T/h)$ on a fixed mesh with a mesh-dependent constant, for layer-specific $Q$ (Theorem D). OPEN: an order above 1 uniform in the mesh stiffness for layer-specific $Q$ (observed 1.75 in the test case and 1.51 to 1.96 in a second synthetic case; 7/4 conjectured as the limit, CLAIMED ONLY).
- O11. Resolved in part on 2026-11-13 (Section 5.3.8 of the derivation). Theorem 1 gives necessary and sufficient conditions for a joint effective time (N1 and N2; PROVED in the class (P22.101)); equal activation energies are necessary and not sufficient, and no linear ramp with $\theta = Q/R_g \ne 0$ qualifies when $b\lambda_P \ne 0$. Proposition 2 gives an exact two-clock cascade whenever each species passes R2.1. Still open: a truncation bound for the cascade double series; transformations outside the class; data for daughter diffusivities (O1). The steps that would use these results are proposed in Section 9.5; until then the finite-volume path is the only route.
- O12. Resolved for shared scaling on 2026-11-13 by Section 5.2.5 of the derivation: option A by (P22.69) to (P22.72), option E by (P22.73), (P22.74) with $\zeta_E$; the present F3 value with $\Lambda_1$ at the highest temperature is a PROVED sufficient cap in that case for option A (for option E the lag $L_1$ enters and $\varepsilon_1$ is not proved for that value). For layer-specific activation energies the rule bounds the frozen-coefficient part by $\varepsilon_1$ ((P22.78); option A for any history, option E on heating only); the non-adiabatic remainder $R_\mathrm{na}$ is CLAIMED ONLY to be $O(\varepsilon_D)$, supported numerically on one history, and has no proved bound. With decay the release-fraction error is $O(\zeta)$ with no sharp a priori bound. A source term is not covered.
- O13. Resolved. The values of $N_A$ and $k_B$ were read in the SI Brochure (BIPM, 2019, Section 2.2 and Table 1, p. 124, version 4.01 of June 2026).
- O14. Partly resolved. The spectral theorem used for completeness was read in Melrose (2009, Lecture 20, Theorem 15, p. 116). A citation for the compactness of an operator with a square-integrable kernel is still NOT VERIFIED (B3 item 2). O14 now also carries the truncation bounds of O4.
- O15. The helper relations (H21.1) to (H21.3) and (H22.1), (H22.2) are not rows of the B3 reconciliation. (H21.x) were proved in the check script and re-derived by substitution here; (H22.1) is a design guard; (H22.2) is a sufficient structural test.
- O16. New 2026-11-13 (Section 4.6.1 of the derivation). No second-order integrator is chosen (decision 15). OPEN: positivity of an order-2 scheme over all matrices of one fixed size $N\ge3$, which matters because one run has a fixed mesh; for the frozen-matrix exponential integrator with $D(T(t))$, which is the staircase of block R4, the part of its stiff-regime order left open by Section 5.4.6, namely an order above 1 uniform in the mesh stiffness for layer-specific $Q$ (O10); the variable-step BDF2 closure; second-order stages with $D(T(t))$.
- O17. New 2026-11-13 (Section 5.3.7 of the derivation). Start of the release quadrature in R2.4 step 3: a layered particle with a long cold start (large $\lambda t_c$) has no proved bound (OPEN); the graded Gauss-Legendre bound (P22.97) on ramp pieces is CLAIMED ONLY; a Robin surface and a source are not treated.

### 9.4 Decisions requested before any Rust is written

1. **Decision 1, source region.** Allow a single-region particle in `Config::validate`, or add a per-region source flag (default: kernel only)? The Booth cases V6a, V6b and V19 need one of the two.
2. **Decision 2, input of the source.** Take the fission rate density directly (the plan above), or also accept burnup or power? The second option needs $n_\mathrm{HM}$ or the energy per fission, which are not in the sources (O1).
3. **Decision 3, step average.** Option A (exact step average, recommended in Section 4.3 of the derivation) as the only mode, or keep option R for comparison with older runs?
4. **Decision 4, outside inventory.** Report the Backward Euler $M_\mathrm{out}$ of Eq. (P21.73) only, or also the exponential form of Eq. (P21.74)?
5. **Decision 5, warning threshold.** Keep the proposed default of 0.1 for the precursor warning, or choose another rule?
6. **Decision 6, step limits.** Make the decay limit (P21.77) and the diffusion limit (P21.78) hard caps on the step, or only warnings with the user step respected?
7. **Decision 7, scope of block R1.** Keep the modal block restricted to a kernel-uniform source and start profile for v2.0, with the general projection as FUTURE?
8. **Decision 8, conductance time level.** The task brief says conductances at $T(t^{n+1})$ (option E). The approved text recommends option A. Both are specified; the default here is A. Which one should be the default?
9. **Decision 9, jump representation.** Two history kinds (linear with strictly increasing times, and constant intervals) or one kind that allows repeated times?
10. **Decision 10, planner.** Accept the merged planner of block F: one `growth` (default 2.0), restart at `dt_first` after a source switch or a temperature jump, landing by halving the remainder, $\varepsilon_D=0.1$, the factor 2 in the guard, and a FIXED mode that reproduces the v1.0 grid?
11. **Decision 11, gas constant.** SI default with a `FIXED_8_3143` option (as proposed), or a per-row value from the register?
12. **Decision 12, access to eigenfunctions.** Block R3 needs the layer states of the shooting solution, which are private in `sturm_liouville.rs`. Add public accessors to that file, or copy the shooting code into the new module?
13. **Decision 13, reference temperature of the effective time.** The highest temperature of the history (proposed) or the temperature at the end of the main hold?
14. **Decision 14, staircase code.** Keep block R4 in the library behind a test-only switch, or only as a test helper?
15. **Decision 15, integrator (added 2026-11-13).** Backward Euler only, Backward Euler with TR-BDF2 as an option (with or without the positivity cap $h\beta_B\le1+\sqrt2$), or Backward Euler with the exponential integrator for constant-temperature segments (Section 4.6.1 of the derivation)?
16. **Decision 16, proposals of Section 9.5 (added 2026-11-13).** Which of the proposed step changes of Section 9.5 should enter the pseudocode before any Rust is written?

The work stops here for the user's review. No Rust or test file of the repository has been touched. The other new repository files are documentation, data and Python only: the check scripts of Section 10 and `scripts/derivation_checks/`, the parameter register under `data/` with its scripts in `scripts/data_register/`, and the change list `docs/development/v1_to_v2_changes.md`.

### 9.5 Proposed step changes from the open-item work (2026-11-13, not applied)

The open-item work of 2026-11-13 (Sections 3.5.3, 4.6.1, 5.2.5, 5.3.7, 5.3.8 and 5.4.6 of the derivation) suggests the step changes below. None of them is applied to the blocks above: every verified result left the existing steps valid, so no algorithm was changed, and only notes, status rows and open items were updated. The two notes that corrected a statement are the F3 note, which now records that the existing value is a proved sufficient cap for shared scaling with option A, and the R2.4 step 3 note, whose remark that the balance form avoids the start problem was wrong ((P22.81)). A third correction is a comment only: the order annotations of the MIDPOINT and AVERAGE lines in step R4.1 now read "order 2 for F; on a fixed mesh up to 1+ln(T/h)" and "order 2 as MIDPOINT" in place of "order 2", and the steps themselves are unchanged. Each proposal below is DERIVED NOT IMPLEMENTED and waits for decision 16.

**Block R1 (from O4, Section 3.5.3).**

- New step R1.0, mode count, before R1.1: from the radii and diffusivities compute $\tau$, $a$, $c$, $J$ and $\Lambda^*$; with $K=$ `gen_max + decay_constant * (inv0 + gen_total_max)` and the user tolerance choose `n_modes` by (P21.110), which needs $N>J$ and $\ell_{N+1}>\Lambda^*$, together with the start-inventory condition $C_w(\ell_{N+1})M_0X_N(t_1)\le\varepsilon$ at the first output time $t_1$; pass `lambda_max` $\ge\min\big(D_\mathrm{max}(N\pi/R)^2,\ a(N+J)^2\big)$ to `ModalRelease::new` so that the first $N$ modes are all included. The condition $\ell_{N+1}>\Lambda^*$ can force a large $N$ (2020 modes in one verification case).
- Step R1.5, diagnostics: add `trunc_bound_rel` from (P21.106) at each output time, with $X_N$ from (P21.105) and $\Lambda_\mathrm{cut}=$ `lambda_covered`, and `trunc_bound_FB`, `trunc_bound_RB` from (P21.107) and (P21.108). Bound B (P21.112) needs $\alpha_n$ and $\beta_n$ separately and stays FUTURE while v1.0 keeps only $w_n$.
- Step R3.5: note that Bound A does not apply after a switch; the general Bound B is the FUTURE route there.

**Block F (from O12, Section 5.2.5, and P2, Section 4.6.1).**

- Step F3: compute $\Theta_\mathrm{end}=\int_0^{t_\mathrm{end}}(\Lambda_1+\lambda)\,dt$ (shared scaling: $\Lambda_1^\mathrm{ref}\tau(t_\mathrm{end})+\lambda t_\mathrm{end}$ from (P22.22); otherwise composite Gauss-Legendre over each history segment with $\Lambda_1(T)$ from the tridiagonal eigenvalue problem or with the upper bound $\hat\Lambda_1$), then $\zeta=2\varepsilon_1/\Theta_\mathrm{end}$ for option A or $\zeta_E$ of (P22.74) for option E, citing (P22.69). For constant $T$ this gives the present `dt_diff`, and grid mode FIXED leaves the block before F3, so cases V1 and VT16 are unchanged.
- Step F4: take `dt_diff` out of `dt_cap`, because it changes from step to step, and add the estimate $\Theta_\mathrm{end}^2/(2\varepsilon_1)$ of (P22.71) to the `step_limit` test.
- Step F5: a new sub-step after F5.3 computes `dt_diff` by (P22.72) with two evaluations of $\Lambda_1$; in F5.4 the step becomes the minimum of `dt_cap`, `grow_cap`, `dt_T` and `dt_diff`. Keep `dt_T` of (P22.20) for layer-specific $Q$, since it controls the remainder $R_\mathrm{na}$. With two nuclides on the finite-volume path, $\zeta$ is the minimum over the nuclides.
- If a second-order integrator is chosen (decision 15): F3 uses (P21.142) with the error constant $c_r$ of that scheme (and the BDF2 start term); F4 gains an optional positivity cap $h\le M_r/\beta_B$ ($M_r=2$ for CN, $1+\sqrt2$ for TR-BDF2) or $h\le h^*_{CN}$ from (P21.125), hard cap or warning as in decision 6.

**Blocks H and J (from P2, Section 4.6.1; only if decision 15 adds an integrator).**

- New step H4b: reuse the subtraction-free elimination of H4.3 with $\tau=h/2$ (CN) or $\tau=\beta h$ (TR-BDF2, two solves with one matrix), or the exponential step (P21.137). The property line of H4.4 holds for H4b only in the exponential case, and there only with (P21.139).
- Steps J1.3 to J1.5: the increments of $M_\mathrm{dec}$, $M_\mathrm{rel}$ and the outside inventory use the quadrature of the chosen scheme: trapezoid (P21.126), stage weights (P21.134), modified inventory (P21.130) or exact integrals (P21.140). Step J1.6: the exact test "minimum below 0.0 is a failure" holds only for BE and the uniformised exponential; other schemes need a tolerance or the step cap.

**Block R2 (from R24, Section 5.3.7, and O11, Section 5.3.8).**

- R2.4 step 3, sub-steps: (3a) compute $M_\mathrm{rel}$ by (P22.82); the balance form is the same number by (P22.81) and is not a separate option. (3b) Start piece: for a bare kernel with a uniform start the image form (P22.86) or (P22.87) on $[0,t_0]$ with $t_0=\min(t_B,t)$ (the 16-point quadrature of (P22.87) on a ramp has no proved error bound); otherwise the modal start (P22.83) with $t_0=\min(t_1,t)$ and the bracket (P22.94) (needs $c_0\ge0$) when $\alpha$ varies on $[0,t_0]$. (3c) Late pieces: breakpoints refined to $\tau(b)/\tau(a)\le2$, 8-point Gauss-Legendre, bound (P22.97) (PROVED for constant $\alpha$ per piece, CLAIMED ONLY on ramp pieces). (3d) Mode count and $t_0$ by the rule of Step 7 of Section 5.3.7, with $\bar W_N$, $\bar V_N$ from (P22.90), (P22.91). (3e) Return the bound (P22.98) with the value; warn when it exceeds the tolerance, and warn when the cold-start term $\lambda t_0\bar W_N$ dominates in a layered particle (O17). Step 5 stays as it is for the reported times; the start piece no longer needs the series to be valid on $[0,t_0]$. All of this assumes an absorbing surface and no source.
- R2.1 chain mode `check_chain(laws_P, laws_D, history)`: run R2.1 on each species and return `Identical` when (H22.2) holds across the two species with ratio 1 in every layer, `SharedClock` when it holds with another ratio (a closed form then exists for $b\lambda_P=0$ by two copies of (P22.27), on isothermal intervals by (P21.21) and on histories that obey (P22.106) by (P22.108); by Corollary 2 no linear ramp with $Q\ne0$ qualifies when $b\lambda_P\ne0$), `TwoClocks` when each species passes alone, and `NotExact` otherwise.
- R2.4 step 6 for `Identical`: $M_P=e^{-\lambda_Pt}\hat M_P(\tau)$ and $M_D=e^{-\lambda_Dt}\hat M_D(\tau)+b\lambda_P\frac{e^{-\lambda_Pt}-e^{-\lambda_Dt}}{\lambda_D-\lambda_P}\hat M_P(\tau)$, both $\hat M$ from the v1.0 series at the same $\tau$ ((P22.109)).
- New block R2.5, two-clock cascade: eigenpairs of both species at their own reference temperature; overlap $O$ by (P21.19), once; $\tau_P$ and $\tau_D$ by R2.2 and R2.3; the double integrals $J_{nm}$ by Gauss-Legendre on the pieces between breakpoints, graded towards both ends of each piece; inventory by (P22.114) and release by the daughter balance of Section 5.3.8. Its truncation rule is OPEN (O11). Routing: identical diffusivities with E1 for the pair go to R2.4 step 6; each species passing R2.1 goes to R2.5; otherwise the finite-volume path, with the surrogate of Theorem 3 ((P22.118) to (P22.120)) as a diagnostic only.

**Block R4 (from O10, Section 5.4.6).**

- New sub-step R4.0: place interval ends on every kink and jump of $T(t)$ (Theorem B: exact for piecewise-constant histories, order 1 for a misaligned jump).
- Step R4.2: measure the observed order on a smooth functional such as $F$; as in the R4 note, a $V$-norm acceptance test requires an order of at least 1 only, and order 2 of MIDPOINT and AVERAGE is tested on $F$ (an order above 1 in the $V$-norm rests on numerical evidence only, and its uniform proof is OPEN, O10). Table 5.6, row (P22.54), (P22.55): add a pointer to (P22.139) for the interface defect. An optional case VT18b can test (P22.130).
- With the exponential integrator of decision 15, the frozen-matrix exponential on the finite-volume matrix is itself a positive staircase; whether R4 then becomes a production path is part of decision 15.

## 10 Evidence

All commands use `.venv/Scripts/python.exe -I -X utf8`.

| Script | Folder | Result when this page was written |
|---|---|---|
| `b4_phase2_pseudocode_integration.py` | `scripts/derivation_checks/` | 32 of 32 PASS |
| `b4_decay_source_pseudocode.py` | `scripts/derivation_checks/` | 55 of 55 PASS (re-run) |
| `b4_arrhenius_checks.py` | working folder, not in the repository | 41 of 41 PASS (re-run) |
| `sec5_arrhenius.py`, `sec5_effective_time.py`, `sec5_fv_shared_alpha.py`, `sec5_projection_step.py`, `sec5_staircase.py`, `sec5_schedules.py` | `scripts/derivation_checks/` | 25, 16, 10, 9, 9 and 17 checks PASS (B3 re-run, not repeated here) |
| `sec6_pseudocode_checks.py` | `scripts/derivation_checks/` | 30 of 30 PASS in the part draft; not reviewed line by line |
| `sec3_o4_layered_truncation_bound.py`, `sec4_p2_positive_second_order.py`, `sec5_o12_variable_D_step_limit.py`, `sec5_r24_quadrature_start.py`, `sec5_o11_parent_daughter_tau.py`, `sec5_o10_stiff_staircase.py` (open items, 2026-11-13) | `scripts/derivation_checks/` | 91, 41, 38, 25, 24 and 22 PASS, 0 FAIL, each ending with ALL PASS (run of 2026-11-13; output in Appendix A.2 of the derivation) |

Table: 10.1 Check scripts.

The new script is a Python transcription of the merged planner, of blocks H and J, and of the v1.0 run loop. It does not import the other scripts and writes no file. It checks:

- the planner landing on every event, the cap rules and the restart after a switch (VI1);
- the closure inside and outside, positivity and the generated amount for a joined run, and the failure of the closure when the first-step $G_R$ is used (VI2, VI3);
- bit identity with the v1.0 run loop for constant data (VI4);
- the left-limit rule at a temperature jump and the failure of the naive rule (VI5);
- the difference between the geometric grid with growth 1 and the v1.0 constant step (finding X2);
- option A against Backward Euler in the effective time (VI6);
- the structural test of the effective-time shortcut on three inputs;
- the closed form with $E_2$ against adaptive quadrature at a change of 16 in $\ln D$ (relative difference $2.3\times10^{-15}$);
- the numbers quoted here: the four decay constants, the Cs-137 source, the two step limits of block F, the scalar example, the five $t_{99}$ values and the three temperature step bounds.

Key output lines of the new script:

```text
PASS  every event time is the end of a step by float equality  8 of 8
PASS  joined run, option A: inside closure below 1e-12  1.31e-15
PASS  negative test: booking with the first-step surface conductance breaks the closure (above 1e-8)  1.22e-05
PASS  v2 loop equals the v1.0 transcription bit for bit over every step  200 steps, dt 2000.000000 s
PASS  inventory after the run: option A against BE in tau agrees to 1e-11  3.95e-16
PASS  naive rule is a trap: it produces steps below 1 ms next to the jump
SUMMARY: 32/32 PASS
```

What the scripts do not establish: they check transcriptions in Python floating point, so bit identity (V1, VT16, VI4), the behaviour of the Rust compiler and the speed of the Rust code stay open until Rust tests exist. Case V8 and the transient Booth tolerance of Section 4.8 test 6 were not run. The joined scheme has not been convergence-tested.

## References

Basu, S. K., & McCutchan, E. A. (2020). Nuclear data sheets for A = 90. *Nuclear Data Sheets, 165*, 1-329. https://doi.org/10.1016/j.nds.2020.04.001

Browne, E., & Tuli, J. K. (2007). Nuclear data sheets for A = 137. *Nuclear Data Sheets, 108*, 2173-2318. https://doi.org/10.1016/j.nds.2007.09.002

Bureau International des Poids et Mesures. (2019). *The International System of Units (SI)* (9th ed., version 4.01, June 2026). BIPM. https://doi.org/10.59161/AUEZ1291 (Section 2.2, pp. 123-124, and Table 1, p. 124; kelvin, p. 129, and mole, p. 130, both in Section 2.3.1; English text PDF opened from https://www.bipm.org/en/publications/si-brochure)

Gürdal, G., & Kondev, F. G. (2012). Nuclear data sheets for A = 110. *Nuclear Data Sheets, 113*, 1315-1561. https://doi.org/10.1016/j.nds.2012.05.002

idaholab. (2026). *moose* [Computer software]. GitHub. https://github.com/idaholab/moose (`force_step_every_function_point`, via audit 08)

International Atomic Energy Agency. (1997). *Fuel performance and fission product behaviour in gas cooled reactors* (IAEA-TECDOC-978). IAEA. (Tables A-1 to A-3, the source of the Cs diffusion correlations in the register; URL NOT VERIFIED.)

International Atomic Energy Agency. (2012). *Advances in high temperature gas cooled reactor fuel technology* (IAEA-TECDOC-CD-1674) [CD-ROM]. IAEA. (Eqs. 7.1 to 7.8, printed pp. 317-319; Eqs. 10.1 to 10.7, printed pp. 529-544; Table 8.1; URL NOT VERIFIED.)

International Atomic Energy Agency, Nuclear Data Section. (n.d.). *LiveChart of Nuclides, data download API v1* [Data set]. Retrieved November 5, 2026, from https://www-nds.iaea.org/relnsd/vcharthtml/api_v0_guide.html (source of the ENSDF half-lives and of the JEFF-3.1.1 fission yields quoted here)

Melrose, R. (2009). *Lecture notes for 18.102 Introduction to functional analysis, Spring 2009* (Lecture 20, Theorem 15, p. 116) [Course notes]. MIT OpenCourseWare. https://ocw.mit.edu/courses/18-102-introduction-to-functional-analysis-spring-2009/ (instructor named on the course page; the lecture PDF prints no author)

STRATO project. (2026). *STRATO v1.0.0* [Computer software]. https://github.com/weewzr/STRATO/tree/v1.0.0 (files `src/numerical/finite_volume.rs`, `src/analytical/sturm_liouville.rs` and `src/config.rs`, quoted by line number).

The equations (P21.1) to (P21.142) and (P22.1) to (P22.142) are those of Sections 3 to 5 of the derivation (`docs/mathematics/STRATO_Phase2_Derivations.md`) and carry their own references there; (P21.93) to (P21.142) and (P22.60) to (P22.142) were added on 2026-11-13.
