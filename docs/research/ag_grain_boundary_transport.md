# Silver grain-boundary transport in polycrystalline SiC

Brick 5. Species: Ag. Material domain: the SiC layer. Regime A (grain-boundary diffusion and segregation in an intact barrier); irradiation-enhanced grain-boundary diffusion connects it to Regime B. Every equation below is a research concept, not solved by STRATO.

## Proposed physics

Silver moves much faster along SiC grain boundaries than through the grain interiors, so the grain-boundary network, not the crystal lattice, would control silver transport through an intact coating. Density-functional work on ideal 3C-SiC finds bulk silver transport very slow and points to an alternative path such as grain boundaries [ref:shrader2011]; silver defects form 2 to 4 eV more easily in a Σ3 boundary than in the bulk, a strong tendency to segregate there [ref:khalil2011]. The effective diffusivity of the layer then depends on the boundary types present and on the grain size and shape, because these decide how many connected boundary paths cross the layer [ref:simon2022].

The AGR-1 SiC is cubic, has a high fraction of Σ3 coincidence-site boundaries and a heavily faulted microstructure; its average grain diameter grows from the IPyC-SiC interface outward except in Variant 3, which has a constant grain size [ref:kirchhofer2013]. Microstructure differs between AGR-1 variants and between coater batches of one variant [ref:gerczak2016]. Σ3 boundaries cannot provide a connected path through the layer and give a lower bound on grain-boundary diffusion; high-energy boundaries are expected to be faster and to percolate [ref:khalil2011].

An ideal 3C-SiC crystal or a single bicrystal does not represent this coating; the boundary-level results below are inputs to a microstructure model, not properties of the layer.

## Candidate governing equations (research concepts)

### Concept 1: microstructure-dependent effective diffusivity (Simon et al.)

A mesoscale effective silver diffusivity, built from atomistic bulk and grain-boundary diffusivities and polycrystals matching the AGR-1 grain dimensions, fitted as a function of temperature $T$ and grain minor-axis length $m_i$ (grain size perpendicular to the diffusion direction) [ref:simon2022]:

$$
D_{\mathrm{eff}}(T, m_i) = \left(D_{m,0} + m_i D_{m,1}\right)\exp\!\left(-\frac{Q_{m,0} + m_i Q_{m,1}}{RT}\right)
$$

$$
D_{\mathrm{eff,corr}}(T, m_i) = \delta_{\mathrm{corr}}\left(D_{m,0} + m_i D_{m,1}\right)\exp\!\left(-\frac{Q_{m,0} + m_i Q_{m,1}}{RT}\right)
$$

Printed values: $D_{m,0} = 1.10\times10^{3}\ \mu\mathrm{m^2\,s^{-1}}$, $D_{m,1} = -2.45\times10^{2}\ \mu\mathrm{m\,s^{-1}}$, $Q_{m,0} = 2.15\times10^{5}$ J mol⁻¹, $Q_{m,1} = 1.74\times10^{4}$ J mol⁻¹ µm⁻¹, $m_i$ in µm, $R = 8.315$ J K⁻¹ mol⁻¹ [ref:simon2022]. Facts about these numbers, from the source:

- They are fitted to mesoscale simulations at 1450, 1505, 1800 and 2073 K for minor axes of 0.3 to 0.82 µm; lower temperatures were numerically difficult and the fit is extrapolated to lower temperatures [ref:simon2022].
- The grain-boundary diffusivity used is that of high-energy random boundaries, one boundary type only, and irradiation effects are not included in the bulk diffusivity [ref:simon2022].
- The activation energy ranges from 2.20 × 10⁵ to 2.30 × 10⁵ J mol⁻¹ over the grain sizes of interest, close to the 2.15 × 10⁵ J mol⁻¹ of the empirical diffusivity used in BISON [ref:simon2022], the same value as register row `data/diffusivity/arrhenius.csv#A3-05` [ref:iaea1997].
- $\delta_{\mathrm{corr}}$ is a correction that brings the mesoscale value to the level of the empirical diffusivity and is attributed mainly to missing mechanisms; the source uses $\delta_{\mathrm{corr}} = 8.5$ [ref:simon2022]. A later report states that the factor "was set to 10" and gives a validity range of 1000 to 2200 K and 0.3 to 0.82 µm [ref:parsons2024]. **The two sources disagree on the value.**
- In the AGR-1 comparison a grain size was selected for each compact within its measured range to come closest to the post-irradiation data, which improved 14 of 17 compacts relative to the empirical diffusivity [ref:simon2022]. That selection is a calibration against the same data, not an independent test.

