# Data-layer checks: what the Python audit scripts check and where Rust checks it

The crate `strato-data` (folder `crates/strato-data`) reads the nine files of `data/` (the six register files and the three irradiation files) into typed Rust values and reproduces, in Rust, every consistency check that the Python audit scripts make on those files. The command is

```
cargo run --locked --release --manifest-path crates/Cargo.toml -p strato-data -- check --repo .
```

It prints one `PASS` or `FAIL` line per check and a `SUMMARY` line, writes nothing, and exits with status 1 if any check fails or a file cannot be read. CI runs it in the job of `.github/workflows/web.yml` that already checks the tools workspace (`crates/Cargo.toml`), and `cargo test` runs the same checks as tests.

## How to read the tables

- **Id**: the stable id of the check. The first part names the Python script: `D5` = `scripts/derivation_checks/D5_parameter_register.py`, `B4D` = `b4_decay_source_pseudocode.py`, `B4P` = `b4_phase2_pseudocode_integration.py`, `B6` = `scripts/data_register/b6_nuclear_crosscheck.py`, `GEN` = `scripts/web/generate_data.py`, `CAT` = `scripts/web/check_catalogue.py`, `LOAD` = new in Rust, `DI` = the data integration of decisions 39 to 46 (new in Rust, no Python counterpart).
- **Python source**: file and line of the check in the script (the line of the `report(...)` or `check(...)` call, or the first line of the block).
- **Rust test**: every check id has one test in `crates/strato-data/tests/audit_checks.rs` named after the id in lower case with every character that is not a letter or digit replaced by `_` (for example `D5.L.lambda` is `d5_l_lambda`). It runs the check on the real files and asserts that the check ran and passed. The test `the_list_of_ids_is_the_list_of_checks_that_run` fails if a check is added or removed without updating the list, and `the_whole_check_set_passes` asserts the summary.
- **Tolerances** are those of the Python scripts, unchanged. Where the Python script used SymPy (symbolic algebra) the Rust check is exact-decimal or numerical and says so in its description; the symbolic residual checks are not retired, they are re-expressed. No tolerance was loosened.
- **Several Python lines, one Rust check**: where Python prints one line per row (stable nuclides, ramp rows) the Rust check covers all rows of the family in one result and reports the count.
- The checks can fail: `crates/strato-data/tests/failures.rs` damages one cell of a copy of the real text and asserts that the named checks fail and that the readers refuse malformed input.

The Python results on the repository state of this branch were run for comparison (`D5_parameter_register.py`: ALL PASS); the Rust check prints the same numbers (for example the worst `lambda` difference 3.91e-07, the 27 FRG values 3.0e-04, F = 0.46741912 at 1200 C).

## D5_parameter_register.py (91 checks in the script, README.md of data/reference)

