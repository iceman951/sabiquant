//! FR-GILT-11/13/17/19/21/23: owned identity, immutable getters and metadata.
//! Post-production public controls, separate from independent numerical oracles.

#![expect(clippy::expect_used, reason = "explicit valid test inputs")]

use sabiquant::amounts::{AmountQuantum, AmountRounding, DecimalAmount};
use sabiquant::bonds::{
    BondPrice, BondPriceKind, ConventionalGilt, GiltCashDecision, GiltCashInstruction,
    GiltCashReceipt, GiltFirstPeriod, GiltPricingPlan,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::math::{RootBracket, SolverOptions, TerminationReason};
use sabiquant::settlement::{
    CashAggregation, SettlementEvidence, SettlementGuarantee, SettlementRounding,
};
use sabiquant::time::Date;
use std::collections::HashSet;

fn d(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("date")
}
fn a(coefficient: i128, scale: u8) -> DecimalAmount {
    DecimalAmount::new(coefficient, scale).expect("amount")
}
fn rounding(mode: AmountRounding) -> SettlementRounding {
    SettlementRounding::new(AmountQuantum::new(1, 2).expect("cent quantum"), mode)
}
fn book(eom: EndOfMonth, cutoff: Date) -> ConventionalGilt {
    let issue = d(2024, 12, 7);
    let maturity = d(2025, 6, 7);
    ConventionalGilt::new(
        issue,
        maturity,
        a(4, 0),
        maturity,
        vec![issue, maturity],
        vec![cutoff],
        eom,
        2,
    )
    .expect("regular source")
}
fn receipt(instruction: GiltCashInstruction) -> GiltCashReceipt {
    let receipt = match instruction.evaluate().expect("cash") {
        GiltCashDecision::Determined(receipt) => Some(receipt),
        _ => None,
    }
    .expect("a supplied exact point must be determined");
    assert_eq!(*receipt.instruction(), instruction);
    receipt
}

#[test]
fn equal_cash_preserves_all_four_price_evidence_identities() {
    let gilt = book(EndOfMonth::Ignore, d(2025, 6, 1));
    let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 1).expect("plan");
    let evidence = [
        SettlementEvidence::exact_decimal(a(1, 0)),
        SettlementEvidence::supplied_binary64(1.0).expect("binary point"),
        SettlementEvidence::declared_decimal_interval(a(1, 0), a(1, 0))
            .expect("declared decimal point"),
        SettlementEvidence::declared_binary64_interval(1.0, 1.0).expect("declared binary point"),
    ];
    let expected = [
        SettlementGuarantee::ForSuppliedDecimals,
        SettlementGuarantee::ForSuppliedBinary64,
        SettlementGuarantee::UniqueForDeclaredEnclosure,
        SettlementGuarantee::UniqueForDeclaredEnclosure,
    ];
    let mut instructions = HashSet::new();
    let mut receipts = HashSet::new();
    for (evidence, guarantee) in evidence.into_iter().zip(expected) {
        let instruction = plan
            .dirty_cash(
                a(100, 0),
                evidence,
                rounding(AmountRounding::NearestTiesEven),
            )
            .expect("instruction");
        let value = receipt(instruction);
        assert_eq!(value.amount(), a(1, 0));
        assert_eq!(value.price_evidence_guarantee(), Some(guarantee));
        assert!(instructions.insert(instruction));
        assert!(receipts.insert(value));
        assert!(!instructions.insert(instruction));
        assert!(!receipts.insert(value));
    }
    assert_eq!(instructions.len(), 4);
    assert_eq!(receipts.len(), 4);
}

