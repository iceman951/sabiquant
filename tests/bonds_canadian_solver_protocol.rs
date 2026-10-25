//! Postproduction public controls for FR-CAD-54/55/58/71 and FR-MATH-01.
//!
//! These derived controls are not frozen blind hands. For a zero-coupon bond
//! settled one day before its terminal payment, the selected MM observations
//! are factor=RN_even(365/(365+y)), then dirty=RN_even(100*factor).
//! Constants below were derived before Rust execution with independent
//! unbounded Fraction arithmetic and integer quotient/remainder IEEE rounding.
//! Secant proposals round each prescribed arithmetic operation separately.
//! No observed library value supplies an expectation or an ideal-root claim.

#![expect(
    clippy::expect_used,
    reason = "explicit valid postproduction protocol inputs and expected failures"
)]

use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    CanadianBond, CanadianCouponSchedule, CanadianFrequency, CanadianPrice, CanadianPriceBasis,
    CanadianPricingPlan, CanadianSourcePolicy, CanadianYieldConvention, CanadianYieldError,
    CanadianYieldSelection,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::math::{
    EvaluationCounts, RootBracket, SolverError, SolverOptions, TerminationReason,
};
use sabiquant::time::Date;
use std::error::Error;

const MIN_SUB: f64 = f64::from_bits(1);
const HUNDRED_BITS: u64 = 0x4059_0000_0000_0000;
const TARGET_BITS: u64 = 0x4058_f999_9999_999a; // represented99.9
const LOWER_RESIDUAL_BITS: u64 = 0x3fb9_9999_9999_9800;
const UPPER_DIRTY_BITS: u64 = 0x4058_ee83_7f4c_f09d;
const FIRST_TRIAL_BITS: u64 = 0x3fd7_6c8b_4395_7ffe;
const FIRST_TRIAL_DIRTY_BITS: u64 = 0x4058_f996_c1c8_f27a;
const FIRST_TRIAL_RESIDUAL_BITS: u64 = 0xbf26_be85_3900_0000;

fn date(month: u32, day: u32) -> Date {
    Date::from_ymd(2025, month, day).expect("valid civil date")
}

fn one_day_zero() -> CanadianBond {
    let start = date(1, 1);
    let maturity = date(7, 1);
    let schedule = CanadianCouponSchedule::new(
        start,
        maturity,
        CanadianFrequency::Semiannual,
        vec![start, maturity],
        vec![maturity],
        EndOfMonth::Ignore,
        2,
    )
    .expect("one regular period");
    CanadianBond::new(
        start,
        DecimalAmount::new(0, 0).expect("exact zero coupon"),
        schedule,
        maturity,
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )
    .expect("valid zero-coupon instrument")
}

fn prepare(bond: &CanadianBond) -> CanadianPricingPlan<'_> {
    let plan = CanadianPricingPlan::new(
        bond,
        date(6, 30),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::MoneyMarket),
        1,
    )
    .expect("one terminal bundle one day away");
    assert_eq!(plan.bundle_count(), 1);
    assert_eq!(plan.valuation_accrued_per_100().to_bits(), 0);
    assert_eq!(plan.settlement_accrued_per_100().to_bits(), 0);
    plan
}

fn target(bits: u64) -> CanadianPrice {
    CanadianPrice::new(f64::from_bits(bits), CanadianPriceBasis::ModelDirty)
        .expect("finite independently selected target")
}

fn assert_counts(counts: EvaluationCounts, function: u32) {
    assert_eq!(counts.function(), function);
    assert_eq!(counts.derivative(), 0);
    assert_eq!(counts.total(), u64::from(function));
}

fn assert_dirty(plan: &CanadianPricingPlan<'_>, yield_bits: u64, expected: u64) {
    let value = plan
        .evaluate(f64::from_bits(yield_bits))
        .expect("valid MM observation");
    assert_eq!(value.model_dirty_per_100().to_bits(), expected);
}

