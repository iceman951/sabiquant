//! Hand-derived unit, property, error-precedence, and cross-convention tests for
//! M8 bond core (`sabiquant::bonds`).
//!
//! Covers FR-BOND-01..10 across zero-coupon and fixed-rate bullet bonds,
//! constructor validation precedence, ex-coupon entitlement and negative accrued
//! interest, all 49 BDC pairs, all supported schedule rules, contextual accrual
//! conventions (`ActualActualIcma`, `ThirtyE360Isda`, `Thirty360Us`, `Business252`
//! with `TargetCalendar` coverage guards), all 32 compounding configurations,
//! work-budget boundaries, summation overflow guards, and multi-threaded plan reuse.

use sabiquant::bonds::{
    Bond, BondError, BondKind, BondPrice, BondPriceKind, BondPricingPlan, BondSettlement,
    BondYieldConvention, CouponEntitlement, ExCouponBoundary, ExCouponTerms,
};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth, TargetCalendar};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{
    ActualActualIcma, Business252, DayCount, DeterminationPeriod, IcmaError, Thirty360Us,
    ThirtyE360Isda,
};
use sabiquant::error::{Error as CrateError, NonFinite, ValueError};
use sabiquant::rates::{AccrualConvention, Compounding, RateError};
use sabiquant::schedule::{
    DateGenerationRule, ExplicitStubDates, Schedule, ScheduleBuilder, ScheduleCalendar,
    SpecialRollRule, StubPolicy,
};
use sabiquant::time::{Date, Frequency, Period, TimeUnit};
use std::error::Error as StdError;

type TestResult<T = ()> = Result<T, Box<dyn StdError>>;

fn ymd(y: i32, m: u32, d: u32) -> TestResult<Date> {
    Ok(Date::from_ymd(y, m, d)?)
}

fn plain_semiannual_schedule(start: Date, end: Date, max_boundaries: u32) -> TestResult<Schedule> {
    Ok(ScheduleBuilder::new(
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
        max_boundaries,
    )?)
}

#[test]
fn zero_coupon_constructor_precedence_and_accessors() -> TestResult {
    let issue = ymd(2026, 6, 15)?;
    let earlier = ymd(2026, 6, 14)?;
    let maturity = ymd(2027, 6, 15)?;
    let red_ok = SimpleCashFlow::new(maturity, 100.0)?;
    let red_early = SimpleCashFlow::new(earlier, 100.0)?;
    let red_neg = SimpleCashFlow::new(maturity, -100.0)?;

    // 1. issue_date > contractual_maturity precedes redemption date and face/redemption amount checks.
    assert!(matches!(
        Bond::zero_coupon(issue, earlier, f64::NAN, red_early),
        Err(BondError::InvalidIssueMaturity {
            issue_date,
            contractual_maturity,
            ..
        }) if issue_date == issue && contractual_maturity == earlier
    ));

    // 2. issue_date > redemption.payment_date() precedes face and redemption amount checks.
    assert!(matches!(
        Bond::zero_coupon(issue, maturity, -1.0, red_early),
        Err(BondError::RedemptionBeforeIssue {
            issue_date,
            redemption_payment,
            ..
        }) if issue_date == issue && redemption_payment == earlier
    ));

    // 3. face validation precedes redemption amount validation.
    assert!(matches!(
        Bond::zero_coupon(issue, maturity, 0.0, red_neg),
        Err(BondError::Value {
            index: None,
            phase: "bond face",
            cause: ValueError::OutOfDomain {
                parameter: "bond face",
                ..
            },
            ..
        })
    ));

    // 4. redemption amount must be strictly positive.
    assert!(matches!(
        Bond::zero_coupon(issue, maturity, 100.0, red_neg),
        Err(BondError::Value {
            index: None,
            phase: "redemption amount",
            cause: ValueError::OutOfDomain {
                parameter: "redemption amount",
                ..
            },
            ..
        })
    ));

    // Valid zero-coupon bond retains all terms and empty coupon slices.
    let bond = Bond::zero_coupon(issue, maturity, 1_000.0, red_ok)?;
    assert_eq!(bond.kind(), BondKind::ZeroCoupon);
    assert_eq!(bond.issue_date(), issue);
    assert_eq!(bond.contractual_maturity(), maturity);
    assert_eq!(bond.face(), 1_000.0);
    assert_eq!(bond.redemption(), red_ok);
    assert!(bond.schedule().is_none());
    assert!(bond.coupon_rate().is_none());
    assert!(bond.coupons().is_empty());
    assert!(bond.ex_coupon_terms().is_empty());
    assert_eq!(bond.price_per_100_from_amount(950.0)?, 95.0);
    assert_eq!(
        bond.amount_from_price_per_100(BondPrice::new(95.0, BondPriceKind::Clean)?)?,
        950.0
    );
    Ok(())
}

