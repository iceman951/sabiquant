//! Exact independent civil-enumeration oracle for the four generic roll rules.

#![expect(
    clippy::panic,
    reason = "unexpected fixture status is malformed evidence"
)]

mod support;

use sabiquant::calendar::{BusinessDayConvention, Calendar};
use sabiquant::schedule::{
    DateGenerationRule, PeriodRegularity, ScheduleBuilder, ScheduleCalendar, ScheduleError,
    SpecialRollRule,
};
use sabiquant::time::{Date, DateError, Period, TimeUnit};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::Fixture;

fn month_coordinate(date: Date) -> i64 {
    i64::from(date.year() - 1) * 12 + i64::from(date.month() - 1)
}

fn civil_ordinal(year: i32, month: u32, day: u32) -> i64 {
    let prior = i64::from(year - 1);
    let mut value = 365 * prior + prior / 4 - prior / 100 + prior / 400;
    for before in 1..month {
        value += i64::from(month_days(year, before));
    }
    value + i64::from(day - 1)
}

fn month_days(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn roll_day(year: i32, month: u32, rule: SpecialRollRule) -> u32 {
    match rule {
        SpecialRollRule::ThirdWednesday | SpecialRollRule::ThirdWednesdayInclusive => {
            let mut count = 0;
            for day in 1..=month_days(year, month) {
                // Gregorian 0001-01-01 is Monday. This civil ordinal uses no
                // production Date weekday/date-difference or roll arithmetic.
                if civil_ordinal(year, month, day) % 7 == 2 {
                    count += 1;
                    if count == 3 {
                        return day;
                    }
                }
            }
            panic!("a Gregorian month must contain three Wednesdays");
        }
        SpecialRollRule::Twentieth | SpecialRollRule::TwentiethIMM => 20,
    }
}

fn membership(date: Date, anchor_month: i64, stride: i64, rule: SpecialRollRule) -> Option<i64> {
    let distance = month_coordinate(date) - anchor_month;
    (distance >= 0
        && distance % stride == 0
        && (rule != SpecialRollRule::TwentiethIMM || date.month().is_multiple_of(3))
        && date.day() == roll_day(date.year(), date.month(), rule))
    .then_some(distance / stride)
}

fn stored_tenor(actual: Period, expected: Period) {
    assert_eq!(actual.length(), expected.length());
    assert_eq!(actual.unit(), expected.unit());
}

#[test]
#[expect(clippy::print_stdout, reason = "reports exact oracle distributions")]
fn independent_roll_oracle_checks_civil_cycle_projections_flags_provenance_and_caps()
-> Result<(), Box<dyn Error>> {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/roll-oracle.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 122_233);
    assert_eq!(
        fixture.header("reference"),
        Some("oracle:Python datetime/calendar")
    );
    assert!(
        fixture
            .header("tolerance")
            .unwrap()
            .starts_with("TOL-E0 exact")
    );
    assert!(
        fixture
            .header("reference-source")
            .unwrap()
            .contains("in-span congruence membership")
    );
    let mut statuses = BTreeMap::<String, usize>::new();
    let mut profiles = BTreeMap::<String, usize>::new();
    let mut rules = BTreeMap::<String, usize>::new();
    let mut flags = BTreeMap::<String, usize>::new();
    let mut wednesday_months = BTreeSet::<(i32, u32)>::new();
    let mut tenor_samples = BTreeSet::<(i32, String)>::new();
    let mut boundaries = 0;
    let mut projected_starts = 0;
    let mut projected_ends = 0;
    let mut caps = 0;
    for row in fixture.records() {
        let start: Date = row.parse("start")?;
        let end: Date = row.parse("end")?;
        let tenor = Period::new(row.parse("length")?, row.parse("unit")?);
        let rule: SpecialRollRule = row.parse("rule")?;
        let maximum: u32 = row.parse("limit")?;
        let status = row.text("status")?;
        let profile = row.text("profile")?;
        *statuses.entry(status.to_owned()).or_default() += 1;
        *profiles.entry(profile.to_owned()).or_default() += 1;
        *rules.entry(row.text("rule")?.to_owned()).or_default() += 1;
        tenor_samples.insert((tenor.length(), row.text("unit")?.to_owned()));
        let builder = ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(Calendar::null()),
            BusinessDayConvention::Unadjusted,
            BusinessDayConvention::Unadjusted,
        )?;
        let result = builder.build_roll(tenor, rule, maximum);
        match status {
            "schedule.invalid_roll_tenor" => {
                let error = result.unwrap_err();
                assert_eq!(error.kind(), status);
                let ScheduleError::InvalidRollTenor {
                    rule: actual_rule,
                    tenor: actual_tenor,
                    ..
                } = error
                else {
                    panic!("{}: wrong typed error {error:?}", row.id());
                };
                assert_eq!(actual_rule, rule);
                stored_tenor(actual_tenor, tenor);
                assert!(row.text("dates")?.is_empty());
                continue;
            }
            "schedule.resolved_roll_range" => {
                let error = result.unwrap_err();
                assert_eq!(error.kind(), status);
                let ScheduleError::ResolvedRollRange {
                    rule: actual_rule,
                    tenor: actual_tenor,
                    requested_start,
                    requested_end,
                    resolved_start,
                    resolved_end,
                    ..
                } = error
                else {
                    panic!("{}: wrong typed error {error:?}", row.id());
                };
                assert_eq!(actual_rule, rule);
                stored_tenor(actual_tenor, tenor);
                assert_eq!((requested_start, requested_end), (start, end));
                assert_eq!(resolved_start, row.parse::<Date>("resolved_start")?);
                assert_eq!(resolved_end, row.parse::<Date>("resolved_end")?);
                assert!(resolved_start >= resolved_end);
                assert!(row.text("dates")?.is_empty());
                continue;
            }
            "schedule.roll_date" => {
                let error = result.unwrap_err();
                assert_eq!(error.kind(), status);
                assert!(error.source().is_some());
                let ScheduleError::RollDate {
                    rule: actual_rule,
                    tenor: actual_tenor,
                    role,
                    input,
                    cause,
                    ..
                } = error
                else {
                    panic!("{}: wrong typed error {error:?}", row.id());
                };
                assert_eq!(actual_rule, rule);
                stored_tenor(actual_tenor, tenor);
                assert_eq!(role, row.text("error_role")?);
                assert_eq!(input, row.parse::<Date>("error_input")?);
                let DateError::OutOfRange {
                    date,
                    operation,
                    count,
                    ..
                } = cause
                else {
                    panic!(
                        "{}: expected checked civil overflow, got {cause:?}",
                        row.id()
                    );
                };
                assert_eq!(date, input);
                assert_eq!(operation, "add months");
                assert_eq!(
                    count,
                    if rule == SpecialRollRule::TwentiethIMM {
                        3
                    } else {
                        1
                    }
                );
                assert!(row.text("dates")?.is_empty());
                continue;
            }
            "value" => {}
            other => panic!("unknown oracle status {other}"),
        }
        let schedule = result.unwrap_or_else(|error| panic!("{}: {error:?}", row.id()));
        let dates = row
            .text("dates")?
            .split(';')
            .map(str::parse::<Date>)
            .collect::<Result<Vec<_>, _>>()?;
        let expected_flags = row
            .text("regularity")?
            .split(';')
            .map(str::parse::<PeriodRegularity>)
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(schedule.contractual_dates(), dates, "{} raw", row.id());
        assert_eq!(schedule.adjusted_dates(), dates, "{} adjusted", row.id());
        assert_eq!(schedule.regularity(), expected_flags, "{} flags", row.id());
        assert_eq!(dates.len(), usize::try_from(maximum)?);
        assert_eq!(schedule.generation_rule(), DateGenerationRule::Roll(rule));
        stored_tenor(schedule.tenor().unwrap(), tenor);
        assert_eq!(schedule.end_of_month(), None);
        assert_eq!(schedule.stub_policy(), None);
        assert_eq!(schedule.explicit_stub_dates(), None);
        assert_eq!(schedule.lattice_anchor(), None);
        assert_eq!(schedule.convention(), BusinessDayConvention::Unadjusted);
        assert_eq!(
            schedule.termination_convention(),
            BusinessDayConvention::Unadjusted
        );
        assert_eq!(schedule.calendar(), builder.calendar());
        let provenance = schedule.roll_provenance().unwrap();
        assert_eq!(provenance.requested_start(), start);
        assert_eq!(provenance.requested_end(), end);
        assert_eq!(
            provenance.resolved_start(),
            row.parse::<Date>("resolved_start")?
        );
        assert_eq!(
            provenance.resolved_end(),
            row.parse::<Date>("resolved_end")?
        );
        assert_eq!(provenance.resolved_start(), dates[0]);
        assert_eq!(provenance.resolved_end(), dates[dates.len() - 1]);
        assert_eq!(provenance.anchor_year(), row.parse::<i32>("anchor_year")?);
        assert_eq!(
            provenance.anchor_month().number(),
            row.parse::<u32>("anchor_month")?
        );
        assert_eq!(provenance.rule(), rule);
        stored_tenor(provenance.tenor(), tenor);
        let anchor = 12 * i64::from(provenance.anchor_year() - 1)
            + i64::from(provenance.anchor_month().number() - 1);
        let stride = i64::from(tenor.length())
            * if tenor.unit() == TimeUnit::Years {
                12
            } else {
                1
            };
        for ((left, right), actual_flag) in dates
            .windows(2)
            .map(|pair| (pair[0], pair[1]))
            .zip(schedule.regularity())
        {
            assert!(left < right);
            let a = membership(left, anchor, stride, rule);
            let b = membership(right, anchor, stride, rule);
            let expected = if a.zip(b).is_some_and(|(a, b)| b == a + 1) {
                PeriodRegularity::Regular
            } else {
                PeriodRegularity::Stub
            };
            assert_eq!(*actual_flag, expected, "{} independent flag", row.id());
            *flags.entry(actual_flag.to_string()).or_default() += 1;
        }
        if profile == "cycle-on-roll" && rule == SpecialRollRule::ThirdWednesday {
            assert_eq!(dates[0].day(), roll_day(start.year(), start.month(), rule));
            wednesday_months.insert((start.year(), start.month()));
        }
        projected_starts += usize::from(dates[0] != start);
        projected_ends += usize::from(dates[dates.len() - 1] != end);
        boundaries += dates.len();
        assert_eq!(schedule, builder.build_roll(tenor, rule, maximum + 1)?);
        let smaller = builder.build_roll(tenor, rule, maximum - 1).unwrap_err();
        if maximum == 2 {
            assert!(matches!(
                smaller,
                ScheduleError::InvalidBoundaryLimit {
                    requested: 1,
                    minimum: 2,
                    ..
                }
            ));
        } else {
            assert!(
                matches!(smaller,ScheduleError::BoundaryLimitExceeded {maximum:actual,required_at_least,..} if actual==maximum-1 && required_at_least==maximum),
                "{} cap {smaller:?}",
                row.id()
            );
        }
        caps += 1;
    }
    assert_eq!(
        statuses,
        BTreeMap::from([
            ("schedule.invalid_roll_tenor".to_owned(), 78),
            ("schedule.resolved_roll_range".to_owned(), 5_290),
            ("schedule.roll_date".to_owned(), 54),
            ("value".to_owned(), 116_811),
        ])
    );
    assert_eq!(
        profiles,
        BTreeMap::from([
            ("century-extreme".to_owned(), 6_897),
            ("cycle-after-before".to_owned(), 19_200),
            ("cycle-first-last".to_owned(), 19_200),
            ("cycle-last-first".to_owned(), 19_200),
            ("cycle-leading-exact".to_owned(), 19_200),
            ("cycle-on-roll".to_owned(), 19_200),
            ("cycle-same-month".to_owned(), 19_200),
            ("huge-stored-tenor".to_owned(), 24),
            ("required-endpoint-extreme".to_owned(), 16),
            ("tenor-domain".to_owned(), 96),
        ])
    );
    assert_eq!(
        rules,
        BTreeMap::from([
            ("ThirdWednesday".to_owned(), 30_715),
            ("ThirdWednesdayInclusive".to_owned(), 30_715),
            ("Twentieth".to_owned(), 30_715),
            ("TwentiethIMM".to_owned(), 30_088),
        ])
    );
    assert_eq!(
        flags,
        BTreeMap::from([
            ("Regular".to_owned(), 114_065),
            ("Stub".to_owned(), 123_795)
        ])
    );
    assert_eq!(boundaries, 354_671);
    assert_eq!(caps, 116_811);
    assert_eq!(wednesday_months.len(), 4_800);
    for year in 2000..2400 {
        for month in 1..=12 {
            assert!(wednesday_months.contains(&(year, month)));
        }
    }
    for sample in [
        (1, "Months"),
        (2, "Months"),
        (3, "Months"),
        (4, "Months"),
        (6, "Months"),
        (12, "Months"),
        (18, "Months"),
        (1, "Years"),
        (2, "Years"),
    ] {
        assert!(tenor_samples.contains(&(sample.0, sample.1.to_owned())));
    }
    println!(
        "roll-oracle statuses={statuses:?} profiles={profiles:?} rules={rules:?} flags={flags:?} boundaries={boundaries} exactcaps={caps} projected_starts={projected_starts} projected_ends={projected_ends} Wednesday_months={}",
        wednesday_months.len()
    );
    Ok(())
}
