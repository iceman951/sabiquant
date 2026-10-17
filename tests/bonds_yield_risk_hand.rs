//! Independent hand/adversarial tests for M8 Phase D (`FR-YR-01..16`).
//!
//! Covers prepared-plan generic bond yield-to-maturity inversion (`yield_from_price`)
//! and analytical yield-basis risk (`risk`) across all compounding branches,
//! non-monotone multi-root and tangent legs, zero-dirty-price raw vs normalized
//! risk, extreme binary64 overflow/underflow rescue, and error taxonomy.

#![expect(
    clippy::expect_used,
    reason = "hand integration tests assert exact IEEE-754 bit patterns and deterministic error shapes"
)]

use sabiquant::bonds::{
    Bond, BondError, BondPrice, BondPriceKind, BondPricingPlan, BondSettlement,
    BondYieldConvention, BondYieldError, BondYieldSensitivity,
};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{ActualActualIcma, DayCount, DeterminationPeriod};
use sabiquant::error::{Error as RootError, NonFinite, ValueError};
use sabiquant::math::{
    EvaluationStage, RootBracket, SolverError, SolverOptions, TerminationReason,
};
use sabiquant::rates::{AccrualConvention, Compounding, RateError};
use sabiquant::schedule::{DateGenerationRule, ScheduleBuilder, ScheduleCalendar, StubPolicy};
use sabiquant::time::{Date, Frequency, Period, TimeUnit};
use std::collections::HashSet;
use std::error::Error as StdError;

fn d(y: i32, m: u32, day: u32) -> Date {
    Date::from_ymd(y, m, day).expect("valid date")
}

fn default_options() -> SolverOptions {
    SolverOptions::new(1e-12, 1e-12, 100).expect("valid solver options")
}

fn tight_options() -> SolverOptions {
    SolverOptions::new(f64::from_bits(1), 0.0, 200).expect("valid tight solver options")
}

fn one_period_schedule(start: Date, end: Date) -> sabiquant::schedule::Schedule {
    let builder = ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        BusinessDayConvention::Unadjusted,
        BusinessDayConvention::Unadjusted,
    )
    .expect("valid builder");
    builder
        .build_recurrent(
            Period::new(100, TimeUnit::Years),
            DateGenerationRule::Forward,
            StubPolicy::Short,
            EndOfMonth::Ignore,
            4,
        )
        .expect("valid one-period schedule")
}

fn single_flow_bond(
    issue: Date,
    maturity: Date,
    coupon_amount: f64,
    redemption_amount: f64,
    face: f64,
    accrual: AccrualConvention,
) -> Bond {
    let redemption = SimpleCashFlow::new(maturity, redemption_amount).expect("redemption");
    if coupon_amount == 0.0 {
        Bond::zero_coupon(issue, maturity, face, redemption).expect("valid zero-coupon bond")
    } else {
        let fraction = accrual
            .year_fraction(issue, maturity)
            .expect("positive fraction");
        let rate = coupon_amount / (face * fraction);
        let coupon_rate = FixedCouponRate::new(rate, accrual).expect("valid coupon rate");
        let schedule = one_period_schedule(issue, maturity);
        Bond::fixed_rate(
            issue,
            schedule,
            face,
            coupon_rate,
            vec![None],
            redemption,
            16,
        )
        .expect("valid fixed-rate bond")
    }
}

fn custom_two_flow_bond(
    issue: Date,
    d1: Date,
    d2: Date,
    c1_amount: f64,
    redemption_amount: f64,
    face: f64,
    accrual: AccrualConvention,
) -> Bond {
    let f1 = accrual.year_fraction(issue, d1).expect("f1 > 0");
    let r1 = c1_amount / (face * f1);
    let cr1 = FixedCouponRate::new(r1, accrual).expect("cr1");
    let schedule = one_period_schedule(issue, d1);
    let redemption = SimpleCashFlow::new(d2, redemption_amount).expect("redemption");
    Bond::fixed_rate(issue, schedule, face, cr1, vec![None], redemption, 16)
        .expect("valid two-flow bond")
}

fn all_compounding_configs() -> Vec<Compounding> {
    let mut configs = vec![Compounding::SIMPLE, Compounding::CONTINUOUS];
    for freq in Frequency::ALL {
        if let Ok(c) = Compounding::compounded(freq) {
            configs.push(c);
        }
        if let Ok(c) = Compounding::simple_then_compounded(freq) {
            configs.push(c);
        }
        if let Ok(c) = Compounding::compounded_then_simple(freq) {
            configs.push(c);
        }
    }
    configs
}

