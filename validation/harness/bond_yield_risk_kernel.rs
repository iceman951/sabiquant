//! FR-YR-03/06/07/16: private primitive checks in an attested temporary crate.
//!
//! This file is appended inside a cfg(test) child of the copied yield_risk.rs.
//! Expected extended states are compared only; all actual sums derive through
//! round_extended_term, accumulate_risk_term and subtract_risk_sums from finite
//! factor recipes. No BondRisk/BondYieldReport or wide financial state is forged.

use super::*;
use crate::support::fixture::{Fixture, Record};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;

type CheckResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const COLUMNS: [&str; 24] = [
    "id",
    "op",
    "family",
    "comp",
    "y_bits",
    "t_bits",
    "amount_bits",
    "dirty_bits",
    "target_bits",
    "numerators",
    "denominators",
    "pow2",
    "negative",
    "recipes",
    "scale",
    "status",
    "sign",
    "significand",
    "exponent",
    "expected_bits",
    "error_kind",
    "error_phase",
    "error_index",
    "detail",
];

fn hex_bits(text: &str) -> CheckResult<u64> {
    if text.len() != 16
        || !text
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(format!("expected 16 lowercase IEEE bits, got {text}").into());
    }
    Ok(u64::from_str_radix(text, 16)?)
}

fn finite(text: &str) -> CheckResult<f64> {
    let value = f64::from_bits(hex_bits(text)?);
    if !value.is_finite() {
        return Err("numeric primitive input is not finite".into());
    }
    Ok(value)
}

fn flag(text: &str) -> CheckResult<bool> {
    match text {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err("sign flag must be0 or1".into()),
    }
}

fn factors(text: &str, maximum: usize, allow_empty: bool) -> CheckResult<Vec<f64>> {
    if text == "-" && allow_empty {
        return Ok(Vec::new());
    }
    let result: CheckResult<Vec<_>> = text.split(':').map(finite).collect();
    let result = result?;
    if result.is_empty() || result.len() > maximum || result.contains(&0.0) {
        return Err(
            "term operands must be nonzero finite values within the contract factor count".into(),
        );
    }
    Ok(result)
}

fn term(
    num: &str,
    den: &str,
    power: &str,
    negative: &str,
    index: usize,
) -> CheckResult<ExtendedTerm> {
    let num = factors(num, 5, false)?;
    let den = factors(den, 2, true)?;
    let power: i32 = power.parse()?;
    if !matches!(power, 0 | 1) {
        return Err("term extra exponent outside the adopted recipe domain".into());
    }
    Ok(round_extended_term(
        &num,
        &den,
        power,
        flag(negative)?,
        index,
        "time weighted amount",
    )?)
}

fn sum_from_recipes(text: &str) -> CheckResult<SignedExtendedSum> {
    let mut positive = RiskWide::ZERO;
    let mut negative = RiskWide::ZERO;
    if text != "-" {
        for (index, recipe) in text.split(';').enumerate() {
            let cells: Vec<_> = recipe.split(',').collect();
            if cells.len() != 4 {
                return Err("sum recipe needs negative,pow2,num,den".into());
            }
            let value = term(cells[2], cells[3], cells[1], cells[0], index)?;
            accumulate_risk_term(
                &mut positive,
                &mut negative,
                value,
                index,
                "time weighted amount",
            )?;
        }
    }
    Ok(subtract_risk_sums(
        positive,
        negative,
        "time weighted amount",
    )?)
}

fn magnitude_hex(mut value: RiskWide) -> CheckResult<String> {
    if value.is_zero() {
        return Ok("0".to_owned());
    }
    // This representation-only division uses fixed native base16^7 chunks;
    // expected digits never become an input to a financial/kernel operation.
    let mut chunks = Vec::new();
    while !value.is_zero() {
        let (quotient, remainder) = value
            .div_rem_small(0x1000_0000)
            .ok_or("hex conversion division")?;
        chunks.push(remainder);
        value = quotient;
    }
    let first = chunks.pop().ok_or("nonzero magnitude lacks first chunk")?;
    let mut text = format!("{first:x}");
    while let Some(chunk) = chunks.pop() {
        text.push_str(&format!("{chunk:07x}"));
    }
    Ok(text)
}

