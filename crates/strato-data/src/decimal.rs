//! Exact decimal arithmetic for the geometry bookkeeping.
//!
//! `scripts/web/generate_data.py` compares the registered outer radii with sums of the printed
//! thicknesses in exact decimal arithmetic (`decimal.Decimal`). This type does the same for the
//! plain decimals of `geometry.csv` (no exponent), so the comparison is exact and not a tolerance.

use std::cmp::Ordering;

/// `mantissa * 10^-scale`.
#[derive(Debug, Clone, Copy)]
pub struct Decimal {
    mantissa: i128,
    scale: u32,
}

impl Decimal {
    /// Parse a plain decimal such as `103.5`, `-2`, `350`. Exponents are rejected.
    pub fn parse(text: &str) -> Result<Decimal, String> {
        let (neg, body) = match text.strip_prefix('-') {
            Some(b) => (true, b),
            None => (false, text),
        };
        let (int, frac) = match body.split_once('.') {
            Some((i, f)) => (i, f),
            None => (body, ""),
        };
        let digits_ok = |s: &str| s.chars().all(|c| c.is_ascii_digit());
        if int.is_empty()
            || !digits_ok(int)
            || !digits_ok(frac)
            || frac.len() > 30
            || int.len() > 30
        {
            return Err(format!("`{text}` is not a plain decimal"));
        }
        let all = format!("{int}{frac}");
        let m: i128 = all
            .parse()
            .map_err(|_| format!("`{text}` is too long for exact arithmetic"))?;
        Ok(Decimal {
            mantissa: if neg { -m } else { m },
            scale: frac.len() as u32,
        })
    }

    /// Build from an integer.
    pub fn from_int(v: i64) -> Decimal {
        Decimal {
            mantissa: v as i128,
            scale: 0,
        }
    }

    fn at_scale(&self, scale: u32) -> i128 {
        self.mantissa * 10i128.pow(scale - self.scale)
    }

    /// Exact half: `x / 2`.
    pub fn half(self) -> Decimal {
        Decimal {
            mantissa: self.mantissa * 5,
            scale: self.scale + 1,
        }
    }

    /// The nearest `f64`, for reporting only.
    pub fn to_f64(self) -> f64 {
        self.mantissa as f64 / 10f64.powi(self.scale as i32)
    }
}

impl std::ops::Add for Decimal {
    type Output = Decimal;
    fn add(self, other: Decimal) -> Decimal {
        let s = self.scale.max(other.scale);
        Decimal {
            mantissa: self.at_scale(s) + other.at_scale(s),
            scale: s,
        }
    }
}

impl std::ops::Sub for Decimal {
    type Output = Decimal;
    fn sub(self, other: Decimal) -> Decimal {
        let s = self.scale.max(other.scale);
        Decimal {
            mantissa: self.at_scale(s) - other.at_scale(s),
            scale: s,
        }
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Decimal {}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> Ordering {
        let s = self.scale.max(other.scale);
        self.at_scale(s).cmp(&other.at_scale(s))
    }
}

impl std::fmt::Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_sums_do_not_suffer_binary_rounding() {
        let a = Decimal::parse("0.1").unwrap();
        let b = Decimal::parse("0.2").unwrap();
        assert_eq!(a + b, Decimal::parse("0.3").unwrap());
        assert_ne!(0.1f64 + 0.2f64, 0.3f64);
    }

    #[test]
    fn agr1_outer_radius_sum() {
        let k = Decimal::parse("350").unwrap().half();
        let r = [("103.5"), ("39.4"), ("35.3"), ("41.0")]
            .iter()
            .fold(k, |acc, t| acc + Decimal::parse(t).unwrap());
        assert_eq!(r, Decimal::parse("394.2").unwrap());
        assert_eq!(r.half() + r.half(), r);
    }

    #[test]
    fn rejects_exponents_and_words() {
        assert!(Decimal::parse("1e3").is_err());
        assert!(Decimal::parse("abc").is_err());
        assert!(Decimal::parse("").is_err());
        assert!(Decimal::parse("-2.50").unwrap() < Decimal::from_int(-2));
    }
}