#[test]
fn sensitivity_variants_are_distinct_and_hashable() {
    let mut set = HashSet::new();
    set.insert(BondYieldSensitivity::MathematicallyStrictlyDecreasing);
    set.insert(BondYieldSensitivity::Uncertified);
    assert_eq!(set.len(), 2);
}

#[test]
fn closed_form_single_flow_inversion_across_all_32_compounding_configurations() {
    let configs = all_compounding_configs();
    assert_eq!(configs.len(), 32);

    let issue = d(2024, 1, 1);
    let settlement_short = d(2024, 1, 1);
    let maturity_short = d(2024, 4, 1); // 91/365 < 0.25 years
    let maturity_long = d(2026, 1, 1); // 731/365 > 2.0 years
    let accrual = AccrualConvention::from(DayCount::Actual365Fixed);

    let bracket = RootBracket::new(-0.20, 0.50).unwrap();
    let options = tight_options();

    for &compounding in &configs {
        for &maturity in &[maturity_short, maturity_long] {
            let bond = single_flow_bond(issue, maturity, 0.0, 100.0, 100.0, accrual.clone());
            let settlement = BondSettlement::new(settlement_short, PaymentDateInclusion::Exclude);
            let convention = BondYieldConvention::new(accrual.clone(), compounding);
            let plan = BondPricingPlan::new(&bond, settlement, &convention, 32).unwrap();

            let target_yield = 0.0475;
            let expected_val = plan.evaluate(target_yield).unwrap();
            let dirty_quote = expected_val.dirty_price().unwrap();
            let clean_quote = expected_val.clean_price().unwrap();

            for target_quote in [dirty_quote, clean_quote] {
                let report = plan
                    .yield_from_price(target_quote, bracket, options)
                    .unwrap();
                assert_eq!(
                    report.sensitivity(),
                    BondYieldSensitivity::MathematicallyStrictlyDecreasing
                );
                assert_eq!(report.target(), target_quote);
                assert_eq!(report.face(), 100.0);
                assert_eq!(report.settlement(), settlement);
                assert_eq!(report.compounding(), compounding);
                assert_eq!(report.initial_bracket(), bracket);
                assert_eq!(report.options(), options);
                assert_eq!(report.residual_scale_exponent(), 0);
                let amt_res = report.amount_residual().unwrap();
                assert_eq!(report.solve_report().residual(), amt_res);
                assert_eq!(report.solve_report().point(), report.yield_rate());
                assert!((report.yield_rate() - target_yield).abs() <= 1e-14);
                assert!(amt_res.abs() <= 1e-13);
            }
        }
    }
}

#[test]
fn clean_and_dirty_targets_agree_with_mid_coupon_accrued_interest() {
    let issue = d(2024, 1, 1);
    let settlement_date = d(2024, 7, 1);
    let maturity = d(2025, 1, 1);
    let accrual = AccrualConvention::from(DayCount::Thirty360BondBasis);
    let bond = single_flow_bond(issue, maturity, 60.0, 1_000.0, 1_000.0, accrual.clone());
    let settlement = BondSettlement::new(settlement_date, PaymentDateInclusion::Exclude);
    let convention = BondYieldConvention::new(
        accrual,
        Compounding::compounded(Frequency::Semiannual).unwrap(),
    );
    let plan = BondPricingPlan::new(&bond, settlement, &convention, 16).unwrap();

    let expected_val = plan.evaluate(0.0525).unwrap();
    assert_eq!(expected_val.accrued_amount(), 30.0);

    let bracket = RootBracket::new(-0.10, 0.40).unwrap();
    let options = tight_options();

    let dirty_report = plan
        .yield_from_price(expected_val.dirty_price().unwrap(), bracket, options)
        .unwrap();
    let clean_report = plan
        .yield_from_price(expected_val.clean_price().unwrap(), bracket, options)
        .unwrap();

    assert_eq!(
        dirty_report.dirty_target_amount(),
        clean_report.dirty_target_amount()
    );
    assert_eq!(dirty_report.yield_rate(), clean_report.yield_rate());
    assert_eq!(dirty_report.dirty_amount(), clean_report.dirty_amount());
    assert_eq!(dirty_report.accrued_amount(), 30.0);
    assert_eq!(
        dirty_report.clean_amount().unwrap(),
        clean_report.clean_amount().unwrap()
    );
    assert_eq!(
        dirty_report.dirty_price().unwrap(),
        clean_report.dirty_price().unwrap()
    );
    assert_eq!(
        dirty_report.clean_price().unwrap(),
        clean_report.clean_price().unwrap()
    );
}