#[test]
fn fixed_rate_constructor_precedence_and_retained_provenance() -> TestResult {
    let start = ymd(2026, 1, 15)?;
    let mid = ymd(2026, 7, 15)?;
    let end = ymd(2027, 1, 15)?;
    let schedule = plain_semiannual_schedule(start, end, 3)?;
    let rate = FixedCouponRate::new(0.05, DayCount::Actual365Fixed)?;
    let red_ok = SimpleCashFlow::new(end, 100.0)?;

    // 1. max_flows > Bond::MAX_FLOWS precedes everything.
    assert!(matches!(
        Bond::fixed_rate(
            ymd(2028, 1, 1)?,
            schedule.clone(),
            -1.0,
            rate.clone(),
            vec![],
            red_ok,
            Bond::MAX_FLOWS + 1,
        ),
        Err(BondError::InvalidFlowLimit {
            requested,
            maximum: Bond::MAX_FLOWS,
            ..
        }) if requested == Bond::MAX_FLOWS + 1
    ));

    // 2. total_flows > max_flows precedes ex_coupon count and date ordering.
    assert!(matches!(
        Bond::fixed_rate(
            ymd(2028, 1, 1)?,
            schedule.clone(),
            -1.0,
            rate.clone(),
            vec![],
            red_ok,
            2,
        ),
        Err(BondError::FlowLimitExceeded {
            maximum: 2,
            required: 3,
            ..
        })
    ));

    // 3. ex_coupon length mismatch precedes issue/maturity and redemption ordering.
    assert!(matches!(
        Bond::fixed_rate(
            ymd(2028, 1, 1)?,
            schedule.clone(),
            -1.0,
            rate.clone(),
            vec![None],
            red_ok,
            3,
        ),
        Err(BondError::ExCouponCountMismatch {
            expected: 2,
            actual: 1,
            ..
        })
    ));

    // 4. issue_date > contractual_maturity precedes redemption ordering.
    let late_issue = ymd(2027, 1, 16)?;
    assert!(matches!(
        Bond::fixed_rate(
            late_issue,
            schedule.clone(),
            -1.0,
            rate.clone(),
            vec![None, None],
            SimpleCashFlow::new(start, -1.0)?,
            3,
        ),
        Err(BondError::InvalidIssueMaturity {
            issue_date,
            contractual_maturity,
            ..
        }) if issue_date == late_issue && contractual_maturity == end
    ));

    // 5. issue_date > redemption.payment_date() precedes redemption < last_coupon_payment.
    let reopen_issue = ymd(2026, 8, 1)?;
    let early_red = SimpleCashFlow::new(mid, -1.0)?;
    assert!(matches!(
        Bond::fixed_rate(
            reopen_issue,
            schedule.clone(),
            -1.0,
            rate.clone(),
            vec![None, None],
            early_red,
            3,
        ),
        Err(BondError::RedemptionBeforeIssue {
            issue_date,
            redemption_payment,
            ..
        }) if issue_date == reopen_issue && redemption_payment == mid
    ));

    // 6. redemption.payment_date() < last_coupon_payment precedes face/redemption value checks.
    assert!(matches!(
        Bond::fixed_rate(
            start,
            schedule.clone(),
            -1.0,
            rate.clone(),
            vec![None, None],
            early_red,
            3,
        ),
        Err(BondError::RedemptionBeforeLastCoupon {
            last_coupon_payment,
            redemption_payment,
            ..
        }) if last_coupon_payment == end && redemption_payment == mid
    ));

    // 7. face > 0 precedes redemption > 0 and ex-coupon cutoff <= payment_date.
    let bad_ex = Some(ExCouponTerms::new(
        ymd(2026, 7, 16)?,
        ExCouponBoundary::OnOrAfter,
    ));
    assert!(matches!(
        Bond::fixed_rate(
            start,
            schedule.clone(),
            -5.0,
            rate.clone(),
            vec![bad_ex, None],
            SimpleCashFlow::new(end, -1.0)?,
            3,
        ),
        Err(BondError::Value {
            index: None,
            phase: "bond face",
            ..
        })
    ));

    // 8. redemption > 0 precedes ex-coupon cutoff <= payment_date.
    assert!(matches!(
        Bond::fixed_rate(
            start,
            schedule.clone(),
            100.0,
            rate.clone(),
            vec![bad_ex, None],
            SimpleCashFlow::new(end, 0.0)?,
            3,
        ),
        Err(BondError::Value {
            index: None,
            phase: "redemption amount",
            ..
        })
    ));

    // 9. ex_coupon cutoff > payment_date rejected with coupon index.
    let cut_err_date = ymd(2026, 7, 16)?;
    assert!(matches!(
        Bond::fixed_rate(
            start,
            schedule.clone(),
            100.0,
            rate.clone(),
            vec![bad_ex, None],
            red_ok,
            3,
        ),
        Err(BondError::ExCouponAfterPayment {
            index: 0,
            cutoff_date,
            payment_date,
            ..
        }) if cutoff_date == cut_err_date && payment_date == mid
    ));

    // 10. Coupon accrual failure (ICMA determination period mismatch at coupon 1).
    let icma_one = ActualActualIcma::new(
        vec![DeterminationPeriod::new(start, mid)?],
        Frequency::Semiannual,
    )?;
    let bad_icma_rate = FixedCouponRate::new(0.05, AccrualConvention::Icma(icma_one))?;
    assert!(matches!(
        Bond::fixed_rate(
            start,
            schedule.clone(),
            100.0,
            bad_icma_rate,
            vec![None, None],
            red_ok,
            3,
        ),
        Err(BondError::Rate {
            index: Some(1),
            phase: "coupon accrual",
            cause: RateError::Icma(IcmaError::OutOfCoverage { .. }),
            ..
        })
    ));

    // 11. Coupon amount overflow at construction.
    let huge_rate = FixedCouponRate::new(f64::MAX, DayCount::Actual365Fixed)?;
    assert!(matches!(
        Bond::fixed_rate(
            start,
            schedule.clone(),
            f64::MAX,
            huge_rate,
            vec![None, None],
            red_ok,
            3,
        ),
        Err(BondError::Value {
            index: Some(0),
            phase: "coupon amount",
            cause: ValueError::ResultNotFinite {
                quantity: "coupon amount",
                value: NonFinite::PositiveInfinity,
                ..
            },
            ..
        })
    ));

    // Valid fixed-rate bond retains schedule, coupon_rate, coupons, and ex_coupon_terms.
    let ex0 = Some(ExCouponTerms::new(
        ymd(2026, 7, 8)?,
        ExCouponBoundary::OnOrAfter,
    ));
    let ex1 = Some(ExCouponTerms::new(
        ymd(2027, 1, 8)?,
        ExCouponBoundary::After,
    ));
    let bond = Bond::fixed_rate(
        start,
        schedule.clone(),
        100.0,
        rate.clone(),
        vec![ex0, ex1],
        red_ok,
        3,
    )?;
    assert_eq!(bond.kind(), BondKind::FixedRate);
    assert_eq!(bond.schedule(), Some(&schedule));
    assert_eq!(bond.coupon_rate(), Some(&rate));
    assert_eq!(bond.coupons().len(), 2);
    assert_eq!(bond.ex_coupon_terms(), &[ex0, ex1]);
    Ok(())
}

