//! FS-RATE/FS-DISC domain, boundary and convention-context regressions.

mod support;

use sabiquant::calendar::{Calendar, TargetCalendar};
use sabiquant::daycount::{
    ActualActualIcma, Business252, DayCount, DeterminationPeriod, ThirtyE360Isda,
};
use sabiquant::rates::{
    AccrualConvention, Compounding, CompoundingMethod, InterestRate, RateError,
};
use sabiquant::time::{Date, Frequency};
use std::error::Error;
use support::float::{Tolerance, assert_close};

#[expect(
    clippy::expect_used,
    reason = "the helper constructs only named, known periodic frequencies"
)]
fn all_compoundings() -> Vec<Compounding> {
    let mut result = vec![Compounding::SIMPLE, Compounding::CONTINUOUS];
    for frequency in Frequency::ALL {
        if frequency.events_per_year().is_some() {
            result.extend([
                Compounding::compounded(frequency).expect("periodic frequency"),
                Compounding::simple_then_compounded(frequency).expect("periodic frequency"),
                Compounding::compounded_then_simple(frequency).expect("periodic frequency"),
            ]);
        }
    }
    result
}

#[test]
fn all_five_method_identifiers_and_32_frequency_values_are_explicit() {
    assert_eq!(all_compoundings().len(), 32);
    for method in CompoundingMethod::ALL {
        let text = method.canonical_identifier();
        assert_eq!(text.parse::<CompoundingMethod>().unwrap(), method);
        assert_eq!(method.to_string(), text);
        for unknown in [
            format!(" {text}"),
            format!("{text} "),
            text.to_ascii_lowercase(),
            format!("{text}\0"),
        ] {
            assert!(unknown.parse::<CompoundingMethod>().is_err());
        }
    }
    for value in all_compoundings() {
        if let Some(frequency) = value.frequency() {
            assert!(frequency.events_per_year().unwrap() > 0);
            assert!(value.to_string().contains(&frequency.to_string()));
        } else {
            assert!(matches!(
                value.method(),
                CompoundingMethod::Simple | CompoundingMethod::Continuous
            ));
        }
    }
    for constructor in [
        Compounding::compounded,
        Compounding::simple_then_compounded,
        Compounding::compounded_then_simple,
    ] {
        assert!(matches!(
            constructor(Frequency::Once),
            Err(RateError::InvalidFrequency {
                frequency: Frequency::Once,
                ..
            })
        ));
    }
}

#[test]
fn zero_rate_and_zero_horizon_are_exact_after_domain_validation() {
    for compounding in all_compoundings() {
        for rate in [-0.005, 0.0, -0.0, 0.05, 10.0] {
            let quote = InterestRate::new(rate, DayCount::Actual365Fixed, compounding).unwrap();
            for time in [0.0, -0.0] {
                assert_eq!(
                    quote.compound_factor(time).unwrap().to_bits(),
                    1.0_f64.to_bits()
                );
                assert_eq!(
                    quote.discount_factor(time).unwrap().to_bits(),
                    1.0_f64.to_bits()
                );
                assert_eq!(
                    quote
                        .equivalent_rate(Compounding::CONTINUOUS, time)
                        .unwrap()
                        .rate()
                        .to_bits(),
                    0
                );
            }
        }
        for rate in [0.0, -0.0] {
            let quote = InterestRate::new(rate, DayCount::Actual360, compounding).unwrap();
            assert_eq!(quote.rate().to_bits(), 0);
            for time in [f64::from_bits(1), 1e-12, 1.0, 100.0, f64::MAX] {
                assert_eq!(
                    quote.compound_factor(time).unwrap().to_bits(),
                    1.0_f64.to_bits()
                );
            }
        }
    }
}

