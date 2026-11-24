# Progress

Update this file at the end of every phase and every session. The next session resumes from here.

## Current state

| Item | Value |
|---|---|
| Current phase | Phase 0 done (2026-11-24); Phase 1 next |
| Work branch | `feat/strato-interactive-web-demo`, created 2026-11-24 from `main` at `e3a558f` |
| Latest commit on `main` | `e3a558f` (2026-11-24). The last commit touching `prompt/` on `main` is `7fe4421` |
| STRATO version | 1.0.1 (licence AGPL-3.0-only since commit `8c44fb7`) |

## Completed milestones (before the web demo)

| Milestone | Reference |
|---|---|
| v1.0 Phase 1 baseline, verified | tag `v1.0.0` (commit `8c5d8a8`) |
| v1.0.0 mathematical documentation | tag `v1.0.1` (commit `e6d0f60`) |
| Phase 2 derivations, pseudocode, review package, parameter register | commit `eac2806` |
| Relicence to AGPL-3.0-only | commit `8c44fb7` |
| Web demo instruction package | commits `3a4319e`, `bc107b1` and `7fe4421` (2026-11-10) |
| Phase 2 open maths items O4, O10, O11, O12, R2.4, P2; equations to (P21.142) and (P22.142); decisions D15 and D16 added | commit `9802b2d` (2026-11-13) |

## Web demo phases

| Phase | Status | Commit | Tests run | Scientific checks |
|---|---|---|---|---|
| 0 Audit and architecture | done 2026-11-24: baseline, wasm check, public API confirmation, fact check of `prompt/`, cross-project review (see "Phase 0 audit") | `054510e` (baseline), `91cb327` (audit and architecture confirmation), `d458dad` (first review fix) and the second review fix recorded in the session log | fmt, clippy (both forms), `cargo test --locked --all-targets` (20 passed), `cargo doc` with `-D warnings`, release build, `verify`, synthetic run, `cargo check --locked --lib --target wasm32-unknown-unknown`, all re-run on the Phase 0 tree; no web build or web tests exist yet (no `web/` until Phase 1) | outputs equal `tests/golden/v1.0/` exactly as specified; the verified numbers of `prompt/SCIENTIFIC_REQUIREMENTS.md` Section 2 traced to their files (one correction, see below) |
| 1 Frontend foundation | not started | | | |
| 2 Staged equation builder and research status | not started | | | |
| 3 v1.0 solver integration | not started | | | |
| 4 Particle visualisation | not started | | | |
| 5 Scientific graphs | not started | | | |
| 5.5 Research catalogue interface | not started (content on branch `docs/research-catalogue`) | | | |
| 6.1 Phase 2.1 physics (decay, source, inventory) | not started; gate open (D1, D2, D3, D6, D9 to D16 decided 2026-11-24) | | | |
| 6.2 Phase 2.2 physics (temperature-dependent diffusivity) | not started; gate open (D4, D5, D7, D8 decided 2026-11-24; D5 sub-items open, see the table); starts after 6.1 passes | | | |
| 7 Validation | not started | | | |
| 8 Polish and deployment | not started | | | |

## Known blockers

| Id | Blocker | Needed from |
|---|---|---|
| B1 | RESOLVED 2026-11-24. Owner decisions D1 to D16 recorded below; the Phase 6 gate is split into 6.1 and 6.2. Remaining sub-items (D5 history format and reference temperature) are open questions for Phase 6.2, not a blocker for 6.1 | owner |
| B2 | OPEN. The Pages source must be switched from "Deploy from a branch" to "GitHub Actions" (Settings, Pages, Source). Until then the live site stays the Jekyll render of `main`. Merging into `main` is done by the gated procedure of `prompt/COMMIT_AND_DEPLOYMENT.md` Section 6 | owner |
| B3 | RESOLVED 2026-11-24. Pushing the work branch to `weewzr/STRATO` works from the development environment | none |
| B4 | Open scientific items from the review package: CRP-6 gas constant and decay treatment; data gaps O1, O7, O8, O9; no stiff-regime staircase proof | owner, later phases |

## Remaining work

Phases 1 to 8, including Phase 5.5 (research catalogue interface) added by owner decision M7.

## Next actions

1. Phase 1 (frontend foundation) under `prompt/IMPLEMENTATION_ROADMAP.md`, in the equation-first page order of `prompt/DESIGN_SPECIFICATION.md` Section 2 (owner decision M7), continuing from the uncommitted scaffold described in `docs/development/frontend_audit.md`, with the visual identity of `prompt/DESIGN_SPECIFICATION.md` Section 9.1 (owner decision M6). `tokens.core.css` is to be copied from the owner's portfolio repository once it exists there; on 2026-11-24 it was not yet on that repository's `main`, so Phase 1 records the version it copies, or writes the file to the written specification and records that.
2. Phase 1 also starts the browser self-test skeleton and the Playwright checks (`prompt/RAYSTORAGE_INTEGRATION.md` Section 8).
3. Phase 3: follow the wrapper facts table and the panic rule added to `prompt/TECHNICAL_ARCHITECTURE.md` Section 3 in Phase 0 (corrected after review, see "Corrections after the Phase 0 review"), and set and record `MAX_TOTAL_CELLS` and `MAX_STEPS`.

