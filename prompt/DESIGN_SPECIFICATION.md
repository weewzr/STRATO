# Design specification

This file agrees with the owner's equation-first brief (`prompt/EQUATION_FIRST_REQUEST.md`, owner policy decision M7 of 2026-11-24, with the rules E1 to E9 in `prompt/PROGRESS.md`). Where this file and the brief differ, the brief applies and this file is corrected in the same commit.

## 1 Design intent

An equation-first scientific instrument that teaches and shows. The reader first sees the governing equation that STRATO is solving, and immediately underneath it the live simulation of exactly that equation: particle, profile, release and inventory change together. Controls, explanation and the future-research catalogue follow below. Physics grows on the page one verified mechanism at a time; what is not implemented and verified is shown as research, with its status, and never as a result.

The honesty devices follow Theodore's Outram Park dispersion demo (https://theodoreonzgit.github.io/outram-park-backend/demos/#dispersion): a status line that names what was computed and how long it took, and explicit "recorded, not live" labels. Its left control column is not used: the page order of Section 2 replaces it (M7). STRATO uses DOM components (React, KaTeX, Plotly) where Outram Park uses an egui canvas, so text stays selectable, maths is typeset and screen readers work.

## 2 Page structure

### 2.1 Explore page: order A to E

The Explore page is one column in this order. The order is the same at every width; only density changes.

| Block | Content | Section of this file |
|---|---|---|
| Header | title "STRATO", subtitle, section navigation, theme toggle, link to the repository | 2.3 |
| A Governing equation | the active governing equation, typeset with KaTeX, with the staged equation builder (stage picker, highlighted new term, status of each term) | 3 |
| B Live simulation | the simulation of the equation shown in A: particle cross-section with layer boundaries, radial concentration profile, time evolution (playback), release fraction, inventory balance, numerical diagnostics; a compact strip with Run, playback and the stage readout sits on its top edge | 4 |
| C Interactive controls | case selector, layer parameters, time controls, mechanism switches, Run, Reset to baseline, Set current as baseline | 5 |
| D Scientific explanation | each active term: meaning, assumptions, units, physical meaning; the boundary, interface and initial conditions; what the case is and is not | 6 |
| E Future research mechanisms | a summary of the research catalogue: the three transport regimes, the nine species and the bricks, each with its research status, linking to the Research section | 7 |
| Status line | footer: label of the current main result, solver time, commit | 9 |

Fold requirement: at a 1440 x 900 CSS-pixel viewport, with the default case, at the top of the page and without scrolling, block A (the whole typeset equation of the active stage and its stage picker) and the top of block B (the particle cross-section and the release-fraction readout, each with its result label) are entirely inside the viewport. The test is in `prompt/TESTING_AND_VALIDATION.md` Section 3.

| Width | Layout |
|---|---|
| Desktop (at least 1200 px) | A full width, under it B with the particle and the profile side by side and the release and balance readouts beside or under them; C as a horizontal panel grouped by layer; D and E as reading columns of at most about 75 characters |
| Tablet (768 to 1199 px) | same order, B stacked (particle, then profile, then release and balance) |
| Phone (under 768 px) | same order; A shows the active equation with horizontal scrolling inside its own box (never the page) and the stage picker as a select; B shows the 2D cross-section and one plot at a time with a plot switcher; C collapses into labelled groups |

No width has sideways page scrolling. The particle and the equation are never hidden behind a control drawer.

### 2.2 Sections

Navigation (tabs or anchored scroll): **Explore** (blocks A to E), **Compare** (model comparison), **Validate** (literature graph), **Verify** (verification and convergence), **Research** (the full research catalogue: regimes, species, pending mechanisms, Materials Studio gallery, brick roadmap; Section 7), **Data** (provenance, references, downloads), **About** (model, assumptions, credits including Outram Park).

### 2.3 Header

Title, subtitle, navigation (tabs on wide screens, a menu on phones), theme toggle, repository link. The header does not grow so tall that it breaks the fold requirement.

## 3 Block A: staged equation builder

### 3.1 Stages

The builder introduces physics one stage at a time, starting from the actual STRATO v1.0 equation and geometry (not a generic approximation). Each stage adds to the previous one.

