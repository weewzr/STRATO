# Archive: the finite-volume solver of STRATO 1.x

STRATO 2.0.0 replaced the finite-volume production solver of versions 1.0.0 and 1.0.1 by the spherical BTCS finite-difference solver (`src/numerical/btcs.rs` and its modules; mathematics in `docs/mathematics/btcs/`). This folder keeps the finite-volume derivation documents so that the provenance of the 1.x results stays readable. Nothing here describes the current solver, and nothing here is rendered on the website.

## Where the removed code is

| What | Where |
|---|---|
| Release tags | `v1.0.0` and `v1.0.1` (the finite-volume solver as released; the tags are never moved) |
| Last commit holding the removed code on `main` | `36f1a98848bec9d66fe515994e075cff09d6cefb`, the tip of `main` when the BTCS integration branch was created. The first parent of the merge commit that brought version 2.0.0 into `main` is the last commit that holds it; `git show <merge>^1:src/numerical/finite_volume.rs` prints the module. |

Retired at version 2.0.0 (recover any of them with `git show 36f1a98:<path>`):

- Solver and interfaces: `src/numerical/finite_volume.rs` (Code B), `src/main.rs` (the finite-volume command line, with its `--fv-only` flag), `src/postprocessing/plots.rs` (figures that drew only the finite-volume comparison), the finite-volume parts of `src/postprocessing/comparison.rs` (comparison and refinement study), `examples/verify.rs` (the `FV_VERIFICATION PASS` program), the wrapper functions `validate_config`, `run_v1`, `analytical_release` and `compare_v1` of `crates/strato-wasm` (ABI 1 and 2, capability `diffusion_layered`, label `COMPUTED v1.0`) and `crates/strato-wasm/tests/parity.rs`.
- Golden files used as test references: `tests/golden/v1.0/` (six files). The series column of `comparison_fv_vs_analytical.csv` is the independent analytical reference and was copied unchanged to `tests/golden/analytical/series_synthetic_five_layer.csv`.
- Python scripts that only served the finite-volume derivation, and every SymPy proof script: all of `scripts/derivation_checks/` except `sec1_synthetic_case.py` (the one Python reference check that stays; it checks the analytical series) and `D5_parameter_register.py` (kept until the Rust data checks of the second integration pull request replace it), and `scripts/data_register/b6_audit_patch.py` (its changes are in the register CSV files; checked before removal).

## Documents archived here

| Folder | Content |
|---|---|
| `v1.0.0/` | Full Mathematical Derivation and Final Deliverable of version 1.0 (analytical solution and finite-volume scheme), with their PDFs |
| `v1.0.0/figures/` | The four figures of those documents, each beside the matplotlib script that draws it (`v1_0_0_*.py`) |
| `phase1/` | The Phase 1 derivation and pseudocode (October 2026), kept as delivered |
| `phase2/` | The Phase 2 derivations and pseudocode with their PDFs: generation, decay and temperature-dependent diffusivity derived for the finite-volume form and never implemented. The BTCS documents supersede them, mechanism by mechanism, as those mechanisms are added |
| `development/` | `provenance.md` (where the v1.0 files came from and the verification evidence of that release), `v1_to_v2_changes.md` (the change plan written before the BTCS solver existed) and `phase2_review_package.md` (the reading guide of the Phase 2 documents) |

The research catalogue (`data/research/mechanisms.csv`), the parameter registers and the README link the Phase 2 documents and the provenance document by their old paths. A short note stays at each old Markdown path (`docs/mathematics/STRATO_Phase2_Derivations.md`, `docs/mathematics/STRATO_Phase2_Pseudocode.md`, `docs/development/provenance.md`, `docs/development/v1_to_v2_changes.md`, `docs/development/phase2_review_package.md`) and points here, so those links stay valid and no data file had to change. The PDFs moved without a note. None of these documents is rendered on the website.

## What happened to each removed Python script

Decision of the project owner: Rust does all scientific computation and numerical testing; SymPy scripts and obsolete Python implementations are removed. Every script below is recoverable with `git show 36f1a98:scripts/derivation_checks/<name>.py`. The class is one of: SymPy (imports `sympy`); finite-volume only (re-implements or uses the retired scheme, or checks a document written for it); replaced (a Rust check covers it); not replaced (neither of the first two, and no Rust check covers it fully, said plainly).

