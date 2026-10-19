//! Pre-implementation public numerical controls for FR-GILT-06..14/20..23.
//!
//! Expectations were derived from the frozen contract before gilt production.
//! Exact zero-yield anchors, actual source-domain range controls and counted
//! solver failures are separate from conditional nonzero-yield comparisons.

#![expect(clippy::expect_used, reason = "explicit valid hand-test inputs")]

use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    BondPrice, BondPriceKind, BondYieldSensitivity, ConventionalGilt, GiltError, GiltFirstPeriod,
    GiltPricingPlan, GiltYieldError,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::error::{Interval, NonFinite, ValueError};
use sabiquant::math::{
    EvaluationStage, RootBracket, SolverError, SolverOptions, TerminationReason,
};
use sabiquant::rates::RateError;
use sabiquant::time::Date;

const ABOVE_FLOOR: f64 = f64::from_bits(0xbfff_ffff_ffff_ffff);

fn d(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid civil date")
}

fn amount(coefficient: i128, scale: u8) -> DecimalAmount {
    DecimalAmount::new(coefficient, scale).expect("valid exact amount")
}

fn quote(value: f64, kind: BondPriceKind) -> BondPrice {
    BondPrice::new(value, kind).expect("finite price")
}

fn options(cap: u32) -> SolverOptions {
    SolverOptions::new(1e-11, 0.0, cap).expect("explicit solver policy")
}

fn regular(periods: u32, coupon: DecimalAmount) -> ConventionalGilt {
    let issue = d(2000, 1, 1);
    let dates: Vec<_> = (0..=periods)
        .map(|k| {
            issue
                .checked_add_months(6 * i64::from(k))
                .expect("bounded cycle")
        })
        .collect();
    ConventionalGilt::new(
        issue,
        dates[dates.len() - 1],
        coupon,
        dates[1],
        dates.clone(),
        dates[1..].to_vec(),
        EndOfMonth::Ignore,
        ConventionalGilt::MAX_QUASI_DATES,
    )
    .expect("complete directly anchored regular cycle")
}

fn half_period(coupon: DecimalAmount) -> ConventionalGilt {
    let issue = d(2024, 12, 7);
    let maturity = d(2025, 6, 7);
    ConventionalGilt::new(
        issue,
        maturity,
        coupon,
        maturity,
        vec![issue, maturity],
        vec![maturity],
        EndOfMonth::Ignore,
        2,
    )
    .expect("182-day regular interval")
}

/// FR-GILT-09/12/13: lambda=1/2, A=102.5, delta=2, all operands exact.
#[test]
fn exact_zero_yield_price_and_all_six_half_period_views() {
    let gilt = half_period(amount(5, 0));
    let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 1).expect("91 days remain");
    assert_eq!((plan.remaining_days(), plan.quasi_period_days()), (91, 182));
    assert_eq!(plan.remaining_quasi_periods(), 0);
    let value = plan.evaluate(0.0).expect("identity discount");
    let risk = plan.risk(0.0).expect("complete risk");
    assert_eq!(value.dirty_per_100(), 102.5);
    assert_eq!(value.accrued_per_100(), 1.25);
    assert_eq!(
        value.clean_price().expect("clean").value_per_100_face(),
        101.25
    );
    assert_eq!(risk.price_derivative().expect("d1"), -25.625);
    assert_eq!(risk.price_second_derivative().expect("d2"), 19.21875);
    assert_eq!(risk.time_weighted_amount().expect("moment"), 25.625);
    assert_eq!(risk.modified_duration().expect("modified"), 0.25);
    assert_eq!(risk.macaulay_duration().expect("macaulay"), 0.25);
    assert_eq!(risk.convexity().expect("convexity"), 0.1875);
    assert_eq!(risk.dirty_per_100(), value.dirty_per_100());
}

