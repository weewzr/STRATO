# Testing and validation

## 1 Existing v1.0 suite (must stay green)

| Check | Command | What it asserts |
|---|---|---|
| Unit and integration tests | `cargo test --all-targets` | finite-volume unit tests in `src/numerical/finite_volume.rs`; Code B against Code C in `tests/finite_volume_vs_analytical.rs`; golden regression in `tests/phase1_regression.rs` |
| Golden regression | part of the above | `tests/phase1_regression.rs` asserts `tests/golden/v1.0/timeseries.csv`, `final_profile.csv` and the analytical series of `comparison_fv_vs_analytical.csv` (points after 2e4 s), relative tolerance 1e-12. `convergence_study.csv`, `main_run_stdout.txt` and `verify_stdout.txt` are not asserted by any test |
| Verification example | `cargo run --release --example verify` | prints `FV_VERIFICATION PASS`; every printed claim is asserted |
| Synthetic run | `cargo run --release` | uses `inputs/synthetic/synthetic_five_layer.toml` and writes `results/synthetic-five-layer/`. CI runs it and uploads its standard output as an artefact, but does **not** compare the outputs with `tests/golden/v1.0/` (correction of 2026-11-24). The comparison is made by hand as in `docs/development/provenance.md` Section 3: `verify` output identical to the golden file apart from the `VERIFY_RUNTIME_S` line; main-run standard output identical after normalising path separators; the four CSV files byte for byte. Automating it (in `web.yml`, since `ci.yml` stays unchanged per `prompt/TECHNICAL_ARCHITECTURE.md` Section 7) is open work |
| Lint and docs | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo doc` with `-D warnings` | as in `.github/workflows/ci.yml` |
| Derivation checks | `python -I -X utf8 scripts/derivation_checks/<script>.py` (project venv) | independent Python checks of the derivations; evidence, not product code |

Run the full suite before and after any change that touches Rust. A change to `src/`, `tests/`, `examples/` or `inputs/` without this evidence is not allowed.

## 2 Tolerance policy

| Comparison | Tolerance |
|---|---|
| Wrapper versus `finite_volume::run`, same platform | exact (bit-identical) |
| Wrapper or wasm versus `tests/golden/v1.0/` | relative 1e-12 (the existing regression tolerance) |
| Browser wasm versus native, same inputs | relative 1e-12 per value, identical array lengths |
| Code B versus Code C | as recorded in `tests/golden/v1.0/comparison_fv_vs_analytical.csv`; discretisation error, not a defect |
| Inventory balance | no worse than the v1.0 level |

Keep tolerances as named constants in one file per language (`crates/strato-wasm/src/tolerances.rs`, `web/src/solver/tolerances.ts`), citing this section. A golden mismatch means the code is wrong: fix the code, keep the golden file (NUS-H2 practice, `engine/vv/src/golden.rs`).

## 3 New tests by phase

| Phase | Tests |
|---|---|
| 1 | Vitest store and component tests; Playwright at 375x812, 768x1024, 1280x800, 1440x900, 1920x1080; block order A to E at every viewport; the fold requirement (Section 3.3); keyboard-only navigation; `prefers-reduced-motion`; no console errors; no failed network requests |
| 2 | term assembly for the six builder stages and for all 8 combinations of the three Phase 6 switches; the status-system tests (Section 3.1); the "no active toggle without solver support" tests (Section 3.2); documentation and code anchors resolve; KaTeX renders without error; no numbers shown apart from equation numbers; the fold requirement with the builder in place |
| 3 | wrapper equals `finite_volume::run` exactly and golden files at 1e-12; invalid and placeholder inputs rejected; no panic crosses into JavaScript; wasm export and size check; browser parity against `web/public/data/golden_v1.json`; worker handshake, timeout and recovery; precompute regeneration identical |
| 4 | colour map on a known profile; wrapper test that the layer inventories of `run_v1` sum to the total; e2e check that the displayed shares equal the wrapper's `layer_share` values (no sums in TypeScript); WebGL fallback; playback frame rate measured |
| 5 | wrapper test that the `compare_v1` difference equals modified minus baseline; e2e check that the plotted difference equals the wrapper's values (no subtraction in TypeScript); downloads round-trip; comparison and validation toggles independent |
| 5.5 | nine species entries and three regimes render with every required heading; every status shown equals `data/research/mechanisms.csv`; every catalogue switch is disabled (Section 3.2); a gallery item shows an image only if its row is `screenshot` or `reconstruction` with an existing file and provenance, placeholders show the word PENDING and no image, and the two image kinds carry different labels; every `[ref:...]` renders as a link; reference links resolve or are marked NOT VERIFIED (link check); screenshots at 1440x900 and 390 px wide inspected |
| 6 | all cases of Section 5; full v1.0 suite unchanged (6.1 before 6.2); `capabilities` gains a mechanism id only in the commit whose cases pass; Section 3.2 re-run with the new capabilities: a switch becomes active only when its id is in `capabilities` and its status is VERIFIED |
| 7 | plotted measurements equal the CSV; metrics recomputed independently in a test; a VALIDATED row in `mechanisms.csv` names validation rows that exist and pass |
| 8 | axe-core accessibility audit via Playwright; performance budget; reproducible build; link check; fold requirement and block order re-run on the final build in both themes |

### 3.1 Research-status system

- `strato-webtools check-catalogue` (crates/strato-webtools) passes on the repository and, run against its own fixture folders, fails with the matching FAIL line for each of: a status that is not one of the five words (including a lower-case or misspelt word such as `Verified` or `PLANNED`), a `reference_ids` id or a `[ref:...]` missing from `references.csv`, a missing entry file, a duplicate `(mechanism_id, species, material_domain)`, a tenth or a missing species, a nuclide absent from `data/nuclear/decay.csv` without `(not in register)`, a VERIFIED row without evidence under `tests/`, a VALIDATED row without `validation_ids`, a `screenshot` row without file or provenance, a `placeholder` row with a file, and a stale `research_catalogue.json`.
- A Vitest guard fails if any of the five status words appears as a string literal in `web/src/` outside `web/src/generated/` and the one module that defines the status union and its constants (`web/src/research/status.ts`, which `isAvailable()` imports); statuses of mechanisms come from the generated JSON only.
- A component test renders every chip from a fixture catalogue and checks word, shape class and accessible name for all five statuses; an e2e test compares every status shown on Explore and Research with the generated JSON.
- The builder's stage statuses on 2026-11-24 equal the table recorded in `prompt/PROGRESS.md` (M7) until a reviewed change updates both.

### 3.2 No active toggle without solver support

- Unit test of `isAvailable()` over every combination of status (five words) and capability (present, absent): it is true only for VERIFIED with the capability present.
- Component test: with the v1.0 capabilities `["diffusion_layered"]`, every mechanism switch and every species switch is rendered with `aria-disabled="true"`, stays off after click, Enter and Space, and announces why; with a fixture where a mechanism is VERIFIED but absent from `capabilities`, and one where it is present but only IMPLEMENTED, it is still disabled.
- Client test: the solver client never sends a request naming a mechanism for which `isAvailable()` is false; the wrapper test shows that such a request returns `rejected`.
- e2e test: selecting builder stages 2 to 6 before Phase 6 shows the "Equation preview only" banner, leaves the simulation's input hash and result unchanged, and sends no worker request.

### 3.3 Fold requirement at 1440 x 900

Playwright at a 1440 x 900 CSS-pixel viewport, device scale factor 1, light and dark themes, with and without reduced motion, after fonts have loaded, scroll position 0, default case: the bounding box of the equation block (A, the typeset equation of the active stage and its stage picker) and the bounding boxes of the particle cross-section and of the release-fraction readout of block B, each with its result label, lie entirely within the viewport. The test also asserts that the top edges of blocks A, B, C, D and E increase in this order, at 1440 x 900 and at 390 px wide (device emulation). It runs for stage 1 and for the stage with the tallest equation.

## 4 Browser self-test

Adopt the NUS-H2 pattern (`frontend/selftest.js`, `scripts/selftest.py` in https://github.com/weewzr/NUSH2 at `b4817b3`): with `?selftest=1` the built site runs checks and writes a JSON verdict to a hidden element and to `document.body.dataset.selftest` (`pass` or `fail`); CI reads it, also with reduced motion. STRATO checks: build stamp equals commit, wasm loads, synthetic run matches golden values, every section renders, every research status shown equals the generated catalogue, no mechanism switch is active outside `capabilities`, no console error, no failed request. The self-test compares values only and computes no physics.

## 5 Phase 2 verification cases (web Phase 6)

Take cases, reference values and tolerances from `docs/mathematics/STRATO_Phase2_Pseudocode.md` and `docs/development/phase2_review_package.md` (Table 3.1). They include: reduction to v1.0 when $\lambda=0$, $S=0$, $T$ constant (cases V1, VT16, VI4, bit-identical); decay identity (P21.80); inventory closure (P21.42, P21.44, P21.75); positivity (P21.62, P22.17); Booth sphere with source and decay (P21.48); CRP-6 cases 1a and 1b (P21.91, with the open gas-constant question); multilayer steady state (P21.84); Arrhenius sum and crossover (P22.1, P22.7); shared-$\alpha$ effective time (P22.57). Copy each reference value from the repository with its location; never type one from memory.

## 6 Verification reporting

CI writes `verification_report.json` (check, quantity, value, tolerance, pass or fail, commit, input hash, command). The Verify section renders it. A failed check is shown as failed, never hidden.

## 7 Validation protocol (web Phase 7)

- Measurements: `data/experimental/release_measurements.csv`; show digitising uncertainty for digitised rows; flag NOT VERIFIED rows.
- Predictions: only COMPUTED v2.0 runs with matching nuclide, geometry (`data/geometry/geometry.csv`) and temperature history (`data/temperature/temperature_histories.csv`).
- Metrics: predicted/measured ratio per point, its geometric mean and spread, log-ratio RMS, number of points. No fitting unless the owner asks, and then disclosed on screen.
- Verification and validation appear in separate sections.
- Status words per validation row (from the NUS-H2 validation matrix practice): `pass` or `fail` only against a stated limit, `reported` when no limit exists, `not_run` with a reason. Each row records `data_use`: `independent` (the measurement played no part in choosing parameters) or `calibration` (it did, so agreement proves only the fit). Tolerances are frozen when a row is added; a changed tolerance is a new row with its reason. A failed row stays failed.

## 8 On failure

Stop the phase; never relax a tolerance or delete a test. Fix new code. If verified v1.0 code seems wrong, record it in `prompt/PROGRESS.md` and do not change `src/` without the owner. If a fix needs an owner decision, record a blocker and continue with independent work.
