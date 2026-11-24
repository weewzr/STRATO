# Materials Studio gallery: planned atomistic calculations

This gallery is meant to show genuine BIOVIA Materials Studio screenshots of atomistic structures relevant to silver, palladium, caesium, strontium, europium and the fission gases in the AGR-1 SiC coating. **No such calculation has been performed and no image exists yet.** Every item below is a placeholder that describes a planned calculation; each is planned for a future phase of the project. No image is generated, drawn or imitated in its place.

Rules that apply to every item when it is done:

- Only genuine Materials Studio screenshots supplied with their method and conditions are shown as screenshots; an interactive view of an exported structure is labelled as a reconstruction and kept apart from screenshots.
- An ideal 3C-SiC crystal or a single bicrystal does not represent the polycrystalline, heavily faulted AGR-1 coating [ref:kirchhofer2013].
- A migration barrier from an atomistic calculation is not an effective diffusivity of the coating and is not converted into one on the page.

The authoritative list, with the same wording, is `data/research/gallery.csv`; the status column there is the one the interface reads.

## Cubic SiC reference cell

- **Item:** `ms-sic-3c-bulk` (Pending placeholder; no image)
- **Caption:** Planned reference cell of cubic (3C) SiC for later defect and boundary models. The AGR-1 SiC is cubic but polycrystalline and heavily faulted; an ideal crystal does not represent the coating. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of the repeating atomic unit of perfect cubic silicon carbide, used only as a starting point; the real coating is made of many small, faulted grains. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC, ideal single crystal (reference only)
- **Species:** none
- **Intended method:** Planned: crystal building and periodic density-functional relaxation in BIOVIA Materials Studio (CASTEP module).
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [ag_grain_boundary_transport](ag_grain_boundary_transport.md)
- **Sources for the planned subject:** [ref:kirchhofer2013] [ref:shrader2011]

## Σ3 grain boundary in 3C-SiC

- **Item:** `ms-sic-sigma3-gb` (Pending placeholder; no image)
- **Caption:** Planned bicrystal of a Σ3 twin boundary, a boundary type present in high fraction in the AGR-1 SiC. Σ3 boundaries cannot give a connected path through the layer and bound boundary diffusion from below. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of two SiC grains meeting at a highly ordered type of boundary that is frequent in the AGR-1 coating. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC Σ3 bicrystal
- **Species:** none
- **Intended method:** Planned: bicrystal built in Materials Studio, pre-relaxed with an empirical potential (Forcite module), then density-functional relaxation (CASTEP module).
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [ag_grain_boundary_transport](ag_grain_boundary_transport.md)
- **Sources for the planned subject:** [ref:khalil2011] [ref:kirchhofer2013]

## Σ5 grain boundary in 3C-SiC

- **Item:** `ms-sic-sigma5-gb` (Pending placeholder; no image)
- **Caption:** Planned Σ5 (210)/[001] boundary, for which faster Ag diffusion than in random high-angle boundaries has been reported, and where Ag and Pd diffusion have been simulated atomistically. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of a less ordered boundary between two SiC grains, a candidate fast path for silver. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC Σ5 (210)/[001] bicrystal
- **Species:** none
- **Intended method:** Planned: as for the Σ3 boundary (Forcite pre-relaxation, CASTEP relaxation).
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [ag_grain_boundary_transport](ag_grain_boundary_transport.md)
- **Sources for the planned subject:** [ref:aagesen2022] [ref:rabone2015]

## High-energy (random) grain boundary

- **Item:** `ms-sic-random-gb` (Pending placeholder; no image)
- **Caption:** Planned model of a high-energy boundary region, represented in the literature as amorphous SiC, which is expected to provide percolating fast paths. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of a disordered boundary region between SiC grains, the kind expected to connect across the coating. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC with an amorphous boundary layer
- **Species:** none
- **Intended method:** Planned: melt-quench molecular dynamics with an empirical potential (Forcite module) to build the disordered region, then density-functional relaxation of a smaller cell.
- **Intended conditions:** Planned: quench and relaxation; finite-temperature runs within the AGR-1 range of time-average, volume-average compact temperatures (955 to 1136 °C, [ref:demkowicz2016]).
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [ag_grain_boundary_transport](ag_grain_boundary_transport.md)
- **Sources for the planned subject:** [ref:ko2016] [ref:simon2022] [ref:khalil2011]

## Vacancies and interstitials in 3C-SiC

