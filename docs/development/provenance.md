# Provenance of STRATO v1.0

STRATO v1.0 is the Phase 1 fission-product diffusion model. It was developed between June and October 2026 in two earlier repositories by the same author, then moved here as a clean project on 2026-11-06. This file records exactly what was taken, from where, and what changed in the move.

## 1. Source repositories

| No. | Repository | Commit used | Role |
|---|---|---|---|
| 1 | https://github.com/weewzr/Derivation-of-TRISO-Source-Term-Modelling | `5984da7855a4575851a4d057446c6a7a66fe2d7d` (2026-10-22) | Source of all v1.0 code, inputs, derivation and verification evidence (folders `deliverables/` and `verification/`) |
| 2 | https://github.com/weewzr/FHR-TRISO-Fuel-Particle-Diffusion-Model-Scale-1 | `675f460833b26e50875e7c15166c18d016d68450` (2026-08-14) | Earlier single-region explicit prototype. Reviewed; nothing taken |

Repository 2 holds a five-node explicit model of the kernel only, without a build file. It loses about 9 % of its inventory over a run, so it is not conservative. The verified finite-volume and analytical solvers in repository 1 supersede it.

## 2. File mapping

| No. | STRATO path | Source path (repository 1) | Change |
|---|---|---|---|
| 1 | `src/numerical/finite_volume.rs` | `deliverables/discrete_finite_volume_tridiagonal.rs` (Code B) | Two `use` paths changed from `super::` to `crate::` |
| 2 | `src/analytical/sturm_liouville.rs` | `deliverables/analytical_sturm_liouville.rs` (Code C) | None (byte-identical) |
| 3 | `src/config.rs` | `deliverables/main.rs`, inline `mod config` | Moved to its own file, indentation removed |
| 4 | `src/postprocessing/comparison.rs` | `deliverables/main.rs`, inline `mod compare` | Moved to its own file; `use` paths updated |
| 5 | `src/postprocessing/plots.rs` | `deliverables/main.rs`, inline `mod plots` | Moved to its own file; `use` path updated |
| 6 | `src/main.rs` | `deliverables/main.rs`, `main()` and helpers (Code A) | Uses the library crate; default input path is now `inputs/synthetic/synthetic_five_layer.toml` |
| 7 | `src/lib.rs`, `src/*/mod.rs` | (new) | Module roots only |
| 8 | `inputs/synthetic/synthetic_five_layer.toml` | `deliverables/synthetic_five_layer.toml` | None |
| 9 | `inputs/literature/literature_inputs.template.toml` | `deliverables/literature_inputs.template.toml` | None |
| 10 | `examples/verify.rs` | `verification/code_checks/examples/verify.rs` | `use` paths and input path updated |
| 11 | `tests/finite_volume_vs_analytical.rs` | `verification/code_checks/tests/finite_volume_vs_analytical.rs` | `use` paths and input paths updated |
| 12 | `tests/golden/v1.0/*.csv` | `verification/delivery_checks/numerical_results/*.csv` | Line endings normalised to LF |
| 13 | `tests/golden/v1.0/main_run_stdout.txt` | `verification/delivery_checks/TRISO_Five_Layer_Run.txt` | Renamed |
| 14 | `tests/golden/v1.0/verify_stdout.txt` | `verification/delivery_checks/TRISO_FV_Numerical_Verification.txt` | Renamed |
| 15 | `docs/mathematics/phase1/*` | `deliverables/TRISO_Finite_Volume_Derivation.{md,tex,pdf}`, `deliverables/TRISO_Finite_Volume_Pseudocode.md` | None (kept as delivered) |
| 16 | `tests/phase1_regression.rs` | (new) | Checks the build against the golden v1.0 outputs |
| 17 | `.github/workflows/ci.yml` | adapted from `.github/workflows/delivery-verification.yml` | Single workflow for the new layout; no bot commits |

The helper script that did the split checks that each path replacement happens exactly once. The numerical code was not edited.

Nothing else from either repository was carried over. Left out on purpose:
- abandoned research lines, legacy verification dumps and planning notes;
- documents written for other projects;
- the prototype in repository 2.

## 3. Evidence that the move preserved the v1.0 results

These checks were run locally on 2026-11-06 with rustc 1.96.0 on Windows 11. The original CI evidence was produced with rustc 1.99.1 on Ubuntu.

| No. | Check | Result |
|---|---|---|
| 1 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | Pass |
| 2 | `cargo test --release`: 14 unit, 3 cross-method and 3 regression tests | 20 passed, 0 failed |
| 3 | `cargo run --release --example verify` | `FV_VERIFICATION PASS`. Output identical to the golden file except the runtime line |
| 4 | `cargo run --release -- inputs/synthetic/synthetic_five_layer.toml`, standard output | Identical to the golden file (path separators normalised) |
| 5 | `timeseries.csv`, `final_profile.csv`, `convergence_study.csv` | Byte-identical to the golden files |
| 6 | `comparison_fv_vs_analytical.csv` | One value differs, at t = 6.8e4 s: 0.1265961866285994 here vs 0.12659618662859928 in the golden file (1.2e-16, platform `libm`) |

## 4. Verification status inherited from Phase 1

The following was checked independently during the move. All inputs are synthetic, so none of this is physical validation.

| No. | Claim | Status |
|---|---|---|
| 1 | Spatial order of the finite-volume solver is 2 | Verified (2.00 on four refinements) |
| 2 | Backward Euler is first order in time | Verified. The printed orders 0.98 to 0.89 are lower than the true value because spatial error enters the comparison. Against a same-mesh time-converged reference, the orders are 0.999 to 1.000 |
| 3 | Inventory is conserved to round-off | Verified (relative balance at most 2.8e-13) |
| 4 | The 32 eigenvalues and weights of the series are correct | Verified against an independent 50-digit Python solution with a different formulation (largest relative eigenvalue difference 3.7e-13) |
| 5 | F(2e5 s) = 0.43255563909304 for the synthetic case | Verified: Rust series, 50-digit Python series and spatially extrapolated finite volume agree within 1.3e-12 |
| 6 | The series stays accurate for diffusivity contrasts near 1e6 | Claimed only (the synthetic case reaches 2e4) |
| 7 | At very early times the series can return slightly negative F (-8.5e-14 at t = 2e3 s) | Observed. This is within the stated accuracy floor. v2.0 should flag or clamp it |
