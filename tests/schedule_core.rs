//! Independent civil-date schedules, adjustment guards and immutable context.

mod support;

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, EndOfMonth, JointRule, TargetCalendar,
    TargetPolicy,
};
use sabiquant::schedule::{
    DateGenerationRule as Rule, PeriodRegularity as Regularity, Schedule, ScheduleBuilder,
    ScheduleCalendar, ScheduleError, StubPolicy,
};
use sabiquant::time::{Date, Period, TimeUnit, Weekday};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use support::fixture::Fixture;

#[expect(
    clippy::expect_used,
    reason = "hand-audited literal test dates are valid ISO calendar dates"
)]
fn date(text: &str) -> Date {
    text.parse().expect("valid literal date")
}

fn builder(start: Date, end: Date) -> Result<ScheduleBuilder, ScheduleError> {
    ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )
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

fn hash<T: Hash>(value: &T) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

#[test]
fn independent_civil_span_oracle_matches_every_date_and_raw_interval_flag()
-> Result<(), Box<dyn Error>> {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/oracles.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 18_916);
    assert_eq!(
        fixture.header("reference"),
        Some("oracle:Python datetime/calendar")
    );
    assert_eq!(
        fixture.header("tolerance"),
        Some("TOL-E0 exact civil dates and raw interval regularity")
    );
    assert!(
        fixture
            .header("reproduction")
            .unwrap()
            .contains("generator-sha256=")
    );
    for row in fixture.records() {
        let start = row.parse("start")?;
        let end = row.parse("end")?;
        let tenor = Period::new(row.parse("length")?, row.parse("unit")?);
        let rule = row.parse("rule")?;
        let stub = row.parse("stub")?;
        let eom = row.parse("eom")?;
        let dates: Vec<Date> = row
            .text("dates")?
            .split(';')
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        let flags: Vec<Regularity> = row
            .text("regularity")?
            .split(';')
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        let schedule = builder(start, end)?
            .build_recurrent(tenor, rule, stub, eom, row.parse("limit")?)
            .unwrap_or_else(|error| panic!("{}: {error}", row.id()));
        assert_eq!(
            schedule.contractual_dates(),
            dates,
            "{} raw dates",
            row.id()
        );
        assert_eq!(
            schedule.adjusted_dates(),
            dates,
            "{} adjusted dates",
            row.id()
        );
        assert_eq!(schedule.regularity(), flags, "{} raw flags", row.id());
        assert_eq!(schedule.generation_rule(), rule);
        assert_eq!(schedule.tenor(), Some(tenor));
        assert_eq!(schedule.stub_policy(), Some(stub));
        assert_eq!(schedule.end_of_month(), Some(eom));
    }
    Ok(())
}

#[test]
fn anchored_monthly_generation_covers_all_month_ends_and_nearby_days_over_400_years()
-> Result<(), Box<dyn Error>> {
    let mut checked = 0;
    for year in 1600..2000 {
        for month in 1..=12 {
            let last = month_days(year, month);
            for day in 28..=31 {
                if day > last {
                    continue;
                }
                let seed = Date::from_ymd(year, month, day)?;
                for rule in [Rule::Forward, Rule::Backward] {
                    for eom in EndOfMonth::ALL {
                        let mut dates = vec![seed];
                        let mut civil_year = year;
                        let mut civil_month = month;
                        for _ in 0..4 {
                            match rule {
                                Rule::Forward => {
                                    if civil_month == 12 {
                                        civil_month = 1;
                                        civil_year += 1;
                                    } else {
                                        civil_month += 1;
                                    }
                                }
                                _ => {
                                    if civil_month == 1 {
                                        civil_month = 12;
                                        civil_year -= 1;
                                    } else {
                                        civil_month -= 1;
                                    }
                                }
                            }
                            let target_last = month_days(civil_year, civil_month);
                            let target_day = if eom == EndOfMonth::Preserve && day == last {
                                target_last
                            } else {
                                day.min(target_last)
                            };
                            dates.push(Date::from_ymd(civil_year, civil_month, target_day)?);
                        }
                        if rule == Rule::Backward {
                            dates.reverse();
                        }
                        let schedule = builder(dates[0], dates[4])?.build_recurrent(
                            Period::new(1, TimeUnit::Months),
                            rule,
                            StubPolicy::None,
                            eom,
                            5,
                        )?;
                        assert_eq!(schedule.contractual_dates(), dates);
                        assert_eq!(schedule.adjusted_dates(), dates);
                        assert_eq!(schedule.regularity(), [Regularity::Regular; 4]);
                        checked += 1;
                    }
                }
            }
        }
    }
    assert_eq!(checked, 65_988);
    Ok(())
}