fn assert_zero_domain(domain: crate::error::Interval, negative: bool) -> CheckResult {
    match (negative, domain.lower(), domain.upper()) {
        (false, Bound::Excluded(zero), Bound::Unbounded) if zero.to_bits() == 0 => Ok(()),
        (true, Bound::Unbounded, Bound::Excluded(zero)) if zero.to_bits() == 0 => Ok(()),
        _ => Err(format!("incorrect signed underflow interval {domain:?}").into()),
    }
}

fn classify(
    result: Result<f64, BondError>,
    phase: &'static str,
) -> CheckResult<(String, Option<u64>)> {
    match result {
        Ok(value) => {
            assert!(value.is_finite());
            if value == 0.0 {
                assert_eq!(value.to_bits(), 0);
            }
            Ok(("ok".to_owned(), Some(value.to_bits())))
        }
        Err(error) => {
            let status = match &error {
                BondError::Value {
                    index,
                    phase: actual_phase,
                    cause,
                } => {
                    assert_eq!(*index, None);
                    assert_eq!(*actual_phase, phase);
                    assert_eq!(
                        error.source().and_then(|e| e.downcast_ref::<ValueError>()),
                        Some(cause)
                    );
                    match cause {
                        ValueError::ResultNotFinite { quantity, value } => {
                            assert_eq!(*quantity, phase);
                            match value {
                                NonFinite::PositiveInfinity => "overflow+",
                                NonFinite::NegativeInfinity => "overflow-",
                                NonFinite::NaN => {
                                    return Err("finite primitive unexpectedly produced NaN".into());
                                }
                            }
                        }
                        ValueError::ResultOutOfDomain {
                            quantity,
                            value,
                            domain,
                        } => {
                            assert_eq!(*quantity, phase);
                            assert_eq!(value.to_bits() & !SIGN_BIT, 0);
                            let negative = value.is_sign_negative();
                            assert_zero_domain(*domain, negative)?;
                            if negative { "underflow-" } else { "underflow+" }
                        }
                        _ => return Err(format!("unexpected value cause {cause:?}").into()),
                    }
                }
                BondError::ZeroDirtyPrice {
                    measure,
                    yield_rate,
                } => {
                    assert_eq!(*measure, phase);
                    assert_eq!(yield_rate.to_bits(), 0);
                    assert!(error.source().is_none());
                    "zero_dirty"
                }
                _ => return Err(format!("unexpected primitive error {error:?}").into()),
            };
            Ok((status.to_owned(), None))
        }
    }
}

fn compare_result(
    id: &str,
    actual: (String, Option<u64>),
    status: &str,
    expected: &str,
) -> CheckResult {
    assert_eq!(actual.0, status, "{id}");
    if status == "ok" {
        assert_eq!(actual.1, Some(hex_bits(expected)?), "{id}");
    } else {
        assert_eq!(expected, "-", "{id}: error bits must be absent");
        assert!(actual.1.is_none());
    }
    Ok(())
}

fn detail_result<'a>(detail: &'a str, prefix: &str) -> CheckResult<(&'a str, &'a str)> {
    let cells: Vec<_> = detail.split(':').collect();
    if cells.len() != 3 || cells[0] != prefix {
        return Err(format!("invalid {prefix} detail {detail}").into());
    }
    Ok((cells[1], cells[2]))
}

fn check_main_error_cells(row: &Record<'_>, status: &str, phase: &str) -> CheckResult {
    if status == "ok" {
        for name in ["error_kind", "error_phase", "error_index"] {
            assert_eq!(row.text(name)?, "-", "{}: {name}", row.id());
        }
    } else {
        assert_eq!(row.text("error_kind")?, "bond.value", "{}", row.id());
        assert_eq!(row.text("error_phase")?, phase, "{}", row.id());
        assert_eq!(row.text("error_index")?, "-", "{}", row.id());
    }
    Ok(())
}

