//! FS1 integrity for the finite M8 enclosure references (FR-ENCL-16/17).
//!
//! These tests validate transport, field types, identities and cross-links.
//! They do not evaluate financial equations, certificates or native prices.
//! One test loads the tables sequentially; the other streams code/asset pins.

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::io::Read;
use std::path::Path;
use support::fixture::Fixture;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const SOURCE_COLUMNS: [&str; 5] = ["id", "model", "families", "input", "source_summary"];
const PRICE_COLUMNS: [&str; 20] = [
    "id",
    "source_id",
    "model",
    "basis",
    "yield_bits",
    "reference_kind",
    "reference_n",
    "reference_d",
    "reference_precision",
    "reference_lower_sign",
    "reference_lower_m",
    "reference_lower_k",
    "reference_upper_sign",
    "reference_upper_m",
    "reference_upper_k",
    "outward_lower_bits",
    "outward_upper_bits",
    "point_encoding",
    "point_value",
    "price_certificate_id",
];
const ADMISSION_COLUMNS: [&str; 13] = [
    "id",
    "model",
    "source_id",
    "yield_bits",
    "predicate",
    "canonical_quote_bits",
    "attempted_quote_bits",
    "frequency",
    "original_flow_index",
    "exact_affine_n",
    "exact_affine_d",
    "tau_days",
    "witnesses",
];
const HAND_COLUMNS: [&str; 21] = [
    "id",
    "context_id",
    "model",
    "basis",
    "input",
    "source_exact",
    "metadata_expected",
    "reference_kind",
    "lower_n",
    "lower_d",
    "upper_n",
    "upper_d",
    "outward_lower_bits",
    "outward_upper_bits",
    "certificate_id",
    "canonical_decimal_point",
    "declared_test_encoding",
    "equation",
    "derivation",
    "future_work_policy",
    "preferred_encoding_qualification",
];
const CASH_COLUMNS: [&str; 10] = [
    "id",
    "case",
    "basis",
    "recipe",
    "nominal",
    "quantum",
    "mode",
    "aggregation",
    "evidence",
    "expected",
];
const HYBRID_COLUMNS: [&str; 27] = [
    "id",
    "method",
    "quote_frequency",
    "m",
    "threshold_bits",
    "threshold_bit_offset",
    "label",
    "issue",
    "settlement",
    "payment",
    "face_literal",
    "redemption_literal",
    "trial_yield_literal",
    "accrual_frequency",
    "accrual_events_per_year",
    "periods",
    "overlap_days",
    "reference_days",
    "exact_unrounded_horizon",
    "expected_horizon_bits",
    "expected_branch_from_adopted_rule",
    "expected_dirty_amount_bits",
    "expected_moment_and_macaulay_bits",
    "expected_entitled_flows",
    "expected_accrual_work",
    "neumaier_trace",
    "market_scope",
];
const BOUNDARY_COLUMNS: [&str; 33] = [
    "id",
    "model",
    "context_id",
    "source_plan_id",
    "admission_case_id",
    "basis",
    "compounding",
    "yield_bits",
    "input",
    "face_bits",
    "accrued_bits",
    "coupon_amount_bits",
    "flows",
    "target_kind",
    "numerator",
    "denominator",
    "coefficient_n",
    "coefficient_d",
    "argument_n",
    "argument_d",
    "lower_n",
    "lower_d",
    "upper_n",
    "upper_d",
    "lower_strict",
    "upper_strict",
    "lower_bits",
    "upper_bits",
    "range_class",
    "point_kind",
    "point_coefficient",
    "point_scale",
    "point_bits",
];

fn fingerprint(path: &Path) -> TestResult<u64> {
    #[expect(
        clippy::disallowed_methods,
        reason = "test streams committed reference files (NFR-13)"
    )]
    let mut file = std::fs::File::open(path)?;
    let mut buffer = [0_u8; 8192];
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        for byte in &buffer[..count] {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    Ok(hash)
}

fn table(name: &str, columns: &[&str], count: usize, pin: u64) -> TestResult<Fixture> {
    let path = Fixture::repository_path(&format!(
        "validation/fixtures/bonds/enclosure-{name}.fixture"
    ));
    assert_eq!(fingerprint(&path)?, pin, "{name} whole-file fingerprint");
    let fixture = Fixture::load(path)?;
    assert_eq!(
        fixture.columns(),
        columns.iter().map(ToString::to_string).collect::<Vec<_>>()
    );
    assert_eq!(fixture.record_count(), count);
    assert_eq!(
        fixture.header("record-count"),
        Some(count.to_string().as_str())
    );
    assert_eq!(fixture.header("domain"), Some("bond-price-enclosure"));
    Ok(fixture)
}

fn header(fixture: &Fixture, key: &str, value: &str) {
    assert_eq!(fixture.header(key), Some(value), "{key}");
}

fn integer(text: &str, positive: bool) -> TestResult {
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty()
        || !digits.bytes().all(|b| b.is_ascii_digit())
        || (digits.len() > 1 && digits.starts_with('0'))
        || text == "-0"
    {
        return Err(format!("invalid canonical decimal integer {text}").into());
    }
    if positive && (text.starts_with('-') || text == "0") {
        return Err("nonpositive denominator/count".into());
    }
    Ok(())
}

fn hex_integer(text: &str, signed: bool) -> TestResult<usize> {
    let digits = if signed {
        text.strip_prefix('-').unwrap_or(text)
    } else {
        text
    };
    if digits.is_empty()
        || !digits
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || (digits.len() > 1 && digits.starts_with('0'))
        || text == "-0"
    {
        return Err(format!("invalid canonical radix16 integer {text}").into());
    }
    if digits == "0" {
        return Ok(0);
    }
    let first = u8::from_str_radix(&digits[..1], 16)?;
    Ok((digits.len() - 1) * 4 + usize::try_from(8 - first.leading_zeros())?)
}

fn bits(text: &str, finite: bool) -> TestResult<u64> {
    if text.len() != 16
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(format!("invalid binary64 field {text}").into());
    }
    let value = u64::from_str_radix(text, 16)?;
    if finite {
        assert!(f64::from_bits(value).is_finite());
    }
    Ok(value)
}

