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

The Phase 2 design documents (`docs/mathematics/STRATO_Phase2_Derivations.md` and `STRATO_Phase2_Pseudocode.md`) stay where they are: the research catalogue and the data registers link to them. They are written for the finite-volume form of generation, decay and temperature dependence and are superseded, mechanism by mechanism, by the BTCS documents as those are added.

The results of these documents are numerical verification of the stated models on a synthetic case. They are not experimental validation.
