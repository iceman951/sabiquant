//! Public finite-input numerical-support and extrapolation boundaries.

mod support;

use sabiquant::error::{NonFinite, ValueError};
use sabiquant::math::{Extrapolation, InterpolationError, InterpolationMethod, Interpolator};
use std::error::Error;
use std::ops::Bound;
use support::float::{Tolerance, assert_close};

const MIN_SUB: f64 = f64::from_bits(1);
const CUBIC: [InterpolationMethod; 2] = [
    InterpolationMethod::NaturalCubic,
    InterpolationMethod::MonotoneCubicHermite,
];

#[expect(
    clippy::panic,
    reason = "test-only public error assertion reports a mismatched variant with its full typed context"
)]
fn grid_error<'a>(
    error: &'a InterpolationError,
    expected_interval: usize,
    expected_quantity: &str,
) -> &'a ValueError {
    let InterpolationError::NumericalGrid {
        interval,
        quantity,
        cause,
        ..
    } = error
    else {
        panic!("expected numerical support failure, got {error:?}");
    };
    assert_eq!(*interval, expected_interval);
    assert_eq!(*quantity, expected_quantity);
    assert_eq!(error.kind(), "interpolation.numerical_grid");
    assert_eq!(error.clone(), *error);
    assert_eq!(
        error
            .source()
            .and_then(|cause| cause.downcast_ref::<ValueError>()),
        Some(cause)
    );
    assert!(!error.to_string().is_empty());
    cause
}

#[test]
fn finite_cubic_definitions_reject_lost_or_subnormal_normalized_input_ratios() {
    // Relative input precision is the support boundary: a physical derivative
    // is deliberately not formed. Every node here is finite and ordered.
    for method in CUBIC {
        for (nodes, quantity, subnormal) in [
            (
                vec![(0.0, 0.0), (f64::MIN_POSITIVE * 0.5, 1.0), (1.0, 2.0)],
                "normalized interval width",
                true,
            ),
            (
                vec![(0.0, 0.0), (MIN_SUB, 1.0), (f64::MAX, 2.0)],
                "normalized interval width",
                false,
            ),
            (
                vec![(0.0, 0.0), (1.0, MIN_SUB), (2.0, 1.0)],
                "normalized ordinate difference",
                true,
            ),
            (
                vec![(0.0, 0.0), (1.0, MIN_SUB), (2.0, f64::MAX)],
                "normalized ordinate-difference magnitude",
                false,
            ),
        ] {
            let error = Interpolator::new(nodes, method, Extrapolation::Reject).unwrap_err();
            let ValueError::ResultOutOfDomain { value, domain, .. } =
                grid_error(&error, 0, quantity)
            else {
                panic!("expected relative-precision domain failure: {error:?}");
            };
            assert_eq!(value.is_subnormal(), subnormal);
            assert!(!domain.contains(*value));
            assert_eq!(domain.upper(), Bound::Unbounded);
            if subnormal {
                assert_eq!(domain.lower(), Bound::Included(f64::MIN_POSITIVE));
            } else {
                assert_eq!(value.to_bits(), 0);
                assert_eq!(domain.lower(), Bound::Excluded(0.0));
            }
        }
        let f = Interpolator::new(
            vec![
                (0.0, 0.0),
                (f64::MIN_POSITIVE, f64::MIN_POSITIVE),
                (1.0, 1.0),
            ],
            method,
            Extrapolation::Reject,
        )
        .unwrap();
        assert_eq!(
            f.value(f64::MIN_POSITIVE / 2.0).unwrap().to_bits(),
            (f64::MIN_POSITIVE / 2.0).to_bits()
        );
    }
}

#[test]
fn overflowing_ordinate_differences_are_distinct_from_invalid_input_nodes() {
    for method in CUBIC {
        for sign in [-1.0, 1.0] {
            let nodes = vec![(0.0, -sign * f64::MAX), (1.0, sign * f64::MAX), (2.0, 0.0)];
            let error = Interpolator::new(nodes, method, Extrapolation::Reject).unwrap_err();
            assert!(matches!(
                grid_error(&error, 0, "ordinate difference"),
                ValueError::ResultNotFinite { value, .. }
                    if *value == if sign.is_sign_positive() {
                        NonFinite::PositiveInfinity
                    } else {
                        NonFinite::NegativeInfinity
                    }
            ));
        }
    }
}