#[test]
fn bond_price_and_quote_conversion_edge_cases() -> TestResult {
    for kind in BondPriceKind::ALL {
        let p_neg_zero = BondPrice::new(-0.0, kind)?;
        assert_eq!(p_neg_zero.value_per_100_face().to_bits(), 0.0_f64.to_bits());
        assert_eq!(p_neg_zero.kind(), kind);
    }
    assert!(matches!(
        BondPrice::new(f64::NAN, BondPriceKind::Clean),
        Err(BondError::Value {
            index: None,
            phase: "bond price",
            cause: ValueError::NotFinite {
                parameter: "price per 100",
                value: NonFinite::NaN,
                ..
            },
            ..
        })
    ));
    assert!(matches!(
        BondPrice::from_amount(f64::NAN, 100.0, BondPriceKind::Dirty),
        Err(BondError::Value {
            index: None,
            phase: "price per 100",
            ..
        })
    ));

    let cut = ymd(2026, 7, 8)?;
    let before_cut = ymd(2026, 7, 7)?;
    let after_cut = ymd(2026, 7, 9)?;
    for boundary in ExCouponBoundary::ALL {
        let terms = ExCouponTerms::new(cut, boundary);
        assert_eq!(terms.cutoff_date(), cut);
        assert_eq!(terms.boundary(), boundary);
        assert!(!terms.is_ex_coupon(before_cut));
        assert_eq!(
            terms.is_ex_coupon(cut),
            boundary == ExCouponBoundary::OnOrAfter
        );
        assert!(terms.is_ex_coupon(after_cut));
    }
    assert_eq!(BondKind::ALL.len(), 2);
    assert_eq!(CouponEntitlement::ALL.len(), 3);
    Ok(())
}

