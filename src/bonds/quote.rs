//! Exact binary64 rational rounding for bond per-100 quote conversions.
//!
//! Each finite nonzero binary64 operand is an integer significand of at most 53
//! bits times a power of two. Converting an amount `X` with positive face `F`
//! to a price per 100 evaluates `(25 * M_X / M_F) * 2^(e_X - e_F + 2)`, while
//! converting a price per 100 `P` back to an amount evaluates
//! `(M_F * M_P / 25) * 2^(e_F + e_P - 2)`. Both numerators have at most 106
//! bits and both denominators have at most 53 bits, so after aligning to the
//! target binary64 unit exponent, the exact integer division and tie comparison
//! fit in `u128` without intermediate overflow or underflow.

use crate::error::{Interval, ValueError};
use crate::numeric;

const FRACTION_MASK: u64 = (1_u64 << 52) - 1;
const SIGN_BIT: u64 = 1_u64 << 63;
const INFINITY_BITS: u64 = 0x7ff0_0000_0000_0000;

/// Normalizes a finite signed amount to a price per 100 of positive finite face.
pub(super) fn per_100_from_amount(amount: f64, face: f64) -> Result<f64, ValueError> {
    if amount.to_bits() & !SIGN_BIT == 0 {
        return Ok(0.0);
    }
    let negative = amount.is_sign_negative();
    let (sig_amount, exp_amount) = decompose(amount.to_bits());
    let (sig_face, exp_face) = decompose(face.to_bits());
    let numerator = 25_u128 * u128::from(sig_amount);
    let denominator = u128::from(sig_face);
    let exponent = exp_amount - exp_face + 2;
    finish_rounded(
        "price per 100",
        negative,
        round_scaled_ratio(numerator, denominator, exponent),
    )
}

/// Converts a finite signed price per 100 into amount units for positive finite face.
pub(super) fn amount_from_per_100(price_per_100: f64, face: f64) -> Result<f64, ValueError> {
    if price_per_100.to_bits() & !SIGN_BIT == 0 {
        return Ok(0.0);
    }
    let negative = price_per_100.is_sign_negative();
    let (sig_price, exp_price) = decompose(price_per_100.to_bits());
    let (sig_face, exp_face) = decompose(face.to_bits());
    let numerator = u128::from(sig_face) * u128::from(sig_price);
    let exponent = exp_face + exp_price - 2;
    finish_rounded(
        "bond amount",
        negative,
        round_scaled_ratio(numerator, 25_u128, exponent),
    )
}

fn finish_rounded(
    quantity: &'static str,
    negative: bool,
    magnitude: u64,
) -> Result<f64, ValueError> {
    let rounded = f64::from_bits(magnitude | if negative { SIGN_BIT } else { 0 });
    let domain = if negative {
        Interval::less_than(0.0)
    } else {
        Interval::greater_than(0.0)
    };
    numeric::result_within(quantity, rounded, domain)
}

/// Extracts the nonzero integer significand (`1..2^53`) and binary exponent (`-1074..=971`).
fn decompose(bits: u64) -> (u64, i32) {
    let field = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & FRACTION_MASK;
    if field == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), field - 1075)
    }
}

/// Rounds `(numerator / denominator) * 2^exponent` once to nearest binary64, ties to even.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "bit counts are in 1..=106, quotient fits in 54 bits, and biased exponents are nonnegative"
)]
fn round_scaled_ratio(numerator: u128, denominator: u128, exponent: i32) -> u64 {
    let num_bits = (128 - numerator.leading_zeros()) as i32;
    let den_bits = (128 - denominator.leading_zeros()) as i32;
    let k0 = num_bits - den_bits;
    let at_least_k0 = if k0 >= 0 {
        numerator >= (denominator << (k0 as u32))
    } else {
        (numerator << ((-k0) as u32)) >= denominator
    };
    let k = if at_least_k0 { k0 } else { k0 - 1 };
    let top = k + exponent;
    if top > 1023 {
        return INFINITY_BITS;
    }
    if top < -1075 {
        return 0;
    }
    let unit_exponent = (top - 52).max(-1074);
    let shift = exponent - unit_exponent;
    // Since k + shift = top - unit_exponent is in -1..=52, scaled_num < 2^106
    // and scaled_den < 2^107, so both and 2 * remainder fit in u128.
    let (scaled_num, scaled_den) = if shift >= 0 {
        (numerator << (shift as u32), denominator)
    } else {
        (numerator, denominator << ((-shift) as u32))
    };
    let mut retained = (scaled_num / scaled_den) as u64;
    let twice_rem = (scaled_num % scaled_den) << 1;
    if twice_rem > scaled_den || (twice_rem == scaled_den && retained & 1 != 0) {
        retained += 1;
    }
    if top < -1022 {
        return retained;
    }
    let mut rounded_top = top;
    if retained == 1_u64 << 53 {
        retained >>= 1;
        rounded_top += 1;
    }
    if rounded_top > 1023 {
        INFINITY_BITS
    } else {
        ((rounded_top + 1023) as u64) << 52 | (retained & FRACTION_MASK)
    }
}

