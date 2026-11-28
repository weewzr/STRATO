# STRATO — Spherical TRISO Analysis and Transport Operations

[![CI](https://github.com/weewzr/STRATO/actions/workflows/ci.yml/badge.svg)](https://github.com/weewzr/STRATO/actions/workflows/ci.yml)
[![Website](https://github.com/weewzr/STRATO/actions/workflows/web.yml/badge.svg)](https://github.com/weewzr/STRATO/actions/workflows/web.yml)

**STRATO** is an open-source, verification-first computational nuclear engineering project implemented in Rust (with WebAssembly browser execution) to model fission-product diffusion across intact five-layer tristructural isotropic (TRISO) fuel particles.

- **Public GitHub Repository:** <https://github.com/weewzr/STRATO>
- **Deployed Interactive Website:** <https://weewzr.github.io/STRATO/>
- **Final Restored Research Report (Phase 3):** [`deliverables/report/STRATO_Research_Report_Phase3_Final.pdf`](deliverables/report/STRATO_Research_Report_Phase3_Final.pdf) ([`.docx`](deliverables/report/STRATO_Research_Report_Phase3_Final.docx))
- **Final Editable PowerPoint (Phase 3, 97 slides):** [`deliverables/powerpoint/STRATO_Presentation_Phase3_Final.pptx`](deliverables/powerpoint/STRATO_Presentation_Phase3_Final.pptx)

---

## Central Research Question

> *To what extent does the progressive incorporation of additional transport physics improve the predictive accuracy of numerical fission-product release models in multilayer TRISO fuel particles?*

---

## Scientific Modelling Basis

- **Primary Nuclide:** $\text{Cs-137}$ ($t_{1/2} = 30.08\ \text{y}$, $\lambda = 7.302187\times 10^{-10}\ \text{s}^{-1}$, cumulative $^{235}\text{U}$ thermal fission yield $y = 0.062208$).
- **AGR-1 Reference Generation Rate:** $\dot{F} = 8.699783\times 10^{19}\ \text{m}^{-3}\text{ s}^{-1}$ (Capsule 3 burnup-derived rate), giving $G_{\text{ref}} = y \dot{F} / N_{\text{A}} = 8.986773\times 10^{-6}\ \text{mol m}^{-3}\text{ s}^{-1}$.
- **Diffusivity Correlations:** Published PARFUME / IAEA-TECDOC-978 effective Arrhenius sets from Collin et al. (2015, 2016) — `C16` for post-irradiation heating tests (`1200–1800 °C`) and `C15` for AGR-1 irradiation conditions (`955–1136 °C`), evaluated at mid-step temperatures ($R = 8.3145\ \text{J mol}^{-1}\text{ K}^{-1}$) without unverified tuning.
- **Principal Temperatures:** Six heating-test temperatures (`1200, 1350, 1500, 1600, 1700, 1800 °C`) and three AGR-1 irradiation temperatures (`955, 1028, 1136 °C`, 620.2 EFPD).

### Five-Layer TRISO Geometry (`AGR1-B-M16`)

| Layer | Outer Radius ($\mu\text{m}$) | Outer Radius ($\text{m}$) | Base Grid Intervals |
| --- | ---: | ---: | ---: |
| Kernel (UCO / $\text{UO}_2$ surrogate) | 175.0 | $1.750\times 10^{-4}$ | 160 |
| Porous Carbon Buffer | 278.5 | $2.785\times 10^{-4}$ | 20 |
| Inner Pyrolytic Carbon (IPyC) | 317.9 | $3.179\times 10^{-4}$ | 20 |
| Silicon Carbide (SiC) | 353.2 | $3.532\times 10^{-4}$ | 160 |
| Outer Pyrolytic Carbon (OPyC) | 394.2 | $3.942\times 10^{-4}$ | 20 |

---

## Solution Methods

### Method A — FDM-BTCS (`#DC2626`)
Conservative spherical-shell Backward-Time, Centred-Space (BTCS) finite differences (`src/numerical/`):
- Nodal placement on every material interface ($R_1 \dots R_4$) with shell conductances $K_{i,\text{outer}} = D_{i,\text{outer}} r_i r_{i+1} / \Delta r_{i,\text{outer}}$ and L'Hôpital centre closure $1 + 6\mathrm{Fo}_{\text{kernel}} + \kappa$.
- Tridiagonal $O(N)$ Thomas M-matrix solver guaranteeing non-negativity, minimum row margin $\ge 1.0$, and a-posteriori floating-point mass-balance rounding bounds (`(F46)`).

### Method B — Sturm–Liouville Eigenfunction Series (`#2563EB`)
Exact analytical multilayer eigenseries (`src/analytical/`):
- Layer-by-layer transfer matrices with continuity of $u = r\varphi$ and jump $q_{\text{out}} = q_{\text{in}} + (D_{\text{out}} - D_{\text{in}}) u / R$, isolated by Sturm oscillation counting and bisection.
- Exact analytical extensions for uniform decay ($\alpha_\mu + \lambda$) and piecewise-constant kernel sources at constant temperature, guarded by an explicit convergence mask (`valid_from_h`, `accuracy_floor`) and strict refusal under time-varying temperature (`TIME_VARYING_TEMPERATURE`).

---

## Option B Physics Matrix (`P01–P08`) & Stage A–D Mapping

All eight combinations of `(Baseline diffusion locked, Generation G, Decay λ, Temperature-dependent D(T))` are simulated in [`crates/strato-results/src/bin/study39.rs`](crates/strato-results/src/bin/study39.rs) (`66` audited runs including controlled matched-inventory runs `CTRL_G` and `CTRL_GL`):

| Config | Historical Alias | Gen ($G$) | Decay ($\lambda$) | Temp ($T(t)$) | Physical Role |
| --- | --- | :---: | :---: | :---: | --- |
| **P01** | Stage A | Off | Off | Off | Baseline intact-particle diffusion ($F_{\text{init}} = M_{\text{rel}}/M^0$) |
| **P02** | Stage B | On | Off | Off | Active fission-product generation from $C=0$ ($F_{\text{gen}}$; controlled twin `CTRL_G`) |
| **P03** | Stage C (decay-only) | Off | On | Off | Source-free post-irradiation annealing with radioactive decay ($F_{\text{init}}$) |
| **P04** | Stage D (heating ramp; $G=0$) | Off | Off | On | Source-free `AGR1-ST-C16` / $50\ ^\circ\text{C/h}$ ramp without decay |
| **P05** | Stage C | On | On | Off | Active generation + decay (`CTRL_GL` controlled matched-inventory twin) |
| **P06** | P06 ($+G, T(t)$) | On | Off | On | Active generation + temperature-dependent heating ramp |
| **P07** | Stage D ($+\lambda, T(t)$) | Off | On | On | Source-free heating ramp with radioactive decay |
| **P08** | Stage D (combined) | On | On | On | Full combined transport (generation + decay + temperature-dependent $D(T)$) |

---

## Scientific Graph & Website Inventory

### Principal 39 Graphs (`graphs/study39/`)
All **39 principal graphs** (`A01–A24`, `B01–B06`, `C01–C03`, `D01–D06`) and **10 supporting diagnostics** (`S01–S10`) are generated in Graph B presentation format (PNG 300 dpi, SVG, CSV) and indexed in [`data/study39/study39_manifest.json`](data/study39/study39_manifest.json):
- **Group A (`A01–A24`, 24 graphs):** Method A BTCS Stages A–D across `1200–1800 °C` with experimental overlays and single endpoint $\Delta$ annotations.
- **Group B (`B01–B06`, 6 graphs):** Method B Sturm–Liouville vs equivalent Method A formulations across `1200–1800 °C`.
- **Group C (`C01–C03`, 3 graphs):** `C01` BTCS Numerical Convergence, `C02` Method A vs Method B Agreement, `C03` Experimental Deviation Summary.
- **Group D (`D01–D06`, 6 graphs):** Experimental comparisons at `1200, 1350, 1500, 1700, 1800 °C` and AGR-1 irradiation at `955–1136 °C`.

### Website 81-Entry Verification Inventory (`data/study39/web81_manifest.json`)
- **48 Method A (BTCS)** curves (`P01–P08` × 6 temperatures),
- **24 Method B (Sturm–Liouville)** potentially supported constant-temperature cases (`P01–P03, P05` × 6 temperatures; `9` available/drawn, `15` explicitly withheld by analytical early-time mask),
- **6 Experimental Datasets** (`D01–D06`),
- **3 Numerical Verification Summaries** (`C01–C03`).

Interactive features on **Explore** (<https://weewzr.github.io/STRATO/>):
- **Select a Modelling Stage:** Locked `Baseline diffusion` checkbox + independent `Fission-product generation`, `Radioactive decay`, and `Temperature-dependent diffusion` checkboxes across `P01–P08`, plus `Species-Specific Physics` selector (`None — Future development`).
- **Independent Operation Selector:** `Steady-state`, `Shutdown`, `Startup`, `Control-rod`, and `Refuelling`, displaying authentic Stage 5 Rust BTCS curves alongside steady-state references for `Cs-137` and `Sr-90`.
- **Independent Overlay Checkboxes:** `Method A — BTCS`, `Method B — Sturm–Liouville`, and `Experimental data` (all 7 non-empty subsets + optional controlled matched-inventory toggle), displaying immutable single final-endpoint $\Delta$ annotations from [`data/study39/endpoint_deviations.csv`](data/study39/endpoint_deviations.csv).

---

## Experimental Validation Methodology & Provenance Audit

Model curves are evaluated at exact experimental timestamps $t_i$ via linear interpolation and separated by physical comparability:
1. **Exact / Bounded Time Series (`1700–1800 °C`):** Minato et al. (2000) HRB-22 $\text{UO}_2$ particles — **ACT3 (`1700 °C`, $N=8$, intact)** and **ACT4 (`1800 °C`, $N=11$ including unresolved $0\text{–}10\ \text{h}$ cluster $t=4.6\pm 2.0\ \text{h}$, $1/25$ failed SiC bracket)**. Single final-endpoint deviation $\Delta = F_{\text{model}}(t_{\text{end}}) - F_{\text{exp}}(t_{\text{end}})$ and full-series MAE/RMSE/IAE.
2. **Upper Bounds Only (`1200–1600 °C`):** Bullock (1984) / IAEA-TECDOC-2090 Table 22 (`5%` counting vs `0.01%` collector readings) and `IAEA97-K31-CS137` (`HFR-K3/1` sphere at `1600 °C`), reported as $\Delta_{\text{bound}} = F_{\text{model}}(t_{\text{end}}) - U_{\text{exp}}$.
3. **Non-Comparable Context:** Degraded Bullock curves (`1500 °C`), AGR-1 4-4-2 `HUNN16-442-CS137`, `IAEA97-JAERI-A`, and AGR-1 irradiation totals (`D06`; `unavailable` for time-resolved intact validation).

---

## Key Numerical and Experimental Findings

1. **Numerical Verification (`66` Audited Runs):** BTCS achieves observed convergence orders of **2.014 in space** ($O(\Delta r^2)$) and **1.077 in time** ($O(\Delta t)$). Across all `66` runs, maximum normalized floating-point mass-balance residual $|\varepsilon|/\text{bound}$ is **0.04976** ($\le 1.0$), minimum row margin is **1.0**, and Method A agrees with Method B within **$\le 1.51\times 10^{-3}$ pp**.
2. **Effect of Progressive Physics (`G=0` Anneals):**
   - At **1700 °C (`ACT3`, $N=8$)**: `P01` / Stage A (`RMSE = 0.240 pp, final Δ = -0.120 pp`) $\to$ `P03` decay-only (`0.241 pp, -0.123 pp`) $\to$ **`P04` heating ramp (`RMSE = 0.226 pp, final Δ = -0.035 pp`, +6.08% RMSE improvement, +7.97% MAE improvement)**.
   - At **1800 °C (`ACT4`, $N=11$)**: `P01` (`RMSE = 7.519 pp, final Δ = +12.847 pp`) $\to$ `P03` (`7.516 pp, +12.838 pp`) $\to$ **`P04` heating ramp (`7.706 pp, +13.132 pp`)**.
3. **Controlled Matched-Inventory Tests (`CTRL_G`, `CTRL_GL`):** Matching $C_{\text{init}}^{\text{ctrl}} = G_{\text{ref}} \times 620.2\ \text{d}$ (`1.817e-1` vs `5.456%` at `1700 °C`, $\Delta = -0.185\ \text{pp}$; `+12.624 pp` at `1800 °C`) confirms that `P02`/`P05` lower raw percentages come from the larger freshly-generated denominator ($F_{\text{gen}}$ from $C=0$) rather than improved particle physics.

---

## Repository Structure

```text
STRATO/
├── src/                        # Core Rust library (Method A BTCS & Method B Sturm–Liouville)
├── crates/
│   ├── strato-data/            # Typed parameter, geometry, Arrhenius, and experimental registers
│   ├── strato-scenarios/       # Scenario builders, heating histories, shared Method B driver
│   ├── strato-compare/         # Convergence studies (spatial_convergence, temporal_convergence)
│   ├── strato-results/         # Baseline pipeline (`strato-results`) + Phase 3 `study39` binary
│   ├── strato-wasm/            # WebAssembly solver wrapper
│   ├── strato-docsite/         # Mathematical documentation builder and link checker
│   └── strato-webtools/        # Website catalogue, Wasm build, and byte-identity checkers
├── data/
│   ├── study39/                # Canonical P01–P08 curves, endpoint_deviations.csv, web81_manifest.json
│   ├── results/                # Verified baseline pipeline outputs (strato-results --check)
│   └── experimental/           # Audited experimental release datasets & curve_assessment.csv
├── graphs/study39/             # All 39 principal Graph B figures + 10 supporting diagnostics
├── deliverables/
│   ├── report/                 # Restored Phase 3 Research Report (DOCX & 92-page PDF)
│   └── powerpoint/             # Finalised Editable Phase 3 Presentation (97 slides, PPTX)
├── web/                        # React + TypeScript + Plotly + Wasm website (Option B UI)
├── scripts/                    # Canonical analysis, plotting, report, deck, and web export scripts
├── docs/                       # Mathematical proofs (Documents 1–6) & ANTIGRAVITY_PROGRESS.md
└── archive/                    # Provenance index + archived Phase 2 intermediate deliverables
```

---

## Reproducibility Instructions

```powershell
# 1. Run core Rust solver & regression tests
cargo test --release
cargo run --release --example btcs_regression -- --commit (git rev-parse HEAD)

# 2. Run canonical P01–P08 + controlled 66-run Rust simulations
cargo run --release --manifest-path crates/Cargo.toml --bin study39 -- --repo . --commit (git rev-parse HEAD)

# 3. Compute unified endpoint deviations, 81-entry web manifest, Graph B figures, report, deck & web JSON
python scripts/study39_deviations.py
python scripts/generate_study39_graphs.py
python scripts/restore_report_phase3.py
python scripts/build_deck_phase3.py
python scripts/export_web_study39.py

# 4. Verify workspace crates, WebAssembly build, docsite links & web content guards
cargo test --manifest-path crates/Cargo.toml --release
cargo run --locked --release --manifest-path crates/Cargo.toml -p strato-webtools -- build-wasm
Set-Location web; npm ci; npm run lint; npm run typecheck; npm test -- --run; npm run build
Set-Location ..; cargo run --locked --release --manifest-path crates/Cargo.toml -p strato-docsite -- build --repo . --out web/dist --commit (git rev-parse HEAD)
cargo run --locked --release --manifest-path crates/Cargo.toml -p strato-docsite -- check-links web/dist --base /STRATO/
Set-Location web; npm run check:content
```

---

## References and Acknowledgements

Full academic references (Bullock 1984; Collin et al. 2015, 2016; Demkowicz et al. 2015, 2016; Gelbard 2003; Gerczak et al. 2016; Hales et al. 2021; Hunn et al. 2014–2019; IAEA 1997, 2025; Miller et al. 2018; Minato et al. 2000; Morris et al. 2016; Parsons et al. 2024; Schappel et al. 2020; Simon et al. 2022; Stempien et al. 2023) are documented in [`deliverables/report/STRATO_Research_Report_Phase3_Final.pdf`](deliverables/report/STRATO_Research_Report_Phase3_Final.pdf) and in Scientific Documentation on the live website.

## Licence

Core solver source code (`src/`), Rust crates (`crates/`), and web application (`web/`) are licensed under **AGPL-3.0-only**.