#[test]
fn initial_and_final_stubs_have_explicit_short_long_and_no_stub_metadata()
-> Result<(), Box<dyn Error>> {
    for (start, end, rule, short, long, short_flags, long_flags) in [
        (
            "2024-01-01",
            "2024-04-15",
            Rule::Forward,
            vec![
                "2024-01-01",
                "2024-02-01",
                "2024-03-01",
                "2024-04-01",
                "2024-04-15",
            ],
            vec!["2024-01-01", "2024-02-01", "2024-03-01", "2024-04-15"],
            vec![
                Regularity::Regular,
                Regularity::Regular,
                Regularity::Regular,
                Regularity::Stub,
            ],
            vec![Regularity::Regular, Regularity::Regular, Regularity::Stub],
        ),
        (
            "2024-01-15",
            "2024-05-01",
            Rule::Backward,
            vec![
                "2024-01-15",
                "2024-02-01",
                "2024-03-01",
                "2024-04-01",
                "2024-05-01",
            ],
            vec!["2024-01-15", "2024-03-01", "2024-04-01", "2024-05-01"],
            vec![
                Regularity::Stub,
                Regularity::Regular,
                Regularity::Regular,
                Regularity::Regular,
            ],
            vec![Regularity::Stub, Regularity::Regular, Regularity::Regular],
        ),
    ] {
        let b = builder(date(start), date(end))?;
        let tenor = Period::new(1, TimeUnit::Months);
        for (policy, expected, flags) in [
            (StubPolicy::Short, short, short_flags),
            (StubPolicy::Long, long, long_flags),
        ] {
            let s = b.build_recurrent(tenor, rule, policy, EndOfMonth::Ignore, 5)?;
            assert_eq!(
                s.contractual_dates(),
                expected.into_iter().map(date).collect::<Vec<_>>()
            );
            assert_eq!(s.regularity(), flags);
        }
        assert!(matches!(
            b.build_recurrent(tenor, rule, StubPolicy::None, EndOfMonth::Ignore, 5),
            Err(ScheduleError::StubRequired { .. })
        ));
        assert!(matches!(
            b.build_recurrent(tenor, rule, StubPolicy::Short, EndOfMonth::Ignore, 4),
            Err(ScheduleError::BoundaryLimitExceeded {
                maximum: 4,
                required_at_least: 5,
                ..
            })
        ));
        assert_eq!(
            b.build_recurrent(tenor, rule, StubPolicy::Long, EndOfMonth::Ignore, 4)?
                .contractual_dates()
                .len(),
            4
        );
    }
    for rule in [Rule::Forward, Rule::Backward] {
        let b = builder(date("2024-01-01"), date("2024-01-15"))?;
        assert_eq!(
            b.build_recurrent(
                Period::new(1, TimeUnit::Months),
                rule,
                StubPolicy::Short,
                EndOfMonth::Ignore,
                2
            )?
            .regularity(),
            [Regularity::Stub]
        );
        assert!(matches!(
            b.build_recurrent(
                Period::new(1, TimeUnit::Months),
                rule,
                StubPolicy::Long,
                EndOfMonth::Ignore,
                2
            ),
            Err(ScheduleError::LongStubWithoutRegularPeriod { .. })
        ));
        for stub in [StubPolicy::None, StubPolicy::Short, StubPolicy::Long] {
            assert_eq!(
                builder(date("2024-01-01"), date("2024-04-01"))?
                    .build_recurrent(
                        Period::new(1, TimeUnit::Months),
                        rule,
                        stub,
                        EndOfMonth::Ignore,
                        4
                    )?
                    .regularity(),
                [Regularity::Regular; 3]
            );
        }
    }
    Ok(())
}

