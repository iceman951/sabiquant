//! FR-GILT-01..06/09/15..22: public consumer of the frozen independent
//! conventional-gilt source, date and cash oracle corpus.
//!
//! The 10,448 expectations in `gilt-source-cash-oracle.fixture` were authored
//! from the frozen contract with Python integer/Fraction arithmetic before any
//! gilt production ([report](../docs/M8_GILT_SOURCE_CASH_ORACLES.md)). This
//! consumer only parses them and compares the public API; it derives no
//! expected value from production. Fixture, generator, contract and
//! requirement-row pins and every row-derivable header census are verified
//! before records are used.
//!
//! Private data stays qualified. Exact source rationals (dividends and
//! accrued interest) are asserted through public exact quantized views only
//! where a canonical `DecimalAmount` exists; otherwise only the typed
//! point-view failure is asserted. Bundle source amounts are not publicly
//! observable: bundle quasi indices are asserted directly and the serialized
//! `L/s` coordinates only for consistency with public `r`, `s` and the next
//! quasi index. The nested `DateError` of the one direct-cycle failure is
//! compared with the fixture's checked-month reference, which the contract
//! does not mandate but the actual parent performs.

mod support;

use sabiquant::amounts::{AmountError, AmountQuantum, AmountRounding, DecimalAmount};
use sabiquant::bonds::{
    ConventionalGilt, CouponEntitlement, ExCouponBoundary, GiltCashDecision, GiltCashInstruction,
    GiltError, GiltFirstPeriod, GiltPricingPlan,
};
use sabiquant::calendar::EndOfMonth;
use sabiquant::error::Error as RootError;
use sabiquant::settlement::{
    CashAggregation, SettlementEvidence, SettlementGuarantee, SettlementRounding,
};
use sabiquant::time::{Date, DateError};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const FIXTURE: &str = "validation/fixtures/bonds/gilt-source-cash-oracle.fixture";
const SOURCE_VERSION: &str = "UK-DMO-Conventional-v4-2024-12-18";
/// Pins recorded in docs/M8_GILT_SOURCE_CASH_ORACLES.md.
const FIXTURE_SHA256: &str = "21ec34fb1f0c59c321a87a8e4ab782fdab9918e99fbed10d791fee3ad3394d88";
const GENERATOR_SHA256: &str = "4a4306414acd2874b8d1973ee1994dc298956b55c5225e3df48e8ccb4693b72a";
const CONTRACT_SHA256: &str = "03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292";
const RECORDS: usize = 10_448;
const COLUMNS: [&str; 38] = [
    "id",
    "profile",
    "op",
    "issue",
    "maturity",
    "coupon",
    "first",
    "quasi",
    "ex_dates",
    "eom",
    "date_cap",
    "settlement",
    "bundle_cap",
    "dividend_index",
    "recipe",
    "nominal",
    "evidence",
    "aggregation",
    "quantum",
    "mode",
    "expected",
    "context",
    "first_period",
    "first_quasi",
    "dividends",
    "source_values",
    "ai",
    "ai_bits",
    "next_quasi",
    "r",
    "s",
    "n",
    "bundles",
    "entitlements",
    "source_value",
    "lower",
    "upper",
    "guarantee",
];
/// The serialized input columns; the generator rejects duplicate inputs.
const INPUTS: [&str; 18] = [
    "op",
    "issue",
    "maturity",
    "coupon",
    "first",
    "quasi",
    "ex_dates",
    "eom",
    "date_cap",
    "settlement",
    "bundle_cap",
    "dividend_index",
    "recipe",
    "nominal",
    "evidence",
    "aggregation",
    "quantum",
    "mode",
];

// ---------------------------------------------------------------------------
// Pins: FNV-1a 64 and FIPS 180-4 SHA-256, standard library only.

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

const SHA256_K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

fn sha256_block(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut schedule = [0_u32; 64];
    for (word, bytes) in schedule.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_be_bytes(*bytes);
    }
    for index in 16..64 {
        let early = schedule[index - 15];
        let late = schedule[index - 2];
        let sigma0 = early.rotate_right(7) ^ early.rotate_right(18) ^ (early >> 3);
        let sigma1 = late.rotate_right(17) ^ late.rotate_right(19) ^ (late >> 10);
        schedule[index] = schedule[index - 16]
            .wrapping_add(sigma0)
            .wrapping_add(schedule[index - 7])
            .wrapping_add(sigma1);
    }
    let mut work = *state;
    for (constant, word) in SHA256_K.iter().zip(schedule) {
        let [a, b, c, d, e, f, g, h] = work;
        let big1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choose = (e & f) ^ (!e & g);
        let first = h
            .wrapping_add(big1)
            .wrapping_add(choose)
            .wrapping_add(*constant)
            .wrapping_add(word);
        let big0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let second = big0.wrapping_add(majority);
        work = [
            first.wrapping_add(second),
            a,
            b,
            c,
            d.wrapping_add(first),
            e,
            f,
            g,
        ];
    }
    for (value, add) in state.iter_mut().zip(work) {
        *value = value.wrapping_add(add);
    }
}

fn sha256(bytes: &[u8]) -> TestResult<String> {
    let mut state: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let (blocks, remainder) = bytes.as_chunks::<64>();
    for block in blocks {
        sha256_block(&mut state, block);
    }
    let mut tail = remainder.to_vec();
    tail.push(0x80);
    while tail.len() % 64 != 56 {
        tail.push(0);
    }
    let bits = u64::try_from(bytes.len())?
        .checked_mul(8)
        .ok_or("message too long for SHA-256")?;
    tail.extend_from_slice(&bits.to_be_bytes());
    for block in tail.as_chunks::<64>().0 {
        sha256_block(&mut state, block);
    }
    Ok(state.iter().map(|word| format!("{word:08x}")).collect())
}

fn read_fixture_text() -> TestResult<String> {
    #[expect(
        clippy::disallowed_methods,
        reason = "test fingerprints independent fixture bytes"
    )]
    let text = std::fs::read_to_string(Fixture::repository_path(FIXTURE))?;
    Ok(text)
}

