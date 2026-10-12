//! Checked normal probabilities, tails and quantiles with explicit location/scale.
//!
//! A positive center series and a bounded Mills-ratio continued fraction are
//! implemented independently from the mathematical identities described in
//! the [local research contract](https://github.com/iceman951/sabiquant/blob/main/docs/M6_NORMAL_RESEARCH.md).
//! Small probabilities are computed directly; logarithmic tails remain useful
//! after probability underflow. Inverse probabilities must lie strictly inside
//! `(0, 1)`, including the smallest positive binary64 probability.
//!
//! Probability zero/one and density underflow to zero are meaningful rounded
//! values. Density overflow, nonfinite inputs and unrepresentable logarithmic
//! results are errors. Elementary precision is platform dependent; these
//! functions do not promise correctly rounded results or bitwise reproducibility.

use std::error::Error as StdError;
use std::fmt;
use std::ops::Bound;

use super::sum::exact_sum;
use crate::error::{Interval, ValueError};
use crate::numeric;

const INVERSE_SQRT_TWO_PI: f64 = 0.398_942_280_401_432_7;
const SQRT_TWO_PI: f64 = 2.506_628_274_631_000_7;
const LOG_SQRT_TWO_PI: f64 = 0.918_938_533_204_672_8;
const CENTER_LIMIT: f64 = 1.5;
const CONTINUED_FRACTION_DEPTH: u32 = 256;
const CENTER_ITERATION_LIMIT: u32 = 128;
const INVERSE_ITERATION_LIMIT: u32 = 256;
const UNIT_ROUNDOFF: f64 = f64::EPSILON / 2.0;
// MAX's overflow midpoint is MAX+2^970. Quarter scaling keeps its exact
// offset representable while product cancellation stays finite near it.
const QUARTER_OVERFLOW_OFFSET: f64 = f64::from_bits((1023 + 968) << 52);
const PROBABILITY_DOMAIN: Interval = Interval::new(Bound::Excluded(0.0), Bound::Excluded(1.0));
const PROBABILITY_RESULT: Interval = Interval::new(Bound::Included(0.0), Bound::Included(1.0));
const DENSITY_RESULT: Interval = Interval::new(Bound::Included(0.0), Bound::Unbounded);
const LOG_PROBABILITY_RESULT: Interval = Interval::new(Bound::Unbounded, Bound::Included(0.0));

/// Evaluates the standard normal probability density at finite `x`.
///
/// Far-tail density may round to positive zero.
///
/// # Errors
/// [`NormalError::Value`] if `x` is nonfinite.
pub fn standard_normal_pdf(x: f64) -> Result<f64, NormalError> {
    let x = numeric::finite("normal variate", x)?;
    Ok(numeric::result_within(
        "normal density",
        standard_density(x),
        DENSITY_RESULT,
    )?)
}

/// Evaluates the standard normal cumulative probability at finite `x`.
///
/// The median is exactly one half. Rounded probabilities may be zero or one.
///
/// # Errors
/// [`NormalError::Value`] if `x` is nonfinite, or
/// [`NormalError::Convergence`] if the bounded center series does not converge.
pub fn standard_normal_cdf(x: f64) -> Result<f64, NormalError> {
    let x = numeric::finite("normal variate", x)?;
    cumulative_probability(x)
}

/// Evaluates the standard normal survival probability `1-Phi(x)` directly.
///
/// Computing a small upper tail directly avoids cancellation from subtracting
/// a rounded cumulative probability from one.
///
/// # Errors
/// [`NormalError::Value`] if `x` is nonfinite, or
/// [`NormalError::Convergence`] if the bounded center series does not converge.
pub fn standard_normal_survival(x: f64) -> Result<f64, NormalError> {
    let x = numeric::finite("normal variate", x)?;
    cumulative_probability(-x)
}

/// Evaluates the natural logarithm of the standard cumulative probability.
///
/// Negative-tail logarithms remain useful after probability underflow.
/// A rounded logarithm of one is canonical positive zero.
///
/// # Errors
/// [`NormalError::Value`] for nonfinite input or an unrepresentable logarithm;
/// [`NormalError::Convergence`] if the bounded center series does not converge.
pub fn standard_normal_log_cdf(x: f64) -> Result<f64, NormalError> {
    let x = numeric::finite("normal variate", x)?;
    cumulative_logarithm(x)
}

