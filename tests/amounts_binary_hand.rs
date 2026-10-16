//! Independent public hand/adversarial cases for FR-BIN-01 through FR-BIN-08.
//!
//! Expectations come from the frozen M8 phase B binary64 boundary contract,
//! IEEE 754 bit fields assembled with `f64::from_bits`, elementary integer
//! identities and the small exact integer oracles below. Literal bit patterns
//! and sweep census counts were derived separately with exact integer and
//! `Fraction` arithmetic, never from production output or float formatting.
//! This suite was authored without reading the binary boundary production code
//! or its private wide-integer kernel. TOL-E0 applies: every binary64 is
//! compared by its bits.

use sabiquant::amounts::{AmountError, AmountQuantum, AmountRounding as Rounding, DecimalAmount};
use sabiquant::error::{Error as CrateError, NonFinite};
use std::cmp::Ordering::{self, Equal, Greater, Less};
use std::error::Error;
use std::fmt::Debug;
use std::marker::PhantomData;

const SIGN: u64 = 1 << 63;
const HIDDEN: u64 = 1 << 52;
const EXACT_IN: &str = "from_binary64_exact";
const ROUNDED_IN: &str = "from_binary64_rounded";
const EXACT_OUT: &str = "to_binary64_exact";

/// NaN sign and payload classes: quiet NaNs have fraction bit 51 set.
const NAN_PATTERNS: [u64; 12] = [
    0x7FF8_0000_0000_0000, // canonical quiet NaN
    0xFFF8_0000_0000_0000, // quiet NaN with the sign bit
    0x7FF8_0000_0000_0001, // quiet NaN, lowest payload bit
    0x7FFC_0000_0000_0000, // quiet NaN, highest payload bit
    0x7FFF_FFFF_FFFF_FFFF, // quiet NaN, every payload bit
    0xFFFF_FFFF_FFFF_FFFF, // negative quiet NaN, every payload bit
    0x7FF0_0000_0000_0001, // signalling NaN, lowest payload bit
    0xFFF0_0000_0000_0001, // negative signalling NaN
    0x7FF4_0000_0000_0000, // signalling NaN, highest payload bit
    0x7FF7_FFFF_FFFF_FFFF, // signalling NaN, every payload bit
    0xFFF7_FFFF_FFFF_FFFF, // negative signalling NaN, every payload bit
    0xFFF0_0000_DEAD_BEEF, // negative signalling NaN, arbitrary payload
];

/// Nearest-even binary64 of `10^-scale`, from exact integer rounding.
const UNIT_BITS: [u64; 39] = [
    0x3FF0_0000_0000_0000, // 1e-0
    0x3FB9_9999_9999_999A, // 1e-1
    0x3F84_7AE1_47AE_147B, // 1e-2
    0x3F50_624D_D2F1_A9FC, // 1e-3
    0x3F1A_36E2_EB1C_432D, // 1e-4
    0x3EE4_F8B5_88E3_68F1, // 1e-5
    0x3EB0_C6F7_A0B5_ED8D, // 1e-6
    0x3E7A_D7F2_9ABC_AF48, // 1e-7
    0x3E45_798E_E230_8C3A, // 1e-8
    0x3E11_2E0B_E826_D695, // 1e-9
    0x3DDB_7CDF_D9D7_BDBB, // 1e-10
    0x3DA5_FD7F_E179_6495, // 1e-11
    0x3D71_9799_812D_EA11, // 1e-12
    0x3D3C_25C2_6849_7682, // 1e-13
    0x3D06_849B_86A1_2B9B, // 1e-14
    0x3CD2_03AF_9EE7_5616, // 1e-15
    0x3C9C_D2B2_97D8_89BC, // 1e-16
    0x3C67_0EF5_4646_D497, // 1e-17
    0x3C32_725D_D1D2_43AC, // 1e-18
    0x3BFD_83C9_4FB6_D2AC, // 1e-19
    0x3BC7_9CA1_0C92_4223, // 1e-20
    0x3B92_E3B4_0A0E_9B4F, // 1e-21
    0x3B5E_3920_1017_5EE6, // 1e-22
    0x3B28_2DB3_4012_B251, // 1e-23
    0x3AF3_57C2_99A8_8EA7, // 1e-24
    0x3ABE_F2D0_F5DA_7DD9, // 1e-25
    0x3A88_C240_C4AE_CB14, // 1e-26
    0x3A53_CE9A_36F2_3C10, // 1e-27
    0x3A1F_B0F6_BE50_6019, // 1e-28
    0x39E9_5A5E_FEA6_B347, // 1e-29
    0x39B4_484B_FEEB_C2A0, // 1e-30
    0x3980_39D6_6589_6880, // 1e-31
    0x3949_F623_D5A8_A733, // 1e-32
    0x3914_C4E9_77BA_1F5C, // 1e-33
    0x38E0_9D87_92FB_4C49, // 1e-34
    0x38AA_95A5_B7F8_7A0F, // 1e-35
    0x3875_4484_932D_2E72, // 1e-36
    0x3841_039D_428A_8B8F, // 1e-37
    0x380B_38FB_9DAA_78E4, // 1e-38
];

#[expect(
    clippy::expect_used,
    reason = "hand inputs explicitly require construction success"
)]
fn a(coefficient: i128, scale: u8) -> DecimalAmount {
    DecimalAmount::new(coefficient, scale).expect("valid hand amount")
}

#[expect(
    clippy::expect_used,
    reason = "hand inputs explicitly require construction success"
)]
fn q(coefficient: i128, scale: u8) -> AmountQuantum {
    AmountQuantum::new(coefficient, scale).expect("positive hand quantum")
}

#[expect(
    clippy::expect_used,
    reason = "the hand spelling is an exact valid decimal"
)]
fn p(text: &str) -> DecimalAmount {
    text.parse().expect("valid hand decimal")
}

#[expect(
    clippy::expect_used,
    reason = "the stated conversion result is representable"
)]
fn good<T>(result: Result<T, AmountError>) -> T {
    result.expect("representable hand result")
}

#[expect(
    clippy::expect_used,
    reason = "the case explicitly requires this public rejection"
)]
fn rejected<T: Debug>(result: Result<T, AmountError>) -> AmountError {
    result.expect_err("the hand precondition requires an error")
}

fn exact_in(x: f64) -> Result<DecimalAmount, AmountError> {
    DecimalAmount::from_binary64_exact(x)
}

fn rounded_in(
    x: f64,
    quantum: AmountQuantum,
    mode: Rounding,
) -> Result<DecimalAmount, AmountError> {
    DecimalAmount::from_binary64_rounded(x, quantum, mode)
}

/// A spread of cent, nickel, odd, finest and extreme quanta.
fn quanta() -> [AmountQuantum; 12] {
    [
        q(1, 0),
        q(1, 1),
        q(1, 2),
        q(5, 2),
        q(25, 2),
        q(3, 0),
        q(7, 1),
        q(37, 3),
        q(1, 38),
        q(3, 38),
        q(i128::MAX, 0),
        q(i128::MAX, 38),
    ]
}

/// The exact binary64 `(-1)^negative · m · 2^exponent`, from IEEE 754 fields.
fn dyadic(negative: bool, mut m: u64, mut exponent: i32) -> f64 {
    let sign = if negative { SIGN } else { 0 };
    if m == 0 {
        return f64::from_bits(sign);
    }
    while m < HIDDEN {
        m <<= 1;
        exponent -= 1;
    }
    while m >= 2 * HIDDEN {
        assert!(m & 1 == 0, "more than 53 significant bits");
        m >>= 1;
        exponent += 1;
    }
    let field = exponent + 1075;
    assert!(field <= 2046, "beyond the binary64 range");
    if field >= 1 {
        f64::from_bits(sign | u64::from(field.unsigned_abs()) << 52 | (m - HIDDEN))
    } else {
        let shift = (1 - field).unsigned_abs();
        assert!(
            shift <= 52 && m.trailing_zeros() >= shift,
            "not a subnormal"
        );
        f64::from_bits(sign | m >> shift)
    }
}

/// Sign, odd significand (zero for zeros) and exponent of a finite binary64.
fn parts(x: f64) -> (bool, u64, i32) {
    let bits = x.to_bits();
    let field = (bits >> 52) & 0x7FF;
    assert!(field != 0x7FF, "finite input required");
    let fraction = bits & (HIDDEN - 1);
    let (m, exponent) = if field == 0 {
        (fraction, -1074)
    } else {
        let biased = i32::from(u16::try_from(field).unwrap_or(u16::MAX));
        (fraction | HIDDEN, biased - 1075)
    };
    if m == 0 {
        return (bits & SIGN != 0, 0, 0);
    }
    let zeros = m.trailing_zeros();
    (bits & SIGN != 0, m >> zeros, exponent + zeros.cast_signed())
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "keeps exactly the low 64-bit limb"
)]
const fn low(value: u128) -> u64 {
    value as u64
}

