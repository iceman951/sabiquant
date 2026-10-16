//! Exact decimal values and explicit quantum rounding (FR-AMT-01–14).
//!
//! A [`DecimalAmount`] is an exact signed i128 coefficient divided by a power
//! of ten, with at most 38 fractional places. It carries no currency, posting
//! rule or presentation scale. Every rounding operation requires both an
//! [`AmountQuantum`] and an [`AmountRounding`]. Existing binary64 analytics
//! are a separate domain; this module supplies no implicit conversion.
//!
//! Arithmetic and decimal parsing use bounded stack storage and no floating
//! point. Operations normalize the complete exact result before checking its
//! representation. Thus cancellation can rescue an overflowing intermediate.
//! Slice summation admits only the final total, independently of input order.
//!
//! ```
//! use sabiquant::amounts::{AmountQuantum, AmountRounding, DecimalAmount};
//! let value: DecimalAmount = "0.125".parse()?;
//! let nickel = AmountQuantum::new(5, 2)?;
//! let rounded = value.quantize(nickel, AmountRounding::NearestTiesEven)?;
//! assert_eq!(rounded.to_string(), "0.1");
//! # Ok::<(), sabiquant::amounts::AmountError>(())
//! ```

mod uint;

use crate::error::IdentifierError;
use crate::identifier::{Entry, Identifiers};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use uint::Uint;

/// An exact canonical decimal, without currency or an implicit rounding rule.
///
/// The value is `coefficient / 10^scale`. Trailing coefficient zeros are
/// removed while scale is positive; zero has coefficient and scale zero.
/// Numeric comparison and hashing therefore ignore original spelling.
/// Arithmetic uses checked methods rather than panicking operators:
///
/// ```compile_fail
/// use sabiquant::amounts::DecimalAmount;
/// let x = DecimalAmount::new(1, 0).unwrap();
/// let _ = x + x;
/// ```
///
/// Binary64 has no implicit conversion into this exact domain:
///
/// ```compile_fail
/// use sabiquant::amounts::DecimalAmount;
/// let _: DecimalAmount = 0.1_f64.into();
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DecimalAmount {
    /// Canonical signed coefficient.
    coefficient: i128,
    /// Canonical decimal scale, at most 38.
    scale: u8,
}

/// A strictly positive exact multiple used by an explicit rounding operation.
///
/// For example `(5, 2)` denotes multiples of .05, not merely two decimal
/// places. No currency denomination or default quantum is inferred.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AmountQuantum {
    /// Canonical positive amount.
    amount: DecimalAmount,
}

/// An explicit rule for rounding an exact value to a positive quantum.
///
/// The canonical case-sensitive text identifiers are the variant names.
/// Parsing accepts no aliases and does not trim whitespace. Unknown text
/// returns [`IdentifierError::Unknown`] and retains the full rejected text.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AmountRounding {
    /// Choose the multiple closest to zero.
    TowardZero,
    /// Choose the multiple furthest from zero.
    AwayFromZero,
    /// Choose the least multiple greater than or equal to the value.
    TowardPositiveInfinity,
    /// Choose the greatest multiple less than or equal to the value.
    TowardNegativeInfinity,
    /// Choose the nearest multiple, breaking ties toward an even quantum count.
    NearestTiesEven,
    /// Choose the nearest multiple, breaking ties away from zero.
    NearestTiesAwayFromZero,
    /// Require an exact multiple; report a nonzero remainder.
    Exact,
}

/// Exact identifiers, without aliases or ambiguous labels.
const ROUNDING_IDENTIFIERS: Identifiers<AmountRounding> = Identifiers::new(
    "amount rounding",
    &[
        Entry {
            value: AmountRounding::TowardZero,
            canonical: "TowardZero",
            aliases: &[],
        },
        Entry {
            value: AmountRounding::AwayFromZero,
            canonical: "AwayFromZero",
            aliases: &[],
        },
        Entry {
            value: AmountRounding::TowardPositiveInfinity,
            canonical: "TowardPositiveInfinity",
            aliases: &[],
        },
        Entry {
            value: AmountRounding::TowardNegativeInfinity,
            canonical: "TowardNegativeInfinity",
            aliases: &[],
        },
        Entry {
            value: AmountRounding::NearestTiesEven,
            canonical: "NearestTiesEven",
            aliases: &[],
        },
        Entry {
            value: AmountRounding::NearestTiesAwayFromZero,
            canonical: "NearestTiesAwayFromZero",
            aliases: &[],
        },
        Entry {
            value: AmountRounding::Exact,
            canonical: "Exact",
            aliases: &[],
        },
    ],
    &[],
);

