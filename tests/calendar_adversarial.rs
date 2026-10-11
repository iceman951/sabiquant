//! Coverage-guided public calendar cases: identifiers, errors, rare openings
//! and range boundaries (FR-CORE-05/06, FR-CAL-01/03/05/06, TS-CAL-07).

use sabiquant::calendar::{BusinessDayConvention as Bdc, Calendar, CalendarError, JointRule};
use sabiquant::time::{Date, Period, TimeUnit, Weekday};

#[expect(
    clippy::unwrap_used,
    reason = "the helper accepts only fixed valid date literals in these tests"
)]
fn date(text: &str) -> Date {
    text.parse().unwrap()
}

#[test]
fn every_calendar_enum_formats_and_parses_exactly_and_rejects_lookalikes() {
    for value in Bdc::ALL {
        let text = value.to_string();
        assert_eq!(text, value.canonical_identifier());
        assert_eq!(text.parse::<Bdc>().unwrap(), value);
        for bad in [
            format!(" {text}"),
            format!("{text} "),
            text.to_lowercase(),
            format!("{text}\0"),
            format!("{text}🙂"),
        ] {
            assert_eq!(bad.parse::<Bdc>().unwrap_err().kind(), "identifier.unknown");
        }
    }
    for value in JointRule::ALL {
        let text = value.to_string();
        assert_eq!(text, value.canonical_identifier());
        assert_eq!(text.parse::<JointRule>().unwrap(), value);
        for bad in [
            format!(" {text}"),
            format!("{text} "),
            text.to_lowercase(),
            format!("{text}\0"),
        ] {
            assert_eq!(
                bad.parse::<JointRule>().unwrap_err().kind(),
                "identifier.unknown"
            );
        }
    }
    for input in [
        "",
        "FOLLOWING",
        "Modified Following",
        "Join",
        "JoinBusiness",
        "\u{feff}Nearest",
        "ถัดไป",
    ] {
        assert!(input.parse::<Bdc>().is_err());
        assert!(input.parse::<JointRule>().is_err());
    }
}

#[test]
fn business_month_predicates_include_closed_dates_before_first_and_after_last() {
    let calendar = Calendar::weekends_only();
    let holidays = [
        "2024-02-01",
        "2024-02-02",
        "2024-02-27",
        "2024-02-28",
        "2024-02-29",
    ];
    let calendar = holidays
        .into_iter()
        .fold(calendar, |value, text| value.with_holiday(date(text)));
    for day in 1..=29 {
        let current = Date::from_ymd(2024, 2, day).unwrap();
        assert_eq!(
            calendar.start_of_month(current).unwrap(),
            date("2024-02-05")
        );
        assert_eq!(calendar.end_of_month(current).unwrap(), date("2024-02-26"));
        assert_eq!(calendar.is_start_of_month(current).unwrap(), day <= 5);
        assert_eq!(calendar.is_end_of_month(current).unwrap(), day >= 26);
    }
    let closed = Calendar::custom(Weekday::ALL, []);
    assert_eq!(
        closed
            .is_start_of_month(date("2024-02-15"))
            .unwrap_err()
            .kind(),
        "calendar.no_business_day_in_month"
    );
    assert_eq!(
        closed
            .is_end_of_month(date("2024-02-15"))
            .unwrap_err()
            .kind(),
        "calendar.no_business_day_in_month"
    );
    assert!(
        closed
            .advance(
                date("2024-02-15"),
                Period::new(1, TimeUnit::Months),
                Bdc::Following,
                true
            )
            .is_err()
    );
}

#[test]
fn nested_closed_calendars_do_not_hide_member_business_overrides() {
    let start = date("2024-01-01");
    let open = start.checked_add_days(7).unwrap();
    let closed = Calendar::custom(Weekday::ALL, []);
    let exceptional = closed.without_holiday(open);
    for rule in JointRule::ALL {
        let other = if rule == JointRule::JoinHolidays {
            Calendar::null()
        } else {
            closed.clone()
        };
        let joint = Calendar::joint(vec![exceptional.clone(), other], rule).unwrap();
        assert!(joint.is_business_day(open));
        assert_eq!(joint.adjust(start, Bdc::Following).unwrap(), open);
        assert_eq!(joint.add_business_days(start, 1).unwrap(), open);
        assert!(!closed.is_business_day(open));
    }
}

#[test]
fn nearest_reports_failure_when_both_sides_exceed_bounds_or_date_range() {
    let center = date("2024-06-01");
    let closed = Calendar::custom(Weekday::ALL, []);
    let rare = closed.without_holiday(center.checked_add_days(500).unwrap());
    assert!(matches!(
        rare.adjust(center, Bdc::Nearest),
        Err(CalendarError::SearchLimit {
            direction: 1,
            rejected: 366,
            ..
        })
    ));
    let rare = closed.without_holiday(Date::MIN.checked_add_days(500).unwrap());
    assert!(matches!(
        rare.adjust(Date::MIN, Bdc::Nearest),
        Err(CalendarError::Date(_))
    ));
}

#[test]
fn business_day_steps_test_real_walk_overflow_and_consecutive_rejection_bound() {
    let null = Calendar::null();
    assert_eq!(
        null.add_business_days(Date::MIN, 1).unwrap(),
        Date::MIN.checked_add_days(1).unwrap()
    );
    assert_eq!(
        null.add_business_days(Date::MAX, -1).unwrap(),
        Date::MAX.checked_sub_days(1).unwrap()
    );
    let calendar = null.with_holiday(Date::MAX);
    assert!(matches!(
        calendar.add_business_days(Date::MAX.checked_sub_days(1).unwrap(), 1),
        Err(CalendarError::Date(_))
    ));
    let calendar = null.with_holiday(Date::MIN);
    assert!(matches!(
        calendar.add_business_days(Date::MIN.checked_add_days(1).unwrap(), -1),
        Err(CalendarError::Date(_))
    ));
    let start = date("2024-01-15");
    for direction in [-1_i64, 1] {
        let holidays = (1..=366).map(|offset| start.checked_add_days(direction * offset).unwrap());
        let calendar = Calendar::custom([], holidays);
        assert!(matches!(
            calendar.add_business_days(start, i32::try_from(direction).unwrap()),
            Err(CalendarError::SearchLimit { rejected: 366, .. })
        ));
    }
    let closed = Calendar::custom(Weekday::ALL, []);
    assert!(matches!(
        closed.add_business_days(start, 1),
        Err(CalendarError::NoBusinessWeekday)
    ));
    for unit in TimeUnit::ALL {
        assert!(
            null.advance(Date::MAX, Period::new(i32::MAX, unit), Bdc::Following, true)
                .is_err()
        );
        assert!(
            null.advance(Date::MIN, Period::new(i32::MIN, unit), Bdc::Preceding, true)
                .is_err()
        );
    }
}
