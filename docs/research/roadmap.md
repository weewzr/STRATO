# Brick-by-brick roadmap

STRATO grows one verified physical mechanism at a time. This page lists the nine bricks of the development sequence, what each adds, what has to be researched, derived, implemented and verified before its status may advance, and its present status. A status describes a mechanism, not a number, and uses the five levels of the research-status system: PENDING, RESEARCHED, IMPLEMENTED, VERIFIED, VALIDATED. Per-mechanism statuses are held in `data/research/mechanisms.csv`. The brick status below summarises the rows of that file on 2026-11-24. For bricks 1 to 3 these are the rows of the equation builder's mechanisms, whose statuses rest on the BTCS verification (brick 1) and the Phase 2 derivations (bricks 2 and 3); brick 3 also holds the PENDING row for interface partition coefficients, which are not in the model. A brick status changes only when its evidence changes and the file records it.

Every future phase on this page is described only as planned. No phase is dated and no result is promised. Nothing is VALIDATED: no computed model-to-measurement comparison exists yet, and no experimental-agreement figure is written anywhere until such comparisons are computed at each temperature from the public data (AGR-1 safety tests, CRP-6 cases) after Phase 2.2 is implemented and verified.

## Status gates (all bricks)

| To reach | What must exist in the repository |
|---|---|
| RESEARCHED | literature with checked primary sources, candidate equations and the parameter requirements documented; as applied in this project (bricks 2 and 3), that means a STRATO derivation (dimensions, interface conditions, limiting cases) and the required parameters registered with sources, so that the `evidence` column of the mechanism row can point to them. A catalogue entry alone does not raise a status |
| IMPLEMENTED | solver code for the derived mechanism |
| VERIFIED | documented conservation, convergence and limiting-case tests that pass, with independent review |
| VALIDATED | predictions compared with independent experimental evidence within a stated applicability domain |

## Summary

| Brick | Adds | Status on 2026-11-24 | Delivered by |
|---|---|---|---|
| 1 | Baseline: layered diffusion, equation presentation and visualisation | VERIFIED on the synthetic five-layer case | web Phases 1 to 5 |
| 2 | Source and decay terms | RESEARCHED | Phase 2.1 (web Phase 6.1) |
| 3 | Temperature- and layer-dependent transport, AGR-1 geometry and materials | RESEARCHED | Phase 2.2 (web Phase 6.2), then the comparison with public data |
| 4 | Silver trapping and detrapping | PENDING | catalogue entry only |
| 5 | Silver grain-boundary transport | PENDING | catalogue entry only |
| 6 | Palladium-assisted degradation | PENDING | catalogue entry only |
| 7 | Caesium retention and barrier integrity | PENDING | catalogue entry only |
| 8 | Irradiation and coating degradation | PENDING | catalogue entry only |
| 9 | Additional species and downstream transport | PENDING | catalogue entry only |

## Brick 1: baseline demonstration

- **Adds.** The governing equation of STRATO, layered spherical diffusion with a constant diffusivity per layer, shown first, with the live simulation of the verified synthetic case under it, solved by Method 1 (finite-difference BTCS) and, where it applies, by Method 2 (the analytical Sturm-Liouville solution).
- **Evidence.** BTCS verification: the finite-difference solution against the analytical series, the observed orders, the balance and the regression tests (`tests/btcs_verification.rs`, `tests/btcs_regression.rs`, `tests/btcs_properties.rs`, golden files in `tests/golden/btcs/`), and the formulation in `docs/mathematics/btcs/`.
- **Status.** VERIFIED for the synthetic five-layer case (`inputs/synthetic/synthetic_five_layer.toml`) only. The synthetic diffusivities are test values, not material data; the AGR-1 particle is shown as geometry only and is never combined with them.
- **To advance to VALIDATED.** Not applicable to a synthetic case. Validation comes with brick 3.

## Brick 2: source and decay terms

- **Adds.** Fission-product generation from the fission rate density with cumulative yields, radioactive decay, and the inventory balance (start plus generated, minus decayed, minus released, minus remaining).
- **Researched.** Derivation (P21.1), (P21.8), (P21.42), (P21.58) in `docs/mathematics/STRATO_Phase2_Derivations.md`, pseudocode in `docs/mathematics/STRATO_Phase2_Pseudocode.md`, half-lives in `data/nuclear/decay.csv`, yields in `data/nuclear/fission_yields.csv`. First nuclides chosen by the owner: Cs-137 and Sr-90, then Kr-85 with its precursor-delay caveat; I-131 deferred because no PyC or SiC iodine diffusivity is registered; Ag-110m later (activation products deferred). See [caesium](species-cs.md), [strontium](species-sr.md), [krypton](species-kr.md), [iodine](species-i.md).
- **Still to do before IMPLEMENTED.** Rust code for the unsplit Backward Euler step with implicit decay and the exact step-average source, kernel-only source by default, through the full BTCS regression suite.
- **Still to do before VERIFIED.** The Phase 2.1 verification cases (conservation of the inventory balance, convergence, limiting cases) passing, and independent review.
- **Status.** RESEARCHED (derived and registered; no solver code).

## Brick 3: temperature- and layer-dependent transport

