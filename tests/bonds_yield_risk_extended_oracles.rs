//! FR-YR-02/03/04/10/11/12/14: independent conditioned financial enclosures.
//!
//! The separately authored Fraction/Decimal fixture propagates accepted supplied
//! day-count and elementary-price intervals through the prescribed extended
//! term rounding, signed reduction and final getter admission. Raw mathematical
//! RN anchors carry per-row absolute bounds, never a net-result ULP allowance.
//! No separately reevaluated PV selects an expected internal risk seed.
//!
//! The one-callback dirty-target checks exercise same-host plan reuse and cached
//! payloads; they are deliberately not independent root-accuracy evidence.

mod support;

use sabiquant::bonds::{
    Bond, BondError, BondPrice, BondPriceKind, BondPricingPlan, BondRisk, BondSettlement,
    BondYieldConvention, BondYieldError, BondYieldSensitivity, CouponEntitlement, ExCouponBoundary,
    ExCouponTerms,
};
use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth, JointRule, TargetCalendar};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{
    ActualActualIcma, Business252, DayCount, DeterminationPeriod, Thirty360Us, ThirtyE360Isda,
};
use sabiquant::error::{Error as RootError, NonFinite, ValueError};
use sabiquant::math::{RootBracket, SolverOptions, TerminationReason};
use sabiquant::rates::{AccrualConvention, Compounding};
use sabiquant::schedule::{
    DateGenerationRule, ExplicitStubDates, ScheduleBuilder, ScheduleCalendar,
};
use sabiquant::time::{Date, Frequency, Period, TimeUnit, Weekday};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::ops::Bound;
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const SIGN: u64 = 1 << 63;
const MEASURES: [(&str, &str); 6] = [
    ("p1", "price derivative"),
    ("p2", "price second derivative"),
    ("moment", "time weighted amount"),
    ("modified", "modified duration"),
    ("macaulay", "macaulay duration"),
    ("convexity", "convexity"),
];
const COLUMNS: [&str; 56] = [
    "id",
    "family",
    "kind",
    "issue",
    "raw_dates",
    "pay_dates",
    "settlement",
    "inclusion",
    "coupon_context",
    "yield_context",
    "face_bits",
    "rate_bits",
    "redemption_date",
    "redemption_bits",
    "ex_terms",
    "comp",
    "yield_bits",
    "status",
    "error_kind",
    "error_phase",
    "error_index",
    "flows",
    "dirty_lo",
    "dirty_hi",
    "p1_status",
    "p1_lo",
    "p1_hi",
    "p2_status",
    "p2_lo",
    "p2_hi",
    "moment_status",
    "moment_lo",
    "moment_hi",
    "modified_status",
    "modified_lo",
    "modified_hi",
    "macaulay_status",
    "macaulay_lo",
    "macaulay_hi",
    "convexity_status",
    "convexity_lo",
    "convexity_hi",
    "ideal_dirty",
    "ideal_p1",
    "ideal_p2",
    "ideal_moment",
    "ideal_modified",
    "ideal_macaulay",
    "ideal_convexity",
    "ideal_dirty_error",
    "ideal_p1_error",
    "ideal_p2_error",
    "ideal_moment_error",
    "ideal_modified_error",
    "ideal_macaulay_error",
    "ideal_convexity_error",
];

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn pinned_fixture() -> TestResult<Fixture> {
    // The frozen generator pins the pre-Treasury policy bytes. Preserve its
    // historical input rather than changing any fixture or expected hash.
    #[expect(
        clippy::disallowed_methods,
        reason = "test fingerprints independent fixture bytes"
    )]
    let text = std::fs::read_to_string(Fixture::repository_path(
        "validation/fixtures/bonds/yield-risk-extended-public.fixture",
    ))?;
    let (_, body) = text.split_once("\n\n").ok_or("fixture body separator")?;
    let fixture = Fixture::parse(&text)?;
    assert_eq!(fixture.columns(), &COLUMNS.map(str::to_owned));
    assert_eq!(fixture.record_count(), 2181);
    assert_eq!(fnv64(text.as_bytes()), 0xefae_fb26_04fe_2e08);
    assert_eq!(fnv64(body.as_bytes()), 0x9b33_1d64_f654_7739);
    for (key, value) in [
        ("fixture-set", "M8-BOND-YIELD-RISK-EXTENDED"),
        ("domain", "bonds"),
        ("record-count", "2181"),
        ("body-fnv64", "9b331d64f6547739"),
        (
            "body-sha256",
            "d8764331c4966ef13527e5151f250c663e3572025cfbd8cc5a01cee6720e6a86",
        ),
        (
            "records-sha256",
            "be9fe770ce84dd27c04a9f2b5f71e9dc2d143dcef0cd573c29abd912e3470a02",
        ),
        (
            "generator-sha256",
            "9c5e77b68dbd9c2212b4ae0fb2cd7cdbc9b420505a55d29c01afa0b559541b85",
        ),
        (
            "requirements-sha256",
            "eaecaec76703c6e7195684cf2f17fe93eec3ed8f00343361676f839379a24e08",
        ),
        ("decimal-precisions", "160,240"),
        (
            "schedule-metadata",
            "Null/Unadjusted/Unadjusted; explicit6M/Forward/Ignore; first=penultimate interior if present; cap3",
        ),
        ("compounding-count", "32"),
        ("public-column-count", "56"),
    ] {
        assert_eq!(fixture.header(key), Some(value), "{key}");
    }
    for (name, bytes, pin, sha) in [
        (
            "m8-bond-yield-risk-contract",
            include_bytes!("../docs/M8_BOND_YIELD_RISK_CONTRACT.md").as_slice(),
            0x2a98_3dab_d670_6130,
            "62e3bffe1f0f95b5742c81f7431539dc135d2fcf2d92b1dc31b6e1de9958605d",
        ),
        (
            "m8-bond-core-contract",
            include_bytes!("../docs/M8_BOND_CORE_CONTRACT.md").as_slice(),
            0xe86f_8537_32e7_dcc0,
            "009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c",
        ),
        (
            "m4-date-contract",
            include_bytes!("../docs/M4_DATE_CONTRACT.md").as_slice(),
            0xe7c7_194f_cb13_d1df,
            "47216e578082eb56daaffb6b49c45951a416f21fe36372baede2bd0a29cdea98",
        ),
        (
            "m3-icma-contract",
            include_bytes!("../docs/M3_ICMA_CONTRACT.md").as_slice(),
            0x8317_e3e3_1fdd_3d7d,
            "87b8da50d887d847919f611f0e3d073893d7065f0378e691ac4832ac979f0bfb",
        ),
        (
            "m3-us-afb-contract",
            include_bytes!("../docs/M3_US_AFB_CONTRACT.md").as_slice(),
            0xd44f_56d8_7d76_8f19,
            "595df058cdd8e64a935b6fc992ef0ca903e7e7fbdfa4ed613bbe598a3afe10b4",
        ),
        (
            "m3-daycount-extensions",
            include_bytes!("../docs/M3_DAYCOUNT_EXTENSIONS.md").as_slice(),
            0x6281_24e5_94b2_1b2c,
            "7ffd2f02d4c430fd8fd095054ce6e67233ee9b27ef260c37a6370790809aee58",
        ),
        (
            "m3-target-contract",
            include_bytes!("../docs/M3_TARGET_CONTRACT.md").as_slice(),
            0x3c3d_89a2_a022_8578,
            "37a4696bbd8a45e3648ba5b8d28a48e30ab480ecf7f833d02875a67960212ede",
        ),
        (
            "m6-solver-research",
            include_bytes!("../docs/M6_SOLVER_RESEARCH.md").as_slice(),
            0xb64e_3465_3d1b_9d41,
            "bdbbf00b6f83bef259c82777bdc62544a616dbc7a13fe4734522d08239040b42",
        ),
        (
            "numerical-correctness",
            include_bytes!("../docs/NUMERICAL_CORRECTNESS_YIELD_RISK_REFERENCE.md").as_slice(),
            0x1df1_aeaa_0859_744a,
            "360bc4cfe69d487cc7a94ba694ee791145530053eff7ee014a54db24a420c061",
        ),
    ] {
        assert_eq!(fnv64(bytes), pin, "{name} source");
        assert_eq!(
            fixture.header(&format!("contract-{name}-fnv64")),
            Some(format!("{pin:016x}").as_str())
        );
        assert_eq!(
            fixture.header(&format!("contract-{name}-sha256")),
            Some(sha)
        );
    }
    let generator = include_bytes!("../validation/generator/bond_yield_risk_extended_oracles.py");
    assert_eq!(fnv64(generator), 0xd9d9_e01d_78bb_d4b2);
    assert_eq!(fixture.header("generator-fnv64"), Some("d9d9e01d78bbd4b2"));
    let requirements = include_str!("../docs/REQUIREMENTS.md")
        .lines()
        .filter(|line| line.starts_with("| FR-YR-"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(fnv64(requirements.as_bytes()), 0xa627_3d93_9267_f859);
    assert_eq!(
        fixture.header("requirements-fnv64"),
        Some("a6273d939267f859")
    );
    Ok(fixture)
}

fn real(text: &str) -> TestResult<f64> {
    if text.len() != 16
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("expected sixteen lowercase binary64 hex digits: {text}").into());
    }
    let value = f64::from_bits(u64::from_str_radix(text, 16)?);
    if !value.is_finite() {
        return Err("fixture real must be finite".into());
    }
    Ok(value)
}