impl AmountRounding {
    /// Every supported mode in canonical registry order.
    pub const ALL: [Self; 7] = [
        Self::TowardZero,
        Self::AwayFromZero,
        Self::TowardPositiveInfinity,
        Self::TowardNegativeInfinity,
        Self::NearestTiesEven,
        Self::NearestTiesAwayFromZero,
        Self::Exact,
    ];

    /// Returns this mode's exact case-sensitive identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::TowardZero => "TowardZero",
            Self::AwayFromZero => "AwayFromZero",
            Self::TowardPositiveInfinity => "TowardPositiveInfinity",
            Self::TowardNegativeInfinity => "TowardNegativeInfinity",
            Self::NearestTiesEven => "NearestTiesEven",
            Self::NearestTiesAwayFromZero => "NearestTiesAwayFromZero",
            Self::Exact => "Exact",
        }
    }
}

impl fmt::Display for AmountRounding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for AmountRounding {
    type Err = IdentifierError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        ROUNDING_IDENTIFIERS.parse(text)
    }
}

/// A failed exact decimal definition, computation or explicit rounding.
///
/// [`kind`](Self::kind) is stable; Display wording is not. The variants retain
/// structured input or final-result context and have no nested source.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AmountError {
    /// Decimal input exceeded the byte bound, before any lexical work.
    #[non_exhaustive]
    TextTooLong {
        /// Supplied UTF-8 byte length.
        length: usize,
        /// Maximum accepted byte length.
        maximum: usize,
    },
    /// Decimal text did not satisfy the exact ASCII grammar.
    #[non_exhaustive]
    InvalidSyntax {
        /// First offending byte, or length for missing digits at EOF.
        index: usize,
        /// Stable lexical reason.
        reason: &'static str,
    },
    /// An input scale or raw fractional length exceeded the bound.
    #[non_exhaustive]
    InvalidScale {
        /// Supplied input scale or fractional length.
        scale: usize,
        /// Maximum permitted scale.
        maximum: u8,
    },
    /// A quantum coefficient was not strictly positive.
    #[non_exhaustive]
    InvalidQuantum {
        /// Raw coefficient, before normalization.
        coefficient: i128,
        /// Raw scale, already validated.
        scale: u8,
    },
    /// A rational operation supplied a zero denominator or divisor.
    #[non_exhaustive]
    ZeroDenominator {
        /// Requested operation.
        operation: &'static str,
    },
    /// Exact rounding encountered a nonzero remainder.
    #[non_exhaustive]
    Inexact {
        /// Requested operation.
        operation: &'static str,
    },
    /// A normalized exact result requires too many fractional places.
    #[non_exhaustive]
    ResultScale {
        /// Requested operation.
        operation: &'static str,
        /// Normalized result scale.
        scale: u16,
        /// Maximum permitted scale.
        maximum: u8,
    },
    /// A normalized final coefficient is outside the signed i128 range.
    #[non_exhaustive]
    OutOfRange {
        /// Requested operation.
        operation: &'static str,
        /// Whether the unrepresentable result is negative.
        negative: bool,
    },
}

impl AmountError {
    /// Returns the stable error kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::TextTooLong { .. } => "amount.text_too_long",
            Self::InvalidSyntax { .. } => "amount.invalid_syntax",
            Self::InvalidScale { .. } => "amount.invalid_scale",
            Self::InvalidQuantum { .. } => "amount.invalid_quantum",
            Self::ZeroDenominator { .. } => "amount.zero_denominator",
            Self::Inexact { .. } => "amount.inexact",
            Self::ResultScale { .. } => "amount.result_scale",
            Self::OutOfRange { .. } => "amount.out_of_range",
        }
    }
}

