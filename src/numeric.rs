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
    use crate::support::prng;
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
}
