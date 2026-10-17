//! FR-BOND-02/03/04/06/07/08/09/10: independent contextual bond oracle.
//!
//! Every expectation comes from the separately authored Fraction/Decimal
//! fixture, including full/partial supplied-product maps, exact query/work
//! ledgers and outward price/accrual enclosures. No global epsilon is used.

mod support;

use sabiquant::bonds::{
    Bond, BondError, BondPriceKind, BondPricingPlan, BondSettlement, BondYieldConvention,
    CouponEntitlement, ExCouponBoundary, ExCouponTerms,
};
use sabiquant::calendar::{
    BusinessDayConvention, Calendar, CalendarError, EndOfMonth, TargetCalendar,
};
use sabiquant::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
use sabiquant::daycount::{
    ActualActualIcma, Business252, DayCount, DayCountError, DeterminationPeriod, IcmaError,
    Thirty360Us, ThirtyE360Isda,
};
use sabiquant::rates::{AccrualConvention, Compounding, RateError};
use sabiquant::schedule::{
    DateGenerationRule, ExplicitStubDates, ScheduleBuilder, ScheduleCalendar,
};
use sabiquant::time::{Date, Frequency, Period, TimeUnit, Weekday};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const COLUMNS: [&str; 52] = [
    "id",
    "profile",
    "issue",
    "raw_dates",
    "pay_dates",
    "redemption_date",
    "redemption_bits",
    "ex_terms",
    "face_bits",
    "coupon_rate_bits",
    "coupon_context",
    "yield_context",
    "settlement",
    "inclusion",
    "comp",
    "yield_bits",
    "work_cap",
    "sched_tenor",
    "sched_rule",
    "sched_first",
    "sched_penultimate",
    "sched_eom",
    "sched_bdc",
    "sched_term_bdc",
    "sched_holidays",
    "sched_business",
    "sched_max",
    "expected",
    "error_index",
    "error_phase",
    "error_kind",
    "error_date",
    "error_max",
    "error_used",
    "error_required",
    "entitlements",
    "redemption_entitled",
    "work",
    "full_queries",
    "plan_queries",
    "accrued_lo_bits",
    "accrued_hi_bits",
    "dirty_lo_bits",
    "dirty_hi_bits",
    "clean_lo_bits",
    "clean_hi_bits",
    "raw_ideal_dirty_bits",
    "raw_ideal_accrued_bits",
    "raw_ideal_clean_bits",
    "raw_ideal_dirty_error_bits",
    "raw_ideal_accrued_error_bits",
    "raw_ideal_clean_error_bits",
];

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn pinned_fixture() -> TestResult<Fixture> {
    #[expect(
        clippy::disallowed_methods,
        reason = "test fingerprints independent fixture bytes"
    )]
    let text = std::fs::read_to_string(Fixture::repository_path(
        "validation/fixtures/bonds/contextual-oracles.fixture",
    ))?;
    let (_, body) = text.split_once("\n\n").ok_or("fixture body separator")?;
    let fixture = Fixture::parse(&text)?;
    assert_eq!(fixture.columns(), &COLUMNS.map(str::to_owned));
    assert_eq!(
        fixture.header("fixture-set"),
        Some("M8-BOND-CONTEXTUAL-ORACLE")
    );
    assert_eq!(
        fixture.header("body-fnv64"),
        Some(format!("{:016x}", fnv64(body.as_bytes())).as_str())
    );
    assert_eq!(fnv64(body.as_bytes()), 0x166b_e7e4_04bf_1b4f);
    for (key, value) in [
        (
            "body-sha256",
            "7c2c7f613fbbf5f081d5c5325a9d49d942ecd4d9a60ec91262586e251342adb6",
        ),
        (
            "generator-sha256",
            "3f2398b98f735c9cafde9321040cfb28009fc67d9b1857223be032e9095ba588",
        ),
        (
            "requirements-rows-sha256",
            "fb65bdd4c3fb3062ab1d44de8a5b442630079fcd943cf42202827f127984d27c",
        ),
        (
            "m8-bond-core-contract-sha256",
            "009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c",
        ),
        (
            "m3-icma-contract-sha256",
            "87b8da50d887d847919f611f0e3d073893d7065f0378e691ac4832ac979f0bfb",
        ),
        (
            "m3-us-afb-contract-sha256",
            "595df058cdd8e64a935b6fc992ef0ca903e7e7fbdfa4ed613bbe598a3afe10b4",
        ),
        (
            "m3-target-contract-sha256",
            "37a4696bbd8a45e3648ba5b8d28a48e30ab480ecf7f833d02875a67960212ede",
        ),
        (
            "m3-daycount-extensions-sha256",
            "7ffd2f02d4c430fd8fd095054ce6e67233ee9b27ef260c37a6370790809aee58",
        ),
        (
            "m4-date-contract-sha256",
            "47216e578082eb56daaffb6b49c45951a416f21fe36372baede2bd0a29cdea98",
        ),
        (
            "m7-cashflow-contract-sha256",
            "04db3050737177b9928011102937b7c979a9c0e25171e242551eec3fd08d80a2",
        ),
    ] {
        assert_eq!(fixture.header(key), Some(value), "{key}");
    }
    for (key, source, frozen) in [
        (
            "generator-fnv64",
            include_bytes!("../validation/generator/bond_contextual_oracles.py").as_slice(),
            0xfe1c_4f7a_8217_402d,
        ),
        (
            "m8-bond-core-contract-fnv64",
            include_bytes!("../docs/M8_BOND_CORE_CONTRACT.md").as_slice(),
            0xe86f_8537_32e7_dcc0,
        ),
        (
            "m3-icma-contract-fnv64",
            include_bytes!("../docs/M3_ICMA_CONTRACT.md").as_slice(),
            0x8317_e3e3_1fdd_3d7d,
        ),
        (
            "m3-us-afb-contract-fnv64",
            include_bytes!("../docs/M3_US_AFB_CONTRACT.md").as_slice(),
            0xd44f_56d8_7d76_8f19,
        ),
        (
            "m3-target-contract-fnv64",
            include_bytes!("../docs/M3_TARGET_CONTRACT.md").as_slice(),
            0x3c3d_89a2_a022_8578,
        ),
        (
            "m3-daycount-extensions-fnv64",
            include_bytes!("../docs/M3_DAYCOUNT_EXTENSIONS.md").as_slice(),
            0x6281_24e5_94b2_1b2c,
        ),
        (
            "m4-date-contract-fnv64",
            include_bytes!("../docs/M4_DATE_CONTRACT.md").as_slice(),
            0xe7c7_194f_cb13_d1df,
        ),
        (
            "m7-cashflow-contract-fnv64",
            include_bytes!("../docs/M7_CASHFLOW_CONTRACT.md").as_slice(),
            0x8f3f_1cf6_fc5f_b390,
        ),
    ] {
        assert_eq!(fnv64(source), frozen, "{key} frozen source");
        assert_eq!(
            fixture.header(key),
            Some(format!("{:016x}", fnv64(source)).as_str()),
            "{key}"
        );
    }
    assert_eq!(
        fnv64(include_bytes!(
            "../validation/generator/bond_contextual_oracles.py"
        )),
        0xfe1c_4f7a_8217_402d
    );
    let requirements = include_str!("../docs/REQUIREMENTS.md")
        .lines()
        .filter(|line| line.starts_with("| FR-BOND-"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(fnv64(requirements.as_bytes()), 0x758b_ecce_7eac_4813);
    assert_eq!(
        fixture.header("requirements-rows-fnv64"),
        Some("758becce7eac4813")
    );
    Ok(fixture)
}

fn bits(text: &str) -> TestResult<u64> {
    let digits = text.strip_prefix("0x").ok_or("missing bits prefix")?;
    if digits.len() != 16 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("expected 16 hexadecimal IEEE754 digits".into());
    }
    Ok(u64::from_str_radix(digits, 16)?)
}

