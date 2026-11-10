# Implementation roadmap

All work happens on `feat/strato-interactive-web-demo`. Each phase ends with tests, scientific checks, a `prompt/PROGRESS.md` update, a commit and a push (details in `prompt/AUTONOMOUS_EXECUTION.md` and `prompt/COMMIT_AND_DEPLOYMENT.md`). Proceed automatically when every acceptance criterion passes.

```
P0 -> P1 -> P2 ----------------------\
        \-> P3 -> P4 -> P5 -----------+-> P8
                    \-> P6 (gated: D1 to D14) -> P7 (predictions part)
                    P7 (data part, no model) can start after P5
```

| Phase | Size |
|---|---|
| 0 Audit and architecture | S |
| 1 Frontend foundation | M |
| 2 Equation explorer | S |
| 3 v1.0 solver integration | L |
| 4 Particle visualisation | M |
| 5 Scientific graphs | M |
| 6 Phase 2 physics | L |
| 7 Validation | M |
| 8 Polish and deployment | M |

---

## Phase 0: repository audit and architecture

- **Objectives**: confirm every fact in `prompt/` against the repository; confirm the wasm build path; confirm tools; record the pinned external references.
- **Files**: `prompt/PROGRESS.md` (audit notes, tool versions, corrections); fix any wrong fact in `prompt/` in the same commit.
- **Dependencies**: none.
- **Tasks**: run `cargo test --all-targets` and `cargo run --release --example verify`; run `cargo check --lib --target wasm32-unknown-unknown`; list public API in `src/` and confirm the signatures in `prompt/TECHNICAL_ARCHITECTURE.md` Section 3; read `docs/development/phase2_review_package.md`; read Outram Park and NUS-H2 files named in `prompt/GITHUB_REFERENCES.md` and `prompt/RAYSTORAGE_INTEGRATION.md` at their pinned commits.
- **Cross-project review (time-bounded, at most about one working session)**: read the NUS-H2 and raystorage files listed in `prompt/RAYSTORAGE_INTEGRATION.md` Section 4 at their pinned commits, in this priority order: Rust/WebAssembly patterns, numerical verification, browser testing, scientific provenance, documentation workflows. Correct that file where needed and note findings in `prompt/PROGRESS.md`. Open questions become notes, not blockers; Phase 1 starts when the time is up.
- **Acceptance**: tests pass on the untouched tree; wasm check passes; every discrepancy found is fixed in `prompt/` or listed as NOT VERIFIED.
- **Scientific checks**: the verified numbers in `prompt/SCIENTIFIC_REQUIREMENTS.md` Section 2 match `tests/golden/v1.0/` and the v1.0.0 documents.
- **Commit**: `docs(prompt): phase 0 - audit and architecture confirmation`.
- **Proceed when**: all of the above pass.

## Phase 1: frontend foundation

- **Objectives**: Vite + React + TypeScript app shell, routing between sections, design tokens, responsive layout, control panel with parameter controls bound to the store (no solver yet).
- **Files**: `web/package.json`, `web/package-lock.json`, `web/vite.config.ts` (`base: '/STRATO/'`), `web/tsconfig.json`, `web/index.html`, `web/src/main.tsx`, `web/src/App.tsx`, `web/src/state/store.ts`, `web/src/design/tokens.css`, `web/src/panels/*`, `web/tests/*`, `web/e2e/*`, `.github/workflows/web.yml`.
- **Dependencies**: Phase 0; `react`, `react-dom`, `vite`, `@vitejs/plugin-react`, `typescript`, `zustand`, `vitest`, `@testing-library/react`, `@playwright/test`, ESLint.
- **Acceptance**: `npm run build` succeeds; layout correct at 375, 768, 1280 and 1920 px; all controls keyboard operable; light and dark themes pass contrast checks; no console errors.
- **Tests**: `npm run lint`, `npm run typecheck`, `npm test`, `npx playwright test` (viewport, keyboard, reduced motion, console errors).
- **Scientific checks**: parameter defaults read from the TOML case (via a generated JSON), not typed by hand.
- **Commit**: `feat(web): phase 1 - dashboard foundation`.
- **Proceed when**: all tests pass and the existing Rust CI is still green.