Later work adds irradiation enhancement of the bulk diffusivity, a function of flux rather than fluence and most important below about 1500 K [ref:aagesen2022], and reports that accounting for irradiation improves agreement where release was underestimated but matters less than microstructure [ref:simon2025]. Silver diffuses faster in Σ5 (210)/[001] boundaries than in random high-angle boundaries [ref:aagesen2022].

For STRATO this concept does not add an equation: it replaces the constant or Arrhenius diffusivity of the SiC layer by $D_{\mathrm{eff}}(T, m_i)$, so the existing layered diffusion equation keeps its form.

### Concept 2: grain-boundary and lattice kinetics regimes (Harrison classification)

In a polycrystal, lattice diffusion from the surface, diffusion along boundaries and leakage from boundaries into the lattice combine; Harrison's classification separates regimes A, B and C [ref:szlufarska2016]. (Harrison's types A, B and C are not STRATO's transport regimes A, B and C.) With lattice diffusivity $D_l$, grain-boundary diffusivity $D_{gb}$, segregation factor $s$, effective boundary width $\delta$, grain diameter $d$ and time $t$, the source states [ref:szlufarska2016]:

- Type A when $(D_l t)^{1/2} > d$;
- Type C, boundary diffusion without substantial leakage, when $(D_l t)^{1/2} < 0.2\, s\delta$;
- Type B when

$$
\beta = \frac{s\,\delta\, D_{gb}}{2 D_l \left(D_l t\right)^{1/2}} > 10,
\qquad
\alpha = \frac{s\,\delta}{2\left(D_l t\right)^{1/2}} < 0.1
$$

and then the boundary diffusivity follows from the slope of the depth profile:

$$
s\,\delta\, D_{gb} = 1.322\left(\frac{D_l}{t}\right)^{1/2}\left[-\frac{\partial \ln C}{\partial\left(d - d_o\right)^{6/5}}\right]^{-5/3}
$$

where $C$ is concentration and $d - d_o$ depth [ref:szlufarska2016]. This is an analysis tool for depth profiles, not a transport model for a particle; it tells which description (effective medium, boundary-only, or coupled boundary-lattice) applies. For carbon-irradiated SiC at 1569 °C the source finds Type B possible for $1 < s < 140$, with $1.74\times10^{-22} < D_l < 3.41\times10^{-18}$ m² s⁻¹ and $3.41\times10^{-16} < D_{gb} < 4.78\times10^{-14}$ m² s⁻¹, against unirradiated estimates of about 10⁻²⁹ and 10⁻¹⁸ m² s⁻¹; an atomistic segregation factor of 3.1 × 10⁸ for Σ3 at 1200 °C is quoted [ref:szlufarska2016].

### Concept 3: solubility-limited steady flux along radial boundaries (Khalil et al.)

To relate boundary diffusivities to integral release, Khalil et al. assume columnar boundaries crossing the SiC radially, a silver concentration at the solubility limit $c_{\mathrm{Ag}}$ at the inner surface and zero at the outer surface, a flat layer of thickness $d_{th}$, and no time spent in other layers [ref:khalil2011]:

$$
\Phi = D^{\mathrm{Ag}}_{\mathrm{eff}}\,\frac{c_{\mathrm{Ag}}}{d_{th}},
\qquad
\mathrm{IR} = \Phi\, t\, \left(f_{gb}\, 4\pi r_i^{2}\right)
$$

with $f_{gb}$ the fraction of the inner SiC surface area made of boundaries and $r_i$ the inner SiC radius [ref:khalil2011]. Unlike the Booth model, it contains a solubility term and assumes a maximal flux at all times [ref:khalil2011]. It is a bounding estimate, not a time-dependent model.

## Required parameters