fn real(text: &str) -> TestResult<f64> {
    Ok(f64::from_bits(bits(text)?))
}

fn dates(text: &str) -> TestResult<Vec<Date>> {
    if text == "-" {
        return Ok(Vec::new());
    }
    text.split(',')
        .map(|date| Ok(date.parse::<Date>()?))
        .collect()
}

fn frequency(text: &str) -> TestResult<Frequency> {
    let count: u32 = text.parse()?;
    Frequency::ALL
        .into_iter()
        .find(|frequency| frequency.events_per_year() == Some(count))
        .ok_or_else(|| format!("unsupported explicit frequency {text}").into())
}

fn compounding(text: &str) -> TestResult<Compounding> {
    match text {
        "Simple" => Ok(Compounding::SIMPLE),
        "Continuous" => Ok(Compounding::CONTINUOUS),
        _ => {
            let (method, count) = text.split_once(':').ok_or("missing compounding count")?;
            let frequency = frequency(count)?;
            Ok(match method {
                "Compounded" => Compounding::compounded(frequency)?,
                "SimpleThenCompounded" => Compounding::simple_then_compounded(frequency)?,
                "CompoundedThenSimple" => Compounding::compounded_then_simple(frequency)?,
                _ => return Err(format!("unknown compounding method {method}").into()),
            })
        }
    }
}