| Stage | Name | What is added | Source equations | Research status on 2026-11-24 |
|---|---|---|---|---|
| 1 | Base diffusion | the v1.0 layered equation as it is: five spherical layers, each with its own constant $D_i$, regular centre, perfect interfaces, absorbing surface, kernel-uniform start | v1.0.0 Full Derivation Sections 2.6 and 3, discrete form (5.14) and (5.15) | VERIFIED (synthetic five-layer case) |
| 2 | Fission-product generation | $+S(r,t)$, kernel source $S = \dot F\,y\,\chi_1/N_A$ | (P21.1), (P21.8) | RESEARCHED |
| 3 | Radioactive decay | $-\lambda c$, $\lambda = \ln 2/t_{1/2}$ | (P21.1), (P21.7), discrete (P21.58) | RESEARCHED |
| 4 | Temperature-dependent diffusivity | $D_i \to D_i(T(t))$ with the Arrhenius law | (P22.1), discrete (P22.15) | RESEARCHED |
| 5 | Layer-dependent transport | a separate temperature law in each layer: layer-specific $D_{0,i,m}$ and $Q_{i,m}$ (rule E1). Interface partition coefficients are not in the model and are listed as PENDING | Phase 2 Derivations Section 5.4 | RESEARCHED (layer-specific laws); PENDING (partition coefficients) |
| 6 | Species-specific mechanisms | only mechanisms that are implemented; today none, so the stage lists the species-specific catalogue entries (Section 7) with their status and disabled switches | catalogue entries | PENDING |

The status column is shown here for orientation only. On the page every status is read from `data/research/mechanisms.csv` (`prompt/TECHNICAL_ARCHITECTURE.md` Section 5.1), never typed into a component. Stage 1 shows the layered equation exactly as v1.0 solves it; stage 5 says plainly that the layering itself is already in stage 1 and that what is new is the temperature law per layer (E1). Stages 2 and 3 follow the brief's order for display; both arrive together in web Phase 6.1 and can then be enabled separately (E8).

### 3.2 What every stage shows

- **The new term, highlighted.** The term introduced by the stage is inserted with a short transition (fade and slide, 200 to 300 ms; instant under reduced motion) and a highlight in the mechanism's colour (Section 9). `D_i` morphs to `D_i(T(t))` at stage 4. Earlier terms stay, unhighlighted.
- **Its meaning.** A card (on hover, focus or tap) gives the physical meaning, the units of every symbol, the assumptions (A1 to A6 of `prompt/SCIENTIFIC_REQUIREMENTS.md` Section 3 where they apply), the equation number and a link to the derivation section in `docs/mathematics/`.
- **Its numerical implementation.** The discrete update that STRATO uses (or will use) for the stage, typeset: the Backward Euler finite-volume cell balance (5.14) for stage 1, (P21.58) for stages 2 and 3, (P22.15) for stages 4 and 5. Each carries a link to the code at the build commit (for stage 1 `backward_euler_step` in `src/numerical/finite_volume.rs`); a stage without code says "derived, not implemented" and links the pseudocode in `docs/mathematics/STRATO_Phase2_Pseudocode.md` instead.
- **A live update only if supported.** Selecting a stage changes the simulation in block B only if the Rust wrapper reports the mechanism as supported and its status is VERIFIED (`prompt/TECHNICAL_ARCHITECTURE.md` Section 4). Otherwise block A still shows the stage's equation, block B keeps the last supported stage, and a banner across both reads "Equation preview only: the simulation below solves stage N". The page never implies that a switch changes the physics when it only changes the display.
- **The inventory balance.** Under the equation, the balance that belongs to the stage, with its residual, from the wrapper (M4): for stage 1, inventory plus released minus start inventory; from stage 2 on, start inventory plus generated, minus decayed, minus released, minus remaining (E9, (P21.42)).
- **A baseline comparison where appropriate.** From stage 2 on, a "compare with stage 1" control that shows the stage's release curve beside the base-diffusion curve and their difference, all from `compare_v1` or its v2 successor in Rust (never subtracted in the browser). Not offered for a stage that is not supported.

### 3.3 Rules

