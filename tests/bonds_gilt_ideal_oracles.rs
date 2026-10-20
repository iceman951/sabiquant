//! FR-GILT-03..13/22: public consumer of the frozen independent ideal oracles.
//!
//! The 374 price/risk rows and 51 target-driven inverse rows were authored from
//! the frozen gilt contract before gilt production existed. This consumer only
//! reads them: it rebuilds each public `ConventionalGilt` and `GiltPricingPlan`
//! from the complete supplied source definition, checks every public context
//! field, and compares each finite metric by direct closed-interval membership
//! in the fixture's outward conditional budget. Signed underflow/overflow
//! classes require the exact typed `GiltError::Value` outcome.
//!
//! An `uninformative` metric supplies no financial comparison. It is never a
//! wildcard pass: its actual outcome is counted and must still be a finite
//! value or a typed financial range/`NumericalBoundary` failure. A factor or
//! present-value failure before any view would make the row's metrics
//! unavailable; it must likewise be typed, and capacity/allocation/unexpected
//! errors fail the test. Operational factor/error order belongs to the
//! separate public numeric hand controls.
//!
//! Inverse rows assert the supplied target, resolved dirty target, residual
//! scale, options, bracket, counts and cached payload metadata, and compare the
//! cached root with the independently conditioned root interval, never with a
//! planted yield or a global ULP allowance. The eight printed DMO displays are a
//! separate check at their printed precision.

mod support;

use sabiquant::amounts::DecimalAmount;
use sabiquant::bonds::{
    BondPrice, BondPriceKind, BondYieldSensitivity, ConventionalGilt, GiltError, GiltFirstPeriod,
    GiltPricingPlan,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::error::{NonFinite, ValueError};
use sabiquant::math::{RootBracket, SolverOptions, TerminationReason};
use sabiquant::rates::RateError;
use sabiquant::time::Date;
use std::collections::BTreeMap;
use std::error::Error;
use std::ops::Bound;
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const PRICE_FIXTURE: &str = "validation/fixtures/bonds/gilt-ideal-oracle.fixture";
const INVERSE_FIXTURE: &str = "validation/fixtures/bonds/gilt-inverse-ideal-oracle.fixture";
const GENERATOR_SHA256: &str = "eef3f534d615d9bb0ffe40788d7826abb73c0a63f96ab7490a1f03f0253726fe";
const CONTRACT_SHA256: &str = "03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292";
const REQUIREMENTS_SHA256: &str =
    "290dc8817459d51dbee1264891db255e9f7fa48036fcd9f8950788d267d6a7fc";

/// Source/context columns shared by both fixtures, in order.
const CONTEXT_COLUMNS: [&str; 19] = [
    "id",
    "family",
    "issue",
    "maturity",
    "coupon",
    "first_dividend",
    "eom",
    "quasi_dates",
    "ex_dates",
    "settlement",
    "first_period",
    "next_index",
    "r",
    "s",
    "n",
    "ai_num",
    "ai_den",
    "bundle_indices",
    "source_coefficients",
];

/// The eight compared metrics: fixture prefix and public getter phase.
const METRICS: [(&str, &str); 8] = [
    ("dirty", "dirty"),
    ("clean", "clean"),
    ("d1", "price derivative"),
    ("d2", "price second derivative"),
    ("moment", "time weighted amount"),
    ("modified", "modified duration"),
    ("macaulay", "macaulay duration"),
    ("convexity", "convexity"),
];

/// Price-row columns after the shared context.
const PRICE_TAIL: [&str; 2] = ["yield_bits", "source_display"];

/// Inverse-row columns after the shared context.
const INVERSE_TAIL: [&str; 18] = [
    "target_kind",
    "target_bits",
    "dirty_target_bits",
    "bracket_lo_bits",
    "bracket_hi_bits",
    "abs_tol_bits",
    "rel_tol_bits",
    "max_evals",
    "residual_scale",
    "ideal_root_lo_bits",
    "ideal_root_hi_bits",
    "conditioned_lo_bits",
    "conditioned_hi_bits",
    "uniform_price_error",
    "slope_lower",
    "solver_width_bound",
    "inverse_method",
    "source_display",
];

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// Parses a flat `{"KEY":count,...}` census header.
fn census(text: &str) -> TestResult<BTreeMap<String, usize>> {
    let inner = text
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
        .ok_or("census braces")?;
    let mut map = BTreeMap::new();
    for entry in inner.split(',').filter(|entry| !entry.is_empty()) {
        let (key, count) = entry.split_once(':').ok_or("census entry")?;
        let key = key
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
            .ok_or("census key")?;
        map.insert(key.to_owned(), count.parse()?);
    }
    Ok(map)
}

fn tally<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for value in values {
        *map.entry(value.to_owned()).or_insert(0) += 1;
    }
    map
}