fn dates(text: &str) -> TestResult<Vec<Date>> {
    if text == "-" {
        return Ok(Vec::new());
    }
    text.split(',').map(|value| Ok(value.parse()?)).collect()
}

fn frequency(text: &str) -> TestResult<Frequency> {
    let count: u32 = text.parse()?;
    Frequency::ALL
        .into_iter()
        .find(|value| value.events_per_year() == Some(count))
        .ok_or_else(|| format!("unsupported explicit frequency {text}").into())
}

fn compounding(text: &str) -> TestResult<Compounding> {
    match text {
        "Simple" => Ok(Compounding::SIMPLE),
        "Continuous" => Ok(Compounding::CONTINUOUS),
        _ => {
            let (method, count) = text
                .split_once(':')
                .ok_or("compounding frequency missing")?;
            let frequency = frequency(count)?;
            match method {
                "Compounded" => Ok(Compounding::compounded(frequency)?),
                "SimpleThenCompounded" => Ok(Compounding::simple_then_compounded(frequency)?),
                "CompoundedThenSimple" => Ok(Compounding::compounded_then_simple(frequency)?),
                _ => Err(format!("unknown compounding {text}").into()),
            }
        }
    }
}

fn plain_calendar(mask: &str, holidays: &str, business: &str) -> TestResult<Calendar> {
    let mask: u8 = mask.parse()?;
    if mask >= 128 {
        return Err("weekend mask exceeds seven bits".into());
    }
    let weekends = Weekday::ALL
        .into_iter()
        .enumerate()
        .filter_map(|(index, day)| (mask & (1 << index) != 0).then_some(day));
    let mut calendar = Calendar::custom(weekends, dates(holidays)?);
    for date in dates(business)? {
        calendar = calendar.without_holiday(date);
    }
    Ok(calendar)
}

