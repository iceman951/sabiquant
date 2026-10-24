//! Blind preproduction public numerical hands, FR-CAD-18/21/24..27/32..50.
//!
//! Written from the adopted complete contract before Canadian production.
//! This frozen body is not executable until the Canadian API exists. Exact
//! expectations use independent ordered-IEEE-lattice rational certificates;
//! no Canadian oracle helper or expected corpus informed these assertions.

#![expect(clippy::expect_used, reason = "explicit valid hand inputs")]
#![expect(clippy::panic, reason = "reject a wrong structured error")]

use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    BondYieldSensitivity, CanadianBond, CanadianCouponSchedule, CanadianError, CanadianFrequency,
    CanadianPrice, CanadianPriceBasis, CanadianPricingPlan, CanadianRisk, CanadianSourcePolicy,
    CanadianYieldConvention, CanadianYieldError, CanadianYieldSelection, CouponEntitlement,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::error::{Error, NonFinite, ValueError};
use sabiquant::math::{EvaluationStage, RootBracket, SolverOptions, TerminationReason};
use sabiquant::time::Date;
use std::error::Error as StdError;
use std::ops::Bound::{Excluded, Unbounded};

const VERSION: &str = "CA-IIAC13-fixed-2026-10-23-local-v1";
const MIN_SUB: f64 = f64::from_bits(1);

fn date(y: i32, m: u32, d: u32) -> Date {
    Date::from_ymd(y, m, d).expect("explicit civil date")
}

fn instrument(
    coupon: i128,
    quasi: Vec<Date>,
    actual: Vec<Date>,
    frequency: CanadianFrequency,
    true_maturity: Date,
) -> CanadianBond {
    let start = quasi[0];
    let maturity = *actual.last().expect("nonempty actual dates");
    let cap = quasi.len().max(actual.len());
    let schedule = CanadianCouponSchedule::new(
        start,
        maturity,
        frequency,
        quasi,
        actual,
        EndOfMonth::Ignore,
        cap,
    )
    .expect("minimal direct anchored cycle");
    CanadianBond::new(
        start,
        DecimalAmount::new(coupon, 0).expect("nonnegative exact coupon"),
        schedule,
        true_maturity,
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    )
    .expect("complete explicitly selected instrument")
}

fn plan<'a>(
    bond: &'a CanadianBond,
    settlement: Date,
    convention: CanadianYieldConvention,
) -> CanadianPricingPlan<'a> {
    CanadianPricingPlan::new(
        bond,
        settlement,
        CanadianYieldSelection::Explicit(convention),
        bond.schedule().coupon_count(),
    )
    .expect("valid explicitly selected plan")
}

fn six(risk: &CanadianRisk) -> [u64; 6] {
    [
        risk.price_derivative(),
        risk.price_second_derivative(),
        risk.time_weighted_amount(),
        risk.modified_duration(),
        risk.macaulay_duration(),
        risk.convexity(),
    ]
    .map(|result| result.expect("independently admitted getter").to_bits())
}

fn one_day() -> CanadianBond {
    instrument(
        0,
        vec![date(2025, 1, 1), date(2025, 7, 1)],
        vec![date(2025, 7, 1)],
        CanadianFrequency::Semiannual,
        date(2025, 7, 1),
    )
}

fn two_regular(coupon: i128) -> CanadianBond {
    instrument(
        coupon,
        vec![date(2025, 3, 1), date(2025, 9, 1), date(2026, 3, 1)],
        vec![date(2025, 9, 1), date(2026, 3, 1)],
        CanadianFrequency::Semiannual,
        date(2026, 3, 1),
    )
}