/// Loads a frozen fixture and verifies every pin, fingerprint and census.
fn pinned(
    path: &str,
    set: &str,
    tail: &[&str],
    metrics: bool,
    records: usize,
) -> TestResult<Fixture> {
    #[expect(
        clippy::disallowed_methods,
        reason = "test fingerprints independent fixture bytes"
    )]
    let text = std::fs::read_to_string(Fixture::repository_path(path))?;
    let (_, body) = text.split_once("\n\n").ok_or("fixture body separator")?;
    let fixture = Fixture::parse(&text)?;
    let mut columns: Vec<String> = CONTEXT_COLUMNS
        .iter()
        .chain(tail)
        .map(|&column| column.to_owned())
        .collect();
    if metrics {
        for (prefix, _) in METRICS {
            for suffix in ["ideal", "class", "lo_bits", "hi_bits"] {
                columns.push(format!("{prefix}_{suffix}"));
            }
        }
    }
    assert_eq!(fixture.columns(), columns.as_slice());
    assert_eq!(fixture.record_count(), records);
    assert_eq!(
        fixture.header("record-count"),
        Some(records.to_string().as_str())
    );
    assert_eq!(fixture.header("fixture-set"), Some(set));
    assert_eq!(fixture.header("domain"), Some("bonds"));
    assert_eq!(fixture.header("schema"), Some("1"));
    assert_eq!(fixture.header("generator-sha256"), Some(GENERATOR_SHA256));
    assert_eq!(
        fixture.header("contract-m8-gilt-contract-sha256"),
        Some(CONTRACT_SHA256)
    );
    assert_eq!(
        fixture.header("requirements-sha256"),
        Some(REQUIREMENTS_SHA256)
    );
    assert_eq!(fixture.header("decimal-precisions"), Some("160,240"));
    assert_eq!(fixture.header("inverse-bisections"), Some("300"));
    assert_eq!(
        fixture.header("body-fnv64"),
        Some(format!("{:016x}", fnv64(body.as_bytes())).as_str())
    );
    let generator = include_bytes!("../validation/generator/gilt_ideal_oracles.py");
    assert_eq!(fnv64(generator), 0xe25a_b4d9_fc40_f570);
    assert_eq!(fixture.header("generator-fnv64"), Some("e25ab4d9fc40f570"));
    let contract = include_bytes!("../docs/M8_GILT_CONTRACT.md");
    assert_eq!(fnv64(contract), 0x6c12_3fce_dcc3_c546);
    assert_eq!(
        fixture.header("contract-m8-gilt-contract-fnv64"),
        Some("6c123fcedcc3c546")
    );
    let requirements = include_str!("../docs/REQUIREMENTS.md")
        .lines()
        .filter(|line| line.starts_with("| FR-GILT-"))
        .collect::<Vec<_>>();
    assert_eq!(requirements.len(), 24);
    assert_eq!(
        fnv64((requirements.join("\n") + "\n").as_bytes()),
        0xe530_bcd5_70ca_7e6c
    );
    assert_eq!(
        fixture.header("requirements-fnv64"),
        Some("e530bcd570ca7e6c")
    );
    let families = tally(
        fixture
            .records()
            .map(|record| record.text("family").unwrap_or("")),
    );
    assert_eq!(census(fixture.header("census").ok_or("census")?)?, families);
    Ok(fixture)
}

fn bits(text: &str) -> TestResult<f64> {
    if text.len() != 16 {
        return Err(format!("expected 16 hexadecimal digits, got {text}").into());
    }
    Ok(f64::from_bits(u64::from_str_radix(text, 16)?))
}

fn dates(text: &str) -> TestResult<Vec<Date>> {
    text.split(',')
        .map(|date| date.parse::<Date>().map_err(Into::into))
        .collect()
}

fn indices(text: &str) -> TestResult<Vec<usize>> {
    if text == "-" {
        return Ok(Vec::new());
    }
    text.split(',')
        .map(|index| index.parse::<usize>().map_err(Into::into))
        .collect()
}

/// Unsigned little-endian 32-bit limbs for exact rational rounding.
#[derive(Clone)]
struct Big(Vec<u32>);