fn requirement_rows() -> String {
    include_str!("../docs/REQUIREMENTS.md")
        .lines()
        .filter(|line| line.starts_with("| FR-GILT-"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Loads the fixture after checking columns, counts and every FNV pin.
fn pinned_fixture() -> TestResult<Fixture> {
    let text = read_fixture_text()?;
    let (_, body) = text.split_once("\n\n").ok_or("fixture body separator")?;
    let (_, records) = body.split_once('\n').ok_or("fixture column line")?;
    let fixture = Fixture::parse(&text)?;
    assert_eq!(fixture.columns(), &COLUMNS.map(str::to_owned));
    assert_eq!(fixture.record_count(), RECORDS);
    let generator = include_bytes!("../validation/generator/gilt_source_cash_oracles.py");
    let contract = include_bytes!("../docs/M8_GILT_CONTRACT.md");
    let requirements = requirement_rows();
    for (key, bytes) in [
        ("body-fnv64", body.as_bytes()),
        ("records-fnv64", records.as_bytes()),
        ("generator-fnv64", generator.as_slice()),
        ("adopted-contract-fnv64", contract.as_slice()),
        ("requirements-rows-fnv64", requirements.as_bytes()),
    ] {
        assert_eq!(
            fixture.header(key),
            Some(format!("{:016x}", fnv64(bytes)).as_str()),
            "{key}"
        );
    }
    let record_count = RECORDS.to_string();
    for (key, value) in [
        ("schema", "1"),
        ("fixture-set", "CONVENTIONAL-GILT-SOURCE-DATE-CASH-ORACLE"),
        ("domain", "bonds"),
        ("source-model", SOURCE_VERSION),
        ("record-count", record_count.as_str()),
        ("unique-input-count", record_count.as_str()),
        ("generator-sha256", GENERATOR_SHA256),
        ("adopted-contract-sha256", CONTRACT_SHA256),
        (
            "reproduction",
            "python3 -I validation/generator/gilt_source_cash_oracles.py --check",
        ),
    ] {
        assert_eq!(fixture.header(key), Some(value), "{key}");
    }
    Ok(fixture)
}

// ---------------------------------------------------------------------------
// Rows and field parsing.

/// One fixture record with every column by name.
#[derive(Clone, Copy, Debug)]
struct Row<'a> {
    id: &'a str,
    profile: &'a str,
    op: &'a str,
    issue: &'a str,
    maturity: &'a str,
    coupon: &'a str,
    first: &'a str,
    quasi: &'a str,
    ex_dates: &'a str,
    eom: &'a str,
    date_cap: &'a str,
    settlement: &'a str,
    bundle_cap: &'a str,
    dividend_index: &'a str,
    recipe: &'a str,
    nominal: &'a str,
    evidence: &'a str,
    aggregation: &'a str,
    quantum: &'a str,
    mode: &'a str,
    expected: &'a str,
    context: &'a str,
    first_period: &'a str,
    first_quasi: &'a str,
    dividends: &'a str,
    source_values: &'a str,
    ai: &'a str,
    ai_bits: &'a str,
    next_quasi: &'a str,
    r: &'a str,
    s: &'a str,
    n: &'a str,
    bundles: &'a str,
    entitlements: &'a str,
    source_value: &'a str,
    lower: &'a str,
    upper: &'a str,
    guarantee: &'a str,
}

impl<'a> Row<'a> {
    fn read(record: &Record<'a>) -> TestResult<Self> {
        let field = |column: &str| record.text(column);
        Ok(Self {
            id: record.id(),
            profile: field("profile")?,
            op: field("op")?,
            issue: field("issue")?,
            maturity: field("maturity")?,
            coupon: field("coupon")?,
            first: field("first")?,
            quasi: field("quasi")?,
            ex_dates: field("ex_dates")?,
            eom: field("eom")?,
            date_cap: field("date_cap")?,
            settlement: field("settlement")?,
            bundle_cap: field("bundle_cap")?,
            dividend_index: field("dividend_index")?,
            recipe: field("recipe")?,
            nominal: field("nominal")?,
            evidence: field("evidence")?,
            aggregation: field("aggregation")?,
            quantum: field("quantum")?,
            mode: field("mode")?,
            expected: field("expected")?,
            context: field("context")?,
            first_period: field("first_period")?,
            first_quasi: field("first_quasi")?,
            dividends: field("dividends")?,
            source_values: field("source_values")?,
            ai: field("ai")?,
            ai_bits: field("ai_bits")?,
            next_quasi: field("next_quasi")?,
            r: field("r")?,
            s: field("s")?,
            n: field("n")?,
            bundles: field("bundles")?,
            entitlements: field("entitlements")?,
            source_value: field("source_value")?,
            lower: field("lower")?,
            upper: field("upper")?,
            guarantee: field("guarantee")?,
        })
    }
}

fn rows(fixture: &Fixture) -> TestResult<Vec<Row<'_>>> {
    fixture.records().map(|record| Row::read(&record)).collect()
}

/// Parses canonical `C:S` and rejects any noncanonical spelling.
fn decimal(text: &str) -> TestResult<DecimalAmount> {
    let (coefficient, scale) = text.split_once(':').ok_or("decimal C:S")?;
    let coefficient: i128 = coefficient.parse()?;
    let scale: u8 = scale.parse()?;
    let value = DecimalAmount::new(coefficient, scale)?;
    if value.coefficient() != coefficient || value.scale() != scale {
        return Err(format!("noncanonical fixture decimal {text}").into());
    }
    Ok(value)
}

fn cell(value: DecimalAmount) -> String {
    format!("{}:{}", value.coefficient(), value.scale())
}

fn quantum(text: &str) -> TestResult<AmountQuantum> {
    let (coefficient, scale) = text.split_once(':').ok_or("quantum C:S")?;
    Ok(AmountQuantum::new(coefficient.parse()?, scale.parse()?)?)
}

fn date(text: &str) -> TestResult<Date> {
    Ok(text.parse()?)
}

fn dates(text: &str) -> TestResult<Vec<Date>> {
    if text == "empty" {
        return Ok(Vec::new());
    }
    text.split(',').map(date).collect()
}

fn binary(text: &str) -> TestResult<f64> {
    if text.len() != 16
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("expected sixteen lowercase hex digits: {text}").into());
    }
    Ok(f64::from_bits(u64::from_str_radix(text, 16)?))
}

fn evidence(text: &str) -> TestResult<SettlementEvidence> {
    let parts = text.split(':').collect::<Vec<_>>();
    let pair = |coefficient: &str, scale: &str| decimal(&format!("{coefficient}:{scale}"));
    Ok(match parts.as_slice() {
        ["d", coefficient, scale] => SettlementEvidence::exact_decimal(pair(coefficient, scale)?),
        ["di", lower, lower_scale, upper, upper_scale] => {
            SettlementEvidence::declared_decimal_interval(
                pair(lower, lower_scale)?,
                pair(upper, upper_scale)?,
            )?
        }
        ["b", bits] => SettlementEvidence::supplied_binary64(binary(bits)?)?,
        ["bi", lower, upper] => {
            SettlementEvidence::declared_binary64_interval(binary(lower)?, binary(upper)?)?
        }
        _ => return Err(format!("unknown evidence {text}").into()),
    })
}

fn hex(text: &str) -> String {
    let mut encoded = String::from("x:");
    for byte in text.bytes() {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn unhex(text: &str) -> TestResult<String> {
    let mut rest = text.strip_prefix("x:").ok_or("hex text prefix")?;
    let mut bytes = Vec::new();
    while !rest.is_empty() {
        let (pair, tail) = rest.split_at_checked(2).ok_or("odd hex length")?;
        bytes.push(u8::from_str_radix(pair, 16)?);
        rest = tail;
    }
    Ok(String::from_utf8(bytes)?)
}

fn context(text: &str) -> TestResult<BTreeMap<&str, &str>> {
    if text == "-" {
        return Ok(BTreeMap::new());
    }
    text.split(',')
        .map(|pair| {
            pair.split_once('=')
                .ok_or_else(|| "context key=value".into())
        })
        .collect()
}

fn first_period_name(period: GiltFirstPeriod) -> &'static str {
    match period {
        GiltFirstPeriod::Regular => "Regular",
        GiltFirstPeriod::Short => "Short",
        GiltFirstPeriod::Long => "Long",
        _ => "unknown",
    }
}

const fn entitlement_name(entitlement: CouponEntitlement) -> &'static str {
    match entitlement {
        CouponEntitlement::Ceased => "Ceased",
        CouponEntitlement::BuyerEntitled => "BuyerEntitled",
        CouponEntitlement::ExCouponDetached => "ExCouponDetached",
    }
}

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// A complete serialized instrument definition.
struct Definition {
    issue: Date,
    maturity: Date,
    coupon: DecimalAmount,
    first: Date,
    quasi: Vec<Date>,
    ex: Vec<Date>,
    eom: EndOfMonth,
    cap: usize,
}

