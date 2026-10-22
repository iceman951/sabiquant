//! FR-UST-01..26: all 30,572 frozen source/date/cash composite operations.
//! All 28 output columns are inspected at the reached public stage. Of these,
//! 23 bind public outputs/context directly or format public inspectors; five
//! rational columns bind independently calculated public projections. Raw/selected
//! first coupons and instruction sources use unit and 1e-38 quantized views
//! (Exact and nearest-even); raw AI uses its correctly rounded binary64 cache,
//! and published AI also uses quantized views. No private rational equality is
//! claimed beyond those public projections. Unreached fields must remain `-`.
//! These are 30,572 rows, not 30,572 Rust test functions. All expectations were
//! frozen before Treasury production. Three Date causes are intentionally only
//! checked at the adopted outer wrapper/index/phase, with a typed source chain.
mod support;
#[path = "support/treasury.rs"]
mod treasury_support;
use sabiquant::amounts::{AmountError, AmountQuantum, AmountRounding, DecimalAmount};
use sabiquant::bonds::{
    CouponEntitlement, TreasuryCashDecision, TreasuryCashInstruction, TreasuryError,
    TreasuryPricingPlan, UsTreasury,
};
use sabiquant::error::Error as RootError;
use sabiquant::settlement::{CashAggregation, SettlementGuarantee, SettlementRounding};
use sabiquant::time::DateError;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::hash::{DefaultHasher, Hash, Hasher};
use support::fixture::{Fixture, Record};
use treasury_support::*;
const PATH: &str = "validation/fixtures/bonds/treasury-source-cash-oracle.fixture";
const SHA: &str = "2347711a897cbf627da1afae1ea17881aea35a6b1609b7ecd3fc8e8f538c5d31";
const OUTPUTS: [&str; 28] = [
    "expected",
    "context",
    "first_period",
    "first_quasi",
    "coupon_count",
    "raw_first",
    "source_first",
    "next_quasi",
    "r",
    "s",
    "n",
    "bundles",
    "entitlements",
    "raw_ai",
    "raw_ai_bits",
    "pub_ai",
    "pub_ai_bits",
    "pieces",
    "recipe_id",
    "src_index",
    "src_date",
    "settle_pair",
    "day_ratios",
    "dated_pair",
    "source_value",
    "lower",
    "upper",
    "guarantee",
];
#[derive(Clone, Copy)]
struct Row<'a>(Record<'a>);
impl<'a> Row<'a> {
    fn get(self, k: &str) -> TestResult<&'a str> {
        Ok(self.0.text(k)?)
    }
    fn id(self) -> &'a str {
        self.0.id()
    }
    fn number<T: std::str::FromStr>(self, k: &str) -> TestResult<T>
    where
        T::Err: Error + 'static,
    {
        Ok(self.get(k)?.parse()?)
    }
}
fn load() -> TestResult<Fixture> {
    let f = pinned(PATH, SHA, 30_572, 53)?;
    assert_eq!(
        f.header("fixture-set"),
        Some("US-TREASURY-SOURCE-DATE-CASH-ORACLE")
    );
    assert_eq!(f.header("source-version"), Some(SOURCE));
    assert_eq!(
        f.header("generator-sha256"),
        Some("046214dc324e0fab1dd5f3387de14b44b61c39547c6e2fcf4f5522896338782e")
    );
    assert_eq!(
        f.header("test-design-sha256"),
        Some("5e91934b538d03297659674cc9f10f51f91b1d35dd21437472a7901d157e0629")
    );
    assert_eq!(
        sha256(include_bytes!("../docs/M8_US_TREASURY_TEST_DESIGN.md"))?,
        f.header("test-design-sha256").ok_or("test design")?
    );
    // The parent primary-source bytes are an external provenance pin; the
    // local source checker is also fingerprinted against its adopted header.
    assert_eq!(
        f.header("parent-source-xml-sha256"),
        Some("35fa6cb3febd8807c06b08e73d1f311a472b31f127796c6ef369237915b6699d")
    );
    assert_eq!(
        f.header("parent-source-checker-sha256"),
        Some("04f268d313b82d1c77421903adc855ebb4ada5f3c25beeaf6db22dd4ef718e8d")
    );
    assert_eq!(
        sha256(include_bytes!(
            "../validation/generator/treasury_source_example_check.py"
        ))?,
        f.header("parent-source-checker-sha256")
            .ok_or("source checker pin")?
    );
    assert_eq!(
        f.header("contract-commit"),
        Some("a1b66f9f90459e2b5b47013149f51251adb2d359")
    );
    Ok(f)
}
fn hash<T: Hash>(a: &T) -> u64 {
    let mut h = DefaultHasher::new();
    a.hash(&mut h);
    h.finish()
}
fn render(error: &TreasuryError) -> TestResult<String> {
    let mut pairs: Vec<(&str, String)> = match error {
        TreasuryError::InvalidDateCap {
            cap,
            minimum,
            maximum,
            ..
        } => vec![
            ("cap", cap.to_string()),
            ("minimum", minimum.to_string()),
            ("maximum", maximum.to_string()),
        ],
        TreasuryError::TooManyQuasiDates { count, cap, .. } => {
            vec![("count", count.to_string()), ("cap", cap.to_string())]
        }
        TreasuryError::TooFewQuasiDates { count, minimum, .. } => {
            vec![
                ("count", count.to_string()),
                ("minimum", minimum.to_string()),
            ]
        }
        TreasuryError::NegativeCoupon { coupon_per_100, .. } => {
            vec![("coupon_per_100", cell(*coupon_per_100))]
        }
        TreasuryError::DatedAfterIssue {
            original_dated_date,
            original_issue_date,
            ..
        } => vec![
            ("original_dated_date", original_dated_date.to_string()),
            ("original_issue_date", original_issue_date.to_string()),
        ],
        TreasuryError::IssueNotBeforeMaturity {
            original_issue_date,
            maturity_date,
            ..
        } => vec![
            ("original_issue_date", original_issue_date.to_string()),
            ("maturity_date", maturity_date.to_string()),
        ],
        TreasuryError::MaturityBoundary {
            maturity_date,
            last_quasi_date,
            ..
        } => vec![
            ("maturity_date", maturity_date.to_string()),
            ("last_quasi_date", last_quasi_date.to_string()),
        ],
        TreasuryError::QuasiDateOrder {
            index,
            previous,
            date,
            ..
        } => vec![
            ("index", index.to_string()),
            ("previous", previous.to_string()),
            ("date", date.to_string()),
        ],
        TreasuryError::QuasiCoverage {
            original_dated_date,
            first_quasi_date,
            next_quasi_date,
            ..
        } => vec![
            ("original_dated_date", original_dated_date.to_string()),
            ("first_quasi_date", first_quasi_date.to_string()),
            ("next_quasi_date", next_quasi_date.to_string()),
        ],
        TreasuryError::QuasiDateMismatch {
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
        TreasuryError::Date { index, phase, .. } => vec![
            ("cause", "DateError".to_owned()),
            (
                "index",
                index.map_or_else(|| "-".to_owned(), |i| i.to_string()),
            ),
            ("phase", hex(phase)),
        ],
        TreasuryError::FirstCoupon {
            original_dated_date,
            original_issue_date,
            first_coupon_date,
            next_quasi_date,
            second_quasi_date,
            ..
        } => vec![
            ("original_issue_date", original_issue_date.to_string()),
            ("original_dated_date", original_dated_date.to_string()),
            ("first_coupon_date", first_coupon_date.to_string()),
            ("next_quasi_date", next_quasi_date.to_string()),
            (
                "second_quasi_date",
                second_quasi_date.map_or_else(|| "none".to_owned(), |date| date.to_string()),
            ),
        ],
        TreasuryError::InvalidBundleCap { cap, maximum, .. } => {
            vec![("cap", cap.to_string()), ("maximum", maximum.to_string())]
        }
        TreasuryError::TooManyBundles { count, cap, .. } => {
            vec![("count", count.to_string()), ("cap", cap.to_string())]
        }
        TreasuryError::SettlementBeforeIssue {
            settlement_date,
            original_issue_date,
            ..
        } => vec![
            ("settlement_date", settlement_date.to_string()),
            ("original_issue_date", original_issue_date.to_string()),
        ],
        TreasuryError::SettlementAtOrAfterMaturity {
            settlement_date,
            maturity_date,
            ..
        } => vec![
            ("settlement_date", settlement_date.to_string()),
            ("maturity_date", maturity_date.to_string()),
        ],
        TreasuryError::InvalidCouponIndex { index, count, .. } => {
            vec![("index", index.to_string()), ("count", count.to_string())]
        }
        TreasuryError::NonPositiveNominal { nominal, .. } => vec![("nominal", cell(*nominal))],
        TreasuryError::CashArithmetic {
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
        other => {
            return Err(format!("unexpected Treasury error in corpus scope: {other:?}").into());
        }
    };
    pairs.sort_unstable();
    Ok(pairs
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(","))
}

fn check_error(r: Row<'_>, e: &TreasuryError, a: &mut BTreeMap<&str, String>) -> TestResult {
    a.insert("expected", e.kind().to_owned());
    a.insert("context", render(e)?);
    assert!(!e.to_string().is_empty(), "{} error Display", r.id());
    match e {
        TreasuryError::CashArithmetic { cause, .. } => assert_eq!(
            e.source().and_then(|s| s.downcast_ref::<AmountError>()),
            Some(cause)
        ),
        TreasuryError::Date { cause, .. } => assert_eq!(
            e.source().and_then(|s| s.downcast_ref::<DateError>()),
            Some(cause)
        ),
        _ => assert!(e.source().is_none()),
    };
    let root = RootError::from(e.clone());
    assert!(matches!(&root,RootError::Treasury(inner)if inner==e));
    assert_eq!(root.kind(), e.kind());
    assert_eq!(
        root.source()
            .and_then(|s| s.downcast_ref::<TreasuryError>()),
        Some(e)
    );
    Ok(())
}
fn projection(
    id: &str,
    rat: &str,
    stage: &str,
    mut view: impl FnMut(AmountQuantum, AmountRounding) -> Result<Option<DecimalAmount>, TreasuryError>,
) -> TestResult {
    let x = Rat::parse(rat)?;
    for scale in [0, 38] {
        for mode in [AmountRounding::Exact, AmountRounding::NearestTiesEven] {
            let expected = x.rounded(scale, mode)?;
            let actual = view(AmountQuantum::new(1, scale)?, mode);
            match (expected, actual) {
                (Ok(v), Ok(Some(a))) => {
                    assert_eq!(a, v, "{id} {stage} {rat} scale{scale} {mode:?}")
                }
                (
                    Err((kind, negative)),
                    Err(TreasuryError::CashArithmetic {
                        stage: s,
                        endpoint: "point",
                        cause,
                        ..
                    }),
                ) => {
                    assert_eq!(s, stage, "{id}");
                    assert_eq!(cause.kind(), kind, "{id}");
                    match cause {
                        AmountError::Inexact { operation, .. } => assert_eq!(operation, "quantize"),
                        AmountError::OutOfRange {
                            operation,
                            negative: n,
                            ..
                        } => {
                            assert_eq!(operation, "quantize");
                            assert_eq!(n, negative)
                        }
                        other => return Err(format!("{id} {other:?}").into()),
                    }
                }
                (e, a) => {
                    return Err(
                        format!("{id} {stage} rational{rat} expected{e:?} actual{a:?}").into(),
                    );
                }
            }
        }
    }
    Ok(())
}
fn option<T: ToString>(a: Option<T>) -> String {
    a.map_or_else(|| "none".to_owned(), |v| v.to_string())
}
fn pair(a: Option<(sabiquant::time::Date, sabiquant::time::Date)>) -> String {
    a.map_or_else(|| "none".to_owned(), |(a, b)| format!("{a},{b}"))
}
type DayRatios = Option<((u32, u32), Option<(u32, u32)>)>;
fn ratios(a: DayRatios, trailing: bool) -> String {
    a.map_or_else(
        || "none".to_owned(),
        |((t, s), b)| match b {
            Some((u, v)) => format!("{t}/{s},{u}/{v}"),
            None => {
                if trailing {
                    format!("{t}/{s},none")
                } else {
                    format!("{t}/{s}")
                }
            }
        },
    )
}
fn build(r: Row<'_>) -> TestResult<Result<UsTreasury, TreasuryError>> {
    Ok(UsTreasury::new(
        date(r.get("dated")?)?,
        date(r.get("issue")?)?,
        date(r.get("maturity")?)?,
        amount(r.get("coupon")?)?,
        date(r.get("first")?)?,
        dates(r.get("quasi")?)?,
        r.get("eom")?.parse()?,
        policy(r.get("basis")?, r.get("tie")?)?,
        r.number("date_cap")?,
    ))
}
fn instruction(
    r: Row<'_>,
    t: &UsTreasury,
    p: Option<&TreasuryPricingPlan<'_>>,
) -> TestResult<Result<TreasuryCashInstruction, TreasuryError>> {
    let n = amount(r.get("nominal")?)?;
    let round = SettlementRounding::new(quantum(r.get("quantum")?)?, r.get("mode")?.parse()?);
    let prepared = || p.ok_or("recipe without plan");
    Ok(match r.get("recipe")? {
        "coupon" => t.coupon_cash(r.number("coupon_index")?, n, round),
        "accrued" => prepared()?.accrued_cash(n, round),
        "invoice" => prepared()?.clean_accrued_cash(
            n,
            evidence(r.get("evidence")?)?,
            r.get("aggregation")?.parse()?,
            round,
        ),
        "dirty" => prepared()?.dirty_cash(n, evidence(r.get("evidence")?)?, round),
        _ => return Err("recipe".into()),
    })
}
fn complete(r: Row<'_>, a: &BTreeMap<&str, String>) -> TestResult {
    assert_eq!(a.len(), 28);
    for key in OUTPUTS {
        assert_eq!(a[key], r.get(key)?, "{} output {key}", r.id());
    }
    Ok(())
}
/// Full physical output inspection for every composite operation, preserving
/// constructor -> plan -> factory -> view/evaluate precedence.
#[test]
fn all_source_cash_rows_public_outputs_and_provenance() -> TestResult {
    let f = load()?;
    let mut identities = HashMap::<TreasuryCashInstruction, String>::new();
    let mut definitions = BTreeMap::<String, TreasuryCashInstruction>::new();
    let mut seen = 0;
    for record in f.records() {
        let r = Row(record);
        let id = r.id();
        seen += 1;
        let mut a: BTreeMap<&str, String> =
            OUTPUTS.map(|k| (k, "-".to_owned())).into_iter().collect();
        a.insert("expected", "ok".to_owned());
        a.insert("context", "-".to_owned());
        let t = match build(r)? {
            Ok(t) => t,
            Err(e) => {
                check_error(r, &e, &mut a)?;
                complete(r, &a)?;
                continue;
            }
        };
        assert_eq!(t.source_version(), SOURCE, "{id}");
        assert_eq!(t.original_dated_date(), date(r.get("dated")?)?);
        assert_eq!(t.original_issue_date(), date(r.get("issue")?)?);
        assert_eq!(t.maturity_date(), date(r.get("maturity")?)?);
        assert_eq!(t.coupon_per_100(), amount(r.get("coupon")?)?);
        assert_eq!(t.first_coupon_date(), date(r.get("first")?)?);
        assert_eq!(t.quasi_dates(), dates(r.get("quasi")?)?);
        assert_eq!(t.end_of_month().to_string(), r.get("eom")?);
        assert_eq!(t.source_policy(), policy(r.get("basis")?, r.get("tie")?)?);
        let again = build(r)?.map_err(|e| format!("{id} duplicate {e:?}"))?;
        assert_eq!(t, again);
        assert_eq!(hash(&t), hash(&again));
        a.insert("first_period", format!("{:?}", t.first_period()));
        a.insert("first_quasi", t.first_coupon_quasi_index().to_string());
        a.insert("coupon_count", t.coupon_count().to_string());
        assert_eq!(
            t.coupon_dates(),
            &t.quasi_dates()[t.first_coupon_quasi_index()..]
        );
        projection(id, r.get("raw_first")?, "raw coupon", |q, m| {
            t.raw_coupon_per_100(0, q, m).map(Some)
        })?;
        a.insert("raw_first", r.get("raw_first")?.to_owned());
        projection(id, r.get("source_first")?, "source coupon", |q, m| {
            t.source_coupon_per_100(0, q, m).map(Some)
        })?;
        a.insert("source_first", r.get("source_first")?.to_owned());
        let p = if r.get("settlement")? == "-" {
            None
        } else {
            match TreasuryPricingPlan::new(&t, date(r.get("settlement")?)?, r.number("bundle_cap")?)
            {
                Ok(p) => Some(p),
                Err(e) => {
                    check_error(r, &e, &mut a)?;
                    complete(r, &a)?;
                    continue;
                }
            }
        };
        if let Some(p) = &p {
            assert!(std::ptr::eq(p.treasury(), &t));
            assert_eq!(p.source_version(), SOURCE);
            assert_eq!(p.source_policy(), t.source_policy());
            assert_eq!(p.settlement_date(), date(r.get("settlement")?)?);
            assert_eq!(p.clone(), *p);
            for (k, v) in [
                ("next_quasi", p.next_quasi_index().to_string()),
                ("r", p.remaining_days().to_string()),
                ("s", p.quasi_period_days().to_string()),
                ("n", p.remaining_quasi_periods().to_string()),
                (
                    "raw_ai_bits",
                    format!("{:016x}", p.raw_accrued_per_100().to_bits()),
                ),
                (
                    "pub_ai_bits",
                    format!("{:016x}", p.accrued_per_100().to_bits()),
                ),
            ] {
                a.insert(k, v);
            }
            for (k, v) in [
                ("raw_ai", p.raw_accrued_per_100()),
                ("pub_ai", p.accrued_per_100()),
            ] {
                assert_eq!(
                    Rat::parse(r.get(k)?)?.binary_round()?.to_bits(),
                    v.to_bits(),
                    "{id} exact rational {k}"
                );
                a.insert(k, r.get(k)?.to_owned());
            }
            let bundles = (0..p.bundle_count())
                .map(|i| p.bundle_quasi_index(i).ok_or("bundle"))
                .collect::<Result<Vec<_>, _>>()?;
            let first = *bundles.first().ok_or("empty bundles")?;
            let last = *bundles.last().ok_or("empty bundles")?;
            assert_eq!(bundles, (first..=last).collect::<Vec<_>>());
            a.insert("bundles", format!("{first}..{last}"));
            assert_eq!(p.bundle_quasi_index(p.bundle_count()), None);
            let mut ceased = 0;
            for i in 0..t.coupon_count() {
                let expected = if t.coupon_dates()[i] <= p.settlement_date() {
                    ceased += 1;
                    CouponEntitlement::Ceased
                } else {
                    CouponEntitlement::BuyerEntitled
                };
                assert_eq!(
                    p.coupon_entitlement(i),
                    Some(expected),
                    "{id} entitlement{i}"
                );
            }
            assert_eq!(p.coupon_entitlement(t.coupon_count()), None);
            a.insert("entitlements", format!("{ceased}/{}", t.coupon_count()));
            let ai = p.accrued_cash(
                DecimalAmount::new(1, 0)?,
                SettlementRounding::new(AmountQuantum::new(1, 0)?, AmountRounding::Exact),
            )?;
            a.insert("pieces", ratios(ai.source_day_ratios(), false));
            projection(id, r.get("pub_ai")?, "source accrued", |q, m| {
                ai.source_value_per_100(q, m)
            })?;
        }
        match r.get("op")? {
            "instrument" | "plan" => {}
            "raw_coupon_view" | "source_coupon_view" => {
                let q = quantum(r.get("view_quantum")?)?;
                let m = r.get("view_mode")?.parse()?;
                let result = if r.get("op")? == "raw_coupon_view" {
                    t.raw_coupon_per_100(r.number("coupon_index")?, q, m)
                } else {
                    t.source_coupon_per_100(r.number("coupon_index")?, q, m)
                };
                match result {
                    Ok(v) => {
                        a.insert("expected", "some".to_owned());
                        a.insert("lower", cell(v));
                        a.insert("upper", cell(v));
                    }
                    Err(e) => check_error(r, &e, &mut a)?,
                }
            }
            "cash" | "instruction_view" => {
                let i = match instruction(r, &t, p.as_ref())? {
                    Ok(i) => i,
                    Err(e) => {
                        check_error(r, &e, &mut a)?;
                        complete(r, &a)?;
                        continue;
                    }
                };
                assert_eq!(i.source_version(), SOURCE);
                assert_eq!(i.source_policy(), t.source_policy());
                assert_eq!(i.nominal(), amount(r.get("nominal")?)?);
                assert_eq!(
                    i.cash_rounding(),
                    SettlementRounding::new(quantum(r.get("quantum")?)?, r.get("mode")?.parse()?)
                );
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
                assert_eq!(i.coupon_per_100(), t.coupon_per_100());
                assert_eq!(i.original_dated_date(), t.original_dated_date());
                assert_eq!(i.original_issue_date(), t.original_issue_date());
                assert_eq!(i.first_coupon_date(), t.first_coupon_date());
                assert_eq!(i.maturity_date(), t.maturity_date());
                assert_eq!(i.first_period(), t.first_period());
                assert_eq!(i.end_of_month(), t.end_of_month());
                assert_eq!(
                    i.dated_quasi_dates(),
                    (t.quasi_dates()[0], t.quasi_dates()[1])
                );
                assert_eq!(
                    i.settlement_date(),
                    if r.get("recipe")? == "coupon" {
                        None
                    } else {
                        p.as_ref().map(TreasuryPricingPlan::settlement_date)
                    }
                );
                a.insert("recipe_id", i.recipe_identifier().to_owned());
                a.insert("src_index", option(i.source_coupon_index()));
                a.insert("src_date", option(i.source_coupon_date()));
                a.insert("settle_pair", pair(i.settlement_quasi_dates()));
                a.insert("day_ratios", ratios(i.source_day_ratios(), true));
                a.insert("dated_pair", pair(Some(i.dated_quasi_dates())));
                if r.get("source_value")? == "none" {
                    assert_eq!(
                        i.source_value_per_100(AmountQuantum::new(1, 0)?, AmountRounding::Exact)?,
                        None
                    );
                    a.insert("source_value", "none".to_owned());
                } else {
                    projection(
                        id,
                        r.get("source_value")?,
                        if r.get("recipe")? == "coupon" {
                            "source coupon"
                        } else {
                            "source accrued"
                        },
                        |q, m| i.source_value_per_100(q, m),
                    )?;
                    a.insert("source_value", r.get("source_value")?.to_owned());
                }
                let key = format!(
                    "{:?}",
                    (
                        i.recipe_identifier(),
                        i.source_policy(),
                        i.nominal(),
                        i.cash_rounding(),
                        i.price_evidence(),
                        i.aggregation(),
                        i.coupon_per_100(),
                        i.original_dated_date(),
                        i.original_issue_date(),
                        i.first_coupon_date(),
                        i.maturity_date()
                    )
                );
                let key = format!(
                    "{key}{:?}",
                    (
                        i.first_period(),
                        i.end_of_month(),
                        i.dated_quasi_dates(),
                        i.settlement_date(),
                        i.settlement_quasi_dates(),
                        i.source_coupon_index(),
                        i.source_coupon_date(),
                        i.source_day_ratios()
                    )
                );
                if let Some(old) = definitions.insert(key.clone(), i) {
                    assert_eq!(i, old, "{id} complete compact definition identity");
                    assert_eq!(hash(&i), hash(&old));
                }
                if let Some(old) = identities.insert(i, key.clone()) {
                    assert_eq!(old, key, "{id} distinct provenance collapsed");
                }
                assert_eq!(instruction(r, &t, p.as_ref())?, Ok(i));
                if r.get("op")? == "instruction_view" {
                    match i.source_value_per_100(
                        quantum(r.get("view_quantum")?)?,
                        r.get("view_mode")?.parse()?,
                    ) {
                        Ok(Some(v)) => {
                            a.insert("expected", "some".to_owned());
                            a.insert("lower", cell(v));
                            a.insert("upper", cell(v));
                        }
                        Ok(None) => {
                            a.insert("expected", "none".to_owned());
                        }
                        Err(e) => check_error(r, &e, &mut a)?,
                    }
                } else {
                    let decision = i.evaluate();
                    assert_eq!(decision, i.evaluate(), "{id} repeat");
                    match decision {
                        Ok(TreasuryCashDecision::Determined(receipt)) => {
                            assert_eq!(*receipt.instruction(), i);
                            let g = receipt.price_evidence_guarantee();
                            let expected = match i.price_evidence() {
                                None => None,
                                Some(_) => Some(match r.get("evidence")?.split(':').next() {
                                    Some("b") => SettlementGuarantee::ForSuppliedBinary64,
                                    Some("di" | "bi") => {
                                        SettlementGuarantee::UniqueForDeclaredEnclosure
                                    }
                                    _ => SettlementGuarantee::ForSuppliedDecimals,
                                }),
                            };
                            assert_eq!(g, expected);
                            a.insert("expected", "determined".to_owned());
                            a.insert("lower", cell(receipt.amount()));
                            a.insert("upper", cell(receipt.amount()));
                            a.insert(
                                "guarantee",
                                g.map_or_else(|| "none".to_owned(), |g| format!("{g:?}")),
                            );
                        }
                        Ok(TreasuryCashDecision::QuantumAmbiguous {
                            lower_amount,
                            upper_amount,
                            ..
                        }) => {
                            a.insert("expected", "ambiguous".to_owned());
                            a.insert("lower", cell(lower_amount));
                            a.insert("upper", cell(upper_amount));
                        }
                        Ok(TreasuryCashDecision::ExactStageUnresolved { stage, .. }) => {
                            a.insert("expected", "exact_unresolved".to_owned());
                            a.insert("context", format!("stage={}", hex(stage)));
                        }
                        Err(e) => check_error(r, &e, &mut a)?,
                        Ok(other) => return Err(format!("{id} unknown decision {other:?}").into()),
                    }
                }
            }
            other => return Err(format!("{id} op{other}").into()),
        };
        complete(r, &a)?;
    }
    assert_eq!(seen, 30_572);
    assert_eq!(identities.len(), definitions.len());
    assert!(identities.len() > 10_000);
    Ok(())
}

/// FR-UST-02..26: literal schema, all row-derivable censuses and complete
/// serialized/canonical input identity. The 66 encoding aliases deliberately
/// represent 30,506 distinct semantic tuples, rather than 30,572 operations.
#[test]
fn source_cash_schema_censuses_and_complete_input_identity() -> TestResult {
    let f = load()?;
    let inputs = [
        "op",
        "dated",
        "issue",
        "maturity",
        "coupon",
        "first",
        "quasi",
        "eom",
        "date_cap",
        "basis",
        "tie",
        "settlement",
        "bundle_cap",
        "coupon_index",
        "recipe",
        "nominal",
        "evidence",
        "aggregation",
        "quantum",
        "mode",
        "view_quantum",
        "view_mode",
    ];
    let columns = [
        vec!["id", "profile"],
        inputs.to_vec(),
        OUTPUTS.to_vec(),
        vec!["identity_group"],
    ]
    .concat();
    assert_eq!(
        f.columns(),
        &columns.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>()
    );
    assert_eq!(f.header("input-columns"), Some(inputs.join(",").as_str()));
    assert_eq!(f.header("output-columns"), Some(OUTPUTS.join(",").as_str()));
    let mut counts = BTreeMap::<&str, BTreeMap<String, usize>>::new();
    let mut semantic = BTreeSet::new();
    let mut scales = BTreeMap::<&str, BTreeSet<u8>>::new();
    for (position, record) in f.records().enumerate() {
        let r = Row(record);
        assert_eq!(r.id(), format!("TSC-{:06}", position + 1));
        let mut add =
            |header: &'static str, value: &str| bump(counts.entry(header).or_default(), value);
        for (header, column) in [
            ("operation-counts", "op"),
            ("expectation-counts", "expected"),
            ("eom-counts", "eom"),
            ("first-period-counts", "first_period"),
            ("recipe-counts", "recipe"),
            ("aggregation-counts", "aggregation"),
            ("guarantee-counts", "guarantee"),
        ] {
            if r.get(column)? != "-" {
                add(header, r.get(column)?);
            }
        }
        add(
            "policy-counts",
            &format!("{}/{}", r.get("basis")?, r.get("tie")?),
        );
        if r.get("recipe")? != "-" {
            add("cash-mode-counts", r.get("mode")?);
        }
        if ["raw_coupon_view", "source_coupon_view", "instruction_view"].contains(&r.get("op")?) {
            add("view-mode-counts", r.get("view_mode")?);
        }
        if r.get("pieces")? != "-" {
            add(
                "ai-piece-counts",
                if r.get("pieces")?.contains(',') {
                    "2-piece"
                } else {
                    "1-piece"
                },
            );
        }
        if r.get("evidence")? != "-" {
            add(
                "evidence-kind-counts",
                r.get("evidence")?
                    .split(':')
                    .next()
                    .ok_or("evidence kind")?,
            );
        }
        let c = context(r.get("context")?)?;
        if let Some(stage) = c.get("stage") {
            let text = stage.strip_prefix("x:").ok_or("stage hex")?;
            let bytes = text
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| {
                    u8::from_str_radix(std::str::from_utf8(b)?, 16)
                        .map_err(|e| Box::new(e) as Box<dyn Error>)
                })
                .collect::<TestResult<Vec<_>>>()?;
            add("stage-counts", &String::from_utf8(bytes)?);
        }
        for (header, key) in [
            ("endpoint-counts", "endpoint"),
            ("cause-counts", "cause"),
            ("operation-label-counts", "operation"),
        ] {
            if r.get("expected")? == "treasury.cash_arithmetic" {
                add(header, c.get(key).ok_or("cash context key")?);
            }
        }
        for (key, column) in [
            ("coupon", "coupon"),
            ("nominal", "nominal"),
            ("quantum", "quantum"),
            ("view-quantum", "view_quantum"),
        ] {
            if r.get(column)? != "-" {
                scales
                    .entry(key)
                    .or_default()
                    .insert(r.get(column)?.split_once(':').ok_or("scale")?.1.parse()?);
            }
        }
        let evidence_parts = r.get("evidence")?.split(':').collect::<Vec<_>>();
        if matches!(evidence_parts.first(), Some(&"d" | &"di")) {
            for i in (2..evidence_parts.len()).step_by(2) {
                scales
                    .entry("decimal-evidence")
                    .or_default()
                    .insert(evidence_parts[i].parse()?);
            }
        }
        let mut normalized = Vec::new();
        for column in inputs {
            let text = r.get(column)?;
            normalized.push(
                if ["coupon", "nominal", "quantum", "view_quantum"].contains(&column) && text != "-"
                {
                    cell(amount(text)?)
                } else if column == "evidence" && text != "-" {
                    format!("{:?}", evidence(text)?)
                } else {
                    text.to_owned()
                },
            );
        }
        semantic.insert(normalized);
    }
    for (header, count) in &counts {
        assert_eq!(
            f.header(header),
            Some(count_json(count).as_str()),
            "{header}"
        );
    }
    assert_eq!(counts.len(), 16);
    assert_eq!(semantic.len(), 30_506);
    assert_eq!(30_572 - semantic.len(), 66);
    assert_eq!(scales.len(), 5);
    for (name, values) in scales {
        assert_eq!(values, (0..=38).collect(), "all 39 scales {name}");
    }
    Ok(())
}

/// Known SHA and exact signed quantization/IEEE anchors validate test arithmetic.
#[test]
fn independent_projection_helpers_known_vectors() -> TestResult {
    assert_eq!(
        sha256(b"")?,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256(b"abc")?,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    for (ratio, value) in [
        ("0/1", 0.0_f64),
        ("1/10", 0.1),
        ("-1/10", -0.1),
        ("27/16", 1.6875),
        ("5/362", 5.0 / 362.0),
    ] {
        assert_eq!(
            Rat::parse(ratio)?.binary_round()?.to_bits(),
            value.to_bits()
        );
    }
    let x = Rat::parse("-5/2")?;
    assert_eq!(
        x.rounded(0, AmountRounding::NearestTiesEven)?,
        Ok(amount("-2:0")?)
    );
    assert_eq!(
        x.rounded(0, AmountRounding::NearestTiesAwayFromZero)?,
        Ok(amount("-3:0")?)
    );
    assert_eq!(
        x.rounded(0, AmountRounding::Exact)?,
        Err(("amount.inexact", true))
    );
    assert_eq!(
        Rat::parse("2/1")?.rounded(38, AmountRounding::Exact)?,
        Ok(amount("2:0")?)
    );
    assert_eq!(
        Rat::parse("3/128")?.rounded(6, AmountRounding::NearestTiesEven)?,
        Ok(amount("23438:6")?)
    );
    // Independent Fraction-to-binary64 boundary vectors include subnormal
    // ties, normal-binade carries and the signed finite/infinity threshold.
    for (ratio, expected_bits) in [
        (
            "-1/404804506614621236704990693437834614099113299528284236713802716054860679135990693783920767402874248990374155728633623822779617474771586953734026799881477019843034848553132722728933815484186432682479535356945490137124014966849385397236206711298319112681620113024717539104666829230461005064372655017292012526615415482186989568",
            0x8000000000000000_u64,
        ),
        (
            "-3/809609013229242473409981386875669228198226599056568473427605432109721358271981387567841534805748497980748311457267247645559234949543173907468053599762954039686069697106265445457867630968372865364959070713890980274248029933698770794472413422596638225363240226049435078209333658460922010128745310034584025053230830964373979136",
            0x8000000000000001_u64,
        ),
        (
            "-1/202402253307310618352495346718917307049556649764142118356901358027430339567995346891960383701437124495187077864316811911389808737385793476867013399940738509921517424276566361364466907742093216341239767678472745068562007483424692698618103355649159556340810056512358769552333414615230502532186327508646006263307707741093494784",
            0x8000000000000001_u64,
        ),
        (
            "-3/404804506614621236704990693437834614099113299528284236713802716054860679135990693783920767402874248990374155728633623822779617474771586953734026799881477019843034848553132722728933815484186432682479535356945490137124014966849385397236206711298319112681620113024717539104666829230461005064372655017292012526615415482186989568",
            0x8000000000000002_u64,
        ),
        (
            "-5/404804506614621236704990693437834614099113299528284236713802716054860679135990693783920767402874248990374155728633623822779617474771586953734026799881477019843034848553132722728933815484186432682479535356945490137124014966849385397236206711298319112681620113024717539104666829230461005064372655017292012526615415482186989568",
            0x8000000000000002_u64,
        ),
        (
            "-9007199254740991/404804506614621236704990693437834614099113299528284236713802716054860679135990693783920767402874248990374155728633623822779617474771586953734026799881477019843034848553132722728933815484186432682479535356945490137124014966849385397236206711298319112681620113024717539104666829230461005064372655017292012526615415482186989568",
            0x8010000000000000_u64,
        ),
        ("-9007199254740993/9007199254740992", 0xbff0000000000000_u64),
        ("-9007199254740995/9007199254740992", 0xbff0000000000002_u64),
        (
            "-18014398509481983/9007199254740992",
            0xc000000000000000_u64,
        ),
        (
            "-179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368/1",
            0xffefffffffffffff_u64,
        ),
        (
            "-179769313486231575804128197568503885939002350117941411767545627891801114536396644853619288305177042633935372685103635187590438437370702292699562517687521668833979406288629832876259672468103520237920172119362601898937975098263032931492834697134299320496935997324255116936540444370309403987146642102044149678080/1",
            0xffefffffffffffff_u64,
        ),
        (
            "-179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497792/1",
            0xfff0000000000000_u64,
        ),
        (
            "1/404804506614621236704990693437834614099113299528284236713802716054860679135990693783920767402874248990374155728633623822779617474771586953734026799881477019843034848553132722728933815484186432682479535356945490137124014966849385397236206711298319112681620113024717539104666829230461005064372655017292012526615415482186989568",
            0x0000000000000000_u64,
        ),
        (
            "3/809609013229242473409981386875669228198226599056568473427605432109721358271981387567841534805748497980748311457267247645559234949543173907468053599762954039686069697106265445457867630968372865364959070713890980274248029933698770794472413422596638225363240226049435078209333658460922010128745310034584025053230830964373979136",
            0x0000000000000001_u64,
        ),
        (
            "1/202402253307310618352495346718917307049556649764142118356901358027430339567995346891960383701437124495187077864316811911389808737385793476867013399940738509921517424276566361364466907742093216341239767678472745068562007483424692698618103355649159556340810056512358769552333414615230502532186327508646006263307707741093494784",
            0x0000000000000001_u64,
        ),
        (
            "3/404804506614621236704990693437834614099113299528284236713802716054860679135990693783920767402874248990374155728633623822779617474771586953734026799881477019843034848553132722728933815484186432682479535356945490137124014966849385397236206711298319112681620113024717539104666829230461005064372655017292012526615415482186989568",
            0x0000000000000002_u64,
        ),
        (
            "5/404804506614621236704990693437834614099113299528284236713802716054860679135990693783920767402874248990374155728633623822779617474771586953734026799881477019843034848553132722728933815484186432682479535356945490137124014966849385397236206711298319112681620113024717539104666829230461005064372655017292012526615415482186989568",
            0x0000000000000002_u64,
        ),
        (
            "9007199254740991/404804506614621236704990693437834614099113299528284236713802716054860679135990693783920767402874248990374155728633623822779617474771586953734026799881477019843034848553132722728933815484186432682479535356945490137124014966849385397236206711298319112681620113024717539104666829230461005064372655017292012526615415482186989568",
            0x0010000000000000_u64,
        ),
        ("9007199254740993/9007199254740992", 0x3ff0000000000000_u64),
        ("9007199254740995/9007199254740992", 0x3ff0000000000002_u64),
        ("18014398509481983/9007199254740992", 0x4000000000000000_u64),
        (
            "179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368/1",
            0x7fefffffffffffff_u64,
        ),
        (
            "179769313486231575804128197568503885939002350117941411767545627891801114536396644853619288305177042633935372685103635187590438437370702292699562517687521668833979406288629832876259672468103520237920172119362601898937975098263032931492834697134299320496935997324255116936540444370309403987146642102044149678080/1",
            0x7fefffffffffffff_u64,
        ),
        (
            "179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497792/1",
            0x7ff0000000000000_u64,
        ),
    ] {
        assert_eq!(
            Rat::parse(ratio)?.binary_round()?.to_bits(),
            expected_bits,
            "{ratio}"
        );
    }
    // All seven signed quantizers, independently tabulated at +/-1.25.
    for (text, expected) in [
        ("5/4", ["1:0", "2:0", "2:0", "1:0", "1:0", "1:0"]),
        ("-5/4", ["-1:0", "-2:0", "-1:0", "-2:0", "-1:0", "-1:0"]),
    ] {
        for (mode, amount_text) in [
            AmountRounding::TowardZero,
            AmountRounding::AwayFromZero,
            AmountRounding::TowardPositiveInfinity,
            AmountRounding::TowardNegativeInfinity,
            AmountRounding::NearestTiesEven,
            AmountRounding::NearestTiesAwayFromZero,
        ]
        .into_iter()
        .zip(expected)
        {
            assert_eq!(
                Rat::parse(text)?.rounded(0, mode)?,
                Ok(amount(amount_text)?)
            );
        }
        assert_eq!(
            Rat::parse(text)?.rounded(0, AmountRounding::Exact)?,
            Err(("amount.inexact", text.starts_with('-')))
        );
    }
    for (text, negative) in [
        ("170141183460469231731687303715884105728/1", false),
        ("-170141183460469231731687303715884105729/1", true),
    ] {
        assert_eq!(
            Rat::parse(text)?.rounded(0, AmountRounding::Exact)?,
            Err(("amount.out_of_range", negative))
        );
    }
    for text in [
        "170141183460469231731687303715884105727",
        "-170141183460469231731687303715884105728",
    ] {
        assert_eq!(
            Rat::parse(text)?.rounded(38, AmountRounding::Exact)?,
            Ok(DecimalAmount::new(text.parse()?, 0)?)
        );
    }
    let wide = Rat::parse("31135836573265869406898776580006791348041/362")?;
    assert_eq!(
        wide.rounded(0, AmountRounding::NearestTiesEven)?,
        Ok(amount("86010598268690246980383360718250804829:0")?)
    );
    assert!(Rat::binary(1.0).sub(&Rat::parse("1E+0")?).is_zero());
    assert!(
        Rat::binary(0.1)
            .sub(&Rat::parse("1E-1")?)
            .abs_le(&Rat::parse("6E-18")?)
    );
    assert!(
        !Rat::binary(0.1)
            .sub(&Rat::parse("1E-1")?)
            .abs_le(&Rat::parse("5E-18")?)
    );
    Ok(())
}