fn accrual(text: &str) -> TestResult<AccrualConvention> {
    Ok(match text.split(':').collect::<Vec<_>>().as_slice() {
        ["A360"] => DayCount::Actual360.into(),
        ["A365F"] => DayCount::Actual365Fixed.into(),
        ["AAISDA"] => DayCount::ActualActualIsda.into(),
        ["AFB"] => DayCount::ActualActualAfb.into(),
        ["30360"] => DayCount::Thirty360BondBasis.into(),
        ["30E360"] => DayCount::ThirtyE360.into(),
        ["US", "0"] => Thirty360Us::new(false).into(),
        ["US", "1"] => Thirty360Us::new(true).into(),
        ["ISDA", termination] => ThirtyE360Isda::new(termination.parse()?).into(),
        ["ICMA", count, references] => {
            let boundaries = dates(references)?;
            let periods = boundaries
                .windows(2)
                .map(|pair| DeterminationPeriod::new(pair[0], pair[1]))
                .collect::<Result<Vec<_>, _>>()?;
            ActualActualIcma::new(periods, frequency(count)?)?.into()
        }
        ["B252", "C", mask, holidays, business] => {
            Business252::new(plain_calendar(mask, holidays, business)?).into()
        }
        ["B252", "T"] => Business252::with_target(TargetCalendar::new()).into(),
        ["B252", "J", rule, mask, holidays, business] => {
            let rule = match *rule {
                "H" => JointRule::JoinHolidays,
                "B" => JointRule::JoinBusinessDays,
                _ => return Err(format!("unknown joint rule {rule}").into()),
            };
            Business252::with_target(
                TargetCalendar::new()
                    .joint_with(vec![plain_calendar(mask, holidays, business)?], rule)?,
            )
            .into()
        }
        _ => return Err(format!("unknown complete accrual context {text}").into()),
    })
}

fn ex_terms(text: &str) -> TestResult<Vec<Option<ExCouponTerms>>> {
    text.split(';')
        .map(|term| {
            if term == "-" {
                return Ok(None);
            }
            let (boundary, cutoff) = term.split_once(':').ok_or("missing ex-coupon cutoff")?;
            let boundary = match boundary {
                "After" => ExCouponBoundary::After,
                "OnOrAfter" => ExCouponBoundary::OnOrAfter,
                _ => return Err(format!("unknown ex-coupon boundary {boundary}").into()),
            };
            Ok(Some(ExCouponTerms::new(cutoff.parse()?, boundary)))
        })
        .collect()
}

