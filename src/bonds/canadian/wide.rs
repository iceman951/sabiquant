//! Caller-owned bounded unsigned integers for the Canadian source model.
//!
//! Magnitudes are little-endian slices of at most 243 u64 words. Every owned
//! array belongs to the caller's explicit frame. Arithmetic takes borrowed
//! inputs and writes supplied outputs; no helper creates or returns a wide
//! value. Failed checked operations may leave their outputs partially changed.
//! Exact IEEE zero/infinity results are successful observations, not failures.

use std::cmp::Ordering;

const MAX_WORDS: usize = 243;
const FRACTION: u64 = (1_u64 << 52) - 1;
const INFINITY: u64 = 0x7ff0_0000_0000_0000;

fn bounded(words: &[u64]) -> bool {
    words.len() <= MAX_WORDS
}

fn word(words: &[u64], index: usize) -> u64 {
    words.get(index).copied().unwrap_or(0)
}

/// Clears a caller-owned magnitude.
pub(super) fn clear(out: &mut [u64]) {
    out.fill(0);
}

/// Reports exact zero, accepting leading zero words.
pub(super) fn is_zero(a: &[u64]) -> bool {
    a.iter().all(|value| *value == 0)
}

/// Reports the low integer bit; an empty magnitude is zero.
pub(super) fn is_odd(a: &[u64]) -> bool {
    word(a, 0) & 1 != 0
}

/// Returns significant bits; callers supply at most 243 words.
pub(super) fn bits(a: &[u64]) -> u32 {
    for (index, value) in a.iter().enumerate().rev() {
        if *value != 0 {
            return u32::try_from(index)
                .ok()
                .and_then(|index| index.checked_mul(64))
                .and_then(|base| base.checked_add(64 - value.leading_zeros()))
                .unwrap_or(u32::MAX);
        }
    }
    0
}

/// Compares mathematical magnitudes independently of their slice widths.
pub(super) fn cmp(a: &[u64], b: &[u64]) -> Ordering {
    match bits(a).cmp(&bits(b)) {
        Ordering::Equal => {
            for index in (0..a.len().max(b.len())).rev() {
                match word(a, index).cmp(&word(b, index)) {
                    Ordering::Equal => {}
                    result => return result,
                }
            }
            Ordering::Equal
        }
        result => result,
    }
}

fn shifted_word(a: &[u64], index: usize, whole: usize, offset: u32) -> u64 {
    let Some(source) = index.checked_sub(whole) else {
        return 0;
    };
    let low = word(a, source) << offset;
    if offset == 0 {
        low
    } else {
        low | source
            .checked_sub(1)
            .map_or(0, |i| word(a, i) >> (64 - offset))
    }
}

/// Compares `a*2^shift` with b without constructing a shifted magnitude.
pub(super) fn cmp_shifted(a: &[u64], shift: u32, b: &[u64]) -> Ordering {
    let ab = bits(a);
    if ab == 0 {
        return if is_zero(b) {
            Ordering::Equal
        } else {
            Ordering::Less
        };
    }
    let shifted_bits = u64::from(ab) + u64::from(shift);
    match shifted_bits.cmp(&u64::from(bits(b))) {
        Ordering::Equal => {
            // Equal widths imply shift <= 243*64 for the admitted workspaces.
            let whole = usize::try_from(shift / 64).unwrap_or(usize::MAX);
            for index in (0..b.len()).rev() {
                match shifted_word(a, index, whole, shift % 64).cmp(&b[index]) {
                    Ordering::Equal => {}
                    result => return result,
                }
            }
            Ordering::Equal
        }
        result => result,
    }
}

/// Copies into a potentially different width, rejecting every lost high bit.
pub(super) fn copy_into(a: &[u64], out: &mut [u64]) -> bool {
    if !bounded(a) || !bounded(out) || a.iter().skip(out.len()).any(|v| *v != 0) {
        return false;
    }
    clear(out);
    for (target, source) in out.iter_mut().zip(a) {
        *target = *source;
    }
    true
}

/// Embeds a u128 exactly into supplied storage.
#[expect(
    clippy::cast_possible_truncation,
    reason = "low cast extracts one exact limb"
)]
pub(super) fn set_u128(out: &mut [u64], value: u128) -> bool {
    if !bounded(out) || (out.is_empty() && value != 0) || (out.len() == 1 && value >> 64 != 0) {
        return false;
    }
    clear(out);
    if let Some(low) = out.first_mut() {
        *low = value as u64;
    }
    if let Some(high) = out.get_mut(1) {
        *high = (value >> 64) as u64;
    }
    true
}