/// FR-GILT-06/12/13: exact two-payment algebra, not a planted-yield round trip.
#[test]
fn exact_two_payment_moments_use_distinct_quasi_coordinates() {
    let issue = d(2024, 12, 7);
    let next = d(2025, 6, 7);
    let maturity = d(2025, 12, 7);
    let gilt = ConventionalGilt::new(
        issue,
        maturity,
        amount(6, 0),
        next,
        vec![issue, next, maturity],
        vec![next, maturity],
        EndOfMonth::Ignore,
        3,
    )
    .expect("regular two-payment source");
    let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 2).expect("half first period");
    let risk = plan.risk(-0.0).expect("canonical identity");
    assert_eq!(risk.yield_rate().to_bits(), 0);
    assert_eq!(risk.dirty_per_100(), 106.0);
    assert_eq!(risk.price_derivative().expect("d1"), -78.0);
    assert_eq!(risk.price_second_derivative().expect("d2"), 97.125);
    assert_eq!(risk.time_weighted_amount().expect("moment"), 78.0);
    assert_eq!(risk.modified_duration().expect("modified"), 39.0 / 53.0);
    assert_eq!(risk.macaulay_duration().expect("macaulay"), 39.0 / 53.0);
    assert_eq!(risk.convexity().expect("convexity"), 777.0 / 848.0);
    assert_eq!(plan.bundle_quasi_index(0), Some(1));
    assert_eq!(plan.bundle_quasi_index(1), Some(2));
    assert_eq!(plan.bundle_quasi_index(2), None);
}

/// FR-GILT-06/08/12: a zero dividend must never remove the grouped par100.
#[test]
fn zero_final_coupon_keeps_positive_price_and_redemption_risk() {
    let gilt = half_period(amount(0, 0));
    let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 1).expect("one bundle");
    let risk = plan.risk(0.0).expect("zero coupon, positive principal");
    assert_eq!(plan.bundle_count(), 1);
    assert_eq!(risk.dirty_per_100(), 100.0);
    assert_eq!(risk.price_derivative().expect("d1"), -25.0);
    assert_eq!(risk.price_second_derivative().expect("d2"), 18.75);
    assert_eq!(risk.time_weighted_amount().expect("moment"), 25.0);
    assert_eq!(risk.modified_duration().expect("modified"), 0.25);
    assert_eq!(risk.macaulay_duration().expect("macaulay"), 0.25);
    assert_eq!(risk.convexity().expect("convexity"), 0.1875);
}

/// FR-GILT-07/11: tiny computed-flat observations do not erase ideal sensitivity.
#[test]
fn adjacent_zero_quotes_keep_identity_price_and_strict_ideal_sensitivity() {
    let gilt = half_period(amount(5, 0));
    let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 1).expect("plan");
    for y in [0.0, -0.0, f64::from_bits(1), -f64::from_bits(1)] {
        assert_eq!(plan.evaluate(y).expect("tiny path").dirty_per_100(), 102.5);
        assert_eq!(
            plan.risk(y)
                .expect("tiny risk")
                .price_derivative()
                .expect("d1"),
            -25.625
        );
    }
    let report = plan
        .yield_from_price(
            quote(102.5, BondPriceKind::Dirty),
            RootBracket::new(-f64::from_bits(1), f64::from_bits(1)).expect("adjacent bracket"),
            options(1),
        )
        .expect("lower computed zero");
    assert_eq!(report.solve_report().evaluations().function(), 1);
    assert_eq!(
        report.sensitivity(),
        BondYieldSensitivity::MathematicallyStrictlyDecreasing
    );
    assert_eq!(
        report.yield_rate().to_bits(),
        (-f64::from_bits(1)).to_bits()
    );
}

/// FR-GILT-07/20: quote validation precedes all bundles even for C=0.
#[test]
fn complete_quote_domain_is_shared_by_evaluation_and_risk() {
    let gilt = regular(2, amount(0, 0));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 2).expect("plan");
    for y in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -2.0, -3.0] {
        for error in [
            plan.evaluate(y).expect_err("invalid quote"),
            plan.risk(y).expect_err("invalid quote"),
        ] {
            match error {
                GiltError::Rate {
                    index: None,
                    phase: "yield quote",
                    cause: RateError::Value(cause),
                    ..
                } => match cause {
                    ValueError::NotFinite {
                        parameter: "rate", ..
                    } if !y.is_finite() => {}
                    ValueError::OutOfDomain {
                        parameter: "rate",
                        value,
                        domain,
                        ..
                    } => {
                        assert_eq!(value.to_bits(), y.to_bits());
                        assert_eq!(domain, Interval::greater_than(-2.0));
                    }
                    other => panic!("incorrect quote cause: {other:?}"),
                },
                other => panic!("incorrect quote wrapper: {other:?}"),
            }
        }
    }
}

