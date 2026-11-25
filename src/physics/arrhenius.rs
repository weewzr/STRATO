//! Arrhenius diffusivity, `D(T) = sum_m D0_m exp(-Ea_m / (R T))`.
//!
//! Units: `D0` in m^2 s^-1, `Ea` in J mol^-1 (kJ mol^-1 is converted on construction),
//! `R` in J mol^-1 K^-1, `T` in K (degrees Celsius is converted on use). The temperature
//! unit and the energy unit are mandatory tags, never guessed.
//!
//! A single term is the general law; a sum of terms is the law for a row that the source
//! documents as parallel mechanisms (fluxes add, `J = -(D_a + D_b) grad c`). The caller
//! decides which rows form one sum; nothing is merged here.
//!
//! The validity range of a term is reported by [`RangeStatus`]; the diffusivity is always
//! evaluated at the requested temperature and never clamped.

use super::{finite, non_negative, positive, PhysicsError, AVOGADRO, BOLTZMANN, CELSIUS_OFFSET_K};

/// Gas constant in J mol^-1 K^-1, passed explicitly to every evaluation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GasConstant(f64);

impl GasConstant {
    /// Exact SI value `N_A k_B = 8.314462618...` J mol^-1 K^-1 (default).
    pub const SI: GasConstant = GasConstant(AVOGADRO * BOLTZMANN);

    /// A value taken from a source (for example the one a benchmark printed), so that the
    /// source can be reproduced. It must be finite and strictly positive.
    pub fn custom(r_j_mol_k: f64) -> Result<Self, PhysicsError> {
        positive("gas constant", r_j_mol_k).map(Self)
    }

    /// The value in J mol^-1 K^-1.
    pub fn value(self) -> f64 {
        self.0
    }
}

impl Default for GasConstant {
    fn default() -> Self {
        Self::SI
    }
}

/// Unit tag of an activation energy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyUnit {
    /// J mol^-1.
    JoulePerMole,
    /// kJ mol^-1 (factor 1000 to J mol^-1).
    KilojoulePerMole,
}

impl EnergyUnit {
    /// Converts an energy in this unit to J mol^-1.
    pub fn to_j_per_mol(self, e: f64) -> f64 {
        match self {
            Self::JoulePerMole => e,
            Self::KilojoulePerMole => e * 1.0e3,
        }
    }
}

/// A temperature with its unit tag.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Temperature {
    /// Kelvin.
    Kelvin(f64),
    /// Degrees Celsius, `T_K = T_C + 273.15`.
    Celsius(f64),
}

impl Temperature {
    /// The temperature in kelvin; it must be finite and strictly positive.
    pub fn to_kelvin(self) -> Result<f64, PhysicsError> {
        match self {
            Self::Kelvin(k) => positive("temperature (K)", k),
            Self::Celsius(c) => {
                finite("temperature (degC)", c)?;
                positive("temperature (K)", c + CELSIUS_OFFSET_K)
            }
        }
    }
}

/// Inclusive temperature range in kelvin over which a source states a correlation valid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TemperatureRange {
    min_k: f64,
    max_k: f64,
}

impl TemperatureRange {
    /// Builds a range; `min <= max` is required.
    pub fn new(min: Temperature, max: Temperature) -> Result<Self, PhysicsError> {
        let (min_k, max_k) = (min.to_kelvin()?, max.to_kelvin()?);
        if min_k > max_k {
            return Err(PhysicsError::EmptyTemperatureRange);
        }
        Ok(Self { min_k, max_k })
    }

    /// Lower bound in K.
    pub fn min_k(&self) -> f64 {
        self.min_k
    }

    /// Upper bound in K.
    pub fn max_k(&self) -> f64 {
        self.max_k
    }
}

/// Where a temperature sits relative to the stated validity range of a term.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeStatus {
    /// Inside the stated range (bounds included).
    Within,
    /// Below the stated minimum: an extrapolation.
    BelowMin,
    /// Above the stated maximum: an extrapolation.
    AboveMax,
    /// The source states no range, so nothing can be checked.
    NotStated,
}

