//! Independent civil/Fraction products and two-precision flat-cash-flow oracles.

#![expect(
    clippy::unwrap_used,
    reason = "committed fixture decoding fails immediately"
)]
#![expect(
    clippy::expect_used,
    reason = "checked independent integer oracle arithmetic"
)]
#![expect(
    clippy::panic,
    reason = "unknown fixture conventions/operations are malformed test data"
)]

mod support;

use sabiquant::calendar::{Calendar, TargetCalendar};
use sabiquant::cashflows::{
    CashFlow, CashFlowError, FixedCouponRate, FixedRateCoupon, Leg, PaymentDateInclusion,
    SimpleCashFlow,
};
use sabiquant::daycount::{
    ActualActualIcma, Business252, DayCount, DeterminationPeriod, ThirtyE360Isda,
};
use sabiquant::error::{NonFinite, ValueError};
use sabiquant::rates::{AccrualConvention, Compounding, CompoundingMethod, InterestRate};
use sabiquant::time::{Date, Frequency, Weekday};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;
use support::fixture::{Fixture, Record};
use support::float::ulp_distance;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Ratio {
    numerator: i128,
    denominator: i128,
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Ratio {
    fn new(n: i128, d: i128) -> Self {
        assert!(d > 0);
        let reduction = gcd(n.abs(), d);
        Self {
            numerator: n / reduction,
            denominator: d / reduction,
        }
    }

    fn add(self, other: Self) -> Self {
        let shared = gcd(self.denominator, other.denominator);
        let n = self
            .numerator
            .checked_mul(other.denominator / shared)
            .and_then(|a| {
                other
                    .numerator
                    .checked_mul(self.denominator / shared)
                    .and_then(|b| a.checked_add(b))
            })
            .expect("bounded independent rational addition");
        let d = self
            .denominator
            .checked_mul(other.denominator / shared)
            .expect("bounded independent denominator");
        Self::new(n, d)
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "explicit 2^53 bounds give exact binary64 quotient inputs"
    )]
    fn rounded(self) -> f64 {
        assert!(self.numerator.abs() <= 1_i128 << 53);
        assert!(self.denominator <= 1_i128 << 53);
        self.numerator as f64 / self.denominator as f64
    }
}

fn month_days(year: i32, month: u32) -> u32 {
    match month {
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn ordinal((year, month, day): (i32, u32, u32)) -> i64 {
    let y = i64::from(year - 1);
    365 * y + y / 4 - y / 100
        + y / 400
        + (1..month)
            .map(|m| i64::from(month_days(year, m)))
            .sum::<i64>()
        + i64::from(day - 1)
}

fn next((year, month, day): (i32, u32, u32)) -> (i32, u32, u32) {
    if day < month_days(year, month) {
        (year, month, day + 1)
    } else if month < 12 {
        (year, month + 1, 1)
    } else {
        (year + 1, 1, 1)
    }
}

fn date(text: &str) -> Date {
    text.parse().unwrap()
}

fn count(frequency: &str) -> u32 {
    match frequency {
        "Annual" => 1,
        "Semiannual" => 2,
        "EveryFourthMonth" => 3,
        "Quarterly" => 4,
        "Bimonthly" => 6,
        "Monthly" => 12,
        "EveryFourthWeek" => 13,
        "Biweekly" => 26,
        "Weekly" => 52,
        "Daily" => 365,
        _ => panic!("invalid periodic frequency {frequency}"),
    }
}

fn descriptor<'a>(fixture: &'a Fixture, context: &str) -> &'a str {
    fixture
        .header(&format!("context-{}", context.to_ascii_lowercase()))
        .unwrap()
}