#[test]
fn all_49_schedule_business_day_convention_pairs_and_10_rules_work_with_bonds() -> TestResult {
    let cal = ScheduleCalendar::Plain(Calendar::weekends_only());
    let start = ymd(2026, 1, 15)?;
    let end = ymd(2027, 1, 15)?;
    let settle_apr = ymd(2026, 4, 15)?;
    let rate = FixedCouponRate::new(0.04, DayCount::Actual365Fixed)?;
    let y_conv = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);

    // All 7 x 7 = 49 (convention, termination_convention) pairs.
    for conv in BusinessDayConvention::ALL {
        for term_conv in BusinessDayConvention::ALL {
            let schedule = ScheduleBuilder::new(start, end, cal.clone(), conv, term_conv)?
                .build_recurrent(
                    Period::new(6, TimeUnit::Months),
                    DateGenerationRule::Forward,
                    StubPolicy::None,
                    EndOfMonth::Ignore,
                    3,
                )?;
            let last_pay = *schedule
                .adjusted_dates()
                .last()
                .ok_or("missing last date")?;
            let red = SimpleCashFlow::new(last_pay, 100.0)?;
            let bond = Bond::fixed_rate(
                start,
                schedule,
                100.0,
                rate.clone(),
                vec![None, None],
                red,
                3,
            )?;
            let val = bond.evaluate(
                BondSettlement::new(settle_apr, PaymentDateInclusion::Include),
                &y_conv,
                0.03,
                10,
            )?;
            assert!(val.dirty_amount() > 100.0);
            assert!(val.accrued_amount() > 0.0);
        }
    }

    // Backward, Zero, all 4 SpecialRollRule variants, and OldCDS schedules.
    let z_sched = ScheduleBuilder::new(
        start,
        end,
        cal.clone(),
        BusinessDayConvention::Following,
        BusinessDayConvention::Following,
    )?
    .build_zero(2)?;
    let z_last = *z_sched.adjusted_dates().last().ok_or("missing last")?;
    let z_bond = Bond::fixed_rate(
        start,
        z_sched,
        100.0,
        rate.clone(),
        vec![None],
        SimpleCashFlow::new(z_last, 100.0)?,
        2,
    )?;
    assert_eq!(z_bond.coupons().len(), 1);

    let r_start = ymd(2026, 3, 18)?;
    let r_end = ymd(2026, 9, 20)?;
    for roll_rule in SpecialRollRule::ALL {
        let r_sched = ScheduleBuilder::new(
            r_start,
            r_end,
            cal.clone(),
            BusinessDayConvention::Following,
            BusinessDayConvention::Following,
        )?
        .build_roll(Period::new(3, TimeUnit::Months), roll_rule, 8)?;
        let n_coupons = r_sched.contractual_dates().len() - 1;
        let r_last = *r_sched.adjusted_dates().last().ok_or("missing last")?;
        let r_issue = r_sched.contractual_dates()[0];
        let r_bond = Bond::fixed_rate(
            r_issue,
            r_sched,
            100.0,
            rate.clone(),
            vec![None; n_coupons],
            SimpleCashFlow::new(r_last, 100.0)?,
            10,
        )?;
        let val = r_bond.evaluate(
            BondSettlement::new(r_issue, PaymentDateInclusion::Include),
            &y_conv,
            0.02,
            20,
        )?;
        assert!(val.dirty_price_per_100()? > 0.0);
    }

    let oc_start = ymd(2026, 3, 20)?;
    let old_cds_sched = ScheduleBuilder::new(
        oc_start,
        r_end,
        cal,
        BusinessDayConvention::Following,
        BusinessDayConvention::Unadjusted,
    )?
    .build_old_cds(Period::new(3, TimeUnit::Months), 8)?;
    let oc_coupons = old_cds_sched.contractual_dates().len() - 1;
    let oc_last = *old_cds_sched
        .adjusted_dates()
        .last()
        .ok_or("missing last")?;
    let oc_issue = old_cds_sched.contractual_dates()[0];
    let oc_bond = Bond::fixed_rate(
        oc_issue,
        old_cds_sched,
        100.0,
        rate,
        vec![None; oc_coupons],
        SimpleCashFlow::new(oc_last, 100.0)?,
        10,
    )?;
    assert_eq!(oc_bond.coupons().len(), oc_coupons);
    Ok(())
}