fn residual(row: &Record<'_>) -> CheckResult {
    let dirty = finite(row.text("dirty_bits")?)?;
    let target = finite(row.text("target_bits")?)?;
    let scale = u8::from((target.to_bits() & !SIGN_BIT) >= RESIDUAL_SCALE_THRESHOLD_BITS);
    assert_eq!(row.text("scale")?.parse::<u8>()?, scale, "{}", row.id());
    let value = compute_scaled_residual(dirty, target, scale)?;
    compare_result(
        row.id(),
        ("ok".to_owned(), Some(value.to_bits())),
        row.text("status")?,
        row.text("expected_bits")?,
    )?;
    check_main_error_cells(row, "ok", "yield residual")?;
    let (magnitude, negative) = min_sub_difference(dirty, target, "yield amount residual")?;
    let amount = if magnitude.is_zero() {
        Ok(0.0)
    } else {
        let bits = round_wide_ratio_to_f64(magnitude, MinSubWide::from_u128(1), -1074)
            .ok_or("valid amount residual exceeded capacity")?;
        finish_signed_risk_f64(bits, negative, "yield amount residual")
    };
    let (status, expected) = detail_result(row.text("detail")?, "amount")?;
    compare_result(
        row.id(),
        classify(amount, "yield amount residual")?,
        status,
        expected,
    )
}

fn extended_term(row: &Record<'_>) -> CheckResult {
    let actual = term(
        row.text("numerators")?,
        row.text("denominators")?,
        row.text("pow2")?,
        row.text("negative")?,
        0,
    )?;
    assert_eq!(row.text("status")?, "ok", "{}", row.id());
    assert_eq!(actual.negative, flag(row.text("sign")?)?, "{}", row.id());
    assert_eq!(
        actual.significand,
        hex_bits(row.text("significand")?)?,
        "{}",
        row.id()
    );
    assert_eq!(
        actual.exponent,
        row.text("exponent")?.parse::<i32>()?,
        "{}",
        row.id()
    );
    assert!((1_u64 << 52..1_u64 << 53).contains(&actual.significand));
    check_main_error_cells(row, "ok", "time weighted amount")
}

fn extended_sum(row: &Record<'_>) -> CheckResult {
    let sum = sum_from_recipes(row.text("recipes")?)?;
    assert_eq!(sum.negative, flag(row.text("sign")?)?, "{}", row.id());
    assert_eq!(
        magnitude_hex(sum.words)?,
        row.text("significand")?,
        "{}",
        row.id()
    );
    assert_eq!(row.text("exponent")?, "-8192");
    if sum.words.is_zero() {
        assert!(!sum.negative);
    }
    compare_result(
        row.id(),
        classify(
            round_raw_risk(sum, "time weighted amount"),
            "time weighted amount",
        )?,
        row.text("status")?,
        row.text("expected_bits")?,
    )?;
    check_main_error_cells(row, row.text("status")?, "time weighted amount")?;
    let dirty = finite(row.text("dirty_bits")?)?;
    let normalized = round_normalized_risk(sum, false, dirty, 0.0, "macaulay duration");
    let (status, expected) = detail_result(row.text("detail")?, "normalized")?;
    compare_result(
        row.id(),
        classify(normalized, "macaulay duration")?,
        status,
        expected,
    )
}

fn compounding(text: &str) -> CheckResult<Compounding> {
    match text {
        "Simple" => Ok(Compounding::SIMPLE),
        "Continuous" => Ok(Compounding::CONTINUOUS),
        _ => {
            let (method, count) = text
                .split_once(':')
                .ok_or("periodic compounding needs method:count")?;
            let count: u32 = count.parse()?;
            let frequency = crate::time::Frequency::ALL
                .into_iter()
                .find(|f| f.events_per_year() == Some(count))
                .ok_or("unsupported periodic count")?;
            Ok(match method {
                "Compounded" => Compounding::compounded(frequency)?,
                "SimpleThenCompounded" => Compounding::simple_then_compounded(frequency)?,
                "CompoundedThenSimple" => Compounding::compounded_then_simple(frequency)?,
                _ => return Err("unknown compounding method".into()),
            })
        }
    }
}

