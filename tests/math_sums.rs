//! Independent exact-rational and reduction-contract checks for M6 sums.

mod support;

use sabiquant::error::{Error, NonFinite, ValueError};
use sabiquant::math::{SumError, compensated_sum, exact_sum};
use std::error::Error as StdError;
use support::fixture::Fixture;
use support::prng::SplitMix64;

#[test]
#[expect(
    clippy::print_stdout,
    reason = "records the independent oracle calibration counts under --nocapture"
)]
fn exact_reduction_matches_all_fraction_oracles_bit_for_bit() {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/math/sum-oracle.fixture",
    ))
    .unwrap();
    assert_eq!(fixture.record_count(), 105_307);
    let mut compensated_values = 0;
    let mut compensated_intermediate_errors = 0;
    let mut compensated_normalized = Vec::new();
    for row in fixture.records() {
        let input = row.text("input_bits").unwrap();
        let values: Vec<f64> = if input.is_empty() {
            Vec::new()
        } else {
            input
                .split(';')
                .map(|text| f64::from_bits(u64::from_str_radix(text, 16).unwrap()))
                .collect()
        };
        // These finite-final-sum counterexamples have independently verified
        // overflowing exact prefixes. The diagnostic is checked per row,
        // rather than allowing an arbitrary seven errors elsewhere.
        let partial_failure = match row.id() {
            "SUM-000003" | "SUM-000008" | "SUM-102299" => Some((1, NonFinite::PositiveInfinity)),
            "SUM-000005" | "SUM-102294" => Some((1, NonFinite::NegativeInfinity)),
            "SUM-105302" => Some((2097, NonFinite::PositiveInfinity)),
            "SUM-105305" => Some((53, NonFinite::PositiveInfinity)),
            _ => None,
        };
        let exact = exact_sum(&values);
        if row.text("behavior").unwrap() == "error" {
            let error = exact.expect_err(row.id());
            assert_eq!(
                error.kind(),
                row.text("error_kind").unwrap(),
                "{}",
                row.id()
            );
            assert!(matches!(&error, SumError::Arithmetic { index: None, .. }));
            let class = match row.text("overflow_sign").unwrap() {
                "positive" => NonFinite::PositiveInfinity,
                "negative" => NonFinite::NegativeInfinity,
                other => panic!("invalid exact overflow sign {other}"),
            };
            assert!(
                matches!(&error, SumError::Arithmetic { index: None,
                error: ValueError::ResultNotFinite { quantity: "exact sum", value, .. }, ..
            } if *value == class),
                "{}: {error}",
                row.id()
            );
        } else {
            let expected = row.real_bits("expected").unwrap();
            assert_eq!(exact.unwrap().to_bits(), expected.to_bits(), "{}", row.id());
            match compensated_sum(&values) {
                Ok(actual) => {
                    assert!(
                        partial_failure.is_none(),
                        "{} must reject its overflowing prefix",
                        row.id()
                    );
                    compensated_values += 1;
                    let budget = row.real("compensated_absolute_budget").unwrap();
                    assert!(
                        budget.is_finite() && budget > 0.0,
                        "{} budget must be informative",
                        row.id()
                    );
                    assert!(
                        (actual - expected).abs() <= budget,
                        "{}: {actual:?} vs {expected:?}, budget {budget:?}",
                        row.id()
                    );
                    compensated_normalized.push((actual - expected).abs() / budget);
                }
                Err(error @ SumError::Arithmetic { .. }) => {
                    compensated_intermediate_errors += 1;
                    let (expected_index, expected_class) =
                        partial_failure.expect("only the proved overflowing prefixes may fail");
                    assert!(
                        matches!(error, SumError::Arithmetic {
                        index: Some(index), error: ValueError::ResultNotFinite {
                            quantity: "partial sum", value, .. }, ..
                    } if index == expected_index && value == expected_class),
                        "{}: {error:?}",
                        row.id()
                    );
                    let message = error.to_string();
                    assert!(
                        message.contains(&expected_index.to_string())
                            && message.contains("partial sum"),
                        "{} contextual message",
                        row.id()
                    );
                    assert!(error.source().unwrap().is::<ValueError>());
                }
                Err(other) => panic!("{}: unexpected {other}", row.id()),
            }
        }
    }
    assert_eq!(compensated_values, 105_288);
    assert_eq!(compensated_intermediate_errors, 7);
    compensated_normalized.sort_by(f64::total_cmp);
    let n = compensated_normalized.len();
    println!(
        "sum oracle: {} exact records (finite errors 0 ULP); {compensated_values} conditioned compensated values; {compensated_intermediate_errors} finite-exact intermediate overflows; compensated normalized p50={} p95={} p99={} max={}",
        fixture.record_count(),
        compensated_normalized[n / 2],
        compensated_normalized[n * 95 / 100],
        compensated_normalized[n * 99 / 100],
        compensated_normalized[n - 1]
    );
}

