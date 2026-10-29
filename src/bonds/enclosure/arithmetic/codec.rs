//! Complete bounded exact rational point encoder, written before oracle release.
//!
//! Input is signed Raw0.N/Raw0.D * 2^exponent. Prefer a fitting canonical decimal
//! point, otherwise an exact finite binary64 point, otherwise NoFittingPoint.
//! Raw0/E0 remain unchanged for a caller's separately justified interval fallback.
//! Raw1/W0/W1/Q may be clobbered. Discard all partial working fields on error.
#![forbid(unsafe_code)]

use super::scalar::{Frame, R, W, Work};
use std::cmp::Ordering;

/// Compact point result. No containment or financial-origin premise is created.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::bonds::enclosure) enum Encoding {
    /// Canonical coefficient with scale at most 38 and asymmetric i128 limits.
    Decimal { coefficient: i128, scale: u32 },
    /// Exact finite IEEE 754 binary64 bits, with no rounding.
    Binary64 { bits: u64 },
    /// The exact rational fits neither preferred point representation.
    NoFittingPoint,
}

/// Private checked failures; resource exhaustion is not a mathematical verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::bonds::enclosure) enum Error {
    DenominatorZero,
    Capacity,
    Invariant,
    WorkLimit,
}

type Result<T> = std::result::Result<T, Error>;

fn charge(work: &mut Work, count: usize) -> Result<()> {
    let count = u64::try_from(count).map_err(|_| Error::Capacity)?;
    let next = work.used.checked_add(count).ok_or(Error::WorkLimit)?;
    if next > work.max {
        return Err(Error::WorkLimit);
    }
    work.used = next;
    Ok(())
}

fn zero(words: &[u64], work: &mut Work) -> Result<bool> {
    charge(work, words.len())?;
    Ok(words.iter().all(|&word| word == 0))
}

fn bits(words: &[u64], work: &mut Work) -> Result<usize> {
    charge(work, words.len())?;
    for (index, &word) in words.iter().enumerate().rev() {
        if word != 0 {
            return Ok(index * 64 + (64 - word.leading_zeros() as usize));
        }
    }
    Ok(0)
}

fn remainder_small(words: &[u64; R], prime: u32, work: &mut Work) -> Result<u32> {
    charge(work, 4 * R + 4)?;
    if !matches!(prime, 2 | 5) {
        return Err(Error::Invariant);
    }
    let mut remainder = 0_u128;
    for &word in words.iter().rev() {
        remainder = ((remainder << 64) | u128::from(word)) % u128::from(prime);
    }
    Ok(remainder as u32)
}

fn divide_small(words: &mut [u64; R], prime: u32, work: &mut Work) -> Result<u32> {
    charge(work, 6 * R + 4)?;
    if !matches!(prime, 2 | 5) {
        return Err(Error::Invariant);
    }
    let mut remainder = 0_u128;
    for word in words.iter_mut().rev() {
        let value = (remainder << 64) | u128::from(*word);
        *word = (value / u128::from(prime)) as u64;
        remainder = value % u128::from(prime);
    }
    Ok(remainder as u32)
}

/// Nonzero input; each successful division strictly decreases its bit length.
fn strip(words: &mut [u64; R], prime: u32, work: &mut Work) -> Result<u32> {
    for count in 0..64 * R {
        charge(work, 2)?;
        let remainder = if prime == 2 {
            (words[0] & 1) as u32
        } else {
            remainder_small(words, prime, work)?
        };
        if remainder != 0 {
            return Ok(count as u32);
        }
        if divide_small(words, prime, work)? != 0 {
            return Err(Error::Invariant);
        }
    }
    // A positive Raw34 cannot contain 2176 factors of either prime.
    Err(Error::Invariant)
}

fn compare(a: &[u64; W], b: &[u64; W], work: &mut Work) -> Result<Ordering> {
    charge(work, W)?;
    for (x, y) in a.iter().zip(b).rev() {
        if x != y {
            return Ok(x.cmp(y));
        }
    }
    Ok(Ordering::Equal)
}