#[test]
fn non_monotone_two_flow_bond_selects_bracketed_roots_and_rejects_tangent_bracket() {
    // P(y) = -1 / (1 + y) + 1 / (1 + y)^2 = v^2 - v where v = 1 / (1 + y).
    // At T = -3/16 (-0.1875), roots are v = 3/4 (y = 1/3) and v = 1/4 (y = 3.0).
    // Minimum is T = -1/4 (-0.25) at v = 1/2 (y = 1.0).
    let issue = d(2024, 1, 1);
    let d1 = d(2025, 1, 1); // 360/360 = 1.0
    let d2 = d(2026, 1, 1); // 720/360 = 2.0
    let accrual = AccrualConvention::from(DayCount::Thirty360BondBasis);
    let bond = custom_two_flow_bond(issue, d1, d2, -1.0, 1.0, 100.0, accrual.clone());
    let settlement = BondSettlement::new(issue, PaymentDateInclusion::Exclude);
    let convention =
        BondYieldConvention::new(accrual, Compounding::compounded(Frequency::Annual).unwrap());
    let plan = BondPricingPlan::new(&bond, settlement, &convention, 16).unwrap();

    let target = BondPrice::from_amount(-0.1875, 100.0, BondPriceKind::Dirty).unwrap();
    let options = tight_options();

    let r1 = plan
        .yield_from_price(target, RootBracket::new(0.0, 1.0).unwrap(), options)
        .unwrap();
    assert_eq!(r1.sensitivity(), BondYieldSensitivity::Uncertified);
    assert!((r1.yield_rate() - (1.0 / 3.0)).abs() <= 1e-15);
    assert!(r1.amount_residual().unwrap().abs() <= 1e-16);

    let r2 = plan
        .yield_from_price(target, RootBracket::new(1.5, 4.0).unwrap(), options)
        .unwrap();
    assert_eq!(r2.sensitivity(), BondYieldSensitivity::Uncertified);
    assert_eq!(r2.yield_rate(), 3.0);
    assert_eq!(r2.amount_residual().unwrap(), 0.0);
    assert_eq!(r2.solve_report().reason(), TerminationReason::FunctionZero);

    // Tangent minimum T = -0.25 at y = 1.0:
    let tangent_target = BondPrice::from_amount(-0.25, 100.0, BondPriceKind::Dirty).unwrap();
    let not_bracketed = plan
        .yield_from_price(tangent_target, RootBracket::new(0.0, 3.0).unwrap(), options)
        .unwrap_err();
    assert_eq!(not_bracketed.kind(), "bond_yield.solver");
    assert!(matches!(
        not_bracketed,
        BondYieldError::Solver(SolverError::NotBracketed { .. })
    ));

    // When an endpoint equals the tangent root y = 1.0, FunctionZero succeeds immediately.
    let endpoint_hit = plan
        .yield_from_price(tangent_target, RootBracket::new(0.0, 1.0).unwrap(), options)
        .unwrap();
    assert_eq!(endpoint_hit.yield_rate(), 1.0);
    assert_eq!(endpoint_hit.amount_residual().unwrap(), 0.0);
    assert_eq!(
        endpoint_hit.solve_report().reason(),
        TerminationReason::FunctionZero
    );

    // Target strictly below minimum (-0.30 < -0.25): no real root.
    let impossible_target = BondPrice::from_amount(-0.30, 100.0, BondPriceKind::Dirty).unwrap();
    let err = plan
        .yield_from_price(
            impossible_target,
            RootBracket::new(0.0, 3.0).unwrap(),
            options,
        )
        .unwrap_err();
    assert_eq!(err.kind(), "bond_yield.solver");
}