#[test]
fn policies_and_source_context_remain_identity_even_when_cash_coincides() {
    let ordinary = book(EndOfMonth::Ignore, d(2025, 6, 1));
    let alternate_cutoff = book(EndOfMonth::Ignore, d(2025, 6, 2));
    let alternate_eom = book(EndOfMonth::Preserve, d(2025, 6, 1));
    let mut instructions = HashSet::new();
    let mut receipts = HashSet::new();
    for gilt in [&ordinary, &alternate_cutoff, &alternate_eom] {
        let plan = GiltPricingPlan::new(gilt, d(2025, 3, 8), 1).expect("plan");
        assert_eq!(plan.accrued_per_100(), 1.0);
        for aggregation in [
            CashAggregation::RoundTotal,
            CashAggregation::RoundComponentsThenSum,
        ] {
            for mode in [AmountRounding::TowardZero, AmountRounding::NearestTiesEven] {
                let instruction = plan
                    .clean_accrued_cash(
                        a(100, 0),
                        SettlementEvidence::exact_decimal(a(0, 0)),
                        aggregation,
                        rounding(mode),
                    )
                    .expect("invoice");
                let value = receipt(instruction);
                assert_eq!(value.amount(), a(1, 0));
                assert!(instructions.insert(instruction));
                assert!(receipts.insert(value));
            }
        }
    }
    assert_eq!(instructions.len(), 12);
    assert_eq!(receipts.len(), 12);
    // Canonical inputs intentionally share identity. Distinct provenance above
    // is checked by set membership, never by assuming collision-free hashes.
    let plan = GiltPricingPlan::new(&ordinary, d(2025, 3, 8), 1).expect("plan");
    let decimal = |value| {
        plan.dirty_cash(
            a(100, 0),
            SettlementEvidence::exact_decimal(value),
            rounding(AmountRounding::Exact),
        )
        .expect("decimal")
    };
    assert_eq!(decimal(a(100, 2)), decimal(a(1, 0)));
    let binary = |value| {
        plan.dirty_cash(
            a(100, 0),
            SettlementEvidence::supplied_binary64(value).expect("finite evidence"),
            rounding(AmountRounding::Exact),
        )
        .expect("binary")
    };
    assert_eq!(binary(-0.0), binary(0.0));
    assert_eq!(receipt(binary(-0.0)), receipt(binary(0.0)));
}

#[test]
fn every_cash_recipe_owns_its_source_after_definition_drop_and_crosses_threads() {
    let instructions = {
        let gilt = book(EndOfMonth::Ignore, d(2025, 6, 1));
        let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 1).expect("plan");
        let cash = rounding(AmountRounding::Exact);
        [
            gilt.dividend_cash(0, a(50, 0), cash).expect("dividend"),
            plan.accrued_cash(a(100, 0), cash).expect("accrued"),
            plan.dirty_cash(a(100, 0), SettlementEvidence::exact_decimal(a(1, 0)), cash)
                .expect("dirty"),
            plan.clean_accrued_cash(
                a(100, 0),
                SettlementEvidence::exact_decimal(a(0, 0)),
                CashAggregation::RoundComponentsThenSum,
                cash,
            )
            .expect("invoice"),
        ]
    };
    for (instruction, source) in
        instructions
            .into_iter()
            .zip([Some(a(2, 0)), Some(a(1, 0)), None, Some(a(1, 0))])
    {
        assert_eq!(
            instruction
                .source_value_per_100(
                    AmountQuantum::new(1, 2).expect("cent view"),
                    AmountRounding::Exact
                )
                .expect("owned source view"),
            source,
        );
    }
    let local = instructions.map(receipt);
    assert_eq!(HashSet::from(local).len(), 4);
    std::thread::scope(|scope| {
        let handles: Vec<_> = instructions
            .into_iter()
            .map(|instruction| {
                scope.spawn(move || {
                    let first = receipt(instruction);
                    for _ in 0..16 {
                        assert_eq!(receipt(instruction), first);
                        assert_eq!(
                            instruction.evaluate().expect("repeated decision"),
                            GiltCashDecision::Determined(first)
                        );
                    }
                    first
                })
            })
            .collect();
        for (handle, expected) in handles.into_iter().zip(local) {
            let actual = handle.join().expect("immutable cash worker");
            assert_eq!(actual, expected);
            assert_eq!(actual.amount(), a(1, 0));
        }
    });
}