/// Returns a native value only if all high words are zero.
pub(super) fn to_u128(a: &[u64]) -> Option<u128> {
    if !bounded(a) || a.iter().skip(2).any(|v| *v != 0) {
        None
    } else {
        Some(u128::from(word(a, 0)) | (u128::from(word(a, 1)) << 64))
    }
}

/// Adds a borrowed magnitude in place, retaining every carry.
pub(super) fn add_assign(out: &mut [u64], a: &[u64]) -> bool {
    if !bounded(a) || !bounded(out) || a.iter().skip(out.len()).any(|v| *v != 0) {
        return false;
    }
    let mut carry = false;
    for (index, target) in out.iter_mut().enumerate() {
        let (sum, first) = target.overflowing_add(word(a, index));
        let (sum, second) = sum.overflowing_add(u64::from(carry));
        *target = sum;
        carry = first || second;
    }
    !carry
}

fn add_word_at(out: &mut [u64], mut index: usize, mut value: u64) -> bool {
    while value != 0 {
        let Some(target) = out.get_mut(index) else {
            return false;
        };
        let (sum, carry) = target.overflowing_add(value);
        *target = sum;
        value = u64::from(carry);
        index += 1;
    }
    true
}

/// Adds one scalar without a temporary wide magnitude.
pub(super) fn add_small_assign(out: &mut [u64], a: u64) -> bool {
    bounded(out) && add_word_at(out, 0, a)
}

/// Inserts a shifted scalar directly into an accumulator, without a term array.
#[expect(
    clippy::cast_possible_truncation,
    reason = "low cast extracts a scalar product limb"
)]
pub(super) fn add_shifted_u64(out: &mut [u64], value: u64, shift: u32) -> bool {
    if !bounded(out) {
        return false;
    }
    if value == 0 {
        return true;
    }
    if u64::from(64 - value.leading_zeros()) + u64::from(shift)
        > u64::try_from(out.len()).unwrap_or(0) * 64
    {
        return false;
    }
    let Ok(index) = usize::try_from(shift / 64) else {
        return false;
    };
    let product = u128::from(value) << (shift % 64);
    add_word_at(out, index, product as u64) && add_word_at(out, index + 1, (product >> 64) as u64)
}

/// Subtracts in place, rejecting a negative result before mutation.
pub(super) fn sub_assign(out: &mut [u64], a: &[u64]) -> bool {
    if !bounded(out) || !bounded(a) || cmp(out, a) == Ordering::Less {
        return false;
    }
    let mut borrow = false;
    for (index, target) in out.iter_mut().enumerate() {
        let (difference, first) = target.overflowing_sub(word(a, index));
        let (difference, second) = difference.overflowing_sub(u64::from(borrow));
        *target = difference;
        borrow = first || second;
    }
    !borrow
}

fn shift_fits(a: &[u64], shift: u32, width: usize) -> bool {
    is_zero(a) || u64::from(bits(a)) + u64::from(shift) <= u64::try_from(width).unwrap_or(0) * 64
}

/// Shifts into separate supplied storage without dropping high nonzero bits.
pub(super) fn shl_into(a: &[u64], shift: u32, out: &mut [u64]) -> bool {
    if !bounded(a) || !bounded(out) || !shift_fits(a, shift, out.len()) {
        return false;
    }
    if is_zero(a) {
        clear(out);
        return true;
    }
    let Ok(whole) = usize::try_from(shift / 64) else {
        return false;
    };
    for (index, target) in out.iter_mut().enumerate() {
        *target = shifted_word(a, index, whole, shift % 64);
    }
    true
}

/// Shifts left in place, reading high words first to preserve its source.
pub(super) fn shl_assign(a: &mut [u64], shift: u32) -> bool {
    if !bounded(a) || !shift_fits(a, shift, a.len()) {
        return false;
    }
    if is_zero(a) {
        return true;
    }
    let Ok(whole) = usize::try_from(shift / 64) else {
        return false;
    };
    for index in (0..a.len()).rev() {
        a[index] = shifted_word(a, index, whole, shift % 64);
    }
    true
}

/// Shifts right in place, discarding only exact low remainder bits.
pub(super) fn shr_assign(a: &mut [u64], shift: u32) {
    let Ok(whole) = usize::try_from(shift / 64) else {
        clear(a);
        return;
    };
    if whole >= a.len() {
        clear(a);
        return;
    }
    let offset = shift % 64;
    for index in 0..a.len() {
        let source = index + whole;
        let mut value = word(a, source) >> offset;
        if offset != 0 {
            value |= word(a, source + 1) << (64 - offset);
        }
        a[index] = value;
    }
}

