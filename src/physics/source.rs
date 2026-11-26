//! Volumetric source of a fission product from a fission-rate density and a yield.
//!
//! `S = F_dot * y / N_A` in mol m^-3 s^-1, with `F_dot` in fissions m^-3 s^-1 and `y` in
//! atoms per fission (the atom form `F_dot * y` is also provided).
//!
//! The yield kind (independent or cumulative) and the product kind (direct fission product
//! or activation product) are types. Which yield a model must use is a modelling decision
//! documented with the derivation: cumulative when the precursors are not modelled as
//! separate species, independent when they are. An activation product (Ag-110m, Cs-134,
//! Eu-154 and similar) is made by neutron capture on a fission product, so a direct-yield
//! source is refused for it: the function returns an error and no number.

use super::{non_negative, PhysicsError, AVOGADRO};

/// Which fission yield the number is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YieldKind {
    /// Atoms formed directly by fission.
    Independent,
    /// Independent yield plus everything that arrives later by decay of precursors.
    Cumulative,
}

/// How the nuclide is produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductKind {
    /// Made by fission or by decay of fission products: a yield source applies.
    DirectFissionProduct,
    /// Made by neutron capture: no direct-yield source is allowed.
    ActivationProduct,
}

/// A fission yield in atoms per fission, with its kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FissionYield {
    kind: YieldKind,
    atoms_per_fission: f64,
}

impl FissionYield {
    /// The yield must be finite and not negative.
    pub fn new(kind: YieldKind, atoms_per_fission: f64) -> Result<Self, PhysicsError> {
        non_negative("fission yield", atoms_per_fission)?;
        Ok(Self {
            kind,
            atoms_per_fission,
        })
    }

    /// The kind of yield.
    pub fn kind(&self) -> YieldKind {
        self.kind
    }

    /// Atoms per fission.
    pub fn atoms_per_fission(&self) -> f64 {
        self.atoms_per_fission
    }
}

/// Source in atoms m^-3 s^-1: `F_dot * y`. Refuses an activation product.
pub fn source_atoms_per_m3_s(
    fission_rate_density_per_m3_s: f64,
    yield_: FissionYield,
    product: ProductKind,
) -> Result<f64, PhysicsError> {
    if product == ProductKind::ActivationProduct {
        return Err(PhysicsError::DirectYieldForActivationProduct);
    }
    non_negative("fission-rate density", fission_rate_density_per_m3_s)?;
    Ok(fission_rate_density_per_m3_s * yield_.atoms_per_fission)
}

/// Source in mol m^-3 s^-1: `F_dot * y / N_A`. Refuses an activation product.
pub fn source_mol_per_m3_s(
    fission_rate_density_per_m3_s: f64,
    yield_: FissionYield,
    product: ProductKind,
) -> Result<f64, PhysicsError> {
    source_atoms_per_m3_s(fission_rate_density_per_m3_s, yield_, product).map(|s| s / AVOGADRO)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn y(kind: YieldKind, v: f64) -> FissionYield {
        FissionYield::new(kind, v).unwrap()
    }

    #[test]
    fn hand_values_and_units() {
        // F_dot = 1e19 fissions m^-3 s^-1, y = 0.05: 5e17 atoms m^-3 s^-1.
        let yy = y(YieldKind::Cumulative, 0.05);
        let a = source_atoms_per_m3_s(1.0e19, yy, ProductKind::DirectFissionProduct).unwrap();
        assert!(((a - 5.0e17) / 5.0e17).abs() < 1e-15);
        // Dividing by N_A: F_dot = N_A, y = 0.5 gives exactly 0.5 mol m^-3 s^-1.
        let s = source_mol_per_m3_s(
            AVOGADRO,
            y(YieldKind::Independent, 0.5),
            ProductKind::DirectFissionProduct,
        )
        .unwrap();
        assert!((s - 0.5).abs() < 1e-15);
        // 5e17 / 6.02214076e23 = 8.3026...e-7 mol m^-3 s^-1.
        let m = source_mol_per_m3_s(1.0e19, yy, ProductKind::DirectFissionProduct).unwrap();
        assert!(((m - 5.0e17 / 6.022_140_76e23) / m).abs() < 1e-15);
        assert!((m - 8.302_6e-7).abs() < 1e-10);
    }

    #[test]
    fn zero_source_limits() {
        let p = ProductKind::DirectFissionProduct;
        assert_eq!(
            source_mol_per_m3_s(0.0, y(YieldKind::Cumulative, 0.06), p).unwrap(),
            0.0
        );
        assert_eq!(
            source_mol_per_m3_s(1.0e19, y(YieldKind::Cumulative, 0.0), p).unwrap(),
            0.0
        );
    }

    #[test]
    fn linear_in_rate_and_yield() {
        let p = ProductKind::DirectFissionProduct;
        let base = source_mol_per_m3_s(2.0e18, y(YieldKind::Cumulative, 0.03), p).unwrap();
        let dbl_f = source_mol_per_m3_s(4.0e18, y(YieldKind::Cumulative, 0.03), p).unwrap();
        let dbl_y = source_mol_per_m3_s(2.0e18, y(YieldKind::Cumulative, 0.06), p).unwrap();
        assert!(((dbl_f - 2.0 * base) / base).abs() < 1e-15);
        assert!(((dbl_y - 2.0 * base) / base).abs() < 1e-15);
    }

    #[test]
    fn activation_product_is_refused_for_both_yield_kinds() {
        for k in [YieldKind::Independent, YieldKind::Cumulative] {
            let r = source_mol_per_m3_s(1.0e19, y(k, 1.0e-7), ProductKind::ActivationProduct);
            assert_eq!(r, Err(PhysicsError::DirectYieldForActivationProduct));
            let r = source_atoms_per_m3_s(0.0, y(k, 0.0), ProductKind::ActivationProduct);
            assert_eq!(r, Err(PhysicsError::DirectYieldForActivationProduct));
        }
    }

    #[test]
    fn invalid_inputs_are_refused() {
        let p = ProductKind::DirectFissionProduct;
        assert!(FissionYield::new(YieldKind::Cumulative, -0.1).is_err());
        assert!(FissionYield::new(YieldKind::Cumulative, f64::NAN).is_err());
        let yy = y(YieldKind::Cumulative, 0.05);
        assert!(source_mol_per_m3_s(-1.0, yy, p).is_err());
        assert!(source_mol_per_m3_s(f64::INFINITY, yy, p).is_err());
        assert_eq!(yy.kind(), YieldKind::Cumulative);
        assert_eq!(yy.atoms_per_fission(), 0.05);
    }
}
