# Strontium (Sr)

Species entry of the STRATO research catalogue. Transport regimes: A and C. Reference fuel: AGR-1 UCO TRISO. STRATO does not simulate strontium today; nothing on this page is a STRATO result.

## Why it matters

Strontium, like caesium, has isotopes that would be radiologically hazardous if released after an accident in a reactor using TRISO fuel [ref:collin2015]. Unlike caesium, strontium passes intact TRISO coatings in small amounts during irradiation and is then largely held in the outer pyrocarbon and the compact matrix [ref:demkowicz2016].

In AGR-1 the capsule-average fractional release of Sr-90 from the compacts was about 8 × 10⁻⁷ to 3 × 10⁻⁵, while the inventory found outside the SiC in some compacts reached about 3 × 10⁻³, well above the inventory of one particle (2.4 × 10⁻⁴), which indicates appreciable release through intact coatings in certain compacts [ref:demkowicz2016].

Isotopes (register `data/nuclear/decay.csv`, ENSDF adopted values, unless marked otherwise):

| Nuclide | Stable or radioactive | Half-life | Origin |
|---|---|---|---|
| Sr-86, Sr-88 | stable (isotopic abundances 9.86 % and 82.58 %); not in the register [ref:iaea_livechart_fp] | none | listed for completeness |
| Sr-89 | radioactive, β− 100 % | 50.563 d | cumulative fission product via Kr-89 and Rb-89 (register) |
| Sr-90 | radioactive, β− 100 % to Y-90 | 28.91 y | cumulative fission product via Kr-90, Rb-90 and Rb-90m (register) |
| Y-90 | radioactive, β− 100 % to Zr-90 | 64.05 h | decay daughter of Sr-90 (register) |

The register decay constant of Sr-90 is 7.597709 × 10⁻¹⁰ s⁻¹. The repository's Phase 2 derivations choose the cumulative yield for Sr-90 because its longest-lived precursor (Rb-90m, 258 s) reaches 99 % of its equilibrium feed within about 0.48 h (`docs/mathematics/STRATO_Phase2_Derivations.md`, Section 3.3.2).

## Relevant TRISO layers

- **Kernel.** Register rows `A1-03` (FRG) and `A1-10` (USA) are for UO2 [ref:iaea1997]; Collin et al. used them for UCO (row `C16-11`); rule E3 applies. PARFUME comparisons suggest the strontium diffusivity in SiC, in UCO or in both is overestimated [ref:collin2015].
- **Buffer.** Non-retentive in the data sets (rows `BUF-01`, `C16-22`).
- **IPyC and OPyC.** Rows `A2-02` (FRG) and `A2-07` (USA) [ref:iaea1997].
- **SiC.** Rows `A3-03`, `A3-04` (FRG, two branches), `A3-10`, `A3-11` (USA), `A3-15` (Japan, reusing the KFA low-temperature branch) [ref:iaea1997]. The register notes that the source text says the combined fit underestimates SiC strontium retention at 1600 °C.
- **OPyC and compact matrix.** Strontium released through intact coatings was retained to a significant degree in the matrix or OPyC [ref:demkowicz2016]; after the safety tests it was found throughout the OPyC, matrix, graphite holder and furnace parts [ref:morris2016].

## Candidate release mechanisms

1. **Diffusion through intact coatings during irradiation** (Regime A), followed by retention in OPyC and matrix [ref:demkowicz2016].
2. **Slow release from the OPyC and matrix during heating.** In the AGR-1 safety tests strontium release paralleled europium release, rose slowly, and was not obviously coupled to SiC failure; the working hypothesis is that it came mainly from inventory deposited in the OPyC and matrix during irradiation [ref:morris2016].
3. **Diffusion through SiC at very high temperature.** Two Variant 3 compacts (fine-grained SiC) at 1800 °C showed increasing release rates that could possibly be linked to diffusion through SiC; the source says further study is required [ref:morris2016].
4. **Release through failed SiC** (Regime C). Hard to see experimentally because of the release from intact particles [ref:collin2015].

Species-specific mechanisms beyond the Phase 2 baseline are grouped in [Additional species and downstream transport](additional_species_downstream_transport.md) (brick 9).

## Current evidence

