# Krypton (Kr)

Species entry of the STRATO research catalogue. Transport regimes: A and C. Reference fuel: AGR-1 UCO TRISO. STRATO does not simulate krypton today; nothing on this page is a STRATO result.

## Why it matters

Krypton is the fission gas used to detect TRISO coating failure: released fission gases such as Kr-85 are taken as the indicator of a broken coating, because an intact SiC layer keeps the particle gas-tight and an intact OPyC significantly delays the gas [ref:iaea1997]. In the AGR-1 safety tests Kr-85 release stayed below 5 × 10⁻⁶ in every test at 1600 °C and 1700 °C; particles with failed SiC but intact outer pyrocarbon retained most of their krypton, and krypton release from TRISO failures was observed only after about 210 h at 1800 °C in one compact, where two particles released essentially all of their inventory [ref:morris2016].

Isotopes (register `data/nuclear/decay.csv` unless marked otherwise):

| Nuclide | Stable or radioactive | Half-life | Origin |
|---|---|---|---|
| Kr-84, Kr-86 | stable (isotopic abundances 56.987 %, 17.279 %); not in the register [ref:iaea_livechart_fp] | none | listed for completeness |
| Kr-85 | radioactive, β− 100 % to Rb-85 | 10.739 y | fission product; small independent yield, mostly via Kr-85m (IT 21.2 %) and Br-85 (register) |
| Kr-85m | radioactive, β− 78.8 %, IT 21.2 % to Kr-85 | 4.480 h | fission product (register) |
| Kr-88 | radioactive, β− 100 % | 2.825 h (uncertainty 0.019 h); not in the register [ref:iaea_livechart_fp] | measured in the AGR-1 sweep gas [ref:scates2010] |

The register decay constant of Kr-85 is 2.045347 × 10⁻⁹ s⁻¹. Kr-89 and Kr-90 appear in `data/nuclear/fission_yields.csv` only as precursors of Sr-89 and Sr-90.

## Relevant TRISO layers

- **Kernel.** TECDOC-978 rows for iodine, krypton and xenon in UO2 (`A1-05` normal operation; `A1-06`, `A1-07` accident conditions) [ref:iaea1997]; Collin et al. used the accident-condition rows for the UCO kernel (`C16-16`, `C16-17`); rule E3 applies.
- **IPyC and OPyC.** FRG two-branch rows `A2-04`, `A2-05` (krypton and xenon), USA rows `A2-09`, `A2-10`, Russian row `A2-14` [ref:iaea1997]. Pyrocarbon matters for krypton: particles with failed SiC kept most of their krypton while one pyrocarbon layer stayed intact [ref:morris2016].
- **SiC.** The FRG data set chosen by owner decision D4 has no krypton (or xenon) row for SiC. Collin et al. used the Japanese xenon-in-SiC row for krypton (register row `C16-20`, equal to `A3-17`); this is a register data gap for Phase 2.2.
- **Matrix.** Russian row `A4-14`; Collin et al. used the TECDOC-978 "iodine in pores" row for the krypton matrix value (`C16-21`); outside STRATO's particle domain.

## Candidate release mechanisms

1. **Release from particles whose three dense layers have failed** (Regime C): rapid, about 100 % of a particle's inventory in the AGR-1 case at 1800 °C [ref:morris2016].
2. **Diffusion through particles with failed SiC and intact pyrocarbon**: lower but detectable Kr-85 at 1800 °C may have come this way [ref:morris2016].
3. **Release from uranium contamination** outside the particles: during the AGR-1 irradiation the Kr-85m R/B stayed mostly below 10⁻⁷ (Capsules 5 and 6 near 2 × 10⁻⁷ at the end, rising with fission power and temperature), consistent with contamination and with no particle failures [ref:scates2010].
4. **Diffusion and recoil from intact particles** (Regime A), represented in R/B models by the Booth equivalent-sphere model [ref:scates2010].

