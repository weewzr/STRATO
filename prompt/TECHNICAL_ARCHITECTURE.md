# Technical architecture

## 1 Architecture decision

Three options were evaluated for running STRATO in the browser.

| Option | Verdict | Reason |
|---|---|---|
| A. Client-side WebAssembly | **Adopt** for live v1.0 runs | One synthetic run is about 200 cells and 1000 Backward Euler steps of a tridiagonal solve, which is milliseconds in wasm. The library already compiles for `wasm32-unknown-unknown` (`cargo check --locked --lib --target wasm32-unknown-unknown` passed on 2026-11-10 and again on 2026-11-24 with rustc 1.100.0). GitHub Pages is static, so no server is needed. Outram Park and NUS-H2 both run Rust in a Web Worker this way. |
| B. Backend API | **Reject** | Needs hosting, secrets and uptime. GitHub Pages cannot run a server. Adds a second deployment to verify. |
| C. Precomputed verified simulations | **Adopt** for heavy or reference results | The convergence study (`src/postprocessing/comparison.rs`, `convergence_study`) runs many refined meshes and is too slow for an interactive page. Reference cases are also better shown exactly as the native binary produced them. |

Decision: **hybrid A + C**. The browser runs the v1.0 solver in a worker for interactive parameter changes, and loads precomputed native results (with provenance) for the convergence study, golden reference cases and, after Phase 6, any long Phase 2 runs. The Rust solver stays authoritative in both paths. No scientific equation is written in JavaScript. NUS-H2 runs a comparable Rust engine as a zero-import wasm module in a worker with JSON requests and replies; the patterns STRATO takes from it (status values, validation, size check, parity tests, commit stamping) are listed in `prompt/RAYSTORAGE_INTEGRATION.md` Section 5.

## 2 Repository layout to add

```
STRATO/
  Cargo.toml                 unchanged package `strato` (verified v1.0 library and CLI)
  src/ tests/ examples/      unchanged (verified)
  crates/strato-wasm/        NEW thin wasm wrapper crate, depends on strato by path
    Cargo.toml               crate-type = ["cdylib", "rlib"], wasm-bindgen pinned
    src/lib.rs               JSON-in / JSON-out entry points, no physics
    tests/parity.rs          native checks of the wrapper against tests/golden/v1.0/
  web/                       NEW Vite + React + TypeScript single-page app
    package.json, package-lock.json, vite.config.ts, tsconfig.json
    index.html
    src/
      main.tsx, App.tsx
      state/store.ts         zustand store
      solver/worker.ts       Web Worker that loads the wasm module
      solver/client.ts       request/response wrapper with handshake and timeouts
      solver/types.ts        TypeScript types mirroring the wrapper JSON
      equation/terms.ts      term table (ids, LaTeX, status) - display only
      equation/EquationExplorer.tsx
      particle/Particle3D.tsx, particle/CrossSection2D.tsx, particle/colormap.ts
      plots/RadialProfile.tsx, plots/ReleaseCurves.tsx, plots/ModelComparison.tsx, plots/Validation.tsx, plots/Convergence.tsx
      panels/Verification.tsx, panels/Provenance.tsx, panels/Downloads.tsx
      design/tokens.css
    public/data/             PRECOMPUTED JSON with provenance headers (generated, not hand edited)
    tests/                   unit and component tests (Vitest)
    e2e/                     browser tests (Playwright)
  scripts/web/
    build_wasm.sh            cargo build + wasm-bindgen CLI, version check against Cargo.lock
    precompute.sh            runs the native CLI and converts outputs to web/public/data/*.json
  .github/workflows/web.yml  NEW: wrapper tests, wasm build, web tests, e2e, Pages artefact
```

Whether to turn the root into a Cargo workspace: prefer **not**. Keep `crates/strato-wasm` as a separate package with its own `Cargo.lock`, built with `--manifest-path crates/strato-wasm/Cargo.toml`, so the verified package, its lock file and CI stay byte for byte as they are. If a workspace becomes necessary, the full regression suite must pass unchanged and the decision goes in `prompt/PROGRESS.md`.