#[test]
fn periodic_base_boundary_is_rejected_even_at_zero_or_simple_hybrid_horizons() {
    for compounding in all_compoundings() {
        if let Some(frequency) = compounding.frequency() {
            let count = f64::from(frequency.events_per_year().unwrap());
            for rate in [-count, -count.next_up(), f64::MIN] {
                assert_eq!(
                    InterestRate::new(rate, DayCount::Actual360, compounding)
                        .unwrap_err()
                        .kind(),
                    "rate.value"
                );
            }
            let rate = (-count).next_up();
            let quote = InterestRate::new(rate, DayCount::Actual360, compounding).unwrap();
            assert_eq!(
                quote.compound_factor(0.0).unwrap().to_bits(),
                1.0_f64.to_bits()
            );
            assert!(quote.compound_factor(1.0 / count).unwrap() > 0.0);
        }
    }
}

#[test]
fn every_nonfinite_numeric_input_returns_a_typed_error() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for compounding in all_compoundings() {
            assert!(InterestRate::new(value, DayCount::Actual360, compounding).is_err());
            let quote = InterestRate::new(0.05, DayCount::Actual360, compounding).unwrap();
            assert!(quote.compound_factor(value).is_err());
            assert!(quote.discount_factor(value).is_err());
            assert!(quote.equivalent_rate(compounding, value).is_err());
            assert!(quote.present_value(value, 1.0).is_err());
            assert!(
                InterestRate::implied_rate(value, 1.0, DayCount::Actual360, compounding).is_err()
            );
            assert!(
                InterestRate::implied_rate(1.0, value, DayCount::Actual360, compounding).is_err()
            );
            assert!(
                InterestRate::implied_rate_from_discount(
                    value,
                    1.0,
                    DayCount::Actual360,
                    compounding
                )
                .is_err()
            );
        }
    }
}

#[test]
fn invalid_horizons_factors_and_zero_time_inference_never_use_approximate_closeness() {
    for compounding in all_compoundings() {
        let quote = InterestRate::new(0.05, DayCount::Actual360, compounding).unwrap();
        for time in [-f64::from_bits(1), -1.0, f64::MIN] {
            assert!(quote.compound_factor(time).is_err());
            assert!(quote.equivalent_rate(compounding, time).is_err());
        }
        for factor in [-1.0, 0.0, -0.0] {
            assert!(
                InterestRate::implied_rate(factor, 1.0, DayCount::Actual360, compounding).is_err()
            );
            assert!(
                InterestRate::implied_rate_from_discount(
                    factor,
                    1.0,
                    DayCount::Actual360,
                    compounding
                )
                .is_err()
            );
        }
        assert_eq!(
            InterestRate::implied_rate(1.0, -0.0, DayCount::Actual360, compounding)
                .unwrap()
                .rate()
                .to_bits(),
            0
        );
        for factor in [1.0_f64.next_up(), 1.0_f64.next_down(), 2.0] {
            assert!(matches!(
                InterestRate::implied_rate(factor, 0.0, DayCount::Actual360, compounding),
                Err(RateError::ZeroTime {
                    quantity: "compound factor",
                    ..
                })
            ));
            assert!(matches!(
                InterestRate::implied_rate_from_discount(
                    factor,
                    0.0,
                    DayCount::Actual360,
                    compounding
                ),
                Err(RateError::ZeroTime {
                    quantity: "discount factor",
                    ..
                })
            ));
        }
    }
}

#[test]
fn stable_conversions_preserve_tiny_rates_and_do_not_form_overflowing_factors() {
    for frequency in Frequency::ALL {
        if frequency.events_per_year().is_none() {
            continue;
        }
        let periodic = Compounding::compounded(frequency).unwrap();
        for rate in [f64::from_bits(1), -f64::from_bits(1), 1e-20, -1e-20] {
            let quote = InterestRate::new(rate, DayCount::Actual365Fixed, periodic).unwrap();
            for time in [f64::from_bits(1), 1e-12, 1.0, f64::MAX] {
                let converted = quote
                    .equivalent_rate(Compounding::CONTINUOUS, time)
                    .unwrap();
                assert_eq!(converted.rate().to_bits(), rate.to_bits());
                assert_eq!(
                    converted
                        .equivalent_rate(periodic, time)
                        .unwrap()
                        .rate()
                        .to_bits(),
                    rate.to_bits()
                );
            }
        }
        let quote = InterestRate::new(0.05, DayCount::Actual365Fixed, periodic).unwrap();
        assert!(quote.compound_factor(f64::MAX).is_err());
        let one = quote
            .equivalent_rate(Compounding::CONTINUOUS, 1.0)
            .unwrap()
            .rate();
        let huge = quote
            .equivalent_rate(Compounding::CONTINUOUS, f64::MAX)
            .unwrap()
            .rate();
        assert_close(
            one,
            huge,
            Tolerance::Mixed {
                absolute: 8.0 * f64::EPSILON / 2.0 * one.abs(),
                relative: 0.0,
            },
        );
    }
}