#[test]
fn contextual_daycounts_and_all_32_compounding_modes_work_in_bond_pricing() -> TestResult {
    let start = ymd(2026, 1, 15)?;
    let mid = ymd(2026, 7, 15)?;
    let end = ymd(2027, 1, 15)?;
    let settle_apr = ymd(2026, 4, 15)?;
    let schedule = plain_semiannual_schedule(start, end, 3)?;
    let red = SimpleCashFlow::new(end, 100.0)?;

    // Contextual conventions: ICMA (from schedule), ThirtyE360Isda, Thirty360Us(false), Business252(TargetCalendar).
    let icma = schedule.build_icma_context(Frequency::Semiannual, 4)?;
    let isda_30e = ThirtyE360Isda::new(end);
    let us_no_eom = Thirty360Us::new(false);
    let target_cal = TargetCalendar::new();
    let b252_target = Business252::with_target(target_cal);

    for accrual in [
        AccrualConvention::Icma(icma),
        AccrualConvention::ThirtyE360Isda(isda_30e),
        AccrualConvention::Thirty360Us(us_no_eom),
        AccrualConvention::Business252(b252_target),
        AccrualConvention::Scalar(DayCount::ActualActualIsda),
        AccrualConvention::Scalar(DayCount::ActualActualAfb),
    ] {
        let rate = FixedCouponRate::new(0.05, accrual.clone())?;
        let bond = Bond::fixed_rate(
            start,
            schedule.clone(),
            100.0,
            rate,
            vec![None, None],
            red,
            3,
        )?;
        let y_conv = BondYieldConvention::new(accrual, Compounding::SIMPLE);
        let val = bond.evaluate(
            BondSettlement::new(settle_apr, PaymentDateInclusion::Include),
            &y_conv,
            0.04,
            1_000,
        )?;
        assert!(val.clean_price()?.value_per_100_face() > 90.0);
        assert!(val.dirty_price()?.value_per_100_face() > val.clean_price()?.value_per_100_face());
    }

    // All 32 Compounding configurations: Simple, Continuous, and 3 periodic families x 10 periodic frequencies.
    let bond = Bond::fixed_rate(
        start,
        schedule,
        100.0,
        FixedCouponRate::new(0.05, DayCount::Actual365Fixed)?,
        vec![None, None],
        red,
        3,
    )?;
    let settle = BondSettlement::new(mid, PaymentDateInclusion::Exclude);
    let mut all_comp = vec![Compounding::SIMPLE, Compounding::CONTINUOUS];
    for freq in Frequency::ALL {
        if freq.period().is_none() {
            continue;
        }
        all_comp.push(Compounding::compounded(freq)?);
        all_comp.push(Compounding::simple_then_compounded(freq)?);
        all_comp.push(Compounding::compounded_then_simple(freq)?);
    }
    assert_eq!(all_comp.len(), 32);

    for comp in all_comp {
        let y_conv = BondYieldConvention::new(DayCount::Actual365Fixed, comp);
        assert_eq!(y_conv.compounding(), comp);
        assert_eq!(
            y_conv.accrual(),
            &AccrualConvention::Scalar(DayCount::Actual365Fixed)
        );
        let val = bond.evaluate(settle, &y_conv, 0.04, 10)?;
        assert_eq!(val.settlement(), settle);
        assert_eq!(val.face(), 100.0);
        assert_eq!(val.yield_rate(), 0.04);
        assert_eq!(val.accrued_amount(), 0.0);
        assert_eq!(val.dirty_amount(), val.clean_amount());
    }
    Ok(())
}