/// FR-CAD-07/32/37/46..48: each convention has its own clock and all six held ratios.
#[test]
fn all_three_frequencies_at_zero_have_exact_independent_six_view_controls() {
    use CanadianFrequency::{Monthly, Quarterly, Semiannual};
    use CanadianYieldConvention::{Compound, MoneyMarket};
    let rows = [
        (
            Semiannual,
            date(2025, 9, 1),
            Compound,
            106.0,
            [
                0xc04a800000000000,
                0x404a800000000000,
                0x404a800000000000,
                0x3fe0000000000000,
                0x3fe0000000000000,
                0x3fe0000000000000,
            ],
        ),
        (
            Semiannual,
            date(2025, 9, 1),
            MoneyMarket,
            106.0,
            [
                0xc04ab7c2478a2b7c,
                0x404aeff9e1ba4c1b,
                0x404ab7c2478a2b7c,
                0x3fe021aa6ed1021b,
                0x3fe021aa6ed1021b,
                0x3fe0439bb3c29837,
            ],
        ),
        (
            Quarterly,
            date(2025, 6, 1),
            Compound,
            103.0,
            [
                0xc039c00000000000,
                0x4029c00000000000,
                0x4039c00000000000,
                0x3fd0000000000000,
                0x3fd0000000000000,
                0x3fc0000000000000,
            ],
        ),
        (
            Quarterly,
            date(2025, 6, 1),
            MoneyMarket,
            103.0,
            [
                0xc039f62e4a585f63,
                0x402a2cce954d2cf8,
                0x4039f62e4a585f63,
                0x3fd021aa6ed1021b,
                0x3fd021aa6ed1021b,
                0x3fc0439bb3c29837,
            ],
        ),
        (
            Monthly,
            date(2025, 4, 1),
            Compound,
            101.0,
            [
                0xc020d55555555555,
                0x3ff671c71c71c71c,
                0x4020d55555555555,
                0x3fb5555555555555,
                0x3fb5555555555555,
                0x3f8c71c71c71c71c,
            ],
        ),
        (
            Monthly,
            date(2025, 4, 1),
            MoneyMarket,
            101.0,
            [
                0xc02127fa63983280,
                0x3ff75046edbd4988,
                0x402127fa63983280,
                0x3fb5be123c515be2,
                0x3fb5be123c515be2,
                0x3f8d8bc1cf82e60f,
            ],
        ),
    ];
    for (frequency, maturity, convention, dirty, expected) in rows {
        let start = date(2025, 3, 1);
        let bond = instrument(
            12,
            vec![start, maturity],
            vec![maturity],
            frequency,
            maturity,
        );
        let plan = plan(&bond, start, convention);
        let value = plan.evaluate(0.0).expect("identity factors");
        let risk = plan.risk(0.0).expect("all retained views");
        assert_eq!(value.model_dirty_per_100(), dirty);
        assert_eq!(risk.model_dirty_per_100(), dirty);
        assert_eq!(six(&risk), expected);
        assert_eq!(
            value.model_dirty_price().basis(),
            CanadianPriceBasis::ModelDirty
        );
        assert_eq!(value.clean_price().expect("clean").value(), dirty);
        assert_eq!(
            value.settlement_dirty_price().expect("all in").value(),
            dirty
        );
        assert_eq!(value.source_version(), VERSION);
        assert_eq!(value.first_coupon_date(), maturity);
        assert_eq!(value.last_coupon_start_date(), start);
        if convention == Compound {
            for tiny in [-0.0, MIN_SUB, -MIN_SUB] {
                assert_eq!(
                    plan.evaluate(tiny)
                        .expect("RNE exponent is zero")
                        .model_dirty_per_100(),
                    dirty
                );
                assert_eq!(
                    six(&plan.risk(tiny).expect("delta rounds to exact frequency")),
                    expected
                );
            }
        }
    }
}

/// FR-CAD-38/47/48: MM's exact domain includes values far below the compound floor.
#[test]
fn one_day_negative_mm_yields_keep_the_observed_macaulay_below_ideal_witness() {
    let bond = one_day();
    let plan = plan(
        &bond,
        date(2025, 6, 30),
        CanadianYieldConvention::MoneyMarket,
    );
    let rows = [
        (
            -300.0,
            0x40818c4ec4ec4ec5,
            [
                0xc0214731fcf86d11,
                0x3fd10325675b010d,
                0x3ff89d89d89d89d9,
                0x3f8f81f81f81f820,
                0x3f66719f3601671a,
                0x3f3f05e09d0dc11c,
            ],
        ),
        (
            -280.0,
            0x407ad69696969696,
            [
                0xc01435261707f8e9,
                0x3fbe6e2749d5cb29,
                0x3ff2d2d2d2d2d2d2,
                0x3f88181818181818,
                0x3f66719f36016719,
                0x3f322436485a6c7e,
            ],
        ),
    ];
    for (y, dirty, expected) in rows {
        let risk = plan.risk(y).expect("positive exact MM denominator");
        assert_eq!(risk.model_dirty_per_100().to_bits(), dirty);
        assert_eq!(
            plan.evaluate(y)
                .expect("MM price")
                .model_dirty_per_100()
                .to_bits(),
            dirty
        );
        assert_eq!(six(&risk), expected);
    }
    let below = plan
        .risk(-280.0)
        .expect("reviewed observed lower-bound correction");
    assert!(below.macaulay_duration().expect("normal") < 1.0 / 365.0);
    assert!(below.macaulay_duration().expect("normal") > 1.0 / 730.0);
}

