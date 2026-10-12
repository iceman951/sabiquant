//! FS-RATE/FS-DISC range bands, exact rational ties and Simple exceptions.

mod support;

use sabiquant::daycount::DayCount;
use sabiquant::rates::{Compounding, CompoundingMethod, InterestRate, RateError};
use std::collections::{BTreeMap, BTreeSet};
use support::fixture::{Fixture, Record};
use support::float::{Tolerance, ulp_distance};

#[expect(
    clippy::unwrap_used,
    reason = "committed fixture validity is a prerequisite of every consumer test"
)]
fn data() -> Fixture {
    Fixture::load(Fixture::repository_path(
        "validation/fixtures/rates/boundaries-oracle.fixture",
    ))
    .unwrap()
}

#[expect(
    clippy::unwrap_used,
    reason = "fixture scalar and frequency decoding must fail immediately on malformed test data"
)]
fn compounding(row: &Record<'_>) -> Compounding {
    match row.parse::<CompoundingMethod>("method").unwrap() {
        CompoundingMethod::Simple => Compounding::SIMPLE,
        CompoundingMethod::Continuous => Compounding::CONTINUOUS,
        CompoundingMethod::Compounded => {
            Compounding::compounded(row.parse("frequency").unwrap()).unwrap()
        }
        CompoundingMethod::SimpleThenCompounded => {
            Compounding::simple_then_compounded(row.parse("frequency").unwrap()).unwrap()
        }
        CompoundingMethod::CompoundedThenSimple => {
            Compounding::compounded_then_simple(row.parse("frequency").unwrap()).unwrap()
        }
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "fixture inputs are validated by the parser and generator; production errors remain propagated"
)]
fn execute(row: &Record<'_>) -> Result<f64, RateError> {
    let quote = InterestRate::new(
        row.real("rate").unwrap(),
        DayCount::Actual365Fixed,
        compounding(row),
    )?;
    let time = row.real("time").unwrap();
    if row.text("operation").unwrap() == "compound" {
        quote.compound_factor(time)
    } else {
        quote.discount_factor(time)
    }
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "records measured native E3 boundary-tier evidence with --nocapture"
)]
fn independent_boundary_tier_has_exact_policy_statuses_and_e3_finite_factors() {
    let fixture = data();
    assert_eq!(fixture.record_count(), 14_068);
    assert_eq!(
        fixture.header("reference"),
        Some("oracle:Python Decimal and Fraction")
    );
    assert!(
        fixture
            .header("reproduction")
            .unwrap()
            .contains("rate-oracles-sha256=")
    );
    let mut counts = BTreeMap::new();
    let mut configurations = BTreeSet::new();
    let mut hybrid_phases = BTreeSet::new();
    let mut maximum_normalized = 0.0_f64;
    let mut maximum_ulps = 0;
    let mut subnormal_values = 0;
    let mut exact_simple_values = 0;
    for row in fixture.records() {
        let method = row.text("method").unwrap();
        let frequency = row.text("frequency").unwrap();
        let phase = row.text("phase").unwrap();
        configurations.insert((method, frequency));
        if method == "SimpleThenCompounded" || method == "CompoundedThenSimple" {
            hybrid_phases.insert((method, frequency, phase));
        }
        let actual = execute(&row);
        if row.text("behavior").unwrap() == "error" {
            let kind = row.text("error_kind").unwrap();
            *counts.entry(kind).or_insert(0) += 1;
            let error = actual.expect_err(row.id());
            assert_eq!(error.kind(), kind, "{}: {error}", row.id());
            if kind == "rate.numerical_boundary" {
                assert_ne!(phase, "Simple", "{}: fused Simple is exempt", row.id());
                assert!(matches!(error, RateError::NumericalBoundary { .. }));
            }
            continue;
        }
        *counts.entry("value").or_insert(0) += 1;
        assert_eq!(row.text("tolerance").unwrap(), "TOL-E3");
        let actual = actual.unwrap_or_else(|error| panic!("{}: {error}", row.id()));
        let expected = row.real("expected").unwrap();
        assert!(actual.is_finite() && actual > 0.0);
        maximum_ulps = maximum_ulps.max(ulp_distance(actual, expected).unwrap());
        let scale = (f64::EPSILON / 2.0) * (1.0 + expected.ln().abs());
        let absolute = if expected.is_subnormal() {
            subnormal_values += 1;
            f64::from_bits(1)
        } else {
            maximum_normalized =
                maximum_normalized.max((actual - expected).abs() / expected / scale);
            0.0
        };
        Tolerance::Mixed {
            absolute,
            relative: 8.0 * scale,
        }
        .check(actual, expected)
        .unwrap_or_else(|error| panic!("{}: {error}", row.id()));
        if phase == "Simple"
            && row.text("origin").unwrap() == "Python Fraction exact binary64 arithmetic"
        {
            exact_simple_values += 1;
            assert_eq!(
                actual.to_bits(),
                expected.to_bits(),
                "{}: correctly rounded fused arithmetic and reciprocal",
                row.id()
            );
        }
    }
    assert_eq!(configurations.len(), 32);
    assert_eq!(hybrid_phases.len(), 40);
    assert_eq!(
        counts,
        BTreeMap::from([
            ("rate.numerical_boundary", 2542),
            ("rate.value", 5518),
            ("value", 6008)
        ])
    );
    assert!(subnormal_values > 0);
    assert!(exact_simple_values > 0);
    println!(
        "boundary E3: max_normalized={maximum_normalized}; max_ulps={maximum_ulps}; subnormal_values={subnormal_values}; exact_simple_values={exact_simple_values}; counts={counts:?}"
    );
}