fn make_context(fixture: &Fixture, context: &str) -> AccrualConvention {
    let fields: Vec<_> = descriptor(fixture, context).split(':').collect();
    match fields[0] {
        "scalar" => match fields[1] {
            "Actual360" => DayCount::Actual360,
            "Actual365Fixed" => DayCount::Actual365Fixed,
            "ActualActualIsda" => DayCount::ActualActualIsda,
            "ThirtyE360" => DayCount::ThirtyE360,
            "Thirty360BondBasis" => DayCount::Thirty360BondBasis,
            other => panic!("unknown scalar {other}"),
        }
        .into(),
        "termination" => ThirtyE360Isda::new(date(fields[1])).into(),
        "icma" => {
            let dates: Vec<_> = fields[2].split('/').map(date).collect();
            let periods = dates
                .windows(2)
                .map(|p| DeterminationPeriod::new(p[0], p[1]).unwrap())
                .collect();
            ActualActualIcma::new(periods, fields[1].parse::<Frequency>().unwrap())
                .unwrap()
                .into()
        }
        "business" => {
            let mask: u32 = fields[1].parse().unwrap();
            let weekends = Weekday::ALL
                .into_iter()
                .filter(|day| mask & (1 << (day.number_from_monday() - 1)) != 0);
            let holidays = fields[2].split('/').filter(|x| !x.is_empty()).map(date);
            Business252::new(Calendar::custom(weekends, holidays)).into()
        }
        "target" => Business252::with_target(TargetCalendar::new()).into(),
        other => panic!("unknown context {other}"),
    }
}

// Civil-coordinate integer oracle; it never calls a production day counter,
// Date::days_since, Date::weekday, or calendar business-day classification.
fn exact_alpha(fixture: &Fixture, context: &str, start: Date, end: Date) -> Ratio {
    assert!(start <= end);
    let zero = Ratio::new(0, 1);
    let s = start.to_ymd();
    let e = end.to_ymd();
    let days = i128::from(ordinal(e) - ordinal(s));
    match context {
        "A360" => Ratio::new(days, 360),
        "A365" => Ratio::new(days, 365),
        "ISDA" => {
            let mut cursor = s;
            let mut total = zero;
            while cursor < e {
                let boundary = e.min((cursor.0 + 1, 1, 1));
                total = total.add(Ratio::new(
                    i128::from(ordinal(boundary) - ordinal(cursor)),
                    i128::from(if month_days(cursor.0, 2) == 29 {
                        366
                    } else {
                        365
                    }),
                ));
                cursor = boundary;
            }
            total
        }
        "30E" | "BOND" | "30E-TERM" => {
            let mut a = s.2;
            let mut b = e.2;
            match context {
                "30E" => {
                    a = a.min(30);
                    b = b.min(30);
                }
                "BOND" => {
                    a = a.min(30);
                    if b == 31 && a == 30 {
                        b = 30;
                    }
                }
                _ => {
                    if a == 31 || s.1 == 2 && a == month_days(s.0, 2) {
                        a = 30;
                    }
                    if b == 31 || e.1 == 2 && b == month_days(e.0, 2) && end != date("2024-02-29") {
                        b = 30;
                    }
                }
            }
            Ratio::new(
                i128::from(
                    360 * (e.0 - s.0)
                        + 30 * (i32::try_from(e.1).unwrap() - i32::try_from(s.1).unwrap())
                        + i32::try_from(b).unwrap()
                        - i32::try_from(a).unwrap(),
                ),
                360,
            )
        }
        name if name.starts_with("ICMA") => {
            let fields: Vec<_> = descriptor(fixture, context).split(':').collect();
            let refs: Vec<_> = fields[2].split('/').map(date).collect();
            assert!(start >= refs[0] && end <= *refs.last().unwrap());
            let annual = i128::from(count(fields[1]));
            let mut total = zero;
            for pair in refs.windows(2) {
                let left = start.max(pair[0]);
                let right = end.min(pair[1]);
                if left < right {
                    total = total.add(Ratio::new(
                        i128::from(ordinal(right.to_ymd()) - ordinal(left.to_ymd())),
                        annual * i128::from(ordinal(pair[1].to_ymd()) - ordinal(pair[0].to_ymd())),
                    ));
                }
            }
            total
        }
        name if name.starts_with("B252") => {
            let (mask, holidays) = if name == "B252-T" {
                (
                    96_u32,
                    vec![
                        "2026-01-01",
                        "2026-04-03",
                        "2026-04-06",
                        "2026-05-01",
                        "2026-12-25",
                        "2026-12-26",
                    ]
                    .into_iter()
                    .map(date)
                    .map(|d| d.to_ymd())
                    .collect::<Vec<_>>(),
                )
            } else {
                let fields: Vec<_> = descriptor(fixture, context).split(':').collect();
                (
                    fields[1].parse::<u32>().unwrap(),
                    fields[2]
                        .split('/')
                        .filter(|x| !x.is_empty())
                        .map(date)
                        .map(|d| d.to_ymd())
                        .collect::<Vec<_>>(),
                )
            };
            let mut cursor = s;
            let mut business = 0;
            while cursor < e {
                let weekday = u32::try_from(ordinal(cursor) % 7).unwrap();
                business += i128::from(mask & (1 << weekday) == 0 && !holidays.contains(&cursor));
                cursor = next(cursor);
            }
            Ratio::new(business, 252)
        }
        other => panic!("unknown oracle context {other}"),
    }
}

