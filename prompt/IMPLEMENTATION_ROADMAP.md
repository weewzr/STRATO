# Implementation roadmap

All work happens on `feat/strato-interactive-web-demo`. Each phase ends with tests, scientific checks, a `prompt/PROGRESS.md` update, a commit and a push (details in `prompt/AUTONOMOUS_EXECUTION.md` and `prompt/COMMIT_AND_DEPLOYMENT.md`). Proceed automatically when every acceptance criterion passes.

The owner's equation-first brief (`prompt/EQUATION_FIRST_REQUEST.md`, owner decision M7 with rules E1 to E9 in `prompt/PROGRESS.md`) sets the page order (equation, live simulation, controls, explanation, future research), the staged equation builder, the five-level research status and the research catalogue. Every phase below builds in that order.

```
P0 -> P1 -> P2 ----------------------\
        \-> P3 -> P4 -> P5 -> P5.5 ---+-> P8
                    \-> P6 (gated: D1 to D16; 6.1 then 6.2) -> P7 (predictions part)
                    P7 (data part, no model) can start after P5
   catalogue content (branch docs/research-catalogue) --> P5.5
```

| Phase | Size |
|---|---|
| 0 Audit and architecture | S |
| 1 Frontend foundation (equation-first page order) | M |
| 2 Staged equation builder and research status | M |
| 3 v1.0 solver integration | L |
| 4 Particle visualisation | M |
| 5 Scientific graphs | M |
| 5.5 Research catalogue interface | M |
| 6 Phase 2 physics | L |
| 7 Validation | M |
| 8 Polish and deployment | M |

### Bricks of the brief mapped to phases

The brief builds the physics "brick by brick". Bricks 1 to 3 are delivered by the phases below; bricks 4 to 9 are catalogue entries only, with status PENDING and no implementation in this project.

| Brick | Brief | Delivered by | In this project |
|---|---|---|---|
| 1 | Baseline demonstration: equation presentation and visualisation of the verified physics | Phases 1 to 5 (equation-first layout) | build now |
| 2 | Source and decay terms | Phase 6.1 (scientific Phase 2.1), through its gate and verification | after Brick 1 |
| 3 | Temperature and layer-dependent transport, AGR-1 geometry and materials | Phase 6.2 (scientific Phase 2.2), then the Phase 7 comparison against the public data at each temperature | after Brick 2 |
| 4 | Silver trapping and detrapping | catalogue entry (content branch, shown in Phase 5.5) | PENDING, no implementation |
| 5 | Silver grain-boundary transport | catalogue entry | PENDING, no implementation |
| 6 | Palladium-assisted degradation | catalogue entry | PENDING, no implementation |
| 7 | Caesium retention and barrier integrity | catalogue entry | PENDING, no implementation |
| 8 | Irradiation and coating degradation | catalogue entry | PENDING, no implementation |
| 9 | Additional species (Sr, Eu, I, Ru, Xe, Kr) and downstream transport, including FLiBe and cover gas | catalogue entry (rule E4) | PENDING, no implementation |

A brick advances to VERIFIED or VALIDATED only with the evidence that `prompt/DESIGN_SPECIFICATION.md` Section 7.1 requires, recorded in `data/research/mechanisms.csv` and `prompt/PROGRESS.md`.

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

