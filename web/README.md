# STRATO web demonstration

Live: https://weewzr.github.io/STRATO/

An equation-first page for the verified STRATO v1.0 solver: the governing equation with a staged builder at the top, the live simulation directly under it (particle cross-section and 3D cutaway, radial profile, release fraction, inventory balance, numerical diagnostics), then the controls, an explanation of each active term and the research catalogue. Every number the page shows comes from the Rust solver compiled to WebAssembly (`crates/strato-wasm`, run in a worker) or from verified precomputed native output with a provenance header; no scientific equation is written in TypeScript.

What the demonstration can and cannot do today:

- **VERIFIED**: five-layer spherical diffusion with a constant diffusivity per layer, verified on the synthetic five-layer case. The live simulation runs that synthetic case and says so. AGR-1 UCO TRISO is shown as the reference fuel by its registered geometry only, never with the synthetic diffusivities.
- **RESEARCHED, not implemented**: fission-product generation, radioactive decay and temperature-dependent diffusivity. They are derived in `docs/mathematics/STRATO_Phase2_Derivations.md`; their switches are shown and disabled.
- **PENDING**: every other mechanism of the research catalogue, the Materials Studio calculations and FLiBe and cover-gas transport (planned for a future phase). Nothing is VALIDATED: no comparison with measurements has been computed yet.

The site also carries the derivation and register documents, rendered from the repository with the mathematics typeset: https://weewzr.github.io/STRATO/docs/index.html.

## Layout

| Path | Content |
|---|---|
| `web/src/` | React and TypeScript interface: sections, equation builder (KaTeX), plots (Plotly), particle views (canvas, three.js), solver client and worker |
| `web/src/generated/` | Data generated from the repository (case inputs, research catalogue, colour maps, AGR-1 geometry); never edited by hand |
| `web/public/data/` | Precomputed native results with provenance headers |
| `web/tests/`, `web/e2e/` | Vitest unit and component tests; Playwright browser tests |
| `crates/strato-wasm/` | WebAssembly wrapper of the verified library (own workspace and lock file) |
| `crates/strato-docsite/` | Tools workspace member: renders the documents into the build and checks every link of the built site |
| `scripts/web/` | Generators and checks of the web data (to be ported to Rust in migration stage R1) |

## Requirements

- Node.js 24 with npm (the lock file is `web/package-lock.json`).
- Rust stable with the `wasm32-unknown-unknown` target, and the wasm-bindgen CLI at the version pinned in `crates/strato-wasm/Cargo.lock` (0.2.130): `cargo install wasm-bindgen-cli --version 0.2.130 --locked`.
- Python 3.11 or later for the existing generators in `scripts/web/` (standard library only).
- For the browser tests: `npx playwright install chromium`, or an installed Microsoft Edge with `PW_CHANNEL=msedge`.

## Build and test

Run from the repository root unless stated. These are the steps of `.github/workflows/web.yml`, in its order.

```sh
# Generated data equal a fresh regeneration; research catalogue check and its fixture tests
python3 -I -X utf8 scripts/web/generate_data.py case inputs/synthetic/synthetic_five_layer.toml --check
python3 -I -X utf8 scripts/web/generate_data.py catalogue --check
python3 -I -X utf8 scripts/web/generate_data.py colormaps --check
python3 -I -X utf8 scripts/web/generate_data.py geometry --check
python3 -I -X utf8 scripts/web/check_catalogue.py
python3 -I -X utf8 scripts/web/tests/test_check_catalogue.py
python3 -I -X utf8 scripts/web/tests/test_catalogue_blocks.py

# WebAssembly wrapper: native parity with tests/golden/v1.0, then the module with its export and size check
cargo test --locked --manifest-path crates/strato-wasm/Cargo.toml
sh scripts/web/build_wasm.sh
sh scripts/web/precompute.sh --check
python3 -I -X utf8 scripts/web/generate_data.py figures --check

# Interface
cd web
npm ci
npm run lint
npm run typecheck
npm test
npm run build            # catalogue check, wasm package check, type check, Vite build into web/dist
cd ..

# Documents and links (tools workspace)
cargo fmt --manifest-path crates/Cargo.toml --all --check
cargo clippy --locked --manifest-path crates/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path crates/Cargo.toml
cargo run --locked --release --manifest-path crates/Cargo.toml -p strato-docsite -- build --repo . --out web/dist
cargo run --locked --release --manifest-path crates/Cargo.toml -p strato-docsite -- check-links web/dist --base /STRATO/

# Content check of the complete site, then the browser tests against `vite preview`
cd web
npm run check:content
EXPECT_COMMIT=$(git rev-parse HEAD) npx playwright test --project=chromium
npx playwright test --project=performance   # frame-rate tests, on their own with one worker
```

On Windows use `python` for `python3` and run `python -I -X utf8 scripts/web/build_wasm.py` and `python -I -X utf8 scripts/web/generate_data.py precomputed --check` for the two shell wrappers. `npm run dev` serves the interface with hot reload at http://localhost:5173/STRATO/ (the documents are not served there; build them into `web/dist` and use `npm run preview`).

Environment variables of the browser tests: `EXPECT_COMMIT` (the build stamp the self-test and the document pages must show), `PW_CHANNEL` (`msedge` for an installed Edge), `SHOTS_DIR` (write screenshots), `LIMITS_OUT` and `PARITY_OUT` (write the timing and parity records as JSON).

## Documents

`crates/strato-docsite build` renders `docs/mathematics/**` and `data/reference/*.md` (the list is `DOCS` in `crates/strato-docsite/src/site.rs`) into `web/dist` at the addresses of the earlier Jekyll site, for example `docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.html`. The mathematics stays TeX and is typeset in the browser by the KaTeX copy that the demonstration already ships, served from `docs/_assets/` (no third-party request). Figures and PDFs are copied beside their pages; links to files that are not published go to GitHub at the build commit. `prompt/`, `README.md`, `CHANGELOG.md` and `docs/development/` are not published. The build fails on any `$` that is not read as mathematics, on a `\tag` outside mathematics and on a link to a missing file; `check-links` fails on any link of the built site that does not resolve.

## Deployment

`.github/workflows/web.yml` runs the full gate above on every pull request and on pushes to `main` and to the work branch. On a push to `main` or a manual run on `main` (workflow_dispatch), the `web` job uploads `web/dist` (demonstration and documents) as the Pages artefact, and the `deploy` job publishes it with `actions/deploy-pages` (permissions `pages: write` and `id-token: write`, environment `github-pages`). Nothing is deployed from a pull request or another branch.

The repository's Pages source is "GitHub Actions". A failed deploy job prints its own explanation in the log.

Merging into `main` follows `prompt/COMMIT_AND_DEPLOYMENT.md` Section 6: `main` is merged into the work branch with a normal merge commit when it has moved, the checks run, and `main` is fast-forwarded to the checked commit.
