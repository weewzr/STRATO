# Cross-project engineering references: raystorage and NUS-H2

## 1 Purpose and scope

https://github.com/weewzr/raystorage holds the owner's NUS-H2 working files (notes, scripts, JSON and CSV exports, logs). https://github.com/weewzr/NUSH2 is the NUS-H2 product repository, a hydrogen-plant model with a Rust/WebAssembly browser simulator. Neither contains STRATO files.

STRATO borrows **engineering practices only**. The scientific models stay independent: no merged engines, no NUS-H2 process equations, no transferred constants, no NUS-H2 verification result used as evidence for STRATO, no replacement of STRATO's analytical reference solution, and no archive file called validated without checking it.

## 2 Pinned commits and licences

| Repository | Commit | Licence |
|---|---|---|
| weewzr/NUSH2 | `b4817b305de5bfada5da147802cbf3aeffb9f4c1` | MIT (GitHub detection) |
| weewzr/raystorage | `3a25c4a767b67faa3e797944ea9ac63f9265d4a1` | none declared; owner's own archive, reference only |

Read files at these commits (`gh api repos/<repo>/contents/<path>?ref=<sha>`). If you use a newer commit, record it in `prompt/PROGRESS.md`.

## 3 raystorage layout

`NUSH2/README.md` describes the layout `NUSH2/<category>/<origin>/<relative path>` with categories derivations, simulations, process-data, PFD-versions, research-and-references and validation, each split by origin session folder. `NUSH2/MANIFEST.csv` lists every file with size and SHA-256 (about 3,680 rows). The README states that build caches, virtual environments, images, downloaded papers, OUTRAM code and files matching a privacy scan were left out.

Treat the archive as untrusted history: much of it is superseded working material. The NUS-H2 repository is the current source for any pattern; use the archive only to understand how a practice evolved. Do not copy archive files into STRATO.

## 4 Relevant files

| File (repo: path) | Purpose | Status as found | Recommended use |
|---|---|---|---|
| NUSH2: `engine/wasm/README.md` | wasm ABI: zero-import `cdylib`, JSON request and reply, command table, parameter records with hard and model ranges, `rejected` status with violations, measured load and solve times | documented, measured on the owner's machine 2026-11-03 | pattern for input validation, status values and honest performance notes |
| NUSH2: `engine/wasm/src/lib.rs`, `solve.rs`, `registry.rs`, `tests.rs` | wasm entry points, solve dispatch, parameter registry, wrapper tests | current | pattern for a thin wrapper with no physics and native tests of the wrapper |
| NUSH2: `scripts/build_wasm.py`, `scripts/check_wasm.py` | build the module; check exports, zero imports and a 3 MB size limit | current | adopt a size and export check for `crates/strato-wasm` |
| NUSH2: `scripts/wasm_smoke.py` | headless browser timing of first solve | current | pattern for the performance check |
| NUSH2: `frontend/worker.js` | Web Worker hosting the module | current | compare with the Outram Park handshake; pick the handshake design |
| NUSH2: `frontend/selftest.js`, `scripts/selftest.py` | `?selftest=1` browser self-test, verdict in `document.body.dataset.selftest`, read over DevTools; also run with reduced motion; checks the commit stamp | current | adopt (see `prompt/TESTING_AND_VALIDATION.md` Section 4) |
| NUSH2: `.github/workflows/site.yml` | builds wasm, checks it, builds the site, runs the self-test twice (normal and reduced motion) against the expected commit, uploads to Pages | current | model for `.github/workflows/web.yml` |
| NUSH2: `engine/vv/src/golden.rs`, `engine/vv/data/engine_goldens.json`, `engine/vv/tests/plant_goldens.rs` | frozen golden values with one named tolerance; a mismatch means a port error and the code is fixed | current | adopt the named-tolerance rule; STRATO already has goldens in `tests/golden/v1.0/` |
| NUSH2: `math/eq_register.csv` | equation register with columns id, section, title, latex, variables, assumptions, source, engine_fn, test_id, validation | current | adopt the idea as `docs/mathematics/equation_register.csv` for STRATO (Phase 8, optional) |
| NUSH2: `math/*.tex`, `math/sections/*.tex`, `math/generated/values.tex` | LaTeX derivation with generated value macros | current | idea only; STRATO keeps its pandoc and XeLaTeX pipeline (`.github/workflows/docs-pdf.yml`) |
| raystorage: `NUSH2/README.md`, `NUSH2/MANIFEST.csv` | archive description and checksums | archive | read for provenance; a manifest with SHA-256 is worth copying as a practice for `web/public/data/` |
| raystorage: `NUSH2/validation/`, `NUSH2/research-and-references/` (for example `.../m3_golden_cmp.py`) | historical golden comparisons and decision notes | archive, partly superseded | reference only; do not execute |

