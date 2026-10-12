//! Independent validation for the staged local M3 core-calendar batch.
//!
//! TS-CAL-01..05/07 and TS-BDC-01..05 are checked against real public
//! QuantLib 1.43 calls and separately generated Python membership/scan oracles.
//! No market calendar, Easter rule, or full M3 completion is claimed.

#![expect(
    clippy::expect_used,
    reason = "test helpers fail immediately on invalid committed fixtures or scenario inputs"
)]

mod support;

use sabiquant::calendar::{BusinessDayConvention, Calendar, CalendarError, JointRule};
use sabiquant::calendar::{EndOfMonth, EndpointInclusion, HolidayWeekendInclusion};
use sabiquant::time::{Date, Period, TimeUnit, Weekday};
use support::fixture::Fixture;

fn fixture(name: &str) -> Fixture {
    Fixture::load(Fixture::repository_path(&format!(
        "validation/fixtures/calendar/{name}.fixture"
    )))
    .expect("committed schema-1 calendar fixture")
}

fn date(text: &str) -> Date {
    text.parse().expect("valid scenario date")
}

fn contexts() -> Vec<(&'static str, Calendar)> {
    let holidays = [
        "1999-12-31",
        "2000-01-03",
        "2000-02-29",
        "2000-03-01",
        "2000-06-15",
        "2000-07-03",
        "2000-07-15",
        "2000-07-17",
        "2000-12-25",
        "2000-12-26",
        "2024-02-29",
        "2024-03-29",
        "2024-04-01",
        "2024-07-03",
        "2024-08-30",
        "2024-09-02",
        "2100-02-26",
        "2100-03-01",
        "2100-12-31",
    ];
    let alternate_holidays = [
        "2000-01-03",
        "2000-02-28",
        "2000-07-03",
        "2024-03-28",
        "2024-07-03",
        "2024-09-03",
        "2100-02-25",
        "2100-03-01",
    ];
    let custom = Calendar::custom([Weekday::Saturday, Weekday::Sunday], holidays.map(date));
    let alternative = Calendar::custom(
        [Weekday::Friday, Weekday::Saturday],
        alternate_holidays.map(date),
    );
    let overridden = custom
        .without_holiday(date("2024-02-24"))
        .without_holiday(date("2024-07-03"));
    let joint_holidays = Calendar::joint(
        vec![custom.clone(), alternative.clone()],
        JointRule::JoinHolidays,
    )
    .expect("nonempty scenario joint calendar");
    let joint_business = Calendar::joint(
        vec![custom.clone(), alternative.clone()],
        JointRule::JoinBusinessDays,
    )
    .expect("nonempty scenario joint calendar");
    vec![
        ("null", Calendar::null()),
        ("weekends", Calendar::weekends_only()),
        ("custom", custom),
        ("alternative", alternative),
        ("overridden", overridden),
        ("joint_holidays", joint_holidays),
        ("joint_business", joint_business),
    ]
}

fn context<'a>(contexts: &'a [(&str, Calendar)], name: &str) -> &'a Calendar {
    contexts
        .iter()
        .find(|(id, _)| *id == name)
        .map(|(_, calendar)| calendar)
        .expect("fixture uses a declared scenario calendar")
}

fn date_list(text: &str) -> Vec<Date> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split(',').map(date).collect()
    }
}

#[test]
fn ts_cal_01_02_03_quantlib_membership_and_joint_weekends() {
    let contexts = contexts();
    let data = fixture("core-membership");
    assert_eq!(data.record_count(), 20_454);
    for record in data.records() {
        let calendar = context(&contexts, record.text("calendar").unwrap());
        let date: Date = record.parse("date").unwrap();
        assert_eq!(
            calendar.is_business_day(date),
            record.parse::<u32>("business").unwrap() == 1,
            "{}",
            record.id()
        );
        assert_eq!(
            calendar.is_holiday(date),
            record.parse::<u32>("holiday").unwrap() == 1,
            "{}",
            record.id()
        );
        assert_eq!(
            calendar.is_weekend(date),
            record.parse::<u32>("weekend").unwrap() == 1,
            "{}",
            record.id()
        );
        assert_ne!(calendar.is_holiday(date), calendar.is_business_day(date));
    }
}

