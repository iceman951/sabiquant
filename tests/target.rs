//! TARGET source anchors, guarded coverage and immutable composition.

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, JointRule, TargetCalendar,
    TargetCoverage, TargetPolicy,
};
use sabiquant::time::{Date, Period, TimeUnit, Weekday};
use std::error::Error;

#[expect(
    clippy::expect_used,
    reason = "all test fixture component triples are valid Gregorian dates"
)]
fn d(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid TARGET test date")
}

// Dated anchors, not a second invocation of production computus. Historical
// year schedules are from ECB releases, Easter dates independently anchored
// to the primary Census table; 2026–2028 dates are explicitly marked by ECB.
fn source_holidays(year: i32) -> Vec<Date> {
    let mut holidays = vec![d(year, 1, 1), d(year, 12, 25)];
    if year >= 2000 {
        let (month, friday, monday) = match year {
            2000 => (4, 21, 24),
            2001 => (4, 13, 16),
            2002 => (3, 29, 0),
            2026 => (4, 3, 6),
            2027 => (3, 26, 29),
            2028 => (4, 14, 17),
            _ => return holidays,
        };
        holidays.push(d(year, month, friday));
        holidays.push(if monday == 0 {
            d(year, 4, 1)
        } else {
            d(year, month, monday)
        });
        holidays.extend([d(year, 5, 1), d(year, 12, 26)]);
    }
    if matches!(year, 1999 | 2001) {
        holidays.push(d(year, 12, 31));
    }
    holidays
}

#[test]
fn every_verified_date_matches_independent_dated_source_rules() -> Result<(), Box<dyn Error>> {
    let target = TargetCalendar::new();
    let mut total = 0;
    for year in [1999, 2000, 2001, 2002, 2026, 2027, 2028] {
        let first = if year == 1999 {
            d(year, 1, 4)
        } else {
            d(year, 1, 1)
        };
        let last = d(year, 12, 31);
        let holidays = source_holidays(year);
        let mut business = 0;
        for offset in 0..=last.days_since(first) {
            let date = first.checked_add_days(i64::from(offset))?;
            let weekend = matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday);
            let expected = !weekend && !holidays.contains(&date);
            assert_eq!(target.coverage(date), TargetCoverage::Verified);
            assert_eq!(target.is_weekend(date)?, weekend);
            assert_eq!(target.is_business_day(date)?, expected, "{date}");
            assert_eq!(target.is_holiday(date)?, !expected);
            business += i32::from(expected);
            total += 1;
        }
        assert_eq!(
            target.business_days_between(first, last, true, true)?,
            business,
            "year {year}"
        );
        assert_eq!(
            target.business_day_list(first, last)?.len(),
            usize::try_from(business)?
        );
        assert_eq!(
            target.holiday_list(first, last, true)?.len(),
            usize::try_from(last.days_since(first) + 1 - business)?
        );
    }
    assert_eq!(total, 2554);
    assert!(target.is_business_day(d(1999, 4, 2))?);
    assert!(target.is_business_day(d(1999, 4, 5))?);
    assert!(!target.is_business_day(d(1999, 12, 31))?);
    assert!(!target.is_business_day(d(2001, 12, 31))?);
    assert!(target.is_business_day(d(2002, 12, 31))?);
    Ok(())
}

