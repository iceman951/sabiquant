//! Blind pre-production Treasury date/source/cash controls, FR-UST-02..09/18..25.
//!
//! Authored from the adopted complete contracts before Treasury production and
//! without reading the independent Treasury source/cash generator or fixture.
//! This is a future integration-test body outside Cargo's test tree. Rustfmt
//! parsing is not compilation/execution. Exact arithmetic derivations, family
//! mapping and limitations live in M8_US_TREASURY_SOURCE_CASH_HAND_DESIGN.md.

#![expect(clippy::expect_used, reason = "explicit blind hand input assertions")]

use sabiquant::amounts::{AmountError, AmountQuantum, AmountRounding as R, DecimalAmount};
use sabiquant::bonds::{
    CouponEntitlement, TreasuryCashDecision as Decision, TreasuryCashInstruction as Instruction,
    TreasuryCashReceipt as Receipt, TreasuryDailyBasis as Basis, TreasuryError,
    TreasuryFirstPeriod as First, TreasuryPricingPlan, TreasurySourcePolicy as Policy,
    TreasuryTieRule as Tie, UsTreasury,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::error::Error as RootError;
use sabiquant::settlement::{
    CashAggregation as Aggregation, SettlementEvidence as Evidence,
    SettlementGuarantee as Guarantee, SettlementRounding as Rounding,
};
use sabiquant::time::Date;
use std::collections::HashSet;
use std::error::Error;
use std::hash::Hash;

const SOURCE: &str = "US-Treasury-31CFR356-AppB-fixed-2026-10-16-local-v1";

fn d(y: i32, m: u32, day: u32) -> Date {
    Date::from_ymd(y, m, day).expect("valid civil date")
}

fn a(coefficient: i128, scale: u8) -> DecimalAmount {
    DecimalAmount::new(coefficient, scale).expect("valid exact amount")
}

fn p(text: &str) -> DecimalAmount {
    text.parse().expect("valid exact decimal")
}

fn q(coefficient: i128, scale: u8) -> AmountQuantum {
    AmountQuantum::new(coefficient, scale).expect("positive exact quantum")
}

fn round(coefficient: i128, scale: u8, mode: R) -> Rounding {
    Rounding::new(q(coefficient, scale), mode)
}

fn decimal(text: &str) -> Evidence {
    Evidence::exact_decimal(p(text))
}

fn binary(value: f64) -> Evidence {
    Evidence::supplied_binary64(value).expect("finite binary evidence")
}

fn interval(lower: DecimalAmount, upper: DecimalAmount) -> Evidence {
    Evidence::declared_decimal_interval(lower, upper).expect("ordered exact enclosure")
}

fn exact_even() -> Policy {
    Policy::new(Basis::ExactRatio, Tie::NearestTiesEven)
}

#[derive(Clone)]
struct Definition {
    dated: Date,
    issue: Date,
    maturity: Date,
    coupon: DecimalAmount,
    first: Date,
    quasi: Vec<Date>,
    eom: EndOfMonth,
    policy: Policy,
}

impl Definition {
    fn build(self, cap: usize) -> Result<UsTreasury, TreasuryError> {
        UsTreasury::new(
            self.dated,
            self.issue,
            self.maturity,
            self.coupon,
            self.first,
            self.quasi,
            self.eom,
            self.policy,
            cap,
        )
    }
}

fn regular(coupon: DecimalAmount, policy: Policy) -> Definition {
    Definition {
        dated: d(2024, 12, 7),
        issue: d(2024, 12, 7),
        maturity: d(2025, 12, 7),
        coupon,
        first: d(2025, 6, 7),
        quasi: vec![d(2024, 12, 7), d(2025, 6, 7), d(2025, 12, 7)],
        eom: EndOfMonth::Ignore,
        policy,
    }
}

fn staged_shape(role: First, coupon: DecimalAmount, policy: Policy) -> Definition {
    let mut value = Definition {
        dated: d(2025, 7, 15),
        issue: d(2025, 7, 15),
        maturity: d(2026, 7, 15),
        coupon,
        first: d(2026, 1, 15),
        quasi: vec![d(2025, 7, 15), d(2026, 1, 15), d(2026, 7, 15)],
        eom: EndOfMonth::Ignore,
        policy,
    };
    if role != First::Regular {
        value.dated = d(2025, 10, 3);
        value.issue = value.dated;
    }
    if role == First::Long {
        value.first = value.maturity;
    }
    value
}

#[expect(
    clippy::panic,
    reason = "a different decision fails the blind hand control"
)]
fn receipt(
    instruction: Instruction,
    expected: DecimalAmount,
    guarantee: Option<Guarantee>,
) -> Receipt {
    match instruction.evaluate().expect("hand cash evaluation") {
        Decision::Determined(value) => {
            assert_eq!(value.amount(), expected);
            assert_eq!(value.instruction(), &instruction);
            assert_eq!(value.price_evidence_guarantee(), guarantee);
            value
        }
        other => panic!("expected determined cash, got {other:?}"),
    }
}

fn inexact(error: TreasuryError, stage: &str, endpoint: &str) {
    assert!(matches!(error, TreasuryError::CashArithmetic {
        stage: actual_stage, endpoint: actual_endpoint,
        cause: AmountError::Inexact { operation: "quantize", .. }, ..
    } if actual_stage==stage && actual_endpoint==endpoint));
}

fn out_of_range(
    error: TreasuryError,
    stage: &str,
    endpoint: &str,
    operation: &str,
    negative: bool,
) {
    assert!(matches!(error, TreasuryError::CashArithmetic {
        stage: actual_stage, endpoint: actual_endpoint,
        cause: AmountError::OutOfRange { operation: actual_operation, negative: actual_negative, .. }, ..
    } if actual_stage==stage && actual_endpoint==endpoint && actual_operation==operation && actual_negative==negative));
}

fn unresolved(instruction: Instruction, expected_stage: &str) {
    assert!(
        matches!(instruction.evaluate(), Ok(Decision::ExactStageUnresolved { stage, .. }) if stage==expected_stage)
    );
}

/// FR-UST-02/06: explicit non-default source choices and compact Copy/hash contracts.
#[test]
fn source_policy_arrays_const_inspectors_and_copy_traits_are_complete() {
    const POLICY: Policy = Policy::new(Basis::NineDecimal, Tie::NearestTiesAwayFromZero);
    const BASIS: Basis = POLICY.daily_basis();
    const TIE: Tie = POLICY.tie_rule();
    const MODE: R = TIE.amount_rounding();
    assert_eq!(BASIS, Basis::NineDecimal);
    assert_eq!(MODE, R::NearestTiesAwayFromZero);
    assert_eq!(Basis::ALL, [Basis::ExactRatio, Basis::NineDecimal]);
    assert_eq!(
        Tie::ALL,
        [Tie::NearestTiesEven, Tie::NearestTiesAwayFromZero]
    );
    assert_eq!(First::ALL, [First::Regular, First::Short, First::Long]);
    assert_eq!(Tie::NearestTiesEven.amount_rounding(), R::NearestTiesEven);
    fn compact<T: Copy + Eq + Hash + Send + Sync>() {}
    compact::<First>();
    compact::<Basis>();
    compact::<Tie>();
    compact::<Policy>();
    compact::<Instruction>();
    compact::<Decision>();
    compact::<Receipt>();
}