fn build_bond(row: &Record<'_>, coupon: &AccrualConvention) -> TestResult<Bond> {
    let raw = dates(row.text("raw_dates")?)?;
    let pay = dates(row.text("pay_dates")?)?;
    assert_eq!(raw, pay, "{} unadjusted dates", row.id());
    assert!((2..=3).contains(&raw.len()));
    let issue: Date = row.text("issue")?.parse()?;
    assert!(
        issue <= raw[0],
        "{} issue precedes contractual start",
        row.id()
    );
    let face = real(row.text("face_bits")?)?;
    assert_eq!(face, 100.0, "normalization identity for reuse checks");
    let rate = real(row.text("rate_bits")?)?;
    let redemption = SimpleCashFlow::new(
        row.text("redemption_date")?.parse()?,
        real(row.text("redemption_bits")?)?,
    )?;
    match row.text("kind")? {
        "Zero" => {
            assert_eq!(raw.len(), 2);
            assert_eq!(rate.to_bits(), 0);
            assert_eq!(row.text("ex_terms")?, "-");
            Ok(Bond::zero_coupon(issue, raw[1], face, redemption)?)
        }
        "Explicit" => {
            let tenor = Period::new(6, TimeUnit::Months);
            let builder = ScheduleBuilder::new(
                raw[0],
                *raw.last().ok_or("missing maturity")?,
                ScheduleCalendar::Plain(Calendar::null()),
                BusinessDayConvention::Unadjusted,
                BusinessDayConvention::Unadjusted,
            )?;
            // The frozen descriptive metadata overstates the6M construction
            // for these three endpoint-only fixed controls. Empty explicit
            // stub roles are invalid; a public Zero schedule retains exactly
            // their fully encoded raw/pay dates. No financial cell changes.
            let schedule = if raw.len() == 2 {
                assert!(
                    [
                        "SIGNED-CANCELLATION-DISCRIMINATOR",
                        "EXACT-ZERO-DIRTY",
                        "NEGATIVE-DIRTY"
                    ]
                    .contains(&row.text("family")?)
                );
                let schedule = builder.build_zero(2)?;
                assert_eq!(schedule.generation_rule(), DateGenerationRule::Zero);
                assert_eq!(schedule.tenor(), None);
                assert_eq!(schedule.end_of_month(), None);
                schedule
            } else {
                let middle = Some(raw[1]);
                let schedule = builder.build_explicit(
                    tenor,
                    DateGenerationRule::Forward,
                    ExplicitStubDates::new(middle, middle)?,
                    EndOfMonth::Ignore,
                    3,
                )?;
                assert_eq!(schedule.tenor(), Some(tenor));
                assert_eq!(schedule.generation_rule(), DateGenerationRule::Forward);
                assert_eq!(schedule.end_of_month(), Some(EndOfMonth::Ignore));
                schedule
            };
            assert_eq!(schedule.contractual_dates(), raw);
            assert_eq!(schedule.adjusted_dates(), pay);
            let terms = ex_terms(row.text("ex_terms")?)?;
            assert_eq!(terms.len(), raw.len() - 1);
            Ok(Bond::fixed_rate(
                issue,
                schedule,
                face,
                FixedCouponRate::new(rate, coupon.clone())?,
                terms,
                redemption,
                u32::try_from(raw.len())?,
            )?)
        }
        kind => Err(format!("unknown public bond kind {kind}").into()),
    }
}

fn enclosure(value: f64, lo: f64, hi: f64, label: &str) {
    assert!(lo <= hi, "{label} ordered bounds {lo:?}..{hi:?}");
    assert!(
        value.is_finite() && lo <= value && value <= hi,
        "{label}: {value:?} ({:016x}) outside {lo:?}..{hi:?}",
        value.to_bits()
    );
    if value == 0.0 {
        assert_eq!(value.to_bits(), 0, "{label} canonical zero");
    }
}