#[test]
fn ts_bdc_01_05_all_seven_adjustments_match_dense_quantlib_windows() {
    let contexts = contexts();
    let data = fixture("core-adjustments");
    assert_eq!(data.record_count(), 15_141);
    for record in data.records() {
        let calendar = context(&contexts, record.text("calendar").unwrap());
        let date: Date = record.parse("date").unwrap();
        let convention: BusinessDayConvention = record.parse("convention").unwrap();
        let expected: Date = record.parse("result").unwrap();
        let actual = calendar.adjust(date, convention).unwrap();
        assert_eq!(actual, expected, "{}", record.id());
        assert_eq!(
            calendar.adjust(actual, convention).unwrap(),
            actual,
            "{}",
            record.id()
        );
        if calendar.is_business_day(date) || convention == BusinessDayConvention::Unadjusted {
            assert_eq!(actual, date);
        } else {
            assert!(calendar.is_business_day(actual));
        }
    }
}

#[test]
fn ts_cal_04_all_units_signs_conventions_and_eom_flags_match_quantlib() {
    let contexts = contexts();
    let data = fixture("core-advancement");
    assert_eq!(data.record_count(), 23_520);
    for record in data.records() {
        let calendar = context(&contexts, record.text("calendar").unwrap());
        let date: Date = record.parse("date").unwrap();
        let length: i32 = record.parse("count").unwrap();
        let unit: TimeUnit = record.parse("unit").unwrap();
        let convention: BusinessDayConvention = record.parse("convention").unwrap();
        let eom = record.parse::<u32>("end_of_month").unwrap() == 1;
        let expected: Date = record.parse("result").unwrap();
        assert_eq!(
            calendar
                .advance(
                    date,
                    Period::new(length, unit),
                    convention,
                    end_of_month_option(eom)
                )
                .unwrap(),
            expected,
            "{}",
            record.id()
        );
        if unit == TimeUnit::Days && length != 0 {
            assert_eq!(
                calendar.add_business_days(date, length).unwrap(),
                expected,
                "{}",
                record.id()
            );
            assert!(calendar.is_business_day(expected));
            assert_eq!(expected > date, length > 0);
        }
    }
}

#[test]
fn ts_cal_05_all_endpoint_options_both_directions_and_equal_dates_match_quantlib() {
    let contexts = contexts();
    let data = fixture("core-counts");
    assert_eq!(data.record_count(), 12_096);
    for record in data.records() {
        let calendar = context(&contexts, record.text("calendar").unwrap());
        let start: Date = record.parse("start").unwrap();
        let end: Date = record.parse("end").unwrap();
        let include_start = record.parse::<u32>("include_start").unwrap() == 1;
        let include_end = record.parse::<u32>("include_end").unwrap() == 1;
        let expected: i32 = record.parse("result").unwrap();
        assert_eq!(
            calendar.business_days_between(
                start,
                end,
                endpoint_option(include_start),
                endpoint_option(include_end)
            ),
            expected,
            "{}",
            record.id()
        );
        if start != end {
            assert_eq!(
                calendar.business_days_between(
                    end,
                    start,
                    endpoint_option(include_end),
                    endpoint_option(include_start)
                ),
                -expected
            );
        }
    }
}

#[test]
fn fr_cal_01_business_month_end_matches_quantlib_including_later_holidays() {
    let contexts = contexts();
    let data = fixture("core-month-edges");
    assert_eq!(data.record_count(), 2163);
    for record in data.records() {
        let calendar = context(&contexts, record.text("calendar").unwrap());
        let date: Date = record.parse("date").unwrap();
        let expected: Date = record.parse("end_of_month").unwrap();
        assert_eq!(
            calendar.end_of_month(date).unwrap(),
            expected,
            "{}",
            record.id()
        );
        assert_eq!(
            calendar.is_end_of_month(date).unwrap(),
            record.parse::<u32>("is_end_of_month").unwrap() == 1,
            "{}",
            record.id()
        );
        assert!(calendar.is_business_day(expected));
        assert_eq!(
            (expected.year(), expected.month()),
            (date.year(), date.month())
        );
    }
}

