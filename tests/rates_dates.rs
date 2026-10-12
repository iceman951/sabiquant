//! Date-context composition checks with independent integer accrual oracles.

mod support;

use sabiquant::calendar::{Calendar, CalendarError, TargetCalendar, TargetPolicy};
use sabiquant::daycount::{
    ActualActualIcma, Business252, DayCount, DayCountError, DeterminationPeriod, IcmaError,
    ThirtyE360Isda,
};
use sabiquant::error::{Error as CrateError, ValueError};
use sabiquant::rates::{AccrualConvention, Compounding, InterestRate, RateError};
use sabiquant::time::{Date, Frequency};
use std::error::Error;
use support::float::{Tolerance, assert_close};

#[expect(
    clippy::expect_used,
    reason = "test fixture dates are valid Gregorian component triples"
)]
fn d(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid rate date fixture")
}

fn compoundings() -> Result<Vec<Compounding>, RateError> {
    let mut values = vec![Compounding::SIMPLE, Compounding::CONTINUOUS];
    for frequency in Frequency::ALL {
        if frequency.events_per_year().is_some() {
            values.extend([
                Compounding::compounded(frequency)?,
                Compounding::simple_then_compounded(frequency)?,
                Compounding::compounded_then_simple(frequency)?,
            ]);
        }
    }
    Ok(values)
}

// Independent Monday-epoch weekday prefix: no calendar classification calls.
fn weekday_prefix(date: Date) -> i32 {
    let days = date.days_since(Date::MIN);
    days / 7 * 5 + (days % 7).min(5)
}

#[expect(
    clippy::expect_used,
    reason = "validated Date year bounds make ordinal-day conversion fit i32"
)]
fn year_kind_prefix(date: Date) -> (i32, i32) {
    let years = date.year() - 1;
    let leap_years = years / 4 - years / 100 + years / 400;
    let leap_days = leap_years * 366
        + if Date::is_leap_year(date.year()) {
            i32::try_from(date.day_of_year() - 1).expect("day of year fits i32")
        } else {
            0
        };
    (date.days_since(Date::MIN) - leap_days, leap_days)
}

// One exact rational using integer counts in common versus leap years;
// independent of the production chronological year-fraction summation.
fn annual_fraction(start: Date, end: Date) -> f64 {
    let (common_start, leap_start) = year_kind_prefix(start);
    let (common_end, leap_end) = year_kind_prefix(end);
    f64::from((common_end - common_start) * 366 + (leap_end - leap_start) * 365) / 133_590.0
}

struct Case {
    start: Date,
    end: Date,
    european: i32,
    bond: i32,
    termination: i32,
}

fn cases() -> Vec<Case> {
    [
        ((2023, 1, 1), (2024, 1, 1), 360, 360, 360),
        ((2024, 1, 1), (2025, 1, 1), 360, 360, 360),
        ((2024, 2, 28), (2024, 3, 1), 3, 3, 3),
        ((2024, 2, 29), (2024, 3, 1), 2, 2, 1),
        ((2023, 2, 28), (2023, 3, 1), 3, 3, 1),
        ((2024, 1, 30), (2024, 1, 31), 0, 0, 0),
        ((2024, 1, 29), (2024, 1, 31), 1, 2, 1),
        ((2024, 8, 31), (2024, 9, 1), 1, 1, 1),
        ((1900, 2, 28), (1900, 3, 1), 3, 3, 1),
        ((2000, 2, 28), (2000, 3, 1), 3, 3, 3),
        ((2100, 2, 28), (2100, 3, 1), 3, 3, 1),
        ((2400, 2, 28), (2400, 3, 1), 3, 3, 3),
        ((1999, 12, 31), (2000, 1, 1), 1, 1, 1),
        ((2099, 12, 31), (2100, 1, 1), 1, 1, 1),
        ((1999, 7, 1), (2000, 7, 1), 360, 360, 360),
        ((2023, 2, 28), (2023, 3, 31), 32, 33, 30),
        ((2024, 2, 29), (2024, 3, 31), 31, 32, 30),
        ((2023, 1, 31), (2023, 2, 28), 28, 28, 28),
        ((2024, 1, 31), (2024, 2, 29), 29, 29, 29),
    ]
    .into_iter()
    .map(|(start, end, european, bond, termination)| Case {
        start: d(start.0, start.1, start.2),
        end: d(end.0, end.1, end.2),
        european,
        bond,
        termination,
    })
    .collect()
}