/// FR-UST-03/23: cap, count and minimum-length validation precede financial fields.
#[test]
fn date_cap_and_vector_count_precede_negative_coupon_and_missing_dates() {
    for cap in [0, 1, UsTreasury::MAX_QUASI_DATES + 1] {
        let mut input = regular(a(-1, 0), exact_even());
        input.quasi.clear();
        assert!(
            matches!(input.build(cap), Err(TreasuryError::InvalidDateCap { cap: actual, minimum: 2, maximum: 20_000, .. }) if actual==cap)
        );
    }
    let input = regular(a(-1, 0), exact_even());
    assert!(matches!(
        input.build(2),
        Err(TreasuryError::TooManyQuasiDates {
            count: 3,
            cap: 2,
            ..
        })
    ));
    let mut input = regular(a(-1, 0), exact_even());
    input.quasi.truncate(1);
    assert!(matches!(
        input.build(2),
        Err(TreasuryError::TooFewQuasiDates {
            count: 1,
            minimum: 2,
            ..
        })
    ));
}

/// FR-UST-03/23: negative C, D>I and I>=M have distinct ordered errors.
#[test]
fn coupon_and_original_date_roles_preserve_constructor_precedence() {
    let mut input = regular(a(-1, 0), exact_even());
    input.dated = d(2026, 1, 1);
    input.issue = input.maturity;
    assert!(matches!(
        input.clone().build(3),
        Err(TreasuryError::NegativeCoupon { .. })
    ));
    input.coupon = a(0, 0);
    assert!(matches!(
        input.clone().build(3),
        Err(TreasuryError::DatedAfterIssue { .. })
    ));
    input.dated = input.issue;
    assert!(matches!(
        input.build(3),
        Err(TreasuryError::IssueNotBeforeMaturity { .. })
    ));
}

/// FR-UST-03/04/23: terminal date precedes order, order precedes coverage/cycle.
#[test]
fn terminal_order_and_coverage_errors_keep_supplied_indices() {
    let mut input = regular(a(2, 0), exact_even());
    input.quasi = vec![input.first, input.dated, input.first];
    assert!(matches!(
        input.clone().build(3),
        Err(TreasuryError::MaturityBoundary { .. })
    ));
    input.quasi[2] = input.maturity;
    assert!(
        matches!(input.clone().build(3), Err(TreasuryError::QuasiDateOrder { index: 1, previous, date, .. }) if previous==input.first && date==input.dated)
    );
    input.quasi = vec![d(2024, 1, 1), d(2024, 6, 1), input.maturity];
    assert!(matches!(
        input.build(3),
        Err(TreasuryError::QuasiCoverage { .. })
    ));
}

/// FR-UST-04: Preserve uses month-end; Ignore is a different explicitly valid cycle.
#[test]
fn preserve_and_ignore_choose_different_october_dates_for_april_maturity() {
    let input = Definition {
        dated: d(2025, 4, 30),
        issue: d(2025, 4, 30),
        maturity: d(2026, 4, 30),
        coupon: a(2, 0),
        first: d(2025, 10, 31),
        quasi: vec![d(2025, 4, 30), d(2025, 10, 31), d(2026, 4, 30)],
        eom: EndOfMonth::Preserve,
        policy: exact_even(),
    };
    let preserve = input.clone().build(3).expect("complete month-end cycle");
    let mut ignore = input;
    ignore.eom = EndOfMonth::Ignore;
    assert!(
        matches!(ignore.clone().build(3), Err(TreasuryError::QuasiDateMismatch { index: 1, expected, supplied, .. }) if expected==d(2025,10,30) && supplied==d(2025,10,31))
    );
    ignore.quasi[1] = d(2025, 10, 30);
    ignore.first = ignore.quasi[1];
    let ignore = ignore.build(3).expect("explicit Ignore mathematical cycle");
    assert_ne!(preserve, ignore);
}

/// FR-UST-04: direct original day30 returns after a February clamp.
#[test]
fn maturity_anchoring_does_not_recursively_keep_february_day28() {
    let mut input = Definition {
        dated: d(2025, 8, 30),
        issue: d(2025, 8, 30),
        maturity: d(2026, 8, 30),
        coupon: a(2, 0),
        first: d(2026, 2, 28),
        quasi: vec![d(2025, 8, 30), d(2026, 2, 28), d(2026, 8, 30)],
        eom: EndOfMonth::Ignore,
        policy: exact_even(),
    };
    input.clone().build(3).expect("direct original-day cycle");
    input.quasi[0] = d(2025, 8, 28);
    assert!(
        matches!(input.build(3), Err(TreasuryError::QuasiDateMismatch { index: 0, expected, .. }) if expected==d(2025,8,30))
    );
}

/// FR-UST-04/24: actual supported date extremes without dates outside year1..9999.
#[test]
fn date_min_and_max_can_be_endpoints_of_real_semiannual_definitions() {
    for (start, end, eom) in [
        (Date::MIN, d(1, 7, 1), EndOfMonth::Ignore),
        (d(9999, 6, 30), Date::MAX, EndOfMonth::Preserve),
    ] {
        let input = Definition {
            dated: start,
            issue: start,
            maturity: end,
            coupon: a(0, 0),
            first: end,
            quasi: vec![start, end],
            eom,
            policy: exact_even(),
        };
        let treasury = input.build(2).expect("civil-valid boundary cycle");
        assert_eq!(treasury.original_dated_date(), start);
        assert_eq!(treasury.maturity_date(), end);
        assert_eq!(treasury.coupon_count(), 1);
    }
}

/// FR-UST-05: complete roles and I<F rejection before role inference.
#[test]
fn regular_short_long_and_malformed_first_coupon_roles_are_explicit() {
    for role in First::ALL {
        let treasury = staged_shape(role, a(2, 0), exact_even())
            .build(3)
            .expect("accepted role");
        assert_eq!(treasury.first_period(), role);
        assert_eq!(
            treasury.first_coupon_quasi_index(),
            if role == First::Long { 2 } else { 1 }
        );
    }
    let mut input = regular(a(2, 0), exact_even());
    input.first = input.maturity;
    assert!(matches!(
        input.build(3),
        Err(TreasuryError::FirstCoupon { .. })
    ));
    let mut input = regular(a(2, 0), exact_even());
    input.issue = input.first;
    assert!(matches!(
        input.build(3),
        Err(TreasuryError::FirstCoupon { .. })
    ));
}