impl Big {
    fn parse(text: &str) -> TestResult<Self> {
        let mut limbs: Vec<u32> = vec![0];
        for digit in text.bytes() {
            let value = u64::from(
                digit
                    .checked_sub(b'0')
                    .filter(|value| *value < 10)
                    .ok_or("digit")?,
            );
            let mut carry = value;
            for limb in &mut limbs {
                let product = u64::from(*limb) * 10 + carry;
                *limb = u32::try_from(product & 0xffff_ffff)?;
                carry = product >> 32;
            }
            if carry != 0 {
                limbs.push(u32::try_from(carry)?);
            }
        }
        Ok(Self(limbs))
    }

    fn bits(&self) -> u32 {
        let mut total = 0;
        for (index, limb) in self.0.iter().enumerate() {
            if *limb != 0 {
                total = u32::try_from(index).unwrap_or(u32::MAX) * 32 + (32 - limb.leading_zeros());
            }
        }
        total
    }

    fn bit(&self, index: u32) -> bool {
        usize::try_from(index / 32)
            .ok()
            .and_then(|limb| self.0.get(limb))
            .is_some_and(|word| word >> (index % 32) & 1 == 1)
    }

    /// Shifts left (positive) or right (negative), reporting lost nonzero bits.
    fn shift(&self, amount: i32) -> TestResult<(Self, bool)> {
        let bits = self.bits();
        let width = usize::try_from(bits.saturating_add_signed(amount.max(0)) / 32 + 2)?;
        let mut result = Self(vec![0; width]);
        let mut lost = false;
        for index in 0..bits {
            if self.bit(index) {
                match usize::try_from(i64::from(index) + i64::from(amount)) {
                    Ok(target) => result.0[target / 32] |= 1 << (target % 32),
                    Err(_) => lost = true,
                }
            }
        }
        Ok((result, lost))
    }

    /// Divides by a nonzero u64, returning the quotient and nonzero remainder flag.
    fn div_small(&self, divisor: u64) -> (Self, bool) {
        let mut quotient = vec![0_u32; self.0.len()];
        let mut remainder = 0_u128;
        for (index, limb) in self.0.iter().enumerate().rev() {
            let value = (remainder << 32) | u128::from(*limb);
            quotient[index] = u32::try_from(value / u128::from(divisor)).unwrap_or(u32::MAX);
            remainder = value % u128::from(divisor);
        }
        (Self(quotient), remainder != 0)
    }

    fn low_u64(&self) -> u64 {
        let low = u64::from(self.0.first().copied().unwrap_or(0));
        let high = u64::from(self.0.get(1).copied().unwrap_or(0));
        low | (high << 32)
    }
}

/// Correctly rounds the exact rational `numerator/denominator` to binary64,
/// ties to even. Inputs are normal-range AI values in these fixtures.
fn exact_ratio(numerator: &str, denominator: &str) -> TestResult<f64> {
    let (negative, digits) = match numerator.strip_prefix('-') {
        Some(digits) => (true, digits),
        None => (false, numerator),
    };
    let numerator = Big::parse(digits)?;
    let denominator: u64 = denominator.parse()?;
    if numerator.bits() == 0 {
        return Ok(0.0);
    }
    let denominator_bits = 64 - denominator.leading_zeros();
    // Scale so that the quotient has 55 or 56 significant bits.
    let shift = 56 - i32::try_from(numerator.bits())? + i32::try_from(denominator_bits)?;
    let (scaled, lost) = numerator.shift(shift)?;
    let (quotient, remainder) = scaled.div_small(denominator);
    let sticky = lost || remainder;
    let quotient_bits = quotient.bits();
    assert!((55..=57).contains(&quotient_bits), "scaled quotient width");
    let extra = quotient_bits - 53;
    let (mantissa_big, _) = quotient.shift(-i32::try_from(extra)?)?;
    let mut mantissa = mantissa_big.low_u64();
    let guard = quotient.bit(extra - 1);
    let mut below = sticky;
    for index in 0..extra - 1 {
        below |= quotient.bit(index);
    }
    if guard && (below || mantissa & 1 == 1) {
        mantissa += 1;
    }
    let mut exponent = i32::try_from(extra)? - shift;
    if mantissa == 1 << 53 {
        mantissa >>= 1;
        exponent += 1;
    }
    let field = u64::try_from(exponent + 52 + 1023)?;
    let value = f64::from_bits((field << 52) | (mantissa & ((1 << 52) - 1)));
    Ok(if negative { -value } else { value })
}

