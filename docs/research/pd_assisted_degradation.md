# Palladium-assisted degradation of SiC (Pd-SiC and Ag-Pd interactions)

Brick 6. Species: Pd (the attacking species) and Ag (the species whose transport palladium may assist). Material domain: the SiC layer. Regime B (palladium-assisted migration, Pd-SiC chemical interaction, microstructural degradation); a layer penetrated by palladium is a failed barrier, which is Regime C. Every equation below is a research concept, not solved by STRATO.

## Proposed physics

Fission-product palladium reaches the IPyC-SiC interface and reacts with silicon carbide. Chemical attack by palladium is described as transforming SiC into a mixture of palladium silicide and graphite [ref:simon2022]. Ab initio molecular dynamics at 2000 K shows palladium atoms disintegrating a SiC surface and removing carbon into chain-like structures, while silver atoms are repelled from the surface; the authors suggest that palladium attack is an important first step in silver diffusion through SiC [ref:aagesen2022].

Two consequences matter for transport:

1. **Pd-SiC interaction (layer degradation).** Penetration of the SiC by palladium thins the barrier. Complete penetration provides an open path for fission products and is a functional failure of the particle; local thinning contributes little to stress-driven failure, while thinning over large areas can cause failure through stress concentrations and IPyC failure [ref:miller2018].
2. **Ag-Pd interaction (palladium-assisted silver transport).** Neethling, Olivier and O'Connell proposed that palladium penetrates SiC grain boundaries, forms palladium silicides, and that silver joins them in Ag-rich Pd-Ag-Si nodules that migrate along boundaries by dissolving SiC at the leading edge and reprecipitating it at the trailing edge; only small palladium concentrations would be needed, and high-temperature neutron irradiation would accelerate the process [ref:abdulhameed2026]. As reported by a DOE-funded study, Ag-Pd transport was significantly enhanced in high-temperature, high-dose neutron-irradiated 3C-SiC, but it is unclear whether this holds for silver without palladium [ref:szlufarska2016]. The original papers ([ref:neethling2012] [ref:olivier2013] [ref:oconnell2014]) were not opened for this entry.

Observations in irradiated AGR-1 fuel:

- Palladium crossed intact SiC [ref:demkowicz2016]; palladium is the most common element in nano-precipitates on grain boundaries, triple junctions and inside grains, sometimes together with silver or uranium; silver occurred alone or with palladium, uranium always with palladium [ref:lillo2015].
- Widespread palladium degradation of the SiC was not observed, even at burnups near 20 % FIMA [ref:demkowicz2016].
- In particles that released caesium during irradiation, the IPyC had cracked and the exposed SiC had suffered concentrated palladium attack; SiC failures after safety testing at 1600 to 1800 °C were related to fabrication defects or to extensive palladium corrosion where the SiC was exposed by similar IPyC cracks [ref:hunn2016].
- Particles with high silver release had palladium-uranium clusters through the whole SiC thickness; particles with low silver release only in about the inner half [ref:demkowicz2016]. This is a correlation reported by the source, not a demonstrated cause.

## Candidate governing equations (research concepts)

This mechanism needs additional coupled equations; it is written as a system and must not be inserted into STRATO's diffusion equation as one extra term.

**(a) Penetration rate of SiC by palladium (PARFUME).** An Arrhenius function fitted to the available Pd penetration data [ref:miller2018]:

$$
P = 38.232\, \exp\!\left(\frac{-14342.3}{T}\right)
$$

with $P$ the rate of penetration of the SiC in µm h⁻¹ and $T$ the absolute centreline temperature in K [ref:miller2018]. The report describes the fit as covering "all available in-reactor data" and cites four sources for it ([ref:minato1990] and [ref:tiegs1982] among them); one of the four cited titles describes out-of-reactor studies. PARFUME applies it as a global, temperature-only rate and notes that examination might show other influencing factors [ref:miller2018].

**(b) Penetration depth.** Integrating (a) over the temperature history gives a penetration depth; this integral is the plain definition of a depth from a rate and is a STRATO research proposal, not an equation printed in the source:

$$
x_{\mathrm{Pd}}(t) = \int_0^{t} P\big(T(t')\big)\, dt'
$$

**(c) Layer state.** Complete penetration is a functional failure of the particle [ref:miller2018]:

$$
\text{SiC layer failed when } x_{\mathrm{Pd}}(t) \ge t_{\mathrm{SiC}}
$$

with $t_{\mathrm{SiC}}$ the SiC thickness (AGR-1 Baseline 35.3 µm, `data/geometry/geometry.csv#G-04`).

**(d) Transport through a failed layer.** In PARFUME, the diffusivity of any layer predicted to fail is set to an arbitrarily large value, printed as "currently 1×10⁻⁶" [ref:miller2018]. For STRATO the switch would act on the diffusivity of the SiC layer for every species, silver included:

$$
D_{\mathrm{SiC}}(t) =
\begin{cases}
D_{\mathrm{SiC,intact}}(T) & x_{\mathrm{Pd}}(t) < t_{\mathrm{SiC}}\\
D_{\mathrm{failed}} & x_{\mathrm{Pd}}(t) \ge t_{\mathrm{SiC}}
\end{cases}
$$

**(e) Ag-Pd coupling.** No source read gives a quantitative law (a rate, an enhancement factor or a coupled concentration equation) for palladium-assisted silver transport. The atomistic palladium diffusivities along Σ5 and Σ3 boundaries at 1500 K (7.96 × 10⁻¹¹ to 7.26 × 10⁻⁹ and 2.38 × 10⁻¹¹ to 8.24 × 10⁻¹⁰ m² s⁻¹) are far faster than the observed penetration of palladium nodules, which points to nodule dissolution and regrowth as a possible ingress mechanism [ref:rabone2015]; these are atomistic values for single boundaries and are not a penetration rate. The INL work states that its Pd penetration model is to be studied further [ref:aagesen2022]. Until a cited law exists, the coupling term stays empty.

## Required parameters

| Parameter | In a source? | In STRATO's register? |
|---|---|---|
| Penetration-rate coefficients (38.232 µm h⁻¹, 14342.3 K) | printed in [ref:miller2018] | no |
| SiC thickness | yes | yes (`data/geometry/geometry.csv#G-04`) |
| Temperature history of the particle (centreline temperature for (a)) | calculated histories exist for AGR-1 compacts, not measured [ref:parsons2024] | not for this use |
| Palladium inventory and its arrival at the IPyC-SiC interface | not as a law; Pd-105 inventories outside SiC measured in AGR-1 [ref:demkowicz2016] | no (no Pd nuclide or yield registered) |
| Failed-layer diffusivity | an arbitrary value in [ref:miller2018] | no |
| Local attack at IPyC cracks | qualitative [ref:hunn2016] | no |
| Ag-Pd interaction law | not found | no |

## Supporting literature

[ref:miller2018] (penetration correlation, layer failure, failed-layer diffusivity), [ref:tiegs1980] and [ref:tiegs1982] (temperature as the main factor), [ref:minato1990] (a data source of the correlation, not opened), [ref:demkowicz2016], [ref:hunn2016], [ref:lillo2015] and [ref:vanrooyen2014a] (AGR-1 observations), [ref:rabone2015] (atomistic Pd at boundaries), [ref:aagesen2022] (ab initio Pd attack), [ref:abdulhameed2026] and [ref:szlufarska2016] (review of the Ag-Pd experiments), [ref:simon2022] (Pd silicide and graphite products).

**Where the literature disagrees.** Out-of-pile couples were read as showing that silver needs palladium to move along SiC boundaries, whereas López-Honorato et al. observed silver transport without palladium; Olivier and Neethling suggested silver vapour transport in the latter experiments, run at 1573 K, above the melting point of silver; and the palladium-silver experiments used much higher concentrations than occur in TRISO particles [ref:abdulhameed2026]. In AGR-1, silver and palladium were found together at some grain-boundary sites and apart at others: no palladium was found in triple junctions where silver was identified in one examination, while another high-resolution examination found cadmium, silver and palladium at the same location [ref:demkowicz2016].

## Missing calculations or experiments

- A quantitative Ag-Pd interaction law for TRISO-relevant concentrations.
- A local (not global) palladium attack model tied to IPyC cracking, since AGR-1 failures were local [ref:hunn2016].
- A palladium source term: no palladium nuclide or fission yield is registered in STRATO.
- Atomistic calculations of palladium at SiC surfaces and boundaries and of Pd-Ag co-location (gallery items `ms-pd-sic-surface`, `ms-pd-gb`, `ms-pd-ag-gb`); none performed.
- STRATO derivation of the switch (d) with conservation across the change of layer state, and verification.

## Interactions with other regimes

- **Regime A.** Palladium at grain boundaries and triple junctions may change silver boundary diffusion and trapping; see [Silver grain-boundary transport](ag_grain_boundary_transport.md) and [Silver trapping and detrapping](ag_trapping_detrapping.md).
- **Regime B, irradiation.** High-temperature neutron irradiation is reported to accelerate Ag-Pd transport [ref:szlufarska2016] [ref:abdulhameed2026].
- **Regime C.** IPyC cracking exposes the SiC to concentrated palladium attack, and complete penetration is a coating failure [ref:hunn2016] [ref:miller2018]; see [Regime C](regime-c.md).
