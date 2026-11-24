# Iodine (I)

Species entry of the STRATO research catalogue. Transport regimes: A and C. Reference fuel: AGR-1 UCO TRISO. STRATO does not simulate iodine today; nothing on this page is a STRATO result.

## Why it matters

Radioiodine is a safety-relevant fission product. In TRISO fuel its release is expected to follow that of the fission gases: the release of fission gases and iodine from coated particles is widely expected to be directly correlated with a defective or failed coating, an intact SiC layer keeps the particle gas-tight, and an intact OPyC significantly delays gaseous fission products that have passed the SiC; it has been standard practice, supported by experimental evidence, to assume that fission gases and iodine show a similar release behaviour under accident conditions [ref:iaea1997].

**Deferred in STRATO.** The owner deferred I-131 from the first Phase 2 nuclides (owner decision D1, 2026-11-24) because the parameter register holds no PyC or SiC iodine diffusivity. Caesium, strontium and krypton come first.

Isotopes (register `data/nuclear/decay.csv` unless marked otherwise):

| Nuclide | Stable or radioactive | Half-life | Origin |
|---|---|---|---|
| I-127 | stable (isotopic abundance 100 %); not in the register [ref:iaea_livechart_fp] | none | listed for completeness |
| I-129 | radioactive, β− 100 % | 1.57 × 10⁷ y (uncertainty 0.04 × 10⁷ y); not in the register [ref:iaea_livechart_fp] | long-lived fission product; listed for completeness |
| I-131 | radioactive, β− 100 % to Xe-131 | 8.0252 d | cumulative fission product via Te-131 and Te-131m (register) |
| I-133 | radioactive, β− 100 % | 20.83 h; not in the register [ref:iaea_livechart_fp] | listed for completeness |
| I-135 | radioactive, β− 100 % | 6.58 h; not in the register [ref:iaea_livechart_fp] | fission-product radioiodine, parent of Xe-135 [ref:scates2010] |

The register decay constant of I-131 is 9.996681 × 10⁻⁷ s⁻¹. Its precursor Te-131m (33.25 h) delays the I-131 birth: the repository's derivations give a time of 9.2 d for the precursor feed to reach 99 % of equilibrium, comparable to the I-131 half-life, so the instantaneous cumulative-yield source of Phase 2 is only approximate for I-131 (`docs/mathematics/STRATO_Phase2_Derivations.md`, Section 3.3). The AGR-1 paper gives 6.57 h for I-135 [ref:scates2010], within the uncertainty of the evaluated 6.58(3) h [ref:iaea_livechart_fp].

## Relevant TRISO layers

- **Kernel.** Register rows `A1-05` (normal operating conditions), `A1-06` and `A1-07` (accident conditions, two branches) are TECDOC-978 rows for iodine, krypton and xenon together in UO2 [ref:iaea1997]; rule E3 applies to UCO.
- **Buffer.** Iodine and xenon release from a German irradiation and heating experiment was interpreted as rapid emission from traps in the kernel and in the buffer, the buffer being loaded by recoil from the kernel surface [ref:iaea1997].
- **IPyC, SiC, OPyC.** No iodine diffusivity is registered for any of these layers. This is the reason for the deferral.
- **Matrix and structural graphite.** Rows `A4-06` (iodine in pores), `A4-07` (iodine in grains, only $D/r^2$ transferable) and `A5-04` [ref:iaea1997]; outside STRATO's particle domain.

## Candidate release mechanisms

1. **Release through failed coatings** (Regime C): the dominant expectation in the literature, as for the fission gases [ref:iaea1997].
2. **Trap emission in kernel and buffer** during temperature transients: the fractional release is described by two empirical functions, a gradual emission from traps and a fraction that escapes the traps in a burst [ref:iaea1997]. The related HRB model of fission-gas and iodine release from defective particles during temperature transients is coded in TRAGAS [ref:iaea1997].
3. **Release from heavy-metal contamination** outside the particles, which in AGR-1 dominated the measured fission-gas release (see [Xenon](species-xe.md)) [ref:scates2010].

## Current evidence

- **AGR-1 irradiation.** I-135 released from the fuel and deposited upstream was estimated after each reactor cycle from the Xe-135m in the sweep gas (Xe-135m is populated through a 16.4 % branch of I-135); the ratio of these values to the calculated I-135 inventory followed the preliminary capsule temperatures and indicated no fuel failures [ref:scates2010].
- **No coating diffusivities.** None was found in the sources read for this entry, and none is registered.

## Proposed governing-equation extensions

Iodine would use the Phase 2 layered equation ((P21.1)) with the I-131 decay constant and the cumulative-yield source, subject to the precursor caveat above. Before that can be written for STRATO it needs PyC and SiC iodine diffusivities from a cited source. A trap model for kernel and buffer, as in the HRB approach [ref:iaea1997], would add coupled equations and is a separate research question.

## Potential Materials Studio calculations

Planned: iodine, krypton and xenon at vacancy sites and in small cavities of 3C-SiC (gallery item `ms-noble-gas-sic`). Not performed; planned for a future phase of the project; Pending placeholder only. Iodine chemistry with caesium (for example CsI) is not covered by this item.

## Experimental comparison opportunities

- **AGR-1.** End-of-cycle I-135 release estimates from Xe-135m measurements [ref:scates2010]; not in the repository's measurement register.
- No iodine release rows exist in `data/experimental/release_measurements.csv`.

The AGR experiments were gas-cooled [ref:davenport2016]. In the Molten Salt Reactor Experiment, whose fluoride fuel salt carried the fuel itself (not TRISO particles in a coolant), iodine was indicated, with less certainty because of a deficient material balance, to remain in the salt [ref:compere1975]. Iodine in FLiBe or a cover gas does not exist in STRATO today and is planned for a future phase; it is not compared with AGR data.

## Research limitations

- No PyC or SiC iodine diffusivity in the register (reason for the deferral, decision D1).
- I-133, I-135, I-127 and I-129 are not in the decay register.
- The Te-131m precursor makes an instantaneous I-131 source approximate.
- AGR-1 iodine data are end-of-cycle estimates derived from a daughter nuclide [ref:scates2010].

## Implementation status

Deferred from the Phase 2 physics (decision D1). Its catalogue row is in `data/research/mechanisms.csv` (`additional_species_downstream_transport`, species `I`). STRATO's live simulation does not include iodine.
