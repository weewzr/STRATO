# Cross-project engineering references: raystorage and NUS-H2

The owner's brief for this file is kept verbatim in `prompt/RAYSTORAGE_REQUEST.md`.

## 1 Why raystorage is relevant

https://github.com/weewzr/raystorage holds the owner's NUS-H2 working files under one folder, `NUSH2/`: derivations, simulations, process data, historical frontend and PFD versions, research and references, validation scripts and logs, and a SHA-256 manifest. https://github.com/weewzr/NUSH2 is the current NUS-H2 product: a hydrogen-plant model with a Rust engine, a Rust/WebAssembly browser simulator, a browser self-test, golden-value tests, an equation register and a validation matrix.

**raystorage contains no STRATO files.** It has no TRISO diffusion code or derivation. It matters to STRATO only because NUS-H2 already solved engineering problems that the STRATO web demo will meet: running a Rust engine in a browser worker, proving browser and native numbers agree, testing a static site automatically, and tracing every number to a source. STRATO adopts those **methods**. It adopts none of the NUS-H2 science.

## 2 Pinned commits and licences

| Repository | Commit | Licence |
|---|---|---|
| weewzr/NUSH2 | `b4817b305de5bfada5da147802cbf3aeffb9f4c1` | MIT (GitHub detection) |
| weewzr/raystorage | `3a25c4a767b67faa3e797944ea9ac63f9265d4a1` | none declared; owner's archive, reference only |

Read files at these commits: `gh api repos/<repo>/contents/<path>?ref=<sha>`. Record any newer commit you use in `prompt/PROGRESS.md`. Public visibility does not authorise redistribution: copy no archive file into STRATO.

## 3 Scientific independence (mandatory)

STRATO and NUS-H2 are separate projects. Do not:

- merge their scientific engines or link one to the other;
- reuse NUS-H2 thermodynamic, property or process models as TRISO diffusion models;
- import NUS-H2 physical assumptions;
- transfer numerical constants without STRATO's own physical justification;
- copy process-flow calculations into diffusion equations;
- claim NUS-H2 verification or validation supports STRATO;
- replace STRATO's analytical reference solution (Code C, `src/analytical/sturm_liouville.rs`) with unrelated benchmarks.

Every new STRATO calculation is justified by STRATO's own derivations (`docs/mathematics/`) and evidence (`data/`).

## 4 Archive scope and files to examine

raystorage blob counts by category at the pinned commit: `NUSH2/validation/` about 1,590 files, `NUSH2/simulations/` about 163, `NUSH2/PFD-versions/` about 72, plus derivations, process-data and research-and-references. Each category is split by origin session folder. `NUSH2/MANIFEST.csv` lists every file with size and SHA-256. `NUSH2/README.md` states that caches, virtual environments, images, downloaded papers, OUTRAM code and files matching a privacy scan were excluded.

The archive is history, much of it superseded. The current NUS-H2 repository is the authority for each pattern; use the archive only to see how a practice was checked over time.

