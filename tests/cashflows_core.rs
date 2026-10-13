//! Independent hand cases for analytical cash-flow values and composition.

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, EndOfMonth, TargetCalendar, TargetPolicy,
};
use sabiquant::cashflows::{
    CashFlow, CashFlowError, FixedCouponRate, FixedRateCoupon, Leg,
    PaymentDateInclusion as Inclusion, SimpleCashFlow,
};
use sabiquant::daycount::{
    ActualActualIcma, Business252, DayCount, DayCountError, DeterminationPeriod, IcmaError,
    ThirtyE360Isda,
};
use sabiquant::error::{Error as CrateError, NonFinite, ValueError};
use sabiquant::math::SumError;
use sabiquant::rates::{AccrualConvention, Compounding, InterestRate, RateError};
use sabiquant::schedule::{
    DateGenerationRule as Rule, ExplicitStubDates, ScheduleBuilder, ScheduleCalendar, StubPolicy,
};
use sabiquant::time::{Date, Frequency, Period, TimeUnit};
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Debug;

#[expect(clippy::expect_used, reason = "hand-audited Gregorian fixture dates")]
fn d(text: &str) -> Date {
    text.parse().expect("valid ISO fixture date")
}

#[expect(clippy::expect_used, reason = "the case explicitly requires rejection")]
fn rejected<T: Debug>(result: Result<T, CashFlowError>) -> CashFlowError {
    result.expect_err("the stated public precondition must be rejected")
}

fn simple(payment: Date, amount: f64) -> Result<CashFlow, CashFlowError> {
    Ok(SimpleCashFlow::new(payment, amount)?.into())
}

fn icma(boundaries: &[Date], frequency: Frequency) -> Result<ActualActualIcma, IcmaError> {
    let periods = boundaries
        .windows(2)
        .map(|pair| DeterminationPeriod::new(pair[0], pair[1]))
        .collect::<Result<Vec<_>, _>>()?;
    ActualActualIcma::new(periods, frequency)
}

fn compoundings() -> Result<Vec<Compounding>, RateError> {
    let mut methods = vec![Compounding::SIMPLE, Compounding::CONTINUOUS];
    for frequency in Frequency::ALL {
        if frequency.events_per_year().is_some() {
            methods.extend([
                Compounding::compounded(frequency)?,
                Compounding::simple_then_compounded(frequency)?,
                Compounding::compounded_then_simple(frequency)?,
            ]);
        }
    }
    assert_eq!(methods.len(), 32);
    Ok(methods)
}

fn assert_work(
    error: CashFlowError,
    index: Option<usize>,
    phase: &'static str,
    maximum: u64,
    used: u64,
    required: u64,
) {
    assert_eq!(error.kind(), "cashflow.accrual_work_limit_exceeded");
    assert!(
        matches!(&error,CashFlowError::AccrualWorkLimitExceeded {
            index:i,phase:p,maximum:m,used:u,required_at_least:r,..
        } if (*i,*p,*m,*u,*r)==(index,phase,maximum,used,required)),
        "{error:?}"
    );
    assert!(error.source().is_none());
}

// Four ULPs of the positive ICMA fraction are at most 4*epsilon*abs(YF).
// Exact scalar multiplication, final rounding and rounded rational oracle
// need two additional epsilon units on these bounded hand examples.
fn assert_rational_accrual(actual: f64, expected: f64) {
    assert!(actual.is_finite() && expected.is_finite());
    assert!(
        (actual - expected).abs() <= 6.0 * f64::EPSILON * actual.abs().max(expected.abs()),
        "actual {actual:?}, rational oracle {expected:?}"
    );
}

#[test]
fn signed_simple_amounts_have_explicit_dates_and_canonical_zero() -> Result<(), Box<dyn Error>> {
    assert_eq!(Inclusion::ALL, [Inclusion::Include, Inclusion::Exclude]);
    assert_eq!(HashSet::from(Inclusion::ALL).len(), 2);
    let payment = d("2026-02-28"); // No implicit weekend adjustment.
    for amount in [
        0.0,
        -0.0,
        1.0,
        -1.0,
        f64::MAX,
        -f64::MAX,
        f64::from_bits(1),
        -f64::from_bits(1),
    ] {
        let flow = SimpleCashFlow::new(payment, amount)?;
        assert_eq!(flow.payment_date(), payment);
        assert_eq!(
            flow.amount().to_bits(),
            if amount == 0.0 { 0 } else { amount.to_bits() }
        );
        let wrapped = CashFlow::from(flow);
        assert_eq!(wrapped.payment_date(), payment);
        assert_eq!(wrapped.amount().to_bits(), flow.amount().to_bits());
        for as_of in [Date::MIN, payment, Date::MAX] {
            for inclusion in Inclusion::ALL {
                assert_eq!(wrapped.accrued_amount(as_of, inclusion)?.to_bits(), 0);
            }
        }
    }
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = rejected(SimpleCashFlow::new(payment, bad));
        assert!(matches!(
            &error,
            CashFlowError::Value {
                index: None,
                phase: "simple amount",
                cause: ValueError::NotFinite {
                    parameter: "amount",
                    ..
                },
                ..
            }
        ));
        assert_eq!(error.kind(), "cashflow.value");
        assert!(
            error
                .source()
                .is_some_and(|e| e.downcast_ref::<ValueError>().is_some())
        );
    }
    Ok(())
}

