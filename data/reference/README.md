# STRATO data folder

This folder holds the literature parameters that STRATO may use for non-synthetic cases (Deliverable 5). Every value comes from the verified Stage 1 audit of the source papers or from an evaluated nuclear data library, and every row names its source, page and status.

## Rule: no synthetic values

Synthetic, notional or demonstration values never appear in `data/`. Synthetic inputs (for example the v1.0 five-layer test case) belong in `inputs/` or `examples/` and must not be copied here. A value enters `data/` only if it was read from a printed page, table, equation or library entry that the row cites. Values computed from such inputs (outer radii, decay constants, ramp durations) are allowed only when the CSV marks them as calculated and the check script recomputes them. Benchmark definitions that are synthetic by construction (CRP-6 sensitivity cases 1 to 5 and case 11) are allowed because they are printed specifications with published reference answers; they carry the class "specified (synthetic benchmark)".

## Files

| Path | Content | Rows |
|---|---|---|
| `reference/STRATO_Parameter_Register.md` | human-readable register: notation, defining relations (R.1) to (R.8), 40 numbered tables, verification output, gaps list, APA references | 503 lines |
| `geometry/geometry.csv` | particle and compact geometry, densities, defect fractions, derived outer radii (AGR-1, TECDOC-978 designs, HTR-10, JAERI 80FPC1, CRP-6 particles) | 149 |
| `diffusivity/arrhenius.csv` | Arrhenius terms for kernel, buffer, PyC, SiC, matrix and structural graphite (TECDOC-978 Tables A-1 to A-5, Eq. 4-10), Collin (2016) Table 2, Hales (2021) Table 1, TECDOC-1674 Table 7.1 and Eq. 10.9, buffer and failed-layer conventions | 118 |
| `nuclear/decay.csv` | ENSDF half-lives, decay constants with uncertainty, decay modes, daughters, production class | 16 |
| `nuclear/fission_yields.csv` | JEFF-3.1.1 independent and cumulative thermal yields for U-235 and Pu-239, ENDF/B cross-check, production class | 56 |
| `temperature/temperature_histories.csv` | AGR-1 safety-test procedure and modelled schedule, as-run deviations, irradiation temperatures, CRP-6 schedules, JAERI conditions | 122 |
| `experimental/release_measurements.csv` | AGR-1 compact release fractions (printed, digitised, upper bounds), failed-particle counts, JAERI Table 4-9, CRP-6 text values | 156 |

## Common columns

- `id`: row identifier, identical to the "No." column of the register tables.
- `source`: short APA citation (for example `Morris et al. (2016)`); `source_location`: file, table or equation, printed page and 0-based PDF index (`idx`; viewer page = idx + 1). The nuclear CSVs use `source`, `source_JEFF`, `source_ENDF` and `source_location`.
- `link`: DOI URL, or `file:Paper/n.pdf` for IAEA documents whose URLs are NOT VERIFIED.
- `status`: one of `verified`, `digitised`, `NOT VERIFIED` (definitions in Section 0.2 of the register). `fission_yields.csv` adds `status_ENDFB_crosscheck` for the NuDat 3 column.
- `evidence_class` (all six CSVs, added by the B6 audit): what kind of value the row holds, one of `measured`, `measured, digitised`, `measured bound`, `literature correlation`, `literature correlation (upper bound)`, `literature value (single temperature)`, `modelling convention`, `specification`, `synthetic benchmark`, `calculated`, `estimate`, `evaluated nuclear data`, `basis not stated` (definitions in Section 0.2 of the register). `status` says whether the value was checked; `evidence_class` says what it is.
- `particle_applicability` (`release_measurements.csv` only): whether the value is compact, particle-batch or sphere level and what it implies for a single intact particle.
- Activation products Ag-110m, Cs-134 and Eu-154 (and their isomer or ground-state partners) carry "NO fission-yield source" in `decay.csv` and "do NOT use as a fission-yield source" in `fission_yields.csv`.
- Units are in the column names or in a `unit` column. Activation energies are stored in $\mathrm{kJ\ mol^{-1}}$ and must be multiplied by $10^3$ before use with $R_g$ in $\mathrm{J\ mol^{-1}\ K^{-1}}$.

## Checking

Run

```bash
.venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/D5_parameter_register.py
```

after any edit. It reads the CSVs, writes nothing and prints PASS/FAIL lines (91 checks including the B6 family V, all PASS on 2026-11-06). If a value is changed, change the CSV first, re-run the script (save its output to a text file), then regenerate the register with

```bash
python -I -X utf8 scripts/data_register/render_register.py <STRATO root> scripts/data_register/register_template.md <out.md> <check_output.txt>
```

so that the CSV and the register keep the same row identifiers and values. Narrative text lives in `register_template.md`; table lines of the form `@@kind args@@` are filled from the CSVs.

## Provenance of the audit

Audit notes (with verification logs) are in the working `audit/` folder, which is not part of the repository: `03_tecdoc978_data.md`, `04_tecdoc1674_benchmarks.md`, `05_agr1_data.md`, `06_bison_parfume.md`, `07_nuclear_data.md`. The nuclear CSVs are copies of the audited `nuclear_decay.csv` and `fission_yields.csv` with three appended columns (`product_class`, `source_location`, `status`, plus `status_ENDFB_crosscheck` for yields); no value was changed.

The B6 audit (2026-11-06) re-checked the CSVs against page renders of the primary sources and a fresh LiveChart download; its log is `B6_audit_log.md` in this folder. The patch `scripts/data_register/b6_audit_patch.py` (idempotent) added the `evidence_class` and `particle_applicability` columns and changed 33 text cells (locations and notes); it changed no numerical value. `scripts/data_register/b6_nuclear_crosscheck.py <STRATO root> <download dir>` repeats the nuclear-data comparison (network required).
