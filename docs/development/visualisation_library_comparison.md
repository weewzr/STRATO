# Visualisation library comparison and selection

Date: 2026-11-24. Deliverable 2 of the owner's equation-first brief (`prompt/EQUATION_FIRST_REQUEST.md`, Section 2; owner decision M7). It evaluates every repository the brief names, plus three.js (required by React Three Fiber) and uPlot (a smaller chart library, as a check on the Plotly choice), and selects the smallest coherent set for the STRATO web demo.

The existing frontend is Vite 8, React 19.3.1, TypeScript 6 and zustand (`docs/development/frontend_audit.md`). The repository is AGPL-3.0-only.

## 1 Method and sources

Facts below come only from pages and machine-readable records opened on 2026-11-24 (between 12:30 and 13:00 UTC):

- GitHub REST API for each repository: `https://api.github.com/repos/<owner>/<repo>` (licence, archived flag, last push, primary language), `.../releases?per_page=5` (latest releases and dates), `.../commits?since=2026-08-24T00:00:00Z&per_page=100` (commits in the last three months; the request returns at most 100, so "100" means "at least 100"), `.../readme`.
- npm registry record of each package: `https://registry.npmjs.org/<package>` (latest version, its publication time, licence, peer dependencies, dependencies, `dist.unpackedSize`).
- Bundle size: where a single prebuilt browser file exists, it was downloaded from jsDelivr at the exact latest version and its raw and gzip size (level 9, Python `gzip`) were measured. Where no single file exists, the npm `unpackedSize` is given and marked as such; it is the size of the whole package on disk, not what a tree-shaken build ships.
- Documentation pages opened are named in each entry with their URL.

Anything not checked on an opened page is marked **not checked**. Licence compatibility is assessed from the licence named by GitHub, npm and, where they disagree, the licence file. MIT and BSD-3-Clause are permissive licences whose code may be combined into an AGPL-3.0-only work, provided their copyright and licence notices are kept; this is the general compatibility position of those licences, not legal advice.

## 2 Summary table

| Library | Written for | Latest release (date) | Commits since 2026-08-24 | Licence | Size | Verdict |
|---|---|---|---|---|---|---|
| KaTeX | framework-free (TypeScript) | v0.19.1 (2026-10-23) | 33 | MIT | `katex.min.js` 273,081 B, 75,947 B gzip; `katex.min.css` 24,793 B, 3,512 B gzip (measured) | **Select** (Phase 2) |
| plotly.js | framework-free (JavaScript) | v4.3.0 (2026-11-10) | at least 100 | MIT | `plotly-basic.min.js` 1,204,205 B, 400,098 B gzip; `plotly-cartesian.min.js` 1,516,234 B, 502,657 B gzip (measured) | **Select**, basic partial bundle, lazy (Phase 5) |
| Recharts | React | v3.11.1 (2026-11-23) | at least 100 | MIT | UMD `Recharts.js` 686,450 B, 180,788 B gzip (measured) | Exclude (duplicates Plotly) |
| React Three Fiber | React | v9.8.2 (2026-10-15; v10 alphas exist) | 44 | MIT | npm unpacked 2,399,634 B (metadata) | **Select** for the optional 3D view, lazy (Phase 4) |
| three.js | framework-free | r187 (2026-10-30), npm `three` 0.187.0 | at least 100 | MIT | `three.module.min.js` 392,177 B, 89,384 B gzip (measured) | **Select** with React Three Fiber |
| drei | React (on React Three Fiber) | v10.7.9 (2026-09-25; v11 alphas exist) | 7 | MIT | npm unpacked 1,753,692 B (metadata) | Exclude |
| MatterViz | Svelte 5 (custom elements for other frameworks) | v0.10.0 (2026-11-21) | 72 | MIT | npm unpacked 25,016,115 B (metadata); element bundle not measured | **Candidate for later**, not installed |
| 3Dmol.js | framework-free (JavaScript, TypeScript declarations) | 3.0.0 (2026-11-22) | 38 | BSD-3-Clause | `3Dmol-min.js` 580,276 B, 168,037 B gzip (measured) | Exclude now; second atomistic candidate |
| NGL | framework-free (TypeScript) | npm 2.5.0 (2026-09-03); newest GitHub release v0.9.3 (2016-10-15) | 2 | MIT | `ngl.js` 1,292,474 B, 356,820 B gzip (measured) | Exclude |
| vtk.js | framework-free (JavaScript ES modules) | v38.1.0 (2026-11-18) | at least 100 | BSD-3-Clause | npm unpacked 14,959,634 B (metadata); tree-shaken size not measured | Exclude |
| Kitware Glance | a Vue 2 application, not a library | v4.25.2 (2024-09-17) | 0 (last push 2024-09-17) | BSD-3-Clause | no npm package; not measured | Exclude |
| uPlot (extra) | framework-free | 1.6.32 (2025-03-14) | at least 100 | MIT | `uPlot.iife.min.js` 51,081 B, 21,991 B gzip (measured) | Exclude (fallback only) |