#[test]
fn coupon_quotes_keep_all_context_and_accept_signed_simple_rates() -> Result<(), Box<dyn Error>> {
    let context = icma(&[d("2026-01-01"), d("2026-07-01")], Frequency::Semiannual)?;
    for rate in [0.0, -0.0, 1.0, -1.0, f64::MAX, -f64::MAX] {
        let quote = FixedCouponRate::new(rate, context.clone())?;
        assert_eq!(
            quote.rate().to_bits(),
            if rate == 0.0 { 0 } else { rate.to_bits() }
        );
        assert_eq!(quote.accrual(), &AccrualConvention::Icma(context.clone()));
        let cloned = quote.clone();
        assert_eq!(quote, cloned);
        // Resource check of the adopted shared-storage guarantee, not a
        // financial definition of equality by pointer identity.
        assert!(std::ptr::eq(quote.accrual(), cloned.accrual()));
        let independent = FixedCouponRate::new(rate, context.clone())?;
        assert_eq!(quote, independent);
    }
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            rejected(FixedCouponRate::new(bad, DayCount::Actual360)),
            CashFlowError::Value {
                index: None,
                phase: "coupon quote",
                cause: ValueError::NotFinite {
                    parameter: "coupon rate",
                    ..
                },
                ..
            }
        ));
    }
    assert_ne!(
        FixedCouponRate::new(1.0, DayCount::Actual360)?,
        FixedCouponRate::new(1.0, DayCount::Actual365Fixed)?
    );
    Ok(())
}

#[test]
fn coupon_construction_orders_raw_notional_and_context_checks() -> Result<(), Box<dyn Error>> {
    let start = d("2026-01-01");
    let end = d("2026-07-01");
    let quote = FixedCouponRate::new(0.0, icma(&[start, end], Frequency::Semiannual)?)?;
    for (a, z) in [(start, start), (end, start)] {
        assert!(
            matches!(rejected(FixedRateCoupon::new(a,z,Date::MIN,f64::NAN,quote.clone())),
            CashFlowError::InvalidAccrualPeriod { start:s,end:e,.. } if (s,e)==(a,z))
        );
    }
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            rejected(FixedRateCoupon::new(
                Date::MIN,
                end,
                Date::MAX,
                bad,
                quote.clone()
            )),
            CashFlowError::Value {
                index: None,
                phase: "coupon notional",
                cause: ValueError::NotFinite {
                    parameter: "notional",
                    ..
                },
                ..
            }
        ));
    }
    for notional in [0.0, -0.0, 1.0] {
        let error = rejected(FixedRateCoupon::new(
            Date::MIN,
            end,
            Date::MAX,
            notional,
            quote.clone(),
        ));
        assert!(
            matches!(&error,CashFlowError::Rate { index:None,phase:"coupon accrual",
            cause:RateError::Icma(IcmaError::OutOfCoverage { start:s,end:e,.. }),.. }
            if (*s,*e)==(Date::MIN,end))
        );
        assert!(
            error
                .source()
                .is_some_and(|e| e.downcast_ref::<RateError>().is_some())
        );
    }
    Ok(())
}

#[test]
fn complete_coupon_terms_and_amount_signs_survive_early_or_late_payments()
-> Result<(), Box<dyn Error>> {
    let start = d("2026-01-01");
    let end = d("2026-07-01");
    let context = icma(&[start, end], Frequency::Semiannual)?;
    for notional in [0.0, -0.0, 8.0, -8.0] {
        for rate in [0.0, -0.0, 0.5, -0.5] {
            let quote = FixedCouponRate::new(rate, context.clone())?;
            for payment in [Date::MIN, start, end, Date::MAX] {
                let coupon = FixedRateCoupon::new(start, end, payment, notional, quote.clone())?;
                assert_eq!(
                    (coupon.start(), coupon.end(), coupon.payment_date()),
                    (start, end, payment)
                );
                assert_eq!(
                    coupon.notional().to_bits(),
                    if notional == 0.0 {
                        0
                    } else {
                        notional.to_bits()
                    }
                );
                assert_eq!(coupon.coupon_rate(), &quote);
                assert_eq!(coupon.accrual_fraction().to_bits(), 0.5_f64.to_bits());
                let expected = notional * rate * 0.5; // All nonzero operations are exact.
                assert_eq!(
                    coupon.amount().to_bits(),
                    if expected == 0.0 {
                        0
                    } else {
                        expected.to_bits()
                    }
                );
                let flow = CashFlow::from(coupon.clone());
                assert_eq!(flow.amount().to_bits(), coupon.amount().to_bits());
                assert_eq!(flow.payment_date(), payment);
                assert_eq!(flow, CashFlow::FixedRateCoupon(coupon));
            }
        }
    }
    let quote = FixedCouponRate::new(0.0, DayCount::Actual360)?;
    let a = FixedRateCoupon::new(start, end, end, 1.0, quote.clone())?;
    let b = FixedRateCoupon::new(start, end, end, 2.0, quote)?;
    assert_eq!(a.amount().to_bits(), b.amount().to_bits());
    assert_ne!(a, b); // Definition equality includes the original notional.
    Ok(())
}

#[test]
fn ordered_zero_accrual_does_not_form_an_overflowing_intermediate_product()
-> Result<(), Box<dyn Error>> {
    let start = d("2026-01-30");
    let end = d("2026-01-31");
    let coupon = FixedRateCoupon::new(
        start,
        end,
        Date::MAX,
        f64::MAX,
        FixedCouponRate::new(f64::MAX, DayCount::ThirtyE360)?,
    )?;
    assert_eq!(coupon.accrual_fraction().to_bits(), 0);
    assert_eq!(coupon.amount().to_bits(), 0);
    assert_eq!(coupon.accrued_amount(end, Inclusion::Include)?.to_bits(), 0);
    Ok(())
}

