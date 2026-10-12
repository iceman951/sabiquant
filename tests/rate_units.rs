//! Explicit rate units and rejection of lost nonzero values (NFR-14).

use sabiquant::daycount::DayCount;
use sabiquant::rates::{
    Compounding, InterestRate, RateError, basis_points_from_decimal, decimal_from_basis_points,
    decimal_from_percent, percent_from_decimal,
};

#[test]
fn percent_and_basis_points_construct_the_same_explicit_decimal_quote() {
    let percent = decimal_from_percent(5.0).unwrap();
    let basis_points = decimal_from_basis_points(500.0).unwrap();
    assert_eq!(percent.to_bits(), 0.05_f64.to_bits());
    assert_eq!(basis_points.to_bits(), percent.to_bits());
    let quote = InterestRate::new(percent, DayCount::Actual365Fixed, Compounding::SIMPLE).unwrap();
    assert_eq!(
        quote.compound_factor(2.0).unwrap().to_bits(),
        1.1_f64.to_bits()
    );
    assert_eq!(
        percent_from_decimal(basis_points).unwrap().to_bits(),
        5.0_f64.to_bits()
    );
    assert_eq!(
        basis_points_from_decimal(percent).unwrap().to_bits(),
        500.0_f64.to_bits()
    );
    // The conversion has no compounding domain: a negative quote is still
    // validated separately by InterestRate and its requested horizon.
    assert_eq!(
        decimal_from_percent(-100.0).unwrap().to_bits(),
        (-1.0_f64).to_bits()
    );
    assert_eq!(
        decimal_from_basis_points(-10_000.0).unwrap().to_bits(),
        (-1.0_f64).to_bits()
    );
}

#[test]
fn unit_conversions_preserve_sign_and_canonicalize_zero_without_losing_nonzero_inputs() {
    for convert in [
        decimal_from_percent,
        decimal_from_basis_points,
        percent_from_decimal,
        basis_points_from_decimal,
    ] {
        for zero in [0.0, -0.0] {
            assert_eq!(convert(zero).unwrap().to_bits(), 0);
        }
        for nonfinite in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(convert(nonfinite), Err(RateError::Value(_))));
        }
        assert!(convert(-0.25).unwrap() < 0.0);
        assert!(convert(0.25).unwrap() > 0.0);
    }
    for convert in [decimal_from_percent, decimal_from_basis_points] {
        for sign in [-1.0, 1.0] {
            assert!(matches!(
                convert(sign * f64::from_bits(1)),
                Err(RateError::Value(_))
            ));
        }
    }
    for convert in [percent_from_decimal, basis_points_from_decimal] {
        for sign in [-1.0, 1.0] {
            assert!(matches!(convert(sign * f64::MAX), Err(RateError::Value(_))));
            assert!(convert(sign * f64::from_bits(1)).unwrap() * sign > 0.0);
        }
    }
}
