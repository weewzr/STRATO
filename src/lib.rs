//! STRATO: Spherical TRISO Analysis and Transport Operations.
//!
//! Fission-product diffusion through a five-layer spherical TRISO particle
//! (kernel, buffer, IPyC, SiC, OPyC), solved two independent ways on the same input:
//!
//! - [`numerical::finite_volume`]: conservative spherical finite volume, Backward Euler,
//!   tridiagonal solve (Code B);
//! - [`analytical::sturm_liouville`]: multilayer Sturm-Liouville eigenfunction expansion
//!   (Code C).
//!
//! Version 1.0 scope: constant temperature, constant diffusivity in each layer, perfect
//! interface contact, absorbing outer surface, initial inventory in the kernel only.
//! See `docs/mathematics/STRATO_Mathematical_Derivations.md`.

// Some helpers are used only by the verification example and tests.
#![allow(dead_code)]

pub mod analytical;
pub mod config;
pub mod numerical;
pub mod postprocessing;
