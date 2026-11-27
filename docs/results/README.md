# Shared results of STRATO

`data/results/` holds every number and every figure data set that the website, the report and the
deck use. One Rust program writes it (`crates/strato-results`); nothing is recreated by hand for
one deliverable. Everything here is the output of a numerical solution of the stated model. A
comparison with a register compares release behaviour; it is not experimental validation, and a
scenario is a prescribed-input simulation, not a validated reactor prediction.

## Regenerate and check

The commands run from the repository root (`crates/Cargo.toml` is the tools workspace).

```text
cargo run --release --locked --manifest-path crates/Cargo.toml -p strato-results -- run
cargo run --release --locked --manifest-path crates/Cargo.toml -p strato-results -- run --check
```

- `run` writes `data/results/`. Options: `--repo <folder>` (default `.`), `--out <folder>`,
  `--commit <hash>` (informational; the committed set is generated with the hash of the source
  commit the outputs are built from, which is the commit before the one that adds them),
  `--quick` (coarse grids for tests of the program itself; a
  quick run is never committed).
- `run --check` regenerates everything in memory and exits with status 1, naming every file that is
  missing, different or no longer produced. The lines that record the commit are ignored, because
  committing the outputs moves the commit.
- Every file starts with a provenance header: solver label, solver version, pipeline version,
  schema version, commit, `solver_source_sha256` (hash of `src/**/*.rs`, so that a change to the
  solver makes the outputs stale) and the SHA-256 of every input file: the registers
  `geometry.csv`, `arrhenius.csv`, `decay.csv`, `fission_yields.csv`, `temperature_histories.csv`
  and `release_measurements.csv`, the three irradiation files (`agr1_irradiation_conditions.csv`,
  `agr1_burnup_by_cycle.csv`, `agr1_power_history_digitised.csv`) and the five scenario files.
  Nothing under `data/reference` is read.
- The numbers are produced by floating-point arithmetic; a different compiler, target or maths
  library can change the last digits, and then `--check` reports the files as stale on that machine.
  Regenerate on the machine that checks.
- A full run takes under 3 minutes (162 s measured) on the development machine and writes about
  8.4 MB (8 446 706 bytes) in 138 files. The numerical crates are optimised in the development
  profile of the tools workspace, so that the tests run in about a minute and a half.

## Layout (web specification part 4, with the differences noted)

