# Changelog

All notable changes to STRATO are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).


## [Unreleased]

The website is rebuilt as four pages (Explore, Basis, Research Catalogue, Scientific Documentation). Explore has six Model Equation and five Operation buttons selected independently, the governing equation with its variable legend, the Numerical Interpretation and interactive graphs with a temperature selector in 100 degC steps; a selection without a results file shows an empty graph panel. Status badges, the live-simulation panel, the numerical controls and the build hash left Explore, Basis and the Research Catalogue; the manual solver with its 3D view (loaded only when opened), the statuses, the parameter table and the downloads are in Scientific Documentation, which reads the same results files as the graphs. Method A is drawn in red and Method B in blue. The plan written before the BTCS solver existed is archived in [docs/archive/finite-volume/development/v1_to_v2_changes.md](docs/archive/finite-volume/development/v1_to_v2_changes.md).

## [2.1.0] - 2026-11-27

The BTCS solver gains generation, decay and temperature-dependent diffusion, the analytical series gains its extensions, and the registers, the comparison program and the shared results that go with them are merged. The published website is not rewired in this release: its live simulation still runs the baseline scope of 2.0.0, and the new interface is called by the following website change. All results remain numerical verification of the stated model; none is experimental validation, and no registered dataset supports a whole-curve R² today.

### Added
- The BTCS solver with fission-product generation (a source in the kernel), radioactive decay and temperature-dependent diffusion (the diffusivities are evaluated at the temperature in the middle of each step and the matrix is reassembled), the combined model of Document 5 (`src/numerical/physics_config.rs`, `physics_run.rs`), its command-line options and exports (`src/postprocessing/physics_export.rs`), and the scenario engine for the five operating scenarios (`src/numerical/scenario.rs`, inputs in `data/scenarios/`). The balance residual is reported beside its derived rounding bound; tests in `tests/btcs_physics.rs`, `tests/btcs_physics_cli.rs` and `tests/scenario_engine.rs`.
- The Sturm-Liouville series with its extensions for decay and for a piecewise-constant kernel source at a constant temperature (Document 6, formulas B2 to B5; `src/analytical/applicability.rs`, `extensions.rs`, `extension_floor.rs`): the applicability table, the drawing mask, the accuracy floor re-estimated over several geometries and mode counts, and a declared geometry regime outside which every extension value is withheld. The constant `EXTENSION_FLOOR_REESTIMATED` is `true`; the independent verification of the rule passed. Method B is drawn at stage 1 and, through the decay extension, at stage 3 for Ag-110m at 1700 degC and Cs-134 at 1800 degC; elsewhere it is refused or withheld with a reason code.
- Experimental comparison of the BTCS solver with the AGR-1 release register: the tool crate `crates/strato-compare` (status rules, BTCS runs at the test temperature, metrics, CSV and JSON files with provenance headers in `data/comparison/`, static SVG figures A to K, and the report `docs/validation/experimental_comparison.md`). It is a comparison of release behaviour, not an experimental validation. The ten matrix-corrected rows of intact compacts (E-010..E-014, E-033..E-037) are read as window upper bounds under premise BP1 over the window from 24 h at temperature to the end of the 300 h plateau (or the shorter test end); no row is a point comparison. The data check `DI.release.window-label` requires their window label and applicability text.
- Release-versus-time curves in the register: rows E-541 to E-658 of `data/experimental/release_measurements.csv` (Cs-134 and Cs-137 of the 4-4-2 loose particles, Cs-134 and Sr-90 of compact 4-3-3, Ag-110m, Cs-137 and Sr-90 of the HFR-K3/1 sphere), the history AGR1-FACS-4-3-3, the new file `data/experimental/curve_assessment.csv` (15 curves, all plot only), tables 25b, 31i to 31k and 31m of the register, sections 12 to 13 of the audit log, and the check families EX and DI of `strato-data` (176 checks). Digitised values are estimates with a stated reading uncertainty, never exact measurements. The register column `fitted_to` names the dataset a diffusivity row was fitted to (A3-14 and E410-01, IAEA (1997) Table 4-9); the independence rule of the R2 records reads it.
- The shared results program `crates/strato-results` and its output `data/results/` (138 files with provenance headers; `strato-results run`, `strato-results run --check`): curves for the stages and the scenarios, the bound comparisons, the verification files (convergence, conservation, and the agreement of Method A and Method B for the stage-1 curves and the stage-3 curves where Method B is drawn), and the R2 records of the fifteen curves (`r2.json`, `csv/r2.csv`, `comparison/<curve_id>.json`, the caption records in `labels.json`: the metric of `metric-r2.md` with its seventeen ordered checks, sixteen reason codes with fixed captions and the list of conditions that made the missing-conditions check fail). With the registered data every record is unavailable (no value, no label), each says which check failed first, and Method A is drawn over the registered history where one is available. CI runs `strato-results run --check` so that a stale results file fails the build.
- WebAssembly interface of ABI 4 (schema 3): `run_stage` (equation stages 1 to 5 with the source, decay and a temperature history), `run_scenario` (the operating scenarios through the scenario engine, with the normal-operation reference), `btcs_system` (the assembled BTCS rows and right-hand side of one step for a small grid), `analytical` (Method B with its applicability, reason and drawing mask) and `options` (stages, scenarios, nuclides and temperatures from the embedded definitions). The six functions of ABI 3 return the same numbers as before; only the sentence that states the balance in the reply of `run` changed (see Changed). The registers and scenario files are embedded in the module, which grows from 0.78 MB to 4.8 MB (about 0.67 MB after gzip). The `units` map of every reply names every quantity of the reply.
- The library crate `crates/strato-scenarios` (scenario definitions, register-based diffusivity laws, registered geometry and the one Method B function), shared by the wrapper, the results program and the comparison program.
- The parity rules of decision 76(a) in `strato_wasm::parity`: the per-value rule (relative 1e-12) for every physical value, a balance residual that passes when it lies within its own derived bound on both platforms, and a drawn-or-withheld flag at the margin of the mask that may differ only where the two estimates agree and lie on either side of the threshold, and a reason-swap rule: the reason code of a value that both platforms withhold may differ between `COVER_SENSITIVITY` and `ERROR_ESTIMATE_UNDEFINED` only (a code against a drawn value, any other pair and every physical value stay outside it); `examples/btcs_parity.rs` reports how many fields each rule covers per case.
- The command `strato-data register (--check | --write)`: the generated tables of `data/reference/STRATO_Parameter_Register.md` (26 tables between marker lines) are produced and checked in Rust; the hand-written text is never changed. CI runs the check.
- Documents 5 (generation, decay and temperature-dependent diffusion) and 6 (analytical extensions) of `docs/mathematics/btcs/`.