| Id | Python source | What it checks | Rust test |
|---|---|---|---|
| `D5.S.header.{geometry,arrhenius,temperature,release,decay,yields}` | `D5...py:72` | source column (`source`, `source_JEFF` for yields) and `status` column present in the header | `d5_s_header_<file>` |
| `D5.S.status.<file>` | `:73-74` | every `status` is one of verified, digitised, NOT VERIFIED | `d5_s_status_<file>` |
| `D5.S.ids.<file>` | `:75-77` | row ids unique (decay: `nuclide`). `fission_yields.csv` has no id column: the Python script does not check it; Rust checks the key (parent, product, level index) | `d5_s_ids_<file>` |
| `D5.S.rect.<file>` | `:78-79` | every row has as many cells as the header | `d5_s_rect_<file>` |
| `D5.G.identity` | `:83-87` | R_i - R_(i-1) = t_i; SymPy replaced by exact decimal arithmetic on every complete design | `d5_g_identity` |
| `D5.G.layers` | `:95-97` | every design with outer radii has the layers 1 to 5 (Rust also demands one row per layer) | `d5_g_layers` |
| `D5.G.radii` | `:94-107` | all outer radii recomputed from the diameter and thicknesses, |diff| < 1e-9 um, count = 5 x designs (16 designs, 80 radii) | `d5_g_radii` |
| `D5.G.crp6` | `:108-109` | CRP-6 sensitivity radii 250/350/390/425/465 | `d5_g_crp6` |
| `D5.G.agr1` | `:110-111` | AGR-1 2R = 788.4 um | `d5_g_agr1` |
| `D5.L.uncertainty-formula` | `:114-117` | u(lambda) = lambda u(T)/T; SymPy replaced by a central difference of ln2/T against the closed form | `d5_l_uncertainty_formula` |
| `D5.L.stable` | `:121-122` | stable rows have lambda = 0 (2 rows) | `d5_l_stable` |
| `D5.L.lambda` | `:124-130` | lambda = ln2/T1/2 within 1e-6 for the 14 radioactive rows | `d5_l_lambda` |
| `D5.L.unc-lambda` | `:126-131` | u(lambda) = lambda u(T)/T within 1e-3 | `d5_l_unc_lambda` |
| `D5.L.iaea.{Cs-137,Sr-90,Ag-110m}` | `:132-137` | half-lives against IAEA (2012) Table 8.1 within 0.5 % | `d5_l_iaea_cs_137` and the two others |
| `D5.F.values` | `:139-144` | exp(Gamma/5) at 0, 1, 2, 3, 4.5 (divisor read from the documented factor of A3-01) | `d5_f_values` |
| `D5.F.limit` | `:145` | factor tends to 1 as Gamma tends to 0 (numerical, in place of SymPy limit) | `d5_f_limit` |
| `D5.F.eq109` | `:146-152` | Eq. (10.9) at 1600 C, Gamma = 2: 1.0113e-16 vs printed 1.011e-16 within 1e-3 | `d5_f_eq109` |
| `D5.K.T71-01` ... `D5.K.T71-05` | `:154-161` | D0 = D'0 r_k^2 with r_k = 250 um to 1e-12 | `d5_k_t71_01` ... `d5_k_t71_05` |
| `D5.X.equals` | `:163-175` | 27 "equals" claims of Collin 2016 and Hales 2021 match the cited TECDOC-978 rows (D0 and Q) | `d5_x_equals` |
| `D5.X.hales.{H21-04,H21-06,H21-09}` | `:176-178` | Hales single term equals the first branch | `d5_x_hales_h21_04` and two others |
| `D5.A.frg-values` | `:180-203` | 27 FRG D(T) values reproduce the audit table to 5e-4 | `d5_a_frg_values` |
| `D5.A.r-sensitivity` | `:204-207` | R = 8.3142 instead of 8.3143 changes D(1600 C) by -3.8e-4 to -1.1e-4 | `d5_a_r_sensitivity` |
| `D5.B.booth-zero` | `:209-213` | Booth F(0) = 0; SymPy summation replaced by a partial sum with tail correction | `d5_b_booth_zero` |
| `D5.B.case-1200`, `D5.B.case-1600` | `:216-225` | CRP-6 case 1: F = 0.4673 (3e-4) and 0.99999959 (5e-9) | `d5_b_case_1200`, `d5_b_case_1600` |
| `D5.B.single-branch` | `:226-227` | single Cs branch of Table 7.1 gives F within 5e-4 of 0.4614 | `d5_b_single_branch` |
| `D5.Y.steady-state` | `:229-239` | y_I(d) Fdot + b lambda_p N_p = y_C(d) Fdot; the SymPy ODE solution is replaced by the closed form, on three parameter sets | `d5_y_steady_state` |
| `D5.Y.overcount` | `:240` | y_C Fdot + lambda_p N_p over-counts by exactly y_I(p) Fdot | `d5_y_overcount` |
| `D5.Y.initial` | `:241` | parent inventory zero at t = 0 | `d5_y_initial` |
| `D5.Y.ode-solution` | `:234` | the closed form satisfies the ODE (replaces `dsolve`; new numerical check of the closed form) | `d5_y_ode_solution` |
| `D5.T.ramps` | `:247-255` | ramp_h = abs(dT)/abs(rate) within 6e-5 h for exactly 12 rows | `d5_t_ramps` |
| `D5.T.cumulative` | `:256-266` | cumulative_end_h_calc = running sum of ramp and hold within 1e-3 h (83 entries) | `d5_t_cumulative` |
| `D5.T.total.<history>` (11) | `:267-272` | schedule totals within 0.5 h of the printed totals | `d5_t_total_agr1_st_c16_1600` etc. |
| `D5.E.range01` | `:276-277` | the 135 dimensionless values are in [0, 1] | `d5_e_range01` |
| `D5.E.minmax` | `:278-280` | value_min <= value_max | `d5_e_minmax` |
| `D5.E.fraction.{E-114,E-115,E-116}` | `:281-284` | failed-particle fraction within 2 % of the audited value | `d5_e_fraction_e_114` and two others |
| `D5.E.failures-1800` | `:285-286` | failures at 1800 C sum to 23 | `d5_e_failures_1800` |
| `D5.E.one-particle` | `:287` | 1/4145 = 2.41e-4 within 5e-7 | `d5_e_one_particle` |
| `D5.E.jaeri-count` | `:288-289` | 33 JAERI rows | `d5_e_jaeri_count` |
| `D5.A.sanity` | `:291-296` | D0 > 0, Q >= 0, T_min <= T_max | `d5_a_sanity` |
| `D5.V.b6-coverage` | `:331-336` | the 81 B6 re-read rows are exactly the Ag/Cs/Sr rows of A1-A5, C16-T2, H21-T1 (the 81 values are copied into `checks.rs`, `B6`) | `d5_v_b6_coverage` |
| `D5.V.b6-values` | `:337-342` | D0, Q, T_min, T_max identical for 81/81 rows | `d5_v_b6_values` |
| `D5.V.rg` | `:343-345` | R convention per set (the Python dictionary keeps the last row of each three-character set prefix; Rust does the same) | `d5_v_rg` |
| `D5.V.evidence.<file>` | `:346-352` | evidence_class in the vocabulary of 13 classes, on the raw text of each file | `d5_v_evidence_<file>` |
| `D5.V.applicability` | `:353-355` | particle_applicability filled for every release row | `d5_v_applicability` |
| `D5.V.bounds` | `:356-357` | every "<" row is classed measured bound | `d5_v_bounds` |
| `D5.V.activation-yields` | `:358-363` | 12 activation-product yield rows carry "do NOT use as a fission-yield source" | `d5_v_activation_yields` |
| `D5.V.fission-sources` | `:364-366` | fission-product nuclides of decay.csv exclude Ag-110m, Cs-134, Eu-154 | `d5_v_fission_sources` |

