//! Exact finite-input binary64 triple-product rounding kernel.
//!
//! Each finite binary64 operand is an integer of at most 53 bits times a
//! power of two. Their exact product therefore needs at most 159 bits, held
//! in three u64 limbs. It is rounded once to nearest, with ties to even.
//! No intermediate floating product, scale, logarithm or exponential occurs.
//! The error wrapper interprets nonzero products rounded to zero/infinity.
//!
//! For highest exact exponent `top`, the retained unit is
//! `2^max(top - 52, -1074)`: 53 significant bits for normals, the fixed minimum
//! subnormal unit below that range. The next discarded bit is the guard;
//! all lower discarded bits form sticky. A guard with sticky rounds upward,
//! while an exact midpoint rounds upward only for an odd retained integer.
//! Carry can cross the minimum-normal or overflow boundary without a second
//! rounding. All exponent arithmetic stays between small signed integers.

const FRACTION_MASK: u64 = (1_u64 << 52) - 1;
const SIGN_BIT: u64 = 1_u64 << 63;
const INFINITY_BITS: u64 = 0x7ff0_0000_0000_0000;

/// Rounds the exact product of three already-validated finite operands once.
///
/// Exact zero is canonical positive zero. Nonzero products can produce signed
/// zero or infinity here; the parent wrapper rejects those arithmetic results.
pub(super) fn multiply(a: f64, b: f64, c: f64) -> f64 {
    let bits = [a.to_bits(), b.to_bits(), c.to_bits()];
    if bits.iter().any(|value| value & !SIGN_BIT == 0) {
        return 0.0;
    }
    let negative = (bits[0] ^ bits[1] ^ bits[2]) & SIGN_BIT != 0;
    let (first, first_exponent) = decompose(bits[0]);
    let (second, second_exponent) = decompose(bits[1]);
    let (third, third_exponent) = decompose(bits[2]);
    let magnitude = integer_product(first, second, third);
    // Each input exponent is between -1074 and 971, so their sum fits i32.
    let exponent = first_exponent + second_exponent + third_exponent;
    f64::from_bits(round(magnitude, exponent) | if negative { SIGN_BIT } else { 0 })
}

/// Returns the nonzero integer significand and its binary exponent.
fn decompose(bits: u64) -> (u64, i32) {
    let field = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & FRACTION_MASK;
    if field == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), field - 1075)
    }
}

/// Forms the exact little-endian, at-most-159-bit integer product.
#[expect(
    clippy::cast_possible_truncation,
    reason = "low-limb casts deliberately retain the bottom 64 bits; high limbs are bounded"
)]
fn integer_product(a: u64, b: u64, c: u64) -> [u64; 3] {
    let pair = u128::from(a) * u128::from(b);
    let low = u128::from(pair as u64) * u128::from(c);
    // pair's high limb has at most 42 bits; its product and the low carry
    // together have at most 95 bits, well inside u128.
    let high = (pair >> 64) * u128::from(c) + (low >> 64);
    [low as u64, high as u64, (high >> 64) as u64]
}

/// Rounds an exact positive integer times 2^exponent into magnitude bits.
#[expect(
    clippy::cast_sign_loss,
    reason = "the retained shift and finite biased exponent are nonnegative by construction"
)]
fn round(words: [u64; 3], exponent: i32) -> u64 {
    let top = highest_bit(&words) + exponent;
    if top > 1023 {
        return INFINITY_BITS;
    }
    let unit_exponent = (top - 52).max(-1074);
    // Any normal input contributes at least 53 significand bits, while a
    // product of three subnormals needs a right shift to the subnormal unit.
    // Consequently this shift is always nonnegative for nonzero finite inputs.
    let shift = (unit_exponent - exponent) as u32;
    let mut retained = shifted_low(&words, shift);
    if shift != 0 {
        let guard = bit(&words, shift - 1);
        let sticky = any_below(&words, shift - 1);
        if guard && (sticky || retained & 1 != 0) {
            retained += 1;
        }
    }
    if top < -1022 {
        // At most 2^52 is retained. That endpoint is the bit encoding of
        // MIN_POSITIVE itself, so rounding across this boundary needs no fixup.
        return retained;
    }
    let mut rounded_top = top;
    if retained == 1_u64 << 53 {
        retained >>= 1;
        rounded_top += 1;
    }
    if rounded_top > 1023 {
        // The exact overflow midpoint is 2^1024 - 2^970. MAX's significand
        // is odd; its tie therefore carries to the even successor/infinity.
        INFINITY_BITS
    } else {
        ((rounded_top + 1023) as u64) << 52 | (retained & FRACTION_MASK)
    }
}

/// Finds the highest set bit of a nonzero, at-most-159-bit magnitude.
#[expect(
    clippy::cast_possible_wrap,
    reason = "leading-zero counts are at most 64"
)]
fn highest_bit(words: &[u64; 3]) -> i32 {
    if words[2] != 0 {
        191 - words[2].leading_zeros() as i32
    } else if words[1] != 0 {
        127 - words[1].leading_zeros() as i32
    } else {
        63 - words[0].leading_zeros() as i32
    }
}

/// Returns the lowest 64 bits after an arbitrary nonnegative right shift.
fn shifted_low(words: &[u64; 3], shift: u32) -> u64 {
    let limb = (shift / 64) as usize;
    if limb >= words.len() {
        return 0;
    }
    let offset = shift % 64;
    let low = words[limb] >> offset;
    if offset == 0 || limb + 1 == words.len() {
        low
    } else {
        low | words[limb + 1] << (64 - offset)
    }
}

/// Reads a bit, treating every position outside the fixed integer as zero.
fn bit(words: &[u64; 3], position: u32) -> bool {
    let limb = (position / 64) as usize;
    limb < words.len() && words[limb] & (1_u64 << (position % 64)) != 0
}

/// Reports whether any bit strictly below `position` is nonzero.
fn any_below(words: &[u64; 3], position: u32) -> bool {
    let complete = (position / 64) as usize;
    let remainder = position % 64;
    if complete >= words.len() {
        return words.iter().any(|value| *value != 0);
    }
    words[..complete].iter().any(|value| *value != 0)
        || (remainder != 0 && words[complete] & ((1_u64 << remainder) - 1) != 0)
}
