# Silver (Ag)

Species entry of the STRATO research catalogue. Transport regimes: A, B and C. Reference fuel: AGR-1 UCO TRISO. STRATO does not simulate silver today; nothing on this page is a STRATO result.

## Why it matters

Silver is one of the metallic fission and activation products that the SiC layer does not fully retain: several fission products, silver and caesium among them, have been observed to escape from intact TRISO particles [ref:simon2022]. The release of silver is a concern because of the isotope Ag-110m [ref:simon2022], a strong gamma-ray emitter produced from the fission product Ag-109 by neutron capture, Ag-109(n,γ)Ag-110m [ref:abdulhameed2026]. Its relatively short half-life raises its radiological impact during planned reactor maintenance [ref:parsons2024].

In the AGR-1 irradiation, Ag-110m release fractions ranged from 1.2 % to 38 % at capsule level and from about 0 % to more than 90 % for individual compacts; within one compact, individual particles ranged from full retention to near-complete release [ref:demkowicz2016]. The relatively high fractional release of silver is described as a well-known characteristic of TRISO fuel [ref:demkowicz2016].

Isotopes in the repository's nuclear-data register (`data/nuclear/decay.csv`, ENSDF adopted values):

| Nuclide | Stable or radioactive | Half-life in the register | Origin noted in the register |
|---|---|---|---|
| Ag-109 | stable (isotopic abundance 48.161 %) | none | cumulative fission product; neutron-capture target for Ag-110m |
| Ag-110m | radioactive, β− 98.67 %, IT 1.33 % | 249.83 d | activation product Ag-109(n,γ), shielded from direct fission yield by stable Pd-110 |
| Ag-110 | radioactive (ground state, IT daughter of Ag-110m) | 24.56 s | IT daughter of Ag-110m |
| Ag-111 | radioactive, β− 100 % | 7.45 d | cumulative fission product (via Pd-111 and Pd-111m) |

The decay constant of Ag-110m in the register is 3.211198 × 10⁻⁸ s⁻¹. Its decay daughter is stable Cd-110 (register row). Because Ag-110m is an activation product, a STRATO source term for it would need a neutron-capture rate of Ag-109, and the register notes that this cross section is not verified (`data/nuclear/fission_yields.csv`); activation products are deferred by owner decision D3.

## Relevant TRISO layers

The AGR-1 particle has a UCO kernel of nominal diameter 350 µm, a porous carbon buffer (100 µm nominal), an inner pyrolytic carbon layer (IPyC, 40 µm), a SiC layer (35 µm) and an outer pyrolytic carbon layer (OPyC, 40 µm) [ref:demkowicz2016]. STRATO's registered AGR-1 geometry is the characterisation set in `data/geometry/geometry.csv` (rows G-01 to G-05).

- **Kernel.** Silver diffusivity rows exist in the register only for UO2 kernels (`data/diffusivity/arrhenius.csv#A1-04`, from [ref:iaea1997]); applying them to the AGR-1 UCO kernel is an assumption that must be labelled (rule E3).
- **Buffer and IPyC.** Silver passes the pyrocarbon layers; register rows `A2-03`, `A2-08` and `A2-13` give PyC diffusivities from [ref:iaea1997].
- **SiC.** The main barrier and the layer where all silver-specific mechanisms of this catalogue act. Silver has been found at SiC grain boundaries and triple points near the IPyC-SiC interface [ref:vanrooyen2014a] and deeper in the layer [ref:demkowicz2016] [ref:lillo2015].
- **OPyC and compact matrix.** Released silver that is retained in the graphite matrix is a source of error in compact-level release measurements [ref:parsons2024].

## Candidate release mechanisms

The mechanisms proposed in the literature for silver release from intact particles are: diffusion through the bulk and grain boundaries of polycrystalline SiC; transport assisted by palladium attack of the SiC; void-mediated transport; vapour diffusion through nanocracks; and irradiation-enhanced transport [ref:simon2022]. A critical review lists further factors thought to influence silver transport: grain-boundary stoichiometry, grain size and shape, free silicon, nano-cracks, thermal decomposition, palladium attack, transmutation products, layer thinning and particle shape [ref:vanrooyen2014b]. Despite about three decades of research the dominant mechanism is not settled [ref:simon2022] [ref:vanrooyen2014b].

Catalogue entries for silver:

| Mechanism entry | Regime | Brick |
|---|---|---|
| [Silver trapping and detrapping](ag_trapping_detrapping.md) | A (traps from irradiation damage link it to B) | 4 |
| [Silver grain-boundary transport](ag_grain_boundary_transport.md) | A | 5 |
| [Palladium-assisted degradation, Ag-Pd interaction](pd_assisted_degradation.md) | B, C | 6 |
| Microcracking, coating failure, failed-particle release | C | 8 (not written in this entry; see [Regime C](regime-c.md)) |

## Current evidence

