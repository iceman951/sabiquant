//! Comparison of floating-point results (F-CORE-05; NUMERICAL_CORRECTNESS §7).
//!
//! There is no global epsilon (§7.1). Every comparison names its metric: bit
//! identity, a distance in units in the last place (ULPs), or a mixed
//! absolute and relative bound (§7.2). The tolerance policies of §7.3 choose
//! among them: TOL-E1 is `Ulps(0)` and TOL-E2 is a small `Ulps(n)`, while the
//! TOL-E3 and TOL-E4 bounds depend on the quantity and are built from these
//! metrics when the rates arrive in M4.

use std::fmt;

/// Returns the number of representable `f64` values between `a` and `b`,
/// counting one end: 0 for equal values, 1 for neighbors.
///
/// `+0.0` and `-0.0` are 0 apart. The largest finite value and infinity are 1
/// apart. Returns `None` if either argument is NaN.
pub fn ulp_distance(a: f64, b: f64) -> Option<u64> {
    if a.is_nan() || b.is_nan() {
        None
    } else {
        Some(ordered_key(a).abs_diff(ordered_key(b)))
    }
}

/// Maps a non-NaN `f64` to an integer, preserving order and adjacency. Both
/// zeros map to 0; positive values map to their bit pattern, and negative
/// values to the mirror image.
fn ordered_key(value: f64) -> i64 {
    let bits = value.to_bits().cast_signed();
    if bits < 0 { i64::MIN - bits } else { bits }
}

/// A rule for comparing a computed value with an expected one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tolerance {
    /// Identical bit patterns. For reproducibility checks (NFR-03, level R1),
    /// where even the sign of zero must not change.
    Bitwise,
    /// At most this many ULPs apart (exactness classes E1 and E2). `Ulps(0)`
    /// accepts equal values only, treating `+0.0` and `-0.0` as equal.
    Ulps(u64),
    /// `|actual - expected| <= max(absolute, relative * max(|actual|, |expected|))`
    /// (§7.2), for values near zero and quantities with a known absolute
    /// error scale. Both bounds must be finite and non-negative.
    Mixed {
        /// The absolute bound.
        absolute: f64,
        /// The relative bound.
        relative: f64,
    },
}

impl Tolerance {
    /// Compares `actual` with `expected`. On failure, returns a report with
    /// both values, their bit patterns, their ULP distance and the platform.
    ///
    /// NaN never satisfies `Ulps` or `Mixed` (NP-07).
    ///
    /// # Panics
    ///
    /// If a `Mixed` bound is negative or not finite: that is a defect in the
    /// test, not a comparison failure.
    pub fn check(self, actual: f64, expected: f64) -> Result<(), String> {
        let passed = match self {
            Self::Bitwise => actual.to_bits() == expected.to_bits(),
            Self::Ulps(limit) => ulp_distance(actual, expected).is_some_and(|d| d <= limit),
            Self::Mixed { absolute, relative } => {
                assert!(
                    absolute.is_finite()
                        && absolute >= 0.0
                        && relative.is_finite()
                        && relative >= 0.0,
                    "invalid tolerance {self:?}"
                );
                let scale = actual.abs().max(expected.abs());
                actual.is_finite()
                    && expected.is_finite()
                    && (actual - expected).abs() <= absolute.max(relative * scale)
            }
        };
        if passed {
            Ok(())
        } else {
            Err(Mismatch {
                actual,
                expected,
                tolerance: self,
            }
            .to_string())
        }
    }
}

/// Asserts that `actual` matches `expected` under `tolerance`.
///
/// # Panics
///
/// If it does not, with the report of [`Tolerance::check`].
#[track_caller]
pub fn assert_close(actual: f64, expected: f64, tolerance: Tolerance) {
    if let Err(report) = tolerance.check(actual, expected) {
        panic!("{report}");
    }
}

struct Mismatch {
    actual: f64,
    expected: f64,
    tolerance: Tolerance,
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let distance = match ulp_distance(self.actual, self.expected) {
            Some(d) => d.to_string(),
            None => "undefined (NaN)".to_owned(),
        };
        write!(
            f,
            "values differ beyond {:?}: actual {:?} ({:#018x}), expected {:?} ({:#018x}), \
             {distance} ULPs apart, on {}-{}",
            self.tolerance,
            self.actual,
            self.actual.to_bits(),
            self.expected,
            self.expected.to_bits(),
            std::env::consts::ARCH,
            std::env::consts::OS,
        )
    }
}