| File | Content |
|---|---|
| `support.json` | Every selector option and every reason why not: stages 1 to 5, the five scenarios, the temperatures of the main experimental comparison per nuclide, the cells (stage, context, nuclide, temperature) with Method A and Method B availability and reasons, the experiment rows, the model-exploration temperatures with the register rows behind each layer. |
| `parameters.json` | Geometry (outer radii, register rows), every diffusivity law with row id, class, phase of use and printed range, and the generation inputs (decay constants, yields, fission-rate density with its label and source rows). |
| `curves/<key>.json` | One curve file per cell. Keys: `s1.exp.<nuclide>.<T>c` (stage 1, experimental-comparison context), `s3.exp...` and `s4.exp...` (decay and temperature-dependent stages in that context), `s1.expl...` (model exploration), `s<N>.<scenario>.<nuclide>.<T>c` for scenario runs (variants: `.exploration` at 1197 degC, `.cycles`, `.heating1600/1700/1800`). |
| `experiment/agr1.<nuclide>.<T>c.json` | Experimental observations of one dataset: bound markers and window bounds with value, reading interval, status, premises (BP1, BP2, A1) and source; point markers only where a point comparison exists (none today). Every bound carries `comparison_class`, `denominator` (as the comparison program states them), `time_window` and `particle_applicability` (the register row's cells), and, for the ten matrix-corrected rows E-010..E-014 and E-033..E-037, `premise_source` and `window_end_origin`: class "window upper bound under premise BP1", denominator "as-run calculated compact inventory", the window from about 24 h at temperature to the end of the 300 h plateau or the shorter test end. Which rows these are comes from the comparison program, not from a list in the pipeline. |
| `bounds.json`, `csv/bounds.csv` | All bound comparisons with their statuses (technical section). |
| `r2.json`, `csv/r2.csv` | The R² records of `metric-r2.md` Section 7.1, two per registered experimental curve (Method A then Method B), in the order of `data/experimental/curve_assessment.csv`. R² is the only whole-curve agreement metric (decision 69). With the registers of today no record has a value: each says which check failed first (`reason_code`, with its fixed caption), every failing check (`failed_checks`) and the premises. This file replaces `deviations.json` of the specification and the superseded records of decisions 64 and 65. |
| `comparison/<curve_id>.json` | One file per curve: the experimental points as registered (value, time, qualifier, reading uncertainty, class, status; markers, never connected), the Method A curve computed over the registered history with the diffusivity rows the assessment names (or the reason that none was computed), Method B only where the applicability function allows it and the mask passes, the two R² records and the caption record. |
| `scenarios.json`, `csv/scenario_histories.csv` | The scenario definitions as run: histories, source ids, labels. |
| `csv/scenario_series.csv`, `csv/curves.csv` | The decimated series of the scenario runs and the curves of the experimental and exploration cells. |
| `labels.json` | Colours, the R² display rule and label templates (LaTeX and plain), the label texts of the graphs generated in Rust, the R² labels of the records (empty where no value exists) and the caption record of every curve (`r2_captions`). |
| `verification/convergence.json` | Observed orders in space and in time against the exact series. |
| `verification/agreement.json` | Solver-to-solver agreement of Method A and Method B at the stage 1 cells. |
| `verification/conservation.json` | Balance residuals against their derived bounds (Document 5, F46), for every curve. |
| `manifest.json` | Every file with the hash of its content and its size (`bytes_without_commit_lines`), both computed with the commit lines excluded, so that the manifest does not depend on the commit the files are generated with. |

## Differences from science design 7.3 and web specification part 4

| Item | Specification | Here |
|---|---|---|
| `deviations.json` | listed | replaced by `r2.json`, `comparison/*.json` and `bounds.json` (decisions 64, 69 and 72) |
| Keys of scenario curves | `s<N>.<scenario>.<nuclide>.<T>c` | a variant segment is added (`.exploration`, `.cycles`, `.heating1600/1700/1800`) |
| Extra files | not listed | `bounds.json`, `labels.json`, `manifest.json`, `csv/*` |
| Refined series | refined `F` and `Phi` in scenario curve files | not written; the scenario curve files carry the resolution flag of every point, the largest base-to-refined difference of the inventory, and the base series; the refined runs are not stored |
| Step sequence | listed in the curve file | in `scenarios.json` (per scenario and variant), not in the curve files; the curve files carry the step count, the number of distinct step lengths and of matrix assemblies |
| Assumption codes | listed per curve | the labels and the basis of every input number are in `scenarios.json`; the curve files carry the label texts |
| C15 range status | per segment | `numerics.c15_documented_application` in every scenario curve file: per plan segment the steps below, inside and above 955 to 1197 degC (bounds read from IRR-361 and IRR-097) and the steps with the C16 set; `panel_title` and `labels` carry the label when any step is outside |

## The grid of the scenarios

One number of kernel intervals is chosen per nuclide and per family of variants from the longest
run of the family (the exploration variant at 1197 degC is one family; the base scenarios, the
cycle variant and the heating variants are the other), so that every scenario, every lead-in and
every normal-operation reference of a family run on the same grid. The Scenario 2 and Scenario 5
runs therefore reproduce the Scenario 1 state at the end of irradiation exactly. The references of
Scenario 3, Scenario 4 and the cycles variant differ from Scenario 1 only by their step boundaries
(Scenario 4 splits the run at 310.1 d; Scenario 3 has its 24 h ramp segment; the cycles variant
has the step boundaries of its cycles, and its Sr-90 released amount at 620.2 d differs from
Scenario 1 by about 1.5e-6 relative). The steps of a segment are equal to the
bit, and the matrix is assembled once per segment.

## Scenario 5 with a heating history: kernel-only comparison

Each heating cell (`heating1600`, `heating1700`, `heating1800`) carries `kernel_only_comparison`:
`F_post` of the heating run beside the run of the same registered history from a kernel-only start
(`C = C_init` in the kernel, no generation, C16 throughout), both as the amount released since the
start of the history over the inventory present at that start, with the ratio and resolution flags,
and the values at the end of the last hold at the highest temperature and at the end of the
history. The premise BP2 refers to this field and states no number.

## The window-start sensitivity

Every matrix-corrected window row (decision 67) in `bounds.json` and `experiment/*.json` carries
`window_start_sensitivity`: the model increase and its ratio to the bound for window starts of 20 h
and 28 h, labelled prescribed input (assumed), because the source gives no exact start ("about
24 h"); the value used for the status is that at 24 h.

## R² records and the experimental curves

`r2.json`, `csv/r2.csv` and `comparison/*.json` apply the definition `metric-r2.md`
(definition id `R2_LINEAR_PER_CURVE_V1`; decisions 69, 72 and 74). R² = 1 - SS_res / SS_tot over the
N matched points of one curve, computed per curve and never pooled; the model value at each point is
the solver output at an exact step boundary of the run (no interpolation of the model enters any
number). The record of a curve and method has `status` `available` only when all seventeen checks
pass (M0, C1 to C6, D1, C7, C8, M1, M2, B1, B2, D2, C0, V1); otherwise `r2`, `display` and the labels
are `null`, `reason_code` is the first failing check and `failed_checks` lists every failing one.
Today every record is unavailable. The registered histories are used with their ramps; where the
register does not state the start temperature of the first ramp the run starts at the end of that ramp
and the record says so (premise `first_ramp_not_modelled`, check C5 fails); the German sphere curves
use the FRG reference correlations of IAEA-TECDOC-978 (decision 72(b)). The assessment file has no
columns for `decay_reference`, `inventory_reference`, `inventory_reference_offset_h` and
`counting_delay_h`: they are `null` in every record (never defaulted), which is why checks C6 and D1
list them. The independent recomputation (V1) and the independent verification of the curve
assessment (C0) are recorded nowhere yet, so those checks also fail.

Further rules of the records, each stated in the record that it affects:

- C4: a negative value passes only when the source presents it as a cumulative release fraction and
  its magnitude does not exceed its stated uncertainty (`metric-r2.md` Section 3.1). No registered
  curve has a negative value.
- Ramp (C5): where the register says that the source's time axis runs through the heating ramp
  ("negative values are the ramp") and the registered history holds the hold only (the two Stempien
  (2023) compact 6-4-3 curves), C5 fails, the premise `temperature_ramp_not_modelled` is stated,
  Method B is not applicable (B1) and is not drawn.
- `y_axis_scale` is the value-axis scale that the register states for a digitised figure; where it
  states none (or only that the gridlines fit a linear or a logarithmic scale), the field is `null`
  and C6 lists "axis scale of the digitised figure". It is never defaulted.
- `independence` follows from the source records of the diffusivity rows used: `calibration_check`
  only where a row used is recorded as fitted to the table that the curve is read from. The JAERI
  curves run with the C16 rows, none of which is recorded as fitted to Table 4-9, so they are
  `not_established`.
- Method B error estimate: `e_num` is `|V_alphamax - V_alphamax/4| + accuracy_floor` (the series with
  the eigenvalue cover reduced by a factor 4, `metric-r2.md` Section 2.3); where the quarter-cover
  value is not defined the point is unresolved and M2 fails.
- The source line of a curve whose link has no recorded check says so (`source_link_note`); no link
  has been checked by this pipeline. The independent coordinate verification of digitised rows and
  the stored run outputs that route (i) of V1 needs are not recorded yet (`digitisation_verification_ref`
  is `null`, `model_run_ref` holds hashes and no paths), so C0 and V1 fail.
- The comparison program (`data/comparison`, `docs/validation`) writes per-point diagnostics only;
  no aggregate deviation over points is computed there (decision 69(a)).

## Units and conventions

- Release fraction: dimensionless, cumulative, fraction of the inventory of one particle (stage 1) or
  of the generated amount (scenarios, field `release_fraction_of_generated`), or the post-event
  fraction `F_post` (shutdown, refuelling). Amounts in mol; generation in mol m^-3 s^-1;
  diffusivity in m^2 s^-1; radii in m; temperature in degC; time in h unless a field says `_s` or `_d`.
- Method A is the BTCS solver (blue, #2563EB); Method B is the Sturm-Liouville solution (red,
  #DC2626), present only where the analytical module applies and drawn only where its mask accepts
  the value. One function (`crates/strato-results/src/methodb.rs`) asks the applicability function
  of the analytical module (science design Section 4.3) and draws through `KernelRun::drawn`
  (Section 4.2, Document 6 Section 8): the baseline at a constant temperature is exact, decay at a
  constant temperature goes through formula (B2), a piecewise-constant kernel source through (B3)
  to (B5); a temperature that changes in time is never drawn
  (`reason_code` `TIME_VARYING_TEMPERATURE`). A value the mask withholds is `null` and its reason
  code stands at the same index of `method_b.withheld_reason_codes`, with the legend in
  `method_b.withheld_reason_legend`; a curve with no drawn value has `method_b.available: false`
  with `reason_code` and `reason` (for example `NOT_DRAWN_AT_ANY_TIME`, which lists the reasons
  and their counts). The existing series now uses the same mask, so its floor factor is the 4 of
  Document 6 Section 5. `EXTENSION_FLOOR_REESTIMATED` is true (decision 68(c)).
- A model value is "not resolved" (`resolved: false`) when the base run and the refined run (twice
  the intervals, half the step) differ by more than 5 % of the refined value (science design
  Section 3.5; the 5 % is a labelled convention, not a derived bound).
- Scenario inputs come from `data/scenarios/`; each number there carries its source row or the label
  "prescribed input (assumed)".
