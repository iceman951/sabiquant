//! Public source projections for the single M8 enclosure boundary reference.
//!
//! FR-BOND-02/03/06/07 and FR-ENCL-02/05/16: construct the eleven Generic
//! prepared sources from frozen public literals. The retained public convention
//! is queried for direct horizons; this does not inspect private cached flows.
//! Price-enclosure producer assertions belong to the subsequent E3 consumers.

mod support;

use sabiquant::bonds::{
    Bond, BondKind, BondPricingPlan, BondSettlement, BondYieldConvention, CouponEntitlement,
};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::DayCount;
use sabiquant::rates::Compounding;
use sabiquant::schedule::{DateGenerationRule, ScheduleBuilder, ScheduleCalendar, StubPolicy};
use sabiquant::time::{Date, Period, TimeUnit};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
type Inputs<'a> = BTreeMap<&'a str, &'a str>;

const COLUMNS: [&str; 33] = [
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

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn pinned_fixture() -> TestResult<Fixture> {
    let text = include_str!("../validation/fixtures/bonds/enclosure-boundary.fixture");
    let fixture = Fixture::parse(text)?;
    assert_eq!(fnv64(text.as_bytes()), 0x2521_a815_7abb_cf0c);
    assert_eq!(fixture.columns(), &COLUMNS.map(str::to_owned));
    for (key, expected) in [
        ("fixture-set", "M8-ENCLOSURE-BOUNDARY"),
        ("domain", "bond-price-enclosure"),
        ("record-count", "35"),
        ("generic-source-plan-count", "11"),
        (
            "family-sha256",
            "295247d3700a0a06c7fd1d9bde830740cb4a66bd927a38061e255150b8eb52aa",
        ),
        (
            "author-sha256",
            "45be375751e10c84c221189896d2908956b178a440214220339af0913ab60c76",
        ),
        (
            "checker-sha256",
            "bd5a90a0e405805cfcba70506dcac35030604f86f9f1f6866c4802c368f8152a",
        ),
        (
            "reused-source-sha256",
            "0e365b4f24c451886702a89719fe279a561a538e4ae437437241d6e57266bf07",
        ),
        (
            "reused-admission-sha256",
            "adfa6155113b77678a8c7d0262e0a83dae48d7ab8a749464bb3750983a6a3c3d",
        ),
    ] {
        assert_eq!(fixture.header(key), Some(expected), "{key}");
    }
    for (bytes, expected) in [
        (
            include_bytes!("../validation/generator/enclosure_boundary/FAMILY.json").as_slice(),
            0x5509_8fd7_8f3f_d199,
        ),
        (
            include_bytes!("../validation/generator/enclosure_boundary/author.py").as_slice(),
            0xaed9_63f3_a218_aa17,
        ),
        (
            include_bytes!("../validation/generator/enclosure_boundary/check.py").as_slice(),
            0x35a8_a1ae_e7d2_da04,
        ),
        (
            include_bytes!("../validation/generator/enclosure_boundary/REUSED_SOURCE_ROWS.jsonl")
                .as_slice(),
            0x25fd_3856_a717_9d85,
        ),
        (
            include_bytes!(
                "../validation/generator/enclosure_boundary/REUSED_ADMISSION_ROWS.jsonl"
            )
            .as_slice(),
            0x6027_c5b3_a480_434f,
        ),
    ] {
        assert_eq!(fnv64(bytes), expected, "reference input/code fingerprint");
    }
    Ok(fixture)
}

fn bits(text: &str) -> TestResult<u64> {
    if text.len() != 16
        || !text
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(format!("noncanonical binary64 bits: {text}").into());
    }
    Ok(u64::from_str_radix(text, 16)?)
}

fn inputs(text: &str) -> TestResult<Inputs<'_>> {
    let mut result = BTreeMap::new();
    for cell in text.split(';') {
        let (key, value) = cell.split_once('=').ok_or("missing input separator")?;
        if key.is_empty() || value.is_empty() || result.insert(key, value).is_some() {
            return Err("empty or duplicate constructor input".into());
        }
    }
    Ok(result)
}

fn input<'a>(values: &Inputs<'a>, name: &str) -> TestResult<&'a str> {
    values
        .get(name)
        .copied()
        .ok_or_else(|| format!("missing constructor input {name}").into())
}

fn dates(text: &str) -> TestResult<Vec<Date>> {
    text.split(',')
        .map(|s| s.parse().map_err(Into::into))
        .collect()
}

#[derive(Debug)]
struct ExpectedFlow {
    index: usize,
    payment: Date,
    amount: u64,
    horizon: u64,
}

