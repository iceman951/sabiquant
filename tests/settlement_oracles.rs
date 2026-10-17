//! FR-SETTLE-01..16: independent frozen exact settlement oracle.
//!
//! Expectations were generated from the frozen contract without settlement
//! production source or outputs. Every record compares exact public results,
//! structured errors, evidence bounds, provenance and complete receipt policy.

mod support;

use sabiquant::amounts::{AmountError, AmountQuantum, AmountRounding, DecimalAmount};
use sabiquant::error::{IdentifierError, NonFinite};
use sabiquant::settlement::{
    CashAggregation, SettlementDecision, SettlementError, SettlementEvidence, SettlementGuarantee,
    SettlementInstruction, SettlementRounding,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
type Fields = BTreeMap<String, String>;

const COLUMNS: [&str; 17] = [
    "id",
    "profile",
    "op",
    "recipe",
    "nominal",
    "primary",
    "secondary",
    "aggregation",
    "price_mode",
    "accrued_mode",
    "quantum",
    "cash_mode",
    "expected",
    "context",
    "lower",
    "upper",
    "guarantee",
];
const INPUT_COLUMNS: [&str; 10] = [
    "op",
    "recipe",
    "nominal",
    "primary",
    "secondary",
    "aggregation",
    "price_mode",
    "accrued_mode",
    "quantum",
    "cash_mode",
];

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn source_pin(fixture: &Fixture, key: &str, bytes: &[u8], frozen: u64) {
    assert_eq!(fnv64(bytes), frozen, "{key}: current bytes changed");
    assert_eq!(fixture.header(key), Some(format!("{frozen:016x}").as_str()));
}

fn fixture() -> TestResult<Fixture> {
    #[expect(
        clippy::disallowed_methods,
        reason = "test reads frozen oracle bytes to fingerprint its exact body"
    )]
    let text = std::fs::read_to_string(Fixture::repository_path(
        "validation/fixtures/settlement/oracles.fixture",
    ))?;
    // Pin metadata too, including the generator's independent cross-check and
    // width censuses that cannot be recomputed merely by consuming records.
    assert_eq!(fnv64(text.as_bytes()), 0x1d74_fd3a_8cd8_ce4d);
    let (_, body) = text.split_once("\n\n").ok_or("missing fixture separator")?;
    let (_, records) = body.split_once('\n').ok_or("missing column line")?;
    assert_eq!(fnv64(body.as_bytes()), 0x1477_ce8b_9fff_859b);
    assert_eq!(fnv64(records.as_bytes()), 0x7969_a6dd_eff2_dec6);
    let fixture = Fixture::parse(&text)?;
    assert_eq!(fixture.columns(), &COLUMNS.map(str::to_owned));
    for (key, value) in [
        ("fixture-set", "EXACT-SETTLEMENT-PHASE-C-ORACLE"),
        ("domain", "settlement"),
        ("body-fnv64", "1477ce8b9fff859b"),
        ("records-fnv64", "7969a6ddeff2dec6"),
        (
            "body-sha256",
            "577cf4f455eae23725b5dc96abbf56588a0435625a3b729ca1eb6ea0495089fd",
        ),
        (
            "records-sha256",
            "8e9d1fac93e94234cab2541a1b0151e2dd011133b4166ef3bd1de1807979dc46",
        ),
        (
            "generator-sha256",
            "dda662320a74b485438b23be7a06124567d0dd6794fc85584006aae3c5043a69",
        ),
        (
            "adopted-contract-sha256",
            "47f1b08ff1ca646c502c90b1e7cb380fa4d74a4763b3aa9e32d729adc89faeba",
        ),
        (
            "requirements-rows-sha256",
            "f500bb6926d8f38f789817e342397719543d7ffd8f73dc1f114b640da770f637",
        ),
        (
            "reference-version",
            "Python >=3.9; Decimal precisions1536,2048",
        ),
        ("record-count", "154241"),
        ("unique-input-count", "154063"),
    ] {
        assert_eq!(fixture.header(key), Some(value), "{key}");
    }
    source_pin(
        &fixture,
        "generator-fnv64",
        include_bytes!("../validation/generator/settlement_oracles.py"),
        0x58b8_eb05_3032_5482,
    );
    source_pin(
        &fixture,
        "adopted-contract-fnv64",
        include_bytes!("../docs/M8_SETTLEMENT_CONTRACT.md"),
        0x63fb_87ff_5504_b2b1,
    );
    let requirements: Vec<_> = include_str!("../docs/REQUIREMENTS.md")
        .lines()
        .filter(|line| line.starts_with("| FR-SETTLE-"))
        .collect();
    assert_eq!(requirements.len(), 16);
    source_pin(
        &fixture,
        "requirements-rows-fnv64",
        requirements.join("\n").as_bytes(),
        0xceab_07dc_dae7_b6a4,
    );
    assert_eq!(fixture.record_count(), 154_241);
    assert!(
        fixture
            .header("tolerance")
            .ok_or("missing tolerance")?
            .starts_with("TOL-E0")
    );
    assert!(
        fixture
            .header("reference")
            .ok_or("missing reference")?
            .contains("no Rust or QuantLib output")
    );
    Ok(fixture)
}