fn numbered(text: &str, prefix: &str, width: usize) -> TestResult<usize> {
    let number = text
        .strip_prefix(prefix)
        .ok_or("wrong reference ID prefix")?;
    assert_eq!(number.len(), width);
    assert!(number.bytes().all(|b| b.is_ascii_digit()));
    let value: usize = number.parse()?;
    assert!(value > 0);
    Ok(value)
}

fn decimal(coefficient: &str, scale: &str) -> TestResult {
    integer(coefficient, false)?;
    let coefficient: i128 = coefficient.parse()?;
    let scale: u8 = scale.parse()?;
    assert!(scale <= 38 && (scale == 0 || coefficient % 10 != 0));
    Ok(())
}

fn decimal_literal(text: &str) -> TestResult {
    let digits = text.strip_prefix('-').unwrap_or(text);
    let mut split = digits.split('.');
    let whole = split.next().ok_or("missing decimal whole part")?;
    assert!(!whole.is_empty() && whole.bytes().all(|b| b.is_ascii_digit()));
    if let Some(fraction) = split.next() {
        assert!(!fraction.is_empty() && fraction.bytes().all(|b| b.is_ascii_digit()));
    }
    assert!(split.next().is_none());
    Ok(())
}

fn date_literal(text: &str) -> TestResult {
    assert_eq!(text.len(), 10);
    for (i, byte) in text.bytes().enumerate() {
        assert!(if matches!(i, 4 | 7) {
            byte == b'-'
        } else {
            byte.is_ascii_digit()
        });
    }
    Ok(())
}

#[derive(Debug)]
struct Node<'a> {
    tag: &'a str,
    value: &'a str,
    declared: usize,
    children: usize,
}
#[derive(Debug)]
struct Flat<'a> {
    nodes: BTreeMap<&'a str, Node<'a>>,
}

impl<'a> Flat<'a> {
    fn parse(cell: &'a str) -> TestResult<Self> {
        let mut nodes: BTreeMap<&str, Node<'_>> = BTreeMap::new();
        for (position, part) in cell.split(';').enumerate() {
            let (path, typed) = part.split_once('=').ok_or("typed node missing equals")?;
            let (tag, value) = typed.split_once(':').ok_or("typed node missing tag")?;
            assert!(!nodes.contains_key(path));
            let declared = match tag {
                "d" | "l" => {
                    integer(value, false)?;
                    value.parse::<usize>()?
                }
                "i" => {
                    integer(value, false)?;
                    0
                }
                "s" => {
                    assert!(value.is_ascii() && !value.contains('='));
                    0
                }
                "b" => {
                    assert!(matches!(value, "0" | "1"));
                    0
                }
                "z" => {
                    assert_eq!(value, "_");
                    0
                }
                other => return Err(format!("unknown typed node tag {other}").into()),
            };
            if path == "$" {
                assert_eq!(position, 0);
                assert!(matches!(tag, "d" | "l"));
            } else {
                let (parent, list_index) = if path.ends_with(']') {
                    let at = path.rfind('[').ok_or("unmatched list index")?;
                    let index = &path[at + 1..path.len() - 1];
                    integer(index, false)?;
                    (
                        if at == 0 { "$" } else { &path[..at] },
                        Some(index.parse::<usize>()?),
                    )
                } else {
                    let (parent, field) = path.rsplit_once('.').unwrap_or(("$", path));
                    assert!(
                        !field.is_empty()
                            && field.bytes().enumerate().all(|(i, b)| b == b'_'
                                || b.is_ascii_alphabetic()
                                || i != 0 && b.is_ascii_digit())
                    );
                    (parent, None)
                };
                let parent = nodes
                    .get_mut(parent)
                    .ok_or("typed node parent missing/reordered")?;
                match list_index {
                    Some(index) => {
                        assert_eq!(parent.tag, "l");
                        assert_eq!(index, parent.children);
                    }
                    None => assert_eq!(parent.tag, "d"),
                }
                parent.children += 1;
            }
            nodes.insert(
                path,
                Node {
                    tag,
                    value,
                    declared,
                    children: 0,
                },
            );
        }
        assert!(nodes.contains_key("$"));
        for node in nodes.values() {
            if matches!(node.tag, "d" | "l") {
                assert_eq!(node.declared, node.children);
            }
        }
        Ok(Self { nodes })
    }
    fn node(&self, path: &str) -> TestResult<&Node<'a>> {
        self.nodes
            .get(path)
            .ok_or_else(|| format!("missing node {path}").into())
    }
    fn string(&self, path: &str) -> TestResult<&'a str> {
        let node = self.node(path)?;
        assert_eq!(node.tag, "s");
        Ok(node.value)
    }
    fn count(&self, path: &str) -> TestResult<usize> {
        let node = self.node(path)?;
        assert_eq!(node.tag, "l");
        Ok(node.declared)
    }
    fn unsigned(&self, path: &str) -> TestResult<usize> {
        let node = self.node(path)?;
        assert_eq!(node.tag, "i");
        Ok(node.value.parse()?)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Model {
    Generic,
    Gilt,
    Treasury,
    Canadian,
}
impl Model {
    fn parse(text: &str) -> TestResult<Self> {
        match text {
            "Generic" | "generic" => Ok(Self::Generic),
            "Gilt" | "gilt" => Ok(Self::Gilt),
            "Treasury" | "treasury" => Ok(Self::Treasury),
            "Canadian" | "canadian" => Ok(Self::Canadian),
            other => Err(format!("unknown model {other}").into()),
        }
    }
}
#[derive(Debug)]
struct Source {
    model: Model,
    original_generic_indices: Vec<usize>,
}
fn source<'a>(id: &str, sources: &'a [Source]) -> TestResult<(usize, &'a Source)> {
    let index = numbered(id, "ENC-SOURCE-", 6)? - 1;
    Ok((index, sources.get(index).ok_or("unknown source ID")?))
}
fn basis(model: Model, value: &str) {
    assert!(if model == Model::Canadian {
        matches!(value, "Clean" | "ModelDirty" | "SettlementDirty")
    } else {
        matches!(value, "Dirty" | "Clean")
    });
}

