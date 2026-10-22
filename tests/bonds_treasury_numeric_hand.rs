//! Blind pre-production Treasury public numerical controls, FR-UST-05..17/23/25.
//!
//! Authored against the complete adopted contract, without Treasury production
//! or other Treasury expected corpora. This file is a future integration-test
//! body copied for execution. This integration copy compares public Interval
//! bounds because its convenience constructors are crate-private; all financial
//! expectations and exact domain endpoints preserve the frozen original.
//! See docs/M8_US_TREASURY_HAND_DESIGN.md for independent arithmetic provenance,
//! exact anchors, prescribed-observation bits and the limited ideal allowance.

#![expect(clippy::expect_used, reason = "explicit valid blind hand inputs")]
#![expect(
    clippy::panic,
    reason = "hand-test helper rejects a wrong structured error"
)]

use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    BondPrice, BondPriceKind, BondYieldSensitivity, TreasuryDailyBasis, TreasuryError,
    TreasuryFirstPeriod, TreasuryPricingPlan, TreasuryRisk, TreasurySourcePolicy, TreasuryTieRule,
    TreasuryYieldError, UsTreasury,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::error::{NonFinite, ValueError};
use sabiquant::math::{
    EvaluationStage, RootBracket, SolverError, SolverOptions, TerminationReason,
};
use sabiquant::time::Date;
use std::ops::Bound::{Excluded, Unbounded};

const ABOVE_FLOOR: f64 = f64::from_bits(0xbfff_ffff_ffff_ffff);
const MIN_SUB: f64 = f64::from_bits(1);

fn d(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid civil date")
}

fn amount(coefficient: i128, scale: u8) -> DecimalAmount {
    DecimalAmount::new(coefficient, scale).expect("valid exact coupon")
}

fn exact_even() -> TreasurySourcePolicy {
    TreasurySourcePolicy::new(
        TreasuryDailyBasis::ExactRatio,
        TreasuryTieRule::NearestTiesEven,
    )
}

fn quote(value: f64, kind: BondPriceKind) -> BondPrice {
    BondPrice::new(value, kind).expect("finite native per100 quote")
}

fn options(cap: u32) -> SolverOptions {
    SolverOptions::new(1e-11, 0.0, cap).expect("explicit bounded solver policy")
}

fn instrument(
    dated: Date,
    issue: Date,
    coupon: DecimalAmount,
    first: Date,
    quasi: Vec<Date>,
    policy: TreasurySourcePolicy,
) -> UsTreasury {
    UsTreasury::new(
        dated,
        issue,
        *quasi.last().expect("nonempty explicit complete cycle"),
        coupon,
        first,
        quasi,
        EndOfMonth::Ignore,
        policy,
        UsTreasury::MAX_QUASI_DATES,
    )
    .expect("valid directly maturity-anchored Treasury")
}

fn regular_from(issue: Date, periods: u32, coupon: DecimalAmount) -> UsTreasury {
    let maturity = issue
        .checked_add_months(6 * i64::from(periods))
        .expect("bounded maturity");
    let quasi: Vec<_> = (0..=periods)
        .map(|k| {
            maturity
                .checked_add_months(-6 * i64::from(periods - k))
                .expect("direct maturity anchoring")
        })
        .collect();
    instrument(issue, issue, coupon, quasi[1], quasi, exact_even())
}

fn regular(periods: u32, coupon: DecimalAmount) -> UsTreasury {
    regular_from(d(2024, 12, 7), periods, coupon)
}

fn long_first(issue: Date) -> UsTreasury {
    instrument(
        d(2025, 3, 8),
        issue,
        amount(6, 0),
        d(2025, 12, 7),
        vec![d(2024, 12, 7), d(2025, 6, 7), d(2025, 12, 7), d(2026, 6, 7)],
        exact_even(),
    )
}

fn six_values(risk: &TreasuryRisk) -> [f64; 6] {
    [
        risk.price_derivative().expect("admitted first derivative"),
        risk.price_second_derivative()
            .expect("admitted second derivative"),
        risk.time_weighted_amount().expect("admitted moment"),
        risk.modified_duration().expect("admitted Modified"),
        risk.macaulay_duration().expect("admitted Macaulay"),
        risk.convexity().expect("admitted Convexity"),
    ]
}