fn raw_contribution(
    num: &[f64],
    den: &[f64],
    power: i32,
    negative: bool,
    phase: &'static str,
) -> Result<f64, BondError> {
    let value = round_extended_term(num, den, power, negative, 0, phase)?;
    let mut positive = RiskWide::ZERO;
    let mut negatives = RiskWide::ZERO;
    accumulate_risk_term(&mut positive, &mut negatives, value, 0, phase)?;
    round_raw_risk(subtract_risk_sums(positive, negatives, phase)?, phase)
}

fn compare_bounds(id: &str, label: &str, value: f64, bounds: &str) -> CheckResult {
    let (lower, upper) = bounds
        .split_once(':')
        .ok_or("flow bounds need lower:upper")?;
    let lower_value = finite(lower)?;
    let upper_value = finite(upper)?;
    assert!(lower_value <= upper_value, "{id}: reversed {label} bounds");
    assert!(
        value.is_finite() && lower_value <= value && value <= upper_value,
        "{id}: {label} {:016x} outside {lower}:{upper}",
        value.to_bits()
    );
    Ok(())
}

fn flow(row: &Record<'_>) -> CheckResult {
    assert_eq!(
        row.text("status")?,
        "ok",
        "{}: only successful flow observations are in this corpus",
        row.id()
    );
    check_main_error_cells(row, "ok", "discount factor")?;
    let compounding = compounding(row.text("comp")?)?;
    let rate = validate_rate(finite(row.text("y_bits")?)?, compounding)
        .map_err(|e| format!("{}: unexpected quote failure {e:?}", row.id()))?;
    let time = finite(row.text("t_bits")?)?;
    let amount = finite(row.text("amount_bits")?)?;
    let observation = observe_present_value_at(rate, compounding, amount, time)
        .map_err(|e| format!("{}: unexpected observation failure {e:?}", row.id()))?;
    let mut expected = BTreeMap::new();
    for entry in row.text("detail")?.split(';') {
        let (key, value) = entry.split_once(':').ok_or("flow detail needs key:value")?;
        if expected.insert(key, value).is_some() {
            return Err("duplicate flow detail key".into());
        }
    }
    assert_eq!(
        expected.keys().copied().collect::<Vec<_>>(),
        ["branch", "d1", "d2", "moment", "pv"]
    );
    let branch = match observation.branch {
        ActiveBranch::Simple => "Simple".to_owned(),
        ActiveBranch::Continuous => "Continuous".to_owned(),
        ActiveBranch::Periodic(count) => format!("Periodic:{count}"),
    };
    assert_eq!(branch, expected["branch"], "{}", row.id());
    let pv = observation.present_value;
    compare_bounds(row.id(), "pv", pv, expected["pv"])?;
    let (first, second, moment) = if time == 0.0 || pv == 0.0 {
        // Accepted risk traversal skips nonzero-term construction for zero PV/time.
        let zero = subtract_risk_sums(RiskWide::ZERO, RiskWide::ZERO, "time weighted amount")?;
        let value = round_raw_risk(zero, "time weighted amount")?;
        (value, value, value)
    } else {
        let negative = pv.is_sign_negative();
        let moment = raw_contribution(&[pv, time], &[], 0, negative, "time weighted amount")?;
        let (first, second) = match observation.branch {
            ActiveBranch::Simple => {
                let q = observation.simple_compound_factor;
                (
                    raw_contribution(&[pv, time], &[q], 0, !negative, "price derivative")?,
                    raw_contribution(
                        &[pv, time, time],
                        &[q, q],
                        1,
                        negative,
                        "price second derivative",
                    )?,
                )
            }
            ActiveBranch::Continuous => (
                raw_contribution(&[pv, time], &[], 0, !negative, "price derivative")?,
                raw_contribution(
                    &[pv, time, time],
                    &[],
                    0,
                    negative,
                    "price second derivative",
                )?,
            ),
            ActiveBranch::Periodic(count) => {
                let m = f64::from(count);
                let delta = m + rate;
                let w = time + 1.0 / m;
                (
                    raw_contribution(&[pv, time, m], &[delta], 0, !negative, "price derivative")?,
                    raw_contribution(
                        &[pv, time, w, m, m],
                        &[delta, delta],
                        0,
                        negative,
                        "price second derivative",
                    )?,
                )
            }
        };
        (first, second, moment)
    };
    for (label, value) in [("d1", first), ("d2", second), ("moment", moment)] {
        compare_bounds(row.id(), label, value, expected[label])?;
    }
    Ok(())
}