impl Definition {
    fn parse(row: &Row<'_>) -> TestResult<Self> {
        Ok(Self {
            issue: date(row.issue)?,
            maturity: date(row.maturity)?,
            coupon: decimal(row.coupon)?,
            first: date(row.first)?,
            quasi: dates(row.quasi)?,
            ex: dates(row.ex_dates)?,
            eom: row.eom.parse()?,
            cap: row.date_cap.parse()?,
        })
    }

    fn build(&self) -> Result<ConventionalGilt, GiltError> {
        ConventionalGilt::new(
            self.issue,
            self.maturity,
            self.coupon,
            self.first,
            self.quasi.clone(),
            self.ex.clone(),
            self.eom,
            self.cap,
        )
    }

    /// Builds a definition that the fixture records as valid.
    fn valid(&self, id: &str) -> TestResult<ConventionalGilt> {
        self.build()
            .map_err(|error| format!("{id}: valid fixture definition rejected: {error:?}").into())
    }
}

fn plan<'a>(row: &Row<'_>, gilt: &'a ConventionalGilt) -> TestResult<GiltPricingPlan<'a>> {
    GiltPricingPlan::new(gilt, date(row.settlement)?, row.bundle_cap.parse()?)
        .map_err(|error| format!("{}: valid fixture plan rejected: {error:?}", row.id).into())
}

// ---------------------------------------------------------------------------
// Exact rational views, error rendering and mismatch collection.

/// Converts a reduced fixture rational `N/D` to its canonical decimal, or
/// None when no `DecimalAmount` (scale<=38, signed i128 coefficient) exists.
fn exact_decimal(rational: &str) -> TestResult<Option<DecimalAmount>> {
    let (numerator, denominator) = rational.split_once('/').ok_or("rational N/D")?;
    let (negative, digits) = match numerator.strip_prefix('-') {
        Some(digits) => (true, digits),
        None => (false, numerator),
    };
    let (Ok(magnitude), Ok(mut reduced)) = (digits.parse::<u128>(), denominator.parse::<u128>())
    else {
        return Ok(None);
    };
    if reduced == 0 {
        return Err(format!("zero denominator {rational}").into());
    }
    let mut twos = 0_u32;
    let mut fives = 0_u32;
    while reduced % 2 == 0 {
        reduced /= 2;
        twos += 1;
    }
    while reduced % 5 == 0 {
        reduced /= 5;
        fives += 1;
    }
    let scale = twos.max(fives);
    if reduced != 1 || scale > 38 {
        return Ok(None);
    }
    let factor = 2_u128
        .checked_pow(scale - twos)
        .and_then(|two| 5_u128.checked_pow(scale - fives)?.checked_mul(two));
    let Some(coefficient) = factor.and_then(|factor| magnitude.checked_mul(factor)) else {
        return Ok(None);
    };
    let signed = if negative {
        0_i128.checked_sub_unsigned(coefficient)
    } else {
        i128::try_from(coefficient).ok()
    };
    match signed {
        Some(signed) => Ok(Some(DecimalAmount::new(signed, u8::try_from(scale)?)?)),
        None => Ok(None),
    }
}

/// The quantum for a public exact view of `rational`: its own canonical
/// scale when representable, otherwise the finest public scale 38.
fn exact_view_quantum(rational: &str) -> TestResult<(Option<DecimalAmount>, AmountQuantum)> {
    let value = exact_decimal(rational)?;
    let scale = value.map_or(38, DecimalAmount::scale);
    Ok((value, AmountQuantum::new(1, scale)?))
}

/// Renders every public field of a gilt error exactly as the fixture context.
fn render(error: &GiltError) -> TestResult<String> {
    let mut pairs: Vec<(&str, String)> = match error {
        GiltError::InvalidDateCap {
            cap,
            minimum,
            maximum,
            ..
        } => vec![
            ("cap", cap.to_string()),
            ("minimum", minimum.to_string()),
            ("maximum", maximum.to_string()),
        ],
        GiltError::TooManyQuasiDates { count, cap, .. } => {
            vec![("count", count.to_string()), ("cap", cap.to_string())]
        }
        GiltError::TooFewQuasiDates { count, minimum, .. } => {
            vec![
                ("count", count.to_string()),
                ("minimum", minimum.to_string()),
            ]
        }
        GiltError::NegativeCoupon { coupon_per_100, .. } => {
            vec![("coupon_per_100", cell(*coupon_per_100))]
        }
        GiltError::IssueNotBeforeMaturity {
            issue_date,
            maturity_date,
            ..
        } => vec![
            ("issue_date", issue_date.to_string()),
            ("maturity_date", maturity_date.to_string()),
        ],
        GiltError::MaturityBoundary {
            maturity_date,
            last_quasi_date,
            ..
        } => vec![
            ("maturity_date", maturity_date.to_string()),
            ("last_quasi_date", last_quasi_date.to_string()),
        ],
        GiltError::QuasiDateOrder {
            index,
            previous,
            date,
            ..
        } => vec![
            ("index", index.to_string()),
            ("previous", previous.to_string()),
            ("date", date.to_string()),
        ],
        GiltError::QuasiCoverage {
            issue_date,
            first_quasi_date,
            next_quasi_date,
            ..
        } => vec![
            ("issue_date", issue_date.to_string()),
            ("first_quasi_date", first_quasi_date.to_string()),
            ("next_quasi_date", next_quasi_date.to_string()),
        ],
        GiltError::QuasiDateMismatch {
            index,
            supplied,
            expected,
            maturity_date,
            end_of_month,
            ..
        } => vec![
            ("index", index.to_string()),
            ("supplied", supplied.to_string()),
            ("expected", expected.to_string()),
            ("maturity_date", maturity_date.to_string()),
            ("end_of_month", end_of_month.to_string()),
        ],
        GiltError::Date {
            index,
            phase,
            cause,
            ..
        } => {
            let mut pairs = vec![
                ("cause_type", "DateError".to_owned()),
                (
                    "index",
                    index.map_or_else(|| "-".to_owned(), |index| index.to_string()),
                ),
                ("phase", hex(phase)),
                ("reference_cause", cause.kind().to_owned()),
            ];
            if let DateError::OutOfRange {
                date,
                operation,
                count,
                ..
            } = cause
            {
                pairs.push(("reference_count", count.to_string()));
                pairs.push(("reference_date", date.to_string()));
                pairs.push(("reference_operation", hex(operation)));
            }
            pairs
        }
        GiltError::FirstDividend {
            issue_date,
            first_dividend_date,
            next_quasi_date,
            second_quasi_date,
            ..
        } => vec![
            ("issue_date", issue_date.to_string()),
            ("first_dividend_date", first_dividend_date.to_string()),
            ("next_quasi_date", next_quasi_date.to_string()),
            (
                "second_quasi_date",
                second_quasi_date.map_or_else(|| "-".to_owned(), |date| date.to_string()),
            ),
        ],
        GiltError::ExDateCount {
            count, expected, ..
        } => vec![
            ("count", count.to_string()),
            ("expected", expected.to_string()),
        ],
        GiltError::ExDateWindow {
            dividend_index,
            cutoff,
            lower,
            dividend_date,
            ..
        } => vec![
            ("dividend_index", dividend_index.to_string()),
            ("cutoff", cutoff.to_string()),
            ("lower", lower.to_string()),
            ("dividend_date", dividend_date.to_string()),
        ],
        GiltError::InvalidBundleCap { cap, maximum, .. } => {
            vec![("cap", cap.to_string()), ("maximum", maximum.to_string())]
        }
        GiltError::TooManyBundles { count, cap, .. } => {
            vec![("count", count.to_string()), ("cap", cap.to_string())]
        }
        GiltError::SettlementBeforeIssue {
            settlement_date,
            issue_date,
            ..
        } => vec![
            ("settlement_date", settlement_date.to_string()),
            ("issue_date", issue_date.to_string()),
        ],
        GiltError::SettlementAtOrAfterMaturity {
            settlement_date,
            maturity_date,
            ..
        } => vec![
            ("settlement_date", settlement_date.to_string()),
            ("maturity_date", maturity_date.to_string()),
        ],
        GiltError::InvalidDividendIndex { index, count, .. } => {
            vec![("index", index.to_string()), ("count", count.to_string())]
        }
        GiltError::NonPositiveNominal { nominal, .. } => vec![("nominal", cell(*nominal))],
        GiltError::CashArithmetic {
            stage,
            endpoint,
            cause,
            ..
        } => {
            let mut pairs = vec![
                ("cause", cause.kind().to_owned()),
                ("endpoint", (*endpoint).to_owned()),
                ("stage", hex(stage)),
            ];
            match cause {
                AmountError::Inexact { operation, .. } => {
                    pairs.push(("operation", (*operation).to_owned()));
                }
                AmountError::OutOfRange {
                    operation,
                    negative,
                    ..
                } => {
                    pairs.push(("operation", (*operation).to_owned()));
                    pairs.push(("negative", negative.to_string()));
                }
                other => return Err(format!("unexpected cash cause {other:?}").into()),
            }
            pairs
        }
        other => return Err(format!("unexpected gilt error in corpus scope: {other:?}").into()),
    };
    pairs.sort_unstable();
    Ok(pairs
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(","))
}