/// FR-GILT-06/07/08/20: zero coupons still observe the first overflowing factor.
#[test]
fn zero_coupon_factor_failure_keeps_original_index_before_redemption() {
    let gilt = regular(21, amount(0, 0));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 21).expect("21 bundles");
    assert_eq!(plan.bundle_count(), 21);
    // q=2^-53: lambda19 gives 2^1007; lambda20 gives 2^1060.
    for error in [
        plan.evaluate(ABOVE_FLOOR).expect_err("factor overflow"),
        plan.risk(ABOVE_FLOOR).expect_err("factor overflow"),
    ] {
        assert!(
            matches!(
                error,
                GiltError::Rate {
                    index: Some(20),
                    phase: "discount factor",
                    cause: RateError::Value(ValueError::ResultNotFinite {
                        quantity: "discount factor",
                        value: NonFinite::PositiveInfinity,
                        ..
                    }),
                    ..
                }
            ),
            "{error:?}"
        );
    }
}

/// FR-GILT-08/20: a tiny nonzero coupon is not an actual zero coupon shortcut.
#[test]
fn nonzero_source_pv_underflow_precedes_later_discount_underflow() {
    let gilt = regular(2, amount(1, 38));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 2).expect("plan");
    for error in [
        plan.evaluate(f64::MAX).expect_err("PV underflow"),
        plan.risk(f64::MAX).expect_err("PV underflow"),
    ] {
        match error {
            GiltError::Value {
                index: Some(1),
                phase: "source present value",
                cause:
                    ValueError::ResultOutOfDomain {
                        quantity: "gilt present value",
                        value,
                        domain,
                        ..
                    },
                ..
            } => {
                assert_eq!(value.to_bits(), 0);
                assert_eq!(domain, Interval::greater_than(0.0));
            }
            other => panic!("wrong original failure: {other:?}"),
        }
    }
}

/// FR-GILT-08/20: full-C PV range is distinct from admissible discount range.
#[test]
fn source_pv_overflow_precedes_a_later_factor_failure() {
    let gilt = regular(21, amount(i128::MAX, 0));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 21).expect("plan");
    // C/2 is about 2^126; q^-17=2^901, while q^-16=2^848.
    for error in [
        plan.evaluate(ABOVE_FLOOR).expect_err("PV overflow"),
        plan.risk(ABOVE_FLOOR).expect_err("PV overflow"),
    ] {
        assert!(
            matches!(
                error,
                GiltError::Value {
                    index: Some(17),
                    phase: "source present value",
                    cause: ValueError::ResultNotFinite {
                        quantity: "gilt present value",
                        value: NonFinite::PositiveInfinity,
                        ..
                    },
                    ..
                }
            ),
            "{error:?}"
        );
    }
}

/// FR-GILT-08/20: successful individual PVs do not guarantee a finite reduction.
#[test]
fn positive_dirty_reduction_overflow_is_reported_after_all_pvs() {
    let gilt = regular(2162, amount(1_i128 << 126, 0));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 2162).expect("bounded long cycle");
    // q=.75: final PV log2≈1022.311, geometric sum log2≈1024.311.
    // The margin is far larger than the conditional elementary allowance.
    for error in [
        plan.evaluate(-0.5).expect_err("dirty sum overflow"),
        plan.risk(-0.5).expect_err("dirty sum overflow"),
    ] {
        assert!(
            matches!(
                error,
                GiltError::Sum {
                    phase: "dirty price reduction",
                    ..
                }
            ),
            "{error:?}"
        );
    }
}

