# Technical architecture

## 1 Architecture decision

Three options were evaluated for running STRATO in the browser.

| Option | Verdict | Reason |
|---|---|---|
| A. Client-side WebAssembly | **Adopt** for live v1.0 runs | One synthetic run is about 200 cells and 1000 Backward Euler steps of a tridiagonal solve, which is milliseconds in wasm. The library already compiles for `wasm32-unknown-unknown` (`cargo check --locked --lib --target wasm32-unknown-unknown` passed on 2026-11-10). GitHub Pages is static, so no server is needed. Outram Park and NUS-H2 both run Rust in a Web Worker this way. |
| B. Backend API | **Reject** | Needs hosting, secrets and uptime. GitHub Pages cannot run a server. Adds a second deployment to verify. |
| C. Precomputed verified simulations | **Adopt** for heavy or reference results | The convergence study (`src/postprocessing/comparison.rs`, `convergence_study`) runs many refined meshes and is too slow for an interactive page. Reference cases are also better shown exactly as the native binary produced them. |

Decision: **hybrid A + C**. The browser runs the v1.0 solver in a worker for interactive parameter changes, and loads precomputed native results (with provenance) for the convergence study, golden reference cases and, after Phase 6, any long Phase 2 runs. The Rust solver stays authoritative in both paths. No scientific equation is written in JavaScript.

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
```

Inside `run_v1`:

1. Parse the request into `strato::config::Config` (serde). Call `Config::validate()` (`src/config.rs`, line 79). `Config::load` also rejects unfilled placeholders, but it reads a file, so the wrapper must repeat the placeholder check on the string itself (the check is private in `src/config.rs`; copy its rule, do not change `src/`).
2. Build the mesh with `strato::numerical::finite_volume::build_mesh` and the start profile with `initial_concentration`.
3. For snapshots, step with `backward_euler_step` using exactly the step rule of `finite_volume::run` (`dt = t_end / ceil(t_end / dt_in)`) and the same release accumulation, recording profiles at the requested output times. A wrapper test must show that the time series equals `finite_volume::run` for the same config (bit-identical on the same platform).
4. Return `RunResponse`:

```json
{
  "schema": 1,
  "label": "COMPUTED v1.0",
  "engine": {"strato_version": "1.0.1", "commit": "<stamped>", "abi": 1},
  "inputs": {"...": "echo of the validated config"},
  "mesh": {"cell_centroids_m": [], "cell_faces_m": [], "region_of_cell": []},
  "times_s": [],
  "release_fraction": [],
  "balance_residual_mol": [],
  "profiles": [{"t_s": 0.0, "c_mol_per_m3": []}],
  "analytical": {"times_s": [], "release_fraction": [], "accuracy_floor": 0.0, "valid_from_s": 0.0},
  "messages": []
}
```

Errors never panic across the boundary: return `{"schema":1,"status":"rejected","errors":[...]}`. Use `std::panic::catch_unwind` where needed and set a panic hook that reports to the console.

Build (pattern from Outram Park `crates/dhoby-ghaut/web/dispersion/build.sh` and NUS-H2 `scripts/build_wasm.py`):

```
cargo build --release --manifest-path crates/strato-wasm/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir web/src/solver/pkg <wasm file>
```

Install `wasm-bindgen-cli` at exactly the `wasm-bindgen` version in `crates/strato-wasm/Cargo.lock`, and have the build script check the match. `wasm-pack build --target web` is an acceptable alternative. Stamp the git short hash into the module and into page and worker URLs to defeat GitHub Pages caching (Outram Park uses `__BUILD__`, NUS-H2 uses `__COMMIT__`).

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
- 2D cross-section: a canvas disc with concentric rings per cell; hovering or tabbing to a layer shows name, radii, $D_i$, inventory in the layer and the parameter's provenance.

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