Phase 0 must open each file above at its pinned commit and correct this table if a path or description is wrong.

## 5 Practices and where they land in STRATO

| Priority | Practice | STRATO phase | Notes |
|---|---|---|---|
| P1 | Thin wasm wrapper with JSON in and out, no physics, `rejected` status with messages | 3 | `prompt/TECHNICAL_ARCHITECTURE.md` Section 3 |
| P1 | Native tests of the wrapper against golden files; browser parity test | 3 | tolerance 1e-12 relative |
| P1 | Worker hosting with a ready handshake, timeouts and recovery after a trap | 3 | combine NUS-H2 and Outram Park patterns |
| P1 | Commit stamp in module and asset URLs; self-test checks the stamp | 3, 8 | defeats Pages caching |
| P1 | `?selftest=1` browser self-test in CI, also under reduced motion | 1 onward | values compared, no physics |
| P2 | Wasm export and size check script | 3 | under 1 MB gzip target |
| P2 | Named tolerance constants, golden mismatch fixes the code | 3 | |
| P2 | Measured performance notes with date and conditions | 4, 8 | never claim unmeasured numbers |
| P2 | Manifest with SHA-256 for precomputed data | 3 | `web/public/data/MANIFEST.csv` |
| P3 | Equation register linking equation id, LaTeX, Rust function and test id | 8 | additive; does not replace the derivations |
| P3 | Generated value macros so documents quote computed numbers | later | only if the owner wants it |

## 6 Numerical verification comparison

| STRATO already has | NUS-H2 practice | Recommendation |
|---|---|---|
| Golden regression `tests/phase1_regression.rs`, relative 1e-12 | goldens with one named tolerance | keep STRATO's suite; add named constants and assert the unasserted golden files (`convergence_study.csv`, stdout files) |
| Code B against Code C (independent analytical reference) | no analytical reference for a plant model | keep; this is STRATO's strongest check and must stay |
| Balance residual with Kahan summation | mass and energy balance checks | keep; add the residual to the web verification report |
| Spatial and temporal convergence study | not applicable | keep; show it from precomputed data |
| Linux CI byte-identical outputs | site build checks commit stamp | add native/wasm parity as a third platform check |

## 7 Incompatible approaches

- NUS-H2 process equations, property correlations, constants and economic models: unrelated physics.
- The NUS-H2 raw ABI (manual `alloc` and `call`): workable, but STRATO uses `wasm-bindgen`, which is simpler for a small API. Either is acceptable; pick one in Phase 3 and record why.
- NUS-H2 LaTeX and Tectonic toolchain: STRATO's documents are Markdown built by pandoc and XeLaTeX; do not switch.
- OUTRAM code in the archive: excluded from raystorage; any Outram Park reuse comes from Theodore's repository under AGPL-3.0-only (see `prompt/GITHUB_REFERENCES.md`).

## 8 Scientific risks

- Mistaking an NUS-H2 tolerance or result for a STRATO result. Every STRATO tolerance must cite a STRATO test.
- Copying a superseded archive script and trusting its output.
- A wasm parity failure hidden by a loose tolerance. Keep 1e-12 and explain any gap.

## 9 Privacy and security

The archive README says a privacy scan removed names, identifiers, course or group text and keys. Still treat every file as possibly sensitive: do not copy archive text into STRATO, do not quote people's names, course codes or group numbers from it, do not execute its scripts, and do not copy any browser profile or temporary storage. Publish nothing from the archive in STRATO.