fn sources() -> TestResult<Vec<Source>> {
    let fixture = table("sources", &SOURCE_COLUMNS, 30920, 0x35e5_0285_4438_66df)?;
    header(
        &fixture,
        "generator-sha256",
        "749f6271990a31aa2cdd6d43b4d6a259310a79ac3056cac5efea18a69454249d",
    );
    header(
        &fixture,
        "source-logical-sha256",
        "71d7464c09c4f55998222e0c37b3c69af3689c0016743f26627b1c199b68e24e",
    );
    header(
        &fixture,
        "source-transport-sha256",
        "d4119eae47ce773310a9e5073bb862387f7f12b10564f8f1d09afb9c4eeeb8a3",
    );
    let mut result = Vec::with_capacity(30920);
    let mut models = BTreeMap::new();
    for (position, row) in fixture.records().enumerate() {
        assert_eq!(numbered(row.id(), "ENC-SOURCE-", 6)?, position + 1);
        let model = Model::parse(row.text("model")?)?;
        *models.entry(model).or_insert(0_usize) += 1;
        assert!(
            row.text("families")?
                .split(',')
                .all(|part| !part.is_empty())
        );
        let input = Flat::parse(row.text("input")?)?;
        let summary = Flat::parse(row.text("source_summary")?)?;
        for flat in [&input, &summary] {
            for (path, node) in &flat.nodes {
                if node.tag == "s" && path.ends_with("_bits") {
                    bits(node.value, true)?;
                }
                if node.tag == "s" && path.ends_with(".n") {
                    integer(node.value, false)?;
                }
                if node.tag == "s" && path.ends_with(".d") {
                    integer(node.value, true)?;
                }
            }
        }
        let mut original_generic_indices = Vec::new();
        if model == Model::Generic {
            bits(input.string("face_bits")?, true)?;
            bits(summary.string("cached_accrued_bits")?, true)?;
            let count = summary.count("prepared_flows")?;
            assert_eq!(summary.count("selected_branches")?, count);
            for i in 0..count {
                original_generic_indices
                    .push(summary.unsigned(&format!("prepared_flows[{i}].index"))?);
                bits(
                    summary.string(&format!("prepared_flows[{i}].amount_bits"))?,
                    true,
                )?;
                bits(
                    summary.string(&format!("prepared_flows[{i}].horizon_bits"))?,
                    true,
                )?;
                date_literal(summary.string(&format!("prepared_flows[{i}].payment"))?)?;
                assert!(matches!(
                    summary.string(&format!("selected_branches[{i}]"))?,
                    "Simple" | "Continuous" | "Compounded"
                ));
            }
            assert!(
                original_generic_indices
                    .windows(2)
                    .all(|pair| pair[0] < pair[1])
            );
        } else {
            let count = summary.unsigned("prepared_bundle_count")?;
            assert_eq!(summary.count("prepared_bundle_indices")?, count);
            assert_eq!(summary.count("prepared_payment_dates")?, count);
            for i in 0..count {
                summary.unsigned(&format!("prepared_bundle_indices[{i}]"))?;
                date_literal(summary.string(&format!("prepared_payment_dates[{i}]"))?)?;
            }
        }
        result.push(Source {
            model,
            original_generic_indices,
        });
    }
    assert_eq!(
        models,
        BTreeMap::from([
            (Model::Generic, 8960),
            (Model::Gilt, 300),
            (Model::Treasury, 2160),
            (Model::Canadian, 19500)
        ])
    );
    Ok(result)
}

fn prices(sources: &[Source]) -> TestResult {
    // The actual first reader rejected v1's uppercase column names. V2
    // changes transport metadata/labels only; all173448 data rows are exact.
    let fixture = table("prices-v2", &PRICE_COLUMNS, 173448, 0x4e32_c482_2641_b858)?;
    header(&fixture, "transport-version", "2");
    header(
        &fixture,
        "generator-sha256",
        "ec8147d94447752edf3a3318c5a3b68be1031c9d8bcdb10277882402bd6766ab",
    );
    header(
        &fixture,
        "provenance-sha256",
        "033e53d520cf504e0395ebc1ba7affd4e29280ef8c62f6dc53f0236f8fda6696",
    );
    header(
        &fixture,
        "original-prices-sha256",
        "726ed651412d0c78e39e4ec6575d8debbcd8ed5bc5a6473bab56f9f031236d2b",
    );
    header(
        &fixture,
        "original-price-certificates-sha256",
        "f008d286fd7ff16e5ae6b4e54ce9a20c8a6032619b9284bbb6bb0113a069a3bc",
    );
    let mut models = BTreeMap::new();
    let mut kinds = BTreeMap::new();
    let mut points = BTreeMap::new();
    let mut used_sources = vec![false; sources.len()];
    let mut certificates = vec![None; 65964];
    for (position, row) in fixture.records().enumerate() {
        assert_eq!(numbered(row.id(), "ENC-PRICE-", 7)?, position + 1);
        let (index, src) = source(row.text("source_id")?, sources)?;
        let model = Model::parse(row.text("model")?)?;
        assert_eq!(model, src.model);
        basis(model, row.text("basis")?);
        used_sources[index] = true;
        let quote = bits(row.text("yield_bits")?, true)?;
        let cert_index = numbered(row.text("price_certificate_id")?, "FPRICE-", 7)? - 1;
        let binding = certificates
            .get_mut(cert_index)
            .ok_or("unknown price certificate ID")?;
        let expected = (index, model, quote);
        if let Some(previous) = binding {
            assert_eq!(*previous, expected);
        } else {
            *binding = Some(expected);
        }
        *models.entry(model).or_insert(0_usize) += 1;
        *kinds.entry(row.text("reference_kind")?).or_insert(0_usize) += 1;
        for side in ["lower", "upper"] {
            let value = row.text(&format!("outward_{side}_bits"))?;
            if value != "-" {
                bits(value, true)?;
            }
        }
        match row.text("reference_kind")? {
            "exact_rational" => {
                // Parse the target before inspecting point encoding. NoFit
                // must never turn a malformed rational into a skipped case.
                integer(row.text("reference_n")?, false)?;
                integer(row.text("reference_d")?, true)?;
                for name in [
                    "reference_precision",
                    "reference_lower_sign",
                    "reference_lower_m",
                    "reference_lower_k",
                    "reference_upper_sign",
                    "reference_upper_m",
                    "reference_upper_k",
                ] {
                    assert_eq!(row.text(name)?, "-");
                }
            }
            "certified_interval" => {
                assert_eq!(
                    (row.text("reference_n")?, row.text("reference_d")?),
                    ("-", "-")
                );
                let precision: usize = row.parse("reference_precision")?;
                assert_eq!(precision, 1120);
                for side in ["lower", "upper"] {
                    let sign: i8 = row.parse(&format!("reference_{side}_sign"))?;
                    assert!(matches!(sign, -1..=1));
                    let mantissa = row.text(&format!("reference_{side}_m"))?;
                    let exponent = row.text(&format!("reference_{side}_k"))?;
                    let width = hex_integer(mantissa, false)?;
                    hex_integer(exponent, true)?; // Signed HEX, including digit-only K.
                    if sign == 0 {
                        assert_eq!((mantissa, exponent), ("0", "0"));
                    } else {
                        assert_eq!(width, precision);
                    }
                }
            }
            other => return Err(format!("unknown price reference kind {other}").into()),
        }
        let encoding = row.text("point_encoding")?;
        let value = row.text("point_value")?;
        *points.entry(encoding).or_insert(0_usize) += 1;
        match encoding {
            "Decimal" => {
                let (coefficient, scale) =
                    value.split_once(':').ok_or("decimal point separator")?;
                decimal(coefficient, scale)?;
                assert_eq!(row.text("reference_kind")?, "exact_rational");
            }
            "Binary" => {
                bits(value, true)?;
                assert_eq!(row.text("reference_kind")?, "exact_rational");
            }
            "NoFittingPoint" => {
                assert_eq!(value, "-");
                assert_eq!(row.text("reference_kind")?, "exact_rational");
            }
            "NotProvedInterval" => {
                assert_eq!(value, "-");
                assert_eq!(row.text("reference_kind")?, "certified_interval");
            }
            other => return Err(format!("unknown point encoding {other}").into()),
        }
    }
    assert!(used_sources.into_iter().all(|used| used));
    assert!(certificates.into_iter().all(|binding| binding.is_some()));
    assert_eq!(
        models,
        BTreeMap::from([
            (Model::Generic, 38400),
            (Model::Gilt, 1464),
            (Model::Treasury, 9024),
            (Model::Canadian, 124560)
        ])
    );
    assert_eq!(
        kinds,
        BTreeMap::from([("exact_rational", 136096), ("certified_interval", 37352)])
    );
    assert_eq!(
        points,
        BTreeMap::from([
            ("Decimal", 32168),
            ("NoFittingPoint", 103928),
            ("NotProvedInterval", 37352)
        ])
    );
    Ok(())
}