### Changed
- Website: the Explore page is rebuilt as a targeted refinement. A progressive governing-equation builder, a row of operating scenarios, an expandable fourteen-item Numerical Interpretation, the live simulation and the results read one shared selection that is kept in the address. Status and control availability come from the capability the WebAssembly wrapper reports; a selection the build does not compute shows no result and says so, and a reply is shown only under the selection it was computed for. The simulation block names the active solver and the physics it implements. The experimental-comparison graph component is merged. New tests walk the learning sequence at three widths, both colour schemes and both motion settings, and with the keyboard only. The site-wide minimal-text clean-up (Compare, Validate, Verify, Research, Data and About) and the Scientific Documentation page are merged with it, and Explore follows the same rules: no introductions, one-sentence captions, no build hash or file path on the page, the details in Scientific Documentation; the content check has no exception left. The roadmap and gallery pages of Research are shorter, and the citation list is kept once. The tablet header is two compact rows, the unsupported-selection condition is stated once, and the touch targets at phone width are 44 px. (This website change itself touches no Rust source.)
- The crate and the wrapper crate `strato-wasm` are version 2.1.0; the wrapper's ABI is 4.
- The R2 independence rule reads the register field `fitted_to`, not a phrase of the notes of a row.
- The balance statements of the replies of `run` and `run_stage` start with a capital letter and no longer mention the directive.
- CI: the format step covers the wrapper crate; a job checks that every file of `data/results/` is current; the register's generated tables are checked.

### Removed
- `scripts/data_register/render_register.py` and `register_template.md` (replaced by `strato-data register`), and the SymPy script `scripts/derivation_checks/D5_parameter_register.py`: every one of its 91 checks is a check of `strato-data` with its own Rust test (`crates/strato-data/CHECKS.md`). `sympy` leaves `requirements.lock.txt`. The remaining Python files are `sec1_synthetic_case.py` (the independent reference check of the analytical series, kept by decision), `b6_nuclear_crosscheck.py` (a one-off audit that needs network access) and the figure scripts of the archive.
## [2.0.0] - 2026-11-26