#[test]
fn fr_cal_01_inclusive_holiday_and_business_lists_match_quantlib() {
    let contexts = contexts();
    let data = fixture("core-lists");
    assert_eq!(data.record_count(), 56);
    for record in data.records() {
        let calendar = context(&contexts, record.text("calendar").unwrap());
        let start = record.parse("start").unwrap();
        let end = record.parse("end").unwrap();
        let include_weekends = record.parse::<u32>("include_weekends").unwrap() == 1;
        let holidays = calendar
            .holiday_list(start, end, holiday_weekend_option(include_weekends))
            .unwrap();
        let business = calendar.business_day_list(start, end).unwrap();
        assert_eq!(
            holidays,
            date_list(record.text("holidays").unwrap()),
            "{}",
            record.id()
        );
        assert_eq!(
            business,
            date_list(record.text("business_days").unwrap()),
            "{}",
            record.id()
        );
        assert!(holidays.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(business.windows(2).all(|pair| pair[0] < pair[1]));
    }
}

fn mask_calendar(mask: u32) -> Calendar {
    Calendar::custom(
        Weekday::ALL
            .into_iter()
            .filter(|weekday| mask & (1 << (weekday.number_from_monday() - 1)) != 0),
        [],
    )
}

#[test]
fn ts_cal_01_all_128_weekend_masks_match_public_quantlib_membership() {
    let data = fixture("weekend-masks");
    assert_eq!(data.record_count(), 1792);
    let calendars: Vec<_> = (0..128).map(mask_calendar).collect();
    for record in data.records() {
        let mask: usize = record.parse("weekend_mask").unwrap();
        let date = record.parse("date").unwrap();
        assert_eq!(
            calendars[mask].is_business_day(date),
            record.parse::<u32>("business").unwrap() == 1,
            "{}",
            record.id()
        );
    }
}

fn independent_mask_count(
    mask: u32,
    start: i64,
    end: i64,
    include_start: bool,
    include_end: bool,
) -> i32 {
    if start > end {
        return -independent_mask_count(mask, end, start, include_end, include_start);
    }
    let business = |offset: i64| mask & (1 << ((offset + 2) % 7)) == 0; // 2000-03-01 Wednesday.
    if start == end {
        return i32::from(include_start && include_end && business(start));
    }
    i32::try_from(
        (start..=end)
            .filter(|&offset| {
                (offset != start || include_start)
                    && (offset != end || include_end)
                    && business(offset)
            })
            .count(),
    )
    .expect("bounded independent calendar count")
}

#[test]
fn ts_cal_05_independent_counting_all_masks_and_100352_endpoint_cases() {
    let origin = date("2000-03-01");
    for mask in 0..128 {
        let calendar = mask_calendar(mask);
        for start in 0..14 {
            for end in 0..14 {
                for include_start in [false, true] {
                    for include_end in [false, true] {
                        let expected =
                            independent_mask_count(mask, start, end, include_start, include_end);
                        assert_eq!(
                            calendar.business_days_between(
                                origin.checked_add_days(start).unwrap(),
                                origin.checked_add_days(end).unwrap(),
                                endpoint_option(include_start),
                                endpoint_option(include_end)
                            ),
                            expected,
                            "mask={mask}, {start}..{end}, endpoints={include_start}/{include_end}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn regression_unadjusted_eom_requires_calendar_end_and_days_zero_is_identity() {
    let calendar = Calendar::custom(
        [Weekday::Saturday, Weekday::Sunday],
        [date("2024-03-29"), date("2024-04-01")],
    );
    let business_end = date("2024-03-28");
    assert!(calendar.is_end_of_month(business_end).unwrap());
    assert_eq!(
        calendar
            .advance(
                business_end,
                Period::new(-2, TimeUnit::Months),
                BusinessDayConvention::Unadjusted,
                EndOfMonth::Preserve
            )
            .unwrap(),
        date("2024-01-28")
    );
    assert_eq!(
        calendar
            .advance(
                business_end,
                Period::new(-2, TimeUnit::Months),
                BusinessDayConvention::Following,
                EndOfMonth::Preserve
            )
            .unwrap(),
        date("2024-01-31")
    );
    assert_eq!(
        calendar
            .advance(
                date("2024-03-31"),
                Period::new(-2, TimeUnit::Months),
                BusinessDayConvention::Unadjusted,
                EndOfMonth::Preserve
            )
            .unwrap(),
        date("2024-01-31")
    );
    let holiday = date("2024-03-29");
    assert_eq!(calendar.add_business_days(holiday, 0).unwrap(), holiday);
    assert_eq!(
        calendar
            .advance(
                holiday,
                Period::new(0, TimeUnit::Days),
                BusinessDayConvention::Following,
                EndOfMonth::Ignore
            )
            .unwrap(),
        date("2024-04-02")
    );
}

#[test]
fn closed_weekend_mask_can_be_opened_by_an_immutable_dated_override() {
    let original = mask_calendar(127);
    let event = date("2024-01-03");
    let changed = original.without_holiday(event);
    assert!(!original.is_business_day(event));
    assert!(changed.is_business_day(event));
    assert!(changed.is_weekend(event));
    assert_eq!(
        changed
            .adjust(event, BusinessDayConvention::Following)
            .unwrap(),
        event
    );
    assert_eq!(
        changed.add_business_days(date("2024-01-02"), 1).unwrap(),
        event
    );
    assert_eq!(
        changed.add_business_days(date("2024-01-04"), -1).unwrap(),
        event
    );
    assert_eq!(
        changed.business_days_between(
            date("2024-01-01"),
            date("2024-01-05"),
            EndpointInclusion::Include,
            EndpointInclusion::Include
        ),
        1
    );
    assert_eq!(
        original
            .adjust(event, BusinessDayConvention::Following)
            .unwrap_err()
            .kind(),
        "calendar.no_business_weekday"
    );
    assert_eq!(
        changed
            .adjust(date("2024-01-04"), BusinessDayConvention::Following)
            .unwrap_err()
            .kind(),
        "calendar.search_limit"
    );
}

#[test]
fn public_calendar_values_and_errors_are_thread_safe_without_shared_mutation() {
    fn value<T: Clone + std::fmt::Debug + Send + Sync + 'static>() {}
    fn error<T: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    value::<Calendar>();
    value::<BusinessDayConvention>();
    value::<JointRule>();
    error::<CalendarError>();
}

#[test]
fn ts_cal_07_exact_365_366_367_rejection_boundary_in_both_directions() {
    let origin = date("2024-01-01");
    for direction in [-1_i64, 1] {
        for rejected in [365_i64, 366, 367] {
            let holidays =
                (0..rejected).map(|offset| origin.checked_add_days(direction * offset).unwrap());
            let calendar = Calendar::custom([], holidays);
            let convention = if direction > 0 {
                BusinessDayConvention::Following
            } else {
                BusinessDayConvention::Preceding
            };
            let result = calendar.adjust(origin, convention);
            let stepping = calendar.add_business_days(
                origin.checked_add_days(-direction).unwrap(),
                i32::try_from(direction).unwrap(),
            );
            if rejected == 365 {
                let expected = origin.checked_add_days(direction * rejected).unwrap();
                assert_eq!(result.unwrap(), expected);
                assert_eq!(stepping.unwrap(), expected);
            } else {
                assert_eq!(result.unwrap_err().kind(), "calendar.search_limit");
                assert_eq!(stepping.unwrap_err().kind(), "calendar.search_limit");
            }
        }
    }
    // Two individually admissible holiday runs remain admissible after the
    // first business day resets the consecutive-rejection counter.
    let calendar = Calendar::custom(
        [],
        (0..732)
            .filter(|&offset| offset != 365 && offset != 731)
            .map(|offset| origin.checked_add_days(offset).unwrap()),
    );
    assert_eq!(
        calendar
            .add_business_days(origin.checked_sub_days(1).unwrap(), 2)
            .unwrap(),
        origin.checked_add_days(731).unwrap()
    );
    assert_eq!(
        calendar
            .add_business_days(origin.checked_add_days(732).unwrap(), -2)
            .unwrap(),
        origin.checked_add_days(365).unwrap()
    );
}

#[test]
fn ts_cal_07_empty_month_range_edges_and_invalid_lists_return_typed_errors() {
    let february = Calendar::custom(
        [],
        (1..=29).map(|day| Date::from_ymd(2024, 2, day).unwrap()),
    );
    for convention in [
        BusinessDayConvention::ModifiedFollowing,
        BusinessDayConvention::ModifiedPreceding,
        BusinessDayConvention::HalfMonthModifiedFollowing,
    ] {
        assert_eq!(
            february
                .adjust(date("2024-02-15"), convention)
                .unwrap_err()
                .kind(),
            "calendar.no_business_day_in_month"
        );
    }
    assert_eq!(
        february
            .end_of_month(date("2024-02-15"))
            .unwrap_err()
            .kind(),
        "calendar.no_business_day_in_month"
    );
    for boundary in [Date::MIN, Date::MAX] {
        let calendar = Calendar::null().with_holiday(boundary);
        let outward = if boundary == Date::MIN {
            BusinessDayConvention::Preceding
        } else {
            BusinessDayConvention::Following
        };
        assert_eq!(
            calendar.adjust(boundary, outward).unwrap_err().kind(),
            "calendar.date"
        );
        let expected = boundary
            .checked_add_days(if boundary == Date::MIN { 1 } else { -1 })
            .unwrap();
        assert_eq!(
            calendar
                .adjust(boundary, BusinessDayConvention::Nearest)
                .unwrap(),
            expected
        );
    }
    let calendar = Calendar::weekends_only();
    for length in [i32::MIN, i32::MAX] {
        for unit in TimeUnit::ALL {
            assert_eq!(
                calendar
                    .advance(
                        date("2024-06-15"),
                        Period::new(length, unit),
                        BusinessDayConvention::Following,
                        EndOfMonth::Preserve
                    )
                    .unwrap_err()
                    .kind(),
                "calendar.date"
            );
        }
    }
    assert_eq!(
        calendar
            .holiday_list(Date::MAX, Date::MIN, HolidayWeekendInclusion::Include)
            .unwrap_err()
            .kind(),
        "calendar.invalid_range"
    );
    assert_eq!(
        calendar
            .business_day_list(Date::MAX, Date::MIN)
            .unwrap_err()
            .kind(),
        "calendar.invalid_range"
    );
}

#[test]
fn finite_reference_search_divergences_have_exact_inputs_and_typed_errors() {
    let data = fixture("bounded-divergences");
    let mut empty = 0;
    let mut bounded = 0;
    for row in data.records() {
        let start = row.parse::<Date>("date").unwrap();
        let convention = row.parse::<BusinessDayConvention>("convention").unwrap();
        let reference = row.parse::<Date>("reference_result").unwrap();
        let (calendar, expected_kind) = match row.text("discrepancy").unwrap() {
            "D9-CALENDAR-EMPTY-MONTH" => {
                empty += 1;
                assert_eq!(start, date("2024-02-15"));
                assert!(matches!(
                    convention,
                    BusinessDayConvention::ModifiedFollowing
                        | BusinessDayConvention::ModifiedPreceding
                        | BusinessDayConvention::HalfMonthModifiedFollowing
                ));
                let expected_reference = if convention == BusinessDayConvention::ModifiedPreceding {
                    date("2024-03-01")
                } else {
                    date("2024-01-31")
                };
                assert_eq!(reference, expected_reference);
                let first = date("2024-02-01");
                let holidays = (0..29).map(|offset| first.checked_add_days(offset).unwrap());
                (
                    Calendar::custom([], holidays),
                    "calendar.no_business_day_in_month",
                )
            }
            "D9-CALENDAR-SEARCH-BOUND" => {
                bounded += 1;
                assert_eq!(start, date("2024-01-15"));
                let direction = match convention {
                    BusinessDayConvention::Following => 1,
                    BusinessDayConvention::Preceding => -1,
                    _ => panic!("unexpected search-bound convention"),
                };
                assert_eq!(reference, start.checked_add_days(direction * 367).unwrap());
                let holidays =
                    (0..367).map(|offset| start.checked_add_days(direction * offset).unwrap());
                (Calendar::custom([], holidays), "calendar.search_limit")
            }
            label => panic!("unexpected calendar discrepancy {label:?}"),
        };
        assert_eq!(row.text("expected_error").unwrap(), expected_kind);
        assert_eq!(
            calendar.adjust(start, convention).unwrap_err().kind(),
            expected_kind
        );
    }
    assert_eq!(empty, 3);
    assert_eq!(bounded, 2);
}

// The independent fixture/oracle uses booleans; the public API uses named choices.
fn end_of_month_option(include: bool) -> EndOfMonth {
    if include {
        EndOfMonth::Preserve
    } else {
        EndOfMonth::Ignore
    }
}

// The independent fixture/oracle uses booleans; the public API uses named choices.
fn endpoint_option(include: bool) -> EndpointInclusion {
    if include {
        EndpointInclusion::Include
    } else {
        EndpointInclusion::Exclude
    }
}

// The independent fixture/oracle uses booleans; the public API uses named choices.
fn holiday_weekend_option(include: bool) -> HolidayWeekendInclusion {
    if include {
        HolidayWeekendInclusion::Include
    } else {
        HolidayWeekendInclusion::Exclude
    }
}
