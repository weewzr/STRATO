//! `strato-results`: the shared results pipeline (decision 59).
//!
//! One program writes every number and figure data set that the website, the report and the
//! deck use: the support matrix, the scenario definitions and runs, the material parameters, the
//! Method A and Method B curves for every equation stage and supported temperature, the
//! experimental bounds and points, the NRMSE records of `metric-nrmse.md`, the bound statuses, the
//! label texts of the graphs and the verification figures. Every file carries a provenance header
//! (solver version, commit, input and solver-source hashes). `strato-results run --check`
//! regenerates everything in memory and fails when a committed file differs.
//!
//! The outputs are numerical solutions of the stated model. A comparison with a register is a
//! comparison of release behaviour, not experimental validation; a scenario is a prescribed-input
//! simulation, not a validated reactor prediction.

pub mod curves;
pub mod experiment;
pub mod labels;
// The one Method B function lives in `strato-scenarios`, shared with the WebAssembly wrapper.
pub use strato_scenarios::methodb;
pub mod nrmse;
pub mod output;
pub mod params;
pub mod pipeline;
pub mod scen;
pub mod verification;
