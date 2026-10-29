//! Bounded original unsigned arithmetic for exact decimal amount operations.
//!
//! Values use a fixed count `N` of little-endian 64-bit limbs: eight (512
//! bits) for phase A scalar operations and nineteen (1216 bits) for the
//! phase B rounded binary64 conversion; settlement uses forty (2560 bits). Checked operations retain all carries
//! and report a result outside `64*N` bits; no low-limb truncation is exposed
//! as a successful result. Multiplication accumulates one bounded row at a
//! time. Division aligns the divisor, then shifts it right, so a doubled
//! remainder is never formed. Every loop has a fixed bit/limb bound.

use std::cmp::Ordering;

/// An unsigned integer in `0..2^(64*N)`, stored in little-endian limbs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct Wide<const N: usize>([u64; N]);

/// The phase A eight-limb (512-bit) magnitude.
pub(crate) type Uint = Wide<8>;

impl<const N: usize> Wide<N> {
    /// Borrows the fixed little-endian words for charged internal source staging.
    pub(crate) const fn limbs(&self) -> &[u64; N] {
        &self.0
    }

    /// The bit capacity; evaluating it also rejects widths below three limbs.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the limb count is a small compile-time constant"
    )]
    const BITS: u32 = {
        assert!(N >= 3, "at least three limbs are required");
        64 * N as u32
    };

    /// The additive identity, with all limbs zero.
    pub(crate) const ZERO: Self = Self([0; N]);

    /// Embeds an unsigned 128-bit integer without rounding.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the low cast extracts exactly 64 bits; the shifted high half fits u64"
    )]
    pub(crate) const fn from_u128(value: u128) -> Self {
        // Evaluating BITS proves at compile time that limbs 0 and 1 exist.
        let _ = Self::BITS;
        let mut words = [0; N];
        words[0] = value as u64;
        words[1] = (value >> 64) as u64;
        Self(words)
    }

    /// Copies the value into another width, or None if a high limb is lost.
    pub(crate) fn resize<const M: usize>(self) -> Option<Wide<M>> {
        let _ = Wide::<M>::BITS;
        if self.0.iter().skip(M).any(|word| *word != 0) {
            return None;
        }
        let mut words = [0; M];
        for (target, word) in words.iter_mut().zip(self.0) {
            *target = word;
        }
        Some(Wide(words))
    }

    /// Reports whether every limb is zero.
    pub(crate) fn is_zero(self) -> bool {
        self.0.iter().all(|word| *word == 0)
    }

    /// Reports the parity of the exact integer.
    pub(crate) const fn is_odd(self) -> bool {
        self.0[0] & 1 != 0
    }

    /// Returns the significant bit count, or zero for zero.
    pub(crate) fn bits(self) -> u32 {
        let mut width = Self::BITS;
        for word in self.0.iter().rev() {
            if *word != 0 {
                return width - word.leading_zeros();
            }
            width -= 64;
        }
        0
    }

    /// Adds exactly, returning None if a carry escapes the high limb.
    pub(crate) fn checked_add(self, other: Self) -> Option<Self> {
        let mut words = [0_u64; N];
        let mut carry = false;
        for ((target, first), second) in words.iter_mut().zip(self.0).zip(other.0) {
            let (sum, first_carry) = first.overflowing_add(second);
            let (sum, second_carry) = sum.overflowing_add(u64::from(carry));
            *target = sum;
            carry = first_carry || second_carry;
        }
        if carry { None } else { Some(Self(words)) }
    }

    /// Subtracts exactly, returning None for a negative difference.
    pub(crate) fn checked_sub(self, other: Self) -> Option<Self> {
        let mut words = [0_u64; N];
        let mut borrow = false;
        for ((target, first), second) in words.iter_mut().zip(self.0).zip(other.0) {
            let (difference, first_borrow) = first.overflowing_sub(second);
            let (difference, second_borrow) = difference.overflowing_sub(u64::from(borrow));
            *target = difference;
            borrow = first_borrow || second_borrow;
        }
        if borrow { None } else { Some(Self(words)) }
    }

    /// Multiplies exactly, rejecting every product contribution beyond the width.
    ///
    /// A row intermediate is at most `(2^64-1)^2 + 2*(2^64-1)`, exactly
    /// `u128::MAX`. Skipped high products and the final row carry cannot cancel
    /// in unsigned arithmetic; either proves that the result does not fit.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "each low cast retains one limb; every row carry is kept separately"
    )]
    pub(crate) fn checked_mul(self, other: Self) -> Option<Self> {
        let mut words = [0_u64; N];
        for (index, first) in self.0.iter().copied().enumerate() {
            if first == 0 {
                continue;
            }
            let remaining = words.len() - index;
            if other.0[remaining..].iter().any(|word| *word != 0) {
                return None;
            }
            let mut carry = 0_u128;
            for (offset, second) in other.0[..remaining].iter().copied().enumerate() {
                let position = index + offset;
                let product =
                    u128::from(first) * u128::from(second) + u128::from(words[position]) + carry;
                words[position] = product as u64;
                carry = product >> 64;
            }
            if carry != 0 {
                return None;
            }
        }
        Some(Self(words))
    }

    /// Multiplies by an unsigned 32-bit factor, preserving its final carry.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "low casts extract limbs; a nonzero carry is explicitly rejected"
    )]
    pub(crate) fn checked_mul_small(self, factor: u32) -> Option<Self> {
        let mut words = [0_u64; N];
        let mut carry = 0_u128;
        for (target, word) in words.iter_mut().zip(self.0) {
            // The factor has at most 32 bits and the incoming carry at most 32.
            let product = u128::from(word) * u128::from(factor) + carry;
            *target = product as u64;
            carry = product >> 64;
        }
        if carry == 0 { Some(Self(words)) } else { None }
    }

    /// Shifts left exactly, returning None if any nonzero high bit is lost.
    ///
    /// Zero remains zero for every shift, including shifts of at least the width.
    pub(crate) fn checked_shl(self, shift: u32) -> Option<Self> {
        if self.is_zero() {
            return Some(Self::ZERO);
        }
        if shift > Self::BITS - self.bits() {
            return None;
        }
        // A nonzero admitted shift is below the width, so this index is < N.
        let whole = (shift / 64) as usize;
        let offset = shift % 64;
        let mut words = [0_u64; N];
        for (index, word) in self.0[..N - whole].iter().copied().enumerate() {
            words[index + whole] |= word << offset;
            if offset != 0 && index + whole + 1 < words.len() {
                words[index + whole + 1] |= word >> (64 - offset);
            }
        }
        Some(Self(words))
    }

    /// Returns the exact quotient and remainder, or None for a zero divisor.
    ///
    /// Alignment never exceeds the dividend's bit count. At each step the
    /// remainder is reduced by the aligned divisor at most once, and the next
    /// divisor is shifted right. No unchecked doubled remainder or high carry
    /// is discarded. The loop has at most `64*N` steps for every valid value.
    pub(crate) fn div_rem(self, divisor: Self) -> Option<(Self, Self)> {
        if divisor.is_zero() {
            return None;
        }
        if self < divisor {
            return Some((Self::ZERO, self));
        }
        let distance = self.bits() - divisor.bits();
        let mut aligned = divisor.checked_shl(distance)?;
        let mut remainder = self;
        let mut quotient = Self::ZERO;
        for position in (0..=distance).rev() {
            if remainder >= aligned {
                remainder = remainder.checked_sub(aligned)?;
                // position < 64*N, so the derived index is always in 0..N.
                quotient.0[(position / 64) as usize] |= 1_u64 << (position % 64);
            }
            aligned = aligned.shr_one();
        }
        Some((quotient, remainder))
    }

    /// Divides by a nonzero 32-bit integer, preserving the exact remainder.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "remainder<divisor<=u32::MAX and each quotient limb is below 2^64"
    )]
    pub(crate) fn div_rem_small(self, divisor: u32) -> Option<(Self, u32)> {
        if divisor == 0 {
            return None;
        }
        let mut words = [0_u64; N];
        let mut remainder = 0_u128;
        for (target, word) in words.iter_mut().rev().zip(self.0.iter().rev()) {
            // remainder<divisor, so the quotient of this 96-bit value fits u64.
            let dividend = (remainder << 64) | u128::from(*word);
            *target = (dividend / u128::from(divisor)) as u64;
            remainder = dividend % u128::from(divisor);
        }
        Some((Self(words), remainder as u32))
    }

    /// Returns the exact u128 value only when all higher limbs are zero.
    pub(crate) fn to_u128(self) -> Option<u128> {
        if self.0[2..].iter().any(|word| *word != 0) {
            None
        } else {
            Some(u128::from(self.0[0]) | (u128::from(self.0[1]) << 64))
        }
    }

    /// Returns an exact decimal power if it fits the width.
    ///
    /// The largest eight-limb exponent is 154; overflow stops evaluation
    /// early even for u16::MAX input.
    pub(crate) fn pow10(exponent: u16) -> Option<Self> {
        let mut result = Self::from_u128(1);
        for _ in 0..exponent {
            result = result.checked_mul_small(10)?;
        }
        Some(result)
    }

    /// Shifts right by one bit, discarding only the low remainder bit.
    fn shr_one(mut self) -> Self {
        let mut high_bit = 0;
        for word in self.0.iter_mut().rev() {
            let next_high = *word << 63;
            *word = (*word >> 1) | high_bit;
            high_bit = next_high;
        }
        self
    }
}

