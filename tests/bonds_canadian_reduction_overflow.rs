//! Postproduction public controls for FR-CAD-41/55/56/71.
//!
//! Independent exact arithmetic, before Rust execution: 1,016 regular
//! semiannual periods, annual coupon202 and y=-1 give ideal factors2^j.
//! Sources are101 for the first1,015 bundles and201 at maturity. Thus the
//! largest ideal PV is201*2^1016, while the total is302*2^1016-202.
//! A deliberately loose1% PV error allowance, derived from the adopted
//! conditional f2 primitive profile, keeps every PV below0.794*MAX and the
//! total above1.167 times the IEEE overflow midpoint. No native factor bits
//! supply expectations. These are derived controls, not frozen blind hands.

#![expect(
    clippy::expect_used,
    reason = "explicit valid postproduction inputs and prescribed failures"
)]

use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    CanadianBond, CanadianCouponSchedule, CanadianError, CanadianFrequency, CanadianPrice,
    CanadianPriceBasis, CanadianPricingPlan, CanadianSourcePolicy, CanadianYieldConvention,
    CanadianYieldError, CanadianYieldSelection,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::error::{NonFinite, ValueError};
use sabiquant::math::{EvaluationStage, RootBracket, SolverOptions, SumError};
use sabiquant::time::Date;
use std::error::Error;

const PERIODS: usize = 1016;

fn overflowing_total_bond() -> CanadianBond {
    let start = Date::from_ymd(1500, 1, 1).expect("valid civil date");
    let quasi: Vec<_> = (0..=PERIODS)
        .map(|index| {
            start
                .checked_add_months(6 * i64::try_from(index).expect("bounded period index"))
                .expect("direct semiannual cycle stays in the civil range")
        })
        .collect();
    let maturity = quasi[PERIODS];
    assert_eq!(
        maturity,
        Date::from_ymd(2008, 1, 1).expect("valid maturity")
    );
    let coupons = quasi[1..].to_vec();
    let schedule = CanadianCouponSchedule::new(
        start,
        maturity,
        CanadianFrequency::Semiannual,
        quasi,
        coupons,
        EndOfMonth::Ignore,
        PERIODS + 1,
    )
    .expect("all 1016 actual periods are regular");
    CanadianBond::new(
        start,
        DecimalAmount::new(202, 0).expect("exact annual coupon202"),
        schedule,
        maturity,
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )
    .expect("valid fixed-coupon instrument")
}

fn prepare(bond: &CanadianBond) -> CanadianPricingPlan<'_> {
    let plan = CanadianPricingPlan::new(
        bond,
        bond.schedule().accrual_start(),
        CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        PERIODS,
    )
    .expect("all 1016 bundles are future payments");
    assert_eq!(plan.bundle_count(), PERIODS);
    assert_eq!(plan.bundle_coupon_index(0), Some(0));
    assert_eq!(plan.bundle_coupon_index(PERIODS - 1), Some(PERIODS - 1));
    plan
}

#[expect(
    clippy::panic,
    reason = "a different structured cause rejects the control"
)]
fn assert_dirty_reduction_overflow(error: &CanadianError) {
    assert_eq!(error.kind(), "canadian.sum");
    let CanadianError::Sum { phase, cause, .. } = error else {
        panic!("expected dirty reduction Sum, got {error:?}");
    };
    assert_eq!(*phase, "dirty price reduction");
    assert!(std::ptr::eq(
        error
            .source()
            .expect("immediate SumError source")
            .downcast_ref::<SumError>()
            .expect("typed SumError"),
        cause
    ));
    assert_eq!(cause.kind(), "sum.arithmetic");
    let SumError::Arithmetic {
        index,
        error: value,
        ..
    } = cause
    else {
        panic!("expected final exact arithmetic failure, got {cause:?}");
    };
    assert_eq!(*index, None);
    assert!(std::ptr::eq(
        cause
            .source()
            .expect("immediate ValueError source")
            .downcast_ref::<ValueError>()
            .expect("typed ValueError"),
        value
    ));
    assert!(matches!(
        value,
        ValueError::ResultNotFinite {
            quantity: "exact sum",
            value: NonFinite::PositiveInfinity,
            ..
        }
    ));
    assert_eq!(value.kind(), "value.result_not_finite");
    assert!(value.source().is_none());
}

/// FR-CAD-41/71: every individual factor/PV is finite under the conditional
/// profile, so evaluate and risk must reach final exact dirty reduction.
/// Risk retains extended terms without imposing raw-getter IEEE limits.
#[test]
fn evaluate_and_risk_preserve_final_dirty_reduction_overflow() {
    let bond = overflowing_total_bond();
    let plan = prepare(&bond);
    // At y0 every factor is the identity:101*1016+100=102716 exactly.
    assert_eq!(
        plan.evaluate(0.0)
            .expect("finite identity-factor control")
            .model_dirty_per_100()
            .to_bits(),
        102_716.0_f64.to_bits()
    );
    let evaluated = plan
        .evaluate(-1.0)
        .expect_err("finite PVs overflow only when reduced together");
    assert_dirty_reduction_overflow(&evaluated);
    let risk = plan
        .risk(-1.0)
        .expect_err("retained terms are admitted before the same dirty reduction");
    assert_dirty_reduction_overflow(&risk);
    assert_eq!(risk, evaluated);
}

/// FR-CAD-55/56/71: actual lower financial invocation fails inside run, after
/// valid quote admission. Its count and immediate Sum chain survive transport;
/// the valid upper endpoint is not queried and no image precheck substitutes.
#[test]
fn yield_lower_callback_preserves_the_actual_dirty_sum_failure() {
    let bond = overflowing_total_bond();
    let plan = prepare(&bond);
    let failure = plan
        .yield_from_price(
            CanadianPrice::new(100.0, CanadianPriceBasis::ModelDirty)
                .expect("positive model-dirty target"),
            RootBracket::new(-1.0, 0.0).expect("valid ordered compound quotes"),
            SolverOptions::new(1e-12, 0.0, 8).expect("valid explicit callback budget"),
        )
        .expect_err("first invoked dirty reduction overflows");
    assert_eq!(failure.kind(), "canadian_yield.callback");
    if let CanadianYieldError::Callback {
        stage,
        point,
        evaluations,
        cause,
        ..
    } = &failure
    {
        assert_eq!(*stage, EvaluationStage::LowerEndpoint);
        assert_eq!(point.to_bits(), (-1.0_f64).to_bits());
        assert_eq!(evaluations.function(), 1);
        assert_eq!(evaluations.derivative(), 0);
        assert_eq!(evaluations.total(), 1);
        assert!(std::ptr::eq(
            failure
                .source()
                .expect("immediate CanadianError source")
                .downcast_ref::<CanadianError>()
                .expect("typed CanadianError"),
            cause
        ));
        assert_dirty_reduction_overflow(cause);
    } else {
        panic!("expected counted lower callback failure, got {failure:?}");
    }
}
