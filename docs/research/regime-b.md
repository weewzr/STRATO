# Regime B: degradation-assisted transport

Regime B covers transport through layers that are still continuous but whose barrier properties have been changed during service, by irradiation or by chemical reaction with fission products. The regimes are transport regimes, not mutually exclusive mechanisms; Regime B processes act on the Regime A paths and can end in Regime C when the layer is breached.

## Mechanisms to investigate

| Mechanism | What it means | Catalogue entry |
|---|---|---|
| Palladium-assisted migration | silver (and other species) moving through SiC together with palladium | [Palladium-assisted degradation](pd_assisted_degradation.md) (brick 6) |
| Pd-SiC chemical interactions | reaction of palladium with SiC, forming silicides and carbon, thinning the layer | same entry |
| Irradiation defects | vacancies, interstitials and their clusters created by neutron damage | [Irradiation and coating degradation](coating_degradation_failure.md) (brick 8); effects on silver noted below |
| Microstructural degradation | changes of grain structure and boundaries in service | [brick 8](coating_degradation_failure.md) |
| Defect-assisted diffusion | faster diffusion through irradiation-created defects | [brick 8](coating_degradation_failure.md); silver evidence below |
| Relevant chemical interactions | other fission product-SiC reactions | [brick 8](coating_degradation_failure.md) |

## Evidence by mechanism (silver and palladium)

- **Pd-SiC interaction.** Palladium corrodes SiC in irradiated particles, and temperature is the main factor in the rate [ref:tiegs1980] [ref:tiegs1982]. PARFUME models it as a temperature-only penetration rate [ref:miller2018]. In AGR-1, widespread palladium degradation was not observed, but palladium corrosion was a key contributor to the SiC failures of caesium-releasing particles [ref:demkowicz2016] [ref:hunn2016].
- **Palladium-assisted silver migration.** Proposed from out-of-pile couples in which Pd-Ag-Si nodules moved along SiC boundaries; contested because silver transport was also seen without palladium and because the couples used much higher concentrations than TRISO particles contain [ref:abdulhameed2026]. In AGR-1 SiC, silver was found alone or with palladium at boundaries and triple junctions [ref:lillo2015].
- **Irradiation defects and defect-assisted diffusion.** Silver diffusion in diffusion couples was greatly enhanced by carbon-ion irradiation, attributed to radiation damage combined with grain boundaries [ref:leng2016]; silver diffusion in single-crystal 4H-SiC was enhanced by implantation-induced defects [ref:gerczak2015]. An INL mesoscale model makes the bulk silver diffusivity depend on neutron flux (not fluence) through the irradiation-enhanced vacancy concentration, most important below about 1500 K [ref:aagesen2022] [ref:simon2025].

## Interactions with other regimes

- Regime B changes Regime A parameters (boundary and bulk diffusivities, trap densities) rather than replacing them.
- Palladium penetration that crosses the SiC turns the particle into a Regime C case: complete penetration is a functional failure [ref:miller2018]. Cracks in the IPyC expose SiC to concentrated palladium attack [ref:hunn2016], a Regime B process triggered by a Regime C event in a neighbouring layer.

## What STRATO would need

Regime B extensions add state variables that evolve during irradiation (a palladium penetration depth, a defect concentration or a flux-dependent diffusivity) and couple them to the transport equation. They need fluence or flux histories and temperature histories, registered parameters, and their own verification. Their research statuses are held only in `data/research/mechanisms.csv`.