- **Adds.** Arrhenius diffusivities per layer with one or two branches (the TECDOC-978 FRG set, SI gas constant with a selectable 8.3143), temperature histories (reconstructed AGR-1 safety-test plans and CRP-6 schedules, each labelled with its evidence class), and AGR-1 geometry with registered materials.
- **Researched.** Derivation (P22.1) to (P22.15) and the fluence factor (P22.9) in `docs/mathematics/STRATO_Phase2_Derivations.md`; diffusivity rows in `data/diffusivity/arrhenius.csv`; histories in `data/temperature/temperature_histories.csv`; measurements in `data/experimental/release_measurements.csv`.
- **Known gaps.** Kernel rows are for UO2 and carry a label when used for UCO; the FRG set has no krypton or xenon row for SiC; the fluence-cutoff conversion for AGR-1 is not verified; the history format and the reference temperature of the effective time are still to be settled before Phase 2.2 code needs them. AGR-1 correction factors are never defaults.
- **Still to do before IMPLEMENTED and VERIFIED.** Code with recomputed conductances per step, the Phase 2.2 verification cases, independent review; it starts only after Phase 2.1 has passed.
- **To reach VALIDATED.** Computed comparisons at each temperature with the AGR-1 safety tests and the CRP-6 cases, within a stated applicability domain.
- **Status.** RESEARCHED.

## Brick 4: silver trapping and detrapping

- **Adds.** A trapped-silver population in SiC coupled to the mobile one ([entry](ag_trapping_detrapping.md)).
- **Before RESEARCHED.** Measured or computed trap densities and energies for silver in AGR-1 SiC (none found), registered with sources, and a derivation of the coupled system on STRATO's spherical finite-difference grid.
- **Before IMPLEMENTED and VERIFIED.** Bricks 2 and 3 for Ag-110m first (activation source, decay, temperature dependence); then code with conservation and limiting-case tests.
- **Status.** PENDING.

## Brick 5: silver grain-boundary transport

- **Adds.** A microstructure-dependent effective silver diffusivity in SiC ([entry](ag_grain_boundary_transport.md)).
- **Before RESEARCHED.** Registered grain sizes per AGR-1 variant and the fit coefficients of the mesoscale model, with the disagreement on the correction factor recorded, and a derivation of the diffusivity law inside Phase 2.2's temperature machinery.
- **Before IMPLEMENTED and VERIFIED.** Code and tests, after brick 3.
- **Status.** PENDING.

## Brick 6: palladium-assisted degradation

- **Adds.** A palladium penetration front in SiC and a change of layer state ([entry](pd_assisted_degradation.md)).
- **Before RESEARCHED.** A palladium source (no palladium nuclide or yield is registered), a local attack model tied to IPyC cracking, a quantitative Ag-Pd law (none found), and a derivation of the layer-state switch with conservation across it.
- **Before IMPLEMENTED and VERIFIED.** Code and tests.
- **Status.** PENDING.

## Brick 7: caesium retention and barrier integrity

- **Adds.** A comparison of caesium release with experiment that combines particles with intact and failed SiC ([entry](cs_retention_barrier_integrity.md)).
- **Before RESEARCHED.** Registered failure counts and times per AGR-1 compact with sources, registered fluences per compact, a verified fluence-cutoff conversion, and a derivation of the population combination and the failed-layer switch.
- **Before IMPLEMENTED and VERIFIED.** Bricks 2 and 3 for Cs-137 first; then code with conservation tests.
- **Status.** PENDING.

## Brick 8: irradiation and coating degradation

- **Adds.** Defect evolution, microcracking and coating failure, and their effect on transport ([entry](coating_degradation_failure.md)).
- **Before RESEARCHED.** Registered material correlations for irradiated PyC and SiC (dimensional change, creep, Weibull strength), fission-gas pressure models, fluence and flux histories, a defect-to-transport law beyond the empirical caesium fluence factor, and a derivation of a coupled mechanical model or of a documented data-driven failure input.
- **Before IMPLEMENTED and VERIFIED.** Code and tests; STRATO has no mechanical model today.
- **Status.** PENDING.

## Brick 9: additional species and downstream transport

- **Adds.** Europium, iodine, ruthenium and xenon, species-specific behaviour of strontium and krypton beyond the Phase 2 baseline, retention outside the particle, and downstream transport in FLiBe and a cover gas ([entry](additional_species_downstream_transport.md)).
- **Before RESEARCHED.** Register rows (decay, yields, diffusivities) for the missing nuclides and layers listed in the species entries ([europium](species-eu.md), [iodine](species-i.md), [ruthenium](species-ru.md), [xenon](species-xe.md)) and a derivation of a matrix region or documented outer boundary condition; for the downstream part, the data named below and a derived transport model for that environment.
- **Before IMPLEMENTED and VERIFIED.** Bricks 2 and 3 for each nuclide; code and tests, the downstream model separately.
- **Status.** PENDING.

## Two items among the future phases

Both are planned for a future phase. Their status stays PENDING, and nothing is shown for them until the material named below exists.

- **Materials Studio calculations** (the [gallery](materials_studio_gallery.md)). Needs genuine BIOVIA Materials Studio screenshots or exported structure files supplied by the owner, each with its computational method and simulation conditions. No imitation image is made in the meantime.
- **FLiBe and cover-gas transport** (brick 9). Needs FLiBe or cover-gas data from a cited source and a transport model for that environment, kept separate from the gas-cooled AGR conditions. The AGR irradiations were made in capsules swept by an inert gas [ref:davenport2016]; the nearest salt experience read for this catalogue, the Molten Salt Reactor Experiment, dissolved the fuel in the salt [ref:compere1975], and a functional review of fluoride-salt-cooled reactors starts the identification of the gaps that still hinder a mechanistic source term [ref:andrews2021].