fn check_unused(row: &Record<'_>) -> CheckResult {
    let active: &[&str] = match row.text("op")? {
        "residual" => &[
            "dirty_bits",
            "target_bits",
            "scale",
            "status",
            "expected_bits",
            "detail",
        ],
        "term" => &[
            "numerators",
            "denominators",
            "pow2",
            "negative",
            "status",
            "sign",
            "significand",
            "exponent",
        ],
        "sum" => &[
            "recipes",
            "dirty_bits",
            "status",
            "sign",
            "significand",
            "exponent",
            "expected_bits",
            "error_kind",
            "error_phase",
            "error_index",
            "detail",
        ],
        "flow" => &[
            "comp",
            "y_bits",
            "t_bits",
            "amount_bits",
            "status",
            "detail",
            "error_kind",
            "error_phase",
            "error_index",
        ],
        _ => return Err("unknown kernel operation".into()),
    };
    for name in COLUMNS.into_iter().skip(3) {
        if !active.contains(&name) {
            assert_eq!(row.text(name)?, "-", "{}: unused {name}", row.id());
        }
    }
    Ok(())
}

fn bump(census: &mut BTreeMap<String, usize>, key: &str) {
    *census.entry(key.to_owned()).or_default() += 1;
}

fn check_census(fixture: &Fixture, key: &str, actual: &BTreeMap<String, usize>) -> CheckResult {
    if let Some(value) = fixture.header(key) {
        let mut expected = BTreeMap::new();
        for item in value.split(';') {
            let (name, count) = item.split_once('=').ok_or("malformed kernel census")?;
            if expected
                .insert(name.to_owned(), count.parse::<usize>()?)
                .is_some()
            {
                return Err("duplicate kernel census key".into());
            }
        }
        assert_eq!(actual, &expected, "{key}");
    }
    Ok(())
}

#[test]
fn independent_extended_kernel_matches_all_rows() -> CheckResult {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/bonds/yield-risk-extended.fixture",
    ))?;
    assert_eq!(fixture.columns(), &COLUMNS.map(str::to_owned));
    let mut operations = BTreeMap::new();
    let mut families = BTreeMap::new();
    let mut statuses = BTreeMap::new();
    let mut identifiers = BTreeSet::new();
    let mut visited = 0;
    for row in fixture.records() {
        assert!(identifiers.insert(row.id()));
        check_unused(&row)?;
        match row.text("op")? {
            "residual" => residual(&row)?,
            "term" => extended_term(&row)?,
            "sum" => extended_sum(&row)?,
            "flow" => flow(&row)?,
            _ => return Err("unrecognized kernel operation; refusing to skip fixture row".into()),
        }
        visited += 1;
        bump(&mut operations, row.text("op")?);
        bump(&mut families, row.text("family")?);
        bump(&mut statuses, row.text("status")?);
    }
    assert_eq!(visited, fixture.record_count());
    assert_eq!(
        visited,
        fixture
            .header("record-count")
            .ok_or("kernel record count missing")?
            .parse::<usize>()?
    );
    assert!(visited > 0);
    check_census(&fixture, "operation-counts", &operations)?;
    check_census(&fixture, "family-counts", &families)?;
    check_census(&fixture, "status-counts", &statuses)?;
    if let Some(expected) = fixture.header("census") {
        let actual = format!(
            "{{{}}}",
            operations
                .iter()
                .map(|(op, count)| format!("\"{op}\":{count}"))
                .collect::<Vec<_>>()
                .join(",")
        );
        assert_eq!(actual, expected, "kernel operation census");
    }
    Ok(())
}