/// FR-GILT-07/20: y rounds the exact lambda2 half-MIN_SUB threshold input.
#[test]
fn direct_discount_zero_midpoint_has_typed_numerical_boundary() {
    let gilt = regular(2, amount(0, 0));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 2).expect("plan");
    // Independent Decimal180: RN(2*exp(1075*ln(2)/2)-2).
    let y = f64::from_bits(0x6196_a09e_667f_3bcd);
    for error in [
        plan.evaluate(y).expect_err("uncertain factor"),
        plan.risk(y).expect_err("uncertain factor"),
    ] {
        match error {
            GiltError::Rate {
                index: Some(2),
                phase: "discount factor",
                cause:
                    RateError::NumericalBoundary {
                        quantity: "discount factor",
                        logarithm,
                        ..
                    },
                ..
            } => {
                assert!((-746.0..-744.0).contains(&logarithm));
            }
            other => panic!("wrong boundary error: {other:?}"),
        }
    }
}

/// FR-GILT-12/13: convexity can underflow although its raw numerator is finite.
#[test]
fn normalized_only_convexity_underflow_from_a_valid_public_source() {
    let gilt = half_period(amount(1_i128 << 126, 0));
    let settlement = d(2025, 6, 6);
    let plan = GiltPricingPlan::new(&gilt, settlement, 1).expect("lambda1/182");
    let risk = plan
        .risk(2_f64.powi(540))
        .expect("finite price and extended sums");
    assert!(risk.price_second_derivative().expect("finite raw d2") > 0.0);
    assert!(risk.price_derivative().expect("finite raw d1") < 0.0);
    assert!(risk.modified_duration().expect("modified finite") > 0.0);
    assert!(risk.macaulay_duration().expect("macaulay finite") > 0.0);
    match risk.convexity().expect_err("positive normalized underflow") {
        GiltError::Value {
            index: None,
            phase: "convexity",
            cause:
                ValueError::ResultOutOfDomain {
                    quantity: "convexity",
                    value,
                    domain,
                    ..
                },
            ..
        } => {
            assert_eq!(value.to_bits(), 0);
            assert_eq!(domain, Interval::greater_than(0.0));
        }
        other => panic!("incorrect normalized error: {other:?}"),
    }
}

/// FR-GILT-12/13/20: raw signed underflow and finite normalized duration coexist.
#[test]
fn raw_signed_underflow_keeps_a_positive_subnormal_modified_duration() {
    let gilt = half_period(amount(0, 0));
    let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 1).expect("lambda one half");
    let risk = plan
        .risk(f64::MAX)
        .expect("finite price, extended derivatives");
    match risk.price_derivative().expect_err("negative raw underflow") {
        GiltError::Value {
            index: None,
            phase: "price derivative",
            cause: ValueError::ResultOutOfDomain { value, domain, .. },
            ..
        } => {
            assert_eq!(value.to_bits(), 1_u64 << 63);
            assert_eq!(domain, Interval::less_than(0.0));
        }
        other => panic!("incorrect raw derivative underflow: {other:?}"),
    }
    match risk
        .price_second_derivative()
        .expect_err("positive raw underflow")
    {
        GiltError::Value {
            index: None,
            phase: "price second derivative",
            cause: ValueError::ResultOutOfDomain { value, domain, .. },
            ..
        } => {
            assert_eq!(value.to_bits(), 0);
            assert_eq!(domain, Interval::greater_than(0.0));
        }
        other => panic!("incorrect raw second derivative underflow: {other:?}"),
    }
    let modified = risk.modified_duration().expect("direct extended ratio");
    assert!(modified > 0.0 && modified < f64::MIN_POSITIVE);
    assert!(
        risk.time_weighted_amount()
            .expect("moment cannot vanish here")
            > 0.0
    );
    assert!((risk.macaulay_duration().expect("direct moment ratio") - 0.25).abs() <= f64::EPSILON);
}