fn expected_flows(text: &str) -> TestResult<Vec<ExpectedFlow>> {
    text.split(';')
        .map(|cell| {
            let columns = cell.split('@').collect::<Vec<_>>();
            let [index, payment, amount, horizon] = columns.as_slice() else {
                return Err("flow needs index, payment, amount and horizon".into());
            };
            Ok(ExpectedFlow {
                index: index.parse()?,
                payment: payment.parse()?,
                amount: bits(amount)?,
                horizon: bits(horizon)?,
            })
        })
        .collect()
}

fn generic_bond(values: &Inputs<'_>) -> TestResult<Bond> {
    let issue = input(values, "issue")?.parse()?;
    let face = f64::from_bits(bits(input(values, "face_bits")?)?);
    let redemption = SimpleCashFlow::new(
        input(values, "redemption_payment")?.parse()?,
        f64::from_bits(bits(input(values, "redemption_bits")?)?),
    )?;
    match input(values, "kind")? {
        "ZeroCoupon" => Ok(Bond::zero_coupon(
            issue,
            input(values, "contractual_maturity")?.parse()?,
            face,
            redemption,
        )?),
        "FixedRate" => {
            for (key, expected) in [
                ("coupon_accrual", "Actual365Fixed"),
                ("schedule_calendar", "Null"),
                ("schedule_convention", "Unadjusted"),
                ("schedule_termination_convention", "Unadjusted"),
                ("schedule_rule", "Forward"),
                ("schedule_stub", "None"),
                ("schedule_end_of_month", "Ignore"),
            ] {
                assert_eq!(input(values, key)?, expected, "{key}");
            }
            let raw = dates(input(values, "schedule_raw_dates")?)?;
            let schedule = ScheduleBuilder::new(
                input(values, "schedule_start")?.parse()?,
                input(values, "schedule_end")?.parse()?,
                ScheduleCalendar::Plain(Calendar::null()),
                BusinessDayConvention::Unadjusted,
                BusinessDayConvention::Unadjusted,
            )?
            .build_recurrent(
                Period::new(
                    input(values, "schedule_step_months")?.parse()?,
                    TimeUnit::Months,
                ),
                DateGenerationRule::Forward,
                StubPolicy::None,
                EndOfMonth::Ignore,
                input(values, "schedule_max_boundaries")?.parse()?,
            )?;
            assert_eq!(schedule.contractual_dates(), raw);
            assert_eq!(schedule.adjusted_dates(), raw);
            let ex = input(values, "ex_coupon")?
                .split(',')
                .map(|cell| {
                    assert_eq!(cell, "_");
                    None
                })
                .collect::<Vec<_>>();
            let rate = FixedCouponRate::new(
                f64::from_bits(bits(input(values, "coupon_rate_bits")?)?),
                DayCount::Actual365Fixed,
            )?;
            Ok(Bond::fixed_rate(
                issue,
                schedule,
                face,
                rate,
                ex,
                redemption,
                input(values, "max_flows")?.parse()?,
            )?)
        }
        other => Err(format!("unexpected instrument kind {other}").into()),
    }
}