fn width(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

/// A trimmed little-endian unsigned integer of 64-bit limbs: just enough
/// exact arithmetic for the independent oracles below.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Big(Vec<u64>);

impl Big {
    fn new(value: u128) -> Self {
        Self(vec![low(value), low(value >> 64)]).trimmed()
    }

    fn trimmed(mut self) -> Self {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
        self
    }

    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }

    fn limb(&self, index: usize) -> u64 {
        self.0.get(index).copied().unwrap_or(0)
    }

    fn bit(&self, index: usize) -> bool {
        (self.limb(index / 64) >> (index % 64)) & 1 == 1
    }

    fn bits(&self) -> usize {
        self.0
            .last()
            .map_or(0, |top| 64 * self.0.len() - width(top.leading_zeros()))
    }

    fn to_u128(&self) -> Option<u128> {
        (self.0.len() <= 2).then(|| (u128::from(self.limb(1)) << 64) | u128::from(self.limb(0)))
    }

    fn shifted_left(&self, bits: usize) -> Self {
        let offset = bits % 64;
        let mut limbs = vec![0; bits / 64];
        let mut carry = 0;
        for &limb in &self.0 {
            limbs.push((limb << offset) | carry);
            carry = if offset == 0 {
                0
            } else {
                limb >> (64 - offset)
            };
        }
        limbs.push(carry);
        Self(limbs).trimmed()
    }

    fn shifted_right(&self, bits: usize) -> Self {
        let offset = bits % 64;
        let rest = self.0.get(bits / 64..).unwrap_or_default();
        let limbs = rest
            .iter()
            .enumerate()
            .map(|(index, &limb)| {
                let high = rest.get(index + 1).copied().unwrap_or(0);
                if offset == 0 {
                    limb
                } else {
                    (limb >> offset) | (high << (64 - offset))
                }
            })
            .collect();
        Self(limbs).trimmed()
    }

    fn plus(&self, other: &Self) -> Self {
        let mut limbs = Vec::new();
        let mut carry = 0_u128;
        for index in 0..self.0.len().max(other.0.len()) {
            let sum = u128::from(self.limb(index)) + u128::from(other.limb(index)) + carry;
            limbs.push(low(sum));
            carry = sum >> 64;
        }
        limbs.push(low(carry));
        Self(limbs).trimmed()
    }

    fn minus(&self, other: &Self) -> Self {
        assert!(
            self >= other,
            "the oracle subtracts only a smaller magnitude"
        );
        let mut limbs = Vec::new();
        let mut borrow = false;
        for (index, &limb) in self.0.iter().enumerate() {
            let (partial, first) = limb.overflowing_sub(other.limb(index));
            let (difference, second) = partial.overflowing_sub(u64::from(borrow));
            limbs.push(difference);
            borrow = first || second;
        }
        assert!(!borrow, "no final borrow");
        Self(limbs).trimmed()
    }

    fn times_small(&self, factor: u64) -> Self {
        let mut limbs = Vec::new();
        let mut carry = 0_u128;
        for &limb in &self.0 {
            let product = u128::from(limb) * u128::from(factor) + carry;
            limbs.push(low(product));
            carry = product >> 64;
        }
        limbs.push(low(carry));
        Self(limbs).trimmed()
    }

    fn times(&self, factor: u128) -> Self {
        self.times_small(low(factor))
            .plus(&self.times_small(low(factor >> 64)).shifted_left(64))
    }

    /// Exact quotient and remainder by a divisor in `1..2^127`, bit by bit.
    fn div_rem(&self, divisor: u128) -> (Self, u128) {
        assert!(divisor != 0 && divisor >> 127 == 0, "divisor below 2^127");
        let mut quotient = vec![0_u64; self.0.len()];
        let mut remainder = 0_u128;
        for index in (0..self.bits()).rev() {
            remainder = (remainder << 1) | u128::from(self.bit(index));
            if remainder >= divisor {
                remainder -= divisor;
                quotient[index / 64] |= 1 << (index % 64);
            }
        }
        (Self(quotient).trimmed(), remainder)
    }
}

impl Ord for Big {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.iter().rev().cmp(other.0.iter().rev()))
    }
}

impl PartialOrd for Big {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Exact `numerator / (divisor · 2^shift)`: the quotient, twice the remainder
/// against the denominator (the half test) and whether the remainder is zero.
fn divide(numerator: &Big, divisor: u128, shift: usize) -> (Big, Ordering, bool) {
    // floor(n / (d·2^s)) = floor(floor(n / 2^s) / d) for integers n, d, s.
    let (quotient, _) = numerator.shifted_right(shift).div_rem(divisor);
    let denominator = Big::new(divisor).shifted_left(shift);
    let remainder = numerator.minus(&quotient.times(divisor).shifted_left(shift));
    assert!(remainder < denominator, "quotient/remainder identity");
    let half = remainder.shifted_left(1).cmp(&denominator);
    (quotient, half, remainder.is_zero())
}

/// One independently derived conversion outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expect {
    /// The canonical coefficient and scale.
    Value(i128, u8),
    /// `ResultScale` with the exact needed scale.
    Scale(u16),
    /// `OutOfRange` with the sign of the rejected result.
    Range(bool),
    /// `Inexact`.
    Inexact,
}

fn signed(negative: bool, magnitude: u128) -> Option<i128> {
    if negative {
        0_i128.checked_sub_unsigned(magnitude)
    } else {
        i128::try_from(magnitude).ok()
    }
}

/// Signs, normalizes at `scale` and only then range-checks a magnitude.
fn admitted(negative: bool, mut magnitude: Big, mut scale: u8) -> Expect {
    if magnitude.is_zero() {
        return Expect::Value(0, 0);
    }
    while scale > 0 {
        let (tenth, rest) = magnitude.div_rem(10);
        if rest != 0 {
            break;
        }
        magnitude = tenth;
        scale -= 1;
    }
    match magnitude
        .to_u128()
        .and_then(|value| signed(negative, value))
    {
        Some(coefficient) => Expect::Value(coefficient, scale),
        None => Expect::Range(negative),
    }
}

/// Contract oracle for `from_binary64_exact`: odd `M` gives scale `t = -e`.
fn expected_exact(x: f64) -> Expect {
    let (negative, m, exponent) = parts(x);
    let places = exponent.unsigned_abs();
    if m == 0 {
        Expect::Value(0, 0)
    } else if exponent >= 0 {
        admitted(
            negative,
            Big::new(u128::from(m)).shifted_left(width(places)),
            0,
        )
    } else if places > 38 {
        Expect::Scale(u16::try_from(places).unwrap_or(u16::MAX))
    } else {
        let magnitude = Big::new(u128::from(m)).times(5_u128.pow(places));
        admitted(negative, magnitude, u8::try_from(places).unwrap_or(u8::MAX))
    }
}

/// The phase A quantization table; `None` is `Exact` on a non-multiple.
fn increment(
    mode: Rounding,
    negative: bool,
    half: Ordering,
    odd: bool,
    exact: bool,
) -> Option<bool> {
    if exact {
        Some(false)
    } else if mode == Rounding::Exact {
        None
    } else if mode == Rounding::NearestTiesEven {
        Some(half == Greater || (half == Equal && odd))
    } else if mode == Rounding::NearestTiesAwayFromZero {
        Some(half != Less)
    } else {
        Some(
            mode == Rounding::AwayFromZero
                || (mode == Rounding::TowardPositiveInfinity && !negative)
                || (mode == Rounding::TowardNegativeInfinity && negative),
        )
    }
}

/// Contract oracle for `from_binary64_rounded` in `Rounding::ALL` order: one
/// exact quotient/remainder of `|x|/q`, then sign, `Q`, normalization, range.
fn rounded_expectations(x: f64, quantum: AmountQuantum) -> [Expect; 7] {
    let (negative, m, exponent) = parts(x);
    if m == 0 {
        return [Expect::Value(0, 0); 7];
    }
    let unit = quantum.amount();
    let divisor = unit.coefficient().unsigned_abs();
    let mut numerator = Big::new(u128::from(m));
    for _ in 0..unit.scale() {
        numerator = numerator.times_small(10);
    }
    let places = width(exponent.unsigned_abs());
    let (quotient, half, exact) = if exponent >= 0 {
        divide(&numerator.shifted_left(places), divisor, 0)
    } else {
        divide(&numerator, divisor, places)
    };
    let odd = quotient.bit(0);
    Rounding::ALL.map(|mode| match increment(mode, negative, half, odd, exact) {
        None => Expect::Inexact,
        Some(up) => {
            let count = if up {
                quotient.plus(&Big::new(1))
            } else {
                quotient.clone()
            };
            admitted(negative, count.times(divisor), unit.scale())
        }
    })
}