- **Objectives**: Vite + React + TypeScript app shell, routing between sections (including the Research section, empty until Phase 5.5), design tokens (with one colour per TRISO layer), the equation-first Explore page with blocks A to E in the order of `prompt/DESIGN_SPECIFICATION.md` Section 2.1 (placeholders where later phases fill in content; no left control column), the control block C with parameter controls bound to the store (no solver yet).
- **Files**: `web/package.json`, `web/package-lock.json`, `web/vite.config.ts` (`base: '/STRATO/'`), `web/tsconfig.json`, `web/index.html`, `web/src/main.tsx`, `web/src/App.tsx`, `web/src/state/store.ts`, `web/src/design/tokens.css`, `web/src/panels/*`, `web/tests/*`, `web/e2e/*`, `.github/workflows/web.yml`.
- **Dependencies**: Phase 0; `react`, `react-dom`, `vite`, `@vitejs/plugin-react`, `typescript`, `zustand`, `vitest`, `@testing-library/react`, `@playwright/test`, ESLint.
- **Starting point**: the uncommitted Phase 1 scaffold in the Stage B clone (described in `docs/development/frontend_audit.md`) is continued, not discarded; its left-panel layout is changed to the block order A to E.
- **Acceptance**: `npm run build` succeeds; blocks A to E appear in this order at every width; the fold requirement holds at 1440 x 900; layout correct at 375, 768, 1280, 1440 and 1920 px; all controls keyboard operable; light and dark themes pass contrast checks; no console errors.
- **Tests**: `npm run lint`, `npm run typecheck`, `npm test`, `npx playwright test` (viewport, block order, fold at 1440 x 900, keyboard, reduced motion, console errors).
- **Scientific checks**: parameter defaults read from the TOML case (via a generated JSON), not typed by hand.
- **Commit**: `feat(web): phase 1 - dashboard foundation`.
- **Proceed when**: all tests pass and the existing Rust CI is still green.

## Phase 2: staged equation builder and research status

- **Objectives**: block A of `prompt/DESIGN_SPECIFICATION.md` Section 3: the KaTeX equation with the six stages (base diffusion, generation, decay, temperature-dependent diffusivity, layer-dependent transport with layer-specific temperature laws, species-specific mechanisms) and the term table of `prompt/TECHNICAL_ARCHITECTURE.md` Section 4; for each stage the highlighted new term, its meaning card, its discrete update with a link to the code (or to the pseudocode while no code exists), the "Equation preview only" banner when the stage is not supported, the place of the inventory balance and of the baseline comparison (filled from Phase 3 and Phase 6); boundary and initial condition rows; the five-level research status system (chips with word and shape) read from the catalogue files; `isAvailable()` of `prompt/TECHNICAL_ARCHITECTURE.md` Section 4.
- **Files**: `web/src/equation/terms.ts`, `web/src/equation/EquationBuilder.tsx`, `web/src/equation/availability.ts`, `data/research/mechanisms.csv` and `data/research/references.csv` with the rows for the builder mechanisms (stages 1 to 5 and `interface_partition`, statuses exactly as recorded in `prompt/PROGRESS.md` for M7), `scripts/web/generate_data.py` (`catalogue` subcommand), `scripts/web/check_catalogue.py` with its own test fixtures, `web/src/generated/research_catalogue.json`, tests. The rows for bricks 4 to 9 come from the catalogue content branch (Phase 5.5); both use the same files and format.
- **Dependencies**: Phase 1; `katex`; `motion` only if CSS transitions are insufficient (record the reason).
- **Acceptance**: stage 1 always visible; each stage shows exactly its terms and highlights exactly its new term; every chip equals the status in `mechanisms.csv`; no status word is typed in a component; every mechanism switch is disabled while `isAvailable()` is false (it is false for everything except `diffusion_layered` once Phase 3 reports capabilities); links resolve to existing headings in `docs/mathematics/` and to existing code paths; no number is shown apart from equation numbers; `check_catalogue.py` passes and fails on each of its bad fixtures.
- **Tests**: unit tests for term assembly for the six stages and for all 8 combinations of the three Phase 6 switches; status-system tests and "no active toggle without solver support" tests (`prompt/TESTING_AND_VALIDATION.md` Section 3); a link checker for the documentation and code anchors; e2e keyboard, reduced-motion and fold checks.
- **Scientific checks**: LaTeX of each term and each discrete update matches the cited equation in `docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md` or `docs/mathematics/STRATO_Phase2_Derivations.md`; stage 5 states that layering is already in stage 1 and that interface partition coefficients are not in the model (rule E1).
- **Commit**: `feat(web): phase 2 - staged equation builder and research status`.
- **Proceed when**: all tests pass.

## Phase 3: verified v1.0 solver integration