## b4_decay_source_pseudocode.py (only the checks that read the register files)

| Id | Python source | What it checks | Rust test |
|---|---|---|---|
| `B4D.A.stable` | `b4_decay_source_pseudocode.py:129-130` | stable rows have lambda = 0 | `b4d_a_stable` |
| `B4D.A.seconds` | `:131-138` | half_life x unit (year = 31556925.97 s) reproduces half_life_s within 5e-9 | `b4d_a_seconds` |
| `B4D.A.lambda` | `:139-140` | ln2/t_half reproduces lambda within 5e-7 | `b4d_a_lambda` |
| `B4D.A.product` | `:136, :141` | lambda x t_half = ln 2 within 1e-12 for every row | `b4d_a_product` |
| `B4D.A.cs137` | `:142-144` | Cs-137 lambda = 7.302187e-10 s-1 within 1e-6 | `b4d_a_cs137` |
| `B4D.A.mean-life` | `:145` | 1/lambda = t_half/ln2 within 1 s | `b4d_a_mean_life` |
| `B4D.A.negative` | `:146-150` | a negative half-life is rejected (Rust `decay_constant`) | `b4d_a_negative` |
| `B4D.B.yields` | `:155-168` | IFY and CFY of Cs-137 and Sr-90, U-235 thermal | `b4d_b_yields` |
| `B4D.B.source-term` | `:187-189` | S_K = y F / N_A for Cs-137 | `b4d_b_source_term` |
| `B4D.B.weighted-yield` | `:190-194` | the 0.7/0.3 fission-weighted Cs-137 yield lies between the pure yields | `b4d_b_weighted_yield` |