fn raw_cell(text: &str) -> TestResult<(i128, u8)> {
    let (coefficient, scale) = text
        .split_once(':')
        .ok_or("decimal needs coefficient:scale")?;
    Ok((coefficient.parse()?, scale.parse()?))
}

fn amount(text: &str) -> TestResult<DecimalAmount> {
    let (c, s) = raw_cell(text)?;
    Ok(DecimalAmount::new(c, s)?)
}

fn cell(value: DecimalAmount) -> String {
    format!("{}:{}", value.coefficient(), value.scale())
}

fn quantum(text: &str) -> TestResult<AmountQuantum> {
    let (c, s) = raw_cell(text)?;
    Ok(AmountQuantum::new(c, s)?)
}

fn hex_decode(text: &str) -> TestResult<String> {
    if text == "_" {
        return Ok(String::new());
    }
    if !text.len().is_multiple_of(2) {
        return Err("odd UTF8 hex length".into());
    }
    let bytes: TestResult<Vec<_>> = (0..text.len())
        .step_by(2)
        .map(|i| Ok(u8::from_str_radix(&text[i..i + 2], 16)?))
        .collect();
    Ok(String::from_utf8(bytes?)?)
}

fn hex_encode(text: &str) -> String {
    if text.is_empty() {
        "_".to_owned()
    } else {
        text.bytes().map(|b| format!("{b:02x}")).collect()
    }
}

fn canonical_bits(raw: u64) -> u64 {
    if raw & 0x7fff_ffff_ffff_ffff == 0 {
        0
    } else {
        raw
    }
}

fn evidence(text: &str) -> TestResult<Result<SettlementEvidence, SettlementError>> {
    let parts: Vec<_> = text.split(':').collect();
    let decimal = |start| -> TestResult<DecimalAmount> {
        amount(&format!("{}:{}", parts[start], parts[start + 1]))
    };
    let binary =
        |start| -> TestResult<f64> { Ok(f64::from_bits(u64::from_str_radix(parts[start], 16)?)) };
    Ok(match parts[0] {
        "d" if parts.len() == 3 => Ok(SettlementEvidence::exact_decimal(decimal(1)?)),
        "di" if parts.len() == 5 => {
            SettlementEvidence::declared_decimal_interval(decimal(1)?, decimal(3)?)
        }
        "b" if parts.len() == 2 => SettlementEvidence::supplied_binary64(binary(1)?),
        "bi" if parts.len() == 3 => {
            SettlementEvidence::declared_binary64_interval(binary(1)?, binary(2)?)
        }
        _ => return Err(format!("invalid evidence cell {text}").into()),
    })
}

