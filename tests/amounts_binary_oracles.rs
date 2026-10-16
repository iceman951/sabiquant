//! Independent oracle for the phase B binary64 boundary of exact amounts.
//!
//! FR-BIN-02..06: every fixture row is evaluated through the public API and
//! compared at TOL-E0, including binary64 bit patterns and every structured
//! error field. The generator decoded IEEE 754 bit fields with Python struct
//! and integer/Fraction arithmetic, without Rust output.

mod support;

use sabiquant::amounts::{AmountError, AmountQuantum, AmountRounding, DecimalAmount};
use sabiquant::error::NonFinite;
use std::collections::BTreeMap;
use std::error::Error;
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

/// One evaluated public outcome.
enum Outcome {
    Amount(DecimalAmount),
    Bits(u64),
    Failure(AmountError),
}

fn decimal_cell(cell: &str) -> TestResult<(i128, u8)> {
    let (coefficient, scale) = cell.split_once(':').ok_or("decimal cell needs ':'")?;
    Ok((coefficient.parse()?, scale.parse()?))
}

fn binary64(cell: &str) -> TestResult<u64> {
    let digits = cell.strip_prefix("0x").ok_or("bits need a 0x prefix")?;
    if digits.len() != 16 {
        return Err("bits need 16 hex digits".into());
    }
    Ok(u64::from_str_radix(digits, 16)?)
}

fn evaluate(row: &Record<'_>) -> TestResult<Outcome> {
    let wrap = |result: Result<DecimalAmount, AmountError>| match result {
        Ok(value) => Outcome::Amount(value),
        Err(error) => Outcome::Failure(error),
    };
    let amount = || -> TestResult<DecimalAmount> {
        let (coefficient, scale) = decimal_cell(row.text("amount")?)?;
        Ok(DecimalAmount::new(coefficient, scale)?)
    };
    Ok(match row.text("op")? {
        "from_exact" => wrap(DecimalAmount::from_binary64_exact(f64::from_bits(
            binary64(row.text("bits")?)?,
        ))),
        "from_rounded" => {
            let (coefficient, scale) = decimal_cell(row.text("quantum")?)?;
            wrap(DecimalAmount::from_binary64_rounded(
                f64::from_bits(binary64(row.text("bits")?)?),
                AmountQuantum::new(coefficient, scale)?,
                row.text("mode")?.parse::<AmountRounding>()?,
            ))
        }
        "to_exact" => match amount()?.to_binary64_exact() {
            Ok(value) => Outcome::Bits(value.to_bits()),
            Err(error) => Outcome::Failure(error),
        },
        "to_nearest" => Outcome::Bits(amount()?.to_binary64_nearest().to_bits()),
        other => return Err(format!("unknown oracle operation {other}").into()),
    })
}

/// Every structured field of an error, as the fixture's `key=value` map.
fn error_fields(error: &AmountError) -> TestResult<BTreeMap<&'static str, String>> {
    let mut fields = BTreeMap::new();
    match error {
        AmountError::NonFinite {
            operation, value, ..
        } => {
            fields.insert("operation", (*operation).to_owned());
            let class = match value {
                NonFinite::NaN => "NaN",
                NonFinite::PositiveInfinity => "PositiveInfinity",
                NonFinite::NegativeInfinity => "NegativeInfinity",
            };
            fields.insert("class", class.to_owned());
        }
        AmountError::ResultScale {
            operation,
            scale,
            maximum,
            ..
        } => {
            fields.insert("operation", (*operation).to_owned());
            fields.insert("scale", scale.to_string());
            fields.insert("maximum", maximum.to_string());
        }
        AmountError::OutOfRange {
            operation,
            negative,
            ..
        } => {
            fields.insert("operation", (*operation).to_owned());
            fields.insert("negative", u8::from(*negative).to_string());
        }
        AmountError::Inexact { operation, .. } => {
            fields.insert("operation", (*operation).to_owned());
        }
        other => return Err(format!("unexpected amount error {other:?}").into()),
    }
    Ok(fields)
}