fn check_flows(
    row: &Record<'_>,
    bond: &Bond,
    plan: &BondPricingPlan<'_>,
    context: &AccrualConvention,
) -> TestResult<BondYieldSensitivity> {
    let settlement: Date = row.text("settlement")?.parse()?;
    let include = match row.text("inclusion")? {
        "Include" => true,
        "Exclude" => false,
        other => return Err(format!("unknown inclusion {other}").into()),
    };
    let eligible = |date| date > settlement || (include && date == settlement);
    let mut flows = Vec::new();
    let terms = if bond.coupons().is_empty() {
        Vec::new()
    } else {
        ex_terms(row.text("ex_terms")?)?
    };
    let mut entitlements = Vec::new();
    for (coupon, term) in bond.coupons().iter().zip(terms) {
        let entitlement = if !eligible(coupon.payment_date()) {
            CouponEntitlement::Ceased
        } else if term.is_some_and(|term| term.is_ex_coupon(settlement)) {
            CouponEntitlement::ExCouponDetached
        } else {
            CouponEntitlement::BuyerEntitled
        };
        entitlements.push(entitlement);
        if entitlement == CouponEntitlement::BuyerEntitled {
            flows.push((
                coupon.amount(),
                context.year_fraction(settlement, coupon.payment_date())?,
            ));
        }
    }
    assert_eq!(plan.coupon_entitlements(), entitlements);
    let redemption = bond.redemption();
    assert_eq!(
        plan.redemption_entitled(),
        eligible(redemption.payment_date())
    );
    if eligible(redemption.payment_date()) {
        flows.push((
            redemption.amount(),
            context.year_fraction(settlement, redemption.payment_date())?,
        ));
    }
    assert_eq!(plan.entitled_flow_count(), flows.len());
    let expected = if row.text("flows")? == "-" {
        Vec::new()
    } else {
        row.text("flows")?.split(';').collect::<Vec<_>>()
    };
    assert_eq!(
        flows.len(),
        expected.len(),
        "{} original flow count",
        row.id()
    );
    let sensitivity = if flows.iter().all(|(amount, _)| *amount >= 0.0)
        && flows
            .iter()
            .any(|(amount, time)| *amount > 0.0 && *time > 0.0)
    {
        BondYieldSensitivity::MathematicallyStrictlyDecreasing
    } else {
        BondYieldSensitivity::Uncertified
    };
    for ((amount, time), expected) in flows.into_iter().zip(expected) {
        let bounds = expected
            .split(':')
            .map(real)
            .collect::<TestResult<Vec<_>>>()?;
        assert_eq!(bounds.len(), 4);
        enclosure(
            amount,
            bounds[0],
            bounds[1],
            &format!("{} supplied flow amount", row.id()),
        );
        enclosure(
            time,
            bounds[2],
            bounds[3],
            &format!("{} supplied flow time", row.id()),
        );
    }
    Ok(sensitivity)
}

fn getter_values(risk: &BondRisk) -> [Result<f64, BondError>; 6] {
    [
        risk.price_derivative(),
        risk.price_second_derivative(),
        risk.time_weighted_amount(),
        risk.modified_duration(),
        risk.macaulay_duration(),
        risk.convexity(),
    ]
}

fn assert_typed_getter_error(
    error: BondError,
    status: &str,
    phase: &str,
    yield_rate: f64,
) -> TestResult {
    if status == "zero_dirty" {
        match &error {
            BondError::ZeroDirtyPrice {
                measure,
                yield_rate: actual,
                ..
            } => {
                assert_eq!(*measure, phase);
                assert_eq!(actual.to_bits(), yield_rate.to_bits());
            }
            _ => return Err(format!("expected zero dirty {phase}: {error:?}").into()),
        }
    } else {
        let negative = status.ends_with('-');
        match (&error, status) {
            (
                BondError::Value {
                    index: None,
                    phase: actual,
                    cause:
                        ValueError::ResultNotFinite {
                            quantity, value, ..
                        },
                    ..
                },
                "overflow+" | "overflow-",
            ) => {
                assert_eq!(*actual, phase);
                assert_eq!(*quantity, phase);
                assert_eq!(
                    *value,
                    if negative {
                        NonFinite::NegativeInfinity
                    } else {
                        NonFinite::PositiveInfinity
                    }
                );
            }
            (
                BondError::Value {
                    index: None,
                    phase: actual,
                    cause:
                        ValueError::ResultOutOfDomain {
                            quantity,
                            value,
                            domain,
                            ..
                        },
                    ..
                },
                "underflow+" | "underflow-",
            ) => {
                assert_eq!(*actual, phase);
                assert_eq!(*quantity, phase);
                assert_eq!(value.to_bits(), if negative { SIGN } else { 0 });
                assert_eq!(
                    domain.lower(),
                    if negative {
                        Bound::Unbounded
                    } else {
                        Bound::Excluded(0.0)
                    }
                );
                assert_eq!(
                    domain.upper(),
                    if negative {
                        Bound::Excluded(0.0)
                    } else {
                        Bound::Unbounded
                    }
                );
            }
            _ => return Err(format!("unexpected {status} {phase} error: {error:?}").into()),
        }
    }
    let root = RootError::from(error.clone());
    assert_eq!(
        root.source()
            .ok_or("missing typed bond source")?
            .downcast_ref::<BondError>(),
        Some(&error)
    );
    if status == "zero_dirty" {
        assert_eq!(root.kind(), "bond.zero_dirty_price");
        assert!(error.source().is_none());
    } else {
        assert_eq!(root.kind(), "bond.value");
        let leaf = error.source().ok_or("missing typed numeric cause")?;
        assert!(leaf.is::<ValueError>());
        assert!(leaf.source().is_none());
    }
    Ok(())
}

