//! Additional public adversarial controls for FR-YR-02/03/04/06/10/12/14.
//!
//! Derived from the frozen generic yield/risk contract, exact powers of two,
//! signed subnormal arithmetic and explicit two-endpoint Simple prices. These
//! regressions use only valid public bond/plan construction, never fabricated
//! report fields, private extended terms or an impossible capacity failure.

#![expect(
    clippy::expect_used,
    reason = "review controls require the stated valid public construction/results"
)]

use sabiquant::bonds::{
    Bond, BondError, BondPrice, BondPriceKind, BondPricingPlan, BondSettlement,
    BondYieldConvention, BondYieldError, BondYieldSensitivity, ExCouponBoundary, ExCouponTerms,
};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::DayCount;
use sabiquant::error::{Error as RootError, NonFinite, ValueError};
use sabiquant::math::{
    EvaluationStage, RootBracket, SolverError, SolverOptions, SumError, TerminationReason,
};
use sabiquant::rates::{Compounding, RateError};
use sabiquant::schedule::{DateGenerationRule, ScheduleBuilder, ScheduleCalendar, StubPolicy};
use sabiquant::time::{Date, Period, TimeUnit};
use std::error::Error;
use std::ops::Bound;

const SIGN: u64 = 1 << 63;

fn date(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid civil review date")
}

fn power(exponent: u64) -> f64 {
    f64::from_bits((1023 + exponent) << 52)
}

fn fixed_one_period(
    end: Date,
    face: f64,
    rate: f64,
    redemption: f64,
    ex: Option<ExCouponTerms>,
) -> Bond {
    let issue = date(2024, 1, 1);
    let schedule = ScheduleBuilder::new(
        issue,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        BusinessDayConvention::Unadjusted,
        BusinessDayConvention::Unadjusted,
    )
    .expect("one-period schedule definition")
    .build_recurrent(
        Period::new(100, TimeUnit::Years),
        DateGenerationRule::Forward,
        StubPolicy::Short,
        EndOfMonth::Ignore,
        4,
    )
    .expect("one short explicit coupon interval");
    Bond::fixed_rate(
        issue,
        schedule,
        face,
        FixedCouponRate::new(rate, DayCount::Thirty360BondBasis).expect("finite signed coupon"),
        vec![ex],
        SimpleCashFlow::new(end, redemption).expect("positive redemption"),
        4,
    )
    .expect("valid public fixed-rate bond")
}

fn options(absolute: f64, cap: u32) -> SolverOptions {
    SolverOptions::new(absolute, 0.0, cap).expect("explicit positive tolerance/cap")
}

fn dirty_target(value: f64) -> BondPrice {
    BondPrice::new(value, BondPriceKind::Dirty).expect("finite signed per100 target")
}