- **Bulk SiC.** Density-functional calculations for ideal 3C-SiC give very high activation energies for every silver defect state; the lowest, for the silver interstitial, is about 7.9 eV, so bulk transport is very slow and another path must operate [ref:shrader2011]. These are atomistic results for a perfect crystal, not effective diffusivities of the coating.
- **Grain boundaries.** Silver defect formation energies are 2 to 4 eV lower in a Σ3 grain boundary than in the bulk, a strong tendency to segregate; at 1600 °C the predicted Σ3 and bulk diffusion coefficients are 3.7 × 10⁻¹⁸ and 3.9 × 10⁻²⁹ m² s⁻¹ [ref:khalil2011]. Kinetic Monte Carlo for high-energy grain boundaries gives an effective activation energy of 2.79 ± 0.18 eV, consistent with ion-implantation data but two to three orders of magnitude below integral-release values [ref:ko2016]. Both are atomistic or atomistic-based results for a specific boundary model.
- **Observations in irradiated AGR-1 SiC.** Nano-sized (about 2 to 5 nm) silver precipitates were found in SiC grain boundaries and triple points, none inside grains [ref:demkowicz2016]; silver was found alone or together with palladium, never together with uranium, at depths up to about 24 µm of the 35 µm layer in one particle [ref:lillo2015].
- **Effect of microstructure.** No clear correlation between SiC microstructure and Ag-110m retention was found in the AGR-1 particles examined, and the variation was attributed mainly to particle-to-particle temperature variation [ref:demkowicz2016]. A mechanistic mesoscale model nevertheless predicts a grain-size effect [ref:simon2022]. The two statements are not reconciled in the sources read.
- **Laboratory diffusion couples.** Ion-implantation diffusion couples at 1500 to 1625 °C suggest that grain-boundary diffusion in undamaged SiC does not describe the release observed from TRISO fuel [ref:gerczak2015]; carbon-ion irradiation greatly enhanced silver diffusion in diffusion couples at 1500 to 1569 °C [ref:leng2016].
- **Empirical diffusivities.** IAEA TECDOC-978 lists Ag-in-SiC Arrhenius fits that differ between countries: D₀ = 3.6 × 10⁻⁹ m² s⁻¹ and Q = 215 kJ mol⁻¹ (FRG and USA), 6.8 × 10⁻¹¹ m² s⁻¹ and 177 kJ mol⁻¹ (Japan, 1200 to 1400 °C), 3.5 × 10⁻¹⁰ m² s⁻¹ and 213 kJ mol⁻¹ (Russian Federation, 1200 to 2300 °C) [ref:iaea1997]. They are registered as rows `A3-05`, `A3-12`, `A3-16` and `A3-21`. These are effective (macroscopic) diffusivities that lump every transport process together.

## Proposed governing-equation extensions

STRATO solves layered spherical diffusion today. For silver the literature suggests three extensions, each a separate catalogue entry with its candidate equations:

1. A trapping-detrapping system that adds a trapped-silver concentration coupled to the mobile one, or the equivalent effective trap-limited diffusivity ([entry](ag_trapping_detrapping.md)).
2. A microstructure-dependent effective diffusivity of the SiC layer that replaces the empirical Arrhenius row by a function of temperature and grain size ([entry](ag_grain_boundary_transport.md)).
3. A palladium penetration front that degrades the SiC layer and, once it crosses the layer, turns it into a failed layer for silver ([entry](pd_assisted_degradation.md)).

Radioactive decay of Ag-110m (register half-life 249.83 d) enters through STRATO's decay term once Phase 6.1 delivers it; stable Ag-109 has no decay term.

## Potential Materials Studio calculations

Planned atomistic calculations relevant to silver are listed in the [Materials Studio gallery](materials_studio_gallery.md): silver substitutional, interstitial and silver-vacancy configurations in 3C-SiC; silver at Σ3, Σ5 and high-energy grain boundaries; migration pathways; trapping sites at pore surfaces and triple junctions; and silver next to palladium at a grain boundary. None has been performed. An ideal 3C-SiC supercell would not represent the polycrystalline, heavily faulted AGR-1 coating [ref:kirchhofer2013], and atomistic migration barriers are not effective diffusivities.

## Experimental comparison opportunities

- **AGR-1 (reference fuel).** Ag-110m inventories of compacts from precision gamma scanning divided by calculated activity [ref:parsons2024]; capsule- and compact-level release fractions and particle-level retention distributions [ref:demkowicz2016]; time-average, volume-average compact temperatures of 955 to 1136 °C over 620 effective full-power days [ref:demkowicz2016]. Computed agreement percentages are not given here; they are computed only after Phase 2.2 is implemented and verified.
- **AGR-2 (later independent comparison).** UCO fuel made in an engineering-scale coater and UO2 fuel [ref:stempien2021]; Ag release from AGR-1 and AGR-2 has been compared with BISON predictions in the literature [ref:simon2025].
- **AGR-3/4 (later failed-particle studies).** AGR-3/4 contained designed-to-fail particles that provide a fission-product source for measuring retention in compact matrix and graphite [ref:grover2013].

The AGR experiments were irradiated in the Advanced Test Reactor in capsules with an inert helium-neon sweep gas used to control temperature [ref:davenport2016]: they are gas-cooled conditions. Transport of silver downstream in FLiBe or a cover gas does not exist in STRATO today and is planned for a future phase; it is not compared with AGR data.

## Research limitations

- The dominant mechanism of silver transport through intact SiC is not established [ref:simon2022] [ref:vanrooyen2014b].
- Atomistic diffusion coefficients for specific boundaries span many orders of magnitude and refer to idealised structures [ref:khalil2011] [ref:ko2016] [ref:rabone2015].
- Fuel temperatures in AGR-1 and AGR-2 were not measured directly but calculated, with a calculated uncertainty of roughly ±50 K (AGR-1) and ±60 K (AGR-2) in peak fuel temperature, and this uncertainty strongly affects predicted silver release [ref:parsons2024].
- Kernel diffusivities in the register are for UO2, not UCO.
- The Ag-109 capture cross section needed for an Ag-110m source is not verified in the register.

## Implementation status

The research status of every silver mechanism is held only in `data/research/mechanisms.csv` (rows with species `Ag`: `ag_trapping_detrapping`, `ag_grain_boundary_transport`, `pd_assisted_degradation`). STRATO has no derivation, register row, code or test for any of them, and its live simulation does not include silver-specific physics.