fn verify_alpha(fixture: &Fixture, context: &str, start: Date, end: Date, actual: f64) -> Ratio {
    let exact = exact_alpha(fixture, context, start, end);
    let radius = if context == "ISDA" || context.starts_with("ICMA") {
        4
    } else {
        0
    };
    assert!(
        ulp_distance(actual, exact.rounded()).is_some_and(|distance| distance <= radius),
        "{context} {start}..{end}: {actual:?} versus {exact:?}"
    );
    exact
}

fn metadata_alpha(row: &Record<'_>, prefix: &str, exact: Ratio) {
    let stem = if prefix.is_empty() { "alpha" } else { "query" };
    assert_eq!(
        row.parse::<i128>(&format!("{stem}_n")).unwrap(),
        exact.numerator
    );
    assert_eq!(
        row.parse::<i128>(&format!("{stem}_d")).unwrap(),
        exact.denominator
    );
    assert_eq!(
        row.real(if prefix.is_empty() {
            "alpha"
        } else {
            "query_alpha"
        })
        .unwrap()
        .to_bits(),
        exact.rounded().to_bits()
    );
}

fn product_outcome(map: &str, alpha: f64) -> &str {
    let wanted = format!("{:016x}", alpha.to_bits());
    map.split(';')
        .find_map(|text| {
            let (key, value) = text.split_once('=')?;
            (key == wanted).then_some(value)
        })
        .unwrap()
}

fn product_bits(map: &str, alpha: f64) -> Option<u64> {
    let entry = product_outcome(map, alpha);
    (!matches!(entry, "overflow" | "underflow")).then(|| u64::from_str_radix(entry, 16).unwrap())
}

fn inclusion(row: &Record<'_>) -> PaymentDateInclusion {
    match row.text("inclusion").unwrap() {
        "Include" => PaymentDateInclusion::Include,
        "Exclude" => PaymentDateInclusion::Exclude,
        other => panic!("invalid inclusion {other}"),
    }
}

fn eligible(payment: Date, cutoff: Date, inclusion: PaymentDateInclusion) -> bool {
    payment > cutoff || payment == cutoff && inclusion == PaymentDateInclusion::Include
}

fn compounding(row: &Record<'_>) -> Compounding {
    match row.parse::<CompoundingMethod>("method").unwrap() {
        CompoundingMethod::Simple => Compounding::SIMPLE,
        CompoundingMethod::Continuous => Compounding::CONTINUOUS,
        CompoundingMethod::Compounded => {
            Compounding::compounded(row.parse("frequency").unwrap()).unwrap()
        }
        CompoundingMethod::SimpleThenCompounded => {
            Compounding::simple_then_compounded(row.parse("frequency").unwrap()).unwrap()
        }
        CompoundingMethod::CompoundedThenSimple => {
            Compounding::compounded_then_simple(row.parse("frequency").unwrap()).unwrap()
        }
    }
}