#[test]
fn constant_plan_certification_covers_post_maturity_zero_horizon_and_paired_opposite_flows() {
    let issue = d(2024, 1, 1);
    let maturity = d(2025, 1, 1);
    let post_maturity = d(2025, 6, 1);
    let accrual = AccrualConvention::from(DayCount::Thirty360BondBasis);
    let bond = single_flow_bond(issue, maturity, 5.0, 100.0, 100.0, accrual.clone());
    let convention = BondYieldConvention::new(
        accrual.clone(),
        Compounding::compounded(Frequency::Annual).unwrap(),
    );
    let bracket = RootBracket::new(-0.05, 0.20).unwrap();
    let options = default_options();

    // 1. Post-maturity plan (0 active flows -> constant C = 0.0).
    let post_plan = BondPricingPlan::new(
        &bond,
        BondSettlement::new(post_maturity, PaymentDateInclusion::Exclude),
        &convention,
        16,
    )
    .unwrap();
    let zero_target = BondPrice::new(0.0, BondPriceKind::Dirty).unwrap();
    let err_match = post_plan
        .yield_from_price(zero_target, bracket, options)
        .unwrap_err();
    assert_eq!(err_match.kind(), "bond_yield.no_yield_sensitivity");
    match &err_match {
        BondYieldError::NoYieldSensitivity {
            bracket: b,
            dirty_target_amount,
            observed_lower_dirty_amount,
            constant_amount,
            evaluations,
            ..
        } => {
            assert_eq!(*b, bracket);
            assert_eq!(*dirty_target_amount, 0.0);
            assert_eq!(*observed_lower_dirty_amount, 0.0);
            assert_eq!(*constant_amount, Some(0.0));
            assert_eq!(evaluations.function(), 1);
            assert_eq!(evaluations.derivative(), 0);
        }
        other => panic!("expected NoYieldSensitivity, got {other:?}"),
    }

    let nonzero_target = BondPrice::new(10.0, BondPriceKind::Dirty).unwrap();
    let err_mismatch = post_plan
        .yield_from_price(nonzero_target, bracket, options)
        .unwrap_err();
    assert_eq!(err_mismatch.kind(), "bond_yield.constant_price_mismatch");
    assert!(matches!(
        err_mismatch,
        BondYieldError::ConstantPriceMismatch {
            dirty_target_amount: 10.0,
            observed_lower_dirty_amount: 0.0,
            constant_amount: Some(0.0),
            ..
        }
    ));

    // 2. Settlement on maturity with Include -> horizon == 0.0, C = 105.0.
    let maturity_plan = BondPricingPlan::new(
        &bond,
        BondSettlement::new(maturity, PaymentDateInclusion::Include),
        &convention,
        16,
    )
    .unwrap();
    let target_105 = BondPrice::new(105.0, BondPriceKind::Dirty).unwrap();
    let err_105 = maturity_plan
        .yield_from_price(target_105, bracket, options)
        .unwrap_err();
    assert_eq!(err_105.kind(), "bond_yield.no_yield_sensitivity");
    assert!(matches!(
        err_105,
        BondYieldError::NoYieldSensitivity {
            dirty_target_amount: 105.0,
            observed_lower_dirty_amount: 105.0,
            constant_amount: Some(105.0),
            ..
        }
    ));

    // 3. Opposite equal positive-horizon flows at same date (-100 coupon and +100 redemption).
    let cancel_bond = single_flow_bond(issue, maturity, -100.0, 100.0, 100.0, accrual);
    let cancel_plan = BondPricingPlan::new(
        &cancel_bond,
        BondSettlement::new(issue, PaymentDateInclusion::Exclude),
        &convention,
        16,
    )
    .unwrap();
    let err_cancel_zero = cancel_plan
        .yield_from_price(zero_target, bracket, options)
        .unwrap_err();
    assert_eq!(err_cancel_zero.kind(), "bond_yield.no_yield_sensitivity");
    assert!(matches!(
        err_cancel_zero,
        BondYieldError::NoYieldSensitivity {
            dirty_target_amount: 0.0,
            observed_lower_dirty_amount: 0.0,
            constant_amount: Some(0.0),
            ..
        }
    ));

    let err_cancel_mismatch = cancel_plan
        .yield_from_price(nonzero_target, bracket, options)
        .unwrap_err();
    assert_eq!(
        err_cancel_mismatch.kind(),
        "bond_yield.constant_price_mismatch"
    );

    // 4. Lower-endpoint callback failure on constant plan (rate = -1.0 is outside Periodic domain).
    let bad_bracket = RootBracket::new(-1.0, 0.10).unwrap();
    let err_lower_fail = cancel_plan
        .yield_from_price(zero_target, bad_bracket, options)
        .unwrap_err();
    assert_eq!(err_lower_fail.kind(), "bond_yield.callback");
    assert!(matches!(
        err_lower_fail,
        BondYieldError::Callback {
            stage: EvaluationStage::LowerEndpoint,
            point: -1.0,
            ..
        }
    ));
}