fn check_ideal(
    row: &Record<'_>,
    name: &str,
    actual: Option<f64>,
    staged_status: &str,
) -> TestResult {
    let expected = row.text(&format!("ideal_{name}"))?;
    let error = row.text(&format!("ideal_{name}_error"))?;
    match expected {
        "overflow+" | "overflow-" | "underflow+" | "underflow-" | "zero_dirty" => {
            assert_eq!(
                error,
                "-",
                "{} {name} nonfinite ideal unused allowance",
                row.id()
            );
            assert_eq!(
                staged_status,
                expected,
                "{} {name} ideal range classification",
                row.id()
            );
        }
        _ => {
            let ideal = real(expected)?;
            if let Some(actual) = actual {
                let bound = real(error)?;
                assert!(bound.is_sign_positive());
                assert!(
                    absolute_difference_within(actual, ideal, bound),
                    "{} {name} ideal anchor {ideal:?}, actual {actual:?}, allowance {bound:?}",
                    row.id()
                );
                for edge in ["lo", "hi"] {
                    let field = if name == "dirty" {
                        format!("dirty_{edge}")
                    } else {
                        format!("{name}_{edge}")
                    };
                    assert!(
                        absolute_difference_within(real(row.text(&field)?)?, ideal, bound),
                        "{} {name} allowance covers entire staged enclosure",
                        row.id()
                    );
                }
            } else {
                assert_eq!(
                    error,
                    "-",
                    "{} {name} failed staged getter allowance",
                    row.id()
                );
            }
        }
    }
    Ok(())
}

// TwoSum retains the exact binary64 difference as high+low. Comparing the
// remainder at equality prevents a rounded subtraction from accepting an
// absolute error just above the supplied outward binary64 bound.
fn absolute_difference_within(actual: f64, ideal: f64, bound: f64) -> bool {
    let high = actual - ideal;
    if !high.is_finite() {
        return false;
    }
    let virtual_right = high - actual;
    let low = (actual - (high - virtual_right)) + (-ideal - virtual_right);
    high.abs() < bound
        || (high.abs().to_bits() == bound.to_bits()
            && if high >= 0.0 { low <= 0.0 } else { low >= 0.0 })
}

fn check_reuse(
    row: &Record<'_>,
    plan: &BondPricingPlan<'_>,
    risk: &BondRisk,
    sensitivity: BondYieldSensitivity,
) -> TestResult {
    let work = plan.accrual_work_used();
    let y = risk.yield_rate();
    let bracket = RootBracket::new(y, y.next_up())?;
    let options = SolverOptions::new(f64::from_bits(1), 0.0, 1)?;
    let target = BondPrice::new(risk.dirty_amount(), BondPriceKind::Dirty)?;
    match plan.yield_from_price(target, bracket, options) {
        Ok(report) => {
            assert_ne!(row.text("family")?, "EXACT-ZERO-DIRTY");
            assert_eq!(report.yield_rate().to_bits(), y.to_bits());
            assert_eq!(
                report.dirty_amount().to_bits(),
                risk.dirty_amount().to_bits()
            );
            assert_eq!(
                report.dirty_target_amount().to_bits(),
                risk.dirty_amount().to_bits()
            );
            assert_eq!(
                report.converted_target_amount().to_bits(),
                risk.dirty_amount().to_bits()
            );
            assert_eq!(report.target(), target);
            assert_eq!(report.face(), 100.0);
            assert_eq!(report.settlement(), risk.settlement());
            assert_eq!(report.compounding(), risk.compounding());
            assert_eq!(
                report.accrued_amount().to_bits(),
                plan.accrued_amount().to_bits()
            );
            assert_eq!(report.initial_bracket(), bracket);
            assert_eq!(report.options(), options);
            assert_eq!(report.sensitivity(), sensitivity);
            assert_eq!(report.amount_residual()?.to_bits(), 0);
            let solve = report.solve_report();
            assert_eq!(solve.reason(), TerminationReason::FunctionZero);
            assert_eq!(solve.point().to_bits(), y.to_bits());
            assert_eq!(solve.residual().to_bits(), 0);
            assert_eq!(solve.evaluations().function(), 1);
            assert_eq!(solve.evaluations().derivative(), 0);
            assert_eq!(solve.iterations(), 0);
            assert_eq!(solve.bracket(), bracket);
        }
        Err(error @ BondYieldError::NoYieldSensitivity { .. }) => {
            assert_eq!(row.text("family")?, "EXACT-ZERO-DIRTY");
            match &error {
                BondYieldError::NoYieldSensitivity {
                    bracket: actual_bracket,
                    constant_amount,
                    dirty_target_amount,
                    observed_lower_dirty_amount,
                    evaluations,
                    ..
                } => {
                    assert_eq!(constant_amount.map(f64::to_bits), Some(0));
                    assert_eq!(dirty_target_amount.to_bits(), 0);
                    assert_eq!(observed_lower_dirty_amount.to_bits(), 0);
                    assert_eq!(*actual_bracket, bracket);
                    assert_eq!(evaluations.function(), 1);
                    assert_eq!(evaluations.derivative(), 0);
                }
                _ => return Err("constant match changed variant".into()),
            }
            assert!(error.source().is_none());
        }
        Err(error) => {
            return Err(format!("{} same-host lower-target reuse: {error:?}", row.id()).into());
        }
    }
    assert_eq!(
        plan.accrual_work_used(),
        work,
        "{} zero new accrual work",
        row.id()
    );
    Ok(())
}

