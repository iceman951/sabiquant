//! The numeric validation policy (internal).
//!
//! Every public operation that takes or computes real numbers validates them
//! with these helpers, so that NP-01 and NP-02 of the
//! [numerical correctness policy](https://github.com/iceman951/sabiquant/blob/main/docs/NUMERICAL_CORRECTNESS.md#5-numerical-policies)
//! are implemented in one place ([ADR-0006](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0006-numeric-policy.md)):
//!
//! - **NP-01**: inputs are checked for finiteness at the public boundary
//!   ([`finite`], [`within`]; FR-CORE-03);
//! - **NP-02**: results are finite and inside their domain ([`finite_result`],
//!   [`result_within`]), so an overflow or underflow that makes a result
//!   meaningless becomes an error instead of an infinity or a zero.
//!
//! The helpers return the value unchanged when it is accepted. They check the
//! value before any domain test, so a NaN is always reported as not finite,
//! never as out of domain. This module contains no financial conventions
//! (ARCHITECTURE §6).
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the domain modules call these helpers from M2 (dates) and M4 (rates)"
    )
)]

use crate::error::{Interval, NonFinite, ValueError};

/// Accepts a real-valued input if it is finite (NP-01).
///
/// # Errors
///
/// [`ValueError::NotFinite`] naming `parameter` if `value` is NaN or infinite.
pub(crate) fn finite(parameter: &'static str, value: f64) -> Result<f64, ValueError> {
    match NonFinite::classify(value) {
        None => Ok(value),
        Some(class) => Err(ValueError::NotFinite {
            parameter,
            value: class,
        }),
    }
}

/// Accepts a real-valued input if it is finite and lies in `domain` (NP-01).
///
/// # Errors
///
/// - [`ValueError::NotFinite`] if `value` is NaN or infinite;
/// - [`ValueError::OutOfDomain`] if it is finite but outside `domain`.
pub(crate) fn within(
    parameter: &'static str,
    value: f64,
    domain: Interval,
) -> Result<f64, ValueError> {
    let value = finite(parameter, value)?;
    if domain.contains(value) {
        Ok(value)
    } else {
        Err(ValueError::OutOfDomain {
            parameter,
            value,
            domain,
        })
    }
}

/// Accepts a computed result if it is finite (NP-02).
///
/// # Errors
///
/// [`ValueError::ResultNotFinite`] naming `quantity` if `value` is NaN or
/// infinite, for example after an overflow.
pub(crate) fn finite_result(quantity: &'static str, value: f64) -> Result<f64, ValueError> {
    match NonFinite::classify(value) {
        None => Ok(value),
        Some(class) => Err(ValueError::ResultNotFinite {
            quantity,
            value: class,
        }),
    }
}

