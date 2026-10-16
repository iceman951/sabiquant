//! One-round binary64 products for immutable cash-flow amounts.
//!
//! The private integer kernel retains all 159 possible significand bits and
//! rounds once. This wrapper applies the common numeric result policy after
//! rounding, rejecting nonzero results rounded to zero or infinity.

use crate::error::{Interval, ValueError};

mod binary;

/// Computes the correctly rounded exact product of three finite operands.
///
/// The parent public boundary has already checked input finiteness. Exact
/// zero is canonical positive zero. A nonzero exact product that rounds to
/// zero or infinity is rejected through the shared numeric result policy.
/// Its signed domain distinguishes negative from positive underflow.
///
/// # Errors
///
/// Returns [`ValueError::ResultNotFinite`] for signed overflow and
/// [`ValueError::ResultOutOfDomain`] for nonzero underflow to signed zero.
pub(crate) fn product(quantity: &'static str, a: f64, b: f64, c: f64) -> Result<f64, ValueError> {
    let rounded = binary::multiply(a, b, c);
    if [a, b, c].iter().any(|value| value.to_bits() << 1 == 0) {
        return Ok(rounded);
    }
    let domain = if rounded.is_sign_negative() {
        Interval::less_than(0.0)
    } else {
        Interval::greater_than(0.0)
    };
    crate::numeric::result_within(quantity, rounded, domain)
}

#[cfg(test)]
mod tests {
    use super::product;
    use crate::error::{Interval, NonFinite, ValueError};

    #[test]
    fn intermediate_overflow_and_underflow_do_not_change_the_exact_product() {
        let tiny = f64::from_bits(1);
        for operands in [
            [f64::MAX, 2.0, 0.5],
            [tiny, 0.5, 2.0],
            [f64::MIN_POSITIVE, f64::MAX, f64::MIN_POSITIVE],
        ] {
            let expected = product("amount", operands[0], operands[1], operands[2])
                .expect("the exact final product is representable");
            for [i, j, k] in [
                [0, 1, 2],
                [0, 2, 1],
                [1, 0, 2],
                [1, 2, 0],
                [2, 0, 1],
                [2, 1, 0],
            ] {
                let actual = product("amount", operands[i], operands[j], operands[k])
                    .expect("a permutation has the same exact final product");
                assert_eq!(actual.to_bits(), expected.to_bits());
            }
        }
        assert_eq!(
            product("amount", f64::MAX, 2.0, 0.5)
                .expect("MAX")
                .to_bits(),
            f64::MAX.to_bits()
        );
        assert_eq!(
            product("amount", tiny, 0.5, 2.0)
                .expect("minimum subnormal")
                .to_bits(),
            1
        );
        // 2^-1022 * 2^-1022 * MAX = 2^-1020 - 2^-1073 exactly.
        assert_eq!(
            product("amount", f64::MIN_POSITIVE, f64::MIN_POSITIVE, f64::MAX)
                .expect("rescued normal product")
                .to_bits(),
            (3_u64 << 52) - 1
        );
    }

    #[test]
    fn exact_zero_is_canonical_for_every_sign_and_position() {
        for zero in [0.0, -0.0] {
            for a in [f64::MAX, -f64::MAX] {
                for b in [f64::from_bits(1), -f64::from_bits(1)] {
                    for [x, y, z] in [[zero, a, b], [a, zero, b], [a, b, zero]] {
                        assert_eq!(product("amount", x, y, z).expect("exact zero").to_bits(), 0);
                    }
                }
            }
        }
    }

    #[test]
    fn normal_and_subnormal_midpoints_round_to_even() {
        let one = 1.0_f64.to_bits();
        let base = 1.5_f64.to_bits();
        for (steps, expected) in [(1, 2), (3, 4), (5, 8), (7, 10)] {
            let result =
                product("amount", f64::from_bits(one + steps), 1.5, 1.0).expect("normal midpoint");
            assert_eq!(result.to_bits(), base + expected);
        }
        for (factor, expected) in [(1.5, 2), (2.5, 2), (3.5, 4), (4.5, 4)] {
            assert_eq!(
                product("amount", f64::from_bits(1), factor, 1.0)
                    .expect("subnormal midpoint")
                    .to_bits(),
                expected
            );
        }
        assert_eq!(
            product("amount", f64::from_bits(1), 9_007_199_254_740_991.0, 0.5)
                .expect("tie carries from largest subnormal to minimum normal")
                .to_bits(),
            f64::MIN_POSITIVE.to_bits()
        );
        assert_eq!(
            product(
                "amount",
                1.0 + 2.0_f64.powi(-27),
                1.0 - 2.0_f64.powi(-27),
                1.0
            )
            .expect("exact product 1 - 2^-54 ties to even")
            .to_bits(),
            1.0_f64.to_bits()
        );
    }

    #[test]
    fn nonzero_underflow_and_overflow_retain_quantity_and_sign() {
        let tiny = f64::from_bits(1);
        for (sign, domain, zero, infinity) in [
            (
                1.0,
                Interval::greater_than(0.0),
                0.0_f64,
                NonFinite::PositiveInfinity,
            ),
            (
                -1.0,
                Interval::less_than(0.0),
                -0.0_f64,
                NonFinite::NegativeInfinity,
            ),
        ] {
            let actual = product("coupon amount", tiny, 0.5, sign).expect_err("nonzero underflow");
            match actual {
                ValueError::ResultOutOfDomain {
                    quantity,
                    value,
                    domain: actual_domain,
                } => {
                    assert_eq!(quantity, "coupon amount");
                    assert_eq!(value.to_bits(), zero.to_bits());
                    assert_eq!(actual_domain, domain);
                }
                error => panic!("unexpected underflow cause: {error}"),
            }
            assert_eq!(
                product("coupon amount", f64::MAX, 2.0, sign),
                Err(ValueError::ResultNotFinite {
                    quantity: "coupon amount",
                    value: infinity
                })
            );
        }
    }

    #[test]
    fn exact_overflow_midpoint_ties_to_infinity() {
        // 3 * 6004799503160661 = 2^54 - 1 exactly, and all three inputs
        // are binary64 values. Multiplying by 2^970 gives the exact midpoint.
        let factor = 6_004_799_503_160_661.0_f64;
        let power = f64::from_bits(1993_u64 << 52);
        assert_eq!(
            product("amount", 3.0, factor, power),
            Err(ValueError::ResultNotFinite {
                quantity: "amount",
                value: NonFinite::PositiveInfinity
            })
        );
        let below =
            product("amount", 3.0, factor.next_down(), power).expect("below overflow midpoint");
        assert_eq!(below.to_bits(), f64::MAX.next_down().to_bits());
    }
}
