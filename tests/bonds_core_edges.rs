//! Public edge regressions for FR-BOND-02/03/06/07/08/09/10.
//!
//! These expectations follow exact powers of two and explicit date roles.
//! They audit the post-core tranche, without changing frozen reference rows.

use sabiquant::bonds::{
    Bond, BondError, BondPricingPlan, BondSettlement, BondYieldConvention, CouponEntitlement,
    ExCouponBoundary, ExCouponTerms, amount_from_price_per_100, price_per_100_from_amount,
};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{Business252, DayCount};
use sabiquant::error::{Error as RootError, NonFinite, ValueError};
use sabiquant::rates::Compounding;
use sabiquant::schedule::{
    DateGenerationRule, Schedule, ScheduleBuilder, ScheduleCalendar, StubPolicy,
};
use sabiquant::time::{Date, Period, TimeUnit, Weekday};
use std::error::Error;
use std::ops::Bound;

type TestResult = Result<(), Box<dyn Error>>;

fn date(text: &str) -> Result<Date, Box<dyn Error>> {
    Ok(text.parse()?)
}

fn single_period(start: Date, end: Date) -> Result<Schedule, Box<dyn Error>> {
    Ok(ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        BusinessDayConvention::Unadjusted,
        BusinessDayConvention::Unadjusted,
    )?
    .build_zero(2)?)
}

#[test]
fn valid_full_minimum_coupon_does_not_make_partial_underflow_unreachable() -> TestResult {
    // FR-BOND-03/06/07/09: minsubnormal * (+/-1) * 1 is representable.
    // Its positive 1/365-year partial amount is strictly below half a
    // subnormal unit, so single nearest rounding gives signed zero and fails.
    let start = date("2026-01-01")?;
    let settlement = date("2026-01-02")?;
    let end = date("2027-01-01")?;
    let yield_convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);
    for rate in [1.0_f64, -1.0] {
        for detached in [false, true] {
            let ex_terms =
                detached.then_some(ExCouponTerms::new(start, ExCouponBoundary::OnOrAfter));
            let bond = Bond::fixed_rate(
                start,
                single_period(start, end)?,
                f64::from_bits(1),
                FixedCouponRate::new(rate, DayCount::Actual365Fixed)?,
                vec![ex_terms],
                SimpleCashFlow::new(end, 100.0)?,
                2,
            )?;
            let expected_full = 1 | (rate.to_bits() & (1_u64 << 63));
            assert_eq!(bond.coupons()[0].amount().to_bits(), expected_full);
            let input = BondSettlement::new(settlement, PaymentDateInclusion::Include);
            assert!(matches!(
                BondPricingPlan::new(&bond, input, &yield_convention, 1),
                Err(BondError::AccrualWorkLimitExceeded {
                    index: Some(0),
                    phase: "partial accrual",
                    maximum: 1,
                    used: 1,
                    required_at_least: 2,
                    ..
                })
            ));
            // Preparation failure precedes the NaN trial quote, even when the
            // coupon would be detached from the dirty-price cash flows.
            let error = bond
                .evaluate(input, &yield_convention, f64::NAN, 2)
                .err()
                .ok_or("partial product must reject nonzero underflow")?;
            let expected_zero = rate.to_bits() & (1_u64 << 63);
            match &error {
                BondError::Value {
                    index: Some(0),
                    phase: "accrued amount",
                    cause:
                        ValueError::ResultOutOfDomain {
                            quantity,
                            value,
                            domain,
                            ..
                        },
                    ..
                } => {
                    assert_eq!(*quantity, "accrued amount");
                    assert_eq!(value.to_bits(), expected_zero);
                    assert!(!domain.contains(*value));
                }
                other => return Err(format!("unexpected partial failure {other:?}").into()),
            }
            let root = RootError::from(error.clone());
            let source = root.source().ok_or("bond cause")?;
            assert_eq!(source.downcast_ref::<BondError>(), Some(&error));
            assert!(source.source().ok_or("value cause")?.is::<ValueError>());
        }
    }
    Ok(())
}