#[test]
fn near_negative_frequency_uses_the_exact_positive_difference_before_division() {
    for frequency in Frequency::ALL {
        let Some(count) = frequency.events_per_year() else {
            continue;
        };
        let count = f64::from(count);
        let rate = (-count).next_up();
        let quote = InterestRate::new(
            rate,
            DayCount::Actual360,
            Compounding::compounded(frequency).unwrap(),
        )
        .unwrap();
        let base = (count + rate) / count;
        let actual = quote.compound_factor(1.0 / count).unwrap();
        // The supplied rounded horizon can shift the real exponent slightly
        // from one. Initial TOL-E3 accounts for that logarithmic sensitivity.
        assert_close(
            actual,
            base,
            Tolerance::Mixed {
                absolute: 0.0,
                relative: 8.0 * f64::EPSILON / 2.0 * (1.0 + base.ln().abs()),
            },
        );
    }
}

#[test]
fn all_conversion_pairs_and_positive_horizons_preserve_factors_and_conditioned_rates() {
    let values = all_compoundings();
    for &source in &values {
        for &target in &values {
            for rate in [-0.005, -1e-20, 0.0, 1e-20, 0.05] {
                let quote = InterestRate::new(rate, DayCount::Actual365Fixed, source).unwrap();
                for time in [1e-12, 1.0 / 365.0, 0.1, 0.5, 1.0, 10.0, 100.0] {
                    let converted = quote.equivalent_rate(target, time).unwrap();
                    let factor = quote.compound_factor(time).unwrap();
                    // Initial TOL-E3/E4 proposals, pending high-precision calibration.
                    let relative = 16.0 * f64::EPSILON / 2.0 * (1.0 + factor.ln().abs());
                    assert_close(
                        converted.compound_factor(time).unwrap(),
                        factor,
                        Tolerance::Mixed {
                            absolute: 0.0,
                            relative,
                        },
                    );
                    let returned = converted.equivalent_rate(source, time).unwrap();
                    assert_close(
                        returned.rate(),
                        rate,
                        Tolerance::Mixed {
                            absolute: 16.0 * f64::EPSILON / 2.0 * (rate.abs() + 1.0 / time),
                            relative: 0.0,
                        },
                    );
                    if source == target {
                        assert_eq!(converted.rate().to_bits(), quote.rate().to_bits());
                    }
                }
            }
        }
    }
}

#[test]
fn hybrid_threshold_and_adjacent_values_follow_the_declared_branch() {
    for frequency in Frequency::ALL {
        let Some(count) = frequency.events_per_year() else {
            continue;
        };
        let threshold = 1.0 / f64::from(count);
        for rate in [-0.005, 0.05] {
            let simple = InterestRate::new(rate, DayCount::Actual360, Compounding::SIMPLE).unwrap();
            let periodic = InterestRate::new(
                rate,
                DayCount::Actual360,
                Compounding::compounded(frequency).unwrap(),
            )
            .unwrap();
            let first = InterestRate::new(
                rate,
                DayCount::Actual360,
                Compounding::simple_then_compounded(frequency).unwrap(),
            )
            .unwrap();
            let last = InterestRate::new(
                rate,
                DayCount::Actual360,
                Compounding::compounded_then_simple(frequency).unwrap(),
            )
            .unwrap();
            for time in [threshold.next_down(), threshold, threshold.next_up()] {
                let expected_first = if time <= threshold {
                    simple.compound_factor(time).unwrap()
                } else {
                    periodic.compound_factor(time).unwrap()
                };
                let expected_last = if time <= threshold {
                    periodic.compound_factor(time).unwrap()
                } else {
                    simple.compound_factor(time).unwrap()
                };
                assert_eq!(
                    first.compound_factor(time).unwrap().to_bits(),
                    expected_first.to_bits()
                );
                assert_eq!(
                    last.compound_factor(time).unwrap().to_bits(),
                    expected_last.to_bits()
                );
            }
        }
    }
}

