//! Explicitly interpreted conversions between exact amounts and binary64.
//!
//! A finite binary64 is read as its exact bit-field rational, never as the
//! decimal its formatting prints. Outgoing conversion rounds the exact decimal
//! rational once, nearest with ties to even, using integer comparisons only.

use super::uint::{Uint, Wide};
use super::{AmountError, AmountQuantum, AmountRounding, DecimalAmount, finish, round_count};
use crate::error::NonFinite;

const FROM_EXACT: &str = "from_binary64_exact";
const FROM_ROUNDED: &str = "from_binary64_rounded";
const TO_EXACT: &str = "to_binary64_exact";

/// Nineteen limbs (1216 bits) hold the widest rounded-conversion ratio.
type Conversion = Wide<19>;

/// The exact value `(-1)^negative * significand * 2^exponent` of a finite input.
struct Parts {
    negative: bool,
    /// Below 2^53; zero for both floating zeros.
    significand: u64,
    /// In `-1074..=971`.
    exponent: i32,
}

/// Reads the IEEE 754 fields after rejecting NaN and infinities first.
fn decompose(value: f64, operation: &'static str) -> Result<Parts, AmountError> {
    if let Some(class) = NonFinite::classify(value) {
        return Err(AmountError::NonFinite {
            operation,
            value: class,
        });
    }
    let bits = value.to_bits();
    let field = (bits >> 52) & 0x7ff;
    let fraction = bits & ((1 << 52) - 1);
    // The masked biased exponent has 11 bits, so it fits i32 exactly.
    let field = field as i32;
    let (significand, exponent) = if field == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1 << 52), field - 1075)
    };
    Ok(Parts {
        negative: bits >> 63 != 0,
        significand,
        exponent,
    })
}

const ZERO: DecimalAmount = DecimalAmount {
    coefficient: 0,
    scale: 0,
};

