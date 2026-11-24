# Irradiation and coating degradation (defect evolution, microcracking, failure)

Brick 8. Species: all (the coating state changes the transport of every species). Material domain: the whole particle (buffer, IPyC, SiC, OPyC and their interfaces). Regimes: B (irradiation defects, microstructural degradation, dimensional change) leading into C (microcracking, coating failure, manufacturing defects, fission-gas pressure, failed-particle release). Every equation below is a research concept, not solved by STRATO.

## Proposed physics

**Defect evolution.** Fast-neutron damage changes the SiC. In SiC from TRISO particles irradiated in three reactors, the content of caesium, cerium, europium, ruthenium and antimony was proportional to the number of neutrons that had passed through the SiC; the source conjectures that damage from slowing-down neutrons creates sites at which fission products bind, or paths by which they enter the coating [ref:iaea1997]. Carbon-ion irradiation greatly enhanced silver diffusion in SiC diffusion couples [ref:leng2016]. The FRG caesium diffusivity in SiC carries an empirical fluence factor [ref:iaea1997].

**Dimensional change and stress.** Densification of the buffer by fission-product recoil and neutron irradiation is unavoidable at the burnup and dose of AGR-1 [ref:hunn2016]. In the particle model of PARFUME the pyrocarbon layers shrink and creep under irradiation: early in irradiation their shrinkage puts the SiC in compression, creep later relieves it, and the growing fission-gas pressure puts the layers in tension; a pressure-vessel failure occurs if the tangential stress in the SiC exceeds the strength of that particle's SiC [ref:miller2018]. PARFUME also treats multi-dimensional behaviour seen in examinations: radial cracking of the IPyC, partial debonding of the IPyC from the SiC, aspherical particles and the amoeba effect [ref:miller2018].

**Microcracking in AGR-1.** In 1288 cross-sectioned AGR-1 particles, fractures became less frequent from the buffer outward: 22 % of buffers fractured, IPyC fractures were found in 3 % of particles (8.1 % in Variant 1, with lower IPyC density, against 1.8 % in the other fuel types), partial-thickness SiC cracks in 0.4 % of SiC layers, no fracture through the entire SiC and none in the OPyC [ref:demkowicz2016]. Where the buffer stayed bonded to the IPyC, buffer fractures propagated into the IPyC; complete buffer-IPyC delamination appeared advantageous [ref:demkowicz2016]. No kernel migration (amoeba effect) was seen, which the authors attribute to the UCO kernel chemistry [ref:demkowicz2016].

**Failure.** Zero TRISO failures were detected during AGR-1, and the in-pile SiC failure fraction was 1.3 × 10⁻⁵ [ref:demkowicz2016]. Apart from two particles with as-fabricated defects, every SiC failure examined occurred where IPyC cracking had exposed the inner surface of the SiC, which then suffered localised palladium corrosion [ref:hunn2016]. In the safety tests, the estimated SiC failures were 3 of about 33,100 particles at 1600 °C, about 4 of 8300 at 1700 °C and about 23 of 16,500 at 1800 °C, and two particles lost all three outer dense layers at 1800 °C [ref:morris2016].

## Candidate governing equations (research concepts)

This mechanism needs additional coupled equations (a stress analysis of the coating layers and a failure model) and must not be inserted into STRATO's diffusion equation as one extra term.

**(a) Failure probability of a layer (PARFUME Eq. (1-3)).** Assuming Weibull statistics, the failure probability of a layer such as the SiC under the computed stress field $\sigma$ is [ref:miller2018]

$$
P_f = 1 - \exp\!\left[-\int_V \left(\frac{\sigma}{\sigma_0}\right)^{m} dV\right]
$$

with $\sigma_0$ the characteristic strength and $m$ the Weibull modulus, both determined from experimental data [ref:miller2018].

**(b) Pressure-vessel criterion.** For a one-dimensional spherical particle, failure occurs when the tangential stress in the SiC reaches that particle's SiC strength [ref:miller2018]:

