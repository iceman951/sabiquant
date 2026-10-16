//! Exact reference-formula checks for staged M3 day-count extensions.
//!
//! Sources: ISDA 2006 §4.16(f,h), and B3's business-day/252 basis; see
//! docs/M3_DAYCOUNT_EXTENSIONS.md. Expected values use integer civil counters
//! and rational arithmetic, with TOL-E1 (zero ULP) for single divisions and
//! TOL-E2 (four ULP) for ISDA's composite formula. Intentional reversed-date
//! and equal-termination differences are tested explicitly.

#![expect(
    clippy::expect_used,
    reason = "independent reference helpers fail immediately on broken test inputs"
)]

mod support;

use sabiquant::calendar::{
    Calendar, CalendarError, EndpointInclusion, JointRule, TargetCalendar, TargetCoverage,
    TargetPolicy,
};
use sabiquant::daycount::{Business252, DayCount, DayCountError, ThirtyE360Isda};
use sabiquant::error::{Error, IdentifierError};
use sabiquant::time::{Date, DateError, Weekday};
use std::collections::{HashSet, hash_map::DefaultHasher};
use std::error::Error as StdError;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use support::float::{Tolerance, assert_close};
use support::prng;

fn date(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid test date")
}

fn leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn month_lengths(year: i32) -> [u32; 12] {
    [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ]
}

#[derive(Clone, Copy, Debug)]
struct Civil {
    date: Date,
    year: i32,
    month: i32,
    day: i32,
    ordinal: i32,
    weight: i128,
    last_february: bool,
}

impl Civil {
    fn new(date: Date) -> Self {
        let (year, month, day) = date.to_ymd();
        let lengths = month_lengths(year);
        let before_month = lengths[..usize::try_from(month - 1).expect("month index")]
            .iter()
            .sum::<u32>();
        let offset = i32::try_from(before_month + day - 1).expect("day of year");
        let completed = year - 1;
        let ordinal = completed * 365 + completed / 4 - completed / 100 + completed / 400 + offset;
        // An entire calendar year contributes exactly 365*366 weighted
        // integer units. Partial days use 365 for leap years, 366 otherwise.
        let weight = i128::from(completed) * 365 * 366
            + i128::from(offset) * if leap(year) { 365 } else { 366 };
        Self {
            date,
            year,
            month: i32::try_from(month).expect("month fits"),
            day: i32::try_from(day).expect("day fits"),
            ordinal,
            weight,
            last_february: month == 2 && day == lengths[1],
        }
    }

    fn european(self) -> i32 {
        self.year * 360 + (self.month - 1) * 30 + self.day.min(30) - 1
    }
}

fn adjusted_reference(start: Civil, end: Civil, first: i32, last: i32) -> i32 {
    360 * (end.year - start.year) + 30 * (end.month - start.month) + last - first
}

fn direct_bond_reference(start: Civil, end: Civil) -> i32 {
    let first = start.day.min(30);
    let last = if first == 30 && end.day == 31 {
        30
    } else {
        end.day
    };
    adjusted_reference(start, end, first, last)
}

fn bond_reference(start: Civil, end: Civil) -> i32 {
    if start.date > end.date {
        -direct_bond_reference(end, start)
    } else {
        direct_bond_reference(start, end)
    }
}

fn isda_thirty_reference(start: Civil, end: Civil, termination: Date) -> i32 {
    if start.date == end.date {
        return 0;
    }
    if start.date > end.date {
        return -isda_thirty_reference(end, start, termination);
    }
    let first = if start.day == 31 || start.last_february {
        30
    } else {
        start.day
    };
    let last = if end.day == 31 || (end.last_february && end.date != termination) {
        30
    } else {
        end.day
    };
    adjusted_reference(start, end, first, last)
}

fn hash<T: Hash>(value: &T) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

