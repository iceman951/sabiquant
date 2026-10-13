//! Per-call query budgets, original error order and shared contextual storage.

use sabiquant::calendar::{BusinessDayConvention as Bdc, Calendar, EndOfMonth};
use sabiquant::cashflows::{
    CashFlow, CashFlowError, FixedCouponRate, FixedRateCoupon, Leg,
    PaymentDateInclusion as Inclusion, SimpleCashFlow,
};
use sabiquant::daycount::{ActualActualIcma, Business252, DeterminationPeriod, IcmaError};
use sabiquant::error::ValueError;
use sabiquant::rates::{Compounding, InterestRate, RateError};
use sabiquant::schedule::{
    DateGenerationRule as Rule, ScheduleBuilder, ScheduleCalendar, StubPolicy,
};
use sabiquant::time::{Date, Frequency, Period, TimeUnit};
use std::error::Error;

fn daily(start: Date, days: u32) -> Result<ActualActualIcma, Box<dyn Error>> {
    let mut periods = Vec::new();
    for offset in 0..days {
        periods.push(DeterminationPeriod::new(
            start.checked_add_days(i64::from(offset))?,
            start.checked_add_days(i64::from(offset) + 1)?,
        )?);
    }
    Ok(ActualActualIcma::new(periods, Frequency::Daily)?)
}

#[test]
fn icma_charges_each_original_horizon_and_preserves_first_failure() -> Result<(), Box<dyn Error>> {
    let start = Date::from_ymd(2026, 1, 1)?;
    let third = start.checked_add_days(3)?;
    let outside = start.checked_add_days(11)?;
    let context = daily(start, 10)?;
    let leg = Leg::new(
        vec![
            SimpleCashFlow::new(start.checked_add_days(-1)?, 1.0)?.into(),
            SimpleCashFlow::new(third, f64::MAX)?.into(),
            SimpleCashFlow::new(outside, 0.0)?.into(),
        ],
        3,
    )?;
    let negative = InterestRate::new(-0.5, context.clone(), Compounding::SIMPLE)?;
    assert!(matches!(
        leg.net_present_value(start, &negative, Inclusion::Exclude, 4),
        Err(CashFlowError::AccrualWorkLimitExceeded {
            index: Some(1),
            maximum: 4,
            used: 1,
            required_at_least: 5,
            phase: "discount horizon",
            ..
        })
    ));
    assert!(matches!(
        leg.net_present_value(start, &negative, Inclusion::Exclude, u64::MAX),
        Err(CashFlowError::Rate {
            index: Some(1),
            cause: RateError::Value(ValueError::ResultNotFinite { .. }),
            ..
        })
    ));
    // A later uncovered zero amount must not move ahead of the first factor/product error.
    let zero = InterestRate::new(0.0, context, Compounding::SIMPLE)?;
    assert!(matches!(
        leg.net_present_value(start, &zero, Inclusion::Exclude, 15),
        Err(CashFlowError::AccrualWorkLimitExceeded {
            index: Some(2),
            used: 5,
            required_at_least: 16,
            ..
        })
    ));
    assert!(matches!(
        leg.net_present_value(start, &zero, Inclusion::Exclude, 16),
        Err(CashFlowError::Rate {
            index: Some(2),
            cause: RateError::Icma(IcmaError::OutOfCoverage { .. }),
            ..
        })
    ));
    Ok(())
}

#[test]
fn business252_budget_counts_calendar_span_including_zero_amount() -> Result<(), Box<dyn Error>> {
    let friday = Date::from_ymd(2026, 1, 2)?;
    let monday = friday.checked_add_days(3)?;
    let quote = InterestRate::new(
        0.0,
        Business252::new(Calendar::weekends_only()),
        Compounding::SIMPLE,
    )?;
    let leg = Leg::new(vec![SimpleCashFlow::new(monday, -0.0)?.into()], 1)?;
    assert!(matches!(
        leg.net_present_value(friday, &quote, Inclusion::Exclude, 4),
        Err(CashFlowError::AccrualWorkLimitExceeded {
            index: Some(0),
            used: 1,
            required_at_least: 5,
            ..
        })
    ));
    assert_eq!(
        leg.net_present_value(friday, &quote, Inclusion::Exclude, 5)?
            .to_bits(),
        0
    );
    assert_eq!(
        leg.net_present_value(monday, &quote, Inclusion::Exclude, 1)?
            .to_bits(),
        0
    );
    assert_eq!(
        leg.net_present_value(monday, &quote, Inclusion::Include, 2)?
            .to_bits(),
        0
    );
    Ok(())
}

