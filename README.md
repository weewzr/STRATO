# STRATO — Spherical TRISO Analysis and Transport Operations

[![CI](https://github.com/weewzr/STRATO/actions/workflows/ci.yml/badge.svg)](https://github.com/weewzr/STRATO/actions/workflows/ci.yml)
[![Website](https://github.com/weewzr/STRATO/actions/workflows/web.yml/badge.svg)](https://github.com/weewzr/STRATO/actions/workflows/web.yml)

**STRATO** is an open-source, verification-first computational nuclear engineering project implemented in Rust (with WebAssembly browser execution) to model fission-product diffusion across intact five-layer tristructural isotropic (TRISO) fuel particles.

- **Public GitHub Repository:** <https://github.com/weewzr/STRATO>
- **Deployed Interactive Website:** <https://weewzr.github.io/STRATO/>
- **Final Research Report:** [`deliverables/report/STRATO_Research_Report_Phase2_Final.pdf`](deliverables/report/STRATO_Research_Report_Phase2_Final.pdf) ([`.docx`](deliverables/report/STRATO_Research_Report_Phase2_Final.docx))
- **Final Editable PowerPoint:** [`deliverables/powerpoint/STRATO_Presentation_Phase2_Final.pptx`](deliverables/powerpoint/STRATO_Presentation_Phase2_Final.pptx)

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

## Stage A–D Modelling Progression

| Stage | Governing Formulation | Physical Interpretation |
| --- | --- | --- |
| **Stage A** | $\partial_t C = r^{-2}\partial_r(r^2 D(r)\partial_r C)$, $G=0, \lambda=0$ | Baseline intact-particle diffusion ($F_{\text{init}} = M_{\text{rel}}/M^0$) |
| **Stage B** | $+ G_{\text{kernel}}(t)\chi_{\text{kernel}}(r)$, $\lambda=0$, fresh start $C(r,0)=0$ | Active fission-product generation ($F_{\text{gen}} = M_{\text{rel}}/M_{\text{gen}}$) |
| **Stage C** | $+ G(r,t) - \lambda C$ | Adds radioactive decay ($\kappa = \lambda\Delta t$); also run with $G=0$ decay bracket |
| **Stage D** | $D(r, T(t))$ at mid-step $t_{\text{mid}} = (j + 0.5)\Delta t$ | Full `AGR1-ST-C16` / 50 °C/h heating-ramp trajectories + combined transport |

---

## Scientific Graph Inventory

All **39 principal graphs** and **10 supporting diagnostics** are stored in PNG (300 dpi), SVG, and CSV under [`graphs/study39/`](graphs/study39/) and indexed in [`data/study39/study39_manifest.json`](data/study39/study39_manifest.json):
- **Group A (`A01–A24`, 24 graphs):** Method A BTCS Stages A, B, C, and D across 1200, 1350, 1500, 1600, 1700, 1800 °C.
- **Group B (`B01–B06`, 6 graphs):** Method B Sturm–Liouville vs equivalent Method A formulations across 1200–1800 °C.
- **Group C (`C01–C03`, 3 graphs):** `C01` BTCS Numerical Convergence Study, `C02` Method A vs Method B Agreement, `C03` STRATO Experimental Deviation Summary.
- **Group D (`D01–D06`, 6 graphs):** Experimental release comparisons at `1200, 1350, 1500, 1700, 1800 °C` and AGR-1 irradiation exploration at `955–1136 °C`.
- **Supporting (`S01–S10`, 10 graphs):** Multi-temperature overviews, 1600 °C context (`HFR-K3/1` & `JAERI-A`), radial profiles $C(r, t)$, `C16` vs `C15` laws, heating ramps, scenarios `S1–S5`, and pointwise residuals.

*(The baseline 212-graph collection, original 105-slide presentation, and 63-page Phase 1 report remain preserved read-only in the external research archive.)*

---

## Experimental Validation Methodology & Provenance Audit

Model curves are evaluated at exact experimental timestamps $t_i$ via linear interpolation (never nearest-sample matching) and strictly separated by physical comparability:
1. **Exact / Bounded Time Series (`1700–1800 °C`):** Minato et al. (2000) HRB-22 $\text{UO}_2$ particles — **ACT3 (`1700 °C`, $N=8$, intact)** and **ACT4 (`1800 °C`, $N=11$ including unresolved $0\text{–}10\ \text{h}$ cluster $t=4.6\pm 2.0\ \text{h}$, $1/25$ failed SiC bracket)** — NOT AGR-1 fuel. Metrics: $\Delta_s(t_i) = 100(F_s - F_{\text{exp}})$, $|\Delta_s|$, MAE, RMSE, max $|\Delta|$, trapezoidal IAE, and Stage A→D change (%).
2. **Upper Bounds Only (`1200–1600 °C`):** Bullock (1984) / IAEA-TECDOC-2090 Table 22 (`5%` counting vs `0.01%` collector readings at `1200, 1350, 1500 °C`) and `IAEA97-K31-CS137` (`HFR-K3/1` sphere at `1600 °C`). Metrics: $\Delta_{\text{bound}}$, violation $V_s = \max(0, F_s - U_{\text{exp}})$, and ratio $F_s / U_{\text{exp}}$; never pooled into MAE/RMSE.
3. **Non-Comparable Context:** Degraded Bullock `UO2`/`UC2` curves (`1500 °C`), AGR-1 4-4-2 `HUNN16-442-CS137` (5 failed particles), `IAEA97-JAERI-A` (incompatible denominator), and AGR-1 irradiation totals (`D06`; no time-resolved intact Cs-137 data).

---

## Key Numerical and Experimental Findings

1. **Numerical Verification:** BTCS achieves observed convergence orders of **2.014 in space** ($O(\Delta r^2)$) and **1.077 in time** ($O(\Delta t)$). Across all **48 Stage A–D simulations**, maximum normalized floating-point mass-balance residual $|\varepsilon|/\text{bound}$ is **0.0498** ($\le 1.0$), minimum row margin is **1.0**, and Method A agrees with Method B within **$\le 1.51\times 10^{-3}$ pp** across `1200–1800 °C`.
2. **Effect of Progressive Physics (`G=0` Anneals):**
   - At **1700 °C (`ACT3`, $N=8$)**: Stage A (`RMSE = 0.240 pp, final Δ = -0.120 pp`) $\to$ Stage C decay-only (`RMSE = 0.241 pp, -0.23%`) $\to$ **Stage D heating ramp (`RMSE = 0.226 pp, final Δ = -0.035 pp`, +6.08% RMSE improvement, +7.97% MAE improvement)**.
   - At **1800 °C (`ACT4`, $N=11$)**: Stage A (`RMSE = 7.519 pp, final Δ = +12.85 pp`, intact bracket `[+12.23, +16.40]` pp) $\to$ Stage C decay-only (`7.516 pp, +0.05%`) $\to$ **Stage D heating ramp (`7.706 pp`, −2.48%)**, where pre-hold ramp diffusion slightly increases existing overprediction.
3. **Why Stages B/C (`+G`) Cannot Validate Furnace Tests:** Adding active fission generation (`+G`, $F_{\text{gen}}$ from $C=0$) shifts the release definition relative to source-free annealing ($G=0$, $F_{\text{init}}$), yielding lower fractions (`1.68%` at 1700 °C; `12.39%` at 1800 °C) that worsen ACT3 (`1.97 pp`, `−720%`) even while numerically sitting closer to ACT4 (`2.44 pp`). Numerical convergence and physical accuracy must never be conflated.
4. **Upper Bounds (`1200–1600 °C`):** Models satisfy `5%` counting bounds at `1200 °C` (`0.0184%`) and `1350 °C` (`1.48%`), exceed `5%` at `1500 °C` (`18.68%`), and exceed the stricter `0.01%` collector readings at all three (`1.84x` to `1868x`) as well as `HFR-K3/1` at `1600 °C` (`153x`).

---

## Repository Structure

```text
STRATO/
├── src/                        # Core Rust library (Method A BTCS & Method B Sturm–Liouville)
├── crates/
│   ├── strato-data/            # Typed parameter, geometry, Arrhenius, and experimental registers
│   ├── strato-scenarios/       # Scenario builders, heating histories, shared Method B driver
│   ├── strato-compare/         # Convergence studies (spatial_convergence, temporal_convergence)
│   ├── strato-results/         # Baseline results pipeline + Phase 2 `study39` simulation binary
│   ├── strato-wasm/            # WebAssembly solver wrapper
│   ├── strato-docsite/         # Mathematical documentation builder and link checker
│   └── strato-webtools/        # Website catalogue, Wasm build, and byte-identity checkers
├── data/
│   ├── study39/                # Canonical Phase 2 curves, experimental subsets, deviations, manifest
│   ├── results/                # Verified baseline pipeline outputs (strato-results --check)
│   ├── experimental/           # Audited experimental release datasets & curve_assessment.csv
│   └── ...                     # Geometry, diffusivity, decay, fission yield, temperature registers
├── graphs/study39/             # All 39 principal graphs + 10 supporting diagnostics (PNG, SVG, CSV)
├── deliverables/
│   ├── report/                 # Phase 2 Research Report (DOCX & PDF)
│   └── powerpoint/             # Phase 2 Editable PowerPoint Presentation (109 slides, PPTX)
├── web/                        # Four-page React + TypeScript + Plotly + Wasm web application
├── scripts/                    # Reproducible Python analysis, plotting, report, & deck builders
├── docs/                       # Mathematical proofs (Documents 1–6) & ANTIGRAVITY_PROGRESS.md
└── archive/                    # Legacy/archival index linking superseded reference material
```

---

## Reproducibility Instructions

```powershell
# 1. Run core Rust solver & regression tests
cargo test --release
cargo run --release --example btcs_regression -- --commit (git rev-parse HEAD)

# 2. Run Phase 2 canonical 39-graph Rust simulations
cargo run --release --manifest-path crates/Cargo.toml --bin study39 -- --repo . --commit (git rev-parse HEAD)

# 3. Compute experimental deviations, Stage A-D improvements, graphs, report & deck
python scripts/study39_deviations.py
python scripts/generate_study39_graphs.py
python scripts/build_report.py
python scripts/update_deck.py
python scripts/export_web_study39.py

# 4. Verify workspace crates & WebAssembly web app
cargo test --manifest-path crates/Cargo.toml --release
cargo run --locked --release --manifest-path crates/Cargo.toml -p strato-webtools -- build-wasm
Set-Location web; npm ci; npm run lint; npm run typecheck; npm test -- --run; npm run build; npm run check:content
```

---

## References and Acknowledgements

Full academic references (Bullock 1984; Collin et al. 2015, 2016; Demkowicz et al. 2015, 2016; Gelbard 2003; Gerczak et al. 2016; Hales et al. 2021; Hunn et al. 2014–2019; IAEA 1997, 2025; Miller et al. 2018; Minato et al. 2000; Morris et al. 2016; Parsons et al. 2024; Schappel et al. 2020; Simon et al. 2022; Stempien et al. 2023) are documented in [`deliverables/report/STRATO_Research_Report_Phase2_Final.pdf`](deliverables/report/STRATO_Research_Report_Phase2_Final.pdf) and in Scientific Documentation on the live website.

## Licence

Core solver source code (`src/`), Rust crates (`crates/`), and web application (`web/`) are licensed under **AGPL-3.0-only**.
