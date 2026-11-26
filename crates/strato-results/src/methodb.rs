//! Method B: the Sturm-Liouville analytical solution, behind one function.
//!
//! The applicability follows the decision table of science design Section 4.3 and the drawing
//! mask of Section 4.2. The extensions for decay (B2) and for a piecewise-constant kernel source
//! (B3 to B5) are implemented by the task `analytical-extensions` on another branch. Until that
//! branch is merged, [`method_b`] answers "not available: extension pending" for every
//! configuration that needs them. **Merging the extensions changes this file only**: the arm
//! marked `EXTENSION POINT` calls the new functions, and nothing else in the pipeline asks which
//! configurations Method B can do.

use strato::analytical::sturm_liouville::{Layer, ModalRelease};

/// Seconds per hour.
const HOUR_S: f64 = 3600.0;
/// Largest number of modes of one series.
const MAX_MODES: usize = 1_000;

/// What a configuration needs from the analytical solution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Needs {
    /// A source in the kernel (stage 2, 5).
    pub generation: bool,
    /// Radioactive decay (stage 3, 5).
    pub decay: bool,
    /// A temperature that changes in time (stage 4 with a history, scenarios 2 to 5).
    pub temperature_varies: bool,
    /// The run starts from the kernel-step state (true) or from a fresh particle or another state
    /// (false).
    pub kernel_step_start: bool,
}

/// Why Method B is not shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotAvailable {
    /// Stable code of the files.
    pub code: &'static str,
    /// Text of the files and of the page.
    pub reason: String,
}

/// The series of Method B at the requested times.
#[derive(Debug, Clone)]
pub struct Available {
    /// Requested times, s.
    pub times_s: Vec<f64>,
    /// `F(t)`, `None` where the drawing mask of Section 4.2 does not accept the value.
    pub values: Vec<Option<f64>>,
    /// Estimated absolute accuracy of the series (`accuracy_floor`).
    pub accuracy_floor: f64,
    /// Smallest time at which the omitted modes are negligible (`min_valid_time`), s.
    pub valid_from_s: f64,
    /// Number of modes.
    pub modes: usize,
    /// Eigenvalue cover `alpha_max` in s^-1 (the module's `lambda_covered`).
    pub alpha_max_per_s: f64,
    /// Weight missing from the partial sum, `1 - sum w`, a diagnostic and never the mask.
    pub missing_weight: f64,
}

/// The answer.
#[derive(Debug, Clone)]
pub enum MethodB {
    /// Drawn where unmasked.
    Available(Available),
    /// Not shown.
    NotAvailable(NotAvailable),
}

/// The label of a curve that needs an extension that is not merged.
pub const EXTENSION_PENDING: &str = "not available: extension pending";

/// The applicability of Method B (decision table of Section 4.3), without computing anything.
///
/// # Errors
///
/// [`NotAvailable`] with the code and the reason.
pub fn applicability(needs: Needs) -> Result<(), NotAvailable> {
    if needs.temperature_varies {
        return Err(NotAvailable {
            code: "TIME_VARYING_TEMPERATURE",
            reason: "the temperature changes in time, so the eigenfunctions change in time; the analytical solution is not defined for this case (science design Section 4.2)".into(),
        });
    }
    if !needs.kernel_step_start {
        return Err(NotAvailable {
            code: "START_STATE_NOT_KERNEL_STEP",
            reason: "the analytical solution is derived for the kernel-step initial state; this run starts from another state".into(),
        });
    }
    // EXTENSION POINT: when `feat/analytical-extensions` is merged, the two arms below become
    // `Ok(())` for the configurations the extensions cover (E-decay for decay without a source
    // at constant temperature, E-source for a piecewise-constant kernel source at constant
    // temperature) and `method_b` calls the new functions for them.
    if needs.generation || needs.decay {
        return Err(NotAvailable {
            code: "EXTENSION_PENDING",
            reason: EXTENSION_PENDING.into(),
        });
    }
    Ok(())
}

fn layers(radii_m: &[f64; 5], d: &[f64; 5]) -> Vec<Layer> {
    (0..5)
        .map(|k| Layer {
            outer_radius: radii_m[k],
            diffusivity: d[k],
        })
        .collect()
}

