# Xenon (Xe)

Species entry of the STRATO research catalogue. Transport regimes: A and C. Reference fuel: AGR-1 UCO TRISO. STRATO does not simulate xenon today; nothing on this page is a STRATO result.

## Why it matters

Xenon and krypton are the fission gases whose release reveals failed TRISO coatings: released fission gases are taken as the indicator of a broken coating, because an intact SiC layer keeps the particle gas-tight [ref:iaea1997]. In AGR-1 a fission product monitoring system measured krypton and xenon isotopes in the sweep gas of every capsule throughout the irradiation and expressed them as release-to-birth ratios (R/B) [ref:scates2010]; the low R/B values showed zero TRISO-coating failures among about 2.98 × 10⁵ particles [ref:demkowicz2016].

Isotopes (none of these xenon nuclides is in the repository's decay register; values from [ref:iaea_livechart_fp]):

| Nuclide | Stable or radioactive | Half-life |
|---|---|---|
| Xe-131, Xe-132 | stable (isotopic abundances 21.232 %, 26.909 %) | none |
| Xe-133 | radioactive, β− 100 % | 5.2475 d (uncertainty 0.0005 d) |
| Xe-135 | radioactive, β− 100 % | 9.14 h (uncertainty 0.02 h) |

The register knows xenon only as a precursor (Xe-137 in `data/nuclear/fission_yields.csv`, parent of Cs-137; Xe-133, parent of stable Cs-133) and as the stable shield nuclide Xe-134 of Cs-134 (register notes).

## Relevant TRISO layers

- **Kernel.** Register rows `A1-05`, `A1-06`, `A1-07` (iodine, krypton and xenon together, UO2) [ref:iaea1997], and `T71-04`, `T71-05`, which the register takes from IAEA TECDOC-1674 (IAEA, 2012; not a catalogue reference); rule E3 applies to UCO.
- **IPyC and OPyC.** Rows `A2-04`, `A2-05` (FRG, two branches) and `A2-09`, `A2-10` (USA), krypton and xenon together [ref:iaea1997]. An intact OPyC significantly delays gaseous fission products that have passed the SiC [ref:iaea1997].
- **SiC.** Only Japanese xenon rows exist in the register (`A3-17`, and `A3-18`, `A3-19` whose nuclide label is inferred) [ref:iaea1997]; the FRG data set chosen by owner decision D4 has no xenon row for SiC.
- **Matrix and structural graphite.** Row `A5-14` (xenon in structural graphite, only $D/r^2$ transferable); outside STRATO's particle domain.

## Candidate release mechanisms

1. **Release from particles with failed coatings** (Regime C) [ref:iaea1997] [ref:scates2010].
2. **Release from heavy-metal contamination** outside the particles: in AGR-1 all measured releases were consistent with, and assumed to come from, uranium contamination, and comparison with models indicated gas diffusing from "tramp" uranium [ref:scates2010]. The source places this contamination in the graphite of the capsule and the matrix surrounding the fuel, while its conclusion speaks of tramp uranium "inside the TRISO coating layers"; the location is therefore not settled by this source [ref:scates2010].
3. **Diffusion and recoil from intact particles** (Regime A): the R/B models used for AGR-1 include terms for failure, contamination and direct recoil of fission fragments, with diffusion modelled by the Booth equivalent-sphere model, which depends on the diffusion coefficient and the half-life of the isotope [ref:scates2010].
4. **Trap emission in kernel and buffer** during temperature transients, as interpreted for xenon and iodine in a German experiment [ref:iaea1997].

Species-specific mechanisms are grouped in [Additional species and downstream transport](additional_species_downstream_transport.md) (brick 9).

## Current evidence

- **AGR-1 R/B.** The R/B values indicated excellent fuel performance compared with historical US and modern German experience; two semi-empirical models (German, with separate krypton and xenon diffusion relationships, and Japanese, with one relationship for both) under-predicted the R/B from contamination, which the authors attribute partly to uncertainty in the contamination fraction and to the lower compact density of AGR-1 [ref:scates2010].
- **Safety tests.** Xenon release is not reported in the AGR-1 safety-test summary, which monitored Kr-85 [ref:morris2016].

## Proposed governing-equation extensions

A short-lived gas such as Xe-133 or Xe-135 reaches a release-to-birth equilibrium during irradiation, which is how AGR-1 reports it [ref:scates2010]. STRATO's Phase 2 layered equation ((P21.1)) with decay and source would give R/B for a single intact particle once a xenon decay row, yield and diffusivities are registered. The population form used for AGR-1 is (research concept, not solved by STRATO; eq. 6 of [ref:scates2010]):

$$
\left(\frac{R}{B}\right)_i^{\mathrm{total}} = f_{\mathrm{fail}}\left(\frac{R}{B}\right)_i^{\mathrm{fail}} + f_{\mathrm{cont}}\left(\frac{R}{B}\right)_i^{\mathrm{contamination}}
$$

with $f_{\mathrm{fail}}$ the fraction of failed TRISO particles and $f_{\mathrm{cont}}$ the contamination fraction (grams of contamination uranium over grams of uranium) [ref:scates2010]. STRATO models one intact particle and has neither term today.

## Potential Materials Studio calculations

Planned: xenon (with krypton and iodine) at vacancy sites and small cavities in 3C-SiC (gallery item `ms-noble-gas-sic`). Not performed; planned for a future phase of the project; Pending placeholder only.

## Experimental comparison opportunities

- **AGR-1 irradiation.** R/B time histories of Kr-85m, Kr-88 and Xe-135 for all six capsules [ref:scates2010]; not in the repository's measurement register.
- No xenon release rows exist in `data/experimental/release_measurements.csv`.

The AGR R/B data come from a gas-cooled irradiation with an inert sweep gas [ref:scates2010] [ref:davenport2016]. In the Molten Salt Reactor Experiment the noble gases krypton and xenon were stripped from the fuel salt to the off-gas and diffused into the moderator graphite; most of the short-lived noble-gas nuclides decayed in the fuel salt [ref:compere1975]. That reactor dissolved the fuel in the salt, unlike a TRISO particle in FLiBe coolant. Xenon in FLiBe or a cover gas does not exist in STRATO today and is planned for a future phase.

## Research limitations

- No xenon nuclide is in the decay register; the only xenon yield rows (Xe-137) are registered as precursors of Cs-137, not as xenon sources.
- The SiC xenon rows are Japanese only, and one of them carries an inferred label.
- AGR-1 gas release is dominated by contamination, not by intact particles [ref:scates2010].

## Implementation status

Not part of the Phase 2 physics. Its catalogue row is in `data/research/mechanisms.csv` (`additional_species_downstream_transport`, species `Xe`). STRATO's live simulation does not include xenon.