/// Evaluates the natural logarithm of standard survival probability directly.
///
/// # Errors
/// [`NormalError::Value`] for nonfinite input or an unrepresentable logarithm;
/// [`NormalError::Convergence`] if the bounded center series does not converge.
pub fn standard_normal_log_survival(x: f64) -> Result<f64, NormalError> {
    let x = numeric::finite("normal variate", x)?;
    cumulative_logarithm(-x)
}

/// Returns the standard normal quantile for finite `0 < probability < 1`.
///
/// The smallest positive binary64 probability is accepted. Probability one
/// half returns exactly positive zero. Centered mass is solved near one half;
/// the logarithmic smaller tail is solved elsewhere, without rounded-CDF loss.
///
/// ```
/// use sabiquant::math::{standard_normal_cdf, standard_normal_inverse_cdf};
/// let z = standard_normal_inverse_cdf(0.975)?;
/// assert!((z - 1.959_963_984_540_054).abs() < 1e-14);
/// assert!((standard_normal_cdf(z)? - 0.975).abs() < 1e-14);
/// # Ok::<(), sabiquant::math::NormalError>(())
/// ```
///
/// # Errors
/// [`NormalError::Value`] for nonfinite input or a probability outside `(0,1)`;
/// [`NormalError::Convergence`] if the bounded inverse iteration does not converge.
#[expect(
    clippy::float_cmp,
    reason = "the exactly supplied median is an identity"
)]
pub fn standard_normal_inverse_cdf(probability: f64) -> Result<f64, NormalError> {
    let probability = numeric::within("normal probability", probability, PROBABILITY_DOMAIN)?;
    if probability == 0.5 {
        return Ok(0.0);
    }
    let below_median = probability < 0.5;
    let magnitude = if (0.25..=0.75).contains(&probability) {
        inverse_center((probability - 0.5).abs())?
    } else {
        let tail = if below_median {
            probability
        } else {
            1.0 - probability
        };
        inverse_tail(tail.ln())?
    };
    Ok(numeric::finite_result(
        "normal quantile",
        if below_median { -magnitude } else { magnitude },
    )?)
}

/// An immutable normal distribution with finite mean and positive finite scale.
///
/// Location and scale are explicit, with no default. The scale is a standard
/// deviation, not a variance. Equality compares supplied parameters; it is
/// not a claim about exact equality of separately computed probabilities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalDistribution {
    mean: f64,
    standard_deviation: f64,
}

impl NormalDistribution {
    /// Validates a finite mean and a strictly positive finite standard deviation.
    ///
    /// ```
    /// use sabiquant::math::NormalDistribution;
    /// let distribution = NormalDistribution::new(100.0, 15.0)?;
    /// assert_eq!(distribution.cdf(100.0)?, 0.5);
    /// assert_eq!(distribution.inverse_cdf(0.5)?, 100.0);
    /// # Ok::<(), sabiquant::math::NormalError>(())
    /// ```
    ///
    /// # Errors
    /// [`NormalError::Value`] if either parameter is nonfinite, or if the
    /// standard deviation is zero or negative.
    pub fn new(mean: f64, standard_deviation: f64) -> Result<Self, NormalError> {
        let mean = numeric::finite("normal mean", mean)?;
        let standard_deviation = numeric::within(
            "normal standard deviation",
            standard_deviation,
            Interval::greater_than(0.0),
        )?;
        Ok(Self {
            mean: canonical_zero(mean),
            standard_deviation,
        })
    }

    /// Returns the validated mean, with canonical positive zero.
    #[must_use]
    pub const fn mean(self) -> f64 {
        self.mean
    }

    /// Returns the validated, positive finite standard deviation.
    #[must_use]
    pub const fn standard_deviation(self) -> f64 {
        self.standard_deviation
    }

    /// Evaluates this distribution's density, combining scale in log space.
    ///
    /// Combining the logarithm before exponentiation preserves finite densities
    /// that would be lost by dividing an already-underflowed standard density.
    /// Far tails may round to zero; an overflowing density is an error.
    ///
    /// # Errors
    /// [`NormalError::Value`] for nonfinite `x` or an overflowing density;
    /// [`NormalError::NumericalBoundary`] near an unresolved density overflow boundary.
    pub fn pdf(self, x: f64) -> Result<f64, NormalError> {
        let x = numeric::finite("normal variate", x)?;
        let z = standardized(x, self.mean, self.standard_deviation);
        let square_term = (-0.5 * z) * z;
        if square_term.is_infinite() {
            return Ok(0.0);
        }
        let log_scale = self.standard_deviation.ln();
        let logarithm = (square_term - log_scale) - LOG_SQRT_TWO_PI;
        let uncertainty_scale = 1.0 + square_term.abs() + log_scale.abs() + LOG_SQRT_TWO_PI;
        if (logarithm - f64::MAX.ln()).abs() <= 8.0 * UNIT_ROUNDOFF * uncertainty_scale {
            return Err(NormalError::NumericalBoundary {
                quantity: "normal density",
                logarithm,
            });
        }
        Ok(canonical_zero(numeric::result_within(
            "normal density",
            logarithm.exp(),
            DENSITY_RESULT,
        )?))
    }