#[test]
fn distinct_weekend_dates_can_have_zero_accrual_and_still_require_query_work()
-> Result<(), Box<dyn Error>> {
    let start = d("2024-01-06"); // Saturday.
    let sunday = d("2024-01-07");
    let quote = FixedCouponRate::new(f64::MAX, Business252::new(Calendar::weekends_only()))?;
    let zero = FixedRateCoupon::new(start, sunday, Date::MAX, f64::MAX, quote)?;
    assert_eq!(zero.accrual_fraction().to_bits(), 0);
    assert_eq!(zero.amount().to_bits(), 0);
    let coupon = FixedRateCoupon::new(
        start,
        d("2024-01-10"),
        Date::MAX,
        252.0,
        FixedCouponRate::new(1.0, Business252::new(Calendar::weekends_only()))?,
    )?;
    assert_eq!(
        coupon.accrued_amount(sunday, Inclusion::Include)?.to_bits(),
        0
    );
    let leg = Leg::new(vec![coupon.into()], 1)?;
    assert_work(
        rejected(leg.accrued_amount(sunday, Inclusion::Include, 1)),
        Some(0),
        "partial accrual",
        1,
        0,
        2,
    );
    assert_eq!(
        leg.accrued_amount(sunday, Inclusion::Include, 2)?.to_bits(),
        0
    );
    let discount = InterestRate::new(
        0.125,
        Business252::new(Calendar::weekends_only()),
        Compounding::SIMPLE,
    )?;
    let payment = Leg::new(vec![simple(sunday, 8.0)?], 1)?;
    assert_eq!(
        payment
            .net_present_value(start, &discount, Inclusion::Include, 3)?
            .to_bits(),
        8.0_f64.to_bits()
    );
    Ok(())
}

#[test]
fn real_partial_product_underflow_preserves_sign_original_index_and_work_order()
-> Result<(), Box<dyn Error>> {
    let cases = [
        (
            d("2023-01-01"),
            d("2025-01-01"),
            d("2025-01-02"),
            d("2023-01-02"),
            AccrualConvention::from(DayCount::Actual365Fixed),
            2_u64,
            1_u64,
        ),
        (
            d("2026-01-01"),
            d("2026-01-03"),
            d("2026-01-04"),
            d("2026-01-02"),
            AccrualConvention::from(icma(
                &[d("2026-01-01"), d("2026-01-03")],
                Frequency::Annual,
            )?),
            1,
            2,
        ),
    ];
    for (start, end, payment, as_of, context, full_bits, charge) in cases {
        for sign in [1.0, -1.0] {
            let coupon = FixedRateCoupon::new(
                start,
                end,
                payment,
                sign * f64::from_bits(1),
                FixedCouponRate::new(1.0, context.clone())?,
            )?;
            assert_eq!(
                coupon.amount().to_bits(),
                (sign * f64::from_bits(full_bits)).to_bits()
            );
            let error = rejected(coupon.accrued_amount(as_of, Inclusion::Include));
            assert!(
                matches!(&error,CashFlowError::Value { index:None,phase:"accrued amount",
                cause:ValueError::ResultOutOfDomain { quantity:"accrued amount",value,domain,.. },.. }
                if value.to_bits()==(sign*0.0).to_bits() && domain.contains(sign)
                    && !domain.contains(-sign) && !domain.contains(0.0))
            );
            let previous = start.checked_add_days(-1)?;
            let leg = Leg::new(
                vec![simple(previous, 0.0)?, simple(as_of, 0.0)?, coupon.into()],
                3,
            )?;
            assert_work(
                rejected(leg.accrued_amount(as_of, Inclusion::Exclude, charge - 1)),
                Some(2),
                "partial accrual",
                charge - 1,
                0,
                charge,
            );
            let indexed = rejected(leg.accrued_amount(as_of, Inclusion::Exclude, charge));
            assert!(
                matches!(&indexed,CashFlowError::Value { index:Some(2),phase:"accrued amount",
                cause:ValueError::ResultOutOfDomain { quantity:"accrued amount",value,.. },.. }
                if value.to_bits()==(sign*0.0).to_bits())
            );
            assert!(
                indexed
                    .source()
                    .is_some_and(|e| e.downcast_ref::<ValueError>().is_some())
            );
            let root = CrateError::from(indexed.clone());
            assert!(
                root.source()
                    .is_some_and(|e| e.downcast_ref::<CashFlowError>() == Some(&indexed))
            );
        }
    }
    Ok(())
}

#[test]
fn exact_public_coupon_products_recover_range_and_round_midpoints() -> Result<(), Box<dyn Error>> {
    let start = d("2026-01-01");
    let one_year = d("2027-01-01");
    let half = d("2026-07-01");
    let max_coupon = FixedRateCoupon::new(
        start,
        half,
        half,
        f64::MAX,
        FixedCouponRate::new(2.0, icma(&[start, half], Frequency::Semiannual)?)?,
    )?;
    assert_eq!(max_coupon.amount().to_bits(), f64::MAX.to_bits());
    let tiny_coupon = FixedRateCoupon::new(
        start,
        d("2028-01-01"),
        Date::MAX,
        f64::from_bits(1),
        FixedCouponRate::new(0.5, DayCount::Actual365Fixed)?,
    )?;
    assert_eq!(tiny_coupon.accrual_fraction().to_bits(), 2.0_f64.to_bits());
    assert_eq!(tiny_coupon.amount().to_bits(), 1);
    for (notional, rate, expected) in [
        (
            1.0_f64.next_up(),
            1.5,
            f64::from_bits(1.5_f64.to_bits() + 2),
        ),
        (
            f64::from_bits(1.0_f64.to_bits() + 2),
            1.25,
            f64::from_bits(1.25_f64.to_bits() + 2),
        ),
        (f64::from_bits(1), 1.5, f64::from_bits(2)),
        (f64::from_bits(1), 2.5, f64::from_bits(2)),
    ] {
        for sign in [1.0, -1.0] {
            let coupon = FixedRateCoupon::new(
                start,
                one_year,
                one_year,
                sign * notional,
                FixedCouponRate::new(rate, DayCount::Actual365Fixed)?,
            )?;
            assert_eq!(coupon.amount().to_bits(), (sign * expected).to_bits());
        }
    }
    Ok(())
}