/// Every finite financial row and every explicit getter range outcome is used.
#[test]
fn conditioned_public_risk_rows_and_cached_plan_reuse() -> TestResult {
    let fixture = pinned_fixture()?;
    let mut statuses = BTreeMap::<String, usize>::new();
    for row in fixture.records() {
        assert_eq!(row.text("status")?, "ok");
        for field in ["error_kind", "error_phase", "error_index"] {
            assert_eq!(row.text(field)?, "-");
        }
        let coupon = accrual(row.text("coupon_context")?)?;
        let yield_context = accrual(row.text("yield_context")?)?;
        let bond = build_bond(&row, &coupon)?;
        let inclusion = match row.text("inclusion")? {
            "Include" => PaymentDateInclusion::Include,
            "Exclude" => PaymentDateInclusion::Exclude,
            other => return Err(format!("unknown payment inclusion {other}").into()),
        };
        let settlement = BondSettlement::new(row.text("settlement")?.parse()?, inclusion);
        let comp = compounding(row.text("comp")?)?;
        let convention = BondYieldConvention::new(yield_context.clone(), comp);
        let plan = BondPricingPlan::new(&bond, settlement, &convention, u64::MAX)?;
        let sensitivity = check_flows(&row, &bond, &plan, &yield_context)?;
        let work = plan.accrual_work_used();
        let y = real(row.text("yield_bits")?)?;
        let risk = plan
            .risk(y)
            .map_err(|error| format!("{} risk: {error:?}", row.id()))?;
        assert_eq!(plan.accrual_work_used(), work);
        assert_eq!(risk.yield_rate().to_bits(), y.to_bits());
        assert_eq!(risk.face().to_bits(), bond.face().to_bits());
        assert_eq!(risk.settlement(), settlement);
        assert_eq!(risk.compounding(), comp);
        enclosure(
            risk.dirty_amount(),
            real(row.text("dirty_lo")?)?,
            real(row.text("dirty_hi")?)?,
            &format!("{} dirty", row.id()),
        );
        check_ideal(&row, "dirty", Some(risk.dirty_amount()), "ok")?;
        for ((name, phase), result) in MEASURES.into_iter().zip(getter_values(&risk)) {
            let status = row.text(&format!("{name}_status"))?;
            *statuses.entry(format!("{name}:{status}")).or_default() += 1;
            let actual = match status {
                "ok" => {
                    let value =
                        result.map_err(|error| format!("{} {name}: {error:?}", row.id()))?;
                    enclosure(
                        value,
                        real(row.text(&format!("{name}_lo"))?)?,
                        real(row.text(&format!("{name}_hi"))?)?,
                        &format!("{} {name}", row.id()),
                    );
                    Some(value)
                }
                "overflow+" | "overflow-" | "underflow+" | "underflow-" | "zero_dirty" => {
                    assert_eq!(row.text(&format!("{name}_lo"))?, "-");
                    assert_eq!(row.text(&format!("{name}_hi"))?, "-");
                    let error = result.err().ok_or("expected typed risk getter failure")?;
                    assert_typed_getter_error(error, status, phase, y)?;
                    None
                }
                other => return Err(format!("unknown getter status {other}").into()),
            };
            check_ideal(&row, name, actual, status)?;
        }
        check_reuse(&row, &plan, &risk, sensitivity)?;
    }
    assert_eq!(statuses.values().sum::<usize>(), 2181 * 6);
    for name in ["modified", "macaulay", "convexity"] {
        assert_eq!(statuses[&format!("{name}:zero_dirty")], 1);
    }
    assert_eq!(statuses["p1:overflow-"], 1);
    assert_eq!(statuses["p2:overflow+"], 1);
    assert_eq!(statuses["moment:overflow+"], 1);
    assert_eq!(statuses["p1:underflow-"], 1);
    assert_eq!(statuses["p2:underflow+"], 1);
    assert_eq!(statuses["moment:underflow+"], 1);
    Ok(())
}