impl fmt::Display for AmountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TextTooLong { length, maximum } => {
                write!(f, "decimal text has {length} bytes; maximum is {maximum}")
            }
            Self::InvalidSyntax { index, reason } => {
                write!(f, "invalid decimal syntax at byte {index}: {reason}")
            }
            Self::InvalidScale { scale, maximum } => {
                write!(f, "decimal scale {scale} exceeds {maximum}")
            }
            Self::InvalidQuantum { coefficient, scale } => {
                write!(
                    f,
                    "quantum coefficient {coefficient} at scale {scale} must be positive"
                )
            }
            Self::ZeroDenominator { operation } => {
                write!(f, "{operation} requires a nonzero denominator")
            }
            Self::Inexact { operation } => {
                write!(f, "{operation} is not an exact quantum multiple")
            }
            Self::ResultScale {
                operation,
                scale,
                maximum,
            } => {
                write!(f, "{operation} result scale {scale} exceeds {maximum}")
            }
            Self::OutOfRange {
                operation,
                negative,
            } => {
                write!(
                    f,
                    "{operation} result coefficient is out of range (negative={negative})"
                )
            }
        }
    }
}

impl std::error::Error for AmountError {}

impl DecimalAmount {
    /// Maximum input and normalized result scale.
    pub const MAX_SCALE: u8 = 38;
    /// Maximum decimal parser UTF-8 byte length.
    pub const MAX_TEXT_BYTES: usize = 128;

    /// Creates a canonical exact value from an explicit coefficient and scale.
    ///
    /// # Errors
    /// Returns [`AmountError::InvalidScale`] before normalization if scale>38,
    /// including when the coefficient is zero.
    pub fn new(mut coefficient: i128, mut scale: u8) -> Result<Self, AmountError> {
        validate_scale(usize::from(scale))?;
        if coefficient == 0 {
            scale = 0;
        } else {
            while scale > 0 && coefficient % 10 == 0 {
                coefficient /= 10;
                scale -= 1;
            }
        }
        Ok(Self { coefficient, scale })
    }

    /// Returns the canonical signed coefficient.
    #[must_use]
    pub const fn coefficient(self) -> i128 {
        self.coefficient
    }

    /// Returns the canonical decimal scale.
    #[must_use]
    pub const fn scale(self) -> u8 {
        self.scale
    }

    /// Adds two exact decimals, admitting only their normalized final result.
    ///
    /// # Errors
    /// Returns [`AmountError::OutOfRange`] if the final coefficient cannot fit.
    pub fn checked_add(self, other: Self) -> Result<Self, AmountError> {
        self.combine(other, false, "add")
    }

    /// Subtracts two exact decimals without first negating the subtrahend.
    ///
    /// # Errors
    /// Returns [`AmountError::OutOfRange`] if the final coefficient cannot fit.
    pub fn checked_sub(self, other: Self) -> Result<Self, AmountError> {
        self.combine(other, true, "subtract")
    }

    /// Negates this exact value.
    ///
    /// # Errors
    /// Returns [`AmountError::OutOfRange`] for canonical i128::MIN.
    pub fn checked_neg(self) -> Result<Self, AmountError> {
        self.coefficient
            .checked_neg()
            .map(|coefficient| Self {
                coefficient,
                scale: self.scale,
            })
            .ok_or(AmountError::OutOfRange {
                operation: "negate",
                negative: false,
            })
    }

    /// Multiplies exactly, normalizing all trailing zeros before admission.
    ///
    /// # Errors
    /// Returns [`AmountError::ResultScale`] for normalized scale>38, before
    /// [`AmountError::OutOfRange`] for an excessive coefficient. Never rounds.
    pub fn checked_mul(self, other: Self) -> Result<Self, AmountError> {
        let negative = (self.coefficient < 0) != (other.coefficient < 0);
        let magnitude = wide(
            magnitude(self).checked_mul(magnitude(other)),
            "multiply",
            negative,
        )?;
        finish(
            magnitude,
            negative,
            u16::from(self.scale) + u16::from(other.scale),
            "multiply",
        )
    }