fn flow_map<'a>(row: &Record<'a>, column: &str, index: usize) -> &'a str {
    let prefix = format!("{index}@");
    row.text(column)
        .unwrap()
        .split('|')
        .find_map(|entry| entry.strip_prefix(&prefix))
        .unwrap()
}

fn decode_flows(fixture: &Fixture, row: &Record<'_>) -> Vec<CashFlow> {
    row.text("flows")
        .unwrap()
        .split(';')
        .filter(|x| !x.is_empty())
        .enumerate()
        .map(|(index, text)| {
            let f: Vec<_> = text.split('@').collect();
            match f[0] {
                "S" => {
                    assert_eq!(f.len(), 3);
                    SimpleCashFlow::new(date(f[1]), f[2].parse().unwrap())
                        .unwrap()
                        .into()
                }
                "C" => {
                    assert_eq!(f.len(), 7);
                    let context = make_context(fixture, f[6]);
                    let start = date(f[1]);
                    let end = date(f[2]);
                    let quote = FixedCouponRate::new(f[5].parse().unwrap(), context).unwrap();
                    let coupon =
                        FixedRateCoupon::new(start, end, date(f[3]), f[4].parse().unwrap(), quote)
                            .unwrap();
                    verify_alpha(fixture, f[6], start, end, coupon.accrual_fraction());
                    assert_eq!(
                        Some(coupon.amount().to_bits()),
                        product_bits(
                            flow_map(row, "flow_products", index),
                            coupon.accrual_fraction()
                        )
                    );
                    coupon.into()
                }
                other => panic!("unknown flow type {other}"),
            }
        })
        .collect()
}