/// FR-YR-02/04: a public dirty solve survives overflow of its unused clean view.
#[test]
fn ex_coupon_dirty_report_and_risk_survive_a_reachable_clean_view_overflow() {
    let settlement_date = date(2024, 7, 1);
    let bond = fixed_one_period(
        date(2025, 1, 1),
        100.0,
        power(1023) / 100.0,
        f64::MAX,
        Some(ExCouponTerms::new(
            settlement_date,
            ExCouponBoundary::OnOrAfter,
        )),
    );
    let convention =
        BondYieldConvention::new(DayCount::Thirty360BondBasis, Compounding::CONTINUOUS);
    let settlement = BondSettlement::new(settlement_date, PaymentDateInclusion::Exclude);
    let plan =
        BondPricingPlan::new(&bond, settlement, &convention, 8).expect("prepared ex-coupon plan");
    // Exact triple-product rounding gives coupon2^1023; half accrued minus
    // full detached coupon is -2^1022. Only MAX redemption remains entitled.
    assert_eq!(
        plan.accrued_amount().to_bits(),
        (SIGN | power(1022).to_bits())
    );
    assert_eq!(plan.entitled_flow_count(), 1);
    assert!(matches!(
        plan.evaluate(0.0),
        Err(BondError::Sum {
            phase: "clean price reduction",
            ..
        })
    ));

    let input_bracket = RootBracket::new(0.0, f64::MAX).expect("finite coordinate bracket");
    let report = plan
        .yield_from_price(dirty_target(f64::MAX), input_bracket, options(1.0, 1))
        .expect("lower dirty match needs only one valid callback");
    assert_eq!(report.dirty_amount().to_bits(), f64::MAX.to_bits());
    assert_eq!(report.yield_rate().to_bits(), 0);
    assert_eq!(
        report.solve_report().reason(),
        TerminationReason::FunctionZero
    );
    assert_eq!(report.solve_report().evaluations().function(), 1);
    assert_eq!(report.solve_report().evaluations().derivative(), 0);
    assert_eq!(report.solve_report().bracket(), input_bracket);
    assert_eq!(
        report
            .dirty_price()
            .expect("dirty quote is finite")
            .value_per_100_face()
            .to_bits(),
        f64::MAX.to_bits()
    );
    assert_eq!(
        report
            .amount_residual()
            .expect("exact dirty match")
            .to_bits(),
        0
    );

    let clean_error = report
        .clean_amount()
        .expect_err("MAX+2^1022 exceeds binary64");
    assert!(matches!(
        clean_error,
        BondError::Sum {
            phase: "clean price reduction",
            cause: SumError::Arithmetic {
                error: ValueError::ResultNotFinite {
                    value: NonFinite::PositiveInfinity,
                    ..
                },
                ..
            },
            ..
        }
    ));
    assert_eq!(
        report
            .clean_price()
            .expect_err("clean quote first forms clean amount"),
        clean_error
    );
    let root = RootError::from(clean_error.clone());
    let domain = root
        .source()
        .expect("typed bond source")
        .downcast_ref::<BondError>()
        .expect("exact BondError source");
    assert_eq!(domain, &clean_error);
    assert!(domain.source().expect("sum source").is::<SumError>());
    assert!(
        domain
            .source()
            .expect("sum source")
            .source()
            .expect("value source")
            .is::<ValueError>()
    );

    let risk = plan
        .risk(0.0)
        .expect("risk uses only the valid dirty value");
    assert_eq!(risk.dirty_amount().to_bits(), f64::MAX.to_bits());
    assert_eq!(
        risk.modified_duration()
            .expect("raw-range independent ratio")
            .to_bits(),
        0.5_f64.to_bits()
    );
    assert_eq!(
        risk.macaulay_duration()
            .expect("retained moment ratio")
            .to_bits(),
        0.5_f64.to_bits()
    );
    assert_eq!(
        risk.convexity()
            .expect("retained second-derivative ratio")
            .to_bits(),
        0.25_f64.to_bits()
    );
}

#[expect(
    clippy::panic,
    reason = "an unexpected typed hand error fails its regression"
)]
fn assert_underflow(error: BondError, phase: &'static str, negative: bool) {
    match &error {
        BondError::Value {
            index: None,
            phase: actual,
            cause:
                ValueError::ResultOutOfDomain {
                    quantity,
                    value,
                    domain,
                    ..
                },
            ..
        } => {
            assert_eq!(*actual, phase);
            assert_eq!(*quantity, phase);
            assert_eq!(value.to_bits(), if negative { SIGN } else { 0 });
            assert_eq!(
                domain.lower(),
                if negative {
                    Bound::Unbounded
                } else {
                    Bound::Excluded(0.0)
                }
            );
            assert_eq!(
                domain.upper(),
                if negative {
                    Bound::Excluded(0.0)
                } else {
                    Bound::Unbounded
                }
            );
        }
        other => panic!("expected signed {phase} underflow, got {other:?}"),
    }
    let root = RootError::from(error.clone());
    assert_eq!(root.kind(), "bond.value");
    let cause = root
        .source()
        .expect("root domain source")
        .downcast_ref::<BondError>()
        .expect("exact typed bond source");
    assert_eq!(cause, &error);
    assert!(cause.source().expect("numeric leaf").is::<ValueError>());
    assert!(cause.source().expect("numeric leaf").source().is_none());
}