/// Collects every mismatch with its row identifier rather than stopping at
/// the first, so production defects are reported by row ID.
#[derive(Debug, Default)]
struct Mismatches {
    count: usize,
    examples: Vec<String>,
}

impl Mismatches {
    fn fail(&mut self, id: &str, detail: String) {
        self.count += 1;
        if self.examples.len() < 40 {
            self.examples.push(format!("{id}: {detail}"));
        }
    }

    fn check<T: PartialEq + Debug>(&mut self, id: &str, what: &str, actual: T, expected: T) {
        if actual != expected {
            self.fail(
                id,
                format!("{what}: actual {actual:?}, expected {expected:?}"),
            );
        }
    }

    /// Compares a typed error with the row's kind and full context, then the
    /// stable kind, Display, typed source chain and root wrapper.
    fn error(&mut self, row: &Row<'_>, error: &GiltError) -> TestResult {
        self.check(row.id, "error kind", error.kind(), row.expected);
        self.check(
            row.id,
            "error context",
            render(error)?.as_str(),
            row.context,
        );
        if error.to_string().is_empty() {
            self.fail(row.id, "empty Display".to_owned());
        }
        let nested_matches = match error {
            GiltError::CashArithmetic { cause, .. } => {
                error
                    .source()
                    .and_then(|source| source.downcast_ref::<AmountError>())
                    == Some(cause)
            }
            GiltError::Date { cause, .. } => {
                error
                    .source()
                    .and_then(|source| source.downcast_ref::<DateError>())
                    == Some(cause)
            }
            _ => error.source().is_none(),
        };
        if !nested_matches {
            self.fail(row.id, format!("typed source chain of {error:?}"));
        }
        let root = RootError::from(error.clone());
        self.check(row.id, "root kind", root.kind(), error.kind());
        let wrapped = matches!(&root, RootError::Gilt(inner) if inner == error);
        let sourced = root
            .source()
            .and_then(|source| source.downcast_ref::<GiltError>())
            == Some(error);
        if !wrapped || !sourced {
            self.fail(row.id, format!("root wrapper of {error:?}"));
        }
        Ok(())
    }

    /// Checks a public exact quantized source view against a fixture rational.
    fn exact_view(
        &mut self,
        id: &str,
        what: &str,
        view: Result<Option<DecimalAmount>, GiltError>,
        expected: Option<DecimalAmount>,
        stage: &str,
    ) {
        match (expected, view) {
            (Some(value), view) => self.check(id, what, view, Ok(Some(value))),
            (
                None,
                Err(GiltError::CashArithmetic {
                    stage: actual,
                    endpoint: "point",
                    cause:
                        AmountError::Inexact {
                            operation: "quantize",
                            ..
                        }
                        | AmountError::OutOfRange {
                            operation: "quantize",
                            ..
                        },
                    ..
                }),
            ) if actual == stage => {}
            (None, other) => self.fail(id, format!("{what}: nonrepresentable view {other:?}")),
        }
    }

    fn finish(self, scope: &str) -> TestResult {
        if self.count == 0 {
            Ok(())
        } else {
            Err(format!(
                "{scope}: {} mismatches; first {:#?}",
                self.count, self.examples
            )
            .into())
        }
    }
}

// ---------------------------------------------------------------------------
// Cash inputs, factories and instruction inspectors.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Recipe {
    Dividend,
    Accrued,
    Clean,
    Dirty,
}

struct CashInputs {
    recipe: Recipe,
    nominal: DecimalAmount,
    rounding: SettlementRounding,
    evidence: Option<SettlementEvidence>,
    aggregation: Option<CashAggregation>,
    dividend_index: Option<usize>,
}

impl CashInputs {
    fn parse(row: &Row<'_>) -> TestResult<Self> {
        let recipe = match row.recipe {
            "PublishedDividendPer100" => Recipe::Dividend,
            "SourceAccruedPer100" => Recipe::Accrued,
            "CleanPriceAndSourceAccruedPer100" => Recipe::Clean,
            "DirtyPricePer100" => Recipe::Dirty,
            other => return Err(format!("unknown recipe {other}").into()),
        };
        let mode: AmountRounding = row.mode.parse()?;
        Ok(Self {
            recipe,
            nominal: decimal(row.nominal)?,
            rounding: SettlementRounding::new(quantum(row.quantum)?, mode),
            evidence: (row.evidence != "-")
                .then(|| evidence(row.evidence))
                .transpose()?,
            aggregation: (row.aggregation != "-")
                .then(|| row.aggregation.parse())
                .transpose()?,
            dividend_index: (row.dividend_index != "-")
                .then(|| row.dividend_index.parse())
                .transpose()?,
        })
    }

    fn build(
        &self,
        gilt: &ConventionalGilt,
        plan: Option<&GiltPricingPlan<'_>>,
    ) -> TestResult<Result<GiltCashInstruction, GiltError>> {
        let prepared = || plan.ok_or("plan recipe without settlement");
        let evidence = || self.evidence.ok_or("price recipe without evidence");
        Ok(match self.recipe {
            Recipe::Dividend => gilt.dividend_cash(
                self.dividend_index.ok_or("dividend recipe without index")?,
                self.nominal,
                self.rounding,
            ),
            Recipe::Accrued => prepared()?.accrued_cash(self.nominal, self.rounding),
            Recipe::Clean => prepared()?.clean_accrued_cash(
                self.nominal,
                evidence()?,
                self.aggregation.ok_or("invoice without aggregation")?,
                self.rounding,
            ),
            Recipe::Dirty => prepared()?.dirty_cash(self.nominal, evidence()?, self.rounding),
        })
    }

