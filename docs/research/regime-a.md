# Regime A: intact-barrier transport

The STRATO research catalogue sorts proposed physics into three transport regimes. They are transport regimes, not mutually exclusive mechanisms: one particle can show several at once, and a mechanism can move a particle from one regime to another. Regime A covers transport through coating layers that are structurally intact, with the barrier properties the layers had when made, changed only by the diffusing species themselves.

STRATO's present model, five-layer diffusion with a constant diffusivity per layer, is the simplest Regime A description: each layer is treated as a homogeneous medium with one effective diffusivity. The mechanisms below are what such an effective diffusivity hides.

Note on names: Harrison's kinetic types A, B and C for grain-boundary diffusion, used in some sources [ref:szlufarska2016], are a different classification from STRATO's regimes A, B and C.

## Mechanisms to investigate

| Mechanism | What it means | Catalogue entry |
|---|---|---|
| Bulk lattice diffusion | movement through the crystal interior of a layer | covered in the species entries; for silver see [species-ag](species-ag.md) |
| Grain-boundary diffusion | faster movement along boundaries between grains of polycrystalline SiC | [Silver grain-boundary transport](ag_grain_boundary_transport.md) (brick 5) |
| Segregation | enrichment of a species at boundaries because it is more stable there | discussed in the grain-boundary entry |
| Trapping and detrapping | capture at sites where the atom is more strongly bound, and thermal release | [Silver trapping and detrapping](ag_trapping_detrapping.md) (brick 4) |
| Species-specific retention | different species are retained to very different degrees by the same intact layers | species entries; caesium retention is brick 7 (not written in this entry) |

## Evidence by mechanism (silver and palladium)

- **Bulk lattice diffusion.** For silver in ideal 3C-SiC every defect state has a very high activation energy, about 7.9 eV for the fastest (the interstitial), so bulk transport is very slow [ref:shrader2011]. This is an atomistic result for a perfect crystal and does not describe the polycrystalline coating.
- **Grain-boundary diffusion and segregation.** Silver defects form 2 to 4 eV more easily in a Σ3 boundary than in the bulk, and Σ3 diffusion is predicted to be far faster than bulk diffusion [ref:khalil2011]. High-energy boundaries modelled as amorphous SiC give still faster diffusion [ref:ko2016]. A mesoscale model turns boundary and bulk diffusivities into an effective diffusivity that depends on temperature and grain size [ref:simon2022]. Ion-implantation couples suggest boundary diffusion in undamaged SiC does not describe the release from TRISO fuel [ref:gerczak2015]: the evidence conflicts.
- **Trapping and detrapping.** Reversible trapping at isolated nano-pores has been proposed to reconcile a boundary-diffusion model with measured silver diffusivities [ref:abdulhameed2026]; triple junctions may also trap silver [ref:simon2022]. The general coupled trapping system is documented for INL's TMAP7 [ref:longhurst2004]; PARFUME implements trapping but its calculations use effective diffusivities [ref:miller2018].
- **Species-specific retention.** In AGR-1, Ag-110m release reached 1.2 % to 38 % per capsule, while caesium release stayed below 3 × 10⁻⁶ where all particles kept intact SiC; europium and strontium passed intact coatings in small amounts and were mostly retained in the OPyC and compact matrix [ref:demkowicz2016]. Palladium passed intact SiC [ref:demkowicz2016].

## Interactions with other regimes

- Irradiation changes Regime A quantities: carbon-ion irradiation greatly enhanced silver diffusion in SiC couples [ref:leng2016], and neutron damage creates trapping sites [ref:longhurst2004] (Regime B).
- Palladium at grain boundaries may change silver boundary transport (Regime B, [entry](pd_assisted_degradation.md)).
- Vapour or surface transport through nanocracks and pores is proposed as an alternative to boundary diffusion [ref:xiao2012] (Regime C when the pathways connect).

## What STRATO would need

Regime A extensions keep the layered diffusion equation and change what goes into it: a temperature- and microstructure-dependent diffusivity for the SiC layer, or a second, trapped, concentration per cell. Each needs registered parameters with sources, a derivation, and verification before any live use. Their research statuses are held only in `data/research/mechanisms.csv`.