fn assert_getter_underflow(error: TreasuryError, phase: &str, negative: bool) {
    match error {
        TreasuryError::Value {
            index: None,
            phase: actual_phase,
            cause:
                ValueError::ResultOutOfDomain {
                    quantity,
                    value,
                    domain,
                    ..
                },
            ..
        } => {
            assert_eq!(actual_phase, phase);
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
        other => panic!("wrong signed getter underflow: {other:?}"),
    }
}

fn assert_pv_failure(error: TreasuryError, expected_index: usize, overflow: bool) {
    match error {
        TreasuryError::Value {
            index: Some(index),
            phase: "source present value",
            cause,
            ..
        } => {
            assert_eq!(index, expected_index);
            if overflow {
                assert!(matches!(
                    cause,
                    ValueError::ResultNotFinite {
                        quantity: "treasury present value",
                        value: NonFinite::PositiveInfinity,
                        ..
                    }
                ));
            } else {
                match cause {
                    ValueError::ResultOutOfDomain {
                        quantity: "treasury present value",
                        value,
                        domain,
                        ..
                    } => {
                        assert_eq!(value.to_bits(), 0);
                        assert_eq!((domain.lower(), domain.upper()), (Excluded(0.0), Unbounded));
                    }
                    other => panic!("wrong positive PV underflow: {other:?}"),
                }
            }
        }
        other => panic!("wrong indexed source PV error: {other:?}"),
    }
}

/// FR-UST-09/11..14: alpha1/2, q=b=1; simple-prefix D'' differs from a power stub.
#[test]
fn half_period_zero_yield_has_exact_price_clean_and_all_six_risks() {
    let treasury = regular(1, amount(5, 0));
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 1).expect("91 of182 remain");
    assert_eq!((plan.remaining_days(), plan.quasi_period_days()), (91, 182));
    assert_eq!(plan.remaining_quasi_periods(), 0);
    let value = plan.evaluate(0.0).expect("identity factor");
    let risk = plan.risk(0.0).expect("complete retained risk");
    assert_eq!(value.dirty_per_100(), 102.5);
    assert_eq!(value.raw_accrued_per_100(), 1.25);
    assert_eq!(value.accrued_per_100(), 1.25);
    assert_eq!(
        value.clean_price().expect("clean").value_per_100_face(),
        101.25
    );
    assert_eq!(
        six_values(&risk),
        [-25.625, 12.8125, 25.625, 0.25, 0.25, 0.125]
    );
    assert_eq!(risk.dirty_per_100(), value.dirty_per_100());
}

/// FR-UST-10..14: blind prescribed bits and an independently conditioned ideal control.
#[test]
fn nonzero_simple_stub_has_macaulay_quarter_and_not_b_times_modified() {
    let treasury = regular(1, amount(5, 0));
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 1).expect("half period");
    let risk = plan.risk(0.05).expect("positive simple prefix");
    assert_eq!(risk.dirty_per_100().to_bits(), 0x4059_4f03_2916_1f9a);
    assert_eq!(
        six_values(&risk).map(f64::to_bits),
        [
            0xc038_ff06_482f_2260,
            0x4028_b006_3454_7a72,
            0x4039_4f03_2916_1f9a,
            0x3fcf_9add_3c0c_a459,
            0x3fd0_0000_0000_0000,
            0x3fbf_36fa_1bb3_fdfa,
        ]
    );
    // In the exact decimal-y ideal: q=81/80, eta=20/81, kappa=800/6561.
    // Represented .05 differs by <2^-58. N3/N5 plus rational reference RN
    // and that input perturbation are <8eps here, for every displayed value.
    let price = 8200.0 / 81.0;
    let ideals = [
        price,
        -164_000.0 / 6561.0,
        6_560_000.0 / 531_441.0,
        2050.0 / 81.0,
        20.0 / 81.0,
        0.25,
        800.0 / 6561.0,
    ];
    let [d1, d2, moment, modified, macaulay, convexity] = six_values(&risk);
    for (observed, ideal) in [
        risk.dirty_per_100(),
        d1,
        d2,
        moment,
        modified,
        macaulay,
        convexity,
    ]
    .into_iter()
    .zip(ideals)
    {
        assert!((observed - ideal).abs() <= 8.0 * f64::EPSILON * ideal.abs());
    }
    assert_eq!(macaulay, 0.25);
    assert!(((1.0 + 0.05 / 2.0) * modified - 41.0 / 162.0).abs() < f64::EPSILON);
    assert!(((1.0 + 0.05 / 2.0) * modified - macaulay).abs() > 0.003);
}

/// FR-UST-11/13/14: two payments use j0,j1, not a power of the fractional prefix.
#[test]
fn two_payment_zero_yield_anchors_exact_second_derivative() {
    let treasury = regular(2, amount(6, 0));
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 2).expect("half initial period");
    let risk = plan.risk(-0.0).expect("canonical zero");
    assert_eq!(risk.yield_rate().to_bits(), 0);
    assert_eq!(risk.dirty_per_100(), 106.0);
    assert_eq!(risk.price_derivative().expect("d1"), -78.0);
    assert_eq!(risk.price_second_derivative().expect("d2"), 90.5);
    assert_eq!(risk.time_weighted_amount().expect("moment"), 78.0);
    assert_eq!(risk.modified_duration().expect("Modified"), 39.0 / 53.0);
    assert_eq!(risk.macaulay_duration().expect("Macaulay"), 39.0 / 53.0);
    assert_eq!(risk.convexity().expect("Convexity"), 181.0 / 212.0);
    assert_eq!(plan.bundle_quasi_index(0), Some(1));
    assert_eq!(plan.bundle_quasi_index(1), Some(2));
    assert_eq!(plan.bundle_quasi_index(2), None);
}

/// FR-UST-10/11/13: alpha1 makes the ordinary complete-period identity exact.
#[test]
fn exact_power_two_factor_and_zero_coupons_keep_principal_risk() {
    let treasury = regular(3, amount(0, 0));
    let plan = TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 3)
        .expect("three zero coupon bundles");
    assert_eq!(plan.bundle_count(), 3);
    let risk = plan.risk(2.0).expect("q=b=2, final factor1/8");
    assert_eq!(risk.dirty_per_100(), 12.5);
    assert_eq!(six_values(&risk), [-9.375, 9.375, 18.75, 0.75, 1.5, 0.75]);
}

