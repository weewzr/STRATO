# Palladium (Pd)

Species entry of the STRATO research catalogue. Transport regimes: B and C. Reference fuel: AGR-1 UCO TRISO. STRATO does not simulate palladium today; nothing on this page is a STRATO result.

## Why it matters

Palladium has no isotope of consequence in terms of radiological hazard; its interest lies in the role it can play in degrading the SiC layer [ref:demkowicz2016]. Electron microprobe analysis identified palladium as a cause of SiC corrosion in irradiated coated particles, and temperature is the major factor in the corrosion rate [ref:tiegs1982]. In AGR-1, palladium corrosion was a key contributor to the SiC failures of particles that released caesium during irradiation and post-irradiation safety testing [ref:demkowicz2016] [ref:hunn2016]. Palladium is also discussed as a possible carrier of silver through SiC (see [Ag-Pd interaction](pd_assisted_degradation.md)).

Isotopes. No palladium nuclide is in the repository's nuclear-data register (`data/nuclear/decay.csv`); the values below come from the IAEA LiveChart of Nuclides (ENSDF adopted values) [ref:iaea_livechart] and are marked "(not in register)" in `data/research/species.csv`.

| Nuclide | Stable or radioactive | Half-life (evaluated source) |
|---|---|---|
| Pd-105 | stable | none |
| Pd-107 | radioactive, β− 100 % | 6.5 × 10⁶ y (uncertainty 0.3 × 10⁶ y) |
| Pd-108 | stable | none |
| Pd-110 | stable | none |

AGR-1 post-irradiation examination measured palladium through the stable isotope Pd-105 [ref:demkowicz2016]. With a half-life of millions of years, decay of Pd-107 is negligible over an irradiation; palladium is treated as a chemically active element rather than as a radiological source.

## Relevant TRISO layers

- **Kernel.** Source of fission-product palladium. Palladium-uranium clusters are common in the IPyC and SiC of AGR-1 particles [ref:demkowicz2016].
- **Buffer and IPyC.** Only Pd-U precipitates, without silver, were found in the IPyC layer of an AGR-1 particle [ref:vanrooyen2014a]. Cracks in the IPyC expose the SiC and let palladium concentrate there [ref:hunn2016].
- **SiC.** Palladium is the most common element in nano-precipitates on SiC grain boundaries, triple junctions and inside grains [ref:lillo2015]; micron-sized precipitates at grain boundaries are mainly palladium, often with uranium and silicon in a structure similar to UPd₂Si₂ but not of that stoichiometry [ref:vanrooyen2014a] [ref:demkowicz2016].
- **OPyC and matrix.** In six AGR-1 compacts, the Pd-105 fraction found outside the SiC layer but inside the compact was about 6 × 10⁻³ to 3 × 10⁻², showing that palladium passes intact SiC [ref:demkowicz2016].

## Candidate release mechanisms

| Mechanism | Regime | Entry |
|---|---|---|
| Pd-SiC chemical interaction (corrosion, penetration of the layer) | B, leading to C when the layer is penetrated | [pd_assisted_degradation](pd_assisted_degradation.md) |
| Palladium transport along SiC grain boundaries, as atoms or as nodules | B | same entry |
| Ag-Pd interaction (palladium-assisted silver transport) | B | same entry |
| Release through a SiC layer failed by corrosion next to a cracked IPyC | C | same entry and [Regime C](regime-c.md) |

## Current evidence