/// One Arrhenius term `D0 exp(-Ea / (R T))`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrheniusTerm {
    d0_m2_s: f64,
    ea_j_mol: f64,
    range: Option<TemperatureRange>,
}

impl ArrheniusTerm {
    /// `d0` in m^2 s^-1 (finite, > 0); `ea` in the given unit (finite, >= 0; `Ea = 0` is a
    /// legitimate row, a temperature-independent diffusivity); `range` as printed by the
    /// source, or `None`.
    pub fn new(
        d0_m2_s: f64,
        ea: f64,
        unit: EnergyUnit,
        range: Option<TemperatureRange>,
    ) -> Result<Self, PhysicsError> {
        positive("D0", d0_m2_s)?;
        non_negative("activation energy", ea)?;
        let ea_j_mol = non_negative("activation energy (J/mol)", unit.to_j_per_mol(ea))?;
        Ok(Self {
            d0_m2_s,
            ea_j_mol,
            range,
        })
    }

    /// `D0` in m^2 s^-1.
    pub fn d0(&self) -> f64 {
        self.d0_m2_s
    }

    /// Activation energy in J mol^-1.
    pub fn ea_j_per_mol(&self) -> f64 {
        self.ea_j_mol
    }

    /// `ln D` of this term at `t_k` kelvin: `ln D0 - Ea / (R T)`. Finite even where
    /// `exp` of it underflows.
    fn ln_d(&self, t_k: f64, r: GasConstant) -> f64 {
        self.d0_m2_s.ln() - self.ea_j_mol / (r.value() * t_k)
    }

    /// Range report at `t_k` kelvin.
    pub fn range_status(&self, t_k: f64) -> RangeStatus {
        match self.range {
            None => RangeStatus::NotStated,
            Some(r) if t_k < r.min_k => RangeStatus::BelowMin,
            Some(r) if t_k > r.max_k => RangeStatus::AboveMax,
            Some(_) => RangeStatus::Within,
        }
    }
}

/// Result of an evaluation: the value and a per-term range report.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffusivityEvaluation {
    /// Diffusivity in m^2 s^-1 (may be 0.0 if it underflows; see `ln_d`).
    pub d_m2_s: f64,
    /// Natural logarithm of the diffusivity (D in m^2 s^-1), finite even on underflow.
    pub ln_d: f64,
    /// Range status of each term, in the order the terms were given.
    pub term_ranges: Vec<RangeStatus>,
}

impl DiffusivityEvaluation {
    /// True if any term was evaluated outside its stated range.
    pub fn is_extrapolated(&self) -> bool {
        self.term_ranges
            .iter()
            .any(|s| matches!(s, RangeStatus::BelowMin | RangeStatus::AboveMax))
    }
}

/// A sum of one or more Arrhenius terms.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrheniusLaw {
    terms: Vec<ArrheniusTerm>,
}

impl ArrheniusLaw {
    /// The single-exponential law.
    pub fn single(term: ArrheniusTerm) -> Self {
        Self { terms: vec![term] }
    }

    /// A documented sum of terms (parallel mechanisms); at least one term.
    pub fn sum(terms: Vec<ArrheniusTerm>) -> Result<Self, PhysicsError> {
        if terms.is_empty() {
            return Err(PhysicsError::NoTerms);
        }
        Ok(Self { terms })
    }

    /// The terms.
    pub fn terms(&self) -> &[ArrheniusTerm] {
        &self.terms
    }

    /// Evaluates `D(T)` with the given gas constant. The sum is formed with log-sum-exp so
    /// that a term whose `exp` underflows does not disturb the others. Out-of-range
    /// temperatures are reported in the result, never clamped.
    pub fn evaluate(
        &self,
        t: Temperature,
        r: GasConstant,
    ) -> Result<DiffusivityEvaluation, PhysicsError> {
        let t_k = t.to_kelvin()?;
        let ln_terms: Vec<f64> = self.terms.iter().map(|x| x.ln_d(t_k, r)).collect();
        let m = ln_terms.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let ln_d = m + ln_terms.iter().map(|l| (l - m).exp()).sum::<f64>().ln();
        Ok(DiffusivityEvaluation {
            d_m2_s: ln_d.exp(),
            ln_d,
            term_ranges: self.terms.iter().map(|x| x.range_status(t_k)).collect(),
        })
    }