#[test]
fn coverage_is_disjoint_policy_independent_and_inception_is_never_projectable()
-> Result<(), CalendarError> {
    let strict = TargetCalendar::default();
    let projected = TargetCalendar::with_policy(TargetPolicy::AllowProjection);
    for (date, expected) in [
        (Date::MIN, TargetCoverage::OutsideValidity),
        (d(1998, 12, 31), TargetCoverage::OutsideValidity),
        (d(1999, 1, 3), TargetCoverage::OutsideValidity),
        (d(1999, 1, 4), TargetCoverage::Verified),
        (d(2002, 12, 31), TargetCoverage::Verified),
        (d(2003, 1, 1), TargetCoverage::Projected),
        (d(2025, 12, 31), TargetCoverage::Projected),
        (d(2026, 1, 1), TargetCoverage::Verified),
        (d(2028, 12, 31), TargetCoverage::Verified),
        (d(2029, 1, 1), TargetCoverage::Projected),
        (Date::MAX, TargetCoverage::Projected),
    ] {
        assert_eq!(strict.coverage(date), expected);
        assert_eq!(projected.coverage(date), expected);
        match expected {
            TargetCoverage::OutsideValidity => {
                for calendar in [&strict, &projected] {
                    assert!(
                        matches!(calendar.is_business_day(date),Err(CalendarError::OutsideValidity {date:rejected,..})if rejected==date)
                    );
                }
            }
            TargetCoverage::Projected => {
                assert!(
                    matches!(strict.is_business_day(date),Err(CalendarError::UnverifiedCoverage {date:rejected,..})if rejected==date)
                );
                assert!(projected.is_business_day(date).is_ok());
            }
            TargetCoverage::Verified => assert_eq!(
                strict.is_business_day(date)?,
                projected.is_business_day(date)?
            ),
        }
    }
    assert_eq!(strict, TargetCalendar::new());
    assert_ne!(strict, projected);
    Ok(())
}

#[test]
fn no_zero_unadjusted_override_or_joint_boolean_shortcut_bypasses_policy()
-> Result<(), CalendarError> {
    let target = TargetCalendar::new();
    for date in [d(1998, 12, 31), d(2003, 1, 1), d(2024, 3, 29)] {
        assert!(target.adjust(date, Bdc::Unadjusted).is_err());
        assert!(target.add_business_days(date, 0).is_err());
        assert!(
            target
                .advance(date, Period::new(0, TimeUnit::Days), Bdc::Unadjusted, false)
                .is_err()
        );
        assert!(
            target
                .business_days_between(date, date, false, false)
                .is_err()
        );
        assert!(target.is_weekend(date).is_err());
        assert!(target.without_holiday(date).is_business_day(date).is_err());
        assert!(target.with_holiday(date).is_holiday(date).is_err());
        for rule in JointRule::ALL {
            let dominant = if rule == JointRule::JoinBusinessDays {
                Calendar::null()
            } else {
                Calendar::custom(Weekday::ALL, [])
            };
            let joint = target.joint_with(vec![dominant], rule)?;
            assert!(joint.is_business_day(date).is_err());
            assert!(joint.adjust(date, Bdc::Unadjusted).is_err());
            assert!(joint.without_holiday(date).is_business_day(date).is_err());
        }
    }
    Ok(())
}

#[test]
fn counted_and_listed_intervals_cannot_bridge_an_unverified_gap() -> Result<(), CalendarError> {
    let target = TargetCalendar::new();
    let first = d(2002, 12, 31);
    let last = d(2026, 1, 1);
    for include_start in [false, true] {
        for include_end in [false, true] {
            for (start, end) in [(first, last), (last, first)] {
                assert!(
                    matches!(target.business_days_between(start,end,include_start,include_end),Err(CalendarError::UnverifiedCoverage {date,..})if date==d(2003,1,1))
                );
            }
        }
    }
    assert!(target.holiday_list(first, last, true).is_err());
    assert!(target.business_day_list(first, last).is_err());
    assert!(matches!(
        target.add_business_days(first, 1),
        Err(CalendarError::UnverifiedCoverage { .. })
    ));
    assert!(matches!(
        target.adjust(d(2028, 12, 31), Bdc::Following),
        Err(CalendarError::UnverifiedCoverage { .. })
    ));
    assert!(matches!(
        target.advance(
            first,
            Period::new(1, TimeUnit::Months),
            Bdc::Unadjusted,
            false
        ),
        Err(CalendarError::UnverifiedCoverage { .. })
    ));
    // Even an omitted endpoint is an input requiring source coverage.
    assert!(
        target
            .business_days_between(d(2003, 1, 1), d(2003, 1, 2), false, false)
            .is_err()
    );
    assert!(target.start_of_month(d(1999, 1, 4)).is_err());
    Ok(())
}

