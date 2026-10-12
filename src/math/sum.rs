//! Checked compensated and fixed-point exact reductions.

use crate::error::{NonFinite, ValueError};
use crate::numeric;
use std::error::Error as StdError;
use std::fmt;

/// A rejected summation input or arithmetic result.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SumError {
    /// A slice element is not finite; kind `sum.input`.
    #[non_exhaustive]
    Input {
        /// The zero-based position of the rejected element.
        index: usize,
        /// The underlying numeric input error.
        error: ValueError,
    },
    /// An intermediate or final result overflowed; kind `sum.arithmetic`.
    #[non_exhaustive]
    Arithmetic {
        /// The current element, or `None` for the final reduction.
        index: Option<usize>,
        /// The underlying numeric result error.
        error: ValueError,
    },
}

impl SumError {
    /// Returns the stable error kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Input { .. } => "sum.input",
            Self::Arithmetic { .. } => "sum.arithmetic",
        }
    }
}

impl fmt::Display for SumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input { index, error } => write!(f, "summation element {index}: {error}"),
            Self::Arithmetic {
                index: Some(index),
                error,
            } => {
                write!(f, "summation at element {index}: {error}")
            }
            Self::Arithmetic { index: None, error } => write!(f, "summation result: {error}"),
        }
    }
}

impl StdError for SumError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Input { error, .. } | Self::Arithmetic { error, .. } => Some(error),
        }
    }
}

/// Sums finite inputs in supplied order with Neumaier compensation.
///
/// The correction retains rounding errors when a term is larger than the
/// running sum as well as when it is smaller. The empty sum and every zero
/// result are positive zero. This is not a correctly rounded or permutation
/// invariant reduction; use [`exact_sum`] when those properties are required.
/// It uses O(n) time and O(1) memory.
/// The correction rule follows method IV, equation IV.1 of
/// [Neumaier (1974, p. 40)](https://arnold-neumaier.at/scan/01.pdf).
///
/// # Errors
///
/// Returns [`SumError::Input`] for the first non-finite element, checked before
/// arithmetic begins. Returns [`SumError::Arithmetic`] when a partial sum,
/// correction or final result is not finite. Later cancellation does not
/// recover an overflowed partial sum.
///
/// # Examples
///
/// ```
/// use sabiquant::math::compensated_sum;
/// let sum = compensated_sum(&[1e16, 1.0, -1e16, 1.0])?;
/// assert_eq!(sum, 2.0);
/// # Ok::<(), sabiquant::math::SumError>(())
/// ```
pub fn compensated_sum(values: &[f64]) -> Result<f64, SumError> {
    for (index, &value) in values.iter().enumerate() {
        numeric::finite("values", value).map_err(|error| SumError::Input { index, error })?;
    }
    let mut sum = 0.0_f64;
    let mut correction = 0.0_f64;
    for (index, &value) in values.iter().enumerate() {
        let next = checked("partial sum", sum + value, Some(index))?;
        let roundoff = if sum.abs() >= value.abs() {
            (sum - next) + value
        } else {
            (value - next) + sum
        };
        correction = checked("summation correction", correction + roundoff, Some(index))?;
        sum = next;
    }
    checked("compensated sum", sum + correction, None).map(canonical_zero)
}

/// Returns the correctly rounded exact sum of finite binary64 inputs.
///
/// Every finite input is accumulated as an exact integer multiple of the
/// smallest subnormal, `2^-1074`. Positive and negative integers are retained
/// separately, subtracted exactly, then rounded once to nearest with ties to
/// even. No intermediate floating-point overflow or underflow occurs. The
/// finite result is invariant to input order and exact cancellation returns
/// positive zero. Subnormal sums are exact. The empty sum is positive zero.
///
/// This original fixed-point implementation follows the general
/// superaccumulator principle discussed by
/// [Neal (2015, pp. 3–4)](https://www.cs.toronto.edu/~radford/ftp/xsum.pdf).
/// It does not implement that paper's overlapping-chunk optimization.
///
/// Time is O(n) with a fixed binary64/`usize` dependent constant; memory is
/// two fixed integer arrays, independent of slice length. This sums supplied
/// values, not exact products or an exact mean: rounding products before
/// summation and dividing the rounded sum can introduce additional error.
///
/// # Errors
///
/// Returns [`SumError::Input`] for the first non-finite element. Returns
/// [`SumError::Arithmetic`] if rounding the final exact sum would overflow to
/// infinity, including the overflow midpoint tie. An exact sum slightly above
/// `f64::MAX` can still round to that finite value and is accepted.
///
/// # Examples
///
/// ```
/// use sabiquant::math::exact_sum;
/// let tiny = f64::from_bits(1);
/// assert_eq!(exact_sum(&[f64::MAX, f64::MAX, -f64::MAX, -f64::MAX, tiny])?, tiny);
/// # Ok::<(), sabiquant::math::SumError>(())
/// ```
pub fn exact_sum(values: &[f64]) -> Result<f64, SumError> {
    let mut positive = [0_u64; LIMBS];
    let mut negative = [0_u64; LIMBS];
    for (index, &value) in values.iter().enumerate() {
        numeric::finite("values", value).map_err(|error| SumError::Input { index, error })?;
        let magnitude = if value.is_sign_negative() {
            &mut negative
        } else {
            &mut positive
        };
        add_float(magnitude, value.to_bits());
    }
    let ordering = positive.iter().rev().cmp(negative.iter().rev());
    let (magnitude, subtrahend, negative_result) = match ordering {
        std::cmp::Ordering::Equal => return Ok(0.0),
        std::cmp::Ordering::Greater => (&mut positive, &negative, false),
        std::cmp::Ordering::Less => (&mut negative, &positive, true),
    };
    subtract(magnitude, subtrahend);
    round_binary64(magnitude, negative_result)
}

