//! STRATO: Spherical TRISO Analysis and Transport Operations.
//!
//! Fission-product diffusion through a five-layer spherical TRISO particle
//! (kernel, buffer, IPyC, SiC, OPyC), solved two independent ways on the same input:
//!
//! - [`numerical::btcs`]: Method A, a spherical backward-time centred-space (BTCS)
//!   finite-difference scheme with interface conditions at the four material interfaces and a
//!   tridiagonal solve (the Thomas routine of Document 2);
//! - [`analytical::sturm_liouville`]: Method B, the multilayer Sturm-Liouville eigenfunction
//!   expansion.
//!
//! Version 2.0 scope of the solver: constant temperature, constant diffusivity in each layer,
//! perfect interface contact, absorbing outer surface, initial inventory in the kernel only.
//! The mathematics is in `docs/mathematics/btcs/` (Documents 00 to 04). The finite-volume
//! solver of version 1.x is retired; `docs/archive/finite-volume/README.md` says where it is
//! kept. Numerical verification is not experimental validation.

pub mod analytical;
pub mod config;
pub mod numerical;
pub mod physics;
pub mod postprocessing;
pub mod validation;