$$
\sigma_{t,\mathrm{SiC}}(t) \ge \sigma_{\mathrm{SiC}}
$$

The stress $\sigma_{t,\mathrm{SiC}}(t)$ comes from PARFUME's analytical solution that accounts for irradiation-induced creep and swelling of the pyrocarbon layers, loaded by the internal fission-gas pressure [ref:miller2018]; STRATO has no mechanical model.

**(c) Change of transport properties on failure.** A failed layer is given a large diffusivity, as in [Caesium retention and barrier integrity](cs_retention_barrier_integrity.md) (c) (register rows `C16-23`, `H21-10`; [ref:miller2018]), and a population of particles is combined as in (b) of that entry.

**(d) Defect-dependent transport.** No quantitative law tying a SiC defect state to the effective diffusivity of the AGR-1 species was found, other than the empirical caesium fluence factor [ref:iaea1997] and the flux-dependent silver model discussed in [Regime B](regime-b.md) [ref:aagesen2022]. This item stays empty until a cited law exists.

## Required parameters

| Parameter | In a source? | In STRATO's register? |
|---|---|---|
| Weibull characteristic strength and modulus of SiC (and PyC) | PARFUME material correlations, Section 7 [ref:miller2018] (not transcribed here) | no |
| Irradiation-induced dimensional change and creep of PyC; buffer densification | same [ref:miller2018] | no |
| Fission-gas and CO pressure in the particle | PARFUME CO production and gas release models [ref:miller2018] | no |
| Fast-neutron fluence and flux histories | AGR-1 compact fluences [ref:morris2016] | no |
| Layer thicknesses and their distributions | AGR-1 characterisation | thicknesses yes (`data/geometry/geometry.csv` G-01 to G-05); distributions no |
| Interface bond strengths (buffer-IPyC, IPyC-SiC) | qualitative [ref:demkowicz2016] | no |
| As-fabricated defect fractions | not in the sources read | no |

## Supporting literature

[ref:miller2018] (PARFUME failure mechanisms and Weibull statistics), [ref:demkowicz2016] (AGR-1 layer fractures and failure fraction), [ref:hunn2016] (causes of SiC failure), [ref:morris2016] (safety-test failures), [ref:iaea1997] (irradiation and fission-product uptake in SiC, fluence factor), [ref:leng2016] and [ref:aagesen2022] (irradiation-enhanced transport).

**Where the literature disagrees, or where the model differs from the observation.** PARFUME's classical failure mechanism is the pressure-vessel failure of a spherical particle [ref:miller2018], while the AGR-1 SiC failures examined were local, caused by IPyC cracks tied to buffer behaviour and followed by palladium attack [ref:hunn2016]. Fabrication defects caused the two 1600 °C safety-test failures in the ORNL tests [ref:morris2016]. A model of this brick would have to reproduce local, rare failures (about 10⁻⁵ per particle in pile [ref:demkowicz2016]), which a mean-particle model does not.

## Missing calculations or experiments

- A mechanical model of the coating layers in STRATO (none exists), or a documented way to take failure fractions and times from data.
- Registered material correlations for PyC and SiC under irradiation, with sources.
- Fluence and flux histories per AGR-1 compact.
- A defect-to-transport law for any species other than the empirical caesium factor.
- Atomistic defect calculations in SiC (gallery item `ms-sic-intrinsic-defects`, planned, not performed).
- Derivation of the layer-state switch with conservation checks, and verification.

## Interactions with other regimes

- **Regime A.** Defects and dimensional change modify the effective diffusivities that Regime A uses; the caesium fluence factor is one example [ref:iaea1997].
- **Regime B.** Palladium attack is the second step of the dominant AGR-1 SiC failure path, after IPyC cracking [ref:hunn2016]; see [Palladium-assisted degradation](pd_assisted_degradation.md).
- **Regime C.** Failures move particles into Regime C; the release from such particles is treated in [Caesium retention and barrier integrity](cs_retention_barrier_integrity.md), the [krypton](species-kr.md) and [xenon](species-xe.md) entries and [Regime C](regime-c.md).
