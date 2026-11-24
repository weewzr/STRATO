# Caesium (Cs)

Species entry of the STRATO research catalogue. Transport regimes: A, B and C (Regime B through irradiation and palladium attack, which lead to the Regime C failures). Reference fuel: AGR-1 UCO TRISO. STRATO does not simulate caesium today; nothing on this page is a STRATO result.

## Why it matters

Caesium is the species whose retention best shows whether the SiC layer is intact. Past experience with TRISO fuel shows that the SiC layer is the primary containment of caesium, so caesium release is an effective indicator of SiC failure; caesium diffuses with relative ease through intact pyrocarbon but is effectively retained by intact SiC [ref:demkowicz2016]. Caesium and strontium both have isotopes that would be radiologically hazardous if released after an accident in a reactor using TRISO fuel [ref:collin2015].

In the AGR-1 irradiation the Cs-134 fractional release from the compacts was below 3 × 10⁻⁶ in the four capsules whose particles all kept intact SiC, and about 10⁻⁵ in the two capsules that held particles with failed SiC; an estimated four of about 2.98 × 10⁵ particles failed their SiC in pile, an in-pile SiC failure fraction of 1.3 × 10⁻⁵ (at most 3.1 × 10⁻⁵ at 95 % confidence) [ref:demkowicz2016]. In the post-irradiation safety tests at 1600 to 1800 °C, caesium release from compacts without failed SiC stayed below 5 × 10⁻⁶ after 300 h at 1600 °C or 100 h at 1800 °C, and the overall caesium release was dominated by the SiC defect and failure fractions [ref:morris2016].