    /// Asserts every instruction inspector from the contract C3 roles, the
    /// row inputs and the public instrument/plan inspectors.
    fn inspect(
        &self,
        m: &mut Mismatches,
        id: &str,
        gilt: &ConventionalGilt,
        plan: Option<&GiltPricingPlan<'_>>,
        instruction: &GiltCashInstruction,
    ) -> TestResult {
        let recipe_name = match self.recipe {
            Recipe::Dividend => "PublishedDividendPer100",
            Recipe::Accrued => "SourceAccruedPer100",
            Recipe::Clean => "CleanPriceAndSourceAccruedPer100",
            Recipe::Dirty => "DirtyPricePer100",
        };
        m.check(
            id,
            "source_version",
            instruction.source_version(),
            SOURCE_VERSION,
        );
        m.check(id, "recipe", instruction.recipe_identifier(), recipe_name);
        m.check(id, "nominal", instruction.nominal(), self.nominal);
        m.check(
            id,
            "cash_rounding",
            instruction.cash_rounding(),
            self.rounding,
        );
        let priced = matches!(self.recipe, Recipe::Clean | Recipe::Dirty);
        m.check(
            id,
            "price_evidence",
            instruction.price_evidence(),
            if priced { self.evidence } else { None },
        );
        m.check(
            id,
            "aggregation",
            instruction.aggregation(),
            self.aggregation,
        );
        m.check(
            id,
            "coupon",
            instruction.coupon_per_100(),
            gilt.coupon_per_100(),
        );
        m.check(id, "issue", instruction.issue_date(), gilt.issue_date());
        m.check(
            id,
            "maturity",
            instruction.maturity_date(),
            gilt.maturity_date(),
        );
        m.check(
            id,
            "first dividend",
            instruction.first_dividend_date(),
            gilt.first_dividend_date(),
        );
        m.check(
            id,
            "first period",
            instruction.first_period(),
            gilt.first_period(),
        );
        m.check(
            id,
            "end of month",
            instruction.end_of_month(),
            gilt.end_of_month(),
        );
        let quasi = gilt.quasi_dates();
        m.check(
            id,
            "issue quasi pair",
            instruction.issue_quasi_dates(),
            (quasi[0], quasi[1]),
        );
        let settlement = match (self.recipe, plan) {
            (Recipe::Dividend, _) => None,
            (_, Some(plan)) => Some(plan),
            (_, None) => return Err(format!("{id}: plan recipe without plan").into()),
        };
        m.check(
            id,
            "settlement date",
            instruction.settlement_date(),
            settlement.map(GiltPricingPlan::settlement_date),
        );
        m.check(
            id,
            "settlement quasi pair",
            instruction.settlement_quasi_dates(),
            settlement.map(|plan| {
                (
                    quasi[plan.next_quasi_index() - 1],
                    quasi[plan.next_quasi_index()],
                )
            }),
        );
        let first = gilt.first_dividend_quasi_index();
        let selected = match (self.recipe, settlement) {
            (Recipe::Dividend, _) => self.dividend_index,
            (Recipe::Accrued | Recipe::Clean, Some(plan)) => {
                Some(plan.next_quasi_index().max(first) - first)
            }
            _ => None,
        };
        m.check(
            id,
            "source dividend index",
            instruction.source_dividend_index(),
            selected,
        );
        m.check(
            id,
            "source dividend date",
            instruction.source_dividend_date(),
            selected.map(|ordinal| gilt.dividend_dates()[ordinal]),
        );
        let ex_context = match (self.recipe, settlement, selected) {
            (Recipe::Accrued | Recipe::Clean, Some(plan), Some(ordinal)) => {
                let cutoff = gilt.ex_dividend_dates()[ordinal];
                Some((cutoff, plan.settlement_date() > cutoff))
            }
            _ => None,
        };
        m.check(
            id,
            "ex-dividend date",
            instruction.ex_dividend_date(),
            ex_context.map(|(cutoff, _)| cutoff),
        );
        m.check(
            id,
            "is ex-dividend",
            instruction.is_ex_dividend(),
            ex_context.map(|(_, ex)| ex),
        );
        if self.recipe == Recipe::Dirty {
            m.check(
                id,
                "dirty source view",
                instruction.source_value_per_100(self.rounding.quantum(), self.rounding.mode()),
                Ok(None),
            );
        }
        // Reproducible compact identity: an identical factory call is equal.
        match self.build(gilt, plan)? {
            Ok(again) => {
                m.check(id, "instruction identity", &again, instruction);
                m.check(
                    id,
                    "instruction hash",
                    hash_of(&again),
                    hash_of(instruction),
                );
            }
            Err(error) => m.fail(id, format!("repeated factory failed: {error:?}")),
        }
        Ok(())
    }