fn first_period(text: &str) -> TestResult<GiltFirstPeriod> {
    match text {
        "Regular" => Ok(GiltFirstPeriod::Regular),
        "Short" => Ok(GiltFirstPeriod::Short),
        "Long" => Ok(GiltFirstPeriod::Long),
        other => Err(format!("unknown first period {other}").into()),
    }
}

/// Rebuilds the public instrument and checks every retained definition field.
fn instrument(record: &Record<'_>) -> TestResult<ConventionalGilt> {
    let issue: Date = record.text("issue")?.parse()?;
    let maturity: Date = record.text("maturity")?.parse()?;
    let coupon: DecimalAmount = record.text("coupon")?.parse()?;
    let first_dividend: Date = record.text("first_dividend")?.parse()?;
    let end_of_month: EndOfMonth = record.text("eom")?.parse()?;
    let quasi = dates(record.text("quasi_dates")?)?;
    let ex = dates(record.text("ex_dates")?)?;
    let gilt = ConventionalGilt::new(
        issue,
        maturity,
        coupon,
        first_dividend,
        quasi.clone(),
        ex.clone(),
        end_of_month,
        ConventionalGilt::MAX_QUASI_DATES,
    )?;
    let id = record.id();
    assert_eq!(
        gilt.source_version(),
        "UK-DMO-Conventional-v4-2024-12-18",
        "{id}"
    );
    assert_eq!(gilt.issue_date(), issue, "{id}");
    assert_eq!(gilt.maturity_date(), maturity, "{id}");
    assert_eq!(gilt.coupon_per_100(), coupon, "{id}");
    assert_eq!(gilt.first_dividend_date(), first_dividend, "{id}");
    assert_eq!(gilt.end_of_month(), end_of_month, "{id}");
    assert_eq!(
        gilt.first_period(),
        first_period(record.text("first_period")?)?,
        "{id}"
    );
    assert_eq!(gilt.quasi_dates(), quasi.as_slice(), "{id}");
    assert_eq!(gilt.ex_dividend_dates(), ex.as_slice(), "{id}");
    assert_eq!(gilt.dividend_count(), ex.len(), "{id}");
    let first = gilt.first_dividend_quasi_index();
    assert_eq!(quasi.get(first), Some(&first_dividend), "{id}");
    assert_eq!(gilt.dividend_dates(), &quasi[first..], "{id}");
    Ok(gilt)
}

/// Prepares the public plan and checks every remaining-period inspector.
fn prepared<'a>(
    record: &Record<'_>,
    gilt: &'a ConventionalGilt,
) -> TestResult<GiltPricingPlan<'a>> {
    let id = record.id();
    let settlement: Date = record.text("settlement")?.parse()?;
    let plan = GiltPricingPlan::new(gilt, settlement, GiltPricingPlan::MAX_BUNDLES)?;
    assert_eq!(plan.settlement_date(), settlement, "{id}");
    assert_eq!(plan.source_version(), gilt.source_version(), "{id}");
    assert_eq!(
        plan.next_quasi_index(),
        record.parse::<usize>("next_index")?,
        "{id}"
    );
    assert_eq!(plan.remaining_days(), record.parse::<u32>("r")?, "{id}");
    assert_eq!(plan.quasi_period_days(), record.parse::<u32>("s")?, "{id}");
    assert_eq!(
        plan.remaining_quasi_periods(),
        record.parse::<u32>("n")?,
        "{id}"
    );
    let bundles = indices(record.text("bundle_indices")?)?;
    let coefficients = record.text("source_coefficients")?.split(',').count();
    assert_eq!(bundles.len(), coefficients, "{id}");
    assert_eq!(plan.bundle_count(), bundles.len(), "{id}");
    for (position, index) in bundles.iter().enumerate() {
        assert_eq!(plan.bundle_quasi_index(position), Some(*index), "{id}");
    }
    assert_eq!(plan.bundle_quasi_index(bundles.len()), None, "{id}");
    let accrued = exact_ratio(record.text("ai_num")?, record.text("ai_den")?)?;
    assert_eq!(
        plan.accrued_per_100().to_bits(),
        accrued.to_bits(),
        "{id} cached AI"
    );
    Ok(plan)
}