fn underflow(error: CanadianError, phase: &'static str, negative: bool) {
    match error {
        CanadianError::Value {
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
            assert_eq!(actual, phase);
            assert_eq!(quantity, phase);
            assert_eq!(value.to_bits(), if negative { 1_u64 << 63 } else { 0 });
            assert_eq!(
                (domain.lower(), domain.upper()),
                if negative {
                    (Unbounded, Excluded(0.0))
                } else {
                    (Excluded(0.0), Unbounded)
                }
            );
        }
        other => panic!("wrong independent getter error: {other:?}"),
    }
}

/// FR-CAD-46..49: failed raw getters never gate a valid held normalized quantity.
#[test]
fn mm_max_yield_retains_normal_moment_and_subnormal_modified_after_raw_underflows() {
    let bond = one_day();
    let plan = plan(
        &bond,
        date(2025, 6, 30),
        CanadianYieldConvention::MoneyMarket,
    );
    let risk = plan
        .risk(f64::MAX)
        .expect("no MM factor/PV overflow or zero");
    assert_eq!(risk.model_dirty_per_100().to_bits(), 0x00e1d28000000001);
    underflow(
        risk.price_derivative().expect_err("negative raw underflow"),
        "price derivative",
        true,
    );
    underflow(
        risk.price_second_derivative()
            .expect_err("positive raw underflow"),
        "price second derivative",
        false,
    );
    underflow(
        risk.convexity().expect_err("positive normalized underflow"),
        "convexity",
        false,
    );
    assert_eq!(
        risk.time_weighted_amount()
            .expect("normal moment")
            .to_bits(),
        0x0059000000000001
    );
    assert_eq!(
        risk.modified_duration()
            .expect("nonzero subnormal allowed")
            .to_bits(),
        0x0004000000000000
    );
    assert_eq!(
        risk.macaulay_duration()
            .expect("normal time ratio")
            .to_bits(),
        0x3f66719f3601671a
    );
    // Requery valid independent getters after failures and after owning clone.
    assert_eq!(
        risk.clone()
            .modified_duration()
            .expect("retained clone")
            .to_bits(),
        0x0004000000000000
    );
}