fn check_scalar_pair(start: Civil, end: Civil) {
    let actual = end.ordinal - start.ordinal;
    for (convention, count, expected, tolerance) in [
        (
            DayCount::Actual360,
            actual,
            f64::from(actual) / 360.0,
            Tolerance::Ulps(0),
        ),
        (
            DayCount::Actual365Fixed,
            actual,
            f64::from(actual) / 365.0,
            Tolerance::Ulps(0),
        ),
        (
            DayCount::ActualActualIsda,
            actual,
            f64::from(i32::try_from(end.weight - start.weight).expect("bounded exact numerator"))
                / (365.0 * 366.0),
            Tolerance::Ulps(4),
        ),
        (
            DayCount::ThirtyE360,
            end.european() - start.european(),
            f64::from(end.european() - start.european()) / 360.0,
            Tolerance::Ulps(0),
        ),
        (
            DayCount::Thirty360BondBasis,
            bond_reference(start, end),
            f64::from(bond_reference(start, end)) / 360.0,
            Tolerance::Ulps(0),
        ),
    ] {
        assert_eq!(convention.day_count(start.date, end.date), count);
        assert_close(
            convention
                .year_fraction(start.date, end.date)
                .expect("bounded year fraction"),
            expected,
            tolerance,
        );
    }
}

#[test]
fn bond_basis_identifiers_are_exact_and_generic_labels_remain_ambiguous() {
    let basis = DayCount::Thirty360BondBasis;
    assert_eq!(basis.canonical_identifier(), "30/360 (Bond Basis)");
    assert_eq!(basis.to_string(), "30/360 (Bond Basis)");
    for text in ["30/360 (Bond Basis)", "30/360 ISDA", "360/360"] {
        assert_eq!(text.parse::<DayCount>(), Ok(basis));
    }
    for text in ["30/360", "Actual/Actual", "Actual/365"] {
        assert!(matches!(
            text.parse::<DayCount>(),
            Err(IdentifierError::Ambiguous { .. })
        ));
    }
    // Stateful conventions cannot silently acquire missing context from a name.
    for text in [
        "30E/360 (ISDA)",
        "30/360 German",
        "Business/252",
        "30/360 (US)",
        "bond basis",
        " 360/360",
        "360/360\n",
        "30/360 isda",
    ] {
        assert!(
            matches!(
                text.parse::<DayCount>(),
                Err(IdentifierError::Unknown { .. })
            ),
            "{text:?}"
        );
    }
    assert!(DayCount::ALL.contains(&basis));
    assert_eq!(DayCount::ALL.into_iter().collect::<HashSet<_>>().len(), 6);
}

#[test]
fn directional_bond_basis_rules_document_reversal_and_nonadditivity() {
    let basis = DayCount::Thirty360BondBasis;
    let january = date(2023, 1, 15);
    let march = date(2023, 3, 31);
    assert_eq!(basis.day_count(january, march), 76);
    assert_eq!(basis.day_count(march, january), -76);
    assert_eq!(
        direct_bond_reference(Civil::new(march), Civil::new(january)),
        -75
    );
    assert_close(
        basis.year_fraction(march, january).unwrap(),
        -76.0 / 360.0,
        Tolerance::Ulps(0),
    );
    let month_end = date(2023, 1, 31);
    let february = date(2023, 2, 15);
    assert_eq!(basis.day_count(january, month_end), 16);
    assert_eq!(basis.day_count(month_end, february), 15);
    assert_eq!(basis.day_count(january, february), 30);
    // A legitimate directional convention is not assumed additive.
    assert_ne!(
        basis.day_count(january, month_end) + basis.day_count(month_end, february),
        basis.day_count(january, february)
    );
}

#[test]
fn primary_variant_examples_and_required_termination_context_are_distinct() {
    for (start, end, bond, european, isda, terminating) in [
        (date(2024, 2, 29), date(2024, 3, 31), 32, 31, 30, 30),
        (date(2023, 1, 31), date(2023, 2, 28), 28, 28, 30, 28),
        (date(2023, 1, 15), date(2023, 3, 31), 76, 75, 75, 75),
        (date(2024, 1, 31), date(2024, 2, 29), 29, 29, 30, 29),
        (date(2023, 2, 28), date(2024, 2, 29), 361, 361, 360, 359),
    ] {
        assert_eq!(DayCount::Thirty360BondBasis.day_count(start, end), bond);
        assert_eq!(DayCount::ThirtyE360.day_count(start, end), european);
        let regular = ThirtyE360Isda::new(date(9999, 12, 31));
        let maturity = ThirtyE360Isda::new(end);
        assert_eq!(regular.day_count(start, end), isda);
        assert_eq!(maturity.day_count(start, end), terminating);
        assert_eq!(maturity.termination_date(), end);
        assert_eq!(maturity.canonical_identifier(), "30E/360 (ISDA)");
        assert_eq!(regular.day_count(end, start), -isda);
        assert_eq!(maturity.day_count(end, start), -terminating);
        assert_close(
            maturity.year_fraction(start, end).unwrap(),
            f64::from(terminating) / 360.0,
            Tolerance::Ulps(0),
        );
    }
}

