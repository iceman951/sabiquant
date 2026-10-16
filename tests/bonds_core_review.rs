//! Adversarial public-API checks for FR-BOND-01..10.
//!
//! Expected results follow exact identities, integer coupon fractions and
//! binary midpoint arithmetic, independently of the frozen Python fixture.

use sabiquant::bonds::{
    Bond, BondError, BondPricingPlan, BondSettlement, BondYieldConvention, CouponEntitlement,
    ExCouponBoundary, ExCouponTerms, amount_from_price_per_100, price_per_100_from_amount,
};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{
    ActualActualIcma, Business252, DayCount, DeterminationPeriod, IcmaError,
};
use sabiquant::error::ValueError;
use sabiquant::rates::{AccrualConvention, Compounding, RateError};
use sabiquant::schedule::{
    DateGenerationRule, Schedule, ScheduleBuilder, ScheduleCalendar, StubPolicy,
};
use sabiquant::time::{Date, Frequency, Period, TimeUnit};
use std::error::Error;
use std::ops::Bound;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn one_period_schedule(start: Date, end: Date) -> TestResult<Schedule> {
    Ok(ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        BusinessDayConvention::Unadjusted,
        BusinessDayConvention::Unadjusted,
    )?
    .build_zero(2)?)
}

fn one_year_bond(coupon_rate: f64, redemption: f64) -> TestResult<Bond> {
    let start: Date = "2026-01-01".parse()?;
    let end: Date = "2027-01-01".parse()?;
    Ok(Bond::fixed_rate(
        start,
        one_period_schedule(start, end)?,
        100.0,
        FixedCouponRate::new(coupon_rate, DayCount::Actual365Fixed)?,
        vec![None],
        SimpleCashFlow::new(end, redemption)?,
        2,
    )?)
}

#[test]
fn opposite_equal_horizon_flows_cancel_for_every_compounding_configuration() -> TestResult {
    // FR-BOND-01/03/07/08: coupon = -100 * 1 exactly and redemption = +100.
    // Their identical positive horizons give exactly opposite rounded PVs.
    let bond = one_year_bond(-1.0, 100.0)?;
    let settlement = BondSettlement::new(bond.issue_date(), PaymentDateInclusion::Include);
    let mut compoundings = vec![Compounding::SIMPLE, Compounding::CONTINUOUS];
    for frequency in Frequency::ALL {
        if frequency.events_per_year().is_some() {
            compoundings.extend([
                Compounding::compounded(frequency)?,
                Compounding::simple_then_compounded(frequency)?,
                Compounding::compounded_then_simple(frequency)?,
            ]);
        }
    }
    assert_eq!(compoundings.len(), 32);
    for compounding in compoundings {
        let convention = BondYieldConvention::new(DayCount::Actual365Fixed, compounding);
        let plan = BondPricingPlan::new(&bond, settlement, &convention, 3)?;
        assert_eq!(
            plan.coupon_entitlements(),
            &[CouponEntitlement::BuyerEntitled]
        );
        assert!(plan.redemption_entitled());
        assert_eq!(plan.entitled_flow_count(), 2);
        assert_eq!(plan.accrual_work_used(), 3);
        for yield_rate in [-0.5, -0.01, -0.0, 0.0, 0.01, 0.04, 1.0] {
            let value = plan.evaluate(yield_rate)?;
            assert_eq!(
                value.dirty_amount().to_bits(),
                0,
                "{compounding:?}, {yield_rate}"
            );
            assert_eq!(value.accrued_amount().to_bits(), 0);
            assert_eq!(value.clean_amount().to_bits(), 0);
            assert_eq!(value.dirty_price_per_100()?.to_bits(), 0);
            assert_eq!(value.clean_price_per_100()?.to_bits(), 0);
        }
    }
    Ok(())
}