    /// The contract C3 guarantee rule from the supplied evidence kind.
    fn guarantee(&self, row: &Row<'_>) -> Option<SettlementGuarantee> {
        match self.recipe {
            Recipe::Dividend | Recipe::Accrued => None,
            Recipe::Clean | Recipe::Dirty => Some(match row.evidence.split(':').next() {
                Some("di" | "bi") => SettlementGuarantee::UniqueForDeclaredEnclosure,
                Some("b") => SettlementGuarantee::ForSuppliedBinary64,
                _ => SettlementGuarantee::ForSuppliedDecimals,
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests.

#[test]
fn sha256_matches_fips_vectors() -> TestResult {
    assert_eq!(
        sha256(b"abc")?,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256(b"")?,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")?,
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    Ok(())
}

fn counter_text(counter: &BTreeMap<String, usize>) -> String {
    counter
        .iter()
        .map(|(key, count)| format!("{key}={count}"))
        .collect::<Vec<_>>()
        .join(";")
}

fn bump(counter: &mut BTreeMap<String, usize>, key: &str) {
    *counter.entry(key.to_owned()).or_insert(0) += 1;
}

/// Bit length of a nonnegative decimal integer of any size.
fn decimal_bits(digits: &str, multiplier: u64) -> TestResult<u32> {
    let mut limbs: Vec<u64> = vec![0];
    let mut multiply_add = |factor: u64, addend: u64| {
        let mut carry = u128::from(addend);
        for limb in &mut limbs {
            let product = u128::from(*limb) * u128::from(factor) + carry;
            *limb = u64::try_from(product & u128::from(u64::MAX)).unwrap_or(0);
            carry = product >> 64;
        }
        if carry != 0 {
            limbs.push(u64::try_from(carry).unwrap_or(0));
        }
    };
    for digit in digits.bytes() {
        if !digit.is_ascii_digit() {
            return Err(format!("decimal digits {digits}").into());
        }
        multiply_add(10, u64::from(digit - b'0'));
    }
    multiply_add(multiplier, 0);
    let top = limbs.iter().rposition(|limb| *limb != 0);
    Ok(match top {
        None => 0,
        Some(index) => u32::try_from(index)? * 64 + (64 - limbs[index].leading_zeros()),
    })
}

/// FR-GILT-22: pins, every row-derivable header census and unique inputs
/// are verified before the corpus is trusted.
#[test]
fn fixture_pins_census_and_unique_inputs() -> TestResult {
    let text = read_fixture_text()?;
    assert_eq!(sha256(text.as_bytes())?, FIXTURE_SHA256);
    let (_, body) = text.split_once("\n\n").ok_or("fixture body separator")?;
    let (_, records) = body.split_once('\n').ok_or("fixture column line")?;
    let fixture = pinned_fixture()?;
    let generator = include_bytes!("../validation/generator/gilt_source_cash_oracles.py");
    let contract = include_bytes!("../docs/M8_GILT_CONTRACT.md");
    let requirements = requirement_rows();
    assert_eq!(requirements.lines().count(), 24);
    assert_eq!(sha256(generator)?, GENERATOR_SHA256);
    assert_eq!(sha256(contract)?, CONTRACT_SHA256);
    for (key, bytes) in [
        ("body-sha256", body.as_bytes()),
        ("records-sha256", records.as_bytes()),
        ("requirements-rows-sha256", requirements.as_bytes()),
    ] {
        assert_eq!(fixture.header(key), Some(sha256(bytes)?.as_str()), "{key}");
    }

    let mut counters: BTreeMap<&str, BTreeMap<String, usize>> = BTreeMap::new();
    let mut scales: BTreeMap<&str, BTreeSet<u8>> = BTreeMap::new();
    let mut inputs = BTreeSet::new();
    let mut dividend_bits = 0;
    let mut micro_bits = 0;
    for (index, record) in fixture.records().enumerate() {
        let row = Row::read(&record)?;
        assert_eq!(row.id, format!("GSC-{:06}", index + 1));
        let key = INPUTS
            .iter()
            .map(|column| record.text(column))
            .collect::<Result<Vec<_>, _>>()?
            .join("\t");
        inputs.insert(key);
        let context = context(row.context)?;
        let mut count = |name: &'static str, key: &str| {
            bump(counters.entry(name).or_default(), key);
        };
        count("profile-counts", row.profile);
        count("operation-counts", row.op);
        count("expectation-counts", row.expected);
        if row.first_period != "-" {
            count("first-period-counts", row.first_period);
            scales
                .entry("coupon")
                .or_default()
                .insert(decimal(row.coupon)?.scale());
        }
        if row.mode != "-" {
            count("cash-mode-counts", row.mode);
            scales
                .entry("quantum")
                .or_default()
                .insert(quantum(row.quantum)?.amount().scale());
        }
        if row.recipe != "-" {
            count("recipe-counts", row.recipe);
        }
        if row.nominal != "-" && decimal(row.nominal)?.coefficient() > 0 {
            scales
                .entry("nominal")
                .or_default()
                .insert(decimal(row.nominal)?.scale());
        }
        if let Some(cause) = context.get("cause") {
            count("nested-cause-counts", cause);
        }
        // Unresolved Exact decisions also carry a stage but are not failures.
        if row.expected.starts_with("gilt.")
            && let Some(stage) = context.get("stage")
        {
            count("arithmetic-stage-counts", stage);
        }
        if let Some(endpoint) = context.get("endpoint") {
            count("arithmetic-endpoint-counts", endpoint);
        }
        if row.guarantee != "-" {
            count("price-guarantee-counts", row.guarantee);
        }
        if row.evidence != "-" {
            let kind = row.evidence.split(':').next().ok_or("evidence kind")?;
            count("evidence-kind-counts", kind);
        }
        if row.ai != "-" {
            // The contract S lifecycle: regular after F, otherwise the role;
            // ex is the next actual dividend's strict-after status.
            let next: usize = row.next_quasi.parse()?;
            let first: usize = row.first_quasi.parse()?;
            let cutoffs = dates(row.ex_dates)?;
            let ex = date(row.settlement)? > cutoffs[next.max(first) - first];
            let branch = match (next > first, row.first_period, ex) {
                (true, _, false) | (false, "Regular", false) => "regular-cum",
                (true, _, true) | (false, "Regular", true) => "regular-ex",
                (false, "Short", false) => "short-cum",
                (false, "Short", true) => "short-ex",
                (false, "Long", _) if next < first => "long-first",
                (false, "Long", false) => "long-second-cum",
                (false, "Long", true) => "long-second-ex",
                _ => return Err(format!("{}: unknown lifecycle", row.id).into()),
            };
            count("ai-branch-counts", branch);
        }
        if row.source_values != "-" {
            for (ordinal, value) in row.source_values.split(',').enumerate() {
                let (numerator, denominator) = value.split_once('/').ok_or("rational")?;
                let digits = numerator.trim_start_matches('-');
                dividend_bits = dividend_bits.max(decimal_bits(digits, 1)?);
                if ordinal == 0 && row.first_period != "Regular" {
                    let denominator: u64 = denominator.parse()?;
                    if denominator == 0 || 1_000_000 % denominator != 0 {
                        return Err(format!("{}: published value is not micro", row.id).into());
                    }
                    micro_bits = micro_bits.max(decimal_bits(digits, 1_000_000 / denominator)?);
                }
            }
        }
    }
    assert_eq!(inputs.len(), RECORDS, "serialized inputs are unique");
    for (name, counter) in &counters {
        assert_eq!(
            fixture.header(name),
            Some(counter_text(counter).as_str()),
            "{name}"
        );
    }
    assert_eq!(counters.len(), 12);
    let census = ["coupon", "nominal", "quantum"]
        .iter()
        .map(|name| {
            let values = scales
                .get(name)
                .map(|set| set.iter().map(u8::to_string).collect::<Vec<_>>().join(","))
                .unwrap_or_default();
            format!("{name}={values}")
        })
        .collect::<Vec<_>>()
        .join(";");
    assert_eq!(fixture.header("scale-census"), Some(census.as_str()));
    for scale in scales.values() {
        assert_eq!(scale.len(), 39);
    }
    let widths = fixture
        .header("maximum-observed-widths")
        .ok_or("widths header")?;
    assert!(widths.contains(&format!("published-micro-coefficient-bits={micro_bits}")));
    assert!(widths.contains(&format!("source-dividend-numerator-bits={dividend_bits}")));
    assert_eq!((micro_bits, dividend_bits), (147, 147));
    Ok(())
}

/// FR-GILT-01/02/03/05/20: every serialized definition, every instrument
/// failure with full context, all instrument inspectors, exact dividend views
/// and the out-of-range dividend selectors.
#[test]
fn instrument_definitions_inspectors_and_failures() -> TestResult {
    let fixture = pinned_fixture()?;
    let mut m = Mismatches::default();
    let mut failures = 0;
    let mut valid = 0;
    let mut exact_values = 0;
    let mut qualified_values = 0;
    for row in rows(&fixture)? {
        let definition = Definition::parse(&row)?;
        let built = definition.build();
        if row.op == "instrument" {
            failures += 1;
            match &built {
                Ok(gilt) => m.fail(row.id, format!("expected failure, built {gilt:?}")),
                Err(error) => m.error(&row, error)?,
            }
            continue;
        }
        let gilt = match built {
            Ok(gilt) => gilt,
            Err(error) => {
                m.fail(row.id, format!("valid definition rejected: {error:?}"));
                continue;
            }
        };
        valid += 1;
        let first: usize = row.first_quasi.parse()?;
        let dividend_dates = dates(row.dividends)?;
        let id = row.id;
        m.check(id, "source version", gilt.source_version(), SOURCE_VERSION);
        m.check(id, "issue", gilt.issue_date(), definition.issue);
        m.check(id, "maturity", gilt.maturity_date(), definition.maturity);
        m.check(id, "coupon", gilt.coupon_per_100(), definition.coupon);
        m.check(
            id,
            "first dividend",
            gilt.first_dividend_date(),
            definition.first,
        );
        m.check(
            id,
            "first period",
            first_period_name(gilt.first_period()),
            row.first_period,
        );
        m.check(
            id,
            "first quasi index",
            gilt.first_dividend_quasi_index(),
            first,
        );
        m.check(
            id,
            "quasi dates",
            gilt.quasi_dates(),
            definition.quasi.as_slice(),
        );
        m.check(
            id,
            "dividend dates",
            gilt.dividend_dates(),
            dividend_dates.as_slice(),
        );
        m.check(
            id,
            "ex dates",
            gilt.ex_dividend_dates(),
            definition.ex.as_slice(),
        );
        m.check(id, "end of month", gilt.end_of_month(), definition.eom);
        m.check(
            id,
            "boundary",
            gilt.ex_coupon_boundary(),
            ExCouponBoundary::After,
        );
        let values = row.source_values.split(',').collect::<Vec<_>>();
        m.check(id, "dividend count", gilt.dividend_count(), values.len());
        let again = definition.valid(id)?;
        m.check(id, "definition identity", &again, &gilt);
        m.check(id, "definition hash", hash_of(&again), hash_of(&gilt));
        for (ordinal, value) in values.iter().enumerate() {
            let (expected, step) = exact_view_quantum(value)?;
            if expected.is_some() {
                exact_values += 1;
            } else {
                qualified_values += 1;
            }
            let view = gilt
                .dividend_per_100(ordinal, step, AmountRounding::Exact)
                .map(Some);
            m.exact_view(id, "exact dividend view", view, expected, "source dividend");
        }
        let count = values.len();
        let past_end = |error: Option<GiltError>| {
            matches!(
                error,
                Some(GiltError::InvalidDividendIndex { index, count: total, .. })
                    if index == count && total == count
            )
        };
        let step = AmountQuantum::new(1, 0)?;
        if !past_end(
            gilt.dividend_per_100(count, step, AmountRounding::Exact)
                .err(),
        ) {
            m.fail(
                id,
                "dividend view past end is not InvalidDividendIndex".to_owned(),
            );
        }
        let rounding = SettlementRounding::new(step, AmountRounding::Exact);
        let one = DecimalAmount::new(1, 0)?;
        if !past_end(gilt.dividend_cash(count, one, rounding).err()) {
            m.fail(
                id,
                "dividend cash past end is not InvalidDividendIndex".to_owned(),
            );
        }
    }
    assert_eq!((failures, valid), (24, RECORDS - 24));
    assert_eq!(exact_values + qualified_values, 36_839);
    m.finish("instrument")
}

/// FR-GILT-04/05/06/09/20/21: every prepared settlement (plan, cash and
/// instruction-view rows) and every plan failure: k/r/s/n, sparse bundle
/// indices, entitlements, cached one-RN source AI bits and exact AI views.
#[test]
fn plan_lifecycle_entitlement_bundles_and_cached_ai() -> TestResult {
    let fixture = pinned_fixture()?;
    let mut m = Mismatches::default();
    let mut prepared = 0;
    let mut failures = 0;
    let mut exact_ai = 0;
    let mut qualified_ai = 0;
    let unit = SettlementRounding::new(AmountQuantum::new(1, 0)?, AmountRounding::NearestTiesEven);
    let one = DecimalAmount::new(1, 0)?;
    for row in rows(&fixture)? {
        if row.settlement == "-" {
            continue;
        }
        let id = row.id;
        let gilt = Definition::parse(&row)?.valid(id)?;
        let result = GiltPricingPlan::new(&gilt, date(row.settlement)?, row.bundle_cap.parse()?);
        if row.op == "plan" && row.expected != "ok" {
            failures += 1;
            match &result {
                Ok(plan) => m.fail(id, format!("expected failure, prepared {plan:?}")),
                Err(error) => m.error(&row, error)?,
            }
            continue;
        }
        let plan = match result {
            Ok(plan) => plan,
            Err(error) => {
                m.fail(id, format!("valid plan rejected: {error:?}"));
                continue;
            }
        };
        prepared += 1;
        let next: usize = row.next_quasi.parse()?;
        let r: u32 = row.r.parse()?;
        let s: u32 = row.s.parse()?;
        m.check(id, "plan gilt", std::ptr::eq(plan.gilt(), &gilt), true);
        m.check(
            id,
            "plan source version",
            plan.source_version(),
            SOURCE_VERSION,
        );
        m.check(
            id,
            "settlement",
            plan.settlement_date(),
            date(row.settlement)?,
        );
        m.check(id, "next quasi index", plan.next_quasi_index(), next);
        m.check(id, "remaining days", plan.remaining_days(), r);
        m.check(id, "quasi period days", plan.quasi_period_days(), s);
        m.check(
            id,
            "remaining periods",
            plan.remaining_quasi_periods(),
            row.n.parse()?,
        );
        m.check(
            id,
            "cached AI bits",
            format!("{:016x}", plan.accrued_per_100().to_bits()),
            row.ai_bits.to_owned(),
        );
        let bundles = row.bundles.split(',').collect::<Vec<_>>();
        m.check(id, "bundle count", plan.bundle_count(), bundles.len());
        for (position, bundle) in bundles.iter().enumerate() {
            let fields = bundle.split('@').collect::<Vec<_>>();
            let [index, _private_amount, coordinate] = fields.as_slice() else {
                return Err(format!("{id}: bundle cell {bundle}").into());
            };
            let index: usize = index.parse()?;
            m.check(
                id,
                "bundle quasi index",
                plan.bundle_quasi_index(position),
                Some(index),
            );
            // L/s is private; check only its consistency with public r/s/k+1.
            let (l, denominator) = coordinate.split_once('/').ok_or("L/s")?;
            let steps = u32::try_from(index.checked_sub(next).ok_or("bundle before next")?)?;
            m.check(id, "bundle s", denominator.parse::<u32>()?, s);
            m.check(id, "bundle L", l.parse::<u32>()?, r + steps * s);
        }
        m.check(
            id,
            "bundle past end",
            plan.bundle_quasi_index(bundles.len()),
            None,
        );
        let entitlements = row.entitlements.split(',').collect::<Vec<_>>();
        for (ordinal, expected) in entitlements.iter().enumerate() {
            m.check(
                id,
                "entitlement",
                plan.dividend_entitlement(ordinal).map(entitlement_name),
                Some(*expected),
            );
        }
        m.check(
            id,
            "entitlement past end",
            plan.dividend_entitlement(entitlements.len()),
            None,
        );
        m.check(id, "plan clone", &plan.clone(), &plan);
        let (expected, step) = exact_view_quantum(row.ai)?;
        if expected.is_some() {
            exact_ai += 1;
        } else {
            qualified_ai += 1;
        }
        match plan.accrued_cash(one, unit) {
            Ok(instruction) => m.exact_view(
                id,
                "exact AI view",
                instruction.source_value_per_100(step, AmountRounding::Exact),
                expected,
                "source accrued",
            ),
            Err(error) => m.fail(id, format!("accrued instruction failed: {error:?}")),
        }
    }
    assert_eq!(failures, 21);
    assert_eq!(prepared, 9_054);
    assert_eq!((exact_ai, qualified_ai), (3_119, 5_935));
    m.finish("plan")
}

/// FR-GILT-03/09/15/20: explicit quantized instrument dividend views.
#[test]
fn dividend_views_quantize_exact_published_and_regular_sources() -> TestResult {
    let fixture = pinned_fixture()?;
    let mut m = Mismatches::default();
    let mut outcomes: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows(&fixture)? {
        if row.op != "dividend_view" {
            continue;
        }
        bump(&mut outcomes, row.expected);
        let id = row.id;
        let gilt = Definition::parse(&row)?.valid(id)?;
        let index: usize = row.dividend_index.parse()?;
        let mode: AmountRounding = row.mode.parse()?;
        let result = gilt.dividend_per_100(index, quantum(row.quantum)?, mode);
        if row.expected == "some" {
            m.check(id, "lower equals upper", row.lower, row.upper);
            let values = row.source_values.split(',').collect::<Vec<_>>();
            m.check(
                id,
                "selected source value",
                Some(row.source_value),
                values.get(index).copied(),
            );
            m.check(id, "dividend view", result, Ok(decimal(row.lower)?));
        } else {
            match &result {
                Ok(value) => m.fail(id, format!("expected failure, got {value:?}")),
                Err(error) => m.error(&row, error)?,
            }
        }
    }
    assert_eq!(
        counter_text(&outcomes),
        "gilt.cash_arithmetic=140;gilt.invalid_dividend_index=2;some=1047"
    );
    m.finish("dividend view")
}

/// FR-GILT-15/18/20: every recipe's instruction inspectors and its
/// separately requested source view, including Dirty `None`.
#[test]
fn instruction_views_and_inspectors() -> TestResult {
    let fixture = pinned_fixture()?;
    let mut m = Mismatches::default();
    let mut outcomes: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows(&fixture)? {
        if row.op != "instruction_view" {
            continue;
        }
        bump(&mut outcomes, row.expected);
        let id = row.id;
        let gilt = Definition::parse(&row)?.valid(id)?;
        let plan = plan(&row, &gilt)?;
        let inputs = CashInputs::parse(&row)?;
        let instruction = match inputs.build(&gilt, Some(&plan))? {
            Ok(instruction) => instruction,
            Err(error) => {
                m.fail(id, format!("instruction factory failed: {error:?}"));
                continue;
            }
        };
        inputs.inspect(&mut m, id, &gilt, Some(&plan), &instruction)?;
        let view =
            instruction.source_value_per_100(inputs.rounding.quantum(), inputs.rounding.mode());
        match row.expected {
            "some" => {
                m.check(id, "lower equals upper", row.lower, row.upper);
                m.check(id, "source view", view, Ok(Some(decimal(row.lower)?)));
            }
            "none" => m.check(id, "source view", view, Ok(None)),
            _ => match &view {
                Ok(value) => m.fail(id, format!("expected failure, got {value:?}")),
                Err(error) => m.error(&row, error)?,
            },
        }
    }
    assert_eq!(
        counter_text(&outcomes),
        "gilt.cash_arithmetic=4;none=7;some=24"
    );
    m.finish("instruction view")
}

/// FR-GILT-15..20: all four cash recipes: factory precedence, every
/// inspector, determined receipts and price-ingredient guarantees,
/// ambiguity, universal Exact, staged arithmetic failures and root wrappers.
#[test]
fn cash_recipes_decisions_receipts_and_failures() -> TestResult {
    let fixture = pinned_fixture()?;
    let mut m = Mismatches::default();
    let mut outcomes: BTreeMap<String, usize> = BTreeMap::new();
    let mut exact_sources = 0;
    let mut qualified_sources = 0;
    for row in rows(&fixture)? {
        if row.op != "cash" {
            continue;
        }
        bump(&mut outcomes, row.expected);
        let id = row.id;
        let gilt = Definition::parse(&row)?.valid(id)?;
        let plan = if row.settlement == "-" {
            None
        } else {
            Some(plan(&row, &gilt)?)
        };
        let inputs = CashInputs::parse(&row)?;
        let instruction = match (inputs.build(&gilt, plan.as_ref())?, row.expected) {
            (Err(error), "gilt.invalid_dividend_index" | "gilt.nonpositive_nominal") => {
                m.error(&row, &error)?;
                continue;
            }
            (Ok(instruction), "gilt.invalid_dividend_index" | "gilt.nonpositive_nominal") => {
                m.fail(
                    id,
                    format!("expected factory failure, built {instruction:?}"),
                );
                continue;
            }
            (Err(error), _) => {
                m.fail(id, format!("cash factory failed: {error:?}"));
                continue;
            }
            (Ok(instruction), _) => instruction,
        };
        inputs.inspect(&mut m, id, &gilt, plan.as_ref(), &instruction)?;
        if row.source_value != "-" {
            let (expected, step) = exact_view_quantum(row.source_value)?;
            if expected.is_some() {
                exact_sources += 1;
            } else {
                qualified_sources += 1;
            }
            let stage = if inputs.recipe == Recipe::Dividend {
                "source dividend"
            } else {
                "source accrued"
            };
            m.exact_view(
                id,
                "exact source view",
                instruction.source_value_per_100(step, AmountRounding::Exact),
                expected,
                stage,
            );
        }
        let decision = instruction.evaluate();
        m.check(
            id,
            "repeated evaluation",
            &instruction.evaluate(),
            &decision,
        );
        match (row.expected, decision) {
            ("determined", Ok(GiltCashDecision::Determined(receipt))) => {
                m.check(id, "lower equals upper", row.lower, row.upper);
                m.check(id, "amount", cell(receipt.amount()), row.lower.to_owned());
                m.check(
                    id,
                    "receipt instruction",
                    receipt.instruction(),
                    &instruction,
                );
                let guarantee = receipt.price_evidence_guarantee();
                m.check(
                    id,
                    "guarantee",
                    guarantee.map_or("-", SettlementGuarantee::canonical_identifier),
                    row.guarantee,
                );
                m.check(id, "guarantee rule", guarantee, inputs.guarantee(&row));
            }
            (
                "ambiguous",
                Ok(GiltCashDecision::QuantumAmbiguous {
                    lower_amount,
                    upper_amount,
                    ..
                }),
            ) => {
                m.check(
                    id,
                    "ambiguous lower",
                    cell(lower_amount),
                    row.lower.to_owned(),
                );
                m.check(
                    id,
                    "ambiguous upper",
                    cell(upper_amount),
                    row.upper.to_owned(),
                );
                m.check(id, "strict order", lower_amount < upper_amount, true);
                m.check(id, "no guarantee", row.guarantee, "-");
            }
            ("exact_unresolved", Ok(GiltCashDecision::ExactStageUnresolved { stage, .. })) => {
                let expected = unhex(context(row.context)?.get("stage").ok_or("stage")?)?;
                m.check(id, "unresolved stage", stage, expected.as_str());
            }
            ("gilt.cash_arithmetic", Err(error)) => m.error(&row, &error)?,
            (expected, actual) => m.fail(id, format!("expected {expected}, got {actual:?}")),
        }
    }
    assert_eq!(
        counter_text(&outcomes),
        "ambiguous=97;determined=2589;exact_unresolved=119;gilt.cash_arithmetic=2728;\
         gilt.invalid_dividend_index=2;gilt.nonpositive_nominal=16"
    );
    assert_eq!(exact_sources + qualified_sources, 1_476);
    m.finish("cash")
}