## 3 Wasm wrapper crate (`crates/strato-wasm`)

Rules: no physics, no file I/O, no `SystemTime` or `Instant` (they panic on `wasm32-unknown-unknown`), no `todo!()`. Each call is short.

Public functions, exported with `wasm-bindgen` (exact names are binding for the frontend):

```rust
#[wasm_bindgen] pub fn abi_version() -> u32;                 // starts at 1
#[wasm_bindgen] pub fn build_info() -> String;               // JSON: crate versions, git commit stamped at build time
#[wasm_bindgen] pub fn validate_config(toml_or_json: &str) -> String; // JSON: {ok, errors[], warnings[]}
#[wasm_bindgen] pub fn run_v1(request_json: &str) -> String; // JSON RunResponse
#[wasm_bindgen] pub fn analytical_release(request_json: &str) -> String; // JSON series from Code C
#[wasm_bindgen] pub fn compare_v1(request_json: &str) -> String; // JSON: baseline run, modified run, difference series
```

Owner decision M4 (`prompt/PROGRESS.md`, 2026-11-24): every quantity shown in the browser comes from the Rust solver through this wrapper or from verified precomputed output. The frontend does no scientific arithmetic, including layer inventory shares and difference curves. Hence `run_v1` returns per-layer inventories and shares for every profile, and `compare_v1` returns the difference curve (modified minus baseline on a common time grid). Both are post-processing in Rust that reuses `strato` data (`Mesh::cells[].volume`, `finite_volume::inventory`); they add no new physics. Display-only transforms (colour mapping, log scaling of an axis, unit formatting) stay in TypeScript and are named as such.

Inside `run_v1`:

1. Parse the request into `strato::config::Config` (serde). Call `Config::validate()` (`src/config.rs`, line 79). `Config::load` also rejects unfilled placeholders, but it reads a file, so the wrapper must repeat the placeholder check on the string itself (the check is private in `src/config.rs`; copy its rule, do not change `src/`).
2. Build the mesh with `strato::numerical::finite_volume::build_mesh` and the start profile with `initial_concentration`.
3. For snapshots, step with `backward_euler_step` using exactly the step rule of `finite_volume::run` (`dt = t_end / ceil(t_end / dt_in)`) and the same release accumulation, recording profiles at the requested output times. A wrapper test must show that the time series equals `finite_volume::run` for the same config (bit-identical on the same platform).
4. Return `RunResponse`:

```json
{
  "schema": 1,
  "status": "ok",
  "label": "COMPUTED v1.0",
  "engine": {"strato_version": "1.0.1", "commit": "<stamped>", "abi": 1},
  "inputs": {"...": "echo of the validated config"},
  "mesh": {"cell_centroids_m": [], "cell_faces_m": [], "region_of_cell": []},
  "times_s": [],
  "release_fraction": [],
  "balance_residual_mol": [],
  "profiles": [{"t_s": 0.0, "c_mol_per_m3": [], "inventory_mol": 0.0, "layer_inventory_mol": [], "layer_share": []}],
  "analytical": {"times_s": [], "release_fraction": [], "accuracy_floor": 0.0, "valid_from_s": 0.0},
  "messages": []
}
```

`inventory_mol` is `finite_volume::inventory` of that profile; `layer_inventory_mol[j]` sums `volume * c` over the cells of region `j` in cell order; `layer_share[j]` is `layer_inventory_mol[j] / inventory_mol`. A wrapper test asserts that the layer inventories sum to `inventory_mol` within relative 1e-12.

`compare_v1` takes `{baseline, modified}` (two run requests), runs both and returns `{"schema":1,"status":"ok","label":"COMPUTED v1.0","baseline":RunResponse,"modified":RunResponse,"difference":{"times_s":[],"release_fraction":[]}}`, where the difference is modified minus baseline computed in Rust on the times both runs share (rejected if the time grids differ, never interpolated in the browser).