#[test]
fn same_february_termination_endpoint_overrides_the_direct_formula_with_zero() {
    for year in [1, 4, 100, 400, 1900, 2000, 2100, 2400, 9996, 9999] {
        let end = date(year, 2, month_lengths(year)[1]);
        let terminating = ThirtyE360Isda::new(end);
        assert_eq!(terminating.day_count(end, end), 0);
        assert_eq!(terminating.year_fraction(end, end).unwrap().to_bits(), 0);
        // Applying the source's D1/D2 adjustments directly at this degenerate
        // termination interval gives -1 in leap years or -2 in common years.
        let direct = i32::try_from(end.day()).unwrap() - 30;
        assert_eq!(direct, if leap(year) { -1 } else { -2 });
    }
}

#[test]
fn every_supported_month_end_pair_matches_bond_and_isda_integer_models() {
    for year in 1..=9999 {
        let ends: Vec<_> = month_lengths(year)
            .into_iter()
            .enumerate()
            .map(|(index, length)| {
                Civil::new(date(year, u32::try_from(index + 1).unwrap(), length))
            })
            .collect();
        for &start in &ends {
            for &end in &ends {
                assert_eq!(
                    DayCount::Thirty360BondBasis.day_count(start.date, end.date),
                    bond_reference(start, end)
                );
                for termination in [start.date, end.date, date(9999, 1, 1)] {
                    let convention = ThirtyE360Isda::new(termination);
                    let expected = isda_thirty_reference(start, end, termination);
                    assert_eq!(convention.day_count(start.date, end.date), expected);
                    assert_close(
                        convention.year_fraction(start.date, end.date).unwrap(),
                        f64::from(expected) / 360.0,
                        Tolerance::Ulps(0),
                    );
                }
            }
        }
    }
}

#[test]
fn full_range_and_seeded_extreme_pairs_preserve_boundaries_and_fixed_context() {
    for termination in [Date::MIN, date(4, 2, 29), date(9999, 2, 28), Date::MAX] {
        let convention = ThirtyE360Isda::new(termination);
        for (start, end) in [
            (Date::MIN, Date::MAX),
            (Date::MAX, Date::MIN),
            (termination, termination),
        ] {
            let expected = isda_thirty_reference(Civil::new(start), Civil::new(end), termination);
            assert_eq!(convention.day_count(start, end), expected);
            assert_close(
                convention.year_fraction(start, end).unwrap(),
                f64::from(expected) / 360.0,
                Tolerance::Ulps(0),
            );
        }
    }
    prng::check(
        "extended day counts match independent civil formulas",
        0x4443_435F_4558_5445,
        30_000,
        |rng| {
            let start = Civil::new(
                Date::MIN
                    .checked_add_days(rng.range_i64(0, 3_652_058))
                    .unwrap(),
            );
            let end = Civil::new(
                Date::MIN
                    .checked_add_days(rng.range_i64(0, 3_652_058))
                    .unwrap(),
            );
            let termination = *rng.choose(&[
                start.date,
                end.date,
                Date::MIN,
                Date::MAX,
                date(2024, 2, 29),
            ]);
            let convention = ThirtyE360Isda::new(termination);
            let expected = isda_thirty_reference(start, end, termination);
            if convention.day_count(start.date, end.date) != expected
                || convention.day_count(end.date, start.date) != -expected
            {
                return Err(format!(
                    "30E/360ISDA {:?}->{:?}, term={termination}, expected={expected}",
                    start.date, end.date
                ));
            }
            Tolerance::Ulps(0).check(
                convention.year_fraction(start.date, end.date).unwrap(),
                f64::from(expected) / 360.0,
            )?;
            if DayCount::Thirty360BondBasis.day_count(start.date, end.date)
                != bond_reference(start, end)
            {
                return Err(format!("BondBasis {:?}->{:?}", start.date, end.date));
            }
            Ok(())
        },
    );
}

fn scalar_window(start: Date, end: Date) {
    let dates: Vec<_> = (0..=end.days_since(start))
        .map(|offset| {
            Civil::new(
                start
                    .checked_add_days(i64::from(offset))
                    .expect("window date"),
            )
        })
        .collect();
    for &start in &dates {
        for &end in &dates {
            check_scalar_pair(start, end);
        }
    }
}