- **In-reactor penetration.** PARFUME models chemical attack of SiC by palladium as a penetration rate fitted to all available in-reactor data, a function of temperature only [ref:miller2018]; the report adds that post-irradiation examination might indicate other factors [ref:miller2018].
- **Temperature dominance.** Across irradiated particles with kernels from UO2 to UC2, including PuO2-x and mixed (Th,Pu) oxides, and enrichments from 0.7 % to 93 % U-235, temperature is the major factor in the penetration rate of SiC by palladium; kernel composition, palladium concentration, other fission products and SiC properties are secondary [ref:tiegs1980] [ref:tiegs1982].
- **AGR-1.** Widespread degradation of the SiC by palladium was not observed, although burnups reached almost 20 % FIMA [ref:demkowicz2016]. In particles with high silver release, palladium-uranium clusters were found through the whole SiC thickness; in particles with low silver release, only in about the inner half [ref:demkowicz2016]. Uranium-bearing precipitates migrated about 23 µm into the 35 µm layer [ref:lillo2015].
- **Atomistic.** Density-functional metadynamics at 1500 K gives palladium atomic diffusion coefficients of 7.96 × 10⁻¹¹ to 7.26 × 10⁻⁹ m² s⁻¹ along a Σ5 boundary and 2.38 × 10⁻¹¹ to 8.24 × 10⁻¹⁰ m² s⁻¹ along a carbon-rich Σ3 boundary, much faster than the observed penetration of palladium nodules; the authors suggest slow dissolution and rapid regrowth of nodules as a possible ingress mechanism [ref:rabone2015]. Ab initio molecular dynamics at 2000 K shows palladium attacking a SiC surface and removing carbon into chain-like structures [ref:aagesen2022]. These are atomistic results, not effective diffusivities.
- **Disagreement.** Out-of-pile couples with Pd and Ag led Neethling, Olivier and O'Connell to propose that silver needs palladium to move along SiC grain boundaries, while López-Honorato et al. observed silver transport without palladium; the first studies used palladium and silver concentrations much higher than in TRISO particles [ref:abdulhameed2026]. The original papers are listed as not verified ([ref:neethling2012] [ref:olivier2013] [ref:lopezhonorato2010]).

## Proposed governing-equation extensions

A palladium extension is a coupled system, not a single extra term: a penetration front in the SiC whose depth follows a temperature-dependent rate, and a layer-state rule that changes the transport of other species once the front crosses the layer. Candidate equations, their sources and the missing pieces are in the [palladium-assisted degradation entry](pd_assisted_degradation.md). No source read gives a quantitative law for how palladium enhances silver transport.

## Potential Materials Studio calculations

Planned and not performed (see the [gallery](materials_studio_gallery.md)): a palladium atom and small palladium clusters at a SiC surface and at Σ3 and Σ5 grain boundaries; palladium silicide formation at a SiC grain boundary; palladium and silver at the same grain-boundary site. Any such calculation is atomistic and would not by itself give a penetration rate.

## Experimental comparison opportunities

- **AGR-1.** Pd-105 inventories outside the SiC from deconsolidation-leach-burn-leach analysis, distribution of Pd-U clusters across the SiC thickness versus silver release, and microanalysis of particles with failed SiC [ref:demkowicz2016] [ref:hunn2016]. Quantification of palladium on capsule components was largely unsuccessful, so release from the compacts is not known [ref:demkowicz2016].
- **AGR-2.** Particles with failed SiC and their causes are enumerated in the AGR-2 post-irradiation examination report [ref:stempien2021].
- **AGR-3/4.** Designed-to-fail particles provide a source for failed-particle studies [ref:grover2013].

These experiments were irradiated under gas-cooled conditions (inert helium-neon sweep gas in the Advanced Test Reactor) [ref:davenport2016].

## Research limitations

- The penetration-rate correlation lumps different fuels and reactors and is a function of temperature only [ref:miller2018] [ref:tiegs1982].
- No source read gives a calibrated relation between palladium attack and silver release.
- Laboratory couples used far higher palladium and silver concentrations than exist in TRISO particles [ref:abdulhameed2026].
- No palladium isotope is in the repository's nuclear-data register, and no palladium fission yield is registered.

## Implementation status

The research status is held only in `data/research/mechanisms.csv` (row `pd_assisted_degradation`, species `Pd`, domain `SiC`). STRATO has no derivation, register row, code or test for palladium transport or Pd-SiC interaction.