- The baseline equation of stage 1 is always visible, whatever stage is selected.
- A mechanism that needs additional coupled equations (for example trapping with a trapped-population balance, or a gas-pressure model) is shown as a coupled system of equations next to the transport equation, never inserted artificially into the single transport equation.
- Each term carries a research-status chip with one of the five words of Section 7.1, as word plus shape. The two-level IMPLEMENTED / PLANNED chip of earlier versions of this file is replaced.
- Below the equation, collapsible rows show the boundary and interface conditions and the initial condition of the active stage.
- Display equations are numbered and right-aligned in brackets (M6).
- No number appears in block A apart from equation numbers and units.

## 4 Block B: live simulation

### 4.1 Particle

- 2D cross-section by default, directly under the equation; a switch offers the 3D cutaway (orbit, zoom, reset view; keyboard rotation with arrow keys) where WebGL 2 is available, with the 2D view as fallback.
- Layers named kernel, buffer, IPyC, SiC, OPyC, with radii from the active configuration and layer boundaries drawn and labelled. A schematic-only mode (no data) is labelled ILLUSTRATIVE.
- Concentration colour from the current profile at the playback time, cividis by default, linear or log, legend with units and maximum.
- Layer inspection: selecting a layer highlights it and shows its radii, $D_i$, current inventory share (the `layer_share` value returned by the Rust wrapper, never computed in the browser) and provenance.
- Reference particle (rule E2): a "Reference fuel: AGR-1 UCO TRISO" view draws the AGR-1 particle from the registered geometry in `data/geometry/geometry.csv` (Section 7.7), labelled with its row ids and status. It is geometry only. The live simulation of Brick 1 stays the synthetic five-layer verification case and says so next to the particle ("Simulation: synthetic five-layer verification case, not AGR-1 data"). AGR-1 geometry is never combined with the synthetic diffusivities.

### 4.2 Playback

Play, pause, step back, step forward, speed (0.25x to 8x), time slider with the current time in seconds and in a readable unit. Space toggles play; arrow keys step. Under reduced motion, playback does not auto-start.

### 4.3 Radial concentration profile

Plot of $c(r)$ at the playback time, with layer boundaries as vertical bands labelled by layer name; optional overlay of the initial profile. Log $y$ option.

### 4.4 Release fraction and time evolution

$F(t)$ from Code B (finite volume) and Code C (series) on the same axes, with the series' validity start and accuracy floor marked, and a playback cursor. A readout states $F$ at the playback time with its label.

### 4.5 Inventory balance and numerical diagnostics

The balance of the active stage (Section 3.2) as a small table or bar with its residual; the diagnostics the wrapper returns: cell count, step count, time step actually used ($\Delta t = t_\mathrm{end}/\lceil t_\mathrm{end}/\Delta t_\mathrm{in}\rceil$), maximum balance residual, Code B against Code C difference, solver time. Every value carries its result label.

## 5 Block C: interactive controls

- Case selector: "Synthetic five-layer verification case" (default, from `inputs/synthetic/synthetic_five_layer.toml`). Further cases only from files in `inputs/` or the parameter register.
- Layer parameters: per layer, $D_i$ (log slider and number field) and outer radius, each with unit and provenance link; values outside the validated range show a warning and the run returns `rejected` if `Config::validate` refuses them.
- Time: end time and step, with the step rule shown.
- Mechanism switches (generation, decay, temperature-dependent diffusivity, layer-specific laws, species-specific mechanisms): each shows its research status. A switch is active only when the wrapper reports the mechanism as supported and its status is VERIFIED. Before that it is visible, keyboard-focusable and disabled (`aria-disabled`), with the text "Not available: status STATUS; awaiting reviewed implementation (see the Phase 2 review package)". Operating a disabled switch changes no result and sends no solver request.
- Buttons: Run, Reset to baseline, Set current as baseline.
- Every control is reachable by keyboard and has a visible label; the stage picker of block A and the mechanism switches stay in step (selecting stage 3 shows the decay switch as the one the stage introduces).

## 6 Block D: scientific explanation

- For each active term: what it represents, its assumptions, the units of every symbol and its physical meaning, in plain language first and then with the symbols, for non-specialists without loss of accuracy.
- The boundary, interface and initial conditions in words.
- What the current case is: the synthetic five-layer case is a verification case whose parameters are not physical TRISO data, and the page says so (`prompt/SCIENTIFIC_REQUIREMENTS.md` Section 2).
- Verification is not validation: a short paragraph and a link to the Verify and Validate sections.
- Stable and radioactive isotopes are named as such; decay constants come from `data/nuclear/decay.csv`.