#[test]
fn ts_dcc_07_every_ordered_date_pair_in_1999_through_2002_checks_all_five_scalars() {
    scalar_window(date(1999, 1, 1), date(2002, 12, 31));
}

#[test]
fn ts_dcc_07_every_ordered_date_pair_in_2099_through_2101_checks_all_five_scalars() {
    scalar_window(date(2099, 1, 1), date(2101, 12, 31));
}

fn mask_calendar(mask: u32) -> Calendar {
    Calendar::custom(
        Weekday::ALL
            .into_iter()
            .enumerate()
            .filter_map(|(index, weekday)| (mask & (1 << index) != 0).then_some(weekday)),
        [],
    )
}

#[test]
fn business_252_every_weekend_mask_and_override_pair_matches_scanned_flags() {
    let anchor = date(2024, 1, 1); // Independently known Monday.
    for mask in 0_u32..128 {
        let base = mask_calendar(mask);
        let with_overrides = base
            .with_holiday(anchor.checked_add_days(8).unwrap())
            .without_holiday(anchor.checked_add_days(12).unwrap());
        for (calendar, overrides) in [(base, false), (with_overrides, true)] {
            let value = Business252::new(calendar.clone());
            assert_eq!(value.calendar(), Some(&calendar));
            assert!(value.target_calendar().is_none());
            let flags: Vec<_> = (0..=28)
                .map(|offset| {
                    if overrides && offset == 8 {
                        false
                    } else if overrides && offset == 12 {
                        true
                    } else {
                        mask & (1 << (offset % 7)) == 0
                    }
                })
                .collect();
            for left in 0..=28 {
                for right in 0..=28 {
                    let start = anchor
                        .checked_add_days(i64::try_from(left).unwrap())
                        .unwrap();
                    let end = anchor
                        .checked_add_days(i64::try_from(right).unwrap())
                        .unwrap();
                    let count = i32::try_from(
                        flags[left.min(right)..left.max(right)]
                            .iter()
                            .filter(|&&business| business)
                            .count(),
                    )
                    .unwrap();
                    let expected = if left > right { -count } else { count };
                    assert_eq!(
                        value.day_count(start, end).unwrap(),
                        expected,
                        "mask={mask}, {left}->{right}, overrides={overrides}"
                    );
                    assert_close(
                        value.year_fraction(start, end).unwrap(),
                        f64::from(expected) / 252.0,
                        Tolerance::Ulps(0),
                    );
                }
            }
        }
    }
}

#[test]
fn business_252_half_open_and_reversed_endpoint_policy_is_explicit() {
    let value = Business252::new(Calendar::weekends_only());
    let monday = date(2024, 1, 1);
    let next_monday = date(2024, 1, 8);
    assert_eq!(value.day_count(monday, next_monday).unwrap(), 5);
    assert_eq!(value.day_count(next_monday, monday).unwrap(), -5);
    assert_close(
        value.year_fraction(monday, next_monday).unwrap(),
        5.0 / 252.0,
        Tolerance::Ulps(0),
    );
    let saturday = date(2024, 1, 6);
    assert_eq!(value.day_count(saturday, next_monday).unwrap(), 0);
    assert_eq!(value.day_count(next_monday, saturday).unwrap(), 0);
    assert_eq!(
        value
            .year_fraction(next_monday, saturday)
            .unwrap()
            .to_bits(),
        0
    );
    assert_eq!(
        Calendar::weekends_only().business_days_between(
            next_monday,
            saturday,
            EndpointInclusion::Include,
            EndpointInclusion::Exclude
        ),
        -1
    );
    for current in [monday, saturday, next_monday] {
        assert_eq!(value.day_count(current, current).unwrap(), 0);
        assert_eq!(value.year_fraction(current, current).unwrap().to_bits(), 0);
    }
}