/// FR-UST-03/04: coverage must be minimal, including exact D=Q1 rejection.
#[test]
fn redundant_prefix_or_uncovered_dated_date_is_rejected_before_cycle() {
    for dated in [d(2024, 12, 6), d(2025, 6, 7), d(2025, 7, 1)] {
        let mut input = regular(a(2, 0), exact_even());
        input.dated = dated;
        input.issue = dated;
        assert!(matches!(
            input.build(3),
            Err(TreasuryError::QuasiCoverage { .. })
        ));
    }
}

/// FR-UST-08/23: plan cap/domain order and payment-on-S exclusion.
#[test]
fn plan_caps_settlement_domain_and_zero_coupon_entitlements_are_ordered() {
    let treasury = regular(a(0, 0), exact_even())
        .build(3)
        .expect("zero coupons remain actual");
    assert!(matches!(
        TreasuryPricingPlan::new(&treasury, d(2024, 12, 6), 20_001),
        Err(TreasuryError::InvalidBundleCap { .. })
    ));
    assert!(matches!(
        TreasuryPricingPlan::new(&treasury, d(2024, 12, 6), 0),
        Err(TreasuryError::SettlementBeforeIssue { .. })
    ));
    assert!(matches!(
        TreasuryPricingPlan::new(&treasury, treasury.maturity_date(), 0),
        Err(TreasuryError::SettlementAtOrAfterMaturity { .. })
    ));
    assert!(matches!(
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 0),
        Err(TreasuryError::TooManyBundles {
            count: 2,
            cap: 0,
            ..
        })
    ));
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 6, 7), 1)
        .expect("actual first coupon on S excluded");
    assert_eq!(plan.bundle_count(), 1);
    assert_eq!(plan.next_quasi_index(), 2);
    assert_eq!(plan.coupon_entitlement(0), Some(CouponEntitlement::Ceased));
    assert_eq!(
        plan.coupon_entitlement(1),
        Some(CouponEntitlement::BuyerEntitled)
    );
    assert_eq!(plan.coupon_entitlement(2), None);
    assert_eq!(plan.bundle_quasi_index(0), Some(2));
}

/// FR-UST-06/07/18: all four policies retain independently derived source fractions.
#[test]
fn short_daily_tie_views_and_instruction_views_are_identical_for_every_quantizer() {
    for basis in Basis::ALL {
        for tie in Tie::ALL {
            let policy = Policy::new(basis, tie);
            let treasury = staged_shape(First::Short, a(23, 7), policy)
                .build(3)
                .expect("104 of184 first days");
            let selected = if basis == Basis::ExactRatio {
                6500
            } else if tie == Tie::NearestTiesEven {
                6448
            } else {
                6552
            };
            assert_eq!(
                treasury.raw_coupon_per_100(0, q(1, 10), R::Exact),
                Ok(a(6500, 10))
            );
            assert_eq!(
                treasury.source_coupon_per_100(0, q(1, 10), R::Exact),
                Ok(a(selected, 10))
            );
            let instruction = treasury
                .coupon_cash(0, a(1000, 0), round(1, 9, R::NearestTiesEven))
                .expect("positive nominal");
            for quantum in [q(1, 10), q(1, 8), q(3, 10)] {
                for mode in R::ALL {
                    assert_eq!(
                        instruction.source_value_per_100(quantum, mode),
                        treasury.source_coupon_per_100(0, quantum, mode).map(Some)
                    );
                }
            }
        }
    }
    let treasury = staged_shape(First::Long, a(i128::MAX, 0), exact_even())
        .build(3)
        .expect("full-C caches stay private and wide");
    let instruction = treasury
        .coupon_cash(0, a(100, 0), round(1, 0, R::Exact))
        .expect("no source Decimal gate at the factory");
    let source = treasury.source_coupon_per_100(0, q(1_i128 << 126, 0), R::NearestTiesEven);
    assert_eq!(
        instruction.source_value_per_100(q(1_i128 << 126, 0), R::NearestTiesEven),
        source.clone().map(Some)
    );
    out_of_range(
        source.expect_err("rounded source view2^127"),
        "source coupon",
        "point",
        "quantize",
        false,
    );
}

/// FR-UST-07/18: a full coupon is H, never a rounded daily decimal multiplied by s.
#[test]
fn regular_full_coupon_does_not_reconstruct_from_daily_nine() {
    let treasury = staged_shape(
        First::Regular,
        a(5, 0),
        Policy::new(Basis::NineDecimal, Tie::NearestTiesEven),
    )
    .build(3)
    .expect("184-day regular period");
    assert_eq!(
        treasury.source_coupon_per_100(0, q(1, 10), R::Exact),
        Ok(p("2.5"))
    );
    receipt(
        treasury
            .coupon_cash(0, a(1000, 0), round(1, 9, R::Exact))
            .expect("cash"),
        a(25, 0),
        None,
    );
    // RN9(25/184)*184=24.999999960; the exact full source coupon is25.
}

/// FR-UST-07/18: long combines unrounded fractional cash with the full half coupon.
#[test]
fn long_source_coupon_keeps_fractional_policy_stage_without_coupon_level_rounding() {
    let treasury = staged_shape(
        First::Long,
        a(23, 7),
        Policy::new(Basis::NineDecimal, Tie::NearestTiesEven),
    )
    .build(3)
    .expect("long first");
    assert_eq!(
        treasury.raw_coupon_per_100(0, q(1, 10), R::Exact),
        Ok(a(18000, 10))
    );
    assert_eq!(
        treasury.source_coupon_per_100(0, q(1, 10), R::Exact),
        Ok(a(17948, 10))
    );
    receipt(
        treasury
            .coupon_cash(0, a(1000, 0), round(1, 9, R::Exact))
            .expect("source cash"),
        a(17948, 9),
        None,
    );
}

/// FR-UST-07/18: optional scale39 view cannot block exact cash scaling.
#[test]
fn regular_half_coupon_at_scale39_can_be_paid_without_public_decimal_admission() {
    let treasury = regular(a(1, 38), exact_even())
        .build(3)
        .expect("full Decimal coupon domain");
    inexact(
        treasury
            .raw_coupon_per_100(0, q(1, 38), R::Exact)
            .expect_err("half quantum"),
        "raw coupon",
        "point",
    );
    assert_eq!(
        treasury.source_coupon_per_100(0, q(1, 38), R::TowardZero),
        Ok(a(0, 0))
    );
    assert_eq!(
        treasury.source_coupon_per_100(0, q(1, 38), R::AwayFromZero),
        Ok(a(1, 38))
    );
    receipt(
        treasury
            .coupon_cash(0, a(2000, 0), round(1, 38, R::Exact))
            .expect("fused exact cash"),
        a(1, 37),
        None,
    );
}