#[derive(Debug)]
struct Admitted {
    source_index: usize,
    model: Model,
    quote: u64,
}
fn admission(sources: &[Source]) -> TestResult<BTreeMap<String, Admitted>> {
    let fixture = table(
        "admission",
        &ADMISSION_COLUMNS,
        119631,
        0x5185_a7ad_71e8_54d9,
    )?;
    header(
        &fixture,
        "generator-sha256",
        "82c75fdd86f9cc17b1610bbb9a2ce917617035fe40213492f42f2e0b6237f653",
    );
    header(
        &fixture,
        "provenance-sha256",
        "e1e27373fb26cd2da0b9a5e15f4f09ca7c9942c9800e99fa66b32060b1f20b00",
    );
    header(
        &fixture,
        "original-admission-sha256",
        "58867b875e85880e451423340ae50616bc35c3dfae005698aaea07588c2f40b9",
    );
    header(&fixture, "distinct-source-count", "7410");
    let mut selected = BTreeMap::new();
    let mut models = BTreeMap::new();
    let mut predicates = BTreeMap::new();
    let mut used_sources = vec![false; sources.len()];
    for (position, row) in fixture.records().enumerate() {
        assert_eq!(numbered(row.id(), "ENC-ADMISSION-", 7)?, position + 1);
        let (index, src) = source(row.text("source_id")?, sources)?;
        let model = Model::parse(row.text("model")?)?;
        assert_eq!(model, src.model);
        used_sources[index] = true;
        let quote = bits(row.text("yield_bits")?, false)?;
        let predicate = row.text("predicate")?;
        *models.entry(model).or_insert(0_usize) += 1;
        *predicates.entry(predicate).or_insert(0_usize) += 1;
        let mut present = BTreeSet::new();
        match predicate {
            "MathematicallyAdmitted" => {
                let canonical = bits(row.text("canonical_quote_bits")?, true)?;
                assert_eq!(
                    canonical,
                    if quote & !(1_u64 << 63) == 0 {
                        0
                    } else {
                        quote
                    }
                );
                present.insert("canonical_quote_bits");
            }
            "NonFiniteYield" => {
                assert_eq!(bits(row.text("attempted_quote_bits")?, false)?, quote);
                assert!(!f64::from_bits(quote).is_finite());
                present.insert("attempted_quote_bits");
            }
            "YieldNotAboveFloor" => {
                assert!(row.parse::<u16>("frequency")? > 0);
                present.insert("frequency");
            }
            "NonPositiveSimpleDenominator" => {
                assert_eq!(model, Model::Generic);
                let ordinal: usize = row.parse("original_flow_index")?;
                assert!(src.original_generic_indices.contains(&ordinal));
                present.insert("original_flow_index");
                present.extend(["exact_affine_n", "exact_affine_d"]);
            }
            "NonPositiveMoneyMarketDenominator" => {
                assert_eq!(model, Model::Canadian);
                assert!(row.parse::<u32>("tau_days")? > 0);
                present.insert("tau_days");
                present.extend(["exact_affine_n", "exact_affine_d"]);
            }
            other => return Err(format!("unknown admission predicate {other}").into()),
        }
        if present.contains("exact_affine_n") {
            integer(row.text("exact_affine_n")?, false)?;
            integer(row.text("exact_affine_d")?, true)?;
        }
        for field in [
            "canonical_quote_bits",
            "attempted_quote_bits",
            "frequency",
            "original_flow_index",
            "exact_affine_n",
            "exact_affine_d",
            "tau_days",
        ] {
            if !present.contains(field) {
                assert_eq!(row.text(field)?, "-");
            }
        }
        if row.text("witnesses")? != "-" {
            for witness in row.text("witnesses")?.split(',') {
                assert!(
                    !witness.is_empty()
                        && witness
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                );
            }
        }
        if matches!(
            row.id(),
            "ENC-ADMISSION-0013165" | "ENC-ADMISSION-0014035" | "ENC-ADMISSION-0016090"
        ) {
            assert_eq!(predicate, "MathematicallyAdmitted");
            selected.insert(
                row.id().to_owned(),
                Admitted {
                    source_index: index,
                    model,
                    quote,
                },
            );
        }
    }
    assert_eq!(used_sources.into_iter().filter(|used| *used).count(), 7410);
    assert_eq!(selected.len(), 3);
    assert_eq!(
        models,
        BTreeMap::from([
            (Model::Generic, 13161),
            (Model::Gilt, 870),
            (Model::Treasury, 2040),
            (Model::Canadian, 103560)
        ])
    );
    assert_eq!(
        predicates,
        BTreeMap::from([
            ("MathematicallyAdmitted", 61846),
            ("NonFiniteYield", 29640),
            ("YieldNotAboveFloor", 14499),
            ("NonPositiveSimpleDenominator", 532),
            ("NonPositiveMoneyMarketDenominator", 13114)
        ])
    );
    Ok(selected)
}

