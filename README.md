# STRATO: Spherical TRISO Analysis and Transport Operations

STRATO models how fission products (for example silver, caesium and strontium) diffuse out of a single TRISO coated fuel particle. It solves the same multilayer spherical diffusion problem two independent ways, with a conservative finite-volume method and with an exact multilayer eigenfunction series, and each method checks the other. Version 1.0 (Phase 1) is released and verified on a synthetic five-layer case. Phase 2 (v2.0) exists as mathematics and pseudocode only; no Phase 2 code has been written.

> **Synthetic inputs.** The only bundled input, `inputs/synthetic/synthetic_five_layer.toml`, uses synthetic radii and diffusivities chosen for verification. They are not measured TRISO properties, and no result from this case is a prediction of real fission-product release.

<p align="center">
  <img src="docs/mathematics/v1.0.0/figures/triso_cross_section.png" alt="Cross-section of a TRISO particle showing the kernel, buffer, inner pyrolytic carbon, silicon carbide and outer pyrolytic carbon layers, with outer radii R1 to R5 marked, and below it the same five layers drawn as intervals along the radius r" width="640">
</p>

The five layers of a TRISO particle and their outer radii $R_1$ to $R_5$. The figure is drawn to average AGR-1 Baseline dimensions (source in `scripts/figures/v1_0_0_triso_cross_section.py`); the synthetic example uses different radii.

## Status

**v1.0 (Phase 1).** Constant temperature, constant diffusivity in each layer, initial inventory in the kernel. Radioactive decay, fission-product generation and temperature-dependent diffusivity are the v2.0 scope and are outside this release. See [Development status](#development-status).

## Scientific objective

The objective is a transparent and verifiable model of fission-product release from a single TRISO particle. Every equation, numerical method and parameter should be traceable to a derivation or a published source. v1.0 provides the verified numerical foundation. Physical inputs from the literature are added in later versions.

## Particle geometry

```text
            r = 0                                                     r = R5
  centre ── kernel ── buffer ── IPyC ── SiC ── OPyC ── outer surface (c = 0)
   region   i = 1      i = 2     i = 3   i = 4   i = 5
   outer    R1         R2        R3      R4      R5
```

There are five concentric spherical layers, each with its own constant diffusivity $D_i$ in $\mathrm{m^2\,s^{-1}}$. The radii (in $\mathrm{m}$) and diffusivities are inputs.

The bundled synthetic case uses these values:

| Layer $i$ | Name | Outer radius $R_i$ | Diffusivity $D_i$ | Cells |
|---|---|---|---|---|
| 1 | Kernel | $2.0\times10^{-4}\ \mathrm{m}$ | $1.0\times10^{-12}\ \mathrm{m^2\,s^{-1}}$ | 40 |
| 2 | Buffer | $3.0\times10^{-4}\ \mathrm{m}$ | $4.0\times10^{-10}\ \mathrm{m^2\,s^{-1}}$ | 40 |
| 3 | IPyC | $3.5\times10^{-4}\ \mathrm{m}$ | $8.0\times10^{-13}\ \mathrm{m^2\,s^{-1}}$ | 40 |
| 4 | SiC | $4.0\times10^{-4}\ \mathrm{m}$ | $2.0\times10^{-14}\ \mathrm{m^2\,s^{-1}}$ | 40 |
| 5 | OPyC | $4.5\times10^{-4}\ \mathrm{m}$ | $7.0\times10^{-13}\ \mathrm{m^2\,s^{-1}}$ | 40 |

The initial kernel concentration is $c_0 = 1.0\ \mathrm{mol\,m^{-3}}$, the time step is $\Delta t = 200\ \mathrm{s}$ and the end time is $2\times10^5\ \mathrm{s}$. The input records a temperature of $1000\ \mathrm{K}$, which v1.0 does not use in the calculation.

## Mathematical model (v1.0)

Inside layer $i$ the concentration $c(r,t)$ in $\mathrm{mol\,m^{-3}}$ obeys spherical diffusion:

$$\frac{\partial c}{\partial t} = \frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_i \frac{\partial c}{\partial r}\right), \qquad R_{i-1} < r < R_i ,$$