/// FR-YR-10/14: negative observed dirty value changes raw signs, not durations.
#[test]
fn signed_subnormal_raw_underflow_preserves_zero_sign_and_interval_while_ratios_survive() {
    let tiny = f64::from_bits(1);
    // At t=.5: coupon -2*MIN_SUB plus redemption MIN_SUB gives dirty -MIN_SUB.
    let bond = fixed_one_period(date(2024, 7, 1), 1.0, -f64::from_bits(4), tiny, None);
    let convention =
        BondYieldConvention::new(DayCount::Thirty360BondBasis, Compounding::CONTINUOUS);
    let plan = BondPricingPlan::new(
        &bond,
        BondSettlement::new(date(2024, 1, 1), PaymentDateInclusion::Exclude),
        &convention,
        8,
    )
    .expect("prepared signed subnormal plan");
    let risk = plan
        .risk(-0.0)
        .expect("per-flow PVs remain nonzero and representable");
    assert_eq!(risk.yield_rate().to_bits(), 0);
    assert_eq!(risk.dirty_amount().to_bits(), SIGN | 1);
    assert_underflow(
        risk.price_derivative()
            .expect_err("+.5MIN_SUB rounds to +0"),
        "price derivative",
        false,
    );
    assert_underflow(
        risk.price_second_derivative()
            .expect_err("-.25MIN_SUB rounds to -0"),
        "price second derivative",
        true,
    );
    assert_underflow(
        risk.time_weighted_amount()
            .expect_err("-.5MIN_SUB rounds to -0"),
        "time weighted amount",
        true,
    );
    assert_eq!(
        risk.modified_duration()
            .expect("extended derivative over negative dirty")
            .to_bits(),
        0.5_f64.to_bits()
    );
    assert_eq!(
        risk.macaulay_duration()
            .expect("extended moment over negative dirty")
            .to_bits(),
        0.5_f64.to_bits()
    );
    assert_eq!(
        risk.convexity()
            .expect("extended second derivative over negative dirty")
            .to_bits(),
        0.25_f64.to_bits()
    );
    let report = plan
        .yield_from_price(
            dirty_target(-f64::from_bits(100)),
            RootBracket::new(0.0, 1.0).expect("bracket"),
            options(tiny, 1),
        )
        .expect("per100 target converts exactly to -MIN_SUB");
    assert_eq!(report.dirty_target_amount().to_bits(), SIGN | 1);
    assert_eq!(report.sensitivity(), BondYieldSensitivity::Uncertified);
    assert_eq!(
        report
            .dirty_price()
            .expect("negative subnormal quote")
            .value_per_100_face()
            .to_bits(),
        SIGN | 100
    );

    // At the same payment date with Include, the negative net price remains
    // nonzero but every horizon/moment/derivative is exactly zero. A negative
    // denominator does not introduce a negative zero or a zero-price error.
    let maturity_plan = BondPricingPlan::new(
        &bond,
        BondSettlement::new(date(2024, 7, 1), PaymentDateInclusion::Include),
        &convention,
        8,
    )
    .expect("included time-zero signed flows");
    let maturity_risk = maturity_plan
        .risk(0.0)
        .expect("negative time-zero dirty price");
    assert_eq!(maturity_risk.dirty_amount().to_bits(), SIGN | 1);
    for value in [
        maturity_risk.price_derivative(),
        maturity_risk.price_second_derivative(),
        maturity_risk.time_weighted_amount(),
        maturity_risk.modified_duration(),
        maturity_risk.macaulay_duration(),
        maturity_risk.convexity(),
    ] {
        assert_eq!(
            value
                .expect("zero numerator with nonzero dirty is defined")
                .to_bits(),
            0
        );
    }
    let constant = maturity_plan
        .yield_from_price(
            dirty_target(-f64::from_bits(100)),
            RootBracket::new(0.0, f64::MAX).expect("unqueried upper may be inadmissible"),
            options(tiny, 1),
        )
        .expect_err("negative exact time-zero sum has no identifiable yield");
    assert!(matches!(constant, BondYieldError::NoYieldSensitivity {
        dirty_target_amount, observed_lower_dirty_amount,
        constant_amount: Some(value), evaluations, ..
    } if dirty_target_amount.to_bits() == SIGN | 1
        && observed_lower_dirty_amount.to_bits() == SIGN | 1
        && value.to_bits() == SIGN | 1
        && evaluations.function() == 1 && evaluations.derivative() == 0));
}