fn subtract(a: &mut [u64; W], b: &[u64; W], work: &mut Work) -> Result<()> {
    charge(work, 5 * W + 2)?;
    let mut borrow = false;
    for (x, y) in a.iter_mut().zip(b) {
        let (first, b1) = x.overflowing_sub(*y);
        let (second, b2) = first.overflowing_sub(u64::from(borrow));
        *x = second;
        borrow = b1 || b2;
    }
    if borrow {
        return Err(Error::Invariant);
    }
    Ok(())
}

fn right_one(words: &mut [u64; W], work: &mut Work) -> Result<()> {
    charge(work, 3 * W + 2)?;
    let mut carry = 0_u64;
    for word in words.iter_mut().rev() {
        let next = *word << 63;
        *word = (*word >> 1) | carry;
        carry = next;
    }
    Ok(())
}

fn left_copy(input: &[u64; R], output: &mut [u64; W], shift: usize, work: &mut Work) -> Result<()> {
    charge(work, W + 8 * R + 8)?;
    output.fill(0);
    if shift >= R * 64 {
        return Err(Error::Capacity);
    }
    let offset = shift / 64;
    let tail = shift % 64;
    for (index, &word) in input.iter().enumerate() {
        if word == 0 {
            continue;
        }
        let destination = offset + index;
        *output.get_mut(destination).ok_or(Error::Capacity)? |= word << tail;
        if tail != 0 {
            let carry = word >> (64 - tail);
            if carry != 0 {
                *output.get_mut(destination + 1).ok_or(Error::Capacity)? |= carry;
            }
        }
    }
    Ok(())
}

/// Divide original Raw0.N by residual Raw1.D; commit quotient only if exact.
fn exact_quotient(frame: &mut Frame, work: &mut Work) -> Result<bool> {
    let numerator_bits = bits(&frame.raw0.n, work)?;
    let denominator_bits = bits(&frame.raw1.d, work)?;
    charge(work, 4)?;
    if denominator_bits == 0 {
        return Err(Error::Invariant);
    }
    if numerator_bits < denominator_bits {
        return Ok(false);
    }
    let shift = numerator_bits - denominator_bits;
    charge(work, 2 * W + R)?;
    frame.w0.fill(0);
    frame.q.fill(0);
    for (to, from) in frame.w0.iter_mut().zip(&frame.raw0.n) {
        *to = *from;
    }
    left_copy(&frame.raw1.d, &mut frame.w1, shift, work)?;
    // Relative quotient width <=2176, independent of the supplied binary E.
    for bit in (0..=shift).rev() {
        charge(work, 2)?;
        if compare(&frame.w0, &frame.w1, work)? != Ordering::Less {
            subtract(&mut frame.w0, &frame.w1, work)?;
            charge(work, 2)?;
            *frame.q.get_mut(bit / 64).ok_or(Error::Capacity)? |= 1_u64 << (bit % 64);
        }
        right_one(&mut frame.w1, work)?;
    }
    if !zero(&frame.w0, work)? {
        return Ok(false);
    }
    charge(work, W)?;
    if frame.q[R..].iter().any(|&word| word != 0) {
        return Err(Error::Capacity);
    }
    for (to, from) in frame.raw1.n.iter_mut().zip(&frame.q) {
        *to = *from;
    }
    Ok(true)
}

/// P is positive and coprime to 2 and 5; denominator exponents may be negative.
fn decimal(
    p: &[u64; R],
    a: i64,
    b: i64,
    negative: bool,
    work: &mut Work,
) -> Result<Option<Encoding>> {
    charge(work, 8)?;
    let scale = a.max(b).max(0);
    let power_two = scale - a;
    let power_five = scale - b;
    if scale > 38 || power_two > 127 || power_five > 55 {
        return Ok(None);
    }
    if power_two < 0 || power_five < 0 {
        return Err(Error::Invariant);
    }
    if bits(p, work)? > 128 {
        return Ok(None);
    }
    charge(work, 8)?;
    let mut magnitude = u128::from(p[0]) | (u128::from(p[1]) << 64);
    let limit = if negative {
        1_u128 << 127
    } else {
        (1_u128 << 127) - 1
    };
    let power_two = power_two as u32;
    if magnitude > (limit >> power_two) {
        return Ok(None);
    }
    magnitude <<= power_two;
    for _ in 0..power_five {
        charge(work, 6)?;
        if magnitude > limit / 5 {
            return Ok(None);
        }
        magnitude *= 5;
    }
    charge(work, 8)?;
    let coefficient = if negative {
        if magnitude == 1_u128 << 127 {
            i128::MIN
        } else {
            -(magnitude as i128)
        }
    } else {
        magnitude as i128
    };
    // At positive scale at least one added prime power is zero, hence canonical.
    Ok(Some(Encoding::Decimal {
        coefficient,
        scale: scale as u32,
    }))
}