    /// Evaluates this distribution's cumulative probability at finite `x`.
    ///
    /// # Errors
    /// [`NormalError::Value`] for nonfinite input;
    /// [`NormalError::Convergence`] if the bounded center series does not converge.
    pub fn cdf(self, x: f64) -> Result<f64, NormalError> {
        let x = numeric::finite("normal variate", x)?;
        cumulative_probability(standardized(x, self.mean, self.standard_deviation))
    }

    /// Evaluates this distribution's survival probability directly.
    ///
    /// # Errors
    /// [`NormalError::Value`] for nonfinite input;
    /// [`NormalError::Convergence`] if the bounded center series does not converge.
    pub fn survival(self, x: f64) -> Result<f64, NormalError> {
        let x = numeric::finite("normal variate", x)?;
        cumulative_probability(-standardized(x, self.mean, self.standard_deviation))
    }

    /// Evaluates the logarithm of this distribution's cumulative probability.
    ///
    /// # Errors
    /// [`NormalError::Value`] for nonfinite input or an unrepresentable logarithm;
    /// [`NormalError::Convergence`] if the bounded center series does not converge.
    pub fn log_cdf(self, x: f64) -> Result<f64, NormalError> {
        let x = numeric::finite("normal variate", x)?;
        cumulative_logarithm(standardized(x, self.mean, self.standard_deviation))
    }

    /// Evaluates the logarithm of this distribution's survival probability.
    ///
    /// # Errors
    /// [`NormalError::Value`] for nonfinite input or an unrepresentable logarithm;
    /// [`NormalError::Convergence`] if the bounded center series does not converge.
    pub fn log_survival(self, x: f64) -> Result<f64, NormalError> {
        let x = numeric::finite("normal variate", x)?;
        cumulative_logarithm(-standardized(x, self.mean, self.standard_deviation))
    }

    /// Returns this distribution's quantile for finite `0 < probability < 1`.
    ///
    /// The affine result uses fused arithmetic to retain finite cancellation.
    /// Probability one half returns the supplied mean exactly.
    ///
    /// # Errors
    /// [`NormalError::Value`] for invalid probability or result overflow;
    /// [`NormalError::NumericalBoundary`] near unresolved affine overflow;
    /// [`NormalError::Convergence`] if the bounded standard inverse does not converge.
    pub fn inverse_cdf(self, probability: f64) -> Result<f64, NormalError> {
        let z = standard_normal_inverse_cdf(probability)?;
        // The median identity is exact and has no elementary uncertainty.
        if z == 0.0 {
            return Ok(self.mean);
        }
        let value = self.standard_deviation.mul_add(z, self.mean);
        guard_affine_quantile(probability, z, self.mean, self.standard_deviation, value)?;
        Ok(canonical_zero(numeric::finite_result(
            "normal quantile",
            value,
        )?))
    }
}

fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

