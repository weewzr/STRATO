# Scenario definitions

One CSV file per operating scenario (science design Section 5, decision 58). Every scenario is a
prescribed-input simulation: its generation and temperature histories are stated inputs, and no
scenario is a validated reactor prediction. STRATO does not model neutron kinetics, control-rod
motion, thermal-hydraulics, power redistribution or mechanical refuelling; only the generation
rate and the particle temperature change, and no control-rod or refuelling term enters the
equation.

| File | Scenario |
|---|---|
| `s1_normal_operation.csv` | S1 normal steady-state operation (variant `cycles`: the 13 ATR cycles) |
| `s2_shutdown_cooldown.csv` | S2 reactor shutdown and cooldown |
| `s3_startup.csv` | S3 reactor startup |
| `s4_control_rod_insertion.csv` | S4 control-rod insertion (a prescribed power reduction) |
| `s5_fuel_removal.csv` | S5 fuel removal (variants `heating-1600`, `heating-1700`, `heating-1800`: the registered AGR-1 safety-test histories after storage) |

## Columns

`row_id, record, variant, quantity, t_value, t_unit, value, unit, interpolation, source_ids, basis, text`

* `record` is `history` (a breakpoint of `G_kernel_factor`, `T_C` or, in the `cycles` variant,
  `fission_rate_density`), `step` (a segment of the step plan: the largest step `dt_max` up to the
  time `t_value`) or `meta` (a statement or one stated number).
* `variant` is empty for the base scenario. The rows of a variant replace all base rows of the same
  quantity (for `history`) or all base step rows (for `step`) and the base metadata row of the same
  key. `exploration-1197C` is the registered exploration variant of decision 66: the compact
  time-average peak temperature IRR-097 (1197 degC) with the fission-rate density derived for the
  same compact (6-2-1); it is labelled exploration and is not a normal-operation reference.
* `G_kernel_factor` is dimensionless: the kernel source is this factor times the reference rate
  `y F_dot / N_A` (Document 5, Eq. (2)). `T_C` is in degC. Times are in `h` or `d` (or `s`); times
  of S2, S4 and S5 count from the event (shutdown, insertion, removal) that follows the lead-in
  `meta` row `lead_in` (the scenario that is run first, as one continuous history, up to that
  time).
* Interpolation rule: `linear` between breakpoints, `step` for a jump, the last value held. The step
  average of the solver (F32) treats both exactly.
* `source_ids` lists register row ids (condition, burnup, temperature, geometry, diffusivity or
  release rows) that every strato-data check resolves. `basis` is one of `registered source row`,
  `derived from published AGR-1 burnup (calculated)`, `prescribed input (assumed)` (with a reason in
  `text`) and, for steps, `numerical setting (chosen; refinement-checked per Section 3.5)`.
* The metadata rows `nuclides_meaningful` and `nuclides_not_meaningful` state for which nuclides the
  scenario is meaningful: Sr-90 and Cs-137 have a registered cumulative yield and decay constant;
  Ag-110m, Cs-134 and Eu-154 get no direct fission-yield source term (decision 15).

* The `rate_density_reference` rows of the exploration variant list, besides IRR-097 (the
  temperature), the rows of the burnup series of compact 6-2-1 (`IRRB-` and cycle-end `IRR-` rows)
  from which the fission-rate density is derived; the base rows list the rows of the Capsule 3
  derivation that the data layer resolves.
* The `cycles` variant runs on the three-digit rates printed in its rows (units of 1e19
  m^-3 s^-1). The check `SC.cycles` confirms that they equal the series derived from the burnup
  register to those digits; the run does not use the unrounded derived values.
* Magnitudes of results (release fractions, inventories) are not stated in these files. They are
  results of the runs and are written, with a resolution flag, to the result files.

## Checks

`strato-data check` (tools workspace) runs the checks `SC.*`: row ids, units, monotone times,
resolution of every source id, labelling of every assumed number, coverage of the step plan, the
nuclide rules, the exploration variant, repeated register values and the per-cycle rates against
the series derived from the burnup register.