/// Nearest-even binary64 bits of `coefficient/10^scale` and its exactness,
/// chosen by exact integer comparisons only.
fn expected_nearest(coefficient: i128, scale: u8) -> (u64, bool) {
    if coefficient == 0 {
        return (0, true);
    }
    let sign = if coefficient < 0 { SIGN } else { 0 };
    let numerator = Big::new(coefficient.unsigned_abs());
    let power = 10_u128.pow(u32::from(scale));
    let length = |value: &Big| i32::try_from(value.bits()).unwrap_or(i32::MAX);
    let mut exponent = length(&numerator) - length(&Big::new(power)) - 53;
    let mut attempts = 0;
    loop {
        attempts += 1;
        assert!(attempts <= 8, "the estimate is within a few binades");
        let places = width(exponent.unsigned_abs());
        let (quotient, half, exact) = if exponent >= 0 {
            divide(&numerator, power, places)
        } else {
            divide(&numerator.shifted_left(places), power, 0)
        };
        let significand = quotient.to_u128().unwrap_or(u128::MAX);
        if significand >> 53 != 0 {
            exponent += 1;
        } else if significand >> 52 == 0 {
            exponent -= 1;
        } else {
            let up = half == Greater || (half == Equal && significand & 1 == 1);
            let mut significand = significand + u128::from(up);
            if significand >> 53 != 0 {
                significand >>= 1;
                exponent += 1;
            }
            let field = exponent + 1075;
            assert!((1..=2046).contains(&field), "amounts stay normal");
            let bits = u64::from(field.unsigned_abs()) << 52 | (low(significand) - HIDDEN);
            return (sign | bits, exact);
        }
    }
}

#[expect(clippy::panic, reason = "a mismatched outcome class is a test failure")]
fn check(
    result: Result<DecimalAmount, AmountError>,
    expected: Expect,
    operation: &'static str,
    context: &dyn Debug,
) {
    let kind = match expected {
        Expect::Value(..) => None,
        Expect::Scale(_) => Some("amount.result_scale"),
        Expect::Range(_) => Some("amount.out_of_range"),
        Expect::Inexact => Some("amount.inexact"),
    };
    assert_eq!(
        result.as_ref().err().map(AmountError::kind),
        kind,
        "{context:?}: {result:?}"
    );
    match (result, expected) {
        (Ok(value), Expect::Value(coefficient, scale)) => {
            assert_eq!(
                (value.coefficient(), value.scale()),
                (coefficient, scale),
                "{context:?}"
            );
            assert_eq!(value, a(coefficient, scale), "{context:?}");
        }
        (
            Err(AmountError::ResultScale {
                operation: o,
                scale: s,
                maximum: m,
                ..
            }),
            Expect::Scale(scale),
        ) => assert_eq!((o, s, m), (operation, scale, 38), "{context:?}"),
        (
            Err(AmountError::OutOfRange {
                operation: o,
                negative: n,
                ..
            }),
            Expect::Range(negative),
        ) => assert_eq!((o, n), (operation, negative), "{context:?}"),
        (Err(AmountError::Inexact { operation: o, .. }), Expect::Inexact) => {
            assert_eq!(o, operation, "{context:?}");
        }
        (result, expected) => panic!("{context:?}: expected {expected:?}, got {result:?}"),
    }
}

fn assert_non_finite(error: &AmountError, operation: &'static str, class: NonFinite) {
    assert_eq!(error.kind(), "amount.non_finite");
    assert!(
        matches!(error, AmountError::NonFinite { operation: o, value: v, .. }
        if *o == operation && *v == class),
        "{error:?}"
    );
    assert!(error.source().is_none());
}

fn assert_inexact(error: &AmountError, operation: &'static str) {
    assert_eq!(error.kind(), "amount.inexact");
    assert!(
        matches!(error, AmountError::Inexact { operation: o, .. } if *o == operation),
        "{error:?}"
    );
}

fn value_of(text: &str) -> Expect {
    let value = p(text);
    Expect::Value(value.coefficient(), value.scale())
}

/// Checks every mode in `Rounding::ALL` order, and that the oracle agrees.
fn assert_all_modes(x: f64, quantum: AmountQuantum, expected: [Expect; 7]) {
    assert_eq!(
        rounded_expectations(x, quantum),
        expected,
        "oracle {x:e} / {quantum:?}"
    );
    for (mode, outcome) in Rounding::ALL.into_iter().zip(expected) {
        check(
            rounded_in(x, quantum, mode),
            outcome,
            ROUNDED_IN,
            &(x, quantum, mode),
        );
    }
}

/// Six decimal answers in `Rounding::ALL` order for a non-multiple input.
fn assert_modes(x: f64, quantum: AmountQuantum, answers: [&str; 6]) {
    let mut expected = [Expect::Inexact; 7];
    for (slot, answer) in expected.iter_mut().zip(answers) {
        *slot = value_of(answer);
    }
    assert_all_modes(x, quantum, expected);
}

/// Checks every mode against the oracle and counts zero, value, range and
/// inexact outcomes in the first four census slots.
fn assert_oracle_modes<const N: usize>(
    x: f64,
    quantum: AmountQuantum,
    census: &mut [usize; N],
) -> [Expect; 7] {
    let expected = rounded_expectations(x, quantum);
    for (mode, outcome) in Rounding::ALL.into_iter().zip(expected) {
        check(
            rounded_in(x, quantum, mode),
            outcome,
            ROUNDED_IN,
            &(x, quantum, mode),
        );
        let slot = match outcome {
            Expect::Value(0, _) => 0,
            Expect::Value(..) => 1,
            Expect::Range(_) | Expect::Scale(_) => 2,
            Expect::Inexact => 3,
        };
        census[slot] += 1;
    }
    expected
}

/// Splits a sweep across scoped threads and adds up the per-part censuses.
fn parallel_census<const N: usize>(work: impl Fn(usize, usize) -> [usize; N] + Sync) -> [usize; N] {
    const PARTS: usize = 4;
    std::thread::scope(|scope| {
        let work = &work;
        let workers: Vec<_> = (0..PARTS)
            .map(|part| scope.spawn(move || work(part, PARTS)))
            .collect();
        let mut total = [0; N];
        for worker in workers {
            match worker.join() {
                Ok(census) => {
                    for (sum, count) in total.iter_mut().zip(census) {
                        *sum += count;
                    }
                }
                Err(payload) => std::panic::resume_unwind(payload),
            }
        }
        total
    })
}

/// Checks both outgoing conversions against the integer nearest-even oracle
/// and, when exact, the decimal round trip; returns the exact binary64.
fn assert_outgoing(value: DecimalAmount) -> Option<f64> {
    let (bits, exact) = expected_nearest(value.coefficient(), value.scale());
    let nearest = value.to_binary64_nearest();
    assert_eq!(nearest.to_bits(), bits, "{value}: {nearest:e}");
    if value.coefficient() != 0 {
        assert!(nearest.is_normal(), "{value}");
        assert_eq!(nearest.is_sign_negative(), value.coefficient() < 0);
    }
    match value.to_binary64_exact() {
        Ok(binary) => {
            assert!(exact, "{value} is not a binary64");
            assert_eq!(binary.to_bits(), bits, "{value}");
            assert_eq!(good(exact_in(binary)), value);
            Some(binary)
        }
        Err(error) => {
            assert!(!exact, "{value} is a binary64");
            assert_inexact(&error, EXACT_OUT);
            None
        }
    }
}

/// Asserts the literal nearest binary64 and the oracle/exactness checks.
fn assert_rounds_to(value: DecimalAmount, expected: f64) -> Option<f64> {
    assert_eq!(
        value.to_binary64_nearest().to_bits(),
        expected.to_bits(),
        "{value}"
    );
    assert_outgoing(value)
}

/// FR-BIN-02, FR-BIN-08: every NaN sign/payload class and both infinities
/// fail `NonFinite` with their class before any mode, quantum or range rule.
#[test]
fn nonfinite_inputs_fail_first_with_their_class_under_every_mode_and_quantum() {
    let mut cases = vec![
        (f64::NAN, NonFinite::NaN),
        (-f64::NAN, NonFinite::NaN),
        (f64::INFINITY, NonFinite::PositiveInfinity),
        (f64::NEG_INFINITY, NonFinite::NegativeInfinity),
        (
            f64::from_bits(0x7FF0_0000_0000_0000),
            NonFinite::PositiveInfinity,
        ),
        (
            f64::from_bits(0xFFF0_0000_0000_0000),
            NonFinite::NegativeInfinity,
        ),
    ];
    cases.extend(NAN_PATTERNS.map(|bits| (f64::from_bits(bits), NonFinite::NaN)));
    for (x, class) in cases {
        assert!(!x.is_finite());
        assert_eq!(x.is_nan(), class == NonFinite::NaN, "{:#018X}", x.to_bits());
        assert_non_finite(&rejected(exact_in(x)), EXACT_IN, class);
        for quantum in quanta() {
            for mode in Rounding::ALL {
                assert_non_finite(&rejected(rounded_in(x, quantum, mode)), ROUNDED_IN, class);
            }
        }
    }
    // The class is recorded, not the payload: distinct NaNs give equal errors.
    let canonical = rejected(exact_in(f64::NAN));
    for bits in NAN_PATTERNS {
        assert_eq!(rejected(exact_in(f64::from_bits(bits))), canonical);
    }
    assert_ne!(
        rejected(exact_in(f64::INFINITY)),
        rejected(exact_in(f64::NEG_INFINITY))
    );
    assert_ne!(
        rejected(exact_in(f64::NAN)),
        rejected(rounded_in(f64::NAN, q(1, 0), Rounding::Exact))
    );
}