fn check_error(row: &Record<'_>, error: CashFlowError) {
    assert_eq!(
        error.kind(),
        row.text("error_kind").unwrap(),
        "{}: {error:?}",
        row.id()
    );
    let expected_index = row.parse_optional::<usize>("error_index").unwrap();
    let expected_phase = row.text("error_phase").unwrap();
    match error {
        CashFlowError::Rate {
            index,
            phase,
            cause,
            ..
        } => {
            assert_eq!(index, expected_index);
            assert_eq!(phase, expected_phase);
            assert_eq!(cause.kind(), row.text("cause_kind").unwrap());
        }
        CashFlowError::Sum { phase, cause, .. } => {
            assert_eq!(phase, expected_phase);
            assert_eq!(cause.kind(), row.text("cause_kind").unwrap());
        }
        CashFlowError::AccrualWorkLimitExceeded {
            index,
            phase,
            maximum,
            used,
            required_at_least,
            ..
        } => {
            assert_eq!(index, expected_index);
            assert_eq!(phase, expected_phase);
            assert_eq!(maximum, row.parse::<u64>("work_budget").unwrap());
            assert_eq!(used, row.parse::<u64>("work_used").unwrap());
            assert_eq!(
                required_at_least,
                row.parse::<u64>("work_required").unwrap()
            );
        }
        other => panic!("unexpected oracle error {other:?}"),
    }
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "reports measured operation/profile distributions with --nocapture"
)]
fn all_cashflow_analytics_match_independent_rational_products_and_absolute_propagation() {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/cashflows/analytics-oracle.fixture",
    ))
    .unwrap();
    let mut operations = BTreeMap::<String, usize>::new();
    let mut profiles = BTreeMap::<String, usize>::new();
    let mut statuses = BTreeMap::<String, usize>::new();
    let mut classifications = BTreeMap::<String, usize>::new();
    let mut configurations = BTreeSet::<(String, String)>::new();
    let mut maximum_normalized = 0.0_f64;
    let mut maximum_row = String::new();
    let mut maximum_details = (0.0, 0.0, 0.0);
    let mut numerical_values = 0;
    for row in fixture.records() {
        *operations
            .entry(row.text("operation").unwrap().to_owned())
            .or_default() += 1;
        *profiles
            .entry(row.text("tolerance").unwrap().to_owned())
            .or_default() += 1;
        let classification = if row.text("behavior").unwrap() == "product-map" {
            let outcomes = row
                .text("alpha_products")
                .unwrap()
                .split(';')
                .map(|entry| match entry.split_once('=').unwrap().1 {
                    "underflow" => "underflow",
                    "overflow" => "overflow",
                    _ => "value",
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join("-or-");
            format!("product-{outcomes}")
        } else if row.text("behavior").unwrap() == "error" {
            row.text("error_kind").unwrap().to_owned()
        } else {
            "aggregate-value".to_owned()
        };
        *classifications.entry(classification).or_default() += 1;
        if row.text("operation").unwrap() == "npv" {
            configurations.insert((
                row.text("method").unwrap().to_owned(),
                row.text("frequency").unwrap().to_owned(),
            ));
        }
        match row.text("operation").unwrap() {
            "coupon" | "accrued" => {
                let label = row.text("context").unwrap();
                let context = make_context(&fixture, label);
                let start = row.parse::<Date>("start").unwrap();
                let end = row.parse::<Date>("end").unwrap();
                let actual_alpha = context.year_fraction(start, end).unwrap();
                let exact = verify_alpha(&fixture, label, start, end, actual_alpha);
                metadata_alpha(&row, "", exact);
                let expected = product_bits(row.text("alpha_products").unwrap(), actual_alpha);
                let quote =
                    FixedCouponRate::new(row.real("coupon_rate").unwrap(), context).unwrap();
                let result = FixedRateCoupon::new(
                    start,
                    end,
                    row.parse("payment").unwrap(),
                    row.real("notional").unwrap(),
                    quote,
                );
                let Some(expected) = expected else {
                    let error = result.unwrap_err();
                    let CashFlowError::Value {
                        index,
                        phase,
                        cause,
                        ..
                    } = error
                    else {
                        panic!("{}: {error:?}", row.id())
                    };
                    assert_eq!(index, None);
                    assert_eq!(phase, "coupon amount");
                    let negative = row.real("notional").unwrap().is_sign_negative()
                        != row.real("coupon_rate").unwrap().is_sign_negative();
                    let outcome =
                        product_outcome(row.text("alpha_products").unwrap(), actual_alpha);
                    match (outcome, cause) {
                        (
                            "overflow",
                            ValueError::ResultNotFinite {
                                quantity, value, ..
                            },
                        ) => {
                            assert_eq!(quantity, "coupon amount");
                            assert_eq!(
                                value,
                                if negative {
                                    NonFinite::NegativeInfinity
                                } else {
                                    NonFinite::PositiveInfinity
                                }
                            );
                        }
                        (
                            "underflow",
                            ValueError::ResultOutOfDomain {
                                quantity,
                                value,
                                domain,
                                ..
                            },
                        ) => {
                            assert_eq!(quantity, "coupon amount");
                            assert_eq!(value.to_bits(), u64::from(negative) << 63);
                            let (open, unbounded) = if negative {
                                (domain.upper(), domain.lower())
                            } else {
                                (domain.lower(), domain.upper())
                            };
                            assert!(matches!(open,Bound::Excluded(x) if x.to_bits()==0));
                            assert!(matches!(unbounded, Bound::Unbounded));
                        }
                        (_, other) => panic!("{}: expected {outcome}, got {other:?}", row.id()),
                    }
                    *statuses.entry(format!("product-{outcome}")).or_default() += 1;
                    continue;
                };
                let coupon = result.unwrap();
                assert_eq!(coupon.accrual_fraction().to_bits(), actual_alpha.to_bits());
                assert_eq!(coupon.amount().to_bits(), expected, "{}", row.id());
                if row.text("operation").unwrap() == "accrued" {
                    let as_of = row.parse::<Date>("as_of").unwrap();
                    let policy = inclusion(&row);
                    let (alpha, exact) = match row.text("query_mode").unwrap() {
                        "zero" => {
                            assert!(
                                !eligible(coupon.payment_date(), as_of, policy) || as_of <= start
                            );
                            (0.0, Ratio::new(0, 1))
                        }
                        "full" => {
                            assert!(eligible(coupon.payment_date(), as_of, policy) && as_of >= end);
                            (actual_alpha, exact)
                        }
                        "partial" => {
                            assert!(
                                eligible(coupon.payment_date(), as_of, policy)
                                    && start < as_of
                                    && as_of < end
                            );
                            let alpha = coupon
                                .coupon_rate()
                                .accrual()
                                .year_fraction(start, as_of)
                                .unwrap();
                            (alpha, verify_alpha(&fixture, label, start, as_of, alpha))
                        }
                        other => panic!("invalid query mode {other}"),
                    };
                    metadata_alpha(&row, "query", exact);
                    let expected =
                        product_bits(row.text("query_products").unwrap(), alpha).unwrap();
                    assert_eq!(
                        coupon.accrued_amount(as_of, policy).unwrap().to_bits(),
                        expected,
                        "{}",
                        row.id()
                    );
                }
                *statuses.entry("product-value".to_owned()).or_default() += 1;
            }
            "npv" | "leg-accrued" => {
                let flows = decode_flows(&fixture, &row);
                let leg = Leg::new(
                    flows,
                    u32::try_from(
                        row.text("flows")
                            .unwrap()
                            .split(';')
                            .filter(|x| !x.is_empty())
                            .count(),
                    )
                    .unwrap(),
                )
                .unwrap();
                let as_of = row.parse::<Date>("as_of").unwrap();
                let policy = inclusion(&row);
                let budget = row.parse::<u64>("work_budget").unwrap();
                let result = if row.text("operation").unwrap() == "npv" {
                    let quote = InterestRate::new(
                        row.real("discount_rate").unwrap(),
                        make_context(&fixture, row.text("discount_context").unwrap()),
                        compounding(&row),
                    )
                    .unwrap();
                    leg.net_present_value(as_of, &quote, policy, budget)
                } else {
                    leg.accrued_amount(as_of, policy, budget)
                };
                if row.text("behavior").unwrap() == "error" {
                    let error = result.unwrap_err();
                    *statuses.entry(error.kind().to_owned()).or_default() += 1;
                    check_error(&row, error);
                    continue;
                }
                let actual = result.unwrap();
                let expected = row.real("expected").unwrap();
                let allowance = row.real("allowance").unwrap();
                assert!(actual.is_finite());
                assert!(allowance.is_finite() && allowance >= 0.0);
                if row.text("tolerance").unwrap() == "TOL-E0-SUM" {
                    assert_eq!(actual.to_bits(), expected.to_bits(), "{}", row.id());
                } else {
                    let difference = (actual - expected).abs();
                    assert!(
                        difference <= allowance,
                        "{}: actual={actual:?} expected={expected:?} difference={difference:?} allowance={allowance:?}",
                        row.id()
                    );
                    if allowance > 0.0 {
                        let normalized = difference / allowance;
                        if normalized > maximum_normalized {
                            maximum_normalized = normalized;
                            maximum_row = row.id().to_owned();
                            maximum_details = (difference, allowance, row.real("gross").unwrap());
                        }
                    }
                    numerical_values += 1;
                }
                if row.text("operation").unwrap() == "leg-accrued" {
                    for (index, flow) in leg.flows().iter().enumerate() {
                        if let CashFlow::FixedRateCoupon(coupon) = flow {
                            let alpha = if !eligible(coupon.payment_date(), as_of, policy)
                                || as_of <= coupon.start()
                            {
                                0.0
                            } else if as_of >= coupon.end() {
                                coupon.accrual_fraction()
                            } else {
                                coupon
                                    .coupon_rate()
                                    .accrual()
                                    .year_fraction(coupon.start(), as_of)
                                    .unwrap()
                            };
                            let bits =
                                product_bits(flow_map(&row, "query_flow_products", index), alpha)
                                    .unwrap();
                            assert_eq!(
                                flow.accrued_amount(as_of, policy).unwrap().to_bits(),
                                bits,
                                "{} flow{index}",
                                row.id()
                            );
                        }
                    }
                }
                *statuses.entry("aggregate-value".to_owned()).or_default() += 1;
            }
            other => panic!("unrecognized operation {other}"),
        }
    }
    assert_eq!(
        fixture.records().len(),
        fixture
            .header("record-count")
            .unwrap()
            .parse::<usize>()
            .unwrap()
    );
    assert_eq!(fixture.records().len(), 47_144);
    assert_eq!(
        operations,
        BTreeMap::from([
            ("accrued".to_owned(), 5_880),
            ("coupon".to_owned(), 10_481),
            ("leg-accrued".to_owned(), 464),
            ("npv".to_owned(), 30_319),
        ])
    );
    assert_eq!(
        profiles,
        BTreeMap::from([
            ("TOL-E0-PRODUCT".to_owned(), 16_361),
            ("TOL-E0-SUM".to_owned(), 8_429),
            ("TOL-E5-CASHFLOW-PROPAGATION".to_owned(), 22_354),
        ])
    );
    assert_eq!(
        classifications,
        BTreeMap::from([
            ("aggregate-value".to_owned(), 30_665),
            ("cashflow.accrual_work_limit_exceeded".to_owned(), 79),
            ("cashflow.rate".to_owned(), 38),
            ("cashflow.sum".to_owned(), 1),
            ("product-overflow".to_owned(), 397),
            ("product-underflow".to_owned(), 895),
            ("product-underflow-or-value".to_owned(), 4),
            ("product-value".to_owned(), 15_065),
        ])
    );
    // Four min-subnormal ICMA products lie exactly at a rounding boundary.
    // A permitted E2 alpha choice determines their precise range status;
    // each returned product/error has already been checked against that alpha.
    assert_eq!(statuses["aggregate-value"], 30_665);
    assert_eq!(statuses["cashflow.accrual_work_limit_exceeded"], 79);
    assert_eq!(statuses["cashflow.rate"], 38);
    assert_eq!(statuses["cashflow.sum"], 1);
    assert_eq!(statuses["product-overflow"], 397);
    assert!((895..=899).contains(&statuses["product-underflow"]));
    assert!((15_065..=15_069).contains(&statuses["product-value"]));
    assert_eq!(
        statuses["product-underflow"] + statuses["product-value"],
        15_964
    );
    assert_eq!(statuses.len(), 7);
    assert_eq!(numerical_values, 22_236);
    let mut expected_configurations = BTreeSet::from([
        ("Simple".to_owned(), String::new()),
        ("Continuous".to_owned(), String::new()),
    ]);
    for frequency in [
        "Annual",
        "Semiannual",
        "EveryFourthMonth",
        "Quarterly",
        "Bimonthly",
        "Monthly",
        "EveryFourthWeek",
        "Biweekly",
        "Weekly",
        "Daily",
    ] {
        for method in ["Compounded", "SimpleThenCompounded", "CompoundedThenSimple"] {
            expected_configurations.insert((method.to_owned(), frequency.to_owned()));
        }
    }
    assert_eq!(configurations, expected_configurations);
    println!(
        "cashflow-oracles operations={operations:?} profiles={profiles:?} classifications={classifications:?} statuses={statuses:?} numerical_values={numerical_values} max_normalized_error={maximum_normalized:?} max_row={maximum_row} (difference,allowance,gross)={maximum_details:?}"
    );
}