/// FR-GILT-12/13: direct normalization must not depend on raw IEEE admission.
#[test]
fn raw_derivative_overflow_keeps_normalized_views_available() {
    let gilt = regular(19, amount(0, 0));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 19).expect("plan");
    let risk = plan.risk(ABOVE_FLOOR).expect("finite par PV");
    assert!(matches!(
        risk.price_derivative(),
        Err(GiltError::Value {
            cause: ValueError::ResultNotFinite {
                value: NonFinite::NegativeInfinity,
                ..
            },
            ..
        })
    ));
    assert!(matches!(
        risk.price_second_derivative(),
        Err(GiltError::Value {
            cause: ValueError::ResultNotFinite {
                value: NonFinite::PositiveInfinity,
                ..
            },
            ..
        })
    ));
    assert!(risk.time_weighted_amount().expect("finite moment") > 0.0);
    let expected_modified = 19.0 * 2_f64.powi(52);
    assert!(
        (risk.modified_duration().expect("direct ratio") / expected_modified - 1.0).abs()
            <= 4.0 * f64::EPSILON
    );
    assert!(
        (risk.macaulay_duration().expect("direct moment ratio") - 9.5).abs()
            <= 4.0 * f64::EPSILON * 9.5
    );
    assert!(risk.convexity().expect("normalized d2 finite") > 0.0);
}

/// FR-GILT-12/13: moment overflow is reachable with genuine positive source terms.
#[test]
fn raw_moment_overflow_does_not_invalidate_macaulay_duration() {
    let gilt = regular(19, amount(1_i128 << 15, 0));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 19).expect("plan");
    let risk = plan.risk(ABOVE_FLOOR).expect("all PVs finite");
    assert!(matches!(
        risk.time_weighted_amount(),
        Err(GiltError::Value {
            phase: "time weighted amount",
            cause: ValueError::ResultNotFinite {
                value: NonFinite::PositiveInfinity,
                ..
            },
            ..
        })
    ));
    let macaulay = risk.macaulay_duration().expect("retained numerator ratio");
    assert!((macaulay - 9.5).abs() <= 8.0 * f64::EPSILON * 9.5);
}

/// FR-GILT-09..11: target-driven exact y0 controls share one native shift.
#[test]
fn clean_dirty_targets_retain_policy_metadata_and_one_call_payload() {
    let gilt = half_period(amount(5, 0));
    let settlement = d(2025, 3, 8);
    let plan = GiltPricingPlan::new(&gilt, settlement, 1).expect("plan");
    let bracket = RootBracket::new(0.0, f64::MAX).expect("explicit bracket");
    let policy = options(1);
    for target in [
        quote(102.5, BondPriceKind::Dirty),
        quote(101.25, BondPriceKind::Clean),
    ] {
        let report = plan
            .yield_from_price(target, bracket, policy)
            .expect("lower exact root");
        assert_eq!(report.target(), target);
        assert_eq!(report.dirty_target_per_100(), 102.5);
        assert_eq!(report.accrued_per_100(), 1.25);
        assert_eq!(report.dirty_per_100(), 102.5);
        assert_eq!(
            report
                .amount_residual()
                .expect("cached exact difference")
                .to_bits(),
            0
        );
        assert_eq!(report.initial_bracket(), bracket);
        assert_eq!(report.options(), policy);
        assert_eq!(
            report.solve_report().reason(),
            TerminationReason::FunctionZero
        );
        assert_eq!(report.solve_report().evaluations().function(), 1);
        assert_eq!(report.solve_report().evaluations().derivative(), 0);
        assert_eq!(report.yield_rate().to_bits(), 0);
        assert_eq!(report.residual_scale_exponent(), 0);
        assert_eq!(report.issue_date(), gilt.issue_date());
        assert_eq!(report.maturity_date(), gilt.maturity_date());
        assert_eq!(report.first_period(), GiltFirstPeriod::Regular);
        assert_eq!(report.settlement_date(), settlement);
        assert_eq!(report.next_quasi_date(), gilt.maturity_date());
        assert_eq!(report.source_version(), gilt.source_version());
    }
}