#[test]
fn cached_start_and_full_accrual_keep_signed_minimum_coupon_bits() -> TestResult {
    // FR-BOND-03/06/07: zero and full/cached paths are distinct from the
    // nonzero partial calculation and perform no coupon day-count query.
    let start = date("2026-01-01")?;
    let end = date("2027-01-01")?;
    let convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);
    for rate in [1.0_f64, -1.0] {
        for detached in [false, true] {
            let bond = Bond::fixed_rate(
                start,
                single_period(start, end)?,
                f64::from_bits(1),
                FixedCouponRate::new(rate, DayCount::Actual365Fixed)?,
                vec![detached.then_some(ExCouponTerms::new(start, ExCouponBoundary::OnOrAfter))],
                SimpleCashFlow::new(end, 100.0)?,
                2,
            )?;
            let at_start = BondPricingPlan::new(
                &bond,
                BondSettlement::new(start, PaymentDateInclusion::Include),
                &convention,
                3,
            )?;
            assert_eq!(at_start.accrual_work_used(), if detached { 2 } else { 3 });
            let full = bond.coupons()[0].amount();
            assert_eq!(
                at_start.accrued_amount().to_bits(),
                if detached { (-full).to_bits() } else { 0 }
            );
            let at_end = BondPricingPlan::new(
                &bond,
                BondSettlement::new(end, PaymentDateInclusion::Include),
                &convention,
                3,
            )?;
            assert_eq!(at_end.accrual_work_used(), if detached { 2 } else { 3 });
            assert_eq!(
                at_end.accrued_amount().to_bits(),
                if detached { 0 } else { full.to_bits() }
            );
            assert_eq!(
                at_end.coupon_entitlements(),
                &[if detached {
                    CouponEntitlement::ExCouponDetached
                } else {
                    CouponEntitlement::BuyerEntitled
                }]
            );
            let ceased = BondPricingPlan::new(
                &bond,
                BondSettlement::new(end, PaymentDateInclusion::Exclude),
                &convention,
                1,
            )?;
            assert_eq!(ceased.accrual_work_used(), 1);
            assert_eq!(ceased.accrued_amount().to_bits(), 0);
            assert_eq!(ceased.coupon_entitlements(), &[CouponEntitlement::Ceased]);
            assert_eq!(ceased.evaluate(0.0)?.dirty_amount().to_bits(), 0);
        }
    }
    Ok(())
}

#[test]
fn contract_maturity_does_not_replace_the_supplied_redemption_payment() -> TestResult {
    // FR-BOND-01/03/06: payment eligibility uses payment, independently of
    // contractual maturity. There is no inferred roll or maturity filter.
    let issue = date("2026-01-01")?;
    let maturity = date("2027-01-01")?;
    let convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);
    for payment in [date("2026-12-31")?, maturity, date("2027-01-04")?] {
        let bond = Bond::zero_coupon(issue, maturity, 100.0, SimpleCashFlow::new(payment, 100.0)?)?;
        for settlement in [
            payment.checked_add_days(-1)?,
            payment,
            payment.checked_add_days(1)?,
        ] {
            for inclusion in [PaymentDateInclusion::Include, PaymentDateInclusion::Exclude] {
                let eligible = payment > settlement
                    || (payment == settlement && inclusion == PaymentDateInclusion::Include);
                let plan = BondPricingPlan::new(
                    &bond,
                    BondSettlement::new(settlement, inclusion),
                    &convention,
                    2,
                )?;
                assert_eq!(plan.redemption_entitled(), eligible);
                assert_eq!(plan.entitled_flow_count(), usize::from(eligible));
                assert_eq!(plan.accrual_work_used(), if eligible { 2 } else { 1 });
                assert_eq!(
                    plan.evaluate(0.0)?.dirty_amount().to_bits(),
                    if eligible { 100.0_f64.to_bits() } else { 0 }
                );
            }
        }
    }
    Ok(())
}