/// FR-CAD-25..27/43/47/50/54: exact bridges and cached lower zero in all three bases.
#[test]
fn midcell_source_difference_and_typed_lower_roots_use_one_shot_bridges() {
    use CanadianPriceBasis::{Clean, ModelDirty, SettlementDirty};
    use CanadianYieldConvention::{Compound, MoneyMarket};
    let bond = two_regular(12);
    let rows = [
        (
            Compound,
            0x405b400000000000,
            0x405c0193fd31cc19,
            [
                0xc054400000000000,
                0x4059200000000000,
                0x4054400000000000,
                0x3fe7249249249249,
                0x3fe7249249249249,
                0x3fecb6db6db6db6e,
            ],
        ),
        (
            MoneyMarket,
            0x405b3e6c02ce33e7,
            0x405c000000000000,
            [
                0xc05432d96cb65b2e,
                0x405e37061ec3a258,
                0x405432d96cb65b2e,
                0x3fe7158ac562b159,
                0x3fe7158ac562b159,
                0x3ff144037f4b3832,
            ],
        ),
    ];
    for (convention, clean, settlement, expected) in rows {
        let plan = plan(&bond, date(2025, 6, 1), convention);
        assert_eq!(plan.valuation_accrued_per_100(), 3.0);
        assert_eq!(
            plan.settlement_accrued_per_100().to_bits(),
            0x4008327fa6398328
        );
        let risk = plan.risk(0.0).expect("full six held sums");
        let value = plan.evaluate(0.0).expect("identity factors");
        assert_eq!(six(&risk), expected);
        assert_eq!(value.clean_price().expect("clean").value().to_bits(), clean);
        assert_eq!(
            value
                .settlement_dirty_price()
                .expect("all in")
                .value()
                .to_bits(),
            settlement
        );
        for (basis, bits) in [
            (ModelDirty, 112.0_f64.to_bits()),
            (Clean, clean),
            (SettlementDirty, settlement),
        ] {
            let target =
                CanadianPrice::new(f64::from_bits(bits), basis).expect("finite typed target");
            let bracket = RootBracket::new(0.0, f64::MAX)
                .expect("invalid unqueried compound upper allowed as solver input");
            let options = SolverOptions::new(1e-12, 0.0, 1).expect("one lower callback budget");
            let report = plan
                .yield_from_price(target, bracket, options)
                .expect("cached lower FunctionZero");
            assert_eq!(report.target(), target);
            assert_eq!(report.initial_bracket(), bracket);
            assert_eq!(report.options(), options);
            assert_eq!(report.model_dirty_target_per_100(), 112.0);
            assert_eq!(report.model_dirty_per_100(), 112.0);
            assert_eq!(report.yield_rate(), 0.0);
            assert_eq!(report.amount_residual().expect("cached amount"), 0.0);
            assert_eq!(report.residual_scale_exponent(), 0);
            assert_eq!(
                report.sensitivity(),
                BondYieldSensitivity::MathematicallyStrictlyDecreasing
            );
            let solve = report.solve_report();
            assert_eq!(solve.reason(), TerminationReason::FunctionZero);
            assert_eq!(solve.evaluations().function(), 1);
            assert_eq!(solve.evaluations().derivative(), 0);
            assert_eq!(solve.iterations(), 0);
            assert_eq!(solve.point(), 0.0);
            assert_eq!(solve.residual(), 0.0);
            assert_eq!(
                report
                    .clean_price()
                    .expect("cached clean")
                    .value()
                    .to_bits(),
                clean
            );
            assert_eq!(
                report
                    .settlement_dirty_price()
                    .expect("cached all in")
                    .value()
                    .to_bits(),
                settlement
            );
        }
    }
}

/// FR-CAD-26/50/54: observed upper zero below the ideal MM image is protocol only.
#[test]
fn mm_observed_image_floor_never_rejects_the_cached_upper_zero() {
    let bond = two_regular(1);
    let plan = plan(
        &bond,
        date(2025, 6, 1),
        CanadianYieldConvention::MoneyMarket,
    );
    let target = CanadianPrice::new(
        f64::from_bits(0x3fd5375375375375),
        CanadianPriceBasis::ModelDirty,
    )
    .expect("observed target");
    let bracket = RootBracket::new(0.0, f64::MAX).expect("finite ordered endpoints");
    let options = SolverOptions::new(1e-12, 0.0, 2).expect("only two endpoint callbacks");
    let report = plan
        .yield_from_price(target, bracket, options)
        .expect("observed target need not have ideal root");
    assert_eq!(report.target(), target);
    assert_eq!(
        report.model_dirty_target_per_100().to_bits(),
        0x3fd5375375375375
    );
    assert_eq!(report.model_dirty_per_100().to_bits(), 0x3fd5375375375375);
    assert_eq!(
        report.solve_report().reason(),
        TerminationReason::FunctionZero
    );
    assert_eq!(report.solve_report().point(), f64::MAX);
    assert_eq!(report.solve_report().evaluations().function(), 2);
    assert_eq!(report.solve_report().evaluations().derivative(), 0);
    assert_eq!(report.solve_report().iterations(), 0);
    assert_eq!(report.amount_residual().expect("unscaled cached zero"), 0.0);
    assert_eq!(report.source_version(), VERSION);
}

