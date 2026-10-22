//! FR-UST-02/24: post-production public Debug contract supplements.
//!
//! Authored after production review. These are not
//! blind financial expectations or additions to any frozen hand/oracle file.
//! The two risk inputs are the already frozen raw-overflow and maximum-yield
//! witnesses. This file never calls the raw/normalized risk getters: formatting
//! must succeed without requesting a getter that could fail or dumping its
//! retained numeric arrays.

#![expect(
    clippy::expect_used,
    reason = "literal valid test inputs and prescribed successful constructors"
)]

use sabiquant::amounts::{AmountQuantum, AmountRounding, DecimalAmount};
use sabiquant::bonds::{
    TreasuryCashInstruction, TreasuryDailyBasis, TreasuryPricingPlan, TreasuryRisk,
    TreasurySourcePolicy, TreasuryTieRule, UsTreasury,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::settlement::{CashAggregation, SettlementEvidence, SettlementRounding};
use sabiquant::time::Date;
use std::fmt;

fn date(text: &str) -> Date {
    text.parse().expect("literal valid civil date")
}

fn amount(text: &str) -> DecimalAmount {
    text.parse().expect("literal valid exact amount")
}

fn policy() -> TreasurySourcePolicy {
    TreasurySourcePolicy::new(
        TreasuryDailyBasis::NineDecimal,
        TreasuryTieRule::NearestTiesAwayFromZero,
    )
}

fn regular(issue: Date, periods: u32, coupon: DecimalAmount) -> UsTreasury {
    let maturity = issue
        .checked_add_months(6 * i64::from(periods))
        .expect("bounded valid maturity");
    let quasi: Vec<_> = (0..=periods)
        .map(|index| {
            maturity
                .checked_add_months(-6 * i64::from(periods - index))
                .expect("direct maturity-anchored cycle")
        })
        .collect();
    UsTreasury::new(
        issue,
        issue,
        maturity,
        coupon,
        quasi[1],
        quasi,
        EndOfMonth::Ignore,
        policy(),
        UsTreasury::MAX_QUASI_DATES,
    )
    .expect("valid regular Treasury")
}

fn risk_witnesses() -> [TreasuryRisk; 2] {
    let long = regular(date("2025-07-15"), 20, amount("5"));
    let long_plan = TreasuryPricingPlan::new(&long, date("2025-07-16"), 20)
        .expect("frozen raw-overflow witness");
    let overflow = long_plan
        .risk(f64::from_bits(0xbfff_ffff_ffff_ffff))
        .expect("finite dirty with retained raw-overflow sums");
    let short = regular(date("2024-12-07"), 1, amount("0"));
    let short_plan = TreasuryPricingPlan::new(&short, date("2025-03-08"), 1)
        .expect("frozen maximum-yield witness");
    let underflow = short_plan
        .risk(f64::MAX)
        .expect("finite principal PV with raw derivative underflows");
    [overflow, underflow]
}

fn cash_witnesses() -> Vec<TreasuryCashInstruction> {
    // Short and long D<I definitions deliberately expose the two original
    // date roles and both Some/None metadata shapes. Long C is large enough
    // that public source/final amount admission need not succeed; Debug never
    // quantizes either source or cash.
    let inputs = [
        (
            date("2025-03-08"),
            date("2025-03-10"),
            date("2025-06-07"),
            vec![date("2024-12-07"), date("2025-06-07"), date("2025-12-07")],
            amount("6.125"),
            date("2025-03-11"),
        ),
        (
            date("2025-03-08"),
            date("2025-07-01"),
            date("2025-12-07"),
            vec![date("2024-12-07"), date("2025-06-07"), date("2025-12-07")],
            DecimalAmount::new(i128::MAX, 0).expect("largest admitted coupon"),
            date("2025-07-02"),
        ),
    ];
    let rounding = SettlementRounding::new(
        AmountQuantum::new(3, 3).expect("arbitrary positive quantum"),
        AmountRounding::Exact,
    );
    let nominal = DecimalAmount::new(i128::MAX, 0).expect("positive large nominal");
    let clean = SettlementEvidence::declared_decimal_interval(amount("-0.005"), amount("0.005"))
        .expect("declared continuous interval");
    let dirty = SettlementEvidence::supplied_binary64(f64::from_bits(1))
        .expect("finite supplied minimum subnormal");
    let mut result = Vec::new();
    for (dated, issue, first, quasi, coupon, settlement) in inputs {
        let treasury = UsTreasury::new(
            dated,
            issue,
            date("2025-12-07"),
            coupon,
            first,
            quasi,
            EndOfMonth::Ignore,
            policy(),
            3,
        )
        .expect("valid source metadata witness");
        let plan = TreasuryPricingPlan::new(&treasury, settlement, 2)
            .expect("valid one/two-piece source settlement");
        result.extend([
            treasury
                .coupon_cash(0, nominal, rounding)
                .expect("coupon snapshot"),
            plan.accrued_cash(nominal, rounding)
                .expect("accrued snapshot"),
            plan.clean_accrued_cash(
                nominal,
                clean,
                CashAggregation::RoundComponentsThenSum,
                rounding,
            )
            .expect("invoice snapshot"),
            plan.dirty_cash(nominal, dirty, rounding)
                .expect("dirty snapshot"),
        ]);
    }
    result
}

fn assert_no_numeric_buffers(text: &str) {
    assert!(
        text.len() < 4096,
        "compact Debug bound: {} bytes",
        text.len()
    );
    for internal in [
        "Wide(",
        "Scalar",
        "magnitude:",
        "first:",
        "second:",
        "moment:",
    ] {
        assert!(
            !text.contains(internal),
            "private numeric buffer leaked: {internal}"
        );
    }
    assert!(
        !text.contains("Err("),
        "Debug must not format failing getter results"
    );
}

/// A byte budget that accepts complete formatter writes up to its limit.
/// Chunk boundaries do not matter: each budget below the complete length must
/// fail somewhere, and the complete length must produce exactly the same text.
struct FailAfter {
    remaining: usize,
    text: String,
}

impl fmt::Write for FailAfter {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if text.len() > self.remaining {
            return Err(fmt::Error);
        }
        self.remaining -= text.len();
        self.text.push_str(text);
        Ok(())
    }
}