/// Independently builds the decimal integer 2^power with small digit carries.
fn decimal_power_of_two(power: u32) -> String {
    let mut digits = vec![1_u8];
    for _ in 0..power {
        let mut carry = 0;
        for digit in &mut digits {
            let doubled = *digit * 2 + carry;
            *digit = doubled % 10;
            carry = doubled / 10;
        }
        if carry != 0 {
            digits.push(carry);
        }
    }
    digits
        .into_iter()
        .rev()
        .map(|digit| char::from(b'0' + digit))
        .collect()
}

/// Constructs a finite exact binary64 power directly from exponent/significand.
#[expect(
    clippy::unwrap_used,
    reason = "callers restrict exponents to the binary64 representable range before conversion"
)]
fn binary_power_of_two(power: i32) -> f64 {
    if power >= -1022 {
        f64::from_bits(u64::try_from(power + 1023).unwrap() << 52)
    } else {
        f64::from_bits(1_u64 << u32::try_from(power + 1074).unwrap())
    }
}

#[test]
fn rational_power_anchors_resolve_overflow_and_halfway_zero_without_decimal_choice() {
    let fixture = data();
    let mut anchors = 0;
    let mut zero_ties = 0;
    let mut definite_overflows = 0;
    for row in fixture
        .records()
        .filter(|row| row.text("category").unwrap() == "exact-power2")
    {
        anchors += 1;
        let power = row.parse::<i32>("boundary").unwrap();
        let magnitude = power.unsigned_abs();
        let integer = decimal_power_of_two(magnitude);
        let (numerator, denominator) = if power < 0 {
            ("1", integer.as_str())
        } else {
            (integer.as_str(), "1")
        };
        assert_eq!(row.text("exact_compound_numerator").unwrap(), numerator);
        assert_eq!(row.text("exact_compound_denominator").unwrap(), denominator);
        let rate = row.real("rate").unwrap();
        let time = row.real("time").unwrap();
        assert_eq!(
            rate.to_bits(),
            if power < 0 {
                (-0.5_f64).to_bits()
            } else {
                1.0_f64.to_bits()
            }
        );
        assert_eq!(time.to_bits(), f64::from(magnitude).to_bits());
        let compound_representable = (-1074..=1023).contains(&power);
        let discount = row.text("operation").unwrap() == "discount";
        let expected_power = if discount { -power } else { power };
        let representable = compound_representable && (-1074..=1023).contains(&expected_power);
        if representable {
            assert_eq!(row.text("mathematical_behavior").unwrap(), "value");
            assert_eq!(
                row.real("mathematical_expected").unwrap().to_bits(),
                binary_power_of_two(expected_power).to_bits()
            );
        } else {
            assert_eq!(row.text("mathematical_behavior").unwrap(), "error");
            assert!(row.text("mathematical_expected").unwrap().is_empty());
        }
        if power == -1075 {
            zero_ties += 1;
            assert_eq!(row.text("behavior").unwrap(), "error");
            assert_eq!(row.text("error_kind").unwrap(), "rate.numerical_boundary");
        }
        if power == 1024 {
            definite_overflows += 1;
            assert_eq!(row.text("error_kind").unwrap(), "rate.numerical_boundary");
        }
    }
    assert_eq!(anchors, 40);
    assert_eq!(zero_ties, 4);
    assert_eq!(definite_overflows, 4);
}