- **Item:** `ms-sic-intrinsic-defects` (Pending placeholder; no image)
- **Caption:** Planned set of intrinsic point defects (carbon and silicon vacancies, interstitials, antisites), the defects that irradiation multiplies and that bind silver. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of missing and extra atoms in the SiC crystal, the kind of damage that radiation creates. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC supercell with point defects
- **Species:** none
- **Intended method:** Planned: density-functional defect formation energies in a supercell (CASTEP module).
- **Intended conditions:** Planned: static relaxation at 0 K; charge states to be chosen from the literature.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [ag_grain_boundary_transport](ag_grain_boundary_transport.md), [ag_trapping_detrapping](ag_trapping_detrapping.md)
- **Sources for the planned subject:** [ref:shrader2011] [ref:aagesen2022]

## Silver impurity configurations in bulk 3C-SiC

- **Item:** `ms-ag-bulk-config` (Pending placeholder; no image)
- **Caption:** Planned silver on silicon and carbon sites, as an interstitial, and bound to vacancies; the literature finds Ag on a silicon site bound to a carbon vacancy most stable in the bulk. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of single silver atoms sitting in different places inside a perfect SiC crystal. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC supercell
- **Species:** Ag
- **Intended method:** Planned: density-functional formation and binding energies (CASTEP module).
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [ag_grain_boundary_transport](ag_grain_boundary_transport.md)
- **Sources for the planned subject:** [ref:shrader2011]

## Silver segregation to a grain boundary

- **Item:** `ms-ag-gb-segregation` (Pending placeholder; no image)
- **Caption:** Planned silver sites in a Σ3 boundary compared with the bulk; the literature reports formation energies 2 to 4 eV lower in the boundary. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of silver atoms preferring to sit in the boundary between two grains rather than inside a grain. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC Σ3 bicrystal
- **Species:** Ag
- **Intended method:** Planned: density-functional energies of Ag at boundary and bulk sites (CASTEP module).
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [ag_grain_boundary_transport](ag_grain_boundary_transport.md), [ag_trapping_detrapping](ag_trapping_detrapping.md)
- **Sources for the planned subject:** [ref:khalil2011]

## Silver migration pathway along a grain boundary

- **Item:** `ms-ag-migration-path` (Pending placeholder; no image)
- **Caption:** Planned minimum-energy path for a silver hop along a boundary. Any barrier from it is an atomistic migration barrier and is not converted on the page into an effective diffusivity of the coating. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of the route a silver atom takes when it jumps from one site to the next along a boundary, with the energy hill it has to climb. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC Σ3 or Σ5 bicrystal
- **Species:** Ag
- **Intended method:** Planned: transition-state search (nudged elastic band) on density-functional energies (CASTEP module).
- **Intended conditions:** Planned: 0 K energy path.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [ag_grain_boundary_transport](ag_grain_boundary_transport.md)
- **Sources for the planned subject:** [ref:khalil2011] [ref:rabone2015]

## Silver at a pore surface (trapping site)

- **Item:** `ms-ag-pore-surface` (Pending placeholder; no image)
- **Caption:** Planned silver adsorption and desorption at a SiC surface representing a nano-pore wall; desorption-limited pores are proposed as reversible traps. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of a silver atom stuck to the wall of a tiny pore in the coating, a place where it could be held for a while. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC surface slab
- **Species:** Ag
- **Intended method:** Planned: density-functional adsorption energies and desorption path (CASTEP module).
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [ag_trapping_detrapping](ag_trapping_detrapping.md)
- **Sources for the planned subject:** [ref:xiao2012] [ref:abdulhameed2026]

## Silver at a triple junction

- **Item:** `ms-ag-triple-junction` (Pending placeholder; no image)
- **Caption:** Planned silver at the junction of three grains, where silver precipitates were observed in irradiated AGR-1 SiC and which may act as traps. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of silver at a point where three grains of the coating meet. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC tricrystal
- **Species:** Ag
- **Intended method:** Planned: empirical-potential construction (Forcite module) and density-functional energies of Ag sites in a reduced cell (CASTEP module).
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [ag_trapping_detrapping](ag_trapping_detrapping.md)
- **Sources for the planned subject:** [ref:simon2022] [ref:vanrooyen2014a] [ref:lillo2015]

## Palladium interaction with a SiC surface

- **Item:** `ms-pd-sic-surface` (Pending placeholder; no image)
- **Caption:** Planned palladium atoms on a SiC surface; ab initio molecular dynamics in the literature shows palladium disintegrating the surface and removing carbon. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of palladium atoms attacking the surface of silicon carbide. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC surface slab
- **Species:** Pd
- **Intended method:** Planned: ab initio molecular dynamics (CASTEP module).
- **Intended conditions:** Planned: finite temperature within the AGR-1 range of time-average, volume-average compact temperatures (955 to 1136 °C, [ref:demkowicz2016]); higher temperatures only if justified by a source.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [pd_assisted_degradation](pd_assisted_degradation.md)
- **Sources for the planned subject:** [ref:aagesen2022]

## Palladium at Σ3 and Σ5 grain boundaries