#[expect(
    clippy::panic,
    reason = "a different typed failure rejects the control"
)]
fn solver_failure(error: CanadianYieldError) -> SolverError {
    assert_eq!(error.kind(), "canadian_yield.solver");
    assert!(
        error
            .source()
            .is_some_and(|cause| cause.is::<SolverError>())
    );
    match error {
        CanadianYieldError::Solver(cause) => cause,
        other => panic!("expected a shared solver failure, got {other:?}"),
    }
}

/// FR-CAD-54/55/71: caps stop before the next callback, retaining actual best
/// observations. The first secant trial lies strictly inside(0,1/2), and its
/// independently rounded dirty-target residual is negative and nonzero.
#[test]
fn caps_one_two_three_exhaust_before_the_next_financial_callback() {
    let bond = one_day_zero();
    let plan = prepare(&bond);
    assert_dirty(&plan, 0, HUNDRED_BITS);
    assert_dirty(&plan, 1.0_f64.to_bits(), UPPER_DIRTY_BITS);
    assert_dirty(&plan, FIRST_TRIAL_BITS, FIRST_TRIAL_DIRTY_BITS);
    let initial = RootBracket::new(0.0, 1.0).expect("ordered finite bracket");
    for (cap, upper_bits, best_bits) in [
        (1, 1.0_f64.to_bits(), (0, LOWER_RESIDUAL_BITS)),
        (2, 1.0_f64.to_bits(), (0, LOWER_RESIDUAL_BITS)),
        (
            3,
            FIRST_TRIAL_BITS,
            (FIRST_TRIAL_BITS, FIRST_TRIAL_RESIDUAL_BITS),
        ),
    ] {
        let error = solver_failure(
            plan.yield_from_price(
                target(TARGET_BITS),
                initial,
                SolverOptions::new(MIN_SUB, 0.0, cap).expect("explicit strict tolerance and cap"),
            )
            .expect_err("no queried residual is zero and tolerance remains false"),
        );
        assert_eq!(error.kind(), "solver.evaluation_limit");
        if let SolverError::EvaluationLimit {
            bracket,
            evaluations,
            best,
            ..
        } = error
        {
            assert_counts(evaluations, cap);
            assert_eq!(bracket.lower().to_bits(), 0);
            assert_eq!(bracket.upper().to_bits(), upper_bits);
            assert_eq!(
                best.map(|(p, r)| (p.to_bits(), r.to_bits())),
                Some(best_bits)
            );
        } else {
            panic!("expected EvaluationLimit, got {error:?}");
        }
    }
}

/// FR-CAD-55/58: full-width tolerance succeeds after exactly two endpoints,
/// before a third callback. The lower endpoint has the smaller residual, so
/// its earlier dirty payload must be returned even though upper was last.
#[test]
fn bracket_tolerance_returns_the_earlier_cached_lower_payload() {
    let bond = one_day_zero();
    let plan = prepare(&bond);
    let bracket = RootBracket::new(0.0, 1.0).expect("ordered finite bracket");
    let options = SolverOptions::new(2.0, 0.0, 2).expect("full-width tolerance and two calls");
    let requested = target(TARGET_BITS);
    let report = plan
        .yield_from_price(requested, bracket, options)
        .expect("two finite opposite-sign endpoint residuals");
    let solve = report.solve_report();
    assert_eq!(solve.reason(), TerminationReason::BracketTolerance);
    assert_eq!(solve.iterations(), 0);
    assert_eq!(solve.bracket(), bracket);
    assert_counts(solve.evaluations(), 2);
    assert_eq!(solve.point().to_bits(), 0);
    assert_eq!(solve.residual().to_bits(), LOWER_RESIDUAL_BITS);
    assert_eq!(report.yield_rate().to_bits(), 0);
    assert_eq!(report.model_dirty_per_100().to_bits(), HUNDRED_BITS);
    assert_eq!(report.model_dirty_price().value().to_bits(), HUNDRED_BITS);
    assert_eq!(
        report
            .clean_price()
            .expect("zero accrued")
            .value()
            .to_bits(),
        HUNDRED_BITS
    );
    assert_eq!(
        report
            .settlement_dirty_price()
            .expect("zero accrued")
            .value()
            .to_bits(),
        HUNDRED_BITS
    );
    assert_eq!(
        report
            .amount_residual()
            .expect("cached difference")
            .to_bits(),
        LOWER_RESIDUAL_BITS
    );
    assert_eq!(report.target(), requested);
    assert_eq!(report.model_dirty_target_per_100().to_bits(), TARGET_BITS);
    assert_eq!(report.residual_scale_exponent(), 0);
    assert_eq!(report.initial_bracket(), bracket);
    assert_eq!(report.options(), options);
    assert_counts(report.solve_report().evaluations(), 2);
}

