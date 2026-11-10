# Scientific requirements

This file fixes what the web demo may claim. It ranks second in authority, after the owner decisions in `prompt/PROGRESS.md`.

## 1 Sources of truth

| Topic | File |
|---|---|
| v1.0 mathematics (teaching level) | `docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md` |
| v1.0 equation to pseudocode to Rust map | `docs/mathematics/v1.0.0/STRATO_Final_Deliverable.md` |
| Phase 1 historical record | `docs/mathematics/phase1/` |
| Phase 2 derivations, equations (P21.1) to (P21.92) and (P22.1) to (P22.59) | `docs/mathematics/STRATO_Phase2_Derivations.md` |
| Phase 2 pseudocode and verification cases | `docs/mathematics/STRATO_Phase2_Pseudocode.md` |
| Every v1.0 to v2.0 change | `docs/development/v1_to_v2_changes.md` |
| Phase 2 review package and decisions D1 to D14 | `docs/development/phase2_review_package.md` (Section 7) |
| Parameters and their status | `data/reference/STRATO_Parameter_Register.md`, `data/*/*.csv` |
| Verified code | `src/` at tag `v1.0.0` (unchanged in `v1.0.1`) |

If this file and one of those disagree, the repository document wins and this file must be corrected in the same commit.

## 2 The v1.0 model (IMPLEMENTED)

Five concentric spherical regions $i = 1..5$ (kernel, buffer, IPyC, SiC, OPyC) with constant diffusivity $D_i$ in each:

$$\frac{\partial c}{\partial t} = \frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_i \frac{\partial c}{\partial r}\right), \qquad R_{i-1} < r < R_i$$

- Regular centre: $\partial c/\partial r = 0$ at $r = 0$.
- Perfect interfaces: $c$ and $D_i\,\partial c/\partial r$ continuous at each $R_i$.
- Absorbing outer surface: $c(R_5, t) = 0$.
- Initial inventory uniform in the kernel only.
- Release fraction $F(t) = M_\mathrm{rel}(t)/M_0$.

Two independent solutions exist in `src/`:

- **Code B**, `src/numerical/finite_volume.rs`: conservative finite volume with exact shell volumes, series-resistance face conductances, Backward Euler, subtraction-free tridiagonal elimination, Kahan-summed release and a balance residual.
- **Code C**, `src/analytical/sturm_liouville.rs`: multilayer Sturm-Liouville eigenfunction series with Sturm-count eigenvalue isolation, modal weights, a reported accuracy floor and a validity time.

Verified numbers (synthetic case `inputs/synthetic/synthetic_five_layer.toml`; source `tests/golden/v1.0/` and `docs/mathematics/v1.0.0/`): $F(2\times10^5\,\mathrm{s}) = 0.43255563909304$, first eigenvalues $3.273255671175805\times10^{-6}$ and $7.791893309400181\times10^{-5}\,\mathrm{s^{-1}}$, observed spatial order 2.00, temporal order 1.00 against a same-mesh reference, balance residual at most about $2.8\times10^{-13}$ of the initial inventory. Re-read these from the repository before displaying them.

The synthetic case is a verification case. Its parameters are not physical TRISO data and the demo must say so.

## 3 Phase 2 extensions (DERIVED NOT IMPLEMENTED)

From `docs/mathematics/STRATO_Phase2_Derivations.md` and `docs/development/phase2_review_package.md`:

| Mechanism | Change to the equation | Key equations | Status |
|---|---|---|---|
| Radioactive decay | adds $-\lambda c$, with $\lambda = \ln 2 / t_{1/2}$ | (P21.1), (P21.7) | DERIVED NOT IMPLEMENTED |
| Fission-product generation | adds $+S(r,t)$, kernel source $S = \dot F\,y\,\chi_1/N_A$ | (P21.1), (P21.8) | DERIVED NOT IMPLEMENTED |
| Inventory accounting | $M_0 + M_\mathrm{gen} = M + M_\mathrm{dec} + M_\mathrm{rel}$ | (P21.42), (P21.44) | DERIVED NOT IMPLEMENTED |
| Temperature-dependent diffusivity | $D_i \to D_i(T(t)) = \sum_m D_{0,i,m}\,e^{-Q_{i,m}/(R_g T)}$ | (P22.1), (P22.15) | DERIVED NOT IMPLEMENTED |
| Effective-time and modal projection | exact only under conditions E1 to E5 | (P22.26) to (P22.44) | DERIVED NOT IMPLEMENTED |
| Activation products (Ag-110m, Cs-134, Eu-154) | two-step source | (P21.16) | FUTURE |
| Parent-daughter chains | modal coupling | (P21.20) | FUTURE |

The full extended equation is

$$\frac{\partial c}{\partial t} = \frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_i(T(t)) \frac{\partial c}{\partial r}\right) - \lambda c + S(r,t).$$