/// FR-BIN-02, FR-BIN-05, FR-BIN-06: both zeros enter as canonical zero under
/// every mode and quantum, and canonical zero leaves as `+0.0`.
#[test]
fn both_zeros_give_canonical_zero_under_every_mode_and_quantum() {
    for zero in [0.0, -0.0, f64::from_bits(0), f64::from_bits(SIGN)] {
        check(exact_in(zero), Expect::Value(0, 0), EXACT_IN, &zero);
        for quantum in quanta() {
            assert_all_modes(zero, quantum, [Expect::Value(0, 0); 7]);
        }
        let value = good(exact_in(zero));
        assert_eq!(value.to_binary64_nearest().to_bits(), 0);
        assert_eq!(good(value.to_binary64_exact()).to_bits(), 0);
    }
    for scale in 0..=38 {
        let zero = a(0, scale);
        assert_eq!(zero.to_binary64_nearest().to_bits(), 0);
        assert_eq!(good(zero.to_binary64_exact()).to_bits(), 0);
    }
    assert_eq!(p("-0.000").to_binary64_nearest().to_bits(), 0);
}

/// FR-BIN-03: exact incoming reads the bit-field rational `M·2^e`, never the
/// shortest decimal spelling of the float.
#[test]
fn exact_incoming_reads_the_bit_field_rational_not_the_shortest_decimal() {
    // Hand bit decompositions behind the discriminators below.
    assert_eq!(0.1_f64.to_bits(), 0x3FB9_9999_9999_999A);
    assert_eq!(
        0.1_f64.to_bits(),
        dyadic(false, 3_602_879_701_896_397, -55).to_bits()
    );
    assert_eq!(
        0.3_f64.to_bits(),
        dyadic(false, 5_404_319_552_844_595, -54).to_bits()
    );
    assert_eq!(
        2.675_f64.to_bits(),
        dyadic(false, 3_011_782_250_804_019, -50).to_bits()
    );
    assert_eq!(
        1e23_f64.to_bits(),
        dyadic(false, 2_980_232_238_769_531, 25).to_bits()
    );
    assert_eq!(
        1e-38_f64.to_bits(),
        dyadic(false, 1_915_619_426_082_361, -177).to_bits()
    );
    let five = 5_i128.pow(38);
    for (x, expected) in [
        (1.0, Expect::Value(1, 0)),
        (-1.0, Expect::Value(-1, 0)),
        (0.5, Expect::Value(5, 1)),
        (-0.5, Expect::Value(-5, 1)),
        (0.25, Expect::Value(25, 2)),
        (0.75, Expect::Value(75, 2)),
        (0.125, Expect::Value(125, 3)),
        (-0.375, Expect::Value(-375, 3)),
        (1.5, Expect::Value(15, 1)),
        (2.5, Expect::Value(25, 1)),
        (3.0, Expect::Value(3, 0)),
        (1024.0, Expect::Value(1024, 0)),
        (1e15, Expect::Value(10_i128.pow(15), 0)),
        (1e22, Expect::Value(10_i128.pow(22), 0)),
        (1e23, Expect::Value(99_999_999_999_999_991_611_392, 0)),
        (dyadic(false, 1, -38), Expect::Value(five, 38)),
        (dyadic(true, 1, -38), Expect::Value(-five, 38)),
        (dyadic(false, 3, -38), Expect::Value(3 * five, 38)),
        (dyadic(false, 1, -37), Expect::Value(5_i128.pow(37), 37)),
        (0.1, Expect::Scale(55)),
        (-0.1, Expect::Scale(55)),
        (0.2, Expect::Scale(54)),
        (0.3, Expect::Scale(54)),
        (2.675, Expect::Scale(50)),
        (1e-38, Expect::Scale(177)),
        (f64::EPSILON, Expect::Scale(52)),
        (dyadic(false, (1 << 53) - 1, -53), Expect::Scale(53)),
        (dyadic(false, 1, -39), Expect::Scale(39)),
        (f64::MIN_POSITIVE, Expect::Scale(1022)),
        (f64::from_bits(1), Expect::Scale(1074)),
        (f64::from_bits(SIGN | 1), Expect::Scale(1074)),
        (f64::from_bits(HIDDEN - 1), Expect::Scale(1074)),
    ] {
        assert_eq!(expected_exact(x), expected, "oracle {x:e}");
        check(exact_in(x), expected, EXACT_IN, &x);
    }
}

/// FR-BIN-03: the scale 37/38 coefficient limits are exact, and scale 39
/// reports `ResultScale` before any coefficient range check.
#[test]
fn exact_incoming_scale_37_38_39_edges_put_result_scale_before_range() {
    // Largest odd M with M·5^t <= i128::MAX, by integer division of i128::MAX.
    for (places, largest) in [(37_u8, 2_338_402_619_729_u64), (38, 467_680_523_945)] {
        let power = 5_i128.pow(u32::from(places));
        let limit = i128::from(largest) * power;
        assert_eq!(i128::MAX / power - i128::from(largest), 0);
        assert!(i128::MAX - limit < 2 * power, "the next odd M overflows");
        let exponent = -i32::from(places);
        for negative in [false, true] {
            let sign = if negative { -1 } else { 1 };
            for (m, expected) in [
                (1, Expect::Value(sign * power, places)),
                (3, Expect::Value(sign * 3 * power, places)),
                (largest, Expect::Value(sign * limit, places)),
                (largest + 2, Expect::Range(negative)),
                ((1 << 53) - 1, Expect::Range(negative)),
            ] {
                let x = dyadic(negative, m, exponent);
                assert_eq!(expected_exact(x), expected, "oracle {x:e}");
                check(exact_in(x), expected, EXACT_IN, &(m, places));
            }
        }
    }
    for negative in [false, true] {
        for (m, places) in [
            (1, 39),
            (3, 39),
            (467_680_523_945, 39),
            ((1 << 53) - 1, 39),
            (1, 40),
            ((1 << 53) - 1, 40),
        ] {
            let x = dyadic(negative, m, -i32::from(places));
            check(exact_in(x), Expect::Scale(places), EXACT_IN, &(m, places));
        }
    }
}

/// FR-BIN-03: exact integers keep the signed i128 asymmetry: `-2^127` is
/// `i128::MIN`, while `+2^127` and every larger magnitude are `OutOfRange`.
#[test]
fn exact_incoming_integer_edges_keep_signed_asymmetry() {
    assert_eq!(
        f64::MAX.to_bits(),
        dyadic(false, (1 << 53) - 1, 971).to_bits()
    );
    let top = 1_i128 << 126;
    let below = i128::MAX - ((1 << 74) - 1);
    for (x, expected) in [
        (dyadic(false, 1, 126), Expect::Value(top, 0)),
        (dyadic(true, 1, 126), Expect::Value(-top, 0)),
        (dyadic(false, (1 << 53) - 1, 74), Expect::Value(below, 0)),
        (dyadic(true, (1 << 53) - 1, 74), Expect::Value(-below, 0)),
        (dyadic(false, 1, 127), Expect::Range(false)),
        (dyadic(true, 1, 127), Expect::Value(i128::MIN, 0)),
        (dyadic(false, (1 << 52) + 1, 75), Expect::Range(false)),
        (dyadic(true, (1 << 52) + 1, 75), Expect::Range(true)),
        (dyadic(false, 1, 128), Expect::Range(false)),
        (dyadic(true, 1, 128), Expect::Range(true)),
        (f64::MAX, Expect::Range(false)),
        (f64::MIN, Expect::Range(true)),
        (
            dyadic(false, (1 << 53) - 1, 0),
            Expect::Value((1 << 53) - 1, 0),
        ),
        (dyadic(false, 1, 53), Expect::Value(1 << 53, 0)),
        (
            dyadic(false, (1 << 52) + 1, 1),
            Expect::Value((1 << 53) + 2, 0),
        ),
        (dyadic(true, 3, 125), Expect::Value(-3 * (1 << 125), 0)),
        (dyadic(false, 3, 126), Expect::Range(false)),
    ] {
        assert_eq!(expected_exact(x), expected, "oracle {x:e}");
        check(exact_in(x), expected, EXACT_IN, &x);
    }
}

/// FR-BIN-03: every subnormal leading-bit position needs scale `1074 - j`
/// (1074 with any lower bit set), so exact incoming reports that scale.
#[test]
fn exact_incoming_subnormal_leading_bits_need_their_full_scale() {
    for position in 0..52_u16 {
        let lead = 1_u64 << position;
        for sign in [0, SIGN] {
            for (bits, scale) in [
                (lead, 1074 - position),
                (lead | 1, 1074),
                ((lead << 1) - 1, 1074),
            ] {
                let x = f64::from_bits(sign | bits);
                assert!(x.is_subnormal());
                check(exact_in(x), Expect::Scale(scale), EXACT_IN, &bits);
            }
        }
    }
    for (bits, scale) in [
        (HIDDEN, 1022),
        (HIDDEN | 1, 1074),
        (2 * HIDDEN - 1, 1074),
        (2 * HIDDEN, 1021),
    ] {
        let x = f64::from_bits(bits);
        assert!(x.is_normal());
        check(exact_in(x), Expect::Scale(scale), EXACT_IN, &bits);
    }
}