fn boundary(sources: &[Source], selected: &BTreeMap<String, Admitted>) -> TestResult {
    let fixture = table("boundary", &BOUNDARY_COLUMNS, 35, 0x2521_a815_7abb_cf0c)?;
    header(
        &fixture,
        "family-sha256",
        "295247d3700a0a06c7fd1d9bde830740cb4a66bd927a38061e255150b8eb52aa",
    );
    header(
        &fixture,
        "author-sha256",
        "45be375751e10c84c221189896d2908956b178a440214220339af0913ab60c76",
    );
    header(
        &fixture,
        "checker-sha256",
        "bd5a90a0e405805cfcba70506dcac35030604f86f9f1f6866c4802c368f8152a",
    );
    header(
        &fixture,
        "reused-source-sha256",
        "0e365b4f24c451886702a89719fe279a561a538e4ae437437241d6e57266bf07",
    );
    header(
        &fixture,
        "reused-admission-sha256",
        "adfa6155113b77678a8c7d0262e0a83dae48d7ab8a749464bb3750983a6a3c3d",
    );
    let mut source_plans = BTreeMap::new();
    let mut contexts = BTreeSet::new();
    let mut quotes = BTreeSet::new();
    let mut named = BTreeSet::new();
    let mut kinds = BTreeMap::new();
    for row in fixture.records() {
        let model = Model::parse(row.text("model")?)?;
        basis(model, row.text("basis")?);
        let (quote_id, suffix) = row.id().rsplit_once(':').ok_or("B2 basis suffix missing")?;
        assert_eq!(suffix, row.text("basis")?);
        quotes.insert(quote_id);
        let quote = bits(row.text("yield_bits")?, true)?;
        let mut literal_keys = BTreeSet::new();
        for node in row.text("input")?.split(';') {
            let (key, value) = node.split_once('=').ok_or("B2 input separator")?;
            assert!(!key.is_empty() && !value.is_empty() && literal_keys.insert(key));
        }
        if model == Model::Generic {
            let context = row.text("context_id")?;
            contexts.insert(context);
            assert_eq!(
                row.text("source_plan_id")?,
                format!("B2-SOURCE:{context}:{}", row.text("compounding")?)
            );
            assert!(matches!(row.text("compounding")?, "Simple" | "Continuous"));
            assert_eq!(row.text("admission_case_id")?, "_");
            bits(row.text("face_bits")?, true)?;
            assert_eq!(row.text("accrued_bits")?, "0000000000000000");
            let coupon_count = if row.text("coupon_amount_bits")? == "_" {
                0
            } else {
                let cells = row
                    .text("coupon_amount_bits")?
                    .split(',')
                    .collect::<Vec<_>>();
                for cell in &cells {
                    bits(cell, true)?;
                }
                cells.len()
            };
            let mut flow_count = 0;
            for (ordinal, flow) in row.text("flows")?.split(';').enumerate() {
                let columns = flow.split('@').collect::<Vec<_>>();
                assert_eq!(columns.len(), 4);
                assert_eq!(columns[0].parse::<usize>()?, ordinal);
                date_literal(columns[1])?;
                bits(columns[2], true)?;
                bits(columns[3], true)?;
                flow_count += 1;
            }
            assert_eq!(flow_count, coupon_count + 1);
            let fields = [
                "context_id",
                "compounding",
                "input",
                "face_bits",
                "accrued_bits",
                "coupon_amount_bits",
                "flows",
            ]
            .map(|key| row.text(key))
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
            if let Some(previous) = source_plans.insert(row.text("source_plan_id")?, fields.clone())
            {
                assert_eq!(previous, fields);
            }
        } else {
            let (index, source) = source(row.text("source_plan_id")?, sources)?;
            assert_eq!(source.model, model);
            let admitted = selected
                .get(row.text("admission_case_id")?)
                .ok_or("B2 admission link missing")?;
            assert_eq!(
                (admitted.source_index, admitted.model, admitted.quote),
                (index, model, quote)
            );
            for key in ["face_bits", "accrued_bits", "coupon_amount_bits", "flows"] {
                assert_eq!(row.text(key)?, "_");
            }
            named.insert(row.text("source_plan_id")?);
        }
        for key in ["lower_bits", "upper_bits", "point_bits"] {
            if row.text(key)? != "_" {
                bits(row.text(key)?, true)?;
            }
        }
        let kind = row.text("target_kind")?;
        *kinds.entry(kind).or_insert(0_usize) += 1;
        match kind {
            "exact_rational" => {
                integer(row.text("numerator")?, false)?;
                integer(row.text("denominator")?, true)?;
                for key in [
                    "coefficient_n",
                    "coefficient_d",
                    "argument_n",
                    "argument_d",
                    "lower_n",
                    "lower_d",
                    "upper_n",
                    "upper_d",
                    "lower_strict",
                    "upper_strict",
                ] {
                    assert_eq!(row.text(key)?, "_");
                }
            }
            "strict_range" => {
                assert_eq!(
                    (row.text("numerator")?, row.text("denominator")?),
                    ("_", "_")
                );
                for prefix in ["coefficient", "argument", "lower", "upper"] {
                    let n = row.text(&format!("{prefix}_n"))?;
                    let d = row.text(&format!("{prefix}_d"))?;
                    assert_eq!(n == "_", d == "_");
                    if n != "_" {
                        integer(n, false)?;
                        integer(d, true)?;
                    }
                }
                assert!(matches!(row.text("lower_strict")?, "0" | "1"));
                assert!(matches!(row.text("upper_strict")?, "0" | "1"));
            }
            other => return Err(format!("unknown B2 target kind {other}").into()),
        }
        assert!(matches!(
            row.text("range_class")?,
            "finite"
                | "zero"
                | "positive_sub_min"
                | "negative_sub_min"
                | "above_max"
                | "below_negative_max"
        ));
        match row.text("point_kind")? {
            "decimal" => {
                decimal(row.text("point_coefficient")?, row.text("point_scale")?)?;
                assert_eq!(row.text("point_bits")?, "_");
            }
            "binary" => {
                bits(row.text("point_bits")?, true)?;
                assert_eq!(
                    (row.text("point_coefficient")?, row.text("point_scale")?),
                    ("_", "_")
                );
            }
            "none" | "unclassified" => {
                assert_eq!(
                    (
                        row.text("point_coefficient")?,
                        row.text("point_scale")?,
                        row.text("point_bits")?
                    ),
                    ("_", "_", "_")
                );
            }
            other => return Err(format!("unknown B2 point class {other}").into()),
        }
    }
    assert_eq!(
        (
            source_plans.len(),
            contexts.len(),
            quotes.len(),
            named.len()
        ),
        (11, 10, 17, 3)
    );
    assert_eq!(
        kinds,
        BTreeMap::from([("exact_rational", 29), ("strict_range", 6)])
    );
    Ok(())
}

