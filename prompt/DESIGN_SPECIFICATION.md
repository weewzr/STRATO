# Design specification

## 1 Design intent

A single-page scientific dashboard that teaches and shows: the reader sees the particle, the equation and the numbers change together. Layout and honesty devices follow Theodore's Outram Park dispersion demo (https://theodoreonzgit.github.io/outram-park-backend/demos/#dispersion): a control side panel, a large central view, a status line that names what was computed and how long it took, and explicit "recorded, not live" labels. STRATO uses DOM components (React, KaTeX, Plotly) where Outram Park uses an egui canvas, so text stays selectable, maths is typeset and screen readers work.

## 2 Page structure

| Region | Desktop (at least 1200 px) | Tablet (768 to 1199 px) | Phone (under 768 px) |
|---|---|---|---|
| Header | title "STRATO", subtitle, section tabs, theme toggle, link to repository | same, tabs scroll | title and a menu button |
| Control panel | left column, about 320 px, scrolls | collapsible drawer | bottom sheet |
| Main view | particle (3D or 2D) over equation explorer | stacked | stacked, 2D cross-section default |
| Plots | right column or below the main view | below | below, one plot at a time |
| Status line | footer: label of the current result, solver time, commit | same | same |

Sections (tabs or anchored scroll): **Explore** (particle, equation, playback, profile), **Compare** (model comparison), **Validate** (literature graph), **Verify** (verification and convergence), **Data** (provenance, references, downloads), **About** (model, assumptions, credits including Outram Park).

## 3 Panels

### 3.1 Control panel

- Case selector: "Synthetic five-layer verification case" (default, from `inputs/synthetic/synthetic_five_layer.toml`). Further cases only from files in `inputs/` or the parameter register.
- Layer parameters: per layer, $D_i$ (log slider and number field) and outer radius, each with unit and provenance link; values outside the validated range show a warning and the run returns `rejected` if `Config::validate` refuses them.
- Time: end time and step, with the step rule shown.
- Physics toggles: Decay, Generation, Temperature-dependent diffusivity. Before web Phase 6 they are visible, keyboard-focusable and disabled, with the text "Planned: awaiting reviewed implementation (see Phase 2 review package)". Toggling them in "preview" mode still grows the equation, but no result changes and a banner says "Equation preview only".
- Buttons: Run, Reset to baseline, Set current as baseline.

### 3.2 Growing equation explorer

- The baseline equation is always visible and typeset with KaTeX.
- Switching on a mechanism inserts its term with a short animation (fade and slide, 200 to 300 ms; instant under reduced motion) and a coloured highlight that matches the toggle's colour. `D_i` morphs to `D_i(T(t))` for the temperature toggle.
- Each term has a status chip: IMPLEMENTED (solid), PLANNED (outlined, with "derived, not implemented").
- Hovering or focusing a term shows a card: meaning, units, equation number, link to the derivation section in `docs/mathematics/`.
- Below the PDE, collapsible rows show the boundary and interface conditions and the initial condition.

### 3.3 Five-layer particle

- 3D cutaway (orbit, zoom, reset view; keyboard rotation with arrow keys) and a 2D cross-section; a switch chooses between them.
- Layers named kernel, buffer, IPyC, SiC, OPyC, with radii from the active config. A schematic-only mode (no data) is labelled ILLUSTRATIVE.
- Concentration colour from the current profile at the playback time, cividis by default, linear or log, legend with units and maximum.
- Layer inspection: selecting a layer highlights it and shows its radii, $D_i$, current inventory share and provenance.

### 3.4 Playback

Play, pause, step back, step forward, speed (0.25x to 8x), time slider with the current time in seconds and in a readable unit. Space toggles play; arrow keys step. Under reduced motion, playback does not auto-start.

### 3.5 Radial concentration profile

Plot of $c(r)$ at the playback time, with layer boundaries as vertical bands labelled by layer name; optional overlay of the initial profile. Log $y$ option.

### 3.6 Release curves

$F(t)$ from Code B (finite volume) and Code C (series) on the same axes, with the series' validity start and accuracy floor marked; a playback cursor.

### 3.7 Model comparison graph (toggle: Compare)

- Baseline curve, modified curve and an optional difference curve (modified minus baseline) on a secondary axis.
- Before web Phase 6, "modified" means a changed v1.0 parameter set (for example a different SiC diffusivity), both COMPUTED v1.0. After Phase 6 it can include the extended physics (COMPUTED v2.0).
- Legend states both configurations; a table lists the parameters that differ.

### 3.8 Literature-validation graph (toggle: Validate, independent of Compare)

- Measurements from `data/experimental/release_measurements.csv` with error bars or digitising uncertainty and a reference for each series.
- Model predictions only after web Phase 6, from COMPUTED v2.0 runs with the matching temperature history and nuclide. Before that the prediction layer is absent and a notice says why.
- Error metrics (Section 7 of `prompt/TESTING_AND_VALIDATION.md`) appear under the graph when predictions exist.

### 3.9 Verification panel

Code B against Code C error, convergence plots (spatial and temporal, with reference slopes 2 and 1), balance residual, golden regression status and native/wasm parity, each with its label (COMPUTED v1.0 or PRECOMPUTED) and the commit that produced it.

### 3.10 Provenance and references

A searchable table of every parameter in use: value, unit, status, source, page or table, register row id. A reference list for the papers cited in `README.md` and the register.

### 3.11 Downloads

CSV and JSON of the current run, the baseline run and the comparison, each with a metadata header (label, STRATO version, commit, full inputs, timestamp, units). Plot images as SVG or PNG.

## 4 Design system

- Tokens in `web/src/design/tokens.css` as CSS custom properties, with light and dark themes (`prefers-color-scheme`, plus a manual toggle). Background, surface, text, muted text, accent, and one colour per mechanism (decay, generation, temperature) chosen to stay distinguishable for common colour-vision deficiencies.
- Typography: a readable sans for UI, a monospace for numbers in tables; tabular figures for changing numbers.
- Spacing on a 4 px scale; minimum touch target 44 px.
- Colour maps: cividis default, viridis option; never rainbow.
- Accessibility: WCAG 2.2 AA contrast; every control reachable and operable by keyboard with visible focus; ARIA labels on canvases with a text summary of what they show; `prefers-reduced-motion` honoured everywhere.

## 5 Labels on screen

Every number, plot and particle colouring shows one of `COMPUTED v1.0`, `COMPUTED v2.0`, `PRECOMPUTED`, `ILLUSTRATIVE`, `MEASURED` (definitions in `prompt/SCIENTIFIC_REQUIREMENTS.md`). The status line names the label of the current main result.

## 6 Credits

The About section credits Theodore's Outram Park project (https://github.com/theodoreOnzGit/outram-park-backend, AGPL-3.0-only) for the interface ideas, and lists any file copied or adapted from it with its commit.
