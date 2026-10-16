//! Independent oracle consumer for M8 bond core (`sabiquant::bonds`).
//!
//! FR-BOND-01..10: evaluates every row of
//! `validation/fixtures/bonds/bond-core-oracles.fixture` through the public
//! `sabiquant::bonds` API and checks quote conversions, coupon/redemption
//! entitlements, work-unit accounting, valuation prices, and structured errors.

mod support;

use sabiquant::bonds::{
    Bond, BondError, BondPrice, BondPriceKind, BondPricingPlan, BondSettlement, BondValuation,
    BondYieldConvention, CouponEntitlement, ExCouponBoundary, ExCouponTerms,
    amount_from_price_per_100, price_per_100_from_amount,
};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{Business252, DayCount, Thirty360Us};
use sabiquant::error::{NonFinite, ValueError};
use sabiquant::rates::{AccrualConvention, Compounding};
use sabiquant::schedule::{
    DateGenerationRule, ExplicitStubDates, ScheduleBuilder, ScheduleCalendar,
};
use sabiquant::time::{Date, Frequency, Period, TimeUnit};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const COLUMNS: [&str; 15] = [
    "id",
    "profile",
    "op",
    "face_bits",
    "value_bits",
    "spec",
    "settlement",
    "inclusion",
    "yield_dcc",
    "yield_comp",
    "yield_bits",
    "work_cap",
    "expected",
    "context",
    "output",
];

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn pinned_fixture() -> TestResult<Fixture> {
    #[expect(
        clippy::disallowed_methods,
        reason = "test fingerprints frozen fixture bytes"
    )]
    let text = std::fs::read_to_string(Fixture::repository_path(
        "validation/fixtures/bonds/bond-core-oracles.fixture",
    ))?;
    let (_, body) = text.split_once("\n\n").ok_or("fixture separator")?;
    let fixture = Fixture::parse(&text)?;
    assert_eq!(fixture.columns(), &COLUMNS.map(str::to_owned));
    assert_eq!(fnv64(body.as_bytes()), 0x2915_133b_0fe0_a519);
    assert_eq!(fixture.header("body-fnv64"), Some("2915133b0fe0a519"));
    assert_eq!(
        fixture.header("body-sha256"),
        Some("b7c7e1f8bf9de9d7b90c807a14b51131fd90d848de1396c0d87f2c024f2e6e09")
    );
    for (key, bytes, frozen) in [
        (
            "generator-fnv64",
            include_bytes!("../validation/generator/bond_core_oracles.py").as_slice(),
            0x3987_3da2_416b_acb9,
        ),
        (
            "adopted-contract-fnv64",
            include_bytes!("../docs/M8_BOND_CORE_CONTRACT.md").as_slice(),
            0xe86f_8537_32e7_dcc0,
        ),
    ] {
        assert_eq!(fnv64(bytes), frozen, "{key}");
        assert_eq!(fixture.header(key), Some(format!("{frozen:016x}").as_str()));
    }
    assert_eq!(
        fixture.header("generator-sha256"),
        Some("afe27c0ce1a362492a98202c2882271cabbc3b1d510bb49dd3d5215460055b18")
    );
    assert_eq!(
        fixture.header("adopted-contract-sha256"),
        Some("009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c")
    );
    let requirements = include_str!("../docs/REQUIREMENTS.md")
        .lines()
        .filter(|line| line.starts_with("| FR-BOND-"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(fnv64(requirements.as_bytes()), 0x0a48_e438_61fb_b02b);
    assert_eq!(
        fixture.header("requirements-rows-sha256"),
        Some("dbae973922e98619094814a7b0edf1a77d747d5cf7246254699063a7e996b8cb")
    );
    Ok(fixture)
}

enum Outcome {
    Bits(u64),
    Plan {
        ent_code: String,
        red_entitled: bool,
        entitled_flows: usize,
        work_used: u64,
        val: BondValuation,
        val_direct: BondValuation,
        plan_accrued_p100: u64,
        dirty_p100: u64,
        accrued_p100: u64,
        clean_p100: u64,
    },
    Failure(BondError),
}

fn parse_bits(cell: &str) -> TestResult<u64> {
    let digits = cell.strip_prefix("0x").ok_or("bits need 0x prefix")?;
    if digits.len() != 16 {
        return Err("bits need 16 hex digits".into());
    }
    Ok(u64::from_str_radix(digits, 16)?)
}

fn parse_f64_bits(cell: &str) -> TestResult<f64> {
    Ok(f64::from_bits(parse_bits(cell)?))
}

fn parse_accrual(name: &str) -> TestResult<AccrualConvention> {
    Ok(match name {
        "Act365F" => AccrualConvention::from(DayCount::Actual365Fixed),
        "Act360" => AccrualConvention::from(DayCount::Actual360),
        "30360BB" => AccrualConvention::from(DayCount::Thirty360BondBasis),
        "30E360" => AccrualConvention::from(DayCount::ThirtyE360),
        "30360US_EOM" => AccrualConvention::Thirty360Us(Thirty360Us::new(true)),
        "B252_WEEKENDS" => {
            AccrualConvention::Business252(Business252::new(Calendar::weekends_only()))
        }
        other => return Err(format!("unknown accrual convention {other}").into()),
    })
}

fn parse_compounding(text: &str) -> TestResult<Compounding> {
    if text == "Simple" {
        return Ok(Compounding::SIMPLE);
    }
    if text == "Continuous" {
        return Ok(Compounding::CONTINUOUS);
    }
    let (kind, freq_s) = text.split_once(':').ok_or("compounding missing ':'")?;
    let freq = match freq_s {
        "1" => Frequency::Annual,
        "2" => Frequency::Semiannual,
        "4" => Frequency::Quarterly,
        other => return Err(format!("unsupported frequency {other}").into()),
    };
    Ok(match kind {
        "Compounded" => Compounding::compounded(freq)?,
        "SimpleThenCompounded" => Compounding::simple_then_compounded(freq)?,
        "CompoundedThenSimple" => Compounding::compounded_then_simple(freq)?,
        other => return Err(format!("unknown compounding {other}").into()),
    })
}

fn parse_ex_terms(spec: &str) -> TestResult<Option<ExCouponTerms>> {
    if spec == "-" {
        return Ok(None);
    }
    let (boundary_s, date_s) = spec.split_once(':').ok_or("ex spec missing ':'")?;
    let boundary = match boundary_s {
        "OnOrAfter" => ExCouponBoundary::OnOrAfter,
        "After" => ExCouponBoundary::After,
        other => return Err(format!("unknown ex boundary {other}").into()),
    };
    let cutoff = date_s.parse::<Date>()?;
    Ok(Some(ExCouponTerms::new(cutoff, boundary)))
}

fn build_bond(face: f64, coupon_rate_val: f64, spec: &str) -> TestResult<Bond> {
    let parts: Vec<&str> = spec.split('|').collect();
    if parts.len() < 7 {
        return Err("malformed bond spec".into());
    }
    let kind = parts[0];
    let issue = parts[1].parse::<Date>()?;
    let maturity = parts[2].parse::<Date>()?;
    let red_pay = parts[3].parse::<Date>()?;
    let red_amt = parse_f64_bits(parts[4])?;
    let redemption = SimpleCashFlow::new(red_pay, red_amt)?;

    if kind == "ZeroCoupon" {
        return Ok(Bond::zero_coupon(issue, maturity, face, redemption)?);
    }
    if kind != "FixedRate" {
        return Err(format!("unknown bond kind {kind}").into());
    }

    let coupon_accrual = parse_accrual(parts[5])?;
    let coupon_rate = FixedCouponRate::new(coupon_rate_val, coupon_accrual)?;
    let coupon_items: Vec<&str> = parts[6].split(';').collect();
    if coupon_items.len() != 2 {
        return Err("expected 2-coupon schedule in oracle".into());
    }

    let mut starts = Vec::new();
    let mut ends = Vec::new();
    let mut pays = Vec::new();
    let mut ex_terms = Vec::new();
    for item in coupon_items {
        let fields: Vec<&str> = item.split('@').collect();
        if fields.len() != 4 {
            return Err("coupon item needs 4 fields".into());
        }
        starts.push(fields[0].parse::<Date>()?);
        ends.push(fields[1].parse::<Date>()?);
        pays.push(fields[2].parse::<Date>()?);
        ex_terms.push(parse_ex_terms(fields[3])?);
    }

    let s0 = starts[0];
    let e1 = ends[0];
    let e2 = ends[1];
    let p1 = pays[0];
    let p2 = pays[1];

    let mut calendar = Calendar::null();
    let conv1 = if p1 == e1 {
        BusinessDayConvention::Unadjusted
    } else if p1 < e1 {
        let mut cur = p1.checked_add_days(1)?;
        while cur <= e1 {
            calendar = calendar.with_holiday(cur);
            cur = cur.checked_add_days(1)?;
        }
        BusinessDayConvention::Preceding
    } else {
        let mut cur = e1;
        while cur < p1 {
            calendar = calendar.with_holiday(cur);
            cur = cur.checked_add_days(1)?;
        }
        BusinessDayConvention::Following
    };

    let conv2 = if p2 == e2 {
        BusinessDayConvention::Unadjusted
    } else if p2 < e2 {
        let mut cur = p2.checked_add_days(1)?;
        while cur <= e2 {
            calendar = calendar.with_holiday(cur);
            cur = cur.checked_add_days(1)?;
        }
        BusinessDayConvention::Preceding
    } else {
        let mut cur = e2;
        while cur < p2 {
            calendar = calendar.with_holiday(cur);
            cur = cur.checked_add_days(1)?;
        }
        BusinessDayConvention::Following
    };

    let builder = ScheduleBuilder::new(s0, e2, ScheduleCalendar::Plain(calendar), conv1, conv2)?;
    let explicit = ExplicitStubDates::new(Some(e1), Some(e1))?;
    let schedule = builder.build_explicit(
        Period::new(6, TimeUnit::Months),
        DateGenerationRule::Forward,
        explicit,
        EndOfMonth::Ignore,
        3,
    )?;
    assert_eq!(schedule.contractual_dates(), &[s0, e1, e2]);
    assert_eq!(schedule.adjusted_dates(), &[s0, p1, p2]);

    Ok(Bond::fixed_rate(
        issue,
        schedule,
        face,
        coupon_rate,
        ex_terms,
        redemption,
        Bond::MAX_FLOWS,
    )?)
}

fn evaluate(row: &Record<'_>) -> TestResult<Outcome> {
    let op = row.text("op")?;
    let face = parse_f64_bits(row.text("face_bits")?)?;
    let val = parse_f64_bits(row.text("value_bits")?)?;

    match op {
        "per_100_from_amount" => Ok(match price_per_100_from_amount(val, face) {
            Ok(p) => {
                let bp = BondPrice::from_amount(val, face, BondPriceKind::Clean)?;
                assert_eq!(bp.value_per_100_face().to_bits(), p.to_bits());
                assert_eq!(bp.kind(), BondPriceKind::Clean);
                Outcome::Bits(p.to_bits())
            }
            Err(e) => Outcome::Failure(e),
        }),
        "amount_from_per_100" => Ok(match amount_from_price_per_100(val, face) {
            Ok(a) => {
                if face > 0.0 && face.is_finite() && val.is_finite() {
                    let bp = BondPrice::new(val, BondPriceKind::Dirty)?;
                    assert_eq!(bp.to_amount(face)?.to_bits(), a.to_bits());
                }
                Outcome::Bits(a.to_bits())
            }
            Err(e) => Outcome::Failure(e),
        }),
        "bond_plan" => {
            let bond = build_bond(face, val, row.text("spec")?)?;
            let settle_date = row.text("settlement")?.parse::<Date>()?;
            let inc = match row.text("inclusion")? {
                "Include" => PaymentDateInclusion::Include,
                "Exclude" => PaymentDateInclusion::Exclude,
                other => return Err(format!("unknown inclusion {other}").into()),
            };
            let settlement = BondSettlement::new(settle_date, inc);
            let y_accrual = parse_accrual(row.text("yield_dcc")?)?;
            let y_comp = parse_compounding(row.text("yield_comp")?)?;
            let y_conv = BondYieldConvention::new(y_accrual, y_comp);
            let y_rate = parse_f64_bits(row.text("yield_bits")?)?;
            let work_cap: u64 = row.text("work_cap")?.parse()?;

            match BondPricingPlan::new(&bond, settlement, &y_conv, work_cap) {
                Ok(plan) => {
                    let v1 = plan.evaluate(y_rate)?;
                    let v2 = bond.evaluate(settlement, &y_conv, y_rate, work_cap)?;
                    let ent_code = if plan.coupon_entitlements().is_empty() {
                        "-".to_owned()
                    } else {
                        plan.coupon_entitlements()
                            .iter()
                            .map(|e| match e {
                                CouponEntitlement::Ceased => 'C',
                                CouponEntitlement::BuyerEntitled => 'B',
                                CouponEntitlement::ExCouponDetached => 'X',
                            })
                            .collect()
                    };
                    let plan_accrued_p100 = plan.accrued_per_100()?.to_bits();
                    let dirty_p100 = v1.dirty_price_per_100()?.to_bits();
                    let accrued_p100 = v1.accrued_per_100()?.to_bits();
                    let clean_p100 = v1.clean_price_per_100()?.to_bits();
                    Ok(Outcome::Plan {
                        ent_code,
                        red_entitled: plan.redemption_entitled(),
                        entitled_flows: plan.entitled_flow_count(),
                        work_used: plan.accrual_work_used(),
                        val: v1,
                        val_direct: v2,
                        plan_accrued_p100,
                        dirty_p100,
                        accrued_p100,
                        clean_p100,
                    })
                }
                Err(e) => {
                    let e2 = bond
                        .evaluate(settlement, &y_conv, y_rate, work_cap)
                        .err()
                        .ok_or("bond.evaluate must fail when BondPricingPlan::new fails")?;
                    assert_eq!(e, e2, "{}: plan vs evaluate error mismatch", row.id());
                    Ok(Outcome::Failure(e))
                }
            }
        }
        other => Err(format!("unknown op {other}").into()),
    }
}

fn ulp_distance(a: u64, b: u64) -> u64 {
    if a == b {
        return 0;
    }
    let fa = f64::from_bits(a);
    let fb = f64::from_bits(b);
    if !fa.is_finite() || !fb.is_finite() || (a >> 63) != (b >> 63) {
        return u64::MAX;
    }
    a.abs_diff(b)
}

fn nonfinite_name(nf: NonFinite) -> &'static str {
    match nf {
        NonFinite::NaN => "NaN",
        NonFinite::PositiveInfinity => "PositiveInfinity",
        NonFinite::NegativeInfinity => "NegativeInfinity",
    }
}

