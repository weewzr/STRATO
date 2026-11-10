# Testing and validation

## 1 Existing v1.0 suite (must stay green)

| Check | Command | What it asserts |
|---|---|---|
| Unit and integration tests | `cargo test --all-targets` | finite-volume unit tests in `src/numerical/finite_volume.rs`; Code B against Code C in `tests/finite_volume_vs_analytical.rs`; golden regression in `tests/phase1_regression.rs` |
| Golden regression | part of the above | `tests/golden/v1.0/timeseries.csv`, `final_profile.csv` and the analytical series, relative tolerance 1e-12 |
| Verification example | `cargo run --release --example verify` | prints `FV_VERIFICATION PASS`; every printed claim is asserted |
| Synthetic run | `cargo run --release` | uses `inputs/synthetic/synthetic_five_layer.toml`; CI compares outputs with `tests/golden/v1.0/` |
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
| 1 | Vitest store and component tests; Playwright at 375x812, 768x1024, 1280x800, 1920x1080; keyboard-only navigation; `prefers-reduced-motion`; no console errors; no failed network requests |
| 2 | term assembly for all 8 toggle combinations; documentation anchors resolve; KaTeX renders without error; no numbers shown |
| 3 | wrapper equals `finite_volume::run` exactly and golden files at 1e-12; invalid and placeholder inputs rejected; no panic crosses into JavaScript; wasm export and size check; browser parity against `web/public/data/golden_v1.json`; worker handshake, timeout and recovery; precompute regeneration identical |
| 4 | colour map on a known profile; layer inventory shares sum to the total; WebGL fallback; playback frame rate measured |
| 5 | difference curve equals modified minus baseline; downloads round-trip; comparison and validation toggles independent |
| 6 | all cases of Section 5; full v1.0 suite unchanged |
| 7 | plotted measurements equal the CSV; metrics recomputed independently in a test |
| 8 | axe-core accessibility audit via Playwright; performance budget; reproducible build; link check |

## 4 Browser self-test

Adopt the NUS-H2 pattern (`frontend/selftest.js`, `scripts/selftest.py` in https://github.com/weewzr/NUSH2 at `b4817b3`): with `?selftest=1` the built site runs checks and writes a JSON verdict to a hidden element and to `document.body.dataset.selftest` (`pass` or `fail`); CI reads it, also with reduced motion. STRATO checks: build stamp equals commit, wasm loads, synthetic run matches golden values, every section renders, no console error, no failed request. The self-test compares values only and computes no physics.

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