/// Accepts a computed result if it is finite and lies in `domain` (NP-02).
///
/// # Errors
///
/// - [`ValueError::ResultNotFinite`] if `value` is NaN or infinite;
/// - [`ValueError::ResultOutOfDomain`] if it is finite but outside `domain`,
///   for example a compound factor that underflowed to zero.
pub(crate) fn result_within(
    quantity: &'static str,
    value: f64,
    domain: Interval,
) -> Result<f64, ValueError> {
    let value = finite_result(quantity, value)?;
    if domain.contains(value) {
        Ok(value)
    } else {
        Err(ValueError::ResultOutOfDomain {
            quantity,
            value,
            domain,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::prng::{self, SplitMix64};
    use std::ops::Bound;

    const EVERYTHING: Interval = Interval::new(Bound::Unbounded, Bound::Unbounded);

    const NON_FINITE: [(f64, NonFinite); 3] = [
        (f64::NAN, NonFinite::NaN),
        (f64::INFINITY, NonFinite::PositiveInfinity),
        (f64::NEG_INFINITY, NonFinite::NegativeInfinity),
    ];

    /// Finite edge cases that every validator must accept unchanged.
    const FINITE_EDGES: [f64; 8] = [
        0.0,
        -0.0,
        f64::MIN_POSITIVE,
        f64::MAX,
        f64::MIN,
        f64::EPSILON,
        1.0,
        -1.0,
    ];

    /// TS-CORE-01, NP-01, FR-CORE-03: non-finite inputs are rejected with the
    /// parameter name and the kind of non-finite value.
    #[test]
    fn non_finite_inputs_are_rejected() {
        for (value, class) in NON_FINITE {
            let expected = ValueError::NotFinite {
                parameter: "rate",
                value: class,
            };
            assert_eq!(finite("rate", value), Err(expected.clone()));
            // Finiteness is checked before the domain, so a NaN is never
            // reported as out of domain.
            assert_eq!(
                within("rate", value, Interval::at_least(0.0)),
                Err(expected)
            );
        }
    }

    /// NP-01, NP-08: finite inputs, including signed zeros and subnormals,
    /// are accepted and returned bit for bit.
    #[test]
    fn finite_inputs_are_accepted_unchanged() {
        let subnormal = f64::from_bits(1);
        for value in FINITE_EDGES.into_iter().chain([subnormal, -subnormal]) {
            let accepted = finite("x", value).expect("finite input");
            assert_eq!(accepted.to_bits(), value.to_bits());
            let accepted = within("x", value, EVERYTHING).expect("in domain");
            assert_eq!(accepted.to_bits(), value.to_bits());
        }
    }

    /// TS-CORE-01: a finite input outside its domain is rejected with the
    /// parameter, the value and the domain.
    #[test]
    fn out_of_domain_inputs_are_rejected() {
        let positive = Interval::greater_than(0.0);
        assert_eq!(
            within("time", -0.25, positive),
            Err(ValueError::OutOfDomain {
                parameter: "time",
                value: -0.25,
                domain: positive,
            })
        );
        // Both zeros lie outside (0, +inf) and inside [0, +inf) (NP-08).
        for zero in [0.0, -0.0] {
            assert!(within("time", zero, positive).is_err());
            assert!(within("time", zero, Interval::at_least(0.0)).is_ok());
        }
        // Bounds: exclusive and inclusive ends.
        let below = Interval::less_than(1.0);
        assert!(within("x", 1.0, below).is_err());
        assert!(within("x", 1.0_f64.next_down(), below).is_ok());
        assert!(within("x", 1.0, Interval::at_most(1.0)).is_ok());
    }

    /// TS-CORE-01, NP-02: non-finite results are rejected with the quantity
    /// name and the kind of non-finite value.
    #[test]
    fn non_finite_results_are_rejected() {
        let overflowed = f64::MAX * 2.0;
        assert_eq!(
            finite_result("compound factor", overflowed),
            Err(ValueError::ResultNotFinite {
                quantity: "compound factor",
                value: NonFinite::PositiveInfinity,
            })
        );
        for (value, class) in NON_FINITE {
            let expected = ValueError::ResultNotFinite {
                quantity: "discount factor",
                value: class,
            };
            assert_eq!(
                finite_result("discount factor", value),
                Err(expected.clone())
            );
            assert_eq!(
                result_within("discount factor", value, Interval::greater_than(0.0)),
                Err(expected)
            );
        }
    }

    /// TS-CORE-01, NP-02: a finite result outside its domain is rejected, for
    /// example a compound factor that underflowed to zero.
    #[test]
    fn out_of_domain_results_are_rejected() {
        let positive = Interval::greater_than(0.0);
        let underflowed = f64::MIN_POSITIVE * f64::EPSILON * f64::EPSILON;
        assert_eq!(underflowed.to_bits(), 0, "the product underflows to +0.0");
        assert_eq!(
            result_within("compound factor", underflowed, positive),
            Err(ValueError::ResultOutOfDomain {
                quantity: "compound factor",
                value: underflowed,
                domain: positive,
            })
        );
        for value in FINITE_EDGES {
            let accepted = finite_result("x", value).expect("finite result");
            assert_eq!(accepted.to_bits(), value.to_bits());
        }
        assert!(result_within("x", 0.5, positive).is_ok());
    }

    /// NP-01 property, over arbitrary bit patterns (NaNs, infinities,
    /// subnormals and normals): `finite` accepts exactly the finite values,
    /// and `within` accepts exactly the finite values inside the domain.
    #[test]
    fn validators_agree_with_the_definition_on_arbitrary_bit_patterns() {
        let domains = [
            Interval::greater_than(-2.0),
            Interval::at_least(0.0),
            Interval::less_than(1.0),
            Interval::at_most(0.0),
            EVERYTHING,
        ];
        prng::check("validators match definition", 0x5AB1_0001, 20_000, |rng| {
            // Mix arbitrary bit patterns with ordinary magnitudes, which
            // arbitrary bits rarely produce near the domain bounds.
            let value = if rng.below(2) == 0 {
                rng.any_f64()
            } else {
                (rng.next_f64() - 0.5) * 8.0
            };
            let accepted = finite("x", value).is_ok();
            if accepted != value.is_finite() {
                return Err(format!("finite({value:?}) accepted = {accepted}"));
            }
            for domain in domains {
                let accepted = within("x", value, domain).is_ok();
                let expected = value.is_finite() && domain.contains(value);
                if accepted != expected {
                    return Err(format!("within({value:?}, {domain}) accepted = {accepted}"));
                }
                let accepted = result_within("x", value, domain).is_ok();
                if accepted != expected {
                    return Err(format!(
                        "result_within({value:?}, {domain}) accepted = {accepted}"
                    ));
                }
            }
            Ok(())
        });
    }

    /// The class of a non-finite value, read from the IEEE 754 encoding
    /// itself: an independent oracle for `NonFinite::classify`.
    fn class_from_bits(value: f64) -> Option<NonFinite> {
        let bits = value.to_bits();
        let exponent = (bits >> 52) & 0x7FF;
        let mantissa = bits & ((1 << 52) - 1);
        match (exponent, mantissa) {
            (0x7FF, 0) if bits >> 63 == 0 => Some(NonFinite::PositiveInfinity),
            (0x7FF, 0) => Some(NonFinite::NegativeInfinity),
            (0x7FF, _) => Some(NonFinite::NaN),
            _ => None,
        }
    }

    /// A finite value: ordinary, arbitrary finite bits, or special.
    fn finite_value(rng: &mut SplitMix64) -> f64 {
        match rng.below(3) {
            0 => (rng.next_f64() - 0.5) * 16.0,
            1 => loop {
                let x = rng.any_f64();
                if x.is_finite() {
                    break x;
                }
            },
            _ => *rng.choose(&[
                0.0,
                -0.0,
                1.0,
                -2.0,
                f64::MAX,
                f64::MIN,
                f64::MIN_POSITIVE,
                f64::from_bits(1),
            ]),
        }
    }

    /// A domain of every shape: one-sided, two-sided, or everything.
    fn random_domain(rng: &mut SplitMix64) -> Interval {
        let a = finite_value(rng);
        match rng.below(6) {
            0 => Interval::greater_than(a),
            1 => Interval::at_least(a),
            2 => Interval::less_than(a),
            3 => Interval::at_most(a),
            4 => EVERYTHING,
            _ => {
                let b = finite_value(rng);
                let (low, high) = if a <= b { (a, b) } else { (b, a) };
                let lower = *rng.choose(&[Bound::Included(low), Bound::Excluded(low)]);
                let upper = *rng.choose(&[Bound::Included(high), Bound::Excluded(high)]);
                if Interval::is_valid(lower, upper) {
                    Interval::new(lower, upper)
                } else {
                    Interval::new(Bound::Included(low), Bound::Included(high))
                }
            }
        }
    }

    /// A value to validate: anything at all, a special value, or a value at
    /// or next to one of the domain's bounds.
    fn random_value(rng: &mut SplitMix64, domain: Interval) -> f64 {
        let bounds: Vec<f64> = [domain.lower(), domain.upper()]
            .into_iter()
            .filter_map(|bound| match bound {
                Bound::Included(x) | Bound::Excluded(x) => Some(x),
                Bound::Unbounded => None,
            })
            .collect();
        match rng.below(4) {
            0 => rng.any_f64(),
            1 => *rng.choose(&[
                f64::NAN,
                -f64::NAN,
                f64::from_bits(0x7FF0_0000_0000_0001),
                f64::INFINITY,
                f64::NEG_INFINITY,
                0.0,
                -0.0,
                f64::MAX,
                f64::MIN,
            ]),
            _ if bounds.is_empty() => finite_value(rng),
            _ => {
                let bound = *rng.choose(&bounds);
                *rng.choose(&[bound, bound.next_up(), bound.next_down(), -bound])
            }
        }
    }

    /// The exact outcome each validator must produce, built from the policy
    /// text of NP-01 and NP-02: non-finite values are rejected first, then
    /// finite values outside the domain; accepted values are returned
    /// unchanged.
    fn expected(
        name: &'static str,
        value: f64,
        domain: Interval,
        result: bool,
    ) -> Result<u64, ValueError> {
        match (class_from_bits(value), domain.contains(value), result) {
            (Some(class), _, false) => Err(ValueError::NotFinite {
                parameter: name,
                value: class,
            }),
            (Some(class), _, true) => Err(ValueError::ResultNotFinite {
                quantity: name,
                value: class,
            }),
            (None, true, _) => Ok(value.to_bits()),
            (None, false, false) => Err(ValueError::OutOfDomain {
                parameter: name,
                value,
                domain,
            }),
            (None, false, true) => Err(ValueError::ResultOutOfDomain {
                quantity: name,
                value,
                domain,
            }),
        }
    }

    /// Compares an outcome with the expected one, bit for bit: an accepted
    /// value, and the value stored in an error, keep their exact bits,
    /// including the sign of zero.
    fn compare(actual: Result<f64, ValueError>, expected: Result<u64, ValueError>) -> bool {
        match (actual, expected) {
            (Ok(value), Ok(bits)) => value.to_bits() == bits,
            (Err(actual), Err(expected)) => {
                let stored = |error: &ValueError| match error {
                    ValueError::OutOfDomain { value, .. }
                    | ValueError::ResultOutOfDomain { value, .. } => Some(value.to_bits()),
                    ValueError::NotFinite { .. } | ValueError::ResultNotFinite { .. } => None,
                };
                actual == expected && stored(&actual) == stored(&expected)
            }
            _ => false,
        }
    }

    /// NP-01, NP-02, FR-CORE-02: the outcome of every validator is exactly
    /// determined by the value and the domain, for every kind of value and
    /// every shape of domain: the right variant, with the given name, value
    /// and domain, and accepted values unchanged bit for bit.
    #[test]
    fn outcomes_are_exactly_as_specified() {
        const NAMES: [&str; 4] = ["rate", "time", "compound factor", "x"];
        prng::check("validator outcomes", 0x5AB1_0002, 50_000, |rng| {
            let domain = random_domain(rng);
            let value = random_value(rng, domain);
            let name = *rng.choose(&NAMES);
            let unbounded = EVERYTHING;
            let outcomes = [
                (finite(name, value), expected(name, value, unbounded, false)),
                (
                    within(name, value, domain),
                    expected(name, value, domain, false),
                ),
                (
                    finite_result(name, value),
                    expected(name, value, unbounded, true),
                ),
                (
                    result_within(name, value, domain),
                    expected(name, value, domain, true),
                ),
            ];
            for (index, (actual, wanted)) in outcomes.into_iter().enumerate() {
                let shown = format!("{actual:?}");
                if !compare(actual, wanted.clone()) {
                    return Err(format!(
                        "validator {index} on {value:?} ({:#018x}) in {domain}: got {shown}, want {wanted:?}",
                        value.to_bits()
                    ));
                }
            }
            Ok(())
        });
    }

    /// The neighbors of a bound fall on the side that the kind of bound
    /// prescribes: inclusive bounds accept the bound, exclusive ones reject
    /// it, and the next representable values on either side follow.
    #[test]
    fn bound_neighbors_fall_on_the_right_side() {
        prng::check("bound neighbors", 0x5AB1_0003, 20_000, |rng| {
            let bound = finite_value(rng);
            let (up, down) = (bound.next_up(), bound.next_down());
            let accepts = |domain: Interval, value: f64| within("x", value, domain).is_ok();
            let cases = [
                (Interval::greater_than(bound), [false, true, false]),
                (Interval::at_least(bound), [true, true, false]),
                (Interval::less_than(bound), [false, false, true]),
                (Interval::at_most(bound), [true, false, true]),
            ];
            for (domain, [at, above, below]) in cases {
                // A neighbor beyond the largest finite value is infinite,
                // and so never accepted.
                let wanted = [at, above && up.is_finite(), below && down.is_finite()];
                let actual = [
                    accepts(domain, bound),
                    accepts(domain, up),
                    accepts(domain, down),
                ];
                if actual != wanted {
                    return Err(format!(
                        "{domain} at {bound:?}: accepts [bound, up, down] = {actual:?}, want {wanted:?}"
                    ));
                }
            }
            Ok(())
        });
    }

    /// NP-08: the sign of zero is never changed, neither in an accepted value
    /// nor in the value that an error reports.
    #[test]
    fn the_sign_of_zero_is_preserved() {
        let accepted = within("time", -0.0, Interval::at_least(0.0)).expect("in domain");
        assert_eq!(accepted.to_bits(), (-0.0_f64).to_bits());
        let error = result_within("compound factor", -0.0, Interval::greater_than(0.0))
            .expect_err("zero is not positive");
        match error {
            ValueError::ResultOutOfDomain { value, .. } => {
                assert_eq!(value.to_bits(), (-0.0_f64).to_bits());
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    /// NFR-03 (level R1), NFR-05: validation depends only on its arguments.
    /// The same inputs give identical outcomes, bit for bit, on four threads
    /// at once and on the calling thread.
    #[test]
    fn validation_is_deterministic_across_threads() {
        let mut rng = SplitMix64::new(0x5AB1_0004);
        let inputs: Vec<(f64, Interval)> = (0..10_000)
            .map(|_| {
                let domain = random_domain(&mut rng);
                (random_value(&mut rng, domain), domain)
            })
            .collect();
        let run = |inputs: &[(f64, Interval)]| -> Vec<String> {
            inputs
                .iter()
                .map(|&(value, domain)| {
                    let outcome = result_within("x", value, domain);
                    format!("{outcome:?} {:?}", outcome.as_ref().map(|v| v.to_bits()))
                })
                .collect()
        };
        let sequential = run(&inputs);
        let parallel: Vec<Vec<String>> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4).map(|_| scope.spawn(|| run(&inputs))).collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("no thread panics"))
                .collect()
        });
        for outcome in parallel {
            assert_eq!(outcome, sequential);
        }
    }
}