/// Accepts only typed financial range or factor-boundary failures.
fn typed_financial(error: &GiltError) -> bool {
    match error {
        GiltError::Rate {
            phase: "discount factor",
            cause: RateError::NumericalBoundary { .. } | RateError::Value(_),
            ..
        } => true,
        GiltError::Value {
            phase,
            cause: ValueError::ResultNotFinite { .. } | ValueError::ResultOutOfDomain { .. },
            ..
        } => *phase != "yield quote",
        GiltError::Sum {
            phase: "dirty price reduction" | "clean price reduction",
            ..
        } => true,
        _ => false,
    }
}

/// Requires the exact typed signed range outcome of one getter.
fn assert_range(id: &str, phase: &str, class: &str, outcome: &Result<f64, GiltError>) {
    let negative = class.starts_with("negative");
    let matched = match outcome {
        Err(GiltError::Value {
            index: None,
            phase: observed,
            cause,
            ..
        }) if *observed == phase => match (class.ends_with("overflow"), cause) {
            (
                true,
                ValueError::ResultNotFinite {
                    quantity, value, ..
                },
            ) => {
                *quantity == phase
                    && *value
                        == if negative {
                            NonFinite::NegativeInfinity
                        } else {
                            NonFinite::PositiveInfinity
                        }
            }
            (
                false,
                ValueError::ResultOutOfDomain {
                    quantity,
                    value,
                    domain,
                    ..
                },
            ) => {
                *quantity == phase
                    && *value == 0.0
                    && value.is_sign_negative() == negative
                    && if negative {
                        domain.lower() == Bound::Unbounded && domain.upper() == Bound::Excluded(0.0)
                    } else {
                        domain.lower() == Bound::Excluded(0.0) && domain.upper() == Bound::Unbounded
                    }
            }
            _ => false,
        },
        _ => false,
    };
    assert!(matched, "{id} {phase} {class}: {outcome:?}");
}

/// Outcome tallies reported by the consumer.
#[derive(Default)]
struct Census {
    finite_checked: usize,
    range_checked: usize,
    uninformative: BTreeMap<String, usize>,
    uninformative_rows: Vec<String>,
    unavailable: Vec<String>,
    failed_rows: Vec<String>,
    displays: usize,
}

fn outcome_label(outcome: &Result<f64, GiltError>) -> String {
    match outcome {
        Ok(value) if value.is_finite() => "finite".to_owned(),
        Ok(value) => format!("nonfinite {value}"),
        Err(error) => error.kind().to_owned(),
    }
}

/// Compares one metric against its class and closed outward interval.
fn compare(
    record: &Record<'_>,
    prefix: &str,
    phase: &str,
    outcome: &Result<f64, GiltError>,
    census: &mut Census,
) -> TestResult {
    let id = record.id();
    let class = record.text(&format!("{prefix}_class"))?;
    match class {
        "finite" => {
            let lower = bits(record.text(&format!("{prefix}_lo_bits"))?)?;
            let upper = bits(record.text(&format!("{prefix}_hi_bits"))?)?;
            assert!(lower <= upper, "{id} {prefix} interval order");
            match outcome {
                Ok(value) => assert!(
                    lower <= *value && *value <= upper,
                    "{id} {prefix}: {value:e} outside [{lower:e}, {upper:e}]"
                ),
                Err(error) => {
                    return Err(format!("{id} {prefix}: admitted view failed: {error:?}").into());
                }
            }
            census.finite_checked += 1;
        }
        "positive_underflow" | "negative_underflow" | "positive_overflow" | "negative_overflow" => {
            assert_range(id, phase, class, outcome);
            census.range_checked += 1;
        }
        "uninformative" => {
            if let Err(error) = outcome {
                assert!(typed_financial(error), "{id} {prefix}: untyped {error:?}");
            }
            *census
                .uninformative
                .entry(format!("{prefix}:{}", outcome_label(outcome)))
                .or_insert(0) += 1;
        }
        other => return Err(format!("{id} {prefix}: unknown class {other}").into()),
    }
    Ok(())
}

/// Checks a printed six-place DMO price display at its own precision.
fn display_price(id: &str, display: &str, dirty: f64) -> TestResult {
    let printed: f64 = display.parse()?;
    // |ideal - printed| <= 0.5e-6; the represented-yield model differs from the
    // printed decimal-yield source by far less than the remaining slack.
    assert!(
        (dirty - printed).abs() <= 0.5e-6 + 1e-9,
        "{id} display {display} versus {dirty}"
    );
    Ok(())
}