#[test]
fn coupon_range_failures_retain_sign_quantity_and_typed_source() -> Result<(), Box<dyn Error>> {
    let start = d("2026-01-01");
    let end = d("2027-01-01");
    for sign in [1.0, -1.0] {
        let error = rejected(FixedRateCoupon::new(
            start,
            end,
            end,
            sign * f64::from_bits(1),
            FixedCouponRate::new(0.5, DayCount::Actual365Fixed)?,
        ));
        assert!(
            matches!(&error,CashFlowError::Value { index:None,phase:"coupon amount",
            cause:ValueError::ResultOutOfDomain { quantity:"coupon amount",value,domain,.. },.. }
            if value.to_bits()==(sign*0.0).to_bits()
                && domain.contains(sign) && !domain.contains(-sign) && !domain.contains(0.0))
        );
        let overflow = rejected(FixedRateCoupon::new(
            start,
            end,
            end,
            sign * f64::MAX,
            FixedCouponRate::new(2.0, DayCount::Actual365Fixed)?,
        ));
        assert!(
            matches!(&overflow,CashFlowError::Value { index:None,phase:"coupon amount",
            cause:ValueError::ResultNotFinite { quantity:"coupon amount",value,.. },.. }
            if *value==if sign>0.0 { NonFinite::PositiveInfinity } else { NonFinite::NegativeInfinity })
        );
        assert!(
            overflow
                .source()
                .is_some_and(|e| e.downcast_ref::<ValueError>().is_some())
        );
        let root = CrateError::from(overflow.clone());
        assert_eq!(root.kind(), "cashflow.value");
        assert!(
            root.source()
                .is_some_and(|e| e.downcast_ref::<CashFlowError>() == Some(&overflow))
        );
    }
    Ok(())
}

#[test]
fn accrued_coupon_cutoffs_clip_raw_dates_and_payment_eligibility() -> Result<(), Box<dyn Error>> {
    let start = d("2026-01-01");
    let end = d("2026-01-11");
    let late = d("2026-01-15");
    let quote = FixedCouponRate::new(1.0, DayCount::Actual360)?;
    let coupon = FixedRateCoupon::new(start, end, late, 360.0, quote.clone())?;
    for (as_of, expected) in [
        (Date::MIN, 0.0_f64),
        (start, 0.0),
        (d("2026-01-06"), 5.0),
        (end, 10.0),
        (d("2026-01-14"), 10.0),
    ] {
        for inclusion in Inclusion::ALL {
            assert_eq!(
                coupon.accrued_amount(as_of, inclusion)?.to_bits(),
                expected.to_bits()
            );
        }
    }
    assert_eq!(
        coupon.accrued_amount(late, Inclusion::Include)?.to_bits(),
        coupon.amount().to_bits()
    );
    assert_eq!(
        coupon.accrued_amount(late, Inclusion::Exclude)?.to_bits(),
        0
    );
    assert_eq!(
        coupon
            .accrued_amount(Date::MAX, Inclusion::Include)?
            .to_bits(),
        0
    );
    let early = d("2026-01-06");
    let early_coupon = FixedRateCoupon::new(start, end, early, 360.0, quote.clone())?;
    assert_eq!(
        early_coupon
            .accrued_amount(early, Inclusion::Include)?
            .to_bits(),
        5.0_f64.to_bits()
    );
    assert_eq!(
        early_coupon
            .accrued_amount(early, Inclusion::Exclude)?
            .to_bits(),
        0
    );
    assert_eq!(early_coupon.amount().to_bits(), 10.0_f64.to_bits());
    let advance = FixedRateCoupon::new(start, end, Date::MIN, 360.0, quote)?;
    assert_eq!(
        advance
            .accrued_amount(Date::MIN, Inclusion::Include)?
            .to_bits(),
        0
    );
    Ok(())
}

#[test]
fn partial_convention_queries_are_not_elapsed_day_scaling() -> Result<(), Box<dyn Error>> {
    let coupon = FixedRateCoupon::new(
        d("2026-01-29"),
        d("2026-02-28"),
        Date::MAX,
        360.0,
        FixedCouponRate::new(1.0, DayCount::Thirty360BondBasis)?,
    )?;
    assert_eq!(
        coupon
            .accrued_amount(d("2026-01-31"), Inclusion::Include)?
            .to_bits(),
        2.0_f64.to_bits()
    );
    assert!(
        (coupon.amount() * 2.0 / 30.0 - 2.0).abs() > 0.01,
        "calendar-day scaling would change Bond Basis accrual"
    );
    let termination = d("2023-02-28");
    let coupon = FixedRateCoupon::new(
        d("2023-01-31"),
        d("2023-03-31"),
        Date::MAX,
        360.0,
        FixedCouponRate::new(1.0, ThirtyE360Isda::new(termination))?,
    )?;
    assert_eq!(
        coupon
            .accrued_amount(termination, Inclusion::Include)?
            .to_bits(),
        28.0_f64.to_bits()
    );
    assert_eq!(
        coupon.coupon_rate().accrual(),
        &AccrualConvention::ThirtyE360Isda(ThirtyE360Isda::new(termination))
    );
    Ok(())
}