impl DecimalAmount {
    /// Converts the exact rational value of a finite binary64, or fails.
    ///
    /// Both floating zeros give canonical zero. The value `M·2^e` with odd
    /// `M` needs scale `−e` when `e < 0`, so binary64 `0.1` (scale 55) fails
    /// [`AmountError::ResultScale`] instead of becoming decimal `0.1`.
    ///
    /// ```
    /// use sabiquant::amounts::DecimalAmount;
    /// assert_eq!(DecimalAmount::from_binary64_exact(0.5)?.to_string(), "0.5");
    /// assert_eq!(DecimalAmount::from_binary64_exact(0.1).unwrap_err().kind(),
    ///     "amount.result_scale");
    /// # Ok::<(), sabiquant::amounts::AmountError>(())
    /// ```
    ///
    /// # Errors
    /// [`AmountError::NonFinite`] for NaN or an infinity, first. Then
    /// [`AmountError::ResultScale`] when more than 38 places are needed, before
    /// [`AmountError::OutOfRange`] for a coefficient outside i128. Never rounds.
    pub fn from_binary64_exact(value: f64) -> Result<Self, AmountError> {
        let Parts {
            negative,
            significand,
            exponent,
        } = decompose(value, FROM_EXACT)?;
        if significand == 0 {
            return Ok(ZERO);
        }
        let zeros = significand.trailing_zeros();
        let odd = significand >> zeros;
        let exponent = exponent.saturating_add_unsigned(zeros);
        let range = || AmountError::OutOfRange {
            operation: FROM_EXACT,
            negative,
        };
        if exponent >= 0 {
            // An odd significand times a power of two: an integer at scale 0.
            let magnitude = Uint::from_u128(u128::from(odd))
                .checked_shl(exponent.unsigned_abs())
                .ok_or_else(range)?;
            return finish(magnitude, negative, 0, FROM_EXACT);
        }
        let places = exponent.unsigned_abs();
        if places > u32::from(Self::MAX_SCALE) {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a binary64 needs at most 1074 fractional places"
            )]
            let scale = places as u16;
            return Err(AmountError::ResultScale {
                operation: FROM_EXACT,
                scale,
                maximum: Self::MAX_SCALE,
            });
        }
        // odd·5^t is odd, so scale t is already canonical; 5^38 < 2^89.
        let power = 5_u128.checked_pow(places).ok_or_else(range)?;
        let magnitude = Uint::from_u128(u128::from(odd))
            .checked_mul(Uint::from_u128(power))
            .ok_or_else(range)?;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "places is at most 38 in this branch"
        )]
        let scale = places as u16;
        finish(magnitude, negative, scale, FROM_EXACT)
    }

    /// Rounds the exact rational value of a finite binary64 once to a quantum.
    ///
    /// With `x = ±M·2^e` and `q = Q/10^s`, the ratio `|x|/q` is reduced
    /// exactly in a 1216-bit integer and rounded by the same table as
    /// [`quantize`](Self::quantize). A nonzero tiny input may round to zero
    /// or to one quantum; huge inputs are classified only after rounding.
    ///
    /// ```
    /// use sabiquant::amounts::{AmountQuantum, AmountRounding, DecimalAmount};
    /// let cent = AmountQuantum::new(1, 2)?;
    /// // Binary64 2.675 is slightly below decimal 2.675, so it rounds down.
    /// let value = DecimalAmount::from_binary64_rounded(2.675, cent,
    ///     AmountRounding::NearestTiesAwayFromZero)?;
    /// assert_eq!(value.to_string(), "2.67");
    /// # Ok::<(), sabiquant::amounts::AmountError>(())
    /// ```
    ///
    /// # Errors
    /// [`AmountError::NonFinite`] for NaN or an infinity, before any other
    /// check. [`AmountError::Inexact`] for `Exact` with a nonzero remainder,
    /// before [`AmountError::OutOfRange`] for an unrepresentable result.
    pub fn from_binary64_rounded(
        value: f64,
        quantum: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<Self, AmountError> {
        let Parts {
            negative,
            significand,
            exponent,
        } = decompose(value, FROM_ROUNDED)?;
        if significand == 0 {
            return Ok(ZERO);
        }
        // The width proof excludes each capacity failure; keep a typed error.
        let capacity = |result: Option<Conversion>| {
            result.ok_or(AmountError::OutOfRange {
                operation: FROM_ROUNDED,
                negative,
            })
        };
        let scale = quantum.amount.scale;
        let decimal = capacity(Conversion::pow10(u16::from(scale)))?;
        let units = Conversion::from_u128(quantum.amount.coefficient.unsigned_abs());
        let significand = Conversion::from_u128(u128::from(significand));
        let shift = exponent.unsigned_abs();
        let (numerator, denominator) = if exponent >= 0 {
            let shifted = capacity(significand.checked_shl(shift))?;
            (capacity(shifted.checked_mul(decimal))?, units)
        } else {
            (
                capacity(significand.checked_mul(decimal))?,
                capacity(units.checked_shl(shift))?,
            )
        };
        let count = round_count(numerator, denominator, negative, mode, FROM_ROUNDED)?;
        let product = capacity(count.checked_mul(units))?;
        // Above 512 bits, removing at most 38 factors of ten still leaves a
        // magnitude beyond 2^127, so the final range check would also fail.
        let product = product.resize::<8>().ok_or(AmountError::OutOfRange {
            operation: FROM_ROUNDED,
            negative,
        })?;
        finish(product, negative, u16::from(scale), FROM_ROUNDED)
    }

    /// Returns the exact binary64 value of this amount, or fails.
    ///
    /// Decimal `0.5` converts; decimal `0.1` has no binary64 representation.
    /// A successful result has the same bits as
    /// [`to_binary64_nearest`](Self::to_binary64_nearest).
    ///
    /// # Errors
    /// [`AmountError::Inexact`] when the exact rational is not a binary64.
    pub fn to_binary64_exact(self) -> Result<f64, AmountError> {
        match self.binary64_parts() {
            Some((bits, true)) => Ok(f64::from_bits(bits)),
            _ => Err(AmountError::Inexact {
                operation: TO_EXACT,
            }),
        }
    }

    /// Returns the binary64 nearest to this exact amount, ties to even.
    ///
    /// Canonical zero gives `+0.0`. Amount magnitudes lie between `10^-38`
    /// and `2^127`, so the result is always a finite normal number with the
    /// amount's sign. The rounding is decided by integer comparisons only.
    ///
    /// ```
    /// use sabiquant::amounts::DecimalAmount;
    /// let tenth: DecimalAmount = "0.1".parse()?;
    /// assert_eq!(tenth.to_binary64_nearest(), 0.1);
    /// assert!(tenth.to_binary64_exact().is_err());
    /// # Ok::<(), sabiquant::amounts::AmountError>(())
    /// ```
    #[must_use]
    pub fn to_binary64_nearest(self) -> f64 {
        // The range proof excludes a missing result; NaN is never returned.
        self.binary64_parts()
            .map_or(f64::NAN, |(bits, _)| f64::from_bits(bits))
    }

    /// Returns the nearest binary64 bits and whether they are exact.
    fn binary64_parts(self) -> Option<(u64, bool)> {
        if self.coefficient == 0 {
            return Some((0, true));
        }
        let sign = if self.coefficient < 0 { 1_u64 << 63 } else { 0 };
        let magnitude = Uint::from_u128(self.coefficient.unsigned_abs());
        // Canonical construction bounds the scale by 38, so 10^scale < 2^127.
        let base = Uint::from_u128(10_u128.pow(u32::from(self.scale)));
        // The value lies in [2^(width-1), 2^(width+1)) for this bit difference.
        #[expect(clippy::cast_possible_wrap, reason = "both bit counts are at most 128")]
        let width = magnitude.bits() as i32 - base.bits() as i32;
        let mut exponent = width - 53;
        let mut quotient = Self::scaled(magnitude, base, exponent)?;
        if quotient.0.bits() > 53 {
            exponent += 1;
            quotient = Self::scaled(magnitude, base, exponent)?;
        }
        let (mut significand, remainder, denominator) = quotient;
        let exact = remainder.is_zero();
        if !exact {
            // Compare r with d-r rather than doubling r; ties go to even.
            let complement = denominator.checked_sub(remainder)?;
            if remainder > complement || (remainder == complement && significand.is_odd()) {
                significand = significand.checked_add(Uint::from_u128(1))?;
                if significand.bits() > 53 {
                    significand = Uint::from_u128(1 << 52);
                    exponent += 1;
                }
            }
        }
        let significand = u64::try_from(significand.to_u128()?).ok()?;
        // significand is in [2^52, 2^53) and value = significand·2^exponent.
        let biased = u64::try_from(exponent + 1075).ok()?;
        Some((sign | (biased << 52) | (significand - (1 << 52)), exact))
    }

    /// Returns `floor(magnitude/(base·2^exponent))`, its remainder and divisor.
    fn scaled(magnitude: Uint, base: Uint, exponent: i32) -> Option<(Uint, Uint, Uint)> {
        let shift = exponent.unsigned_abs();
        let (numerator, denominator) = if exponent <= 0 {
            (magnitude.checked_shl(shift)?, base)
        } else {
            (magnitude, base.checked_shl(shift)?)
        };
        let (quotient, remainder) = numerator.div_rem(denominator)?;
        Some((quotient, remainder, denominator))
    }
}
