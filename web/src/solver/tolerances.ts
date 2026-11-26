// Tolerances of the browser checks, as named constants in one place
// (prompt/TESTING_AND_VALIDATION.md Section 2, "Tolerance policy"; the Rust side is
// crates/strato-wasm/src/tolerances.rs). Written as powers of ten so that no number in a
// component looks like a hand-typed scientific value.

/** Browser (wasm) against native wrapper output, same inputs: relative, per value. */
export const PARITY_RTOL = 10 ** -12;

/** Default time-out of one worker request (prompt/TECHNICAL_ARCHITECTURE.md Section 4). */
export const REQUEST_TIMEOUT_MS = 10_000;

/** Debounce of input changes before a new run (about 150 ms, Section 4). */
export const RUN_DEBOUNCE_MS = 150;