#[test]
fn simple_domain_overflow_underflow_and_reciprocal_boundaries_are_checked() {
    let negative = InterestRate::new(-1.0, DayCount::Actual360, Compounding::SIMPLE).unwrap();
    assert!(negative.compound_factor(1.0).is_err());
    assert!(negative.compound_factor(f64::MAX).is_err());
    assert!(
        negative
            .equivalent_rate(Compounding::CONTINUOUS, f64::MAX)
            .is_err()
    );
    let continuous = InterestRate::new(1.0, DayCount::Actual360, Compounding::CONTINUOUS).unwrap();
    assert!(continuous.compound_factor(710.0).is_err());
    let inverse = InterestRate::new(-1.0, DayCount::Actual360, Compounding::CONTINUOUS).unwrap();
    assert!(inverse.compound_factor(750.0).is_err());
    assert!(inverse.compound_factor(744.0).unwrap().is_subnormal());
    assert!(inverse.discount_factor(744.0).is_err());
    let inferred = InterestRate::implied_rate_from_discount(
        f64::from_bits(1),
        1.0,
        DayCount::Actual360,
        Compounding::CONTINUOUS,
    )
    .unwrap();
    assert!(inferred.rate().is_finite() && inferred.rate() > 700.0);
}

#[test]
fn direct_simple_inference_and_same_phase_conversion_avoid_canceling_large_logs() {
    let horizon = f64::from_bits(0x7fe0_0000_0000_0000); // exactly 2^1023
    let quote = InterestRate::implied_rate_from_discount(
        f64::from_bits(1),
        horizon,
        DayCount::Actual360,
        Compounding::SIMPLE,
    )
    .unwrap();
    assert_eq!(quote.rate().to_bits(), (2.0_f64.powi(51)).to_bits());
    let source = InterestRate::new(f64::MAX, DayCount::Actual360, Compounding::SIMPLE).unwrap();
    let target = Compounding::compounded_then_simple(Frequency::Daily).unwrap();
    assert_eq!(
        source
            .equivalent_rate(target, f64::MAX)
            .unwrap()
            .rate()
            .to_bits(),
        f64::MAX.to_bits()
    );
}

#[test]
fn derived_simple_rates_reject_a_rounded_nonpositive_active_factor() {
    let source =
        InterestRate::new(-40.0, DayCount::Actual365Fixed, Compounding::CONTINUOUS).unwrap();
    for target in [
        Compounding::SIMPLE,
        Compounding::simple_then_compounded(Frequency::Annual).unwrap(),
        Compounding::compounded_then_simple(Frequency::Monthly).unwrap(),
    ] {
        assert!(matches!(
            InterestRate::implied_rate(f64::from_bits(1), 1.0, DayCount::Actual365Fixed, target),
            Err(RateError::Value(_))
        ));
        assert!(matches!(
            InterestRate::implied_rate_from_discount(
                f64::MAX,
                1.0,
                DayCount::Actual365Fixed,
                target
            ),
            Err(RateError::Value(_))
        ));
        assert!(matches!(
            source.equivalent_rate(target, 1.0),
            Err(RateError::Value(_))
        ));
        assert!(matches!(
            source.equivalent_rate_between(
                DayCount::Actual365Fixed,
                target,
                "2026-01-01".parse().unwrap(),
                "2027-01-01".parse().unwrap()
            ),
            Err(RateError::Value(_))
        ));
    }
}