#[cfg(test)]
mod tests {
    use super::{amount_from_per_100, per_100_from_amount};
    use crate::error::{Interval, NonFinite, ValueError};

    #[test]
    fn intermediate_overflow_and_underflow_are_rescued_by_single_rounding() {
        let tiny = f64::from_bits(1);
        // 100 * MAX would overflow f64, but with face = 100 the exact ratio is MAX.
        assert_eq!(
            per_100_from_amount(f64::MAX, 100.0)
                .expect("MAX per 100")
                .to_bits(),
            f64::MAX.to_bits()
        );
        // tiny / 100 would underflow f64, but 100 * tiny / 100 is tiny.
        assert_eq!(
            per_100_from_amount(tiny, 100.0)
                .expect("tiny per 100")
                .to_bits(),
            tiny.to_bits()
        );
        // MAX * 100 / 100 = MAX without intermediate overflow.
        assert_eq!(
            amount_from_per_100(100.0, f64::MAX)
                .expect("MAX amount")
                .to_bits(),
            f64::MAX.to_bits()
        );
        assert_eq!(
            amount_from_per_100(tiny, 100.0)
                .expect("tiny amount")
                .to_bits(),
            tiny.to_bits()
        );
    }

    #[test]
    fn subnormal_boundaries_and_overflow_carry_round_ties_to_even() {
        let tiny = f64::from_bits(1);
        // Exact midpoint 0.5 * tiny: 100 * tiny / 200 = 0.5 * tiny -> ties to even 0.0 (underflow).
        assert_eq!(
            per_100_from_amount(tiny, 200.0),
            Err(ValueError::ResultOutOfDomain {
                quantity: "price per 100",
                value: 0.0,
                domain: Interval::greater_than(0.0),
            })
        );
        // Slightly above 0.5 * tiny: 100 * tiny / 199 > 0.5 * tiny -> rounds up to tiny.
        assert_eq!(
            per_100_from_amount(tiny, 199.0)
                .expect("rounds up to tiny")
                .to_bits(),
            tiny.to_bits()
        );
        // Subnormal-to-normal carry: (2^52 - 1) * tiny * 100 / 99.99999999999999.
        let max_subnormal = f64::from_bits((1_u64 << 52) - 1);
        assert_eq!(
            per_100_from_amount(max_subnormal, 100.0_f64.next_down())
                .expect("carries into MIN_POSITIVE")
                .to_bits(),
            f64::MIN_POSITIVE.to_bits()
        );
        // Binade carry in normal range: (2^53 - 1) * 25 / 25 with remainder > 0.5.
        let near_two = f64::from_bits((1023_u64 << 52) | ((1_u64 << 52) - 1));
        assert_eq!(
            amount_from_per_100(near_two, 100.0_f64.next_up()).expect("carries to 2.0"),
            2.0
        );
        // Overflow carry at top = 1023 when significand rounds from 2^53 - 1 to 2^53.
        let max_minus_two = f64::from_bits(f64::MAX.to_bits() - 2);
        let hundred_minus_two = f64::from_bits(100.0_f64.to_bits() - 2);
        let hundred_plus_two = f64::from_bits(100.0_f64.to_bits() + 2);
        assert_eq!(
            per_100_from_amount(max_minus_two, hundred_minus_two),
            Err(ValueError::ResultNotFinite {
                quantity: "price per 100",
                value: NonFinite::PositiveInfinity,
            })
        );
        assert_eq!(
            amount_from_per_100(max_minus_two, hundred_plus_two),
            Err(ValueError::ResultNotFinite {
                quantity: "bond amount",
                value: NonFinite::PositiveInfinity,
            })
        );
        // Direct top > 1023 before rounding.
        assert_eq!(
            amount_from_per_100(f64::MAX, 100.0_f64.next_up()),
            Err(ValueError::ResultNotFinite {
                quantity: "bond amount",
                value: NonFinite::PositiveInfinity,
            })
        );
    }
}
