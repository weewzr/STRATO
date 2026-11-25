# Ruthenium (Ru)

Species entry of the STRATO research catalogue. Transport regime: A (the evidence read concerns intact coatings). Reference fuel: AGR-1 UCO TRISO. STRATO does not simulate ruthenium today; nothing on this page is a STRATO result.

## Why it matters

Ruthenium belongs to the noble-metal group of fission products [ref:compere1975]; its isotope Ru-106 is measured in individual irradiated TRISO particles [ref:demkowicz2016]. The evidence read for this entry suggests that intact TRISO coatings retain it well, which makes it a reference species rather than a release concern: in AGR-1, the three particles with failed SiC that had released 38 % to 65 % of their Cs-134 still held Ru-106 (with Zr-95, Sb-125, Ce-144 and Eu-154) within the normal distribution of particles with intact SiC [ref:demkowicz2016].

Isotopes (none of them is in the repository's decay register; values from the evaluated data of [ref:iaea_livechart_fp]):

| Nuclide | Stable or radioactive | Half-life |
|---|---|---|
| Ru-101, Ru-102, Ru-104 | stable (isotopic abundances 17.06 %, 31.55 %, 18.62 %) | none |
| Ru-103 | radioactive, β− 100 % | 39.247 d (uncertainty 0.013 d) |
| Ru-106 | radioactive, β− 100 % | 371.8 d (uncertainty 1.8 d) |

No ruthenium fission yield and no ruthenium decay row are registered.

## Relevant TRISO layers

- **Kernel and coatings.** No ruthenium diffusivity is registered for any layer.
- **SiC.** In SiC from TRISO particles irradiated in three reactors, the ruthenium content (with caesium, cerium, europium and antimony) was found proportional to the number of neutrons that had passed through the SiC, with no explicit dependence on time, burnup, temperature or enrichment [ref:iaea1997].
- **Comparison with ZrC.** In heating tests that compared ZrC-TRISO with SiC-TRISO particles, the ZrC layer retained caesium better but was a less effective barrier to Ru-106 than the SiC layer [ref:iaea1997].

## Candidate release mechanisms

1. **Diffusion through intact coatings** (Regime A); evidence suggests strong retention by SiC [ref:iaea1997] [ref:demkowicz2016].
2. **Uptake into SiC at irradiation-damage sites** (Regime B): the qualitative model in the source conjectures that fast-neutron damage in the SiC creates sites at which fission products bind, or paths by which they enter the coating [ref:iaea1997].

Species-specific mechanisms are grouped in [Additional species and downstream transport](additional_species_downstream_transport.md) (brick 9).

## Current evidence

- AGR-1: Ru-106 retained in particles with failed SiC within the normal range [ref:demkowicz2016].
- Older irradiations: ruthenium in SiC proportional to neutron exposure [ref:iaea1997].
- ZrC versus SiC: ruthenium diffusion in ZrC was reported somewhat larger than in SiC, and an estimated Ru-106 diffusion coefficient in ZrC of 3 × 10⁻¹⁶ m² s⁻¹ at 1600 °C is quoted [ref:iaea1997]; this is a value for ZrC, not for any layer of the AGR-1 particle.

No ruthenium diffusivity for the AGR-1 layers was found in the sources read for this entry.

## Proposed governing-equation extensions

Ruthenium would use the Phase 2 layered equation ((P21.1) in `docs/mathematics/STRATO_Phase2_Derivations.md`) with a decay term for Ru-103 or Ru-106 and none for the stable isotopes. Missing before it can be written for STRATO: a fission yield, decay rows in the register, and diffusivities for every layer. If the binding at irradiation-damage sites [ref:iaea1997] matters, a trapping term coupled to the damage state would be needed (see [Irradiation and coating degradation](coating_degradation_failure.md)).

## Potential Materials Studio calculations

None planned in the gallery yet. A possible later item is ruthenium at SiC vacancy and grain-boundary sites; it would be added as a Pending placeholder only with a cited motivation.

## Experimental comparison opportunities

- AGR-1 particle inventories of Ru-106 measured with the IMGA [ref:demkowicz2016]; not in the repository's measurement register.
- No ruthenium release rows exist in `data/experimental/release_measurements.csv`.

The AGR experiments were gas-cooled [ref:davenport2016]. In the Molten Salt Reactor Experiment (fuel dissolved in a fluoride salt, not TRISO particles in a coolant), ruthenium belonged to the noble-metal group that deposited on metal, graphite and the salt-gas interface, with some holdup before final deposition [ref:compere1975]. Ruthenium in FLiBe or a cover gas does not exist in STRATO today and is planned for a future phase.

## Research limitations

- Evidence is sparse and mostly qualitative; no layer diffusivity, yield or decay row is registered.
- The ZrC numbers are for a different coating material.

## Implementation status

Not part of the Phase 2 physics. Its catalogue row is in `data/research/mechanisms.csv` (`additional_species_downstream_transport`, species `Ru`). STRATO's live simulation does not include ruthenium.
