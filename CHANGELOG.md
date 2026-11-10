# Changelog

All notable changes to STRATO are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

Work toward v2.0 (Phase 2). The order is mathematics first, then pseudocode, then a review, then code.

- Planned: radioactive decay, fission-product generation, inventory accounting, Arrhenius diffusivity and temperature histories.
- The list of v1.0 to v2.0 changes (design only, no Phase 2 code yet) is kept in [docs/development/v1_to_v2_changes.md](docs/development/v1_to_v2_changes.md).

### Changed
- Licence changed from MIT to AGPL-3.0-only, the licence of the Outram Park project that the planned web demo builds on. Releases v1.0.0 and v1.0.1 remain available under MIT.

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
- Synthetic inputs only, so there is no physical validation.

[Unreleased]: https://github.com/weewzr/STRATO/compare/v1.0.1...HEAD
[1.0.1]: https://github.com/weewzr/STRATO/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/weewzr/STRATO/releases/tag/v1.0.0
