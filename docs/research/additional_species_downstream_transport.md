# Additional species and downstream transport

Brick 9. Species: Sr, Eu, I, Ru, Xe and Kr inside the particle, and all species for downstream transport. Material domains: the whole particle for the species part; a separate, planned downstream domain (FLiBe coolant and cover gas) for the transport part. Regimes: A and C inside the particle; the downstream row records the particle regimes that feed it (release from intact particles, A, and from failed particles, C). Every equation below is a research concept, not solved by STRATO.

This entry has two parts that must stay apart. Part 1 is built on experiments made under gas-cooled conditions. Part 2 describes planned work for a molten-salt environment; no STRATO model, data or result exists for it.

## Proposed physics

### Part 1: additional species in the TRISO particle

| Species | What the evidence says | Entry |
|---|---|---|
| Sr | passes intact coatings in small amounts during irradiation and is held mostly in OPyC and matrix; heating-test release dominated by that inventory [ref:demkowicz2016] [ref:morris2016] | [species-sr](species-sr.md) |
| Eu | as strontium, at about one order of magnitude higher levels; possible diffusion through fine-grained SiC at 1800 °C [ref:demkowicz2016] [ref:morris2016] | [species-eu](species-eu.md) |
| I | expected to behave like the fission gases; released mainly through failed coatings [ref:iaea1997] | [species-i](species-i.md) |
| Ru | retained even in particles with failed SiC; content in SiC proportional to neutron exposure [ref:demkowicz2016] [ref:iaea1997] | [species-ru](species-ru.md) |
| Xe, Kr | gas-tight behind intact SiC; released by failed particles and by heavy-metal (uranium) contamination, whose location the AGR-1 gas-release source does not settle (see [species-xe](species-xe.md#candidate-release-mechanisms)); an intact pyrocarbon layer retains most krypton even when the SiC has failed [ref:iaea1997] [ref:scates2010] [ref:morris2016] | [species-xe](species-xe.md), [species-kr](species-kr.md) |

Two physical features recur and are not in STRATO's model:

1. **Retention outside the SiC.** Strontium, europium and part of the caesium and silver released from the particles stay in the OPyC and compact matrix, and this inventory dominates the heating-test release of strontium and europium [ref:morris2016]. STRATO's domain ends at the OPyC outer surface, which it treats as absorbing (free release).
2. **Release that does not come from intact particles.** Fission-gas release in AGR-1 was attributed to heavy-metal (uranium) contamination [ref:scates2010], which the body of that paper places in the capsule graphite and compact matrix and its conclusion inside the TRISO coating layers (see the caveat in the [xenon entry](species-xe.md#candidate-release-mechanisms)); a later AGR-1 summary refers to uranium contamination in the compact matrix or OPyC [ref:demkowicz2016]. Caesium and krypton release from individual failed particles [ref:morris2016]. Both need a population description (see [brick 7](cs_retention_barrier_integrity.md) and [brick 8](coating_degradation_failure.md)).

### Part 2: downstream transport in FLiBe and a cover gas (planned work)

**This modelling does not exist in STRATO today and is planned for a future phase.** No FLiBe or cover-gas data, model or result is in the repository. The text below says only what a later treatment would need.

What is established in the sources read:

- In a pebble-bed fluoride-salt-cooled high-temperature reactor (FHR) design, TRISO fuel pebbles sit in a FLiBe (lithium fluoride and beryllium fluoride) coolant, and an off-gas system removes gaseous radionuclides from the primary coolant; the TRISO particles are part of the containment of radionuclides [ref:andrews2021]. That report is a functional review that starts the identification of the phenomenological gaps still hindering a mechanistic source term for such reactors [ref:andrews2021].
- The nearest operating experience read is the Molten Salt Reactor Experiment, in which the fuel was dissolved in a circulating fluoride salt, a different configuration from a TRISO particle in FLiBe coolant. Its fission products fell into distinct chemical groups: krypton and xenon were stripped to the off-gas and diffused into the graphite; Rb, Cs, Sr, Ba, Y, Zr and the lanthanides formed stable fluorides that stayed in the salt; iodine was indicated, less certainly, to stay in the salt; and the noble metals Nb, Mo, Tc, Ru, Ag, Sb and Te deposited on metal, graphite and the salt-gas interface [ref:compere1975].

What a later treatment would need before it could start (owner answer of 2026-11-24 for the roadmap): FLiBe or cover-gas data from a cited source, and a transport model for that environment, kept separate from the gas-cooled AGR conditions. In more detail, for each species group:

- the chemical form and solubility of the species in FLiBe under the coolant's redox condition (the MSRE grouping applies to a fuel salt and would have to be confirmed for a coolant salt);
- a boundary condition at the outer surface of the particle or pebble that replaces STRATO's absorbing surface by the exchange with the salt;
- for Kr and Xe, the stripping from the salt to the cover gas and the off-gas path [ref:andrews2021] [ref:compere1975];
- for the noble metals (Ru, Ag), deposition on surfaces and its holdup [ref:compere1975];
- the decay of every radioactive nuclide along the path, with half-lives from the register or an evaluated source.

**Gas-cooled AGR conditions are a different data set.** The AGR irradiations were made in the Advanced Test Reactor in capsules swept by an inert gas, with a helium-neon mixture used for temperature control [ref:davenport2016] [ref:scates2010], and the AGR-1 safety tests heated compacts in flowing helium [ref:morris2016]. None of these data describes a salt environment, and STRATO never shows AGR results and any FLiBe or cover-gas result as one data set.

## Candidate governing equations (research concepts)

**Part 1.** The species use STRATO's Phase 2 layered equation ((P21.1) in `docs/mathematics/STRATO_Phase2_Derivations.md`) once their decay rows, yields and diffusivities are registered; the species entries list what is missing. For the fission gases, the population form used for AGR-1 combines failed particles and contamination (eq. 6 of [ref:scates2010]; see the [xenon entry](species-xe.md)):

$$
\left(\frac{R}{B}\right)_i^{\mathrm{total}} = f_{\mathrm{fail}}\left(\frac{R}{B}\right)_i^{\mathrm{fail}} + f_{\mathrm{cont}}\left(\frac{R}{B}\right)_i^{\mathrm{contamination}}
$$

**Part 2.** No equation is proposed. None of the sources read gives a transport law for a species leaving a TRISO particle into FLiBe or from FLiBe into a cover gas, and the catalogue does not write one without a cited source.

## Required parameters

| Parameter | In a source? | In STRATO's register? |
|---|---|---|
| Decay data for Eu-155, Ru-103, Ru-106, Xe-133, Xe-135, I-133, I-135, Kr-88 | evaluated data [ref:iaea_livechart_fp] | no (Sr-89, Sr-90, Eu-154, I-131, Kr-85, Kr-85m are registered in `data/nuclear/decay.csv`) |
| Fission yields for Eu-155, Ru, Xe and I-133/I-135 products | not read for this entry | no |
| Layer diffusivities: Eu, Ru (all layers); I (PyC, SiC) | none found | no |
| Layer diffusivities: Sr, Kr, Xe | TECDOC-978 [ref:iaea1997] | yes, with gaps (no FRG Kr or Xe row for SiC) |
| Contamination fraction and failed fraction | AGR-1 values in [ref:scates2010] [ref:morris2016] | no |
| Matrix retention parameters | some TECDOC-978 matrix rows [ref:iaea1997] | yes (`A4-*` rows), outside the particle domain |
| FLiBe and cover-gas properties per species | not found | no |

## Supporting literature

Part 1: [ref:demkowicz2016], [ref:morris2016], [ref:scates2010], [ref:iaea1997], [ref:collin2016]. Part 2: [ref:andrews2021], [ref:compere1975]; gas-cooled conditions: [ref:davenport2016].

**Where the sources disagree or do not transfer.** Semi-empirical gas-release models under-predicted the AGR-1 R/B from contamination [ref:scates2010], and PARFUME over-predicted the safety-test release of strontium and krypton by orders of magnitude [ref:collin2016]. The MSRE grouping comes from a fuel salt [ref:compere1975] and cannot be transferred to a coolant salt around TRISO particles without new evidence.

## Missing calculations or experiments

- Register rows (decay, yield, diffusivity) for the species and layers listed above.
- A matrix region or a documented boundary condition for retention outside the particle.
- A cited data set and a transport model for FLiBe and the cover gas (planned for a future phase).
- Atomistic calculations for these species in SiC (gallery items `ms-sr-eu-sic` and `ms-noble-gas-sic`), none performed.

## Interactions with other regimes

- **Regime A.** Retention by intact layers differs strongly between species: compare caesium ([brick 7](cs_retention_barrier_integrity.md)), strontium and europium, and the gases [ref:demkowicz2016] [ref:morris2016].
- **Regime C.** Gas and iodine release is driven by coating failure ([brick 8](coating_degradation_failure.md)); downstream transport receives what both regimes release.