## Phase 0 audit (2026-11-24)

### Tools

Windows 11, PowerShell 5.1, per-user installs, no administrator rights. rustc 1.100.0 (0ef1bc386 2026-11-09), cargo 1.100.0, LLVM 23.1.5, host `x86_64-pc-windows-gnu`; installed targets `x86_64-pc-windows-gnu` and `wasm32-unknown-unknown`; components clippy, rustfmt, rust-src, llvm-tools; `wasm-bindgen` CLI 0.2.130; Node v24.22.3 with npm 11.19.1; Python 3.12.10; pandoc 3.14; git 2.56.1; Microsoft Edge 158.0.4370.33 (the Chromium for browser tests). No TeX and no LibreOffice on this machine, so the docs-PDF workflow cannot be run locally. No `gh` CLI: the cross-project files were read from blobless clones (see `prompt/RAYSTORAGE_INTEGRATION.md` Section 2).

### Checks re-run on the Phase 0 tree

| Check | Result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --locked -- -D warnings`; `cargo clippy --locked --all-targets --all-features -- -D warnings` | exit 0; exit 0 |
| `cargo test --locked --all-targets` | 14 + 0 + 3 + 3 + 0 = 20 passed, 0 failed, 0 ignored |
| `cargo doc --locked --no-deps` with `RUSTDOCFLAGS=-D warnings` | exit 0 |
| `cargo build --locked --release` | exit 0 |
| `cargo run --locked --release --example verify` | `FV_VERIFICATION PASS`; standard output equal to `tests/golden/v1.0/verify_stdout.txt` line for line (34 lines, all equal apart from `VERIFY_RUNTIME_S`; `VERIFY_RUNTIME_S` was 0.218527 here; an earlier version of this row said 33 lines in error) |
| `cargo run --locked --release -- inputs/synthetic/synthetic_five_layer.toml` | standard output equal to `main_run_stdout.txt` after normalising path separators (23 lines, 0 differing); the four CSV files byte-identical to the golden copies (SHA-256 equal) |
| `cargo check --locked --lib --target wasm32-unknown-unknown` | exit 0 |
| `git diff v1.0.1..HEAD -- src tests examples inputs` and the same against the working tree | empty |

### Public API confirmed against `prompt/TECHNICAL_ARCHITECTURE.md` Section 3

Every item the wrapper needs is public in `src/` at `v1.0.1`: `config::Config` with `RegionConfig`, `InitialConfig`, `TimeConfig`, `OutputConfig` (all fields public, `Deserialize` only), `Config::validate` at `src/config.rs` line 79, `config::PLACEHOLDER`; `numerical::finite_volume::{build_mesh, initial_concentration, inventory, backward_euler_step, run, explicit_dt_bound, Mesh, Cell, StepDiagnostics, TimeSeries}` with `Mesh.cells`, `Cell.volume` and `Cell.material` public; `postprocessing::comparison::{compare_release, convergence_study, layers_from_config, exact_release_at, MAX_MODES, ReleaseComparison, ConvergenceStudy}`; `analytical::sturm_liouville::{ModalRelease, exact_release, eigenvalues, Layer}`. Private as stated: `unfilled_placeholders` and the shooting layer states (D7 later adds read-only accessors). The step rule, the Kahan release sum and the balance residual of `finite_volume::run` are as Section 3 step 3 describes. Added to Section 3: a table of the facts the wrapper must respect (no `Serialize`, the required `output.directory`, the `validate` rules (first version incomplete, corrected after review below), `Cell.material` as region index, the Code C truncation `45 / t_min`) and a correction of the panic rule: `wasm32-unknown-unknown` uses `panic="abort"`, so `catch_unwind` cannot catch a panic in the browser; the wrapper must validate first and the worker must recover a trapped instance.

### Scientific check of `prompt/SCIENTIFIC_REQUIREMENTS.md` Section 2

Every number was traced to a repository file. All values agree with the repository, but the text called $F(2\times10^5\,\mathrm{s}) = 0.43255563909304$ a value from `tests/golden/v1.0/`: it is the 40-digit independent reference of the v1.0.0 derivation (0.43255563909304174), while the golden Code C value is 0.43255563909285666 (difference $1.9\times10^{-13}$, below the series accuracy floor $3.997\times10^{-12}$). The eigenvalues and the same-mesh temporal orders are in the v1.0.0 documents only, not in any golden file. Section 2 now lists each value with its file, including the Code B value 0.4324286068785298 and the printed temporal orders 0.98 to 0.89.

### Cross-project review (time-bounded, one session)

Read at the pinned commits, in the priority order of `prompt/RAYSTORAGE_INTEGRATION.md` Section 8; nothing executed, nothing copied.

1. Rust/WebAssembly patterns. NUS-H2 uses a raw zero-import `cdylib` (`alloc`, `dealloc`, `call`, `abi_version`), status words `ok`, `not_converged`, `rejected`, `null` plus `engine_fields_missing` for values the engine lacks (the page prints `n/a` and never fills the gap), and measured timings with date and conditions. Outram Park uses `wasm-bindgen` with the CLI pinned to `Cargo.lock` and the hello handshake. STRATO keeps `wasm-bindgen` (decided in `prompt/TECHNICAL_ARCHITECTURE.md`); the zero-import check therefore does not transfer, the export and size check does.
2. Numerical verification. NUS-H2 `GOLD_TOL = 1e-9` in `engine/vv/src/golden.rs` confirmed; STRATO keeps 1e-12. NUS-H2 CI regenerates its result files and compares them with the committed copies (`scripts/compare_results.py`), the model for STRATO's precompute regeneration check.
3. Browser testing. `frontend/selftest.js` writes the verdict to `document.body.dataset.selftest`, checks the engine commit against `build.json` and the page stamp, and fetches every asset with `?v=__COMMIT__`; `site.yml` runs the self-test at normal and reduced motion with `--expect-commit`. Adopted as written in `prompt/TESTING_AND_VALIDATION.md` Section 4.
4. Scientific provenance. Besides `data_use`, NUS-H2 classifies each reference value by `data_kind`; only `measured` rows validate physics. Added to `prompt/RAYSTORAGE_INTEGRATION.md` Section 4 for STRATO's Phase 7.
5. Documentation workflows. `math/eq_register.csv` columns confirmed (`id, section, tier, title, latex, variables, assumptions, source, tag, engine_fn, test_id, validation`); STRATO's planned register keeps its own columns (Section 7 of that file). NUS-H2's `scripts/check_site.py` content guard is the model for the Phase 8 check of owner decisions M3 and M5.

Corrections made: `prompt/RAYSTORAGE_INTEGRATION.md` (reading route without `gh`, newer `main` commits noted and not used, exact archive counts, exact `check_wasm.py` rule and why its zero-import rule does not transfer, two NUS-H2 script rows added, `data_kind` rule, Phase 0 path check recorded); `prompt/GITHUB_REFERENCES.md` Appendix B (Phase 0 path check; `__BUILD__` is in four other Outram Park web builds, not in the dispersion build); `prompt/TECHNICAL_ARCHITECTURE.md` Sections 1 and 3 (re-run wasm check, wrapper facts, panic rule, stamp locations, CLI version); `prompt/SCIENTIFIC_REQUIREMENTS.md` Section 2 (sources of the verified numbers); `prompt/DESIGN_SPECIFICATION.md` Section 4.1 (owner decision M6).

### Fact check of `prompt/`

Every repository path cited in `prompt/*.md` was tested. All existing-file references resolve, except `docs/mathematics/STRATO_Mathematical_Derivations.md` (cited by `src/lib.rs` line 13, frozen; already noted below). The other unresolved paths are files the plan creates (`web/`, `crates/strato-wasm/`, `scripts/web/`, `src/physics/`, `.github/workflows/web.yml`, `docs/mathematics/equation_register.csv`), paths inside the other repositories (checked there), or the suggested project allow-list file of `prompt/AUTONOMOUS_EXECUTION.md` Section 2, which the agent does not apply to itself.

Noted, not changed (outside `prompt/`):

- `docs/development/phase2_review_package.md` Section 3 still says the full set is (P21.1) to (P21.92) and (P22.1) to (P22.59); since `9802b2d` it runs to (P21.142) and (P22.142). Its Section 7 still shows D15 and D16 as PENDING; the owner's choices in this file rank above it.
- The four golden files that no test asserts (`convergence_study.csv`, both stdout files) are compared only by hand; automating that comparison in `web.yml` stays open work (`prompt/TESTING_AND_VALIDATION.md` Section 1).

### Corrections after the Phase 0 review (2026-11-24)

An independent review of the Phase 0 commit found errors in text that Phase 0 itself added. Corrected:

- `prompt/TECHNICAL_ARCHITECTURE.md` Section 3, wrapper facts table. It listed the `Config::validate` rules as exact and called them "the only rejection rules". That was wrong: `validate` also rejects a blank `species` (`src/config.rs` line 83) and a non-finite or non-positive `temperature_k` (line 86), and it accepts a start concentration of 0, which `finite_volume::run` then rejects (`initial inventory must be positive`, `src/numerical/finite_volume.rs` line 187). The table now lists every `bail!` and `expect(` rejection found by the search named below (index and arithmetic panics were not audited separately; see the second review below): serde parse errors (every field required, `temperature_k` included), each `validate` rule with its line, `build_mesh` (fewer than two cells), the run-time positive-inventory check (which the wrapper applies itself before stepping, because it steps with `backward_euler_step`; the UI requires `c0 > 0`), the step and solve checks, the Code C rejections (FV result kept, `analytical` set to `null` with the message), and the wrapper limits. It was rebuilt from a search of `src/` for `bail!`, `expect(`, `unwrap(`, `panic!` and `assert!`.
- `prompt/TECHNICAL_ARCHITECTURE.md` Section 3, panic paragraph. The list of library `expect` calls missed `src/postprocessing/comparison.rs` line 147, and the text claimed that every panic path was unreachable although `validate` sets no upper bound on cells or steps (a failed allocation also traps the wasm instance; a very large `end_time_s / dt_s` gives an unbounded loop). The paragraph now names all four library `expect` calls, adds the wrapper constants `MAX_TOTAL_CELLS` and `MAX_STEPS` (values to be set in web Phase 3 from measured browser run times and memory, and recorded here), and describes the worker time-out and instance recreation as the containment for whatever the guards miss.
- This file, Phase 0 check table: the `verify` standard output has 34 lines, not 33 (all equal apart from `VERIFY_RUNTIME_S`).
- `prompt/SCIENTIFIC_REQUIREMENTS.md` Section 2: the observed orders are recorded in `main_run_stdout.txt` and `verify_stdout.txt`; `convergence_study.csv` holds only the errors from which they follow.

### Corrections after the second Phase 0 review (2026-11-24)

A second independent review of the fix commit `d458dad` found three defects, corrected here:

- `prompt/TECHNICAL_ARCHITECTURE.md` Section 3: the paragraph about the rejection list had been placed inside the wrapper facts table, so the six rows after it lost the table header and rendered as plain text. The paragraph now follows the last row, and the table renders as one table (checked with `pandoc -f gfm -t html`).
- The same paragraph claimed that the rows were "the complete list of rejection paths". It now says that the rejection rows, the `expect` calls of defence 1 and the copied placeholder rule of `Config::load` cover every `bail!` and `expect(` found by the stated search of `src/` at `v1.0.1` except two the wrapper never calls (`convergence_study`, `src/postprocessing/comparison.rs` line 188, precomputed only; `src/main.rs` line 75, CLI; the search was re-run for this fix and lists 27 sites), and that index and arithmetic panics were not audited separately and are contained by the worker (defence 3). The `backward_euler_step` row now also names the internal check `invalid system dimensions` in `solve_diffusion_system` (`src/numerical/finite_volume.rs` line 134), which the wrapper cannot reach.
- This file: the `verify` row repeated "apart from `VERIFY_RUNTIME_S`"; the Phase 0 status row and the session log named the audit commit only by its subject and did not record `d458dad`.

NOT VERIFIED after Phase 0: the exact commit behind the live Outram Park site; the performance numbers of `prompt/TECHNICAL_ARCHITECTURE.md` Section 6 (no build exists); the docs-PDF build of the two Phase 2 documents (no TeX here).

## Phase 0 baseline (2026-11-24)

Run on the untouched v1.0 tree (`main` at `e3a558f` plus `prompt/` edits only) on Windows 11 with rustc 1.100.0 (0ef1bc386 2026-11-09) and cargo 1.100.0, host `x86_64-pc-windows-gnu`, targets `x86_64-pc-windows-gnu` and `wasm32-unknown-unknown`; wasm-bindgen CLI 0.2.130 present.

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt --all --check` | pass (exit 0, no output) |
| Lint | `cargo clippy -- -D warnings` | pass (exit 0) |
| Lint, CI form | `cargo clippy --all-targets --all-features -- -D warnings` | pass (exit 0) |
| Tests | `cargo test --all-targets` | pass: 14 unit, 3 cross-method (`tests/finite_volume_vs_analytical.rs`), 3 regression (`tests/phase1_regression.rs`); 20 passed, 0 failed, 0 ignored. Re-run on the commit that recorded the decisions: same result |
| Docs | `cargo doc --no-deps` with `RUSTDOCFLAGS=-D warnings` | pass (exit 0) |
| Release build | `cargo build --release` | pass (exit 0) |
| Wasm check | `cargo check --lib --target wasm32-unknown-unknown` | pass (exit 0) |
| Verification | `cargo run --release --example verify` | prints `FV_VERIFICATION PASS` (exit 0); `VERIFY_RUNTIME_S 0.222089` on this machine |
| Synthetic run | `cargo run --release -- inputs/synthetic/synthetic_five_layer.toml` | exit 0; `final_release_fraction=4.324286068785e-1`, `max_final_balance_residual_mol=9.402717119255e-24` |
| Frozen code | `git diff v1.0.1..HEAD -- src tests examples inputs` | empty (0 lines) |

Comparison with `tests/golden/v1.0/`, made as in `docs/development/provenance.md` Section 3:

| Output | Rule | Result |
|---|---|---|
| `verify` standard output | identical apart from the `VERIFY_RUNTIME_S` line | identical: same line count, 0 differing lines |
| main-run standard output | identical after normalising path separators (the Windows run prints `results/synthetic-five-layer\timeseries.csv`, the golden file has `/`) | identical: 0 differing lines |
| `timeseries.csv` | byte for byte | byte-identical (14244 bytes, same SHA-256) |
| `final_profile.csv` | byte for byte | byte-identical (9701 bytes) |
| `convergence_study.csv` | byte for byte | byte-identical (650 bytes) |
| `comparison_fv_vs_analytical.csv` | byte for byte | byte-identical (7159 bytes). The one last-digit difference at t = 6.8e4 s recorded in provenance Section 3 for rustc 1.96.0 does not occur with this GNU-host toolchain |

No golden file was edited. The CLI writes its CSV files to `results/synthetic-five-layer/`, which is git-ignored.

## Corrections made on 2026-11-24 (Stage A audit)

- `prompt/TESTING_AND_VALIDATION.md` Section 1 said CI compares the synthetic-run outputs with `tests/golden/v1.0/`. It does not: CI only uploads them. The golden regression test asserts the time series, the final profile and the analytical series; `convergence_study.csv` and the two stdout files are asserted by no test. The text now says so.
- `prompt/SCIENTIFIC_REQUIREMENTS.md` gave the Phase 2 equation ranges as (P21.1) to (P21.92) and (P22.1) to (P22.59). Since `9802b2d` they run to (P21.142) and (P22.142) (counted from the `\tag` entries of `docs/mathematics/STRATO_Phase2_Derivations.md`).
- `prompt/README.md` said the package was written from `8c44fb7` without naming the package commits; it now names `3a4319e`, `bc107b1`, `7fe4421` and this update.
- This file had placeholder commits ("the commit that added `prompt/`"); they are filled in above.
- D15 and D16 (added to the review package on 2026-11-13) were missing from the decision table and from every gate reference; the gate now reads D1 to D16 everywhere.
- `prompt/IMPLEMENTATION_ROADMAP.md` Phase 4 and Phase 5 computed layer inventory shares and the difference curve in TypeScript; both now come from the Rust wrapper (M4).
- `prompt/COMMIT_AND_DEPLOYMENT.md` asked for an assistant attribution trailer; replaced by owner-only attribution (M2).
- Noted, not changed (frozen `src/`): `src/lib.rs` line 13 cites `docs/mathematics/STRATO_Mathematical_Derivations.md`, which does not exist.
- Noted, not changed (outside `prompt/`): nine scripts under `scripts/derivation_checks/` and `scripts/figures/` carry a local absolute virtual-environment path in their docstrings, against the package's own privacy rule.

## Owner decisions

Only the repository owner decides the "Owner choice" and "Date" columns. An agent never chooses them; it may write them only to record a choice the owner has made, naming the source. Proposed defaults come from `docs/development/phase2_review_package.md` Section 7 and are not approved by being listed here. The choices below were made by the owner at the Stage A checkpoint and recorded on the owner's instruction.

Gate split (owner, 2026-11-24): web Phase 6.1 (scientific Phase 2.1: decay, source, inventory) needs D1, D2, D3, D6, D9, D10, D11, D12, D13, D14, D15 and D16; web Phase 6.2 (scientific Phase 2.2: temperature-dependent diffusivity and temperature histories) also needs D4, D5, D7 and D8.

| Id | Topic | Options | Proposed default | Owner choice | Date | Source |
|---|---|---|---|---|---|---|
| D1 | Nuclides first | any of Cs-137, Sr-90, Kr-85, I-131, Ag-110m | Cs-137, Sr-90, Kr-85, I-131; Ag-110m once activation data exist | Cs-137 and Sr-90 first; Kr-85 second, with the precursor-delay caveat; I-131 deferred (no PyC or SiC diffusivity in the register); Ag-110m later | 2026-11-24 | owner, Stage A checkpoint |
| D2 | Yields | cumulative $y^C$ or independent $y^I$ | $y^C$ only, keep the precursor warning | cumulative yields $y^C$ only | 2026-11-24 | owner, Stage A checkpoint |
| D3 | Activation products | FUTURE, or equations without data | FUTURE | later; not in v2.0 (stay FUTURE) | 2026-11-24 | owner, Stage A checkpoint |
| D4 | Diffusivity set and gas constant | TECDOC-978 FRG, Collin et al. (2016), Hales et al. (2021); SI or per-row $R_g$ | owner to choose set; SI with `FIXED_8_3143` option | TECDOC-978 FRG two-branch diffusivity rows; SI gas constant by default with a selectable 8.3143 option (`FIXED_8_3143`); AGR-1 correction factors are never shipped as defaults | 2026-11-24 | owner, Stage A checkpoint |
| D5 | Temperature histories | AGR-1 plans, CRP-6 schedules, or both; history format; reference temperature | highest temperature of the history | ship both the reconstructed AGR-1 safety-test plans and the CRP-6 schedules, each labelled with its evidence class. Not decided by this choice: the history format (one kind with repeated times or two kinds) and the reference temperature of the effective time; the proposal stays a proposal until the owner settles them before Phase 6.2 code needs them | 2026-11-24 | owner, Stage A checkpoint |
| D6 | Decay time integration | Backward Euler with implicit decay (P21.58) or exponential split (P21.66) | Backward Euler; outside inventory by (P21.73) | unsplit Backward Euler with implicit decay (P21.58) | 2026-11-24 | owner, Stage A checkpoint |
| D7 | Modal projection under piecewise temperature | in scope or not; accessors or a copy of shooting code; staircase as test helper | owner to choose | block R3 in scope as a verification reference, through read-only accessors to the shooting layer states; the staircase only as a test helper | 2026-11-24 | owner, Stage A checkpoint |
| D8 | Conductance time level | option A (step average) or E (end of step) | option A | option A (step-average diffusivity) as default, option E selectable | 2026-11-24 | owner, Stage A checkpoint |
| D9 | Source region | single-region particle or per-region source flag | owner to choose | configurable region count 1 to 5, with a kernel-only source as default; the change to `Config::validate` is made under the full v1.0 regression suite | 2026-11-24 | owner, Stage A checkpoint |
| D10 | Source input | fission rate density only, or burnup or power | fission rate density only | fission rate density is the only source input | 2026-11-24 | owner, Stage A checkpoint |
| D11 | Source step average | exact average (A) only, or keep rectangle (R) | exact average only | exact step-average source (A) only | 2026-11-24 | owner, Stage A checkpoint |
| D12 | Step limits | hard caps or warnings | owner to choose | hard caps in the adaptive planner; warnings in FIXED mode | 2026-11-24 | owner, Stage A checkpoint |
| D13 | Modal block R1 scope | kernel-uniform source and start profile only | kernel-uniform only, general projection FUTURE | kernel-uniform source and start profile only | 2026-11-24 | owner, Stage A checkpoint |
| D14 | Step planner | merged planner of pseudocode block F | accept | the merged step planner of pseudocode block F | 2026-11-24 | owner, Stage A checkpoint |
| D15 | Integrator (review package item 15, added 2026-11-13) | Backward Euler only; Backward Euler with TR-BDF2; Backward Euler with the exponential integrator | Backward Euler only for v2.0 (Stage A recommendation) | Backward Euler as the only integrator | 2026-11-24 | owner, Stage A checkpoint |
| D16 | Proposed step changes of pseudocode Section 9.5 (review package item 16, added 2026-11-13) | any of the proposals from O4, O10, O11, O12, P2 and R2.4 | the F3 to F5 diffusion-step rule and the R2.4 step-3 correction; defer the rest (Stage A recommendation) | adopt only the F3 to F5 diffusion-step rule and the R2.4 step-3 correction; defer the rest (including R1.0 mode counting, R2.5 and H4b) | 2026-11-24 | owner, Stage A checkpoint |

### Owner policy decisions

These rank first in the authority order of `prompt/MASTER_PROMPT.md`, above every other file in `prompt/`. They were recorded on the owner's instruction.

| Id | Decision | Replaces | Source | Date |
|---|---|---|---|---|
| M1 | Merge policy. After all mandatory tests, the independent verification and every acceptance criterion pass, the coordinator merges `main` into the work branch with a normal commit if needed, re-runs CI, and then fast-forward pushes the work branch to `main` (`prompt/COMMIT_AND_DEPLOYMENT.md` Section 6). Never the GitHub merge button or `gh pr merge`, never force, never a history rewrite. Every scientific safeguard and every other rule of the package stays as it is. | every earlier statement that forbids merging into `main` or requires the final pull request to stay unmerged, including the verbatim records `prompt/ORIGINAL_REQUEST.md` ("Do not merge into `main` without explicit authorisation"; this row is that authorisation) and `prompt/RAYSTORAGE_REQUEST.md` ("Do not force-push or merge into main"; the force-push ban stands) | owner, Stage A checkpoint | 2026-11-24 |
| M2 | Commit attribution. Commits are attributed to the owner only; no assistant attribution of any kind in commit messages, pull requests or files | the attribution-trailer sentence of `prompt/COMMIT_AND_DEPLOYMENT.md` Section 2 | owner, Stage A checkpoint | 2026-11-24 |
| M3 | The `prompt/` folder is not published on the website (not in the Pages artefact, not linked from the site) | none (new) | owner, Stage A checkpoint | 2026-11-24 |
| M4 | Every quantity shown in the browser comes from the Rust solver through the WebAssembly wrapper or from verified precomputed output. No scientific equation or derived quantity is computed in JavaScript or TypeScript, including layer inventory shares and difference curves | the TypeScript computation of layer inventory shares (roadmap Phase 4) and of the difference curve (roadmap Phase 5) | owner, Stage A checkpoint | 2026-11-24 |
| M5 | The sentence in the repository `README.md` about the internal SNRSI comparison is frozen exactly as it stands: not deleted, extended, repeated or quoted anywhere else (not in the demo, the portfolio or any presentation). Until computed per-temperature comparisons exist, which the owner will ask for later, no experimental-agreement figure is written anywhere | none (new) | owner, Stage A checkpoint | 2026-11-24 |
| M6 | Visual identity and design rules. STRATO is "an instrument with a monograph behind it": open-licensed (OFL) fonts only, self-hosted (Literata, IBM Plex Sans, IBM Plex Mono); KaTeX with numbered equations; cividis colour map; the package's own result badges; a copied `tokens.core.css` from the shared written design specification in the portfolio repository; no shared component library; no command palette on this site; third-party component kits are inspiration only, nothing copied. Details in `prompt/DESIGN_SPECIFICATION.md` Section 9.1 (Section 4.1 before the restructuring for M7) | the open font and token choices of `prompt/DESIGN_SPECIFICATION.md` Section 4 (made specific, not replaced) | owner, Stage A checkpoint | 2026-11-24 |
| M7 | Equation-first brief (`prompt/EQUATION_FIRST_REQUEST.md`, verbatim). The demo page shows the governing equation first and the live simulation immediately under it, then the controls, the scientific explanation and the future-research mechanisms; a staged equation builder; a five-level research status per mechanism (PENDING, RESEARCHED, IMPLEMENTED, VERIFIED, VALIDATED); a research catalogue in three transport regimes with nine species (Ag, Pd, Cs, Sr, Eu, I, Ru, Xe, Kr); a Materials Studio gallery that shows only genuine screenshots and otherwise Pending placeholders; AGR-1 UCO TRISO as the reference fuel; physics grows one verified mechanism at a time. The brief outranks `prompt/DESIGN_SPECIFICATION.md` where they differ. The points it leaves open are settled by the rules E1 to E9 below, which are in force as stated. M1 to M6, D1 to D16 and every scientific safeguard of the package stay as they are | the left-panel page structure of `prompt/DESIGN_SPECIFICATION.md` Section 2, the independent-toggle explorer of Section 3.2 and its two-level IMPLEMENTED / PLANNED chip (all rewritten to agree with the brief) | owner, brief of 2026-11-24 | 2026-11-24 |

### Rules in force from the equation-first brief (M7, 2026-11-24)

The brief was compared with the package and the repository before any change (comparison made at `414c591`). Where it leaves a point open, the rule below applies until the owner says otherwise. Each rule is binding on every later phase.

| Id | Point | Rule in force |
|---|---|---|
| E1 | Builder stage 5, "layer-dependent transport", while the v1.0 equation is already layered (each of the five layers has its own $D_i$) | Stage 1 shows the layered v1.0 equation as it is. Stage 5 adds only what is new: a separate temperature law for each layer (layer-specific Arrhenius coefficients $D_{0,i,m}$, $Q_{i,m}$, `docs/mathematics/STRATO_Phase2_Derivations.md` Section 5.4, part of web Phase 6.2). Interface partition coefficients are not in the model and are listed as PENDING. The page says this plainly |
| E2 | AGR-1 UCO as the reference fuel, while the verified case is synthetic | The AGR-1 particle geometry from `data/geometry/geometry.csv` is shown as the reference particle, labelled with the row ids and register status of its values. The live simulation of Brick 1 stays the verified synthetic case and says so. AGR-1 simulations arrive with Brick 3, when temperature-dependent diffusivities exist (web Phase 6.2). AGR-1 geometry is never combined with the synthetic diffusivities, because that would produce numbers with no physical meaning |
| E3 | UCO kernel diffusivity: the registered kernel rows are for UO2 (IAEA TECDOC-978 Table A-1; Collin et al. (2016) Table 2 records "UO2 values used for UCO") | Recorded as a data gap. Any AGR-1 run that uses UO2 kernel values carries a visible label that names this assumption and the rows used |
| E4 | "STRATO's downstream FLiBe and cover-gas modelling": the repository contains nothing on FLiBe or cover gas | Listed under Brick 9 as PENDING and described only as planned. The site states that the AGR experiments were made under gas-cooled conditions and keeps them apart from any later FLiBe or cover-gas work |
| E5 | Materials Studio screenshots: none exist in the repository | The gallery ships with Pending placeholders that describe each planned calculation. No imitation image, rendering or mock-up of a Materials Studio screen. A genuine screenshot goes in only when the owner supplies it with its method and conditions |
| E6 | MatterViz first for atomistic display | Assessed in `docs/development/visualisation_library_comparison.md`. No atomistic viewer is installed while there is no structure file to show; the choice is recorded for when one exists |
| E7 | "Preserve the existing deployment workflow", while the package adds a Pages workflow that needs the Pages source switched | `.github/workflows/ci.yml` and `docs-pdf.yml` stay unchanged. `web.yml` is added as planned, and the new build republishes the derivation pages that the Jekyll render publishes today. The Pages source switch (blocker B2) is still needed |
| E8 | The builder order puts generation before decay | Followed for display (stage 2 generation, stage 3 decay). Both arrive together in web Phase 6.1 and can be enabled separately |
| E9 | "Inventory conservation" once decay and generation are on | Shown as a balance: start inventory plus generated, minus decayed, minus released, minus remaining, with the residual ((P21.42)); every term and the residual come from the Rust wrapper (M4). Before Phase 6.1 the balance is the v1.0 one: inventory plus released minus start inventory |

Further rules taken directly from the brief:

- The five-level research status describes a mechanism; the result labels (`COMPUTED v1.0`, `COMPUTED v2.0`, `PRECOMPUTED`, `MEASURED`, `ILLUSTRATIVE`) describe a number. Both stay, and neither replaces the other.
- A status is assigned from evidence in the repository, per mechanism and, where needed, per species and material domain; never from the existence of a page, paper, placeholder or configuration. Newly proposed mechanisms start at PENDING. VALIDATED is not used until a computed model-to-measurement comparison exists (web Phase 7), which owner decision M5 also requires before any agreement figure is written.
- No live-simulation toggle is active unless the solver implements the mechanism and it has reached VERIFIED; a mechanism that needs coupled equations is shown as a coupled system, not inserted into the single transport equation.
- Nothing in bricks 4 to 9 is implemented in this project; they are catalogue entries with status PENDING.

Status of every mechanism on 2026-11-24, assigned from the repository:

| Mechanism | Status | Domain | Basis |
|---|---|---|---|
| Five-layer diffusion, constant diffusivity per layer | VERIFIED | synthetic five-layer case (`inputs/synthetic/synthetic_five_layer.toml`) | v1.0: second-order spatial and first-order temporal convergence, balance residual at most $2.806\times10^{-13}$ of the start inventory, agreement of Code B with the exact series (`prompt/SCIENTIFIC_REQUIREMENTS.md` Section 2) |
| Fission-product generation | RESEARCHED | none run | derived ((P21.1), (P21.8)), pseudocode written, parameters registered; no Rust code |
| Radioactive decay | RESEARCHED | none run | derived ((P21.1), (P21.7), (P21.58)), pseudocode written, half-lives in `data/nuclear/decay.csv`; no Rust code |
| Inventory accounting with decay and source | RESEARCHED | none run | derived ((P21.42), (P21.44)); no Rust code |
| Temperature-dependent diffusivity, layer-specific laws | RESEARCHED | none run | derived ((P22.1), (P22.15), Section 5.4), diffusivity rows and temperature histories registered; no Rust code |
| Interface partition coefficients | PENDING | none | not in the model (rule E1) |
| Activation products, parent-daughter chains | PENDING | none | outlined as (P21.16) and (P21.20) but deferred (D3); the activation cross-section is NOT VERIFIED in `data/nuclear/fission_yields.csv` |
| Every mechanism of bricks 4 to 9 | PENDING | none | nothing derived or registered in the repository |
| Any mechanism | none is VALIDATED | none | no model-to-measurement comparison exists |

Generation, decay and temperature dependence move to IMPLEMENTED and then VERIFIED only when web Phases 6.1 and 6.2 pass their tests and independent review.

Questions for the owner from the brief (none blocks the work): (1) whether genuine Materials Studio screenshots or structure files exist, and with what method and conditions; (2) whether FLiBe or cover-gas material for STRATO exists outside the repository; (3) whether the live simulation may stay on the verified synthetic case until temperature-dependent diffusivity is implemented, with AGR-1 shown as geometry (rule E2 applies until then).

## Pinned external references

| Repository | Commit |
|---|---|
| theodoreOnzGit/outram-park-backend | `0efd0553310db6088052fef8b79be69518913ede` (main, 2026-11-10) |
| weewzr/NUSH2 | `b4817b305de5bfada5da147802cbf3aeffb9f4c1` |
| weewzr/raystorage | `3a25c4a767b67faa3e797944ea9ac63f9265d4a1` |

Checked in Phase 0 (2026-11-24): all three commits exist and every file the package names is present at them. Newer `main` commits seen and not used: outram-park-backend `a4dc9c7` (2026-11-23), NUSH2 `cfef6d5` (2026-11-19); raystorage `main` is still the pinned commit.

## Session log

| Date | Session | Phases worked | Commits | Notes |
|---|---|---|---|---|
| 2026-11-10 | package author | none (instructions only) | `3a4319e`, `bc107b1`, `7fe4421` | package written; deep audit left to Phase 0 |
| 2026-11-24 | Stage B setup | Phase 0 (part) | policy, decisions and baseline commits on `feat/strato-interactive-web-demo` | merge policy M1 adopted; D1 to D16 and M2 to M5 recorded; Phase 6 gate split; Stage A corrections; v1.0 baseline and wasm check pass and match the golden files exactly |
| 2026-11-24 | Stage B, Phase 0 completion | Phase 0 | `91cb327` (`docs(prompt): phase 0 - audit and architecture confirmation`) | public API confirmed; wrapper facts and the wasm panic rule added to the architecture; verified numbers traced to their files (one source correction); cross-project review done and recorded; M6 recorded; full v1.0 suite re-run and unchanged |
| 2026-11-24 | Stage B, Phase 0 review fix | Phase 0 | `d458dad` (`docs(prompt): complete the wrapper rejection list and fix Phase 0 counts`) | wrapper facts table rebuilt from a search of `src/`; panic paragraph corrected (four `expect` calls, wrapper limits, worker containment); `verify` line count corrected; see "Corrections after the Phase 0 review" |
| 2026-11-24 | Stage B, second Phase 0 review fix | Phase 0 | the commit `docs(prompt): rejoin the wrapper facts table and narrow its completeness claim` | table rendering repaired, completeness claim narrowed to the stated search, line 134 check added, `PROGRESS.md` wording and commit records fixed; see "Corrections after the second Phase 0 review" |
| 2026-11-24 | Stage B, equation-first brief | none (instructions only) | `82486f2` (`docs(prompt): record the owner's equation-first brief`) and `docs(prompt): align the package with the equation-first brief` | brief stored verbatim as `prompt/EQUATION_FIRST_REQUEST.md`; recorded as owner policy decision M7 with rules E1 to E9 and the status of every mechanism; design specification (page order A to E, staged builder, five-level status, research catalogue, gallery, reference fuel), architecture (term table with status, catalogue files and check), roadmap (Phase 2 as staged builder, Phase 5.5, brick mapping), testing (status system, no active toggle without solver support, fold at 1440 x 900), scientific requirements (status column, new prohibitions) and the other package files brought into agreement |