/// Multiplies into a distinct output with one scalar u128 row carry.
#[expect(
    clippy::cast_possible_truncation,
    reason = "low casts retain limbs; row carry is checked"
)]
pub(super) fn mul_into(a: &[u64], b: &[u64], out: &mut [u64]) -> bool {
    if !bounded(a) || !bounded(b) || !bounded(out) {
        return false;
    }
    clear(out);
    for (i, first) in a.iter().copied().enumerate() {
        if first == 0 {
            continue;
        }
        let mut carry = 0_u128;
        for (j, second) in b.iter().copied().enumerate() {
            let position = i + j;
            if position >= out.len() {
                if second != 0 || carry != 0 {
                    return false;
                }
                continue;
            }
            let product =
                u128::from(first) * u128::from(second) + u128::from(out[position]) + carry;
            out[position] = product as u64;
            carry = product >> 64;
        }
        if !add_word_at(out, i + b.len(), carry as u64) {
            return false;
        }
    }
    true
}

/// Multiplies in place by a scalar, checking the final row carry.
#[expect(
    clippy::cast_possible_truncation,
    reason = "low cast extracts a limb; final carry is checked"
)]
pub(super) fn mul_small_assign(a: &mut [u64], factor: u64) -> bool {
    if !bounded(a) {
        return false;
    }
    let mut carry = 0_u128;
    for target in a {
        let product = u128::from(*target) * u128::from(factor) + carry;
        *target = product as u64;
        carry = product >> 64;
    }
    carry == 0
}

/// Constructs a decimal power in supplied storage, stopping at first overflow.
pub(super) fn pow10_into(exponent: u16, out: &mut [u64]) -> bool {
    if !set_u128(out, 1) {
        return false;
    }
    for _ in 0..exponent {
        if !mul_small_assign(out, 10) {
            return false;
        }
    }
    true
}

/// Divides in place by a scalar and returns its exact remainder.
#[expect(
    clippy::cast_possible_truncation,
    reason = "quotient limb and remainder fit their declared widths"
)]
pub(super) fn div_rem_small_assign(a: &mut [u64], divisor: u32) -> Option<u32> {
    if !bounded(a) || divisor == 0 {
        return None;
    }
    let mut remainder = 0_u128;
    for target in a.iter_mut().rev() {
        let dividend = (remainder << 64) | u128::from(*target);
        *target = (dividend / u128::from(divisor)) as u64;
        remainder = dividend % u128::from(divisor);
    }
    Some(remainder as u32)
}

/// Full quotient/remainder division with caller-owned quotient and two work slots.
pub(super) fn div_rem_into(
    n: &[u64],
    d: &[u64],
    q: &mut [u64],
    r: &mut [u64],
    aligned: &mut [u64],
) -> bool {
    if !bounded(d) || !bounded(q) || is_zero(d) || !copy_into(n, r) {
        return false;
    }
    clear(q);
    clear(aligned);
    if cmp(r, d) == Ordering::Less {
        return true;
    }
    let distance = bits(r) - bits(d);
    if !shl_into(d, distance, aligned) {
        return false;
    }
    for position in (0..=distance).rev() {
        if cmp(r, aligned) != Ordering::Less {
            if !sub_assign(r, aligned) {
                return false;
            }
            let Ok(index) = usize::try_from(position / 64) else {
                return false;
            };
            let Some(target) = q.get_mut(index) else {
                return false;
            };
            *target |= 1_u64 << (position % 64);
        }
        shr_assign(aligned, 1);
    }
    true
}

fn scalar_division_in_place(r: &mut [u64], d: &[u64], aligned: &mut [u64]) -> Option<u64> {
    if !bounded(r) || !bounded(d) || !bounded(aligned) || is_zero(d) {
        return None;
    }
    if cmp(r, d) == Ordering::Less {
        clear(aligned);
        return Some(0);
    }
    let distance = bits(r) - bits(d);
    // distance64 may still yield a64-bit quotient if its high test subtracts0.
    if distance > 64 || !shl_into(d, distance, aligned) {
        return None;
    }
    let mut quotient = 0_u64;
    for position in (0..=distance).rev() {
        if cmp(r, aligned) != Ordering::Less {
            quotient |= 1_u64.checked_shl(position)?;
            if !sub_assign(r, aligned) {
                return None;
            }
        }
        shr_assign(aligned, 1);
    }
    Some(quotient)
}