#[test]
fn business_252_calendar_values_clone_hash_and_preserve_explicit_immutable_rules() {
    let original = Business252::new(Calendar::weekends_only());
    let copied = original.clone();
    assert_eq!(copied, original);
    assert_eq!(hash(&copied), hash(&original));
    let start = date(2024, 1, 1);
    let end = date(2024, 1, 8);
    let holiday = Business252::new(Calendar::weekends_only().with_holiday(start));
    assert_ne!(holiday, original);
    assert_eq!(original.day_count(start, end).unwrap(), 5);
    assert_eq!(holiday.day_count(start, end).unwrap(), 4);
    let jointly_closed = Calendar::joint(
        vec![
            Calendar::weekends_only(),
            Calendar::null().with_holiday(start),
        ],
        JointRule::JoinHolidays,
    )
    .unwrap();
    assert_eq!(
        Business252::new(jointly_closed)
            .day_count(start, end)
            .unwrap(),
        4
    );
    let jointly_open = Calendar::joint(
        vec![Calendar::weekends_only(), Calendar::null()],
        JointRule::JoinBusinessDays,
    )
    .unwrap();
    assert_eq!(
        Business252::new(jointly_open)
            .day_count(start, end)
            .unwrap(),
        7
    );
    let closed = Business252::new(Calendar::custom(Weekday::ALL, []));
    assert_eq!(closed.day_count(start, end).unwrap(), 0);
}

#[test]
fn business_252_full_supported_range_remains_exact_without_default_market_rules() {
    let days = Date::MAX.days_since(Date::MIN);
    // 0001-01-01 is Monday; the interval length excludes Date::MAX.
    let full_weeks = days / 7;
    let remainder = days % 7;
    let weekdays = full_weeks * 5 + remainder.min(5);
    for (calendar, expected) in [
        (Calendar::null(), days),
        (Calendar::weekends_only(), weekdays),
        (Calendar::custom(Weekday::ALL, []), 0),
    ] {
        let value = Business252::new(calendar);
        assert_eq!(value.day_count(Date::MIN, Date::MAX).unwrap(), expected);
        assert_eq!(value.day_count(Date::MAX, Date::MIN).unwrap(), -expected);
        assert_close(
            value.year_fraction(Date::MIN, Date::MAX).unwrap(),
            f64::from(expected) / 252.0,
            Tolerance::Ulps(0),
        );
    }
}

#[test]
fn wrapped_calendar_date_error_retains_the_complete_inspectable_source_chain() {
    let date_error = Date::MIN.checked_sub_days(1).unwrap_err();
    let calendar_error = CalendarError::from(date_error.clone());
    let domain = DayCountError::from(calendar_error.clone());
    assert_eq!(domain.kind(), "daycount.calendar");
    assert_eq!(domain.to_string(), calendar_error.to_string());
    assert_eq!(
        domain.source().unwrap().downcast_ref::<CalendarError>(),
        Some(&calendar_error)
    );
    assert_eq!(
        domain
            .source()
            .unwrap()
            .source()
            .unwrap()
            .downcast_ref::<DateError>(),
        Some(&date_error)
    );
    let outer = Error::from(domain.clone());
    assert_eq!(outer.kind(), "daycount.calendar");
    assert_eq!(
        outer.source().unwrap().downcast_ref::<DayCountError>(),
        Some(&domain)
    );
}

#[test]
fn business_252_guarded_target_preserves_errors_even_for_equal_and_excluded_endpoints() {
    let target = TargetCalendar::new();
    let convention = Business252::with_target(target.clone());
    assert!(convention.calendar().is_none());
    assert_eq!(convention.target_calendar(), Some(&target));
    let invalid = date(1999, 1, 3);
    let covered = date(1999, 1, 4);
    let gap = date(2003, 1, 1);
    for (start, end, expected_kind, rejected) in [
        (invalid, invalid, "calendar.outside_validity", invalid),
        (invalid, covered, "calendar.outside_validity", invalid),
        (covered, invalid, "calendar.outside_validity", invalid),
        (gap, gap, "calendar.unverified_coverage", gap),
        (date(2002, 12, 31), gap, "calendar.unverified_coverage", gap),
        (gap, date(2002, 12, 31), "calendar.unverified_coverage", gap),
        // Both endpoints are verified, but the counted interior crosses a
        // source-coverage gap. Valid endpoints do not license the interior.
        (
            date(2002, 12, 31),
            date(2026, 1, 1),
            "calendar.unverified_coverage",
            gap,
        ),
    ] {
        let error = convention.day_count(start, end).unwrap_err();
        assert_eq!(error.kind(), "daycount.calendar");
        assert_eq!(convention.year_fraction(start, end), Err(error.clone()));
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<CalendarError>()
            .unwrap();
        assert_eq!(source.kind(), expected_kind);
        match source {
            CalendarError::OutsideValidity { date, .. }
            | CalendarError::UnverifiedCoverage { date, .. } => assert_eq!(*date, rejected),
            _ => panic!("expected guarded coverage or validity context"),
        }
        assert_eq!(error.to_string(), source.to_string());
        assert_eq!(error.clone(), error);
    }
    for current in [covered, date(2000, 1, 1), date(2026, 1, 1)] {
        assert_eq!(convention.day_count(current, current), Ok(0));
        assert_eq!(
            convention
                .year_fraction(current, current)
                .unwrap()
                .to_bits(),
            0
        );
    }
}

