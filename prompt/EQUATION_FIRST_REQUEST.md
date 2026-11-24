# Owner request: equation-first live demo

This is the owner's brief of 2026-11-24 for the equation-first layout of the web demo, the research-status system and the future-research catalogue, kept verbatim for reference. It is recorded in `prompt/PROGRESS.md` as owner policy decision M7, together with the rules that settle the points it leaves open; where it differs from `prompt/DESIGN_SPECIFICATION.md`, this brief applies. Later owner decisions in `prompt/PROGRESS.md` and the other `prompt/` files take precedence where they differ.

---

# STRATO — Equation-First Live Web Demo, Scientific Visualisation and Progressive Research Roadmap

## Mission

Improve STRATO's live web demonstration into a visually polished, interactive and scientifically rigorous research platform.

The interface must prioritise the governing equation, followed immediately by the live numerical simulation.

The project must develop progressively, beginning with STRATO's existing verified physics and introducing additional mechanisms only after sufficient research, implementation and verification.

**Do not implement every proposed fission-product mechanism immediately. All unverified future mechanisms must initially be labelled PENDING.**

Preserve the existing STRATO numerical baseline, repository structure, verification results and scientific provenance.

## 1. Audit the existing website first

Before modifying code:

1. Inspect the current STRATO website, frontend framework, numerical solver and deployment architecture.
2. Identify existing visualisation libraries and reusable components.
3. Review the current governing equations, numerical implementation and verified features.
4. Identify what is already implemented versus merely proposed.
5. Assess frontend performance, mobile responsiveness and accessibility.
6. Determine which visual improvements can be made without altering the numerical model.

Avoid unnecessary dependency additions or framework migrations.

## 2. Research open-source visualisation repositories

Evaluate the following GitHub repositories:

### Scientific equations and plots

- https://github.com/KaTeX/KaTeX
- https://github.com/plotly/plotly.js
- https://github.com/recharts/recharts

### Interactive TRISO particle visualisation

- https://github.com/pmndrs/react-three-fiber
- https://github.com/pmndrs/drei

### Atomistic materials visualisation

- https://github.com/janosh/matterviz
- https://github.com/3dmol/3Dmol.js
- https://github.com/nglviewer/ngl

### Advanced scientific visualisation

- https://github.com/Kitware/vtk-js
- https://github.com/Kitware/glance

Research additional suitable repositories where useful.

For each candidate, evaluate:

- Scientific visualisation capabilities.
- Compatibility with the existing frontend.
- Maintenance and release activity.
- Licence compatibility.
- Performance and bundle size.
- Mobile support.
- Accessibility.
- Documentation.
- Integration complexity.
- Whether it adds meaningful functionality beyond existing components.

Prioritise MatterViz for potential future atomic-scale SiC visualisation, KaTeX for equations, and Plotly for scientific graphs, subject to compatibility checks.

Consider React Three Fiber for an interactive multilayer TRISO visualisation only if the existing architecture supports it without unnecessary complexity.

Do not install every library. Select the smallest coherent set that provides the best results.

## 3. Main live-demo layout

The page must follow this order:

### A. Governing equation — near the top

Display the active STRATO governing equation prominently, using clear mathematical typography.

Include a progressive equation builder that introduces physical mechanisms one at a time.

### B. Live simulation — immediately underneath

Display the simulation corresponding to the selected governing equation.

Include:

- TRISO particle cross-section.
- Layer boundaries.
- Radial concentration profiles.
- Time evolution.
- Release fraction.
- Inventory conservation.
- Relevant numerical diagnostics.

### C. Interactive controls

Provide controls for supported parameters and mechanisms.

### D. Scientific explanation

Explain each active term, its assumptions, units and physical meaning.

### E. Future research mechanisms

Display planned mechanisms and species-specific extensions with clearly labelled research statuses.

Keep the initial equation and live simulation visible without excessive scrolling.

## 4. Progressive equation builder

Start with the simplest relevant STRATO transport equation.

Introduce additional physics progressively:

1. Base diffusion.
2. Fission-product generation.
3. Radioactive decay.
4. Temperature-dependent diffusivity.
5. Layer-dependent transport.
6. Species-specific mechanisms, only when implemented.

Use the actual existing STRATO equations and geometry rather than replacing them with a generic approximation.

For every stage:

- Highlight the newly introduced term.
- Explain its physical meaning.
- Show the corresponding numerical implementation.
- Update the live simulation if the physics is supported.
- Show inventory conservation.
- Provide a baseline comparison where appropriate.