/// FR-UST-05/07/11/13: short first raw coupon is1.5, with no daily publication in PV.
#[test]
fn short_first_coupon_uses_dated_fraction_in_price_and_all_six_risks() {
    let treasury = instrument(
        d(2025, 3, 8),
        d(2025, 3, 8),
        amount(6, 0),
        d(2025, 6, 7),
        vec![d(2024, 12, 7), d(2025, 6, 7), d(2025, 12, 7)],
        exact_even(),
    );
    assert_eq!(treasury.first_period(), TreasuryFirstPeriod::Short);
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 2).expect("alpha1/2");
    let risk = plan.risk(0.0).expect("raw A0=1.5, A1=103");
    assert_eq!(risk.dirty_per_100(), 104.5);
    assert_eq!(risk.raw_accrued_per_100(), 0.0);
    assert_eq!(risk.accrued_per_100(), 0.0);
    assert_eq!(risk.price_derivative().expect("d1"), -77.625);
    assert_eq!(risk.price_second_derivative().expect("d2"), 90.3125);
    assert_eq!(risk.time_weighted_amount().expect("moment"), 77.625);
    assert_eq!(risk.modified_duration().expect("Modified"), 621.0 / 836.0);
    assert_eq!(risk.macaulay_duration().expect("Macaulay"), 621.0 / 836.0);
    assert_eq!(risk.convexity().expect("Convexity"), 1445.0 / 1672.0);
}

/// FR-UST-05/08/11/13: skipped Q1 leaves actual first cash at original j1.
#[test]
fn long_first_missing_quasi_date_does_not_renumber_price_or_risk() {
    let treasury = long_first(d(2025, 3, 8));
    assert_eq!(treasury.first_period(), TreasuryFirstPeriod::Long);
    assert_eq!(treasury.first_coupon_quasi_index(), 2);
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 2).expect("D settlement");
    assert_eq!(
        (plan.next_quasi_index(), plan.remaining_quasi_periods()),
        (1, 2)
    );
    assert_eq!(plan.bundle_count(), 2);
    assert_eq!(plan.bundle_quasi_index(0), Some(2));
    assert_eq!(plan.bundle_quasi_index(1), Some(3));
    let risk = plan.risk(0.0).expect("raw A1=4.5, A2=103");
    assert_eq!(risk.dirty_per_100(), 107.5);
    assert_eq!(risk.price_derivative().expect("d1"), -132.125);
    assert_eq!(risk.price_second_derivative().expect("d2"), 222.8125);
    assert_eq!(risk.time_weighted_amount().expect("moment"), 132.125);
    assert_eq!(risk.modified_duration().expect("Modified"), 1057.0 / 860.0);
    assert_eq!(risk.macaulay_duration().expect("Macaulay"), 1057.0 / 860.0);
    assert_eq!(risk.convexity().expect("Convexity"), 713.0 / 344.0);
}

/// FR-UST-05/09/12: original dated accrual persists across skipped Q1, then resets.
#[test]
fn long_first_accrual_survives_skipped_date_and_original_issue_after_it() {
    let treasury = long_first(d(2025, 3, 8));
    for (settlement, raw, published) in [
        (d(2025, 3, 8), 0.0, 0.0),
        (d(2025, 6, 7), 1.5, 1.5),
        (d(2025, 12, 7), 0.0, 0.0),
        (d(2025, 12, 8), 3.0 / 182.0, 0.016_484),
    ] {
        let plan = TreasuryPricingPlan::new(&treasury, settlement, 2).expect("long reset control");
        assert_eq!(plan.raw_accrued_per_100(), raw);
        assert_eq!(plan.accrued_per_100(), published);
        let value = plan.evaluate(0.0).expect("zero quote");
        assert_eq!(value.raw_accrued_per_100(), raw);
        assert_eq!(value.accrued_per_100(), published);
    }
    let later_issue = long_first(d(2025, 7, 1));
    let plan = TreasuryPricingPlan::new(&later_issue, d(2025, 7, 1), 2)
        .expect("I>skipped Q1 while D remains accrual origin");
    assert_eq!(plan.raw_accrued_per_100(), 231.0 / 122.0);
    assert_eq!(plan.accrued_per_100(), 1.893_443);
    let report = plan.evaluate(0.0).expect("long issued in second interval");
    assert_eq!(report.original_dated_date(), d(2025, 3, 8));
    assert_eq!(report.original_issue_date(), d(2025, 7, 1));
    assert_eq!(report.dirty_per_100(), 107.5);
}