#[test]
fn overlapping_manual_accruals_share_a_per_call_budget_but_cached_paths_cost_zero()
-> Result<(), Box<dyn Error>> {
    let start = Date::from_ymd(2026, 1, 1)?;
    let end = start.checked_add_days(10)?;
    let payment = end.checked_add_days(2)?;
    let quote = FixedCouponRate::new(1.0, daily(start, 10)?)?;
    let coupon = FixedRateCoupon::new(start, end, payment, 365.0, quote)?;
    let leg = Leg::new(vec![coupon.clone().into(), coupon.clone().into()], 2)?;
    let middle = start.checked_add_days(5)?;
    assert!(matches!(
        leg.accrued_amount(middle, Inclusion::Include, 11),
        Err(CashFlowError::AccrualWorkLimitExceeded {
            index: Some(1),
            used: 6,
            required_at_least: 12,
            ..
        })
    ));
    let single = coupon.accrued_amount(middle, Inclusion::Include)?;
    assert_eq!(
        leg.accrued_amount(middle, Inclusion::Include, 12)?
            .to_bits(),
        (single * 2.0).to_bits()
    );
    for at in [Date::MIN, start, payment.checked_add_days(1)?] {
        assert_eq!(leg.accrued_amount(at, Inclusion::Include, 0)?.to_bits(), 0);
    }
    assert_eq!(
        leg.accrued_amount(end, Inclusion::Include, 0)?.to_bits(),
        (coupon.amount() * 2.0).to_bits()
    );
    assert_eq!(
        leg.accrued_amount(payment, Inclusion::Exclude, 0)?
            .to_bits(),
        0
    );
    Ok(())
}

#[test]
fn empty_legs_require_only_the_discount_anchor_query() -> Result<(), Box<dyn Error>> {
    let start = Date::from_ymd(2026, 1, 1)?;
    let quote = InterestRate::new(0.0, daily(start, 1)?, Compounding::SIMPLE)?;
    let leg = Leg::new(Vec::new(), 0)?;
    assert!(matches!(
        leg.net_present_value(start, &quote, Inclusion::Exclude, 0),
        Err(CashFlowError::AccrualWorkLimitExceeded {
            index: None,
            maximum: 0,
            used: 0,
            required_at_least: 1,
            ..
        })
    ));
    assert_eq!(
        leg.net_present_value(start, &quote, Inclusion::Exclude, 1)?
            .to_bits(),
        0
    );
    assert!(matches!(
        leg.net_present_value(Date::MIN, &quote, Inclusion::Exclude, 1),
        Err(CashFlowError::Rate {
            index: None,
            phase: "valuation anchor",
            cause: RateError::Icma(IcmaError::OutOfCoverage { .. }),
            ..
        })
    ));
    assert_eq!(
        leg.accrued_amount(Date::MIN, Inclusion::Include, 0)?
            .to_bits(),
        0
    );
    Ok(())
}

#[test]
fn maximum_scalar_leg_retains_every_duplicate_and_exact_count_budget() -> Result<(), Box<dyn Error>>
{
    let flow = CashFlow::from(SimpleCashFlow::new(Date::MIN, 1.0)?);
    let count = Leg::MAX_FLOWS as usize;
    let leg = Leg::new(vec![flow; count], Leg::MAX_FLOWS)?;
    assert_eq!(leg.flows().len(), count);
    let quote = InterestRate::new(
        0.0,
        sabiquant::daycount::DayCount::Actual360,
        Compounding::SIMPLE,
    )?;
    let required = u64::from(Leg::MAX_FLOWS) + 1;
    assert!(
        matches!(leg.net_present_value(Date::MIN, &quote, Inclusion::Include, required-1),
        Err(CashFlowError::AccrualWorkLimitExceeded { index:Some(index),used,required_at_least, .. })
            if index==count-1 && used==required-1 && required_at_least==required)
    );
    assert_eq!(
        leg.net_present_value(Date::MIN, &quote, Inclusion::Include, required)?
            .to_bits(),
        f64::from(Leg::MAX_FLOWS).to_bits()
    );
    assert_eq!(
        leg.net_present_value(Date::MIN, &quote, Inclusion::Exclude, 1)?
            .to_bits(),
        0
    );
    assert_eq!(
        leg.accrued_amount(Date::MIN, Inclusion::Include, 0)?
            .to_bits(),
        0
    );
    Ok(())
}