Do not activate a mechanism in the interface unless the underlying solver actually implements it.

A mechanism requiring additional coupled equations must be displayed as such, rather than being artificially inserted into a single equation.

## 5. Primary reference fuel

Use **AGR-1 UCO TRISO** as the primary reference fuel.

Represent the five physical layers:

1. Fuel kernel.
2. Buffer.
3. Inner pyrolytic carbon.
4. Silicon carbide.
5. Outer pyrolytic carbon.

Use verified geometry and material properties.

Where available, use published microstructural evidence to guide SiC illustrations.

Do not claim an ideal 3C-SiC crystal represents the full polycrystalline SiC coating.

AGR-2 may be used for future independent comparisons.

AGR-3/4 may be used for future failed-particle transport studies.

Distinguish gas-cooled AGR experimental conditions from STRATO's downstream FLiBe and cover-gas modelling.

## 6. Future research regimes

Organise all proposed research under three categories.

### Regime A — Intact-barrier transport

Investigate:

- Bulk lattice diffusion.
- Grain-boundary diffusion.
- Segregation.
- Trapping and detrapping.
- Species-specific retention.

### Regime B — Degradation-assisted transport

Investigate:

- Palladium-assisted migration.
- Pd–SiC chemical interactions.
- Irradiation defects.
- Microstructural degradation.
- Defect-assisted diffusion.
- Relevant chemical interactions.

### Regime C — Barrier failure and bypass transport

Investigate:

- Microcracking.
- Connected transport pathways.
- Coating failure.
- Manufacturing defects.
- Fission-gas pressure.
- Failed-particle release.

These are transport regimes, not mutually exclusive physical mechanisms.

Allow mechanisms to interact where scientifically justified.

## 7. Explicit species coverage

Include the following species in the future research catalogue:

- Ag — silver.
- Pd — palladium.
- Cs — caesium.
- Sr — strontium.
- Eu — europium.
- I — iodine.
- Ru — ruthenium.
- Xe — xenon.
- Kr — krypton.

Each species should have a research page or expandable entry explaining:

- Why it matters.
- Relevant TRISO layers.
- Candidate release mechanisms.
- Current evidence.
- Proposed governing-equation extensions.
- Potential Materials Studio calculations.
- Experimental comparison opportunities.
- Research limitations.
- Implementation status.

Do not imply that all species are already simulated.

Do not automatically activate all mechanisms for every species.

## 8. Research-status system

Every mechanism must have one of the following statuses:

**PENDING**

Research is proposed but has not been sufficiently established or implemented.

**RESEARCHED**

Relevant literature, candidate equations and parameter requirements have been documented.

**IMPLEMENTED**

The numerical mechanism exists in STRATO but may not yet be fully verified.

**VERIFIED**

Numerical implementation passes documented conservation, convergence and limiting-case tests.

**VALIDATED**

Predictions have been compared with relevant independent experimental evidence within a clearly defined applicability domain.

Only the final status should be used when the necessary experimental evidence supports it.

Statuses must be assigned per mechanism and, where necessary, per species and material domain.

Do not infer status from the existence of a webpage, paper or placeholder configuration.

Default all newly proposed advanced mechanisms to PENDING.

## 9. Pending mechanisms in the web interface

Create an attractive but scientifically transparent future-research section.

For pending mechanisms:

- Display the mechanism name.
- Show the relevant species.
- Explain the proposed physics.
- Display candidate governing equations as research concepts.
- Identify required parameters.
- Link to supporting literature.
- Include conceptual diagrams where useful.
- Show genuine Materials Studio screenshots when available.
- Clearly identify missing calculations or experiments.
- Disable live-simulation toggles until the numerical mechanism is implemented and sufficiently verified.

Do not fabricate concentration profiles, migration barriers, diffusivities or release fractions.

Do not display hypothetical outputs as real STRATO predictions.

## 10. Materials Studio integration

Create a dedicated atomistic research gallery.

Prioritise genuine BIOVIA Materials Studio screenshots showing:

- AGR-1-relevant SiC crystal structures.
- Representative grain boundaries.
- Ag impurity configurations.
- Vacancies and interstitials.
- Migration pathways.
- Trapping sites.
- Pd–SiC interactions.
- Relevant defect structures.

For each image, provide:

- A scientific caption.
- An accessible explanation.
- Material and species.
- Computational method.
- Simulation conditions.
- Source or provenance.
- Research status.

When a calculation has not been performed, display a Pending placeholder with a description of the planned calculation.

Do not generate imitation Materials Studio screenshots.

Where appropriate, evaluate MatterViz for interactive display of exported atomic structures.

Keep actual screenshots and interactive reconstructions clearly distinguished.

## 11. Build physics brick by brick

Use the following development sequence.

### Brick 1 — Baseline demonstration

Improve the governing-equation presentation and visualisation of existing verified physics.

### Brick 2 — Source and decay terms

Expose existing supported physics progressively.

Implement missing terms only through the established numerical verification workflow.

### Brick 3 — Temperature and layer-dependent transport

Improve the representation of actual TRISO geometry and material properties.

### Brick 4 — Silver trapping and detrapping

Research and reproduce published models before implementation.

Initially mark as Pending.

### Brick 5 — Silver grain-boundary transport

Investigate microstructure-informed diffusivity models.

Initially mark as Pending.

### Brick 6 — Palladium-assisted degradation

Investigate Pd–SiC and Ag–Pd interactions.

Initially mark as Pending.

### Brick 7 — Caesium retention and barrier integrity

Develop species-specific comparisons based on experimental evidence.

Initially mark as Pending.

### Brick 8 — Irradiation and coating degradation

Investigate defect evolution, microcracking and failure.

Initially mark as Pending.

### Brick 9 — Additional species and downstream transport

Extend research to Sr, Eu, I, Ru, Xe and Kr, including relevant FLiBe and cover-gas behaviour.

Initially mark as Pending.

Do not advance a brick to Verified or Validated without the corresponding evidence.

## 12. Scientific accuracy and numerical consistency

Preserve STRATO's existing conservative finite-volume formulation.

Every implemented extension must include:

- Mathematical derivation.
- Dimensional consistency.
- Conservation checks.
- Appropriate interface conditions.
- Limiting-case tests.
- Numerical convergence.
- Parameter provenance.
- Documented assumptions.
- Comparison with relevant literature.

Keep the equation display synchronised with the actual solver.

The website must never imply that a toggle changes the physics if it only changes an illustration.

Distinguish stable isotopes from radioactive isotopes and use appropriate decay constants.

Do not conflate atomistic migration barriers with macroscopic effective diffusivity.

## 13. Visual quality requirements

Aim for a polished, research-grade interface suitable for academic demonstrations and technical presentations.

Use:

- Clear mathematical typography.
- Readable scientific charts.
- Consistent colours for TRISO layers.
- Smooth but restrained interactions.
- Well-labelled axes and units.
- Responsive layouts.
- Accessible controls.
- High-resolution genuine scientific screenshots.
- Clear visual separation between implemented and pending research.

The website should help non-specialists understand how physical mechanisms influence fission-product release without sacrificing scientific accuracy.

Use screenshots of the actual running website to inspect layout quality and fix problems before delivery.

## 14. Required deliverables

Produce:

1. A frontend audit.
2. A GitHub library comparison and selection rationale.
3. An equation-first live-demo implementation.
4. Improved TRISO visualisation and scientific charts.
5. A progressive equation builder.
6. A future research catalogue organised by the three regimes.
7. Explicit entries for all nine species.
8. A research-status system.
9. A Materials Studio screenshot gallery with genuine images or Pending placeholders.
10. A brick-by-brick implementation roadmap.
11. Numerical and frontend test results.
12. Updated scientific documentation and references.

## 15. Execution and quality requirements

Work incrementally.

First inspect and preserve the current website and solver.

Then research the relevant GitHub libraries, choose compatible components and improve the existing live demonstration.

Implement the equation-first interface using verified existing physics.

Add the future research catalogue and mark unverified mechanisms Pending.

Integrate genuine Materials Studio screenshots only when actual source images are available.

Perform visual checks at desktop and mobile widths.

Verify that the equation display, controls and simulation outputs remain consistent.

Preserve the existing deployment workflow.

Do not perform expensive Materials Studio calculations or introduce unverified advanced transport mechanisms as part of this frontend task.

Do not silently alter the validated scientific baseline.

## Final principle

**STRATO should show what has been demonstrated, explain what remains uncertain, and grow one verified physical mechanism at a time.**

The website should make this progression visible: from the baseline governing equation to increasingly sophisticated, evidence-supported models of fission-product transport and release.
