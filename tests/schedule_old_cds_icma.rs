//! Generated ICMA references on the B-anchored OldCDS stride lattice.
//!
//! FR-SCHED-38: an independent Fraction oracle fixes hulls, exact rational
//! accruals, caps and civil causes; hand cases cover calendar coverage,
//! adjustment invariance, stored aliases and generic coupon composition.

mod support;

use sabiquant::calendar::{BusinessDayConvention as Bdc, Calendar, TargetCalendar, TargetPolicy};
use sabiquant::cashflows::{CashFlow, FixedCouponRate, Leg};
use sabiquant::daycount::ActualActualIcma;
use sabiquant::schedule::{Schedule, ScheduleBuilder, ScheduleCalendar, ScheduleError};
use sabiquant::time::{Date, DateError, Frequency, Period, TimeUnit};
use std::collections::BTreeMap;
use std::error::Error;
use support::fixture::{Fixture, Record};
use support::float::ulp_distance;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const CONVENTIONS: [Bdc; 7] = [
    Bdc::Unadjusted,
    Bdc::Following,
    Bdc::ModifiedFollowing,
    Bdc::Preceding,
    Bdc::ModifiedPreceding,
    Bdc::HalfMonthModifiedFollowing,
    Bdc::Nearest,
];

fn d(text: &str) -> TestResult<Date> {
    Ok(text.parse()?)
}