- **Objectives**: `crates/strato-wasm` wrapper, worker and client, live `run_v1` with intermediate profiles, analytical series, precomputed data pipeline; `build_info` reports `capabilities` (`["diffusion_layered"]`); block B (under the equation) shows the run's release readout, inventory balance and numerical diagnostics.
- **Files**: `crates/strato-wasm/**`, `scripts/web/build_wasm.sh`, `scripts/web/precompute.sh`, `web/src/solver/**`, `web/public/data/*.json`, `.github/workflows/web.yml`.
- **Dependencies**: Phases 1 and 2; `wasm-bindgen` (crate and pinned CLI) or `wasm-pack`; `serde_json` in the wrapper.
- **Acceptance**: native wrapper output equals `tests/golden/v1.0/timeseries.csv` and `final_profile.csv` to the regression tolerance (relative 1e-12) and equals `finite_volume::run` exactly on the same platform; browser output equals native output within the parity tolerance (relative 1e-12, see `prompt/TESTING_AND_VALIDATION.md`); invalid inputs return `rejected` with messages; no panic reaches JavaScript; `git diff v1.0.1 -- src tests examples inputs` is empty.
- **Tests**: `cargo test --manifest-path crates/strato-wasm/Cargo.toml`; wasm export and size check; e2e parity test in the browser; precompute regeneration check.
- **Scientific checks**: balance residual reported and below the v1.0 level; Code B against Code C error matches `tests/golden/v1.0/comparison_fv_vs_analytical.csv`.
- **Commit**: `feat(web): phase 3 - rust solver integration`.
- **Proceed when**: parity and regression pass.

## Phase 4: interactive particle visualisation

- **Objectives**: 2D cross-section (default, at the top of block B directly under the equation) and 3D cutaway, concentration colour mapping, layer boundaries in the layer colours, layer inspection, playback; the AGR-1 reference-particle view drawn from `data/geometry/geometry.csv` through the generator (geometry only, rule E2).
- **Files**: `web/src/particle/**`, `web/src/panels/Playback.tsx`, tests.
- **Dependencies**: Phase 3; `three`, `@react-three/fiber`.
- **Acceptance**: radii from config; colours equal the colour map applied to solver values (unit test on a known profile); playback at 30 fps or more measured in e2e; WebGL failure falls back to 2D; keyboard rotation and layer selection work.
- **Tests**: colour-map unit tests; e2e screenshots at four viewports; frame-rate measurement; reduced motion.
- **Scientific checks**: the displayed layer inventory shares are the `layer_share` values returned by `run_v1` (owner decision M4: computed in the Rust wrapper, never in TypeScript); a wrapper test asserts that the layer inventories sum to the solver's total inventory within 1e-12 relative, and an e2e test asserts that the displayed values equal the wrapper's values.
- **Commit**: `feat(web): phase 4 - triso particle visualisation`.
- **Proceed when**: all pass.

## Phase 5: interactive scientific graphs

- **Objectives**: radial profile and release curves (Code B and Code C) in block B under the equation, the stage baseline comparison of block A, model comparison with optional difference curve, verification and convergence panel, provenance table, downloads.
- **Files**: `web/src/plots/**`, `web/src/panels/Verification.tsx`, `Provenance.tsx`, `Downloads.tsx`, tests.
- **Dependencies**: Phase 3; `plotly.js` partial bundle.
- **Acceptance**: comparison and validation toggles are independent; the difference curve is the `difference` series returned by `compare_v1` (owner decision M4: computed in Rust, never subtracted in TypeScript) and a wrapper test shows that it equals modified minus baseline point by point; downloads contain metadata headers and reproduce the plotted values; every plot shows its label.
- **Tests**: wrapper test for the difference series; unit tests for download serialisation; e2e toggles and downloads, and an e2e check that the plotted difference equals the wrapper's values.
- **Scientific checks**: convergence plot shows observed orders from `convergence_study.json` (spatial about 2, temporal about 1) with the explanation already in `docs/mathematics/v1.0.0/` for the low printed temporal orders.
- **Commit**: `feat(web): phase 5 - scientific comparison graphs`.
- **Proceed when**: all pass.

## Phase 5.5: research catalogue interface

