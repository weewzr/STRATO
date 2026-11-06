# STRATO

**Spherical TRISO Analysis and Transport Operations**

STRATO models how fission products (for example silver, caesium and strontium) diffuse out of a TRISO coated fuel particle. It solves the same problem two independent ways, a conservative finite-volume method and an exact multilayer eigenfunction series. Each method checks the other.

> **Status: v1.0 (Phase 1).** Constant temperature, constant diffusivity in each layer, initial inventory in the kernel. Radioactive decay, fission-product generation and temperature-dependent diffusivity are the v2.0 scope and are **not** in this release. See [Development status](#development-status).

## Scientific objective

The objective is a transparent and verifiable model of fission-product release from a single TRISO particle. Every equation, numerical method and parameter should be traceable to a derivation or a published source. v1.0 provides the verified numerical foundation. Physical inputs from the literature are added in later versions.

## Particle geometry

```text
            r = 0                                                     r = R5
  centre ── kernel ── buffer ── IPyC ── SiC ── OPyC ── outer surface (c = 0)
   region   i = 1      i = 2     i = 3   i = 4   i = 5
   outer    R1         R2        R3      R4      R5
```

There are five concentric spherical layers, each with its own constant diffusivity $D_i$. The radii and diffusivities are inputs. The bundled example `inputs/synthetic/synthetic_five_layer.toml` uses **synthetic** values chosen for verification, not measured TRISO properties.

## Mathematical model (v1.0)

Inside layer $i$ the concentration $c(r,t)$ obeys spherical diffusion:

$$\frac{\partial c}{\partial t} = \frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2 D_i \frac{\partial c}{\partial r}\right), \qquad R_{i-1} < r < R_i .$$

| Assumption | v1.0 treatment |
|---|---|
| Centre | Symmetry, $\partial c/\partial r = 0$ at $r = 0$ |
| Interfaces | Perfect contact: $c$ and the flux $D\,\partial c/\partial r$ are continuous |
| Outer surface | Absorbing, $c(R_5, t) = 0$ |
| Initial state | Uniform $c_0$ in the kernel, zero elsewhere |
| Temperature | Constant; the input value is recorded only |
| Decay, generation | Not included |

The release fraction is $F(t) = M_\mathrm{rel}(t)/M_0$, the share of the initial inventory that has left through the outer surface.

### Analytical solver (Code C)

`src/analytical/sturm_liouville.rs` uses the multilayer Sturm-Liouville eigenfunction expansion. It writes $u = rc$ in each layer and propagates $u$ and the flux through the layers. Eigenvalues are isolated by a Sturm count and refined by bisection. The release is the series $F(t) = 1 - \sum_n w_n e^{-\Lambda_n t}$. The code reports how many modes it used, the time from which the series is valid, and its round-off accuracy floor.

### Finite-volume solver (Code B)

`src/numerical/finite_volume.rs` integrates the exact balance of each spherical shell:
- shell volumes and wall areas are exact;
- interface conductances are series resistances;
- time stepping is Backward Euler;
- each step is one tridiagonal solve, done by subtraction-free elimination.

Inventory, cumulative release and their balance are tracked at every step.

The derivation of both methods is in [`docs/mathematics/phase1/`](docs/mathematics/phase1/).

## Installation

Install a stable Rust toolchain (https://rustup.rs), then build:

```bash
git clone https://github.com/weewzr/STRATO.git
cd STRATO
cargo build --release
```

## Running

```bash
cargo run --release -- inputs/synthetic/synthetic_five_layer.toml
```

Options: `--fv-only` (finite volume only, no comparison) and `--no-convergence` (skip the refinement study).

To run your own case, copy `inputs/literature/literature_inputs.template.toml` and fill in every `INSERT_VALUE_HERE`. The program refuses to run while any placeholder is left, and lists each one.

## Outputs

Files are written to the `[output] directory` of the input file:

| File | Content |
|---|---|
| `timeseries.csv` | time, inventory, cumulative release, release fraction, release rate, balance residual, minimum concentration |
| `final_profile.csv` | concentration in every cell at the end time |
| `comparison_fv_vs_analytical.csv` | finite-volume vs analytical release fraction over time |
| `convergence_study.csv` | spatial and temporal refinement against the exact series |
| `final_profile.svg`, `release_fv_vs_analytical.svg`, `convergence.svg` | figures |

Standard output gives a `key=value` summary, including the observed convergence orders.

## Verification

```bash
cargo test --release                      # unit, cross-method and Phase 1 regression tests
cargo run --release --example verify      # numerical verification; every printed claim is asserted
```

| Check | v1.0 result (synthetic case) |
|---|---|
| Spatial order, finite volume vs exact series | 2.00 |
| Temporal order, Backward Euler | 1.00 against a same-mesh reference |
| Inventory balance | relative residual at most 2.8e-13 |
| Finite volume vs series at $t = 2\times10^5$ s | spatially extrapolated FV agrees with the series to 1.3e-12 |
| Regression against the delivered v1.0 outputs | identical to round-off |

This is **verification**: it shows that the code solves its equations correctly. It is **not validation**. Two methods can agree with each other and both still rest on the wrong physics or the wrong inputs. Comparison with measured AGR-1 release data is planned for v2.0. See [`docs/development/provenance.md`](docs/development/provenance.md) for the evidence and the claims that remain unverified.

## Development status

| Version | Stage | Scope | Status |
|---|---|---|---|
| v1.0 | Phase 1 (developed June to October 2026) | Multilayer analytical and finite-volume diffusion model | Released |
| v2.0 | Phase 2 (November 2026) | Radioactive decay, fission-product generation, inventory accounting, Arrhenius diffusivity, temperature histories | In development: mathematics and pseudocode first |
| v3.0 | Future | Partition coefficients, matrix transport, inverse modelling, uncertainty quantification, particle populations | Planned |

## Known limitations of v1.0

- Inputs are synthetic. No literature parameters are bundled yet.
- Temperature is constant, and each layer's diffusivity does not depend on temperature.
- There is no radioactive decay and no fission-product generation. All inventory starts in the kernel.
- Interfaces are in perfect contact, with no partition coefficients.
- The outer surface is a perfect sink.
- The model covers a single intact particle. Matrix, failed particles and populations are not modelled.
- Backward Euler is only first order in time, so the time step controls accuracy at early times.

## Documentation

| Topic | Location |
|---|---|
| Phase 1 derivation and pseudocode | [`docs/mathematics/phase1/`](docs/mathematics/phase1/) |
| Where v1.0 came from, and verification evidence | [`docs/development/provenance.md`](docs/development/provenance.md) |
| Version history | [`CHANGELOG.md`](CHANGELOG.md) |

## Key references

- Gelbard, F. (2003). *Analytical modeling of fission product releases by diffusion from multicoated fuel particles* (SAND2002-3966). Sandia National Laboratories.
- International Atomic Energy Agency. (1997). *Fuel performance and fission product behaviour in gas cooled reactors* (IAEA-TECDOC-978). IAEA.
- Hales, J. D., Jiang, W., Toptan, A., & Gamble, K. A. (2021). Modeling fission product diffusion in TRISO fuel particles with BISON. *Journal of Nuclear Materials, 548*, 152840. https://doi.org/10.1016/j.jnucmat.2021.152840

## Citation

If you use STRATO, please cite it with the metadata in [`CITATION.cff`](CITATION.cff).

## License

MIT. See [`LICENSE`](LICENSE).