/// FR-CAD-18/21/24: caller true maturity changes MM horizons, not compound geometry.
#[test]
fn ceased_original_ordinals_and_explicit_last_coupon_strategy_remain_distinct() {
    let bond = two_regular(12);
    let settlement = date(2025, 9, 1);
    let selected = CanadianPricingPlan::new(
        &bond,
        settlement,
        CanadianYieldSelection::LastCouponMoneyMarket,
        1,
    )
    .expect("one remaining actual coupon");
    assert_eq!(selected.bundle_count(), 1);
    assert_eq!(selected.bundle_coupon_index(0), Some(1));
    assert_eq!(selected.bundle_coupon_index(1), None);
    assert_eq!(
        selected.coupon_entitlement(0),
        Some(CouponEntitlement::Ceased)
    );
    assert_eq!(
        selected.coupon_entitlement(1),
        Some(CouponEntitlement::BuyerEntitled)
    );
    assert_eq!(selected.coupon_entitlement(2), None);
    assert_eq!(
        selected.yield_convention(),
        CanadianYieldConvention::MoneyMarket
    );
    assert_eq!(
        selected
            .evaluate(0.0)
            .expect("terminal coupon once")
            .model_dirty_per_100(),
        106.0
    );
    assert_eq!(selected.valuation_accrued_per_100(), 0.0);
    assert_eq!(selected.settlement_accrued_per_100(), 0.0);
    let before = CanadianPricingPlan::new(
        &bond,
        date(2025, 6, 1),
        CanadianYieldSelection::LastCouponMoneyMarket,
        2,
    )
    .expect("two remaining actual coupons");
    assert_eq!(before.yield_convention(), CanadianYieldConvention::Compound);
    let delayed = instrument(
        0,
        vec![date(2025, 1, 1), date(2025, 7, 1)],
        vec![date(2025, 7, 1)],
        CanadianFrequency::Semiannual,
        date(2025, 7, 2),
    );
    let p = plan(
        &delayed,
        date(2025, 6, 30),
        CanadianYieldConvention::MoneyMarket,
    );
    assert_eq!(
        p.evaluate(0.0)
            .expect("no added delay interest")
            .model_dirty_per_100(),
        100.0
    );
    assert_eq!(
        p.risk(0.0)
            .expect("two day moment")
            .macaulay_duration()
            .expect("normal")
            .to_bits(),
        0x3f76719f3601671a
    );
}