| File (repo: path) | Purpose | Status as found | Use for STRATO |
|---|---|---|---|
| NUSH2: `engine/wasm/README.md` | wasm ABI: zero-import `cdylib`, JSON request and reply, command table (`registry`, `solve`, `sweep`), parameter records with hard and model ranges, `status` values `ok`, `not_converged`, `rejected`, measured load and solve times with date and conditions | current | pattern for request and response design, validation and honest timing notes |
| NUSH2: `engine/wasm/src/lib.rs`, `solve.rs`, `registry.rs`, `tests.rs` | thin entry points, dispatch, parameter registry with `class`, `source` and rigour status words, native tests of the wrapper | current | pattern; STRATO's wrapper stays thin and physics-free |
| NUSH2: `scripts/build_wasm.py`, `scripts/check_wasm.py` | build; check required exports, zero imports, size under 3 MB | current | adopt an export and size check |
| NUSH2: `scripts/wasm_smoke.py` | headless browser timing of the first solve | current | pattern for STRATO's performance check |
| NUSH2: `frontend/worker.js`, `frontend/selftest.js`, `scripts/selftest.py` | worker host; `?selftest=1` self-test writing a JSON verdict and `document.body.dataset.selftest`; read over DevTools; checks the commit stamp; compares values only | current | adopt the self-test design |
| NUSH2: `.github/workflows/site.yml` | builds and checks wasm, builds the site, runs the self-test at normal and reduced motion against the expected commit, deploys to Pages | current | model for STRATO's `.github/workflows/web.yml` |
| NUSH2: `engine/vv/src/golden.rs`, `engine/vv/data/engine_goldens.json`, `engine/vv/tests/plant_goldens.rs` | frozen golden values with one named tolerance (`GOLD_TOL = 1e-9`); a mismatch means a port error, so the code is fixed and the goldens stay | current | adopt the rule and the named constant; STRATO's own tolerance stays 1e-12 |
| NUSH2: `docs/VALIDATION_MATRIX.md`, `results/Validation_Results.csv` | benchmark rules: tolerances frozen when added; failed rows stay failed; `pass`/`fail` only against a limit, `reported` without one, `not_run` with a reason; code tests separate from literature benchmarks; data use `independent` or `calibration` | current | adopt the status vocabulary and data-use column for STRATO's Phase 7 |
| NUSH2: `math/eq_register.csv` | equation register: id, section, tier, title, latex, variables, assumptions, source, tag, engine_fn, test_id, validation | current | adopt the idea for STRATO (Section 7) |
| NUSH2: `math/NUSH2_First_Principles_Derivations.tex`, `math/sections/*.tex`, `math/generated/values.tex`, `math/sections/M_missing_sources.tex` | LaTeX derivation with generated value macros and an explicit missing-sources section | current | ideas only; STRATO keeps Markdown with pandoc and XeLaTeX |
| raystorage: `NUSH2/README.md`, `NUSH2/MANIFEST.csv` | archive description and checksums | archive | read for provenance; a SHA-256 manifest is a practice to copy for `web/public/data/` |
| raystorage: `NUSH2/validation/session-*/stagef/*/selftest_375.log`, `selftest_1440.log`, `wasm_smoke.log`, `build_wasm.log` | historical self-test and smoke logs at 375 px and 1440 px | archive, superseded by the current repo | evidence of how the self-test was run; do not execute anything |
| raystorage: `NUSH2/validation/session-*/deriv2/b2/scripts/check_wasm.py` | older copy of the wasm check | superseded | use the current NUSH2 copy instead |
| raystorage: `NUSH2/PFD-versions/` | historical frontend and diagram versions | archive | layout history only; not a STRATO design source |
| raystorage: `NUSH2/simulations/` | simulation exports and scripts | archive | look only for test configuration and reporting formats |

Phase 0 checks every path above at its pinned commit and corrects this table if needed.

## 5 Comparison: NUS-H2 versus STRATO

| Area | Existing STRATO capability | Relevant NUS-H2 implementation | Transferable lesson | Recommended STRATO improvement | Priority | Risks or limitations |
|---|---|---|---|---|---|---|
| Scientific engine | Rust crate `strato`: finite volume (Code B) and Sturm-Liouville series (Code C) | Rust plant engine (several crates) | keep the engine pure and callable without I/O | wrap `strato` in a separate physics-free crate `crates/strato-wasm` | P1 | wrapper must reproduce `finite_volume::run` exactly |
| Numerical methods | conservative spherical FV, Backward Euler, eigenfunction reference | steady process balances and solvers | none for the methods themselves | none; methods stay as derived | n/a | importing anything here would break independence |
| Verification | golden regression at 1e-12, Code B against Code C, convergence study, balance residual, `verify` example | named golden tolerance, goldens never edited, separate code tests and benchmarks | one named constant per comparison; fix code, never goldens; assert every golden file | name tolerances in one file; add assertions for `convergence_study.csv` and the stdout goldens; add native/wasm parity | P1 | tightening below 1e-12 would break on libm differences |
| Data provenance | `data/*.csv` with `status` (verified, digitised, NOT VERIFIED) and the register | parameter `class`, `source`, rigour status words; validation data use `independent` or `calibration`; missing-sources section | separate a value's origin from its trust level; record how a number was used | add a display category to each shown value (Section 6); add `data_use` to Phase 7 validation rows | P2 | do not duplicate the register; extend only what the web needs |
| WebAssembly architecture | none yet; library compiles for `wasm32-unknown-unknown` | zero-import `cdylib`, JSON in and out, commands, `rejected` with violations, size check | small JSON API, explicit status, no panics across the boundary | `run_v1`, `analytical_release`, `validate_config` with `status`, size and export check | P1 | raw ABI versus `wasm-bindgen`: STRATO uses `wasm-bindgen` for a smaller API |
| Frontend architecture | none yet | plain JavaScript modules with a worker | worker isolation and commit-stamped assets | React + Vite (chosen for the equation explorer and 3D), worker per `prompt/TECHNICAL_ARCHITECTURE.md` | P1 | do not copy the NUS-H2 frontend structure |
| Browser testing | none yet | `?selftest=1` self-test; CI runs it at normal and reduced motion and checks the commit; logs at 375 px and 1440 px | build verdict into the page; CI reads it | STRATO self-test plus Playwright for viewports, keyboard, reduced motion, console errors, failed requests, parity | P1 | self-test must compare values only, never compute physics |
| Scientific documentation | Markdown derivations built by pandoc and XeLaTeX; v1.0.0 Final Deliverable maps equations to Rust lines | `eq_register.csv` linking equation, source, engine function and test; generated value macros | one machine-readable row per equation | `docs/mathematics/equation_register.csv` that the equation explorer reads | P3 | must stay in sync with the documents; generate or check it in CI |
| Deployment | CI and docs-PDF workflows; no site | Pages workflow with wasm check and self-test before upload | gate deployment on checks | `.github/workflows/web.yml` with the same gates; Pages enabled by the owner | P2 | Pages only after the owner merges into `main` |
| Maintenance | pinned lock file, CHANGELOG, provenance doc | measured timings with date and machine, frozen tolerances with reasons, manifest | write down when and how a number was measured | timings and verification report with date, commit and command | P2 | measurements on one machine do not generalise |