## 3 Entries

### 3.1 KaTeX (https://github.com/KaTeX/KaTeX)

- **What it does**: "Fast math typesetting for the web" (repository description). Renders TeX to HTML and MathML synchronously.
- **Compatibility**: framework-free; called from a React component (`katex.render` or `renderToString`). TypeScript declarations in the package (`types/katex.d.ts`). One runtime dependency (`commander`, for the command-line tool).
- **Release and maintenance**: v0.19.1 published 2026-10-23 (GitHub release and npm); six releases between 2026-09-23 and 2026-10-23 (v0.18.8 to v0.19.1, GitHub releases list); 33 commits since 2026-08-24; last push 2026-11-23; not archived.
- **Licence**: MIT; compatible with AGPL-3.0-only (keep the notice).
- **Size**: 75,947 B gzip for the script and 3,512 B gzip for the stylesheet (measured, jsDelivr `katex@0.19.1/dist/`); the fonts are extra and must be self-hosted (owner decision M6 allows the KaTeX fonts only if their licence is OFL; **not checked** here, to be checked in Phase 2 before shipping).
- **Mobile**: no statement found; output is HTML text, so it scales with the page. Long equations need a scroll container (design specification Section 2.1).
- **Accessibility**: the options page (https://katex.org/docs/options) says the default output `htmlAndMathml` "Outputs HTML for visual rendering and includes MathML for accessibility". The `trust` option defaults to `false`; a narrow `trust` function is needed for `\htmlClass`, which the builder uses to highlight the new term.
- **Documentation**: https://katex.org (options page opened).
- **Integration effort**: low.
- **Adds something the set lacks**: yes, typeset mathematics; nothing else in the set does this. **Selected.**

### 3.2 plotly.js (https://github.com/plotly/plotly.js)

- **What it does**: "Open-source JavaScript charting library behind Plotly and Dash"; scientific 2D plots with log axes, error bars, hover, zoom and image export.
- **Compatibility**: framework-free; used from React through a thin wrapper component (the package's own `plotly.js` record declares no peer dependencies). Types ship with the full package (`./lib/index.d.ts`); the partial dist packages carry none.
- **Release and maintenance**: v4.3.0 published 2026-11-10; v4.2.0 2026-10-29, v4.1.2 2026-10-05; at least 100 commits since 2026-08-24; last push 2026-11-24.
- **Licence**: MIT (full and partial packages); compatible.
- **Size**: the full package is 98,684,236 B unpacked (npm metadata). Partial bundles at 4.3.0: `plotly.js-basic-dist-min` 400,098 B gzip, `plotly.js-cartesian-dist-min` 502,657 B gzip (measured).
- **Mobile**: https://plotly.com/javascript/responsive-fluid-layout/ says that with `responsive: true` "your figures will be automatically resized when the browser window size changes", "especially useful ... on mobile devices". Touch behaviour **not checked**.
- **Accessibility**: no accessibility statement found in the README; **not checked** further. The page must therefore give every plot a text summary and a data table or download (design specification Section 9).
- **Documentation**: https://plotly.com/javascript/ (one page opened).
- **Integration effort**: moderate (wrapper, lazy loading, theming through layout objects).
- **Adds something**: yes; profile, release, comparison, convergence (log-log) and measurement plots with error bars. The brief prioritises it. **Selected**, as the **basic** partial bundle loaded only on the views with plots; the cartesian bundle (about 100 kB gzip more) only if a phase records a need for heat maps or contours. Whether the basic bundle includes error bars and log axes is **not checked** and is the first check of Phase 5; if not, the cartesian bundle is used.

### 3.3 Recharts (https://github.com/recharts/recharts)

- **What it does**: "Redefined chart library built with React and D3"; SVG charts as React components.
- **Compatibility**: React; peer dependencies `react`, `react-dom` and `react-is` `^16.8.0 || ^17.0.0 || ^18.0.0 || ^19.0.0` (npm 3.11.1), so React 19.3.1 fits. Types included. Runtime dependencies include `@reduxjs/toolkit`, `react-redux`, `immer` and `victory-vendor`.
- **Release and maintenance**: v3.11.1 published 2026-11-23; at least 100 commits since 2026-08-24; very active.
- **Licence**: MIT; compatible.
- **Size**: UMD file 180,788 B gzip (measured); npm unpacked 8,912,050 B.
- **Mobile**: **not checked**.
- **Accessibility**: GitHub discussion #4484 (https://github.com/recharts/recharts/discussions/4484) describes an `accessibilityLayer` with arrow-key navigation between data points, treated there as off by default; whether version 3 changed the default was **not checked**.
- **Documentation**: https://recharts.github.io (not opened).
- **Integration effort**: low for React.
- **Adds something**: no. It duplicates Plotly, which the brief prioritises and which also covers log-log convergence plots and error bars. Installing both would add a second chart style and about 180 kB gzip. **Excluded.**

### 3.4 React Three Fiber (https://github.com/pmndrs/react-three-fiber) and three.js (https://github.com/mrdoob/three.js)

- **What they do**: three.js is a WebGL 3D library ("JavaScript 3D Library"); React Three Fiber is "A React renderer for Three.js", so a 3D scene is written as React components.
- **Compatibility**: React Three Fiber 9.8.2 declares peers `react` and `react-dom` `>=19 <19.4` and `three >=0.156` (npm). The scaffold's React 19.3.1 is inside that range; React 19.4 would not be until a fiber release allows it. The installation page (https://r3f.docs.pmnd.rs/getting-started/installation) says "@react-three/fiber@9 pairs with react@19" and that Vite "will also work out of the box". The fiber package also lists optional Expo and React Native peers, which a web build does not install.
- **Release and maintenance**: fiber v9.8.2 released 2026-10-15 (v10.0.0-alpha.6 on 2026-10-17 is a prerelease); 44 commits since 2026-08-24; last push 2026-10-21. three.js r187 released 2026-10-30 (npm `three` 0.187.0); about one release a month (r185 2026-07-01, r186 2026-09-24); at least 100 commits since 2026-08-24.
- **Licence**: both MIT; compatible.
- **Size**: `three.module.min.js` 89,384 B gzip (measured; a tree-shaken build can be smaller or larger depending on imports, **not measured**); fiber 2,399,634 B unpacked (metadata).
- **Mobile**: WebGL 2 is required for the 3D view (`prompt/TECHNICAL_ARCHITECTURE.md` Section 8); the 2D cross-section is the default and the fallback. Touch controls come from `three/addons` orbit controls; **not checked** on a device.
- **Accessibility**: the fiber README points to `@react-three/a11y` ("real a11y for your scene"); not evaluated. The design keeps all information in the DOM (layer list, inspector, text summary) so the canvas is never the only carrier.
- **Documentation**: https://r3f.docs.pmnd.rs (installation page opened); https://threejs.org (not opened).
- **Integration effort**: moderate; contained in one lazily loaded component.
- **Adds something**: yes, the interactive multilayer cutaway that the brief asks for "only if the existing architecture supports it without unnecessary complexity". It does: React and Vite are already the stack and the versions fit. **Selected**, loaded only when the reader switches to the 3D view; the 2D cross-section (hand-written SVG or canvas, no library) stays the default under the equation.

### 3.5 drei (https://github.com/pmndrs/drei)

- **What it does**: "useful helpers for react-three-fiber" (cameras, controls, text, loaders and more).
- **Compatibility**: peers `react ^19`, `react-dom ^19`, `three >=0.159`, `@react-three/fiber ^9.0.0` (npm 10.7.9). Runtime dependencies include `hls.js`, `@mediapipe/tasks-vision`, `troika-three-text`, `three-mesh-bvh`, `camera-controls`, `maath`, `detect-gpu` and others.
- **Release and maintenance**: v10.7.9 released 2026-09-25 (v11 alphas since 2026-01-14, latest v11.0.0-alpha.7 on 2026-09-05, npm registry times); 7 commits since 2026-08-24; last push 2026-11-17.
- **Licence**: MIT; compatible.
- **Size**: 1,753,692 B unpacked (metadata); a tree-shaken import was **not measured**.
- **Mobile, accessibility**: **not checked**.
- **Documentation**: https://docs.pmnd.rs/drei (not opened).
- **Integration effort**: low per helper, but a large dependency tree.
- **Adds something**: not for STRATO's particle (nested spheres, a cutaway, orbit controls from `three/addons`). **Excluded**; reconsider only if a phase records a concrete need (the package rule "drei unless a phase records a need" stands).

### 3.6 MatterViz (https://github.com/janosh/matterviz)

- **What it does**: "Interactive browser visualizations for materials science: crystal structures/molecules, trajectories, convex hulls, phase diagrams, Fermi surfaces, bands+DOS, Brillouin zones". The README lists structure formats "CIF, POSCAR, (ext)XYZ, pymatgen/ASE/OPTIMADE JSON, PDB" and trajectory formats including extXYZ, ASE `.traj`, HDF5, LAMMPS and XDATCAR, and says it "builds on three.js, threlte, d3, Svelte and Vite".
- **Compatibility**: written for Svelte (npm peer dependency `svelte ^5.48.0`, so Svelte 5). For React, the guide https://matterviz.janosh.dev/how-to/use-without-svelte offers prebuilt custom elements (`mv-structure`, `mv-trajectory` and others) imported from `matterviz/elements`; with React 19, callbacks named `on_*` must be assigned through a ref, and the `mv-*` tags must be declared for TSX. Self-hosting requires copying the whole `dist/elements/` folder because chunks and workers load relative to the entry file. It depends on `three ^0.187.0`, the same three.js line as the selected set.
- **Release and maintenance**: v0.10.0 released 2026-11-21; v0.9.0 2026-11-10, v0.8.0 2026-10-04; 72 commits since 2026-08-24; last push 2026-11-22. Pre-1.0: the README says the changelog covers "breaking changes and migration notes for each release".
- **Licence**: MIT; compatible. Licences of its dependencies (for example `h5wasm`, `@spglib/moyo-wasm`, `@threlte/core`) **not checked**.
- **Size**: 25,016,115 B unpacked (metadata); the custom-element bundle was **not measured**.
- **Mobile, accessibility**: no statement found on the pages opened; **not checked**.
- **Documentation**: https://matterviz.janosh.dev (one guide opened).
- **Integration effort**: moderate (custom elements in React, a second component runtime, self-hosted chunk folder).
- **Adds something**: yes, but only once there is a structure to show: interactive display of an exported SiC, grain-boundary or defect structure for the Materials Studio gallery (brief Section 10). It reads the crystal formats a Materials Studio export can be converted to (CIF, POSCAR, XYZ), which suits solid-state SiC better than the biomolecule-oriented viewers. **The candidate atomistic viewer for later. Not installed now** (rule E6): the repository holds no structure file, and the gallery ships with Pending placeholders until the owner supplies genuine screenshots or structure files with method and conditions. When one exists, check the element bundle size, the custom-element integration and the dependency licences first, and label every view as an interactive reconstruction, never as a Materials Studio screenshot.

### 3.7 3Dmol.js (https://github.com/3dmol/3Dmol.js)

- **What it does**: "WebGL accelerated JavaScript molecular graphics library". The README lists "pdb, sdf, mol2, xyz, cif, cdjson, mmtf, prmtop, gro, pqr, cube, and vasp formats".
- **Compatibility**: framework-free; TypeScript declarations under `build/types` (README). Usable from React through a ref to a container element.
- **Release and maintenance**: 3.0.0 released 2026-11-22 (2.5.5 on 2026-05-22); 38 commits since 2026-08-24; last push 2026-11-23.
- **Licence**: GitHub reports `NOASSERTION`; npm says BSD-3-Clause; the licence file (https://github.com/3dmol/3Dmol.js/blob/master/LICENSE) says "3Dmol.js incorporates code from GLmol, Three.js, and jQuery and is licensed under a BSD-3-Clause license", with GLmol dual MIT or LGPL3 and three.js and jQuery under MIT-style notices. Compatible.
- **Size**: `3Dmol-min.js` 168,037 B gzip (measured).
- **Mobile, accessibility**: not mentioned on the README; **not checked**.
- **Documentation**: https://3dmol.org (not opened).
- **Integration effort**: low to moderate.
- **Adds something**: an alternative atomistic viewer with a single file and no framework. **Excluded now** for the same reason as MatterViz (nothing to show); second candidate if MatterViz's custom-element route proves awkward.

### 3.8 NGL (https://github.com/nglviewer/ngl)

- **What it does**: "WebGL protein viewer"; the README lists structure formats mmCIF, PDB, PQR, GRO, SDF, MOL2, MMTF, density volumes and MD trajectories; it credits Mol* for CIF parsing.
- **Compatibility**: framework-free TypeScript; npm 2.5.0 depends on `three ^0.158.0` (an older three.js line than the selected 0.187) and `molstar ^4.1.0`.
- **Release and maintenance**: npm 2.5.0 published 2026-09-03; the newest GitHub release is v0.9.3 of 2016-10-15 (releases are no longer cut on GitHub); 2 commits since 2026-08-24, last push 2026-09-03. No maintenance statement on the README.
- **Licence**: MIT; compatible. Licence of `molstar` **not checked**.
- **Size**: `dist/ngl.js` 356,820 B gzip (measured).
- **Mobile, accessibility**: **not checked**.
- **Documentation**: http://nglviewer.org/ngl/ (not opened).
- **Adds something**: no; biomolecular focus, a second three.js version and the largest atomistic bundle measured. **Excluded.**

### 3.9 vtk.js (https://github.com/Kitware/vtk-js)

- **What it does**: "Visualization Toolkit for the Web"; scientific volume, mesh and scalar-field rendering. The documentation (https://kitware.github.io/vtk-js/docs/) calls it "an ES6 class library which can be integrated into any web application" that "leverages WebGL (+WebGPU soon)"; its sidebar lists guides for React, Vue and plain use (not opened).
- **Compatibility**: framework-free ES modules (`type: module`), types included; npm peers `wslink >=2.0.0` and `autoprefixer ^10.4.7`.
- **Release and maintenance**: v38.1.0 released 2026-11-18; several releases a week in November 2026; at least 100 commits since 2026-08-24.
- **Licence**: BSD-3-Clause; compatible.
- **Size**: 14,959,634 B unpacked (metadata); a tree-shaken import was **not measured**.
- **Mobile, accessibility**: **not checked**.
- **Integration effort**: high for STRATO's needs.
- **Adds something**: nothing STRATO needs: a one-dimensional radial solution on five spherical shells is drawn by the 2D cross-section and three.js. It would matter only for 3D field data (for example a resolved microstructure), which STRATO does not compute. **Excluded.**

### 3.10 Kitware Glance (https://github.com/Kitware/glance)

- **What it does**: the README calls it "an open-source web application developed at Kitware for visualizing volumetric images" built on VTK, which "can serve as a foundation for building custom web-based visualization applications involving ITK.js and VTK.js".
- **Compatibility**: an application, not a component library; its `package.json` (https://raw.githubusercontent.com/Kitware/glance/master/package.json) depends on `vue` 2.6.14, `vuetify` 2.6.9 and `vuex` 3.6.2. It does not fit a React 19 site except as a separate page.
- **Release and maintenance**: v4.25.2 of 2024-09-17 is the newest release and the last push; no commits since 2026-08-24. No maintenance statement on the README.
- **Licence**: BSD-3-Clause; compatible.
- **Size**: no npm package; **not measured**.
- **Adds something**: no. **Excluded.**

### 3.11 uPlot (https://github.com/leeoniya/uPlot), additional candidate

- **What it does**: "A small, fast chart for time series, lines, areas, ohlc & bars".
- **Compatibility**: framework-free, types included; the README links third-party React, Vue and Svelte wrappers.
- **Release and maintenance**: newest release 1.6.32 of 2025-03-14 (npm the same), but at least 100 commits since 2026-08-24 and a push on 2026-11-23.
- **Licence**: MIT; compatible.
- **Size**: 21,991 B gzip (measured), about one twentieth of the Plotly basic bundle.
- **Mobile**: the README notes "No built-in drag scrolling/panning", which can be added through plugins.
- **Accessibility**: **not checked**.
- **Adds something**: no capability beyond Plotly; it is lighter. **Excluded**, kept as the fallback if the Plotly basic bundle cannot meet the performance budget of `prompt/TECHNICAL_ARCHITECTURE.md` Section 6 even when loaded lazily.

## 4 Selection

The smallest coherent set:

| Need | Selected | Why | Loaded |
|---|---|---|---|
| Governing equation and discrete updates (block A) | KaTeX | the only maths renderer in the set; synchronous rendering for the staged builder; MathML output for screen readers by default | with the Explore page |
| Scientific plots (block B, Compare, Validate, Verify) | plotly.js, basic partial bundle | brief priority; log axes, error bars, hover and export without a second library; partial bundle avoids the full package | lazily, on first plot |
| Optional 3D cutaway of the particle | three.js with React Three Fiber | fits React 19.3.1 and Vite; contained in one component; the 2D cross-section stays the default and fallback | lazily, when the 3D view is chosen |
| 2D cross-section, layer boundaries, inspector | no library (SVG or canvas, as in the scaffold) | keeps the view under the equation light and fully accessible | with the Explore page |
| Atomistic structures for the gallery | none now; MatterViz is the candidate for later | no structure file exists (rule E6) | not installed |

Excluded and why: Recharts and uPlot duplicate Plotly; drei adds a large dependency tree for helpers the particle does not need; 3Dmol.js and NGL are atomistic viewers with nothing to show yet (3Dmol.js is the second candidate, NGL is biomolecular and pins an older three.js); vtk.js addresses 3D field data STRATO does not produce; Glance is a Vue 2 application with no release since 2024.

Measured worst case of the selected set before tree shaking, for orientation only: KaTeX 79,459 B gzip (script and stylesheet), three.js 89,384 B gzip, Plotly basic 400,098 B gzip, plus the scaffold's own 82,740 B gzip JavaScript (`docs/development/frontend_audit.md`). Initial JavaScript without the lazy parts stays well under the 1.5 MB gzip budget; the real figures are measured on the production build in Phase 8.

## 5 Consequences for the package

- `prompt/TECHNICAL_ARCHITECTURE.md` Section 4 and `prompt/GITHUB_REFERENCES.md` record this selection; a new visualisation library needs a row in this document first.
- The KaTeX font licence and the contents of the Plotly basic bundle are the two open checks (Phases 2 and 5).
- When a genuine structure file exists, MatterViz is evaluated again (element bundle size, dependency licences, React integration) before it is installed, and its views are labelled as interactive reconstructions.
