//! FS-DCC, FS-RATE, FS-CF and NFR-11: explicit US/AFB contexts through rates and legs.
//! Hand fractions below are derived from the frozen contract, not public daycounter calls.

mod support;

use sabiquant::calendar::{BusinessDayConvention as Bdc, Calendar};
use sabiquant::cashflows::{
    CashFlow, CashFlowError, FixedCouponRate, FixedRateCoupon, Leg,
    PaymentDateInclusion as Inclusion, SimpleCashFlow,
};
use sabiquant::daycount::{DayCount, Thirty360Us};
use sabiquant::rates::{AccrualConvention, Compounding, InterestRate, RateError};
use sabiquant::schedule::{ScheduleBuilder, ScheduleCalendar};
use sabiquant::time::{Date, Frequency};
use std::error::Error;
use support::float::{Tolerance, assert_close};

fn methods() -> Result<Vec<Compounding>, RateError> {
    let mut result = vec![Compounding::SIMPLE, Compounding::CONTINUOUS];
    for frequency in Frequency::ALL {
        if frequency.events_per_year().is_some() {
            result.extend([
                Compounding::compounded(frequency)?,
                Compounding::simple_then_compounded(frequency)?,
                Compounding::compounded_then_simple(frequency)?,
            ]);
        }
    }
    Ok(result)
}

// Two E3 evaluations, k=8 each, with normal-domain logarithmic conditioning.
fn factor_tolerance(a: f64, b: f64) -> Tolerance {
    Tolerance::Mixed {
        absolute: f64::from_bits(1),
        relative: 16.0 * (f64::EPSILON / 2.0) * (1.0 + a.ln().abs().max(b.ln().abs())),
    }
}

#[test]
fn all_thirty_two_compoundings_use_the_hand_derived_date_fraction() -> Result<(), Box<dyn Error>> {
    let cases = [
        (
            Thirty360Us::new(true).into(),
            Date::from_ymd(2023, 2, 28)?,
            Date::from_ymd(2023, 3, 31)?,
            30.0 / 360.0,
        ),
        (
            Thirty360Us::new(false).into(),
            Date::from_ymd(2023, 2, 28)?,
            Date::from_ymd(2023, 3, 31)?,
            33.0 / 360.0,
        ),
        (
            AccrualConvention::from(DayCount::ActualActualAfb),
            Date::from_ymd(2024, 2, 28)?,
            Date::from_ymd(2025, 2, 28)?,
            1.0 + 1.0 / 365.0,
        ),
    ];
    let all = methods()?;
    assert_eq!(all.len(), 32);
    for (context, start, end, time) in cases {
        for method in &all {
            for rate in [-0.02, 0.0, 0.06] {
                let quote = InterestRate::new(rate, context.clone(), *method)?;
                let expected = quote.compound_factor(time)?;
                let actual = quote.compound_factor_between(start, end)?;
                assert_close(actual, expected, factor_tolerance(actual, expected));
                let expected = quote.discount_factor(time)?;
                let actual = quote.discount_factor_between(start, end)?;
                assert_close(actual, expected, factor_tolerance(actual, expected));
                let expected = quote.present_value(-125.0, time)?;
                let actual = quote.present_value_between(-125.0, start, end)?;
                assert_close(
                    actual,
                    expected,
                    Tolerance::Mixed {
                        absolute: 0.0,
                        relative: 16.0 * f64::EPSILON,
                    },
                );
                assert_eq!(quote.accrual(), &context);
                assert!(quote.compound_factor_between(end, start).is_err());
            }
        }
    }
    Ok(())
}