Errors never panic across the boundary: return `{"schema":1,"status":"rejected","errors":[...]}`. On `wasm32-unknown-unknown` the panic strategy is `abort` (`rustc --print cfg --target wasm32-unknown-unknown` prints `panic="abort"`, checked 2026-11-24 with rustc 1.100.0), so `std::panic::catch_unwind` catches nothing in the browser: a panic, and also a failed allocation, traps the instance (`RuntimeError: unreachable`). The wrapper therefore uses three defences, none of which alone is complete:

1. Validate first. Every `strato` call the wrapper makes returns `Result`, and the wrapper applies every rejection rule listed in the table below before it calls `strato`. The `expect` calls in the library are `finite_volume.rs` lines 88 and 89 (non-empty mesh, non-empty region list), `sturm_liouville.rs` line 262 (finite eigenvalue roots) and `postprocessing/comparison.rs` line 147 (a time series has its initial row; `finite_volume::run` always pushes it). With a configuration that passes the rules below, none of them is expected to fire. `src/main.rs` line 75 has one more, but the CLI is not linked into the wrapper.
2. Cap the work. `Config::validate` sets no upper bound on cell count or step count. A very large cell count can exhaust wasm memory (a trap, not an error), and a very large `end_time_s / dt_s` makes the step loop of step 3 run for an unbounded time. The wrapper defines two named constants, `MAX_TOTAL_CELLS` (sum of `cells` over all regions) and `MAX_STEPS` (`ceil(end_time_s / dt_s)`), and returns `status: "rejected"` with a message naming the limit before it calls `strato`. Their values are set in web Phase 3 from measured wasm run times and memory in the browser and recorded in `prompt/PROGRESS.md`; they are wrapper limits, not physical parameters, and the native CLI keeps no such limit.
3. Contain what remains. The wrapper sets a panic hook that reports the message to the console; the worker enforces a time-out, reports a trap or a time-out to the page, and recreates the instance (Section 4, worker protocol).

So a panic path is not claimed to be unreachable: it is guarded by the rules and the caps, and contained by the worker if a guard is incomplete. `catch_unwind` may still be used in the native wrapper tests.

Facts about `strato` that the wrapper must respect (confirmed against `src/` at `v1.0.1` in Phase 0, 2026-11-24):

