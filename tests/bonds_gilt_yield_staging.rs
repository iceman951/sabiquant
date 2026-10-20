//! FR-GILT-10/11/20: additive public controls for target-fixed residual units.
//!
//! These controls were written after production. The large targets are chosen
//! independently as adjacent binary values around 2^970. They check counted
//! solver staging and cached arithmetic; they do not certify an ideal root.

#![expect(clippy::expect_used, reason = "explicit valid test definitions")]

use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    BondPrice, BondPriceKind, ConventionalGilt, GiltPricingPlan, GiltYieldError,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::math::{RootBracket, SolverError, SolverOptions, TerminationReason};
use sabiquant::time::Date;

fn regular(periods: u32) -> ConventionalGilt {
    let issue = Date::from_ymd(2000, 1, 1).expect("issue");
    let dates: Vec<_> = (0..=periods)
        .map(|period| {
            issue
                .checked_add_months(6 * i64::from(period))
                .expect("bounded cycle")
        })
        .collect();
    ConventionalGilt::new(
        issue,
        dates[dates.len() - 1],
        DecimalAmount::new(0, 0).expect("zero coupon"),
        dates[1],
        dates.clone(),
        dates[1..].to_vec(),
        EndOfMonth::Ignore,
        ConventionalGilt::MAX_QUASI_DATES,
    )
    .expect("complete regular cycle")
}

/// Adjacent targets straddle the exact policy boundary without repricing them
/// to manufacture targets. Zero coupon bundles retain their observations.
#[test]
fn adjacent_large_targets_select_scale_once_and_keep_cached_residual_units() {
    let gilt = regular(19);
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 19).expect("plan");
    let threshold = f64::from_bits(1993_u64 << 52); // 2^970
    let bracket = RootBracket::new(-2.0 + 2_f64.powi(-50), -2.0 + 2_f64.powi(-49))
        .expect("finite positive-price bracket");
    // The yield grid here has spacing 2^-52. Explicitly admit that width;
    // the separate control below requires PrecisionExhausted below it.
    let options = SolverOptions::new(2_f64.powi(-52), 0.0, 200).expect("explicit policy");
    for (target, scale) in [
        (threshold.next_down(), 0),
        (threshold, 1),
        (threshold.next_up(), 1),
    ] {
        let target = BondPrice::new(target, BondPriceKind::Dirty).expect("finite target");
        let report = plan
            .yield_from_price(target, bracket, options)
            .expect("bounded inverse");
        let dirty_target = target.value_per_100_face();
        assert_eq!(report.target(), target);
        assert_eq!(
            report.dirty_target_per_100().to_bits(),
            dirty_target.to_bits()
        );
        assert_eq!(report.residual_scale_exponent(), scale);
        assert_eq!(report.initial_bracket(), bracket);
        assert_eq!(report.options(), options);
        let solved = report.solve_report();
        assert_eq!(solved.reason(), TerminationReason::BracketTolerance);
        assert!((2..=200).contains(&solved.evaluations().function()));
        assert_eq!(solved.evaluations().derivative(), 0);
        assert_eq!(report.yield_rate(), solved.point());
        assert!(bracket.lower() <= solved.bracket().lower());
        assert!(solved.bracket().upper() <= bracket.upper());
        assert!(solved.bracket().lower() <= solved.point());
        assert!(solved.point() <= solved.bracket().upper());
        let dirty = report.dirty_per_100();
        // These explicit inequalities establish Sterbenz exact subtraction.
        // Halving the huge finite difference is exact too: no double rounding
        // or subnormal arithmetic is hidden in this independent identity.
        assert!(dirty_target / 2.0 <= dirty && dirty <= dirty_target * 2.0);
        let difference = dirty - dirty_target;
        assert_eq!(
            report.amount_residual().expect("amount residual").to_bits(),
            difference.to_bits()
        );
        let scaled = if scale == 0 {
            difference
        } else {
            difference / 2.0
        };
        assert_eq!(solved.residual().to_bits(), scaled.to_bits());
        assert_eq!(
            report.dirty_price().value_per_100_face().to_bits(),
            dirty.to_bits()
        );
    }
}