fn public_generic_source(row: Record<'_>, values: &Inputs<'_>) -> TestResult {
    let bond = generic_bond(values)?;
    assert_eq!(
        bond.face().to_bits(),
        bits(row.text("face_bits")?)?,
        "{} face",
        row.id()
    );
    assert_eq!(bond.face().to_bits(), bits(input(values, "face_bits")?)?);
    assert_eq!(bond.issue_date(), input(values, "issue")?.parse::<Date>()?);
    assert_eq!(
        bond.redemption().amount().to_bits(),
        bits(input(values, "redemption_bits")?)?
    );
    assert_eq!(
        bond.redemption().payment_date(),
        input(values, "redemption_payment")?.parse::<Date>()?
    );
    let expected = expected_flows(row.text("flows")?)?;
    let coupon_bits = if row.text("coupon_amount_bits")? == "_" {
        Vec::new()
    } else {
        row.text("coupon_amount_bits")?
            .split(',')
            .map(bits)
            .collect::<TestResult<Vec<_>>>()?
    };
    assert_eq!(bond.coupons().len(), coupon_bits.len());
    let compounding = match row.text("compounding")? {
        "Simple" => Compounding::SIMPLE,
        "Continuous" => Compounding::CONTINUOUS,
        other => return Err(format!("unexpected compounding {other}").into()),
    };
    let convention = BondYieldConvention::new(DayCount::Actual365Fixed, compounding);
    assert_eq!(input(values, "inclusion")?, "Include");
    let settlement = BondSettlement::new(
        input(values, "settlement")?.parse()?,
        PaymentDateInclusion::Include,
    );
    let plan = BondPricingPlan::new(
        &bond,
        settlement,
        &convention,
        input(values, "max_accrual_work")?.parse()?,
    )?;
    assert!(std::ptr::eq(plan.bond(), &bond));
    assert!(std::ptr::eq(plan.yield_convention(), &convention));
    assert_eq!(plan.settlement(), settlement);
    assert_eq!(plan.yield_convention().compounding(), compounding);
    assert_eq!(
        plan.accrued_amount().to_bits(),
        bits(row.text("accrued_bits")?)?
    );
    assert_eq!(plan.accrued_amount().to_bits(), 0);
    assert!(plan.redemption_entitled());
    assert_eq!(plan.entitled_flow_count(), expected.len());
    assert_eq!(
        plan.coupon_entitlements(),
        vec![CouponEntitlement::BuyerEntitled; coupon_bits.len()]
    );
    for (index, coupon) in bond.coupons().iter().enumerate() {
        assert_eq!(
            coupon.amount().to_bits(),
            coupon_bits[index],
            "{} coupon{index}",
            row.id()
        );
        assert_eq!(coupon.notional().to_bits(), bond.face().to_bits());
        assert_eq!(
            coupon.coupon_rate().rate().to_bits(),
            bits(input(values, "coupon_rate_bits")?)?
        );
        assert_eq!(coupon.accrual_fraction().to_bits(), 1.0_f64.to_bits());
        assert!(
            bond.ex_coupon_terms()
                .get(index)
                .ok_or("missing ex term")?
                .is_none()
        );
    }
    if bond.kind() == BondKind::ZeroCoupon {
        assert!(bond.schedule().is_none() && bond.coupon_rate().is_none());
        assert_eq!(
            bond.contractual_maturity(),
            input(values, "contractual_maturity")?.parse::<Date>()?
        );
    } else {
        let raw = dates(input(values, "schedule_raw_dates")?)?;
        let schedule = bond.schedule().ok_or("missing retained fixed schedule")?;
        assert_eq!(schedule.contractual_dates(), raw);
        assert_eq!(schedule.adjusted_dates(), raw);
        for (coupon, pair) in bond.coupons().iter().zip(raw.windows(2)) {
            assert_eq!((coupon.start(), coupon.end()), (pair[0], pair[1]));
        }
    }
    for (ordinal, flow) in expected.iter().enumerate() {
        assert_eq!(flow.index, ordinal, "{} original ordinal", row.id());
        let (amount, payment) = if let Some(coupon) = bond.coupons().get(flow.index) {
            (coupon.amount().to_bits(), coupon.payment_date())
        } else {
            assert_eq!(flow.index, bond.coupons().len());
            (
                bond.redemption().amount().to_bits(),
                bond.redemption().payment_date(),
            )
        };
        assert_eq!(
            (amount, payment),
            (flow.amount, flow.payment),
            "{} original source{ordinal}",
            row.id()
        );
        // This checks the retained PUBLIC direct-accrual convention. The
        // prepared vector is private; no cached-flow inspection is claimed.
        let direct = plan
            .yield_convention()
            .accrual()
            .year_fraction(settlement.date(), flow.payment)?;
        assert_eq!(
            direct.to_bits(),
            flow.horizon,
            "{} direct horizon{ordinal}",
            row.id()
        );
    }
    Ok(())
}

fn integer_lexeme(text: &str, positive: bool) -> TestResult {
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty()
        || !digits.bytes().all(|b| b.is_ascii_digit())
        || (digits.len() > 1 && digits.starts_with('0'))
        || text == "-0"
    {
        return Err(format!("noncanonical integer {text}").into());
    }
    if positive && (text.starts_with('-') || text == "0") {
        return Err("nonpositive denominator".into());
    }
    Ok(())
}

fn consume_target(row: Record<'_>) -> TestResult {
    assert!(f64::from_bits(bits(row.text("yield_bits")?)?).is_finite());
    for field in ["lower_bits", "upper_bits", "point_bits"] {
        if row.text(field)? != "_" {
            assert!(f64::from_bits(bits(row.text(field)?)?).is_finite());
        }
    }
    match row.text("target_kind")? {
        "exact_rational" => {
            integer_lexeme(row.text("numerator")?, false)?;
            integer_lexeme(row.text("denominator")?, true)?;
            for field in [
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
                assert_eq!(row.text(field)?, "_");
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
                    integer_lexeme(n, false)?;
                    integer_lexeme(d, true)?;
                }
            }
            assert!(matches!(row.text("lower_strict")?, "0" | "1"));
            assert!(matches!(row.text("upper_strict")?, "0" | "1"));
        }
        other => return Err(format!("unexpected target kind {other}").into()),
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
            let coefficient: i128 = row.parse("point_coefficient")?;
            let scale: u8 = row.parse("point_scale")?;
            assert!(scale <= 38 && (scale == 0 || coefficient % 10 != 0));
            assert_eq!(row.text("point_bits")?, "_");
        }
        "binary" => {
            bits(row.text("point_bits")?)?;
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
        other => return Err(format!("unexpected point kind {other}").into()),
    }
    Ok(())
}