/// FR-CAD-26/43/50/53: signed/zero external targets are resolved before positivity.
#[test]
fn signed_targets_reach_the_actual_lower_callback_when_resolved_dirty_is_positive() {
    let bond = instrument(
        12,
        vec![date(2024, 9, 1), date(2025, 3, 1)],
        vec![date(2025, 3, 1)],
        CanadianFrequency::Semiannual,
        date(2025, 3, 1),
    );
    let plan = plan(&bond, date(2024, 9, 2), CanadianYieldConvention::Compound);
    assert_eq!(
        plan.valuation_accrued_per_100().to_bits(),
        0x3fa0f8ed9cfe95ec
    );
    assert_eq!(
        plan.settlement_accrued_per_100().to_bits(),
        0x3fa0d53768810d53
    );
    for (basis, external) in [
        (CanadianPriceBasis::Clean, 0.0),
        (CanadianPriceBasis::Clean, -MIN_SUB),
        (CanadianPriceBasis::SettlementDirty, 0.0),
    ] {
        let target = CanadianPrice::new(external, basis).expect("finite signed input");
        let error = plan
            .yield_from_price(
                target,
                RootBracket::new(-2.0, 0.0).expect("invalid invoked compound lower"),
                SolverOptions::new(1e-12, 0.0, 1).expect("one failed invocation"),
            )
            .expect_err("domain failure at actual lower callback");
        match error {
            CanadianYieldError::Callback {
                stage: EvaluationStage::LowerEndpoint,
                point,
                evaluations,
                cause:
                    CanadianError::Value {
                        index: None,
                        phase: "yield quotation",
                        cause:
                            ValueError::OutOfDomain {
                                parameter: "rate",
                                value,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                assert_eq!(point, -2.0);
                assert_eq!(value, -2.0);
                assert_eq!(evaluations.function(), 1);
                assert_eq!(evaluations.derivative(), 0);
            }
            other => panic!("wrong resolved-target order or callback transport: {other:?}"),
        }
    }
    let zero =
        CanadianPrice::new(0.0, CanadianPriceBasis::ModelDirty).expect("typed zero admitted");
    let error = plan
        .yield_from_price(
            zero,
            RootBracket::new(-2.0, 0.0).expect("bracket"),
            SolverOptions::new(1e-12, 0.0, 1).expect("budget"),
        )
        .expect_err("resolved dirty zero");
    match error {
        CanadianYieldError::NonPositiveTarget {
            target,
            model_dirty_target_per_100,
            evaluations,
            ..
        } => {
            assert_eq!(target, zero);
            assert_eq!(model_dirty_target_per_100, 0.0);
            assert_eq!(evaluations.function(), 0);
            assert_eq!(evaluations.derivative(), 0);
        }
        other => panic!("wrong zero target admission: {other:?}"),
    }
}

/// FR-CAD-25/32/38/71: finite admission precedes model work, with immediate causes.
#[test]
fn finite_signed_prices_and_nonfinite_quotes_keep_all_error_layers() {
    for basis in CanadianPriceBasis::ALL {
        assert_eq!(
            CanadianPrice::new(-0.0, basis)
                .expect("canonical zero")
                .value()
                .to_bits(),
            0
        );
        assert_eq!(
            CanadianPrice::new(-MIN_SUB, basis)
                .expect("signed finite price")
                .value()
                .to_bits(),
            (1_u64 << 63) | 1
        );
        for (bad, class) in [
            (f64::NAN, NonFinite::NaN),
            (f64::INFINITY, NonFinite::PositiveInfinity),
            (f64::NEG_INFINITY, NonFinite::NegativeInfinity),
        ] {
            let error = CanadianPrice::new(bad, basis).expect_err("finite price admission");
            match &error {
                CanadianError::Value {
                    index: None,
                    phase: "price quotation",
                    cause:
                        ValueError::NotFinite {
                            parameter: "canadian price",
                            value,
                            ..
                        },
                    ..
                } => {
                    assert_eq!(*value, class);
                }
                other => panic!("wrong price error: {other:?}"),
            }
            let wrapped = Error::from(error);
            let domain = wrapped
                .source()
                .expect("root wraps CanadianError immediately");
            assert!(domain.is::<CanadianError>());
            assert!(
                domain
                    .source()
                    .expect("domain wraps ValueError immediately")
                    .is::<ValueError>()
            );
        }
    }
    let bond = one_day();
    for convention in CanadianYieldConvention::ALL {
        let p = plan(&bond, date(2025, 6, 30), convention);
        for (bad, class) in [
            (f64::NAN, NonFinite::NaN),
            (f64::INFINITY, NonFinite::PositiveInfinity),
            (f64::NEG_INFINITY, NonFinite::NegativeInfinity),
        ] {
            for error in [
                p.evaluate(bad).expect_err("nonfinite quote"),
                p.risk(bad).expect_err("same quote admission"),
            ] {
                match error {
                    CanadianError::Value {
                        index: None,
                        phase: "yield quotation",
                        cause:
                            ValueError::NotFinite {
                                parameter: "rate",
                                value,
                                ..
                            },
                        ..
                    } => {
                        assert_eq!(value, class);
                    }
                    other => panic!("wrong quote admission: {other:?}"),
                }
            }
        }
    }
    let p = plan(
        &bond,
        date(2025, 6, 30),
        CanadianYieldConvention::MoneyMarket,
    );
    for y in [-365.0, f64::from_bits((-365.0_f64).to_bits() + 1)] {
        match p
            .evaluate(y)
            .expect_err("exact affine denominator nonpositive")
        {
            CanadianError::MoneyMarketDomain {
                yield_rate,
                days_to_true_maturity: 1,
                ..
            } => assert_eq!(yield_rate.to_bits(), y.to_bits()),
            other => panic!("wrong exact MM domain: {other:?}"),
        }
    }
    let above = f64::from_bits((-365.0_f64).to_bits() - 1);
    assert!(
        p.evaluate(above)
            .expect("strict positive neighboring affine")
            .model_dirty_per_100()
            .is_finite()
    );
}