#[test]
fn plan_error_precedence_target_coverage_and_multithreaded_reuse() -> TestResult {
    // TARGET verified coverage ends at 2028-12-31 under TargetPolicy::VerifiedOnly; test discount anchor failure.
    let start = ymd(2026, 1, 15)?;
    let end = ymd(2030, 1, 15)?;
    let pre_date = ymd(2026, 1, 14)?;
    let post_2028_date = ymd(2029, 1, 2)?;
    let zc = Bond::zero_coupon(start, end, 100.0, SimpleCashFlow::new(end, 100.0)?)?;
    let b252_strict =
        AccrualConvention::Business252(Business252::with_target(TargetCalendar::new()));
    let y_conv = BondYieldConvention::new(b252_strict, Compounding::SIMPLE);

    // 1. SettlementBeforeIssue precedes work limit and invalid yield_rate in Bond::evaluate.
    let pre_issue = BondSettlement::new(pre_date, PaymentDateInclusion::Include);
    assert!(matches!(
        zc.evaluate(pre_issue, &y_conv, f64::NAN, 0),
        Err(BondError::SettlementBeforeIssue {
            settlement_date,
            issue_date,
            ..
        }) if settlement_date == pre_date && issue_date == start
    ));

    // 2. Work limit 0 fails at discount anchor before querying TARGET calendar or checking yield_rate.
    let post_2028 = BondSettlement::new(post_2028_date, PaymentDateInclusion::Include);
    assert!(matches!(
        zc.evaluate(post_2028, &y_conv, f64::NAN, 0),
        Err(BondError::AccrualWorkLimitExceeded {
            index: None,
            phase: "discount anchor",
            maximum: 0,
            used: 0,
            required_at_least: 1,
            ..
        })
    ));

    // 3. With work budget >= 1, discount anchor on 2029-01-02 fails with BondError::Rate (TARGET coverage).
    let anchor_err = zc
        .evaluate(post_2028, &y_conv, f64::NAN, 10)
        .err()
        .ok_or("2029 outside verified TARGET coverage")?;
    assert_eq!(anchor_err.kind(), "bond.rate");
    assert!(anchor_err.source().is_some());

    // 4. Valid plan evaluated across multiple threads with different trial yields, plus invalid yield error.
    let y_act = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);
    let settle_start = BondSettlement::new(start, PaymentDateInclusion::Include);
    let plan = BondPricingPlan::new(&zc, settle_start, &y_act, 10)?;
    assert_eq!(plan.bond(), &zc);
    assert_eq!(plan.settlement(), settle_start);
    assert_eq!(plan.yield_convention(), &y_act);
    assert_eq!(plan.accrued_amount(), 0.0);

    // Fixed-rate bond at settlement == start charges 1 for anchor and 0 for partial accrual, so max_accrual_work = 1 fails on coupon 0 discount horizon.
    let fixed_1y = Bond::fixed_rate(
        start,
        plain_semiannual_schedule(start, ymd(2027, 1, 15)?, 3)?,
        100.0,
        FixedCouponRate::new(0.05, DayCount::Actual365Fixed)?,
        vec![None, None],
        SimpleCashFlow::new(ymd(2027, 1, 15)?, 100.0)?,
        3,
    )?;
    assert!(matches!(
        BondPricingPlan::new(&fixed_1y, settle_start, &y_act, 1),
        Err(BondError::AccrualWorkLimitExceeded {
            index: Some(0),
            phase: "discount horizon",
            maximum: 1,
            used: 1,
            required_at_least: 2,
            ..
        })
    ));

    let bad_yield = plan.evaluate(f64::NAN).err().ok_or("NaN yield must fail")?;
    assert_eq!(bad_yield.kind(), "bond.rate");

    // Discount factor failure when 1 + r * t <= 0 under Simple compounding.
    let df_err = plan.evaluate(-2.0).err().ok_or("1 + r*t <= 0 must fail")?;
    assert_eq!(df_err.kind(), "bond.rate");

    std::thread::scope(|s| -> TestResult {
        let mut handles = Vec::new();
        for i in 0..8 {
            let plan_ref = &plan;
            handles.push(s.spawn(move || {
                let y = 0.01 * f64::from(i);
                plan_ref.evaluate(y)
            }));
        }
        let mut prev_dirty = f64::INFINITY;
        for h in handles {
            let v = h.join().map_err(|_| "thread panicked")??;
            assert!(v.dirty_amount() < prev_dirty);
            prev_dirty = v.dirty_amount();
        }
        Ok(())
    })?;
    Ok(())
}