fn contexts(case: &Case) -> Result<Vec<(AccrualConvention, f64)>, IcmaError> {
    let actual = f64::from(case.end.days_since(case.start));
    let annual = annual_fraction(case.start, case.end);
    let mut periods = Vec::new();
    for year in case.start.year()..=case.end.year() {
        periods.push(DeterminationPeriod::new(d(year, 1, 1), d(year + 1, 1, 1))?);
    }
    Ok(vec![
        (DayCount::Actual360.into(), actual / 360.0),
        (DayCount::Actual365Fixed.into(), actual / 365.0),
        (DayCount::ActualActualIsda.into(), annual),
        (
            DayCount::ThirtyE360.into(),
            f64::from(case.european) / 360.0,
        ),
        (
            DayCount::Thirty360BondBasis.into(),
            f64::from(case.bond) / 360.0,
        ),
        (
            ThirtyE360Isda::new(case.end).into(),
            f64::from(case.termination) / 360.0,
        ),
        (
            ActualActualIcma::new(periods, Frequency::Annual)?.into(),
            annual,
        ),
        (
            Business252::new(Calendar::weekends_only()).into(),
            f64::from(weekday_prefix(case.end) - weekday_prefix(case.start)) / 252.0,
        ),
    ])
}

#[test]
fn date_factors_and_present_values_use_all_eight_owned_contexts_and_all_frequencies()
-> Result<(), Box<dyn Error>> {
    let methods = compoundings()?;
    assert_eq!(methods.len(), 32);
    let mut scenarios = 0;
    for case in cases() {
        let values = contexts(&case)?;
        assert_eq!(values.len(), 8);
        for (context, expected_time) in values {
            for &method in &methods {
                for rate in [-0.5, -0.005, 0.0, 0.05, 1.0] {
                    let quote = InterestRate::new(rate, context.clone(), method)?;
                    assert_eq!(quote.accrual(), &context);
                    assert_eq!(quote.clone(), quote);
                    // TOL-E2 input fractions can differ by a few roundings; this
                    // moderate-rate/time grid uses a bounded E3 comparison.
                    let factor = quote.compound_factor_between(case.start, case.end)?;
                    assert_close(
                        factor,
                        quote.compound_factor(expected_time)?,
                        Tolerance::Ulps(8),
                    );
                    assert_close(
                        quote.discount_factor_between(case.start, case.end)?,
                        quote.discount_factor(expected_time)?,
                        Tolerance::Ulps(8),
                    );
                    let positive = quote.present_value_between(9.0, case.start, case.end)?;
                    assert_close(
                        positive,
                        quote.present_value(9.0, expected_time)?,
                        Tolerance::Ulps(8),
                    );
                    let negative = quote.present_value_between(-9.0, case.start, case.end)?;
                    let doubled = quote.present_value_between(18.0, case.start, case.end)?;
                    let scaling_tolerance = if method == Compounding::SIMPLE {
                        // The same FMA factor and reciprocal use only basic
                        // arithmetic. Sign and power-of-two scaling are exact
                        // in this normal, representable test grid.
                        Tolerance::Bitwise
                    } else {
                        // Two independently evaluated discounts combine their
                        // E3 budgets, plus product rounding. Scaling the cash
                        // flow does not make repeated libm calls bitwise exact.
                        let unit_roundoff = f64::EPSILON / 2.0;
                        Tolerance::Mixed {
                            absolute: 0.0,
                            relative: unit_roundoff * (16.0 * (1.0 + factor.ln().abs()) + 2.0),
                        }
                    };
                    assert_close(negative, -positive, scaling_tolerance);
                    assert_close(doubled, 2.0 * positive, scaling_tolerance);
                    assert_eq!(
                        quote
                            .present_value_between(-0.0, case.start, case.end)?
                            .to_bits(),
                        0
                    );
                    assert_eq!(
                        quote
                            .compound_factor_between(case.start, case.start)?
                            .to_bits(),
                        1.0_f64.to_bits()
                    );
                    assert_eq!(
                        quote.discount_factor_between(case.end, case.end)?.to_bits(),
                        1.0_f64.to_bits()
                    );
                    scenarios += 1;
                }
            }
        }
    }
    assert_eq!(scenarios, 19 * 8 * 32 * 5);
    Ok(())
}