fn error_fields(error: &BondError) -> TestResult<BTreeMap<&'static str, String>> {
    let mut fields = BTreeMap::new();
    match error {
        BondError::Value {
            index,
            phase,
            cause,
            ..
        } => {
            assert_eq!(*index, None);
            fields.insert("phase", (*phase).to_owned());
            fields.insert("kind", cause.kind().to_owned());
            match cause {
                ValueError::NotFinite { value, .. } | ValueError::ResultNotFinite { value, .. } => {
                    fields.insert("class", nonfinite_name(*value).to_owned());
                }
                ValueError::OutOfDomain { .. } | ValueError::ResultOutOfDomain { .. } => {}
                other => return Err(format!("unexpected ValueError {other:?}").into()),
            }
        }
        BondError::SettlementBeforeIssue {
            settlement_date,
            issue_date,
            ..
        } => {
            fields.insert("settlement_date", settlement_date.to_string());
            fields.insert("issue_date", issue_date.to_string());
        }
        BondError::AccrualWorkLimitExceeded {
            index,
            phase,
            maximum,
            used,
            required_at_least,
            ..
        } => {
            let idx_str = index.map_or_else(|| "none".to_owned(), |i| i.to_string());
            fields.insert("index", idx_str);
            fields.insert("phase", (*phase).to_owned());
            fields.insert("maximum", maximum.to_string());
            fields.insert("used", used.to_string());
            fields.insert("required_at_least", required_at_least.to_string());
        }
        other => return Err(format!("unexpected BondError {other:?}").into()),
    }
    Ok(fields)
}