/// Scalar-quotient division, rejecting an actual quotient beyond u64.
#[cfg(test)]
pub(super) fn div_rem_u64(n: &[u64], d: &[u64], r: &mut [u64], aligned: &mut [u64]) -> Option<u64> {
    if !copy_into(n, r) {
        return None;
    }
    scalar_division_in_place(r, d, aligned)
}

fn ratio_floor_log(n: &[u64], d: &[u64]) -> Option<i32> {
    if !bounded(n) || !bounded(d) || is_zero(n) || is_zero(d) {
        return None;
    }
    let k0 = i32::try_from(bits(n)).ok()? - i32::try_from(bits(d)).ok()?;
    let at_least = if k0 >= 0 {
        cmp_shifted(d, k0.unsigned_abs(), n) != Ordering::Greater
    } else {
        cmp_shifted(n, k0.unsigned_abs(), d) != Ordering::Less
    };
    Some(if at_least { k0 } else { k0 - 1 })
}

/// Feeds one zero bit to an exact remainder, including a virtual high carry.
/// The invariant0<=r<d makes the result fit the original denominator width.
fn double_remainder_in_place(r: &mut [u64], d: &[u64]) -> Option<bool> {
    if !bounded(r)
        || !bounded(d)
        || is_zero(d)
        || cmp(r, d) != Ordering::Less
        || u64::from(bits(d)) > u64::try_from(r.len()).ok()? * 64
    {
        return None;
    }
    let subtract = cmp_shifted(r, 1, d) != Ordering::Less;
    let mut carry = 0;
    let mut borrow = false;
    for (index, target) in r.iter_mut().enumerate() {
        let high = *target >> 63;
        let shifted = (*target << 1) | carry;
        carry = high;
        if subtract {
            let (difference, first) = shifted.overflowing_sub(word(d, index));
            let (difference, second) = difference.overflowing_sub(u64::from(borrow));
            *target = difference;
            borrow = first || second;
        } else {
            *target = shifted;
        }
    }
    // Any virtual carry cancels the subtraction's final borrow. No high bit
    // is discarded:2*r-d<d, while a non-subtracted2*r was already below d.
    (carry == u64::from(borrow)).then_some(subtract)
}

fn scalar_division_with_left_shift(
    n: &[u64],
    d: &[u64],
    mut shift: u32,
    u: &mut [u64],
    v: &mut [u64],
    aligned: &mut [u64],
) -> Option<u64> {
    if !copy_into(n, u) || !copy_into(d, v) {
        return None;
    }
    let mut quotient = scalar_division_in_place(u, v, aligned)?;
    if quotient == 0 && !is_zero(u) {
        // Skip leading quotient zeroes with a shift that is strictly below
        // the denominator's bit width. Only the final at-most64 quotient
        // bits need the virtual-carry feed loop.
        let leap = bits(v).saturating_sub(bits(u) + 1).min(shift);
        if !shl_assign(u, leap) {
            return None;
        }
        shift -= leap;
    }
    for _ in 0..shift {
        let bit = double_remainder_in_place(u, v)?;
        quotient = quotient.checked_mul(2)?.checked_add(u64::from(bit))?;
    }
    Some(quotient)
}

fn rounded_scalar(
    n: &[u64],
    d: &[u64],
    shift: i64,
    u: &mut [u64],
    v: &mut [u64],
    aligned: &mut [u64],
) -> Option<u64> {
    let shift_bits = u32::try_from(shift.unsigned_abs()).ok()?;
    if shift >= 0 && !shift_fits(n, shift_bits, u.len()) {
        // Generic maximal input pairs can require53 more dividend bits than
        // their storage. Stream those bits through a denominator-sized
        // remainder instead of constructing the shifted dividend or widening
        // the caller's fixed workspace. This is an exact preclassified route,
        // not a precision retry after a failed rounding observation.
        let q = scalar_division_with_left_shift(n, d, shift_bits, u, v, aligned)?;
        let tie = cmp_shifted(u, 1, v);
        return q.checked_add(u64::from(
            tie == Ordering::Greater || (tie == Ordering::Equal && q & 1 != 0),
        ));
    }
    let fits = if shift >= 0 {
        shl_into(n, shift_bits, u) && copy_into(d, v)
    } else {
        copy_into(n, u) && shl_into(d, shift_bits, v)
    };
    if !fits {
        return None;
    }
    let q = scalar_division_in_place(u, v, aligned)?;
    let tie = cmp_shifted(u, 1, v);
    q.checked_add(u64::from(
        tie == Ordering::Greater || (tie == Ordering::Equal && q & 1 != 0),
    ))
}