- **AGR-1 irradiation:** Sr-90 release 8 × 10⁻⁷ to 3 × 10⁻⁵ per capsule, retained mainly outside the SiC in the compact [ref:demkowicz2016].
- **AGR-1 safety tests:** six compacts at 1600 °C released slightly below to well above one particle's strontium, but only three of them had a particle with failed SiC; the greatest release came from a compact without failed SiC; recovering Sr-90 from furnace parts was difficult, so part of the spread may be measurement [ref:morris2016].
- **Model comparisons disagree with the data.** PARFUME over-predicted strontium release from intact AGR-1 particles during irradiation [ref:collin2015] and in the safety tests by more than three orders of magnitude for compacts with only intact particles and up to two for compacts with failed SiC; correction factors could not be derived for strontium because safety-test release could not be separated from matrix inventory [ref:collin2016].

## Proposed governing-equation extensions

**Phase 2 physics (owner's first choice).** STRATO's owner has chosen strontium (Sr-90), with caesium (Cs-137), as the first nuclides for the Phase 2 physics, with krypton (Kr-85) second (owner decision D1, 2026-11-24). Generation, radioactive decay and temperature-dependent diffusivity are RESEARCHED and not yet implemented: derived in `docs/mathematics/STRATO_Phase2_Derivations.md` ((P21.1), (P21.8), (P22.1)), with parameters registered, and no solver code. As a research concept not solved by STRATO today, the layer diffusivity is a sum of Arrhenius branches (TECDOC-978 Eq. (A-1) [ref:iaea1997]; (P22.1) of the derivations):

$$
D_i(T) = \sum_{m=1}^{m_i} D_{0,i,m}\exp\!\left(-\frac{Q_{i,m}}{R\,T}\right)
$$

with the FRG strontium rows (two branches in SiC, `A3-03` and `A3-04`) chosen by owner decision D4, and $\lambda_{\mathrm{Sr\text{-}90}} = 7.597709\times10^{-10}\ \mathrm{s^{-1}}$ in the decay term. The Y-90 daughter is not tracked in Phase 2 (parent-daughter chains are deferred).

**Beyond Phase 2 (brick 9).** Retention in and slow release from the OPyC and matrix, which the AGR-1 data suggest dominate heating-test release [ref:morris2016]; STRATO's particle model ends at the OPyC outer surface and has no matrix region.

## Potential Materials Studio calculations

Planned: strontium and europium substitutional and interstitial configurations in 3C-SiC and at a grain boundary (gallery item `ms-sr-eu-sic`). None has been performed; the calculation is planned for a future phase of the project, and the gallery shows a Pending placeholder only. An ideal crystal does not represent the polycrystalline coating [ref:kirchhofer2013].

## Experimental comparison opportunities

- **AGR-1 safety tests (reference fuel).** Sr-90 release at 1600, 1700 and 1800 °C [ref:morris2016]; register rows with nuclide `Sr-90` in `data/experimental/release_measurements.csv` (source Collin et al. (2016)). Any comparison has to separate inventory already in the matrix from release through the coatings, which the sources could not do [ref:collin2016].
- **AGR-1 irradiation.** Capsule-level release and compact inventories outside the SiC [ref:demkowicz2016].
- **CRP-6 (UO2 fuel).** Row `E-151` (HFR-K3/1 sphere, 1600 °C).

No agreement figure is given; agreement is computed at each temperature only after Phase 2.2 is implemented and verified. The AGR experiments were gas-cooled [ref:davenport2016]; strontium in FLiBe or a cover gas does not exist in STRATO today and is planned for a future phase.

## Research limitations

- The measured heating-test release mixes inventory deposited in the matrix during irradiation with release during heating [ref:morris2016] [ref:collin2016].
- Sr-90 recovery from furnace parts was incomplete in some tests [ref:morris2016].
- Kernel rows are for UO2, not UCO (rule E3).
- STRATO has no matrix region, so the retention that dominates the AGR-1 strontium data lies outside today's model domain.

## Implementation status

Owner's Phase 2 choice: Sr-90 generation, decay and temperature-dependent diffusivity are RESEARCHED and not yet implemented; their status rows are written by the Phase 2 builder work. Strontium-specific mechanisms of this catalogue are held in `data/research/mechanisms.csv` (row `additional_species_downstream_transport`, species `Sr`). STRATO's live simulation does not include strontium.
