# Regime C: barrier failure and bypass transport

Regime C covers transport that bypasses a coating barrier: through cracks, connected pores or a layer that has failed, or from particles that were defective when made. The regimes are transport regimes, not mutually exclusive mechanisms; in a population of particles most can be in Regime A while a few are in Regime C.

## Mechanisms to investigate

| Mechanism | What it means | Catalogue entry |
|---|---|---|
| Microcracking | cracks in SiC or pyrocarbon that open a faster path | brick 8 (not written in this entry) |
| Connected transport pathways | pores, nanocracks or voids that link across a layer | brick 8 (not written in this entry) |
| Coating failure | loss of the SiC barrier, or of all three dense layers | brick 8 (not written in this entry); palladium route in [entry](pd_assisted_degradation.md) |
| Manufacturing defects | particles with missing or defective layers from fabrication | brick 8 (not written in this entry) |
| Fission-gas pressure | internal gas pressure loading the coating layers | brick 8 (not written in this entry) |
| Failed-particle release | release from particles whose barrier has failed | brick 8 (not written in this entry) |

## Evidence relevant to silver and palladium

- **Failure mechanisms in fuel performance codes.** PARFUME considers IPyC cracking, IPyC-SiC debonding, internal fission-gas pressure, kernel-SiC interaction through the amoeba effect, and SiC thinning by fission product-SiC interaction; pressure-vessel failure occurs when gas pressure makes the tangential stress in the SiC exceed its strength [ref:miller2018].
- **Failed layers in transport.** PARFUME sets the diffusivity of any layer predicted to fail to an arbitrarily large value [ref:miller2018].
- **AGR-1.** Zero TRISO failures (all three dense layers) were detected during irradiation from fission-gas release-to-birth ratios; an estimated four of about 298,000 particles lost SiC integrity and released caesium [ref:demkowicz2016]. In the caesium-releasing particles the IPyC had cracked and the exposed SiC was attacked by palladium; SiC failures after safety testing at 1600 to 1800 °C were related to fabrication defects or extensive palladium corrosion [ref:hunn2016].
- **Silver.** Vapour transport of silver through cracks or nanocracks in SiC has been proposed as a dominant release path, supported by ab initio results that silver adsorbs on SiC surfaces and moves easily along them [ref:xiao2012]; it is one of several competing proposals [ref:simon2022].
- **Failed-particle experiments.** AGR-3/4 included designed-to-fail particles to provide a fission-product source for measuring retention in compact matrix and graphite, and to give data on fission-gas release from failed particles [ref:grover2013]. AGR-2 post-irradiation examination distinguishes SiC failures (SiC failed, at least one pyrocarbon layer intact) from TRISO failures [ref:stempien2021].

All AGR irradiations used capsules with an inert helium-neon sweep gas for temperature control in the Advanced Test Reactor [ref:davenport2016]: gas-cooled conditions. Transport of released species in FLiBe or a cover gas does not exist in STRATO today and is planned for a future phase; it is a separate topic from these gas-cooled results.

## Interactions with other regimes

- Regime B processes are common routes into Regime C: in AGR-1 the IPyC cracked through radiation-induced dimensional change of the shrinking buffer, and palladium then attacked the exposed SiC [ref:hunn2016] [ref:demkowicz2016].
- Connected nano-porosity is the limit beyond which the pore-trap picture of Regime A no longer holds [ref:abdulhameed2026].

## What STRATO would need

Regime C needs a description of a particle population (intact, SiC-failed, TRISO-failed, as-fabricated defective) and a rule for the transport properties of failed layers, with failure fractions taken from cited data. STRATO models one intact particle today. Research statuses of Regime C mechanisms are held only in `data/research/mechanisms.csv`; most Regime C mechanisms have no catalogue row yet.
