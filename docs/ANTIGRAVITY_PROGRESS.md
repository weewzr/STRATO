# Antigravity Progress — STRATO Phase 2 Execution

## Checkpoints

1. **Phase 1 Complete — Archive Recovery & Repository Sync**
   - Inspected the read-only local `STRATO_Archive` (2,447 files, 192.1 MB).
   - Verified `https://github.com/weewzr/STRATO` (`main` = `47c6175465bf5a10d1b741881280da3adebbdceb`, identical to archive snapshot).
   - Synced a fresh working clone of `weewzr/STRATO` on branch `feat/phase2-study39-final`.
   - Baseline checks passed: `cargo test --release` (`184 passed, 1 ignored` unit tests; `4+10+6` integration tests) and `BTCS_REGRESSION_CHECK PASS`.

2. **Phase 2 Step 2 Complete — Canonical 39-Graph Rust Simulations (`study39`)**
   - Added [`crates/strato-results/src/bin/study39.rs`](../crates/strato-results/src/bin/study39.rs) (`Cargo.toml` bin entry) running Method A (`ScenarioConfig` / `run_scenario`, `[160, 20, 20, 160, 20]` + `[320, 40, 40, 320, 40]` refinement) and Method B (`method_b`) across all 6 temperatures (`1200, 1350, 1500, 1600, 1700, 1800 °C`), Stages A, B (`InitialState::Fresh` + `1e-9` cross-check), C (`+G+λ` and `G=0+λ` decay-only), D (isothermal dynamic C16, `AGR1-ST-C16` / 50 °C/h heating ramp decay OFF/ON, and combined transient), plus `D06` AGR-1 irradiation (`955, 1028, 1136 °C`, 620.2 EFPD, C15 laws), radial profiles, `C01` convergence, and `C02` Method A vs B checks.
   - Outputs written under `data/study39/`: 48 runs, `max |ε|/bound = 0.04976 <= 1.0`, spatial order `2.014`, temporal order `1.077`, Method A vs B `<= 1.51e-3 pp`. `cargo fmt --check`, `cargo clippy`, and `cargo test --release` in `crates/` passed.

3. **Phase 2 Step 3 Complete — Quantitative Experimental Deviations & Stage A–D Improvement**
   - Implemented [`scripts/study39_deviations.py`](../scripts/study39_deviations.py), generating `data/study39/experimental/`, `deviations_exact_points.csv`, `deviations_exact_summary.csv`, `deviations_bounds.csv`, `stage_improvement.csv`, and `study39_manifest.json` (39 principal graphs).
   - Key verified metrics: `ACT3 1700 °C (N=8)` RMSE improves `+6.08%` (`0.240 -> 0.226 pp`) under Stage D heating ramp; `ACT4 1800 °C (N=11)` shifts `-2.48%` (`7.519 -> 7.706 pp`); Stages B/C `+G` use `F_gen` (distinct from `G=0` furnace `F_init`); 1200–1600 °C upper bounds audited separately (`<= 5%` at 1200/1350 °C; exceeds `0.01%` collector readings & 1600 °C `HFR-K3/1`).

4. **Phase 2 Step 4 Complete — 39 Principal Graphs + 10 Supporting Diagnostics**
   - Generated PNG (300 dpi), SVG, and CSV for all 39 principal IDs (`A01–A24`, `B01–B06`, `C01–C03`, `D01–D06`) + `S01–S10` under `graphs/study39/` via [`scripts/generate_study39_graphs.py`](../scripts/generate_study39_graphs.py); visually inspected and refined `C03` & `D05` legibility.

5. **Phase 2 Step 5 Complete — Finalised Research Report (DOCX & PDF)**
   - Built [`deliverables/report/STRATO_Research_Report_Phase2_Final.docx`](../deliverables/report/STRATO_Research_Report_Phase2_Final.docx) and 41-page [`deliverables/report/STRATO_Research_Report_Phase2_Final.pdf`](../deliverables/report/STRATO_Research_Report_Phase2_Final.pdf) via [`scripts/build_report.py`](../scripts/build_report.py), incorporating literature review (BISON, PARFUME, FV, FD, analytical), progressive equations, verbatim Rust source listings, all 39 embedded graphs, deviation/improvement tables, APA references, and blank governance statement bodies. Visually inspected rendered PDF pages.

6. **Phase 2 Step 6 Complete — Finalised Editable PowerPoint Presentation**
   - Reconstructed [`deliverables/powerpoint/STRATO_Presentation_Phase2_Final.pptx`](../deliverables/powerpoint/STRATO_Presentation_Phase2_Final.pptx) (109 slides, zero `REGISTERED` tags) from the 105-slide baseline via [`scripts/update_deck.py`](../scripts/update_deck.py), ending strictly with: `Experimental Deviation Summary` -> `Scientific Conclusions` -> `Limitations and Possible Mechanisms` -> `Future Research Directions`.

7. **Phase 2 Steps 7–8 Complete / In Verification**
   - Integrated `web/src/data/study39.json` & `web/src/documentation/Phase2StudyEvidence.tsx` into Scientific Documentation, fixed `about.tsx` Method B applicability text (`m5`), fixed `explore.css` sticky footer clearance at `1440×900`.
   - Added `archive/README.md`, rewrote root `README.md`, and updated `docs/ANTIGRAVITY_PROGRESS.md`.