fn month_days(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// Independent Gregorian day ordinal; shares no Date difference with ICMA.
fn ordinal(date: Date) -> i64 {
    let prior = i64::from(date.year() - 1);
    let elapsed: u32 = (1..date.month())
        .map(|month| month_days(date.year(), month))
        .sum();
    365 * prior + prior / 4 - prior / 100 + prior / 400 + i64::from(elapsed + date.day())
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.abs()
}

/// Exact reduced Actual/Actual (ICMA) accrual over supplied references.
fn exact_fraction(references: &[Date], events: u32, start: Date, end: Date) -> (i128, i128) {
    let (mut n, mut d) = (0_i128, 1_i128);
    for pair in references.windows(2) {
        let a = start.max(pair[0]);
        let b = end.min(pair[1]);
        if a >= b {
            continue;
        }
        let tn = i128::from(ordinal(b) - ordinal(a));
        let td = i128::from(events) * i128::from(ordinal(pair[1]) - ordinal(pair[0]));
        n = n * td + tn * d;
        d *= td;
        let reduce = gcd(n, d);
        n /= reduce;
        d /= reduce;
    }
    (n, d)
}

fn rational(text: &str) -> TestResult<(i128, i128)> {
    let (n, d) = text.split_once('/').ok_or("fraction needs num/den")?;
    Ok((n.parse()?, d.parse()?))
}

fn dates(text: &str) -> TestResult<Vec<Date>> {
    text.split(';').map(|field| Ok(field.parse()?)).collect()
}

fn references(context: &ActualActualIcma) -> Vec<Date> {
    std::iter::once(context.coverage_start())
        .chain(context.periods().iter().map(|period| period.end()))
        .collect()
}

fn null_old_cds(start: Date, end: Date, tenor: Period) -> TestResult<Schedule> {
    Ok(ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?
    .build_old_cds(tenor, ScheduleBuilder::MAX_BOUNDARIES)?)
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

fn bump(map: &mut BTreeMap<String, usize>, key: String) {
    *map.entry(key).or_default() += 1;
}

struct Census {
    statuses: BTreeMap<String, usize>,
    frequencies: BTreeMap<String, usize>,
    period_counts: BTreeMap<String, usize>,
    lower_steps: BTreeMap<String, usize>,
    raw_fractions: usize,
    maximum_ulp: u64,
}

fn check_value(
    row: &Record<'_>,
    schedule: &Schedule,
    context: &ActualActualIcma,
    census: &mut Census,
) -> TestResult {
    let id = row.id();
    let events: u32 = row.parse("events")?;
    let hull = references(context);
    assert_eq!(hull, dates(row.text("references")?)?, "{id}");
    let periods: usize = row.parse("periods")?;
    assert_eq!(context.periods().len(), periods, "{id}");
    let provenance = schedule.old_cds_provenance().ok_or("provenance")?;
    let anchor = provenance.recurrence_anchor();
    let lower: usize = row.parse("lower_steps")?;
    assert_eq!(
        hull.iter().position(|&date| date == anchor),
        Some(lower),
        "{id}"
    );
    let tenor = schedule.tenor().ok_or("tenor")?;
    let stride = match tenor.unit() {
        TimeUnit::Years => 12 * i64::from(tenor.length()),
        _ => i64::from(tenor.length()),
    };
    bump(&mut census.period_counts, periods.to_string());
    bump(&mut census.lower_steps, format!("{stride}M:{lower}"));
    let raw = schedule.contractual_dates();
    let exact_values: Vec<&str> = row.text("raw_fractions")?.split(';').collect();
    let float_values: Vec<&str> = row.text("raw_floats")?.split(';').collect();
    assert_eq!(exact_values.len(), raw.len() - 1, "{id}");
    assert_eq!(float_values.len(), raw.len() - 1, "{id}");
    let mut queries: Vec<(Date, Date, &str, &str)> = raw
        .windows(2)
        .zip(exact_values.iter().zip(&float_values))
        .map(|(pair, (exact, float))| (pair[0], pair[1], *exact, *float))
        .collect();
    census.raw_fractions += queries.len();
    let span_exact = row.text("span_fraction")?;
    let span_float = row.text("span_float")?;
    let (first, last) = (
        *raw.first().ok_or("raw start")?,
        *raw.last().ok_or("raw end")?,
    );
    queries.push((first, last, span_exact, span_float));
    for (start, end, exact, float) in queries {
        // The fixture's rational is recomputed here from the oracle hull.
        assert_eq!(
            exact_fraction(&hull, events, start, end),
            rational(exact)?,
            "{id} {start}..{end}"
        );
        let expected: f64 = float.parse()?;
        let actual = context.year_fraction(start, end)?;
        let distance = ulp_distance(actual, expected).ok_or("nonfinite fraction")?;
        assert!(distance <= 4, "{id} {start}..{end}: {actual} vs {expected}");
        census.maximum_ulp = census.maximum_ulp.max(distance);
    }
    Ok(())
}

fn check_error(row: &Record<'_>, schedule: &Schedule, error: &ScheduleError) -> TestResult {
    let id = row.id();
    assert_eq!(error.kind(), row.text("status")?, "{id}");
    let limit: u32 = row.parse("limit")?;
    let frequency: Frequency = row.parse("frequency")?;
    match error {
        ScheduleError::IcmaPeriodLimitExceeded {
            maximum,
            required_at_least,
            ..
        } => {
            assert_eq!(*maximum, limit, "{id}");
            assert_eq!(*required_at_least, row.parse::<u32>("required")?, "{id}");
        }
        ScheduleError::Date {
            index,
            phase,
            cause,
            ..
        } => {
            assert_eq!(*index, row.parse::<usize>("error_index")?, "{id}");
            assert_eq!(*phase, "ICMA reference extension", "{id}");
            let DateError::OutOfRange {
                date,
                operation,
                count,
                ..
            } = cause
            else {
                return Err(format!("{id}: unexpected cause {cause:?}").into());
            };
            assert_eq!(*date, row.parse::<Date>("error_anchor")?, "{id}");
            assert_eq!(*operation, row.text("error_operation")?, "{id}");
            assert_eq!(*count, row.parse::<i64>("error_count")?, "{id}");
            // Required references step from the retained recurrence anchor.
            let provenance = schedule.old_cds_provenance().ok_or("provenance")?;
            assert_eq!(*date, provenance.recurrence_anchor(), "{id}");
        }
        ScheduleError::IcmaFrequencyMismatch {
            tenor,
            frequency: actual,
            expected_tenor,
            ..
        } => {
            assert_eq!(
                (tenor.length(), tenor.unit()),
                (row.parse("length")?, row.parse("unit")?),
                "{id}"
            );
            assert_eq!(*actual, frequency, "{id}");
            assert_eq!(Some(*expected_tenor), frequency.period(), "{id}");
        }
        ScheduleError::InvalidIcmaFrequency {
            frequency: actual, ..
        } => {
            assert_eq!(*actual, frequency, "{id}");
            assert_eq!(frequency.period(), None, "{id}");
        }
        ScheduleError::InvalidIcmaPeriodLimit {
            requested,
            minimum,
            maximum,
            ..
        } => {
            assert_eq!(
                (*requested, *minimum, *maximum),
                (
                    limit,
                    Schedule::MIN_ICMA_PERIODS,
                    Schedule::MAX_ICMA_PERIODS
                ),
                "{id}"
            );
        }
        other => return Err(format!("{id}: unexpected error {other:?}").into()),
    }
    Ok(())
}

/// FR-SCHED-38: every independent ICMA oracle row agrees exactly in hull,
/// period count, lower-step position, rational accrual, typed status and
/// civil cause; binary64 fractions stay within the existing 4-ULP profile.
#[test]
fn independent_icma_oracle_matches_hulls_fractions_caps_and_causes() -> TestResult {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/old-cds-icma-oracle.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 10_363);
    assert!(
        fixture
            .header("tolerance")
            .ok_or("tolerance")?
            .contains("TOL-E2 4 ULP")
    );
    let mut census = Census {
        statuses: BTreeMap::new(),
        frequencies: BTreeMap::new(),
        period_counts: BTreeMap::new(),
        lower_steps: BTreeMap::new(),
        raw_fractions: 0,
        maximum_ulp: 0,
    };
    for row in fixture.records() {
        let id = row.id();
        let tenor = Period::new(row.parse("length")?, row.parse("unit")?);
        let schedule = null_old_cds(row.parse("start")?, row.parse("end")?, tenor)?;
        assert_eq!(
            schedule.contractual_dates(),
            dates(row.text("raw_dates")?)?,
            "{id}"
        );
        let provenance = schedule.old_cds_provenance().ok_or("provenance")?;
        assert_eq!(
            provenance.initial_quarter(),
            row.parse::<Date>("initial_quarter")?
        );
        assert_eq!(
            provenance.recurrence_anchor(),
            row.parse::<Date>("recurrence_anchor")?
        );
        assert_eq!(
            provenance.resolved_end(),
            row.parse::<Date>("resolved_end")?
        );
        assert_eq!(
            provenance.omitted_quarter(),
            row.parse_optional::<Date>("omitted_quarter")?
        );
        let frequency: Frequency = row.parse("frequency")?;
        assert_eq!(
            frequency.events_per_year(),
            row.parse_optional::<u32>("events")?,
            "{id}"
        );
        bump(&mut census.statuses, row.text("status")?.to_owned());
        bump(&mut census.frequencies, row.text("frequency")?.to_owned());
        match schedule.build_icma_context(frequency, row.parse("limit")?) {
            Ok(context) => {
                assert_eq!(row.text("status")?, "value", "{id}");
                check_value(&row, &schedule, &context, &mut census)?;
            }
            Err(error) => check_error(&row, &schedule, &error)?,
        }
    }
    assert_eq!(census.statuses, header_counts(&fixture, "status-counts")?);
    assert_eq!(
        census.frequencies,
        header_counts(&fixture, "frequency-counts")?
    );
    assert_eq!(
        census.period_counts,
        header_counts(&fixture, "value-period-counts")?
    );
    assert_eq!(
        census.lower_steps,
        header_counts(&fixture, "value-lower-step-counts")?
    );
    assert_eq!(census.raw_fractions, 24_972);
    assert!(census.maximum_ulp <= 4);
    Ok(())
}

/// FR-SCHED-38: the whole extended hull is guarded. Monthly references stay
/// inside verified 2026 TARGET coverage; coarser frequencies need 2025
/// references, which strict TARGET rejects and explicit projection admits.
#[test]
fn extended_target_hull_is_validated_independently_of_the_raw_schedule() -> TestResult {
    let start = d("2026-02-19")?;
    let end = d("2026-06-01")?;
    for policy in [TargetPolicy::VerifiedOnly, TargetPolicy::AllowProjection] {
        for (length, frequency, needs_2025) in [
            (1, Frequency::Monthly, false),
            (2, Frequency::Bimonthly, true),
            (3, Frequency::Quarterly, true),
            (4, Frequency::EveryFourthMonth, true),
            (6, Frequency::Semiannual, true),
            (12, Frequency::Annual, true),
        ] {
            let schedule = ScheduleBuilder::new(
                start,
                end,
                ScheduleCalendar::Target(TargetCalendar::with_policy(policy)),
                Bdc::Following,
                Bdc::Following,
            )?
            .build_old_cds(Period::new(length, TimeUnit::Months), 2)?;
            assert_eq!(schedule.contractual_dates(), [start, d("2026-06-20")?]);
            let result = schedule.build_icma_context(frequency, 10);
            if needs_2025 && policy == TargetPolicy::VerifiedOnly {
                let Err(ScheduleError::Calendar {
                    index: None,
                    phase: "ICMA reference coverage",
                    cause,
                    ..
                }) = result
                else {
                    return Err(format!("{frequency}: expected coverage error").into());
                };
                assert_eq!(cause.kind(), "calendar.unverified_coverage");
            } else {
                let context = result?;
                assert_eq!(context.coverage_end(), d("2026-06-20")?);
                assert_eq!(context.coverage_start() < d("2026-01-01")?, needs_2025);
            }
        }
    }
    Ok(())
}

/// FR-SCHED-38: references depend only on raw dates, never on adjustment.
/// All 49 convention pairs yield equal contexts; 1Y and 12M schedules differ
/// as definitions while their normalized-frequency contexts compare equal.
#[test]
fn references_ignore_adjustment_and_normalize_stored_annual_aliases() -> TestResult {
    let start = d("2026-02-07")?;
    let end = d("2028-11-21")?;
    let baseline = null_old_cds(start, end, Period::new(3, TimeUnit::Months))?
        .build_icma_context(Frequency::Quarterly, 100)?;
    let mut checked = 0;
    for convention in CONVENTIONS {
        for termination in CONVENTIONS {
            let schedule = ScheduleBuilder::new(
                start,
                end,
                ScheduleCalendar::Plain(Calendar::weekends_only()),
                convention,
                termination,
            )?
            .build_old_cds(Period::new(3, TimeUnit::Months), 100)?;
            assert_eq!(
                schedule.build_icma_context(Frequency::Quarterly, 100)?,
                baseline
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 49);
    let years = null_old_cds(start, end, Period::new(1, TimeUnit::Years))?;
    let months = null_old_cds(start, end, Period::new(12, TimeUnit::Months))?;
    assert_ne!(years, months);
    assert_eq!(years.contractual_dates(), months.contractual_dates());
    assert_eq!(
        years.build_icma_context(Frequency::Annual, 10)?,
        months.build_icma_context(Frequency::Annual, 10)?
    );
    Ok(())
}

/// FR-SCHED-38 and FR-CF-02: generic raw-interval coupon composition uses the
/// supplied ICMA context. March 20 to June 20 spans three Monthly reference
/// periods, so the first fraction is exactly 1/4 and 1000 * 1/8 * 1/4 = 31.25.
#[test]
fn generic_fixed_leg_accrues_raw_intervals_with_the_supplied_context() -> TestResult {
    let schedule = null_old_cds(
        d("2026-03-20")?,
        d("2026-08-01")?,
        Period::new(1, TimeUnit::Months),
    )?;
    assert_eq!(
        schedule.contractual_dates(),
        [
            d("2026-03-20")?,
            d("2026-06-20")?,
            d("2026-07-20")?,
            d("2026-09-20")?
        ]
    );
    let context = schedule.build_icma_context(Frequency::Monthly, 10)?;
    let quote = FixedCouponRate::new(0.125, context.clone())?;
    let leg = Leg::fixed_rate(&schedule, 1000.0, &quote, 3)?;
    let raw = schedule.contractual_dates();
    let mut amounts = Vec::new();
    for (index, flow) in leg.flows().iter().enumerate() {
        let CashFlow::FixedRateCoupon(coupon) = flow else {
            return Err("fixed-rate factory returned another flow".into());
        };
        // Raw accrual interval, adjusted payment, and the supplied context.
        assert_eq!((coupon.start(), coupon.end()), (raw[index], raw[index + 1]));
        assert_eq!(coupon.payment_date(), schedule.adjusted_dates()[index + 1]);
        assert_eq!(
            coupon.accrual_fraction().to_bits(),
            context.year_fraction(raw[index], raw[index + 1])?.to_bits()
        );
        amounts.push(coupon.amount());
    }
    assert_eq!(amounts[0].to_bits(), 31.25_f64.to_bits());
    assert_eq!(amounts.len(), 3);
    Ok(())
}
