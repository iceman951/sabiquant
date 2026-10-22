//! Post-production coverage supplement: FR-UST-12/17, FR-UST-02/24.
//! Authored after inspecting coverage v2. This is not a frozen blind hand.
//! Uses public APIs and independent exact one-payment arithmetic only.

#![expect(
    clippy::expect_used,
    reason = "explicit valid supplemental test inputs"
)]

use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    BondPrice, BondPriceKind, BondYieldSensitivity, TreasuryDailyBasis, TreasuryFirstPeriod,
    TreasuryPricingPlan, TreasurySourcePolicy, TreasuryTieRule, TreasuryYieldError, UsTreasury,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::math::{RootBracket, SolverError, SolverOptions, TerminationReason};
use sabiquant::time::Date;

fn date(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid civil date")
}

fn policy() -> TreasurySourcePolicy {
    TreasurySourcePolicy::new(
        TreasuryDailyBasis::ExactRatio,
        TreasuryTieRule::NearestTiesEven,
    )
}

fn one_payment() -> UsTreasury {
    let issue = date(2024, 12, 7);
    let maturity = date(2025, 6, 7);
    UsTreasury::new(
        issue,
        issue,
        maturity,
        DecimalAmount::new(0, 0).expect("zero coupon"),
        maturity,
        vec![issue, maturity],
        EndOfMonth::Ignore,
        policy(),
        UsTreasury::MAX_QUASI_DATES,
    )
    .expect("valid regular one-payment Treasury")
}

/// FR-UST-12/17: the two cached endpoints suffice for positional termination.
/// The 182-day interval has 91 remaining days, so the one-payment discount is
/// 1/(1+y/4). At y=0 and y=1, its native dirty amounts are exactly 100 and 80.
/// Target 81 selects the cached upper endpoint with amount residual -1.
#[test]
fn positional_yield_report_preserves_negative_cached_amount_residual() {
    let treasury = one_payment();
    let settlement = date(2025, 3, 8);
    let plan = TreasuryPricingPlan::new(&treasury, settlement, 1).expect("half period");
    let target = BondPrice::new(81.0, BondPriceKind::Dirty).expect("positive dirty target");
    let bracket = RootBracket::new(0.0, 1.0).expect("finite ordered bracket");
    let options = SolverOptions::new(1.0, 0.0, 2).expect("two callbacks and full-width tolerance");
    let result = plan
        .yield_from_price(target, bracket, options)
        .expect("positional termination");
    for _ in 0..3 {
        assert_eq!(result.target(), target);
        assert_eq!(result.initial_bracket(), bracket);
        assert_eq!(result.options(), options);
        assert_eq!(result.yield_rate().to_bits(), 1.0_f64.to_bits());
        assert_eq!(result.dirty_target_per_100().to_bits(), 81.0_f64.to_bits());
        assert_eq!(result.dirty_per_100().to_bits(), 80.0_f64.to_bits());
        assert_eq!(
            result.dirty_price().value_per_100_face().to_bits(),
            80.0_f64.to_bits()
        );
        assert_eq!(
            result
                .amount_residual()
                .expect("finite exact difference")
                .to_bits(),
            (-1.0_f64).to_bits()
        );
        assert_eq!(result.raw_accrued_per_100().to_bits(), 0);
        assert_eq!(result.accrued_per_100().to_bits(), 0);
        assert_eq!(result.source_policy(), policy());
        assert_eq!(result.source_version(), treasury.source_version());
        assert_eq!(result.original_dated_date(), date(2024, 12, 7));
        assert_eq!(result.original_issue_date(), date(2024, 12, 7));
        assert_eq!(result.maturity_date(), date(2025, 6, 7));
        assert_eq!(result.first_coupon_date(), date(2025, 6, 7));
        assert_eq!(result.first_period(), TreasuryFirstPeriod::Regular);
        assert_eq!(
            result.coupon_per_100(),
            DecimalAmount::new(0, 0).expect("zero")
        );
        assert_eq!(result.end_of_month(), EndOfMonth::Ignore);
        assert_eq!(result.settlement_date(), settlement);
        assert_eq!(result.next_quasi_date(), date(2025, 6, 7));
        assert_eq!(result.residual_scale_exponent(), 0);
        assert_eq!(
            result.sensitivity(),
            BondYieldSensitivity::MathematicallyStrictlyDecreasing
        );
        let solve = result.solve_report();
        assert_eq!(solve.point().to_bits(), 1.0_f64.to_bits());
        assert_eq!(solve.residual().to_bits(), (-1.0_f64).to_bits());
        assert_eq!(solve.bracket(), bracket);
        assert_eq!(solve.evaluations().function(), 2);
        assert_eq!(solve.evaluations().derivative(), 0);
        assert_eq!(solve.evaluations().total(), 2);
        assert_eq!(solve.iterations(), 0);
        assert_eq!(solve.reason(), TerminationReason::BracketTolerance);
    }
}

/// FR-UST-12/17: exact positive residual at the lower endpoint is 2-2^-53.
/// It is the binary midpoint below 2, so ties-to-even rounds to 2 and carries
/// the significand into the next exponent. Both endpoints remain positive.
/// Upper independently: RN(RN(1/51)*100) is 0x3fff5f5f5f5f5f5f;
/// RN(that - 2^-53) is 0x3fff5f5f5f5f5f5e (odd lower significand loses the tie).
#[test]
fn same_sign_solver_error_retains_midpoint_residual_carry_and_counts() -> Result<(), String> {
    let treasury = one_payment();
    let plan = TreasuryPricingPlan::new(&treasury, date(2025, 3, 8), 1).expect("half period");
    let target = BondPrice::new(f64::from_bits(0x3ca0_0000_0000_0000), BondPriceKind::Dirty)
        .expect("positive exact 2^-53 target");
    let bracket = RootBracket::new(196.0, 200.0).expect("finite same-sign endpoint bracket");
    let options = SolverOptions::new(1.0, 0.0, 2).expect("endpoint-only budget");
    let result = plan.yield_from_price(target, bracket, options);
    match result {
        Err(TreasuryYieldError::Solver(SolverError::NotBracketed {
            bracket: retained,
            lower_value,
            upper_value,
            evaluations,
            ..
        })) => {
            assert_eq!(retained, bracket);
            assert_eq!(lower_value.to_bits(), 2.0_f64.to_bits());
            assert_eq!(upper_value.to_bits(), 0x3fff_5f5f_5f5f_5f5e);
            assert!(lower_value.is_finite() && lower_value > 0.0);
            assert!(upper_value.is_finite() && upper_value > 0.0);
            assert_eq!(evaluations.function(), 2);
            assert_eq!(evaluations.derivative(), 0);
            assert_eq!(evaluations.total(), 2);
        }
        other => {
            return Err(format!(
                "expected counted same-sign residuals, received {other:?}"
            ));
        }
    }
    Ok(())
}