    /// Apparent activation energy `Q_eff = sum Q_m D_m / sum D_m` in J mol^-1 at `t`
    /// (derivation, equation P22.6 of the Phase 2 derivations).
    pub fn effective_activation_energy(
        &self,
        t: Temperature,
        r: GasConstant,
    ) -> Result<f64, PhysicsError> {
        let t_k = t.to_kelvin()?;
        let ln_terms: Vec<f64> = self.terms.iter().map(|x| x.ln_d(t_k, r)).collect();
        let m = ln_terms.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let w: Vec<f64> = ln_terms.iter().map(|l| (l - m).exp()).collect();
        let num: f64 = self.terms.iter().zip(&w).map(|(x, w)| x.ea_j_mol * w).sum();
        Ok(num / w.iter().sum::<f64>())
    }
}

/// The reduced coefficient `D / r^2` in s^-1 for a physical radius `r` in m. Some sources
/// state `D` for a fixed reference radius; the caller must use one form consistently.
pub fn reduced_diffusivity(d_m2_s: f64, radius_m: f64) -> Result<f64, PhysicsError> {
    non_negative("diffusivity", d_m2_s)?;
    positive("radius", radius_m)?;
    Ok(d_m2_s / (radius_m * radius_m))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rel(a: f64, b: f64) -> f64 {
        ((a - b) / b).abs()
    }

    fn term(d0: f64, ea: f64, u: EnergyUnit) -> ArrheniusTerm {
        ArrheniusTerm::new(d0, ea, u, None).unwrap()
    }

    #[test]
    fn si_gas_constant_is_n_a_times_k_b() {
        assert!(rel(GasConstant::SI.value(), 8.314_462_618_153_24) < 1e-14);
        assert_eq!(GasConstant::default(), GasConstant::SI);
    }

    #[test]
    fn exponent_one_gives_d0_over_e() {
        // R = 8, Ea = 8000 J/mol, T = 1000 K: Ea/(R T) = 1 exactly.
        let r = GasConstant::custom(8.0).unwrap();
        let law = ArrheniusLaw::single(term(2.0e-9, 8000.0, EnergyUnit::JoulePerMole));
        let d = law.evaluate(Temperature::Kelvin(1000.0), r).unwrap();
        assert!(rel(d.d_m2_s, 2.0e-9 * (-1.0f64).exp()) < 1e-15);
        assert!(rel(d.ln_d, (2.0e-9f64).ln() - 1.0) < 1e-15);
    }

    #[test]
    fn units_kj_and_celsius_agree_with_si_inputs() {
        let r = GasConstant::custom(8.0).unwrap();
        let a = ArrheniusLaw::single(term(1.0e-8, 8.0, EnergyUnit::KilojoulePerMole));
        let b = ArrheniusLaw::single(term(1.0e-8, 8000.0, EnergyUnit::JoulePerMole));
        // 726.85 degC is 1000 K.
        let da = a.evaluate(Temperature::Celsius(726.85), r).unwrap();
        let db = b.evaluate(Temperature::Kelvin(1000.0), r).unwrap();
        assert!(rel(da.d_m2_s, db.d_m2_s) < 1e-13);
        assert!(rel(da.d_m2_s, 1.0e-8 * (-1.0f64).exp()) < 1e-13);
    }

    #[test]
    fn zero_activation_energy_is_temperature_independent() {
        let law = ArrheniusLaw::single(term(6.0e-6, 0.0, EnergyUnit::KilojoulePerMole));
        for t in [300.0, 1500.0, 3000.0] {
            let d = law
                .evaluate(Temperature::Kelvin(t), GasConstant::SI)
                .unwrap();
            assert_eq!(d.d_m2_s, 6.0e-6);
        }
    }

    #[test]
    fn high_temperature_limit_is_d0() {
        let law = ArrheniusLaw::single(term(3.0e-7, 200.0, EnergyUnit::KilojoulePerMole));
        let d = law
            .evaluate(Temperature::Kelvin(1.0e15), GasConstant::SI)
            .unwrap();
        // Relative deviation is about Ea / (R T) = 2.4e-11.
        assert!(rel(d.d_m2_s, 3.0e-7) < 1e-10);
    }

    #[test]
    fn sum_of_two_equal_terms_doubles() {
        let r = GasConstant::custom(8.0).unwrap();
        let t = term(1.0, 8000.0, EnergyUnit::JoulePerMole);
        let law = ArrheniusLaw::sum(vec![t, t]).unwrap();
        let d = law.evaluate(Temperature::Kelvin(1000.0), r).unwrap();
        assert!(rel(d.d_m2_s, 2.0 * (-1.0f64).exp()) < 1e-15);
    }

    #[test]
    fn sum_of_unequal_terms_is_the_plain_sum() {
        // Term 1: exponent 1; term 2: D0 = 3, Ea = 16000 J/mol, exponent 2.
        let r = GasConstant::custom(8.0).unwrap();
        let law = ArrheniusLaw::sum(vec![
            term(1.0, 8000.0, EnergyUnit::JoulePerMole),
            term(3.0, 16.0, EnergyUnit::KilojoulePerMole),
        ])
        .unwrap();
        let d = law.evaluate(Temperature::Kelvin(1000.0), r).unwrap();
        let expect = (-1.0f64).exp() + 3.0 * (-2.0f64).exp();
        assert!(rel(d.d_m2_s, expect) < 1e-14);
    }

    #[test]
    fn effective_energy_is_d_weighted_mean_and_equals_q_for_one_term() {
        let r = GasConstant::custom(8.0).unwrap();
        let law = ArrheniusLaw::sum(vec![
            term(1.0, 8000.0, EnergyUnit::JoulePerMole),
            term(3.0, 16000.0, EnergyUnit::JoulePerMole),
        ])
        .unwrap();
        let (w1, w2) = ((-1.0f64).exp(), 3.0 * (-2.0f64).exp());
        let expect = (8000.0 * w1 + 16000.0 * w2) / (w1 + w2);
        let q = law
            .effective_activation_energy(Temperature::Kelvin(1000.0), r)
            .unwrap();
        assert!(rel(q, expect) < 1e-14);
        let one = ArrheniusLaw::single(term(1.0, 8000.0, EnergyUnit::JoulePerMole));
        let q1 = one
            .effective_activation_energy(Temperature::Kelvin(500.0), r)
            .unwrap();
        assert!(rel(q1, 8000.0) < 1e-15);
    }

    #[test]
    fn gas_constant_choice_changes_d_as_the_derivation_says() {
        // D(R') / D(R) = exp[(Q / T)(1/R - 1/R')] for one term.
        let law = ArrheniusLaw::single(term(1.0, 100.0, EnergyUnit::KilojoulePerMole));
        let r2 = GasConstant::custom(8.3143).unwrap();
        let t = Temperature::Kelvin(1273.15);
        let d1 = law.evaluate(t, GasConstant::SI).unwrap().d_m2_s;
        let d2 = law.evaluate(t, r2).unwrap().d_m2_s;
        let expect = ((1.0e5 / 1273.15) * (1.0 / GasConstant::SI.value() - 1.0 / 8.3143)).exp();
        assert!(rel(d2 / d1, expect) < 1e-12);
    }

    #[test]
    fn range_is_reported_and_value_is_not_clamped() {
        let rng = TemperatureRange::new(Temperature::Celsius(1400.0), Temperature::Celsius(1720.0))
            .unwrap();
        let t1 =
            ArrheniusTerm::new(1.0e-9, 100.0, EnergyUnit::KilojoulePerMole, Some(rng)).unwrap();
        let law = ArrheniusLaw::single(t1);
        let at = |c: f64| {
            law.evaluate(Temperature::Celsius(c), GasConstant::SI)
                .unwrap()
        };
        assert_eq!(at(1500.0).term_ranges, vec![RangeStatus::Within]);
        assert_eq!(at(1400.0).term_ranges, vec![RangeStatus::Within]);
        assert_eq!(at(1720.0).term_ranges, vec![RangeStatus::Within]);
        assert_eq!(at(1399.0).term_ranges, vec![RangeStatus::BelowMin]);
        assert_eq!(at(1800.0).term_ranges, vec![RangeStatus::AboveMax]);
        assert!(at(1800.0).is_extrapolated() && !at(1500.0).is_extrapolated());
        // Outside the range the value is the same formula, not the boundary value.
        let outside = at(1800.0).d_m2_s;
        let expect = 1.0e-9 * (-1.0e5 / (GasConstant::SI.value() * (1800.0 + 273.15))).exp();
        assert!(rel(outside, expect) < 1e-13);
        // No range stated.
        let nr = ArrheniusLaw::single(term(1.0, 1.0, EnergyUnit::JoulePerMole));
        let e = nr
            .evaluate(Temperature::Kelvin(300.0), GasConstant::SI)
            .unwrap();
        assert_eq!(e.term_ranges, vec![RangeStatus::NotStated]);
        assert!(!e.is_extrapolated());
    }

    #[test]
    fn underflow_keeps_ln_d_finite() {
        // Ea = 923 kJ/mol at 100 K: exp underflows, ln D stays finite (about -1110.1 for D0 = 1).
        let law = ArrheniusLaw::single(term(1.0, 923.0, EnergyUnit::KilojoulePerMole));
        let d = law
            .evaluate(Temperature::Kelvin(100.0), GasConstant::SI)
            .unwrap();
        assert_eq!(d.d_m2_s, 0.0);
        assert!(rel(d.ln_d, -923.0e3 / (GasConstant::SI.value() * 100.0)) < 1e-14);
    }

    #[test]
    fn invalid_inputs_are_refused() {
        let ok = EnergyUnit::JoulePerMole;
        assert_eq!(
            ArrheniusTerm::new(0.0, 1.0, ok, None),
            Err(PhysicsError::NotPositive("D0"))
        );
        assert!(ArrheniusTerm::new(-1.0, 1.0, ok, None).is_err());
        assert!(ArrheniusTerm::new(f64::NAN, 1.0, ok, None).is_err());
        assert!(ArrheniusTerm::new(1.0, -1.0, ok, None).is_err());
        assert!(ArrheniusTerm::new(1.0, f64::INFINITY, ok, None).is_err());
        assert_eq!(ArrheniusLaw::sum(vec![]), Err(PhysicsError::NoTerms));
        assert!(GasConstant::custom(0.0).is_err());
        assert!(GasConstant::custom(f64::NAN).is_err());
        assert!(Temperature::Kelvin(0.0).to_kelvin().is_err());
        assert!(Temperature::Celsius(-273.15).to_kelvin().is_err());
        assert!(Temperature::Celsius(-300.0).to_kelvin().is_err());
        assert!(Temperature::Kelvin(f64::NAN).to_kelvin().is_err());
        let a = Temperature::Kelvin(2.0);
        let b = Temperature::Kelvin(1.0);
        assert_eq!(
            TemperatureRange::new(a, b),
            Err(PhysicsError::EmptyTemperatureRange)
        );
        let law = ArrheniusLaw::single(term(1.0, 1.0, ok));
        assert!(law
            .evaluate(Temperature::Kelvin(-5.0), GasConstant::SI)
            .is_err());
    }

    #[test]
    fn reduced_diffusivity_units() {
        // 1e-18 m2/s over a 250 um radius: 1e-18 / 6.25e-8 = 1.6e-11 s^-1.
        let v = reduced_diffusivity(1.0e-18, 250.0e-6).unwrap();
        assert!(rel(v, 1.6e-11) < 1e-14);
        assert!(reduced_diffusivity(1.0, 0.0).is_err());
        assert!(reduced_diffusivity(-1.0, 1.0).is_err());
    }
}