Isotopes (half-lives from the repository's nuclear-data register `data/nuclear/decay.csv`, ENSDF adopted values, unless marked otherwise):

| Nuclide | Stable or radioactive | Half-life | Origin |
|---|---|---|---|
| Cs-133 | stable (isotopic abundance 100 %) | none | cumulative fission product via Xe-133; neutron-capture target for Cs-134 (register) |
| Cs-134 | radioactive, β− 99.9997 %, EC 0.0003 % | 2.0652 y | activation product Cs-133(n,γ), shielded from direct fission yield by Xe-134 (register) |
| Cs-134m | radioactive, IT 100 % to Cs-134 | 2.912 h | Cs-133(n,γ) partial; tiny independent fission yield (register) |
| Cs-135 | radioactive, β− 100 % | 2.3 × 10⁶ y (uncertainty 0.3 × 10⁶ y); not in the register [ref:iaea_livechart_fp] | long-lived fission product; listed for completeness |
| Cs-137 | radioactive, β− 100 % (94.70 % to Ba-137m) | 30.08 y | cumulative fission product via I-137 and Xe-137 (register) |
| Ba-137m | radioactive, IT 100 % | 2.552 min | decay daughter of Cs-137 (register) |

The register decay constant of Cs-137 is 7.302187 × 10⁻¹⁰ s⁻¹. Cs-134 is an activation product: its source needs a neutron-capture rate of Cs-133, and activation products are deferred by owner decision D3. Cs-134 is nevertheless the isotope the AGR-1 examinations report for release, because its shorter half-life makes it less affected by hot-cell contamination than Cs-137 [ref:demkowicz2016] [ref:morris2016].

## Relevant TRISO layers

- **Kernel.** The register holds caesium kernel diffusivities only for UO2 (`data/diffusivity/arrhenius.csv#A1-01`, `#A1-02`, two branches, from [ref:iaea1997]); Collin et al. used the same UO2 values for the AGR-1 UCO kernel for lack of UCO data (register rows `C16-05`, `C16-06`). Using them for AGR-1 is an assumption that must be labelled (rule E3). PARFUME comparisons with AGR-1 suggest the caesium diffusivity in UCO may be overestimated by a factor of up to 250 [ref:collin2015].
- **Buffer.** Treated as non-retentive in the literature data sets (register rows `BUF-01`, `C16-22`).
- **IPyC and OPyC.** Caesium passes intact pyrocarbon with relative ease [ref:demkowicz2016]; register rows `A2-01`, `A2-06`, `A2-11`, `A2-12` (several national fits).
- **SiC.** The barrier on which caesium retention depends. Register rows `A3-01`, `A3-02` (FRG, two branches, the first with a fast-fluence factor), `A3-06` to `A3-09` (USA, lower and upper curves), `A3-13`, `A3-14` (Japan), `A3-20` (Russian Federation) [ref:iaea1997].
- **OPyC and compact matrix.** Caesium released from particles with failed SiC was partly retained in the OPyC and matrix: the Cs-134 inventory found outside the SiC in as-irradiated compacts was on average slightly higher than the inventory released from the compacts [ref:demkowicz2016]. Particles with failed SiC released between 0.38 and 0.65 of their Cs-134, so SiC is the most effective caesium barrier but not the only one [ref:demkowicz2016].

## Candidate release mechanisms

1. **Diffusion through intact SiC** (Regime A). Very slow; the measured release from intact particles is close to background [ref:morris2016].
2. **Release through a failed or defective SiC layer** (Regime C). In AGR-1 the caesium-releasing particles had IPyC cracks caused by the shrinking buffer, and the exposed SiC had suffered concentrated palladium attack; SiC failures during safety testing came either from fabrication defects or from palladium corrosion where similar IPyC cracks exposed the SiC [ref:hunn2016]. At 1600 °C the two particles that released caesium in the ORNL furnace tests were traced to as-fabricated coating defects; at 1700 and 1800 °C the failures analysed were palladium attack after IPyC fracture [ref:morris2016].
3. **Irradiation-dependent transport in SiC** (Regime B). The German (FRG) Cs-in-SiC diffusivity carries a fast-neutron-fluence factor [ref:iaea1997]; measurements of fission metals in irradiated SiC found the caesium (and cerium, europium, ruthenium, antimony) content proportional to the number of neutrons that had passed through the SiC [ref:iaea1997].
4. **A spread of particle behaviour.** IMGA measurements after a German heating test showed caesium release varying between 10 % and 90 % of the particle inventory with a bimodal distribution; the source discusses diffusion combined with a failure fraction, degraded SiC, or an inherent variance of diffusion coefficients as possible explanations [ref:iaea1997].

The catalogue entry for these mechanisms is [Caesium retention and barrier integrity](cs_retention_barrier_integrity.md) (brick 7). Coating failure itself is [brick 8](coating_degradation_failure.md).

## Current evidence

- **AGR-1 irradiation.** Cs-134 release below 3 × 10⁻⁶ from compacts with intact SiC; four particles with in-pile SiC failure; the caesium released in Capsules 5 and 6 came mostly from those particles [ref:demkowicz2016].
- **AGR-1 safety tests.** At 1600 °C, five of eight compacts released less than 10⁻⁵ Cs-134 (a few percent of one particle); each of the other three had one particle with failed SiC. Estimated SiC failures: 3 of about 33,100 particles at 1600 °C, about 4 of 8300 at 1700 °C and about 23 of 16,500 at 1800 °C. In this limited sample, the Variant 3 compacts (finer-grained SiC) released caesium later at 1800 °C [ref:morris2016].
- **Empirical diffusivities disagree.** TECDOC-978 states that the US reference model assumes no diffusion of caesium (and strontium) through SiC, while US, German, Japanese and Russian diffusion fits exist [ref:iaea1997]. A PARFUME comparison with AGR-1 irradiation data over-predicted caesium release by a factor of up to 3 for particles with failed SiC and up to 100 for intact particles, which the authors attribute to diffusivities overestimated in UCO (up to a factor of 250) and in SiC (about 40 % on average) [ref:collin2015]. For the safety tests the over-prediction for intact particles reached up to five orders of magnitude [ref:collin2016]. Collin et al. derived tentative correction factors; by owner decision D4 such AGR-1 correction factors are never used as STRATO defaults.
- **Atomistic.** Density-functional metadynamics at 1500 K gives caesium diffusion coefficients at β-SiC grain boundaries of 3.91 × 10⁻²³ to 2.15 × 10⁻²¹ m² s⁻¹, very much slower than experiment, which the authors read as a sign that a different mechanism operates [ref:rabone2015]. These are atomistic values for specific boundaries, not effective diffusivities of the coating.

## Proposed governing-equation extensions

**Phase 2 physics (owner's first choice).** STRATO's owner has chosen caesium (Cs-137), with strontium (Sr-90), as the first nuclides for the Phase 2 physics, with krypton (Kr-85) second (owner decision D1, 2026-11-24). The Phase 2 physics adds fission-product generation, radioactive decay and temperature-dependent diffusivity to the layered diffusion equation. It is RESEARCHED and not yet implemented: it is derived in `docs/mathematics/STRATO_Phase2_Derivations.md` and its parameters are registered, but the solver has no code for it. The derived equation is, as a research concept not solved by STRATO today ((P21.1) of the derivations):

$$
\frac{\partial c_k}{\partial t} = \frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_{i,k}(T)\,\frac{\partial c_k}{\partial r}\right) - \lambda_k c_k + S_{i,k}(r,t)
$$

with the cumulative-yield source of (P21.8) and, for Cs-137, $\lambda = 7.302187\times10^{-10}\ \mathrm{s^{-1}}$ from the register. Stable Cs-133 has no decay term. For caesium in SiC the FRG data set (owner decision D4) uses two Arrhenius branches, the first with the fluence factor of TECDOC-978 Table A-3 [ref:iaea1997] ((P22.9) of the derivations; research concept):

$$
D_{\mathrm{SiC}}(T,\Gamma) = 5.5\times10^{-14}\,e^{\Gamma/5}\exp\!\left(-\frac{125\ \mathrm{kJ\,mol^{-1}}}{R\,T}\right) + 1.6\times10^{-2}\exp\!\left(-\frac{514\ \mathrm{kJ\,mol^{-1}}}{R\,T}\right)\ \mathrm{m^2\,s^{-1}}
$$

with $\Gamma$ the fast-neutron fluence in $10^{25}\ \mathrm{m^{-2}}$ ($E > 16$ fJ), which the source marks as reactor-specific [ref:iaea1997]. The conversion of the AGR-1 fluence (reported for $E > 0.18$ MeV) to this cutoff is recorded as not verified in the derivations.

**Beyond Phase 2 (brick 7).** A particle-population description that combines intact particles with particles whose SiC has failed, as PARFUME does for AGR-1 [ref:collin2015]; see [Caesium retention and barrier integrity](cs_retention_barrier_integrity.md).

## Potential Materials Studio calculations

Planned atomistic calculations for caesium are listed in the [Materials Studio gallery](materials_studio_gallery.md) (item `ms-cs-sic-gb`: caesium at SiC grain boundaries and in the bulk). None has been performed. An ideal 3C-SiC cell would not represent the polycrystalline AGR-1 coating [ref:kirchhofer2013], and atomistic barriers or diffusion coefficients are not effective diffusivities; the atomistic caesium values already published disagree with experiment by orders of magnitude [ref:rabone2015].

## Experimental comparison opportunities

- **AGR-1 safety tests (reference fuel).** Cs-134 release curves at 1600, 1700 and 1800 °C for 14 compacts with burnup, fluence and irradiation temperature per compact [ref:morris2016]; the register holds digitised and printed values as rows with nuclide `Cs-134` in `data/experimental/release_measurements.csv` (sources Morris et al. (2016) and Collin et al. (2016)). Most measured caesium release comes from individual particles with failed SiC, so a comparison needs the failure count per compact, not only a single-particle model.
- **AGR-1 irradiation.** Capsule-level Cs-134 release and inventories retained outside the SiC [ref:demkowicz2016].
- **CRP-6 and JAERI heating tests (UO2 fuel).** Rows `E-150`, `E-153`, `E-154` (CRP-6) and the JAERI Cs-134 and Cs-137 rows of the same register; these are UO2 TRISO, not the reference fuel.

Agreement percentages are not given here; they are computed at each temperature only after Phase 2.2 is implemented and verified. The AGR experiments were gas-cooled (capsules swept by an inert gas in the Advanced Test Reactor) [ref:davenport2016]; transport of caesium in FLiBe or a cover gas does not exist in STRATO today and is planned for a future phase; it is not compared with AGR data.

## Research limitations

- Intact-particle caesium release is close to background and hot-cell contamination, so it often gives only an upper limit [ref:morris2016] [ref:iaea1997].
- The effective Cs-in-SiC diffusivities differ between data sets, and AGR-1 comparisons point to overestimated diffusivities [ref:collin2015] [ref:collin2016].
- The fluence factor of the FRG fit is reactor-specific and its fluence cutoff differs from the one in which AGR-1 fluences are reported [ref:iaea1997].
- Kernel diffusivities in the register are for UO2, not UCO (rule E3).
- The Cs-134 source needs the Cs-133 capture cross section, which is not registered (decision D3).

## Implementation status

Owner's Phase 2 choice: Cs-137 generation, decay and temperature-dependent diffusivity are RESEARCHED and not yet implemented (see above); their status rows are written by the Phase 2 builder work. The caesium-specific mechanism of this catalogue is held in `data/research/mechanisms.csv` (row `cs_retention_barrier_integrity`, species `Cs`). STRATO's live simulation does not include caesium.
