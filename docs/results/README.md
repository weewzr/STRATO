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
- A full run takes about 2.5 minutes (136 s measured) in a release build on the development
  machine and writes about 7.5 MB in 123 files. The numerical crates are optimised in the development profile of the tools
  workspace, so that the tests run in about a minute.

## Layout (web specification part 4, with the differences noted)

| File | Content |
|---|---|
| `support.json` | Every selector option and every reason why not: stages 1 to 5, the five scenarios, the temperatures of the main experimental comparison per nuclide, the cells (stage, context, nuclide, temperature) with Method A and Method B availability and reasons, the experiment rows, the model-exploration temperatures with the register rows behind each layer. |
| `parameters.json` | Geometry (outer radii, register rows), every diffusivity law with row id, class, phase of use and printed range, and the generation inputs (decay constants, yields, fission-rate density with its label and source rows). |
| `curves/<key>.json` | One curve file per cell. Keys: `s1.exp.<nuclide>.<T>c` (stage 1, experimental-comparison context), `s3.exp...` and `s4.exp...` (decay and temperature-dependent stages in that context), `s1.expl...` (model exploration), `s<N>.<scenario>.<nuclide>.<T>c` for scenario runs (variants: `.exploration` at 1197 degC, `.cycles`, `.heating1600/1700/1800`). |
| `experiment/agr1.<nuclide>.<T>c.json` | Experimental observations of one dataset: bound markers and window bounds with value, reading interval, status, premises (BP1, BP2, A1) and source; point markers only where a point comparison exists (none today). Every bound carries `comparison_class`, `denominator` (as the comparison program states them), `time_window` and `particle_applicability` (the register row's cells), and, for the ten matrix-corrected rows E-010..E-014 and E-033..E-037, `premise_source` and `window_end_origin`: class "window upper bound under premise BP1", denominator "as-run calculated compact inventory", the window from about 24 h at temperature to the end of the 300 h plateau or the shorter test end. Which rows these are comes from the comparison program, not from a list in the pipeline. |
| `bounds.json`, `csv/bounds.csv` | All bound comparisons with their statuses (technical section). |
| `nrmse.json`, `csv/nrmse.csv` | The NRMSE records exactly as `metric-nrmse.md` Section 7 defines them. Today N = 0 everywhere: each record states its exclusion reasons and no value is produced. This file replaces `deviations.json` of the specification (decision 64). The model slope behind the time-reading term is the difference over the actual spacing of two samples; where no difference is possible the reading uncertainty is "not available", never computed with a zero slope. |
| `scenarios.json`, `csv/scenario_histories.csv` | The scenario definitions as run: histories, source ids, labels. |
| `csv/scenario_series.csv`, `csv/curves.csv` | The decimated series of the scenario runs and the curves of the experimental and exploration cells. |
| `labels.json` | Colours, the NRMSE display rule and label templates (LaTeX and plain), and the label texts of the graphs generated in Rust. |
| `verification/convergence.json` | Observed orders in space and in time against the exact series. |
| `verification/agreement.json` | Solver-to-solver agreement of Method A and Method B at the stage 1 cells. |
| `verification/conservation.json` | Balance residuals against their derived bounds (Document 5, F46), for every curve. |
| `manifest.json` | Every file with the hash of its content and its size (`bytes_without_commit_lines`), both computed with the commit lines excluded, so that the manifest does not depend on the commit the files are generated with. |

## Differences from science design 7.3 and web specification part 4

| Item | Specification | Here |
|---|---|---|
| `deviations.json` | listed | replaced by `nrmse.json` and `bounds.json` (decision 64) |
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