#[test]
fn limits_and_unused_huge_out_of_range_candidates_do_not_corrupt_short_schedules()
-> Result<(), Box<dyn Error>> {
    for rule in [Rule::Forward, Rule::Backward] {
        for (start, end) in [
            (Date::MIN, Date::MIN.checked_add_days(7)?),
            (Date::MAX.checked_sub_days(7)?, Date::MAX),
        ] {
            let b = builder(start, end)?;
            for unit in TimeUnit::ALL {
                let p = Period::new(i32::MAX, unit);
                let s = b.build_recurrent(p, rule, StubPolicy::Short, EndOfMonth::Ignore, 2)?;
                assert_eq!(s.contractual_dates(), [start, end]);
                assert_eq!(s.regularity(), [Regularity::Stub]);
                assert!(matches!(
                    b.build_recurrent(p, rule, StubPolicy::None, EndOfMonth::Ignore, 2),
                    Err(ScheduleError::StubRequired { .. })
                ));
            }
        }
    }
    let b = builder(date("2024-01-01"), date("2024-01-05"))?;
    for limit in [5, 6] {
        assert_eq!(
            b.build_recurrent(
                Period::new(1, TimeUnit::Days),
                Rule::Forward,
                StubPolicy::None,
                EndOfMonth::Ignore,
                limit
            )?
            .contractual_dates()
            .len(),
            5
        );
    }
    assert!(matches!(
        b.build_recurrent(
            Period::new(1, TimeUnit::Days),
            Rule::Forward,
            StubPolicy::None,
            EndOfMonth::Ignore,
            4
        ),
        Err(ScheduleError::BoundaryLimitExceeded {
            maximum: 4,
            required_at_least: 5,
            ..
        })
    ));
    for limit in [0, 1, 3_652_060, u32::MAX] {
        assert!(matches!(
            b.build_zero(limit),
            Err(ScheduleError::InvalidBoundaryLimit { .. })
        ));
        assert!(matches!(
            b.build_recurrent(
                Period::new(0, TimeUnit::Days),
                Rule::Forward,
                StubPolicy::Short,
                EndOfMonth::Preserve,
                limit
            ),
            Err(ScheduleError::InvalidBoundaryLimit { .. })
        ));
    }
    assert_eq!(b.build_zero(3_652_059)?.contractual_dates().len(), 2);
    Ok(())
}

#[test]
fn zero_metadata_and_recurrent_option_errors_are_explicit() -> Result<(), Box<dyn Error>> {
    let start = date("2026-01-01");
    let end = date("2026-03-01");
    for (a, b) in [(start, start), (end, start)] {
        assert!(matches!(
            builder(a, b),
            Err(ScheduleError::InvalidRange { .. })
        ));
    }
    let b = builder(start, end)?;
    let zero = b.build_zero(2)?;
    assert_eq!(zero.contractual_dates(), [start, end]);
    assert_eq!(zero.regularity(), [Regularity::NonPeriodic]);
    assert_eq!(zero.generation_rule(), Rule::Zero);
    assert_eq!(zero.tenor(), None);
    assert_eq!(zero.stub_policy(), None);
    assert_eq!(zero.end_of_month(), None);
    for rule in [Rule::Forward, Rule::Backward] {
        for unit in TimeUnit::ALL {
            for length in [i32::MIN, -1, 0] {
                assert!(matches!(
                    b.build_recurrent(
                        Period::new(length, unit),
                        rule,
                        StubPolicy::Short,
                        EndOfMonth::Ignore,
                        10
                    ),
                    Err(ScheduleError::InvalidTenor { .. })
                ));
            }
        }
        for unit in [TimeUnit::Days, TimeUnit::Weeks] {
            assert!(matches!(
                b.build_recurrent(
                    Period::new(1, unit),
                    rule,
                    StubPolicy::Short,
                    EndOfMonth::Preserve,
                    100
                ),
                Err(ScheduleError::IncompatibleEndOfMonth { .. })
            ));
        }
    }
    assert!(matches!(
        b.build_recurrent(
            Period::new(0, TimeUnit::Days),
            Rule::Zero,
            StubPolicy::Short,
            EndOfMonth::Preserve,
            2
        ),
        Err(ScheduleError::InvalidRule { .. })
    ));
    Ok(())
}