/// Compare a quarter-scaled exact supplied-product sum with the actual
/// rounding overflow midpoint, retaining its low-order offset. Approximating
/// the boundary merely as MAX, or scaling mean/MAX, would reject MAX means
/// with harmless tiny scales. Correctly rounded exact_sum avoids that trap.
fn guard_affine_quantile(
    probability: f64,
    z: f64,
    mean: f64,
    scale: f64,
    value: f64,
) -> Result<(), NormalError> {
    let quarter_z = z * 0.25;
    let product = scale * quarter_z;
    if !product.is_finite() {
        // A quarter product that overflows cannot be rescued by finite mean:
        // the original product exceeds four MAX while |mean|<=MAX.
        return Ok(());
    }
    let residual = scale.mul_add(quarter_z, -product);
    let sign = if value.is_sign_negative() { -1.0 } else { 1.0 };
    let gap = exact_sum(&[
        product,
        residual,
        mean * 0.25,
        -sign * (f64::MAX * 0.25),
        -sign * QUARTER_OVERFLOW_OFFSET,
    ]);
    let Ok(gap) = gap else {
        // An overflowing gap is far beyond the finite boundary; the original
        // fused result supplies the public numeric error below. All five
        // supplied values here are finite by construction.
        return Ok(());
    };
    let magnitude = z.abs();
    let standard_allowance = if (0.25..=0.75).contains(&probability) {
        16.0 * UNIT_ROUNDOFF * magnitude
    } else {
        let log_tail = probability.min(1.0 - probability).ln().abs();
        8.0 * UNIT_ROUNDOFF * (1.0 + magnitude + log_tail / magnitude)
    };
    // Subnormal quarter products/means and the final rounded exact sum can
    // lose an absolute fraction of MIN_SUB. This floor also retains zero
    // allowances at tiny scale; it is negligible at genuine MAX boundaries.
    let allowance = (scale * 0.25) * standard_allowance
        + 2.0 * UNIT_ROUNDOFF * gap.abs()
        + 2.0 * f64::from_bits(1);
    if gap.abs() <= allowance {
        let quarter_value = scale.mul_add(quarter_z, mean * 0.25);
        return Err(NormalError::NumericalBoundary {
            quantity: "normal quantile",
            logarithm: quarter_value.abs().ln() + std::f64::consts::LN_2 * 2.0,
        });
    }
    Ok(())
}

/// Finite public inputs can produce an infinite internal standardized value;
/// this still has meaningful rounded probabilities and density zero.
fn standardized(x: f64, mean: f64, scale: f64) -> f64 {
    let difference = x - mean;
    if difference.is_finite() {
        difference / scale
    } else {
        // Overflow of subtraction requires opposite signs. Dividing before
        // subtracting then loses no cancellation and can recover finite z.
        (x / scale) - (mean / scale)
    }
}

fn standard_density(x: f64) -> f64 {
    // Multiplication by one half precedes the square to postpone overflow.
    // An infinite internal square legitimately produces tail density zero.
    ((-0.5 * x) * x).exp() * INVERSE_SQRT_TWO_PI
}

/// The real positive series has an explicit omitted-tail bound. Only inputs
/// below CENTER_LIMIT reach it, so every intermediate remains finite.
fn centered_probability(x: f64) -> Result<f64, NormalError> {
    if x == 0.0 {
        return Ok(0.0);
    }
    let square = x * x;
    let mut term = x;
    let mut sum = x;
    let mut correction = 0.0;
    for iteration in 1..=CENTER_ITERATION_LIMIT {
        let divisor = f64::from(2 * iteration + 1);
        term *= square / divisor;
        let next = sum + term;
        correction += if sum.abs() >= term.abs() {
            (sum - next) + term
        } else {
            (term - next) + sum
        };
        sum = next;
        let remaining_ratio = square / f64::from(2 * iteration + 3);
        let remaining_bound = term * (remaining_ratio / (1.0 - remaining_ratio));
        if remaining_ratio < 1.0 && remaining_bound <= (UNIT_ROUNDOFF / 8.0) * (sum + correction) {
            return Ok(standard_density(x) * (sum + correction));
        }
    }
    Err(NormalError::Convergence {
        quantity: "normal centered probability",
        iterations: CENTER_ITERATION_LIMIT,
    })
}

fn mills_ratio(x: f64) -> f64 {
    let mut denominator_tail = 0.0;
    for numerator in (1..=CONTINUED_FRACTION_DEPTH).rev() {
        denominator_tail = f64::from(numerator) / (x + denominator_tail);
    }
    1.0 / (x + denominator_tail)
}

fn smaller_tail(x: f64) -> Result<f64, NormalError> {
    if x < CENTER_LIMIT {
        Ok(0.5 - centered_probability(x)?)
    } else if x >= 40.0 {
        // Q(40) < phi(40)/40 < MIN_SUB/2, from the first Mills bound.
        Ok(0.0)
    } else {
        Ok(standard_density(x) * mills_ratio(x))
    }
}

fn cumulative_probability(x: f64) -> Result<f64, NormalError> {
    let tail = smaller_tail(x.abs())?;
    Ok(numeric::result_within(
        "normal probability",
        if x <= 0.0 { tail } else { 1.0 - tail },
        PROBABILITY_RESULT,
    )?)
}

fn smaller_log_tail(x: f64) -> Result<f64, NormalError> {
    if x < CENTER_LIMIT {
        Ok((0.5 - centered_probability(x)?).ln())
    } else {
        let logarithm = (((-0.5 * x) * x) - LOG_SQRT_TWO_PI) + mills_ratio(x).ln();
        Ok(numeric::finite_result(
            "normal logarithmic tail",
            logarithm,
        )?)
    }
}