/// FR-YR-06/10: individually unrepresentable risk contributions cancel publicly.
#[test]
fn huge_opposite_coupon_and_redemption_cancel_risk_before_raw_admission() {
    // At t2, coupon -2^1023 and redemption +2^1023 have individual
    // |P'|=2^1024 and |P''|=2^1025, yet each exact signed sum is zero.
    let bond = fixed_one_period(date(2026, 1, 1), 1.0, -power(1022), power(1023), None);
    let settlement = BondSettlement::new(date(2024, 1, 1), PaymentDateInclusion::Exclude);
    let convention =
        BondYieldConvention::new(DayCount::Thirty360BondBasis, Compounding::CONTINUOUS);
    let plan =
        BondPricingPlan::new(&bond, settlement, &convention, 8).expect("valid cancellation plan");
    let risk = plan
        .risk(0.0)
        .expect("wide terms are admitted only after exact cancellation");
    assert_eq!(risk.dirty_amount().to_bits(), 0);
    for value in [
        risk.price_derivative(),
        risk.price_second_derivative(),
        risk.time_weighted_amount(),
    ] {
        assert_eq!(value.expect("exactly canceled raw sum").to_bits(), 0);
    }
    for (measure, value) in [
        ("modified duration", risk.modified_duration()),
        ("macaulay duration", risk.macaulay_duration()),
        ("convexity", risk.convexity()),
    ] {
        assert!(
            matches!(value, Err(BondError::ZeroDirtyPrice { measure: actual, yield_rate, .. })
            if actual == measure && yield_rate.to_bits() == 0)
        );
    }
    let bracket = RootBracket::new(0.0, f64::MAX).expect("upper need not have valid factors");
    let error = plan
        .yield_from_price(dirty_target(0.0), bracket, options(1.0, 1))
        .expect_err("mathematically constant zero has no identifiable yield");
    assert!(matches!(error, BondYieldError::NoYieldSensitivity {
        bracket: actual, dirty_target_amount, observed_lower_dirty_amount,
        constant_amount: Some(constant), evaluations, ..
    } if actual == bracket && dirty_target_amount.to_bits() == 0
        && observed_lower_dirty_amount.to_bits() == 0 && constant.to_bits() == 0
        && evaluations.function() == 1 && evaluations.derivative() == 0));

    let mismatch = plan
        .yield_from_price(dirty_target(100.0), bracket, options(1.0, 1))
        .expect_err("constant zero cannot identify a yield for target amount1");
    assert!(matches!(mismatch, BondYieldError::ConstantPriceMismatch {
        bracket: actual, dirty_target_amount, observed_lower_dirty_amount,
        constant_amount: Some(constant), evaluations, ..
    } if actual == bracket && dirty_target_amount.to_bits() == 1.0_f64.to_bits()
        && observed_lower_dirty_amount.to_bits() == 0 && constant.to_bits() == 0
        && evaluations.function() == 1 && evaluations.derivative() == 0));

    let simple = BondYieldConvention::new(DayCount::Thirty360BondBasis, Compounding::SIMPLE);
    let simple_plan = BondPricingPlan::new(&bond, settlement, &simple, 8)
        .expect("same exact-pair certificate under Simple");
    let failure = simple_plan
        .yield_from_price(
            dirty_target(0.0),
            RootBracket::new(-1.0, 0.0).expect("lower produces factor-1 at t2"),
            options(1.0, 1),
        )
        .expect_err("one normal lower evaluation precedes constant conclusions");
    assert!(matches!(&failure, BondYieldError::Callback {
        stage: EvaluationStage::LowerEndpoint, point, evaluations,
        cause: BondError::Rate {
            index: Some(0), phase: "discount factor",
            cause: RateError::Value(ValueError::ResultOutOfDomain {
                quantity: "compound factor", value, domain, ..
            }), ..
        }, ..
    } if point.to_bits() == (-1.0_f64).to_bits()
        && evaluations.function() == 1 && evaluations.derivative() == 0
        && value.to_bits() == (-1.0_f64).to_bits()
        && domain.lower() == Bound::Excluded(0.0) && domain.upper() == Bound::Unbounded));
    let root = RootError::from(failure.clone());
    let yield_cause = root
        .source()
        .expect("typed yield cause")
        .downcast_ref::<BondYieldError>()
        .expect("exact yield wrapper");
    assert_eq!(yield_cause, &failure);
    let bond_cause = yield_cause
        .source()
        .expect("typed bond cause")
        .downcast_ref::<BondError>()
        .expect("exact bond wrapper");
    let rate_cause = bond_cause
        .source()
        .expect("typed rate cause")
        .downcast_ref::<RateError>()
        .expect("exact rate wrapper");
    assert!(
        rate_cause
            .source()
            .expect("typed numeric cause")
            .is::<ValueError>()
    );
    assert!(
        rate_cause
            .source()
            .expect("typed numeric cause")
            .source()
            .is_none()
    );
}