fn check_evidence(text: &str, value: SettlementEvidence) -> TestResult {
    let parts: Vec<_> = text.split(':').collect();
    let declared = matches!(parts[0], "di" | "bi");
    assert_eq!(value.is_declared_interval(), declared, "{text}");
    if matches!(parts[0], "d" | "di") {
        let lower = amount(&format!("{}:{}", parts[1], parts[2]))?;
        let upper = if declared {
            amount(&format!("{}:{}", parts[3], parts[4]))?
        } else {
            lower
        };
        assert_eq!(value.decimal_bounds(), Some((lower, upper)), "{text}");
        assert!(value.binary64_bounds().is_none());
    } else {
        let lower = canonical_bits(u64::from_str_radix(parts[1], 16)?);
        let upper = if declared {
            canonical_bits(u64::from_str_radix(parts[2], 16)?)
        } else {
            lower
        };
        let actual = value.binary64_bounds().ok_or("binary bounds absent")?;
        assert_eq!(
            (actual.0.to_bits(), actual.1.to_bits()),
            (lower, upper),
            "{text}"
        );
        assert!(value.decimal_bounds().is_none());
    }
    assert_eq!(evidence(text)??, value);
    Ok(())
}

#[expect(
    clippy::large_enum_variant,
    reason = "test retains Copy instruction and complete decision together for receipt identity checks"
)]
enum Outcome {
    Evidence(SettlementEvidence),
    Decision(SettlementInstruction, SettlementDecision),
    Aggregation(CashAggregation),
    Guarantee(SettlementGuarantee),
    SettlementFailure(SettlementError),
    IdentifierFailure(IdentifierError),
}

fn instruction(row: &Record<'_>) -> TestResult<Result<SettlementInstruction, SettlementError>> {
    let nominal = amount(row.text("nominal")?)?;
    let primary = evidence(row.text("primary")?)??;
    check_evidence(row.text("primary")?, primary)?;
    let mode: AmountRounding = row.text("cash_mode")?.parse()?;
    assert_eq!(mode.to_string(), row.text("cash_mode")?);
    let policy = SettlementRounding::new(quantum(row.text("quantum")?)?, mode);
    let recipe = row.text("recipe")?;
    if recipe == "DirtyPer100" {
        return Ok(SettlementInstruction::dirty_per_100(
            nominal, primary, policy,
        ));
    }
    if recipe == "CouponPer1000" {
        return Ok(SettlementInstruction::coupon_per_1000(
            nominal, primary, policy,
        ));
    }
    let secondary = evidence(row.text("secondary")?)??;
    check_evidence(row.text("secondary")?, secondary)?;
    let aggregation: CashAggregation = row.text("aggregation")?.parse()?;
    Ok(match recipe {
        "CleanAccruedPer100" => SettlementInstruction::clean_accrued_per_100(
            nominal,
            primary,
            secondary,
            aggregation,
            policy,
        ),
        "StagedPrice100Accrued1000" => SettlementInstruction::staged_price100_accrued1000(
            nominal,
            primary,
            secondary,
            row.text("price_mode")?.parse()?,
            row.text("accrued_mode")?.parse()?,
            aggregation,
            policy,
        ),
        _ => return Err(format!("unknown recipe {recipe}").into()),
    })
}

fn check_instruction(row: &Record<'_>, value: SettlementInstruction) -> TestResult {
    assert_eq!(
        value.recipe_identifier(),
        row.text("recipe")?,
        "{}",
        row.id()
    );
    assert_eq!(value.nominal(), amount(row.text("nominal")?)?);
    assert_eq!(value.primary(), evidence(row.text("primary")?)??);
    assert_eq!(
        value.cash_rounding().quantum(),
        quantum(row.text("quantum")?)?
    );
    assert_eq!(
        value.cash_rounding().mode().canonical_identifier(),
        row.text("cash_mode")?
    );
    assert_eq!(
        value.secondary(),
        if row.text("secondary")? == "-" {
            None
        } else {
            Some(evidence(row.text("secondary")?)??)
        }
    );
    assert_eq!(
        value.aggregation(),
        if row.text("aggregation")? == "-" {
            None
        } else {
            Some(row.text("aggregation")?.parse()?)
        }
    );
    if row.text("recipe")? == "StagedPrice100Accrued1000" {
        for (actual, source_cell, scale) in [
            (value.source_price_rounding(), "price_mode", 6),
            (value.source_accrued_rounding(), "accrued_mode", 5),
        ] {
            let policy = actual.ok_or("missing source policy")?;
            assert_eq!(policy.quantum(), AmountQuantum::new(1, scale)?);
            assert_eq!(policy.mode().canonical_identifier(), row.text(source_cell)?);
            assert_eq!(policy.mode().to_string(), row.text(source_cell)?);
        }
    } else {
        assert_eq!(value.source_price_rounding(), None);
        assert_eq!(value.source_accrued_rounding(), None);
    }
    Ok(())
}