/// FR-UST-06/09/18: a published zero does not erase positive raw accrual or nominal checks.
#[test]
fn tiny_daily_nine_rounds_source_to_zero_and_retains_positive_raw_values() {
    let treasury = staged_shape(
        First::Short,
        a(1, 38),
        Policy::new(Basis::NineDecimal, Tie::NearestTiesEven),
    )
    .build(3)
    .expect("tiny exact coupon");
    assert_eq!(
        treasury.source_coupon_per_100(0, q(1, 38), R::Exact),
        Ok(a(0, 0))
    );
    let plan =
        TreasuryPricingPlan::new(&treasury, d(2025, 10, 4), 2).expect("one elapsed short day");
    assert!(plan.raw_accrued_per_100() > 0.0);
    assert_eq!(plan.accrued_per_100().to_bits(), 0);
    receipt(
        treasury
            .coupon_cash(0, a(i128::MAX, 0), round(1, 38, R::Exact))
            .expect("positive nominal"),
        a(0, 0),
        None,
    );
    assert!(matches!(
        treasury.coupon_cash(0, a(0, 0), round(1, 0, R::Exact)),
        Err(TreasuryError::NonPositiveNominal { .. })
    ));
}

/// FR-UST-09/19: cash views use the published point, not the nonterminating raw AI.
#[test]
fn published_accrued_view_is_exact_at_its_six_place_quantum() {
    let treasury = regular(a(5, 0), exact_even()).build(3).expect("regular");
    let plan = TreasuryPricingPlan::new(&treasury, d(2024, 12, 8), 2).expect("one day of182");
    assert_eq!(plan.raw_accrued_per_100(), 5.0 / 364.0);
    assert_eq!(plan.accrued_per_100(), 0.013736);
    let instruction = plan
        .accrued_cash(a(1000, 0), round(1, 5, R::Exact))
        .expect("source cached");
    assert_eq!(
        instruction.source_value_per_100(q(1, 6), R::Exact),
        Ok(Some(p("0.013736")))
    );
    receipt(instruction, p("0.13736"), None);
}

/// FR-UST-06/09/19: long pieces are added before their single AI5 publication.
#[test]
fn long_accrued_publication_is_total_first_under_all_four_policies() {
    for basis in Basis::ALL {
        for tie in Tie::ALL {
            let mut input = staged_shape(First::Long, p("0.125"), Policy::new(basis, tie));
            input.dated = d(2026, 1, 14);
            input.issue = input.dated;
            let treasury = input.build(3).expect("one first day plus long full period");
            let plan =
                TreasuryPricingPlan::new(&treasury, d(2026, 1, 17), 1).expect("two second days");
            let instruction = plan
                .accrued_cash(a(100000, 0), round(1, 3, R::Exact))
                .expect("source AI");
            assert_eq!(
                instruction.source_day_ratios(),
                Some(((1, 184), Some((2, 181))))
            );
            assert_eq!(
                instruction.source_value_per_100(q(1, 6), R::Exact),
                Ok(Some(p("0.001030")))
            );
            receipt(instruction, p("1.030"), None);
            // Component-first publication gives .001031 per100 and1.031 cash.
        }
    }
}

/// FR-UST-19: no extra5dp stage after arbitrary nominal; final tie mode is independent.
#[test]
fn fractional_nominal_discriminates_an_extra_accrued_quantizer() {
    for basis in Basis::ALL {
        for tie in Tie::ALL {
            let treasury = staged_shape(First::Regular, p("0.000368"), Policy::new(basis, tie))
                .build(3)
                .expect("daily1e-5 per1000");
            let plan = TreasuryPricingPlan::new(&treasury, d(2025, 7, 16), 2)
                .expect("one elapsed day of184");
            receipt(
                plan.accrued_cash(a(499600, 0), round(1, 2, R::NearestTiesAwayFromZero))
                    .expect("cash .004996"),
                a(0, 0),
                None,
            );
            receipt(
                plan.accrued_cash(a(500000, 0), round(1, 2, R::NearestTiesEven))
                    .expect("half cent"),
                a(0, 0),
                None,
            );
            receipt(
                plan.accrued_cash(a(500000, 0), round(1, 2, R::NearestTiesAwayFromZero))
                    .expect("half cent"),
                a(1, 2),
                None,
            );
        }
    }
}

/// FR-UST-07/18/22: every source/nominal/quantum scale stays a fused exact operation.
#[test]
fn all_39_source_quantum_and_nominal_scales_have_exact_fused_controls() {
    for scale in 0..=38 {
        let treasury = regular(a(2, scale), exact_even())
            .build(3)
            .expect("exact coupon scale");
        receipt(
            treasury
                .coupon_cash(0, a(100, 0), round(1, scale, R::Exact))
                .expect("matching quantum"),
            a(1, scale),
            None,
        );
        let treasury = regular(a(2000, 0), exact_even())
            .build(3)
            .expect("large exact coupon");
        receipt(
            treasury
                .coupon_cash(0, a(1, scale), round(1, 38, R::Exact))
                .expect("exact nominal scale"),
            a(10, scale),
            None,
        );
    }
}