#[test]
fn explicit_projection_preserves_recurring_rule_behavior_and_source_labels()
-> Result<(), CalendarError> {
    let target = TargetCalendar::with_policy(TargetPolicy::AllowProjection);
    assert_eq!(target.coverage(d(2024, 3, 29)), TargetCoverage::Projected);
    assert_eq!(
        target.adjust(d(2024, 3, 29), Bdc::Following)?,
        d(2024, 4, 2)
    );
    assert_eq!(
        target.adjust(d(2024, 3, 29), Bdc::ModifiedFollowing)?,
        d(2024, 3, 28)
    );
    assert_eq!(target.end_of_month(d(2024, 3, 29))?, d(2024, 3, 28));
    assert!(target.is_end_of_month(d(2024, 3, 29))?);
    assert_eq!(target.add_business_days(d(2024, 3, 28), 1)?, d(2024, 4, 2));
    assert!(target.is_business_day(d(2021, 12, 27))?);
    assert!(target.is_business_day(d(2024, 12, 24))?);
    assert!(target.is_business_day(d(2024, 12, 31))?);
    assert!(target.is_business_day(d(2026, 5, 14))?);
    assert_eq!(target.coverage(d(2026, 5, 14)), TargetCoverage::Verified);
    Ok(())
}

#[test]
fn pure_target_fast_count_matches_daily_classification_with_flags_and_overrides()
-> Result<(), Box<dyn Error>> {
    let plain = TargetCalendar::with_policy(TargetPolicy::AllowProjection);
    let calendars = [
        plain.clone(),
        plain
            .with_holiday(d(2000, 2, 29))
            .without_holiday(d(2000, 4, 21))
            .without_holiday(d(1999, 12, 25)),
        plain.joint_with(vec![Calendar::weekends_only()], JointRule::JoinHolidays)?,
    ];
    let base = d(1999, 12, 15);
    for calendar in calendars {
        for a in 0..=40 {
            for b in 0..=40 {
                let start = base.checked_add_days(a)?;
                let end = base.checked_add_days(b)?;
                for include_start in [false, true] {
                    for include_end in [false, true] {
                        let expected = if a == b {
                            i32::from(
                                include_start && include_end && calendar.is_business_day(start)?,
                            )
                        } else {
                            let mut count = 0;
                            for offset in a.min(b)..=a.max(b) {
                                if (offset != a || include_start)
                                    && (offset != b || include_end)
                                    && calendar.is_business_day(base.checked_add_days(offset)?)?
                                {
                                    count += 1;
                                }
                            }
                            if a > b { -count } else { count }
                        };
                        assert_eq!(
                            calendar.business_days_between(
                                start,
                                end,
                                include_start,
                                include_end
                            )?,
                            expected,
                            "{start} {end} {include_start}/{include_end}"
                        );
                    }
                }
            }
        }
        for (first, last) in [
            (d(1999, 1, 4), d(2002, 12, 31)),
            (d(2026, 1, 1), d(2028, 12, 31)),
            (d(2000, 2, 1), d(2000, 5, 2)),
        ] {
            let expected = calendar.business_day_list(first, last)?.len();
            assert_eq!(
                usize::try_from(calendar.business_days_between(first, last, true, true)?)?,
                expected
            );
        }
    }
    Ok(())
}

#[test]
fn guarded_mutations_and_core_composition_remain_immutable() -> Result<(), CalendarError> {
    let original = TargetCalendar::new();
    let holiday = d(2026, 4, 3);
    let changed = original.without_holiday(holiday);
    assert!(!original.is_business_day(holiday)?);
    assert!(changed.is_business_day(holiday)?);
    assert_eq!(changed.with_holiday(holiday), original);
    let weekend = d(2026, 4, 4);
    assert!(changed.without_holiday(weekend).is_business_day(weekend)?);
    assert!(original.is_weekend(weekend)?);
    let extra = Calendar::null().with_holiday(d(2026, 4, 7));
    assert!(
        !original
            .joint_with(vec![extra.clone()], JointRule::JoinHolidays)?
            .is_business_day(d(2026, 4, 7))?
    );
    assert!(
        original
            .joint_with(vec![extra], JointRule::JoinBusinessDays)?
            .is_business_day(holiday)?
    );
    assert!(!original.is_business_day(holiday)?);
    Ok(())
}