/// FR-GILT-03..13/22: all 374 price/risk rows against independent budgets.
#[test]
fn ideal_price_and_risk_rows_meet_their_classified_budgets() -> TestResult {
    let fixture = pinned(
        PRICE_FIXTURE,
        "M8-GILT-IDEAL-PRICE-RISK",
        &PRICE_TAIL,
        true,
        374,
    )?;
    let statuses = tally(fixture.records().flat_map(|record| {
        METRICS.map(|(prefix, _)| record.text(&format!("{prefix}_class")).unwrap_or(""))
    }));
    assert_eq!(
        census(
            fixture
                .header("metric-status-counts")
                .ok_or("status counts")?
        )?,
        statuses
    );
    assert_eq!(fixture.header("inverse-method-counts"), Some("{}"));
    let mut tallies = Census::default();
    for (position, record) in fixture.records().enumerate() {
        let id = record.id();
        assert_eq!(id, format!("GILT-I-{:05}", position + 1));
        let gilt = instrument(&record)?;
        let plan = prepared(&record, &gilt)?;
        let rate = bits(record.text("yield_bits")?)?;
        let canonical = if rate == 0.0 { 0.0 } else { rate };
        let valuation = plan.evaluate(rate);
        let risk = plan.risk(rate);
        let (valuation, risk) = match (valuation, risk) {
            (Ok(valuation), Ok(risk)) => (valuation, risk),
            (Err(first), Err(second)) => {
                // The same traversal fails before any view: every metric of the
                // row is unavailable. Uninformative cells count with this typed
                // outcome; an unavailable informative cell is reported.
                assert!(typed_financial(&first), "{id}: untyped {first:?}");
                assert_eq!(first, second, "{id}: evaluate/risk traversal differ");
                let mut informative = false;
                for (prefix, _) in METRICS {
                    let class = record.text(&format!("{prefix}_class"))?;
                    if class == "uninformative" {
                        *tallies
                            .uninformative
                            .entry(format!("{prefix}:{}", first.kind()))
                            .or_insert(0) += 1;
                    } else {
                        informative = true;
                        tallies
                            .unavailable
                            .push(format!("{id}:{prefix}:{class}:{}", first.kind()));
                    }
                }
                if !informative {
                    tallies.uninformative_rows.push(id.to_owned());
                }
                tallies.failed_rows.push(format!("{id}:{first:?}"));
                continue;
            }
            (valuation, risk) => {
                return Err(format!("{id}: evaluate {valuation:?} versus risk {risk:?}").into());
            }
        };
        let next = plan.next_quasi_index();
        assert_eq!(
            valuation.yield_rate().to_bits(),
            canonical.to_bits(),
            "{id}"
        );
        assert_eq!(risk.yield_rate().to_bits(), canonical.to_bits(), "{id}");
        assert_eq!(
            valuation.next_quasi_date(),
            gilt.quasi_dates()[next],
            "{id}"
        );
        assert_eq!(valuation.settlement_date(), plan.settlement_date(), "{id}");
        assert_eq!(risk.settlement_date(), plan.settlement_date(), "{id}");
        assert_eq!(
            (
                valuation.remaining_quasi_periods(),
                valuation.remaining_days(),
                valuation.quasi_period_days()
            ),
            (
                plan.remaining_quasi_periods(),
                plan.remaining_days(),
                plan.quasi_period_days()
            ),
            "{id}"
        );
        assert_eq!(valuation.first_period(), gilt.first_period(), "{id}");
        assert_eq!(risk.coupon_per_100(), gilt.coupon_per_100(), "{id}");
        assert_eq!(
            valuation.accrued_per_100().to_bits(),
            plan.accrued_per_100().to_bits(),
            "{id}"
        );
        let dirty = valuation.dirty_per_100();
        let dirty_price = valuation.dirty_price();
        assert_eq!(dirty_price.kind(), BondPriceKind::Dirty, "{id}");
        assert_eq!(
            dirty_price.value_per_100_face().to_bits(),
            dirty.to_bits(),
            "{id}"
        );
        let clean = valuation.clean_price().map(BondPrice::value_per_100_face);
        if let Ok(value) = clean {
            // exact_sum of two binary64 values is their correctly rounded sum.
            let expected = dirty - valuation.accrued_per_100();
            let expected = if expected == 0.0 { 0.0 } else { expected };
            assert_eq!(value.to_bits(), expected.to_bits(), "{id} clean reduction");
        }
        let views: [Result<f64, GiltError>; 8] = [
            Ok(dirty),
            clean,
            risk.price_derivative(),
            risk.price_second_derivative(),
            risk.time_weighted_amount(),
            risk.modified_duration(),
            risk.macaulay_duration(),
            risk.convexity(),
        ];
        let informative = METRICS.iter().any(|(prefix, _)| {
            record
                .text(&format!("{prefix}_class"))
                .is_ok_and(|class| class != "uninformative")
        });
        if !informative {
            tallies.uninformative_rows.push(id.to_owned());
        }
        for ((prefix, phase), outcome) in METRICS.iter().zip(views.iter()) {
            compare(&record, prefix, phase, outcome, &mut tallies)?;
        }
        let display = record.text("source_display")?;
        if display != "-" {
            display_price(id, display, dirty)?;
            tallies.displays += 1;
        }
    }
    let uninformative: usize = tallies.uninformative.values().sum();
    assert_eq!(
        tallies.finite_checked + tallies.range_checked + uninformative,
        374 * 8,
        "every cell is classified once"
    );
    // Every informative row evaluated on this host; no metric was unavailable.
    assert_eq!(tallies.unavailable, Vec::<String>::new());
    assert_eq!(tallies.finite_checked, 2805);
    assert_eq!(tallies.range_checked, 11);
    assert_eq!(uninformative, 176);
    assert_eq!(tallies.uninformative_rows.len(), 22);
    assert_eq!(tallies.displays, 8);
    // Rows failing before views are typed and wholly uninformative.
    assert!(tallies.failed_rows.len() <= tallies.uninformative_rows.len());
    Ok(())
}