with $R_0 = 0$.

| Assumption | v1.0 treatment |
|---|---|
| Centre | Symmetry, $\partial c/\partial r = 0$ at $r = 0$ |
| Interfaces | Perfect contact: $c$ and the flux $D\,\partial c/\partial r$ are continuous |
| Outer surface | Absorbing, $c(R_5, t) = 0$ |
| Initial state | Uniform $c_0$ in the kernel, zero elsewhere |
| Temperature | Constant; the input value is recorded only |
| Decay, generation | Not included |

The release fraction is

$$F(t) = \frac{M_\mathrm{rel}(t)}{M_0},$$

the share of the initial inventory $M_0$ (in $\mathrm{mol}$) that has left through the outer surface by time $t$.

### Analytical solver (Code C)

`src/analytical/sturm_liouville.rs` uses the multilayer Sturm-Liouville eigenfunction expansion. It writes $u = rc$ in each layer and propagates $u$ and the flux through the layers. Eigenvalues are isolated by a Sturm count and refined by bisection. The release is the series

$$F(t) = 1 - \sum_n w_n e^{-\Lambda_n t},$$

with modal weights $w_n$ and decay rates $\Lambda_n$ in $\mathrm{s^{-1}}$. The code reports how many modes it used, the time from which the series is valid, and its round-off accuracy floor.

### Finite-volume solver (Code B)

`src/numerical/finite_volume.rs` integrates the exact balance of each spherical shell:

- shell volumes $V_j = \tfrac{4\pi}{3}\left(r_{j+1/2}^3 - r_{j-1/2}^3\right)$ and wall areas $4\pi r_{j+1/2}^2$ are exact, and each cell value sits at the volume centroid $r_j$ of its shell;
- interface conductances are series resistances, $G_{j+1/2} = 4\pi r_{j+1/2}^2 \big/ \left[(r_{j+1/2} - r_j)/D_j + (r_{j+1} - r_{j+1/2})/D_{j+1}\right]$ in $\mathrm{m^3\,s^{-1}}$, and the outer face uses $G_R = 4\pi R_5^2 D_N/(R_5 - r_N)$ with $c = 0$ at $R_5$;
- time stepping is Backward Euler,

  $$V_j\,\frac{c_j^{n+1} - c_j^{n}}{\Delta t} = G_{j-1/2}\left(c_{j-1}^{n+1} - c_j^{n+1}\right) - G_{j+1/2}\left(c_j^{n+1} - c_{j+1}^{n+1}\right);$$

- each step is one tridiagonal solve, done by subtraction-free elimination.

Inventory, cumulative release (the sum of $G_R\,c_N^{n+1}\,\Delta t$) and their balance are tracked at every step.

### Derivations

The derivation of both methods is in two documents under [`docs/mathematics/v1.0.0/`](docs/mathematics/v1.0.0/). The [Full Mathematical Derivation](docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md) ([PDF](docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.pdf)) is the detailed learning and derivation document. It builds the analytical solution (Code C) and the finite-volume scheme (Code B) one step at a time. The [Final Deliverable](docs/mathematics/v1.0.0/STRATO_Final_Deliverable.md) ([PDF](docs/mathematics/v1.0.0/STRATO_Final_Deliverable.pdf)) is the concise implementation reference. It connects each final equation to its pseudocode and to the Rust code with file and line references. The Phase 1 documents in [`docs/mathematics/phase1/`](docs/mathematics/phase1/) stay as the historical record.

## Installation