fn plain_calendar(mask: &str, holidays: &str, business: &str) -> TestResult<Calendar> {
    let mask: u8 = mask.parse()?;
    if mask >= 128 {
        return Err("weekend mask must use seven Monday-first bits".into());
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
    let parts: Vec<_> = text.split(':').collect();
    Ok(match parts.as_slice() {
        ["A365F"] => DayCount::Actual365Fixed.into(),
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
        _ => return Err(format!("unknown complete context {text}").into()),
    })
}

fn inclusion(text: &str) -> TestResult<PaymentDateInclusion> {
    match text {
        "Include" => Ok(PaymentDateInclusion::Include),
        "Exclude" => Ok(PaymentDateInclusion::Exclude),
        _ => Err(format!("unknown inclusion {text}").into()),
    }
}

fn ex_terms(text: &str) -> TestResult<Vec<Option<ExCouponTerms>>> {
    text.split(';')
        .map(|item| {
            if item == "-" {
                return Ok(None);
            }
            let (boundary, cutoff) = item.split_once(':').ok_or("ex-coupon encoding")?;
            let boundary = match boundary {
                "OnOrAfter" => ExCouponBoundary::OnOrAfter,
                "After" => ExCouponBoundary::After,
                _ => return Err(format!("unknown ex-coupon boundary {boundary}").into()),
            };
            Ok(Some(ExCouponTerms::new(cutoff.parse()?, boundary)))
        })
        .collect()
}

fn build_bond(row: &Record<'_>, coupon: &AccrualConvention) -> TestResult<Bond> {
    let raw = dates(row.text("raw_dates")?)?;
    let pay = dates(row.text("pay_dates")?)?;
    assert_eq!(raw.len(), 3, "{} raw shape", row.id());
    assert_eq!(pay.len(), 3, "{} adjusted shape", row.id());
    let mut calendar = Calendar::null();
    for date in dates(row.text("sched_holidays")?)? {
        calendar = calendar.with_holiday(date);
    }
    for date in dates(row.text("sched_business")?)? {
        calendar = calendar.without_holiday(date);
    }
    let tenor_parts: Vec<_> = row.text("sched_tenor")?.split(':').collect();
    let tenor = match tenor_parts.as_slice() {
        ["6M"] | ["6", "Months"] => Period::new(6, TimeUnit::Months),
        _ => return Err("expected explicit 6M schedule tenor".into()),
    };
    let first: Date = row.text("sched_first")?.parse()?;
    let penultimate: Date = row.text("sched_penultimate")?.parse()?;
    assert_eq!(
        (first, penultimate),
        (raw[1], raw[1]),
        "{} stub roles",
        row.id()
    );
    let rule: DateGenerationRule = row.text("sched_rule")?.parse()?;
    let eom: EndOfMonth = row.text("sched_eom")?.parse()?;
    let bdc: BusinessDayConvention = row.text("sched_bdc")?.parse()?;
    let terminal_bdc: BusinessDayConvention = row.text("sched_term_bdc")?.parse()?;
    let maximum: u32 = row.text("sched_max")?.parse()?;
    let schedule = ScheduleBuilder::new(
        raw[0],
        raw[2],
        ScheduleCalendar::Plain(calendar),
        bdc,
        terminal_bdc,
    )?
    .build_explicit(
        tenor,
        rule,
        ExplicitStubDates::new(Some(first), Some(penultimate))?,
        eom,
        maximum,
    )?;
    assert_eq!(schedule.contractual_dates(), raw, "{} raw dates", row.id());
    assert_eq!(
        schedule.adjusted_dates(),
        pay,
        "{} adjusted dates",
        row.id()
    );
    assert_eq!(schedule.generation_rule(), rule);
    assert_eq!(schedule.tenor(), Some(tenor));
    assert_eq!(schedule.end_of_month(), Some(eom));
    Ok(Bond::fixed_rate(
        row.text("issue")?.parse()?,
        schedule,
        real(row.text("face_bits")?)?,
        FixedCouponRate::new(real(row.text("coupon_rate_bits")?)?, coupon.clone())?,
        ex_terms(row.text("ex_terms")?)?,
        SimpleCashFlow::new(
            row.text("redemption_date")?.parse()?,
            real(row.text("redemption_bits")?)?,
        )?,
        3,
    )?)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rational {
    numerator: i128,
    denominator: i128,
}

impl Rational {
    fn parse(text: &str) -> TestResult<Self> {
        let (numerator, denominator) = text.split_once('/').ok_or("rational separator")?;
        let numerator: i128 = numerator.parse()?;
        let denominator: i128 = denominator.parse()?;
        if numerator < 0 || denominator <= 0 {
            return Err(
                "chronological rational must be nonnegative with positive denominator".into(),
            );
        }
        Ok(Self::normalized(numerator, denominator))
    }

    fn normalized(numerator: i128, denominator: i128) -> Self {
        let (mut a, mut b) = (numerator, denominator);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        Self {
            numerator: numerator / a,
            denominator: denominator / a,
        }
    }

    fn checked_add(self, other: Self) -> TestResult<Self> {
        let numerator = self
            .numerator
            .checked_mul(other.denominator)
            .and_then(|left| {
                other
                    .numerator
                    .checked_mul(self.denominator)
                    .and_then(|right| left.checked_add(right))
            })
            .ok_or("fixture rational sum exceeds test integer capacity")?;
        let denominator = self
            .denominator
            .checked_mul(other.denominator)
            .ok_or("fixture rational denominator exceeds test integer capacity")?;
        Ok(Self::normalized(numerator, denominator))
    }
}

struct Query<'a> {
    role: &'a str,
    start: Date,
    end: Date,
    rational: &'a str,
    nearest: &'a str,
    ulps: &'a str,
    cost: u64,
    parts: &'a str,
    products: &'a str,
    status: &'a str,
}

impl<'a> Query<'a> {
    fn parse(text: &'a str) -> TestResult<Self> {
        let fields: Vec<_> = text.split('~').collect();
        let [
            role,
            start,
            end,
            rational,
            nearest,
            ulps,
            cost,
            parts,
            products,
            status,
        ] = fields.as_slice()
        else {
            return Err(format!("query needs ten fields: {text}").into());
        };
        Ok(Self {
            role,
            start: start.parse()?,
            end: end.parse()?,
            rational,
            nearest,
            ulps,
            cost: cost.parse()?,
            parts,
            products,
            status,
        })
    }

    fn index(&self) -> TestResult<Option<usize>> {
        match self.role {
            "A" => Ok(None),
            role => Ok(Some(role.get(1..).ok_or("query role index")?.parse()?)),
        }
    }

    fn phase(&self) -> TestResult<&'static str> {
        match self.role.as_bytes().first() {
            Some(b'C') => Ok("coupon accrual"),
            Some(b'A') => Ok("discount anchor"),
            Some(b'P') => Ok("partial accrual"),
            Some(b'H') => Ok("discount horizon"),
            Some(b'R') => Ok("redemption discount horizon"),
            _ => Err(format!("unknown query role {}", self.role).into()),
        }
    }

    fn coupon_query(&self) -> bool {
        self.role.starts_with(['C', 'P'])
    }
}