#[test]
fn provenance_constants_are_explicit_without_invented_maintainer_appointment() {
    assert_eq!(TargetCalendar::VALID_FROM, "1999-01-04");
    assert_eq!(
        TargetCalendar::VERIFIED_INTERVALS,
        [("1999-01-04", "2002-12-31"), ("2026-01-01", "2028-12-31")]
    );
    assert_eq!(TargetCalendar::LAST_REVIEWED, "2026-10-11");
    assert!(TargetCalendar::MAINTAINER_ROLE.contains("appointment required"));
    assert_eq!(TargetCalendar::SOURCES.len(), 6);
    for (_, url) in TargetCalendar::SOURCES {
        assert!(url.starts_with("https://www.ecb.europa.eu/"));
    }
    fn value<T: Clone + std::fmt::Debug + Eq + std::hash::Hash + Send + Sync + 'static>() {}
    value::<TargetCalendar>();
    value::<TargetPolicy>();
    value::<TargetCoverage>();
}

#[test]
fn full_projection_range_count_matches_daily_oracle_through_max_and_exact_overrides()
-> Result<(), Box<dyn Error>> {
    let target = TargetCalendar::with_policy(TargetPolicy::AllowProjection);
    let first = d(1999, 1, 4);
    let mut expected = 0;
    for offset in 0..=Date::MAX.days_since(first) {
        expected += i32::from(target.is_business_day(first.checked_add_days(i64::from(offset))?)?);
    }
    assert_eq!(
        target.business_days_between(first, Date::MAX, true, true)?,
        expected
    );
    assert_eq!(
        target.business_days_between(Date::MAX, first, true, true)?,
        -expected
    );
    assert_eq!(
        target.business_days_between(first, Date::MAX, false, true)?,
        expected - 1
    );
    assert_eq!(
        target.business_days_between(first, Date::MAX, true, false)?,
        expected - 1
    );
    assert_eq!(
        target.business_days_between(first, Date::MAX, false, false)?,
        expected - 2
    );
    assert_eq!(
        target
            .with_holiday(Date::MAX)
            .business_days_between(first, Date::MAX, true, true)?,
        expected - 1
    );
    let opened_christmas = d(9999, 12, 25);
    assert!(!target.is_business_day(opened_christmas)?);
    assert_eq!(
        target
            .without_holiday(opened_christmas)
            .business_days_between(first, Date::MAX, true, true)?,
        expected + 1
    );
    assert_eq!(
        target.business_days_between(Date::MAX, Date::MAX, true, true)?,
        1
    );
    Ok(())
}

#[test]
fn nearest_preserves_coverage_errors_when_other_direction_exhausts_search_bound()
-> Result<(), Box<dyn Error>> {
    for first in [d(1999, 1, 4), d(2026, 1, 1)] {
        let mut target = TargetCalendar::new();
        for offset in 0..366 {
            target = target.with_holiday(first.checked_add_days(offset)?);
        }
        assert!(matches!(
            target.adjust(first, Bdc::Following),
            Err(CalendarError::SearchLimit { rejected: 366, .. })
        ));
        let result = target.adjust(first, Bdc::Nearest);
        if first.year() == 1999 {
            assert!(
                matches!(result,Err(CalendarError::OutsideValidity {date,..})if date==d(1999,1,3))
            );
        } else {
            assert!(
                matches!(result,Err(CalendarError::UnverifiedCoverage {date,..})if date==d(2025,12,31))
            );
        }
    }
    Ok(())
}