#[test]
fn negative_ex_coupon_accrual_adds_the_detached_liability() -> TestResult {
    // FR-BOND-03/04/07: Actual/360, 360-day full coupon and 180-day earned
    // coupon give A=-100 and E=-50 exactly; detached E-A is +50.
    let start: Date = "2026-01-01".parse()?;
    let middle: Date = "2026-06-30".parse()?;
    let end: Date = "2026-12-27".parse()?;
    let bond = Bond::fixed_rate(
        start,
        one_period_schedule(start, end)?,
        100.0,
        FixedCouponRate::new(-1.0, DayCount::Actual360)?,
        vec![Some(ExCouponTerms::new(start, ExCouponBoundary::OnOrAfter))],
        SimpleCashFlow::new(end, 100.0)?,
        2,
    )?;
    assert_eq!(bond.coupons()[0].amount().to_bits(), (-100.0_f64).to_bits());
    let convention = BondYieldConvention::new(DayCount::Actual360, Compounding::SIMPLE);
    for (date, inclusion, accrued, dirty, clean, work, entitlement) in [
        (
            start,
            PaymentDateInclusion::Include,
            100.0_f64,
            100.0_f64,
            0.0_f64,
            2,
            CouponEntitlement::ExCouponDetached,
        ),
        (
            middle,
            PaymentDateInclusion::Include,
            50.0,
            100.0,
            50.0,
            3,
            CouponEntitlement::ExCouponDetached,
        ),
        (
            end,
            PaymentDateInclusion::Include,
            0.0,
            100.0,
            100.0,
            2,
            CouponEntitlement::ExCouponDetached,
        ),
        (
            end,
            PaymentDateInclusion::Exclude,
            0.0,
            0.0,
            0.0,
            1,
            CouponEntitlement::Ceased,
        ),
    ] {
        let plan = BondPricingPlan::new(
            &bond,
            BondSettlement::new(date, inclusion),
            &convention,
            work,
        )?;
        assert_eq!(plan.coupon_entitlements(), &[entitlement]);
        assert_eq!(plan.accrual_work_used(), work);
        let valuation = plan.evaluate(0.0)?;
        assert_eq!(valuation.accrued_amount().to_bits(), accrued.to_bits());
        assert_eq!(valuation.dirty_amount().to_bits(), dirty.to_bits());
        assert_eq!(valuation.clean_amount().to_bits(), clean.to_bits());
    }
    Ok(())
}

#[test]
fn zero_coupon_amount_still_validates_its_factor_before_redemption() -> TestResult {
    // FR-BOND-06/07/09: zero rate gives a zero coupon, but the factor at
    // its original index 0 must still be checked before redemption index 1.
    let bond = one_year_bond(0.0, 100.0)?;
    let convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);
    let plan = BondPricingPlan::new(
        &bond,
        BondSettlement::new(bond.issue_date(), PaymentDateInclusion::Include),
        &convention,
        3,
    )?;
    let error = plan.evaluate(-1.0).err().ok_or("zero factor must fail")?;
    assert!(matches!(
        error,
        BondError::Rate {
            index: Some(0),
            phase: "discount factor",
            cause: RateError::Value(ValueError::ResultOutOfDomain {
                quantity: "compound factor",
                value: 0.0,
                domain,
                ..
            }),
            ..
        } if domain.lower() == Bound::Excluded(0.0) && domain.upper() == Bound::Unbounded
    ));
    Ok(())
}

#[test]
fn icma_work_budget_precedes_coupon_coverage_and_trial_yield_errors() -> TestResult {
    // FR-BOND-06/08/09: an equal-date ICMA anchor uses one unit. The next
    // query touches one determination period, so it costs two more units,
    // although its endpoint exceeds coverage and its flow amount is zero.
    let bond = one_year_bond(0.0, 100.0)?;
    let middle: Date = "2026-07-01".parse()?;
    let icma = ActualActualIcma::new(
        vec![DeterminationPeriod::new(bond.issue_date(), middle)?],
        Frequency::Semiannual,
    )?;
    let convention = BondYieldConvention::new(AccrualConvention::Icma(icma), Compounding::SIMPLE);
    let settlement = BondSettlement::new(bond.issue_date(), PaymentDateInclusion::Include);
    assert!(matches!(
        bond.evaluate(settlement, &convention, f64::NAN, 2),
        Err(BondError::AccrualWorkLimitExceeded {
            index: Some(0),
            phase: "discount horizon",
            maximum: 2,
            used: 1,
            required_at_least: 3,
            ..
        })
    ));
    let error = bond
        .evaluate(settlement, &convention, f64::NAN, 3)
        .err()
        .ok_or("ICMA coverage must fail before NaN yield")?;
    assert!(matches!(
        error,
        BondError::Rate {
            index: Some(0),
            phase: "discount horizon",
            cause: RateError::Icma(IcmaError::OutOfCoverage { .. }),
            ..
        }
    ));
    assert!(error.source().is_some_and(|cause| cause.source().is_some()));
    Ok(())
}