fn queries(text: &str) -> TestResult<Vec<Query<'_>>> {
    if text == "-" {
        return Ok(Vec::new());
    }
    text.split(';').map(Query::parse).collect()
}

fn expected_work(context: &AccrualConvention, start: Date, end: Date) -> TestResult<u64> {
    let extra = match context {
        AccrualConvention::Business252(_) => u64::from(end.days_since(start).unsigned_abs()),
        AccrualConvention::Icma(icma) => u64::try_from(
            icma.periods()
                .iter()
                .filter(|period| start.max(period.start()) < end.min(period.end()))
                .count(),
        )?,
        _ => 0,
    };
    Ok(1 + extra)
}

fn check_context_error(
    error: &RateError,
    query: &Query<'_>,
    context: &AccrualConvention,
) -> TestResult {
    match (error, context) {
        (
            RateError::Icma(IcmaError::OutOfCoverage {
                start,
                end,
                coverage_start,
                coverage_end,
                ..
            }),
            AccrualConvention::Icma(icma),
        ) => {
            assert_eq!((*start, *end), (query.start, query.end));
            assert_eq!(
                (*coverage_start, *coverage_end),
                (icma.coverage_start(), icma.coverage_end())
            );
            assert!(error.source().is_some());
        }
        (
            RateError::DayCount(DayCountError::Calendar(
                CalendarError::OutsideValidity { .. } | CalendarError::UnverifiedCoverage { .. },
            )),
            AccrualConvention::Business252(_),
        ) => {
            assert!(error.source().is_some_and(|cause| cause.source().is_some()));
        }
        _ => return Err(format!("unexpected contextual query failure: {error:?}").into()),
    }
    Ok(())
}

