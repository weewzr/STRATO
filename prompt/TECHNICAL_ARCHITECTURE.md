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
      equation/terms.ts      term and stage table (ids, LaTeX, stage, mechanism id) - display only, no status
      equation/EquationBuilder.tsx   staged builder (block A of the design specification)
      research/              catalogue views (Research section and block E), read from generated JSON;
                             research/status.ts is the only module that names the five status words
      particle/Particle3D.tsx, particle/CrossSection2D.tsx, particle/colormap.ts
      plots/RadialProfile.tsx, plots/ReleaseCurves.tsx, plots/ModelComparison.tsx, plots/Validation.tsx, plots/Convergence.tsx
      panels/Verification.tsx, panels/Provenance.tsx, panels/Downloads.tsx
      generated/             JSON written by strato-webtools (case data, research catalogue)
      design/tokens.css
    public/data/             PRECOMPUTED JSON with provenance headers (generated, not hand edited)
    tests/                   unit and component tests (Vitest)
    e2e/                     browser tests (Playwright)
  data/research/             NEW research catalogue tables (Section 5.1): mechanisms.csv, species.csv, references.csv, gallery.csv
  docs/research/             NEW one Markdown file per catalogue entry (Section 5.1)
  crates/strato-webtools/    one Rust command-line tool of the tools workspace (stage R1 of the Rust migration; it replaced scripts/web/*.py and *.sh)
    src/                     subcommands: build-wasm (cargo build + wasm-bindgen CLI, version check against Cargo.lock, export and size check),
                             precomputed (runs the native CLI and converts outputs to web/public/data/*.json), case, catalogue, colormaps,
                             geometry, figures (the generators of the web data files, Section 5.1), check-catalogue (fails the build, Section 5.2)
    src/solver.rs            the one place that names the solver command lines and output files
    tests/                   the catalogue check against its fixture folders, the entry-text conversion, and the byte-for-byte comparison with the committed data files
  .github/workflows/web.yml  NEW: wrapper tests, wasm build, catalogue check, web tests, e2e, Pages artefact
```

Whether to turn the root into a Cargo workspace: prefer **not**. Keep `crates/strato-wasm` as a separate package with its own `Cargo.lock`, built with `--manifest-path crates/strato-wasm/Cargo.toml`, so the verified package, its lock file and CI stay byte for byte as they are. If a workspace becomes necessary, the full regression suite must pass unchanged and the decision goes in `prompt/PROGRESS.md`.

## 3 Wasm wrapper crate (`crates/strato-wasm`)

Rules: no physics, no file I/O, no `SystemTime` or `Instant` (they panic on `wasm32-unknown-unknown`), no `todo!()`. Each call is short.

Public functions, exported with `wasm-bindgen` (exact names are binding for the frontend):

```rust
#[wasm_bindgen] pub fn abi_version() -> u32;                 // starts at 1
#[wasm_bindgen] pub fn build_info() -> String;               // JSON: crate versions, git commit stamped at build time, capabilities
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

As built in web Phase 3 (`crates/strato-wasm/src/lib.rs`, 2026-11-25), `RunResponse` also carries, so that block B computes nothing (M4): the full time series of `finite_volume::run` (`inventory_mol`, `cumulative_release_mol`, `release_rate_mol_s`, `min_concentration_mol_m3` beside `release_fraction` and `balance_residual_mol`); `mechanisms` (the ids solved); `mesh.region_names`; `balance` (the v1.0 balance: `start_inventory_mol`, the `final` inventory, released amount and residual, `max_abs_residual_mol` and `max_abs_residual_relative`); `diagnostics` (cells, steps, requested and used time step, output rows, `explicit_dt_bound_s`, final release fraction, largest residual absolute and relative to the start inventory, smallest concentration, the largest layer-sum difference, and `code_c` with modes, largest and final |F_B − F_C|, accuracy floor, validity time, `min_flux_ratio` and `resolved`). `analytical` carries `times_s`, `release_fraction`, `accuracy_floor`, `valid_from_s` and `modes`; it does **not** carry the pointwise difference |F_B − F_C|, because that derived series fails the browser parity rule (Section 2 of `prompt/TESTING_AND_VALIDATION.md`; see `prompt/PROGRESS.md`, "Phase 3 record"). The solver time is measured by the worker (`performance.now()` around the call) and shown as a wall time, not as a result.

`build_info` also returns `capabilities`: the list of mechanism ids (the `mechanism_id` values of Section 5.1) that this wrapper build can solve. In v1.0 it is exactly `["diffusion_layered"]`. Web Phase 6.1 adds `generation`, `decay` and `inventory_balance_v2` and Phase 6.2 adds `diffusivity_temperature` and `diffusivity_layer_laws`, each only in the commit whose tests and independent review pass. The interface activates a mechanism switch or a builder stage's live update only when its id is in `capabilities` and its status in `data/research/mechanisms.csv` is VERIFIED (Section 4, "Mechanism availability"); a request that names a mechanism outside `capabilities` is returned `rejected`. From Phase 6.1, `RunResponse` also carries the inventory terms of (P21.42) (start, generated, decayed, released, remaining, residual) for the balance of design specification Section 3.2 (rule E9); v1.0 responses carry `balance_residual_mol` as above.

`inventory_mol` is `finite_volume::inventory` of that profile; `layer_inventory_mol[j]` sums `volume * c` over the cells of region `j` in cell order; `layer_share[j]` is `layer_inventory_mol[j] / inventory_mol`. A wrapper test asserts that the layer inventories sum to `inventory_mol` within relative 1e-12.

`compare_v1` takes `{baseline, modified}` (two run requests), runs both and returns `{"schema":1,"status":"ok","label":"COMPUTED v1.0","baseline":RunResponse,"modified":RunResponse,"difference":{"times_s":[],"release_fraction":[]}}`, where the difference is modified minus baseline computed in Rust on the times both runs share (rejected if the time grids differ, never interpolated in the browser).

Errors never panic across the boundary: return `{"schema":1,"status":"rejected","errors":[...]}`. On `wasm32-unknown-unknown` the panic strategy is `abort` (`rustc --print cfg --target wasm32-unknown-unknown` prints `panic="abort"`, checked 2026-11-24 with rustc 1.100.0), so `std::panic::catch_unwind` catches nothing in the browser: a panic, and also a failed allocation, traps the instance (`RuntimeError: unreachable`). The wrapper therefore uses three defences, none of which alone is complete:

1. Validate first. Every `strato` call the wrapper makes returns `Result`, and the wrapper applies every rejection rule listed in the table below before it calls `strato`. The `expect` calls in the library are `finite_volume.rs` lines 88 and 89 (non-empty mesh, non-empty region list), `sturm_liouville.rs` line 262 (finite eigenvalue roots) and `postprocessing/comparison.rs` line 147 (a time series has its initial row; `finite_volume::run` always pushes it). With a configuration that passes the rules below, none of them is expected to fire. `src/main.rs` line 75 has one more, but the CLI is not linked into the wrapper.
2. Cap the work. `Config::validate` sets no upper bound on cell count or step count. A very large cell count can exhaust wasm memory (a trap, not an error), and a very large `end_time_s / dt_s` makes the step loop of step 3 run for an unbounded time. The wrapper defines two named constants, `MAX_TOTAL_CELLS` (sum of `cells` over all regions) and `MAX_STEPS` (`ceil(end_time_s / dt_s)`), and returns `status: "rejected"` with a message naming the limit before it calls `strato`. Their values are set in web Phase 3 from measured wasm run times and memory in the browser and recorded in `prompt/PROGRESS.md`; they are wrapper limits, not physical parameters, and the native CLI keeps no such limit. Set in web Phase 3: `MAX_TOTAL_CELLS = 2000`, `MAX_STEPS = 20000`, and two further limits that the measurements showed to be needed: `MAX_PROFILE_VALUES = 1000000` (output rows times cells; it bounds the response, about 20 bytes of JSON per value) and `MAX_SERIES_MODES = 1000` (Code C keeps every mode up to `45 / t_min`, its cost grows with the square of the mode count, and a very small first output time made `ModalRelease::new` panic with a capacity overflow in the native wrapper test; the wrapper bounds the mode count from above by the per-layer phase count of `eigenvalue_count`, `sum(k_i L_i / pi + 1)`, before it calls the library, and returns the FV result with `analytical` set to `null` and a message when the bound exceeds the limit).
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

Build (pattern from Outram Park `crates/dhoby-ghaut/web/dispersion/build.sh` and NUS-H2 `scripts/build_wasm.py`; STRATO runs this as `strato-webtools build-wasm`):

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
| Plots | `plotly.js` partial bundle via a thin React wrapper, loaded lazily: `plotly.js-basic-dist-min`, or `plotly.js-cartesian-dist-min` if Phase 5 finds that error bars or log axes need it or a phase records a need for heat maps (`docs/development/visualisation_library_comparison.md`) | avoid the full bundle |
| Maths | KaTeX, synchronous render, `\htmlClass` / `\textcolor` for term highlighting | MathJax rejected |
| State | `zustand` | playback clock kept outside React re-render paths |
| UI primitives | hand-written CSS tokens and native controls; shadcn components optional (switch, slider, tabs, tooltip only) | must pass keyboard and screen-reader checks |
| Animation | CSS transitions; `motion` optional | respect `prefers-reduced-motion` |
| Data fetching | plain `fetch` of static JSON | TanStack Query rejected |
| Atomistic viewer | none installed (rule E6) | candidate recorded in `docs/development/visualisation_library_comparison.md`; installed only when a genuine structure file exists to show |
| Library choice | `docs/development/visualisation_library_comparison.md` | the comparison and selection rationale required by the equation-first brief; a new library needs a row there first |

### Store shape (`web/src/state/store.ts`)

```ts
type Mechanism = 'decay' | 'source' | 'arrhenius';   // ids map to mechanism_id rows: decay, generation, diffusivity_temperature
type Stage = 1 | 2 | 3 | 4 | 5 | 6;                 // builder stage shown in block A
interface StratoState {
  config: V1Config;                 // mirrors inputs/synthetic/synthetic_five_layer.toml
  baselineConfig: V1Config;
  stage: Stage;                     // displayed stage; the solved stage is the highest available stage <= stage
  capabilities: string[];           // from build_info(); empty until the wasm module has loaded
  mechanisms: Record<Mechanism, { enabled: boolean; available: boolean }>; // available computed by isAvailable(), false until Phase 6
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

### Equation builder term model (`web/src/equation/terms.ts`)

Display only; no evaluation. The stage order and the meaning of each stage are in `prompt/DESIGN_SPECIFICATION.md` Section 3 (owner decision M7). `terms.ts` holds the term ids, LaTeX, stage and mechanism id; the **status column is not in `terms.ts`**: the builder reads it from the generated catalogue JSON (Section 5.1), so a status is never typed into a component. The values below are the evidence-based statuses of 2026-11-24 (`prompt/PROGRESS.md`, M7) and are the rows that `data/research/mechanisms.csv` must hold for these mechanisms.

| Term id | LaTeX | Stage | Mechanism id | Toggle | Discrete form and code | Status on 2026-11-24 |
|---|---|---|---|---|---|---|
| `lhs` | `\frac{\partial c}{\partial t}` | 1 | `diffusion_layered` | always | (5.14), `backward_euler_step` in `src/numerical/finite_volume.rs` | VERIFIED (synthetic five-layer case) |
| `diffusion` | `\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_i \frac{\partial c}{\partial r}\right)` | 1 | `diffusion_layered` | always | (5.14), (5.15), same function | VERIFIED (synthetic five-layer case) |
| `source` | `+S(r,t)` | 2 | `generation` | `source` | (P21.58), pseudocode only | RESEARCHED |
| `decay` | `-\lambda c` | 3 | `decay` | `decay` | (P21.58), pseudocode only | RESEARCHED |
| `diffusion_T` | replaces `D_i` with `D_i(T(t))` | 4 | `diffusivity_temperature` | `arrhenius` | (P22.15), pseudocode only | RESEARCHED |
| `arrhenius_layers` | `D_i(T)=\sum_{m} D_{0,i,m}\,e^{-Q_{i,m}/(R_g T)}` shown under the equation | 5 | `diffusivity_layer_laws` | `arrhenius` | (P22.1), Phase 2 Derivations Section 5.4, pseudocode only | RESEARCHED |
| `partition` | none (listed, not inserted: not in the model) | 5 | `interface_partition` | none | none | PENDING |
| species entries | coupled systems from the catalogue, shown beside the equation | 6 | catalogue ids | none | none | PENDING |

The old "DERIVED NOT IMPLEMENTED" status of this table maps to RESEARCHED, "FUTURE" to PENDING and "IMPLEMENTED" (v1.0) to VERIFIED for its stated domain. Each term links to its section in `docs/mathematics/` (v1.0.0 Full Derivation for the baseline; `STRATO_Phase2_Derivations.md` for (P21.1), (P21.8), (P22.1)), and each discrete form links to the code at the build commit, or to `docs/mathematics/STRATO_Phase2_Pseudocode.md` while no code exists.

### Mechanism availability (`web/src/equation/availability.ts`)

One function decides whether a mechanism is live: `isAvailable(id) = capabilities.includes(id) && status(id) === 'VERIFIED'`, where `capabilities` comes from `build_info()` and `status` from the generated catalogue JSON. It is a lookup, not physics. No component decides availability by itself; switches, stage live updates and solver requests all call this function, and the solver client drops (and reports) any mechanism for which it returns false. Until the wasm module has loaded, `capabilities` is empty and nothing is live.

### Particle rendering

- Radii come from the active config (`regions[].outer_radius_m`). Never hard-code TRISO dimensions.
- Concentration colour: per-cell value from the selected profile mapped through cividis (default) or viridis, linear or logarithmic scale, with a legend that states units (mol m^-3) and the maximum.
- 3D: nested spheres with a cutaway wedge; shells drawn as thin translucent surfaces plus a radial texture of cell values on the cut faces.
- 2D cross-section: a canvas disc with concentric rings per cell; hovering or tabbing to a layer shows name, radii, $D_i$, inventory in the layer (`layer_inventory_mol` and `layer_share` from `RunResponse`, never summed in TypeScript) and the parameter's provenance.

## 5 Precomputed data (`web/public/data/`)

Produced only by `strato-webtools precomputed` from the native release binary. Each JSON file starts with

```json
{"label":"PRECOMPUTED","generator":"strato <version>","commit":"<sha>","input_file":"inputs/synthetic/synthetic_five_layer.toml","input_sha256":"...","command":"...","created_utc":"...","data":{}}
```

Initial files: `synthetic_run.json` (time series and final profile), `comparison_fv_vs_analytical.json`, `convergence_study.json`, `golden_v1.json` (copy of `tests/golden/v1.0/` values for the parity check). A CI step regenerates them and fails if they differ from the committed copies.

### 5.1 Research catalogue files (owner decision M7)

The research statuses, species, mechanisms, gallery items and references that the interface shows live in files in the repository. No status, species, mechanism, reference or caption is typed into a component: `scripts/web/generate_data.py catalogue` reads the files below and writes `web/src/generated/research_catalogue.json` (with a provenance header like that of Section 5 and of the generated case files: `label` `CATALOGUE`, `kind` `research_catalogue`, generator, commit, `created_utc` and the SHA-256 of every input file; `CATALOGUE` marks a data file, it is not a result label), and the components import only that JSON. A CI step regenerates it and fails if it differs from the committed copy.

Format of every table: CSV as in RFC 4180, UTF-8 without byte-order mark, LF line ends, one header row with exactly the columns below in this order, one record per line, lists inside a cell separated by `;`, empty cell for "none". Free text is plain (no HTML); Markdown belongs in the entry files.

`data/research/mechanisms.csv`: one row per mechanism, species and material domain (the unit to which a status is assigned).

| Column | Content |
|---|---|
| `mechanism_id` | lower-case id with underscores, for example `diffusion_layered`, `generation`, `decay`, `diffusivity_temperature`, `diffusivity_layer_laws`, `interface_partition`, `ag_trapping_detrapping` |
| `name` | display name |
| `regime` | `baseline` (stages 1 to 5) or `A`, `B`, `C` (design specification Section 7.2); several regimes separated by `;` where a mechanism spans them |
| `brick` | 1 to 9 (brief Section 11) |
| `builder_stage` | 1 to 6, or empty for a catalogue-only entry |
| `species` | one of `Ag`, `Pd`, `Cs`, `Sr`, `Eu`, `I`, `Ru`, `Xe`, `Kr`, or `all` |
| `material_domain` | `kernel`, `buffer`, `IPyC`, `SiC`, `OPyC`, `interface`, `particle`, or a named case such as `synthetic_five_layer` |
| `status` | exactly one of `PENDING`, `RESEARCHED`, `IMPLEMENTED`, `VERIFIED`, `VALIDATED` |
| `status_basis` | one or two sentences: the evidence for the status |
| `evidence` | `;`-separated repository paths (tests, derivation sections, register rows such as `data/diffusivity/arrhenius.csv#A1-01`) that support the status; empty only for PENDING |
| `validation_ids` | `;`-separated validation-row ids of web Phase 7 (`prompt/TESTING_AND_VALIDATION.md` Section 7); required for VALIDATED, empty otherwise |
| `equation_ids` | `;`-separated equation numbers such as `P21.58`, `5.14`, or empty |
| `coupled` | `yes` if the mechanism needs additional coupled equations (shown as a coupled system), else `no` |
| `solver_capability` | the `capabilities` id the wrapper must report for a live switch, or empty |
| `required_parameters` | `;`-separated parameters the mechanism needs, each followed by `(registered)` with a register row id or `(missing)` |
| `missing_work` | the calculations or experiments that are still missing |
| `reference_ids` | `;`-separated ids from `references.csv` |
| `entry` | path of the Markdown entry under `docs/research/` |
| `status_date` | ISO date of the last status change |

`data/research/species.csv`: exactly nine rows.

| Column | Content |
|---|---|
| `species` | `Ag`, `Pd`, `Cs`, `Sr`, `Eu`, `I`, `Ru`, `Xe`, `Kr` |
| `element` | English name (silver, palladium, caesium, strontium, europium, iodine, ruthenium, xenon, krypton) |
| `nuclides` | `;`-separated nuclides of interest; each must be a `nuclide` in `data/nuclear/decay.csv` or carry the suffix `(not in register)` |
| `layers` | `;`-separated relevant TRISO layers |
| `regimes` | `;`-separated regimes `A`, `B`, `C` |
| `summary` | one sentence: why the species matters |
| `reference_ids` | `;`-separated ids from `references.csv` |
| `entry` | path of the Markdown entry under `docs/research/` |

A species row holds no status: a species' implementation status is the set of statuses of its rows in `mechanisms.csv` (and of the `all` rows), listed on its entry.

`data/research/references.csv`

| Column | Content |
|---|---|
| `reference_id` | short id, for example `morris2016`, `collin2016`, `iaea1997` |
| `citation` | APA citation as in the parameter register |
| `doi_or_url` | DOI URL or URL |
| `link_checked` | ISO date on which the link was opened and found to resolve, or `NOT VERIFIED` |
| `register_link` | the matching citation in `data/reference/STRATO_Parameter_Register.md`, or empty |
| `notes` | what the reference is used for |

`data/research/gallery.csv`: one row per Materials Studio item.

| Column | Content |
|---|---|
| `item_id` | short id |
| `kind` | `screenshot` (a genuine Materials Studio screenshot), `reconstruction` (an interactive view of an exported structure) or `placeholder` (Pending) |
| `title`, `caption`, `alt_text` | scientific caption and accessible explanation |
| `material`, `species` | for example `3C-SiC grain boundary`, `Ag` |
| `method`, `conditions` | computational method and simulation conditions (for a placeholder: the planned calculation) |
| `provenance` | who produced the file, when, with which software version; empty for a placeholder |
| `file` | path of the image or structure file; empty for a placeholder |
| `status` | one of the five status words |
| `mechanism_ids`, `reference_ids` | links to `mechanisms.csv` and `references.csv` |

`docs/research/<entry>.md`: one Markdown file per catalogue entry, named `species-<symbol>.md` for the nine species (for example `species-ag.md`), `regime-a.md` to `regime-c.md` for the regimes, `materials_studio_gallery.md` for the gallery, `roadmap.md` for the brick-by-brick roadmap and `<mechanism_id>.md` for a mechanism with its own page. The regime, gallery and roadmap files are overview pages: no row names them in its `entry` column (rule 4 of Section 5.2). A species file has the headings of the brief, in this order: Why it matters; Relevant TRISO layers; Candidate release mechanisms; Current evidence; Proposed governing-equation extensions; Potential Materials Studio calculations; Experimental comparison opportunities; Research limitations; Implementation status. A mechanism file has: Proposed physics; Candidate governing equations (research concepts); Required parameters; Supporting literature; Missing calculations or experiments; Interactions with other regimes. Statuses are not written into these files (the CSV is the only place); every factual statement cites a `reference_id` in the form `[ref:morris2016]`, which the generator turns into a link. Equations are written in TeX between `$$` and are labelled "research concept".

### 5.2 Catalogue check (`strato-webtools check-catalogue`)

A Rust subcommand of `crates/strato-webtools`, run as `cargo run --locked --release --manifest-path crates/Cargo.toml -p strato-webtools -- check-catalogue` from the repository root (it replaced `scripts/web/check_catalogue.py`, same rules and messages). It reads the files of Section 5.1, writes nothing, prints one PASS or FAIL line per rule and exits 1 if any rule fails. `web.yml` runs it before `npm run build`, and the web build script runs it too, so a failure stops the build. Rules:

1. Every table has exactly its header, in order; every row has as many cells as the header.
2. Every `status` (in `mechanisms.csv` and `gallery.csv`) is exactly one of `PENDING`, `RESEARCHED`, `IMPLEMENTED`, `VERIFIED`, `VALIDATED`.
3. Every id in a `reference_ids` cell exists in `references.csv`; every `[ref:...]` in `docs/research/*.md` exists there too; reference ids are unique.
4. Every `entry` path exists; every file in `docs/research/` is referenced by at least one row, except the five overview pages `regime-a.md`, `regime-b.md`, `regime-c.md`, `materials_studio_gallery.md` and `roadmap.md`, which describe several rows at once and are listed by name in the check. No other file is exempt, and rule 3 applies to the overview pages as to every other file.
5. `(mechanism_id, species, material_domain)` is unique; `species.csv` has exactly the nine species, each once; every `species` value of `mechanisms.csv` is one of the nine or `all`; every `regime` is `baseline`, `A`, `B` or `C`.
6. Every nuclide in `species.csv` exists in `data/nuclear/decay.csv` unless it carries `(not in register)`.
7. Evidence for the higher statuses: RESEARCHED needs a non-empty `evidence`; IMPLEMENTED and VERIFIED need a non-empty `solver_capability` and `evidence` paths that exist, and VERIFIED needs at least one path under `tests/` or `crates/strato-wasm/tests/`; VALIDATED needs non-empty `validation_ids`. A brick 4 to 9 row above PENDING needs the same evidence as any other row (no exception).
8. Gallery: a `screenshot` or `reconstruction` row needs an existing `file` and a non-empty `provenance`; a `placeholder` row needs an empty `file`; every `file` under the gallery folder is listed in `gallery.csv`.
9. The generated `web/src/generated/research_catalogue.json` equals a fresh regeneration (ignoring `created_utc`).

The check enforces form and references, not scientific truth: a status change still needs the evidence and the independent verification of the phase that delivers it, and is recorded in `prompt/PROGRESS.md`.

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
3. `strato-webtools check-catalogue` (Section 5.2); then `npm ci`, `npm run lint`, `npm run typecheck`, `npm test` in `web/`.
4. `npm run build`, then `npx playwright test` against `vite preview` (viewports, keyboard, reduced motion, console errors, failed requests, wasm parity, self-test).
5. On the work branch: upload the build as an artefact. Deployment to Pages follows `prompt/COMMIT_AND_DEPLOYMENT.md`.

## 8 Compatibility requirements

- Rust: the toolchain that builds STRATO today (CI records the version). Target `wasm32-unknown-unknown`.
- Node LTS; commit the lock file; run `npm audit` before deployment.
- Browsers: current Chrome, Firefox, Safari and Edge. WebGL 2 for the 3D view, with the 2D cross-section as fallback.
- GitHub Pages base path `/STRATO/`.