#[test]
fn all_owned_price_risk_and_inverse_context_survives_long_first_definition() {
    let issue = d(1999, 5, 7);
    let maturity = d(1999, 12, 7);
    let settlement = d(1999, 9, 7);
    let gilt = ConventionalGilt::new(
        issue,
        maturity,
        a(6, 0),
        maturity,
        vec![d(1998, 12, 7), d(1999, 6, 7), maturity],
        vec![d(1999, 11, 30)],
        EndOfMonth::Ignore,
        3,
    )
    .expect("Long first");
    let plan = GiltPricingPlan::new(&gilt, settlement, 1).expect("plan");
    let value = plan.evaluate(0.0).expect("price");
    let risk = plan.risk(0.0).expect("risk");
    let report = plan
        .yield_from_price(
            value.dirty_price(),
            RootBracket::new(0.0, 1.0).expect("bracket"),
            SolverOptions::new(1e-12, 0.0, 1).expect("one call"),
        )
        .expect("cached lower payload");
    macro_rules! context {
        ($value:expr) => {{
            assert_eq!($value.source_version(), gilt.source_version());
            assert_eq!($value.issue_date(), issue);
            assert_eq!($value.maturity_date(), maturity);
            assert_eq!($value.coupon_per_100(), a(6, 0));
            assert_eq!($value.first_dividend_date(), maturity);
            assert_eq!($value.first_period(), GiltFirstPeriod::Long);
            assert_eq!($value.end_of_month(), EndOfMonth::Ignore);
            assert_eq!($value.settlement_date(), settlement);
            assert_eq!($value.next_quasi_date(), maturity);
            assert_eq!($value.remaining_quasi_periods(), 0);
            assert_eq!($value.remaining_days(), 91);
            assert_eq!($value.quasi_period_days(), 183);
            assert_eq!($value.yield_rate().to_bits(), 0);
        }};
    }
    context!(value);
    context!(risk);
    context!(report);
    assert_eq!(value.dirty_per_100(), risk.dirty_per_100());
    assert_eq!(value.dirty_per_100(), report.dirty_per_100());
    // Root finding here only tests owned metadata, not an independent root.
    assert_eq!(report.target(), value.dirty_price());
    assert_eq!(report.solve_report().evaluations().function(), 1);
    let text = format!("{risk:?}");
    assert!(text.contains("GiltRisk") && text.contains("metadata"));
    assert!(!text.contains("Wide") && !text.contains("significand"));
}

#[test]
fn upper_endpoint_zero_retains_cached_payload_with_exactly_two_callbacks() {
    let gilt = book(EndOfMonth::Ignore, d(2025, 6, 1));
    let plan = GiltPricingPlan::new(&gilt, gilt.issue_date(), 1).expect("plan");
    let target = BondPrice::new(102.0, BondPriceKind::Dirty).expect("exact y0 target");
    let report = plan
        .yield_from_price(
            target,
            RootBracket::new(-1.0, 0.0).expect("bracket"),
            SolverOptions::new(1e-12, 0.0, 2).expect("two calls"),
        )
        .expect("upper endpoint root");
    assert_eq!(
        report.solve_report().reason(),
        TerminationReason::FunctionZero
    );
    assert_eq!(report.solve_report().evaluations().function(), 2);
    assert_eq!(report.solve_report().evaluations().derivative(), 0);
    assert_eq!(report.yield_rate().to_bits(), 0);
    assert_eq!(report.dirty_price(), target);
    assert_eq!(
        report
            .amount_residual()
            .expect("cached difference")
            .to_bits(),
        0
    );
}

#[test]
fn failed_raw_risk_getters_do_not_change_repeated_or_reordered_views() {
    let gilt = book(EndOfMonth::Ignore, d(2025, 6, 1));
    let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 1).expect("half period plan");
    let risk = plan.risk(f64::MAX).expect("finite positive price");
    let forward = [
        risk.price_derivative(),
        risk.price_second_derivative(),
        risk.time_weighted_amount(),
        risk.modified_duration(),
        risk.macaulay_duration(),
        risk.convexity(),
    ];
    assert!(forward[0].is_err() && forward[1].is_err());
    assert!(
        forward[3]
            .as_ref()
            .is_ok_and(|value| value.is_subnormal() && *value > 0.0)
    );
    let reverse = [
        risk.convexity(),
        risk.macaulay_duration(),
        risk.modified_duration(),
        risk.time_weighted_amount(),
        risk.price_second_derivative(),
        risk.price_derivative(),
    ];
    assert_eq!(
        forward,
        reverse.into_iter().rev().collect::<Vec<_>>().as_slice()
    );
    for _ in 0..16 {
        assert_eq!(
            forward,
            [
                risk.price_derivative(),
                risk.price_second_derivative(),
                risk.time_weighted_amount(),
                risk.modified_duration(),
                risk.macaulay_duration(),
                risk.convexity()
            ]
        );
    }
}

