//! Typed, validated readers for the scientific registers of STRATO, and the consistency checks of
//! the Python audit scripts, in Rust.
//!
//! The six register files under `data/` are read by one module each: [`geometry`], [`arrhenius`],
//! [`decay`], [`yields`], [`temperature`], [`release`]. Every column of every file is kept, with the
//! register's own number text beside the parsed number. A row is never dropped and no cell is
//! replaced by a default: a cell the reader cannot type is an error that names the file, the line,
//! the row id and the column.
//!
//! The distinctions the owner's directive requires are types:
//! independent and cumulative yield ([`yields::YieldValue`] with the markers [`yields::Independent`]
//! and [`yields::Cumulative`]); direct fission product and activation product
//! ([`common::ProductKind`]); point value and upper bound ([`release::ReleaseValue`],
//! [`arrhenius::BoundKind`], [`geometry::GeometryValue`]); evaluated data and modelling assumption
//! ([`common::EvidenceClass`]).
//!
//! [`checks`] reproduces every consistency check the Python audit scripts make on these files; the
//! `strato-data check` command runs them. This crate has no dependency and is not part of the
//! WebAssembly solver.

pub mod arrhenius;
pub mod checks;
pub mod common;
pub mod csv;
pub mod curve;
pub mod decay;
pub mod decimal;
pub mod error;
pub mod experiment_checks;
pub mod geometry;
pub mod integration_checks;
pub mod irradiation;
pub mod register;
pub mod release;
pub mod select;
pub mod table;
pub mod temperature;
pub mod yields;

pub use error::{DataError, Result};
pub use register::Register;