/// FR-UST-06/09/12/14: both tie rules affect published AI, never raw dirty/risk.
#[test]
fn policy_is_retained_while_dirty_and_all_six_risks_stay_identical() {
    let dated = d(2025, 7, 15);
    let maturity = d(2026, 1, 15);
    let mut observations = Vec::new();
    for basis in [
        TreasuryDailyBasis::ExactRatio,
        TreasuryDailyBasis::NineDecimal,
    ] {
        for tie in [
            TreasuryTieRule::NearestTiesEven,
            TreasuryTieRule::NearestTiesAwayFromZero,
        ] {
            let policy = TreasurySourcePolicy::new(basis, tie);
            let treasury = instrument(
                dated,
                dated,
                amount(23, 7),
                maturity,
                vec![dated, maturity],
                policy,
            );
            let plan =
                TreasuryPricingPlan::new(&treasury, d(2025, 10, 3), 1).expect("80 of184 elapsed");
            let published = if tie == TreasuryTieRule::NearestTiesEven {
                0.0
            } else {
                0.000_001
            };
            assert_eq!(plan.raw_accrued_per_100(), 0.000_000_5);
            assert_eq!(plan.accrued_per_100(), published);
            let value = plan.evaluate(0.0).expect("identity price");
            let risk = plan.risk(0.0).expect("policy-independent raw coefficients");
            assert_eq!(value.source_policy(), policy);
            assert_eq!(risk.source_policy(), policy);
            assert_eq!(value.raw_accrued_per_100(), 0.000_000_5);
            assert_eq!(risk.raw_accrued_per_100(), 0.000_000_5);
            assert_eq!(value.accrued_per_100(), published);
            assert_eq!(risk.accrued_per_100(), published);
            assert_eq!(
                value
                    .clean_price()
                    .expect("published clean")
                    .value_per_100_face(),
                value.dirty_per_100() - published
            );
            observations.push((
                value.dirty_per_100().to_bits(),
                six_values(&risk).map(f64::to_bits),
            ));
        }
    }
    assert!(observations.iter().all(|item| *item == observations[0]));
}

/// FR-UST-10/12/17: a flat computed observation leaves ideal sensitivity strict.
#[test]
fn signed_zero_and_adjacent_quotes_keep_input_metadata_and_flat_prices() {
    let treasury = regular(1, amount(5, 0));
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 1).expect("half period");
    for y in [0.0, -0.0, MIN_SUB, -MIN_SUB] {
        let value = plan.evaluate(y).expect("bounded tiny exact seed");
        assert_eq!(value.dirty_per_100(), 102.5);
        assert_eq!(
            value.yield_rate().to_bits(),
            if y == 0.0 { 0 } else { y.to_bits() }
        );
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
            RootBracket::new(-MIN_SUB, MIN_SUB).expect("two tiny endpoints"),
            options(1),
        )
        .expect("lower observed zero");
    assert_eq!(report.yield_rate().to_bits(), (-MIN_SUB).to_bits());
    assert_eq!(report.solve_report().evaluations().function(), 1);
    assert_eq!(
        report.sensitivity(),
        BondYieldSensitivity::MathematicallyStrictlyDecreasing
    );
}

/// FR-UST-10/23: evaluation and risk share finite-y>-2 Value admission.
#[test]
fn invalid_yield_is_a_quote_value_error_even_with_zero_coupons() {
    let treasury = regular(2, amount(0, 0));
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    for y in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -2.0, -3.0] {
        for error in [
            plan.evaluate(y).expect_err("invalid quote"),
            plan.risk(y).expect_err("invalid quote"),
        ] {
            match error {
                TreasuryError::Value {
                    index: None,
                    phase: "yield quote",
                    cause,
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
                    } if y.is_finite() => {
                        assert_eq!(value.to_bits(), y.to_bits());
                        assert_eq!(
                            (domain.lower(), domain.upper()),
                            (Excluded(-2.0), Unbounded)
                        );
                    }
                    other => panic!("wrong quote cause: {other:?}"),
                },
                other => panic!("wrong Treasury quote wrapper: {other:?}"),
            }
        }
    }
}

/// FR-UST-10/11/23: extended factors have no IEEE gate; final principal fails at Q21.
#[test]
fn zero_coupons_do_not_create_an_early_factor_range_failure() {
    let treasury = regular(21, amount(0, 0));
    let plan = TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 21)
        .expect("21 bundles");
    // q=b=2^-53; zero Q20 has an extended factor2^1060 but exact PV0.
    // The actual final source amount100 at Q21 is the first failed nonzero PV.
    for error in [
        plan.evaluate(ABOVE_FLOOR).expect_err("principal overflow"),
        plan.risk(ABOVE_FLOOR).expect_err("principal overflow"),
    ] {
        assert_pv_failure(error, 21, true);
    }
}

/// FR-UST-11/23: first source amount1/(2e38) rounds to zero before later cash.
#[test]
fn first_nonzero_tiny_coupon_underflow_preserves_original_index() {
    let treasury = regular(2, amount(1, 38));
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    for error in [
        plan.evaluate(f64::MAX).expect_err("PV underflow"),
        plan.risk(f64::MAX).expect_err("PV underflow"),
    ] {
        assert_pv_failure(error, 1, false);
    }
}

/// FR-UST-07/11/23: full valid i128 coupon is not narrowed through float/Decimal views.
#[test]
fn full_coupon_overflow_is_a_source_pv_failure_at_q17() {
    let treasury = regular(21, amount(i128::MAX, 0));
    let plan = TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 21)
        .expect("full-C plan");
    // C/2 ~2^126; Q16 factor2^848 is finite after scaling, Q17 factor2^901 is not.
    for error in [
        plan.evaluate(ABOVE_FLOOR).expect_err("source overflow"),
        plan.risk(ABOVE_FLOOR).expect_err("source overflow"),
    ] {
        assert_pv_failure(error, 17, true);
    }
}