#[test]
fn unequal_date_fractions_preserve_period_growth_across_every_context_and_method_pair()
-> Result<(), Box<dyn Error>> {
    let methods = compoundings()?;
    for case in cases() {
        let values = contexts(&case)?;
        for (source_context, source_time) in &values {
            for (target_context, target_time) in &values {
                for &source_method in &methods {
                    for &target_method in &methods {
                        let source =
                            InterestRate::new(0.05, source_context.clone(), source_method)?;
                        let result = source.equivalent_rate_between(
                            target_context.clone(),
                            target_method,
                            case.start,
                            case.end,
                        );
                        if target_time.to_bits() == 0 && source_time.to_bits() != 0 {
                            assert!(
                                matches!(result,Err(RateError::ZeroTargetTime {rate,..})if rate.to_bits()==0.05_f64.to_bits())
                            );
                        } else {
                            let converted = result?;
                            assert_eq!(converted.accrual(), target_context);
                            assert_eq!(converted.compounding(), target_method);
                            assert_close(
                                converted.compound_factor_between(case.start, case.end)?,
                                source.compound_factor_between(case.start, case.end)?,
                                Tolerance::Ulps(8),
                            );
                            if source_time.to_bits() == 0 {
                                assert_eq!(converted.rate().to_bits(), 0);
                            }
                        }
                    }
                }
            }
        }
    }
    // Continuous-to-continuous conversion has a rational horizon ratio.
    let start = d(2024, 1, 1);
    let end = d(2025, 1, 1);
    let source = InterestRate::new(0.05, DayCount::Actual360, Compounding::CONTINUOUS)?;
    let target = source.equivalent_rate_between(
        DayCount::Actual365Fixed,
        Compounding::CONTINUOUS,
        start,
        end,
    )?;
    assert_close(target.rate(), 0.05 * 365.0 / 360.0, Tolerance::Ulps(2));
    Ok(())
}

#[test]
fn reversed_dates_reject_before_accrual_lookup_or_amount_shortcuts() -> Result<(), Box<dyn Error>> {
    let case = &cases()[1];
    for (context, _) in contexts(case)? {
        for method in compoundings()? {
            let quote = InterestRate::new(0.0, context.clone(), method)?;
            for result in [
                quote.compound_factor_between(case.end, case.start),
                quote.discount_factor_between(case.end, case.start),
                quote.present_value_between(0.0, case.end, case.start),
            ] {
                assert!(
                    matches!(result,Err(RateError::ReversedDates {start,end,..})if start==case.end&&end==case.start)
                );
            }
            assert!(matches!(
                quote.equivalent_rate_between(
                    DayCount::Actual360,
                    Compounding::SIMPLE,
                    case.end,
                    case.start
                ),
                Err(RateError::ReversedDates { .. })
            ));
        }
    }
    let guarded = InterestRate::new(
        0.0,
        Business252::with_target(TargetCalendar::new()),
        Compounding::SIMPLE,
    )?;
    assert!(matches!(
        guarded.compound_factor_between(d(2024, 1, 2), d(2024, 1, 1)),
        Err(RateError::ReversedDates { .. })
    ));
    Ok(())
}