The web demo may show these terms in the equation explorer, marked as planned, but no result may use them until web Phase 6 is complete.

Model assumptions A1 to A6 (derivations Section 3.1): spherical symmetry and Fickian diffusion with constant $D_i$ during a step; perfect interfaces; absorbing outer surface and regular centre; first-order decay with the same $\lambda$ in every layer; uniform kernel source (no recoil, no precursor migration); released atoms never return. The temperature is uniform across the particle, $T = T(t)$.

## 4 Owner decisions D1 to D14 (gate for web Phase 6)

Phase 6 needs the owner's choices, recorded in `prompt/PROGRESS.md` section "Owner decisions". The proposed defaults below come from `docs/development/phase2_review_package.md` Section 7 and are **not approved** until the owner writes a choice.

| Id | Topic | Proposed in the review package |
|---|---|---|
| D1 | Nuclides first | Cs-137, Sr-90, Kr-85, I-131; Ag-110m once activation data exist |
| D2 | Cumulative or independent yields | cumulative $y^C$ only, keep the precursor warning |
| D3 | Activation products | keep FUTURE |
| D4 | Diffusivity set and gas constant | choose among TECDOC-978 FRG, Collin et al. (2016), Hales et al. (2021); SI gas constant with a `FIXED_8_3143` option |
| D5 | Temperature histories | AGR-1 plans and/or CRP-6 schedules; reference temperature = highest of the history |
| D6 | Time integration of decay | Backward Euler with implicit decay (P21.58); outside inventory by (P21.73) |
| D7 | Modal projection under piecewise temperature | scope of block R3; accessors versus a copy of shooting code |
| D8 | Conductance time level | option A, step-average diffusivity |
| D9 | Source region | single-region particle or a per-region source flag |
| D10 | Source input | fission rate density only |
| D11 | Step average of the source | exact step average (A) only, or keep the rectangle rule (R) |
| D12 | Step limits | hard caps or warnings |
| D13 | Scope of modal block R1 | kernel-uniform source and start profile only |
| D14 | Step planner | merged planner of pseudocode block F |

While any row is PENDING: Phase 6 code is not written, the Phase 2 toggles stay disabled with the text "Planned: awaiting reviewed implementation", and the roadmap continues with independent work.

## 5 Parameters and provenance

- Use only values from `data/*/*.csv` and `data/reference/STRATO_Parameter_Register.md`. Each row has a `status` of `verified`, `digitised` or `NOT VERIFIED`.
- A `NOT VERIFIED` row may be shown, but any result that depends on it carries a visible warning naming the row id.
- `digitised` values show their digitising uncertainty.
- Synthetic benchmark rows (CRP-6 cases and similar) are for verification only and never appear as physical predictions.
- Activation products are marked in `data/nuclear/fission_yields.csv` as not to be used as a fission-yield source; respect that.
- Every displayed parameter links to its register row and source (author, year, table or page).

## 6 Verification versus validation

- **Verification**: the code solves the equations correctly. Evidence: Code B against Code C, convergence orders, balance residual, golden regression, native/wasm parity. Show this in the verification panel.
- **Validation**: the model matches reality. Evidence: comparison with measured release (`data/experimental/release_measurements.csv`). This needs Phase 2 physics (temperature histories, Arrhenius diffusivity, decay), so a v1.0-only model is never compared with heating-test measurements. Until Phase 6 is complete, the validation graph shows measurements with provenance and the message "Model predictions require Phase 2 physics".

Never call agreement on the synthetic case "validation".

## 7 Result labels

| Label | Meaning |
|---|---|
| `COMPUTED v1.0` | produced in this session by the v1.0 solver (wasm or native) from the shown inputs |
| `COMPUTED v2.0` | produced by the reviewed Phase 2 implementation (after web Phase 6) |
| `PRECOMPUTED` | produced earlier by the native STRATO binary; the file records commit, input hash and command |
| `ILLUSTRATIVE` | a schematic or animation not derived from solver output |
| `MEASURED` | an experimental value from `data/experimental/`, with source and uncertainty |

## 8 Scientific independence from NUS-H2

NUS-H2 (https://github.com/weewzr/NUSH2) and the archive https://github.com/weewzr/raystorage are a separate hydrogen-plant project. Only engineering practices transfer (see `prompt/RAYSTORAGE_INTEGRATION.md`). Do not merge numerical engines, import process equations, transfer physical constants, reuse NUS-H2 verification results as evidence for STRATO, replace STRATO's analytical reference solution, or call an archive file validated without checking it.

## 9 Forbidden

- Inventing or tuning parameters to fit measurements without disclosure.
- Comparing v1.0 constant-temperature results with heating-test data.
- Changing verified v1.0 code without the full regression suite passing.
- Implementing any equation in JavaScript or TypeScript.
- Hiding a failed check, relaxing a tolerance to pass, or dropping a `NOT VERIFIED` flag.
- Choosing D1 to D14 on the owner's behalf.