fn check_query(
    row: &Record<'_>,
    query: &Query<'_>,
    context: &AccrualConvention,
    bond: &Bond,
    settlement: BondSettlement,
    full: bool,
) -> TestResult {
    assert!(
        query.start <= query.end,
        "{} {} chronology",
        row.id(),
        query.role
    );
    assert_eq!(
        query.cost,
        if full {
            0
        } else {
            expected_work(context, query.start, query.end)?
        },
        "{} {} work charge",
        row.id(),
        query.role
    );
    if query.status != "ok" {
        for field in [
            query.rational,
            query.nearest,
            query.ulps,
            query.parts,
            query.products,
        ] {
            assert_eq!(field, "-", "{} failed-query unused field", row.id());
        }
        match query.status {
            "work" => {}
            "coverage" => {
                let error = context
                    .year_fraction(query.start, query.end)
                    .err()
                    .ok_or("coverage query must fail")?;
                check_context_error(&error, query, context)?;
            }
            _ => return Err(format!("unknown ledger status {}", query.status).into()),
        }
        return Ok(());
    }
    let raw = Rational::parse(query.rational)?;
    let sum = query.parts.split('+').try_fold(
        Rational {
            numerator: 0,
            denominator: 1,
        },
        |sum, part| sum.checked_add(Rational::parse(part)?),
    )?;
    assert_eq!(
        sum,
        raw,
        "{} {} rational period decomposition",
        row.id(),
        query.role
    );
    let nearest = bits(query.nearest)?;
    let ulps: u64 = query.ulps.parse()?;
    assert!(matches!(ulps, 0 | 4), "{} known YF profile", row.id());
    let value = context.year_fraction(query.start, query.end)?;
    assert!(value.is_finite() && value >= 0.0);
    assert!(
        value.to_bits().abs_diff(nearest) <= ulps,
        "{} {} YF: {value:?}",
        row.id(),
        query.role
    );
    if raw.numerator == 0 {
        assert_eq!(
            value.to_bits(),
            0,
            "{} {} canonical zero",
            row.id(),
            query.role
        );
    }
    if query.coupon_query() {
        let index = query.index()?.ok_or("coupon query index")?;
        let coupon = &bond.coupons()[index];
        let mut mapping = BTreeMap::new();
        for entry in query.products.split(',') {
            let (yf, amount) = entry
                .split_once('=')
                .ok_or("YF/product mapping separator")?;
            let yf = bits(yf)?;
            assert!(
                yf.abs_diff(nearest) <= ulps,
                "{} product-map YF profile",
                row.id()
            );
            assert!(
                mapping.insert(yf, bits(amount)?).is_none(),
                "{} duplicate product mapping",
                row.id()
            );
        }
        let expected = *mapping
            .get(&value.to_bits())
            .ok_or("actual YF missing product mapping")?;
        let amount = if full {
            assert_eq!(coupon.accrual_fraction().to_bits(), value.to_bits());
            assert_eq!((coupon.start(), coupon.end()), (query.start, query.end));
            coupon.amount()
        } else {
            assert_eq!(coupon.start(), query.start);
            assert_eq!(settlement.date(), query.end);
            coupon.accrued_amount(settlement.date(), settlement.payment_date_inclusion())?
        };
        assert_eq!(
            amount.to_bits(),
            expected,
            "{} {} exact supplied triple product",
            row.id(),
            query.role
        );
    } else {
        assert_eq!(query.products, "-", "{} non-product query", row.id());
    }
    Ok(())
}