/// FR-GILT-10/20: incompatible resolved targets perform no quote callback.
#[test]
fn nonpositive_resolved_target_precedes_invalid_lower_quote() {
    let gilt = half_period(amount(5, 0));
    let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 1).expect("plan");
    for target in [
        quote(0.0, BondPriceKind::Dirty),
        quote(-1.0, BondPriceKind::Dirty),
        quote(-1.25, BondPriceKind::Clean),
    ] {
        let error = plan
            .yield_from_price(
                target,
                RootBracket::new(-3.0, -2.0).expect("finite bracket"),
                options(2),
            )
            .expect_err("no positive-price target");
        match error {
            GiltYieldError::NonPositiveTarget {
                target: original,
                dirty_target_per_100,
                accrued_per_100,
                evaluations,
                ..
            } => {
                assert_eq!(original, target);
                assert!(dirty_target_per_100 <= 0.0);
                assert_eq!(accrued_per_100, 1.25);
                assert_eq!(evaluations.function(), 0);
                assert_eq!(evaluations.derivative(), 0);
            }
            other => panic!("incorrect target precedence: {other:?}"),
        }
    }
}

/// FR-GILT-11: a successful lower callback does not inspect an invalid upper factor.
#[test]
fn lower_computed_zero_avoids_unqueried_upper_factor_failure() {
    let gilt = regular(2, amount(0, 0));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 2).expect("plan");
    assert!(plan.evaluate(f64::MAX).is_err());
    let report = plan
        .yield_from_price(
            quote(100.0, BondPriceKind::Dirty),
            RootBracket::new(0.0, f64::MAX).expect("bracket"),
            options(1),
        )
        .expect("one lower callback");
    assert_eq!(report.solve_report().evaluations().function(), 1);
    assert_eq!(
        report.solve_report().reason(),
        TerminationReason::FunctionZero
    );
    assert_eq!(report.dirty_per_100(), 100.0);
}

/// FR-GILT-11/20: failed invocations count; an unperformed over-budget call does not.
#[test]
fn lower_upper_callback_errors_and_preinvocation_budget_are_distinct() {
    let gilt = regular(2, amount(0, 0));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 2).expect("plan");
    let target = quote(99.0, BondPriceKind::Dirty);
    let invalid_lower = plan
        .yield_from_price(
            target,
            RootBracket::new(-2.0, 0.0).expect("bracket"),
            options(2),
        )
        .expect_err("lower quote error");
    assert!(matches!(invalid_lower, GiltYieldError::Callback {
        stage: EvaluationStage::LowerEndpoint, evaluations,
        cause: GiltError::Rate { index: None, phase: "yield quote", .. }, ..
    } if evaluations.function()==1 && evaluations.derivative()==0));
    let bracket = RootBracket::new(0.0, f64::MAX).expect("bracket");
    let cap_error = plan
        .yield_from_price(target, bracket, options(1))
        .expect_err("upper call exceeds cap");
    assert!(
        matches!(cap_error, GiltYieldError::Solver(SolverError::EvaluationLimit { evaluations, .. }) if evaluations.function()==1)
    );
    let callback_error = plan
        .yield_from_price(target, bracket, options(2))
        .expect_err("upper factor underflow");
    assert!(matches!(callback_error, GiltYieldError::Callback {
        stage: EvaluationStage::UpperEndpoint, point, evaluations,
        cause: GiltError::Rate { index: Some(2), phase: "discount factor", .. }, ..
    } if point.to_bits()==f64::MAX.to_bits() && evaluations.function()==2 && evaluations.derivative()==0));
}