fn context_map(context: &str) -> TestResult<BTreeMap<&str, String>> {
    let mut map = BTreeMap::new();
    for item in context.split(',') {
        let (k, v) = item.split_once('=').ok_or("context needs key=value")?;
        if map.insert(k, v.to_owned()).is_some() {
            return Err(format!("duplicate context key {k}").into());
        }
    }
    Ok(map)
}

fn check(row: &Record<'_>, outcome: &Outcome) -> TestResult {
    let id = row.id();
    let expected = row.text("expected")?;
    let context = row.text("context")?;
    let output = row.text("output")?;

    match outcome {
        Outcome::Bits(bits) => {
            assert_eq!((expected, context), ("ok", "-"), "{id}");
            assert_eq!(format!("0x{bits:016x}"), output, "{id}");
        }
        Outcome::Plan {
            ent_code,
            red_entitled,
            entitled_flows,
            work_used,
            val,
            val_direct,
            plan_accrued_p100,
            dirty_p100,
            accrued_p100,
            clean_p100,
        } => {
            assert_eq!((expected, context), ("ok", "-"), "{id}");
            assert_eq!(val, val_direct, "{id}: two-stage vs direct valuation");
            assert_eq!(
                plan_accrued_p100, accrued_p100,
                "{id}: plan vs val accrued_per_100"
            );

            let parts: Vec<&str> = output.split(':').collect();
            if parts.len() != 10 {
                return Err(format!("{id}: expected 10 output fields, got {}", parts.len()).into());
            }
            assert_eq!(ent_code, parts[0], "{id}: entitlements");
            assert_eq!(
                if *red_entitled { "1" } else { "0" },
                parts[1],
                "{id}: redemption_entitled"
            );
            assert_eq!(
                entitled_flows.to_string(),
                parts[2],
                "{id}: entitled_flow_count"
            );
            assert_eq!(work_used.to_string(), parts[3], "{id}: accrual_work_used");

            let exp_dirty = parse_bits(parts[4])?;
            let exp_accrued = parse_bits(parts[5])?;
            let exp_clean = parse_bits(parts[6])?;
            let exp_dirty_p100 = parse_bits(parts[7])?;
            let exp_accrued_p100 = parse_bits(parts[8])?;
            let exp_clean_p100 = parse_bits(parts[9])?;

            assert_eq!(
                val.accrued_amount().to_bits(),
                exp_accrued,
                "{id}: accrued_amount bits"
            );
            assert_eq!(
                *accrued_p100, exp_accrued_p100,
                "{id}: accrued_per_100 bits"
            );
            assert!(
                ulp_distance(val.dirty_amount().to_bits(), exp_dirty) <= 2,
                "{id}: dirty_amount bits actual=0x{:016x} expected=0x{exp_dirty:016x}",
                val.dirty_amount().to_bits()
            );
            assert!(
                ulp_distance(val.clean_amount().to_bits(), exp_clean) <= 2,
                "{id}: clean_amount bits actual=0x{:016x} expected=0x{exp_clean:016x}",
                val.clean_amount().to_bits()
            );
            assert!(
                ulp_distance(*dirty_p100, exp_dirty_p100) <= 2,
                "{id}: dirty_price_per_100 bits actual=0x{dirty_p100:016x} expected=0x{exp_dirty_p100:016x}"
            );
            assert!(
                ulp_distance(*clean_p100, exp_clean_p100) <= 2,
                "{id}: clean_price_per_100 bits actual=0x{clean_p100:016x} expected=0x{exp_clean_p100:016x}"
            );
        }
        Outcome::Failure(error) => {
            assert_eq!(expected, error.kind(), "{id}: {error}");
            assert_eq!(output, "-", "{id}");
            let actual = error_fields(error)?;
            let recorded = context_map(context)?;
            let actual_borrowed: BTreeMap<&str, String> = actual.into_iter().collect();
            assert_eq!(actual_borrowed, recorded, "{id}");
        }
    }
    Ok(())
}