/// Check the independent query ledger against the retained public date/term
/// roles, including the attempted final query when work or coverage fails.
fn check_ledger(
    row: &Record<'_>,
    bond: &Bond,
    settlement: BondSettlement,
    coupon: &AccrualConvention,
    yield_context: &AccrualConvention,
) -> TestResult {
    let full = queries(row.text("full_queries")?)?;
    assert_eq!(full.len(), bond.coupons().len());
    for (index, query) in full.iter().enumerate() {
        assert_eq!(query.role, format!("C{index}"));
        assert_eq!(query.status, "ok");
        check_query(row, query, coupon, bond, settlement, true)?;
    }
    let eligible = |payment: Date| {
        payment > settlement.date()
            || (payment == settlement.date()
                && settlement.payment_date_inclusion() == PaymentDateInclusion::Include)
    };
    let mut expected = vec![("A".to_owned(), settlement.date(), settlement.date())];
    for (index, item) in bond.coupons().iter().enumerate() {
        if eligible(item.payment_date()) {
            if item.start() < settlement.date() && settlement.date() < item.end() {
                expected.push((format!("P{index}"), item.start(), settlement.date()));
            }
            let detached = bond.ex_coupon_terms()[index]
                .is_some_and(|terms| terms.is_ex_coupon(settlement.date()));
            if !detached {
                expected.push((format!("H{index}"), settlement.date(), item.payment_date()));
            }
        }
    }
    if eligible(bond.redemption().payment_date()) {
        expected.push((
            format!("R{}", bond.coupons().len()),
            settlement.date(),
            bond.redemption().payment_date(),
        ));
    }
    let ledger = queries(row.text("plan_queries")?)?;
    assert!(!ledger.is_empty() && ledger.len() <= expected.len());
    let maximum: u64 = row.text("work_cap")?.parse()?;
    let mut used = 0_u64;
    for (position, (query, (role, start, end))) in ledger.iter().zip(&expected).enumerate() {
        assert_eq!(
            (query.role, query.start, query.end),
            (role.as_str(), *start, *end),
            "{} ledger role/date",
            row.id()
        );
        let context = if query.role.starts_with('P') {
            coupon
        } else {
            yield_context
        };
        check_query(row, query, context, bond, settlement, false)?;
        match query.status {
            "work" => {
                assert!(used + query.cost > maximum);
                assert_eq!(position + 1, ledger.len());
                assert_eq!(row.text("error_used")?.parse::<u64>()?, used);
                assert_eq!(
                    row.text("error_required")?.parse::<u64>()?,
                    used + query.cost
                );
            }
            "ok" | "coverage" => {
                used += query.cost;
                assert!(used <= maximum);
                if query.status == "coverage" {
                    assert_eq!(position + 1, ledger.len());
                }
            }
            _ => return Err("unknown checked ledger status".into()),
        }
    }
    assert_eq!(
        used,
        row.text("work")?.parse::<u64>()?,
        "{} charged work",
        row.id()
    );
    if row.text("expected")? == "ok" {
        assert_eq!(ledger.len(), expected.len());
        assert!(ledger.iter().all(|query| query.status == "ok"));
    } else {
        let final_query = ledger.last().ok_or("failure query")?;
        assert_ne!(final_query.status, "ok");
        assert_eq!(
            final_query
                .index()?
                .map_or_else(|| "none".to_owned(), |index| index.to_string()),
            row.text("error_index")?
        );
        assert_eq!(final_query.phase()?, row.text("error_phase")?);
    }
    Ok(())
}

fn check_error(
    row: &Record<'_>,
    error: &BondError,
    yield_context: &AccrualConvention,
) -> TestResult {
    let final_queries = queries(row.text("plan_queries")?)?;
    let query = final_queries.last().ok_or("error query")?;
    let expected_index = query.index()?;
    match error {
        BondError::AccrualWorkLimitExceeded {
            index,
            phase,
            maximum,
            used,
            required_at_least,
            ..
        } => {
            assert_eq!(row.text("expected")?, "work");
            assert_eq!((*index, *phase), (expected_index, query.phase()?));
            assert_eq!(error.kind(), row.text("error_kind")?);
            assert_eq!(*maximum, row.text("error_max")?.parse::<u64>()?);
            assert_eq!(*used, row.text("error_used")?.parse::<u64>()?);
            assert_eq!(
                *required_at_least,
                row.text("error_required")?.parse::<u64>()?
            );
            assert!(error.source().is_none());
            assert_eq!(row.text("error_date")?, "-");
        }
        BondError::Rate {
            index,
            phase,
            cause,
            ..
        } => {
            assert_eq!((*index, *phase), (expected_index, query.phase()?));
            assert_eq!(error.kind(), "bond.rate");
            assert!(error.source().is_some());
            let direct = yield_context
                .year_fraction(query.start, query.end)
                .err()
                .ok_or("expected contextual source failure")?;
            assert_eq!(*cause, direct, "{} exact typed context error", row.id());
            check_context_error(cause, query, yield_context)?;
            match cause {
                RateError::Icma(inner) => {
                    assert_eq!(row.text("expected")?, "icma");
                    assert_eq!(inner.kind(), row.text("error_kind")?);
                    assert_eq!(row.text("error_date")?, "-");
                }
                RateError::DayCount(DayCountError::Calendar(inner)) => {
                    assert_eq!(row.text("expected")?, "calendar");
                    assert_eq!(inner.kind(), row.text("error_kind")?);
                    let date = match inner {
                        CalendarError::OutsideValidity { date, .. }
                        | CalendarError::UnverifiedCoverage { date, .. } => *date,
                        _ => return Err("unexpected TARGET error variant".into()),
                    };
                    assert_eq!(date, row.text("error_date")?.parse::<Date>()?);
                }
                _ => return Err("unexpected contextual bond cause".into()),
            }
            for field in ["error_max", "error_used", "error_required"] {
                assert_eq!(row.text(field)?, "-");
            }
        }
        _ => return Err(format!("unexpected bond failure {error:?}").into()),
    }
    for field in COLUMNS[35..37].iter().chain(&COLUMNS[40..]) {
        assert_eq!(row.text(field)?, "-", "{} unused {field}", row.id());
    }
    let root = sabiquant::error::Error::from(error.clone());
    assert_eq!(root.kind(), error.kind());
    assert_eq!(
        root.source()
            .and_then(|source| source.downcast_ref::<BondError>()),
        Some(error)
    );
    Ok(())
}