#[test]
fn generated_multi_period_icma_context_is_retained_for_partial_coupons()
-> Result<(), Box<dyn Error>> {
    let start = d("2023-12-15");
    let end = d("2024-03-15");
    let seed = d("2024-02-01");
    let schedule = ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?
    .build_explicit(
        Period::new(1, TimeUnit::Months),
        Rule::Forward,
        ExplicitStubDates::new(Some(seed), Some(seed))?,
        EndOfMonth::Ignore,
        3,
    )?;
    let context = schedule.build_icma_context(Frequency::Monthly, 4)?;
    let quote = FixedCouponRate::new(1.0, context.clone())?;
    let coupon = FixedRateCoupon::new(start, end, Date::MAX, 12.0, quote.clone())?;
    assert_rational_accrual(coupon.amount(), 3.0);
    // 17/31 + one full period + 14/29; independent exact overlap rational.
    assert_rational_accrual(
        coupon.accrued_amount(d("2024-02-15"), Inclusion::Include)?,
        1826.0 / 899.0,
    );
    let leg = Leg::fixed_rate(&schedule, 12.0, &quote, 2)?;
    assert_eq!(leg.flows().len(), 2);
    for flow in leg.flows() {
        assert!(matches!(flow,CashFlow::FixedRateCoupon(value)
            if value.coupon_rate().accrual()==&AccrualConvention::Icma(context.clone())));
    }
    assert_eq!(context.periods().len(), 4); // No context trimming to a coupon.
    Ok(())
}

#[test]
fn leg_count_order_and_duplicate_payment_metadata_are_explicit() -> Result<(), Box<dyn Error>> {
    assert_eq!(Leg::MAX_FLOWS, 3_652_059);
    assert!(Leg::new(vec![], 0)?.flows().is_empty());
    let a = d("2026-01-01");
    let z = d("2026-01-02");
    let reversed = vec![simple(z, 1.0)?, simple(a, 2.0)?];
    assert!(matches!(
        rejected(Leg::new(reversed.clone(), Leg::MAX_FLOWS + 1)),
        CashFlowError::InvalidFlowLimit {
            requested: 3_652_060,
            maximum: 3_652_059,
            ..
        }
    ));
    assert!(matches!(
        rejected(Leg::new(reversed.clone(), 1)),
        CashFlowError::FlowLimitExceeded {
            maximum: 1,
            required: 2,
            ..
        }
    ));
    assert!(matches!(rejected(Leg::new(reversed,2)),
        CashFlowError::PaymentOrder { index:1,previous,current,.. } if (previous,current)==(z,a)));
    let flows = vec![simple(a, 1.0)?, simple(a, -1.0)?, simple(z, 0.0)?];
    let exact = Leg::new(flows.clone(), 3)?;
    assert_eq!(exact.flows(), flows);
    assert_eq!(exact, Leg::new(flows, 7)?); // Cap is not financial metadata.
    Ok(())
}

#[test]
fn schedule_leg_uses_raw_accrual_and_adjusted_payment_for_all_49_convention_pairs()
-> Result<(), Box<dyn Error>> {
    let quote = FixedCouponRate::new(1.0, DayCount::Actual360)?;
    let mut pairs = 0;
    let mut shifted = 0;
    for convention in Bdc::ALL {
        for termination in Bdc::ALL {
            let schedule = ScheduleBuilder::new(
                d("2026-01-15"),
                d("2026-07-31"),
                ScheduleCalendar::Plain(Calendar::weekends_only()),
                convention,
                termination,
            )?
            .build_recurrent(
                Period::new(1, TimeUnit::Months),
                Rule::Forward,
                StubPolicy::Short,
                EndOfMonth::Ignore,
                8,
            )?;
            let count = schedule.contractual_dates().len() - 1;
            let leg = Leg::fixed_rate(&schedule, 360.0, &quote, u32::try_from(count)?)?;
            assert_eq!(leg.flows().len(), count);
            for (index, flow) in leg.flows().iter().enumerate() {
                let raw = schedule.contractual_dates();
                let payment = schedule.adjusted_dates()[index + 1];
                assert_eq!(flow.payment_date(), payment);
                assert!(matches!(flow,CashFlow::FixedRateCoupon(coupon)
                    if (coupon.start(),coupon.end())==(raw[index],raw[index+1])
                    && coupon.coupon_rate()==&quote));
                let actual_days = f64::from(raw[index + 1].days_since(raw[index]));
                // Act/360's single rational division and then one product;
                // no production convention or coupon method supplies this oracle.
                assert_eq!(
                    flow.amount().to_bits(),
                    (360.0 * (actual_days / 360.0)).to_bits()
                );
                shifted += usize::from(payment != raw[index + 1]);
            }
            assert!(
                matches!(rejected(Leg::fixed_rate(&schedule,f64::NAN,&quote,0)),
                CashFlowError::FlowLimitExceeded { maximum:0,required,.. } if required==count)
            );
            pairs += 1;
        }
    }
    assert_eq!(pairs, 49);
    assert!(shifted > 0);
    Ok(())
}