| Fact | Consequence for the wrapper |
|---|---|
| `Config` and its parts derive `Deserialize` only, not `Serialize` | the `inputs` echo uses a wrapper-side mirror struct that derives `Serialize`; `src/` is not changed |
| `Config` has a required `output.directory` field (used only by the CLI) | the request schema either carries it or the wrapper fills a fixed dummy value before deserialising; it never writes files |
| Deserialising `Config` (serde) rejects a missing field or a wrong type: every field of `Config` is required (`case_name`, `species`, `temperature_k`, `regions[].name`, `outer_radius_m`, `diffusivity_m2_s`, `cells`, `initial.kernel_concentration_mol_m3`, `time.dt_s`, `end_time_s`, `output_every_steps`, `output.directory`), and `cells` and `output_every_steps` are `usize`, so a negative or fractional value is a parse error | the UI never leaves a field unset, `temperature_k` included, although v1.0 physics does not use it; parse errors are returned as `status: "rejected"` |
| `Config::validate` (`src/config.rs`, lines 79 to 113) rejects, in this order: a region count other than exactly five (line 80); a blank `species` (line 83, `species.trim().is_empty()`); a `temperature_k` that is not finite or not positive (line 86); per region, a non-finite radius or diffusivity, a radius not greater than the previous one (the first must be greater than 0), a diffusivity not greater than 0, or `cells == 0` (lines 90 to 99); a non-finite or non-positive `dt_s` or `end_time_s`, or `output_every_steps == 0` (lines 101 to 107); a non-finite or negative start concentration (line 109). A start concentration of exactly 0 passes `validate` | the wrapper calls `validate` and also mirrors each rule as a UI check; D9 (web Phase 6) later allows 1 to 5 regions |
| `finite_volume::build_mesh` (`src/numerical/finite_volume.rs`, line 74) rejects a mesh of fewer than two cells in total. With five regions of at least one cell each this cannot happen in v1.0, but it can after D9 (one region with one cell) | the wrapper passes the error through as `status: "rejected"`; after D9 the UI requires at least two cells in total |
| `finite_volume::run` (line 187) rejects an initial inventory that is not positive (`initial inventory must be positive`), so a start concentration of 0, which `validate` accepts, fails at run time | the wrapper steps with `backward_euler_step` itself (step 3), so it must apply the same check before stepping: it rejects `kernel_concentration_mol_m3 <= 0` (more exactly, initial inventory `<= 0`) with `status: "rejected"` and the same message; the UI requires `c0 > 0`. This also guarantees that `layer_share` divides by a positive `inventory_mol` at `t = 0`; for later profiles the wrapper returns `layer_share` as `null` with a message if `inventory_mol` is not positive, and never divides by zero |
| `backward_euler_step` (line 164) rejects a state of wrong length or a non-positive or non-finite step, and the tridiagonal solve `solve_diffusion_system` rejects mismatched or empty system arrays (line 134, `invalid system dimensions`, an internal check that the wrapper cannot reach: `backward_euler_step` builds all four arrays at the mesh length, and a mesh from `build_mesh` has at least two cells) and a non-positive or non-finite pivot (line 147) | not reachable with a validated config; passed through as `status: "rejected"` if it ever occurs |
| Code C: `compare_release` (`src/postprocessing/comparison.rs`, lines 55 and 76) rejects a series with no positive output time, and one whose exact series resolves no output time; `ModalRelease::new` and `eigenvalues` (`src/analytical/sturm_liouville.rs`, lines 114, 122, 200, 236, 248, 354, 410) reject invalid layers, a failed eigenvalue bracket, an unresolved eigenvalue cluster and a non-finite modal weight | for `run_v1` the FV result stays valid when only Code C fails: the wrapper returns the run with `analytical` set to `null` and the Code C message in `messages`; `analytical_release` returns `status: "rejected"` |
| Wrapper limits (this section, defence 2): `MAX_TOTAL_CELLS` and `MAX_STEPS` | rejected with `status: "rejected"` before any `strato` call |
| `unfilled_placeholders` is private | the wrapper copies its rule for strings (Section 3, step 1) |
| `Cell.material` is the zero-based region index of each cell, `Cell.volume` the exact shell volume | `region_of_cell` and `layer_inventory_mol` are built from them |
| `finite_volume::run` takes `steps = ceil(end_time_s / dt_s)`, `dt = end_time_s / steps`, records rows at `step % output_every_steps == 0` and at the last step, accumulates the release with Kahan compensation and reports `balance_residual = inventory + released - initial` | the snapshot loop in step 3 copies this exactly, including the Kahan update order |
| Code C: `postprocessing::comparison::compare_release(cfg, rows)` builds `ModalRelease::new(layers, 45 / t_min, MAX_MODES)` with `MAX_MODES = 20_000` and returns `accuracy_floor` and `min_valid_time_s`; `sturm_liouville::exact_release(layers, times, max_modes)` returns `None` before the validity time | `analytical.accuracy_floor` and `analytical.valid_from_s` come from these; times before `valid_from_s` are omitted, never filled |
| `postprocessing::plots` writes SVG files through `plotters` | never called from the wrapper |
| No `Instant`, `SystemTime`, `todo!` or `unimplemented!` in `src/` | the library is safe to call from wasm; the CLI (`src/main.rs`) and `examples/verify.rs` are not linked into the wrapper |