fn consume_reused_named(row: Record<'_>, values: &Inputs<'_>) -> TestResult {
    let expected = match row.text("model")? {
        "Gilt" => (
            "B2-REUSE-GILT",
            "ENC-SOURCE-008961",
            "ENC-ADMISSION-0013165",
            "Periodic",
        ),
        "Treasury" => (
            "B2-REUSE-TREASURY",
            "ENC-SOURCE-009261",
            "ENC-ADMISSION-0014035",
            "Periodic",
        ),
        "Canadian" => (
            "B2-REUSE-CANADIAN",
            "ENC-SOURCE-011422",
            "ENC-ADMISSION-0016090",
            "MoneyMarket",
        ),
        other => return Err(format!("unexpected reused model {other}").into()),
    };
    assert_eq!(
        (
            row.text("context_id")?,
            row.text("source_plan_id")?,
            row.text("admission_case_id")?,
            row.text("compounding")?
        ),
        expected
    );
    assert_eq!(row.text("yield_bits")?, "7fefffffffffffff");
    for field in ["face_bits", "accrued_bits", "coupon_amount_bits", "flows"] {
        assert_eq!(row.text(field)?, "_");
    }
    for (key, value) in values {
        match *key {
            "issue" | "settlement" | "maturity" | "first_coupon" | "dated" | "true_maturity" => {
                value.parse::<Date>()?;
            }
            "quasi" | "ex_cutoffs" | "coupons" => {
                dates(value)?;
            }
            "coupon_coefficient" => assert_eq!(*value, "0"),
            "coupon_scale" => assert_eq!(*value, "0"),
            "construction_date_cap" | "construction_bundle_cap" | "frequency" => {
                assert!(value.parse::<u32>()? > 0);
            }
            "preserve_eom" => assert_eq!(*value, "false"),
            "daily_basis" => assert_eq!(*value, "ExactRatio"),
            "tie" => assert_eq!(*value, "NearestTiesAwayFromZero"),
            "selection" => assert_eq!(*value, "MoneyMarket"),
            "source_policy" => assert_eq!(*value, "CanonicalIiac13LocalV1"),
            other => return Err(format!("unexpected named constructor field {other}").into()),
        }
    }
    // Released source construction/cache gates are reused. There is no named
    // observed price replay or fabricated financial enclosure in this test.
    Ok(())
}

#[test]
fn boundary_fixture_projects_public_generic_sources() -> TestResult {
    let fixture = pinned_fixture()?;
    assert_eq!(fixture.record_count(), 35);
    let mut models = BTreeMap::new();
    let mut kinds = BTreeMap::new();
    let mut generic_plans: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut contexts = BTreeSet::new();
    let mut quote_ids = BTreeSet::new();
    let mut named = BTreeSet::new();
    for row in fixture.records() {
        let model = row.text("model")?;
        *models.entry(model).or_insert(0_usize) += 1;
        *kinds.entry(row.text("target_kind")?).or_insert(0_usize) += 1;
        let basis = row.text("basis")?;
        let (quote_id, suffix) = row.id().rsplit_once(':').ok_or("missing basis suffix")?;
        assert_eq!(suffix, basis);
        quote_ids.insert(quote_id);
        assert!(matches!(
            (model, basis),
            ("Canadian", "Clean" | "ModelDirty" | "SettlementDirty")
                | ("Generic" | "Gilt" | "Treasury", "Dirty" | "Clean")
        ));
        let values = inputs(row.text("input")?)?;
        consume_target(row)?;
        if model == "Generic" {
            assert_eq!(row.text("admission_case_id")?, "_");
            contexts.insert(row.text("context_id")?);
            let source_fields = [
                "context_id",
                "compounding",
                "input",
                "face_bits",
                "accrued_bits",
                "coupon_amount_bits",
                "flows",
            ]
            .map(|column| row.text(column))
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
            if let Some(previous) =
                generic_plans.insert(row.text("source_plan_id")?, source_fields.clone())
            {
                assert_eq!(
                    previous, source_fields,
                    "duplicate source expectations agree"
                );
            } else {
                public_generic_source(row, &values)?;
            }
        } else {
            consume_reused_named(row, &values)?;
            named.insert((
                row.text("model")?,
                row.text("source_plan_id")?,
                row.text("admission_case_id")?,
            ));
        }
    }
    assert_eq!(
        models,
        BTreeMap::from([
            ("Generic", 28),
            ("Gilt", 2),
            ("Treasury", 2),
            ("Canadian", 3)
        ])
    );
    assert_eq!(
        kinds,
        BTreeMap::from([("exact_rational", 29), ("strict_range", 6)])
    );
    assert_eq!(
        (
            generic_plans.len(),
            contexts.len(),
            quote_ids.len(),
            named.len()
        ),
        (11, 10, 17, 3)
    );
    Ok(())
}