#[test]
fn schedule_directions_stubs_eom_explicit_and_zero_need_no_coupon_inference()
-> Result<(), Box<dyn Error>> {
    let quote = FixedCouponRate::new(0.0, DayCount::Actual360)?;
    let b = ScheduleBuilder::new(
        d("2026-01-31"),
        d("2026-07-15"),
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?;
    for rule in [Rule::Forward, Rule::Backward] {
        for stub in [StubPolicy::Short, StubPolicy::Long] {
            for eom in [EndOfMonth::Ignore, EndOfMonth::Preserve] {
                let schedule =
                    b.build_recurrent(Period::new(1, TimeUnit::Months), rule, stub, eom, 8)?;
                let leg = Leg::fixed_rate(&schedule, 1.0, &quote, 8)?;
                assert_eq!(leg.flows().len(), schedule.contractual_dates().len() - 1);
                assert!(leg.flows().iter().all(|flow| flow.amount().to_bits() == 0));
            }
        }
        let explicit = b.build_explicit(
            Period::new(1, TimeUnit::Months),
            rule,
            ExplicitStubDates::new(Some(d("2026-04-30")), Some(d("2026-04-30")))?,
            EndOfMonth::Preserve,
            3,
        )?;
        assert_eq!(Leg::fixed_rate(&explicit, 1.0, &quote, 2)?.flows().len(), 2);
    }
    let zero = b.build_zero(2)?;
    let leg = Leg::fixed_rate(&zero, 1.0, &quote, 1)?;
    assert_eq!(leg.flows().len(), 1);
    assert_eq!(leg.flows()[0].payment_date(), b.end());
    Ok(())
}

#[test]
fn delayed_completed_and_new_partial_coupons_are_both_outstanding() -> Result<(), Box<dyn Error>> {
    let quote = FixedCouponRate::new(1.0, DayCount::Actual360)?;
    let payment = d("2026-01-15");
    let previous = FixedRateCoupon::new(
        d("2026-01-01"),
        d("2026-01-11"),
        payment,
        360.0,
        quote.clone(),
    )?;
    let next = FixedRateCoupon::new(
        d("2026-01-11"),
        d("2026-01-21"),
        d("2026-01-25"),
        360.0,
        quote,
    )?;
    let leg = Leg::new(
        vec![previous.into(), simple(payment, 1000.0)?, next.into()],
        3,
    )?;
    assert_eq!(
        leg.accrued_amount(d("2026-01-14"), Inclusion::Include, 1)?
            .to_bits(),
        13.0_f64.to_bits()
    );
    assert_eq!(
        leg.accrued_amount(payment, Inclusion::Include, 1)?
            .to_bits(),
        14.0_f64.to_bits()
    );
    assert_eq!(
        leg.accrued_amount(payment, Inclusion::Exclude, 1)?
            .to_bits(),
        4.0_f64.to_bits()
    );
    assert_work(
        rejected(leg.accrued_amount(payment, Inclusion::Exclude, 0)),
        Some(2),
        "partial accrual",
        0,
        0,
        1,
    );
    assert_eq!(
        leg.accrued_amount(Date::MIN, Inclusion::Include, 0)?
            .to_bits(),
        0
    );
    assert_eq!(
        leg.accrued_amount(Date::MAX, Inclusion::Include, 0)?
            .to_bits(),
        0
    );
    Ok(())
}

#[test]
fn flat_npv_uses_direct_original_horizons_and_retains_duplicate_payments()
-> Result<(), Box<dyn Error>> {
    let valuation = d("2026-01-01");
    let quote = InterestRate::new(0.125, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
    let leg = Leg::new(
        vec![
            simple(d("2027-01-01"), 100.0)?,
            simple(d("2028-01-01"), 100.0)?,
        ],
        2,
    )?;
    let first: f64 = 100.0 * (1.0 / 1.125);
    let second: f64 = 100.0 * (1.0 / 1.25);
    assert_eq!(
        leg.net_present_value(valuation, &quote, Inclusion::Include, 3)?
            .to_bits(),
        (first + second).to_bits()
    );
    let chained = first + 100.0 * ((1.0 / 1.125) * (1.0 / 1.125));
    assert!((first + second - chained).abs() > 0.5);
    let same_day = Leg::new(
        vec![
            simple(valuation, 4.0)?,
            simple(valuation, -1.0)?,
            simple(d("2026-01-02"), 0.0)?,
        ],
        3,
    )?;
    let zero = InterestRate::new(0.0, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
    assert_eq!(
        same_day
            .net_present_value(valuation, &zero, Inclusion::Include, 4)?
            .to_bits(),
        3.0_f64.to_bits()
    );
    assert_eq!(
        same_day
            .net_present_value(valuation, &zero, Inclusion::Exclude, 2)?
            .to_bits(),
        0
    );
    assert_work(
        rejected(leg.net_present_value(valuation, &quote, Inclusion::Include, 2)),
        Some(1),
        "discount horizon",
        2,
        2,
        3,
    );
    Ok(())
}

#[test]
fn every_discount_configuration_retains_exact_zero_rate_identity_and_query_charges()
-> Result<(), Box<dyn Error>> {
    let valuation = d("2026-01-01");
    let leg = Leg::new(
        vec![
            simple(d("2027-01-01"), 9_007_199_254_740_992.0)?,
            simple(d("2028-01-01"), 1.0)?,
            simple(d("2029-01-01"), -9_007_199_254_740_992.0)?,
        ],
        3,
    )?;
    for compounding in compoundings()? {
        let quote = InterestRate::new(-0.0, DayCount::Actual365Fixed, compounding)?;
        assert_eq!(
            leg.net_present_value(valuation, &quote, Inclusion::Include, 4)?
                .to_bits(),
            1.0_f64.to_bits()
        );
        assert_work(
            rejected(leg.net_present_value(valuation, &quote, Inclusion::Include, 3)),
            Some(2),
            "discount horizon",
            3,
            3,
            4,
        );
    }
    Ok(())
}

#[test]
fn empty_and_filtered_legs_validate_the_anchor_and_original_flow_indices()
-> Result<(), Box<dyn Error>> {
    let valuation = d("2026-01-01");
    let quote = InterestRate::new(0.0, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
    let empty = Leg::new(vec![], 0)?;
    assert_work(
        rejected(empty.net_present_value(valuation, &quote, Inclusion::Exclude, 0)),
        None,
        "valuation anchor",
        0,
        0,
        1,
    );
    assert_eq!(
        empty
            .net_present_value(valuation, &quote, Inclusion::Exclude, 1)?
            .to_bits(),
        0
    );
    let leg = Leg::new(
        vec![
            simple(d("2025-12-31"), 1.0)?,
            simple(valuation, 1.0)?,
            simple(d("2026-01-02"), 0.0)?,
            simple(d("2026-01-03"), 1.0)?,
        ],
        4,
    )?;
    assert_work(
        rejected(leg.net_present_value(valuation, &quote, Inclusion::Exclude, 2)),
        Some(3),
        "discount horizon",
        2,
        2,
        3,
    );
    assert_eq!(
        leg.net_present_value(valuation, &quote, Inclusion::Exclude, 3)?
            .to_bits(),
        1.0_f64.to_bits()
    );
    let outside = InterestRate::new(
        0.0,
        icma(&[valuation, d("2026-02-01")], Frequency::Monthly)?,
        Compounding::SIMPLE,
    )?;
    let error = rejected(empty.net_present_value(Date::MIN, &outside, Inclusion::Exclude, 1));
    assert!(
        matches!(&error,CashFlowError::Rate { index:None,phase:"valuation anchor",
        cause:RateError::Icma(IcmaError::OutOfCoverage { start,end,.. }),.. }
        if (*start,*end)==(Date::MIN,Date::MIN))
    );
    Ok(())
}

#[test]
fn icma_work_counts_actual_overlaps_even_when_zero_amount_or_coverage_later_fails()
-> Result<(), Box<dyn Error>> {
    let boundaries = [
        d("2026-01-01"),
        d("2026-02-01"),
        d("2026-03-01"),
        d("2026-04-01"),
    ];
    let context = icma(&boundaries, Frequency::Monthly)?;
    let rate = InterestRate::new(0.0, context.clone(), Compounding::SIMPLE)?;
    let valuation = d("2026-01-15");
    let leg = Leg::new(
        vec![simple(d("2026-02-15"), 0.0)?, simple(d("2026-03-01"), 1.0)?],
        2,
    )?;
    assert_work(
        rejected(leg.net_present_value(valuation, &rate, Inclusion::Include, 3)),
        Some(0),
        "discount horizon",
        3,
        1,
        4,
    );
    assert_work(
        rejected(leg.net_present_value(valuation, &rate, Inclusion::Include, 6)),
        Some(1),
        "discount horizon",
        6,
        4,
        7,
    );
    assert_eq!(
        leg.net_present_value(valuation, &rate, Inclusion::Include, 7)?
            .to_bits(),
        1.0_f64.to_bits()
    );
    let outside = Leg::new(vec![simple(d("2026-05-01"), 0.0)?], 1)?;
    assert_work(
        rejected(outside.net_present_value(valuation, &rate, Inclusion::Include, 4)),
        Some(0),
        "discount horizon",
        4,
        1,
        5,
    );
    assert!(matches!(
        rejected(outside.net_present_value(valuation, &rate, Inclusion::Include, 5)),
        CashFlowError::Rate {
            index: Some(0),
            phase: "discount horizon",
            cause: RateError::Icma(IcmaError::OutOfCoverage { .. }),
            ..
        }
    ));
    let coupon = FixedRateCoupon::new(
        boundaries[0],
        boundaries[3],
        Date::MAX,
        0.0,
        FixedCouponRate::new(0.0, context)?,
    )?;
    let coupon_leg = Leg::new(vec![coupon.into()], 1)?;
    assert_work(
        rejected(coupon_leg.accrued_amount(d("2026-02-15"), Inclusion::Include, 2)),
        Some(0),
        "partial accrual",
        2,
        0,
        3,
    );
    assert_eq!(
        coupon_leg
            .accrued_amount(d("2026-02-15"), Inclusion::Include, 3)?
            .to_bits(),
        0
    );
    assert_eq!(
        coupon_leg
            .accrued_amount(boundaries[3], Inclusion::Include, 0)?
            .to_bits(),
        0
    );
    Ok(())
}

#[test]
fn target_work_guard_policy_and_typed_root_sources_survive_zero_shortcuts()
-> Result<(), Box<dyn Error>> {
    let valuation = d("2028-12-30");
    let payment = d("2029-01-01");
    let leg = Leg::new(
        vec![
            simple(d("2028-12-01"), 0.0)?,
            simple(valuation, 0.0)?,
            simple(payment, 0.0)?,
        ],
        3,
    )?;
    for policy in [TargetPolicy::VerifiedOnly, TargetPolicy::AllowProjection] {
        let rate = InterestRate::new(
            0.0,
            Business252::with_target(TargetCalendar::with_policy(policy)),
            Compounding::SIMPLE,
        )?;
        assert_work(
            rejected(leg.net_present_value(valuation, &rate, Inclusion::Exclude, 3)),
            Some(2),
            "discount horizon",
            3,
            1,
            4,
        );
        let result = leg.net_present_value(valuation, &rate, Inclusion::Exclude, 4);
        if policy == TargetPolicy::AllowProjection {
            assert_eq!(result?.to_bits(), 0);
        } else {
            let error = rejected(result);
            assert!(
                matches!(&error,CashFlowError::Rate { index:Some(2),phase:"discount horizon",
                cause:RateError::DayCount(DayCountError::Calendar(CalendarError::UnverifiedCoverage { date,.. })),.. }
                if *date==payment)
            );
            let root = CrateError::from(error.clone());
            assert_eq!(root.kind(), "cashflow.rate");
            let first = root.source().ok_or("root must preserve CashFlowError")?;
            assert_eq!(first.downcast_ref::<CashFlowError>(), Some(&error));
            let rate_source = first.source().ok_or("cash flow must preserve RateError")?;
            assert!(rate_source.downcast_ref::<RateError>().is_some());
            let day_source = rate_source
                .source()
                .ok_or("rate must preserve DayCountError")?;
            assert!(day_source.downcast_ref::<DayCountError>().is_some());
            assert!(
                day_source
                    .source()
                    .is_some_and(|e| e.downcast_ref::<CalendarError>().is_some())
            );
        }
    }
    let bad_quote = FixedCouponRate::new(0.0, Business252::with_target(TargetCalendar::new()))?;
    assert!(matches!(
        rejected(FixedRateCoupon::new(
            d("2025-12-31"),
            d("2026-01-02"),
            Date::MAX,
            0.0,
            bad_quote
        )),
        CashFlowError::Rate {
            phase: "coupon accrual",
            cause: RateError::DayCount(DayCountError::Calendar(
                CalendarError::UnverifiedCoverage { .. }
            )),
            ..
        }
    ));
    Ok(())
}

#[test]
fn reduction_recovers_prefix_overflow_but_not_failed_individual_present_values()
-> Result<(), Box<dyn Error>> {
    let valuation = d("2026-01-01");
    let rate = InterestRate::new(0.0, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
    let leg = Leg::new(
        vec![
            simple(valuation, f64::MAX)?,
            simple(valuation, f64::MAX)?,
            simple(valuation, -f64::MAX)?,
            simple(valuation, -f64::MAX)?,
            simple(valuation, f64::from_bits(1))?,
        ],
        5,
    )?;
    assert_eq!(
        leg.net_present_value(valuation, &rate, Inclusion::Include, 6)?
            .to_bits(),
        1
    );
    let overflow = Leg::new(
        vec![simple(valuation, f64::MAX)?, simple(valuation, f64::MAX)?],
        2,
    )?;
    let error = rejected(overflow.net_present_value(valuation, &rate, Inclusion::Include, 3));
    assert!(matches!(
        &error,
        CashFlowError::Sum {
            phase: "NPV reduction",
            cause: SumError::Arithmetic {
                index: None,
                error: ValueError::ResultNotFinite {
                    value: NonFinite::PositiveInfinity,
                    ..
                },
                ..
            },
            ..
        }
    ));
    assert!(
        error
            .source()
            .is_some_and(|e| e.downcast_ref::<SumError>().is_some())
    );
    let coupon = FixedRateCoupon::new(
        valuation,
        d("2027-01-01"),
        Date::MAX,
        f64::MAX,
        FixedCouponRate::new(1.0, DayCount::Actual365Fixed)?,
    )?;
    let cached = Leg::new(vec![coupon.clone().into(), coupon.into()], 2)?;
    assert!(matches!(
        rejected(cached.accrued_amount(d("2027-01-01"), Inclusion::Include, 0)),
        CashFlowError::Sum {
            phase: "accrued reduction",
            cause: SumError::Arithmetic { index: None, .. },
            ..
        }
    ));
    let negative = InterestRate::new(-0.5, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
    let payment = d("2027-01-01");
    let cancelling = Leg::new(
        vec![simple(payment, -f64::MAX)?, simple(payment, f64::MAX)?],
        2,
    )?;
    assert!(matches!(
        rejected(cancelling.net_present_value(valuation, &negative, Inclusion::Include, 3)),
        CashFlowError::Rate {
            index: Some(0),
            phase: "discount horizon",
            cause: RateError::Value(ValueError::ResultNotFinite {
                value: NonFinite::NegativeInfinity,
                ..
            }),
            ..
        }
    ));
    let factorless = InterestRate::new(-745.0, DayCount::Actual365Fixed, Compounding::CONTINUOUS)?;
    let scaled = Leg::new(vec![simple(payment, f64::from_bits(1))?], 1)?;
    assert!(matches!(
        rejected(scaled.net_present_value(valuation, &factorless, Inclusion::Include, 2)),
        CashFlowError::Rate {
            index: Some(0),
            cause: RateError::Value(_),
            ..
        }
    ));
    Ok(())
}

#[test]
fn immutable_quotes_coupons_and_legs_reuse_across_threads_with_fresh_budgets()
-> Result<(), Box<dyn Error>> {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<SimpleCashFlow>();
    send_sync::<FixedCouponRate>();
    send_sync::<FixedRateCoupon>();
    send_sync::<CashFlow>();
    send_sync::<Leg>();
    let start = d("2026-01-01");
    let end = d("2027-01-01");
    let quote = FixedCouponRate::new(1.0, icma(&[start, end], Frequency::Annual)?)?;
    let coupon = FixedRateCoupon::new(start, end, Date::MAX, 1.0, quote.clone())?;
    let leg = Leg::new(vec![coupon.into(), simple(Date::MAX, -1.0)?], 2)?;
    let methods = compoundings()?;
    let rates = methods
        .into_iter()
        .map(|method| InterestRate::new(0.0, DayCount::Actual365Fixed, method))
        .collect::<Result<Vec<_>, _>>()?;
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let leg = &leg;
            let rates = &rates;
            let quote = &quote;
            scope.spawn(move || {
                for _ in 0..16 {
                    let clone = leg.clone();
                    assert_eq!(&clone, leg);
                    assert_eq!(quote, &quote.clone());
                    for rate in rates {
                        let result = clone.net_present_value(start, rate, Inclusion::Include, 3);
                        assert!(matches!(result,Ok(value) if value.to_bits()==0));
                        assert_work(
                            rejected(clone.net_present_value(start, rate, Inclusion::Include, 0)),
                            None,
                            "valuation anchor",
                            0,
                            0,
                            1,
                        );
                    }
                    let accrued = clone.accrued_amount(end, Inclusion::Include, 0);
                    assert!(matches!(accrued,Ok(value) if value.to_bits()==1.0_f64.to_bits()));
                }
            });
        }
    });
    Ok(())
}
