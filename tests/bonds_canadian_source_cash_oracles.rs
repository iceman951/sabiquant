//! FR-CAD-01..32/60..71: all79,794 frozen source/date/cash operations.
//! The36 documentary observations are checked as documentary evidence, not
//! financial Rust runtime cases. The79,758 public operations preserve complete
//! constructor/plan/factory/query/evaluation order and exact contextual errors.
//!
//! All74 physical columns have explicit input/output/annotation roles. Exact
//! source ratios bind public q1/q1e-38 views (Exact and nearest/even), plus the
//! once-rounded A/AI observations; this is not a new public rational inspector.
//! Selected bundle coefficients additionally bind the observed y0 dirty sum.
//! Lambda ingredients are independently checked against supplied date geometry;
//! their actual kernel use is an N/P/Y adapter obligation. MM h/r ingredients
//! bind public payment dates. No fixture count closes a complete CAD family.
#[path = "support/treasury.rs"]
mod arithmetic;
mod support;
use arithmetic::{
    Rat, TestResult, amount, bump, cell, count_json, date, dates, evidence, fnv64, hex, quantum,
    sha256,
};
use sabiquant::amounts::{AmountError, AmountQuantum, AmountRounding, DecimalAmount};
use sabiquant::bonds::{
    CanadianBond, CanadianCashDecision, CanadianCashInstruction, CanadianCouponSchedule,
    CanadianError, CanadianFrequency, CanadianPricingPlan, CanadianSourcePolicy,
    CanadianYieldConvention, CanadianYieldSelection, CouponEntitlement,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::error::Error as RootError;
use sabiquant::settlement::{CashAggregation, SettlementGuarantee, SettlementRounding};
use sabiquant::time::{Date, DateError};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::hash::{DefaultHasher, Hash, Hasher};
use support::fixture::{Fixture, Record};
const PATH: &str = "validation/fixtures/bonds/canadian-source-cash-oracle.fixture";
const SOURCE: &str = "CA-IIAC13-fixed-2026-10-23-local-v1";
const COLUMNS: [&str; 74] = [
    "id",
    "profile",
    "scope",
    "op",
    "dated",
    "issue",
    "stated_maturity",
    "true_maturity",
    "frequency",
    "eom",
    "quasi",
    "coupon_dates",
    "coupon",
    "date_cap",
    "source_policy",
    "settlement",
    "selection",
    "bundle_cap",
    "coupon_index",
    "source_item",
    "nominal",
    "evidence",
    "aggregation",
    "quantum",
    "mode",
    "view_quantum",
    "view_mode",
    "reference",
    "expected",
    "context",
    "source_version",
    "first_period",
    "last_period",
    "first_coupon",
    "last_coupon_start",
    "quasi_count",
    "coupon_count",
    "anchor",
    "valuation_coupons",
    "payable_coupons",
    "next_coupon_index",
    "next_coupon_date",
    "next_payment_date",
    "actual_accrual_start",
    "valuation_ai",
    "settlement_ai",
    "valuation_ai_bits",
    "settlement_ai_bits",
    "quote_accrued_bits",
    "resolved_convention",
    "bundle_ordinals",
    "bundle_coefficients",
    "bundle_lambdas",
    "bundle_h_days",
    "bundle_r_days",
    "entitlements",
    "source_value",
    "src_interval",
    "src_quasi_pairs",
    "src_full_count",
    "src_day_ratios",
    "recipe_identifier",
    "src_index",
    "src_date",
    "src_payment",
    "view_amount",
    "lower_amount",
    "upper_amount",
    "guarantee",
    "reference_classification",
    "fr_families",
    "identity_group",
    "definition_sha256",
    "source_observation",
];
const INPUTS: [&str; 25] = [
    "op",
    "dated",
    "issue",
    "stated_maturity",
    "true_maturity",
    "frequency",
    "eom",
    "quasi",
    "coupon_dates",
    "coupon",
    "date_cap",
    "source_policy",
    "settlement",
    "selection",
    "bundle_cap",
    "coupon_index",
    "source_item",
    "nominal",
    "evidence",
    "aggregation",
    "quantum",
    "mode",
    "view_quantum",
    "view_mode",
    "reference",
];
const OUTPUTS: [&str; 43] = [
    "expected",
    "context",
    "source_version",
    "first_period",
    "last_period",
    "first_coupon",
    "last_coupon_start",
    "quasi_count",
    "coupon_count",
    "anchor",
    "valuation_coupons",
    "payable_coupons",
    "next_coupon_index",
    "next_coupon_date",
    "next_payment_date",
    "actual_accrual_start",
    "valuation_ai",
    "settlement_ai",
    "valuation_ai_bits",
    "settlement_ai_bits",
    "quote_accrued_bits",
    "resolved_convention",
    "bundle_ordinals",
    "bundle_coefficients",
    "bundle_lambdas",
    "bundle_h_days",
    "bundle_r_days",
    "entitlements",
    "source_value",
    "src_interval",
    "src_quasi_pairs",
    "src_full_count",
    "src_day_ratios",
    "recipe_identifier",
    "src_index",
    "src_date",
    "src_payment",
    "view_amount",
    "lower_amount",
    "upper_amount",
    "guarantee",
    "reference_classification",
    "source_observation",
];
const ANNOTATIONS: [&str; 5] = [
    "profile",
    "scope",
    "fr_families",
    "identity_group",
    "definition_sha256",
];
fn input_projection(op: &str) -> TestResult<&'static [&'static str]> {
    Ok(match op {
        "cash_accrued" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
            "settlement",
            "selection",
            "bundle_cap",
            "nominal",
            "quantum",
            "mode",
        ],
        "cash_coupon" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
            "coupon_index",
            "nominal",
            "quantum",
            "mode",
        ],
        "cash_dirty" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
            "settlement",
            "selection",
            "bundle_cap",
            "nominal",
            "evidence",
            "quantum",
            "mode",
        ],
        "cash_invoice" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
            "settlement",
            "selection",
            "bundle_cap",
            "nominal",
            "evidence",
            "aggregation",
            "quantum",
            "mode",
        ],
        "instruction_view_accrued" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
            "settlement",
            "selection",
            "bundle_cap",
            "nominal",
            "quantum",
            "mode",
            "view_quantum",
            "view_mode",
        ],
        "instruction_view_coupon" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
            "coupon_index",
            "nominal",
            "quantum",
            "mode",
            "view_quantum",
            "view_mode",
        ],
        "instruction_view_dirty" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
            "settlement",
            "selection",
            "bundle_cap",
            "nominal",
            "evidence",
            "quantum",
            "mode",
            "view_quantum",
            "view_mode",
        ],
        "instruction_view_invoice" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
            "settlement",
            "selection",
            "bundle_cap",
            "nominal",
            "evidence",
            "aggregation",
            "quantum",
            "mode",
            "view_quantum",
            "view_mode",
        ],
        "instrument" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
        ],
        "plan" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
            "settlement",
            "selection",
            "bundle_cap",
        ],
        "schedule" => &[
            "op",
            "dated",
            "stated_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "date_cap",
        ],
        "source_reference" => &["op", "reference"],
        "source_view_accrued" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
            "settlement",
            "selection",
            "bundle_cap",
            "source_item",
            "quantum",
            "mode",
        ],
        "source_view_coupon" => &[
            "op",
            "dated",
            "issue",
            "stated_maturity",
            "true_maturity",
            "frequency",
            "eom",
            "quasi",
            "coupon_dates",
            "coupon",
            "date_cap",
            "source_policy",
            "coupon_index",
            "source_item",
            "quantum",
            "mode",
        ],
        _ => return Err("unapproved Canadian operation".into()),
    })
}
const DOCUMENTARY: [(&str, &str, &str, &str); 36] = [
    (
        "iiac11#pdf-index=16;section=4.5.3;cell=short-last-48day-result",
        "0.65734247",
        "printed_arithmetic_error",
        "48/73",
    ),
    (
        "iiac11#pdf-index=17;section=4.5.3;cell=short-last-183day-result",
        "4.986301370",
        "extra_full_coupon_in_printed_result",
        "363/146",
    ),
    (
        "iiac11#pdf-index=54;section=appendix2;cell=nominal10000-settlementAI",
        "83.298",
        "printed_AI_cell_error",
        "6080/73",
    ),
    (
        "iiac12#pdf-index=16;section=4.5.3;cell=short-last-48day-result",
        "0.65734247",
        "printed_arithmetic_error",
        "48/73",
    ),
    (
        "iiac12#pdf-index=17;section=4.5.3;cell=short-last-183day-result",
        "4.986301370",
        "extra_full_coupon_in_printed_result",
        "363/146",
    ),
    (
        "iiac12#pdf-index=54;section=appendix2;cell=nominal10000-settlementAI",
        "83.298",
        "printed_AI_cell_error",
        "6080/73",
    ),
    (
        "iiac13#pdf-index=16;section=4.5.3;cell=short-last-48day-result",
        "0.65734247",
        "printed_arithmetic_error",
        "48/73",
    ),
    (
        "iiac13#pdf-index=17;section=4.5.3;cell=short-last-183day-result",
        "4.986301370",
        "extra_full_coupon_in_printed_result",
        "363/146",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal10000-settlementAI",
        "83.298",
        "printed_AI_cell_error",
        "6080/73",
    ),
    (
        "iiac11#pdf-index=20;section=6;cell=late-Canadian365-branch",
        "C*(1/f-(s-d)/365)",
        "late_branch_retained",
        "-",
    ),
    (
        "iiac12#pdf-index=20;section=6;cell=late-Canadian365-branch",
        "late expression/example omitted; below182.5 branch atpdf30",
        "release12_omission_not_equivalent_formula",
        "-",
    ),
    (
        "iiac13#pdf-index=20;section=6;cell=late-Canadian365-branch",
        "C*(1/f-(s-d)/365)",
        "late_branch_retained",
        "-",
    ),
    (
        "iiac13#pdf-index=42;section=10.7.2;cell=monthly-settlement-AI",
        "C*DCS/365",
        "monthly_reference_conflicts_with_general_ActAct",
        "-",
    ),
    (
        "iiac13#pdf-index=49;section=10.10.3;cell=general-MM-price-formula",
        "sum i=1..NC coupon_i reinvested plus terminalCP+CM",
        "literal_sum_double_counts_terminal_coupon",
        "-",
    ),
    (
        "iiac13#pdf-index=33;section=10.2;cell=DIC1-definition",
        "days accrual commencement to first actual coupon, alongside additional fullC/f",
        "actual-versus-first-quasi-definition-conflict",
        "-",
    ),
    (
        "boc#pdf-index=0;section=clean-price;cell=formula",
        "(100+C/2)/(1+y*T/365)-C*AD/365",
        "scoped_cash_management_true_maturity_Actual365",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal1000-principal",
        "999.87",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal1000-settlementAI",
        "8.33",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal1000-componentTotal",
        "1008.20",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal10000-principal",
        "9,998.71",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal10000-componentTotal",
        "10,082.00",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal100000-principal",
        "99,987.13",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal100000-settlementAI",
        "832.88",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal100000-componentTotal",
        "100,820,01",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal1000000-principal",
        "999,871.35",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal1000000-settlementAI",
        "8,328.77",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal1000000-componentTotal",
        "1,008,200.12",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal10000000-principal",
        "9,998,713.46",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal10000000-settlementAI",
        "83,287.67",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal10000000-componentTotal",
        "10,082,001.13",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal100000000-principal",
        "99,987,134.59",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal100000000-settlementAI",
        "832,876.71",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal100000000-componentTotal",
        "100,820,011.30",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal1000000000-principal",
        "999,871,345.93",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal1000000000-settlementAI",
        "8,328,767.12",
        "retained_printed_appendix_cell",
        "-",
    ),
    (
        "iiac13#pdf-index=54;section=appendix2;cell=nominal1000000000-componentTotal",
        "1,008,200,113.05",
        "retained_printed_appendix_cell",
        "-",
    ),
];
const PINNED_INPUTS: [(&str, &[u8]); 22] = [
    (
        "adoption-pins",
        include_bytes!("../validation/generator/canadian_source_cash_adopted_pins.json"),
    ),
    (
        "appendix-certificate",
        include_bytes!("../validation/fixtures/bonds/canadian-appendix-declared-price.json"),
    ),
    (
        "component-M8_CANADIAN_API_REVIEW.md",
        include_bytes!("../docs/M8_CANADIAN_API_REVIEW.md"),
    ),
    (
        "component-M8_CANADIAN_CASH_RESOURCE_REVIEW.md",
        include_bytes!("../docs/M8_CANADIAN_CASH_RESOURCE_REVIEW.md"),
    ),
    (
        "component-M8_CANADIAN_COMPOUND_GUARD_DESIGN.md",
        include_bytes!("../docs/M8_CANADIAN_COMPOUND_GUARD_DESIGN.md"),
    ),
    (
        "component-M8_CANADIAN_COMPOUND_PROFILE_REVIEW.md",
        include_bytes!("../docs/M8_CANADIAN_COMPOUND_PROFILE_REVIEW.md"),
    ),
    (
        "component-M8_CANADIAN_CONTRACT_INTEGRATION_REVIEW.md",
        include_bytes!("../docs/M8_CANADIAN_CONTRACT_INTEGRATION_REVIEW.md"),
    ),
    (
        "component-M8_CANADIAN_LOCAL_DESIGN.md",
        include_bytes!("../docs/M8_CANADIAN_LOCAL_DESIGN.md"),
    ),
    (
        "component-M8_CANADIAN_MM_RANGE_REVIEW.md",
        include_bytes!("../docs/M8_CANADIAN_MM_RANGE_REVIEW.md"),
    ),
    (
        "component-M8_CANADIAN_NUMERICAL_DESIGN_REVIEW.md",
        include_bytes!("../docs/M8_CANADIAN_NUMERICAL_DESIGN_REVIEW.md"),
    ),
    (
        "component-M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md",
        include_bytes!("../docs/M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md"),
    ),
    (
        "component-M8_CANADIAN_NUMERIC_RESOURCE_REVIEW.md",
        include_bytes!("../docs/M8_CANADIAN_NUMERIC_RESOURCE_REVIEW.md"),
    ),
    (
        "component-M8_CANADIAN_PREPRODUCTION_PROTOCOL.md",
        include_bytes!("../docs/M8_CANADIAN_PREPRODUCTION_PROTOCOL.md"),
    ),
    (
        "component-M8_CANADIAN_RESEARCH_VALIDATION.md",
        include_bytes!("../docs/M8_CANADIAN_RESEARCH_VALIDATION.md"),
    ),
    (
        "component-M8_CANADIAN_RESOURCE_INTEGRATION_REVIEW.md",
        include_bytes!("../docs/M8_CANADIAN_RESOURCE_INTEGRATION_REVIEW.md"),
    ),
    (
        "component-M8_CANADIAN_SOURCE_REVIEW.md",
        include_bytes!("../docs/M8_CANADIAN_SOURCE_REVIEW.md"),
    ),
    (
        "contract",
        include_bytes!("../docs/M8_CANADIAN_CONTRACT.md"),
    ),
    (
        "generator",
        include_bytes!("../validation/generator/canadian_source_cash_oracles.py"),
    ),
    ("requirements", include_bytes!("../docs/REQUIREMENTS.md")),
    (
        "schema",
        include_bytes!("../validation/generator/canadian_source_cash_schema.json"),
    ),
    (
        "shared-reader",
        include_bytes!("../tests/support/fixture.rs"),
    ),
    (
        "source-bindings",
        include_bytes!("../validation/generator/canadian_source_cash_sources.json"),
    ),
];
#[derive(Clone, Copy)]
struct Row<'a>(Record<'a>);
impl<'a> Row<'a> {
    fn get(self, key: &str) -> TestResult<&'a str> {
        Ok(self.0.text(key)?)
    }
    fn id(self) -> &'a str {
        self.0.id()
    }
    fn number<T: std::str::FromStr>(self, key: &str) -> TestResult<T>
    where
        T::Err: Error + 'static,
    {
        Ok(self.get(key)?.parse()?)
    }
}
type Outputs = BTreeMap<&'static str, String>;
fn empty() -> Outputs {
    let mut out: Outputs = OUTPUTS
        .map(|key| (key, "-".to_owned()))
        .into_iter()
        .collect();
    out.insert("expected", "ok".to_owned());
    out
}
fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
fn json_map(values: &BTreeMap<&str, String>) -> String {
    format!(
        "{{{}}}",
        values
            .iter()
            .map(|(key, value)| format!("{}:{}", json_string(key), json_string(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn frequency(text: &str) -> TestResult<CanadianFrequency> {
    Ok(match text {
        "2" => CanadianFrequency::Semiannual,
        "4" => CanadianFrequency::Quarterly,
        "12" => CanadianFrequency::Monthly,
        _ => return Err("frequency".into()),
    })
}
fn selection(text: &str) -> TestResult<CanadianYieldSelection> {
    Ok(match text {
        "Explicit:Compound" => CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
        "Explicit:MoneyMarket" => {
            CanadianYieldSelection::Explicit(CanadianYieldConvention::MoneyMarket)
        }
        "LastCouponMoneyMarket" => CanadianYieldSelection::LastCouponMoneyMarket,
        _ => return Err("selection".into()),
    })
}
fn hash<T: Hash>(value: &T) -> u64 {
    let mut hash = DefaultHasher::new();
    value.hash(&mut hash);
    hash.finish()
}
fn option<T: ToString>(value: Option<T>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}
fn pair(value: Option<(Date, Date)>) -> String {
    value.map_or_else(|| "none".to_owned(), |(a, b)| format!("{a}:{b}"))
}
fn context(mut values: Vec<(&str, String)>) -> String {
    values.sort_unstable();
    values
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(";")
}
fn render(error: &CanadianError) -> TestResult<String> {
    let values = match error {
        CanadianError::InvalidDateCap {
            cap,
            minimum,
            maximum,
            ..
        } => vec![
            ("cap", cap.to_string()),
            ("minimum", minimum.to_string()),
            ("maximum", maximum.to_string()),
        ],
        CanadianError::TooManyQuasiDates { count, cap, .. }
        | CanadianError::TooManyCoupons { count, cap, .. }
        | CanadianError::TooManyBundles { count, cap, .. } => {
            vec![("count", count.to_string()), ("cap", cap.to_string())]
        }
        CanadianError::TooFewQuasiDates { count, minimum, .. } => vec![
            ("count", count.to_string()),
            ("minimum", minimum.to_string()),
        ],
        CanadianError::NoCoupons => vec![],
        CanadianError::AccrualNotBeforeMaturity {
            accrual_start,
            stated_maturity,
            ..
        } => vec![
            ("accrual_start", accrual_start.to_string()),
            ("stated_maturity", stated_maturity.to_string()),
        ],
        CanadianError::QuasiDateOrder {
            index,
            previous,
            date,
            ..
        }
        | CanadianError::CouponDateOrder {
            index,
            previous,
            date,
            ..
        }
        | CanadianError::IrregularInteriorPeriod {
            index,
            previous,
            date,
            ..
        } => vec![
            ("index", index.to_string()),
            ("previous", previous.to_string()),
            ("date", date.to_string()),
        ],
        CanadianError::QuasiCoverage {
            accrual_start,
            stated_maturity,
            first,
            next,
            penultimate,
            last,
            ..
        } => vec![
            ("accrual_start", accrual_start.to_string()),
            ("stated_maturity", stated_maturity.to_string()),
            ("first", first.to_string()),
            ("next", next.to_string()),
            ("penultimate", penultimate.to_string()),
            ("last", last.to_string()),
        ],
        CanadianError::QuasiDateMismatch {
            index,
            supplied,
            expected,
            anchor,
            frequency,
            end_of_month,
            ..
        } => vec![
            ("index", index.to_string()),
            ("supplied", supplied.to_string()),
            ("expected", expected.to_string()),
            ("anchor", anchor.to_string()),
            ("frequency", frequency.payments_per_year().to_string()),
            ("end_of_month", hex(&end_of_month.to_string())),
        ],
        CanadianError::CouponBoundary {
            last_coupon,
            stated_maturity,
            ..
        } => vec![
            ("last_coupon", last_coupon.to_string()),
            ("stated_maturity", stated_maturity.to_string()),
        ],
        CanadianError::CouponBeforeAccrual {
            index,
            date,
            accrual_start,
            ..
        } => vec![
            ("index", index.to_string()),
            ("date", date.to_string()),
            ("accrual_start", accrual_start.to_string()),
        ],
        CanadianError::CouponOffCycle { index, date, .. } => {
            vec![("index", index.to_string()), ("date", date.to_string())]
        }
        CanadianError::Date {
            index,
            phase,
            cause,
            ..
        } => {
            let mut values = vec![
                (
                    "index",
                    index.map_or_else(|| "none".to_owned(), |i| i.to_string()),
                ),
                ("phase", hex(phase)),
            ];
            match cause {
                DateError::YearOutOfRange { year, .. } => values.extend([
                    ("cause_variant", hex("YearOutOfRange")),
                    ("cause_year", year.to_string()),
                ]),
                other => return Err(format!("unlisted Date cause {other:?}").into()),
            };
            values
        }
        CanadianError::NegativeCoupon { coupon_per_100, .. } => {
            vec![("coupon_per_100", hex(&cell(*coupon_per_100)))]
        }
        CanadianError::DatedAfterIssue {
            accrual_start,
            original_issue,
            ..
        } => vec![
            ("accrual_start", accrual_start.to_string()),
            ("original_issue", original_issue.to_string()),
        ],
        CanadianError::IssueNotBeforeFirstCoupon {
            original_issue,
            first_coupon,
            ..
        } => vec![
            ("original_issue", original_issue.to_string()),
            ("first_coupon", first_coupon.to_string()),
        ],
        CanadianError::TrueMaturityBeforeStated {
            true_maturity,
            stated_maturity,
            ..
        } => vec![
            ("true_maturity", true_maturity.to_string()),
            ("stated_maturity", stated_maturity.to_string()),
        ],
        CanadianError::InvalidBundleCap { cap, maximum, .. } => {
            vec![("cap", cap.to_string()), ("maximum", maximum.to_string())]
        }
        CanadianError::SettlementBeforeIssue {
            settlement,
            original_issue,
            ..
        } => vec![
            ("settlement", settlement.to_string()),
            ("original_issue", original_issue.to_string()),
        ],
        CanadianError::SettlementAtOrAfterMaturity {
            settlement,
            stated_maturity,
            ..
        } => vec![
            ("settlement", settlement.to_string()),
            ("stated_maturity", stated_maturity.to_string()),
        ],
        CanadianError::InvalidCouponIndex { index, count, .. } => {
            vec![("index", index.to_string()), ("count", count.to_string())]
        }
        CanadianError::NonPositiveNominal { nominal, .. } => {
            vec![("nominal", hex(&cell(*nominal)))]
        }
        CanadianError::CashArithmetic {
            stage,
            endpoint,
            cause,
            ..
        } => {
            let mut values = vec![("stage", hex(stage)), ("endpoint", hex(endpoint))];
            match cause {
                AmountError::Inexact { operation, .. } => values.extend([
                    ("cause_variant", hex("Inexact")),
                    ("cause_operation", hex(operation)),
                ]),
                AmountError::OutOfRange {
                    operation,
                    negative,
                    ..
                } => values.extend([
                    ("cause_variant", hex("OutOfRange")),
                    ("cause_operation", hex(operation)),
                    ("cause_negative", negative.to_string()),
                ]),
                other => return Err(format!("unlisted cash cause {other:?}").into()),
            };
            values
        }
        other => return Err(format!("unexpected Canadian failure in S scope {other:?}").into()),
    };
    Ok(context(values))
}
fn error_output(error: &CanadianError) -> TestResult<Outputs> {
    let mut out = empty();
    out.insert("expected", error.kind().to_owned());
    out.insert("context", render(error)?);
    assert!(!error.to_string().is_empty());
    match error {
        CanadianError::Date { cause, .. } => assert!(
            error
                .source()
                .and_then(|x| x.downcast_ref::<DateError>())
                .is_some_and(|x| std::ptr::eq(x, cause))
        ),
        CanadianError::CashArithmetic { cause, .. } => assert!(
            error
                .source()
                .and_then(|x| x.downcast_ref::<AmountError>())
                .is_some_and(|x| std::ptr::eq(x, cause))
        ),
        _ => assert!(error.source().is_none()),
    }
    let root = RootError::from(error.clone());
    assert_eq!(root.kind(), error.kind());
    assert_eq!(root.to_string(), error.to_string());
    assert_eq!(
        root.source()
            .and_then(|x| x.downcast_ref::<CanadianError>()),
        Some(error)
    );
    Ok(out)
}
fn projection(
    id: &str,
    rational: &str,
    stage: &str,
    mut view: impl FnMut(AmountQuantum, AmountRounding) -> Result<Option<DecimalAmount>, CanadianError>,
) -> TestResult {
    let expected = Rat::parse(rational)?;
    for scale in [0, 38] {
        for mode in [AmountRounding::Exact, AmountRounding::NearestTiesEven] {
            match (
                expected.rounded(scale, mode)?,
                view(AmountQuantum::new(1, scale)?, mode),
            ) {
                (Ok(expected), Ok(Some(actual))) => assert_eq!(
                    actual, expected,
                    "{id}: {stage} sourceprojection scale{scale} {mode:?}"
                ),
                (
                    Err((kind, negative)),
                    Err(CanadianError::CashArithmetic {
                        stage: actual_stage,
                        endpoint: "point",
                        cause,
                        ..
                    }),
                ) => {
                    assert_eq!(actual_stage, stage, "{id}");
                    assert_eq!(cause.kind(), kind, "{id}");
                    match cause {
                        AmountError::Inexact { operation, .. } => assert_eq!(operation, "quantize"),
                        AmountError::OutOfRange {
                            operation,
                            negative: actual,
                            ..
                        } => {
                            assert_eq!(operation, "quantize");
                            assert_eq!(actual, negative)
                        }
                        other => return Err(format!("{id}: projection cause{other:?}").into()),
                    }
                }
                (expected, actual) => return Err(format!(
                    "{id}: {stage} projection {rational}, expected{expected:?}, actual{actual:?}"
                )
                .into()),
            }
        }
    }
    Ok(())
}
fn build_schedule(r: Row<'_>) -> TestResult<Result<CanadianCouponSchedule, CanadianError>> {
    Ok(CanadianCouponSchedule::new(
        date(r.get("dated")?)?,
        date(r.get("stated_maturity")?)?,
        frequency(r.get("frequency")?)?,
        dates(r.get("quasi")?)?,
        dates(r.get("coupon_dates")?)?,
        r.get("eom")?.parse()?,
        r.number("date_cap")?,
    ))
}
fn build_bond(
    r: Row<'_>,
    schedule: CanadianCouponSchedule,
) -> TestResult<Result<CanadianBond, CanadianError>> {
    assert_eq!(r.get("source_policy")?, "CanonicalIiac13LocalV1");
    Ok(CanadianBond::new(
        date(r.get("issue")?)?,
        amount(r.get("coupon")?)?,
        schedule,
        date(r.get("true_maturity")?)?,
        CanadianSourcePolicy::CanonicalIiac13LocalV1,
    ))
}
fn build_plan<'a>(
    r: Row<'_>,
    bond: &'a CanadianBond,
) -> TestResult<Result<CanadianPricingPlan<'a>, CanadianError>> {
    Ok(CanadianPricingPlan::new(
        bond,
        date(r.get("settlement")?)?,
        selection(r.get("selection")?)?,
        r.number("bundle_cap")?,
    ))
}
fn paint_schedule(out: &mut Outputs, s: &CanadianCouponSchedule, r: Row<'_>) -> TestResult {
    assert_eq!(s.accrual_start(), date(r.get("dated")?)?);
    assert_eq!(s.stated_maturity(), date(r.get("stated_maturity")?)?);
    assert_eq!(s.frequency(), frequency(r.get("frequency")?)?);
    assert_eq!(s.end_of_month(), r.get("eom")?.parse::<EndOfMonth>()?);
    assert_eq!(s.quasi_dates(), dates(r.get("quasi")?)?);
    assert_eq!(s.coupon_dates(), dates(r.get("coupon_dates")?)?);
    for (key, value) in [
        ("first_period", format!("{:?}", s.first_period())),
        ("last_period", format!("{:?}", s.last_period())),
        ("first_coupon", s.first_coupon_date().to_string()),
        ("last_coupon_start", s.last_coupon_start_date().to_string()),
        ("quasi_count", s.quasi_dates().len().to_string()),
        ("coupon_count", s.coupon_count().to_string()),
        ("anchor", s.direct_cycle_anchor().to_string()),
    ] {
        out.insert(key, value);
    }
    assert_eq!(s.coupon_start_date(s.coupon_count()), None);
    assert_eq!(s.coupon_period_kind(s.coupon_count()), None);
    Ok(())
}
fn paint_bond(out: &mut Outputs, bond: &CanadianBond, r: Row<'_>) -> TestResult {
    assert_eq!(bond.source_version(), SOURCE);
    assert_eq!(bond.original_issue_date(), date(r.get("issue")?)?);
    assert_eq!(bond.true_maturity_date(), date(r.get("true_maturity")?)?);
    assert_eq!(bond.coupon_per_100(), amount(r.get("coupon")?)?);
    assert_eq!(
        bond.source_policy(),
        CanadianSourcePolicy::CanonicalIiac13LocalV1
    );
    out.insert("source_version", bond.source_version().to_owned());
    for (column, stage, valuation) in [
        ("valuation_coupons", "source valuation coupon", true),
        ("payable_coupons", "source payable coupon", false),
    ] {
        if r.get(column)? != "-" {
            let ratios = r.get(column)?.split(',').collect::<Vec<_>>();
            assert_eq!(
                ratios.len(),
                bond.schedule().coupon_count(),
                "{} {column}",
                r.id()
            );
            for (index, ratio) in ratios.iter().enumerate() {
                projection(r.id(), ratio, stage, |q, m| {
                    if valuation {
                        bond.valuation_coupon_per_100(index, q, m).map(Some)
                    } else {
                        bond.payable_coupon_per_100(index, q, m).map(Some)
                    }
                })?;
            }
            out.insert(column, r.get(column)?.to_owned());
        }
    }
    Ok(())
}
fn coordinate(q: &[Date], time: Date) -> TestResult<(u128, u128)> {
    if Some(&time) == q.last() {
        return Ok((u128::try_from(q.len() - 1)?, 1));
    }
    let index = q
        .partition_point(|x| *x <= time)
        .checked_sub(1)
        .ok_or("coordinate lower")?;
    let (a, b) = (
        *q.get(index).ok_or("coordinatea")?,
        *q.get(index + 1).ok_or("coordinateb")?,
    );
    let den = u128::from(b.days_since(a).unsigned_abs());
    let num = u128::try_from(index)? * den + u128::from(time.days_since(a).unsigned_abs());
    Ok((num, den))
}
fn lambda(q: &[Date], start: Date, end: Date) -> TestResult<String> {
    let (a, b) = coordinate(q, start)?;
    let (c, d) = coordinate(q, end)?;
    let mut n = c * b - a * d;
    let mut den = b * d;
    let (mut x, mut y) = (n, den);
    while y != 0 {
        let rem = x % y;
        x = y;
        y = rem;
    }
    n /= x;
    den /= x;
    Ok(format!("{n}/{den}"))
}
fn paint_plan(out: &mut Outputs, plan: &CanadianPricingPlan<'_>, r: Row<'_>) -> TestResult {
    let bond = plan.bond();
    assert!(std::ptr::eq(bond, plan.bond()));
    assert_eq!(plan.source_version(), SOURCE);
    assert_eq!(plan.settlement_date(), date(r.get("settlement")?)?);
    assert_eq!(plan.yield_selection(), selection(r.get("selection")?)?);
    let next = plan.bundle_coupon_index(0).ok_or("empty admittedplan")?;
    let coupons = bond.schedule().coupon_dates();
    let next_date = *coupons.get(next).ok_or("nextoriginal")?;
    let ai_instruction = plan.accrued_cash(
        DecimalAmount::new(1, 0)?,
        SettlementRounding::new(AmountQuantum::new(1, 0)?, AmountRounding::Exact),
    )?;
    for (key, value) in [
        ("next_coupon_index", next.to_string()),
        ("next_coupon_date", next_date.to_string()),
        (
            "next_payment_date",
            bond.coupon_payment_date(next)
                .ok_or("nextpayment")?
                .to_string(),
        ),
        (
            "actual_accrual_start",
            ai_instruction
                .source_interval()
                .ok_or("AIinterval")?
                .0
                .to_string(),
        ),
        (
            "valuation_ai_bits",
            format!("{:016x}", plan.valuation_accrued_per_100().to_bits()),
        ),
        (
            "settlement_ai_bits",
            format!("{:016x}", plan.settlement_accrued_per_100().to_bits()),
        ),
        (
            "quote_accrued_bits",
            format!("{:016x}", plan.quote_accrued_per_100().to_bits()),
        ),
        (
            "resolved_convention",
            format!("{:?}", plan.yield_convention()),
        ),
    ] {
        out.insert(key, value);
    }
    for (column, observed, stage, valuation) in [
        (
            "valuation_ai",
            plan.valuation_accrued_per_100(),
            "source valuation accrued",
            true,
        ),
        (
            "settlement_ai",
            plan.settlement_accrued_per_100(),
            "source settlement accrued",
            false,
        ),
    ] {
        if r.get(column)? != "-" {
            assert_eq!(
                Rat::parse(r.get(column)?)?.binary_round()?.to_bits(),
                observed.to_bits(),
                "{} {column} sourceonce",
                r.id()
            );
            projection(r.id(), r.get(column)?, stage, |q, m| {
                if valuation {
                    plan.valuation_accrued_at_quantum(q, m).map(Some)
                } else {
                    plan.settlement_accrued_at_quantum(q, m).map(Some)
                }
            })?;
            out.insert(column, r.get(column)?.to_owned());
        }
    }
    let ordinals = (0..plan.bundle_count())
        .map(|i| plan.bundle_coupon_index(i).ok_or("ordinal"))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(plan.bundle_coupon_index(plan.bundle_count()), None);
    out.insert(
        "bundle_ordinals",
        ordinals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(","),
    );
    let mut entitlements = Vec::new();
    for (i, d) in coupons.iter().enumerate() {
        let e = plan.coupon_entitlement(i).ok_or("entitlement")?;
        assert_eq!(
            e,
            if *d <= plan.settlement_date() {
                CouponEntitlement::Ceased
            } else {
                CouponEntitlement::BuyerEntitled
            },
            "{} originalordinal{i}",
            r.id()
        );
        entitlements.push(format!("{e:?}"));
    }
    assert_eq!(plan.coupon_entitlement(coupons.len()), None);
    out.insert("entitlements", entitlements.join(","));
    let mut lambdas = Vec::new();
    let mut h = Vec::new();
    let mut rd = Vec::new();
    for &i in &ordinals {
        let contractual = *coupons.get(i).ok_or("bundlecontractual")?;
        let payment = bond.coupon_payment_date(i).ok_or("bundlepayment")?;
        lambdas.push(lambda(
            bond.schedule().quasi_dates(),
            plan.settlement_date(),
            contractual,
        )?);
        h.push(payment.days_since(plan.settlement_date()).to_string());
        rd.push(bond.true_maturity_date().days_since(payment).to_string());
    }
    out.insert("bundle_lambdas", lambdas.join(","));
    out.insert("bundle_h_days", h.join(","));
    out.insert("bundle_r_days", rd.join(","));
    if r.get("bundle_coefficients")? != "-" {
        let coefficients = r.get("bundle_coefficients")?.split(',').collect::<Vec<_>>();
        assert_eq!(coefficients.len(), ordinals.len());
        let selected = r
            .get(
                if plan.yield_convention() == CanadianYieldConvention::Compound {
                    "valuation_coupons"
                } else {
                    "payable_coupons"
                },
            )?
            .split(',')
            .collect::<Vec<_>>();
        let mut zero_sum = Rat::parse("0/1")?;
        for (&i, coefficient) in ordinals.iter().zip(&coefficients) {
            let difference = Rat::parse(coefficient)?.sub(&Rat::parse(selected[i])?);
            assert!(
                if i + 1 == coupons.len() {
                    difference.sub(&Rat::parse("100/1")?).is_zero()
                } else {
                    difference.is_zero()
                },
                "{} selectedcoefficient{i}",
                r.id()
            );
            let observed = Rat::parse(coefficient)?.binary_round()?;
            zero_sum = zero_sum.sub(&Rat::binary(-observed));
        }
        assert_eq!(
            plan.evaluate(0.0)?.model_dirty_per_100().to_bits(),
            zero_sum.binary_round()?.to_bits(),
            "{} observedzero dirty selectedsources",
            r.id()
        );
        out.insert(
            "bundle_coefficients",
            r.get("bundle_coefficients")?.to_owned(),
        );
    }
    Ok(())
}
fn make_instruction(
    r: Row<'_>,
    bond: &CanadianBond,
    plan: Option<&CanadianPricingPlan<'_>>,
) -> TestResult<Result<CanadianCashInstruction, CanadianError>> {
    let nominal = amount(r.get("nominal")?)?;
    let rounding = SettlementRounding::new(quantum(r.get("quantum")?)?, r.get("mode")?.parse()?);
    let prepared = || plan.ok_or("cashwithoutplan");
    Ok(match r.get("op")?.rsplit('_').next().ok_or("recipe")? {
        "coupon" => bond.coupon_cash(r.number("coupon_index")?, nominal, rounding),
        "accrued" => prepared()?.accrued_cash(nominal, rounding),
        "dirty" => prepared()?.dirty_cash(nominal, evidence(r.get("evidence")?)?, rounding),
        "invoice" => prepared()?.clean_accrued_cash(
            nominal,
            evidence(r.get("evidence")?)?,
            r.get("aggregation")?.parse()?,
            rounding,
        ),
        _ => return Err("recipe".into()),
    })
}
fn paint_instruction(
    out: &mut Outputs,
    i: &CanadianCashInstruction,
    bond: &CanadianBond,
    plan: Option<&CanadianPricingPlan<'_>>,
    r: Row<'_>,
) -> TestResult {
    let s = bond.schedule();
    assert_eq!(i.source_version(), SOURCE);
    assert_eq!(i.source_policy(), bond.source_policy());
    assert_eq!(i.nominal(), amount(r.get("nominal")?)?);
    assert_eq!(
        i.cash_rounding(),
        SettlementRounding::new(quantum(r.get("quantum")?)?, r.get("mode")?.parse()?)
    );
    assert_eq!(i.coupon_per_100(), bond.coupon_per_100());
    assert_eq!(i.frequency(), s.frequency());
    assert_eq!(i.accrual_start(), s.accrual_start());
    assert_eq!(i.original_issue_date(), bond.original_issue_date());
    assert_eq!(i.stated_maturity_date(), s.stated_maturity());
    assert_eq!(i.true_maturity_date(), bond.true_maturity_date());
    assert_eq!(i.first_coupon_date(), bond.first_coupon_date());
    assert_eq!(i.last_coupon_start_date(), bond.last_coupon_start_date());
    assert_eq!(i.first_period(), s.first_period());
    assert_eq!(i.last_period(), s.last_period());
    assert_eq!(i.end_of_month(), s.end_of_month());
    assert_eq!(i.direct_cycle_anchor(), s.direct_cycle_anchor());
    assert_eq!(
        i.price_evidence(),
        if r.get("evidence")? == "-" {
            None
        } else {
            Some(evidence(r.get("evidence")?)?)
        }
    );
    assert_eq!(
        i.aggregation(),
        if r.get("aggregation")? == "-" {
            None
        } else {
            Some(r.get("aggregation")?.parse::<CashAggregation>()?)
        }
    );
    assert_eq!(
        i.settlement_date(),
        plan.map(CanadianPricingPlan::settlement_date)
    );
    assert_eq!(
        i.yield_selection(),
        plan.map(CanadianPricingPlan::yield_selection)
    );
    assert_eq!(
        i.yield_convention(),
        plan.map(CanadianPricingPlan::yield_convention)
    );
    out.insert("recipe_identifier", i.recipe_identifier().to_owned());
    out.insert("src_index", option(i.source_coupon_index()));
    out.insert("src_date", option(i.source_coupon_date()));
    out.insert("src_payment", option(i.source_payment_date()));
    out.insert("src_interval", pair(i.source_interval()));
    out.insert(
        "src_quasi_pairs",
        i.source_quasi_endpoints().map_or_else(
            || "none".to_owned(),
            |(a, b)| format!("{}:{},{}:{}", a.0, a.1, b.0, b.1),
        ),
    );
    out.insert("src_full_count", option(i.source_full_period_count()));
    out.insert(
        "src_day_ratios",
        i.source_day_ratios().map_or_else(
            || "none".to_owned(),
            |(a, b)| {
                b.map_or_else(
                    || format!("{}/{}", a.0, a.1),
                    |b| format!("{}/{},{}/{}", a.0, a.1, b.0, b.1),
                )
            },
        ),
    );
    if r.get("source_value")? == "none" {
        assert_eq!(
            i.source_value_per_100(AmountQuantum::new(1, 0)?, AmountRounding::Exact)?,
            None
        );
        out.insert("source_value", "none".to_owned());
    } else if r.get("source_value")? != "-" {
        let stage = if i.recipe_identifier() == "PayableCouponPer100" {
            "source payable coupon"
        } else {
            "source settlement accrued"
        };
        projection(r.id(), r.get("source_value")?, stage, |q, m| {
            i.source_value_per_100(q, m)
        })?;
        out.insert("source_value", r.get("source_value")?.to_owned());
    }
    Ok(())
}
fn instruction_key(i: &CanadianCashInstruction, source: &str) -> String {
    format!(
        "{:?}|{:?}|{:?}|{}",
        (
            i.source_policy(),
            i.recipe_identifier(),
            i.nominal(),
            i.cash_rounding(),
            i.price_evidence(),
            i.aggregation(),
            i.coupon_per_100(),
            i.frequency(),
            i.end_of_month()
        ),
        (
            i.accrual_start(),
            i.original_issue_date(),
            i.stated_maturity_date(),
            i.true_maturity_date(),
            i.first_coupon_date(),
            i.last_coupon_start_date(),
            i.first_period(),
            i.last_period(),
            i.direct_cycle_anchor()
        ),
        (
            i.settlement_date(),
            i.yield_selection(),
            i.yield_convention(),
            i.source_coupon_index(),
            i.source_coupon_date(),
            i.source_payment_date(),
            i.source_interval(),
            i.source_quasi_endpoints(),
            i.source_full_period_count(),
            i.source_day_ratios()
        ),
        source
    )
}
fn execute(
    r: Row<'_>,
    definitions: &mut BTreeMap<String, CanadianCashInstruction>,
    identities: &mut HashMap<CanadianCashInstruction, String>,
) -> TestResult<Result<Outputs, CanadianError>> {
    let mut out = empty();
    let schedule = match build_schedule(r)? {
        Ok(value) => value,
        Err(error) => return Ok(Err(error)),
    };
    paint_schedule(&mut out, &schedule, r)?;
    if r.get("op")? == "schedule" {
        return Ok(Ok(out));
    }
    let bond = match build_bond(r, schedule)? {
        Ok(value) => value,
        Err(error) => return Ok(Err(error)),
    };
    paint_bond(&mut out, &bond, r)?;
    if r.get("op")? == "instrument" {
        return Ok(Ok(out));
    }
    let plan = if r.get("settlement")? == "-" {
        None
    } else {
        match build_plan(r, &bond)? {
            Ok(value) => Some(value),
            Err(error) => return Ok(Err(error)),
        }
    };
    if let Some(plan) = &plan {
        paint_plan(&mut out, plan, r)?;
    }
    match r.get("op")? {
        "plan" => {}
        "source_view_coupon" | "source_view_accrued" => {
            let q = quantum(r.get("quantum")?)?;
            let m = r.get("mode")?.parse()?;
            let value = match r.get("source_item")? {
                "valuation_coupon" => {
                    bond.valuation_coupon_per_100(r.number("coupon_index")?, q, m)
                }
                "payable_coupon" => bond.payable_coupon_per_100(r.number("coupon_index")?, q, m),
                "valuation_accrued" => plan
                    .as_ref()
                    .ok_or("sourceplan")?
                    .valuation_accrued_at_quantum(q, m),
                "settlement_accrued" => plan
                    .as_ref()
                    .ok_or("sourceplan")?
                    .settlement_accrued_at_quantum(q, m),
                _ => return Err("source_item".into()),
            };
            match value {
                Ok(value) => {
                    out.insert("expected", "amount".to_owned());
                    out.insert("view_amount", cell(value));
                    out.insert("source_value", r.get("source_value")?.to_owned());
                }
                Err(error) => return Ok(Err(error)),
            }
        }
        op if op.starts_with("cash_") || op.starts_with("instruction_view_") => {
            let i = match make_instruction(r, &bond, plan.as_ref())? {
                Ok(value) => value,
                Err(error) => return Ok(Err(error)),
            };
            paint_instruction(&mut out, &i, &bond, plan.as_ref(), r)?;
            if r.get("source_value")? != "-" {
                let key = instruction_key(&i, r.get("source_value")?);
                if let Some(previous) = definitions.insert(key.clone(), i) {
                    assert_eq!(previous, i, "{} reconstructibleidentity", r.id());
                    assert_eq!(hash(&previous), hash(&i));
                }
                if let Some(previous) = identities.insert(i, key.clone()) {
                    assert_eq!(previous, key, "{} provenancecollapsed", r.id());
                }
            }
            if op.starts_with("instruction_view_") {
                match i.source_value_per_100(
                    quantum(r.get("view_quantum")?)?,
                    r.get("view_mode")?.parse()?,
                ) {
                    Ok(Some(value)) => {
                        out.insert("expected", "some".to_owned());
                        out.insert("view_amount", cell(value));
                    }
                    Ok(None) => {
                        out.insert("expected", "none".to_owned());
                        out.insert("view_amount", "none".to_owned());
                    }
                    Err(error) => return Ok(Err(error)),
                }
            } else {
                match i.evaluate() {
                    Ok(CanadianCashDecision::Determined(receipt)) => {
                        assert_eq!(receipt.instruction(), &i);
                        out.insert("expected", "determined".to_owned());
                        out.insert("lower_amount", cell(receipt.amount()));
                        out.insert("upper_amount", cell(receipt.amount()));
                        let guarantee = receipt.price_evidence_guarantee();
                        let expected = i.price_evidence().map(|e| {
                            if e.is_declared_interval() {
                                SettlementGuarantee::UniqueForDeclaredEnclosure
                            } else if e.decimal_bounds().is_some() {
                                SettlementGuarantee::ForSuppliedDecimals
                            } else {
                                SettlementGuarantee::ForSuppliedBinary64
                            }
                        });
                        assert_eq!(guarantee, expected);
                        out.insert(
                            "guarantee",
                            guarantee.map_or_else(|| "none".to_owned(), |g| format!("{g:?}")),
                        );
                    }
                    Ok(CanadianCashDecision::QuantumAmbiguous {
                        lower_amount,
                        upper_amount,
                        ..
                    }) => {
                        out.insert("expected", "ambiguous".to_owned());
                        out.insert("lower_amount", cell(lower_amount));
                        out.insert("upper_amount", cell(upper_amount));
                    }
                    Ok(CanadianCashDecision::ExactStageUnresolved { stage, .. }) => {
                        out.insert("expected", "exact_unresolved".to_owned());
                        out.insert("context", context(vec![("stage", hex(stage))]));
                    }
                    Err(error) => return Ok(Err(error)),
                    Ok(other) => return Err(format!("unlisteddecision{other:?}").into()),
                }
            }
        }
        other => return Err(format!("unknown publicoperation{other}").into()),
    }
    Ok(Ok(out))
}
fn documentary(r: Row<'_>) -> TestResult<Outputs> {
    let entry = DOCUMENTARY
        .iter()
        .find(|entry| entry.0 == r.get("reference").unwrap_or("invalid"))
        .ok_or("unknown fixed primary locator")?;
    let mut out = empty();
    out.insert("expected", "source_observation".to_owned());
    out.insert("source_observation", entry.1.to_owned());
    out.insert("reference_classification", entry.2.to_owned());
    out.insert("source_value", entry.3.to_owned());
    Ok(out)
}
/// Every physical column is classified without treating a mathematical
/// ingredient as an unavailable public ratio/coordinate inspector.
fn column_role(key: &str) -> TestResult<&'static str> {
    if key == "id" {
        return Ok("arbitrary_row_id");
    }
    if INPUTS.contains(&key) {
        return Ok("operation_specific_actual_input");
    }
    if ANNOTATIONS.contains(&key) {
        return Ok("annotation_not_a_financial_result");
    }
    Ok(match key {
        "valuation_coupons" | "payable_coupons" | "valuation_ai" | "settlement_ai" => {
            "independent_rational_public_quantum_projection"
        }
        "source_value" => "selected_public_quantum_projection_or_fixed_documentary_arithmetic",
        "bundle_coefficients" => {
            "selected_source_plus_terminal_par_integrity_and_public_observed_y0_dirty"
        }
        "bundle_lambdas" => "exact_dated_reference_integrity_actual_private_use_belongs_N_P_Y",
        "bundle_h_days" | "bundle_r_days" => {
            "public_payment_date_horizon_projection_actual_private_use_belongs_N"
        }
        "reference_classification" | "source_observation" => {
            "fixed_documentary_binding_not_financial_Rust_runtime"
        }
        key if OUTPUTS.contains(&key) => "direct_public_value_metadata_decision_or_typed_context",
        _ => return Err(format!("unclassified physicalcolumn{key}").into()),
    })
}
#[expect(
    clippy::disallowed_methods,
    reason = "Integration tests may read frozen reference evidence."
)]
fn load() -> TestResult<Fixture> {
    let raw = std::fs::read_to_string(Fixture::repository_path(PATH))?;
    assert!(raw.ends_with('\n') && !raw.contains('\r'));
    assert_eq!(
        sha256(raw.as_bytes())?,
        "e59aa8a9097bdeb39e80e08511d658808233049cfa3aafc45e6b109d58e61c1d"
    );
    let fixture = Fixture::parse(&raw)?;
    assert_eq!(fixture.record_count(), 79_794);
    assert_eq!(fixture.columns(), COLUMNS.map(str::to_owned).as_slice());
    assert_eq!(fixture.header("source-version"), Some(SOURCE));
    assert_eq!(
        fixture.header("adopted-contract-sha256"),
        Some("4fb02d8e185ba3b2482934a81b4291d0aeb122f99a9e353e71cc2e977de5b8b3")
    );
    let body = raw.split_once("\n\n").ok_or("body")?.1;
    assert_eq!(
        fixture.header("records-sha256"),
        Some(sha256(body.as_bytes())?.as_str())
    );
    assert_eq!(
        fixture.header("records-fnv64"),
        Some(format!("{:016x}", fnv64(body.as_bytes())).as_str())
    );
    let inputs = PINNED_INPUTS
        .iter()
        .map(|(name, bytes)| {
            Ok((
                name,
                format!(
                    "{{\"bytes\":{},\"fnv64\":\"{:016x}\",\"sha256\":\"{}\"}}",
                    bytes.len(),
                    fnv64(bytes),
                    sha256(bytes)?
                ),
            ))
        })
        .collect::<TestResult<BTreeMap<_, _>>>()?;
    let inputs = format!(
        "{{{}}}",
        inputs
            .iter()
            .map(|(key, value)| format!("{}:{value}", json_string(key)))
            .collect::<Vec<_>>()
            .join(",")
    );
    assert_eq!(fixture.header("inputs"), Some(inputs.as_str()));
    let req = include_str!("../docs/REQUIREMENTS.md")
        .lines()
        .filter(|line| line.starts_with("|FR-CAD-"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(req.lines().count(), 72);
    assert_eq!(
        fixture.header("requirements-rows-sha256"),
        Some(sha256(req.as_bytes())?.as_str())
    );
    assert_eq!(
        fixture.header("requirements-rows-fnv64"),
        Some(format!("{:016x}", fnv64(req.as_bytes())).as_str())
    );
    let unique = COLUMNS.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), 74);
    for key in COLUMNS {
        assert!(!column_role(key)?.is_empty());
    }
    assert_eq!(INPUTS.len() + OUTPUTS.len() + ANNOTATIONS.len() + 1, 74);
    Ok(fixture)
}
/// FR-CAD-01..32/60..71: actual thirteen public operation routes for every
/// public record, plus36 separately labelled fixed documentary observations.
#[test]
fn all_source_cash_operations_outputs_identity_and_censuses() -> TestResult {
    let fixture = load()?;
    let mut profiles = BTreeMap::new();
    let mut operations = BTreeMap::new();
    let mut scopes = BTreeMap::new();
    let mut expectations = BTreeMap::new();
    let mut input_identities = BTreeSet::new();
    let mut canonical_inputs = String::new();
    let mut financial = BTreeSet::new();
    let mut schedules = BTreeSet::new();
    let mut all_definitions = BTreeSet::new();
    let mut identities = HashMap::new();
    let mut definitions = BTreeMap::new();
    let mut public = 0;
    let mut documentary_count = 0;
    for (position, record) in fixture.records().enumerate() {
        let r = Row(record);
        assert_eq!(r.id(), format!("CA{:06}", position + 1));
        let op = r.get("op")?;
        let projection = input_projection(op)?;
        for key in INPUTS {
            assert_eq!(
                r.get(key)? == "-",
                !projection.contains(&key),
                "{} ignored/missingoperand{key}",
                r.id()
            );
        }
        let canonical = json_map(
            &projection
                .iter()
                .map(|key| Ok((*key, r.get(key)?.to_owned())))
                .collect::<TestResult<BTreeMap<_, _>>>()?,
        );
        assert!(
            input_identities.insert(canonical.clone()),
            "{} duplicateactualinput",
            r.id()
        );
        canonical_inputs.push_str(&canonical);
        canonical_inputs.push('\n');
        let definition = json_map(
            &[
                "dated",
                "issue",
                "stated_maturity",
                "true_maturity",
                "frequency",
                "eom",
                "quasi",
                "coupon_dates",
                "coupon",
                "source_policy",
            ]
            .into_iter()
            .filter_map(|key| match r.get(key) {
                Ok("-") => None,
                Ok(value) => Some(Ok((key, value.to_owned()))),
                Err(error) => Some(Err(error)),
            })
            .collect::<TestResult<BTreeMap<_, _>>>()?,
        );
        let digest = if definition == "{}" {
            "-".to_owned()
        } else {
            sha256(definition.as_bytes())?
        };
        assert_eq!(
            digest,
            r.get("definition_sha256")?,
            "{} supplieddefinitiondigest",
            r.id()
        );
        if digest != "-" {
            all_definitions.insert(digest.clone());
        }
        if op == "schedule" {
            schedules.insert(digest.clone());
        }
        if r.get("coupon")? != "-" {
            financial.insert(digest);
        }
        let actual = if op == "source_reference" {
            documentary_count += 1;
            assert_eq!(r.get("scope")?, "source_reference");
            documentary(r)?
        } else {
            public += 1;
            assert_eq!(r.get("scope")?, "public");
            match execute(r, &mut definitions, &mut identities)? {
                Ok(output) => output,
                Err(error) => error_output(&error)?,
            }
        };
        assert_eq!(actual.len(), 43);
        for key in OUTPUTS {
            assert_eq!(actual[key], r.get(key)?, "{} op{op} output{key}", r.id());
        }
        bump(&mut profiles, r.get("profile")?);
        bump(&mut operations, op);
        bump(&mut scopes, r.get("scope")?);
        bump(&mut expectations, &actual["expected"]);
    }
    for (key, value) in [
        ("profile-counts", count_json(&profiles)),
        ("operation-counts", count_json(&operations)),
        ("scope-counts", count_json(&scopes)),
        ("expectation-counts", count_json(&expectations)),
        (
            "unique-serialized-inputs",
            input_identities.len().to_string(),
        ),
        ("unique-financial-definitions", financial.len().to_string()),
        ("unique-schedule-definitions", schedules.len().to_string()),
        (
            "unique-supplied-definition-digests",
            all_definitions.len().to_string(),
        ),
        (
            "canonical-inputs-sha256",
            sha256(canonical_inputs.as_bytes())?,
        ),
        (
            "canonical-inputs-fnv64",
            format!("{:016x}", fnv64(canonical_inputs.as_bytes())),
        ),
    ] {
        assert_eq!(fixture.header(key), Some(value.as_str()), "census{key}");
    }
    assert_eq!(input_identities.len(), 79_794);
    assert_eq!(public, 79_758);
    assert_eq!(documentary_count, 36);
    assert_eq!(operations.len(), 14);
    assert_eq!(financial.len(), 2421);
    assert_eq!(schedules.len(), 321);
    assert_eq!(definitions.len(), identities.len());
    assert!(!definitions.is_empty());
    Ok(())
}