    /// Rounds this exact value once to an explicitly supplied quantum.
    ///
    /// # Errors
    /// Returns [`AmountError::Inexact`] for Exact with a nonzero remainder,
    /// before [`AmountError::OutOfRange`] for an unrepresentable rounded value.
    pub fn quantize(
        self,
        quantum: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<Self, AmountError> {
        let negative = self.coefficient < 0;
        let numerator = scale_up(
            magnitude(self),
            u16::from(quantum.amount.scale),
            "quantize",
            negative,
        )?;
        let denominator = scale_up(
            magnitude(quantum.amount),
            u16::from(self.scale),
            "quantize",
            negative,
        )?;
        round_ratio(numerator, denominator, negative, quantum, mode, "quantize")
    }

    /// Tests exact quantum divisibility, with no rounding or range conversion.
    #[must_use]
    pub fn is_multiple_of(self, quantum: AmountQuantum) -> bool {
        // An exact quantum multiple reproduces this representable input, so
        // final range admission cannot reject an otherwise exact result.
        self.quantize(quantum, AmountRounding::Exact).is_ok()
    }

    /// Rounds the exact signed ratio once to the explicit quantum.
    ///
    /// # Errors
    /// Rejects denominator zero first, then reports Inexact (Exact mode) or
    /// OutOfRange for an unrepresentable rounded final coefficient.
    pub fn from_ratio(
        numerator: i128,
        denominator: u128,
        quantum: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<Self, AmountError> {
        nonzero_denominator(denominator, "ratio")?;
        let negative = numerator < 0;
        let numerator = scale_up(
            Uint::from_u128(numerator.unsigned_abs()),
            u16::from(quantum.amount.scale),
            "ratio",
            negative,
        )?;
        let denominator = wide(
            Uint::from_u128(denominator).checked_mul(magnitude(quantum.amount)),
            "ratio",
            negative,
        )?;
        round_ratio(numerator, denominator, negative, quantum, mode, "ratio")
    }

    /// Rounds exact `self * numerator / denominator` once, without admitting
    /// the intermediate product as a DecimalAmount.
    ///
    /// # Errors
    /// Rejects denominator zero even for zero operands, then reports Inexact
    /// (Exact mode) or OutOfRange for the rounded final coefficient.
    pub fn multiply_ratio(
        self,
        numerator: i128,
        denominator: u128,
        quantum: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<Self, AmountError> {
        nonzero_denominator(denominator, "multiply_ratio")?;
        let negative = (self.coefficient < 0) != (numerator < 0);
        let product = wide(
            magnitude(self).checked_mul(Uint::from_u128(numerator.unsigned_abs())),
            "multiply_ratio",
            negative,
        )?;
        let numerator = scale_up(
            product,
            u16::from(quantum.amount.scale),
            "multiply_ratio",
            negative,
        )?;
        let denominator = wide(
            Uint::from_u128(denominator).checked_mul(magnitude(quantum.amount)),
            "multiply_ratio",
            negative,
        )?;
        let denominator = scale_up(
            denominator,
            u16::from(self.scale),
            "multiply_ratio",
            negative,
        )?;
        round_ratio(
            numerator,
            denominator,
            negative,
            quantum,
            mode,
            "multiply_ratio",
        )
    }

    /// Rounds exact `self * other / divisor` once, preserving both decimal
    /// scales and avoiding any intermediate decimal representation limit.
    ///
    /// # Errors
    /// Rejects divisor zero even for zero operands, then reports Inexact
    /// (Exact mode) or OutOfRange for the rounded final coefficient.
    pub fn multiply_divide(
        self,
        other: Self,
        divisor: u128,
        quantum: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<Self, AmountError> {
        nonzero_denominator(divisor, "multiply_divide")?;
        let negative = (self.coefficient < 0) != (other.coefficient < 0);
        let product = wide(
            magnitude(self).checked_mul(magnitude(other)),
            "multiply_divide",
            negative,
        )?;
        let numerator = scale_up(
            product,
            u16::from(quantum.amount.scale),
            "multiply_divide",
            negative,
        )?;
        let denominator = wide(
            Uint::from_u128(divisor).checked_mul(magnitude(quantum.amount)),
            "multiply_divide",
            negative,
        )?;
        let denominator = scale_up(
            denominator,
            u16::from(self.scale) + u16::from(other.scale),
            "multiply_divide",
            negative,
        )?;
        round_ratio(
            numerator,
            denominator,
            negative,
            quantum,
            mode,
            "multiply_divide",
        )
    }

    /// Combines aligned unsigned magnitudes with separate signs.
    fn combine(
        self,
        other: Self,
        subtract: bool,
        operation: &'static str,
    ) -> Result<Self, AmountError> {
        let scale = self.scale.max(other.scale);
        let left_negative = self.coefficient < 0;
        let right_negative = (other.coefficient < 0) != subtract;
        let left = scale_up(
            magnitude(self),
            u16::from(scale - self.scale),
            operation,
            left_negative,
        )?;
        let right = scale_up(
            magnitude(other),
            u16::from(scale - other.scale),
            operation,
            right_negative,
        )?;
        let (magnitude, negative) =
            combine_signed(left, left_negative, right, right_negative, operation)?;
        finish(magnitude, negative, u16::from(scale), operation)
    }
}

impl AmountQuantum {
    /// Creates a strictly positive exact quantum, with no inferred currency.
    ///
    /// # Errors
    /// Checks InvalidScale first (including zero/negative coefficients), then
    /// InvalidQuantum for coefficient<=0, retaining the raw input context.
    pub fn new(coefficient: i128, scale: u8) -> Result<Self, AmountError> {
        validate_scale(usize::from(scale))?;
        if coefficient <= 0 {
            return Err(AmountError::InvalidQuantum { coefficient, scale });
        }
        Ok(Self {
            amount: DecimalAmount::new(coefficient, scale)?,
        })
    }

    /// Returns the canonical positive amount represented by this quantum.
    #[must_use]
    pub const fn amount(self) -> DecimalAmount {
        self.amount
    }
}

impl Ord for DecimalAmount {
    fn cmp(&self, other: &Self) -> Ordering {
        let sign_order = self.coefficient.signum().cmp(&other.coefficient.signum());
        if sign_order != Ordering::Equal {
            return sign_order;
        }
        let order = compare_magnitudes(*self, *other);
        if self.coefficient < 0 {
            order.reverse()
        } else {
            order
        }
    }
}

impl PartialOrd for DecimalAmount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for DecimalAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut digits = [b'0'; 39];
        let mut position = digits.len();
        let mut value = self.coefficient.unsigned_abs();
        loop {
            position -= 1;
            // Remainder is a decimal digit, hence fits u8.
            {
                digits[position] = b'0' + (value % 10) as u8;
            }
            value /= 10;
            if value == 0 {
                break;
            }
        }
        let count = digits.len() - position;
        let scale = usize::from(self.scale);
        let mut text = [b'0'; 80];
        let mut end = 0;
        if self.coefficient < 0 {
            text[end] = b'-';
            end += 1;
        }
        if scale == 0 {
            text[end..end + count].copy_from_slice(&digits[position..]);
            end += count;
        } else if count > scale {
            let integer_count = count - scale;
            text[end..end + integer_count]
                .copy_from_slice(&digits[position..position + integer_count]);
            end += integer_count;
            text[end] = b'.';
            end += 1;
            text[end..end + scale].copy_from_slice(&digits[position + integer_count..]);
            end += scale;
        } else {
            text[end] = b'0';
            text[end + 1] = b'.';
            end += 2;
            end += scale - count;
            text[end..end + count].copy_from_slice(&digits[position..]);
            end += count;
        }
        // All bytes were initialized as or replaced by ASCII punctuation/digits.
        // write_str deliberately ignores precision: this is an exact value.
        f.write_str(std::str::from_utf8(&text[..end]).map_err(|_| fmt::Error)?)
    }
}

impl FromStr for DecimalAmount {
    type Err = AmountError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let bytes = text.as_bytes();
        if bytes.len() > Self::MAX_TEXT_BYTES {
            return Err(AmountError::TextTooLong {
                length: bytes.len(),
                maximum: Self::MAX_TEXT_BYTES,
            });
        }
        let negative = bytes.first() == Some(&b'-');
        let start = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
        let mut index = start;
        require_digit(bytes, index, "missing integer digits")?;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        let mut fraction_start = bytes.len();
        if bytes.get(index) == Some(&b'.') {
            index += 1;
            fraction_start = index;
            require_digit(bytes, index, "missing fractional digits")?;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
        }
        if index != bytes.len() {
            return Err(AmountError::InvalidSyntax {
                index,
                reason: "unexpected byte",
            });
        }
        let scale = bytes.len() - fraction_start;
        validate_scale(scale)?;
        let mut end = bytes.len();
        while end > fraction_start && bytes[end - 1] == b'0' {
            end -= 1;
        }
        let normalized_scale = end.saturating_sub(fraction_start);
        let mut magnitude = 0_u128;
        for &byte in &bytes[start..end] {
            if byte != b'.' {
                magnitude = magnitude
                    .checked_mul(10)
                    .and_then(|value| value.checked_add(u128::from(byte - b'0')))
                    .ok_or(AmountError::OutOfRange {
                        operation: "parse",
                        negative,
                    })?;
            }
        }
        // Scale is at most 38 after the preceding validation.
        #[expect(
            clippy::cast_possible_truncation,
            reason = "fractional length was validated <=38"
        )]
        finish(
            Uint::from_u128(magnitude),
            negative,
            normalized_scale as u16,
            "parse",
        )
    }
}