/// FR-UST-18..21: all four recipes carry the exact inspector Some/None matrix.
#[test]
fn four_recipe_inspectors_distinguish_source_values_evidence_and_metadata() {
    let treasury = regular(a(5, 0), exact_even())
        .build(3)
        .expect("regular source");
    let settlement = d(2025, 3, 8);
    let plan = TreasuryPricingPlan::new(&treasury, settlement, 2).expect("91 elapsed days");
    let rounding = round(1, 2, R::Exact);
    let nominal = a(1000, 0);
    let coupon = treasury.coupon_cash(0, nominal, rounding).expect("coupon");
    let accrued = plan.accrued_cash(nominal, rounding).expect("accrued");
    let clean = plan
        .clean_accrued_cash(nominal, decimal("100"), Aggregation::RoundTotal, rounding)
        .expect("invoice");
    let dirty = plan
        .dirty_cash(nominal, decimal("101.25"), rounding)
        .expect("dirty");
    for instruction in [coupon, accrued, clean, dirty] {
        assert_eq!(instruction.source_version(), SOURCE);
        assert_eq!(instruction.source_policy(), exact_even());
        assert_eq!(instruction.nominal(), nominal);
        assert_eq!(instruction.cash_rounding(), rounding);
        assert_eq!(instruction.coupon_per_100(), a(5, 0));
        assert_eq!(instruction.original_dated_date(), d(2024, 12, 7));
        assert_eq!(instruction.original_issue_date(), d(2024, 12, 7));
        assert_eq!(instruction.first_coupon_date(), d(2025, 6, 7));
        assert_eq!(instruction.maturity_date(), d(2025, 12, 7));
        assert_eq!(instruction.first_period(), First::Regular);
        assert_eq!(instruction.end_of_month(), EndOfMonth::Ignore);
        assert_eq!(
            instruction.dated_quasi_dates(),
            (d(2024, 12, 7), d(2025, 6, 7))
        );
    }
    assert_eq!(
        [
            coupon.recipe_identifier(),
            accrued.recipe_identifier(),
            clean.recipe_identifier(),
            dirty.recipe_identifier()
        ],
        [
            "SourceCouponPer1000",
            "PublishedAccruedPer1000",
            "CleanPricePer100AndPublishedAccruedPer1000",
            "DirtyPricePer100"
        ]
    );
    assert_eq!(coupon.settlement_date(), None);
    assert_eq!(coupon.settlement_quasi_dates(), None);
    assert_eq!(coupon.source_coupon_index(), Some(0));
    assert_eq!(coupon.source_coupon_date(), Some(d(2025, 6, 7)));
    assert_eq!(coupon.source_day_ratios(), None);
    assert_eq!(coupon.price_evidence(), None);
    assert_eq!(coupon.aggregation(), None);
    assert_eq!(
        coupon.source_value_per_100(q(1, 2), R::Exact),
        Ok(Some(p("2.5")))
    );
    for instruction in [accrued, clean, dirty] {
        assert_eq!(instruction.settlement_date(), Some(settlement));
        assert_eq!(
            instruction.settlement_quasi_dates(),
            Some((d(2024, 12, 7), d(2025, 6, 7)))
        );
    }
    for instruction in [accrued, clean] {
        assert_eq!(instruction.source_coupon_index(), Some(0));
        assert_eq!(instruction.source_coupon_date(), Some(d(2025, 6, 7)));
        assert_eq!(instruction.source_day_ratios(), Some(((91, 182), None)));
        assert_eq!(
            instruction.source_value_per_100(q(1, 2), R::Exact),
            Ok(Some(p("1.25")))
        );
    }
    assert_eq!(accrued.price_evidence(), None);
    assert_eq!(accrued.aggregation(), None);
    assert_eq!(clean.price_evidence(), Some(decimal("100")));
    assert_eq!(clean.aggregation(), Some(Aggregation::RoundTotal));
    assert_eq!(dirty.price_evidence(), Some(decimal("101.25")));
    assert_eq!(dirty.aggregation(), None);
    assert_eq!(dirty.source_coupon_index(), None);
    assert_eq!(dirty.source_coupon_date(), None);
    assert_eq!(dirty.source_day_ratios(), None);
    assert_eq!(
        dirty.source_value_per_100(q(i128::MAX, 38), R::Exact),
        Ok(None)
    );
    receipt(coupon, a(25, 0), None);
    receipt(accrued, p("12.50"), None);
    receipt(clean, p("1012.50"), Some(Guarantee::ForSuppliedDecimals));
    receipt(dirty, p("1012.50"), Some(Guarantee::ForSuppliedDecimals));
}

/// FR-UST-05/09/19: long provenance carries both original pieces across skipped Q1.
#[test]
fn long_issue_after_skipped_date_preserves_dated_metadata_and_actual_reset() {
    let mut input = staged_shape(First::Long, a(6, 0), exact_even());
    input.dated = d(2025, 10, 3);
    input.issue = d(2026, 2, 1);
    let treasury = input.build(3).expect("long I>Q1");
    let plan =
        TreasuryPricingPlan::new(&treasury, d(2026, 2, 1), 1).expect("17 second-period days");
    let instruction = plan
        .accrued_cash(a(1000, 0), round(1, 5, R::Exact))
        .expect("two-piece snapshot");
    assert_eq!(instruction.original_dated_date(), d(2025, 10, 3));
    assert_eq!(instruction.original_issue_date(), d(2026, 2, 1));
    assert_eq!(
        instruction.source_day_ratios(),
        Some(((104, 184), Some((17, 181))))
    );
    assert_eq!(instruction.source_coupon_index(), Some(0));
    assert_eq!(instruction.source_coupon_date(), Some(d(2026, 7, 15)));
    assert_eq!(
        instruction.dated_quasi_dates(),
        (d(2025, 7, 15), d(2026, 1, 15))
    );
    assert_eq!(
        instruction.settlement_quasi_dates(),
        Some((d(2026, 1, 15), d(2026, 7, 15)))
    );
    let mut input = regular(a(6, 0), exact_even());
    input.dated = d(2025, 3, 8);
    input.issue = input.dated;
    input.first = input.maturity;
    input.maturity = d(2026, 6, 7);
    input.quasi.push(input.maturity);
    let treasury = input.build(4).expect("long first before final maturity");
    let plan =
        TreasuryPricingPlan::new(&treasury, d(2025, 6, 7), 2).expect("skipped date is tradable");
    assert_eq!(plan.accrued_per_100(), 1.5);
    assert_eq!(
        plan.coupon_entitlement(0),
        Some(CouponEntitlement::BuyerEntitled)
    );
    let reset = TreasuryPricingPlan::new(&treasury, d(2025, 12, 7), 1)
        .expect("actual first coupon on S is excluded");
    assert_eq!(reset.raw_accrued_per_100().to_bits(), 0);
    assert_eq!(reset.accrued_per_100().to_bits(), 0);
    assert_eq!(reset.coupon_entitlement(0), Some(CouponEntitlement::Ceased));
    let snapshot = reset
        .accrued_cash(a(1000, 0), round(1, 0, R::Exact))
        .expect("next actual coupon metadata");
    assert_eq!(snapshot.source_coupon_index(), Some(1));
    assert_eq!(snapshot.source_coupon_date(), Some(d(2026, 6, 7)));
    assert_eq!(snapshot.source_day_ratios(), Some(((0, 182), None)));
    receipt(snapshot, a(0, 0), None);
}