#[test]
fn large_schedule_leg_shares_the_complete_icma_definition() -> Result<(), Box<dyn Error>> {
    const DAYS: u32 = 200_000;
    let schedule = ScheduleBuilder::new(
        Date::MIN,
        Date::MIN.checked_add_days(i64::from(DAYS))?,
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?
    .build_recurrent(
        Period::new(1, TimeUnit::Days),
        Rule::Forward,
        StubPolicy::None,
        EndOfMonth::Ignore,
        DAYS + 1,
    )?;
    let quote = FixedCouponRate::new(1.0, schedule.build_icma_context(Frequency::Daily, DAYS)?)?;
    assert!(
        matches!(Leg::fixed_rate(&schedule, f64::NAN, &quote, DAYS-1),
        Err(CashFlowError::FlowLimitExceeded { maximum,required, .. }) if maximum==DAYS-1 && required==DAYS as usize)
    );
    let leg = Leg::fixed_rate(&schedule, 365.0, &quote, DAYS)?;
    assert_eq!(leg.flows().len(), DAYS as usize);
    for (index, flow) in leg.flows().iter().enumerate() {
        if let CashFlow::FixedRateCoupon(coupon) = flow {
            assert!(std::ptr::eq(
                coupon.coupon_rate().accrual(),
                quote.accrual()
            ));
            assert_eq!(coupon.start(), schedule.contractual_dates()[index]);
            assert_eq!(coupon.end(), schedule.contractual_dates()[index + 1]);
            assert_eq!(coupon.payment_date(), schedule.adjusted_dates()[index + 1]);
        } else {
            return Err("fixed factory returned a simple flow".into());
        }
    }
    Ok(())
}

#[test]
fn equal_horizon_signed_cancellation_preserves_the_small_payment_exactly()
-> Result<(), Box<dyn Error>> {
    let start = Date::from_ymd(2026, 1, 1)?;
    let payment = Date::from_ymd(2027, 1, 1)?;
    let mut methods = vec![Compounding::SIMPLE, Compounding::CONTINUOUS];
    for frequency in Frequency::ALL {
        if frequency == Frequency::Once {
            continue;
        }
        methods.push(Compounding::compounded(frequency)?);
        methods.push(Compounding::simple_then_compounded(frequency)?);
        methods.push(Compounding::compounded_then_simple(frequency)?);
    }
    assert_eq!(methods.len(), 32);
    for method in methods {
        for rate in [-0.03, 0.0, 0.03] {
            let quote =
                InterestRate::new(rate, sabiquant::daycount::DayCount::Actual365Fixed, method)?;
            let discount = quote.discount_factor_between(start, payment)?;
            // Opposite identical rounded products cancel EXACTLY. Multiplying
            // the remaining amount 1 by the supplied discount is exact, so a
            // gross-conditioned oracle allowance must not hide a lost residual.
            for amounts in [
                [1e16, 1.0, -1e16],
                [1e16, -1e16, 1.0],
                [1.0, 1e16, -1e16],
                [1.0, -1e16, 1e16],
                [-1e16, 1e16, 1.0],
                [-1e16, 1.0, 1e16],
            ] {
                let flows = amounts
                    .into_iter()
                    .map(|amount| SimpleCashFlow::new(payment, amount).map(CashFlow::from))
                    .collect::<Result<Vec<_>, _>>()?;
                let leg = Leg::new(flows, 3)?;
                assert_eq!(
                    leg.net_present_value(start, &quote, Inclusion::Exclude, 4)?
                        .to_bits(),
                    discount.to_bits(),
                    "{method:?} rate {rate} amounts {amounts:?}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn every_hybrid_frequency_crosses_its_switch_without_adjacent_factor_chaining()
-> Result<(), Box<dyn Error>> {
    let boundaries = [
        Date::from_ymd(2024, 1, 1)?,
        Date::from_ymd(2025, 1, 1)?,
        Date::from_ymd(2026, 1, 1)?,
        Date::from_ymd(2027, 1, 1)?,
    ];
    let context = ActualActualIcma::new(
        boundaries
            .windows(2)
            .map(|pair| DeterminationPeriod::new(pair[0], pair[1]))
            .collect::<Result<Vec<_>, _>>()?,
        Frequency::Annual,
    )?;
    let valuation = boundaries[0];
    let first = valuation.checked_add_days(1)?;
    // Leap-year annual ICMA gives 1/366: below even Daily's 1/365 switch.
    // Thus every named hybrid genuinely uses its first branch on this flow.
    for frequency in Frequency::ALL {
        let Some(count) = frequency.events_per_year() else {
            continue;
        };
        let last = valuation.checked_add_days(i64::from((732 / count).max(3)))?;
        let leg = Leg::new(
            vec![
                SimpleCashFlow::new(first, 1.0)?.into(),
                SimpleCashFlow::new(last, 1.0)?.into(),
            ],
            2,
        )?;
        for method in [
            Compounding::simple_then_compounded(frequency)?,
            Compounding::compounded_then_simple(frequency)?,
        ] {
            let quote = InterestRate::new(0.1, context.clone(), method)?;
            let early = quote.discount_factor_between(valuation, first)?;
            let direct = quote.discount_factor_between(valuation, last)?;
            let chained = early * quote.discount_factor_between(first, last)?;
            assert!((direct - chained).abs() > 16.0 * f64::EPSILON, "{method:?}");
            // Two finite positive returned factors need exactly one addition;
            // no coupon/product uncertainty or gross tolerance is involved.
            assert_eq!(
                leg.net_present_value(valuation, &quote, Inclusion::Exclude, 32)?
                    .to_bits(),
                (early + direct).to_bits(),
                "{method:?}"
            );
        }
    }
    Ok(())
}

#[test]
fn schedule_factory_failures_preserve_the_original_interval_and_cause() -> Result<(), Box<dyn Error>>
{
    let start = Date::from_ymd(2026, 1, 31)?;
    let middle = Date::from_ymd(2026, 2, 28)?;
    let end = Date::from_ymd(2026, 3, 31)?;
    let schedule = ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?
    .build_recurrent(
        Period::new(1, TimeUnit::Months),
        Rule::Forward,
        StubPolicy::None,
        EndOfMonth::Preserve,
        3,
    )?;
    assert_eq!(schedule.contractual_dates(), [start, middle, end]);
    let plain = FixedCouponRate::new(12.0, sabiquant::daycount::DayCount::Actual365Fixed)?;
    // The 28-day first amount is finite; the 31-day second amount overflows.
    let first = FixedRateCoupon::new(start, middle, middle, f64::MAX, plain.clone())?;
    assert!(first.amount().is_finite());
    assert!(matches!(
        Leg::fixed_rate(&schedule, f64::MAX, &plain, 2),
        Err(CashFlowError::Value {
            index: Some(1),
            phase: "coupon amount",
            cause: ValueError::ResultNotFinite { .. },
            ..
        })
    ));
    for notional in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            Leg::fixed_rate(&schedule, notional, &plain, 2),
            Err(CashFlowError::Value {
                index: Some(0),
                phase: "coupon notional",
                cause: ValueError::NotFinite {
                    parameter: "notional",
                    ..
                },
                ..
            })
        ));
    }
    let partial_context = ActualActualIcma::new(
        vec![DeterminationPeriod::new(start, middle)?],
        Frequency::Monthly,
    )?;
    let zero = FixedCouponRate::new(0.0, partial_context)?;
    // Zero scalars do not waive the uncovered second interval; the factory
    // returns no partial leg and retains the true interval position/source.
    let error = Leg::fixed_rate(&schedule, 0.0, &zero, 2).expect_err("second interval uncovered");
    assert!(
        matches!(&error,CashFlowError::Rate { index:Some(1),phase:"coupon accrual",
        cause:RateError::Icma(IcmaError::OutOfCoverage { start:a,end:b, .. }), .. }
            if *a==middle && *b==end)
    );
    let root = sabiquant::error::Error::from(error.clone());
    assert_eq!(
        root.source()
            .and_then(|e| e.downcast_ref::<CashFlowError>()),
        Some(&error)
    );
    assert!(
        root.source()
            .and_then(|e| e.source())
            .is_some_and(|e| e.is::<RateError>())
    );
    assert!(
        root.source()
            .and_then(|e| e.source())
            .and_then(|e| e.source())
            .is_some_and(|e| e.is::<IcmaError>())
    );
    Ok(())
}
