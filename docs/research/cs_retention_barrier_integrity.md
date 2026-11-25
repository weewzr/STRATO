# Caesium retention and barrier integrity

Brick 7. Species: Cs. Material domain: the SiC layer, with the particle population it belongs to. Regimes: A (retention by intact SiC) and C (release through failed or defective SiC); the SiC failures themselves come from Regime B processes and are treated in [Irradiation and coating degradation](coating_degradation_failure.md) (brick 8). Every equation below is a research concept, not solved by STRATO. The caesium species entry is [species-cs](species-cs.md).

## Proposed physics

Caesium diffuses with relative ease through intact pyrocarbon but is effectively retained by intact SiC, so caesium release measures SiC integrity [ref:demkowicz2016]. Two populations of particles therefore matter:

1. **Particles with intact SiC.** Release is extremely low: below 3 × 10⁻⁶ of the compact inventory during the AGR-1 irradiation [ref:demkowicz2016], below 5 × 10⁻⁶ after 300 h at 1600 °C or 100 h at 1800 °C in the safety tests, often hard to separate from background contamination [ref:morris2016].
2. **Particles with failed or defective SiC.** They release a large but variable fraction: 0.38 to 0.65 of the Cs-134 in the three in-pile SiC failures located in Capsule 5 compacts (of four estimated in pile; the fourth was in Compact 6-3-2) [ref:demkowicz2016]; particles with failed SiC did not release all their caesium in the safety tests either, and what they released varied [ref:morris2016].

The measured compact release is therefore governed by the number of particles with failed SiC, not by diffusion through intact SiC [ref:demkowicz2016] [ref:morris2016]. Part of the released caesium stays in the OPyC and compact matrix [ref:demkowicz2016].

Brick 7 asks for species-specific comparisons with experimental evidence. For caesium that means comparing a model of both populations with the AGR-1 compact data, with the failure count taken from the examinations, not fitted.

## Candidate governing equations (research concepts)

The intact-particle part is the layered diffusion equation of STRATO's Phase 2 ((P21.1) in `docs/mathematics/STRATO_Phase2_Derivations.md`); this brick adds the items below. The population is a set of separate single-particle problems, not a coupled system.

**(a) Fluence-dependent caesium diffusivity in SiC (FRG).** TECDOC-978 Table A-3 gives two Arrhenius branches, the first multiplied by a fluence factor [ref:iaea1997]:

$$
D_{\mathrm{Cs,SiC}}(T,\Gamma) = 5.5\times10^{-14}\,e^{\Gamma/5}\exp\!\left(-\frac{125\ \mathrm{kJ\,mol^{-1}}}{R\,T}\right) + 1.6\times10^{-2}\exp\!\left(-\frac{514\ \mathrm{kJ\,mol^{-1}}}{R\,T}\right)\ \mathrm{m^2\,s^{-1}}
$$

with $\Gamma$ the fast-neutron fluence in $10^{25}\ \mathrm{m^{-2}}$ for $E > 16$ fJ, marked in the source as reactor-specific [ref:iaea1997], and $R$ the gas constant (SI by default, 8.3143 selectable; owner decision D4). This form is already derived as (P22.9) and belongs to Phase 2.2, not to this brick; it is repeated here because the barrier-integrity comparison depends on it.

**(b) Particle population.** PARFUME models intact particles and particles with failed layers separately and combines the release fractions for each compact [ref:collin2015]. Written for a compact of $N$ particles of which $n_{\mathrm{f}}$ have failed SiC (a STRATO research proposal that states this combination; the symbols are not from the source):

$$
F_{\mathrm{compact}}(t) = \left(1-\frac{n_{\mathrm{f}}}{N}\right)F_{\mathrm{intact}}(t) + \frac{n_{\mathrm{f}}}{N}\,F_{\mathrm{failed}}(t)
$$

where $F_{\mathrm{intact}}$ and $F_{\mathrm{failed}}$ are single-particle release fractions computed with the intact and failed SiC properties, and each failed particle may fail at a different time. One AGR-1 particle corresponds to a compact fraction of about 2.4 × 10⁻⁴ [ref:morris2016].