fn check_amount(row: &Record<'_>, amount: f64, name: &str) -> TestResult {
    let lower = real(row.text(&format!("{name}_lo_bits"))?)?;
    let upper = real(row.text(&format!("{name}_hi_bits"))?)?;
    assert!(lower.is_finite() && upper.is_finite() && lower <= upper);
    assert!(
        amount.is_finite() && lower <= amount && amount <= upper,
        "{} {name}: production {amount:?} outside [{lower:?}, {upper:?}]",
        row.id()
    );
    if amount == 0.0 {
        assert_eq!(amount.to_bits(), 0, "{} canonical {name} zero", row.id());
    }
    let anchor = real(row.text(&format!("raw_ideal_{name}_bits"))?)?;
    let allowance = real(row.text(&format!("raw_ideal_{name}_error_bits"))?)?;
    assert!(anchor.is_finite() && allowance.is_finite() && allowance >= 0.0);
    // Exact rounded difference cannot exceed an upper-directed representable
    // bound on the exact distance. This adds no net-price ULP allowance.
    let distance = sabiquant::math::exact_sum(&[amount, -anchor])?.abs();
    assert!(
        distance <= allowance,
        "{} {name} raw-ideal RN anchor linkage",
        row.id()
    );
    Ok(())
}

fn check_row(row: &Record<'_>) -> TestResult {
    let coupon = accrual(row.text("coupon_context")?)?;
    let yield_context = accrual(row.text("yield_context")?)?;
    let bond = build_bond(row, &coupon)?;
    let settlement = BondSettlement::new(
        row.text("settlement")?.parse()?,
        inclusion(row.text("inclusion")?)?,
    );
    check_ledger(row, &bond, settlement, &coupon, &yield_context)?;
    let convention =
        BondYieldConvention::new(yield_context.clone(), compounding(row.text("comp")?)?);
    let maximum: u64 = row.text("work_cap")?.parse()?;
    let yield_rate = real(row.text("yield_bits")?)?;
    match BondPricingPlan::new(&bond, settlement, &convention, maximum) {
        Ok(plan) => {
            assert_eq!(
                row.text("expected")?,
                "ok",
                "{} unexpected plan success",
                row.id()
            );
            for field in &COLUMNS[28..35] {
                assert_eq!(row.text(field)?, "-");
            }
            assert_eq!(plan.bond(), &bond);
            assert_eq!(plan.settlement(), settlement);
            assert_eq!(plan.yield_convention(), &convention);
            assert_eq!(plan.accrual_work_used(), row.text("work")?.parse::<u64>()?);
            let code: String = plan
                .coupon_entitlements()
                .iter()
                .map(|entitlement| match entitlement {
                    CouponEntitlement::Ceased => 'C',
                    CouponEntitlement::BuyerEntitled => 'B',
                    CouponEntitlement::ExCouponDetached => 'X',
                })
                .collect();
            assert_eq!(code, row.text("entitlements")?);
            assert_eq!(
                plan.redemption_entitled(),
                row.text("redemption_entitled")? == "1"
            );
            assert!(matches!(row.text("redemption_entitled")?, "0" | "1"));
            assert_eq!(
                plan.entitled_flow_count(),
                code.bytes().filter(|code| *code == b'B').count()
                    + usize::from(plan.redemption_entitled())
            );
            check_amount(row, plan.accrued_amount(), "accrued")?;
            for valuation in [
                plan.evaluate(yield_rate)?,
                bond.evaluate(settlement, &convention, yield_rate, maximum)?,
            ] {
                assert_eq!(valuation.settlement(), settlement);
                assert_eq!(valuation.face().to_bits(), bond.face().to_bits());
                assert_eq!(valuation.yield_rate().to_bits(), yield_rate.to_bits());
                assert_eq!(
                    valuation.accrued_amount().to_bits(),
                    plan.accrued_amount().to_bits()
                );
                check_amount(row, valuation.dirty_amount(), "dirty")?;
                check_amount(row, valuation.accrued_amount(), "accrued")?;
                check_amount(row, valuation.clean_amount(), "clean")?;
                assert_eq!(valuation.dirty_price()?.kind(), BondPriceKind::Dirty);
                assert_eq!(valuation.clean_price()?.kind(), BondPriceKind::Clean);
                assert_eq!(
                    valuation.accrued_per_100()?.to_bits(),
                    plan.accrued_per_100()?.to_bits()
                );
            }
        }
        Err(error) => {
            assert_ne!(row.text("expected")?, "ok", "{}: {error}", row.id());
            check_error(row, &error, &yield_context)?;
            let direct = bond
                .evaluate(settlement, &convention, yield_rate, maximum)
                .err()
                .ok_or("direct evaluation must preserve preparation failure")?;
            assert_eq!(direct, error);
        }
    }
    Ok(())
}