/// Sums exact supplied decimals, checking only the normalized final result.
///
/// Positive and negative magnitudes are accumulated separately at scale38;
/// overflowing prefixes may cancel. Empty input returns canonical zero.
/// Storage is constant and work is O(n), independently of input order.
///
/// # Errors
/// Returns [`AmountError::OutOfRange`] if the final coefficient cannot fit.
pub fn exact_amount_sum(values: &[DecimalAmount]) -> Result<DecimalAmount, AmountError> {
    // Each term <2^254; a slice has fewer than 2^usize::BITS terms.
    // This also checks capacity for every target on which this crate compiles.
    const {
        assert!(254 + usize::BITS <= 512);
    }
    let mut positive = Uint::ZERO;
    let mut negative = Uint::ZERO;
    for &value in values {
        let is_negative = value.coefficient < 0;
        let term = scale_up(
            magnitude(value),
            u16::from(DecimalAmount::MAX_SCALE - value.scale),
            "sum",
            is_negative,
        )?;
        let accumulator = if is_negative {
            &mut negative
        } else {
            &mut positive
        };
        *accumulator = wide(accumulator.checked_add(term), "sum", is_negative)?;
    }
    let (total, negative) = combine_signed(positive, false, negative, true, "sum")?;
    finish(total, negative, u16::from(DecimalAmount::MAX_SCALE), "sum")
}