#[test]
fn fixed_scale_residual_switches_at_exact_two_pow_970_threshold() {
    let issue = d(2024, 1, 1);
    let maturity = d(2025, 1, 1);
    let accrual = AccrualConvention::from(DayCount::Thirty360BondBasis);
    let redemption_amount = f64::from_bits(0x7c90_0000_0000_0000); // 2^970
    let bond = single_flow_bond(
        issue,
        maturity,
        0.0,
        redemption_amount,
        100.0,
        accrual.clone(),
    );
    let settlement = BondSettlement::new(issue, PaymentDateInclusion::Exclude);
    let convention = BondYieldConvention::new(accrual, Compounding::SIMPLE);
    let plan = BondPricingPlan::new(&bond, settlement, &convention, 16).unwrap();

    let below_threshold = f64::from_bits(0x7c8f_ffff_ffff_ffff); // 2^970 - ulp
    let at_threshold = f64::from_bits(0x7c90_0000_0000_0000); // 2^970

    let bracket = RootBracket::new(-0.05, 0.20).unwrap();
    let options = SolverOptions::new(1e-6, 1e-6, 100).unwrap();

    let target_below =
        BondPrice::from_amount(below_threshold, 100.0, BondPriceKind::Dirty).unwrap();
    let rep_below = plan
        .yield_from_price(target_below, bracket, options)
        .unwrap();
    assert_eq!(rep_below.residual_scale_exponent(), 0);
    assert_eq!(
        rep_below.solve_report().residual(),
        rep_below.amount_residual().unwrap()
    );

    let target_at = BondPrice::from_amount(at_threshold, 100.0, BondPriceKind::Dirty).unwrap();
    let rep_at = plan
        .yield_from_price(target_at, RootBracket::new(0.0, 0.20).unwrap(), options)
        .unwrap();
    assert_eq!(rep_at.residual_scale_exponent(), 1);
    assert_eq!(rep_at.yield_rate(), 0.0);
    assert_eq!(rep_at.amount_residual().unwrap(), 0.0);
    assert_eq!(rep_at.solve_report().residual(), 0.0);
}

#[test]
fn brent_cached_endpoint_retains_matching_valuation_and_residual_payload() {
    let issue = d(2024, 1, 1);
    let maturity = d(2026, 1, 1);
    let accrual = AccrualConvention::from(DayCount::Thirty360BondBasis);
    let bond = single_flow_bond(issue, maturity, 0.0, 100.0, 100.0, accrual.clone());
    let settlement = BondSettlement::new(issue, PaymentDateInclusion::Exclude);
    let convention = BondYieldConvention::new(accrual, Compounding::CONTINUOUS);
    let plan = BondPricingPlan::new(&bond, settlement, &convention, 16).unwrap();

    let target_val = plan.evaluate(0.010_000_1).unwrap();
    let bracket = RootBracket::new(0.01, 0.50).unwrap();
    let loose_options = SolverOptions::new(0.001, 0.0, 100).unwrap();

    let report = plan
        .yield_from_price(target_val.dirty_price().unwrap(), bracket, loose_options)
        .unwrap();
    assert_eq!(report.solve_report().point(), report.yield_rate());
    let reevaluated = plan.evaluate(report.yield_rate()).unwrap();
    assert_eq!(report.dirty_amount(), reevaluated.dirty_amount());
    assert_eq!(
        report.solve_report().residual(),
        report.amount_residual().unwrap()
    );
}