/// FR-GILT-11/20: the wrapper preserves an honestly exhausted coordinate
/// grid, including scale1 residuals, instead of installing an epsilon floor.
#[test]
fn unattainable_coordinate_tolerance_preserves_precision_failure_and_scaled_best() {
    let gilt = regular(19);
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 19).expect("plan");
    let target = f64::from_bits(1993_u64 << 52);
    let input = RootBracket::new(-2.0 + 2_f64.powi(-50), -2.0 + 2_f64.powi(-49))
        .expect("finite positive-price bracket");
    let error = plan
        .yield_from_price(
            BondPrice::new(target, BondPriceKind::Dirty).expect("finite target"),
            input,
            SolverOptions::new(f64::from_bits(1), 0.0, 200).expect("explicit tiny tolerance"),
        )
        .expect_err("no representable interior at the requested tolerance");
    match error {
        GiltYieldError::Solver(SolverError::PrecisionExhausted {
            bracket,
            evaluations,
            best,
            ..
        }) => {
            assert!(input.lower() <= bracket.lower() && bracket.upper() <= input.upper());
            assert_eq!(bracket.lower().next_up(), bracket.upper());
            assert!((2..=200).contains(&evaluations.function()));
            assert_eq!(evaluations.derivative(), 0);
            assert!(
                best.0.to_bits() == bracket.lower().to_bits()
                    || best.0.to_bits() == bracket.upper().to_bits()
            );
            // A test-only observation verifies the returned cached numeric
            // identity. It is outside the library's reported callback count.
            let dirty = plan
                .evaluate(best.0)
                .expect("accepted endpoint")
                .dirty_per_100();
            assert!(target / 2.0 <= dirty && dirty <= target * 2.0);
            assert_eq!(best.1.to_bits(), ((dirty - target) / 2.0).to_bits());
        }
        other => panic!("wrong precision outcome: {other:?}"),
    }
}

/// FR-GILT-11: a one-invocation success retains scale1 even when the cached
/// residual is zero. This is a payload/control test, not a root reference.
#[test]
fn large_endpoint_payload_uses_one_callback_without_unrequested_upper_quote() {
    let gilt = regular(19);
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 19).expect("plan");
    let lower = -2.0 + 2_f64.powi(-50);
    let observed = plan
        .evaluate(lower)
        .expect("finite lower price")
        .dirty_price();
    assert!(observed.value_per_100_face() >= f64::from_bits(1993_u64 << 52));
    assert!(plan.evaluate(f64::MAX).is_err());
    let report = plan
        .yield_from_price(
            observed,
            RootBracket::new(lower, f64::MAX).expect("explicit bracket"),
            SolverOptions::new(f64::from_bits(1), 0.0, 1).expect("one callback"),
        )
        .expect("lower callback zero");
    assert_eq!(report.yield_rate(), lower);
    assert_eq!(report.residual_scale_exponent(), 1);
    assert_eq!(report.solve_report().evaluations().function(), 1);
    assert_eq!(report.solve_report().evaluations().derivative(), 0);
    assert_eq!(
        report.solve_report().reason(),
        TerminationReason::FunctionZero
    );
    assert_eq!(report.solve_report().residual().to_bits(), 0);
    assert_eq!(
        report.amount_residual().expect("cached residual").to_bits(),
        0
    );
    assert_eq!(report.dirty_price(), observed);
}

/// FR-GILT-10/11/20: both admitted endpoints precede the not-bracketed result;
/// values are exact at y=0 and y=2 for one full zero-coupon quasi period.
#[test]
fn unbracketed_target_preserves_both_exact_callback_observations() {
    let gilt = regular(1);
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 1).expect("plan");
    let bracket = RootBracket::new(0.0, 2.0).expect("bracket");
    let error = plan
        .yield_from_price(
            BondPrice::new(120.0, BondPriceKind::Dirty).expect("target"),
            bracket,
            SolverOptions::new(1e-12, 0.0, 2).expect("two callbacks"),
        )
        .expect_err("target is above both endpoint prices");
    match error {
        GiltYieldError::Solver(SolverError::NotBracketed {
            bracket: original,
            lower_value,
            upper_value,
            evaluations,
            ..
        }) => {
            assert_eq!(original, bracket);
            assert_eq!(lower_value, -20.0);
            // exp(-ln(2)) is accepted before exact source-PV rounding. On this
            // host it is 1/2; retain a source-profile budget on other hosts.
            assert!((upper_value + 70.0).abs() < 1e-12);
            assert_eq!(evaluations.function(), 2);
            assert_eq!(evaluations.derivative(), 0);
        }
        other => panic!("wrong staging: {other:?}"),
    }
}