/// FR-GILT-12/13/22: all 51 target-driven inverses against conditioned roots.
#[test]
fn ideal_inverse_rows_land_inside_conditioned_roots() -> TestResult {
    let fixture = pinned(
        INVERSE_FIXTURE,
        "M8-GILT-IDEAL-INVERSE",
        &INVERSE_TAIL,
        false,
        51,
    )?;
    assert_eq!(fixture.header("metric-status-counts"), Some("{}"));
    let methods = tally(
        fixture
            .records()
            .map(|record| record.text("inverse_method").unwrap_or("")),
    );
    assert_eq!(
        census(
            fixture
                .header("inverse-method-counts")
                .ok_or("method counts")?
        )?,
        methods
    );
    let mut displays = 0;
    for (position, record) in fixture.records().enumerate() {
        let id = record.id();
        assert_eq!(id, format!("GILT-V-{:05}", position + 1));
        let gilt = instrument(&record)?;
        let plan = prepared(&record, &gilt)?;
        let kind = match record.text("target_kind")? {
            "Dirty" => BondPriceKind::Dirty,
            "Clean" => BondPriceKind::Clean,
            other => return Err(format!("{id}: target kind {other}").into()),
        };
        let target = BondPrice::new(bits(record.text("target_bits")?)?, kind)?;
        let bracket = RootBracket::new(
            bits(record.text("bracket_lo_bits")?)?,
            bits(record.text("bracket_hi_bits")?)?,
        )?;
        let options = SolverOptions::new(
            bits(record.text("abs_tol_bits")?)?,
            bits(record.text("rel_tol_bits")?)?,
            record.parse("max_evals")?,
        )?;
        let report = plan.yield_from_price(target, bracket, options)?;
        assert_eq!(report.target(), target, "{id}");
        let dirty_target = bits(record.text("dirty_target_bits")?)?;
        assert_eq!(
            report.dirty_target_per_100().to_bits(),
            dirty_target.to_bits(),
            "{id} resolved T"
        );
        assert_eq!(
            report.residual_scale_exponent(),
            record.parse::<u8>("residual_scale")?,
            "{id}"
        );
        assert_eq!(report.initial_bracket(), bracket, "{id}");
        assert_eq!(report.options(), options, "{id}");
        assert_eq!(
            report.sensitivity(),
            BondYieldSensitivity::MathematicallyStrictlyDecreasing,
            "{id}"
        );
        assert_eq!(
            report.accrued_per_100().to_bits(),
            plan.accrued_per_100().to_bits(),
            "{id}"
        );
        let solve = report.solve_report();
        let rate = report.yield_rate();
        assert_eq!(solve.point().to_bits(), rate.to_bits(), "{id} cached point");
        assert!(
            solve.evaluations().function() <= options.max_evaluations(),
            "{id}"
        );
        assert!(solve.evaluations().function() >= 1, "{id}");
        assert_eq!(solve.evaluations().derivative(), 0, "{id}");
        assert!(
            matches!(
                solve.reason(),
                TerminationReason::FunctionZero | TerminationReason::BracketTolerance
            ),
            "{id}"
        );
        let final_bracket = solve.bracket();
        assert!(
            bracket.lower() <= final_bracket.lower() && final_bracket.upper() <= bracket.upper(),
            "{id} final bracket inside the input"
        );
        assert!(
            final_bracket.lower() <= rate && rate <= final_bracket.upper(),
            "{id} cached point inside its bracket"
        );
        // Scale zero: the residual and amount residual are RN(Dhat - T).
        let difference = report.dirty_per_100() - dirty_target;
        let difference = if difference == 0.0 { 0.0 } else { difference };
        assert_eq!(
            solve.residual().to_bits(),
            difference.to_bits(),
            "{id} residual"
        );
        assert_eq!(
            report.amount_residual()?.to_bits(),
            difference.to_bits(),
            "{id} amount residual"
        );
        assert_eq!(
            report.dirty_price().value_per_100_face().to_bits(),
            report.dirty_per_100().to_bits(),
            "{id}"
        );
        let clean = report.clean_price()?.value_per_100_face();
        let expected = report.dirty_per_100() - report.accrued_per_100();
        let expected = if expected == 0.0 { 0.0 } else { expected };
        assert_eq!(clean.to_bits(), expected.to_bits(), "{id} clean view");
        assert_eq!(
            (report.settlement_date(), report.next_quasi_date()),
            (
                plan.settlement_date(),
                gilt.quasi_dates()[plan.next_quasi_index()]
            ),
            "{id}"
        );
        assert_eq!(report.first_period(), gilt.first_period(), "{id}");
        // The cached point lies inside the independent conditioned interval,
        // which itself contains the ideal root interval.
        let ideal = (
            bits(record.text("ideal_root_lo_bits")?)?,
            bits(record.text("ideal_root_hi_bits")?)?,
        );
        let conditioned = (
            bits(record.text("conditioned_lo_bits")?)?,
            bits(record.text("conditioned_hi_bits")?)?,
        );
        assert!(
            conditioned.0 <= ideal.0 && ideal.0 <= ideal.1 && ideal.1 <= conditioned.1,
            "{id} interval nesting"
        );
        assert!(
            conditioned.0 <= rate && rate <= conditioned.1,
            "{id}: root {rate:e} outside [{:e}, {:e}]",
            conditioned.0,
            conditioned.1
        );
        let display = record.text("source_display")?;
        if display != "-" {
            let printed: f64 = display.parse()?;
            // Nine printed decimals: half a unit plus binary slack.
            assert!(
                (rate - printed).abs() <= 0.5e-9 + 1e-15,
                "{id} printed yield {display} versus {rate}"
            );
            displays += 1;
        }
    }
    assert_eq!(displays, 8);
    Ok(())
}