fn evaluate(row: &Record<'_>) -> TestResult<Outcome> {
    Ok(match row.text("op")? {
        "evidence" => match evidence(row.text("primary")?)? {
            Ok(value) => Outcome::Evidence(value),
            Err(error) => Outcome::SettlementFailure(error),
        },
        "evaluate" => match instruction(row)? {
            Ok(value) => {
                check_instruction(row, value)?;
                match value.evaluate() {
                    Ok(decision) => Outcome::Decision(value, decision),
                    Err(error) => Outcome::SettlementFailure(error),
                }
            }
            Err(error) => Outcome::SettlementFailure(error),
        },
        op @ ("aggregation" | "guarantee") => {
            let text = hex_decode(
                row.text("primary")?
                    .strip_prefix("x:")
                    .ok_or("identifier needs x:")?,
            )?;
            if op == "aggregation" {
                match text.parse::<CashAggregation>() {
                    Ok(value) => Outcome::Aggregation(value),
                    Err(error) => Outcome::IdentifierFailure(error),
                }
            } else {
                match text.parse::<SettlementGuarantee>() {
                    Ok(value) => Outcome::Guarantee(value),
                    Err(error) => Outcome::IdentifierFailure(error),
                }
            }
        }
        other => return Err(format!("unknown operation {other}").into()),
    })
}

fn context_map(text: &str) -> TestResult<Fields> {
    let mut fields = Fields::new();
    if text != "-" {
        for item in text.split(',') {
            let (key, value) = item.split_once('=').ok_or("context needs key=value")?;
            if key.is_empty()
                || value.is_empty()
                || fields.insert(key.to_owned(), value.to_owned()).is_some()
            {
                return Err(format!("invalid or duplicate context key {key}").into());
            }
        }
    }
    Ok(fields)
}

fn error_fields(error: &SettlementError) -> TestResult<Fields> {
    let mut fields = Fields::new();
    match error {
        SettlementError::NonPositiveNominal { nominal, .. } => {
            fields.insert("nominal".to_owned(), cell(*nominal));
        }
        SettlementError::DecimalIntervalOrder { lower, upper, .. } => {
            fields.insert("lower".to_owned(), cell(*lower));
            fields.insert("upper".to_owned(), cell(*upper));
        }
        SettlementError::BinaryIntervalOrder {
            lower_bits,
            upper_bits,
            ..
        } => {
            fields.insert("lower_bits".to_owned(), format!("{lower_bits:016x}"));
            fields.insert("upper_bits".to_owned(), format!("{upper_bits:016x}"));
        }
        SettlementError::NonFinite {
            endpoint, value, ..
        } => {
            fields.insert("endpoint".to_owned(), (*endpoint).to_owned());
            let name = match value {
                NonFinite::NaN => "NaN",
                NonFinite::PositiveInfinity => "PositiveInfinity",
                NonFinite::NegativeInfinity => "NegativeInfinity",
            };
            fields.insert("value".to_owned(), name.to_owned());
        }
        SettlementError::Arithmetic {
            stage,
            endpoint,
            cause,
            ..
        } => {
            fields.insert("stage".to_owned(), (*stage).to_owned());
            fields.insert("endpoint".to_owned(), (*endpoint).to_owned());
            fields.insert("cause".to_owned(), cause.kind().to_owned());
            let operation = match cause {
                AmountError::Inexact { operation, .. } => *operation,
                AmountError::OutOfRange {
                    operation,
                    negative,
                    ..
                } => {
                    fields.insert("negative".to_owned(), u8::from(*negative).to_string());
                    *operation
                }
                _ => return Err(format!("unreachable settlement amount cause {cause:?}").into()),
            };
            fields.insert("operation".to_owned(), operation.to_owned());
            assert_eq!(
                error.source().and_then(|e| e.downcast_ref::<AmountError>()),
                Some(cause)
            );
        }
        _ => return Err(format!("proof-excluded/unclassified settlement error {error:?}").into()),
    }
    if !matches!(error, SettlementError::Arithmetic { .. }) {
        assert!(error.source().is_none());
    }
    let root = sabiquant::error::Error::from(error.clone());
    assert_eq!(root.kind(), error.kind());
    assert_eq!(
        root.source()
            .and_then(|e| e.downcast_ref::<SettlementError>()),
        Some(error)
    );
    Ok(fields)
}