#[derive(Debug)]
struct Hand {
    model: Model,
    bases: BTreeSet<String>,
}
fn controls_headers(fixture: &Fixture, current: bool, logical: &str, transport: &str) {
    if current {
        header(
            fixture,
            "generator-sha256",
            "96f81d9ae40ab8869bf4dd085712ae5d5217b600d6b58155449d98c58c81c58b",
        );
        header(
            fixture,
            "provenance-sha256",
            "de5764b302848afc3ce030de9186b26bc4104e6ec0cfc67dc82b1cffdaffaae0",
        );
    } else {
        // The actual first H/HYBRID files retain their truthful historical
        // writer/provenance. The second test pins that real recorded source.
        header(
            fixture,
            "generator-sha256",
            "b334b087e6c16957d8a20e3f3c702a058b7b16f0f06d200e24d7d3f87fa941de",
        );
        header(
            fixture,
            "provenance-sha256",
            "968cd0fb2080a9d69e57789f5a6255b7111c88f4a5a60158d562a07207eed8ec",
        );
    }
    header(
        fixture,
        "flat-helper-sha256",
        "749f6271990a31aa2cdd6d43b4d6a259310a79ac3056cac5efea18a69454249d",
    );
    header(fixture, "source-logical-sha256", logical);
    header(fixture, "source-transport-sha256", transport);
    header(
        fixture,
        "hand-certificates-logical-sha256",
        "c200a1903b9dfecde3ca4977050b4c283a6cbaa77fa478d18723b76f27f2fbbe",
    );
}

fn hands() -> TestResult<BTreeMap<String, Hand>> {
    let fixture = table("hands", &HAND_COLUMNS, 43, 0x86af_2488_7f4c_37d8)?;
    controls_headers(
        &fixture,
        false,
        "1f47eb6e07b70b8c52d8da24583613d84c261baf6563a7c443a06f79d48c19c4",
        "d931a227560981937623b68d4f8cfa628d1d746fe9664e1d67bafdbb76d79e28",
    );
    let mut contexts: BTreeMap<String, Hand> = BTreeMap::new();
    let mut models = BTreeMap::new();
    let mut kinds = BTreeMap::new();
    for row in fixture.records() {
        let model = Model::parse(row.text("model")?)?;
        let context = row.text("context_id")?;
        let price_basis = row.text("basis")?;
        basis(model, price_basis);
        assert_eq!(row.id(), format!("{context}:{price_basis}"));
        let input = Flat::parse(row.text("input")?)?;
        bits(input.string("y_bits")?, true)?;
        Flat::parse(row.text("source_exact")?)?;
        Flat::parse(row.text("metadata_expected")?)?;
        // Every bound is parsed even when no point encoding is available.
        for side in ["lower", "upper"] {
            integer(row.text(&format!("{side}_n"))?, false)?;
            integer(row.text(&format!("{side}_d"))?, true)?;
            bits(row.text(&format!("outward_{side}_bits"))?, true)?;
        }
        let kind = row.text("reference_kind")?;
        assert!(matches!(kind, "exact_rational" | "certified_irrational"));
        *kinds.entry(kind).or_insert(0_usize) += 1;
        match row.text("certificate_id")? {
            "-" => assert_eq!(kind, "exact_rational"),
            "e" | "sqrt2" => assert_eq!(kind, "certified_irrational"),
            other => return Err(format!("unknown old hand certificate {other}").into()),
        }
        match row.text("declared_test_encoding")? {
            "decimal_point" => {
                assert_ne!(row.text("canonical_decimal_point")?, "-");
                decimal_literal(row.text("canonical_decimal_point")?)?;
            }
            "binary_bounds" => {
                if row.text("canonical_decimal_point")? != "-" {
                    decimal_literal(row.text("canonical_decimal_point")?)?;
                }
            }
            other => return Err(format!("unknown original hand encoding {other}").into()),
        }
        for key in [
            "equation",
            "derivation",
            "future_work_policy",
            "preferred_encoding_qualification",
        ] {
            assert!(!row.text(key)?.is_empty());
        }
        let entry = contexts.entry(context.to_owned()).or_insert_with(|| Hand {
            model,
            bases: BTreeSet::new(),
        });
        assert_eq!(entry.model, model);
        assert!(entry.bases.insert(price_basis.to_owned()));
        *models.entry(model).or_insert(0_usize) += 1;
    }
    assert_eq!(contexts.len(), 19);
    assert_eq!(
        models,
        BTreeMap::from([
            (Model::Generic, 8),
            (Model::Gilt, 8),
            (Model::Treasury, 12),
            (Model::Canadian, 15)
        ])
    );
    assert_eq!(
        kinds,
        BTreeMap::from([("exact_rational", 34), ("certified_irrational", 9)])
    );
    Ok(contexts)
}