/// FR-YR-03/14: overflowing unscaled differences occur on a public error path.
#[test]
fn opposite_maximum_target_keeps_both_not_bracketed_residuals_finite_and_scaled() {
    let issue = date(2024, 1, 1);
    let maturity = date(2025, 1, 1);
    let bond = Bond::zero_coupon(
        issue,
        maturity,
        100.0,
        SimpleCashFlow::new(maturity, f64::MAX).expect("MAX redemption"),
    )
    .expect("finite zero-coupon bond");
    let convention = BondYieldConvention::new(DayCount::Thirty360BondBasis, Compounding::SIMPLE);
    let plan = BondPricingPlan::new(
        &bond,
        BondSettlement::new(issue, PaymentDateInclusion::Exclude),
        &convention,
        8,
    )
    .expect("one-year direct horizon");
    let bracket = RootBracket::new(0.0, 1.0).expect("ordered explicit bracket");
    let error = plan
        .yield_from_price(dirty_target(-f64::MAX), bracket, options(1.0, 2))
        .expect_err("positive dirty prices cannot bracket a negative target");
    // Lower: (MAX-(-MAX))/2=MAX. Upper: (MAX/2+MAX)/2=RN(3MAX/4).
    // Fraction(3*(2^53-1)*2^971,4) rounds to this exact binary64 bit pattern.
    assert!(
        matches!(&error, BondYieldError::Solver(SolverError::NotBracketed {
        bracket: actual, lower_value, upper_value, evaluations, ..
    }) if *actual == bracket && lower_value.to_bits() == f64::MAX.to_bits()
        && upper_value.to_bits() == 0x7fe7_ffff_ffff_ffff
        && evaluations.function() == 2 && evaluations.derivative() == 0)
    );
    let root = RootError::from(error.clone());
    assert_eq!(root.kind(), "bond_yield.solver");
    let yield_error = root
        .source()
        .expect("yield source")
        .downcast_ref::<BondYieldError>()
        .expect("exact typed yield source");
    assert_eq!(yield_error, &error);
    assert!(
        yield_error
            .source()
            .expect("solver cause")
            .is::<SolverError>()
    );
}