#[test]
fn equivalent_rates_independently_compute_target_context_and_preserve_identity()
-> Result<(), Box<dyn Error>> {
    let start = Date::from_ymd(2023, 2, 28)?;
    let end = Date::from_ymd(2023, 3, 31)?;
    let contexts: [(AccrualConvention, f64); 3] = [
        (Thirty360Us::new(true).into(), 30.0 / 360.0),
        (Thirty360Us::new(false).into(), 33.0 / 360.0),
        (DayCount::ActualActualAfb.into(), 31.0 / 365.0),
    ];
    for (source, source_time) in &contexts {
        let quote = InterestRate::new(0.06, source.clone(), Compounding::SIMPLE)?;
        for (target, target_time) in &contexts {
            let equivalent =
                quote.equivalent_rate_between(target.clone(), Compounding::SIMPLE, start, end)?;
            let expected = 0.06 * (source_time / target_time);
            // E4 simple target derivative scale; normal-domain horizons and small growth.
            let tolerance = 8.0
                * (f64::EPSILON / 2.0)
                * (expected.abs()
                    + 1.0 / target_time
                    + (expected.abs() + 1.0 / target_time) * (0.06 * source_time).ln_1p().abs());
            assert_close(
                equivalent.rate(),
                expected,
                Tolerance::Mixed {
                    absolute: tolerance,
                    relative: 0.0,
                },
            );
            assert_eq!(equivalent.accrual(), target);
            for method in methods()? {
                let converted =
                    quote.equivalent_rate_between(target.clone(), method, start, end)?;
                let a = quote.compound_factor(*source_time)?;
                let b = converted.compound_factor(*target_time)?;
                assert_close(a, b, factor_tolerance(a, b));
                assert_eq!(converted.accrual(), target);
            }
            let zero = quote.equivalent_rate_between(
                target.clone(),
                Compounding::CONTINUOUS,
                start,
                start,
            )?;
            assert_eq!(zero.rate().to_bits(), 0);
            assert_eq!(zero.accrual(), target);
            let zero = InterestRate::new(-0.0, source.clone(), Compounding::SIMPLE)?
                .equivalent_rate_between(target.clone(), Compounding::SIMPLE, start, end)?;
            assert_eq!(zero.rate().to_bits(), 0);
            assert_eq!(zero.accrual(), target);
        }
    }
    Ok(())
}

#[test]
fn adjusted_payment_does_not_replace_raw_coupon_or_partial_accrual_endpoints()
-> Result<(), Box<dyn Error>> {
    let cases: [(AccrualConvention, Date, Date, Date, Date, f64, f64); 3] = [
        (
            Thirty360Us::new(true).into(),
            Date::from_ymd(2023, 2, 28)?,
            Date::from_ymd(2023, 4, 30)?,
            Date::from_ymd(2023, 5, 1)?,
            Date::from_ymd(2023, 3, 31)?,
            60.0 / 360.0,
            30.0 / 360.0,
        ),
        (
            Thirty360Us::new(false).into(),
            Date::from_ymd(2023, 2, 28)?,
            Date::from_ymd(2023, 4, 30)?,
            Date::from_ymd(2023, 5, 1)?,
            Date::from_ymd(2023, 3, 31)?,
            62.0 / 360.0,
            33.0 / 360.0,
        ),
        (
            DayCount::ActualActualAfb.into(),
            Date::from_ymd(2024, 2, 28)?,
            Date::from_ymd(2025, 3, 1)?,
            Date::from_ymd(2025, 3, 3)?,
            Date::from_ymd(2025, 2, 28)?,
            1.0 + 2.0 / 366.0,
            1.0 + 1.0 / 365.0,
        ),
    ];
    for (context, start, end, payment, partial, full_time, partial_time) in cases {
        let schedule = ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(Calendar::weekends_only()),
            Bdc::Following,
            Bdc::Following,
        )?
        .build_zero(2)?;
        assert_eq!(schedule.contractual_dates(), &[start, end]);
        assert_eq!(schedule.adjusted_dates(), &[start, payment]);
        let rate = FixedCouponRate::new(1.0, context.clone())?;
        let leg = Leg::fixed_rate(&schedule, 1.0, &rate, 1)?;
        let CashFlow::FixedRateCoupon(coupon) = &leg.flows()[0] else {
            return Err("expected fixed coupon".into());
        };
        assert_eq!(coupon.start(), start);
        assert_eq!(coupon.end(), end);
        assert_eq!(coupon.payment_date(), payment);
        assert_eq!(coupon.coupon_rate().accrual(), &context);
        assert_eq!(coupon.accrual_fraction().to_bits(), full_time.to_bits());
        assert_eq!(coupon.amount().to_bits(), full_time.to_bits());
        assert_eq!(
            coupon
                .accrued_amount(partial, Inclusion::Exclude)?
                .to_bits(),
            partial_time.to_bits()
        );
        assert_eq!(
            leg.accrued_amount(partial, Inclusion::Exclude, 1)?
                .to_bits(),
            partial_time.to_bits()
        );
        assert!(matches!(
            leg.accrued_amount(partial, Inclusion::Exclude, 0),
            Err(CashFlowError::AccrualWorkLimitExceeded {
                index: Some(0),
                used: 0,
                required_at_least: 1,
                ..
            })
        ));
        assert_eq!(
            leg.accrued_amount(end, Inclusion::Exclude, 0)?.to_bits(),
            full_time.to_bits()
        );
        assert_eq!(
            leg.accrued_amount(payment, Inclusion::Include, 0)?
                .to_bits(),
            full_time.to_bits()
        );
        assert_eq!(
            leg.accrued_amount(payment, Inclusion::Exclude, 0)?
                .to_bits(),
            0
        );
        let discount = InterestRate::new(0.05, context, Compounding::SIMPLE)?;
        let expected = discount.present_value_between(full_time, start, payment)?;
        let actual = leg.net_present_value(start, &discount, Inclusion::Exclude, 2)?;
        assert_close(actual, expected, factor_tolerance(actual, expected));
        assert!(matches!(
            leg.net_present_value(start, &discount, Inclusion::Exclude, 1),
            Err(CashFlowError::AccrualWorkLimitExceeded {
                index: Some(0),
                used: 1,
                required_at_least: 2,
                ..
            })
        ));
    }
    Ok(())
}