fn cash(hands: &BTreeMap<String, Hand>) -> TestResult {
    let fixture = table("cash-hands", &CASH_COLUMNS, 38, 0x157d_5bd8_2fdf_317d)?;
    controls_headers(
        &fixture,
        true,
        "c61fc72d1d14b160692beb22dd93a3cb0b2b7f4a0a8370cba10e9c3fe009d3ad",
        "5e26ff3ac19923ab5085a320e47c2205412b2d7dfa90931db749eec3a5013410",
    );
    let mut modes = BTreeSet::new();
    let mut decisions = BTreeMap::new();
    let mut source_only = 0;
    for (position, row) in fixture.records().enumerate() {
        assert_eq!(numbered(row.id(), "HC-CASH-", 3)?, position + 1);
        let hand = hands
            .get(row.text("case")?)
            .ok_or("cash hand context missing")?;
        decimal_literal(row.text("nominal")?)?;
        decimal_literal(row.text("quantum")?)?;
        let mode = row.text("mode")?;
        assert!(matches!(
            mode,
            "TowardZero"
                | "AwayFromZero"
                | "TowardPositiveInfinity"
                | "TowardNegativeInfinity"
                | "NearestTiesEven"
                | "NearestTiesAwayFromZero"
                | "Exact"
        ));
        modes.insert(mode);
        assert!(matches!(
            row.text("aggregation")?,
            "-" | "RoundTotal" | "RoundComponentsThenSum"
        ));
        assert!(!row.text("evidence")?.is_empty() && !row.text("recipe")?.is_empty());
        let expected = Flat::parse(row.text("expected")?)?;
        let decision = expected.string("decision")?;
        *decisions.entry(decision).or_insert(0_usize) += 1;
        match decision {
            "determined" => decimal_literal(expected.string("amount")?)?,
            "error" => {
                assert!(!expected.string("class")?.is_empty());
                assert!(!expected.string("endpoint")?.is_empty());
                assert!(!expected.string("stage")?.is_empty());
            }
            "exact_unresolved" => assert!(!expected.string("stage")?.is_empty()),
            other => return Err(format!("unknown original cash decision {other}").into()),
        }
        if row.text("basis")? == "-" {
            source_only += 1;
            assert_eq!(row.text("recipe")?, "source_coupon");
            assert_eq!(expected.node("price_evidence_guarantee")?.tag, "z");
        } else {
            basis(hand.model, row.text("basis")?);
            assert_ne!(row.text("basis")?, "ModelDirty");
            assert!(hand.bases.contains(row.text("basis")?));
            if decision == "determined" {
                assert_eq!(
                    expected.string("price_evidence_guarantee")?,
                    "UniqueForDeclaredEnclosure"
                );
            }
        }
    }
    assert_eq!((source_only, modes.len()), (4, 7));
    assert_eq!(
        decisions,
        BTreeMap::from([("determined", 33), ("error", 1), ("exact_unresolved", 4)])
    );
    Ok(())
}

fn hybrid() -> TestResult {
    let fixture = table("hybrid", &HYBRID_COLUMNS, 144, 0x5b4a_ebfc_a46c_794a)?;
    controls_headers(
        &fixture,
        false,
        "75cbfcfd161df45b7fb1297793f16c034396f8c56397e783ad480d45bbc57404",
        "d34072999bca4134863e7fece7cb55c067864e7e0c9d7d27ca47e1b569985f1e",
    );
    let mut configurations = BTreeSet::new();
    let mut methods = BTreeMap::new();
    let mut branches = BTreeMap::new();
    for (position, row) in fixture.records().enumerate() {
        assert_eq!(numbered(row.id(), "ICMA-REACH-", 4)?, position + 1);
        for key in ["issue", "settlement", "payment"] {
            date_literal(row.text(key)?)?;
        }
        assert_eq!(row.text("trial_yield_literal")?, "0.0");
        decimal_literal(row.text("face_literal")?)?;
        decimal_literal(row.text("redemption_literal")?)?;
        let method = row.text("method")?;
        assert!(matches!(
            method,
            "Simple"
                | "Continuous"
                | "Compounded"
                | "SimpleThenCompounded"
                | "CompoundedThenSimple"
        ));
        configurations.insert((method, row.text("quote_frequency")?, row.text("m")?));
        *methods.entry(method).or_insert(0_usize) += 1;
        if row.text("m")? == "-" {
            assert_eq!(
                (
                    row.text("quote_frequency")?,
                    row.text("threshold_bits")?,
                    row.text("threshold_bit_offset")?
                ),
                ("-", "-", "-")
            );
        } else {
            assert!(row.parse::<u16>("m")? > 0);
            bits(row.text("threshold_bits")?, true)?;
            integer(row.text("threshold_bit_offset")?, false)?;
            assert_ne!(row.text("quote_frequency")?, "-");
        }
        assert!(
            !row.text("accrual_frequency")?.is_empty()
                && !row.text("label")?.is_empty()
                && !row.text("market_scope")?.is_empty()
        );
        assert!(row.parse::<u16>("accrual_events_per_year")? > 0);
        let periods = Flat::parse(row.text("periods")?)?;
        let overlaps = Flat::parse(row.text("overlap_days")?)?;
        let reference_days = Flat::parse(row.text("reference_days")?)?;
        let count = periods.count("$")?;
        assert_eq!(
            (overlaps.count("$")?, reference_days.count("$")?),
            (count, count)
        );
        for i in 0..count {
            assert_eq!(periods.count(&format!("[{i}]"))?, 2);
            date_literal(periods.string(&format!("[{i}][0]"))?)?;
            date_literal(periods.string(&format!("[{i}][1]"))?)?;
            overlaps.unsigned(&format!("[{i}]"))?;
            assert!(reference_days.unsigned(&format!("[{i}]"))? > 0);
        }
        let trace = Flat::parse(row.text("neumaier_trace")?)?;
        for i in 0..trace.count("$")? {
            for key in ["sum_bits", "term_bits", "correction_bits"] {
                bits(trace.string(&format!("[{i}].{key}"))?, true)?;
            }
        }
        let fraction = row.text("exact_unrounded_horizon")?;
        if let Some((n, d)) = fraction.split_once('/') {
            integer(n, false)?;
            integer(d, true)?;
        } else {
            integer(fraction, false)?;
        }
        for key in [
            "expected_horizon_bits",
            "expected_dirty_amount_bits",
            "expected_moment_and_macaulay_bits",
        ] {
            bits(row.text(key)?, true)?;
        }
        let branch = row.text("expected_branch_from_adopted_rule")?;
        assert!(matches!(branch, "Simple" | "Periodic" | "Continuous"));
        *branches.entry(branch).or_insert(0_usize) += 1;
        assert!(row.parse::<usize>("expected_entitled_flows")? > 0);
        assert!(row.parse::<u64>("expected_accrual_work")? > 0);
        // Selection, RNE/Neumaier values and reachability are copied facts;
        // this transport test neither recomputes them nor infers impossibility.
    }
    assert_eq!(configurations.len(), 32);
    assert_eq!(
        methods,
        BTreeMap::from([
            ("Compounded", 46),
            ("SimpleThenCompounded", 46),
            ("CompoundedThenSimple", 46),
            ("Simple", 3),
            ("Continuous", 3)
        ])
    );
    assert_eq!(
        branches,
        BTreeMap::from([("Periodic", 92), ("Simple", 49), ("Continuous", 3)])
    );
    Ok(())
}