/// FR-YR-04: returned payload belongs to the best initial endpoint, not last call.
#[test]
fn two_call_coordinate_termination_returns_the_earlier_endpoint_payload() {
    let issue = date(2024, 1, 1);
    let maturity = date(2025, 1, 1);
    let bond = Bond::zero_coupon(
        issue,
        maturity,
        100.0,
        SimpleCashFlow::new(maturity, 100.0).expect("redemption"),
    )
    .expect("zero coupon");
    let convention = BondYieldConvention::new(DayCount::Thirty360BondBasis, Compounding::SIMPLE);
    let plan = BondPricingPlan::new(
        &bond,
        BondSettlement::new(issue, PaymentDateInclusion::Exclude),
        &convention,
        8,
    )
    .expect("one-year plan");
    let bracket = RootBracket::new(0.0, 3.0).expect("bracket");
    let report = plan
        .yield_from_price(dirty_target(75.0), bracket, options(3.0, 2))
        .expect("full-width criterion succeeds after exactly two endpoint calls");
    // Calls: lower0 -> dirty100/residual25, upper3 -> dirty25/residual-50.
    // Best is the earlier lower endpoint, and no remaining budget is needed.
    assert_eq!(
        report.solve_report().reason(),
        TerminationReason::BracketTolerance
    );
    assert_eq!(report.solve_report().evaluations().function(), 2);
    assert_eq!(report.solve_report().evaluations().derivative(), 0);
    assert_eq!(report.solve_report().iterations(), 0);
    assert_eq!(report.yield_rate().to_bits(), 0);
    assert_eq!(report.dirty_amount().to_bits(), 100.0_f64.to_bits());
    assert_eq!(
        report.solve_report().residual().to_bits(),
        25.0_f64.to_bits()
    );
    assert_eq!(
        report
            .amount_residual()
            .expect("finite amount difference")
            .to_bits(),
        25.0_f64.to_bits()
    );
    assert_eq!(report.solve_report().bracket(), bracket);
}

/// FR-YR-04/06/12: budget checks and zero amounts cannot hide invalid factors.
#[test]
fn endpoint_budget_zero_short_circuit_and_zero_coupon_factor_failure_are_ordered() {
    let bond = fixed_one_period(date(2026, 1, 1), 100.0, 0.0, 100.0, None);
    let convention = BondYieldConvention::new(DayCount::Thirty360BondBasis, Compounding::SIMPLE);
    let plan = BondPricingPlan::new(
        &bond,
        BondSettlement::new(date(2024, 1, 1), PaymentDateInclusion::Exclude),
        &convention,
        8,
    )
    .expect("zero coupon amount retained alongside positive redemption");
    assert_eq!(plan.entitled_flow_count(), 2);
    let bracket = RootBracket::new(0.0, f64::MAX).expect("upper Simple factor overflows");
    let report = plan
        .yield_from_price(dirty_target(100.0), bracket, options(1.0, 1))
        .expect("exact lower zero stops before the invalid unqueried upper");
    assert_eq!(report.solve_report().evaluations().function(), 1);
    assert_eq!(
        report.solve_report().reason(),
        TerminationReason::FunctionZero
    );

    let budget = plan
        .yield_from_price(dirty_target(50.0), bracket, options(1.0, 1))
        .expect_err("one call cannot query the invalid upper");
    assert!(
        matches!(budget, BondYieldError::Solver(SolverError::EvaluationLimit {
        evaluations, best: Some((point, residual)), ..
    }) if evaluations.function() == 1 && evaluations.derivative() == 0
        && point.to_bits() == 0 && residual.to_bits() == 50.0_f64.to_bits())
    );

    let failure = plan
        .yield_from_price(dirty_target(50.0), bracket, options(1.0, 2))
        .expect_err("second invocation validates the zero coupon's invalid factor first");
    assert!(matches!(&failure, BondYieldError::Callback {
        stage: EvaluationStage::UpperEndpoint, point, evaluations,
        cause: BondError::Rate {
            index: Some(0), phase: "discount factor",
            cause: RateError::Value(ValueError::ResultNotFinite {
                quantity: "compound factor", value: NonFinite::PositiveInfinity, ..
            }), ..
        }, ..
    } if point.to_bits() == f64::MAX.to_bits()
        && evaluations.function() == 2 && evaluations.derivative() == 0));
    let risk = plan
        .risk(f64::MAX)
        .expect_err("risk also validates zero-flow factor before skipping its terms");
    let cause = failure
        .source()
        .expect("typed callback source")
        .downcast_ref::<BondError>()
        .expect("exact BondError callback source");
    assert_eq!(cause, &risk);
}