/// FR-UST-18..23: index precedes nominal; all plan recipes validate nominal first.
#[test]
fn factories_and_views_preserve_index_then_positive_nominal_checks() {
    let treasury = regular(a(0, 0), exact_even())
        .build(3)
        .expect("zero source");
    let rounding = round(1, 0, R::Exact);
    assert!(matches!(
        treasury.coupon_cash(2, a(0, 0), rounding),
        Err(TreasuryError::InvalidCouponIndex {
            index: 2,
            count: 2,
            ..
        })
    ));
    assert!(matches!(
        treasury.raw_coupon_per_100(2, q(1, 0), R::Exact),
        Err(TreasuryError::InvalidCouponIndex {
            index: 2,
            count: 2,
            ..
        })
    ));
    assert!(matches!(
        treasury.source_coupon_per_100(2, q(1, 0), R::Exact),
        Err(TreasuryError::InvalidCouponIndex {
            index: 2,
            count: 2,
            ..
        })
    ));
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    for nominal in [a(0, 0), a(-1, 0)] {
        for result in [
            treasury.coupon_cash(0, nominal, rounding),
            plan.accrued_cash(nominal, rounding),
            plan.clean_accrued_cash(nominal, decimal("0"), Aggregation::RoundTotal, rounding),
            plan.dirty_cash(nominal, decimal("0"), rounding),
        ] {
            assert!(
                matches!(result,Err(TreasuryError::NonPositiveNominal { nominal: actual, .. }) if actual==nominal)
            );
        }
    }
}

/// FR-UST-20/22: total and component quantizers differ at genuine half cents.
#[test]
fn invoice_aggregation_and_signed_capital_have_distinct_exact_results() {
    let treasury = regular(a(2, 0), exact_even())
        .build(3)
        .expect("AI .5 per100");
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 2).expect("half period");
    for (aggregation, mode, expected) in [
        (Aggregation::RoundTotal, R::NearestTiesEven, p("0.01")),
        (
            Aggregation::RoundComponentsThenSum,
            R::NearestTiesEven,
            p("0"),
        ),
        (
            Aggregation::RoundComponentsThenSum,
            R::NearestTiesAwayFromZero,
            p("0.02"),
        ),
    ] {
        receipt(
            plan.clean_accrued_cash(a(1, 0), decimal("0.5"), aggregation, round(1, 2, mode))
                .expect("two half-cent components"),
            expected,
            Some(Guarantee::ForSuppliedDecimals),
        );
    }
    for (aggregation, expected) in [
        (Aggregation::RoundTotal, p("-0.01")),
        (Aggregation::RoundComponentsThenSum, p("-0.02")),
    ] {
        receipt(
            plan.clean_accrued_cash(
                a(1, 0),
                decimal("-1.5"),
                aggregation,
                round(1, 2, R::NearestTiesEven),
            )
            .expect("signed capital plus positive AI"),
            expected,
            Some(Guarantee::ForSuppliedDecimals),
        );
    }
}

/// FR-UST-18..22: all seven cash modes and arbitrary .25 quantum across four recipes.
#[test]
fn every_cash_mode_handles_signed_quarter_ties_and_point_exact_failures() {
    let coupon_treasury = regular(p("1.25"), exact_even())
        .build(3)
        .expect("coupon .625 per100");
    let treasury = regular(p("2.5"), exact_even())
        .build(3)
        .expect("AI .625 per100");
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 2).expect("half period");
    for (mode, positive, negative) in [
        (R::TowardZero, p("0.5"), p("-0.5")),
        (R::AwayFromZero, p("0.75"), p("-0.75")),
        (R::TowardPositiveInfinity, p("0.75"), p("-0.5")),
        (R::TowardNegativeInfinity, p("0.5"), p("-0.75")),
        (R::NearestTiesEven, p("0.5"), p("-0.5")),
        (R::NearestTiesAwayFromZero, p("0.75"), p("-0.75")),
    ] {
        let rounding = round(25, 2, mode);
        receipt(
            coupon_treasury
                .coupon_cash(0, a(100, 0), rounding)
                .expect("coupon"),
            positive,
            None,
        );
        receipt(
            plan.accrued_cash(a(100, 0), rounding).expect("AI"),
            positive,
            None,
        );
        receipt(
            plan.clean_accrued_cash(a(100, 0), decimal("0"), Aggregation::RoundTotal, rounding)
                .expect("invoice"),
            positive,
            Some(Guarantee::ForSuppliedDecimals),
        );
        receipt(
            plan.dirty_cash(a(100, 0), decimal("0.625"), rounding)
                .expect("dirty"),
            positive,
            Some(Guarantee::ForSuppliedDecimals),
        );
        receipt(
            plan.dirty_cash(a(100, 0), decimal("-0.625"), rounding)
                .expect("signed dirty"),
            negative,
            Some(Guarantee::ForSuppliedDecimals),
        );
    }
    let rounding = round(25, 2, R::Exact);
    for (instruction, stage) in [
        (
            coupon_treasury
                .coupon_cash(0, a(100, 0), rounding)
                .expect("coupon"),
            "cash coupon",
        ),
        (
            plan.accrued_cash(a(100, 0), rounding).expect("accrued"),
            "cash accrued",
        ),
        (
            plan.clean_accrued_cash(a(100, 0), decimal("0"), Aggregation::RoundTotal, rounding)
                .expect("invoice"),
            "cash total",
        ),
        (
            plan.dirty_cash(a(100, 0), decimal("0.625"), rounding)
                .expect("dirty"),
            "cash total",
        ),
    ] {
        inexact(
            instruction.evaluate().expect_err("nonmultiple point"),
            stage,
            "point",
        );
    }
}

/// FR-UST-22: equal endpoint values retain decimal/binary and declared provenance.
#[test]
fn four_equal_price_evidence_kinds_keep_distinct_instructions_and_receipts() {
    let treasury = regular(a(0, 0), exact_even()).build(3).expect("source");
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    let evidence = [
        decimal("1.25"),
        binary(1.25),
        interval(p("1.25"), p("1.2500")),
        Evidence::declared_binary64_interval(1.25, 1.25).expect("point binary enclosure"),
    ];
    let guarantees = [
        Guarantee::ForSuppliedDecimals,
        Guarantee::ForSuppliedBinary64,
        Guarantee::UniqueForDeclaredEnclosure,
        Guarantee::UniqueForDeclaredEnclosure,
    ];
    let mut instructions = HashSet::new();
    let mut receipts = HashSet::new();
    for (evidence, guarantee) in evidence.into_iter().zip(guarantees) {
        let instruction = plan
            .dirty_cash(a(100, 0), evidence, round(1, 2, R::Exact))
            .expect("owned evidence");
        instructions.insert(instruction);
        receipts.insert(receipt(instruction, p("1.25"), Some(guarantee)));
    }
    assert_eq!(instructions.len(), 4);
    assert_eq!(receipts.len(), 4);
}