/// FR-UST-11/23: all2,162 finite positive PVs precede an overflowing exact reduction.
#[test]
fn dirty_sum_overflow_follows_successful_individual_pvs() {
    let treasury = regular_from(d(2000, 1, 15), 2162, amount(1_i128 << 126, 0));
    assert_eq!(treasury.maturity_date(), d(3081, 1, 15));
    let plan = TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2162)
        .expect("civil-valid long plan");
    // b=q=.75; final log2PV~1022.31, positive geometric sum log2~1024.31.
    for error in [
        plan.evaluate(-0.5).expect_err("dirty sum overflow"),
        plan.risk(-0.5).expect_err("dirty sum overflow"),
    ] {
        assert!(matches!(
            error,
            TreasuryError::Sum {
                phase: "dirty price reduction",
                ..
            }
        ));
    }
}

/// FR-UST-14/15: genuine10y plan admits D but all three raw getters overflow.
#[test]
fn raw_overflow_does_not_preempt_any_normalized_getter() {
    let treasury = regular_from(d(2025, 7, 15), 20, amount(5, 0));
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 7, 16), 20).expect("r183/s184");
    let risk = plan
        .risk(ABOVE_FLOOR)
        .expect("all accepted PVs and finite dirty");
    assert_eq!(risk.dirty_per_100().to_bits(), 0x7fc2_6aff_ffff_ff96);
    for (getter, phase, sign) in [
        (
            risk.price_derivative(),
            "price derivative",
            NonFinite::NegativeInfinity,
        ),
        (
            risk.price_second_derivative(),
            "price second derivative",
            NonFinite::PositiveInfinity,
        ),
        (
            risk.time_weighted_amount(),
            "time weighted amount",
            NonFinite::PositiveInfinity,
        ),
    ] {
        match getter.expect_err("independent raw overflow") {
            TreasuryError::Value {
                index: None,
                phase: actual_phase,
                cause:
                    ValueError::ResultNotFinite {
                        quantity, value, ..
                    },
                ..
            } => {
                assert_eq!(actual_phase, phase);
                assert_eq!(quantity, phase);
                assert_eq!(value, sign);
            }
            other => panic!("wrong raw range error: {other:?}"),
        }
    }
    assert_eq!(
        risk.modified_duration().expect("retained ratio").to_bits(),
        0x4373_0000_0000_0006
    );
    assert_eq!(
        risk.macaulay_duration().expect("retained ratio").to_bits(),
        0x4023_fe9b_d37a_6f4e
    );
    assert_eq!(
        risk.convexity().expect("retained ratio").to_bits(),
        0x46f7_c000_0000_000d
    );
}

/// FR-UST-14/15: signed raw zero, subnormal Modified and finite Macaulay coexist.
#[test]
fn maximum_yield_keeps_subnormal_modified_and_typed_signed_raw_underflows() {
    let treasury = regular(1, amount(0, 0));
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 1).expect("alpha1/2");
    let risk = plan.risk(f64::MAX).expect("finite principal PV");
    assert_getter_underflow(
        risk.price_derivative().expect_err("negative zero"),
        "price derivative",
        true,
    );
    assert_getter_underflow(
        risk.price_second_derivative().expect_err("positive zero"),
        "price second derivative",
        false,
    );
    assert_getter_underflow(
        risk.convexity().expect_err("positive zero"),
        "convexity",
        false,
    );
    assert_eq!(
        risk.modified_duration()
            .expect("positive subnormal")
            .to_bits(),
        0x0004_0000_0000_0000
    );
    assert_eq!(risk.macaulay_duration().expect("direct moment ratio"), 0.25);
    assert_eq!(
        risk.time_weighted_amount()
            .expect("raw moment cannot vanish")
            .to_bits(),
        0x0059_0000_0000_0002
    );
}

/// FR-UST-11/14/15: single-positive-PV cancellation remains informative at MIN_SUB.
#[test]
fn minimum_subnormal_dirty_keeps_nonzero_moment_and_accurate_normalization() {
    let treasury = regular(2, amount(0, 0));
    let plan = TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2)
        .expect("alpha1,j1 principal");
    let y = f64::from_bits(0x61c8_0000_0000_0000); // 3*2^540.
    let value = plan.evaluate(y).expect("direct PV rounds to MIN_SUB");
    let risk = plan.risk(y).expect("retained extended terms");
    assert_eq!(value.dirty_per_100().to_bits(), 1);
    assert_eq!(risk.dirty_per_100().to_bits(), 1);
    assert_getter_underflow(
        risk.price_derivative().expect_err("negative raw underflow"),
        "price derivative",
        true,
    );
    assert_getter_underflow(
        risk.price_second_derivative()
            .expect_err("positive raw underflow"),
        "price second derivative",
        false,
    );
    assert_getter_underflow(
        risk.convexity().expect_err("positive normalized underflow"),
        "convexity",
        false,
    );
    assert_eq!(
        risk.time_weighted_amount()
            .expect("finite subnormal moment")
            .to_bits(),
        1
    );
    assert_eq!(
        risk.modified_duration()
            .expect("single-term PV cancels")
            .to_bits(),
        0x1e25_5555_5555_5555
    );
    assert_eq!(
        risk.macaulay_duration().expect("single-term PV cancels"),
        1.0
    );
}