/// The exact rational rounding helper agrees with IEEE division where exact.
#[test]
fn exact_ratio_helper_matches_correctly_rounded_division() -> TestResult {
    // Both operands are exact binary64 integers, so IEEE division is the
    // correctly rounded quotient.
    for (numerator, denominator, quotient) in [
        ("48", "13", 48.0_f64 / 13.0),
        ("-1", "61", -1.0 / 61.0),
        ("1", "3", 1.0 / 3.0),
        ("2", "3", 2.0 / 3.0),
        ("123456789", "97", 123_456_789.0 / 97.0),
        ("-36", "362", -36.0 / 362.0),
        ("7", "1", 7.0),
        ("0", "5", 0.0),
    ] {
        let observed = exact_ratio(numerator, denominator)?;
        assert_eq!(
            observed.to_bits(),
            quotient.to_bits(),
            "{numerator}/{denominator}"
        );
    }
    // 2^53+1 is a tie that rounds to even 2^53; 2^53+3 rounds up to 2^53+4.
    assert_eq!(
        exact_ratio("9007199254740993", "1")?.to_bits(),
        9_007_199_254_740_992.0_f64.to_bits()
    );
    assert_eq!(
        exact_ratio("9007199254740995", "1")?.to_bits(),
        9_007_199_254_740_996.0_f64.to_bits()
    );
    // A 40-digit numerator: RN(10^39/2) is the literal 5e38.
    let huge = exact_ratio("1000000000000000000000000000000000000000", "2")?;
    assert_eq!(huge.to_bits(), 5e38_f64.to_bits());
    Ok(())
}