The rejection rows of this table (serde parsing, `Config::validate`, `build_mesh`, `finite_volume::run`, `backward_euler_step` with the tridiagonal solve, and Code C), together with the `expect` calls of defence 1 and the placeholder rule of `Config::load` (`src/config.rs` line 66, which the wrapper copies, Section 3, step 1), cover every `bail!` and `expect(` found by a search of `src/` at `v1.0.1` for `bail!`, `expect(`, `unwrap(`, `panic!` and `assert!` (2026-11-24; the `unwrap` and `assert!` hits are all in `#[cfg(test)]` modules), except two that the wrapper never calls: `convergence_study` (`src/postprocessing/comparison.rs` line 188, shown only as precomputed output, option C of Section 1) and `src/main.rs` line 75 (CLI). The wrapper limits are the wrapper's own rejections, not `strato` ones. Index and arithmetic panics (slicing, out-of-range indexing, integer overflow) were not audited separately; they are guarded by the rules and caps and contained by the worker (defence 3).

Build (pattern from Outram Park `crates/dhoby-ghaut/web/dispersion/build.sh` and NUS-H2 `scripts/build_wasm.py`):

```
cargo build --release --manifest-path crates/strato-wasm/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir web/src/solver/pkg <wasm file>
```

Install `wasm-bindgen-cli` at exactly the `wasm-bindgen` version in `crates/strato-wasm/Cargo.lock`, and have the build script check the match. `wasm-pack build --target web` is an acceptable alternative. Stamp the git short hash into the module and into page and worker URLs to defeat GitHub Pages caching (Outram Park uses `__BUILD__` in the build scripts of its `dem`, `foam_studio`, `helical_sg` and `triso_atops` demos, not in the dispersion build; NUS-H2 uses `__COMMIT__` as a `?v=` query and in `build.json`). The installed CLI on the development machine is `wasm-bindgen 0.2.130` (2026-11-24); pin the crate to the same version.

## 4 Frontend

| Concern | Choice | Notes |
|---|---|---|
| Framework and build | React 19 (stay within the range `@react-three/fiber` accepts), Vite, TypeScript strict | static build to `web/dist` with `base: '/STRATO/'` |
| 3D particle | `three` + `@react-three/fiber` | `@react-three/drei` only if a phase needs it; record the reason |
| Plots | `plotly.js` partial bundle (cartesian) via a thin React wrapper | avoid the full bundle |
| Maths | KaTeX, synchronous render, `\htmlClass` / `\textcolor` for term highlighting | MathJax rejected |
| State | `zustand` | playback clock kept outside React re-render paths |
| UI primitives | hand-written CSS tokens and native controls; shadcn components optional (switch, slider, tabs, tooltip only) | must pass keyboard and screen-reader checks |
| Animation | CSS transitions; `motion` optional | respect `prefers-reduced-motion` |
| Data fetching | plain `fetch` of static JSON | TanStack Query rejected |

### Store shape (`web/src/state/store.ts`)

```ts
type Mechanism = 'decay' | 'source' | 'arrhenius';
interface StratoState {
  config: V1Config;                 // mirrors inputs/synthetic/synthetic_five_layer.toml
  baselineConfig: V1Config;
  mechanisms: Record<Mechanism, { enabled: boolean; available: boolean }>; // available=false until Phase 6
  run: RunResponse | null;          // current (modified) result
  baselineRun: RunResponse | null;
  status: 'idle' | 'running' | 'ok' | 'rejected' | 'error';
  playback: { tIndex: number; playing: boolean; speed: number };
  view: { showComparison: boolean; showDifference: boolean; showValidation: boolean; logColour: boolean; selectedLayer: number | null };
}
```

### Worker protocol (`web/src/solver/worker.ts`, `client.ts`)