/// FR-UST-11/25: a dated public witness kills extended53-then-IEEE PV double rounding.
#[test]
fn direct_subnormal_pv_rounds_seven_min_sub_instead_of_double_rounded_six() {
    let treasury = regular(2, amount(0, 0));
    let plan = TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2)
        .expect("one positive final PV at original j1");
    // Independent target-driven Decimal180 candidate, exact Fraction check:
    // factor_hat=1170935903116329/2^1128, so100*factor_hat/MIN_SUB
    // is13/2+2^-52. Direct RNE gives7; extended53 first loses the2^-52
    // excess, then6.5 ties to even6. Both source/date inputs are admitted.
    let y = f64::from_bits(0x61af_60ea_b9a5_d3a2);
    assert_eq!(
        plan.evaluate(y)
            .expect("direct PV")
            .dirty_per_100()
            .to_bits(),
        7
    );
    let risk = plan.risk(y).expect("same direct source PV");
    assert_eq!(risk.dirty_per_100().to_bits(), 7);
    assert_eq!(risk.time_weighted_amount().expect("tau1").to_bits(), 7);
    assert_eq!(risk.macaulay_duration().expect("direct normalization"), 1.0);
}

/// FR-UST-14/15: nonzero subnormal Convexity is independent of a rawD2 failure.
#[test]
fn convexity_two_min_sub_is_admitted_while_raw_second_derivative_underflows() {
    let treasury = regular(1, amount(0, 0));
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 1).expect("alpha1");
    let risk = plan
        .risk(f64::from_bits(0x6180_0000_0000_0000))
        .expect("yield2^537");
    assert_getter_underflow(
        risk.price_second_derivative().expect_err("tiny rawD2"),
        "price second derivative",
        false,
    );
    assert_eq!(
        risk.convexity()
            .expect("positive subnormal Convexity")
            .to_bits(),
        2
    );
    assert_eq!(
        risk.price_derivative()
            .expect("finite subnormal derivative")
            .to_bits(),
        0x8000_0000_0000_00c8
    );
    assert_eq!(
        risk.modified_duration().expect("Modified").to_bits(),
        0x1e60_0000_0000_0000
    );
    assert_eq!(risk.macaulay_duration().expect("Macaulay"), 0.5);
}

/// FR-UST-09/12/16/17: clean target shift uses published0.013736, not raw5/364.
#[test]
fn published_ai_drives_clean_target_and_all_reports_retain_both_views() {
    let treasury = regular(1, amount(5, 0));
    let settlement = d(2024, 12, 8);
    let plan = TreasuryPricingPlan::new(&treasury, settlement, 1).expect("one elapsed day of182");
    assert_eq!(plan.raw_accrued_per_100(), 5.0 / 364.0);
    assert_eq!(plan.accrued_per_100(), 0.013_736);
    let value = plan.evaluate(0.0).expect("identity discount");
    let clean = f64::from_bits(0x4059_9f1e_f30a_4e38);
    assert_eq!(
        value
            .clean_price()
            .expect("published clean")
            .value_per_100_face()
            .to_bits(),
        clean.to_bits()
    );
    let report = plan
        .yield_from_price(
            quote(clean, BondPriceKind::Clean),
            RootBracket::new(0.0, f64::MAX).expect("explicit bracket"),
            options(1),
        )
        .expect("published shift makes lower exactly zero");
    assert_eq!(report.dirty_target_per_100(), 102.5);
    assert_eq!(report.raw_accrued_per_100(), 5.0 / 364.0);
    assert_eq!(report.accrued_per_100(), 0.013_736);
    assert_eq!(report.source_policy(), exact_even());
    assert_eq!(report.yield_rate().to_bits(), 0);
    assert_eq!(report.solve_report().evaluations().function(), 1);
    assert_eq!(
        report
            .amount_residual()
            .expect("cached difference")
            .to_bits(),
        0
    );
}

/// FR-UST-12/17: native targets and cached getters retain the selected one-call payload.
#[test]
fn exact_dirty_and_clean_targets_keep_cached_metadata_without_repricing() {
    let treasury = regular(1, amount(5, 0));
    let settlement = d(2025, 3, 8);
    let plan = TreasuryPricingPlan::new(&treasury, settlement, 1).expect("half period");
    let bracket = RootBracket::new(0.0, f64::MAX).expect("explicit bracket");
    let policy = options(1);
    for target in [
        quote(102.5, BondPriceKind::Dirty),
        quote(101.25, BondPriceKind::Clean),
    ] {
        let report = plan
            .yield_from_price(target, bracket, policy)
            .expect("lower exact root");
        for _ in 0..4 {
            assert_eq!(report.target(), target);
            assert_eq!(report.dirty_target_per_100(), 102.5);
            assert_eq!(report.dirty_per_100(), 102.5);
            assert_eq!(report.raw_accrued_per_100(), 1.25);
            assert_eq!(report.accrued_per_100(), 1.25);
            assert_eq!(
                report
                    .clean_price()
                    .expect("cached clean")
                    .value_per_100_face(),
                101.25
            );
            assert_eq!(report.dirty_price().value_per_100_face(), 102.5);
            assert_eq!(
                report
                    .amount_residual()
                    .expect("cached difference")
                    .to_bits(),
                0
            );
            assert_eq!(report.initial_bracket(), bracket);
            assert_eq!(report.options(), policy);
            assert_eq!(report.solve_report().evaluations().function(), 1);
            assert_eq!(report.solve_report().evaluations().derivative(), 0);
            assert_eq!(
                report.solve_report().reason(),
                TerminationReason::FunctionZero
            );
            assert_eq!(report.residual_scale_exponent(), 0);
            assert_eq!(report.original_dated_date(), treasury.original_dated_date());
            assert_eq!(report.original_issue_date(), treasury.original_issue_date());
            assert_eq!(report.maturity_date(), treasury.maturity_date());
            assert_eq!(report.first_period(), TreasuryFirstPeriod::Regular);
            assert_eq!(report.settlement_date(), settlement);
            assert_eq!(report.next_quasi_date(), treasury.maturity_date());
            assert_eq!(report.source_version(), treasury.source_version());
            assert_eq!(report.source_policy(), treasury.source_policy());
        }
    }
}