/// Validates a raw scale before any normalization.
fn validate_scale(scale: usize) -> Result<(), AmountError> {
    if scale > usize::from(DecimalAmount::MAX_SCALE) {
        Err(AmountError::InvalidScale {
            scale,
            maximum: DecimalAmount::MAX_SCALE,
        })
    } else {
        Ok(())
    }
}

/// Validates the first mandatory digit of a lexical state.
fn require_digit(bytes: &[u8], index: usize, reason: &'static str) -> Result<(), AmountError> {
    if bytes.get(index).is_some_and(u8::is_ascii_digit) {
        Ok(())
    } else {
        Err(AmountError::InvalidSyntax { index, reason })
    }
}

/// Converts a validated amount to an unsigned magnitude, including MIN.
fn magnitude(value: DecimalAmount) -> Uint {
    Uint::from_u128(value.coefficient.unsigned_abs())
}

/// Maps an impossible-under-width-proof internal capacity failure to a typed
/// error rather than a panic. Public final admission is performed separately.
fn wide(value: Option<Uint>, operation: &'static str, negative: bool) -> Result<Uint, AmountError> {
    value.ok_or(AmountError::OutOfRange {
        operation,
        negative,
    })
}

/// Multiplies a bounded magnitude by an exact decimal power.
fn scale_up(
    value: Uint,
    scale: u16,
    operation: &'static str,
    negative: bool,
) -> Result<Uint, AmountError> {
    let power = wide(Uint::pow10(scale), operation, negative)?;
    wide(value.checked_mul(power), operation, negative)
}

/// Adds/subtracts nonnegative magnitudes without signed minimum traps.
fn combine_signed(
    left: Uint,
    left_negative: bool,
    right: Uint,
    right_negative: bool,
    operation: &'static str,
) -> Result<(Uint, bool), AmountError> {
    if left_negative == right_negative {
        Ok((
            wide(left.checked_add(right), operation, left_negative)?,
            left_negative,
        ))
    } else if left >= right {
        Ok((
            wide(left.checked_sub(right), operation, left_negative)?,
            left_negative,
        ))
    } else {
        Ok((
            wide(right.checked_sub(left), operation, right_negative)?,
            right_negative,
        ))
    }
}