- Hello handshake: the worker posts `{kind:'hello', abi, build}` once it is listening. The client queues requests until it arrives (a message sent before a module worker's `onmessage` exists is lost; Outram Park `crates/bugis/src/web_demo/link.rs` documents this).
- Request `{id, kind:'run_v1'|'analytical'|'validate', payload}`; response `{id, ok, body | error}`.
- Timeout per request (default 10 s), with a visible error card and a retry button. A trapped wasm instance is recreated.
- Debounce slider changes (about 150 ms) and cancel stale requests by id.

### Equation explorer term model (`web/src/equation/terms.ts`)

Display only; no evaluation.

| Term id | LaTeX | Toggle | Status before Phase 6 |
|---|---|---|---|
| `lhs` | `\frac{\partial c}{\partial t}` | always | IMPLEMENTED |
| `diffusion` | `\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_i \frac{\partial c}{\partial r}\right)` | always | IMPLEMENTED |
| `diffusion_T` | replaces `D_i` with `D_i(T(t))` | `arrhenius` | DERIVED NOT IMPLEMENTED |
| `decay` | `-\lambda c` | `decay` | DERIVED NOT IMPLEMENTED |
| `source` | `+S(r,t)` | `source` | DERIVED NOT IMPLEMENTED |

Each term links to its section in `docs/mathematics/` (v1.0.0 Full Derivation for the baseline; `STRATO_Phase2_Derivations.md` for (P21.1), (P21.8), (P22.1)).

### Particle rendering

- Radii come from the active config (`regions[].outer_radius_m`). Never hard-code TRISO dimensions.
- Concentration colour: per-cell value from the selected profile mapped through cividis (default) or viridis, linear or logarithmic scale, with a legend that states units (mol m^-3) and the maximum.
- 3D: nested spheres with a cutaway wedge; shells drawn as thin translucent surfaces plus a radial texture of cell values on the cut faces.
- 2D cross-section: a canvas disc with concentric rings per cell; hovering or tabbing to a layer shows name, radii, $D_i$, inventory in the layer (`layer_inventory_mol` and `layer_share` from `RunResponse`, never summed in TypeScript) and the parameter's provenance.

## 5 Precomputed data (`web/public/data/`)

Produced only by `scripts/web/precompute.sh` from the native release binary. Each JSON file starts with

```json
{"label":"PRECOMPUTED","generator":"strato <version>","commit":"<sha>","input_file":"inputs/synthetic/synthetic_five_layer.toml","input_sha256":"...","command":"...","created_utc":"...","data":{}}
```

Initial files: `synthetic_run.json` (time series and final profile), `comparison_fv_vs_analytical.json`, `convergence_study.json`, `golden_v1.json` (copy of `tests/golden/v1.0/` values for the parity check). A CI step regenerates them and fails if they differ from the committed copies.

## 6 Performance budget

| Item | Budget |
|---|---|
| wasm module (gzip) | under 1 MB; NUS-H2 enforces a size limit in `scripts/check_wasm.py`, do the same |
| first interactive solve after load | under 1 s on a mid-range laptop |
| single synthetic `run_v1` in the worker | under 200 ms |
| playback | 60 fps target, 30 fps minimum, no React re-render per frame for the 3D scene |
| initial JavaScript (gzip) | under 1.5 MB, with Plotly and three.js loaded lazily per section if needed |

Measure with the e2e suite and record the numbers in `prompt/PROGRESS.md`. Do not claim a number you did not measure.

## 7 CI

Keep `.github/workflows/ci.yml` and `docs-pdf.yml` unchanged. Add `.github/workflows/web.yml`:

1. `cargo test --manifest-path crates/strato-wasm/Cargo.toml` (native parity tests).
2. Build the wasm module, check exports and size.
3. `npm ci`, `npm run lint`, `npm run typecheck`, `npm test` in `web/`.
4. `npm run build`, then `npx playwright test` against `vite preview` (viewports, keyboard, reduced motion, console errors, failed requests, wasm parity, self-test).
5. On the work branch: upload the build as an artefact. Deployment to Pages follows `prompt/COMMIT_AND_DEPLOYMENT.md`.

## 8 Compatibility requirements

- Rust: the toolchain that builds STRATO today (CI records the version). Target `wasm32-unknown-unknown`.
- Node LTS; commit the lock file; run `npm audit` before deployment.
- Browsers: current Chrome, Firefox, Safari and Edge. WebGL 2 for the 3D view, with the 2D cross-section as fallback.
- GitHub Pages base path `/STRATO/`.
