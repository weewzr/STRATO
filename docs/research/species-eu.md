# Europium (Eu)

Species entry of the STRATO research catalogue. Transport regimes: A and C. Reference fuel: AGR-1 UCO TRISO. STRATO does not simulate europium today; nothing on this page is a STRATO result.

## Why it matters

Europium is a lanthanide fission product that passes intact TRISO coatings in small amounts during irradiation. In AGR-1 the capsule-average fractional release of Eu-154 from the compacts was about 1 × 10⁻⁴ to 5 × 10⁻⁴, and up to about 10⁻² of the inventory was found outside the SiC in the compacts, roughly one order of magnitude more than strontium; like strontium, europium was retained to a significant degree in the matrix or OPyC [ref:demkowicz2016]. In the AGR-1 safety tests at 1600 °C, six of eight compacts released europium equivalent to 3 to 12 particles' inventory although most of them had no particle with failed SiC, so europium release was not strongly connected with SiC failure [ref:morris2016].

Isotopes (register `data/nuclear/decay.csv` unless marked otherwise):

| Nuclide | Stable or radioactive | Half-life | Origin |
|---|---|---|---|
| Eu-153 | stable (isotopic abundance 52.19 %); not in the decay register [ref:iaea_livechart_fp] | none | fission product (yield rows in `data/nuclear/fission_yields.csv`); neutron-capture target for Eu-154 (register) |
| Eu-154 | radioactive, β− 99.982 %, EC+β+ 0.018 % | 8.592 y | activation product Eu-153(n,γ), shielded from direct fission yield by stable Sm-154 (register) |
| Eu-155 | radioactive, β− 100 % | 4.753 y (uncertainty 0.014 y); not in the register [ref:iaea_livechart_fp] | monitored as a fission product in the AGR-1 safety tests [ref:morris2016] |

The register decay constant of Eu-154 is 2.556445 × 10⁻⁹ s⁻¹. Because Eu-154 is an activation product, a STRATO source term for it needs the Eu-153 capture rate, and activation products are deferred by owner decision D3. Eu-155 was measured in AGR-1 and gave results matching Eu-154 [ref:demkowicz2016]; it is not in the register and has no registered fission yield.

## Relevant TRISO layers

- **Kernel, buffer, IPyC, SiC, OPyC.** No europium diffusivity is registered for any layer (`data/diffusivity/arrhenius.csv` has no Eu row).
- **SiC.** Europium content in irradiated SiC was found proportional to the number of neutrons that had passed through the layer, together with caesium, cerium, ruthenium and antimony [ref:iaea1997].
- **OPyC and compact matrix.** Europium released through intact coatings was largely held there; after the safety tests it was distributed through the OPyC, matrix, graphite holder and furnace parts [ref:demkowicz2016] [ref:morris2016].

## Candidate release mechanisms

1. **Diffusion through intact coatings during irradiation** (Regime A), with retention in OPyC and matrix [ref:demkowicz2016].
2. **Slow release of the inventory deposited in the OPyC and matrix during heating**, the working hypothesis for the AGR-1 safety tests [ref:morris2016].
3. **Diffusion through SiC at high temperature.** Increasing release rates of the two Variant 3 compacts at 1800 °C could possibly be linked to diffusion through SiC; further study is required [ref:morris2016]. Morris et al. report that in the earlier HRB-15B test loose UCO particles heated at 1500 °C released 37 % of their europium, uniformly from all particles and consistent with classical diffusion, and 6 % after 10,000 h at 1200 °C (as reported by [ref:morris2016]; the original report was not opened).
4. **Fission-product-coating interaction.** In a 10,000 h anneal at 1500 °C of five types of irradiated SiC-TRISO particles, only the type with a ZrC layer on the UO2 kernel retained all of its silver and europium [ref:iaea1997].

Species-specific mechanisms are grouped in [Additional species and downstream transport](additional_species_downstream_transport.md) (brick 9).

## Current evidence

See above. The evidence is integral (compact or capsule release, compact inventories outside the SiC) and does not separate kernel, coating and matrix contributions [ref:morris2016]. No measured or recommended europium diffusivity in TRISO layers was found in the sources read for this entry, and none is registered.

## Proposed governing-equation extensions

Europium would use the same layered equation as the Phase 2 nuclides ((P21.1) in `docs/mathematics/STRATO_Phase2_Derivations.md`), with a decay term for the radioactive isotopes and none for stable Eu-153. Two terms are missing before it can be written for STRATO:

- a source: Eu-154 needs the activation source of the derivations' Section 3.3.3 (deferred, decision D3); Eu-155 needs a fission yield, which is not registered;
- diffusivities for every layer, none of which is registered.

A matrix region (or an outer boundary condition other than free release) would be needed to represent the retention that dominates the AGR-1 europium data; STRATO's domain ends at the OPyC.

## Potential Materials Studio calculations

Planned: europium (with strontium) in 3C-SiC and at a SiC grain boundary (gallery item `ms-sr-eu-sic`). Not performed; Pending placeholder only. Atomistic energies would not be effective diffusivities.

## Experimental comparison opportunities

- **AGR-1 safety tests.** Eu-154 release at 1600, 1700 and 1800 °C [ref:morris2016]; register rows with nuclide `Eu-154` in `data/experimental/release_measurements.csv` (source Morris et al. (2016)).
- **JAERI heating tests (UO2 TRISO).** Eu-154 rows of the same register (source IAEA (1997)); not the reference fuel.

No agreement figure is given before Phase 2.2 is implemented and verified, and a europium comparison additionally needs the missing source and diffusivities. The AGR experiments were gas-cooled [ref:davenport2016]; europium in FLiBe or a cover gas does not exist in STRATO today and is planned for a future phase.

## Research limitations

- No europium diffusivity and no Eu-155 fission yield are registered.
- Eu-154 is an activation product (decision D3 defers activation).
- Release data mix in-pile matrix inventory and heating-test release [ref:morris2016].

## Implementation status

Europium is not part of the Phase 2 physics. Its catalogue row is in `data/research/mechanisms.csv` (`additional_species_downstream_transport`, species `Eu`). STRATO's live simulation does not include europium.