fn shifted(mut value: f64, offset: i32) -> f64 {
    for _ in 0..offset.unsigned_abs() {
        value = if offset < 0 {
            value.next_down()
        } else {
            value.next_up()
        };
    }
    value
}

#[test]
fn every_adjacent_continuous_argument_through_both_range_bands_obeys_explicit_policy() {
    for boundary in [
        f64::MAX.ln(),
        f64::from_bits(1).ln() - std::f64::consts::LN_2,
    ] {
        let rate = if boundary < 0.0 { -1.0 } else { 1.0 };
        let quote =
            InterestRate::new(rate, DayCount::Actual365Fixed, Compounding::CONTINUOUS).unwrap();
        let mut guarded = 0;
        let mut finite = 0;
        let mut out_of_range = 0;
        for offset in -64..=64 {
            let logarithm = shifted(boundary, offset);
            let allowance = 8.0 * (f64::EPSILON / 2.0) * (1.0 + logarithm.abs());
            let result = quote.compound_factor(logarithm.abs());
            if (logarithm - boundary).abs() <= allowance {
                guarded += 1;
                let error = result.expect_err("declared binary64 band");
                assert_eq!(error.kind(), "rate.numerical_boundary");
                if let RateError::NumericalBoundary {
                    quantity,
                    logarithm: actual,
                    ..
                } = error
                {
                    assert_eq!(quantity, "compound factor");
                    assert_eq!(actual.to_bits(), logarithm.to_bits());
                }
            } else if boundary > 0.0 && logarithm < boundary
                || boundary < 0.0 && logarithm > boundary
            {
                finite += 1;
                assert!(result.unwrap() > 0.0);
            } else {
                out_of_range += 1;
                assert_eq!(result.unwrap_err().kind(), "rate.value");
            }
        }
        assert!(guarded > 0 && finite > 0 && out_of_range > 0);
    }
}

#[test]
fn reciprocal_band_classifies_the_actual_positive_factor_and_one_binary64_reciprocal() {
    let quote = InterestRate::new(-1.0, DayCount::Actual365Fixed, Compounding::CONTINUOUS).unwrap();
    let boundary = f64::MAX.ln();
    let mut guarded = 0;
    let mut finite = 0;
    let mut overflow = 0;
    for offset in -64..=64 {
        let time = shifted(boundary, offset);
        let compound = quote.compound_factor(time).unwrap();
        let logarithm = -compound.ln();
        let allowance = 8.0 * (f64::EPSILON / 2.0) * (1.0 + logarithm.abs());
        let result = quote.discount_factor(time);
        if (logarithm - boundary).abs() <= allowance {
            guarded += 1;
            let error = result.unwrap_err();
            assert_eq!(error.kind(), "rate.numerical_boundary");
            assert!(matches!(
                error,
                RateError::NumericalBoundary {
                    quantity: "discount factor",
                    ..
                }
            ));
        } else {
            let reciprocal = 1.0 / compound;
            if reciprocal.is_finite() {
                finite += 1;
                assert_eq!(result.unwrap().to_bits(), reciprocal.to_bits());
            } else {
                overflow += 1;
                assert_eq!(result.unwrap_err().kind(), "rate.value");
            }
        }
    }
    assert!(guarded > 0 && finite > 0 && overflow > 0);
}

#[test]
fn fused_simple_boundaries_and_definite_infinite_exponents_have_distinct_errors() {
    let quote = InterestRate::new(f64::MAX, DayCount::Actual365Fixed, Compounding::SIMPLE).unwrap();
    assert_eq!(
        quote.compound_factor(1.0).unwrap().to_bits(),
        f64::MAX.to_bits()
    );
    assert_eq!(
        quote.discount_factor(1.0).unwrap().to_bits(),
        (1.0 / f64::MAX).to_bits()
    );
    let cancellation = InterestRate::new(
        -1.0 - f64::EPSILON,
        DayCount::Actual365Fixed,
        Compounding::SIMPLE,
    )
    .unwrap();
    assert_eq!(
        cancellation
            .compound_factor(1.0 - f64::EPSILON)
            .unwrap()
            .to_bits(),
        binary_power_of_two(-104).to_bits()
    );
    for rate in [-f64::MAX, f64::MAX] {
        let quote =
            InterestRate::new(rate, DayCount::Actual365Fixed, Compounding::CONTINUOUS).unwrap();
        assert_eq!(
            quote.compound_factor(f64::MAX).unwrap_err().kind(),
            "rate.value"
        );
        assert_eq!(
            quote.discount_factor(f64::MAX).unwrap_err().kind(),
            "rate.value"
        );
    }
}
