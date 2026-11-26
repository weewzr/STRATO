# Analytical series reference

`series_synthetic_five_layer.csv` holds the release fraction `F(t)` of the layered Sturm-Liouville
series (Method B) for `inputs/synthetic/synthetic_five_layer.toml`, at the 100 output times of that
input. The values are the series column of the comparison file that version 1.0.1 of the command
line wrote (`comparison_fv_vs_analytical.csv`); they are copied unchanged, digit for digit, from
that file (git tag `v1.0.1`, `tests/golden/v1.0/`). The finite-volume column of that file, and the
other finite-volume files, are not kept: the finite-volume solver is retired (see
`docs/archive/finite-volume/README.md`).

`tests/analytical_series_regression.rs` recomputes the series with the current code and compares
at the relative level 1e-12, for times above 2e4 s (earlier values lie below the validity time of
the series, where it is not resolved). The independent anchor of the series is the separate-language
value in `tests/analytical_reference.rs` and the Python reference check
`scripts/derivation_checks/sec1_synthetic_case.py`.

This is a regression reference, not truth. Agreement is numerical verification of the stated
model; it is not experimental validation.