/// FR-BIN-03, FR-BIN-06: on every biased exponent, at binade edges and their
/// neighbours, exact incoming matches the bit-field oracle and round-trips.
#[test]
fn exact_incoming_matches_bit_field_oracle_on_every_exponent_bin_and_round_trips() {
    const FRACTIONS: [u64; 8] = [
        0,
        1,
        2,
        3,
        1 << 51,
        (1 << 51) + 1,
        0x5_5555_5555_5555,
        HIDDEN - 1,
    ];
    let mut census = [0_usize; 4];
    for field in 0..=2046_u64 {
        for fraction in FRACTIONS {
            for sign in [0, SIGN] {
                let x = f64::from_bits(sign | field << 52 | fraction);
                let (negative, m, exponent) = parts(x);
                assert_eq!(dyadic(negative, m, exponent).to_bits(), x.to_bits());
                let expected = expected_exact(x);
                check(exact_in(x), expected, EXACT_IN, &x);
                let slot = match expected {
                    Expect::Value(0, _) => 0,
                    Expect::Value(..) => {
                        // -0.0 is the only input whose bits do not come back.
                        let value = good(exact_in(x));
                        let bits = if m == 0 { 0 } else { x.to_bits() };
                        assert_eq!(good(value.to_binary64_exact()).to_bits(), bits);
                        assert_eq!(value.to_binary64_nearest().to_bits(), bits);
                        1
                    }
                    Expect::Scale(_) => 2,
                    Expect::Range(_) | Expect::Inexact => 3,
                };
                census[slot] += 1;
            }
        }
    }
    // Independent Fraction census: zero, value, scale and range outcomes.
    assert_eq!(census, [2, 1_943, 16_382, 14_425]);
}

/// FR-BIN-04: binary64 half-quantum ties follow every mode with parity of the
/// quantum count, and their immediate neighbours leave the tie.
#[test]
fn rounded_incoming_half_quantum_ties_and_neighbours_follow_every_mode() {
    for (x, quantum, ties) in [
        (0.125, q(25, 2), ["0", "0.25", "0.25", "0", "0", "0.25"]),
        (-0.125, q(25, 2), ["0", "-0.25", "0", "-0.25", "0", "-0.25"]),
        (
            0.375,
            q(25, 2),
            ["0.25", "0.5", "0.5", "0.25", "0.5", "0.5"],
        ),
        (
            -0.375,
            q(25, 2),
            ["-0.25", "-0.5", "-0.25", "-0.5", "-0.5", "-0.5"],
        ),
        (0.5, q(1, 0), ["0", "1", "1", "0", "0", "1"]),
        (-0.5, q(1, 0), ["0", "-1", "0", "-1", "0", "-1"]),
        (1.5, q(1, 0), ["1", "2", "2", "1", "2", "2"]),
        (2.5, q(1, 0), ["2", "3", "3", "2", "2", "3"]),
        (3.5, q(1, 0), ["3", "4", "4", "3", "4", "4"]),
        (-2.5, q(1, 0), ["-2", "-3", "-2", "-3", "-2", "-3"]),
        (-3.5, q(1, 0), ["-3", "-4", "-3", "-4", "-4", "-4"]),
        (
            0.125,
            q(1, 2),
            ["0.12", "0.13", "0.13", "0.12", "0.12", "0.13"],
        ),
        (
            0.375,
            q(1, 2),
            ["0.37", "0.38", "0.38", "0.37", "0.38", "0.38"],
        ),
        (
            -0.625,
            q(1, 2),
            ["-0.62", "-0.63", "-0.62", "-0.63", "-0.62", "-0.63"],
        ),
        (
            0.875,
            q(1, 2),
            ["0.87", "0.88", "0.88", "0.87", "0.88", "0.88"],
        ),
        (
            0.125,
            q(5, 2),
            ["0.1", "0.15", "0.15", "0.1", "0.1", "0.15"],
        ),
        (0.375, q(5, 2), ["0.35", "0.4", "0.4", "0.35", "0.4", "0.4"]),
        (
            -0.125,
            q(5, 2),
            ["-0.1", "-0.15", "-0.1", "-0.15", "-0.1", "-0.15"],
        ),
        (1.5, q(3, 0), ["0", "3", "3", "0", "0", "3"]),
        (4.5, q(3, 0), ["3", "6", "6", "3", "6", "6"]),
        (7.5, q(3, 0), ["6", "9", "9", "6", "6", "9"]),
        (-4.5, q(3, 0), ["-3", "-6", "-3", "-6", "-6", "-6"]),
        (0.75, q(3, 1), ["0.6", "0.9", "0.9", "0.6", "0.6", "0.9"]),
        (2.25, q(3, 1), ["2.1", "2.4", "2.4", "2.1", "2.4", "2.4"]),
        (
            2.3125,
            q(37, 3),
            ["2.294", "2.331", "2.331", "2.294", "2.294", "2.331"],
        ),
        (
            -2.3125,
            q(37, 3),
            ["-2.294", "-2.331", "-2.294", "-2.331", "-2.294", "-2.331"],
        ),
    ] {
        assert_modes(x, quantum, ties);
        let [toward_zero, away, positive, negative, ..] = ties;
        // One ulp further from zero leaves the tie upward in magnitude, one ulp
        // nearer to zero leaves it downward; directed modes do not move.
        let farther = f64::from_bits(x.to_bits() + 1);
        let nearer = f64::from_bits(x.to_bits() - 1);
        assert_modes(
            farther,
            quantum,
            [toward_zero, away, positive, negative, away, away],
        );
        assert_modes(
            nearer,
            quantum,
            [
                toward_zero,
                away,
                positive,
                negative,
                toward_zero,
                toward_zero,
            ],
        );
    }
}

/// FR-BIN-04: binary64 `2.675` lies below the decimal `2.675`, so nearest
/// modes disagree with the decimal at cents while directed modes agree.
#[test]
fn binary_2_675_is_not_decimal_2_675_at_cent_and_mill_quanta() {
    // 2.675·2^51 = 6023564501608038.4, so binary64 2.675 is 3011782250804019/2^50.
    let binary = 2.675_f64;
    assert_eq!(
        binary.to_bits(),
        dyadic(false, 3_011_782_250_804_019, -50).to_bits()
    );
    // Cents: 100·M = 267·2^50 + 2^49 − 20, just below the half cent.
    assert_eq!(
        3_011_782_250_804_019_u128 * 100 + 20,
        267 * (1 << 50) + (1 << 49)
    );
    // Mills: 1000·M = 2675·2^50 − 200, far above the half mill.
    assert_eq!(3_011_782_250_804_019_u128 * 1000 + 200, 2675 * (1 << 50));
    let decimal = p("2.675");
    let cents = q(1, 2);
    let mills = q(1, 3);
    assert_modes(
        binary,
        cents,
        ["2.67", "2.68", "2.68", "2.67", "2.67", "2.67"],
    );
    assert_modes(
        -binary,
        cents,
        ["-2.67", "-2.68", "-2.67", "-2.68", "-2.67", "-2.67"],
    );
    assert_modes(
        binary,
        mills,
        ["2.674", "2.675", "2.675", "2.674", "2.675", "2.675"],
    );
    for (mode, answer) in Rounding::ALL
        .into_iter()
        .zip(["2.67", "2.68", "2.68", "2.67", "2.68", "2.68"])
    {
        assert_eq!(good(decimal.quantize(cents, mode)), p(answer));
        assert_eq!(good(decimal.quantize(mills, mode)), decimal);
    }
    assert_eq!(good(decimal.quantize(mills, Rounding::Exact)), decimal);
    assert_inexact(
        &rejected(decimal.quantize(cents, Rounding::Exact)),
        "quantize",
    );
    check(exact_in(binary), Expect::Scale(50), EXACT_IN, &binary);
    // Outgoing: the decimal rounds to this binary64, but is not one.
    assert!(assert_rounds_to(decimal, binary).is_none());
    // 1.005 = 1131529406376837/2^50; 100·M = 201·2^49 − 12 lies below the tie.
    let binary = 1.005_f64;
    assert_eq!(
        binary.to_bits(),
        dyadic(false, 1_131_529_406_376_837, -50).to_bits()
    );
    assert_eq!(1_131_529_406_376_837_u128 * 100 + 12, 201 * (1 << 49));
    assert_modes(binary, cents, ["1", "1.01", "1.01", "1", "1", "1"]);
    for (mode, answer) in Rounding::ALL
        .into_iter()
        .zip(["1", "1.01", "1.01", "1", "1", "1.01"])
    {
        assert_eq!(good(p("1.005").quantize(cents, mode)), p(answer));
    }
}