/// FR-CAD-58: both exact factors at y=±MIN_SUB round to1 and both PVs to100.
/// Target=next_up(100) leaves equal negative residuals after two callbacks.
/// NotBracketed describes only these rounded endpoint observations.
#[test]
fn flat_observed_image_preserves_counted_same_sign_failure() {
    let bond = one_day_zero();
    let plan = prepare(&bond);
    for yield_bits in [(1_u64 << 63) | 1, 1] {
        assert_dirty(&plan, yield_bits, HUNDRED_BITS);
    }
    let bracket = RootBracket::new(-MIN_SUB, MIN_SUB).expect("distinct subnormal quotes");
    let error = solver_failure(
        plan.yield_from_price(
            target(HUNDRED_BITS + 1),
            bracket,
            SolverOptions::new(MIN_SUB, 0.0, 2).expect("no implicit tolerance floor"),
        )
        .expect_err("equal negative endpoint observations do not form a sign bracket"),
    );
    assert_eq!(error.kind(), "solver.not_bracketed");
    if let SolverError::NotBracketed {
        bracket: actual,
        lower_value,
        upper_value,
        evaluations,
        ..
    } = error
    {
        assert_eq!(actual, bracket);
        assert_eq!(lower_value.to_bits(), 0xbd10_0000_0000_0000); // -2^-46
        assert_eq!(upper_value.to_bits(), lower_value.to_bits());
        assert_counts(evaluations, 2);
    } else {
        panic!("expected NotBracketed, got {error:?}");
    }
}

/// FR-CAD-55/58: adjacent quotes around-300 have a four-ULP dirty gap.
/// The independently chosen target is one dirty ULP above the upper price,
/// giving residuals+3ULPs and-1ULP. Full-width MIN_SUB tolerance fails, but no
/// represented interior quote exists: this is genuine PrecisionExhausted.
#[test]
fn adjacent_valid_mm_quotes_preserve_genuine_precision_exhaustion() {
    const LOWER_YIELD: u64 = 0xc072_c000_0000_0000;
    const UPPER_YIELD: u64 = LOWER_YIELD - 1;
    const LOWER_DIRTY: u64 = 0x4081_8c4e_c4ec_4ec5;
    const UPPER_DIRTY: u64 = 0x4081_8c4e_c4ec_4ec1;
    let bond = one_day_zero();
    let plan = prepare(&bond);
    assert_dirty(&plan, LOWER_YIELD, LOWER_DIRTY);
    assert_dirty(&plan, UPPER_YIELD, UPPER_DIRTY);
    assert_eq!(LOWER_DIRTY - UPPER_DIRTY, 4);
    let bracket = RootBracket::new(f64::from_bits(LOWER_YIELD), f64::from_bits(UPPER_YIELD))
        .expect("adjacent finite valid MM quotes");
    let error = solver_failure(
        plan.yield_from_price(
            target(UPPER_DIRTY + 1),
            bracket,
            SolverOptions::new(MIN_SUB, 0.0, 16).expect("cap cannot hide precision exhaustion"),
        )
        .expect_err("no representable interior quote and no endpoint zero"),
    );
    assert_eq!(error.kind(), "solver.precision_exhausted");
    if let SolverError::PrecisionExhausted {
        bracket: actual,
        evaluations,
        best,
        ..
    } = error
    {
        assert_eq!(actual, bracket);
        assert_counts(evaluations, 2);
        assert_eq!(best.0.to_bits(), UPPER_YIELD);
        assert_eq!(best.1.to_bits(), 0xbd40_0000_0000_0000); // -2^-43
    } else {
        panic!("expected PrecisionExhausted, got {error:?}");
    }
}