#[test]
fn quote_errors_retain_finite_input_bits_signed_zero_and_full_numeric_domain() -> TestResult {
    // FR-BOND-04/09: all typed fields matter, not only the stable kind.
    for inverse in [false, true] {
        let convert = if inverse {
            amount_from_price_per_100
        } else {
            price_per_100_from_amount
        };
        for face in [0.0, -0.0, -1.0, -f64::MAX] {
            let error = convert(f64::NAN, face)
                .err()
                .ok_or("face rejected before value")?;
            match error {
                BondError::Value {
                    index: None,
                    phase: "bond face",
                    cause:
                        ValueError::OutOfDomain {
                            parameter,
                            value,
                            domain,
                            ..
                        },
                    ..
                } => {
                    assert_eq!(parameter, "bond face");
                    assert_eq!(value.to_bits(), face.to_bits());
                    assert_eq!(domain.lower(), Bound::Excluded(0.0));
                    assert_eq!(domain.upper(), Bound::Unbounded);
                }
                other => return Err(format!("face failure {other:?}").into()),
            }
        }
        let result_quantity = if inverse {
            "bond amount"
        } else {
            "price per 100"
        };
        for sign in [1.0_f64, -1.0] {
            let face = if inverse { 50.0 } else { 200.0 };
            let error = convert(sign * f64::from_bits(1), face)
                .err()
                .ok_or("half minsubnormal rejected")?;
            match error {
                BondError::Value {
                    index: None,
                    phase,
                    cause:
                        ValueError::ResultOutOfDomain {
                            quantity,
                            value,
                            domain,
                            ..
                        },
                    ..
                } => {
                    assert_eq!(phase, result_quantity);
                    assert_eq!(quantity, result_quantity);
                    assert_eq!(value.to_bits(), sign.to_bits() & (1_u64 << 63));
                    let expected = if sign > 0.0 {
                        (Bound::Excluded(0.0), Bound::Unbounded)
                    } else {
                        (Bound::Unbounded, Bound::Excluded(0.0))
                    };
                    assert_eq!((domain.lower(), domain.upper()), expected);
                }
                other => return Err(format!("underflow failure {other:?}").into()),
            }
            let face = if inverse { f64::MAX } else { 1.0 };
            let error = convert(sign * f64::MAX, face)
                .err()
                .ok_or("signed overflow rejected")?;
            match error {
                BondError::Value {
                    index: None,
                    phase,
                    cause:
                        ValueError::ResultNotFinite {
                            quantity, value, ..
                        },
                    ..
                } => {
                    assert_eq!(phase, result_quantity);
                    assert_eq!(quantity, result_quantity);
                    assert_eq!(
                        value,
                        if sign > 0.0 {
                            NonFinite::PositiveInfinity
                        } else {
                            NonFinite::NegativeInfinity
                        }
                    );
                }
                other => return Err(format!("overflow failure {other:?}").into()),
            }
        }
    }
    Ok(())
}

#[test]
fn reopened_business252_plans_succeed_at_the_exact_cap_for_all_weekend_masks() -> TestResult {
    // FR-BOND-01/03/06/07: issue after raw start is an explicit reopening.
    // Work depends on calendar span, independently of actual business count.
    let raw_start = date("2026-01-01")?;
    let issue = date("2026-03-15")?;
    let settlement_date = date("2026-04-01")?;
    let middle = date("2026-07-01")?;
    let end = date("2027-01-01")?;
    let schedule = ScheduleBuilder::new(
        raw_start,
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
    let bond = Bond::fixed_rate(
        issue,
        schedule,
        360.0,
        FixedCouponRate::new(0.5, DayCount::ThirtyE360)?,
        vec![None, None],
        SimpleCashFlow::new(end, 360.0)?,
        3,
    )?;
    let settlement = BondSettlement::new(settlement_date, PaymentDateInclusion::Include);
    let first_span = u64::try_from(middle.days_since(settlement_date))?;
    let second_span = u64::try_from(end.days_since(settlement_date))?;
    let required = 5 + first_span + 2 * second_span;
    for mask in 0_u8..128 {
        let weekends = Weekday::ALL
            .into_iter()
            .enumerate()
            .filter_map(|(index, weekday)| ((mask & (1 << index)) != 0).then_some(weekday));
        let calendar = Calendar::custom(weekends, []);
        let convention = BondYieldConvention::new(Business252::new(calendar), Compounding::SIMPLE);
        let plan = BondPricingPlan::new(&bond, settlement, &convention, required)?;
        assert_eq!(plan.accrual_work_used(), required, "mask {mask}");
        assert_eq!(plan.accrued_amount().to_bits(), 45.0_f64.to_bits());
        assert_eq!(
            plan.coupon_entitlements(),
            &[CouponEntitlement::BuyerEntitled; 2]
        );
        let valuation = plan.evaluate(0.0)?;
        assert_eq!(valuation.dirty_amount().to_bits(), 540.0_f64.to_bits());
        assert_eq!(valuation.clean_amount().to_bits(), 495.0_f64.to_bits());
        assert!(
            matches!(BondPricingPlan::new(&bond, settlement, &convention, required - 1),
            Err(BondError::AccrualWorkLimitExceeded { index: Some(2), phase: "redemption discount horizon",
                maximum, used, required_at_least, .. })
                if maximum == required - 1 && required_at_least == required && used == required - 1 - second_span)
        );
    }
    Ok(())
}