fn absent_outputs(row: &Record<'_>, columns: &[&str]) -> TestResult {
    for column in columns {
        assert_eq!(row.text(column)?, "-", "{}: unused {column}", row.id());
    }
    Ok(())
}

fn check(row: &Record<'_>, actual: &Outcome) -> TestResult {
    let id = row.id();
    let expected = row.text("expected")?;
    let context = context_map(row.text("context")?)?;
    match actual {
        Outcome::Evidence(value) => {
            assert_eq!(expected, "evidence", "{id}");
            check_evidence(row.text("primary")?, *value)?;
            let mut fields = Fields::new();
            fields.insert(
                "declared".to_owned(),
                u8::from(value.is_declared_interval()).to_string(),
            );
            if let Some((lower, upper)) = value.decimal_bounds() {
                fields.insert("kind".to_owned(), "decimal".to_owned());
                assert_eq!(cell(lower), row.text("lower")?, "{id}");
                assert_eq!(cell(upper), row.text("upper")?, "{id}");
            } else {
                fields.insert("kind".to_owned(), "binary64".to_owned());
                let (lower, upper) = value.binary64_bounds().ok_or("no evidence bounds")?;
                assert_eq!(
                    format!("{:016x}", lower.to_bits()),
                    row.text("lower")?,
                    "{id}"
                );
                assert_eq!(
                    format!("{:016x}", upper.to_bits()),
                    row.text("upper")?,
                    "{id}"
                );
            }
            assert_eq!(context, fields, "{id}");
            absent_outputs(row, &["guarantee"])?;
        }
        Outcome::Decision(instruction, decision) => match decision {
            SettlementDecision::Determined(receipt) => {
                assert_eq!(expected, "determined", "{id}");
                assert_eq!(cell(receipt.amount()), row.text("lower")?, "{id}");
                assert_eq!(receipt.instruction(), instruction, "{id}");
                check_instruction(row, *receipt.instruction())?;
                assert_eq!(
                    receipt.guarantee().canonical_identifier(),
                    row.text("guarantee")?,
                    "{id}"
                );
                assert_eq!(
                    receipt.guarantee().to_string(),
                    row.text("guarantee")?,
                    "{id}"
                );
                assert_eq!(
                    context,
                    Fields::from([("text".to_owned(), hex_encode(&receipt.amount().to_string()))]),
                    "{id}"
                );
                let evidence = [Some(instruction.primary()), instruction.secondary()];
                let guarantee = if evidence.iter().flatten().any(|e| e.is_declared_interval()) {
                    SettlementGuarantee::UniqueForDeclaredEnclosure
                } else if evidence
                    .iter()
                    .flatten()
                    .any(|e| e.binary64_bounds().is_some())
                {
                    SettlementGuarantee::ForSuppliedBinary64
                } else {
                    SettlementGuarantee::ForSuppliedDecimals
                };
                assert_eq!(receipt.guarantee(), guarantee, "{id}");
                absent_outputs(row, &["upper"])?;
            }
            SettlementDecision::QuantumAmbiguous {
                lower_amount,
                upper_amount,
                ..
            } => {
                assert_eq!(expected, "ambiguous", "{id}");
                assert_eq!(cell(*lower_amount), row.text("lower")?, "{id}");
                assert_eq!(cell(*upper_amount), row.text("upper")?, "{id}");
                assert!(lower_amount < upper_amount, "{id}");
                assert!(context.is_empty(), "{id}");
                absent_outputs(row, &["guarantee"])?;
            }
            SettlementDecision::ExactStageUnresolved { stage, .. } => {
                assert_eq!(expected, "exact_unresolved", "{id}");
                assert_eq!(
                    context,
                    Fields::from([("stage".to_owned(), (*stage).to_owned())]),
                    "{id}"
                );
                absent_outputs(row, &["lower", "upper", "guarantee"])?;
            }
            _ => return Err(format!("{id}: unknown settlement decision {decision:?}").into()),
        },
        Outcome::Aggregation(value) => {
            assert_eq!(expected, "identifier", "{id}");
            assert_eq!(value.canonical_identifier(), row.text("lower")?, "{id}");
            assert_eq!(value.to_string(), row.text("lower")?, "{id}");
            assert!(context.is_empty());
            absent_outputs(row, &["upper", "guarantee"])?;
        }
        Outcome::Guarantee(value) => {
            assert_eq!(expected, "identifier", "{id}");
            assert_eq!(value.canonical_identifier(), row.text("lower")?, "{id}");
            assert_eq!(value.to_string(), row.text("lower")?, "{id}");
            assert!(context.is_empty());
            absent_outputs(row, &["upper", "guarantee"])?;
        }
        Outcome::SettlementFailure(error) => {
            assert_eq!(expected, error.kind(), "{id}: {error:?}");
            assert_eq!(context, error_fields(error)?, "{id}");
            absent_outputs(row, &["lower", "upper", "guarantee"])?;
        }
        Outcome::IdentifierFailure(error) => {
            assert_eq!(expected, error.kind(), "{id}");
            let IdentifierError::Unknown { concept, text, .. } = error else {
                return Err("unexpected identifier error".into());
            };
            assert_eq!(
                context,
                Fields::from([
                    ("concept".to_owned(), hex_encode(concept)),
                    ("text".to_owned(), hex_encode(text))
                ]),
                "{id}"
            );
            absent_outputs(row, &["lower", "upper", "guarantee"])?;
        }
    }
    Ok(())
}