#[test]
fn calendar_day_and_week_recurrences_agree_without_business_day_advancement()
-> Result<(), Box<dyn Error>> {
    let b = ScheduleBuilder::new(
        date("2026-01-09"),
        date("2026-02-06"),
        ScheduleCalendar::Plain(Calendar::weekends_only()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?;
    let day = b.build_recurrent(
        Period::new(7, TimeUnit::Days),
        Rule::Forward,
        StubPolicy::None,
        EndOfMonth::Ignore,
        5,
    )?;
    let week = b.build_recurrent(
        Period::new(1, TimeUnit::Weeks),
        Rule::Forward,
        StubPolicy::None,
        EndOfMonth::Ignore,
        5,
    )?;
    let expected = [
        "2026-01-09",
        "2026-01-16",
        "2026-01-23",
        "2026-01-30",
        "2026-02-06",
    ]
    .map(date);
    assert_eq!(day.contractual_dates(), expected);
    assert_eq!(week.contractual_dates(), expected);
    assert_eq!(day.tenor(), week.tenor()); // Period normalized equality remains true.
    assert_ne!(day, week); // Schedule definition retains its explicitly stored unit.
    assert_eq!(day.tenor().unwrap().unit(), TimeUnit::Days);
    assert_eq!(week.tenor().unwrap().unit(), TimeUnit::Weeks);
    Ok(())
}

#[test]
fn all_start_and_termination_conventions_have_independent_weekend_goldens()
-> Result<(), Box<dyn Error>> {
    let starts = [
        "2026-02-02",
        "2026-01-30",
        "2026-01-30",
        "2026-01-30",
        "2026-01-31",
        "2026-01-30",
        "2026-01-30",
    ]
    .map(date);
    let ends = [
        "2026-03-02",
        "2026-03-02",
        "2026-02-27",
        "2026-03-02",
        "2026-03-01",
        "2026-03-02",
        "2026-03-02",
    ]
    .map(date);
    for (i, ordinary) in Bdc::ALL.into_iter().enumerate() {
        for (j, termination) in Bdc::ALL.into_iter().enumerate() {
            let b = ScheduleBuilder::new(
                date("2026-01-31"),
                date("2026-03-01"),
                ScheduleCalendar::Plain(Calendar::weekends_only()),
                ordinary,
                termination,
            )?;
            assert_eq!(b.convention(), ordinary);
            assert_eq!(b.termination_convention(), termination);
            let s = b.build_zero(2)?;
            assert_eq!(
                s.contractual_dates(),
                [date("2026-01-31"), date("2026-03-01")]
            );
            assert_eq!(s.adjusted_dates(), [starts[i], ends[j]]);
            assert_eq!(s.convention(), ordinary);
            assert_eq!(s.termination_convention(), termination);
        }
    }
    let half = ScheduleBuilder::new(
        date("2026-02-14"),
        date("2026-02-18"),
        ScheduleCalendar::Plain(Calendar::weekends_only()),
        Bdc::HalfMonthModifiedFollowing,
        Bdc::Unadjusted,
    )?
    .build_zero(2)?;
    assert_eq!(
        half.adjusted_dates(),
        [date("2026-02-13"), date("2026-02-18")]
    );
    Ok(())
}

#[test]
fn daily_adjustment_collisions_and_terminal_reversal_preserve_both_raw_contexts()
-> Result<(), Box<dyn Error>> {
    for convention in Bdc::ALL {
        let b = ScheduleBuilder::new(
            date("2026-01-09"),
            date("2026-01-13"),
            ScheduleCalendar::Plain(Calendar::weekends_only()),
            convention,
            convention,
        )?;
        let result = b.build_recurrent(
            Period::new(1, TimeUnit::Days),
            Rule::Forward,
            StubPolicy::None,
            EndOfMonth::Ignore,
            5,
        );
        if convention == Bdc::Unadjusted {
            assert_eq!(result?.contractual_dates(), result_dates());
            continue;
        }
        let ScheduleError::AdjustedOrder {
            index,
            previous_contractual,
            contractual,
            previous_adjusted,
            adjusted,
            ..
        } = result.unwrap_err()
        else {
            panic!("expected adjustment collision");
        };
        let expected_index = if matches!(
            convention,
            Bdc::Following | Bdc::ModifiedFollowing | Bdc::HalfMonthModifiedFollowing
        ) {
            2
        } else {
            1
        };
        assert_eq!(index, expected_index);
        assert_eq!(contractual.days_since(previous_contractual), 1);
        assert_eq!(previous_adjusted, adjusted);
    }
    let error = ScheduleBuilder::new(
        date("2026-01-10"),
        date("2026-01-11"),
        ScheduleCalendar::Plain(Calendar::weekends_only()),
        Bdc::Following,
        Bdc::Preceding,
    )?
    .build_zero(2)
    .unwrap_err();
    assert!(
        matches!(error,ScheduleError::AdjustedOrder {index:1,previous_contractual,contractual,previous_adjusted,adjusted,..}
        if previous_contractual==date("2026-01-10")&&contractual==date("2026-01-11")&&previous_adjusted==date("2026-01-12")&&adjusted==date("2026-01-09"))
    );
    Ok(())
}

fn result_dates() -> [Date; 5] {
    [
        "2026-01-09",
        "2026-01-10",
        "2026-01-11",
        "2026-01-12",
        "2026-01-13",
    ]
    .map(date)
}

#[test]
fn checked_target_span_guards_cannot_be_bypassed_by_zero_unadjusted_overrides_or_joints()
-> Result<(), Box<dyn Error>> {
    let strict = TargetCalendar::new();
    let joint = strict
        .without_holiday(date("2003-01-01"))
        .joint_with(vec![Calendar::null()], JointRule::JoinBusinessDays)?;
    for calendar in [strict.clone(), joint] {
        let b = ScheduleBuilder::new(
            date("2002-12-31"),
            date("2026-01-01"),
            ScheduleCalendar::Target(calendar),
            Bdc::Unadjusted,
            Bdc::Unadjusted,
        )?;
        for result in [
            b.build_zero(2),
            b.build_recurrent(
                Period::new(100, TimeUnit::Years),
                Rule::Forward,
                StubPolicy::Short,
                EndOfMonth::Ignore,
                2,
            ),
        ] {
            let error = result.unwrap_err();
            assert_eq!(error.kind(), "schedule.calendar");
            assert!(
                matches!(error,ScheduleError::Calendar {index:None,cause:CalendarError::UnverifiedCoverage {date:d,..},..} if d==date("2003-01-01"))
            );
            assert!(error.source().unwrap().is::<CalendarError>());
        }
        assert!(matches!(
            b.build_recurrent(
                Period::new(0, TimeUnit::Days),
                Rule::Forward,
                StubPolicy::Short,
                EndOfMonth::Preserve,
                2
            ),
            Err(ScheduleError::InvalidTenor { .. })
        ));
    }
    let projected = ScheduleBuilder::new(
        date("2002-12-31"),
        date("2026-01-01"),
        ScheduleCalendar::Target(TargetCalendar::with_policy(TargetPolicy::AllowProjection)),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?
    .build_zero(2)?;
    assert_eq!(projected.regularity(), [Regularity::NonPeriodic]);
    for policy in [TargetPolicy::VerifiedOnly, TargetPolicy::AllowProjection] {
        let error = ScheduleBuilder::new(
            date("1999-01-03"),
            date("1999-01-04"),
            ScheduleCalendar::Target(
                TargetCalendar::with_policy(policy).without_holiday(date("1999-01-03")),
            ),
            Bdc::Unadjusted,
            Bdc::Unadjusted,
        )?
        .build_zero(2)
        .unwrap_err();
        assert!(
            matches!(error, ScheduleError::Calendar {index:None,cause:CalendarError::OutsideValidity {date:d,..},..} if d==date("1999-01-03"))
        );
    }
    Ok(())
}

#[test]
fn calendar_eom_activation_and_nested_date_arithmetic_errors_are_explicit()
-> Result<(), Box<dyn Error>> {
    // Jan30 is a business month-end here, but never a calendar month-end.
    // Contractual EOM activation uses that raw calendar property, independently
    // of business-EOM advancement behavior under other adjustment conventions.
    let schedule = ScheduleBuilder::new(
        date("2026-01-30"),
        date("2026-05-30"),
        ScheduleCalendar::Plain(Calendar::weekends_only()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?
    .build_recurrent(
        Period::new(1, TimeUnit::Months),
        Rule::Forward,
        StubPolicy::None,
        EndOfMonth::Preserve,
        5,
    )?;
    assert_eq!(
        schedule.contractual_dates(),
        [
            "2026-01-30",
            "2026-02-28",
            "2026-03-30",
            "2026-04-30",
            "2026-05-30"
        ]
        .map(date)
    );
    for (start, end, calendar, ordinary, termination, index) in [
        (
            Date::MIN,
            Date::MIN.checked_add_days(1)?,
            Calendar::null().with_holiday(Date::MIN),
            Bdc::Preceding,
            Bdc::Unadjusted,
            0,
        ),
        (
            Date::MAX.checked_sub_days(1)?,
            Date::MAX,
            Calendar::null().with_holiday(Date::MAX),
            Bdc::Unadjusted,
            Bdc::Following,
            1,
        ),
    ] {
        let error = ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(calendar),
            ordinary,
            termination,
        )?
        .build_zero(2)
        .unwrap_err();
        let ScheduleError::Calendar {
            index: actual,
            cause: CalendarError::Date(ref date_error),
            ..
        } = error
        else {
            panic!("expected nested date arithmetic failure: {error:?}");
        };
        assert_eq!(actual, Some(index));
        assert_eq!(error.kind(), "schedule.calendar");
        let calendar_error = error
            .source()
            .unwrap()
            .downcast_ref::<CalendarError>()
            .unwrap();
        assert_eq!(calendar_error.kind(), "calendar.date");
        assert_eq!(
            calendar_error
                .source()
                .unwrap()
                .downcast_ref::<sabiquant::time::DateError>(),
            Some(date_error)
        );
        assert_eq!(error.clone(), error);
    }
    Ok(())
}

#[test]
fn target_easter_overrides_and_calendar_search_errors_stay_owned_and_typed()
-> Result<(), Box<dyn Error>> {
    let target = TargetCalendar::new();
    for (calendar, expected) in [
        (target.clone(), "2026-04-07"),
        (target.without_holiday(date("2026-04-03")), "2026-04-03"),
    ] {
        let s = ScheduleBuilder::new(
            date("2026-04-02"),
            date("2026-04-03"),
            ScheduleCalendar::Target(calendar),
            Bdc::Unadjusted,
            Bdc::Following,
        )?
        .build_zero(2)?;
        assert_eq!(s.adjusted_dates(), [date("2026-04-02"), date(expected)]);
    }
    assert!(target.is_holiday(date("2026-04-03"))?);
    let days = (1..=28)
        .map(|d| Date::from_ymd(2026, 2, d))
        .collect::<Result<Vec<_>, _>>()?;
    let closed = Calendar::custom([Weekday::Saturday, Weekday::Sunday], days);
    let error = ScheduleBuilder::new(
        date("2026-02-01"),
        date("2026-03-31"),
        ScheduleCalendar::Plain(closed),
        Bdc::ModifiedFollowing,
        Bdc::Unadjusted,
    )?
    .build_zero(2)
    .unwrap_err();
    assert!(matches!(
        error,
        ScheduleError::Calendar {
            index: Some(0),
            cause: CalendarError::NoBusinessDayInMonth { .. },
            ..
        }
    ));
    let start = date("2026-01-01");
    let holidays = (0..366)
        .map(|offset| start.checked_add_days(offset))
        .collect::<Result<Vec<_>, _>>()?;
    let error = ScheduleBuilder::new(
        start,
        date("2028-01-01"),
        ScheduleCalendar::Plain(Calendar::custom([], holidays)),
        Bdc::Following,
        Bdc::Unadjusted,
    )?
    .build_zero(2)
    .unwrap_err();
    assert!(matches!(
        error,
        ScheduleError::Calendar {
            index: Some(0),
            cause: CalendarError::SearchLimit { .. },
            ..
        }
    ));
    Ok(())
}

#[test]
fn public_traits_catalogs_and_shared_builder_reuse_keep_definition_equality()
-> Result<(), Box<dyn Error>> {
    fn choices<T: Copy + Eq + Hash + std::fmt::Debug + Send + Sync + 'static>() {}
    fn values<T: Clone + Eq + Hash + std::fmt::Debug + Send + Sync + 'static>() {}
    fn errors<T: Error + Clone + PartialEq + Send + Sync + 'static>() {}
    choices::<Rule>();
    choices::<StubPolicy>();
    choices::<Regularity>();
    values::<Schedule>();
    values::<ScheduleBuilder>();
    values::<ScheduleCalendar>();
    errors::<ScheduleError>();
    macro_rules! catalog {
        ($ty:ty) => {
            for value in <$ty>::ALL {
                let text = value.to_string();
                assert_eq!(text.parse::<$ty>()?, value);
                for bad in [
                    format!(" {text}"),
                    format!("{text} "),
                    text.to_lowercase(),
                    format!("{text}\0"),
                ] {
                    assert!(bad.parse::<$ty>().is_err());
                }
            }
        };
    }
    catalog!(Rule);
    catalog!(StubPolicy);
    catalog!(Regularity);
    let b = Arc::new(builder(date("2024-01-31"), date("2026-01-31"))?);
    let baseline = b.build_recurrent(
        Period::new(1, TimeUnit::Months),
        Rule::Forward,
        StubPolicy::None,
        EndOfMonth::Preserve,
        25,
    )?;
    assert_eq!(b.start(), date("2024-01-31"));
    assert_eq!(b.end(), date("2026-01-31"));
    assert_eq!(baseline.calendar(), b.calendar());
    assert_eq!(hash(&baseline), hash(&baseline.clone()));
    let mut workers = Vec::new();
    for _ in 0..8 {
        let b = Arc::clone(&b);
        #[expect(
            clippy::disallowed_methods,
            reason = "test-only concurrent immutable schedule-builder reuse"
        )]
        let worker = std::thread::spawn(move || {
            let mut results = Vec::new();
            for _ in 0..16 {
                results.push(b.build_recurrent(
                    Period::new(1, TimeUnit::Months),
                    Rule::Forward,
                    StubPolicy::None,
                    EndOfMonth::Preserve,
                    25,
                ));
            }
            results
        });
        workers.push(worker);
    }
    for worker in workers {
        for result in worker.join().map_err(|_| "schedule worker panicked")? {
            let s = result?;
            assert_eq!(s, baseline);
            assert_eq!(hash(&s), hash(&baseline));
        }
    }
    Ok(())
}