/// FR-UST-22: Exact must hold for continuous interiors, including adjacent floats.
#[test]
fn nonpoint_exact_is_unresolved_even_when_endpoints_are_quantum_multiples() {
    let treasury = regular(a(0, 0), exact_even()).build(3).expect("source");
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    for evidence in [
        interval(a(0, 0), a(1, 0)),
        Evidence::declared_binary64_interval(1.0, f64::from_bits(1.0_f64.to_bits() + 1))
            .expect("adjacent binary enclosure"),
    ] {
        unresolved(
            plan.dirty_cash(a(100, 0), evidence, round(1, 0, R::Exact))
                .expect("continuous evidence"),
            "cash total",
        );
        unresolved(
            plan.clean_accrued_cash(
                a(100, 0),
                evidence,
                Aggregation::RoundComponentsThenSum,
                round(1, 0, R::Exact),
            )
            .expect("continuous capital"),
            "cash capital",
        );
    }
}

/// FR-UST-22/23: point Exact reports its actual source/view/cash stage.
#[test]
fn point_exact_errors_retain_source_view_and_component_stage_names() {
    let treasury = regular(p("1.25"), exact_even())
        .build(3)
        .expect("source .625");
    inexact(
        treasury
            .raw_coupon_per_100(0, q(1, 0), R::Exact)
            .expect_err("raw view"),
        "raw coupon",
        "point",
    );
    inexact(
        treasury
            .source_coupon_per_100(0, q(1, 0), R::Exact)
            .expect_err("source view"),
        "source coupon",
        "point",
    );
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 2).expect("AI .3125");
    let ai = plan
        .accrued_cash(a(100, 0), round(1, 0, R::Exact))
        .expect("source snapshot");
    inexact(
        ai.source_value_per_100(q(1, 0), R::Exact)
            .expect_err("published AI view"),
        "source accrued",
        "point",
    );
    inexact(ai.evaluate().expect_err("AI cash"), "cash accrued", "point");
    let invoice = plan
        .clean_accrued_cash(
            a(100, 0),
            decimal("1.25"),
            Aggregation::RoundComponentsThenSum,
            round(1, 0, R::Exact),
        )
        .expect("two inexact components");
    inexact(
        invoice.evaluate().expect_err("capital before accrued"),
        "cash capital",
        "point",
    );
}

/// FR-UST-22: quantization collapse does not make a continuous input point Exact.
#[test]
fn collapsed_rounded_enclosure_stays_unresolved_under_exact_mode() {
    let treasury = regular(a(0, 0), exact_even()).build(3).expect("source");
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    let evidence = interval(p("1.01"), p("1.02"));
    receipt(
        plan.dirty_cash(a(100, 0), evidence, round(1, 0, R::NearestTiesEven))
            .expect("collapse"),
        a(1, 0),
        Some(Guarantee::UniqueForDeclaredEnclosure),
    );
    unresolved(
        plan.dirty_cash(a(100, 0), evidence, round(1, 0, R::Exact))
            .expect("same input continuum"),
        "cash total",
    );
}

/// FR-UST-20/22: final endpoint labels reflect the stage's own input image.
#[test]
fn component_net_pointness_differs_from_nonpoint_total_after_rounding_collapse() {
    let treasury = regular(a(1_i128 << 126, 0), exact_even())
        .build(3)
        .expect("full-C source");
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 2).expect("AI2^124");
    let evidence = interval(a((1_i128 << 124) - 1, 0), a((1_i128 << 124) + 1, 0));
    let rounding = round(1_i128 << 126, 0, R::NearestTiesEven);
    let total = plan
        .clean_accrued_cash(a(400, 0), evidence, Aggregation::RoundTotal, rounding)
        .expect("nonpoint total");
    out_of_range(
        total.evaluate().expect_err("2^127 final"),
        "cash total",
        "lower",
        "quantize",
        false,
    );
    let components = plan
        .clean_accrued_cash(
            a(400, 0),
            evidence,
            Aggregation::RoundComponentsThenSum,
            rounding,
        )
        .expect("capital endpoints both round to2^126");
    out_of_range(
        components.evaluate().expect_err("point net2^127"),
        "cash net",
        "point",
        "sum",
        false,
    );
}

/// FR-UST-22: ambiguous diagnostics have bounds, while provenance stays on the instruction.
#[test]
fn ambiguous_decisions_compare_diagnostics_without_erasing_instruction_identity() {
    let mut instructions = Vec::new();
    let mut decisions = Vec::new();
    for basis in Basis::ALL {
        let treasury = regular(a(0, 0), Policy::new(basis, Tie::NearestTiesEven))
            .build(3)
            .expect("source policy");
        let plan =
            TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
        let instruction = plan
            .dirty_cash(
                a(100, 0),
                interval(p("1.24"), p("1.26")),
                round(1, 2, R::NearestTiesEven),
            )
            .expect("nonpoint image");
        let decision = instruction.evaluate().expect("ambiguity is a decision");
        assert!(
            matches!(decision,Decision::QuantumAmbiguous { lower_amount,upper_amount,.. } if lower_amount==p("1.24") && upper_amount==p("1.26"))
        );
        instructions.push(instruction);
        decisions.push(decision);
    }
    assert_ne!(instructions[0], instructions[1]);
    assert_eq!(decisions[0], decisions[1]);
}

/// FR-UST-22: supplied binary.1 is its exact dyadic, never silently decimal.1.
#[test]
fn binary_and_decimal_point_evidence_differ_at_an_exact_fine_quantum() {
    let treasury = regular(a(0, 0), exact_even()).build(3).expect("source");
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    receipt(
        plan.dirty_cash(a(100, 0), decimal("0.1"), round(1, 20, R::Exact))
            .expect("decimal"),
        p("0.1"),
        Some(Guarantee::ForSuppliedDecimals),
    );
    let instruction = plan
        .dirty_cash(a(100, 0), binary(0.1), round(1, 20, R::Exact))
        .expect("binary point");
    inexact(
        instruction
            .evaluate()
            .expect_err("dyadic denominator exceeds quantum's power2"),
        "cash total",
        "point",
    );
}

/// FR-UST-20/22/24: components may exceed i128 and still cancel to a valid final net.
#[test]
fn wide_components_cancel_without_an_intermediate_decimal_or_range_gate() {
    let treasury = regular(a(1_i128 << 126, 0), exact_even())
        .build(3)
        .expect("C remains valid i128");
    let plan =
        TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 2).expect("published AI exactly2^124");
    for aggregation in Aggregation::ALL {
        let instruction = plan
            .clean_accrued_cash(
                a(1000, 0),
                Evidence::exact_decimal(a(-(1_i128 << 124), 0)),
                aggregation,
                round(1, 0, R::Exact),
            )
            .expect("wide snapshot");
        receipt(instruction, a(0, 0), Some(Guarantee::ForSuppliedDecimals));
    }
}