## Phase 2: growing equation explorer

- **Objectives**: KaTeX equation with the term table of `prompt/TECHNICAL_ARCHITECTURE.md` Section 4; mechanism toggles in preview mode; term cards and documentation links; boundary and initial condition rows.
- **Files**: `web/src/equation/terms.ts`, `web/src/equation/EquationExplorer.tsx`, tests.
- **Dependencies**: Phase 1; `katex`; `motion` only if CSS transitions are insufficient (record the reason).
- **Acceptance**: baseline always visible; each toggle inserts and removes exactly its term; planned terms carry the PLANNED chip; links resolve to existing headings in `docs/mathematics/`; no number is shown.
- **Tests**: unit tests for term assembly per toggle combination (8 combinations); a link checker for the documentation anchors; e2e keyboard and reduced-motion checks.
- **Scientific checks**: LaTeX of each term matches the cited equation in `docs/mathematics/STRATO_Phase2_Derivations.md`.
- **Commit**: `feat(web): phase 2 - interactive equation explorer`.
- **Proceed when**: all tests pass.

## Phase 3: verified v1.0 solver integration

- **Objectives**: `crates/strato-wasm` wrapper, worker and client, live `run_v1` with intermediate profiles, analytical series, precomputed data pipeline.
- **Files**: `crates/strato-wasm/**`, `scripts/web/build_wasm.sh`, `scripts/web/precompute.sh`, `web/src/solver/**`, `web/public/data/*.json`, `.github/workflows/web.yml`.
- **Dependencies**: Phases 1 and 2; `wasm-bindgen` (crate and pinned CLI) or `wasm-pack`; `serde_json` in the wrapper.
- **Acceptance**: native wrapper output equals `tests/golden/v1.0/timeseries.csv` and `final_profile.csv` to the regression tolerance (relative 1e-12) and equals `finite_volume::run` exactly on the same platform; browser output equals native output within the parity tolerance (relative 1e-12, see `prompt/TESTING_AND_VALIDATION.md`); invalid inputs return `rejected` with messages; no panic reaches JavaScript; `git diff v1.0.1 -- src tests examples inputs` is empty.
- **Tests**: `cargo test --manifest-path crates/strato-wasm/Cargo.toml`; wasm export and size check; e2e parity test in the browser; precompute regeneration check.
- **Scientific checks**: balance residual reported and below the v1.0 level; Code B against Code C error matches `tests/golden/v1.0/comparison_fv_vs_analytical.csv`.
- **Commit**: `feat(web): phase 3 - rust solver integration`.
- **Proceed when**: parity and regression pass.

## Phase 4: interactive particle visualisation

- **Objectives**: 3D cutaway and 2D cross-section, concentration colour mapping, layer inspection, playback.
- **Files**: `web/src/particle/**`, `web/src/panels/Playback.tsx`, tests.
- **Dependencies**: Phase 3; `three`, `@react-three/fiber`.
- **Acceptance**: radii from config; colours equal the colour map applied to solver values (unit test on a known profile); playback at 30 fps or more measured in e2e; WebGL failure falls back to 2D; keyboard rotation and layer selection work.
- **Tests**: colour-map unit tests; e2e screenshots at four viewports; frame-rate measurement; reduced motion.
- **Scientific checks**: displayed layer inventory shares sum to the total inventory from the solver within 1e-12 relative.
- **Commit**: `feat(web): phase 4 - triso particle visualisation`.
- **Proceed when**: all pass.

## Phase 5: interactive scientific graphs