/// FR-BIN-04: at the finest quantum exact scale-39 inputs are ties resolved
/// by quantum-count parity, and exact multiples pass every mode unchanged.
#[test]
fn finest_quantum_ties_use_count_parity_and_multiples_pass_every_mode() {
    let finest = q(1, 38);
    let five = 5_i128.pow(38);
    // 2^-39 = (5^38/2)·1e-38; 5^38 ≡ 1 (mod 4), so the count (5^38−1)/2 is even.
    let even = (five - 1) / 2;
    // 3·2^-39: 3·5^38 ≡ 3 (mod 4), so the count (3·5^38−1)/2 is odd.
    let odd = (3 * five - 1) / 2;
    // 2^-40: a quarter quantum above the count (5^38−1)/4.
    let quarter = (five - 1) / 4;
    assert_eq!((even % 2, odd % 2, (five - 1) % 4), (0, 1, 0));
    for negative in [false, true] {
        let sign = if negative { -1 } else { 1 };
        let value = |count: i128| Expect::Value(sign * count, 38);
        for (m, exponent, down, nearest_even, nearest_away) in [
            (1, -39, even, even, even + 1),
            (3, -39, odd, odd + 1, odd + 1),
            (1, -40, quarter, quarter, quarter),
        ] {
            let up = down + 1;
            let (positive, negative_mode) = if negative { (down, up) } else { (up, down) };
            assert_all_modes(
                dyadic(negative, m, exponent),
                finest,
                [
                    value(down),
                    value(up),
                    value(positive),
                    value(negative_mode),
                    value(nearest_even),
                    value(nearest_away),
                    Expect::Inexact,
                ],
            );
        }
        for (m, exponent, coefficient, scale) in [
            (1, -38, five, 38),
            (3, -38, 3 * five, 38),
            (1, -37, 5_i128.pow(37), 37),
            (5, 0, 5, 0),
        ] {
            assert_all_modes(
                dyadic(negative, m, exponent),
                finest,
                [Expect::Value(sign * coefficient, scale); 7],
            );
        }
    }
    // Binary64 0.1 = 3602879701896397/2^55 keeps all 38 places: the count is
    // 10000000000000000555111512312578270211 with a remainder above the half.
    let count = 10_000_000_000_000_000_555_111_512_312_578_270_211_i128;
    let (quotient, half, exact) = divide(
        &Big::new(3_602_879_701_896_397).times(10_u128.pow(38)),
        1,
        55,
    );
    assert_eq!(
        (quotient.to_u128(), half, exact),
        (Some(count.unsigned_abs()), Greater, false)
    );
    for (x, sign) in [(0.1, 1), (-0.1, -1)] {
        let down = Expect::Value(sign * count, 38);
        let up = Expect::Value(sign * (count + 1), 38);
        let (positive, negative) = if sign > 0 { (up, down) } else { (down, up) };
        assert_all_modes(
            x,
            finest,
            [down, up, positive, negative, up, up, Expect::Inexact],
        );
    }
}

/// FR-BIN-04: subnormal and other tiny inputs validly round to canonical zero
/// or to exactly one quantum by mode and sign; `Exact` reports `Inexact`.
#[test]
fn tiny_inputs_round_to_zero_or_one_quantum_by_mode_and_sign() {
    // Every input here is at most 2^-128 < 0.3·1e-38: below half of any quantum.
    for x in [
        f64::from_bits(1),
        f64::from_bits(2),
        f64::from_bits(HIDDEN - 1),
        f64::MIN_POSITIVE,
        dyadic(false, 1, -1000),
        dyadic(false, (1 << 53) - 1, -181),
        dyadic(false, 1, -128),
        1e-300,
    ] {
        for quantum in [
            q(1, 38),
            q(3, 38),
            q(1, 2),
            q(5, 2),
            q(1, 0),
            q(i128::MAX, 0),
            q(i128::MAX, 38),
        ] {
            let unit = quantum.amount();
            let (one, scale) = (unit.coefficient(), unit.scale());
            let zero = Expect::Value(0, 0);
            assert_all_modes(
                x,
                quantum,
                [
                    zero,
                    Expect::Value(one, scale),
                    Expect::Value(one, scale),
                    zero,
                    zero,
                    zero,
                    Expect::Inexact,
                ],
            );
            assert_all_modes(
                -x,
                quantum,
                [
                    zero,
                    Expect::Value(-one, scale),
                    zero,
                    Expect::Value(-one, scale),
                    zero,
                    zero,
                    Expect::Inexact,
                ],
            );
        }
    }
    // 2^126 < 10^38 < 2^127 < 2·10^38 < 3·2^126: 2^-127 is above half of 1e-38,
    // and 2^-126 is one quantum plus less than a half.
    let ten = 10_u128.pow(38);
    assert!(1 << 126 < ten && ten < 1 << 127 && ten.checked_mul(2) < Some(3 << 126));
    let finest = q(1, 38);
    let units = |counts: [i128; 6]| {
        let mut expected = [Expect::Inexact; 7];
        for (slot, count) in expected.iter_mut().zip(counts) {
            *slot = Expect::Value(count, if count == 0 { 0 } else { 38 });
        }
        expected
    };
    assert_all_modes(dyadic(false, 1, -127), finest, units([0, 1, 1, 0, 1, 1]));
    assert_all_modes(dyadic(true, 1, -127), finest, units([0, -1, 0, -1, -1, -1]));
    assert_all_modes(dyadic(false, 1, -126), finest, units([1, 2, 2, 1, 1, 1]));
    assert_all_modes(
        dyadic(true, 1, -126),
        finest,
        units([-1, -2, -1, -2, -1, -1]),
    );
}

/// FR-BIN-04, FR-BIN-07: huge inputs are rounded first and range-classified
/// after; `Exact` reports `Inexact` only for a nonzero remainder.
#[test]
fn huge_inputs_are_range_classified_after_rounding() {
    // f64::MAX = (2^53−1)·2^971 and 2^53−1 = 6361·69431·20394401, while
    // 2^53−1 ≡ 1 (mod 3) and the Mersenne prime 2^127−1 divides no factor.
    assert_eq!(6361_u64 * 69_431 * 20_394_401, (1 << 53) - 1);
    for (quantum, divisible) in [
        (q(1, 0), true),
        (q(2, 0), true),
        (q(1, 2), true),
        (q(5, 2), true),
        (q(5, 1), true),
        (q(1, 38), true),
        (q(6361, 0), true),
        (q((1 << 53) - 1, 0), true),
        (q(3, 0), false),
        (q(5, 0), false),
        (q(7, 0), false),
        (q(37, 3), false),
        (q(3, 38), false),
        (q(i128::MAX, 0), false),
        (q(i128::MAX, 38), false),
    ] {
        for (x, negative) in [(f64::MAX, false), (f64::MIN, true)] {
            let mut expected = [Expect::Range(negative); 7];
            if !divisible {
                expected[6] = Expect::Inexact;
            }
            assert_all_modes(x, quantum, expected);
        }
    }
    let power = dyadic(false, 1, 127);
    let near = i128::MAX - 1;
    // 2^127 = 3·((2^127−2)/3) + 2: down is 2^127−2, up is 2^127+1.
    assert_all_modes(
        power,
        q(3, 0),
        [
            Expect::Value(near, 0),
            Expect::Range(false),
            Expect::Range(false),
            Expect::Value(near, 0),
            Expect::Range(false),
            Expect::Range(false),
            Expect::Inexact,
        ],
    );
    assert_all_modes(
        -power,
        q(3, 0),
        [
            Expect::Value(-near, 0),
            Expect::Range(true),
            Expect::Value(-near, 0),
            Expect::Range(true),
            Expect::Range(true),
            Expect::Range(true),
            Expect::Inexact,
        ],
    );
    // 2^127 = 1·(2^127−1) + 1: one quantum is in range, two are not.
    let max = Expect::Value(i128::MAX, 0);
    let min = Expect::Value(-i128::MAX, 0);
    assert_all_modes(
        power,
        q(i128::MAX, 0),
        [
            max,
            Expect::Range(false),
            Expect::Range(false),
            max,
            max,
            max,
            Expect::Inexact,
        ],
    );
    assert_all_modes(
        -power,
        q(i128::MAX, 0),
        [
            min,
            Expect::Range(true),
            min,
            Expect::Range(true),
            min,
            min,
            Expect::Inexact,
        ],
    );
    // Exact multiples: +2^127 fails even under Exact, −2^127 is i128::MIN after
    // normalizing counts as large as 2^127·10^38.
    for quantum in [q(1, 0), q(2, 0), q(1 << 126, 0), q(5, 2), q(1, 38)] {
        assert_all_modes(power, quantum, [Expect::Range(false); 7]);
        assert_all_modes(-power, quantum, [Expect::Value(i128::MIN, 0); 7]);
    }
    let below = i128::MAX - ((1 << 74) - 1);
    for quantum in [q(1, 0), q(1, 38), q(1 << 74, 0)] {
        assert_all_modes(
            dyadic(false, (1 << 53) - 1, 74),
            quantum,
            [Expect::Value(below, 0); 7],
        );
    }
    // 2^126 = 3·((2^126−1)/3) + 1: both neighbours stay in range.
    let top = 1_i128 << 126;
    assert_all_modes(
        dyadic(false, 1, 126),
        q(3, 0),
        [
            Expect::Value(top - 1, 0),
            Expect::Value(top + 2, 0),
            Expect::Value(top + 2, 0),
            Expect::Value(top - 1, 0),
            Expect::Value(top - 1, 0),
            Expect::Value(top - 1, 0),
            Expect::Inexact,
        ],
    );
    assert_all_modes(
        dyadic(true, (1 << 52) + 1, 75),
        q(1, 0),
        [Expect::Range(true); 7],
    );
}

