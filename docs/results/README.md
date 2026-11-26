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
  `--commit <hash>` (informational), `--quick` (coarse grids for tests of the program itself; a
  quick run is never committed).
- `run --check` regenerates everything in memory and exits with status 1, naming every file that is
  missing, different or no longer produced. The lines that record the commit are ignored, because
  committing the outputs moves the commit.
- Every file starts with a provenance header: solver label, solver version, pipeline version,
  schema version, commit, `solver_source_sha256` (hash of `src/**/*.rs`, so that a change to the
  solver makes the outputs stale) and the SHA-256 of every input file (the registers under
  `data/experimental`, `data/irradiation`, `data/reference`, and the five scenario files).
- The numbers are produced by floating-point arithmetic; a different compiler, target or maths
  library can change the last digits, and then `--check` reports the files as stale on that machine.
  Regenerate on the machine that checks.
- A full run takes about 3 minutes in a release build on the development machine and writes about
  7 MB in 123 files. The numerical crates are optimised in the development profile of the tools
  workspace, so that the tests run in about a minute.

## Layout (web specification part 4, with the differences noted)

| File | Content |
|---|---|
| `support.json` | Every selector option and every reason why not: stages 1 to 5, the five scenarios, the temperatures of the main experimental comparison per nuclide, the cells (stage, context, nuclide, temperature) with Method A and Method B availability and reasons, the experiment rows, the model-exploration temperatures with the register rows behind each layer. |
| `parameters.json` | Geometry (outer radii, register rows), every diffusivity law with row id, class, phase of use and printed range, and the generation inputs (decay constants, yields, fission-rate density with its label and source rows). |
| `curves/<key>.json` | One curve file per cell. Keys: `s1.exp.<nuclide>.<T>c` (stage 1, experimental-comparison context), `s3.exp...` and `s4.exp...` (decay and temperature-dependent stages in that context), `s1.expl...` (model exploration), `s<N>.<scenario>.<nuclide>.<T>c` for scenario runs (variants: `.exploration` at 1197 degC, `.cycles`, `.heating1600/1700/1800`). |
| `experiment/agr1.<nuclide>.<T>c.json` | Experimental observations of one dataset: bound markers and window bounds with value, reading interval, status, premises (BP1, BP2, A1) and source; point markers only where a point comparison exists (none today). |
| `bounds.json`, `csv/bounds.csv` | All bound comparisons with their statuses (technical section). |
| `nrmse.json`, `csv/nrmse.csv` | The NRMSE records exactly as `metric-nrmse.md` Section 7 defines them. Today N = 0 everywhere: each record states its exclusion reasons and no value is produced. This file replaces `deviations.json` of the specification (decision 64). |
| `scenarios.json`, `csv/scenario_histories.csv` | The scenario definitions as run: histories, source ids, labels. |
| `csv/scenario_series.csv`, `csv/curves.csv` | The decimated series of the scenario runs and the curves of the experimental and exploration cells. |
| `labels.json` | Colours, the NRMSE display rule and label templates (LaTeX and plain), and the label texts of the graphs generated in Rust. |
| `verification/convergence.json` | Observed orders in space and in time against the exact series. |
| `verification/agreement.json` | Solver-to-solver agreement of Method A and Method B at the stage 1 cells. |
| `verification/conservation.json` | Balance residuals against their derived bounds (Document 5, F46), for every curve. |
| `manifest.json` | Every file with the hash of its content (commit lines excluded) and its size. |

## Units and conventions

- Release fraction: dimensionless, cumulative, fraction of the inventory of one particle (stage 1) or
  of the generated amount (scenarios, field `release_fraction_of_generated`), or the post-event
  fraction `F_post` (shutdown, refuelling). Amounts in mol; generation in mol m^-3 s^-1;
  diffusivity in m^2 s^-1; radii in m; temperature in degC; time in h unless a field says `_s` or `_d`.
- Method A is the BTCS solver (blue, #2563EB); Method B is the Sturm-Liouville solution (red,
  #DC2626), present only where the analytical module applies and drawn only where its mask accepts
  the value. A curve that needs the analytical extensions reads "not available: extension pending";
  merging `feat/analytical-extensions` changes only `crates/strato-results/src/methodb.rs`.
- A model value is "not resolved" (`resolved: false`) when the base run and the refined run (twice
  the intervals, half the step) differ by more than 5 % of the refined value (science design
  Section 3.5; the 5 % is a labelled convention, not a derived bound).
- Scenario inputs come from `data/scenarios/`; each number there carries its source row or the label
  "prescribed input (assumed)".