fn context_map(context: &str) -> TestResult<BTreeMap<&str, String>> {
    let mut map = BTreeMap::new();
    for item in context.split(',') {
        let (key, value) = item.split_once('=').ok_or("context needs key=value")?;
        map.insert(key, value.to_owned());
    }
    Ok(map)
}

fn check(row: &Record<'_>, outcome: &Outcome) -> TestResult<Option<&'static str>> {
    let id = row.id();
    let expected = row.text("expected")?;
    let context = row.text("context")?;
    let output = row.text("output")?;
    match outcome {
        Outcome::Amount(value) => {
            assert_eq!((expected, context), ("ok", "-"), "{id}");
            assert_eq!(
                format!("{}:{}", value.coefficient(), value.scale()),
                output,
                "{id}"
            );
            Ok(None)
        }
        Outcome::Bits(bits) => {
            assert_eq!((expected, context), ("ok", "-"), "{id}");
            assert_eq!(format!("0x{bits:016x}"), output, "{id}");
            // An amount never converts to a non-finite or subnormal binary64.
            let value = f64::from_bits(*bits);
            assert!(value == 0.0 || value.is_normal(), "{id}");
            Ok(None)
        }
        Outcome::Failure(error) => {
            assert_eq!(expected, error.kind(), "{id}: {error}");
            assert_eq!(output, "-", "{id}");
            let actual = error_fields(error)?;
            let recorded = context_map(context)?;
            let actual: BTreeMap<&str, String> = actual.into_iter().collect();
            assert_eq!(actual, recorded, "{id}");
            // Phase B errors, like phase A, carry no nested source.
            assert!(error.source().is_none(), "{id}");
            let operation = match error {
                AmountError::NonFinite { operation, .. }
                | AmountError::ResultScale { operation, .. }
                | AmountError::OutOfRange { operation, .. }
                | AmountError::Inexact { operation, .. } => *operation,
                other => return Err(format!("{id}: unexpected {other:?}").into()),
            };
            Ok(Some(operation))
        }
    }
}

fn header_counts(fixture: &Fixture, key: &str) -> TestResult<BTreeMap<String, usize>> {
    let text = fixture.header(key).ok_or("missing census header")?;
    let mut counts = BTreeMap::new();
    for item in text.split(';') {
        let (name, count) = item.split_once('=').ok_or("malformed census entry")?;
        counts.insert(name.to_owned(), count.parse()?);
    }
    Ok(counts)
}

fn bump(map: &mut BTreeMap<String, usize>, key: &str) {
    *map.entry(key.to_owned()).or_default() += 1;
}

/// FR-BIN-02..06: all 196,773 independent oracle rows agree exactly in
/// canonical amounts, binary64 bit patterns and every structured error
/// field. Operation and expectation censuses are tallied over these
/// row-validated outcomes; error operations come from the actual errors.
#[test]
fn independent_binary_boundary_oracle_matches_every_row_exactly() -> TestResult {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/amounts/binary-boundary-oracles.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 196_773);
    assert!(
        fixture
            .header("tolerance")
            .ok_or("tolerance")?
            .starts_with("TOL-E0")
    );
    assert!(
        fixture
            .header("reference")
            .ok_or("reference")?
            .contains("no Rust or QuantLib output")
    );
    let mut operations = BTreeMap::new();
    let mut expectations = BTreeMap::new();
    let mut error_operations = BTreeMap::new();
    for row in fixture.records() {
        let outcome = evaluate(&row)?;
        if let Some(operation) = check(&row, &outcome)? {
            bump(&mut error_operations, operation);
        }
        bump(&mut operations, row.text("op")?);
        bump(&mut expectations, row.text("expected")?);
    }
    assert_eq!(operations, header_counts(&fixture, "operation-counts")?);
    assert_eq!(expectations, header_counts(&fixture, "expectation-counts")?);
    assert_eq!(
        error_operations,
        header_counts(&fixture, "error-operation-counts")?
    );
    assert_eq!(expectations["ok"], 113_622);
    Ok(())
}