/// The family/profile cross-product is checked independently of financial values.
#[test]
fn complete_profile_compounding_and_input_census() -> TestResult {
    let fixture = pinned_fixture()?;
    let mut families = BTreeMap::<String, usize>::new();
    let mut profiles = BTreeMap::<String, BTreeSet<String>>::new();
    let mut observed_comps = BTreeSet::new();
    let mut comp_counts = BTreeMap::<String, usize>::new();
    let mut contexts = BTreeSet::new();
    let mut expected_comps = BTreeSet::from(["Simple".to_owned(), "Continuous".to_owned()]);
    for method in ["Compounded", "SimpleThenCompounded", "CompoundedThenSimple"] {
        for frequency in [1, 2, 3, 4, 6, 12, 13, 26, 52, 365] {
            expected_comps.insert(format!("{method}:{frequency}"));
        }
    }
    let expected_pairs = expected_comps
        .iter()
        .flat_map(|comp| {
            ["3fa3333333333333", "bfb0000000000000"].map(move |y| format!("{comp}/{y}"))
        })
        .collect::<BTreeSet<_>>();
    for (index, row) in fixture.records().enumerate() {
        assert_eq!(row.id(), format!("BYRPX-{:06}", index + 1));
        let family = row.text("family")?;
        *families.entry(family.to_owned()).or_default() += 1;
        let comp = row.text("comp")?;
        observed_comps.insert(comp.to_owned());
        *comp_counts.entry(comp.to_owned()).or_default() += 1;
        if index < 2176 {
            let key = [
                family,
                row.text("coupon_context")?,
                row.text("yield_context")?,
            ]
            .join("/");
            assert!(
                profiles
                    .entry(key)
                    .or_default()
                    .insert(format!("{comp}/{}", row.text("yield_bits")?))
            );
            contexts.insert(row.text("yield_context")?.to_owned());
        }
    }
    assert_eq!(observed_comps, expected_comps);
    assert_eq!(profiles.len(), 34);
    let refs = "2026-08-31,2027-02-28,2027-08-31,2028-02-29,2028-08-31";
    let expected_contexts = [
        "A360".to_owned(),
        "A365F".to_owned(),
        "AAISDA".to_owned(),
        "AFB".to_owned(),
        "30360".to_owned(),
        "30E360".to_owned(),
        "ISDA:2028-02-29".to_owned(),
        "ISDA:2028-03-01".to_owned(),
        "US:0".to_owned(),
        "US:1".to_owned(),
        format!("ICMA:2:{refs}"),
        "B252:C:48:2027-09-06,2028-02-28:2027-09-05".to_owned(),
        "B252:T".to_owned(),
        "B252:J:H:65:2027-09-06:2027-09-05".to_owned(),
        "B252:J:B:48:2027-09-06:2027-09-05".to_owned(),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(contexts, expected_contexts);
    for pairs in profiles.values() {
        assert_eq!(pairs, &expected_pairs);
    }
    let expected = BTreeMap::from([
        ("CONTEXT-SIGNED-COUPONS".to_owned(), 960),
        ("CONTEXT-ZERO".to_owned(), 960),
        ("EX-DETACHED".to_owned(), 64),
        ("EXACT-ZERO-DIRTY".to_owned(), 1),
        ("ICMA-LONG-STUB-REOPENING".to_owned(), 64),
        ("ICMA-SHORT-STUB".to_owned(), 64),
        ("MAX-RAW-OVERFLOW-RESCUE".to_owned(), 1),
        ("MIN-RAW-UNDERFLOW-RESCUE".to_owned(), 1),
        ("NEGATIVE-DIRTY".to_owned(), 1),
        ("PAYMENT-DATE-INCLUDED".to_owned(), 64),
        ("SIGNED-CANCELLATION-DISCRIMINATOR".to_owned(), 1),
    ]);
    assert_eq!(families, expected);
    let encoded = format!(
        "{{{}}}",
        families
            .iter()
            .map(|(key, value)| format!("\"{key}\":{value}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    assert_eq!(fixture.header("census"), Some(encoded.as_str()));
    for (name, counts) in [
        ("family-counts", &families),
        ("compounding-counts", &comp_counts),
    ] {
        let encoded = counts
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(fixture.header(name), Some(encoded.as_str()));
    }
    Ok(())
}