/// Normalizes the complete result, then checks scale before signed range.
fn finish(
    mut value: Uint,
    negative: bool,
    mut scale: u16,
    operation: &'static str,
) -> Result<DecimalAmount, AmountError> {
    if value.is_zero() {
        return Ok(DecimalAmount {
            coefficient: 0,
            scale: 0,
        });
    }
    while scale > 0 {
        match value.div_rem_small(10) {
            Some((quotient, 0)) => {
                value = quotient;
                scale -= 1;
            }
            _ => break,
        }
    }
    if scale > u16::from(DecimalAmount::MAX_SCALE) {
        return Err(AmountError::ResultScale {
            operation,
            scale,
            maximum: DecimalAmount::MAX_SCALE,
        });
    }
    let value = value.to_u128().ok_or(AmountError::OutOfRange {
        operation,
        negative,
    })?;
    let limit = if negative {
        1_u128 << 127
    } else {
        (1_u128 << 127) - 1
    };
    if value > limit {
        return Err(AmountError::OutOfRange {
            operation,
            negative,
        });
    }
    let coefficient = if negative && value == (1_u128 << 127) {
        i128::MIN
    } else {
        // This branch excludes 2^127 and the range test excludes larger values.
        #[expect(
            clippy::cast_possible_wrap,
            reason = "magnitude is at most i128::MAX in this branch"
        )]
        let signed = value as i128;
        if negative { -signed } else { signed }
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "normalized scale validated <=38"
    )]
    Ok(DecimalAmount {
        coefficient,
        scale: scale as u8,
    })
}

/// Compares exact unsigned decimals using quotient/remainder, avoiding scaled
/// cross-products. A scale difference at most38 has a u128 power of ten.
fn compare_magnitudes(left: DecimalAmount, right: DecimalAmount) -> Ordering {
    if left.scale < right.scale {
        return compare_magnitudes(right, left).reverse();
    }
    let divisor = 10_u128.pow(u32::from(left.scale - right.scale));
    let left = left.coefficient.unsigned_abs();
    match (left / divisor).cmp(&right.coefficient.unsigned_abs()) {
        Ordering::Equal => (left % divisor).cmp(&0),
        other => other,
    }
}

/// Rejects denominator zero before any zero numerator shortcut.
fn nonzero_denominator(denominator: u128, operation: &'static str) -> Result<(), AmountError> {
    if denominator == 0 {
        Err(AmountError::ZeroDenominator { operation })
    } else {
        Ok(())
    }
}

/// Divides an exact rational measured in quantum counts and rounds once.
fn round_ratio(
    numerator: Uint,
    denominator: Uint,
    negative: bool,
    quantum: AmountQuantum,
    mode: AmountRounding,
    operation: &'static str,
) -> Result<DecimalAmount, AmountError> {
    let (mut count, remainder) = numerator
        .div_rem(denominator)
        .ok_or(AmountError::OutOfRange {
            operation,
            negative,
        })?;
    if !remainder.is_zero() {
        if mode == AmountRounding::Exact {
            return Err(AmountError::Inexact { operation });
        }
        // Compare r with d-r rather than doubling r and losing a high carry.
        let complement = wide(denominator.checked_sub(remainder), operation, negative)?;
        let increment = match mode {
            AmountRounding::TowardZero | AmountRounding::Exact => false,
            AmountRounding::AwayFromZero => true,
            AmountRounding::TowardPositiveInfinity => !negative,
            AmountRounding::TowardNegativeInfinity => negative,
            AmountRounding::NearestTiesEven => {
                remainder > complement || (remainder == complement && count.is_odd())
            }
            AmountRounding::NearestTiesAwayFromZero => remainder >= complement,
        };
        if increment {
            count = wide(count.checked_add(Uint::from_u128(1)), operation, negative)?;
        }
    }
    let result = wide(
        count.checked_mul(magnitude(quantum.amount)),
        operation,
        negative,
    )?;
    finish(result, negative, u16::from(quantum.amount.scale), operation)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-AMT-10: the shared registry has one exact canonical spelling per mode.
    #[test]
    fn rounding_identifier_table_is_complete() {
        ROUNDING_IDENTIFIERS.check().unwrap();
        ROUNDING_IDENTIFIERS
            .check_round_trip(&AmountRounding::ALL, |mode| mode.to_string())
            .unwrap();
    }
}
