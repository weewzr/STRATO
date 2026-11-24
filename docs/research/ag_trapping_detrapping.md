# Silver trapping and detrapping in SiC

Brick 4. Species: Ag. Material domain: the SiC layer. Regime A (trapping and detrapping in an intact barrier); traps created by irradiation damage connect it to Regime B. Every equation below is a research concept, not solved by STRATO.

## Proposed physics

Trapping is the capture of diffusing atoms at sites where the energy needed to move on is higher than for ordinary diffusion; such sites can come from impurities, structural irregularities, precipitated phases including gas bubbles, or neutron displacement damage [ref:longhurst2004]. A trapped atom stops contributing to transport until it is released, so trapping lowers the apparent diffusivity, delays release and stores inventory in the layer.

For silver in SiC, the sites proposed in the literature are:

- **Nano-pores.** Silver in TRISO coatings has been reported to occupy both grain boundaries and isolated nano-pores; ab initio work shows silver adsorbs strongly on SiC surfaces and moves along them easily, so pore surfaces can act as reversible traps whose escape is limited by desorption [ref:abdulhameed2026] [ref:xiao2012].
- **Triple junctions.** Triple junctions may act as trapping sites for silver; a model with only grain-boundary paths would not capture this [ref:simon2022]. Silver precipitates have been observed at SiC triple points in irradiated AGR-1 particles [ref:vanrooyen2014a] [ref:lillo2015].
- **Implantation or irradiation damage.** In ion-implanted single-crystal SiC, silver may be trapped in the implantation-damage region [ref:szlufarska2016].

## Candidate governing equations (research concepts)

### Concept 1: coupled mobile-trapped system (TMAP7)

TMAP7, an INL code written for tritium transport, conserves the mobile concentration $C_s$ of species $s$ together with the trapped concentration $C_s^{t_k}$ in each trap type $k$, including radioactive decay of mobile and trapped atoms [ref:longhurst2004]:

$$
\frac{\partial C_s}{\partial t} = -\nabla\cdot J_s + S_s - \sum_k\left(\frac{\partial C_s^{t_k}}{\partial t} + \upsilon_s C_s^{t_k}\right) - \upsilon_s C_s + \sum_m \upsilon_m^{s}\left(C_m + \sum_k C_m^{t_k}\right)
$$

$$
\frac{\partial C_s^{t_k}}{\partial t} = \frac{\alpha_{t_s} C_{t_k}^{e}}{N}\, C_s - \left(\alpha_{r_k} + \upsilon_s\right) C_s^{t_k},
\qquad
C_{t_k}^{e} = C_{t_k}^{o} - \sum_s C_s^{t_k}
$$

$$
\alpha_{t_s} = \frac{D_s}{\lambda^2},
\qquad
\alpha_{r_k} = \nu_o \exp\!\left(-\frac{E_{t_k}}{kT}\right)
$$

with $J_s$ the diffusive flux, $S_s$ the local source, $\upsilon_s$ the decay frequency of species $s$, $\upsilon_m^{s}$ that of a parent $m$ decaying to $s$, $C_{t_k}^{e}$ and $C_{t_k}^{o}$ the empty and total concentrations of trap type $k$, $E_{t_k}$ its trapping energy, $\alpha_{t_s}$ the trapping and $\alpha_{r_k}$ the release rate coefficients, $N$ the lattice atom density, $\lambda$ the lattice parameter and $\nu_o$ the Debye frequency (about 10¹³ s⁻¹) [ref:longhurst2004]. TMAP7 writes the flux with a Soret (thermal-diffusion) term, $\nabla\cdot J_s = \frac{d}{dx}\left[-D\left(\frac{dC_s}{dx} + C_s\frac{Q^*}{kT^2}\frac{dT}{dx}\right)\right]$; dropping $Q^*$ gives Fick's law [ref:longhurst2004]. The model assumes a number of trap sites that is fixed in time but may vary in space [ref:longhurst2004].

Why this source: PARFUME's fission-product transport is built on code extracted from TMAP4, and trapping models are implemented in PARFUME, but its current calculations use diffusivities that combine all transport processes [ref:miller2018]. No source read applies this system to silver in TRISO SiC with measured trap parameters.

For STRATO this would mean a second unknown per cell in the SiC layer (the trapped silver concentration), solved together with the existing layered diffusion equation; the decay frequency of Ag-110m is the register value $\lambda = 3.211198\times10^{-8}\ \mathrm{s^{-1}}$ (`data/nuclear/decay.csv`), and stable Ag-109 has none.

### Concept 2: trap-limited effective diffusivity (pore traps)

AbdulHameed et al. multiply a trap-free effective diffusivity of polycrystalline 3C-SiC by a correction for reversible trapping at isolated nano-pores, using the sequence trap → short-lived free state → mobile state in the grain-boundary network [ref:abdulhameed2026]:

$$
D'_{\mathrm{eff}} = \alpha D_{\mathrm{eff}} = \alpha_0 \exp\!\left(-\frac{Q_t}{kT}\right) D_{\mathrm{eff}}
$$

