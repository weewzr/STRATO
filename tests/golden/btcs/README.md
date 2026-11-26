# BTCS regression reference files

These files are **regression references, not truth**. Each one is the output of the BTCS solver
itself, produced by `cargo run --release --example btcs_regression -- --write tests/golden/btcs
--commit <hash>` at the commit written in its header. They are never edited by hand. A mismatch in
`tests/btcs_regression.rs` means that the code changed; it needs a recorded reason before the files
are regenerated.

Each case directory holds:

| File | Content |
|---|---|
| `timeseries.csv` | inventory, cumulative release, release fraction, release rate, balance residual with its derived bound, concentration range, at the output times of the input |
| `final_profile.csv` | nodal concentration at the end time, with node class and material |
| `independent_check.csv` | why the values are believable: the release fraction at the checkpoint times against a reference that contains no BTCS code, with the observed orders (asserted to round to 1 in time and 2 in space), the limit estimated from the run's own `dt/2`, `dt/4`, `dt/8`, `2n`, `4n`, `8n` and `(dt/2, 2n)` refinements, and the difference between that limit and the reference under a stated 5 percent convention |

| Case | Input | Independent reference |
|---|---|---|
| `s5_n40_dt200` | `inputs/synthetic/synthetic_five_layer.toml` | multilayer Sturm-Liouville series; also a separate-language value pinned by the repository |
| `mild_contrast_n40_dt10` | `inputs/btcs_regression/synthetic_mild_contrast.toml` | multilayer Sturm-Liouville series |
| `homogeneous_n40_steps200` | `inputs/btcs_regression/synthetic_homogeneous_sphere.toml` | closed-form series of the homogeneous sphere |

All three cases are synthetic numerical benchmarks, not literature data. Agreement with the
references is numerical verification of the stated model; it is not experimental validation. The
finite-volume files of version 1.0 are retired (see `docs/archive/finite-volume/README.md`) and are not used here.

## Regeneration record

The files were regenerated once, at the commit named in their headers, for one reason: the exported column headers now carry the full physical name with its unit and the header comments carry the source row and class of the parameters (decisions 30 and 31). A second regeneration followed the merge of the upstream tip, which derives the balance bound without the earlier constant `16 u` (which had no derivation); the production loop reports that derived bound. The data rows were compared before and after: the only column that changed is the derived rounding bound of the mass-balance residual (it is smaller); every other value, including every concentration, inventory, release and observed residual, is identical to the last digit. The regression level of 1e-12 was not touched.

The three `independent_check.csv` files were regenerated a third time, at the commit named in their headers, because the earlier check multiplied the leading-order error estimate by the safety factor 1.25 of verification-plan rule R2 and the floor by 3, and neither constant has a derivation (upstream removed the same R2 checks). Without the factor, four of the ten checkpoints would have failed the old criterion by up to 2.7 percent. The new check records the observed orders (0.93 to 1.02 in time, 1.999 to 2.000 in space), the uncertainty of each estimate and the difference between the reference and the estimated limit, and passes under the stated 5 percent convention. `timeseries.csv` and `final_profile.csv` were not regenerated for this; their data rows and their level of 1e-12 are untouched.

The three `independent_check.csv` files were regenerated a fourth time, at the commit named in their headers, for two reasons. First, the cross term of the limit estimate had the wrong sign: the time estimate at fixed mesh and the space estimate at fixed step each already contain the cross term `c dt h^2` of the error expansion `F* + a dt + b h^2 + c dt h^2`, so the limit is `F_0 - e_t - e_h + e_x`, not `F_0 - e_t - e_h - e_x` (a unit test on an exact synthetic expansion now recovers `F*` to rounding). Only the columns of the limit estimate, of the limit difference and of its ratio to the tolerance changed; the run values, the references, the orders, the uncertainties and the tolerances are identical to the last digit. Second, the check now asserts that the observed orders round to 1 in time and 2 in space (coordinator decision 49), which does not change a value. With the corrected sign the difference between the reference and the estimated limit is 0.008 to 1.91 percent of the run-reference difference (the largest limit difference over tolerance is 0.38). The 5 percent is a convention chosen after the first measurement and is not changed.