- **Item:** `ms-pd-gb` (Pending placeholder; no image)
- **Caption:** Planned palladium sites and paths at Σ3 and Σ5 boundaries, where atomistic palladium diffusion far faster than observed nodule penetration has been reported. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of palladium atoms in the boundaries between SiC grains. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC Σ3 and Σ5 bicrystals
- **Species:** Pd
- **Intended method:** Planned: density-functional energies and migration paths (CASTEP module).
- **Intended conditions:** Planned: 0 K energies and paths.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [pd_assisted_degradation](pd_assisted_degradation.md)
- **Sources for the planned subject:** [ref:rabone2015]

## Palladium and silver at the same boundary site

- **Item:** `ms-pd-ag-gb` (Pending placeholder; no image)
- **Caption:** Planned co-location of palladium and silver at a boundary or triple junction, as observed in some precipitates of irradiated AGR-1 SiC. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of a silver atom and a palladium atom side by side in a grain boundary, to test whether palladium helps silver move. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC bicrystal
- **Species:** Ag;Pd
- **Intended method:** Planned: density-functional binding energies of Ag-Pd pairs at boundary sites (CASTEP module).
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [pd_assisted_degradation](pd_assisted_degradation.md)
- **Sources for the planned subject:** [ref:lillo2015] [ref:abdulhameed2026]

## Palladium silicide at a SiC grain boundary

- **Item:** `ms-pd-silicide` (Pending placeholder; no image)
- **Caption:** Planned palladium silicide in contact with SiC, the reaction product reported for palladium attack; precipitates of Pd, U and Si were found in AGR-1 SiC. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of the compound that forms when palladium reacts with silicon carbide. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** Palladium silicide / 3C-SiC interface
- **Species:** Pd
- **Intended method:** Planned: interface construction and density-functional relaxation (CASTEP module); silicide phase to be taken from a cited source.
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [pd_assisted_degradation](pd_assisted_degradation.md)
- **Sources for the planned subject:** [ref:simon2022] [ref:demkowicz2016]

## Caesium in bulk 3C-SiC and at a grain boundary

- **Item:** `ms-cs-sic-gb` (Pending placeholder; no image)
- **Caption:** Planned caesium sites in the bulk and at a grain boundary of 3C-SiC. Caesium is effectively retained by intact SiC; published atomistic boundary diffusion coefficients for caesium are far slower than experiment, so any result would be compared with that disagreement, not converted into an effective diffusivity. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of single caesium atoms inside silicon carbide and in the boundary between two grains, the layer that holds caesium back. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC supercell and bicrystal
- **Species:** Cs
- **Intended method:** Planned: density-functional formation and binding energies (CASTEP module).
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [cs_retention_barrier_integrity](cs_retention_barrier_integrity.md)
- **Sources for the planned subject:** [ref:rabone2015] [ref:demkowicz2016] [ref:kirchhofer2013]

## Strontium and europium in 3C-SiC

- **Item:** `ms-sr-eu-sic` (Pending placeholder; no image)
- **Caption:** Planned strontium and europium configurations in 3C-SiC and at a grain boundary. Both elements passed intact AGR-1 coatings in small amounts and were held in the outer pyrocarbon and matrix. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of strontium and europium atoms inside silicon carbide, two elements that leak slowly through intact coatings. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC supercell and bicrystal
- **Species:** Sr;Eu
- **Intended method:** Planned: density-functional formation energies of substitutional and interstitial sites (CASTEP module).
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [additional_species_downstream_transport](additional_species_downstream_transport.md)
- **Sources for the planned subject:** [ref:demkowicz2016] [ref:morris2016] [ref:kirchhofer2013]

## Krypton, xenon and iodine at vacancies in 3C-SiC

- **Item:** `ms-noble-gas-sic` (Pending placeholder; no image)
- **Caption:** Planned krypton, xenon and iodine atoms at vacancy sites and small cavities of 3C-SiC. An intact SiC layer keeps the particle gas-tight; the calculation would examine where these atoms sit, not predict a release. This calculation has not been performed. It is planned for a future phase of the project.
- **Plain-language explanation:** A planned picture of large gas atoms trapped in gaps of the silicon carbide crystal, the layer that keeps fission gases inside the particle. This calculation has not been performed. It is planned for a future phase of the project.
- **Material:** 3C-SiC supercell with vacancies
- **Species:** Kr;Xe;I
- **Intended method:** Planned: density-functional incorporation energies at vacancy sites (CASTEP module).
- **Intended conditions:** Planned: static relaxation at 0 K.
- **Provenance:** none; nothing has been produced.
- **Status:** PENDING
- **Related catalogue entries:** [additional_species_downstream_transport](additional_species_downstream_transport.md)
- **Sources for the planned subject:** [ref:iaea1997] [ref:morris2016] [ref:kirchhofer2013]
