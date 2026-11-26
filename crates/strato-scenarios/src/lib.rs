//! `strato-scenarios`: what the results pipeline and the WebAssembly wrapper share.
//!
//! * [`laws`]: the published diffusivity law of each layer for an element and a phase, the
//!   nuclide data of the registers, and the reference fission-rate density (decisions 39, 40, 42,
//!   47);
//! * [`definitions`]: the scenario definitions of `data/scenarios/` turned into the histories and
//!   the step plan of the scenario engine, and a run of a definition;
//! * [`methodb`]: the one function that decides where Method B (Sturm-Liouville) is available and
//!   draws a value only where the drawing mask accepts it;
//! * [`support`]: the lists of nuclides, temperatures and equation stages that the selectors of
//!   the website offer.
//!
//! The code was written for `strato-results` and moved here unchanged, so that the wrapper
//! computes every scenario history once, in Rust, with the same functions that write
//! `data/results/`. The crate has no file access and no clock; it builds for
//! `wasm32-unknown-unknown`.

pub mod definitions;
pub mod laws;
pub mod methodb;
pub mod support;
