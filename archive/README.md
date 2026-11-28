# STRATO Archival Index

This directory preserves superseded intermediate deliverables and historical snapshots kept for provenance without interfering with active Phase 3 execution.

## Authoritative Phase 3 Deliverables
- **Canonical simulation data, 66-run audit & 81-entry web manifest:** [`../data/study39/`](../data/study39/)
- **Principal 39 Graph B figures + 10 supporting diagnostics (PNG 300 dpi, SVG, CSV):** [`../graphs/study39/`](../graphs/study39/)
- **Final Restored Research Report (DOCX & 92-page PDF):** [`../deliverables/report/STRATO_Research_Report_Phase3_Final.pdf`](../deliverables/report/STRATO_Research_Report_Phase3_Final.pdf) ([`.docx`](../deliverables/report/STRATO_Research_Report_Phase3_Final.docx))
- **Final Editable PowerPoint Presentation (97 slides, PPTX):** [`../deliverables/powerpoint/STRATO_Presentation_Phase3_Final.pptx`](../deliverables/powerpoint/STRATO_Presentation_Phase3_Final.pptx)
- **Website (Option B P01–P08 + 3 overlays + s5 scenarios) & WebAssembly Solver:** [`../web/`](../web/) and [`../crates/strato-wasm/`](../crates/strato-wasm/)
- **Baseline Results Pipeline:** [`../data/results/`](../data/results/) and [`../crates/strato-results/`](../crates/strato-results/)

## Archived Items
- **Superseded Phase 2 Report & PowerPoint (`archive/phase2/`):** [`phase2/report/`](phase2/report/) (`STRATO_Research_Report_Phase2_Final.{docx,pdf}`) and [`phase2/powerpoint/`](phase2/powerpoint/) (`STRATO_Presentation_Phase2_Final.pptx`) were archived when Phase 3 restored the original pre-Phase-2 full research report formatting/derivations and reconstructed the 97-slide Graph B PowerPoint deck.
- **External Read-Only Phase 1 Archive (`STRATO_Archive`):** Preserves the original pre-Phase-2 research report, 105-slide baseline PowerPoint, 212 historical graphs, parameter registers, and historical agent handovers outside the active repository tree.
- **Reference Documentation Folders:** `docs/development/` and `prompt/` remain in their repository paths where verified by `strato-webtools`, `strato-docsite`, and Vitest documentation link checks.