| Script | Class and reason |
|---|---|
| `sec1_symbolic_checks.py`, `sec3_o4_layered_truncation_bound.py`, `sec3_p21_decay_source.py`, `sec3_p21_modal_ode.py`, `sec4_p21_decay_identity.py`, `sec4_p21_steady_state.py`, `sec4_p2_positive_second_order.py`, `sec5_arrhenius.py`, `sec5_effective_time.py`, `sec5_r24_quadrature_start.py`, `v1_0_0_inventory_codeC.py`, `v1_0_0_p1_geometry.py`, `v1_0_0_p2_conservation.py`, `v1_0_0_p4_separation.py`, `v1_0_0_p6_series.py`, `sec2_fv_geometry_conductance.py`, `sec4_p21_matrix_schemes.py`, `sec4_p21_mms_source.py`, `sec5_o10_stiff_staircase.py`, `sec5_o11_parent_daughter_tau.py`, `sec5_staircase.py`, `sec6_pseudocode_checks.py`, `v1_0_0_inventory_code_audit.py`, `v1_0_0_p3_conditions.py`, `v1_0_0_p5_fv_scheme.py`, `v1_0_0_p6_orders.py`, `v1_0_0_p6_properties.py`, `v1_0_0_p6_truncation.py`, `sec5_o12_variable_D_step_limit.py`, `v1_0_0_inventory_codeB.py` | SymPy (30 scripts) |
| `b4_decay_source_pseudocode.py` | SymPy; the checks that read the register files are replaced by the Rust checks `B4D.*` of `crates/strato-data` (`CHECKS.md`) |
| `b4_phase2_pseudocode_integration.py` | Finite-volume only (a step of the Phase 2 design in the finite-volume form); the checks that read the register files are replaced by the Rust checks `B4P.*` of `crates/strato-data` |
| `sec2_fv_tridiagonal_conservation.py`, `sec2_fv_truncation_order.py`, `sec3_p21_chain_overlap.py`, `sec3_p21_common.py`, `sec3_p21_fv_two_layer.py`, `sec4_p21_be_closure_order.py`, `sec4_p21_booth_crp6.py`, `sec5_fv_shared_alpha.py`, `sec5_projection_step.py`, `v1_0_0_p5_fv_replica.py` | Finite-volume only: each re-implements the retired scheme or uses it as the reference. The owner cancelled their ports to Rust |
| `sec3_p21_booth.py` | Not SymPy and not finite-volume: it checks the Phase 2.1 modal formulas with decay and source against the Booth single-sphere solutions. Not replaced one for one. Those formulas belong to the archived Phase 2 documents and the BTCS solver does not implement decay or generation yet; the Booth series is covered in part by `D5.B.booth-zero` and the other Booth checks of `crates/strato-data` and by the closed homogeneous-sphere series with its tests. A port belongs to the work that adds decay and generation |
| `sec5_schedules.py` | Not SymPy and not finite-volume: it checks the AGR-1 and CRP-6 schedule totals. Replaced for the totals by the Rust checks `D5.T.total.*` of `crates/strato-data`; its effective-time illustration (an input-pipeline demonstration of the archived Phase 2 text) is not replaced |
| `v1_0_0_p4_sturm_weights.py` | Not SymPy and not finite-volume: an independent mpmath re-implementation of the Sturm-Liouville eigenproblem of the analytical solver. Covered in part: the solver is kept with its unit tests, `tests/analytical_reference.rs` and `tests/analytical_series_regression.rs`, and the kept Python check `sec1_synthetic_case.py` verifies the same series independently. The independent high-precision eigenvalue search itself has no Rust counterpart and is a remaining opportunity |
| `b6_audit_patch.py` | One-off patch of the register CSV files; its changes were confirmed to be present in the files (running it on the final data leaves them unchanged), then removed |

Python that stays after version 2.0.0, and why: `scripts/derivation_checks/sec1_synthetic_case.py` (the one reference check of the analytical series; numpy and mpmath, no SymPy, no finite-volume code); `scripts/derivation_checks/D5_parameter_register.py` (still runs, and imports SymPy, which is why `sympy` stays in `requirements.lock.txt`; the second integration pull request replaces it with the Rust data checks and removes it together with that dependency); `scripts/data_register/b6_nuclear_crosscheck.py` (a register check whose network part has no Rust port; the offline checks are in `crates/strato-data`) and `scripts/data_register/render_register.py` (generator of the register document); the last two are retained until a Rust port is scheduled and are not part of any workflow. The three figure scripts are kept in the archive beside the figures.

The results of these documents are numerical verification of the stated models on a synthetic case. They are not experimental validation.