- **Objectives**: block E of the Explore page and the Research section of `prompt/DESIGN_SPECIFICATION.md` Section 7: the three transport regimes, the nine species entries, the pending mechanism entries with candidate equations shown as research concepts and disabled switches, the Materials Studio gallery (Pending placeholders until genuine screenshots exist, rule E5), the brick roadmap page and the reference list, all rendered from `web/src/generated/research_catalogue.json`.
- **Content**: written in parallel on the branch `docs/research-catalogue` (its own clone): `data/research/*.csv` rows for bricks 4 to 9 and the species, `docs/research/*.md`, every statement of evidence with a primary source and a checked link, half-lives from `data/nuclear/decay.csv` or a cited source, no invented number. It enters the work branch by a normal merge, after `check_catalogue.py` passes on the merged tree.
- **Files**: `web/src/research/**`, the catalogue files of `prompt/TECHNICAL_ARCHITECTURE.md` Section 5.1, tests.
- **Dependencies**: Phase 2 (status system, catalogue format, generator and check); the catalogue content branch. Independent of Phase 6.
- **Acceptance**: nine species entries and three regimes render with every heading of the format; every status shown equals the CSV; every live-simulation switch in the catalogue is disabled; no image is shown unless its gallery row is `screenshot` or `reconstruction` with a file and provenance, and the two kinds are labelled differently; no concentration profile, barrier, diffusivity or release fraction appears that is not from a cited source or computed by STRATO; every reference link resolves or is marked NOT VERIFIED; screenshots of the running site at desktop and phone widths inspected and corrected.
- **Tests**: those of `prompt/TESTING_AND_VALIDATION.md` Section 3, row 5.5.
- **Scientific checks**: AGR experiments described as gas-cooled and kept apart from any FLiBe or cover-gas entry (rule E4); ideal 3C-SiC not presented as the polycrystalline coating; atomistic barriers not presented as effective diffusivities; stable and radioactive isotopes distinguished.
- **Commit**: `feat(web): phase 5.5 - research catalogue interface`.
- **Proceed when**: all pass.

## Phase 6: Phase 2 physics (gated)

- **Gate**: every row D1 to D16 in `prompt/PROGRESS.md` "Owner decisions" has an owner choice. The gate is split (owner decision of 2026-11-24):
  - **Phase 6.1** (scientific Phase 2.1: decay, source, inventory accounting at constant diffusivity) needs D1, D2, D3, D6, D9, D10, D11, D12, D13, D14, D15 and D16.
  - **Phase 6.2** (scientific Phase 2.2: temperature-dependent diffusivity and temperature histories) also needs D4, D5, D7 and D8, and starts only after Phase 6.1 has passed its acceptance.
  If a needed row is PENDING, skip that sub-phase, go to the independent parts of Phases 7 and 8 and record the skip. All sixteen rows were filled on 2026-11-24; open sub-items recorded beside a row (for example the D5 history format and reference temperature) must be settled by the owner before the code that needs them is written.
- **Objectives**: implement decay, source, inventory accounting (6.1, Brick 2), then Arrhenius diffusivity with layer-specific laws and temperature histories (6.2, Brick 3) in Rust following `docs/mathematics/STRATO_Phase2_Pseudocode.md` and the owner's decisions; expose them in the wrapper and add their ids to `capabilities`; enable the switches and the builder stages' live updates only for what is complete, tested and reviewed. Status changes in `data/research/mechanisms.csv`: RESEARCHED to IMPLEMENTED when the code exists, to VERIFIED only when the Phase 6 verification cases pass and the independent review accepts them, each with its evidence paths and a `prompt/PROGRESS.md` entry.
- **Files**: new Rust modules beside the verified ones (for example `src/physics/decay.rs`, `src/physics/source.rs`, `src/physics/arrhenius.rs`, `src/physics/temperature.rs`), new tests, wrapper updates, docs updates (`docs/development/v1_to_v2_changes.md`, CHANGELOG). Owner decisions D7 (read-only accessors to the layer states of `src/analytical/sturm_liouville.rs`) and D9 (region count 1 to 5 in `Config::validate`) need small changes inside `src/`; this phase allows exactly those, with the full v1.0 regression suite passing before and after and the change listed in `docs/development/v1_to_v2_changes.md`.
- **Dependencies**: Phase 3; owner decisions.
- **Acceptance**: with decay and source off and constant temperature, results are bit-identical to v1.0 (cases V1, VT16, VI4 of the pseudocode); inventory closure (P21.42) holds to the documented tolerance; positivity holds; Booth single-sphere and CRP-6 cases 1a and 1b match their reference values; all v1.0 tests still pass.
- **Tests**: `cargo test --all-targets` plus the new Phase 2 tests; native/wasm parity on Phase 2 cases.
- **Scientific checks**: every Phase 2 verification case in `prompt/TESTING_AND_VALIDATION.md` Section 5.
- **Commit**: `feat(physics): phase 6 - verified model extensions` (only for what is complete and verified).
- **Proceed when**: all pass. Unfinished mechanisms stay disabled.

