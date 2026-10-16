//! Independent exact decimal oracle for the phase A amount foundation.
//!
//! FR-AMT-01..12: every fixture row is evaluated through the public API and
//! compared at TOL-E0, including every structured error field. The generator
//! derived its expectations from the frozen contract with Python integer,
//! Fraction and Decimal arithmetic, without Rust output.

mod support;

use sabiquant::amounts::{
    AmountError, AmountQuantum, AmountRounding, DecimalAmount, exact_amount_sum,
};
use sabiquant::error::IdentifierError;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::error::Error;
use std::str::FromStr;
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

/// One evaluated public outcome.
enum Outcome {
    Amount(DecimalAmount),
    Bool(bool),
    Order(Ordering),
    Mode(AmountRounding),
    AmountFailure(AmountError),
    IdentifierFailure(IdentifierError),
}

fn raw_cell(cell: &str) -> TestResult<(i128, u8)> {
    let (coefficient, scale) = cell.split_once(':').ok_or("decimal cell needs ':'")?;
    Ok((coefficient.parse()?, scale.parse()?))
}

fn amount(cell: &str) -> TestResult<DecimalAmount> {
    let (coefficient, scale) = raw_cell(cell)?;
    Ok(DecimalAmount::new(coefficient, scale)?)
}

fn quantum(cell: &str) -> TestResult<AmountQuantum> {
    let (coefficient, scale) = raw_cell(cell)?;
    Ok(AmountQuantum::new(coefficient, scale)?)
}

fn hex_decode(text: &str) -> TestResult<String> {
    if text == "_" {
        return Ok(String::new());
    }
    if !text.len().is_multiple_of(2) {
        return Err("odd hex length".into());
    }
    let bytes: TestResult<Vec<u8>> = (0..text.len())
        .step_by(2)
        .map(|index| Ok(u8::from_str_radix(&text[index..index + 2], 16)?))
        .collect();
    Ok(String::from_utf8(bytes?)?)
}

/// Lowercase UTF-8 hex; the fixture spells empty text as `_`.
fn hex_encode(text: &str) -> String {
    if text.is_empty() {
        return "_".to_owned();
    }
    text.bytes().map(|byte| format!("{byte:02x}")).collect()
}

fn cell(value: DecimalAmount) -> String {
    format!("{}:{}", value.coefficient(), value.scale())
}

fn evaluate(row: &Record<'_>) -> TestResult<Outcome> {
    let op = row.text("op")?;
    let wrap = |result: Result<DecimalAmount, AmountError>| match result {
        Ok(value) => Outcome::Amount(value),
        Err(error) => Outcome::AmountFailure(error),
    };
    let mode = || -> TestResult<AmountRounding> { Ok(row.text("mode")?.parse()?) };
    Ok(match op {
        "new" => {
            let (coefficient, scale) = raw_cell(row.text("lhs")?)?;
            wrap(DecimalAmount::new(coefficient, scale))
        }
        "quantum" => {
            let (coefficient, scale) = raw_cell(row.text("lhs")?)?;
            wrap(AmountQuantum::new(coefficient, scale).map(AmountQuantum::amount))
        }
        "parse" => wrap(DecimalAmount::from_str(&hex_decode(row.text("texthex")?)?)),
        "mode" => match AmountRounding::from_str(&hex_decode(row.text("texthex")?)?) {
            Ok(value) => Outcome::Mode(value),
            Err(error) => Outcome::IdentifierFailure(error),
        },
        "add" => wrap(amount(row.text("lhs")?)?.checked_add(amount(row.text("rhs")?)?)),
        "sub" => wrap(amount(row.text("lhs")?)?.checked_sub(amount(row.text("rhs")?)?)),
        "mul" => wrap(amount(row.text("lhs")?)?.checked_mul(amount(row.text("rhs")?)?)),
        "neg" => wrap(amount(row.text("lhs")?)?.checked_neg()),
        "sum" => {
            let terms = row.text("terms")?;
            let values: TestResult<Vec<DecimalAmount>> = if terms == "-" {
                Ok(Vec::new())
            } else {
                terms.split(';').map(amount).collect()
            };
            wrap(exact_amount_sum(&values?))
        }
        "cmp" => Outcome::Order(amount(row.text("lhs")?)?.cmp(&amount(row.text("rhs")?)?)),
        "multiple" => {
            Outcome::Bool(amount(row.text("lhs")?)?.is_multiple_of(quantum(row.text("quantum")?)?))
        }
        "quantize" => {
            wrap(amount(row.text("lhs")?)?.quantize(quantum(row.text("quantum")?)?, mode()?))
        }
        "ratio" => wrap(DecimalAmount::from_ratio(
            row.parse("numerator")?,
            row.parse("denominator")?,
            quantum(row.text("quantum")?)?,
            mode()?,
        )),
        "multiply_ratio" => wrap(amount(row.text("lhs")?)?.multiply_ratio(
            row.parse("numerator")?,
            row.parse("denominator")?,
            quantum(row.text("quantum")?)?,
            mode()?,
        )),
        "multiply_divide" => wrap(amount(row.text("lhs")?)?.multiply_divide(
            amount(row.text("rhs")?)?,
            row.parse("denominator")?,
            quantum(row.text("quantum")?)?,
            mode()?,
        )),
        other => return Err(format!("unknown oracle operation {other}").into()),
    })
}

