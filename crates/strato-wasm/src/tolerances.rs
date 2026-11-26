//! Tolerances of the wrapper tests, as named constants in one place
//! (prompt/TESTING_AND_VALIDATION.md Section 2, "Tolerance policy").
//!
//! | Comparison | Tolerance |
//! |---|---|
//! | BTCS interface versus `numerical::btcs::run`, same platform | exact (bit-identical) |
//! | BTCS interface, browser wasm versus native, same inputs | relative 1e-12 per value ([`PARITY_RTOL`]); `examples/btcs_parity.rs` applies it to every field and lists each failure |
//! | BTCS interface, series-derived values (`analytical.*`, `diagnostics.analytical_comparison.*`) | relative 1e-12 per value plus the series' own `accuracy_floor` (absolute) of that case ([`SERIES_FLOOR_RULE`]) |
//! | BTCS balance residual | within the a-posteriori rounding bound reported with it |

/// Browser (wasm) against native wrapper output, same inputs: relative, per value.
pub const PARITY_RTOL: f64 = 1e-12;

/// Layer inventories of one profile against the total inventory of that profile, relative
/// (prompt/TECHNICAL_ARCHITECTURE.md Section 3; prompt/IMPLEMENTATION_ROADMAP.md Phase 4).
pub const LAYER_SUM_RTOL: f64 = 1e-12;

/// Why series-derived values get an absolute allowance in the native-versus-WebAssembly check.
///
/// The Sturm-Liouville series reports `accuracy_floor` (about 2.2e-16 divided by its smallest
/// flux ratio, at least 2.2e-16): the absolute accuracy of its release fraction, limited by
/// cancellation in the shooting solution. The native build and the WebAssembly build evaluate
/// exp, cos and sin with different maths libraries, so the series (never the BTCS solver, which
/// uses only arithmetic and square roots of the grid) differs between them by rounding noise
/// that the cancellation magnifies, always below that floor. A relative rule alone then fails
/// for values near zero (series release fractions of about -7.7e-14 at the earliest times,
/// below the floor, hence not resolved by the series itself) and for the difference BTCS minus
/// series, whose BTCS part is identical. The allowance is the floor, taken from the native
/// reply of the same case; `accuracy_floor` and `min_flux_ratio` themselves and all BTCS values
/// keep the strict relative rule.
pub const SERIES_FLOOR_RULE: &str = "relative 1e-12 per value plus the series accuracy_floor";