**(c) Failed SiC layer.** In PARFUME and in the BISON data set the diffusivity of a failed layer is set to a large value, 1 × 10⁻⁶ m² s⁻¹ (register rows `data/diffusivity/arrhenius.csv#C16-23`, `#H21-10`; [ref:miller2018]):

$$
D_{\mathrm{SiC}} = D_{\mathrm{failed}} \quad \text{for } t \ge t_{\mathrm{fail}}
$$

**(d) Spread of retention.** IMGA measurements show particle-to-particle variation of caesium retention, including a bimodal distribution after one German heating test; one analysis replaced the single diffusion coefficient by a log-normal distribution around the average [ref:iaea1997]. No distribution parameters for AGR-1 SiC were found; this item stays descriptive.

## Required parameters

| Parameter | In a source? | In STRATO's register? |
|---|---|---|
| FRG Cs-in-SiC branches and fluence factor | [ref:iaea1997] | yes (`A3-01`, `A3-02`) |
| Fast-neutron fluence per compact, converted to the 16 fJ cutoff | AGR-1 fluences reported for another cutoff | fluences no; conversion recorded as not verified in the derivations |
| Number and timing of particles with failed SiC per compact | counts and estimates per test [ref:morris2016] [ref:demkowicz2016] | no |
| Failed-layer diffusivity | [ref:miller2018] (arbitrary large value) | yes (`C16-23`, `H21-10`) |
| Caesium diffusivities in UCO kernel and PyC | UO2 values only [ref:iaea1997] | yes, UO2 rows (`A1-01`, `A1-02`, `A2-01`; rule E3) |
| OPyC and matrix retention of released caesium | qualitative [ref:demkowicz2016] | matrix row `A4-01`, outside the particle domain |
| Distribution of SiC diffusivities | method only [ref:iaea1997] | no |

## Supporting literature

[ref:demkowicz2016] (AGR-1 irradiation, in-pile SiC failures), [ref:morris2016] (AGR-1 safety tests), [ref:hunn2016] (causes of SiC failure), [ref:collin2015] and [ref:collin2016] (PARFUME comparisons), [ref:iaea1997] (data sets, fluence factor, IMGA distributions), [ref:miller2018] (failed-layer treatment), [ref:rabone2015] (atomistic caesium at boundaries).

**Where the literature disagrees.** The US reference model assumes no caesium diffusion through SiC, while US, German, Japanese and Russian diffusion data exist [ref:iaea1997]. PARFUME over-predicted AGR-1 caesium release from intact particles by up to a factor of 100 in pile [ref:collin2015] and, with the TECDOC-978 values of its Table 2 (register rows `C16-05` to `C16-10`), by up to five orders of magnitude in the safety tests [ref:collin2016]; the authors derived tentative correction factors, which STRATO does not use as defaults (owner decision D4). Atomistic boundary diffusion coefficients are far slower than experiment [ref:rabone2015]. The causes of SiC failure also differ by temperature: fabrication defects at 1600 °C, palladium attack after IPyC cracking at 1700 and 1800 °C in the analysed particles [ref:morris2016].

## Missing calculations or experiments

- A failure count and failure times per AGR-1 compact registered as data (with their source), so that (b) is driven by measurements.
- A registered fluence per compact and a verified conversion of the fluence cutoff.
- A STRATO derivation of (b) and (c) with conservation across the change of layer state, and verification tests.
- A matrix region, or a documented boundary condition, for the caesium held outside the SiC.
- Atomistic caesium calculations at SiC boundaries (gallery item `ms-cs-sic-gb`); none performed.

## Interactions with other regimes

- **Regime B.** IPyC cracking from buffer densification exposes the SiC to concentrated palladium attack, which caused most of the analysed SiC failures [ref:hunn2016]; see [Palladium-assisted degradation](pd_assisted_degradation.md) and [Irradiation and coating degradation](coating_degradation_failure.md). The fluence factor in (a) is itself an irradiation effect [ref:iaea1997].
- **Regime C.** A particle with failed SiC but intact pyrocarbon releases caesium but retains krypton [ref:morris2016]; see [Regime C](regime-c.md) and the [krypton entry](species-kr.md).