/// FR-UST-16/23: resolved nonpositive targets reject before invalid lower quotes.
#[test]
fn nonpositive_target_retains_policy_and_performs_zero_callbacks() {
    let treasury = regular(1, amount(5, 0));
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 1).expect("plan");
    for target in [
        quote(0.0, BondPriceKind::Dirty),
        quote(-1.0, BondPriceKind::Dirty),
        quote(-1.25, BondPriceKind::Clean),
    ] {
        let error = plan
            .yield_from_price(
                target,
                RootBracket::new(-3.0, -2.0).expect("invalid quotes in valid coordinate bracket"),
                options(2),
            )
            .expect_err("nonpositive resolved target");
        match error {
            TreasuryYieldError::NonPositiveTarget {
                target: original,
                dirty_target_per_100,
                accrued_per_100,
                source_policy,
                evaluations,
                ..
            } => {
                assert_eq!(original, target);
                assert!(dirty_target_per_100 <= 0.0);
                assert_eq!(accrued_per_100, 1.25);
                assert_eq!(source_policy, exact_even());
                assert_eq!(evaluations.function(), 0);
                assert_eq!(evaluations.derivative(), 0);
            }
            other => panic!("wrong target rejection order: {other:?}"),
        }
    }
}

/// FR-UST-12/16/17: a negative clean quote can resolve to a positive exact dirty target.
#[test]
fn negative_clean_target_is_admitted_after_its_published_ai_shift() {
    let treasury = regular(1, amount(5, 0));
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 1).expect("AI1.25");
    // q=1+406/4=102.5; source A=102.5, observed dirty1, clean-.25.
    assert_eq!(
        plan.evaluate(406.0).expect("finite price").dirty_per_100(),
        1.0
    );
    let report = plan
        .yield_from_price(
            quote(-0.25, BondPriceKind::Clean),
            RootBracket::new(406.0, 407.0).expect("explicit bracket"),
            options(1),
        )
        .expect("negative clean with positive resolved dirty");
    assert_eq!(report.dirty_target_per_100(), 1.0);
    assert_eq!(report.dirty_per_100(), 1.0);
    assert_eq!(
        report
            .clean_price()
            .expect("cached clean")
            .value_per_100_face(),
        -0.25
    );
    assert_eq!(report.solve_report().evaluations().function(), 1);
    assert_eq!(
        report.solve_report().reason(),
        TerminationReason::FunctionZero
    );
}

/// FR-UST-17: unqueried invalid upper PV is irrelevant after lower computed zero.
#[test]
fn lower_zero_one_call_success_does_not_validate_the_upper_endpoint() {
    let treasury = regular(2, amount(0, 0));
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    assert_pv_failure(
        plan.evaluate(f64::MAX)
            .expect_err("upper principal underflow"),
        2,
        false,
    );
    let report = plan
        .yield_from_price(
            quote(100.0, BondPriceKind::Dirty),
            RootBracket::new(0.0, f64::MAX).expect("explicit bracket"),
            options(1),
        )
        .expect("lower exact root only");
    assert_eq!(report.solve_report().evaluations().function(), 1);
    assert_eq!(report.solve_report().evaluations().derivative(), 0);
    assert_eq!(
        report.solve_report().reason(),
        TerminationReason::FunctionZero
    );
    assert_eq!(report.dirty_per_100(), 100.0);
}

/// FR-UST-17/23: failed callbacks count; a rejected over-budget callback never occurs.
#[test]
fn lower_upper_and_budget_failures_preserve_actual_counted_order() {
    let treasury = regular(2, amount(0, 0));
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    let target = quote(99.0, BondPriceKind::Dirty);
    let lower = plan
        .yield_from_price(
            target,
            RootBracket::new(-2.0, 0.0).expect("coordinates"),
            options(2),
        )
        .expect_err("invalid lower quote");
    assert!(matches!(lower, TreasuryYieldError::Callback {
        stage: EvaluationStage::LowerEndpoint, point, evaluations,
        cause: TreasuryError::Value { index: None, phase: "yield quote", .. }, ..
    } if point.to_bits()==(-2.0_f64).to_bits() && evaluations.function()==1 && evaluations.derivative()==0));
    let bracket = RootBracket::new(0.0, f64::MAX).expect("coordinates");
    let cap_error = plan
        .yield_from_price(target, bracket, options(1))
        .expect_err("upper not invoked");
    assert!(
        matches!(cap_error, TreasuryYieldError::Solver(SolverError::EvaluationLimit { evaluations, .. }) if evaluations.function()==1 && evaluations.derivative()==0)
    );
    let upper = plan
        .yield_from_price(target, bracket, options(2))
        .expect_err("upper PV failure");
    match upper {
        TreasuryYieldError::Callback {
            stage: EvaluationStage::UpperEndpoint,
            point,
            evaluations,
            cause,
            ..
        } => {
            assert_eq!(point.to_bits(), f64::MAX.to_bits());
            assert_eq!(evaluations.function(), 2);
            assert_eq!(evaluations.derivative(), 0);
            assert_pv_failure(cause, 2, false);
        }
        other => panic!("wrong upper failure: {other:?}"),
    }
    let same_sign = plan
        .yield_from_price(
            quote(0.1, BondPriceKind::Dirty),
            RootBracket::new(0.0, 1.0).expect("coordinates"),
            options(2),
        )
        .expect_err("same-sign residuals");
    assert!(
        matches!(same_sign, TreasuryYieldError::Solver(SolverError::NotBracketed { lower_value, upper_value, evaluations, .. }) if lower_value>0.0 && upper_value>0.0 && evaluations.function()==2 && evaluations.derivative()==0)
    );
}