| Parameter | In a source? | In STRATO's register? |
|---|---|---|
| Grain minor-axis length per AGR-1 variant | yes: as-fabricated EBSD [ref:gerczak2016]; ranges per compact as compiled in [ref:simon2022] | no |
| $D_{m,0}$, $D_{m,1}$, $Q_{m,0}$, $Q_{m,1}$ | printed in [ref:simon2022] | no |
| $\delta_{\mathrm{corr}}$ | 8.5 [ref:simon2022] or 10 [ref:parsons2024] | no |
| Lattice and boundary diffusivities of Ag for each boundary type | atomistic values for specific boundaries [ref:khalil2011] [ref:ko2016] [ref:rabone2015] | no |
| Segregation factor, boundary width | atomistic or assumed values in [ref:szlufarska2016] [ref:khalil2011] | no |
| Empirical effective Ag diffusivity in SiC | yes [ref:iaea1997] | yes, rows `A3-05`, `A3-12`, `A3-16`, `A3-21` |
| Temperature history of each AGR-1 compact | calculated, not measured [ref:parsons2024] | not for this use |

## Supporting literature

Atomistic: [ref:shrader2011] (bulk, ideal 3C-SiC), [ref:khalil2011] (Σ3 boundary: at 1600 °C predicted 3.7 × 10⁻¹⁸ m² s⁻¹ in the boundary and 3.9 × 10⁻²⁹ m² s⁻¹ in the bulk), [ref:ko2016] (high-energy boundaries modelled as amorphous SiC: effective activation energy 2.79 ± 0.18 eV and prefactor (2.73 ± 1.09) × 10⁻¹⁰ m² s⁻¹ between 1200 and 1600 °C), [ref:rabone2015] (Σ5 boundary at 1500 K: 7.04 × 10⁻¹⁹ to 3.69 × 10⁻¹⁷ m² s⁻¹). These are diffusion coefficients for one boundary model each, derived from atomistic barriers; they are not effective diffusivities of the coating.

Mesoscale: [ref:simon2022], [ref:aagesen2022], [ref:simon2025], [ref:abdulhameed2026]. Microstructure: [ref:kirchhofer2013], [ref:gerczak2016]. Experiments: [ref:gerczak2015], [ref:leng2016], [ref:szlufarska2016]. Empirical: [ref:iaea1997].

**Where the literature disagrees.**

- Integral-release measurements give activation energies of about 1.83 to 2.26 eV and ion implantation about 2.5 eV, while the Σ3 calculation is higher by more than about 1.69 eV [ref:khalil2011]. The high-energy-boundary model agrees with ion-implantation data but is two to three orders of magnitude below integral-release values, which its authors suggest may reflect radiation-enhanced diffusion [ref:ko2016].
- Ion-implantation diffusion couples suggest that grain-boundary diffusion in undamaged SiC does not describe the release observed from TRISO fuel [ref:gerczak2015], whereas the mesoscale model with a correction factor reproduces AGR-1 release for most compacts [ref:simon2022].
- AGR-1 post-irradiation examination found no clear correlation between SiC microstructure and silver retention [ref:demkowicz2016], while the mesoscale model predicts that thinner grains raise release [ref:simon2022].

## Missing calculations or experiments

- Registered grain-size data for the AGR-1 SiC layer (per variant), with the source table.
- Boundary-type-resolved diffusivities that represent the AGR-1 boundary population, including Σ3 fractions [ref:kirchhofer2013]; gallery items `ms-sic-sigma3-gb`, `ms-sic-sigma5-gb`, `ms-sic-random-gb`.
- An independent determination of $\delta_{\mathrm{corr}}$, or its removal by adding the missing mechanisms [ref:simon2022].
- STRATO derivation and verification for a temperature- and microstructure-dependent SiC diffusivity (this rides on Phase 2.2, temperature-dependent diffusivity).
- Per-temperature comparison with AGR-1 data after Phase 2.2; no agreement figure is written before that.

## Interactions with other regimes

- **Regime A, trapping.** Pore and triple-junction traps lower the effective boundary diffusivity; see [Silver trapping and detrapping](ag_trapping_detrapping.md).
- **Regime B, irradiation.** Carbon-ion irradiation greatly enhanced silver diffusion in diffusion couples, through radiation damage combined with grain boundaries [ref:leng2016]; the INL mesoscale model accounts for the irradiation-enhanced vacancy concentration in the bulk, which it finds substantially influences bulk diffusivity at low temperatures [ref:aagesen2022].
- **Regime B, palladium.** Palladium at grain boundaries may change boundary transport of silver; see [Palladium-assisted degradation](pd_assisted_degradation.md).
- **Regime C.** Vapour transport through nanocracks and void-mediated transport are alternatives to boundary diffusion [ref:simon2022] [ref:xiao2012]; see [Regime C](regime-c.md).