/// FR-GILT-06/10/11: positive subnormal dirty prices remain native per100
/// observations. This is a supplied-value/control check, not an ideal root.
#[test]
fn positive_subnormal_dirty_payload_is_not_flushed_or_rescaled_to_amount_units() {
    let issue = d(2000, 1, 1);
    let first = d(2000, 7, 1);
    let maturity = d(2001, 1, 1);
    let gilt = ConventionalGilt::new(
        issue,
        maturity,
        a(0, 0),
        first,
        vec![issue, first, maturity],
        vec![first, maturity],
        EndOfMonth::Ignore,
        3,
    )
    .expect("regular zero-coupon source retains both observations");
    let plan = GiltPricingPlan::new(&gilt, d(2000, 6, 23), 2).expect("eight days before Q1");
    assert_eq!((plan.remaining_days(), plan.quasi_period_days()), (8, 182));
    let lower = f64::MAX / 2.0;
    let value = plan.evaluate(lower).expect("positive subnormal PV");
    assert!(value.dirty_per_100().is_subnormal() && value.dirty_per_100() > 0.0);
    let risk = plan.risk(lower).expect("retained subnormal-price risk");
    assert_eq!(
        risk.dirty_per_100().to_bits(),
        value.dirty_per_100().to_bits()
    );
    assert!(
        risk.modified_duration()
            .expect("direct normalization")
            .is_subnormal()
    );
    let report = plan
        .yield_from_price(
            value.dirty_price(),
            RootBracket::new(lower, f64::MAX).expect("finite bracket"),
            SolverOptions::new(1.0, 0.0, 1).expect("one callback"),
        )
        .expect("lower cached payload");
    assert_eq!(report.dirty_price(), value.dirty_price());
    assert_eq!(report.residual_scale_exponent(), 0);
    assert_eq!(report.solve_report().evaluations().function(), 1);
    assert_eq!(report.solve_report().residual().to_bits(), 0);
    assert_eq!(
        report
            .amount_residual()
            .expect("exact subnormal cancellation")
            .to_bits(),
        0
    );
}

/// FR-GILT-18/21: every recipe's public Debug remains readable and omits its
/// private arithmetic buffers, including accrued/invoice source formulas.
#[test]
fn all_cash_recipe_debug_views_preserve_context_without_private_buffers() {
    let gilt = book(EndOfMonth::Ignore, d(2025, 6, 1));
    let plan = GiltPricingPlan::new(&gilt, d(2025, 3, 8), 1).expect("plan");
    let cash = rounding(AmountRounding::Exact);
    let instructions = [
        gilt.dividend_cash(0, a(100, 0), cash).expect("dividend"),
        plan.accrued_cash(a(100, 0), cash).expect("accrued"),
        plan.clean_accrued_cash(
            a(100, 0),
            SettlementEvidence::exact_decimal(a(0, 0)),
            CashAggregation::RoundTotal,
            cash,
        )
        .expect("invoice"),
        plan.dirty_cash(a(100, 0), SettlementEvidence::exact_decimal(a(1, 0)), cash)
            .expect("dirty"),
    ];
    for (index, instruction) in instructions.into_iter().enumerate() {
        let text = format!("{instruction:?}");
        assert!(text.contains("GiltCashInstruction"));
        for field in [
            "source_version",
            "coupon_per_100",
            "issue_date",
            "settlement_date",
            "cash_rounding",
            "price_evidence",
            "aggregation",
        ] {
            assert!(text.contains(field), "missing readable {field}: {text}");
        }
        assert!(text.contains(instruction.recipe_identifier()));
        assert!(text.contains(if index == 3 {
            "source: None"
        } else {
            "CouponDayRatio"
        }));
        assert!(!text.contains("Wide") && !text.contains("limbs"));
        let value = receipt(instruction);
        for view in [
            format!("{value:?}"),
            format!("{:?}", GiltCashDecision::Determined(value)),
        ] {
            assert!(view.contains("GiltCashReceipt") && view.contains("GiltCashInstruction"));
            assert!(!view.contains("Wide") && !view.contains("limbs"));
        }
    }
}