The other checks of this script (H21.x identities, t99 formulas, time grids, the finite-volume integration, cases F1 to F15, E) do not read the register files; they check the finite-volume pseudocode and are outside this task (the coordinator's decision 26 cancels the ports of finite-volume-specific checks).

## b4_phase2_pseudocode_integration.py (only the checks that read the register files)

| Id | Python source | What it checks | Rust test |
|---|---|---|---|
| `B4P.nuclide.{Cs-137,Sr-90,Ag-110m,I-131}` | `b4_phase2_pseudocode_integration.py:589-599` | lambda = ln2/t_half matches the pseudocode table (5e-7), t_half matches (1e-6) | `b4p_nuclide_cs_137` and three others |
| `B4P.source-example` | `:600-605` | Cs-137 y = 6.2208e-2 (1e-9), S_K = 5.1649e-6 (5e-11) | `b4p_source_example` |
| `B4P.step-sizes` | `:606-611` | Ag-110m 3.62e4 s (0.5 %) and I-131 400 s (1 s) from lambda of decay.csv | `b4p_step_sizes` |

The script's own copy of the A3-01 and A3-02 constants (line 420) is not read from the CSV, so no check reads the register there. The other checks of this script are about the finite-volume pseudocode and do not read the register files.

## b6_nuclear_crosscheck.py

| Id | Python source | What it checks | Rust test |
|---|---|---|---|
| `B6.activation.decay` | `b6_nuclear_crosscheck.py:126-129` | Ag-110m, Cs-134, Eu-154 are classed as activation products in decay.csv | `b6_activation_decay` |
| `B6.activation.yields` | `:130-134` | the six activation-product rows per parent are activation-class and their use note contains "not" or "no " | `b6_activation_yields` |
| `B6.activation.no-source-term` | new | no fission source-term yield can be built for an activation-product row (hard error in the type `SourceTermRefusal`) | `b6_activation_no_source_term` |

Not ported: lines 56-123 compare `decay.csv` and `fission_yields.csv` with a fresh download of the IAEA LiveChart API (half-lives 1e-9, lambda 1e-6, branches, yields 1e-9; 204/204 PASS on 2026-11-06). They need the network and a download folder and cannot run in CI. The result of 2026-11-06 (`data/reference/B6_audit_log.md`) is the last full comparison. The offline remainder of the script is the two rows above. A Rust on-demand tool for the network comparison is not part of this task.

## generate_data.py (geometry extraction for the web build)

| Id | Python source | What it checks | Rust test |
|---|---|---|---|
| `GEN.geometry.rows` | `scripts/web/generate_data.py:1062-1070` | each of the 7 AGR-1 reference sets has exactly one kernel diameter row and one thickness row per coating layer, unit um, with a value | `gen_geometry_rows` |
| `GEN.geometry.radii` | `:1105-1117` | where the register gives an outer radius it equals the sum of diameter/2 and thicknesses, in exact decimal arithmetic (`decimal.Decimal` in Python) | `gen_geometry_radii` |
| `GEN.geometry.default` | `:1044, :1173` | the default set is one of the sets | `gen_geometry_default` |

The generator itself (writing `web/src/generated/agr1_reference.json` and the geometry CSV) is not ported here; the coordinator's decision 27 assigns it to a separate task. `check_catalogue.py` rules 1 to 5 and 7 to 9 read only `data/research/*` and `docs/research/*`, not the six register files.

## check_catalogue.py

| Id | Python source | What it checks | Rust test |
|---|---|---|---|
| `CAT.nuclides` | `scripts/web/check_catalogue.py:204-221` | every nuclide named in `data/research/species.csv` is in decay.csv or marked "(not in register)"; skipped, as in Python, when species.csv is absent | `cat_nuclides` |

## New in Rust

| Id | What it checks | Rust test |
|---|---|---|
| `LOAD.register` | the six files are read by the typed readers (exact header, rectangular rows, closed vocabularies, typed values) | `load_register` |
| `LOAD.rows.<file>` | the number of typed rows equals the number of data rows of the file: no row is dropped | `load_rows_<file>` |

## Scripts that are not checks

- `scripts/data_register/b6_audit_patch.py`: the one-time idempotent patch that added `evidence_class` and `particle_applicability`. Its only assertion (evidence class in the vocabulary) is `D5.V.evidence.<file>`. History, not a check.
- `scripts/data_register/render_register.py` and `register_template.md`: generate `STRATO_Parameter_Register.md` from the CSVs. A generator, not a check. No Rust renderer is part of this task.
- `scripts/derivation_checks/D5_parameter_register.py` stays in place; no Python script is deleted by this task.

## Family DI: the data integration (new in Rust; expected numbers are those printed in the three research reports)

| Id | What it checks | Rust test |
|---|---|---|
| `DI.class.one-per-row` | every row of `arrhenius.csv` has exactly one of the four classes of owner ruling 16; the class follows the set (TECDOC-978, BUF, E410, T71, E109: transferred; C15, C16, H21: PARFUME; S20: AGR-1 fitted); no row is `NOT VERIFIED` | `di_class_one_per_row` |
| `DI.class.no-measurement` | no row is an AGR-1 experimental measurement | `di_class_no_measurement` |
| `DI.phase.rules` | `phase_of_use`: irradiation for C15, H21 and A1-05, heating test for C16, S20, A1-06, A1-07, E410-01, A3-14, not specified for the rest | `di_phase_rules` |
| `DI.radius.kernel` | `reference_radius_um` is 250 on the kernel rows of A1, T71, C15, C16, H21 and empty elsewhere | `di_radius_kernel` |
| `DI.fluence.definition` | `fluence_definition` is filled on A3-01, E109-01, C15-07, C16-08, H21-06 and on every row that documents the exp(Gamma/5) factor | `di_fluence_definition` |
| `DI.select.resolves` | every entry of the default table of `select` resolves to rows of one set whose species, layer, term order and class agree with the file | `di_select_resolves` |
| `DI.select.decision39` | irradiation selections use single-term C15 rows, heating-test selections use C16 rows with both terms for Cs and Kr in the kernel, Cs and Sr in SiC and Kr in PyC | `di_select_decision39` |
| `DI.select.c15-equals-c16` | first term of every C16 row equals the C15 row of the same species and layer (D0 and Q as printed) | `di_select_c15_equals_c16` |
| `DI.select.second-term-need` | ratio of the two-term coefficient to its first term at 1500, 1600, 1700, 1800 C reproduces section 4 of the diffusivity report (1.503, 2.297, 91.1, 2.011, 5.134, 47.0, 1.461, 3.581, 1.001) to 1.1e-3 (below 10) or 2.5e-3 | `di_select_second_term_need` |
| `DI.select.refuses-outside-range` | Kr in SiC (C16-20, 1400 to 1750 C) is refused at 1800 and 1399 C, an extrapolation is marked, rows without a printed range give a value anywhere | `di_select_refuses_outside_range` |
| `DI.select.alternative` | Schappel rows S20-01 to S20-03 only for heating-test Ag, Cs, Sr in SiC, never the default, refused for irradiation | `di_select_alternative` |
| `DI.select.default-class` | every default selection is `published PARFUME correlation` except A1-05 (transferred) | `di_select_default_class` |
| `DI.select.unavailable` | Eu, Kr in SiC during irradiation and the Schappel set for the kernel (Err, no fallback to the default rows) return DATA UNAVAILABLE; Xe has a row only where it is printed for Xe (A1-05, labelled) | `di_select_unavailable` |
| `DI.select.schappel-range` | the Schappel rows S20-01 to S20-03 carry the typed fitted range 1600 to 1800 C; 1000 C is refused or marked; the label says R is not printed | `di_select_schappel_range` |
| `DI.select.scaling-labels` | no selection with the kernel scaling carries a label saying unscaled and each says it is scaled; default kernel selections say unscaled | `di_select_scaling_labels` |
| `DI.select.kernel-scale` | the sensitivity factor equals (174.85/250)^2 = 0.4892 (decision text 0.489) and scales the evaluated coefficient exactly | `di_select_kernel_scale` |
| `DI.release.reading-uncertainty` | empty exactly for rows not read from a figure; 187 stated and 32 not recorded (Morris 2016 vector figures) among the 219 digitised rows, every digitised Collin 2016 row stated | `di_release_reading_uncertainty` |
| `DI.release.non-detect` | class M-ND on exactly E-217, E-220, E-221, E-225, E-228, E-231, each an upper bound | `di_release_non_detect` |
| `DI.release.group-label` | the group label for no counted failure but a releasing particle on exactly E-406 and E-417 | `di_release_group_label` |
| `DI.release.withdrawn-ids` | E-316 to E-373 absent, ids unique | `di_release_withdrawn_ids` |
| `DI.release.denominator` | denominators by dataset; 61 rows not established | `di_release_denominator` |
| `DI.irr.completeness` | 2028 burnup rows: 72 compacts and 6 capsules, 13 cycles, burnup and fluence, once each | `di_irr_completeness` |
| `DI.irr.monotone` | cumulative values never decrease; cycle ends equal the running sum of the 13 cycle lengths (0.1 EFPD) | `di_irr_monotone` |
| `DI.irr.rev3-difference` | Appendix D after cycle 13 minus Tables 3 and 4: -0.01 to +0.05 percent FIMA | `di_irr_rev3_difference` |
| `DI.irr.power-windows` | 124 five-EFPD windows for each of the 6 capsules | `di_irr_power_windows` |
| `DI.irr.inventory` | steps 1 to 4: 2.32627e21 atoms per compact, 5.6001e17 per particle, 2.23916e-11 m3, 2.50097e28 m-3 (2e-5) | `di_irr_inventory` |
| `DI.irr.step8` | time-average fission-rate density for eight burnups (5.27e19 to 9.13e19) to the printed three figures | `di_irr_step8` |
| `DI.irr.step9` | all 78 capsule-average cycle values equal the printed table (half a unit of the last printed digit) | `di_irr_step9` |
| `DI.irr.derived-series` | weighted means 8.709e19 (capsule 3) and 6.240e19 (capsule 6), extremes 2.485e19 and 1.269e20, label text | `di_irr_derived_series` |
| `DI.register-md.matches-csv` | every `| E-xxx |` row of `STRATO_Parameter_Register.md` restates class, qualifier, value and notes of `release_measurements.csv`, and every coefficient row restates pre-factor, activation energy, source, temperature range and class of `arrhenius.csv` | `di_register_md_matches_csv` |

Failure tests added in `tests/failures.rs`: a register row that drifts from the CSV (class, value, range), an unknown class, a wrong class for a set, a lost printed range, a changed C15 value, a non-detect that loses its class, a digitised row without reading uncertainty, a malformed reading uncertainty, a reused withdrawn id, a falling burnup, a changed particle count.

Workspace: the Rust tests and checks belong to the workspace of `crates/Cargo.toml`. At the repository root `cargo test` runs only the `strato` package; run `cargo test --locked --manifest-path crates/Cargo.toml` for the `strato-data` tests (155 check tests, 29 failure tests, 17 and 22 in the type and reader tests, 16 unit tests at the time of writing) and `cargo run --locked --manifest-path crates/Cargo.toml -p strato-data -- check --repo .` for the checks.