Install a stable Rust toolchain (https://rustup.rs), then build:

```bash
git clone https://github.com/weewzr/STRATO.git
cd STRATO
cargo build --release
```

Dependency versions are pinned in `Cargo.lock`.

## Running

```bash
cargo run --release -- inputs/synthetic/synthetic_five_layer.toml
```

Options: `--fv-only` (finite volume only, no comparison) and `--no-convergence` (skip the refinement study).

To run your own case, copy `inputs/literature/literature_inputs.template.toml` and fill in every `INSERT_VALUE_HERE`. The program refuses to run while any placeholder is left, and lists each one.

## Outputs

Files are written to the `[output] directory` of the input file (`results/synthetic-five-layer/` for the bundled case):

| File | Content |
|---|---|
| `timeseries.csv` | time (s), inventory (mol), cumulative release (mol), release fraction, release rate (mol/s), balance residual (mol), minimum concentration (mol/m³) |
| `final_profile.csv` | concentration (mol/m³) in every cell, with its radius (m) and material, at the end time |
| `comparison_fv_vs_analytical.csv` | finite-volume vs analytical release fraction over time, with the absolute difference |
| `convergence_study.csv` | spatial and temporal refinement against the exact series |
| `final_profile.svg`, `release_fv_vs_analytical.svg`, `convergence.svg` | figures |

Standard output gives a `key=value` summary, including the observed convergence orders. The expected outputs of the synthetic case are committed in [`tests/golden/v1.0/`](tests/golden/v1.0/).

## Verification

```bash
cargo test --release
cargo run --release --example verify
```

The first command runs the unit, cross-method and Phase 1 regression tests. The second runs the numerical verification, in which every printed claim is asserted; it ends with `FV_VERIFICATION PASS`. Continuous integration ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)) runs the tests, the verification program and the synthetic case on pushes to `main`, on version tags and on pull requests.

| Check | v1.0 result (synthetic case) |
|---|---|
| Spatial order, finite volume vs exact series | 2.00 on four refinements (100 to 1600 cells) |
| Temporal order, Backward Euler | 0.98 to 0.89 against the exact series, where spatial error enters the comparison; 0.999 to 1.000 against a same-mesh time-converged reference |
| Inventory balance | relative residual at most 2.8e-13 |
| Finite volume vs series at $t = 2\times10^5\ \mathrm{s}$ | $F = 0.43255563909$; spatially extrapolated FV agrees with the series to 1.3e-12 |
| Regression against the delivered v1.0 outputs | identical to round-off |

The spatial orders, temporal orders against the series, balance residuals and the series value of $F$ are in [`tests/golden/v1.0/verify_stdout.txt`](tests/golden/v1.0/verify_stdout.txt). The same-mesh temporal order and the three-way agreement of the Rust series, a 50-digit Python series and the extrapolated finite volume are recorded in [`docs/development/provenance.md`](docs/development/provenance.md).

This is **verification**: it shows that the code solves its equations correctly. **Validation** against measured release data has not been done. Two methods can agree with each other and both still rest on the wrong physics or the wrong inputs. Comparison with measured AGR-1 release data is planned for v2.0. See [`docs/development/provenance.md`](docs/development/provenance.md) for the evidence and the claims that remain unverified.

## Development status

| Version | Stage | Scope | Status |
|---|---|---|---|
| v1.0 | Phase 1 (developed June to October 2026) | Multilayer analytical and finite-volume diffusion model | Released (v1.0.0, v1.0.1) |
| v2.0 | Phase 2 (November 2026) | Radioactive decay, fission-product generation, inventory accounting, Arrhenius diffusivity, temperature histories | In development: mathematics and pseudocode first; no Phase 2 code yet |
| v3.0 | Future | Partition coefficients, matrix transport, inverse modelling, uncertainty quantification, particle populations | Planned |

## Known limitations of v1.0

- The only bundled input is synthetic. Literature parameters for Phase 2 are collected in [`data/`](data/reference/README.md), but v1.0 does not read them and no literature input file is bundled.
- Temperature is constant, and each layer's diffusivity does not depend on temperature.
- There is no radioactive decay and no fission-product generation. All inventory starts in the kernel.
- Interfaces are in perfect contact, with no partition coefficients.
- The outer surface is a perfect sink.
- The model covers a single intact particle. Matrix, failed particles and populations are not modelled.
- Backward Euler is only first order in time, so the time step controls accuracy at early times.