$$
D'_{\mathrm{eff}}(T) = \frac{\pi}{\phi}\left(\frac{d_p}{d_g}\right)^{2} D_{0,\mathrm{eff}} \exp\!\left(-\frac{Q_{\mathrm{eff}} + Q_t}{kT}\right)
$$

with $\phi$ the porosity, $d_p$ and $d_g$ the average pore and grain sizes, $Q_t$ the desorption energy and $D_{0,\mathrm{eff}}$, $Q_{\mathrm{eff}}$ the parameters of the trap-free diffusivity [ref:abdulhameed2026]. The multiplicative form follows Shewmon's result that trapping can be represented by a factor on the trap-free diffusivity, and it holds only while pores are sparse and isolated [ref:abdulhameed2026].

What the source reports, and of which kind each value is:

- Trap-free model (molecular dynamics for two special boundary types, literature values for other boundaries and the bulk, bounds averaging): $Q_{\mathrm{eff}} = 0.90$ eV, $D_{0,\mathrm{eff}} = 6.05\times10^{-12}$ m² s⁻¹, which over-predicts silver diffusivity against the experimental data used [ref:abdulhameed2026].
- $Q_t = 1.27$ eV is an average of the desorption barriers from a density-functional study of silver on SiC surfaces [ref:abdulhameed2026]; that study reports 0.85 to 1.68 eV [ref:xiao2012]. It is an atomistic barrier, not a measured trap energy.
- Adding $Q_t$ raises the activation energy to 2.17 eV, inside the 95 % credible interval of the source's Bayesian analysis [ref:abdulhameed2026].
- The mean number of hops between traps, which sets the prefactor, varies over two orders of magnitude across porosities of 0.25 % to 1.50 % and pore radii of 50 to 350 nm (grain radius 500 nm); with mid-range values it is about 57 and $\alpha_0 D_{0,\mathrm{eff}} = 3.45\times10^{-10}$ m² s⁻¹, close to the posterior mode of 3.03 × 10⁻¹⁰ m² s⁻¹ [ref:abdulhameed2026].

The authors call the correction approximate and based on qualitative arguments [ref:abdulhameed2026]. Equations and numbers were read in the arXiv version 1 of the paper; the published version was not compared.

## Required parameters

| Parameter | In a source? | In STRATO's register? |
|---|---|---|
| Trap site density of silver traps in AGR-1 SiC | not found | no |
| Trap (detrapping) energy for silver in SiC | only an atomistic surface-desorption range, 0.85 to 1.68 eV [ref:xiao2012] | no |
| Trapping rate coefficient (or jump distance and mobile diffusivity) | generic form only [ref:longhurst2004] | mobile Ag diffusivity in SiC yes (`data/diffusivity/arrhenius.csv#A3-05`, effective, lumps all processes) |
| Lattice atom density of SiC | not taken from a source here | no |
| Porosity, pore size and grain size of the SiC layer | porosity and pore-size ranges as used in [ref:abdulhameed2026]; grain sizes of AGR-1 and AGR-2 SiC in [ref:gerczak2016] | no |
| Decay constant of Ag-110m | yes | yes (`data/nuclear/decay.csv`, Ag-110m) |

## Supporting literature

[ref:longhurst2004] (coupled trapping system), [ref:miller2018] (trapping in PARFUME), [ref:abdulhameed2026] (pore-trap correction for silver in 3C-SiC), [ref:xiao2012] (silver adsorption and desorption on SiC), [ref:simon2022] (triple junctions as possible traps), [ref:vanrooyen2014a] and [ref:lillo2015] (silver at triple points in AGR-1 SiC), [ref:szlufarska2016] (trapping in implantation damage).

## Missing calculations or experiments

- A measured or computed trap density and trap energy for silver in the AGR-1 SiC microstructure. None was found.
- Atomistic energies of silver at pore surfaces and triple junctions in models that represent the polycrystalline coating (gallery items `ms-ag-pore-surface`, `ms-ag-triple-junction`).
- A derivation of Concept 1 on STRATO's spherical finite-volume mesh with its own conservation and limiting-case checks (zero trap density must return the present equation).
- A test of whether Concept 2 is consistent with Concept 1 in the limit of fast exchange; not derived here.
- Comparison with AGR-1 Ag-110m retention after Phase 2.2.

## Interactions with other regimes

- **Regime A, grain-boundary transport.** The pore-trap correction acts on a grain-boundary effective diffusivity, so it is used together with [Silver grain-boundary transport](ag_grain_boundary_transport.md) [ref:abdulhameed2026].
- **Regime B, irradiation defects.** Neutron displacement damage creates trapping sites [ref:longhurst2004], so trap density could depend on fluence; no source read quantifies this for silver in SiC.
- **Regime B, palladium.** Silver co-located with palladium at grain boundaries and triple junctions [ref:lillo2015] could be a chemical trap or a carrier; see [Palladium-assisted degradation](pd_assisted_degradation.md).
