# Phase 1 derivation (v1.0, as delivered)

These files are the Phase 1 derivation and pseudocode as they were delivered in October 2026. They are kept unchanged as the historical record.

| File | Content |
|---|---|
| `TRISO_Finite_Volume_Derivation.md` | Plain-language derivation: conservation law, spherical diffusion equation, conditions, exact eigenfunction solution (Case A), conservative finite volumes, Backward Euler, inventory and release |
| `TRISO_Finite_Volume_Derivation.tex`, `.pdf` | LaTeX source and PDF of the same derivation |
| `TRISO_Finite_Volume_Pseudocode.md` | Pseudocode of the finite-volume solver and the exact reference series |

Two things to know when reading them:

- **File names.** They name the code files of the original package. In STRATO these are `src/numerical/finite_volume.rs` (Code B), `src/analytical/sturm_liouville.rs` (Code C) and `src/main.rs` with `src/config.rs` and `src/postprocessing/` (Code A). See `docs/development/provenance.md`.
- **Equation labels.** Labels such as `TRISO-GOV-020` and `TRISO-FV-100` come from an earlier working document that is not part of STRATO. The equations themselves are written out in full where they are used.

The full derivation of the v1.0 equations is now in [`../v1.0.0/STRATO_Full_Mathematical_Derivation.md`](../v1.0.0/STRATO_Full_Mathematical_Derivation.md) (detailed learning and derivation document), and the concise implementation reference that connects equations, pseudocode and Rust code is [`../v1.0.0/STRATO_Final_Deliverable.md`](../v1.0.0/STRATO_Final_Deliverable.md). The files in this folder remain the historical record of what was delivered in October 2026.