## Repository layout

| Path | Content |
|---|---|
| `src/` | Rust library and command-line program: analytical solver, finite-volume solver, input handling, CSV and SVG output |
| `tests/`, `examples/verify.rs` | Unit, cross-method and regression tests, golden v1.0 outputs, and the numerical verification program |
| `inputs/` | Synthetic five-layer case and the literature input template |
| `data/` | Literature parameter register and CSV tables for Phase 2 |
| `docs/mathematics/` | v1.0 derivations, Phase 2 derivations and pseudocode, Phase 1 historical record |
| `docs/development/` | Provenance, Phase 2 review package and the list of v1.0 to v2.0 changes |
| `scripts/` | Python derivation checks, figure scripts and parameter-register tools (versions pinned in `requirements.lock.txt`) |
| `prompt/` | Instruction package for the planned web demo |

## Documentation

| Topic | Location |
|---|---|
| Full Mathematical Derivation (detailed learning and derivation document: physics, conservation law, analytical solution, finite volume, numerical properties) | [Markdown](docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.md), [PDF](docs/mathematics/v1.0.0/STRATO_Full_Mathematical_Derivation.pdf) |
| Final Deliverable (concise implementation reference: final equation, pseudocode and Rust code with file and line references) | [Markdown](docs/mathematics/v1.0.0/STRATO_Final_Deliverable.md), [PDF](docs/mathematics/v1.0.0/STRATO_Final_Deliverable.pdf) |
| Phase 1 derivation and pseudocode (historical record, kept as delivered) | [`docs/mathematics/phase1/`](docs/mathematics/phase1/) |
| Phase 2 derivations (design only, not implemented) | [Markdown](docs/mathematics/STRATO_Phase2_Derivations.md), [PDF](docs/mathematics/STRATO_Phase2_Derivations.pdf) |
| Phase 2 pseudocode (design only, not implemented) | [Markdown](docs/mathematics/STRATO_Phase2_Pseudocode.md), [PDF](docs/mathematics/STRATO_Phase2_Pseudocode.pdf) |
| Changes planned from v1.0 to v2.0 | [`docs/development/v1_to_v2_changes.md`](docs/development/v1_to_v2_changes.md) |
| Phase 2 review package | [`docs/development/phase2_review_package.md`](docs/development/phase2_review_package.md) |
| Where v1.0 came from, and verification evidence | [`docs/development/provenance.md`](docs/development/provenance.md) |
| Contribution rules | [`CONTRIBUTING.md`](CONTRIBUTING.md) |
| Version history | [`CHANGELOG.md`](CHANGELOG.md) |

## Key references

- Gelbard, F. (2003). *Analytical modeling of fission product releases by diffusion from multicoated fuel particles* (SAND2002-3966). Sandia National Laboratories.
- International Atomic Energy Agency. (1997). *Fuel performance and fission product behaviour in gas cooled reactors* (IAEA-TECDOC-978). IAEA.
- Hales, J. D., Jiang, W., Toptan, A., & Gamble, K. A. (2021). Modeling fission product diffusion in TRISO fuel particles with BISON. *Journal of Nuclear Materials, 548*, 152840. https://doi.org/10.1016/j.jnucmat.2021.152840

## Citation

If you use STRATO, please cite it with the metadata in [`CITATION.cff`](CITATION.cff) (version 1.0.1, released 2026-11-06).

## Licence

The `LICENSE` file on `main` is the GNU Affero General Public License, version 3, and `Cargo.toml` and `CITATION.cff` declare `AGPL-3.0-only`. See [`LICENSE`](LICENSE).

Releases v1.0.0 and v1.0.1 were published under the MIT licence (the `LICENSE` file at both tags) and remain available under it. The change to AGPL-3.0-only was made after v1.0.1 to match the licence of the [Outram Park](https://github.com/theodoreOnzGit/outram-park-backend) project, whose interface the planned STRATO web demo builds on.