/// Direct nearest/even IEEE magnitude, using exactly three supplied work slots.
///
/// Zero and infinity bits are legitimate rounded results. None denotes a zero
/// denominator or insufficient caller storage, never an IEEE range result.
pub(super) fn round_to_f64(
    n: &[u64],
    d: &[u64],
    binary_exp: i32,
    work_num: &mut [u64],
    work_den: &mut [u64],
    aligned: &mut [u64],
) -> Option<u64> {
    if !bounded(n) || !bounded(d) || is_zero(d) {
        return None;
    }
    if is_zero(n) {
        return Some(0);
    }
    let k = ratio_floor_log(n, d)?;
    let top = i64::from(k) + i64::from(binary_exp);
    if top > 1023 {
        return Some(INFINITY);
    }
    if top < -1075 {
        return Some(0);
    }
    let unit_exp = (top - 52).max(-1074);
    let mut q = rounded_scalar(
        n,
        d,
        i64::from(binary_exp) - unit_exp,
        work_num,
        work_den,
        aligned,
    )?;
    if top < -1022 {
        return Some(q);
    }
    let mut rounded_top = top;
    if q == 1_u64 << 53 {
        q >>= 1;
        rounded_top += 1;
    }
    if rounded_top > 1023 {
        Some(INFINITY)
    } else {
        Some((u64::try_from(rounded_top + 1023).ok()? << 52) | (q & FRACTION))
    }
}