impl<const N: usize> Ord for Wide<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }
}

impl<const N: usize> PartialOrd for Wide<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::{Uint, Wide};
    use std::cmp::Ordering;

    /// Returns one valid 512-bit basis vector.
    fn power(position: u32) -> Uint {
        let mut words = [0; 8];
        words[(position / 64) as usize] = 1_u64 << (position % 64);
        Wide(words)
    }

    /// Returns an integer with exactly `count` low one bits, including 0/512.
    fn low_ones(count: u32) -> Uint {
        let mut words = [0; 8];
        for (index, word) in words.iter_mut().enumerate() {
            let lower = u32::try_from(index).expect("eight limbs") * 64;
            *word = if count >= lower + 64 {
                u64::MAX
            } else if count > lower {
                (1_u64 << (count - lower)) - 1
            } else {
                0
            };
        }
        Wide(words)
    }

    /// Advances deterministic local state without external randomness.
    fn next(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    /// Draws a valid bounded value with the requested nonzero significant width.
    fn wide(state: &mut u64, width: u32) -> Uint {
        let mut words = std::array::from_fn(|_| next(state));
        for (index, word) in words.iter_mut().enumerate() {
            let lower = u32::try_from(index).expect("eight limbs") * 64;
            if lower >= width {
                *word = 0;
            } else if width < lower + 64 {
                *word &= (1_u64 << (width - lower)) - 1;
            }
        }
        words[((width - 1) / 64) as usize] |= 1_u64 << ((width - 1) % 64);
        Wide(words)
    }

    /// FR-AMT-01/09: every bit position and little-endian comparison boundary.
    #[test]
    fn basis_bits_parity_conversion_and_numeric_order() {
        let zero = Uint::ZERO;
        assert!(zero.is_zero());
        assert!(!zero.is_odd());
        assert_eq!(zero.bits(), 0);
        assert_eq!(zero.to_u128(), Some(0));
        for position in 0..512 {
            let value = power(position);
            assert_eq!(value.bits(), position + 1);
            assert!(!value.is_zero());
            assert_eq!(value.is_odd(), position == 0);
            assert_eq!(value.to_u128(), 1_u128.checked_shl(position));
            assert_eq!(value.partial_cmp(&zero), Some(Ordering::Greater));
            assert_eq!(value.partial_cmp(&value), Some(Ordering::Equal));
            if position != 0 {
                assert!(power(position - 1) < value);
                assert!(low_ones(position) < value);
            }
        }
        for value in [0, 1, u128::MAX, 1_u128 << 64, u128::from(u64::MAX)] {
            assert_eq!(Uint::from_u128(value).to_u128(), Some(value));
        }
    }

    /// FR-AMT-04/09: every carry/borrow width, including the full capacity.
    #[test]
    fn carries_borrows_and_exact_overflow_boundaries() {
        let one = Uint::from_u128(1);
        let maximum = low_ones(512);
        assert_eq!(maximum.checked_add(one), None);
        assert_eq!(maximum.checked_add(maximum), None);
        assert_eq!(Uint::ZERO.checked_sub(one), None);
        assert_eq!(maximum.checked_sub(maximum), Some(Uint::ZERO));
        assert_eq!(maximum.checked_add(Uint::ZERO), Some(maximum));
        for width in 1..512 {
            let top = power(width);
            let lower = low_ones(width);
            assert_eq!(lower.checked_add(one), Some(top));
            assert_eq!(top.checked_sub(one), Some(lower));
            assert_eq!(top.checked_sub(lower), Some(one));
            assert_eq!(lower.checked_sub(top), None);
            assert_eq!(
                maximum.checked_sub(lower),
                Some(Wide(lower.0.map(|word| !word)))
            );
        }
        let alternating = Wide([0, u64::MAX, 0, u64::MAX, 0, u64::MAX, 0, u64::MAX]);
        let complement = Wide([u64::MAX, 0, u64::MAX, 0, u64::MAX, 0, u64::MAX, 0]);
        assert_eq!(alternating.checked_add(complement), Some(maximum));
        assert_eq!(maximum.checked_sub(alternating), Some(complement));
        assert_eq!(maximum.checked_sub(complement), Some(alternating));
    }

    /// FR-AMT-09: no bit is dropped at any admitted or rejected left shift.
    #[test]
    fn every_basis_shift_and_dense_shift_boundaries() {
        for position in 0..512 {
            for shift in 0..=513 {
                let expected = if position + shift < 512 {
                    Some(power(position + shift))
                } else {
                    None
                };
                assert_eq!(power(position).checked_shl(shift), expected);
            }
        }
        for shift in [0, 1, 63, 64, 65, 511, 512, 513, u32::MAX] {
            assert_eq!(Uint::ZERO.checked_shl(shift), Some(Uint::ZERO));
        }
        for width in 1..=512 {
            for shift in [0, 1, 31, 63, 64, 65, 127, 255, 511, 512, u32::MAX] {
                let expected = if shift <= 512 - width {
                    let mut result = [0; 8];
                    for position in shift..shift + width {
                        result[(position / 64) as usize] |= 1_u64 << (position % 64);
                    }
                    Some(Wide(result))
                } else {
                    None
                };
                assert_eq!(low_ones(width).checked_shl(shift), expected);
            }
        }
    }

    /// FR-AMT-04/07/09: native arithmetic independently checks 128-bit inputs.
    #[test]
    fn deterministic_native_u128_cross_checks() {
        let mut state = 0x5e72_a1d4_9bc6_308f;
        for _ in 0..20_000 {
            let first = (u128::from(next(&mut state)) << 64) | u128::from(next(&mut state));
            let second = (u128::from(next(&mut state)) << 64) | u128::from(next(&mut state));
            let a = Uint::from_u128(first);
            let b = Uint::from_u128(second);
            assert_eq!(a.cmp(&b), first.cmp(&second));
            assert_eq!(
                a.checked_add(b).and_then(Uint::to_u128),
                first.checked_add(second)
            );
            assert_eq!(
                a.checked_sub(b).and_then(Uint::to_u128),
                first.checked_sub(second)
            );
            assert_eq!(
                a.checked_mul(b).and_then(Uint::to_u128),
                first.checked_mul(second)
            );
            assert_eq!(
                a.div_rem(b).map(|(q, r)| (q.to_u128(), r.to_u128())),
                first
                    .checked_div(second)
                    .map(|q| (Some(q), Some(first % second)))
            );
            let small = u32::try_from(next(&mut state) & u64::from(u32::MAX)).expect("masked");
            assert_eq!(
                a.checked_mul_small(small).and_then(Uint::to_u128),
                first.checked_mul(u128::from(small))
            );
            assert_eq!(
                a.div_rem_small(small)
                    .map(|(q, r)| (q.to_u128(), u128::from(r))),
                first
                    .checked_div(u128::from(small))
                    .map(|q| (Some(q), first % u128::from(small)))
            );
        }
    }

    /// FR-AMT-09: shifted exact native products cover every high-limb overflow.
    #[test]
    fn shifted_multiplication_and_dense_carry_products() {
        let mut state = 0x3a2d_9780_5b6c_e41f;
        for first_shift in (0..512).step_by(17) {
            for second_shift in (0..512).step_by(19) {
                let first = next(&mut state);
                let second = next(&mut state);
                let Some(a) = Uint::from_u128(u128::from(first)).checked_shl(first_shift) else {
                    continue;
                };
                let Some(b) = Uint::from_u128(u128::from(second)).checked_shl(second_shift) else {
                    continue;
                };
                let expected = Uint::from_u128(u128::from(first) * u128::from(second))
                    .checked_shl(first_shift + second_shift);
                assert_eq!(a.checked_mul(b), expected);
                assert_eq!(b.checked_mul(a), expected);
            }
        }
        for first_width in 1..=256 {
            for second_width in [1, 2, 63, 64, 65, 127, 128, 255, 256] {
                let a = low_ones(first_width);
                let b = low_ones(second_width);
                // (2^a-1)(2^b-1)=2^(a+b)-2^a-2^b+1, represented without
                // constructing the unrepresentable endpoint 2^512.
                let expected = low_ones(first_width + second_width)
                    .checked_sub(low_ones(first_width))
                    .expect("ordered")
                    .checked_sub(low_ones(second_width))
                    .expect("ordered");
                assert_eq!(a.checked_mul(b), Some(expected));
            }
        }
        let maximum = low_ones(512);
        assert_eq!(maximum.checked_mul(Uint::ZERO), Some(Uint::ZERO));
        assert_eq!(Uint::ZERO.checked_mul(maximum), Some(Uint::ZERO));
        assert_eq!(maximum.checked_mul(Uint::from_u128(1)), Some(maximum));
        assert_eq!(maximum.checked_mul(Uint::from_u128(2)), None);
        assert_eq!(maximum.checked_mul_small(0), Some(Uint::ZERO));
        assert_eq!(maximum.checked_mul_small(1), Some(maximum));
        assert_eq!(maximum.checked_mul_small(2), None);
    }

    /// FR-AMT-07/09: exact full-capacity quotient/remainder identities.
    #[test]
    fn division_basis_maximum_and_zero_divisors() {
        let maximum = low_ones(512);
        assert_eq!(Uint::ZERO.div_rem(Uint::ZERO), None);
        assert_eq!(maximum.div_rem(Uint::ZERO), None);
        assert_eq!(Uint::ZERO.div_rem_small(0), None);
        assert_eq!(maximum.div_rem_small(0), None);
        for shift in 0..512 {
            assert_eq!(
                maximum.div_rem(power(shift)),
                Some((low_ones(512 - shift), low_ones(shift)))
            );
        }
        assert_eq!(
            maximum.div_rem(maximum),
            Some((Uint::from_u128(1), Uint::ZERO))
        );
        assert_eq!(Uint::ZERO.div_rem(maximum), Some((Uint::ZERO, Uint::ZERO)));
        assert_eq!(
            Uint::from_u128(1).div_rem(maximum),
            Some((Uint::ZERO, Uint::from_u128(1)))
        );
        for (divisor, word) in [
            (3, 0x5555_5555_5555_5555),
            (5, 0x3333_3333_3333_3333),
            (15, 0x1111_1111_1111_1111),
            (u32::MAX, 0x0000_0001_0000_0001),
        ] {
            let quotient = Wide([word; 8]);
            assert_eq!(maximum.div_rem_small(divisor), Some((quotient, 0)));
            assert_eq!(
                maximum.div_rem(Uint::from_u128(u128::from(divisor))),
                Some((quotient, Uint::ZERO))
            );
        }
        let dividend = power(511)
            .checked_add(power(510))
            .expect("fits")
            .checked_add(Uint::from_u128(1))
            .expect("fits");
        let divisor = power(510).checked_add(Uint::from_u128(1)).expect("fits");
        assert_eq!(
            dividend.div_rem(divisor),
            Some((Uint::from_u128(2), low_ones(510)))
        );
    }

    /// FR-AMT-07/09: every divisor width, dense operands and independent small division.
    #[test]
    fn deterministic_full_width_division_and_arithmetic_identities() {
        let mut state = 0x7d95_23c8_a164_bef0;
        for width in 1..=512 {
            for _ in 0..8 {
                let dividend = wide(&mut state, 512);
                let divisor = wide(&mut state, width);
                let (quotient, remainder) = dividend.div_rem(divisor).expect("positive divisor");
                assert!(remainder < divisor);
                assert_eq!(
                    quotient
                        .checked_mul(divisor)
                        .expect("product<=dividend")
                        .checked_add(remainder),
                    Some(dividend)
                );
                assert_eq!(
                    dividend.checked_sub(remainder),
                    quotient.checked_mul(divisor)
                );
                let small = u32::try_from(next(&mut state) & u64::from(u32::MAX)).expect("masked");
                let small = small.max(1);
                let small_result = dividend.div_rem_small(small).expect("positive divisor");
                assert_eq!(
                    dividend.div_rem(Uint::from_u128(u128::from(small))),
                    Some((small_result.0, Uint::from_u128(u128::from(small_result.1))))
                );
                assert_eq!(
                    small_result
                        .0
                        .checked_mul_small(small)
                        .expect("product<=dividend")
                        .checked_add(Uint::from_u128(u128::from(small_result.1))),
                    Some(dividend)
                );
                let other = wide(&mut state, width);
                if let Some(sum) = dividend.checked_add(other) {
                    assert_eq!(sum.checked_sub(other), Some(dividend));
                    assert_eq!(sum.checked_sub(dividend), Some(other));
                } else {
                    assert!(dividend > low_ones(512).checked_sub(other).expect("ordered"));
                }
                if let Some(difference) = dividend.checked_sub(other) {
                    assert_eq!(difference.checked_add(other), Some(dividend));
                } else {
                    assert!(dividend < other);
                }
            }
        }
    }

    /// FR-AMT-09: all representable decimal powers and first rejected exponent.
    #[test]
    fn decimal_powers_and_small_divisor_carry_edges() {
        let mut previous = Uint::from_u128(1);
        for exponent in 0..=154 {
            let actual = Uint::pow10(exponent).expect("10^154 fits 512 bits");
            if exponent <= 38 {
                assert_eq!(actual.to_u128(), Some(10_u128.pow(u32::from(exponent))));
            }
            if exponent != 0 {
                assert_eq!(actual.div_rem_small(10), Some((previous, 0)));
            }
            previous = actual;
        }
        assert_eq!(Uint::pow10(155), None);
        assert_eq!(Uint::pow10(u16::MAX), None);
        assert_eq!(Uint::pow10(154).expect("largest decimal power").bits(), 512);
        for divisor in [1, 2, 3, 10, 1_u32 << 31, u32::MAX - 1, u32::MAX] {
            for width in [1, 63, 64, 65, 127, 128, 255, 511, 512] {
                let dividend = low_ones(width);
                let (q, r) = dividend.div_rem_small(divisor).expect("positive divisor");
                assert!(r < divisor);
                assert_eq!(
                    q.checked_mul_small(divisor)
                        .expect("product<=dividend")
                        .checked_add(Uint::from_u128(u128::from(r))),
                    Some(dividend)
                );
            }
        }
    }
}