#[test]
fn business_252_target_projection_is_explicit_and_cannot_extend_market_validity() {
    let strict = Business252::with_target(TargetCalendar::new());
    let projected_target = TargetCalendar::with_policy(TargetPolicy::AllowProjection);
    let projection = Business252::with_target(projected_target.clone());
    let gap_start = date(2003, 1, 6); // Monday, after New Year's holiday.
    let gap_end = date(2003, 1, 13);
    assert!(strict.day_count(gap_start, gap_end).is_err());
    assert_eq!(
        projected_target.coverage(gap_start),
        TargetCoverage::Projected
    );
    assert_eq!(projection.day_count(gap_start, gap_end), Ok(5));
    assert_eq!(projection.day_count(gap_end, gap_start), Ok(-5));
    assert_ne!(strict, projection);
    assert_eq!(projection.clone(), projection);
    assert_eq!(hash(&projection.clone()), hash(&projection));
    let prelaunch = date(1999, 1, 3);
    assert_eq!(
        projection
            .day_count(prelaunch, prelaunch)
            .unwrap_err()
            .source()
            .unwrap()
            .downcast_ref::<CalendarError>()
            .unwrap()
            .kind(),
        "calendar.outside_validity"
    );
    let holiday_override = projected_target.with_holiday(gap_start);
    assert_eq!(
        Business252::with_target(holiday_override).day_count(gap_start, gap_end),
        Ok(4)
    );
    let saturday = date(2003, 1, 11);
    let opened = projected_target.without_holiday(saturday);
    assert_eq!(
        Business252::with_target(opened).day_count(gap_start, gap_end),
        Ok(6)
    );
}

#[test]
fn business_252_target_overrides_and_joint_calendars_do_not_bypass_coverage() {
    let gap = date(2003, 1, 1);
    let strict = TargetCalendar::new();
    for target in [
        strict.with_holiday(gap),
        strict.without_holiday(gap),
        strict
            .joint_with(vec![Calendar::null()], JointRule::JoinBusinessDays)
            .unwrap(),
        strict
            .joint_with(
                vec![Calendar::custom(Weekday::ALL, [])],
                JointRule::JoinHolidays,
            )
            .unwrap(),
    ] {
        let value = Business252::with_target(target);
        assert_eq!(
            value
                .day_count(gap, gap)
                .unwrap_err()
                .source()
                .unwrap()
                .downcast_ref::<CalendarError>()
                .unwrap()
                .kind(),
            "calendar.unverified_coverage"
        );
    }
    // In the verified interval, good Friday and Easter Monday closures are
    // retained by the guarded calendar rather than inferred by the counter.
    let value = Business252::with_target(strict);
    assert_eq!(value.day_count(date(2026, 4, 2), date(2026, 4, 7)), Ok(1));
    assert_eq!(value.day_count(date(2026, 4, 7), date(2026, 4, 2)), Ok(-1));
}

#[test]
fn stateful_values_and_scalar_conventions_have_the_required_standard_traits() {
    fn copy<T: Copy + Debug + Eq + Hash + Send + Sync + 'static>() {}
    fn immutable<T: Clone + Debug + Eq + Hash + Send + Sync + 'static>() {}
    copy::<DayCount>();
    copy::<ThirtyE360Isda>();
    immutable::<Business252>();
    let february = date(2024, 2, 29);
    let value = ThirtyE360Isda::new(february);
    assert_eq!(value, ThirtyE360Isda::new(february));
    assert_eq!(hash(&value), hash(&ThirtyE360Isda::new(february)));
    assert_ne!(value, ThirtyE360Isda::new(date(2024, 3, 1)));
}