#[test]
fn reopened_business252_plan_succeeds_at_the_exact_cumulative_work_cap() -> TestResult {
    // FR-BOND-02/06/08: raw accrual starts before the supplied issue date.
    // Work counts calendar spans, independently of holiday/business counts:
    // anchor 1; partial Jan1->May1 121; coupon May1->Jul1 62;
    // final coupon and redemption May1->Jan1 246 each: total 676.
    let start: Date = "2026-01-01".parse()?;
    let issue: Date = "2026-03-01".parse()?;
    let settlement_date: Date = "2026-05-01".parse()?;
    let end: Date = "2027-01-01".parse()?;
    let schedule = ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        BusinessDayConvention::Unadjusted,
        BusinessDayConvention::Unadjusted,
    )?
    .build_recurrent(
        Period::new(6, TimeUnit::Months),
        DateGenerationRule::Forward,
        StubPolicy::None,
        EndOfMonth::Ignore,
        3,
    )?;
    let accrual = AccrualConvention::Business252(Business252::new(Calendar::weekends_only()));
    let bond = Bond::fixed_rate(
        issue,
        schedule,
        1000.0,
        FixedCouponRate::new(0.0, accrual.clone())?,
        vec![None, None],
        SimpleCashFlow::new(end, 1000.0)?,
        3,
    )?;
    let convention = BondYieldConvention::new(accrual, Compounding::SIMPLE);
    let settlement = BondSettlement::new(settlement_date, PaymentDateInclusion::Include);
    assert!(matches!(
        BondPricingPlan::new(&bond, settlement, &convention, 675),
        Err(BondError::AccrualWorkLimitExceeded {
            index: Some(2),
            phase: "redemption discount horizon",
            maximum: 675,
            used: 430,
            required_at_least: 676,
            ..
        })
    ));
    let plan = BondPricingPlan::new(&bond, settlement, &convention, 676)?;
    assert_eq!(plan.accrual_work_used(), 676);
    assert_eq!(
        plan.coupon_entitlements(),
        &[CouponEntitlement::BuyerEntitled; 2]
    );
    assert_eq!(plan.entitled_flow_count(), 3);
    assert!(plan.redemption_entitled());
    let value = plan.evaluate(0.0)?;
    assert_eq!(value.dirty_amount().to_bits(), 1000.0_f64.to_bits());
    assert_eq!(value.accrued_amount().to_bits(), 0);
    assert_eq!(value.clean_amount().to_bits(), 1000.0_f64.to_bits());
    Ok(())
}

#[test]
fn per_flow_rounding_does_not_guarantee_small_relative_error_after_cancellation() -> TestResult {
    // FR-BOND-07/08: the contract rounds each PV before its exact reduction.
    // At simple yield 1/4, rounded coupon PV is -80 and redemption PV is
    // next_up(80). Their exact reduction is 2^-46. The ideal real net PV is
    // (next_up(100)-100)/(5/4) = (4/5)*2^-46, a different quantity.
    let bond = one_year_bond(-1.0, 100.0_f64.next_up())?;
    let convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);
    let valuation = bond.evaluate(
        BondSettlement::new(bond.issue_date(), PaymentDateInclusion::Include),
        &convention,
        0.25,
        3,
    )?;
    let staged_net = 2.0_f64.powi(-46);
    let rounded_ideal_net = 0.8 * staged_net;
    assert_eq!(valuation.dirty_amount().to_bits(), staged_net.to_bits());
    assert_eq!(valuation.clean_amount().to_bits(), staged_net.to_bits());
    assert!(
        valuation
            .dirty_amount()
            .to_bits()
            .abs_diff(rounded_ideal_net.to_bits())
            > 1_000_000
    );
    Ok(())
}

#[test]
fn per_100_identities_and_exact_midpoints_cover_every_finite_normal_binade() -> TestResult {
    // FR-BOND-05: with face=100 both conversions are exact identities.
    // With face=150 the inverse is exactly 3*price/2. Source significands
    // 2^52 + j for odd j produce ties in the SAME output binade; parity
    // gives retained offsets 2,4,8,10 for j=1,3,5,7, respectively.
    let fraction_mask = (1_u64 << 52) - 1;
    for field in 1_u64..=2046 {
        let base = field << 52;
        for fraction in [0, 1, 1_u64 << 51, fraction_mask] {
            for sign in [0, 1_u64 << 63] {
                let bits = sign | base | fraction;
                let input = f64::from_bits(bits);
                assert_eq!(price_per_100_from_amount(input, 100.0)?.to_bits(), bits);
                assert_eq!(amount_from_price_per_100(input, 100.0)?.to_bits(), bits);
            }
        }
        for (steps, retained_offset) in [(1, 2), (3, 4), (5, 8), (7, 10)] {
            for sign in [0, 1_u64 << 63] {
                let input = f64::from_bits(sign | base | steps);
                let expected = sign | (base + (1_u64 << 51) + retained_offset);
                assert_eq!(amount_from_price_per_100(input, 150.0)?.to_bits(), expected);
            }
        }
    }
    Ok(())
}
