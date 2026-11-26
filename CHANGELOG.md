# Changelog

All notable changes to STRATO are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

Work toward v2.0 (Phase 2). The order is mathematics first, then pseudocode, then a review, then code.

- Planned: radioactive decay, fission-product generation, inventory accounting, Arrhenius diffusivity and temperature histories.
- The list of v1.0 to v2.0 changes (design only, no Phase 2 code yet) is kept in [docs/development/v1_to_v2_changes.md](docs/development/v1_to_v2_changes.md).

### Added
- Experimental comparison of the BTCS solver with the AGR-1 release register: the tool crate `crates/strato-compare` (status rules, BTCS runs at the test temperature, metrics, CSV and JSON files with provenance headers in `data/comparison/`, static SVG figures A to K, and the report `docs/validation/experimental_comparison.md`). It is a comparison of release behaviour, not an experimental validation; decay and generation are not yet in the solver.
- Property functions and closed-form references for the Phase G mechanisms in `src/physics/` (Arrhenius diffusivity, decay constant, volumetric fission source, well-mixed and homogeneous-sphere references), derived in `docs/mathematics/STRATO_Property_Functions.md`. They are not coupled to any solver, and the document is not yet published on the site.

### Changed
- Licence changed from MIT to AGPL-3.0-only, the licence of the Outram Park project that the planned web demo builds on. Releases v1.0.0 and v1.0.1 remain available under MIT.
- GitHub Pages is built and deployed by `.github/workflows/web.yml` instead of the Jekyll render of the repository. The derivation and register documents stay at their earlier addresses, now with rendered mathematics; the three links of the earlier home page that returned 404 (`.github/workflows/ci.yml`, `docs/mathematics/v1.0.0/`, `tests/golden/v1.0/`) are replaced by a GitHub link and two list pages. `prompt/`, the README, this changelog and `docs/development/` are not published on the site.

### Added
- `prompt/`: the instruction package for building the interactive STRATO web demo, including cross-project engineering references from NUS-H2.
- Interactive web demonstration, live at https://weewzr.github.io/STRATO/ (`web/`, `crates/strato-wasm/`, `.github/workflows/web.yml`): the governing equation with a staged builder at the top and the live simulation under it, computed in the browser by the verified v1.0 solver compiled to WebAssembly; particle cross-section and 3D cutaway, radial profile, release curves, inventory balance and numerical diagnostics; comparison, verification and data sections with downloads; the research catalogue (three transport regimes, nine species, Materials Studio gallery placeholders, brick roadmap) with the five-level research status. Five-layer diffusion with constant diffusivity is VERIFIED on the synthetic case; generation, decay and temperature-dependent diffusivity are RESEARCHED and their switches are disabled; nothing is VALIDATED.
- `crates/strato-docsite`, the first member of the tools workspace `crates/Cargo.toml`: renders the derivation and register documents (`docs/mathematics/`, `data/reference/`) into the published site with the mathematics typeset by the KaTeX copy the site ships, and checks every link of the built site.

## [1.0.1] - 2026-11-06

Documentation release. There is no change to the code or to any numerical result: the Rust sources, tests and golden outputs are identical to v1.0.0.

### Added
- `docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md` and its PDF: the detailed learning and derivation document for the v1.0.0 model, from the physical foundation to the analytical solution (Code C), the finite-volume scheme (Code B), the numerical properties and the link to the implementation.
- `docs/mathematics/v1.0.0/STRATO_Final_Deliverable.md` and its PDF: the concise implementation reference that puts the final equation, the pseudocode and the Rust code (with file and line references) side by side.
- Four figures in `docs/mathematics/v1.0.0/figures/` and the scripts that draw them in `scripts/figures/`.
- Derivation check scripts `scripts/derivation_checks/v1_0_0_*.py`, which verify the equations, the code inventory and the numerical properties quoted in the two documents.
- A documentation PDF workflow (`.github/workflows/docs-pdf.yml`) and the pandoc filters and settings in `docs/pandoc/` that build the PDFs with XeLaTeX and render every page for inspection.

### Changed
- `README.md` links both documents and says what each is for. The Phase 1 documents stay linked as the historical record, and `docs/mathematics/phase1/README.md` points to the new documents.

## [1.0.0] - 2026-11-06

Phase 1 baseline. It was developed from June to October 2026 in earlier repositories and established here as a clean project. See `docs/development/provenance.md` for the source commit and file mapping.

### Added
- Conservative spherical finite-volume solver (Code B): exact shell volumes, series-resistance interface conductances, Backward Euler, and subtraction-free tridiagonal elimination.
- Multilayer Sturm-Liouville eigenfunction series (Code C): Sturm-count eigenvalue isolation, release-fraction series, and a reported accuracy floor and validity time.
- Command-line program (Code A): TOML input with placeholder checks, CSV and SVG outputs, a finite-volume vs series comparison, and a spatial and temporal refinement study.
- Synthetic five-layer verification input and a literature input template.
- Verification example (`examples/verify.rs`) that asserts every claim it prints.
- Cross-method integration tests and a Phase 1 regression test against the delivered v1.0 outputs (`tests/golden/v1.0/`).
- Phase 1 derivation and pseudocode as delivered (`docs/mathematics/phase1/`).
- Continuous integration: format, clippy, tests, documentation, release build, verification run, dependency audit.

### Known limitations
- Constant temperature and constant diffusivity in each layer. No decay and no generation.
- Perfect interface contact, perfect-sink outer surface, kernel-only initial inventory.
- The bundled inputs are synthetic, so the public release contains numerical verification only. An internal experimental comparison conducted at SNRSI is not yet public.

[Unreleased]: https://github.com/weewzr/STRATO/compare/v1.0.1...HEAD
[1.0.1]: https://github.com/weewzr/STRATO/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/weewzr/STRATO/releases/tag/v1.0.0