The population form of the release (failed particles plus contamination) is written in the [xenon entry](species-xe.md); coating failure is [brick 8](coating_degradation_failure.md).

## Current evidence

- **AGR-1 safety tests:** see above [ref:morris2016]; register rows with nuclide `Kr-85` in `data/experimental/release_measurements.csv` (sources Morris et al. (2016) and Collin et al. (2016)).
- **Model comparison:** PARFUME over-predicted the safety-test krypton release by up to two orders of magnitude for compacts with intact particles and by more than one for compacts with failed SiC; no correction factors could be derived because all coating layers are partly retentive for krypton and the data did not resolve the retention per layer [ref:collin2016]. By owner decision D4, AGR-1 correction factors are never STRATO defaults.
- **AGR-1 irradiation:** zero TRISO failures inferred from low fission-gas R/B [ref:demkowicz2016] [ref:scates2010].

## Proposed governing-equation extensions

**Phase 2 physics (owner's second choice).** STRATO's owner has chosen krypton (Kr-85) as the second nuclide for the Phase 2 physics, after Cs-137 and Sr-90, with a precursor-delay caveat (owner decision D1, 2026-11-24). Generation, radioactive decay and temperature-dependent diffusivity are RESEARCHED and not yet implemented: derived in `docs/mathematics/STRATO_Phase2_Derivations.md` ((P21.1), (P21.8), (P22.1)), parameters registered, no solver code. The caveat: Kr-85 is fed mostly through Kr-85m (4.480 h), and the derivations give 29.8 h for that feed to reach 99 % of equilibrium, so the instantaneous cumulative-yield source of (P21.8) runs ahead of the true Kr-85 birth for about a day after a source change. As a research concept not solved by STRATO today:

$$
\frac{\partial c}{\partial t} = \frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_i(T)\,\frac{\partial c}{\partial r}\right) - \lambda_{\mathrm{Kr\text{-}85}}\,c + \frac{\dot F\,y^{C}_{\mathrm{Kr\text{-}85}}}{N_A}\,\chi_1(r)
$$

with $\lambda_{\mathrm{Kr\text{-}85}} = 2.045347\times10^{-9}\ \mathrm{s^{-1}}$ and the kernel-only source $\chi_1$ (owner decisions D9 and D13). Stable Kr-84 and Kr-86 have no decay term.

**Beyond Phase 2 (brick 9).** Failed-particle and contamination release terms, recoil, and an explicit Kr-85m precursor if the caveat matters; see [Additional species and downstream transport](additional_species_downstream_transport.md).

## Potential Materials Studio calculations

Planned: krypton (with xenon and iodine) at vacancy sites and small cavities in 3C-SiC (gallery item `ms-noble-gas-sic`). Not performed; Pending placeholder only.

## Experimental comparison opportunities

- **AGR-1 safety tests (reference fuel):** Kr-85 release at 1600, 1700 and 1800 °C [ref:morris2016], register rows as above. Below 1800 °C the values are near or below detection, which bounds a model rather than testing it.
- **AGR-1 irradiation:** Kr-85m and Kr-88 R/B histories [ref:scates2010]; not in the measurement register.

No agreement figure is given before Phase 2.2 is implemented and verified. The AGR experiments were gas-cooled [ref:davenport2016]; krypton in FLiBe or a cover gas does not exist in STRATO today and is planned for a future phase.

## Research limitations

- No FRG krypton row for SiC (data gap for Phase 2.2).
- The precursor delay makes an instantaneous Kr-85 source approximate.
- Krypton release from intact AGR-1 fuel is near detection limits [ref:morris2016].
- Kr-88 and the stable isotopes are not in the decay register.

## Implementation status

Owner's Phase 2 choice (second): Kr-85 generation, decay and temperature-dependent diffusivity are RESEARCHED and not yet implemented; their status rows are written by the Phase 2 builder work. Krypton-specific mechanisms of this catalogue are held in `data/research/mechanisms.csv` (`additional_species_downstream_transport`, species `Kr`). STRATO's live simulation does not include krypton.