fn header_counts(fixture: &Fixture, key: &str) -> TestResult<BTreeMap<String, usize>> {
    let text = fixture.header(key).ok_or("missing census header")?;
    let mut counts = BTreeMap::new();
    for item in text.split(';') {
        let (name, count) = item.split_once('=').ok_or("malformed census entry")?;
        if counts.insert(name.to_owned(), count.parse()?).is_some() {
            return Err(format!("duplicate {key} census entry {name}").into());
        }
    }
    Ok(counts)
}

fn bump(map: &mut BTreeMap<String, usize>, key: &str) {
    *map.entry(key.to_owned()).or_default() += 1;
}

#[test]
fn independent_bond_core_oracle_matches_every_row() -> TestResult {
    let fixture = pinned_fixture()?;
    assert_eq!(fixture.record_count(), 18_152);
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
    let mut profiles = BTreeMap::new();

    let mut distinct = BTreeSet::new();
    for (index, row) in fixture.records().enumerate() {
        assert_eq!(row.id(), format!("BOND-{:06}", index + 1));
        let inputs = COLUMNS[2..12]
            .iter()
            .map(|column| row.text(column))
            .collect::<Result<Vec<_>, _>>()?;
        distinct.insert(inputs.join("\t"));
        if row.text("op")? != "bond_plan" {
            for column in &COLUMNS[5..12] {
                assert_eq!(row.text(column)?, "-", "{}: unused {column}", row.id());
            }
        }
        let outcome = evaluate(&row)?;
        check(&row, &outcome)?;
        bump(&mut operations, row.text("op")?);
        bump(&mut expectations, row.text("expected")?);
        bump(&mut profiles, row.text("profile")?);
    }

    assert_eq!(distinct.len(), 18_128);
    assert_eq!(fixture.header("unique-input-count"), Some("18128"));
    assert_eq!(fixture.header("record-count"), Some("18152"));
    assert_eq!(operations, header_counts(&fixture, "operation-counts")?);
    assert_eq!(expectations, header_counts(&fixture, "expectation-counts")?);
    assert_eq!(profiles, header_counts(&fixture, "profile-counts")?);
    assert_eq!(expectations["ok"], 17_011);
    Ok(())
}