The finite-volume solver of version 1.x is replaced by a spherical BTCS (backward-time centred-space) finite-difference solver, entirely in Rust. Results are labelled `COMPUTED v2.0`. This is a breaking change of the library, the command line, the WebAssembly interface and the data files; the finite-volume code stays in Git history (tags `v1.0.0` and `v1.0.1`) and is described in `docs/archive/finite-volume/README.md`. All results are numerical verification of the stated model; none is experimental validation.

### Added
- The BTCS solver (`src/numerical/btcs.rs` and its modules `grid`, `interfaces`, `boundary_conditions`, `coefficients`, `tridiagonal`, `inventory`, `diagnostics`, `btcs_report`) and its command line `strato-btcs`, with unit, property and verification tests and regression references in `tests/golden/btcs/`. The mathematics is in `docs/mathematics/btcs/` (Documents 0 to 4); Document 2 is normative and Document 4 traces every equation to its pseudocode, Rust item and test.
- The closed series of the homogeneous sphere (`src/analytical/homogeneous_sphere.rs`), the validation metrics (`src/validation/`) and the regression and export modules, with `examples/btcs_regression.rs`, which ends with `BTCS_REGRESSION_CHECK PASS` and fails otherwise.
- Method-neutral WebAssembly interface `validate`, `run`, `compare` and `export_run` (capability `diffusion_btcs_layered`, ABI 3), and a web demonstration that offers exactly two methods, Method A (BTCS) and Method B (Sturm-Liouville).
- Property functions and closed-form references for the Phase G mechanisms in `src/physics/` (Arrhenius diffusivity, decay constant, volumetric fission source, well-mixed and homogeneous-sphere references), derived in `docs/mathematics/STRATO_Property_Functions.md`. They are not coupled to any solver, and the document is not yet published on the site.
- `crates/strato-data`: the register files of `data/` read into typed Rust values, with every consistency check of the Python audit scripts reproduced in Rust (`crates/strato-data/CHECKS.md`).
- `docs/archive/finite-volume/`: the finite-volume derivation documents and figures of version 1.x, with a README naming the release tags and the last commit that holds the removed code.
- `tests/golden/analytical/series_synthetic_five_layer.csv`: the Sturm-Liouville series values of the synthetic case, copied unchanged from the version 1.0.1 comparison file, as the reference of `tests/analytical_series_regression.rs`.
- A CI job that runs the Python reference check of the analytical series (`scripts/derivation_checks/sec1_synthetic_case.py`, 24 checks).
- `prompt/`: the instruction package for building the interactive STRATO web demo, including cross-project engineering references from NUS-H2.
- `crates/strato-docsite`, the first member of the tools workspace: renders the derivation and register documents into the published site with the mathematics typeset by the KaTeX copy the site ships, and checks every link of the built site.

### Changed
- The crate version is 2.0.0; the wrapper crate `strato-wasm` is 2.0.0 and its ABI is 3.
- The web tooling is Rust (stage R1 of the Rust migration): `crates/strato-webtools`, a member of the tools workspace `crates/Cargo.toml`, is one command-line tool with the subcommands `case`, `catalogue`, `colormaps`, `geometry`, `precomputed`, `check-catalogue` and `build-wasm`. It replaces `scripts/web/generate_data.py`, `check_catalogue.py`, `build_wasm.py`, `build_wasm.sh`, `precompute.sh`, their Python tests and `web/scripts/check_catalogue.mjs`; Git history keeps them. The precomputed web data are regenerated from the BTCS command line and the native wrapper; `web.yml` and the documentation call the Rust tool.
- The web demonstration solves with the BTCS solver (before: the finite-volume solver). The equation builder shows the BTCS equations; the stages for generation, decay and temperature dependence stay labelled previews.
- The document site lists the BTCS Documents 0 to 4 and no longer renders the finite-volume documents or the golden-file page. `README.md`, `CONTRIBUTING.md` and `CITATION.cff` describe version 2.0.0.
- `ci.yml` gates on `BTCS_REGRESSION_CHECK PASS` in place of `FV_VERIFICATION PASS` and runs the BTCS command line on the synthetic case.
- Licence changed from MIT to AGPL-3.0-only, the licence of the Outram Park project that the web demo builds on. Releases v1.0.0 and v1.0.1 remain available under MIT.
- GitHub Pages is built and deployed by `.github/workflows/web.yml` instead of the Jekyll render of the repository. `prompt/`, the README, this changelog and `docs/development/` are not published on the site.