#[test]
fn enclosure_reference_rows_are_complete_typed_and_cross_linked() -> TestResult {
    let sources = sources()?; // Its large Fixture is dropped on return.
    prices(&sources)?; // Each following helper drops its table before returning.
    let selected = admission(&sources)?;
    boundary(&sources, &selected)?;
    drop(selected);
    drop(sources);
    let hands = hands()?;
    cash(&hands)?;
    drop(hands);
    hybrid()?;
    Ok(())
}

#[test]
fn enclosure_reference_writers_and_archives_match_recorded_bytes() -> TestResult {
    // Only streaming buffers exist here; no JSON/gzip/table decoder and no
    // second simultaneous large Fixture. SHA header strings bind the known
    // provenance; FNV byte pins follow the existing repository test pattern.
    for (relative, pin) in [
        (
            "validation/generator/enclosure_sources.py",
            0x503d_ebaf_8e9f_2307,
        ),
        (
            "validation/generator/enclosure_prices.py",
            0xa13a_8389_82e6_d432,
        ),
        (
            "validation/generator/enclosure_prices.provenance.json",
            0x1a1d_4424_f469_4bd0,
        ),
        (
            "validation/generator/enclosure_admission.py",
            0x9bc8_90d7_74b2_7f7f,
        ),
        (
            "validation/generator/enclosure_admission.provenance.json",
            0x615b_9aad_f065_6535,
        ),
        (
            "validation/generator/enclosure_controls.py",
            0x30ea_1020_d795_bb2d,
        ),
        (
            "validation/generator/enclosure_controls.provenance.json",
            0xd41f_beb6_b15f_31f8,
        ),
        (
            "validation/generator/enclosure_controls_history/first-source.py.txt",
            0xb373_fb00_98a9_83d8,
        ),
        (
            "validation/generator/enclosure_controls_history/first-provenance.json",
            0x088a_58bf_53df_3748,
        ),
        (
            "validation/generator/enclosure_boundary/FAMILY.json",
            0x5509_8fd7_8f3f_d199,
        ),
        (
            "validation/generator/enclosure_boundary/author.py",
            0xaed9_63f3_a218_aa17,
        ),
        (
            "validation/generator/enclosure_boundary/check.py",
            0x35a8_a1ae_e7d2_da04,
        ),
        (
            "validation/generator/enclosure_boundary/REUSED_SOURCE_ROWS.jsonl",
            0x25fd_3856_a717_9d85,
        ),
        (
            "validation/generator/enclosure_boundary/REUSED_ADMISSION_ROWS.jsonl",
            0x6027_c5b3_a480_434f,
        ),
        (
            "validation/enclosure/references/admission.jsonl.gz",
            0x71e3_8f70_491b_70a8,
        ),
        (
            "validation/enclosure/references/cash-hands.json.gz",
            0x1e3f_e9ff_e442_eb90,
        ),
        (
            "validation/enclosure/references/hand-certificates.json.gz",
            0xa5eb_fd8b_b600_dace,
        ),
        (
            "validation/enclosure/references/hands.json.gz",
            0xe9fe_c867_aadc_4007,
        ),
        (
            "validation/enclosure/references/hybrid-controls.jsonl.gz",
            0xcda1_4dd0_f6e3_75b4,
        ),
        (
            "validation/enclosure/references/price-factor-certificates.jsonl.gz",
            0x5b5c_4a0d_0836_a011,
        ),
        (
            "validation/enclosure/references/price-ln2-certificates.json.gz",
            0x88bf_a9af_3c30_af66,
        ),
        (
            "validation/enclosure/references/price-log-certificates.jsonl.gz",
            0xd403_cddb_7c50_efb4,
        ),
        (
            "validation/enclosure/references/price-target-certificates.jsonl.gz",
            0xd7e0_deb3_056f_a3f2,
        ),
        (
            "validation/enclosure/references/prices.jsonl.gz",
            0x420f_305c_2fb5_eb15,
        ),
        (
            "validation/enclosure/references/sources.jsonl.gz",
            0x605c_f1b0_19d3_5d04,
        ),
    ] {
        assert_eq!(
            fingerprint(&Fixture::repository_path(relative))?,
            pin,
            "{relative}"
        );
    }
    Ok(())
}