#[test]
fn public_metadata_month_queries_and_lists_preserve_source_and_weekend_meanings()
-> Result<(), CalendarError> {
    assert_eq!(TargetPolicy::VerifiedOnly.to_string(), "VerifiedOnly");
    assert_eq!(TargetPolicy::AllowProjection.to_string(), "AllowProjection");
    assert_eq!(TargetCoverage::Verified.to_string(), "Verified");
    assert_eq!(TargetCoverage::Projected.to_string(), "Projected");
    assert_eq!(
        TargetCoverage::OutsideValidity.to_string(),
        "OutsideValidity"
    );
    let target = TargetCalendar::new();
    let holiday = d(2026, 1, 1);
    let first_business = d(2026, 1, 2);
    let saturday = d(2026, 1, 3);
    let sunday = d(2026, 1, 4);
    assert_eq!(target.start_of_month(sunday)?, first_business);
    assert!(target.is_start_of_month(holiday)?);
    assert!(target.is_start_of_month(first_business)?);
    assert!(!target.is_start_of_month(saturday)?);
    assert_eq!(target.end_of_month(sunday)?, d(2026, 1, 30));
    assert!(target.is_end_of_month(d(2026, 1, 31))?);
    assert!(!target.is_end_of_month(d(2026, 1, 29))?);
    assert_eq!(
        target.holiday_list(holiday, sunday, true)?,
        vec![holiday, saturday, sunday]
    );
    assert_eq!(target.holiday_list(holiday, sunday, false)?, vec![holiday]);
    assert_eq!(
        target.business_day_list(holiday, sunday)?,
        vec![first_business]
    );
    assert!(
        target
            .holiday_list(first_business, first_business, true)?
            .is_empty()
    );
    assert!(target.business_day_list(holiday, holiday)?.is_empty());
    assert!(target.holiday_list(saturday, sunday, false)?.is_empty());
    let unverified = d(2024, 1, 15);
    assert!(
        matches!(target.start_of_month(unverified),Err(CalendarError::UnverifiedCoverage {date,..})if date==unverified)
    );
    assert!(
        matches!(target.end_of_month(unverified),Err(CalendarError::UnverifiedCoverage {date,..})if date==unverified)
    );
    assert!(
        matches!(target.is_start_of_month(unverified),Err(CalendarError::UnverifiedCoverage {date,..})if date==unverified)
    );
    assert!(
        matches!(target.is_end_of_month(unverified),Err(CalendarError::UnverifiedCoverage {date,..})if date==unverified)
    );
    assert!(matches!(
        target.holiday_list(sunday, holiday, true),
        Err(CalendarError::InvalidRange { .. })
    ));
    assert!(matches!(
        target.business_day_list(sunday, holiday),
        Err(CalendarError::InvalidRange { .. })
    ));
    Ok(())
}

#[test]
fn guarded_date_arithmetic_preserves_max_boundary_and_rejects_pre_inception_results()
-> Result<(), CalendarError> {
    let projected = TargetCalendar::with_policy(TargetPolicy::AllowProjection);
    for unit in TimeUnit::ALL {
        assert!(matches!(
            projected.advance(Date::MAX, Period::new(1, unit), Bdc::Unadjusted, false),
            Err(CalendarError::Date(_))
        ));
        assert_eq!(
            projected.advance(Date::MAX, Period::new(0, unit), Bdc::Unadjusted, false)?,
            Date::MAX
        );
    }
    assert!(matches!(
        projected.add_business_days(Date::MAX, 1),
        Err(CalendarError::Date(_))
    ));
    assert!(
        matches!(projected.add_business_days(d(1999,1,4),-1),Err(CalendarError::OutsideValidity {date,..})if date==d(1999,1,3))
    );
    assert!(
        matches!(projected.advance(d(1999,1,4),Period::new(-1,TimeUnit::Years),Bdc::Unadjusted,false),Err(CalendarError::OutsideValidity {date,..})if date==d(1998,1,4))
    );
    Ok(())
}