fn header_counts(fixture: &Fixture, key: &str) -> TestResult<BTreeMap<String, usize>> {
    let mut counts = BTreeMap::new();
    for entry in fixture
        .header(key)
        .ok_or("missing census header")?
        .split(';')
    {
        let (name, count) = entry.split_once('=').ok_or("census entry")?;
        assert!(
            counts.insert(name.to_owned(), count.parse()?).is_none(),
            "duplicate census key"
        );
    }
    Ok(counts)
}

#[test]
fn every_independent_contextual_bond_row_matches() -> TestResult {
    let fixture = pinned_fixture()?;
    assert_eq!(fixture.record_count(), 11_136);
    assert_eq!(fixture.header("record-count"), Some("11136"));
    assert_eq!(fixture.header("unique-input-count"), Some("11136"));
    let mut profiles = BTreeMap::<String, usize>::new();
    let mut configurations = BTreeMap::<String, usize>::new();
    let mut expectations = BTreeMap::<String, usize>::new();
    let mut query_counts = BTreeMap::<String, usize>::new();
    let mut yf_counts = BTreeMap::<String, usize>::new();
    let mut product_entries = 0_usize;
    let mut distinct = BTreeSet::new();
    for (index, row) in fixture.records().enumerate() {
        assert_eq!(row.id(), format!("BCTX-{:06}", index + 1));
        let input = COLUMNS[2..27]
            .iter()
            .map(|column| row.text(column))
            .collect::<Result<Vec<_>, _>>()?
            .join("\t");
        assert!(distinct.insert(input), "{} duplicate input", row.id());
        check_row(&row).map_err(|error| format!("{}: {error}", row.id()))?;
        for field in ["full_queries", "plan_queries"] {
            for query in queries(row.text(field)?)? {
                let role = query.role.get(..1).ok_or("empty query role")?;
                *query_counts
                    .entry(format!("{role}-{}", query.status))
                    .or_default() += 1;
                if query.status == "ok" {
                    *yf_counts.entry(query.ulps.to_owned()).or_default() += 1;
                    if query.products != "-" {
                        product_entries += query.products.split(',').count();
                    }
                }
            }
        }
        *profiles.entry(row.text("profile")?.to_owned()).or_default() += 1;
        *configurations
            .entry(row.text("comp")?.to_owned())
            .or_default() += 1;
        *expectations
            .entry(row.text("expected")?.to_owned())
            .or_default() += 1;
    }
    assert_eq!(profiles, header_counts(&fixture, "profile-counts")?);
    assert_eq!(
        configurations,
        header_counts(&fixture, "configuration-counts")?
    );
    assert_eq!(expectations, header_counts(&fixture, "expectation-counts")?);
    assert_eq!(
        query_counts,
        header_counts(&fixture, "ledger-query-counts")?
    );
    assert_eq!(yf_counts, header_counts(&fixture, "yf-profile-counts")?);
    assert_eq!(product_entries, 114_400);
    assert_eq!(fixture.header("product-map-entry-count"), Some("114400"));
    assert_eq!(fixture.header("precision-agreement-count"), Some("11136"));
    assert_eq!(fixture.header("equal-date-guard-control-count"), Some("12"));
    assert_eq!(configurations.len(), 32);
    assert!(configurations.values().all(|count| *count == 348));
    assert_eq!(
        expectations,
        BTreeMap::from([
            ("ok".to_owned(), 10432),
            ("work".to_owned(), 512),
            ("icma".to_owned(), 64),
            ("calendar".to_owned(), 128)
        ])
    );
    Ok(())
}