fn check_unused(row: &Record<'_>) -> TestResult {
    let mut active = vec!["primary"];
    match row.text("op")? {
        "evaluate" => {
            active.extend(["recipe", "nominal", "quantum", "cash_mode"]);
            match row.text("recipe")? {
                "DirtyPer100" | "CouponPer1000" => {}
                "CleanAccruedPer100" => active.extend(["secondary", "aggregation"]),
                "StagedPrice100Accrued1000" => {
                    active.extend(["secondary", "aggregation", "price_mode", "accrued_mode"])
                }
                _ => return Err("unknown recipe".into()),
            }
        }
        "evidence" | "aggregation" | "guarantee" => {}
        _ => return Err("unknown operation".into()),
    }
    for column in INPUT_COLUMNS.into_iter().skip(1) {
        if active.contains(&column) {
            assert_ne!(row.text(column)?, "-", "{}: active {column}", row.id());
        } else {
            assert_eq!(row.text(column)?, "-", "{}: unused {column}", row.id());
        }
    }
    Ok(())
}

fn header_counts(fixture: &Fixture, key: &str) -> TestResult<BTreeMap<String, usize>> {
    let mut counts = BTreeMap::new();
    for item in fixture.header(key).ok_or("missing census")?.split(';') {
        let (name, count) = item.split_once('=').ok_or("malformed census")?;
        if counts.insert(name.to_owned(), count.parse()?).is_some() {
            return Err("duplicate census key".into());
        }
    }
    Ok(counts)
}

