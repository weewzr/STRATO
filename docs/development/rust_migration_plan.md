# Rust migration plan (stage R0: inventory and plan)

This document is stage R0 of the Rust-first migration that the owner asked for on 2026-11-25 ([`prompt/RUST_FIRST_REQUEST.md`](../../prompt/RUST_FIRST_REQUEST.md)). It records where the code base stands, classifies every Python, TypeScript, JavaScript, shell and HTML file, names the small independent Python suite that stays, sets the target layout of the Cargo packages and divides the work into stages R1 to R4. Every later migration stage follows it.

Rules that apply to every stage, from the brief:

- Rust becomes the dominant language by computational responsibility. GitHub's language statistics are reported after every stage; they are a measure, not the goal.
- The interface stays React and TypeScript with KaTeX, Plotly and three.js. Only logic that is not rendering moves behind the WebAssembly wrapper.
- A small, genuinely independent Python verification suite stays. No independent reference implementation is ported only to raise the Rust percentage.
- Generated outputs stay byte-identical where required, validated numerical behaviour is preserved, and the regression and conservation tests run at every stage.
- No new Python, shell or Node tooling is written. A new generator, checker, converter or build tool is a Rust program of its own package, called from CI with `cargo run`.
- Nothing is padded, generated only to add Rust, or hidden from the statistics with linguist attributes.

Every byte count below comes from `git ls-tree -r -l` at the commit named; every class comes from reading the file. Section 8 lists the commands.

## 1. Baseline

### 1.1 GitHub's numbers for main

`gh api repos/weewzr/STRATO/languages` reports the default branch. Queried on 2026-11-25 with main at `3e38398`:

| Language | Bytes | Share |
|---|---:|---:|
| Python | 804,228 | 50.7% |
| TypeScript | 577,397 | 36.4% |
| Rust | 133,035 | 8.4% |
| CSS | 63,103 | 4.0% |
| JavaScript | 5,311 | 0.3% |
| HTML | 1,147 | 0.1% |
| Shell | 800 | 0.1% |
| **Total counted** | **1,585,021** | |

### 1.2 The rules GitHub applies, and a check that they are understood

GitHub's linguist counts only programming and markup languages, and leaves out files it classes as documentation, vendored or generated. The patterns that matter for this repository (from linguist's `documentation.yml` and `vendor.yml`):

| Rule | Pattern | Files of this repository it removes from the count |
|---|---|---|
| Documentation | `^[Dd]ocs?/` | `docs/mathematics/phase1/TRISO_Finite_Volume_Derivation.tex` (TeX, 21,149 bytes) and the five pandoc filters `docs/pandoc/*.lua` (Lua, 13,771 bytes) |
| Documentation | `^[Ee]xamples/` (root only) | `examples/verify.rs` (Rust, 8,998 bytes), the verification program that `ci.yml` runs with `cargo run --release --example verify`. `crates/strato-wasm/examples/*.rs` are not at the root and are counted. |
| Documentation | README, LICENSE, CHANGELOG, CONTRIBUTING, CITATION | prose and data files that are not counted in any case |
| Vendored | `(.*?)\.d\.ts$` | `web/src/plots/plotly-basic.d.ts` (TypeScript, 285 bytes; a hand-written module declaration) |
| Vendored | `(^\|/)[Tt]ests?/fixtures/` | `scripts/web/tests/fixtures/catalogue/good/tests/fixture_check.rs` (Rust, 94 bytes; a fixture of the catalogue check) |
| Data languages | JSON, CSV, YAML, TOML, Markdown | all `.json`, `.csv`, `.yml`, `.yaml`, `.toml` and `.md` files, including the committed generated files of the web demo |

`.gitattributes` holds `* text=auto eol=lf` and three `binary` lines (`*.pdf`, `*.png`, `*.pptx`). There is no `linguist-*` attribute, so no override exists.

Applying these rules to the tree of main `3e38398` gives Python 804,228 (all `.py`), TypeScript 577,682 − 285 = 577,397, Rust 142,127 − 8,998 − 94 = 133,035, CSS 63,103, JavaScript (`.js` and `.mjs`) 5,311, HTML 1,147 and Shell 800: all seven numbers equal GitHub's to the byte. The same rules are used for every count in this plan.

### 1.3 Count of the web branch

`origin/feat/strato-interactive-web-demo` at `38769f7` has the research catalogue and is what main will soon be. Counted by the rules of Section 1.2:

| Language | Bytes | Share | Change against main |
|---|---:|---:|---:|
| Python | 824,988 | 47.9% | +20,760 (catalogue tests and the catalogue part of the generator) |
| TypeScript | 672,924 | 39.1% | +95,527 (Research section and its tests) |
| Rust | 133,035 | 7.7% | 0 |
| CSS | 84,739 | 4.9% | +21,636 (`research.css`) |
| JavaScript | 5,311 | 0.3% | 0 |
| HTML | 1,147 | 0.1% | 0 |
| Shell | 800 | 0.0% | 0 |
| **Total counted** | **1,722,944** | | |

Not counted on this commit, as on main: TeX 21,149 and Lua 13,771 (under `docs/`), `examples/verify.rs` 8,998, the `.d.ts` file 285 and the fixture `.rs` 94.

### 1.4 Generated files

The committed generated files are data, not code: `web/src/generated/*.json` (4 files, 884,320 bytes) and `web/public/data/*.json` and `*.csv` (13 files, 870,750 bytes), all written by `scripts/web/generate_data.py`, plus `tests/golden/v1.0/` (the v1.0 golden output, which no stage regenerates). They are JSON and CSV, which linguist does not count, so they do not enter any percentage. Marking the files written by the generator `linguist-generated` would be an honest description (they are generated, and GitHub would then fold them in pull-request diffs), but it would not change the language statistics. The golden files are reference evidence and must not be marked generated. No counted file of the repository is generated output: the WebAssembly package `web/src/solver/pkg` is built in CI and is not committed.

## 2. Inventory

### 2.1 Classes

- **PORT**: computation, generation, checking, validation, conversion or other logic that belongs in Rust.
- **KEEP-INTERFACE**: React components, rendering, styling, interaction, the glue to the wrapper, and tests of rendering.
- **KEEP-PYTHON-REFERENCE**: an independent implementation or check whose value is that it is not written in the solver's language; the table says what it cross-checks.
- **SYMBOLIC**: depends on SymPy. The table says whether a numerical Rust check could prove the same statement. A symbolic identity checked at sample points is tested, not proved, so a symbolic statement stays in Python. "Mixed" marks a file that also has numerical parts; "(split)" in the stage column marks the numerical parts that can become Rust tests at the stage named, while the symbolic parts stay.
- **REMOVE**: dead or superseded, with the evidence.

The stage column says when a PORT or REMOVE happens. "after gate 2.1", "after gate 2.2" and "after chains (D3)" mean that the subject of the check does not exist in Rust yet: the check is ported when the stage that implements Phase 2.1, Phase 2.2 or decay chains in Rust delivers it, as a test of the real solver instead of a second implementation. Until then the Python file stays and keeps running.

### 2.2 Totals by class

All 163 Python, TypeScript, JavaScript, shell and HTML files of `38769f7`:

| Language | PORT | KEEP-INTERFACE | KEEP-PYTHON-REFERENCE | SYMBOLIC | REMOVE | Total |
|---|---:|---:|---:|---:|---:|---:|
| Python (58 files) | 221,412 (19) | - | 169,759 (9) | 423,624 (29) | 10,193 (1) | 824,988 |
| TypeScript (98 files) | 19,707 (5) | 653,502 (93) | - | - | - | 673,209 |
| JavaScript (4 files) | 3,593 (2) | 762 (1) | - | - | 956 (1) | 5,311 |
| Shell (2 files) | - | - | - | - | 800 (2) | 800 |
| HTML (1 file) | - | 1,147 (1) | - | - | - | 1,147 |

The TypeScript total includes the 285-byte `.d.ts` file that GitHub does not count (counted TypeScript: 672,924). Of the KEEP-INTERFACE TypeScript, 415,537 bytes are `web/src` (285 of them the uncounted `.d.ts`), 127,771 bytes unit and component tests (`web/tests`), 105,027 bytes browser tests (`web/e2e`) and 5,167 bytes the Vite and Playwright configuration.

PORT and REMOVE by stage:

| Stage | Python | TypeScript | JavaScript | Shell |
|---|---:|---:|---:|---:|
| R1 | 92,691 PORT (5) | - | 3,593 PORT (2), 956 REMOVE (1) | 800 REMOVE (2) |
| R2 | - | 14,397 PORT (4) | - | - |
| R3 | 18,533 PORT (2), of which 6,465 leave Python in R3 (Section 5.3) | - | - | - |
| R4 | 38,202 PORT (5), 10,193 REMOVE (1, owner's word) | 5,310 PORT (1) | - | - |
| after gate 2.1 | 39,371 PORT (4) | - | - | - |
| after gate 2.2 | 27,462 PORT (2) | - | - | - |
| after chains (D3) | 5,153 PORT (1) | - | - | - |

### 2.3 Every file

#### Python: 58 files, 824,988 bytes

| File | Bytes | Class | Stage | Reason |
|---|---:|---|---|---|
| `scripts/data_register/b6_audit_patch.py` | 10,193 | REMOVE | R4 (owner word) | One-off, idempotent patch of the B6 audit (2026-11-06) that adds the `evidence_class` and `particle_applicability` columns. Both columns are present in the committed CSVs (`evidence_class` in all six register tables, `particle_applicability` in release_measurements.csv), so its work is done; data/reference/B6_audit_log.md and README.md name it as the record of that change. Not ported; retire only with the owner's word (history keeps it). |
| `scripts/data_register/b6_nuclear_crosscheck.py` | 6,113 | PORT | R4 | Data check: downloads IAEA LiveChart decay data and thermal yields and compares them cell by cell with data/nuclear/decay.csv and fission_yields.csv. Manual, network, writes nothing. A Rust tool with an HTTP client does the same. |
| `scripts/data_register/render_register.py` | 12,117 | PORT | R4 | Generator: renders data/reference/STRATO_Parameter_Register.md from register_template.md and the data CSVs. Acceptance: byte-identical register. |
| `scripts/derivation_checks/D5_parameter_register.py` | 22,563 | SYMBOLIC | R4 (split) | Mixed. Checks S, G, F, K, X, A, B, T, E are statements about committed register values; a Rust data check proves them equally (G with exact decimal sums). L (first-order propagation through lambda = ln2/T) and Y (SymPy ODE solution of the daughter-source rule) are symbolic identities; a numerical check at sample points tests, not proves, them, so L and Y stay in Python. |
| `scripts/derivation_checks/b4_decay_source_pseudocode.py` | 35,616 | KEEP-PYTHON-REFERENCE | - | Literal Python transcription of the Phase 2.1 pseudocode blocks A to E (decay constant, step-average source, Backward Euler step with decay and source, inventory and M_out bookkeeping, modal solution), written independently of src/. Becomes the cross-language reference of the Rust implementation of gate 2.1. Its SymPy/mpmath proofs of H21.1 to H21.3 stay with it. |
| `scripts/derivation_checks/b4_phase2_pseudocode_integration.py` | 28,256 | KEEP-PYTHON-REFERENCE | - | Joined v2.0 run in Python (merged step planner landing on every event, ramp plus decay plus switched source with inventory closure, v1.0 run loop reproduced bit for bit in Python floats, option A against effective time). Cross-language reference of the Phase 2.2 Rust run loop. |
| `scripts/derivation_checks/sec1_symbolic_checks.py` | 10,727 | SYMBOLIC | - | SymPy proofs of every Phase 1 formula (items 1 to 14). A numerical Rust check would test instances only. Stays. |
| `scripts/derivation_checks/sec1_synthetic_case.py` | 18,701 | KEEP-PYTHON-REFERENCE | - | Independent eigen-solver of Code C in a different formulation (A sin + B cos in absolute r, 2x2 interface matching), 40-digit mpmath roots, closed-form integrals. Cross-checks the Code C eigenvalues and F(2e5 s) of the v1.0 synthetic case, the Sturm count, orthogonality and the weight sum. |
| `scripts/derivation_checks/sec2_fv_geometry_conductance.py` | 12,825 | SYMBOLIC | - | SymPy: control-volume geometry, interface conductance, Backward Euler truncation, subtraction-free elimination (P1.40 to P1.79). Stays. |
| `scripts/derivation_checks/sec2_fv_tridiagonal_conservation.py` | 14,438 | KEEP-PYTHON-REFERENCE | - | Float64 re-implementation of Code B written operation by operation, plus dense and 50-digit solves. Cross-checks mesh, explicit bound, inventory and final release of the Rust run, conservation to round-off and positivity. |
| `scripts/derivation_checks/sec2_fv_truncation_order.py` | 12,068 | PORT | R3 | NumPy only. Observed space order of the Code B operator (manufactured steady source) and time order of Backward Euler against the semi-discrete eigen-solution. In Rust it runs the real Code B: part B in R3; part A needs a source term, so after gate 2.1. |
| `scripts/derivation_checks/sec3_o4_layered_truncation_bound.py` | 30,354 | SYMBOLIC | after 2.1 (split) | Mixed. Closed-form layer integrals and Green identities are symbolic (stay). The numerical truncation bounds A and B become Rust tests of the Phase 2.1 modal code once it exists. |
| `scripts/derivation_checks/sec3_p21_booth.py` | 8,527 | PORT | after gate 2.1 | Multilayer modal formulas against the Booth single-sphere solutions of IAEA-TECDOC-1674 Eqs (7.6) to (7.8). Becomes a verification test of the Rust Phase 2.1 modal solution. Needs quadrature and the trigamma function in Rust. |
| `scripts/derivation_checks/sec3_p21_chain_overlap.py` | 5,153 | PORT | after chains (D3) | Parent-daughter chain overlap matrix and coupled modal solution. Chains are not in the Phase 2.1 scope, so the port waits for the stage that implements them. |
| `scripts/derivation_checks/sec3_p21_common.py` | 8,269 | KEEP-PYTHON-REFERENCE | - | Shared helper of the Phase 2.1 checks: an independent layered Sturm-Liouville eigen-solver and a conservative theta-method FV solver. Used by sec3_p21_booth, sec3_p21_chain_overlap, sec3_p21_fv_two_layer, b4_decay_source_pseudocode and sec5_o11; stays while any user stays. |
| `scripts/derivation_checks/sec3_p21_decay_source.py` | 7,514 | SYMBOLIC | - | SymPy: decay law, sources, precursors, chains (P21.5 to P21.23). Stays. |
| `scripts/derivation_checks/sec3_p21_fv_two_layer.py` | 7,464 | PORT | after gate 2.1 | Modal solution with source and decay against an FV solver, two-layer sphere. In Rust it compares the Rust modal and FV codes of Phase 2.1. |
| `scripts/derivation_checks/sec3_p21_modal_ode.py` | 9,963 | SYMBOLIC | - | SymPy modal solution, release, closure and limits (P21.24 to P21.49) plus mpmath sums. Stays. |
| `scripts/derivation_checks/sec4_p21_be_closure_order.py` | 14,397 | PORT | after gate 2.1 | Backward Euler with decay and source: per-step and cumulative closure, positivity, decay identity, and N5 (lambda = 0, S = 0 reproduces the v1.0 step bit for bit). Becomes Rust tests of the real solver; N5 becomes a regression gate. |
| `scripts/derivation_checks/sec4_p21_booth_crp6.py` | 8,983 | PORT | after gate 2.1 | FV against Booth solutions; CRP-6 cases 1a/1b printed values 0.4673 and 0.99999959 (TECDOC-1674 Table 10.6) with the two-branch FRG D. Uses mpmath for the Booth series; the Rust test needs f64 with error control. |
| `scripts/derivation_checks/sec4_p21_decay_identity.py` | 7,554 | SYMBOLIC | - | SymPy decay identity and inventory partition (P21.80, P21.81, P21.87). Stays. |
| `scripts/derivation_checks/sec4_p21_matrix_schemes.py` | 7,113 | SYMBOLIC | - | SymPy only (pivots, row excess, fixed points, M_out updates). Stays. |
| `scripts/derivation_checks/sec4_p21_mms_source.py` | 8,483 | SYMBOLIC | after 2.1 (split) | Mixed. M1 and M2 (manufactured solution satisfies the conditions) are symbolic and stay; M3 to M6 (cell averages, observed orders) become Rust tests of the Phase 2.1 solver. |
| `scripts/derivation_checks/sec4_p21_steady_state.py` | 9,547 | SYMBOLIC | - | SymPy exact steady states and a 60-digit 9x9 solve. A float64 Rust solve would check a weaker statement. Stays. |
| `scripts/derivation_checks/sec4_p2_positive_second_order.py` | 35,159 | SYMBOLIC | - | Stability functions, positivity barrier and runs of second-order schemes (CN, TR-BDF2, BDF2, Richardson) that D15 rejected. The symbolic part cannot be proved numerically and the numerical part has no Rust subject (Backward Euler only). Stays whole. |
| `scripts/derivation_checks/sec5_arrhenius.py` | 11,986 | SYMBOLIC | after 2.2 (split) | Mixed. P22.3 to P22.9 symbolic (stay); the numerical handling P22.19 to P22.23 becomes tests of the Rust Arrhenius module at gate 2.2. |
| `scripts/derivation_checks/sec5_effective_time.py` | 9,788 | SYMBOLIC | - | SymPy proof of the effective-time transform and decay factorisation (P22.24 to P22.38). Stays. |
| `scripts/derivation_checks/sec5_fv_shared_alpha.py` | 14,945 | PORT | after gate 2.2 | FV with a temperature ramp against the series in effective time; Backward Euler options; decay factorisation. Becomes a Rust test of the gate 2.2 solver. |
| `scripts/derivation_checks/sec5_o10_stiff_staircase.py` | 21,457 | SYMBOLIC | after 2.2 (split) | Mixed. S0 symbolic (stay); S1 to S8 numerical theorem checks on FV matrices become Rust tests (D7: staircase as a test helper). |
| `scripts/derivation_checks/sec5_o11_parent_daughter_tau.py` | 30,787 | SYMBOLIC | - | Ten symbolic parts plus numerical parts on chains, which are not in scope. Stays. |
| `scripts/derivation_checks/sec5_o12_variable_D_step_limit.py` | 28,708 | SYMBOLIC | after 2.2 (split) | Mixed. S1 to S4 symbolic (stay); N0 to N3 become Rust tests of the F3 to F5 step rule of the planner (D16). |
| `scripts/derivation_checks/sec5_projection_step.py` | 12,517 | PORT | after gate 2.2 | Eigenfunction projection at temperature switches against an FV reference. Becomes a Rust test of the R3 reference accessors (D7). |
| `scripts/derivation_checks/sec5_r24_quadrature_start.py` | 22,560 | SYMBOLIC | after 2.2 (split) | Mixed. C1 and C3 symbolic (stay); C2, C4, C5, C7, C8 numerical (Rust tests of the R2.4 step-3 correction); C6 needs a 30-digit reference and stays. |
| `scripts/derivation_checks/sec5_schedules.py` | 6,465 | PORT | R3 | Input-definition checks of the temperature schedules D5 ships: AGR-1 plan durations 334, 336, 338 h (Collin et al. 2016, p. 380) and CRP-6 totals (IAEA 2012, Table 10.11). No solver needed; needs the exponential integral E_n for part 3. |
| `scripts/derivation_checks/sec5_staircase.py` | 9,111 | SYMBOLIC | - | SymPy Magnus-term order proof plus a Radau IIA reference integration (SciPy). The symbolic part stays, and the Radau reference is independent of the Rust Backward Euler by design. Stays. |
| `scripts/derivation_checks/sec6_pseudocode_checks.py` | 29,773 | KEEP-PYTHON-REFERENCE | - | Literal transcription of the Phase 2 pseudocode blocks B3 to B10 plus SymPy proofs of the helper relations H1 to H6. Cross-language reference of the Phase 2.2 Rust implementation. |
| `scripts/derivation_checks/v1_0_0_inventory_codeB.py` | 10,243 | SYMBOLIC | - | Mixed: SymPy (shell volume, centroid, conductance) plus a NumPy replica of Code B. Stays. |
| `scripts/derivation_checks/v1_0_0_inventory_codeC.py` | 10,518 | SYMBOLIC | - | Mixed: SymPy (transfer, jump, Sturm count, weights) plus a numerical replica of Code C. Stays. |
| `scripts/derivation_checks/v1_0_0_inventory_code_audit.py` | 14,377 | SYMBOLIC | - | Mixed: SymPy derivations plus a float64 re-implementation of each listed formula; both parts get their value from being outside the solver's language. Stays. |
| `scripts/derivation_checks/v1_0_0_p1_geometry.py` | 6,043 | SYMBOLIC | - | SymPy geometry, units, inventory. Stays. Reads the input through a hard-coded absolute path of the author's machine (pre-existing; fix the path in R3 without changing a check). |
| `scripts/derivation_checks/v1_0_0_p2_conservation.py` | 13,701 | SYMBOLIC | - | SymPy Part II (Eqs 2.1 to 2.9). Stays. Same hard-coded input path as v1_0_0_p1_geometry.py. |
| `scripts/derivation_checks/v1_0_0_p3_conditions.py` | 22,918 | SYMBOLIC | - | Mixed SymPy and numerical shooting, Part III. Stays. Same hard-coded input path. |
| `scripts/derivation_checks/v1_0_0_p4_separation.py` | 10,964 | SYMBOLIC | - | SymPy Part IV algebra. Stays. |
| `scripts/derivation_checks/v1_0_0_p4_sturm_weights.py` | 17,445 | KEEP-PYTHON-REFERENCE | - | Second independent Code C eigen-solver (same formulation as sec1_synthetic_case, mpmath 40 digits) plus a float64 replica of the code-specific rules (Sturm count, bracket, panels, flux ratio, floor, validity time). Backs Part IV of the v1.0.0 derivation. Overlaps sec1_synthetic_case. |
| `scripts/derivation_checks/v1_0_0_p5_fv_replica.py` | 9,252 | KEEP-PYTHON-REFERENCE | - | Line-by-line float64 replica of Code B at v1.0.0 that checks the numbers printed in Part V. A Rust port would be the code itself. Overlaps sec2_fv_tridiagonal_conservation. |
| `scripts/derivation_checks/v1_0_0_p5_fv_scheme.py` | 11,973 | SYMBOLIC | - | SymPy plus exact rationals for the floating-point steps of Part V. Stays. |
| `scripts/derivation_checks/v1_0_0_p6_orders.py` | 7,165 | SYMBOLIC | - | Richardson extrapolation, observed orders and the energy estimate from printed v1.0.0 values, with SymPy algebra. Stays. |
| `scripts/derivation_checks/v1_0_0_p6_properties.py` | 12,423 | SYMBOLIC | - | Mixed SymPy and NumPy replica (M-matrix, positivity, maximum principle, stability). Stays. |
| `scripts/derivation_checks/v1_0_0_p6_series.py` | 8,009 | KEEP-PYTHON-REFERENCE | - | Double-precision replica of the Code C series with an independent 40-digit mpmath oracle for every root and weight (one small SymPy step). Checks truncation, validity time and error propagation into F. Overlaps the two Code C references above. |
| `scripts/derivation_checks/v1_0_0_p6_truncation.py` | 7,100 | SYMBOLIC | - | SymPy truncation error of the half-cell face flux and the BE step. Stays. |
| `scripts/figures/v1_0_0_control_volume.py` | 6,803 | PORT | R4 | Matplotlib drawing of Figure 5.1 of the v1.0.0 derivation (illustrative radii). The root crate already depends on plotters. A PNG is not byte-comparable across renderers: acceptance is owner review of the new figure; the committed PNG stays until then. |
| `scripts/figures/v1_0_0_eigenfunctions.py` | 5,011 | PORT | R4 | Figure 4.1: eigenfunctions computed independently of Rust and drawn with matplotlib. A Rust tool would take them from Code C itself (a figure needs no independence). Acceptance as above plus eigenvalues equal to Code C. |
| `scripts/figures/v1_0_0_triso_cross_section.py` | 8,158 | PORT | R4 | Figure 1.1: to-scale TRISO cross section from AGR-1 dimensions (Morris et al. 2016). Drawing only. Acceptance as above. |
| `scripts/web/build_wasm.py` | 6,912 | PORT | R1 | Build tool: checks the wasm-bindgen CLI against Cargo.lock, builds the wrapper, runs wasm-bindgen, checks exports by parsing the wasm module and the 1 MB gzip budget. |
| `scripts/web/check_catalogue.py` | 13,581 | PORT | R1 | Checker: catalogue rules 1 to 9 of TECHNICAL_ARCHITECTURE Section 5.2 over data/research, docs/research, decay.csv and the generated catalogue JSON. |
| `scripts/web/generate_data.py` | 56,404 | PORT | R1 | Generator, six subcommands (case, catalogue with the Markdown-to-blocks conversion, precomputed, colormaps, geometry, figures), each with --check. |
| `scripts/web/tests/test_catalogue_blocks.py` | 9,318 | PORT | R1 | Tests of the catalogue block conversion of generate_data.py; move with it as Rust tests. |
| `scripts/web/tests/test_check_catalogue.py` | 6,476 | PORT | R1 | Fixture tests of check_catalogue.py (good fixture passes, every bad fixture fails with its message); move with it. |

#### TypeScript: web/src: 66 files, 427,036 bytes

| File | Bytes | Class | Stage | Reason |
|---|---:|---|---|---|
| `web/src/App.tsx` | 3,969 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/build.ts` | 585 | KEEP-INTERFACE | - | Build stamp. |
| `web/src/data/case.ts` | 2,300 | KEEP-INTERFACE | - | Typed import of the generated case JSON. |
| `web/src/design/layers.ts` | 949 | KEEP-INTERFACE | - | Colour token of each layer. |
| `web/src/downloads/serialise.ts` | 7,159 | PORT | R2 | Export formatting of the downloads (CSV and JSON headers, shortest round-trip number text, units, column assembly) and the matching of series values to run times (seriesAtRunTimes). Not returned by the wrapper today. |
| `web/src/equation/EquationBuilder.tsx` | 24,144 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/equation/TermExplanations.tsx` | 3,589 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/equation/Tex.tsx` | 1,746 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/equation/availability.ts` | 2,406 | KEEP-INTERFACE | - | Lookup: capability reported by build_info AND catalogue status VERIFIED. UI gating; the wrapper's own check_mechanisms already refuses an unsupported mechanism, which is the rule that counts. |
| `web/src/equation/links.ts` | 1,390 | KEEP-INTERFACE | - | Links to documents and code anchors. |
| `web/src/equation/terms.ts` | 33,196 | KEEP-INTERFACE | - | Term and stage table of the equation builder: LaTeX strings with their sources; nothing is evaluated. |
| `web/src/format.ts` | 2,912 | KEEP-INTERFACE | - | Display formatting (notation, digit groups, readable duration s to min/h/d). Decides only how a returned number looks on screen. |
| `web/src/main.tsx` | 1,427 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/panels/Badge.tsx` | 1,300 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/panels/ControlPanel.tsx` | 14,799 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/panels/Downloads.tsx` | 7,432 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/panels/Header.tsx` | 2,549 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/panels/NumberField.tsx` | 2,656 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/panels/Placeholder.tsx` | 1,080 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/panels/Playback.tsx` | 4,457 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/panels/Provenance.tsx` | 9,227 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/panels/SimResults.tsx` | 14,559 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/panels/StatusLine.tsx` | 1,757 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/panels/Verification.tsx` | 12,627 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/particle/CrossSection2D.tsx` | 6,370 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/particle/Particle3D.tsx` | 14,555 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/particle/ParticleView.tsx` | 15,880 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/particle/ReferenceParticle.tsx` | 6,100 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/particle/colormap.ts` | 4,702 | KEEP-INTERFACE | - | Colour mapping of returned concentrations (scale position, log10, table index, cell lookup for the 3D cut faces). Display mapping evaluated per frame; the tables are generated data. |
| `web/src/particle/geometry.ts` | 2,276 | KEEP-INTERFACE | - | SVG drawing geometry of the cross-section (display scaling). |
| `web/src/particle/playback.ts` | 2,325 | KEEP-INTERFACE | - | Playback clock over the returned output rows; no interpolation. |
| `web/src/particle/webgl.ts` | 661 | KEEP-INTERFACE | - | WebGL availability probe. |
| `web/src/plots/Plot.tsx` | 7,563 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/plots/SimPlots.tsx` | 12,094 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/plots/figures.ts` | 19,732 | KEEP-INTERFACE | - | Plotly traces and layouts; arrays are the returned arrays unchanged; log10 only for axis ranges and label positions (lines 56, 226, 368). |
| `web/src/plots/plotly-basic.d.ts` | 285 | KEEP-INTERFACE | - | Hand-written module declaration. Linguist counts every .d.ts file as vendored, so it is not in GitHub's statistics. |
| `web/src/plots/plotly.ts` | 2,926 | KEEP-INTERFACE | - | Lazy loading and typing of Plotly. |
| `web/src/plots/precomputed.ts` | 1,418 | KEEP-INTERFACE | - | Fetch of the precomputed files. |
| `web/src/plots/theme.ts` | 3,125 | KEEP-INTERFACE | - | Plot colours from the design tokens. |
| `web/src/research/Blocks.tsx` | 6,991 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/research/FutureResearch.tsx` | 6,272 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/research/StatusChip.tsx` | 2,080 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/research/catalogue.ts` | 5,159 | KEEP-INTERFACE | - | Typed access to the generated catalogue JSON. |
| `web/src/research/entries.ts` | 7,879 | KEEP-INTERFACE | - | Selection and grouping over the generated catalogue. Three data-shaping places move into the Rust generator in R1/R2: roadmapSummary (line 160, number parsed from a table cell), linkCheckedDate (line 172, date check of a cell), gallerySpecies (line 177, splits a list cell the generator does not split). |
| `web/src/research/parts.tsx` | 14,540 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/research/status.ts` | 2,184 | KEEP-INTERFACE | - | The five status words and their definitions (vocabulary of the interface). |
| `web/src/route.ts` | 2,522 | KEEP-INTERFACE | - | Hash routing. |
| `web/src/sections/About.tsx` | 9,025 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/sections/Compare.tsx` | 9,531 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/sections/Data.tsx` | 5,804 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/sections/Explore.tsx` | 9,072 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/sections/Research.tsx` | 19,462 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/sections/Validate.tsx` | 2,983 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/sections/Verify.tsx` | 5,697 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/sections/common.tsx` | 1,479 | KEEP-INTERFACE | - | React component: rendering and interaction. |
| `web/src/selftest/selftest.ts` | 14,592 | KEEP-INTERFACE | - | Browser self-test harness (DOM, theme, parity verdict). |
| `web/src/solver/client.ts` | 7,690 | KEEP-INTERFACE | - | Worker protocol and session glue to the wrapper (requests, time-outs, debounce). |
| `web/src/solver/compare.ts` | 2,115 | KEEP-INTERFACE | - | Value comparison of the browser parity check and the self-test. Kept outside the module it checks: moving it into the wrapper would let the WebAssembly build check itself. |
| `web/src/solver/protocol.ts` | 1,301 | KEEP-INTERFACE | - | Worker protocol and session glue to the wrapper (requests, time-outs, debounce). |
| `web/src/solver/session.ts` | 4,859 | KEEP-INTERFACE | - | Worker protocol and session glue to the wrapper (requests, time-outs, debounce). |
| `web/src/solver/tolerances.ts` | 884 | PORT | R2 | PARITY_RTOL and GOLDEN_RTOL duplicate crates/strato-wasm/src/tolerances.rs; build_info can return them. The two time constants stay in TypeScript. |
| `web/src/solver/types.ts` | 4,251 | KEEP-INTERFACE | - | TypeScript types of the wrapper replies (could be generated from the Rust types later; still TypeScript). |
| `web/src/solver/worker.ts` | 2,654 | KEEP-INTERFACE | - | Worker protocol and session glue to the wrapper (requests, time-outs, debounce). |
| `web/src/state/store.ts` | 14,177 | KEEP-INTERFACE | - | Zustand UI state; inputKey (line 142, FNV hash of the config for staleness) and configDifferences (line 323) are UI state, not results. |
| `web/src/state/validation.ts` | 3,456 | PORT | R2 | validateInputs repeats the rules of Config::validate plus c0 > 0 with per-field messages; parseNumber parses typed numbers. The wrapper's validate_config checks the same rules but returns one coarse message per region, so it needs per-field issues first. |
| `web/src/theme.ts` | 2,150 | KEEP-INTERFACE | - | Light and dark theme switch. |

#### TypeScript: web/tests (unit and component tests): 17 files, 135,979 bytes

| File | Bytes | Class | Stage | Reason |
|---|---:|---|---|---|
| `web/tests/builder.test.tsx` | 14,176 | KEEP-INTERFACE | - | Unit or component test of the interface. |
| `web/tests/catalogueFixture.ts` | 665 | KEEP-INTERFACE | - | Test fixture: generated catalogue with replaced statuses. |
| `web/tests/client.test.ts` | 6,836 | KEEP-INTERFACE | - | Unit or component test of the interface. |
| `web/tests/components.test.tsx` | 10,095 | KEEP-INTERFACE | - | Unit or component test of the interface. |
| `web/tests/contrast.test.ts` | 4,256 | KEEP-INTERFACE | - | WCAG contrast of the design tokens (design check of the styling). |
| `web/tests/format.test.ts` | 1,084 | KEEP-INTERFACE | - | Tests of format.ts, which stays. |
| `web/tests/guards.test.ts` | 5,310 | PORT | R4 | Repository guard over the web sources (no hand-typed scientific values, provenance headers, shared token file unchanged, banned text). A checker, not a rendering test; optional move to the Rust web-tools crate. |
| `web/tests/palette.test.ts` | 5,481 | KEEP-INTERFACE | - | Colour-vision check of the layer colours (design check of the styling). |
| `web/tests/particle.test.tsx` | 10,918 | KEEP-INTERFACE | - | Unit or component test of the interface. |
| `web/tests/plots.test.tsx` | 18,362 | KEEP-INTERFACE | - | Unit or component test of the interface. |
| `web/tests/research.test.tsx` | 22,539 | KEEP-INTERFACE | - | Unit or component test of the interface. |
| `web/tests/results.test.tsx` | 6,249 | KEEP-INTERFACE | - | Unit or component test of the interface. |
| `web/tests/setup.ts` | 1,868 | KEEP-INTERFACE | - | Test setup: records the figure handed to Plotly. |
| `web/tests/status.test.tsx` | 7,091 | KEEP-INTERFACE | - | Unit or component test of the interface. |
| `web/tests/store.test.ts` | 4,684 | KEEP-INTERFACE | - | Unit or component test of the interface. |
| `web/tests/terms.test.ts` | 13,467 | KEEP-INTERFACE | - | Builder terms: LaTeX copies against the documents and KaTeX rendering; tests terms.ts, which stays. |
| `web/tests/validation.test.ts` | 2,898 | PORT | R2 | Tests of validation.ts; move to Rust unit tests of the wrapper, leaving a test of the call. |

#### TypeScript: web/e2e (browser tests): 13 files, 105,027 bytes

| File | Bytes | Class | Stage | Reason |
|---|---:|---|---|---|
| `web/e2e/behaviour.spec.ts` | 4,918 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |
| `web/e2e/builder.spec.ts` | 13,438 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |
| `web/e2e/explore.spec.ts` | 6,919 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |
| `web/e2e/fixtures.ts` | 2,814 | KEEP-INTERFACE | - | Shared widths and helpers of the browser tests. |
| `web/e2e/keyboard.spec.ts` | 6,127 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |
| `web/e2e/layout.spec.ts` | 3,263 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |
| `web/e2e/limits.spec.ts` | 4,342 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |
| `web/e2e/particle.spec.ts` | 20,960 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |
| `web/e2e/plots.spec.ts` | 18,515 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |
| `web/e2e/research.spec.ts` | 7,338 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |
| `web/e2e/screenshots.spec.ts` | 2,300 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |
| `web/e2e/selftest.spec.ts` | 1,673 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |
| `web/e2e/solver.spec.ts` | 12,420 | KEEP-INTERFACE | - | Browser test (Playwright) of the running build. |

#### TypeScript configuration, JavaScript, shell and HTML: 9 files, 12,425 bytes

| File | Bytes | Class | Stage | Reason |
|---|---:|---|---|---|
| `scripts/web/build_wasm.sh` | 334 | REMOVE | R1 | Two-line wrapper that execs build_wasm.py; superseded by a cargo run call in web.yml (the name is cited in prompt/TECHNICAL_ARCHITECTURE.md Section 3, which R1 updates). |
| `scripts/web/precompute.sh` | 466 | REMOVE | R1 | Two-line wrapper of generate_data.py precomputed; superseded the same way. |
| `web/eslint.config.js` | 762 | KEEP-INTERFACE | - | Lint configuration of the interface. |
| `web/index.html` | 1,147 | KEEP-INTERFACE | - | Page shell. |
| `web/playwright.config.ts` | 1,599 | KEEP-INTERFACE | - | Browser-test configuration. |
| `web/scripts/check_catalogue.mjs` | 956 | REMOVE | R1 | Node launcher that finds a Python interpreter and runs check_catalogue.py before the build; superseded when package.json calls the Rust checker. |
| `web/scripts/check_content.mjs` | 2,798 | PORT | R1 | Content check of the built site (no prompt/ file or link, no internal-comparison sentence or agreement figure, no local absolute path). A checker, so it belongs in the Rust web-tools crate. |
| `web/scripts/check_wasm_pkg.mjs` | 795 | PORT | R1 | Fails the build early when src/solver/pkg is missing. Folds into the Rust build tool's check. |
| `web/vite.config.ts` | 3,568 | KEEP-INTERFACE | - | Vite build configuration with the build-stamp and gallery plugins. |

### 2.4 Logic in the TypeScript that is not rendering

`web/src` was searched for arithmetic on results, unit conversion, interpolation, formatting of exported files, input validation, parsing, data shaping and colour mapping of values (patterns `Math.*`, `toPrecision`, `toExponential`, `toFixed`, `parseFloat`, `Number(`, `JSON.*`, `split(`, and a reading of every `.ts` module). Every place found:

| Place | What it does | Does the wrapper return it? | Plan |
|---|---|---|---|
| `downloads/serialise.ts:29` `numberText` | shortest round-trip text of a double, `-0` kept | No (the wrapper returns numbers in JSON, not export text) | R2: Rust, formatted exactly as ECMAScript `Number.prototype.toString` |
| `downloads/serialise.ts:35-57` `exportMeta` | metadata header of a download (source, solver and page commits, case file, UTC time) | No | R2; the time is passed in from the page, the module has no clock |
| `downloads/serialise.ts:62-75` `csvText`, `jsonText` | export formatting of CSV (with `# key: value` header and units line) and JSON | No | R2 |
| `downloads/serialise.ts:107-116` `seriesAtRunTimes` | data shaping: Code C values matched to the Code B output times by equal time | No: `run_v1` returns the series on its own times | R2 |
| `downloads/serialise.ts:118-163` units table and column assembly | the units of every exported column, columns of time series, profiles and comparison | No (the reply carries no units) | R2 |
| `downloads/serialise.ts:84-104` `parseCsv` | reads a download back, for tests and the end-to-end download check | - | stays in TypeScript as a test helper, so that the round-trip check reads the Rust output with code other than the code that wrote it |
| `state/validation.ts:19-65` `validateInputs` | input validation with a message per field; mirrors `Config::validate` (`src/config.rs` lines 79 to 113) and adds c0 > 0 | Partly: `validate_config` applies the same rules but answers with one coarse message per region ("invalid region ...") | R2: a field-level validation function in the wrapper |
| `state/validation.ts:68-73` `parseNumber` | parses a typed number (plain, scientific, "x10^" and the minus sign) | No | R2 |
| `solver/tolerances.ts:7,13` | `PARITY_RTOL`, `GOLDEN_RTOL` duplicate `crates/strato-wasm/src/tolerances.rs` | No (not in `build_info`) | R2: returned by `build_info`; the two time constants (time-out, debounce) stay |
| `research/entries.ts:160-169` `roadmapSummary` | parses a brick number out of a table cell of `roadmap.md` | No (the generator writes the table as blocks) | R1, after the byte-identity step: the Rust generator writes the summary as data (a reviewed format change of the catalogue JSON) |
| `research/entries.ts:172` `linkCheckedDate` | date check of a `link_checked` cell | No | R1: the Rust checker validates the cell; the lookup stays |
| `research/entries.ts:177-182` `gallerySpecies` | splits the `species` cell of `gallery.csv` | No: the generator's list columns do not include that column | R1, after the byte-identity step: the generator splits it (same format change) |
| `panels/ControlPanel.tsx:25,42` | log10 slider position of D and three significant digits of the value set by the slider | - | stays: a property of the control, and the typed value is not rounded |
| `format.ts:5-54` | display notation, digit groups, readable duration (s to min, h, d) | - | stays: decides only how a returned number looks |
| `particle/colormap.ts:33-88` | colour-scale position (linear or six-decade log10), table index, cell lookup for the 3D cut faces | - | stays: display mapping evaluated per frame from generated tables |
| `particle/geometry.ts:25-54`, `plots/figures.ts:56,226,368`, `particle/ParticleView.tsx:66`, `particle/CrossSection2D.tsx:58-68` | drawing geometry, axis ranges and label positions | - | stays: layout |
| `particle/playback.ts:23-24`, `state/store.ts:250,274,299` | playback row from wall time, row clamping | - | stays: interaction |
| `state/store.ts:142` `inputKey`, `:323` `configDifferences` | hash of the inputs for staleness; list of changed inputs for the comparison panel | - | stays: UI state |
| `solver/compare.ts:25-49` | value comparison of the browser parity check and the self-test | - | stays: moving it into the module it checks would let the WebAssembly build check itself |
| `panels/SimResults.tsx:88,98`, `sections/Compare.tsx:80,84`, `plots/SimPlots.tsx:146` | six significant digits of returned values | - | stays: display |
| `panels/Verification.tsx:89,152-153` | reads a number and two comma lists out of the precomputed summary | No: the summary is the CLI's standard output, copied as text on purpose | stays: display of copied text; changing it would change the precomputed format that R1 keeps byte-identical |

No place computes a physical quantity from returned values: the owner rule that nothing is subtracted or computed in TypeScript holds, and the pointwise difference of the two analytical codes is shown only from the precomputed native file.

## 3. The independent Python suite that stays

The suite cross-checks the Rust solver from outside its language. Its members are chosen because each one is an implementation written from the equations, not from the Rust code, that would lose its point if it were rewritten in Rust: a Rust version would share the compiler, the standard library's mathematical functions and very likely the author's reading of the Rust code, so an error common to both would no longer show.

Core suite (6 files, 135,053 bytes):

| File | What it cross-checks | What would be lost if it were ported |
|---|---|---|
| `sec1_synthetic_case.py` | Code C (`src/analytical/sturm_liouville.rs`): eigenvalues, F(2e5 s), Sturm count, orthogonality, weights, from a different formulation (A sin + B cos in absolute r) with 40-digit mpmath roots | the only check of Code C that uses other arithmetic (40 digits) and another formulation |
| `sec2_fv_tridiagonal_conservation.py` | Code B (`src/numerical/finite_volume.rs`): mesh, inventory, final release, positivity and conservation, against dense and 50-digit solves | the check of the subtraction-free elimination against an exact solve outside f64 |
| `sec3_p21_common.py` | shared independent multilayer eigen-solver and FV solver of the Phase 2.1 checks | the reference solvers of the Phase 2.1 checks |
| `b4_decay_source_pseudocode.py` | Phase 2.1 pseudocode blocks A to E; the reference for the Rust implementation of gate 2.1 | an independent second implementation of decay, source and inventory before the Rust one exists |
| `b4_phase2_pseudocode_integration.py` | the joined v2.0 run (planner events, closure, v1.0 run loop bit for bit) | the same for the Phase 2.2 run loop |
| `sec6_pseudocode_checks.py` | Phase 2 pseudocode blocks B3 to B10; the reference for Phase 2.2 | the same for Arrhenius, temperature history, effective time and projection |

Three further files are cross-language references of the same v1.0 code: `v1_0_0_p4_sturm_weights.py` and `v1_0_0_p6_series.py` (Code C, overlapping `sec1_synthetic_case.py`) and `v1_0_0_p5_fv_replica.py` (Code B, overlapping `sec2_fv_tridiagonal_conservation.py`), 34,706 bytes together. They back numbers printed in the v1.0.0 derivation document. They are never ported (a Rust replica of Rust code proves nothing); whether they stay beside the core suite or are retired is an owner question (Section 7).

The 29 SYMBOLIC files (423,624 bytes) also stay, for a different reason: SymPy proves identities that a numerical check can only sample. They are not part of the cross-language suite and are not counted in it.

Two facts the later stages must handle:

- None of the 49 files of `scripts/derivation_checks` runs in CI today. They run in a virtual environment from `requirements.lock.txt` (SymPy 1.14.0, mpmath 1.3.0, NumPy 2.2.6, SciPy 1.15.3). R3 adds one CI job that runs the core suite and the SYMBOLIC files on every pull request, so that the suite keeps its value while the Rust code changes. A CI job is workflow YAML, not new Python.
- `v1_0_0_p1_geometry.py` (line 11), `v1_0_0_p2_conservation.py` (line 218) and `v1_0_0_p3_conditions.py` (line 57) read the synthetic input through a hard-coded absolute path of the author's machine, so they cannot run in CI as they are. R3 replaces the path by one relative to the script, changing no check.

## 4. Target architecture

### 4.1 Conventions already in the repository

- The verified package `strato` is the root crate (`Cargo.toml`, library `src/lib.rs`, binary `src/main.rs`, edition 2021, licence `AGPL-3.0-only`). It is not a workspace.
- `crates/strato-wasm` is a package of its own with an empty `[workspace]` table and its own `Cargo.lock`, "so the verified package `strato`, its lock file and its CI stay byte for byte as they are" (its `Cargo.toml`). It depends on `strato = { path = "../.." }`, is `publish = false`, and CI calls it with `--manifest-path crates/strato-wasm/Cargo.toml`.
- There is no `rust-toolchain` file. CI uses `dtolnay/rust-toolchain@stable`; the wasm-bindgen CLI is pinned to 0.2.130 and checked against `crates/strato-wasm/Cargo.lock`.

### 4.2 Layout after the migration

The root package stays untouched. The new tools live in one tools workspace under `crates/`, with its own lock file, so the verified package and the wrapper keep theirs:

```text
Cargo.toml, Cargo.lock          strato (unchanged): Code A, B, C; CLI; examples/verify.rs
crates/strato-wasm/             wrapper (own workspace and lock, as now); gains the web logic of R2
crates/Cargo.toml, Cargo.lock   tools workspace: members below; exclude = ["strato-wasm"]
crates/strato-webtools/         R1: library + binary `strato-webtools`
crates/strato-checks/           R3: library + binary `strato-checks`, and integration tests
crates/strato-data/             R4: library + binary `strato-data`
crates/strato-figures/          R4: binary `strato-figures` (plotters, as the root crate already uses)
```

| Package | Kind | Owns |
|---|---|---|
| `strato-webtools` | library and command-line tool | `generate {case, catalogue, precomputed, colormaps, geometry, figures} [--check]`, `check-catalogue [--root]`, `check-content <dist>`, `build-wasm [--check]` (CLI version against the lock, cargo build, wasm-bindgen, export list, gzip budget). The library holds the catalogue table definitions shared by generator and checker, the Markdown-to-blocks conversion and a formatter that writes floats exactly as Python's `repr` and `json.dumps` did. The catalogue fixtures move to `crates/strato-webtools/tests/fixtures/` (still a `tests/fixtures/` path, still vendored for linguist). |
| `strato-checks` | library, command-line tool and tests | numerical checks of derivation statements that have a Rust subject (R3 now; the "after gate" ports later). It depends on `strato` by path and runs the real Code B and Code C; it is not a second solver. |
| `strato-data` | library and command-line tool | `render-register` (byte-identical `STRATO_Parameter_Register.md`), `check-register` (the data checks of `D5_parameter_register.py`), `crosscheck-nuclear` (manual, network). |
| `strato-figures` | command-line tool | the three figures of the v1.0.0 derivation. |
| `strato-wasm` | `cdylib` + `rlib` (as now) | after R2 also: field-level input validation, typed-number parsing, export files (CSV and JSON text of the downloads), the tolerances in `build_info`. The new functions sit in a module of their own (`src/web_logic.rs`), are plain Rust functions tested natively, and are exported through `#[wasm_bindgen]` with the same JSON-in, JSON-out convention as `run_v1`. |

Every new package: edition 2021, `license = "AGPL-3.0-only"`, `repository = "https://github.com/weewzr/STRATO"`, `publish = false`, no `build.rs` unless needed.

### 4.3 How CI calls them

- `web.yml` replaces each `python3 -I -X utf8 scripts/web/...` step by `cargo run --locked --release --manifest-path crates/Cargo.toml -p strato-webtools -- <subcommand>`, and the two `sh scripts/web/*.sh` steps by the same tool. The existing `deploy` job is kept and not duplicated.
- `web/package.json`: the `build` script calls the Rust checker (`cargo run --locked --manifest-path ../crates/Cargo.toml -p strato-webtools -- check-catalogue`) instead of `node scripts/check_catalogue.mjs`, and `check:content` calls `check-content dist`.
- `ci.yml` gains a job for the tools workspace: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`, each with `--manifest-path crates/Cargo.toml`. The `strato` job and the `audit` job stay as they are, and `cargo audit` is also run on `crates/Cargo.lock`.
- From R3, one job runs the independent Python suite (Section 3).

### 4.4 How the wrapper exposes the moved web logic

R2 adds exported functions next to the six of today (`abi_version`, `build_info`, `validate_config`, `run_v1`, `analytical_release`, `compare_v1`), for example `validate_inputs(config_json) -> {issues: [{path, message}]}`, `parse_number(text) -> number | null` and `export_text(kind, format, reply_json, meta_json) -> text`, raises `abi_version`, and adds the tolerances to `build_info`. The TypeScript keeps only the calls. The export list in `prompt/TECHNICAL_ARCHITECTURE.md` Section 3 and in the build tool's export check are updated together in R2. Because the wrapper runs in the worker, validation becomes an asynchronous request, debounced like a run (`RUN_DEBOUNCE_MS`); R2 records the decision and measures the delay.

## 5. Stages

Every stage is its own pull request, implemented, independently verified and then pushed to main by the coordinator. The gates that run in every stage, in addition to the stage's own acceptance tests:

- **v1.0 regression and golden comparison**: `cargo test --all-targets` (including `tests/phase1_regression.rs`), `cargo run --release --example verify` with `FV_VERIFICATION PASS`, and the precomputed check, whose `golden_comparison` compares the four CSV files of the native run with `tests/golden/v1.0/` byte for byte and its standard output after normalising path separators.
- **Inventory-balance and conservation checks**: the balance residual of the verify program (`CONSERVATION cells=200 ... balance_rel=2.806e-13` in `tests/golden/v1.0/verify_stdout.txt`, level `V1_BALANCE_REL_LEVEL`) and the layer-sum checks of the wrapper tests.
- **Native-versus-WebAssembly parity**: `cargo test --locked --manifest-path crates/strato-wasm/Cargo.toml` and the browser parity tests and self-test, at 1e-12 per value, unchanged.
- **Full web gate**: lint, type check, unit tests, build, content check, Playwright (including the frame-rate project), with the deploy job unchanged.
- **Language report**: GitHub's numbers after the push, and the count by the rules of Section 1.2 on the branch before it.

Expected percentages assume a Rust/source byte ratio k. No code has been ported in this repository yet (`git log --all --diff-filter=D` finds no deleted Python, TypeScript, shell or JavaScript file), so there is no observed ratio. The closest same-purpose pair is the v1.0 input rules: about 1,308 bytes of Rust in `Config::validate` against about 2,348 bytes of TypeScript in `validateInputs`, but the TypeScript writes a message per field, so the pair is not like for like. The plan therefore assumes k = 1.3, with 1.0 and 1.6 as the band: Rust tools carry explicit types, error handling and serde structures, the byte-identical output needs a hand-written Python-compatible float formatter, and the tests travel with the code; against that, crates such as `clap`, `csv` and `serde_json` replace code that Python's standard library also provides. The arithmetic below uses k = 1.3; Section 6 gives the band. Bytes removed are measured; bytes added are estimates.

### 5.1 R1: web tooling

**Moves**: `scripts/web/generate_data.py` (56,404), `check_catalogue.py` (13,581), `build_wasm.py` (6,912), `tests/test_catalogue_blocks.py` (9,318), `tests/test_check_catalogue.py` (6,476) and their fixtures; `web/scripts/check_content.mjs` (2,798) and `check_wasm_pkg.mjs` (795) into `strato-webtools`. Removed without a port: `scripts/web/build_wasm.sh` (334), `precompute.sh` (466), `web/scripts/check_catalogue.mjs` (956). After the byte-identity step, the data shaping of `research/entries.ts` (Section 2.4) moves into the generator's output in a separate, reviewed commit that raises the catalogue's schema number.

**Acceptance**:

1. Byte identity of every generated file. In the R1 pull request, before the Python files are deleted, one CI step runs the Python generator and the Rust generator on the same checkout into two temporary trees and compares every output byte for byte: `web/src/generated/` (`synthetic_five_layer.case.json`, `research_catalogue.json`, `colormaps.json`, `agr1_reference.json`) and `web/public/data/` (13 files). Allowed differences are only the header fields that name the program that wrote the file and record when: `generator`/`converter` (today `scripts/web/generate_data.py v1`), `command` (today `python -I -X utf8 ...`), `generator_sha256` and `created_utc`. A Rust test in `strato-webtools` checks that the two trees differ in nothing else. Then one regeneration commit writes the committed files with the new generator name; its diff must touch only those header lines.
2. `--check` of each subcommand passes on the committed tree with the same normalisation as today (`created_utc`; `commit` for the catalogue and the precomputed files; `rustc` and `engine.commit` for the precomputed files; `commit` and `generator_sha256` for the colour tables, the geometry and the guides).
3. The catalogue checker prints the same PASS, FAIL and SKIP lines as the Python checker on the repository, on the good fixture and on each of the 15 bad fixtures; the block-conversion tests pass in Rust on every entry file.
4. `build-wasm` produces a `strato_wasm_bg.wasm` with the same SHA-256 as `build_wasm.py` in the same job, the same export list and the same gzip-size result.
5. The precomputed check still compares the native run with `tests/golden/v1.0/`, and all gates of Section 5 pass.

**Risks**: Python's float text (`repr`, `json.dumps`: `1e-05`, `2e-15`, `200000.0`) differs from what `serde_json` writes (`1e-5`) and from Rust's `Display` (`0.00001`); key order and `ensure_ascii=False` must be kept (`serde_json` with `preserve_order`); Python's `csv` quoting and line endings; the Markdown-to-blocks conversion uses regex lookarounds (`generate_data.py` lines 403 and 407), which the `regex` crate does not support (a hand-written scanner or `fancy-regex`); `decimal` sums of the geometry (line 1110 onwards) need exact decimal arithmetic; Python's `str.strip` and `splitlines` treat more characters as white space than ASCII. The generator's name is asserted by `web/tests/guards.test.ts` lines 63 and 68 and `web/tests/status.test.tsx` line 132 and appears in page text (`sections/Data.tsx` lines 91 and 99, `panels/Verification.tsx` line 157): R1 updates these to the new name exactly as strictly (an identity string is changed, no assertion is loosened). Documents that cite the script names (`prompt/TECHNICAL_ARCHITECTURE.md`, `docs/development/provenance.md`, `README.md`) are updated in the same pull request. `npm run build` then needs cargo, as CI already has.

**Expected percentages** (counted bytes; Rust added = 1.3 × (92,691 + 3,593) = 125,169):

| | Python | TypeScript | Rust | CSS | JS | HTML | Shell | Total |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| before (38769f7) | 824,988 | 672,924 | 133,035 | 84,739 | 5,311 | 1,147 | 800 | 1,722,944 |
| change | −92,691 | 0 | +125,169 | 0 | −4,549 | 0 | −800 | |
| after R1 | 732,297 (41.8%) | 672,924 (38.5%) | 258,204 (14.8%) | 84,739 (4.8%) | 762 (0.0%) | 1,147 (0.1%) | 0 | 1,750,073 |

### 5.2 R2: non-rendering web logic behind the wrapper

**Moves**: `web/src/downloads/serialise.ts` (7,159; `parseCsv` stays as a test helper), `web/src/state/validation.ts` (3,456), the two tolerances of `web/src/solver/tolerances.ts` (884), and `web/tests/validation.test.ts` (2,898) as Rust unit tests of the wrapper.

**Acceptance**:

1. Downloads byte-identical: for the fixtures of `web/tests/plots.test.tsx` and `results.test.tsx` (the native wrapper response) every CSV and JSON download text from Rust equals the text the TypeScript writes today, captured before the change and committed as test data; a browser test compares a download of the live page with the same text.
2. Number text: Rust's formatting equals ECMAScript `String(v)` for the values of the fixtures and for a large set of random doubles, checked in a TypeScript test against the browser's own `String`.
3. Validation: for a corpus of valid and invalid configurations (every rule, every field, boundary values, NaN and infinities) the field-level function rejects exactly what `Config::validate` plus the c0 > 0 rule rejects, and with the same per-field messages as the TypeScript today.
4. Native-versus-WebAssembly parity of the new functions (native unit tests and the browser self-test), and all gates of Section 5.

**Risks**: ECMAScript number-to-text rules (exponent form from 1e21 and below 1e-6) differ from Rust's; the validation round trip to the worker; a larger module (the 1 MB gzip budget is checked by the build tool); an ABI change that the page and module must make together.

**Expected percentages** (Rust added = 1.3 × 14,397 = 18,716): Python 732,297 (41.7%), TypeScript 658,527 (37.5%), Rust 276,920 (15.8%), CSS 84,739 (4.8%), total 1,754,392.

### 5.3 R3: numerical tooling and the derivation checks classed PORT

**Moves now** (a Rust subject exists): `sec5_schedules.py` (6,465) into `strato-checks`, with the E_n function it needs; part B (time order) of `sec2_fv_truncation_order.py` as a Rust test that runs the real Code B. Part A of that file needs a source term, so the file stays until gate 2.1 (Section 5.5). Also in R3: the CI job of the independent Python suite and the path fix of the three files named in Section 3.

**Acceptance**: the AGR-1 plan durations 334, 336 and 338 h and the seven CRP-6 totals reproduce exactly as the Python file prints them; the observed time order of Backward Euler on the 200-cell mesh agrees with the Python result to the digits that file prints; the Python suite job is green; all gates.

**Risks**: E_n and other special functions without SciPy (a small, tested implementation or a crate); a generalised symmetric eigen-decomposition for the semi-discrete reference (for example `nalgebra`), whose accuracy limit the Python file documents (about 1e-9 in F).

**Expected percentages** (Rust added = 1.3 × 6,465 = 8,405): Python 725,832 (41.3%), TypeScript 658,527 (37.5%), Rust 285,324 (16.2%), total 1,756,331. Part B of the truncation check adds Rust without removing Python and is not in this estimate.

### 5.4 R4: the remainder

**Moves**: `scripts/data_register/render_register.py` (12,117) and `b6_nuclear_crosscheck.py` (6,113) into `strato-data`; the three figure scripts (19,972) into `strato-figures`; `web/tests/guards.test.ts` (5,310) into `strato-webtools` as a repository check; the data checks of `D5_parameter_register.py` into `strato-data` (L and Y stay in Python); `b6_audit_patch.py` (10,193) retired only with the owner's word, without a port.

**Acceptance**: `STRATO_Parameter_Register.md` byte-identical; the cross-check prints the same PASS and FAIL lines on the same downloaded responses; the figures are reviewed by the owner (a PNG is not byte-comparable between matplotlib and plotters; the committed PNGs stay until the owner accepts the new ones; the eigenvalues drawn in Figure 4.1 equal Code C); the repository check reports the same findings on the current tree and on a planted violation of each rule; all gates; the final language report.

**Risks**: figure appearance in the published derivation and its PDF; the network check depends on the IAEA service.

**Expected percentages** (Rust added = 1.3 × (38,202 + 5,310) = 56,566; the D5 split is not in this estimate): Python 677,437 (38.5%), TypeScript 653,217 (37.1%), Rust 341,890 (19.4%), CSS 84,739 (4.8%), total 1,759,192.

### 5.5 Ports that travel with the Phase 2 implementation

The eight PORT checks marked "after gate 2.1", "after gate 2.2" or "after chains (D3)" (84,054 bytes, including `sec2_fv_truncation_order.py`) become verification tests of the Rust solver in the stage that implements each mechanism; the numerical parts of the "(split)" SYMBOLIC files follow the same rule. They are not part of R1 to R4. With them (Rust added = 1.3 × 84,054 = 109,270): Python 593,383 (33.3%), TypeScript 653,217 (36.6%), Rust 451,160 (25.3%), total 1,784,408. What then remains in Python is the independent suite and the SymPy files.

## 6. Is 60 to 70% Rust reachable?

Not while the interface stays in TypeScript and the independent Python suite and the SymPy proofs stay, and the plan says so plainly.

After R1 to R4 and the Phase 2 ports, counted TypeScript is 653,217 bytes and stays (415,252 bytes of components and interface modules, 232,798 bytes of tests of rendering and 5,167 bytes of configuration), CSS 84,739, and Python at least 593,383 (independent suite 169,759 and SymPy files 423,624). With those fixed, 60% Rust needs 1.5 × (593,383 + 653,217 + 84,739 + 762 + 1,147) = 1,999,872 bytes of Rust, fifteen times today's 133,035. Even with no Python at all, 60% would need 1.5 × (653,217 + 84,739 + 762 + 1,147) = 1,109,798 bytes of Rust.

| Scenario (k = 1.3 unless stated) | Python | TypeScript | Rust |
|---|---:|---:|---:|
| Today (38769f7) | 47.9% | 39.1% | 7.7% |
| After R1 to R4 | 38.5% | 37.1% | 19.4% |
| Same, k = 1.0 / 1.6 | 39.6% / 37.5% | 38.2% / 36.1% | 17.2% / 21.6% |
| Plus the Phase 2 ports (Section 5.5) | 33.3% | 36.6% | 25.3% |
| Plus the Phase 2 solver itself, assuming it adds as much Rust as v1.0 has today (133,035 bytes; an assumption, not a measurement) | 30.9% | 34.1% | 30.5% |
| Plus every numerical part of the "(split)" files moved (upper bound: assumes the whole of those files, 146,111 bytes, leaves Python) | 22.8% | 33.3% | 39.5% |
| Plus the three overlapping references retired (owner's word) | 21.4% | 33.9% | 40.2% |
| Plus a further 50,000 bytes of TypeScript logic moved behind the wrapper (assumed upper bound; Section 2.4 finds about 15,000) | 21.2% | 31.1% | 43.2% |

So the honest expectation: about 19% after R1 to R4, about 25 to 30% once Phase 2 is implemented in Rust with its checks ported, and about 40% at the most with every legitimate move. From the fifth row on, Rust is the largest language of the repository, and by computational responsibility it is so much earlier: after R1 every generated file, check and build step of the web demo is Rust, and after R2 every number the visitor downloads is written by Rust.

Legitimate moves that raise the share further, in order of value:

1. Implement Phase 2.1 and 2.2 in Rust and port their checks as tests of the real solver (Section 5.5). This is the largest real gain and the purpose of the project.
2. Move more logic behind the wrapper where it is logic, not display, and move its tests with it (R2).
3. Split the mixed SymPy files and move their numerical parts as Rust tests once their subject exists.
4. Retire, with the owner's word, the references that only duplicate another reference.

Not proposed: padding, generated Rust, porting the independent references or the SymPy proofs for the count, moving `examples/verify.rs` out of the root `examples/` folder only so that it is counted (8,998 bytes), and `linguist-*` attributes that hide TypeScript, CSS or Python. Marking the committed generated JSON and CSV files `linguist-generated` would be honest but changes nothing (Section 1.4). The only route to 60% would be to rewrite the interface in a Rust web framework, which the owner has excluded ("the framework can be in javascript yes for the interface").

## 7. Open points for the owner

1. The target of 60 to 70% is not reachable under the brief's own constraints (Section 6); the realistic end state is Rust as the largest language at about 40%, with Rust responsible for all computation, generation, checking and export.
2. Keep or retire the three overlapping cross-language references (`v1_0_0_p4_sturm_weights.py`, `v1_0_0_p5_fv_replica.py`, `v1_0_0_p6_series.py`)?
3. Retire `scripts/data_register/b6_audit_patch.py`, whose changes are already in the CSVs and which the B6 audit log cites?
4. Accept new figures of the v1.0.0 derivation drawn in Rust (R4), or keep the matplotlib figures and their script?

## 8. How the numbers were measured

```text
gh api repos/weewzr/STRATO/languages                         # Section 1.1, main at 3e38398
git ls-tree -r -l 3e38398        > tree_main.txt             # main
git ls-tree -r -l 38769f7        > tree_web.txt              # web branch, 336 files
git show 38769f7:.gitattributes                              # no linguist attribute
git log --all --diff-filter=D --name-only -- '*.py' '*.ts' '*.sh' '*.js'   # empty: nothing ported yet
git archive 38769f7 | tar -x                                  # files read for the classification
```

The bytes per language are the sums of the size column of `git ls-tree -l` by extension (`.py`; `.ts` and `.tsx`; `.rs`; `.css`; `.js` and `.mjs`; `.html`; `.sh`), minus the files of Section 1.2. Linguist's rules were read from `lib/linguist/documentation.yml` and `lib/linguist/vendor.yml` of github-linguist/linguist; that the reading is right is shown by the exact match with GitHub's numbers for main.