// Each input occupies at most 2098 bits when scaled by 2^1074. A slice
// contains fewer than 2^usize::BITS elements. Their separate unsigned sums
// therefore need at most 2098 + usize::BITS bits; round up to full limbs.
const LIMBS: usize = (2098 + usize::BITS as usize).div_ceil(64);
const FRACTION_MASK: u64 = (1_u64 << 52) - 1;

#[expect(
    clippy::cast_possible_truncation,
    reason = "exponent is at most 2046; low-word conversion intentionally extracts 64 bits of a u128"
)]
fn add_float(words: &mut [u64; LIMBS], bits: u64) {
    let exponent = ((bits >> 52) & 0x7ff) as usize;
    let significand = (bits & FRACTION_MASK) | if exponent == 0 { 0 } else { 1 << 52 };
    let shift = exponent.saturating_sub(1);
    let position = shift / 64;
    let shifted = u128::from(significand) << (shift % 64);
    add_word(words, position, shifted as u64);
    add_word(words, position + 1, (shifted >> 64) as u64);
}

fn add_word(words: &mut [u64; LIMBS], mut position: usize, mut addend: u64) {
    // Capacity proof above ensures a carry cannot run past the final limb.
    while addend != 0 {
        let (next, carry) = words[position].overflowing_add(addend);
        words[position] = next;
        addend = u64::from(carry);
        position += 1;
    }
}

fn subtract(words: &mut [u64; LIMBS], smaller: &[u64; LIMBS]) {
    let mut borrow = false;
    for (word, &other) in words.iter_mut().zip(smaller) {
        let (difference, first_borrow) = word.overflowing_sub(other);
        let (difference, second_borrow) = difference.overflowing_sub(u64::from(borrow));
        *word = difference;
        borrow = first_borrow || second_borrow;
    }
}

fn round_binary64(words: &[u64; LIMBS], negative: bool) -> Result<f64, SumError> {
    let Some(position) = words.iter().rposition(|&word| word != 0) else {
        return Ok(0.0);
    };
    let highest = position * 64 + (63 - words[position].leading_zeros() as usize);
    let sign = u64::from(negative) << 63;
    if highest < 52 {
        return Ok(f64::from_bits(sign | words[0]));
    }
    if highest > 2097 {
        return Err(final_overflow(negative));
    }
    let shift = highest - 52;
    let position = shift / 64;
    let offset = shift % 64;
    let mut significand = words[position] >> offset;
    if offset != 0 {
        significand |= words[position + 1] << (64 - offset);
    }
    if shift != 0 {
        let round_bit = bit(words, shift - 1);
        let sticky = any_below(words, shift - 1);
        if round_bit && (sticky || significand & 1 != 0) {
            significand += 1;
        }
    }
    let mut exponent = highest - 51;
    if significand == 1 << 53 {
        significand >>= 1;
        exponent += 1;
    }
    if exponent >= 2047 {
        return Err(final_overflow(negative));
    }
    Ok(f64::from_bits(
        sign | (exponent as u64) << 52 | (significand & FRACTION_MASK),
    ))
}

fn bit(words: &[u64; LIMBS], position: usize) -> bool {
    words[position / 64] & (1_u64 << (position % 64)) != 0
}

fn any_below(words: &[u64; LIMBS], exclusive: usize) -> bool {
    let limb = exclusive / 64;
    words[..limb].iter().any(|&word| word != 0)
        || words[limb] & ((1_u64 << (exclusive % 64)) - 1) != 0
}

fn final_overflow(negative: bool) -> SumError {
    SumError::Arithmetic {
        index: None,
        error: ValueError::ResultNotFinite {
            quantity: "exact sum",
            value: if negative {
                NonFinite::NegativeInfinity
            } else {
                NonFinite::PositiveInfinity
            },
        },
    }
}

fn checked(quantity: &'static str, value: f64, index: Option<usize>) -> Result<f64, SumError> {
    numeric::finite_result(quantity, value).map_err(|error| SumError::Arithmetic { index, error })
}

fn canonical_zero(value: f64) -> f64 {
    if value.to_bits() << 1 == 0 {
        0.0
    } else {
        value
    }
}