/// Every structured field of an error, as the fixture's `key=value` map.
fn error_fields(error: &AmountError) -> TestResult<BTreeMap<&'static str, String>> {
    let mut fields = BTreeMap::new();
    match error {
        AmountError::TextTooLong {
            length, maximum, ..
        } => {
            fields.insert("length", length.to_string());
            fields.insert("maximum", maximum.to_string());
        }
        AmountError::InvalidSyntax { index, reason, .. } => {
            fields.insert("index", index.to_string());
            fields.insert("reason", hex_encode(reason));
        }
        AmountError::InvalidScale { scale, maximum, .. } => {
            fields.insert("scale", scale.to_string());
            fields.insert("maximum", maximum.to_string());
        }
        AmountError::InvalidQuantum {
            coefficient, scale, ..
        } => {
            fields.insert("coefficient", coefficient.to_string());
            fields.insert("scale", scale.to_string());
        }
        AmountError::ZeroDenominator { operation, .. } | AmountError::Inexact { operation, .. } => {
            fields.insert("operation", (*operation).to_owned());
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
        other => return Err(format!("unclassified amount error {other:?}").into()),
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

fn check(row: &Record<'_>, outcome: &Outcome) -> TestResult<&'static str> {
    let id = row.id();
    let expected = row.text("expected")?;
    let context = row.text("context")?;
    let output = row.text("output")?;
    let class = match outcome {
        Outcome::Amount(value) => {
            assert_eq!(expected, "ok", "{id}");
            assert_eq!(cell(*value), output, "{id}");
            let display = value.to_string();
            assert_eq!(context, format!("text={}", hex_encode(&display)), "{id}");
            // Canonical text round-trips to the identical normalized value.
            assert_eq!(display.parse::<DecimalAmount>()?, *value, "{id}");
            "ok"
        }
        Outcome::Bool(value) => {
            assert_eq!((expected, context), ("bool", "-"), "{id}");
            assert_eq!(output, if *value { "1" } else { "0" }, "{id}");
            "bool"
        }
        Outcome::Order(order) => {
            assert_eq!((expected, context), ("cmp", "-"), "{id}");
            let text = match order {
                Ordering::Less => "-1",
                Ordering::Equal => "0",
                Ordering::Greater => "1",
            };
            assert_eq!(output, text, "{id}");
            "cmp"
        }
        Outcome::Mode(mode) => {
            assert_eq!(expected, "mode", "{id}");
            assert_eq!(output, mode.canonical_identifier(), "{id}");
            assert_eq!(output, mode.to_string(), "{id}");
            "mode"
        }
        Outcome::AmountFailure(error) => {
            assert_eq!(expected, error.kind(), "{id}: {error}");
            assert_eq!(output, "-", "{id}");
            let actual = error_fields(error)?;
            let recorded = context_map(context)?;
            let actual: BTreeMap<&str, String> = actual.into_iter().collect();
            assert_eq!(actual, recorded, "{id}");
            "error"
        }
        Outcome::IdentifierFailure(error) => {
            assert_eq!(expected, "identifier.unknown", "{id}");
            assert_eq!(error.kind(), expected, "{id}");
            let IdentifierError::Unknown { concept, text, .. } = error else {
                return Err(format!("{id}: unexpected identifier error {error:?}").into());
            };
            let recorded = context_map(context)?;
            assert_eq!(recorded.get("concept"), Some(&hex_encode(concept)), "{id}");
            assert_eq!(recorded.get("text"), Some(&hex_encode(text)), "{id}");
            assert_eq!(recorded.len(), 2, "{id}");
            "error"
        }
    };
    Ok(class)
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

/// FR-AMT-01..12: all 277,577 independent oracle rows agree exactly in
/// canonical values, Display text, predicates, comparisons, mode identifiers
/// and every structured error field. Operation and expectation censuses are
/// tallied over these row-validated outcomes; error operations come from the
/// actual error values.
#[test]
fn independent_amount_oracle_matches_every_row_exactly() -> TestResult {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/amounts/oracles.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 277_577);
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
        check(&row, &outcome)?;
        bump(&mut operations, row.text("op")?);
        let expected = row.text("expected")?;
        bump(&mut expectations, expected);
        if let Outcome::AmountFailure(error) = &outcome {
            let fields = error_fields(error)?;
            if let Some(operation) = fields.get("operation") {
                bump(&mut error_operations, operation);
            }
        }
    }
    assert_eq!(operations, header_counts(&fixture, "operation-counts")?);
    assert_eq!(expectations, header_counts(&fixture, "expectation-counts")?);
    assert_eq!(
        error_operations,
        header_counts(&fixture, "error-operation-counts")?
    );
    assert_eq!(expectations["ok"], 131_947);
    Ok(())
}
