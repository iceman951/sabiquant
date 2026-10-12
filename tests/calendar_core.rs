//! Core calendar boundary and invariant tests independent of market data.

use sabiquant::calendar::{BusinessDayConvention as Bdc, Calendar, CalendarError, JointRule};
use sabiquant::calendar::{EndOfMonth, EndpointInclusion, HolidayWeekendInclusion};
use sabiquant::time::{Date, Period, TimeUnit, Weekday};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};

#[expect(
    clippy::expect_used,
    reason = "the literal test fixtures are valid Gregorian dates"
)]
fn d(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid calendar test date")
}

fn digest(calendar: &Calendar) -> u64 {
    let mut hasher = DefaultHasher::new();
    calendar.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn every_weekend_set_classifies_a_full_leap_year_and_respects_overrides()
-> Result<(), Box<dyn Error>> {
    let start = d(2000, 1, 1);
    for mask in 0_u32..128 {
        let weekends: Vec<_> = Weekday::ALL
            .into_iter()
            .filter(|weekday| mask & (1 << (weekday.number_from_monday() - 1)) != 0)
            .collect();
        let holidays = [d(2000, 2, 29), d(2000, 1, 1), d(2000, 12, 31)];
        let calendar = Calendar::custom(weekends.clone(), holidays);
        for offset in 0..366 {
            let date = start.checked_add_days(offset)?;
            let weekend = mask & (1 << (date.weekday().number_from_monday() - 1)) != 0;
            assert_eq!(calendar.is_weekend(date), weekend);
            assert_eq!(
                calendar.is_business_day(date),
                !weekend && !holidays.contains(&date)
            );
            assert_eq!(
                calendar.is_holiday(date),
                weekend || holidays.contains(&date)
            );
        }
        let overridden = calendar.without_holiday(holidays[0]);
        assert!(overridden.is_business_day(holidays[0]));
        assert!(!calendar.is_business_day(holidays[0]));
        assert_eq!(
            overridden.is_weekend(holidays[0]),
            calendar.is_weekend(holidays[0])
        );
        assert_eq!(overridden.with_holiday(holidays[0]), calendar);
    }
    assert_eq!(Calendar::custom([], []), Calendar::null());
    assert_eq!(
        Calendar::custom([Weekday::Sunday, Weekday::Saturday], []),
        Calendar::weekends_only()
    );
    Ok(())
}

#[test]
fn canonical_content_order_duplicate_inputs_and_immutable_branches() -> Result<(), CalendarError> {
    let first = d(2024, 7, 3);
    let second = d(2024, 7, 4);
    let a = Calendar::custom([Weekday::Saturday, Weekday::Sunday], [first, second]);
    let b = Calendar::custom(
        [Weekday::Sunday, Weekday::Saturday, Weekday::Sunday],
        [second, first, second],
    );
    assert_eq!(a, b);
    assert_eq!(digest(&a), digest(&b));
    let changed = a.without_holiday(first);
    assert!(changed.is_business_day(first));
    assert!(a.is_holiday(first));
    assert_eq!(changed.with_holiday(first), a);
    assert_eq!(
        Calendar::null().with_holiday(first),
        Calendar::custom([], [first])
    );
    for rule in JointRule::ALL {
        let left = Calendar::joint(vec![a.clone(), Calendar::null(), b.clone()], rule)?;
        let right = Calendar::joint(vec![Calendar::null(), b.clone()], rule)?;
        assert_eq!(left, right);
        assert_eq!(digest(&left), digest(&right));
        let nested = Calendar::joint(vec![left.clone(), a.clone()], rule)?;
        assert_eq!(nested, left);
        let override_joint = left.without_holiday(first);
        assert!(override_joint.is_business_day(first));
        // An overridden joint must remain a whole member when nested; the
        // override must not be discarded by same-rule flattening.
        let nested_override = Calendar::joint(vec![override_joint.clone()], rule)?;
        assert!(nested_override.is_business_day(first));
        assert_eq!(
            left.is_business_day(first),
            rule == JointRule::JoinBusinessDays
        );
    }
    Ok(())
}

#[test]
fn both_joint_rules_match_boolean_definitions_for_every_weekend_pair() -> Result<(), Box<dyn Error>>
{
    let monday = d(2024, 4, 1);
    for left_mask in 0_u32..128 {
        for right_mask in 0_u32..128 {
            let weekends = |mask| {
                Weekday::ALL
                    .into_iter()
                    .filter(move |weekday| mask & (1 << (weekday.number_from_monday() - 1)) != 0)
            };
            let left = Calendar::custom(weekends(left_mask), [monday]);
            let right = Calendar::custom(weekends(right_mask), [monday.checked_add_days(1)?]);
            for rule in JointRule::ALL {
                let joint = Calendar::joint(vec![left.clone(), right.clone()], rule)?;
                for offset in 0..7 {
                    let date = monday.checked_add_days(offset)?;
                    let expected_business = match rule {
                        JointRule::JoinHolidays => {
                            left.is_business_day(date) && right.is_business_day(date)
                        }
                        JointRule::JoinBusinessDays => {
                            left.is_business_day(date) || right.is_business_day(date)
                        }
                    };
                    let expected_weekend = match rule {
                        JointRule::JoinHolidays => left.is_weekend(date) || right.is_weekend(date),
                        JointRule::JoinBusinessDays => {
                            left.is_weekend(date) && right.is_weekend(date)
                        }
                    };
                    assert_eq!(joint.is_business_day(date), expected_business);
                    assert_eq!(joint.is_weekend(date), expected_weekend);
                }
            }
        }
    }
    Ok(())
}

#[test]
fn adjustment_examples_cover_all_conventions_half_month_and_nearest_ties()
-> Result<(), CalendarError> {
    let weekends = Calendar::weekends_only();
    let rows = [
        (
            d(2024, 8, 31),
            [
                d(2024, 9, 2),
                d(2024, 8, 30),
                d(2024, 8, 30),
                d(2024, 8, 30),
                d(2024, 8, 31),
                d(2024, 8, 30),
                d(2024, 8, 30),
            ],
        ),
        (
            d(2024, 6, 15),
            [
                d(2024, 6, 17),
                d(2024, 6, 17),
                d(2024, 6, 14),
                d(2024, 6, 14),
                d(2024, 6, 15),
                d(2024, 6, 14),
                d(2024, 6, 14),
            ],
        ),
        (
            d(2024, 9, 1),
            [
                d(2024, 9, 2),
                d(2024, 9, 2),
                d(2024, 8, 30),
                d(2024, 9, 2),
                d(2024, 9, 1),
                d(2024, 9, 2),
                d(2024, 9, 2),
            ],
        ),
    ];
    for (date, expected) in rows {
        for (convention, expected) in Bdc::ALL.into_iter().zip(expected) {
            assert_eq!(
                weekends.adjust(date, convention)?,
                expected,
                "{date} {convention}"
            );
        }
    }
    let tie = weekends.with_holiday(d(2024, 7, 3));
    assert_eq!(tie.adjust(d(2024, 7, 3), Bdc::Nearest)?, d(2024, 7, 4));
    Ok(())
}

#[test]
fn adjustment_is_identity_idempotent_and_respects_direction_over_century_windows()
-> Result<(), Box<dyn Error>> {
    let calendars = [
        Calendar::null(),
        Calendar::weekends_only(),
        Calendar::custom(
            [Weekday::Friday, Weekday::Saturday],
            [d(2000, 2, 29), d(2100, 3, 1)],
        ),
    ];
    for start in [d(1999, 1, 1), d(2099, 1, 1)] {
        for calendar in &calendars {
            for offset in 0..1096 {
                let date = start.checked_add_days(offset)?;
                for convention in Bdc::ALL {
                    let adjusted = calendar.adjust(date, convention)?;
                    assert_eq!(calendar.adjust(adjusted, convention)?, adjusted);
                    if calendar.is_business_day(date) || convention == Bdc::Unadjusted {
                        assert_eq!(adjusted, date);
                    } else {
                        assert!(calendar.is_business_day(adjusted));
                    }
                    match convention {
                        Bdc::Following => assert!(adjusted >= date),
                        Bdc::Preceding => assert!(adjusted <= date),
                        Bdc::ModifiedFollowing
                        | Bdc::ModifiedPreceding
                        | Bdc::HalfMonthModifiedFollowing => assert_eq!(
                            (adjusted.year(), adjusted.month()),
                            (date.year(), date.month())
                        ),
                        Bdc::Unadjusted | Bdc::Nearest => {}
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn bounded_search_counts_rejected_candidates_and_accepts_reachable_overrides()
-> Result<(), Box<dyn Error>> {
    let base = d(2024, 1, 1);
    for direction in [-1_i64, 1] {
        for closed_count in [365, 366, 367] {
            let holidays: Vec<_> = (0..closed_count)
                .map(|offset| base.checked_add_days(direction * offset))
                .collect::<Result<_, _>>()?;
            let calendar = Calendar::custom([], holidays);
            let convention = if direction > 0 {
                Bdc::Following
            } else {
                Bdc::Preceding
            };
            let result = calendar.adjust(base, convention);
            if closed_count == 365 {
                assert_eq!(result?, base.checked_add_days(direction * 365)?);
            } else {
                assert!(
                    matches!(result, Err(CalendarError::SearchLimit { start, direction: step, rejected: 366, .. }) if start == base && i64::from(step) == direction)
                );
            }
        }
    }
    let closed = Calendar::custom(Weekday::ALL, []);
    assert!(closed.is_holiday(base));
    for convention in Bdc::ALL {
        if convention == Bdc::Unadjusted {
            assert_eq!(closed.adjust(base, convention)?, base);
        } else {
            assert!(matches!(
                closed.adjust(base, convention),
                Err(CalendarError::NoBusinessWeekday)
            ));
        }
    }
    assert_eq!(closed.add_business_days(base, 0)?, base);
    let rare = closed.without_holiday(base.checked_add_days(7)?);
    assert_eq!(
        rare.adjust(base, Bdc::Following)?,
        base.checked_add_days(7)?
    );
    assert_eq!(rare.add_business_days(base, 1)?, base.checked_add_days(7)?);
    Ok(())
}

#[test]
fn modified_searches_do_not_leave_empty_month_and_handle_date_edges() -> Result<(), Box<dyn Error>>
{
    let first = d(2024, 2, 1);
    let all_february: Vec<_> = (0..29)
        .map(|offset| first.checked_add_days(offset))
        .collect::<Result<_, _>>()?;
    let calendar = Calendar::custom([], all_february);
    for convention in [
        Bdc::ModifiedFollowing,
        Bdc::ModifiedPreceding,
        Bdc::HalfMonthModifiedFollowing,
    ] {
        assert!(matches!(
            calendar.adjust(d(2024, 2, 15), convention),
            Err(CalendarError::NoBusinessDayInMonth { .. })
        ));
    }
    assert!(calendar.end_of_month(first).is_err());
    assert!(calendar.start_of_month(first).is_err());
    let closed_min = Calendar::null().with_holiday(Date::MIN);
    assert!(matches!(
        closed_min.adjust(Date::MIN, Bdc::Preceding),
        Err(CalendarError::Date(_))
    ));
    assert_eq!(
        closed_min.adjust(Date::MIN, Bdc::ModifiedPreceding)?,
        Date::MIN.checked_add_days(1)?
    );
    assert_eq!(
        closed_min.adjust(Date::MIN, Bdc::Nearest)?,
        Date::MIN.checked_add_days(1)?
    );
    let closed_max = Calendar::null().with_holiday(Date::MAX);
    assert!(matches!(
        closed_max.adjust(Date::MAX, Bdc::Following),
        Err(CalendarError::Date(_))
    ));
    assert_eq!(
        closed_max.adjust(Date::MAX, Bdc::ModifiedFollowing)?,
        Date::MAX.checked_sub_days(1)?
    );
    assert_eq!(
        closed_max.adjust(Date::MAX, Bdc::Nearest)?,
        Date::MAX.checked_sub_days(1)?
    );
    Ok(())
}

#[test]
fn business_day_addition_ignores_start_holiday_and_resets_run_bound() -> Result<(), Box<dyn Error>>
{
    let calendar = Calendar::weekends_only();
    assert_eq!(
        calendar.add_business_days(d(2024, 6, 15), 1)?,
        d(2024, 6, 17)
    );
    assert_eq!(
        calendar.add_business_days(d(2024, 6, 15), -1)?,
        d(2024, 6, 14)
    );
    assert_eq!(
        calendar.add_business_days(d(2024, 6, 15), 0)?,
        d(2024, 6, 15)
    );
    for count in -30..=30 {
        let start = d(2024, 6, 17);
        let result = calendar.add_business_days(start, count)?;
        assert_eq!(calendar.add_business_days(result, -count)?, start);
        if count != 0 {
            assert!(calendar.is_business_day(result));
        }
    }
    let base = d(2020, 1, 1);
    let holidays: Vec<_> = (1..=365)
        .chain(367..=731)
        .map(|offset| base.checked_add_days(offset))
        .collect::<Result<_, _>>()?;
    let long_runs = Calendar::custom([], holidays);
    assert_eq!(
        long_runs.add_business_days(base, 2)?,
        base.checked_add_days(732)?
    );
    for count in [i32::MIN, i32::MAX] {
        assert!(matches!(
            calendar.add_business_days(base, count),
            Err(CalendarError::Date(_))
        ));
    }
    assert!(calendar.add_business_days(Date::MIN, -1).is_err());
    assert!(calendar.add_business_days(Date::MAX, 1).is_err());
    Ok(())
}

#[test]
fn advancement_distinguishes_business_days_calendar_weeks_and_zero_adjustment()
-> Result<(), CalendarError> {
    let calendar = Calendar::weekends_only().with_holiday(d(2024, 4, 2));
    let start = d(2024, 3, 29);
    let days = calendar.advance(
        start,
        Period::new(7, TimeUnit::Days),
        Bdc::Unadjusted,
        EndOfMonth::Ignore,
    )?;
    let weeks = calendar.advance(
        start,
        Period::new(1, TimeUnit::Weeks),
        Bdc::Unadjusted,
        EndOfMonth::Ignore,
    )?;
    assert_eq!(days, d(2024, 4, 10));
    assert_eq!(weeks, d(2024, 4, 5));
    let holiday = d(2024, 3, 30);
    for unit in TimeUnit::ALL {
        assert_eq!(
            calendar.advance(
                holiday,
                Period::new(0, unit),
                Bdc::Following,
                EndOfMonth::Preserve
            )?,
            d(2024, 4, 1)
        );
    }
    for convention in Bdc::ALL {
        assert_eq!(
            calendar.advance(
                holiday,
                Period::new(1, TimeUnit::Days),
                convention,
                EndOfMonth::Preserve
            )?,
            d(2024, 4, 1)
        );
    }
    Ok(())
}

#[test]
fn equal_tenors_keep_their_stored_business_day_or_calendar_week_dispatch()
-> Result<(), CalendarError> {
    let calendar = Calendar::weekends_only();
    let start = d(2026, 1, 2);
    let business_days = Period::new(7, TimeUnit::Days);
    let calendar_week = Period::new(1, TimeUnit::Weeks);
    assert_eq!(business_days, calendar_week);
    assert_eq!(
        start.checked_add_period(business_days)?,
        start.checked_add_period(calendar_week)?
    );
    for convention in Bdc::ALL {
        for end_of_month in EndOfMonth::ALL {
            assert_eq!(
                calendar.advance(start, business_days, convention, end_of_month)?,
                d(2026, 1, 13)
            );
            assert_eq!(
                calendar.advance(start, calendar_week, convention, end_of_month)?,
                d(2026, 1, 9)
            );
        }
    }
    Ok(())
}

#[test]
fn eom_distinguishes_unadjusted_calendar_eom_from_adjusted_business_eom()
-> Result<(), CalendarError> {
    let calendar = Calendar::weekends_only()
        .with_holiday(d(2024, 3, 29))
        .with_holiday(d(2024, 4, 1));
    let last_business = d(2024, 3, 28);
    assert_eq!(calendar.end_of_month(last_business)?, last_business);
    assert!(calendar.is_end_of_month(last_business)?);
    assert!(calendar.is_end_of_month(d(2024, 3, 29))?);
    assert!(!calendar.is_end_of_month(d(2024, 3, 27))?);
    for (months, unadjusted, adjusted) in [
        (-2, d(2024, 1, 28), d(2024, 1, 31)),
        (1, d(2024, 4, 28), d(2024, 4, 30)),
    ] {
        assert_eq!(
            calendar.advance(
                last_business,
                Period::new(months, TimeUnit::Months),
                Bdc::Unadjusted,
                EndOfMonth::Preserve
            )?,
            unadjusted
        );
        for convention in Bdc::ALL.into_iter().filter(|c| *c != Bdc::Unadjusted) {
            assert_eq!(
                calendar.advance(
                    last_business,
                    Period::new(months, TimeUnit::Months),
                    convention,
                    EndOfMonth::Preserve
                )?,
                adjusted
            );
        }
    }
    assert_eq!(
        calendar.advance(
            d(2024, 3, 31),
            Period::new(1, TimeUnit::Months),
            Bdc::Unadjusted,
            EndOfMonth::Preserve
        )?,
        d(2024, 4, 30)
    );
    assert_eq!(
        calendar.advance(
            d(2024, 2, 29),
            Period::new(1, TimeUnit::Months),
            Bdc::Unadjusted,
            EndOfMonth::Ignore
        )?,
        d(2024, 3, 29)
    );
    assert_eq!(
        calendar.advance(
            d(2024, 2, 29),
            Period::new(1, TimeUnit::Months),
            Bdc::Unadjusted,
            EndOfMonth::Preserve
        )?,
        d(2024, 3, 31)
    );
    let closed = Calendar::custom(Weekday::ALL, []);
    assert_eq!(
        closed.advance(
            d(2024, 3, 28),
            Period::new(1, TimeUnit::Months),
            Bdc::Unadjusted,
            EndOfMonth::Preserve
        )?,
        d(2024, 4, 28)
    );
    Ok(())
}

#[test]
fn endpoint_counting_matches_explicit_lists_for_every_small_pair_and_flag()
-> Result<(), Box<dyn Error>> {
    let start = d(2024, 2, 20);
    let calendars = [
        Calendar::null(),
        Calendar::weekends_only(),
        Calendar::custom(
            [Weekday::Monday, Weekday::Sunday],
            [d(2024, 2, 29), d(2024, 3, 1)],
        ),
        Calendar::custom(Weekday::ALL, []),
    ];
    for calendar in calendars {
        for a in 0..20 {
            for b in 0..20 {
                let left = start.checked_add_days(a)?;
                let right = start.checked_add_days(b)?;
                for include_left in [false, true] {
                    for include_right in [false, true] {
                        let expected = if a == b {
                            i32::from(
                                include_left && include_right && calendar.is_business_day(left),
                            )
                        } else {
                            let mut count = 0;
                            for offset in a.min(b)..=a.max(b) {
                                if (offset != a || include_left)
                                    && (offset != b || include_right)
                                    && calendar.is_business_day(start.checked_add_days(offset)?)
                                {
                                    count += 1;
                                }
                            }
                            if a > b { -count } else { count }
                        };
                        assert_eq!(
                            calendar.business_days_between(
                                left,
                                right,
                                endpoint_option(include_left),
                                endpoint_option(include_right)
                            ),
                            expected
                        );
                        if a != b {
                            assert_eq!(
                                calendar.business_days_between(
                                    right,
                                    left,
                                    endpoint_option(include_right),
                                    endpoint_option(include_left)
                                ),
                                -expected
                            );
                        }
                    }
                }
            }
        }
    }
    // At equal endpoints the signed-reversal property is intentionally
    // replaced by the explicit both-included same-date rule.
    assert_eq!(
        Calendar::null().business_days_between(
            Date::MIN,
            Date::MIN,
            EndpointInclusion::Include,
            EndpointInclusion::Include
        ),
        1
    );
    assert_eq!(
        Calendar::null().business_days_between(
            Date::MIN,
            Date::MAX,
            EndpointInclusion::Include,
            EndpointInclusion::Include
        ),
        3_652_059
    );
    Ok(())
}

#[test]
fn inclusive_lists_distinguish_weekends_and_explicit_business_overrides()
-> Result<(), CalendarError> {
    let saturday = d(2024, 6, 15);
    let sunday = d(2024, 6, 16);
    let monday = d(2024, 6, 17);
    let calendar = Calendar::weekends_only()
        .without_holiday(saturday)
        .with_holiday(monday);
    assert_eq!(
        calendar.business_day_list(saturday, monday)?,
        vec![saturday]
    );
    assert_eq!(
        calendar.holiday_list(saturday, monday, HolidayWeekendInclusion::Include)?,
        vec![sunday, monday]
    );
    assert_eq!(
        calendar.holiday_list(saturday, monday, HolidayWeekendInclusion::Exclude)?,
        vec![monday]
    );
    assert_eq!(
        calendar.business_day_list(saturday, saturday)?,
        vec![saturday]
    );
    assert_eq!(
        calendar.holiday_list(sunday, sunday, HolidayWeekendInclusion::Include)?,
        vec![sunday]
    );
    for error in [
        calendar.business_day_list(monday, saturday),
        calendar.holiday_list(monday, saturday, HolidayWeekendInclusion::Include),
    ] {
        assert!(
            matches!(error,Err(CalendarError::InvalidRange { start,end,.. }) if start==monday && end==saturday)
        );
    }
    Ok(())
}

#[test]
fn joint_nesting_is_bounded_without_suppressing_explicit_override_semantics()
-> Result<(), CalendarError> {
    assert!(matches!(
        Calendar::joint(vec![], JointRule::JoinHolidays),
        Err(CalendarError::EmptyJoint)
    ));
    let mut calendar = Calendar::null();
    for depth in 1..=Calendar::MAX_JOINT_DEPTH {
        let rule = if depth % 2 == 0 {
            JointRule::JoinHolidays
        } else {
            JointRule::JoinBusinessDays
        };
        calendar = Calendar::joint(vec![calendar], rule)?;
    }
    let clone = calendar.clone();
    assert_eq!(clone, calendar);
    assert!(!format!("{calendar:?}").is_empty());
    assert!(calendar.is_business_day(Date::MIN));
    assert!(matches!(
        Calendar::joint(vec![calendar], JointRule::JoinBusinessDays),
        Err(CalendarError::NestingLimit {
            depth: 65,
            maximum: 64,
            ..
        })
    ));
    // Hundreds of same-rule combinations flatten rather than consume depth.
    let mut flat = Calendar::null();
    for _ in 0..1000 {
        flat = Calendar::joint(
            vec![flat, Calendar::weekends_only()],
            JointRule::JoinHolidays,
        )?;
    }
    assert!(!flat.is_business_day(d(2024, 6, 15)));
    Ok(())
}

// The independent fixture/oracle uses booleans; the public API uses named choices.
fn endpoint_option(include: bool) -> EndpointInclusion {
    if include {
        EndpointInclusion::Include
    } else {
        EndpointInclusion::Exclude
    }
}