/// FR-BIN-04: every multiple of 1/64 in [−8, 8] and both binary64 neighbours
/// match the exact quotient/remainder oracle for every mode and quantum.
#[test]
fn rounded_incoming_matches_quotient_remainder_oracle_on_a_dyadic_grid() {
    let grid_quanta = [
        q(1, 0),
        q(1, 1),
        q(1, 2),
        q(5, 2),
        q(25, 2),
        q(125, 3),
        q(3, 0),
        q(3, 1),
        q(7, 2),
        q(37, 3),
        q(1, 38),
        q(3, 38),
        q(i128::MAX, 38),
    ];
    let numerators: Vec<i64> = (-512..=512).collect();
    let census = parallel_census(|part, parts| {
        let mut census = [0_usize; 5];
        for &numerator in numerators.iter().skip(part).step_by(parts) {
            let center = dyadic(numerator < 0, numerator.unsigned_abs(), -6);
            let bits = center.to_bits();
            let inputs = if numerator == 0 {
                [center, f64::from_bits(1), f64::from_bits(SIGN | 1)]
            } else {
                [center, f64::from_bits(bits - 1), f64::from_bits(bits + 1)]
            };
            for x in inputs {
                for quantum in grid_quanta {
                    let expected = assert_oracle_modes(x, quantum, &mut census);
                    // Ties-even and ties-away differ only on an even-count tie.
                    census[4] += usize::from(expected[4] != expected[5]);
                }
            }
        }
        census
    });
    // Independent Fraction census: zero, value, range and inexact outcomes,
    // then the number of even-count ties.
    assert_eq!(census, [7_723, 202_152, 31_744, 38_206, 202]);
}

/// FR-BIN-04, FR-BIN-07: across the binades from the smallest subnormal to
/// `f64::MAX`, rounded incoming matches the exact oracle at extreme quanta.
#[test]
fn rounded_incoming_matches_quotient_remainder_oracle_across_the_binades() {
    let extreme_quanta = [q(1, 0), q(1, 38), q(3, 0), q(i128::MAX, 38)];
    // Every biased exponent up to 1249 (below 2^227). Beyond it only Range and
    // Inexact remain, so a stride of 37, coprime to the remainder periods 2
    // (q=3) and 127 (2^127-1), plus the top edge, samples the widest inputs.
    let fields: Vec<u64> = (0..1250)
        .chain((1250..=2046).step_by(37))
        .chain(2040..=2046)
        .collect();
    let census = parallel_census(|part, parts| {
        let mut census = [0_usize; 4];
        for &field in fields.iter().skip(part).step_by(parts) {
            for fraction in [0, 1, HIDDEN - 1] {
                for sign in [0, SIGN] {
                    let x = f64::from_bits(sign | field << 52 | fraction);
                    for quantum in extreme_quanta {
                        assert_oracle_modes(x, quantum, &mut census);
                    }
                }
            }
        }
        census
    });
    // Independent Fraction census: zero, value, range and inexact outcomes.
    assert_eq!(census, [95_210, 66_746, 25_112, 27_804]);
}

/// FR-BIN-05, FR-BIN-06: exact dyadic decimals leave as those binary64
/// values, while uncancelled powers of five are `Inexact` with nearest bits.
#[test]
fn outgoing_dyadics_are_exact_and_uncancelled_fives_are_inexact() {
    let five = 5_i128.pow(38);
    for (value, bits, exact) in [
        (a(1, 0), 0x3FF0_0000_0000_0000, true),
        (a(-1, 0), 0xBFF0_0000_0000_0000, true),
        (a(5, 1), 0x3FE0_0000_0000_0000, true),
        (a(-5, 1), 0xBFE0_0000_0000_0000, true),
        (a(25, 2), 0x3FD0_0000_0000_0000, true),
        (a(75, 2), 0x3FE8_0000_0000_0000, true),
        (a(five, 38), 0x3D90_0000_0000_0000, true),
        (a(-five, 38), 0xBD90_0000_0000_0000, true),
        (a(3 * five, 38), 0x3DA8_0000_0000_0000, true),
        (a(1 << 126, 0), 0x47D0_0000_0000_0000, true),
        (
            a(i128::MAX - ((1 << 74) - 1), 0),
            0x47DF_FFFF_FFFF_FFFF,
            true,
        ),
        (a(i128::MIN, 0), 0xC7E0_0000_0000_0000, true),
        (a((1 << 53) - 1, 0), 0x433F_FFFF_FFFF_FFFF, true),
        (a(1 << 53, 0), 0x4340_0000_0000_0000, true),
        (a((1 << 53) + 2, 0), 0x4340_0000_0000_0001, true),
        (a(i128::MAX, 0), 0x47E0_0000_0000_0000, false),
        (a(i128::MIN + 1, 0), 0xC7E0_0000_0000_0000, false),
        (a((1 << 53) + 1, 0), 0x4340_0000_0000_0000, false),
        (a((1 << 53) + 3, 0), 0x4340_0000_0000_0002, false),
        (a(1, 1), 0x3FB9_9999_9999_999A, false),
        (a(-1, 1), 0xBFB9_9999_9999_999A, false),
        (a(5_i128.pow(37), 38), 0x3D69_9999_9999_999A, false),
        (a(1, 38), 0x380B_38FB_9DAA_78E4, false),
        (a(i128::MAX, 38), 0x3FFB_38FB_9DAA_78E4, false),
        (a(i128::MIN, 38), 0xBFFB_38FB_9DAA_78E4, false),
        (a(i128::MAX, 37), 0x4031_039D_428A_8B8F, false),
        (a(i128::MIN, 37), 0xC031_039D_428A_8B8F, false),
        (a(i128::MAX, 19), 0x43ED_83C9_4FB6_D2AC, false),
        (a(i128::MIN, 19), 0xC3ED_83C9_4FB6_D2AC, false),
        (a(i128::MAX, 1), 0x47A9_9999_9999_999A, false),
        (a(i128::MIN, 1), 0xC7A9_9999_9999_999A, false),
    ] {
        assert_eq!(
            expected_nearest(value.coefficient(), value.scale()),
            (bits, exact),
            "oracle {value}"
        );
        let binary = assert_rounds_to(value, f64::from_bits(bits));
        assert_eq!(binary.is_some(), exact, "{value}");
    }
    assert_eq!(a(1, 1).to_binary64_nearest().to_bits(), 0.1_f64.to_bits());
    assert_eq!(
        a(1, 38).to_binary64_nearest().to_bits(),
        1e-38_f64.to_bits()
    );
}

/// FR-BIN-05, FR-BIN-06: a unit coefficient at every scale 0..=38 leaves as
/// the nearest-even binary64 of `10^-scale`; only scale 0 is exact.
#[test]
fn outgoing_unit_coefficient_at_every_scale_has_nearest_even_bits() {
    for (scale, bits) in (0..=38_u8).zip(UNIT_BITS) {
        for (coefficient, sign) in [(1, 0), (-1, SIGN)] {
            let value = a(coefficient, scale);
            assert_eq!(
                expected_nearest(coefficient, scale),
                (sign | bits, scale == 0)
            );
            let binary = assert_rounds_to(value, f64::from_bits(sign | bits));
            assert_eq!(binary.is_some(), scale == 0);
        }
        // 10^scale at that scale is canonical one, exactly.
        let one = a(10_i128.pow(u32::from(scale)), scale);
        assert_eq!(good(one.to_binary64_exact()).to_bits(), UNIT_BITS[0]);
    }
}

/// FR-BIN-05, FR-BIN-06: decimals equal to exact binary64 midpoints round to
/// the even significand, including a carry into the next binade.
#[test]
fn outgoing_midpoints_tie_to_even_including_carry_into_the_next_binade() {
    let two53 = 1_i128 << 53;
    // A midpoint needs 54 significant bits, so (2^54−1)·5^k fits only for k<=31.
    assert!((2 * two53 - 1).checked_mul(5_i128.pow(32)).is_none());
    for places in 0..=31_u8 {
        let power = 5_i128.pow(u32::from(places));
        let shift = i32::from(places);
        for negative in [false, true] {
            let sign = if negative { -1 } else { 1 };
            // (2^54−1)·2^-k lies midway between (2^53−1)·2^(1−k) and 2^(54−k);
            // the even significand is the first of the next binade.
            let carry = (2 * two53 - 1) * power;
            let next_binade = dyadic(negative, 1, 54 - shift);
            let largest = dyadic(negative, (1 << 53) - 1, 1 - shift);
            assert!(assert_rounds_to(a(sign * carry, places), next_binade).is_none());
            assert_rounds_to(a(sign * (carry - 1), places), largest);
            assert_rounds_to(a(sign * (carry + 1), places), next_binade);
            // (2^53+1)·2^-k: the lower neighbour 2^52·2^(1−k) is even.
            let low_tie = (two53 + 1) * power;
            let even_low = dyadic(negative, 1, 53 - shift);
            assert!(assert_rounds_to(a(sign * low_tie, places), even_low).is_none());
            // (2^53+3)·2^-k: the upper neighbour (2^52+2)·2^(1−k) is even.
            let high_tie = (two53 + 3) * power;
            let even_high = dyadic(negative, (1 << 52) + 2, 1 - shift);
            assert!(assert_rounds_to(a(sign * high_tie, places), even_high).is_none());
            let odd_high = dyadic(negative, (1 << 52) + 1, 1 - shift);
            assert_rounds_to(a(sign * (high_tie - 1), places), odd_high);
        }
    }
}

