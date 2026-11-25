//! `strato-compare`: the experimental comparison of the BTCS solver with the release register.
//!
//! The crate decides the status of every row of `data/experimental/release_measurements.csv`
//! from the row itself and the coordinator's decisions ([`status`]), runs the BTCS solver for
//! the registered geometry with the registered diffusion coefficients at the test temperature
//! ([`data`], [`model`]), computes the metrics of directive section 13 ([`metrics`]), and writes
//! CSV and JSON files ([`outputs`]), static SVG figures A to H ([`figures`]) and the report
//! ([`report`]). It is a tool: it is not part of the WebAssembly solver, and a comparison it
//! produces is a comparison of release behaviour, not an experimental validation.

pub mod compare;
pub mod data;
pub mod figures;
pub mod json;
pub mod metrics;
pub mod model;
pub mod outputs;
pub mod pipeline;
pub mod report;
pub mod status;
pub mod studies;