/// FR-UST-17/25: target-driven y=4*(102.5/T-1), without evaluation-built expectations.
#[test]
fn independent_simple_stub_inverse_anchors_include_conditioning() {
    let treasury = regular(1, amount(5, 0));
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 1).expect("half period");
    for (target, root, lower, upper) in [
        (102.5, 0.0, -0.1, 0.1),
        (51.25, 4.0, 3.0, 5.0),
        (164.0, -1.5, -1.6, -1.4),
        (82.0, 1.0, 0.5, 1.5),
    ] {
        let report = plan
            .yield_from_price(
                quote(target, BondPriceKind::Dirty),
                RootBracket::new(lower, upper).expect("independent root bracket"),
                options(200),
            )
            .expect("bounded inverse");
        let final_bracket = report.solve_report().bracket();
        // On all listed brackets D<=171 and |D'|>4. N3 single-PV error
        // is <2^-40, with the p256 width term included, not fitted to output.
        let bound = (final_bracket.upper() - final_bracket.lower())
            + report
                .amount_residual()
                .expect("cached amount residual")
                .abs()
                / 4.0
            + 2_f64.powi(-40);
        assert!((report.yield_rate() - root).abs() <= bound);
        assert!(report.solve_report().evaluations().function() <= 200);
        assert_eq!(report.solve_report().evaluations().derivative(), 0);
    }
}

/// FR-UST-10/11/17: the prescribed observation can exceed the excluded ideal image.
#[test]
fn observed_supremum_overshoot_is_an_admitted_one_call_target() {
    let treasury = regular(1, amount(5, 0));
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 6, 6), 1).expect("r1/s182,n0");
    let observed = f64::from_bits(0x4059_c43e_3b67_3fa6);
    assert_eq!(
        plan.evaluate(ABOVE_FLOOR)
            .expect("finite observation")
            .dirty_per_100()
            .to_bits(),
        observed.to_bits()
    );
    // Exact ideal supremum18655/181, while RN64(ideal PV) has low bitsfa5.
    // This fa6 value follows lower-endpoint53 factor plus direct source PV.
    assert!(observed > 18_655.0 / 181.0);
    let report = plan
        .yield_from_price(
            quote(observed, BondPriceKind::Dirty),
            RootBracket::new(ABOVE_FLOOR, 0.0).expect("explicit domain bracket"),
            options(1),
        )
        .expect("no ideal-upper precheck may reject an observed roundtrip");
    assert_eq!(report.yield_rate().to_bits(), ABOVE_FLOOR.to_bits());
    assert_eq!(report.dirty_per_100().to_bits(), observed.to_bits());
    assert_eq!(report.solve_report().evaluations().function(), 1);
    assert_eq!(
        report.solve_report().reason(),
        TerminationReason::FunctionZero
    );
}

/// FR-UST-02/12/24: reports own observations and immutable plan reuse is safe.
#[test]
fn retained_reports_outlive_the_definition_and_scoped_threads_share_a_plan() {
    let (value, risk, report) = {
        let treasury = regular(1, amount(5, 0));
        let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 1).expect("prepared plan");
        let value = plan.evaluate(0.0).expect("value");
        let risk = plan.risk(0.0).expect("retained risk");
        std::thread::scope(|scope| {
            for _ in 0..4 {
                let shared = &plan;
                let retained = &risk;
                scope.spawn(move || {
                    for _ in 0..8 {
                        assert_eq!(shared.evaluate(0.0).expect("shared observation"), value);
                        assert_eq!(
                            retained.price_second_derivative().expect("shared getter"),
                            12.8125
                        );
                    }
                });
            }
        });
        let report = plan
            .yield_from_price(
                quote(102.5, BondPriceKind::Dirty),
                RootBracket::new(0.0, 1.0).expect("coordinates"),
                options(1),
            )
            .expect("owned cached report");
        (value, risk, report)
    };
    assert_eq!(value.raw_accrued_per_100(), 1.25);
    assert_eq!(
        value
            .clean_price()
            .expect("owned clean")
            .value_per_100_face(),
        101.25
    );
    assert_eq!(
        six_values(&risk),
        [-25.625, 12.8125, 25.625, 0.25, 0.25, 0.125]
    );
    assert_eq!(report.dirty_per_100(), 102.5);
    assert_eq!(report.solve_report().evaluations().function(), 1);
}