### Removed
- The finite-volume solver (`src/numerical/finite_volume.rs`), its command line `strato` and the flag `--fv-only`, the plotting module `src/postprocessing/plots.rs`, `examples/verify.rs` and the dependency `plotters`.
- The WebAssembly functions `validate_config`, `run_v1`, `analytical_release` and `compare_v1`, the capability `diffusion_layered` and the label `COMPUTED v1.0`.
- The golden files `tests/golden/v1.0/` (test references of the finite-volume solver) and the web data files that were generated from them.
- Every Python derivation check except `sec1_synthetic_case.py` (the analytical reference) and `D5_parameter_register.py` (kept until the Rust data checks replace it), including all SymPy proof scripts, and `scripts/data_register/b6_audit_patch.py` (its changes are in the register files). The finite-volume derivation documents and the figure scripts moved to `docs/archive/finite-volume/`.
- The Phase 2 design documents (derivations, pseudocode and their PDFs), `docs/development/provenance.md`, `v1_to_v2_changes.md` and `phase2_review_package.md` moved to `docs/archive/finite-volume/` and are no longer rendered on the site. A short note stays at each old Markdown path so that the links of the research catalogue, the registers and the README keep working.
- Public addresses of the former Jekyll site stop working after the deployment of this version: `/STRATO/docs/mathematics/v1.0.0/*.html`, `/STRATO/docs/mathematics/STRATO_Phase2_*.html` and `/STRATO/tests/golden/v1.0/` return 404. The documents are in the archive folder of the repository.

## [1.0.1] - 2026-11-06

Documentation release. There is no change to the code or to any numerical result: the Rust sources, tests and golden outputs are identical to v1.0.0.

### Added
- `docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md` and its PDF: the detailed learning and derivation document for the v1.0.0 model, from the physical foundation to the analytical solution (Code C), the finite-volume scheme (Code B), the numerical properties and the link to the implementation.
- `docs/mathematics/v1.0.0/STRATO_Final_Deliverable.md` and its PDF: the concise implementation reference that puts the final equation, the pseudocode and the Rust code (with file and line references) side by side.
- Four figures in `docs/mathematics/v1.0.0/figures/` and the scripts that draw them in `scripts/figures/`.
- Derivation check scripts `scripts/derivation_checks/v1_0_0_*.py`, which verify the equations, the code inventory and the numerical properties quoted in the two documents.
- A documentation PDF workflow (`.github/workflows/docs-pdf.yml`) and the pandoc filters and settings in `docs/pandoc/` that build the PDFs with XeLaTeX and render every page for inspection.

### Changed
- `README.md` links both documents and says what each is for. The Phase 1 documents stay linked as the historical record, and `docs/mathematics/phase1/README.md` points to the new documents.

## [1.0.0] - 2026-11-06

Phase 1 baseline. It was developed from June to October 2026 in earlier repositories and established here as a clean project. See `docs/development/provenance.md` for the source commit and file mapping.

### Added
- Conservative spherical finite-volume solver (Code B): exact shell volumes, series-resistance interface conductances, Backward Euler, and subtraction-free tridiagonal elimination.
- Multilayer Sturm-Liouville eigenfunction series (Code C): Sturm-count eigenvalue isolation, release-fraction series, and a reported accuracy floor and validity time.
- Command-line program (Code A): TOML input with placeholder checks, CSV and SVG outputs, a finite-volume vs series comparison, and a spatial and temporal refinement study.
- Synthetic five-layer verification input and a literature input template.
- Verification example (`examples/verify.rs`) that asserts every claim it prints.
- Cross-method integration tests and a Phase 1 regression test against the delivered v1.0 outputs (`tests/golden/v1.0/`).
- Phase 1 derivation and pseudocode as delivered (`docs/mathematics/phase1/`).
- Continuous integration: format, clippy, tests, documentation, release build, verification run, dependency audit.

### Known limitations
- Constant temperature and constant diffusivity in each layer. No decay and no generation.
- Perfect interface contact, perfect-sink outer surface, kernel-only initial inventory.
- The bundled inputs are synthetic, so the public release contains numerical verification only. An internal experimental comparison conducted at SNRSI is not yet public.

[Unreleased]: https://github.com/weewzr/STRATO/compare/v1.0.1...HEAD
[1.0.1]: https://github.com/weewzr/STRATO/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/weewzr/STRATO/releases/tag/v1.0.0
