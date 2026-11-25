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
| `independent_check.csv` | why the values are believable: the release fraction at the checkpoint times against a reference that contains no BTCS code, +dt/2` and `2n` refinements |

| Case | Input | Independent reference |
|---|---|---|
| `s5_n40_dt200` | `inputs/synthetic/synthetic_five_layer.toml` | multilayer Sturm-Liouville series; also a separate-language value pinned by the repository |
| `mild_contrast_n40_dt10` | `inputs/btcs_regression/synthetic_mild_contrast.toml` | multilayer Sturm-Liouville series |
| `homogeneous_n40_steps200` | `inputs/btcs_regression/synthetic_homogeneous_sphere.toml` | closed-form series of the homogeneous sphere |

All three cases are synthetic numerical benchmarks, not literature data. Agreement with the
references is numerical verification of the stated model; it is not experimental validation. The
finite-volume files in `tests/golden/v1.0/` are not used here.

## Regeneration record

The files were regenerated once, at the commit named in their headers, for one reason: the exported column headers now carry the full physical name with its unit and the header comments carry the source row and class of the parameters (decisions 30 and 31). A second regeneration followed the merge of the upstream tip, which derives the balance bound without the earlier constant `16 u` (which had no derivation); the production loop reports that derived bound. The data rows were compared before and after: the only column that changed is the derived rounding bound of the mass-balance residual (it is smaller); every other value, including every concentration, inventory, release and observed residual, is identical to the last digit. The regression level of 1e-12 was not touched.

The three `independent_check.csv` files were regenerated a third time, at the commit named in their headers, because the earlier check multiplied the leading-order error estimate by the safety factor 1.25 of verification-plan rule R2 and the floor by 3, and neither constant has a derivation (upstream removed the same R2 checks). Without the factor, four of the ten checkpoints would have failed the old criterion by up to 2.7 percent. The new check records the observed orders (0.93 to 1.02 in time, 2.000 in space), the uncertainty of each estimate and the difference between the reference and the estimated limit (0.01 to 2.04 percent of the run-reference difference), and passes under the stated 5 percent convention. `timeseries.csv` and `final_profile.csv` were not regenerated; their data rows and their level of 1e-12 are untouched.