/// Method B for a configuration at times in s. Exact for the baseline (and for a constant
/// temperature, with the diffusivities evaluated at it).
///
/// A value is drawn only if (1) `t >= min_valid_time()`, (2) the series with the full
/// eigenvalue cover and with the cover reduced by a factor 4 differ by at most 5 % of the full
/// value (the labelled 5 % convention of decision 49, not a derived bound), and (3) the
/// module's `accuracy_floor()` is at most 5 % of the value.
pub fn method_b(needs: Needs, radii_m: &[f64; 5], d: &[f64; 5], times_s: &[f64]) -> MethodB {
    if let Err(n) = applicability(needs) {
        return MethodB::NotAvailable(n);
    }
    let ls = layers(radii_m, d);
    let t_min = times_s
        .iter()
        .copied()
        .filter(|t| *t > 0.0)
        .fold(f64::INFINITY, f64::min);
    if !t_min.is_finite() {
        return MethodB::NotAvailable(NotAvailable {
            code: "NO_POSITIVE_TIME",
            reason: "the series needs at least one positive time".into(),
        });
    }
    // The oscillation count of the eigenvalue search takes the integer part of k L / pi in every
    // layer; it is reliable only while that number stays far below 2^53. The cover is limited so
    // that it stays below 1e9 (at extreme diffusivity contrasts, for example Sr-90 at 400 degC,
    // an unlimited cover makes the root search diverge). The limit only hides early times.
    const MAX_HALF_WAVES: f64 = 1.0e9;
    let mut left = 0.0;
    let mut cover_limit = f64::INFINITY;
    for (r, dk) in radii_m.iter().zip(d) {
        let len = r - left;
        left = *r;
        cover_limit = cover_limit.min(dk * (std::f64::consts::PI * MAX_HALF_WAVES / len).powi(2));
    }
    let lambda_max = (45.0 / t_min).min(cover_limit);
    // The cost of the weights grows with the square of the number of modes, so the number is
    // capped; the cap only moves the first drawn time (`min_valid_time`) later, never the values.
    let full = match ModalRelease::new(&ls, lambda_max, MAX_MODES) {
        Ok(s) => s,
        Err(e) => {
            return MethodB::NotAvailable(NotAvailable {
                code: "SERIES_NOT_BUILT",
                reason: format!("the modal series could not be built: {e}"),
            })
        }
    };
    // The series with the cover reduced by 4 is the same series truncated: the eigenvalues and
    // the weights of the modes do not depend on the cover.
    let cut = full.lambda_covered / 4.0;
    let quarter_value = |t: f64| -> f64 {
        1.0 - full
            .eigenvalues
            .iter()
            .zip(&full.weights)
            .filter(|(l, _)| **l <= cut)
            .map(|(l, w)| w * (-l * t).exp())
            .sum::<f64>()
    };
    let valid = full.min_valid_time();
    let floor = full.accuracy_floor();
    let values = times_s
        .iter()
        .map(|&t| {
            if t < valid {
                return None;
            }
            let v = full.release_fraction(t);
            if !(v.is_finite() && v > 0.0) || floor > 0.05 * v {
                return None;
            }
            let q = quarter_value(t);
            ((v - q).abs() <= 0.05 * v).then_some(v)
        })
        .collect::<Vec<Option<f64>>>();
    if values.iter().all(Option::is_none) {
        return MethodB::NotAvailable(NotAvailable {
            code: "NOT_DRAWN_AT_ANY_TIME",
            reason: format!(
                "the drawing mask of science design Section 4.2 accepts no value of this curve (first valid time {:.3e} s, accuracy floor {:.1e})",
                valid, floor
            ),
        });
    }
    MethodB::Available(Available {
        times_s: times_s.to_vec(),
        values,
        accuracy_floor: floor,
        valid_from_s: valid,
        modes: full.eigenvalues.len(),
        alpha_max_per_s: full.lambda_covered,
        missing_weight: 1.0 - full.weights.iter().sum::<f64>(),
    })
}

/// Hours of a time in s (helper for the files).
pub fn hours(t_s: f64) -> f64 {
    t_s / HOUR_S
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: Needs = Needs {
        generation: false,
        decay: false,
        temperature_varies: false,
        kernel_step_start: true,
    };

    #[test]
    fn the_decision_table_of_section_4_3() {
        assert!(applicability(BASE).is_ok());
        let e = applicability(Needs {
            temperature_varies: true,
            ..BASE
        })
        .unwrap_err();
        assert_eq!(e.code, "TIME_VARYING_TEMPERATURE");
        // Until the extensions are merged: decay and generation are pending.
        for needs in [
            Needs {
                decay: true,
                ..BASE
            },
            Needs {
                generation: true,
                ..BASE
            },
        ] {
            let e = applicability(needs).unwrap_err();
            assert_eq!(e.code, "EXTENSION_PENDING");
            assert_eq!(e.reason, "not available: extension pending");
        }
        // Temperature variation wins over a pending extension.
        let e = applicability(Needs {
            generation: true,
            temperature_varies: true,
            ..BASE
        })
        .unwrap_err();
        assert_eq!(e.code, "TIME_VARYING_TEMPERATURE");
        assert_eq!(
            applicability(Needs {
                kernel_step_start: false,
                ..BASE
            })
            .unwrap_err()
            .code,
            "START_STATE_NOT_KERNEL_STEP"
        );
    }

    #[test]
    fn the_mask_hides_early_values_and_keeps_late_ones() {
        let radii = [175.0e-6, 278.5e-6, 317.9e-6, 353.2e-6, 394.2e-6];
        let d = [1.0e-13, 1.0e-12, 1.0e-14, 1.0e-15, 1.0e-14];
        let times: Vec<f64> = vec![60.0, 3600.0, 36000.0, 360000.0, 1.0e6];
        match method_b(BASE, &radii, &d, &times) {
            MethodB::Available(a) => {
                assert_eq!(a.values.len(), 5);
                assert!(a.values[4].is_some_and(|v| v > 0.0 && v < 1.0));
                assert!(a.accuracy_floor > 0.0 && a.valid_from_s > 0.0);
            }
            MethodB::NotAvailable(n) => panic!("{n:?}"),
        }
    }
}