#[test]
fn analytical_risk_matches_closed_form_across_simple_continuous_periodic_and_mixed_branches() {
    let issue = d(2024, 1, 1);
    let d1 = d(2024, 4, 1); // 90/360 = 0.25 years
    let d2 = d(2026, 1, 1); // 720/360 = 2.0 years
    let accrual = AccrualConvention::from(DayCount::Thirty360BondBasis);

    let single_bond = single_flow_bond(issue, d2, 0.0, 100.0, 100.0, accrual.clone());
    let settlement = BondSettlement::new(issue, PaymentDateInclusion::Exclude);

    // Simple: v = 100 / (1 + 0.05*2) = 100 / 1.1, P' = -v * 2 / 1.1, P'' = 2 * v * 4 / 1.21, M = 2 * v.
    let conv_simple = BondYieldConvention::new(accrual.clone(), Compounding::SIMPLE);
    let plan_simple = BondPricingPlan::new(&single_bond, settlement, &conv_simple, 16).unwrap();
    let risk_simple = plan_simple.risk(0.05).unwrap();
    let v_s = risk_simple.dirty_amount();
    assert!((risk_simple.price_derivative().unwrap() - (-v_s * 2.0 / 1.1)).abs() <= 1e-13);
    assert!(
        (risk_simple.price_second_derivative().unwrap() - (2.0 * v_s * 4.0 / (1.1 * 1.1))).abs()
            <= 1e-12
    );
    assert_eq!(risk_simple.time_weighted_amount().unwrap(), 2.0 * v_s);
    assert_eq!(risk_simple.macaulay_duration().unwrap(), 2.0);
    assert!((risk_simple.modified_duration().unwrap() - (2.0 / 1.1)).abs() <= 1e-15);
    assert!((risk_simple.convexity().unwrap() - (8.0 / 1.21)).abs() <= 1e-15);

    // Continuous: P' = -2 * v, P'' = 4 * v, M = 2 * v -> D_mod = 2, D_mac = 2, C = 4.
    let conv_cont = BondYieldConvention::new(accrual.clone(), Compounding::CONTINUOUS);
    let plan_cont = BondPricingPlan::new(&single_bond, settlement, &conv_cont, 16).unwrap();
    let risk_cont = plan_cont.risk(0.05).unwrap();
    let v_c = risk_cont.dirty_amount();
    assert_eq!(risk_cont.price_derivative().unwrap(), -2.0 * v_c);
    assert_eq!(risk_cont.price_second_derivative().unwrap(), 4.0 * v_c);
    assert_eq!(risk_cont.time_weighted_amount().unwrap(), 2.0 * v_c);
    assert_eq!(risk_cont.modified_duration().unwrap(), 2.0);
    assert_eq!(risk_cont.macaulay_duration().unwrap(), 2.0);
    assert_eq!(risk_cont.convexity().unwrap(), 4.0);

    // Periodic(Semiannual, m = 2): P' = -v * 2 * 2 / 2.05, P'' = v * 2 * 2.5 * 4 / 2.05^2.
    let semi = Compounding::compounded(Frequency::Semiannual).unwrap();
    let conv_semi = BondYieldConvention::new(accrual.clone(), semi);
    let plan_semi = BondPricingPlan::new(&single_bond, settlement, &conv_semi, 16).unwrap();
    let risk_semi = plan_semi.risk(0.05).unwrap();
    let v_p = risk_semi.dirty_amount();
    assert!((risk_semi.price_derivative().unwrap() - (-v_p * 4.0 / 2.05)).abs() <= 1e-13);
    assert!(
        (risk_semi.price_second_derivative().unwrap() - (v_p * 20.0 / (2.05 * 2.05))).abs()
            <= 1e-12
    );
    assert_eq!(risk_semi.macaulay_duration().unwrap(), 2.0);
    assert!((risk_semi.modified_duration().unwrap() - (2.0 / 1.025)).abs() <= 1e-15);
    assert!((risk_semi.convexity().unwrap() - (5.0 / (1.025 * 1.025))).abs() <= 1e-15);

    // Mixed branch bond: flow 1 at t1 = 0.25 (< 0.5) and flow 2 at t2 = 2.0 (>= 0.5)
    // under SimpleThenCompounded(Semiannual) vs CompoundedThenSimple(Semiannual).
    let mixed_bond = custom_two_flow_bond(issue, d1, d2, 10.0, 100.0, 100.0, accrual.clone());
    let stp = Compounding::simple_then_compounded(Frequency::Semiannual).unwrap();
    let pts = Compounding::compounded_then_simple(Frequency::Semiannual).unwrap();
    let conv_stp = BondYieldConvention::new(accrual.clone(), stp);
    let conv_pts = BondYieldConvention::new(accrual, pts);

    let plan_stp = BondPricingPlan::new(&mixed_bond, settlement, &conv_stp, 16).unwrap();
    let plan_pts = BondPricingPlan::new(&mixed_bond, settlement, &conv_pts, 16).unwrap();

    let risk_stp = plan_stp.risk(0.08).unwrap();
    let risk_pts = plan_pts.risk(0.08).unwrap();
    assert_eq!(risk_stp.compounding(), stp);
    assert_eq!(risk_pts.compounding(), pts);
    assert_ne!(
        risk_stp.price_derivative().unwrap(),
        risk_pts.price_derivative().unwrap()
    );
    assert_ne!(
        risk_stp.price_second_derivative().unwrap(),
        risk_pts.price_second_derivative().unwrap()
    );
}

#[test]
fn zero_dirty_price_succeeds_for_raw_derivatives_and_rejects_normalized_getters() {
    let issue = d(2024, 1, 1);
    let d1 = d(2025, 1, 1);
    let d2 = d(2026, 1, 1);
    let accrual = AccrualConvention::from(DayCount::Thirty360BondBasis);
    let bond = custom_two_flow_bond(issue, d1, d2, -1.0, 1.0, 100.0, accrual.clone());
    let settlement = BondSettlement::new(issue, PaymentDateInclusion::Exclude);
    let convention =
        BondYieldConvention::new(accrual, Compounding::compounded(Frequency::Annual).unwrap());
    let plan = BondPricingPlan::new(&bond, settlement, &convention, 16).unwrap();

    let risk = plan.risk(-0.0).unwrap();
    assert_eq!(risk.yield_rate().to_bits(), 0.0_f64.to_bits());
    assert_eq!(risk.settlement(), settlement);
    assert_eq!(risk.face(), 100.0);
    assert_eq!(risk.dirty_amount(), 0.0);
    assert_eq!(risk.price_derivative().unwrap(), -1.0);
    assert_eq!(risk.price_second_derivative().unwrap(), 4.0);
    assert_eq!(risk.time_weighted_amount().unwrap(), 1.0);

    for (measure, res) in [
        ("modified duration", risk.modified_duration()),
        ("macaulay duration", risk.macaulay_duration()),
        ("convexity", risk.convexity()),
    ] {
        let err = res.unwrap_err();
        assert_eq!(err.kind(), "bond.zero_dirty_price");
        assert!(matches!(
            err,
            BondError::ZeroDirtyPrice {
                measure: m,
                yield_rate: 0.0,
                ..
            } if m == measure
        ));
    }
}