#[test]
fn full_civil_span_queries_cost_one_unit_even_for_zero_rates_and_amounts()
-> Result<(), Box<dyn Error>> {
    for context in [
        AccrualConvention::from(Thirty360Us::new(false)),
        Thirty360Us::new(true).into(),
        DayCount::ActualActualAfb.into(),
    ] {
        let discount = InterestRate::new(0.0, context.clone(), Compounding::CONTINUOUS)?;
        let leg = Leg::new(vec![SimpleCashFlow::new(Date::MAX, 0.0)?.into()], 1)?;
        assert_eq!(
            leg.net_present_value(Date::MIN, &discount, Inclusion::Exclude, 2)?
                .to_bits(),
            0
        );
        assert!(matches!(
            leg.net_present_value(Date::MIN, &discount, Inclusion::Exclude, 1),
            Err(CashFlowError::AccrualWorkLimitExceeded {
                index: Some(0),
                used: 1,
                required_at_least: 2,
                ..
            })
        ));
        assert!(matches!(
            leg.net_present_value(Date::MIN, &discount, Inclusion::Exclude, 0),
            Err(CashFlowError::AccrualWorkLimitExceeded {
                index: None,
                used: 0,
                required_at_least: 1,
                ..
            })
        ));
        let coupon = FixedRateCoupon::new(
            Date::MIN,
            Date::MAX,
            Date::MAX,
            0.0,
            FixedCouponRate::new(0.0, context.clone())?,
        )?;
        assert_eq!(coupon.coupon_rate().accrual(), &context);
        let leg = Leg::new(vec![coupon.into()], 1)?;
        let partial = Date::from_ymd(9999, 12, 30)?;
        assert_eq!(
            leg.accrued_amount(partial, Inclusion::Exclude, 1)?
                .to_bits(),
            0
        );
        assert!(matches!(
            leg.accrued_amount(partial, Inclusion::Exclude, 0),
            Err(CashFlowError::AccrualWorkLimitExceeded {
                index: Some(0),
                used: 0,
                required_at_least: 1,
                ..
            })
        ));
    }
    Ok(())
}