fn check_every_formatter_budget(value: &impl fmt::Debug) {
    let complete = format!("{value:?}");
    assert_no_numeric_buffers(&complete);
    for budget in 0..=complete.len() {
        let mut output = FailAfter {
            remaining: budget,
            text: String::new(),
        };
        let result = fmt::write(&mut output, format_args!("{value:?}"));
        assert!(output.text.len() <= budget);
        assert!(complete.starts_with(&output.text));
        if budget == complete.len() {
            assert!(result.is_ok(), "complete byte budget must succeed");
            assert_eq!(output.text, complete);
        } else {
            assert!(
                result.is_err(),
                "short byte budget{budget} must propagate fmt failure"
            );
        }
    }
}

/// FR-UST-02/24: Debug uses metadata, yield and dirty even when raw getters
/// would fail. The known financial getter states are already tested in the
/// frozen blind suite; this supplement never requests those getters.
#[test]
fn risk_debug_extreme_reports_remain_compact_and_unchanged() {
    for risk in risk_witnesses() {
        let before = risk.clone();
        let text = format!("{risk:?}");
        assert_no_numeric_buffers(&text);
        for field in [
            "TreasuryRisk",
            "metadata",
            "yield_rate",
            "dirty_per_100",
            "source_policy",
        ] {
            assert!(text.contains(field), "missing public context {field}");
        }
        assert!(text.contains(&format!("{:?}", risk.yield_rate())));
        assert!(text.contains(&format!("{:?}", risk.dirty_per_100())));
        assert!(text.contains(&format!("{:?}", risk.original_dated_date())));
        assert!(text.contains(&format!("{:?}", risk.original_issue_date())));
        assert_eq!(
            risk, before,
            "formatting must not change the retained report"
        );
    }
}

/// FR-UST-02/24: all four recipe shapes retain explicit option provenance in
/// Debug without forcing source/final quantization or dumping source caches.
#[test]
fn cash_debug_all_recipes_retains_public_snapshot_and_options() {
    for instruction in cash_witnesses() {
        let before = instruction;
        let text = format!("{instruction:?}");
        assert_no_numeric_buffers(&text);
        assert!(text.contains("TreasuryCashInstruction"));
        assert!(text.contains(instruction.recipe_identifier()));
        assert!(text.contains(instruction.source_version()));
        assert!(text.contains(&format!("{:?}", instruction.source_policy())));
        assert!(text.contains(&format!("{:?}", instruction.original_dated_date())));
        assert!(text.contains(&format!("{:?}", instruction.original_issue_date())));
        for option in [
            format!("price_evidence: {:?}", instruction.price_evidence()),
            format!("aggregation: {:?}", instruction.aggregation()),
            format!("settlement_date: {:?}", instruction.settlement_date()),
            format!(
                "source_coupon_index: {:?}",
                instruction.source_coupon_index()
            ),
        ] {
            assert!(
                text.contains(&option),
                "missing explicit option context {option}"
            );
        }
        assert_eq!(instruction, before);
    }
}

/// FR-UST-02/24: every insufficient byte budget propagates fmt::Error, while
/// the exact complete budget succeeds; no stable whole-string fixture is used.
#[test]
fn treasury_custom_debug_propagates_every_formatter_byte_budget() {
    for risk in risk_witnesses() {
        let before = risk.clone();
        check_every_formatter_budget(&risk);
        assert_eq!(risk, before);
    }
    for instruction in cash_witnesses() {
        let before = instruction;
        check_every_formatter_budget(&instruction);
        assert_eq!(instruction, before);
    }
}

/// FR-UST-02/06: the four policy definitions retain distinct explicit values,
/// and their runtime inspectors and tie-to-cash mappings preserve every pair.
#[test]
fn source_policy_cross_product_retains_exact_inspectors_and_identity() {
    use std::collections::HashSet;
    let mut policies = HashSet::new();
    for basis in [
        TreasuryDailyBasis::ExactRatio,
        TreasuryDailyBasis::NineDecimal,
    ] {
        for (tie, rounding) in [
            (
                TreasuryTieRule::NearestTiesEven,
                AmountRounding::NearestTiesEven,
            ),
            (
                TreasuryTieRule::NearestTiesAwayFromZero,
                AmountRounding::NearestTiesAwayFromZero,
            ),
        ] {
            let policy = std::hint::black_box(TreasurySourcePolicy::new(basis, tie));
            assert_eq!(policy.daily_basis(), basis);
            assert_eq!(policy.tie_rule(), tie);
            assert_eq!(policy.tie_rule().amount_rounding(), rounding);
            assert!(policies.insert(policy));
        }
    }
    assert_eq!(policies.len(), 4);
}