#[test]
fn extreme_scale_risk_rescues_normalized_duration_and_convexity_when_raw_moments_overflow_or_underflow()
 {
    let issue = d(2024, 1, 1);
    let d_half = d(2024, 7, 1); // 180/360 = 0.5
    let d_two = d(2026, 1, 1); // 720/360 = 2.0
    let accrual = AccrualConvention::from(DayCount::Thirty360BondBasis);
    let settlement = BondSettlement::new(issue, PaymentDateInclusion::Exclude);
    let convention = BondYieldConvention::new(accrual.clone(), Compounding::CONTINUOUS);

    // 1. Overflow rescue: c = f64::MAX at t = 2.0, y = 0.0.
    let max_bond = single_flow_bond(issue, d_two, 0.0, f64::MAX, 100.0, accrual.clone());
    let max_plan = BondPricingPlan::new(&max_bond, settlement, &convention, 16).unwrap();
    let max_risk = max_plan.risk(0.0).unwrap();
    assert_eq!(max_risk.dirty_amount(), f64::MAX);
    assert!(matches!(
        max_risk.price_derivative().unwrap_err(),
        BondError::Value {
            index: None,
            phase: "price derivative",
            cause: ValueError::ResultNotFinite {
                quantity: "price derivative",
                value: NonFinite::NegativeInfinity,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        max_risk.price_second_derivative().unwrap_err(),
        BondError::Value {
            index: None,
            phase: "price second derivative",
            cause: ValueError::ResultNotFinite {
                quantity: "price second derivative",
                value: NonFinite::PositiveInfinity,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        max_risk.time_weighted_amount().unwrap_err(),
        BondError::Value {
            index: None,
            phase: "time weighted amount",
            cause: ValueError::ResultNotFinite {
                quantity: "time weighted amount",
                value: NonFinite::PositiveInfinity,
                ..
            },
            ..
        }
    ));
    assert_eq!(max_risk.modified_duration().unwrap(), 2.0);
    assert_eq!(max_risk.macaulay_duration().unwrap(), 2.0);
    assert_eq!(max_risk.convexity().unwrap(), 4.0);

    // 2. Underflow rescue: c = MIN_SUB (2^-1074) at t = 0.5, y = 0.0.
    let min_sub = f64::from_bits(1);
    let sub_bond = single_flow_bond(issue, d_half, 0.0, min_sub, 100.0, accrual);
    let sub_plan = BondPricingPlan::new(&sub_bond, settlement, &convention, 16).unwrap();
    let sub_risk = sub_plan.risk(0.0).unwrap();
    assert_eq!(sub_risk.dirty_amount(), min_sub);
    assert!(matches!(
        sub_risk.price_derivative().unwrap_err(),
        BondError::Value {
            index: None,
            phase: "price derivative",
            cause: ValueError::ResultOutOfDomain {
                quantity: "price derivative",
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        sub_risk.price_second_derivative().unwrap_err(),
        BondError::Value {
            index: None,
            phase: "price second derivative",
            cause: ValueError::ResultOutOfDomain {
                quantity: "price second derivative",
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        sub_risk.time_weighted_amount().unwrap_err(),
        BondError::Value {
            index: None,
            phase: "time weighted amount",
            cause: ValueError::ResultOutOfDomain {
                quantity: "time weighted amount",
                ..
            },
            ..
        }
    ));
    assert_eq!(sub_risk.modified_duration().unwrap(), 0.5);
    assert_eq!(sub_risk.macaulay_duration().unwrap(), 0.5);
    assert_eq!(sub_risk.convexity().unwrap(), 0.25);
}

#[test]
fn target_and_risk_error_paths_preserve_causes_and_convert_to_root_error() {
    let issue = d(2024, 1, 1);
    let maturity = d(2025, 1, 1);
    let accrual = AccrualConvention::from(DayCount::Thirty360BondBasis);
    let bond = single_flow_bond(issue, maturity, 0.0, 100.0, f64::MAX, accrual.clone());
    let settlement = BondSettlement::new(issue, PaymentDateInclusion::Exclude);
    let convention = BondYieldConvention::new(
        accrual.clone(),
        Compounding::compounded(Frequency::Annual).unwrap(),
    );
    let plan = BondPricingPlan::new(&bond, settlement, &convention, 16).unwrap();

    // Target conversion overflow: value_per_100_face = 200.0 * face (MAX) / 100.0 overflows.
    let big_target = BondPrice::new(200.0, BondPriceKind::Dirty).unwrap();
    let target_err = plan
        .yield_from_price(
            big_target,
            RootBracket::new(0.0, 0.10).unwrap(),
            default_options(),
        )
        .unwrap_err();
    assert_eq!(target_err.kind(), "bond_yield.target");
    assert!(target_err.source().is_some());
    let root_err = RootError::from(target_err.clone());
    assert_eq!(root_err.kind(), "bond_yield.target");
    assert_eq!(root_err.to_string(), target_err.to_string());

    // Target clean shift overflow: accrued >= 2^970 and clean target = f64::MAX.
    let accrued_bond = single_flow_bond(issue, maturity, f64::MAX / 2.0, 100.0, 100.0, accrual);
    let mid_settlement = BondSettlement::new(d(2024, 7, 1), PaymentDateInclusion::Exclude);
    let accrued_plan =
        BondPricingPlan::new(&accrued_bond, mid_settlement, &convention, 16).unwrap();
    let max_clean = BondPrice::from_amount(f64::MAX, 100.0, BondPriceKind::Clean).unwrap();
    let shift_err = accrued_plan
        .yield_from_price(
            max_clean,
            RootBracket::new(0.0, 0.10).unwrap(),
            default_options(),
        )
        .unwrap_err();
    assert_eq!(shift_err.kind(), "bond_yield.target");
    assert!(matches!(
        shift_err,
        BondYieldError::Target {
            phase: "target clean shift",
            ..
        }
    ));

    // Risk input validation: NaN yield rate and out-of-domain periodic yield rate (-1.0).
    let nan_risk_err = plan.risk(f64::NAN).unwrap_err();
    assert_eq!(nan_risk_err.kind(), "bond.rate");
    assert!(matches!(
        nan_risk_err,
        BondError::Rate {
            index: None,
            phase: "yield quote",
            cause: RateError::Value(ValueError::NotFinite {
                parameter: "rate",
                value: NonFinite::NaN,
                ..
            }),
            ..
        }
    ));

    let domain_risk_err = plan.risk(-1.0).unwrap_err();
    assert_eq!(domain_risk_err.kind(), "bond.rate");
    assert!(matches!(
        domain_risk_err,
        BondError::Rate {
            index: None,
            phase: "yield quote",
            ..
        }
    ));

    let simple_conv = BondYieldConvention::new(
        AccrualConvention::from(DayCount::Thirty360BondBasis),
        Compounding::SIMPLE,
    );
    let simple_plan = BondPricingPlan::new(&bond, settlement, &simple_conv, 16).unwrap();
    let flow_domain_err = simple_plan.risk(-1.0).unwrap_err();
    assert_eq!(flow_domain_err.kind(), "bond.rate");
    assert!(matches!(
        flow_domain_err,
        BondError::Rate {
            index: Some(0),
            phase: "discount factor",
            ..
        }
    ));

    // ICMA schedule works seamlessly with yield_from_price and risk.
    let icma = ActualActualIcma::new(
        vec![DeterminationPeriod::new(issue, maturity).unwrap()],
        Frequency::Annual,
    )
    .unwrap();
    let icma_accrual = AccrualConvention::from(icma);
    let icma_bond = single_flow_bond(issue, maturity, 5.0, 100.0, 100.0, icma_accrual.clone());
    let icma_conv = BondYieldConvention::new(
        icma_accrual,
        Compounding::compounded(Frequency::Annual).unwrap(),
    );
    let icma_plan = BondPricingPlan::new(&icma_bond, settlement, &icma_conv, 16).unwrap();
    let icma_val = icma_plan.evaluate(0.05).unwrap();
    let icma_rep = icma_plan
        .yield_from_price(
            icma_val.clean_price().unwrap(),
            RootBracket::new(-0.05, 0.25).unwrap(),
            tight_options(),
        )
        .unwrap();
    assert!((icma_rep.yield_rate() - 0.05).abs() <= 1e-14);
    let icma_risk = icma_plan.risk(0.05).unwrap();
    assert_eq!(icma_risk.macaulay_duration().unwrap(), 1.0);
}