- **Objectives**: radial profile, release curves (Code B and Code C), model comparison with optional difference curve, verification and convergence panel, provenance table, downloads.
- **Files**: `web/src/plots/**`, `web/src/panels/Verification.tsx`, `Provenance.tsx`, `Downloads.tsx`, tests.
- **Dependencies**: Phase 3; `plotly.js` partial bundle.
- **Acceptance**: comparison and validation toggles are independent; difference curve equals modified minus baseline point by point; downloads contain metadata headers and reproduce the plotted values; every plot shows its label.
- **Tests**: unit tests for difference and download serialisation; e2e toggles and downloads.
- **Scientific checks**: convergence plot shows observed orders from `convergence_study.json` (spatial about 2, temporal about 1) with the explanation already in `docs/mathematics/v1.0.0/` for the low printed temporal orders.
- **Commit**: `feat(web): phase 5 - scientific comparison graphs`.
- **Proceed when**: all pass.

## Phase 6: Phase 2 physics (gated)

- **Gate**: every row D1 to D14 in `prompt/PROGRESS.md` "Owner decisions" has an owner choice. If any is PENDING, skip to the independent parts of Phases 7 and 8 and record the skip.
- **Objectives**: implement decay, source, inventory accounting, Arrhenius diffusivity and temperature histories in Rust following `docs/mathematics/STRATO_Phase2_Pseudocode.md` and the owner's decisions; expose them in the wrapper; enable the toggles.
- **Files**: new Rust modules beside the verified ones (for example `src/physics/decay.rs`, `src/physics/source.rs`, `src/physics/arrhenius.rs`, `src/physics/temperature.rs`), new tests, wrapper updates, docs updates (`docs/development/v1_to_v2_changes.md`, CHANGELOG).
- **Dependencies**: Phase 3; owner decisions.
- **Acceptance**: with decay and source off and constant temperature, results are bit-identical to v1.0 (cases V1, VT16, VI4 of the pseudocode); inventory closure (P21.42) holds to the documented tolerance; positivity holds; Booth single-sphere and CRP-6 cases 1a and 1b match their reference values; all v1.0 tests still pass.
- **Tests**: `cargo test --all-targets` plus the new Phase 2 tests; native/wasm parity on Phase 2 cases.
- **Scientific checks**: every Phase 2 verification case in `prompt/TESTING_AND_VALIDATION.md` Section 5.
- **Commit**: `feat(physics): phase 6 - verified model extensions` (only for what is complete and verified).
- **Proceed when**: all pass. Unfinished mechanisms stay disabled.

## Phase 7: experimental validation

- **Data part (no Phase 6 needed)**: load `data/experimental/release_measurements.csv`, show measurements with uncertainty and references in the validation graph, provenance and digitising caveats.
- **Prediction part (needs Phase 6)**: run COMPUTED v2.0 cases matching each measured series (nuclide, temperature history, particle geometry from `data/geometry/geometry.csv`), overlay predictions, compute error metrics.
- **Files**: `web/src/plots/Validation.tsx`, `scripts/web/precompute.sh` (validation cases), `web/public/data/validation_*.json`, tests.
- **Acceptance**: measurements plotted exactly as in the CSV; predictions only from COMPUTED v2.0 runs; metrics computed in Rust or in a documented script, not hand-entered; verification and validation clearly separated on screen.
- **Commit**: `feat(validation): phase 7 - literature comparison` (or `feat(validation): phase 7 - measurement display` if only the data part is done).
- **Proceed when**: all pass.

## Phase 8: polish and deployment

- **Objectives**: responsive and accessibility review, performance tuning, documentation (README section, `web/README.md`), Pages workflow, final PR.
- **Files**: `.github/workflows/web.yml` (Pages job), `README.md` (web demo section), `web/README.md`, `CHANGELOG.md`.
- **Acceptance**: all e2e checks pass; performance budget of `prompt/TECHNICAL_ARCHITECTURE.md` Section 6 measured and met or recorded; `npm audit` reviewed; build is reproducible (two clean builds give identical assets apart from the stamp).
- **Commit**: `feat(web): phase 8 - deployment and polish`.
- **Finish**: open a pull request into `main`; do not merge it. Pages deployment from `main` needs the owner to merge and to enable Pages.