#[test]
fn exact_sum_is_permutation_invariant_even_after_overflowing_prefixes() {
    let tiny = f64::from_bits(1);
    let mut rng = SplitMix64::new(0x6e_0006);
    for tail in [
        tiny,
        -tiny,
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        0.1,
        -0.1,
        1.0,
        -1.0,
    ] {
        let mut values = [f64::MAX, f64::MAX, -f64::MAX, -f64::MAX, tail];
        for _ in 0..1000 {
            for index in (1..values.len()).rev() {
                let other = usize::try_from(rng.below(u64::try_from(index + 1).unwrap())).unwrap();
                values.swap(index, other);
            }
            assert_eq!(exact_sum(&values).unwrap().to_bits(), tail.to_bits());
        }
    }
}

#[test]
fn every_finite_singleton_and_exact_cancellation_is_preserved() {
    let mut rng = SplitMix64::new(0x6e_0007);
    for _ in 0..100_000 {
        let value = rng.any_f64();
        if !value.is_finite() {
            continue;
        }
        let expected = if value.to_bits() << 1 == 0 {
            0
        } else {
            value.to_bits()
        };
        assert_eq!(exact_sum(&[value]).unwrap().to_bits(), expected);
        assert_eq!(compensated_sum(&[value]).unwrap().to_bits(), expected);
        assert_eq!(exact_sum(&[value, -value]).unwrap().to_bits(), 0);
        assert_eq!(compensated_sum(&[value, -value]).unwrap().to_bits(), 0);
    }
}

#[test]
fn checked_compensation_and_exact_sum_have_explicit_distinct_overflow_contracts() {
    let values = [f64::MAX, f64::MAX, -f64::MAX];
    assert_eq!(exact_sum(&values).unwrap().to_bits(), f64::MAX.to_bits());
    assert!(matches!(
        compensated_sum(&values),
        Err(SumError::Arithmetic { index: Some(1), .. })
    ));
    for values in [&[f64::MAX, f64::MAX][..], &[-f64::MAX, -f64::MAX][..]] {
        assert!(matches!(
            exact_sum(values),
            Err(SumError::Arithmetic { index: None, .. })
        ));
        assert!(matches!(
            compensated_sum(values),
            Err(SumError::Arithmetic { index: Some(1), .. })
        ));
    }
    assert_eq!(
        compensated_sum(&[1e16, 1.0, -1e16, 1.0]).unwrap().to_bits(),
        2.0_f64.to_bits()
    );
}

#[test]
fn compensated_final_overflow_retains_its_distinct_quantity_and_sign() {
    // Each partial sum rounds to MAX. The two corrections are individually
    // 2^969 and add exactly to 2^970, the positive overflow midpoint offset.
    // No partial addition overflows; only the final corrected result does.
    let correction = f64::from_bits(1992_u64 << 52); // 2^969
    for (sign, class) in [
        (1.0, NonFinite::PositiveInfinity),
        (-1.0, NonFinite::NegativeInfinity),
    ] {
        let values = [sign * f64::MAX, sign * correction, sign * correction];
        let error = compensated_sum(&values).unwrap_err();
        assert!(matches!(&error, SumError::Arithmetic { index: None,
            error: ValueError::ResultNotFinite { quantity: "compensated sum", value, .. }, ..
        } if *value == class));
        assert!(error.to_string().contains("summation result"));
        assert!(error.source().unwrap().is::<ValueError>());
        assert!(matches!(
            exact_sum(&values),
            Err(SumError::Arithmetic { index: None, .. })
        ));
    }
}

#[test]
fn non_finite_inputs_have_indices_and_take_precedence_over_arithmetic() {
    for (invalid, class) in [
        (f64::NAN, NonFinite::NaN),
        (f64::INFINITY, NonFinite::PositiveInfinity),
        (f64::NEG_INFINITY, NonFinite::NegativeInfinity),
    ] {
        for sum in [compensated_sum, exact_sum] {
            let error = sum(&[f64::MAX, f64::MAX, invalid, f64::NAN]).unwrap_err();
            assert!(matches!(&error, SumError::Input {
                index: 2,
                error: ValueError::NotFinite { parameter: "values", value, .. },
                ..
            } if *value == class));
            let root = Error::from(error.clone());
            assert_eq!(root.kind(), "sum.input");
            assert!(root.source().unwrap().downcast_ref::<SumError>().is_some());
            assert!(
                root.source()
                    .unwrap()
                    .source()
                    .unwrap()
                    .downcast_ref::<ValueError>()
                    .is_some()
            );
            assert!(error.to_string().contains("2"));
        }
    }
}

#[test]
fn empty_and_signed_zero_reductions_are_canonical_and_long_carries_are_exact() {
    for values in [&[][..], &[-0.0][..], &[-0.0, 0.0, -0.0][..]] {
        assert_eq!(exact_sum(values).unwrap().to_bits(), 0);
        assert_eq!(compensated_sum(values).unwrap().to_bits(), 0);
    }
    let count = 1_000_000;
    let tiny = f64::from_bits(1);
    let values = vec![tiny; count];
    assert_eq!(
        exact_sum(&values).unwrap().to_bits(),
        u64::try_from(count).unwrap()
    );
    assert_eq!(
        compensated_sum(&values).unwrap().to_bits(),
        u64::try_from(count).unwrap()
    );
    let values = vec![1.0; count];
    assert_eq!(
        exact_sum(&values).unwrap().to_bits(),
        1_000_000.0_f64.to_bits()
    );
    assert_eq!(
        compensated_sum(&values).unwrap().to_bits(),
        1_000_000.0_f64.to_bits()
    );
}