/// FR-BIN-05, FR-BIN-06: coefficient limits, 53/54-bit transitions and
/// uncancelled fives at every scale match the integer nearest-even oracle.
#[test]
fn outgoing_conversions_match_the_integer_nearest_oracle_at_every_scale() {
    let mut coefficients = vec![
        1,
        2,
        3,
        7,
        9,
        10,
        11,
        99,
        101,
        999_999_999_999_999_999,
        (1 << 52) - 1,
        1 << 52,
        (1 << 52) + 1,
        (1 << 53) - 1,
        1 << 53,
        (1 << 53) + 1,
        (1 << 53) + 2,
        (1 << 53) + 3,
        (1 << 54) - 1,
        (1 << 54) + 1,
        (1 << 64) + 1,
        1 << 126,
        (1 << 126) + 1,
        10_i128.pow(38) - 1,
        10_i128.pow(38),
        10_i128.pow(38) + 1,
        i128::MAX - 1,
        i128::MAX,
    ];
    for power in 0..=54 {
        let five = 5_i128.pow(power);
        coefficients.extend([five, five + 1, five - 1]);
        coefficients.extend(five.checked_mul(3));
        coefficients.extend(five.checked_mul((1 << 53) - 1));
        coefficients.extend(five.checked_mul((1 << 53) + 1));
    }
    let mut exact = 0;
    let mut total = 0;
    for scale in 0..=38 {
        for &coefficient in &coefficients {
            for signed in [coefficient, -coefficient] {
                total += 1;
                exact += usize::from(assert_outgoing(a(signed, scale)).is_some());
            }
        }
        for extreme in [i128::MIN, i128::MIN + 1] {
            total += 1;
            exact += usize::from(assert_outgoing(a(extreme, scale)).is_some());
        }
    }
    assert!(exact > 0 && exact < total, "{exact} of {total}");
}

/// FR-BIN-06: exact conversions round-trip in both directions, `-0.0` comes
/// back as `+0.0`, and a representable decimal is every mode's multiple.
#[test]
fn exact_round_trips_hold_in_both_directions() {
    let mut checked = 0;
    for places in 0..=38_u8 {
        let power = 5_i128.pow(u32::from(places));
        for odd in [1_u64, 3, 5, 7, 999, (1 << 41) + 1, (1 << 53) - 1] {
            let Some(coefficient) = i128::from(odd).checked_mul(power) else {
                continue;
            };
            for negative in [false, true] {
                let signed = if negative { -coefficient } else { coefficient };
                let value = a(signed, places);
                let binary = dyadic(negative, odd, -i32::from(places));
                assert_eq!(good(value.to_binary64_exact()).to_bits(), binary.to_bits());
                assert_eq!(good(exact_in(binary)), value);
                for mode in Rounding::ALL {
                    assert_eq!(good(rounded_in(binary, q(1, places), mode)), value);
                }
                checked += 1;
            }
        }
    }
    for exponent in 0..=127_i32 {
        for odd in [1_u64, 3, (1 << 53) - 1] {
            for negative in [false, true] {
                let binary = dyadic(negative, odd, exponent);
                let Ok(value) = exact_in(binary) else {
                    continue;
                };
                assert_eq!(good(value.to_binary64_exact()).to_bits(), binary.to_bits());
                assert_eq!(value.scale(), 0);
                checked += 1;
            }
        }
    }
    assert_eq!(good(good(exact_in(-0.0)).to_binary64_exact()).to_bits(), 0);
    assert!(checked > 300, "{checked}");
}

/// FR-BIN-02, FR-BIN-08: binary boundary errors keep stable kinds, their own
/// operation strings and no nested source; the root keeps one source layer.
#[test]
fn binary_boundary_errors_keep_kinds_operations_and_one_root_source_layer() {
    let errors = [
        (rejected(exact_in(f64::NAN)), "amount.non_finite", EXACT_IN),
        (
            rejected(rounded_in(f64::INFINITY, q(1, 2), Rounding::Exact)),
            "amount.non_finite",
            ROUNDED_IN,
        ),
        (
            rejected(rounded_in(
                f64::NEG_INFINITY,
                q(1, 38),
                Rounding::TowardZero,
            )),
            "amount.non_finite",
            ROUNDED_IN,
        ),
        (rejected(exact_in(0.1)), "amount.result_scale", EXACT_IN),
        (
            rejected(exact_in(dyadic(false, 1, 127))),
            "amount.out_of_range",
            EXACT_IN,
        ),
        (
            rejected(rounded_in(0.1, q(1, 2), Rounding::Exact)),
            "amount.inexact",
            ROUNDED_IN,
        ),
        (
            rejected(rounded_in(f64::MAX, q(1, 0), Rounding::Exact)),
            "amount.out_of_range",
            ROUNDED_IN,
        ),
        (
            rejected(a(1, 1).to_binary64_exact()),
            "amount.inexact",
            EXACT_OUT,
        ),
    ];
    for (error, kind, operation) in &errors {
        assert_eq!(error.kind(), *kind);
        assert!(error.source().is_none());
        assert_eq!(error, &error.clone());
        assert!(!format!("{error:?}").is_empty());
        // Display wording is not stable, but like phase A it names the operation.
        let text = error.to_string();
        assert!(text.contains(operation), "{text}");
        let root: CrateError = error.clone().into();
        assert_eq!(root.kind(), *kind);
        assert_eq!(root.to_string(), text);
        assert!(matches!(&root, CrateError::Amount(cause) if cause == error));
        assert!(
            root.source()
                .is_some_and(|source| source.downcast_ref::<AmountError>() == Some(error)
                    && source.source().is_none())
        );
    }
    for (left, right) in [(0, 1), (1, 2), (5, 7), (4, 6)] {
        assert_ne!(errors[left].0, errors[right].0);
    }
}

/// Inherent constants apply only when their bound holds; otherwise the trait
/// default answers `false`. This is a std-only trait-absence probe.
struct Probe<T>(PhantomData<T>);

trait Absent {
    const FROM_F64: bool = false;
    const FROM_F32: bool = false;
    const INTO_F64: bool = false;
    const INTO_F32: bool = false;
}

impl<T> Absent for Probe<T> {}

impl<T: TryFrom<f64>> Probe<T> {
    const FROM_F64: bool = true;
}

impl<T: TryFrom<f32>> Probe<T> {
    const FROM_F32: bool = true;
}

impl<T> Probe<T>
where
    f64: TryFrom<T>,
{
    const INTO_F64: bool = true;
}

impl<T> Probe<T>
where
    f32: TryFrom<T>,
{
    const INTO_F32: bool = true;
}

/// FR-BIN-01, FR-BIN-07: no binary64/binary32 `From`/`TryFrom` exists in either
/// direction, and conversions share immutable values across threads.
#[test]
fn conversions_add_no_implicit_binary_traits_and_share_across_threads() {
    // Controls: the probe sees std's real impls and the missing f64 -> f32.
    assert_eq!(
        [
            Probe::<f64>::FROM_F64,
            Probe::<f64>::FROM_F32,
            Probe::<f32>::INTO_F64,
            Probe::<f32>::INTO_F32,
            Probe::<f32>::FROM_F64,
        ],
        [true, true, true, true, false]
    );
    assert_eq!(
        [
            Probe::<DecimalAmount>::FROM_F64,
            Probe::<DecimalAmount>::FROM_F32,
            Probe::<DecimalAmount>::INTO_F64,
            Probe::<DecimalAmount>::INTO_F32,
            Probe::<AmountQuantum>::FROM_F64,
            Probe::<AmountQuantum>::FROM_F32,
            Probe::<AmountQuantum>::INTO_F64,
            Probe::<AmountQuantum>::INTO_F32,
        ],
        [false; 8]
    );
    fn shared<T: Clone + Debug + Eq + Send + Sync + 'static>() {}
    shared::<AmountError>();
    shared::<NonFinite>();
    shared::<DecimalAmount>();
    let inputs = [
        0.1,
        -2.675,
        f64::from_bits(1),
        f64::MAX,
        f64::NAN,
        dyadic(true, 1, 127),
        -0.0,
    ];
    let cents = q(1, 2);
    let expected: Vec<_> = inputs
        .iter()
        .map(|&x| (rounded_in(x, cents, Rounding::NearestTiesEven), exact_in(x)))
        .collect();
    assert_eq!(expected[0].0, Ok(p("0.1")));
    assert_eq!(expected[1].0, Ok(p("-2.67")));
    assert_eq!(expected[5].1, Ok(a(i128::MIN, 0)));
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|offset| {
                let (inputs, expected) = (&inputs, &expected);
                scope.spawn(move || {
                    for round in 0..64 {
                        let index = (offset + round) % inputs.len();
                        let x = inputs[index];
                        let (rounded, exact) = &expected[index];
                        assert_eq!(&rounded_in(x, cents, Rounding::NearestTiesEven), rounded);
                        assert_eq!(&exact_in(x), exact);
                        if let Ok(value) = exact {
                            let back = good(value.to_binary64_exact());
                            assert_eq!(good(exact_in(back)), *value);
                        }
                    }
                })
            })
            .collect();
        for worker in workers {
            assert!(worker.join().is_ok());
        }
    });
}