/// Whole-rational normalized53 RNE, retaining a separate scalar unit exponent.
pub(super) fn round_to_extended53(
    n: &[u64],
    d: &[u64],
    binary_exp: i32,
    work_num: &mut [u64],
    work_den: &mut [u64],
    aligned: &mut [u64],
) -> Option<(u64, i32)> {
    if !bounded(n) || !bounded(d) || is_zero(d) {
        return None;
    }
    if is_zero(n) {
        return Some((0, 0));
    }
    let k = ratio_floor_log(n, d)?;
    let mut q = rounded_scalar(n, d, i64::from(52 - k), work_num, work_den, aligned)?;
    let mut e = i64::from(binary_exp) + i64::from(k) - 52;
    if q == 1_u64 << 53 {
        q >>= 1;
        e += 1;
    }
    Some((q, i32::try_from(e).ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn native(value: u128) -> [u64; 5] {
        let mut out = [0; 5];
        assert!(set_u128(&mut out, value));
        out
    }

    fn step(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    /// FR-CAD-32/40/47/51/65: native exact integers cross-check all base operations.
    #[test]
    fn independent_u128_arithmetic_and_division() {
        let mut state = 0x5e29_b14a_981c_063d;
        for _ in 0..4000 {
            let a = (u128::from(step(&mut state)) << 64) | u128::from(step(&mut state));
            let b = (u128::from(step(&mut state)) << 64) | u128::from(step(&mut state));
            let na = native(a);
            let nb = native(b);
            assert_eq!(cmp(&na, &nb), a.cmp(&b));
            assert_eq!(is_odd(&na), a & 1 != 0);
            let mut sum = na;
            assert!(add_assign(&mut sum, &nb));
            assert_eq!(to_u128(&sum), a.checked_add(b));
            let mut difference = na;
            assert_eq!(sub_assign(&mut difference, &nb), a >= b);
            if a >= b {
                assert_eq!(to_u128(&difference), Some(a - b));
            }
            let mut product = [0; 5];
            assert!(mul_into(&na, &nb, &mut product));
            assert_eq!(to_u128(&product), a.checked_mul(b));
            let mut q = [0; 5];
            let mut r = [0; 5];
            let mut aligned = [0; 5];
            assert_eq!(div_rem_into(&na, &nb, &mut q, &mut r, &mut aligned), b != 0);
            if let Some(expected) = a.checked_div(b) {
                assert_eq!(to_u128(&q), Some(expected));
                assert_eq!(to_u128(&r), Some(a % b));
            }
            let divisor = u32::try_from(step(&mut state) & u64::from(u32::MAX)).unwrap_or(0);
            let mut small = na;
            let remainder = div_rem_small_assign(&mut small, divisor);
            assert_eq!(
                remainder.map(u128::from),
                a.checked_rem(u128::from(divisor))
            );
            if let Some(expected) = a.checked_div(u128::from(divisor)) {
                assert_eq!(to_u128(&small), Some(expected));
            }
        }
    }

    /// FR-CAD-51/65: every limb boundary, carry/borrow and maximum-width insertion.
    #[test]
    fn every_supported_capacity_and_carry_boundary() {
        for width in [5, 8, 20, 40, 112, 243] {
            let mut value = vec![0; width];
            for bit in 0..width * 64 {
                clear(&mut value);
                value[bit / 64] = 1_u64 << (bit % 64);
                assert_eq!(bits(&value), u32::try_from(bit + 1).unwrap_or(0));
                let mut previous = value.clone();
                assert!(sub_assign(&mut previous, &[1]));
                assert!(add_small_assign(&mut previous, 1));
                assert_eq!(previous, value);
                let mut inserted = vec![0; width];
                assert!(add_shifted_u64(
                    &mut inserted,
                    1,
                    u32::try_from(bit).unwrap_or(0)
                ));
                assert_eq!(inserted, value);
                let mut shifted = vec![0; width];
                assert!(shl_into(
                    &[1],
                    u32::try_from(bit).unwrap_or(0),
                    &mut shifted
                ));
                assert_eq!(shifted, value);
                assert_eq!(
                    cmp_shifted(&[1], u32::try_from(bit).unwrap_or(0), &value),
                    Ordering::Equal
                );
            }
            value.fill(u64::MAX);
            assert!(!add_small_assign(&mut value, 1));
            assert!(is_zero(&value));
            value[0] = 1;
            assert!(!shl_assign(
                &mut value,
                u32::try_from(width * 64).unwrap_or(0)
            ));
            clear(&mut value);
            assert!(shl_assign(&mut value, u32::MAX));
            assert!(add_shifted_u64(&mut value, 0, u32::MAX));
            shr_assign(&mut value, u32::MAX);
            assert!(is_zero(&value));
        }
    }

    /// FR-CAD-32/40: cross-width copies and huge shift classification never truncate.
    #[test]
    fn narrowing_shift_and_scalar_capacity_failures() {
        let mut a = [0; 5];
        let mut b = [0; 8];
        assert!(set_u128(&mut a, u128::MAX));
        assert!(copy_into(&a, &mut b));
        assert_eq!(to_u128(&b), Some(u128::MAX));
        b[7] = 1;
        assert!(!copy_into(&b, &mut a));
        assert!(!shl_into(&[1], u32::MAX, &mut a));
        assert_eq!(cmp_shifted(&[1], u32::MAX, &b), Ordering::Greater);
        assert_eq!(cmp_shifted(&[], u32::MAX, &b), Ordering::Less);
        assert!(set_u128(&mut [], 0));
        assert!(!set_u128(&mut [], 1));
        assert!(!set_u128(&mut [0], u128::MAX));
        assert!(!mul_into(&[u64::MAX], &[u64::MAX], &mut [0]));
        assert!(mul_into(&[0; 5], &[u64::MAX; 5], &mut []));
        let mut maximum = [u64::MAX; 5];
        assert!(!mul_small_assign(&mut maximum, u64::MAX));
        let mut zero = [u64::MAX; 5];
        assert!(mul_small_assign(&mut zero, 0));
        assert!(is_zero(&zero));
        let mut power = [0; 8];
        assert!(pow10_into(38, &mut power));
        assert_eq!(to_u128(&power), Some(10_u128.pow(38)));
        assert!(!pow10_into(u16::MAX, &mut power));
    }

    /// FR-CAD-51/65: shifted wide products and quotient identities at full sizes.
    #[test]
    fn wide_product_division_recovery_and_dense_carries() {
        for width in [5, 8, 20, 40, 112, 243] {
            let mut a = vec![0; width];
            let mut b = vec![0; width];
            let mut product = vec![0; width];
            let mut quotient = vec![0; width];
            let mut remainder = vec![0; width];
            let mut aligned = vec![0; width];
            for shift in [0, 1, 63, 64, 65, width * 32 - 64] {
                assert!(set_u128(&mut a, u128::from(u64::MAX)));
                assert!(shl_assign(&mut a, u32::try_from(shift).unwrap_or(0)));
                assert!(set_u128(&mut b, u128::from(u64::MAX)));
                assert!(mul_into(&a, &b, &mut product));
                assert!(div_rem_into(
                    &product,
                    &a,
                    &mut quotient,
                    &mut remainder,
                    &mut aligned
                ));
                assert_eq!(cmp(&quotient, &b), Ordering::Equal);
                assert!(is_zero(&remainder));
                assert!(add_small_assign(&mut product, 17));
                assert!(div_rem_into(
                    &product,
                    &a,
                    &mut quotient,
                    &mut remainder,
                    &mut aligned
                ));
                assert_eq!(cmp(&quotient, &b), Ordering::Equal);
                assert_eq!(to_u128(&remainder), Some(17));
            }
        }
    }

    /// FR-CAD-40/47/51: direct IEEE midpoint parity, carry and extended-round trap.
    #[test]
    fn exact_ieee_subnormal_overflow_and_extended_cells() {
        let mut u = [0; 5];
        let mut v = [0; 5];
        let mut w = [0; 5];
        for (n, d, e, want) in [
            (0, 1, i32::MAX, 0),
            (1, 1, -1076, 0),
            (1, 1, -1075, 0),
            (3, 1, -1076, 1),
            (3, 1, -1075, 2),
            ((1_u128 << 53) - 1, 1, -1075, 0x0010_0000_0000_0000),
            ((1_u128 << 54) - 3, 1, 970, 0x7fef_ffff_ffff_fffe),
            ((1_u128 << 54) - 2, 1, 970, 0x7fef_ffff_ffff_ffff),
            ((1_u128 << 54) - 1, 1, 970, INFINITY),
            (1, 1, i32::MAX, INFINITY),
            (1, 1, i32::MIN, 0),
        ] {
            assert_eq!(
                round_to_f64(&native(n), &native(d), e, &mut u, &mut v, &mut w),
                Some(want)
            );
        }
        // Just ABOVE half-MIN_SUB: direct IEEE givesMIN_SUB; first rounding
        // to extended53 erases its tiny excess and would then round to zero.
        let n = (1_u128 << 100) + 1;
        let d = 1_u128 << 101;
        assert_eq!(
            round_to_f64(&native(n), &native(d), -1074, &mut u, &mut v, &mut w),
            Some(1)
        );
        assert_eq!(
            round_to_extended53(&native(n), &native(d), -1074, &mut u, &mut v, &mut w),
            Some((1_u64 << 52, -1127))
        );
        // Whole-term normal carry at 2, and ties choosing even low53 bits.
        assert_eq!(
            round_to_extended53(
                &native((1_u128 << 54) - 1),
                &native(1),
                -53,
                &mut u,
                &mut v,
                &mut w
            ),
            Some((1_u64 << 52, -51))
        );
        assert_eq!(round_to_f64(&[0], &[0], 0, &mut u, &mut v, &mut w), None);
        assert_eq!(
            round_to_extended53(&[1], &[1], i32::MIN, &mut u, &mut v, &mut w),
            None
        );
    }

    /// FR-CAD-47/51: exact ratio powers, signed-exponent separation and243 getters.
    #[test]
    fn relative_normalization_and_retained_direct_getter() {
        let mut n = [0; 112];
        let mut d = [0; 112];
        let mut u = [0; 112];
        let mut v = [0; 112];
        let mut w = [0; 112];
        assert!(set_u128(&mut n, 1));
        assert!(shl_assign(&mut n, 7167));
        assert!(add_small_assign(&mut n, 1));
        assert!(set_u128(&mut d, 2));
        assert_eq!(
            round_to_extended53(&n, &d, 0, &mut u, &mut v, &mut w),
            Some((1_u64 << 52, 7114))
        );
        // A retained value2^-1074 directly divided by2 gives an exact IEEE tie.
        let mut retained = [0; 243];
        assert!(add_shifted_u64(&mut retained, 1, 7118));
        let mut denominator = [0; 243];
        assert!(set_u128(&mut denominator, 2));
        let mut a = [0; 243];
        let mut b = [0; 243];
        let mut c = [0; 243];
        assert_eq!(
            round_to_f64(&retained, &denominator, -8192, &mut a, &mut b, &mut c),
            Some(0)
        );
        assert!(add_small_assign(&mut retained, 1));
        assert_eq!(
            round_to_f64(&retained, &denominator, -8192, &mut a, &mut b, &mut c),
            Some(1)
        );
    }

    /// FR-CAD-51/65: zero-divisor, insufficient scratch and scalar-quotient overflow.
    #[test]
    fn explicit_capacity_and_scalar_quotient_boundaries() {
        let mut q = [0; 5];
        let mut r = [0; 5];
        let mut aligned = [0; 5];
        assert!(!div_rem_into(&[1], &[0], &mut q, &mut r, &mut aligned));
        assert!(!div_rem_into(&[0, 1], &[1], &mut [], &mut r, &mut aligned));
        assert_eq!(
            div_rem_u64(&native((1_u128 << 65) - 1), &[3], &mut r, &mut aligned),
            Some(u64::MAX / 3 * 2)
        );
        assert_eq!(to_u128(&r), Some(1));
        assert_eq!(div_rem_u64(&[0, 1], &[1], &mut r, &mut aligned), None);
        let mut one = [0];
        let mut two = [0];
        let mut three = [0];
        assert_eq!(
            round_to_f64(&native(1), &native(3), 0, &mut one, &mut two, &mut three),
            Some((1.0_f64 / 3.0).to_bits())
        );
        assert_eq!(
            round_to_extended53(
                &native(1),
                &native(1_u128 << 100),
                0,
                &mut one,
                &mut two,
                &mut three
            ),
            None
        );
    }

    /// FR-CAD-47/51: unrelated maximal ratios use virtual alignment, not wider
    /// scratch or truncated input. Selected-term bounds remain separate.
    #[test]
    fn relative_alignment_streams_at_the_full_input_capacity() {
        let mut numerator = [0; 112];
        assert!(set_u128(&mut numerator, 1));
        let denominator = [u64::MAX; 112];
        {
            let mut u = [0; 112];
            let mut v = [0; 112];
            let mut aligned = [0; 112];
            assert_eq!(
                round_to_extended53(
                    &numerator,
                    &denominator,
                    -8192,
                    &mut u,
                    &mut v,
                    &mut aligned
                ),
                Some((1_u64 << 52, -15412))
            );
            assert_eq!(
                round_to_f64(&numerator, &denominator, 7168, &mut u, &mut v, &mut aligned),
                Some(1.0_f64.to_bits())
            );
            assert_eq!(
                round_to_extended53(&denominator, &[1], 2048, &mut u, &mut v, &mut aligned),
                Some((1_u64 << 52, 9164))
            );
        }
        // 1/(2^7168-1) is above2^-7168 by less than half of its53-bit
        // significand cell. Its exact observation is unchanged when the same
        // primitive is given a larger caller frame.
        let mut u = [0; 243];
        let mut v = [0; 243];
        let mut aligned = [0; 243];
        assert_eq!(
            round_to_extended53(
                &numerator,
                &denominator,
                -8192,
                &mut u,
                &mut v,
                &mut aligned
            ),
            Some((1_u64 << 52, -15412))
        );
        assert_eq!(
            round_to_f64(&numerator, &denominator, 7168, &mut u, &mut v, &mut aligned),
            Some(1.0_f64.to_bits())
        );
    }

    /// FR-CAD-40/47/51: virtual-carry streaming preserves both tie parities,
    /// their strict neighbors and the53-bit carry, even at243-word capacity.
    #[test]
    fn streaming_midpoint_parity_neighbors_and_significand_carry() {
        for width in [1, 5, 112, 243] {
            let total_bits = u32::try_from(width * 64).unwrap_or(0);
            let mut n = vec![0; width];
            let mut d = vec![0; width];
            let mut u = vec![0; width];
            let mut v = vec![0; width];
            let mut aligned = vec![0; width];
            assert!(set_u128(&mut d, 1));
            assert!(shl_assign(&mut d, total_bits - 1));
            for base in [(1_u64 << 53) + 1, (1_u64 << 53) + 3, (1_u64 << 54) - 1] {
                for neighbor in [-1_i8, 0, 1] {
                    assert!(set_u128(&mut n, u128::from(base)));
                    assert!(shl_assign(&mut n, total_bits - 55));
                    if neighbor < 0 {
                        assert!(sub_assign(&mut n, &[1]));
                    } else if neighbor > 0 {
                        assert!(add_small_assign(&mut n, 1));
                    }
                    let significand = match base {
                        value if value == (1_u64 << 53) + 1 => {
                            (1_u64 << 52) + u64::from(neighbor > 0)
                        }
                        value if value == (1_u64 << 53) + 3 => {
                            (1_u64 << 52) + if neighbor < 0 { 1 } else { 2 }
                        }
                        _ => (1_u64 << 53) - u64::from(neighbor < 0),
                    };
                    let expected = if significand == 1_u64 << 53 {
                        (1_u64 << 52, -52)
                    } else {
                        (significand, -53)
                    };
                    assert_eq!(
                        round_to_extended53(&n, &d, 0, &mut u, &mut v, &mut aligned),
                        Some(expected),
                        "width{width} midpoint{base} neighbor{neighbor}"
                    );
                    let expected_bits = if expected.1 == -52 {
                        1.0_f64.to_bits()
                    } else {
                        0x3fe0_0000_0000_0000 | (expected.0 & FRACTION)
                    };
                    assert_eq!(
                        round_to_f64(&n, &d, 0, &mut u, &mut v, &mut aligned),
                        Some(expected_bits),
                        "width{width} midpoint{base} neighbor{neighbor}"
                    );
                }
            }
        }
    }
}
