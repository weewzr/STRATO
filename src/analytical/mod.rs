//! Analytical solution methods.

pub mod applicability;
#[cfg(any(test, feature = "extension-estimate"))]
pub mod extension_floor;
pub mod extensions;
pub mod homogeneous_sphere;
pub mod sturm_liouville;
