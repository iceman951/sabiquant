//! Independent civil-month and rational-overlap oracles for roll ICMA hulls.

mod support;

use sabiquant::calendar::{BusinessDayConvention, Calendar};
use sabiquant::schedule::{
    DateGenerationRule, ScheduleBuilder, ScheduleCalendar, ScheduleError, SpecialRollRule,
};
use sabiquant::time::{Date, DateError, Frequency, Period, TimeUnit};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::Fixture;
use support::float::ulp_distance;

fn month_coordinate(date: Date) -> i64 {
    i64::from(date.year() - 1) * 12 + i64::from(date.month() - 1)
}

fn month_days(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn ordinal(date: Date) -> i64 {
    let prior = i64::from(date.year() - 1);
    let elapsed_months: u32 = (1..date.month())
        .map(|month| month_days(date.year(), month))
        .sum();
    365 * prior + prior / 4 - prior / 100
        + prior / 400
        + i64::from(elapsed_months)
        + i64::from(date.day() - 1)
}

fn assert_roll_point(date: Date, rule: SpecialRollRule) -> Result<(), Box<dyn Error>> {
    match rule {
        SpecialRollRule::ThirdWednesday | SpecialRollRule::ThirdWednesdayInclusive => {
            assert_eq!(ordinal(date) % 7, 2);
            let mut preceding = 0;
            for day in 1..date.day() {
                if ordinal(Date::from_ymd(date.year(), date.month(), day)?) % 7 == 2 {
                    preceding += 1;
                }
            }
            assert_eq!(preceding, 2);
        }
        SpecialRollRule::Twentieth => assert_eq!(date.day(), 20),
        SpecialRollRule::TwentiethIMM => {
            assert_eq!(date.day(), 20);
            assert!(date.month().is_multiple_of(3));
        }
    }
    Ok(())
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

// Independent Gregorian ordinal overlaps and reduced integer fractions. This
// shares neither Date differences nor floating division/summation with ICMA.
fn exact_fraction(
    dates: &[Date],
    events: u32,
    start: Date,
    end: Date,
) -> Result<(i128, i128), Box<dyn Error>> {
    let low = start.min(end);
    let high = start.max(end);
    let (mut n, mut d) = (0_i128, 1_i128);
    for pair in dates.windows(2) {
        let a = low.max(pair[0]);
        let b = high.min(pair[1]);
        if a >= b {
            continue;
        }
        let mut tn = i128::from(ordinal(b) - ordinal(a));
        let mut td = i128::from(events)
            .checked_mul(i128::from(ordinal(pair[1]) - ordinal(pair[0])))
            .ok_or("term denominator overflow")?;
        let reduce = gcd(tn, td);
        tn /= reduce;
        td /= reduce;
        let common = gcd(d, td);
        let combined = n
            .checked_mul(td / common)
            .and_then(|left| {
                tn.checked_mul(d / common)
                    .and_then(|right| left.checked_add(right))
            })
            .ok_or("numerator overflow")?;
        let reduce = gcd(combined, common);
        n = combined / reduce;
        d = (d / common)
            .checked_mul(td / reduce)
            .ok_or("denominator overflow")?;
    }
    Ok((
        if start > end {
            n.checked_neg().ok_or("negation overflow")?
        } else {
            n
        },
        d,
    ))
}

fn stored_tenor(actual: Period, expected: Period) {
    assert_eq!(actual.length(), expected.length());
    assert_eq!(actual.unit(), expected.unit());
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "records measured independent oracle evidence"
)]
fn civil_cycle_minimal_roll_hulls_caps_provenance_errors_and_exact_fraction_queries()
-> Result<(), Box<dyn Error>> {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/roll-icma-oracle.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 42_647);
    assert_eq!(
        fixture.header("reference"),
        Some("oracle:Python datetime/calendar/Fraction")
    );
    assert!(
        fixture
            .header("reference-source")
            .unwrap()
            .contains("full civil-month envelope enumeration")
    );
    assert!(
        fixture
            .header("tolerance")
            .unwrap()
            .contains("TOL-E2 4 ULP")
    );
    let mut statuses = BTreeMap::<String, usize>::new();
    let mut cycle_months = BTreeSet::new();
    let mut frequencies = BTreeSet::new();
    let mut annual_stored = BTreeSet::new();
    let mut maximum_ulp = 0;
    let mut periods = 0;
    let mut queries = 0;
    let mut zero = 0;
    let mut reversed = 0;
    let mut annual_aliases = 0;
    for row in fixture.records() {
        let start: Date = row.parse("start")?;
        let end: Date = row.parse("end")?;
        let rule: SpecialRollRule = row.parse("rule")?;
        let tenor = Period::new(row.parse("length")?, row.parse("unit")?);
        let frequency: Frequency = row.parse("frequency")?;
        let anchor: Date = row.parse("anchor")?;
        let raw_start: Date = row.parse("resolved_start")?;
        let raw_end: Date = row.parse("resolved_end")?;
        let limit: u32 = row.parse("limit")?;
        let status = row.text("status")?;
        *statuses.entry(status.to_owned()).or_default() += 1;
        let builder = ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(Calendar::null()),
            BusinessDayConvention::Unadjusted,
            BusinessDayConvention::Unadjusted,
        )?;
        let schedule = builder
            .build_roll(tenor, rule, 32)
            .unwrap_or_else(|error| panic!("{} source schedule: {error:?}", row.id()));
        let before = schedule.clone();
        let provenance = schedule
            .roll_provenance()
            .ok_or("missing roll provenance")?;
        assert_eq!(schedule.generation_rule(), DateGenerationRule::Roll(rule));
        assert_eq!(
            (provenance.requested_start(), provenance.requested_end()),
            (start, end)
        );
        assert_eq!(
            (provenance.resolved_start(), provenance.resolved_end()),
            (raw_start, raw_end)
        );
        assert_eq!(
            (provenance.anchor_year(), provenance.anchor_month().number()),
            (anchor.year(), anchor.month())
        );
        assert_eq!(provenance.rule(), rule);
        stored_tenor(provenance.tenor(), tenor);
        stored_tenor(schedule.tenor().ok_or("missing tenor")?, tenor);
        assert_eq!(schedule.contractual_dates().first(), Some(&raw_start));
        assert_eq!(schedule.contractual_dates().last(), Some(&raw_end));
        assert_eq!(schedule.contractual_dates(), schedule.adjusted_dates());
        assert_eq!(schedule.lattice_anchor(), None);
        assert_eq!(schedule.end_of_month(), None);
        assert_eq!(schedule.stub_policy(), None);
        assert_eq!(schedule.explicit_stub_dates(), None);
        assert_roll_point(anchor, rule)?;
        // Cap validation wins for every value and error domain in the fixture.
        assert!(matches!(
            schedule.build_icma_context(frequency, 0),
            Err(ScheduleError::InvalidIcmaPeriodLimit { requested: 0, .. })
        ));
        let result = schedule.build_icma_context(frequency, limit);
        assert_eq!(schedule, before);
        match status {
            "schedule.invalid_icma_frequency" => {
                assert!(matches!(
                    result,
                    Err(ScheduleError::InvalidIcmaFrequency {
                        frequency: Frequency::Once,
                        ..
                    })
                ));
                assert!(row.text("references")?.is_empty());
                continue;
            }
            "schedule.icma_frequency_mismatch" => {
                let Err(ScheduleError::IcmaFrequencyMismatch {
                    tenor: actual_tenor,
                    frequency: actual_frequency,
                    expected_tenor,
                    ..
                }) = result
                else {
                    panic!("{} wrong mismatch {result:?}", row.id());
                };
                stored_tenor(actual_tenor, tenor);
                assert_eq!(actual_frequency, frequency);
                assert_eq!(Some(expected_tenor), frequency.period());
                assert!(row.text("references")?.is_empty());
                continue;
            }
            "schedule.date" => {
                let error = result.unwrap_err();
                assert_eq!(error.kind(), status);
                assert!(error.source().is_some());
                let ScheduleError::Date {
                    index,
                    phase,
                    cause,
                    ..
                } = error
                else {
                    panic!("{} wrong date cause {error:?}", row.id());
                };
                assert_eq!(index, row.parse::<usize>("error_index")?, "{}", row.id());
                assert_eq!(phase, "ICMA reference extension");
                let DateError::OutOfRange {
                    date,
                    operation,
                    count,
                    ..
                } = cause
                else {
                    panic!("{} wrong civil cause {cause:?}", row.id());
                };
                assert_eq!(date, anchor);
                assert_eq!(operation, "add months");
                assert_eq!(count, row.parse::<i64>("error_offset")?);
                assert!(row.text("references")?.is_empty());
                continue;
            }
            "value" => {}
            other => panic!("unknown oracle status {other}"),
        }
        let context = result.unwrap_or_else(|error| panic!("{} ICMA: {error:?}", row.id()));
        let references: Vec<Date> = row
            .text("references")?
            .split(';')
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        assert_eq!(references.len() - 1, usize::try_from(limit)?);
        assert_eq!(context.periods().len(), usize::try_from(limit)?);
        let stride = i64::from(tenor.length())
            * if tenor.unit() == TimeUnit::Years {
                12
            } else {
                1
            };
        for point in &references {
            assert_roll_point(*point, rule)?;
            assert_eq!(
                (month_coordinate(*point) - month_coordinate(anchor)) % stride,
                0
            );
        }
        for (actual, expected) in context.periods().iter().zip(references.windows(2)) {
            assert_eq!(
                (actual.start(), actual.end()),
                (expected[0], expected[1]),
                "{}",
                row.id()
            );
            assert_eq!(
                month_coordinate(expected[1]) - month_coordinate(expected[0]),
                stride
            );
            assert!(expected[0] < expected[1]);
        }
        // Exact endpoint bracketing plus adjacent positive stride proves that
        // deleting either exterior period would lose source-range coverage.
        assert!(references[0] <= raw_start && references[1] > raw_start);
        assert!(
            references[references.len() - 1] >= raw_end
                && references[references.len() - 2] < raw_end
        );
        assert_eq!(context.coverage_start(), references[0]);
        assert_eq!(context.coverage_end(), references[references.len() - 1]);
        assert_eq!(context.frequency(), frequency);
        assert_eq!(context, schedule.build_icma_context(frequency, limit + 1)?);
        if limit > 1 {
            assert!(
                matches!(schedule.build_icma_context(frequency, limit-1),
                Err(ScheduleError::IcmaPeriodLimitExceeded { maximum, required_at_least, .. })
                if maximum == limit-1 && required_at_least == limit),
                "{} cap",
                row.id()
            );
        }
        if frequency == Frequency::Annual {
            annual_stored.insert((tenor.length(), tenor.unit().to_string()));
            let alias = if tenor.unit() == TimeUnit::Years {
                Period::new(12, TimeUnit::Months)
            } else {
                Period::new(1, TimeUnit::Years)
            };
            let aliased = builder.build_roll(alias, rule, 32)?;
            assert_ne!(schedule, aliased);
            assert_ne!(schedule.roll_provenance(), aliased.roll_provenance());
            assert_eq!(context, aliased.build_icma_context(frequency, limit)?);
            annual_aliases += 1;
        }
        let events: u32 = row.parse("events")?;
        assert_eq!(frequency.events_per_year(), Some(events));
        let mut row_queries = 0;
        for query in row.text("queries")?.split(';') {
            let fields: Vec<&str> = query.split(',').collect();
            assert_eq!(fields.len(), 5);
            let first: Date = fields[0].parse()?;
            let last: Date = fields[1].parse()?;
            let expected_exact = (fields[2].parse::<i128>()?, fields[3].parse::<i128>()?);
            assert_eq!(
                exact_fraction(&references, events, first, last)?,
                expected_exact,
                "{} rational",
                row.id()
            );
            let expected: f64 = fields[4].parse()?;
            let actual = context.year_fraction(first, last)?;
            assert!(actual.is_finite());
            let distance = ulp_distance(actual, expected).ok_or("nonfinite fraction")?;
            assert!(
                distance <= 4,
                "{} {first}..{last}: actual {actual} expected {expected} ULP {distance}",
                row.id()
            );
            maximum_ulp = maximum_ulp.max(distance);
            if first == last {
                assert_eq!(actual.to_bits(), 0.0_f64.to_bits());
                zero += 1;
            }
            if first > last {
                reversed += 1;
            }
            queries += 1;
            row_queries += 1;
        }
        assert_eq!(row_queries, 3);
        periods += context.periods().len();
        frequencies.insert(frequency.to_string());
        if row.text("profile")?.starts_with("cycle-") {
            assert!((2000..2400).contains(&start.year()));
            cycle_months.insert((start.year(), start.month(), rule.to_string()));
        }
    }
    assert_eq!(
        statuses,
        BTreeMap::from([
            ("value".to_owned(), 42_179),
            ("schedule.date".to_owned(), 260),
            ("schedule.icma_frequency_mismatch".to_owned(), 188),
            ("schedule.invalid_icma_frequency".to_owned(), 20),
        ])
    );
    assert_eq!(cycle_months.len(), 19_200);
    assert_eq!(frequencies.len(), 6);
    assert_eq!(
        annual_stored,
        BTreeSet::from([(1, "Years".to_owned()), (12, "Months".to_owned())])
    );
    assert_eq!(queries, 126_537);
    assert_eq!(
        (periods, zero, reversed, annual_aliases),
        (87_558, 10_548, 42_179, 14_155)
    );
    println!(
        "roll ICMA oracle: 42,647 contexts; {periods} periods; {queries} exact Fraction queries; {zero} zero; {reversed} reversed; {annual_aliases} annual aliases; maximum {maximum_ulp} ULP"
    );
    Ok(())
}