#[test]
fn icma_context_checks_survive_zero_rates_equal_dates_conversion_and_cloning()
-> Result<(), Box<dyn Error>> {
    let start = d(2023, 11, 1);
    let middle = d(2024, 5, 1);
    let end = d(2024, 11, 1);
    let icma = ActualActualIcma::new(
        vec![
            DeterminationPeriod::new(start, middle)?,
            DeterminationPeriod::new(middle, end)?,
        ],
        Frequency::Semiannual,
    )?;
    let quote = InterestRate::new(0.0, icma.clone(), Compounding::SIMPLE)?;
    assert_eq!(
        quote.compound_factor_between(start, middle)?.to_bits(),
        1.0_f64.to_bits()
    );
    assert_eq!(
        quote.compound_factor_between(end, end)?.to_bits(),
        1.0_f64.to_bits()
    );
    let uncovered = end.checked_add_days(1)?;
    for error in [
        quote
            .compound_factor_between(uncovered, uncovered)
            .expect_err("outside reference coverage"),
        quote
            .present_value_between(0.0, uncovered, uncovered)
            .expect_err("zero amount retains context"),
    ] {
        assert_eq!(error.clone(), error);
        assert!(
            matches!(&error,RateError::Icma(IcmaError::OutOfCoverage {start:actual_start,end:actual_end,coverage_start,coverage_end,..})if *actual_start==uncovered&&*actual_end==uncovered&&*coverage_start==start&&*coverage_end==end)
        );
        assert!(
            error
                .source()
                .is_some_and(|source| source.downcast_ref::<IcmaError>().is_some())
        );
        let outer = CrateError::from(error);
        assert!(
            outer
                .source()
                .is_some_and(|source| source.downcast_ref::<RateError>().is_some())
        );
    }
    let scalar = InterestRate::new(0.0, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
    assert!(matches!(
        scalar.equivalent_rate_between(icma.clone(), Compounding::SIMPLE, uncovered, uncovered),
        Err(RateError::Icma(_))
    ));
    let nonzero = InterestRate::new(0.05, icma.clone(), Compounding::CONTINUOUS)?;
    for method in compoundings()? {
        let converted = nonzero.equivalent_rate_between(icma.clone(), method, start, end)?;
        assert_eq!(converted.accrual(), nonzero.accrual());
        assert_close(
            converted.compound_factor_between(start, end)?,
            nonzero.compound_factor_between(start, end)?,
            Tolerance::Ulps(8),
        );
    }
    Ok(())
}

#[test]
fn target_calendar_guard_propagates_typed_sources_through_rate_date_operations()
-> Result<(), Box<dyn Error>> {
    let guarded = Business252::with_target(TargetCalendar::new());
    for rate in [0.0, 0.05] {
        for method in compoundings()? {
            let quote = InterestRate::new(rate, guarded.clone(), method)?;
            for date in [d(1998, 12, 31), d(2024, 3, 29)] {
                for error in [
                    quote
                        .compound_factor_between(date, date)
                        .expect_err("guarded source rejects date"),
                    quote
                        .discount_factor_between(date, date)
                        .expect_err("guarded discount rejects date"),
                    quote
                        .present_value_between(0.0, date, date)
                        .expect_err("guarded zero PV rejects date"),
                ] {
                    assert_eq!(error.kind(), "rate.day_count");
                    assert_eq!(error.clone(), error);
                    let source = error
                        .source()
                        .and_then(|source| source.downcast_ref::<DayCountError>())
                        .expect("typed day-count cause");
                    let calendar = source
                        .source()
                        .and_then(|source| source.downcast_ref::<CalendarError>())
                        .expect("typed calendar cause");
                    assert!(
                        matches!(calendar,CalendarError::OutsideValidity {date:rejected,..}|CalendarError::UnverifiedCoverage {date:rejected,..}if *rejected==date)
                    );
                }
            }
            assert!(
                quote
                    .compound_factor_between(d(2026, 4, 2), d(2026, 4, 7))
                    .is_ok()
            );
            assert!(matches!(
                quote.compound_factor_between(d(2002, 12, 31), d(2026, 1, 1)),
                Err(RateError::DayCount(DayCountError::Calendar(
                    CalendarError::UnverifiedCoverage { .. }
                )))
            ));
        }
    }
    let source = InterestRate::new(0.0, DayCount::Actual360, Compounding::SIMPLE)?;
    assert!(matches!(
        source.equivalent_rate_between(guarded, Compounding::SIMPLE, d(2024, 1, 1), d(2024, 1, 1)),
        Err(RateError::DayCount(_))
    ));
    let projected = InterestRate::new(
        0.05,
        Business252::with_target(TargetCalendar::with_policy(TargetPolicy::AllowProjection)),
        Compounding::SIMPLE,
    )?;
    assert_close(
        projected.compound_factor_between(d(2024, 3, 28), d(2024, 4, 2))?,
        projected.compound_factor(1.0 / 252.0)?,
        Tolerance::Ulps(1),
    );
    Ok(())
}

#[test]
fn termination_and_business_calendar_overrides_remain_owned_across_conversions()
-> Result<(), Box<dyn Error>> {
    let start = d(2023, 1, 31);
    let end = d(2023, 2, 28);
    let terminal = ThirtyE360Isda::new(end);
    let intermediate = ThirtyE360Isda::new(d(2023, 3, 31));
    let source = InterestRate::new(0.05, terminal, Compounding::CONTINUOUS)?;
    assert_close(
        source.compound_factor_between(start, end)?,
        source.compound_factor(28.0 / 360.0)?,
        Tolerance::Ulps(1),
    );
    let converted =
        source.equivalent_rate_between(intermediate, Compounding::CONTINUOUS, start, end)?;
    assert_close(converted.rate(), 0.05 * 28.0 / 30.0, Tolerance::Ulps(2));
    assert!(
        matches!(converted.accrual(),AccrualConvention::ThirtyE360Isda(value)if value.termination_date()==d(2023,3,31))
    );
    assert_close(
        converted.compound_factor_between(start, end)?,
        source.compound_factor_between(start, end)?,
        Tolerance::Ulps(2),
    );
    assert_eq!(source.accrual(), &AccrualConvention::from(terminal));
    let calendar = Calendar::weekends_only();
    let holiday = d(2024, 2, 29);
    let original = InterestRate::new(
        0.05,
        Business252::new(calendar.clone()),
        Compounding::SIMPLE,
    )?;
    let modified = InterestRate::new(
        0.05,
        Business252::new(calendar.with_holiday(holiday)),
        Compounding::SIMPLE,
    )?;
    let first = d(2024, 2, 28);
    let last = d(2024, 3, 1);
    assert_close(
        original.compound_factor_between(first, last)?,
        original.compound_factor(2.0 / 252.0)?,
        Tolerance::Ulps(1),
    );
    assert_close(
        modified.compound_factor_between(first, last)?,
        modified.compound_factor(1.0 / 252.0)?,
        Tolerance::Ulps(1),
    );
    let cloned = modified.clone();
    assert_eq!(cloned.accrual(), modified.accrual());
    assert!(calendar.is_business_day(holiday));
    let guarded = InterestRate::new(
        0.05,
        Business252::with_target(TargetCalendar::new().without_holiday(d(2026, 4, 3))),
        Compounding::SIMPLE,
    )?;
    assert_close(
        guarded.compound_factor_between(d(2026, 4, 2), d(2026, 4, 7))?,
        guarded.compound_factor(2.0 / 252.0)?,
        Tolerance::Ulps(1),
    );
    Ok(())
}

#[test]
fn distinct_ordered_dates_can_have_zero_accrual_and_zero_target_time_errors()
-> Result<(), Box<dyn Error>> {
    for (start, end, target) in [
        (
            d(2024, 1, 30),
            d(2024, 1, 31),
            AccrualConvention::from(DayCount::ThirtyE360),
        ),
        (
            d(2024, 8, 31),
            d(2024, 9, 1),
            AccrualConvention::from(Business252::new(Calendar::weekends_only())),
        ),
    ] {
        assert!(start < end);
        for method in compoundings()? {
            let zero_horizon = InterestRate::new(0.05, target.clone(), method)?;
            assert_eq!(
                zero_horizon.compound_factor_between(start, end)?.to_bits(),
                1.0_f64.to_bits()
            );
            let nonzero_source = InterestRate::new(0.05, DayCount::Actual365Fixed, method)?;
            assert!(
                matches!(nonzero_source.equivalent_rate_between(target.clone(),method,start,end),Err(RateError::ZeroTargetTime {source_time,rate,..})if source_time>0.0&&rate.to_bits()==0.05_f64.to_bits())
            );
            let zero_rate = InterestRate::new(-0.0, DayCount::Actual365Fixed, method)?;
            assert_eq!(
                zero_rate
                    .equivalent_rate_between(target.clone(), method, start, end)?
                    .rate()
                    .to_bits(),
                0
            );
            assert_eq!(
                zero_horizon
                    .equivalent_rate_between(DayCount::Actual365Fixed, method, start, end)?
                    .rate()
                    .to_bits(),
                0
            );
        }
    }
    Ok(())
}

#[test]
fn date_present_values_have_exact_identity_paths_and_typed_extreme_product_failures()
-> Result<(), Box<dyn Error>> {
    let start = d(2023, 1, 1);
    let end = d(2024, 1, 1);
    let simple = InterestRate::new(0.125, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
    assert_eq!(
        simple.compound_factor_between(start, end)?.to_bits(),
        1.125_f64.to_bits()
    );
    assert_eq!(
        simple.present_value_between(9.0, start, end)?.to_bits(),
        8.0_f64.to_bits()
    );
    for amount in [
        f64::MAX,
        -f64::MAX,
        9.0,
        -9.0,
        f64::from_bits(1),
        -f64::from_bits(1),
    ] {
        assert_eq!(
            simple
                .present_value_between(amount, start, start)?
                .to_bits(),
            amount.to_bits()
        );
    }
    assert_eq!(
        simple.present_value_between(-0.0, start, start)?.to_bits(),
        0
    );
    let growth = InterestRate::new(1.0, DayCount::Actual365Fixed, Compounding::CONTINUOUS)?;
    for amount in [f64::from_bits(1), -f64::from_bits(1)] {
        assert!(matches!(
            growth.present_value_between(amount, start, end),
            Err(RateError::Value(ValueError::ResultOutOfDomain {
                quantity: "present value",
                ..
            }))
        ));
    }
    let negative = InterestRate::new(-1.0, DayCount::Actual365Fixed, Compounding::CONTINUOUS)?;
    for amount in [f64::MAX, -f64::MAX] {
        assert!(matches!(
            negative.present_value_between(amount, start, end),
            Err(RateError::Value(ValueError::ResultNotFinite {
                quantity: "present value",
                ..
            }))
        ));
    }
    assert!(matches!(
        growth.present_value_between(0.0, d(1000, 1, 1), d(2000, 1, 1)),
        Err(RateError::Value(ValueError::ResultNotFinite {
            quantity: "compound factor",
            ..
        }))
    ));
    for amount in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            simple.present_value_between(amount, start, end),
            Err(RateError::Value(ValueError::NotFinite {
                parameter: "amount",
                ..
            }))
        ));
    }
    Ok(())
}