#[test]
fn finite_natural_systems_report_rhs_overflow_and_required_curvature_loss() {
    let narrow = f64::MIN_POSITIVE;
    let error = Interpolator::new(
        vec![(0.0, 0.0), (narrow, 1.0), (1.0, 0.0)],
        InterpolationMethod::NaturalCubic,
        Extrapolation::Reject,
    )
    .unwrap_err();
    assert!(matches!(
        grid_error(&error, 1, "spline right-hand side"),
        ValueError::ResultNotFinite {
            value: NonFinite::NegativeInfinity,
            ..
        }
    ));

    let error = Interpolator::new(
        vec![(0.0, 0.0), (narrow, 0.0), (1.0, 1.0)],
        InterpolationMethod::NaturalCubic,
        Extrapolation::Reject,
    )
    .unwrap_err();
    assert!(matches!(
        grid_error(&error, 0, "scaled right-curvature magnitude"),
        ValueError::ResultOutOfDomain { value, .. } if value.to_bits() == 0
    ));

    // The same plateau is a different mathematical method and has no required
    // natural curvature contribution. PCHIP legitimately keeps it constant.
    let monotone = Interpolator::new(
        vec![(0.0, 0.0), (narrow, 0.0), (1.0, 1.0)],
        InterpolationMethod::MonotoneCubicHermite,
        Extrapolation::Reject,
    )
    .unwrap();
    assert_eq!(monotone.value(narrow / 2.0).unwrap().to_bits(), 0);
    assert!(monotone.value(0.5).unwrap() > 0.0);
}

#[test]
fn relative_precision_failures_inside_solves_slopes_and_coefficients_remain_typed() {
    let small = f64::MIN_POSITIVE;
    for (method, quantity, interval) in [
        (
            InterpolationMethod::NaturalCubic,
            "spline forward coefficient",
            1,
        ),
        (
            InterpolationMethod::MonotoneCubicHermite,
            "endpoint slope",
            0,
        ),
    ] {
        let error = Interpolator::new(
            vec![(0.0, 0.0), (1.0, small), (2.0, 2.5 * small), (3.0, 1.0)],
            method,
            Extrapolation::Reject,
        )
        .unwrap_err();
        assert!(matches!(
            grid_error(&error, interval, quantity),
            ValueError::ResultOutOfDomain { value, .. } if value.is_subnormal()
        ));
    }

    let error = Interpolator::new(
        vec![(0.0, 0.0), (small, small), (1.0, 0.5)],
        InterpolationMethod::MonotoneCubicHermite,
        Extrapolation::Reject,
    )
    .unwrap_err();
    assert!(matches!(
        grid_error(&error, 0, "normalized cubic coefficient"),
        ValueError::ResultOutOfDomain { value, .. } if value.is_subnormal()
    ));
}

#[test]
fn finite_extreme_queries_obey_each_policy_before_coordinate_evaluation() {
    for method in InterpolationMethod::ALL {
        for policy in Extrapolation::ALL {
            let f = Interpolator::new(vec![(0.0, 1.0), (MIN_SUB, 2.0)], method, policy).unwrap();
            assert_eq!(f.value(0.0).unwrap().to_bits(), 1.0_f64.to_bits());
            assert_eq!(f.value(MIN_SUB).unwrap().to_bits(), 2.0_f64.to_bits());
            for query in [-f64::MAX, f64::MAX] {
                match policy {
                    Extrapolation::Flat => assert_eq!(
                        f.value(query).unwrap().to_bits(),
                        if query.is_sign_positive() {
                            2.0_f64
                        } else {
                            1.0_f64
                        }
                        .to_bits()
                    ),
                    Extrapolation::Reject => assert!(matches!(
                        f.value(query),
                        Err(InterpolationError::OutsideDomain { query: actual, lower, upper, .. })
                            if actual.to_bits() == query.to_bits()
                                && lower.to_bits() == 0 && upper.to_bits() == MIN_SUB.to_bits()
                    )),
                    Extrapolation::ExtendBoundarySegment => {
                        let error = f.value(query).unwrap_err();
                        assert!(matches!(
                            grid_error(&error, 0, "relative interpolation coordinate"),
                            ValueError::ResultNotFinite { value, .. }
                                if *value == if query.is_sign_positive() {
                                    NonFinite::PositiveInfinity
                                } else {
                                    NonFinite::NegativeInfinity
                                }
                        ));
                    }
                    _ => panic!("unhandled extrapolation policy"),
                }
            }
        }
    }

    for method in CUBIC {
        // Three non-affine nodes exercise the actual cubic query path;
        // a finite extreme query cannot form a finite relative coordinate.
        let h = f64::from_bits(23_u64 << 52); // exactly 2^-1000
        let f = Interpolator::new(
            vec![(0.0, 0.0), (h, 1.0), (2.0 * h, 0.0)],
            method,
            Extrapolation::ExtendBoundarySegment,
        )
        .unwrap();
        let error = f.value(f64::MAX).unwrap_err();
        assert!(matches!(
            grid_error(&error, 1, "relative interpolation coordinate"),
            ValueError::ResultNotFinite {
                value: NonFinite::PositiveInfinity,
                ..
            }
        ));
    }

    for method in [InterpolationMethod::Linear, CUBIC[0], CUBIC[1]] {
        let f = Interpolator::new(
            vec![(0.0, -f64::MAX), (MIN_SUB, f64::MAX)],
            method,
            Extrapolation::ExtendBoundarySegment,
        )
        .unwrap();
        // Exact knots need no intermediate. The initial centered width rounds
        // to zero; ordinate rescaling recovers the affine computation, whose
        // extrapolated output 3*MAX is genuinely outside the rounded range.
        assert_eq!(f.value(0.0).unwrap().to_bits(), (-f64::MAX).to_bits());
        assert_eq!(f.value(MIN_SUB).unwrap().to_bits(), f64::MAX.to_bits());
        assert!(matches!(
            f.value(2.0 * MIN_SUB),
            Err(InterpolationError::Value(ValueError::ResultNotFinite {
                quantity: "interpolated value",
                value: NonFinite::PositiveInfinity,
                ..
            }))
        ));
    }
}