fn cumulative_logarithm(x: f64) -> Result<f64, NormalError> {
    let logarithm = if x <= 0.0 {
        smaller_log_tail(x.abs())?
    } else {
        (-smaller_tail(x)?).ln_1p()
    };
    Ok(canonical_zero(numeric::result_within(
        "normal logarithmic probability",
        logarithm,
        LOG_PROBABILITY_RESULT,
    )?))
}

/// Center Newton residuals retain small mass erased by a rounded CDF plateau.
#[expect(
    clippy::float_cmp,
    reason = "equal iterates identify the binary64 precision limit"
)]
fn inverse_center(mass: f64) -> Result<f64, NormalError> {
    let mut lower = 0.0;
    let mut upper = 1.0;
    let mut x = mass * SQRT_TWO_PI;
    for _ in 0..INVERSE_ITERATION_LIMIT {
        let residual = centered_probability(x)? - mass;
        if residual == 0.0 {
            return Ok(x);
        }
        if residual < 0.0 {
            lower = x;
        } else {
            upper = x;
        }
        let proposed = x - residual / standard_density(x);
        if proposed == x {
            return Ok(x);
        }
        let next = if proposed > lower && proposed < upper {
            proposed
        } else {
            lower + (upper - lower) * 0.5
        };
        if next == lower || next == upper {
            return Ok(x);
        }
        x = next;
    }
    Err(NormalError::Convergence {
        quantity: "normal inverse probability",
        iterations: INVERSE_ITERATION_LIMIT,
    })
}

#[expect(
    clippy::float_cmp,
    reason = "equal iterates identify the binary64 precision limit"
)]
fn inverse_tail(log_probability: f64) -> Result<f64, NormalError> {
    let mut lower = 0.0;
    let mut upper = 40.0;
    let mut x = (-2.0 * log_probability).sqrt();
    for _ in 0..INVERSE_ITERATION_LIMIT {
        let residual = smaller_log_tail(x)? - log_probability;
        if residual == 0.0 {
            return Ok(x);
        }
        if residual > 0.0 {
            lower = x;
        } else {
            upper = x;
        }
        let ratio = if x < CENTER_LIMIT {
            smaller_tail(x)? / standard_density(x)
        } else {
            mills_ratio(x)
        };
        let proposed = residual.mul_add(ratio, x);
        if proposed == x {
            return Ok(x);
        }
        let next = if proposed > lower && proposed < upper {
            proposed
        } else {
            lower + (upper - lower) * 0.5
        };
        if next == lower || next == upper {
            return Ok(x);
        }
        x = next;
    }
    Err(NormalError::Convergence {
        quantity: "normal inverse probability",
        iterations: INVERSE_ITERATION_LIMIT,
    })
}

/// A rejected normal input, numerical boundary or bounded iteration.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum NormalError {
    /// Numeric validation failed; kind `normal.value`, preserving its source.
    Value(ValueError),
    /// Elementary uncertainty straddles a density or affine-quantile boundary.
    #[non_exhaustive]
    NumericalBoundary {
        /// Density or affine quantile whose overflow classification is unresolved.
        quantity: &'static str,
        /// Density's computed logarithm, or log magnitude of the scaled quantile.
        logarithm: f64,
    },
    /// A fixed iteration limit was reached; kind `normal.convergence`.
    #[non_exhaustive]
    Convergence {
        /// Requested quantity.
        quantity: &'static str,
        /// Maximum iterations attempted.
        iterations: u32,
    },
}

impl NormalError {
    /// Returns the stable `normal.<variant>` kind name.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Value(_) => "normal.value",
            Self::NumericalBoundary { .. } => "normal.numerical_boundary",
            Self::Convergence { .. } => "normal.convergence",
        }
    }
}

impl fmt::Display for NormalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(error) => error.fmt(f),
            Self::NumericalBoundary {
                quantity,
                logarithm,
            } => write!(
                f,
                "{quantity} overflow is unresolved near a binary64 boundary (logarithm {logarithm})"
            ),
            Self::Convergence {
                quantity,
                iterations,
            } => write!(
                f,
                "{quantity} did not converge within {iterations} iterations"
            ),
        }
    }
}

impl StdError for NormalError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Value(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ValueError> for NormalError {
    fn from(value: ValueError) -> Self {
        Self::Value(value)
    }
}