/// FR-UST-20/22: net cancellation cannot waive a preceding capital Exact failure.
#[test]
fn component_capital_exact_failure_precedes_an_otherwise_zero_net() {
    let treasury = regular(a(1_i128 << 126, 0), exact_even())
        .build(3)
        .expect("full source");
    let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 2).expect("AI2^124");
    let clean = Evidence::exact_decimal(a(-(1_i128 << 124), 0));
    receipt(
        plan.clean_accrued_cash(
            a(1000, 0),
            clean,
            Aggregation::RoundTotal,
            round(3, 0, R::Exact),
        )
        .expect("total zero"),
        a(0, 0),
        Some(Guarantee::ForSuppliedDecimals),
    );
    let components = plan
        .clean_accrued_cash(
            a(1000, 0),
            clean,
            Aggregation::RoundComponentsThenSum,
            round(3, 0, R::Exact),
        )
        .expect("wide components");
    inexact(
        components
            .evaluate()
            .expect_err("10*2^124 is not divisible by3"),
        "cash capital",
        "point",
    );
}

/// FR-UST-21/22: both signed final limits are admitted, with final-only overflow afterwards.
#[test]
fn exact_dirty_cash_admits_both_i128_limits_and_rejects_wider_final_amounts() {
    let treasury = regular(a(0, 0), exact_even()).build(3).expect("source");
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    for limit in [i128::MIN, i128::MAX] {
        let evidence = Evidence::exact_decimal(a(limit, 0));
        receipt(
            plan.dirty_cash(a(100, 0), evidence, round(1, 0, R::Exact))
                .expect("final exact limit"),
            a(limit, 0),
            Some(Guarantee::ForSuppliedDecimals),
        );
        let wider = plan
            .dirty_cash(a(200, 0), evidence, round(1, 0, R::Exact))
            .expect("wide factory");
        out_of_range(
            wider.evaluate().expect_err("final overflow"),
            "cash total",
            "point",
            "quantize",
            limit < 0,
        );
    }
}

/// FR-UST-21..23: final lower admission precedes upper; signs/endpoints stay explicit.
#[test]
fn interval_final_range_admits_lower_before_upper() {
    let treasury = regular(a(0, 0), exact_even()).build(3).expect("source");
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    for (lower, endpoint, negative) in [(i128::MIN, "lower", true), (0, "upper", false)] {
        let instruction = plan
            .dirty_cash(
                a(200, 0),
                interval(a(lower, 0), a(i128::MAX, 0)),
                round(1, 0, R::TowardZero),
            )
            .expect("ordered price interval");
        out_of_range(
            instruction
                .evaluate()
                .expect_err("one or both final bounds overflow"),
            "cash total",
            endpoint,
            "quantize",
            negative,
        );
    }
}

/// FR-UST-21/22/24: binary subnormals/MAX are decoded exactly without implicit float money.
#[test]
fn extreme_binary_evidence_keeps_price_guarantee_and_final_admission() {
    let treasury = regular(a(0, 0), exact_even()).build(3).expect("source");
    let plan =
        TreasuryPricingPlan::new(&treasury, treasury.original_issue_date(), 2).expect("plan");
    for value in [f64::from_bits(1), -f64::from_bits(1)] {
        receipt(
            plan.dirty_cash(
                a(i128::MAX, 0),
                binary(value),
                round(1, 38, R::NearestTiesEven),
            )
            .expect("tiny exact dyadic"),
            a(0, 0),
            Some(Guarantee::ForSuppliedBinary64),
        );
    }
    for value in [f64::MAX, -f64::MAX] {
        let instruction = plan
            .dirty_cash(a(1, 38), binary(value), round(1, 0, R::TowardZero))
            .expect("finite supplied binary");
        out_of_range(
            instruction
                .evaluate()
                .expect_err("even minimum nominal cannot fit MAX money"),
            "cash total",
            "point",
            "quantize",
            value < 0.0,
        );
    }
}

/// FR-UST-02/18..22: equal source cash retains policies and original issue provenance.
#[test]
fn determined_source_identity_retains_policy_and_original_issue_date() {
    let mut definitions = HashSet::new();
    let mut instructions = HashSet::new();
    let mut receipts = HashSet::new();
    for basis in Basis::ALL {
        for tie in Tie::ALL {
            for reopened in [false, true] {
                let mut input = regular(a(2, 0), Policy::new(basis, tie));
                if reopened {
                    input.issue = d(2024, 12, 8);
                }
                let treasury = input.build(3).expect("D<=I<F original roles");
                definitions.insert(treasury.clone());
                let instruction = treasury
                    .coupon_cash(0, a(100, 0), round(1, 0, R::Exact))
                    .expect("full coupon1");
                instructions.insert(instruction);
                receipts.insert(receipt(instruction, a(1, 0), None));
            }
        }
    }
    assert_eq!(instructions.len(), 8);
    assert_eq!(receipts.len(), 8);
    assert_eq!(definitions.len(), 8);
    let input = regular(a(2, 0), exact_even());
    let small_cap = input.clone().build(3).expect("tight explicit cap");
    let large_cap = input
        .build(UsTreasury::MAX_QUASI_DATES)
        .expect("resource cap differs");
    assert_eq!(small_cap, large_cap);
    assert_eq!(
        small_cap.coupon_cash(0, a(100, 0), round(1, 0, R::Exact)),
        large_cap.coupon_cash(0, a(100, 0), round(1, 0, R::Exact))
    );
}

/// FR-UST-02/18..24: owned Copy snapshots survive borrowed plan/definition destruction.
#[test]
fn cash_snapshot_outlives_instrument_and_preserves_typed_root_error_chain() {
    let instruction = {
        let treasury = regular(a(5, 0), exact_even()).build(3).expect("source");
        let plan = TreasuryPricingPlan::new(&treasury, d(2025, 3, 8), 2).expect("plan");
        plan.accrued_cash(a(1000, 0), round(1, 0, R::Exact))
            .expect("owned snapshot")
    };
    assert_eq!(instruction.source_version(), SOURCE);
    assert_eq!(
        instruction.source_value_per_100(q(1, 2), R::Exact),
        Ok(Some(p("1.25")))
    );
    let error = instruction
        .evaluate()
        .expect_err("12.5 is not integer cash");
    inexact(error.clone(), "cash accrued", "point");
    assert_eq!(error.kind(), "treasury.cash_arithmetic");
    assert!(
        error
            .source()
            .expect("typed AmountError cause")
            .downcast_ref::<AmountError>()
            .is_some()
    );
    let root = RootError::from(error);
    assert_eq!(root.kind(), "treasury.cash_arithmetic");
    assert!(
        root.source()
            .expect("Treasury wrapper")
            .downcast_ref::<TreasuryError>()
            .is_some()
    );
}