No change above is recommended for consistency alone; each fills a gap STRATO has in its planned web demo.

## 6 Provenance categories on the STRATO website

Map each displayed quantity to one category. These categories sit on top of STRATO's existing register status (verified, digitised, NOT VERIFIED); they do not replace it.

| Category | STRATO source | Result label |
|---|---|---|
| Synthetic verification case | `inputs/synthetic/synthetic_five_layer.toml`, synthetic benchmark rows | `COMPUTED v1.0` or `PRECOMPUTED`, with "synthetic, not physical data" |
| Analytical reference solution | Code C series | `COMPUTED v1.0` (analytical) |
| Literature correlation | `data/diffusivity/arrhenius.csv` and the register | value with source, page and status |
| Experimental measurement | `data/experimental/release_measurements.csv` | `MEASURED`, with uncertainty and `data_use` (`independent` or `calibration`) |
| Model prediction | Phase 2 runs after web Phase 6 | `COMPUTED v2.0` |
| Unverified assumption | rows with status NOT VERIFIED, model assumptions A1 to A6, assumed start temperatures | visible warning with the row id |
| Superseded result | older precomputed files | not shown; kept out of `web/public/data/` |

Validation rows follow the NUS-H2 status words: `pass` or `fail` only against a stated limit, `reported` when no limit exists, `not_run` with a reason. A failed row stays failed.

## 7 Documentation practices

- **Equation register (P3)**: `docs/mathematics/equation_register.csv` with columns `id, title, latex, variables, assumptions, source_doc, rust_fn, test_id, status`. Seed it from the v1.0.0 Final Deliverable (which already maps equations to Rust lines) and the Phase 2 derivations (status DERIVED NOT IMPLEMENTED). The equation explorer reads term LaTeX and links from it. A CI check confirms every `rust_fn` exists and every `test_id` names a real test.
- **Equation-to-Rust links**: keep the pinned links of the Final Deliverable; add links from the register.
- **Automated builds**: keep `.github/workflows/docs-pdf.yml`; add the register check. Do not move to LaTeX or Tectonic.
- **Verification-status labels**: IMPLEMENTED, DERIVED NOT IMPLEMENTED, FUTURE, as already used.
- **Reproducible figures**: figures for the site come from `scripts/web/precompute.sh` with commit, input hash and command recorded.

Do not replace or simplify the existing derivations.

## 8 Effect on the roadmap

- **Phase 0** includes a time-bounded cross-project review (at most about one working session). Priority order: Rust/WebAssembly patterns, numerical verification, browser testing, scientific provenance, documentation workflows. Output: corrections to this file and a short note in `prompt/PROGRESS.md`. Phase 1 starts when the review ends, whatever is left open; leftovers become notes, not blockers.
- **Phase 1**: browser self-test skeleton and Playwright checks from the start.
- **Phase 3**: wrapper design, status values, export and size check, parity tests, commit stamping.
- **Phase 5**: verification report rendering.
- **Phase 7**: validation status vocabulary and `data_use`.
- **Phase 8**: equation register (if time allows), Pages gating.

## 9 Archive security and provenance

Before using any archive file: verify the path, record the source commit, read the contents, check provenance, decide whether it is superseded, confirm no sensitive material, and check licence obligations. Do not execute archived scripts without review (prefer not at all). Do not copy browser profiles, credentials, session data or temporary files. Do not quote people's names, course codes or group text from the archive. Copy no archive file into STRATO.