#[test]
fn a_valid_near_boundary_simple_rate_uses_absolute_reconstruction_conditioning() {
    // Decimal at 120 and 200 digits independently rounds exp(-34.5)-1 to
    // this binary64. Its rate representation loses about 3.88% of the tiny
    // factor even though the inferred rate itself is accurate.
    let source =
        InterestRate::new(-34.5, DayCount::Actual365Fixed, Compounding::CONTINUOUS).unwrap();
    let target = source.equivalent_rate(Compounding::SIMPLE, 1.0).unwrap();
    let expected = f64::from_bits(0xbfef_ffff_ffff_fff7);
    assert_close(target.rate(), expected, Tolerance::Ulps(1));
    let factor = source.compound_factor(1.0).unwrap();
    let recovered = target.compound_factor(1.0).unwrap();
    assert!(recovered > 0.0);
    assert!((recovered - factor).abs() / factor > 0.01);
    let unit = f64::EPSILON / 2.0;
    // dC/dr=t=1: at most unit*abs(rate) from nearest rate rounding,
    // plus the independently measured E3 factor evaluation allowance.
    assert_close(
        recovered,
        factor,
        Tolerance::Mixed {
            absolute: unit * expected.abs(),
            relative: 8.0 * unit * (1.0 + factor.ln().abs()),
        },
    );
}

#[test]
fn every_daycount_context_and_zero_horizon_keeps_its_required_domain() {
    let start = Date::from_ymd(2024, 1, 1).unwrap();
    let end = Date::from_ymd(2024, 7, 1).unwrap();
    let icma = ActualActualIcma::new(
        vec![DeterminationPeriod::new(start, end).unwrap()],
        Frequency::Semiannual,
    )
    .unwrap();
    let contexts = [
        DayCount::Actual360.into(),
        AccrualConvention::from(icma.clone()),
        ThirtyE360Isda::new(end).into(),
        Business252::new(Calendar::weekends_only()).into(),
    ];
    for context in contexts {
        let time = context.year_fraction(start, end).unwrap();
        let quote = InterestRate::new(0.05, context, Compounding::SIMPLE).unwrap();
        assert_eq!(
            quote.compound_factor_between(start, end).unwrap().to_bits(),
            quote.compound_factor(time).unwrap().to_bits()
        );
        assert!(matches!(
            quote.compound_factor_between(end, start),
            Err(RateError::ReversedDates { .. })
        ));
    }
    let unknown = Date::from_ymd(2024, 1, 1).unwrap();
    let guarded = InterestRate::new(
        0.0,
        Business252::with_target(TargetCalendar::new()),
        Compounding::SIMPLE,
    )
    .unwrap();
    let error = guarded
        .compound_factor_between(unknown, unknown)
        .unwrap_err();
    assert_eq!(error.kind(), "rate.day_count");
    assert_eq!(
        error.source().unwrap().source().unwrap().to_string(),
        TargetCalendar::new()
            .is_business_day(unknown)
            .unwrap_err()
            .to_string()
    );
    let quote = InterestRate::new(0.0, icma, Compounding::SIMPLE).unwrap();
    assert_eq!(
        quote
            .compound_factor_between(
                start.checked_sub_days(1).unwrap(),
                start.checked_sub_days(1).unwrap()
            )
            .unwrap_err()
            .kind(),
        "rate.icma"
    );
}

#[test]
fn date_equivalence_uses_both_fractions_and_rejects_zero_target_growth_time() {
    let start = Date::from_ymd(2024, 1, 1).unwrap();
    let end = Date::from_ymd(2025, 1, 1).unwrap();
    let source = InterestRate::new(0.05, DayCount::Actual360, Compounding::CONTINUOUS).unwrap();
    let target = source
        .equivalent_rate_between(
            DayCount::Actual365Fixed,
            Compounding::CONTINUOUS,
            start,
            end,
        )
        .unwrap();
    assert_close(target.rate(), 0.05 * 365.0 / 360.0, Tolerance::Ulps(2));
    assert_close(
        target.compound_factor_between(start, end).unwrap(),
        source.compound_factor_between(start, end).unwrap(),
        Tolerance::Ulps(2),
    );
    let closed = Business252::new(Calendar::custom(sabiquant::time::Weekday::ALL, []));
    assert!(matches!(
        source.equivalent_rate_between(closed, Compounding::SIMPLE, start, end),
        Err(RateError::ZeroTargetTime { .. })
    ));
}