## Phase 7: experimental validation

- **Data part (no Phase 6 needed)**: load `data/experimental/release_measurements.csv`, show measurements with uncertainty and references in the validation graph, provenance and digitising caveats.
- **Prediction part (needs Phase 6)**: run COMPUTED v2.0 cases matching each measured series (nuclide, temperature history, particle geometry from `data/geometry/geometry.csv`, AGR-1 UCO as the reference fuel), overlay predictions, compute error metrics at each temperature from the public data (AGR-1 safety tests, CRP-6 cases). Runs that use the UO2 kernel diffusivity rows carry the label of rule E3. A mechanism moves to VALIDATED only for the species and domain of a validation row that passes against its stated limit with `data_use` independent; until computed comparisons exist, no agreement figure is written anywhere (M5).
- **Files**: `web/src/plots/Validation.tsx`, `scripts/web/precompute.sh` (validation cases), `web/public/data/validation_*.json`, tests.
- **Acceptance**: measurements plotted exactly as in the CSV; predictions only from COMPUTED v2.0 runs; metrics computed in Rust or in a documented script, not hand-entered; verification and validation clearly separated on screen.
- **Commit**: `feat(validation): phase 7 - literature comparison` (or `feat(validation): phase 7 - measurement display` if only the data part is done).
- **Proceed when**: all pass.

## Phase 8: polish and deployment

- **Objectives**: responsive and accessibility review, performance tuning, documentation (README section, `web/README.md`), Pages workflow, final PR.
- **Files**: `.github/workflows/web.yml` (Pages job), `README.md` (web demo section), `web/README.md`, `CHANGELOG.md`.
- **Site content** (owner decisions M3 and M5): the `prompt/` folder is not published on the site (excluded from the Pages artefact, no link to it from the site); the README sentence about the internal SNRSI comparison is not repeated, quoted or extended anywhere on the site.
- **Deployment** (rule E7): `ci.yml` and `docs-pdf.yml` stay unchanged; the Pages artefact of `web.yml` also republishes the derivation pages that the Jekyll render publishes today, so no published document disappears when the Pages source is switched.
- **Acceptance**: all e2e checks pass, including the fold requirement at 1440 x 900 and the block order at every width; screenshots of the running site at desktop and phone widths inspected and every layout problem fixed; a link and content check shows no `prompt/` page and no SNRSI comparison text in the built site; performance budget of `prompt/TECHNICAL_ARCHITECTURE.md` Section 6 measured and met or recorded; `npm audit` reviewed; build is reproducible (two clean builds give identical assets apart from the stamp).
- **Commit**: `feat(web): phase 8 - deployment and polish`.
- **Finish**: open a pull request into `main`. When all mandatory tests, the independent verification and every acceptance criterion pass, the coordinator merges by the gated procedure of `prompt/COMMIT_AND_DEPLOYMENT.md` Section 6 (merge `main` into the work branch with a normal commit if needed, re-run CI, fast-forward push to `main`); `web.yml` then deploys to Pages, which needs the Pages source set to GitHub Actions (owner setting).