## 7 Block E and the Research section: future research mechanisms

Block E on the Explore page is a summary (the three regimes, the nine species, the bricks, each with status counts and a link). The Research section holds the full catalogue. Everything in it is read from the catalogue files of `prompt/TECHNICAL_ARCHITECTURE.md` Section 5.1.

### 7.1 Research-status system

Every mechanism has exactly one of five statuses, with these definitions (from the brief):

| Status | Definition | Shape on screen |
|---|---|---|
| PENDING | Research is proposed but has not been sufficiently established or implemented. | dotted outline |
| RESEARCHED | Relevant literature, candidate equations and parameter requirements have been documented. | dashed outline |
| IMPLEMENTED | The numerical mechanism exists in STRATO but may not yet be fully verified. | solid outline |
| VERIFIED | Numerical implementation passes documented conservation, convergence and limiting-case tests. | solid fill |
| VALIDATED | Predictions have been compared with relevant independent experimental evidence within a clearly defined applicability domain. | solid fill with a check mark |

Rules:

- A status is assigned from evidence in the repository, per mechanism and, where necessary, per species and material domain (for example "five-layer diffusion, any species, synthetic five-layer case: VERIFIED"). It is never inferred from the existence of a webpage, paper, placeholder or configuration.
- Every newly proposed advanced mechanism starts at PENDING.
- VALIDATED is used only when the necessary experimental evidence supports it: a computed model-to-measurement comparison of web Phase 7 within a stated applicability domain (`prompt/TESTING_AND_VALIDATION.md` Section 7). Until such comparisons exist, owner decision M5 also forbids any experimental-agreement figure.
- No brick advances to VERIFIED or VALIDATED without the corresponding evidence.
- The status is shown as the word plus its shape, never by colour alone, and is distinct from the result labels of Section 10: the status describes a mechanism, the label describes a number.
- The status of every mechanism today is recorded in `prompt/PROGRESS.md` (M7).

### 7.2 Three transport regimes

The catalogue is organised under three regimes. They are transport regimes, not mutually exclusive mechanisms; an entry may name interactions with entries of another regime where that is scientifically justified.

| Regime | Mechanisms to investigate |
|---|---|
| A Intact-barrier transport | bulk lattice diffusion; grain-boundary diffusion; segregation; trapping and detrapping; species-specific retention |
| B Degradation-assisted transport | palladium-assisted migration; Pd-SiC chemical interactions; irradiation defects; microstructural degradation; defect-assisted diffusion; relevant chemical interactions |
| C Barrier failure and bypass transport | microcracking; connected transport pathways; coating failure; manufacturing defects; fission-gas pressure; failed-particle release |

### 7.3 Pending mechanism entries

For each mechanism that is not implemented, the entry shows: the mechanism name; the relevant species; the proposed physics; candidate governing equations as research concepts (typeset, labelled "research concept, not solved by STRATO", each taken from a cited source); the required parameters and whether the register holds them; links to supporting literature; conceptual diagrams where useful (labelled ILLUSTRATIVE); genuine Materials Studio screenshots when available (Section 7.5); the missing calculations or experiments; and a disabled live-simulation switch until the mechanism is implemented and VERIFIED.

Never shown for a pending mechanism: concentration profiles, migration barriers, diffusivities or release fractions that were not taken from a cited source or computed by STRATO, and any hypothetical output presented as a STRATO prediction.

### 7.4 Species entries