#[test]
fn present_values_are_signed_linear_and_check_nonzero_product_underflow() {
    let quote = InterestRate::new(0.05, DayCount::Actual360, Compounding::SIMPLE).unwrap();
    for time in [0.0, 1.0, 100.0] {
        let positive = quote.present_value(100.0, time).unwrap();
        let negative = quote.present_value(-100.0, time).unwrap();
        assert_eq!(negative.to_bits(), (-positive).to_bits());
        assert_eq!(quote.present_value(-0.0, time).unwrap().to_bits(), 0);
        assert_eq!(
            quote.present_value(200.0, time).unwrap().to_bits(),
            (2.0 * positive).to_bits()
        );
    }
    let large = InterestRate::new(1.0, DayCount::Actual360, Compounding::CONTINUOUS).unwrap();
    assert!(large.present_value(f64::from_bits(1), 700.0).is_err());
    let negative = InterestRate::new(-1.0, DayCount::Actual360, Compounding::CONTINUOUS).unwrap();
    assert!(negative.present_value(f64::MAX, 700.0).is_err());
}

#[test]
fn reciprocal_identity_accounts_for_subnormal_discount_rounding() {
    let rate = f64::from_bits(0x7fee_f80b_83df_d192);
    let quote = InterestRate::new(rate, DayCount::Actual360, Compounding::SIMPLE).unwrap();
    let compound = quote.compound_factor(1.0).unwrap();
    let discount = quote.discount_factor(1.0).unwrap();
    assert!(discount.is_subnormal());
    let unit = f64::EPSILON / 2.0;
    let product = compound * discount;
    assert!((product - 1.0).abs() > 2.0 * unit);
    // A subnormal reciprocal has absolute error at most half MIN_SUB;
    // multiply that allowance by C before halving, then add product rounding.
    let bound = unit + 0.5 * (compound * f64::from_bits(1));
    assert!((product - 1.0).abs() <= bound);
}

#[test]
fn simple_growth_conversion_rescales_a_huge_product_and_rejects_unrepresentable_negative_growth() {
    let quote = InterestRate::new(f64::MAX, DayCount::Actual360, Compounding::SIMPLE).unwrap();
    assert!(quote.compound_factor(f64::MAX).is_err());
    let converted = quote
        .equivalent_rate(Compounding::CONTINUOUS, f64::MAX)
        .unwrap();
    // Decimal120/200: ln(1+MAX*MAX)/MAX, without a binary64 product.
    let expected = 7.896_594_798_396_956e-306_f64;
    assert_close(
        converted.rate(),
        expected,
        Tolerance::Mixed {
            absolute: 8.0 * (f64::EPSILON / 2.0) * expected.abs(),
            relative: 0.0,
        },
    );
    let negative = InterestRate::new(-f64::MAX, DayCount::Actual360, Compounding::SIMPLE).unwrap();
    let time = 1.0 / f64::MAX;
    assert!(negative.compound_factor(time).unwrap() > 0.0);
    assert!(matches!(
        negative.equivalent_rate(Compounding::CONTINUOUS, time),
        Err(RateError::Value(_))
    ));
}

#[test]
fn discount_inference_checks_its_time_and_unequal_date_conversion_checks_both_active_domains() {
    for time in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        assert!(matches!(
            InterestRate::implied_rate_from_discount(
                1.0,
                time,
                DayCount::Actual360,
                Compounding::SIMPLE
            ),
            Err(RateError::Value(_))
        ));
    }
    let start = "2026-01-01".parse().unwrap();
    let end = "2027-01-01".parse().unwrap();
    for (rate, source) in [
        (-2.0, Compounding::SIMPLE),
        (-40.0, Compounding::CONTINUOUS),
    ] {
        let quote = InterestRate::new(rate, DayCount::Actual360, source).unwrap();
        assert!(matches!(
            quote.equivalent_rate_between(
                DayCount::Actual365Fixed,
                Compounding::SIMPLE,
                start,
                end
            ),
            Err(RateError::Value(_))
        ));
    }
}
