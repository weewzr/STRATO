# Changelog

All notable changes to STRATO are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

Work toward v2.0 (Phase 2). The order is mathematics first, then pseudocode, then a review, then code.

- Planned: radioactive decay, fission-product generation, inventory accounting, Arrhenius diffusivity and temperature histories.
- A full list of v1.0 to v2.0 changes will be kept in `docs/development/v1_to_v2_changes.md`.

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

[Unreleased]: https://github.com/weewzr/STRATO/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/weewzr/STRATO/releases/tag/v1.0.0