Nine species, each with its own expandable entry or page: Ag (silver), Pd (palladium), Cs (caesium), Sr (strontium), Eu (europium), I (iodine), Ru (ruthenium), Xe (xenon), Kr (krypton). Each entry explains: why it matters; the relevant TRISO layers; candidate release mechanisms; current evidence; proposed governing-equation extensions; potential Materials Studio calculations; experimental comparison opportunities; research limitations; implementation status (the statuses of that species' mechanism rows).

Rules: no entry implies that a species is already simulated; mechanisms are never activated automatically for every species; stable and radioactive isotopes are distinguished, and half-lives and decay constants come from `data/nuclear/decay.csv` or a cited source (a nuclide missing from the register is marked as such, not filled in).

### 7.5 Materials Studio gallery

A dedicated atomistic research gallery. Priority subjects: AGR-1-relevant SiC crystal structures; representative grain boundaries; Ag impurity configurations; vacancies and interstitials; migration pathways; trapping sites; Pd-SiC interactions; relevant defect structures.

Each item has: a scientific caption; an accessible explanation (also the image's text alternative); material and species; computational method; simulation conditions; source or provenance; research status.

Rules (rule E5):

- Only genuine BIOVIA Materials Studio screenshots supplied by the owner with method and conditions are shown as screenshots. No imitation screenshot, rendering or mock-up is made.
- Where a calculation has not been performed, the item is a Pending placeholder: a plain panel with the word PENDING and a description of the planned calculation, no picture. The owner confirmed on 2026-11-24 that no Materials Studio screenshots or structure files exist yet; the gallery says in words that these calculations will be built in future phases.
- Actual screenshots and interactive reconstructions (for example a later atomistic viewer of an exported structure, rule E6) are labelled differently and never mixed in one figure.
- An ideal 3C-SiC crystal is never presented as representing the full polycrystalline SiC coating; published microstructural evidence guides SiC illustrations where available.
- Atomistic migration barriers are never presented as, or converted on the page into, a macroscopic effective diffusivity.

### 7.6 Brick roadmap

A page that shows the brick-by-brick development sequence of the brief (bricks 1 to 9), each with its research status and the web phase that delivers it (mapping in `prompt/IMPLEMENTATION_ROADMAP.md`). Bricks 4 to 9 are catalogue entries with status PENDING and no implementation.

### 7.7 Reference fuel and experimental conditions

- AGR-1 UCO TRISO is the primary reference fuel. Its five layers (fuel kernel, buffer, inner pyrolytic carbon, silicon carbide, outer pyrolytic carbon) are drawn from the registered geometry in `data/geometry/geometry.csv`: by default the AGR-1 Baseline characterisation means (design `AGR1-B-M16`, rows G-01 to G-05, Morris et al. (2016) Table 2, status verified); other registered AGR-1 sets are selectable and labelled with their own row ids.
- Material properties are shown only from registered rows. The registered kernel diffusivities are for UO2; any AGR-1 use of them is labelled with that assumption (rule E3).
- AGR-2 is named as a future independent comparison and AGR-3/4 as a future failed-particle transport study; neither is shown as data before rows exist in `data/`.
- The AGR experiments were made under gas-cooled conditions. Any later FLiBe or cover-gas work is a separate, downstream topic (Brick 9, PENDING, described only as planned, rule E4; the owner confirmed on 2026-11-24 that no FLiBe or cover-gas material exists yet, and the Brick 9 entry and the brick roadmap page say that this work will be built in future phases) and is never shown on the same axes or in the same table as AGR results without saying so.

## 8 Other sections

### 8.1 Model comparison graph (Compare)

- Baseline curve, modified curve and an optional difference curve (modified minus baseline, as returned by the wrapper's `compare_v1`; never subtracted in the browser) on a secondary axis.
- Before web Phase 6, "modified" means a changed v1.0 parameter set (for example a different SiC diffusivity), both COMPUTED v1.0. After Phase 6 it can include the extended physics (COMPUTED v2.0).
- Legend states both configurations; a table lists the parameters that differ.

### 8.2 Literature-validation graph (Validate, independent of Compare)

- Measurements from `data/experimental/release_measurements.csv` with error bars or digitising uncertainty and a reference for each series.
- Model predictions only after web Phase 6, from COMPUTED v2.0 runs with the matching temperature history and nuclide. Before that the prediction layer is absent and a notice says why.
- Error metrics (Section 7 of `prompt/TESTING_AND_VALIDATION.md`) appear under the graph when predictions exist.

### 8.3 Verification panel (Verify)

Code B against Code C error, convergence plots (spatial and temporal, with reference slopes 2 and 1), balance residual, golden regression status and native/wasm parity, each with its label (COMPUTED v1.0 or PRECOMPUTED) and the commit that produced it.

### 8.4 Provenance and references (Data)

A searchable table of every parameter in use: value, unit, status, source, page or table, register row id. A reference list for the papers cited in `README.md`, the register and the research catalogue (`data/research/references.csv`).

### 8.5 Downloads (Data)

CSV and JSON of the current run, the baseline run and the comparison, each with a metadata header (label, STRATO version, commit, full inputs, timestamp, units). Plot images as SVG or PNG.

## 9 Design system

- Tokens in `web/src/design/tokens.css` as CSS custom properties, with light and dark themes (`prefers-color-scheme`, plus a manual toggle). Background, surface, text, muted text, accent, one colour per mechanism (generation, decay, temperature) and one consistent colour per TRISO layer (kernel, buffer, IPyC, SiC, OPyC), used identically in the particle, the profile bands, the layer inspector and the reference-fuel view; all chosen to stay distinguishable for common colour-vision deficiencies.
- Typography: a readable sans for UI, a monospace for numbers in tables; tabular figures for changing numbers.
- Spacing on a 4 px scale; minimum touch target 44 px.
- Colour maps: cividis default, viridis option; never rainbow.
- Charts: every axis labelled with quantity and unit; readable at phone width.
- Interactions: smooth but restrained; an interaction must tell the reader something.
- Implemented and pending research are visually separated: pending content sits in its own outlined panels with the status shape and never uses the chart and particle styles of computed results.
- Accessibility: WCAG 2.2 AA contrast; every control reachable and operable by keyboard with visible focus; ARIA labels on canvases with a text summary of what they show; `prefers-reduced-motion` honoured everywhere.

### 9.1 Visual identity (owner decision M6, 2026-11-24)

The owner chose the identity "an instrument with a monograph behind it" for STRATO (recorded in `prompt/PROGRESS.md`, owner policy decision M6). It gives the rules above a visual form and replaces none of them; where this specification is more specific, this specification applies.

- The Explore view is a calm instrument (equation, simulation, controls and explanation in that order, and a status line). Behind it sit typeset, numbered and versioned derivation pages.
- Fonts: open-licensed (SIL OFL) faces only, self-hosted from the site's own origin: Literata for document text and headings, IBM Plex Sans for interface text and controls, IBM Plex Mono for values, units and the status line, with tabular figures. No font from an outside host and no font under any other licence.
- Maths: KaTeX (Section 3), display equations numbered and right-aligned in brackets.
- Colour maps: cividis default, viridis option (unchanged from above).
- Result badges: `COMPUTED v1.0`, `COMPUTED v2.0`, `PRECOMPUTED`, `MEASURED`, `ILLUSTRATIVE` (Section 10), each shown as a word plus a shape, never by colour alone. Research statuses (Section 7.1) use their own shapes, distinct from the badges.
- Tokens: `web/src/design/tokens.css` holds STRATO's own colours and faces and imports a copied `tokens.core.css` (shared spacing, radii, durations, focus width, minimum sizes and colour-vision-safe palette). The shared written specification lives in `docs/DESIGN_SYSTEM.md` of the owner's portfolio repository; there is no shared component library and no command palette on this site.
- Third-party component kits are inspiration only; nothing is copied from them.
- Avoid what reads as generic: purple gradients, glowing statistic cards, decorative blobs, repeated oversized rounded cards, glass effects without purpose, vague slogans, invented metrics, decoration-only animation. Prefer editorial typography on a real type scale, one consistent card and table system, labelled navigation, figures with units and sources, and interactions that tell the reader something.
- Every figure and exported file carries its source, the model version and the commit. Figure data come from one generator that writes CSV and JSON with a provenance header (Section 8.5 and `prompt/TECHNICAL_ARCHITECTURE.md` Section 5), never from numbers typed into components.

## 10 Labels on screen

Every number, plot and particle colouring shows one of `COMPUTED v1.0`, `COMPUTED v2.0`, `PRECOMPUTED`, `ILLUSTRATIVE`, `MEASURED` (definitions in `prompt/SCIENTIFIC_REQUIREMENTS.md`). The status line names the label of the current main result. Every mechanism, term and catalogue entry shows its research status (Section 7.1).

## 11 Credits

The About section credits Theodore's Outram Park project (https://github.com/theodoreOnzGit/outram-park-backend, AGPL-3.0-only) for the interface ideas, and lists any file copied or adapted from it with its commit.

The site neither publishes nor links the `prompt/` folder (owner decision M3), and it does not repeat, quote or extend the README sentence about the internal SNRSI comparison (owner decision M5).