/// FR-GILT-11/22: y=2*((102.5/T)^2-1), independent of evaluation output.
#[test]
fn independent_half_period_inverse_anchors_have_conditioned_error_budget() {
    let gilt = half_period(amount(5, 0));
    let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 1).expect("plan");
    for (target, root, lower, upper) in [
        (102.5, 0.0, -0.1, 0.1),
        (51.25, 6.0, 5.0, 7.0),
        (205.0, -1.5, -1.6, -1.4),
    ] {
        let report = plan
            .yield_from_price(
                quote(target, BondPriceKind::Dirty),
                RootBracket::new(lower, upper).expect("root bracket"),
                options(200),
            )
            .expect("bounded inverse");
        let final_bracket = report.solve_report().bracket();
        // On these brackets P<=230, |P'|>2. Exact lambda=.5 and A=102.5.
        // The selected4u log/exp profile, exponent RN, PV RN and reduction
        // give uniform absolute price error <2^-40. Divide by slope floor2;
        // include cached amount residual and full final bracket width.
        let bound = (final_bracket.upper() - final_bracket.lower())
            + report
                .amount_residual()
                .expect("unscaled cached residual")
                .abs()
                / 2.0
            + 2_f64.powi(-40);
        assert!((report.yield_rate() - root).abs() <= bound);
        assert!(report.solve_report().evaluations().function() <= 200);
        assert_eq!(report.solve_report().evaluations().derivative(), 0);
    }
}

/// FR-GILT-01/21/23: immutable borrowed plans and retained reports share safely.
#[test]
fn scoped_threads_reuse_one_prepared_plan_and_owned_exact_reports() {
    let gilt = half_period(amount(5, 0));
    let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 1).expect("prepared plan");
    let expected = plan.evaluate(0.0).expect("exact value");
    let retained_risk = plan.risk(0.0).expect("retained extended state");
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let plan_ref = &plan;
            let risk_ref = &retained_risk;
            scope.spawn(move || {
                for _ in 0..16 {
                    assert_eq!(plan_ref.evaluate(0.0).expect("shared evaluation"), expected);
                    assert_eq!(risk_ref.price_derivative().expect("shared getter"), -25.625);
                    assert_eq!(
                        plan_ref.risk(0.0).expect("independent workspace"),
                        *risk_ref
                    );
                }
            });
        }
    });
}

/// FR-GILT-02/06/12/14/21: actual civil-domain maximum, without an invented date.
#[test]
fn complete_year_one_to_year_9999_cycle_keeps_every_zero_coupon_bundle() {
    let maturity = Date::MAX;
    let periods = 19_997_u32;
    let dates: Vec<_> = (0..=periods)
        .map(|k| {
            maturity
                .checked_add_months(-6 * i64::from(periods - k))
                .expect("direct six-month offset remains inside Date")
                .last_day_of_month()
        })
        .collect();
    assert_eq!(dates[0], d(1, 6, 30));
    assert_eq!(dates[dates.len() - 1], maturity);
    let count = usize::try_from(periods).expect("bounded count");
    let gilt = ConventionalGilt::new(
        dates[0],
        maturity,
        amount(0, 0),
        dates[1],
        dates.clone(),
        dates[1..].to_vec(),
        EndOfMonth::Preserve,
        ConventionalGilt::MAX_QUASI_DATES,
    )
    .expect("maximal complete civil cycle");
    assert!(matches!(
        GiltPricingPlan::new(&gilt, gilt.issue_date(), count - 1),
        Err(GiltError::TooManyBundles { count: required, cap, .. })
            if required == count && cap == count - 1
    ));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), count).expect("exact count cap");
    assert_eq!(plan.bundle_count(), count);
    assert_eq!(plan.remaining_quasi_periods(), periods - 1);
    assert_eq!(plan.bundle_quasi_index(count - 1), Some(count));
    assert_eq!(plan.bundle_quasi_index(count), None);
    let risk = plan
        .risk(0.0)
        .expect("zero coupons, positive far principal");
    assert_eq!(risk.dirty_per_100(), 100.0);
    assert_eq!(risk.price_derivative().expect("exact d1"), -999_850.0);
    assert_eq!(
        risk.price_second_derivative().expect("exact d2"),
        9_997_500_150.0
    );
    assert_eq!(
        risk.time_weighted_amount().expect("exact moment"),
        999_850.0
    );
    assert_eq!(risk.modified_duration().expect("exact modified"), 9_998.5);
    assert_eq!(risk.macaulay_duration().expect("exact macaulay"), 9_998.5);
    assert_eq!(risk.convexity().expect("exact convexity"), 99_975_001.5);
}