#[test]
fn centered_extrapolation_recovers_finite_identity_after_both_distances_overflow() {
    for method in [InterpolationMethod::Linear, CUBIC[0], CUBIC[1]] {
        for sign in [-1.0, 1.0] {
            let f = Interpolator::new(
                vec![(-f64::MAX, -sign * f64::MAX), (0.0, 0.0)],
                method,
                Extrapolation::ExtendBoundarySegment,
            )
            .unwrap();
            assert_eq!(
                f.value(f64::MAX).unwrap().to_bits(),
                (sign * f64::MAX).to_bits()
            );
        }
    }
}

#[test]
fn signed_cubic_and_logarithmic_extrapolation_have_separate_output_domains() {
    for (method, expected) in [
        (
            InterpolationMethod::NaturalCubic,
            NonFinite::NegativeInfinity,
        ),
        (
            InterpolationMethod::MonotoneCubicHermite,
            NonFinite::PositiveInfinity,
        ),
    ] {
        let f = Interpolator::new(
            vec![(0.0, 1.0), (1.0, 0.0), (2.0, 1.0)],
            method,
            Extrapolation::ExtendBoundarySegment,
        )
        .unwrap();
        assert!(matches!(
            f.value(f64::MAX),
            Err(InterpolationError::Value(ValueError::ResultNotFinite {
                quantity: "interpolated value", value, ..
            })) if value == expected
        ));
    }

    let log = Interpolator::new(
        vec![(0.0, 1.0), (1.0, 2.0)],
        InterpolationMethod::LogLinear,
        Extrapolation::ExtendBoundarySegment,
    )
    .unwrap();
    assert!(matches!(
        log.value(2048.0),
        Err(InterpolationError::Value(ValueError::ResultNotFinite {
            quantity: "interpolated value",
            value: NonFinite::PositiveInfinity,
            ..
        }))
    ));
    assert!(matches!(
        log.value(-2048.0),
        Err(InterpolationError::Value(ValueError::ResultOutOfDomain {
            quantity: "interpolated value", value, ..
        })) if value.to_bits() == 0
    ));
}

#[test]
fn rounded_finite_range_validation_is_not_an_exact_real_extrapolation_certificate() {
    let f = Interpolator::new(
        vec![(0.0, f64::MAX), (1.0, f64::MAX / 2.0)],
        InterpolationMethod::Linear,
        Extrapolation::ExtendBoundarySegment,
    )
    .unwrap();
    // The exact real value is MAX+(MAX/2)*MIN_SUB. Its increment is tiny
    // compared with an ordinate ULP: a finite correctly rounded MAX is valid.
    assert_eq!(f.value(-MIN_SUB).unwrap().to_bits(), f64::MAX.to_bits());
    assert!(matches!(
        f.value(-1.0),
        Err(InterpolationError::Value(ValueError::ResultNotFinite {
            quantity: "interpolated value",
            value: NonFinite::PositiveInfinity,
            ..
        }))
    ));
}

#[test]
fn subnormal_scaled_cubic_outputs_round_at_evaluation_and_preserve_exact_nodes()
-> Result<(), Box<dyn Error>> {
    // This previously had an unrepresentable physical secant and curvature.
    // The normalized node definition is affine for both cubic methods.
    for method in CUBIC {
        let f = Interpolator::new(
            vec![
                (0.0, 0.0),
                (f64::MAX / 2.0, MIN_SUB),
                (f64::MAX, 2.0 * MIN_SUB),
            ],
            method,
            Extrapolation::Reject,
        )?;
        assert_eq!(f.value(f64::MAX / 2.0)?.to_bits(), MIN_SUB.to_bits());
        assert_eq!(f.value(f64::MAX)?.to_bits(), (2.0 * MIN_SUB).to_bits());
        assert_close(
            f.value(f64::MAX / 4.0)?,
            0.0,
            Tolerance::Mixed {
                absolute: MIN_SUB,
                relative: 0.0,
            },
        );
    }
    Ok(())
}