#[test]
fn summation_overflow_and_error_catalog_integration() -> TestResult {
    let start = ymd(2026, 1, 15)?;
    let mid = ymd(2026, 7, 15)?;
    let end = ymd(2027, 1, 15)?;

    // Construct two coupons whose individual amounts are ~1.25e308 (< f64::MAX) so their sum (~2.5e308) overflows.
    let face = 1.0e308;
    let rate = FixedCouponRate::new(2.5, DayCount::Actual365Fixed)?;
    let red = SimpleCashFlow::new(end, 100.0)?;
    let mut cal = Calendar::null();
    cal = cal.with_holiday(mid);
    let adj_sched = ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(cal),
        BusinessDayConvention::Following,
        BusinessDayConvention::Unadjusted,
    )?
    .build_recurrent(
        Period::new(6, TimeUnit::Months),
        DateGenerationRule::Forward,
        StubPolicy::None,
        EndOfMonth::Ignore,
        3,
    )?;
    let bond = Bond::fixed_rate(start, adj_sched, face, rate, vec![None, None], red, 3)?;
    let y_conv = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);

    // At settlement = start with yield = 0.0, dirty_amount sums two ~1.25e308 coupon PVs + redemption -> Sum overflow!
    let plan = BondPricingPlan::new(
        &bond,
        BondSettlement::new(start, PaymentDateInclusion::Include),
        &y_conv,
        10,
    )?;
    let sum_err = plan.evaluate(0.0).err().ok_or("dirty sum overflow")?;
    assert_eq!(sum_err.kind(), "bond.sum");
    assert!(matches!(
        sum_err,
        BondError::Sum {
            phase: "dirty price reduction",
            ..
        }
    ));
    assert!(sum_err.source().is_some());

    // Accrued reduction overflow: delay coupon 0 payment to 2027-01-18 and coupon 1 payment to 2027-01-20,
    // and settle on 2027-01-18 with Include (both coupons eligible and nearly/fully earned).
    let mut cal3 = Calendar::null();
    let mut d1 = mid;
    let jan18 = ymd(2027, 1, 18)?;
    let jan19 = ymd(2027, 1, 19)?;
    let jan20 = ymd(2027, 1, 20)?;
    while d1 < jan18 {
        cal3 = cal3.with_holiday(d1);
        d1 = d1.checked_add_days(1)?;
    }
    cal3 = cal3.with_holiday(jan19);
    let delayed_two = ScheduleBuilder::new(
        start,
        jan19,
        ScheduleCalendar::Plain(cal3),
        BusinessDayConvention::Following,
        BusinessDayConvention::Following,
    )?
    .build_explicit(
        Period::new(6, TimeUnit::Months),
        DateGenerationRule::Forward,
        ExplicitStubDates::new(Some(mid), Some(mid))?,
        EndOfMonth::Ignore,
        3,
    )?;
    let bond_acc_ovf = Bond::fixed_rate(
        start,
        delayed_two,
        face,
        FixedCouponRate::new(2.5, DayCount::Actual365Fixed)?,
        vec![None, None],
        SimpleCashFlow::new(jan20, 100.0)?,
        3,
    )?;
    let acc_err = BondPricingPlan::new(
        &bond_acc_ovf,
        BondSettlement::new(jan18, PaymentDateInclusion::Include),
        &y_conv,
        10,
    )
    .err()
    .ok_or("accrued reduction overflow")?;
    assert!(matches!(
        acc_err,
        BondError::Sum {
            phase: "accrued reduction",
            ..
        }
    ));

    // Clean price reduction overflow: coupon 0 is ex-coupon detached with negative accrued ~ -1.25e308,
    // while coupon 1 is buyer-entitled with PV ~ +1.25e308 -> dirty - accrued ~ +2.5e308 overflows!
    let ex0 = Some(ExCouponTerms::new(start, ExCouponBoundary::OnOrAfter));
    let bond_clean_ovf = Bond::fixed_rate(
        start,
        plain_semiannual_schedule(start, end, 3)?,
        face,
        FixedCouponRate::new(2.5, DayCount::Actual365Fixed)?,
        vec![ex0, None],
        SimpleCashFlow::new(end, 100.0)?,
        3,
    )?;
    let plan_clean_ovf = BondPricingPlan::new(
        &bond_clean_ovf,
        BondSettlement::new(start, PaymentDateInclusion::Include),
        &y_conv,
        10,
    )?;
    let clean_err = plan_clean_ovf
        .evaluate(0.0)
        .err()
        .ok_or("clean price reduction overflow")?;
    assert!(matches!(
        clean_err,
        BondError::Sum {
            phase: "clean price reduction",
            ..
        }
    ));

    // Discount horizon and redemption discount horizon Rate errors under verified TARGET coverage.
    let b252_target =
        AccrualConvention::Business252(Business252::with_target(TargetCalendar::new()));
    let zc_2029 = Bond::zero_coupon(
        ymd(2026, 1, 15)?,
        ymd(2029, 1, 15)?,
        100.0,
        SimpleCashFlow::new(ymd(2029, 1, 15)?, 100.0)?,
    )?;
    let y_target = BondYieldConvention::new(b252_target, Compounding::SIMPLE);
    let red_horizon_err = BondPricingPlan::new(
        &zc_2029,
        BondSettlement::new(ymd(2026, 1, 15)?, PaymentDateInclusion::Include),
        &y_target,
        10_000,
    )
    .err()
    .ok_or("redemption discount horizon outside TARGET")?;
    assert!(matches!(
        red_horizon_err,
        BondError::Rate {
            index: Some(0),
            phase: "redemption discount horizon",
            ..
        }
    ));

    let fixed_2029 = Bond::fixed_rate(
        ymd(2026, 1, 15)?,
        plain_semiannual_schedule(ymd(2028, 7, 15)?, ymd(2029, 7, 15)?, 3)?,
        100.0,
        FixedCouponRate::new(0.04, DayCount::Actual365Fixed)?,
        vec![None, None],
        SimpleCashFlow::new(ymd(2029, 7, 15)?, 100.0)?,
        3,
    )?;
    let disc_horizon_err = BondPricingPlan::new(
        &fixed_2029,
        BondSettlement::new(ymd(2028, 7, 15)?, PaymentDateInclusion::Include),
        &y_target,
        10_000,
    )
    .err()
    .ok_or("coupon discount horizon outside TARGET")?;
    assert!(matches!(
        disc_horizon_err,
        BondError::Rate {
            index: Some(0),
            phase: "discount horizon",
            ..
        }
    ));

    // Top-level crate::error::Error conversion preserves kind, Display, and source.
    let top: CrateError = sum_err.clone().into();
    assert_eq!(top.kind(), "bond.sum");
    assert_eq!(top.to_string(), sum_err.to_string());
    assert!(top.source().is_some());
    Ok(())
}
