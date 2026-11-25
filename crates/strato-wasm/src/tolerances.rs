//! Tolerances of the wrapper tests, as named constants in one place
//! (prompt/TESTING_AND_VALIDATION.md Section 2, "Tolerance policy").
//!
//! | Comparison | Tolerance |
//! |---|---|
//! | Wrapper versus `finite_volume::run`, same platform | exact (bit-identical) |
//! | Wrapper or wasm versus `tests/golden/v1.0/` | relative 1e-12 (the existing regression tolerance) |
//! | Browser wasm versus native, same inputs | relative 1e-12 per value, identical array lengths |
//! | Inventory balance | no worse than the v1.0 level |
//! | BTCS interface versus `numerical::btcs::run`, same platform | exact (bit-identical) |
//! | BTCS interface, browser wasm versus native, same inputs | relative 1e-12 per value ([`PARITY_RTOL`]); `examples/btcs_parity.rs` applies it to every field and lists each failure |
//! | BTCS balance residual | within the a-posteriori rounding bound reported with it |

/// Wrapper or wasm against the golden files: the tolerance of `tests/phase1_regression.rs`
/// (`RTOL`), used with the same scale rule: |a - b| <= RTOL * max(scale, |a|, |b|).
pub const GOLDEN_RTOL: f64 = 1e-12;

/// Browser (wasm) against native wrapper output, same inputs: relative, per value.
pub const PARITY_RTOL: f64 = 1e-12;

/// Layer inventories of one profile against the total inventory of that profile, relative
/// (prompt/TECHNICAL_ARCHITECTURE.md Section 3; prompt/IMPLEMENTATION_ROADMAP.md Phase 4).
pub const LAYER_SUM_RTOL: f64 = 1e-12;

/// The v1.0 balance level of the synthetic case: the largest balance residual relative to the
/// start inventory that `cargo run --release --example verify` prints for 200 cells
/// (`CONSERVATION cells=200 ... balance_rel=2.806e-13`, tests/golden/v1.0/verify_stdout.txt),
/// rounded up at its last printed digit.
pub const V1_BALANCE_REL_LEVEL: f64 = 2.8065e-13;