fn bump(map: &mut BTreeMap<String, usize>, key: &str) {
    *map.entry(key.to_owned()).or_default() += 1;
}

/// Exact results and contextual errors for every independent oracle row;
/// frozen bytes, sequential IDs, active fields and complete record censuses.
#[test]
fn independent_settlement_oracle_matches_every_row_exactly() -> TestResult {
    let fixture = fixture()?;
    let census_keys = [
        "profile-counts",
        "operation-counts",
        "expectation-counts",
        "recipe-counts",
        "stage-context-counts",
        "guarantee-counts",
        "arithmetic-cause-counts",
        "error-endpoint-counts",
        "amount-operation-counts",
        "cash-mode-counts",
    ];
    let mut censuses: BTreeMap<&str, BTreeMap<String, usize>> = census_keys
        .into_iter()
        .map(|key| (key, BTreeMap::new()))
        .collect();
    let mut distinct = BTreeSet::new();
    for (index, row) in fixture.records().enumerate() {
        assert_eq!(row.id(), format!("SET-{:06}", index + 1));
        check_unused(&row)?;
        let inputs: TestResult<Vec<_>> = INPUT_COLUMNS
            .into_iter()
            .map(|name| Ok(row.text(name)?))
            .collect();
        distinct.insert(inputs?);
        let actual = evaluate(&row)?;
        check(&row, &actual)?;
        for (key, value) in [
            ("profile-counts", row.text("profile")?),
            ("operation-counts", row.text("op")?),
            ("expectation-counts", row.text("expected")?),
        ] {
            bump(censuses.get_mut(key).ok_or("missing census map")?, value);
        }
        if row.text("op")? == "evaluate" {
            bump(
                censuses.get_mut("recipe-counts").ok_or("recipe census")?,
                row.text("recipe")?,
            );
            bump(
                censuses
                    .get_mut("cash-mode-counts")
                    .ok_or("cash mode census")?,
                row.text("cash_mode")?,
            );
        }
        if let Outcome::Decision(_, SettlementDecision::Determined(receipt)) = &actual {
            bump(
                censuses
                    .get_mut("guarantee-counts")
                    .ok_or("guarantee census")?,
                receipt.guarantee().canonical_identifier(),
            );
        }
        // Censuses for contextual results come from actual typed outcomes.
        let fields = match &actual {
            Outcome::SettlementFailure(error) => error_fields(error)?,
            Outcome::Decision(_, SettlementDecision::ExactStageUnresolved { stage, .. }) => {
                Fields::from([("stage".to_owned(), (*stage).to_owned())])
            }
            _ => Fields::new(),
        };
        for (field, key) in [
            ("stage", "stage-context-counts"),
            ("cause", "arithmetic-cause-counts"),
            ("endpoint", "error-endpoint-counts"),
            ("operation", "amount-operation-counts"),
        ] {
            if let Some(value) = fields.get(field) {
                bump(censuses.get_mut(key).ok_or("context census")?, value);
            }
        }
    }
    assert_eq!(distinct.len(), 154_063);
    for key in census_keys {
        assert_eq!(censuses[key], header_counts(&fixture, key)?, "{key}");
    }
    assert_eq!(
        CashAggregation::ALL.map(CashAggregation::canonical_identifier),
        ["RoundTotal", "RoundComponentsThenSum"]
    );
    assert_eq!(
        SettlementGuarantee::ALL.map(SettlementGuarantee::canonical_identifier),
        [
            "ForSuppliedDecimals",
            "ForSuppliedBinary64",
            "UniqueForDeclaredEnclosure"
        ]
    );
    Ok(())
}

/// A malformed context cannot silently overwrite a contextual expectation.
#[test]
fn oracle_context_rejects_duplicate_or_empty_fields() -> TestResult {
    assert!(context_map("stage=cash total,stage=cash net").is_err());
    assert!(context_map("=value").is_err());
    assert!(context_map("stage=").is_err());
    assert!(context_map("stage=cash total,").is_err());
    assert!(context_map("-")?.is_empty());
    Ok(())
}