fn binary(
    p: &[u64; R],
    a: i64,
    b: i64,
    negative: bool,
    work: &mut Work,
) -> Result<Option<Encoding>> {
    charge(work, 8)?;
    if b > 0 || -b > 22 {
        return Ok(None);
    }
    if bits(p, work)? > 53 {
        return Ok(None);
    }
    charge(work, 4)?;
    let mut odd = p[0];
    let limit = (1_u64 << 53) - 1;
    for _ in 0..-b {
        charge(work, 6)?;
        if odd > limit / 5 {
            return Ok(None);
        }
        odd *= 5;
    }
    charge(work, 12)?;
    if odd == 0 {
        return Err(Error::Invariant);
    }
    let significant_bits = 64 - odd.leading_zeros();
    let unit_exponent = -a;
    let k = i64::from(significant_bits) - 1 + unit_exponent;
    if unit_exponent < -1074 || k > 1023 {
        return Ok(None);
    }
    let magnitude;
    if k >= -1022 {
        // Only bounded shifts enter IEEE assembly after the range proof above.
        let significand = odd << (53 - significant_bits);
        let encoded_exponent = (k + 1023) as u64;
        magnitude = (encoded_exponent << 52) | (significand & ((1_u64 << 52) - 1));
    } else {
        let shift = unit_exponent + 1074;
        if !(0..=51).contains(&shift) {
            return Err(Error::Invariant);
        }
        magnitude = odd.checked_shl(shift as u32).ok_or(Error::Invariant)?;
        if magnitude == 0 || magnitude >= 1_u64 << 52 {
            return Err(Error::Invariant);
        }
    }
    Ok(Some(Encoding::Binary64 {
        bits: magnitude | if negative { 1_u64 << 63 } else { 0 },
    }))
}

/// Encode the exact signed rational in Raw0, preserving original Raw0 and E0.
///
/// Every variable loop is capped by Raw34 or a small proven point-fit bound,
/// and every block is charged before execution. An unfitting point is a normal
/// outcome; WorkLimit never masquerades as NoFittingPoint. The supplied i32 E
/// is handled as signed counts, never an absolute allocation/shift/loop length.
pub(in crate::bonds::enclosure) fn encode(
    negative: bool,
    exponent: i32,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<Encoding> {
    charge(work, 4)?;
    if zero(&frame.raw0.d, work)? {
        return Err(Error::DenominatorZero);
    }
    if zero(&frame.raw0.n, work)? {
        return Ok(Encoding::Decimal {
            coefficient: 0,
            scale: 0,
        });
    }
    charge(work, 2 * R + 2)?;
    for (to, from) in frame.raw1.n.iter_mut().zip(&frame.raw0.n) {
        *to = *from;
    }
    for (to, from) in frame.raw1.d.iter_mut().zip(&frame.raw0.d) {
        *to = *from;
    }
    frame.raw1.controls.fill(0);
    let denominator_twos = strip(&mut frame.raw1.d, 2, work)?;
    let denominator_fives = strip(&mut frame.raw1.d, 5, work)?;
    if !exact_quotient(frame, work)? {
        return Ok(Encoding::NoFittingPoint);
    }
    let numerator_twos = strip(&mut frame.raw1.n, 2, work)?;
    let numerator_fives = strip(&mut frame.raw1.n, 5, work)?;
    charge(work, 8)?;
    // Counts <=2175 and E is i32, so all net counts fit i64 even beyond i32.
    let a = i64::from(denominator_twos) - i64::from(numerator_twos) - i64::from(exponent);
    let b = i64::from(denominator_fives) - i64::from(numerator_fives);
    if let Some(point) = decimal(&frame.raw1.n, a, b, negative, work)? {
        return Ok(point);
    }
    if let Some(point) = binary(&frame.raw1.n, a, b, negative, work)? {
        return Ok(point);
    }
    Ok(Encoding::NoFittingPoint)
}
