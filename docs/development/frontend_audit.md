# Frontend audit for the equation-first web demo

Date: 2026-11-24. Branch `feat/strato-interactive-web-demo` at `19067ed` (package updated for the owner's equation-first brief, `prompt/EQUATION_FIRST_REQUEST.md`, owner decision M7). This is deliverable 1 of the brief ("audit the existing website first"). It records what exists, what is implemented and what is only proposed, and which visual improvements need no change to the numerical model. Nothing here changes code.

## 1 Summary

- There is no live web demonstration yet. The live site is a Jekyll render of the repository's Markdown, built from `main`.
- A Phase 1 frontend scaffold exists in the development clone but is **not committed**: Vite, React 19, TypeScript, zustand, self-hosted OFL fonts. It contains a parameter panel, input checks that mirror `Config::validate`, a schematic particle and six section pages. It computes nothing, shows no equation and has no plotting, maths or 3D library installed.
- The numerical model is the verified v1.0 Rust solver (`src/`), unchanged since tag `v1.0.0`. Decay, generation and temperature-dependent diffusivity are derived and specified but not implemented.
- Because nothing has been built on the scaffold's left-panel layout yet, the brief's page order (equation, simulation, controls, explanation, future research) can be adopted now at little cost. "Avoid framework migrations" therefore means staying on React and Vite.

## 2 The live site

| Item | Finding | How it was checked |
|---|---|---|
| URL | https://weewzr.github.io/STRATO/ | opened 2026-11-24, HTTP 200 |
| Generator | `<meta name="generator" content="Jekyll v3.10.0">`; stylesheet `/STRATO/assets/css/style.css?v=e3a558f27134f778340380bef212661c77cd0c21`, so the page was built from `main` at `e3a558f` | page source |
| Content | a rendering of `README.md`: status, objective, geometry, mathematical model, installation, running, outputs, verification, limitations, layout, documentation, references, citation, licence; one figure (the TRISO cross-section) | page text |
| Mathematics | written in LaTeX in the README; no maths renderer is loaded, so formulas appear as source text | page source (no KaTeX or MathJax script) |
| Scripts | `anchor-js` 4.1.0 from cdnjs (heading anchors) and the theme's own inline script; nothing interactive | page source |
| Further pages | the Markdown documents are rendered too, for example `/STRATO/docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.html` (HTTP 200) | opened |
| Deployment | GitHub Pages "Deploy from a branch" (blocker B2 in `prompt/PROGRESS.md`); workflows `ci.yml` (format, clippy, tests, docs, release build, `verify`, synthetic run, `cargo audit`) and `docs-pdf.yml` (PDF build); no web build | `.github/workflows/` |

Consequence for rule E7: when the Pages source is switched to GitHub Actions, the Jekyll render stops. The new `web.yml` artefact must republish the derivation pages so that no published document disappears.

## 3 The numerical model behind the demo

| Part | State | Evidence |
|---|---|---|
| Five-layer spherical diffusion with constant $D_i$ per layer, regular centre, perfect interfaces, absorbing surface, kernel-uniform start | implemented and verified (research status VERIFIED, synthetic five-layer case) | `src/numerical/finite_volume.rs` (Code B), `src/analytical/sturm_liouville.rs` (Code C); `tests/`; `examples/verify.rs`; `tests/golden/v1.0/` |
| Verification evidence | spatial order 2.00, temporal order about 1, balance residual at most $2.806\times10^{-13}$ of the start inventory, Code B against Code C within the discretisation error | `prompt/SCIENTIFIC_REQUIREMENTS.md` Section 2, each value with its file |
| Browser readiness | the library compiles for `wasm32-unknown-unknown`; no wrapper crate exists yet | `prompt/PROGRESS.md`, Phase 0 |
| Fission-product generation, radioactive decay, inventory accounting, temperature-dependent diffusivity with layer-specific laws | derived, pseudocode written, parameters registered; not implemented (RESEARCHED) | `docs/mathematics/STRATO_Phase2_Derivations.md`, `docs/mathematics/STRATO_Phase2_Pseudocode.md`, `data/` |
| Every mechanism of bricks 4 to 9 of the brief (silver trapping and grain-boundary transport, palladium attack, caesium retention, irradiation damage, further species, FLiBe and cover gas) | nothing in the repository (PENDING) | search of `docs/` and `data/` |
| Reference fuel data | AGR-1 particle geometry registered (`data/geometry/geometry.csv`, design `AGR1-B-M16`, Morris et al. (2016) Table 2); kernel diffusivities registered only for UO2 (rule E3) | `data/` |

## 4 The uncommitted Phase 1 scaffold

The scaffold was written by an interrupted Phase 1 session and left untracked in `web/` and `scripts/web/` of the development clone (last file written 2026-11-24 16:38 local time, a production build at that time). It was read, built output was served locally and screenshotted, and nothing in it was changed.

### 4.1 Framework and dependencies

From `web/package.json` and `web/package-lock.json`:

| Kind | Packages (locked versions) |
|---|---|
| Runtime | `react` 19.3.1, `react-dom` 19.3.1, `zustand` 5.1.0, `@fontsource/literata` 5.3.0, `@fontsource/ibm-plex-sans` 5.3.0, `@fontsource/ibm-plex-mono` 5.3.0 |
| Build | `vite` 8.3.8, `@vitejs/plugin-react` 6.3.0, `typescript` 6.0.3 (strict, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`) |
| Quality | `eslint` 10.15.0 with `typescript-eslint` and `eslint-plugin-react-hooks`; `vitest` 5.1.1 with `jsdom` and Testing Library; `@playwright/test` 1.65.1 |
| Not installed | KaTeX, Plotly, three.js, React Three Fiber, any atomistic viewer |

Licence field `AGPL-3.0-only`. `vite.config.ts` sets `base: '/STRATO/'`, stamps the commit and STRATO version into `build.json`, a meta tag and a global `__STRATO_BUILD__`, and marks a build from a dirty tree. React 19.3.1 is inside the range `>=19 <19.4` that `@react-three/fiber` 9.8.2 declares (see the library comparison).

### 4.2 Structure

| Path | Content |
|---|---|
| `web/src/App.tsx`, `route.ts` | hash routing between six sections (Explore, Compare, Validate, Verify, Data, About); on desktop (at least 1200 px) a left parameter column; below that a parameter drawer (tablet) or bottom sheet (phone) opened by a button; skip link |
| `web/src/panels/` | `ControlPanel` (case, per-layer $D_i$ with log slider and number field, radius, cells, time controls, three physics switches kept `aria-disabled`, Run kept unavailable, Reset and Set baseline), `LayerSchematic` (SVG rings drawn to the input radii, labelled ILLUSTRATIVE, keyboard layer inspector), `Badge` (result labels as word plus shape) and `StatusChip` (two-level IMPLEMENTED / PLANNED), `Header`, `NumberField`, `StatusLine` |
| `web/src/sections/` | the six section pages; Explore holds the schematic and a "computes nothing yet" note; Verify lists the checks with links to the evidence files; Validate states that predictions need Phase 2 physics; About lists the labels, the build contents and the credits |
| `web/src/state/` | zustand store shaped as `prompt/TECHNICAL_ARCHITECTURE.md` Section 4 (no `stage` or `capabilities` yet); `validation.ts` mirrors `Config::validate` as UI checks |
| `web/src/data/`, `web/src/generated/`, `web/public/data/` | the synthetic case as JSON and CSV with a provenance header, written by `scripts/web/generate_data.py case` from `inputs/synthetic/synthetic_five_layer.toml` (standard library only, with `--check`) |
| `web/src/solver/types.ts` | TypeScript mirror of the planned `RunResponse`; no worker, no wasm |
| `web/src/selftest/` | the `?selftest=1` browser self-test skeleton |
| `web/src/design/` | `tokens.core.css` (shared core 1.0.0, copied), `tokens.css` (STRATO colours and faces, one colour per mechanism), `app.css` (975 lines) |
| `web/tests/` | Vitest: store, components, input validation, contrast ratios of the tokens, format, and guard tests (no exponent literal outside generated files, generated JSON equals the CSV, `tokens.core.css` copied unchanged, no SNRSI comparison text and no `prompt/` link in the sources) |
| `web/e2e/` | Playwright: behaviour (themes, no console error, no failed request, CSV provenance header, no `prompt/` link), keyboard (skip link, focus rings, slider, disabled physics switches, layer inspector, menu and sheet with Escape), layout (one `h1`, no sideways scroll, panel placement by breakpoint), screenshots, self-test with and without reduced motion |

Not yet present: `.github/workflows/web.yml`; `crates/strato-wasm`; `web/scripts/check_content.mjs`, which the `check:content` script in `package.json` names.

The test and lint commands were not run for this audit (the task was to read the scaffold, and running them would write build output into the untracked tree); the production build from 16:38 was used as it was.

### 4.3 Implemented in the scaffold versus only proposed

| Feature (design specification) | Scaffold |
|---|---|
| App shell, sections, themes, tokens, OFL fonts self-hosted | implemented |
| Parameter controls bound to the store, defaults from the generated case file, `Config::validate` mirrored | implemented |
| Physics switches present, focusable and disabled | implemented (two-level status, to be replaced by the five-level status) |
| Schematic particle to the input radii, layer inspection | implemented (ILLUSTRATIVE; all layers drawn in the same grey) |
| Equation (block A) and staged builder | not started; no maths renderer installed |
| Live simulation (block B): solver, profiles, release, balance, diagnostics | not started (Phase 3 to 5) |
| Equation-first page order A to E, fold requirement | not met: the page opens with a heading, a lede and the schematic beside a left parameter column; no equation |
| Research status system, catalogue, species, gallery, brick roadmap | not started (Phase 2 and Phase 5.5) |
| AGR-1 reference particle | not started |
| Web CI workflow | not started |

### 4.4 Performance

Measured on the 16:38 production build (files in `web/dist/assets`, gzip level 9 measured with Python's `gzip` module):

| Asset | Raw bytes | Gzip bytes |
|---|---|---|
| JavaScript (one chunk) | 270,039 | 82,740 |
| CSS | 21,021 | 4,870 |
| Fonts (7 `.woff2` files, plus `.woff` fallbacks) | 149,160 for the `.woff2` files | not compressed further |

The JavaScript is well inside the 1.5 MB gzip budget of `prompt/TECHNICAL_ARCHITECTURE.md` Section 6. The budget will be decided by what Phases 2 to 5 add: KaTeX about 76 kB gzip, three.js about 89 kB gzip and the Plotly cartesian bundle about 503 kB gzip (measured in `docs/development/visualisation_library_comparison.md`), so Plotly and three.js should be loaded lazily. No timing was measured (no solver yet).

### 4.5 Mobile responsiveness and accessibility

Screenshots of the served build at 1440 x 900 and at 390 px wide (a 390 px iframe in headless Microsoft Edge) were inspected:

- 1440 x 900: header with tabs, left parameter column of about 320 px, then the Explore heading, lede and the schematic with the layer list; the "computes nothing yet" note starts at the bottom edge. No equation is on screen.
- 390 px: header with a Menu button, a Parameters button that opens a bottom sheet, the heading and the schematic; no sideways scroll. The status line wraps to four lines and takes about 80 px of the 844 px height.
- Accessibility features present in the code and tests: skip link, labelled controls, visible focus rings, `aria-disabled` switches that announce why they are off, keyboard layer inspector, theme toggle that follows the system setting, `prefers-reduced-motion` honoured, contrast ratios of the colour tokens checked by a unit test, result labels shown as word plus shape.
- Not yet checked: an automated axe-core audit (planned for Phase 8), screen-reader reading of the SVG schematic beyond its `aria-label`.

## 5 Visual improvements that need no change to the numerical model

Each item below changes only presentation; none changes `src/`, the inputs, the solver output or any number.

1. Put the governing equation at the top of Explore, typeset with KaTeX and numbered, with the staged builder; stage 1 is the v1.0 equation exactly as solved.
2. Move the parameter column below the simulation (block C) and keep only Run, playback and the stage readout next to the simulation, so that the equation and the top of the simulation share the first screen at 1440 x 900.
3. Give each TRISO layer one consistent colour token (kernel, buffer, IPyC, SiC, OPyC) used in the schematic, the profile bands and the inspector; today all layers are the same grey.
4. Show the discrete update of each stage (the finite-volume cell balance (5.14) for stage 1) with a link to `backward_euler_step` at the build commit.
5. Replace the two-level status chip by the five-level research status with distinct shapes, read from the catalogue files.
6. Draw layer boundaries and labels on the cross-section and, from Phase 3, colour cells from the solver profile with cividis.
7. Show the AGR-1 particle from the registered geometry as the reference fuel, beside the synthetic simulation and clearly labelled as geometry only.
8. Shorten the phone status line (one or two lines) and give block B the phone width first.
9. Add the research catalogue, the species entries, the gallery placeholders and the brick roadmap as reading pages.
10. Load Plotly and three.js only on the views that need them.

Improvements that do need the model to change are not visual: anything about generation, decay or temperature dependence becomes live only through web Phase 6 and its verification.

## 6 Recommendations

- Continue from the scaffold (it already meets most Phase 1 rules: fonts, tokens, labels, guards, keyboard tests); change its layout to blocks A to E rather than starting again.
- Add the store fields `stage` and `capabilities` and the single `isAvailable()` function of `prompt/TECHNICAL_ARCHITECTURE.md` Section 4 before any switch can become active.
- Add `web.yml` with the catalogue check, and restore or remove the `check:content` script reference.
- Keep the dependency set to the one selected in `docs/development/visualisation_library_comparison.md`.
